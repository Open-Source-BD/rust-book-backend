# Glossary

Every technical word in this book, in plain language. Lessons link here the first time they use a
word. Terms are in A–Z order.

### ACID

Four promises a [database](#database) makes about every [transaction](#transaction), named by their
first letters: **A**tomic (all of its changes happen, or none do), **C**onsistent (every
[constraint](#constraint) still holds afterwards), **I**solated (others don't see its half-done
work), **D**urable (once committed, it stays, even if the power goes out).
**First used in:** [SQL transactions](a1-postgres/sql-transactions.md)

### API

A set of addresses a program answers, so that other programs — a website, a phone app, another
service — can ask it for data or ask it to do something (short for Application Programming
Interface).
**First used in:** [Introduction](introduction.md)

### API key

A secret text that a program sends with each [request](#request), usually in a [header](#header)
such as `x-api-key`, to show it's allowed to call an [API](#api): a password for a program rather
than for a person. The [server](#server) answers `401` when it's missing and `403` when it's wrong.
**First used in:** [Custom extractors](a2-axum/custom-extractors.md)

### Async

Code that can pause while it waits for something slow (the network, the [database](#database)…) so
the computer can do other work in the meantime. In Rust you mark such a function `async fn` and
pause it with `.await`.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Backend

The part of a website or app that runs on a [server](#server) instead of on your own device: it
stores data, checks rules, and sends answers back. Think of a restaurant kitchen — customers never
walk into it, but every order goes through it before food comes out.
**First used in:** [Introduction](introduction.md)

### Backslash command

A short command that starts with a backslash (`\`), such as `\dt` or `\q`, typed into
[psql](#psql). It isn't [SQL](#sql): psql handles it itself instead of sending it to the
[database](#database) server, so it needs no semicolon and ends at the end of the line.
**First used in:** [Tables, rows and psql](a1-postgres/tables-rows-and-psql.md)

### Cast

Turning a value into a different [data type](#data-type), such as the text `'42'` into the number
`42`. In Postgres you write it with two colons: `'42'::integer`. If the value can't be turned into
that type, Postgres refuses with an error.
**First used in:** [Postgres data types](a1-postgres/postgres-data-types.md)

### Check constraint

A [constraint](#constraint) that tests every new or changed [row](#row) against a condition you
write, such as `CHECK (stars >= 1 AND stars <= 5)`, and refuses the row if the condition is false.
**First used in:** [Keys and relations](a1-postgres/keys-and-relations.md)

### Client

A program that asks for information or asks for something to be done, and shows the answer to a
person. A web browser or a phone app is a client; it talks to a [server](#server).
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Column

One named field that every [row](#row) in a [table](#table) has, such as `name` or `city`. A
column holds the same kind of value in every row: all text, or all numbers, and so on.
**First used in:** [What is a database?](a1-postgres/what-is-a-database.md)

### Commit

The command that ends a [transaction](#transaction) and makes all of its changes permanent, in one
step: `COMMIT;`. Until then, nobody else can see them, and they can still be undone with a
[rollback](#rollback).
**First used in:** [CRUD in SQL](a1-postgres/crud-in-sql.md)

### Constraint

A rule that a [table](#table) enforces on every row that goes into it, such as "this
[column](#column) is never empty" (`NOT NULL`) or "no two rows share this value" (`UNIQUE`).
Postgres refuses any `INSERT` or `UPDATE` that would break it, whichever program sends it.
**First used in:** [Keys and relations](a1-postgres/keys-and-relations.md)

### Container

A small, sealed-off box that runs a program together with its own copy of everything that program
needs, kept separate from the rest of your computer. [Docker](#docker) starts, stops and manages
containers.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### CORS

Cross-Origin Resource Sharing: the rules a web browser follows before it lets a web page's
JavaScript read an answer from a **different** website (a different *origin*: `https://myapp.com`
and `https://api.myapp.com` are two origins). Your [server](#server) takes part by sending
[headers](#header) such as `access-control-allow-origin`; the browser reads them and decides.
Programs that aren't browsers, such as curl, ignore CORS completely.
**First used in:** [Middleware and Tower layers](a2-axum/middleware-and-tower-layers.md)

### Crate

A package of Rust code that you can add to your own project, written by you or by someone else. It
is Rust's word for what other languages call a library or a package.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### CRUD

The four things a program does with stored data: **C**reate, **R**ead, **U**pdate and **D**elete.
In [SQL](#sql) they are `INSERT`, `SELECT`, `UPDATE` and `DELETE`; over [HTTP](#http) they are
usually `POST`, `GET`, `PUT` and `DELETE`.
**First used in:** [CRUD in SQL](a1-postgres/crud-in-sql.md)

### Data type

The kind of value a [column](#column) (or any value) holds, such as `integer`, `text`, `boolean` or
`date`. It's a promise the [database](#database) enforces: a value that doesn't fit the type is
refused.
**First used in:** [Postgres data types](a1-postgres/postgres-data-types.md)

### Database

A program that stores data on disk in an organized way, so the data survives after your program
stops, and lets you search, add, change and delete it safely. This book uses a database called
Postgres.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Dependency

A [crate](#crate) your project uses but didn't write itself, such as Axum or Tokio. You list each
one under `[dependencies]` in the project's `Cargo.toml`, and Cargo downloads and builds it for you.
**First used in:** [Hello, Axum](a2-axum/hello-axum.md)

### Deserialize

To build a value in your program, such as a Rust struct, out of data that arrived as text, such as
a [query string](#query-string) or [JSON](#json). The opposite, turning a value into text, is to
[serialize](#serialize). In Rust, the [serde](#serde) crate does both, and `#[derive(Deserialize)]`
gives a struct the ability to be built this way.
**First used in:** [Path and Query extractors](a2-axum/path-and-query-extractors.md)

### Docker

A tool that runs programs inside [containers](#container), so that everyone runs the exact same
setup on their own computer, no matter what is already installed there.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Endpoint

One specific address that a [server](#server) understands and answers, such as `/users` or
`/health`. Each endpoint usually does one job.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Environment variable

A named value that lives outside your code — set on the computer, or in a settings file — that your
program can read when it starts. This book stores the [database](#database)'s location and password
in environment variables, so they are never written directly into the code.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Error type

A type whose values describe what went wrong, instead of what went right. In an Axum app it's
usually one `enum` with a variant for each kind of failure, such as "book not found", so that every
[handler](#handler) reports its errors the same way and each one becomes the right
[status code](#status-code).
**First used in:** [Error handling in Axum](a2-axum/error-handling-in-axum.md)

### EXPLAIN

A command you put in front of a [query](#query) to see the plan Postgres will use to run it: for
example, whether it reads every [row](#row) of a [table](#table) (`Seq Scan`) or uses an
[index](#index). `EXPLAIN (ANALYZE)` also runs the query and shows what really happened.
**First used in:** [Indexes](a1-postgres/indexes.md)

### Extractor

A [handler](#handler) parameter that Axum fills from the [request](#request) before the handler
runs, such as `Path` (values from the path) or `Query` (values from the
[query string](#query-string)). If the request doesn't fit, Axum answers with an error, usually
`400 Bad Request`, and the handler never runs.
**First used in:** [Routes and HTTP methods](a2-axum/routes-and-methods.md)

### Foreign key

A [column](#column) whose values must match the [primary key](#primary-key) of a row in another
[table](#table), such as `books.author_id` pointing at `authors.id`. Postgres refuses a value that
points at nothing, and by default refuses to delete a row that others still point at.
**First used in:** [Keys and relations](a1-postgres/keys-and-relations.md)

### Framework

A set of tools and rules that handles the repeated parts of a task, such as receiving a
[request](#request) and matching it to a [route](#route), so you write only the parts that are
specific to your app. Axum is the framework this book uses to build a [backend](#backend).
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Handler

The function that runs when a [request](#request) matches a [route](#route); it receives the
request's data and returns the [response](#response).
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Header

One `name: value` line of extra information at the top of a [request](#request) or
[response](#response), before the body, such as `content-type: text/html` (what the body is) or
`cache-control: no-store` (don't keep a copy). Header names don't care about upper or lower case.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### HTML

The language web pages are written in (short for HyperText Markup Language). Text is wrapped in
*tags*, such as `<h1>…</h1>` for a big heading or `<a href="/about">…</a>` for a link, and a
browser reads the tags to draw the page.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### HTTP

The set of rules computers follow to ask for and send back information over the web (short for
HyperText Transfer Protocol). Every [request](#request) and [response](#response) in this book
travels using HTTP.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### HTTP method

The verb at the start of every HTTP [request](#request) that says what the client wants done with
the path: `GET` to read, `POST` to create, `PUT` to replace, `PATCH` to change part of something,
`DELETE` to remove. One path can answer several methods, each with its own [handler](#handler).
**First used in:** [Introduction](introduction.md)

### Index

A sorted list of one [column](#column)'s values, each with the addresses of its [rows](#row), kept
next to a [table](#table), like the index at the back of a book. It lets Postgres find matching rows
without reading the whole table. It costs disk space, and every write must update it.
**First used in:** [Indexes](a1-postgres/indexes.md)

### Join

Combining rows from two [tables](#table) into one answer, by matching a column in one with a column
in the other: `FROM books JOIN authors ON authors.id = books.author_id` puts each book next to its
author. A `LEFT JOIN` also keeps the left table's rows that have no match.
**First used in:** [Keys and relations](a1-postgres/keys-and-relations.md)

### JSON

A plain-text way of writing structured data, using `{ }` for an object and `[ ]` for a list, that
both people and programs can read. Most [request](#request) and [response](#response) bodies in
this book are JSON.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Layer

In Axum, a wrapper added with `.layer(…)` around the [routes](#route) added before it. It sees each
[request](#request) on its way in and each [response](#response) on its way out, so it can run code
for many [handlers](#handler) at once. [Middleware](#middleware) is added to Axum as a layer. The
last `.layer` call is the outermost wrapper: it sees a request first and its response last.
**First used in:** [Middleware and Tower layers](a2-axum/middleware-and-tower-layers.md)

### Localhost

A name that always means "this computer" — used when a program on your machine talks to a
[server](#server) that is also running on your machine, without going out to the internet.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Middleware

Code that runs around [handlers](#handler), for every [request](#request) or for a group of them:
before the handler, to look at the request, change it or refuse it, and after it, to look at or
change the [response](#response). Request logging, [CORS](#cors) and [timeouts](#timeout) are
usually middleware. In Axum, you add it as a [layer](#layer).
**First used in:** [Routes and HTTP methods](a2-axum/routes-and-methods.md)

### Module

A named part of a Rust program, with its own items (functions, types…) that are private to it
unless marked `pub`. A file is usually a module: `mod books;` in `main.rs` adds `books.rs` to the
program as the module `books`. Rust for Humans teaches them in
[Modules and crates](https://open-source-bd.github.io/rustbook-for-human/language-basics/modules-and-crates.html).
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### NULL

SQL's marker for "unknown" or "missing": no value at all. It isn't `0` and it isn't empty text
`''`. Test for it with `IS NULL` or `IS NOT NULL`; comparing with `= NULL` never matches.
**First used in:** [Postgres data types](a1-postgres/postgres-data-types.md)

### ORM

A library that lets you work with [database](#database) rows as normal Rust structs instead of
writing [SQL](#sql) strings by hand (short for Object-Relational Mapper). SeaORM is the ORM this
book uses.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Port

A numbered door on a computer; one program listens behind each door, e.g. our server on 3000 and
Postgres on 5433.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Primary key

The [column](#column) (or columns) that identifies each [row](#row) of a [table](#table): its value
is unique and never [NULL](#null), like a member number. Other tables point at a row through its
primary key.
**First used in:** [Keys and relations](a1-postgres/keys-and-relations.md)

### psql

Postgres's own command-line program. You type [SQL](#sql) into it (or feed it a file of SQL), it
sends the SQL to the [database](#database) server, and it prints the answer as a text table.
**First used in:** [What is a database?](a1-postgres/what-is-a-database.md)

### Query

One question or instruction you send to a [database](#database), written in [SQL](#sql), such as
`SELECT * FROM friends;`. Strictly, a query is a question that reads data, but people often call
any SQL statement a query.
**First used in:** [What is a database?](a1-postgres/what-is-a-database.md)

### Query planner

The part of Postgres that decides **how** to run each [query](#query): read the whole
[table](#table), or use an [index](#index), and which one. It uses statistics about the table's
contents to pick the plan it expects to be fastest. [EXPLAIN](#explain) shows its choice.
**First used in:** [Indexes](a1-postgres/indexes.md)

### Query string

The part of an address after the `?`, holding optional extra options as `name=value` pairs joined
with `&`, such as `?q=rust&page=2`. The path says *what* you want; the query string says *how you'd
like it*.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Rejection

The answer an [extractor](#extractor) sends instead of running the [handler](#handler), when the
[request](#request) doesn't fit: for example `400 Bad Request` when a `Path` value isn't a number.
Built-in extractors have their own; an extractor you write chooses its own with `type Rejection`.
**First used in:** [Path and Query extractors](a2-axum/path-and-query-extractors.md)

### Request

The message a [client](#client) sends to ask a [server](#server) for something, or ask it to do
something — such as "give me user 4" or "create this order".
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Response

The message a [server](#server) sends back to the [client](#client) that made a
[request](#request), carrying a [status code](#status-code) and usually some data.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### REST

A common style for designing a [backend](#backend) where each [endpoint](#endpoint) stands for one
"thing" (like a user or an order), and you act on it using the request's method — GET to read, POST
to create, and so on.
**First used in:** [Introduction](introduction.md)

### Rollback

The command that ends a [transaction](#transaction) and throws away every change made inside it, as
if it never started: `ROLLBACK;`. Postgres also rolls back on its own when a transaction fails or
its connection closes before a [commit](#commit).
**First used in:** [CRUD in SQL](a1-postgres/crud-in-sql.md)

### Route

A rule that pairs one address and one HTTP method (like GET `/users`) with the
[handler](#handler) that should answer it.
**First used in:** [Introduction](introduction.md)

### Row

One record in a [table](#table): one friend, one book, one order. A row has one value for each
[column](#column) of its table.
**First used in:** [What is a database?](a1-postgres/what-is-a-database.md)

### Runtime

The program running underneath your code that carries out [async](#async) work, deciding when each
paused task picks back up. Tokio is the runtime this book uses.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Serde

The Rust [crate](#crate) that turns Rust values into text formats such as [JSON](#json)
([serialize](#serialize)) and builds them back out of text ([deserialize](#deserialize)). Its name
is *ser* + *de*. Axum's `Json` and `Query` extractors use it.
**First used in:** [Path and Query extractors](a2-axum/path-and-query-extractors.md)

### Serialize

To turn a value in your program, such as a Rust struct, into text that can be sent or saved, such
as [JSON](#json). The opposite is to [deserialize](#deserialize). In Rust, `#[derive(Serialize)]`
from the [serde](#serde) crate gives a struct this ability.
**First used in:** [Path and Query extractors](a2-axum/path-and-query-extractors.md)

### Server

A program that waits for [request](#request)s and sends back [response](#response)s. It keeps
running in the background, ready for the next request at any time.
**First used in:** [Introduction](introduction.md)

### SQL

The language used to ask a [database](#database) questions and give it instructions, such as "add
this row" or "find every order from this user" (short for Structured Query Language).
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### State

Data an Axum app keeps for as long as the [server](#server) runs, handed to the Router once with
`.with_state(…)` and shared by every [handler](#handler), which reaches it with the `State`
[extractor](#extractor). It lives in memory, so it's lost when the program stops.
**First used in:** [Shared state](a2-axum/shared-state.md)

### Status code

A short number a [server](#server) sends back with every [response](#response) to say what
happened, such as `200` for success or `404` for "not found".
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Table

A named list of data inside a [database](#database), laid out like a spreadsheet: the
[columns](#column) go across the top, and each [row](#row) below them is one record.
**First used in:** [What is a database?](a1-postgres/what-is-a-database.md)

### Timeout

A time limit for waiting. If the work isn't finished when the time is up, the side that waits gives
up and treats it as a failure: a health check counts as failed, or a [server](#server) answers
`408 Request Timeout` instead of waiting any longer for a slow [handler](#handler).
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Transaction

A group of [SQL](#sql) statements that the [database](#database) treats as one: either all of their
changes are kept, or none are. It starts with `BEGIN` and ends with a [commit](#commit) (keep) or a
[rollback](#rollback) (throw away).
**First used in:** [CRUD in SQL](a1-postgres/crud-in-sql.md)

### WHERE

The part of an [SQL](#sql) statement that picks which [rows](#row) it works on, such as
`WHERE id = 4`. Postgres keeps only the rows where the condition is true. A `SELECT` without it
reads every row; an `UPDATE` or `DELETE` without it changes or removes every row.
**First used in:** [Postgres data types](a1-postgres/postgres-data-types.md)
