//! ConsensusDriver - glues NetworkService, TbftConsensus, and BlockExecutor
//!
//! Implements the OEV pipeline:
//! 1. Ordering: proposer collects txs, TBFT consensus agrees on ordering
//! 2. Execution: after consensus, each node independently executes the agreed txs
//! 3. Storage: persist the full block with execution results

use bach_consensus::{
    ConsensusMessage, Proposal, Step, TbftConsensus, Vote,
};
use bach_core::BlockExecutor;
use bach_crypto::{keccak256, public_key_to_address, PrivateKey};
use bach_network::{Message, MessageType, NetworkEvent, NetworkService};
use bach_primitives::{Address, H256};
use bach_storage::{BlockDb, StateDb};
use bach_txpool::{PooledTransaction, TxPool};
use bach_types::{Block, BlockBody, BlockHeader, Bloom, SignedTransaction};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::genesis::{compute_block_hash, encode_body, encode_header, encode_receipts};

/// Consensus wire message: wraps Proposal or Vote for network transport.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum WireMsg {
    Proposal(Proposal),
    Vote(Vote),
}

/// ConsensusDriver orchestrates network ↔ consensus ↔ execution.
#[allow(dead_code)]
pub struct ConsensusDriver {
    consensus: TbftConsensus,
    network: Arc<NetworkService>,
    txpool: Arc<TxPool>,
    executor: Arc<RwLock<BlockExecutor>>,
    state_db: Arc<StateDb>,
    block_db: Arc<BlockDb>,
    chain_id: u64,
    gas_limit: u64,
    signing_key: Option<PrivateKey>,
    /// Stores tx_data keyed by block_hash so all validators can execute after finalization.
    pending_tx_data: HashMap<H256, Vec<u8>>,
}

impl ConsensusDriver {
    /// Create a new ConsensusDriver
    pub fn new(
        consensus: TbftConsensus,
        network: Arc<NetworkService>,
        txpool: Arc<TxPool>,
        executor: Arc<RwLock<BlockExecutor>>,
        state_db: Arc<StateDb>,
        block_db: Arc<BlockDb>,
        chain_id: u64,
        gas_limit: u64,
        signing_key: Option<PrivateKey>,
    ) -> Self {
        Self {
            consensus,
            network,
            txpool,
            executor,
            state_db,
            block_db,
            chain_id,
            gas_limit,
            signing_key,
            pending_tx_data: HashMap::new(),
        }
    }

