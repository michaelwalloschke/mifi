# FinTS/HBCI Client Library Landscape

Research date: 2026-07-21. Method: GitHub API (repo metadata, issues), crates.io/npm/PyPI/Maven/NuGet registry APIs, library READMEs/docs, fints.org. All maturity numbers are as-of the research date. Claims marked *(unverified)* could not be confirmed against a primary source.

Context: mifi is a local-first, single-user Tauri desktop app. Primary bank Consorsbank (Giro + Tagesgeld) via FinTS. Needed: PSD2 login SCA with decoupled TAN, HKKAZ/HKCAZ transaction fetch, ideally HKWPD. Depot at Scalable Capital is out of scope here.

---

## 1. python-fints (Python)

- Repo: https://github.com/raphaelm/python-fints — "Pure-python FinTS (formerly known as HBCI) implementation"
- **Maturity**: 413 stars, last push 2026-06-02, latest release v5.0.0 (tag dated 2026-01-06, on PyPI as `fints` 5.0.0). Maintained by Raphael Michel (author of pretix); used by community importers (e.g. Firefly III FinTS importers). Actively maintained.
- **PSD2/TAN**: Full SCA support. Docs (https://python-fints.readthedocs.io/en/latest/tans.html) document `get_tan_mechanisms()`, `set_tan_mechanism()`, TAN media selection, and a `NeedTANResponse` object carrying `challenge`, `challenge_html`, `challenge_hhduc` (chipTAN flicker), `challenge_matrix` (photoTAN image tuple), and a `decoupled` flag. Caller submits via `send_tan(challenge, tan)`; for decoupled methods the TAN parameter is empty and `send_tan()` is polled until the app approval lands. TAN state is serializable across sessions (`get_data()` / `NeedRetryResponse.from_data()`) — ideal for a desktop UI that shows a dialog and resumes. Mechanisms: pushTAN, chipTAN (flicker/optiTAN), photoTAN, decoupled (https://python-fints.readthedocs.io/en/latest/).
- **Transactions**: HKKAZ (MT940) and HKCAZ (camt) both documented on the docs front page.
- **HKWPD**: Yes — `get_holdings()` exists. But see the open Consorsbank depot issue below (#51); works generally, Consorsbank depot access failed for at least one user with 9380 "Keine Berechtigung".
- **Registration**: Bring-your-own. Quickstart requires registering a product ID as a setup step; the client constructor takes it.
- **License**: **LGPL-3.0** (GitHub license detection). Used as a Python package (dynamic import), so LGPL obligations are trivially met even in a closed app — no static-linking question arises.
- **Consorsbank track record**: Best-documented of all libraries — open PRs/issues #218, #210, #99, #51 (see Consorsbank section).

## 2. hbci4j / HBCI4Java (Java)

- Repo: https://github.com/hbci4j/hbci4java (the willuhn/hbci4java repo is archived and redirects here: https://github.com/willuhn/hbci4java)
- **Maturity**: 185 stars, last push 2026-06-26. This is the engine of **Hibiscus**, the long-running German open-source banking app — the strongest production pedigree of any library here (readme: "offizielle Quelle von HBCI4Java, welches u.a. in Hibiscus zum Einsatz kommt"). Maven Central: `com.github.hbci4j:hbci4j-core`, 3.1.x line actively released (3.1.88, April 2026); 4.0.0 (Jakarta EE) published March 2023.
- **PSD2/TAN**: Readme explicitly lists "Unterstützung für PSD2 (SCA), welche seit September 2019 für FinTS verpflichtend ist", smsTAN, photoTAN, chipTAN incl. HHD flicker, chipTAN USB, and "PushTAN Decoupled (Direktfreigabe per App)", plus Verification of Payee. TAN flow is exposed via the `HBCICallback` interface (callback-driven rather than object-returning — workable but clunkier to bridge into async UI). *(callback design from prior knowledge of the API; not re-verified in this pass)*
- **Transactions**: HKKAZ and HKCAZ ("Abruf von Umsätzen im CAMT-Format (HKCAZ)" in readme).
- **HKWPD**: Yes — `GVWPDepotList` job exists; Hibiscus has depot support built on it. *(job class name from prior knowledge; not re-verified)*
- **Registration**: Bring-your-own product ID (Hibiscus registers its own; library consumers must register theirs).
- **License**: **LGPL-2.1** (relicensed from GPLv2 on 2016-05-02, see willuhn/hbci4java#36, linked in readme). As a JVM dependency, dynamic linking — no practical constraint on a closed app shipping it as a jar.

## 3. lib-fints (TypeScript/Node) — robocode13

- Repo: https://github.com/robocode13/lib-fints — "FinTS 3.0 protocol with PIN/TAN, supporting PSD2 and decoupled TAN methods"
- **Maturity**: 20 stars, last push 2026-06-13, npm `lib-fints` 1.4.8 (published 2026-06-13). Single maintainer, CI on GitHub Actions, only runtime dep is `fast-xml-parser`. Young but genuinely active — this is the viable JS option, **not** the old `fints` package.
- **PSD2/TAN**: Every response can set `requiresTan`; caller shows `tanChallenge`, then calls the matching `...WithTan(tanReference, tan)` continuation. **Decoupled TAN explicitly supported**: omit the tan parameter and poll the `...WithTan()` method until `requiresTan=false` ("The continuation methods will keep returning requiresTan=true as long as the user hasn't approved"). TAN method/media selection via BPD (`selectTanMethod`, `selectTanMedia`). Clean challenge/response exposure for desktop UX. No explicit photoTAN-image/flicker helpers documented *(matrix-challenge rendering support unverified)*.
- **Transactions**: `getAccountStatements()` — README table lists **HKKAZ, HKCAZ (MT940 or CAMT)**.
- **HKWPD**: **Yes** — `getPortfolio(accountNumber, ...)` mapped to HKWPD in the README's transaction table, plus `canGetPortfolio()` capability check. Also DKKKU credit-card statements.
- **Registration**: Bring-your-own; README quotes the DK registration requirement and links https://www.fints.org/de/hersteller/produktregistrierung, noting registration "is currently offered free of charge".
- **License**: **LGPL-2.1-or-later** (npm metadata). Node dependency = dynamic use; no constraint in practice.

## 4. fints (TypeScript) — Prior99 — NOT viable

- Repo: https://github.com/Prior99/fints — 20 stars, last commit 2020-11-19 (a 2024 push touched only metadata; commit history effectively ends 2020). npm `fints` 0.5.0, last modified 2022-05-02.
- **PSD2/TAN**: Confirmed questionable. Open issues: #37 "TAN on login" (login SCA not handled), #13 "Unimplemented TAN method version 7 encountered", #15 stalled WIP PR "add HKTAN Segment Version 7". PSD2 work stopped at basic photoTAN (#9, #10, closed 2019-era). No decoupled TAN. Effectively pre-PSD2-complete and abandoned.
- License MIT. Listed only to close the question; do not use.

## 5. libfintx (C#/.NET)

- Repo: https://github.com/libfintx/libfintx — ".NET banking client library for HBCI 2.2, FinTS 3.0, EBICS H004 and EBICS H005"
- **Maturity**: 60 stars, last push 2026-06-09, NuGet `libfintx.FinTS` latest 1.3.0. Moderately active; README lists Consorsbank among "Tested banks".
- **PSD2/TAN**: PIN/TAN supported; README feature list covers HKSAL, HKKAZ, SEPA transfers. TAN mechanism detail (decoupled, photoTAN image exposure) is **not documented in the README** — *(decoupled TAN support unverified)*.
- **Transactions**: HKKAZ yes; HKCAZ not listed *(camt support unverified)*.
- **HKWPD**: Not in the feature list — treat as **no**.
- **License**: **LGPL-3.0**. .NET assembly = dynamic linking, no practical constraint.
- Only relevant if a .NET sidecar were on the table, which it isn't for this stack. Listed for completeness.

## 6. Rust: fints-rs (floffel/fints) — exists, but too young

- crates.io search results for fints/hbci (2026-07-21): `fints-rs` 0.1.0, `fints` 0.1.0 (2019, single stub release, dead), `fints-institute-db` 1.5.0 (bank database, **not a client**), `mt940` 1.1.0 (parser only).
- Repo: https://github.com/floffel/fints — "Native Rust FinTS 3.0 PinTan client", MIT license.
- **Maturity**: **5 stars, entire history pushed on 2026-04-06 (created and last committed the same day), one release 0.1.0, 111 downloads.** Single author, no track record, no known production use. The README is ambitious: full FinTS 3.0 PIN/TAN incl. two-step **and decoupled TAN**, typestate dialog layer, built-in bank registry, "Account balance, transaction history (MT940), and securities/depot holdings", mock server, CLI. None of this is battle-tested against real banks beyond DKB (which has a dedicated workflow module).
- **Honest verdict**: a real Rust FinTS client now exists, but at 0.1.0/one-day-old-history it is not a foundation you can bet a banking sync on today. No HKCAZ/camt mentioned *(camt support unverified; MT940 only per README)*. Worth watching, possibly worth contributing to later.
- Supporting crates that WOULD help a Rust implementation: `fints-institute-db` (svenstaro, 1.5.0, updated 2026-04, bank URL lookup by BLZ) and `mt940` (strict MT940 parser, updated 2025-10). But the protocol layer (dialog, HNSHK/HNVSK security wrapping, HKTAN state machine) would be yours to write or to harden in fints-rs.

## 7. Go: go-hbci — NOT viable

- Repo: https://github.com/mitch000001/go-hbci — 68 stars, last push 2024-12-11, Apache-2.0 per README badge (GitHub reports NOASSERTION).
- README self-assessment: "this library is only at the beginning of being useful… conforms to HBCI 2.2 and FINTS 3.0", roadmap still has unchecked read-only items. No PSD2 SCA/decoupled TAN documented. Not viable.

---

## FinTS product registration (Deutsche Kreditwirtschaft)

Source: https://www.fints.org/de/hersteller/produktregistrierung (fints.org is the DK's FinTS site; registration mailbox is registrierung@hbci-zka.de).

- **Mandatory**: yes — "Seit dem 1. August 2019 wird aufgrund regulatorischer Vorgaben nur noch registrierten Produkten der Zugang über FinTS gewährt." (Note: the page says **1 August 2019**; the commonly cited "September 2019" is the PSD2 SCA enforcement date, 14 Sept 2019.)
- **Who registers**: the *product* (your app), not the library. All libraries surveyed are bring-your-own-ID: python-fints, lib-fints and fints-rs take the product ID as a constructor/config parameter; none ship a usable built-in ID. mifi must register its own.
- **How**: completed registration form submitted by email to registrierung [at] hbci-zka [dot] de. Turnaround "in der Regel innerhalb von 10–15 Werktagen". Free of charge (fints.org per lib-fints README quote: "FinTS product registration is currently offered free of charge"; the registration page itself states no fee — *cost stated as free per secondary quote, page shows no fee schedule*).
- **Usage**: the assigned registration number goes into the product name field of every dialog initialization (HKVVB).
- Practical: register early — the 2–3 week turnaround is likely the longest lead time in the whole FinTS integration.

## Consorsbank specifics

- **FinTS URL**: `https://brokerage-hbci.consorsbank.de/hbci`, BLZ **76030080** (user-supplied in python-fints issue #51 and PR #218/#210; not verified against a Consorsbank official page — their FinTS info page requires login *(unverified against official source)*).
- **TAN**: SecurePlus app with **decoupled approval** confirmed working: python-fints PR #218 documents Consorsbank returning `0030` + `3955` ("Sicherheitsfreigabe erfolgt über andere…") for app approval, i.e. genuine decoupled flow.
- **Known quirks** (all from python-fints issues, found by diffing against working hbci4j traffic — hbci4j handles Consorsbank out of the box):
  - #218 (open PR): Consorsbank attaches the login-SCA response to **HKIDN instead of HKTAN**; stock python-fints missed it and the bank aborted with 9800/9120. Fix pending upstream.
  - #210 (open PR): three fixes needed — `security_method_version=2` in HNSHK for two-step TAN (spec-correct, python-fints hardcoded 1); KTI1 must carry full account details, not just IBAN/BIC; Consorsbank requires HKTAN even when HIPINS says HKKAZ:N (error 9075 otherwise) → needs a `force_twostep_tan` override.
  - #51 (open, 2019): depot fetch (`get_holdings`/HKWPD) against the brokerage endpoint failed with 9380 "Keine Berechtigung für diese Auftragsart mit diesem Konto" for at least one user. Consorsbank HKWPD via python-fints is **unproven**. (For mifi: depot is at Scalable anyway, so this mostly doesn't matter.)
  - libfintx lists Consorsbank as a tested bank; hbci4j has only minor Consorsbank issues (UTF-8 in account holder names, #99 hbci4j).
- Net: Consorsbank is a quirky FinTS peer. hbci4j handles it best today; python-fints handles it with the two pending PRs (#218/#210) applied — which, being a Python package, can be vendored/patched trivially.

## Sources

- https://github.com/raphaelm/python-fints · https://python-fints.readthedocs.io/en/latest/ · https://python-fints.readthedocs.io/en/latest/tans.html
- https://github.com/hbci4j/hbci4java · https://central.sonatype.com/artifact/com.github.hbci4j/hbci4j-core/versions
- https://github.com/robocode13/lib-fints · https://www.npmjs.com/package/lib-fints
- https://github.com/Prior99/fints · https://github.com/libfintx/libfintx · https://github.com/mitch000001/go-hbci
- https://github.com/floffel/fints · https://crates.io/crates/fints-rs · https://crates.io/crates/fints-institute-db · https://crates.io/crates/mt940
- https://www.fints.org/de/hersteller/produktregistrierung
- python-fints issues/PRs: #218, #210, #99, #51 (https://github.com/raphaelm/python-fints/issues)

---

## Ranking

1. **python-fints (Python sidecar)** — Best fit. Actively maintained (v5.0.0, Jan 2026; pushes through June 2026), the cleanest TAN challenge/response API of the field (`NeedTANResponse` with decoupled flag, photoTAN image bytes, flicker data, and serializable TAN state — maps 1:1 onto a Tauri dialog + resume flow), HKKAZ + HKCAZ + HKWPD, LGPL-3 with zero linking concerns from Python. Its Consorsbank gaps are precisely known (open PRs #218/#210) and trivially vendorable until merged. Cost: requires shipping a Python runtime as a Tauri sidecar.

2. **lib-fints (Node sidecar or Tauri-adjacent JS)** — Strong runner-up. The only actively maintained JS/TS option (1.4.8, June 2026), explicit PSD2 + decoupled TAN with a simple `requiresTan`/`...WithTan()` polling contract, HKKAZ/HKCAZ and even HKWPD. Riskier than python-fints only on ecosystem depth: 20 stars, one maintainer, no documented Consorsbank track record, and photoTAN/flicker rendering support is undocumented. If the frontend stack already drags in Node tooling, this narrows the sidecar cost gap considerably.

3. **hbci4j/HBCI4Java (JVM sidecar)** — Most battle-tested engine (Hibiscus), best out-of-the-box Consorsbank behavior, full TAN matrix including decoupled and VoP, HKKAZ/HKCAZ/HKWPD, LGPL-2.1. Ranked third only because a JVM sidecar is the heaviest runtime to ship with a Tauri app and the `HBCICallback` design is the most awkward to bridge into an async desktop UI. If Consorsbank protocol pain ever exceeds patching python-fints, this is the fallback that definitely works.

4. **fints-rs (pure Rust)** — The only path to a sidecar-free pure-Rust Tauri backend, and its feature list (decoupled TAN, MT940, depot) matches mifi's needs on paper. But it is a 0.1.0 crate whose entire git history is one day old (2026-04-06), 5 stars, untested against Consorsbank, MIT-licensed. Today it's a bet, not a foundation. Re-evaluate in 6–12 months, or adopt it as an upstream to harden if pure Rust becomes a hard requirement.

Not viable: Prior99 `fints` (abandoned pre-complete-PSD2), go-hbci (self-declared incomplete, no SCA), libfintx (fine but .NET buys nothing here; no HKWPD).

## Implications for backend stack

A pure-Rust Tauri backend is not realistically available today — no production-grade Rust FinTS client exists (fints-rs is 3 months old at 0.1.0) — so ticket 06 should plan for a **sidecar for the FinTS sync path**: Python (python-fints) as primary recommendation, Node (lib-fints) as the lighter-weight alternative if the toolchain already includes Node. This decision is confined to the sync module: keep the FinTS boundary a narrow IPC contract (accounts, balances, transactions, TAN challenge/response events) so a future migration to a matured Rust crate swaps the sidecar without touching the app core.
