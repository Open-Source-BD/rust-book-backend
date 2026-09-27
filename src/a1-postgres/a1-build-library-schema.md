# Build it: a library database

> **Beginner** · Part A1 · PostgreSQL & SQL

## What you'll build

You've learned Part A1 one idea at a time: tables, types, keys, joins, indexes, transactions. Now
you'll put all of them together and build something real: the database behind a small lending
library. It has four [tables](../glossary.md#table): **authors**, **books**, **members**, and the
**loans** that connect a member to a book they borrowed.

By the end of this page:

- You designed a real schema of four related tables.
- You wrote the queries a library app needs.
- You changed data safely with transactions.

A **schema** is the design of a database: which tables it has, which columns each one holds, and
the rules that connect them. You'll write the schema, fill it with a few books and members, and then
ask it the four questions every library asks. This is the output of your finished queries, run for
real against the book's Postgres:

```text
{{#include ../../code/sql/a1-build-library-schema/03-queries.out}}
```

Four answers, top to bottom: the **catalogue** (every book, its author, and how many copies are on
the shelf), **who has what** right now, what's **overdue**, and how **popular** each book is. A real
library app runs these same four [queries](../glossary.md#query) behind its pages: the catalogue
page, a member's "my loans" page, the reminder emails for late books, and a "most borrowed" list.
Later in this book, your Rust code sends queries like these for you. Here, you write them yourself.

This project is **fully guided**. The spec tells you what to build, each step gives you a goal, a
hint you can open if you're stuck, and a ✅ checkpoint with the exact output you should see. A full
reference solution, explained line by line, comes after the steps. Try each step yourself first: the
feeling of "I wrote that, and it works" is the whole point.

## What you need to know

Everything here comes from Part A1. If a step feels unfamiliar, the lesson in brackets is where it
was taught:

- [What is a database?](what-is-a-database.md): running a `.sql` file with
  `docker compose exec -T db psql …`, and making a database with `createdb`.
- [Tables, rows and psql](tables-rows-and-psql.md): `CREATE TABLE`, `INSERT`, and `\dt` to list
  your tables.
- [Postgres data types](postgres-data-types.md): `integer`, `text`, `date`, date literals like
  `DATE '2026-09-24'`, and `IS NULL` for a missing value.
- [CRUD in SQL](crud-in-sql.md): `SELECT … WHERE … ORDER BY`, `UPDATE … SET … WHERE`, and `DESC`.
- [Keys and relations](keys-and-relations.md): `PRIMARY KEY`, `GENERATED ALWAYS AS IDENTITY`,
  `UNIQUE`, foreign keys with `REFERENCES`, `CHECK`, `JOIN`, and `LEFT JOIN` with `GROUP BY` and
  `count()` (in its *More examples*).
- [Indexes](indexes.md): `CREATE INDEX`, and why foreign key columns need one of their own.
- [SQL transactions](sql-transactions.md): `BEGIN` … `COMMIT`, and what happens when a statement
  fails inside a transaction.

Three small things are new, and each is explained where it appears: `BETWEEN` (Step 2), `IN (…)`
(Step 5), and, in a stretch goal, `HAVING`.

## The spec

A **spec** (short for *specification*) is a description of what to build, written before anyone
builds it. Real projects start with one. This is yours.

### What the library needs to keep track of

- **Authors.** Each author has a name, and no two authors have the same name.
- **Books.** Each book has a title and one author. The library may own several copies of a book, so
  it records how many copies it owns in total (`copies_total`) and how many are on the shelf right
  now (`copies_available`).
- **Members.** Each member has a name, an email address and the date they joined. Two members may
  share a name, but never an email: the email is how the library tells them apart.
- **Loans.** A loan records that one member borrowed one book: the day they took it (`loaned_on`),
  the day it must come back (`due_on`), and the day it really came back (`returned_on`). A book
  that hasn't come back yet has no `returned_on`: it's [NULL](../glossary.md#null).

Every table gets an `id` [primary key](../glossary.md#primary-key) that Postgres numbers for you.

### The rules

The database itself must refuse data that breaks these rules, whatever program sends it:

1. `copies_available` is never below 0, and never more than `copies_total`.
2. `copies_total` is never below 0.
3. A loan's `due_on` is **after** its `loaned_on`.
4. A loan's `returned_on`, when there is one, is **not before** its `loaned_on`.
5. A loan must point at a book and a member that really exist.
6. No two members have the same email, and no two authors have the same name.
7. No column may be empty (NULL), except `returned_on`.

### The four questions

The library's app must be able to ask:

1. **The catalogue:** every book, with its author's name and how many copies are available. Sorted
   by author, then title.
2. **Who has what:** every loan that hasn't been returned: the member's name, the book's title, and
   the due date. Soonest due first.
3. **What's overdue** on 2026-09-24: loans not returned whose due date is **before** that day.
4. **How popular is each book:** how many times each book has been borrowed, **including** books
   borrowed zero times. Most borrowed first.

### Borrowing and returning

- **Borrowing** a book means two changes: add a loan, and take one from the book's
  `copies_available`.
- **Returning** a book means two changes: set the loan's `returned_on`, and add one back to
  `copies_available`.

Each pair must be **all-or-nothing**: a loan with no copy taken off the shelf, or a copy back on the
shelf with the loan still open, would make the numbers lie.

### The tables

This is the design, drawn as boxes. **PK** marks a primary key, **FK** a
[foreign key](../glossary.md#foreign-key), and each arrow goes from a foreign key to the row it
points at:

```text
 authors               books                      loans                      members
┌──────────────┐      ┌───────────────────┐      ┌───────────────────┐      ┌───────────────┐
│ id        PK │◀──┐  │ id             PK │◀──┐  │ id             PK │  ┌──▶│ id         PK │
│ name  UNIQUE │   └──│ author_id      FK │   └──│ book_id        FK │  │   │ name          │
└──────────────┘      │ title             │      │ member_id      FK │──┘   │ email  UNIQUE │
                      │ copies_total      │      │ loaned_on         │      │ joined_on     │
                      │ copies_available  │      │ due_on            │      └───────────────┘
                      └───────────────────┘      │ returned_on       │
                                                 └───────────────────┘
```

### Line by line

`authors` box
- **What:** one row per author: an `id` and a `name`.
- **Why:** the author's name lives in **one** place. If it's ever misspelled, you fix one row, and
  every book by that author shows the fix.
- **How:** `name` is `UNIQUE` (rule 6), so the same author can't be entered twice.
- **Remove it and…** each book would carry its author's name as text, and "Jane Austen" and
  "Jane Austin" would count as two different people.

`books.author_id FK ──▶ authors.id`
- **What:** each book points at its author's row.
- **Why:** one author has **many** books, and each book has **one** author. The pointer goes on the
  "many" side, in `books`, as in [Keys and relations](keys-and-relations.md).
- **How:** a foreign key: Postgres refuses a book whose `author_id` isn't a real author's `id`.
- **Remove it and…** (the foreign key) a book could name author `42`, who doesn't exist.

`copies_total` · `copies_available`
- **What:** how many copies the library owns, and how many are on the shelf.
- **Why:** the catalogue shows `copies_available`, so a member can see at a glance whether they can
  borrow the book today.
- **How:** borrowing takes one from `copies_available`, returning adds one back. `copies_total`
  changes only when the library buys or loses a copy.
- **Remove it and…** (`copies_available`) the app would have to count open loans for each book
  every time it shows the catalogue.

`members` box
- **What:** one row per member.
- **Why:** a loan needs to say **who** has the book, and the library needs a way to contact them.
- **How:** `email` is `UNIQUE` (rule 6); `name` isn't, because two people can share a name.
- **Remove it and…** (`UNIQUE` on `email`) the same person could sign up twice and get two library
  cards.

`loans` box, with its two arrows
- **What:** one row per borrowing: which book, which member, and three dates.
- **Why:** a member can borrow many books over time, and a book is borrowed by many members over
  time. That's many-to-many, and `loans` is the table in the middle, like the join table in
  [Keys and relations](keys-and-relations.md). Unlike a plain join table, it carries facts of its
  own: the dates.
- **How:** `book_id` points at `books`, `member_id` points at `members` (rule 5). A loan is
  **open** while `returned_on` is NULL.
- **Remove it and…** you could store "who has this book" in `books`, but then a book with three
  copies could only be lent to one person, and the history of who borrowed what would be lost.

### Acceptance checks

**Acceptance checks** are the tests that say "done". Yours are:

- ✅ Your four queries print exactly the output in *What you'll build*.
- ✅ After Grace borrows *A Wizard of Earthsea* and Ada returns *Emma* (Step 5), *Emma* has **2**
  copies available and *A Wizard of Earthsea* has **1**.
- ✅ Running `05-the-rules-hold.sql` (Step 6) shows **4 refusals**, one `ERROR` for each broken rule,
  and *Persuasion* still has **1** copy available at the end.

## Build it, step by step

Step 1 makes the database. Steps 2 to 6 each write one file, `01` to `05`, and run it against that
database.

**Where to put your files.** Next to your `docker-compose.yml`, make a folder called `my-library`,
and save your files in it with the names the steps give. The book's own answers are in
`code/sql/a1-build-library-schema/`, with the **same** file names, so you can compare, or run the
book's file when you're stuck and carry on from there.

### Step 1: this project's database

**Goal:** an empty database named `a1_build_library_schema`.

As in every A1 lesson, the project gets a database of its own, named after the page with each `-`
turned into `_`. From the book's folder, make sure Postgres is running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres a1_build_library_schema
```

<details><summary>Hint</summary>

`docker compose up -d --wait` starts the book's Postgres, or does nothing if it's already running.
`createdb` makes the database, and `-U postgres` logs in as the user `postgres`. You did exactly this
at the start of every A1 lesson; see [What is a database?](what-is-a-database.md) for the details.

</details>

**✅ Checkpoint.** `createdb` prints **nothing** when it works. If you run it a second time, you see
this:

```text
createdb: error: database creation failed: ERROR:  database "a1_build_library_schema" already exists
```

That's **harmless**: the database is already there, and nothing in it was touched. You only ever
need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: the schema

**Goal:** `my-library/01-schema.sql` creates the four tables from the spec, with every rule, and
lists them at the end with `\dt`. Running it twice must give the same output.

<details><summary>Hint 1: where to start</summary>

Create the tables in an order where every table a foreign key points at already exists: `authors`
and `members` point at nothing, `books` points at `authors`, and `loans` points at `books` and
`members`. So: `authors`, `books`, `members`, `loans`.

For "run it twice": start the file with `SET client_min_messages = warning;` and a
`DROP TABLE IF EXISTS` that lists all four tables, as every lesson's `01` file does.

</details>

<details><summary>Hint 2: the rules</summary>

Every rule in the spec is a constraint you've met: `NOT NULL`, `UNIQUE`, `REFERENCES`, or a `CHECK`.
A `CHECK` can compare two columns of the same row, for example `CHECK (due_on > loaned_on)`.

For rule 1, "between 0 and `copies_total`", SQL has a word you haven't seen yet: `x BETWEEN a AND b`
means `x >= a AND x <= b`. Both ends are included.

The spec also says every loan will be looked up by its book and by its member. Remember from
[Indexes](indexes.md) that Postgres doesn't index foreign key columns for you.

</details>

**✅ Checkpoint.** Run your file:

```bash
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/01-schema.sql
```

You should see this (the book's file, `code/sql/a1-build-library-schema/01-schema.sql`, prints the
same):

```text
{{#include ../../code/sql/a1-build-library-schema/01-schema.out}}
```

Each line is a command tag: one `CREATE TABLE` per table, one `CREATE INDEX` per index. Then `\dt`
lists the four tables. (`\dt` sorts them by name, which is why `loans` comes before `members`.)

- ✅ If you see four `CREATE TABLE`s and all four tables in the list, you're right.
- ✅ Run it a second time: the output is identical.
- ❌ If you see `relation "authors" does not exist` on a `CREATE TABLE`, a table points at one you
  haven't created yet: check the order.
- ❌ If a second run fails with `cannot drop table authors because other objects depend on it`, you
  dropped `authors` while `books` still pointed at it. Drop all four tables in one statement.
- ❌ If you see `FATAL:  database "a1_build_library_schema" does not exist`, go back to Step 1.

### Step 3: some data

**Goal:** `my-library/02-seed.sql` fills the tables with this data. *Seed data* is the name for the
starting rows you put into a new database.

```text
authors (id is numbered by Postgres, in this order)
  1  Jane Austen
  2  Frank Herbert
  3  Ursula K. Le Guin

books                        author  copies_total  copies_available
  1  Emma                         1             2                 1
  2  Persuasion                   1             1                 1
  3  Dune                         2             3                 1
  4  A Wizard of Earthsea         3             2                 2
  5  The Left Hand of Darkness    3             1                 1

members                      email               joined_on
  1  Ada Lovelace            ada@example.com     2026-01-15
  2  Alan Turing             alan@example.com    2026-02-01
  3  Grace Hopper            grace@example.com   2026-03-10

loans     book  member  loaned_on   due_on      returned_on
  1          1       1  2026-09-01  2026-09-15  (not returned)
  2          3       1  2026-09-10  2026-09-24  (not returned)
  3          3       2  2026-09-20  2026-10-04  (not returned)
  4          2       3  2026-08-01  2026-08-15  2026-08-14
  5          3       3  2026-07-01  2026-07-15  2026-07-20
```

The numbers add up: *Emma* has 2 copies and 1 open loan, so 1 is available. *Dune* has 3 copies and
2 open loans, so 1 is available.

<details><summary>Hint</summary>

One `INSERT` per table, each with several rows in its `VALUES` list. Insert the parents first
(`authors`, then `books`, then `members`, then `loans`), or the foreign keys refuse the children.

Don't give an `id`: Postgres numbers the rows 1, 2, 3… in the order you insert them, and the loans
rely on those numbers. For "not returned", write `NULL`.

</details>

**✅ Checkpoint.** Run `01` first (it empties everything), then your seed file:

```bash
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/01-schema.sql
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/02-seed.sql
```

The seed file prints:

```text
{{#include ../../code/sql/a1-build-library-schema/02-seed.out}}
```

`INSERT 0 3` means three rows went in (the **last** number counts them): 3 authors, 5 books,
3 members, 5 loans.

- ✅ If you see those four lines, you're right.
- ❌ If you see `violates foreign key constraint`, you inserted a child before its parent, or a
  number points at the wrong row.
- ❌ If you see `violates check constraint "books_check"`, a book has more copies available than it
  owns.
- ❌ If you see `cannot insert a non-DEFAULT value into column "id"`, remove `id` from your
  `INSERT`.
- ❌ If you run the seed file twice without `01` in between, you get
  `duplicate key value violates unique constraint "authors_name_key"`. That's rule 6 working. The
  books and loans have no such rule, so they went in a second time. Run `01`, then `02`.

### Step 4: the four questions

**Goal:** `my-library/03-queries.sql` answers the four questions from the spec with four `SELECT`s.

<details><summary>Hint 1: the catalogue</summary>

`books` has the `author_id`; `authors` has the name. A `JOIN` follows the pointer. Give the column a
clearer name with `AS author`.

</details>

<details><summary>Hint 2: who has what, and what's overdue</summary>

A loan row has two pointers, so the query needs **two** `JOIN`s: one to `members` for the name,
one to `books` for the title. You can write one `JOIN` after another.

"Not returned" is `returned_on IS NULL` (never `= NULL`, from
[Postgres data types](postgres-data-types.md)). Overdue is the same query with one more condition,
joined with `AND`: the due date is before `DATE '2026-09-24'`.

</details>

<details><summary>Hint 3: how popular</summary>

This is "books per author" from the *More examples* of [Keys and relations](keys-and-relations.md),
with loans instead of books. A plain `JOIN` would drop the books nobody has borrowed; which join
keeps them? And which `count(…)` counts a missing loan as 0?

</details>

**✅ Checkpoint.** Run it (the data from Step 3 must be in):

```bash
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/03-queries.sql
```

```text
{{#include ../../code/sql/a1-build-library-schema/03-queries.out}}
```

This is the output from *What you'll build*, and your first acceptance check.

- ✅ If your four tables match these, row for row, you're right.
- ✅ In the overdue list, only *Emma* shows. Ada's *Dune* is due **on** 2026-09-24, not before it,
  so it isn't late yet.
- ❌ If the last table has only 3 rows, you used `JOIN` instead of `LEFT JOIN`, and the two
  never-borrowed books dropped out.
- ❌ If those two books show `1` instead of `0`, you wrote `count(*)`: use `count(loans.id)`.
- ❌ If the rows are the same but in another order, check your `ORDER BY`.

### Step 5: borrow and return

**Goal:** `my-library/04-borrow-and-return.sql` does two things, each as one all-or-nothing change:

- Grace (member 3) borrows *A Wizard of Earthsea* (book 4) on 2026-09-24, due back 2026-10-08.
- Ada returns *Emma*: loan 1 gets `returned_on` 2026-09-24.

Then it shows books 1 and 4 with their `copies_available`.

<details><summary>Hint</summary>

Each change is two statements, one on `loans` and one on `books`, between a `BEGIN` and a `COMMIT`:
that's the transfer from [SQL transactions](sql-transactions.md), with a book instead of money.

To pick two rows at once, `WHERE id IN (1, 4)` means `WHERE id = 1 OR id = 4`: `IN (…)` is true when
the value is **any** of the ones in the list.

</details>

**✅ Checkpoint.** Run `01`, `02`, then your file:

```bash
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/04-borrow-and-return.sql
```

```text
{{#include ../../code/sql/a1-build-library-schema/04-borrow-and-return.out}}
```

Each transaction prints its `BEGIN`, a tag for each change, and `COMMIT`. *Emma* is back up to 2,
*A Wizard of Earthsea* down to 1: your second acceptance check.

- ✅ If you see two `COMMIT`s and `2` and `1`, you're right.
- ❌ If you see `violates check constraint "books_check"` and *A Wizard of Earthsea* shows `0`, you
  ran the file twice. The second run lent Grace another copy (from 1 to 0), then tried to put a
  third copy of *Emma* on a shelf that owns two: rule 1 refused it, and that transaction answered
  `ROLLBACK`. Run `01`, `02`, then this file once.

### Step 6: prove the rules hold

**Goal:** `my-library/05-the-rules-hold.sql` tries to break the rules on purpose, and shows that the
database refuses every attempt:

1. In one transaction, borrow *Persuasion* (book 2) **twice**, taking 1 from `copies_available`
   each time. The library has only one copy.
2. Add a member called `Ada Again` with Ada's email.
3. Add a loan (book 5, member 2) loaned on 2026-09-24 and due on 2026-09-01.
4. Add a loan for book 99, which doesn't exist.

Then show *Persuasion*'s `copies_available`.

<details><summary>Hint</summary>

Write each attempt as you would a normal change; the point is that it fails. `psql` carries on after
an error, so all four attempts run. For attempt 1, remember what happens to a transaction when one
of its statements fails, and what `COMMIT` prints then.

</details>

**✅ Checkpoint.** Run your file, straight after Step 5's:

```bash
docker compose exec -T db psql -U postgres -d a1_build_library_schema < my-library/05-the-rules-hold.sql
```

```text
{{#include ../../code/sql/a1-build-library-schema/05-the-rules-hold.out}}
```

Four `ERROR`s, one per attempt, each naming the rule it broke:

1. `books_check`: *Persuasion* would have had `-1` copies (the last number in `DETAIL:`). The first
   `UPDATE` worked (`UPDATE 1`), but the failed second one aborted the transaction, and `COMMIT`
   answered `ROLLBACK`: the first one was thrown away too.
2. `members_email_key`: the `UNIQUE` rule on `email`. `DETAIL:` names the email that's taken.
3. `loans_check`: due before it starts. (The `7` in `DETAIL:` is the `id` Postgres had picked for
   the new loan: loans 1–6 already exist.)
4. `loans_book_id_fkey`: the foreign key. `DETAIL:` says book `99` isn't in `books`.

And the last table: *Persuasion* still has `1` copy. That's your final acceptance check. Every rule
held, and nothing half-done was kept.

- ✅ If you see these four `ERROR`s, `ROLLBACK`, and `1`, you're done. You built it.
- ❌ If one of the attempts prints `INSERT 0 1` or `UPDATE 1` instead of an `ERROR`, the matching
  rule is missing from your `01-schema.sql`. Add it, and run `01` to `05` again, in order.
- ❌ If *Persuasion* shows `0`, you ran the two `UPDATE`s without `BEGIN`: the first one was kept on
  its own.

## Reference solution, line by line

These are the book's files: the book's automatic checks run them against a real Postgres on every
change, so the outputs you've seen are exactly what they print. The lines you've seen in
earlier lessons get a short note; the ones that are new **in combination** get the full treatment.
Each file runs like the others:
`docker compose exec -T db psql -U postgres -d a1_build_library_schema < code/sql/a1-build-library-schema/<file>`,
and its output is the one in the step's checkpoint.

### `01-schema.sql`

```sql
{{#include ../../code/sql/a1-build-library-schema/01-schema.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP TABLE IF EXISTS loans, books, members, authors;`
- **What:** hides small notes, then deletes all four tables if they're there.
- **Why:** this is the project's reset button: running `01` always starts from empty, whatever the
  later files did.
- **How:** one `DROP TABLE` for all four at once, so Postgres works out the order itself. (One
  table at a time, `authors` first, would fail: `books` still points at it.) `IF EXISTS` skips the
  drop on the very first run, and the `SET` line hides the note that would say so.
- **Remove it and…** a second run fails with `ERROR:  relation "authors" already exists`.

`CREATE TABLE authors (` … `name text NOT NULL UNIQUE`
- **What:** the authors table: a numbered `id` and a name that's never missing or repeated.
- **Why:** rules 6 and 7.
- **How:** `integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY` is the numbered primary key from
  [Keys and relations](keys-and-relations.md): Postgres picks 1, 2, 3… and refuses a number you type
  yourself. The other three tables start the same way.
- **Remove it and…** (`UNIQUE`) "Jane Austen" could be entered twice, and her books would be split
  between two authors.

`author_id integer NOT NULL REFERENCES authors (id),`
- **What:** each book's pointer to its author.
- **Why:** rule 5's idea, applied to books: no book by a missing author.
- **How:** a foreign key. `NOT NULL` adds "every book has an author".
- **Remove it and…** (`REFERENCES`) `author_id` would be a plain number, and nothing would stop
  `42`.

`copies_total integer NOT NULL CHECK (copies_total >= 0),`
- **What:** how many copies the library owns, never negative.
- **Why:** rule 2.
- **How:** a [check constraint](../glossary.md#check-constraint) on one column. Postgres names it
  after the table and that column: `books_copies_total_check`.
- **Remove it and…** (the `CHECK`) a typo could give a book `-2` copies.

`copies_available integer NOT NULL CHECK (copies_available BETWEEN 0 AND copies_total)`
- **What:** how many copies are on the shelf: at least 0, at most `copies_total`.
- **Why:** rule 1, and the rule that saved *Persuasion* in Step 6. It turns "can't lend a book we
  don't have" and "can't have more on the shelf than we own" into something the database checks on
  every change.
- **How:** `x BETWEEN a AND b` means `x >= a AND x <= b`, both ends included, so `0` and
  `copies_total` are both allowed. The check compares **two columns of the same row**: when
  Postgres tests a book, it reads that book's own `copies_total`. Because the check mentions two
  columns, Postgres doesn't name it after one of them: it's called `books_check`, the name you saw
  in Step 6. (You can choose a name yourself by writing `CONSTRAINT copies_in_range` before
  `CHECK`.)
- **Remove it and…** Step 6's double borrow would leave *Persuasion* at `-1`, and the catalogue
  would show a negative number of books on the shelf.

`CREATE TABLE members (` … `email text NOT NULL UNIQUE,` … `joined_on date NOT NULL`
- **What:** the members table.
- **Why:** rules 6 and 7. `email` is `UNIQUE`; `name` isn't, because two people can share a name.
- **How:** `joined_on` is a `date`, a day with no time, from
  [Postgres data types](postgres-data-types.md). `UNIQUE` gives the rule the name
  `members_email_key` you saw in Step 6.
- **Remove it and…** (`UNIQUE` on `email`) `Ada Again` goes in, and the library can no longer tell
  which Ada an email belongs to.

`book_id integer NOT NULL REFERENCES books (id),` · `member_id integer NOT NULL REFERENCES members (id),`
- **What:** the loan's two pointers: which book, and who has it.
- **Why:** rule 5.
- **How:** two foreign keys in one table: that's what makes `loans` the table in the middle of a
  many-to-many. Postgres names the first one `loans_book_id_fkey`.
- **Remove it and…** a loan for book `99` goes in, and "who has what" silently loses it: a `JOIN`
  finds no book to match.

`loaned_on date NOT NULL,` · `due_on date NOT NULL CHECK (due_on > loaned_on),`
- **What:** when the book went out, and when it must come back, which is always later.
- **Why:** rule 3. A loan due before it starts would be overdue from the moment it's made.
- **How:** `>` on two dates means "later than". Like `books_check`, this check mentions two columns,
  so it's called `loans_check`.
- **Remove it and…** (the `CHECK`) Step 6's third attempt goes in, and shows up in the overdue list
  at once.

`returned_on date CHECK (returned_on >= loaned_on)`
- **What:** when the book came back, if it has. Not before it went out.
- **Why:** rule 4, and rule 7's one exception: this is the only column with no `NOT NULL`, because
  an open loan has no return date yet.
- **How:** here is a subtle, useful fact: a `CHECK` refuses a row only when its test is **false**.
  For an open loan, `returned_on` is NULL, and `NULL >= loaned_on` isn't true or false, it's NULL
  (unknown). So the check lets it through, which is exactly what you want. Postgres names it
  `loans_check1`, because `loans_check` is taken.
- **Remove it and…** (the `CHECK`) a book could be "returned" a week before it was borrowed. Add
  `NOT NULL`, and you could never record an open loan.

`CREATE INDEX loans_book_id_idx ON loans (book_id);` · `CREATE INDEX loans_member_id_idx ON loans (member_id);`
- **What:** an [index](../glossary.md#index) on each of the loan's foreign key columns.
- **Why:** the app looks up loans **by book** ("who has *Dune*?") and **by member** (Ada's "my
  loans" page) all the time. With five loans it makes no difference; with five million, it's the
  difference between an instant answer and a slow one.
- **How:** from [Indexes](indexes.md): Postgres indexes every primary key and `UNIQUE` column for
  you, but not foreign keys. The `_idx` ending is a naming habit, not a rule.
- **Remove it and…** everything still works, and gets slower as the library grows.

`\dt`
- **What:** lists the tables in this database.
- **Why:** proof that all four were created.
- **How:** a psql [backslash command](../glossary.md#backslash-command), from
  [Tables, rows and psql](tables-rows-and-psql.md). psql runs it itself instead of sending it to
  Postgres, so it has no `;`.
- **Remove it and…** the tables are there; you don't see them.

### `02-seed.sql`

```sql
{{#include ../../code/sql/a1-build-library-schema/02-seed.sql}}
```

### Line by line

`INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert'), ('Ursula K. Le Guin');`
- **What:** three authors, in one statement.
- **Why:** the parents go in first, so the books have something to point at.
- **How:** no `id` column: Postgres numbers them 1, 2, 3 in the order they're listed.
- **Remove it and…** the `books` insert fails: `author_id` 1 isn't present in `authors`.

`INSERT INTO books (title, author_id, copies_total, copies_available) VALUES` …
- **What:** five books, each pointing at its author by number.
- **Why:** enough variety for every question: *Dune* is popular (3 copies, 2 out), and two books
  have never been borrowed, so question 4 has zeros to show.
- **How:** the spaces after `'Emma',` line the columns up for human eyes. SQL ignores them.
- **Remove it and…** the loans insert fails: `book_id` 1 isn't present.

`INSERT INTO members (name, email, joined_on) VALUES` …
- **What:** three members.
- **Why:** someone to borrow the books.
- **How:** `'2026-01-15'` is text in the `YYYY-MM-DD` form, which Postgres reads as a `date` because
  the column is a `date`.
- **Remove it and…** the loans insert fails: `member_id` 1 isn't present.

`INSERT INTO loans (book_id, member_id, loaned_on, due_on, returned_on) VALUES` …
- **What:** five loans: three still out, two returned.
- **Why:** the four questions need open loans, a late one (loan 1, due 2026-09-15), one due on the
  very day of the overdue check (loan 2), and returned ones that still count as "times borrowed".
- **How:** `NULL` in the last column means "not returned yet". Loan 5 came back late (due 07-15,
  returned 07-20), which is allowed: rule 4 only checks it wasn't returned before it went out.
- **Remove it and…** every loan query answers `(0 rows)`, and every book was borrowed `0` times.

### `03-queries.sql`

```sql
{{#include ../../code/sql/a1-build-library-schema/03-queries.sql}}
```

### Line by line

`-- The catalogue: every book with its author.`
- **What:** a comment: `--` to the end of the line is for humans, and Postgres ignores it.
- **Why:** four queries in one file are easier to read with a title each.
- **How:** psql prints nothing for a comment, so the output has no titles; the blank lines between
  the tables come from the queries themselves.
- **Remove it and…** the queries work the same.

`SELECT books.title, authors.name AS author, books.copies_available` · `FROM books JOIN authors ON authors.id = books.author_id` · `ORDER BY authors.name, books.title;`
- **What:** the catalogue: each book next to its author's name.
- **Why:** question 1.
- **How:** the [join](../glossary.md#join) from [Keys and relations](keys-and-relations.md). With
  two tables, each column is written `table.column`, so there's never a doubt which `name` or `id`
  you mean. `AS author` renames the column in the output. `ORDER BY` with two columns sorts by the
  first, then by the second among equal firsts: Jane Austen's two books come out as *Emma*, then
  *Persuasion*.
- **Remove it and…** (`ORDER BY`) the rows come in whatever order is quickest for Postgres.

`FROM loans` · `JOIN members ON members.id = loans.member_id` · `JOIN books ON books.id = loans.book_id`
- **What:** a **three-table** join: each loan, with its member's row and its book's row beside it.
- **Why:** a loan row on its own says `3, 1`: book 3, member 1. A person wants "Ada Lovelace,
  *Dune*". The names live in two other tables.
- **How:** start from `loans`, the table in the middle, and follow **each** of its pointers with its
  own `JOIN`. The first `JOIN` adds the member's columns to each loan; the second adds the book's
  columns to that. There's no limit: you can keep adding `JOIN`s, one per pointer you follow.
- **Remove it and…** (the `JOIN books` line) `books.title` fails with
  `missing FROM-clause entry for table "books"`: the query never brought that table in.

`WHERE loans.returned_on IS NULL`
- **What:** keeps only loans that haven't come back.
- **Why:** question 2 asks what's out **right now**.
- **How:** `IS NULL`, never `= NULL`, from [Postgres data types](postgres-data-types.md): `= NULL`
  is never true, and would return no rows.
- **Remove it and…** the returned loans (Grace's) show up too, as if she still had the books.

`AND loans.due_on < DATE '2026-09-24'`
- **What:** in the third query, also keeps only loans whose due date has passed.
- **Why:** question 3: overdue means **both** not returned **and** past its due date.
- **How:** the date is written out as a fixed `DATE '…'` so the answer never changes. A real app
  would use today's date instead: `CURRENT_DATE`. `<` means "strictly before", so a book due today
  isn't late yet: Ada's *Dune* (due 2026-09-24) isn't in the list.
- **Remove it and…** you get the "who has what" list again, all three open loans.

`SELECT books.title, count(loans.id) AS times_borrowed` · `FROM books LEFT JOIN loans ON loans.book_id = books.id` · `GROUP BY books.title`
- **What:** each book, and how many loans point at it: including books with **none**.
- **Why:** question 4. A never-borrowed book is a real answer ("nobody wants this"), so it must
  show with `0`, not vanish.
- **How:** the "books per author" pattern from [Keys and relations](keys-and-relations.md).
  `LEFT JOIN` keeps every row of `books` (the table after `FROM`), matched or not; a book with no
  loans gets one row whose loan columns are all NULL. `GROUP BY books.title` gathers the rows into
  one group per book, and `count(loans.id)` counts the loan ids in each group that aren't NULL: 3
  for *Dune*, and 0 for a book whose only row is the NULL one. Grouping by `title` works here
  because no two of these books share a title. In a real library, two books can, so you'd write
  `GROUP BY books.id, books.title`.
- **Remove it and…** (`LEFT`) the two never-borrowed books drop out. Use `count(*)` instead, and
  they show `1`: `count(*)` counts rows, and the NULL row is still a row.

`ORDER BY times_borrowed DESC, books.title;`
- **What:** most borrowed first; ties in title order.
- **Why:** a "popular books" list reads from the top.
- **How:** `ORDER BY` may use a name you gave with `AS`. `DESC` is biggest first, from
  [CRUD in SQL](crud-in-sql.md).
- **Remove it and…** (`books.title`) *Emma* and *Persuasion* (1 each) could swap places between
  runs, and so could the two `0`s.

### `04-borrow-and-return.sql`

```sql
{{#include ../../code/sql/a1-build-library-schema/04-borrow-and-return.sql}}
```

### Line by line

`BEGIN;` · `INSERT INTO loans …` · `UPDATE books SET copies_available = copies_available - 1 WHERE id = 4;` · `COMMIT;`
- **What:** Grace borrows *A Wizard of Earthsea*: a new loan, and one copy fewer on the shelf.
- **Why:** this is the most important idea in the project. Borrowing is **two** changes to **two**
  tables, and they only make sense together. A loan with no copy taken off would let the library
  lend out a copy it doesn't have. A copy taken off with no loan would lose a book: nobody would
  know who has it.
- **How:** the [transaction](../glossary.md#transaction) from
  [SQL transactions](sql-transactions.md): the bank transfer, with a book instead of money. If
  either statement fails, or the app crashes between them, Postgres keeps neither. `COMMIT` keeps
  both at once. The `INSERT` leaves out `returned_on`, so it's NULL: an open loan.
- **Remove it and…** (`BEGIN` and `COMMIT`) each statement is saved on its own, and a failure
  between them leaves the loan and the shelf disagreeing.

`BEGIN;` · `UPDATE loans SET returned_on = '2026-09-24' WHERE id = 1;` · `UPDATE books SET copies_available = copies_available + 1 WHERE id = 1;` · `COMMIT;`
- **What:** Ada returns *Emma*: loan 1 is closed, and the copy goes back on the shelf.
- **Why:** the same two-sided change, the other way round.
- **How:** `copies_available + 1` adds to whatever the value is now, as the transfer did with
  money. Rule 1 guards this side too: a third copy of *Emma* would break `books_check`.
- **Remove it and…** (the second `UPDATE`) *Emma* would stay at 1 available forever, with both
  copies really on the shelf.

`SELECT id, title, copies_available FROM books WHERE id IN (1, 4) ORDER BY id;`
- **What:** shows the two books that changed.
- **Why:** to check both transactions did their job.
- **How:** `IN (1, 4)` is new: it's true when `id` is **any** of the values in the list, the same as
  `id = 1 OR id = 4`, and shorter when the list grows.
- **Remove it and…** (`WHERE`) all five books are shown; the two you care about are harder to spot.

### `05-the-rules-hold.sql`

```sql
{{#include ../../code/sql/a1-build-library-schema/05-the-rules-hold.sql}}
```

### Line by line

`BEGIN;` · two `UPDATE books SET copies_available = copies_available - 1 WHERE id = 2;` · `COMMIT;`
- **What:** borrows the only copy of *Persuasion* twice, in one transaction.
- **Why:** the most likely real bug: two members click "borrow" on the last copy. The database, not
  the app, must be the final guard.
- **How:** the first `UPDATE` takes it from 1 to 0, which `BETWEEN 0 AND copies_total` allows. The
  second tries `-1`, `books_check` refuses it, and the transaction is aborted, as in
  [SQL transactions](sql-transactions.md). `COMMIT` answers `ROLLBACK`, so the first `UPDATE` is
  thrown away too.
- **Remove it and…** (`BEGIN` and `COMMIT`) the first `UPDATE` is saved on its own, and
  *Persuasion* ends at 0 with no loan to explain it.

`INSERT INTO members … ('Ada Again', 'ada@example.com', '2026-09-24');`
- **What:** a second member with Ada's email.
- **Why:** tests rule 6.
- **How:** `UNIQUE` on `email` refuses it: `duplicate key value violates unique constraint`.
- **Remove it and…** (this line) rule 6 goes untested.

`INSERT INTO loans … VALUES (5, 2, '2026-09-24', '2026-09-01');`
- **What:** a loan due three weeks before it starts.
- **Why:** tests rule 3.
- **How:** `due_on > loaned_on` is false, so `loans_check` refuses it.
- **Remove it and…** (this line) rule 3 goes untested.

`INSERT INTO loans … VALUES (99, 2, '2026-09-24', '2026-10-08');`
- **What:** a loan for book 99.
- **Why:** tests rule 5.
- **How:** there's no book 99, so the foreign key `loans_book_id_fkey` refuses it.
- **Remove it and…** (this line) rule 5 goes untested.

`SELECT title, copies_available FROM books WHERE id = 2;`
- **What:** *Persuasion*'s copies, after everything.
- **Why:** the proof: still `1`, as if nobody had tried.
- **How:** a plain `SELECT`, outside any transaction.
- **Remove it and…** the rules still held; you'd only have the errors as proof.

## Stretch goals

Each goal adds one query to the library you built. They read the data as Step 6 left it. Run each
solution file like the others:
`docker compose exec -T db psql -U postgres -d a1_build_library_schema < code/sql/a1-build-library-schema/<file>`.

### A limit of 3 open loans per member

The library wants a new rule: nobody may have more than **3** books out at once. Write a query that
finds every member **over** the limit, with how many open loans they have.

<details><summary>Solution</summary>

`code/sql/a1-build-library-schema/90-open-loan-limit.sql`:

```sql
{{#include ../../code/sql/a1-build-library-schema/90-open-loan-limit.sql}}
```

```text
{{#include ../../code/sql/a1-build-library-schema/90-open-loan-limit.out}}
```

The query counts each member's open loans with `JOIN`, `WHERE … IS NULL` and `GROUP BY`. It
groups by `members.id, members.name`, not the name alone: two members can share a name (only
the email is `UNIQUE`), and grouping by the name would merge them into one row and add their loans
together. The `id`
keeps them apart; the name is there so you can show it. The new
word is **`HAVING`**: it's a `WHERE` for groups. `WHERE` filters rows **before** they're grouped,
so it can't see a count yet; `HAVING` filters the groups **after** they're counted.
`HAVING count(loans.id) > 3` keeps only members with more than 3.

The first answer is `(0 rows)`: nobody is over the limit. That's correct, but it doesn't prove the
query works, so the second half **tests** it: inside a transaction, Alan borrows three more books
(he had 1), the same query finds him with `4`, and `ROLLBACK` undoes the three loans, as the safe
`DELETE` did in [SQL transactions](sql-transactions.md). (A real borrow would also update
`copies_available`; this test rolls everything back, so it skips that.)

Why a query and not a `CHECK`? A check constraint sees **one row** at a time, and "at most 3" is
about **many** rows. The usual answer is to check in your app, inside the borrowing transaction,
before the `INSERT`: count the member's open loans, and refuse if it's already 3. One catch: if
the same member borrows twice at the same moment (two browser tabs, say), both transactions can
count 2 open loans and both go ahead, ending at 4. Real apps close that gap by locking the
member's row first (`SELECT … FOR UPDATE`) or with a constraint; that's beyond A1, so here it's
enough to know the gap exists.

</details>

### The member who borrowed most

Find the member with the most loans ever, returned or not, and how many.

<details><summary>Solution</summary>

`code/sql/a1-build-library-schema/91-top-borrower.sql`:

```sql
{{#include ../../code/sql/a1-build-library-schema/91-top-borrower.sql}}
```

```text
{{#include ../../code/sql/a1-build-library-schema/91-top-borrower.out}}
```

Count loans per member (no `WHERE` this time: every loan counts), grouped by `members.id,
members.name` again so two members who share a name stay two rows. Sort the biggest count first, and
keep one row with `LIMIT 1`, from [CRUD in SQL](crud-in-sql.md). Grace has 3: two from the seed
data, and *A Wizard of Earthsea* from Step 5. A plain `JOIN` is right here: a member with no loans
can't be the top borrower. If two members tied, `members.name` would pick the first by name. To see
every tied member, drop `LIMIT 1` and read the top of the list.

</details>

### Books never borrowed

List the titles of books that have never been borrowed, not even once.

<details><summary>Solution</summary>

`code/sql/a1-build-library-schema/92-never-borrowed.sql`:

```sql
{{#include ../../code/sql/a1-build-library-schema/92-never-borrowed.sql}}
```

```text
{{#include ../../code/sql/a1-build-library-schema/92-never-borrowed.out}}
```

`LEFT JOIN` keeps every book, and gives a book with no loans one row whose loan columns are NULL.
`WHERE loans.id IS NULL` then keeps **only** those rows: the books no loan matched. In Step 4's
output, two books had `0`. Now only *The Left Hand of Darkness* is left, because Grace borrowed
*A Wizard of Earthsea* in Step 5. (The loan for book 5 in Step 6 was refused, so it never counted.)

</details>

## Remember this

- Design first, on paper: one table per **thing** (authors, books, members), and a table in the
  middle (loans) for a **relationship** that has facts of its own, such as dates.
- Put every rule in the table (`NOT NULL`, `UNIQUE`, `REFERENCES`, `CHECK`), and the database
  refuses bad data from every program, forever. A `CHECK` can compare two columns of one row.
- Follow each pointer with its own `JOIN`; use `LEFT JOIN` with `count(column)` when "none" is a
  real answer.
- When one action changes two things (a loan **and** the shelf), wrap it in `BEGIN` … `COMMIT`, so
  it happens all at once or not at all.

## Go deeper

- [Constraints](https://www.postgresql.org/docs/18/ddl-constraints.html) — Every kind of rule a table can enforce: CHECK, NOT NULL, UNIQUE, primary and foreign keys.
- [Aggregate functions (tutorial)](https://www.postgresql.org/docs/18/tutorial-agg.html) — count, GROUP BY and HAVING, with worked examples.

<!-- next:start -->

**Next:**

- [Cheat sheet: SQL](../a1-postgres/cheatsheet-sql.md)
- Hello, Axum (coming soon)

<!-- next:end -->
