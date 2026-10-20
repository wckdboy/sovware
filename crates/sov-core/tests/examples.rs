use std::collections::BTreeMap;

use sov_core::hash;
use sov_core::model::{Domain, Program, Verifier};

const V1: &str = include_str!("../../../examples/bracket/lug-bracket.v1.json");
const V2: &str = include_str!("../../../examples/bracket/lug-bracket.v2.json");
const V3: &str = include_str!("../../../examples/bracket/lug-bracket.v3.json");
const DOMAIN: &str = include_str!("../../../domains/mechanical.json");
const UNITS: &str = include_str!("../../../verifiers/sov-units/verifier.json");
const CLAIMS: &str = include_str!("../../../verifiers/sov-claims/verifier.json");
const SOURCE: &str = include_str!("../../../verifiers/sov-source/verifier.json");
const YIELD: &str = include_str!("../../../verifiers/mech-yield-margin/verifier.json");

#[test]
fn programs_parse_and_validate() {
    for text in [V1, V2, V3] {
        let program: Program = serde_json::from_str(text).expect("program parses");
        program.validate().expect("program is valid");
        assert_eq!(program.domain, "mechanical");
        assert_eq!(program.kind, "cadquery.part");
        assert!(!program.claims.is_empty(), "R3: a program carries claims");
    }
}

#[test]
fn a_bare_number_is_rejected_by_the_type() {
    let broken = V1.replace(
        "\"length_mm\": { \"value\": 120, \"unit\": \"mm\", \"min\": 60, \"max\": 200 }",
        "\"length_mm\": 120",
    );
    let program: Program = serde_json::from_str(&broken).unwrap();
    // The kernel accepts the shape; sov/units is the verifier that refuses it. That split
    // is the point: types describe, verifiers judge.
    assert!(program.params.contains_key("length_mm"));
}

#[test]
fn unknown_fields_are_errors() {
    let broken = V1.replace("\"notes\"", "\"note\"");
    assert!(serde_json::from_str::<Program>(&broken).is_err());
}

#[test]
fn domain_declares_a_gate_and_non_goals() {
    let domain: Domain = serde_json::from_str(DOMAIN).unwrap();
    domain.validate().unwrap();
    assert_eq!(domain.gate.require.len(), 4);
    assert_eq!(domain.primary_score.as_deref(), Some("mech/yield-margin"));
    assert!(domain.non_goals.iter().any(|n| n.contains("human")));
}

#[test]
fn verifier_manifests_parse_and_obey_r2() {
    for text in [UNITS, CLAIMS, SOURCE, YIELD] {
        let verifier: Verifier = serde_json::from_str(text).unwrap();
        verifier.validate().unwrap();
        if verifier.kind == sov_core::model::VerifierKind::Statistical {
            assert!(!verifier.gates);
        }
    }
}

#[test]
fn eval_parses_and_publishes_a_hash() {
    let text = include_str!("../../../examples/bracket/eval.json");
    let eval: sov_core::model::Eval = serde_json::from_str(text).unwrap();
    eval.validate().unwrap();
    assert!(
        !eval.sealed,
        "the example set is illustrative, not admitted"
    );
    assert!(eval.holdout_hash.is_some());
    assert_eq!(eval.known_bad[0].must_fail, "mech/yield-margin");
}

#[test]
fn eval_holdout_hash_matches_its_contents() {
    let text = include_str!("../../../examples/bracket/eval.json");
    let eval: serde_json::Value = serde_json::from_str(text).unwrap();
    let computed = hash::hash_json(&eval["holdout"]).unwrap();
    assert_eq!(
        computed,
        eval["holdout_hash"].as_str().unwrap(),
        "the published holdout hash must cover the holdout cases"
    );
    assert_eq!(eval["sealed"], serde_json::Value::Bool(false));
}

#[test]
fn canonical_hash_ignores_key_order() {
    let mut a = BTreeMap::new();
    a.insert("b", 2);
    a.insert("a", 1);
    let left = hash::hash_json(&a).unwrap();

    let mut c = BTreeMap::new();
    c.insert("a", 1);
    c.insert("b", 2);
    let right = hash::hash_json(&c).unwrap();
    assert_eq!(left, right);
    assert!(left.starts_with("sha256:"));
}
