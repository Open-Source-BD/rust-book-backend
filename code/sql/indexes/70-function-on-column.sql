SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_lower_status_idx;
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE lower(status) = 'refunded';
CREATE INDEX orders_lower_status_idx ON orders (lower(status));
ANALYZE orders;
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE lower(status) = 'refunded';
DROP INDEX orders_lower_status_idx;
