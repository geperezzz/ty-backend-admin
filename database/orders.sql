CREATE TABLE orders (
    CONSTRAINT orders_pk
        PRIMARY KEY (id),
    CONSTRAINT orders_vehicle_plate_fk
        FOREIGN KEY (vehicle_plate) REFERENCES vehicles (plate),
    CONSTRAINT orders_analist_national_id_fk
        FOREIGN KEY (analist_national_id) REFERENCES staff (national_id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    vehicle_plate TEXT NOT NULL,
    reservation_timestamp TIMESTAMP NOT NULL,
    checkin_timestamp TIMESTAMP,
    estimated_checkout_timestamp TIMESTAMP,
    checkout_timestamp TIMESTAMP,
    analist_national_id national_id NOT NULL,
    vehicle_caretaker_national_id national_id,
    vehicle_caretaker_name TEXT,
    vehicle_kilometrage NUMERIC NOT NULL
        CONSTRAINT valid_vehicle_kilometrage
            CHECK (vehicle_kilometrage > 0),
    CONSTRAINT consistency_between_reservation_checkin_and_checkout_timestamps
        CHECK (
            reservation_timestamp <= checkin_timestamp
            AND checkin_timestamp <= estimated_checkout_timestamp
            AND estimated_checkout_timestamp <= checkout_timestamp
        )
);