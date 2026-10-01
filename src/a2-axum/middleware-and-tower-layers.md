# Middleware and Tower layers

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can run code before and after every request with middleware.
- You can add ready-made layers: request logging, CORS and timeouts.
- You know the order layers run in.

## What & why

Middleware wraps every request: write your own with `from_fn`, and add tower-http's TraceLayer, CorsLayer and TimeoutLayer.

Picture an airport. Passengers fly to a hundred different cities, but every one of them passes the
same checks on the way in: the ticket desk, then security, then passport control. And every
passenger coming **back** passes checks too, in the opposite order. Nobody builds a separate
security gate for each destination. The checks stand around **all** the gates, once.

A backend has jobs like that. So far, each thing our server did lived in one
[handler](../glossary.md#handler). But some jobs belong to **every** request, whichever route it
takes:

- **Write a line in the server's log** for every request, with its status and how long it took, so
  that you can see what your server is doing.
- **Tell browsers which other websites may read your answers.** A web page at `https://myapp.com`
  that calls your API from its JavaScript needs your permission, in the form of
  [CORS](../glossary.md#cors) headers.
- **Give up on a request that takes too long**, instead of keeping the client waiting forever: a
  [timeout](../glossary.md#timeout).
- **Add the same header to every answer**, such as `x-powered-by: axum`.

You could copy the same lines into every handler. With twenty handlers, that's twenty copies to keep
the same, and one day someone forgets one. Worse, some answers have **no** handler at all: the
`404 Not Found` for a path nobody wrote is Axum's own answer, so no line you put in a handler ever
runs for it.

The answer is [**middleware**](../glossary.md#middleware): code that runs **around** your handlers.
It sees each request before the handler does, and each response after the handler has made it,
like the airport checks that stand around all the gates. In Axum, you add middleware to your Router
as a [**layer**](../glossary.md#layer), with `.layer(…)`.

This lesson has three ideas:

1. **Your own middleware, with `from_fn`.** An `async fn` that receives the request and a `Next`,
   calls `next.run(request).await` to let the rest of the app answer, and can change the response
   before it goes out. `middleware::from_fn` turns that function into a layer.
2. **Ready-made layers from tower-http.** Axum is built on a crate called **Tower**, and a sister
   crate, **tower-http**, has ready-made layers for common jobs. We add three: `TraceLayer` for the
   log, `CorsLayer` for CORS, and `TimeoutLayer` for a time limit.
3. **The order of the layers.** Each `.layer` wraps everything added before it, so the **last**
   `.layer` is the outermost: it sees the request first and the response last.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/middleware-and-tower-layers`. Its `Cargo.toml` has two new lines,
`tower-http` and `tracing-subscriber`:

```toml
{{#include ../../code/topics/middleware-and-tower-layers/Cargo.toml}}
```

### Line by line

`[package]` · `name = "middleware-and-tower-layers"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p middleware-and-tower-layers`.
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
- **Why:** Axum answers requests; Tokio runs the server, and its `time` feature gives the
  `tokio::time::sleep` that Step 4's slow handler uses.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`tower-http.workspace = true`
- **What:** tower-http 0.7.1, a crate of ready-made layers for HTTP servers.
- **Why:** `TraceLayer`, `CorsLayer` and `TimeoutLayer` all come from it.
- **How:** its layers work with any server built on Tower, and Axum is one. Step 2 shows the
  **features** this line switches on.
- **Remove it and…** ``error[E0433]: cannot find module or crate `tower_http` in this scope``.

`tracing-subscriber.workspace = true`
- **What:** tracing-subscriber 0.3.23, the crate that prints log messages to the terminal.
- **Why:** `TraceLayer` **writes** a message for each request, but something has to **show** it.
  Step 8 sets this crate up to do that.
- **How:** Rust for Humans'
  [Logging and tracing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/logging-and-tracing.html)
  lesson explains the two halves: the `tracing` crate that code uses to *write* messages, and a
  *subscriber* that decides which ones to show, and where.
- **Remove it and…** ``error[E0433]: cannot find module or crate `tracing_subscriber` in this
  scope``, at the first line of `main`.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the answers, without opening a
  port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net,tokio/time`, then
`cargo add tower-http --features trace,cors,timeout`, then
`cargo add tracing-subscriber --features env-filter`, and the test crates from
`cargo add --dev tower http-body-util --features tower/util`.

### Step 2: the features, in the workspace's `Cargo.toml`

tower-http has more than thirty layers, and compiling all of them would make every build slower. So
each layer is a **feature**, an optional part of a crate that's compiled only when you ask for it,
as with Tokio's features in [Tour of the stack](../part-0-start/tour-of-the-stack.md). These are the
two lines in `code/Cargo.toml`:

```toml
{{#include ../../code/Cargo.toml:tower_http}}
{{#include ../../code/Cargo.toml:tracing_subscriber}}
```

### Line by line

`tower-http = { version = "0.7.1", features = ["trace", "cors", "timeout"] }`
- **What:** tower-http version 0.7.1, with three features: `trace` for `TraceLayer`, `cors` for
  `CorsLayer`, and `timeout` for `TimeoutLayer`.
- **Why:** one feature per layer we use, and no more.
- **How:** tower-http's documentation says which feature each layer needs. The next layer you want,
  such as `CatchPanicLayer` from [Error handling in Axum](error-handling-in-axum.md#common-mistakes),
  means one more word here: `"catch-panic"`.
- **Remove it and…** (`"cors"`) the build stops at the `use` line:

  ```text
  error[E0432]: unresolved import `tower_http::cors`
     --> src/main.rs:11:18
      |
   11 | use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
      |                  ^^^^ could not find `cors` in `tower_http`
      |
  note: found an item that was configured out
     --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tower-http-0.7.1/src/lib.rs:297:9
      |
  296 | #[cfg(feature = "cors")]
      |       ---------------- the item is gated behind the `cors` feature
  297 | pub mod cors;
      |         ^^^^
  ```

  The `note:` says exactly what's wrong: the `cors` part of tower-http exists, but it's "gated
  behind the `cors` feature". Add the feature, and it builds.

`tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }`
- **What:** tracing-subscriber version 0.3.23, with its `env-filter` feature.
- **Why:** `env-filter` gives the `.with_env_filter(…)` method that Step 8 uses to choose which log
  messages to show.
- **How:** same idea: an optional part, switched on by name.
- **Remove it and…** (the feature) ``error[E0599]: no method named `with_env_filter` found for
  struct `SubscriberBuilder<N, E, F, W>` ``, and the compiler suggests `with_file`, a different
  method with a similar name. Don't take that suggestion: add the feature.

### Step 3: the `use` lines

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/src/main.rs:imports}}
```

📁 Full code: code/topics/middleware-and-tower-layers · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers`

### Line by line

`use axum::{Router, …, routing::get};`
- **What:** brings `Router` and `get` from Axum into this file, as in every lesson.
- **Why:** the Router in Step 6 lists the routes and the layers.
- **How:** the `{ … }` lists several names from one crate in one line.
- **Remove it and…** ``cannot find type `Router` in this scope``.

`extract::Request,`
- **What:** the type of an incoming HTTP request: its method, path, headers and body, all together.
- **Why:** our middleware receives the **whole** request, not one piece of it, because it has to
  pass the whole thing on to the handler.
- **How:** until now, [extractors](../glossary.md#extractor) such as `Path` and `Json` each took one
  piece out of the request. `Request` is the request itself.
- **Remove it and…** ``cannot find type `Request` in this scope``, at the middleware's first line.

`http::{HeaderValue, StatusCode},`
- **What:** `HeaderValue`, the type of a [header](../glossary.md#header)'s value, and `StatusCode`,
  the [status codes](../glossary.md#status-code) you know.
- **Why:** the middleware adds a header, and the timeout needs to know which status to answer with.
- **How:** both come from the `http` crate, which Axum passes on to you as `axum::http`.
- **Remove it and…** ``cannot find type `HeaderValue` in this scope``, and the same for
  `StatusCode`.

`middleware::{self, Next},`
- **What:** Axum's `middleware` module itself (`self`), and its type `Next`.
- **Why:** `self` lets us write `middleware::from_fn` in Step 6; `Next` is the second parameter of
  our middleware.
- **How:** in a `use` list, `self` means "the module itself, as well as the names inside it".
- **Remove it and…** (`Next`) ``error[E0425]: cannot find type `Next` in this scope``, at the
  middleware's first line.

`response::Response,`
- **What:** the type of a finished HTTP response.
- **Why:** the middleware returns one.
- **How:** it's the type that `into_response()` returns, as in
  [Error handling in Axum, Step 5](error-handling-in-axum.md#step-5-teach-axum-how-to-send-an-apperror).
- **Remove it and…** ``cannot find type `Response` in this scope``.

`use std::time::Duration;`
- **What:** Rust's type for a length of time, such as 1 second.
- **Why:** the slow handler waits 3 seconds, and the timeout allows 1.
- **How:** `Duration::from_secs(3)` is 3 seconds.
- **Remove it and…** ``cannot find type `Duration` in this scope`` everywhere a duration is made.

`use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};`
- **What:** the three ready-made layers, each from its own module in tower-http.
- **Why:** Step 6 adds all three to the Router.
- **How:** each module exists only when its feature is on (Step 2).
- **Remove it and…** ``error[E0433]: cannot find type `TimeoutLayer` in this scope``, and the same
  for `CorsLayer` and `TraceLayer`.

### Step 4: two handlers, one fast and one slow

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/src/main.rs:handlers}}
```

📁 Full code: code/topics/middleware-and-tower-layers · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers`

### Line by line

`async fn hello() -> &'static str {` · `"hello"` · `}`
- **What:** the plainest handler: it answers `hello`.
- **Why:** the handlers in this lesson are deliberately dull, so you can watch what the layers add
  **around** them.
- **How:** as in [Hello, Axum](hello-axum.md).
- **Remove it and…** `.route("/", get(hello))` fails with ``cannot find value `hello` in this
  scope``.

`async fn slow() -> &'static str {` · `tokio::time::sleep(Duration::from_secs(3)).await;` · `"finally done"` · `}`
- **What:** a handler that waits 3 seconds, then answers `finally done`.
- **Why:** it stands in for a slow job, such as a database that's taking its time, so that we can
  watch the timeout give up on it.
- **How:** `tokio::time::sleep(…)` makes a *future* that finishes after 3 seconds, and `.await`
  pauses the handler until then. While it's paused, the thread is free to serve other requests, and
  the timeout layer, which is waiting too, can step in. *Common mistakes* shows what happens with a
  sleep that **doesn't** pause politely.
- **Remove it and…** (the `sleep` line) `/slow` answers at once, and the timeout never fires.

### Step 5: your own middleware

Here's the first new idea. A middleware is an ordinary `async fn` with a particular shape:

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/src/main.rs:middleware}}
```

📁 Full code: code/topics/middleware-and-tower-layers · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers`

### Line by line

`async fn add_powered_by(request: Request, next: Next) -> Response {`
- **What:** a middleware function. It takes the incoming request and a `Next`, and returns a
  response.
- **Why:** this is the shape Axum's `from_fn` accepts (Step 6): "give me the request and a way to
  the rest of the app, and I'll give you back the response".
- **How:** `Next` stands for **everything inside** this middleware: the inner layers, then the
  handler that the Router picks for this path. The middleware doesn't know, or need to know, which
  handler that is. The name `add_powered_by` is ours to choose.
- **Remove it and…** (the `next: Next` parameter) there's no way to reach the handler, so the
  middleware could only answer by itself, for every path.

`let mut response = next.run(request).await;`
- **What:** hands the request on, inward, waits for the answer, and keeps it in `response`.
- **Why:** this line splits the function in two. Code **above** it runs on the way in, before the
  handler; code **below** it runs on the way out, after the handler has answered.
- **How:** `next.run(request)` takes the request (it's moved in, so you can't use `request` after
  this line) and returns a *future*; `.await` waits for it to finish, and gives the `Response`. It's
  `mut` because the next line changes it.
- **Remove it and…** (`mut`) ``error[E0596]: cannot borrow `response` as mutable, as it is not
  declared as mutable``, with the hint ``consider changing this to be mutable``. (Remove `.await`,
  and the build fails in a more puzzling way: *Common mistakes* shows it.)

`response` · `.headers_mut()` · `.insert("x-powered-by", HeaderValue::from_static("axum"));`
- **What:** adds the header `x-powered-by: axum` to the response.
- **Why:** it's our "same thing on every answer". A real app might add a security header here
  instead; this one is easy to spot in curl's output.
- **How:** `.headers_mut()` gives the response's headers in a form you may change, and `.insert`
  sets one. `HeaderValue::from_static("axum")` turns fixed text into a header value; it checks the
  text as the program runs, and panics if the text has a character that isn't allowed in a header,
  such as a line break.
- **Remove it and…** (`HeaderValue::from_static(…)`, writing `"axum"`) ``error[E0308]: mismatched
  types``: ``expected `HeaderValue`, found `&str` ``. A header value isn't any text: it has rules,
  and `HeaderValue` is the type that keeps them.

`response` · `}`
- **What:** returns the changed response, which carries on outward and, in the end, to the client.
- **Why:** the function promised to return a `Response`.
- **How:** the last line of a function, without a `;`, is its return value.
- **Remove it and…** ``error[E0308]: mismatched types``: ``expected `Response<Body>`, found `()` ``,
  and the compiler suggests the fix: ``consider returning the local binding `response` ``.

### Step 6: the Router, and its layers

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/src/main.rs:app}}
```

📁 Full code: code/topics/middleware-and-tower-layers · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers`

### Line by line

`Router::new()` · `.route("/", get(hello))` · `.route("/slow", get(slow))`
- **What:** two routes: `GET /` runs `hello`, and `GET /slow` runs `slow`.
- **Why:** the routes come **first**, because each `.layer` wraps only what was added before it.
- **How:** as in [Routes and HTTP methods](routes-and-methods.md).
- **Remove it and…** (a route) its path answers `404`. That `404` still passes through every layer
  below, as *Run it* shows.

`.layer(middleware::from_fn(add_powered_by))`
- **What:** wraps the routes in our middleware from Step 5.
- **Why:** `.layer` wants a *layer*, and `add_powered_by` is only a function. `from_fn` ("from a
  function") builds a layer that runs it.
- **How:** it's the **first** `.layer`, so it's the **innermost**: closest to the handler.
- **Remove it and…** the `x-powered-by` header disappears from every answer. (Remove only
  `middleware::from_fn(…)`, writing `.layer(add_powered_by)`, and the build fails: *Common
  mistakes* shows the error.)

`.layer(TimeoutLayer::with_status_code(` · `StatusCode::REQUEST_TIMEOUT,` · `Duration::from_secs(1),` · `))`
- **What:** a time limit of 1 second on everything inside it. If the inside hasn't answered by
  then, the layer stops waiting and answers `408 Request Timeout` itself.
- **Why:** a client shouldn't wait forever for a stuck request, and the server shouldn't spend its
  work on an answer nobody is waiting for any more.
- **How:** you choose the status. `408 Request Timeout` is the common choice; `503 Service
  Unavailable` is another. (tower-http also has `TimeoutLayer::new(…)`, with no status, but it's
  *deprecated*: it still works, but the compiler warns ``use of deprecated associated function
  `tower_http::timeout::TimeoutLayer::new`: Use `TimeoutLayer::with_status_code` instead``.)
- **Remove it and…** `/slow` makes the client wait the full 3 seconds, then answers `200 OK` with
  `finally done`.

`.layer(CorsLayer::permissive())`
- **What:** adds CORS headers to every answer. `permissive()` means "any website may read my
  answers".
- **Why:** so that a web page on another website can call this API from its JavaScript. (*You might
  be wondering…* explains why "any website" is only for trying things out.)
- **How:** it adds `access-control-…` headers to each response, and answers the browser's
  "may I?" questions, called *preflight requests*, by itself. *Run it* shows both.
- **Remove it and…** no `access-control-…` headers. curl doesn't care, but a browser refuses to
  hand your answer to another website's JavaScript.

`.layer(TraceLayer::new_for_http())`
- **What:** writes a log message when each request starts, and another when its response is ready,
  with the status and how long it took.
- **Why:** so you can see what your server is doing, request by request.
- **How:** it's the **last** `.layer`, so it's the **outermost**. It sees every request first and
  every response last, so it can time the whole trip, and it logs even the answers that the layers
  inside it make up, such as the `408`.
- **Remove it and…** the server prints `Listening on http://127.0.0.1:3000` and then nothing, however
  many requests arrive.

### Step 7: the onion

Each `.layer` wraps everything above it in the code, like a new skin around an onion. So the layer
you add **last** ends up **outside**:

```text
request  ──▶ ┌─ TraceLayer ─────────────────────────────────────────┐
             │  ┌─ CorsLayer ────────────────────────────────────┐  │
             │  │  ┌─ TimeoutLayer ───────────────────────────┐  │  │
             │  │  │  ┌─ from_fn(add_powered_by) ──────────┐  │  │  │
             │  │  │  │              handler               │  │  │  │
             │  │  │  └────────────────────────────────────┘  │  │  │
             │  │  └──────────────────────────────────────────┘  │  │
             │  └────────────────────────────────────────────────┘  │
response ◀── └──────────────────────────────────────────────────────┘
```

### Line by line

`request  ──▶ ┌─ TraceLayer ──…┐`
- **What:** the request arrives at the outermost skin first: `TraceLayer`, the **last** `.layer` in
  the code.
- **Why:** each `.layer` call wraps the whole Router built so far, and the last call wraps
  everything.
- **How:** `TraceLayer` writes "started processing request", then passes the request inward.
- **Remove it and…** `CorsLayer` becomes the outermost skin.

`│  ┌─ CorsLayer ──…┐  │`
- **What:** the second skin: `CorsLayer`.
- **Why:** it's the second-to-last `.layer`.
- **How:** on the way in, it notes the request's origin. On the way out, it adds its
  `access-control-…` headers to whatever response comes back.
- **Remove it and…** no CORS headers, but the other skins work as before.

`│  │  ┌─ TimeoutLayer ──…┐  │  │`
- **What:** the third skin: the 1-second limit on everything inside it.
- **Why:** it has to be **outside** the handler, to be able to give up on it.
- **How:** it starts a 1-second timer and passes the request inward. If the timer rings first, it
  answers `408` itself, and whatever was inside is dropped, unfinished.
- **Remove it and…** nothing limits how long the inside may take.

`│  │  │  ┌─ from_fn(add_powered_by) ──…┐  │  │  │`
- **What:** the innermost skin: our own middleware, the **first** `.layer` in the code.
- **Why:** it was added first, so everything added later wraps around it.
- **How:** its `next.run(request)` is what finally reaches the handler.
- **Remove it and…** the handler sits directly inside `TimeoutLayer`.

`│  │  │  │              handler               │  │  │  │`
- **What:** the handler the Router chose for this path: `hello` for `/`, `slow` for `/slow`.
- **Why:** the handler is the core of the onion: the only part that knows what this route is for.
- **How:** Axum wraps each route in the layers separately, and also its own `404` answer for paths
  that match no route. That's why `/missing`, which has no handler, still gets every layer.
- **Remove it and…** there's nothing to answer, and every path is a `404`.

`└──…┘` (the four bottom edges, from the inside out)
- **What:** the response's way out: from the handler, through `from_fn`, then `TimeoutLayer`, then
  `CorsLayer`, then `TraceLayer`, which is the **last** to see it.
- **Why:** each skin gets the response back from the skin inside it, in reverse order.
- **How:** in our middleware, the way out is the code after `next.run(request).await`.
- **Remove it and…** there's no way out: a request that goes in has to come back out, through the
  same skins.

`response ◀──`
- **What:** the finished response leaves the outermost skin, and goes to the client.
- **Why:** by now every layer has had its say: `x-powered-by`, then the CORS headers, then the log
  line.
- **How:** Axum's `serve` sends it over the connection.
- **Remove it and…** the client waits forever.

So a request passes **Trace → Cors → Timeout → from_fn → handler**, and the response comes back out
**handler → from_fn → Timeout → Cors → Trace**. The first `.layer` in the code is the innermost;
the last is the outermost. Step 9 proves it with a small program.

### Step 8: `main`, with a log subscriber

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/src/main.rs:main}}
```

📁 Full code: code/topics/middleware-and-tower-layers · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's start, running on the Tokio runtime.
- **Why:** `main` needs `.await`, and `#[tokio::main]` is what allows that.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line of `main`.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`tracing_subscriber::fmt()`
- **What:** starts building a *subscriber*: the part that receives log messages and prints them,
  one line each, to the terminal.
- **Why:** `TraceLayer` only **writes** its messages, using the `tracing` crate. With no subscriber,
  they go nowhere.
- **How:** `fmt` is short for "format": it formats each message as a line of text, with the time,
  the level and the text.
- **Remove it and…** (the whole `tracing_subscriber::…init();` statement) the program still builds
  and runs, and answers every request, but prints no log lines at all.

`.with_env_filter("tower_http=debug")`
- **What:** a filter that decides which messages to show: those that come **from** the `tower_http`
  crate, at the `debug` level or above.
- **Why:** every message has a *level*, from the chattiest to the most serious: `trace`, `debug`,
  `info`, `warn`, `error`. `TraceLayer` writes its "started" and "finished" messages at `debug`, so
  we have to ask for `debug`.
- **How:** the text is `crate=level`. You can list several, separated by commas, such as
  `"tower_http=debug,my_app=info"`. The name says *env* because this filter is often read from an
  [environment variable](../glossary.md#environment-variable) (`RUST_LOG`), which *Logging with
  tracing*, in *Part A4*, does. Here we write it in the code, so it's always the same.
- **Remove it and…** (only this line) the server runs, but prints no request lines at all. That was
  checked for real: without a filter, the subscriber shows `info` and above, and `debug` is below
  `info`.

`.init();`
- **What:** finishes the subscriber and makes it the one for the whole program.
- **Why:** from now on, every log message anywhere in the program goes to it.
- **How:** there can be only one, so this runs once, first thing in `main`.
- **Remove it and…** the subscriber is built and thrown away, and nothing is printed.

`let listener = …bind("127.0.0.1:3000")…;` · `println!(…);` · `axum::serve(listener, app())…;`
- **What:** claims port 3000, prints `Listening on http://127.0.0.1:3000`, and answers requests
  until you stop the server.
- **Why:** a server must own a port before anyone can reach it.
- **How:** exactly as in every Axum lesson. The layers are part of `app()`, so `serve` needs no
  change.
- **Remove it and…** there's no port for curl to reach.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p middleware-and-tower-layers
```

In the second terminal, ask for the home page, and for a path that doesn't exist:

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/01-header.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/01-header.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`/`: `x-powered-by: axum`.** Our middleware ran after `hello` and added its header.
- **`/missing`: `404`, and still `x-powered-by: axum`.** No handler of ours ran, but the middleware
  did: Axum's own `404` answer is wrapped in the layers too. This is the thing a line inside a
  handler could never do.
- **`access-control-expose-headers: *` and `access-control-allow-origin: *` on both.** That's
  `CorsLayer::permissive()`. `*` means "any website". Now send what a browser sends: an `origin`
  header, which names the website whose JavaScript is asking, and a preflight request.

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/02-cors.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/02-cors.out}}
```

- **The first request:** a `GET` from a page on `https://example.com`. `access-control-allow-origin:
  *` tells the browser that this website, like any other, may read the answer.
  `access-control-expose-headers: *` lets that page's JavaScript read every header, including our
  `x-powered-by`.
- **The second request is a preflight.** Before a web page may send some requests to another
  website, such as a `DELETE`, the browser first **asks**: an `OPTIONS` request, with the origin and
  the method it would like to use (`access-control-request-method: DELETE`). `CorsLayer` answers it
  by itself, without reaching the handler: notice there's no `x-powered-by`, and no body. Its
  answer says any method (`access-control-allow-methods: *`) and any header
  (`access-control-allow-headers: *`) may be used. The `allow: GET,HEAD` line isn't from
  `CorsLayer`: Axum adds it, listing the methods the `/` route really has, as it did for `405` in
  [Routes and HTTP methods](routes-and-methods.md).

Now the slow route. It needs 3 seconds, and the timeout allows 1:

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/03-timeout.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/03-timeout.out}}
```

After about a second, `408 Request Timeout`, with an empty body. `slow` never finished: when the
timer rang, `TimeoutLayer` dropped it, halfway through its sleep, and answered by itself. Now look
at the headers. The CORS headers are there, but **`x-powered-by` is not**. That's the onion at
work: the `408` was made by `TimeoutLayer`, so only the layers **outside** it (`CorsLayer`, and
`TraceLayer`) saw it on its way out. Our middleware is **inside** the timeout; it was dropped along
with the handler, before it could add its header.

Finally, look at the first terminal. Here it is after the two requests `GET /` and `GET /slow`,
captured for real (with colours turned off; your terminal shows the levels in colour):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/middleware-and-tower-layers`
Listening on http://127.0.0.1:3000
2026-09-30T18:08:30.368231Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-09-30T18:08:30.368394Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=0 ms status=200
2026-09-30T18:08:30.368473Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_eos: end of stream stream_duration=0 ms
2026-09-30T18:08:30.380135Z DEBUG request{method=GET uri=/slow version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-09-30T18:08:31.382763Z DEBUG request{method=GET uri=/slow version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=1002 ms status=408
```

Your times and latencies will be different: they're the real clock, and the real time each request
took. Read one line from left to right:

- **`2026-09-30T18:08:30.368231Z`** is when it happened, in UTC (the `Z` at the end).
- **`DEBUG`** is the level, the one `.with_env_filter` asked for.
- **`request{method=GET uri=/ version=HTTP/1.1}`** says which request the line belongs to. When
  many requests arrive at once, their lines mix together, and this is how you tell them apart.
- **`tower_http::trace::on_request`** is where the message came from, and the text after it is the
  message itself.
- **`latency=1002 ms status=408`**: the `/slow` request took a little over a second, and ended with the
  timeout's `408`. `TraceLayer` saw that `408` because it's outside `TimeoutLayer`.

`GET /` has a third line, `end of stream`: `TraceLayer` also notes when the last byte of a body has
gone out. The `408` has an empty body, so there's no third line for it.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/` and `/missing` both show `x-powered-by: axum`, your middleware wraps every answer, even
  the ones no handler made.
- ✅ If `/slow` answers `408` after about a second, **without** `x-powered-by`, you've seen the order
  of the layers for yourself.
- ✅ If each request adds lines to the first terminal, `TraceLayer` and the subscriber are working
  together.
- ❌ If the first terminal shows nothing after `Listening…`, check the `tracing_subscriber` lines in
  `main`, and that the filter says `tower_http=debug`, with an underscore: it's the crate's name in
  Rust code, not the `tower-http` of `Cargo.toml`.
- ❌ If `/slow` answers `200 OK` with `finally done` after 3 seconds, the timeout isn't around it:
  check that its `.layer` comes **after** the routes.

### Step 9: the same order, seen from inside

The `408` above already showed the order: the layer inside the timeout was cut off, the ones outside
weren't. Here's a small program that shows it directly. Two middlewares, `one` and `two`, each
write their name on a header on the way **in** (into the request) and on the way **out** (into the
response). The handler answers with the names it received
(`cargo run -p middleware-and-tower-layers --example mw-order`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-order.rs:middleware}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-order.rs:handler}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-order.rs:app}}
```

📁 Full code: code/topics/middleware-and-tower-layers/examples/mw-order.rs · ▶ Run it: `cd code && cargo run -p middleware-and-tower-layers --example mw-order`

### Line by line

`async fn one(mut request: Request, next: Next) -> Response {`
- **What:** a middleware with the same shape as `add_powered_by`, except that `request` is `mut`.
- **Why:** this one changes the **request** too, before passing it on.
- **How:** `mut` on a parameter lets the function change the value it received.
- **Remove it and…** (`mut`) ``cannot borrow `request` as mutable``, at `.headers_mut()`.

`request.headers_mut().append("x-trail", HeaderValue::from_static("one"));`
- **What:** the way in: adds the header `x-trail: one` to the **request**.
- **Why:** it's a note for everyone further in: "`one` was here".
- **How:** `.append` adds a header even if one with that name is already there, so the notes pile up
  instead of replacing each other, as `.insert` would.
- **Remove it and…** the handler never hears of `one`.

`let mut response = next.run(request).await;`
- **What:** passes the request inward and waits for the response, as before.
- **Why:** it's the line between "on the way in" and "on the way out".
- **How:** exactly as in Step 5.
- **Remove it and…** there's no response to return.

`response.headers_mut().append("x-trail", HeaderValue::from_static("one"));` · `response`
- **What:** the way out: adds `x-trail: one` to the **response**, and returns it.
- **Why:** so the client sees in which order the middlewares handled the response.
- **How:** the same `.append`, on the response's headers.
- **Remove it and…** the response carries only `two`'s note.

`async fn two(…)` (the same four lines, with `"two"`)
- **What:** an exact copy of `one`, writing `two` instead.
- **Why:** two identical middlewares; the only difference between them is their place in the code.
- **How:** a second function, so it can be layered separately.
- **Remove it and…** there's nothing to compare `one` with.

`async fn show_trail(request: Request) -> String {`
- **What:** a handler that takes the **whole** request, as a middleware does.
- **Why:** it needs to read the request's `x-trail` headers.
- **How:** `Request` works as a handler parameter too. It takes everything, body included, so it
  must be the last parameter, like `Json` in [JSON with serde](json-and-serde.md#common-mistakes).
- **Remove it and…** there's nothing to read the trail from.

`let names: Vec<&str> = request.headers().get_all("x-trail").iter().filter_map(|value| value.to_str().ok()).collect();`
- **What:** gathers every `x-trail` value in the request, in the order they were added, into a list
  of text.
- **Why:** that order **is** the order the request passed the middlewares on its way in.
- **How:** `.get_all` finds every header with that name (`.get` would find only the first).
  `.to_str()` turns a header value into text, which can fail if it isn't plain text, so it gives a
  `Result`; `.ok()` turns that into an `Option`, and `filter_map` keeps the `Some`s. The `|value| …`
  is a [closure](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html).
- **Remove it and…** there's no list to answer with.

`format!("on the way in: {}\n", names.join(" → "))`
- **What:** the answer: the names, joined with arrows.
- **Why:** so the client sees the way in, while the headers show the way out.
- **How:** `.join(" → ")` puts `" → "` between the items; `\n` ends the line, so the terminal's
  prompt starts on a new one.
- **Remove it and…** (`\n`) the answer and your prompt share a line.

`.route("/", get(show_trail))` · `.layer(middleware::from_fn(one))` · `.layer(middleware::from_fn(two))`
- **What:** one route, then `one`, then `two`.
- **Why:** so `two`, added last, should be the outer skin.
- **How:** exactly as in Step 6.
- **Remove it and…** (swap the two `.layer` lines) every `one` and `two` in the answer swaps too.

### Run it

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/04-order.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/04-order.out}}
```

- **On the way in: `two → one`.** `two`, the **last** `.layer`, saw the request first.
- **On the way out: `x-trail: one`, then `x-trail: two`.** `one`, the inner skin, got the response
  first; `two` got it last.

- ✅ If the body says `two → one` and the headers say `one`, then `two`, you've seen the onion from
  both sides.
- ❌ If the body says `one → two`, check the order of the two `.layer` lines.

## You might be wondering…

**"Middleware or a handler?"**
Ask: "is this about **one** route, or about **all** of them (or a group)?" Finding a book, adding a
book: one route, so a handler. Logging, CORS, time limits, a header on every answer, checking a key
for every admin page: the same for many routes, so middleware. A good sign that something should
become middleware is when you find yourself pasting the same first lines into several handlers.

**"Is `CorsLayer::permissive()` safe?"**
For trying things out on your own machine, yes. For a real API, no. `permissive()` tells every
browser that **any** website's JavaScript may call your API and read the answer. A page on a
stranger's website could then use your API from its visitors' browsers. In production, name the
websites that may call you: *More examples* has the version for one website, `https://myapp.com`.
Remember that CORS protects **browser users**; it doesn't lock your API. curl, a phone app or
another server ignores CORS completely, as all our curls show. Protecting the API itself is a
different job, such as the key check in *More examples*.

**"Why does order matter?"**
Because each layer only sees what reaches it. In this lesson, `TraceLayer` is outermost, so it logs
**everything**, even the `408` that `TimeoutLayer` made up, and times the whole trip. `CorsLayer`
is outside `TimeoutLayer`, so even a timed-out answer carries the CORS headers, and a browser page
can read "408" instead of seeing a confusing CORS error. Our `add_powered_by` is inside the timeout,
so a timed-out answer has no `x-powered-by`, as *Run it* showed. A rule of thumb: put the layers
that must see **every** answer (logging, CORS) on the outside, which means **last** in the code.

**"What is Tower?"**
Tower is a crate that describes, in a general way, "something that takes a request and returns a
response" (Tower calls it a *service*) and "something that wraps a service to make a new one" (a
*layer*). It isn't about HTTP in particular. Axum is built on it: your Router, with its handlers, is
a Tower service, which is why a `.layer` from **any** crate built on Tower fits it. tower-http is a
collection of such layers written for HTTP. Besides the three in this lesson, it has
`CatchPanicLayer` (feature `catch-panic`), the safety net for panicking handlers from
[Error handling in Axum](error-handling-in-axum.md#common-mistakes); `CompressionLayer`
(`compression-gzip`), which makes answers smaller; `SetResponseHeaderLayer` (`set-header`), which
could replace our `add_powered_by`; and many more.

**"Where do my `500` errors go in the log?"**
`TraceLayer` treats every `5xx` answer as a failure, and adds an `error`-level line for it. Here it
is for a handler that answers `500`, captured for real in a scratch copy of this lesson's code:

```text
2026-09-30T18:11:19.370832Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-09-30T18:11:19.370993Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=0 ms status=500
2026-09-30T18:11:19.371040Z ERROR request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_failure: response failed classification=Status code: 500 Internal Server Error latency=0 ms
2026-09-30T18:11:19.371093Z DEBUG request{method=GET uri=/ version=HTTP/1.1}: tower_http::trace::on_eos: end of stream stream_duration=0 ms
```

So every `500` from [Error handling in Axum](error-handling-in-axum.md) now shows up as an `ERROR`
line, with no extra code. That line says **that** a request failed, not **why**. For the why, that
lesson wrote the real reason with `eprintln!`; with the `tracing` crate, you'd write it with
`tracing::error!(…)` instead, and it would join these lines, with the same time, level and request.
*Logging with tracing*, in *Part A4*, does that.

## Coming from another language?

Every web framework has middleware. What differs most is the **order** rule, so watch for it.

**Express (Node.js).** Middleware is a function with a third parameter, `next`, registered with
`app.use`:

```js
app.use((req, res, next) => {
  res.set("x-powered-by", "express");
  next();
});

app.use(cors());
```

`next()` is Axum's `next.run(request)`. One difference: Express writes the response **as you go**, so
you set headers **before** calling `next()`; there's no finished response to change afterwards.
(Express adds `X-Powered-By: Express` by itself, unless you call `app.disable("x-powered-by")`.)
`cors()` is the `cors` package, and with no options it allows every origin, like `permissive()`.
And the order is the **opposite** of Axum's: Express runs middleware in the order you register it,
so the **first** `app.use` runs first.

**Flask and FastAPI (Python).** Flask has hooks that run before and after every request:

```python
@app.after_request
def add_powered_by(response):
    response.headers["X-Powered-By"] = "flask"
    return response
```

FastAPI's middleware looks almost exactly like `from_fn`:

```python
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

@app.middleware("http")
async def add_powered_by(request: Request, call_next):
    response = await call_next(request)
    response.headers["X-Powered-By"] = "fastapi"
    return response

app.add_middleware(CORSMiddleware, allow_origins=["https://myapp.com"])
```

`await call_next(request)` is `next.run(request).await`, word for word. And FastAPI's order rule is
**the same** as Axum's: the middleware added last is the outermost.

**Spring Boot (Java).** A servlet *filter* wraps every request:

```java
@Component
class PoweredByFilter extends OncePerRequestFilter {
    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response,
                                    FilterChain chain) throws ServletException, IOException {
        response.setHeader("X-Powered-By", "spring");
        chain.doFilter(request, response);
    }
}
```

`chain.doFilter` is `next.run`. As in Express, the response is written as the controller runs, so
headers go on **before** `chain.doFilter`. The order comes from an `@Order(…)` number on each filter,
lowest first, and CORS is configured with `@CrossOrigin(origins = "https://myapp.com")` on a
controller, or in a `WebMvcConfigurer`.

**Go (`net/http` and Gin).** In Go, middleware is a function that takes a handler and returns a
new handler around it, the onion written by hand:

```go
func poweredBy(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        w.Header().Set("X-Powered-By", "go")
        next.ServeHTTP(w, r)
    })
}

handler := logging(cors(poweredBy(mux)))
```

The last line **is** the onion: `logging` is outermost because it's written outside the others.
`http.TimeoutHandler(h, time.Second, "timeout")` is Go's timeout layer; it answers `503 Service
Unavailable`. Gin's `r.Use(gin.Logger())` adds middleware in registration order, first runs first,
and a Gin middleware calls `c.Next()` where Axum calls `next.run`.

## Common mistakes

The compiler errors below were captured in a scratch copy of this lesson's code, a project called
`my-middleware`, so their line numbers are from that file.

**Forgetting `.await` on `next.run`.**

```rust,noplayground,ignore
async fn add_powered_by(request: Request, next: Next) -> Response {
    let mut response = next.run(request);
    response
        .headers_mut()
        .insert("x-powered-by", HeaderValue::from_static("axum"));
    response
}
```

```text
error[E0599]: no method named `headers_mut` found for opaque type `impl Future<Output = Response<Body>>` in the current scope
  --> src/main.rs:29:10
   |
28 | /     response
29 | |         .headers_mut()
   | |         -^^^^^^^^^^^ method not found in `impl Future<Output = Response<Body>>`
   | |_________|
   |
   |
help: consider `await`ing on the `Future` and calling the method on its `Output`
   |
29 |         .await.headers_mut()
   |          ++++++

error[E0308]: mismatched types
  --> src/main.rs:31:5
   |
31 |     response
   |     ^^^^^^^^ expected `Response<Body>`, found future
   |
note: calling an async function returns a future
  --> src/main.rs:31:5
   |
31 |     response
   |     ^^^^^^^^
```

Read the type in the first error: ``impl Future<Output = Response<Body>>``. Without `.await`,
`next.run(request)` doesn't **run** anything yet: it gives you a *future*, a promise of a response
that will exist once someone waits for it. A future has no headers, so `.headers_mut()` fails, and
it isn't a `Response`, so returning it fails too. **Fix:** put `.await` on the `next.run` line, as
in Step 5. (The compiler's first `help:` would put `.await` on the next line instead, which fixes
that error but not the second: add it where the future is made.)

**Giving `.layer` the function itself.**

```rust,noplayground,ignore
        .layer(add_powered_by)
```

```text
error[E0277]: the trait bound `fn(Request<Body>, Next) -> ... {add_powered_by}: Layer<...>` is not satisfied
   --> src/main.rs:40:16
    |
 40 |         .layer(add_powered_by)
    |          ----- ^^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_layer::Layer<Route>` is not implemented for fn item `fn(axum::http::Request<Body>, Next) -> impl Future<Output = Response<Body>> {add_powered_by}`
```

`.layer` wants a Tower *layer*, and a function isn't one, however middleware-shaped it is. The
warning above the error is a clue: ``unused import: `self` ``, the `middleware` module you imported
for `from_fn` and never used. **Fix:** `.layer(middleware::from_fn(add_powered_by))`.

**Adding `.layer` before the routes it should cover.**
Here, a route is added **after** the layer
(`cargo run -p middleware-and-tower-layers --example mw-late-route`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-late-route.rs:app}}
```

It compiles, with no warning. Now ask for both routes:

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/70-late-route.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/70-late-route.out}}
```

`/` has `x-powered-by`; `/late` doesn't. `.layer` wraps the routes that exist **when it's called**,
and `/late` didn't exist yet. With a logging or timeout layer, that route would silently have no
log lines or no time limit. **Fix:** all `.route`s first, then all `.layer`s, as in Step 6.

**Expecting the timeout to stop a blocking sleep.**
This handler waits with `std::thread::sleep` instead of `tokio::time::sleep(…).await`
(`cargo run -p middleware-and-tower-layers --example mw-blocking-sleep`, which has the same 1-second
`TimeoutLayer` and a `TraceLayer`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-blocking-sleep.rs:handler}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/71-blocking-sleep.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/71-blocking-sleep.out}}
```

Not `408`: a `200 OK`, and `finally done`, as if there were no timeout. The first terminal, captured
for real, shows how long it took:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/examples/mw-blocking-sleep`
Listening on http://127.0.0.1:3000
2026-09-30T18:08:58.107836Z DEBUG request{method=GET uri=/slow version=HTTP/1.1}: tower_http::trace::on_request: started processing request
2026-09-30T18:09:01.113092Z DEBUG request{method=GET uri=/slow version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=3005 ms status=200
2026-09-30T18:09:01.113506Z DEBUG request{method=GET uri=/slow version=HTTP/1.1}: tower_http::trace::on_eos: end of stream stream_duration=0 ms
```

`latency=3005 ms status=200`: three full seconds, and the timeout never fired. Why? The timeout is
not a separate guard watching from the side. It's part of the same async code as the handler, and
it only gets a chance to check its timer while the handler is **paused** at an `.await`.
`tokio::time::sleep(…).await` pauses; `std::thread::sleep` doesn't. It *blocks*: it holds the whole
thread for 3 seconds, and nothing else can run on that thread in the meantime, not the timeout, and
not other requests waiting for that thread. `TimeoutLayer` checked its timer once, right before it
let the handler run, when no time had passed. The handler then returned its finished answer in the
same step, and the layer passed it on. (Another timeout library might check its timer once more and
answer `408` after the 3 seconds; what no library can do is interrupt a thread that's blocked.) **Fix:** in async code, wait with `.await` (`tokio::time::sleep`), and use async versions
of slow work, such as the async database library in *Part A3*.

## More examples

Each example is a complete program in `code/topics/middleware-and-tower-layers/examples/`. Stop any
running server with `Ctrl+C`, start the example with the command shown, and send its requests from
your second terminal.

### A timing header, `x-response-time-ms`

A middleware can measure how long the rest of the app took, because it runs both before and after
it (`cargo run -p middleware-and-tower-layers --example mw-response-time`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-response-time.rs:imports}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-response-time.rs:middleware}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-response-time.rs:app}}
```

The number changes on every request, so the book's checker can't compare it, and this session was
captured by hand. `/slowish` is a handler that sleeps for a quarter of a second (250 ms):

```text
$ curl -i http://127.0.0.1:3000/
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
x-response-time-ms: 0
content-length: 5
date: Wed, 30 Sep 2026 18:09:11 GMT

hello

$ curl -i http://127.0.0.1:3000/slowish
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
x-response-time-ms: 252
content-length: 18
date: Wed, 30 Sep 2026 18:09:12 GMT

that took a moment
```

Your numbers will differ a little. `Instant::now()` reads a clock **before** `next.run`, and
`started.elapsed()` measures the time since **after** it. `.as_millis()` gives the milliseconds as a
`u128`, a very large number type, and `HeaderValue` can be made from a `u64` but not a `u128`, so
`as u64` converts it; a `u64` of milliseconds lasts over 500 million years, so nothing is lost. The
example's test checks that `/slowish` reports at least 250.

### CORS for one website only

The production version of `permissive()`: only `https://myapp.com` may read the answers
(`cargo run -p middleware-and-tower-layers --example mw-cors-origin`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-cors-origin.rs:app}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/50-cors-origin.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/50-cors-origin.out}}
```

Both answers say `access-control-allow-origin: https://myapp.com`, even the one to
`https://example.com`. That's how CORS works: the server states **who** is allowed, and the
**browser** compares it with the page's origin. A page on `https://myapp.com` gets the answer; a
page on `https://example.com` sees a different name, and the browser refuses to hand the answer to
its JavaScript. `"https://myapp.com".parse::<HeaderValue>()` turns the text into a header value,
checking it as it goes, and `.unwrap()` is safe here because the text is fixed and valid. For
several websites, `allow_origin` also accepts a list: `[a, b]`. With a list, tower-http compares
on the server: it sends back the request's origin only when it's on the list, and sends no
`access-control-allow-origin` at all to the others; the browser still has the final say. With
nothing else set, this layer
answers preflight requests without allowing any extra method or header, so a browser lets the page
send only the simplest requests, such as a plain `GET`. For a `DELETE`, or a `POST` with a JSON
body, add `.allow_methods(…)` and `.allow_headers(…)` too; the tower-http documentation has
examples.

