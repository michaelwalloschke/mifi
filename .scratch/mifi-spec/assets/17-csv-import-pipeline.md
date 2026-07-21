# 17 — CSV Import Pipeline: PayPal & Scalable Capital formats

Researched 2026-07-21 against official docs, open-source parser source code, and real-world CSV fixtures found in public repos. Claims that can only be confirmed against Michael's own export are marked **UNVERIFIED**.

## TL;DR

- **PayPal activity export** (German account): UTF-8 **with BOM**, comma-separated, every field double-quoted, 41 columns. Date `DD.MM.YYYY`, separate `Uhrzeit` + `Zeitzone` (CET/CEST) columns, German decimals (`1.234,56`), leading minus for debits. Filter rows to `Auswirkung auf Guthaben ∈ {Soll, Haben}` (drop `Memo`). Unique ID: `Transaktionscode`; FX legs link via `Zugehöriger Transaktionscode`.
- **Scalable Capital broker export**: single-line header `date;time;status;reference;description;assetType;type;isin;shares;price;amount;fee;tax;currency`. Semicolon-separated, ISO date `YYYY-MM-DD`, **German decimals despite English headers** (`7,34`, thousands dot `1.526,72`). Types = the broker's order-type filter list: Buy, Sell, Savings plan, Deposit, Withdrawal, Corporate Action, Distribution, Fee, Interest, Security transfer, Taxes. Export gated to **PRIME/PRIME+** tiers.
- Both formats map cleanly onto the Import-Hash fields (booking date, amount, normalized counterparty, normalized purpose); both also carry a stable native transaction ID worth storing as `external_ref` metadata.
- "Saveback" is a **Trade Republic** feature, not Scalable — it will not appear in Scalable exports.

---

## 1. PayPal activity export (Aktivitäten-Download, German account)

### 1.1 Download variants

PayPal offers two distinct CSV products; they have **different headers** — the importer must detect which one it got:

