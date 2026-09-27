-- Borrow "Persuasion" twice: there is only one copy.
BEGIN;
UPDATE books SET copies_available = copies_available - 1 WHERE id = 2;
UPDATE books SET copies_available = copies_available - 1 WHERE id = 2;
COMMIT;

-- A second member with Ada's email.
INSERT INTO members (name, email, joined_on) VALUES ('Ada Again', 'ada@example.com', '2026-09-24');

-- A loan due before it starts.
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (5, 2, '2026-09-24', '2026-09-01');

-- A loan for a book that does not exist.
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (99, 2, '2026-09-24', '2026-10-08');

SELECT title, copies_available FROM books WHERE id = 2;
