# Review: bach-consensus

**Reviewer**: reviewer
**Date**: 2026-02-09
**Module**: bach-consensus
**Files Reviewed**:
- `/Users/moonshot/dev/working/bachledger/rust/bach-consensus/src/lib.rs`
- `/Users/moonshot/dev/working/bachledger/rust/bach-consensus/tests/integration_tests.rs`

## Summary

| Category | Status | Issues |
|----------|--------|--------|
| Code Quality | PASS | 0 |
| Security | PASS | 1 (LOW) |
| Logic | PASS | 0 |
| Tests | PASS | 0 |

**Verdict**: APPROVED

## Test Results

- Unit tests: 15/15 passed
- Integration tests: 26/26 passed
- Clippy: 0 warnings (in bach-consensus)

## Code Quality Analysis

### Positive Findings

1. **No todo!(), unimplemented!(), or panic!("not implemented")** - All code paths implemented.

2. **`#![forbid(unsafe_code)]`** (line 14) - Excellent: No unsafe code allowed.

3. **No #[allow(unused)] or dead_code** - All code actively used.

4. **Clean TBFT Implementation**:
   - `ValidatorSet`: Validator management with weighted voting power
   - `ConsensusState`: Height, round, step, votes, locking
   - `TbftConsensus`: Main consensus engine
   - Message types: `Proposal`, `PreVote`, `PreCommit`

5. **Proper Error Types** (lines 22-40):
   - `UnknownValidator`, `InvalidSignature`, `WrongHeight`, `WrongRound`
   - `NotProposer`, `DuplicateVote`, `InvalidProposal`, `NoProposal`

### Documentation

Excellent module documentation (lines 1-12) explaining:
- Protocol phases (Propose, Pre-vote, Pre-commit, Commit)
- Byzantine fault tolerance (n > 3f + 1)

## Security Analysis (Critical for BFT)

### Byzantine Fault Tolerance

1. **Quorum Calculation** (lines 120-127):
   ```rust
   pub fn quorum_power(&self) -> u64 {
       (2 * self.total_voting_power).div_ceil(3)
   }
   ```
   - Uses ceiling division - **CORRECT** for BFT safety
   - Ensures strictly more than 2/3

2. **Signature Verification** - All messages verified:
   - `Proposal::verify()` (lines 249-252)
   - `PreVote::verify()` (lines 184-187)
   - `PreCommit::verify()` (lines 219-222)

3. **Duplicate Vote Protection** (lines 632-635, 776-779):
   ```rust
   if self.state.prevotes.contains_key(&prevote.validator) {
       return Err(ConsensusError::DuplicateVote(prevote.validator));
   }
   ```

4. **Proposer Verification** (lines 511-515):
   ```rust
   let expected_proposer = self.validator_set.get_proposer(proposal.height, proposal.round);
   if proposal.proposer != expected_proposer.address {
       return Err(ConsensusError::NotProposer);
   }
   ```

5. **Height/Round Verification** (lines 495-509, 610-624):
   - Rejects messages from wrong height or round

6. **Unknown Validator Rejection** (lines 518-521, 627-630):
   ```rust
   let validator = self.validator_set.get(&prevote.validator)
       .ok_or(ConsensusError::UnknownValidator(prevote.validator))?;
   ```

### Locking Mechanism (lines 556-573, 696-704)

Correctly implements TBFT locking:
- Lock on block after 2/3+ prevotes
- Re-propose locked block in subsequent rounds
- Vote nil if locked on different block

### Issue #1: No Equivocation Detection (LOW)

- **Location**: Throughout voting logic
- **Severity**: LOW
- **Description**: If a validator sends two different votes for the same height/round (equivocation), the second is rejected as duplicate. However, there's no explicit equivocation evidence collection for slashing.
- **Impact**: Low for initial deployment - equivocation is prevented, just not punished.
- **Recommendation**: Consider adding equivocation evidence collection in future for slashing.

## Logic Correctness Analysis

### Consensus Flow

1. **Start Height** (lines 425-429): Initializes state correctly.

2. **Proposal Creation** (lines 434-477):
   - Verifies proposer eligibility
   - Re-proposes locked block if locked
   - Signs with proposer's private key

3. **Handle Proposal** (lines 494-553):
   - Validates height, round, proposer, signature, block height
   - Triggers prevote if validator

4. **PreVote Decision** (lines 556-573):
   - Votes nil if locked on different block
   - Respects lock round for unlocking

5. **PreVote Quorum** (lines 654-714):
   - Counts voting power by block hash
   - Locks on block with quorum
   - Moves to PreCommit step

6. **PreCommit Quorum** (lines 797-843):
   - Commits block when 2/3+ precommits for same block

7. **Timeout/View Change** (lines 848-855):
   - Advances round
   - Preserves locked block across rounds (line 362)

### Proposer Rotation (lines 113-116)

```rust
let index = ((height as usize) + (round as usize)) % self.validators.len();
```
- Simple round-robin - correct for basic TBFT

### State Transitions

```
NewHeight -> Propose -> PreVote -> PreCommit -> Commit
                |          |           |
                +----------+-----------+---> next_round() on timeout
```

## Test Coverage Analysis

### Unit Tests (15 tests)
- Validator set creation and quorum calculation
- Proposer rotation
- Consensus start, proposal creation
- Proposal handling with signature verification
- Full consensus round simulation
- Wrong height/signature rejection
- Duplicate vote rejection
- Timeout/round advancement
- Lock persistence across rounds
- Nil voting when locked on different block
- BFT threshold verification

### Integration Tests (26 tests)
- Empty, single, weighted validator sets
- Proposer rotation across heights and rounds
- Quorum calculation for various sizes (1-100 validators)
- Basic consensus flow
- Multiple heights
- Timeout/view change
- Lock persistence
- Proposer re-proposing locked block
- Security: unknown validator, wrong proposer, duplicate votes
- Wrong height/round messages
- BFT with 1 faulty of 4
- Weighted voting power quorum
- Signature verification for all message types
- Nil votes
- State reset on new height

## Positive Observations

1. **Correct BFT Mathematics** - Quorum uses `div_ceil(3)` for proper > 2/3
2. **Complete TBFT Implementation** - All phases implemented correctly
3. **Comprehensive Security Checks** - Signatures, proposer, height/round, duplicates
4. **Locking Mechanism** - Properly implements safety across rounds
5. **Weighted Voting Power** - Supports stake-weighted consensus
6. **Excellent Test Coverage** - 41 tests covering normal flow, edge cases, and Byzantine scenarios
7. **Clean Code Structure** - Well-organized with clear separation of concerns

## TBFT Protocol Compliance

| Feature | Status |
|---------|--------|
| Round-robin proposer | PASS |
| Signed proposals | PASS |
| Pre-vote phase | PASS |
| Pre-commit phase | PASS |
| 2/3+ quorum for votes | PASS |
| Block locking | PASS |
| Lock across rounds | PASS |
| View change (timeout) | PASS |
| Duplicate vote rejection | PASS |
| Signature verification | PASS |
| Unknown validator rejection | PASS |

## Conclusion

The bach-consensus module implements a correct TBFT consensus protocol with:
- No critical or high severity issues
- Proper Byzantine fault tolerance (tolerates f < n/3 faulty validators)
- Comprehensive signature verification
- Correct locking mechanism for safety
- Excellent test coverage including BFT scenarios

**Approved for integration.**
