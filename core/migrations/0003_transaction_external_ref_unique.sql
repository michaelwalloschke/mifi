-- Idempotency belt-and-braces for CSV re-imports (SPEC.md §6): a second dedup key
-- alongside Import Hash. SQLite unique indexes treat NULLs as distinct, so this only
-- constrains rows that actually carry a native external_ref.
CREATE UNIQUE INDEX idx_transaction_source_external_ref
    ON "transaction" (source, external_ref)
    WHERE external_ref IS NOT NULL;
