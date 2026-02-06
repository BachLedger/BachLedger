//! Consensus types

use bach_primitives::{Address, H256};

/// Vote type in TBFT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoteType {
    /// Prevote - first voting round
    Prevote,
    /// Precommit - second voting round
    Precommit,
}

/// A vote in the consensus protocol
#[derive(Debug, Clone)]
pub struct Vote {
    /// Vote type
    pub vote_type: VoteType,
    /// Block height
    pub height: u64,
    /// Consensus round
    pub round: u32,
    /// Block hash being voted on (None for nil vote)
    pub block_hash: Option<H256>,
    /// Voter address
    pub voter: Address,
    /// Signature (r, s, v)
    pub signature: [u8; 65],
}

impl Vote {
    /// Create a new vote
    pub fn new(
        vote_type: VoteType,
        height: u64,
        round: u32,
        block_hash: Option<H256>,
        voter: Address,
    ) -> Self {
        Self {
            vote_type,
            height,
            round,
            block_hash,
            voter,
            signature: [0u8; 65],
        }
    }

    /// Set signature
    pub fn with_signature(mut self, signature: [u8; 65]) -> Self {
        self.signature = signature;
        self
    }

    /// Check if this is a nil vote
    pub fn is_nil(&self) -> bool {
        self.block_hash.is_none()
    }

    /// Get signing message
    pub fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.push(self.vote_type as u8);
        msg.extend_from_slice(&self.height.to_le_bytes());
        msg.extend_from_slice(&self.round.to_le_bytes());
        if let Some(hash) = &self.block_hash {
            msg.extend_from_slice(hash.as_bytes());
        }
        msg
    }
}

/// Block proposal
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Block height
    pub height: u64,
    /// Consensus round
    pub round: u32,
    /// Block hash
    pub block_hash: H256,
    /// Proposer address
    pub proposer: Address,
    /// Proposal timestamp
    pub timestamp: u64,
    /// Signature
    pub signature: [u8; 65],
}

impl Proposal {
    /// Create a new proposal
    pub fn new(height: u64, round: u32, block_hash: H256, proposer: Address, timestamp: u64) -> Self {
        Self {
            height,
            round,
            block_hash,
            proposer,
            timestamp,
            signature: [0u8; 65],
        }
    }

    /// Set signature
    pub fn with_signature(mut self, signature: [u8; 65]) -> Self {
        self.signature = signature;
        self
    }

    /// Get signing message
    pub fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.height.to_le_bytes());
        msg.extend_from_slice(&self.round.to_le_bytes());
        msg.extend_from_slice(self.block_hash.as_bytes());
        msg.extend_from_slice(&self.timestamp.to_le_bytes());
        msg
    }
}

/// Validator info
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    /// Validator address
    pub address: Address,
    /// Voting power
    pub voting_power: u64,
}

impl Validator {
    /// Create a new validator
    pub fn new(address: Address, voting_power: u64) -> Self {
        Self {
            address,
            voting_power,
        }
    }
}

/// Validator set
#[derive(Debug, Clone, Default)]
pub struct ValidatorSet {
    /// List of validators
    validators: Vec<Validator>,
    /// Total voting power
    total_power: u64,
}

impl ValidatorSet {
    /// Create an empty validator set
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a list of validators
    pub fn from_validators(validators: Vec<Validator>) -> Self {
        let total_power = validators.iter().map(|v| v.voting_power).sum();
        Self {
            validators,
            total_power,
        }
    }

    /// Add a validator
    pub fn add(&mut self, validator: Validator) {
        self.total_power += validator.voting_power;
        self.validators.push(validator);
    }

    /// Get validator by address
    pub fn get(&self, address: &Address) -> Option<&Validator> {
        self.validators.iter().find(|v| &v.address == address)
    }

    /// Check if address is a validator
    pub fn contains(&self, address: &Address) -> bool {
        self.validators.iter().any(|v| &v.address == address)
    }

