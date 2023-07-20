CREATE TABLE products_applications (
    CONSTRAINT products_applications_pk
        PRIMARY KEY (order_id, activity_number, service_id, product_id, employee_national_id),
    CONSTRAINT products_applications_order_id_activity_number_service_id_fk
        FOREIGN KEY (order_id, activity_number, service_id) REFERENCES orders_details (order_id, activity_number, service_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    CONSTRAINT products_applications_employee_national_id_fk
        FOREIGN KEY (employee_national_id) REFERENCES staff (national_id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    order_id INTEGER NOT NULL,
    activity_number INTEGER NOT NULL,
    service_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    employee_national_id national_id NOT NULL,
    application_count INTEGER NOT NULL
        CONSTRAINT valid_application_count
            CHECK (application_count > 0),
    product_cost NUMERIC NOT NULL
        CONSTRAINT valid_product_cost
            CHECK (product_cost >= 0)
);