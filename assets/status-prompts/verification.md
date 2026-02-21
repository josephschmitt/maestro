<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Verification Before Completion

## Overview

Claiming work is complete without verification is dishonesty, not efficiency.

**Core principle:** Evidence before claims, always.

## The Iron Law

```
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

If you haven't run the verification command in this session, you cannot claim it passes.

## The Gate Function

Before claiming any completion status:

1. **Identify** — What command proves this claim?
2. **Run** — Execute the full command fresh
3. **Read** — Full output, check exit code, count failures
4. **Verify** — Does the output confirm the claim?
   - If NO: State actual status with evidence
   - If YES: State claim with evidence
5. **Only then** — Make the claim

Skip any step = unverified claim.

**This gate is mandatory before calling:**
```bash
maestro-cli set-status in-review
maestro-cli set-status completed
```

## Verification Requirements

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| Tests pass | Test command output: 0 failures | Previous run, "should pass" |
| Linter clean | Linter output: 0 errors | Partial check, extrapolation |
| Build succeeds | Build command: exit 0 | Linter passing, "looks good" |
| Bug fixed | Test original symptom: passes | Code changed, assumed fixed |
| Requirements met | Line-by-line checklist verified | Tests passing alone |

## Red Flags — Stop

- Using "should", "probably", "seems to"
- Expressing satisfaction before verification ("Great!", "Done!")
- About to call `set-status` without running verification
- Relying on partial verification
- Thinking "just this once"

## Rationalization Prevention

| Excuse | Reality |
|--------|---------|
| "Should work now" | Run the verification |
| "I'm confident" | Confidence ≠ evidence |
| "Just this once" | No exceptions |
| "Linter passed" | Linter ≠ compiler ≠ tests |
| "Partial check is enough" | Partial proves nothing |

## Patterns

**Tests:**
```
✅ Run test command → see "34/34 pass" → claim "all tests pass"
❌ "Should pass now" / "Looks correct"
```

**Build:**
```
✅ Run build → see exit 0 → claim "build passes"
❌ "Linter passed so build should be fine"
```

**Requirements:**
```
✅ Re-read requirements → checklist each item → verify → report
❌ "Tests pass, must be complete"
```

## The Bottom Line

Run the command. Read the output. Then claim the result.

This is non-negotiable.
