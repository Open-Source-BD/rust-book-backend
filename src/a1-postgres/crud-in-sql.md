# CRUD in SQL

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can create, read, update and delete rows.
- You can filter, sort and limit results.
- You know why an UPDATE or DELETE without WHERE is dangerous.

## What & why

Create, Read, Update, Delete — the four things every backend does to data, in SQL.

Open any shopping app and watch what it does with its data. The product page **reads** a product.
"Add to cart" **creates** a line in your cart. Changing the quantity from 1 to 2 **updates** that
line. The little bin icon **deletes** it. Every screen of every app you've used is made of these
four actions, and nothing else. Programmers call them [**CRUD**](../glossary.md#crud): **C**reate,
**R**ead, **U**pdate, **D**elete.

SQL has one statement for each. You've already used two of them: `INSERT` adds rows, and `SELECT`
reads them. This lesson adds `UPDATE`, which changes rows that are already there, and `DELETE`,
which removes them. It also teaches `SELECT` to be picky: only *these* rows, in *this* order, and
no more than *this* many.

You've seen the same four actions before, from the other side. In
[How a web backend works](../part-0-start/how-a-web-backend-works.md#step-6-rest-a-naming-plan-for-endpoints),
a REST backend had four HTTP methods for products. Each one turns into one SQL statement when the
request reaches the database:

| CRUD | HTTP request | SQL statement | What it does to the table |
|---|---|---|---|
| Create | `POST /products` | `INSERT` | adds a new row |
| Read | `GET /products` | `SELECT` | reads rows, changes nothing |
| Update | `PUT /products/{id}` | `UPDATE` | changes a row that's already there |
| Delete | `DELETE /products/{id}` | `DELETE` | removes a row |

By Part B, your Rust backend will receive a `DELETE /products/42` request and send a `DELETE` to
Postgres. Here, you write that SQL yourself, so you know exactly what your code will be asking for.

## The idea, slowly

### Step 1: this lesson's database

As in the last three lessons, this lesson gets a database of its own, named after the lesson:
`crud_in_sql`. From the book's folder (the one with `docker-compose.yml`), make sure Postgres is
running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres crud_in_sql
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres crud_in_sql`
- **What:** creates a new, empty database called `crud_in_sql`.
- **Why:** this lesson makes a table called `books`, and changes and deletes its rows. In a database
  of its own, nothing you do here can touch another lesson's tables (the first lesson has a `books`
  table too).
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  name is the lesson's web address, `crud-in-sql`, with each `-` turned into `_`.
- **Remove it and…** every later command fails with `database "crud_in_sql" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres crud_in_sql
```

`createdb` prints **nothing** when it works. If you run it a second time, you see this:

```text
createdb: error: database creation failed: ERROR:  database "crud_in_sql" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: Create — a table of books, and four rows

A small bookshop keeps one row per book: its number, title, author, the year it came out, and how
many copies are on the shelf. This is `code/sql/crud-in-sql/01-create-and-insert.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/01-create-and-insert.sql}}
```

### Line by line

`SET client_min_messages = warning;`
- **What:** tells Postgres to show only warnings and errors, not small notes.
- **Why:** the `DROP TABLE IF EXISTS` line prints a note on the very first run only. Hiding notes
  makes every run look the same.
- **How:** the setting lasts for this one connection.
- **Remove it and…** the first run also prints `NOTICE:  table "books" does not exist, skipping`.
  Harmless, but different from the book.

`DROP TABLE IF EXISTS books;`
- **What:** deletes the `books` table, and every row in it, if it exists.
- **Why:** this lesson changes and deletes rows. Running this file again puts the shop back to these
  exact four books, whatever you did to them.
- **How:** `IF EXISTS` means "if there's no such table, skip it quietly".
- **Remove it and…** a second run fails with `ERROR:  relation "books" already exists`.

`CREATE TABLE books (` … `);`
- **What:** creates the `books` table with five columns.
- **Why:** rows need a table to go into.
- **How:** as in the last two lessons: one `name type` pair per line, commas between them, none
  after the last. `id`, `year` and `copies` are whole numbers, so they're `integer`; `title` and
  `author` are `text`.
- **Remove it and…** the `INSERT` fails with `relation "books" does not exist`.

`INSERT INTO books (id, title, author, year, copies) VALUES`
- **What:** the **C** in CRUD: adds new rows to `books`.
- **Why:** it's the SQL behind every "create" in an app: signing up, placing an order, adding a
  book to the shop.
- **How:** the column list says which box each value goes in, in order, as in
  [Tables, rows and psql](tables-rows-and-psql.md). One pair of parentheses per row follows.
- **Remove it and…** the table stays empty, and every query below finds nothing.

`(1, 'Dune', 'Frank Herbert', 1965, 3),` … `(4, 'Persuasion', 'Jane Austen', 1817, 0);`
- **What:** four books. Two are by Jane Austen, and *Persuasion* has `0` copies: it's sold out.
- **Why:** these four rows are picked so each query in this lesson finds something different.
- **How:** numbers without quotes, text in single quotes, a comma after every row but the last, and
  a `;` after the last.
- **Remove it and…** (a row) that book is missing from every answer below.

### Run it

Every file in this lesson runs the same way: feed it to `psql` in this lesson's database.

```bash
docker compose exec -T db psql -U postgres -d crud_in_sql < code/sql/crud-in-sql/01-create-and-insert.sql
```

```text
{{#include ../../code/sql/crud-in-sql/01-create-and-insert.out}}
```

Each line is a **command tag**: `psql` reporting what one statement did. `SET`, `DROP TABLE` and
`CREATE TABLE` you've seen. `INSERT 0 4` means four rows went in: the **last** number is the row
count, and the `0` in the middle is always `0`.

- ✅ If you see `INSERT 0 4`, you're right: the shop has four books.
- ✅ Run it a second time: the output is identical, and the table is back to these four books.
- ❌ If you see `FATAL:  database "crud_in_sql" does not exist`, go back to Step 1 and run
  `createdb`.

### Step 3: Read — pick the rows, the order and how many

`SELECT` is the **R** in CRUD, and it's the statement you'll write most. So far you've read whole
tables. Real apps almost never want the whole table: a search page wants *Austen's* books, sorted,
ten at a time. This file, `code/sql/crud-in-sql/02-select.sql`, asks five questions:

```sql
{{#include ../../code/sql/crud-in-sql/02-select.sql}}
```

### Line by line

`SELECT title, year FROM books;`
- **What:** every book's title and year.
- **Why:** it's the starting point: every row, two columns.
- **How:** as in the last lessons. There's no `ORDER BY` (below), so Postgres returns the rows in
  **whatever order is quickest** for it. Today that's the order you inserted them, but nothing
  promises it: after rows are changed or deleted, the order can change (*More examples* shows it
  happen).
- **Remove it and…** you lose the plain, unfiltered list to compare the next answers with.

`SELECT title FROM books WHERE author = 'Jane Austen';`
- **What:** only the titles of the books by Jane Austen.
- **Why:** filtering is how an app finds *some* rows: one user's orders, one author's books.
- **How:** a [**`WHERE` clause**](../glossary.md#where) (a *clause* is one part of a statement)
  keeps only the rows where its condition is true. Postgres checks every row: is its `author`
  equal to `'Jane Austen'`? For *Emma* and *Persuasion*, yes; for the other two, no. `=` means
  "is equal to" here, not "set to". The text must match exactly, capital letters included.
- **Remove it and…** (the `WHERE …` part) you get all four titles.

`SELECT title, year FROM books WHERE year > 1900 ORDER BY year DESC;`
- **What:** the books from after 1900, newest first.
- **Why:** "newest first" is how feeds, order histories and search results are shown.
- **How:** `>` means "greater than". The others are `<` (less than), `>=` (greater than or equal),
  `<=` (less than or equal) and `<>` (not equal; `!=` works too). `ORDER BY year` sorts the answer
  by `year`, smallest first; `DESC` (short for *descending*) turns it around: biggest first.
  `ORDER BY` always comes **after** `WHERE`: first pick the rows, then sort them.
- **Remove it and…** (`DESC`) the order is oldest first: *Dune*, then *Neuromancer*. Remove the
  whole `ORDER BY year DESC`, and the order is whatever's quickest again.

`SELECT title FROM books ORDER BY title LIMIT 2;`
- **What:** the first two titles in alphabetical order.
- **Why:** apps show results a page at a time: the first 10 search results, the 20 latest posts.
  `LIMIT` is how you ask for "no more than this many".
- **How:** `ORDER BY title` sorts text A to Z (smallest first is the normal direction, so there's
  no `DESC`). Then `LIMIT 2` keeps the first two rows of the sorted answer and drops the rest.
  `LIMIT` goes last.
- **Remove it and…** (`ORDER BY title`) you still get two titles, but *which* two is up to
  Postgres. `LIMIT` without `ORDER BY` means "any two".

`SELECT count(*) AS austen_books FROM books WHERE author = 'Jane Austen';`
- **What:** **how many** books are by Jane Austen, instead of which ones.
- **Why:** apps need counts all the time: "3 items in your cart", "page 1 of 5".
- **How:** `count(*)` is a function that counts the rows the `WHERE` kept, and gives back one
  number. The `*` here means "count whole rows". `AS austen_books` names the answer's column.
- **Remove it and…** (`AS austen_books`) the heading is `count` instead.

### Run it

```bash
docker compose exec -T db psql -U postgres -d crud_in_sql < code/sql/crud-in-sql/02-select.sql
```

```text
{{#include ../../code/sql/crud-in-sql/02-select.out}}
```

One table per `SELECT`, from the top:

- All four books, in the order they were inserted this time, `(4 rows)`.
- `WHERE author = 'Jane Austen'`: *Emma* and *Persuasion*, `(2 rows)`.
- After 1900, newest first: *Neuromancer* (1984) above *Dune* (1965). *Emma* and *Persuasion* are
  from the 1800s, so `WHERE` dropped them before sorting.
- A to Z, first two: *Dune* and *Emma*. *Neuromancer* and *Persuasion* came third and fourth, so
  `LIMIT 2` dropped them.
- `austen_books` is `2`: one row with one number, `(1 row)`.

Reading never changes anything, so you can run this file as often as you like.

- ✅ If you see those five answers, you're right.
- ❌ If an answer has `(0 rows)`, look for a typo in the text between the quotes: `'Jane austen'`
  is not `'Jane Austen'` (*Common mistakes* shows it).

### Step 4: Update — change a row that's already there

A new copy of *Persuasion* arrives. The book is already in the table, so you don't add a row; you
change the one that's there. This is `code/sql/crud-in-sql/03-update.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/03-update.sql}}
```

### Line by line

`UPDATE books`
- **What:** the **U** in CRUD: changes rows in the `books` table.
- **Why:** it's the SQL behind every "edit" or "save changes" in an app.
- **How:** `UPDATE` names the table; the rest of the statement says what to change, and where.
- **Remove it and…** there's no statement; nothing changes.

`SET copies = copies + 1`
- **What:** makes `copies` one more than it was.
- **Why:** "one more arrived" is a change to the **current** number, whatever it is.
- **How:** `SET column = value` gives a column a new value. Here the value is worked out from the
  **old** one: `copies + 1` reads the current `copies` (`0`), adds 1, and stores `1`. In `SET`, the
  `=` means "becomes", not "is equal to".
- **Remove it and…** (the `+ 1`, leaving `SET copies = copies`) the statement runs, but every value
  stays the same.

`WHERE id = 4`
- **What:** only the row whose `id` is `4`, *Persuasion*.
- **Why:** this is the most important part of the statement. It picks **which** rows change.
- **How:** exactly the `WHERE` from Step 3: Postgres checks every row, and changes only those where
  the condition is true. An `id` is a good way to pick one row, because each book has a different
  one.
- **Remove it and…** **every** book gets one more copy. *Common mistakes* shows this happen, safely.

`SELECT id, title, copies FROM books WHERE id = 4;`
- **What:** reads the row back.
- **Why:** to see the change for yourself.
- **How:** a `SELECT` with the same `WHERE` as the `UPDATE`.
- **Remove it and…** the change still happens; you don't see it.

### Run it

```bash
docker compose exec -T db psql -U postgres -d crud_in_sql < code/sql/crud-in-sql/03-update.sql
```

```text
{{#include ../../code/sql/crud-in-sql/03-update.out}}
```

`UPDATE 1` is the command tag for an update: the number is how many rows changed. **One**, as
planned. Then the `SELECT` shows *Persuasion* with `1` copy instead of `0`.

- ✅ If you see `UPDATE 1` and `Persuasion` with `1` copy, you're right.
- ❌ If you see `2` copies, you ran the file twice: each run adds one. Run `01` again to start
  clean, then `03` once.
- ❌ If you see `UPDATE 0`, no row had `id` `4`: run `01` first.

### Step 5: Delete — remove a row

The shop stops selling *Emma*. This is `code/sql/crud-in-sql/04-delete.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/04-delete.sql}}
```

### Line by line

`DELETE FROM books`
- **What:** the **D** in CRUD: removes rows from the `books` table.
- **Why:** it's the SQL behind every bin icon, "remove from cart" and "close my account".
- **How:** there's no column list. `DELETE` always removes **whole rows**, never single values. (To
  empty one box, you'd `UPDATE` it to `NULL` instead.)
- **Remove it and…** there's no statement; nothing is deleted.

`WHERE id = 2`
- **What:** only the row whose `id` is `2`, *Emma*.
- **Why:** as with `UPDATE`, this picks which rows go.
- **How:** the same `WHERE` again: Postgres removes every row where the condition is true, and
  only those. The table itself stays; only the row is gone, and it's gone for good: there's no bin
  to restore it from (see *You might be wondering…*).
- **Remove it and…** `DELETE FROM books;` removes **every** row, and the table is left empty.

`SELECT id, title FROM books ORDER BY id;`
- **What:** the books that are left, in `id` order.
- **Why:** to check exactly one row went, and it was the right one.
- **How:** `ORDER BY id` sorts by `id`, smallest first, so the answer comes back in the same order
  every time.
- **Remove it and…** the delete still happens; you don't see what's left.

### Run it

```bash
docker compose exec -T db psql -U postgres -d crud_in_sql < code/sql/crud-in-sql/04-delete.sql
```

```text
{{#include ../../code/sql/crud-in-sql/04-delete.out}}
```

`DELETE 1` is the command tag for a delete: one row removed. The `SELECT` shows three books, and
`id` `2` is missing: *Emma* is gone.

- ✅ If you see `DELETE 1` and three books (1, 3 and 4), you're right. You've done all four letters
  of CRUD.
- ✅ Run it a second time, and the tag is `DELETE 0`: there's no *Emma* left to delete. That isn't
  an error; zero rows matched the `WHERE`.

From here on, the lesson's files expect the table as it is now: *Dune*, *Neuromancer* and
*Persuasion*, with 3, 2 and 1 copies. To get back to it at any time, run `01`, `03` and `04` again,
in that order.

## You might be wondering…

**"Why does `DELETE` print `DELETE 1`?"**
It's the command tag: `psql` telling you what the statement did, and how many rows it did it to.
`DELETE 1` means one row was deleted; `UPDATE 3` means three rows changed. If the `WHERE` matched
nothing, you get `DELETE 0` or `UPDATE 0`, and that's not an error. Read the number every time: if
you meant to change one row and it says `UPDATE 3`, something's wrong with your `WHERE`. (Your
Rust code gets the same number back, as "rows affected".)

**"Can I undo a `DELETE`?"**
Not once it's done. There's no bin and no Ctrl+Z: a deleted row is gone, and so is an `UPDATE`'s
old value. The one way to undo is to start a [**transaction**](../glossary.md#transaction) first,
with `BEGIN`: until you say [`COMMIT`](../glossary.md#commit), you can take everything back with
[`ROLLBACK`](../glossary.md#rollback). *Common mistakes* uses it as an undo button,
and the lesson [SQL transactions](sql-transactions.md), a few lessons from now, teaches it properly. Because there's no undo, careful
people run a `SELECT` with the same `WHERE` first: it shows exactly which rows the `DELETE` would
remove, before anything is removed.

**"Is there a way to get the new row back after `INSERT`?"**
Yes: add `RETURNING` and the columns you want to the end of the `INSERT`, and it answers like a
`SELECT`. It saves asking again. It's how a backend answers `201 Created` with the new product in
the body. *More examples* shows it.

**"Why not `SELECT *` everywhere?"**
`SELECT *` is fine at the `psql` prompt, when you're looking around. In an app's code, name the
columns. You get only what you need: a product list doesn't need every product's long description,
and sending it anyway is slower. And the answer's shape stays the same: if someone adds a `password`
column to `users` next year, `SELECT *` starts sending it wherever it's used, and code that expected
five columns suddenly gets six. Naming the columns says, in the query, exactly what the code relies
on.

## Coming from another language?

If you've used an [ORM](../glossary.md#orm) or query builder, you've written CRUD already. It wrote this SQL for you:

- **Prisma:** `prisma.book.create({ data })` is `INSERT`;
  `findMany({ where: { author: "Jane Austen" }, orderBy: { year: "desc" }, take: 2 })` is
  `SELECT … WHERE … ORDER BY year DESC LIMIT 2`; `update({ where: { id: 4 }, data })` is `UPDATE`;
  `delete({ where: { id: 2 } })` is `DELETE`. Prisma's `updateMany({ data })` with no `where` is
  the "forgot the `WHERE`" mistake below.
- **Sequelize:** `Book.create`, `Book.findAll({ where, order: [["year", "DESC"]], limit: 2 })`,
  `Book.update(values, { where })` and `Book.destroy({ where })`.
- **Django ORM:** `Book.objects.create(…)`, `Book.objects.filter(author="Jane Austen")`,
  `.order_by("-year")[:2]` (the `-` is `DESC`, the slice is `LIMIT`), `.filter(id=4).update(…)` and
  `.filter(id=2).delete()`. `Book.objects.all().delete()` is `DELETE FROM books;`.
- **SQLAlchemy:** `select(Book).where(Book.author == "Jane Austen").order_by(Book.year.desc()).limit(2)`
  reads almost like the SQL; `update(Book).where(…).values(…)` and `delete(Book).where(…)` too.
- **JPA/Hibernate** (with Spring Data): `repository.save(book)` inserts or updates, `findAll(Sort.by("year").descending())`
  reads, and `deleteById(2)` deletes.
- **GORM:** `db.Create(&book)`, `db.Where("author = ?", "Jane Austen").Order("year desc").Limit(2).Find(&books)`,
  `db.Model(&book).Update("copies", 1)` and `db.Delete(&Book{}, 2)`. GORM refuses an update or
  delete with no conditions unless you tell it you mean it, because of the mistake below.

In Part A3, SeaORM gives you the same four operations in Rust. Knowing the SQL underneath means you
can read what it sends, and spot a missing `WHERE` before your users do.

If you know spreadsheets: `WHERE` is the filter button, `ORDER BY` is "sort A→Z" or "Z→A", `LIMIT`
is looking at the top few rows, `UPDATE` is typing over a cell, and `DELETE` is deleting a row. The
big difference: a spreadsheet has Undo, and SQL, outside a transaction, doesn't.

## Common mistakes

These files read the table as Step 5 left it: *Dune*, *Neuromancer* and *Persuasion*.

**An `UPDATE` without `WHERE`.**

```sql
{{#include ../../code/sql/crud-in-sql/70-forgot-where.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/70-forgot-where.out}}
```

`UPDATE books SET copies = 0;` has no `WHERE`, so it applies to **every** row. There's no error and
no "are you sure?": `UPDATE 3` says three rows changed, and the first `SELECT` shows the whole shop
sold out. A `DELETE` without `WHERE` is the same, and worse: `DELETE FROM books;` empties the table.

This file is safe to run, because of its first and fourth lines. `BEGIN` starts a transaction and
`ROLLBACK` throws away every change made since `BEGIN` (each prints its own name as its command
tag). Think of them as an undo button; you'll learn them properly in [SQL transactions](sql-transactions.md). The second
`SELECT`, after `ROLLBACK`, shows every book back to 3, 2 and 1 copies. Without that button, the old
numbers would be gone.

**Fix:** write the `WHERE` **first**, before the `SET`, whenever you type an `UPDATE` or `DELETE`,
and check the tag's row count afterwards: `UPDATE books SET copies = 0 WHERE id = 3;` prints
`UPDATE 1`.

**Text that differs only in capital letters.**

```sql
{{#include ../../code/sql/crud-in-sql/71-case-sensitive.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/71-case-sensitive.out}}
```

The first query finds nothing, `(0 rows)`, although *Persuasion*'s author is Jane Austen. To
Postgres, `'jane austen'` and `'Jane Austen'` are different text: `j` and `J` are different
letters, and `=` compares text exactly. Again there's no error, which makes this mistake easy to
miss. **Fix:** when capitals shouldn't matter (a search box, say), use `ILIKE` instead of `=`, as
in the second query. `ILIKE` compares text **ignoring** capital letters, and finds *Persuasion*.
(It's the case-blind version of `LIKE`, which *More examples* shows.)

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d crud_in_sql < code/sql/crud-in-sql/<file>`. They
read the table as Step 5 left it, and each one leaves it that way again, so you can run them in any
order, as often as you like.

### Get the new row back with `RETURNING`

Add `RETURNING` and a list of columns to an `INSERT`, and it answers with the new row, like a
`SELECT`. It works on `DELETE` too.

```sql
{{#include ../../code/sql/crud-in-sql/50-insert-returning.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/50-insert-returning.out}}
```

The `INSERT` adds *Kindred* and answers with its `id` and `title`, then its tag, `INSERT 0 1`. The
`DELETE … RETURNING title` removes it again, and answers with the title of the row it deleted, so
you can see exactly what went. That also leaves the shop as it was. In Part B, `RETURNING` is how
your backend gets the new product to send back with `201 Created`.

### Two conditions with `AND`

`AND` joins two conditions: a row is kept only if **both** are true.

```sql
{{#include ../../code/sql/crud-in-sql/51-where-and.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/51-where-and.out}}
```

Only *Dune* is from after 1900 **and** has at least 3 copies. *Neuromancer* passes the first test
(1984) but fails the second (2 copies); *Persuasion* fails the first. Its twin is `OR`: a row is
kept if **either** condition is true. (The statement spans three lines to be easy to read; it ends
at the `;`.)

### Match part of the text with `LIKE`

`LIKE` compares text with a pattern. In the pattern, `%` stands for "any letters, or none".

```sql
{{#include ../../code/sql/crud-in-sql/52-like.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/52-like.out}}
```

`'Per%'` means "starts with `Per`": *Persuasion*. `'%er%'` means "has `er` somewhere": *Neuromancer*
(at the end) and *Persuasion* (near the start). Like `=`, `LIKE` cares about capital letters;
`ILIKE`, from *Common mistakes*, doesn't.

### Change two columns at once

`SET` takes several `column = value` pairs, separated by commas.

```sql
{{#include ../../code/sql/crud-in-sql/53-update-two-columns.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/53-update-two-columns.out}}
```

The shop now sells *Dune* in paperback, and has 10: one `UPDATE` changes both the title and the
copies, and the tag says `UPDATE 1`, one row. Now look at the order of the `SELECT`, which has no
`ORDER BY`: *Dune* has moved from the top to the **bottom**. When Postgres updates a row, it writes
the new version as a fresh row, and that changed where it lives in the table. This is why you can't
rely on the order without `ORDER BY`. The last line puts *Dune* back as it was (its title and 3
copies), so the rest of the lesson sees the same shop.

## Your turn

These exercises start from the table as Step 5 left it. If you've been experimenting, run `01`,
`03` and `04` again first.

### 🟢 Guided

Show the title and year of every book that came out **after 1950**, oldest first. Start from the
third query of Step 3, and fill in the blanks:

```text
SELECT title, year FROM books WHERE year > ____ ORDER BY ____;
```

<details><summary>Solution</summary>

`code/sql/crud-in-sql/90-after-1950.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/90-after-1950.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/90-after-1950.out}}
```

`WHERE year > 1950` keeps *Dune* (1965) and *Neuromancer* (1984), and drops *Persuasion* (1817).
`ORDER BY year` sorts them oldest first: no `DESC`, because smallest first is the normal direction.

</details>

### 🟡 Tweak

A delivery arrives: 5 copies of *The Hobbit*, by J. R. R. Tolkien, from 1937. Add it to the shop
with `id` `6`, then read back its `id`, title and copies.

<details><summary>Solution</summary>

`code/sql/crud-in-sql/91-add-book.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/91-add-book.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/91-add-book.out}}
```

The `INSERT` from Step 2, with one row: `INSERT 0 1`. The `SELECT` uses `WHERE id = 6` to read
back only the new book. (You could also have used `RETURNING id, title, copies`, from *More
examples*, and skipped the `SELECT`.) If you run this file twice, you get two Hobbits, both with
`id` `6`: nothing stops a repeated id yet. That's what [Keys and relations](keys-and-relations.md), the next lesson, fixes.

</details>

### 🔴 From scratch

*Neuromancer* has sold its last copy. Record that its copies are now `0`. Then clear every sold-out
book (every book with `0` copies) out of the table, and show what's left, in `id` order.

<details><summary>Solution</summary>

`code/sql/crud-in-sql/92-delete-sold-out.sql`:

```sql
{{#include ../../code/sql/crud-in-sql/92-delete-sold-out.sql}}
```

```text
{{#include ../../code/sql/crud-in-sql/92-delete-sold-out.out}}
```

The `UPDATE` changes one row, *Neuromancer* (`UPDATE 1`). The `DELETE` picks rows by their copies,
not their `id`: `WHERE copies = 0` removes every sold-out book at once, and here that's one
(`DELETE 1`). *Persuasion* stays: it had 0 copies in Step 2, but Step 4 gave it one. The `SELECT`
shows what's left: *Dune*, *Persuasion* and *The Hobbit* (if you did the 🟡 exercise). Before
running a `DELETE` like this on real data, run `SELECT title FROM books WHERE copies = 0;` first:
same `WHERE`, and it shows exactly what would go.

</details>

## Quick check

<div class="quiz" data-topic="crud-in-sql"></div>

## Remember this

- CRUD is Create, Read, Update, Delete: `INSERT`, `SELECT`, `UPDATE`, `DELETE`. They match HTTP's
  `POST`, `GET`, `PUT` and `DELETE`.
- `WHERE` picks the rows; `ORDER BY … DESC` sorts them, biggest first; `LIMIT` keeps the first few.
- Without `ORDER BY`, the order of rows is **not guaranteed**. When order matters, say it.
- An `UPDATE` or `DELETE` without `WHERE` changes **every** row, with no warning and no undo. Write
  the `WHERE` first, and read the row count in the tag.
- `=` compares text exactly, capitals included; `ILIKE` ignores capitals.

## Go deeper

- [SELECT](https://www.postgresql.org/docs/18/sql-select.html) — Every clause SELECT accepts, including WHERE, ORDER BY and LIMIT.
- [UPDATE](https://www.postgresql.org/docs/18/sql-update.html) — The full reference for changing rows, including RETURNING.
- [DELETE](https://www.postgresql.org/docs/18/sql-delete.html) — The full reference for removing rows.

<!-- next:start -->

**Next:**

- [Keys and relations](../a1-postgres/keys-and-relations.md)

<!-- next:end -->
