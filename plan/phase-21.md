# Phase 21: Settings + Configuration UI

**Prerequisites:** Phase 3 (config system)
**Goal:** Full settings UI for global config and per-project overrides.

## Steps

1. **Global settings page** — Storage base path, default agent selection, per-status-group defaults.
2. **Agent profile editor** — List of configured agent profiles. Add/edit/remove profiles with fields: name, binary path, flags, custom command, environment variables.
3. **Per-status-group config** — For each group: agent dropdown, model text input, instructions textarea.
4. **Project settings page** — Same structure but as overrides. Each field shows "inherited from global" or "overridden" state.
5. **Config write-back** — Global settings save to `~/.maestro/config.toml`. Project settings save to `agent_config` JSON field on the Project record.
6. **Resolution preview** — "Using: {resolved value}" indicator showing what will actually be used after the inheritance chain resolves.

## Key Files to Create

```
src/routes/settings/global/+page.svelte              — Global settings page
src/lib/components/settings/agent-profiles.svelte     — Agent profile list + management
src/lib/components/settings/agent-profile-form.svelte — Single profile form
src/lib/components/settings/status-group-config.svelte — Per-group settings (agent, model, instructions)
src/lib/components/settings/project-overrides.svelte  — Project-level override toggles
```

## Key Files to Modify

```
src/routes/settings/+page.svelte     — Expand project settings
src-tauri/src/commands/config.rs     — Add write commands for config
```

## Key Details

- **Agent profile form fields:** Name (text), binary path (text + browse button), flags (array editor — add/remove string items), custom command (text, overrides binary+flags if set), env vars (key-value pairs)
- **Status group config:** Each of the 5 groups gets a card/section with: agent dropdown (populated from profiles), model input, instructions textarea
- **Override UX:** Project settings fields have a toggle: "Use global default" / "Override". When overriding, the field becomes editable. When using default, it shows the global value grayed out.
- **Resolution preview:** Below each field, small text: "Resolved: claude-code (from global default)" or "Resolved: codex (project override)"
- **Config TOML write:** Serialize the `GlobalConfig` struct back to TOML and write to `~/.maestro/config.toml`
- **Project JSON write:** Serialize project overrides to JSON and update the `agent_config` field on the Project record
- **Validation:** Binary path should exist on disk (warn if not). Agent name must be unique. Custom command overrides binary+flags entirely.
