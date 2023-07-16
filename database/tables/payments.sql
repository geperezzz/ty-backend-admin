CREATE TYPE payment_type AS ENUM (
    'bolivares',
    'foreign-currency',
    'transfer',
    'debit-card',
    'credit-card'
);

CREATE TABLE payments (
    CONSTRAINT payments_pk
        PRIMARY KEY (payment_number, invoice_id),
    CONSTRAINT payments_invoice_id_pk
        FOREIGN KEY (invoice_id) REFERENCES invoices (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    payment_number INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    invoice_id INTEGER NOT NULL,
    amount_paid NUMERIC NOT NULL
        CONSTRAINT valid_amount_paid
            CHECK (amount_paid > 0),
    payment_date DATE NOT NULL,
    payment_type payment_type NOT NULL,
    card_number TEXT NOT NULL,
    card_bank TEXT NOT NULL
);