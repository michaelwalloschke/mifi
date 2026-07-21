# Verify CSV formats against real exports

Type: task
Status: open

## Question

Confirm the UNVERIFIED items in [CSV import pipeline summary](../assets/17-csv-import-pipeline.md) against Michael's real files. Checklist (HITL — Michael downloads, session inspects):

1. **PayPal**: download Aktivitäten-Export (Aktivitäten → Download → CSV, "Alle Transaktionen", ~1 month is enough, include at least one foreign-currency purchase if available). Verify: report-dialog options offered, actual header column set (fields are user-selectable), FX row-triple sign conventions, `Zeitzone` values.
2. **PayPal fallback**: request one Monatlicher Kontoauszug CSV; capture its full header (`Beschreibung`/`Entgelt` variant).
3. **Scalable**: check whether Michael's tier shows the CSV export button (export is PRIME/PRIME+-gated — if not available, decide fallback: upgrade, or scalable-cli-only for depot). If available, export and verify: encoding/BOM, `reference` uniqueness across rows, whether `amount` is net or gross of `fee`/`tax`, rounding of >2-decimal amounts, row shapes for Interest / Fee / Corporate Action / Security transfer.

Answer records each confirmed/corrected fact and updates the asset in place; sample files stay out of the repo (privacy).
