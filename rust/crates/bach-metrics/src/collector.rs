//! Metrics collector implementation

use crate::Histogram;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

/// Thread-safe metrics storage
pub struct Metrics {
    /// Histogram metrics for latency tracking
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
    /// Counter metrics for event counting
    counters: RwLock<HashMap<String, Arc<AtomicU64>>>,
    /// Gauge metrics for current values
    gauges: RwLock<HashMap<String, Arc<AtomicI64>>>,
}

impl Metrics {
    /// Create a new metrics store
    pub fn new() -> Self {
        Self {
            histograms: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
        }
    }

    /// Record a histogram observation
    pub fn histogram(&self, name: &str, value: f64) {
        let histograms = self.histograms.read();
        if let Some(h) = histograms.get(name) {
            h.observe(value);
            return;
        }
        drop(histograms);

        let mut histograms = self.histograms.write();
        let h = histograms
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Histogram::new()));
        h.observe(value);
    }

    /// Increment a counter
    pub fn counter(&self, name: &str, delta: u64) {
        let counters = self.counters.read();
        if let Some(c) = counters.get(name) {
            c.fetch_add(delta, Ordering::Relaxed);
            return;
        }
        drop(counters);

        let mut counters = self.counters.write();
        let c = counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        c.fetch_add(delta, Ordering::Relaxed);
    }

    /// Set a gauge value
    pub fn gauge(&self, name: &str, value: i64) {
        let gauges = self.gauges.read();
        if let Some(g) = gauges.get(name) {
            g.store(value, Ordering::Relaxed);
            return;
        }
        drop(gauges);

        let mut gauges = self.gauges.write();
        let g = gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicI64::new(0)));
        g.store(value, Ordering::Relaxed);
    }

    /// Get histogram mean for a metric
    pub fn get_histogram_mean(&self, name: &str) -> Option<f64> {
        self.histograms.read().get(name).map(|h| h.mean())
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters
            .read()
            .get(name)
            .map(|c| c.load(Ordering::Relaxed))
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        self.gauges
            .read()
            .get(name)
            .map(|g| g.load(Ordering::Relaxed))
    }

    /// Get all counter names and values
    pub fn all_counters(&self) -> Vec<(String, u64)> {
        self.counters
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get all gauge names and values
    pub fn all_gauges(&self) -> Vec<(String, i64)> {
        self.gauges
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get all histogram names and mean values
    pub fn all_histograms(&self) -> Vec<(String, f64, u64)> {
        self.histograms
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.mean(), v.total_count()))
            .collect()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics collector
pub struct MetricsCollector {
    metrics: Arc<Metrics>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Metrics::new()),
        }
    }

    /// Get a reference to the metrics store
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Get a clone of the metrics Arc for sharing
    pub fn shared(&self) -> Arc<Metrics> {
        Arc::clone(&self.metrics)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let metrics = Metrics::new();
        metrics.counter("test", 1);
        metrics.counter("test", 2);
        assert_eq!(metrics.get_counter("test"), Some(3));
    }

    #[test]
    fn test_gauge() {
        let metrics = Metrics::new();
        metrics.gauge("test", 42);
        assert_eq!(metrics.get_gauge("test"), Some(42));
        metrics.gauge("test", -10);
        assert_eq!(metrics.get_gauge("test"), Some(-10));
    }

    #[test]
    fn test_histogram() {
        let metrics = Metrics::new();
        metrics.histogram("test", 100.0);
        metrics.histogram("test", 200.0);
        assert_eq!(metrics.get_histogram_mean("test"), Some(150.0));
    }
}
