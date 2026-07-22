-- Initial schema: SPEC.md §4. All money in integer cents. EUR-only.

CREATE TABLE account (
    id INTEGER PRIMARY KEY,
    institution TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('checking', 'savings', 'depot', 'cash', 'e-money')),
    source_kind TEXT NOT NULL,
    fints_client_state BLOB
);

CREATE TABLE category (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES category (id),
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('income', 'expense'))
);

CREATE TABLE contract (
    id INTEGER PRIMARY KEY,
    normalized_counterparty TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('income', 'expense')),
    expected_amount_cents INTEGER NOT NULL,
    tolerance INTEGER NOT NULL,
    interval TEXT NOT NULL CHECK (interval IN ('weekly', 'biweekly', 'monthly', 'quarterly', 'yearly')),
    category_id INTEGER REFERENCES category (id),
    status TEXT NOT NULL CHECK (status IN ('detected', 'confirmed', 'dismissed', 'ended')),
    creditor_id TEXT,
    mandate_reference TEXT,
    next_expected_date TEXT
);

CREATE TABLE "transaction" (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES account (id),
    booking_date TEXT NOT NULL,
    amount_cents INTEGER NOT NULL,
    counterparty_raw TEXT NOT NULL,
    counterparty_normalized TEXT NOT NULL,
    purpose_raw TEXT NOT NULL,
    purpose_normalized TEXT NOT NULL,
    import_hash TEXT NOT NULL,
    occurrence_index INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL CHECK (
        source IN (
            'fints', 'scalable-cli', 'csv-paypal', 'csv-scalable',
            'csv-consorsbank', 'finanzguru-seed'
        )
    ),
    external_ref TEXT,
    fx_metadata TEXT,
    contract_id INTEGER REFERENCES contract (id),
    UNIQUE (account_id, import_hash, occurrence_index)
);

CREATE INDEX idx_transaction_account_date ON "transaction" (account_id, booking_date);

CREATE TABLE split (
    id INTEGER PRIMARY KEY,
    transaction_id INTEGER NOT NULL REFERENCES "transaction" (id),
    amount_cents INTEGER NOT NULL,
    category_id INTEGER REFERENCES category (id),
    category_source TEXT NOT NULL CHECK (category_source IN ('auto', 'user'))
);

CREATE INDEX idx_split_transaction ON split (transaction_id);
CREATE INDEX idx_split_category ON split (category_id);

CREATE TABLE transfer (
    id INTEGER PRIMARY KEY,
    leg_a_txn_id INTEGER NOT NULL REFERENCES "transaction" (id),
    leg_b_txn_id INTEGER NOT NULL REFERENCES "transaction" (id),
    link_source TEXT NOT NULL CHECK (link_source IN ('auto', 'user')),
    UNIQUE (leg_a_txn_id, leg_b_txn_id)
);

CREATE TABLE budget_target (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES category (id),
    amount_cents INTEGER,
    effective_from_month TEXT NOT NULL
);

CREATE INDEX idx_budget_target_category ON budget_target (category_id, effective_from_month);

CREATE TABLE balance_snapshot (
    account_id INTEGER NOT NULL REFERENCES account (id),
    date TEXT NOT NULL,
    balance_cents INTEGER NOT NULL,
    PRIMARY KEY (account_id, date)
);

CREATE TABLE position_snapshot (
    account_id INTEGER NOT NULL REFERENCES account (id),
    isin TEXT NOT NULL,
    date TEXT NOT NULL,
    quantity REAL NOT NULL,
    price INTEGER NOT NULL,
    valuation_cents INTEGER NOT NULL,
    PRIMARY KEY (account_id, isin, date)
);

CREATE TABLE price (
    isin TEXT NOT NULL,
    date TEXT NOT NULL,
    price INTEGER NOT NULL,
    PRIMARY KEY (isin, date)
);

CREATE TABLE merchant_rule (
    normalized_merchant TEXT PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES category (id)
);

CREATE TABLE nb_token_count (
    token TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES category (id),
    count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (token, category_id)
);

CREATE TABLE sync_state (
    source TEXT PRIMARY KEY,
    last_success_at TEXT,
    last_error TEXT
);
