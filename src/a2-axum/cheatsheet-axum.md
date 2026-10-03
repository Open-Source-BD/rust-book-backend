# Cheat sheet: Axum

> **Intermediate** · Part A2 · Axum

Every Axum pattern from Part A2, on one page. Each one is a short "How do I…?" question, the Rust
that answers it, and a link to the lesson (and the step inside it) that explains it line by line.
Come back here when you remember *that* something exists but not *how* it's spelled.

All the snippets come from one small program, `code/topics/cheatsheet-axum`. Each pattern is a
[handler](../glossary.md#handler) (or two) plus a tiny `Router` of its own, and at the end of the
page one `app()` function joins them all into one server. The program has one
[test](../glossary.md#test) per pattern, and the book's checks compile it and run those tests on
every change. So every snippet on this page compiles, and does what the text says it does.

## Try every snippet

Run the tests from the book's `code/` folder:

```bash
cd code && cargo test -p cheatsheet-axum
```

```text
   Compiling cheatsheet-axum v0.1.0 (/Users/you/rust-book-backend/code/topics/cheatsheet-axum)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.97s
     Running unittests src/main.rs (target/debug/deps/cheatsheet_axum-9a562f36f4c42531)

running 14 tests
test tests::a_new_book_comes_back_as_json ... ok
test tests::return_status_and_headers ... ok
test tests::return_errors_with_question_mark ... ok
test tests::read_a_path_value ... ok
test tests::receive_and_send_json ... ok
test tests::share_state ... ok
test tests::add_middleware ... ok
test tests::start_a_server ... ok
test tests::read_the_query_string ... ok
test tests::write_a_custom_extractor ... ok
test tests::split_into_modules_and_nest ... ok
test tests::validate_input ... ok
test tests::route_by_method ... ok
test tests::add_cors_timeout_and_logging_layers ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
```

- ✅ `14 passed` means every pattern below works. The tests run at the same time, so their order
  changes from run to run.
- ✅ `finished in 1.01s`: one test waits a full second on purpose, to prove the timeout works.

To start the server itself and try a route with `curl`, run `cd code && cargo run -p
cheatsheet-axum`. It listens on `http://127.0.0.1:3000`, like every server in Part A2.

📁 Full code: code/topics/cheatsheet-axum · ▶ Run it: `cd code && cargo run -p cheatsheet-axum`

## The `use` lines

Every snippet below uses names from this one block at the top of `src/main.rs`. Nothing is hidden:
if a snippet says `Json`, `State` or `TimeoutLayer`, this is where it comes from.

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:imports}}
```

The crate's `Cargo.toml` asks the workspace for `axum`, `tokio`, `serde`, `serde_json`,
`thiserror`, `tower-http`, `tracing-subscriber` and `validator`, plus `tower` and `http-body-util`
for the tests. Each one is explained in the lesson that first needs it.

## How do I…?

### …start a server?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:start}}

{{#include ../../code/topics/cheatsheet-axum/src/main.rs:main}}
```

