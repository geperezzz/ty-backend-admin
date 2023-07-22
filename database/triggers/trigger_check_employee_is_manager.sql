CREATE FUNCTION check_employee_is_manager() RETURNS trigger AS 
$BODY$
    DECLARE
        manager_role_id INTEGER;
    BEGIN
        SELECT role_id
        INTO manager_role_id
        FROM roles
        WHERE
            name = 'Encargado';
        
        IF NEW.role_id != manager_role_id THEN
            RAISE EXCEPTION 'Provided employee is not a manager, cannot create as manager.'
                USING HINT = E'Check if employee\'s role is \'Encargado\'.';
        END IF;

        RETURN NEW;
    END;
$BODY$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_check_employee_is_manager
BEFORE INSERT ON managers
FOR EACH ROW EXECUTE FUNCTION check_employee_is_manager();