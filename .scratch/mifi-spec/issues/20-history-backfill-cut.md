# History backfill cut

Type: grilling
Status: open

## Question

Where does history start per account? Salvage gives Giro from 2022-05, Tagesgeld from 2022-06 (rows end 2025-09 — gap to now closes via FinTS), PayPal from 2023-02; Scalable has zero history until CSV export / scalable-cli reach back. Decide: is the salvaged depth enough for v1 (flows, budgets, net worth), or does older bank-CSV archaeology (pre-2022 Consorsbank, pre-2023 PayPal) join the seed import? Also: does Scalable's short history need any mitigation (e.g. manual opening balances) for net-worth continuity?
