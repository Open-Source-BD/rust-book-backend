UPDATE accounts SET balance_cents = balance_cents + 20000 WHERE id = 2;
UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
UPDATE accounts SET balance_cents = 5000 WHERE id = 2;
