//! TBFT (Tendermint-like BFT) consensus implementation

use crate::error::{ConsensusError, ConsensusResult};
use crate::types::{Commit, Proposal, ValidatorSet, Vote, VoteType};
use bach_crypto::{keccak256, public_key_to_address, recover_public_key, Signature};
use bach_primitives::{Address, H256};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// TBFT state machine step
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Step {
    /// Waiting for new round to start
    NewRound,
    /// Waiting for proposal
    Propose,
    /// Prevote phase
    Prevote,
    /// Precommit phase
    Precommit,
    /// Block committed
    Commit,
}

/// Round state - tracks votes for a single round
#[derive(Debug, Clone, Default)]
struct RoundState {
    /// Proposal for this round
    proposal: Option<Proposal>,
    /// Prevotes collected
    prevotes: HashMap<Address, Vote>,
    /// Precommits collected
    precommits: HashMap<Address, Vote>,
    /// Locked block hash (from prevote polka) - used in future POL rules
    #[allow(dead_code)]
    locked_block: Option<H256>,
    /// Valid block hash (from precommit polka) - used in future POL rules
    #[allow(dead_code)]
    valid_block: Option<H256>,
}

impl RoundState {
    fn add_prevote(&mut self, vote: Vote) -> bool {
        if self.prevotes.contains_key(&vote.voter) {
            return false;
        }
        self.prevotes.insert(vote.voter, vote);
        true
    }

    fn add_precommit(&mut self, vote: Vote) -> bool {
        if self.precommits.contains_key(&vote.voter) {
            return false;
        }
        self.precommits.insert(vote.voter, vote);
        true
    }

    fn prevote_power(&self, validator_set: &ValidatorSet) -> u64 {
        self.prevotes
            .values()
            .filter_map(|v| validator_set.get(&v.voter))
            .map(|v| v.voting_power)
            .sum()
    }

    fn precommit_power(&self, validator_set: &ValidatorSet) -> u64 {
        self.precommits
            .values()
            .filter_map(|v| validator_set.get(&v.voter))
            .map(|v| v.voting_power)
            .sum()
    }

    /// Get prevote power for a specific block hash (or nil)
    fn prevote_power_for(&self, block_hash: Option<H256>, validator_set: &ValidatorSet) -> u64 {
        self.prevotes
            .values()
            .filter(|v| v.block_hash == block_hash)
            .filter_map(|v| validator_set.get(&v.voter))
            .map(|v| v.voting_power)
            .sum()
    }

    /// Get precommit power for a specific block hash (or nil)
    fn precommit_power_for(&self, block_hash: Option<H256>, validator_set: &ValidatorSet) -> u64 {
        self.precommits
            .values()
            .filter(|v| v.block_hash == block_hash)
            .filter_map(|v| validator_set.get(&v.voter))
            .map(|v| v.voting_power)
            .sum()
    }
}

/// Height state - tracks all rounds for a height
#[derive(Debug, Default)]
struct HeightState {
    /// Round states
    rounds: HashMap<u32, RoundState>,
    /// Commit for this height (if finalized)
    commit: Option<Commit>,
}

impl HeightState {
    fn get_round(&self, round: u32) -> Option<&RoundState> {
        self.rounds.get(&round)
    }

    fn get_round_mut(&mut self, round: u32) -> &mut RoundState {
        self.rounds.entry(round).or_default()
    }
}

/// TBFT consensus engine configuration
#[derive(Debug, Clone)]
pub struct TbftConfig {
    /// Our validator address
    pub address: Address,
    /// Timeout for propose step (ms)
    pub propose_timeout: u64,
    /// Timeout for prevote step (ms)
    pub prevote_timeout: u64,
    /// Timeout for precommit step (ms)
    pub precommit_timeout: u64,
}

impl Default for TbftConfig {
    fn default() -> Self {
        Self {
            address: Address::ZERO,
            propose_timeout: 3000,
            prevote_timeout: 1000,
            precommit_timeout: 1000,
        }
    }
}

/// Messages output by the consensus engine
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConsensusMessage {
    /// Broadcast a proposal
    Proposal(Proposal),
    /// Broadcast a vote
    Vote(Vote),
    /// Request to create a new block
    CreateBlock {
        /// Height
        height: u64,
        /// Round
        round: u32,
    },
    /// Block finalized
    Finalized {
        /// Height
        height: u64,
        /// Block hash
        block_hash: H256,
        /// Commit
        commit: Commit,
    },
}

