# SQL transactions

> **Beginner** · Part A1 · PostgreSQL & SQL

## By the end of this lesson

- You can group changes so they all happen or none do.
- You can undo work with ROLLBACK.
- You know what an aborted transaction is and how to recover.

## What & why

BEGIN, COMMIT and ROLLBACK: all-or-nothing changes, and why a bank transfer needs them.

Ada has 100 dollars in her account, and Linus has 50. Ada sends Linus 200 dollars, which she
doesn't have. To the database, a transfer isn't one change but **two**: add the money to Linus's
account, and take it out of Ada's. Here is what really happened when this lesson's Postgres ran
those two changes one after the other, with money stored as whole cents:

```text
{{#include ../../code/sql/sql-transactions/02-without-transaction.out}}
```

The first change worked (`UPDATE 1`): Linus went from 5000 cents to 25000. The second one failed
(`ERROR`), because a rule on the table says no balance may go below zero, and Ada would have ended
up at `-10000`. So Ada still has her 10000 cents, Linus has 20000 more than before, and the bank
has **made 200 dollars out of nothing**. The rule did its job for Ada's row, and the damage was done
anyway, one line earlier.

A failed rule is one way to be stopped halfway. Others are worse: your app crashes between the two
changes, the server loses power, the network drops. Whenever one change has happened and the other
hasn't, money has appeared, or vanished.

