CREATE TABLE invoices (
    CONSTRAINT invoices_pk
        PRIMARY KEY (id),
    CONSTRAINT invoices_order_id_fk
        FOREIGN KEY (order_id) REFERENCES orders (id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    order_id INTEGER NOT NULL,
    amount_due NUMERIC NOT NULL
        CONSTRAINT valid_amount_due
            CHECK (amount_due >= 0),
    discount NUMERIC NOT NULL
        CONSTRAINT valid_discount
            CHECK (discount BETWEEN 0 AND 1),
    issue_date DATE NOT NULL
);