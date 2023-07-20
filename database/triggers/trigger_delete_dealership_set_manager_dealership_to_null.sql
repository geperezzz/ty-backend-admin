BEGIN;

CREATE FUNCTION delete_dealership_set_manager_dealership_to_null() RETURNS trigger AS $BODY$
    BEGIN
        UPDATE staff
        SET
            employer_dealership_rif = NULL
        WHERE
            national_id = OLD.manager_national_id;

        RETURN OLD;
    END;
$BODY$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_delete_dealership_set_manager_dealership_to_null
BEFORE DELETE ON dealerships
FOR EACH ROW EXECUTE FUNCTION delete_dealership_set_manager_dealership_to_null();

END;