### A middleware that refuses requests without a key

A middleware doesn't have to call `next.run`. It can answer by itself, and the handler never runs
(`cargo run -p middleware-and-tower-layers --example mw-require-key`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-require-key.rs:imports}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-require-key.rs:middleware}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-require-key.rs:app}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/51-require-key.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/51-require-key.out}}
```

`request.headers().get("x-api-key")` gives `Some(value)` or `None`. `.is_some_and(|key| key ==
"letmein")` is `true` only for `Some` with the right value; a `HeaderValue` can be compared with a
`&str` directly. Then either `next.run(request).await` lets the request in, or a
`(StatusCode, &str)` tuple, turned into a `Response` with `.into_response()`, sends it away with
`401 Unauthorized`. Notice the last request: `/missing` also gets `401`, not `404`. The guard wraps
Axum's `404` answer too, so a stranger can't even find out which paths exist. (A real key would come
from a setting, not from the code; *Custom extractors*, two lessons on, checks a key in another way.)

### `route_layer`: protect one route only

Usually, only some routes need a key. `route_layer` puts a layer on **one** route
(`cargo run -p middleware-and-tower-layers --example mw-route-layer`):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-route-layer.rs:app}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/52-route-layer.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/52-route-layer.out}}
```

`require_key` is the same function as in the previous example. `get(admin).route_layer(…)` wraps the
`/admin` route's `GET` handler, and nothing else: `/` answers without a key, `/admin` needs one. And
this time `/missing` is an honest `404`, because the guard sits on the `/admin` route only. A
`Router` has a `.route_layer` too: it wraps every route added before it, but not the `404` answer
for paths that match none.

