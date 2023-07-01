CREATE TABLE supply_lines (
    CONSTRAINT supply_lines_pk
        PRIMARY KEY (id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL
);