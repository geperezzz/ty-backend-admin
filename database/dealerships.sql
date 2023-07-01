CREATE TABLE dealerships (
    CONSTRAINT dealerships_pk
        PRIMARY KEY (rif),
    CONSTRAINT dealerships_city_id_fk
        FOREIGN KEY (city_id) REFERENCES cities (id),
    CONSTRAINT dealerships_manager_national_id_fk
        FOREIGN KEY (manager_national_id) REFERENCES staff (national_id),
    rif rif NOT NULL,
    name TEXT NOT NULL,
    city_id INTEGER NOT NULL,
    manager_national_id national_id NOT NULL
);