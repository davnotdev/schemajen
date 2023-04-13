//! See [`ACCUMULATOR_SUPPORT_LIST`] for the list of supported accumulators.
//!
//! For contributors, I recommend reading and playing with the mock implementation first, then
//! reading the rust implementation.

use super::*;

mod dart;
mod java;
mod mock;
mod python;
mod rust;
mod typescript;

/// Dart support.
pub use dart::DartAccumulator;
/// Java Serializable support.
pub use java::JavaAccumulator;
/// Testing purposes only.
pub use mock::MockAccumulator;
/// Python 3.8+ support.
pub use python::PythonAccumulator;
/// Rust support using serde.
pub use rust::RustAccumulator;
/// Typescript support.
pub use typescript::TypescriptAccumulator;

/// The list of supported accumulators: `["typescript", "python", "dart", "rust", "java", "mock"]`.
pub const ACCUMULATOR_SUPPORT_LIST: &[&str] =
    &["typescript", "python", "dart", "rust", "java", "mock"];

/// Choose an accumulator from [`ACCUMULATOR_SUPPORT_LIST`]
pub fn accumulator_choose_with_str(s: &str) -> Option<Box<dyn TypeAccumulator>> {
    Some(match s {
        "typescript" => Box::new(TypescriptAccumulator::begin()),
        "rust" => Box::new(RustAccumulator::begin()),
        "java" => Box::new(JavaAccumulator::begin()),
        "dart" => Box::new(DartAccumulator::begin()),
        "python" => Box::new(PythonAccumulator::begin()),
        "mock" => Box::new(MockAccumulator::begin()),
        _ => None?,
    })
}
