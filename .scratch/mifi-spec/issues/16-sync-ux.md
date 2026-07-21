# Refresh/sync UX

Type: grilling
Status: closed
Assignee: michael
Blocked by: 09

## Question

How syncing feels: manual-only vs scheduled/background fetch per source (FinTS needs TAN — can't be silent; scalable-cli session lifetime?), how TAN challenges surface in the UI (mechanics fixed: challenge/response events over sidecar stdio per [Backend stack decision](06-backend-stack.md)), sync status/error surfacing per Account, and what triggers Snapshot writes vs what a failed/partial sync leaves behind (domain model fixed: Snapshots are append-only per-sync observations, transfer detection re-runs after import).

## Resolution (2026-07-21, grilled with Michael)

1. **Manual-only sync.** One global Sync action fires all syncable sources (FinTS + scalable-cli) concurrently; per-account sync from the account view as secondary affordance. No scheduler, no background fetch, no sync-on-launch. If scalable-cli tokens turn out long-lived, background Scalable refresh can be added later — nothing blocks it.
2. **TAN: blocking modal sheet, two shapes.** Other sources proceed in background; FinTS challenge opens a modal wherever the user is: bank's challenge text verbatim + either code-entry field or decoupled mode ("Confirm in SecurePlus app", automatic polling, cancel). Cancel fails the FinTS source cleanly; rest of run unaffected. No TAN persistence; no photoTAN image rendering in v1 unless Consorsbank requires it (decoupled SecurePlus is the expected path — verify during build).
3. **Status/errors: three passive layers.** (a) Per-account relative "last synced" timestamp — CSV accounts show last import in the same slot, one uniform staleness surface. (b) During run: global spinner + per-account pending/done ticks, app stays usable. (c) Errors: persistent badge on the account + one post-run summary line; click → verbatim sidecar/CLI message + retry. No toasts, no notification center, no sync history — per-source state is last_success_at + last_error only.
4. **Partial failure: atomic per account, nothing on failure.** Commit unit = one account within one source; success writes its transactions + balance/position/price Snapshots in one DB transaction, failure writes nothing (no cleanup, no partial state; last-synced advances only on commit). Snapshot trigger = successful fetch of that data class: depot holdings success → position + price + Verrechnungskonto balance Snapshots; CSV import never writes balance Snapshots (transactions only). Transfer detection + contract matching re-run once at end of run over all newly committed accounts (catches cross-account pairs from the same run).
5. **Fully passive staleness.** No launch prompt, no nudge thresholds — timestamps carry the signal.

Sync Run pinned in [/CONTEXT.md](../../../CONTEXT.md). Unblocks [UI information architecture](18-ui-information-architecture.md) (last blocker).
