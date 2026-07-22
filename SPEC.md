# mifi — Build Spec (v1)

Local-first, single-user Tauri desktop app for Michael's German finances: fetches bank data, auto-categorizes transactions, visualizes money flows, tracks net worth including the depot, budgets per category, and detects recurring contracts.

This spec assembles every decision from the [wayfinder map](.scratch/mifi-spec/map.md). The ubiquitous language is pinned in [CONTEXT.md](CONTEXT.md) — capitalized terms (Account, Transaction, Transfer, Split, Category, Contract, Budget, Sync Run, Snapshot, Position, Amount) mean exactly what CONTEXT.md says. Where this spec and CONTEXT.md could disagree, CONTEXT.md wins.

**Prime constraint: privacy.** All data stays on the machine. No cloud services, no third-party data processors, no cloud LLM — ever, no toggle.

---

## 1. Scope

Exactly five Accounts (closed set — no credit card, cash, loans, pension):

| Account | Type | Sync path | History epoch |
|---|---|---|---|
| Consorsbank Giro | checking | FinTS (ongoing) · one-off Consorsbank CSV bridge | 2022-05-30 (seed) |
| Consorsbank Tagesgeld | savings | FinTS (ongoing) · one-off Consorsbank CSV bridge | 2022-06-28 (seed) |
| Scalable depot | depot | scalable-cli (primary) · Scalable CSV (transactions baseline) | first CLI sync (valuation) |
| Scalable Verrechnungskonto | cash | same as depot | first CLI sync |
| PayPal | e-money | PayPal CSV export | 2023-02-20 (seed) |

No aggregator ([Aggregator selection](.scratch/mifi-spec/issues/05-aggregator-selection.md)): GoCardless closed to signups, finAPI fails privacy/personal-scale, PSD2 can't deliver depots anyway. €0, zero third-party processors.

Features: money flows (Sankey + trends), net worth incl. depot, per-category budgets, recurring-contract detection. Out of scope: crypto, multi-user, mobile/cloud sync.

## 2. Architecture

Decided in [Backend stack](.scratch/mifi-spec/issues/06-backend-stack.md) + [FinTS library landscape](.scratch/mifi-spec/issues/02-fints-library-landscape.md).

```
┌─ Tauri shell ────────────────────────────────────────────┐
│  Frontend: Foldkit (TS, Elm architecture on Effect,      │
│  Snabbdom, Vite) — pin versions (pre-1.0);               │
│  Svelte 5 is the named fallback                          │
│                    │ Tauri commands (no secrets cross)   │
│  Rust core: domain logic — categorization, recurring     │
│  detection, net worth, budgets, money math;              │
│  owns SQLite exclusively (rusqlite +                     │
│  rusqlite_migration, numbered .sql migrations)           │
│     │                          │                         │
│     │ JSON-lines over stdio    │ subprocess, --json      │
│  Python sidecar (uv run,       scalable-cli              │
│  python-fints + vendored       (holdings, transactions,  │
│  PRs #218/#210; fetch only,    quotes; OAuth device      │
│  long-running for TAN)         flow done once in a       │
│                                terminal)                 │
└──────────────────────────────────────────────────────────┘
```

