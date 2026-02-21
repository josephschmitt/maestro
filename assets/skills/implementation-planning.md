<!-- Adapted from Superpowers plugin (MIT). See: https://github.com/pcvelz/superpowers -->

# Implementation Planning

## Overview

Turn a design into a set of concrete, delegatable work items. Each work item should be completable by an agent working independently with no knowledge beyond what's provided.

**Core principle:** Every sub-task must be self-contained. If an agent needs to read the parent card or ask a question to understand what to do, the task description is incomplete.

## When to Use

After a design or architecture document exists and the decision to proceed has been made. This skill bridges the gap between "we know what to build" and "agents are building it."

## The Decomposition Process

### 1. Review the Design

- Read the card and its artifacts via `maestro-cli get-card` and `maestro-cli get-artifacts`
- Identify the major components, boundaries, and dependencies
- Note which pieces can be built independently and which have ordering constraints

### 2. Identify Work Items

Break the design into tasks that are:

- **Independent** — completable without waiting on other tasks, or with clearly stated prerequisites
- **Specific** — exact files, functions, interfaces, and behaviors described
- **Testable** — each task has a clear definition of done with verification steps
- **Right-sized** — small enough for a single focused session, large enough to be meaningful

Each task should take roughly one agent session to complete. If a task requires multiple sessions or has too many moving parts, split it further.

### 3. Write Task Descriptions

Every task description must include:

1. **Objective** — one sentence stating what this task produces
2. **Context** — what the agent needs to know about the broader system (architecture, conventions, related components). Do not assume the agent has read the parent card — include or reference the relevant details directly
3. **Scope** — exact files to create or modify, interfaces to implement, behaviors to produce
4. **Dependencies** — what must exist before this task can start (other tasks, existing code, external services)
5. **Acceptance criteria** — specific, verifiable conditions that prove the task is complete
6. **Out of scope** — what this task explicitly does NOT include, to prevent scope creep

### 4. Establish Ordering

Identify which tasks block others and structure the execution order:

- **Foundation first** — shared types, database schema, core interfaces
- **Independent work in parallel** — tasks with no dependencies on each other
- **Integration last** — tasks that wire components together

Surface any ordering questions:
```bash
maestro-cli question "Tasks 3 and 4 both modify the user service. Should they be sequential or can they touch different parts safely?"
```

### 5. Create Sub-Cards

Create sub-cards for each work item. Each sub-card inherits context from the parent but must be self-contained in its description.

Record the plan:
```bash
maestro-cli log "Decomposed into 6 sub-tasks: 2 foundation, 3 parallel implementation, 1 integration"
maestro-cli add-artifact --file work-breakdown.md --name "Work Breakdown"
```

## Writing Good Task Descriptions

### Good

> **Objective:** Implement the `TokenRefreshService` that automatically refreshes expired JWT tokens.
>
> **Context:** The auth system uses short-lived JWTs (15min) with refresh tokens (7d). The `AuthClient` (in `src/lib/services/auth.ts`) makes API calls and needs transparent token refresh on 401 responses.
>
> **Scope:** Create `src/lib/services/token-refresh.ts`. Implement `TokenRefreshService` class with `refreshIfNeeded(response: Response): Promise<Response>` method. Integrate as middleware in `AuthClient.fetch()`.
>
> **Acceptance criteria:**
> - 401 responses trigger exactly one refresh attempt
> - Concurrent requests during refresh are queued, not duplicated
> - Failed refresh clears stored tokens and surfaces auth error
> - Unit tests cover: successful refresh, failed refresh, concurrent request queuing

### Bad

> Implement token refresh for the auth system. Look at the existing auth code to understand how it works. Add tests.

The bad version forces the agent to explore and make assumptions. The good version gives it everything it needs.

## Anti-Patterns

| Pattern | Problem |
|---------|---------|
| "See parent card for details" | Agent may misinterpret context. Include what matters directly. |
| Tasks with vague scope | Agent doesn't know when it's done |
| Skipping dependency analysis | Agents block each other or produce incompatible work |
| One massive task | Too much scope for a single session, too many failure modes |
| Many tiny tasks | Overhead exceeds value. Group related changes. |
| No acceptance criteria | "Done" becomes subjective. Be specific. |

## Verification

Before finalizing the work breakdown:

- [ ] Each task is understandable without reading other tasks
- [ ] Each task has specific acceptance criteria
- [ ] Dependencies between tasks are explicit
- [ ] No circular dependencies exist
- [ ] Foundation tasks are identified and scheduled first
- [ ] The full set of tasks covers the entire design — nothing is missing

```bash
maestro-cli log "Work breakdown verified — 6 tasks, dependency chain validated, ready for execution"
```
