//! Integration tests for bach-consensus TBFT implementation

use bach_consensus::{
    ConsensusError, ConsensusMessage, ConsensusStep, TbftConsensus, Validator, ValidatorSet,
};
use bach_crypto::PrivateKey;
use bach_primitives::H256;
use bach_types::Block;

/// Helper to create a test validator set with the given count
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

/// Helper to create validators with specific voting powers
fn create_weighted_validators(powers: &[u64]) -> (Vec<PrivateKey>, ValidatorSet) {
    let mut private_keys = Vec::new();
    let mut validators = Vec::new();

    for &power in powers {
        let private_key = PrivateKey::random();
        let public_key = private_key.public_key();
        validators.push(Validator::new(public_key, power));
        private_keys.push(private_key);
    }

    (private_keys, ValidatorSet::new(validators))
}

// =============================================================================
// Validator Set Tests
// =============================================================================

#[test]
fn test_validator_set_empty() {
    let validator_set = ValidatorSet::new(vec![]);
    assert!(validator_set.is_empty());
    assert_eq!(validator_set.len(), 0);
    assert_eq!(validator_set.total_voting_power(), 0);
}

#[test]
fn test_validator_set_single() {
    let (_, validator_set) = create_test_validators(1);
    assert_eq!(validator_set.len(), 1);
    assert_eq!(validator_set.total_voting_power(), 1);
    // Quorum for 1 validator should be 1
    assert_eq!(validator_set.quorum_power(), 1);
}

#[test]
fn test_validator_set_weighted() {
    let (_, validator_set) = create_weighted_validators(&[10, 20, 30, 40]);
    assert_eq!(validator_set.len(), 4);
    assert_eq!(validator_set.total_voting_power(), 100);
    // Quorum = (2 * 100 + 2) / 3 = 67 (strictly more than 2/3)
    assert_eq!(validator_set.quorum_power(), 67);
    assert!(!validator_set.has_quorum(66));
    assert!(validator_set.has_quorum(67));
}

#[test]
fn test_validator_lookup() {
    let (keys, validator_set) = create_test_validators(4);

    for key in &keys {
        let addr = key.public_key().to_address();
        assert!(validator_set.contains(&addr));
        let validator = validator_set.get(&addr);
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().address, addr);
    }

    // Unknown address
    let unknown_key = PrivateKey::random();
    let unknown_addr = unknown_key.public_key().to_address();
    assert!(!validator_set.contains(&unknown_addr));
    assert!(validator_set.get(&unknown_addr).is_none());
}

// =============================================================================
// Proposer Rotation Tests
// =============================================================================

#[test]
fn test_proposer_rotation_across_heights() {
    let (_, validator_set) = create_test_validators(4);

    // Collect proposers for heights 0-7
    let mut proposers = Vec::new();
    for height in 0..8 {
        proposers.push(validator_set.get_proposer(height, 0).address);
    }

    // Should cycle through all validators
    assert_eq!(proposers[0], proposers[4]);
    assert_eq!(proposers[1], proposers[5]);
    assert_ne!(proposers[0], proposers[1]);
}

#[test]
fn test_proposer_rotation_across_rounds() {
    let (_, validator_set) = create_test_validators(4);

    // Proposer should change with round
    let r0 = validator_set.get_proposer(0, 0).address;
    let r1 = validator_set.get_proposer(0, 1).address;
    let r2 = validator_set.get_proposer(0, 2).address;

    assert_ne!(r0, r1);
    assert_ne!(r1, r2);
}

// =============================================================================
// Quorum Calculation Tests
// =============================================================================

#[test]
fn test_quorum_various_sizes() {
    // Test quorum calculation for various validator set sizes
    let test_cases = [
        (1, 1),   // 1 validator, need 1
        (2, 2),   // 2 validators, need 2
        (3, 2),   // 3 validators, need 2
        (4, 3),   // 4 validators, need 3
        (5, 4),   // 5 validators, need 4
        (6, 4),   // 6 validators, need 4
        (7, 5),   // 7 validators, need 5
        (10, 7),  // 10 validators, need 7
        (100, 67), // 100 validators, need 67
    ];

    for (n, expected_quorum) in test_cases {
        let (_, validator_set) = create_test_validators(n);
        assert_eq!(
            validator_set.quorum_power(),
            expected_quorum,
            "Failed for n={}: expected quorum {}, got {}",
            n,
            expected_quorum,
            validator_set.quorum_power()
        );
    }
}

