# Phase 1: Project Scaffolding + Git Init

**Prerequisites:** None (first phase)
**Goal:** SvelteKit + Tauri + Tailwind + shadcn-svelte skeleton. `npm run tauri dev` opens a window with a basic app shell.

## Steps

1. **Git init** — `git init` + create `.gitignore` (node_modules, dist, build, target, .DS_Store, etc.). Commit immediately.
2. **Scaffold SvelteKit** — Use `npx sv create` with TypeScript. Use `adapter-static` with `fallback: 'index.html'` for SPA mode inside Tauri.
3. **Set `ssr = false`** in `src/routes/+layout.ts` since this is a desktop app.
4. **Initialize Tauri v2** — `npx @tauri-apps/cli init` to create `src-tauri/`. Configure window: title "Maestro", 1200x800 default, 800x600 min.
5. **Add Tailwind CSS** — Install and configure Tailwind with PostCSS.
6. **Add shadcn-svelte** — Initialize with `npx shadcn-svelte@latest init`. Set up dark mode support. Add base components (Button, Input, Dialog, etc.).
7. **Create app shell layout** — Root layout (`+layout.svelte`) with a sidebar placeholder (left) and main content area (right).
8. **Add ESLint + Prettier** — Standard config for Svelte + TypeScript.
9. **Verify** — `npm run dev` shows the app in browser, `npm run tauri dev` opens a native window.

## Key Files to Create

```
.gitignore
package.json
svelte.config.js
vite.config.ts
tailwind.config.ts
tsconfig.json
src/app.html
src/app.css                          — Tailwind directives, global styles
src/routes/+layout.svelte            — App shell (sidebar + main)
src/routes/+layout.ts                — ssr = false
src/routes/+page.svelte              — Placeholder "Welcome to Maestro"
src/lib/components/ui/               — shadcn-svelte base components
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
src-tauri/src/main.rs                — Tauri entry point
```

## Key Details

- `adapter-static` is required for Tauri — SvelteKit must output static files, not a Node server
- Tauri `allowlist` should enable: fs, shell, path, process, dialog APIs
- The sidebar is just a styled `<aside>` with a placeholder logo/text — no functionality yet
- All CSS uses Tailwind utility classes; no custom CSS files beyond `app.css`

## Commits

1. `git init` + `.gitignore`
2. SvelteKit scaffold
3. Tauri integration
4. Tailwind + shadcn-svelte setup
5. App shell layout + ESLint/Prettier
