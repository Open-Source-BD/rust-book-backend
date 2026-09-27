SET client_min_messages = warning;
DROP TABLE IF EXISTS reviews, book_tags, books;
CREATE TABLE books (
    id        integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title     text NOT NULL,
    author_id integer NOT NULL REFERENCES authors (id)
);
INSERT INTO books (title, author_id) VALUES ('Emma', 1), ('Persuasion', 1), ('Dune', 2);
INSERT INTO books (title, author_id) VALUES ('Ghost book', 99);
SELECT * FROM books;
