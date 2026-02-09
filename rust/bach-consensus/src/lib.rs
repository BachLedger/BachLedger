//! BachLedger Consensus
//!
//! TBFT (Tendermint BFT) consensus implementation for medical blockchain.
//!
//! # Protocol Phases
//! 1. **Propose**: Leader broadcasts block proposal
//! 2. **Pre-vote**: Validators vote for valid proposals
//! 3. **Pre-commit**: After 2/3+ pre-votes, validators pre-commit
//! 4. **Commit**: After 2/3+ pre-commits, block is finalized
//!
//! # Byzantine Fault Tolerance
//! Tolerates up to f faulty validators where n > 3f + 1

#![forbid(unsafe_code)]

use bach_crypto::{keccak256, keccak256_concat, PrivateKey, PublicKey, Signature};
use bach_primitives::{Address, H256};
use bach_types::{Block, Transaction};
use std::collections::HashMap;

/// Consensus errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusError {
    /// Validator is not in the set
    UnknownValidator(Address),
    /// Invalid signature on message
    InvalidSignature,
    /// Message is for wrong height
    WrongHeight { expected: u64, actual: u64 },
    /// Message is for wrong round
    WrongRound { expected: u32, actual: u32 },
    /// Not the proposer for this round
    NotProposer,
    /// Duplicate vote from validator
    DuplicateVote(Address),
    /// Invalid proposal
    InvalidProposal(String),
    /// No proposal to vote on
    NoProposal,
}

/// A validator in the consensus set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    /// Validator's address (derived from public key)
    pub address: Address,
    /// Validator's public key for signature verification
    pub public_key: PublicKey,
    /// Voting power (stake weight)
    pub voting_power: u64,
}

impl Validator {
    /// Creates a new validator.
    pub fn new(public_key: PublicKey, voting_power: u64) -> Self {
        let address = public_key.to_address();
        Self {
            address,
            public_key,
            voting_power,
        }
    }
}

/// The set of validators participating in consensus.
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    validators: Vec<Validator>,
    total_voting_power: u64,
    address_to_index: HashMap<Address, usize>,
}

impl ValidatorSet {
    /// Creates a new validator set.
    pub fn new(validators: Vec<Validator>) -> Self {
        let total_voting_power = validators.iter().map(|v| v.voting_power).sum();
        let address_to_index = validators
            .iter()
            .enumerate()
            .map(|(i, v)| (v.address, i))
            .collect();

        Self {
            validators,
            total_voting_power,
            address_to_index,
        }
    }

    /// Returns the total voting power.
    pub fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    /// Returns the number of validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Gets a validator by address.
    pub fn get(&self, address: &Address) -> Option<&Validator> {
        self.address_to_index
            .get(address)
            .map(|&i| &self.validators[i])
    }

    /// Gets the proposer for a given height and round using round-robin.
    pub fn get_proposer(&self, height: u64, round: u32) -> &Validator {
        let index = ((height as usize) + (round as usize)) % self.validators.len();
        &self.validators[index]
    }

    /// Returns the voting power needed for a quorum (2/3+).
    /// Uses ceiling division to ensure strictly more than 2/3.
    pub fn quorum_power(&self) -> u64 {
        (2 * self.total_voting_power).div_ceil(3)
    }

    /// Checks if the given voting power meets the quorum threshold.
    pub fn has_quorum(&self, voting_power: u64) -> bool {
        voting_power >= self.quorum_power()
    }

    /// Returns all validators.
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Checks if an address is a validator.
    pub fn contains(&self, address: &Address) -> bool {
        self.address_to_index.contains_key(address)
    }
}

/// Current phase of the consensus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusStep {
    /// Waiting to start a new height
    NewHeight,
    /// Waiting for/creating a proposal
    Propose,
    /// Collecting pre-votes
    PreVote,
    /// Collecting pre-commits
    PreCommit,
    /// Block has been committed
    Commit,
}

/// A pre-vote message.
#[derive(Debug, Clone)]
pub struct PreVote {
    pub height: u64,
    pub round: u32,
    /// None means voting for nil (no valid proposal)
    pub block_hash: Option<H256>,
    pub validator: Address,
    pub signature: Signature,
}

impl PreVote {
    /// Computes the signing hash for this pre-vote.
    pub fn signing_hash(&self) -> H256 {
        let mut data = Vec::new();
        data.push(0x01); // message type: prevote
        data.extend_from_slice(&self.height.to_be_bytes());
        data.extend_from_slice(&self.round.to_be_bytes());
        if let Some(hash) = &self.block_hash {
            data.push(1);
            data.extend_from_slice(hash.as_bytes());
        } else {
            data.push(0);
        }
        data.extend_from_slice(self.validator.as_bytes());
        keccak256(&data)
    }

