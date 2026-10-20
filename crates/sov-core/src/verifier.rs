//! Verifier discovery and execution. Every verifier is a subprocess (SPEC R6).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::Result;
use crate::hash;
use crate::model::*;

#[derive(Debug, Clone)]
pub struct VerifierEntry {
    pub verifier: Verifier,
    pub dir: PathBuf,
    /// Hash of the verifier definition, recorded in the ledger so a score can never be
    /// detached from the code that produced it.
    pub definition_hash: String,
}

#[derive(Debug, Clone)]
pub struct ExecOutcome {
    pub result: VerifierResult,
    pub wall_ms: f64,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Load every `verifier.json` under the given directories, sorted by id.
pub fn discover(dirs: &[PathBuf]) -> Result<Vec<VerifierEntry>> {
    let mut found = Vec::new();
    for dir in dirs {
        walk(dir, 0, &mut found)?;
    }
    for entry in &mut found {
        if let Ok(absolute) = std::fs::canonicalize(&entry.dir) {
            entry.dir = absolute;
        }
    }
    found.sort_by(|a, b| a.verifier.id.cmp(&b.verifier.id));
    Ok(found)
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<VerifierEntry>) -> Result<()> {
    if !dir.exists() {
        return Err(format!("verifier directory {} does not exist", dir.display()).into());
    }
    let manifest = dir.join("verifier.json");
    if manifest.is_file() {
        let text = std::fs::read_to_string(&manifest)?;
        let verifier: Verifier =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", manifest.display()))?;
        verifier.validate()?;
        out.push(VerifierEntry {
            definition_hash: hash::hash_json(&verifier)?,
            dir: dir.to_path_buf(),
            verifier,
        });
        return Ok(());
    }
    if depth >= 3 {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            walk(&entry.path(), depth + 1, out)?;
        }
    }
    Ok(())
}

pub fn run_verifier(
    entry: &VerifierEntry,
    program: &Program,
    program_path: &Path,
    program_bytes: &[u8],
) -> Result<ExecOutcome> {
    let v = &entry.verifier;
    let workdir = match &v.invoke.workdir {
        Some(w) => entry.dir.join(w),
        None => entry.dir.clone(),
    };
    let program_path =
        std::fs::canonicalize(program_path).unwrap_or_else(|_| program_path.to_path_buf());
    let program_dir = program_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut cmd = Command::new(&v.invoke.cmd[0]);
    cmd.args(&v.invoke.cmd[1..])
        .current_dir(&workdir)
        .env("SOV_PROGRAM_DIR", &program_dir)
        .env("SOV_PROGRAM_PATH", &program_path)
        .env("SOV_VERIFIER_DIR", &entry.dir)
        .env("SOV_PROGRAM_ID", &program.id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if matches!(v.invoke.input, InputMode::Path) {
        cmd.arg(program_path);
    }
    for (key, value) in &v.invoke.env {
        cmd.env(key, value);
    }

    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("could not start {}: {e}", v.id))?;

    if matches!(v.invoke.input, InputMode::Stdin)
        && let Some(mut stdin) = child.stdin.take()
    {
        stdin
            .write_all(program_bytes)
            .map_err(|e| format!("could not write program to {}: {e}", v.id))?;
    }

    let deadline = Duration::from_secs_f64(v.invoke.timeout_s);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            let wall_ms = start.elapsed().as_secs_f64() * 1000.0;
            return Ok(ExecOutcome {
                result: error_result(
                    v,
                    "SOV-VERIFIER-TIMEOUT",
                    format!("{} exceeded {}s", v.id, v.invoke.timeout_s),
                ),
                wall_ms,
                stderr: String::new(),
                exit_code: None,
            });
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        use std::io::Read;
        let _ = out.read_to_string(&mut stdout);
    }
    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        use std::io::Read;
        let _ = err.read_to_string(&mut stderr);
    }
    let wall_ms = start.elapsed().as_secs_f64() * 1000.0;

    let result = match serde_json::from_str::<VerifierResult>(&stdout) {
        Ok(mut result) => {
            result.clamp_score();
            if result.verifier != v.id {
                error_result(
                    v,
                    "SOV-VERIFIER-IDENTITY",
                    format!("{} returned a result for {}", v.id, result.verifier),
                )
            } else {
                result
            }
        }
        Err(e) => error_result(
            v,
            "SOV-VERIFIER-PROTOCOL",
            format!("{} did not return one valid result object: {e}", v.id),
        ),
    };

    Ok(ExecOutcome {
        result,
        wall_ms,
        stderr,
        exit_code: status.code(),
    })
}

pub fn error_result(v: &Verifier, code: &str, message: String) -> VerifierResult {
    VerifierResult {
        schema: RESULT_SCHEMA.to_string(),
        verifier: v.id.clone(),
        version: v.version.clone(),
        kind: v.kind,
        verdict: Verdict::Error,
        score: 0.0,
        findings: vec![Finding {
            level: FindingLevel::Error,
            code: code.to_string(),
            message,
            path: None,
            hint: None,
        }],
        measures: Default::default(),
        cost: None,
    }
}
