# Recurring/contract detection — research summary

Researched 2026-07-22 against source code and first-party docs. Decisions below are bound by mifi constraints: Rust core, SQLite, fully local, detection re-runs once per Sync Run over the full table (~5k rows), transfer legs excluded from input, Contract model per CONTEXT.md.

## Prior art

### Actual Budget — `findSchedules()` (closest prior art)

Read in full: [`packages/loot-core/src/server/schedules/find-schedules.ts`](https://github.com/actualbudget/actual/blob/master/packages/loot-core/src/server/schedules/find-schedules.ts). The algorithm is template-driven, not clustering-driven: it enumerates candidate schedule patterns — weekly, **every-2-weeks**, monthly on day X (day ≤ 28 only), monthly-last-day, monthly 1st/3rd weekday, monthly 2nd/4th weekday — over a sliding range of trial start dates, materializes the next 3 expected occurrence dates per candidate, and looks for one transaction per expected date within **±2 days** (`getTransactions` filters `date >= expected-2 && date <= expected+2`). Amount must match within a threshold of **7.5 % of the amount** (`getApproxNumberThreshold = Math.round(Math.abs(number) * 0.075)`, [`shared/rules.ts`](https://github.com/actualbudget/actual/blob/master/packages/loot-core/src/shared/rules.ts)); payee must match exactly. A candidate only survives if **all 3 occurrences** matched (`found.indexOf(null) !== -1 → skip`). Candidates are ranked by date closeness (`rank = Σ 1/(dayDiff+1)`), grouped by payee, and the best-ranked schedule per payee wins; the start date is then walked backward month-by-month while matching transactions keep appearing. Transfers are explicitly excluded from the input (`'payee.transfer_acct': null`, with the comment "Don't match transfers") — same rule mifi already has.

Takeaways adopted: exclude transfers; require 3 occurrences; per-interval date windows; percentage amount tolerance with exact/approx distinction; biweekly as a first-class pattern (Actual scans it separately from weekly because the trial-start ranges differ). Takeaway rejected: the trial-start-date enumeration — it exists because Actual matches against forward-generated rschedule occurrences; grouping by merchant first and classifying observed gaps is simpler and equivalent at mifi's scale. Notably Actual detects **no quarterly or yearly** patterns at all; mifi needs both (ground truth has jaehrlich and vierteljaehrlich contracts).

### Plaid — recurring transaction streams (the reference data model)

From Plaid's first-party OpenAPI spec ([`plaid/plaid-openapi`, schemas `TransactionStream`, `RecurringTransactionFrequency`, `TransactionStreamStatus`](https://github.com/plaid/plaid-openapi/blob/master/2020-09-14.yml)):

- Frequency enum: `WEEKLY`, `BIWEEKLY` ("approximately every 2 weeks"), `SEMI_MONTHLY` ("approximately twice per month … typically seen for inflow transaction streams"), `MONTHLY`, `ANNUALLY`, `UNKNOWN`. **No quarterly.** All definitions say "approximately" — tolerance is built into the frequency concept.
- A stream is "a grouping of related transactions" per **account**, carrying `merchant_name`, `description`, `average_amount`, `last_amount`, `first_date`, `last_date`, `predicted_next_date`, `is_active`, `status`.
- Maturity: "A `MATURE` recurring stream should have at least **3 transactions** and happen on a regular cadence (For Annual recurring stream, we will mark it `MATURE` after **2 instances**)." Before that a stream is `EARLY_DETECTION`; it becomes `TOMBSTONED` "when no further transactions were found at the next expected date."
- Plaid's [Transactions API docs](https://plaid.com/docs/api/products/transactions/) recommend "at least 180 days of history for optimal results" for recurring detection — mifi's 4.2-year seed is far past that.

Takeaways adopted: 3-occurrence minimum with a 2-instance exception for yearly; both `average_amount` and `last_amount` tracked (rolling last amount matters for price hikes); a tombstone-like "ended after missed expected dates" rule; `predicted_next_date` as a first-class field.

### Maybe Finance — nothing to reuse

Read [`app/models/family/auto_merchant_detector.rb`](https://github.com/maybe-finance/maybe/blob/main/app/models/family/auto_merchant_detector.rb): merchant detection is **delegated to an OpenAI LLM call** (`llm_provider.auto_detect_merchants`), which returns business name + URL per transaction — no normalization heuristics to borrow, and it conflicts with mifi's no-cloud rule anyway. Recurring transactions never shipped: they appear only on the [roadmap wiki](https://github.com/maybe-finance/maybe/wiki/Roadmap) and in feature requests ([#2091](https://github.com/maybe-finance/maybe/issues/2091)); the repo was archived in 2025 with no detection code in the tree (verified via full tree listing — only merchant/rule files exist).

### Firefly III — declares, does not detect

Firefly's recurring transactions are **user-created**, then a job generates future transactions from them: [`app/Factory/RecurrenceFactory.php`](https://github.com/firefly-iii/firefly-iii/blob/main/app/Factory/RecurrenceFactory.php), [`app/Jobs/CreateRecurringTransactions.php`](https://github.com/firefly-iii/firefly-iii/blob/main/app/Jobs/CreateRecurringTransactions.php), plus CRUD controllers under `app/Http/Controllers/Recurring/`. There is no discovery-from-history. The adjacent Bills/subscriptions feature is the one useful data point: a [`Bill`](https://github.com/firefly-iii/firefly-iii/blob/main/app/Models/Bill.php) carries `amount_min`, `amount_max`, expected `date`, `repeat_freq`, `skip`, and an `automatch` flag — i.e. Firefly models expected recurring amounts as a **[min, max] band around a date**, and auto-links incoming transactions into that band. mifi's "expected Amount with tolerance" is the same shape.

### Tink / GoCardless — nothing public

Tink's public docs expose transaction enrichment but no documented recurring-stream detection model (recurring *payments* are an enterprise offering, not a detection spec) — nothing citable at [docs.tink.com](https://docs.tink.com/). GoCardless Bank Account Data likewise ships raw transactions. Skipped per the no-speculation rule; Plaid remains the only aggregator with a public first-party stream model. Finanzguru publishes no algorithm — its export (asset 03) is used as ground truth only.

### Academic period detection — noted, deliberately not used

The standard signal-processing approach is periodogram + autocorrelation (AUTOPERIOD, [Vlachos/Yu/Castelli, SDM 2005](https://epubs.siam.org/doi/10.1137/1.9781611972757.40), [PDF](http://alumni.cs.ucr.edu/~mvlachos/pubs/sdm05.pdf)). It targets long event series with unknown periods. mifi merchant groups have 3–50 events and the period is one of six known values — a histogram/median of inter-arrival gaps answers the question directly. Spectral methods are the upgrade path if free-form periods ever matter; they don't for v1.

### SEPA mandate fields — semantics verified against the EPC rulebook

From the EPC SDD Core Scheme Rulebook, EPC016-06 (verified against the [v7.0 rulebook text](https://www.sepaesp.es/f/websepa/secciones/Instrumentos/EPC016-06_core_SDDRB.pdf); current version at [europeanpaymentscouncil.eu](https://www.europeanpaymentscouncil.eu/document-library/rulebooks/sepa-direct-debit-core-rulebook)):

- **AT-01, Unique Mandate Reference** (= `Mandatsreferenz`): "This reference identifies for a given Creditor, each Mandate signed by any Debtor for that Creditor. This number must be unique for each Mandate in combination with the identifier of the Creditor."
- **AT-02, Creditor Identifier** (= `Glaeubiger-ID`): "unique in the Scheme: each identifier allows the identification of one Creditor without ambiguity in SEPA. A Creditor may use more than one Identifier."
- Caveat that kills mandate-ref-as-primary-key: mandate **amendments can change both** — AT-24 (reason for amendment) explicitly enumerates "Change of AT-01 (the Creditor defining a new unique Mandate reference)", "Change of AT-02", and "Change of AT-01 and change of AT-02". So the same real-world contract can legitimately show up under a new mandate reference mid-life.

Conclusion: `(Glaeubiger-ID, Mandatsreferenz)` identifies one mandate and is the strongest per-transaction signal that two direct debits belong to the same contract — but it is absent on 1840/4633 rows (card + PayPal), can change on amendment, and one creditor ID spans many customers' products. Use it as evidence and as an exact matcher, not as the grouping key.

## Chosen approach

### Input and grouping key

Detection runs once at the end of each Sync Run over all booked transactions, **excluding Transfer legs** and excluding transactions already linked to a Contract. Grouping key: **(normalized merchant, direction)** — the exact same normalized-merchant function ticket 07 defines (strip mid-word spaces, casefold, strip legal suffixes), so `HelloFre sh`/`HelloFresh` and `Cleverbr idge`/`Cleverbridge` land in one group and card rows join their SEPA siblings. Direction (sign) splits the group so a refund never pollutes an expense stream. Mandate data is not the key (per the EPC findings above) but is stored: each Contract keeps the set of `(Glaeubiger-ID, Mandatsreferenz)` pairs observed on its linked transactions.

### Merge policy: one Contract per (normalized merchant, direction, amount band)

This matches the Finanzguru ground truth semantics: Otto ×3 split by amount are genuinely three flows (installment plans) and stay three Contracts; Cleverbridge ×2 split only by spelling collapses to one after normalization; Spotify ×2 / HelloFresh ×2 merge if their amount bands overlap, else stay split. A new mandate reference whose transactions fall into an existing contract's (merchant, band) is a mandate amendment (EPC AT-24), not a new contract — the reference is added to the contract's mandate set, no second contract is created. Same merchant, disjoint amount bands → separate Contracts, always.

### Candidate generation

Per (merchant, direction) group of unlinked transactions:

1. **Amount clustering.** Sort amounts, greedy-cluster around medians. A transaction belongs to a cluster if within **max(±100 cents, ±10 %)** of the cluster median for expenses, **max(±100 cents, ±25 %)** for income — salaries vary with tax and one-off components far more than subscription prices, while the cadence stays monthly (Plaid likewise models inflow streams with the same machinery and only frequency-level looseness). The fixed 100-cent floor exists because 10 % of a 2.99 € subscription is smaller than real billing jitter; the percentage handles utilities. (Prior-art anchor: Actual uses a flat 7.5 % with no floor.)
2. **Gap classification.** Within a cluster, sort by booking date, compute consecutive gaps in days, take the **median gap**, and classify:

   | interval | nominal | median-gap range | next-date window |
   |---|---|---|---|
   | weekly | 7 | 5–9 | ±2 d |
   | biweekly | 14 | 11–17 | ±3 d |
   | monthly | 28–31 | 25–35 | ±5 d |
   | quarterly | 90–92 | 80–100 | ±7 d |
   | yearly | 365/366 | 350–380 | ±14 d |

   The monthly range absorbs both calendar month-length variation (28–31) and German Buchungstag shifts around weekends/holidays; expected-next-date computation is **day-of-month anchored and clamped** (anchor day 31 → Apr 30, Feb 28/29), which is what Actual needs two extra patterns (`monthly`, capped at day 28, plus `monthlyLastDay`) to express. Individual gaps must fall inside the interval's range, except a gap of ≈2× nominal (within 2× the range) is tolerated as **one missed cycle** and doesn't break the chain — it just doesn't count as an occurrence.
3. **Thresholds.** A cluster becomes a `detected` Contract proposal when it has **≥3 occurrences** for weekly/biweekly/monthly/quarterly, or **≥2 for yearly** (Plaid: mature at 3, annual at 2). Because a 2-instance yearly candidate rests on a single gap, it additionally requires near-exact amounts — within max(±100 cents, ±2 %) — to keep two coincidental Amazon orders a year apart from becoming a Contract. If one cluster's gaps admit two interval readings (rare), the interval covering more transactions wins; ties break on smallest total deviation from nominal (Actual's rank idea, reduced to a tie-breaker).

Proposal fields: normalized counterparty, direction, interval, expected amount = median of linked amounts, tolerance = the band used, anchor = last booking date, mandate set from linked rows, state `detected`.

### Interval set decision: add `biweekly`

`biweekly` becomes a first-class interval alongside weekly/monthly/quarterly/yearly (CONTEXT.md Contract definition to be amended). Grounds: Plaid's frequency enum has `BIWEEKLY` as its own value distinct from `WEEKLY`; Actual scans every-2-weeks as its own pattern; and the ground truth contains `zweiwoechentlich` (HelloFresh). Folding it into weekly-with-tolerance fails mechanically: a weekly window wide enough to accept 14-day gaps (±7 d) accepts everything, and the predicted next date would be a week off every cycle. Plaid's `SEMI_MONTHLY` is **not** adopted — zero instances in 4.2 years of ground truth, and German salaries are monthly; revisit only if a real stream ever misclassifies.

### Ongoing matching and re-run stability

Each detection run is two stages, and stage order is what guarantees stability:

- **Stage A — feed existing Contracts first.** Every existing Contract (`detected`, `confirmed`, `dismissed`, and `ended`) claims matching new transactions before any candidate generation: exact match on a known `(Glaeubiger-ID, Mandatsreferenz)` pair wins outright; otherwise merchant match + booking date within the interval's next-date window + amount within **max(±100 cents, ±15 %) of the last linked amount** (rolling, so gradual price steps track without user action; a hard jump >15 % deliberately falls through and surfaces as a new `detected` candidate at the new price, which the user can merge). A successful match links the transaction and advances the expected next date.
- **Stage B — candidate generation** runs only over transactions no contract claimed. Consequences: an existing contract can never be re-proposed as a duplicate; **dismissed contracts keep silently absorbing their matching transactions** (rows stay linked, nothing is surfaced), which is the mechanism that keeps dismissed dismissed across re-runs; unconfirmed `detected` proposals are recomputed, but before inserting, a proposal matching an existing `detected` contract on (merchant, direction, interval, overlapping band) updates that row in place instead of inserting — stable IDs, no flapping.

### Lifecycle: ended

A cycle is missed when the next-date window closes with no linked transaction. After **2 consecutive missed cycles** a Contract is auto-marked `ended` (mirror of Plaid's tombstone rule "no further transactions … at the next expected date", made one cycle more forgiving because German direct debits pause for card replacements and Urlaub). Yearly contracts end after 1 missed cycle plus a 90-day grace. `ended` is reversible: a stage-A match on an ended contract clears the flag and relinks — a resumed HelloFresh is the same Contract, not a new one.

### Seeding from the Finanzguru salvage

The 49 labeled contracts (`Analyse-Vertrags-ID`, asset 03) import as **`confirmed`** Contracts — the user already curated them in Finanzguru, re-asking is noise. Turnus maps directly: monatlich→monthly, vierteljaehrlich→quarterly, jaehrlich→yearly, zweiwoechentlich→biweekly. Linked transactions come from the export's contract-ID column; expected amount = median of linked amounts; mandate sets harvested from linked rows. The merge policy applies at import: spelling-splits (Cleverbridge ×2) collapse after normalization, amount-splits (Otto ×3) stay separate — so the seeded count is ≤49. The seed doubles as the acceptance test: running the detector from scratch over the seeded history (links removed) must re-propose the labeled contracts that have ≥3 occurrences (≥2 for yearly); measured recall on that set is the tuning target for the tolerances above before any constant gets changed.

## Open questions

- Merge UX for price hikes: a >15 % jump produces a second `detected` contract by design; the confirm screen needs a "merge into existing contract" action (UI ticket 18 territory, not a detection question).
- `SEMI_MONTHLY` and free-form intervals: excluded with evidence; the noted upgrade path (per-merchant periodogram) exists if post-v1 data ever demands it.
