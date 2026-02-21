---
name: maestro-integration
description: Communicates with the Maestro orchestration app via the maestro-cli tool. Use when running inside a Maestro agent session with MAESTRO_SOCKET and MAESTRO_CARD_ID environment variables set.
---

# Maestro CLI

Communicate with the Maestro app using `maestro-cli`. Environment variables `MAESTRO_SOCKET` and `MAESTRO_CARD_ID` are set automatically.

## Commands

### Queries (read-only)

```bash
maestro-cli get-card          # Current task card details (JSON)
maestro-cli get-parent        # Parent card details, if any (JSON)
maestro-cli get-artifacts     # List artifacts for current card (JSON)
```

### Actions (write)

```bash
maestro-cli question "How should auth be handled?"
maestro-cli resolve-question --id <id> --resolution "Use JWT"
maestro-cli add-artifact --file plan.md --name "Architecture Plan"
maestro-cli set-status in-review
maestro-cli set-status completed
maestro-cli log "Finished implementing auth module"
```

## Workflow

1. **Start** — Run `get-card` to understand the task
2. **Surface blockers early** — Use `question` rather than making assumptions
3. **Preserve outputs** — Use `add-artifact` for plans, docs, and analysis
4. **Record progress** — Use `log` for significant milestones
5. **Signal completion** — Use `set-status in-review` or `set-status completed`
