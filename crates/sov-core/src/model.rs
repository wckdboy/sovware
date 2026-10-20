//! Types mirroring `schema/`. Deserialization is strict: an unknown field is an error.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gate::GateResult;

pub const PROGRAM_SCHEMA: &str = "sovware.dev/program/v1";
pub const VERIFIER_SCHEMA: &str = "sovware.dev/verifier/v1";
pub const RESULT_SCHEMA: &str = "sovware.dev/verifier-result/v1";
pub const DOMAIN_SCHEMA: &str = "sovware.dev/domain/v1";
pub const RUN_SCHEMA: &str = "sovware.dev/run/v1";
pub const EVAL_SCHEMA: &str = "sovware.dev/eval/v1";

// ---------------------------------------------------------------------------
// program
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub schema: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub domain: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub source: Source,
    pub params: BTreeMap<String, Param>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub derived: Vec<Derived>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabrication: Option<Fabrication>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Program {
    pub fn validate(&self) -> crate::Result<()> {
        if self.schema != PROGRAM_SCHEMA {
            return Err(format!("schema must be {PROGRAM_SCHEMA}, got {}", self.schema).into());
        }
        if !self.id.contains('/') {
            return Err(format!("program id {} must be namespaced domain/name", self.id).into());
        }
        if self.params.is_empty() {
            return Err("program declares no params".into());
        }
        Ok(())
    }

    /// The mechanical mass the gate compares against, if the program declares one.
    pub fn declared_mass_g(&self) -> Option<f64> {
        self.budget.as_ref().and_then(|b| b.mass_g)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub sha256: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Param {
    Detailed(ParamObject),
    Literal(Value),
}

impl Param {
    pub fn value(&self) -> &Value {
        match self {
            Param::Detailed(p) => &p.value,
            Param::Literal(v) => v,
        }
    }

    pub fn unit(&self) -> Option<&str> {
        match self {
            Param::Detailed(p) => p.unit.as_deref(),
            Param::Literal(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParamObject {
    pub value: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub source: ClaimSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimSource {
    pub kind: ClaimKind,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClaimKind {
    Standard,
    Paper,
    Measurement,
    Test,
    Derivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Derived {
    pub path: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fabrication {
    #[serde(default)]
    pub ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<String>,
}

// ---------------------------------------------------------------------------
// verifier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verifier {
    pub schema: String,
    pub id: String,
    pub version: String,
    pub kind: VerifierKind,
    pub gates: bool,
    pub applies_to: AppliesTo,
    pub invoke: Invoke,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub executes_source: bool,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Verifier {
    pub fn validate(&self) -> crate::Result<()> {
        if self.schema != VERIFIER_SCHEMA {
            return Err(format!("schema must be {VERIFIER_SCHEMA}, got {}", self.schema).into());
        }
        if self.kind == VerifierKind::Statistical && self.gates {
            return Err(format!("{} is statistical and must not gate (R2)", self.id).into());
        }
        if self.invoke.cmd.is_empty() {
            return Err(format!("{} has no command", self.id).into());
        }
        Ok(())
    }

    pub fn applies(&self, domain: &str, kind: &str) -> bool {
        let d = self
            .applies_to
            .domains
            .iter()
            .any(|x| x == "*" || x == domain);
        let k = self.applies_to.kinds.iter().any(|x| x == "*" || x == kind);
        d && k
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifierKind {
    Exact,
    Numeric,
    Statistical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliesTo {
    pub domains: Vec<String>,
    pub kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invoke {
    pub cmd: Vec<String>,
    pub input: InputMode,
    pub timeout_s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    Stdin,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// verifier result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierResult {
    pub schema: String,
    pub verifier: String,
    pub version: String,
    pub kind: VerifierKind,
    pub verdict: Verdict,
    pub score: f64,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default)]
    pub measures: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<ResultCost>,
}

impl VerifierResult {
    pub fn clamp_score(&mut self) {
        self.score = self.score.clamp(0.0, 1.0);
    }

    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.level == FindingLevel::Error)
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Fail,
    Error,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub level: FindingLevel,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultCost {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
}

// ---------------------------------------------------------------------------
// domain
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    pub artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_score: Option<String>,
    pub gate: GateSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    #[serde(default)]
    pub sources: Vec<ClaimSource>,
    pub non_goals: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Domain {
    pub fn validate(&self) -> crate::Result<()> {
        if self.schema != DOMAIN_SCHEMA {
            return Err(format!("schema must be {DOMAIN_SCHEMA}, got {}", self.schema).into());
        }
        if self.gate.require.is_empty() {
            return Err(format!("domain {} has an empty gate", self.id).into());
        }
        if self.non_goals.is_empty() {
            return Err(format!("domain {} has no non-goals (R5)", self.id).into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateSpec {
    pub require: Vec<String>,
    #[serde(default)]
    pub min_score: BTreeMap<String, f64>,
    pub on_error: OnError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    Fail,
    Report,
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub schema: String,
    pub id: String,
    pub program: String,
    pub program_hash: String,
    pub domain: String,
    pub gate: RunGate,
    pub results: Vec<RunResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default)]
    pub cost: RunCost,
    pub actor: String,
    pub created: String,
    pub prev: Option<String>,
    pub hash: String,
    #[serde(default)]
    pub advisory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunGate {
    pub result: GateResult,
    pub required: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub min_score: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResult {
    pub verifier: String,
    pub version: String,
    pub kind: VerifierKind,
    pub verdict: Verdict,
    pub score: f64,
    pub hash: String,
    #[serde(default)]
    pub findings: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCost {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_g: Option<f64>,
}

// ---------------------------------------------------------------------------
// eval
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eval {
    pub schema: String,
    pub id: String,
    pub domain: String,
    pub version: String,
    #[serde(default)]
    pub train: Vec<EvalCase>,
    pub holdout: Vec<EvalCase>,
    pub known_bad: Vec<BadCase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holdout_hash: Option<String>,
    pub sealed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Eval {
    pub fn validate(&self) -> crate::Result<()> {
        if self.schema != EVAL_SCHEMA {
            return Err(format!("schema must be {EVAL_SCHEMA}, got {}", self.schema).into());
        }
        if self.holdout.is_empty() {
            return Err("eval declares no holdout cases (SPEC 7)".into());
        }
        if self.sealed && self.holdout_hash.is_none() {
            return Err("a sealed eval must publish holdout_hash (SPEC 7)".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalCase {
    pub program: String,
    pub expect: CaseExpect,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseExpect {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BadCase {
    pub program: String,
    pub must_fail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defect: Option<String>,
}