## Your turn

Each solution is also a complete program in `code/topics/middleware-and-tower-layers/examples/`,
with a test inside.

### 🟢 Guided

Make the middleware say `x-powered-by: rust and coffee` instead of `axum`. Change one piece of text
in Step 5, run the server, and check with `curl -i`.

<details><summary>Solution</summary>

`cargo run -p middleware-and-tower-layers --example mw-powered-by-rust`:

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-powered-by-rust.rs:middleware}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/90-powered-by-rust.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/90-powered-by-rust.out}}
```

Only the text inside `HeaderValue::from_static(…)` changed. Spaces are allowed in a header value; a
line break isn't, and `from_static` would panic on one when the middleware first runs.

</details>

### 🟡 Tweak

Add a **second** middleware, `add_request_count`, that numbers the requests: the first answer gets
`x-request-count: 1`, the next `2`, and so on. Keep the counter in a `static` holding an
`AtomicU64`, as in [Shared state's visit counter](shared-state.md#a-visit-counter-with-no-mutex).
Hints: `static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);` and `HeaderValue::from(number)`, which
turns a `u64` into a header value.

<details><summary>Solution</summary>

`cargo run -p middleware-and-tower-layers --example mw-request-count`:

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-request-count.rs:imports}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-request-count.rs:middleware}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-request-count.rs:app}}
```

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/91-request-count.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/91-request-count.out}}
```

`fetch_add(1, Ordering::Relaxed)` adds one and gives back the **old** value, hence the `+ 1`, so the
first request is number 1. The number is taken **before** `next.run`, on the way in, so it's the
order requests **arrived** in. The `404` for `/missing` is counted too: the layer wraps it like the
rest. No `Arc` is needed, because a `static` lives for the whole program and every request can reach
it. That's also its weak point, from [Shared state](shared-state.md#you-might-be-wondering): every
`app()` and every test shares the same counter. That's why the example's test checks that the
second number is one more than the first, rather than expecting exactly `1` and `2`.

</details>

### 🔴 From scratch

Maintenance mode: when the environment variable `MAINTENANCE` is `1`, every path answers `503 Service Unavailable` with the text `maintenance`,
except `/health`, which keeps answering `ok` so that monitoring tools can still see the server is
alive. Without the variable, everything works as usual. Hints: read the variable once, in `main`,
into a `bool`; pass it to the middleware with `middleware::from_fn_with_state(maintenance, …)`,
whose function takes `State(on): State<bool>` as its **first** parameter; and
`request.uri().path()` is the request's path.

<details><summary>Solution</summary>

`cargo run -p middleware-and-tower-layers --example mw-maintenance` (normal), or
`MAINTENANCE=1 cargo run -p middleware-and-tower-layers --example mw-maintenance` (maintenance):

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-maintenance.rs:imports}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-maintenance.rs:middleware}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-maintenance.rs:app}}

{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-maintenance.rs:main}}
```

