# Path and Query extractors

> **Beginner** · Part A2 · Axum

## By the end of this lesson

- You can read values from the URL path into typed Rust variables.
- You can read query-string options, including optional ones.
- You know what Axum answers when the URL can't be parsed.

## What & why

Extractors pull data out of the request: `Path` for /books/42 and `Query` for ?q=rust&page=2 — typed, checked, and rejected with 400 when wrong.

Back to the library counter from [Routes and HTTP methods](routes-and-methods.md). You hand in a
form that says *show me book number 42*. The librarian doesn't read the form herself: a **clerk**
at the counter reads it first. The clerk finds the box marked "book number", reads `42`, checks
that it really is a number, and hands the librarian a clean `42`. If you wrote `dune` in the number
box, the clerk hands the form straight back to you, *"that's not a number"*, and the librarian is
never disturbed.

In Axum, the librarian is your [handler](../glossary.md#handler), and the clerks are
[**extractors**](../glossary.md#extractor): handler parameters that Axum fills from the
[request](../glossary.md#request) **before** your handler runs. So far, your handlers answered
`/books/{id}` without knowing which book was asked for, because none of them read the `{id}`. This
lesson adds two clerks:

- **`Path`** reads values out of the **path**, the part of the address that says *which thing*:
  the `42` in `/books/42`.
- **`Query`** reads values out of the [**query string**](../glossary.md#query-string), the part after
  the `?` that says *how you'd like it*: `rust` and `2` in `/search?q=rust&page=2`.

Both clerks hand your handler values of the **type you ask for**: a whole number, a piece of text,
or a struct of your own. And both check their work: if the address doesn't fit that type, Axum
answers `400 Bad Request` itself, and your handler never runs. That's the third idea in this
lesson: what those `400` answers say, and why you get them for free.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/path-and-query-extractors`. Its `Cargo.toml` has one new line:
`serde`.

```toml
{{#include ../../code/topics/path-and-query-extractors/Cargo.toml}}
```

### Line by line

`[package]` · `name = "path-and-query-extractors"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p path-and-query-extractors`.
- **How:** each lesson is its own small project, named after the lesson.
- **Remove it and…** (`name`) Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "take the edition and publish settings from the workspace's `code/Cargo.toml`".
- **Why:** every lesson shares `edition = "2024"` and `publish = false`.
- **How:** [Hello, Axum, Step 2](hello-axum.md#step-2-the-books-copy-of-the-same-project) explains
  `.workspace = true` in full.
- **Remove it and…** (`edition`) Cargo falls back to edition 2015, where `async fn` isn't allowed.

`[dependencies]` · `axum.workspace = true` · `tokio.workspace = true`
- **What:** Axum 0.8.9, and Tokio 1.53.1 with the features our `main` needs.
- **Why:** the two crates every Axum server needs. `Path` and `Query` come with Axum.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`serde.workspace = true`
- **What:** the new line. [**serde**](../glossary.md#serde) is Rust's standard crate for turning
  Rust values into text ([*ser*ializing](../glossary.md#serialize)) and building them back out of
  text ([*de*serializing](../glossary.md#deserialize)): its name is *ser* + *de*.
- **Why:** `Query` fills a struct of yours with the query string's options. serde is the part that
  knows how to build a struct out of `q=rust&page=2`; Step 6 shows how you ask it to.
- **How:** the version and its features are in `code/Cargo.toml`: Step 2 shows that line.
- **Remove it and…** ``error[E0432]: unresolved import `serde` ``, pointing at `use
  serde::Deserialize;`, with a `help:` line that tells you to run `cargo add serde`.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the response bodies, without
  opening a port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

### Step 2: the `serde` line in the workspace

Here is the line that `serde.workspace = true` points at, in `code/Cargo.toml`:

```toml
{{#include ../../code/Cargo.toml:serde}}
```

### Line by line

`serde =`
- **What:** the crate's name, as it's listed on [crates.io](https://crates.io/crates/serde), Rust's
  public shelf of crates.
- **Why:** every project that says `serde.workspace = true` gets this one line's settings.
- **How:** the braces `{ … }` hold more than a version: here, a version **and** a list of features.
- **Remove it and…** `serde.workspace = true` has nothing to point at, and Cargo stops before
  building anything: ``error inheriting `serde` from workspace root manifest's
  `workspace.dependencies.serde` ``.

`version = "1.0.229"`
- **What:** the version of serde the whole book uses.
- **Why:** pinned, so your build matches the book's.
- **How:** Cargo reads it as "1.0.229 or any newer 1.x", and `Cargo.lock` records the exact one.
- **Remove it and…** Cargo stops before building: ``dependency (serde) specified without
  providing a local path, Git repository, version, or workspace dependency to use``.

`features = ["derive"]`
- **What:** switches on serde's optional `derive` feature, which adds the `#[derive(Deserialize)]`
  (and `#[derive(Serialize)]`) shortcuts.
- **Why:** with it, one line above a struct is enough to teach serde how to build that struct. Step
  6 uses it.
- **How:** a crate's **features** are optional parts you switch on by name, so a project only
  compiles the parts it uses. serde keeps `derive` optional because some programs never need it.
- **Remove it and…** (write `serde = "1.0.229"`) the build stops at the struct with
  ``cannot find derive macro `Deserialize` in this scope``, and a note that says it plainly:
  `` `Deserialize` is imported here, but it is only a trait, without a derive macro ``. The name is
  there; the shortcut isn't.

In your own project, the dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, and the test crates from
`cargo add --dev tower http-body-util --features tower/util`.

### Step 3: the `use` lines

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/src/main.rs:imports}}
```

📁 Full code: code/topics/path-and-query-extractors · ▶ Run it: `cd code && cargo run -p path-and-query-extractors`

### Line by line

`use axum::{` · `};`
- **What:** brings four names from Axum into this file.
- **Why:** so the code can write `Path` instead of `axum::extract::Path`.
- **How:** the braces list several names from one crate; `cargo fmt` puts one per line.
- **Remove it and…** none of the four names is known, and the build stops with many errors.

`Router` · `routing::get`
- **What:** Axum's table of [routes](../glossary.md#route), and the function that makes a route
  answer `GET`.
- **Why:** Step 7 builds the Router; every route here is a `GET`, so you can try each one in a
  browser too.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** (`get`) ``cannot find function `get` in this scope``.

`extract::{Path, Query}`
- **What:** the two extractors, from Axum's `extract` module (the part of Axum that holds every
  built-in extractor).
- **Why:** `Path` reads the path; `Query` reads the query string.
- **How:** the inner braces take two names from the same module in one go.
- **Remove it and…** (`Path`) ``cannot find type `Path` in this scope``, and the same for `Query`.

`use serde::Deserialize;`
- **What:** brings in serde's `Deserialize`, both the ability and the `#[derive(Deserialize)]`
  shortcut for it.
- **Why:** Step 6's struct uses the shortcut.
- **How:** it's a separate `use` line because it comes from a different crate: serde, not Axum.
- **Remove it and…** ``cannot find derive macro `Deserialize` in this scope``, pointing at
  `#[derive(Deserialize)]`.

### Step 4: read a value from the path

Two handlers that read the path: one reads one value, the other reads two.

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/src/main.rs:path}}
```

📁 Full code: code/topics/path-and-query-extractors · ▶ Run it: `cd code && cargo run -p path-and-query-extractors`

### Line by line

`async fn show_book(Path(id): Path<u32>) -> String {`

This line is the heart of the lesson, so here it is in three pieces.

`Path<u32>`, after the colon
- **What:** the parameter's **type**: "a `Path` holding a `u32`". A `u32` is a whole number from
  `0` to `4294967295`: no minus sign, no decimal point.
- **Why:** the type is how you **ask** for an extractor. Axum looks at the type of each parameter
  and sends the matching clerk: `Path` means "read the path". The `u32` inside says what to turn
  the text into.
- **How:** everything in an address is text, so the path segment arrives as the text `"42"`. The
  `Path` extractor turns it into the number `42`. If it can't (`"dune"`), Axum answers `400` and
  `show_book` never runs.
- **Remove it and…** (write `id: u32`, with no `Path`) the compiler stops with the
  ``the trait bound `…: Handler<_, _>` is not satisfied`` error from Handlers and IntoResponse,
  pointing at the route:
  a bare `u32` doesn't say *where* the number comes from (path? query? body?), so Axum can't fill
  it.

`Path(id)`, before the colon
- **What:** a **pattern**. It opens the `Path` box and calls what's inside `id`.
- **Why:** you don't want the box, you want the number. With the pattern, `id` **is** the `u32`,
  ready to use.
- **How:** `Path` is a box with one thing inside: in Rust terms, a *tuple struct*, a struct whose
  one field has no name. Where a parameter name usually goes, Rust also lets you write a pattern
  with the same shape as the value: `Path(id)` matches a `Path` with something inside, and calls
  that something `id`. It's the same pattern as in `let Path(id) = …;`. Rust for Humans'
  [Pattern matching](https://open-source-bd.github.io/rustbook-for-human/language-basics/pattern-matching.html)
  lesson covers patterns in full.
- **Remove it and…** (write `id: Path<u32>`) `id` is now the box, not the number, and
  `format!("book number {id}")` stops with ``error[E0277]: `axum::extract::Path<u32>` doesn't
  implement `std::fmt::Display` ``: Rust doesn't know how to print a box. (Writing `id.0` would
  take the number out, `.0` being "the first field", but the pattern is the usual way.)

`-> String {` · `format!("book number {id}")`
- **What:** answers with the text `book number 42`.
- **Why:** so you can see which number the handler received.
- **How:** `{id}` inside `format!` is replaced by the value of `id`. A `String` is sent as
  `200 OK`, plain text, as in [Handlers and IntoResponse](handlers-and-into-response.md).
- **Remove it and…** (the `-> String`) the function returns nothing, and the compiler stops with
  `mismatched types`.

`async fn show_chapter(Path((book, chapter)): Path<(u32, u32)>) -> String {`
- **What:** reads **two** values from the path: a book number and a chapter number.
- **Why:** an address like `/books/3/chapters/7` has two placeholders, `{book}` and `{chapter}`.
- **How:** the type is a `Path` holding a **tuple** of two `u32`s, `(u32, u32)`, and the pattern
  has the same shape: `Path((book, chapter))`, a box holding a pair. The outer brackets belong to
  `Path(…)`, the inner ones to the pair. The values are read **left to right**: the first
  placeholder in the route goes into `book`, the second into `chapter`. The names inside the braces
  of the route don't matter to a tuple; the order does.
- **Remove it and…** (one of the two `u32`s, leaving `Path<u32>`) the program compiles, but every
  request to that route gets a `500` error. *Common mistakes* shows it.

`format!("book {book}, chapter {chapter}")`
- **What:** answers with `book 3, chapter 7`.
- **Why:** it shows both values arrived, in the right order.
- **How:** each `{name}` is replaced by the variable of that name.
- **Remove it and…** the handler has nothing to return.

### Step 5: what happens to `/books/42`

Here is the whole journey of one request, from the address to your handler's answer:

```text
GET /books/42
  │  1. the Router matches the route "/books/{id}", so {id} is the text "42"
  ▼
Path<u32>
  │  2. the Path extractor turns "42" into the number 42
  │     (for "dune" it stops here and answers 400 Bad Request)
  ▼
show_book(Path(42))
  │  3. the pattern Path(id) opens the box: id = 42
  ▼
"book number 42"      4. the String becomes a 200 OK response
```

### Line by line

`GET /books/42` · `1. the Router matches the route`
- **What:** the request arrives, and the Router finds the route whose path fits.
- **Why:** before anything can be read, Axum must know which handler, and so which extractors,
  apply.
- **How:** `{id}` matches any one segment, so at this point `42` and `dune` both match. The
  Router only remembers that `{id}` is `"42"`, as text.
- **Remove it and…** (no matching route) Axum answers `404`, and no extractor runs.

`Path<u32>` · `2. the Path extractor turns "42" into the number 42`
- **What:** the clerk's check: can this text become a `u32`?
- **Why:** so your handler receives a real number, never text that might not be one.
- **How:** it runs **before** your handler. If the check fails, Axum sends the extractor's own
  answer, a **rejection**, and the handler isn't called at all.
- **Remove it and…** you'd get text, and have to check it yourself in every handler.

`show_book(Path(42))` · `3. the pattern Path(id) opens the box`
- **What:** Axum calls your handler with the filled-in `Path`.
- **Why:** this is the first moment your code runs.
- **How:** the pattern in the parameter unpacks it, so the body sees `id = 42`.
- **Remove it and…** (the pattern) you'd hold the box, not the number, as in Step 4.

`"book number 42"` · `4. the String becomes a 200 OK response`
- **What:** your answer goes back to the client.
- **Why:** it's the same `IntoResponse` step as in the last lesson.
- **How:** a `String` becomes `200 OK` with `content-type: text/plain`.
- **Remove it and…** there's no answer to send.

### Step 6: read options from the query string

The query string is different from the path: it can hold **several** options, **by name**, and
some of them may be missing. So instead of a number or a tuple, you describe the options with a
struct:

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/src/main.rs:query}}
```

📁 Full code: code/topics/path-and-query-extractors · ▶ Run it: `cd code && cargo run -p path-and-query-extractors`

### Line by line

`#[derive(Deserialize)]`
- **What:** asks serde to write the code that builds a `Search` out of text such as
  `q=rust&page=2`.
- **Why:** `Query` needs a type it can fill from the query string, and it can only fill types that
  know how to be **deserialized**: built from outside data.
- **How:** `#[derive(…)]` is an instruction to the compiler: "write this for me". `Deserialize` is
  a
  [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html),
  a named ability, like `IntoResponse` in the last lesson; `derive` writes the ability for your
  struct by looking at its fields. You never see that code, but it's there.
- **Remove it and…** the program no longer compiles. *Common mistakes* shows the error, which
  doesn't mention `Deserialize` at first.

`struct Search {` · `}`
- **What:** a struct listing the options this route understands.
- **Why:** each field is one `name=value` option. The struct is the clerk's form: which boxes
  exist, and what goes in each.
- **How:** the struct's name is yours; the **field names** are not: they must match the names in
  the query string exactly.
- **Remove it and…** `Query<Search>` has no type to fill.

`q: String,`
- **What:** the option `q` (short for *query*, the search words), as text.
- **Why:** a search needs something to search for, so `q` is **required**.
- **How:** a plain type (not `Option`) means "must be there". `/search?q=rust` fills it with
  `"rust"`; `/search` alone is rejected with `400`.
- **Remove it and…** `q` is no longer read, and `params.q` below stops the build with ``no field
  `q` on type `Search` ``.

`page: Option<u32>,`
- **What:** the option `page`, a whole number that may be missing.
- **Why:** most people want page 1, so they shouldn't have to say so.
- **How:** an `Option<u32>` holds either `Some(number)` or `None` (nothing). If the query string
  has `page=2`, the field is `Some(2)`; if there's no `page`, it's `None`, and that's not an error.
  Rust for Humans'
  [Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
  lesson covers `Option` in full.
- **Remove it and…** (write `page: u32`) `page` becomes required: `/search?q=rust` gets `400`,
  with the body ``Failed to deserialize query string: missing field `page` `` (tried in a scratch
  copy).

`async fn search(Query(params): Query<Search>) -> String {`
- **What:** the handler, with one extractor: a `Query` holding a `Search`.
- **Why:** the type `Query<Search>` asks the query-string clerk to fill in a `Search`.
- **How:** the same box-and-pattern idea as `Path(id)`: `Query(params)` opens the box, and
  `params` is the `Search` struct.
- **Remove it and…** (the parameter) nothing reads the query string, and `params` doesn't exist.

`let page = params.page.unwrap_or(1);`
- **What:** the page number, or `1` if none was given.
- **Why:** the rest of the code wants a plain number, not "maybe a number".
- **How:** `unwrap_or(1)` says: "if it's `Some(n)`, give me `n`; if it's `None`, give me `1`". So
  `page` is a `u32` from here on.
- **Remove it and…** (write `let page = params.page;`) `page` is still an `Option<u32>`, and
  `{page}` in `format!` stops the build: an `Option` has no default way to be printed.

`format!("searching for {:?}, page {page}", params.q)`
- **What:** answers with `searching for "rust", page 1`.
- **Why:** so you can see exactly what arrived.
- **How:** `{:?}` prints the value's **debug** form: for text, that means in double quotes. The
  quotes show you exactly where the text starts and ends, which matters when it has spaces in it
  (Run it has one). It takes the value after the comma, `params.q`, because a `{name}` placeholder
  can hold only a plain variable name, not `params.q`.
- **Remove it and…** (write `{}` instead of `{:?}`) you'd see `searching for rust`, with no quotes.

### Step 7: the Router

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/src/main.rs:app}}
```

📁 Full code: code/topics/path-and-query-extractors · ▶ Run it: `cd code && cargo run -p path-and-query-extractors`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router, starting from an empty one.
- **Why:** a separate function, so `main` and the tests use the same Router.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** there's no Router to add routes to.

`.route("/books/{id}", get(show_book))`
- **What:** one placeholder, `{id}`, for `show_book`'s one value.
- **Why:** the placeholder is where the value comes from.
- **How:** the number of placeholders must match what the handler asks for: one here, for a
  `Path<u32>`.
- **Remove it and…** `/books/42` gets `404`.

`.route("/books/{book}/chapters/{chapter}", get(show_chapter))`
- **What:** two placeholders, for `show_chapter`'s pair.
- **Why:** a chapter lives inside a book, and the path says so.
- **How:** `{book}` fills the pair's first value, `{chapter}` the second, left to right.
- **Remove it and…** `/books/3/chapters/7` gets `404`.

`.route("/search", get(search))`
- **What:** the search route. It has **no** placeholder and no `?` in it.
- **Why:** the query string isn't part of the route: `/search`, `/search?q=rust` and
  `/search?q=rust&page=2` all match this one route. The `Query` extractor reads the rest.
- **How:** the Router matches on the path only, and ignores everything from the `?` on.
- **Remove it and…** `/search?q=rust` gets `404`.

### Step 8: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/src/main.rs:main}}
```

📁 Full code: code/topics/path-and-query-extractors · ▶ Run it: `cd code && cargo run -p path-and-query-extractors`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's start, running on the Tokio runtime.
- **Why:** `main` needs `.await`, and `#[tokio::main]` is what allows that.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line of `main`.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`let listener = …bind("127.0.0.1:3000")…;` · `println!(…);`
- **What:** claims port 3000, then prints `Listening on http://127.0.0.1:3000`.
- **Why:** a server must own a port before anyone can reach it.
- **How:** if the port is taken, `.expect` stops the program with our message.
- **Remove it and…** there's no port for curl to reach.

`axum::serve(listener, app())` · `.await` · `.expect(…);`
- **What:** builds the Router with `app()`, then answers requests forever.
- **Why:** this is the loop that makes the program a server.
- **How:** for every request, Axum finds the route, runs the handler's extractors, and then, only
  if they all succeed, calls the handler.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p path-and-query-extractors
```

```text
   Compiling path-and-query-extractors v0.1.0 (/Users/you/rust-book-backend/code/topics/path-and-query-extractors)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.06s
     Running `target/debug/path-and-query-extractors`
Listening on http://127.0.0.1:3000
```

In the second terminal, try the path first:

```bash
{{#include ../../code/topics/path-and-query-extractors/http/01-path.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/01-path.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`/books/42`:** `200 OK`, `book number 42`. The text `"42"` from the address arrived in
  `show_book` as the number `42`.
- **`/books/3/chapters/7`:** `book 3, chapter 7`. Both values, in order.
- **`/books/dune`:** `HTTP/1.1 400 Bad Request`, and a body you didn't write:
  ``Invalid URL: Cannot parse `dune` to a `u32` ``. This is the `Path` extractor's rejection. The
  route **did** match (`{id}` accepts any segment), so it's not a `404`; the value was the problem,
  and `400` is HTTP's code for "your request is wrong, don't send it again unchanged". Read the
  message as: "the address is invalid: the text `dune` can't become a `u32`". `show_book` never
  ran.

Now the query string. The addresses are in double quotes because `?` and `&` mean something to
your shell: without the quotes, `&` would end the command, and curl would never see `page=2`.

```bash
{{#include ../../code/topics/path-and-query-extractors/http/02-query.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/02-query.out}}
```

- **`?q=rust`:** `searching for "rust", page 1`. `q` was filled in; `page` was missing, so it was
  `None`, and `unwrap_or(1)` gave `1`.
- **`?q=rust&page=2`:** `page 2`. Now `page` was `Some(2)`.
- **`/search`, no query string:** `400 Bad Request`,
  ``Failed to deserialize query string: missing field `q` ``. This is the `Query` extractor's
  rejection: `q` isn't an `Option`, so it's required, and it wasn't there. "Failed to deserialize"
  means "couldn't build your `Search` struct from it".
- **`?q=hello%20world`:** `searching for "hello world", page 1`, with a real space. An address
  can't contain a space, so the client writes it as `%20`: a `%` followed by the character's code in
  hexadecimal (`20` is the code for a space). This is called **percent-encoding**, and browsers do
  it for you when you type a space into a search box. `Query` decodes it before your handler sees
  the text, and the `{:?}` quotes show the space is really inside the value.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/books/42` says `book number 42` and `/books/dune` says `400 Bad Request`, the `Path`
  extractor is checking your numbers.
- ✅ If `/search?q=rust&page=2` says `page 2` and `/search?q=rust` says `page 1`, your optional
  option works.
- ❌ If your shell answers `zsh: no matches found`, or curl answers `page 1` for
  `?q=rust&page=2`, the address wasn't in double quotes: zsh reads the `?` as a file-name pattern,
  and bash cuts the command at the `&`.

## You might be wondering…

**"Why wrap it in `Path(...)` and then unwrap it?"**
Because the two halves do different jobs. The type, `Path<u32>`, is the **label** that tells Axum
where the value comes from. A bare `u32` doesn't say: the path, the query string and the body could
all hold a number, so Axum refuses it (the `Handler<_, _>` error from Step 4). The pattern,
`Path(id)`, is how **you** take the value out again, so the rest of the function works with a plain
number. Label on the way in, unwrap on the way out: you write both in one parameter.

**"Does the name in `{id}` have to match `id`?"**
No. For a single value or a tuple, `Path` reads by **position**, not by name. With the route
`"/books/{number}"`, `Path(id): Path<u32>` still gets the number (tried in a scratch copy). Pick
names that make the route easy to read. (When you read the path into a struct with named fields,
which the axum docs show, the names do have to match.)

**"What if the number is negative?"**
A `u32` can't be negative, so `/books/-1` is rejected with `400`, before your handler runs: the
`-` isn't something a `u32` can hold. *More examples* shows the real answers, for both the path and
the query string. That's usually what you want for ids and page numbers. If negatives are
meaningful, such as a temperature, ask for an `i32` instead, a whole number that can be negative.

**"Why `400` and not `404`?"**
`404` means "nothing lives at this path", and a route **did** match `/books/dune`. The problem was
inside the request: a value that doesn't fit. That's what `400 Bad Request` means: "you sent
something I can't use". Axum picks the right one for you.

**"Path or query — which should I use?"**
Ask: does this value say **which thing**, or **how you'd like it**?

- **Which thing:** the path. `/books/42` is book 42; without the `42`, it's a different thing (the
  whole collection). Ids and slugs go here.
- **How you'd like it:** the query string. Search words, the page number, the sort order, a filter:
  options that change the answer, not the thing. They're usually optional and can come in any
  order.

A good test: if the address makes sense with the value removed (`/search` still means "search"),
it's a query option.

**"Can I have both in one handler?"**
Yes: give the handler two parameters, one `Path` and one `Query`. Each clerk reads its own part of
the address. *More examples* has one, `/books/7/reviews?page=2`.

**"What happens to query options I didn't ask for?"**
They're ignored. `/search?q=rust&color=red` works exactly like `/search?q=rust`: the `Search`
struct has no `color` field, so no one reads it. That's friendly, but it also means a misspelt
option is silently ignored. *Common mistakes* shows where that bites.

## Coming from another language?

Every framework reads the path and the query string. The difference is who does the checking: you,
or the framework, based on the type you ask for.

**Express (Node.js).** Placeholders start with `:`, and everything arrives as text, unchecked:

```js
app.get("/books/:id", (req, res) => {
  const id = Number(req.params.id); // "dune" becomes NaN, and nothing stops it
  res.send(`book number ${id}`);
});
app.get("/search", (req, res) => {
  const page = req.query.page ?? "1"; // undefined when missing
  res.send(`searching for ${req.query.q}, page ${page}`);
});
```

In Axum, the type does that `Number(…)` and the `NaN` check for you, and answers `400`.

**Flask and FastAPI (Python).** FastAPI is the closest to Axum: the parameter's type annotation is
the check, and a parameter that isn't in the path is read from the query string:

```python
@app.get("/books/{id}")
def show_book(id: int):
    return f"book number {id}"

@app.get("/search")
def search(q: str, page: int = 1):
    return f"searching for {q}, page {page}"
```

FastAPI answers a bad value with `422 Unprocessable Entity` and a JSON description; Axum answers
`400` with plain text. In Flask, `@app.get("/books/<int:id>")` gives `404` for `/books/dune` (the
route doesn't match), and `request.args.get("page", 1, type=int)` quietly falls back to `1` when
`page` isn't a number.

**Spring Boot (Java).** Annotations mark where each parameter comes from, like Axum's `Path` and
`Query` types:

```java
@GetMapping("/books/{id}")
public String showBook(@PathVariable int id) {
    return "book number " + id;
}

@GetMapping("/search")
public String search(@RequestParam String q, @RequestParam(defaultValue = "1") int page) {
    return "searching for " + q + ", page " + page;
}
```

Spring answers `400` for `/books/dune` and for a missing `q`, as Axum does.
`@RequestParam(required = false) Integer page` is the equivalent of `Option<u32>`.

**Go (`net/http` and Gin).** Since Go 1.22, `net/http` routes have `{id}` placeholders like Axum's,
but values come out as strings, and the checking is yours:

```go
mux.HandleFunc("GET /books/{id}", func(w http.ResponseWriter, r *http.Request) {
    id, err := strconv.Atoi(r.PathValue("id"))
    if err != nil {
        http.Error(w, "id must be a number", http.StatusBadRequest)
        return
    }
    fmt.Fprintf(w, "book number %d", id)
})
```

`r.URL.Query().Get("q")` reads the query string, and gives `""` when `q` is missing. Gin has
`c.Param("id")`, `c.Query("q")` and `c.DefaultQuery("page", "1")`, all strings. Its
`c.ShouldBindQuery(&search)` fills a struct, much like Axum's `Query<Search>`.

## Common mistakes

**Asking for two values from a route with one placeholder.**
You copy `show_chapter` but give it a route with only `{id}`. It compiles, and it runs
(`cargo run -p path-and-query-extractors --example extractors-wrong-count`):

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-wrong-count.rs:mistake}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/70-wrong-count.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/70-wrong-count.out}}
```

`500 Internal Server Error`, not `400`. Axum's message is exact: the handler's `Path` expected 2
values, and the route gave it 1. It's a `500`, "the server is broken", because it **is** the
server's mistake, not the client's: no address could ever satisfy this route and this handler
together. The compiler can't catch it, because it doesn't read the text of your route. **Fix:**
make the two agree: one placeholder with a `Path<u32>`, or two placeholders
(`"/books/{book}/chapters/{chapter}"`) with a `Path<(u32, u32)>`. If you ever see this message,
compare the route with the handler it calls.

**Forgetting `#[derive(Deserialize)]`.**
The struct looks complete without it:

```rust,noplayground,ignore
struct Search {
    q: String,
    page: Option<u32>,
}
```

This error was captured in a scratch copy of this lesson's code, a project called
`my-extractors`, so its line numbers are from that file:

```text
error[E0277]: the trait bound `fn(Query<Search>) -> impl Future<Output = String> {search}: Handler<_, _>` is not satisfied
   --> src/main.rs:37:31
    |
 37 |         .route("/search", get(search))
    |                           --- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Query<Search>) -> impl Future<Output = String> {search}`
    |                           |
    |                           required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
…
```

The same `Handler<_, _>` error as in the last lesson, pointing at the route, with no mention of
serde. (Above it, the compiler also warns ``unused import: `serde::Deserialize` ``: a clue, since
nothing uses the name any more.) Add `#[axum::debug_handler]` above `async fn search`, as in
Handlers and IntoResponse (it needs Axum's `macros` feature), and the first error names the real
problem:

```text
error[E0277]: the trait bound `Search: serde::Deserialize<'de>` is not satisfied
  --> src/main.rs:27:32
   |
27 | async fn search(Query(params): Query<Search>) -> String {
   |                                ^^^^^ unsatisfied trait bound
   |
help: the trait `for<'de> Deserialize<'de>` is not implemented for `Search`
  --> src/main.rs:21:1
   |
21 | struct Search {
   | ^^^^^^^^^^^^^
   = note: for local types consider adding `#[derive(serde::Deserialize)]` to your `Search` type
…
```

"`Search` doesn't have the `Deserialize` ability", and the `note:` line even tells you the fix.
(`'de` is a detail of how serde works; you can read past it.) **Fix:** put `#[derive(Deserialize)]`
back on the line above `struct Search`.

**A query option with the wrong name.**
The struct's field is `q`, but the address says `query`:

```bash
{{#include ../../code/topics/path-and-query-extractors/http/71-wrong-name.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/71-wrong-name.out}}
```

The same answer as for `/search` with nothing at all. `query=rust` is an option the struct doesn't
have, so it's ignored, and `q` is missing. The field names in your struct **are** the names of the
options: the struct is the list of what clients may send. **Fix:** send `?q=rust`, or, if you want
the option to be called `query`, rename the field to `query` (and use `params.query`).

## More examples

Each example is a complete program in `code/topics/path-and-query-extractors/examples/`. Stop any
running server with `Ctrl+C`, start the example with the command shown, and send its requests from
your second terminal.

### `Path` and `Query` in one handler

The reviews of one book, a page at a time: *which book* comes from the path, *which page* from the
query string (`cargo run -p path-and-query-extractors --example extractors-both`):

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-both.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-both.rs:reviews}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-both.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/50-both.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/50-both.out}}
```

Two parameters, two clerks. Each reads its own part of the address, and both must succeed before
`reviews` runs. `ReviewOptions` has only an optional field, so the query string may be left out
entirely. (For these two extractors the order of the parameters doesn't matter.
[JSON with serde](json-and-serde.md) meets one that has to come last.)

### `HashMap<String, String>`: any options at all

When you don't know the option names in advance, ask for a `HashMap`, Rust's lookup table from
names to values (`cargo run -p path-and-query-extractors --example extractors-query-map`):

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-query-map.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-query-map.rs:filters}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-query-map.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/51-query-map.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/51-query-map.out}}
```

Every option lands in the map, whatever its name: `params.len()` counts them. `params.get("color")`
looks one up and gives an `Option`: `Some("red")` when it's there, `None` when it's not. No request
is ever rejected, because nothing is required. The price: every value is text, and nothing is
checked for you. A struct is the better choice whenever you know the names.

### `Path<String>`: a slug

Not every path value is a number. A **slug** is a readable, URL-safe name, such as
`my-first-post` (`cargo run -p path-and-query-extractors --example extractors-slug`):

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-slug.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-slug.rs:article}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-slug.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/52-slug.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/52-slug.out}}
```

A `String` accepts any segment, so there's nothing to reject: even `42` is fine, as the text
`"42"` (the `{:?}` quotes show it's text, not a number). Choose the type that says what you really
expect, and let the extractor do the checking.

### Negative numbers

This one needs no new code: it's the lesson's own server
(`cargo run -p path-and-query-extractors`), sent a minus sign in the path and in the query string:

```bash
{{#include ../../code/topics/path-and-query-extractors/http/53-negative.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/53-negative.out}}
```

Both are `400 Bad Request`, from two different clerks, so they're worded differently. The `Path`
rejection says `-1` can't become a `u32`. The `Query` rejection names the field first, `page:`, then
the reason: `invalid digit found in string`, because `-` isn't a digit, and a `u32` is only digits.
Either way, your handler never saw a negative number.

## Your turn

Each solution is also a complete program in `code/topics/path-and-query-extractors/examples/`,
with a test inside.

### 🟢 Guided

Add a route `/authors/{name}` that answers with a greeting: `GET /authors/ada` answers
`hello, ada!`. A name is text, so copy `show_book` and change the type inside `Path<…>`. The slug
example above is a close cousin.

<details><summary>Solution</summary>

`cargo run -p path-and-query-extractors --example extractors-authors`:

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-authors.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-authors.rs:greet_author}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-authors.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/90-authors.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/90-authors.out}}
```

`Path<String>` instead of `Path<u32>`, and `{name}` in both the route and the `format!`. The route's
`{name}` and the variable `name` don't have to match, but when they do, the code is easier to read.
Here `format!` uses `{name}`, not `{name:?}`, so there are no quotes in the answer.

</details>

### 🟡 Tweak

Add a `sort` option to `Search`: `/search?q=rust&sort=year` should say `sorted by year`, and when
there's no `sort`, it should say `sorted by title`. You need one new field and one new
`unwrap_or`. Hint: `unwrap_or` must give back the same type the `Option` holds, so the default
has to be a `String` too: `"title".to_string()`.

<details><summary>Solution</summary>

`cargo run -p path-and-query-extractors --example extractors-sort`:

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-sort.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-sort.rs:search}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-sort.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/91-sort.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/91-sort.out}}
```

`sort: Option<String>` makes the option optional, and `unwrap_or("title".to_string())` fills the
gap. `"title"` on its own is a `&'static str`, while the `Option` holds a `String`, which is why
`.to_string()` is needed. The `format!` got long enough that `cargo fmt` split it over several
lines. If you change `src/main.rs` itself, the test `query_string_is_parsed_with_an_optional_field`
still expects the old answer, so update it too.

</details>

### 🔴 From scratch

Write a route `/range?from=1&to=5` that answers the numbers from `from` to `to`, joined by commas:
`1,2,3,4,5`. If `from` is bigger than `to`, answer `400 Bad Request` with a message saying so.
Return a `(StatusCode, String)` pair from the handler, as in Handlers and IntoResponse, so you can
choose the status. Hints: `from..=to` counts from `from` to `to`, including both, in a `for` loop;
`n.to_string()` turns a number into text; and a `Vec<String>` has `.join(",")`.

<details><summary>Solution</summary>

`cargo run -p path-and-query-extractors --example extractors-range`:

```rust,noplayground
{{#include ../../code/topics/path-and-query-extractors/examples/extractors-range.rs:imports}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-range.rs:range}}

{{#include ../../code/topics/path-and-query-extractors/examples/extractors-range.rs:app}}
```

```bash
{{#include ../../code/topics/path-and-query-extractors/http/92-range.sh}}
```

```text
{{#include ../../code/topics/path-and-query-extractors/http/92-range.out}}
```

Three answers, and two kinds of `400`:

- `from=1&to=5`: `200 OK`, `1,2,3,4,5`. The loop pushed each number, as text, into `numbers`, and
  `.join(",")` glued them together with commas.
- `from=5&to=1`: **your** `400`. Both values were fine numbers, so the `Query` extractor let them
  through, and your `if` caught the problem that only your code can know about. `return` sends
  that answer at once and skips the rest of the handler.
- `from=1`, no `to`: **Axum's** `400`, ``missing field `to` ``, before your handler even ran.

Extractors check the **shape** of the request (is it there, is it a number); your handler checks
what the values **mean** together. One more thing a real API would add: a limit on how many numbers
it builds, so that `from=0&to=4000000000` can't keep the server busy.

</details>

## Quick check

<div class="quiz" data-topic="path-and-query-extractors"></div>

## Remember this

- An **extractor** is a handler parameter Axum fills from the request before your handler runs.
  The type asks for it (`Path<u32>`); the pattern unpacks it (`Path(id)`).
- `Path<u32>` reads one placeholder, `Path<(u32, u32)>` reads two, left to right. The number of
  placeholders and values must match, or every request gets a `500`.
- `Query<Search>` fills a `#[derive(Deserialize)]` struct from the query string. The field names
  are the option names; `Option<T>` fields are optional, and `unwrap_or` gives them a default.
- When the address doesn't fit the type, Axum answers `400 Bad Request` with a plain-text reason,
  and your handler never runs.
- Path for *which thing*, query string for *how you'd like it*.

## Go deeper

- [Rust for Humans: Serde and JSON](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/serde-and-json.html)
- [Rust for Humans: Pattern matching](https://open-source-bd.github.io/rustbook-for-human/language-basics/pattern-matching.html)
- [axum::extract](https://docs.rs/axum/0.8.9/axum/extract/index.html) — Official reference: every built-in extractor, and how rejections work.

<!-- next:start -->

**Next:**

- [JSON with serde](../a2-axum/json-and-serde.md)

<!-- next:end -->
