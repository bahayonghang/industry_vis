<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Vue 3 + TypeScript frontend. Components live in `src/components/`, views in `src/views/`, shared state in `src/stores/`, and shared models/types in `src/types/`. Routes are defined under `src/router`.
- `src-tauri/src/`: Rust backend for Tauri. Key modules include `commands.rs` (IPC entrypoints), `config.rs` (app config), `data_processing.rs` (Polars-based transforms), and `datasource/` for SQL Server access.
- `docs/`: VitePress documentation; serves the contributor and user docs site.
- `dist/`: Frontend build output; Tauri release artifacts land in `src-tauri/target/**`.
- `justfile`: Shortcut commands mirroring npm/cargo workflows; see below for the common ones.

## Build, Test, and Development Commands
- Frontend dev only: `npm run dev` (Vite dev server at localhost:5173).
- Full app dev: `npm run tauri:dev` or `just dev` (starts Vite + Tauri shell).
- Build frontend bundle: `npm run build`.
- Build desktop app: `npm run tauri:build` or `just release` (produces NSIS installer under `src-tauri/target/release/bundle/nsis/`).
- Lint TypeScript/Vue: `npm run lint`.
- Unit tests: `npm run test` (Vitest).
- E2E tests: `npm run test:e2e` (Playwright; ensure browsers installed via `npx playwright install` if missing).
- Rust checks: `cd src-tauri && cargo test` (or `just test-rust`), `cargo check`, `cargo fmt`.

## Coding Style & Naming Conventions
- TypeScript + Vue SFCs with `<script setup>`; prefer composition API and strongly typed props/emits.
- Components/files: PascalCase (`LineChart.vue`), composables `useXxx.ts`, stores `useXxxStore.ts`, utility modules kebab-case where appropriate.
- Indentation: 2 spaces; keep imports sorted and unused code removed. Run `npm run lint -- --fix` for TS/Vue and `cargo fmt` for Rust.
- Favor small, pure functions; avoid duplicating data transforms between frontend and Rust—centralize logic in `data_processing.rs` or shared utilities.

## Testing Guidelines
- Add unit specs near the code (`*.spec.ts`) covering data shaping and rendering logic; mock Tauri calls when possible.
- For Rust, pair new commands or processing steps with `#[cfg(test)]` module tests.
- E2E: cover critical flows (connect DB, search tags, chart renders, export CSV). Keep tests deterministic; reset app state between cases.
- Aim for meaningful assertions over snapshots; add regression tests for bug fixes.

## Commit & Pull Request Guidelines
- Follow Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `test:`); keep subject to 72 chars and in English where possible.
- One logical change per commit; include tests/doc updates alongside code changes.
- PRs should describe purpose, key changes, test evidence (`npm run test`, `cargo test`, `npm run test:e2e` when relevant), and screenshots for UI-impacting work.
- Link related issues; note any config or migration steps (e.g., `config.toml` changes, new env vars).

## Security & Configuration Tips
- Do not commit real connection strings or credentials. Use `config.example.toml` as a template and keep per-machine configs local.
- Ensure SQL queries remain parameterized in `datasource/`; avoid string interpolation with user input.
- Before publishing builds, verify artifacts come from clean `npm run build` + `cargo build --release` outputs and that Playwright tests pass on the release binary.
