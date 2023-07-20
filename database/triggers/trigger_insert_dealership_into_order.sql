CREATE FUNCTION insert_dealership_into_order() RETURNS trigger AS $$
    BEGIN
        SELECT COALESCE(helped_dealership_rif, employer_dealership_rif)
        INTO STRICT NEW.dealership_rif
        FROM staff
        WHERE national_id = NEW.analist_national_id;

        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_insert_dealership_into_order
BEFORE INSERT ON orders
FOR EACH ROW EXECUTE FUNCTION insert_dealership_into_order();