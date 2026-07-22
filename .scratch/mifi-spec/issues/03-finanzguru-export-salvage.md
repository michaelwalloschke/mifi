# Finanzguru export salvage

Type: task
Status: closed
Assignee: michael

## Question

Get the data out of Finanzguru and see what it holds. Michael requests/downloads the export (may need support request or premium); then inspect: transaction history depth, category labels, contract/recurring data, account coverage, file format. Answer records format, row counts, and what is usable as seed data for history, categorization examples, and contract detection.

## Checklist (HITL — Michael)

Session checked 2026-07-22: Finanzguru.app is installed at /Applications but has never run on this Mac — no local data to salvage. Export must come from the app/account. Two paths, in preference order:

1. **In-app Excel export** (needs Finanzguru Plus): open Finanzguru (Mac app or phone) → Profil → look for "Daten exportieren" / export option under Analysen. Choose the widest date range offered, all accounts. Expected: `.xlsx`.
2. **GDPR data request** (free, complete, slower): in-app Profil → Datenschutz, or email support (support@finanzguru.de) citing DSGVO Art. 15 Datenauskunft. Gets full stored data regardless of Plus status; turnaround up to 30 days — if path 1 works, still consider this for anything the Excel export omits (contract/recurring metadata).

Drop the file(s) into `~/Downloads` (any name containing "finanzguru") or paste the path in a session — the next session inspects format, history depth, categories, contracts, and records the answer. Raw export stays out of the repo (privacy).

## Resolution (2026-07-22)

Michael exported via path 1 (in-app Excel export) → `~/Downloads/finanzguru.xlsx`. Full findings: [Finanzguru export salvage findings](../assets/03-finanzguru-export.md).

Gist: single-sheet XLSX, 4633 rows, 28 German columns. Consorsbank Giro 2022-05→now (3831 rows), Tagesgeld 2022-06→2025-09 (125), PayPal 2023-02→now (677); **no Scalable** — no depot history seed. Every row categorized in a depth-2 taxonomy (14 main / 64 pairs) → strong categorization seed. 49 labeled contracts with turnus incl. **zweiwoechentlich** (not in mifi's interval set — flagged on [Recurring/contract detection](11-recurring-detection.md)). 912 labeled transfer legs incl. lone legs. Running balance column present. Buchungs-ID is Finanzguru-internal; dedup stays on Import Hash. GDPR path not needed.