    /// Verifies the signature.
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let hash = self.signing_hash();
        self.signature.verify(public_key, &hash)
    }
}

/// A pre-commit message.
#[derive(Debug, Clone)]
pub struct PreCommit {
    pub height: u64,
    pub round: u32,
    /// None means committing to nil
    pub block_hash: Option<H256>,
    pub validator: Address,
    pub signature: Signature,
}

impl PreCommit {
    /// Computes the signing hash for this pre-commit.
    pub fn signing_hash(&self) -> H256 {
        let mut data = Vec::new();
        data.push(0x02); // message type: precommit
        data.extend_from_slice(&self.height.to_be_bytes());
        data.extend_from_slice(&self.round.to_be_bytes());
        if let Some(hash) = &self.block_hash {
            data.push(1);
            data.extend_from_slice(hash.as_bytes());
        } else {
            data.push(0);
        }
        data.extend_from_slice(self.validator.as_bytes());
        keccak256(&data)
    }

    /// Verifies the signature.
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let hash = self.signing_hash();
        self.signature.verify(public_key, &hash)
    }
}

/// A block proposal message.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub height: u64,
    pub round: u32,
    pub block: Block,
    pub proposer: Address,
    pub signature: Signature,
}

impl Proposal {
    /// Computes the signing hash for this proposal.
    pub fn signing_hash(&self) -> H256 {
        let block_hash = self.block.hash();
        keccak256_concat(&[
            &[0x00], // message type: proposal
            &self.height.to_be_bytes(),
            &self.round.to_be_bytes(),
            block_hash.as_bytes(),
            self.proposer.as_bytes(),
        ])
    }

    /// Verifies the signature.
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let hash = self.signing_hash();
        self.signature.verify(public_key, &hash)
    }
}

/// Messages exchanged in the consensus protocol.
#[derive(Debug, Clone)]
pub enum ConsensusMessage {
    /// Block proposal from the leader
    Proposal(Proposal),
    /// Pre-vote for a block
    PreVote(PreVote),
    /// Pre-commit for a block
    PreCommit(PreCommit),
}

impl ConsensusMessage {
    /// Returns the height of this message.
    pub fn height(&self) -> u64 {
        match self {
            ConsensusMessage::Proposal(p) => p.height,
            ConsensusMessage::PreVote(v) => v.height,
            ConsensusMessage::PreCommit(c) => c.height,
        }
    }

    /// Returns the round of this message.
    pub fn round(&self) -> u32 {
        match self {
            ConsensusMessage::Proposal(p) => p.round,
            ConsensusMessage::PreVote(v) => v.round,
            ConsensusMessage::PreCommit(c) => c.round,
        }
    }
}

/// The current consensus state.
#[derive(Debug)]
pub struct ConsensusState {
    /// Current block height
    pub height: u64,
    /// Current round within the height
    pub round: u32,
    /// Current step in the protocol
    pub step: ConsensusStep,
    /// The proposal for this round (if received)
    pub proposal: Option<Proposal>,
    /// Pre-votes received for this round
    pub prevotes: HashMap<Address, PreVote>,
    /// Pre-commits received for this round
    pub precommits: HashMap<Address, PreCommit>,
    /// Block we've locked on (after seeing 2/3+ pre-votes)
    pub locked_block: Option<Block>,
    /// Round at which we locked
    pub locked_round: Option<u32>,
    /// Block that was committed (finalized)
    pub committed_block: Option<Block>,
}

impl ConsensusState {
    /// Creates a new consensus state starting at the given height.
    pub fn new(height: u64) -> Self {
        Self {
            height,
            round: 0,
            step: ConsensusStep::NewHeight,
            proposal: None,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            locked_block: None,
            locked_round: None,
            committed_block: None,
        }
    }

    /// Returns the current height.
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Returns the current round.
    pub fn round(&self) -> u32 {
        self.round
    }

    /// Returns the current step.
    pub fn step(&self) -> ConsensusStep {
        self.step
    }

    /// Returns the locked block, if any.
    pub fn locked_block(&self) -> Option<&Block> {
        self.locked_block.as_ref()
    }

    /// Returns the committed block, if any.
    pub fn committed_block(&self) -> Option<&Block> {
        self.committed_block.as_ref()
    }

    /// Returns the current proposal.
    pub fn proposal(&self) -> Option<&Proposal> {
        self.proposal.as_ref()
    }

    /// Moves to the next round.
    fn next_round(&mut self) {
        self.round += 1;
        self.step = ConsensusStep::Propose;
        self.proposal = None;
        self.prevotes.clear();
        self.precommits.clear();
        // Note: locked_block and locked_round persist across rounds
    }

