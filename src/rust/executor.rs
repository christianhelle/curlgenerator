//! Driving asynchronous work from the synchronous command line entry point.

use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
};

use tokio::runtime::Runtime;

/// Blocks the current thread until `future` completes.
///
/// The future is driven on a Tokio runtime with the I/O and timer drivers enabled. The HTTP
/// clients behind telemetry reporting and Azure Entra ID authentication panic without one.
pub fn block_on<F: Future>(future: F) -> F::Output {
    Runtime::new()
        .expect("the Tokio runtime should start")
        .block_on(future)
}

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
    use std::time::{Duration, Instant};

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

    #[test]
    fn drives_futures_that_depend_on_the_tokio_reactor() {
        let elapsed = block_on(async {
            let started = Instant::now();
            tokio::time::sleep(Duration::from_millis(10)).await;

            started.elapsed()
        });

        assert!(elapsed >= Duration::from_millis(10));
    }
}
