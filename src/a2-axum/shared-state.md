# Shared state

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can give every handler access to shared data with `State`.
- You know why the data is wrapped in `Arc<Mutex<…>>`.
- You know this memory is lost on restart — and why that's the database's job.

## What & why

Share data between requests with `State<T>`, `Arc` and `Mutex`: a books list every handler can read and change.

Every server so far has had a short memory. In [JSON with serde](json-and-serde.md), `POST /books`
answered with a new book, and then forgot it at once: ask for the list of books and there was
nowhere to find it. A real backend has to **remember**. When one request adds Dune, the next
request, from anyone, should see Dune in the list.

That's harder than it sounds, because a server doesn't serve one request at a time. Picture a
library help desk with several librarians. Visitors walk up to whichever librarian is free, and
they're all served **at the same time**. Axum works the same way: Tokio, the
[runtime](../glossary.md#runtime) underneath Axum, runs several *worker threads*, workers that each
run code at the same moment as the others, usually one per core of your computer's processor. Each
request is handed to whichever worker is free.

So where does the list of books go? Not in one librarian's head, because the next visitor may get
a different librarian. It goes on **one shared whiteboard** that every librarian can see. In Axum,
that whiteboard is called the app's [**state**](../glossary.md#state): data you hand to the Router
once, which every [handler](../glossary.md#handler) can then reach.

A shared whiteboard brings two problems, and Rust has a tool for each:

- **Many hands must hold it.** Every request needs its own way to reach the same list. That's
  [**`Arc`**](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/shared-state-mutex-and-arc.html):
  "many hands can hold it". Rust for Humans reminder: an `Arc` is a pointer to one value that
  can have many owners, and the value stays alive until the last owner lets go.
- **Two writers at once make a mess.** If two librarians write on the same spot of the
  whiteboard at the same instant, you get scribbles. That's
  [**`Mutex`**](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/shared-state-mutex-and-arc.html):
  "one writer at a time". Rust for Humans reminder: a `Mutex` is a lock around a value; you
  `.lock()` it to get in, and anyone else who wants in waits until you're done.

```text
 request A ──▶ worker 1 ──┐
 request B ──▶ worker 2 ──┼──▶  Arc  ──▶  Mutex  ──▶  [Dune, Emma]
 request C ──▶ worker 3 ──┘   (shared)   (one at     (the list)
                                          a time)
```

This lesson has three ideas:

1. **`State`**: give the Router your state once with `.with_state(…)`, and take it in any handler
   with the `State(state): State<AppState>` [extractor](../glossary.md#extractor).
2. **`Arc<Mutex<Vec<Book>>>`**: the list, wrapped so that every request can share it, and only one
   can change it at a time.
3. **Memory dies with the process.** Stop the server, and the books are gone. Keeping them is the
   database's job, in *Part A3*.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/shared-state`. Its `Cargo.toml` has the same lines as in
JSON with serde: `Arc` and `Mutex` come with Rust itself, in its standard library, so there's
nothing new to install.

```toml
{{#include ../../code/topics/shared-state/Cargo.toml}}
```

### Line by line

`[package]` · `name = "shared-state"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p shared-state`.
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
- **Why:** the two crates every Axum server needs. `State` comes with Axum.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`serde.workspace = true`
- **What:** serde, with its `derive` feature.
- **Why:** `Book` goes out as JSON and `NewBook` comes in as JSON, as in JSON with serde.
- **How:** `#[derive(Serialize)]` and `#[derive(Deserialize)]` come from it.
- **Remove it and…** ``error[E0432]: unresolved import `serde` ``, pointing at the
  `use serde::…;` line.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the answers, without opening a
  port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, and the test crates from
`cargo add --dev tower http-body-util --features tower/util`. There's no `cargo add` for `Arc` or
`Mutex`: they're part of Rust's standard library, `std`, which every Rust program already has.

### Step 2: the `use` lines

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:imports}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`use axum::{Json, Router, extract::State, http::StatusCode, routing::get};`
- **What:** brings five names from Axum into this file.
- **Why:** `Json`, `Router` and `get` are old friends; `State` is this lesson's new extractor, and
  `StatusCode` lets `POST` answer `201 Created`.
- **How:** `State` lives in `axum::extract`, next to `Path`, `Query` and `Json`. `StatusCode` is the
  one from [Handlers and IntoResponse](handlers-and-into-response.md).
- **Remove it and…** (`extract::State`) ``cannot find type `State` in this scope``, and the same
  for the function-like `State(…)` in each handler.

`use serde::{Deserialize, Serialize};`
- **What:** serde's two abilities, as in JSON with serde.
- **Why:** `Book` is serialized (sent out); `NewBook` is deserialized (received).
- **How:** a separate `use` line, because it comes from a different crate.
- **Remove it and…** ``cannot find derive macro `Serialize` in this scope``.

`use std::sync::{Arc, Mutex};`
- **What:** brings in `Arc` and `Mutex` from the standard library's `sync` module.
- **Why:** Step 4 wraps the list of books in both.
- **How:** `std` is Rust's standard library, and `sync` is its corner for things that are shared
  between threads. It needs no line in `Cargo.toml`.
- **Remove it and…** ``cannot find type `Arc` in this scope``, and the same for `Mutex`.

### Step 3: the two structs

The same `Book` and `NewBook` as in JSON with serde, with one change and one field fewer:

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:types}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`#[derive(Clone, Serialize)]`
- **What:** a `Book` can be sent as JSON (`Serialize`) and **copied** (`Clone`).
- **Why:** the list of books lives in the state, and the handlers in Step 5 hand **copies** of the
  books to the response, so that the originals stay in the list. (`in_stock` is gone: this lesson is
  about remembering, so a book is an id and a title.)
- **How:** `#[derive(Clone)]` writes a `.clone()` method that copies every field: here, the number
  and the text of the title.
- **Remove it and…** (`Clone`) both handlers stop compiling. In `list_books`, the error is
  ``the method `clone` exists for struct `std::sync::MutexGuard<'_, Vec<Book>>`, but its trait
  bounds were not satisfied``, and its note says why: `` `Book: Clone` `` … ``which is required by
  `Vec<Book>: Clone` ``. A list can be copied only if its items can.

`struct Book {` · `id: u32,` · `title: String,` · `}`
- **What:** a book as the server stores it and shows it.
- **Why:** the server chooses the `id`, the client chooses the `title`.
- **How:** as in JSON with serde: `id` becomes a JSON number, `title` a JSON string.
- **Remove it and…** (`id`) there's no number to find a book by, which later lessons need.

`#[derive(Deserialize)]` · `struct NewBook {` · `title: String,` · `}`
- **What:** what the client sends to add a book: only a title.
- **Why:** no `id`, because the server decides it (JSON with serde explained why).
- **How:** the `Json` extractor fills it from the request's body.
- **Remove it and…** (`Deserialize`) `add_book` is no longer a valid handler, and the build stops
  with the familiar ``Handler<_, …>`` error at `.post(add_book)`, plus a warning
  ``unused import: `Deserialize` ``.

### Step 4: the state

Here's the whiteboard: one struct holding everything the handlers share.

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:state}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`struct AppState {` · `}`
- **What:** a struct that holds the app's shared data.
- **Why:** today it has one field. Real apps grow more, such as settings and, in *Part A3*, a
  connection to the database. A struct gives them all one home, and *More examples* shows a state
  with two fields.
