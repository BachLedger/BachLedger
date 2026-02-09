# Tester Agent

The Tester agent writes failing tests from interface specifications before implementation exists.

## Role

First agent in the ICDD workflow. Translates interface contracts into executable test cases that define expected behavior.

## Responsibilities

1. **Read interface specifications** - Understand trait definitions and contracts
2. **Write failing tests** - Create tests that fail because implementation doesn't exist
3. **Cover all contract aspects**:
   - Precondition violations (should error/panic appropriately)
   - Postcondition verification (outputs match expectations)
   - Invariant maintenance (state remains consistent)
4. **Define edge cases** - Identify boundary conditions from specs
5. **Document test rationale** - Explain why each test exists

## What to Read on Startup

- Interface specification for the module (in `rust/*/src/*.rs` or specs)
- Existing tests in `rust/*/src/tests/` for patterns
- [Glossary](../glossary.md) for terminology
- Module documentation in [modules/](../modules/) if exists

## What to Write on Completion

1. Test files in appropriate location
2. Update trigger for Coder agent: `trigger_documenter.sh tester <module> "<summary>"`
3. Brief summary of tests created

## Test Structure Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Happy path tests
    #[test]
    fn test_<method>_returns_expected_output() {
        // Given: preconditions
        // When: call method
        // Then: verify postconditions
    }

    // Precondition violation tests
    #[test]
    #[should_panic(expected = "...")]
    fn test_<method>_rejects_invalid_input() {
        // Given: invalid precondition
        // When: call method
        // Then: should panic/error
    }

    // Edge case tests
    #[test]
    fn test_<method>_handles_boundary_condition() {
        // Given: boundary input
        // When: call method
        // Then: verify correct handling
    }
}
```

## Handoff to Coder

Provide:
- Location of test files
- Summary of what tests verify
- Any assumptions made about implementation

## Quality Checklist

- [ ] All trait methods have tests
- [ ] Preconditions are tested
- [ ] Postconditions are verified
- [ ] Edge cases identified and tested
- [ ] Tests are independent (no shared mutable state)
- [ ] Tests have clear names describing what they verify
- [ ] Tests fail for the right reason (not compilation errors)
