CREATE TABLE managers (
    CONSTRAINT manager_pk
        PRIMARY KEY (national_id),
    CONSTRAINT manager_national_id_fk
        FOREIGN KEY (national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT manager_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    national_id national_id NOT NULL,
    dealership_rif rif UNIQUE NOT NULL
);