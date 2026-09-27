SET client_min_messages = warning;
DROP TABLE IF EXISTS friends;
CREATE TABLE friends (
    name text,
    city text
);
INSERT INTO friends (name, city) VALUES ('Ada', 'London'), ('Linus', 'Helsinki');
SELECT * FROM friends;
