SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_customer_status_idx;
CREATE INDEX orders_customer_status_idx ON orders (customer_id, status);
EXPLAIN (COSTS OFF)
SELECT * FROM orders WHERE customer_id = 42 AND status = 'refunded';
DROP INDEX orders_customer_status_idx;
