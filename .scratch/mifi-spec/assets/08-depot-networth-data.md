# Depot Valuation & Net-Worth Data: scalable-cli, Price Feeds, Snapshot Storage

Research date: 2026-07-21. Method: source inspection of the scalable-cli repo (cloned at v0.5.0, commit as of this date), official docs/pricing/ToS pages, and prior-art source code (Ghostfolio, Portfolio Performance, Beancount). Claims marked *(unverified)* could not be confirmed against a primary source. File/line references are into the scalable-cli checkout at repo root.

Context: builds on asset 05 (decision: no aggregator; `scalable-cli` beta as primary path for the Scalable depot, official CSV export as baseline). Open questions: does the CLI deliver market values (or do we need a price feed), which feed if so, and how to store net-worth snapshots in SQLite.

---

## 1. Does scalable-cli deliver current market values? — Yes, fully

Inspected: https://github.com/ScalableCapital/scalable-cli (Rust, Apache-2.0, v0.5.0 per `Cargo.toml:3`). The CLI is a thin client over Scalable's GraphQL API; every broker read command supports `--json` ("Work interactively in the terminal or use `--json` for local scripts and agent workflows", README.md:35; "Broker commands support `--json` for compact structured output", README.md:316).

### `sc broker holdings --json` — per-position quantity, cost basis, current value AND live quote

The `BrokerHoldings` GraphQL query (`src/broker_queries.rs:396-434`) requests per inventory item: `isin`, `name`, `type`, `inventory.position { filled pending blocked fifoPrice }`, `portfolioIsinPerformance { valuation currency }`, and `quoteTick { midPrice currency timestampUtc isOutdated }`.

The JSON projection (`project_broker_holdings_response`, `src/broker_projections.rs:97-155`) emits per position:

| field | meaning | source path |
|---|---|---|
| `isin`, `name`, `security_type` | identity | item |
| `quantity` / `pending_quantity` / `blocked_quantity` | position size | `inventory.position.filled/pending/blocked` |
| `fifo_price` | FIFO cost basis | `position.fifoPrice` |
| **`valuation`**, `valuation_currency` | **current market value of the position** | `portfolioIsinPerformance.valuation` |
| **`quote_mid_price`**, `quote_currency` | current instrument price | `quoteTick.midPrice` |
| `quote_timestamp_utc`, `quote_is_outdated` | quote freshness | `quoteTick` |

So holdings output is *not* quantity/ISIN-only — it carries the current price, the position's market value, the value currency, and a staleness flag.

### Supporting commands (all `--json`, `src/cli.rs:80-117`)

- **`sc broker overview`** → portfolio-level `valuation.total` / `.securities` / `.crypto`, valuation + inventory timestamps, time-weighted return per timeframe (`project_broker_overview_response`, `src/broker_projections.rs:9-44`).
- **`sc broker cash-breakdown`** → `cash_balance`, `cash_available_to_invest`, credit/derivatives availability (`project_broker_cash_breakdown_response`, `src/broker_projections.rs:827-870`) — the Verrechnungskonto balance.
- **`sc broker quote --isin <ISIN>`** → `quote_mid_price`, `quote_bid_price`, `quote_ask_price`, currency, timestamp for any tradable security (`BROKER_QUOTE_QUERY` `src/broker_queries.rs:871`; projection `src/broker_projections.rs:397-476`).
- **`sc broker chart --isin <ISIN> --timeframe 1m`** → **historical price time series** (`timeSeriesBySecurity` with `dataPoints { midPrice timestampUtc }`, `BROKER_CHART_QUERY` `src/broker_queries.rs:437-463`; projection `src/broker_projections.rs:477-560`; README example `README.md:231`). This is a first-party ISIN-keyed EOD history source — usable for backfilling price history.

Caveats (unchanged from asset 05): beta, allowlist required before login (README.md:84-96), and a `sc login --local-read-only` mode that blocks write commands locally (README.md:115-117) — exactly what mifi wants.

### Scalable's own CSV export — transactions only, no positions/values

