SET client_min_messages = warning;
DROP TABLE IF EXISTS products;
CREATE TABLE products (
    id          bigint,
    name        text,
    price_cents integer,
    in_stock    boolean,
    added_on    date,
    sku         uuid
);
INSERT INTO products VALUES (1, 'Mug', 1299, true, '2026-09-01', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11');
INSERT INTO products VALUES (2, 'Poster', 'cheap', true, '2026-09-02', NULL);
INSERT INTO products VALUES (3, 'Pen', 199, NULL, NULL, NULL);
SELECT * FROM products;
SELECT name FROM products WHERE in_stock IS NULL;
SELECT name FROM products WHERE in_stock = NULL;
