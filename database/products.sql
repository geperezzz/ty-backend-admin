CREATE TABLE products (
    CONSTRAINT products_pk
        PRIMARY KEY (id),
    CONSTRAINT products_supply_line_id_fk
        FOREIGN KEY (supply_line_id) REFERENCES supply_lines (id),
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_ecologic BOOLEAN NOT NULL,
    supply_line_id INTEGER NOT NULL
);