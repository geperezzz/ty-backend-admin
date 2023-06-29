CREATE TABLE staff (
    CONSTRAINT staff_pk
        PRIMARY KEY (id),
    CONSTRAINT staff_role_id_fk
        FOREIGN KEY (role_id) REFERENCES roles (id),
    id TEXT,
    full_name TEXT NOT NULL,
    main_phone_no TEXT NOT NULL,
    secondary_phone_no TEXT NOT NULL,
    email TEXT NOT NULL,
    address TEXT NOT NULL,
    role_id INTEGER NOT NULL,
    salary NUMERIC NOT NULL
);