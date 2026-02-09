# Glossary

Key terms used in the BachLedger ICDD workflow.

## Development Methodology

### ICDD (Interface-Contract Driven Development)
Development approach where interfaces and their contracts (preconditions, postconditions, invariants) are defined before implementation. Tests are derived from these contracts.

### TDD (Test-Driven Development)
Development cycle: write failing test -> implement minimal code to pass -> refactor. Red-Green-Refactor.

### BDD (Behavior-Driven Development)
Extension of TDD focusing on behavior specifications using Given-When-Then format.

### Design by Contract
Software design approach where interfaces specify obligations (preconditions), guarantees (postconditions), and invariants.

## Rust Concepts

### Trait
Rust's mechanism for defining shared behavior. Similar to interfaces in other languages.

### Interface Contract
The specification of a trait including:
- Method signatures
- Preconditions (what must be true before calling)
- Postconditions (what will be true after calling)
- Invariants (what remains true throughout)

### Acceptance Criteria
Specific, testable conditions that must be met for a feature to be considered complete.

### Property-Based Testing
Testing approach where properties that should hold for all inputs are verified with random data.

## Blockchain Concepts

### EVM (Ethereum Virtual Machine)
Stack-based virtual machine that executes smart contract bytecode.

### Transaction
Signed message that changes blockchain state.

### Block
Collection of transactions with a header containing metadata.

### State Trie
Merkle Patricia Trie storing account states.

### Gas
Unit measuring computational work in EVM execution.

### Nonce
Counter preventing transaction replay attacks.

### Signature (ECDSA)
Cryptographic proof of transaction authorization using secp256k1 curve.

### Address
20-byte identifier derived from public key hash.

## Testing Terms

### Unit Test
Tests a single function or method in isolation.

### Integration Test
Tests multiple components working together.

### Property-Based Test
Tests properties that should hold for all inputs.

### Fuzz Test
Tests with random/malformed inputs to find edge cases.

### Attack Vector
Potential method of exploiting a vulnerability.

### Edge Case
Unusual or extreme input that may cause unexpected behavior.

### Test Coverage
Percentage of code exercised by tests.

### Regression Test
Test ensuring previously fixed bugs don't reappear.

## Agent Workflow Terms

### Handoff
Transfer of work from one agent to another with context.

### Context Broadcast
Notification to agents about relevant changes.

### Work Unit
Discrete piece of work completed by an agent.

### Trigger
Signal that activates an agent or workflow step.

### Review Gate
Checkpoint requiring approval before proceeding.

### Iteration Cycle
Loop back through workflow stages to address issues.
