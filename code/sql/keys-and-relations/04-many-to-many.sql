SET client_min_messages = warning;
DROP TABLE IF EXISTS book_tags, tags;
CREATE TABLE tags (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
CREATE TABLE book_tags (
    book_id integer NOT NULL REFERENCES books (id) ON DELETE CASCADE,
    tag_id  integer NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);
INSERT INTO tags (name) VALUES ('classic'), ('sci-fi'), ('romance');
INSERT INTO book_tags (book_id, tag_id) VALUES (1, 1), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2);
SELECT books.title, tags.name AS tag
FROM book_tags
JOIN books ON books.id = book_tags.book_id
JOIN tags  ON tags.id  = book_tags.tag_id
WHERE tags.name = 'classic'
ORDER BY books.title;
