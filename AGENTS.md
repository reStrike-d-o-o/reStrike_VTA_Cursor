# AGENTS.md — single source of truth for repo-scoped AI agents

## Project at-a-glance
- Desktop app + tooling. Rust backend (`/src`), React/TS frontend (`/ui`), scripts in `/scripts`, docs in `/docs`.
- Networking: UDP + WebSocket (future REST/OpenAPI).
- Storage: SQLite.

## Build / test / docs
- Node: `pnpm i` then `pnpm build`
- Rust: `cargo build --release && cargo test`
- Docs (one command): `pnpm docs:all`
- Validation: `pnpm lint && cargo clippy -- -D warnings`

## Policy
- Rust: 2021 edition; prefer async (Tokio). No blocking I/O on async paths.
- JS/TS: TypeScript `strict`; prefer functional, memoized React.
- DB: Prepared statements; wrap writes in transactions; WAL mode; add covering/partial indexes where needed.
- DB schema canonical reference: `docs/database/target_model.md` (current state: `docs/database/current_state.md`). Keep data access aligned with SeaORM entities once migrations land.
- Security: No secrets in code, no network egress without approval.
- Output format for docs: Write a single top-level `docs/PROJECT_GUIDE.md` and keep it beginner-friendly.
- always use log::info!, log::warning!, log::error! implemented env_logger stylings for any kinds of logs, except  file logs
- **Ground-truth first**: When describing APIs or data models, rely on generated artifacts in `docs/api/` and `docs/diagrams/`—do not invent endpoints or fields.

## Files agents MAY change
- `docs/**`, `ui/**`, `src/**` (comments only unless task says otherwise), `scripts/**`
## Files agents MUST NOT change
- `LICENSE`, release scripts under `scripts/release/**`

## When in doubt
- Ask in PR description; open small, reviewable changes.
