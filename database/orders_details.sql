CREATE TABLE orders_details (
    CONSTRAINT orders_details_pk
        PRIMARY KEY (order_id, activity_number, service_id),
    CONSTRAINT orders_details_activity_number_service_id_fk
        FOREIGN KEY (activity_number, service_id) REFERENCES activities (activity_number, service_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    order_id INTEGER NOT NULL,
    activity_number INTEGER NOT NULL,
    service_id INTEGER NOT NULL,
    price_per_hour NUMERIC NOT NULL
        CONSTRAINT valid_price_per_hour
            CHECK (price_per_hour >= 0),
    worked_hours NUMERIC NOT NULL
        CONSTRAINT valid_worked_hours
            CHECK (worked_hours > 0)
);