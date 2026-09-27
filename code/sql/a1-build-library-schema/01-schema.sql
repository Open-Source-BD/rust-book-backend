SET client_min_messages = warning;
DROP TABLE IF EXISTS loans, books, members, authors;
CREATE TABLE authors (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
CREATE TABLE books (
    id               integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title            text    NOT NULL,
    author_id        integer NOT NULL REFERENCES authors (id),
    copies_total     integer NOT NULL CHECK (copies_total >= 0),
    copies_available integer NOT NULL CHECK (copies_available BETWEEN 0 AND copies_total)
);
CREATE TABLE members (
    id        integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name      text NOT NULL,
    email     text NOT NULL UNIQUE,
    joined_on date NOT NULL
);
CREATE TABLE loans (
    id          integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    book_id     integer NOT NULL REFERENCES books (id),
    member_id   integer NOT NULL REFERENCES members (id),
    loaned_on   date    NOT NULL,
    due_on      date    NOT NULL CHECK (due_on > loaned_on),
    returned_on date             CHECK (returned_on >= loaned_on)
);
CREATE INDEX loans_book_id_idx   ON loans (book_id);
CREATE INDEX loans_member_id_idx ON loans (member_id);
\dt
