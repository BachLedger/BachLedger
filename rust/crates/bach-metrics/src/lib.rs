//! # bach-metrics
//!
//! Observability and metrics collection for BachLedger.
//!
//! Features:
//! - Histogram for latency tracking
//! - Counter for event counting
//! - Gauge for current values
//! - JSON export
//! - CLI dashboard

#![warn(missing_docs)]
#![warn(clippy::all)]

mod histogram;
mod collector;
mod export;

pub use histogram::Histogram;
pub use collector::{Metrics, MetricsCollector};
pub use export::MetricsSnapshot;

/// Macro for timing a block of code
#[macro_export]
macro_rules! timed {
    ($metrics:expr, $name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        $metrics.histogram($name, start.elapsed().as_micros() as f64);
        result
    }};
}
