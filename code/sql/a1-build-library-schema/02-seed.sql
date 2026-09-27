INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert'), ('Ursula K. Le Guin');
INSERT INTO books (title, author_id, copies_total, copies_available) VALUES
    ('Emma',                     1, 2, 1),
    ('Persuasion',               1, 1, 1),
    ('Dune',                     2, 3, 1),
    ('A Wizard of Earthsea',     3, 2, 2),
    ('The Left Hand of Darkness', 3, 1, 1);
INSERT INTO members (name, email, joined_on) VALUES
    ('Ada Lovelace', 'ada@example.com',  '2026-01-15'),
    ('Alan Turing',  'alan@example.com', '2026-02-01'),
    ('Grace Hopper', 'grace@example.com', '2026-03-10');
INSERT INTO loans (book_id, member_id, loaned_on, due_on, returned_on) VALUES
    (1, 1, '2026-09-01', '2026-09-15', NULL),
    (3, 1, '2026-09-10', '2026-09-24', NULL),
    (3, 2, '2026-09-20', '2026-10-04', NULL),
    (2, 3, '2026-08-01', '2026-08-15', '2026-08-14'),
    (3, 3, '2026-07-01', '2026-07-15', '2026-07-20');
