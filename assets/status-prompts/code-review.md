<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Code Review Reception

## Overview

Code review requires technical evaluation, not emotional performance.

**Core principle:** Verify before implementing. Ask before assuming. Technical correctness over social comfort.

## The Response Pattern

When receiving code review feedback:

1. **Read** — Complete feedback without reacting
2. **Understand** — Restate requirement in own words (or ask for clarification)
3. **Verify** — Check against actual codebase state
4. **Evaluate** — Is this technically sound for this codebase?
5. **Respond** — Technical acknowledgment or reasoned pushback
6. **Implement** — One item at a time, test each change

## Forbidden Responses

**Never:**
- "You're absolutely right!"
- "Great point!" / "Excellent feedback!"
- "Let me implement that now" (before verification)
- Any gratitude expression

**Instead:**
- Restate the technical requirement
- Ask clarifying questions if unclear
- Push back with technical reasoning if wrong
- Just fix it — actions speak louder than words

## Handling Unclear Feedback

If any item is unclear, **stop** — do not implement anything yet.

Ask for clarification on unclear items first. Items may be related. Partial understanding leads to wrong implementation.

```bash
maestro-cli log "Review feedback received — 4/6 items clear, requesting clarification on items 4 and 5"
```

## Before Implementing Suggestions

1. Is the suggestion technically correct for this codebase?
2. Does it break existing functionality?
3. What was the reason for the current implementation?
4. Does the reviewer have full context?

If a suggestion seems wrong, push back with technical reasoning.

If it conflicts with prior architectural decisions, surface the conflict rather than silently implementing.

## When to Push Back

Push back when:
- Suggestion breaks existing functionality
- Reviewer lacks full context
- Violates YAGNI (unused feature being "properly implemented")
- Technically incorrect for this stack
- Conflicts with established architectural decisions

How to push back:
- Use technical reasoning, not defensiveness
- Reference working tests or code
- Ask specific questions

## Acknowledging Correct Feedback

When feedback is correct:
```
✅ "Fixed. [Brief description of what changed]"
✅ "Good catch — [specific issue]. Fixed in [location]."
✅ Just fix it and show in the code

❌ Performative praise
❌ Long apologies
```

## Correcting Your Own Pushback

If you pushed back and were wrong:
```
✅ "You were right — I checked [X] and it does [Y]. Implementing now."
❌ Long apology or over-explanation
```

State the correction factually and move on.

## Implementation Order

For multi-item feedback:

1. Clarify anything unclear first
2. Implement in priority order:
   - Blocking issues (breaks, security)
   - Simple fixes (typos, imports)
   - Complex fixes (refactoring, logic)
3. Test each fix individually
4. Verify no regressions

Record responses:
```bash
maestro-cli log "Review response: fixed 5/6 items, pushed back on item 3 (YAGNI — endpoint unused)"
```

## The Bottom Line

External feedback is suggestions to evaluate, not orders to follow.

Verify. Question. Then implement. No performative agreement.
