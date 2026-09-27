SET client_min_messages = warning;
DROP INDEX IF EXISTS orders_refunded_idx;
CREATE INDEX orders_refunded_idx ON orders (customer_id) WHERE status = 'refunded';
EXPLAIN (COSTS OFF)
SELECT * FROM orders WHERE customer_id = 42 AND status = 'refunded';
SELECT pg_size_pretty(pg_relation_size('orders_customer_id_idx')) AS all_orders_index,
       pg_size_pretty(pg_relation_size('orders_refunded_idx'))    AS refunded_only_index;
DROP INDEX orders_refunded_idx;
