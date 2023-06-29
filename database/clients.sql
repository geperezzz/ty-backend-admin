CREATE TABLE clients (
    CONSTRAINT clients_pk
        PRIMARY KEY (id),
    id TEXT,
    full_name TEXT NOT NULL,
    main_phone_no TEXT NOT NULL,
    secondary_phone_no TEXT NOT NULL
    email TEXT NOT NULL,
);