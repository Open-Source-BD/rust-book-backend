-- Grace borrows "A Wizard of Earthsea" (book 4): both changes or neither.
BEGIN;
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (4, 3, '2026-09-24', '2026-10-08');
UPDATE books SET copies_available = copies_available - 1 WHERE id = 4;
COMMIT;

-- Ada returns "Emma" (loan 1): both changes or neither.
BEGIN;
UPDATE loans SET returned_on = '2026-09-24' WHERE id = 1;
UPDATE books SET copies_available = copies_available + 1 WHERE id = 1;
COMMIT;

SELECT id, title, copies_available FROM books WHERE id IN (1, 4) ORDER BY id;
