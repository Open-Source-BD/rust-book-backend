DROP INDEX orders_status_idx;
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'refunded';
CREATE INDEX orders_status_idx ON orders (status);
