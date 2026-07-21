# Refresh/sync UX

Type: grilling
Status: open
Blocked by: 09

## Question

How syncing feels: manual-only vs scheduled/background fetch per source (FinTS needs TAN — can't be silent; scalable-cli session lifetime?), how TAN challenges surface in the UI (mechanics fixed: challenge/response events over sidecar stdio per [Backend stack decision](06-backend-stack.md)), sync status/error surfacing per Account, and what triggers Snapshot writes vs what a failed/partial sync leaves behind (domain model fixed: Snapshots are append-only per-sync observations, transfer detection re-runs after import).
