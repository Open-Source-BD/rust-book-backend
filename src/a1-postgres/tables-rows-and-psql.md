# Tables, rows and psql

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can create a table with named, typed columns.
- You can insert rows and select exactly the columns you want.
- You can find your way around with psql's backslash commands.

## What & why

Create tables, add and read rows, and explore your database with psql's backslash commands.

Picture a paper sign-up form for a book club. It has four boxes, each with a label: **Member
number**, **Name**, **Email**, **Date joined**. The number box has little squares that only fit
digits, and the date box is printed as `____-__-__`. Every new member fills in the same four boxes,
and the club files the forms in one folder. Because every form has the same boxes, anyone can open
the folder and answer "who joined in February?" in seconds.

A [**table**](../glossary.md#table) is that form, made strict. When you create one, you promise its
shape: *every member has these four fields, and each field holds this kind of value*. Each filled-in
form is a [**row**](../glossary.md#row); each labelled box is a [**column**](../glossary.md#column).
Postgres keeps the promise for you: it refuses a row with a box that doesn't exist, or a date that
isn't a date. In the last lesson every column held text. In this one you give columns real types (a
whole number, a date), so the table's shape says what belongs in it.

To see that shape, and the rows inside it, you need a window into the database. That window is
[`psql`](../glossary.md#psql), the program you've already fed files to. This lesson opens it for
real: you'll type into it, and learn its own short commands that start with a backslash (`\`), like
`\d members` ("describe the table `members`") and `\dt` ("list my tables").

## The idea, slowly

### Step 1: this lesson's database

As in [What is a database?](what-is-a-database.md), this lesson gets a database of its own, named
after the lesson: `tables_rows_and_psql`. From the book's folder (the one with
`docker-compose.yml`), make sure Postgres is running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres tables_rows_and_psql
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres tables_rows_and_psql`
- **What:** creates a new, empty database called `tables_rows_and_psql`.
- **Why:** this lesson makes tables called `members`, `scratch` and `events`. In their own database,
  they can never clash with another lesson's tables.
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  name is the lesson's web address, `tables-rows-and-psql`, with each `-` turned into `_`.
- **Remove it and…** every later command fails with
  `database "tables_rows_and_psql" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres tables_rows_and_psql
```

`createdb` prints **nothing** when it works: no news is good news. If you run it a second time, you
see this:

```text
createdb: error: database creation failed: ERROR:  database "tables_rows_and_psql" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once; it survives restarts and `docker compose down` (only
`docker compose down -v` deletes it).

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: open psql and look around

So far you've fed `psql` whole files and read what came out. You can also **talk to it**: open it,
type one thing, see the answer, type the next. This is how you'll poke around a database, check
what's in a table, or try a query before you save it in a file.

```bash
docker compose exec db psql -U postgres -d tables_rows_and_psql
```

### Line by line

`docker compose exec db`
- **What:** runs the next command inside the `db` container, where `psql` lives.
- **Why:** you don't need `psql` installed on your own computer; the container already has it.
- **How:** notice there's **no `-T`** this time. `-T` was for feeding a file with `<`. Without it,
  Docker connects your keyboard and screen to the program, so you can type into it.
- **Remove it and…** (the `exec db` part) your own computer looks for a `psql` program, and most
  likely says `command not found`.

`psql -U postgres -d tables_rows_and_psql`
- **What:** starts `psql`, logged in as the user `postgres`, connected to this lesson's database.
- **Why:** `-d` picks **which database** everything you type goes to.
- **How:** with no file to read, `psql` waits for you to type, and shows a prompt when it's ready.
- **Remove it and…** (the `-d tables_rows_and_psql` part) `psql` connects to a database named
  after the user, `postgres`, and your tables for this lesson aren't there.

### Run it

Run the command. `psql` greets you and waits. Type `\q` and press Enter to leave. This is the whole
conversation:

```text
psql (18.6 (Debian 18.6-1.pgdg13+2))
Type "help" for help.

tables_rows_and_psql=# \q
```

Line by line:

- `psql (18.6 (Debian 18.6-1.pgdg13+2))`: the version of `psql` you're running. Yours may show a
  slightly different number; that's fine.
- `Type "help" for help.`: a friendly hint. (Typing `help` shows a short list of the most useful
  commands.)
- `tables_rows_and_psql=#`: the **prompt**. It tells you two things: you're connected to the
  `tables_rows_and_psql` database, and `psql` is ready for you to type. The `#` means you're logged
  in as a user with full powers, which the book's `postgres` user is.
- `\q`: what **you** typed: "quit". `psql` closes, and you're back in your own terminal.

`\q` is your first [**backslash command**](../glossary.md#backslash-command): a command that starts
with `\`. It isn't SQL. Postgres, the server, never sees it: `psql` itself reads it and acts on it.
That's why it has **no semicolon**: a backslash command ends at the end of the line. You'll meet two
more in this lesson, `\d` and `\dt`.

- ✅ If you saw the `tables_rows_and_psql=#` prompt, and `\q` gave you your own terminal back,
  you're right: you've opened and closed `psql` by hand.
- ❌ If you see `FATAL:  database "tables_rows_and_psql" does not exist`, go back to Step 1 and run
  `createdb`.
- ❌ If you're stuck and `\q` doesn't seem to work, press `Ctrl+C` (to clear a half-typed line),
  then type `\q` and Enter. `Ctrl+D` also quits.

### Step 3: create a table with a shape

Now a real table. A member of the book club has a number, a name, an email address and the date
they joined. This is `code/sql/tables-rows-and-psql/01-create-members.sql`:

```sql
{{#include ../../code/sql/tables-rows-and-psql/01-create-members.sql}}
```

### Line by line

`SET client_min_messages = warning;`
- **What:** tells Postgres to show only warnings and errors, not small notes.
- **Why:** the `DROP TABLE IF EXISTS` lines print a note on the very first run only. Hiding notes
  makes every run look the same.
- **How:** the setting lasts for this one connection; nothing changes for good.
- **Remove it and…** the first run also prints `NOTICE:  table "members" does not exist, skipping`.
  Harmless, but different from the book.

`DROP TABLE IF EXISTS members;`
- **What:** deletes the `members` table, and every row in it, if it exists.
- **Why:** so you can run this file again and start clean, as in the last lesson.
- **How:** `IF EXISTS` means "if there's no such table, skip it quietly".
- **Remove it and…** a second run fails with `ERROR:  relation "members" already exists`.

`DROP TABLE IF EXISTS events;`
- **What:** deletes a table called `events`, if it exists.
- **Why:** `events` is a table **you** will create at the end of this lesson, in *Your turn*.
  Dropping it here too means that whenever you run this file, the database is back to the very
  start of the lesson, and the list of tables you'll see in Step 4 matches the book's.
- **How:** the same as the line above. Right now there's no `events` table, so it's skipped.
- **Remove it and…** nothing breaks today. But after you've done *Your turn*, rerunning the lesson
  would show `events` in Step 4's table list as well.

`CREATE TABLE members (` … `);`
- **What:** creates a new, empty table called `members`.
- **Why:** a table, with its columns, must exist before any row can go in.
- **How:** the parentheses hold the list of columns, separated by commas, no comma after the last.
  The statement spans six lines and ends at `;`. The spaces that line the types up are only for
  your eyes; Postgres ignores them.
- **Remove it and…** there's no `members` table, and `\d members` answers
  `Did not find any relation named "members".`

`id integer,`
- **What:** a column called `id` that holds an `integer`: a whole number, such as `1` or `42`.
- **Why:** a number gives every member a short label that's easy to refer to. ("id" is short for
  "identifier".)
- **How:** Postgres refuses a value that isn't a whole number: `'Ada'` in this column is an
  error, `invalid input syntax for type integer: "Ada"`.
- **Remove it and…** members have no number; you can still tell them apart by name, until two
  people share one.

`name text,` · `email text,`
- **What:** two columns that hold `text`: any letters, any length.
- **Why:** names and email addresses are text.
- **How:** exactly like the `text` columns in the last lesson.
- **Remove it and…** (one of them) that fact isn't stored, and an `INSERT` that tries to fill it
  fails with `column "email" of relation "members" does not exist`.

`joined date`
- **What:** a column called `joined` that holds a `date`: a day on the calendar, such as
  `2026-01-15`.
- **Why:** a real `date` column means Postgres checks every value is a real day. It also knows that
  February comes before March, so later you can sort and compare dates correctly.
- **How:** dates are written year-month-day, `YYYY-MM-DD`. No comma after it: it's the last column.
- **Remove it and…** you don't know when anyone joined.

`\d members`
- **What:** a backslash command: **d**escribe the table `members`.
- **Why:** it shows the table's shape, so you can check it came out the way you meant.
- **How:** `psql` handles it itself. It asks Postgres for the table's details and prints them.
  There's no semicolon: it ends at the end of the line. It works the same in a file as typed at
  the prompt.
- **Remove it and…** the table is still created, but you don't see its shape.

### Run it

This lesson's files run exactly like the last lesson's: the same command, with this lesson's
database and file.

```bash
docker compose exec -T db psql -U postgres -d tables_rows_and_psql < code/sql/tables-rows-and-psql/01-create-members.sql
```

```text
{{#include ../../code/sql/tables-rows-and-psql/01-create-members.out}}
```

The first four lines are **command tags**, `psql` reporting what each statement did: `SET` (the
setting changed), `DROP TABLE` twice (one per drop line; both were skipped quietly on your first
run, since neither table existed yet), and `CREATE TABLE` (the table now exists).

Then comes `\d members`. Its title, `Table "public.members"`, names the table. `public` is the
section of the database where Postgres puts every table unless you say otherwise; you can ignore it
for this whole book. Below the title is one line per column, in the order you created them, under
five headings:

- **Column:** the column's name, exactly as you wrote it.
- **Type:** the kind of value it holds: `integer`, `text`, `date`. This is the promise from
  *What & why*, written down.
- **Collation:** the rules for sorting and comparing text (for example, whether `a` comes before
  `B`). Empty means "the database's normal rules", which is what you want.
- **Nullable:** whether the column may be left **empty**. Empty here means "yes, it may". A later
  lesson shows how to say "this column must always be filled in", and then this box reads
  `not null`.
- **Default:** the value Postgres fills in when an `INSERT` doesn't give one. Empty means "no
  default": a column you leave out is left empty.

- ✅ If you see `CREATE TABLE` and four columns, `id | integer`, `name | text`, `email | text` and
  `joined | date`, you're right: the table has the shape you promised.
- ✅ Run it a second time: the output is identical, thanks to the `DROP TABLE` lines.
- ❌ If you see `ERROR:  syntax error at or near ")"`, there's a comma after `joined date`. Remove
  it.

### Step 4: rows in, rows out

The table is empty. This file, `code/sql/tables-rows-and-psql/02-rows.sql`, adds two members, reads
them back two ways, and lists the tables in the database:

```sql
{{#include ../../code/sql/tables-rows-and-psql/02-rows.sql}}
```

### Line by line

`INSERT INTO members (id, name, email, joined) VALUES`
- **What:** starts adding rows to `members`, filling the columns `id`, `name`, `email` and
  `joined`, in that order.
- **Why:** naming the columns says which value goes in which box. It also keeps the statement
  working if someone adds a column to the table later.
- **How:** after `VALUES` comes one pair of parentheses per row. The statement goes on over the next
  two lines and ends at the `;`.
- **Remove it and…** (the column list) Postgres fills the columns in the table's own order, which
  works today but silently puts values in the wrong box if the table's column order ever changes.

`(1, 'Ada Lovelace', 'ada@example.com', '2026-01-15'),`
- **What:** the first row: member `1`, Ada Lovelace, her email, and the day she joined.
- **Why:** one pair of parentheses is one row, one filled-in form.
- **How:** values go in the same order as the column list. `1` has no quotes, because it's a
  number. The name and email are text, so they're in single quotes. The date is in single quotes
  too: you write it like text, and Postgres turns it into a `date` because that's the column's
  type. The comma at the end means "another row follows".
- **Remove it and…** (the whole line) only Alan is added, and the tag says `INSERT 0 1`.

`(2, 'Alan Turing', 'alan@example.com', '2026-02-01');`
- **What:** the second row: member `2`, Alan Turing.
- **Why:** two rows in one statement is shorter, and faster, than two separate `INSERT`s.
- **How:** it ends with `;` instead of a comma, because it's the last row.
- **Remove it and…** (the `;`) Postgres reads the next line as part of the same statement and
  stops with a syntax error at `SELECT`.

`SELECT * FROM members;`
- **What:** reads every row, with every column.
- **Why:** to check both rows really went in.
- **How:** `*` means "all columns", in the table's own order.
- **Remove it and…** the rows are still stored; you don't see them.

`SELECT name, joined FROM members;`
- **What:** reads every row, but only the `name` and `joined` columns.
- **Why:** you rarely need every column. Asking only for what you need keeps the answer short and
  readable, and sends less data. This is how most real queries look.
- **How:** instead of `*`, list the columns you want, separated by commas. They come back in the
  order you list them, whatever the table's order.
- **Remove it and…** (a column name, say `joined`) only `name` comes back. The dates are still
  stored.

`\dt`
- **What:** a backslash command: list the **t**ables in this database (**d** for describe, **t**
  for tables).
- **Why:** it answers "what's in here?" whenever you open a database you don't know.
- **How:** handled by `psql`, no semicolon, like `\d`.
- **Remove it and…** nothing changes in the data; you don't see the list.

### Run it

```bash
docker compose exec -T db psql -U postgres -d tables_rows_and_psql < code/sql/tables-rows-and-psql/02-rows.sql
```

```text
{{#include ../../code/sql/tables-rows-and-psql/02-rows.out}}
```

From the top:

- `INSERT 0 2`: the command tag for an insert. The **last** number is how many rows went in: `2`.
  The `0` in the middle is a leftover from old versions of Postgres and is always `0`; ignore it.
- The first table is `SELECT *`: all four columns, both rows, and `(2 rows)`, the count of rows in
  the answer. Numbers are lined up on the right, text and dates on the left.
- The second table is `SELECT name, joined`: the same two rows, only the columns you asked for.
- `List of tables` is `\dt`. Each line is one table: its **Schema** (the `public` section from
  Step 3), its **Name**, its **Type** (`table`) and its **Owner**, the user who created it
  (`postgres`). `(1 row)` here means one table: `members`.

- ✅ If you see `INSERT 0 2`, both members in both tables, and `members` in the list of tables,
  you're right.
- ❌ If you see `ERROR:  relation "members" does not exist`, run `01-create-members.sql` first.
- ❌ If you see four rows instead of two, you ran this file twice. That's expected: each run adds
  two more members. Run `01-create-members.sql` again to start clean, then this file once.

## You might be wondering…

**"Why does `\d` have no semicolon?"**
Because it isn't SQL. The `;` tells `psql` "this SQL statement is finished, send it to the server".
Backslash commands are never sent to the server; `psql` handles them itself, and each one ends at
the end of its line. SQL statements need `;`, backslash commands don't.

**"Can I change a table later?"**
Yes. `ALTER TABLE` changes an existing table: add a column, remove one, rename one, and more. The
rows already in the table stay. *More examples* adds a `phone` column to `members`.

**"What's `id` for, if nothing checks it?"**
Right now, it's only a number we promise to keep different for each member. Nothing stops two
members from both having `id` `1`: run `02-rows.sql` twice and you get two Adas, both with `1`.
Postgres *can* check that an id is never repeated, and let other tables point at a row by its id;
that's what **keys** are for, and they get their own lesson, [Keys and relations](keys-and-relations.md), a few lessons
from now.

**"Why is the date in quotes?"**
Because in SQL, anything that isn't a plain number or an SQL word is written in single quotes.
`'2026-01-15'` looks like text, but when it goes into a `date` column, Postgres reads it as a date
and checks it's a real one. Without quotes, `2026-01-15` is maths: 2026 minus 1 minus 15, which is
the number `2010`, and Postgres refuses a number in a date column (see **Common mistakes**). And a
date in quotes that isn't a real day is refused too:

```sql
{{#include ../../code/sql/tables-rows-and-psql/74-impossible-date.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/74-impossible-date.out}}
```

February has no 30th, so the row is refused, and nothing is added. (`psql` cuts a long line short
and marks the cut with `...`; the `^` still points at the value it didn't like.)

**"Why did my prompt change to `-#`?"**
If you type SQL at the prompt and press Enter **before** the `;`, `psql` waits for the rest of the
statement:

```text
tables_rows_and_psql=# SELECT name FROM members
tables_rows_and_psql-# ;
     name
--------------
 Ada Lovelace
 Alan Turing
(2 rows)

tables_rows_and_psql=# \q
```

The `-#` means "I'm still listening, the statement isn't finished". Type the `;` and press Enter,
and it runs. This is how you type a long statement over several lines. To throw away a half-typed
statement instead, press `Ctrl+C`.

## Coming from another language?

If you've used an ORM, you've written this lesson's table already, in another form. In **Prisma**
it's a `model Member` block with `Int`, `String` and `DateTime` fields; in
**Sequelize**, `sequelize.define('Member', { id: DataTypes.INTEGER, … })`; in the **Django ORM**, a
`class Member(models.Model)` with `models.IntegerField()`, `models.TextField()` and
`models.DateField()`; in **SQLAlchemy**, a class with `Column(Integer)`, `Column(Text)` and
`Column(Date)`; in **JPA/Hibernate**, an `@Entity class Member` with typed fields; in **GORM**, a Go
`type Member struct` with `int`, `string` and `time.Time` fields. When you run your migrations, each of
those becomes a `CREATE TABLE members (…)` like Step 3.

Two habits carry straight over. Creating an object (`prisma.member.create(…)`, `Member.objects.create(…)`)
becomes an `INSERT`. And picking fields (`select: { name: true, joined: true }` in Prisma,
`.values('name', 'joined')` in Django, `.Select("name", "joined")` in GORM) becomes
`SELECT name, joined FROM members`.

`psql`'s backslash commands are like a database browser (pgAdmin, DBeaver, TablePlus, or your
IDE's database panel) in text form: `\dt` is the list of tables in the sidebar, and `\d members` is
the "structure" tab.

If you know spreadsheets: `CREATE TABLE` is typing the header row, and then **locking** each
column to one kind of value, which a spreadsheet never does. `SELECT name, joined` is hiding every
other column.

## Common mistakes

**A misspelled SQL word.**

```sql
{{#include ../../code/sql/tables-rows-and-psql/70-misspelled-keyword.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/70-misspelled-keyword.out}}
```

`SELEC` isn't a word SQL knows, so Postgres couldn't make sense of the statement from the very
first word. `syntax error at or near "SELEC"` names the word it tripped on. `LINE 1:` repeats the
line it was reading, and the `^` below it points at the exact spot: here, the first letter.
**Fix:** `SELECT * FROM members;`.

**A misspelled table name.**

```sql
{{#include ../../code/sql/tables-rows-and-psql/71-misspelled-table.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/71-misspelled-table.out}}
```

This time the grammar is fine, but there is no table called `member`: the table is `members`, with
an `s`. **Relation** is Postgres's formal word for a table. The `^` points at the name it couldn't
find. **Fix:** check the real name with `\dt`, then use it: `SELECT * FROM members;`.

**Asking for a column that doesn't exist.**

```sql
{{#include ../../code/sql/tables-rows-and-psql/72-unknown-column.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/72-unknown-column.out}}
```

The table exists, but it has no `nickname` column, and the `^` points at that name. A table only
has the columns you gave it in `CREATE TABLE`. **Fix:** check the columns with `\d members`, and
ask for one that exists, such as `SELECT name FROM members;`. (Or, if members really need a
nickname, add the column with `ALTER TABLE`; see *More examples*.)

**A date without quotes.**

```sql
{{#include ../../code/sql/tables-rows-and-psql/73-unquoted-date.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/73-unquoted-date.out}}
```

Without quotes, `2026-04-01` is a sum: 2026 minus 4 minus 1. That's the whole number `2021`, an
`integer`, and the `joined` column only takes a `date`. The message says exactly that, and the `^`
points at the value. (`LINE 2:` because the value is on the statement's second line.) The `HINT`
suggests converting it, but the real fix is simpler. **Fix:** put the date in single quotes:
`'2026-04-01'`.

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d tables_rows_and_psql < code/sql/tables-rows-and-psql/<file>`.
They read the `members` table, so run `01` and `02` first.

### Add a column later

`ALTER TABLE` changes the shape of a table that already exists.

```sql
{{#include ../../code/sql/tables-rows-and-psql/50-add-column.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/50-add-column.out}}
```

`ADD COLUMN phone text` adds a fifth column, and `\d members` shows it at the end. Ada and Alan are
still there; their `phone` is empty, because they were added before the column existed. The last
line, `DROP COLUMN phone`, removes the column again, so the rest of the lesson sees the table from
Step 3, and you can run this file as many times as you like. Each change prints the tag
`ALTER TABLE`.

### Rename columns in the answer

`AS` gives a column in the answer a different name, as in the last lesson.

```sql
{{#include ../../code/sql/tables-rows-and-psql/51-column-alias.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/51-column-alias.out}}
```

The headings now read `member_name` and `member_since`. The table itself is unchanged: its columns
are still called `name` and `joined`. The new name is called an *alias*, and it only exists in this
one answer.

### List every database with `\l`

Open `psql` as in Step 2 and type `\l` (a lowercase L, for **l**ist). It prints `List of
databases`: one line per database on your Postgres server, with its name, its owner and some
settings. You'll see `rbh` from Your toolbox, `what_is_a_database` from the last lesson,
`tables_rows_and_psql`, and three that Postgres made itself: `postgres` (the database `psql` connects to when you leave
out `-d`) and `template0` and `template1` (the blueprints that `createdb` copies to make a new database). The
exact list depends on which lessons you've done, which is why the book doesn't print it here. The
lines are wide; if they wrap onto two lines in a narrow window, that's fine.

### Ask psql for help with `\?`

Type `\?` at the prompt, and `psql` lists **every** backslash command, grouped under headings like
`General`, `Help` and `Informational`, each with a one-line description. You'll find `\q` ("quit
psql") in the first group. The list is much longer than your screen, so `psql` shows it one screen
at a time, with a `:` at the bottom: press Space for the next screen, and `q` to go back to your
prompt. Any long answer in `psql` works this way. (For help with SQL itself, `\h` followed by a
statement's name, like `\h CREATE TABLE`, shows how to write it.)

### Delete a table with `DROP TABLE`

`DROP TABLE` throws a table away, with every row in it. Here it's a throwaway table called
`scratch`, so nothing you care about is lost.

```sql
{{#include ../../code/sql/tables-rows-and-psql/52-drop-table.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/52-drop-table.out}}
```

The first `\dt` shows two tables, `members` and the new `scratch`. After `DROP TABLE scratch`, the
second `\dt` shows `members` alone: `scratch` is gone. There's no "are you sure?" and no undo, so
double-check the name before you drop. (If you've done *Your turn*, `events` shows up in both
lists too.)

## Your turn

### 🟢 Guided

Open `psql` by hand, as in Step 2, and describe the `members` table with a backslash command. Then
leave. Fill in the blanks:

```text
tables_rows_and_psql=# __ members
tables_rows_and_psql=# __
```

<details><summary>Solution</summary>

Run `docker compose exec db psql -U postgres -d tables_rows_and_psql` (no `-T`: you're typing, not
feeding a file). Then type `\d members`, Enter, and `\q`, Enter:

```text
psql (18.6 (Debian 18.6-1.pgdg13+2))
Type "help" for help.

tables_rows_and_psql=# \d members
              Table "public.members"
 Column |  Type   | Collation | Nullable | Default
--------+---------+-----------+----------+---------
 id     | integer |           |          |
 name   | text    |           |          |
 email  | text    |           |          |
 joined | date    |           |          |

tables_rows_and_psql=# \q
```

It's the same shape Step 3's file printed: typed at the prompt or read from a file, a backslash
command does the same thing. No semicolon on either line. If `\d members` answers
`Did not find any relation named "members".`, run `01-create-members.sql` first.

</details>

### 🟡 Tweak

Add a third member to the book club: Grace Hopper, member `3`, `grace@example.com`, who joined on
10 March 2026. Then show the whole table.

<details><summary>Solution</summary>

`code/sql/tables-rows-and-psql/90-third-member.sql`:

```sql
{{#include ../../code/sql/tables-rows-and-psql/90-third-member.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/90-third-member.out}}
```

It's Step 4's `INSERT` with one row, so the tag says `INSERT 0 1`. The date is written
year-month-day, `'2026-03-10'`, in quotes. Run `01` and `02` first; this file adds to their table.
Run it twice and Grace appears twice, both with `id` `3`: nothing checks ids yet.

</details>

### 🔴 From scratch

Create a table called `events` with three columns: `id` (a whole number), `title` (text) and
`happens_on` (a date). Put two events in it, then show **only** their titles. Make the file safe to
run twice.

<details><summary>Solution</summary>

`code/sql/tables-rows-and-psql/91-events.sql`:

```sql
{{#include ../../code/sql/tables-rows-and-psql/91-events.sql}}
```

```text
{{#include ../../code/sql/tables-rows-and-psql/91-events.out}}
```

The top two lines make it safe to run twice: each run drops the old `events` and starts fresh, so
you always see exactly two titles. `CREATE TABLE` follows Step 3, with `integer`, `text` and `date`.
`SELECT title` asks for one column, so the answer has one column. Run `\dt` now and you'll see
`events` next to `members`.

</details>

## Quick check

<div class="quiz" data-topic="tables-rows-and-psql"></div>

## Remember this

- A table promises a shape: `CREATE TABLE name (column type, …);`. Postgres refuses a value that
  doesn't fit its column's type, such as a number in a `date` column.
- `INSERT INTO t (a, b) VALUES (…), (…);` adds rows; the tag `INSERT 0 2` means 2 rows went in.
- `SELECT *` gives every column; `SELECT name, joined` gives only the ones you name.
- Open `psql` by hand with `docker compose exec db psql -U postgres -d <db>` (no `-T`). The prompt
  `<db>=#` means "ready".
- Backslash commands belong to `psql`, not SQL, and take no `;`: `\dt` lists tables, `\d name`
  describes one, `\l` lists databases, `\?` lists them all, `\q` quits.

## Go deeper

- [psql](https://www.postgresql.org/docs/18/app-psql.html) — Every psql option and backslash command.
- [CREATE TABLE](https://www.postgresql.org/docs/18/sql-createtable.html) — The full reference for creating tables.

<!-- next:start -->

**Next:**

- [Postgres data types](../a1-postgres/postgres-data-types.md)

<!-- next:end -->
