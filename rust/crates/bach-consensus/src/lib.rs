//! # bach-consensus
//!
//! TBFT (Tendermint-like BFT) consensus for BachLedger.
//!
//! This crate provides:
//! - TBFT consensus protocol
//! - Block proposal
//! - Voting rounds
//! - Finality guarantees
//!
//! ## Architecture
//!
//! ```text
//! +-------------------+
//! |  TbftConsensus    |  <- State machine
//! +-------------------+
//!          |
//! +--------+--------+
//! | Propose| Prevote|  <- Voting steps
//! +--------+--------+
//!          |
//! +-------------------+
//! |    Precommit      |  <- Finalization
//! +-------------------+
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use bach_consensus::{TbftConsensus, TbftConfig, ValidatorSet, Validator};
//!
//! // Create validator set
//! let vs = ValidatorSet::from_validators(vec![
//!     Validator::new(addr1, 100),
//!     Validator::new(addr2, 100),
//!     Validator::new(addr3, 100),
//! ]);
//!
//! // Create consensus engine
//! let config = TbftConfig { address: my_addr, ..Default::default() };
//! let mut consensus = TbftConsensus::new(config, vs);
//!
//! // Start a height
//! consensus.start_height(1);
//!
//! // If we're the proposer, propose a block
//! if consensus.is_proposer() {
//!     consensus.propose_block(block_hash, timestamp)?;
//! }
//!
//! // Process incoming votes
//! consensus.on_vote(vote)?;
//!
//! // Check for finality
//! if consensus.is_finalized(1) {
//!     let commit = consensus.get_commit(1);
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod tbft;
mod types;

pub use error::{ConsensusError, ConsensusResult};
pub use tbft::{ConsensusMessage, Step, TbftConfig, TbftConsensus};
pub use types::{Commit, Proposal, Validator, ValidatorSet, Vote, VoteType};
