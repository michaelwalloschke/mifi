-- Fixed closed set of 5 Accounts (SPEC.md §1). No IBANs/PII here — those live only in
-- local, untracked runtime config for the FinTS sidecar and seed importers.

INSERT INTO account (id, institution, name, type, source_kind) VALUES
    (1, 'Consorsbank', 'Consorsbank Giro', 'checking', 'fints'),
    (2, 'Consorsbank', 'Consorsbank Tagesgeld', 'savings', 'fints'),
    (3, 'Scalable Capital', 'Scalable depot', 'depot', 'scalable-cli'),
    (4, 'Scalable Capital', 'Scalable Verrechnungskonto', 'cash', 'scalable-cli'),
    (5, 'PayPal', 'PayPal', 'e-money', 'csv-paypal');