    /// Moves to the next height.
    fn next_height(&mut self) {
        self.height += 1;
        self.round = 0;
        self.step = ConsensusStep::NewHeight;
        self.proposal = None;
        self.prevotes.clear();
        self.precommits.clear();
        self.locked_block = None;
        self.locked_round = None;
        self.committed_block = None;
    }
}

/// TBFT consensus engine.
pub struct TbftConsensus {
    /// The validator set
    validator_set: ValidatorSet,
    /// Our private key for signing
    private_key: PrivateKey,
    /// Our address (derived from private key)
    our_address: Address,
    /// Current consensus state
    state: ConsensusState,
}

impl TbftConsensus {
    /// Creates a new TBFT consensus instance.
    pub fn new(validator_set: ValidatorSet, private_key: PrivateKey) -> Self {
        let our_address = private_key.public_key().to_address();
        Self {
            validator_set,
            private_key,
            our_address,
            state: ConsensusState::new(0),
        }
    }

    /// Returns our validator address.
    pub fn our_address(&self) -> &Address {
        &self.our_address
    }

    /// Returns the current consensus state.
    pub fn state(&self) -> &ConsensusState {
        &self.state
    }

    /// Returns a mutable reference to the consensus state.
    /// Primarily for testing purposes.
    pub fn state_mut(&mut self) -> &mut ConsensusState {
        &mut self.state
    }

    /// Returns the validator set.
    pub fn validator_set(&self) -> &ValidatorSet {
        &self.validator_set
    }

    /// Starts consensus at a new height.
    pub fn start_height(&mut self, height: u64) -> Vec<ConsensusMessage> {
        self.state = ConsensusState::new(height);
        self.state.step = ConsensusStep::Propose;
        Vec::new()
    }

    /// Creates a proposal if we are the proposer for this round.
    ///
    /// Returns None if we are not the proposer.
    pub fn create_proposal(
        &mut self,
        transactions: Vec<Transaction>,
        parent_hash: H256,
        timestamp: u64,
    ) -> Option<ConsensusMessage> {
        // Check if we are the proposer
        let proposer = self.validator_set.get_proposer(self.state.height, self.state.round);
        if proposer.address != self.our_address {
            return None;
        }

        // If we're locked on a block, propose that block
        let block = if let Some(locked) = &self.state.locked_block {
            locked.clone()
        } else {
            Block::new(self.state.height, parent_hash, transactions, timestamp)
        };

        // Compute signing hash for the proposal
        let block_hash = block.hash();
        let signing_hash = keccak256_concat(&[
            &[0x00], // message type: proposal
            &self.state.height.to_be_bytes(),
            &self.state.round.to_be_bytes(),
            block_hash.as_bytes(),
            self.our_address.as_bytes(),
        ]);

        let signature = self.private_key.sign(&signing_hash);

        let signed_proposal = Proposal {
            height: self.state.height,
            round: self.state.round,
            block,
            proposer: self.our_address,
            signature,
        };

        self.state.proposal = Some(signed_proposal.clone());
        self.state.step = ConsensusStep::PreVote;

        Some(ConsensusMessage::Proposal(signed_proposal))
    }

    /// Handles an incoming consensus message.
    ///
    /// Returns any messages we should broadcast in response.
    pub fn handle_message(
        &mut self,
        msg: ConsensusMessage,
    ) -> Result<Vec<ConsensusMessage>, ConsensusError> {
        match msg {
            ConsensusMessage::Proposal(proposal) => self.handle_proposal(proposal),
            ConsensusMessage::PreVote(prevote) => self.handle_prevote(prevote),
            ConsensusMessage::PreCommit(precommit) => self.handle_precommit(precommit),
        }
    }

    /// Handles a proposal message.
    fn handle_proposal(&mut self, proposal: Proposal) -> Result<Vec<ConsensusMessage>, ConsensusError> {
        // Verify height
        if proposal.height != self.state.height {
            return Err(ConsensusError::WrongHeight {
                expected: self.state.height,
                actual: proposal.height,
            });
        }

        // Verify round
        if proposal.round != self.state.round {
            return Err(ConsensusError::WrongRound {
                expected: self.state.round,
                actual: proposal.round,
            });
        }

        // Verify proposer is correct for this round
        let expected_proposer = self.validator_set.get_proposer(proposal.height, proposal.round);
        if proposal.proposer != expected_proposer.address {
            return Err(ConsensusError::NotProposer);
        }

        // Verify signature
        let validator = self
            .validator_set
            .get(&proposal.proposer)
            .ok_or(ConsensusError::UnknownValidator(proposal.proposer))?;

        if !proposal.verify(&validator.public_key) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Verify block height matches
        if proposal.block.height != proposal.height {
            return Err(ConsensusError::InvalidProposal(
                "Block height mismatch".to_string(),
            ));
        }

        // Store the proposal
        self.state.proposal = Some(proposal.clone());

        // Move to pre-vote step if we haven't already
        if self.state.step == ConsensusStep::Propose {
            self.state.step = ConsensusStep::PreVote;
        }

        // If we are a validator, send our pre-vote
        let mut messages = Vec::new();
        if self.validator_set.contains(&self.our_address) {
            // Decide what to vote for
            let block_hash = self.decide_prevote(&proposal);

            let prevote = self.create_prevote(block_hash);
            messages.push(ConsensusMessage::PreVote(prevote));
        }

        Ok(messages)
    }

