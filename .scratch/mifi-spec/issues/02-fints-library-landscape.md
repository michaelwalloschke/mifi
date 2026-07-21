# FinTS library landscape

Type: research
Status: closed
Assignee: michael

## Question

What FinTS/HBCI client libraries exist per language (Python python-fints, JS/TS fints, Rust, Java hbci4j, ...)? For each: maturity, PSD2/TAN-flow support (decoupled TAN, photoTAN, chipTAN), transaction fetch (HKKAZ/camt), depot positions (HKWPD), registration requirements (FinTS product ID from Deutsche Kreditwirtschaft), and licensing. Output: markdown summary ranking viable options — this drives the backend stack decision.

## Resolution

Full findings with citations: [FinTS library landscape summary](../assets/02-fints-library-landscape.md).

Ranking (viable options for the Tauri app):

1. **python-fints** (LGPL-3, v5.0.0 Jan 2026, active) — best fit. Cleanest TAN challenge/response API: `NeedTANResponse` with `decoupled` flag, photoTAN image bytes, chipTAN flicker data, serializable TAN state for dialog-and-resume UX. HKKAZ + HKCAZ + HKWPD.
2. **lib-fints** (robocode13, TypeScript, LGPL-2.1, v1.4.8 Jun 2026) — the real JS option: active, PSD2 + decoupled TAN, HKKAZ/HKCAZ/HKWPD. NOT the abandoned Prior99 `fints` (dead since 2020, PSD2 incomplete).
3. **hbci4j** (LGPL-2.1, Hibiscus engine, v3.1.88 Apr 2026) — most battle-tested, handles Consorsbank out of the box, but means shipping a JVM.
4. **fints-rs** (Rust, MIT, v0.1.0 created 2026-04-06) — too young to bet on; no other Rust client exists, only helper crates (`fints-institute-db`, `mt940`).

Consorsbank specifics: URL `https://brokerage-hbci.consorsbank.de/hbci`, BLZ 76030080, SecurePlus decoupled TAN confirmed working via python-fints PR #218. Two open python-fints PRs needed for Consorsbank (#218: login SCA attached to HKIDN + decoupled 0030/3955 approval; #210: 3 protocol fixes) — both vendorable.

Registration: FinTS product ID mandatory since Aug 2019, bring-your-own for all libraries, free, email form to Deutsche Kreditwirtschaft, 10–15 business days turnaround — register early (spawned [Register FinTS product ID](14-fints-product-id-registration.md)).

Implication for [Backend stack decision](06-backend-stack.md): pure-Rust backend is off the table today; plan a Python (or Node) sidecar behind a narrow IPC contract so a matured Rust crate can replace it later.
