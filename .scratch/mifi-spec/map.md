# Map: mifi — build-ready spec

Label: wayfinder:map

## Destination

A build-ready spec for **mifi**: a local-first, single-user Tauri desktop app that fetches German bank data (FinTS first, aggregator fallback), auto-categorizes transactions, visualizes money flows beautifully, tracks net worth incl. depot, supports budgeting, and detects recurring contracts. Map is done when every decision needed to start building v1 is made — data model, stack, categorization approach, viz direction, fetch/sync strategy — and assembled into a SPEC.md.

## Notes

- Privacy is the prime constraint: data stays on the machine; aggregator only where FinTS can't reach.
- Settled while charting: Tauri desktop app; single user (Michael); FinTS-first with aggregator fallback; scope = flows + net worth + budgeting + recurring detection; existing data lives in Finanzguru.
- Skills per ticket type: grilling → /grilling + /domain-modeling; prototype → /prototype (+ dataviz skill for charts); research → /research.
- Wayfinder default holds: this map plans, it does not build. Building v1 is the follow-on effort.

## Decisions so far

<!-- one line per closed ticket: gist + link -->

- [Account inventory](issues/01-account-inventory.md) — closed set: Consorsbank (Giro + Tagesgeld, FinTS), Scalable (depot + Verrechnungskonto), PayPal as first-class account; aggregator scope = Scalable + PayPal only.
- [FinTS library landscape](issues/02-fints-library-landscape.md) — python-fints ranked first (active, best TAN UX hooks, HKWPD; Consorsbank works with two vendorable PRs); lib-fints (TS) and hbci4j viable fallbacks; no usable Rust client → backend needs a sidecar. Product-ID registration spawned as [Register FinTS product ID](issues/14-fints-product-id-registration.md).
- [Aggregator selection](issues/05-aggregator-selection.md) — **no aggregator**: GoCardless closed to signups, finAPI fails privacy/personal-scale, PSD2 can't deliver depots anyway. Scalable via official scalable-cli (beta — spawned [Apply for scalable-cli beta](issues/15-scalable-cli-beta.md)) + CSV baseline; PayPal via CSV export. €0, zero third-party processors.

## Not yet specified

- UI information architecture and design language for the whole app — one flow-viz prototype comes first; the rest of the UI can't be sketched until scope of screens falls out of the domain model.
- Refresh/sync UX: how often to fetch, background vs manual, how TAN challenges surface. Sources now fixed (FinTS + scalable-cli + manual CSV imports for PayPal/Scalable baseline); shape depends on backend stack and domain model.
- CSV import pipeline: PayPal/Scalable export formats, dedup on re-import, overlap with fetched data. Sharpens once domain model fixes transaction identity.
- Historical backfill beyond what Finanzguru export + bank APIs return (old CSV archaeology?). Depends on what the Finanzguru salvage actually yields.
- Concrete category taxonomy. Depends on salvaged Finanzguru categories and the chosen categorization approach.

## Out of scope

- Crypto holdings — not in the account mix; returns only as a fresh effort if the destination is redrawn.
- Household/multi-user support — single user decided while charting.
- Mobile app / cloud sync — local desktop only.
- Building the app itself — the destination is the spec; implementation is the next effort.