    /// Decides what block hash to pre-vote for.
    fn decide_prevote(&self, proposal: &Proposal) -> Option<H256> {
        // If we're locked on a different block, vote nil
        if let Some(locked) = &self.state.locked_block {
            let locked_hash = locked.hash();
            let proposal_hash = proposal.block.hash();
            if locked_hash != proposal_hash {
                // Locked on different block - vote nil unless proposal is for higher round
                if let Some(locked_round) = self.state.locked_round {
                    if proposal.round <= locked_round {
                        return None; // Vote nil
                    }
                }
            }
        }

        // Vote for the proposed block
        Some(proposal.block.hash())
    }

    /// Creates and signs a pre-vote message.
    /// Also stores the prevote in our own state.
    pub fn create_prevote(&mut self, block_hash: Option<H256>) -> PreVote {
        // Compute signing hash
        let mut data = Vec::new();
        data.push(0x01); // message type: prevote
        data.extend_from_slice(&self.state.height.to_be_bytes());
        data.extend_from_slice(&self.state.round.to_be_bytes());
        if let Some(hash) = &block_hash {
            data.push(1);
            data.extend_from_slice(hash.as_bytes());
        } else {
            data.push(0);
        }
        data.extend_from_slice(self.our_address.as_bytes());
        let signing_hash = keccak256(&data);

        let signature = self.private_key.sign(&signing_hash);

        let prevote = PreVote {
            height: self.state.height,
            round: self.state.round,
            block_hash,
            validator: self.our_address,
            signature,
        };

        // Store our own prevote
        self.state.prevotes.insert(self.our_address, prevote.clone());

        prevote
    }

    /// Handles a pre-vote message.
    fn handle_prevote(&mut self, prevote: PreVote) -> Result<Vec<ConsensusMessage>, ConsensusError> {
        // Verify height
        if prevote.height != self.state.height {
            return Err(ConsensusError::WrongHeight {
                expected: self.state.height,
                actual: prevote.height,
            });
        }

        // Verify round
        if prevote.round != self.state.round {
            return Err(ConsensusError::WrongRound {
                expected: self.state.round,
                actual: prevote.round,
            });
        }

        // Verify validator is in the set
        let validator = self
            .validator_set
            .get(&prevote.validator)
            .ok_or(ConsensusError::UnknownValidator(prevote.validator))?;

        // Check for duplicate
        if self.state.prevotes.contains_key(&prevote.validator) {
            return Err(ConsensusError::DuplicateVote(prevote.validator));
        }

        // Verify signature
        if !prevote.verify(&validator.public_key) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Store the pre-vote
        self.state.prevotes.insert(prevote.validator, prevote);

        // Check if we have quorum
        let mut messages = Vec::new();
        if self.check_prevote_quorum() {
            messages.extend(self.on_prevote_quorum()?);
        }

        Ok(messages)
    }

    /// Checks if we have 2/3+ pre-votes for any block.
    fn check_prevote_quorum(&self) -> bool {
        // Count voting power for each block hash
        let mut power_by_hash: HashMap<Option<H256>, u64> = HashMap::new();

        for (addr, prevote) in &self.state.prevotes {
            if let Some(validator) = self.validator_set.get(addr) {
                *power_by_hash.entry(prevote.block_hash).or_insert(0) += validator.voting_power;
            }
        }

        // Check if any has quorum
        for (_, power) in power_by_hash {
            if self.validator_set.has_quorum(power) {
                return true;
            }
        }

        false
    }

