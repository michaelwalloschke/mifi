# Security Model: Credentials, Encryption at Rest, Backup

Research date: 2026-07-21. Method: official docs (Apple Platform Security, docs.rs, readthedocs, sqlite.org) and first-party GitHub repos. Claims marked *(unverified)* could not be confirmed against a primary source.

Context: mifi is local-first, single-user, macOS-only (Michael's machine). Secrets in play: FinTS PIN (Consorsbank), scalable-cli OAuth session, python-fints client-state blob. Architecture (asset 06): Rust core owns SQLite; Python sidecar speaks FinTS over stdio; scalable-cli runs as subprocess; Foldkit frontend in the Tauri webview.

## Threat model (locked)

**In scope: the machine is lost or stolen, and backup media falling into the wrong hands.** An attacker has the powered-off (or locked) device or a backup disk and wants the financial data or the bank credentials.

**Out of scope: a compromised OS.** Malware running as Michael's user can read the keychain-unlocked secrets, the DB, and process memory no matter what we build — defending that layer is the OS's job, not a desktop app's. Consequences: no in-memory zeroization gymnastics, no app-level master password, no defense against a debugger.

---

## Decision 1 — Credentials live in the macOS Keychain, accessed only from the Rust core

**FinTS PIN → macOS login keychain via the [`keyring` crate](https://github.com/open-source-cooperative/keyring-rs)** (v4.x, actively maintained, MIT/Apache) used directly in the Rust core — *not* via any Tauri keychain plugin. On macOS the crate's [apple-native-keyring-store](https://docs.rs/apple-native-keyring-store/latest/apple_native_keyring_store/) backend uses Security.framework (`security-framework` bindings); "credentials are stored in keychain entries in encrypted files" in the user's keychain.

Why no Tauri plugin: the community plugins (tauri-plugin-keyring, tauri-plugin-keychain) exist to expose the keychain **to webview JavaScript**. mifi's frontend never needs a secret — it triggers a Sync Run; the Rust core fetches the PIN from the keychain and hands it to the Python sidecar **over stdio, in memory, per session**. Keeping secrets out of the IPC surface entirely is both less code and a smaller attack surface. Rejected likewise: `tauri-plugin-stronghold` — a password-derived vault would add a second master password and Argon2 parameter management to protect against exactly nothing extra under this threat model.

**python-fints needs the PIN only per session.** The client takes the PIN as a constructor argument ([quickstart](https://python-fints.readthedocs.io/en/latest/quickstart.html)) and does not persist it. Its resumable client state (`deconstruct()`/`set_data()`) explicitly contains **no PIN and no connection secrets** — it holds the FinTS system ID, BPD, and UPD ("mostly: user name and account numbers") ([client docs](https://python-fints.readthedocs.io/en/latest/client.html)). Decision: store that blob in SQLite with the account row, `include_private=True` (account numbers are in the DB anyway). The docs recommend an HMAC against tampering; skipped — the blob only ever round-trips through our own DB, and a tampering attacker is the out-of-scope compromised-OS case.

**scalable-cli already does the right thing.** Its [README](https://github.com/ScalableCapital/scalable-cli) documents `session_backend: keyring | file`, **default `keyring` on macOS** — the OAuth session lands in the same login keychain with no work from us. Decision: keep the default; never set `file`. Login (OAuth device flow) is performed once by Michael in a terminal, per the README's own advice ("complete login yourself"); mifi only invokes read commands afterwards.

**Code-signing caveat:** macOS keychain access control is tied to the accessing binary's identity; an app whose signature changes between builds re-triggers "allow access" prompts. Expect prompts in dev builds; sign release builds with a stable identity. *(Behavior known from Apple's keychain ACL design; exact prompt behavior per keyring-rs not documented — verify once during implementation.)*

## Decision 2 — Plain SQLite; FileVault is the encryption at rest. No SQLCipher.

Apple's [Platform Security guide](https://support.apple.com/guide/security/volume-encryption-with-filevault-sec4c6dc1b6e/web): on Apple Silicon "all APFS volumes are created with a volume encryption key by default"; internal storage is always hardware-encrypted. Turning **FileVault on** wraps that key with a KEK "protected by a combination of the user's password and hardware UID" — which is precisely the lost/stolen-machine defense: without the login password the volume key is unreachable.

SQLCipher via rusqlite is available (the [`bundled-sqlcipher` and `bundled-sqlcipher-vendored-openssl` features](https://github.com/rusqlite/rusqlite) exist) but buys nothing here: its key would have to live in the keychain, which is unlocked whenever the user is logged in — identical protection window to FileVault — while costing an OpenSSL vendored build, loss of plain `sqlite3`-CLI inspectability, and a key-rotation story. Rejected.

**Spec requirement instead:** at startup, check FileVault status (`fdesetup status`) and show a persistent warning in Konten & Sync if it is off. On Apple Silicon without FileVault the volume key "is protected only by the hardware UID" — encrypted against disk removal, but not against someone powering on the stolen Mac.

Revisit SQLCipher only if the DB ever needs to leave the FileVault boundary (cloud sync — explicitly out of scope).

## Decision 3 — Backup = encrypted Time Machine + a `VACUUM INTO` export; no custom crypto

- The DB is one plain SQLite file in the app-data dir; **Time Machine with the "Encrypt Backup" option** ([Apple: choose a backup disk and set encryption](https://support.apple.com/guide/mac-help/choose-a-backup-disk-set-encryption-options-mh11421/mac)) covers it. Spec documents "use encrypted Time Machine" as the supported backup path; mifi builds no backup scheduler.
- Naively copying an open SQLite file risks a torn copy. mifi gets one **"Export backup…" action** that runs [`VACUUM INTO`](https://sqlite.org/lang_vacuum.html) — SQLite's documented "alternative to the backup API for generating backup copies of a live database", which also purges deleted content from the copy. That file is a plain snapshot the user can drop anywhere they trust (their encrypted disk, their own archive tooling). No age/passphrase archive exporter — encryption of media is the OS's job under this threat model.
- **Keychain items do not travel with file backups.** Restore on a new machine = copy DB back, re-enter the FinTS PIN once, re-run `scalable-cli` login once. Two prompts on a once-per-machine-lifetime event — acceptable; document it in the spec's restore note.

## Decision 4 — Hygiene rules (spec-level invariants)

- **Never persisted anywhere** (DB, config.toml, logs, error messages): FinTS PIN, TANs, OAuth tokens. The TAN from the sync modal passes UI → Rust → sidecar stdin and dies with the dialog.
- **Sidecar logging:** python-fints logs FinTS wire traffic at DEBUG; ship the sidecar at INFO and never write its stderr to a persisted log file at DEBUG level. *(Exact logger names unverified — pin during implementation.)*
- **Tauri surface:** keep the default strict CSP, load no remote content (the app is fully local — this is already the plan), and expose no secret-returning Tauri command to the webview. One line in the spec, enforced by the Decision-1 architecture.
- **No zeroize/mlock:** out-of-scope threat (compromised OS / memory forensics on a running machine).

## Open questions

- None blocking the spec. Two implementation-time verifications flagged above: keychain prompt behavior across dev/release signatures, and python-fints logger configuration.
- scalable-cli is beta ([Apply for scalable-cli beta](../issues/15-scalable-cli-beta.md)); if the beta ever forces `session_backend: file`, wrap the config dir permissions check into the FileVault startup warning. *(Speculative — default is keyring today.)*

## Sources

- https://support.apple.com/guide/security/volume-encryption-with-filevault-sec4c6dc1b6e/web
- https://support.apple.com/guide/mac-help/choose-a-backup-disk-set-encryption-options-mh11421/mac
- https://github.com/open-source-cooperative/keyring-rs · https://docs.rs/apple-native-keyring-store/latest/apple_native_keyring_store/
- https://github.com/rusqlite/rusqlite (feature list)
- https://python-fints.readthedocs.io/en/latest/client.html · https://python-fints.readthedocs.io/en/latest/quickstart.html
- https://sqlite.org/lang_vacuum.html
- https://github.com/ScalableCapital/scalable-cli (session_backend, config paths)
