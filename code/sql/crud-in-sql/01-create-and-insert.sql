SET client_min_messages = warning;
DROP TABLE IF EXISTS books;
CREATE TABLE books (
    id     integer,
    title  text,
    author text,
    year   integer,
    copies integer
);
INSERT INTO books (id, title, author, year, copies) VALUES
    (1, 'Dune', 'Frank Herbert', 1965, 3),
    (2, 'Emma', 'Jane Austen', 1815, 1),
    (3, 'Neuromancer', 'William Gibson', 1984, 2),
    (4, 'Persuasion', 'Jane Austen', 1817, 0);
