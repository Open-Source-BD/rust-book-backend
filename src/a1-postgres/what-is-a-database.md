# What is a database?

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can say what a database is and why apps don't keep their data in plain files.
- You know what a table, a row and a column are.
- You have run your first two SQL queries against your own Postgres.

## What & why

Why apps store data in a database, what Postgres and SQL are, and your first queries.

Imagine a small app that you and your flatmates share: a shopping list with prices. The quickest
way to build it is to keep everything in one text file, `shopping.txt`, with one item per line:

```text
milk,1.99
bread,2.49
coffee,12,99
```

It works on day one. Then real life arrives:

- **Two people save at the same moment.** Your app reads the file, adds "eggs", and writes it back.
  At the same second, your flatmate's app reads the *old* file, adds "rice", and writes it back.
  The second write wins, and "eggs" is gone. Nobody gets an error; the item silently disappears.
- **Finding one line in a million.** To find "coffee", the app reads the file from the very first
  line until it gets there. With three lines that's instant. With a million, every search reads
  the whole file.
- **The power goes out halfway through a save.** Half of the new file is on the disk, half isn't.
  The next time the app starts, the file is broken, and so is everything in it.
- **A typo gets in.** Look at the last line: someone typed `12,99` instead of `12.99`. The file
  doesn't care; it stores whatever it's given. Now the "coffee" line has *three* pieces instead of
  two, and the code that adds up prices either crashes or quietly gets the total wrong.

