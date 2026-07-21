# CSV import pipeline

Type: research
Status: open
Blocked by: 09

## Question

Pin the import pipeline for the two CSV sources: PayPal activity export and Scalable transaction export — column layouts, encodings, date/amount formats, which columns map to which Transaction fields (counterparty, purpose, FX metadata for PayPal). Dedup and overlap are settled by the domain model ([Domain model](09-domain-model.md): Import Hash + occurrence index, bank source wins, booked-only); remaining: per-format normalization rules feeding that hash, import UX (file picker vs watched folder), and error handling for malformed/partial files. Output: markdown summary with per-format mapping tables.
