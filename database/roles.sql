CREATE TABLE roles (
    CONSTRAINT roles_pk
        PRIMARY KEY (id),
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);