| Variant | Where | Header style | Notes |
|---|---|---|---|
| **Aktivitäten-Export** (custom report) | Aktivitäten → Download-Symbol → Zeitraum wählen → CSV ([PayPal Hilfe help145](https://www.paypal.com/de/cshelp/article/wie-kann-ich-kontoausz%C3%BCge-und-berichte-anzeigen-und-herunterladen-help145)) | `Typ`, `Gebühr`, `Auswirkung auf Guthaben` | Up to 7 years history, max 12 months per report, 50 000 rows per CSV (larger → ZIP split) ([PayPal developer docs, Activity Download](https://developer.paypal.com/docs/reports/online-reports/activity-download/)). Column set is user-configurable in "Berichtsfelder anpassen"; the 41-column list below is the full/default set. In the report dialog the *Transaktionstyp* dropdown offers "Alle Transaktionen" vs. narrower sets ("Abgeschlossene Zahlungen", balance-affecting only) — **UNVERIFIED** which options Michael's personal account currently shows; PayPal has changed this dialog repeatedly ([tabelle.at guide](https://www.tabelle.at/csv-tabelle/paypal/), [pathway-solutions guide](https://hilfe.pathway-solutions.de/de/knowledge/paypal/transaktionsbericht-csv-format-herunterladen)). |
| **Monatlicher Kontoauszug** (statement) | paypal.com/reports/accountStatements → "Anfordern" | `Beschreibung` (instead of `Typ`), `Entgelt` (instead of `Gebühr`), extra `Name der Bank`, `Bankkonto`; **no** `Auswirkung auf Guthaben` | Evidence: header aliases in the [k-fin PayPal importer](https://github.com/max5800/k-fin/blob/main/src/normalization/paypal_csv.py) which ingests exactly this statement CSV. Full column list **UNVERIFIED**. |

**Recommendation for mifi**: use the **Aktivitäten-Export with "Alle Transaktionen"** and filter on `Auswirkung auf Guthaben`. It is the variant every open-source importer targets, it carries the balance-impact column (clean row filtering), and 12-month spans beat monthly statements. Support the Kontoauszug header aliases (`Beschreibung`→type, `Entgelt`→fee) as a cheap fallback.

### 1.2 File format

- **Encoding**: UTF-8 **with BOM** — official spec says UTF-8 ([PayPal developer docs](https://developer.paypal.com/docs/reports/online-reports/activity-download/)); the BOM (`﻿` before `"Datum"`) is visible in real fixtures ([kontor test fixture](https://github.com/replikativ/kontor/blob/main/modules/bank-de/test/resources/paypal.csv), [paypal2homebank fixture](https://github.com/sercxanto/small_scripts/blob/master/_archive/tests/paypal2homebank/paypal.csv)) and paypal2homebank explicitly opens with `encoding="utf-8-sig"`.
- **Separator**: comma. **Quoting**: every field wrapped in `"…"` (both fixtures). Fields may contain commas and semicolons — a real CSV parser is required, not `split(',')`.
- **Date**: `DD.MM.YYYY` (`"18.06.2025"`). **Time**: `HH:MM:SS` in `Uhrzeit`. **Timezone**: `Zeitzone` column with values `CET`/`CEST` (fixtures; the [beancount-paypal importer](https://github.com/nils-werner/beancount-paypal/blob/master/beancount_paypal/lang.py) parses `%d.%m.%Y` for German).
- **Amounts**: German format — decimal comma, dot as thousands separator (`"1.234,56"`), leading `-` for debits (`"-41,00"`). All parsers strip `.` then swap `,`→`.` ([OpenAccounting GermanActivityPayPalCSVParser](https://github.com/aczwink/OpenAccounting/blob/main/backend/src/payment-parsers/GermanActivityPayPalCSVParser.ts), beancount-paypal).
- `Gebühr` is itself **negative** when a fee was charged (`"-1,94"`); `Netto = Brutto + Gebühr` (kontor/sercxanto fixtures).
- `Guthaben` is the **running PayPal balance after the row**, in the row's currency.

### 1.3 Exact column layout (Aktivitäten-Export, full set — 41 columns)

Confirmed verbatim by two independent real-export fixtures ([kontor](https://github.com/replikativ/kontor/blob/main/modules/bank-de/test/resources/paypal.csv), [sercxanto](https://github.com/sercxanto/small_scripts/blob/master/_archive/tests/paypal2homebank/paypal.csv)):

```
"Datum","Uhrzeit","Zeitzone","Name","Typ","Status","Währung","Brutto","Gebühr","Netto",
"Absender E-Mail-Adresse","Empfänger E-Mail-Adresse","Transaktionscode","Lieferadresse",
"Adress-Status","Artikelbezeichnung","Artikelnummer","Versand- und Bearbeitungsgebühr",
"Versicherungsbetrag","Umsatzsteuer","Option 1 Name","Option 1 Wert","Option 2 Name",
"Option 2 Wert","Zugehöriger Transaktionscode","Rechnungsnummer","Zollnummer","Anzahl",
"Empfangsnummer","Guthaben","Adresszeile 1","Adresszusatz","Ort","Bundesland","PLZ",
"Land","Telefon","Betreff","Hinweis","Ländervorwahl","Auswirkung auf Guthaben"
```

Since the field set is user-configurable at report creation, **map by header name, never by position** (k-fin does exactly this and errors loudly on missing required headers). Required minimum: `Datum, Name, Typ, Status, Währung, Brutto, Transaktionscode, Auswirkung auf Guthaben`.

Key semantic columns:

| Column | Meaning |
|---|---|
| `Name` | Counterparty display name (merchant/person). Empty on PayPal-internal plumbing rows. |
| `Absender/Empfänger E-Mail-Adresse` | Sender/recipient identity. Which one is "the other party" depends on direction: incoming (`Haben`) → sender is counterparty; outgoing (`Soll`) → recipient is (OpenAccounting logic). |
| `Typ` | German transaction-type label, e.g. `PayPal Express-Zahlung`, `Website-Zahlung`, `Allgemeine Zahlung`, `Handyzahlung`, `Bankgutschrift auf PayPal-Konto`, `Allgemeine Abbuchung` (payout to bank), `Allgemeine Währungsumrechnung` (FX leg), `Allgemeine Einbehaltung`/`Freigabe allgemeiner Einbehaltung`/`Einbehaltung für offene Autorisierung`/`Rückbuchung allgemeiner Einbehaltung` (holds), `Überweisung als Zahlungsquelle`. |
| `Status` | `Abgeschlossen`, `Ausstehend`, … Only import `Abgeschlossen` (pending ≠ Transaction per CONTEXT.md). |
| `Transaktionscode` | Unique, stable 17-char transaction ID. |
| `Zugehöriger Transaktionscode` | Links a child row (FX leg, hold release, refund) to its parent transaction. |
| `Betreff` / `Hinweis` | User-visible subject / note — the purpose text. `Artikelbezeichnung` as fallback. |
| `Rechnungsnummer` | Merchant invoice number. |
| `Auswirkung auf Guthaben` | `Soll` (debit) / `Haben` (credit) / `Memo` (no balance effect) — German rendering of official Debit/Credit/Memo ([PayPal developer docs](https://developer.paypal.com/docs/reports/online-reports/activity-download/)). |

### 1.4 Balance-affecting vs memo rows

Import a row as a Transaction only if:

1. `Auswirkung auf Guthaben` is `Soll` or `Haben` — `Memo` rows (money requests, denied/uncompleted events) never moved the balance ([paypal2homebank](https://github.com/sercxanto/small_scripts/blob/master/_archive/paypal2homebank.py) skips exactly these; official semantics per PayPal developer docs), **and**
2. `Status` = `Abgeschlossen`, **and**
3. `Typ` is not a hold/release pair (`*Einbehaltung*`) — these net to zero and disappear again (OpenAccounting skips them), **and**
4. `Typ` ≠ `Allgemeine Währungsumrechnung` — FX legs are folded into their parent (below).

Caution: with an EUR-funded German account, `Bankgutschrift auf PayPal-Konto` (bank top-up) and `Allgemeine Abbuchung` (withdrawal to bank) **are** balance-affecting and must be imported — they are the PayPal-side legs of Transfers to/from Consorsbank Giro (k-fin treats them the same way).

### 1.5 Multi-currency / FX

A foreign-currency payment produces a **row triple** (evidence: [beancount-paypal importer](https://github.com/nils-werner/beancount-paypal/blob/master/beancount_paypal/__init__.py), [k-fin `_harvest_eur_conversions`](https://github.com/max5800/k-fin/blob/main/src/normalization/paypal_csv.py), [StarMoney forum report of multi-record bookings](https://www.starmoney.de/forum/viewtopic.php?t=43799)):

1. The payment row in the **original currency** (e.g. `Währung=USD`, `Brutto=-10,99`), balance impact on the USD sub-balance;
2. an `Allgemeine Währungsumrechnung` row **debiting EUR** (`Währung=EUR`, negative) — this is the real EUR cost;
3. an `Allgemeine Währungsumrechnung` row **crediting the foreign currency** (offsets row 1).

Both conversion legs carry the payment's ID in `Zugehöriger Transaktionscode`. FX fees are embedded in the conversion rate, not a separate fee column.

**mifi rule** (per CONTEXT.md: EUR-only, FX is inert metadata): two-pass import. Pass 1 harvests `Allgemeine Währungsumrechnung` EUR legs into `{Zugehöriger Transaktionscode → EUR amount}`. Pass 2: for a non-EUR payment row, book **the EUR leg's amount** as `amount_cents` and store `{original_currency, original_amount}` from the payment row as display metadata. All three raw rows collapse into one Transaction. A non-EUR payment with no matching EUR leg → import error surfaced to the user, never a silent EUR=foreign assumption.

---

## 2. Scalable Capital transaction export (Baader Bank backend)

### 2.1 Availability & location

Broker → Transaktionen → filter/select → **"Export CSV"** button. Available in the **PRIME and PRIME+ broker tiers** ("With PRIME and PRIME+ exporting all Broker transactions is just 2 clicks away" — [Scalable product news](https://de.scalable.capital/en/product-news/transactions-export); confirmed by [Parqet FAQ](https://faq.parqet.com/de/articles/651200-scalable-capital-pdf-csv-import): "Prime- und Prime+-Nutzer haben außerdem die Möglichkeit, eine CSV-Datei mit der gesamten Transaktionshistorie zu exportieren"). **Check Michael's tier** — on FREE the fallback is per-transaction PDFs or the scalable-cli Source. Official FAQ: [How can I view and export my transactions?](https://help.scalable.capital/en/account-management-f3197dc7/how-can-i-view-and-export-my-transactions-b0b78717)

### 2.2 Exact column layout

Header (verbatim from real exports posted in [Portfolio Performance forum](https://forum.portfolio-performance.info/t/csv-import-von-scalable-capital/30113) and [Export-To-Ghostfolio issue #272](https://github.com/dickwolff/Export-To-Ghostfolio/issues/272)):

```
date;time;status;reference;description;assetType;type;isin;shares;price;amount;fee;tax;currency
```

Real sample rows:

```
2024-10-23;13:10:35;Executed;"SCALHGJwmX8Bo9W";"Uranium Energy Co";Security;Buy;US9168961038;80;7,34;-587,20;0,00;0,00;EUR
2024-10-22;16:04:22;Executed;"SCALNAy7S3rcUkQ";"Uranium Energy Co";Security;Sell;US9168961038;208;7,34;1.526,72;0,00;112,23;EUR
2025-08-01;14:11:01;Executed;"SCALT68r1eJqD4Q";"iShares BIC 50 (Dist)";Security;Savings plan;IE00B1W57M07;3,505;21,395;-74,989475;0,00;0,00;EUR
2025-09-25;02:00:00;Executed;"WWEK 51597383";"iShares Core FTSE 100 (Dist)";Cash;Distribution;IE0005042456;;;56,78;0,00;0,00;EUR
```

### 2.3 Format details

- **Separator**: semicolon. **Quoting**: `reference` and `description` quoted; dates/numbers unquoted.
- **Headers English, numbers German**: decimal comma (`7,34`), dot thousands separator (`1.526,72`).
- **Date**: ISO `YYYY-MM-DD`; **time** `HH:MM:SS` (timezone not stated in file — assume Europe/Berlin local; **UNVERIFIED**).
- **Sign convention**: `amount` negative = cash out (Buy, Withdrawal, Fee), positive = cash in (Sell, Distribution, Deposit, Interest).
- **Precision quirk**: `amount` can carry **more than 2 decimals** (`-74,989475` = shares × price for a fractional savings-plan execution). The actually booked cash amount is the cent-rounded value — rounding rule **UNVERIFIED**, needs a real export cross-checked against the settlement PDF.
- `fee` and `tax` are separate columns, positive values, already reflected in… **UNVERIFIED** whether `amount` is net or gross of `fee`/`tax` (the Sell sample: `1.526,72` with `tax 112,23` — whether the credit was 1.526,72 or 1.414,49 must be checked against the Verrechnungskonto booking).
- **Encoding**: **UNVERIFIED** (expected UTF-8, no BOM reports found).
- `status` values: `Pending, Executed, Cancelled, Expired, Rejected` ([official FAQ](https://help.scalable.capital/en/account-management-f3197dc7/how-can-i-view-and-export-my-transactions-b0b78717)) — import only `Executed`.

### 2.4 Transaction types & row shapes

Official order-type filter list = the `type` vocabulary ([Scalable FAQ](https://help.scalable.capital/en/account-management-f3197dc7/how-can-i-view-and-export-my-transactions-b0b78717)):
`Buy, Sell, Savings plan, Deposit, Withdrawal, Corporate Action, Distribution, Fee, Interest, Security transfer, Taxes`.

- `assetType` distinguishes `Security` rows (isin+shares+price filled) from `Cash` rows (Deposit/Withdrawal/Distribution/Interest; shares/price empty — note Distribution is `Cash` but still carries the ISIN).
- `Savings plan` = ETF-Sparplan execution with fractional `shares` (up to 6 decimals).
- **Deposit/Withdrawal** rows are moves between the reference account (Consorsbank Giro) and the Verrechnungskonto → in mifi these are **Transfer legs**, and they answer the question of which Account the row belongs to: `Security` rows change the depot, `Cash` rows change the Verrechnungskonto balance (a Buy is simultaneously a Verrechnungskonto debit — decide whether depot Transactions are modeled at all or only Snapshots; CONTEXT.md leans Snapshot-based for the depot).
- **PRIME+ interest** on cash appears as `type=Interest` rows; `Taxes` can appear as separate line items ([Export-To-Ghostfolio issue](https://github.com/dickwolff/Export-To-Ghostfolio/issues/272)). Exact shapes of `Interest`, `Fee`, `Corporate Action`, `Security transfer` rows: **UNVERIFIED**.
- **Saveback does not exist at Scalable** — it is a Trade Republic feature; no such type will appear.
- `reference` formats seen: `SCAL…` (orders), `WWEK …` (distributions) — treat as opaque string, unique per row (**uniqueness UNVERIFIED**).

---

## 3. Mapping tables (CSV column → mifi Transaction field)

mifi Transaction fields per [CONTEXT.md](/Users/michael/development/mifi/CONTEXT.md): Amount = EUR integer cents; Import Hash = hash(booking date, amount, normalized counterparty, normalized purpose) + occurrence index; FX originals inert metadata.

### 3.1 PayPal (`source = csv-paypal`, Account = PayPal)

| mifi field | CSV column(s) | Rule |
|---|---|---|
| `booked_at` (date) | `Datum` | `DD.MM.YYYY` → ISO date. Time/`Uhrzeit`+`Zeitzone` optionally kept as metadata; **date only** feeds the Import Hash (FinTS/other sources won't have time). |
| `amount_cents` | `Brutto` (EUR rows) / EUR FX leg's `Netto` or `Brutto` (non-EUR rows) | German decimal → integer cents. Use `Brutto`: for a private account fees are rare and `Netto=Brutto+Gebühr`; when `Gebühr≠0` the balance moves by `Netto` → **use `Netto` where present, else `Brutto`** so the account ledger sums to `Guthaben`. |
| `counterparty` | `Name`; fallback direction-aware email (`Haben`→`Absender E-Mail-Adresse`, `Soll`→`Empfänger E-Mail-Adresse`) | Trim; empty `Name` + empty mail → literal `"PayPal"`. |
| `purpose` | `Betreff`, else `Hinweis`, else `Artikelbezeichnung`; append `Rechnungsnummer` if present | Join non-empty with `", "` (paypal2homebank pattern). |
| `external_ref` (metadata, not hashed) | `Transaktionscode` | Stable unique ID — store for idempotent re-import diagnostics and FX/refund linking via `Zugehöriger Transaktionscode`. |
| `type_hint` (metadata) | `Typ`, `Auswirkung auf Guthaben` | Drives Transfer detection (`Bankgutschrift auf PayPal-Konto`, `Allgemeine Abbuchung` ↔ Consorsbank legs). |
| FX metadata | payment row's `Währung` + `Brutto` when ≠ EUR | `{original_currency, original_amount}` display-only. |
| balance check (optional) | `Guthaben` of last row | Sanity-check against Snapshot. |

Row filter before mapping: keep `Auswirkung auf Guthaben ∈ {Soll, Haben}` ∧ `Status = Abgeschlossen` ∧ `Typ ∉ {holds, Allgemeine Währungsumrechnung}` (§1.4/§1.5).

### 3.2 Scalable (`source = csv-scalable`, Account = Verrechnungskonto / depot)

| mifi field | CSV column(s) | Rule |
|---|---|---|
| `booked_at` | `date` | Already ISO. `time` as metadata only. |
| `amount_cents` | `amount` | German decimal → **round half-away-from-zero to cents** (fractional-execution quirk, §2.3); rounding rule to confirm against real booking. |
| `counterparty` | `description` (+ `type` for Cash rows) | Security rows: instrument name; Deposit/Withdrawal: description names the counter-account — real content **UNVERIFIED**. |
| `purpose` | `type` + `description` + `isin` + `shares`@`price` | e.g. `"Savings plan iShares BIC 50 (Dist) IE00B1W57M07 3,505 @ 21,395"` — deterministic concatenation. |
| `external_ref` (metadata) | `reference` | Opaque broker reference. |
| depot metadata | `isin`, `shares`, `price`, `assetType` | Feeds Position/Snapshot logic, not the Transaction hash. |
| fees/taxes | `fee`, `tax` | Keep as metadata; whether to split depends on the net/gross answer (§2.3). |
| currency | `currency` | Expect `EUR` always; any other value → import error (single-currency rule). |

Row filter: `status = Executed` only. Account routing: `assetType = Cash` → Verrechnungskonto Transaction; `assetType = Security` → depot event (Snapshot/Position input + matching Verrechnungskonto cash effect — model decision pending).

---

## 4. Normalization rules feeding the Import Hash

Shared pipeline, per format-specific decode:

1. **Decode**: PayPal → `utf-8-sig` (strips BOM); Scalable → `utf-8` (fallback `latin-1` with warning; encoding UNVERIFIED). Never trust extension — sniff separator from the header line (`,` + `"Datum"` vs `;` + `date`), which doubles as format auto-detection.
2. **Parse CSV properly** (quoted fields contain commas/semicolons/newlines): Rust `csv` crate with the sniffed delimiter, headers mapped **by name** with an alias table (`Typ|Beschreibung`, `Gebühr|Entgelt`); missing required header = loud error naming the column (k-fin pattern).
3. **Decimal**: strip `.` (thousands), swap `,` → `.`, parse as decimal, × 100 → integer cents; Scalable amounts with >2 decimals: round half-away-from-zero, flag row for review until rounding rule verified.
4. **Date**: PayPal `strptime("%d.%m.%Y")`; Scalable `%Y-%m-%d`. Store as date; time+timezone in metadata only so hashes are stable across sources.
5. **Counterparty/purpose normalization** (pre-hash, both formats): trim, collapse internal whitespace, casefold, strip `PAYPAL *`/`PP*` processor prefixes (k-fin's `strip_paypal_prefix`) — so a later FinTS row `PayPal Europe … Steamgames` and the PayPal-CSV `Steam` row can be matched by the Transfer/dedup logic.
6. **Row filtering before hashing** (so re-exports with different report settings dedupe identically): PayPal §1.4 + FX collapse §1.5; Scalable `Executed` only.
7. **Occurrence index**: after filtering, number identical (date, amount, counterparty, purpose) tuples in file order — PayPal same-day identical micro-payments and Scalable same-day repeated savings-plan buys both need it.
8. **Idempotency belt-and-braces**: additionally store `(source, external_ref)` (`Transaktionscode` / `reference`) and skip exact re-imports on that key before hashing — cheaper and immune to purpose-normalization drift.

---

## 5. Open questions — need Michael's real export files

1. **PayPal report dialog today**: which *Transaktionstyp* options exist ("Alle Transaktionen" / "Abgeschlossene Zahlungen" / balance-affecting) and whether the chosen option changes the column set. Docs conflict across years.
2. **PayPal column set**: fixtures show the 41-column layout, but fields are user-selectable at report creation — confirm Michael's default export header verbatim.
3. **PayPal FX triple**: confirm the three-row pattern and that exactly one conversion leg is EUR, incl. sign conventions, from a real foreign-currency purchase.
4. **PayPal Kontoauszug variant**: full header list of the monthly statement CSV (only partial evidence via k-fin aliases) — decide whether to support it at all.
5. **Scalable tier**: does Michael's account (FREE?) have the "Export CSV" button — export is documented as PRIME/PRIME+ only.
6. **Scalable `amount` vs `fee`/`tax`**: net or gross? Check one Sell with tax against the Verrechnungskonto booking.
7. **Scalable rounding**: booked cash value of a fractional savings-plan execution (`-74,989475` → `-74,99`?).
8. **Scalable encoding/BOM**, `reference` uniqueness, and exact row shapes for `Interest` (PRIME+), `Fee`, `Corporate Action`, `Security transfer`.

### Sources

- PayPal Hilfe: https://www.paypal.com/de/cshelp/article/wie-kann-ich-kontoausz%C3%BCge-und-berichte-anzeigen-und-herunterladen-help145
- PayPal developer, Activity Download spec: https://developer.paypal.com/docs/reports/online-reports/activity-download/
- Real German export fixtures: https://github.com/replikativ/kontor/blob/main/modules/bank-de/test/resources/paypal.csv , https://github.com/sercxanto/small_scripts/blob/master/_archive/tests/paypal2homebank/paypal.csv
- Parsers: https://github.com/aczwink/OpenAccounting/blob/main/backend/src/payment-parsers/GermanActivityPayPalCSVParser.ts , https://github.com/nils-werner/beancount-paypal , https://github.com/max5800/k-fin/blob/main/src/normalization/paypal_csv.py , https://github.com/sercxanto/small_scripts/blob/master/_archive/paypal2homebank.py
- Scalable FAQ: https://help.scalable.capital/en/account-management-f3197dc7/how-can-i-view-and-export-my-transactions-b0b78717
- Scalable product news: https://de.scalable.capital/en/product-news/transactions-export
- Real Scalable CSV samples: https://forum.portfolio-performance.info/t/csv-import-von-scalable-capital/30113 , https://github.com/dickwolff/Export-To-Ghostfolio/issues/272
- Parqet: https://faq.parqet.com/de/articles/651200-scalable-capital-pdf-csv-import
