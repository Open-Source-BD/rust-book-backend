# Cheat sheet: SQL

> **Beginner** · Part A1 · PostgreSQL & SQL

Every SQL pattern from Part A1, on one page. Each one is a short "How do I…?" question, the SQL
that answers it, and a link to the lesson that explains it line by line. Come back here when you
remember *that* something exists but not *how* it's spelled.

All the snippets use one tiny shop: a `shop_items` table (the things for sale) and a
`shop_orders` table (who bought how many). They come from one file,
`code/sql/cheatsheet-sql/01-cheatsheet.sql`, in the order you see them here, and the book's checks
run that file against a real Postgres on every change. So every snippet on this page runs, and it
runs in this order.

## Try every snippet

Like every page in Part A1, the cheat sheet has a [database](../glossary.md#database) of its own.
From the book's folder, create it once, then run the whole file:

```bash
docker compose exec db createdb -U postgres cheatsheet_sql
docker compose exec -T db psql -U postgres -d cheatsheet_sql < code/sql/cheatsheet-sql/01-cheatsheet.sql
```

This is the same pair of commands you met in [What is a database?](what-is-a-database.md): the
first makes an empty database called `cheatsheet_sql` (a second run says `already exists`, which is
harmless), and the second feeds the file to [psql](../glossary.md#psql). The file starts by
dropping its two tables, so you can run it as often as you like and get the same result.

## How do I…?

### …create a table?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:create-table}}
```

One line per [column](../glossary.md#column): its name, then its
[data type](../glossary.md#data-type). `NOT NULL` makes a value required. The `id` line makes
Postgres number the rows for you and turns `id` into the table's
[primary key](../glossary.md#primary-key).

Explained in: [Tables, rows and psql](tables-rows-and-psql.md) (`CREATE TABLE`),
[Postgres data types](postgres-data-types.md) (which type for which value), and
[Keys and relations](keys-and-relations.md) (the `id` line).

### …add rows?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:insert}}
```

List the columns, then one bracketed group of values per [row](../glossary.md#row). Text and dates
go in single quotes. `id` is missing on purpose: Postgres fills it in.

Explained in: [CRUD in SQL](crud-in-sql.md).

### …find rows that match, sorted?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:select-filter-sort}}
```

`SELECT` picks the columns, [`WHERE`](../glossary.md#where) keeps only the rows that pass the test,
`ORDER BY … DESC` sorts biggest first, and `LIMIT` keeps the first few. They always come in this
order.

Explained in: [CRUD in SQL](crud-in-sql.md).

### …change a value?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:update}}
```

`SET` says what the new value is, and it may use the old one (`price_cents + 100`). The `WHERE`
says which rows change. Without it, **every** row changes.

Explained in: [CRUD in SQL](crud-in-sql.md).

### …remove rows?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:delete}}
```

Same rule as `UPDATE`: the `WHERE` picks the rows, and without it the table is emptied. There's no
undo, unless you're inside a transaction (below).

Explained in: [CRUD in SQL](crud-in-sql.md).

### …find missing values?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:null-checks}}
```

A missing value is [`NULL`](../glossary.md#null). Test for it with `IS NULL` (or `IS NOT NULL`),
never with `= NULL`, which matches nothing. `COALESCE(in_stock, false)` shows `false` wherever
`in_stock` is missing.

Explained in: [Postgres data types](postgres-data-types.md).

### …give each row an id, and point one table at another?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:primary-foreign-keys}}
```

`PRIMARY KEY` gives each order a unique id. `REFERENCES shop_items (id)` makes `item_id` a
[foreign key](../glossary.md#foreign-key): Postgres refuses an order for an item that doesn't
exist. `CHECK (quantity > 0)` is a [check constraint](../glossary.md#check-constraint): it refuses
a zero or negative quantity. The `INSERT` adds three orders: the join, count and explain examples
below all read these rows.

Explained in: [Keys and relations](keys-and-relations.md).

### …combine rows from two tables?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:join}}
```

A [join](../glossary.md#join) pairs each order with the item its `item_id` points at. The `ON`
line says which columns must be equal. Write `table.column` so it's always clear which table a
column comes from.

Explained in: [Keys and relations](keys-and-relations.md).

### …count per group, including groups with none?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:left-join-count}}
```

`LEFT JOIN` keeps every item, even one with no orders at all. `count(shop_orders.id)` counts only
real orders, so such an item shows `0` instead of vanishing. `GROUP BY shop_items.id,
shop_items.name` makes one result row per item: the `id` keeps two items that share a name apart,
and the name is there so you can show it.

Explained in: [Keys and relations](keys-and-relations.md).

### …make a lookup fast?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:index}}
```

An [index](../glossary.md#index) on `item_id` lets Postgres jump straight to one item's orders
instead of reading the whole table. Index the columns you search by often, and not the rest: each
index slows down every write.

Explained in: [Indexes](indexes.md).

### …see how Postgres runs a query?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:explain}}
```

[`EXPLAIN`](../glossary.md#explain) prints the plan instead of the rows. `Seq Scan` means "read
every row"; `Index Scan` or `Bitmap Index Scan` means an index was used. On this three-row table
Postgres picks `Seq Scan` even with the index: reading three rows is quicker than using an index. Add `ANALYZE` inside the brackets to run the query for real and see actual row counts.

Explained in: [Indexes](indexes.md).

### …make several changes all-or-nothing?

```sql
{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:transaction}}
```

Between `BEGIN` and `COMMIT`, the changes form one
[transaction](../glossary.md#transaction): they all happen, or none do. `ROLLBACK` throws away
everything since `BEGIN`, so the second block deletes nothing. If a statement fails inside a
transaction, run `ROLLBACK` and start again.

Explained in: [SQL transactions](sql-transactions.md).

### …put it all together?

Every pattern above, in one real design: four related tables, the questions an app asks them, and
changes wrapped in transactions. See [Build it: a library database](a1-build-library-schema.md).

## psql commands

These are [backslash commands](../glossary.md#backslash-command): you type them at the psql prompt
(`cheatsheet_sql=#`), and psql answers them itself instead of sending them to the database. They
need no semicolon.

| Type this      | It does                                                 | Explained in                                    |
| -------------- | ------------------------------------------------------- | ----------------------------------------------- |
| `\dt`          | lists your tables                                       | [Tables, rows and psql](tables-rows-and-psql.md) |
| `\d shop_items` | describes one table: its columns, types and rules      | [Tables, rows and psql](tables-rows-and-psql.md) |
| `\di`          | lists your indexes                                      | [Indexes](indexes.md)                           |
| `\?`           | lists every backslash command, with a short description | [Tables, rows and psql](tables-rows-and-psql.md) |
| `\q`           | quits psql and gives you your terminal back             | [Tables, rows and psql](tables-rows-and-psql.md) |

To get the prompt in the first place, open psql on this page's database with
`docker compose exec db psql -U postgres -d cheatsheet_sql`.

## Go deeper

- [SQL Commands](https://www.postgresql.org/docs/18/sql-commands.html) — The official reference page for every SQL command, from ABORT to VALUES.
- [psql](https://www.postgresql.org/docs/18/app-psql.html) — Every psql option and backslash command.

<!-- next:start -->

**Next:**

- Hello, Axum (coming soon)

<!-- next:end -->
