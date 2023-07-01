CREATE TABLE stock (
    CONSTRAINT stock_pk
        PRIMARY KEY (product_id, dealership_rif),
    CONSTRAINT stock_product_id_fk
        FOREIGN KEY (product_id) REFERENCES products (id),
    CONSTRAINT stock_dealership_rif_fk
        FOREIGN KEY (dealership_rif) REFERENCES dealerships (rif),
    product_id INTEGER NOT NULL,
    dealership_rif rif NOT NULL,
    product_cost NUMERIC NOT NULL
        CONSTRAINT valid_product_cost
            CHECK (product_cost >= 0),
    stock_count INTEGER NOT NULL,
    vendor_name TEXT NOT NULL,
    max_capacity INTEGER NOT NULL,
    min_capacity INTEGER NOT NULL
        CONSTRAINT valid_min_capacity
            CHECK (min_capacity >= 0),
    CONSTRAINT consistency_between_min_capacity_and_stock_count
        CHECK (stock_count >= min_capacity),
    CONSTRAINT consistency_between_min_capacity_and_max_capacity
        CHECK (max_capacity >= min_capacity)
);