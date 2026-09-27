# Indexes

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can explain what an index is with the back-of-the-book analogy.
- You can read EXPLAIN to see whether Postgres scanned every row.
- You know when an index helps and when it doesn't.

## What & why

Make lookups fast with indexes, read EXPLAIN, and learn when an index isn't worth it.

Pick up a thick book about databases and try to find every page that mentions "Postgres". You could
start at page 1 and read every page to the end, writing down the page numbers as you go. That
works, but it takes an afternoon. Or you could turn to the **index** at the back of the book, find
"Postgres" in its alphabetical list, and read off `12, 88, 204`. Three pages, found in seconds.

A table in Postgres has the same problem. A shop keeps 100,000 orders, and a customer opens "My
orders". Your app asks for the orders `WHERE customer_id = 42`, and there are 20 of them, scattered
through the table. With nothing else to go on, Postgres does the afternoon version: it reads **all
100,000 rows** and keeps the 20 that match. At 100,000 rows that's still quick. At 100 million, and
a hundred customers clicking at once, it's the reason a website feels slow.

An [**index**](../glossary.md#index) is the back-of-the-book list, for one column: every
`customer_id`, in order, each with the addresses of its rows. With it, Postgres finds customer 42 in
the list and goes straight to those 20 rows.

In this lesson you build the 100,000 orders, and you ask Postgres to **show its working**: the
[**EXPLAIN**](../glossary.md#explain) command prints the plan Postgres made for a query, and whether
it read every row. Then you add an index and watch the plan change. And you'll see the part most
tutorials skip: the [**query planner**](../glossary.md#query-planner), the part of Postgres that
chooses **how** to run each query, sometimes decides an index isn't worth using, and it's right.

## The idea, slowly

### Step 1: this lesson's database

As in the last lessons, this lesson gets a database of its own, named after the lesson: `indexes`.
From the book's folder (the one with `docker-compose.yml`), make sure Postgres is running, then
create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres indexes
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres indexes`
- **What:** creates a new, empty database called `indexes`.
- **Why:** this lesson fills a table with 100,000 rows. In a database of its own, it can't get in
  the way of the tables from the last lessons.
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  lesson's web address is `indexes`, with no `-` to turn into `_`, so the name stays `indexes`.
- **Remove it and…** every later command fails with `database "indexes" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres indexes
```

`createdb` prints **nothing** when it works. If you run it a second time, you see this:

```text
createdb: error: database creation failed: ERROR:  database "indexes" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: 100,000 orders

An index only shows what it's worth on a big table, and nobody wants to type 100,000 `INSERT`s.
This file has Postgres **make up** the rows instead. This is `code/sql/indexes/01-setup.sql`:

```sql
{{#include ../../code/sql/indexes/01-setup.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP TABLE IF EXISTS orders;`
- **What:** hides small notes, then deletes the `orders` table if it's there.
- **Why:** this is the lesson's reset button. Running `01` puts you back at the very start, and
  deleting a table deletes every index on it too, so every index you add later goes with it.
- **How:** as in the last lessons. `IF EXISTS` skips the drop on the very first run, and the `SET`
  line hides the note that would say so.
- **Remove it and…** a second run fails with `ERROR:  relation "orders" already exists`.

`CREATE TABLE orders (` … `);`
- **What:** a table of shop orders: an `id`, which customer placed it, its status, and its total.
- **Why:** "show me my orders" is one of the most common questions an app asks its database.
- **How:** the rules you know from the last lessons: an `id` numbered by Postgres, and `NOT NULL`
  on every column. `id` is a `bigint`, the bigger whole-number type from
  [Postgres data types](postgres-data-types.md), because a busy shop can pass 2 billion orders.
  `total_cents` stores money as whole cents, as that lesson recommends.
- **Remove it and…** there's no table, and the `INSERT` fails with
  `relation "orders" does not exist`.

`INSERT INTO orders (customer_id, status, total_cents)` · `SELECT` …
- **What:** adds rows, but instead of a `VALUES` list, the rows come from a `SELECT`.
- **Why:** a `SELECT` can **produce** rows, not only read them from a table. Whatever rows it
  produces, the `INSERT` adds.
- **How:** the `SELECT`'s three answer columns go into the three columns named on the `INSERT`
  line, in order: the first into `customer_id`, the second into `status`, the third into
  `total_cents`.
- **Remove it and…** (the whole `SELECT … FROM …` part) the `INSERT` has nothing to add, and
  Postgres stops with `syntax error at or near ";"`.

`FROM generate_series(1, 100000) AS n;`
- **What:** a made-up "table" of the numbers 1, 2, 3, … up to 100,000, one per row, with the
  column name `n`.
- **Why:** it's a way to make lots of fake rows: 100,000 numbers become 100,000 orders.
- **How:** `generate_series` is a Postgres function that gives back rows instead of one value, so
  you can put it after `FROM`, where a table would go. `AS n` names its one column, so the lines
  above can use `n`.
- **Remove it and…** `n` means nothing, and Postgres stops with `column "n" does not exist`.

`SELECT n % 5000,`
- **What:** the order's `customer_id`: the remainder when `n` is divided by 5,000.
- **Why:** it spreads the orders over 5,000 customers, numbered `0` to `4999`, with exactly 20
  orders each.
- **How:** `%` is the **remainder** after dividing, as in school: `42 % 5000` is `42`, `5042 % 5000`
  is `42` too, and so is `10042 % 5000`. So customer 42 gets orders 42, 5042, 10042, …, 95042: 20
  orders, spread evenly through the table. That spreading matters later.
- **Remove it and…** (the line) the `INSERT` names three columns but the `SELECT` gives only two,
  and Postgres refuses with `INSERT has more target columns than expressions`.

`CASE WHEN n % 10 = 0 THEN 'refunded' ELSE 'paid' END,`
- **What:** the order's `status`: every tenth order is `'refunded'`, the rest are `'paid'`.
- **Why:** real shops have a few refunds among many sales. Here it's 10,000 refunded and 90,000
  paid, and that imbalance is the point of Step 5.
- **How:** `CASE` is SQL's "if": **when** the test is true, **then** this value, **else** that one,
  and `END` closes it. `n % 10 = 0` is true when `n` divides by 10 with nothing left over: 10, 20,
  30, …
- **Remove it and…** as the line above: `INSERT has more target columns than expressions`.

`(n * 37) % 10000`
- **What:** the order's `total_cents`, a made-up price from 0 to 9,999 cents.
- **Why:** so the prices look mixed up rather than counting 1, 2, 3.
- **How:** multiplying by 37 and keeping the remainder jumbles the order. It works out so that each
  price between 0 and 9,999 appears exactly 10 times (*Your turn* uses that).
- **Remove it and…** as the lines above: `INSERT has more target columns than expressions`.

`ANALYZE orders;`
- **What:** tells Postgres to look through `orders` and note what's in it.
- **Why:** so it can plan well. Before it chooses how to run a query, the query planner wants to
  know things like "how many rows does this table have?" and "how common is `'paid'`?"
- **How:** Postgres reads a sample of the rows and stores a summary, called **statistics**: roughly
  how many rows there are, how many different values each column has, and which values are most
  common. It doesn't change your data.
- **Remove it and…** Postgres collects statistics on its own a little later, in the background.
  Until it does, it plans from rough guesses, and your `EXPLAIN` output could differ from the book's.

`SELECT count(*) AS orders FROM orders;`
- **What:** counts the rows.
- **Why:** to check all 100,000 went in.
- **How:** `count(*)` from [CRUD in SQL](crud-in-sql.md).
- **Remove it and…** the rows are still there; you don't see the count.

### Run it

Every file in this lesson runs the same way: feed it to `psql` in this lesson's database.

```bash
docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/01-setup.sql
```

```text
{{#include ../../code/sql/indexes/01-setup.out}}
```

`INSERT 0 100000` means a hundred thousand rows went in, from one statement. `ANALYZE` is the
command tag of the `ANALYZE` line: it prints nothing else. The count confirms it: `100000`.

- ✅ If you see `INSERT 0 100000` and a count of `100000`, you're right. It may take a second or
  two.
- ✅ Run it a second time: the output is identical, because the `DROP TABLE` line starts over.
- ❌ If you see `FATAL:  database "indexes" does not exist`, go back to Step 1.

### Step 3: EXPLAIN — ask Postgres how it found the rows

Now the question from *What & why*: customer 42's orders. You could run the `SELECT` and get 20
rows back, but that wouldn't tell you **how** Postgres found them. Put `EXPLAIN` in front, and
Postgres shows you its plan instead. This is `code/sql/indexes/02-without-index.sql`:

```sql
{{#include ../../code/sql/indexes/02-without-index.sql}}
```

### Line by line

`EXPLAIN`
- **What:** "don't give me the rows; tell me how you'd get them."
- **Why:** it's the only way to see whether Postgres read the whole table or went straight to the
  rows. Two queries can give the same answer, one a thousand times slower than the other.
- **How:** before running any query, the query planner considers the ways it could run it (read
  the whole table, use this index, use that one) and picks the one it thinks is fastest. That
  choice is the **plan**. `EXPLAIN` prints it.
- **Remove it and…** you get the 20 rows themselves, and no idea how they were found.

`(ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)`
- **What:** options for `EXPLAIN`, in brackets. Together they mean "run it for real, but hide the
  numbers that change every run."
- **Why:** `ANALYZE` makes the plan show what **really** happened, such as how many rows each step
  found, not only what Postgres expected. The four `OFF`s remove guesses and timings that would
  differ on your computer, on mine, and on the next run.
- **How:** `ANALYZE` here runs the query and counts the rows as they go by (it is not the
  `ANALYZE orders;` statement from Step 2, which gathers statistics; same word, different job).
  `COSTS OFF` hides the planner's estimates, `TIMING OFF` hides the milliseconds, `SUMMARY OFF`
  hides the total time at the end, and `BUFFERS OFF` hides how much of the table was already in
  memory.
- **Remove it and…** (the whole bracket) you see the plan with the planner's estimated costs and
  row counts, but without what really happened.

`SELECT * FROM orders WHERE customer_id = 42;`
- **What:** the query being explained: every order of customer 42.
- **Why:** it's the "My orders" page from *What & why*.
- **How:** a plain `SELECT`, as in [CRUD in SQL](crud-in-sql.md). With `ANALYZE`, Postgres runs it
  and throws the 20 rows away, keeping only the plan.
- **Remove it and…** `EXPLAIN` has nothing to explain, and Postgres stops with
  `syntax error at or near ";"`.

### Run it

```bash
docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/02-without-index.sql
```

```text
{{#include ../../code/sql/indexes/02-without-index.out}}
```

A plan comes back as a small table with one column, `QUERY PLAN`, one line per row, so `(3 rows)`
counts plan lines, not orders. Read it top to bottom:

- `Seq Scan on orders` is the **step** Postgres used. *Seq* is short for *sequential*: it read the
  table from the first row to the last, in order. That's the afternoon version from *What & why*.
- `(actual rows=20.00 loops=1)` is what really happened: the step produced **20** rows, and ran
  **once** (`loops=1`). Postgres 18 prints row counts with two decimals, because when a step runs
  several times, the number is an average; here it's a whole 20.
- `Filter: (customer_id = 42)` is the test applied to each row it read: your `WHERE`.
- `Rows Removed by Filter: 99980` is the bad news. Postgres read every row, and **threw away
  99,980** of them to keep 20. 20 + 99,980 = 100,000: the whole table.

This file only reads, so you can run it as often as you like.

- ✅ If you see `Seq Scan on orders`, `actual rows=20.00` and `Rows Removed by Filter: 99980`,
  you're right.
- ❌ If you see `Bitmap Heap Scan` instead, the table already has the index from Step 4: run `01`
  to start over, then this file.

### Step 4: add an index

Here's the idea of the index you're about to build, drawn as the back of a book. Each
`customer_id` appears once, in order, with the addresses of its rows in the table:

```text
customer_id   where its rows are
-----------   --------------------------------
       0      → 20 row addresses
       1      → 20 row addresses
       …
      42      → 20 row addresses   ◀ WHERE customer_id = 42
       …
    4999      → 20 row addresses
```

### Line by line

`customer_id   where its rows are`
- **What:** the index's two parts: the value it's sorted by, and where to find the rows.
- **Why:** it's the book's index: a word, then its page numbers.
- **How:** the index is stored separately from the table. The table itself stays in whatever order
  the rows arrived; only the index is sorted.
- **Remove it and…** (the second part) the index could tell you customer 42 exists, but not where
  its orders are.

`0 → 20 row addresses` · `1 → 20 row addresses`
- **What:** the smallest customer numbers come first.
- **Why:** because the list is **sorted**, Postgres doesn't read it from the top. It jumps into the
  middle, sees whether 42 is before or after, jumps again, and lands on 42 in a handful of steps,
  the way you'd find "Postgres" in a book's index without reading from "A".
- **How:** Postgres's usual kind of index, a **B-tree**, keeps the values sorted in a tree-shaped
  structure made for these jumps, and keeps it sorted as rows are added.
- **Remove it and…** (the sorting) finding 42 would mean reading the list from the top, which is
  no better than reading the table.

`42 → 20 row addresses ◀ WHERE customer_id = 42`
- **What:** the entry your query needs: customer 42, and where its 20 orders live.
- **Why:** from here, Postgres reads only those 20 rows, and none of the other 99,980.
- **How:** an address says which **page** of the table's storage the row is on, and where on that
  page. Postgres stores a table in pages of 8 kB each.
- **Remove it and…** there's no way to jump to customer 42, and you're back to the `Seq Scan`.

`…` · `4999 → 20 row addresses`
- **What:** the rest of the list, up to the biggest customer number.
- **Why:** an index covers **every** row of the table, not only the ones you've asked about.
- **How:** 5,000 entries, one per customer, 100,000 addresses in all.
- **Remove it and…** (an entry) that customer's orders would be missing from any answer that used
  the index, so Postgres never lets that happen: it updates the index on every write.

This is `code/sql/indexes/03-with-index.sql`:

```sql
{{#include ../../code/sql/indexes/03-with-index.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP INDEX IF EXISTS orders_customer_id_idx;`
- **What:** hides small notes, then deletes the index if it's already there.
- **Why:** so you can run this file twice. The second time, the index exists, and creating it again
  would fail.
- **How:** `DROP INDEX` deletes an index, and only the index: the table and its rows aren't
  touched. `IF EXISTS` skips it quietly the first time.
- **Remove it and…** a second run fails with `ERROR:  relation "orders_customer_id_idx" already
  exists`.

`CREATE INDEX orders_customer_id_idx ON orders (customer_id);`
- **What:** builds an index on the `customer_id` column of `orders`.
- **Why:** this is the index from the picture above.
- **How:** `CREATE INDEX`, then a name you choose, then `ON` the table, and the column in brackets.
  Postgres reads the whole table once, sorts the values, and stores the list. From now on, every
  `INSERT`, `UPDATE` and `DELETE` on `orders` keeps it up to date. The name follows a common habit:
  table, column, then `idx`.
- **Remove it and…** the plan below is the same `Seq Scan` as in Step 3.

`EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)` · `SELECT * FROM orders WHERE customer_id = 42;`
- **What:** the very same `EXPLAIN` as Step 3.
- **Why:** same question, so the only difference in the answer is the index.
- **How:** as in Step 3. Notice that the query doesn't mention the index. You never tell Postgres
  to use one: the planner sees it exists and decides for itself.
- **Remove it and…** you've built the index, and you can't see whether it helped.

### Run it

```bash
docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/03-with-index.sql
```

```text
{{#include ../../code/sql/indexes/03-with-index.out}}
```

`DROP INDEX` and `CREATE INDEX` are the command tags of those two lines. (`DROP INDEX` shows up even
the first time, when there was nothing to drop.) Then the new plan. It has two steps now, and the
`->` means "this step feeds the one above it", so read the **indented** one first:

- `Bitmap Index Scan on orders_customer_id_idx` is the step that used your index. It found customer
  42 in the sorted list and collected the 20 addresses (`actual rows=20.00`). *Bitmap* is
  Postgres's word for the checklist it builds from them: "these pages of the table hold matching
  rows".
- `Index Cond: (customer_id = 42)` is the condition it looked up in the index, your `WHERE`. In a
  `Filter:`, Postgres tests rows it has already read; an `Index Cond` is used to **find** rows in
  the first place.
- `Index Searches: 1` means it went into the index once. (One lookup, one value.)
- `Bitmap Heap Scan on orders` is the step above it. *Heap* is Postgres's name for the table's own
  storage, the pages that hold the rows. This step takes the checklist and reads **only** the pages
  on it, in page order.
- `Recheck Cond: (customer_id = 42)` is the condition Postgres would test again on each row if the
  checklist were only approximate. It becomes approximate when a very large number of rows match,
  and Postgres marks a whole page instead of each row.
- `Heap Blocks: exact=20` says it read **20 pages** (*blocks*) of the table, and that the checklist
  was **exact**: it knew exactly which rows to take, so there was nothing to recheck. Customer 42's
  orders are spread through the table, 5,000 rows apart, so each one is on its own page.

And the line that's **gone**: `Rows Removed by Filter`. Before, Postgres read 100,000 rows and threw
away 99,980. Now it read 20 rows, on 20 pages, and threw away none.

- ✅ If you see `Bitmap Index Scan on orders_customer_id_idx`, `Heap Blocks: exact=20` and no
  `Rows Removed by Filter` line, you're right.
- ✅ Run it a second time: the output is identical, because the `DROP INDEX` line starts over.
- ❌ If you see `Seq Scan on orders` instead, check the `CREATE INDEX` line printed `CREATE INDEX`,
  not an error.

On other tables, or with a few rows more or less, you may see an `Index Scan` step instead: it
does the lookup and the page reading in one step. `Index Scan`, `Bitmap Index Scan` and
`Index Only Scan` all mean "an index was used". `Seq Scan` means it wasn't.

### Step 5: when an index doesn't help

If an index made one query faster, why not index `status` too, for "show me the paid orders"?
This is `code/sql/indexes/04-when-not-to-index.sql`:

```sql
{{#include ../../code/sql/indexes/04-when-not-to-index.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP INDEX IF EXISTS orders_status_idx;`
- **What:** hides notes, then deletes this file's index if it's there.
- **Why:** so you can run this file twice.
- **How:** as in `03`.
- **Remove it and…** a second run fails with `ERROR:  relation "orders_status_idx" already exists`.

`CREATE INDEX orders_status_idx ON orders (status);`
- **What:** an index on `status`.
- **Why:** to see what the planner does with it.
- **How:** as in `03`. The index lists two values, `paid` and `refunded`, each with a very long
  list of addresses.
- **Remove it and…** there's no index for the planner to choose, and both plans below are a
  `Seq Scan`.

`ANALYZE orders;`
- **What:** refreshes the statistics, as in `01`.
- **Why:** so the planner knows, freshly, how common each status is. That's the whole decision
  below.
- **How:** as in `01`.
- **Remove it and…** the plans here come out the same: the statistics from `01` already say 90%
  of rows are `'paid'`. It's here as a good habit after big changes.

`EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'paid';`
- **What:** the plan for "all paid orders", **without** running the query.
- **Why:** 90,000 of the 100,000 rows match. What does the planner do with the index?
- **How:** there's no `ANALYZE` in the brackets this time, so Postgres only plans, and shows no
  `actual rows`. `COSTS OFF` still hides the estimates.
- **Remove it and…** you don't see the first plan.

`EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'refunded';`
- **What:** the same, for refunded orders.
- **Why:** 10,000 rows match: far fewer.
- **How:** as the line above.
- **Remove it and…** you don't see the second plan.

`\di`
- **What:** a [backslash command](../glossary.md#backslash-command) that lists the indexes in this
  database ("**d**escribe **i**ndexes").
- **Why:** to see every index on `orders`, including one you never made.
- **How:** like `\dt` for tables, from [Tables, rows and psql](tables-rows-and-psql.md).
- **Remove it and…** you don't see the list.

### Run it

```bash
docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/04-when-not-to-index.sql
```

```text
{{#include ../../code/sql/indexes/04-when-not-to-index.out}}
```

(The `::text` after each value is Postgres saying the value is text: the [cast](../glossary.md#cast)
from [Postgres data types](postgres-data-types.md), added by Postgres itself.)

**For `'paid'`, the planner ignored the index.** `Seq Scan on orders`, as if the index weren't
there. It isn't broken, and Postgres isn't being lazy. Think of the book again: if a word is on 9
pages out of 10, you don't look it up in the index and flip back and forth; you read the book.
Using an index costs something per row: look up the address, then jump to that page. For a handful
of rows that's a bargain. For 90% of the table, all that jumping is **slower** than reading every
page once, in order. The statistics from `ANALYZE` told the planner that 90% of rows are `'paid'`,
so it chose the plain scan.

**For `'refunded'`, it used the index**: `Bitmap Index Scan on orders_status_idx`. Only 10% of rows
match, so the planner expected it to pay off. (*Your turn* asks you to check whether it really
does.)

That's the rule of thumb: **an index pays off when a query matches few rows.** "One customer's
orders" (20 of 100,000) is perfect. "All paid orders" (90,000 of 100,000) isn't.

The `\di` list has three indexes: the two you made, and **`orders_pkey`**, which you didn't.
Postgres builds an index for every `PRIMARY KEY` and every `UNIQUE` rule on its own. It's how it
checks, on every `INSERT`, that the new `id` isn't taken yet, without reading the whole table. Its
name is the table's plus `_pkey`, like the `_key` and `_fkey` names in
[Keys and relations](keys-and-relations.md).

- ✅ If you see `Seq Scan` for `'paid'`, `Bitmap Index Scan on orders_status_idx` for
  `'refunded'`, and three indexes in the list, you're right.
- ❌ If `\di` shows more than three, you've added some of your own. Run `01` to start over, then
  `02`, `03` and `04`.

### Step 6: what an index costs

An index isn't free. It's a second copy of the column's values, plus addresses, and it lives on
disk next to the table. This is `code/sql/indexes/05-what-indexes-cost.sql`:

```sql
{{#include ../../code/sql/indexes/05-what-indexes-cost.sql}}
```

### Line by line

`pg_size_pretty(pg_relation_size('orders')) AS table_size`
- **What:** how much disk space the `orders` table takes.
- **Why:** to compare the indexes against it.
- **How:** `pg_relation_size('orders')` gives the size in bytes, a long number.
  `pg_size_pretty(…)` turns bytes into something readable, like `5200 kB`.
- **Remove it and…** you see the indexes' sizes with nothing to compare them to.

`pg_relation_size('orders_pkey')` · `('orders_customer_id_idx')` · `('orders_status_idx')`
- **What:** the same measurement, for each of the three indexes from `\di`.
- **Why:** every index is stored separately, and takes its own space.
- **How:** `pg_relation_size` accepts the name of a table or of an index. The long lines are
  lined up with spaces only to be easier to read.
- **Remove it and…** (a line) you don't see that index's size.

### Run it

```bash
docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/05-what-indexes-cost.sql
```

```text
{{#include ../../code/sql/indexes/05-what-indexes-cost.out}}
```

The table takes 5200 kB (about 5 MB), and its three indexes together take another 3680 kB: 70% on
top of the table. (`orders_pkey` is the biggest because every `id` is different, so it can't
share entries; `customer_id` and `status` repeat, and Postgres stores a repeated value once.)

Space is the cost you can see. The one you can't see is **time on every write**. Each `INSERT` now
adds a row to the table **and** an entry to all three indexes. Each `DELETE` removes them all. An
`UPDATE` of `status` must also fix `orders_status_idx`. Reading got faster; writing got a little
slower. That's the trade, every time you add an index.

- ✅ If you see four sizes, with `table_size` the biggest, you're right.
- ❌ If you see `relation "orders_status_idx" does not exist`, run `04` first.

## You might be wondering…

**"Why not index every column?"**
Because of Step 6. Every index slows down every write to the table and takes disk space, and
Step 5 showed the planner ignores an index that doesn't narrow things down much. Add an index when
you have a real query that needs it: a `WHERE`, `JOIN` or `ORDER BY` on a column that picks out a
few rows, run often. Then check with `EXPLAIN` that it's used.

**"Does the primary key already have an index?"**
Yes: `orders_pkey`, in the `\di` list of Step 5. Every `PRIMARY KEY` and `UNIQUE` rule gets one
automatically, so `WHERE id = 42` is already fast. Don't add a second index on `id`; it would only
slow down writes.

**"Should foreign key columns be indexed?"**
Usually, yes, and **Postgres doesn't do it for you.** In [Keys and relations](keys-and-relations.md),
`authors.id` has an index (it's a primary key), but `books.author_id` doesn't. "Every book by author
1" reads the whole `books` table, and so does deleting an author: Postgres must check that no book
still points at them. On a small table it doesn't matter. On a big one, add
`CREATE INDEX books_author_id_idx ON books (author_id);`.

**"Why show `actual rows` but no times?"**
Because times change on every run, and on every computer: the book's output would never match
yours. Row counts don't change, and they tell the real story anyway: 100,000 rows read against 20.
On your own, try `EXPLAIN ANALYZE SELECT …` without the other options: you'll see
`actual time=…` on each step, and an `Execution Time` line at the end, in milliseconds. Run it a
few times, and watch the numbers wobble.

**"Does `EXPLAIN` change my data?"**
Plain `EXPLAIN` never runs the query. `EXPLAIN (ANALYZE …)` **does** run it, which is harmless for a
`SELECT`, but for an `UPDATE` or `DELETE` it really changes the rows. Only use `ANALYZE` on
statements you'd be happy to run.

**"Do I have to rebuild an index when the data changes?"**
No. Postgres updates every index on every write, in the same statement. The index is never out of
date, and that's exactly why writes cost more.

## Coming from another language?

If you've used an [ORM](../glossary.md#orm), you may have added indexes as one line in a model, and
the ORM's migration sent a `CREATE INDEX` like the one in Step 4:

- **Prisma:** `@@index([customerId])` inside a model creates an index; `@unique` on a field, or
  `@@unique([a, b])`, creates a unique one. A relation field doesn't get an index automatically on
  Postgres.
- **Sequelize:** in a model's options, `indexes: [{ fields: ["customer_id"] }]`, or
  `{ unique: true, fields: ["email"] }` for a unique one.
- **Django ORM:** `class Meta: indexes = [models.Index(fields=["status"])]`. Unlike Postgres,
  Django adds an index to every `ForeignKey` for you (`db_index=True` is its default).
  `Order.objects.filter(customer_id=42).explain()` prints Postgres's `EXPLAIN` for the query.
- **SQLAlchemy:** `mapped_column(index=True)`, or `Index("orders_customer_id_idx",
  Order.customer_id)` for a named one. Foreign keys are not indexed automatically.
- **JPA/Hibernate:** `@Table(indexes = @Index(name = "orders_customer_id_idx", columnList =
  "customer_id"))`. It's only used when Hibernate generates the tables for you.
- **GORM:** the struct tag `gorm:"index"` on a field creates an index, and `gorm:"uniqueIndex"` a
  unique one.

In Part A3, SeaORM's migrations create indexes in Rust. However you create them, `EXPLAIN` on the
SQL your code sends is how you check they're used.

If you know spreadsheets: a spreadsheet has no index. `Ctrl+F` in a sheet of 100,000 rows checks
every cell, a `Seq Scan`. The closest thing is a column you keep **sorted**, where you can scroll
straight to the right place. An index is a sorted copy of one column that Postgres keeps sorted for
you, however the rows arrive.

## Common mistakes

These files read the table as Step 5 left it: 100,000 orders, and three indexes (`orders_pkey`,
`orders_customer_id_idx`, `orders_status_idx`). They put back anything they change.

**Wrapping the column in a function.**

```sql
{{#include ../../code/sql/indexes/70-function-on-column.sql}}
```

```text
{{#include ../../code/sql/indexes/70-function-on-column.out}}
```

The first plan is a `Seq Scan`, even though `orders_status_idx` exists and Step 5 showed
`status = 'refunded'` using it. The query here asks for `lower(status)`: the status turned into
small letters. The index holds `status` values, not `lower(status)` values, so Postgres can't look
the answer up in it. It has to read every row, work out `lower(status)`, and test that. The same
happens with any function or sum around the column: `WHERE customer_id + 1 = 43` can't use
`orders_customer_id_idx` either.

**Fix:** write the `WHERE` on the plain column when you can (`status = 'refunded'`). If you really
need the function, index the **expression** itself: `CREATE INDEX … ON orders (lower(status))`, as
the file does next. The `ANALYZE` after it lets Postgres gather statistics for the new expression.
The second plan uses `orders_lower_status_idx`. The last line removes that index again, to put the
table back as it was.

**Expecting an index to help `LIKE '%…'`.**

```sql
{{#include ../../code/sql/indexes/71-like-leading-wildcard.sql}}
```

```text
{{#include ../../code/sql/indexes/71-like-leading-wildcard.out}}
```

The two queries find the same 10,000 refunded rows. The first uses the index; the second is a
`Seq Scan`. (`~~` is Postgres's own way of writing `LIKE` in a plan.) `LIKE '%unded'`, from
[CRUD in SQL](crud-in-sql.md), means "ends with *unded*", and the `%` at the front means "any
letters". An index is sorted by the **start** of each value, like a dictionary, and a dictionary
is no help at all for finding words that **end** in *unded*: you'd have to read every word. So
Postgres reads every row. (Even `LIKE 'ref%'`, with the `%` at the end, can't use this index in the
book's Postgres: it needs an index built a different way, which is beyond this lesson.)

**Fix:** if you know the whole value, use `=`. If you really need "contains" searches on a big
table, Postgres has other kinds of index for that (the `pg_trgm` extension, and
full-text search), which are beyond this lesson.

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d indexes < code/sql/indexes/<file>`. They start from
the table as Step 5 left it, and they drop anything they create, so you can run them in any order,
as often as you like.

### One index, two columns

"Customer 42's refunds" tests two columns. An index can hold both:

```sql
{{#include ../../code/sql/indexes/50-multi-column.sql}}
```

```text
{{#include ../../code/sql/indexes/50-multi-column.out}}
```

`ON orders (customer_id, status)` sorts by `customer_id` first and, within each customer, by
`status`, the way a phone book sorts by last name, then first name. The `Index Cond` now holds
**both** tests: the index went straight to "customer 42, refunded", without looking at 42's paid
orders at all. The order of the columns matters, as in the phone book: the index is at its best
for queries that test the **first** column. A query on `status` alone gets much less help from
it. The last line drops the index again.

### A unique index

A `UNIQUE` rule is an index underneath. You can also create one directly:

```sql
{{#include ../../code/sql/indexes/51-unique-index.sql}}
```

```text
{{#include ../../code/sql/indexes/51-unique-index.out}}
```

`CREATE UNIQUE INDEX` does two jobs: it speeds up lookups by email, **and** it refuses a second row
with an email already in the index. The duplicate `ada@example.com` is refused with the same
`duplicate key value violates unique constraint` error as `UNIQUE` in
[Keys and relations](keys-and-relations.md), and the "constraint" it names is the index. `\di
users*` lists the indexes whose names start with `users`: yours, and `users_pkey`, which the
`PRIMARY KEY` made. The last line drops the whole `users` table; its indexes go with it.

### Removing an index

An index that isn't earning its keep can go:

```sql
{{#include ../../code/sql/indexes/52-drop-index.sql}}
```

```text
{{#include ../../code/sql/indexes/52-drop-index.out}}
```

`DROP INDEX orders_status_idx` deletes the index, and nothing else: all 100,000 rows are still
there. The query still works, and gives the same answer; only the plan changed, back to a
`Seq Scan`. That's worth remembering: **an index never changes what a query returns, only how fast
it runs.** The last line builds the index again, to leave the table as Step 5 left it.

### An index on some of the rows

Refunds are 10% of the orders. If your app only ever looks up a customer's **refunds** by
customer, you can index only those rows:

```sql
{{#include ../../code/sql/indexes/53-partial-index.sql}}
```

```text
{{#include ../../code/sql/indexes/53-partial-index.out}}
```

The `WHERE status = 'refunded'` at the end of `CREATE INDEX` makes a **partial index**: only rows
that pass the test get an entry. The plan uses it, and its `Index Cond` only needs
`customer_id = 42`, because every entry in this index is already a refund. It's 96 kB against
776 kB for the index on every order, and `INSERT`s of paid orders don't touch it at all. Postgres
uses it only for queries whose `WHERE` includes `status = 'refunded'`. The last line drops it.

## Your turn

The 🟡 and 🔴 exercises start from the table as Step 5 left it. If you've been experimenting, run
`01` to `04` again first, in order.

### 🟢 Guided

Run `01`, then `02`, then `03` (Steps 2 to 4), and fill in the blanks from the output of `02` and
`03`. (`01` comes first to remove the index, so that `02` really runs without one.)

```text
Without the index, Postgres read the whole table and removed ____ rows to keep 20.
With the index, the plan's top step is a ____________, it read ____ pages of the table,
and the "Rows Removed by Filter" line is ____.
```

<details><summary>Solution</summary>

```text
{{#include ../../code/sql/indexes/02-without-index.out}}
```

```text
{{#include ../../code/sql/indexes/03-with-index.out}}
```

Without the index, `Rows Removed by Filter: 99980`: Postgres read all 100,000 rows and removed
99,980. With the index, the top step is a `Bitmap Heap Scan`, fed by a `Bitmap Index Scan` on
`orders_customer_id_idx`. `Heap Blocks: exact=20` says it read 20 pages, and the
`Rows Removed by Filter` line is **gone**: nothing was read only to be thrown away. (If `02` showed
the index plan too, you skipped `01`: `02` only reads, so it uses the index if one is already
there.) Run `04` afterwards to put the status index back for the next exercises.

</details>

### 🟡 Tweak

Every price between 0 and 9,999 cents appears exactly 10 times. Find the orders with
`total_cents = 1234`: first `EXPLAIN` the query with no index, then create an index on
`total_cents` and `EXPLAIN` it again. Use the same `EXPLAIN` options as Step 3. Drop your index at
the end, to leave the table as you found it.

<details><summary>Solution</summary>

`code/sql/indexes/90-total-cents.sql`:

```sql
{{#include ../../code/sql/indexes/90-total-cents.sql}}
```

```text
{{#include ../../code/sql/indexes/90-total-cents.out}}
```

Before: a `Seq Scan` that removed 99,990 rows to keep 10. After: a `Bitmap Index Scan on
orders_total_cents_idx` found the 10 rows, and the `Bitmap Heap Scan` read 10 pages. The same story
as `customer_id`, because a single price picks out very few rows. The `DROP INDEX IF EXISTS` at the
top lets you run the file twice.

</details>

### 🔴 From scratch

Does `status` deserve an index? Step 5 showed the planner using `orders_status_idx` for
`'refunded'` and ignoring it for `'paid'`. But did the index really save work for `'refunded'`?
Drop the index, `EXPLAIN (ANALYZE, …)` the `'refunded'` query, create the index, and `EXPLAIN` it
again. Compare how many **pages** each plan read. (Hint: the table is 5200 kB, and a page is 8 kB,
so the table has 650 pages.) Then decide, and say why.

<details><summary>Solution</summary>

`code/sql/indexes/91-does-status-deserve-an-index.sql`:

```sql
{{#include ../../code/sql/indexes/91-does-status-deserve-an-index.sql}}
```

```text
{{#include ../../code/sql/indexes/91-does-status-deserve-an-index.out}}
```

Without the index, the `Seq Scan` read the whole table, all 650 pages, and removed 90,000 rows to
keep 10,000. With it, the plan used the index, but look at `Heap Blocks: exact=650`: it **still
read all 650 pages.** Every tenth order is a refund, so every page of the table holds some refunds.
The index told Postgres which rows to take on each page, but it couldn't skip a single page, and
it had to read the index too. And for `'paid'`, the planner doesn't use it at all.

**The verdict: no.** On this data, `orders_status_idx` saves almost nothing, and it costs 696 kB
plus a little time on every write. So the file drops it. (It would be a different story if refunds
were rare, say 1 in 10,000, or bunched together at the end of the table. That's how you decide: run
`EXPLAIN (ANALYZE …)` on the real query, with and without, and look at how much was read.) To get
the index back for the other files, run `04` again.

</details>

## Quick check

<div class="quiz" data-topic="indexes"></div>

## Remember this

- An **index** is a sorted list of one column's values with the addresses of their rows, like the
  index at the back of a book. `CREATE INDEX name ON table (column);`
- `EXPLAIN` shows the plan: `Seq Scan` read the whole table; `Index Scan` or `Bitmap Index Scan`
  used an index. `Rows Removed by Filter` counts rows read only to be thrown away.
- The **query planner** decides whether to use an index. It pays off when a query matches **few**
  rows; for most of the table, a plain scan is faster.
- Every `PRIMARY KEY` and `UNIQUE` rule gets an index automatically; foreign key columns don't.
- Every index costs disk space and time on every write. Add one for a real query, and check it with
  `EXPLAIN`.

## Go deeper

- [Indexes](https://www.postgresql.org/docs/18/indexes.html) — The official chapter on index types, multi-column, partial and expression indexes.
- [Using EXPLAIN](https://www.postgresql.org/docs/18/using-explain.html) — How to read a query plan, node by node.

<!-- next:start -->

**Next:**

- [SQL transactions](../a1-postgres/sql-transactions.md)

<!-- next:end -->