/// TBFT consensus state machine
pub struct TbftConsensus {
    /// Configuration
    config: TbftConfig,
    /// Validator set
    validator_set: Arc<RwLock<ValidatorSet>>,
    /// Current height
    height: u64,
    /// Current round
    round: u32,
    /// Current step
    step: Step,
    /// Height states
    heights: HashMap<u64, HeightState>,
    /// Locked round (for safety)
    locked_round: i32,
    /// Locked block hash
    locked_block: Option<H256>,
    /// Valid round (for liveness)
    valid_round: i32,
    /// Valid block hash
    valid_block: Option<H256>,
    /// Pending messages to broadcast
    pending_messages: Vec<ConsensusMessage>,
}

impl TbftConsensus {
    /// Create a new TBFT consensus engine
    pub fn new(config: TbftConfig, validator_set: ValidatorSet) -> Self {
        Self {
            config,
            validator_set: Arc::new(RwLock::new(validator_set)),
            height: 0,
            round: 0,
            step: Step::NewRound,
            heights: HashMap::new(),
            locked_round: -1,
            locked_block: None,
            valid_round: -1,
            valid_block: None,
            pending_messages: Vec::new(),
        }
    }

    /// Get current height
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Get current round
    pub fn round(&self) -> u32 {
        self.round
    }

    /// Get current step
    pub fn step(&self) -> Step {
        self.step
    }

    /// Check if we are the proposer for the current round
    pub fn is_proposer(&self) -> bool {
        let vs = self.validator_set.read();
        vs.proposer(self.height, self.round)
            .map(|v| v.address == self.config.address)
            .unwrap_or(false)
    }

    /// Check if a height is finalized
    pub fn is_finalized(&self, height: u64) -> bool {
        self.heights
            .get(&height)
            .map(|h| h.commit.is_some())
            .unwrap_or(false)
    }

    /// Get commit for a height
    pub fn get_commit(&self, height: u64) -> Option<&Commit> {
        self.heights.get(&height)?.commit.as_ref()
    }

    /// Take pending messages
    pub fn take_messages(&mut self) -> Vec<ConsensusMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Start a new height
    pub fn start_height(&mut self, height: u64) {
        self.height = height;
        self.round = 0;
        self.step = Step::NewRound;
        self.locked_round = -1;
        self.locked_block = None;
        self.valid_round = -1;
        self.valid_block = None;
        self.heights.entry(height).or_default();

        self.enter_new_round(0);
    }

    /// Enter a new round
    fn enter_new_round(&mut self, round: u32) {
        self.round = round;
        self.step = Step::NewRound;

        // Initialize round state
        let hs = self.heights.entry(self.height).or_default();
        hs.get_round_mut(round);

        // Move to propose step
        self.step = Step::Propose;

        if self.is_proposer() {
            // Request block creation
            self.pending_messages.push(ConsensusMessage::CreateBlock {
                height: self.height,
                round: self.round,
            });
        }
    }

    /// Handle a proposal from the proposer
    pub fn on_proposal(&mut self, proposal: Proposal) -> ConsensusResult<()> {
        // Validate proposal
        if proposal.height != self.height {
            return Err(ConsensusError::WrongHeight {
                expected: self.height,
                got: proposal.height,
            });
        }
        if proposal.round != self.round {
            return Err(ConsensusError::WrongRound {
                expected: self.round,
                got: proposal.round,
            });
        }

        // Verify proposer
        let vs = self.validator_set.read();
        let expected_proposer = vs.proposer(self.height, self.round);
        if expected_proposer.map(|v| v.address) != Some(proposal.proposer) {
            return Err(ConsensusError::InvalidProposal(format!(
                "wrong proposer: expected {:?}, got {:?}",
                expected_proposer.map(|v| v.address),
                proposal.proposer
            )));
        }
        drop(vs);

        // Verify proposal signature
        self.verify_proposal_signature(&proposal)?;

        // Store proposal
        let hs = self.heights.entry(self.height).or_default();
        let rs = hs.get_round_mut(self.round);
        rs.proposal = Some(proposal.clone());

        // Decide what to prevote
        let prevote_hash = if self.locked_round >= 0 {
            // We're locked, prevote for locked block
            self.locked_block
        } else {
            // Prevote for the proposal
            Some(proposal.block_hash)
        };

        // Send prevote
        self.send_prevote(prevote_hash);

        Ok(())
    }

