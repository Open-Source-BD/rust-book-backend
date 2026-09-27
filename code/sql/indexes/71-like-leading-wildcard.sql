EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'refunded';
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status LIKE '%unded';
