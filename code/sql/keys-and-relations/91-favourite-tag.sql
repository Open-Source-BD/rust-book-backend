INSERT INTO tags (name) VALUES ('favourite') RETURNING id;
INSERT INTO book_tags (book_id, tag_id) VALUES (2, 4);
SELECT tags.name AS tag
FROM book_tags
JOIN tags ON tags.id = book_tags.tag_id
WHERE book_tags.book_id = 2
ORDER BY tags.name;
