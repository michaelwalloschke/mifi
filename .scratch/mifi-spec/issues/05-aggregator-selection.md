# Aggregator selection

Type: research
Status: closed
Assignee: michael
Blocked by: 01, 02

## Question

For the accounts FinTS can't reach (per Account inventory), which aggregator fits: GoCardless Bank Account Data, finAPI, Enable Banking, others? Compare: coverage of the specific institutions, personal/developer access (no company needed?), pricing at personal scale, data-privacy posture, API ergonomics. Output: markdown summary with a recommendation.

## Resolution

Full findings with citations: [Aggregator selection summary](../assets/05-aggregator-selection.md).

**Decision: no aggregator. Split first-party solution per institution.**

Two facts reframed the question:

1. **Scalable Capital is its own bank** since the ECB licence (Sept 2025); all depots/Verrechnungskonten migrated off Baader Bank in Nov 2025. Baader coverage in any aggregator is moot.
2. **GoCardless Bank Account Data is closed to new signups** — the well-known free tier is unreachable for new projects.

Candidate verdicts:

- **Enable Banking** — best privacy (pass-through, no server-side storage; free restricted-mode production on own accounts), but no public evidence it covers Scalable Capital Bank or PayPal, and as pure PSD2 it could never deliver the depot (not a payment account).
- **finAPI** — only aggregator confirmed to deliver Scalable incl. depot (proprietary connector) and PayPal, but B2B (€60–200+/month), Schufa subsidiary, stores data on AWS Frankfurt, and Scalable actively firewalls its scrapers. Fails privacy + personal-scale.
- **Tink / Klarna (ex-Kosma)** — enterprise-gated, no single-user path.
- **Salt Edge** — covers PayPal, free tier is business-framed grey zone; PSD2-only → no depot.
- **PayPal direct API** — Transaction Search requires a Business account; dead for personal. Official CSV export works: 7 years back, 12-month chunks, incl. P2P.
- **Scalable direct** — official **`scalable-cli`** (github.com/ScalableCapital/scalable-cli, Rust, Apache-2.0, beta v0.5.0 Jul 2026): holdings/transactions/overview with `--json`, OAuth device-code flow, currently allowlist-gated. Plus official transactions-CSV export from the web app.

**Chosen path:**
- **Scalable Capital**: official `scalable-cli` as primary path (apply for beta allowlist now — spawned [Apply for scalable-cli beta](15-scalable-cli-beta.md)); official CSV export as today's working baseline.
- **PayPal**: CSV import ("Aktivitäten herunterladen").

€0, works today for a private individual, zero third-party data processors — wins on privacy and feasibility, since PSD2 aggregators could never deliver the depot anyway.

Watchlist: scalable-cli leaving beta; Enable Banking's ASPSP list (free signup check) if cash-account automation is ever wanted.
