CREATE DOMAIN national_id AS TEXT
    CONSTRAINT valid_national_id
        CHECK (VALUE SIMILAR TO '(V|E)-[0-9]{1,}');