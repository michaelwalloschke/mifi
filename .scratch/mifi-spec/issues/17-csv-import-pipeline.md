# CSV import pipeline

Type: research
Status: closed
Assignee: michael
Blocked by: 09

## Question

Pin the import pipeline for the two CSV sources: PayPal activity export and Scalable transaction export — column layouts, encodings, date/amount formats, which columns map to which Transaction fields (counterparty, purpose, FX metadata for PayPal). Dedup and overlap are settled by the domain model ([Domain model](09-domain-model.md): Import Hash + occurrence index, bank source wins, booked-only); remaining: per-format normalization rules feeding that hash, import UX (file picker vs watched folder), and error handling for malformed/partial files. Output: markdown summary with per-format mapping tables.

## Resolution

Full findings with citations and mapping tables: [CSV import pipeline summary](../assets/17-csv-import-pipeline.md).

**Formats pinned:**

- **PayPal Aktivitäten-Export** (target variant: "Alle Transaktionen"): UTF-8 with BOM (`utf-8-sig`), comma-separated, all fields quoted, 41-column German header (confirmed by two independent real fixtures). `DD.MM.YYYY` + `Uhrzeit`/`Zeitzone` (CET/CEST), German decimals (`1.234,56`), leading minus. Row filter: `Auswirkung auf Guthaben ∈ {Soll, Haben}` and `Status = Abgeschlossen` — drops Memo/hold/release rows. `Transaktionscode` = stable native ID; `Zugehöriger Transaktionscode` links children. FX purchase = row triple (foreign leg + two `Allgemeine Währungsumrechnung` legs); book the EUR leg as `amount_cents`, keep original currency/amount as inert FX metadata. Monatlicher Kontoauszug variant (header aliases `Beschreibung`/`Entgelt`, no balance-impact column) supported as cheap fallback via header aliasing.
- **Scalable Capital export**: `date;time;status;reference;description;assetType;type;isin;shares;price;amount;fee;tax;currency` — semicolon-separated, ISO dates, German decimals despite English headers. Types: Buy, Sell, Savings plan, Deposit, Withdrawal, Corporate Action, Distribution, Fee, Interest, Security transfer, Taxes. Export gated to PRIME/PRIME+ tiers ("Saveback" is Trade Republic — never appears).
- **Mapping + normalization**: per-format column→Transaction tables in the asset; normalization (decode, decimal/date parsing, row filtering, counterparty/purpose normalization) feeds the Import Hash per the domain model; native IDs stored as `(source, external_ref)` second idempotency belt.

**Import UX: manual file picker + drag-and-drop onto the import screen. No watched folder** — imports are monthly-cadence, and Import-Hash idempotency makes manual re-import safe; a watcher adds background machinery for zero decisions saved.

**Error handling:**
- Variant detection by header row; unknown header or undecodable encoding → reject whole file with a clear message, nothing written.
- Malformed individual rows (unparseable date/amount) → import valid rows, report skipped rows with reasons; no silent drops.
- Truncated/partial files and retries are safe by construction: re-importing the corrected file is idempotent (Import Hash + occurrence index + external_ref), so partial imports self-heal on re-import.

Facts confirmable only against real exports (report-dialog variants, Michael's actual header set, FX sign conventions, Scalable tier/encoding/`reference` uniqueness) are listed as UNVERIFIED in the asset → spawned [Verify CSV formats against real exports](19-csv-format-verification.md).
