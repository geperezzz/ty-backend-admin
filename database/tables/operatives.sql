CREATE TABLE operatives (
    CONSTRAINT operative_pk
        PRIMARY KEY (national_id),
    CONSTRAINT operative_national_id_fk
        FOREIGN KEY (national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT operative_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    national_id national_id NOT NULL,
    dealership_rif rif NOT NULL
);