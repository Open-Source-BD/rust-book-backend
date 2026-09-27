SET client_min_messages = warning;
DROP TABLE IF EXISTS reviews;
CREATE TABLE reviews (
    id      integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    book_id integer NOT NULL REFERENCES books (id),
    stars   integer NOT NULL CHECK (stars >= 1 AND stars <= 5),
    body    text
);
INSERT INTO reviews (book_id, stars, body) VALUES (1, 5, 'Witty and warm.'), (2, 4, 'Quietly moving.');
INSERT INTO reviews (book_id, stars, body) VALUES (1, 6, 'Best book ever!');
SELECT books.title, reviews.stars, reviews.body
FROM reviews
JOIN books ON books.id = reviews.book_id
ORDER BY books.title;
