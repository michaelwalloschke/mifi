# Categorization approach

Type: grilling
Status: closed
Assignee: michael
Blocked by: 03

## Question

How are transactions auto-categorized: deterministic rules (regex/merchant match), local ML/LLM, hybrid (rules first, model for the rest)? Include: how salvaged Finanzguru categories seed the system, correction UX (user fixes → system learns?), and whether any cloud LLM is acceptable given the privacy stance (default: no).

## Resolution (2026-07-22)

Three-layer pipeline, all local. Cloud LLM: hard no — transactions never leave the machine, no fallback toggle.

1. **Merchant memory (deterministic, first).** Normalize counterparty (strip mid-word spaces like `HelloFre sh`, casefold, strip legal suffixes) → learned `normalized merchant → category` table. Match assigns with provenance `auto`.
2. **Naive Bayes (fallback).** Hand-rolled in Rust core, token counts in SQLite, tokens from normalized counterparty + purpose. Assigns only when top1/top2 posterior ratio ≥ ~3 (tunable); below threshold → transaction stays uncategorized (review queue). No silent low-confidence guesses.
3. **Local LLM (v1, third auto-layer).** Async post-sync sweep over rows layers 1–2 left uncategorized, plus on-demand trigger from the review queue. Single configurable OpenAI-compatible base URL, default `http://localhost:11434/v1` (ollama; LM Studio via port change). Prompt carries the fixed taxonomy; only a valid category id is accepted — anything else leaves the row uncategorized. Endpoint unreachable → pass silently skipped (passive status only). Never blocks Sync Run, never joins its atomicity: LLM writes are ordinary post-commit categorizations with provenance `auto`.

**Learning loop.** A user correction (a) writes/overwrites the merchant-memory rule, (b) increments NB counts, (c) offers one-click retro-apply to other same-merchant rows — touching only `auto`/uncategorized rows, never user-set (CONTEXT.md provenance rule). The LLM layer never learns; corrections feed layers 1–2 only.

**Seeding from Finanzguru salvage (4.6k rows; the 912 `Umbuchung=ja` transfer legs excluded from both layers).**
- Memory rules purity-gated: merchant needs ≥2 rows and ≥80 % one category (defaults, tunable at import). Mixed merchants (Amazon-likes) fall through to NB/queue.
- All rows seed NB token counts.

**Taxonomy.** Finanzguru's depth-2 set adopted verbatim: 14 mains / 64 pairs, including `Sonstiges/Bargeld` and `Sonstiges/Kreditkartenabrechnung` (kept as genuine expense subs — card and cash sit outside the 5-account perimeter, so those rows are real outflows under the lone-leg rule). Trimming/merging is a post-v1 in-app activity via category CRUD, not a spec decision.
