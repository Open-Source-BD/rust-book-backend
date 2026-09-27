BEGIN;
UPDATE accounts SET balance_cents = 0 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
ROLLBACK;
SELECT * FROM accounts ORDER BY id;
