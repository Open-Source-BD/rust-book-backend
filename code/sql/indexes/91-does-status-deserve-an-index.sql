SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_status_idx;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE status = 'refunded';
CREATE INDEX orders_status_idx ON orders (status);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE status = 'refunded';
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'paid';
DROP INDEX orders_status_idx;
