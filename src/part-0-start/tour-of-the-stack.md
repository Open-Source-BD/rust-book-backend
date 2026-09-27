# Tour of the stack

> **Beginner** · Part 0 · Before you start

## By the end of this lesson

- You know what Tokio, Axum, SeaORM and Postgres each do.
- You can follow one request through all four layers.
- You have run your first Axum server and seen it answer.

## What & why

Meet the four tools of this book, see how one request flows through them, and run your first Axum server: under twenty lines of Rust.

In the last lesson you saw what a [request](../glossary.md#request) and a
[response](../glossary.md#response) look like. Now: who, inside a Rust
[backend](../glossary.md#backend), actually reads that request and writes that response? Not one
program but a small team of [crates](../glossary.md#crate) (Rust libraries), each doing one job.
The group of tools a backend is built from is called its **stack**, because each one sits on top
of the one below. This book's stack has four members. Picture them as the staff of our restaurant:

| Tool | Restaurant role | Its job |
|---|---|---|
| **Tokio** | The shift manager | Lets one cook juggle many orders: while one order waits for the oven, the cook starts the next. Tokio is the [**runtime**](../glossary.md#runtime) for [**async**](../glossary.md#async) Rust. |
| **Axum** | The head waiter and the order system | Takes each order from the door and hands it to the right cook. Axum is the web [**framework**](../glossary.md#framework). |
| **SeaORM** | The pantry clerk | Fetches and stores ingredients for the cooks, speaking "Rust" to them and "pantry" to the pantry. SeaORM is the [**ORM**](../glossary.md#orm). |
| **PostgreSQL** | The pantry | Keeps every ingredient safe, even when the restaurant closes for the night. Postgres is the [**database**](../glossary.md#database). |

You met Postgres in [Your toolbox](your-toolbox.md). This lesson follows one request through all
four, then you'll run a real Axum [server](../glossary.md#server): under twenty lines of Rust that
answer your curl. SeaORM and Postgres join in Part A3; today you'll only see where they fit.

## The idea, slowly

### Step 1: one request, through the whole stack

Here's the trip one request takes when someone runs `curl http://127.0.0.1:3000/products/42`
against a finished backend, and the trip its answer takes back:

```text
curl ──▶ TCP port 3000 ──▶ Tokio ──▶ Axum Router ──▶ handler ──▶ SeaORM ──▶ Postgres
curl ◀── TCP port 3000 ◀── Tokio ◀── Axum Router ◀── handler ◀── SeaORM ◀── Postgres
```

### Line by line

`curl ──▶ TCP port 3000`
- **What:** curl (the [client](../glossary.md#client)) opens a connection to
  [port](../glossary.md#port) 3000 and writes the request into it.
- **Why:** the server is listening behind door 3000; that's where requests must arrive.
- **How:** the connection uses **TCP**, the ordinary way two programs open a reliable two-way line
  across a network (or inside one computer). [HTTP](../glossary.md#http) messages are text sent
  over a TCP connection.
- **Remove it and…** with nobody listening on port 3000, curl gets `Couldn't connect to server`.

`TCP port 3000 ──▶ Tokio`
- **What:** Tokio notices the new connection and the bytes arriving on it.
- **Why:** waiting for network data is slow. Tokio waits for *all* connections at once, so one
  program can serve thousands of people without one slow client holding up the rest.
- **How:** it runs each connection as a small **task** (a piece of async work it can pause and
  resume), and wakes the task up when its data is ready.
- **Remove it and…** Axum has nothing to run on: Axum is built on top of Tokio.

`Tokio ──▶ Axum Router`
- **What:** Axum reads the bytes as an HTTP request, and its **Router** looks at the method and
  path (`GET /products/42`).
- **Why:** a backend has many [endpoints](../glossary.md#endpoint), and each needs different code.
- **How:** the Router holds a list of [**routes**](../glossary.md#route): rules like "`GET /`
  goes to the function `hello`". It picks the one that matches.
- **Remove it and…** there's no way to tell which code should answer. (A request that matches no
  route gets `404 Not Found`.)

`Axum Router ──▶ handler`
- **What:** Axum calls the [**handler**](../glossary.md#handler) the route points to: an ordinary
  async Rust function you write.
- **Why:** this is *your* code, the one part that knows what a product is.
- **How:** Axum passes the handler what it asks for (the id `42`, a [JSON](../glossary.md#json) body, and so on; that
  comes in Part A2).
- **Remove it and…** there's nobody to cook the dish.

`handler ──▶ SeaORM ──▶ Postgres`
- **What:** the handler asks SeaORM for product 42; SeaORM turns that into
  [**SQL**](../glossary.md#sql) (the language databases understand) and sends it to Postgres.
- **Why:** the product's data lives in the database, not in the program.
- **How:** SeaORM writes the SQL (`SELECT … WHERE id = 42`) for you, and sends it over its own
  TCP connection to Postgres on port 5433.
- **Remove it and…** your backend has no memory: everything is gone when it stops.

The bottom line (every `◀──`)
- **What:** the answer travels back the same way. Postgres returns a row, SeaORM turns it into a
  Rust struct, the handler returns it, Axum turns it into an HTTP response, Tokio writes the bytes
  to the TCP connection, and curl prints them.
- **Why:** every request gets exactly one response, sent back along the connection it came in on.
- **How:** each layer only talks to its neighbours. Your handler never touches TCP bytes, and
  Postgres never sees HTTP.
- **Remove it and…** (any layer) the chain breaks at that point, and curl never gets its answer.

The server you run today uses the first half of this chain only: curl, TCP, Tokio, Axum, handler.
Its handler returns a fixed piece of text instead of asking a database.

### Step 2: the crate's `Cargo.toml`

Every listing in this lesson comes from one small Rust project in the book's repository:
`code/topics/tour-of-the-stack`. (No copy of the repository? See *You might be wondering…* for
how to build the same project in a folder of your own.) First, its `Cargo.toml`, the settings file
that says which crates the project uses:

```toml
{{#include ../../code/topics/tour-of-the-stack/Cargo.toml}}
```

### Line by line

`[package]`
- **What:** starts the section that describes this project itself.
- **Why:** Cargo needs a name and version for every project it builds.
- **How:** everything below it, up to the next `[…]` line, belongs to it.
- **Remove it and…** Cargo refuses the file: a project without a `[package]` section isn't a
  project.

`name = "tour-of-the-stack"` · `version = "0.1.0"`
- **What:** the project's name and version number.
- **Why:** the name is what you type in `cargo run -p tour-of-the-stack` (`-p` means *package*,
  which is Cargo's word for a project), and it becomes the name of the program Cargo builds.
- **How:** `0.1.0` is the usual starting version for a new project.
- **Remove it and…** Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "use the edition and publish settings from the **workspace**."
- **Why:** the book's `code/` folder is a *workspace*: one folder holding many small projects that
  share settings. Writing a setting once, in `code/Cargo.toml`, keeps every project the same.
- **How:** `.workspace = true` means "look this value up in the workspace's `Cargo.toml`". There,
  `edition = "2024"` and `publish = false` (which stops anyone accidentally uploading this practice
  project to crates.io, the public library of Rust crates).
- **Remove it and…** (`edition`) Cargo warns that it's ``defaulting to `2015` ``, the oldest
  edition, and the build fails with ``error[E0670]: `async fn` is not permitted in Rust 2015``.

`[dependencies]`
- **What:** starts the list of crates this program needs to run.
- **Why:** Cargo downloads and builds each one the first time you build.
- **How:** one line per crate.
- **Remove it and…** (the section) nothing from `axum` or `tokio` can be used, and every `use`
  line fails.

`axum.workspace = true`
- **What:** adds **Axum**, the web framework: the Router, handlers, and turning Rust values into
  HTTP responses.
- **Why:** it's the head waiter from *What & why*.
- **How:** as with `edition`, `.workspace = true` means "the version (and options) are set once, in
  `code/Cargo.toml`". Every project in the book uses the same Axum version: `0.8.9`.
- **Remove it and…** `use axum::…` fails with ``unresolved import `axum` ``.

`tokio.workspace = true`
- **What:** adds **Tokio**, the async runtime.
- **Why:** Axum runs on top of it, and our `main` function needs it to start (Step 5).
- **How:** version `1.53.1` and three *features*, set in the workspace file below.
- **Remove it and…** `#[tokio::main]` and `tokio::net::TcpListener` are unknown names.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** crates used only by the project's **tests** (small functions that check the code works,
  run with `cargo test`), not by the program itself.
- **Why:** the tests at the bottom of `src/main.rs` send a pretend request to the Router without
  opening a real port. `tower` provides the `oneshot` method that sends one request; `http-body-util`
  helps read the response body.
- **How:** Cargo builds `[dev-dependencies]` only for tests, so the real program stays smaller.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

Those `.workspace = true` lines point at the workspace's own `Cargo.toml`, `code/Cargo.toml`. This
is where the versions really live:

```toml
{{#include ../../code/Cargo.toml}}
```

### Line by line

`[workspace]` · `resolver = "3"` · `members = ["topics/*"]`
- **What:** declares `code/` as a workspace whose members are every folder inside `topics/`.
- **Why:** one `cargo test --workspace --all-targets` from `code/` builds and tests every project in the book — `--all-targets` also runs the tests inside each project's `examples/` folder, which plain `cargo test --workspace` skips.
- **How:** `resolver = "3"` picks the newest rules Cargo uses to choose crate versions (the
  default for edition 2024).
- **Remove it and…** (`members`) Cargo doesn't know `tour-of-the-stack` belongs here:
  ``package(s) `tour-of-the-stack` not found in workspace``.

`[workspace.package]` · `edition = "2024"` · `publish = false`
- **What:** the shared package settings that `edition.workspace = true` looks up.
- **Why:** set once, used everywhere.
- **How:** an **edition** is a set of Rust language rules a project opts into; 2024 is the newest.
- **Remove it and…** each project's `edition.workspace = true` has nothing to look up, and Cargo
  refuses to build.

`[workspace.dependencies]` · `axum = "0.8.9"`
- **What:** the one place the Axum version is written.
- **Why:** if each project picked its own version, two lessons could behave differently.
- **How:** `"0.8.9"` lets Cargo use 0.8.9 or any later 0.8.x fix release, but never 0.9, which
  could change how things work.
- **Remove it and…** `axum.workspace = true` fails: there's no version to look up.

`tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread", "net"] }`
- **What:** the Tokio version, plus three **features**. A feature is an optional part of a crate
  that's only compiled if you ask for it, which keeps programs small.
- **Why:** Tokio is big, so it's split into features. We turn on exactly the three we use:
  - `macros` gives us `#[tokio::main]` (Step 5) and `#[tokio::test]` (used by the tests).
  - `rt-multi-thread` gives us the runtime that spreads work across all your computer's
    processor cores. `#[tokio::main]` uses this one unless you ask for another.
  - `net` gives us `TcpListener`, which listens on a port (Step 5).
- **How:** the `{ … }` form lets one line hold a version *and* options.
- **Remove it and…** (a feature) the part that needs it fails to compile. **Common mistakes** shows
  the exact errors.

`tower` · `http-body-util`
- **What:** the versions for the two test-only crates.
- **Why:** the same reason as above: one place, one version.
- **How:** `features = ["util"]` turns on the part of `tower` that contains `oneshot`.
- **Remove it and…** `cargo test` stops compiling.

### Step 3: the handler

Now the Rust. This is the handler, the function that writes the answer:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/src/main.rs:handler}}
```

📁 Full code: code/topics/tour-of-the-stack · ▶ Run it: `cd code && cargo run -p tour-of-the-stack`

### Line by line

`async fn hello() -> &'static str {`
- **What:** declares an async function called `hello` that takes nothing and returns a piece of
  text.
- **Why:** every Axum handler is an async function. This one doesn't wait for anything yet, but
  later handlers wait for the database, and `async` is what lets Tokio run other requests during
  that wait. (A reminder from Rust for Humans: an `async fn` can pause at each `.await`; see
  [Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html).)
- **How:** `&'static str` is the return type, read in two parts. `&str` is a borrowed piece of
  text (a *string slice*). `'static` is a *lifetime* meaning "valid for the whole time the program
  runs": the text `"Hello from Axum!"` is baked into the compiled program itself, so it never goes
  away. See Rust for Humans:
  [Strings and &str](https://open-source-bd.github.io/rustbook-for-human/abstractions/strings-and-str.html)
  and [Lifetime elision and 'static](https://open-source-bd.github.io/rustbook-for-human/ownership/lifetime-elision-and-static.html).
- **Remove it and…** (the `async`) Axum no longer accepts `hello` as a handler:
  ``the trait bound `fn() -> &'static str {hello}: Handler<_, _>` is not satisfied``.

`"Hello from Axum!"`
- **What:** the text the handler returns. No `return` keyword, no semicolon: the last expression
  in a Rust function is its return value.
- **Why:** returning a string is enough. You don't build a response yourself; Axum does it.
- **How:** Axum turns anything a handler returns into an HTTP response, using a trait called
  `IntoResponse` (a *trait* is a set of abilities a type promises to have). For text, it builds
  status `200 OK`, the header `content-type: text/plain; charset=utf-8`, and your text as the body.
  Other types become other responses: a [status code](../glossary.md#status-code) plus text, JSON,
  and more. Part A2's lesson
  *Handlers and IntoResponse* covers it in full.
- **Remove it and…** the function returns nothing (`()`), which doesn't match `&'static str`, and
  the compiler stops with `mismatched types`.

### Step 4: the Router

Next, the table of routes, with the `use` line from the top of `src/main.rs` that brings `Router`
and `get` into scope:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/src/main.rs:imports}}

{{#include ../../code/topics/tour-of-the-stack/src/main.rs:app}}
```

📁 Full code: code/topics/tour-of-the-stack · ▶ Run it: `cd code && cargo run -p tour-of-the-stack`

### Line by line

`use axum::{Router, routing::get};`
- **What:** makes two names from the `axum` crate usable here: `Router` and `get`.
- **Why:** without it you'd have to write `axum::Router` and `axum::routing::get` every time.
- **How:** the braces list several names from the same crate; `routing::get` means "`get`, which
  lives in axum's `routing` module".
- **Remove it and…** ``cannot find type `Router` in this scope`` and the same for `get`.

`fn app() -> Router {`
- **What:** a normal (not async) function that builds and returns the app's `Router`.
- **Why:** it's a separate function so that **two** places can use it: `main` (to serve real
  requests) and the tests at the bottom of the file (to send pretend requests without opening a
  port). If the Router were built inside `main`, the tests couldn't reach it.
- **How:** it returns a `Router`, Axum's route table.
- **Remove it and…** (the function, with its body moved into `main`) the server still runs, but
  the tests can't be written.

`Router::new().route("/", get(hello))`
- **What:** creates an empty Router and adds one route: `GET /` is answered by `hello`.
- **Why:** Axum must know which function answers which method and path.
- **How:** `Router::new()` starts an empty table. `.route(path, …)` adds a rule for the path
  `"/"` (the root: the address with nothing after the port). `get(hello)` says "for `GET`
  requests, call `hello`". Notice `hello` has **no `()`**: we're not calling it now, we're
  *handing the function itself* to Axum, which calls it later, once for every matching request.
  Other methods on `/` (like `POST`) get `405 Method Not Allowed`; other paths get
  `404 Not Found`.
- **Remove it and…** (the `.route(…)`) the Router is empty, and every request gets
  `404 Not Found`.

### Step 5: `main`, where it all starts

Last, the `main` function, which opens port 3000 and starts serving:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/src/main.rs:main}}
```

📁 Full code: code/topics/tour-of-the-stack · ▶ Run it: `cd code && cargo run -p tour-of-the-stack`

### Line by line

`#[tokio::main]`
- **What:** an **attribute**, an instruction to the compiler written `#[…]` above an item. This one
  is a *macro* (code that writes code) from Tokio: it rewrites the `main` function below it.
- **Why:** Rust won't let `main` be `async` on its own. Someone has to create the runtime that runs
  async code, and `main` is where the program starts, so nobody else could.
- **How:** behind the scenes, it turns your `async fn main` into roughly this ordinary `main`:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/tokio-main-expanded.rs:expanded}}
```

That is: build a multi-threaded Tokio runtime, switch on everything it can do (network, timers),
stop with the message `Failed building the Runtime` in the rare case it can't be built, then
`block_on` your async code: run it and don't return until it's finished.

- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`async fn main() {`
- **What:** the program's starting point, written as an async function.
- **Why:** the lines inside need `.await` (the port, the server), and `.await` only works inside
  async code.
- **How:** `#[tokio::main]` above makes this allowed (see the previous entry).
- **Remove it and…** (the `async`) `#[tokio::main]` itself complains:
  ``the `async` keyword is missing from the function declaration``.

`let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")`
- **What:** asks the operating system for port 3000 on address `127.0.0.1`, and gets back a
  **listener**: an object that waits for new TCP connections on that port.
- **Why:** a server must claim a port before any client can reach it.
- **How:** `127.0.0.1` is the numeric address of [localhost](../glossary.md#localhost), meaning
  "this computer": only programs on your own machine can connect. `bind` is async (it may first have
  to look the address up, which can take a moment), so it returns a *future* (a value that will
  be ready later) instead of the listener itself.
- **Remove it and…** there's no port, and `axum::serve` below has nothing to listen on.

`.await`
- **What:** waits for `bind` to finish and takes out its result.
- **Why:** a future does nothing until you `.await` it.
- **How:** while `main` waits here, Tokio is free to run other tasks.
- **Remove it and…** the compiler stops: you'd be calling `.expect` on the future, not on its
  result. The exact error is in **Common mistakes**.

`.expect("port 3000 is busy: stop the other program using it, or change the port");`
- **What:** `bind` returns a `Result`: `Ok(listener)` or `Err(error)`. `.expect` takes out the
  listener, or, on an error, stops the program with this message.
- **Why:** if the port can't be opened, the server can't do anything useful, so stopping is right.
  The message is written for *you*, at the moment it fails: it says what happened (the port is
  busy, by far the most common cause) and what to do about it. A bare `.unwrap()` would print only
  Rust's error, with no advice.
- **How:** this kind of stop is a **panic**: Rust prints your message, then the underlying error,
  and ends the program. You'll see exactly that in Step 6's ❌ line.
- **Remove it and…** `listener` would be a `Result`, not a listener, and `axum::serve` refuses it:
  ``the trait bound `Result<tokio::net::TcpListener, std::io::Error>: Listener` is not satisfied``.

`println!("Listening on http://127.0.0.1:3000");`
- **What:** prints the server's address to the terminal.
- **Why:** it tells you the server is ready and which address to use.
- **How:** it runs after `bind` succeeded, so when you see it, the port is really open.
- **Remove it and…** the server works the same, but it sits there in silence and you can't tell
  whether it started.

`axum::serve(listener, app())`
- **What:** starts the server: accept each connection on `listener`, read its HTTP requests, and
  hand each one to the Router from `app()`.
- **Why:** this is the loop that makes the program a server.
- **How:** it runs forever, handling every connection as its own Tokio task. It only returns if
  something goes badly wrong.
- **Remove it and…** (the whole `axum::serve(…).await.expect(…)` statement) the program prints
  `Listening on …` and immediately exits, and curl gets `Couldn't connect to server`.

`.await` · `.expect("the server stopped because of an error");`
- **What:** runs the server (a future, like `bind`) and, in the rare case it stops with an error,
  panics with this message.
- **Why:** `.await` is what actually starts the serving; `.expect` makes a crash say what happened.
- **How:** in normal use this line never finishes: you stop the server with `Ctrl+C`.
- **Remove it and…** (the `.await`) the compiler stops, for the same reason as with `bind`: the
  unstarted server has no `.expect`
  (``no method named `expect` found for struct `Serve<L, M, S>` ``).

### Step 6: run it

Time to see it answer. You need **two terminal windows**: one for the server (which keeps
running) and one for curl.

```bash
cd code && cargo run -p tour-of-the-stack
```

```bash
curl -i http://127.0.0.1:3000/
```

### Line by line

`cd code && cargo run -p tour-of-the-stack` (first terminal)
- **What:** moves into the book's `code/` folder, then builds and starts the server.
- **Why:** the workspace lives in `code/`, so Cargo must run from there.
- **How:** `&&` runs the second command only if the first worked. `-p tour-of-the-stack` picks
  which project in the workspace to run.
- **Remove it and…** (the `-p …`) today it still works, because `tour-of-the-stack` is the only
  program in the workspace. Once the workspace holds more than one project, as later lessons add
  their own, Cargo needs `-p` to know which one to run, and stops with an error asking you to
  choose.

`curl -i http://127.0.0.1:3000/` (second terminal)
- **What:** sends `GET /` to your server and prints the whole response.
- **Why:** it's the client in the diagram from Step 1.
- **How:** the same `-i` as in the last lesson. This URL starts with `http`, not `https`: your
  local server doesn't use encryption, which is fine inside one computer. `:3000` is the port.
- **Remove it and…** (the `:3000`) curl knocks on port 80, the usual door for `http`, and your
  server isn't there.

### Run it

In the first terminal:

```bash
cd code && cargo run -p tour-of-the-stack
```

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/tour-of-the-stack`
Listening on http://127.0.0.1:3000
```

The very first time, Cargo downloads and compiles Axum, Tokio and the crates they use, so you'll
see about fifty `Compiling …` lines first, ending with:

```text
…
   Compiling axum v0.8.9
   Compiling tour-of-the-stack v0.1.0 (/Users/you/rust-book-backend/code/topics/tour-of-the-stack)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.17s
```

The terminal then stays busy: the server is running, waiting for requests. Leave it open. In the
second terminal:

```bash
curl -i http://127.0.0.1:3000/
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 16
date: Sat, 26 Sep 2026 07:33:00 GMT

Hello from Axum!
```

That's the response from the last lesson's Step 2, made by your own Rust: a status line
(`200 OK`), headers, an empty line, and the body. `content-type` is the `text/plain` Axum chose
for a string (Step 3), and `content-length: 16` is the number of characters in
`Hello from Axum!`. When you're done, go back to the first terminal and press **`Ctrl+C`** to stop
the server.

- ✅ If you see `HTTP/1.1 200 OK` and `Hello from Axum!`, you're right: your first Rust server
  answered.
- ❌ If `cargo run` stops with
  `port 3000 is busy: stop the other program using it, or change the port: Os { code: 48, kind: AddrInUse, message: "Address already in use" }`,
  that's our own `.expect` message: another program (often an earlier copy of this server, still
  running in another terminal) holds port 3000. Stop it with `Ctrl+C` in its terminal, or see
  **Common mistakes**. (The `code` number differs between systems: 48 on macOS, 98 on Linux.)
- ❌ If curl says `Failed to connect to 127.0.0.1 port 3000 … Couldn't connect to server` (on some
  systems it says `Connection refused`), the server isn't running, or it's on a different
  port. Check the first terminal shows `Listening on http://127.0.0.1:3000`.
- ❌ If `cargo run` says ``could not find `Cargo.toml` in … or any parent directory``, you're not
  in the `code/` folder (or a folder inside it). `cd` into it and try again.

### Step 7: where SeaORM and Postgres come in

Today's handler returns fixed text, so it needs neither SeaORM nor Postgres. Real handlers need
data. From Part A3 on, the ShopRS product handler will follow this flow. There's **no code for it
yet**, on purpose: first you'll learn SQL in Part A1 and Axum properly in Part A2.

```text
GET /products/42
  → Axum calls the handler get_product, with id = 42
  → the handler asks SeaORM: "find the product with id 42"
  → SeaORM sends SQL to Postgres:  SELECT … FROM products WHERE id = 42
  → Postgres returns one row (or none)
  → SeaORM turns the row into a Rust struct
  → the handler returns it as JSON: 200 OK (or 404 Not Found if there was no row)
```

### Line by line

`GET /products/42` → `Axum calls the handler get_product, with id = 42`
- **What:** the Router matches the path and pulls the `42` out of it for the handler.
- **Why:** the same handler serves every product; only the id changes.
- **How:** Part A2 shows how a handler asks Axum for pieces of the request.
- **Remove it and…** you'd need one handler per product.

`the handler asks SeaORM` → `SeaORM sends SQL to Postgres`
- **What:** the handler describes what it wants in Rust; SeaORM writes the SQL.
- **Why:** Rust code stays Rust code, and the compiler can check it.
- **How:** Part A3 teaches SeaORM; Part A1 teaches the SQL it writes, so nothing is a mystery.
- **Remove it and…** (SeaORM) you'd write the SQL yourself as text, which the book also teaches.

`Postgres returns one row` → `SeaORM turns the row into a Rust struct`
- **What:** Postgres answers with a table row; SeaORM fills in a Rust `struct` from it.
- **Why:** your handler works with normal Rust values (`product.price`), not raw database rows.
- **How:** that translation between rows and structs is what an ORM is for.
- **Remove it and…** you'd copy each column into your struct by hand.

`the handler returns it as JSON`
- **What:** the struct becomes a JSON body, with `200 OK`, or `404 Not Found` if there was no row.
- **Why:** it's the response from the last lesson, now with real data.
- **How:** the same `IntoResponse` idea as Step 3, for JSON instead of plain text.
- **Remove it and…** the client gets no data.

## You might be wondering…

**"Why does `main` need to be `async` at all?"**
Because the two things it does, claiming a port (`bind`) and serving (`serve`), are async: they
return futures you must `.await`, and `.await` only works inside async code. Rust doesn't allow an
async `main` by itself, because *something* must start the runtime before any async code can run.
`#[tokio::main]` is that something: it writes the ordinary `main` that starts Tokio for you.

**"Why `127.0.0.1` and not `0.0.0.0`?"**
`127.0.0.1` means "only this computer": no other machine can reach your server, even on the same
Wi-Fi. That's the safe choice while learning. `0.0.0.0` means "every network this computer is
connected to", which is what a server inside a [Docker](../glossary.md#docker)
[container](../glossary.md#container) or on a real host needs so the outside world can reach it.
You'll switch to it when you deploy in Part B.

**"Why does the program never end?"**
Because it's a server: its whole job is to wait for the next request, forever. `axum::serve` only
returns if something breaks. Stop it with `Ctrl+C` when you're done. That's also why you need two
terminals: one is busy being the server.

**"Why SeaORM and not raw SQL?"**
Both are good, and the book teaches both. Raw SQL is precise and you'll learn it first, in Part A1.
SeaORM saves you from writing the same `SELECT` and `INSERT` text a hundred times, and it lets the
compiler check your queries match your Rust structs. For unusual queries, SeaORM lets you drop down
to raw SQL, and a Part A3 lesson shows how.

**"I don't have the book's `code/` folder. Can I build this myself?"**
Yes. Create a project with `cargo new tour-of-the-stack`, paste the code from Steps 3 to 5 into
`src/main.rs` (the `use` line first, then `hello`, `app` and `main`), and replace its `Cargo.toml`
with this stand-alone version, which writes the versions in directly instead of using
`.workspace = true`:

```toml
[package]
name = "tour-of-the-stack"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
axum = "0.8.9"
tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread", "net"] }

[dev-dependencies]
tower = { version = "0.5.3", features = ["util"] }
http-body-util = "0.1.5"
```

Then run `cargo run` inside that folder (no `-p` needed: it's the only project).

## Coming from another language?

The four lines that matter look much the same everywhere. Here is the same "Hello" server in
Express (Node.js), Flask (Python) and Go's `net/http` (the Go lines go inside `func main()`, with
`fmt` and `net/http` imported; the `"GET /"` pattern needs Go 1.22 or newer):

```js
const express = require("express");
const app = express();

app.get("/", (req, res) => res.send("Hello from Express!"));

app.listen(3000, "127.0.0.1");
```

```python
from flask import Flask

app = Flask(__name__)

@app.route("/")
def hello():
    return "Hello from Flask!"

app.run(host="127.0.0.1", port=3000)
```

```go
http.HandleFunc("GET /", func(w http.ResponseWriter, r *http.Request) {
    fmt.Fprint(w, "Hello from Go!")
})
http.ListenAndServe("127.0.0.1:3000", nil)
```

Side by side with the Axum lines:

| Idea | Express | Flask | Go | Axum |
|---|---|---|---|---|
| Handler | `(req, res) => res.send(…)` | `def hello(): return …` | `func(w, r) { fmt.Fprint(w, …) }` | `async fn hello() -> &'static str { … }` |
| Route | `app.get("/", …)` | `@app.route("/")` | `http.HandleFunc("GET /", …)` | `Router::new().route("/", get(hello))` |
| Listen and serve | `app.listen(3000, "127.0.0.1")` | `app.run(host="127.0.0.1", port=3000)` | `http.ListenAndServe("127.0.0.1:3000", nil)` | `TcpListener::bind(…).await` + `axum::serve(…).await` |

Three differences to notice. Like Flask's, an Axum handler **returns** its answer instead of
writing into a `res` (Express) or `w` (Go) object, and Axum builds the response. You start the
async runtime yourself, with `#[tokio::main]`; Node and Go have theirs built in. And if a handler doesn't
fit what Axum expects, you find out when you *compile*, not when the first request arrives.

## Common mistakes

The file paths in these messages start at the project's own folder (`src/main.rs`); run from
`code/`, the same errors say `topics/tour-of-the-stack/src/main.rs`.

**Forgetting `.await`.**
Delete the first `.await` in `main` (after `bind(…)`) and build:

```text
error[E0599]: no method named `expect` found for opaque type `impl Future<Output = Result<tokio::net::TcpListener, std::io::Error>>` in the current scope
  --> src/main.rs:21:10
   |
20 |       let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
   |  ____________________-
21 | |         .expect("port 3000 is busy: stop the other program using it, or change the port");
   | |         -^^^^^^ method not found in `impl Future<Output = Result<tokio::net::TcpListener, std::io::Error>>`
   | |_________|
   |
   |
help: consider `await`ing on the `Future` and calling the method on its `Output`
   |
21 |         .await.expect("port 3000 is busy: stop the other program using it, or change the port");
   |          ++++++
```

Read the type in the message: `impl Future<Output = Result<…>>`. Without `.await` you're holding
the *future* (the promise of a result), not the `Result`, and a future has no `.expect`. **Fix:**
do what the `help:` line says and put `.await` back before `.expect`.

**Calling the handler: `get(hello())`.**

```text
error[E0277]: the trait bound `impl Future<Output = &'static str>: Handler<_, _>` is not satisfied
   --> src/main.rs:13:34
    |
 13 |     Router::new().route("/", get(hello()))
    |                              --- ^^^^^^^ the trait `Handler<_, _>` is not implemented for `impl Future<Output = &'static str>`
    |                              |
    |                              required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-0.8.9/src/routing/method_routing.rs:167:16
…
```

`hello()` *calls* `hello` right now, and since it's async, that gives a future, not a function.
`get` wants something it can call later, for every request: a **handler**. **Fix:** remove the
parentheses, `get(hello)`.

**The port is already in use.**

```text
thread 'main' (3233257) panicked at topics/tour-of-the-stack/src/main.rs:22:10:
port 3000 is busy: stop the other program using it, or change the port: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Our `.expect` message, followed by the operating system's reason. Most often it's the same server
still running in another terminal. **Fix:** stop the other copy with `Ctrl+C`. If you can't find
it, `lsof -i :3000` (macOS and Linux) shows which program has the port, as in
[Your toolbox](your-toolbox.md). Or give your server another port (see **More examples**).

**A missing Tokio feature.**
Take `rt-multi-thread` out of the `tokio` line's features in `code/Cargo.toml` and build:

```text
error: The default runtime flavor is `multi_thread`, but the `rt-multi-thread` feature is disabled.
  --> src/main.rs:18:1
   |
18 | #[tokio::main]
   | ^^^^^^^^^^^^^^
   |
   = note: this error originates in the attribute macro `tokio::main` (in Nightly builds, run with -Z macro-backtrace for more info)
```

`#[tokio::main]` builds the multi-threaded runtime unless told otherwise, and that runtime is behind
the `rt-multi-thread` feature. Now take out `macros` instead. In *this* project, nothing breaks,
because Axum switches on Tokio's `macros` feature for itself (a crate can turn on features in the
crates it uses). In a project that uses Tokio **without** Axum, such as a command-line tool,
removing `macros` gives:

```text
error[E0433]: cannot find `main` in `tokio`
 --> src/main.rs:1:10
  |
1 | #[tokio::main]
  |          ^^^^ could not find `main` in `tokio`

error[E0752]: `main` function is not allowed to be `async`
 --> src/main.rs:2:1
  |
2 | async fn main() {
  | ^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`
```

The attribute doesn't exist without the feature, so the second error follows from the first.
**Fix:** list every feature you use yourself, `["macros", "rt-multi-thread", "net"]`, rather than
counting on another crate to switch it on for you.

## More examples

Try each one in the crate: change `src/main.rs`, run `cargo run -p tour-of-the-stack` again (stop
the old server with `Ctrl+C` first), and curl it. Each one is also a complete program in
`code/topics/tour-of-the-stack/examples/`, which you can run as it is with
`cargo run -p tour-of-the-stack --example <name>` (for example `--example about`).

### A second page: `GET /about`

A second route is one more handler and one more `.route(…)`.

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/about.rs:about}}
```

```text
$ curl -i http://127.0.0.1:3000/about
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 55
date: Sat, 26 Sep 2026 07:35:13 GMT

ShopRS: a small online shop, built while learning Rust.
```

Each `.route` returns the Router with one more rule, so you chain them one after another.

### Text built at run time: return a `String`

When the text isn't fixed, build it with `format!` and return an owned `String` instead of
`&'static str`.

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/greet.rs:greet}}
```

Add `.route("/greet", get(greet))` to `app()`:

```text
$ curl -i http://127.0.0.1:3000/greet
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 42
date: Sat, 26 Sep 2026 07:35:13 GMT

Hello, Ada! You have 3 items in your cart.
```

`format!` fills each `{name}` with the variable of that name
([Rust for Humans: formatting](https://open-source-bd.github.io/rustbook-for-human/language-basics/formatting-with-format.html)).
A `String` gets the same `200 OK` plain-text response as a `&'static str`.

### Choose the status code: `(StatusCode::CREATED, "made")`

Return a pair (a *tuple*) of a status code and a body, and Axum uses your status instead of `200`.

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/status-code.rs:imports}}

{{#include ../../code/topics/tour-of-the-stack/examples/status-code.rs:handler}}
```

Add `.route("/things", post(make_thing))` to `app()`. `post` works like `get`, for `POST`
requests:

```text
$ curl -i -X POST http://127.0.0.1:3000/things
HTTP/1.1 201 Created
content-type: text/plain; charset=utf-8
content-length: 4
date: Sat, 26 Sep 2026 07:35:13 GMT

made
```

`StatusCode::CREATED` is `201`, the "created" code from the last lesson. A `GET` to `/things` gets
`405 Method Not Allowed`, because this route only accepts `POST`.

### Another port: 8080

Change the address in two places in `main`, the `bind` and the message (and the port in the
`.expect` text, so it stays honest):

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/other-port.rs:bind}}
```

```text
$ curl -i http://127.0.0.1:8080/
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 16
date: Sat, 26 Sep 2026 07:35:24 GMT

Hello from Axum!
```

Now curl must say `:8080`; port 3000 has nobody listening. Changing the port to `3001` is also the
quickest way around a busy port.

## Your turn

### 🟢 Guided

Make the server greet you by name. In `src/main.rs`, change one line of `hello` so it returns
`"Hello from ____!"` with your name in the blank. Stop the server (`Ctrl+C`), run it again, and
curl it.

<details><summary>Solution</summary>

Change the text inside `hello` (here for someone called Ada):

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/hello-name.rs:hello}}
```

Stop the old server with `Ctrl+C`, then `cargo run -p tour-of-the-stack` again. A running program
never picks up changes to its source; you must rebuild and restart it. Then:

```text
$ curl -i http://127.0.0.1:3000/
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 15
date: Sat, 26 Sep 2026 07:35:13 GMT

Hello from Ada!
```

`content-length` changed from 16 to 15, because `Ada` is one letter shorter than `Axum`. (The
test `root_says_hello` at the bottom of the file checks for the old text, so `cargo test` now
fails. Change the text in the test too if you want it to pass.)

</details>

### 🟡 Tweak

Add a second endpoint, `GET /health`, that returns `"ok"`. Real backends have one like it, so that
monitoring tools can ask "are you alive?". Then add a test for it: open `src/main.rs`, find
`root_says_hello` inside `mod tests` at the bottom, and copy its pattern.

<details><summary>Solution</summary>

Add the handler and its route:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/health.rs:health}}
```

And, inside `mod tests`, a test for it:

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/health.rs:health_test}}
```

How the test works, line by line:

- `#[tokio::test]` is `#[tokio::main]`'s twin for tests: it runs this async test on a Tokio
  runtime (it's why the `macros` feature is on).
- `Request::builder().uri("/health").body(Body::empty()).unwrap()` builds a pretend `GET /health`
  request with an empty body. (`GET` is the default method.)
- `app().oneshot(request).await.unwrap()` hands that one request straight to the Router, with no
  port and no network. This is why `app()` is its own function.
- `assert_eq!(response.status(), StatusCode::OK)` checks the status is `200`.
- The last two lines read the whole body into bytes and check it's exactly `ok`. The `b"ok"` is
  the text `ok` written as bytes, so the two can be compared.

Run the tests from the `code/` folder:

```bash
cargo test -p tour-of-the-stack
```

```text
running 3 tests
test tests::unknown_path_is_404 ... ok
test tests::root_says_hello ... ok
test tests::health_says_ok ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The three tests run at the same time, so their order can change.) And against the real server:

```text
$ curl -i http://127.0.0.1:3000/health
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 2
date: Sat, 26 Sep 2026 07:34:57 GMT

ok
```

</details>

### 🔴 From scratch

Add `GET /time`, which returns the number of whole seconds since the **Unix epoch**, midnight
(UTC) on 1 January 1970, the moment most computers count time from. Use `std::time::SystemTime`
from Rust's standard library; nothing new goes in `Cargo.toml`.

<details><summary>Solution</summary>

```rust,noplayground
{{#include ../../code/topics/tour-of-the-stack/examples/time.rs:use_time}}

{{#include ../../code/topics/tour-of-the-stack/examples/time.rs:time}}
```

and in `app()`, `.route("/time", get(time))`.

```text
$ curl -i http://127.0.0.1:3000/time
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 10
date: Sat, 26 Sep 2026 07:35:13 GMT

1790408113
```

Your number will be bigger: it goes up by one every second.

- `use std::time::{SystemTime, UNIX_EPOCH};` brings in the clock type and the constant for
  1 January 1970. Put it at the top of the file, next to the `use axum…` line.
- `SystemTime::now()` reads the computer's clock.
- `.duration_since(UNIX_EPOCH)` measures the time from 1970 until now. It returns a `Result`,
  because it fails if the clock is set *before* 1970. `.expect(…)` handles that case, with a
  message that says what went wrong.
- `.as_secs()` turns the length of time into whole seconds, a `u64` (a whole number that can't be
  negative).
- `seconds.to_string()` turns the number into text. The handler returns a `String`, because Axum
  knows how to turn text into a response. A bare `u64` isn't one of the types it accepts.

</details>

## Quick check

<div class="quiz" data-topic="tour-of-the-stack"></div>

## Remember this

- **Tokio** runs the async code, **Axum** routes each request to a handler, **SeaORM** turns
  database rows into Rust structs, and **Postgres** stores the data. A request passes through them
  in that order, and the response comes back the same way.
- A handler is an `async fn` that returns something Axum can turn into a response; a `&'static str`
  becomes `200 OK` with a plain-text body.
- `Router::new().route("/", get(hello))` pairs a method and path with a handler. Pass `hello`, not
  `hello()`: Axum calls it later, once per request.
- `#[tokio::main]` starts the Tokio runtime so `main` can be async; `TcpListener::bind(…).await`
  claims the port, and `axum::serve(…).await` serves forever (`Ctrl+C` to stop).
- Versions are set once in `code/Cargo.toml`; each project says `.workspace = true`.

## Go deeper

- [Rust for Humans: Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)
- [Rust for Humans: Tokio runtime and tasks](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/tokio-runtime-and-tasks.html)
- [Rust for Humans: Web services](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/web-services.html)
- [Rust for Humans: Strings and &str](https://open-source-bd.github.io/rustbook-for-human/abstractions/strings-and-str.html)
- [Rust for Humans: Workspaces and crates](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/workspaces-and-crates.html)
- [Rust for Humans: cfg and feature flags](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/cfg-and-feature-flags.html)
- [Axum docs](https://docs.rs/axum/0.8.9/axum/) — Official API reference.
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) — How the async runtime works.
- [SeaORM docs](https://www.sea-ql.org/SeaORM/docs/index/) — Official guide.
- [`#[tokio::main]` docs](https://docs.rs/tokio/1.53.1/tokio/attr.main.html) — Exactly what the
  attribute expands to.

<!-- next:start -->

**Next:**

- [What is a database?](../a1-postgres/what-is-a-database.md)

<!-- next:end -->
