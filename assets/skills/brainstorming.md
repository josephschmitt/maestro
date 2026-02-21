<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Brainstorming

## Overview

Design before implementation. No exceptions.

**Core principle:** Explore the problem space, propose alternatives, get approval, then build.

## The Hard Gate

```
Do NOT write any code, scaffold any project, or take any implementation action
until you have presented a design and the user has approved it.
```

This applies to "simple" projects too. Straightforward work often masks costly assumptions.

## Process

### 1. Explore Context

- Read the card details via `maestro-cli get-card`
- Check parent card for broader context via `maestro-cli get-parent`
- Review existing artifacts via `maestro-cli get-artifacts`
- Examine relevant files, docs, and recent commits in the working directory

### 2. Ask Clarifying Questions

Ask questions **one at a time** to understand:
- Purpose and success criteria
- Constraints (performance, compatibility, dependencies)
- User preferences and non-negotiables

When a question needs user input, surface it:
```bash
maestro-cli question "How should authentication be handled — JWT or session-based?"
```

Do not stack multiple questions. Wait for each answer before proceeding.

### 3. Propose Alternatives

Present **2-3 approaches** with:
- Brief description of each
- Trade-offs (complexity, performance, maintainability)
- Your recommendation and why

### 4. Present Design

Walk through the design in sections scaled to complexity:
- Architecture and component breakdown
- Data flow and key interfaces
- Error handling strategy
- Testing approach

Seek approval on each section before moving on.

### 5. Document the Design

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
| Jumping to code after one question | Incomplete understanding leads to rework |
| Single approach presented | No trade-off analysis, no informed choice |
| Design exists only in conversation | Lost context — save it as an artifact |

## When Stuck

If requirements are ambiguous or conflicting, surface the blocker:
```bash
maestro-cli question "Requirements conflict: X says A but Y implies B. Which takes priority?"
```

Do not guess. Do not assume. Ask.
