INSERT INTO books (id, title, author, year, copies)
VALUES (5, 'Kindred', 'Octavia E. Butler', 1979, 4)
RETURNING id, title;
DELETE FROM books WHERE id = 5 RETURNING title;