Think of a shop checkout. You hand over the money and the shop hands over the goods: both happen,
or neither does. Nobody walks out with the goods and no payment, or pays and gets nothing. A
database gives you the same promise with a [**transaction**](../glossary.md#transaction): a group of
statements that Postgres treats as **one**. You start one with `BEGIN`. At the end, you either
[**commit**](../glossary.md#commit) (`COMMIT`: keep every change, all at once) or
[**roll back**](../glossary.md#rollback) (`ROLLBACK`: throw every change away, as if you never
started). And if anything fails in between, Postgres makes sure the half-done work is never kept.

You met `BEGIN` and `ROLLBACK` briefly in [CRUD in SQL](crud-in-sql.md), as an undo button. This
lesson shows what they really do, and why every backend that moves money, stock or anything else
that must add up relies on them.

## The idea, slowly

### Step 1: this lesson's database

As in the last lessons, this lesson gets a database of its own, named after the lesson:
`sql_transactions`. From the book's folder (the one with `docker-compose.yml`), make sure Postgres
is running, then create it:

```bash
docker compose up -d --wait
docker compose exec db createdb -U postgres sql_transactions
```

### Line by line

`docker compose up -d --wait`
- **What:** starts the book's Postgres, or does nothing if it's already running.
- **Why:** every command in this lesson talks to Postgres, so it has to be up first.
- **How:** `-d` runs it in the background; `--wait` returns only once Postgres is ready to answer.
- **Remove it and…** if Postgres is stopped, the next command fails with
  `service "db" is not running`.

`docker compose exec db createdb -U postgres sql_transactions`
- **What:** creates a new, empty database called `sql_transactions`.
- **Why:** this lesson makes its own `accounts` table. In a database of its own, it can't get in the
  way of the tables from the last lessons.
- **How:** `docker compose exec db` runs the command inside the `db` container; `createdb` is
  Postgres's small program for making databases; `-U postgres` logs in as the user `postgres`. The
  name is the lesson's web address, `sql-transactions`, with the `-` turned into `_`, because a
  database name can't easily contain a `-`.
- **Remove it and…** every later command fails with `database "sql_transactions" does not exist`.

### Run it

```bash
docker compose exec db createdb -U postgres sql_transactions
```

`createdb` prints **nothing** when it works. If you run it a second time, you see this:

```text
createdb: error: database creation failed: ERROR:  database "sql_transactions" already exists
```

That's **harmless**: it says you already made this database, and nothing in it was touched. You
only ever need to create it once.

- ✅ If you see no output, or the `already exists` message, you're right: the database is there.
- ❌ If you see `service "db" is not running`, run `docker compose up -d --wait` first.
- ❌ If you see `no configuration file provided: not found`, `cd` into the folder with
  `docker-compose.yml`.

### Step 2: two bank accounts, and a rule about money

Every example in this lesson moves money between two accounts. This is
`code/sql/sql-transactions/01-setup.sql`:

```sql
{{#include ../../code/sql/sql-transactions/01-setup.sql}}
```

### Line by line

`SET client_min_messages = warning;` · `DROP TABLE IF EXISTS accounts;`
- **What:** hides small notes, then deletes the `accounts` table if it's there.
- **Why:** this is the lesson's reset button. Whatever the later files have done to the balances,
  running `01` puts you back at the very start: Ada 10000, Linus 5000.
- **How:** as in the last lessons. `IF EXISTS` skips the drop on the very first run, and the `SET`
  line hides the note that would say so.
- **Remove it and…** a second run fails with `ERROR:  relation "accounts" already exists`.

`CREATE TABLE accounts (`
- **What:** a table of bank accounts, one row per account.
- **Why:** a transfer between two accounts is the classic example of two changes that must happen
  together.
- **How:** the columns follow, one per line, and `);` closes the list.
- **Remove it and…** there's no table, and the `INSERT` fails with
  `relation "accounts" does not exist`.

`id integer PRIMARY KEY,`
- **What:** the account number.
- **Why:** each `UPDATE` in this lesson picks one account with `WHERE id = 1` or `WHERE id = 2`, so
  the number must never be shared.
- **How:** a [primary key](../glossary.md#primary-key), from
  [Keys and relations](keys-and-relations.md). This time there's no
  `GENERATED ALWAYS AS IDENTITY`: the file types the numbers itself (`1` and `2`), so every example
  can rely on Ada being `1` and Linus being `2`.
- **Remove it and…** (`PRIMARY KEY`) nothing stops a second account `1`, and `WHERE id = 1` could
  change two rows at once.

`owner text NOT NULL,`
- **What:** whose account it is.
- **Why:** so the output reads "Ada" and "Linus" instead of bare numbers.
- **How:** text, and never missing.
- **Remove it and…** (`NOT NULL`) an account with no owner could slip in.

`balance_cents integer NOT NULL CHECK (balance_cents >= 0)`
- **What:** how much money is in the account, in whole cents, and a rule that it's never negative.
- **Why:** the rule is what stops Ada from spending money she doesn't have. It's also how this
  lesson makes a change **fail** on purpose, so you can see what a transaction does when something
  goes wrong.
- **How:** money as whole cents is the habit from [Postgres data types](postgres-data-types.md):
  10000 cents is 100 dollars, with no rounding surprises. `CHECK (…)` is a
  [check constraint](../glossary.md#check-constraint), the rule-you-write-yourself from
  [Keys and relations](keys-and-relations.md#a-rule-you-write-yourself-check): Postgres tests it on
  every `INSERT` and `UPDATE`, and refuses any row where `balance_cents >= 0` is false. Postgres
  names it `accounts_balance_cents_check` (table, column, `check`), and that name appears in the
  error when it's broken.
- **Remove it and…** (the `CHECK`) Postgres happily stores `-10000`, and nothing stops an overdraft.

`INSERT INTO accounts (id, owner, balance_cents) VALUES (1, 'Ada', 10000), (2, 'Linus', 5000);`
- **What:** opens two accounts: Ada with 100 dollars, Linus with 50.
- **Why:** someone to send money, and someone to receive it.
- **How:** one `INSERT` with two rows in its `VALUES` list, as in [CRUD in SQL](crud-in-sql.md).
- **Remove it and…** the table is empty, and every `UPDATE` in this lesson says `UPDATE 0`.

`SELECT * FROM accounts ORDER BY id;`
- **What:** shows both accounts, Ada first.
- **Why:** you'll see this same line many times in this lesson. It's how you check the balances
  after each experiment.
- **How:** `ORDER BY id` keeps Ada (`1`) above Linus (`2`) every time.
- **Remove it and…** the accounts are there; you don't see them.

### Run it

Every file in this lesson runs the same way: feed it to `psql` in this lesson's database.

```bash
docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/01-setup.sql
```

```text
{{#include ../../code/sql/sql-transactions/01-setup.out}}
```

Each statement prints a **command tag**, psql's short report of what it did: `SET`, `DROP TABLE`
(it shows up even the first time, when there was nothing to drop), `CREATE TABLE`, and
`INSERT 0 2`, whose **last** number says two rows went in. Then the `SELECT` prints the table, and `(2 rows)` counts its rows.

- ✅ If you see Ada with `10000` and Linus with `5000`, you're right.
- ✅ Run it a second time: the output is identical, because the `DROP TABLE` line starts over.
- ❌ If you see `FATAL:  database "sql_transactions" does not exist`, go back to Step 1.

### Step 3: a transfer without a transaction

This is the file from *What & why*: Ada tries to send Linus 20000 cents, as two plain `UPDATE`s.
This is `code/sql/sql-transactions/02-without-transaction.sql`:

```sql
{{#include ../../code/sql/sql-transactions/02-without-transaction.sql}}
```

### Line by line

`UPDATE accounts SET balance_cents = balance_cents + 20000 WHERE id = 2;`
- **What:** adds 20000 cents to Linus's account.
- **Why:** half one of the transfer: the money arrives.
- **How:** `balance_cents + 20000` takes the balance the row has **now** and adds to it, so you
  don't need to know it's 5000. `WHERE id = 2` picks Linus's row, and only his.
- **Remove it and…** (the line) Ada's half fails as before, and nothing changes at all. The problem
  needs both halves: one that works, one that doesn't.

`UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;`
- **What:** takes 20000 cents out of Ada's account.
- **Why:** half two: the money leaves.
- **How:** 10000 − 20000 = −10000, which breaks the `CHECK` rule, so Postgres refuses this
  statement. An error stops only its own statement: `psql` carries on with the next line, as you
  saw in [Keys and relations](keys-and-relations.md). And the first `UPDATE` is **already saved**.
  Without `BEGIN`, each statement stands alone, and Postgres keeps it the moment it succeeds.
- **Remove it and…** (the `WHERE`) the `UPDATE` would try every row, and fail for the same reason.

`SELECT * FROM accounts ORDER BY id;`
- **What:** shows the balances after the failed transfer.
- **Why:** to see the damage.
- **How:** as in `01`.
- **Remove it and…** the damage is still done; you don't see it.

`UPDATE accounts SET balance_cents = 5000 WHERE id = 2;`
- **What:** puts Linus back to 5000 cents: a repair.
- **Why:** so the next files start from the balances `01` made. It also shows what fixing a
  half-done change costs without a transaction: someone has to notice, work out what the right
  number was, and type it in by hand. In a real bank, with thousands of transfers a minute, nobody
  could.
- **How:** `SET balance_cents = 5000` writes a fixed number, not `+` or `−` something, so it gives
  the right answer however many times you run the file.
- **Remove it and…** Linus keeps his free 20000 cents, and every later output in this lesson has
  different numbers from the book's.

### Run it

```bash
docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/02-without-transaction.sql
```

```text
{{#include ../../code/sql/sql-transactions/02-without-transaction.out}}
```

`UPDATE 1` is the command tag for "one row changed". Then the error, in two lines: `ERROR:` names
the rule that was broken, `accounts_balance_cents_check`, and `DETAIL:` shows the row Postgres
refused: Ada with `-10000`. The table shows Linus with **25000** and Ada still with 10000: 20000
cents that came from nowhere. The last `UPDATE 1` is the repair.

- ✅ If you see the `ERROR` and Linus with `25000`, you're right: you've made money out of thin air.
- ✅ Run it a second time: the output is identical, because the repair line put Linus back.
- ❌ If Ada shows `-10000` and there's no `ERROR`, your table has no `CHECK` rule: run `01` again.

### Step 4: the same kind of transfer, as one transaction

Now a transfer that Ada **can** afford, 2500 cents, wrapped in a transaction. This is
`code/sql/sql-transactions/03-transfer-commit.sql`:

```sql
{{#include ../../code/sql/sql-transactions/03-transfer-commit.sql}}
```

### Line by line

`BEGIN;`
- **What:** starts a transaction.
- **Why:** from here on, every change is part of one group, and none of it is permanent yet.
- **How:** each `psql` you start opens its own **connection** to Postgres: one conversation between
  that program and the server, which lasts until the program ends. Postgres now remembers that this
  connection is "inside a transaction". Changes are made for real, but kept private until the end,
  and no other connection sees them.
- **Remove it and…** each `UPDATE` is saved on its own, as in Step 3. `COMMIT` then has nothing to
  end, and prints `WARNING:  there is no transaction in progress` before its `COMMIT` tag.

`UPDATE accounts SET balance_cents = balance_cents - 2500 WHERE id = 1;`
- **What:** takes 2500 cents from Ada.
- **Why:** half one of the transfer.
- **How:** 10000 − 2500 = 7500, which passes the `CHECK`. This time the money leaves **first**. It
  doesn't matter which half goes first: inside a transaction, neither half is kept without the
  other.
- **Remove it and…** Linus gets 2500 cents from nowhere, in a transaction or not. A transaction
  makes the statements you wrote all-or-nothing; it can't tell that you forgot one.

`UPDATE accounts SET balance_cents = balance_cents + 2500 WHERE id = 2;`
- **What:** gives the 2500 cents to Linus.
- **Why:** half two.
- **How:** 5000 + 2500 = 7500.
- **Remove it and…** 2500 cents vanish: Ada pays and Linus gets nothing.

`COMMIT;`
- **What:** ends the transaction and keeps both changes.
- **Why:** this is the moment the transfer really happens. Before it, both changes could still be
  undone; after it, both are saved for good.
- **How:** Postgres makes the whole group permanent in one step. Other connections see Ada's 7500
  and Linus's 7500 appear at the same instant: never one without the other.
- **Remove it and…** the transaction is still open when the file ends, and both changes are
  thrown away. *Common mistakes* shows it happening.

`SELECT * FROM accounts ORDER BY id;`
- **What:** shows the balances after the commit.
- **Why:** to check the transfer is really there.
- **How:** it runs after `COMMIT`, outside the transaction, as a statement of its own.
- **Remove it and…** the transfer is saved; you don't see it.

### Run it

```bash
docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/03-transfer-commit.sql
```

```text
{{#include ../../code/sql/sql-transactions/03-transfer-commit.out}}
```

`BEGIN` and `COMMIT` are the command tags of those two lines: each prints its own name. Ada and
Linus now have 7500 cents each, and the total is still 15000. Money moved; none appeared, none
vanished.

- ✅ If you see `COMMIT` and `7500` twice, you're right.
- ❌ If you see Ada `5000` and Linus `10000`, you ran this file twice, and it made two transfers.
  That's correct behaviour. Run `01`, then this file, to get the book's numbers again.

### Step 5: ROLLBACK — change your mind

`COMMIT` keeps the changes. `ROLLBACK` does the opposite. This is
`code/sql/sql-transactions/04-rollback.sql`:

```sql
{{#include ../../code/sql/sql-transactions/04-rollback.sql}}
```

### Line by line

`BEGIN;`
- **What:** starts a transaction, as in `03`.
- **Why:** without one, there's nothing to roll back.
- **How:** as in `03`.
- **Remove it and…** the `UPDATE` is saved at once, Ada really has 0, and `ROLLBACK` prints
  `WARNING:  there is no transaction in progress`.

`UPDATE accounts SET balance_cents = 0 WHERE id = 1;`
- **What:** empties Ada's account.
- **Why:** a change you'll want to take back.
- **How:** 0 passes the `CHECK` (`0 >= 0` is true), so Postgres makes the change.
- **Remove it and…** there's nothing to undo, and both `SELECT`s show the same thing.

`SELECT * FROM accounts ORDER BY id;` (the first one)
- **What:** shows the balances **inside** the transaction.
- **Why:** to prove the change really happened, not only "maybe later".
- **How:** inside a transaction, you see your own changes as you make them. Other connections don't
  see them until you commit.
- **Remove it and…** you'd have no proof that `ROLLBACK` undid anything.

`ROLLBACK;`
- **What:** ends the transaction and throws away every change made since `BEGIN`.
- **Why:** you changed your mind. Or, in an app, something went wrong halfway, and the code gives
  up cleanly.
- **How:** Postgres discards the whole group. The rows go back to exactly what they were before
  `BEGIN`, and nobody else ever saw the 0.
- **Remove it and…** the file ends with the transaction still open, and Postgres throws the change
  away anyway (the forgotten-`COMMIT` mistake, below). The second `SELECT` would show Ada with 0,
  because it would still be inside the transaction.

`SELECT * FROM accounts ORDER BY id;` (the second one)
- **What:** shows the balances after the rollback.
- **Why:** to see Ada's money back.
- **How:** as in `03`.
- **Remove it and…** the change is still undone; you don't see it.

### Run it

```bash
docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/04-rollback.sql
```

```text
{{#include ../../code/sql/sql-transactions/04-rollback.out}}
```

Inside the transaction, Ada has `0`. After `ROLLBACK` (its command tag is its own name), she has
`7500` again, as if the `UPDATE` never happened. This file changes nothing, so you can run it as
often as you like.

- ✅ If you see `0` in the first table and `7500` in the second, you're right.
- ❌ If both tables show `0`, check the `ROLLBACK;` line is there and spelled right.

### Step 6: the shape of every transaction

Every transaction in this book, in SQL or later in Rust, has the same shape:

```text
BEGIN
  │
  ├─ change 1        (made, but private)
  ├─ change 2        (made, but private)
  ├─ …
  │
  ├──▶ COMMIT     →  every change is saved, all at once
  │
  └──▶ ROLLBACK   →  every change is thrown away
       (or an error, or the connection closes)
```

### Line by line

`BEGIN`
- **What:** the start of the group.
- **Why:** everything after it belongs together.
- **How:** it's a statement of its own, sent before the changes.
- **Remove it and…** there's no group: each change is saved alone, as in Step 3.

`├─ change 1 (made, but private)` · `├─ change 2` · `├─ …`
- **What:** the statements inside: `UPDATE`s, `INSERT`s, `DELETE`s, and `SELECT`s to look around.
- **Why:** a transaction can hold as many as the job needs: two for a transfer, many for a big
  order.
- **How:** each one runs for real, and your own connection sees the results, as in Step 5. Nobody
  else does yet.
- **Remove it and…** (all of them) an empty transaction is allowed, and does nothing.

`├──▶ COMMIT → every change is saved, all at once`
- **What:** one way to end: keep everything.
- **Why:** the job is done, and every step worked.
- **How:** Step 4. Postgres saves the group in one step, and everyone sees it at the same moment.
- **Remove it and…** (both ways to end) the transaction stays open until the connection closes,
  and then it's thrown away.

`└──▶ ROLLBACK → every change is thrown away`
- **What:** the other way to end: keep nothing.
- **Why:** you changed your mind, or something went wrong.
- **How:** Step 5. The rows go back to how they were before `BEGIN`.
- **Remove it and…** the only way to undo would be by hand, as with the repair line in Step 3.

`(or an error, or the connection closes)`
- **What:** the two ways a transaction ends in a rollback without you typing `ROLLBACK`.
- **Why:** this is the safety net. A crashed app or a failed statement can never leave half a
  transfer behind.
- **How:** Step 7 shows the error; *Common mistakes* shows the closed connection.
- **Remove it and…** (this safety net) you'd be back to Step 3: a crash between two changes would
  keep one of them.

There's never a third way: no ending where some changes are kept and others aren't.

### Step 7: when something fails inside a transaction

Step 3's transfer broke the bank. Here is the very same transfer, now inside a transaction. This is
`code/sql/sql-transactions/05-failure-inside.sql`:

```sql
{{#include ../../code/sql/sql-transactions/05-failure-inside.sql}}
```

### Line by line

`BEGIN;`
- **What:** starts a transaction.
- **Why:** this time, both halves of the transfer are one group.
- **How:** as in `03`.
- **Remove it and…** you're back to Step 3: Linus keeps money that came from nowhere (and `COMMIT`
  prints `WARNING:  there is no transaction in progress`).

`UPDATE accounts SET balance_cents = balance_cents + 20000 WHERE id = 2;`
- **What:** gives Linus 20000 cents, as in Step 3.
- **Why:** half one works, exactly as before.
- **How:** Linus goes from 7500 to 27500, but only inside the transaction.
- **Remove it and…** only the failing half is left, and nothing changes either way.

`UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;`
- **What:** tries to take 20000 cents from Ada, who has 7500.
- **Why:** half two fails, exactly as before, on the `CHECK` rule.
- **How:** here's the difference. Inside a transaction, a failed statement doesn't only stop
  itself: it marks the **whole transaction** as failed. Postgres calls it **aborted**. From now on,
  the transaction can't be committed. It can only end.
- **Remove it and…** nothing fails, and the `COMMIT` below saves Linus's 20000 free cents. The
  transaction guards against failures, not against a missing step.

`SELECT * FROM accounts ORDER BY id;` (inside)
- **What:** tries to look at the balances.
- **Why:** to show what "aborted" means: Postgres refuses **every** statement now, even a harmless
  `SELECT`.
- **How:** it answers `current transaction is aborted, commands ignored until end of transaction
  block`. *Transaction block* is Postgres's name for everything between `BEGIN` and the end.
- **Remove it and…** you'd miss the clearest sign that the transaction is aborted.

`COMMIT;`
- **What:** asks to keep the changes.
- **Why:** to show that Postgres won't: a failed transaction can't be committed.
- **How:** Postgres ends the transaction, but with a rollback, and its command tag says
  `ROLLBACK`, not `COMMIT`. Linus's 20000 cents are thrown away with the rest.
- **Remove it and…** the file ends and the connection closes, which rolls back too.

`SELECT * FROM accounts ORDER BY id;` (after)
- **What:** shows the balances after it's all over.
- **Why:** the proof.
- **How:** it's outside the transaction now, so it runs normally.
- **Remove it and…** the balances are still safe; you don't see them.

### Run it

```bash
docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/05-failure-inside.sql
```

```text
{{#include ../../code/sql/sql-transactions/05-failure-inside.out}}
```

Read it line by line. `BEGIN`, then `UPDATE 1` for Linus. Then Ada's `UPDATE` breaks the `CHECK`
rule: `DETAIL:` shows she'd have had `-12500`. Then the `SELECT` is refused with
`current transaction is aborted`. Then the line that surprises everyone: you typed `COMMIT`, and the
tag says **`ROLLBACK`**. That's Postgres telling you what it really did. And the balances: 7500 and
7500, exactly as before `BEGIN`. The same failed transfer that made 200 dollars appear in Step 3
did **nothing at all** here. That's the whole point of a transaction.

- ✅ If you see `current transaction is aborted`, then `ROLLBACK`, and `7500` twice, you're right.
- ✅ Run it a second time: the output is identical, because the transaction never changed anything.

### Step 8: ACID — four promises, in plain words

Every transaction comes with four promises, known by their first letters as
[**ACID**](../glossary.md#acid). You'll meet the word in job ads and database docs. You've already
seen three of them happen:

```text
A  Atomic       all of it happens, or none of it does       (Step 7)
C  Consistent   the rules on your tables always hold        (the CHECK, every step)
I  Isolated     others don't see your half-done work        (Step 5)
D  Durable      once committed, it stays                    (Step 4)
```

### Line by line

`A  Atomic       all of it happens, or none of it does`
- **What:** the transaction can't be split. *Atom* comes from the Greek for "can't be cut".
- **Why:** it's the promise the transfer needed: no half-transfers, ever.
- **How:** Step 7: one statement failed, and Postgres threw away the one that had worked.
- **Remove it and…** you get Step 3: one half kept, the other lost.

`C  Consistent   the rules on your tables always hold`
- **What:** a transaction takes the database from one valid state to another. Every
  [constraint](../glossary.md#constraint) (`CHECK`, `NOT NULL`, `UNIQUE`, foreign keys) holds at
  the end, as it did at the start.
- **Why:** so the data always makes sense: no negative balances, no books pointing at missing
  authors.
- **How:** Postgres checks the rules on every change, and a transaction that breaks one can't
  commit.
- **Remove it and…** the `CHECK` rule would stop protecting you inside a transaction, and Ada
  could end up at `-12500`.

`I  Isolated     others don't see your half-done work`
- **What:** while your transaction is open, other connections don't see your changes: to them,
  every row you've touched still looks as it did before.
- **Why:** a customer loading their balance in the middle of a transfer must never see the money
  gone from Ada and not yet at Linus.
- **How:** Postgres keeps the old version of each row you change until you commit. Other
  connections keep reading the old version; yours reads the new one. In Step 5, nobody else could
  ever have seen Ada's 0.
- **Remove it and…** other people could read, and act on, numbers you're about to roll back.

`D  Durable      once committed, it stays`
- **What:** when `COMMIT` answers, the changes are safely on disk.
- **Why:** if the app says "transfer done", it must still be done after a crash or a power cut.
- **How:** Postgres writes the changes to a log on disk before it answers `COMMIT`. If the server
  crashes, it replays that log when it starts again.
- **Remove it and…** a transfer the customer was told had worked could vanish in the next power
  cut.

You'll use exactly this in Rust. In Part A3, *Transactions in SeaORM* wraps the same `BEGIN` …
`COMMIT` around your Rust code, and rolls back if it returns an error. And in the capstone,
*ShopRS 12: Checkout* uses one to take the payment, reduce the stock and create the order together:
the shop checkout from *What & why*, for real.

## You might be wondering…

**"Does every single statement run in a transaction?"**
Yes. Without `BEGIN`, Postgres wraps **each statement** in a tiny transaction of its own, and
commits it the moment it succeeds. That's called **autocommit**, and it's why Step 3's first
`UPDATE` was saved at once. So a single statement is always all-or-nothing on its own: an
`UPDATE` that changes 1,000 rows and fails on the 1,000th changes none of them. `BEGIN` is only
needed when **several** statements must succeed or fail together.

**"What do other connections see in the middle of my transaction?"**
Nothing of yours, until you commit. To them, every row you've changed still looks as it did
before (the *I* in ACID). If one of them tries to **change** a row you've changed, it has to wait: Postgres makes it
queue until your transaction commits or rolls back, then lets it go on. Two transfers can't both
change Ada's balance at the same moment.

**"Why did `COMMIT` print `ROLLBACK`?"**
Because that's what happened. After an error inside a transaction (Step 7), the transaction is
aborted and can't be saved. Postgres still accepts `COMMIT` as a way to end it, but it rolls back
instead, and the tag tells you so. Your Rust code gets told the same thing: the commit fails, and
you can react to it. Always read the tag: `COMMIT` means saved, `ROLLBACK` means not.

**"Can a transaction be too long?"**
Yes. Every row you change stays **locked** until you commit or roll back, so any other connection
that wants to change it waits, and in a busy app the queue grows fast. A transaction left open for
minutes also stops Postgres from cleaning up the old versions of rows it's keeping for you. Keep
transactions short: do the slow work (calling another website, waiting for a user) **before**
`BEGIN`, then open the transaction, make the changes, and commit straight away.

## Coming from another language?

If you've used an [ORM](../glossary.md#orm), it sent this `BEGIN` … `COMMIT` for you, usually around
a function or a block of code:

- **Prisma:** `prisma.$transaction(async (tx) => { … })` runs everything that uses `tx` in one
  transaction, and rolls back if the function throws. `prisma.$transaction([a, b])` does the same
  for a list of queries.
- **Sequelize:** `sequelize.transaction(async (t) => { … })` commits when the function finishes and
  rolls back if it throws. Each query inside must be given `{ transaction: t }`, or it runs outside
  the transaction.
- **Django ORM:** Django autocommits each query, like Postgres. `with transaction.atomic():` wraps a
  block in a transaction that rolls back if the block raises an exception, and a nested `atomic()`
  uses a savepoint (see *More examples*).
- **SQLAlchemy:** a `Session` starts a transaction on its first query, and nothing is saved until
  `session.commit()`: forgetting it is the forgotten-`COMMIT` mistake below. `with session.begin():`
  commits at the end of the block, or rolls back on an exception.
- **JPA/Hibernate** (with Spring): a method marked `@Transactional` runs in one transaction. By
  default it rolls back on unchecked exceptions (a `RuntimeException`), but **not** on checked
  ones.
- **GORM:** `db.Transaction(func(tx *gorm.DB) error { … })` commits if the function returns `nil`
  and rolls back if it returns an error. Inside, use `tx`, not `db`, or the query runs outside the
  transaction.

The pattern is the same everywhere, and it's the one SeaORM uses in Part A3: start, do the work,
commit if it all worked, roll back if anything failed.

If you know spreadsheets: a spreadsheet saves each cell the moment you type it, like autocommit.
The nearest thing to a transaction is making all your changes in a **copy** of the file, checking
them, and only then replacing the original (commit), or deleting the copy (rollback). Anyone who
opens the original meanwhile sees the old numbers.

## Common mistakes

These files start from the balances Step 7 left: 7500 and 7500.

**Forgetting `COMMIT`.**

`code/sql/sql-transactions/70-forgot-commit.sql` starts a transaction, takes 1000 cents from Ada,
looks, and then the file ends:

```sql
{{#include ../../code/sql/sql-transactions/70-forgot-commit.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/70-forgot-commit.out}}
```

No error, no warning, and the table shows Ada with `6500`. It looks done. Now run
`code/sql/sql-transactions/71-after-forgot-commit.sql`, which only looks:

```sql
{{#include ../../code/sql/sql-transactions/71-after-forgot-commit.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/71-after-forgot-commit.out}}
```

Ada has `7500` again: the change is **gone**. When the `70` file ended, `psql` ended too, and closed
its connection to Postgres. A transaction belongs to one connection, and when the connection closes
with the transaction still open, Postgres rolls it back. It never guesses you meant `COMMIT`. The
`SELECT` inside `70` showed `6500` only because it ran inside the same transaction, where you see
your own changes.

In an app, the same mistake is worse: the connection may stay open for a long time, holding locks
on Ada's row, while every other request that wants to change it waits (the "too long" question
above).

**Fix:** end every `BEGIN` with `COMMIT` or `ROLLBACK`, and read the last tag. In interactive
`psql`, the prompt helps: after `BEGIN`, `sql_transactions=#` turns into `sql_transactions=*#`. The
`*` means "you're inside a transaction", and stays until you end it.

**Carrying on after an error.**

```sql
{{#include ../../code/sql/sql-transactions/72-carry-on-after-error.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/72-carry-on-after-error.out}}
```

After the failed `UPDATE`, the transaction is aborted, as in Step 7, and Postgres refuses the
`SELECT`, and anything else you send, with `current transaction is aborted, commands ignored until
end of transaction block`. Typing the next statement, or the same one again, won't help: every one
gets the same answer.

**Fix:** `ROLLBACK`, then start again from `BEGIN`. After the `ROLLBACK` above, the same `SELECT`
works. Here's the same thing typed into interactive `psql`
(`docker compose exec db psql -U postgres -d sql_transactions`). Watch the prompt:

```text
psql (18.6 (Debian 18.6-1.pgdg13+2))
Type "help" for help.

sql_transactions=# BEGIN;
BEGIN
sql_transactions=*# UPDATE accounts SET balance_cents = balance_cents - 1000 WHERE id = 1;
UPDATE 1
sql_transactions=*# UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;
ERROR:  new row for relation "accounts" violates check constraint "accounts_balance_cents_check"
DETAIL:  Failing row contains (1, Ada, -13500).
sql_transactions=!# SELECT * FROM accounts ORDER BY id;
ERROR:  current transaction is aborted, commands ignored until end of transaction block
sql_transactions=!# ROLLBACK;
ROLLBACK
sql_transactions=# SELECT * FROM accounts ORDER BY id;
 id | owner | balance_cents
----+-------+---------------
  1 | Ada   |          7500
  2 | Linus |          7500
(2 rows)

sql_transactions=# \q
```

(The version number on the first line depends on when you downloaded Postgres; yours may differ.)
`=*#` means "inside a transaction". After the error it turns into `=!#`: the `!` means "inside a
transaction that has failed". Only `ROLLBACK` (or `COMMIT`, which rolls back) brings back the plain
`=#`. Notice that the first `UPDATE` (Ada −1000), which worked, was thrown away too: a rollback
undoes everything since `BEGIN`. (To undo only part of a transaction, see savepoints, in *More
examples*.)

## More examples

Each file runs like the others:
`docker compose exec -T db psql -U postgres -d sql_transactions < code/sql/sql-transactions/<file>`.
They start from the balances Step 7 left, 7500 and 7500, and put back anything they change.

### Undo only part of a transaction: `SAVEPOINT`

A three-step change: Ada sends Linus 1000 cents (steps 1 and 2), then the bank tries to charge
Linus a fee far bigger than his balance (step 3). The fee fails, but the transfer should still go
through:

```sql
{{#include ../../code/sql/sql-transactions/50-savepoint.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/50-savepoint.out}}
```

`SAVEPOINT transfer_done` is a bookmark inside the transaction, with a name you choose. When the fee
fails and the transaction is aborted, `ROLLBACK TO SAVEPOINT transfer_done` undoes only what came
**after** the bookmark, and un-aborts the transaction. (Its command tag is `ROLLBACK`, too.) The
transfer is still there, 6500 and 8500, and `COMMIT` saves it. Without the `ROLLBACK TO` line, the
`SELECT` would be refused, `COMMIT` would print `ROLLBACK`, and the transfer would be lost with the
fee. The last line puts both balances back to 7500 for the next files. It has no `WHERE` **on
purpose**: it sets every row, which is exactly what it means to do here.

### An order and its items, together

A shop order is one row in `orders` and several rows in `order_items`. An order with no items makes
no sense, so the rows go in together:

```sql
{{#include ../../code/sql/sql-transactions/51-order-and-items.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/51-order-and-items.out}}
```

The first lines reset the two tables, as `01` does for `accounts` (items first, because they point
at orders). The tables use what you learned in [Keys and relations](keys-and-relations.md):
`order_items.order_id` is a foreign key to `orders`, and a `CHECK` insists every quantity is more
than 0. Ada's order goes in: one order row, two items, `COMMIT`. Linus's order asks for 0 copies of
*Emma*, and the item `INSERT` fails. Both items go in one `INSERT`, and a single statement is
all-or-nothing on its own, so neither item is stored. The transaction then throws away the order
row too, and `COMMIT` answers `ROLLBACK`. The final `SELECT`s show Ada's order and nothing of
Linus's. Without `BEGIN`, the order row would have been saved on its own: an order with no items in
it.

### A safer `DELETE`

In [CRUD in SQL](crud-in-sql.md), you learned there's no undo for `DELETE`. Inside a transaction,
there is. Say you want to close "the small account", and you think only Linus has less than 8000
cents:

```sql
{{#include ../../code/sql/sql-transactions/52-safe-delete.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/52-safe-delete.out}}
```

`DELETE 2`: it removed **both** accounts, because Ada has 7500 too. The `SELECT` inside the
transaction shows an empty table, `(0 rows)`. Outside a transaction, that would be the end of both
accounts. Here, `ROLLBACK` brings them back. The habit: `BEGIN`, run the `DELETE` or `UPDATE`, read
the tag and look at the result, and only then decide `COMMIT` or `ROLLBACK`.

### Check before you commit

A transaction lets you look at the result before anyone else can see it. Here, a new account for
Grace, and a count to check it went in:

```sql
{{#include ../../code/sql/sql-transactions/53-count-check.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/53-count-check.out}}
```

Inside the transaction, `count(*)` says `3` accounts: the new one is there, and nothing else
changed. So the file commits, and the table shows Grace. Until that `COMMIT`, no other connection
could see her account. The last line deletes Grace's account again, to leave the table as it was.

## Your turn

The 🟡 and 🔴 exercises start from the balances Step 7 left: 7500 and 7500. If you've been
experimenting, run `01` and `03` again, in that order.

### 🟢 Guided

Run `01`, then `03`, then `04` (Steps 2, 4 and 5), and fill in the blanks from the output of `03`
and `04`:

```text
After 03, Ada has ____ cents and Linus has ____.
Inside 04's transaction, Ada has ____ cents.
After 04's ROLLBACK, Ada has ____ cents.
The command tag that ends 03 is ______; the one that ends 04's transaction is ________.
```

<details><summary>Solution</summary>

```text
{{#include ../../code/sql/sql-transactions/03-transfer-commit.out}}
```

```text
{{#include ../../code/sql/sql-transactions/04-rollback.out}}
```

After `03`, Ada has **7500** cents and Linus has **7500**. Inside `04`'s transaction, Ada has **0**.
After `04`'s rollback, Ada has **7500** again. `03` ends with **`COMMIT`**, and `04`'s transaction
ends with **`ROLLBACK`**. (If `03` gave you 5000 and 10000, you skipped `01`: `03` made a second
transfer on top of an earlier one.)

</details>

### 🟡 Tweak

Linus pays Ada back: move 1000 cents from Linus to Ada, in a transaction, and show the balances
afterwards.

<details><summary>Solution</summary>

`code/sql/sql-transactions/90-linus-to-ada.sql`:

```sql
{{#include ../../code/sql/sql-transactions/90-linus-to-ada.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/90-linus-to-ada.out}}
```

The same shape as `03`, the other way round: `WHERE id = 2` loses 1000, `WHERE id = 1` gains 1000.
Ada ends up with 8500 and Linus with 6500, and the total is still 15000. Each run makes another
transfer, so a second run gives 9500 and 5500.

</details>

### 🔴 From scratch

Write a transfer that would **overdraw** an account: send more money than the sender has. Show the
balances before and after, and prove they didn't change.

<details><summary>Solution</summary>

`code/sql/sql-transactions/91-overdraw.sql`:

```sql
{{#include ../../code/sql/sql-transactions/91-overdraw.sql}}
```

```text
{{#include ../../code/sql/sql-transactions/91-overdraw.out}}
```

Ada tries to send Linus 100000 cents. The book ran it after the 🟡 solution, so the balances start
at 8500 and 6500; yours may differ, and that's fine. What matters is that **the two tables match**.
Linus's `UPDATE 1` worked, Ada's broke the `CHECK` (she'd have had `-91500`), the transaction was
aborted, and `COMMIT` answered `ROLLBACK`, which threw Linus's 100000 cents away. Putting Linus's
half first, as here, is the tougher test: it proves the transaction undoes a change that had already
succeeded.

</details>

## Quick check

<div class="quiz" data-topic="sql-transactions"></div>

## Remember this

- A **transaction** groups statements so they all happen or none do: `BEGIN; …; COMMIT;`.
- `COMMIT` keeps every change at once. `ROLLBACK` throws every change since `BEGIN` away.
- After an error inside a transaction, it's **aborted**: every statement is refused until you end
  it, and `COMMIT` answers `ROLLBACK`. End it and start again.
- A transaction that's never committed is rolled back when its connection closes.
- Without `BEGIN`, each statement is its own tiny transaction (**autocommit**).
- **ACID**: all-or-nothing, rules always hold, others don't see half-done work, committed means
  saved. Keep transactions short.

## Go deeper

- [Transactions (tutorial)](https://www.postgresql.org/docs/18/tutorial-transactions.html) — The official tutorial page: a bank transfer, BEGIN, COMMIT, ROLLBACK and savepoints.
- [BEGIN](https://www.postgresql.org/docs/18/sql-begin.html) — The reference page for BEGIN and its options.

<!-- next:start -->

**Next:**

- Build it: a library database (coming soon)
- Cheat sheet: SQL (coming soon)

<!-- next:end -->
