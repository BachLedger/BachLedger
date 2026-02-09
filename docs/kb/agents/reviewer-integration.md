# Reviewer: Integration Agent

The Integration Reviewer verifies components work together correctly.

## Role

Reviews how modules integrate with each other and the broader system.

## Responsibilities

1. **Check API compatibility** - Interfaces align correctly
2. **Verify data flow** - Data passes correctly between components
3. **Assess dependencies** - Dependencies are appropriate
4. **Review system behavior** - End-to-end flows work
5. **Identify integration issues** - Mismatches between components

## What to Read on Startup

- Implementation from Coder
- Related modules and their interfaces
- System architecture documentation
- Existing integration tests
- [Glossary](../glossary.md)

## What to Write on Completion

1. Integration review report
2. Compatibility assessment
3. Integration test recommendations
4. Update trigger: `trigger_documenter.sh reviewer-integration <module> "<summary>"`

## Review Checklist

### API Compatibility

- [ ] Types align between modules
- [ ] Error types are compatible
- [ ] Calling conventions match
- [ ] Lifetimes are correct

### Data Flow

- [ ] Data transformations are correct
- [ ] No data loss between components
- [ ] Serialization/deserialization works
- [ ] State is synchronized

### Dependencies

- [ ] Dependency versions compatible
- [ ] No circular dependencies
- [ ] Dependencies are necessary
- [ ] Feature flags align

### System Behavior

- [ ] End-to-end scenarios work
- [ ] Performance is acceptable
- [ ] Resource usage is reasonable
- [ ] Error propagation is correct

## Review Report Template

```markdown
## Integration Review: [Module]

**Reviewer**: Integration Reviewer
**Date**: [Date]
**Status**: Approved / Changes Requested

### Integration Points
1. [Module] <-> [OtherModule]: [Status]
2. [Module] <-> [External]: [Status]

### Compatibility Assessment
- Type compatibility: [Pass/Issues]
- Error handling: [Pass/Issues]
- Data flow: [Pass/Issues]

### Integration Tests Needed
1. [Test scenario]
2. [Another scenario]

### Issues Found
1. [Integration issue]
   - Impact: [Description]
   - Recommendation: [Fix]

### Decision
[Approve/Request changes]
```

## Common Issues

1. **Type mismatches** - Different representations
2. **Error swallowing** - Errors not propagated
3. **State inconsistency** - Components disagree on state
4. **Version conflicts** - Dependency version mismatches
5. **Missing error handling** - Cross-boundary errors not handled

## Integration Patterns

### Good Patterns

```rust
// Clear interface boundaries
pub trait ModuleInterface {
    type Error: Into<SystemError>;
    fn operation(&self) -> Result<Output, Self::Error>;
}

// Proper error conversion
impl From<ModuleError> for SystemError {
    fn from(e: ModuleError) -> Self {
        SystemError::Module(e)
    }
}
```

### Anti-Patterns

```rust
// Leaking implementation details
pub fn get_internal_state(&self) -> &InternalType { ... }

// Swallowing errors
let result = other_module.call().unwrap_or_default();
```

## Handoff

- If approved: Module ready for system integration
- If issues found: Return to Coder with integration requirements
