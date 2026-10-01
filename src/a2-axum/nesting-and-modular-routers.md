# Nesting and modular routers

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can split routes into modules, one file per area.
- You can mount a group of routes under a prefix with `nest`.
- You can combine routers with `merge`.

## What & why

Grow beyond one file: routers in modules, `merge` to combine them, `nest` to mount them under /api/books.

Picture a department store. Books are on the third floor, garden tools on the fourth. Each floor
looks after its own shelves: the books floor knows that crime novels are in aisle 7, and nobody in
the lobby needs to know that. The lobby has one map, and the map only says "Books: 3rd floor". If
the books department moves to the fifth floor, you change one line on the lobby map, and every aisle
number upstairs stays the same. A few things, such as the information desk, stand in the lobby
itself.

Every app in this book so far has lived in one file, `src/main.rs`, with one Router listing every
route. That works for six routes. A real API has fifty: books, authors, reviews, users, orders. One
file that long is hard to read, two people editing it at once get in each other's way, and every
route repeats its full address, `/api/books/…`, `/api/books/…`, `/api/books/…`.

This lesson builds the department store:

1. **One file per area, as a [module](../glossary.md#module).** `books.rs` holds the books
   handlers and a small Router of its own; `health.rs` holds a health check. Each file is a
   *module*, Rust's word for a named, separate part of a program. A module keeps its insides to
   itself, unless you mark something `pub` (public). Rust for Humans teaches modules in
   [Modules and crates](https://open-source-bd.github.io/rustbook-for-human/language-basics/modules-and-crates.html)
   and `pub` in
   [Visibility and privacy](https://open-source-bd.github.io/rustbook-for-human/language-basics/visibility-and-privacy.html);
   the one-line reminder is: **`mod books;` adds `books.rs` to the program, and only what's `pub`
   in it can be used from outside.**
2. **`nest`: put a whole Router under a prefix.** `.nest("/api/books", books::router())` is the
   lobby map's "Books: 3rd floor". The books Router says `/` and `/{id}`; `nest` adds `/api/books`
   in front of each, so the world sees `/api/books` and `/api/books/7`.
3. **`merge`: combine two Routers as they are.** `.merge(health::router())` is the information desk
   in the lobby: its route, `/health`, joins the main Router with its path unchanged.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/nesting-and-modular-routers`. Its `Cargo.toml` has nothing new:
splitting a program into files needs no extra crate.

```toml
{{#include ../../code/topics/nesting-and-modular-routers/Cargo.toml}}
```

### Line by line

`[package]` · `name = "nesting-and-modular-routers"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p nesting-and-modular-routers`.
- **How:** each lesson is its own small project, named after the lesson.
- **Remove it and…** (`name`) Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "take the edition and publish settings from the workspace's `code/Cargo.toml`".
- **Why:** every lesson shares `edition = "2024"` and `publish = false`.
- **How:** [Hello, Axum, Step 2](hello-axum.md#step-2-the-books-copy-of-the-same-project) explains
  `.workspace = true` in full.
- **Remove it and…** (`edition`) Cargo falls back to edition 2015, where `async fn` isn't allowed.

`[dependencies]` · `axum.workspace = true` · `tokio.workspace = true`
- **What:** Axum 0.8.9 and Tokio 1.53.1, as in every Axum lesson.
- **Why:** Axum gives `Router`, with its `nest` and `merge`; Tokio runs the server.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the test at the bottom of `src/main.rs`.
- **Why:** the test sends pretend requests to the Router and reads the answers, without opening a
  port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the same lines come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, and the test crates
from `cargo add --dev tower http-body-util --features tower/util`.

### Step 2: the files

This is the first lesson with more than one `.rs` file. Here is the project's folder:

```text
nesting-and-modular-routers/
├── Cargo.toml
└── src/
    ├── main.rs      the top: the mod lines, app() and main()
    ├── health.rs    the health module: GET /health
    └── books.rs     the books module: the list, and one book
```

### Line by line

`nesting-and-modular-routers/`
- **What:** the project's own folder, inside `code/topics/`.
- **Why:** one folder per lesson, as always.
- **How:** everything the project needs is inside it.
- **Remove it and…** there's no project.

`├── Cargo.toml`
- **What:** the file from Step 1.
- **Why:** it names the project and its dependencies.
- **How:** Cargo reads it first, then compiles the code in `src/`.
- **Remove it and…** Cargo doesn't know this folder is a project.

`└── src/`
- **What:** the folder that holds the Rust code.
- **Why:** Cargo looks for the code here, by convention.
- **How:** the files below are all inside it, side by side.
- **Remove it and…** Cargo has no code to compile.

`├── main.rs      the top: the mod lines, app() and main()`
- **What:** the program's starting file, the **crate root**: the top of the tree of modules.
- **Why:** Cargo compiles `main.rs`, and only the files that `main.rs` names, with `mod`. A
  `books.rs` sitting in `src/` that nobody names is ignored, as if it weren't there.
- **How:** Step 5 shows its `mod` lines, which pull in the two files below. Its `app()` puts their
  Routers together, as the lobby map does.
- **Remove it and…** there's no program: Cargo needs a `main.rs` to build one.

`├── health.rs    the health module: GET /health`
- **What:** a module called `health`, one small file with one handler and one Router.
- **Why:** a health check is an address that monitoring tools call to ask "are you alive?". It has
  nothing to do with books, so it gets its own file.
- **How:** the file's name **is** the module's name: `health.rs` is the module `health`.
- **Remove it and…** `mod health;` in `main.rs` fails, because the file it names is missing.

`└── books.rs     the books module: the list, and one book`
- **What:** a module called `books`, with everything about books.
- **Why:** when books get reviews, prices and authors, they all go here, and `main.rs` doesn't grow.
- **How:** same rule: `books.rs` is the module `books`.
- **Remove it and…** `mod books;` fails, as for `health`.

(The folder also has `examples/` and `http/`, for *More examples* and *Your turn*, as in earlier
lessons. They're left out of the tree to keep it short.)

### Step 3: the health module

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/src/health.rs:health}}
```

📁 Full code: code/topics/nesting-and-modular-routers · ▶ Run it: `cd code && cargo run -p nesting-and-modular-routers`

### Line by line

`use axum::{Router, routing::get};`
- **What:** brings `Router` and `get` into **this file**.
- **Why:** each module has its own list of names. A `use` line in `main.rs` does nothing for
  `health.rs`, so every file says what it uses.
- **How:** exactly the `use` line you've written since [Hello, Axum](hello-axum.md).
- **Remove it and…** ``cannot find type `Router` in this scope``, now pointing at `health.rs`.

`async fn health() -> &'static str {` · `"ok"` · `}`
- **What:** a handler that answers `ok`.
- **Why:** a real health check might also ask the database if it's awake; this one only shows the
  server is running.
- **How:** it has no `pub`, so it's **private** to the `health` module: `main.rs` can't call it, or
  even see it. Only `router` below can.
- **Remove it and…** `get(health)` below fails with ``cannot find value `health` in this scope``.

`pub fn router() -> Router {`
- **What:** a function that builds and returns this module's Router. `pub` makes it public: code
  outside the module may call it.
- **Why:** this is the one door into the module. `main.rs` asks for "the health Router" and gets it,
  without knowing which handlers are inside.
- **How:** `fn router()` is an ordinary function, not `async`: building a Router doesn't wait for
  anything. The name `router` is our choice; *Common mistakes* shows what happens without `pub`.
- **Remove it and…** (`pub`) ``error[E0603]: function `router` is private``, in `main.rs`.

`Router::new().route("/health", get(health))` · `}`
- **What:** a Router with one route: `GET /health` runs `health`.
- **Why:** it's a complete Router, the same kind you've built in every lesson, only smaller.
- **How:** `main.rs` will `merge` it, which keeps the path as it is: `/health`.
- **Remove it and…** (the `.route`) the Router is empty, and `/health` answers `404`.

### Step 4: the books module

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/src/books.rs:books}}
```

📁 Full code: code/topics/nesting-and-modular-routers · ▶ Run it: `cd code && cargo run -p nesting-and-modular-routers`

### Line by line

`use axum::{Router, extract::Path, routing::get};`
- **What:** `Router`, the `Path` [extractor](../glossary.md#extractor), and `get`.
- **Why:** `show_book` reads the `{id}` from the path.
- **How:** as in [Path and Query extractors](path-and-query-extractors.md).
- **Remove it and…** (`extract::Path`) ``cannot find tuple struct or tuple variant `Path` in this
  scope``.

`async fn list_books() -> &'static str {` · `"all books"` · `}`
- **What:** the handler for the list of books. It answers `all books`.
- **Why:** it stands in for a real list; in *Part A3*, it reads from the database.
- **How:** private, like `health`: only this module's Router uses it.
- **Remove it and…** `get(list_books)` below fails with ``cannot find value `list_books` in this
  scope``.

`async fn show_book(Path(id): Path<u32>) -> String {` · `format!("book {id}")` · `}`
- **What:** the handler for one book. For `/api/books/7`, it answers `book 7`.
- **Why:** the usual "one item from a collection" route.
- **How:** `Path<u32>` takes the `{id}` piece from the path and turns it into a number.
- **Remove it and…** `get(show_book)` below fails with ``cannot find value `show_book` in this
  scope``.

`pub fn router() -> Router {`
- **What:** this module's one door, as in `health.rs`.
- **Why:** `main.rs` needs the books Router, and nothing else from this file.
- **How:** two modules may each have a `router`. They don't clash, because outside their files they
  have full names: `health::router` and `books::router`. The module's name works like a surname.
- **Remove it and…** (`pub`) ``function `router` is private``, as in *Common mistakes*.

`Router::new()` · `.route("/", get(list_books))`
- **What:** `GET /` runs `list_books`.
- **Why:** `/` is "the start of **this** Router". This file doesn't say where it's mounted; `main.rs`
  decides that with `nest`. So `/` becomes `/api/books`.
- **How:** paths in a nested Router are **relative** to where it's mounted, like aisle numbers on a
  floor, which don't change when the department moves upstairs.
- **Remove it and…** `/api/books` answers `404`, and `/api/books/7` still works.

`.route("/{id}", get(show_book))` · `}`
- **What:** `GET /{id}` runs `show_book`.
- **Why:** mounted at `/api/books`, this becomes `/api/books/{id}`.
- **How:** `nest` adds the prefix in front of every route, path parameters included.
- **Remove it and…** `/api/books/7` answers `404`.

### Step 5: `main.rs` names its modules

Here's the first new idea in `main.rs`. It starts by naming the two files, then its one `use`
line:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/src/main.rs:modules}}

{{#include ../../code/topics/nesting-and-modular-routers/src/main.rs:imports}}
```

📁 Full code: code/topics/nesting-and-modular-routers · ▶ Run it: `cd code && cargo run -p nesting-and-modular-routers`

### Line by line

`mod books;`
- **What:** "this program has a module called `books`; its code is in `books.rs`, next to me".
- **Why:** without this line, Cargo never even opens `books.rs`.
- **How:** `mod` **declares** a module. From now on, `main.rs` can use what's public inside it, by
  its full name: `books::router()`. That's different from `use`, which only makes a long name
  shorter (*You might be wondering…* has more).
- **Remove it and…** ``error[E0433]: cannot find module or crate `books` in this scope``, and the
  compiler suggests the fix (*Common mistakes* shows it).

`mod health;`
- **What:** the same, for `health.rs`.
- **Why:** `main.rs` needs `health::router()`.
- **How:** the order of `mod` lines doesn't matter. `cargo fmt` sorts them alphabetically.
- **Remove it and…** the same `cannot find module or crate` error, for `health`.

`use axum::Router;`
- **What:** brings `Router` into `main.rs`.
- **Why:** `fn app() -> Router` names the type. The handlers and `get` live in the modules now, so
  `main.rs` needs nothing else from Axum.
- **How:** a one-name `use`, without the braces.
- **Remove it and…** ``cannot find type `Router` in this scope``, in `main.rs`.

### Step 6: the lobby map, with `merge` and `nest`

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/src/main.rs:app}}
```

📁 Full code: code/topics/nesting-and-modular-routers · ▶ Run it: `cd code && cargo run -p nesting-and-modular-routers`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the whole app's Router, starting from an empty one.
- **Why:** as in every lesson, a separate function, so `main` and the test use the same Router.
- **How:** this Router has no routes of its own. Everything comes from the modules.
- **Remove it and…** there's nothing to serve.

`.merge(health::router())`
- **What:** adds every route of the health Router to this one, **with its paths unchanged**:
  `/health` stays `/health`.
- **Why:** a health check belongs at the top level, where monitoring tools expect it, not under
  `/api/books`.
- **How:** `merge` ("join together") copies the other Router's routes in. If both Routers had the
  same method on the same path, Axum would panic when the app starts (*Common mistakes*).
- **Remove it and…** `/health` answers `404`; the books routes still work.

`.nest("/api/books", books::router())` · `}`
- **What:** mounts the books Router **under** `/api/books`: its `/` becomes `/api/books`, and its
  `/{id}` becomes `/api/books/{id}`.
- **Why:** the prefix is written **once**, here. Every books route gets it, and moving them all to
  `/api/v2/books` one day is a one-line change.
- **How:** `nest` ("put inside") glues the prefix in front of each of the inner Router's paths. The
  prefix starts with `/` and has no `/` at the end; *Common mistakes* shows what a trailing `/`
  does.
- **Remove it and…** `/api/books` and `/api/books/7` answer `404`; `/health` still works.

### Step 7: `main`

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/src/main.rs:main}}
```

📁 Full code: code/topics/nesting-and-modular-routers · ▶ Run it: `cd code && cargo run -p nesting-and-modular-routers`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's start, running on the Tokio runtime.
- **Why:** `main` needs `.await`, and `#[tokio::main]` is what allows that.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line of `main`.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`let listener = …bind("127.0.0.1:3000")…;` · `println!(…);` · `axum::serve(listener, app())…;`
- **What:** claims port 3000, prints `Listening on http://127.0.0.1:3000`, and answers requests
  until you stop the server.
- **Why:** a server must own a port before anyone can reach it.
- **How:** word for word the `main` of every Axum lesson. Splitting the routes into modules changed
  `app()`, and nothing here.
- **Remove it and…** there's no port for curl to reach.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p nesting-and-modular-routers
```

In the second terminal, ask for the health check, the books list, one book, and two near misses:

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/01-routes.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/01-routes.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`/health`: `ok`.** The merged route, at its own path.
- **`/api/books`: `all books`.** The books Router's `/`, with the prefix in front.
- **`/api/books/7`: `book 7`.** Its `/{id}`, with the prefix in front.
- **`/books`: `404`.** The books routes live **only** under `/api/books` now. `nest` doesn't keep a
  copy at the old address.
- **`/api/books/`, with a slash at the end: `404`.** To Axum, `/api/books` and `/api/books/` are
  two different paths, and only the first one is a route. *You might be wondering…* explains
  what to do if you want both to work.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/health`, `/api/books` and `/api/books/7` answer `ok`, `all books` and `book 7`, your
  three files work as one program.
- ✅ If `/books` and `/api/books/` answer `404`, you've seen that paths match exactly.
- ❌ If the build fails with `function router is private` or `cannot find module or crate`, see
  *Common mistakes*: a missing `pub`, or a missing `mod` line.
- ❌ If `/api/books` answers `404` but `/api/books/` works, check the `nest` line for a slash at the
  end of `"/api/books"`.

## You might be wondering…

**"`merge` or `nest`: which one, when?"**
Ask "should this Router's paths change?". `merge` keeps them: use it for routes that already have
their final address, such as `/health`, or to combine two Routers that each wrote full paths. `nest`
adds a prefix: use it for a group that shares a beginning, such as everything under `/api/books`.
Most apps use both: a few `merge`s for top-level routes, and one `nest` per area of the API.

**"Why is the books route `/`, and not `/api/books`?"**
Because `books.rs` shouldn't need to know where it's mounted. `main.rs` decides the address, once,
in its `nest` line; `books.rs` only knows its own aisles: `/` for the list, `/{id}` for one book.
If `books.rs` said `.route("/api/books", …)` **and** was nested at `/api/books`, the real address
would be `/api/books/api/books`: the prefix, then the route's own path.

**"What about the trailing slash?"**
Axum matches paths **exactly**, so `/api/books/` is a different path from `/api/books`, and gets
`404`, as *Run it* showed. Some frameworks quietly accept both; Axum doesn't guess. If you want
both to work, you have two good choices. The first: add one more route for the slashed path, on the
outer Router, that **redirects** to the real one; *More examples* has it. The second: tower-http
has a `NormalizePathLayer` that removes the trailing slash from every request before routing. It has
to wrap the whole app from the outside (a layer added with `.layer` runs after the Router has chosen
a route, which is too late), so it takes a few more lines; the tower-http documentation shows how.
For an API that programs call, plain `404` is usually fine: programs use the exact address they're
given.

**"How do nested Routers share state?"**
All of them use the **same** state type, and `.with_state(…)` is called **once**, on the outer
Router, after the `merge`s and `nest`s. Each module's `router` function returns
`Router<AppState>`, "a Router that still needs an `AppState`", exactly like the Router before
`.with_state` in [Shared state](shared-state.md#step-6-the-router-with-with_state). *More examples*
has the complete program.

**"What's the difference between `mod` and `use`?"**
`mod books;` **adds** a module to the program: it's the line that makes Cargo compile `books.rs`.
You write it once, in the file that owns the module (here, `main.rs`). `use` only **shortens a
name** that already exists: after `use axum::Router;` you can write `Router` instead of
`axum::Router`. `main.rs` calls `books::router()` by its full name, so it needs a `mod` line but no
`use` line. You could write `use books::router;` too, but with two functions called `router`, the
full names read better.

**"Can a module have modules inside it?"**
Yes. When `books.rs` grows, it can say `mod reviews;`, and the file goes in a folder named after the
module: `src/books/reviews.rs`. Its handlers are then `books::reviews::…`. Rust for Humans'
[Modules and crates](https://open-source-bd.github.io/rustbook-for-human/language-basics/modules-and-crates.html)
shows the folder layouts. One level is plenty for this lesson.

## Coming from another language?

Every web framework can group routes under a prefix. The two differences to watch: how a file
becomes part of the program, and what happens to a trailing slash.

**Express (Node.js).** An `express.Router()` in its own file, mounted with `app.use`:

```js
// books.js
const express = require("express");
const router = express.Router();

router.get("/", (req, res) => res.send("all books"));
router.get("/:id", (req, res) => res.send(`book ${req.params.id}`));

module.exports = router;

// app.js
const books = require("./books");
app.use("/api/books", books);
```

`app.use("/api/books", books)` is `.nest("/api/books", books::router())`, and `app.use(health)`,
with no path, is `.merge(health::router())`. `module.exports` plays the part of `pub`: anything not
exported stays inside the file. `require("./books")` is close to `mod books;`. One difference:
Express's routers aren't strict about slashes by default, so `/api/books/` reaches the same handler
as `/api/books`. In Axum it's a `404`.

**Flask and FastAPI (Python).** Flask calls a group of routes a *blueprint*:

```python
# books.py
from flask import Blueprint

bp = Blueprint("books", __name__)

@bp.get("/")
def list_books():
    return "all books"

@bp.get("/<int:id>")
def show_book(id):
    return f"book {id}"

# app.py
app.register_blueprint(books.bp, url_prefix="/api/books")
```

FastAPI calls it a router:

```python
# books.py
from fastapi import APIRouter

router = APIRouter()

@router.get("/{id}")
def show_book(id: int):
    return f"book {id}"

# main.py
app.include_router(books.router, prefix="/api/books")
```

`url_prefix=` and `prefix=` are `nest`'s first argument. Python has no `pub`: every name in a module
can be imported, and a leading underscore (`_helper`) is only a polite "please don't". Both
frameworks also handle the trailing slash for you, with a redirect from one form to the other.

**Spring Boot (Java).** Each controller class carries its own prefix:

```java
@RestController
@RequestMapping("/api/books")
class BookController {
    @GetMapping
    String list() { return "all books"; }

    @GetMapping("/{id}")
    String show(@PathVariable int id) { return "book " + id; }
}
```

`@RequestMapping("/api/books")` on the class is the prefix, and the methods' paths are relative to
it, like `books.rs`. The big difference: Spring finds controllers by itself, by scanning your
packages, so there's no `mod` line and no `app()` listing them; in Axum, nothing joins the app
unless `main.rs` names it. Since Spring Boot 3, `/api/books/` no longer matches `/api/books` by
default, the same as Axum.

**Go (`net/http` and Gin).** Gin calls it a group:

```go
books := r.Group("/api/books")
books.GET("", listBooks)
books.GET("/:id", showBook)
```

`r.Group("/api/books")` is `nest`. Go's rule for `pub` is the first letter of a name: `Router()` is
exported (public), `router()` isn't, and a folder is a package, Go's nearest thing to a module.
Gin redirects a trailing slash by default. The standard library's `http.ServeMux` has no groups:
you write the full path in each pattern, or wrap a second mux in `http.StripPrefix`.

## Common mistakes

The compiler errors and panics below were captured in a scratch copy of this lesson's code, a
project called `my-routers`, so their line numbers are from that copy.

**Forgetting `pub` on `router`.**

```rust,noplayground,ignore
fn router() -> Router {
    Router::new()
        .route("/", get(list_books))
        .route("/{id}", get(show_book))
}
```

```text
error[E0603]: function `router` is private
  --> src/main.rs:14:36
   |
14 |         .nest("/api/books", books::router())
   |                                    ^^^^^^ private function
   |
note: the function `router` is defined here
  --> src/books.rs:12:1
   |
12 | fn router() -> Router {
   | ^^^^^^^^^^^^^^^^^^^^^
```

The error points at `main.rs`, where `router` is **called**, and the `note:` points at `books.rs`,
where it's **defined**. Everything in a module is private unless marked `pub`, so `main.rs`, which is
a different module, may not call it. **Fix:** `pub fn router() -> Router`. Only the door needs `pub`;
the handlers behind it stay private.

**Forgetting `mod books;`.**

```rust,noplayground,ignore
mod health;
```

```text
error[E0433]: cannot find module or crate `books` in this scope
  --> src/main.rs:13:29
   |
13 |         .nest("/api/books", books::router())
   |                             ^^^^^ use of unresolved module or unlinked crate `books`
   |
help: to make use of source file src/books.rs, use `mod books` in this file to declare the module
   |
 2 + mod books;
   |
```

`books.rs` is right there in `src/`, but without a `mod` line, Cargo never compiled it, and `books`
means nothing. The `help:` sees the file and says exactly what to add. **Fix:** `mod books;` at the
top of `main.rs`.

**Nesting at `"/api/books/"`, with a slash at the end.**
Here's the same app with one extra character
(`cargo run -p nesting-and-modular-routers --example nest-slash-mistake`):

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-slash-mistake.rs:app}}
```

It builds, with no warning, and the server starts. Now ask for the list both ways, and for one book:

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/70-slash-mistake.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/70-slash-mistake.out}}
```

The list moved: `/api/books` is now `404`, and `/api/books/` (with the slash) answers. `nest` glued
`"/api/books/"` in front of the inner `/`, and the slash at the end of the prefix stayed. `/api/books/7`
still works, which makes this easy to miss: half your routes look fine. **Fix:** write the prefix
with no slash at the end, `"/api/books"`, as in Step 6.

**Nesting at `"/"`.**
Nesting at the very top, `.nest("/", books::router())`, compiles, but the app panics as soon as
`main` builds it:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s
     Running `target/debug/my-routers`
Listening on http://127.0.0.1:3000

thread 'main' (16290077) panicked at src/main.rs:14:10:
Nesting at the root is no longer supported. Use merge instead.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(`Listening…` appears first because `main` prints it before calling `app()`. The number after
`'main'` is an ID for the thread, and is different every run.) A prefix of `/` adds nothing, so
Axum tells you to say what you mean. **Fix:** `.merge(books::router())`. Forgetting the first `/`,
`.nest("api/books", …)`, panics too, with ``assertion failed: path.starts_with('/')``.

**Two Routers with the same route.**
If two Routers you `merge` both have `GET /health` (here, the same `.merge(health::router())` line
written twice), the app also panics at startup:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.81s
     Running `target/debug/my-routers`
Listening on http://127.0.0.1:3000

thread 'main' (16292424) panicked at src/main.rs:14:10:
Overlapping method route. Handler for `GET /health` already exists
```

Axum refuses to choose between two handlers for one address. It's a panic, not a compiler error,
because paths are only text to the compiler: Axum checks them when the Router is built. That's
also why every lesson's test calls `app()`: a test that builds the Router finds this panic before
the server ever runs. **Fix:** keep each path in one module. With `nest`, two areas can't collide,
because their prefixes differ.

## More examples

Each example is a complete program in `code/topics/nesting-and-modular-routers/examples/`. Stop
any running server with `Ctrl+C`, start the example with the command shown, and send its requests
from your second terminal.

### Nested Routers sharing one `AppState`

Two modules, one state: the books module counts how many times a book was looked up, and the health
check reports the count. The example is a folder, `examples/nest-shared-state/`, with its own
`main.rs`, `health.rs` and `books.rs`
(`cargo run -p nesting-and-modular-routers --example nest-shared-state`). First `main.rs`:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/main.rs:modules}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/main.rs:imports}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/main.rs:state}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/main.rs:app}}
```

Then the two modules:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/health.rs:health}}
```

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-shared-state/books.rs:books}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/50-shared-state.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/50-shared-state.out}}
```

