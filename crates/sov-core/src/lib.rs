//! The sovware kernel.
//!
//! Four objects, one loop: a program is scored by verifiers, a gate turns scores into a
//! verdict, and the run is appended to a hash-chained ledger. Nothing else belongs here.

pub mod gate;
pub mod hash;
pub mod ledger;
pub mod model;
pub mod verifier;

pub use gate::{GateReport, GateResult};
pub use model::*;
pub use verifier::{ExecOutcome, VerifierEntry};

/// The kernel's error type. Strings are enough: every error here is read by a human or
/// written to a ledger, never matched on.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
