-- The catalogue: every book with its author.
SELECT books.title, authors.name AS author, books.copies_available
FROM books
JOIN authors ON authors.id = books.author_id
ORDER BY authors.name, books.title;

-- Who has what right now (not returned yet)?
SELECT members.name AS member, books.title, loans.due_on
FROM loans
JOIN members ON members.id = loans.member_id
JOIN books   ON books.id   = loans.book_id
WHERE loans.returned_on IS NULL
ORDER BY loans.due_on;

-- Overdue on 2026-09-24: not returned and the due date has passed.
SELECT members.name AS member, books.title, loans.due_on
FROM loans
JOIN members ON members.id = loans.member_id
JOIN books   ON books.id   = loans.book_id
WHERE loans.returned_on IS NULL
  AND loans.due_on < DATE '2026-09-24'
ORDER BY loans.due_on;

-- How many times has each book been borrowed?
SELECT books.title, count(loans.id) AS times_borrowed
FROM books
LEFT JOIN loans ON loans.book_id = books.id
GROUP BY books.title
ORDER BY times_borrowed DESC, books.title;
