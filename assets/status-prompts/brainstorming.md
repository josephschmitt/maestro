<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Brainstorming

## Overview

Flesh out ideas into architecture before committing to implementation. This is a pre-implementation skill — the goal is to explore the problem space and produce a design, not to build anything.

**Core principle:** Understand the problem, propose alternatives, document the architecture. No code, no scaffolding, no implementation.

## The Hard Gate

```
Do NOT write any code, scaffold any project, or take any implementation action.
This skill is purely for exploration and design.
```

This applies even when the solution seems obvious. Straightforward work often masks costly assumptions.

## Process

### 1. Explore Context

- Read the card details via `maestro-cli get-card`
- Check parent card for broader context via `maestro-cli get-parent`
- Review existing artifacts via `maestro-cli get-artifacts`
- Examine relevant files, docs, and recent commits in the working directory

### 2. Ask Clarifying Questions

Surface all open questions early. Ask about:
- Purpose and success criteria
- Constraints (performance, compatibility, dependencies)
- User preferences and non-negotiables

Use `maestro-cli question` to surface questions that need user input:
```bash
maestro-cli question "How should authentication be handled — JWT or session-based?"
maestro-cli question "What are the performance requirements for the data pipeline?"
maestro-cli question "Should this support offline mode from day one?"
```

Ask as many questions as needed — the UI supports multiple open questions. Front-load questions rather than trickling them out one at a time.

### 3. Propose Alternatives

Present **2-3 approaches** with:
- Brief description of each
- Trade-offs (complexity, performance, maintainability)
- Your recommendation and why

### 4. Present Architecture

Walk through the architecture in sections scaled to complexity:
- Component breakdown and responsibilities
- Data flow and key interfaces
- Error handling strategy
- Open risks and unknowns

This is about producing a blueprint, not a finished product. Flag areas that need more investigation rather than making premature decisions.

### 5. Document the Architecture

Save the approved design as an artifact:
```bash
maestro-cli add-artifact --file design.md --name "Design Document"
```

Record the decision:
```bash
maestro-cli log "Design approved — going with approach B (event-driven architecture)"
```

## Anti-Patterns

| Pattern | Problem |
|---------|---------|
| "Too simple to design" | Simple projects harbor unexamined assumptions |
| Jumping to code after surface-level questions | Incomplete understanding leads to rework |
| Single approach presented | No trade-off analysis, no informed choice |
| Design exists only in conversation | Lost context — save it as an artifact |

## When Stuck

If requirements are ambiguous or conflicting, surface the blocker:
```bash
maestro-cli question "Requirements conflict: X says A but Y implies B. Which takes priority?"
```

Do not guess. Do not assume. Ask.
