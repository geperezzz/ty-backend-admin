CREATE DOMAIN email AS TEXT
    CONSTRAINT valid_email
        CHECK (VALUE ~ '^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$');