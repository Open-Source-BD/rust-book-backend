# Error handling in Axum

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can define one error type for your whole app.
- You can turn each error into the right status code and a JSON body.
- You can use `?` in handlers so errors flow out automatically.

## What & why

One `AppError` enum, `IntoResponse` for it, and `?` in handlers: clean, consistent JSON errors instead of unwraps.

Picture a restaurant. You ask for the fish, and the kitchen has run out. A good waiter doesn't run
out of the kitchen shouting "KITCHEN EXCEPTION, LINE 42!", and doesn't walk off without a word
either. They come back and say "Sorry, we're out of fish." And if the oven has caught fire, they
say "Sorry, the kitchen is closed", without describing the fire. Every waiter in the restaurant
answers the same polite way, because the restaurant taught them one way to say bad news.

A backend has the same job. A client asks for book 9, and there is no book 9. So far, our
[handlers](../glossary.md#handler) have dealt with trouble in three different ways:

- [Shared state](shared-state.md) called `.unwrap()` on the lock, which crashes the request if
  anything goes wrong: the waiter walking off without a word.
- [Handlers and IntoResponse](handlers-and-into-response.md#resultstring-statuscode-an-answer-or-an-error-code)
  answered `Err(StatusCode::NOT_FOUND)`: a correct [status code](../glossary.md#status-code), but an
  empty body, so the client doesn't learn what was missing.
- [JSON with serde](json-and-serde.md) let Axum refuse bad bodies with plain text, while our good
  answers were [JSON](../glossary.md#json).

Three handlers, three styles. A client that talks to your API has to guess which one it will get.
This lesson gives the whole app **one** way to say bad news: an
[**error type**](../glossary.md#error-type), a type whose values describe what went wrong. Every
handler returns the same error type, and one piece of code decides how each kind of error sounds:
its status code, and a JSON body such as `{"error":"book 9 not found"}`.

```text
                     Ok(Json(book))
 GET /books/2 ──▶ show_book ───────────────────────────────────▶ 200 {"id":2,"title":"Emma"}

                     Err(AppError::NotFound(9))
 GET /books/9 ──▶ show_book ─────────────▶ into_response ──────▶ 404 {"error":"book 9 not found"}
```

This lesson has three ideas:

1. **One error type for the app.** An
   [`enum`](https://open-source-bd.github.io/rustbook-for-human/language-basics/enums.html)
   called `AppError`, with one *variant*, one choice on its list, for each kind of failure. A crate
   called `thiserror` writes each variant's message for you.
2. **`impl IntoResponse for AppError`.** You teach Axum how to send an `AppError`, in one place:
   which status code each variant gets, and what the JSON body says.
3. **`Result<_, AppError>` and `?` in handlers.** A handler returns `Ok(answer)` or
   `Err(an AppError)`, and the
   [`?` operator](https://open-source-bd.github.io/rustbook-for-human/abstractions/the-question-mark-operator.html)
   sends errors out of the handler the moment they happen.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/error-handling-in-axum`. Its `Cargo.toml` has two new lines,
`serde_json` and `thiserror`:

```toml
{{#include ../../code/topics/error-handling-in-axum/Cargo.toml}}
```

### Line by line

`[package]` · `name = "error-handling-in-axum"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p error-handling-in-axum`.
- **How:** each lesson is its own small project, named after the lesson.
- **Remove it and…** (`name`) Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "take the edition and publish settings from the workspace's `code/Cargo.toml`".
- **Why:** every lesson shares `edition = "2024"` and `publish = false`.
- **How:** [Hello, Axum, Step 2](hello-axum.md#step-2-the-books-copy-of-the-same-project) explains
  `.workspace = true` in full.
- **Remove it and…** (`edition`) Cargo falls back to edition 2015, where `async fn` isn't allowed.

`[dependencies]` · `axum.workspace = true` · `tokio.workspace = true` · `serde.workspace = true`
- **What:** Axum 0.8.9, Tokio 1.53.1, and serde with its `derive` feature, as in
  [Shared state](shared-state.md).
- **Why:** Axum and Tokio run the server; serde lets `Book` go out as JSON.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`serde_json.workspace = true`
- **What:** serde_json 1.0.151, the crate that reads and writes JSON text.
- **Why:** Step 5 builds the error's JSON body with its `json!` macro.
- **How:** [JSON with serde](json-and-serde.md) needed no `serde_json` line, because Axum brings
  that crate in for its own use. But a crate you **name in your own code**, as in
  `use serde_json::json;`, must be in your own `Cargo.toml`, even when another crate already uses
  it.
- **Remove it and…** ``error[E0432]: unresolved import `serde_json` ``, pointing at the
  `use serde_json::json;` line.

`thiserror.workspace = true`
- **What:** thiserror 2.0.21, a small crate that writes error-type code for you.
- **Why:** Step 4 uses its `#[derive(thiserror::Error)]` and `#[error("…")]` to give each error its
  message.
- **How:** it only runs while your program compiles: it writes Rust code, then gets out of the
  way.
- **Remove it and…** four errors. The first is ``error[E0433]: cannot find module or crate
  `thiserror` in this scope``, pointing at `#[derive(Debug, thiserror::Error)]`; the others follow
  from it.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the answers, without opening a
  port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, then `cargo add serde_json thiserror`, and the test crates
from `cargo add --dev tower http-body-util --features tower/util`.

### Step 2: the `use` lines

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:imports}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, routing::get};`
- **What:** brings eight names from Axum into this file.
- **Why:** `Json`, `Router`, `Path`, `State`, `StatusCode` and `get` are old friends. `IntoResponse`
  and `Response` are for Step 5, where we teach Axum how to send an error.
- **How:** `IntoResponse` is the trait from
  [Handlers and IntoResponse](handlers-and-into-response.md), and `Response` is the finished HTTP
  response type that its `into_response()` method returns.
- **Remove it and…** (`response::{IntoResponse, Response}`) ``cannot find trait `IntoResponse` in
  this scope``, and the same for `Response`, at Step 5's `impl` line.

`use serde::Serialize;`
- **What:** serde's ability to turn a Rust value into JSON.
- **Why:** `Book` goes out as JSON. Nothing comes in as JSON in this lesson's main program, so
  there's no `Deserialize`.
- **How:** as in JSON with serde.
- **Remove it and…** ``cannot find derive macro `Serialize` in this scope``.

`use serde_json::json;`
- **What:** brings in the `json!` macro.
- **Why:** Step 5 uses it to write the error's JSON body right in the code.
- **How:** a *macro* is code that writes code while the program compiles; you call it with a `!`,
  like `println!`.
- **Remove it and…** ``error: cannot find macro `json` in this scope``, at Step 5's `json!(…)`.

`use std::sync::{Arc, Mutex};`
- **What:** the shared-list tools from Shared state.
- **Why:** the books live in the app's [state](../glossary.md#state), wrapped in
  `Arc<Mutex<…>>`, as before.
- **How:** both come with Rust's standard library.
- **Remove it and…** ``cannot find type `Arc` in this scope``, and the same for `Mutex`.

### Step 3: the book and the state

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:types}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`#[derive(Clone, Serialize)]` · `struct Book {` · `id: u32,` · `title: String,` · `}`
- **What:** the same `Book` as in Shared state: an id and a title, sent as JSON, and copyable.
- **Why:** the handler in Step 6 hands a **copy** of a book to the response, so the original stays
  in the list.
- **How:** `Clone` writes a `.clone()` method; `Serialize` lets `Json(book)` turn it into JSON.
- **Remove it and…** (`Clone`) ``the trait bound `Book: Clone` is not satisfied``, at the
  `.cloned()` in Step 6.

`#[derive(Clone)]` · `struct AppState {` · `books: Arc<Mutex<Vec<Book>>>,` · `}`
- **What:** the app's state: one shared, locked list of books.
- **Why:** it's where the handler looks for the book it was asked for.
- **How:** exactly as in [Shared state, Step 4](shared-state.md#step-4-the-state), without
  `Default`: this time the app starts with two books, so Step 8 builds the state by hand.
- **Remove it and…** (`Clone`) five errors: the familiar ``Handler<_, _>`` one at `get(show_book)`,
  and four of ``the trait bound `AppState: Clone` is not satisfied``.

### Step 4: the error type

Here's the new idea: one `enum` that lists everything that can go wrong in this app.

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:error}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`enum AppError {` · `}`
- **What:** our error type. An `enum` is a type whose value is **one of** a fixed list of
  choices, called *variants*; Rust for Humans'
  [Enums](https://open-source-bd.github.io/rustbook-for-human/language-basics/enums.html) lesson
  teaches them.
- **Why:** each variant is one kind of failure. A value of type `AppError` is always exactly one of
  them, so code that handles an `AppError` knows every case it might meet.
- **How:** `AppError` is only a name we chose, a common one in Axum apps. Axum doesn't know it
  exists until Step 5 introduces them.
- **Remove it and…** nothing in the handler can say what went wrong.

`#[error("book {0} not found")]` · `NotFound(u32),`
- **What:** the variant for "there's no book with that id". It carries one value, the id that was
  asked for, and its message is `book 9 not found` when that id is 9.
- **Why:** the message will be the `"error"` text the client reads, so it names the missing id.
- **How:** `{0}` means "the variant's first value": the `u32`. `AppError::NotFound(9).to_string()`
  gives the text `book 9 not found`. The `#[error("…")]` line is an instruction to thiserror, not
  to Rust itself.
- **Remove it and…** (the `#[error]` line) thiserror stops the build with ``error: missing
  #[error("...")] display attribute``, pointing at `NotFound(u32)`: every variant needs a message.
  (Remove only `{0}`, and it compiles, but every missing book says `book not found`, and the
  client can't see which id was wrong.)

`#[error("the book list is unavailable after an earlier crash")]` · `Poisoned,`
- **What:** the variant for "the lock around the book list is broken". It carries no value, so its
  message is fixed.
- **Why:** it's the one failure that isn't the client's fault. Step 6 explains when it happens.
- **How:** a variant with no value is written with no brackets, like `Poisoned`.
- **Remove it and…** two errors, one at Step 5's `match` and one at Step 6's `.map_err`: ``no
  variant, associated function, or constant named `Poisoned` found for enum `AppError` ``.

`#[derive(Debug, thiserror::Error)]`
- **What:** asks for two abilities to be written for `AppError`: `Debug`, a way to print the value
  for programmers, and thiserror's `Error`.
- **Why:** Rust has a standard trait for errors, `std::error::Error`, and it has two requirements:
  the type must be printable for programmers (`Debug`) and for people (`Display`, the trait behind
  `.to_string()`). thiserror's derive writes `Display` from your `#[error("…")]` messages, and then
  the `Error` trait itself. *More examples* shows the same enum with that code written by hand.
- **How:** the path `thiserror::Error` names the derive without a `use` line. Rust for Humans'
  [Error crates: thiserror and anyhow](https://open-source-bd.github.io/rustbook-for-human/abstractions/error-crates-thiserror-and-anyhow.html)
  lesson covers the crate in depth.
- **Remove it and…** (`Debug`) ``error[E0277]: `AppError` doesn't implement `Debug` ``, with the
  hint ``add `#[derive(Debug)]` to `AppError` ``. (`thiserror::Error`) three errors: Rust no longer
  knows `#[error]` (``cannot find attribute `error` in this scope``, once per variant), and Step 5's
  `self.to_string()` fails with ``error[E0599]: `AppError` doesn't implement
  `std::fmt::Display` ``.

### Step 5: teach Axum how to send an `AppError`

An error type on its own is only Rust. To send it over HTTP, Axum needs to know which status code
and body each variant becomes. That's the job of the
[trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
`IntoResponse`, which you've met on Axum's own types. Now you write it for yours:

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:into_response}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`impl IntoResponse for AppError {` · `}`
- **What:** gives `AppError` the `IntoResponse` ability: "I know how to become an HTTP response".
- **Why:** Axum can only send types that have this ability. With it, a handler may answer with an
  `AppError`, and Axum knows what to put on the wire.
- **How:** `impl TraitName for TypeName { … }` is how you give a type a trait's ability, by writing
  the trait's methods for it. `IntoResponse` has one method to write, `into_response`.
- **Remove it and…** the handler no longer compiles. *Common mistakes* shows the error.

`fn into_response(self) -> Response {`
- **What:** the method Axum calls when an `AppError` has to be sent.
- **Why:** this is the **one place** where every error's status and body are decided.
- **How:** `self` is the error being sent. The method takes it by value (without `&`), because a
  response is built only once, and the error isn't needed afterwards. `Response` is the finished
  HTTP response.
- **Remove it and…** ``not all trait items implemented, missing: `into_response` ``: a trait's
  methods aren't optional.

`let status = match self {` · `AppError::NotFound(_) => StatusCode::NOT_FOUND,` · `AppError::Poisoned => StatusCode::INTERNAL_SERVER_ERROR,` · `};`
- **What:** picks the status code for this error: `404 Not Found` for a missing book, `500 Internal
  Server Error` for a broken lock.
- **Why:** the status code is the part of the answer that programs read. `404` says "your request
  was fine, but that thing doesn't exist"; `500` says "the fault is on our side".
- **How:** `match` compares `self` with each pattern in turn and gives the value after the first
  `=>` that fits. `NotFound(_)` fits any `NotFound`, whatever its id: `_` means "any value, and I
  don't need it". Because nothing is taken **out** of `self`, `self` is still whole after the
  `match`, ready for the next line.
- **Remove it and…** (the `NotFound` arm) ``error[E0004]: non-exhaustive patterns:
  `AppError::NotFound(_)` not covered``. `match` must handle every variant, so when you add a
  variant to `AppError` one day, the compiler shows you every `match` that has to decide about it.

`let body = Json(json!({ "error": self.to_string() }));`
- **What:** builds the JSON body `{"error":"book 9 not found"}`.
- **Why:** a client, and the person debugging it, learns **what** went wrong, not only the code.
- **How:** read it from the inside out. `self.to_string()` is the message from Step 4's
  `#[error("…")]`. `json!({ … })` builds a JSON value from something that looks like JSON: a key,
  `"error"`, and a value, our message. `Json(…)` wraps it so that Axum sends it as JSON, with
  `content-type: application/json`.
- **Remove it and…** (the `use serde_json::json;` line) ``error: cannot find macro `json` in this
  scope``.

`(status, body).into_response()`
- **What:** puts the status and the body together, and turns them into a `Response`.
- **Why:** the method promised to return a `Response`, not a tuple.
- **How:** a `(StatusCode, body)` tuple already implements `IntoResponse`, as you saw in
  [Handlers and IntoResponse, Step 6](handlers-and-into-response.md#step-6-the-shape-of-a-response-tuple).
  So we let the tuple do the work, and call its `.into_response()`.
- **Remove it and…** (`.into_response()`) ``error[E0308]: mismatched types``: ``expected
  `Response<Body>`, found `(StatusCode, Json<Value>)` ``. (`Value` is serde_json's type for "any
  JSON value", which is what `json!` makes.)

### Step 6: the handler

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:handler}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`async fn show_book(` · `State(state): State<AppState>,` · `Path(id): Path<u32>,`
- **What:** a handler that takes the state and the book id from the path, such as `2` in
  `/books/2`.
- **Why:** it answers `GET /books/{id}`: "show me this one book".
- **How:** two [extractors](../glossary.md#extractor): `State` from
  [Shared state](shared-state.md), and `Path` from
  [Path and Query extractors](path-and-query-extractors.md).
- **Remove it and…** (`Path(id): Path<u32>,`) ``cannot find value `id` in this scope``, where the
  handler uses `id`.

`) -> Result<Json<Book>, AppError> {`
- **What:** the handler returns either `Ok(Json<Book>)`, a book, or `Err(AppError)`, an error.
- **Why:** this line says, in the handler's signature, that it can fail and **how**: always with
  an `AppError`.
- **How:** [`Result`](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
  is Rust's type for "a success or an error". Axum can send a `Result` whenever both of its sides
  implement `IntoResponse`: for `Ok`, it sends the `Json<Book>` as `200 OK`; for `Err`, it calls
  **your** `into_response` from Step 5. You never call it yourself.
- **Remove it and…** (write `-> Json<Book>`) both `?`s fail with ``the `?` operator can only be used
  in an async function that returns `Result` or `Option` ``: `?` sends an error **out**, so the
  function must be able to return one.

`let books = state.books.lock().map_err(|_| AppError::Poisoned)?;`
- **What:** locks the list of books, or stops the handler with `AppError::Poisoned` if the lock is
  broken.
- **Why:** in Shared state, this line ended in `.unwrap()`. Here's what that `Result` is about.
  A `Mutex` becomes **poisoned** when some code panics (crashes) **while it holds the lock**. Rust
  can't know whether that code had finished changing the list, or left it half-changed, so from
  then on it won't hand the list over quietly: every `.lock()` returns an `Err` instead. And a
  poisoned lock **stays poisoned**. It isn't one bad request: **every** later `.lock()`, from every
  request, returns `Err`, until the server process restarts and builds a fresh `Mutex`. (The
  standard library has a `clear_poison` method for code that can check and repair the data; our
  handlers don't use it.) So `.unwrap()` here would turn one earlier crash into a crash on every
  request that needs the books, for as long as the server runs: *Common mistakes* shows exactly
  that. With `.map_err(…)?`, each of those requests gets a clear `500` JSON error instead, and the
  server keeps answering; *More examples* shows that too.
- **How:** `.lock()` returns `Result<guard, PoisonError>`. `.map_err(…)` leaves an `Ok` alone and
  changes what's inside an `Err`: here, the `PoisonError` becomes `AppError::Poisoned`.
  `|_| AppError::Poisoned` is a
  [closure](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html), a small
  function written in place: "given the error, which I ignore (`_`), make `AppError::Poisoned`".
  Then `?` does the rest: on `Ok`, it takes the guard out, and `books` is the locked list; on
  `Err`, it **returns** the error from `show_book` right there, and the lines below never run.
- **Remove it and…** (`.map_err(…)`, writing `state.books.lock()?`) ``error[E0277]: `?` couldn't
  convert the error to `AppError` ``, with the note: ``the question mark operation (`?`)
  implicitly performs a conversion on the error value using the `From` trait``. `?` doesn't know
  how to turn a `PoisonError` into an `AppError`, and `.map_err` is how you tell it. *More
  examples* uses that `From` conversion on purpose. (Remove only the `?`, and `books` stays a
  `Result` instead of the list, and the build fails in the lines below, with three errors.)

`let book = books` · `.iter()` · `.find(|book| book.id == id)` · `.cloned()` · `.ok_or(AppError::NotFound(id))?;`
- **What:** finds the book with this id and copies it out of the list, or stops the handler with
  `AppError::NotFound(id)`.
- **Why:** a missing book is **expected**: clients ask for ids that don't exist all the time. It
  deserves a clear `404`, not a crash.
- **How:** it's a chain, one step per line, each working on what the one above gave it. Step 7
  takes it one step at a time.
- **Remove it and…** see Step 7: each step has its own error.

`Ok(Json(book))`
- **What:** the success answer: the book, as JSON, wrapped in `Ok`.
- **Why:** the function returns a `Result`, so the success has to be marked as one.
- **How:** Axum sends it as `200 OK` with `content-type: application/json`.
- **Remove it and…** (write `Json(book)`) ``error[E0308]: mismatched types``: ``expected
  `Result<Json<Book>, AppError>`, found `Json<Book>` ``, and the compiler even suggests the fix:
  ``help: try wrapping the expression in `Ok` ``.

### Step 7: finding a book, one step at a time

The chain from Step 6, with the type each line gives to the next:

```text
books                            the locked list         MutexGuard<Vec<Book>>
  .iter()                        each book, lent         &Book, &Book, …
  .find(|book| book.id == id)    the first match, if any Option<&Book>
  .cloned()                      a copy of it, if any    Option<Book>
  .ok_or(AppError::NotFound(id)) a book, or our error    Result<Book, AppError>
  ?                              the book (or return)    Book
```

### Line by line

`books`
- **What:** the guard from the line above: the locked list.
- **Why:** it's where the books are.
- **How:** it behaves like a `Vec<Book>`, and the lock stays closed while it exists.
- **Remove it and…** there's nothing to search.

`.iter()`
- **What:** walks the list, handing out each book in turn, as a **reference** (`&Book`): a way to
  look at a book that stays in the list.
- **Why:** we only want to look at the books, not take them out of the shared list.
- **How:** `.iter()` makes an *iterator*, a thing that gives values one at a time; `.find` asks it
  for values until one fits.
- **Remove it and…** ``error[E0599]: no method named `find` found for struct
  `std::sync::MutexGuard<'_, Vec<Book>>` ``: a list has no `.find` of its own; its iterator does.

`.find(|book| book.id == id)`
- **What:** the first book whose `id` equals the one asked for: `Some(&book)`, or `None` if there
  is none.
- **Why:** "no such book" is a normal answer, and `Option` is Rust's way to say "maybe nothing".
- **How:** `|book| book.id == id` is a closure that answers "is this the one?" for each book, and
  `.find` stops at the first `true`.
- **Remove it and…** there's no search: the next steps would work on the whole iterator.

`.cloned()`
- **What:** turns `Option<&Book>` into `Option<Book>`: a copy of the book, if there is one.
- **Why:** the reference points **into** the locked list, and the lock opens when the handler ends.
  The response needs a book of its own that lives on after that.
- **How:** it calls `.clone()` on the book inside the `Some`, which is why `Book` has `Clone`
  (Step 3). A `None` stays `None`.
- **Remove it and…** ``error[E0308]: mismatched types``: ``expected `Book`, found `&Book` `` at
  `Ok(Json(book))`, with the hint ``consider using clone here``.

`.ok_or(AppError::NotFound(id))`
- **What:** turns the `Option` into a `Result`: `Some(book)` becomes `Ok(book)`, and `None` becomes
  `Err(AppError::NotFound(id))`.
- **Why:** `?` in this handler works on `Result`s with an `AppError` inside, and "no such book" is
  now one of our errors, carrying the id the client asked for.
- **How:** you give `.ok_or` the error to use in case of `None`.
- **Remove it and…** (with its `?`) ``expected `Book`, found `Option<Book>` ``, and the compiler
  suggests ``Option::expect``, which would panic. That's the one suggestion not to take: see
  *Common mistakes*.

`?`
- **What:** on `Ok(book)`, takes the book out, so `book` is a plain `Book`. On
  `Err(AppError::NotFound(id))`, returns that error from `show_book` at once.
- **Why:** it's the whole idea of the lesson in one character: the error leaves the handler, and
  Axum turns it into a response with your `into_response`.
- **How:** it's the same `?` as on the lock line. Every step that can fail ends in `?`, and the
  code below it can assume it worked.
- **Remove it and…** ``expected `Book`, found `Result<Book, AppError>` ``, with the hint
  ``use the `?` operator to extract the `Result<Book, AppError>` value``.

### Step 8: the Router, with two books

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:app}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

### Line by line

`let books = vec![` · `Book { id: 1, title: "Dune".to_string() },` · `Book { id: 2, title: "Emma".to_string() },` · `];`
- **What:** the starting list: Dune with id 1, and Emma with id 2.
- **Why:** so there's a book to find (`/books/2`) and a book that isn't there (`/books/9`) as soon
  as the server starts.
- **How:** `vec![…]` builds a `Vec` from the items listed; `.to_string()` turns the fixed text into
  a `String`, the type the `title` field wants.
- **Remove it and…** every request answers `404`: there's nothing to find.

`Router::new()` · `.route("/books/{id}", get(show_book))`
- **What:** one route: `GET /books/{id}` runs `show_book`.
- **Why:** `{id}` is the part of the path that `Path(id)` reads.
- **How:** as in Path and Query extractors.
- **Remove it and…** (the `.route`) every request answers `404 Not Found` with an empty body:
  that's Axum's own "no such route", not our `AppError`.

`.with_state(AppState {` · `books: Arc::new(Mutex::new(books)),` · `})`
- **What:** wraps the starting list in a `Mutex`, then in an `Arc`, and hands the state to the
  Router.
- **Why:** Shared state used `AppState::default()`, an empty list; here we start with books, so we
  build the state by hand.
- **How:** read `Arc::new(Mutex::new(books))` from the inside out, as in
  [Shared state, Step 4½](shared-state.md#step-4½-arcmutexvecbook-one-layer-at-a-time).
- **Remove it and…** (`Arc::new(…)`, writing `books: Mutex::new(books),`) ``error[E0308]: mismatched
  types``, with the note ``expected struct `Arc<std::sync::Mutex<_>>` ``.

### Step 9: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/src/main.rs:main}}
```

📁 Full code: code/topics/error-handling-in-axum · ▶ Run it: `cd code && cargo run -p error-handling-in-axum`

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
- **What:** answers requests until you stop the server.
- **Why:** this is the server's life: wait for a request, run the handler, send the answer.
- **How:** for a handler that returns `Err`, this is where Axum calls `AppError::into_response`.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p error-handling-in-axum
```

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/error-handling-in-axum`
Listening on http://127.0.0.1:3000
```

(The first time, you'll also see `Compiling` lines above these.) In the second terminal, ask for a
book that exists, one that doesn't, and one whose id isn't a number:

```bash
{{#include ../../code/topics/error-handling-in-axum/http/01-found-and-missing.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/01-found-and-missing.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`/books/2`: `200 OK`.** The chain found Emma, `?` let her through, and `Ok(Json(book))` sent
  her as JSON.
- **`/books/9`: `404 Not Found`, in JSON.** `.find` gave `None`, `.ok_or` made it
  `Err(AppError::NotFound(9))`, and `?` returned it. Axum then called **your** `into_response`: the
  status came from the `match`, and the body from `json!` and the `#[error("book {0} not found")]`
  message. The header says `content-type: application/json`, exactly like a success, so the client
  can read errors the same way it reads books.
- **`/books/abc`: `400 Bad Request`, in plain text.** Look at the header:
  `content-type: text/plain; charset=utf-8`. This answer didn't come from `AppError` at all.
  `abc` can't become a `u32`, so the `Path` extractor **rejected** the request, as in Path and
  Query extractors, and `show_book` never ran. Axum's built-in rejections don't know about your
  error type, so they answer in plain text. That breaks our "one polite language" promise; *More
  examples* fixes it, so that rejections become JSON too.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/books/9` answers `404` with `{"error":"book 9 not found"}`, your error type, your
  `IntoResponse` and your `?` are all working together.
- ✅ If `/books/abc` answers `400` in plain text, you've seen the one kind of error that doesn't
  pass through `AppError` yet.
- ❌ If `/books/9` makes curl print `Empty reply from server`, look for an `.unwrap()` or
  `.expect(…)` where the `.ok_or(…)?` should be: *Common mistakes* shows this one.
- ❌ If the build stops with a ``Handler<_, _>`` error at `get(show_book)`, check that
  `impl IntoResponse for AppError` is there: *Common mistakes* has the full error.

## You might be wondering…

**"Why not return `(StatusCode, String)` everywhere?"**
You could: `Result<Json<Book>, (StatusCode, String)>` compiles, and `Err((StatusCode::NOT_FOUND,
"book 9 not found".to_string()))` sends a `404`. It works for one handler. With twenty, each one
picks its own status and writes its own sentence, so the same problem ends up with different codes
and different words, and changing the format of every error (from plain text to JSON, say) means
editing every handler. With `AppError`, a handler says **what** went wrong (`NotFound(id)`), and
`into_response` alone decides **how** it sounds. One change there changes every error in the app.

**"What is `thiserror` doing for me?"**
Writing boilerplate. Rust's `std::error::Error` trait needs your type to implement `Display`, the
text for people, and `thiserror` writes that `Display` from your `#[error("…")]` lines, then adds
`impl std::error::Error for AppError`. Nothing else: it adds no code that runs in your server
beyond what you'd write by hand, and it doesn't talk to Axum at all. *More examples* shows the
hand-written version, so you can see exactly what it saves. Rust for Humans'
[Custom error types](https://open-source-bd.github.io/rustbook-for-human/abstractions/custom-error-types.html)
lesson builds an error type by hand from the start.

**"Should the error message be shown to users?"**
For errors that are the **client's** doing, yes: "book 9 not found" or "title must not be empty"
help them fix their request. For `500` errors, be careful. Their real reason is about **your**
server, and it can reveal what a stranger shouldn't know: file paths, setting names, database
details. Send a short, polite message instead, and write the real reason to your server's log,
where only you can read it. *More examples* shows this with `eprintln!`, and
[Middleware and Tower layers](middleware-and-tower-layers.md) sets up proper logging.

**"What about anyhow?"**
[anyhow](https://open-source-bd.github.io/rustbook-for-human/abstractions/error-crates-thiserror-and-anyhow.html)
is thiserror's sibling crate. Its `anyhow::Error` holds **any** error, which is perfect when you only
need to report a failure, such as in a command-line tool's `main`. A handler needs more: it must
know **which** error it has, to choose between `404` and `500`. `anyhow::Error` hides the concrete
type, so getting it back needs a runtime check (`downcast_ref`); an enum lets the compiler check
every case. So Axum apps usually keep an enum like `AppError`, and some add one catch-all variant that
holds an `anyhow::Error` for "anything else", sent as a `500`. You don't need it in this book.

**"Who calls `into_response`? I never do."**
Axum does. Your handler returns a `Result`; Axum sees an `Err`, and calls `into_response` on the
`AppError` inside. That's why the handler can be so short: it only says what went wrong, and
returns.

## Coming from another language?

Every framework has to turn "something went wrong" into an HTTP answer. What differs is whether
errors are *thrown* past your code, or *returned* as values you can see.

**Express (Node.js).** You throw an error, and one error-handling middleware, recognised by its
**four** parameters, turns it into the response:

```js
class NotFound extends Error {
  status = 404;
}

app.get("/books/:id", (req, res) => {
  const book = books.find((b) => b.id === Number(req.params.id));
  if (!book) throw new NotFound(`book ${req.params.id} not found`);
  res.json(book);
});

app.use((err, req, res, next) => {
  res.status(err.status ?? 500).json({ error: err.message });
});
```

That last function is Express's `impl IntoResponse for AppError`: one place that decides the status
and the body. The difference is that `throw` is invisible in a function's signature: any function
might throw anything. In Rust, `Result<Json<Book>, AppError>` says it in the signature.

**Flask and FastAPI (Python).** FastAPI has a ready-made exception:

```python
from fastapi import FastAPI, HTTPException

app = FastAPI()

@app.get("/books/{book_id}")
def show_book(book_id: int):
    for book in books:
        if book["id"] == book_id:
            return book
    raise HTTPException(status_code=404, detail=f"book {book_id} not found")
```

The client gets `404` and `{"detail":"book 9 not found"}`, and `/books/abc` gets a JSON `422` from
FastAPI's own checks. For your own exception classes, `@app.exception_handler(MyError)` plays the
part of `into_response`. Flask does the same with `@app.errorhandler(MyError)`, and a function that
returns `{"error": str(e)}, 404`.

**Spring Boot (Java).** Throw an exception, and a `@RestControllerAdvice` class turns it into a
response for every controller:

```java
class BookNotFound extends RuntimeException {
    BookNotFound(long id) { super("book " + id + " not found"); }
}

@RestControllerAdvice
class ApiErrors {
    @ExceptionHandler(BookNotFound.class)
    ResponseEntity<Map<String, String>> notFound(BookNotFound e) {
        return ResponseEntity.status(HttpStatus.NOT_FOUND)
            .body(Map.of("error", e.getMessage()));
    }
}
```

The advice class is your `impl IntoResponse`, and each `@ExceptionHandler` is one arm of the
`match`. A `RuntimeException` doesn't have to be declared, so, as in JavaScript, nothing in the
controller's signature tells you it can fail. Forget a handler, and Spring answers `500`; forget a
`match` arm in Rust, and it doesn't compile.

**Go (`net/http` and Gin).** Go returns errors as values, like Rust, and checks them by hand:

```go
book, err := store.Find(id)
if errors.Is(err, ErrNotFound) {
    writeJSON(w, http.StatusNotFound, map[string]string{"error": err.Error()})
    return
}
```

`if err != nil { …; return }` is Go's spelled-out version of `?`, and a helper such as `writeJSON`
(you'd write it yourself) plays the part of `into_response`. Gin's `c.JSON(http.StatusNotFound,
gin.H{"error": "book not found"})` does the same in one call. Panics work much like Axum's, too:
`net/http` catches a panicking handler, logs it, and closes the connection, and Gin's
`gin.Default()` adds a *Recovery* middleware that answers `500` instead, which is what tower-http's
`CatchPanicLayer` does for Axum (*Common mistakes*).

## Common mistakes

The compiler errors below were captured in a scratch copy of this lesson's code, a project called
`my-errors`, so their line numbers are from that file.

**`.unwrap()` on a missing book.**
It's tempting to write the handler the short way, as Shared state did for the lock
(`cargo run -p error-handling-in-axum --example errors-unwrap-panics`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-unwrap-panics.rs:handler}}
```

It compiles, and `/books/2` works. Now ask for book 9, then for book 2 again. The book's checker
can't record this session, because curl reports a failure, so it was captured by hand. The checker
treats any curl that exits with an error (here `curl: (52)`) as a failed command and records
nothing, so the `date:` line the book usually hides is still there. The second terminal:

```text
$ curl -i http://127.0.0.1:3000/books/2
HTTP/1.1 200 OK
content-type: application/json
content-length: 23
date: Wed, 30 Sep 2026 17:45:51 GMT

{"id":2,"title":"Emma"}
$ curl -i http://127.0.0.1:3000/books/9
curl: (52) Empty reply from server
$ curl -i http://127.0.0.1:3000/books/2
curl: (52) Empty reply from server
```

Not a `404`, and not even a `500`: **no answer at all**. The first terminal shows why:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/examples/errors-unwrap-panics`
Listening on http://127.0.0.1:3000

thread 'tokio-rt-worker' (13207202) panicked at topics/error-handling-in-axum/examples/errors-unwrap-panics.rs:26:65:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'tokio-rt-worker' (13207202) panicked at topics/error-handling-in-axum/examples/errors-unwrap-panics.rs:25:36:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

Read the two panics in order:

- **The first, for book 9, at line 26.** `.find` gave `None`, and `.unwrap()` on a `None` panics:
  ``called `Option::unwrap()` on a `None` value``. A panic stops the handler before it has an answer
  to give, and Axum, with nothing to send, closes the connection. That's curl's
  `Empty reply from server`. The server itself keeps running: each connection is handled on its own.
- **The second, for book 2, at line 25, the lock line.** Book 2 exists, and worked a moment ago. But
  the first panic happened while `books`, the guard, was holding the lock, and that **poisoned** the
  `Mutex` (Step 6). Now every `.lock()` returns an `Err`, `.unwrap()` turns it into another panic,
  and the whole book list is out of reach until the server restarts. One request for a missing book
  broke every request after it.

**Fix:** a missing book is an expected answer, not a crash. Use `.ok_or(AppError::NotFound(id))?`,
as in Step 6, and `.map_err(|_| AppError::Poisoned)?` on the lock.

In production, add a safety net for the panics you didn't foresee: tower-http's
[`CatchPanicLayer`](https://docs.rs/tower-http/0.7.1/tower_http/catch_panic/index.html) catches a
panicking handler and answers `500 Internal Server Error` instead of closing the connection. (It
needs tower-http's `catch-panic` feature, and
[Middleware and Tower layers](middleware-and-tower-layers.md) shows how layers are added.) It's a net, not a fix: the lock is still poisoned after the panic, so an expected case such
as a missing book still belongs in `AppError`.

**Forgetting `impl IntoResponse for AppError`.**
You write the enum and the handler, but not Step 5:

```rust,noplayground,ignore
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("book {0} not found")]
    NotFound(u32),
    #[error("the book list is unavailable after an earlier crash")]
    Poisoned,
}

async fn show_book(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Book>, AppError> {
```

The build stops at the route, with the error you know from Handlers and IntoResponse:

```text
error[E0277]: the trait bound `fn(State<AppState>, Path<u32>) -> ... {show_book}: Handler<_, _>` is not satisfied
   --> src/main.rs:56:35
    |
 56 |         .route("/books/{id}", get(show_book))
    |                               --- ^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<AppState>, Path<u32>) -> ... {show_book}`
    |                               |
    |                               required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
```

It says **that** `show_book` isn't a valid handler, but not **why**. Two clues are in the warnings
above it: ``unused imports: `IntoResponse`, `Response`, and `http::StatusCode` `` and
``unused import: `serde_json::json` ``. You imported the tools for Step 5, and never used them. Add
`#[axum::debug_handler]` on the line above `async fn show_book` (it needs Axum's `macros` feature,
as in [Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes)), and the real
reason comes first:

```text
error[E0277]: the trait bound `AppError: IntoResponse` is not satisfied
  --> src/main.rs:35:6
   |
35 | ) -> Result<Json<Book>, AppError> {
   |      ^^^^^^ unsatisfied trait bound
   |
help: the trait `IntoResponse` is not implemented for `AppError`
  --> src/main.rs:24:1
   |
24 | enum AppError {
   | ^^^^^^^^^^^^^
   = help: the following other types implement trait `IntoResponse`:
             &'static [u8; N]
             &'static [u8]
             &'static str
             ()
             (R,)
             (Response<()>, R)
             (Response<()>, T1, R)
             (Response<()>, T1, T2, R)
           and 120 others
   = note: required for `Result<Json<Book>, AppError>` to implement `IntoResponse`
```

Read the last `note:`: a `Result` can be sent only if **both** sides can, and `AppError` can't.
Axum has no idea which status a `NotFound` deserves, and it won't guess. **Fix:** write Step 5.

**Every error becomes a `500`.**
A shortcut in `into_response`: one status for every error
(`cargo run -p error-handling-in-axum --example errors-all-500`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-all-500.rs:into_response}}
```

It compiles, and it runs:

```bash
{{#include ../../code/topics/error-handling-in-axum/http/71-all-500.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/71-all-500.out}}
```

The body says "not found", but the status says `500 Internal Server Error`, "the server is broken".
Programs believe the status, not the text. A client may retry, because a `500` might go away; a
monitoring tool counts it as an outage and can wake someone up at night; and the client has no way to
tell "you asked for a book that doesn't exist" from "our server is on fire". **Fix:** a `match`
with one arm per variant, as in Step 5, so each error gets the status that describes it.

## More examples

Each example is a complete program in `code/topics/error-handling-in-axum/examples/`. Stop any
running server with `Ctrl+C`, start the example with the command shown, and send its requests from
your second terminal.

### `BadInput`: 422 for a request that makes no sense

Adding a book with a blank title is valid JSON, but not a valid book. A new variant says so, and one
new arm gives it `422 Unprocessable Entity`
(`cargo run -p error-handling-in-axum --example errors-bad-input`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-bad-input.rs:error}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-bad-input.rs:add_book}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/50-bad-input.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/50-bad-input.out}}
```

`BadInput(String)` carries its own message, and `#[error("{0}")]` uses it as it is. `.trim()` cuts the
spaces from both ends, so `"   "` becomes empty. `return Err(…)` leaves the handler early, like `?`
does, and it happens **before** the lock, so a bad request never waits for it. `422` is the same code
Axum uses for well-formed JSON with the wrong shape, as in JSON with serde.

### Axum's rejections, as JSON too

In *Run it*, `/books/abc` answered in plain text, because the `Path` extractor refused it before
`show_book` ran. You can ask for the rejection **itself**, and turn it into an `AppError`
(`cargo run -p error-handling-in-axum --example errors-json-rejection`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-json-rejection.rs:imports}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-json-rejection.rs:error}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-json-rejection.rs:from}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-json-rejection.rs:handlers}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/51-json-rejection.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/51-json-rejection.out}}
```

Every answer is now JSON, with the same `{"error": …}` shape as `NotFound`, and each keeps the status
Axum chose: `400`, `400`, `422`, `415`, as in JSON with serde. Three pieces make it work:

- **`Result<Path<u32>, PathRejection>` as the extractor.** Normally, a failed extractor answers for
  you and your handler never runs. Wrap it in `Result`, and Axum always runs the handler, handing you
  `Ok(path)` or `Err(rejection)` to deal with yourself. The same works for `Json` with
  `JsonRejection`.
- **`impl From<JsonRejection> for AppError`.** `From` is Rust's standard trait for "a value of this
  type can become one of mine". Remember the note from Step 6: ``the question mark operation (`?`)
  implicitly performs a conversion on the error value using the `From` trait``. With these two
  `impl`s, `path?` and `body?` turn a rejection into an `AppError` by themselves, with no
  `.map_err`.
- **`Rejected { status, message }`.** A variant with **named** fields. `rejection.status()` and
  `rejection.body_text()` are Axum's own status and text for that rejection, so nothing is lost. In
  the `match`, `Rejected { status, .. }` takes out the `status` and ignores the rest (`..`), and
  `#[error("{message}")]` uses the text as the message.

### Log the real reason for a `500`

A `500`'s real reason is for you, not for the client. Here, `GET /books/{id}/cover` builds a link to a
book's cover from a setting, the [environment variable](../glossary.md#environment-variable)
`COVERS_URL`, and that setting is missing
(`cargo run -p error-handling-in-axum --example errors-logged`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-logged.rs:error}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-logged.rs:cover}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-logged.rs:app}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/52-logged.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/52-logged.out}}
```

The client gets a polite `something went wrong on our side`. The real reason went to the server's
terminal, captured here for real:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/examples/errors-logged`
Listening on http://127.0.0.1:3000
500 Internal Server Error: setting COVERS_URL is missing: environment variable not found
```

`eprintln!` is `println!` for errors: it writes to the terminal's error output, where logs belong.
`std::env::var` reads an environment variable, and gives an `Err` when it isn't set. The `404` for
book 9 wasn't logged: it's the client's doing, not a problem with the server. (A real app reads its
settings once, at startup, and keeps them in the state; *Config and .env*, in *Part A4*, does that.
And instead of `eprintln!`, a real app uses a logging library:
[Middleware and Tower layers](middleware-and-tower-layers.md) sets one up.)

### An error that names the book

A client program often wants the details as separate fields, not only inside a sentence
(`cargo run -p error-handling-in-axum --example errors-with-id`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-with-id.rs:into_response}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/53-with-id.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/53-with-id.out}}
```

This time the `match` gives two things, the status **and** the body, because the two variants need
differently shaped JSON. The pattern `AppError::NotFound(id)` names the variant's value `id`, so
`json!` can put it in its own field, `"id":9`, a JSON number. The message is made once, before the
`match`, with `let message = self.to_string();`, so both arms can use it. (Without extra settings, serde_json writes the keys of a `json!`
object in alphabetical order, whatever order you type them in.)

### A poisoned lock, handled

Step 6 said a poisoned lock stays poisoned, and that `.map_err(|_| AppError::Poisoned)?` answers
with a clear `500`. Here it is for real. This example is the lesson's program with one extra route,
`POST /crash`, which stands in for any bug that panics while holding the lock
(`cargo run -p error-handling-in-axum --example errors-poisoned`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-poisoned.rs:crash}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-poisoned.rs:app}}
```

Like the `.unwrap()` mistake, this session was captured by hand, because the crash makes curl report
a failure, which the checker can't record, so the `date:` lines the book usually hides are still
there. The second terminal:

```text
$ curl -i http://127.0.0.1:3000/books/2
HTTP/1.1 200 OK
content-type: application/json
content-length: 23
date: Wed, 30 Sep 2026 17:46:05 GMT

{"id":2,"title":"Emma"}
$ curl -i -X POST http://127.0.0.1:3000/crash
curl: (52) Empty reply from server
$ curl -i http://127.0.0.1:3000/books/2
HTTP/1.1 500 Internal Server Error
content-type: application/json
content-length: 63
date: Wed, 30 Sep 2026 17:46:05 GMT

{"error":"the book list is unavailable after an earlier crash"}
$ curl -i http://127.0.0.1:3000/books/1
HTTP/1.1 500 Internal Server Error
content-type: application/json
content-length: 63
date: Wed, 30 Sep 2026 17:46:05 GMT

{"error":"the book list is unavailable after an earlier crash"}
```

And the first terminal:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/examples/errors-poisoned`
Listening on http://127.0.0.1:3000

thread 'tokio-rt-worker' (13208039) panicked at topics/error-handling-in-axum/examples/errors-poisoned.rs:61:5:
a bug in this handler, while it holds the lock
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Before the crash, book 2 is fine. `/crash` takes the lock (`_books` keeps the guard alive) and panics,
so curl gets no answer, and the `Mutex` is poisoned. After that, **both** books answer `500`, and so
would every request that needs the list, until the server restarts. Compare it with the `.unwrap()`
mistake: the lock is equally broken, but the server still answers, in JSON, with a status that tells
the client the truth: it's our fault, not theirs.

### The same error type, without thiserror

Here's what `#[derive(thiserror::Error)]` and the `#[error("…")]` lines write for you, done by hand
(`cargo run -p error-handling-in-axum --example errors-without-thiserror`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-without-thiserror.rs:error}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/54-without-thiserror.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/54-without-thiserror.out}}
```

The answer is byte for byte the same as the lesson's. `impl std::fmt::Display` is the text for
people: its `fmt` method writes each variant's message with `write!`, which works like `format!`,
and `{id}` puts the variant's value in. `impl std::error::Error for AppError {}` has an empty body,
because every method of that trait has a ready-made default. That's all thiserror saves, but it
saves it for every variant, and it keeps each message on the line above the variant it belongs to.

## Your turn

Each solution is also a complete program in `code/topics/error-handling-in-axum/examples/`, with a
test inside.

### 🟢 Guided

Book 13 was removed from the catalogue, and the API should say so with `410 Gone`, which means "this
existed, and it's gone for good", instead of `404`. Add a variant `Gone(u32)` with the message
`book 13 was removed from the catalogue`, give it `StatusCode::GONE` in `into_response`, and at the
top of `show_book`, return `Err(AppError::Gone(id))` when `id` is 13.

<details><summary>Solution</summary>

`cargo run -p error-handling-in-axum --example errors-gone`:

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-gone.rs:error}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-gone.rs:handler}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/90-gone.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/90-gone.out}}
```

Three small changes, and the compiler checks one of them for you: forget the `Gone(_)` arm in the
`match`, and the build stops with ``non-exhaustive patterns``. The `if id == 13` check comes before
the lock, so book 13 is answered without touching the list. Books that were never there still get
`404`.

</details>

### 🟡 Tweak

Give every error body a second field, `"code"`, a short fixed word that a client program can compare
against, such as `{"code":"not_found","error":"book 9 not found"}`. Use `"internal"` for `Poisoned`.
Hint: let the `match` give a `(status, code)` pair.

<details><summary>Solution</summary>

`cargo run -p error-handling-in-axum --example errors-code-field`:

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-code-field.rs:into_response}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/91-code-field.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/91-code-field.out}}
```

`let (status, code) = match self { … };` takes the pair apart into two variables. Why a code, when
there's already a message? The message is for people, and you may reword it tomorrow; the code is
for programs, and it never changes, so a client can write `if (error.code === "not_found")` and
trust it.

</details>

### 🔴 From scratch

Add `DELETE /books/{id}`. It removes the book and answers `204 No Content`, or answers the usual
`404` JSON error through `AppError` when there's no such book. Hints: the lock needs `mut`;
`.iter().position(|book| book.id == id)` gives the book's place in the list as an `Option<usize>`;
`books.remove(index)` takes it out; and a handler can return `Result<StatusCode, AppError>`.

<details><summary>Solution</summary>

`cargo run -p error-handling-in-axum --example errors-delete`:

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-delete.rs:delete_book}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-delete.rs:app}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/92-delete.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/92-delete.out}}
```

The first `DELETE` answers `204` with no body; after it, `GET` and a second `DELETE` both answer
`404`, and book 1 is untouched. The handler has the same shape as `show_book`: `.position` instead of
`.find`, because `.remove` needs the book's place in the list, not the book, and the same
`.ok_or(AppError::NotFound(id))?` for "no such book". `.delete(delete_book)` chains onto the same
route, as in [Routes and HTTP methods](routes-and-methods.md).

</details>

## Quick check

<div class="quiz" data-topic="error-handling-in-axum"></div>

## Remember this

- One `enum AppError` for the app, one variant per kind of failure; `#[derive(thiserror::Error)]`
  and `#[error("…")]` give each one its message.
- `impl IntoResponse for AppError` is the one place that decides each error's status code and JSON
  body. Give each variant the status that tells the truth: `404` is the client's miss, `500` is your
  fault.
- Handlers return `Result<T, AppError>`; `?` sends an error out, and `.ok_or(…)?` turns "maybe
  nothing" into a clear error.
- Don't `.unwrap()` an expected case: a panic sends no answer at all, and a panic while holding a
  lock poisons it for every later request. `.map_err(|_| AppError::Poisoned)?` answers with a clear
  `500` instead.
- Extractor rejections bypass `AppError` unless you take them as `Result<…, Rejection>` and convert
  them with `From`.

## Go deeper

- [Rust for Humans: Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
- [Rust for Humans: The question mark operator](https://open-source-bd.github.io/rustbook-for-human/abstractions/the-question-mark-operator.html)
- [Rust for Humans: Custom error types](https://open-source-bd.github.io/rustbook-for-human/abstractions/custom-error-types.html)
- [Rust for Humans: Error crates: thiserror and anyhow](https://open-source-bd.github.io/rustbook-for-human/abstractions/error-crates-thiserror-and-anyhow.html)
- [thiserror](https://docs.rs/thiserror/2.0.21/thiserror/) — Official reference: every #[error(…)] form, #[from], #[source] and transparent errors.
- [axum::error_handling](https://docs.rs/axum/0.8.9/axum/error_handling/index.html) — Official reference: Axum's error model, and why handlers can't fail without an answer.

<!-- next:start -->

**Next:**

- [Middleware and Tower layers](../a2-axum/middleware-and-tower-layers.md)

<!-- next:end -->
