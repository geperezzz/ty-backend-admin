CREATE FUNCTION generate_invoice() RETURNS trigger AS $$
    DECLARE
        client_national_id national_id;
        dealership_rif rif;
        last_year_paid_services INTEGER;
    BEGIN
        SELECT v.owner_national_id, o.dealership_rif
        INTO client_national_id, dealership_rif
        FROM
            orders AS o
            INNER JOIN vehicles AS v ON o.vehicle_plate = v.plate
        WHERE
            o.id = NEW.order_id;
        
        SELECT COUNT(*)
        INTO last_year_paid_services
        FROM
            invoices AS i
            INNER JOIN orders AS o ON i.order_id = o.id
            INNER JOIN orders_details AS od ON o.id = od.order_id
            INNER JOIN vehicles AS v ON o.vehicle_plate = v.plate
        WHERE
            v.owner_national_id = client_national_id
            AND o.dealership_rif = dealership_rif
            AND AGE(o.checkin_timestamp) <= '1 year'
        GROUP BY
            od.order_id,
            od.service_id;
        
        SELECT di.discount_percentage
        INTO NEW.discount
        FROM
            dealerships AS de
            INNER JOIN discounts AS di ON de.rif = di.dealership_rif
        WHERE
            de.rif = dealership_rif
            AND last_year_paid_services >= di.required_annual_service_usage_count
        ORDER BY
            di.discount_percentage DESC;
        NEW.discount := COALESCE(NEW.discount, 0);

        SELECT SUM(od.worked_hours * od.price_per_hour) * (1 - NEW.discount)
        INTO NEW.amount_due
        FROM
            orders AS o
            INNER JOIN orders_details AS od ON o.id = od.order_id
        WHERE
            o.id = NEW.order_id;

        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_generate_invoice
BEFORE INSERT ON invoices
FOR EACH ROW EXECUTE FUNCTION generate_invoice();