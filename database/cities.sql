CREATE TABLE cities (
    CONSTRAINT cities_pk
        PRIMARY KEY (id),
    CONSTRAINT cities_state_id_fk
        FOREIGN KEY (state_id) REFERENCES states (id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    state_id INTEGER NOT NULL
);