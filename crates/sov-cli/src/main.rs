//! `sov` — the reference kernel.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sov_core::Result;
use sov_core::gate::{self, GateResult};
use sov_core::hash;
use sov_core::ledger;
use sov_core::model::*;
use sov_core::verifier::{self, run_verifier};

#[derive(Parser)]
#[command(
    name = "sov",
    version,
    about = "the sovware kernel: score a program, gate it, record it"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a program through its domain gate and append the run to the ledger.
    Verify(Verify),
    /// Compare two ledger runs under the improvement protocol (SPEC 7).
    Compare(Compare),
    /// Ledger operations.
    Ledger {
        #[command(subcommand)]
        command: LedgerCommand,
    },
    /// Print a file hash in ledger form.
    Hash { path: PathBuf },
}

#[derive(Parser)]
struct Verify {
    /// Program JSON.
    program: PathBuf,
    /// Directory holding verifier manifests.
    #[arg(short, long, default_value = "verifiers")]
    verifiers: PathBuf,
    /// Domain gate.
    #[arg(short, long, default_value = "domains/mechanical.json")]
    domain: PathBuf,
    /// Append-only ledger.
    #[arg(long, default_value = "runs.jsonl")]
    ledger: PathBuf,
    /// human:<id> or agent:<id>.
    #[arg(long, default_value = "agent:sov")]
    actor: String,
    /// Print the run object instead of the terminal report.
    #[arg(long)]
    json: bool,
    /// Refuse unless the actor is human.
    #[arg(long)]
    require_signature: bool,
    /// Wall-clock budget in seconds; a run over it fails and is still recorded.
    #[arg(long)]
    budget_wall_s: Option<f64>,
}

#[derive(Parser)]
struct Compare {
    parent: String,
    child: String,
    #[arg(long, default_value = "runs.jsonl")]
    ledger: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum LedgerCommand {
    /// Recompute every hash and link in a ledger.
    Verify {
        #[arg(default_value = "runs.jsonl")]
        path: PathBuf,
    },
    /// List runs, newest last.
    List {
        #[arg(default_value = "runs.jsonl")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match execute(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("✗ {err}");
            2
        }
    };
    std::process::exit(code);
}

fn execute(cli: Cli) -> Result<i32> {
    match cli.command {
        Command::Verify(args) => verify(args),
        Command::Compare(args) => compare(args),
        Command::Ledger { command } => match command {
            LedgerCommand::Verify { path } => {
                let report = ledger::verify_chain(&path)?;
                if report.ok {
                    println!(
                        "✓ ledger ok · {} runs · last {}",
                        report.runs,
                        short(&report.last)
                    );
                    Ok(0)
                } else {
                    println!("✗ ledger broken · {}", report.problem.unwrap_or_default());
                    Ok(1)
                }
            }
            LedgerCommand::List { path } => {
                for run in ledger::read_runs(&path)? {
                    println!(
                        "{}  {:<10} {:<28} score {:<5} {}",
                        run.id,
                        format!("{:?}", run.gate.result).to_lowercase(),
                        run.program,
                        run.score
                            .map(|s| format!("{s:.3}"))
                            .unwrap_or_else(|| "—".into()),
                        run.actor,
                    );
                }
                Ok(0)
            }
        },
        Command::Hash { path } => {
            println!("{}", hash::hash_file(&path)?);
            Ok(0)
        }
    }
}

