CREATE TABLE clients (
    CONSTRAINT clients_pk
        PRIMARY KEY (national_id),
    national_id national_id NOT NULL,
    full_name TEXT NOT NULL,
    main_phone_no phone_no NOT NULL,
    secondary_phone_no phone_no NOT NULL,
    email email NOT NULL
);