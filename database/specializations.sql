CREATE TABLE specializations (
    CONSTRAINT specializations_pk
        PRIMARY KEY (employee_national_id, service_id),
    CONSTRAINT specializations_employee_national_id_fk
        FOREIGN KEY (employee_national_id) REFERENCES staff (national_id),
    CONSTRAINT specializations_service_id_fk
        FOREIGN KEY (service_id) REFERENCES services (id),
    employee_national_id national_id NOT NULL,
    service_id INTEGER NOT NULL
);