-- Members over the limit: more than 3 loans not returned yet.
SELECT members.name AS member, count(loans.id) AS open_loans
FROM members
JOIN loans ON loans.member_id = members.id
WHERE loans.returned_on IS NULL
GROUP BY members.name
HAVING count(loans.id) > 3
ORDER BY members.name;

-- Test it: Alan borrows three more books, we look again, then undo it all.
BEGIN;
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES
    (1, 2, '2026-09-24', '2026-10-08'),
    (4, 2, '2026-09-24', '2026-10-08'),
    (5, 2, '2026-09-24', '2026-10-08');
SELECT members.name AS member, count(loans.id) AS open_loans
FROM members
JOIN loans ON loans.member_id = members.id
WHERE loans.returned_on IS NULL
GROUP BY members.name
HAVING count(loans.id) > 3
ORDER BY members.name;
ROLLBACK;
