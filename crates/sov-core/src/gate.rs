//! The gate: scores in, one verdict out. An errored verifier is a refusal (SPEC R8).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateResult {
    Pass,
    Fail,
    Refused,
}

#[derive(Debug, Clone)]
pub struct GateReport {
    pub result: GateResult,
    pub reason: Option<String>,
    pub score: Option<f64>,
    pub required: Vec<String>,
    pub min_score: BTreeMap<String, f64>,
    pub advisory: bool,
}

pub fn evaluate(domain: &Domain, results: &[RunResult]) -> GateReport {
    let required = domain.gate.require.clone();
    let min_score = domain.gate.min_score.clone();
    let mut advisory = false;

    let missing: Vec<&str> = required
        .iter()
        .filter(|id| !results.iter().any(|r| &r.verifier == *id))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        return GateReport {
            result: GateResult::Refused,
            reason: Some(format!("missing verifier: {}", missing.join(", "))),
            score: score_of(domain, results),
            required,
            min_score,
            advisory,
        };
    }

    let errored: Vec<&str> = results
        .iter()
        .filter(|r| required.contains(&r.verifier))
        .filter(|r| matches!(r.verdict, Verdict::Error | Verdict::Skip))
        .map(|r| r.verifier.as_str())
        .collect();
    if !errored.is_empty() {
        let reason = format!("verifier errored or skipped: {}", errored.join(", "));
        return match domain.gate.on_error {
            OnError::Fail => GateReport {
                result: GateResult::Refused,
                reason: Some(reason),
                score: score_of(domain, results),
                required,
                min_score,
                advisory,
            },
            OnError::Report => {
                advisory = true;
                GateReport {
                    result: GateResult::Fail,
                    reason: Some(reason),
                    score: score_of(domain, results),
                    required,
                    min_score,
                    advisory,
                }
            }
        };
    }

    let failed: Vec<&str> = results
        .iter()
        .filter(|r| required.contains(&r.verifier) && r.verdict == Verdict::Fail)
        .map(|r| r.verifier.as_str())
        .collect();
    if !failed.is_empty() {
        return GateReport {
            result: GateResult::Fail,
            reason: Some(format!("failed: {}", failed.join(", "))),
            score: score_of(domain, results),
            required,
            min_score,
            advisory,
        };
    }

    for (id, threshold) in &min_score {
        if let Some(result) = results.iter().find(|r| &r.verifier == id)
            && result.score < *threshold
        {
            return GateReport {
                result: GateResult::Fail,
                reason: Some(format!(
                    "{} scored {:.3}, below the gate minimum {:.3}",
                    id, result.score, threshold
                )),
                score: score_of(domain, results),
                required,
                min_score,
                advisory,
            };
        }
    }

    GateReport {
        result: GateResult::Pass,
        reason: None,
        score: score_of(domain, results),
        required,
        min_score,
        advisory,
    }
}

/// The domain's score is its declared primary, or the weakest gating numeric score.
fn score_of(domain: &Domain, results: &[RunResult]) -> Option<f64> {
    if let Some(primary) = &domain.primary_score
        && let Some(result) = results.iter().find(|r| &r.verifier == primary)
    {
        return Some(result.score);
    }
    results
        .iter()
        .filter(|r| domain.gate.require.contains(&r.verifier))
        .filter(|r| r.kind != VerifierKind::Exact)
        .map(|r| r.score)
        .fold(None, |acc: Option<f64>, s| {
            Some(acc.map_or(s, |a| a.min(s)))
        })
}
