CREATE TABLE services (
    CONSTRAINT services_pk
        PRIMARY KEY (id),
    CONSTRAINT services_coordinator_id_fk
        FOREIGN KEY (coordinator_id) REFERENCES staff (id),
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    coordinator_id TEXT NOT NULL
);