    /// Called when we have 2/3+ pre-votes.
    fn on_prevote_quorum(&mut self) -> Result<Vec<ConsensusMessage>, ConsensusError> {
        // Find which block has quorum
        let mut power_by_hash: HashMap<Option<H256>, u64> = HashMap::new();

        for (addr, prevote) in &self.state.prevotes {
            if let Some(validator) = self.validator_set.get(addr) {
                *power_by_hash.entry(prevote.block_hash).or_insert(0) += validator.voting_power;
            }
        }

        let quorum_hash = power_by_hash
            .iter()
            .find(|(_, &power)| self.validator_set.has_quorum(power))
            .map(|(hash, _)| *hash);

        // Move to pre-commit step
        if self.state.step == ConsensusStep::PreVote {
            self.state.step = ConsensusStep::PreCommit;
        }

        // If quorum is for a block (not nil), lock on it
        if let Some(Some(block_hash)) = quorum_hash {
            if let Some(proposal) = &self.state.proposal {
                if proposal.block.hash() == block_hash {
                    self.state.locked_block = Some(proposal.block.clone());
                    self.state.locked_round = Some(self.state.round);
                }
            }
        }

        // Send our pre-commit
        let mut messages = Vec::new();
        if self.validator_set.contains(&self.our_address) {
            let precommit = self.create_precommit(quorum_hash.flatten());
            messages.push(ConsensusMessage::PreCommit(precommit));
        }

        Ok(messages)
    }

    /// Creates and signs a pre-commit message.
    /// Also stores the precommit in our own state.
    pub fn create_precommit(&mut self, block_hash: Option<H256>) -> PreCommit {
        // Compute signing hash
        let mut data = Vec::new();
        data.push(0x02); // message type: precommit
        data.extend_from_slice(&self.state.height.to_be_bytes());
        data.extend_from_slice(&self.state.round.to_be_bytes());
        if let Some(hash) = &block_hash {
            data.push(1);
            data.extend_from_slice(hash.as_bytes());
        } else {
            data.push(0);
        }
        data.extend_from_slice(self.our_address.as_bytes());
        let signing_hash = keccak256(&data);

        let signature = self.private_key.sign(&signing_hash);

        let precommit = PreCommit {
            height: self.state.height,
            round: self.state.round,
            block_hash,
            validator: self.our_address,
            signature,
        };

        // Store our own precommit
        self.state.precommits.insert(self.our_address, precommit.clone());

        precommit
    }

    /// Handles a pre-commit message.
    fn handle_precommit(
        &mut self,
        precommit: PreCommit,
    ) -> Result<Vec<ConsensusMessage>, ConsensusError> {
        // Verify height
        if precommit.height != self.state.height {
            return Err(ConsensusError::WrongHeight {
                expected: self.state.height,
                actual: precommit.height,
            });
        }

        // Verify round
        if precommit.round != self.state.round {
            return Err(ConsensusError::WrongRound {
                expected: self.state.round,
                actual: precommit.round,
            });
        }

        // Verify validator is in the set
        let validator = self
            .validator_set
            .get(&precommit.validator)
            .ok_or(ConsensusError::UnknownValidator(precommit.validator))?;

        // Check for duplicate
        if self.state.precommits.contains_key(&precommit.validator) {
            return Err(ConsensusError::DuplicateVote(precommit.validator));
        }

        // Verify signature
        if !precommit.verify(&validator.public_key) {
            return Err(ConsensusError::InvalidSignature);
        }

        // Store the pre-commit
        self.state.precommits.insert(precommit.validator, precommit);

        // Check if we have quorum
        if self.check_precommit_quorum() {
            self.on_precommit_quorum()?;
        }

        Ok(Vec::new())
    }

    /// Checks if we have 2/3+ pre-commits for any block.
    fn check_precommit_quorum(&self) -> bool {
        let mut power_by_hash: HashMap<Option<H256>, u64> = HashMap::new();

        for (addr, precommit) in &self.state.precommits {
            if let Some(validator) = self.validator_set.get(addr) {
                *power_by_hash.entry(precommit.block_hash).or_insert(0) += validator.voting_power;
            }
        }

        for (_, power) in power_by_hash {
            if self.validator_set.has_quorum(power) {
                return true;
            }
        }

        false
    }

    /// Called when we have 2/3+ pre-commits.
    fn on_precommit_quorum(&mut self) -> Result<(), ConsensusError> {
        // Find which block has quorum
        let mut power_by_hash: HashMap<Option<H256>, u64> = HashMap::new();

        for (addr, precommit) in &self.state.precommits {
            if let Some(validator) = self.validator_set.get(addr) {
                *power_by_hash.entry(precommit.block_hash).or_insert(0) += validator.voting_power;
            }
        }

        let quorum_hash = power_by_hash
            .iter()
            .find(|(_, &power)| self.validator_set.has_quorum(power))
            .map(|(hash, _)| *hash);

        // If quorum is for a block (not nil), commit it
        if let Some(Some(block_hash)) = quorum_hash {
            if let Some(proposal) = &self.state.proposal {
                if proposal.block.hash() == block_hash {
                    self.state.committed_block = Some(proposal.block.clone());
                    self.state.step = ConsensusStep::Commit;
                }
            }
        }

        Ok(())
    }