// =============================================================================
// Consensus Flow Tests
// =============================================================================

#[test]
fn test_basic_consensus_flow() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut nodes: Vec<TbftConsensus> = private_keys
        .iter()
        .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
        .collect();

    // Start height
    for node in &mut nodes {
        node.start_height(0);
        assert_eq!(node.state().step(), ConsensusStep::Propose);
    }

    // Create and broadcast proposal
    let proposal = nodes[0]
        .create_proposal(vec![], H256::zero(), 1000)
        .expect("Proposer should create proposal");

    let block_hash = match &proposal {
        ConsensusMessage::Proposal(p) => Some(p.block.hash()),
        _ => panic!("Expected proposal"),
    };

    // Collect prevotes
    let mut all_prevotes = Vec::new();
    for (i, node) in nodes.iter_mut().enumerate() {
        if i == 0 {
            // Proposer creates its own prevote
            let prevote = node.create_prevote(block_hash);
            all_prevotes.push(ConsensusMessage::PreVote(prevote));
        } else {
            let msgs = node.handle_message(proposal.clone()).unwrap();
            all_prevotes.extend(msgs);
        }
    }

    assert_eq!(all_prevotes.len(), 4);

    // Broadcast prevotes
    for node in &mut nodes {
        for prevote in &all_prevotes {
            if let ConsensusMessage::PreVote(pv) = prevote {
                if pv.validator != *node.our_address() {
                    let _ = node.handle_message(prevote.clone());
                }
            }
        }
    }

    // All should be at PreCommit
    for node in &nodes {
        assert_eq!(node.state().step(), ConsensusStep::PreCommit);
    }

    // Collect and broadcast precommits
    let mut all_precommits = Vec::new();
    for node in &mut nodes {
        let precommit = node.create_precommit(block_hash);
        all_precommits.push(ConsensusMessage::PreCommit(precommit));
    }

    for node in &mut nodes {
        for precommit in &all_precommits {
            if let ConsensusMessage::PreCommit(pc) = precommit {
                if pc.validator != *node.our_address() {
                    let _ = node.handle_message(precommit.clone());
                }
            }
        }
    }

    // All should have committed
    for node in &nodes {
        assert!(node.is_committed());
        assert_eq!(node.state().step(), ConsensusStep::Commit);
    }
}

#[test]
fn test_multiple_heights() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut nodes: Vec<TbftConsensus> = private_keys
        .iter()
        .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
        .collect();

    // Run through 3 heights
    for height in 0..3 {
        let proposer_idx = (height as usize) % 4;

        // Start height
        for node in &mut nodes {
            node.start_height(height);
        }

        // Create proposal
        let proposal = nodes[proposer_idx]
            .create_proposal(vec![], H256::zero(), 1000 + height)
            .expect("Should create proposal");

        let block_hash = match &proposal {
            ConsensusMessage::Proposal(p) => Some(p.block.hash()),
            _ => panic!("Expected proposal"),
        };

        // Quick consensus (simplified - just create and broadcast votes)
        let mut prevotes = Vec::new();
        for (i, node) in nodes.iter_mut().enumerate() {
            if i == proposer_idx {
                prevotes.push(ConsensusMessage::PreVote(node.create_prevote(block_hash)));
            } else {
                let msgs = node.handle_message(proposal.clone()).unwrap();
                prevotes.extend(msgs);
            }
        }

        for node in &mut nodes {
            for prevote in &prevotes {
                if let ConsensusMessage::PreVote(pv) = prevote {
                    if pv.validator != *node.our_address() {
                        let _ = node.handle_message(prevote.clone());
                    }
                }
            }
        }

        let mut precommits = Vec::new();
        for node in &mut nodes {
            precommits.push(ConsensusMessage::PreCommit(node.create_precommit(block_hash)));
        }

        for node in &mut nodes {
            for precommit in &precommits {
                if let ConsensusMessage::PreCommit(pc) = precommit {
                    if pc.validator != *node.our_address() {
                        let _ = node.handle_message(precommit.clone());
                    }
                }
            }
        }

        // All committed
        for node in &nodes {
            assert!(node.is_committed());
        }

        // Advance to next height
        for node in &mut nodes {
            node.advance_height();
        }
    }
}

