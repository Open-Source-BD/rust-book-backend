SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_customer_id_idx;
CREATE INDEX orders_customer_id_idx ON orders (customer_id);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE customer_id = 42;
