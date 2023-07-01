CREATE TABLE vehicle_models (
    CONSTRAINT vehicle_models_pk
        PRIMARY KEY (id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    seat_count INTEGER NOT NULL
        CONSTRAINT valid_seat_count
            CHECK (seat_count > 0),
    weight_in_kg NUMERIC NOT NULL
        CONSTRAINT valid_weight_in_kg
            CHECK (weight_in_kg > 0),
    octane_rating SMALLINT NOT NULL
        CONSTRAINT valid_octane_rating
            CHECK (octane_rating > 0),
    gearbox_oil_type TEXT NOT NULL,
    engine_oil_type TEXT NOT NULL,
    engine_coolant_type TEXT NOT NULL
);