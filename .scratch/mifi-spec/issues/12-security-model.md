# Security model

Type: research
Status: open
Blocked by: 05, 06

## Question

Where do bank credentials and scalable-cli OAuth tokens live (no aggregator per [Aggregator selection](05-aggregator-selection.md)) (OS keychain via Tauri plugin?), is the SQLite DB encrypted at rest (SQLCipher vs OS full-disk-encryption suffices?), and what does backup look like without leaking data? Decide the threat model explicitly: protecting against lost/stolen machine, not against a compromised OS. Output: markdown summary with decisions for the spec.
