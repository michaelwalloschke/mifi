# Finanzguru export salvage

Type: task
Status: open
Assignee: michael

## Question

Get the data out of Finanzguru and see what it holds. Michael requests/downloads the export (may need support request or premium); then inspect: transaction history depth, category labels, contract/recurring data, account coverage, file format. Answer records format, row counts, and what is usable as seed data for history, categorization examples, and contract detection.

## Checklist (HITL — Michael)

Session checked 2026-07-22: Finanzguru.app is installed at /Applications but has never run on this Mac — no local data to salvage. Export must come from the app/account. Two paths, in preference order:

1. **In-app Excel export** (needs Finanzguru Plus): open Finanzguru (Mac app or phone) → Profil → look for "Daten exportieren" / export option under Analysen. Choose the widest date range offered, all accounts. Expected: `.xlsx`.
2. **GDPR data request** (free, complete, slower): in-app Profil → Datenschutz, or email support (support@finanzguru.de) citing DSGVO Art. 15 Datenauskunft. Gets full stored data regardless of Plus status; turnaround up to 30 days — if path 1 works, still consider this for anything the Excel export omits (contract/recurring metadata).

Drop the file(s) into `~/Downloads` (any name containing "finanzguru") or paste the path in a session — the next session inspects format, history depth, categories, contracts, and records the answer. Raw export stays out of the repo (privacy).
