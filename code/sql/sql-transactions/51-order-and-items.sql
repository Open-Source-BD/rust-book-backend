SET client_min_messages = warning;
DROP TABLE IF EXISTS order_items;
DROP TABLE IF EXISTS orders;
CREATE TABLE orders (
    id       integer PRIMARY KEY,
    customer text NOT NULL
);
CREATE TABLE order_items (
    order_id integer NOT NULL REFERENCES orders (id),
    product  text NOT NULL,
    quantity integer NOT NULL CHECK (quantity > 0)
);
BEGIN;
INSERT INTO orders (id, customer) VALUES (1, 'Ada');
INSERT INTO order_items (order_id, product, quantity) VALUES (1, 'Dune', 1), (1, 'Emma', 2);
COMMIT;
BEGIN;
INSERT INTO orders (id, customer) VALUES (2, 'Linus');
INSERT INTO order_items (order_id, product, quantity) VALUES (2, 'Dune', 1), (2, 'Emma', 0);
COMMIT;
SELECT * FROM orders ORDER BY id;
SELECT * FROM order_items ORDER BY order_id, product;
