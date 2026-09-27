SET client_min_messages = warning;
DROP TABLE IF EXISTS orders;
CREATE TABLE orders (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer_id integer NOT NULL,
    status      text NOT NULL,
    total_cents integer NOT NULL
);
INSERT INTO orders (customer_id, status, total_cents)
SELECT n % 5000,
       CASE WHEN n % 10 = 0 THEN 'refunded' ELSE 'paid' END,
       (n * 37) % 10000
FROM generate_series(1, 100000) AS n;
ANALYZE orders;
SELECT count(*) AS orders FROM orders;