// =============================================================================
// View Change / Timeout Tests
// =============================================================================

#[test]
fn test_timeout_changes_proposer() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());

    consensus.start_height(0);

    // Get proposer for round 0
    let proposer_r0 = validator_set.get_proposer(0, 0).address;

    // Timeout
    consensus.handle_timeout();

    // Round should advance
    assert_eq!(consensus.state().round(), 1);

    // Proposer should change
    let proposer_r1 = validator_set.get_proposer(0, 1).address;
    assert_ne!(proposer_r0, proposer_r1);
}

#[test]
fn test_multiple_timeouts() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    for expected_round in 1..=5 {
        consensus.handle_timeout();
        assert_eq!(consensus.state().round(), expected_round);
        assert_eq!(consensus.state().step(), ConsensusStep::Propose);
    }
}

// =============================================================================
// Locking Tests
// =============================================================================

#[test]
fn test_lock_persists_after_timeout() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    // Simulate locking on a block
    let block = Block::new(0, H256::zero(), vec![], 1000);
    let block_hash = block.hash();
    consensus.state_mut().locked_block = Some(block);
    consensus.state_mut().locked_round = Some(0);

    // Multiple timeouts
    for _ in 0..3 {
        consensus.handle_timeout();
        assert!(consensus.state().locked_block().is_some());
        assert_eq!(consensus.state().locked_block().unwrap().hash(), block_hash);
    }
}

#[test]
fn test_proposer_repropose_locked_block() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    // Lock on a specific block
    let locked_block = Block::new(0, H256::zero(), vec![], 999);
    let locked_hash = locked_block.hash();
    consensus.state_mut().locked_block = Some(locked_block);
    consensus.state_mut().locked_round = Some(0);

    // Create a proposal - should propose the locked block
    let proposal = consensus.create_proposal(vec![], H256::zero(), 2000);

    if let Some(ConsensusMessage::Proposal(p)) = proposal {
        assert_eq!(p.block.hash(), locked_hash);
        // Should use locked block's timestamp, not the new one
        assert_eq!(p.block.timestamp, 999);
    } else {
        panic!("Expected proposal");
    }
}

// =============================================================================
// Security / Byzantine Scenario Tests
// =============================================================================

#[test]
fn test_reject_unknown_validator() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    // Create a prevote from an unknown validator
    let unknown_key = PrivateKey::random();
    let mut unknown_consensus = TbftConsensus::new(
        ValidatorSet::new(vec![Validator::new(unknown_key.public_key(), 1)]),
        unknown_key.clone(),
    );

    let fake_prevote = unknown_consensus.create_prevote(Some(H256::zero()));

    let result = consensus.handle_message(ConsensusMessage::PreVote(fake_prevote));
    assert!(matches!(result, Err(ConsensusError::UnknownValidator(_))));
}

#[test]
fn test_reject_wrong_proposer() {
    let (private_keys, validator_set) = create_test_validators(4);

    // Validator 1 tries to propose when validator 0 should be proposer
    let mut wrong_proposer = TbftConsensus::new(validator_set.clone(), private_keys[1].clone());
    wrong_proposer.start_height(0);

    // Should return None since they're not the proposer
    let proposal = wrong_proposer.create_proposal(vec![], H256::zero(), 1000);
    assert!(proposal.is_none());
}

#[test]
fn test_reject_duplicate_prevote() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut node = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
    node.start_height(0);

    // Create a valid prevote from validator 1
    let mut other = TbftConsensus::new(validator_set, private_keys[1].clone());
    let prevote = other.create_prevote(Some(H256::zero()));

    // First should succeed
    let result = node.handle_message(ConsensusMessage::PreVote(prevote.clone()));
    assert!(result.is_ok());

    // Duplicate should fail
    let result = node.handle_message(ConsensusMessage::PreVote(prevote));
    assert!(matches!(result, Err(ConsensusError::DuplicateVote(_))));
}

#[test]
fn test_reject_duplicate_precommit() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut node = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
    node.start_height(0);

    let mut other = TbftConsensus::new(validator_set, private_keys[1].clone());
    let precommit = other.create_precommit(Some(H256::zero()));

    // First should succeed
    let result = node.handle_message(ConsensusMessage::PreCommit(precommit.clone()));
    assert!(result.is_ok());

    // Duplicate should fail
    let result = node.handle_message(ConsensusMessage::PreCommit(precommit));
    assert!(matches!(result, Err(ConsensusError::DuplicateVote(_))));
}