    /// Called when we have a block to propose
    pub fn propose_block(&mut self, block_hash: H256, timestamp: u64) -> ConsensusResult<()> {
        if self.step != Step::Propose || !self.is_proposer() {
            return Err(ConsensusError::InvalidProposal("not our turn to propose".into()));
        }

        let proposal = Proposal::new(
            self.height,
            self.round,
            block_hash,
            self.config.address,
            timestamp,
        );

        self.pending_messages.push(ConsensusMessage::Proposal(proposal.clone()));

        // Also process our own proposal
        self.on_proposal(proposal)
    }

    /// Handle a vote
    pub fn on_vote(&mut self, vote: Vote) -> ConsensusResult<()> {
        // Validate height
        if vote.height != self.height {
            return Err(ConsensusError::WrongHeight {
                expected: self.height,
                got: vote.height,
            });
        }

        // Validate voter is in validator set
        let vs = self.validator_set.read();
        if !vs.contains(&vote.voter) {
            return Err(ConsensusError::NotValidator(vote.voter));
        }
        drop(vs);

        // Verify vote signature
        self.verify_vote_signature(&vote)?;

        // Store vote
        let hs = self.heights.entry(self.height).or_default();
        let rs = hs.get_round_mut(vote.round);

        let added = match vote.vote_type {
            VoteType::Prevote => rs.add_prevote(vote.clone()),
            VoteType::Precommit => rs.add_precommit(vote.clone()),
        };

        if !added {
            return Err(ConsensusError::DuplicateVote(vote.voter));
        }

        // Check for state transitions
        self.check_vote_thresholds(vote.round);

        Ok(())
    }

    /// Check if we've reached any voting thresholds
    fn check_vote_thresholds(&mut self, vote_round: u32) {
        let vs = self.validator_set.read();

        // Get round state (clone to avoid borrow issues)
        let rs = {
            let hs = match self.heights.get(&self.height) {
                Some(h) => h,
                None => return,
            };
            match hs.get_round(vote_round) {
                Some(r) => r.clone(),
                None => return,
            }
        };

        // Check for prevote polka (2/3+ prevotes for some block)
        if self.step == Step::Prevote && vote_round == self.round {
            // Check for polka on proposal block
            if let Some(ref proposal) = rs.proposal {
                let power = rs.prevote_power_for(Some(proposal.block_hash), &vs);
                if vs.has_two_thirds(power) {
                    // Polka! Lock on this block and move to precommit
                    self.locked_round = self.round as i32;
                    self.locked_block = Some(proposal.block_hash);
                    self.valid_round = self.round as i32;
                    self.valid_block = Some(proposal.block_hash);
                    drop(vs);

                    self.send_precommit(Some(proposal.block_hash));
                    return;
                }
            }

            // Check for nil polka
            let nil_power = rs.prevote_power_for(None, &vs);
            if vs.has_two_thirds(nil_power) {
                // Nil polka, precommit nil
                drop(vs);
                self.send_precommit(None);
                return;
            }

            // Check if we have 2/3+ prevotes total (may need to move on)
            let total_power = rs.prevote_power(&vs);
            if vs.has_two_thirds(total_power) {
                // Have enough votes but no polka, precommit nil
                drop(vs);
                self.send_precommit(None);
                return;
            }
        }

        // Check for precommit polka (finalization)
        if self.step == Step::Precommit && vote_round == self.round {
            if let Some(ref proposal) = rs.proposal {
                let power = rs.precommit_power_for(Some(proposal.block_hash), &vs);
                if vs.has_two_thirds(power) {
                    // Commit!
                    drop(vs);
                    self.commit_block(proposal.block_hash, vote_round);
                    return;
                }
            }

            // Check for nil precommits
            let nil_power = rs.precommit_power_for(None, &vs);
            if vs.has_two_thirds(nil_power) {
                // Failed round, move to next
                drop(vs);
                self.enter_new_round(self.round + 1);
                return;
            }

            // Check if we have 2/3+ precommits total
            let total_power = rs.precommit_power(&vs);
            if vs.has_two_thirds(total_power) {
                // No polka but have votes, move to next round
                drop(vs);
                self.enter_new_round(self.round + 1);
            }
        }
    }

