# Routes and HTTP methods

> **Beginner** · Part A2 · Axum

## By the end of this lesson

- You can send different HTTP methods on one path to different handlers.
- You know what 404 and 405 mean and when Axum sends each.
- You can give unknown paths your own fallback response.

## What & why

One path, many methods: GET, POST, PUT and DELETE on the same URL, plus 404, 405 and a custom fallback.

Picture the desk of a small library. There's **one counter**, and you hand the librarian a form:
a *borrow* form, a *return* form, or a *renew* form. Same counter, same librarian, three different
jobs, and the form decides which one happens. If you ask for a counter the library doesn't have
("the magazine desk, please"), you're told it doesn't exist. If you hand the right counter a form
it doesn't take ("I'd like to *sell* this book"), you're told *this counter doesn't do that*, and
shown the list of forms it does take.

A web server works the same way. The **path** is the counter, and the
[**HTTP method**](../glossary.md#http-method) is the form: the verb at the start of every request
(`GET`, `POST`, `PUT`, `DELETE`…) that says what you want done. In
[How a web backend works](../part-0-start/how-a-web-backend-works.md#step-6-rest-a-naming-plan-for-endpoints)
you saw the REST plan: **two paths, four methods**. In [Hello, Axum](hello-axum.md) every route
answered only `GET`. This lesson builds the whole plan for the library's books, in Axum, and shows
the two ways a request can miss:

- **a path nobody handles** (the counter that doesn't exist): `404 Not Found`;
- **a path that exists, with a method it doesn't take** (the wrong form): `405 Method Not Allowed`.

You'll also give the first kind your own answer, with a **fallback**: a handler that runs for every
request whose path matched no route.

## The idea, slowly

### Step 1: the plan

Here are the six things this server does. Each row is a method and a path, and the name after the
arrow is the handler that answers it:

```text
GET     /books         list every book      →  list_books
POST    /books         add a book           →  add_book
GET     /books/{id}    show one book        →  show_book
PUT     /books/{id}    replace one book     →  update_book
DELETE  /books/{id}    delete one book      →  delete_book
any other path                              →  not_found
```

### Line by line

`GET /books → list_books`
- **What:** a `GET` to the collection `/books` lists every book.
- **Why:** `GET` means "give me something, don't change anything", so it's the method for reading.
- **How:** `/books` names the whole collection, as `/products` did in How a web backend works.
- **Remove it and…** there's no way to see the list.

`POST /books → add_book`
- **What:** a `POST` to the **same** path adds a new book.
- **Why:** a new book has no id yet, so you send it to the collection, and the server picks one.
- **How:** same path as the row above, different method, different handler. This is the heart of
  the lesson.
- **Remove it and…** `POST /books` gets `405 Method Not Allowed`: the path exists, but only for
  `GET`.

`GET /books/{id} → show_book`
- **What:** shows one book, such as `GET /books/7`.
- **Why:** a book's page needs only that book.
- **How:** `{id}` is a **placeholder**: it stands for any one path segment (one piece between
  slashes), so `/books/7` and `/books/42` both match it.
- **Remove it and…** `GET /books/7` gets `405`, because `PUT` and `DELETE` still use that path.

`PUT /books/{id} → update_book`
- **What:** replaces book `{id}` with a new version.
- **Why:** to correct a title or an author.
- **How:** the path says *which* book; the method says *replace it*.
- **Remove it and…** books can never change.

`DELETE /books/{id} → delete_book`
- **What:** deletes book `{id}`.
- **Why:** the library gives a book away.
- **How:** no body is needed: the path already says which book.
- **Remove it and…** books can never be removed.

`any other path → not_found`
- **What:** every path that matches none of the rows above, such as `/magazines`, goes to
  `not_found`.
- **Why:** so an unknown path gets a helpful answer instead of an empty one.
- **How:** this is the fallback. It's about the **path** only: a known path with the wrong method
  doesn't come here (Step 7 shows why).
- **Remove it and…** Axum answers unknown paths itself, with the empty `404` from Hello, Axum.

The handlers in this lesson don't store any books yet: each one answers with a fixed sentence, so
you can see which handler ran. Keeping real data comes later, in [Shared state](shared-state.md),
and in a database in Part A3.

### Step 2: the crate's `Cargo.toml`

The code lives in `code/topics/routes-and-methods`. Its `Cargo.toml` is the same as Hello, Axum's,
with a different name:

```toml
{{#include ../../code/topics/routes-and-methods/Cargo.toml}}
```

### Line by line

`[package]` · `name = "routes-and-methods"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p routes-and-methods`.
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
- **Why:** the same two crates every Axum server needs.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router (with tower's `oneshot`) and read the
  response bodies (with `http-body-util`), without opening a port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project (such as `my-first-server` from Hello, Axum), the same dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, and the test crates
from `cargo add --dev tower http-body-util --features tower/util`.

### Step 3: the `use` line

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/src/main.rs:imports}}
```

📁 Full code: code/topics/routes-and-methods · ▶ Run it: `cd code && cargo run -p routes-and-methods`

### Line by line

`use axum::{…};`
- **What:** brings three names from Axum into this file.
- **Why:** so the code can write `StatusCode` instead of `axum::http::StatusCode`.
- **How:** the braces list several names from the same crate, as in Hello, Axum.
- **Remove it and…** all three names are unknown, and the build stops with many errors.

`Router`
- **What:** Axum's table of [routes](../glossary.md#route).
- **Why:** Step 5 builds one.
- **How:** the same `Router` as in Hello, Axum.
- **Remove it and…** ``cannot find type `Router` in this scope``.

`http::StatusCode`
- **What:** the list of every HTTP [status code](../glossary.md#status-code), with a name for each,
  such as `StatusCode::CREATED` for `201`.
- **Why:** `add_book`, `delete_book` and `not_found` choose their own status code, instead of the
  usual `200 OK`.
- **How:** it lives in Axum's `http` module. You met it in Tour of the stack's
  [More examples](../part-0-start/tour-of-the-stack.md#choose-the-status-code-statuscodecreated-made).
- **Remove it and…** seven errors: six ``cannot find type `StatusCode` in this scope``, one for
  each place it's used, and one knock-on error saying `delete_book` isn't a valid handler.

`routing::get`
- **What:** the function that starts a route's list of methods with `GET`.
- **Why:** both routes in Step 5 start with `get(…)`.
- **How:** notice there's **no** `post`, `put` or `delete` here, although Step 5 uses all three.
  Step 5 explains why: they're written *after* a dot, as `.post(…)`, on what `get(…)` returns, and
  a name after a dot doesn't need a `use`.
- **Remove it and…** ``cannot find function `get` in this scope``.

### Step 4: six handlers

One handler per job from Step 1, plus the fallback's:

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/src/main.rs:handlers}}
```

📁 Full code: code/topics/routes-and-methods · ▶ Run it: `cd code && cargo run -p routes-and-methods`

### Line by line

`async fn list_books() -> &'static str {` · `"all books"`
- **What:** the handler for `GET /books`. It answers with the text `all books`.
- **Why:** a stand-in for the real list, so you can see which handler ran.
- **How:** a `&'static str` goes out as `200 OK` with `content-type: text/plain`, exactly like
  `about` in Hello, Axum.
- **Remove it and…** Step 5's `get(list_books)` points at nothing:
  ``cannot find value `list_books` in this scope``.

`async fn add_book() -> (StatusCode, &'static str) {` · `(StatusCode::CREATED, "book added")`
- **What:** the handler for `POST /books`. It answers `201 Created`, with the text `book added`.
- **Why:** `201` is the status code for "a new thing was made", which is what a `POST` to a
  collection does.
- **How:** the return value is a *tuple*, a pair in brackets: the status code first, then the body.
  Axum uses your status instead of `200`. (How Axum turns a tuple into a response is the topic of
  the next lesson, [Handlers and IntoResponse](handlers-and-into-response.md).)
- **Remove it and…** (the `StatusCode::CREATED` part, returning only `"book added"`) the book is
  "added" with `200 OK`, which works, but tells the client less.

`async fn show_book() -> &'static str {` · `"one book"`
- **What:** the handler for `GET /books/{id}`.
- **Why:** a stand-in for "the book with this id".
- **How:** it doesn't read the id yet. Reading the value of `{id}` needs an
  [*extractor*](../glossary.md#extractor), which the lesson
  [Path and Query extractors](path-and-query-extractors.md) covers. Here, the point is only *which
  handler runs*.
- **Remove it and…** ``cannot find value `show_book` in this scope``.

`async fn update_book() -> &'static str {` · `"book updated"`
- **What:** the handler for `PUT /books/{id}`.
- **Why:** it confirms the replace happened.
- **How:** `200 OK` and plain text, like `list_books`.
- **Remove it and…** ``cannot find value `update_book` in this scope``.

`async fn delete_book() -> StatusCode {` · `StatusCode::NO_CONTENT`
- **What:** the handler for `DELETE /books/{id}`. It returns **only** a status code, `204 No
  Content`, and no text at all.
- **Why:** after a delete there's nothing left to show. `204` means "it worked, and there's no
  body", the code How a web backend works gave for a `DELETE`.
- **How:** a `StatusCode` on its own is a complete response: that status, with an empty body.
- **Remove it and…** (returning `"book deleted"` instead) the client gets `200 OK` and a sentence,
  which works, but isn't the usual answer to a `DELETE`.

`async fn not_found() -> (StatusCode, &'static str) {` · `(StatusCode::NOT_FOUND, "nothing lives here")`
- **What:** the fallback's handler: `404 Not Found`, with the text `nothing lives here`.
- **Why:** Axum's own 404 has an empty body. A short sentence tells a human what happened.
- **How:** the same tuple as `add_book`, with a different status. A fallback handler is an ordinary
  handler; only the way it's attached (Step 5) is different.
- **Remove it and…** ``cannot find value `not_found` in this scope`` in Step 5.

### Step 5: one Router, two paths, five methods

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/src/main.rs:app}}
```

📁 Full code: code/topics/routes-and-methods · ▶ Run it: `cd code && cargo run -p routes-and-methods`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router, starting from an empty one.
- **Why:** a separate function, so `main` and the tests use the same Router.
- **How:** exactly as in Hello, Axum: each call below returns the Router with one more rule.
- **Remove it and…** there's no Router to add routes to.

`.route("/books", get(list_books).post(add_book))`
- **What:** one route for the path `/books` that answers **two** methods: `GET` runs `list_books`,
  `POST` runs `add_book`.
- **Why:** one path, many methods is the REST plan from Step 1. The path is written once, and each
  method gets its own handler.
- **How:** this is **method chaining**. `get(list_books)` makes a small table for one path, which
  Axum calls a *method router*: "for `GET`, call `list_books`". `.post(add_book)` is a method
  called on that table, and it returns the table with one more row: "for `POST`, call `add_book`".
  That's why `post` isn't in the `use` line: `get` is a function you call on its own, so it must be
  imported; `.post` is reached through the dot, from the method router `get` returned.
- **Remove it and…** (the `.post(add_book)`) `POST /books` gets `405 Method Not Allowed`, and the
  compiler warns ``function `add_book` is never used``.

`.route(` · `"/books/{id}",`
- **What:** a second route, for paths like `/books/7`.
- **Why:** one book's address is the collection's path plus its id.
- **How:** `{id}` in braces is a placeholder. It matches **any one segment**: `7`, `42`, or even
  `abc`, but not `7/reviews`, which is two segments. The name inside the braces is yours to choose;
  [Path and Query extractors](path-and-query-extractors.md) shows how you read the value. The route
  spans several lines because it's too long for one: `cargo fmt`, Rust's formatter, splits it this
  way.
- **Remove it and…** (writing `":id"` instead of `"{id}"`, the old style) the program compiles,
  then **panics** as it starts. *Common mistakes* shows it.

`get(show_book).put(update_book).delete(delete_book),`
- **What:** three methods on `/books/{id}`: `GET`, `PUT` and `DELETE`, each with its handler.
- **Why:** the last three rows of Step 1.
- **How:** the same chaining, three links long. The order doesn't matter: `.delete(…).put(…)`
  would work the same. There's a matching method for every common HTTP method: `.get`, `.post`,
  `.put`, `.patch`, `.delete`, `.head`, `.options` and `.trace`.
- **Remove it and…** (the `.delete(delete_book)`) `DELETE /books/7` gets `405`.

`.fallback(not_found)`
- **What:** sets the **fallback**: the handler for every request whose path matched no route.
- **Why:** so `/magazines` gets our sentence instead of an empty `404`.
- **How:** it doesn't care about the method: `GET /magazines` and `POST /magazines` both come here.
  It only runs when **no path** matched; `not_found` itself decides the status code, here `404`.
- **Remove it and…** unknown paths get Axum's own empty `404`, as `/contact` did in Hello, Axum.

### Step 6: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/src/main.rs:main}}
```

📁 Full code: code/topics/routes-and-methods · ▶ Run it: `cd code && cargo run -p routes-and-methods`

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
- **What:** calls `app()` to build the Router, then answers requests forever.
- **Why:** this is the loop that makes the program a server.
- **How:** `app()` runs **here**, after the `println!`. That detail matters in *Common mistakes*: a
  broken route is only found when `app()` builds the Router, so the `Listening…` line comes first.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p routes-and-methods
```

```text
   Compiling routes-and-methods v0.1.0 (/Users/you/rust-book-backend/code/topics/routes-and-methods)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running `target/debug/routes-and-methods`
Listening on http://127.0.0.1:3000
```

In the second terminal, send one request for each row of Step 1. `-X` (short for `--request`)
sets the method, as in [How a web backend works, Step 4](../part-0-start/how-a-web-backend-works.md#step-4-send-data-with-a-post);
without it, curl sends `GET`:

```bash
{{#include ../../code/topics/routes-and-methods/http/01-methods.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/01-methods.out}}
```

As in Hello, Axum, your terminal also shows a `date:` line in each response, which the book hides.
Reading it one response at a time:

- **`GET /books`:** `200 OK`, plain text, `all books`: `list_books` ran.
- **`POST /books`:** `201 Created`, `book added`. **Same path**, different method, so a different
  handler, `add_book`, ran, with its own status code.
- **`GET /books/7`:** `200 OK`, `one book`. The `7` filled the `{id}` placeholder, so `show_book`
  ran.
- **`PUT /books/7`:** `200 OK`, `book updated`: same path as the line above, a different method,
  so `update_book` ran.
- **`DELETE /books/7`:** `HTTP/1.1 204 No Content`, and **nothing else**: no `content-type`, no
  `content-length`, no body. A `204` means "there is no body", so the server leaves out the headers
  that would describe one. (Compare the empty 404 in Hello, Axum, which *did* say
  `content-length: 0`.)

Now, the two ways to miss. `PATCH` is a method no route here handles, and `/magazines` is a path no
route has:

```bash
{{#include ../../code/topics/routes-and-methods/http/02-wrong-method.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/02-wrong-method.out}}
```

**`PATCH /books`, the right counter with the wrong form:**
- `HTTP/1.1 405 Method Not Allowed`: the path `/books` exists, but nothing on it handles `PATCH`.
- `allow: GET,HEAD,POST` is the list of methods this path **does** take, separated by commas.
  Axum builds it from your route, so a client that got the method wrong can see what would work.
  `GET` and `POST` are the two you wrote. `HEAD` is there because every `GET` route answers `HEAD`
  too, automatically (*More examples* shows it).
- `content-length: 0`, with no `content-type` and no body: the status line and the `allow` list
  say everything.
- Notice our fallback did **not** run: the body isn't `nothing lives here`. The path matched, so
  this isn't a fallback case.

**`GET /magazines`, a counter that doesn't exist:**
- `HTTP/1.1 404 Not Found`: no route has the path `/magazines`.
- `content-type: text/plain; charset=utf-8` and `content-length: 18`: this 404 **has a body**,
  unlike the one in Hello, Axum.
- `nothing lives here`: the fallback's text. Our `not_found` answered, with its own `404`.

Last, how much a placeholder matches. It fills one segment, whatever that segment is:

```bash
{{#include ../../code/topics/routes-and-methods/http/03-placeholder.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/03-placeholder.out}}
```

- **`GET /books/abc`:** `200 OK`, `one book`. `abc` isn't a number, but `{id}` doesn't check: any
  one segment matches.
- **`GET /books/7/reviews`:** `404`, `nothing lives here`. That's **two** segments after `/books`,
  and `{id}` matches only one, so no route matched, and the fallback answered.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `POST /books` says `201 Created` while `GET /books` says `200 OK`, you're right: one path,
  two handlers, chosen by the method.
- ✅ If `PATCH /books` gets `405` with `allow: GET,HEAD,POST` and `/magazines` gets `404` with
  `nothing lives here`, you've seen both ways a request can miss.
- ❌ If `-X POST /books` gets `405` instead of `201`, check that the `/books` route has
  `.post(add_book)` after `get(list_books)`.
- ❌ If `cargo run` ends with a `panicked at …` message right after `Listening on…`, a route is
  written wrong. See *Common mistakes*.

### Step 7: 404 or 405? How Axum decides

Every request goes through the same two questions, in this order:

```text
request ─→ does its path match a route? ── no ──→ fallback (not_found) → 404
                      │ yes
                      ▼
           is its method on that route? ── no ──→ 405, with an allow: list
                      │ yes
                      ▼
           that method's handler runs
```

### Line by line

`request ─→ does its path match a route?`
- **What:** the first question is about the **path** only.
- **Why:** paths decide *which route*; methods only matter inside a route.
- **How:** Axum compares the path with every route's path, placeholders included. The order of the
  `.route` lines doesn't matter.
- **Remove it and…** there's no way to tell `/books` from `/books/7`.

`── no ──→ fallback (not_found) → 404`
- **What:** a path no route has goes to the fallback.
- **Why:** there's nothing else it could go to.
- **How:** with `.fallback(not_found)`, `not_found` answers (our `404` with a sentence). Without a
  fallback, Axum answers an empty `404` itself.
- **Remove it and…** (the fallback) `GET /magazines` gets an empty `404`.

`│ yes ▼ is its method on that route?`
- **What:** the second question, about the **method**, inside the route that matched.
- **Why:** one route holds a handler per method.
- **How:** Axum looks for the request's method in that route's method router (`GET`, `POST`…).
- **Remove it and…** one path could only ever do one job.

`── no ──→ 405, with an allow: list`
- **What:** the path is right, the method isn't, so Axum answers `405 Method Not Allowed`, with an
  `allow:` header listing the methods that do work.
- **Why:** it's a different mistake from a wrong path, so it gets a different code.
- **How:** the fallback is **not** asked: it only handles paths that matched nothing.
- **Remove it and…** a client couldn't tell "wrong address" from "wrong verb".

`│ yes ▼ that method's handler runs`
- **What:** both questions said yes: the handler for this path and this method answers.
- **Why:** that's the normal case: `POST /books` runs `add_book`.
- **How:** Axum calls the handler, and sends back what it returns.
- **Remove it and…** nothing would ever answer.

## You might be wondering…

**"Why not one handler with an `if` on the method?"**
You could write one handler for `/books` that asks "is this a `GET` or a `POST`?". But then Axum
no longer knows which methods the path supports, so it can't send `405` with an `allow:` list for
you: every other method would reach your handler, and you'd have to answer it yourself. The handler
also grows with every method, while one handler per method stays small, and can be tested on its
own. And in later lessons, each method needs different things from the request (a `POST` reads the
new book from the body; a `GET` doesn't), which is much easier to write as separate handlers.

**"What's the difference between 404 and 405?"**
`404 Not Found`: no route has this **path**. `405 Method Not Allowed`: a route has this path, but
not this **method**. Step 7's diagram shows the order Axum asks the two questions in. Your
fallback changes what a `404` says; a `405` still comes from Axum, with its `allow:` list.

**"Does `{id}` check that it's a number?"**
No, not yet: the transcript above shows `GET /books/abc` reaching `show_book`. The placeholder only
says "one segment goes here". The lesson [Path and Query extractors](path-and-query-extractors.md)
shows how to read the value as a number, and what Axum answers when it isn't one.

**"Can two routes share a handler?"**
Yes. A handler is a function, and you can hand the same function to Axum as many times as you like.
For example, `.route("/", get(list_books))` next to the `/books` route makes the home page show the
book list too.

**"Can I write two `.route` calls for the same path?"**
Yes, if they handle **different** methods: `.route("/books", get(list_books))` followed by
`.route("/books", post(add_book))` (with `post` in the `use` line) works exactly like the chained
version, because Axum merges them into one route. The chained form is the usual style, because it
shows everything a path does in one place. What you can't do is give the same method on the same
path two handlers; *Common mistakes* shows what happens.

## Coming from another language?

Every framework lets one path answer several methods, and most answer a wrong method with `405`.
The differences are in the placeholder syntax and in how you set a fallback.

**Express (Node.js).** One call per method and path, or chained on `app.route`, which is close to
Axum's method chaining. The placeholder is `:id`, which is what Axum used before 0.8:

```js
app.route("/books").get(listBooks).post(addBook);
app.route("/books/:id").get(showBook).put(updateBook).delete(deleteBook);
app.use((req, res) => res.status(404).send("nothing lives here"));
```

The last line is Express's fallback: [middleware](../glossary.md#middleware) registered after
every route. One difference: Express has no automatic `405`. `PATCH /books` gets a `404` page
saying `Cannot PATCH /books`.

**Flask and FastAPI (Python).** Flask lists methods per route, or has one decorator per method
since Flask 2.0. Its placeholder uses angle brackets, and can check the type with a *converter*:

```python
@app.get("/books")
def list_books(): return "all books"

@app.post("/books")
def add_book(): return "book added", 201

@app.get("/books/<int:id>")
def show_book(id): return "one book"

@app.errorhandler(404)
def not_found(e): return "nothing lives here", 404
```

FastAPI uses braces like Axum: `@app.get("/books/{id}")`, and a type hint, `def show_book(id: int)`,
makes it check for a number. Both answer a wrong method with `405` and an `Allow` header.

**Spring Boot (Java).** One annotation per method: `@GetMapping("/books")`,
`@PostMapping("/books")`, `@GetMapping("/books/{id}")`, `@PutMapping`, `@DeleteMapping`. The
placeholder uses braces, read with `@PathVariable`. A wrong method gets `405` with an `Allow` header.

**Go (`net/http` and Gin).** Since Go 1.22, the standard router puts the method in the pattern, and
uses braces:

```go
mux.HandleFunc("GET /books", listBooks)
mux.HandleFunc("POST /books", addBook)
mux.HandleFunc("GET /books/{id}", showBook)
```

A wrong method gets `405` with an `Allow` header; an unknown path gets `404 page not found`. Gin
uses `r.GET("/books/:id", …)` and `r.NoRoute(…)` for the fallback, and answers a wrong method with
`404` unless you set `r.HandleMethodNotAllowed = true`.

## Common mistakes

**Writing the placeholder as `:id`.**
Axum 0.7 and older wrote placeholders with a colon, and so does Express, so you'll see `:id` in
older tutorials and answers online. In Axum 0.8 it's a mistake:

```rust,noplayground,ignore
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books).post(add_book))
        .route(
            "/books/:id",
            get(show_book).put(update_book).delete(delete_book),
        )
        .fallback(not_found)
}
```

It compiles without a single warning: to the compiler, `"/books/:id"` is text like any other. The
mistake shows up when you run it:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/my-routes`
Listening on http://127.0.0.1:3000

thread 'main' (12846898) panicked at src/main.rs:30:10:
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

A **panic** is Rust stopping the program on purpose, with a message, because it found something it
can't carry on from. Read it in order:

- `Listening on http://127.0.0.1:3000` comes **first**, because the port opens before `app()` runs
  (Step 6). The server looks as if it started, then dies straight away.
- `panicked at src/main.rs:30:10` is the file, line and column of the `.route(` call with the bad
  path (this was captured in a project called `my-routes`; your line number depends on your file).
- ``Path segments must not start with `:`. For capture groups, use `{capture}`.`` is the real
  message: a *capture group* is Axum's name for a placeholder, and it must be written in braces.
- `without_v07_checks` is for the rare app that really wants a path segment starting with a colon.
  You don't need it.

**Fix:** write `"/books/{id}"`. Axum checks every path when `app()` builds the Router, so a mistake
like this one stops the program at once, instead of quietly never matching.

**Giving the same method two handlers on one path.**
You copy the `/books` line to add the `POST` route, change the handler, and forget to change `get`
to `post`:

```rust,noplayground,ignore
fn app() -> Router {
    Router::new()
        .route("/books", get(list_books))
        .route("/books", get(add_book))
        .route(
            "/books/{id}",
            get(show_book).put(update_book).delete(delete_book),
        )
        .fallback(not_found)
}
```

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s
     Running `target/debug/my-routes`
Listening on http://127.0.0.1:3000

thread 'main' (12847711) panicked at src/main.rs:30:10:
Overlapping method route. Handler for `GET /books` already exists
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Two handlers for `GET /books` would leave Axum no way to choose, so it refuses at startup. (Line
30 is the second `.route("/books", …)` here.)
"Overlapping method route" means two routes claim the same method on the same path, and the
message names it: `GET /books`. **Fix:** chain the second method onto the first route,
`.route("/books", get(list_books).post(add_book))`, as in Step 5. (Two `.route` calls for the same
path with *different* methods are fine; see *You might be wondering…*.)

**Expecting the fallback to catch a wrong method.**
`.fallback(not_found)` doesn't make `PATCH /books` say `nothing lives here`. As the `02-wrong-method`
transcript in Step 6 shows, it gets Axum's `405` with an empty body: the path matched, so the
fallback isn't asked (Step 7). **Fix:** nothing to fix. That's the right answer for a wrong method,
and the `allow:` list tells the client what to send instead.

## More examples

Each example is a complete program in `code/topics/routes-and-methods/examples/`. Stop any running
server with `Ctrl+C`, start the example with the command shown, and send its requests from your
second terminal.

### `any`: every method, one handler

Sometimes a path should answer whatever the method is, such as a "ping" address that a monitoring
tool checks to see if the server is up (`cargo run -p routes-and-methods --example routes-any`):

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-any.rs:imports}}

{{#include ../../code/topics/routes-and-methods/examples/routes-any.rs:ping}}

{{#include ../../code/topics/routes-and-methods/examples/routes-any.rs:app}}
```

```bash
{{#include ../../code/topics/routes-and-methods/http/50-any.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/50-any.out}}
```

`any(ping)` makes a method router that sends **every** method to `ping`, so there's never a `405`
on this path. Use it rarely: for most paths, a separate handler per method is clearer, as the
questions above explain.

### A `GET` route answers `HEAD` too

`HEAD` is the method for "send me only the headers, not the body": a way to check that a page
exists, and how big it is, without downloading it. You never write a `HEAD` handler for it
(`cargo run -p routes-and-methods --example routes-head`):

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-head.rs:home}}

{{#include ../../code/topics/routes-and-methods/examples/routes-head.rs:app}}
```

`curl -I` (a **capital** i, short for `--head`) sends a `HEAD` request, and prints only the headers
that come back. (A lowercase `-i` is the `GET` with headers that you've used all along.)

```bash
{{#include ../../code/topics/routes-and-methods/http/51-head.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/51-head.out}}
```

The `HEAD` answer has the same status and the same headers as the `GET`, down to
`content-length: 22`, but no body. `content-length` still says how big the body **would** be. Axum
does this for every `get(…)` route: it runs the `GET` handler, then throws the body away. That's
why `HEAD` appeared in the `allow:` list in Step 6.

### A fallback that returns an HTML page

A fallback is a normal handler, so it can return anything a handler can, such as a small web page
with a link home (`cargo run -p routes-and-methods --example routes-html-fallback`):

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-html-fallback.rs:imports}}

{{#include ../../code/topics/routes-and-methods/examples/routes-html-fallback.rs:not_found}}

{{#include ../../code/topics/routes-and-methods/examples/routes-html-fallback.rs:app}}
```

```bash
{{#include ../../code/topics/routes-and-methods/http/52-html-fallback.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/52-html-fallback.out}}
```

The tuple holds the status and an `Html` body, so the answer is `404 Not Found` **and**
`content-type: text/html`: a browser draws the heading and the link. The `\"` inside the text is a
double quote written with a backslash in front, so it doesn't end the string (Hello, Axum's raw
strings, `r#"…"#`, are the other way).

### Only `.post`, so `GET` gets 405

A path that only accepts new data, such as a contact form's "send message" address
(`cargo run -p routes-and-methods --example routes-post-only`):

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-post-only.rs:imports}}

{{#include ../../code/topics/routes-and-methods/examples/routes-post-only.rs:send_message}}

{{#include ../../code/topics/routes-and-methods/examples/routes-post-only.rs:app}}
```

```bash
{{#include ../../code/topics/routes-and-methods/http/53-post-only.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/53-post-only.out}}
```

This time the route starts with `post(…)`, so `post` is in the `use` line and `get` isn't. The
`allow:` list is `POST` alone: there's no `GET` route, so there's no automatic `HEAD` either. This
is the same `405` a browser gets if you type the address into its address bar, because a browser's
address bar always sends `GET`.

## Your turn

Each solution is also a complete program in `code/topics/routes-and-methods/examples/`, with a test
inside.

### 🟢 Guided

`PATCH` means "change **part** of a thing", where `PUT` replaces the whole thing. Add a `PATCH`
handler to `/books/{id}` that answers `book patched`: write an `async fn patch_book`, then add
`.patch(patch_book)` to the `/books/{id}` route. Then try `PATCH` on `/books/7` and on `/books`.

<details><summary>Solution</summary>

`cargo run -p routes-and-methods --example routes-patch`:

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-patch.rs:patch_book}}

{{#include ../../code/topics/routes-and-methods/examples/routes-patch.rs:app}}
```

```bash
{{#include ../../code/topics/routes-and-methods/http/90-patch.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/90-patch.out}}
```

`PATCH /books/7` now reaches `patch_book`. `PATCH /books` is still `405`, with the same
`allow: GET,HEAD,POST` as before: you added `PATCH` to `/books/{id}` only, and each route has its own
list of methods. The chain got long enough that `cargo fmt` put each method on its own line.

</details>

### 🟡 Tweak

Make the fallback say **which** path was missing: `GET /magazines` should answer
`nothing lives at /magazines`. You need one new tool: `Uri`, from `axum::http`.

<details><summary>Solution</summary>

`cargo run -p routes-and-methods --example routes-fallback-uri`:

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-fallback-uri.rs:imports}}

{{#include ../../code/topics/routes-and-methods/examples/routes-fallback-uri.rs:not_found}}
```

Giving a handler a parameter of type `Uri` makes Axum fill it with the request's address, the path
plus any `?query` part: a handler parameter that Axum fills from the request like this is called an
**extractor**, and the next lessons are full of them. `format!` builds the sentence, so the body is
now a `String` instead of a `&'static str`. The Router doesn't change.

```bash
{{#include ../../code/topics/routes-and-methods/http/91-fallback-uri.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/91-fallback-uri.out}}
```

The second request shows the query string is part of a `Uri`: `/magazines/3?page=2`. (The address
is in double quotes because `?` has a special meaning in some shells, such as zsh.) In the book's
copy, the test `wrong_method_is_405_and_unknown_path_hits_the_fallback` in `src/main.rs` checks for
the old text, `nothing lives here`, so it fails if you change `src/main.rs` itself. Update the
expected text in the test too.

</details>

### 🔴 From scratch

Add authors to the library: `/authors` with `GET` (answers `all authors`) and `POST` (`201
Created`, `author added`), and `/authors/{id}` with `GET` (`one author`) and `DELETE` (`204 No
Content`). What does `PUT /authors/3` get, and what does its `allow:` line say?

<details><summary>Solution</summary>

`cargo run -p routes-and-methods --example routes-authors`:

```rust,noplayground
{{#include ../../code/topics/routes-and-methods/examples/routes-authors.rs:imports}}

{{#include ../../code/topics/routes-and-methods/examples/routes-authors.rs:handlers}}

{{#include ../../code/topics/routes-and-methods/examples/routes-authors.rs:app}}
```

```bash
{{#include ../../code/topics/routes-and-methods/http/92-authors.sh}}
```

```text
{{#include ../../code/topics/routes-and-methods/http/92-authors.out}}
```

Four handlers, two routes, and each route's methods chained after its first `get(…)`. `PUT
/authors/3` gets `405`, with `allow: GET,HEAD,DELETE`: the methods you gave `/authors/{id}`, plus
the `HEAD` that comes with `GET`.

</details>

## Quick check

<div class="quiz" data-topic="routes-and-methods"></div>

## Remember this

- One path, many methods: `.route("/books", get(list_books).post(add_book))`. Only the first
  method (`get`) is imported; the rest are chained with a dot.
- A placeholder is written in braces, `"/books/{id}"`, and matches any one segment. The old `:id`
  style panics when the app starts.
- `404`: no route has the path. `405`: the path exists, but not for this method, and the `allow:`
  header lists the methods that work. Every `GET` route answers `HEAD` too.
- `.fallback(handler)` answers every request whose path matched no route; it doesn't change `405`s.

## Go deeper

- [axum::routing](https://docs.rs/axum/0.8.9/axum/routing/index.html) — Official reference for Router, route, the method functions and fallback.
- [MDN: HTTP request methods](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Methods) — What each HTTP method means, with examples.
- [MDN: 405 Method Not Allowed](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/405) — The status code, and the `Allow` header that comes with it.

<!-- next:start -->

**Next:**

- [Handlers and IntoResponse](../a2-axum/handlers-and-into-response.md)

<!-- next:end -->
