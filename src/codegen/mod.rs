//! See [`ACCUMULATOR_SUPPORT_LIST`] for the list of supported accumulators.
//!
//! For contributors, I recommend reading and playing with the mock implementation first, then
//! reading the rust implementation.

use super::*;

mod mock;
mod rust;
mod typescript;

/// Testing purposes only.
pub use mock::MockAccumulator;
/// Rust support using serde.
pub use rust::RustAccumulator;
/// Typescript support.
pub use typescript::TypescriptAccumulator;

/// The list of supported accumulators: `["typescript", "rust", "mock"]`.
pub const ACCUMULATOR_SUPPORT_LIST: &[&str] = &["typescript", "rust", "mock"];

/// Choose an accumulator from [`ACCUMULATOR_SUPPORT_LIST`]
pub fn accumulator_choose_with_str(s: &str) -> Option<Box<dyn TypeAccumulator>> {
    Some(match s {
        "typescript" => Box::new(TypescriptAccumulator::begin()),
        "rust" => Box::new(RustAccumulator::begin()),
        "mock" => Box::new(MockAccumulator::begin()),
        _ => None?,
    })
}
