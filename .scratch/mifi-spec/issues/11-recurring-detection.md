# Recurring/contract detection

Type: research
Status: open
Blocked by: 03, 07

## Question

How to detect subscriptions/contracts/standing orders from transaction streams: interval+amount clustering heuristics, merchant normalization, and whether salvaged Finanzguru contract data can bootstrap it. Cover known algorithms/prior art (e.g. what Finanzguru/Subscript-style detectors do) and define the detection rules for the spec. Output: markdown summary with the chosen approach.

Facts from [Finanzguru export salvage](../assets/03-finanzguru-export.md): 49 labeled contracts with turnus available as bootstrap + ground truth; real data contains **zweiwoechentlich** (biweekly, HelloFresh) — mifi's interval set (weekly/monthly/quarterly/yearly, CONTEXT.md) must add biweekly or fold it into weekly-with-tolerance; card-row merchant names carry mid-word spaces (`HelloFre sh`) — normalization must handle; same merchant splits across multiple Finanzguru contract IDs — decide merge policy.