fn verify(args: Verify) -> Result<i32> {
    let bytes = std::fs::read(&args.program)?;
    let program: Program = serde_json::from_slice(&bytes)?;
    program.validate()?;

    let domain: Domain = serde_json::from_str(&std::fs::read_to_string(&args.domain)?)?;
    domain.validate()?;

    if !domain.artifacts.iter().any(|k| k == &program.kind) {
        return Err(format!(
            "domain {} does not admit artifact kind {}",
            domain.id, program.kind
        )
        .into());
    }

    if args.require_signature && !args.actor.starts_with("human:") {
        return Err("--require-signature needs --actor human:<id> (SPEC R9)".into());
    }
    if program
        .fabrication
        .as_ref()
        .map(|f| f.ready)
        .unwrap_or(false)
        && !args.actor.starts_with("human:")
    {
        return Err(format!(
            "{} is marked fabrication-ready; the actor must be human (SPEC R9)",
            program.id
        )
        .into());
    }

    let entries = verifier::discover(std::slice::from_ref(&args.verifiers))?;
    let mut selected = Vec::new();
    for id in &domain.gate.require {
        match entries.iter().find(|e| &e.verifier.id == id) {
            Some(entry) if entry.verifier.applies(&program.domain, &program.kind) => {
                selected.push(entry.clone())
            }
            Some(_) => {
                return Err(format!(
                    "{id} does not apply to {} / {}",
                    program.domain, program.kind
                )
                .into());
            }
            None => return Err(format!("no verifier with id {id}").into()),
        }
    }

    let mut results = Vec::new();
    let mut wall_ms = 0.0_f64;
    let mut mass_g = None;
    let mut rendered = Vec::new();

    for entry in &selected {
        let outcome = run_verifier(entry, &program, &args.program, &bytes)?;
        wall_ms += outcome.wall_ms;
        if let Some(mass) = outcome
            .result
            .measures
            .get("mass_g")
            .and_then(|v| v.as_f64())
        {
            mass_g = Some(mass);
        }
        results.push(RunResult {
            verifier: entry.verifier.id.clone(),
            version: entry.verifier.version.clone(),
            kind: entry.verifier.kind,
            verdict: outcome.result.verdict,
            score: outcome.result.score,
            hash: entry.definition_hash.clone(),
            findings: outcome.result.errors(),
        });
        rendered.push((entry.verifier.id.clone(), outcome));
    }

    let report = gate::evaluate(&domain, &results);
    let mut result = report.result;
    let mut reason = report.reason.clone();

    if let Some(limit) = args.budget_wall_s
        && wall_ms / 1000.0 > limit
    {
        result = GateResult::Fail;
        reason = Some(format!(
            "wall time {:.2}s over budget {limit:.2}s",
            wall_ms / 1000.0
        ));
    }
    if let (Some(limit), Some(actual)) = (program.budget.as_ref().and_then(|b| b.mass_g), mass_g)
        && actual > limit
    {
        result = GateResult::Fail;
        reason = Some(format!("mass {actual:.1}g over budget {limit:.1}g"));
    }

    let created = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let program_hash = hash::hash_json(&program)?;
    let prev = ledger::last_hash(&args.ledger)?;

    let mut run = Run {
        schema: RUN_SCHEMA.to_string(),
        id: run_id(&program_hash, &created, &args.actor),
        program: format!(
            "{}@{}",
            program.id,
            program
                .version
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".into())
        ),
        program_hash,
        domain: domain.id.clone(),
        gate: RunGate {
            result,
            required: report.required.clone(),
            min_score: report.min_score.clone(),
            reason,
        },
        results,
        score: report.score,
        cost: RunCost {
            wall_ms: Some((wall_ms * 100.0).round() / 100.0),
            usd: None,
            mass_g,
        },
        actor: args.actor.clone(),
        created,
        prev,
        hash: String::new(),
        advisory: report.advisory,
    };
    run.seal()?;
    ledger::append(&args.ledger, &run)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&run)?);
    } else {
        print_report(&program, &domain, &rendered, &run);
    }

    Ok(match result {
        GateResult::Pass => 0,
        _ => 1,
    })
}

