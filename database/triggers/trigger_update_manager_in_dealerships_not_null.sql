BEGIN;

CREATE FUNCTION update_manager_in_dealerships_not_null() RETURNS trigger AS $BODY$
    BEGIN
        IF NEW.manager_national_id IS NULL THEN
            RAISE EXCEPTION 'manager_national_id cannot be updated to be null'
                USING HINT = E'Check the value you\'re passing as manager\'s id';
        END IF;
        
        RETURN NEW;
    END;
$BODY$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_manager_in_dealerships_not_null
AFTER UPDATE ON dealerships
FOR EACH ROW EXECUTE FUNCTION update_manager_in_dealerships_not_null();

END;