<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Test-Driven Development

## Overview

Write the test first. Watch it fail. Write minimal code to pass.

**Core principle:** If you didn't watch the test fail, you don't know if it tests the right thing.

## The Iron Law

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

Write code before the test? Delete it. Start over.

- Don't keep it as "reference"
- Don't "adapt" it while writing tests
- Delete means delete

## When to Use

**Always:** New features, bug fixes, refactoring, behavior changes.

**Exceptions (require explicit approval):** Throwaway prototypes, generated code, configuration files.

Thinking "skip TDD just this once"? That's rationalization.

## Red-Green-Refactor

### RED — Write Failing Test

Write one minimal test showing what should happen.

Requirements:
- One behavior per test
- Clear name describing the behavior
- Real code, not mocks (unless unavoidable)

### Verify RED — Watch It Fail

**Mandatory. Never skip.**

Run the test. Confirm:
- Test fails (not errors)
- Failure message matches expectation
- Fails because feature is missing, not because of typos

Test passes immediately? You're testing existing behavior. Fix the test.

### GREEN — Minimal Code

Write the simplest code to pass the test. Nothing more.

Don't add features, refactor other code, or "improve" beyond what the test requires.

### Verify GREEN — Watch It Pass

Run the test. Confirm:
- Test passes
- Other tests still pass
- No errors or warnings in output

### REFACTOR — Clean Up

After green only: remove duplication, improve names, extract helpers. Keep tests green. Don't add behavior.

### Repeat

Next failing test for next behavior.

## Bug Fix Pattern

1. **RED:** Write a test that reproduces the bug
2. **Verify RED:** Confirm it fails with the buggy behavior
3. **GREEN:** Fix the bug with minimal code
4. **Verify GREEN:** Test passes, no regressions

Record progress:
```bash
maestro-cli log "Bug fix: empty email validation — red-green cycle complete"
```

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks. Test takes 30 seconds. |
| "I'll test after" | Tests passing immediately prove nothing. |
| "Already manually tested" | Ad-hoc ≠ systematic. No record, can't re-run. |
| "Deleting X hours is wasteful" | Sunk cost fallacy. Unverified code is debt. |
| "Need to explore first" | Fine. Throw away exploration, start with TDD. |
| "TDD will slow me down" | TDD is faster than debugging. |

## Red Flags — Stop and Start Over

- Code written before test
- Test passes immediately
- Can't explain why test failed
- "Just this once"
- "Keep as reference"
- "Tests after achieve the same purpose"

All of these mean: delete code, start over with TDD.

## Verification Checklist

Before marking work complete:

- [ ] Every new function/method has a test
- [ ] Watched each test fail before implementing
- [ ] Each test failed for the expected reason
- [ ] Wrote minimal code to pass each test
- [ ] All tests pass with clean output

Can't check all boxes? You skipped TDD. Start over.
