# Postgres data types

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can choose a sensible type for numbers, money, text, true/false, dates and IDs.
- You know why money is stored as whole cents.
- You know what NULL means and how to test for it.

## What & why

Integers, exact numbers, text, booleans, dates, timestamps, UUIDs and NULL — and how to choose.

Remember the shopping list in [What is a database?](what-is-a-database.md), where someone typed
`12,99` instead of `12.99` and the file stored it without a word? And in the last lesson, Postgres
refused a `date` column value that wasn't a real day. That refusal came from the column's
[**data type**](../glossary.md#data-type): the kind of value the column holds.

A data type is a **promise the database enforces**. When you write `price_cents integer`, you're
saying "every price in this table is a whole number", and Postgres keeps that promise for you,
forever, for every program that ever writes to the table. A price of `'cheap'` is turned away at
the door, with an error, before it can break anything. Your Rust code, later, can trust what it
reads back.

Picking the type also decides how the value **behaves**: how it's sorted, what maths does to it,
and how exact it is. Some of those behaviours surprise newcomers. `7 / 2` is `3`, not `3.5`. Adding
`0.1` and `0.2` can give `0.30000000000000004`. And an empty box, [**NULL**](../glossary.md#null),
isn't equal to anything, not even to another NULL. This lesson shows each surprise for real, so
none of them surprise you later in an app.

You already met one type decision in
[How a web backend works](../part-0-start/how-a-web-backend-works.md#step-2-a-response-written-out):
the price `1299` was in **cents**, because decimals on a computer aren't always exact. Here you'll
see that problem inside Postgres, and the types that avoid it.

## The idea, slowly

### Step 1: this lesson's database

As in the last two lessons, this lesson gets a database of its own, named after the lesson:
`postgres_data_types`. From the book's folder (the one with `docker-compose.yml`), make sure
Postgres is running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres postgres_data_types
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres postgres_data_types`
- **What:** creates a new, empty database called `postgres_data_types`.
- **Why:** this lesson makes tables called `products` and `events`. In their own database, they can
  never clash with another lesson's tables (the last lesson has an `events` table too).
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  name is the lesson's web address, `postgres-data-types`, with each `-` turned into `_`.
- **Remove it and…** every later command fails with
  `database "postgres_data_types" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres postgres_data_types
```

`createdb` prints **nothing** when it works. If you run it a second time, you see this:

```text
createdb: error: database creation failed: ERROR:  database "postgres_data_types" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: numbers

Postgres has several number types, and they don't all do maths the same way. This file,
`code/sql/postgres-data-types/01-numbers.sql`, asks Postgres four small sums, each chosen to show
one surprise:

```sql
{{#include ../../code/sql/postgres-data-types/01-numbers.sql}}
```

### Line by line

`SELECT 7 / 2 AS whole_numbers,`
- **What:** divides 7 by 2, and names the answer's column `whole_numbers`.
- **Why:** to show that dividing two whole numbers gives a whole number.
- **How:** `7` and `2` are written without a decimal point, so Postgres treats both as `integer`,
  its type for whole numbers. `integer` divided by `integer` gives an `integer`: Postgres keeps
  only the whole part and throws the rest away. `AS` names the column, as in the last two lessons.
- **Remove it and…** (`AS whole_numbers`) the answer still comes back, under the made-up heading
  `?column?`.

`7 / 2.0 AS with_decimals;`
- **What:** the same division, but with `2.0` instead of `2`.
- **Why:** to show the fix: one decimal number is enough to get a decimal answer.
- **How:** a number written with a decimal point, like `2.0`, is a `numeric`: Postgres's type for
  **exact** decimal numbers. When one side of a sum is `numeric`, the whole sum is done in
  `numeric`, so nothing is thrown away.
- **Remove it and…** (the `.0`) you're back to `7 / 2`, and the answer is `3`.

`SELECT 0.1::double precision + 0.2::double precision AS floating_point,`
- **What:** adds 0.1 and 0.2 as `double precision` numbers.
- **Why:** `double precision` is Postgres's **floating-point** type: the same fast, approximate
  kind of number that JavaScript, Python, Java, Go and Rust use by default for decimals (`f64` in
  Rust). This line shows its famous rounding error.
- **How:** `::` is the [**cast**](../glossary.md#cast) operator (an operator is a symbol that does
  something to values, like `+` or `/`): "turn the value on the left into
  the type on the right". `0.1::double precision` means "0.1, as a floating-point number". The type
  name really is two words, `double precision`.
- **Remove it and…** (both `::double precision` casts) `0.1` and `0.2` stay `numeric`, and the
  answer is exactly `0.3`, like the next line.

`0.1::numeric + 0.2::numeric AS exact_numeric;`
- **What:** the same sum, done in `numeric`.
- **Why:** to put the exact answer right next to the approximate one.
- **How:** the casts to `numeric` make the type visible on the page. (`0.1` on its own is already
  a `numeric`, so here the casts change nothing; they're there for you to read.)
- **Remove it and…** (the casts) the answer is the same, `0.3`, but you can no longer see *why* at
  a glance.

`SELECT 2147483647 + 1 AS too_big_for_integer;`
- **What:** adds 1 to 2,147,483,647.
- **Why:** 2,147,483,647 (about 2.1 billion) is the **largest** value an `integer` can hold. This
  line shows what happens when you go past it.
- **How:** both numbers are `integer`, so the answer must be an `integer` too, and 2,147,483,648
  doesn't fit. Postgres refuses with an error instead of giving a wrong answer.
- **Remove it and…** the error disappears from the output; nothing else changes.

`SELECT 2147483647::bigint + 1 AS fits_in_bigint;`
- **What:** the same sum, but with the first number cast to `bigint`.
- **Why:** `bigint` is the bigger whole-number type. It goes up to 9,223,372,036,854,775,807
  (about 9.2 quintillion), so 2,147,483,648 fits easily.
- **How:** `bigint` plus `integer` is done as `bigint`, the bigger of the two.
- **Remove it and…** (the `::bigint`) you get the line above again, and the same error.

### Run it

Every file in this lesson runs the same way: feed it to `psql` in this lesson's database.

```bash
docker compose exec -T db psql -U postgres -d postgres_data_types < code/sql/postgres-data-types/01-numbers.sql
```

```text
{{#include ../../code/sql/postgres-data-types/01-numbers.out}}
```

One answer per `SELECT`, from the top:

- `whole_numbers` is `3`, and `with_decimals` is `3.5000000000000000`. That's exactly 3.5; the
  zeros after it are digits Postgres worked out and kept. Here's why there are so many: when
  `numeric` divides, it can't know how many decimal places you want (1 ÷ 3 never ends), so it
  calculates at least 16 digits and shows every one it calculated, trailing zeros included.
  They're not an error, only precision you didn't ask for. *More examples* shows how to pick the
  number of decimal places yourself.
- `floating_point` is `0.30000000000000004`, while `exact_numeric` is `0.3`. A floating-point
  number is stored in binary (ones and zeros), and most decimals, 0.1 among them, have no exact
  binary form, the same way ⅓ has no exact decimal form (0.333…). The tiny leftover shows up in the
  sum. `numeric` stores decimal digits as they are, so it's exact.
- `ERROR:  integer out of range` is the third `SELECT`. It has no table: the statement failed, so
  there is no answer. `psql` reports the error and goes on to the next statement in the file.
- `fits_in_bigint` is `2147483648`: the same sum, done in a type that's big enough.

`(1 row)` under each table is the number of rows in that answer: each of these `SELECT`s makes
exactly one row.

- ✅ If you see `3`, `3.5000000000000000`, `0.30000000000000004`, `0.3`, the `integer out of range`
  error and `2147483648`, you're right: every surprise showed up.
- ❌ If you see `FATAL:  database "postgres_data_types" does not exist`, go back to Step 1 and run
  `createdb`.

### Step 3: text, true/false and dates

Numbers are one kind of value. Most tables also hold names, yes/no answers and dates. This file,
`code/sql/postgres-data-types/02-text-bool-dates.sql`, tries each one:

```sql
{{#include ../../code/sql/postgres-data-types/02-text-bool-dates.sql}}
```

### Line by line

`SELECT 'Rust' || ' ' || 'book' AS joined_text,`
- **What:** glues three pieces of text together: `Rust`, a space, and `book`.
- **Why:** building text from pieces (a full name from a first and last name, say) is an everyday
  job.
- **How:** `||` (two vertical bars) is SQL's "join text" operator. Text values are `text`, in
  single quotes, as in the last lessons.
- **Remove it and…** (the `' ' ||` in the middle) the answer is `Rustbook`, with no space.

`length('héllo') AS letters;`
- **What:** counts the letters in `héllo`.
- **Why:** to show that `text` counts **letters**, even letters with accents.
- **How:** `length` is a function: a name followed by parentheses, which takes a value and gives
  back an answer. `é` counts as one letter, although the computer needs more room to store it than
  a plain `e`.
- **Remove it and…** you don't see the count; nothing else changes.

`SELECT true AS in_stock,`
- **What:** the value `true`.
- **Why:** yes/no facts (in stock? paid? admin?) are `boolean` values: `true` or `false`.
- **How:** `true` and `false` are SQL words, written without quotes.
- **Remove it and…** you don't see how `psql` shows a boolean (next section).

`DATE '2026-09-24' AS published,`
- **What:** the date 24 September 2026.
- **Why:** a `date` is a day on the calendar, with no time of day.
- **How:** writing a type name **before** a value in quotes, `DATE '…'`, says "this text is a date".
  It's another way to cast, and means the same as `'2026-09-24'::date`.
- **Remove it and…** (the word `DATE`, in both places) Postgres no longer knows the quotes hold a
  date. For `+ 14` it guesses you meant a number, and the statement fails with
  `invalid input syntax for type integer: "2026-09-24"`.

`DATE '2026-09-24' + 14 AS due_back;`
- **What:** the date 14 days after 24 September 2026.
- **Why:** "a library book is due back in two weeks" is date maths every app needs.
- **How:** adding a whole number to a `date` moves it forward that many days. Postgres knows
  September has 30 days, so the answer rolls over into October.
- **Remove it and…** (`+ 14`) the answer is the same date, 2026-09-24.

`SELECT TIMESTAMPTZ '2026-09-24 10:30:00+06' AS dhaka_time_shown_in_utc;`
- **What:** a single **moment**: 10:30 in the morning on 24 September 2026, in Dhaka.
- **Why:** things that happen at a moment (an order placed, a user signing in) need a date **and** a
  time **and** a time zone. `timestamptz` holds all three. (The name is short for "timestamp with
  time zone".)
- **How:** `+06` is Dhaka's time zone: 6 hours ahead of UTC, the world's reference time. Postgres
  stores the exact moment, then **shows** it in the time zone of your **session** (your one
  connection to the database). The book's Postgres uses UTC.
- **Remove it and…** (the `+06`) Postgres reads the time in the session's time zone, UTC here, so
  you'd store 10:30 in UTC: a moment six hours later than 10:30 in Dhaka.

### Run it

```bash
docker compose exec -T db psql -U postgres -d postgres_data_types < code/sql/postgres-data-types/02-text-bool-dates.sql
```

```text
{{#include ../../code/sql/postgres-data-types/02-text-bool-dates.out}}
```

- `joined_text` is `Rust book`, and `letters` is `5`: h, é, l, l, o.
- `in_stock` shows as `t`. That's how `psql` prints `true`; `false` prints as `f`. The value is a
  real boolean; `t` is only how it's shown.
- `published` is `2026-09-24`, and `due_back` is `2026-10-08`: 14 days later, in the next month.
- `dhaka_time_shown_in_utc` is `2026-09-24 04:30:00+00`. It's the same moment you typed, shown in
  UTC: 10:30 in Dhaka is 04:30 in UTC, six hours earlier. The `+00` on the end says "this is UTC,
  zero hours ahead". Nothing was lost; only the clock it's shown on changed.

- ✅ If you see `Rust book`, `5`, `t`, `2026-10-08` and `2026-09-24 04:30:00+00`, you're right.
- ❌ If your timestamp shows a different hour and a different ending (like `+01`), your session
  uses another time zone. That happens if you connect with a different tool; it's the same moment,
  shown on a different clock.

### Step 4: a table that uses the types

Now put the types to work in a table of shop products. This is
`code/sql/postgres-data-types/03-typed-table.sql`:

```sql
{{#include ../../code/sql/postgres-data-types/03-typed-table.sql}}
```

### Line by line

`SET client_min_messages = warning;`
- **What:** tells Postgres to show only warnings and errors, not small notes.
- **Why:** the `DROP TABLE IF EXISTS` line prints a note on the very first run only. Hiding notes
  makes every run look the same.
- **How:** the setting lasts for this one connection.
- **Remove it and…** the first run also prints
  `NOTICE:  table "products" does not exist, skipping`. Harmless, but different from the book.

`DROP TABLE IF EXISTS products;`
- **What:** deletes the `products` table, and every row in it, if it exists.
- **Why:** so you can run this file again and start clean.
- **How:** `IF EXISTS` means "if there's no such table, skip it quietly".
- **Remove it and…** a second run fails with `ERROR:  relation "products" already exists`.

`CREATE TABLE products (` … `);`
- **What:** creates the `products` table with six columns, each with a type.
- **Why:** each column's type is the promise from *What & why*, written down.
- **How:** as in the last lesson: one `name type` pair per line, commas between them, none after
  the last.
- **Remove it and…** every `INSERT` below fails with `relation "products" does not exist`.

`id          bigint,`
- **What:** the product's number, as a `bigint`.
- **Why:** ids only grow. A busy shop, or a table of page views, can pass 2.1 billion rows, and
  then an `integer` id breaks, as you saw in Step 2. `bigint` takes a little more room per row and
  never runs out in practice.
- **How:** it holds whole numbers up to about 9.2 quintillion.
- **Remove it and…** products have no number to refer to them by.

`name        text,`
- **What:** the product's name, as `text`.
- **Why:** `text` holds text of any length. It's the right choice for almost every piece of text
  (see *You might be wondering…* for `varchar`).
- **How:** as in the last lessons.
- **Remove it and…** you can't tell what product a row is.

`price_cents integer,`
- **What:** the price, as a whole number of **cents**: `1299` means $12.99.
- **Why:** whole numbers are always exact, so totals always add up to the cent. The column's name
  says the unit, so nobody reads `1299` as $1,299.
- **How:** an `integer` is plenty here: its limit is over 21 million dollars, in cents.
- **Remove it and…** (the `_cents` in the name) the column still works, but the next person to read
  `1299` has to guess what it means.

`in_stock    boolean,`
- **What:** whether the product is in stock: `true` or `false`.
- **Why:** a yes/no fact is a `boolean`. Text like `'yes'`, `'Y'` or `'in stock'` invites typos, and
  Postgres can't check them.
- **How:** stores `true`, `false`, or nothing at all (NULL, below).
- **Remove it and…** the shop can't say what's available.

`added_on    date,`
- **What:** the day the product was added to the shop.
- **Why:** it's a day, not a moment, so `date` fits: nobody needs to know it was added at 14:07.
- **How:** written `'YYYY-MM-DD'` in quotes, as in the last lesson.
- **Remove it and…** you don't know which products are new.

`sku         uuid`
- **What:** the product's stock code (SKU, "stock-keeping unit"), as a `uuid`.
- **Why:** a **UUID** ("universally unique identifier") is a very large number that looks random,
  written as 32 characters from `0`–`9` and `a`–`f`, in five groups joined by `-`. They're so
  unlikely to repeat that any computer can make one without asking anybody, and nobody can guess
  the next one. That makes them good **public** IDs, for addresses like `/products/a0eebc99-…`,
  where a plain `42` would invite someone to try `43`.
- **How:** the `uuid` type checks the format and stores it compactly. (Postgres can make a new one
  for you with the function `gen_random_uuid()`; the book uses a fixed one here, so your output
  matches.) No comma after it: it's the last column.
- **Remove it and…** the products have no public code.

`INSERT INTO products VALUES (1, 'Mug', 1299, true, '2026-09-01', 'a0eebc99-…');`
- **What:** adds the Mug: id `1`, $12.99, in stock, added on 1 September, with a stock code.
- **Why:** a row where every value fits its column's type.
- **How:** there's no column list this time. With no list, Postgres fills the columns in the
  table's own order, so the six values must come in exactly that order. It keeps these lines short;
  the last lesson explains why naming the columns is safer in real code.
- **Remove it and…** (the whole line) there's no Mug, and the first answer below has one row.

`INSERT INTO products VALUES (2, 'Poster', 'cheap', true, '2026-09-02', NULL);`
- **What:** tries to add a Poster whose price is the word `'cheap'`.
- **Why:** to watch the promise being kept: `'cheap'` is not a whole number.
- **How:** Postgres checks every value against its column's type **before** storing anything. One
  bad value, and the whole row is refused.
- **Remove it and…** the error disappears from the output. The table is the same either way,
  because this row never goes in.

`INSERT INTO products VALUES (3, 'Pen', 199, NULL, NULL, NULL);`
- **What:** adds a Pen for $1.99, with no stock status, no date and no stock code.
- **Why:** real data has gaps. Maybe nobody has counted the pens yet.
- **How:** `NULL` (an SQL word, no quotes) means **no value: unknown or missing**. It isn't `0`,
  it isn't `false`, and it isn't empty text `''`. Any column of any type can hold it (a later
  lesson shows how to forbid it).
- **Remove it and…** there's no Pen, and the two NULL queries below find nothing.

`SELECT * FROM products;`
- **What:** shows every row, every column.
- **Why:** to see which rows really went in, and what NULL looks like.
- **How:** as in the last lesson.
- **Remove it and…** the rows are still stored; you don't see them.

`SELECT name FROM products WHERE in_stock IS NULL;`
- **What:** the names of the products whose stock status is unknown.
- **Why:** "which products haven't been counted yet?" is a real question.
- **How:** [`WHERE`](../glossary.md#where) keeps only the rows where the condition is **true**. `IS NULL` is SQL's way to
  test "has no value": it's true for the Pen, and false for the Mug.
- **Remove it and…** (the `WHERE …` part) you get every product's name.

`SELECT name FROM products WHERE in_stock = NULL;`
- **What:** looks like the same question, written with `=`.
- **Why:** it's the most common NULL mistake, so you should see it fail once.
- **How:** NULL means "unknown", so "is the Pen's unknown value equal to unknown?" has the answer
  "unknown", not "true". `WHERE` keeps only rows where the answer is true, so it keeps none.
- **Remove it and…** nothing is lost: this line never finds anything.

### Run it

```bash
docker compose exec -T db psql -U postgres -d postgres_data_types < code/sql/postgres-data-types/03-typed-table.sql
```

```text
{{#include ../../code/sql/postgres-data-types/03-typed-table.out}}
```

From the top:

- `SET`, `DROP TABLE` and `CREATE TABLE` are **command tags**: `psql` reporting what each statement
  did. `INSERT 0 1` means one row went in (the last number is the row count; the `0` is always
  `0`). The Mug is in.
- `ERROR:  invalid input syntax for type integer: "cheap"` is the Poster. The message names the
  type (`integer`) and the value it refused. `LINE 1:` repeats the start of the statement, and the
  `^` points at `'cheap'`. (`psql` cuts long lines short and marks the cut with `...`.)
- The next `INSERT 0 1` is the Pen.
- `SELECT *` shows **two** rows: the Mug and the Pen. The Poster is not there, because its row was
  refused. Notice the Pen's empty boxes: `psql` shows NULL as **nothing at all**.
- `IS NULL` finds `Pen`: `(1 row)`.
- `= NULL` finds nothing: `(0 rows)`, even though the Pen's `in_stock` is NULL.

- ✅ If you see the `"cheap"` error, two rows in the table, `Pen` for `IS NULL` and `(0 rows)` for
  `= NULL`, you're right.
- ✅ Run it a second time: the output is identical, thanks to the `DROP TABLE` line.

### Step 5: which type do I pick?

You've now seen every type this book uses day to day. When you design a table, ask "what am I
storing?" and read across:

```text
What you're storing        Type to pick
-------------------        ------------------------------------------
A whole number             integer, or bigint if it may pass 2 billion
Money                      integer holding cents, or numeric
Text                       text
Yes or no                  boolean
A day on the calendar      date
A moment in time           timestamptz
A public ID                uuid
```

### Line by line

`A whole number → integer, or bigint if it may pass 2 billion`
- **What:** counts, quantities, ages, ids.
- **Why:** whole numbers are exact and fast.
- **How:** `integer` stops at about 2.1 billion; `bigint` at about 9.2 quintillion. Pick `bigint`
  for ids and anything that keeps growing.
- **Remove it and…** (use `numeric` for everything) it works, but it's slower and hides what the
  column means.

`Money → integer holding cents, or numeric`
- **What:** prices, balances, totals.
- **Why:** money must be exact. `integer` cents is exact and matches the book's JSON
  (`"price": 1299`). `numeric` is exact too, for when you need fractions of a cent (currency
  exchange, tax rates).
- **How:** name the column with its unit, like `price_cents`.
- **Remove it and…** (use `double precision`) you get `0.30000000000000004`-style errors in
  someone's bill.

`Text → text`
- **What:** names, emails, descriptions, anything made of letters.
- **Why:** `text` has no length limit to trip over.
- **How:** in single quotes.
- **Remove it and…** (store text in a number column) Postgres refuses it, like `'cheap'`.

`Yes or no → boolean`
- **What:** in stock, paid, admin, done.
- **Why:** only `true` and `false` fit, so no typos get in.
- **How:** `psql` shows them as `t` and `f`.
- **Remove it and…** (use text) you end up with `'yes'`, `'Yes'` and `'Y'` in one column.

`A day on the calendar → date`
- **What:** birthdays, due dates, the day something was added.
- **Why:** Postgres checks it's a real day and can do day maths with it.
- **How:** write it `'YYYY-MM-DD'`.
- **Remove it and…** (use `timestamptz`) you store a time of day nobody meant, and time zones can
  move it onto a different day.

`A moment in time → timestamptz`
- **What:** when something happened: an order placed, a login, a message sent.
- **Why:** it stores the exact moment, correct for every time zone.
- **How:** write the time zone on the end, like `+06`.
- **Remove it and…** (use `date`) you lose the time of day.

`A public ID → uuid`
- **What:** an ID that appears in web addresses or is shown to users.
- **Why:** it can't be guessed or counted.
- **How:** 32 characters from `0`–`9` and `a`–`f`, in five groups.
- **Remove it and…** (use the `bigint` id in public addresses) anyone can walk through your data by
  counting `1`, `2`, `3`…

## You might be wondering…

**"Why does `7 / 2` give 3?"**
Because both numbers are `integer`, and an `integer` answer can't hold `.5`, so Postgres drops it
(it cuts it off; it doesn't round: `9 / 5` is `1`, not `2`). This is deliberate, and many
programming languages, Rust among them, do the same with whole numbers. When you want the decimals,
make one side a decimal: `7 / 2.0`, or cast it, `7::numeric / 2`.

**"`timestamp` or `timestamptz`?"**
Always `timestamptz`. Postgres also has a plain `timestamp` ("timestamp without time zone"), and it
looks almost the same, but it **throws away** the time zone you give it:

```sql
{{#include ../../code/sql/postgres-data-types/54-timestamp-vs-timestamptz.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/54-timestamp-vs-timestamptz.out}}
```

The plain `timestamp` kept `10:30:00` and dropped the `+06`: it no longer knows *whose* 10:30 it
was, so it can't be compared correctly with a time from London or New York. `timestamptz` turned
Dhaka's 10:30 into the exact moment, and showed it in the session's time zone, UTC here:
`04:30:00+00`. Same moment, different clock. Your Rust app will read it back as that exact moment
too.

**"Why not a float for money?"**
Because floats (`double precision`, and `f64` in Rust) are approximate. Step 2 showed
`0.1 + 0.2` giving `0.30000000000000004`. Add up thousands of prices and the tiny errors pile up;
compare two totals with `=` and they can come out "not equal" when they should be. With an
`integer` of cents, `10 + 20` is exactly `30`, every time. That's the reason for the cents in
[How a web backend works](../part-0-start/how-a-web-backend-works.md#step-2-a-response-written-out).

**"What is NULL: zero? An empty string?"**
Neither. `0` is a number, and `''` (empty text) is text with no letters; both are real values.
NULL is **no value**: unknown, or missing. A Pen with `in_stock` NULL doesn't mean "not in
stock"; it means "nobody knows". That's why `= NULL` never matches (*Common mistakes* shows it
again), and why `psql` shows it as an empty box. Rust has the same idea with a different name: a
NULL column becomes an `Option` in Rust, and NULL is `None`.

**"`varchar(n)` or `text`?"**
Use `text`. Many tutorials use `varchar(255)` ("text of at most 255 letters"), a habit from other
databases. In Postgres, `text` and `varchar` are stored the same way and are equally fast; the only
difference is the limit. And 255 is rarely a real rule: it's a number somebody picked. When you do
need a limit (a username of at most 30 letters, say), keep `text` and add a `CHECK` rule to the
column, which you'll meet in *Keys and relations*. Its error message is clearer, and changing the
limit later is easier.

## Coming from another language?

Every ORM maps its field types to these Postgres types, and you've probably picked them without
seeing the SQL:

- **Prisma:** `Int` is `integer`, `BigInt` is `bigint`, `Decimal` is `numeric`, `Float` is
  `double precision`, `String` is `text`, `Boolean` is `boolean`, `DateTime` is a timestamp, and
  `String @db.Uuid` is `uuid`. A `?` after a type (`Int?`) means "may be NULL".
- **Sequelize:** `DataTypes.INTEGER`, `BIGINT`, `DECIMAL`, `TEXT`, `BOOLEAN`, `DATEONLY` (a
  `date`), `DATE` (a timestamp) and `UUID`, with `allowNull` for NULL.
- **Django ORM:** `IntegerField`, `BigIntegerField`, `DecimalField`, `TextField`, `BooleanField`,
  `DateField`, `DateTimeField` and `UUIDField`, with `null=True` for NULL.
- **SQLAlchemy:** `Integer`, `BigInteger`, `Numeric`, `Text`, `Boolean`, `Date`,
  `DateTime(timezone=True)` and `Uuid`, with `nullable=True`.
- **JPA/Hibernate:** Java `int` or `Integer` is `integer`, `long` or `Long` is `bigint`,
  `BigDecimal` is `numeric`, and `@Column(nullable = …)` controls NULL. Java's `int` has the same
  2.1 billion limit as `integer`.
- **GORM:** Go `int32` and `int64` are `integer` and `bigint`, `string` is text, `bool` is
  `boolean`, `time.Time` is a timestamp, and a pointer (`*string`) is how a field says "may be
  NULL".

The money rule is the same everywhere: `Decimal`/`DecimalField`/`BigDecimal` or whole cents, never
`Float`. And in JavaScript every plain number is a float, so `0.1 + 0.2` is `0.30000000000000004`
there too.

In Rust, which you'll use with Postgres from Part A3 on, the matches are close: `integer` ↔ `i32`,
`bigint` ↔ `i64`, `boolean` ↔ `bool`, `text` ↔ `String`, and NULL ↔ `None`. An `i32` has the same
limit as `integer`, because it *is* the same kind of number.

If you know spreadsheets: formatting a column as "Date" or "Number" in a spreadsheet only changes
how cells **look**, and you can still type anything into them. A Postgres type is a lock: `'cheap'`
never gets into a price column. An empty spreadsheet cell is the closest thing to NULL.

## Common mistakes

**Testing for NULL with `=`.**

```sql
{{#include ../../code/sql/postgres-data-types/70-equals-null.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/70-equals-null.out}}
```

There's no error, which makes this mistake sneaky: the first query quietly returns `(0 rows)`,
although the Pen's `in_stock` is NULL. The second query shows why. `NULL = NULL` is **not** true:
it's NULL itself ("is unknown equal to unknown? unknown"), and `psql` prints that as an empty box
under `equals_says`. `NULL IS NULL` is `t`, true. **Fix:** test with `IS NULL`, and its opposite
`IS NOT NULL`: `SELECT name FROM products WHERE in_stock IS NULL;`. (Run `03` first; this file reads
its table.)

**A date in the wrong format.**

```sql
{{#include ../../code/sql/postgres-data-types/71-date-format.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/71-date-format.out}}
```

`24/09/2026` is how much of the world writes 24 September. But the book's Postgres reads dates with
slashes as **month/day/year**, the American order, so it took `24` as the month, and there is no
24th month. That's `date/time field value out of range`, and the `^` points at the value. The
`HINT` mentions `DateStyle`, the setting that chooses that order. Worse, a date where both numbers
are 12 or less doesn't fail at all: `'03/04/2026'` is quietly stored as 4 March, even if you meant
3 April. **Fix:** always write dates year-month-day, `'2026-09-24'`. Postgres reads that one the
same way whatever the setting, and so does everyone else.

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d postgres_data_types < code/sql/postgres-data-types/<file>`.
Some read the `products` table, so run `03` first.

### Exactly two decimal places with `numeric(10,2)`

`numeric(10,2)` means "at most 10 digits in all, 2 of them after the point". Postgres rounds to fit.

```sql
{{#include ../../code/sql/postgres-data-types/50-numeric-rounding.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/50-numeric-rounding.out}}
```

`12.345` rounds up to `12.35`, `12.344` rounds down to `12.34`, and `12.5` gains a zero, `12.50`:
always exactly two decimal places. That's how you'd hold money in `numeric`. The second query
fails: 10 digits with 2 after the point leaves only 8 before it, and `123456789.5` has 9, so
Postgres refuses with `numeric field overflow`, and its `DETAIL` line spells out that limit
(`10^8` is 100,000,000).

### Change the case with `upper()` and `lower()`

Two functions for text: `upper` makes every letter a capital, `lower` makes every letter small.

```sql
{{#include ../../code/sql/postgres-data-types/51-upper-lower.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/51-upper-lower.out}}
```

The table itself is unchanged: `upper` and `lower` build new text for the answer only. `lower` is
handy for emails, where `Ada@Example.com` and `ada@example.com` are the same address.

### Days between two dates

Subtract one `date` from another, and you get the number of days between them, as an `integer`.

```sql
{{#include ../../code/sql/postgres-data-types/52-days-between.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/52-days-between.out}}
```

From 24 September to 25 December is `92` days. The second query counts how long each product has
been on sale on 30 September: the Mug, added on 1 September, `29` days. The Pen's box is empty,
because its `added_on` is NULL, and **any maths with NULL gives NULL**: unknown minus a date is
still unknown.

### A default for NULL with `COALESCE`

`COALESCE(a, b)` gives back `a`, unless `a` is NULL; then it gives `b`.

```sql
{{#include ../../code/sql/postgres-data-types/53-coalesce.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/53-coalesce.out}}
```

The Mug's `in_stock` is `t`, so `COALESCE` keeps `t`. The Pen's is NULL, so `COALESCE` puts
`false` in its place: `f`. Here the shop has decided "unknown means don't sell it". The table still
holds NULL; `COALESCE` only changes the answer. (The name means "come together", and it takes as
many values as you like: it gives back the first one that isn't NULL.)

## Your turn

### 🟢 Guided

Run `01-numbers.sql` (Step 2) again, and for each of its four answers, say in one sentence **why**
it came out that way. Fill in the blanks:

```text
7 / 2 is 3 because both sides are ______.
0.1 + 0.2 as double precision is 0.30000000000000004 because ______.
2147483647 + 1 fails because ______.
2147483647::bigint + 1 works because ______.
```

<details><summary>Solution</summary>

```bash
docker compose exec -T db psql -U postgres -d postgres_data_types < code/sql/postgres-data-types/01-numbers.sql
```

The output is the same as in Step 2. The four reasons:

- `7 / 2` is `3` because both sides are **`integer`**, so the answer is an `integer`, and the `.5`
  is dropped. (`7 / 2.0` is a `numeric` sum, so the answer keeps it: `3.5000000000000000`.)
- `0.1 + 0.2` as `double precision` is `0.30000000000000004` because **floating-point numbers are
  stored in binary, and 0.1 has no exact binary form**. The `numeric` version is exactly `0.3`.
- `2147483647 + 1` fails because **2,147,483,647 is the largest `integer`**, and the answer must be
  an `integer` too: `ERROR:  integer out of range`.
- `2147483647::bigint + 1` works because **the `::` cast makes it a `bigint`**, which goes up to
  about 9.2 quintillion, so `2147483648` fits.

</details>

### 🟡 Tweak

The shop now needs each product's weight, in whole grams. Add a `weight_grams integer` column to
`products`, then add a Notebook (id `4`, $4.50, in stock, added on 20 September 2026, no stock
code) that weighs 200 grams. Show every product's name and weight.

<details><summary>Solution</summary>

`code/sql/postgres-data-types/90-weight.sql`:

```sql
{{#include ../../code/sql/postgres-data-types/90-weight.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/90-weight.out}}
```

`ALTER TABLE … ADD COLUMN` (from the last lesson) adds a seventh column, and prints the tag
`ALTER TABLE`. The price is `450` cents, and the unit is in the column's name again: `_grams`. The
`INSERT` has no column list, so it needs all **seven** values, in the table's order, with the
weight last. The Mug and the Pen have **empty** weights: they were added before the column existed,
so their weight is NULL, unknown, not `0`. Run `03` first to start from its table. If you run this
file twice, the second run reports
`ERROR:  column "weight_grams" of relation "products" already exists` and adds a second Notebook;
run `03` again to start clean.

</details>

### 🔴 From scratch

Design a table called `events` for a meetup site, with five columns: `id` (a whole number that may
grow large), `title` (text), `starts_at` (the moment it starts), `is_free` (yes or no) and
`price_cents` (the ticket price, in cents). Pick a type for each, add two events, one free and one
paid, both in Dhaka time, then show the whole table. Make the file safe to run twice.

<details><summary>Solution</summary>

`code/sql/postgres-data-types/91-events.sql`:

```sql
{{#include ../../code/sql/postgres-data-types/91-events.sql}}
```

```text
{{#include ../../code/sql/postgres-data-types/91-events.out}}
```

Each type comes from Step 5's table: `bigint` for an id that grows, `text` for the title,
`timestamptz` for a moment, `boolean` for yes/no, and `integer` cents for money. The top two lines
make the file safe to run twice. The start times are written in Dhaka time (`+06`), and shown in
UTC: 18:00 in Dhaka is `12:00:00+00`, and 10:00 is `04:00:00+00`. The free event's price is `0`,
not NULL: the price is known, and it's zero.

</details>

## Quick check

<div class="quiz" data-topic="postgres-data-types"></div>

## Remember this

- A data type is a promise Postgres enforces: `'cheap'` never gets into an `integer` column.
- Whole numbers: `integer` (up to about 2.1 billion) or `bigint`. `integer / integer` drops the
  decimals: `7 / 2` is `3`.
- Money: whole cents in an `integer`, or `numeric`. Never `double precision`: `0.1 + 0.2` is
  `0.30000000000000004`.
- Text is `text`, yes/no is `boolean`, a day is `date` (`'YYYY-MM-DD'`), a moment is `timestamptz`,
  a public ID is `uuid`. `::` casts a value to another type.
- NULL means unknown, not `0` or `''`. Test it with `IS NULL`, never `= NULL`.

## Go deeper

- [Data types](https://www.postgresql.org/docs/18/datatype.html) — Every type Postgres has, with its range and rules.

<!-- next:start -->

**Next:**

- [CRUD in SQL](../a1-postgres/crud-in-sql.md)

<!-- next:end -->
