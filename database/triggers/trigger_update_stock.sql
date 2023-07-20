CREATE FUNCTION update_stock() RETURNS trigger AS $$
    DECLARE
        source_dealership_rif rif;
        available_products INTEGER;
    BEGIN
        SELECT dealership_rif
        INTO source_dealership_rif
        FROM orders
        WHERE id = NEW.order_id;

        SELECT product_count
        INTO available_products
        FROM stock
        WHERE
            product_id = NEW.product_id
            AND dealership_rif = source_dealership_rif;
        
        IF NEW.application_count > available_products THEN
            RAISE EXCEPTION 'Not enough produc   ts to apply. Trying to apply % products while having in stock % for dealership with rif %',
                NEW.application_coun t, available_products, source_dealership_rif;
        END IF;
    
        UPDATE stock
        SET product_count = product_count - NEW.application_count
        WHERE
            product_id = NEW.product_id
            AND dealership_rif = source_dealership_rif;

        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_stock
BEFORE INSERT OR UPDATE ON products_applications
FOR EACH ROW EXECUTE FUNCTION update_stock();