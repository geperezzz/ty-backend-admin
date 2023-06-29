CREATE TABLE dealerships (
    CONSTRAINT dealerships_pk
        PRIMARY KEY (rif),
    CONSTRAINT dealerships_city_id_fk
        FOREIGN KEY (city_id) REFERENCES cities (id),
    CONSTRAINT dealerships_manager_id_fk
        FOREIGN KEY (manager_id) REFERENCES staff (id),
    rif TEXT,
    name TEXT NOT NULL,
    city_id INTEGER NOT NULL,
    manager_id TEXT NOT NULL
);