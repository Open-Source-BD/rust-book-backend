INSERT INTO books (id, title, author, year, copies) VALUES
    (6, 'The Hobbit', 'J. R. R. Tolkien', 1937, 5);
SELECT id, title, copies FROM books WHERE id = 6;
