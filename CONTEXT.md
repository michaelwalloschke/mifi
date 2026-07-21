# mifi

Local-first, single-user desktop app tracking Michael's German bank accounts: money flows, net worth (incl. depot), budgeting, recurring contracts.

## Language

**Account**:
One tracked money container at an institution, bound to a Source. Exactly five: Consorsbank Giro (checking), Consorsbank Tagesgeld (savings), Scalable depot, Scalable Verrechnungskonto (cash), PayPal (e-money).
_Avoid_: wallet, bank connection (a Source, not an Account)

**Transaction**:
A booked money movement on one Account, imported from a Source. Identity = surrogate key; duplicates across Sources/re-imports are collapsed by Import Hash. Pending/unbooked entries are not Transactions.
_Avoid_: booking, entry, payment (overloaded)

**Import Hash**:
Per-account dedup fingerprint: hash(booking date, amount, normalized counterparty, normalized purpose) plus an occurrence index for same-day identical movements. A re-import matching an existing hash+occurrence is skipped.

**Source**:
Where a Transaction came from: fints, scalable-cli, csv-paypal, csv-scalable, or finanzguru-seed. On overlap the bank Source wins; the finanzguru-seed row survives only as a categorization hint.

**Transfer**:
A link joining exactly two Transactions with opposite signs and equal Amounts on two different own Accounts — an internal move. Transfer legs never count as income or expense and carry no Category; they still move account balances. Detected automatically (amount + sign + ±4-day window + own-account counterparty), user-confirmable, always manually linkable/unlinkable. A lone leg is an ordinary expense until its counterpart arrives.
_Avoid_: internal booking, Umbuchung (use Transfer)

**Split**:
A portion of one Transaction with its own Amount and Category; a Transaction's Splits sum exactly to its Amount. Flat — no nesting, never across Accounts. Manual-only; auto-categorization works whole-transaction. Reports and Budgets consume Splits, an unsplit Transaction acting as one implicit Split. Each Split records who categorized it — auto or user; user-set Categories are never auto-overwritten and serve as learning signal.
_Avoid_: sub-transaction, line item

**Category**:
A label classifying spending or income, in a two-level tree: parent Category with optional subcategories, never deeper. Every parent carries a kind — income or expense (sign alone can't tell a refund from income). Splits assign to any Category; reports roll subcategories up. Transfer legs are uncategorized by definition.
_Avoid_: tag, label, Hauptkategorie/Unterkategorie (use parent/subcategory)

**Contract**:
A recognized recurring money flow with an external counterparty, either direction — Netflix and salary both. Carries normalized counterparty, expected Amount with tolerance, interval (weekly/monthly/quarterly/yearly), Category, and lifecycle: detected → confirmed | dismissed, plus ended. Transactions link to at most one Contract; amount history derives from linked Transactions. Recurring internal moves (ETF sparplan, Giro→Tagesgeld) are Transfer patterns, never Contracts.
_Avoid_: subscription, standing order, Vertrag (all are Contracts)

**Budget**:
A monthly target Amount on one expense-kind Category, at parent or subcategory level independently (parent counts rolled-up spending, sub counts itself; no sum constraint). Calendar-month period, no rollover in either direction. Spent = net sum of the month's Splits in the Category; refunds reduce it. Targets are effective-dated append-only rows — past months judge against the target then in force; a null Amount ends the budget. States: on-track < 80 %, warning ≥ 80 %, over ≥ 100 %; never pace-adjusted.
_Avoid_: envelope, allocation, rollover

**Snapshot**:
An append-only, per-sync-day observation: an Account balance, a depot position (ISIN, quantity, valuation), or a price. Ground truth for all valuation over time; same-day re-sync upserts.
_Avoid_: history table, cache

**Position**:
The current holding of one ISIN in the depot — the latest position Snapshot. Not a separate entity.

**Net Worth**:
Derived, never stored: sum of latest balance Snapshots plus depot Position valuations at a date.
_Avoid_: NetWorthSnapshot (no such entity)

**Amount**:
A quantity of money in EUR, stored as integer cents. mifi is single-currency; foreign-currency originals (PayPal FX) are inert display metadata, never computed with.
_Avoid_: multi-currency, FX conversion
