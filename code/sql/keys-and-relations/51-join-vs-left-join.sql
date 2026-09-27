INSERT INTO authors (name) VALUES ('Ursula K. Le Guin');
SELECT authors.name, books.title
FROM authors
JOIN books ON books.author_id = authors.id
ORDER BY authors.name, books.title;
SELECT authors.name, books.title
FROM authors
LEFT JOIN books ON books.author_id = authors.id
ORDER BY authors.name, books.title;
DELETE FROM authors WHERE name = 'Ursula K. Le Guin';