    /// Handles a timeout event.
    ///
    /// Returns messages to broadcast (may include a proposal for the new round).
    pub fn handle_timeout(&mut self) -> Vec<ConsensusMessage> {
        // Move to the next round
        self.state.next_round();

        // If we're the new proposer, we might need to propose
        // (The caller should call create_proposal if appropriate)
        Vec::new()
    }

    /// Advances to the next height after a block is committed.
    ///
    /// Should be called after the committed block has been applied to state.
    pub fn advance_height(&mut self) {
        self.state.next_height();
    }

    /// Returns true if a block has been committed for this height.
    pub fn is_committed(&self) -> bool {
        self.state.committed_block.is_some()
    }

    /// Returns the total pre-vote voting power for a specific block hash.
    pub fn prevote_power(&self, block_hash: Option<H256>) -> u64 {
        let mut power = 0;
        for (addr, prevote) in &self.state.prevotes {
            if prevote.block_hash == block_hash {
                if let Some(validator) = self.validator_set.get(addr) {
                    power += validator.voting_power;
                }
            }
        }
        power
    }

    /// Returns the total pre-commit voting power for a specific block hash.
    pub fn precommit_power(&self, block_hash: Option<H256>) -> u64 {
        let mut power = 0;
        for (addr, precommit) in &self.state.precommits {
            if precommit.block_hash == block_hash {
                if let Some(validator) = self.validator_set.get(addr) {
                    power += validator.voting_power;
                }
            }
        }
        power
    }

    /// Returns the number of pre-votes received.
    pub fn prevote_count(&self) -> usize {
        self.state.prevotes.len()
    }