    /// Get total voting power
    pub fn total_power(&self) -> u64 {
        self.total_power
    }

    /// Get number of validators
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Get all validators
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Get the proposer for a given height and round
    pub fn proposer(&self, height: u64, round: u32) -> Option<&Validator> {
        if self.validators.is_empty() {
            return None;
        }
        // Simple round-robin based on height + round
        let index = ((height + round as u64) as usize) % self.validators.len();
        Some(&self.validators[index])
    }

    /// Calculate if we have 2/3+ voting power
    pub fn has_two_thirds(&self, power: u64) -> bool {
        // 2/3 majority means > 2/3 of total power
        power * 3 > self.total_power * 2
    }

    /// Calculate if we have 1/3+ voting power (for liveness)
    pub fn has_one_third(&self, power: u64) -> bool {
        power * 3 >= self.total_power
    }
}

/// Commit (aggregated precommit votes for a block)
#[derive(Debug, Clone)]
pub struct Commit {
    /// Block height
    pub height: u64,
    /// Round
    pub round: u32,
    /// Block hash
    pub block_hash: H256,
    /// Precommit votes
    pub votes: Vec<Vote>,
}

impl Commit {
    /// Create a new commit
    pub fn new(height: u64, round: u32, block_hash: H256) -> Self {
        Self {
            height,
            round,
            block_hash,
            votes: Vec::new(),
        }
    }

    /// Add a vote
    pub fn add_vote(&mut self, vote: Vote) {
        self.votes.push(vote);
    }

    /// Get total voting power in this commit
    pub fn voting_power(&self, validator_set: &ValidatorSet) -> u64 {
        self.votes
            .iter()
            .filter_map(|v| validator_set.get(&v.voter))
            .map(|v| v.voting_power)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_address(n: u8) -> Address {
        Address::from_bytes([n; 20])
    }

    #[test]
    fn test_vote_creation() {
        let voter = test_address(1);
        let block_hash = H256::from_bytes([0x42; 32]);

        let vote = Vote::new(VoteType::Prevote, 1, 0, Some(block_hash), voter);

        assert_eq!(vote.height, 1);
        assert_eq!(vote.round, 0);
        assert!(!vote.is_nil());
        assert_eq!(vote.block_hash, Some(block_hash));
    }

    #[test]
    fn test_nil_vote() {
        let voter = test_address(1);
        let vote = Vote::new(VoteType::Prevote, 1, 0, None, voter);
        assert!(vote.is_nil());
    }

    #[test]
    fn test_validator_set() {
        let mut vs = ValidatorSet::new();

        vs.add(Validator::new(test_address(1), 100));
        vs.add(Validator::new(test_address(2), 100));
        vs.add(Validator::new(test_address(3), 100));

        assert_eq!(vs.len(), 3);
        assert_eq!(vs.total_power(), 300);
        assert!(vs.contains(&test_address(1)));
        assert!(!vs.contains(&test_address(4)));
    }

    #[test]
    fn test_two_thirds_majority() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        // Total power = 300
        // 2/3 of 300 = 200, need > 200
        assert!(!vs.has_two_thirds(200)); // Exactly 2/3 not enough
        assert!(vs.has_two_thirds(201));  // > 2/3 is enough
        assert!(vs.has_two_thirds(300));  // All votes
    }

