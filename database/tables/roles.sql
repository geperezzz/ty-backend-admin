CREATE TABLE roles (
    CONSTRAINT roles_pk
        PRIMARY KEY (id),
    id INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

INSERT INTO roles
    (name, description)
VALUES
    ('Encargado', 'Empleado que gestiona un concesionario.');

INSERT INTO roles
    (name, description)
VALUES
    ('Analista', 'Empleado que recibe vehiculos para su revisión y genera órdenes si es necesario.');
