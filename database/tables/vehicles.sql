CREATE TABLE vehicles (
    CONSTRAINT vehicles_pk
        PRIMARY KEY (plate),
    CONSTRAINT vehicles_model_id_fk
        FOREIGN KEY (model_id) REFERENCES vehicle_models (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    plate TEXT NOT NULL,
    brand TEXT NOT NULL,
    model_id INTEGER NOT NULL,
    serial_no TEXT NOT NULL,
    engine_serial_no TEXT NOT NULL,
    color TEXT NOT NULL,
    purchase_date DATE NOT NULL,
    additional_info TEXT,
    maintenance_summary TEXT,
    owner_national_id national_id NOT NULL
);