Started without the variable, the server works as usual:

```bash
{{#include ../../code/topics/middleware-and-tower-layers/http/92-maintenance.sh}}
```

```text
{{#include ../../code/topics/middleware-and-tower-layers/http/92-maintenance.out}}
```

The maintenance case is checked by the example's test, which builds the app with `app(true)`, as if
`MAINTENANCE=1` were set:

```rust,noplayground
{{#include ../../code/topics/middleware-and-tower-layers/examples/mw-maintenance.rs:test}}
```

(`send` is a small helper in the same file: it sends one pretend request and gives back the status
and the body. *Testing handlers* explains how.) Run it with
`cargo test -p middleware-and-tower-layers --example mw-maintenance`:

```text
   Compiling middleware-and-tower-layers v0.1.0 (/Users/you/rust-book-backend/code/topics/middleware-and-tower-layers)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.92s
     Running unittests examples/mw-maintenance.rs (target/debug/examples/mw_maintenance-914029fb67b7d230)

running 2 tests
test tests::normally_everything_answers ... ok
test tests::in_maintenance_only_health_answers ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`from_fn_with_state` is `from_fn` plus a piece of [state](../glossary.md#state), handed to the
middleware through the same `State` extractor as in [Shared state](shared-state.md). Here the state
is a plain `bool`, which is `Clone`, so no `Arc` is needed. Why not read `MAINTENANCE` inside the
middleware, on every request? It would work, but settings are read once, at startup, and passing the
`bool` in is what lets the test say `app(true)` without touching the real environment.
`std::env::var(…)` gives `Ok(text)` when the variable is set, and `.is_ok_and(|value| value == "1")`
is `true` only for the text `1`. `/missing` answers `503` in maintenance too: the check is about the
path, whatever route it would reach.

</details>

## Quick check

<div class="quiz" data-topic="middleware-and-tower-layers"></div>

## Remember this

- Middleware runs around your handlers: code before `next.run(request).await` runs on the way in,
  code after it on the way out. `middleware::from_fn` turns an `async fn(Request, Next) -> Response`
  into a layer.
- The last `.layer` is the outermost: it sees the request first and the response last. Put layers
  that must see every answer, such as `TraceLayer` and `CorsLayer`, last in the code.
- `.layer` wraps only the routes added before it, plus Axum's `404` answer. Routes first, layers
  after; `route_layer` wraps one route only.
- tower-http has ready-made layers, one Cargo feature each: `TraceLayer` (with a
  `tracing_subscriber` to print its lines), `CorsLayer` (name your websites in production), and
  `TimeoutLayer` (which only works when handlers `.await` instead of blocking).

## Go deeper

- [Rust for Humans: Logging and tracing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/logging-and-tracing.html)
- [Rust for Humans: Closures](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html)
- [tower-http](https://docs.rs/tower-http/0.7.1/tower_http/) — Official reference: every ready-made layer, and the Cargo feature each one needs.
- [axum::middleware](https://docs.rs/axum/0.8.9/axum/middleware/index.html) — Official reference: from_fn, from_fn_with_state, and the order layers run in.
- [MDN: Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS) — How browsers decide whether a web page may read another site's answers, preflight requests included.

<!-- next:start -->

**Next:**

- Nesting and modular routers (coming soon)

<!-- next:end -->
