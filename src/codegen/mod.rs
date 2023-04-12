use super::*;

mod mock;
mod rust;

pub use mock::MockAccumulator;
pub use rust::RustAccumulator;

pub const ACCUMULATOR_SUPPORT_LIST: &[&str] = &["rust", "mock"];
pub fn accumulator_choose_with_str(s: &str) -> Option<Box<dyn TypeAccumulator>> {
    Some(match s {
        "rust" => Box::new(RustAccumulator::begin()),
        "mock" => Box::new(MockAccumulator::begin()),
        _ => None?,
    })
}
