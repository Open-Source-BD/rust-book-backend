SET client_min_messages = warning;
DROP TABLE IF EXISTS reviews, members, book_tags, tags, books, authors;
CREATE TABLE authors (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert');
SELECT * FROM authors;
INSERT INTO authors (name) VALUES ('Jane Austen');
INSERT INTO authors (id, name) VALUES (7, 'Someone');
