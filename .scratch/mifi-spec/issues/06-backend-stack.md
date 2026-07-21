# Backend stack decision

Type: grilling
Status: closed
Assignee: michael
Blocked by: 02

## Question

Inside the Tauri shell, where does the fetch/domain logic live: pure Rust, or a sidecar (Python for python-fints, Node)? Decide backend language, frontend framework, and database (SQLite assumed — confirm, plus migration story). The FinTS library landscape largely forces this hand; grill Michael on the remaining taste choices.

## Resolution (2026-07-21, grilled with Michael)

- **Frontend: Foldkit** (TypeScript, Elm architecture on Effect; Snabbdom views, Vite). Chosen over Svelte 5 after research (https://foldkit.dev/, https://foldkit.dev/ai/overview): active (651★, create-foldkit-app 0.22.0 July 2026), single state tree + pure update functions fit a finance app, and the agent tooling is unmatched — official Claude Code skills plugin (`/plugin install foldkit-skills@foldkit`), DevTools MCP (live Model inspection, Message history, time-travel; dev-only, stripped from production builds), git-subtree vendoring of examples. **Caveat: pre-1.0, breaking changes in minor releases → pin versions, upgrade deliberately.** Svelte 5 remains the named fallback if Foldkit hurts in practice.
- **Domain logic: Rust core** behind Tauri commands — categorization, recurring detection, net worth, budgets, money math. Keeps the sidecar swappable per the narrow-boundary plan in [FinTS library landscape](02-fints-library-landscape.md).
- **Database: SQLite** (confirmed) via **rusqlite + rusqlite_migration**, numbered `.sql` migrations run at startup; DB owned exclusively by the Rust core. sqlx/Diesel rejected — no concurrency to justify async/ORM machinery.
- **FinTS sidecar: Python** (python-fints, vendored Consorsbank PRs #218/#210). Thin — fetch only. IPC contract: accounts, balances, transactions, TAN challenge/response events as JSON-lines over stdio; long-running child process spawned and supervised by the Rust core (TAN needs a live session). Node/lib-fints and Java/hbci4j rejected (younger library / JVM weight).
- **Sidecar runtime: uv-managed** — Rust core launches `uv run` against a checked-in pyproject pinning Python + deps. PyInstaller bundling rejected as packaging pain with no payoff for a single-machine personal app.
- **scalable-cli**: plain subprocess from the Rust core, `--json` output parsed in Rust.
