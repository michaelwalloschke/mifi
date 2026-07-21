# Flow visualization prototype

Type: prototype
Status: closed
Assignee: michael
Closed: 2026-07-21

## Question

What should "visualize the flows beautifully" concretely look like? Build a throwaway HTML prototype with dummy data — Sankey income→categories→savings flow, monthly trend, category drilldown — and iterate with Michael until the direction is locked. Use /prototype and the dataviz skill. Answer records the chosen viz direction and links the prototype.

## Resolution

**Direction locked: Hybrid "D" — Kennzahlen-Kacheln + Sankey-Hero + klickbarer Monatsverlauf.**

Main screen layout (top to bottom):
1. **Stat tiles** — Einnahmen, Ausgaben (delta vs. Vormonat), Sparquote (delta), Puffer übrig; each with 12-month sparkline. Tiles follow the selected month.
2. **Sankey hero** — Einnahmen → Kategorien (+ Sparen + Puffer) → Sparziele, for the selected month; month picker top-right; hover tooltips on every ribbon.
3. **Monatsverlauf** — expenses stacked by category + income line; clicking a month switches tiles *and* Sankey. Table fallback under the chart.

Judged against three structurally different variants: A (Sankey-only hero) — too little context; B (dashboard, Sankey demoted to a card) — right tiles, wrong hero; C (drilldown ledger, no Sankey) — rejected as main screen, but its master–detail category drilldown (subcategories + erkannte Verträge) stays a strong candidate for a dedicated category screen once UI IA is charted.

Implementation notes that carry into the spec:
- Sankey is hand-rolled SVG (~60 lines, 3 columns, bezier ribbons) — no d3 dependency needed; keeps the Tauri bundle lean.
- Dataviz method followed: categorical palette validated light+dark (all checks pass; light-mode contrast WARN relieved via direct labels + table view); marks/spacers/tooltips per dataviz skill.

Assets: [prototype (variants D/A/B/C, `?variant=`)](../assets/04-flow-viz-prototype.html) · screenshots [D](../assets/04-shot-D.png), [A](../assets/04-shot-A.png), [B](../assets/04-shot-B.png), [C](../assets/04-shot-C.png).
