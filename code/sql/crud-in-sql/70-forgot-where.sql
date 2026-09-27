BEGIN;
UPDATE books SET copies = 0;
SELECT title, copies FROM books ORDER BY id;
ROLLBACK;
SELECT title, copies FROM books ORDER BY id;
