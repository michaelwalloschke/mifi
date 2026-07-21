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

## Not yet specified

- UI information architecture and design language for the whole app — one flow-viz prototype comes first; the rest of the UI can't be sketched until scope of screens falls out of the domain model.
- Refresh/sync UX: how often to fetch, background vs manual, how TAN challenges surface. Depends on FinTS library capabilities and aggregator choice.
- Historical backfill beyond what Finanzguru export + bank APIs return (old CSV archaeology?). Depends on what the Finanzguru salvage actually yields.
- Concrete category taxonomy. Depends on salvaged Finanzguru categories and the chosen categorization approach.

## Out of scope

- Crypto holdings — not in the account mix; returns only as a fresh effort if the destination is redrawn.
- Household/multi-user support — single user decided while charting.
- Mobile app / cloud sync — local desktop only.
- Building the app itself — the destination is the spec; implementation is the next effort.
