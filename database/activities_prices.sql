CREATE TABLE activities_prices (
    CONSTRAINT activities_prices_pk
        PRIMARY KEY (activity_number, service_id, dealership_rif),
    CONSTRAINT activities_prices_activity_number_service_id_fk
        FOREIGN KEY (activity_number, service_id) REFERENCES activities (activity_number, service_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT activities_prices_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    activity_number INTEGER NOT NULL,
    service_id INTEGER NOT NULL,
    dealership_rif rif NOT NULL,
    price_per_hour NUMERIC NOT NULL
        CONSTRAINT valid_price_per_hour
            CHECK (price_per_hour >= 0)
);