    /// Returns the number of pre-commits received.
    pub fn precommit_count(&self) -> usize {
        self.state.precommits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validators(count: usize) -> (Vec<PrivateKey>, ValidatorSet) {
        let mut private_keys = Vec::new();
        let mut validators = Vec::new();

        for _ in 0..count {
            let private_key = PrivateKey::random();
            let public_key = private_key.public_key();
            validators.push(Validator::new(public_key, 1));
            private_keys.push(private_key);
        }

        (private_keys, ValidatorSet::new(validators))
    }

    #[test]
    fn test_validator_set_creation() {
        let (_, validator_set) = create_test_validators(4);
        assert_eq!(validator_set.len(), 4);
        assert_eq!(validator_set.total_voting_power(), 4);
    }

    #[test]
    fn test_quorum_calculation() {
        // n = 4, need 2/3+ = 3
        let (_, validator_set) = create_test_validators(4);
        assert_eq!(validator_set.quorum_power(), 3);
        assert!(!validator_set.has_quorum(2));
        assert!(validator_set.has_quorum(3));
        assert!(validator_set.has_quorum(4));

        // n = 7, need 2/3+ = 5
        let (_, validator_set) = create_test_validators(7);
        assert_eq!(validator_set.quorum_power(), 5);
        assert!(!validator_set.has_quorum(4));
        assert!(validator_set.has_quorum(5));
    }

    #[test]
    fn test_proposer_rotation() {
        let (_, validator_set) = create_test_validators(4);

        // Proposer should rotate based on height + round
        let p0 = validator_set.get_proposer(0, 0);
        let p1 = validator_set.get_proposer(0, 1);
        let p2 = validator_set.get_proposer(1, 0);

        assert_ne!(p0.address, p1.address);
        assert_eq!(p1.address, p2.address); // height 0 round 1 == height 1 round 0
    }

    #[test]
    fn test_consensus_start_height() {
        let (private_keys, validator_set) = create_test_validators(4);
        let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

        consensus.start_height(1);
        assert_eq!(consensus.state().height(), 1);
        assert_eq!(consensus.state().round(), 0);
        assert_eq!(consensus.state().step(), ConsensusStep::Propose);
    }

    #[test]
    fn test_create_proposal() {
        let (private_keys, validator_set) = create_test_validators(4);
        let proposer_idx = 0; // First validator is proposer for height 0, round 0
        let mut consensus = TbftConsensus::new(validator_set.clone(), private_keys[proposer_idx].clone());

        consensus.start_height(0);

        let proposal_msg = consensus.create_proposal(vec![], H256::zero(), 1000);
        assert!(proposal_msg.is_some());

        if let Some(ConsensusMessage::Proposal(proposal)) = proposal_msg {
            assert_eq!(proposal.height, 0);
            assert_eq!(proposal.round, 0);
            assert_eq!(proposal.block.height, 0);

            // Verify the signature
            let public_key = private_keys[proposer_idx].public_key();
            assert!(proposal.verify(&public_key));
        }
    }

    #[test]
    fn test_non_proposer_cannot_propose() {
        let (private_keys, validator_set) = create_test_validators(4);
        // Use validator 1, who is not the proposer for height 0, round 0
        let mut consensus = TbftConsensus::new(validator_set, private_keys[1].clone());

        consensus.start_height(0);

        let proposal_msg = consensus.create_proposal(vec![], H256::zero(), 1000);
        assert!(proposal_msg.is_none());
    }

    #[test]
    fn test_handle_proposal() {
        let (private_keys, validator_set) = create_test_validators(4);

        // Create proposal from validator 0
        let mut proposer = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
        proposer.start_height(0);
        let proposal_msg = proposer.create_proposal(vec![], H256::zero(), 1000).unwrap();

        // Validator 1 receives the proposal
        let mut receiver = TbftConsensus::new(validator_set, private_keys[1].clone());
        receiver.start_height(0);

        let result = receiver.handle_message(proposal_msg);
        assert!(result.is_ok());

        let messages = result.unwrap();
        // Should send a pre-vote
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], ConsensusMessage::PreVote(_)));
    }

    #[test]
    fn test_full_consensus_round() {
        let (private_keys, validator_set) = create_test_validators(4);

        // Create consensus instances for all validators
        let mut nodes: Vec<TbftConsensus> = private_keys
            .iter()
            .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
            .collect();

        // Start height 0
        for node in &mut nodes {
            node.start_height(0);
        }

        // Node 0 creates proposal
        let proposal_msg = nodes[0].create_proposal(vec![], H256::zero(), 1000).unwrap();
        let block_hash = if let ConsensusMessage::Proposal(p) = &proposal_msg {
            Some(p.block.hash())
        } else {
            panic!("Expected proposal");
        };

        // Broadcast proposal to all nodes and collect prevotes
        let mut prevotes = Vec::new();
        for i in 0..4 {
            if i == 0 {
                // Proposer already has proposal, create prevote manually
                let prevote = nodes[0].create_prevote(block_hash);
                prevotes.push(ConsensusMessage::PreVote(prevote));
            } else {
                let result = nodes[i].handle_message(proposal_msg.clone());
                assert!(result.is_ok());
                prevotes.extend(result.unwrap());
            }
        }

        // Broadcast prevotes to all nodes
        for i in 0..4 {
            for prevote in &prevotes {
                // Skip if it's our own prevote
                if let ConsensusMessage::PreVote(pv) = prevote {
                    if pv.validator == *nodes[i].our_address() {
                        continue;
                    }
                }
                let _ = nodes[i].handle_message(prevote.clone());
            }
        }

        // Check that all nodes moved to pre-commit and sent precommits
        for node in &nodes {
            assert!(
                node.state().step() == ConsensusStep::PreCommit
                    || node.prevote_count() >= 3
            );
        }

        // Collect all precommits
        let mut precommits = Vec::new();
        for node in &mut nodes {
            let precommit = node.create_precommit(block_hash);
            precommits.push(ConsensusMessage::PreCommit(precommit));
        }

        // Broadcast precommits
        for i in 0..4 {
            for precommit in &precommits {
                if let ConsensusMessage::PreCommit(pc) = precommit {
                    if pc.validator == *nodes[i].our_address() {
                        continue;
                    }
                }
                let _ = nodes[i].handle_message(precommit.clone());
            }
        }

        // All nodes should have committed the block
        for node in &nodes {
            assert!(node.is_committed());
            assert!(node.state().committed_block().is_some());
        }
    }

    #[test]
    fn test_reject_wrong_height() {
        let (private_keys, validator_set) = create_test_validators(4);

        let mut proposer = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
        proposer.start_height(0);
        let proposal_msg = proposer.create_proposal(vec![], H256::zero(), 1000).unwrap();

        let mut receiver = TbftConsensus::new(validator_set, private_keys[1].clone());
        receiver.start_height(1); // Different height

        let result = receiver.handle_message(proposal_msg);
        assert!(matches!(result, Err(ConsensusError::WrongHeight { .. })));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let (private_keys, validator_set) = create_test_validators(4);

        let mut proposer = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
        proposer.start_height(0);
        let proposal_msg = proposer.create_proposal(vec![], H256::zero(), 1000).unwrap();

        // Corrupt the signature by signing with a different key
        let bad_proposal = if let ConsensusMessage::Proposal(mut p) = proposal_msg {
            // Create a valid-format but wrong signature using a different key
            let wrong_key = PrivateKey::random();
            p.signature = wrong_key.sign(&p.signing_hash());
            ConsensusMessage::Proposal(p)
        } else {
            panic!("Expected proposal");
        };

        let mut receiver = TbftConsensus::new(validator_set, private_keys[1].clone());
        receiver.start_height(0);

        let result = receiver.handle_message(bad_proposal);
        assert!(matches!(result, Err(ConsensusError::InvalidSignature)));
    }

    #[test]
    fn test_duplicate_vote_rejected() {
        let (private_keys, validator_set) = create_test_validators(4);

        let mut node = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
        node.start_height(0);

        // Create a prevote from validator 1
        let mut other_consensus = TbftConsensus::new(validator_set, private_keys[1].clone());
        let prevote = other_consensus.create_prevote(Some(H256::zero()));

        // First vote should succeed
        let result = node.handle_message(ConsensusMessage::PreVote(prevote.clone()));
        assert!(result.is_ok());

        // Duplicate should fail
        let result = node.handle_message(ConsensusMessage::PreVote(prevote));
        assert!(matches!(result, Err(ConsensusError::DuplicateVote(_))));
    }

    #[test]
    fn test_timeout_advances_round() {
        let (private_keys, validator_set) = create_test_validators(4);
        let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

        consensus.start_height(0);
        assert_eq!(consensus.state().round(), 0);

        consensus.handle_timeout();
        assert_eq!(consensus.state().round(), 1);
        assert_eq!(consensus.state().step(), ConsensusStep::Propose);
    }

    #[test]
    fn test_locked_block_persists_across_rounds() {
        let (private_keys, validator_set) = create_test_validators(4);

        // Create a consensus with a locked block
        let mut consensus = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
        consensus.start_height(0);

        // Simulate getting locked on a block
        let block = Block::new(0, H256::zero(), vec![], 1000);
        consensus.state.locked_block = Some(block.clone());
        consensus.state.locked_round = Some(0);

        // Timeout to next round
        consensus.handle_timeout();

        // Locked block should persist
        assert!(consensus.state().locked_block().is_some());
        assert_eq!(consensus.state().locked_block().unwrap().hash(), block.hash());
    }

    #[test]
    fn test_nil_prevote_when_locked_on_different_block() {
        let (private_keys, validator_set) = create_test_validators(4);

        let mut consensus = TbftConsensus::new(validator_set.clone(), private_keys[1].clone());
        consensus.start_height(0);

        // Lock on a block
        let locked_block = Block::new(0, H256::zero(), vec![], 999);
        consensus.state.locked_block = Some(locked_block.clone());
        consensus.state.locked_round = Some(0);

        // Create a different proposal
        let different_block = Block::new(0, H256::zero(), vec![], 1000);
        let proposal = Proposal {
            height: 0,
            round: 0,
            block: different_block,
            proposer: private_keys[0].public_key().to_address(),
            signature: private_keys[0].sign(&H256::zero()), // Will be replaced
        };

        // Should vote nil because locked on different block
        let vote_hash = consensus.decide_prevote(&proposal);
        assert!(vote_hash.is_none());
    }

    #[test]
    fn test_byzantine_fault_tolerance_threshold() {
        // With n=4 validators, we need n > 3f + 1
        // So f < (n-1)/3 = 1, meaning we can tolerate f=0 faulty validators
        // But with n=4, quorum is 3, so 1 faulty validator means 3 honest can still reach quorum

        let (private_keys, validator_set) = create_test_validators(4);
        assert_eq!(validator_set.quorum_power(), 3);

        // With 3 honest validators (1 faulty/silent), we can still reach consensus
        let mut nodes: Vec<TbftConsensus> = private_keys[0..3]
            .iter()
            .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
            .collect();

        for node in &mut nodes {
            node.start_height(0);
        }

        // Node 0 creates proposal
        let proposal_msg = nodes[0].create_proposal(vec![], H256::zero(), 1000).unwrap();
        let block_hash = if let ConsensusMessage::Proposal(p) = &proposal_msg {
            Some(p.block.hash())
        } else {
            panic!("Expected proposal");
        };

        // Only 3 nodes participate
        let mut prevotes = Vec::new();
        for i in 0..3 {
            if i == 0 {
                let prevote = nodes[0].create_prevote(block_hash);
                prevotes.push(ConsensusMessage::PreVote(prevote));
            } else {
                let result = nodes[i].handle_message(proposal_msg.clone());
                prevotes.extend(result.unwrap());
            }
        }

        // Broadcast prevotes among the 3 honest nodes
        for i in 0..3 {
            for prevote in &prevotes {
                if let ConsensusMessage::PreVote(pv) = prevote {
                    if pv.validator == *nodes[i].our_address() {
                        continue;
                    }
                }
                let _ = nodes[i].handle_message(prevote.clone());
            }
        }

        // Should have quorum with 3 votes
        for node in &nodes {
            assert_eq!(node.prevote_power(block_hash), 3);
            assert!(validator_set.has_quorum(3));
        }
    }
}
