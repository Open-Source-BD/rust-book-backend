BEGIN;
UPDATE accounts SET balance_cents = balance_cents - 2500 WHERE id = 1;
UPDATE accounts SET balance_cents = balance_cents + 2500 WHERE id = 2;
COMMIT;
SELECT * FROM accounts ORDER BY id;