The health check counted the two lookups that the books module made: one state, two modules.
Three things make it work:

- **`Router<AppState>`.** Both `router` functions return a Router that still **needs** an
  `AppState`. Write plain `Router` there, and the build fails with
  ``expected `MethodRouter`, found `MethodRouter<AppState>` ``: a handler that asks for
  `State<AppState>` doesn't fit a Router that has no state.
- **`.with_state(…)` once, at the top**, after the `merge` and the `nest`. That one `AppState`
  reaches every handler, in every module. If each module called `.with_state(AppState::default())`
  itself, each would get its **own** counter, and the health check would always say `0`.
- **`use crate::AppState;`.** `crate` means "the top of this program", `main.rs`, where `AppState`
  is defined. `AppState` has no `pub`, and still the modules can use it: a module can always see
  the private items of the modules **above** it. (The counter is an `AtomicU64`, as in
  [Shared state's visit counter](shared-state.md#a-visit-counter-with-no-mutex).)

### Two levels: `/api`, then `/v1` and `/v2`

A nested Router can nest Routers of its own. Here, an API with two versions, side by side
(`cargo run -p nesting-and-modular-routers --example nest-two-levels`):

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-two-levels.rs:handlers}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-two-levels.rs:app}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/51-two-levels.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/51-two-levels.out}}
```

The prefixes add up from the outside in: `/api`, then `/v1`, then the route's own `/books/{id}`.
`/v1/books/7`, without `/api`, is `404`. Versions are the classic reason for this shape: when
version 2 changes what a book looks like, programs written for version 1 keep calling `/api/v1/…`
and keep working. (The example keeps all the Routers in one file, to show only the nesting; in a
real app, each version would be its own module.)

### A redirect for the trailing slash

If you'd like `/api/books/` to work, send it to the real address
(`cargo run -p nesting-and-modular-routers --example nest-trailing-redirect`):

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-trailing-redirect.rs:imports}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-trailing-redirect.rs:redirect}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-trailing-redirect.rs:app}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/52-trailing-redirect.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/52-trailing-redirect.out}}
```

