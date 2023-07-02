CREATE TABLE recommended_services (
    CONSTRAINT recommended_services_pk
        PRIMARY KEY (service_id, vehicle_model_id, required_usage_time, required_kilometrage),
    CONSTRAINT recommended_services_service_id_fk
        FOREIGN KEY (service_id) REFERENCES services (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT recommended_services_vehicle_model_id_fk
        FOREIGN KEY (vehicle_model_id) REFERENCES vehicle_models (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    service_id INTEGER NOT NULL,
    vehicle_model_id INTEGER NOT NULL,
    required_usage_time INTERVAL YEAR TO MONTH NOT NULL
        CONSTRAINT valid_required_usage_time
            CHECK (required_usage_time >= '0 months'), -- unbound intervals could be negative
    required_kilometrage NUMERIC NOT NULL
        CONSTRAINT valid_required_kilometrage
            CHECK (required_kilometrage >= 0)
);