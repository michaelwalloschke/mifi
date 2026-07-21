# Security model

Type: research
Status: closed
Assignee: michael
Blocked by: 05, 06

## Question

Where do bank credentials and scalable-cli OAuth tokens live (no aggregator per [Aggregator selection](05-aggregator-selection.md)) (OS keychain via Tauri plugin?), is the SQLite DB encrypted at rest (SQLCipher vs OS full-disk-encryption suffices?), and what does backup look like without leaking data? Decide the threat model explicitly: protecting against lost/stolen machine, not against a compromised OS. Output: markdown summary with decisions for the spec.

## Resolution

Full findings with citations: [Security model summary](../assets/12-security-model.md).

**Threat model locked: lost/stolen machine + backup media. Compromised OS explicitly out of scope** — no zeroize, no app master password, no anti-debugger measures.

1. **Credentials → macOS login keychain, accessed only from the Rust core** via the `keyring` crate (Security.framework backend). No Tauri keychain plugin and no Stronghold: the webview never needs a secret — the Rust core fetches the FinTS PIN and hands it to the Python sidecar per-session over stdio, keeping secrets off the IPC surface entirely. python-fints takes the PIN per session and its `deconstruct()` client-state blob contains no secrets per its docs → stored plain in SQLite with the account row, no HMAC. scalable-cli already defaults to `session_backend: keyring` on macOS — keep the default, never set `file`; OAuth device login done once by Michael in a terminal.
2. **DB encryption: plain SQLite, FileVault is the encryption at rest. SQLCipher rejected** — its key would sit in the same keychain, giving an identical protection window at the cost of a vendored-OpenSSL build, lost CLI inspectability, and key rotation. Spec requirement: startup `fdesetup status` check with persistent warning in Konten & Sync when FileVault is off. Revisit only if the DB ever leaves the FileVault boundary.
3. **Backup: encrypted Time Machine as the documented path + one "Export backup…" action using `VACUUM INTO`** (SQLite's documented live-backup method; naive file copy of an open DB risks torn copies). No custom crypto/exporter. Keychain items don't travel with file backups → restore = re-enter PIN once + re-run CLI login once; documented as acceptable.
4. **Hygiene invariants for the spec:** PIN/TAN/OAuth tokens never in DB, config, or logs; TAN dies with the sync modal; sidecar logs at INFO (python-fints DEBUG logs wire traffic); default strict Tauri CSP, no remote content, no secret-returning Tauri command.

Open questions: none blocking the spec. Implementation-time checks flagged in the asset: keychain prompt behavior across dev/release code signatures, python-fints logger names, and (speculative) scalable-cli beta changing its token backend.
