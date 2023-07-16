CREATE DOMAIN telephone_no AS TEXT
    CONSTRAINT valid_telephone_no
        CHECK (VALUE ~ '^\+?\d{1,4}?[-.\s]?\(?\d{1,3}?\)?[-.\s]?\d{1,4}[-.\s]?\d{1,4}[-.\s]?\d{1,9}$');