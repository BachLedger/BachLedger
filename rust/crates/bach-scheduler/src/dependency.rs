//! Dependency graph for transaction ordering
//!
//! Builds a directed acyclic graph (DAG) representing dependencies
//! between transactions based on their read/write sets.

use crate::error::{SchedulerError, SchedulerResult};
use crate::rw_set::RWSet;
use crate::state_key::TxId;
use std::collections::{HashMap, HashSet, VecDeque};

/// Type of dependency between transactions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyType {
    /// Read-after-write: tx reads what another tx writes
    ReadAfterWrite,
    /// Write-after-write: both transactions write same key
    WriteAfterWrite,
    /// Write-after-read: tx writes what another tx reads
    WriteAfterRead,
}

/// Edge in the dependency graph
#[derive(Clone, Debug)]
struct DependencyEdge {
    /// Target transaction (depends on source)
    to: TxId,
    /// Type of dependency
    #[allow(dead_code)]
    dep_type: DependencyType,
}

/// Dependency graph for transactions
///
/// Represents a DAG where edges indicate that one transaction
/// must execute before another due to state dependencies.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Forward edges: tx -> transactions that depend on it
    forward: HashMap<TxId, Vec<DependencyEdge>>,
    /// Backward edges: tx -> transactions it depends on
    backward: HashMap<TxId, HashSet<TxId>>,
    /// All registered transactions
    transactions: HashSet<TxId>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Build dependency graph from a list of RW sets
    ///
    /// Analyzes all pairs of transactions to find dependencies.
    /// For transactions i < j, if j depends on i, add edge i -> j.
    pub fn build(rw_sets: &[(TxId, RWSet)]) -> Self {
        let mut graph = Self::new();

        // Register all transactions
        for (tx_id, _) in rw_sets {
            graph.add_transaction(*tx_id);
        }

        // Check all pairs for dependencies
        for i in 0..rw_sets.len() {
            for j in (i + 1)..rw_sets.len() {
                let (tx_i, rw_i) = &rw_sets[i];
                let (tx_j, rw_j) = &rw_sets[j];

                // Check if tx_j depends on tx_i (RAW: j reads what i writes)
                if rw_j.has_raw_dependency(rw_i) {
                    graph.add_dependency(*tx_i, *tx_j, DependencyType::ReadAfterWrite);
                }

                // Check for WAW conflict (both write same key)
                if rw_j.has_waw_conflict(rw_i) {
                    graph.add_dependency(*tx_i, *tx_j, DependencyType::WriteAfterWrite);
                }

                // Check if tx_j has WAR with tx_i (j writes what i reads)
                // In serial order, i reads before j writes, so j depends on i
                if rw_j.has_war_conflict(rw_i) {
                    graph.add_dependency(*tx_i, *tx_j, DependencyType::WriteAfterRead);
                }
            }
        }

        graph
    }

    /// Add a transaction to the graph
    pub fn add_transaction(&mut self, tx_id: TxId) {
        self.transactions.insert(tx_id);
        self.forward.entry(tx_id).or_default();
        self.backward.entry(tx_id).or_default();
    }

    /// Add a dependency edge: `from` must execute before `to`
    pub fn add_dependency(&mut self, from: TxId, to: TxId, dep_type: DependencyType) {
        self.forward
            .entry(from)
            .or_default()
            .push(DependencyEdge { to, dep_type });
        self.backward.entry(to).or_default().insert(from);
    }

    /// Get transactions that depend on the given transaction
    pub fn get_dependents(&self, tx_id: TxId) -> Vec<TxId> {
        self.forward
            .get(&tx_id)
            .map(|edges| edges.iter().map(|e| e.to).collect())
            .unwrap_or_default()
    }

    /// Get transactions that the given transaction depends on
    pub fn get_dependencies(&self, tx_id: TxId) -> Vec<TxId> {
        self.backward
            .get(&tx_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if transaction has any dependencies
    pub fn has_dependencies(&self, tx_id: TxId) -> bool {
        self.backward
            .get(&tx_id)
            .map(|deps| !deps.is_empty())
            .unwrap_or(false)
    }

    /// Get in-degree (number of dependencies) for a transaction
    pub fn in_degree(&self, tx_id: TxId) -> usize {
        self.backward.get(&tx_id).map(|deps| deps.len()).unwrap_or(0)
    }

    /// Remove a dependency edge
    pub fn remove_dependency(&mut self, from: TxId, to: TxId) {
        if let Some(edges) = self.forward.get_mut(&from) {
            edges.retain(|e| e.to != to);
        }
        if let Some(deps) = self.backward.get_mut(&to) {
            deps.remove(&from);
        }
    }

    /// Get all transactions with no dependencies (ready to execute)
    pub fn get_ready_transactions(&self) -> Vec<TxId> {
        self.transactions
            .iter()
            .filter(|tx| self.in_degree(**tx) == 0)
            .cloned()
            .collect()
    }

    /// Perform topological sort using Kahn's algorithm
    ///
    /// Returns transactions in valid execution order, or error if cycle detected.
    pub fn topological_sort(&self) -> SchedulerResult<Vec<TxId>> {
        let mut in_degree: HashMap<TxId, usize> = self
            .transactions
            .iter()
            .map(|tx| (*tx, self.in_degree(*tx)))
            .collect();

        let mut queue: VecDeque<TxId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&tx, _)| tx)
            .collect();

        let mut result = Vec::with_capacity(self.transactions.len());

        while let Some(tx) = queue.pop_front() {
            result.push(tx);

            for dep in self.get_dependents(tx) {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        if result.len() != self.transactions.len() {
            // Cycle detected - find a transaction in the cycle
            let in_cycle = self
                .transactions
                .iter()
                .find(|tx| in_degree.get(tx).copied().unwrap_or(0) > 0)
                .cloned()
                .unwrap_or(TxId::new(0));
            return Err(SchedulerError::CircularDependency(in_cycle));
        }

        Ok(result)
    }

    /// Generate parallel execution batches
    ///
    /// Each batch contains transactions that can execute in parallel
    /// (no dependencies between them within the batch).
    pub fn generate_batches(&self) -> SchedulerResult<Vec<Vec<TxId>>> {
        let mut batches = Vec::new();
        let mut remaining: HashSet<TxId> = self.transactions.clone();
        let mut completed: HashSet<TxId> = HashSet::new();

        while !remaining.is_empty() {
            // Find all transactions whose dependencies are satisfied
            let batch: Vec<TxId> = remaining
                .iter()
                .filter(|tx| {
                    self.backward
                        .get(tx)
                        .map(|deps| deps.iter().all(|d| completed.contains(d)))
                        .unwrap_or(true)
                })
                .cloned()
                .collect();

            if batch.is_empty() {
                // Cycle detected
                let in_cycle = remaining.iter().next().cloned().unwrap();
                return Err(SchedulerError::CircularDependency(in_cycle));
            }

            // Move batch transactions from remaining to completed
            for tx in &batch {
                remaining.remove(tx);
                completed.insert(*tx);
            }

            batches.push(batch);
        }

        Ok(batches)
    }

    /// Get total number of transactions
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Get total number of dependency edges
    pub fn edge_count(&self) -> usize {
        self.forward.values().map(|edges| edges.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_key::StateKey;
    use bach_primitives::{Address, H256};

    fn make_key(id: u8) -> StateKey {
        StateKey::new(
            Address::from_bytes([id; 20]),
            H256::from_bytes([id; 32]),
        )
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
    }

    #[test]
    fn test_no_dependencies() {
        // Three independent transactions
        let mut rw_sets = Vec::new();

        let mut rw1 = RWSet::new();
        rw1.record_write(make_key(1));
        rw_sets.push((TxId::new(0), rw1));

        let mut rw2 = RWSet::new();
        rw2.record_write(make_key(2));
        rw_sets.push((TxId::new(1), rw2));

        let mut rw3 = RWSet::new();
        rw3.record_write(make_key(3));
        rw_sets.push((TxId::new(2), rw3));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.edge_count(), 0);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 3);
    }

    #[test]
    fn test_raw_dependency() {
        // tx0 writes key1, tx1 reads key1
        let key = make_key(1);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_read(key.clone());
        rw_sets.push((TxId::new(1), rw1));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.edge_count(), 1);
        assert!(graph.has_dependencies(TxId::new(1)));
        assert!(!graph.has_dependencies(TxId::new(0)));

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec![TxId::new(0), TxId::new(1)]);
    }

    #[test]
    fn test_waw_dependency() {
        // tx0 and tx1 both write key1
        let key = make_key(1);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_write(key.clone());
        rw_sets.push((TxId::new(1), rw1));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.edge_count(), 1);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_chain_dependency() {
        // tx0 -> tx1 -> tx2 chain
        let key1 = make_key(1);
        let key2 = make_key(2);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key1.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_read(key1.clone());
        rw1.record_write(key2.clone());
        rw_sets.push((TxId::new(1), rw1));

        let mut rw2 = RWSet::new();
        rw2.record_read(key2.clone());
        rw_sets.push((TxId::new(2), rw2));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.edge_count(), 2);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec![TxId::new(0), TxId::new(1), TxId::new(2)]);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 3);
    }

    #[test]
    fn test_parallel_batches() {
        // tx0 writes key1
        // tx1 writes key2
        // tx2 reads key1 (depends on tx0)
        // tx3 reads key2 (depends on tx1)
        let key1 = make_key(1);
        let key2 = make_key(2);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key1.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_write(key2.clone());
        rw_sets.push((TxId::new(1), rw1));

        let mut rw2 = RWSet::new();
        rw2.record_read(key1.clone());
        rw_sets.push((TxId::new(2), rw2));

        let mut rw3 = RWSet::new();
        rw3.record_read(key2.clone());
        rw_sets.push((TxId::new(3), rw3));

        let graph = DependencyGraph::build(&rw_sets);

        let batches = graph.generate_batches().unwrap();
        // Batch 0: tx0, tx1 (independent)
        // Batch 1: tx2, tx3 (both dependencies satisfied)
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 2);
    }

    #[test]
    fn test_complex_dependencies() {
        // Diamond pattern:
        //     tx0
        //    /   \
        //  tx1   tx2
        //    \   /
        //     tx3
        let key1 = make_key(1);
        let key2 = make_key(2);
        let key3 = make_key(3);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key1.clone());
        rw0.record_write(key2.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_read(key1.clone());
        rw1.record_write(key3.clone());
        rw_sets.push((TxId::new(1), rw1));

        let mut rw2 = RWSet::new();
        rw2.record_read(key2.clone());
        rw2.record_write(key3.clone());
        rw_sets.push((TxId::new(2), rw2));

        let mut rw3 = RWSet::new();
        rw3.record_read(key3.clone());
        rw_sets.push((TxId::new(3), rw3));

        let graph = DependencyGraph::build(&rw_sets);

        let batches = graph.generate_batches().unwrap();
        // Batch 0: tx0
        // Batch 1: tx1, tx2 (but WAW on key3 creates dependency)
        // Actually tx1 -> tx2 due to WAW, so:
        // Batch 0: tx0
        // Batch 1: tx1
        // Batch 2: tx2
        // Batch 3: tx3
        // Or if tx2 depends on tx1:
        assert!(batches.len() >= 2);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted[0], TxId::new(0)); // tx0 first
        assert_eq!(*sorted.last().unwrap(), TxId::new(3)); // tx3 last
    }

    // ==================== Additional Graph API Tests ====================

    #[test]
    fn test_add_transaction() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_dependency(TxId::new(0), TxId::new(1), DependencyType::ReadAfterWrite);

        assert_eq!(graph.edge_count(), 1);
        assert!(graph.has_dependencies(TxId::new(1)));
        assert!(!graph.has_dependencies(TxId::new(0)));
    }

    #[test]
    fn test_get_dependents() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));
        graph.add_dependency(TxId::new(0), TxId::new(1), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(0), TxId::new(2), DependencyType::WriteAfterWrite);

        let dependents = graph.get_dependents(TxId::new(0));
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&TxId::new(1)));
        assert!(dependents.contains(&TxId::new(2)));
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));
        graph.add_dependency(TxId::new(0), TxId::new(2), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(1), TxId::new(2), DependencyType::WriteAfterWrite);

        let deps = graph.get_dependencies(TxId::new(2));
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&TxId::new(0)));
        assert!(deps.contains(&TxId::new(1)));
    }

    #[test]
    fn test_in_degree() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));
        graph.add_dependency(TxId::new(0), TxId::new(2), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(1), TxId::new(2), DependencyType::WriteAfterWrite);

        assert_eq!(graph.in_degree(TxId::new(0)), 0);
        assert_eq!(graph.in_degree(TxId::new(1)), 0);
        assert_eq!(graph.in_degree(TxId::new(2)), 2);
    }

    #[test]
    fn test_remove_dependency() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_dependency(TxId::new(0), TxId::new(1), DependencyType::ReadAfterWrite);

        assert_eq!(graph.edge_count(), 1);

        graph.remove_dependency(TxId::new(0), TxId::new(1));

        assert_eq!(graph.edge_count(), 0);
        assert!(!graph.has_dependencies(TxId::new(1)));
    }

    #[test]
    fn test_get_ready_transactions() {
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));
        graph.add_dependency(TxId::new(0), TxId::new(2), DependencyType::ReadAfterWrite);

        let ready = graph.get_ready_transactions();
        assert_eq!(ready.len(), 2); // tx0 and tx1 have no dependencies
        assert!(ready.contains(&TxId::new(0)));
        assert!(ready.contains(&TxId::new(1)));
        assert!(!ready.contains(&TxId::new(2)));
    }

    // ==================== Circular Dependency Detection ====================

    #[test]
    fn test_circular_dependency_topological_sort() {
        // Manually create a cycle: tx0 -> tx1 -> tx0
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_dependency(TxId::new(0), TxId::new(1), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(1), TxId::new(0), DependencyType::ReadAfterWrite);

        let result = graph.topological_sort();
        assert!(result.is_err());

        if let Err(SchedulerError::CircularDependency(tx)) = result {
            // The cycle involves tx0 or tx1
            assert!(tx == TxId::new(0) || tx == TxId::new(1));
        } else {
            panic!("Expected CircularDependency error");
        }
    }

    #[test]
    fn test_circular_dependency_generate_batches() {
        // Create a 3-node cycle: tx0 -> tx1 -> tx2 -> tx0
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_transaction(TxId::new(1));
        graph.add_transaction(TxId::new(2));
        graph.add_dependency(TxId::new(0), TxId::new(1), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(1), TxId::new(2), DependencyType::ReadAfterWrite);
        graph.add_dependency(TxId::new(2), TxId::new(0), DependencyType::ReadAfterWrite);

        let result = graph.generate_batches();
        assert!(result.is_err());
        matches!(result.unwrap_err(), SchedulerError::CircularDependency(_));
    }

    #[test]
    fn test_self_loop_cycle() {
        // Self-loop: tx0 -> tx0
        let mut graph = DependencyGraph::new();

        graph.add_transaction(TxId::new(0));
        graph.add_dependency(TxId::new(0), TxId::new(0), DependencyType::WriteAfterWrite);

        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    // ==================== WAR Dependency Tests ====================

    #[test]
    fn test_war_dependency() {
        // tx0 reads key1, tx1 writes key1 (WAR: tx1 depends on tx0)
        let key = make_key(1);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_read(key.clone());
        rw_sets.push((TxId::new(0), rw0));

        let mut rw1 = RWSet::new();
        rw1.record_write(key.clone());
        rw_sets.push((TxId::new(1), rw1));

        let graph = DependencyGraph::build(&rw_sets);

        // tx1 has WAR conflict: it writes what tx0 reads
        // In serial order, tx0 reads first, then tx1 writes
        // So tx1 depends on tx0 completing its read
        assert_eq!(graph.edge_count(), 1);
        assert!(graph.has_dependencies(TxId::new(1)));

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec![TxId::new(0), TxId::new(1)]);
    }

    // ==================== Dependency Type Tests ====================

    #[test]
    fn test_dependency_type_enum() {
        let raw = DependencyType::ReadAfterWrite;
        let waw = DependencyType::WriteAfterWrite;
        let war = DependencyType::WriteAfterRead;

        // Test equality
        assert_eq!(raw, DependencyType::ReadAfterWrite);
        assert_ne!(raw, waw);
        assert_ne!(raw, war);

        // Test copy
        let raw_copy = raw;
        assert_eq!(raw, raw_copy);
    }

    // ==================== Large Scale Tests ====================

    #[test]
    fn test_many_independent_transactions() {
        let mut rw_sets = Vec::new();

        // 100 independent transactions, each writing a unique key
        for i in 0..100u8 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i));
            rw_sets.push((TxId::new(i as u32), rw));
        }

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.len(), 100);
        assert_eq!(graph.edge_count(), 0);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 100);
    }

    #[test]
    fn test_fully_serial_transactions() {
        let key = make_key(1);
        let mut rw_sets = Vec::new();

        // 20 transactions all writing the same key -> fully serial
        for i in 0..20 {
            let mut rw = RWSet::new();
            rw.record_write(key.clone());
            rw_sets.push((TxId::new(i), rw));
        }

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.len(), 20);
        // Each tx i depends on all previous tx j (j < i) for WAW conflicts
        // Total edges: n*(n-1)/2 = 20*19/2 = 190
        assert_eq!(graph.edge_count(), 190);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 20);

        for batch in &batches {
            assert_eq!(batch.len(), 1);
        }
    }

    #[test]
    fn test_long_chain() {
        // tx0 -> tx1 -> tx2 -> ... -> tx19
        let mut rw_sets = Vec::new();

        for i in 0..20 {
            let mut rw = RWSet::new();
            if i > 0 {
                rw.record_read(make_key(i - 1));
            }
            rw.record_write(make_key(i));
            rw_sets.push((TxId::new(i as u32), rw));
        }

        let graph = DependencyGraph::build(&rw_sets);

        let sorted = graph.topological_sort().unwrap();

        // Should be in order 0, 1, 2, ..., 19
        for (idx, tx) in sorted.iter().enumerate() {
            assert_eq!(tx.as_u32(), idx as u32);
        }
    }

    #[test]
    fn test_fan_out_pattern() {
        // tx0 writes key0
        // tx1, tx2, tx3, tx4 all read key0 (fan-out from tx0)
        let key0 = make_key(0);
        let mut rw_sets = Vec::new();

        let mut rw0 = RWSet::new();
        rw0.record_write(key0.clone());
        rw_sets.push((TxId::new(0), rw0));

        for i in 1..5 {
            let mut rw = RWSet::new();
            rw.record_read(key0.clone());
            rw_sets.push((TxId::new(i), rw));
        }

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.edge_count(), 4); // tx0 -> tx1, tx0 -> tx2, tx0 -> tx3, tx0 -> tx4

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 1); // tx0
        assert_eq!(batches[1].len(), 4); // tx1, tx2, tx3, tx4
    }

    #[test]
    fn test_fan_in_pattern() {
        // tx0, tx1, tx2, tx3 all write different keys
        // tx4 reads all of them (fan-in to tx4)
        let mut rw_sets = Vec::new();

        for i in 0..4 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i));
            rw_sets.push((TxId::new(i as u32), rw));
        }

        let mut rw4 = RWSet::new();
        for i in 0..4 {
            rw4.record_read(make_key(i));
        }
        rw_sets.push((TxId::new(4), rw4));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.edge_count(), 4);
        assert_eq!(graph.in_degree(TxId::new(4)), 4);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 4); // tx0, tx1, tx2, tx3
        assert_eq!(batches[1].len(), 1); // tx4
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_build_from_empty_rw_sets() {
        let rw_sets: Vec<(TxId, RWSet)> = Vec::new();
        let graph = DependencyGraph::build(&rw_sets);

        assert!(graph.is_empty());
        assert_eq!(graph.topological_sort().unwrap().len(), 0);
        assert_eq!(graph.generate_batches().unwrap().len(), 0);
    }

    #[test]
    fn test_single_transaction() {
        let mut rw_sets = Vec::new();

        let mut rw = RWSet::new();
        rw.record_write(make_key(1));
        rw_sets.push((TxId::new(0), rw));

        let graph = DependencyGraph::build(&rw_sets);

        assert_eq!(graph.len(), 1);
        assert_eq!(graph.edge_count(), 0);

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec![TxId::new(0)]);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
    }

    #[test]
    fn test_transaction_with_empty_rw_set() {
        let mut rw_sets = Vec::new();

        // tx0 has empty RW set
        let rw0 = RWSet::new();
        rw_sets.push((TxId::new(0), rw0));

        // tx1 writes key1
        let mut rw1 = RWSet::new();
        rw1.record_write(make_key(1));
        rw_sets.push((TxId::new(1), rw1));

        let graph = DependencyGraph::build(&rw_sets);

        // No dependencies between them
        assert_eq!(graph.edge_count(), 0);

        let batches = graph.generate_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
    }

    #[test]
    fn test_get_dependents_nonexistent() {
        let graph = DependencyGraph::new();
        let dependents = graph.get_dependents(TxId::new(99));
        assert!(dependents.is_empty());
    }

    #[test]
    fn test_get_dependencies_nonexistent() {
        let graph = DependencyGraph::new();
        let deps = graph.get_dependencies(TxId::new(99));
        assert!(deps.is_empty());
    }

    #[test]
    fn test_in_degree_nonexistent() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.in_degree(TxId::new(99)), 0);
    }

    #[test]
    fn test_has_dependencies_nonexistent() {
        let graph = DependencyGraph::new();
        assert!(!graph.has_dependencies(TxId::new(99)));
    }
}
