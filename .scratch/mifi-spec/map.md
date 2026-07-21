# Map: mifi — build-ready spec

Label: wayfinder:map

## Destination

A build-ready spec for **mifi**: a local-first, single-user Tauri desktop app that fetches German bank data (FinTS first, aggregator fallback), auto-categorizes transactions, visualizes money flows beautifully, tracks net worth incl. depot, supports budgeting, and detects recurring contracts. Map is done when every decision needed to start building v1 is made — data model, stack, categorization approach, viz direction, fetch/sync strategy — and assembled into a SPEC.md.

## Notes

- Privacy is the prime constraint: data stays on the machine; aggregator only where FinTS can't reach.
- Settled while charting: Tauri desktop app; single user (Michael); FinTS-first with aggregator fallback; scope = flows + net worth + budgeting + recurring detection; existing data lives in Finanzguru.
- Skills per ticket type: grilling → /grilling + /domain-modeling; prototype → /prototype (+ dataviz skill for charts); research → /research.
- Wayfinder default holds: this map plans, it does not build. Building v1 is the follow-on effort.
- Long-lead tasks first: [Register FinTS product ID](issues/14-fints-product-id-registration.md) and [Apply for scalable-cli beta](issues/15-scalable-cli-beta.md) have external wait time (10–15 business days / allowlist) — kick them off before or alongside any grilling session.

## Decisions so far

<!-- one line per closed ticket: gist + link -->

- [Account inventory](issues/01-account-inventory.md) — closed set: Consorsbank (Giro + Tagesgeld, FinTS), Scalable (depot + Verrechnungskonto), PayPal as first-class account; aggregator scope = Scalable + PayPal only.
- [FinTS library landscape](issues/02-fints-library-landscape.md) — python-fints ranked first (active, best TAN UX hooks, HKWPD; Consorsbank works with two vendorable PRs); lib-fints (TS) and hbci4j viable fallbacks; no usable Rust client → backend needs a sidecar. Product-ID registration spawned as [Register FinTS product ID](issues/14-fints-product-id-registration.md).
- [Aggregator selection](issues/05-aggregator-selection.md) — **no aggregator**: GoCardless closed to signups, finAPI fails privacy/personal-scale, PSD2 can't deliver depots anyway. Scalable via official scalable-cli (beta — spawned [Apply for scalable-cli beta](issues/15-scalable-cli-beta.md)) + CSV baseline; PayPal via CSV export. €0, zero third-party processors.
- [Depot positions and prices for net worth](issues/08-depot-networth-data.md) — scalable-cli holdings JSON includes current market values (verified in source); price feed fallback-only (CLI quote/chart primary, Alpha Vantage free tier if needed; Yahoo rejected on ToS); net worth = append-only price + balance + position snapshot tables per sync, recompute as primary valuation path. Scalable CSV export is transactions-only — CSV baseline can't value the depot alone.
- [Backend stack decision](issues/06-backend-stack.md) — Foldkit frontend (TEA on Effect; pin pre-1.0, Svelte 5 fallback); domain logic + SQLite (rusqlite + rusqlite_migration) in Rust core; thin uv-managed Python sidecar for FinTS only (JSON-lines/stdio, long-running for TAN); scalable-cli as subprocess.
- [Flow visualization prototype](issues/04-flow-viz-prototype.md) — direction locked: hybrid main screen = stat tiles (Einnahmen/Ausgaben/Sparquote/Puffer + sparklines) → Sankey hero (Einnahmen→Kategorien→Sparziele, month picker) → clickable stacked monthly trend driving tiles + Sankey; hand-rolled SVG, no d3; C-style category drilldown reserved for a later category screen.
- [Domain model](issues/09-domain-model.md) — ubiquitous language pinned in /CONTEXT.md: EUR-only integer cents; transaction identity = import hash + occurrence index, booked-only, bank-beats-seed; Transfer = first-class two-leg link (±4-day auto-match, auto-heals, excluded from flows); flat Splits with auto|user provenance; depth-2 kind-typed categories; Contract covers income+expense, never internal moves; Budget = minimal per-category target (mechanic deferred); net worth derived from append-only Snapshots, NetWorthSnapshot entity dropped.
- [Budgeting model](issues/10-budgeting-model.md) — per-category monthly targets on expense Categories (parent or sub, independent); calendar month, no rollover; spent = net Splits; states 80 %/100 % thresholds, no pace-adjustment; targets effective-dated append-only; one aggregate unbudgeted line.
- [CSV import pipeline](issues/17-csv-import-pipeline.md) — formats pinned: PayPal Aktivitäten-Export (utf-8-sig, 41 German columns, balance-impact row filter, FX row-triples → book EUR leg) and Scalable export (semicolon, ISO dates, German decimals, PRIME-gated); import UX = file picker + drag-drop, no watched folder; row-level error reporting, idempotent re-import heals partial files. Real-file checks spawned as [Verify CSV formats against real exports](issues/19-csv-format-verification.md).

## Not yet specified

- Historical backfill beyond what Finanzguru export + bank APIs return (old CSV archaeology?). Depends on what the Finanzguru salvage actually yields.
- Concrete category taxonomy. Depends on salvaged Finanzguru categories and the chosen categorization approach.

## Out of scope

- Crypto holdings — not in the account mix; returns only as a fresh effort if the destination is redrawn.
- Household/multi-user support — single user decided while charting.
- Mobile app / cloud sync — local desktop only.
- Building the app itself — the destination is the spec; implementation is the next effort.