- **Foldkit**: single state tree + pure update functions; official Claude Code skills plugin + DevTools MCP (dev-only). Breaking changes in minors → pin, upgrade deliberately.
- **Rust core** is the only DB writer and the only component touching secrets (§10).
- **Python sidecar**: thin, fetch-only. IPC contract: accounts, balances, transactions, TAN challenge/response events as JSON-lines over stdio. Long-running child process (TAN needs a live session), spawned and supervised by the Rust core. Narrow boundary so a matured Rust FinTS crate can replace it later. Consorsbank: URL `https://brokerage-hbci.consorsbank.de/hbci`, BLZ 76030080, SecurePlus decoupled TAN (python-fints PR #218 confirms; #210 protocol fixes — both vendored).
- **scalable-cli**: plain subprocess; `sc broker holdings --json` delivers per-position quantity, FIFO cost basis, current valuation + `quote_mid_price` (no separate price feed needed on the primary path). Beta, allowlist-gated — see §12.
- **SQLite**: plain (no SQLCipher, §10), migrations run at startup.

## 3. Domain model

[CONTEXT.md](CONTEXT.md) is normative. Build-critical rules ([Domain model](.scratch/mifi-spec/issues/09-domain-model.md)):

- **EUR-only, integer cents.** PayPal FX originals stored as inert display metadata; no FX computation anywhere.
- **Transaction identity**: surrogate PK + per-account Import Hash = hash(booking date, amount, normalized counterparty, normalized purpose) + occurrence index for same-day identicals. Booked only; pending ignored. Native IDs stored as `(source, external_ref)` second idempotency belt. On Source overlap, bank beats `finanzguru-seed`; the seed row survives only as categorization hint.
- **Transfer**: first-class link, exactly two Transactions, opposite sign, equal Amount, different own Accounts. Auto-detect: amount + ±4-day window + own-account counterparty; confident → auto-link, ambiguous → one-click confirm; manual link/unlink always. Legs: excluded from income/expense/flows/budgets, uncategorized, still move balances. Lone leg counts as expense until the counterpart imports — detection re-runs and auto-heals.
- **Splits**: flat, manual-only, sum exactly to the Transaction Amount, never cross-account. Reporting/budgets consume Splits; unsplit Transaction = one implicit Split (single code path). Each Split carries `category_source` auto|user; user-set never auto-overwritten.
- **Categories**: depth-2 hard cap, parent kind-typed income|expense. Taxonomy = Finanzguru's verbatim: 14 mains / 64 pairs (from the salvage, incl. `Sonstiges/Bargeld`, `Sonstiges/Kreditkartenabrechnung` as genuine expense subs). Category CRUD exists in-app; trimming/merging is a post-v1 user activity.
- **Contract**: both directions (Netflix and salary); recurring internal moves (sparplan) are Transfer patterns, never Contracts. Lifecycle detected → confirmed | dismissed, + ended (reversible).
- **Net worth**: derived, never stored — sum of latest balance Snapshots + Position valuations at a date. No NetWorthSnapshot entity.

## 4. Data schema

Derived from the domain model; final DDL lands as numbered `rusqlite_migration` files. Tables (columns indicative, not exhaustive):

- `account` — id, institution, name, type, source_kind; FinTS client-state blob (python-fints `deconstruct()`, no secrets per its docs — stored plain).
- `transaction` — id, account_id, booking_date, amount_cents, counterparty_raw/normalized, purpose_raw/normalized, import_hash, occurrence_index, source (fints | scalable-cli | csv-paypal | csv-scalable | csv-consorsbank | finanzguru-seed), external_ref, fx_metadata (nullable JSON), contract_id (nullable).
- `split` — id, transaction_id, amount_cents, category_id (nullable = uncategorized), category_source (auto | user).
- `transfer` — id, leg_a_txn_id, leg_b_txn_id, link_source (auto | user).
- `category` — id, parent_id (nullable), name, kind (income | expense, parent-level).
- `contract` — id, normalized_counterparty, direction, expected_amount_cents, tolerance, interval (weekly | biweekly | monthly | quarterly | yearly), category_id, status (detected | confirmed | dismissed | ended), creditor_id + mandate_reference (nullable, exact-match signal only), next_expected_date.
- `budget_target` — id, category_id, amount_cents (nullable = ends budget), effective_from_month. Append-only.
- `balance_snapshot` — account_id, date, balance_cents. Append-only, same-day upsert.
- `position_snapshot` — account_id, isin, date, quantity, price, valuation_cents. Append-only, same-day upsert.
- `price` — isin, date, price. Append-only, same-day upsert.
- `merchant_rule` — normalized_merchant, category_id (learned memory, seeded from salvage).
- `nb_token_count` — token, category_id, count (naive-Bayes state).
- `sync_state` — source, last_success_at, last_error. Nothing more — no sync history.

Note `csv-consorsbank` joins the Source enum (bridge import, §11) — CONTEXT.md's Source list should gain it during build.

## 5. Sync pipeline

Decided in [Refresh/sync UX](.scratch/mifi-spec/issues/16-sync-ux.md); Sync Run pinned in CONTEXT.md.

- **Manual-only.** One global Sync fires all syncable sources (FinTS + scalable-cli) concurrently; per-account sync as secondary affordance. No scheduler, no background fetch, no sync-on-launch, no launch nudge.
- **TAN: blocking modal sheet**, two shapes — code-entry field, or decoupled "Confirm in SecurePlus app" with automatic polling. Bank's challenge text verbatim. Cancel fails the FinTS source cleanly; other sources proceed in background. No TAN persistence. No photoTAN image rendering in v1 unless Consorsbank requires it (decoupled SecurePlus is the expected path — verify during build).
- **Atomic per account.** Commit unit = one account within one source: transactions + its Snapshots in one DB transaction; failure writes nothing. Snapshot trigger = successful fetch of that data class (depot holdings success → position + price + Verrechnungskonto balance Snapshots). CSV import never writes balance Snapshots (transactions only). Transfer detection + Contract matching re-run once at end of run over all newly committed accounts.
- **Status: three passive layers.** (a) Per-account relative "last synced" (CSV accounts show last import in the same slot). (b) During run: global spinner + per-account pending/done ticks; app stays usable. (c) Errors: persistent badge on the account + one post-run summary line; click → verbatim sidecar/CLI message + retry. No toasts, no notification center.

## 6. CSV import

Decided in [CSV import pipeline](.scratch/mifi-spec/issues/17-csv-import-pipeline.md); full mapping tables in [the asset](.scratch/mifi-spec/assets/17-csv-import-pipeline.md).

- **PayPal Aktivitäten-Export** ("Alle Transaktionen"): `utf-8-sig`, comma, all fields quoted, 41 German columns. `DD.MM.YYYY`, German decimals, leading minus. Row filter: `Auswirkung auf Guthaben ∈ {Soll, Haben}` and `Status = Abgeschlossen`. `Transaktionscode` = external_ref. FX purchase = row triple → book the EUR leg, keep original currency/amount as inert metadata. Monatlicher Kontoauszug variant supported via header aliasing.
- **Scalable export**: semicolon, ISO dates, German decimals despite English headers; `date;time;status;reference;description;assetType;type;isin;shares;price;amount;fee;tax;currency`. PRIME-gated.
- **Consorsbank Umsätze CSV** (bridge-only, §11): format captured in [Verify CSV formats](.scratch/mifi-spec/issues/19-csv-format-verification.md) — open at spec time, verify before building the parser.
- **UX**: file picker + drag-and-drop onto Konten & Sync. No watched folder. Idempotent by construction (Import Hash + occurrence index + external_ref) — re-import heals partial files.
- **Errors**: unknown header/encoding → reject whole file, message, nothing written. Malformed rows → import valid rows, report skipped rows with reasons. No silent drops.

## 7. Categorization

Decided in [Categorization approach](.scratch/mifi-spec/issues/07-categorization-approach.md). Three local layers, in order; cloud LLM hard no.

1. **Merchant memory** (deterministic): normalize counterparty (strip mid-word spaces like `HelloFre sh`, casefold, strip legal suffixes) → learned `normalized merchant → category` table → assign with provenance `auto`.
2. **Naive Bayes** (fallback): hand-rolled in Rust core, token counts in SQLite, tokens from normalized counterparty + purpose. Assigns only when top1/top2 posterior ratio ≥ ~3 (tunable); below → honest uncategorized queue. No silent low-confidence guesses.
3. **Local LLM** (async post-sync sweep + on-demand from the review queue): one configurable OpenAI-compatible base URL, default `http://localhost:11434/v1` (ollama; LM Studio via port). Prompt carries the fixed taxonomy; only a valid category id accepted, else row stays uncategorized. Endpoint unreachable → pass silently skipped. Never blocks a Sync Run, never joins its atomicity — LLM writes are ordinary post-commit `auto` categorizations.

**Learning loop**: a user correction (a) writes/overwrites the merchant-memory rule, (b) increments NB counts, (c) offers one-click retro-apply to same-merchant rows — touching only `auto`/uncategorized, never user-set. The LLM layer never learns.

**Seeding** (from the Finanzguru salvage, §11): the 912 transfer legs excluded from both layers; memory rules purity-gated (merchant ≥2 rows, ≥80 % one category — tunable at import); all rows seed NB counts.

## 8. Recurring/contract detection

Decided in [Recurring detection](.scratch/mifi-spec/issues/11-recurring-detection.md); research detail in [the asset](.scratch/mifi-spec/assets/11-recurring-detection.md).

- **Grouping**: (normalized merchant per §7, direction); one Contract per (merchant, direction, amount band). `(Gläubiger-ID, Mandatsreferenz)` stored per Contract as exact-match signal, never the grouping key.
- **Two stages, re-run stable**: Stage A feeds existing Contracts first — mandate exact match, else merchant + next-date window + ±15 % of last amount (rolling, absorbs price creep). Stage B generates candidates only from unclaimed rows. Dismissed Contracts silently absorb their rows forever; `detected` proposals update in place.
- **Rules**: amount clusters max(±1 €, ±10 %) expense / ±25 % income; median-gap classification, per-interval ranges (monthly 25–35 d, day-of-month anchored, month-end clamped); propose `detected` at ≥3 occurrences (yearly ≥2 + near-exact amounts); one ≈2× gap tolerated as missed cycle; next-date windows ±2/±3/±5/±7/±14 d per interval.
- **Intervals**: weekly | biweekly | monthly | quarterly | yearly (`biweekly` first-class — HelloFresh ground truth).
- **Lifecycle**: 2 consecutive missed cycles → `ended` (yearly: 1 + 90 d grace), reversible on new match.
- **Seeding**: 49 Finanzguru contracts import as `confirmed`, turnus mapped 1:1, merge policy applied; the seed doubles as a recall/acceptance test for the tolerances.
- **UI requirement**: the Verträge confirm screen needs a "merge into existing contract" action — a >15 % price jump surfaces as a second `detected` by design.

## 9. Budgeting

Decided in [Budgeting model](.scratch/mifi-spec/issues/10-budgeting-model.md); Budget pinned in CONTEXT.md.

Per-category monthly targets on expense Categories, parent or sub independently (no sum constraint — independent alarms, not an allocation tree). Calendar month, no rollover either direction. Spent = |net sum of the month's Splits|; refunds reduce spent; Transfer legs excluded. States: on-track < 80 %, warning ≥ 80 %, over ≥ 100 % (shows € overshoot) — never pace-adjusted. Targets effective-dated append-only; past months judge against the target then in force; null Amount ends the budget. One aggregate "Ohne Budget" line (sum of expense Splits in target-less Categories, rolled up; number only, no state, no prompts). Main screen gets one aggregate signal only (e.g. "2 over budget").

## 10. Net worth & valuation

Decided in [Depot positions & net worth](.scratch/mifi-spec/issues/08-depot-networth-data.md).

- **Primary**: scalable-cli holdings include full market values — no separate price feed. Append-only `price` + `balance_snapshot` + `position_snapshot` written per successful sync (same-day upsert); recompute from Snapshots is the primary valuation path; Snapshots are ground truth.
- **Fallback price feed** (CSV-baseline phase pre-allowlist, or CLI breakage): first the CLI's own `quote`/`chart`; then Alpha Vantage free tier (Xetra via `.DEX`, 25 req/day, ToS allows personal use). Yahoo/yfinance rejected (ToS).
- Depot valuation starts at first CLI sync — no manual opening balances, no reconstruction (§11). Pre-sync net-worth chart shows cash accounts only.

## 11. Seed & history backfill

Decided in [History backfill cut](.scratch/mifi-spec/issues/20-history-backfill-cut.md) + [Finanzguru export salvage](.scratch/mifi-spec/issues/03-finanzguru-export-salvage.md) (findings: [asset](.scratch/mifi-spec/assets/03-finanzguru-export.md)).

1. **Finanzguru XLSX seed** (`~/Downloads/finanzguru.xlsx`, stays out of the repo): 4633 rows, 28 German columns — Giro 2022-05→now, Tagesgeld 2022-06→2025-09, PayPal 2023-02→now, no Scalable. Seeds: transaction history (source `finanzguru-seed`), categorization (§7), 49 Contracts as `confirmed` (§8), 912 transfer legs as link hints.
2. **No archaeology**: salvage start dates are the per-account epochs; pre-epoch import is post-v1 if ever.
3. **Consorsbank CSV bridge** (one-off): closes the Tagesgeld 2025-09→now gap and the Giro tail — FinTS registration (10–15 business days) isn't available at seed time. Runs through the normal CSV pipeline; Import-Hash dedup heals overlap when FinTS takes over. Bridge-only — FinTS stays the sync path.
4. **Cash net-worth curve from salvage**: month-end balances per account from the `Kontostand` column → seed balance Snapshots → ~4-year cash curve on day one. Bank-beats-seed on overlapping dates. Build-time check: whether PayPal rows carry usable Kontostand; if not, Giro + Tagesgeld only.
5. **Scalable**: CSV transactions import as far back as the export reaches — for flows and transfer-healing, not valuation.

## 12. Security

Decided in [Security model](.scratch/mifi-spec/issues/12-security-model.md); detail in [the asset](.scratch/mifi-spec/assets/12-security-model.md).

- **Threat model: lost/stolen machine + backup media.** Compromised OS explicitly out of scope — no zeroize, no app master password, no anti-debugger.
- **Secrets → macOS login keychain**, accessed only from the Rust core via the `keyring` crate. No Tauri keychain plugin, no Stronghold — the webview never sees a secret. FinTS PIN handed to the sidecar per-session over stdio. scalable-cli keeps its default `session_backend: keyring`; OAuth device login once in a terminal.
- **DB: plain SQLite; FileVault is the encryption at rest.** SQLCipher rejected. Startup `fdesetup status` check → persistent warning in Konten & Sync when FileVault is off.
- **Backup**: encrypted Time Machine (documented path) + one "Export backup…" action using `VACUUM INTO`. Keychain items don't travel → restore = re-enter PIN once + re-run CLI login once.
- **Hygiene invariants**: PIN/TAN/OAuth tokens never in DB, config, or logs; TAN dies with the sync modal; sidecar logs at INFO (python-fints DEBUG logs wire traffic); default strict Tauri CSP; no remote content; no secret-returning Tauri command.
- Implementation-time checks (from the asset): keychain prompt behavior across dev/release signatures; python-fints logger names; scalable-cli beta token backend.

## 13. UI

Decided in [UI information architecture](.scratch/mifi-spec/issues/18-ui-information-architecture.md) + [Flow visualization prototype](.scratch/mifi-spec/issues/04-flow-viz-prototype.md). Clickable prototypes: [UI IA (variants A/B/C)](.scratch/mifi-spec/assets/18-ui-ia-prototype.html), [flow viz (variants D/A/B/C)](.scratch/mifi-spec/assets/04-flow-viz-prototype.html) — build from variant **A** shell + variant **D** main screen; screenshots linked on the tickets.

**Navigation**: fixed left sidebar (~216 px), badges on items (errors, newly detected Contracts), sync block at the bottom (timestamp + button, always visible).

**7 screens**:
1. **Übersicht** — stat tiles (Einnahmen, Ausgaben Δ, Sparquote Δ, Puffer; 12-month sparklines) → Sankey hero (Einnahmen → Kategorien → Sparziele; month picker; ribbon tooltips) → clickable stacked monthly trend (expenses by category + income line) driving tiles *and* Sankey. Tiles and Sankey nodes navigate (category click → Kategorien detail).
2. **Transaktionen** — date-grouped list, account filter + search; Splits as indented rows with auto/user tag; Transfer legs as "⇄ Umbuchung — nicht in Auswertungen"; inline recategorize via category chip.
3. **Kategorien** — master–detail drilldown: list left; right = single-category trend + subcategories + the category's Contracts.
4. **Budget** — target rows with progress bars in category color, 80 % mark in the bar, states as icon+text (▲ ≥ 80 %, ⚠ over — never color alone), aggregate "Ohne Budget" row.
5. **Verträge** — stat tiles (fixed costs/month normalized, contract income, count); "Neu erkannt" card with confirm/dismiss (+ merge-into-existing, §8); active list with next payment + monthly normalization.
6. **Vermögen** — tiles (net, depot, cash, Δ month), stacked curve (depot + cash, from Snapshots), positions table, account list.
7. **Konten & Sync** — account cards (balance, source, timestamp/error badge, per-account sync), global sync with per-account ticks, TAN modal, CSV dropzone.

**Design language** (ticket-04 system): warm gray paper (#f9f9f7 light / #0d0d0d dark), 10-px-radius cards, system font, tabular numerals, validated category palette light+dark ("Sonstiges" deliberately gray as other-bucket), status always icon+text. All charts hand-rolled SVG — no d3.

## 14. v1 cut-line

**v1 ships**: everything above — 5 accounts, FinTS sync (Consorsbank) + TAN modal, scalable-cli sync, 3 CSV importers, Finanzguru seed + Consorsbank bridge, 3-layer categorization incl. local-LLM sweep, transfers, splits, contracts + detection, budgets, net worth, all 7 screens, security model.

**Explicitly post-v1** (decided deferrals, not fog):
- Background/scheduled Scalable refresh (if CLI tokens prove long-lived).
- photoTAN image rendering (unless Consorsbank forces it during build).
- Forecast/projected-balance display over Contracts; salary-date month shifting.
- Taxonomy trimming/merging (in-app category CRUD activity).
- Pre-epoch history archaeology; budget rollover (rejected, not deferred); pace-adjusted budget states (rejected).
- Enable Banking (watchlist only, if cash-account automation ever wanted).

## 15. External dependencies open at spec time

Three tasks ride into the build effort — none blocks starting:

1. **[Register FinTS product ID](.scratch/mifi-spec/issues/14-fints-product-id-registration.md)** (10–15 business days) — required before live FinTS; CSV bridge covers the wait.
2. **[Apply for scalable-cli beta](.scratch/mifi-spec/issues/15-scalable-cli-beta.md)** (allowlist) — CSV baseline covers the wait; depot valuation starts at first CLI sync.
3. **[Verify CSV formats against real exports](.scratch/mifi-spec/issues/19-csv-format-verification.md)** — confirm UNVERIFIED items (PayPal header set/FX signs, Scalable tier/encoding, Consorsbank format) before finishing each parser; the pipeline design does not change.

## 16. Build-order sketch (non-binding)

1. Skeleton: Tauri + Foldkit + Rust core + SQLite migrations; schema (§4).
2. Finanzguru seed importer + taxonomy + categorization seeding (§7, §11) — data on screen early.
3. CSV importers (PayPal, Scalable, Consorsbank bridge) behind the shared normalization/Import-Hash path (§6).
4. Domain engines: transfers, categorization layers 1–2, recurring detection (test against the 49-contract seed), budgets, net worth.
5. Screens in dependency order: Transaktionen → Übersicht → Kategorien/Budget/Verträge/Vermögen → Konten & Sync.
6. scalable-cli integration; LLM sweep layer.
7. FinTS sidecar + TAN modal (product ID permitting) — last, it has the longest external tail.