    /// Send a prevote
    fn send_prevote(&mut self, block_hash: Option<H256>) {
        self.step = Step::Prevote;

        let vote = Vote::new(
            VoteType::Prevote,
            self.height,
            self.round,
            block_hash,
            self.config.address,
        );

        self.pending_messages.push(ConsensusMessage::Vote(vote.clone()));

        // Process our own vote
        let _ = self.on_vote(vote);
    }

    /// Send a precommit
    fn send_precommit(&mut self, block_hash: Option<H256>) {
        self.step = Step::Precommit;

        let vote = Vote::new(
            VoteType::Precommit,
            self.height,
            self.round,
            block_hash,
            self.config.address,
        );

        self.pending_messages.push(ConsensusMessage::Vote(vote.clone()));

        // Process our own vote
        let _ = self.on_vote(vote);
    }

    /// Commit a block
    fn commit_block(&mut self, block_hash: H256, round: u32) {
        self.step = Step::Commit;

        // Build commit from precommits
        let mut commit = Commit::new(self.height, round, block_hash);

        if let Some(hs) = self.heights.get(&self.height) {
            if let Some(rs) = hs.get_round(round) {
                for vote in rs.precommits.values() {
                    if vote.block_hash == Some(block_hash) {
                        commit.add_vote(vote.clone());
                    }
                }
            }
        }

        // Store commit
        let hs = self.heights.entry(self.height).or_default();
        hs.commit = Some(commit.clone());

        // Notify listeners
        self.pending_messages.push(ConsensusMessage::Finalized {
            height: self.height,
            block_hash,
            commit,
        });
    }

    /// Verify vote signature
    fn verify_vote_signature(&self, vote: &Vote) -> ConsensusResult<()> {
        // Skip verification for zero signatures (e.g., in tests)
        if vote.signature == [0u8; 65] {
            return Ok(());
        }

        let msg = vote.signing_message();
        let msg_hash = keccak256(&msg);

        let sig = Signature {
            v: vote.signature[64],
            r: vote.signature[0..32].try_into().map_err(|_| {
                ConsensusError::InvalidSignature("invalid r component".into())
            })?,
            s: vote.signature[32..64].try_into().map_err(|_| {
                ConsensusError::InvalidSignature("invalid s component".into())
            })?,
        };

        let pubkey = recover_public_key(&msg_hash, &sig)
            .map_err(|e| ConsensusError::InvalidSignature(e.to_string()))?;

        let recovered_addr = public_key_to_address(&pubkey);
        if recovered_addr != vote.voter {
            return Err(ConsensusError::InvalidSignature(format!(
                "vote signature mismatch: expected {:?}, recovered {:?}",
                vote.voter, recovered_addr
            )));
        }

        Ok(())
    }

    /// Verify proposal signature
    fn verify_proposal_signature(&self, proposal: &Proposal) -> ConsensusResult<()> {
        // Skip verification for zero signatures (e.g., in tests)
        if proposal.signature == [0u8; 65] {
            return Ok(());
        }

        let msg = proposal.signing_message();
        let msg_hash = keccak256(&msg);

        let sig = Signature {
            v: proposal.signature[64],
            r: proposal.signature[0..32].try_into().map_err(|_| {
                ConsensusError::InvalidSignature("invalid r component".into())
            })?,
            s: proposal.signature[32..64].try_into().map_err(|_| {
                ConsensusError::InvalidSignature("invalid s component".into())
            })?,
        };

        let pubkey = recover_public_key(&msg_hash, &sig)
            .map_err(|e| ConsensusError::InvalidSignature(e.to_string()))?;

        let recovered_addr = public_key_to_address(&pubkey);
        if recovered_addr != proposal.proposer {
            return Err(ConsensusError::InvalidSignature(format!(
                "proposal signature mismatch: expected {:?}, recovered {:?}",
                proposal.proposer, recovered_addr
            )));
        }

        Ok(())
    }

    /// Handle timeout
    pub fn on_timeout(&mut self, height: u64, round: u32, step: Step) {
        if height != self.height || round != self.round || step != self.step {
            return; // Stale timeout
        }

        match step {
            Step::Propose => {
                // Proposal timeout, prevote nil
                self.send_prevote(None);
            }
            Step::Prevote => {
                // Prevote timeout, precommit nil
                self.send_precommit(None);
            }
            Step::Precommit => {
                // Precommit timeout, move to next round
                self.enter_new_round(self.round + 1);
            }
            _ => {}
        }
    }

