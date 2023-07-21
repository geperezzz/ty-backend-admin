ALTER TABLE dealerships
    ADD CONSTRAINT dealerships_manager_national_id_fk
        FOREIGN KEY (manager_national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT;