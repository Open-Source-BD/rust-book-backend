SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_total_cents_idx;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE total_cents = 1234;
CREATE INDEX orders_total_cents_idx ON orders (total_cents);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE total_cents = 1234;
DROP INDEX orders_total_cents_idx;