`/api/books/` now answers `308 Permanent Redirect`, with `location: /api/books`. With `-L`, curl
follows it, as in
[Handlers and IntoResponse](handlers-and-into-response.md#redirectto-go-to-another-address), and
gets `all books`. The extra route lives on the **outer** Router, next to the `nest`: the books
Router can't say "my prefix, with a slash", because its `/` already means `/api/books`. The two
don't clash, because `/api/books/` and `/api/books` are different paths. `Redirect::permanent`
sends `308`, which tells browsers to remember the move, and to send a `POST` again as a `POST`.

### A layer on one nested Router only

A Router can have its own layers before it's nested, and they wrap **its** routes only
(`cargo run -p nesting-and-modular-routers --example nest-layer-one-router`):

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-layer-one-router.rs:middleware}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-layer-one-router.rs:app}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/53-layer-one-router.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/53-layer-one-router.out}}
```

`add_area` is a middleware with the same shape as `add_powered_by` in
[Middleware and Tower layers](middleware-and-tower-layers.md#step-5-your-own-middleware). The books
answer carries `x-area: books`; `/health`, on the outer Router, doesn't. So does
`/api/books/7/extra`, a path under the prefix that matches no books route: the books Router only
owns the routes it lists, and for anything else, the **outer** Router answers with its own `404`,
outside the books layer. In that way, a nested Router's `.layer` works like the `route_layer` from
that lesson: its routes get it, and the `404` doesn't. It's how you'd protect a whole area, such as everything under `/api/admin`, with the
key check from that lesson's *More examples*, and leave the rest of the app open.

## Your turn

Each solution is also a complete program, with a test inside. The first two are folders in
`code/topics/nesting-and-modular-routers/examples/`.

### 🟢 Guided

Add a route for a book's reviews: `GET /api/books/7/reviews` answers `reviews of book 7`. Change
**only** `books.rs`: one handler, and one `.route`. Which path do you write in the `.route`, given
that the Router is nested at `/api/books`?

<details><summary>Solution</summary>

`cargo run -p nesting-and-modular-routers --example nest-reviews`:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-reviews/books.rs:books}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/90-reviews.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/90-reviews.out}}
```

