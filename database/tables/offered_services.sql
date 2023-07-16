CREATE TABLE offered_services (
    CONSTRAINT offered_services_pk
        PRIMARY KEY (service_id, dealership_rif),
    CONSTRAINT offered_services_service_id_fk
        FOREIGN KEY (service_id) REFERENCES services (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT offered_services_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    service_id INTEGER NOT NULL,
    dealership_rif rif NOT NULL,
    min_reservation_time INTERVAL DAY TO MINUTE NOT NULL
        CONSTRAINT valid_min_reservation_time
            CHECK (min_reservation_time >= '0 minutes'), -- unbound intervals could be negative
    max_reservation_time INTERVAL DAY TO MINUTE NOT NULL,
        CONSTRAINT valid_max_reservation_time
            CHECK (max_reservation_time >= '0 minutes'), -- same as above
    service_capacity INTEGER NOT NULL
        CONSTRAINT valid_service_capacity
            CHECK (service_capacity > 0),
    CONSTRAINT consistency_between_reservation_times
        CHECK (max_reservation_time >= min_reservation_time)
);