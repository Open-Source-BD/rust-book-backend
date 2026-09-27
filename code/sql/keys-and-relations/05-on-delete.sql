DELETE FROM authors WHERE name = 'Frank Herbert';
DELETE FROM books WHERE title = 'Dune';
SELECT count(*) AS dune_tag_rows FROM book_tags WHERE book_id = 3;
DELETE FROM authors WHERE name = 'Frank Herbert';
