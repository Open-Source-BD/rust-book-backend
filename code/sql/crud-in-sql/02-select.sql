SELECT title, year FROM books;
SELECT title FROM books WHERE author = 'Jane Austen';
SELECT title, year FROM books WHERE year > 1900 ORDER BY year DESC;
SELECT title FROM books ORDER BY title LIMIT 2;
SELECT count(*) AS austen_books FROM books WHERE author = 'Jane Austen';