- **How:** `AppState` is only a name we chose; Axum doesn't care what you call it.
- **Remove it and…** there's nothing to hand to `.with_state` in Step 6.

`books: Arc<Mutex<Vec<Book>>>,`
- **What:** the list of books, in two wrappers. Read it from the inside out: a `Vec<Book>` (the
  list), inside a `Mutex` (a lock: one at a time), inside an `Arc` (shared by many).
- **Why:** each wrapper answers one of the two problems from *What & why*. Step 4½ below takes the
  three layers one at a time.
- **How:** `Vec<Book>` is Rust's growable list, which you met as the JSON array in JSON with serde.
- **Remove it and…** see Step 4½: each layer has its own error.

`#[derive(Clone, Default)]`
- **What:** gives `AppState` two abilities: to be copied (`Clone`), and to be made "empty" with
  `AppState::default()` (`Default`).
- **Why (`Clone`):** Axum gives **each request its own copy** of the state. That sounds wasteful, but
  it isn't: cloning an `Arc` doesn't copy the list. It copies the **pointer** to the list, and
  counts one more owner. Every copy points at the same, single list, which is exactly what makes it
  shared. Cloning `AppState` costs about as much as copying one number.
- **Why (`Default`):** `AppState::default()` builds the starting state: an `Arc` holding a `Mutex`
  holding an empty `Vec`. Each type has its own "empty" value, and `Default` builds them all.
- **How:** `#[derive(Default)]` fills every field with its type's default; for our field that's an
  empty list, wrapped.
- **Remove it and…** (`Clone`) the build stops with the ``Handler<_, _>`` error at both routes;
  *You might be wondering…* shows the clearer message. (`Default`) the build stops at Step 6 with
  ``no associated function or constant named `default` found for struct `AppState` ``.

### Step 4½: `Arc<Mutex<Vec<Book>>>`, one layer at a time

```text
Arc<Mutex<Vec<Book>>>
│   │     └── Vec<Book>   the list itself: [Dune, Emma, …]
│   └──────── Mutex<…>    a lock: one handler at a time may touch the list
└──────────── Arc<…>      shared: every request holds a pointer to the same lock
```

### Line by line

`Vec<Book>`
- **What:** the list itself.
- **Why:** it's what we want to remember.
- **How:** `.push(book)` adds a book to the end; `.len()` counts them.
- **Remove it and…** there's nothing to remember.

`Mutex<…>`
- **What:** a lock around the list. To read or change the list, a handler calls `.lock()`, and gets
  the list only when no other handler has it. Everyone else waits for their turn.
- **Why:** two requests adding a book at the same instant must not trample each other. Without a
  lock, both could read "the list has 1 book", both could pick id 2, and one book could be lost.
  Rust won't even let you try: an `Arc` on its own only lets you **read** what it holds.
- **How:** the lock opens again, by itself, when the handler is done with the list (Step 5 shows
  exactly when).
- **Remove it and…** (write `Arc<Vec<Book>>`) the build stops at the first `.lock()` with
  ``no method named `lock` found for struct `Arc<Vec<Book>>` ``. Take the `.lock()` calls out too,
  and the `.push` fails instead: ``cannot borrow data in an `Arc` as mutable`` (both tried in a
  scratch copy).

