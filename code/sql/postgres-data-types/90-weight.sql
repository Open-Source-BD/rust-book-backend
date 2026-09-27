ALTER TABLE products ADD COLUMN weight_grams integer;
INSERT INTO products VALUES (4, 'Notebook', 450, true, '2026-09-20', NULL, 200);
SELECT name, weight_grams FROM products;
