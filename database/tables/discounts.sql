CREATE TABLE discounts (
    CONSTRAINT discounts_pk
        PRIMARY KEY (discount_number, dealership_rif),
    CONSTRAINT discounts_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif)
            ON UPDATE CASCADE
            ON DELETE RESTRICT,
    discount_number INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    dealership_rif rif NOT NULL,
    discount_percentage NUMERIC NOT NULL
        CONSTRAINT valid_discount_percentage
            CHECK (discount_percentage BETWEEN 0 AND 1),
    required_annual_service_usage_count SMALLINT NOT NULL
        CONSTRAINT valid_required_annual_service_usage_count
            CHECK (required_annual_service_usage_count >= 0)
);