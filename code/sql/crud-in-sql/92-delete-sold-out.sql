UPDATE books SET copies = 0 WHERE id = 3;
DELETE FROM books WHERE copies = 0;
SELECT id, title, copies FROM books ORDER BY id;
