SELECT b.title, a.name AS author
FROM books AS b
JOIN authors AS a ON a.id = b.author_id
ORDER BY b.id;
