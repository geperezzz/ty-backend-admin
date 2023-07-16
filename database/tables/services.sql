CREATE TABLE services (
    CONSTRAINT services_pk
        PRIMARY KEY (id),
    CONSTRAINT services_coordinator_national_id_fk
        FOREIGN KEY (coordinator_national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    coordinator_national_id national_id NOT NULL
);