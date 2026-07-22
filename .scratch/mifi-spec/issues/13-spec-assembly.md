# Spec assembly

Type: task
Status: closed
Assignee: michael
Closed: 2026-07-22
Blocked by: 04, 05, 06, 07, 08, 09, 10, 11, 12, 16, 17, 18, 20

## Question

Assemble SPEC.md from all closed tickets: architecture (Tauri + chosen backend), data schema (from domain model), fetch pipeline (FinTS + aggregator + TAN UX), categorization, viz direction (linked prototype), budgeting, recurring detection, security, and a v1 cut-line (what ships first vs later). Done when a fresh implementation session could start building from SPEC.md alone — that closes the map.

## Resolution (2026-07-22)

[SPEC.md](../../../SPEC.md) written at repo root, next to CONTEXT.md. 16 sections assembled from all 16 closed tickets: scope table (5 accounts, epochs, sync paths), architecture diagram (Tauri + Foldkit + Rust core + Python sidecar + scalable-cli), domain-model build rules (CONTEXT.md normative), SQLite schema (13 tables, indicative columns), sync pipeline, 3 CSV formats, 3-layer categorization, recurring detection, budgeting, net worth, seed/backfill plan, security, UI (7 screens + design language, prototypes linked), **v1 cut-line** (ships vs. explicit deferrals/rejections), external dependencies open at build time, and a non-binding build-order sketch.

Notes:
- Spec is an assembly, not a re-derivation — each section states the decision + build detail and links the ticket/asset for rationale. CONTEXT.md wins on conflict.
- One schema addition surfaced: `csv-consorsbank` joins the Source enum (bridge import from History backfill cut) — CONTEXT.md Source list to be amended during build.
- Open tasks 14 (FinTS product ID), 15 (scalable-cli beta), 19 (CSV verification) are build-effort inputs, listed in SPEC.md §15; none blocks starting.

This closes the map: a fresh implementation session can start building from SPEC.md alone.
