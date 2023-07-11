CREATE TABLE dealerships (
    CONSTRAINT dealerships_pk
        PRIMARY KEY (rif),
    CONSTRAINT dealerships_city_id_fk
        FOREIGN KEY (city_number, state_id) REFERENCES cities (city_number, state_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT dealerships_manager_national_id_fk
        FOREIGN KEY (manager_national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    rif rif NOT NULL,
    name TEXT NOT NULL,
    city_number INTEGER NOT NULL,
    state_id INTEGER NOT NULL,
    manager_national_id national_id NOT NULL
);