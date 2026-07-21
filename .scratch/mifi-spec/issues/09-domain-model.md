# Domain model

Type: grilling
Status: closed
Assignee: michael
Blocked by: 01

## Question

Pin the ubiquitous language and core model via /domain-modeling: Account, Transaction, Category, Transfer (internal moves must not count as income/expense!), Contract/Recurrence, Budget, Position, NetWorthSnapshot. Resolve: category hierarchy depth (provisional — may be refined when [Categorization approach](07-categorization-approach.md) closes), split transactions yes/no, how transfers between own accounts are detected/linked, multi-currency (EUR-only?). Output: the domain model that the spec's data schema derives from.

## Resolution

Ubiquitous language captured in [/CONTEXT.md](../../../CONTEXT.md) (created this session). Decisions:

1. **EUR-only.** Amounts are integer EUR cents; PayPal FX originals stored as inert display metadata. No FX logic anywhere.
2. **Transaction identity = Import Hash.** Surrogate PK + per-account hash(booking date, amount, normalized counterparty, normalized purpose) + occurrence index for same-day identicals (Firefly III/Actual pattern). Booked transactions only, pending ignored. `source` provenance: fints | scalable-cli | csv-paypal | csv-scalable | finanzguru-seed. On overlap bank wins; Finanzguru row survives as categorization hint only.
3. **Transfer = first-class link** joining exactly two Transactions (opposite sign, equal amount, different own accounts). Auto-detect: amount match + ±4-day window + own-account counterparty; confident → auto-link, ambiguous → one-click confirm; manual link/unlink always. Legs excluded from income/expense/flows/budgets, uncategorized, still move balances. Lone leg counts as expense until counterpart imports — detection re-runs and auto-heals.
4. **Splits: yes, flat.** 2+ per Transaction, own Amount+Category, sum exactly, no nesting, never cross-account. Manual-only; auto-categorization is whole-transaction. Reporting consumes Splits (unsplit = one implicit Split — single code path).
5. **Categories: depth-2 hard cap**, parent + optional subcategory, each parent kind-typed income|expense (sign can't distinguish refund from income). Maps 1:1 to Finanzguru's Haupt-/Unterkategorie. Taxonomy itself stays provisional (salvage + categorization tickets).
6. **Contract covers both directions** (Netflix and salary). Shape: normalized counterparty, expected amount + tolerance, interval, category, lifecycle detected → confirmed | dismissed, + ended. Transactions link to ≤1 Contract; amount history derived from linked transactions. Recurring internal moves (sparplan) are Transfer patterns, never Contracts. Detection algorithm stays with [Recurring/contract detection](11-recurring-detection.md).
7. **Budget provisional minimum:** target Amount per Category per period; rollover/period/envelope semantics deferred to [Budgeting model](10-budgeting-model.md) (now unblocked).
8. **Net worth is derived, never stored.** "NetWorthSnapshot" entity dropped. Snapshot = append-only per-sync observation (balance | position | price, same-day upsert) per [Depot positions](08-depot-networth-data.md); Position = latest position Snapshot; net worth = sum at date.
9. **Categorization provenance:** each Split carries category_source auto|user; user-set is never auto-overwritten and feeds learning ([Categorization approach](07-categorization-approach.md)).
