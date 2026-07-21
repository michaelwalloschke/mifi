# Aggregator Selection for Scalable Capital & PayPal

Research date: 2026-07-21. Method: official docs, coverage/pricing pages, developer-portal signup flows, GitHub repos; community forums used only where marked. Claims marked *(unverified)* could not be confirmed against a primary source.

Context: mifi is a local-first, single-user, privacy-first Tauri desktop app (Germany). Consorsbank is handled via FinTS (asset 02). Two accounts remain: **Scalable Capital** (ETF depot + Verrechnungskonto) and a **personal PayPal account** (balance + transactions incl. P2P). Question: aggregator, direct API, or file import per institution.

**Structural change that reframes the whole question**: Scalable Capital received a **full banking licence from the ECB in September 2025** and migrated all depots and Verrechnungskonten off Baader Bank onto its own "Scalable Capital Bank" infrastructure (own IBANs) in **Q4 2025** (migration weekend 8–9 Nov 2025) (https://www.ftd.de/boerse/aktien/vollbank-lizenz-scalable-capital/, https://stadt-bremerhaven.de/scalable-capital-gibt-zeitplan-fuer-die-depot-uebertragungen-bekannt/ — press coverage; Scalable has no single official summary page *(exact ECB licence date verified only via press)*). Consequence: as of 2026, **Baader Bank coverage in any aggregator is irrelevant** for mifi — the accounts no longer live there.

A second structural point: **PSD2 AIS covers payment accounts only.** The Verrechnungskonto (now a deposit account at Scalable Capital Bank with its own IBAN) is a payment account; **the depot is not**. Any pure-PSD2 aggregator can at best deliver the cash account. Depot positions come only from (a) non-PSD2 proprietary connectors (scraping — see finAPI below), (b) Scalable's own tooling, or (c) file export.

---

## 1. GoCardless Bank Account Data (ex-Nordigen) — NOT AVAILABLE

- **Closed to new signups.** https://bankaccountdata.gocardless.com/new-signups-disabled states verbatim: "New signups for Bank Account Data are currently disabled." (confirmed 2026-07-21; the old coverage URL now redirects to a Google-Sheets export list). Third-party trackers describe the product as being wound down for new users (https://www.openbankingtracker.com/guides/free-open-banking-apis — secondary).
- The famous free tier (the one Actual Budget / Firefly III importers were built on) is therefore unreachable for a new project. **Dead end regardless of coverage.**

## 2. Enable Banking

- **Model**: Finnish licensed AISP (FIN-FSA); single API over 2,500+ ASPSPs in ~30 countries (https://enablebanking.com/).
- **Privacy — best in class**: FAQ states Enable Banking "does not store, cache, or process data for any purpose other than delivering it to the application authorised by the user" — pass-through, no server-side transaction store (https://enablebanking.com/docs/faq/).
- **Personal/developer access — genuinely open**: you can create an account and use **both sandbox and production before signing any contract**; production apps can be activated in **"Restricted Mode (Account Linking)"** — you whitelist your **own** bank accounts and data retrieval is limited to them (https://enablebanking.com/docs/api/control-panel/, https://enablebanking.com/docs/faq/). Only making an app available to the public requires a signed contract + KYB. For a single-user app syncing the developer's own accounts this reads as a legitimate free production path *(no explicit statement that restricted mode is free forever; no price is attached to it in the docs)*.
- **Pricing (contracted)**: volume-based, minimum monthly invoice, sales contact required (https://enablebanking.com/docs/faq/).
- **Coverage — the problem**: the ASPSP list is only available via authenticated `GET /aspsps?country=DE` (https://enablebanking.com/docs/api/reference/); there is no public searchable list. No public evidence that **Scalable Capital Bank** (a bank only since late 2025) or **PayPal** is connected; the July-2025 changelog shows only established German banks (apoBank, Comdirect, Commerzbank, Santander) (https://enablebanking.com/blog/2025/08/22/changelog-july-2025). **Scalable and PayPal coverage: unverified, likely absent** *(unverified — check GET /aspsps after free signup, ~15 min)*.
- **Ergonomics**: JWT-signed API auth, user-consent redirect per session; PSD2 consent validity is now 180 days (EBA RTS amendment 2022/2360 raised the 90-day reconfirmation to 180 days).

## 3. finAPI (Schufa group)

- **Coverage — the only aggregator positively confirmed for Scalable**: finAPI partner integrations advertise connecting "Scalable Capital Bank" via "PSD2-Schnittstelle und finAPI" (https://www.finban.io/integrationen/scalable-capital-bank). Rentablo (a finAPI consumer) confirms in its forum that the Scalable import delivers **both depot positions and cash account**, i.e. finAPI reaches beyond PSD2 with a proprietary connector — with real-world reliability problems: Scalable has **blocked finAPI's IPs for excessive requests** and "refuses to adjust the firewall"; dividends sometimes arrive miscategorised (https://forum.rentablo.de/t/import-scalable-capital-depots-baader-bank/926 — community but first-hand from a finAPI customer). So: works, including depot, but brittle and adversarial to Scalable.
- **PayPal: covered.** finAPI clients document connecting PayPal accounts (private and business) with daily transaction/balance updates (https://wissen.buchhaltungsbutler.de/hc/de/articles/14437779384989, https://www.finapi.io/en/connecting-accounts-with-erp-software/).
- **Access/pricing — kills it for mifi**: B2B, sales-led, no self-serve production. 30-day free test system only (https://www.finapi.io/en/free-trial/). Price list: Access B2C basic licence €60/month (up to 200 users), Access B2X €100/month; there is even an explicit **"Access for own use (max. 10 own accounts)"** tier — "No additional PSD2 licence is required for finAPI Access for own users" — listed around **€200/month** (https://www.finapi.io/en/prices/) *(own-use price read from the price table; confirm exact figure before ever considering it)*. €720–2,400/year for one user is absurd for mifi.
- **Privacy**: server-side aggregation platform; upon contract, operations run on AWS Frankfurt (https://www.finapi.io/wp-content/uploads/2025/03/20241002_SaaS_Vertrag_online-version.pdf). finAPI is a data processor holding account data server-side, and is a **Schufa subsidiary** (https://www.finapi.io/en/home/) — maximal tension with mifi's privacy-first stance.

## 4. Tink

- **Access**: free sandbox via console.tink.com; pre-production apps allow testing with your own real credentials; **production is enterprise-gated** — contract required (https://docs.tink.com/entries/articles/set-up-your-tink-account). No published pricing; sales-led. Not reachable for a private individual in production.
- **Coverage**: claims 95%+ of German banks (https://tink.com/). No primary evidence for Scalable Capital Bank. Note the trap: Tink's PayPal relationship is PayPal *using Tink* to link bank accounts to PayPal wallets (https://tink.com/press/tink-paypal-investment/) — it is **not** evidence that Tink serves PayPal account data to third parties.
- **Verdict**: enterprise product; dead end at single-user scale.

## 5. Klarna (ex-Kosma)

- The Kosma brand was dissolved in Aug 2023 and folded into the main Klarna brand; the open-banking product itself continued (https://www.finextra.com/newsarticle/42716/klarna-ditches-open-banking-brand, https://tech.eu/2023/07/31/klarna-scraps-open-banking-brand-klarna-kosma/). Docs are still live at https://docs.openbanking.klarna.com/.
- Access is sales-gated ("ask your Klarna contact"); no self-serve signup, no public pricing *(go-live page returned 404 during research; sales-gating inferred from docs navigation and third-party trackers — unverified in detail)*. **Dead end at single-user scale**; no evidence of Scalable/PayPal coverage either.

## 6. Salt Edge

- **PayPal: covered** — Salt Edge's coverage catalogue lists PayPal as a provider connected **via API** with AIS (accounts, balances, transactions), categorised as a non-regulated/e-wallet provider (https://www.saltedge.com/products/account_information/coverage — catalogue; PayPal entry surfaced via site search). One of only two candidates positively covering PayPal.
- **Scalable Capital**: no public evidence in the DE coverage catalogue *(unverified — the DE list is paginated 60+ pages; not exhaustively checked)*. As a pure-PSD2 connection it would at best be the cash account, never the depot.
- **Personal/developer access**: Dashboard signup is free. Statuses: **Pending** (fake providers only, 10 connections) → **Test** (**live providers, up to 100 connections**, requires completed app info + **company details**) → **Live** (compliance review, contract) (https://docs.saltedge.com/general/v5/). The 100-connection Test status is technically enough for a single user forever (Firefly III community has historically used exactly this), but the upgrade form expects company details and the ToS are business-framed — a private individual is in a grey zone *(whether Salt Edge tolerates indefinite personal Test-status use: unverified)*.
- **Privacy**: server-side aggregation platform storing connection data (data processor); consent/retention managed platform-side (https://docs.saltedge.com/account_information/v5/). Weaker than Enable Banking's pass-through, better than nothing.

## 7. PayPal direct (own APIs and export)

- **Transaction Search / Reporting API**: lists transactions of the **previous three years**, max **31-day window per request**, up to 3h reporting lag (https://developer.paypal.com/docs/api/transaction-search/v1/). The blocker: **live REST credentials require a PayPal Business account** — "You'll need a PayPal Business account to go live" (https://developer.paypal.com/api/rest/, https://www.paypal.com/us/cshelp/article/how-do-i-create-paypal-rest-api-credentials-ts1949). A personal account cannot use it. Converting the account to Business just for an API is possible but changes the account's legal character — not recommended for a private P2P account.
- **PSD2 XS2A interface**: PayPal (Europe) exposes a dedicated TPP interface, but only to **licensed TPPs** with a verified eIDAS certificate (https://developer.paypal.com/limited-release/psd2/, https://www.paypalobjects.com/devdoc/xs2a.pdf). Not usable directly by mifi; this is what Salt Edge/finAPI ride on.
- **CSV export ("Aktivitäten herunterladen") — works for personal accounts**: official help: download activity as **CSV** (also PDF/Quicken/QuickBooks/TAB), **up to 7 years back, max 12 months per file**; also a custom report from the Aktivitäten view (https://www.paypal.com/de/cshelp/article/wie-kann-ich-kontoausz%C3%BCge-und-berichte-anzeigen-und-herunterladen-help145). P2P activity is part of the account activity. Zero third parties involved.

## 8. Scalable Capital direct

- **FinTS**: not offered — Scalable support states it "currently does not offer a direct FinTS interface" (community/support quote, Dec 2025: https://www.starmoney.de/forum/viewtopic.php?t=47121, https://homebanking-hilfe.de/forum/topic.php?t=24432 — *community sources*). The old Baader FinTS route died with the migration.
- **PSD2 XS2A**: as a licensed bank Scalable must expose one, but there is **no public developer portal**; only finAPI demonstrably consumes it (cash account) *(no primary Scalable XS2A doc found)*.
- **Official CSV export**: transactions exportable as CSV (semicolon-separated) from Broker → Transaktionen, with filters; officially documented (https://help.scalable.capital/kontoverwaltung-f3197dc7/kann-ich-informationen-zu-meinen-transaktionen-exportier-4c3e0a38, product note: https://de.scalable.capital/produkt-news/transaktionen-exportieren). Portfolio Performance has a community CSV importer for this format (https://forum.portfolio-performance.info/t/csv-import-von-scalable-capital/30113).
- **Official CLI — the surprise winner**: **`scalable-cli`** (https://github.com/ScalableCapital/scalable-cli), "The official, agent-ready command line for the Scalable Broker". Rust, Apache-2.0, 302 stars, v0.5.0 released 2026-07-02, **beta**. Commands include portfolio **holdings, transactions, overview, analytics**, watchlists, quotes — all with `--json` output. Auth: **OAuth 2.0 device-code flow**; users must currently be **allowlisted** before first login (beta gate); offers a local read-only mode. This is a first-party, scriptable, JSON-emitting path to exactly the data mifi needs (depot positions + transactions), with credentials staying on the user's machine.
- **Unofficial**: `ffischbach/unofficial-scalable-capital-api` — local proxy around Scalable's **internal GraphQL/WebSocket API**, headed-browser login incl. 2FA, exposes portfolio/transactions via localhost REST/SSE (https://github.com/ffischbach/unofficial-scalable-capital-api). **Unofficial, can break at any time.** Older scraping hobby projects exist (https://github.com/Bibo-Joshi/scalable-capital-utils, https://github.com/roboes/neobroker-portfolio-importer). All clearly unofficial; only relevant as fallback if the CLI allowlist never opens.

---

## Comparison summary

| | Scalable (depot) | Scalable (cash) | PayPal (personal) | Individual signup | Cost @ 1 user | Data stored server-side by provider |
|---|---|---|---|---|---|---|
| GoCardless BAD | — | — | — | **closed to new signups** | — | — |
| Enable Banking | no (PSD2-only) | unverified | unverified/likely no | yes (restricted-mode prod, own accounts) | free (pre-contract) | **no (pass-through)** |
| finAPI | **yes** (proprietary connector, brittle) | yes | **yes** | no (B2B; "own use" tier ≈ €200/mo) | €60–200+/mo | yes (AWS Frankfurt; Schufa subsidiary) |
| Tink | unverified | unverified | no (PayPal is Tink's customer, not a source) | sandbox only; prod = enterprise | n/a | yes |
| Klarna | unverified | unverified | unverified | no (sales-gated) | n/a | yes |
| Salt Edge | no (PSD2-only) | unverified | **yes** | grey zone (Test status, 100 conns, company details asked) | free in Test status | yes |
| PayPal direct API | | | **no — Business account required** | n/a | n/a | first-party |
| PayPal CSV export | | | **yes** (7y back, 12-mo chunks) | yes | free | first-party only |
| Scalable CLI / CSV | **yes (CLI, beta)** / partial (CSV: transactions) | yes (CLI overview) | | yes (CLI: allowlist gate) | free | first-party only |

## Recommendation for mifi

**No aggregator. Split, first-party solution per institution:**

1. **Scalable Capital → official `scalable-cli` as primary, official CSV export as baseline.**
   - Apply for the CLI beta allowlist now (the same lead-time logic as the FinTS product registration in asset 02). Once in: `sc broker transactions --json` / holdings gives depot positions, transactions and cash overview as structured JSON via OAuth device flow — local-first, first-party, no data processor in the middle. Wrap it behind the same narrow sync-module IPC boundary planned for FinTS.
   - Until/unless allowlisted: **manual CSV import** of Broker → Transaktionen (documented, stable enough that Portfolio Performance parses it). Positions can be derived from the transaction history plus market prices, which mifi needs anyway.
   - Explicitly rejected: finAPI (only aggregator that demonstrably delivers the depot, but B2B, ≥€60–200/month, Schufa-owned, server-side data storage, and Scalable actively firewalls it — the opposite of privacy-first and reliability).

2. **PayPal → CSV import ("Aktivitäten herunterladen"), no aggregator.**
   - Personal accounts get no Transaction Search API (Business only), and the two aggregators that do cover PayPal (Salt Edge, finAPI) both route the data through their servers and have business-framed signup — poor privacy trade for one wallet. The official activity export covers 7 years of history in 12-month CSV chunks including P2P, at zero privacy cost. Build one CSV mapper (PayPal's CSV format is stable and widely parsed) with dedupe on transaction ID.

3. **Watchlist** (re-check in 6–12 months):
   - Scalable CLI leaving beta / allowlist opening → promotes path 1 from CSV to fully automatic.
   - Enable Banking `GET /aspsps?country=DE` after a free signup: if Scalable Capital Bank appears, its restricted-mode production (own-accounts whitelist, pass-through, no storage) would be an acceptable *privacy-compatible* automation path for the **cash account only** — still never the depot.
   - Any sign of Scalable opening its XS2A/Broker API publicly or shipping FinTS.

This honestly wins on privacy (no data processor ever sees the data), feasibility (both paths work today for a private individual at €0), and maintenance (two file importers + one optional CLI wrapper vs. an aggregator contract).

## Sources

- https://bankaccountdata.gocardless.com/new-signups-disabled
- https://enablebanking.com/docs/faq/ · https://enablebanking.com/docs/api/control-panel/ · https://enablebanking.com/docs/api/reference/ · https://enablebanking.com/blog/2025/08/22/changelog-july-2025
- https://www.finapi.io/en/prices/ · https://www.finapi.io/en/free-trial/ · https://www.finapi.io/wp-content/uploads/2025/03/20241002_SaaS_Vertrag_online-version.pdf · https://www.finban.io/integrationen/scalable-capital-bank · https://forum.rentablo.de/t/import-scalable-capital-depots-baader-bank/926
- https://docs.tink.com/entries/articles/set-up-your-tink-account · https://tink.com/press/tink-paypal-investment/
- https://www.finextra.com/newsarticle/42716/klarna-ditches-open-banking-brand · https://docs.openbanking.klarna.com/
- https://docs.saltedge.com/general/v5/ · https://www.saltedge.com/products/account_information/coverage
- https://developer.paypal.com/docs/api/transaction-search/v1/ · https://developer.paypal.com/api/rest/ · https://developer.paypal.com/limited-release/psd2/ · https://www.paypal.com/de/cshelp/article/wie-kann-ich-kontoausz%C3%BCge-und-berichte-anzeigen-und-herunterladen-help145
- https://github.com/ScalableCapital/scalable-cli · https://help.scalable.capital/kontoverwaltung-f3197dc7/kann-ich-informationen-zu-meinen-transaktionen-exportier-4c3e0a38 · https://de.scalable.capital/produkt-news/transaktionen-exportieren · https://github.com/ffischbach/unofficial-scalable-capital-api
- Scalable bank licence/migration (press): https://www.ftd.de/boerse/aktien/vollbank-lizenz-scalable-capital/ · https://stadt-bremerhaven.de/scalable-capital-gibt-zeitplan-fuer-die-depot-uebertragungen-bekannt/
- FinTS absence (community): https://www.starmoney.de/forum/viewtopic.php?t=47121 · https://homebanking-hilfe.de/forum/topic.php?t=24432
