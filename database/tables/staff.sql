CREATE TABLE staff (
    CONSTRAINT staff_pk
        PRIMARY KEY (national_id),
    CONSTRAINT staff_role_id_fk
        FOREIGN KEY (role_id) REFERENCES roles (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    national_id national_id NOT NULL,
    full_name TEXT NOT NULL,
    main_phone_no TEXT NOT NULL,
    secondary_phone_no TEXT NOT NULL,
    email email NOT NULL,
    address TEXT NOT NULL,
    role_id INTEGER NOT NULL,
    salary NUMERIC NOT NULL
        CONSTRAINT valid_salary
            CHECK (salary >= 0)
);