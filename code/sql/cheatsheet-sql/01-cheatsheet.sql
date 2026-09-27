SET client_min_messages = warning;
DROP TABLE IF EXISTS shop_orders, shop_items;
-- ANCHOR: create-table
CREATE TABLE shop_items (
    id          integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name        text    NOT NULL,
    price_cents integer NOT NULL,
    in_stock    boolean
);
-- ANCHOR_END: create-table
-- ANCHOR: insert
INSERT INTO shop_items (name, price_cents, in_stock) VALUES
    ('Mug', 1299, true),
    ('Poster', 499, false),
    ('Pen', 199, NULL),
    ('Sticker', 99, true);
-- ANCHOR_END: insert
-- ANCHOR: select-filter-sort
SELECT name, price_cents
FROM shop_items
WHERE price_cents < 1000
ORDER BY price_cents DESC
LIMIT 2;
-- ANCHOR_END: select-filter-sort
-- ANCHOR: update
UPDATE shop_items SET price_cents = price_cents + 100 WHERE name = 'Pen';
-- ANCHOR_END: update
-- ANCHOR: delete
DELETE FROM shop_items WHERE name = 'Sticker';
-- ANCHOR_END: delete
SELECT * FROM shop_items ORDER BY id;
-- ANCHOR: null-checks
SELECT name FROM shop_items WHERE in_stock IS NULL;
SELECT name, COALESCE(in_stock, false) AS in_stock_or_false FROM shop_items ORDER BY name;
-- ANCHOR_END: null-checks
-- ANCHOR: primary-foreign-keys
CREATE TABLE shop_orders (
    id         integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    item_id    integer NOT NULL REFERENCES shop_items (id),
    quantity   integer NOT NULL CHECK (quantity > 0),
    ordered_on date    NOT NULL
);
INSERT INTO shop_orders (item_id, quantity, ordered_on) VALUES
    (1, 2, '2026-09-01'),
    (3, 5, '2026-09-02'),
    (1, 1, '2026-09-03');
-- ANCHOR_END: primary-foreign-keys
-- ANCHOR: join
SELECT shop_orders.id, shop_items.name, shop_orders.quantity
FROM shop_orders
JOIN shop_items ON shop_items.id = shop_orders.item_id
ORDER BY shop_orders.id;
-- ANCHOR_END: join
-- ANCHOR: left-join-count
SELECT shop_items.name, count(shop_orders.id) AS orders
FROM shop_items
LEFT JOIN shop_orders ON shop_orders.item_id = shop_items.id
GROUP BY shop_items.id, shop_items.name
ORDER BY shop_items.name;
-- ANCHOR_END: left-join-count
-- ANCHOR: index
CREATE INDEX shop_orders_item_id_idx ON shop_orders (item_id);
-- ANCHOR_END: index
-- ANCHOR: explain
EXPLAIN (COSTS OFF)
SELECT * FROM shop_orders WHERE item_id = 1;
-- ANCHOR_END: explain
-- ANCHOR: transaction
BEGIN;
UPDATE shop_items SET in_stock = false WHERE name = 'Mug';
INSERT INTO shop_orders (item_id, quantity, ordered_on) VALUES (1, 1, '2026-09-04');
COMMIT;

BEGIN;
DELETE FROM shop_orders;
ROLLBACK;
-- ANCHOR_END: transaction
SELECT count(*) AS orders_kept FROM shop_orders;