- The official help article is titled "Kann ich Informationen zu meinen **Transaktionen** exportieren, z.B. als CSV-Datei?" (https://help.scalable.capital/kontoverwaltung-f3197dc7/kann-ich-informationen-zu-meinen-transaktionen-exportier-4c3e0a38; the body is client-rendered — a "Decision Tree" gem, content not statically fetchable — but title, breadcrumb and last-update 2026-07-06 are in the page payload).
- The product-news announcement confirms scope: export **all transactions** (filterable) as a CSV file — nothing about positions or valuations (https://de.scalable.capital/produkt-news/transaktionen-exportieren).
- Community CSV samples (Portfolio Performance forum, first-hand files) show transaction rows — date/status/reference/description/assetType/type/isin/shares/price/amount/fee/tax/currency — where `price` is the **execution** price of that transaction, not a current quote (https://forum.portfolio-performance.info/t/csv-import-von-scalable-capital/30113 — *community, sample files*).
- No official per-position/valuation export is documented anywhere in the help center *(absence checked via help-center browsing, not exhaustive)*.

### Conclusion for Q1

- **CLI path (allowlisted): no separate price feed is needed at all.** Holdings deliver value + quote per position, overview delivers portfolio totals, chart backfills history — all first-party, all EUR.
- **CSV-baseline path (not yet allowlisted, or CLI broken): a price feed is required.** The CSV gives transactions only; positions can be derived (qty from buys/sells) but valuing them needs external quotes.
- Net: **price feed = fallback-only concern**, not a core dependency.

## 2. Price feed for the fallback path (EUR, German/Irish UCITS ETFs)

Privacy frame first: any third-party feed receives the full ISIN list — a readable map of the user's portfolio. The CLI leaks nothing new (Scalable already knows the holdings), which is an additional argument for treating it as the primary quote source too. Daily EOD is sufficient; realtime is a non-goal.

### Yahoo Finance unofficial API / yfinance — rejected as a dependency

- **ToS**: Yahoo's Terms of Service §2(d)(9) prohibit "access or collect data … from our Services using any automated means … robots, spiders, scrapers … for any purpose without our express, prior permission" (https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html). There is no official public finance API; `query1.finance.yahoo.com` is an internal endpoint.
- **yfinance's own README**: "not affiliated, endorsed, or vetted by Yahoo, Inc.", "intended for research and educational purposes", users must consult Yahoo's terms (https://github.com/ranaroussi/yfinance).
- **Breakage history**: repeated Yahoo-side crackdowns — new rate limiting Nov 2024 (https://github.com/ranaroussi/yfinance/issues/2128), waves of `YFRateLimitError: Too Many Requests` through spring 2025 across releases 0.2.57–0.2.59 (https://github.com/ranaroussi/yfinance/issues/2422, https://github.com/ranaroussi/yfinance/issues/2480). It keeps being patched around, but each outage is total until the community ships a workaround.
- Coverage would actually be good (UCITS ETFs listed as `VWCE.DE` etc.), but a ToS-violating, regularly-breaking endpoint is the wrong foundation even for a fallback. Not recommended.

### Alpha Vantage — viable free fallback

- **Coverage**: official docs show Xetra listings via exchange-suffixed symbols — "Sample ticker traded in Germany - XETRA: `symbol=MBG.DEX`" — alongside `.LON`, `.PAR` etc.; prices in local currency (EUR for `.DEX`) (https://www.alphavantage.co/documentation/). German/Irish UCITS ETFs trade on Xetra, so the depot's ISINs are reachable.
- **Lookup**: no ISIN parameter — symbols only; `SYMBOL_SEARCH` exists for discovery (same docs page). For a depot-sized list this is a one-time manual ISIN→`XXX.DEX` mapping stored locally.
- **Free tier**: API key required (free, instant); "up to 25 requests per day" (https://www.alphavantage.co/support/). 25/day covers up to 25 ISINs at one EOD fetch each — ample for one depot.
- **ToS — personal use explicitly licensed**: license grant is "for personal, non-commercial use"; the commercial-use definition carves out "investment analysis, research, testing, monitoring, and any other activities that are individual in nature" as personal usage (https://www.alphavantage.co/terms_of_service/, §2a). mifi's single-user local use fits squarely.

### Others — checked and rejected

| Feed | Free tier | EUR/UCITS coverage on free tier | ISIN lookup | Verdict |
|---|---|---|---|---|
| **Twelve Data** | 800 credits/day, 8/min | **No — Basic plan is US equities/forex/crypto only; EU data from Pro (~$79/mo)** (https://twelvedata.com/pricing) | `isin` param exists but is a **paid Data add-on** (https://twelvedata.com/docs) | No |
| **Finnhub** | 60 calls/min | **No — international exchanges are premium**; free-tier calls for LSE/EU symbols return 401/403 "You don't have access to this resource" (https://github.com/finnhubio/Finnhub-API/issues/405 — vendor's own tracker; https://finnhub.io/pricing) | — | No |
| **EODHD** | 20 calls/day, restricted data types | Full "All World" coverage (Xetra incl.) only from **€19.99/mo** plan (https://eodhd.com/pricing) | search API *(paid)* | No at €0 |
| **justETF scraping** | n/a | good ETF data, but **no public/documented API** — only third-party scrapers (e.g. https://github.com/druzsan/justetf-scraping) | ISIN-keyed pages | No — same legal/fragility class as Yahoo scraping, worse tooling |
| **boerse-frankfurt / Deutsche Börse** | n/a | site (now redirecting to live.deutsche-boerse.com, operated by Deutsche Börse AG with ARIVA.DE AG) exposes **no official API**; known internal JSON endpoints are undocumented and have already survived one site migration that broke scrapers (https://live.deutsche-boerse.com/en/imprint) | — | No |

### Conclusion for Q2

**Primary: `sc broker quote` / `sc broker chart` (first-party, ISIN-native, EUR, zero extra data leak). Fallback: Alpha Vantage free tier** (daily EOD via `.DEX` symbols, 25 req/day, personal use licensed, plain API key). Yahoo/yfinance explicitly rejected as a dependency; if everything else dies it remains a manual-emergency option, not an architectural component.

## 3. Net-worth snapshot storage — prior art and recommendation

### Prior art

- **Beancount**: prices live as append-only `price` directives in the ledger (`2015-11-20 price ITOT 95.46 USD`), fetched/maintained by `bean-price`; "the latest prices of commodities are *never* fetched automatically" — reports are recomputed deterministically from transactions + stored price points at query time. Pure recompute model over an append-only price log (https://beancount.github.io/docs/fetching_prices_in_beancount/).
- **Portfolio Performance**: stores a historical price series per security inside the data file; all valuations are computed at report time via snapshot classes — `ClientSnapshot` builds the portfolio valuation for a given date from stored prices and transactions (https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/snapshot/ClientSnapshot.java). Recompute model over persisted price history.
- **Ghostfolio**: persists price history in a `MarketData` table — `@@unique([dataSource, date, symbol])`, one `marketPrice` per symbol/day (https://github.com/ghostfolio/ghostfolio/blob/main/prisma/schema.prisma, model `MarketData`); portfolio value/performance is recomputed from `Order` + `MarketData` (`PortfolioSnapshot` is an in-memory computed model, `libs/common/src/lib/models/portfolio-snapshot.ts`, produced by `portfolioCalculator.computeSnapshot()` and cached, not persisted). **But cash balances are persisted as a time series**: `AccountBalance` with `@@unique([accountId, date])` and a `value` per day (same schema file) — because a cash balance is an observation, not derivable from prices.

Pattern across all three: **persist append-only price history + recompute security valuations; persist observed account balances as their own dated series.** Nobody persists a redundant "net worth" number as the primary record; nobody relies on recompute for cash either.

### Recommendation for mifi (SQLite)

Store three small append-only tables; treat recompute as the primary valuation path and snapshots as observed ground truth:

1. **`price` (isin, date, price, currency, source)** — `UNIQUE(isin, date)` (add `source` to the key only if two feeds ever coexist). Filled by CLI `chart`/`quote` or Alpha Vantage. This is the Beancount/Ghostfolio price log; it makes historical net-worth charts recomputable and offline-deterministic.
2. **`account_balance_snapshot` (account_id, date, balance, currency)** — `UNIQUE(account_id, date)`, upsert on same-day re-sync. Written on **every successful sync/import** for every cash account (FinTS Konten, Scalable Verrechnungskonto via `cash-breakdown`, PayPal via CSV). Mirrors Ghostfolio `AccountBalance`. Not optional: balances cannot be recomputed from prices, and CSV/FinTS transaction history has gaps and finite lookback.
3. **`position_snapshot` (account_id, isin, date, quantity, price, value, currency, source)** — `UNIQUE(account_id, isin, date)`, upsert. Written on every successful depot sync, straight from `sc broker holdings --json` (`quantity`, `quote_mid_price`, `valuation`). In CSV-only mode, written from derived qty × fetched price, `source` marking it as derived.

Why recompute-only is not enough here (i.e. why 2 and 3 exist even though 1 enables recompute):

- **Cash is an observation** — no price history reconstructs it (the reason Ghostfolio has `AccountBalance`).
- **The baseline path is lossy**: CSV transactions + external prices means derived quantities can silently drift (corporate actions, splits, fusions/ISIN changes, missed rows). A stored per-day observed snapshot from the CLI is ground truth for that day and exposes drift instead of hiding it.
- **Source volatility**: the CLI is beta, allowlist-gated; feeds have quotas and outages. If the price log has a hole, the net-worth chart falls back to the nearest snapshot instead of showing nothing or lying.
- **Cost is nil**: one row per account plus one per position per sync day — tens of rows a day in SQLite.

Net-worth on date D = sum of `account_balance_snapshot` (latest ≤ D per account) + per-position value: recomputed qty×`price` where derivable, `position_snapshot.value` as the observed anchor/override where present. Snapshots are never deleted or rewritten (append-only, same-day upsert only) — that keeps the history auditable and the chart stable retroactively.

## Recommendation

1. **No standalone price-feed dependency in the core.** `sc broker holdings --json` already delivers per-position quantity, cost basis, current price and market value (`src/broker_projections.rs:97-155`); `overview` and `cash-breakdown` cover totals and cash. While the CLI works, mifi is fully valued first-party.
2. **Price history: fill the local `price` table from `sc broker chart`/`quote` (primary), Alpha Vantage free tier (fallback for the CSV-only phase)** — Xetra `.DEX` symbols, 25 req/day, personal use licensed, one-time local ISIN→symbol map. Yahoo/yfinance rejected (ToS §2(d)(9), recurring breakage); Twelve Data/Finnhub/EODHD rejected (EU data paywalled at €0); justETF/boerse-frankfurt rejected (scraping-only).
3. **Storage: append-only `price` + `account_balance_snapshot` + `position_snapshot` tables, written on each successful sync with same-day upsert; net worth recomputed from price history with snapshots as observed anchors.** This copies the proven Beancount/Portfolio-Performance/Ghostfolio split — recompute securities, persist observed cash — and hardens it for mifi's beta-gated, multi-source reality.

## Sources

- https://github.com/ScalableCapital/scalable-cli — inspected at v0.5.0: `src/broker_projections.rs` (holdings :97-155, overview :9-44, quote :397-476, chart :477-560, cash-breakdown :827-870), `src/broker_queries.rs` (BrokerHoldings :396-434, BrokerChart :437-463, BrokerQuote :871), `src/cli.rs:80-117`, `README.md` (:35, :84-96, :115-117, :231, :316)
- https://help.scalable.capital/kontoverwaltung-f3197dc7/kann-ich-informationen-zu-meinen-transaktionen-exportier-4c3e0a38 · https://de.scalable.capital/produkt-news/transaktionen-exportieren · https://forum.portfolio-performance.info/t/csv-import-von-scalable-capital/30113
- https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html · https://github.com/ranaroussi/yfinance · https://github.com/ranaroussi/yfinance/issues/2128 · https://github.com/ranaroussi/yfinance/issues/2422 · https://github.com/ranaroussi/yfinance/issues/2480
- https://www.alphavantage.co/documentation/ · https://www.alphavantage.co/support/ · https://www.alphavantage.co/terms_of_service/
- https://twelvedata.com/pricing · https://twelvedata.com/docs · https://finnhub.io/pricing · https://github.com/finnhubio/Finnhub-API/issues/405 · https://eodhd.com/pricing · https://github.com/druzsan/justetf-scraping · https://live.deutsche-boerse.com/en/imprint
- https://beancount.github.io/docs/fetching_prices_in_beancount/ · https://github.com/portfolio-performance/portfolio/blob/master/name.abuchen.portfolio/src/name/abuchen/portfolio/snapshot/ClientSnapshot.java · https://github.com/ghostfolio/ghostfolio/blob/main/prisma/schema.prisma (models `MarketData`, `AccountBalance`, `Order`) · https://github.com/ghostfolio/ghostfolio/blob/main/libs/common/src/lib/models/portfolio-snapshot.ts
