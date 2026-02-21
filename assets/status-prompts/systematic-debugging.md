<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Systematic Debugging

## Overview

Find the root cause before attempting fixes. Symptom fixes waste time and create new bugs.

**Core principle:** No fixes without root cause investigation first.

## The Four Phases

### Phase 1: Root Cause Investigation

Before any fix attempt:

1. **Read error messages carefully** — stack traces and error codes frequently contain the answer
2. **Reproduce consistently** — understand the exact conditions triggering the problem
3. **Review recent changes** — what modifications could have introduced this?
4. **Gather diagnostic evidence** — instrument system boundaries to determine where failure occurs
5. **Trace data flow backward** — follow bad values to their origin, not downstream symptoms

### Phase 2: Pattern Analysis

Establish context:

- Locate similar working code in the codebase
- Study reference implementations thoroughly
- Document differences between working and broken code
- Understand environmental dependencies and assumptions

### Phase 3: Hypothesis and Testing

Apply scientific method:

- State a specific hypothesis with supporting reasoning
- Test with minimal, isolated changes
- Verify results before proceeding
- Acknowledge knowledge gaps honestly

When blocked on an ambiguous bug, surface it:
```bash
maestro-cli question "Intermittent failure in auth flow — reproduced 3/10 times. Suspect race condition in token refresh but cannot confirm. Need guidance."
```

### Phase 4: Implementation

Execute the fix:

- Write a failing test that reproduces the bug
- Implement a single targeted change
- Verify the fix resolves the issue without breaking other functionality

Record findings:
```bash
maestro-cli log "Root cause: stale cache entry after config reload. Fix: invalidate cache on config change event."
```

## The Three-Strike Rule

If three fix attempts fail, **stop**. Do not continue patching.

This pattern indicates incomplete investigation or a fundamental architectural problem. Step back and reconsider the approach before proceeding.

## Red Flags — Stop Current Approach

Cease immediately if:

- Considering a "quick temporary fix"
- Making multiple simultaneous changes
- Each fix reveals a new problem in a different place
- Already attempted three or more fixes

These patterns indicate you don't yet understand the problem.

## When Stuck

| Problem | Action |
|---------|--------|
| Can't reproduce | Instrument boundaries, check environment differences |
| Multiple possible causes | Isolate with minimal test cases |
| Fix works but you don't know why | Keep investigating — accidental fixes recur |
| Unfamiliar subsystem | Read reference implementations before touching code |
| Blocked on ambiguity | `maestro-cli question "..."` to surface the blocker |

## Anti-Patterns

| Pattern | Problem |
|---------|---------|
| Fixing symptoms | Root cause still exists, will resurface |
| Shotgun debugging | Multiple changes obscure what actually fixed it |
| Assuming the fix worked | Verify with tests, not intuition |
| Skipping reproduction | You can't fix what you can't reliably trigger |
