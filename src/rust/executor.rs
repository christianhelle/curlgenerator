//! Running side effects that must never change the outcome of a run.

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Runs a side effect that must never change the outcome of a run.
///
/// Returns `false` when `task` panicked. Generation has already finished and its files are on
/// disk by the time reporting runs, so a reporting fault must not turn a successful run into a
/// failed one.
pub fn isolate(task: impl FnOnce()) -> bool {
    catch_unwind(AssertUnwindSafe(task)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_a_panicking_side_effect() {
        assert!(!isolate(|| panic!("reporting blew up")));
    }

    #[test]
    fn reports_that_a_successful_side_effect_completed() {
        let mut ran = false;
        assert!(isolate(|| ran = true));
        assert!(ran);
    }
}