`Arc<…>`
- **What:** a pointer to the lock that can have many owners. Its name stands for *Atomically
  Reference Counted*: it counts its owners, safely even when threads count at the same moment, and
  frees the list when the last owner is gone.
- **Why:** each request gets its own clone of `AppState`, and every clone must reach the **same**
  lock and the **same** list.
- **How:** Rust for Humans'
  [Smart pointers](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/smart-pointers.html)
  lesson shows `Arc` next to its single-thread sibling, `Rc`. *Common mistakes* shows what happens
  when you pick `Rc` here.
- **Remove it and…** (write `Mutex<Vec<Book>>`) the build stops at `#[derive(Clone, Default)]` with
  ``the trait bound `std::sync::Mutex<Vec<Book>>: Clone` is not satisfied``. A `Mutex` can't be
  copied, and it shouldn't be: two copies of the lock would be two whiteboards.

### Step 5: the handlers

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:handlers}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {`
- **What:** a handler that takes the app's state and answers with every book, as a JSON array.
- **Why:** this is `GET /books`: "show me the list".
- **How:** read the parameter in its two halves, as with `Path` and `Json`. The type,
  `State<AppState>`, asks Axum for the state extractor. The pattern, `State(state)`, opens the box,
  so `state` **is** this request's copy of `AppState`. Unlike `Path` or `Json`, `State` doesn't read
  the request at all: it hands you what you gave `.with_state` in Step 6.
- **Remove it and…** (the parameter) ``cannot find value `state` in this scope`` on the next line.

`let books = state.books.lock().unwrap();`
- **What:** locks the list, and names what's inside `books`.
- **Why:** you can't touch the list without holding the lock. That's the whole point of the `Mutex`.
- **How:** `state.books` is the `Arc`, and `.lock()` goes through it to the `Mutex`. If another
  handler holds the lock, this line waits. What comes back is a **guard** (its type is
  `MutexGuard`): it lets you use the list as if it were a plain `Vec`, and it keeps the lock closed
  for as long as it exists. `.unwrap()` is explained in *You might be wondering…*.
- **Remove it and…** (`.unwrap()`) the next line fails with ``the method `clone` exists for enum
  `Result<MutexGuard<'_, Vec<Book>>, PoisonError<MutexGuard<'_, ...>>>`, but its trait bounds were
  not satisfied``, because `.lock()` returns a `Result` that still has to be opened.

`Json(books.clone())`
- **What:** copies the list, and sends the copy as JSON.
- **Why:** `books` is the guard, a key to the list, not the list itself. The response needs a
  `Vec<Book>` of its own, which lives on after the lock opens again.
- **How:** `.clone()` goes through the guard and copies the `Vec` and every `Book` in it. That's why
  `Book` needed `Clone` in Step 3.
- **Remove it and…** (write `Json(books)`) ``mismatched types``: ``expected `Vec<Book>`, found
  `MutexGuard<'_, Vec<Book>>` ``. You can't send the key; you send a copy of what it opens.

`}` (the end of `list_books`)
- **What:** the end of the function, where `books`, the guard, goes away.
- **Why:** when a guard goes away, the lock **opens by itself**. There's no `unlock()` to forget.
- **How:** Rust drops every local variable at the closing `}` of its block, and dropping a guard
  unlocks the `Mutex`.
- **Remove it and…** you can't: every block ends somewhere. *Common mistakes* shows why *where* it
  ends matters.

`async fn add_book(` · `State(state): State<AppState>,` · `Json(input): Json<NewBook>,` · `) -> (StatusCode, Json<Book>) {`
- **What:** a handler that takes the state **and** a JSON body, and answers with a status code and
  the new book.
- **Why:** this is `POST /books`: "add this book".
- **How:** two extractors. `Json` reads the body, so it comes **last**, as you learned in
  [JSON with serde](json-and-serde.md#common-mistakes); `State` reads nothing from the request, so it
  can go first. The tuple answer `(StatusCode, Json<Book>)` is from Handlers and IntoResponse: status
  first, then the body.
- **Remove it and…** (`State(state): …`) the handler has no way to reach the list.

`let mut books = state.books.lock().unwrap();`
- **What:** locks the list, this time to **change** it.
- **Why:** adding a book changes the list, so the variable must be `mut`.
- **How:** the same lock as in `list_books`. While this handler holds it, a `GET /books` that
  arrives waits, for the tiny moment it takes to push one book.
- **Remove it and…** (`mut`) ``cannot borrow `books` as mutable, as it is not declared as
  mutable``, pointing at `books.push`.

`let book = Book {` · `id: books.len() as u32 + 1,` · `title: input.title,` · `};`
- **What:** builds the new book. Its id is "how many books there are, plus one".
- **Why:** the first book gets `1`, the second `2`, and so on, which is enough for this lesson.
- **How:** `.len()` counts the list and gives a `usize`, Rust's type for sizes; `as u32` converts it
  to the `u32` the `id` field wants. It's safe to count here **because we hold the lock**: no other
  request can add a book between our counting and our pushing, so no two books get the same id.
  (Once books can be deleted one by one, counting stops working; a database chooses ids properly, in
  *Part A3*.)
- **Remove it and…** (`as u32`) ``mismatched types``: ``expected `u32`, found `usize` ``.

`books.push(book.clone());`
- **What:** adds a copy of the new book to the end of the shared list.
- **Why:** this line is the memory: from now on, every request sees this book.
- **How:** the copy goes into the list, and the original `book` stays in our hands to send back.
- **Remove it and…** (`.clone()`, writing `books.push(book)`) ``use of moved value: `book` ``,
  pointing at the next line: `push` took the book, so there's nothing left to send.

`(StatusCode::CREATED, Json(book))`
- **What:** answers `201 Created`, with the new book as JSON.
- **Why:** `201` is the status code for "I made something new", and the client learns the id the
  server chose.
- **How:** the guard `books` is dropped at the `}` below, so the lock opens as the handler ends.
- **Remove it and…** (`StatusCode::CREATED,`) ``mismatched types``: the return type still promises
  ``(StatusCode, Json<Book>)``, and found a lone ``Json<Book>``. Change both, to `-> Json<Book>` and
  `Json(book)`, and it compiles, answering `200 OK`, which tells the client less.

### Step 6: the Router, with `.with_state`

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:app}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router, starting from an empty one.
- **Why:** a separate function, so `main` and the tests use the same Router.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** there's no Router to add routes to.

`.route("/books", get(list_books).post(add_book))`
- **What:** one path, two methods: `GET /books` lists, `POST /books` adds.
- **Why:** it's the same collection of books, so it's the same address, as in
  [Routes and HTTP methods](routes-and-methods.md).
- **How:** `.post(…)` chains a second method onto the same route.
- **Remove it and…** (`.post(add_book)`) `POST /books` gets `405 Method Not Allowed`.

`.with_state(AppState::default())`
- **What:** creates the starting state, an empty list, and hands it to the Router.
- **Why:** the handlers asked for `State<AppState>`; this is where that state comes from. It's
  created **once**, when the app starts, and every request after that gets a cheap clone of it.
- **How:** until `.with_state`, the Router is one that still **needs** an `AppState` before it can
  answer anything. `.with_state` fills that need, and what comes out is a plain `Router`, which is
  what `fn app() -> Router` promised and what `axum::serve` accepts.
- **Remove it and…** the build stops with ``mismatched types``. *Common mistakes* shows the full
  error, and what it means.

### Step 7: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/shared-state/src/main.rs:main}}
```

📁 Full code: code/topics/shared-state · ▶ Run it: `cd code && cargo run -p shared-state`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's start, running on the Tokio runtime.
- **Why:** `main` needs `.await`, and `#[tokio::main]` is what allows that. It's also what starts
  the worker threads from *What & why*.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line of `main`.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`let listener = …bind("127.0.0.1:3000")…;` · `println!(…);`
- **What:** claims port 3000, then prints `Listening on http://127.0.0.1:3000`.
- **Why:** a server must own a port before anyone can reach it.
- **How:** if the port is taken, `.expect` stops the program with our message.
- **Remove it and…** there's no port for curl to reach.

`axum::serve(listener, app())` · `.await` · `.expect(…);`
- **What:** builds the Router, state included, then answers requests until you stop it.
- **Why:** `app()` runs **once**, so there's exactly one `AppState::default()`, and one list, for the
  whole life of the server.
- **How:** for every request, Axum clones the state and hands the clone to the handler.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p shared-state
```

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/shared-state`
Listening on http://127.0.0.1:3000
```

(The first time, you'll also see `Compiling` lines above these.) In the second terminal, ask for the
list, add two books, and ask again:

```bash
{{#include ../../code/topics/shared-state/http/01-remembers.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/01-remembers.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **The first `GET`: `[]`.** An empty JSON array: the server has only now started, and the list is
  empty. It's 2 bytes long, `content-length: 2`.
- **The two `POST`s: `201 Created`.** Dune gets id `1`, Emma gets id `2`: each `add_book` counted
  the list under the lock.
- **The last `GET`: both books.** The server **remembered**. Each of the four requests got its own
  clone of `AppState`, and every clone pointed at the same list.

Now see what a restart does. With a freshly started server, add Dune in the second terminal and
check that it's in the list. Then, in the first terminal, press **`Ctrl+C`** to stop the server,
and start it again with the same command. Finally, ask for the list again in the second terminal.
This session was captured for real. The first terminal:

```text
$ cargo run -p shared-state
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/shared-state`
Listening on http://127.0.0.1:3000
^C
$ cargo run -p shared-state
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/shared-state`
Listening on http://127.0.0.1:3000
```

The second terminal: the first two commands ran before the `^C`, the last one after the restart.
Because this session was typed by hand, not run by the book's checker, the `date:` lines are still
there:

```text
$ curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Dune"}'
HTTP/1.1 201 Created
content-type: application/json
content-length: 23
date: Wed, 30 Sep 2026 17:25:50 GMT

{"id":1,"title":"Dune"}
$ curl -i http://127.0.0.1:3000/books
HTTP/1.1 200 OK
content-type: application/json
content-length: 25
date: Wed, 30 Sep 2026 17:25:50 GMT

[{"id":1,"title":"Dune"}]
$ curl -i http://127.0.0.1:3000/books
HTTP/1.1 200 OK
content-type: application/json
content-length: 2
date: Wed, 30 Sep 2026 17:25:51 GMT

[]
```

Dune is **gone**. `^C` is how the terminal shows the `Ctrl+C` you pressed. It ended the program, and
the list lived only in that program's memory, so it ended with it. The new server started with a new
`AppState::default()`: an empty list. The same happens when the computer restarts, when the server
crashes, and every time you deploy a new version.

That's not a bug in our code: it's what memory is. State is the right place for things that may be
lost, such as a visit counter or settings read at startup. Anything that must **survive**, such as
books, users or orders, belongs in a database, which writes it to disk. That's
[Part A1](../a1-postgres/what-is-a-database.md)'s Postgres, and *Part A3* connects it to Axum.

- ✅ If the last `GET` of the script shows Dune and Emma, your state is shared between requests.
- ✅ If the list is `[]` after a restart, you've seen memory die with the process.
- ❌ If the build fails with ``mismatched types`` at `.route(…)`, check that `.with_state(…)` is
  there: *Common mistakes* has the full error.
- ❌ If the last `GET` shows `[]` without a restart, check that `app()` is called only once, in
  `main`. Every call to `app()` makes a brand-new, empty state.

## You might be wondering…

**"Why not a global variable?"**
Rust does allow one, such as `static BOOKS: Mutex<Vec<Book>> = Mutex::new(Vec::new());`, still with
a `Mutex`, for the same reason. But state is better in three ways. First, **each `app()` gets its
own**: this crate's tests build a fresh app for every test (`a_new_app_starts_empty`), while a
global would be shared by every test, and one test's books would leak into the next. Second, a
handler's parameters now **say what it uses**: `State<AppState>` in the signature, instead of a
hidden global somewhere. Third, a `static` must be built before the program runs, and in *Part A3*
the state holds a connection to the database, which only exists once the program has started and
connected.

**"What does `.unwrap()` on `.lock()` mean here?"**
`.lock()` returns a `Result`, which is an error only if the lock is **poisoned**: another handler
panicked (crashed) while it held the lock, so the list might be half-changed. `.unwrap()` means
"if that ever happens, crash this request too", which is a fair choice for a lesson; [Error handling
in Axum](error-handling-in-axum.md) comes back to it.

**"Is `Mutex` slow?"**
Not in the way that matters here. Locking a `Mutex` that nobody else holds takes a tiny fraction of
a microsecond, far less than the time a request spends travelling over the network. It only slows
you down when handlers **wait** for each other, and they only wait as long as someone holds the
lock. So hold it briefly: lock, do the quick change, let go. Never keep it while doing something
slow (*Common mistakes* shows the compiler stopping you). For data that is read far more often than
written, `RwLock` lets many readers in at once, and for a single number, an atomic needs no lock at
all: *More examples* shows both.

**"Why must `AppState` be `Clone`?"**
Because Axum gives every handler call its **own** copy of the state: the `State` extractor clones
what you passed to `.with_state`. With `Arc` inside, that clone copies a pointer, not the list. If
you forget `Clone`, the plain build shows the ``Handler<_, _>`` error at both routes. With
`#[axum::debug_handler]` above `list_books` (it needs Axum's `macros` feature, as in
[Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes)), a scratch copy of this
lesson's code, a project called `my-state`, names the real cause:

```text
error[E0277]: the trait bound `AppState: Clone` is not satisfied
  --> src/main.rs:22:35
   |
22 | async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
   |                                   ^^^^^ the trait `Clone` is not implemented for `AppState`
…
help: consider annotating `AppState` with `#[derive(Clone)]`
   |
17 + #[derive(Clone)]
18 | struct AppState {
   |
```

**"Does every request really get its own copy? Then how is anything shared?"**
Every request gets its own copy of `AppState`, and each copy holds an `Arc`. All those `Arc`s point
at the **one** `Mutex`, which holds the **one** list. Think of the copies as keys to the same room:
handing out another key doesn't build another room.

**"Why `std::sync::Mutex`? Doesn't Tokio have its own?"**
It does: `tokio::sync::Mutex`, whose lock can be held across an `.await`. It's slower, and Tokio's
own documentation recommends the standard `Mutex` whenever you only hold the lock for quick work
with no `.await` inside, which is the case here, and in most handlers.

## Coming from another language?

Every framework has to share data between requests. What differs is who stops two requests from
writing at once: you, the runtime, or the compiler.

**Express (Node.js).** You'd put the list in a variable at the top of the file:

```js
const books = [];

app.get("/books", (req, res) => res.json(books));
app.post("/books", (req, res) => {
  const book = { id: books.length + 1, title: req.body.title };
  books.push(book);
  res.status(201).json(book);
});
```

No lock is needed, because Node runs your JavaScript on **one** thread: two handlers never run at
the same instant. The price is that one busy handler holds up every other request. The memory is lost
on restart, exactly as in Axum.

**Flask and FastAPI (Python).** A module-level `books = []` works too, but Flask's development server
serves requests on several threads, so a careful app guards changes with a `threading.Lock()`,
Python's `Mutex`. Production servers such as Gunicorn often run several **processes**, and each has
its **own** copy of the list, so different requests may see different lists: another reason real
data goes in a database. FastAPI's `async def` handlers all run on one thread, like Node, and
`app.state` is its home for shared objects.

**Spring Boot (Java).** Shared data lives in a *bean*, one object that Spring creates and passes to
every controller:

```java
@Service
public class BookStore {
    private final List<Book> books = new ArrayList<>();

    public synchronized Book add(String title) {
        Book book = new Book(books.size() + 1, title);
        books.add(book);
        return book;
    }
}
```

The bean is Spring's `AppState`, and passing it into a controller's constructor is its
`.with_state`. `synchronized` is Java's `Mutex`, but it's up to you to remember it: leave it out and
the code compiles, and fails only when two requests collide. In Rust, forgetting the `Mutex` doesn't
compile, as Step 4½ showed.

**Go (`net/http` and Gin).** The usual pattern is a struct with a `sync.Mutex` next to the data, and
handlers as its methods:

```go
type Store struct {
    mu    sync.Mutex
    books []Book
}

func (s *Store) add(w http.ResponseWriter, r *http.Request) {
    s.mu.Lock()
    defer s.mu.Unlock()
    // ... decode the body, append to s.books, write the JSON answer
}
```

`defer s.mu.Unlock()` is Go's version of our guard opening the lock at the closing `}`. Go lets you
touch `s.books` without locking, and its race detector (`go run -race`) can catch that while the
program runs; Rust refuses it before the program exists. Gin works the same way, with the store
reached through a closure or a method.

## Common mistakes

The errors below were captured in the scratch copy called `my-state`, so their line numbers are
from that file (the same code as this lesson, without the `ANCHOR` comment lines).

**Forgetting `.with_state`.**
You write the routes, and forget the line that hands over the state:

```rust,noplayground,ignore
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
}
```

```text
error[E0308]: mismatched types
   --> src/main.rs:41:26
    |
 41 |         .route("/books", get(list_books).post(add_book))
    |          -----           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `MethodRouter`, found `MethodRouter<AppState>`
    |          |
    |          arguments to this method are incorrect
    |
    = note: expected struct `MethodRouter<()>`
               found struct `MethodRouter<AppState>`
note: method defined here
   --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-0.8.9/src/routing/mod.rs:178:12
    |
178 |     pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    |            ^^^^^
```

Read the `note:` lines. A Router and its routes carry a label saying **which state they still
need**. `MethodRouter<AppState>` is a route whose handlers need an `AppState`. `MethodRouter<()>`
is one that needs nothing: `()`, the empty value, means "no state". The function says it returns a
plain `Router`, which is short for `Router<()>`, a Router that needs nothing more. So Rust expected
routes that need nothing, and found routes that need an `AppState` no one has given. The fix is the
missing line: `.with_state(AppState::default())` gives the state, and turns the Router that needs an
`AppState` into one that needs nothing.

If you try to fix it by changing the return type to `fn app() -> Router<AppState>`, the error moves
to `main` instead, because a Router that still needs its state can't be served:

```text
error[E0277]: the trait bound `for<'a> Router<AppState>: tower_service::Service<IncomingStream<'a, tokio::net::TcpListener>>` is not satisfied
   --> src/main.rs:50:27
    |
 50 |     axum::serve(listener, app())
    |     -----------           ^^^^^ the trait `for<'a> tower_service::Service<IncomingStream<'a, tokio::net::TcpListener>>` is not implemented for `Router<AppState>`
    |     |
    |     required by a bound introduced by this call
    |
help: `Router` implements trait `tower_service::Service<Request>`
   --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-0.8.9/src/routing/mod.rs:549:5
    |
549 | /     impl<L> Service<serve::IncomingStream<'_, L>> for Router<()>
…
```

It's long, but the words to find are `Router<AppState>` (what you have) and `for Router<()>` (the
only kind `axum::serve` accepts). **Fix:** keep `fn app() -> Router`, and end the chain with
`.with_state(…)`.

**Using `Rc` instead of `Arc`.**
[Rust for Humans](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/smart-pointers.html)
taught `Rc` first, and it looks the same, so you write:

```rust,noplayground,ignore
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct AppState {
    books: Rc<Mutex<Vec<Book>>>,
}
```

The build gives four errors. The first two are the ``Handler<_, _>`` error you know, at
`get(list_books)` and at `.post(add_book)`. The third says what's wrong:

```text
error[E0277]: `Rc<std::sync::Mutex<Vec<Book>>>` cannot be sent between threads safely
   --> src/main.rs:41:5
    |
 41 |     Router::new()
    |     ^^^^^^^^^^^^^ `Rc<std::sync::Mutex<Vec<Book>>>` cannot be sent between threads safely
    |
    = help: within `AppState`, the trait `Send` is not implemented for `Rc<std::sync::Mutex<Vec<Book>>>`
note: required because it appears within the type `AppState`
   --> src/main.rs:18:8
    |
 18 | struct AppState {
    |        ^^^^^^^^
note: required by a bound in `Router::<S>::new`
   --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-0.8.9/src/routing/mod.rs:140:16
    |
140 |     S: Clone + Send + Sync + 'static,
    |                ^^^^ required by this bound in `Router::<S>::new`
…
```

and the fourth is its twin, ``cannot be shared between threads safely``, for the trait `Sync`.
`Send` is the ability "may be sent to another thread", and `Sync` is "may be used from several
threads at once". Axum's line `S: Clone + Send + Sync + 'static` demands both of your state, because
its worker threads pass requests, and your state, between them. `Rc` counts its owners in a way
that's only safe on one thread: two threads adding an owner at the same instant could miscount, and
the list could be freed while still in use. So Rust marks `Rc` as not `Send`, and refuses at build
time. **Fix:** `Arc`, the *atomic* version, whose counting is safe across threads. The `A` is the
whole difference.

**Holding the lock across an `.await`.**
Suppose `add_book` must do something slow after saving the book, such as sending an email, and you
write it inside the locked part. Here a 100-millisecond `tokio::time::sleep` stands in for the slow
work:

```rust,noplayground,ignore
async fn add_book(
    State(state): State<AppState>,
    Json(input): Json<NewBook>,
) -> (StatusCode, Json<Book>) {
    let mut books = state.books.lock().unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let book = Book {
        id: books.len() as u32 + 1,
        title: input.title,
    };
    books.push(book.clone());
    (StatusCode::CREATED, Json(book))
}
```

The plain build gives only the familiar error, ``the trait bound `fn(State<AppState>, Json<NewBook>)
-> ... {add_book}: Handler<_, ...>` is not satisfied``, at `.post(add_book)`. Add
`#[axum::debug_handler]` on the line above `async fn add_book`, and the real reason comes first:

```text
error: future cannot be sent between threads safely
  --> src/main.rs:27:1
   |
27 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ future returned by `add_book` is not `Send`
   |
   = help: within `impl Future<Output = (StatusCode, Json<Book>)>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Vec<Book>>`
note: future is not `Send` as this value is used across an await
  --> src/main.rs:33:52
   |
32 |     let mut books = state.books.lock().unwrap();
   |         --------- has type `std::sync::MutexGuard<'_, Vec<Book>>` which is not `Send`
33 |     tokio::time::sleep(Duration::from_millis(100)).await;
   |                                                    ^^^^^ await occurs here, with `mut books` maybe used later
…
```

Read the `note:`: the guard `books` is still alive at the `.await`. An `.await` is a pause: the
handler steps aside so its worker can serve other requests, and when it wakes up, it may continue on
a **different** worker thread. A standard `MutexGuard` must be unlocked by the same thread that
locked it, so it's not `Send`, and a handler that carries one across a pause can't be moved between
threads. Rust refuses it. That's the compiler protecting you: while this handler sleeps, holding the
lock, every other request that wants the books would wait too.

**Fix:** finish with the lock **before** the `.await`. Put the locked work in its own block, `{ … }`,
so the guard is dropped at the block's `}`, and do the slow part afterwards
(`cargo run -p shared-state --example state-short-lock`):

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-short-lock.rs:add_book}}
```

```bash
{{#include ../../code/topics/shared-state/http/70-short-lock.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/70-short-lock.out}}
```

The block's last line, `book` without a semicolon, is the block's value, so `let book = { … };`
takes the new book out of the block while the guard stays behind. By the time `.await` runs, the
lock is already open, and other requests can read the list while this one waits.

## More examples

Each example is a complete program in `code/topics/shared-state/examples/`. Stop any running server
with `Ctrl+C`, start the example with the command shown, and send its requests from your second
terminal.

### A visit counter, with no `Mutex`

For a single number, there's something lighter than a lock: an **atomic** integer, which can be
changed safely from many threads at once (`cargo run -p shared-state --example state-visit-counter`):

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-visit-counter.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-visit-counter.rs:state}}

