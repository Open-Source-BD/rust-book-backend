INSERT INTO authors (name) VALUES ('Ursula K. Le Guin');
SELECT authors.name, count(books.id) AS books
FROM authors
LEFT JOIN books ON books.author_id = authors.id
GROUP BY authors.name
ORDER BY authors.name;
DELETE FROM authors WHERE name = 'Ursula K. Le Guin';
