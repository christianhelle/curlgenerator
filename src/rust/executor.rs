//! Driving asynchronous work from the synchronous command line entry point.

use std::future::Future;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

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
