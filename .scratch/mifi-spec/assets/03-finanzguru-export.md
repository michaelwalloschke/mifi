# Finanzguru export — salvage findings

Source file: `~/Downloads/finanzguru.xlsx` (stays out of repo — privacy). Inspected 2026-07-22.

## Format

- Single-sheet XLSX, sheet name `20260722_Export_Alle_Buchungen`, 4633 data rows, 28 columns, German headers (ASCII-transliterated: `Waehrung`, `Beguenstigter`).
- Dates are real datetimes, amounts are EUR floats, `Waehrung` = `EUR` throughout.
- Columns: `Buchungstag, Referenzkonto, Name Referenzkonto, Betrag, Kontostand, Waehrung, Beguenstigter/Auftraggeber, IBAN Beguenstigter/Auftraggeber, Verwendungszweck, E-Ref, Mandatsreferenz, Glaeubiger-ID, Analyse-Hauptkategorie, Analyse-Unterkategorie, Analyse-Vertrag, Analyse-Vertragsturnus, Analyse-Vertrags-ID, Analyse-Umbuchung, Analyse-Vom frei verfuegbaren Einkommen ausgeschlossen, Analyse-Umsatzart, Analyse-Betrag, Analyse-Woche, Analyse-Monat, Analyse-Quartal, Analyse-Jahr, Buchungs-ID, Referenz-Original-ID, Split-Typ`.
- Column quirk: `Analyse-Betrag` is NOT an amount — it's the direction label (`Einnahmen`/`Ausgaben`), Finanzguru's own income/expense kind per row. `Analyse-Umsatzart` is the payment type (SEPA-Lastschrift, Kartenzahlung, Ueberweisung, Dauerauftrag, …).
- `Buchungs-ID`: 40-hex (sha1-like), unique across all 4633 rows. Finanzguru-internal — no shared identity with FinTS; dedup against future bank imports goes through mifi's Import Hash as planned.
- `Kontostand` = running balance after each booking → usable for balance validation/backfill.
- `Split-Typ` empty everywhere (splits unused), `Referenz-Original-ID` empty everywhere.

## Account coverage & history depth

| Account | Rows | Range |
|---|---|---|
| Consorsbank Giro (DE64…8070) | 3831 | 2022-05-30 → 2026-07-23 |
| Consorsbank Tagesgeld (DE07…3441) | 125 | 2022-06-28 → 2025-09-30 |
| PayPal (keyed by email, no IBAN) | 677 | 2023-02-20 → 2026-07-21 |

- **No Scalable accounts** — depot and Verrechnungskonto were never in Finanzguru. No depot history seed exists; Scalable history starts with CSV export / scalable-cli.
- ~4.2 years of Giro history. Counterparty IBAN missing on 1840 rows (card payments, PayPal).

## Categories (seed for taxonomy + auto-categorization)

- 14 main categories, 64 main/sub pairs — exactly depth-2, matching mifi's Category model.
- Every row is categorized (fallbacks land in `Sonstiges / Sonstige Ausgaben` etc.).
- Mains by volume: Essen & Trinken 1195, Freizeit 754, Lifestyle 656, Sonstiges 562, Mobilitaet 433, Wohnen 286, Finanzen 230, Einnahmen 159, Sparen 131, Versicherungen 118, Gesundheit 52, Drogerie 40, Haustiere 16, Kinder 1.
- Full pair list preserved in the export itself; notable for mifi: `Sonstiges / Kreditkartenabrechnung` (245 rows) and `Sonstiges / Bargeld` (56) are really transfer-ish, not spending — taxonomy adoption should reconsider them.
- 4.6k labeled (merchant, purpose) → category examples: strong training/seed set for the categorization approach ticket.

## Contracts (seed for recurring detection)

- `Analyse-Vertrag = ja` on 735 rows; 49 distinct `Analyse-Vertrags-ID`s with turnus: monatlich (majority), jaehrlich, vierteljaehrlich, **zweiwoechentlich** (HelloFresh).
- ⚠️ `zweiwoechentlich` (biweekly) occurs in real data but mifi's Contract interval set (CONTEXT.md) is weekly/monthly/quarterly/yearly — recurring-detection ticket must decide: add biweekly or fold into weekly-with-tolerance.
- Same merchant appears under multiple contract IDs (Otto ×3 split by amount, Spotify ×2, HelloFresh ×2, Cleverbridge ×2 split by name spelling) — Finanzguru splits per mandate/amount; detection design should decide merge policy.
- Merchant name corruption in card rows: mid-word spaces (`HelloFre sh`, `Cleverbr idge`, `JetBrain s s.r.o.`, `DisneyPl us`) — merchant normalization must strip/fuzzy-match these.

## Transfers

- `Analyse-Umbuchung = ja` on 912 rows across all three accounts. Sum ≈ −520 € ≠ 0: includes lone legs to untracked accounts (Scalable sparplan, credit card settlement), confirming mifi's lone-leg rule.
- Labeled transfer legs → validation set for the ±4-day auto-matcher.

## Other flags

- `Analyse-Vom frei verfuegbaren Einkommen ausgeschlossen = ja` on 677 rows: all 335 PayPal transfer legs + 342 Giro rows (credit-card settlements etc.) — Finanzguru's double-count guard; mifi's Transfer model covers the same need.

## Usable as seed data — verdict

1. **History**: yes — Giro from 2022-05, Tagesgeld from 2022-06 (ends 2025-09), PayPal from 2023-02. Import via a one-off `finanzguru-seed` reader (XLSX or converted CSV).
2. **Categorization examples**: yes — 4.6k labeled rows, taxonomy directly adoptable/trimmable.
3. **Contract bootstrap**: yes — 49 labeled contracts with turnus; plus ground truth to score the detector.
4. **Depot/net-worth**: no — Scalable never tracked; nothing to salvage there.
