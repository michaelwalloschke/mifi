# Account inventory

Type: grilling
Status: closed
Assignee: michael

## Question

Which concrete accounts does mifi track? For each: institution, account type (Girokonto/Tagesgeld/credit card/depot/neobank), and whether the institution is FinTS-reachable. Output: a table that fixes FinTS coverage vs aggregator-fallback scope and what net worth must include.

## Resolution

Closed set — exactly these accounts, nothing else (no credit card, cash, loans, pension):

| Institution | Account | Type | Expected data access |
|---|---|---|---|
| Consorsbank | Girokonto | checking | FinTS (verify in [FinTS library landscape](02-fints-library-landscape.md)) |
| Consorsbank | Tagesgeld | savings | FinTS |
| Scalable Capital | Depot (ETFs) | depot | no FinTS — aggregator or CSV ([Aggregator selection](05-aggregator-selection.md), [Depot positions](08-depot-networth-data.md)) |
| Scalable Capital | Verrechnungskonto | cash | same source as depot |
| PayPal | PayPal account | e-money | no FinTS — aggregator or PayPal API/CSV |

Decisions:
- **PayPal is a first-class account** (option 1): own transactions + balance; Michael receives P2P payments from friends. Giro↔PayPal funding debits are internal transfers; categorization happens on the PayPal-side transaction where the merchant is visible.
- **Scalable Verrechnungskonto is tracked**: balance counts toward net worth; Consorsbank↔Scalable transfers are internal, not expenses.
- **Net worth** = Consorsbank Giro + Tagesgeld + Scalable depot value + Verrechnungskonto + PayPal balance.

Implications for other tickets:
- Aggregator scope = Scalable Capital + PayPal only; FinTS must reach only Consorsbank.
- Domain model: internal-transfer linking needed for Giro↔PayPal and Giro↔Verrechnungskonto.
