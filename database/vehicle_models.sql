CREATE TABLE vehicle_models (
    CONSTRAINT vehicle_models_pk
        PRIMARY KEY (id),
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    seat_count INTEGER NOT NULL,
    weight_in_kg INTEGER NOT NULL,
    octane_rating SMALLINT NOT NULL,
    gearbox_oil_type TEXT NOT NULL,
    engine_oil_type TEXT NOT NULL,
    engine_coolant_type TEXT NOT NULL
);