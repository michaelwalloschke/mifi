# History backfill cut

Type: grilling
Status: closed
Assignee: michael

## Question

Where does history start per account? Salvage gives Giro from 2022-05, Tagesgeld from 2022-06 (rows end 2025-09 — gap to now closes via FinTS), PayPal from 2023-02; Scalable has zero history until CSV export / scalable-cli reach back. Decide: is the salvaged depth enough for v1 (flows, budgets, net worth), or does older bank-CSV archaeology (pre-2022 Consorsbank, pre-2023 PayPal) join the seed import? Also: does Scalable's short history need any mitigation (e.g. manual opening balances) for net-worth continuity?

## Resolution

Grilled 2026-07-22. Four decisions:

1. **No archaeology.** Salvage start dates are the per-account epoch: Giro 2022-05-30, Tagesgeld 2022-06-28, PayPal 2023-02-20. Flows/budgets/recurring operate on recent windows; pre-epoch rows would arrive unlabeled and dilute NB seed quality. Pre-epoch import is post-v1 if ever.

2. **Gap bridge = one-off Consorsbank CSV, not FinTS.** Correction to the ticket premise: FinTS can't close the Tagesgeld gap (2025-09 → now) or the Giro tail in time — product-ID registration wait (10–15 business days) means FinTS isn't available at seed time. Bridge via one-off Consorsbank CSV export through the existing CSV import pipeline; import-hash dedup heals overlap when FinTS takes over post-registration. **FinTS remains the ongoing sync path** — this is bridge-only, sidecar/TAN architecture untouched. Consequence: Consorsbank CSV format is now a planned import → added to [Verify CSV formats against real exports](19-csv-format-verification.md).

3. **Scalable: no net-worth mitigation.** No manual opening balances (unverifiable, poison the curve), no transaction+price reconstruction. Depot net worth starts at first scalable-cli sync. Scalable CSV transactions still import as far back as the export reaches — for flows and transfer-healing (Giro sparplan legs pair up), not valuation. Pre-sync net-worth chart shows cash accounts only.

4. **Cash net-worth curve seeded from salvage.** Derive month-end balances per account from the salvage `Kontostand` column → seed append-only balance Snapshots → 4-year cash curve on day one. Fits Snapshot model, no new entity; bank-beats-seed wins on overlapping dates. Build-time check: whether PayPal rows carry usable Kontostand; if not, Giro + Tagesgeld only.
