SET client_min_messages = warning;
DROP TABLE IF EXISTS events;
CREATE TABLE events (
    id          bigint,
    title       text,
    starts_at   timestamptz,
    is_free     boolean,
    price_cents integer
);
INSERT INTO events VALUES (1, 'Rust meetup', '2026-10-15 18:00:00+06', true, 0);
INSERT INTO events VALUES (2, 'SQL workshop', '2026-10-22 10:00:00+06', false, 1500);
SELECT * FROM events;