Every app that keeps data meets these four problems sooner or later. A
[**database**](../glossary.md#database) is a program whose only job is to solve them: to keep data
**safe** (nothing lost, even in a crash), **correct** (a price is always a number) and **fast to
find** (even among millions), while many users read and write at once.

Think of the difference between a pile of papers on the floor and a library with a librarian. In
the pile, anyone can drop a page anywhere, pages get lost, and finding one means digging through
everything. In the library, you *ask the librarian*: they file every book in its place, refuse a
book with no title, keep a catalogue so they can find any book in seconds, and deal with one
visitor at a time at the desk, so two people never grab the same book. Your app never touches the
shelves itself. It asks, and the librarian does the work.

This book's librarian is **PostgreSQL**, or **Postgres** for short: a free database that you
started in Docker in [Your toolbox](../part-0-start/your-toolbox.md). You talk to it in
[**SQL**](../glossary.md#sql) (Structured Query Language, said "S-Q-L" or "sequel"), a language made
for one thing: asking a database questions and giving it instructions. Postgres is the program; SQL
is the language you speak to it. In this lesson you'll write your first SQL and run it against your
own Postgres.

## The idea, slowly

### Step 1: the shape of data

A database stores data in [**tables**](../glossary.md#table). A table looks like a spreadsheet:
names across the top, one line per thing you store. Here is a small table called `friends`:

```text
              table: friends
                 ┌─────────┬────────────┐
   columns ───▶  │  name   │  city      │
                 ├─────────┼────────────┤
   row 1   ───▶  │  Ada    │  London    │
   row 2   ───▶  │  Linus  │  Helsinki  │
                 └─────────┴────────────┘
```

### Line by line

`columns ───▶ │ name │ city │`
- **What:** the [**columns**](../glossary.md#column): `name` and `city`. A column is one field that
  every entry has, and it has a name. Columns go **across**.
- **Why:** the names say what each value means: `London` is a `city`, not a name.
- **How:** every row below has exactly one value under each column.
- **Remove it and…** you have values with no names, and nobody can tell what `Helsinki` is.

`row 1 ───▶ │ Ada │ London │` · `row 2 ───▶ │ Linus │ Helsinki │`
- **What:** two [**rows**](../glossary.md#row). A row is one entry: one friend. Rows go **down**.
  Ada is one row; Linus is another.
- **Why:** each thing you store gets a row of its own, so adding a friend means adding a row.
- **How:** a row holds one value for each column, in the column's place: `Ada` under `name`,
  `London` under `city`.
- **Remove it and…** (a row) that friend is no longer stored; the columns stay.

`table: friends` (the whole box)
- **What:** the **table**: the whole thing, with a name of its own, `friends`.
- **Why:** one database holds many tables (friends, pets, books…), and the name says which one you
  mean.
- **How:** you use the name in SQL to say which table to read or change, as you'll see in Step 4.
- **Remove it and…** (the name) there is no way to ask for this table and not another one.

If you've used a spreadsheet, you already know this shape: a sheet is a table, the header line
holds the column names, and each line below it is a row. There is one big difference. In a
spreadsheet you can type anything into any cell. In a database, every column holds one *kind* of
value (text, whole numbers, dates…), and the database refuses a value of the wrong kind. That's how
it stops the `12,99` typo from the shopping list. In this lesson every column holds text; the kinds
of values get their own lesson two lessons from now.

One Postgres server can hold many **databases**, and each database holds its own tables. That's
the next step.

### Step 2: one database per lesson

Your Postgres already has one database, `rbh`, from [Your toolbox](../part-0-start/your-toolbox.md).
In Part A1, **every lesson gets a database of its own**, named after the lesson. This lesson's is
called `what_is_a_database`. Make sure Postgres is running, then create it. Run both commands from
the book's folder, the one that contains `docker-compose.yml`:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres what_is_a_database
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, exactly as in [Your toolbox](../part-0-start/your-toolbox.md).
- **Why:** every command in this lesson talks to Postgres, so it must be running first.
- **How:** if Postgres is already running, this does nothing and returns at once. It's safe to run
  any time.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres what_is_a_database`
- **What:** creates a new, empty database called `what_is_a_database` inside your Postgres.
- **Why:** this lesson makes tables called `friends`, `pets` and `books`. Later lessons make their
  own tables, and some reuse the same names with different columns (a later lesson has its own
  `books` table). If every lesson shared one database, one lesson's `books` would clash with
  another's. With one database per lesson, each lesson has its own clean space, and nothing you do
  here can break a table from another lesson. It's like keeping one notebook per school subject
  instead of writing everything in one.
- **How:**
  - `docker compose exec db` means "run the next command inside the running `db` container", the
    same as in Your toolbox. You don't need anything installed except Docker.
  - `createdb` is a small program that comes with Postgres. Its only job is to create a database.
  - `-U postgres` says which Postgres **user** to log in as: `postgres`, the user from the Compose
    file.
  - `what_is_a_database` is the new database's name: the lesson's name (`what-is-a-database`, from
    the page's web address), with each `-` changed to `_`. Underscores, because SQL would read a
    `-` in a name as a minus sign, so a name with dashes would need extra quoting everywhere.
- **Remove it and…** (the whole command) there is no `what_is_a_database` database, so every
  command in the rest of the lesson fails with `database "what_is_a_database" does not exist`
  (see **Common mistakes**).

### Run it

```bash
docker compose exec db createdb -U postgres what_is_a_database
```

`createdb` prints **nothing** when it works: it creates the database and gives you your prompt
back. No news is good news.

If you run it a second time (tomorrow, say, when you've forgotten you already did), you see this:

```text
createdb: error: database creation failed: ERROR:  database "what_is_a_database" already exists
```

That's **harmless**. It says "you already made this database", and nothing was changed or deleted.
Your database and everything in it are still there. Carry on with the next step.

You create each lesson's database **once**. It lives in the `pgdata` volume, so it survives
`docker compose down`, restarts and reboots. The only thing that removes it is
`docker compose down -v`, which deletes every database; after that, run `createdb` again.

- ✅ If you see no output at all, or the `already exists` message, you're right: the database is
  there.
- ❌ If you see `service "db" is not running`, Postgres isn't started. Run
  `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, you're in the wrong folder. `cd` into
  the folder that contains `docker-compose.yml`.

### Step 3: your first query

A [**query**](../glossary.md#query) is one question or instruction you send to the database, in
SQL. The book keeps each lesson's SQL in files under `code/sql/`, one folder per lesson, so you run
exactly what's shown here. This is the whole of `code/sql/what-is-a-database/01-first-query.sql`:

```sql
{{#include ../../code/sql/what-is-a-database/01-first-query.sql}}
```

If you built your own folder in Your toolbox instead of using the book's, create the same folders
(`code/sql/what-is-a-database/`) next to your `docker-compose.yml`, and save each listing in this
lesson under the file name the lesson gives it.

### Line by line

`SELECT`
- **What:** the SQL word for "give me back…". It's the start of the statement.
- **Why:** it's the command you'll use most: every question you ask a database starts with
  `SELECT`.
- **How:** whatever follows `SELECT` is what you want back. Here it's a fixed value, so no table is
  needed yet; in Step 4 you'll `SELECT` from a table.
- **Remove it and…** Postgres doesn't know what you're asking for and stops with a *syntax error*
  (a sentence that breaks the grammar of SQL).

`'Hello, Postgres!'`
- **What:** a piece of **text** (programmers say a *string*).
- **Why:** it's the value we ask Postgres to give back.
- **How:** in SQL, text always goes in **single quotes** `'…'`. Everything between them is kept
  exactly as you typed it, spaces and capitals included.
- **Remove it and…** (swap the single quotes for double quotes) you get
  `column "Hello, Postgres!" does not exist`, because double quotes mean something else in SQL.
  **Common mistakes** shows this one.

`AS greeting`
- **What:** gives the result's column a name: `greeting`.
- **Why:** every answer from a database is a table, and every column in a table has a name. `AS`
  lets you choose a name you can read.
- **How:** `AS` is followed by the name you want. It doesn't store anything; it only labels the
  answer.
- **Remove it and…** Postgres makes up a name itself. For a value like this one, the column is
  called `?column?`, which tells you nothing (you saw it in Your toolbox's `select 1 + 1;`).

`;`
- **What:** the semicolon ends the statement.
- **Why:** a file can hold many statements, and the `;` is how [`psql`](../glossary.md#psql) (the
  program that runs the file, coming up in *Run it*) knows where one ends and the next begins.
- **How:** everything up to the `;` is sent to Postgres as one statement. A statement may span
  several lines; the `;` marks its end, not the line break.
- **Remove it and…** two statements run together into one broken one. **Common mistakes** shows
  the real error.

`SELECT 2 + 3 AS answer;`
- **What:** a second statement: Postgres adds 2 and 3, and names the result column `answer`.
- **Why:** it shows that `SELECT` can compute a value, not only repeat one back.
- **How:** `+`, `-`, `*` and `/` work as in any calculator. The numbers have no quotes, because
  they're numbers, not text.
- **Remove it and…** the file gives one answer instead of two. Nothing breaks.

### Run it

This is the command you'll use for every SQL file in Part A1:

```bash
docker compose exec -T db psql -U postgres -d what_is_a_database < code/sql/what-is-a-database/01-first-query.sql
```

Piece by piece:

- `docker compose exec … db` runs a command inside the `db` container, as before.
- `-T` tells Docker "no keyboard screen, please": we're feeding the command a file, not typing into
  it. Leave `-T` out and Docker refuses with
  `cannot attach stdin to a TTY-enabled container because stdin is not a terminal`.
- `psql` is Postgres's command-line program. It reads SQL, sends it to the
  Postgres server, and prints the answers.
- `-U postgres` logs in as the user `postgres`, and `-d what_is_a_database` picks **this lesson's
  database**. Change this part to change which database the SQL runs in.
- `< code/sql/what-is-a-database/01-first-query.sql` feeds the file to `psql` as if you had typed
  it. `<` is your terminal's "take the input from this file" sign. The path starts from the book's
  folder, so run the command there.

```text
{{#include ../../code/sql/what-is-a-database/01-first-query.out}}
```

That's one answer per `SELECT`, in order. Each answer is a small table: the column name
(`greeting`, then `answer`), a line of dashes, the value, and a count of rows, `(1 row)`. The blank
line between them separates the two answers.

- ✅ If you see `Hello, Postgres!` under `greeting` and `5` under `answer`, you're right: you've run
  your first SQL file.
- ❌ If you see `FATAL:  database "what_is_a_database" does not exist`, you skipped Step 2. Run the
  `createdb` command, then this one again.
- ❌ If your terminal says `no such file or directory`, you're not in the book's folder, or the
  file isn't saved at that path. `cd` into the folder with `docker-compose.yml`.

### Step 4: your first table

Fixed values are a start, but a database is for *storing* things. This file creates the `friends`
table from Step 1, puts two friends in it, and reads them back. It's
`code/sql/what-is-a-database/02-first-table.sql`:

```sql
{{#include ../../code/sql/what-is-a-database/02-first-table.sql}}
```

The first two lines are there so you can **run this file as many times as you like** and always
get the same result. Think of them as "start clean". The real work starts at `CREATE TABLE`.

### Line by line

`SET client_min_messages = warning;`
- **What:** tells Postgres "only show me warnings and errors, not small notes".
- **Why:** the next line prints a note the very first time you run the file, and not after that.
  Hiding notes makes every run print exactly the same thing.
- **How:** `SET` changes a setting for this one connection only; `client_min_messages` is the
  setting for which messages are shown. Nothing is changed for good.
- **Remove it and…** the first run also prints `NOTICE:  table "friends" does not exist, skipping`.
  It's harmless, but it makes your first run look different from the book's.

`DROP TABLE IF EXISTS friends;`
- **What:** deletes the `friends` table, with everything in it, **if** it exists.
- **Why:** so the file can be run again. On a second run, `friends` already exists, and without
  this line `CREATE TABLE` would fail. Deleting it first means every run starts from nothing.
- **How:** `DROP TABLE` deletes a table; `IF EXISTS` means "and if there's no such table, don't
  complain, skip it" (that skip is the note the `SET` line hides).
- **Remove it and…** the first run works, but the second prints
  `ERROR:  relation "friends" already exists`. ("Relation" is Postgres's formal word for a table.)
  Worse, the `INSERT` below still runs, so the table now holds Ada and Linus **twice**.

`CREATE TABLE friends (` … `);`
- **What:** creates a new, empty table called `friends`.
- **Why:** a table must exist, with its columns, before you can put rows in it.
- **How:** after the table's name, the parentheses `( … )` hold the list of columns, separated by
  commas. The whole statement spans four lines and ends at the `;` after the closing parenthesis.
- **Remove it and…** there's nowhere to put the rows: the `INSERT` fails with
  `relation "friends" does not exist`.

`name text,` · `city text`
- **What:** the two columns: one called `name` and one called `city`. Both hold `text`.
- **Why:** these are the two facts we store about each friend.
- **How:** each column is its name followed by its **type**, the kind of value it holds. `text`
  means "any text, any length". There are types for whole numbers, prices, dates and more; they're
  the topic of the lesson after next. There's a comma between the columns, but none after the last
  one.
- **Remove it and…** (the type) Postgres refuses to create the table and reports a syntax error:
  every column needs a type. (A comma after `city text` is a syntax error too, because Postgres
  expects another column after a comma.)

`INSERT INTO friends (name, city) VALUES ('Ada', 'London'), ('Linus', 'Helsinki');`
- **What:** adds two rows to `friends`: Ada from London, and Linus from Helsinki.
- **Why:** a new table is empty. This is how data gets in.
- **How:** `INSERT INTO friends` names the table; `(name, city)` lists which columns you're filling,
  in which order; `VALUES` is followed by one pair of parentheses per row, with the values in the
  same order as the columns. The values are text, so they're in single quotes. Commas separate the
  rows.
- **Remove it and…** the table stays empty, and the `SELECT` below answers with the column names
  and `(0 rows)`.

`SELECT * FROM friends;`
- **What:** reads every row of `friends`, with every column.
- **Why:** to see that the rows really went in.
- **How:** `*` means "all columns"; `FROM friends` says which table to read. It's the same
  `SELECT` as in Step 3, now reading from a table instead of fixed values.
- **Remove it and…** the rows are still saved, but you don't see them. The data is in the database
  either way; `SELECT` only reads it.

### Run it

The same command as before, with the new file name. The database stays the same:

```bash
docker compose exec -T db psql -U postgres -d what_is_a_database < code/sql/what-is-a-database/02-first-table.sql
```

```text
{{#include ../../code/sql/what-is-a-database/02-first-table.out}}
```

The first four lines are `psql` telling you what each statement did. They're called **command
tags**:

- `SET`: the setting was changed.
- `DROP TABLE`: the drop ran. (On the first run there was nothing to drop, so it was skipped
  quietly, thanks to the `SET` line.)
- `CREATE TABLE`: the table was created.
- `INSERT 0 2`: rows were inserted. The **last** number is how many: `2`. The `0` in the middle is
  a leftover from old versions of Postgres and is always `0` today; you can ignore it.

Then comes the answer to `SELECT *`: both columns, both rows, and `(2 rows)`.

Now run the exact same command a second time. The output is **identical**, still two rows, not
four, because the `DROP TABLE` line threw away the old table before the new one was made.

- ✅ If you see `INSERT 0 2` and a table with Ada and Linus, ending in `(2 rows)`, you're right: you've
  created your first table and stored data in it.
- ✅ If a second run prints the same output, you're right again: that's the "start clean" lines
  doing their job.
- ❌ If you see `ERROR:  relation "friends" already exists`, the `DROP TABLE IF EXISTS friends;`
  line is missing from your file. Add it back.

### Step 5: where your SQL actually goes

It's worth seeing the trip your file takes, because every SQL command in the book takes the same
one:

```text
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ your command │ ──▶ │     psql     │ ──▶ │   Postgres   │ ──▶ │ data on disk │
│  (terminal)  │     │              │     │    server    │     │ (the pgdata  │
│              │ ◀── │              │ ◀── │              │ ◀── │   volume)    │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                     └───── inside the db container ─────┘
```

The arrows pointing right (`──▶`) carry your SQL in; the arrows pointing left (`◀──`) carry the
answer back.

### Line by line

`your command ──▶ psql`
- **What:** your terminal runs `docker compose exec`, which hands the file to `psql` **inside** the
  `db` container.
- **Why:** `psql` is the program that can talk to Postgres, and it's already in the container, so
  you don't need it installed.
- **How:** the `<` in your command feeds the file to `psql`, and `-T` lets Docker accept a file
  instead of a keyboard.
- **Remove it and…** your SQL never leaves your terminal.

`psql ──▶ Postgres server`
- **What:** `psql` is only a messenger. It sends each statement, up to its `;`, to the **Postgres
  server**, the program that's always running in that container.
- **Why:** the server is the librarian from *What & why*; only it touches the data.
- **How:** one statement at a time, in the order they appear in the file.
- **Remove it and…** `psql` has nobody to ask, and nothing runs.

`Postgres server ──▶ data on disk`
- **What:** the server does the real work. It creates tables, stores rows, and saves them to disk,
  in the `pgdata` volume from Your toolbox.
- **Why:** so your data survives a restart.
- **How:** Docker keeps the `pgdata` volume outside the container.
- **Remove it and…** (the volume) after `docker compose down` and `up`, your tables would seem
  to have vanished, as Your toolbox showed.

The bottom line (every `◀──`)
- **What:** the server sends the answer back to `psql`, and `psql` draws it as the text table you
  saw, in your terminal.
- **Why:** every statement gets an answer: rows, a command tag, or an error.
- **How:** the answer travels back along the same path it came in on.
- **Remove it and…** your SQL would still run, but you'd never see what it did.

Later in the book, your Rust program will take `psql`'s place: it sends SQL to the same server and
gets rows back. The server and the data don't change.

## You might be wondering…

**"Is Postgres the same as SQL?"**
No. SQL is a **language**; Postgres is a **program** that understands it. Many databases speak SQL
(MySQL, SQLite, SQL Server, Oracle…), the way many people speak English. Each has a slightly
different accent (a few extra words of its own), but the basics you learn in Part A1 (`SELECT`,
`CREATE TABLE`, `INSERT`) work almost everywhere.

**"Why Postgres and not MySQL, SQLite or MongoDB?"**
They're all good tools. **MySQL** is also a free SQL database, and Postgres and MySQL can both run
very large apps. **SQLite** keeps the whole database in one file inside your app; it's great for
phone apps and small tools, but less suited to a server with many users writing at once.
**MongoDB** stores JSON-like documents instead of tables, and checks much less about their shape
by default. Postgres is free, very strict about keeping data correct (it's the librarian who
refuses the `12,99`), and one of the most common choices for Rust backends. SeaORM, the library
this book uses from Part A3, supports it fully.

**"Where is my data actually stored?"**
In the Docker volume from [Your toolbox](../part-0-start/your-toolbox.md) (`pgdata`, which Compose
names `rust-book-backend_pgdata` after the book's folder). `rbh`, `what_is_a_database` and every
lesson database you make later all live in the same Postgres server, so they all live in that one
volume. It survives `docker compose down`; only `docker compose down -v` deletes it.

**"Do I have to type SQL words in uppercase?"**
No. `select 2 + 3 as answer;` works exactly like `SELECT 2 + 3 AS answer;`. Writing SQL's own
words in UPPERCASE and your names (`friends`, `city`) in lowercase is a habit, not a rule: it makes
it easy to see which words are SQL's and which are yours. Two things *do* care about case: text in
single quotes keeps exactly the letters you typed (`'Ada'` is not `'ADA'`), and Postgres turns an
unquoted name into lowercase, so `AS Greeting` gives a column called `greeting`.

**"Do I need to run `createdb` every time I start?"**
No, only once per lesson. The database is stored in the volume and is still there next time. If
you run it again anyway, you get the harmless `already exists` message from Step 2.

## Coming from another language?

If you've built a backend before, you've almost certainly used SQL, even if you never typed it. An
**[ORM](../glossary.md#orm)** or query builder writes the SQL for you: in JavaScript,
`prisma.friend.findMany()` (Prisma) or `Friend.findAll()` (Sequelize); in Python,
`Friend.objects.all()` (Django) or `session.query(Friend).all()` (SQLAlchemy); in Java,
`friendRepository.findAll()` (Spring Data JPA on top of Hibernate); in Go, `db.Find(&friends)`
(GORM). Every one of them sends Postgres a `SELECT … FROM friends` like the one in Step 4. The same
goes for "create" methods, which become an `INSERT`, and for *migrations* (the files that set up
your tables), which run a `CREATE TABLE`. The SQL was always there.

This book teaches the SQL first, on purpose. In Part A3 you'll use SeaORM, Rust's ORM, and when a
query is slow or returns the wrong rows, you'll be able to read the SQL it wrote and see why.

If you know spreadsheets rather than code: a table is a sheet, a column is a header, and a row is
a line. The differences are that the database checks every value against its column's type, lets
many people change it at the same time safely, and is driven by typed SQL instead of clicks.

## Common mistakes

**Putting text in double quotes.**
Many languages let you write text in double quotes. SQL doesn't:

```sql
{{#include ../../code/sql/what-is-a-database/70-double-quoted-text.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/70-double-quoted-text.out}}
```

In SQL, double quotes mean a **name**: of a column, or of a table. So Postgres went looking for a
column called `Hello`, found none, and said so. `LINE 1:` shows the line it was reading, and the
`^` points at the exact spot where it got stuck. **Fix:** text always goes in single quotes:
`SELECT 'Hello';`.

**Forgetting the `;` between two statements.**

```sql
{{#include ../../code/sql/what-is-a-database/71-missing-semicolon.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/71-missing-semicolon.out}}
```

Without a `;` after the first line, `psql` sends both lines to Postgres as **one** statement:
`SELECT 'one' AS first SELECT 'two' AS second;`. That isn't valid SQL, and the `^` points at the
second `SELECT`, where the sentence stopped making sense. Notice that you don't even get `one`
back: the whole statement failed. **Fix:** end every statement with `;`.

**Running a file before creating the lesson's database.**

```text
psql: error: connection to server on socket "/var/run/postgresql/.s.PGSQL.5432" failed: FATAL:  database "what_is_a_database" does not exist
```

`psql` couldn't even connect, because the database named after `-d` isn't there yet. (The "socket"
part is how `psql` reaches the server inside the container; you don't need to do anything with
it.) **Fix:** run `docker compose exec db createdb -U postgres what_is_a_database` once (Step 2),
then run the file again. Also check the spelling: underscores, not dashes.

**Leaving out `-T`.**

```text
cannot attach stdin to a TTY-enabled container because stdin is not a terminal
```

Without `-T`, Docker expects you to be typing at a keyboard, sees a file instead, and refuses.
**Fix:** use `docker compose exec -T db psql …` whenever you feed a file with `<`. (Without a
file, `docker compose exec db psql -U postgres -d what_is_a_database` opens an interactive prompt
where you type SQL yourself. That's the next lesson.)

## More examples

Run each file with the same command as in Step 3, changing only the file name. They all use the
`what_is_a_database` database.

### Maths and joining text

`SELECT` can calculate, and `||` joins pieces of text into one.

```sql
{{#include ../../code/sql/what-is-a-database/50-math-and-text.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/50-math-and-text.out}}
```

`*` is multiply. `||` (two vertical bars) glues text together, left to right, so three pieces
become `Hello, Ada!`. The space after the comma is inside the quotes, which is why it survives.

### Several values at once

Separate values with commas, and each becomes its own column in one row.

```sql
{{#include ../../code/sql/what-is-a-database/51-several-values.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/51-several-values.out}}
```

Three values, three columns, one row. Each `AS` names its own column. `36` has no quotes, so it's a
number, and `psql` lines numbers up on the right, text on the left.

### A second table

A database can hold as many tables as you like. This one is for pets, with three rows.

```sql
{{#include ../../code/sql/what-is-a-database/52-pets.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/52-pets.out}}
```

The same pattern as `friends`: start clean, create, insert, select. `INSERT 0 3` says three rows
went in. `pets` sits next to `friends` in the same database; neither affects the other.

### Only the columns you want

Instead of `*`, name the columns you want back.

```sql
{{#include ../../code/sql/what-is-a-database/53-one-column.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/53-one-column.out}}
```

Only `name` comes back; `city` is still stored, you didn't ask for it. This file reads the table
made in Step 4, so run `02-first-table.sql` first. On a database without `friends`, you get
`ERROR:  relation "friends" does not exist`.

## Your turn

### 🟢 Guided

Run both of this lesson's files in your own database, in order. Fill in the blanks (the database
name, then the file numbers):

```bash
docker compose exec -T db psql -U postgres -d ______ < code/sql/what-is-a-database/__-first-query.sql
docker compose exec -T db psql -U postgres -d ______ < code/sql/what-is-a-database/__-first-table.sql
```

<details><summary>Solution</summary>

```bash
docker compose exec -T db psql -U postgres -d what_is_a_database < code/sql/what-is-a-database/01-first-query.sql
docker compose exec -T db psql -U postgres -d what_is_a_database < code/sql/what-is-a-database/02-first-table.sql
```

The database is `what_is_a_database` (the lesson's name with underscores), and the files are `01`
and `02`: the two-digit number at the front sets the order. The first prints the `greeting` and
`answer` tables from Step 3; the second prints the command tags and the two friends from Step 4.
If `psql` says the database doesn't exist, run the `createdb` command from Step 2 first.

</details>

### 🟡 Tweak

Add **yourself** to `friends`: write a file that inserts one row with your name and your city, then
shows the whole table.

<details><summary>Solution</summary>

`code/sql/what-is-a-database/90-add-me.sql`, here for someone called Mina, from Dhaka:

```sql
{{#include ../../code/sql/what-is-a-database/90-add-me.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/90-add-me.out}}
```

It's Step 4's `INSERT` with one pair of parentheses instead of two, so the tag says `INSERT 0 1`.
There's no `CREATE TABLE` or `DROP TABLE` here: the file adds to the table that
`02-first-table.sql` made (run that first). That also means that if you run this file twice, you're
in the table twice. To start over, run `02-first-table.sql` again: its `DROP TABLE` line gives you
back the two original friends.

</details>

### 🔴 From scratch

Create a table called `books` with two text columns, `title` and `author`. Put two books in it,
then show all of them. Make your file safe to run twice.

<details><summary>Solution</summary>

`code/sql/what-is-a-database/91-books.sql`:

```sql
{{#include ../../code/sql/what-is-a-database/91-books.sql}}
```

```text
{{#include ../../code/sql/what-is-a-database/91-books.out}}
```

It follows `02-first-table.sql` line for line. The `SET` and `DROP TABLE IF EXISTS books;` lines
at the top are what make it safe to run twice: each run throws away the old `books` table and
makes a fresh one, so you always see exactly two books. Leave them out and your second run fails
with `relation "books" already exists`, and the books appear twice.

</details>

## Quick check

<div class="quiz" data-topic="what-is-a-database"></div>

## Remember this

- A **database** keeps data safe, correct and fast to find, even with many users at once. Postgres
  is the database; SQL is the language you talk to it in.
- A **table** is a named list; a **column** is a field every row has; a **row** is one record.
- Text goes in **single quotes** (`'Ada'`); double quotes mean a name. Every statement ends with
  `;`.
- Each Part A1 lesson has its own database. Create it once with
  `docker compose exec db createdb -U postgres <name>` (`already exists` later is harmless), then
  run files with `docker compose exec -T db psql -U postgres -d <name> < file.sql`.
- A file that creates a table starts "clean" with `SET client_min_messages = warning;` and
  `DROP TABLE IF EXISTS …;`, so you can run it again and again.

## Go deeper

- [PostgreSQL tutorial](https://www.postgresql.org/docs/18/tutorial.html) — The official beginner's tour of Postgres.
- [SQL Language](https://www.postgresql.org/docs/18/sql.html) — The full reference for Postgres's SQL.

<!-- next:start -->

**Next:**

- [Tables, rows and psql](../a1-postgres/tables-rows-and-psql.md)

<!-- next:end -->
