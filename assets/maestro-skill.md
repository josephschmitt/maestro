# Maestro Integration

You are running inside Maestro, an AI agent orchestration tool.
You have access to the `maestro-cli` command-line tool for communicating with the Maestro app.

The environment variables `MAESTRO_SOCKET` and `MAESTRO_CARD_ID` are set automatically.

## Available Commands

### During Exploration / Planning
- `maestro-cli question "<question>"` — Surface an open question for the user to answer
- `maestro-cli resolve-question --id <id>` — Resolve an open question (optionally with `--resolution "<text>"`)
- `maestro-cli add-artifact --file <path>` — Attach a document or file as an artifact (optionally with `--name "<name>"`)
- `maestro-cli get-card` — Get details about your current task card (returns JSON)
- `maestro-cli get-parent` — Get details about the parent card, if any (returns JSON)
- `maestro-cli get-artifacts` — List all artifacts for your current card (returns JSON)

### During Implementation
- `maestro-cli set-status in-review` — Signal that your work is ready for review
- `maestro-cli set-status completed` — Signal that your work is complete
- `maestro-cli add-artifact --file <path>` — Attach generated documentation or output files
- `maestro-cli log "<message>"` — Record a progress note visible in the Maestro UI

## Guidelines

- Use `question` to surface blocking questions early rather than making assumptions
- Use `add-artifact` to preserve important outputs (plans, documentation, analysis)
- Use `log` to record significant progress milestones
- Use `set-status` to signal work completion — the user will be notified in real-time
- All commands read `MAESTRO_SOCKET` and `MAESTRO_CARD_ID` from the environment automatically
