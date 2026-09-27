BEGIN;
INSERT INTO accounts (id, owner, balance_cents) VALUES (3, 'Grace', 0);
SELECT count(*) AS accounts FROM accounts;
COMMIT;
SELECT * FROM accounts ORDER BY id;
DELETE FROM accounts WHERE id = 3;