{{#include ../../code/topics/shared-state/examples/state-visit-counter.rs:handler}}

{{#include ../../code/topics/shared-state/examples/state-visit-counter.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/50-visit-counter.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/50-visit-counter.out}}
```

`AtomicU64` is a `u64` whose changes happen in one indivisible step, done by the processor itself,
so two requests can never both read `1` and both write `2`. `fetch_add(1, …)` adds one and returns
the **old** value, hence the `+ 1`. `Ordering::Relaxed` is the simplest setting, enough for a
counter that doesn't coordinate with other data. There's still an `Arc`, because every request must
reach the same counter, but no `Mutex`, and no `.lock()`.

### Two fields in one state

A real state holds several things, each wrapped in what **it** needs
(`cargo run -p shared-state --example state-two-fields`):

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-two-fields.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-two-fields.rs:state}}

{{#include ../../code/topics/shared-state/examples/state-two-fields.rs:handlers}}

{{#include ../../code/topics/shared-state/examples/state-two-fields.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/51-two-fields.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/51-two-fields.out}}
```

The books need a `Mutex`; the counter is atomic. Every handler still takes the whole
`State<AppState>` and uses only the fields it needs. `.load(…)` reads an atomic without changing it.
The `POST` wasn't counted: only `list_books` adds to `visits`.

### `RwLock`: many readers, or one writer

Most requests only **read** the books. A `Mutex` makes readers wait for each other too; an
[`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html) (read-write lock) lets any number
of readers in together, and only makes everyone wait while someone writes
(`cargo run -p shared-state --example state-rwlock`):

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-rwlock.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-rwlock.rs:state}}

{{#include ../../code/topics/shared-state/examples/state-rwlock.rs:handlers}}
```

```bash
{{#include ../../code/topics/shared-state/http/52-rwlock.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/52-rwlock.out}}
```

The answers are the same as with a `Mutex`; the difference is only in who waits. `.read()` replaces
`.lock()` for readers, and `.write()` for the writer. For a lesson's tiny list, either is fine;
`RwLock` pays off when reads are many and slow enough to overlap.

### State that holds settings

State isn't only for data that changes. It's also where you put settings every handler needs, such
as the app's name (`cargo run -p shared-state --example state-config`):

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-config.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-config.rs:state}}

{{#include ../../code/topics/shared-state/examples/state-config.rs:handler}}

{{#include ../../code/topics/shared-state/examples/state-config.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/53-config.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/53-config.out}}
```

No `Mutex`, because nobody changes the name, and no `Arc`, because `&'static str` is a reference to
text built into the program: cloning it copies an address, as cheap as cloning an `Arc`. No
`Default` either: an empty name makes no sense, so `app()` builds the state by hand. When settings
come from a file or from environment variables (*Config and .env*, in *Part A4*), they're `String`s
made while the program runs; wrap them in an `Arc` so that each clone stays cheap.

## Your turn

Each solution is also a complete program in `code/topics/shared-state/examples/`, with a test
inside.

### 🟢 Guided

Add `GET /books/count`, which answers with the number of books, such as `2`. You need one new
handler, which locks the list and returns its `.len()` inside `Json`, and one new `.route(…)`.

<details><summary>Solution</summary>

`cargo run -p shared-state --example state-count`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-count.rs:count}}

{{#include ../../code/topics/shared-state/examples/state-count.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/90-count.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/90-count.out}}
```

`Json(books.len())` sends a bare number, which is valid JSON too, with
`content-type: application/json`. `.len()` gives a `usize`, and serde serializes it as a JSON number.
Only a lock is needed, no clone: the handler reads the length and lets go.

</details>

### 🟡 Tweak

Add `DELETE /books`, which empties the list and answers `204 No Content`. Hints: `Vec` has a
`.clear()` method; a handler can return a bare `StatusCode`; and `.delete(…)` chains onto the route
like `.post(…)`.

<details><summary>Solution</summary>

`cargo run -p shared-state --example state-clear`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-clear.rs:clear}}

{{#include ../../code/topics/shared-state/examples/state-clear.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/91-clear.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/91-clear.out}}
```

`204 No Content` means "done, and there's nothing to send back", so the answer has no body, and no
`content-type` or `content-length` either. The lock needs `mut`, because `.clear()` changes the
list. Notice that the next book you add gets id `1` again: counting the list is a simple way to make
ids, not a lasting one.

</details>

### 🔴 From scratch

Let readers like books. `POST /likes/{id}` adds one like to book `id` and answers
`{"book_id":1,"likes":2}` with its new total; `GET /likes/{id}` answers the same shape without
changing anything, with `0` likes for a book nobody has liked. Store the totals in the state as a
`HashMap<u32, u32>`, from book id to likes. Hints: `use std::collections::HashMap;`,
`map.entry(key).or_insert(0)` gives you the count to change, and `map.get(&key)` returns an
`Option`.

<details><summary>Solution</summary>

`cargo run -p shared-state --example state-likes`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-likes.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-likes.rs:types}}

{{#include ../../code/topics/shared-state/examples/state-likes.rs:handlers}}

{{#include ../../code/topics/shared-state/examples/state-likes.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/92-likes.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/92-likes.out}}
```

A `HashMap` is a lookup table: give it a key (the book id), and it finds the value (the likes).
`likes.entry(book_id).or_insert(0)` finds the count for that book, putting in a `0` first if there
isn't one, and hands back a **reference** to it, a way to change it in place: `*count += 1` adds one
to the number it points at (the `*` means "the value it points at"). In `get_likes`,
`.get(&book_id)` gives `Some(&count)` or `None`, `.copied()` turns `&count` into a plain number, and
`.unwrap_or(0)` turns `None` into `0`. Both handlers take `State` first and `Path` second: neither
reads the body, so their order is up to you.

</details>

## Quick check

<div class="quiz" data-topic="shared-state"></div>

## Remember this

- Hand the Router your state once with `.with_state(…)`, and take it in any handler with
  `State(state): State<AppState>`.
- `AppState` must be `Clone`: every request gets a copy. With `Arc` inside, the copy is a pointer,
  and every copy reaches the same data.
- `Arc` = many owners, across threads. `Mutex` = one at a time may change it. `Rc` won't compile
  here: it isn't `Send`.
- Lock, do the quick work, let go: never hold a `std::sync::Mutex` guard across an `.await`.
- State is memory, and memory dies with the process. Data that must survive a restart belongs in the
  database.

## Go deeper

- [Rust for Humans: Shared state: Arc and Mutex](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/shared-state-mutex-and-arc.html)
- [Rust for Humans: Smart pointers](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/smart-pointers.html)
- [axum::extract::State](https://docs.rs/axum/0.8.9/axum/extract/struct.State.html) — Official reference: the State extractor, with_state, and sharing state between handlers.

<!-- next:start -->

**Next:**

- [Error handling in Axum](../a2-axum/error-handling-in-axum.md)

<!-- next:end -->
