-- Books nobody has ever borrowed.
SELECT books.title
FROM books
LEFT JOIN loans ON loans.book_id = books.id
WHERE loans.id IS NULL
ORDER BY books.title;