fn print_report(
    program: &Program,
    domain: &Domain,
    rendered: &[(String, verifier::ExecOutcome)],
    run: &Run,
) {
    println!(
        "▪ {} @{} · {} · {}",
        program.id,
        program.version.unwrap_or(0),
        domain.id,
        program.kind
    );
    for (_, outcome) in rendered {
        let mark = match outcome.result.verdict {
            Verdict::Pass => "✓",
            Verdict::Fail => "✗",
            Verdict::Error => "!",
            Verdict::Skip => "-",
        };
        let conflict = if outcome.result.verdict == Verdict::Error {
            " (protocol)"
        } else {
            ""
        };
        println!(
            "  {mark} {:<20} {:>5.2}s  score {:.3}{conflict}",
            outcome.result.verifier,
            outcome.wall_ms / 1000.0,
            outcome.result.score
        );
        if let Some(finding) = outcome
            .result
            .findings
            .iter()
            .find(|f| f.level == FindingLevel::Error)
        {
            println!("     └ {} {}", finding.code, finding.message);
            if let Some(hint) = &finding.hint {
                println!("       {hint}");
            }
        }
        if !outcome.stderr.trim().is_empty() {
            for line in outcome.stderr.lines().take(3) {
                println!("     ⚠ {line}");
            }
        }
    }

    let mark = match run.gate.result {
        GateResult::Pass => "✓",
        GateResult::Fail => "✗",
        GateResult::Refused => "!",
    };
    let score = run
        .score
        .map(|s| format!(" · score {s:.3}"))
        .unwrap_or_default();
    let mass = run
        .cost
        .mass_g
        .map(|m| format!(" · {m:.1}g"))
        .unwrap_or_default();
    let wall = run.cost.wall_ms.map(|w| w / 1000.0).unwrap_or_default();
    let why = run
        .gate
        .reason
        .as_ref()
        .map(|r| format!(" · {r}"))
        .unwrap_or_default();
    println!(
        "{mark} gate {:?}{score}{mass} · {wall:.2}s{why}",
        run.gate.result
    );
    if run.advisory {
        println!("  ! advisory run: not usable for fabrication");
    }
    println!("  {} → ledger", run.id);
}

fn compare(args: Compare) -> Result<i32> {
    let runs = ledger::read_runs(&args.ledger)?;
    let find = |id: &str| {
        runs.iter()
            .find(|r| r.id == id || r.program == id)
            .cloned()
            .ok_or_else(|| format!("no run {id} in {}", args.ledger.display()))
    };
    let parent = find(&args.parent)?;
    let child = find(&args.child)?;

    let mut conditions: BTreeMap<&str, bool> = BTreeMap::new();
    conditions.insert("gate", child.gate.result == GateResult::Pass);
    let parent_score = parent.score.unwrap_or(0.0);
    let child_score = child.score.unwrap_or(0.0);
    conditions.insert("score", child_score >= parent_score);
    let cost = cost_axis(&child, &parent);
    conditions.insert("cost", cost.as_ref().map(|(_, ok)| *ok).unwrap_or(true));

    let verdict = if !conditions["gate"] {
        "change"
    } else if !conditions["score"] {
        "regression"
    } else if !conditions["cost"] {
        "change"
    } else {
        "improvement"
    };

    if args.json {
        let out = serde_json::json!({
            "parent": parent.id,
            "child": child.id,
            "verdict": verdict,
            "conditions": conditions,
            "parent_score": parent_score,
            "child_score": child_score,
            "cost": cost.map(|(c, _)| c),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("▪ {} → {}", parent.id, child.id);
        println!(
            "  {} gate      parent {:?} · child {:?}",
            mark(conditions["gate"]),
            parent.gate.result,
            child.gate.result
        );
        println!(
            "  {} score     {parent_score:.3} → {child_score:.3}",
            mark(conditions["score"])
        );
        if let Some((note, ok)) = &cost {
            println!("  {} cost      {note}", mark(*ok));
        }
        println!("  {verdict}");
        if verdict != "improvement" {
            println!("  not an improvement. recorded either way (SPEC R11).");
        }
    }
    Ok(0)
}

fn cost_axis(child: &Run, parent: &Run) -> Option<(String, bool)> {
    if let (Some(c), Some(p)) = (child.cost.mass_g, parent.cost.mass_g) {
        return Some((format!("mass {p:.1}g → {c:.1}g"), c <= p));
    }
    if let (Some(c), Some(p)) = (child.cost.wall_ms, parent.cost.wall_ms) {
        return Some((
            format!("wall {:.2}s → {:.2}s", p / 1000.0, c / 1000.0),
            c <= p,
        ));
    }
    None
}

fn mark(ok: bool) -> &'static str {
    if ok { "✓" } else { "✗" }
}

fn short(hash: &Option<String>) -> String {
    hash.as_ref()
        .map(|h| h.chars().take(12).collect::<String>())
        .unwrap_or_else(|| "—".into())
}

fn run_id(program_hash: &str, created: &str, actor: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let seed = format!("{program_hash}{created}{actor}{nanos}");
    let digest = hash::sha256_tag(seed.as_bytes());
    let hex = digest.trim_start_matches("sha256:");
    format!("run_{}", &hex[..12])
}
