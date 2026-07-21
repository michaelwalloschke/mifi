# Depot positions and prices for net worth

Type: research
Status: closed
Assignee: michael
Blocked by: 01, 02

## Question

How does mifi value the depot and compute net worth over time? Holdings source is settled by [Aggregator selection](05-aggregator-selection.md): official scalable-cli (`--json` holdings) with CSV export as baseline — FinTS HKWPD is moot (depot lives at Scalable, no FinTS). Remaining: does the CLI/CSV already include current position values (if so, a separate price feed may be unnecessary)? If not: price feed for valuation (Yahoo Finance-ish APIs, reliability/ToS for personal use). Also: how net worth snapshots are computed and stored over time. Output: markdown summary with recommended path.

## Resolution

Full findings with citations: [Depot positions & net worth summary](../assets/08-depot-networth-data.md).

**1. scalable-cli delivers full market values — no separate price feed needed on the primary path.** Verified in source (v0.5.0): `sc broker holdings --json` emits per position quantity, FIFO cost basis, current `valuation` + currency, and `quote_mid_price` with timestamp + staleness flag. `overview` gives portfolio totals, `cash-breakdown` the Verrechnungskonto, `chart --isin` historical price series. Scalable's official CSV export is **transactions only** — no positions/valuation export exists, so the CSV baseline cannot value the depot on its own.

**2. Price feed is fallback-only** (CSV-baseline phase pre-allowlist, or CLI breakage). Primary: the CLI's own `quote`/`chart` (first-party, ISIN-native, EUR, no new data leak). Fallback: **Alpha Vantage free tier** — Xetra via `.DEX` symbols, 25 req/day suffices for one depot's EOD, ToS explicitly allows personal investment analysis. Rejected: Yahoo/yfinance (ToS forbids automated collection, repeated breakage), Twelve Data/Finnhub (EU data paywalled), EODHD (EU coverage paid), justETF/boerse-frankfurt (scraping only).

**3. Net worth snapshots: append-only observations + recompute.** Prior art (Beancount, Portfolio Performance, Ghostfolio) converges on persisted price history + recomputed security valuations, with cash balances persisted as observations. mifi: three append-only SQLite tables — `price` (isin+date), `account_balance_snapshot` (account+date), `position_snapshot` (account+isin+date, qty/price/value) — written on every successful sync with same-day upsert. Recompute is the primary valuation path; snapshots are ground truth. Recompute-only fails: cash is an observation, CSV-derived quantities drift (splits/ISIN changes), beta-gated sources leave holes.

Feeds [Domain model](09-domain-model.md) (Position, NetWorthSnapshot) and the spec's data schema.
