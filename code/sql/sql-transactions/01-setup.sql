SET client_min_messages = warning;
DROP TABLE IF EXISTS accounts;
CREATE TABLE accounts (
    id            integer PRIMARY KEY,
    owner         text NOT NULL,
    balance_cents integer NOT NULL CHECK (balance_cents >= 0)
);
INSERT INTO accounts (id, owner, balance_cents) VALUES (1, 'Ada', 10000), (2, 'Linus', 5000);
SELECT * FROM accounts ORDER BY id;
