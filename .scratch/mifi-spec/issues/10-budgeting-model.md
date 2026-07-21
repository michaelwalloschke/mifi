# Budgeting model

Type: grilling
Status: closed
Assignee: michael
Blocked by: 09

## Question

Which budgeting mechanic: per-category monthly targets, envelope/zero-based (YNAB-style), or forecast-oriented (projected balance)? Plus: budget period handling (calendar month vs salary-to-salary), rollover of unspent amounts, and what "over budget" surfaces in the UI.

## Resolution

Grilled 2026-07-21. Envelope/zero-based rejected (ceremony fails for a single user who doesn't allocate daily; contradicts minimal Budget entity). Forecast rejected as a *mechanic* — it is a later display over Contracts, requiring no Budget change.

1. **Mechanic**: per-category monthly targets.
2. **Period**: calendar month — one "this month" definition shared with Sankey/trend/tiles. Salary-date shifting is a display tweak later, not a period model.
3. **Rollover**: none, either direction. Rollover is envelope semantics smuggled back in (accumulating-slack absurdity, sweep rules); overspend carry-forward punishes without actioning. Multi-month slack is visible in the monthly trend.
4. **Scope**: expense-kind Categories only (income = forecast territory; Contracts track salary). Target attachable at parent or subcategory, independently: parent counts rolled-up subcategory spending, subcategory counts only itself, both may coexist, no sum-consistency constraint — independent alarms, not an allocation tree.
5. **Spent = net**: |sum of the Category's Splits in the month|; refunds into the category reduce spent; Transfer legs excluded by definition. Cross-month refund timing noise accepted.
6. **States**: on-track < 80 %, warning ≥ 80 %, over ≥ 100 % (shows € overshoot, not just red). Deliberately **not** pace-adjusted — lumpy spending (annual insurance, bulk grocery runs) makes pace alarms cry wolf. Surfaces: budget screen primary (per-category progress bars); main screen gets one aggregate signal only (e.g. "2 over budget" — placement decided in [UI information architecture](18-ui-information-architecture.md)); no OS notifications (data only moves on sync).
7. **Targets effective-dated**: append-only rows (Category, Amount, effective-from-month). Past months evaluate against the target then in force — history stays honest. Editing writes a new row effective the current month; null Amount ends the budget.
8. **Unbudgeted spending**: one aggregate line at the bottom of the budget screen — sum of the month's expense Splits in target-less Categories (rolled up to parents), number only, no state, no prompts to create targets.