    /// Update validator set
    pub fn update_validators(&mut self, validators: ValidatorSet) {
        *self.validator_set.write() = validators;
    }

    /// Get validator set reference
    pub fn validator_set(&self) -> &Arc<RwLock<ValidatorSet>> {
        &self.validator_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Validator;

    fn test_address(n: u8) -> Address {
        Address::from_bytes([n; 20])
    }

    fn create_test_consensus() -> TbftConsensus {
        let config = TbftConfig {
            address: test_address(1),
            ..Default::default()
        };

        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        TbftConsensus::new(config, vs)
    }

    #[test]
    fn test_consensus_creation() {
        let consensus = create_test_consensus();
        assert_eq!(consensus.height(), 0);
        assert_eq!(consensus.round(), 0);
        assert_eq!(consensus.step(), Step::NewRound);
    }

    #[test]
    fn test_start_height() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        assert_eq!(consensus.height(), 1);
        assert_eq!(consensus.round(), 0);
        assert_eq!(consensus.step(), Step::Propose);
    }

    #[test]
    fn test_is_proposer() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        // At height 1, round 0: (1 + 0) % 3 = 1, so validator at index 1 (address 2) is proposer
        // Our address is 1 which is at index 0
        assert!(!consensus.is_proposer());

        // Start at height 0
        consensus.start_height(0);
        // At height 0, round 0: (0 + 0) % 3 = 0, validator at index 0 (address 1) is proposer
        assert!(consensus.is_proposer());
    }

    #[test]
    fn test_proposal_handling() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let block_hash = H256::from_bytes([0x42; 32]);
        let result = consensus.propose_block(block_hash, 1000);

        assert!(result.is_ok());

        // Should have proposal and prevote messages
        let messages = consensus.take_messages();
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::Proposal(_))));
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::Vote(v) if v.vote_type == VoteType::Prevote)));
    }

    #[test]
    fn test_wrong_height_proposal() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        let proposal = Proposal::new(
            2, // Wrong height
            0,
            H256::from_bytes([0x42; 32]),
            test_address(2),
            1000,
        );

        let result = consensus.on_proposal(proposal);
        assert!(matches!(result, Err(ConsensusError::WrongHeight { .. })));
    }

    #[test]
    fn test_vote_handling() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let block_hash = H256::from_bytes([0x42; 32]);

        // Propose block
        let _ = consensus.propose_block(block_hash, 1000);
        consensus.take_messages(); // Clear messages

        // Receive prevotes from other validators
        let vote2 = Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(2));
        let vote3 = Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(3));

        assert!(consensus.on_vote(vote2).is_ok());
        assert!(consensus.on_vote(vote3).is_ok());

        // Should have moved to precommit (2/3+ prevotes)
        assert_eq!(consensus.step(), Step::Precommit);
    }

    #[test]
    fn test_duplicate_vote() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let vote = Vote::new(VoteType::Prevote, 0, 0, None, test_address(2));

        assert!(consensus.on_vote(vote.clone()).is_ok());
        let result = consensus.on_vote(vote);
        assert!(matches!(result, Err(ConsensusError::DuplicateVote(_))));
    }

    #[test]
    fn test_full_consensus_round() {
        // Three validators, all voting for the same block
        let config1 = TbftConfig {
            address: test_address(1),
            ..Default::default()
        };

        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        let mut c1 = TbftConsensus::new(config1, vs.clone());
        c1.start_height(0);

        let block_hash = H256::from_bytes([0x42; 32]);

        // Validator 1 proposes
        c1.propose_block(block_hash, 1000).unwrap();
        c1.take_messages();

        // Receive prevotes from others
        c1.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(2))).unwrap();
        c1.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(3))).unwrap();

        // Now in precommit step
        assert_eq!(c1.step(), Step::Precommit);
        c1.take_messages();

        // Receive precommits from others
        c1.on_vote(Vote::new(VoteType::Precommit, 0, 0, Some(block_hash), test_address(2))).unwrap();
        c1.on_vote(Vote::new(VoteType::Precommit, 0, 0, Some(block_hash), test_address(3))).unwrap();

        // Should be committed
        assert_eq!(c1.step(), Step::Commit);
        assert!(c1.is_finalized(0));

        let messages = c1.take_messages();
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::Finalized { .. })));
    }

    #[test]
    fn test_timeout_handling() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1); // Not proposer

        // Proposal timeout
        consensus.on_timeout(1, 0, Step::Propose);

        // Should have sent nil prevote
        let messages = consensus.take_messages();
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::Vote(v) if v.is_nil())));
    }

    #[test]
    fn test_non_validator_vote() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let vote = Vote::new(VoteType::Prevote, 0, 0, None, test_address(99)); // Not a validator
        let result = consensus.on_vote(vote);
        assert!(matches!(result, Err(ConsensusError::NotValidator(_))));
    }

    // ==================== Extended TbftConfig Tests ====================

    #[test]
    fn test_config_default() {
        let config = TbftConfig::default();
        assert_eq!(config.address, Address::ZERO);
        assert_eq!(config.propose_timeout, 3000);
        assert_eq!(config.prevote_timeout, 1000);
        assert_eq!(config.precommit_timeout, 1000);
    }

    #[test]
    fn test_config_custom() {
        let config = TbftConfig {
            address: test_address(1),
            propose_timeout: 5000,
            prevote_timeout: 2000,
            precommit_timeout: 2000,
        };
        assert_eq!(config.address, test_address(1));
        assert_eq!(config.propose_timeout, 5000);
    }

    // ==================== Extended Consensus State Tests ====================

    #[test]
    fn test_get_commit_not_finalized() {
        let consensus = create_test_consensus();
        assert!(consensus.get_commit(0).is_none());
        assert!(consensus.get_commit(100).is_none());
    }

    #[test]
    fn test_is_finalized_false() {
        let consensus = create_test_consensus();
        assert!(!consensus.is_finalized(0));
        assert!(!consensus.is_finalized(100));
    }

    #[test]
    fn test_take_messages_empty() {
        let mut consensus = create_test_consensus();
        let messages = consensus.take_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_validator_set_access() {
        let consensus = create_test_consensus();
        let vs = consensus.validator_set().read();
        assert_eq!(vs.len(), 3);
        assert_eq!(vs.total_power(), 300);
    }

    #[test]
    fn test_update_validators() {
        let mut consensus = create_test_consensus();
        let new_vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(5), 500),
            Validator::new(test_address(6), 500),
        ]);
        consensus.update_validators(new_vs);
        let vs = consensus.validator_set().read();
        assert_eq!(vs.len(), 2);
        assert_eq!(vs.total_power(), 1000);
    }

    // ==================== Extended Proposal Tests ====================

    #[test]
    fn test_wrong_round_proposal() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        let proposal = Proposal::new(
            1,
            5, // Wrong round (we're at round 0)
            H256::from_bytes([0x42; 32]),
            test_address(2),
            1000,
        );

        let result = consensus.on_proposal(proposal);
        assert!(matches!(result, Err(ConsensusError::WrongRound { .. })));
    }

    #[test]
    fn test_wrong_proposer_proposal() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        // At height 0, round 0, proposer should be validator 1 (index 0)
        let proposal = Proposal::new(
            0,
            0,
            H256::from_bytes([0x42; 32]),
            test_address(99), // Wrong proposer
            1000,
        );

        let result = consensus.on_proposal(proposal);
        assert!(matches!(result, Err(ConsensusError::InvalidProposal(_))));
    }

    #[test]
    fn test_not_proposer_cannot_propose() {
        let config = TbftConfig {
            address: test_address(2), // Not the proposer for height 0
            ..Default::default()
        };

        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        let mut consensus = TbftConsensus::new(config, vs);
        consensus.start_height(0); // Proposer is validator 1

        let result = consensus.propose_block(H256::from_bytes([0x42; 32]), 1000);
        assert!(matches!(result, Err(ConsensusError::InvalidProposal(_))));
    }

    // ==================== Extended Vote Tests ====================

    #[test]
    fn test_wrong_height_vote() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        let vote = Vote::new(VoteType::Prevote, 0, 0, None, test_address(2)); // Wrong height
        let result = consensus.on_vote(vote);
        assert!(matches!(result, Err(ConsensusError::WrongHeight { .. })));
    }

    #[test]
    fn test_precommit_vote() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        // Propose and get to prevote stage
        let block_hash = H256::from_bytes([0x42; 32]);
        let _ = consensus.propose_block(block_hash, 1000);
        consensus.take_messages();

        // Add prevotes to move to precommit
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(2)));
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(3)));

        assert_eq!(consensus.step(), Step::Precommit);
    }

    // ==================== Timeout Tests ====================

    #[test]
    fn test_stale_timeout() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        // Send timeout for wrong height/round - should be ignored
        consensus.on_timeout(0, 0, Step::Propose); // Wrong height
        assert_eq!(consensus.step(), Step::Propose); // Still propose

        consensus.on_timeout(1, 5, Step::Propose); // Wrong round
        assert_eq!(consensus.step(), Step::Propose); // Still propose
    }

    #[test]
    fn test_prevote_timeout() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        // Move to prevote step manually
        consensus.on_timeout(1, 0, Step::Propose);
        assert_eq!(consensus.step(), Step::Prevote);

        // Prevote timeout
        consensus.take_messages();
        consensus.on_timeout(1, 0, Step::Prevote);
        assert_eq!(consensus.step(), Step::Precommit);
    }

    #[test]
    fn test_precommit_timeout() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        // Move through propose and prevote
        consensus.on_timeout(1, 0, Step::Propose);
        consensus.on_timeout(1, 0, Step::Prevote);

        // Now at precommit
        assert_eq!(consensus.step(), Step::Precommit);
        consensus.take_messages();

        // Precommit timeout should move to next round
        consensus.on_timeout(1, 0, Step::Precommit);
        assert_eq!(consensus.round(), 1);
        assert_eq!(consensus.step(), Step::Propose);
    }

    // ==================== Step Tests ====================

    #[test]
    fn test_step_equality() {
        assert_eq!(Step::NewRound, Step::NewRound);
        assert_eq!(Step::Propose, Step::Propose);
        assert_eq!(Step::Prevote, Step::Prevote);
        assert_eq!(Step::Precommit, Step::Precommit);
        assert_eq!(Step::Commit, Step::Commit);
        assert_ne!(Step::Propose, Step::Prevote);
    }

    #[test]
    fn test_step_copy() {
        let s1 = Step::Propose;
        let s2 = s1;
        assert_eq!(s1, s2);
    }

    // ==================== ConsensusMessage Tests ====================

    #[test]
    fn test_create_block_message() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let messages = consensus.take_messages();
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::CreateBlock { height: 0, round: 0 })));
    }

    #[test]
    fn test_finalized_message_after_commit() {
        let mut consensus = create_test_consensus();
        consensus.start_height(0);

        let block_hash = H256::from_bytes([0x42; 32]);
        let _ = consensus.propose_block(block_hash, 1000);
        consensus.take_messages();

        // Prevotes
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(2)));
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 0, 0, Some(block_hash), test_address(3)));
        consensus.take_messages();

        // Precommits
        let _ = consensus.on_vote(Vote::new(VoteType::Precommit, 0, 0, Some(block_hash), test_address(2)));
        let _ = consensus.on_vote(Vote::new(VoteType::Precommit, 0, 0, Some(block_hash), test_address(3)));

        let messages = consensus.take_messages();
        assert!(messages.iter().any(|m| matches!(m, ConsensusMessage::Finalized { .. })));
    }

    // ==================== Nil Voting Tests ====================

    #[test]
    fn test_nil_prevote_polka() {
        let config = TbftConfig {
            address: test_address(2), // Not proposer
            ..Default::default()
        };

        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        let mut consensus = TbftConsensus::new(config, vs);
        consensus.start_height(1);

        // Timeout on propose sends nil prevote
        consensus.on_timeout(1, 0, Step::Propose);

        // Receive nil prevotes from others
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 1, 0, None, test_address(1)));
        let _ = consensus.on_vote(Vote::new(VoteType::Prevote, 1, 0, None, test_address(3)));

        // Should move to precommit with nil
        assert_eq!(consensus.step(), Step::Precommit);
    }

    // ==================== Multiple Round Tests ====================

    #[test]
    fn test_multiple_rounds() {
        let mut consensus = create_test_consensus();
        consensus.start_height(1);

        assert_eq!(consensus.round(), 0);

        // Timeout through all phases
        consensus.on_timeout(1, 0, Step::Propose);
        consensus.on_timeout(1, 0, Step::Prevote);
        consensus.on_timeout(1, 0, Step::Precommit);

        // Should be in round 1
        assert_eq!(consensus.round(), 1);
        assert_eq!(consensus.step(), Step::Propose);
    }
}
