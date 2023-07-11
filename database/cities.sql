CREATE TABLE cities (
    CONSTRAINT cities_pk
        PRIMARY KEY (city_number, state_id),
    CONSTRAINT cities_state_id_fk
        FOREIGN KEY (state_id) REFERENCES states (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    city_number INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    state_id INTEGER NOT NULL,
    CONSTRAINT unique_name_state_id
        UNIQUE (name, state_id)
);