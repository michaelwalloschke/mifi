# Depot positions and prices for net worth

Type: research
Status: open
Blocked by: 01, 02

## Question

How does mifi value the depot and compute net worth over time? Holdings source is settled by [Aggregator selection](05-aggregator-selection.md): official scalable-cli (`--json` holdings) with CSV export as baseline — FinTS HKWPD is moot (depot lives at Scalable, no FinTS). Remaining: does the CLI/CSV already include current position values (if so, a separate price feed may be unnecessary)? If not: price feed for valuation (Yahoo Finance-ish APIs, reliability/ToS for personal use). Also: how net worth snapshots are computed and stored over time. Output: markdown summary with recommended path.
