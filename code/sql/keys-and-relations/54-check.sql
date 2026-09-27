SET client_min_messages = warning;
DROP TABLE IF EXISTS members;
CREATE TABLE members (
    id       integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username text NOT NULL UNIQUE CHECK (length(username) <= 30)
);
INSERT INTO members (username) VALUES ('austen_fan');
INSERT INTO members (username) VALUES ('this_username_is_much_too_long_to_fit');
SELECT * FROM members;