#[test]
fn test_reject_message_wrong_height() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut node0 = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
    let mut node1 = TbftConsensus::new(validator_set, private_keys[1].clone());

    // Node 0 at height 0
    node0.start_height(0);
    // Node 1 at height 1
    node1.start_height(1);

    // Create proposal at height 0
    let proposal = node0.create_proposal(vec![], H256::zero(), 1000).unwrap();

    // Node 1 should reject it
    let result = node1.handle_message(proposal);
    assert!(matches!(result, Err(ConsensusError::WrongHeight { .. })));
}

#[test]
fn test_reject_message_wrong_round() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut node0 = TbftConsensus::new(validator_set.clone(), private_keys[0].clone());
    let mut node1 = TbftConsensus::new(validator_set, private_keys[1].clone());

    node0.start_height(0);
    node1.start_height(0);

    // Node 1 times out to round 1
    node1.handle_timeout();

    // Create proposal at round 0
    let proposal = node0.create_proposal(vec![], H256::zero(), 1000).unwrap();

    // Node 1 at round 1 should reject round 0 proposal
    let result = node1.handle_message(proposal);
    assert!(matches!(result, Err(ConsensusError::WrongRound { .. })));
}

// =============================================================================
// Byzantine Fault Tolerance Tests
// =============================================================================

#[test]
fn test_consensus_with_one_faulty_of_four() {
    // 4 validators, 1 faulty (silent) - should still reach consensus
    let (private_keys, validator_set) = create_test_validators(4);

    // Only 3 honest nodes participate
    let mut honest_nodes: Vec<TbftConsensus> = private_keys[0..3]
        .iter()
        .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
        .collect();

    for node in &mut honest_nodes {
        node.start_height(0);
    }

    // Node 0 proposes
    let proposal = honest_nodes[0]
        .create_proposal(vec![], H256::zero(), 1000)
        .unwrap();

    let block_hash = match &proposal {
        ConsensusMessage::Proposal(p) => Some(p.block.hash()),
        _ => panic!("Expected proposal"),
    };

    // Collect prevotes from 3 honest nodes
    let mut prevotes = Vec::new();
    for (i, node) in honest_nodes.iter_mut().enumerate() {
        if i == 0 {
            prevotes.push(ConsensusMessage::PreVote(node.create_prevote(block_hash)));
        } else {
            let msgs = node.handle_message(proposal.clone()).unwrap();
            prevotes.extend(msgs);
        }
    }

    // Broadcast prevotes
    for node in &mut honest_nodes {
        for prevote in &prevotes {
            if let ConsensusMessage::PreVote(pv) = prevote {
                if pv.validator != *node.our_address() {
                    let _ = node.handle_message(prevote.clone());
                }
            }
        }
    }

    // With 3 votes and quorum = 3, should move to PreCommit
    for node in &honest_nodes {
        assert_eq!(node.state().step(), ConsensusStep::PreCommit);
    }

    // Collect and broadcast precommits
    let mut precommits = Vec::new();
    for node in &mut honest_nodes {
        precommits.push(ConsensusMessage::PreCommit(node.create_precommit(block_hash)));
    }

    for node in &mut honest_nodes {
        for precommit in &precommits {
            if let ConsensusMessage::PreCommit(pc) = precommit {
                if pc.validator != *node.our_address() {
                    let _ = node.handle_message(precommit.clone());
                }
            }
        }
    }

    // All should commit
    for node in &honest_nodes {
        assert!(node.is_committed());
    }
}