A handler is an `async fn` whose return value becomes the [response](../glossary.md#response).
`.route("/", get(hello))` connects the [route](../glossary.md#route) `GET /` to it. `main` opens
[port](../glossary.md#port) 3000 on `127.0.0.1` with a `TcpListener`, then `axum::serve` answers
every request that arrives there. `#[tokio::main]` starts the
[runtime](../glossary.md#runtime) that runs the `async` code. The `tracing_subscriber` lines print
the request log that `TraceLayer` (below) writes.

Explained in: [Hello, Axum](hello-axum.md) ([Step 6: `main`](hello-axum.md#step-6-main), and
[Step 7: start the server](hello-axum.md#step-7-start-the-server-then-talk-to-it)).

### …send different methods to different handlers?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:methods}}
```

Chain one method per [HTTP method](../glossary.md#http-method) on the same path:
`get(list_notes).post(add_note)`. A path that exists but doesn't accept the method (here `PATCH
/notes`) gets `405 Method Not Allowed`; a path that doesn't exist at all gets `404 Not Found`.
`{id}` is a path segment that can hold any value; [reading it](#read-a-value-from-the-path) is a
few items down.

Explained in: [Routes and HTTP methods](routes-and-methods.md)
([Step 5: one Router, two paths, five methods](routes-and-methods.md#step-5-one-router-two-paths-five-methods),
and [Step 7: 404 or 405?](routes-and-methods.md#step-7-404-or-405-how-axum-decides)).

### …choose the status code and add a header?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:status_headers}}
```

Return a tuple: the [status code](../glossary.md#status-code) first, then any
[headers](../glossary.md#header) as an array of `(name, value)` pairs, then the body last.
`impl IntoResponse` means "some type Axum can turn into a response", so you don't have to spell
the tuple's full type.

Explained in: [Handlers and IntoResponse](handlers-and-into-response.md)
([Step 4: choose the status code](handlers-and-into-response.md#step-4-choose-the-status-code),
[Step 5: add a header](handlers-and-into-response.md#step-5-add-a-header), and
[Step 6: the shape of a response tuple](handlers-and-into-response.md#step-6-the-shape-of-a-response-tuple)).

### …read a value from the path?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:path}}
```

Name the segment in the route with braces, `{id}`, and ask for it with the
[extractor](../glossary.md#extractor) `Path<u32>`. Axum turns the text into a `u32` for you. Two
segments come out as a tuple, in the order they appear in the path. `/books/abc` never reaches the
handler: `abc` isn't a number, so Axum answers `400 Bad Request` itself.

Explained in: [Path and Query extractors](path-and-query-extractors.md)
([Step 4: read a value from the path](path-and-query-extractors.md#step-4-read-a-value-from-the-path),
and [Step 5: what happens to `/books/42`](path-and-query-extractors.md#step-5-what-happens-to-books42)).

### …read the query string?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:query}}
```

Describe the [query string](../glossary.md#query-string) (`?q=rust&page=2`) as a struct with
`#[derive(Deserialize)]`, and ask for `Query<Search>`. A field that may be missing is an
`Option`. A required field that's missing (`/search` with no `q`) gets `400 Bad Request`.

Explained in: [Path and Query extractors](path-and-query-extractors.md)
([Step 6: read options from the query string](path-and-query-extractors.md#step-6-read-options-from-the-query-string)).

### …receive JSON and send JSON back?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:json}}
```

`Json<NewBook>` as an argument reads the request body as [JSON](../glossary.md#json) into the
struct; `Json<Book>` as the return value writes the struct out as JSON, with
`content-type: application/json`. The struct coming in needs `Deserialize`, the one going out
needs `Serialize`. A body with the wrong fields gets `422 Unprocessable Entity`.

Explained in: [JSON with serde](json-and-serde.md)
([Step 3: two structs, one for each direction](json-and-serde.md#step-3-two-structs-one-for-each-direction),
and [Step 4: the handlers, `Json` out and `Json` in](json-and-serde.md#step-4-the-handlers-json-out-and-json-in)).

### …share data between requests?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:state}}
```

Put the shared data in a struct that derives `Clone`, hand one to the Router with `.with_state`,
and ask for it in a handler with `State<AppState>`. Each request gets a clone of the struct, but
the `Arc` inside means every clone points at the same counter; the `Mutex` lets one request at a
time change it. That's [state](../glossary.md#state).

Explained in: [Shared state](shared-state.md)
([Step 4: the state](shared-state.md#step-4-the-state),
[Step 4½: `Arc<Mutex<Vec<Book>>>`, one layer at a time](shared-state.md#step-4½-arcmutexvecbook-one-layer-at-a-time),
and [Step 6: the Router, with `.with_state`](shared-state.md#step-6-the-router-with-with_state)).

### …return errors with `?`?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:errors}}
```

Make an [error type](../glossary.md#error-type) (an `enum`, with `thiserror` writing each
message), and teach Axum to send it by implementing `IntoResponse`: pick a status, put the message
in a JSON body. Then the handler returns `Result<…, AppError>`, and `?` stops it early with the
error. `/titles/9` answers `404` with `{"error":"book 9 not found"}`.

Explained in: [Error handling in Axum](error-handling-in-axum.md)
([Step 4: the error type](error-handling-in-axum.md#step-4-the-error-type),
[Step 5: teach Axum how to send an `AppError`](error-handling-in-axum.md#step-5-teach-axum-how-to-send-an-apperror),
and [Step 7: finding a book, one step at a time](error-handling-in-axum.md#step-7-finding-a-book-one-step-at-a-time)).

### …run my own code before and after every handler?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:middleware}}
```

That's [middleware](../glossary.md#middleware): an `async fn` that gets the request and `next`.
Code before `next.run(request).await` runs before the handler; code after it can change the
response. Here it adds `x-powered-by: axum` to every response, even a `404`. It's switched on by
the `.layer(middleware::from_fn(add_powered_by))` line in `app()`, at the
[end of this page](#join-all-the-pieces-into-one-app).

Explained in: [Middleware and Tower layers](middleware-and-tower-layers.md)
([Step 5: your own middleware](middleware-and-tower-layers.md#step-5-your-own-middleware)).

### …add CORS, a timeout and a request log?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:slow}}
```

Three ready-made [layers](../glossary.md#layer) from `tower-http`, all added in `app()` at the
[end of this page](#join-all-the-pieces-into-one-app):

- `TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(1))` gives
  each request one second. The `slow` handler above sleeps for three, so it gets
  `408 Request Timeout` (that's the [timeout](../glossary.md#timeout)).
- `CorsLayer::permissive()` adds `access-control-allow-origin: *`, which lets a web page from any
  website call this API ([CORS](../glossary.md#cors)). Fine while you learn; a real API lists its
  own websites instead.
- `TraceLayer::new_for_http()` logs every request and response. The `tracing_subscriber` lines in
  `main` print that log.

Layers wrap the Router like the skins of an onion: the **last** `.layer` is the outermost, so it
sees the request first and the response last.

Explained in: [Middleware and Tower layers](middleware-and-tower-layers.md)
([Step 6: the Router, and its layers](middleware-and-tower-layers.md#step-6-the-router-and-its-layers),
and [Step 7: the onion](middleware-and-tower-layers.md#step-7-the-onion)).

### …split routes into modules and put them under a prefix?

`src/authors.rs`, a [module](../glossary.md#module) with a Router of its own:

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/authors.rs:authors}}
```

And the line at the top of `src/main.rs` that makes the file part of the program:

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:modules}}
```

The module's routes are written without the prefix (`/` and `/{id}`). In `app()`, at the
[end of this page](#join-all-the-pieces-into-one-app), `.nest("/api/authors", authors::router())`
puts them under `/api/authors`, so they answer `/api/authors` and `/api/authors/7`. `.merge(…)`
joins a Router in as it is, with its paths unchanged. `pub fn` lets `main.rs` call `router()`.

Explained in: [Nesting and modular routers](nesting-and-modular-routers.md)
([Step 4: the books module](nesting-and-modular-routers.md#step-4-the-books-module),
[Step 5: `main.rs` names its modules](nesting-and-modular-routers.md#step-5-mainrs-names-its-modules),
and [Step 6: the lobby map, with `merge` and `nest`](nesting-and-modular-routers.md#step-6-the-lobby-map-with-merge-and-nest)).

### …write my own extractor?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:extractor}}
```

Implement `FromRequestParts` for your own type. Axum calls `from_request_parts` before the
handler: `Ok` hands the value to the handler, `Err` sends the
[rejection](../glossary.md#rejection) straight back, and the handler never runs. Here a missing
`x-api-key` [header](../glossary.md#header) gets `401`, a wrong one gets `403`, and the handler
only ever sees a good key. (`FromRequestParts` reads everything but the body; to read the body,
implement `FromRequest`, as the next item does.)

Explained in: [Custom extractors](custom-extractors.md)
([Step 3: the extractor](custom-extractors.md#step-3-the-extractor), and
[Step 4: what happens to one request](custom-extractors.md#step-4-what-happens-to-one-request)).

### …check the values in a JSON body?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:validation}}
```

Write the rules on the struct with `#[derive(Validate)]` and `#[validate(…)]`, then read the body
with `ValidatedJson<SignUp>` instead of `Json<SignUp>`. The extractor first lets `Json` read the
body (broken JSON keeps Axum's own rejection), then calls `.validate()`. Any broken rule gets
`422` with a JSON list of every problem, so the handler only runs with good input. That's
[validation](../glossary.md#validation). The `serde_json::to_value` line puts the problems in the
same order on every run.

Explained in: [Input validation](input-validation.md)
([Step 3: the rules](input-validation.md#step-3-the-rules), and
[Step 4: the `ValidatedJson` extractor](input-validation.md#step-4-the-validatedjson-extractor)).

### …test a handler without starting a server?

The helpers, inside the `#[cfg(test)] mod tests { … }` block at the bottom of `src/main.rs` (that's
why they're indented):

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:test_helpers}}
```

And one test that uses them:

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:test_example}}
```

`oneshot` (from `tower::ServiceExt`) sends one request to the Router and gives back its response,
with no network and no port. `collect` (from `http_body_util::BodyExt`) reads the whole body.
`send_to` takes the Router you give it, so a test can `clone` one app and send it several requests
that share state; `send` builds a fresh `app()` each time. Run them with `cargo test -p
cheatsheet-axum`, as at the top of this page.

Here the tests sit at the bottom of `src/main.rs`, like in every A2 lesson's crate.
[Testing handlers](testing-handlers.md) moves them into a `tests/` folder, which needs a
`src/lib.rs` so the tests can reach `app()`.

Explained in: [Testing handlers](testing-handlers.md)
([Step 5: the test helpers](testing-handlers.md#step-5-the-test-helpers),
[Step 6: the first test](testing-handlers.md#step-6-the-first-test), and
[Step 7: testing a JSON answer](testing-handlers.md#step-7-testing-a-json-answer)).

### …join all the pieces into one app?

```rust,noplayground
{{#include ../../code/topics/cheatsheet-axum/src/main.rs:app}}
```

`.merge` joins each pattern's small Router into one, with every path unchanged. `.route` adds the
`/slow` route directly. `.nest` puts the authors module under `/api/authors`. Then the four
`.layer` lines wrap everything above them: the middleware, the timeout, CORS and the request log.
`main` hands this `app()` to `axum::serve`.

Explained in: [Nesting and modular routers](nesting-and-modular-routers.md) (`merge` and `nest`),
and [Middleware and Tower layers](middleware-and-tower-layers.md) (the `.layer` order).

### …put it all together?

Routes, JSON, state, errors, validation, modules and tests in one real API: create, list, show,
update and delete todos. See [Build it: a Todo API](a2-build-todo-api.md).

## Go deeper

- [axum 0.8.9 documentation](https://docs.rs/axum/0.8.9/axum/) — The official reference for every type and function on this page.
- [Axum's examples](https://github.com/tokio-rs/axum/tree/main/examples) — Official example apps, one small idea each, from CORS to WebSockets.

<!-- next:start -->

**Next:**

- What is an ORM? (coming soon)

<!-- next:end -->
