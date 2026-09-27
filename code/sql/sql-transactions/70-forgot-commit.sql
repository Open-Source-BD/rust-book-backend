BEGIN;
UPDATE accounts SET balance_cents = balance_cents - 1000 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
