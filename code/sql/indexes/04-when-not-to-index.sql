SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_status_idx;
CREATE INDEX orders_status_idx ON orders (status);
ANALYZE orders;
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'paid';
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'refunded';
\di
