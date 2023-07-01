CREATE TABLE activities (
    CONSTRAINT activities_pk
        PRIMARY KEY (activity_number, service_id),
    CONSTRAINT activities_service_id_fk
        FOREIGN KEY (service_id) REFERENCES services (id),
    activity_number INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    service_id INTEGER NOT NULL,
    description TEXT NOT NULL,
    price_per_hour NUMERIC NOT NULL
        CONSTRAINT valid_price_per_hour
            CHECK (price_per_hour >= 0)
);