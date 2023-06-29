CREATE TABLE activities (
    CONSTRAINT activities_pk
        PRIMARY KEY (id, service_id),
    CONSTRAINT activities_service_id_fk
        FOREIGN KEY (service_id) REFERENCES services (id),
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    service_id INTEGER,
    description TEXT NOT NULL,
    cost_per_hour NUMERIC NOT NULL
);