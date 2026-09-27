SELECT tags.name AS tag
FROM book_tags
JOIN books ON books.id = book_tags.book_id
JOIN tags  ON tags.id  = book_tags.tag_id
WHERE books.title = 'Emma'
ORDER BY tags.name;
