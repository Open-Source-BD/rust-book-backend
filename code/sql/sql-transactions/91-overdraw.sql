SELECT * FROM accounts ORDER BY id;
BEGIN;
UPDATE accounts SET balance_cents = balance_cents + 100000 WHERE id = 2;
UPDATE accounts SET balance_cents = balance_cents - 100000 WHERE id = 1;
COMMIT;
SELECT * FROM accounts ORDER BY id;