    /// Run the consensus event loop.
    pub async fn run(
        mut self,
        mut net_events: tokio::sync::mpsc::Receiver<NetworkEvent>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        let start_height = self
            .block_db
            .get_latest_block()
            .unwrap_or(Some(0))
            .unwrap_or(0)
            + 1;

        tracing::info!("Consensus starting at height {}", start_height);
        self.consensus.start_height(start_height);
        self.drain_messages().await;

        let mut timeout = tokio::time::interval(self.step_timeout());
        timeout.reset();

        loop {
            tokio::select! {
                Some(event) = net_events.recv() => {
                    self.on_network_event(event).await;
                }
                _ = timeout.tick() => {
                    let (h, r, s) = (self.consensus.height(), self.consensus.round(), self.consensus.step());
                    tracing::debug!("Timeout h={} r={} step={:?}", h, r, s);
                    self.consensus.on_timeout(h, r, s);
                    self.drain_messages().await;
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        tracing::info!("Consensus driver stopping");
                        break;
                    }
                }
            }
            // Reset timeout after each event
            timeout = tokio::time::interval(self.step_timeout());
            timeout.reset();
        }
    }

    fn step_timeout(&self) -> Duration {
        Duration::from_millis(match self.consensus.step() {
            Step::Propose | Step::NewRound => 3000,
            Step::Prevote => 1000,
            Step::Precommit => 1000,
            Step::Commit => 500,
        })
    }

    // ── Network event dispatch ──────────────────────────────────────────

    async fn on_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::PeerConnected(id) => {
                tracing::info!("Peer connected: {}", id);
            }
            NetworkEvent::PeerDisconnected(id) => {
                tracing::info!("Peer disconnected: {}", id);
            }
            NetworkEvent::Message { message, .. } => match message.msg_type {
                MessageType::Consensus => self.on_consensus_wire(&message.payload).await,
                MessageType::TxBroadcast => self.on_tx_broadcast(&message.payload),
                _ => {}
            },
        }
    }

    async fn on_consensus_wire(&mut self, payload: &[u8]) {
        let wire: WireMsg = match serde_json::from_slice(payload) {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("Bad consensus msg: {}", e);
                return;
            }
        };
        match wire {
            WireMsg::Proposal(p) => {
                tracing::debug!("Recv proposal h={} r={} hash={}", p.height, p.round, p.block_hash.to_hex());
                if !p.tx_data.is_empty() {
                    self.pending_tx_data.insert(p.block_hash, p.tx_data.clone());
                }
                let _ = self.consensus.on_proposal(p);
            }
            WireMsg::Vote(v) => {
                tracing::debug!("Recv {:?} h={} r={} from={}", v.vote_type, v.height, v.round, v.voter);
                let _ = self.consensus.on_vote(v);
            }
        }
        self.drain_messages().await;
    }

    fn on_tx_broadcast(&self, payload: &[u8]) {
        let bcast: bach_network::TxBroadcast = match serde_json::from_slice(payload) {
            Ok(b) => b,
            Err(_) => return,
        };
        if let Some(tx) = bach_types::codec::decode_signed_tx(&bcast.data) {
            let hash = bcast.hash;
            // Use Address::ZERO as sender placeholder - actual sender verification
            // happens during execution
            let _ = self.txpool.add(tx, Address::ZERO, hash);
        }
    }

    // ── Drain consensus engine output (loop to handle cascading events) ─

    async fn drain_messages(&mut self) {
        // Use a loop instead of recursion to handle cascading events
        // (e.g. CreateBlock -> propose_block -> more messages)
        loop {
            let msgs = self.consensus.take_messages();
            if msgs.is_empty() {
                break;
            }
            for msg in msgs {
                match msg {
                    ConsensusMessage::CreateBlock { height, round } => {
                        self.do_create_block(height, round);
                        // propose_block may produce more messages; loop will pick them up
                    }
                    ConsensusMessage::Proposal(mut p) => {
                        if let Some(data) = self.pending_tx_data.get(&p.block_hash) {
                            p.tx_data = data.clone();
                        }
                        self.broadcast(WireMsg::Proposal(p)).await;
                    }
                    ConsensusMessage::Vote(v) => {
                        self.broadcast(WireMsg::Vote(v)).await;
                    }
                    ConsensusMessage::Finalized { height, block_hash, .. } => {
                        self.on_finalized(height, block_hash).await;
                        // on_finalized calls start_height which may produce CreateBlock
                    }
                }
            }
        }
    }

    async fn broadcast(&self, wire: WireMsg) {
        if let Ok(payload) = serde_json::to_vec(&wire) {
            let msg = Message::new(MessageType::Consensus, payload.into());
            self.network.broadcast(msg).await;
        }
    }

    // ── OEV step 1: Ordering (create block proposal) ────────────────────

    fn do_create_block(&mut self, height: u64, round: u32) {
        let pending = self.txpool.get_pending(100);
        tracing::info!("Proposing block h={} with {} txs", height, pending.len());

        let tx_data = encode_txs(&pending);
        let block_hash = proposal_hash(height, round, &tx_data);

        self.pending_tx_data.insert(block_hash, tx_data);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Err(e) = self.consensus.propose_block(block_hash, timestamp) {
            tracing::error!("propose_block failed: {}", e);
        }
        // Messages will be drained by the outer loop
    }

    // ── OEV step 2 & 3: Execution + Storage ─────────────────────────────

    async fn on_finalized(&mut self, height: u64, block_hash: H256) {
        let tx_data = self.pending_tx_data.remove(&block_hash).unwrap_or_default();
        let transactions = decode_txs(&tx_data);

        let latest_number = self.block_db.get_latest_block().unwrap_or(Some(0)).unwrap_or(0);
        let parent_hash = self.block_db.get_hash_by_number(latest_number)
            .unwrap_or(Some(H256::ZERO)).unwrap_or(H256::ZERO);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let block = Block {
            header: BlockHeader {
                parent_hash,
                ommers_hash: H256::ZERO,
                beneficiary: self.our_address(),
                state_root: H256::ZERO,
                transactions_root: H256::ZERO,
                receipts_root: H256::ZERO,
                logs_bloom: Bloom::default(),
                difficulty: 0,
                number: height,
                gas_limit: self.gas_limit,
                gas_used: 0,
                timestamp,
                extra_data: Bytes::new(),
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000),
            },
            body: BlockBody { transactions },
        };

        // Execute
        let mut executor = self.executor.write().await;
        let result = match executor.execute_block(&block) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Execution failed at h={}: {}", height, e);
                drop(executor);
                self.consensus.start_height(height + 1);
                return;
            }
        };

        // Commit state
        let cache = executor.state().cache();
        if let Err(e) = self.state_db.commit(cache) {
            tracing::error!("State commit failed: {}", e);
        }
        drop(executor);

        // Persist block
        let final_hash = compute_block_hash(&block);
        let _ = (|| -> Result<(), bach_storage::StorageError> {
            self.block_db.put_header(&final_hash, &encode_header(&block.header))?;
            self.block_db.put_body(&final_hash, &encode_body(&block.body))?;
            self.block_db.put_receipts(&final_hash, &encode_receipts(&result.receipts))?;
            self.block_db.put_hash_by_number(height, &final_hash)?;
            self.block_db.set_latest_block(height)?;
            Ok(())
        })();

        // Update txpool
        for tx in &block.body.transactions {
            let tx_hash = bach_types::codec::tx_hash(tx);
            self.txpool.remove(&tx_hash);
        }

        self.network.set_height(height);

        tracing::info!(
            "Block {} produced: hash={}, txs={}, gas_used={}",
            height, final_hash.to_hex(), block.body.transactions.len(), result.gas_used
        );

        // Advance to next height
        // Note: start_height may produce CreateBlock; the outer drain_messages loop handles it.
        self.consensus.start_height(height + 1);
    }

    fn our_address(&self) -> Address {
        self.signing_key
            .as_ref()
            .map(|k| public_key_to_address(k.verifying_key()))
            .unwrap_or(Address::ZERO)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn proposal_hash(height: u64, round: u32, tx_data: &[u8]) -> H256 {
    let mut buf = Vec::new();
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(&round.to_le_bytes());
    buf.extend_from_slice(tx_data);
    keccak256(&buf)
}

fn encode_txs(txs: &[PooledTransaction]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(txs.len() as u32).to_le_bytes());
    for pt in txs {
        let encoded = bach_types::codec::encode_signed_tx(&pt.tx);
        buf.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        buf.extend_from_slice(&encoded);
    }
    buf
}

fn decode_txs(data: &[u8]) -> Vec<SignedTransaction> {
    if data.len() < 4 { return Vec::new(); }
    let mut pos = 0;
    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
    pos += 4;
    let mut txs = Vec::with_capacity(count);
    for _ in 0..count {
        if pos + 4 > data.len() { break; }
        let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;
        if pos + len > data.len() { break; }
        if let Some(tx) = bach_types::codec::decode_signed_tx(&data[pos..pos + len]) {
            txs.push(tx);
        }
        pos += len;
    }
    txs
}
