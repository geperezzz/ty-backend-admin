BEGIN;

CREATE DOMAIN rif AS TEXT
    CONSTRAINT valid_rif
        CHECK (VALUE SIMILAR TO '(V|E|J)-[0-9]{1,}');

CREATE DOMAIN national_id AS TEXT
    CONSTRAINT valid_national_id
        CHECK (VALUE SIMILAR TO '(V|E)-[0-9]{1,}');

CREATE DOMAIN email AS TEXT
    CONSTRAINT valid_email
        CHECK (VALUE ~ '^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$');

CREATE DOMAIN telephone_no AS TEXT
    CONSTRAINT valid_telephone_no
        CHECK (VALUE ~ '^\+?\d{1,4}?[-.\s]?\(?\d{1,3}?\)?[-.\s]?\d{1,4}[-.\s]?\d{1,4}[-.\s]?\d{1,9}$');

COMMIT;