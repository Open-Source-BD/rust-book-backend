# Testing handlers

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can test an Axum app without starting a server.
- You can split an app into a library and a thin `main` so tests can import it.
- You can test JSON, shared state and error responses.

## What & why

Test handlers with `tower::ServiceExt::oneshot`: a lib/main split, a tests/ folder, and tests for JSON, state and errors.

In every lesson so far, you checked your server by hand. Start it in one terminal, send `curl`
requests from a second, read each answer with your own eyes, press `Ctrl+C`. That is a
[test](../glossary.md#test): you give the code a known input and compare its answer with the one
you expect. It works, but it has three problems. It's slow: half a minute for a handful of checks.
It's forgettable: change one line in a handler, and will you remember to re-run every `curl`
command from three lessons ago? And it lives in your head: nobody else can run "the checks Ada
does on Tuesdays".

Think of a mechanic checking a car. They could drive it around town and listen. Or they could put
it on a test bench in the garage: the engine runs, the wheels turn, every gauge is read, and the
car never touches the road. That's what this lesson does for your app. A test hands a request
straight to your Router, reads the response, and compares it with what it should be, with no port,
no network and no second terminal. It takes milliseconds, so you run it every time you change
something. And the book's automatic checks (often called *CI*, continuous integration) run every
test in this book on every change pushed to GitHub.

You've been reading tests for a while: every lesson's `src/main.rs` ends with a `#[cfg(test)]`
block, and [Hello, Axum](hello-axum.md) promised a later lesson would explain them properly. This is
that lesson. Thank you for your patience. Three new ideas:

1. **A library and a thin `main`.** The app moves into `src/lib.rs`, a library that other code can
   import. `src/main.rs` shrinks to a few lines that start the server.
2. **`oneshot`: one request in, one response out, no network.** A method from the Tower crate
   that drives your Router like a function call, entirely in memory.
3. **A `tests/` folder.** Tests that live outside your code and use the library the way any other
   program would. Rust calls them [integration tests](../glossary.md#integration-test). We test a
   plain-text answer, JSON, shared [state](../glossary.md#state), and an error.

## The idea, slowly

### Step 1: three files, two jobs

Earlier lessons kept everything in one file, `src/main.rs`. This lesson's crate has three:

```text
code/topics/testing-handlers/
├── Cargo.toml
├── src/
│   ├── lib.rs     the app: types, handlers, app()    →  the library  testing_handlers
│   └── main.rs    a thin main: serve app()            →  the program  testing-handlers
└── tests/
    └── api.rs     tests that use the library, like any outside program
```

### Line by line

`code/topics/testing-handlers/` · `Cargo.toml`
- **What:** the crate's folder, and its settings file.
- **Why:** one folder per lesson, as always.
- **How:** Step 2 shows `Cargo.toml`. Notice what's **not** in it: no line says "there's a library"
  or "there's a `tests/` folder". Cargo finds them by their names and places.
- **Remove it and…** (`Cargo.toml`) Cargo doesn't know this folder is a project.

`src/lib.rs  the app: types, handlers, app()  →  the library  testing_handlers`
- **What:** everything the app is made of: the `Note` type, the handlers, the state, and `app()`,
  which builds the Router.
- **Why:** code in a **library** can be imported by other code. The tests need `app()`, so `app()`
  must live in a library.
- **How:** when a crate has a file called `src/lib.rs`, Cargo builds it as a library. The library
  is named after the package, with `-` turned into `_`: `testing-handlers` becomes
  `testing_handlers`, because a `-` isn't allowed in a Rust name. Inside it, the rule from
  [Nesting and modular routers](nesting-and-modular-routers.md) applies: items are private unless
  marked `pub`, and only `pub` items can be used from outside (Rust for Humans:
  [Visibility and privacy](https://open-source-bd.github.io/rustbook-for-human/language-basics/visibility-and-privacy.html)).
- **Remove it and…** the tests have nothing to import, and `main.rs` has no `app()` to serve.

`src/main.rs  a thin main: serve app()  →  the program  testing-handlers`
- **What:** the program you start with `cargo run`. It opens port 3000 and serves the library's
  `app()`.
- **Why:** a library can't run by itself; something must open the port. Keeping that part tiny
  means almost everything worth testing is in the library.
- **How:** when a crate has **both** `src/lib.rs` and `src/main.rs`, Cargo builds two things from
  one package: the library, and a program that uses it. `main.rs` is the library's first outside
  user: it can only reach `pub` items, by the name `testing_handlers`.
- **Remove it and…** `cargo run -p testing-handlers` fails: there's no program to run. The tests
  still pass, since they never start the server.

`tests/  api.rs  tests that use the library, like any outside program`
- **What:** a folder of integration tests. Each `.rs` file in
  it is built as its own small program, which `cargo test` runs.
- **Why:** a test here sees your app from the outside, the way a real program would: through its
  `pub` door, never its private rooms. If the test can do it, so can any program that uses your
  library.
- **How:** the folder must be called `tests`, next to `src`, and Cargo finds it on its own. Each
  file can `use testing_handlers::…` without any extra line in `Cargo.toml`.
- **Remove it and…** `cargo test` still runs, and finds no tests to run.

> **Why did earlier lessons put the tests inside `main.rs`?** A crate with only `src/main.rs` has
> no library, so a `tests/` folder would have nothing to import. Tests inside a file, in a
> `#[cfg(test)] mod tests` block, need no library: they're part of that file. *You might be
> wondering…* compares the two kinds.

### Step 2: the crate's `Cargo.toml`

Nothing new to install. One line is in a new place, though: `serde_json`, which Input validation
listed under `[dependencies]`, is a dev-dependency here.

```toml
{{#include ../../code/topics/testing-handlers/Cargo.toml}}
```

The versions and features are written once, in the workspace's `code/Cargo.toml`:

```toml
{{#include ../../code/Cargo.toml:test_deps}}
{{#include ../../code/Cargo.toml:serde_json}}
```

### Line by line

`[package]` · `name = "testing-handlers"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo test -p testing-handlers`. It's also where
  the library's name, `testing_handlers`, comes from.
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
  [JSON and serde](json-and-serde.md).
- **Why:** the app itself needs them: Axum for the Router and `Json`, Tokio to run, serde to turn
  notes into JSON and back.
- **How:** these are built for `cargo run` **and** `cargo test`.
- **Remove it and…** (`serde`) ``error[E0432]: unresolved import `serde` ``.

`[dev-dependencies]` · `tower.workspace = true` · `tower = { version = "0.5.3", features = ["util"] }`
- **What:** the Tower crate, version 0.5.3, with its `util` feature, for the tests only.
- **Why:** Tower gives the `oneshot` method that sends one request to the Router (Step 5).
- **How:** `[dev-dependencies]` are built only for `cargo test` (and for examples), never for the
  server you `cargo run`. The `util` feature switches on the part of Tower that has `oneshot`.
- **Remove it and…** `cargo run` still works, but the tests fail with
  ``error[E0432]: unresolved import `tower` ``, then ``no method named `oneshot` found for struct
  `Router<S>` ``.

`http-body-util.workspace = true` · `http-body-util = "0.1.5"`
- **What:** a small crate of tools for response bodies, for the tests only.
- **Why:** it gives `.collect()`, which reads a whole response body into memory (Step 5).
- **How:** a body can arrive in pieces; `.collect()` waits for every piece and joins them.
- **Remove it and…** ``error[E0432]: unresolved import `http_body_util` ``.

`serde_json.workspace = true` · `serde_json = "1.0.151"`
- **What:** the crate that reads and writes JSON text, for the tests only.
- **Why:** the tests turn the JSON the app sends back into a `Note`, with `serde_json::from_str`
  (Step 7). The app never calls `serde_json` itself: Axum's `Json` does that for it.
- **How:** in [Input validation](input-validation.md), `serde_json` was a normal dependency,
  because the app's own code called `serde_json::to_value`. Here only the tests need it, so it's a
  dev-dependency, and the server you ship doesn't carry it.
- **Remove it and…** the tests fail with ``error[E0433]: cannot find module or crate `serde_json`
  in this scope``.

In your own project, the same lines come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, and the test crates from
`cargo add --dev tower http-body-util serde_json --features tower/util`. `--dev` is what puts them
under `[dev-dependencies]`. If you start with `cargo new`, you get `src/main.rs` only; create
`src/lib.rs` next to it yourself, and a `tests/` folder next to `src/`.

### Step 3: the library, `src/lib.rs`

Here's the whole app: a small notes service with a health check. If
[Shared state](shared-state.md) is fresh in your mind, most of it will look familiar.

```rust,noplayground
{{#include ../../code/topics/testing-handlers/src/lib.rs:lib}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo run -p testing-handlers`

### Line by line

`use axum::{Json, Router, extract::State, http::StatusCode, routing::get};`
- **What:** the five names this file takes from Axum.
- **Why:** `Json` for JSON bodies, `Router` for the routes, `State` for the shared list,
  `StatusCode` for `201 Created`, and `get` for `GET` routes.
- **How:** all met before: `State` in Shared state, `Json` in JSON and serde.
- **Remove it and…** (`State`) ``cannot find tuple struct or tuple variant `State` in this
  scope``, once per handler.

`use serde::{Deserialize, Serialize};` · `use std::sync::{Arc, Mutex};`
- **What:** serde's two derive names, and the two standard-library types that make a shared list.
- **Why:** notes travel as JSON both ways; the list must be shared safely between requests.
- **How:** as in Shared state:
  [Arc and Mutex](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/shared-state-mutex-and-arc.html)
  are part of Rust itself.
- **Remove it and…** (`Arc, Mutex`) ``cannot find type `Arc` in this scope``.

`#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]`
- **What:** asks for five abilities to be written for `Note`: copy it (`Clone`), turn it into JSON
  (`Serialize`), build it from JSON (`Deserialize`), print it for a programmer (`Debug`), and
  compare two notes with `==` (`PartialEq`).
- **Why:** the server needs only `Clone` and `Serialize`. The last three are **for the tests**:
  Step 7 reads the JSON answer back into a `Note` (`Deserialize`), then checks it with
  `assert_eq!`, which compares with `==` (`PartialEq`) and, when they differ, prints both
  (`Debug`).
- **How:** each name in `derive(…)` writes one piece of code for you (Rust for Humans:
  [Derive traits](https://open-source-bd.github.io/rustbook-for-human/language-basics/derive-traits.html)).
  Deriving abilities a test needs is normal and costs nothing at run time.
- **Remove it and…** (`PartialEq`) the tests fail with ``error[E0369]: binary operation `==`
  cannot be applied to type `Note` ``. (`Debug`) ``error[E0277]: `Note` doesn't implement
  `Debug` ``. (`Deserialize`) ``the trait bound `Note: serde::Deserialize<'de>` is not
  satisfied``. The server itself builds fine in all three cases.

`pub struct Note {` · `pub id: u32,` · `pub text: String,` · `}`
- **What:** one note: a number and its text. `pub` on the struct **and** on each field.
- **Why:** the tests, living outside the library, name `Note`, build one with
  `Note { id: 1, … }`, and read `notes[1].text`. Each of those needs the matching `pub`.
- **How:** `pub struct` lets outsiders **name** the type; `pub` on a field lets them **see and set**
  that field. They're separate choices.
- **Remove it and…** (the struct's `pub`) ``error[E0603]: struct `Note` is private``, in
  `tests/api.rs`. (the fields' `pub`) ``error[E0616]: field `text` of struct `Note` is private``.

`#[derive(Deserialize)]` · `pub struct NewNote {` · `pub text: String,` · `}`
- **What:** what a client sends to create a note: only the text. The server picks the `id`.
- **Why:** the same "one struct for each direction" idea as in
  [JSON and serde, Step 3](json-and-serde.md#step-3-two-structs-one-for-each-direction).
- **How:** the tests send this as JSON text, so they never use `NewNote` by name. The `pub` lets
  other programs build one; take it away, and everything here still compiles.
- **Remove it and…** (`Deserialize`) `Json<NewNote>` can't be built, and `add_note` stops being a
  handler: the `Handler<_, _>` error you've met before.

`#[derive(Clone, Default)]` · `pub struct AppState {` · `notes: Arc<Mutex<Vec<Note>>>,` · `}`
- **What:** the app's state: one shared, lockable list of notes.
  `Default` writes `AppState::default()`, which starts with an empty list.
- **Why:** the field `notes` has **no** `pub`. Only code inside `lib.rs` can touch the list, so
  outsiders, tests included, must go through the HTTP routes, as a real client would.
- **How:** exactly as in [Shared state, Step 4](shared-state.md#step-4-the-state). `Clone` here
  copies the `Arc`, a pointer, not the list, so every clone shares one list. Step 8 relies on that.
- **Remove it and…** (`Default`) ``no associated function or constant named `default` found for
  struct `AppState` ``. (`Clone`) a pile of ``the trait bound `AppState: Clone` is not satisfied``
  errors: Axum hands each request its own clone of the state.

`async fn list_notes(State(state): State<AppState>) -> Json<Vec<Note>> {` · `Json(state.notes.lock().unwrap().clone())` · `}`
- **What:** `GET /notes`: lock the list, copy it, send the copy as a JSON array.
- **Why:** the copy lets the lock go at once, before the JSON is written.
- **How:** as in Shared state. No `pub`: the handler is the library's private business; outsiders
  reach it through `app()`.
- **Remove it and…** (`.clone()`) ``error[E0308]: mismatched types``, ``expected `Vec<Note>`, found
  `MutexGuard<'_, Vec<Note>>` ``: that's the lock, not a list you can send.

`async fn add_note(` · `State(state): State<AppState>,` · `Json(input): Json<NewNote>,` · `) -> (StatusCode, Json<Note>) {`
- **What:** `POST /notes`: receives the state and the parsed body, and answers with a status and a
  `Note` as JSON.
- **Why:** creating something answers `201 Created` and shows what was made.
- **How:** `Json` is last because it reads the body, as in JSON and serde.
- **Remove it and…** (`Json(input): …`) there's no text to save.

`let mut notes = state.notes.lock().unwrap();` · `let note = Note {` · `id: notes.len() as u32 + 1,` · `text: input.text,` · `};`
- **What:** lock the list, then build the new note. Its `id` is "how many notes there are, plus one".
- **Why:** the first note gets `1`, the second `2`. Tests can predict that, which is what Step 7
  checks.
- **How:** `notes.len()` is a `usize`; `as u32` turns it into the field's type.
- **Remove it and…** (`as u32`) ``error[E0308]: mismatched types``: expected `u32`, found `usize`.

`notes.push(note.clone());` · `(StatusCode::CREATED, Json(note))` · `}`
- **What:** store a copy in the list, and send the original back with `201`.
- **Why:** the list and the response each need their own `Note`.
- **How:** the lock is released when `notes` goes out of scope, at the closing `}`.
- **Remove it and…** (`.clone()`) ``use of moved value: `note` ``: `push` took it.

`pub fn app() -> Router {`
- **What:** the one door into the library: a function that builds the whole app.
- **Why:** this is the only item `main.rs` and the tests need, so it's the item that **must** be
  `pub`. Everything behind it can stay private.
- **How:** `Router` here means `Router<()>`, a Router whose state is already filled in, so it's
  ready to serve, or to test.
- **Remove it and…** (`pub`) ``error[E0603]: function `app` is private``, in both `main.rs` and
  `tests/api.rs`. *Common mistakes* shows it.

`Router::new()` · `.route("/health", get(|| async { "ok" }))`
- **What:** `GET /health` answers `ok`.
- **Why:** the simplest route there is, for the simplest test. Real services have one, so that
  monitoring tools can ask "are you alive?".
- **How:** the handler is a [closure](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html)
  written in place: `|| async { "ok" }` takes nothing and returns `"ok"`.
- **Remove it and…** `GET /health` answers `404`, and the first test fails.

`.route("/notes", get(list_notes).post(add_note))` · `.with_state(AppState::default())` · `}`
- **What:** `GET /notes` lists, `POST /notes` adds; then the Router gets its state, one empty list.
- **Why:** **each call to `app()` makes a brand-new, empty list.** Keep that in mind for Step 8.
- **How:** `.with_state` turns a `Router<AppState>` into a `Router<()>`, as in
  [Shared state, Step 6](shared-state.md#step-6-the-router-with-with_state).
- **Remove it and…** (`.with_state(…)`) ``error[E0308]: mismatched types``: the function promised a
  ready `Router`, and this one still waits for its state.

### Step 4: the program, `src/main.rs`

The whole of `src/main.rs`:

```rust,noplayground
{{#include ../../code/topics/testing-handlers/src/main.rs:main}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo run -p testing-handlers`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's start, running on the Tokio runtime.
- **Why:** `main` needs `.await`, and `#[tokio::main]` is what allows that.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line of `main`.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`let listener = …bind("127.0.0.1:3000")…;` · `println!(…);`
- **What:** claims port 3000, then prints `Listening on http://127.0.0.1:3000`.
- **Why:** a server must own a port before anyone can reach it.
- **How:** word for word the `main` of every Axum lesson.
- **Remove it and…** there's no port for curl to reach.

`axum::serve(listener, testing_handlers::app())` · `.await` · `.expect(…);`
- **What:** serves the library's `app()`.
- **Why:** this is the one line that changed: `app()` lives in the library now, so `main.rs` names
  it with the library's name in front.
- **How:** `testing_handlers::app()` is a path: "the `app` inside the crate `testing_handlers`". It
  works without any `use` line or `Cargo.toml` entry: a crate's program can always reach its own
  library. Note the `_`: write `testing-handlers::app()` and Rust reads it as a subtraction,
  `testing` minus `handlers::app()`, and fails with ``cannot find value `testing` in this scope``.
- **Remove it and…** (writing `app()` alone) ``error[E0425]: cannot find function `app` in this
  scope``: `main.rs` is a separate file from `lib.rs`, and doesn't see its items by itself.

So `main.rs` is now ten short lines that will hardly ever change. Everything worth testing is in the
library.

### Step 5: the test helpers

Now the tests, in `tests/api.rs`. The file starts with its `use` lines and three small helper
functions that every test below uses:

```rust,noplayground
{{#include ../../code/topics/testing-handlers/tests/api.rs:helpers}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo test -p testing-handlers`

### Line by line

`use axum::{` · `Router,` · `body::Body,` · `http::{Request, StatusCode},` · `};`
- **What:** four names from Axum: `Router`, `Body` (the body of a request or response), `Request`
  (a whole HTTP request) and `StatusCode`.
- **Why:** the helpers take a `Router`, build `Request<Body>`s, and give back a `StatusCode`.
- **How:** `Request<Body>` reads "a request whose body is a `Body`". `http::` is the crate of HTTP
  types that Axum is built on, passed through so you don't need it in `Cargo.toml`.
- **Remove it and…** (`Body`) ``cannot find type `Body` in this scope``.

`use http_body_util::BodyExt;`
- **What:** brings in the trait that gives a body its `.collect()` method.
- **Why:** to read the whole response body in `send`.
- **How:** a method that comes from a
  [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html) only
  works where the trait is in scope; the `use` line puts it there. `Ext` is short for "extension":
  it **extends** a type you didn't write with new methods.
- **Remove it and…** a puzzling ``error[E0599]: `Body` is not an iterator``. Rust looked for some
  other `.collect()` it knew. Read on, though: the compiler's `help:` says ``trait `BodyExt` which
  provides `collect` is implemented but not in scope; perhaps you want to import it``.

`use testing_handlers::{Note, app};`
- **What:** the two items the tests take from **our** library: the `Note` type and `app()`.
- **Why:** this line is why the lib/main split exists. A test in `tests/` is an outsider, and this
  is how an outsider imports a library: by its name, `testing_handlers`.
- **How:** it only works because `Note` and `app` are `pub`. `AppState` and the handlers aren't,
  and the tests don't need them.
- **Remove it and…** ``cannot find function `app` in this scope`` in every test, and the same for
  `Note`.

`use tower::ServiceExt;`
- **What:** the trait that gives a Router its `.oneshot()` method.
- **Why:** `oneshot` is the heart of every test here.
- **How:** another "extension" trait, from the Tower crate. Remember from
  [Middleware and Tower layers](middleware-and-tower-layers.md#you-might-be-wondering) that your
  Router **is** a Tower *service*: something that takes a request and returns a response.
  `ServiceExt` adds handy methods to every service, and `oneshot` is one of them.
- **Remove it and…** ``error[E0599]: no method named `oneshot` found for struct `Router<S>` in the
  current scope``.

`async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {`
- **What:** a helper: give it an app and a request, and it gives back the status and the body as
  text.
- **Why:** every test does those same four steps. Written once, each test shrinks to one line of
  sending and a few lines of checking.
- **How:** it's an ordinary `async fn`, not a test: no `#[tokio::test]` above it, so `cargo test`
  doesn't run it by itself. It takes the `Router` **by value**, because `oneshot` needs to own it
  (Step 8 explains why that matters).
- **Remove it and…** each test repeats the four lines below.

`let response = app.oneshot(request).await.unwrap();`
- **What:** sends **one** request into the Router and waits for its **one** response. This is the
  test bench from *What & why*.
- **Why:** no port, no `curl`, no network: the request is a value in memory, handed to the Router
  the way `axum::serve` hands it one after reading it from the network. Same routing, same
  extractors, same handlers, same layers. Only the network is skipped.
- **How:** `oneshot` means "one shot": it uses up the service for a single request. It first waits
  until the Router is ready, then calls it with `request`, and gives back a
  [future](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)
  that `.await` turns into a `Result`. For a Router, that `Result` is never an `Err` (Axum's
  routers can't fail at this level, they answer `404` or `500` instead), so `.unwrap()` takes the
  response out safely.
- **Remove it and…** (`.await`) ``error[E0308]: mismatched types``: you'd be holding a promise of
  a response, not a response.

`let status = response.status();`
- **What:** copies the status code out of the response.
- **Why:** the next line takes the body, and taking the body uses up the response; we save the
  status first.
- **How:** `StatusCode` is a small number type, so this is a plain copy.
- **Remove it and…** there's no status to give back.

`let bytes = response.into_body().collect().await.unwrap().to_bytes();`
- **What:** reads the whole body into memory, as raw bytes.
- **Why:** a response body is a **stream**: it can arrive in pieces, and a test wants all of it
  at once to compare it.
- **How:** read it left to right. `.into_body()` takes the body out of the response. `.collect()`
  (from `BodyExt`) waits for every piece; `.await` lets it wait. `.unwrap()` is safe here: this
  app's bodies are already complete in memory, so reading them can't fail. `.to_bytes()` joins the pieces into one block of bytes.
- **Remove it and…** (`.to_bytes()`) ``no method named `to_vec` found for struct `Collected<B>` ``
  on the next line: you'd have the pile of pieces, not one block.

`(status, String::from_utf8(bytes.to_vec()).unwrap())` · `}`
- **What:** turns the bytes into text, and returns the pair.
- **Why:** comparing with `"ok"` or reading JSON is easiest with a `String`.
- **How:** `String::from_utf8` checks the bytes are valid UTF-8 text (every answer in this app is),
  and returns a `Result`; `.unwrap()` takes the `String` out, or stops the test if the body wasn't
  text.
- **Remove it and…** (`.unwrap()`) ``mismatched types``: the function promised a `String`, not a
  `Result`.

`fn get(uri: &str) -> Request<Body> {` · `Request::get(uri).body(Body::empty()).unwrap()` · `}`
- **What:** builds a `GET` request for an address such as `/health`, with an empty body.
- **Why:** this is the in-memory version of `curl -i http://127.0.0.1:3000/health`. There's no
  `http://127.0.0.1:3000` part: the request never travels anywhere, so it needs only the path.
- **How:** `Request::get(uri)` starts a request *builder* with the method and address filled in.
  `.body(…)` finishes it, and returns a `Result`, because the builder only now checks what you gave
  it, such as an address with a space in it. `.unwrap()` takes the request out. Older lessons'
  tests wrote `Request::builder().uri(…)`: the same thing, with `GET` as the default method.
- **Remove it and…** (`.unwrap()`) ``error[E0308]: mismatched types``: a `Result`, where a
  `Request<Body>` was promised.

`fn post_json(uri: &str, body: &str) -> Request<Body> {` · `Request::post(uri)`
- **What:** builds a `POST` request that carries JSON.
- **Why:** this is the in-memory version of `curl -X POST … -H 'content-type: application/json'
  -d '…'`.
- **How:** `Request::post` is `Request::get`'s twin for `POST`.
- **Remove it and…** you'd repeat the next three lines in every test that sends JSON.

`.header("content-type", "application/json")`
- **What:** labels the body as JSON, like `curl`'s `-H`.
- **Why:** Axum's `Json` refuses a body without this label, as you saw in
  [JSON and serde](json-and-serde.md#run-it).
- **How:** the builder adds the header and hands itself back, so the next call can follow.
- **Remove it and…** every `POST` gets `415 Unsupported Media Type`, and the three tests that post
  notes fail.

`.body(Body::from(body.to_owned()))` · `.unwrap()` · `}`
- **What:** puts the JSON text in the body, and finishes the request.
- **Why:** `body` is a borrowed `&str`, and a `Body` must own its bytes: the request may live
  longer than the text it was built from.
- **How:** `.to_owned()` makes an owned `String` copy, and `Body::from` wraps it. (`Body::from`
  does accept a `&str` directly, but only text written in the program itself, which lives
  forever.)
- **Remove it and…** (`.to_owned()`) ``error[E0521]: borrowed data escapes outside of function``:
  the borrowed text isn't allowed to go into the body.

### Step 6: the first test

```rust,noplayground
{{#include ../../code/topics/testing-handlers/tests/api.rs:first_test}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo test -p testing-handlers`

### Line by line

`#[tokio::test]`
- **What:** marks the function below as a test, and runs it on a Tokio runtime.
- **Why:** this test is `async` (it `.await`s `send`), and an async function needs a runtime to
  run, the way an async `main` needs `#[tokio::main]`. Rust's plain `#[test]` can't run async
  functions.
- **How:** it's `#[tokio::main]`'s twin for tests, and comes from the same Tokio `macros` feature.
  Each test gets its own small runtime, with one thread.
- **Remove it and…** the function is no longer a test: `cargo test` skips it, with only a warning.
  *Common mistakes* shows this, and the error you get for `#[test]` instead.

`async fn health_says_ok() {`
- **What:** the test. Its name is what `cargo test` prints, so it says what's being checked.
- **Why:** when a test fails months from now, its name is the first thing you read. `health_says_ok`
  tells you what broke before you open the file.
- **How:** a test takes nothing and returns nothing. It **passes** if it reaches the end, and
  **fails** if anything inside panics, such as an `assert_eq!` that doesn't hold.
- **Remove it and…** there's no test.

`let (status, body) = send(app(), get("/health")).await;`
- **What:** build a fresh app, send it `GET /health`, and keep the status and body.
- **Why:** this one line replaces "start the server, switch terminals, type the `curl` command".
- **How:** `app()` builds a whole new Router, with its own empty state, for this test alone. The
  `let (status, body) = …` pattern opens the pair that `send` returns.
- **Remove it and…** there's nothing to check.

`assert_eq!(status, StatusCode::OK);` · `assert_eq!(body, "ok");` · `}`
- **What:** the checks: the status must be `200 OK`, and the body exactly `ok`.
- **Why:** these are the two things you read with your eyes in every *Run it*. Now the computer
  reads them for you.
- **How:** `assert_eq!(left, right)` compares the two with `==`. Equal: nothing happens, and the test
  goes on. Different: the test stops and fails, printing both values. A `String` compared with
  `"ok"` works, because Rust knows how to compare the two.
- **Remove it and…** the test passes no matter what the app answers: a test with no checks tests
  nothing.

### Step 7: testing a JSON answer

```rust,noplayground
{{#include ../../code/topics/testing-handlers/tests/api.rs:json_test}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo test -p testing-handlers`

### Line by line

`let (status, body) = send(app(), post_json("/notes", r#"{"text":"buy milk"}"#)).await;`
- **What:** posts the note `buy milk` to a fresh app.
- **Why:** this is `curl -X POST … -d '{"text":"buy milk"}'`, done in memory.
- **How:** `r#"…"#` is a **raw string**: inside it, `"` is an ordinary character, so the JSON needs
  no `\"`. Without it, you'd write `"{\"text\":\"buy milk\"}"`.
- **Remove it and…** there's no new note to check.

`assert_eq!(status, StatusCode::CREATED);`
- **What:** the answer must be `201 Created`.
- **Why:** it proves the request reached `add_note`, and not some error.
- **How:** if the body had no `text`, this would fail with `left: 422` and `right: 201`.
- **Remove it and…** an error answer might slip through to the next line, with a more confusing
  failure.

`let note: Note = serde_json::from_str(&body).unwrap();`
- **What:** reads the JSON text back into a real `Note`.
- **Why:** comparing JSON **text** is fragile: `{"id":1,"text":"buy milk"}` and
  `{"text":"buy milk","id":1}` mean the same, but aren't the same text. Comparing **values** is
  what we mean.
- **How:** `serde_json::from_str` is the reverse of what `Json(note)` did in the handler. The
  `: Note` after the name tells it what to build; that's why `Note` derives `Deserialize`. `.unwrap()` stops the test
  if the text isn't a valid `Note`.
- **Remove it and…** (`: Note`) ``error[E0283]: type annotations needed``: `from_str` can build many
  types, and Rust can't guess which.

`assert_eq!(` · `note,` · `Note {` · `id: 1,` · `text: "buy milk".to_string()` · `}` · `);` · `}`
- **What:** the note must be exactly `Note { id: 1, text: "buy milk" }`: both fields at once.
- **Why:** it's the first note in a fresh app, so its `id` must be `1`.
- **How:** this needs two of `Note`'s derives: `PartialEq` for the `==` inside `assert_eq!`, and
  `Debug` to print both notes if they differ. `"buy milk".to_string()` makes a `String`, the field's
  type.
- **Remove it and…** the test checks the status only, and a note saved with the wrong text would
  pass.

### Step 8: testing shared state

The trickiest test, and the one that answers "what's that `.clone()` doing in every lesson's
tests?":

```rust,noplayground
{{#include ../../code/topics/testing-handlers/tests/api.rs:state_test}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo test -p testing-handlers`

### Line by line

`let app = app();`
- **What:** builds **one** app, with one empty list, and keeps it in a variable.
- **Why:** this test checks memory: what one request saves, a later request must see. That only
  works if every request goes to the **same** app, the way every `curl` went to the same server.
- **How:** the variable is called `app` too; from here on, `app` means this Router, not the
  function. Rust allows that, and the next lines read naturally.
- **Remove it and…** (calling `app()` in each line instead) every request gets its own new, empty
  list. The test fails; *You might be wondering…* shows the real failure.

`send(app.clone(), post_json("/notes", r#"{"text":"one"}"#)).await;` · `send(app.clone(), post_json("/notes", r#"{"text":"two"}"#)).await;`
- **What:** posts two notes, each through a **clone** of the app.
- **Why:** `oneshot` uses up the Router it's given, so the original can't be handed over: it would
  be gone after the first request. A clone is a second handle on the **same** app, and it's the
  clone that gets used up.
- **How:** cloning a Router is cheap: inside, it's an
  [`Arc`](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/shared-state-mutex-and-arc.html),
  a shared pointer, so a clone copies a pointer, not the routes. The state inside holds its list in
  an `Arc` as well, so every clone points at the **one** list. The status and body are ignored
  here (the result isn't stored): the last request checks the effect.
- **Remove it and…** (`.clone()`) ``error[E0382]: use of moved value: `app` ``. *Common mistakes*
  shows it.

`let (_, body) = send(app, get("/notes")).await;`
- **What:** the last request lists the notes, and the original `app` itself goes in.
- **Why:** nothing needs `app` after this line, so there's no need to clone it.
- **How:** `_` means "I don't need this part": the status is thrown away.
- **Remove it and…** there's nothing to check.

`let notes: Vec<Note> = serde_json::from_str(&body).unwrap();`
- **What:** reads the JSON array back into a list of `Note`s.
- **Why:** the same reason as Step 7: compare values, not text.
- **How:** `Vec<Note>` tells `from_str` to expect an array of notes.
- **Remove it and…** there's no list to count.

`assert_eq!(notes.len(), 2);` · `assert_eq!(notes[1].text, "two");` · `}`
- **What:** there must be two notes, and the second (counting from `0`) must say `two`.
- **Why:** two notes means the list was shared across three requests; `two` in second place means
  they're kept in the order they arrived.
- **How:** `notes[1].text` works from outside the library because `text` is `pub`.
- **Remove it and…** the test sends requests and checks nothing.

### Step 9: testing an error

Errors are answers too, and worth testing: a client relies on getting `422`, not `500`, for bad
input.

```rust,noplayground
{{#include ../../code/topics/testing-handlers/tests/api.rs:error_test}}
```

📁 Full code: code/topics/testing-handlers · ▶ Run it: `cd code && cargo test -p testing-handlers`

### Line by line

`#[tokio::test]` · `async fn bad_json_is_rejected() {`
- **What:** a test that sends a note without its `text` field.
- **Why:** it checks the unhappy path, which is easy to forget when checking by hand.
- **How:** the same shape as every test here.
- **Remove it and…** nothing warns you if a later change starts accepting bad input, or answers
  it with a `500`.

`let (status, _) = send(app(), post_json("/notes", r#"{"words":"oops"}"#)).await;`
- **What:** posts JSON that's well-formed but has `words` where a `NewNote` needs `text`.
- **Why:** this is the "right grammar, wrong shape" case from
  [JSON and serde, Step 5](json-and-serde.md#step-5-what-happens-to-a-json-body).
- **How:** `_` throws away the body: the plain-text message is Axum's to word, and could change in a
  later Axum. The status is the promise this app makes.
- **Remove it and…** nothing is sent.

`assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);` · `}`
- **What:** the answer must be `422 Unprocessable Entity`.
- **Why:** `Json` refuses the body before `add_note` runs, so no note is saved and the client hears
  why.
- **How:** `UNPROCESSABLE_ENTITY` is `422`'s name in `StatusCode`.
- **Remove it and…** the test sends bad input and checks nothing.

### Step 10: a request on the test bench, as a picture

Here are the two ways of checking, side by side:

```text
by hand:   curl ──▶ network ──▶ port 3000 ──▶ axum::serve ──▶ Router ──▶ handler
in a test: send(app(), get("/health")) ─────────────────────▶ Router ──▶ handler
                     .oneshot(request): no curl, no port, no network
```

### Line by line

`by hand:   curl ──▶ network ──▶ port 3000 ──▶ axum::serve ──▶ Router ──▶ handler`
- **What:** what happened in every *Run it* so far: `curl` sends text over the network to port
  3000, `axum::serve` reads it and turns it into a `Request`, then hands it to the Router.
- **Why:** this is how real clients reach your app, so it's worth trying by hand, once.
- **How:** it needs two terminals, a free port, and a running server.
- **Remove it and…** (any piece) no answer: a server that isn't running can't reply.

`in a test: send(app(), get("/health")) ─────────────────────▶ Router ──▶ handler`
- **What:** a test builds the `Request` value itself and hands it **straight** to the Router.
- **Why:** everything from the Router onward is the same code as in production: routing,
  extractors, layers, handlers. That's the part you wrote, and the part worth testing.
- **How:** the long arrow skips the network, the port and `axum::serve`. Those are Axum's and
  Tokio's code, tested by their own authors.
- **Remove it and…** you're back to checking by hand.

`.oneshot(request): no curl, no port, no network`
- **What:** the method that makes the short cut.
- **Why:** with no port, tests can run side by side without fighting over port 3000, and they never
  wait for a network.
- **How:** `oneshot` calls the Router directly, as a function, and gets the `Response` value
  back.
- **Remove it and…** a test would have to start a real server and send real network requests,
  which is slower and needs a free port.

### Run it

There's no server to start. From the `code/` folder:

```bash
cd code && cargo test -p testing-handlers
```

```text
   Compiling testing-handlers v0.1.0 (/Users/you/rust-book-backend/code/topics/testing-handlers)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.18s
     Running unittests src/lib.rs (target/debug/deps/testing_handlers-59d17d50b023ce52)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/testing_handlers-33ac46eb1f578869)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/api.rs (target/debug/deps/api-1621518921069762)

running 4 tests
test health_says_ok ... ok
test a_new_note_comes_back_as_json ... ok
test bad_json_is_rejected ... ok
test notes_are_remembered_between_requests ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests testing_handlers

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Read it from the top:

- **`Compiling` and `Finished`.** Cargo builds the library, the program and the tests, in the
  `test` profile (a build for checking, not for shipping). The paths in brackets, with
  their long hex endings, are the built test programs; yours will differ.
- **`Running unittests src/lib.rs` → `running 0 tests`.** Cargo checks the library for tests
  **inside** it, in a `#[cfg(test)]` block. This library has none, so: zero, and `ok`.
- **`Running unittests src/main.rs` → `running 0 tests`.** The same, for the program.
- **`Running tests/api.rs` → `running 4 tests`.** Our file, built as its own program. One line per
  test, with its name and `ok`. Your order may differ: the tests run **at the same time**, on
  separate threads, and are listed as they finish.
- **`test result: ok. 4 passed; 0 failed`.** The line to look for. `finished in 0.00s`: four
  tests, six requests in all, in under five thousandths of a second.
- **`Doc-tests testing_handlers` → `running 0 tests`.** Rust can also run code examples written in
  a library's documentation comments (`///`). We wrote none.

To run only some tests, add part of a name: `cargo test -p testing-handlers health` runs only
`health_says_ok`, and its summary counts the other three as `3 filtered out`.

Now the same four checks by hand, for comparison. Use the two-terminal routine from
[Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it). In the first
terminal:

```bash
cd code && cargo run -p testing-handlers
```

In the second terminal:

```bash
{{#include ../../code/topics/testing-handlers/http/01-by-hand.sh}}
```

```text
{{#include ../../code/topics/testing-handlers/http/01-by-hand.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides. Every
answer is the one a test expects: `ok`, a `201` with `"id":1`, two notes in the list, and a `422`.
One difference is worth spotting: here the second note has `"id":2`, and the list holds `buy milk`
**and** `two`, because one server kept everything. Each test starts from its own fresh `app()`, so
in `notes_are_remembered_between_requests`, the notes are `one` and `two`. When you're done, press
**`Ctrl+C`** in the first terminal to stop the server.

- ✅ If you see `test result: ok. 4 passed`, every check passed, without a server.
- ❌ If you see ``error[E0603]: function `app` is private``, `app` has lost its `pub` (*Common
  mistakes*).
- ❌ If a test says `FAILED`, read its `panicked at` line: it names the file, the line and the two
  values that differed (*You might be wondering…* shows one).

## You might be wondering…

**"Unit tests or a `tests/` folder?"**
Both, for different jobs. A test inside a file, in a `#[cfg(test)] mod tests` block, is a **unit
test**: it's part of that file, so it can reach the file's private items too, which suits checking
one small piece from the inside. A file in `tests/` holds **integration tests**: outsiders that see
only `pub` items, which suits checking that your app works as others will use it. For an HTTP app,
the public door is `app()`, and nearly every useful test goes through it, so `tests/` is a natural
home. The earlier lessons used unit tests only because they had no library. Rust for Humans covers
both: [Unit testing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/unit-testing.html)
and [Integration testing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/integration-testing.html).

**"Does `oneshot` start a server?"**
No. Nothing listens on a port, and nothing goes over the network. `oneshot` calls your Router the
way you'd call a function, with a `Request` value, and gets a `Response` value back. You can run
these tests with a server already running on port 3000, or with no network at all, and they behave
the same.

**"Why `.clone()` the app? Why not call `app()` each time?"**
`oneshot` takes the Router by value, and uses it up. Its definition in Tower says so:
`fn oneshot(self, req: Request)`, where `self` (not `&self`) means "I take ownership". So after
`app.oneshot(…)`, `app` is gone. Calling `app()` again gives you a Router that works, but it's a
**different** app with a **new**, empty list. Here's what happens if Step 8 does that, captured in a
scratch copy:

```text
running 4 tests
test health_says_ok ... ok
test a_new_note_comes_back_as_json ... ok
test bad_json_is_rejected ... ok
test notes_are_remembered_between_requests ... FAILED

failures:

---- notes_are_remembered_between_requests stdout ----

thread 'notes_are_remembered_between_requests' (16633237) panicked at tests/api.rs:55:5:
assertion `left == right` failed
  left: 0
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    notes_are_remembered_between_requests

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test api`
```

This is what a failing test looks like, so it's worth reading slowly. `panicked at
tests/api.rs:55:5` is the file, line and column of the `assert_eq!` that failed (line 55 in that
scratch copy is `assert_eq!(notes.len(), 2)`). `left: 0` is what the code produced: the third
`app()` had an empty list. `right: 2` is what the test expected. The number in brackets after the
test's name is its thread's ID, different on every run. The last line tells you how to re-run only
this file. A `.clone()` instead gives a second handle on the **same** app, and the same list:
`left` becomes `2`, and the test passes.

**"How do I test with a database?"**
These tests are fast partly because the notes live in memory. Once Part A3 stores them in Postgres,
each test also needs a database to talk to, with known data in it. *Testing with SeaORM*, in Part A3,
covers that. The `oneshot` part stays exactly the same.

**"Should I test every route?"**
Test what you'd be upset to see break: each route's happy path, and the errors a client relies on,
such as a `404` for a missing book or a `422` for bad input. A test is cheap to add the moment you
write the code, and expensive to recreate from memory later. When you find a bug, write a test that
fails because of it first, then fix the code: that bug can never quietly come back.

## Coming from another language?

Every framework has a way to send a pretend request to your app. The question is whether it goes
through the network (slower, needs a port) or straight in, like `oneshot`.

**Express (Node.js).** The usual pair is the Jest test runner and the supertest library. The same
split as ours appears: `app.js` builds the app, and `server.js` is the thin main that calls
`app.listen(3000)`.

```js
// app.test.js — run with: npx jest
const request = require("supertest");
const app = require("./app"); // app.js ends with: module.exports = app;

test("health says ok", async () => {
  const res = await request(app).get("/health");
  expect(res.status).toBe(200);
  expect(res.text).toBe("ok");
});
```

One difference: supertest does use the network. If your app isn't listening yet, it starts it on a
free temporary port behind the scenes, sends a real HTTP request, and stops it again.

**FastAPI and Flask (Python).** Both have a test client that calls the app in memory, like
`oneshot`. With FastAPI and pytest:

```python
from fastapi.testclient import TestClient
from main import app

client = TestClient(app)

def test_a_new_note_comes_back_as_json():
    response = client.post("/notes", json={"text": "buy milk"})
    assert response.status_code == 201
    assert response.json() == {"id": 1, "text": "buy milk"}
```

`response.json()` reads the body into a Python value, like our `serde_json::from_str`, and
`assert ==` is our `assert_eq!`. Flask's version is `client = app.test_client()`, then
`response = client.get("/health")` and `response.get_data(as_text=True) == "ok"`.

**Spring Boot (Java).** `MockMvc` sends requests to your controllers without starting a real
server:

```java
@WebMvcTest(NotesController.class)
class NotesControllerTest {
    @Autowired MockMvc mvc;

    @Test
    void healthSaysOk() throws Exception {
        mvc.perform(get("/health"))
           .andExpect(status().isOk())
           .andExpect(content().string("ok"));
    }
}
```

`@WebMvcTest` starts only the web part of the app; `get`, `status` and `content` are static imports
from Spring's test classes. To test against a real running server instead, Spring has
`@SpringBootTest(webEnvironment = WebEnvironment.RANDOM_PORT)`.

**Go (`net/http` and Gin).** The standard library's `httptest` package is the closest match to
`oneshot`: it calls the handler directly, with a recorder in place of the network.

```go
func TestHealthSaysOK(t *testing.T) {
    req := httptest.NewRequest(http.MethodGet, "/health", nil)
    rec := httptest.NewRecorder()
    router().ServeHTTP(rec, req)
    if rec.Code != http.StatusOK {
        t.Fatalf("status = %d, want 200", rec.Code)
    }
    if rec.Body.String() != "ok" {
        t.Fatalf("body = %q, want ok", rec.Body.String())
    }
}
```

`router()` is your app-building function, like our `app()`. It works the same with Gin: a
`*gin.Engine` has a `ServeHTTP` method too.

## Common mistakes

The compiler errors and outputs below were captured in a scratch copy of this lesson's code, with
the `// ANCHOR` lines removed, so their line numbers differ a little from the book's files.

**Forgetting `pub` on `app`.**
In `lib.rs`, the function is written without `pub`:

```rust,noplayground,ignore
fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/notes", get(list_notes).post(add_note))
        .with_state(AppState::default())
}
```

```text
error[E0603]: function `app` is private
  --> tests/api.rs:7:30
   |
 7 | use testing_handlers::{Note, app};
   |                              ^^^ private function
   |
note: the function `app` is defined here
  --> src/lib.rs:38:1
   |
38 | fn app() -> Router {
   | ^^^^^^^^^^^^^^^^^^
…
```

The same error appears for `src/main.rs:7:45`, at `testing_handlers::app()`. Four warnings come
first, the last of them ``warning: function `app` is never used``: inside the library, nothing
calls the private `app`, and nobody outside is allowed to (the other three follow from it: the
handlers and the list are only used through `app`). Inside the same file, a private function
is fine, which is why the earlier lessons' tests, inside `main.rs`, never needed `pub`. A test in
`tests/` is a different crate, an outsider. **Fix:** `pub fn app()`. Make `pub` exactly what the
tests and `main.rs` need: here `app` and `Note`.

**Forgetting `#[tokio::test]`.**
Out of habit from plain Rust tests, you write `#[test]` on an async test:

```rust,noplayground,ignore
#[test]
async fn health_says_ok() {
    let (status, body) = send(app(), get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}
```

```text
error: async functions cannot be used for tests
  --> tests/api.rs:29:1
   |
29 |   async fn health_says_ok() {
   |   ^----
   |   |
   |  _`async` because of this
   | |
30 | |     let (status, body) = send(app(), get("/health")).await;
31 | |     assert_eq!(status, StatusCode::OK);
32 | |     assert_eq!(body, "ok");
33 | | }
   | |_^
…
```

Plain `#[test]` knows how to call a normal function, but an `async fn` only **describes** work; it
needs a runtime to actually run it. That's a loud mistake. The quiet one is leaving the attribute
out altogether:

```rust,noplayground,ignore
async fn health_says_ok() {
    let (status, body) = send(app(), get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}
```

```text
warning: function `health_says_ok` is never used
  --> tests/api.rs:28:10
   |
28 | async fn health_says_ok() {
   |          ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
…
running 3 tests
test notes_are_remembered_between_requests ... ok
test bad_json_is_rejected ... ok
test a_new_note_comes_back_as_json ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
…
```

It builds, and everything is `ok`. But it says `3 tests`, not 4: without an attribute, the
function is an ordinary function nobody calls, and the only sign is a warning. A missing test can't
fail. **Fix:** `#[tokio::test]` on every async test. And read your warnings, and the test count.

**Reusing the app after `oneshot`, without `.clone()`.**
In Step 8, you hand `app` itself to the first two requests:

```rust,noplayground,ignore
let app = app();
send(app, post_json("/notes", r#"{"text":"one"}"#)).await;
send(app, post_json("/notes", r#"{"text":"two"}"#)).await;
let (_, body) = send(app, get("/notes")).await;
```

```text
error[E0382]: use of moved value: `app`
  --> tests/api.rs:53:10
   |
51 |     let app = app();
   |         --- move occurs because `app` has type `Router`, which does not implement the `Copy` trait
52 |     send(app, post_json("/notes", r#"{"text":"one"}"#)).await;
   |          --- value moved here
53 |     send(app, post_json("/notes", r#"{"text":"two"}"#)).await;
   |          ^^^ value used here after move
   |
note: consider changing this parameter type in function `send` to borrow instead if owning the value isn't necessary
  --> tests/api.rs:10:20
   |
10 | async fn send(app: Router, request: Request<Body>) -> (StatusCode, String) {
   |          ----      ^^^^^^ this parameter takes ownership of the value
   |          |
   |          in this function
help: consider cloning the value if the performance cost is acceptable
   |
52 |     send(app.clone(), post_json("/notes", r#"{"text":"one"}"#)).await;
   |             ++++++++
…
```

A second, identical error follows for line 54. The first request **moved** `app` into `send`, so
line 53 has nothing left to send. The compiler offers two ideas. The `note:` suggests making `send`
borrow (`&Router`), but that only moves the problem: inside `send`, `oneshot` needs to own the
Router, and Rust then says ``cannot move out of `*app` which is behind a shared reference``. The
`help:` is the real fix, and costs almost nothing, since cloning a Router copies a pointer. **Fix:**
`app.clone()` for every request except the last.

## More examples

The first three live in `code/topics/testing-handlers/examples/` and test this lesson's library: an
example can import the library the same way `tests/api.rs` does. These examples are only tests, so
their `main` prints how to run them; run each with the `cargo test` command shown. Each test sits
inside a `mod tests` block in its file, which is why it's indented.

### Checking a header

A response is more than a status and a body. Here the tests check the `content-type` header
(`cargo test -p testing-handlers --example th-headers`):

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-headers.rs:test}}
```

These tests don't use `send`, because `send` throws the headers away; they keep the whole
`response`. `response.headers()[header::CONTENT_TYPE]` looks a header up by name, and
`header::CONTENT_TYPE` is Axum's ready-made name for `content-type`, so a typo can't creep in. The
`/notes` list is labelled `application/json` (`Json` did that); `/health`'s `"ok"` is
`text/plain; charset=utf-8`, the same values `curl -i` showed in *Run it*.

### A helper that gives back JSON

When most answers are JSON, a helper can return the body already parsed, as a
`serde_json::Value`, a JSON value of any shape
(`cargo test -p testing-handlers --example th-json-value`):

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-json-value.rs:helper}}

{{#include ../../code/topics/testing-handlers/examples/th-json-value.rs:test}}
```

`serde_json::from_slice` reads JSON from bytes, so the `String` step disappears. A `Value` needs no
struct: `body["text"]` reaches into it by key, and compares with `"buy milk"` directly. The
`json!` macro writes a `Value` in JSON's own syntax, so the last line compares the **whole** answer
in one go. Handy for testing an API whose types you don't want to import, or a response with no
struct at all, such as Input validation's error lists.

### One test, many URLs

When several cases share the same check, a **table** of cases and one loop beat four copies of the
same test (`cargo test -p testing-handlers --example th-table`):

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-table.rs:test}}
```

Each row is a pair: an address, and the status it must get. `/note` (no `s`) and `/health/` (an
extra `/`) are near-misses, and must be `404`. The `"GET {uri}"` after the two values is a message
printed if the check fails, because with a loop, the line number alone doesn't tell you **which**
row broke. Here's a scratch copy with one wrong row added, `("/notes/", StatusCode::OK)`:

```text
running 1 test
test tests::every_url_gets_the_right_status ... FAILED

failures:

---- tests::every_url_gets_the_right_status stdout ----

thread 'tests::every_url_gets_the_right_status' (16637068) panicked at examples/th-table.rs:28:13:
assertion `left == right` failed: GET /notes/
  left: 404
 right: 200
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::every_url_gets_the_right_status

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--example th-table`
```

`failed: GET /notes/` names the row at once. One weakness to know: the loop stops at the first
failing row, so a second broken row only shows up after you fix the first.

### Testing that a slow route times out

In [Middleware and Tower layers](middleware-and-tower-layers.md#step-6-the-router-and-its-layers),
a `TimeoutLayer` answered `408` when a handler took too long. A test can check that promise, and
check it **quickly**. This example lives in the middleware lesson's folder, because that crate
already has the tower-http dependency
(`cargo test -p middleware-and-tower-layers --example mw-timeout-test`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-timeout-test.rs:app}}
```

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-timeout-test.rs:test}}
```

The handler would sleep for 10 seconds; the layer gives up after 200 milliseconds. The first test
checks two things: the status is `408 Request Timeout`, **and** it came back in under 2 seconds,
measured with `Instant::now()` and `.elapsed()` from Rust's standard library. That second check
proves the layer stopped the wait, rather than the handler finishing. A short limit keeps the test
fast: it takes about a fifth of a second. The second test makes sure the layer doesn't hurt a fast
route. Layers run inside `oneshot` exactly as they do on a server. By hand
(`cargo run -p middleware-and-tower-layers --example mw-timeout-test`):

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/53-timeout-test.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/53-timeout-test.out}}
```

## Your turn

### 🟢 Guided

Add a test that `GET /notes` on a fresh app answers `200` with an empty list. Put it in
`tests/api.rs`, after the others, using the `send` and `get` helpers.

<details><summary>Solution</summary>

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-notes-start-empty.rs:test}}
```

In your copy, this goes in `tests/api.rs`. The book keeps it in
`examples/th-notes-start-empty.rs` (with its own copy of the helpers), so the lesson's
`tests/api.rs` still has the four tests *Run it* shows.
`cargo test -p testing-handlers --example th-notes-start-empty`:

```text
   Compiling testing-handlers v0.1.0 (/Users/you/rust-book-backend/code/topics/testing-handlers)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.83s
     Running unittests examples/th-notes-start-empty.rs (target/debug/examples/th_notes_start_empty-8bcd96d23697c3dc)

running 1 test
test tests::notes_start_empty ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

An empty JSON array is the two characters `[]`, so comparing the text is safe here. The last two
lines say the same thing with values: read it as a `Vec<Note>`, and check `.is_empty()`. Either check
alone is enough. Notice the test's name in the output: `tests::notes_start_empty`, because in the
example it sits inside `mod tests`; in `tests/api.rs` it would be plain `notes_start_empty`.

</details>

### 🟡 Tweak

Add `DELETE /notes` to `lib.rs`: it empties the list and answers `204 No Content`. Then write a test
that posts two notes, deletes them, and checks that the list is empty. Hint: `Request::delete` builds
a `DELETE` request, and `MethodRouter` has a `.delete(…)` method, like `.post(…)`.

<details><summary>Solution</summary>

`examples/th-delete-notes.rs` is the whole `lib.rs` after the change, with a `main` and the test in
the same file. The new handler, and `app()` with its one changed line:

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-delete-notes.rs:handler}}
```

The test:

```rust,noplayground
{{#include ../../code/topics/testing-handlers/examples/th-delete-notes.rs:test}}
```

Run the test with `cargo test -p testing-handlers --example th-delete-notes`. Or try it by hand,
with `cargo run -p testing-handlers --example th-delete-notes` in the first terminal:

```bash
{{#include ../../code/topics/testing-handlers/http/91-delete-notes.sh}}
```

```text
{{#include ../../code/topics/testing-handlers/http/91-delete-notes.out}}
```

`clear_notes` takes the state, locks the list, and `.clear()`s it. A bare `StatusCode` is a complete
answer: `204 No Content` means "done, and there's nothing to send back", so the body is empty, and
the test checks that too (`body == ""`). `.delete(clear_notes)` joins the other two methods on the
same path. The test clones the app for every request but the last, as in Step 8. The book's copy
lives in an example because the lesson's `lib.rs` must stay as shown; in your copy, you change
`lib.rs` itself, and put the test in `tests/api.rs`.

</details>

### 🔴 From scratch

Write integration tests for the rules of [Input validation](input-validation.md): a good sign-up is
welcomed; each rule refuses its own bad value (a username of 2 or 21 characters, an email of `nope`,
an age of 12); the edges are allowed (an age of 13, a username of 20 characters); a form that breaks
all three rules gets all three back; and broken JSON never reaches the rules. Put the tests in a
`tests/` folder. You'll need to split that lesson's crate into a library and a thin `main` first.

<details><summary>Solution</summary>

The Input validation lesson's own crate stays as that lesson shows it, so the book does this split in
a copy of it: a separate crate, `code/topics/input-validation-split`, laid out like this lesson's.
`src/lib.rs` is the old `src/main.rs`, without `main` and without its `#[cfg(test)]` block, and with
**one** change:

```rust,noplayground
{{#include ../../code/topics/input-validation-split/src/lib.rs:handler}}
```

`pub fn app()`. That's all the tests need. `SignUp`, `ValidatedJson` and `sign_up` stay private: the
tests only send JSON text, as any client would. The new `src/main.rs` is this lesson's thin `main`,
with the library's name, `input_validation_split`:

```rust,noplayground
{{#include ../../code/topics/input-validation-split/src/main.rs:main}}
```

And the tests, `tests/sign_up_rules.rs`:

```rust,noplayground
{{#include ../../code/topics/input-validation-split/tests/sign_up_rules.rs}}
```

`cd code && cargo test -p input-validation-split`:

```text
   Compiling input-validation-split v0.1.0 (/Users/you/rust-book-backend/code/topics/input-validation-split)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.86s
     Running unittests src/lib.rs (target/debug/deps/input_validation_split-b778d326901b903b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/input_validation_split-50c2423205ce30dd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/sign_up_rules.rs (target/debug/deps/sign_up_rules-2d90a4174e2df67d)

running 5 tests
test broken_json_never_reaches_the_rules ... ok
test every_problem_comes_back_at_once ... ok
test a_good_sign_up_is_welcomed ... ok
test the_edges_of_each_rule_are_allowed ... ok
test each_rule_refuses_its_bad_value ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests input_validation_split

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

A few choices worth noticing:

- **`sign_up(body)`** builds the `POST` and sends it, all in one helper, since every test posts
  to the same address.
- **`bad_fields`** reads the error list as a `serde_json::Value` and returns only its keys: the
  names of the bad fields. The tests check **which** fields failed, not every byte of the message,
  so they won't break if validator rewords something. (`serde_json` is a normal dependency of this
  crate, as it was in Input validation, so the tests can use it too.) The `///` line above it is a
  documentation comment.
- **Tables** for the edges and the bad values, each row checked with its own body as the failure
  message. Testing **both sides** of each limit (12 refused, 13 allowed; 21 characters refused, 20
  allowed) is how you catch a `min = 14` typo, or a rule that was never there.
- **`["age", "email", "username"]`**, in alphabetical order. That order is reliable only because
  of the `serde_json::to_value` line in Input validation's Step 4; without it, this test would
  fail now and then. A test that sometimes fails is a sign of something random in the code, and
  here it would have failed on most runs, pointing straight at it.

The split didn't change what the server answers. With `cargo run -p input-validation-split` in the
first terminal:

```bash
{{#include ../../code/topics/input-validation-split/http/90-same-answers.sh}}
```

```text
{{#include ../../code/topics/input-validation-split/http/90-same-answers.out}}
```

Word for word the first two answers of Input validation's *Run it*.

</details>

## Quick check

<div class="quiz" data-topic="testing-handlers"></div>

## Remember this

- **Split the app:** `src/lib.rs` holds everything and a `pub fn app()`; `src/main.rs` only serves
  `your_crate::app()` (with `_`, not `-`). Then tests in `tests/` can import it.
- **`app().oneshot(request).await`** sends one request straight into the Router and gives back the
  response: no server, no port, no network. It needs `use tower::ServiceExt;`.
- Read the body with `.into_body().collect().await` (`use http_body_util::BodyExt;`), and check it
  with `assert_eq!`, or with `serde_json::from_str` into your type (derive `PartialEq` and `Debug`).
- **`oneshot` uses up the Router.** To share state across requests, build the app once and send
  `app.clone()`: clones share the same `Arc`'d state. A fresh `app()` starts empty.
- Async tests need **`#[tokio::test]`**. Without it, `#[test]` refuses an `async fn`, and no
  attribute at all skips the test with only a warning: check the count.

## Go deeper

- [Rust for Humans: Unit testing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/unit-testing.html)
- [Rust for Humans: Integration testing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/integration-testing.html)
- [tower::ServiceExt::oneshot](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot) — Official reference: the method that sends one request to a service and gives back its response.
- [Axum's testing example](https://github.com/tokio-rs/axum/tree/main/examples/testing) — Official example: more ways to test an Axum app, including against a real server on a random port.

<!-- next:start -->

**Next:**

- Build it: a Todo API (coming soon)
- Cheat sheet: Axum (coming soon)

<!-- next:end -->
