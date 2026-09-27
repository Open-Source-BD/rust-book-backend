SELECT name, in_stock, COALESCE(in_stock, false) AS in_stock_or_false FROM products;
