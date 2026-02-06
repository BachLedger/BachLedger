//! Metrics export and snapshot functionality

use crate::Metrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Snapshot of all metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Counter values
    pub counters: HashMap<String, u64>,
    /// Gauge values
    pub gauges: HashMap<String, i64>,
    /// Histogram summaries (mean, count)
    pub histograms: HashMap<String, HistogramSummary>,
}

/// Summary of a histogram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSummary {
    /// Mean value
    pub mean: f64,
    /// Total observation count
    pub count: u64,
}

impl MetricsSnapshot {
    /// Create a snapshot from a Metrics instance
    pub fn from_metrics(metrics: &Metrics) -> Self {
        let counters = metrics.all_counters().into_iter().collect();
        let gauges = metrics.all_gauges().into_iter().collect();
        let histograms = metrics
            .all_histograms()
            .into_iter()
            .map(|(name, mean, count)| (name, HistogramSummary { mean, count }))
            .collect();

        Self {
            counters,
            gauges,
            histograms,
        }
    }

    /// Export snapshot as JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export snapshot as compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_json() {
        let metrics = Metrics::new();
        metrics.counter("requests", 100);
        metrics.gauge("connections", 5);
        metrics.histogram("latency", 50.0);

        let snapshot = MetricsSnapshot::from_metrics(&metrics);
        let json = snapshot.to_json().unwrap();

        assert!(json.contains("requests"));
        assert!(json.contains("100"));
        assert!(json.contains("connections"));
        assert!(json.contains("5"));
        assert!(json.contains("latency"));
    }
}
