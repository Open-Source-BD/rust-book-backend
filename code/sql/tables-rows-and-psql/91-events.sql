SET client_min_messages = warning;
DROP TABLE IF EXISTS events;
CREATE TABLE events (
    id         integer,
    title      text,
    happens_on date
);
INSERT INTO events (id, title, happens_on) VALUES
    (1, 'Book club', '2026-10-01'),
    (2, 'Rust meetup', '2026-10-15');
SELECT title FROM events;
