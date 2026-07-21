# UI information architecture

Type: prototype
Status: closed
Assignee: michael
Blocked by: 10, 16

## Assets

- [Clickable prototype](../assets/18-ui-ia-prototype.html) — open in browser; `?variant=A|B|C` or ←/→ switch navigation shells (A Seitenleiste, B Kopfleiste/Tabs, C Hub & Spoke); screens via hash (`#/transaktionen` …). All 7 surfaces: Übersicht (locked 04-D), Transaktionen (Splits, Umbuchungen, Umkategorisieren), Kategorien (C-Drilldown), Budget (80 %/100 %-Schwellen), Verträge (erkannt → bestätigen/ablehnen), Vermögen (Snapshot-Kurve + Positionen), Konten & Sync (TAN-Modal, per-Konto-Status, CSV-Drop).
- Screenshots: [A Übersicht](../assets/18-A-uebersicht.png) · [A Transaktionen](../assets/18-A-transaktionen.png) · [B Budget](../assets/18-B-budget.png) · [C Übersicht](../assets/18-C-uebersicht.png) · [A Vermögen](../assets/18-A-vermoegen.png) · [A Konten + TAN](../assets/18-A-konten-tan.png)

## Question

Screen map + design language for the whole app. Main screen is locked ([Flow visualization prototype](04-flow-viz-prototype.md): stat tiles → Sankey hero → monthly trend). Domain model fixes the remaining scope: transactions list (splits, transfer links, category corrections), category screen (C-style drilldown candidate), contracts screen (detected → confirm/dismiss lifecycle), net worth/depot screen (Positions + derived net-worth curve), budget screen (mechanic from [Budgeting model](10-budgeting-model.md)), sync/TAN surface (from [Refresh/sync UX](16-sync-ux.md)). Output: clickable prototype or annotated screen map fixing navigation and design language.

## Resolution (2026-07-21, prototype reviewed with Michael)

Navigation: **Variante A — feste Seitenleiste** (Desktop-App-Muster). Linke Leiste (~216 px) mit allen Bereichen, Fehler-/Neuerkannt-Badges an den Einträgen (z. B. Verträge ②), Sync-Block unten in der Leiste (Zeitstempel + Button, immer sichtbar). B (Tabs) und C (Hub & Spoke) verworfen.

Screen-Zuschnitt bestätigt — 7 Screens:
1. **Übersicht** — locked aus [Flow visualization prototype](04-flow-viz-prototype.md) Variante D: Kacheln → Sankey-Hero → Monatsverlauf; Kacheln und Sankey-Knoten sind Navigationseinstiege (Kategorie-Klick → Kategorien-Detail).
2. **Transaktionen** — datumsgruppierte Liste, Konto-Filter + Suche; Splits als eingerückte Zeilen mit Herkunfts-Tag (auto/manuell); Transfer-Legs als „⇄ Umbuchung — nicht in Auswertungen"; Umkategorisieren inline über Kategorie-Chip.
3. **Kategorien** — C-Drilldown aus Ticket 04 (Master–Detail): Liste links, rechts Einzeltrend + Unterkategorien + Verträge der Kategorie.
4. **Budget** — Zielzeilen mit Fortschrittsbalken in Kategoriefarbe, 80 %-Marke im Balken, Zustände als Text+Icon (▲ ab 80 %, ⚠ überzogen — nie Farbe allein), aggregierte „Ohne Budget"-Zeile.
5. **Verträge** — Kennzahl-Kacheln (Fixkosten/Monat normalisiert, Vertrags-Einnahmen, Anzahl); „Neu erkannt"-Karte mit Bestätigen/Ablehnen; Aktiv-Liste mit nächster Zahlung + Monats-Normalisierung.
6. **Vermögen** — Kacheln (Netto, Depot, Cash, Δ Monat), gestapelte Verlaufskurve (Depot + Cash, aus Snapshots), Depotpositions-Tabelle, Kontenliste.
7. **Konten & Sync** — Konto-Karten (Saldo, Quelle, Zeitstempel/Fehler-Badge, per-Konto-Sync), globaler Sync mit per-Konto-Ticks, TAN-Modal (SecurePlus-Polling | TAN-Eingabe, Abbrechen failt nur FinTS), CSV-Dropzone (idempotent).

Design-Sprache bestätigt = Ticket-04-System: warmes Grau-Papier (#f9f9f7/#0d0d0d), Karten mit 10-px-Radius, System-Font, tabellarische Ziffern, Kategoriefarben aus 04 (Validator-geprüft light+dark, „Sonstiges" bewusst grau als Other-Bucket), Status nur mit Icon+Text. Hand-rolled SVG, kein d3.

Assets: klickbarer Prototyp + 6 Screenshots (oben verlinkt). [Spec assembly](13-spec-assembly.md) wartet damit nur noch auf [Recurring detection](11-recurring-detection.md) und [Security model](12-security-model.md).
