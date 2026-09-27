# Keys and relations

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can give every row a unique ID the database manages.
- You can link tables with foreign keys and join them back together.
- You can model one-to-many and many-to-many relationships.

## What & why

Primary keys, unique rules, foreign keys, JOIN, and modelling one-to-many and many-to-many.

Imagine a library app with two members, both called **Alan Turing**. One of them borrows *Dune*.
Which one? The name can't tell you, and neither can the city or the birthday: two people can share
all of those. What the app needs is a number that belongs to **one** row and nobody else, like a
library card number. And it needs a guarantee that the number is never shared, never missing, and
never typed in wrong. In SQL, that number is called a
[**primary key**](../glossary.md#primary-key), and this lesson has Postgres look after it for you.

The second problem appears as soon as you store two kinds of thing. Books have authors. You could
type the author's name into every book's row:

```text
title       | author
------------+--------------
Emma        | Jane Austen
Persuasion  | Jane Austin
```

Look closely: the second row says `Austin`. One typo, and to the database these are now two
different people. A search for Jane Austen's books misses *Persuasion*, and fixing the spelling of
her name means changing every one of her rows, hoping you find them all.

Your phone has already solved this. A text message doesn't store a copy of the sender's phone
number and name; it **points at** a contact. Fix the contact's name once, and every message from
them shows the new name. Databases work the same way: store each author **once**, in an `authors`
table, and let each book **point at** its author by number. This lesson shows you how to make that
pointer (a [**foreign key**](../glossary.md#foreign-key)), how to make Postgres refuse pointers to
nobody, and how to follow the pointers back to get "each book with its author's name" in one answer
(a [**JOIN**](../glossary.md#join)).

## The idea, slowly

### Step 1: this lesson's database

As in the last lessons, this lesson gets a database of its own, named after the lesson:
`keys_and_relations`. From the book's folder (the one with `docker-compose.yml`), make sure
Postgres is running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres keys_and_relations
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres keys_and_relations`
- **What:** creates a new, empty database called `keys_and_relations`.
- **Why:** this lesson makes tables called `authors`, `books` and `tags`. In a database of its own,
  they can't collide with the `books` table from the last lesson.
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  name is the lesson's web address, `keys-and-relations`, with each `-` turned into `_`.
- **Remove it and…** every later command fails with `database "keys_and_relations" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres keys_and_relations
```

`createdb` prints **nothing** when it works. If you run it a second time, you see this:

```text
createdb: error: database creation failed: ERROR:  database "keys_and_relations" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: a primary key, numbered by Postgres

In the last lesson, you typed every book's `id` yourself, and nothing stopped two books from
getting the same one. This time, Postgres hands out the numbers, and refuses duplicates. This is
`code/sql/keys-and-relations/01-primary-keys.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/01-primary-keys.sql}}
```

### Line by line

`SET client_min_messages = warning;`
- **What:** tells Postgres to show only warnings and errors, not small notes.
- **Why:** the `DROP TABLE IF EXISTS` line prints a note for each table that isn't there yet.
  Hiding notes makes every run look the same.
- **How:** the setting lasts for this one connection.
- **Remove it and…** the first run also prints `NOTICE:  table "…" does not exist, skipping` lines.
  Harmless, but different from the book.

`DROP TABLE IF EXISTS reviews, members, book_tags, tags, books, authors;`
- **What:** deletes every table this lesson makes, if it exists, including `reviews` and `members`,
  which you'll meet near the end.
- **Why:** this is the lesson's reset button. Running `01` puts you back at the very start, whatever
  you did afterwards.
- **How:** one `DROP TABLE` can name several tables, separated by commas. `IF EXISTS` skips any that
  aren't there.
- **Remove it and…** a second run fails with `ERROR:  relation "authors" already exists`.

`CREATE TABLE authors (` … `);`
- **What:** a table with one row per author: an `id` and a `name`.
- **Why:** each author is stored **once**, here. Books will point at these rows.
- **How:** one line per column, as in the last lessons. What's new is the extra words after each
  type: they're rules, explained next.
- **Remove it and…** there's nowhere to put authors, and the `INSERT` fails with
  `relation "authors" does not exist`.

`id integer`
- **What:** a column called `id` that holds whole numbers.
- **Why:** a number makes a good ID: short, and quick to compare.
- **How:** `integer` goes up to about 2.1 billion, as you saw in
  [Postgres data types](postgres-data-types.md).
- **Remove it and…** (the whole line) authors have no ID, and books have nothing to point at.

`GENERATED ALWAYS AS IDENTITY`
- **What:** means "**Postgres numbers the rows for you**": 1, 2, 3, and so on.
- **Why:** you never have to work out the next free number, and two programs adding authors at the
  same moment never pick the same one.
- **How:** Postgres keeps a counter for this column. Each `INSERT` that leaves `id` out takes the
  next number from it. `ALWAYS` means only Postgres may choose: an `INSERT` that brings its own `id`
  is refused (the last line of this file shows it).
- **Remove it and…** you must type every `id` yourself, as in the last lesson, and nothing stops
  you from typing one that's already taken, unless you keep the next rule.

`PRIMARY KEY`
- **What:** makes `id` this table's **primary key**: the column that
  says which row is which.
- **Why:** everything else can repeat. Two authors can share a name, in theory; they can never
  share an `id`. Other tables will point at an author by this column.
- **How:** `PRIMARY KEY` means two rules at once: **unique** (no two rows have the same `id`) and
  **not null** (every row has one). A table has at most one primary key.
- **Remove it and…** nothing stops a second row with the same `id`, and a pointer to "author 1"
  could mean two different people.

`name text NOT NULL UNIQUE`
- **What:** the author's name, which must be filled in, and must be different for every author.
- **Why:** a nameless author is useless, and in this small library, two rows called "Jane Austen"
  would always be a mistake.
- **How:** `NOT NULL` refuses a row where `name` is [NULL](../glossary.md#null) (missing). `UNIQUE`
  refuses a row whose `name` is already in the table. (`\d authors` would now show `not null` under
  *Nullable*, the box that was empty in [Tables, rows and psql](tables-rows-and-psql.md).)
- **Remove it and…** (`UNIQUE`) the duplicate insert below succeeds, and there are two Jane
  Austens. Remove `NOT NULL`, and an author with no name is allowed.

`PRIMARY KEY`, `NOT NULL` and `UNIQUE` are all **[constraints](../glossary.md#constraint)**: rules
the table checks on **every** `INSERT` and `UPDATE`, whichever program sends it. Your Rust code, a
colleague's script and you at the `psql` prompt all get the same answer. There's one more kind you'll
use often, `CHECK`, which tests a condition you write (*More examples* shows it). The foreign key,
in Step 3, is a constraint too.

`INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert');`
- **What:** adds two authors, giving only their names.
- **Why:** the `id`s are Postgres's job now.
- **How:** the column list names only `name`, so `id` gets the next number from the counter: 1 for
  Jane Austen, 2 for Frank Herbert.
- **Remove it and…** the table stays empty, and the next lines have nothing to show or collide
  with.

`SELECT * FROM authors;`
- **What:** shows both authors with the `id`s Postgres chose.
- **Why:** to see the numbers you didn't type.
- **How:** as in every lesson so far.
- **Remove it and…** the rows are still there; you don't see them.

`INSERT INTO authors (name) VALUES ('Jane Austen');`
- **What:** tries to add Jane Austen a second time.
- **Why:** to watch `UNIQUE` refuse it.
- **How:** Postgres sees that `'Jane Austen'` is already in the `name` column, and stops the
  `INSERT`. Nothing is added.
- **Remove it and…** you don't see the error; the table is the same either way.

`INSERT INTO authors (id, name) VALUES (7, 'Someone');`
- **What:** tries to choose an `id` by hand.
- **Why:** to watch `GENERATED ALWAYS` refuse it.
- **How:** the column list includes `id`, and a value is given for it. `ALWAYS` means that's not
  allowed.
- **Remove it and…** you don't see the error; nothing changes.

### Run it

Every file in this lesson runs the same way: feed it to `psql` in this lesson's database.

```bash
docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/01-primary-keys.sql
```

```text
{{#include ../../code/sql/keys-and-relations/01-primary-keys.out}}
```

From the top: `SET`, `DROP TABLE` and `CREATE TABLE` are the command tags you know. `INSERT 0 2`
means two rows went in (the last number is the row count). The table shows Postgres numbered them
`1` and `2`.

Then two errors, and **both are the point**. Each error has up to three lines:

- `ERROR:` says what went wrong: the new row breaks the **unique constraint**
  `authors_name_key`. Postgres names every constraint; when you don't choose a name, it builds one
  from the table, the column and the kind of rule (`_key` means unique).
- `DETAIL:` says exactly which value: `Key (name)=(Jane Austen) already exists.`
- `HINT:`, on the second error, says what you'd have to write to do it anyway:
  `OVERRIDING SYSTEM VALUE`. You won't need that in this book; it's for copying in rows from another
  database, whose IDs must be kept.

An error stops only its own statement. `psql` carries on with the next line, and the two good rows
stay.

- ✅ If you see Jane Austen with `id` `1`, Frank Herbert with `2`, and the two `ERROR:` lines,
  you're right.
- ✅ Run it a second time: the output is identical, because the `DROP TABLE` line starts over.
- ❌ If you see `FATAL:  database "keys_and_relations" does not exist`, go back to Step 1.

### Step 3: a foreign key — books that point at their author

Now the books. Each book row stores its author's `id`, not the name. Here is the idea as a picture,
with the rows the next file creates:

```text
               books                               authors
┌────┬────────────┬───────────┐           ┌────┬───────────────┐
│ id │ title      │ author_id │           │ id │ name          │
├────┼────────────┼───────────┤           ├────┼───────────────┤
│  1 │ Emma       │     1 ────┼──┬──────▶ │  1 │ Jane Austen   │
│  2 │ Persuasion │     1 ────┼──┘   ┌──▶ │  2 │ Frank Herbert │
│  3 │ Dune       │     2 ────┼──────┘    └────┴───────────────┘
└────┴────────────┴───────────┘
```

### Line by line

`books` · `│ id │ title │ author_id │`
- **What:** the `books` table, with its own `id`, a `title`, and an `author_id` column.
- **Why:** `author_id` is where each book says who wrote it.
- **How:** it holds a plain number, but not any number: it must be the `id` of a real author.
- **Remove it and…** (`author_id`) nothing links a book to its author.

`│ 1 │ Emma │ 1 ────┼──┬──────▶ │ 1 │ Jane Austen │`
- **What:** *Emma*'s `author_id` is `1`, so it points at the author whose `id` is `1`: Jane Austen.
- **Why:** this arrow is the whole idea: a number in one table that means "that row, over there".
- **How:** the arrow isn't stored anywhere. It's only the number `1`, plus the rule that `1` must
  exist in `authors`.
- **Remove it and…** (the `1`) Postgres wouldn't know who wrote *Emma*.

`│ 2 │ Persuasion │ 1 ────┼──┘`
- **What:** *Persuasion* points at author `1` too.
- **Why:** this is **one-to-many**: **one** author has **many** books. Jane Austen's name is still
  stored once. Fix its spelling in `authors`, and both books show the new spelling.
- **How:** two rows in `books` hold the same `author_id`. That's allowed: `author_id` isn't
  unique. (In `authors`, `id` is.)
- **Remove it and…** you're back to one book per author, which isn't how books work.

`│ 3 │ Dune │ 2 ────┼──────┘` · `┌──▶ │ 2 │ Frank Herbert │`
- **What:** *Dune* points at author `2`, Frank Herbert.
- **Why:** each book points at exactly one author.
- **How:** same as *Emma*, a different number.
- **Remove it and…** Frank Herbert is an author with no books. That's allowed, as you'll see in
  *More examples*.

The rule of thumb for one-to-many: **the pointer goes on the "many" side.** Many books, one
author, so the column is `books.author_id`. (The other way round, an `authors.book_id`, would hold
only one book per author.)

This is `code/sql/keys-and-relations/02-foreign-keys.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/02-foreign-keys.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP TABLE IF EXISTS reviews, book_tags, books;`
- **What:** hides notes, then deletes `books` and the two tables that will point **at** `books`
  (`book_tags` in Step 5, `reviews` in *Your turn*).
- **Why:** so you can run this file again. A table that others point at can't be dropped on its
  own, so they go with it.
- **How:** as in `01`.
- **Remove it and…** a second run fails with `ERROR:  relation "books" already exists`.

`id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,`
- **What:** the same numbered primary key as `authors` has.
- **Why:** books get pointed at later too (by tags), so they need an ID of their own.
- **How:** each table has its own counter: the first book is `1`, like the first author.
- **Remove it and…** nothing can point at a book.

`title text NOT NULL,`
- **What:** the book's title, which must be filled in.
- **Why:** a book with no title is a mistake.
- **How:** `NOT NULL`, as in `authors`. It's not `UNIQUE`: two different books can have the same
  title.
- **Remove it and…** a book with a NULL title is allowed.

`author_id integer NOT NULL REFERENCES authors (id)`
- **What:** makes `author_id` a **foreign key**: its value must be
  the `id` of a row that exists in `authors`.
- **Why:** a book that points at author `99`, when there's no author `99`, is a broken arrow. The
  foreign key makes that impossible, whichever program writes the row.
- **How:** `REFERENCES authors (id)` means "look it up in `authors.id`". Postgres checks it on every
  `INSERT` and `UPDATE` of `books`, and also on every `DELETE` from `authors` (Step 6). The type is
  `integer`, the same as `authors.id`, so the values can match. `NOT NULL` adds "every book has an
  author".
- **Remove it and…** (`REFERENCES authors (id)`) `author_id` is a plain number, and the Ghost book
  below goes in, pointing at nobody.

`INSERT INTO books (title, author_id) VALUES ('Emma', 1), ('Persuasion', 1), ('Dune', 2);`
- **What:** three books, each with the `id` of its author.
- **Why:** these are the rows from the picture.
- **How:** the `1`s and the `2` are the `id`s from Step 2's output. Postgres checks each one exists
  in `authors`.
- **Remove it and…** the table is empty, and the next steps find nothing.

`INSERT INTO books (title, author_id) VALUES ('Ghost book', 99);`
- **What:** tries to add a book by author `99`, who doesn't exist.
- **Why:** to watch the foreign key refuse it.
- **How:** Postgres looks for `99` in `authors.id`, doesn't find it, and stops the `INSERT`.
- **Remove it and…** you don't see the error; the table is the same.

`SELECT * FROM books;`
- **What:** shows the books.
- **Why:** to check that the three good ones are there and the Ghost book isn't.
- **How:** as always.
- **Remove it and…** you don't see the result.

### Run it

```bash
docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/02-foreign-keys.sql
```

```text
{{#include ../../code/sql/keys-and-relations/02-foreign-keys.out}}
```

`INSERT 0 3`: three books went in. Then the foreign key's error: the row breaks the **foreign key
constraint** `books_author_id_fkey` (table, column, and `_fkey` for "foreign key"), and the
`DETAIL:` line says why in plain words: `Key (author_id)=(99) is not present in table "authors".`
The table shows three books, and no Ghost book.

- ✅ If you see three books, with `author_id` `1`, `1` and `2`, and the `foreign key` error, you're
  right.
- ❌ If you see `relation "authors" does not exist`, run `01` first: `books` points at `authors`,
  so `authors` must exist.
- ❌ If you see a second foreign key error, for `(author_id)=(2)`, you ran this file after Step 6,
  which deletes Frank Herbert. Run `01`, then each file in order.

### Step 4: JOIN — follow the pointers back

The books table now holds numbers where names used to be. To show "each title with its author's
name", you need rows from **both** tables side by side. That's a **join**.
This is `code/sql/keys-and-relations/03-join.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/03-join.sql}}
```

Read it aloud, and it's almost English: "Select the book's title and the author's name, **from**
books, **joined with** authors **on** the rule that the author's id equals the book's author_id,
in book order."

### Line by line

`SELECT books.title, authors.name AS author`
- **What:** the two columns of the answer: a title from `books`, a name from `authors`.
- **Why:** this is the list an app would show: titles with author names, no bare numbers.
- **How:** with two tables in play, `books.title` means "the `title` column of `books`": the table
  name, a dot, the column name. `AS author` renames the answer's column, as in
  [Tables, rows and psql](tables-rows-and-psql.md).
- **Remove it and…** (`AS author`) the heading says `name`, which is less clear next to `title`.

`FROM books`
- **What:** start from the `books` table.
- **Why:** you want one answer row per book.
- **How:** as in every `SELECT` so far.
- **Remove it and…** Postgres reaches `JOIN` with no table to join to, and stops with
  `syntax error at or near "JOIN"`.

`JOIN authors ON authors.id = books.author_id`
- **What:** for each book, finds the author row whose `id` equals the book's `author_id`, and puts
  the two rows together, side by side.
- **Why:** it's the pointer from Step 3, followed back. *Emma*'s `author_id` is `1`, so it's paired
  with author `1`, Jane Austen.
- **How:** `JOIN authors` brings in the second table; `ON` gives the rule for which rows belong
  together. Postgres keeps only the pairs where the rule is true. The `=` compares, as in `WHERE`.
- **Remove it and…** (`ON …`) Postgres refuses to guess the rule: it's a syntax error (*Common
  mistakes* shows it).

`ORDER BY books.id;`
- **What:** sorts the answer by the book's `id`.
- **Why:** without `ORDER BY`, the order isn't guaranteed, as you learned in
  [CRUD in SQL](crud-in-sql.md), and a join can hand rows back in an order that matches neither
  table.
- **How:** `books.id`, not `id`: both tables have an `id` column, so you must say which.
- **Remove it and…** (`books.`) Postgres can't tell which `id` you mean
  (*Common mistakes* again).

### Run it

```bash
docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/03-join.sql
```

```text
{{#include ../../code/sql/keys-and-relations/03-join.out}}
```

Three rows, one per book, and a name where each number was. *Emma* and *Persuasion* both found
Jane Austen, and *Dune* found Frank Herbert: Postgres followed each arrow in the picture. This file
only reads, so you can run it as often as you like.

- ✅ If you see three titles next to their authors' names, you're right.
- ❌ If you see `(0 rows)`, the `books` table is empty: run `01` and `02` first.

### Step 5: many-to-many — books and tags

A bookshop tags its books: *classic*, *sci-fi*, *romance*. One book has **many** tags, and one tag
is on **many** books. A foreign key column holds only **one** number, so neither side can hold the
pointers: `books.tag_id` would allow one tag per book, and `tags.book_id` one book per tag.

The answer is a third table whose only job is to hold the **pairs**. Each row says "this book has
this tag":

```text
      books                     book_tags                   tags
┌────┬────────────┐      ┌─────────┬────────┐      ┌────┬─────────┐
│ id │ title      │      │ book_id │ tag_id │      │ id │ name    │
├────┼────────────┤      ├─────────┼────────┤      ├────┼─────────┤
│  1 │ Emma       │ ◀─── │    1    │   1    │ ───▶ │  1 │ classic │
│  2 │ Persuasion │      │    1    │   3    │      │  2 │ sci-fi  │
│  3 │ Dune       │      │    2    │   1    │      │  3 │ romance │
└────┴────────────┘      │    2    │   3    │      └────┴─────────┘
                         │    3    │   1    │
                         │    3    │   2    │
                         └─────────┴────────┘
```

### Line by line

`books` and `tags`
- **What:** the two things being linked. Each has its own `id`.
- **Why:** each tag's name is stored once, like each author's.
- **How:** neither table has a column about the other.
- **Remove it and…** (either) there's nothing to pair.

`book_tags` · `│ book_id │ tag_id │`
- **What:** a **join table**: one row per pairing, and nothing else.
- **Why:** a book with two tags gets two rows here; a tag on three books gets three. Any number of
  pairings fits.
- **How:** both columns are foreign keys: `book_id` points at `books`, `tag_id` at `tags`. The name
  is the two tables' names together, a common habit.
- **Remove it and…** books and tags have no way to find each other.

`│ 1 │ Emma │ ◀─── │ 1 │ 1 │ ───▶ │ 1 │ classic │`
- **What:** the first pair: book `1` (*Emma*) has tag `1` (*classic*).
- **Why:** one row, one fact.
- **How:** follow `book_id` left to the book, and `tag_id` right to the tag.
- **Remove it and…** *Emma* is no longer tagged *classic*; the book and the tag are both still
  there.

`│ 1 │ 3 │` … `│ 3 │ 2 │`
- **What:** the other five pairs: *Emma* is also *romance* (1, 3); *Persuasion* is *classic* and
  *romance* (2, 1 and 2, 3); *Dune* is *classic* and *sci-fi* (3, 1 and 3, 2).
- **Why:** *classic* appears on three books, and *Emma* has two tags: many on both sides.
- **How:** read each row as "book *n* has tag *m*".
- **Remove it and…** (a row) that one book loses that one tag.

This is `code/sql/keys-and-relations/04-many-to-many.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/04-many-to-many.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP TABLE IF EXISTS book_tags, tags;`
- **What:** hides notes, then deletes the two tables this file makes.
- **Why:** so you can run this file again.
- **How:** `book_tags` points at `tags`, so it's named too.
- **Remove it and…** a second run fails with `ERROR:  relation "tags" already exists`.

`CREATE TABLE tags (` … `);`
- **What:** one row per tag: a numbered `id` and a `name` that must be filled in and different
  from every other tag's.
- **Why:** exactly the shape of `authors`, for the same reasons.
- **How:** the same column rules as in Step 2.
- **Remove it and…** `book_tags` has no tags to point at.

`book_id integer NOT NULL REFERENCES books (id) ON DELETE CASCADE,`
- **What:** a foreign key to `books`, which must be filled in.
- **Why:** each pair must be about a real book.
- **How:** as in Step 3. `ON DELETE CASCADE` says what happens when that book is deleted: delete
  this pair too. Step 6 shows it, and explains why it's right here.
- **Remove it and…** (`ON DELETE CASCADE`) deleting a tagged book is refused, as Step 6 shows for
  authors.

`tag_id integer NOT NULL REFERENCES tags (id) ON DELETE CASCADE,`
- **What:** the same for `tags`.
- **Why:** each pair must be about a real tag, and deleting a tag should untag every book.
- **How:** as the line above.
- **Remove it and…** a pair could point at a tag that doesn't exist.

`PRIMARY KEY (book_id, tag_id)`
- **What:** a primary key made of **two** columns together.
- **Why:** a pairing should be stored once. "Emma is classic" twice would be a duplicate.
- **How:** written on its own line, after the columns, because it's about two of them. The **pair**
  must be unique: `(1, 1)` and `(1, 3)` are both allowed (same book, different tags), but a second
  `(1, 1)` is refused. Both columns are also not null.
- **Remove it and…** you could tag *Emma* *classic* twice, and she'd show up twice in every list of
  classics.

`INSERT INTO tags (name) VALUES ('classic'), ('sci-fi'), ('romance');`
- **What:** three tags, numbered `1`, `2`, `3` by Postgres.
- **Why:** the pairs below refer to them by these numbers.
- **How:** the counter numbers them in the order given.
- **Remove it and…** the next `INSERT` fails with a foreign key error: no tag `1`.

`INSERT INTO book_tags (book_id, tag_id) VALUES (1, 1), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2);`
- **What:** the six pairs from the picture.
- **Why:** this is the tagging itself.
- **How:** each `(book, tag)` is one row. Every number is checked against `books` or `tags`.
- **Remove it and…** no book has any tag.

`SELECT books.title, tags.name AS tag` … `FROM book_tags`
- **What:** the answer's columns (a title and a tag name), starting from the pairs.
- **Why:** the pairs are what connects the other two tables, so it's the natural place to start.
- **How:** `FROM book_tags` gives one row per pair; the joins below fill in the names.
- **Remove it and…** there's no table to start from.

`JOIN books ON books.id = book_tags.book_id` · `JOIN tags ON tags.id = book_tags.tag_id`
- **What:** two joins in a row: each pair gets its book, then its tag.
- **Why:** each join follows one arrow of the picture, left then right.
- **How:** you can chain as many `JOIN … ON …` as you need; each one adds a table. (The extra
  spaces after `tags` only line the two up; SQL ignores them.)
- **Remove it and…** (the second join) `tags.name` has no table to come from, and the query fails
  with `missing FROM-clause entry for table "tags"`.

`WHERE tags.name = 'classic'`
- **What:** keeps only the pairs whose tag is *classic*.
- **Why:** the question is "which books are classics?"
- **How:** `WHERE` works on the joined rows, so it can test a column from any of the three tables.
- **Remove it and…** you get all six pairs.

`ORDER BY books.title;`
- **What:** sorts the answer A to Z by title.
- **Why:** a predictable order, always.
- **How:** as in Step 4.
- **Remove it and…** the order is whatever's quickest.

### Run it

```bash
docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/04-many-to-many.sql
```

```text
{{#include ../../code/sql/keys-and-relations/04-many-to-many.out}}
```

`CREATE TABLE` twice, one tag per table. `INSERT 0 3` and `INSERT 0 6`: three tags and six pairs.
Then all three books, because all three are tagged *classic*.

- ✅ If you see *Dune*, *Emma* and *Persuasion*, each next to `classic`, you're right.
- ❌ If you see `relation "books" does not exist`, run `01` and `02` first.

### Step 6: deleting a row that others point at

What should happen if you delete Frank Herbert, while *Dune* still points at him? *Dune* would be
left pointing at an author who isn't there: exactly the broken arrow the foreign key exists to
prevent. This is `code/sql/keys-and-relations/05-on-delete.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/05-on-delete.sql}}
```

### Line by line

`DELETE FROM authors WHERE name = 'Frank Herbert';` (the first time)
- **What:** tries to delete Frank Herbert.
- **Why:** to see what a foreign key does when you delete a row it points at.
- **How:** `books.author_id` has no `ON DELETE` rule, so it follows the **default: refuse**. While
  any book still points at author `2`, author `2` can't be deleted. (Postgres calls the default
  `NO ACTION`; you'll also see `RESTRICT`, which refuses in the same way.)
- **Remove it and…** you don't see the refusal.

`DELETE FROM books WHERE title = 'Dune';`
- **What:** deletes *Dune*.
- **Why:** it's the book that pointed at Frank Herbert, and it has two tags, so it shows
  `ON DELETE CASCADE` at work.
- **How:** the `DELETE` from [CRUD in SQL](crud-in-sql.md). Two rows in `book_tags` point at book
  `3`, and their foreign key says `ON DELETE CASCADE`: when the book goes, they go too, in the same
  statement.
- **Remove it and…** *Dune* still points at Frank Herbert, and the last line is refused again.

`SELECT count(*) AS dune_tag_rows FROM book_tags WHERE book_id = 3;`
- **What:** counts the pairs left for book `3`.
- **Why:** to prove the cascade removed *Dune*'s two tag rows.
- **How:** `count(*)`, from the last lesson, counts the rows `WHERE` kept.
- **Remove it and…** the cascade still happens; you don't see it.

`DELETE FROM authors WHERE name = 'Frank Herbert';` (the second time)
- **What:** the same `DELETE` as the first line.
- **Why:** nothing points at Frank Herbert now, so it's allowed.
- **How:** the foreign key checks `books` again, finds no book with `author_id` `2`, and lets the
  delete through.
- **Remove it and…** Frank Herbert stays, an author with no books.

### Run it

```bash
docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/05-on-delete.sql
```

```text
{{#include ../../code/sql/keys-and-relations/05-on-delete.out}}
```

The first line is refused: deleting author `2` would break the `books_author_id_fkey` constraint,
because `Key (id)=(2) is still referenced from table "books"`. *Referenced* means "pointed at".
Then `DELETE 1`: *Dune* is gone. `dune_tag_rows` is `0`: its two pairs in `book_tags` went with it,
though you never deleted them yourself. And the last `DELETE 1` is Frank Herbert, now allowed.

So a foreign key gives you two choices, and you pick one per foreign key:

- **The default:** deleting a row that's pointed at is **refused**. `books.author_id` uses it: an
  author who still has books stays.
- **`ON DELETE CASCADE`:** the rows pointing at it are **deleted too**. `book_tags` uses it: a pair
  means nothing without its book.

- ✅ If you see the `foreign key` error, then `DELETE 1`, `0`, and `DELETE 1`, you're right.
- ✅ Run it a second time, and you see `DELETE 0` three times and `0`: there's no *Dune* and no Frank
  Herbert left, so there's nothing to refuse or delete.

From here on, the lesson's files expect the tables as they are now: one author, Jane Austen, with
two books, *Emma* and *Persuasion*, each tagged *classic* and *romance*. To get back to this at any
time, run `01` to `05` again, in order.

## You might be wondering…

**"What's `serial`? Tutorials use `id serial PRIMARY KEY`."**
It's the older way of getting an auto-numbered column in Postgres, and you'll see it in lots of
tutorials and older code. It works, but it's Postgres-only, and it quietly accepts an `id` you type
yourself, which can later make the counter hand out a number that's already taken.
`GENERATED ALWAYS AS IDENTITY` is the SQL standard way, available since Postgres 10, and it refuses
a hand-typed `id` (you saw it). Use identity in new tables. (There's also
`GENERATED BY DEFAULT AS IDENTITY`, which numbers rows for you but accepts an `id` if you give one.)

**"Why did the numbers skip? The next book will be `5`, not `4`."**
A failed `INSERT` still uses up a number. The Ghost book took `4` from the counter before the
foreign key refused it, and the number isn't handed back. Gaps are normal and harmless: an `id`
only has to be unique, not tidy. Never use `id`s to count rows; use `count(*)`.

**"Should IDs be integers or UUIDs?"**
Both are fine primary keys. Integers are small, fast and easy to read, which is why this part of the
book uses them. Their weakness shows when they're **public**: if your web address is
`/orders/41`, anyone can try `/orders/42`, and the numbers tell competitors how many orders you
have. A **UUID** (the long random-looking value from
[Postgres data types](postgres-data-types.md)) gives nothing away. The capstone project uses UUIDs
for the IDs it shows the world.

**"Why is `book_tags`' primary key two columns?"**
Because the pair **is** the identity of a `book_tags` row. "*Emma* is *classic*" is one fact, and
it should be stored once. A single column couldn't say that: `book_id` repeats (a book has many
tags), and so does `tag_id`. Only the **combination** is unique. You could add a separate `id`
column instead, but then you'd still need a unique rule on `(book_id, tag_id)` to stop duplicates,
so the two-column key does both jobs at once.

**"Should I always use `ON DELETE CASCADE`?"**
No: pick it on purpose, one foreign key at a time. Cascade on `books.author_id` would mean deleting
an author silently deletes all their books, and every tag pair of those books, in one line. That's
rarely what anyone wants. The default refusal is the safe choice: it makes you decide what happens
to the books first. `CASCADE` fits rows that mean nothing without their parent, like a pair in a
join table, or the lines of a deleted order. (There's also `ON DELETE SET NULL`, which empties the
pointer instead, for a column that allows NULL.)

**"Does a `JOIN` need a foreign key?"**
No. `JOIN … ON` works on any two columns you can compare. The foreign key doesn't make the join
happen; it guarantees the join finds what it's looking for, because no book can point at a missing
author. In practice, you join along foreign keys almost every time.

## Coming from another language?

If you've used an [ORM](../glossary.md#orm), you've described these relations in its own words, and
it wrote this SQL for you:

- **Prisma:** `id Int @id @default(autoincrement())` is an auto-numbered primary key. On `Book`,
  `author Author @relation(fields: [authorId], references: [id])` creates the `author_id` foreign
  key, and `include: { author: true }` loads each book's author. A list field on both sides
  (`tags Tag[]` and `books Book[]`) makes Prisma create a join table for you.
- **Sequelize:** `Author.hasMany(Book)` and `Book.belongsTo(Author)` add the `authorId` foreign
  key; `Book.belongsToMany(Tag, { through: "book_tags" })` is the join table. `include: Author` in
  a `findAll` becomes a `LEFT OUTER JOIN`, or a plain `JOIN` with `required: true`.
- **Django ORM:** every model gets an `id` primary key for free. `models.ForeignKey(Author,
  on_delete=models.PROTECT)` is the foreign key, and `on_delete` is required: `PROTECT` refuses,
  `CASCADE` deletes too, the same choice as Step 6. `models.ManyToManyField(Tag)` creates the join
  table, and `Book.objects.select_related("author")` does the `JOIN`.
- **SQLAlchemy:** `mapped_column(ForeignKey("authors.id", ondelete="CASCADE"))` is the foreign key,
  `relationship()` follows it, and `relationship(secondary=book_tags)` names the join table.
  `select(Book).join(Book.author)` reads like the SQL.
- **JPA/Hibernate:** `@Id @GeneratedValue(strategy = GenerationType.IDENTITY)` is an identity
  primary key; `@ManyToOne @JoinColumn(name = "author_id")` is the foreign key; `@ManyToMany` with
  `@JoinTable(name = "book_tags")` is the join table.
- **GORM:** a field `AuthorID uint` next to `Author Author` makes a "belongs to" relation;
  `Tags []Tag` with the tag `gorm:"many2many:book_tags;"` is the join table. `Joins("Author")` does a
  `JOIN`, while `Preload("Author")` runs a second query instead.

In Part A3, SeaORM describes the same relations in Rust, and writes these `JOIN`s for you. When a
page is slow or a list is missing a row, knowing the SQL underneath is how you find out why.

If you know spreadsheets: a foreign key is like an "author ID" column in a *Books* sheet, and a
`JOIN` is a `VLOOKUP` into the *Authors* sheet for every row at once. The difference: a spreadsheet
lets you type author `99` and shows `#N/A` later; Postgres refuses it the moment you type it.

## Common mistakes

These files read the tables as Step 6 left them: Jane Austen, with *Emma* and *Persuasion*.

**A `JOIN` without `ON`.**

```sql
{{#include ../../code/sql/keys-and-relations/70-join-without-on.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/70-join-without-on.out}}
```

The first line forgets to say how books and authors match. Postgres doesn't guess, not even from
the foreign key: `JOIN` must be followed by `ON`, so it reports a **syntax error** at the `;`, the
point where it was still waiting for `ON`. The `^` under `LINE 1` marks that spot. **Fix:** add the
`ON` rule, as the second line does. It works, and shows `SELECT *`'s answer: every column of both
tables, side by side, with `id` twice (the book's, then the author's).

**A column name that's in both tables.**

```sql
{{#include ../../code/sql/keys-and-relations/71-ambiguous-column.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/71-ambiguous-column.out}}
```

`books` and `authors` both have an `id` column, so `SELECT id` could mean either. Postgres refuses
to pick one: `column reference "id" is ambiguous`. (A *column reference* is a column's name used in
a query.) **Fix:** put the table's name and a dot in front, as the second line does: `books.id`
gives the books' IDs, `1` and `2`. A column that's in only one table (like `title`) doesn't need
it, but writing the table name anyway makes a join easier to read.

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d keys_and_relations < code/sql/keys-and-relations/<file>`.
They read the tables as Step 6 left them, and any row they add, they delete again, so you can run
them in any order, as often as you like.

### How many books per author? (`LEFT JOIN` and `GROUP BY`)

Ursula K. Le Guin has joined the library, but none of her books have arrived yet. Count each
author's books, and make sure she's in the list with `0`.

```sql
{{#include ../../code/sql/keys-and-relations/50-count-per-author.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/50-count-per-author.out}}
```

`LEFT JOIN` keeps **every** author, even one with no matching book; her book columns are filled
with NULL. (A plain `JOIN` would drop her: the next example shows it.) `GROUP BY authors.id,
authors.name` gathers the joined rows into one group per author, and `count(books.id)` counts the
books in each group. Why `authors.id` as well as the name? Here `name` is `UNIQUE`, so the name
alone would work, but in most tables names can repeat: two members, or two customers, can share a
name. Grouping by the name alone would then merge them into one row and add their counts
together. The `id` is always unique, so grouping by it keeps them apart, and the name is there so
you can show it. Make it a habit: group by the `id`, plus whatever you want to show. It counts only the values that aren't NULL, so Ursula's group, whose one row has a NULL
`books.id`, counts `0`. (`count(*)` would count that row and say `1`.) The last line removes her
again.

### `JOIN` and `LEFT JOIN`, side by side

The same question, "each author with their books", asked both ways:

```sql
{{#include ../../code/sql/keys-and-relations/51-join-vs-left-join.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/51-join-vs-left-join.out}}
```

`JOIN` keeps only authors that **match** a book: Jane Austen twice, and no Ursula, `(2 rows)`.
`LEFT JOIN` keeps every row of the **left** table (the one after `FROM`), matched or not: Ursula
appears with an empty `title`, which is NULL, `(3 rows)`. Use `LEFT JOIN` when "has none" is a
real answer you want to see: authors with no books, customers with no orders.

### Short names for tables

Writing `books.` and `authors.` everywhere gets long. You can give each table a short **alias**
for this one query:

```sql
{{#include ../../code/sql/keys-and-relations/52-aliases.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/52-aliases.out}}
```

`books AS b` means "in this query, call `books` `b`", so `b.title` is `books.title`. The answer is
the same as Step 4's, minus *Dune*. (The `AS` is optional: `FROM books b` works too.) Aliases save
typing in long queries; in short ones, the full names are easier to read.

### Which tags does *Emma* have?

Step 5 went from a tag to its books. The same two joins go the other way, from a book to its tags:

```sql
{{#include ../../code/sql/keys-and-relations/53-emma-tags.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/53-emma-tags.out}}
```

Only the `WHERE` changed: it now picks a book's title instead of a tag's name. *Emma* is *classic*
and *romance*. The join table works in both directions; that's what makes it many-to-many.

### A rule you write yourself: `CHECK`

`NOT NULL` and `UNIQUE` are ready-made rules. A
[**check constraint**](../glossary.md#check-constraint) is one you write: a condition every row
must pass. [Postgres data types](postgres-data-types.md) promised this one: a username of at most
30 letters.

```sql
{{#include ../../code/sql/keys-and-relations/54-check.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/54-check.out}}
```

`CHECK (length(username) <= 30)` runs on every `INSERT` and `UPDATE`: `length()` counts the
letters, and if the answer isn't 30 or less, the row is refused. `austen_fan` (10 letters) goes in;
the long name (37 letters) breaks the check constraint `members_username_check`, and `DETAIL:`
shows the whole row that failed. (The `2` in it is the `id` the counter had picked.) Any condition
you could write in a `WHERE` works: `price_cents >= 0`, `stars >= 1 AND stars <= 5`. Put a rule
like this in the table, and it holds for every program that ever writes to it.

## Your turn

These exercises start from the tables as Step 6 left them. If you've been experimenting, run `01`
to `05` again first, in order.

### 🟢 Guided

List every book's title next to its author's name, sorted A to Z by title. Fill in the blanks:

```text
SELECT books.title, authors.name AS author
FROM books
JOIN ____ ON ____ = books.author_id
ORDER BY ____;
```

<details><summary>Solution</summary>

`code/sql/keys-and-relations/90-join-yourself.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/90-join-yourself.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/90-join-yourself.out}}
```

`JOIN authors` brings in the second table, `authors.id = books.author_id` follows each book's
pointer to its author, and `ORDER BY books.title` sorts by title. `ORDER BY title` works too, since
only `books` has a `title` column.

</details>

### 🟡 Tweak

Add a new tag, `favourite`, and put it on *Persuasion*. Then list *Persuasion*'s tags. Hints:
*Persuasion*'s `id` is `2` (Step 3's output shows it). `RETURNING id`, from
[CRUD in SQL](crud-in-sql.md), tells you the new tag's `id`.

<details><summary>Solution</summary>

`code/sql/keys-and-relations/91-favourite-tag.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/91-favourite-tag.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/91-favourite-tag.out}}
```

Two `INSERT`s: one row in `tags`, whose `RETURNING id` answers `4` (then its tag, `INSERT 0 1`),
and one pair in `book_tags`, `(2, 4)`: book 2, tag 4. Tagging never touches `books` or `tags`;
it's one new pair. The `SELECT` starts from the pairs for book `2` and joins each one to its tag's
name. If you run this file twice, both `INSERT`s are refused: `favourite` already exists
(`UNIQUE`), and so does the pair `(2, 4)` (the two-column primary key). The constraints are doing
their job.

</details>

### 🔴 From scratch

Readers want to review books. Create a table `reviews`: a numbered primary key `id`, a `book_id`
that must point at a real book, `stars` from 1 to 5 (a `CHECK` rule), and a `body` of text. Add a
5-star review of *Emma* and a 4-star review of *Persuasion*. Try a 6-star review too, and watch it
be refused. Then list every review next to its book's title.

<details><summary>Solution</summary>

`code/sql/keys-and-relations/92-reviews.sql`:

```sql
{{#include ../../code/sql/keys-and-relations/92-reviews.sql}}
```

```text
{{#include ../../code/sql/keys-and-relations/92-reviews.out}}
```

`reviews` is one-to-many again: one book, many reviews, so the foreign key `book_id` goes on the
"many" side. `CHECK (stars >= 1 AND stars <= 5)` uses `AND` from the last lesson: both halves must
be true. The 6-star review breaks `reviews_stars_check`, and the two good reviews stay. The join
starts `FROM reviews`, because you want one row per review. There's no `ON DELETE` rule, so a
reviewed book can't be deleted until its reviews are; `ON DELETE CASCADE` would be a fair choice
here too. (`body` has no `NOT NULL`: a star rating with no words is allowed.)

</details>

## Quick check

<div class="quiz" data-topic="keys-and-relations"></div>

## Remember this

- A **primary key** identifies each row: unique and never NULL. `GENERATED ALWAYS AS IDENTITY`
  lets Postgres number the rows, and refuses hand-typed IDs.
- **Constraints** (`PRIMARY KEY`, `NOT NULL`, `UNIQUE`, `CHECK`, `REFERENCES`) are rules the table
  enforces on every write, whichever program sends it.
- **One-to-many:** a foreign key column on the "many" side (`books.author_id → authors.id`).
  **Many-to-many:** a join table of pairs, with a two-column primary key.
- `JOIN … ON` puts matching rows side by side; `LEFT JOIN` also keeps left rows with no match.
- A foreign key refuses to delete a row that others point at, unless you choose
  `ON DELETE CASCADE`. Choose it on purpose.

## Go deeper

- [Constraints](https://www.postgresql.org/docs/18/ddl-constraints.html) — Every kind of rule a table can enforce: CHECK, NOT NULL, UNIQUE, primary and foreign keys.
- [Joins between tables](https://www.postgresql.org/docs/18/tutorial-join.html) — The official tutorial on combining rows from two tables.

<!-- next:start -->

**Next:**

- [Indexes](../a1-postgres/indexes.md)

<!-- next:end -->