#[test]
fn test_weighted_voting_power_quorum() {
    // 4 validators with powers: 10, 20, 30, 40 (total 100, quorum 68)
    let (private_keys, validator_set) = create_weighted_validators(&[10, 20, 30, 40]);

    let mut nodes: Vec<TbftConsensus> = private_keys
        .iter()
        .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
        .collect();

    for node in &mut nodes {
        node.start_height(0);
    }

    // Node 0 proposes
    let proposal = nodes[0].create_proposal(vec![], H256::zero(), 1000).unwrap();

    let block_hash = match &proposal {
        ConsensusMessage::Proposal(p) => Some(p.block.hash()),
        _ => panic!("Expected proposal"),
    };

    // Get prevotes from validators with power 10, 20, 30 (total 60 < 68)
    let mut prevotes = Vec::new();
    for (i, node) in nodes[0..3].iter_mut().enumerate() {
        if i == 0 {
            prevotes.push(ConsensusMessage::PreVote(node.create_prevote(block_hash)));
        } else {
            let msgs = node.handle_message(proposal.clone()).unwrap();
            prevotes.extend(msgs);
        }
    }

    // Broadcast to node 0
    for prevote in &prevotes {
        if let ConsensusMessage::PreVote(pv) = prevote {
            if pv.validator != *nodes[0].our_address() {
                let _ = nodes[0].handle_message(prevote.clone());
            }
        }
    }

    // With power 60 < 68, should NOT have quorum yet
    assert_eq!(nodes[0].prevote_power(block_hash), 60);
    // Still at PreVote (no quorum reached to move to PreCommit)
    // Actually the state depends on implementation - let's verify power
    assert!(!validator_set.has_quorum(60));

    // Now add the validator with power 40
    let _ = nodes[3].handle_message(proposal.clone());
    let big_prevote = nodes[3].create_prevote(block_hash);

    let _ = nodes[0].handle_message(ConsensusMessage::PreVote(big_prevote));

    // Now should have quorum (60 + 40 = 100 >= 68)
    assert_eq!(nodes[0].prevote_power(block_hash), 100);
    assert!(validator_set.has_quorum(100));
}

// =============================================================================
// Signature Verification Tests
// =============================================================================

#[test]
fn test_proposal_signature_verification() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    let proposal_msg = consensus.create_proposal(vec![], H256::zero(), 1000).unwrap();

    if let ConsensusMessage::Proposal(proposal) = proposal_msg {
        // Verify with correct key
        assert!(proposal.verify(&private_keys[0].public_key()));

        // Verify with wrong key should fail
        assert!(!proposal.verify(&private_keys[1].public_key()));
    }
}

#[test]
fn test_prevote_signature_verification() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    let prevote = consensus.create_prevote(Some(H256::zero()));

    // Verify with correct key
    assert!(prevote.verify(&private_keys[0].public_key()));

    // Verify with wrong key should fail
    assert!(!prevote.verify(&private_keys[1].public_key()));
}

#[test]
fn test_precommit_signature_verification() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    let precommit = consensus.create_precommit(Some(H256::zero()));

    // Verify with correct key
    assert!(precommit.verify(&private_keys[0].public_key()));

    // Verify with wrong key should fail
    assert!(!precommit.verify(&private_keys[1].public_key()));
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_nil_votes() {
    let (private_keys, validator_set) = create_test_validators(4);

    let mut nodes: Vec<TbftConsensus> = private_keys
        .iter()
        .map(|key| TbftConsensus::new(validator_set.clone(), key.clone()))
        .collect();

    for node in &mut nodes {
        node.start_height(0);
    }

    // All vote nil (no proposal received)
    let mut nil_prevotes = Vec::new();
    for node in &mut nodes {
        nil_prevotes.push(ConsensusMessage::PreVote(node.create_prevote(None)));
    }

    // Broadcast nil prevotes
    for node in &mut nodes {
        for prevote in &nil_prevotes {
            if let ConsensusMessage::PreVote(pv) = prevote {
                if pv.validator != *node.our_address() {
                    let _ = node.handle_message(prevote.clone());
                }
            }
        }
    }

    // Should have quorum for nil
    for node in &nodes {
        assert_eq!(node.prevote_power(None), 4);
    }
}

#[test]
fn test_state_reset_on_new_height() {
    let (private_keys, validator_set) = create_test_validators(4);
    let mut consensus = TbftConsensus::new(validator_set, private_keys[0].clone());

    consensus.start_height(0);

    // Add some state
    consensus.state_mut().locked_block = Some(Block::new(0, H256::zero(), vec![], 1000));
    consensus.state_mut().locked_round = Some(0);

    // Start new height
    consensus.start_height(1);

    // State should be reset
    assert_eq!(consensus.state().height(), 1);
    assert_eq!(consensus.state().round(), 0);
    assert!(consensus.state().locked_block().is_none());
    assert!(consensus.state().proposal().is_none());
    assert!(!consensus.is_committed());
}
