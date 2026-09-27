SELECT books.title, authors.name AS author
FROM books
JOIN authors ON authors.id = books.author_id
ORDER BY books.title;
