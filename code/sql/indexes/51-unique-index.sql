SET client_min_messages = warning;
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id    integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email text NOT NULL
);
CREATE UNIQUE INDEX users_email_idx ON users (email);
INSERT INTO users (email) VALUES ('ada@example.com'), ('alan@example.com');
INSERT INTO users (email) VALUES ('ada@example.com');
\di users*
DROP TABLE users;
