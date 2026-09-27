BEGIN;
DELETE FROM accounts WHERE balance_cents < 8000;
SELECT * FROM accounts ORDER BY id;
ROLLBACK;
SELECT * FROM accounts ORDER BY id;
