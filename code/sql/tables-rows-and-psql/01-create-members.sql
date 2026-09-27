SET client_min_messages = warning;
DROP TABLE IF EXISTS members;
DROP TABLE IF EXISTS events;
CREATE TABLE members (
    id     integer,
    name   text,
    email  text,
    joined date
);
\d members