The path is `/{id}/reviews`, relative to the mount point, so `nest` turns it into
`/api/books/{id}/reviews`. `main.rs` didn't change at all: that's the point of one module per area.

</details>

### 🟡 Tweak

Add a third module, `authors.rs`, with `GET /` answering `all authors` and `GET /{id}` answering
`author 3` for 3, and mount it at `/api/authors`. You need a new file, a new `mod` line, and one
more line in `app()`.

<details><summary>Solution</summary>

`cargo run -p nesting-and-modular-routers --example nest-authors`. The new file, `authors.rs`:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-authors/authors.rs:authors}}
```

And `main.rs`, with one new `mod` line and one new `nest`:

```rust,noplayground
{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-authors/main.rs:modules}}

{{#include ../../code/topics/nesting-and-modular-routers/examples/nest-authors/main.rs:app}}
```

```bash
{{#include ../../code/topics/nesting-and-modular-routers/http/91-authors.sh}}
```

```text
{{#include ../../code/topics/nesting-and-modular-routers/http/91-authors.out}}
```

`authors.rs` is `books.rs` with the words changed. Now there are three functions called `router`,
in three modules, and `main.rs` tells them apart by their full names.

</details>

### 🔴 From scratch

Take the app from [Shared state](shared-state.md), the book list with `GET /books` and
`POST /books`, and split it into modules: `state.rs` for `AppState`, and `books.rs` for `Book`,
`NewBook`, the two handlers and a `router()` that returns `Router<AppState>`. `main.rs` keeps only
the `mod` lines, `app()` (which nests the books Router at `/books` and calls `.with_state` once) and
`main`. Copy the lesson's two tests into your `main.rs`, unchanged: they must still pass. Hint: the
`books` module reads `AppState`'s field, and `state.rs` names the `Book` type, so look carefully at
which items need `pub`.

<details><summary>Solution</summary>

This solution uses serde, so it lives in the shared-state lesson's project, as the folder
`code/topics/shared-state/examples/state-split-modules/`
(`cargo run -p shared-state --example state-split-modules`). First `state.rs`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-split-modules/state.rs:state}}
```

Then `books.rs`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-split-modules/books.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-split-modules/books.rs:types}}

{{#include ../../code/topics/shared-state/examples/state-split-modules/books.rs:handlers}}

{{#include ../../code/topics/shared-state/examples/state-split-modules/books.rs:router}}
```

And `main.rs`:

```rust,noplayground
{{#include ../../code/topics/shared-state/examples/state-split-modules/main.rs:modules}}

{{#include ../../code/topics/shared-state/examples/state-split-modules/main.rs:imports}}

{{#include ../../code/topics/shared-state/examples/state-split-modules/main.rs:app}}
```

```bash
{{#include ../../code/topics/shared-state/http/93-split-modules.sh}}
```

```text
{{#include ../../code/topics/shared-state/http/93-split-modules.out}}
```

The same answers as in Shared state. And the lesson's two tests, copied unchanged into the bottom
of `main.rs`, still pass (`cargo test -p shared-state --example state-split-modules`):

```text
   Compiling shared-state v0.1.0 (/Users/you/rust-book-backend/code/topics/shared-state)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.96s
     Running unittests examples/state-split-modules/main.rs (target/debug/examples/state_split_modules-beab91ecf71447e9)

running 2 tests
test tests::a_new_app_starts_empty ... ok
test tests::added_books_are_remembered_by_the_same_app ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The `pub`s are the interesting part:

- **`pub struct AppState` and `pub books`.** `AppState` now lives in `state.rs`, a **sibling** of
  `books.rs`, not above it, so it must be public, and so must its field: the handlers in `books.rs`
  read `state.books`. Without the field's `pub`, the build stops with
  ``error[E0616]: field `books` of struct `AppState` is private``, once for each handler.
- **`pub struct Book`, with private fields.** `state.rs` names the type (`Vec<Book>`), so the type
  must be public. But only `books.rs` creates a `Book` or reads its fields, so `id` and `title` stay
  private. Make public only what another module really uses.
- **`NewBook` and the handlers stay private.** Nothing outside `books.rs` needs them.

`use crate::books::Book;` and `use crate::state::AppState;` name the types by their path from the
top of the program; each module points at the other, and Rust is fine with that. In `main.rs`,
`use state::AppState;` lets `app()` write `AppState::default()`. The route is `.nest("/books", …)`
with the books Router's `/`, so the address is still `/books`, which is what the tests ask for.
(`.merge`, with the route written as `/books` in `books.rs`, would work too.)

</details>

## Quick check

<div class="quiz" data-topic="nesting-and-modular-routers"></div>

## Remember this

- One file per area: `mod books;` in `main.rs` adds `books.rs` to the program, and only items
  marked `pub` (such as `pub fn router()`) can be used from outside it.
- `.nest("/api/books", books::router())` puts every books route under the prefix. Inside
  `books.rs`, paths are relative: `/` is `/api/books`, `/{id}` is `/api/books/{id}`. Write the
  prefix with a leading `/` and no trailing one.
- `.merge(other)` combines Routers with their paths unchanged. Two Routers with the same route
  panic when the app starts.
- Paths match exactly: `/api/books/` is not `/api/books`. Add a redirect if you want both.
- Nested Routers share state by all returning `Router<AppState>`; call `.with_state` once, at the
  top.

## Go deeper

- [Rust for Humans: Modules and crates](https://open-source-bd.github.io/rustbook-for-human/language-basics/modules-and-crates.html)
- [Rust for Humans: Visibility and privacy](https://open-source-bd.github.io/rustbook-for-human/language-basics/visibility-and-privacy.html)
- [axum::Router::nest](https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.nest) — Official reference: how nest adds a prefix, and what it does with paths and fallbacks.

<!-- next:start -->

**Next:**

- [Custom extractors](../a2-axum/custom-extractors.md)

<!-- next:end -->
