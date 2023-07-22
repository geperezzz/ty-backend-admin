CREATE FUNCTION check_employee_is_operative() RETURNS trigger AS 
$BODY$
    DECLARE
        manager_role_id INTEGER;
    BEGIN
        SELECT role_id
        INTO manager_role_id
        FROM roles
        WHERE
            name = 'Encargado';
        
        IF NEW.role_id = manager_role_id THEN
            RAISE EXCEPTION 'Provided employee is not an operative, cannot create as operative.'
                USING HINT = E'Check employee\'s role, for example, it cannot be \'Encargado\'.';
        END IF;

        RETURN NEW;
    END;
$BODY$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_check_employee_is_operative
BEFORE INSERT ON operatives
FOR EACH ROW EXECUTE FUNCTION check_employee_is_operative();