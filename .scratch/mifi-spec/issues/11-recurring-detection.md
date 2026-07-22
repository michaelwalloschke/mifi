# Recurring/contract detection

Type: research
Status: closed
Assignee: michael
Blocked by: 03, 07

## Question

How to detect subscriptions/contracts/standing orders from transaction streams: interval+amount clustering heuristics, merchant normalization, and whether salvaged Finanzguru contract data can bootstrap it. Cover known algorithms/prior art (e.g. what Finanzguru/Subscript-style detectors do) and define the detection rules for the spec. Output: markdown summary with the chosen approach.

Facts from [Finanzguru export salvage](../assets/03-finanzguru-export.md): 49 labeled contracts with turnus available as bootstrap + ground truth; real data contains **zweiwoechentlich** (biweekly, HelloFresh) — mifi's interval set (weekly/monthly/quarterly/yearly, CONTEXT.md) must add biweekly or fold it into weekly-with-tolerance; card-row merchant names carry mid-word spaces (`HelloFre sh`) — normalization must handle; same merchant splits across multiple Finanzguru contract IDs — decide merge policy.

## Resolution (2026-07-22)

Full findings with citations: [Recurring detection research summary](../assets/11-recurring-detection.md). Prior art read at source: Actual Budget's `findSchedules()` (closest match, template-scan approach), Plaid's recurring-stream OpenAPI model (frequency enum, maturity rules), Firefly III (declares only, but Bill = amount-band model), Maybe Finance (LLM-based, nothing reusable), EPC SDD Core Rulebook (mandate field semantics).

1. **Grouping = (ticket-07 normalized merchant, direction); one Contract per (merchant, direction, amount band).** Otto's amount-splits stay 3 contracts (real installment flows), Cleverbridge's spelling-split merges after normalization, Spotify/HelloFresh merge iff bands overlap. `(Glaeubiger-ID, Mandatsreferenz)` stored per contract as an exact-match signal, never the grouping key — EPC AT-24 lets both change on mandate amendment, and 1840/4633 rows lack them.
2. **`biweekly` added as first-class interval** (CONTEXT.md amended): weekly/biweekly/monthly/quarterly/yearly. Grounds: Plaid `BIWEEKLY`, Actual's every-2-weeks pattern, HelloFresh ground truth; weekly-with-±7d-tolerance would accept everything. `SEMI_MONTHLY` rejected — zero instances in 4.2 y.
3. **Rules:** amount clusters max(±1 €, ±10 %) expense / ±25 % income; median-gap classification with per-interval ranges (monthly 25–35 d, day-of-month anchored + clamped for month-end); propose `detected` at ≥3 occurrences (yearly ≥2 + near-exact amounts); one ≈2× gap tolerated as missed cycle; next-date windows ±2/±3/±5/±7/±14 d.
4. **Re-run stability:** stage A feeds existing contracts first (mandate exact match, else merchant + window + ±15 % of last amount, rolling for price creep); stage B generates candidates only from unclaimed rows; dismissed contracts silently absorb their rows forever; `detected` proposals update in place. 2 consecutive missed cycles → `ended` (yearly: 1 + 90 d grace), reversible on new match.
5. **Seeding:** 49 Finanzguru contracts import as `confirmed`, turnus mapped 1:1, merge policy applied (count ≤49); seed doubles as recall/acceptance test for the tolerances.

Flag for [Spec assembly](13-spec-assembly.md): confirm-screen needs "merge into existing contract" action (>15 % price jump surfaces as second `detected` by design).
