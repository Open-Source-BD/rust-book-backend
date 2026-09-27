SET client_min_messages = warning;
DROP TABLE IF EXISTS books;
CREATE TABLE books (
    title text,
    author text
);
INSERT INTO books (title, author) VALUES ('The Hobbit', 'J. R. R. Tolkien'), ('Matilda', 'Roald Dahl');
SELECT * FROM books;