    #[test]
    fn test_proposer_selection() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);

        // Round-robin based on (height + round) % len
        let p0 = vs.proposer(0, 0).unwrap();
        let p1 = vs.proposer(0, 1).unwrap();
        let p2 = vs.proposer(0, 2).unwrap();
        let p3 = vs.proposer(0, 3).unwrap();

        assert_eq!(p0.address, test_address(1)); // (0+0) % 3 = 0
        assert_eq!(p1.address, test_address(2)); // (0+1) % 3 = 1
        assert_eq!(p2.address, test_address(3)); // (0+2) % 3 = 2
        assert_eq!(p3.address, test_address(1)); // (0+3) % 3 = 0, wraps
    }

    #[test]
    fn test_commit() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
        ]);

        let block_hash = H256::from_bytes([0x42; 32]);
        let mut commit = Commit::new(1, 0, block_hash);

        commit.add_vote(Vote::new(VoteType::Precommit, 1, 0, Some(block_hash), test_address(1)));
        commit.add_vote(Vote::new(VoteType::Precommit, 1, 0, Some(block_hash), test_address(2)));

        assert_eq!(commit.voting_power(&vs), 200);
    }

    #[test]
    fn test_vote_signing_message() {
        let voter = test_address(1);
        let block_hash = H256::from_bytes([0x42; 32]);

        let vote = Vote::new(VoteType::Prevote, 1, 0, Some(block_hash), voter);
        let msg = vote.signing_message();

        // Should contain vote_type byte + height (8) + round (4) + block_hash (32)
        assert_eq!(msg.len(), 1 + 8 + 4 + 32);
    }

    #[test]
    fn test_proposal_signing_message() {
        let proposer = test_address(1);
        let block_hash = H256::from_bytes([0x42; 32]);

        let proposal = Proposal::new(1, 0, block_hash, proposer, 1000);
        let msg = proposal.signing_message();

        // height (8) + round (4) + block_hash (32) + timestamp (8)
        assert_eq!(msg.len(), 8 + 4 + 32 + 8);
    }

    // ==================== Extended Vote Tests ====================

    #[test]
    fn test_vote_with_signature() {
        let voter = test_address(1);
        let sig = [0xab; 65];
        let vote = Vote::new(VoteType::Prevote, 1, 0, None, voter).with_signature(sig);
        assert_eq!(vote.signature, sig);
    }

    #[test]
    fn test_vote_types() {
        let voter = test_address(1);
        let prevote = Vote::new(VoteType::Prevote, 1, 0, None, voter);
        let precommit = Vote::new(VoteType::Precommit, 1, 0, None, voter);
        assert_eq!(prevote.vote_type, VoteType::Prevote);
        assert_eq!(precommit.vote_type, VoteType::Precommit);
    }

    #[test]
    fn test_vote_signing_message_nil() {
        let voter = test_address(1);
        let vote = Vote::new(VoteType::Prevote, 1, 0, None, voter);
        let msg = vote.signing_message();
        // No block hash, so shorter
        assert_eq!(msg.len(), 1 + 8 + 4);
    }

    #[test]
    fn test_vote_clone() {
        let voter = test_address(1);
        let vote = Vote::new(VoteType::Prevote, 5, 2, Some(H256::from_bytes([0x42; 32])), voter);
        let cloned = vote.clone();
        assert_eq!(vote.height, cloned.height);
        assert_eq!(vote.round, cloned.round);
        assert_eq!(vote.block_hash, cloned.block_hash);
    }

    // ==================== Extended Proposal Tests ====================

    #[test]
    fn test_proposal_with_signature() {
        let proposer = test_address(1);
        let block_hash = H256::from_bytes([0x42; 32]);
        let sig = [0xcd; 65];
        let proposal = Proposal::new(1, 0, block_hash, proposer, 1000).with_signature(sig);
        assert_eq!(proposal.signature, sig);
    }

    #[test]
    fn test_proposal_clone() {
        let proposer = test_address(1);
        let block_hash = H256::from_bytes([0x42; 32]);
        let proposal = Proposal::new(5, 3, block_hash, proposer, 2000);
        let cloned = proposal.clone();
        assert_eq!(proposal.height, cloned.height);
        assert_eq!(proposal.round, cloned.round);
        assert_eq!(proposal.block_hash, cloned.block_hash);
        assert_eq!(proposal.timestamp, cloned.timestamp);
    }

    // ==================== Extended ValidatorSet Tests ====================

    #[test]
    fn test_validator_set_empty() {
        let vs = ValidatorSet::new();
        assert!(vs.is_empty());
        assert_eq!(vs.len(), 0);
        assert_eq!(vs.total_power(), 0);
        assert!(vs.proposer(0, 0).is_none());
    }

    #[test]
    fn test_validator_set_from_validators() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 50),
            Validator::new(test_address(2), 100),
        ]);
        assert_eq!(vs.len(), 2);
        assert_eq!(vs.total_power(), 150);
    }

    #[test]
    fn test_validator_set_get() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
        ]);
        let v = vs.get(&test_address(1)).unwrap();
        assert_eq!(v.voting_power, 100);
        assert!(vs.get(&test_address(99)).is_none());
    }

    #[test]
    fn test_validator_set_validators() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 200),
        ]);
        let validators = vs.validators();
        assert_eq!(validators.len(), 2);
    }

    #[test]
    fn test_one_third_majority() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);
        // Total power = 300, 1/3 = 100
        assert!(vs.has_one_third(100));
        assert!(!vs.has_one_third(99));
    }

    #[test]
    fn test_validator_equality() {
        let v1 = Validator::new(test_address(1), 100);
        let v2 = Validator::new(test_address(1), 100);
        let v3 = Validator::new(test_address(2), 100);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_proposer_rotation_height() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 100),
            Validator::new(test_address(3), 100),
        ]);
        // Different heights should select different proposers
        let p_h0 = vs.proposer(0, 0).unwrap().address;
        let p_h1 = vs.proposer(1, 0).unwrap().address;
        let p_h2 = vs.proposer(2, 0).unwrap().address;
        assert_eq!(p_h0, test_address(1));
        assert_eq!(p_h1, test_address(2));
        assert_eq!(p_h2, test_address(3));
    }

    // ==================== Extended Commit Tests ====================

    #[test]
    fn test_commit_empty() {
        let block_hash = H256::from_bytes([0x42; 32]);
        let commit = Commit::new(1, 0, block_hash);
        assert_eq!(commit.height, 1);
        assert_eq!(commit.round, 0);
        assert!(commit.votes.is_empty());
    }

    #[test]
    fn test_commit_voting_power_empty() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
        ]);
        let block_hash = H256::from_bytes([0x42; 32]);
        let commit = Commit::new(1, 0, block_hash);
        assert_eq!(commit.voting_power(&vs), 0);
    }

    #[test]
    fn test_commit_voting_power_partial() {
        let vs = ValidatorSet::from_validators(vec![
            Validator::new(test_address(1), 100),
            Validator::new(test_address(2), 200),
            Validator::new(test_address(3), 300),
        ]);
        let block_hash = H256::from_bytes([0x42; 32]);
        let mut commit = Commit::new(1, 0, block_hash);
        // Only add votes from validators 1 and 2
        commit.add_vote(Vote::new(VoteType::Precommit, 1, 0, Some(block_hash), test_address(1)));
        commit.add_vote(Vote::new(VoteType::Precommit, 1, 0, Some(block_hash), test_address(2)));
        assert_eq!(commit.voting_power(&vs), 300); // 100 + 200
    }

    #[test]
    fn test_commit_clone() {
        let block_hash = H256::from_bytes([0x42; 32]);
        let mut commit = Commit::new(5, 2, block_hash);
        commit.add_vote(Vote::new(VoteType::Precommit, 5, 2, Some(block_hash), test_address(1)));
        let cloned = commit.clone();
        assert_eq!(commit.height, cloned.height);
        assert_eq!(commit.round, cloned.round);
        assert_eq!(commit.votes.len(), cloned.votes.len());
    }

    // ==================== VoteType Tests ====================

    #[test]
    fn test_vote_type_eq() {
        assert_eq!(VoteType::Prevote, VoteType::Prevote);
        assert_eq!(VoteType::Precommit, VoteType::Precommit);
        assert_ne!(VoteType::Prevote, VoteType::Precommit);
    }

    #[test]
    fn test_vote_type_copy() {
        let t1 = VoteType::Prevote;
        let t2 = t1; // Copy
        assert_eq!(t1, t2);
    }
}
