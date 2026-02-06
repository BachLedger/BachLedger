//! Histogram implementation for latency tracking

use std::sync::atomic::{AtomicU64, Ordering};

/// Histogram for tracking value distributions
pub struct Histogram {
    /// Bucket boundaries (in microseconds)
    buckets: Vec<f64>,
    /// Counts per bucket
    counts: Vec<AtomicU64>,
    /// Sum of all values
    sum: AtomicU64,
    /// Total count
    count: AtomicU64,
}

impl Histogram {
    /// Create histogram with default buckets
    pub fn new() -> Self {
        Self::with_buckets(vec![
            10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
        ])
    }

    /// Create histogram with custom buckets
    pub fn with_buckets(buckets: Vec<f64>) -> Self {
        let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
        Histogram {
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// Record a value
    pub fn observe(&self, value: f64) {
        self.sum.fetch_add(value as u64, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        for (i, boundary) in self.buckets.iter().enumerate() {
            if value <= *boundary {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Value exceeds all buckets
        if let Some(last) = self.counts.last() {
            last.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get mean value
    pub fn mean(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        self.sum.load(Ordering::Relaxed) as f64 / count as f64
    }

    /// Get total count
    pub fn total_count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}
