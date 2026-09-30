# Hello, Axum

> **Beginner** · Part A2 · Axum

## By the end of this lesson

- You can start a new Axum project with cargo and add the right dependencies.
- You can serve an HTML page and a plain-text page.
- You can read every line of a raw HTTP response from your own server.

## What & why

Build your own Axum server from an empty folder: dependencies, handlers, routes, and a first look at real responses.

In [Tour of the stack](../part-0-start/tour-of-the-stack.md) you ran a [server](../glossary.md#server)
that came ready-made with the book. That was a test drive: someone else built the car, and you
turned the key. This lesson is getting your licence. You start from an empty folder, add the
[crates](../glossary.md#crate) you need yourself, and write a server with two pages:

- a home page in [**HTML**](../glossary.md#html), the language web pages are written in, which a
  browser draws as a big heading;
- an "about" page in plain text, which a browser shows exactly as it is.

Then you'll send it [requests](../glossary.md#request) with curl and read every line of each
[response](../glossary.md#response), including the one for a page that doesn't exist.

This is the first lesson of Part A2, so it also sets up the routine every Axum lesson after it
uses: **one terminal runs the server, a second terminal sends it requests**, and the book shows
you the exact response you should see. Learn the routine once here; from the next lesson on, it's
the same every time.

## The idea, slowly

### Step 1: an empty project and two dependencies

Everything starts with `cargo new`. Open a terminal in a folder where you keep your own projects
(your home folder is fine, but **not** inside the book's `code/` folder, which is the book's own
workspace), and run these three commands, one at a time:

```bash
cargo new my-first-server
cd my-first-server
cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net
```

### Line by line

`cargo new my-first-server`
- **What:** creates a new folder, `my-first-server`, holding a complete, tiny Rust program.
- **Why:** a Rust program needs a few files in the right places before Cargo will build it. `cargo
  new` writes them for you.
- **How:** it writes `Cargo.toml` (the project's settings file), `src/main.rs` (a "Hello,
  world!" program), and a `.gitignore` file, and makes the folder a Git repository. The folder's
  name becomes the project's name.
- **Remove it and…** there's no project: `cd` says `no such file or directory: my-first-server`,
  and `cargo add` says ``could not find `Cargo.toml` in … or any parent directory``.

`cd my-first-server`
- **What:** moves your terminal into the new folder.
- **Why:** Cargo commands work on the project in the folder you're standing in.
- **How:** `cd` means *change directory*.
- **Remove it and…** `cargo add` runs in the folder *above*, where there's no `Cargo.toml`, and
  stops with the same "could not find `Cargo.toml`" error.

`cargo add axum tokio`
- **What:** adds two [**dependencies**](../glossary.md#dependency): crates your project uses but
  didn't write itself. `axum` is the web [framework](../glossary.md#framework), and `tokio` is the
  async [runtime](../glossary.md#runtime) it runs on.
- **Why:** your program can only `use` a crate that its `Cargo.toml` lists. `cargo add` writes that
  line for you, with the newest version, so you don't have to look it up.
- **How:** it asks crates.io (the public library of Rust crates) for each crate's newest version,
  then adds one line per crate under `[dependencies]` in `Cargo.toml`. It doesn't download the
  crates' code yet; `cargo build` or `cargo run` does that, the first time you build.
- **Remove it and…** (`tokio`) Axum is there, but `#[tokio::main]` and `tokio::net::TcpListener`
  are unknown names, and the program can't start.

`--features tokio/macros,tokio/rt-multi-thread,tokio/net`
- **What:** switches on three **features** of Tokio. A feature is an optional part of a crate that's
  compiled only if you ask for it.
- **Why:** Tokio is big, so almost everything in it is optional. These are three of the
  workspace's Tokio features, the ones our `main` uses (see
  [Tour of the stack](../part-0-start/tour-of-the-stack.md)): `macros`
  gives `#[tokio::main]`, `rt-multi-thread` gives the runtime that `#[tokio::main]` starts, and
  `net` gives `TcpListener`, which opens the port.
- **How:** the list is separated by commas, with no spaces. Each feature starts with `tokio/`
  because this one command adds *two* crates, and Cargo needs to know which crate each feature
  belongs to.
- **Remove it and…** (the `tokio/` part, as `--features macros,rt-multi-thread,net`) Cargo refuses,
  and adds nothing:
  ``error: feature `macros` must be qualified by the dependency it's being activated for, like `axum/macros`, `tokio/macros` ``.

### Run it

```bash
cargo new my-first-server
```

```text
    Creating binary (application) `my-first-server` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

`Creating binary (application)` means the project is a program you can run (a *binary*), not a
library for other programs to use. The `note:` line is a link to Cargo's manual; you can ignore it.

```bash
cd my-first-server
cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net
```

```text
    Updating crates.io index
      Adding axum v0.8.9 to dependencies
             Features:
             + form
             + http1
             + json
             + matched-path
             + original-uri
             + query
             + tokio
             + tower-log
             + tracing
             - __private
             - __private_docs
             - http2
             - macros
             - multipart
             - ws
      Adding tokio v1.53.1 to dependencies
             Features:
             + libc
             + macros
             + net
             + rt
             + rt-multi-thread
             + socket2
             + tokio-macros
             - bytes
             - fs
             - full
             - io-std
             - io-uring
             - io-util
             - mio
             - parking_lot
             - process
             - schedule-latency
             - signal
             - signal-hook-registry
             - sync
             - taskdump
             - test-util
             - time
             - tracing
             - windows-sys
    Updating crates.io index
     Locking 52 packages to latest Rust 1.96.0 compatible versions
      Adding matchit v0.8.4 (available: v0.8.6)
```

It looks like a lot, but it's one idea, repeated:

- `Updating crates.io index` (twice): Cargo fetches the up-to-date list of crates and their
  versions.
- `Adding axum v0.8.9 to dependencies`: the version it chose, the newest one.
- `Features:` and the list under it: every feature the crate has. `+` means **on**, `-` means
  **off**. Axum switches some on by default (`http1`, `json`, `tokio`…). For Tokio, the three you
  asked for are on, plus a few that those three need themselves (`rt`, `libc`, `socket2`,
  `tokio-macros`). Everything else is off, and won't be compiled.
- `Locking 52 packages`: Axum and Tokio use other crates, which use others. Cargo picks a version
  for all 52 (the newest that works with your version of Rust, here `1.96.0`; yours may differ)
  and writes them into a new file, `Cargo.lock`, so that every later build uses exactly the same
  ones.
- `Adding matchit v0.8.4 (available: v0.8.6)`: one of those 52. Axum asks for exactly `0.8.4`, so
  Cargo notes that a newer one exists and uses the one Axum wants. Nothing to do.

Now open `Cargo.toml`. This is the whole file:

```toml
[package]
name = "my-first-server"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8.9"
tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread", "net"] }
```

Everything down to `[dependencies]` is what `cargo new` wrote; Tour of the stack's
[Step 2](../part-0-start/tour-of-the-stack.md#step-2-the-crates-cargotoml) explains those lines.
The last two lines are what `cargo add` wrote. `axum = "0.8.9"` means "Axum 0.8.9, or any later 0.8 fix
release". The Tokio line needs curly braces because it holds a version *and* a list of features.

- ✅ If your `[dependencies]` section has those two lines, you're right: the project is ready for
  the code.
- ✅ If your version numbers are **higher**, that's fine too: a newer release came out after this
  book was written. To get exactly the book's versions, run `cargo add axum@0.8.9`, then
  `cargo add tokio@1.53.1 --features macros,rt-multi-thread,net`. (Two commands, because the
  `tokio/…` form and the `@version` form don't work together in one command.)
- ❌ If you see ``error: feature `macros` must be qualified…``, you left out a `tokio/` in front of
  a feature name.

### Step 2: the book's copy of the same project

Every listing in this lesson comes from the book's copy of this project, in
`code/topics/hello-axum`. Its `Cargo.toml` asks for the same crates, written a little differently:

```toml
{{#include ../../code/topics/hello-axum/Cargo.toml}}
```

### Line by line

`[package]` · `name = "hello-axum"` · `version = "0.1.0"`
- **What:** the project's name and version, as in your own `Cargo.toml`.
- **Why:** the name is what you type after `-p` to run it: `cargo run -p hello-axum`.
- **How:** each lesson in the book is its own small project, named after the lesson.
- **Remove it and…** (`name`) Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "use the edition and publish settings from the **workspace**".
- **Why:** the book's `code/` folder is a *workspace*, one folder holding many small projects that
  share settings. Every project gets `edition = "2024"` from `code/Cargo.toml`.
- **How:** `.workspace = true` means "look this value up in `code/Cargo.toml`". `publish = false`
  there stops anyone uploading a practice project to crates.io by accident.
- **Remove it and…** (`edition`) Cargo falls back to the oldest edition, 2015, where `async fn`
  isn't allowed, and the build fails.

`[dependencies]` · `axum.workspace = true` · `tokio.workspace = true`
- **What:** the same two dependencies as your project: Axum and Tokio.
- **Why:** these lines are the book's version of `axum = "0.8.9"` and the Tokio line. They say
  "the version and the features are written in `code/Cargo.toml`". The versions are the **same**
  as yours: `code/Cargo.toml` says `axum = "0.8.9"` and Tokio `1.53.1`, with `macros`,
  `rt-multi-thread`, `net`, and `time` (which a later lesson uses).
- **How:** writing each version once, in one shared place, means all the book's projects use the
  same Axum, and changing it later is a one-line edit. In your own single project there's no
  workspace, so you write the versions directly, as `cargo add` did.
- **Remove it and…** (`axum.workspace = true`) every `use axum::…` line fails with
  ``unresolved import `axum` ``.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** crates used only by the project's **tests**, not by the server itself.
- **Why:** `src/main.rs` ends with three small tests that send pretend requests to the server's
  Router and check the answers, without opening a port. `tower` sends one request (its `oneshot`
  method), and `http-body-util` reads the response body. (Tests are a Rust topic: see
  [Rust for Humans: Unit testing](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/unit-testing.html).
  A later lesson, *Testing handlers*, covers them properly.)
- **How:** Cargo compiles `[dev-dependencies]` only for `cargo test`, so the server itself stays
  smaller.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

To give your own `my-first-server` the same test crates, the equivalent command is
`cargo add --dev tower http-body-util --features tower/util` (`--dev` puts them under
`[dev-dependencies]`; `util` is the part of `tower` that has `oneshot`). You don't need them to
follow this lesson.

### Step 3: the `use` line

From here on, the listings are Rust, from `code/topics/hello-axum/src/main.rs`. If you're building
`my-first-server`, open its `src/main.rs`, delete the "Hello, world!" program that `cargo new` put
there, and type in the listings from Steps 3 to 6, in order. First, the line at the top of the
file:

```rust,noplayground
{{#include ../../code/topics/hello-axum/src/main.rs:imports}}
```

📁 Full code: code/topics/hello-axum · ▶ Run it: `cd code && cargo run -p hello-axum`

### Line by line

`use axum::{…};`
- **What:** makes three names from the `axum` crate usable in this file without their full path.
- **Why:** without it, you'd write `axum::response::Html` every time instead of `Html`.
- **How:** the braces list several names from the same crate. A name with a `::` inside, like
  `routing::get`, means "`get`, which lives in axum's `routing` module" (a *module* is a named
  section of a crate).
- **Remove it and…** every one of the three names is unknown, and the build stops with several
  `cannot find … in this scope` errors.

`Router`
- **What:** Axum's table of [routes](../glossary.md#route): which path goes to which function.
- **Why:** Step 5 builds one.
- **How:** it lives at the top of the crate, so it's written without a module name.
- **Remove it and…** ``cannot find type `Router` in this scope``.

`response::Html`
- **What:** a wrapper that marks text as HTML. It's the **one new name** in this lesson.
- **Why:** the home page is HTML, and Axum must say so in the response (Step 4).
- **How:** `Html` lives in axum's `response` module, next to other tools for building responses.
- **Remove it and…** ``cannot find type `Html` in this scope``. *Common mistakes* shows the full
  error.

`routing::get`
- **What:** the function that says "for `GET` requests, call this handler".
- **Why:** each route in Step 5 needs it.
- **How:** it's in axum's `routing` module. There's a matching `post`, `put` and `delete`, which
  the next lesson uses.
- **Remove it and…** ``cannot find function `get` in this scope``.

### Step 4: two handlers, two kinds of page

A [handler](../glossary.md#handler) is the async function that writes one page's answer. This
server has two pages, so it has two handlers:

```rust,noplayground
{{#include ../../code/topics/hello-axum/src/main.rs:handlers}}
```

📁 Full code: code/topics/hello-axum · ▶ Run it: `cd code && cargo run -p hello-axum`

### Line by line

`async fn home() -> Html<&'static str> {`
- **What:** the home page's handler. It returns an `Html` holding a piece of text.
- **Why:** every Axum handler is an `async fn`, as in Tour of the stack. The return type is what's
  new: it tells Axum this text is a web page.
- **How:** read `Html<&'static str>` from the outside in: an `Html` wrapper, and inside the angle
  brackets, the type of what it holds. (Angle brackets like these are Rust's
  [generics](https://open-source-bd.github.io/rustbook-for-human/abstractions/generics.html): one
  `Html` type that can hold different kinds of text.) `&'static str` is a borrowed piece of text
  that lives as long as the program does, because it's written into the program itself.
- **Remove it and…** (the `Html<…>`, leaving `-> &'static str`) the function's body no longer
  matches its type, and the compiler stops with `mismatched types`.

`Html("<h1>Welcome to my first Axum app</h1>")`
- **What:** the text of the page, wrapped in `Html(…)`. It's the handler's return value: no
  `return`, no semicolon.
- **Why:** the wrapper changes one header in the response: `content-type: text/html` instead of
  `text/plain`. That header tells a browser to *draw* the tags instead of printing them.
- **How:** `<h1>` and `</h1>` are HTML **tags**: they mark the start and end of a top-level
  heading, which a browser shows big and bold. `Html(…)` builds the wrapper around the text, the
  same way you'd build a tuple struct. When the handler returns, Axum turns the `Html` into a
  response, and uses the wrapper to pick the `content-type`. (How Axum turns any value into a
  response is a [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
  called `IntoResponse`; the lesson *Handlers and IntoResponse* covers it.)
- **Remove it and…** (the `Html(…)` around the text, and the `Html<…>` in the return type) the
  page is sent as `text/plain`, and a browser shows the tags themselves:
  `<h1>Welcome to my first Axum app</h1>`, letters and brackets, in small plain type.

`async fn about() -> &'static str {` · `"This server is written in Rust."`
- **What:** the about page's handler. It returns plain text, with no wrapper.
- **Why:** it's here to show the difference. Plain text is the right answer for a page with no
  layout, and it's what Axum sends when you return a string.
- **How:** exactly the handler from Tour of the stack. Axum sends a `&'static str` as
  `content-type: text/plain; charset=utf-8`.
- **Remove it and…** (the whole handler) Step 5's `get(about)` refers to a function that doesn't
  exist, and the compiler says ``cannot find value `about` in this scope``.

Why two handlers and not one? A handler answers exactly one route. Two pages means two functions,
each doing one small job. That's how every Axum app grows: one more page, one more handler.

### Step 5: the Router, with two routes

The Router connects each path to its handler:

```rust,noplayground
{{#include ../../code/topics/hello-axum/src/main.rs:app}}
```

📁 Full code: code/topics/hello-axum · ▶ Run it: `cd code && cargo run -p hello-axum`

### Line by line

`fn app() -> Router {`
- **What:** a normal (not async) function that builds the app's Router.
- **Why:** it's a function of its own so that both `main` (the real server) and the tests at the
  bottom of the file can use the same Router.
- **How:** it returns the finished `Router`, the last expression in its body.
- **Remove it and…** (the function, with its body moved into `main`) the server still works, but
  the tests have nothing to call.

`Router::new()`
- **What:** creates an empty Router, with no routes yet.
- **Why:** every Router starts empty; the lines below fill it.
- **How:** each `.route(…)` after it returns the Router with one more rule, so the calls chain,
  one per line.
- **Remove it and…** there's no Router to add routes to, and the code doesn't compile.

`.route("/", get(home))`
- **What:** the first rule: a `GET` request for the path `/` is answered by `home`.
- **Why:** `/` is the root, the address with nothing after the port. It's where a browser goes
  when you type `http://127.0.0.1:3000`.
- **How:** `get(home)` says "for `GET`, call `home`". `home` has no `()`: you hand Axum the
  function itself, and it calls it later, once for every matching request.
- **Remove it and…** `GET /` gets `404 Not Found`, while `/about` still works.

`.route("/about", get(about))`
- **What:** the second rule: `GET /about` is answered by `about`.
- **Why:** a second page needs a second rule. A handler with no route is never called.
- **How:** the path must match exactly: `/about` is not `/About`, and not `/about/`.
- **Remove it and…** `/about` gets `404 Not Found`, even though the `about` handler is still in the
  file. *Common mistakes* shows this happen.

Any other path, like `/contact`, matches no rule, so Axum answers `404 Not Found`.

### Step 6: `main`

Last, `main`. It's the same `main` as in
[Tour of the stack, Step 5](../part-0-start/tour-of-the-stack.md#step-5-main-where-it-all-starts),
word for word, and every Axum lesson in this book starts its server this way:

```rust,noplayground
{{#include ../../code/topics/hello-axum/src/main.rs:main}}
```

📁 Full code: code/topics/hello-axum · ▶ Run it: `cd code && cargo run -p hello-axum`

### Line by line

`#[tokio::main]` · `async fn main() {`
- **What:** the program's starting point, with Tokio's attribute on top.
- **Why:** `main` needs `.await`, and only async code can `.await`. Rust won't let `main` be async
  on its own, so `#[tokio::main]` starts the Tokio runtime and runs your async `main` on it.
- **How:** it's a macro (code that writes code) from Tokio's `macros` feature, the one you switched
  on in Step 1. Tour of the stack shows what it expands to.
- **Remove it and…** ``error[E0752]: `main` function is not allowed to be `async` ``.

`let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")` · `.await` · `.expect(…);`
- **What:** claims [port](../glossary.md#port) 3000 on `127.0.0.1` (this computer only), waits for
  it, and stops the program with a helpful message if it can't.
- **Why:** a server must own a port before anyone can reach it.
- **How:** `bind` returns a future; `.await` waits for it and gives back a `Result`; `.expect`
  takes the listener out of it, or stops with our message if the port is taken.
- **Remove it and…** (the `.await`) the compiler stops: a future has no `.expect`.

`println!("Listening on http://127.0.0.1:3000");`
- **What:** prints the address, once the port is really open.
- **Why:** it's your sign that the server started.
- **How:** it runs only after `bind` succeeded.
- **Remove it and…** the server works, but sits in silence, and you can't tell whether it started.

`axum::serve(listener, app())` · `.await` · `.expect(…);`
- **What:** starts serving: every request that arrives on the port goes to the Router from `app()`.
- **Why:** this is the loop that makes the program a server.
- **How:** it never finishes on its own; the `.expect` message appears only if the server stops
  with an error. You stop it with `Ctrl+C`.
- **Remove it and…** the program prints `Listening on …` and exits at once, and every curl gets
  `Couldn't connect to server`.

### Step 7: start the server, then talk to it

Here is the routine for every Axum lesson in this book. You need **two terminal windows**.

In the **first** terminal, start the server:

```bash
cd code && cargo run -p hello-axum
```

### Line by line

`cd code`
- **What:** moves into the book's `code/` folder, from the book's main folder.
- **Why:** that's where the workspace's `Cargo.toml` is, and Cargo must run inside the workspace.
- **How:** `&&` runs the next command only if `cd` worked.
- **Remove it and…** Cargo stops with ``could not find `Cargo.toml` in … or any parent directory``.

`cargo run -p hello-axum`
- **What:** builds this lesson's project, if anything changed, then starts it.
- **Why:** the workspace holds many projects, so `-p` (short for *package*) says which one to run.
- **How:** in your own `my-first-server` folder, there's only one project, so there you type
  `cargo run`, with no `-p`. Both servers answer every curl in Step 7 the same way.
- **Remove it and…** (the `-p hello-axum`) Cargo can't choose among the workspace's programs:
  ``error: `cargo run` could not determine which binary to run``, followed by the list of
  programs it found.

### Run it

In the first terminal:

```bash
cd code && cargo run -p hello-axum
```

```text
   Compiling hello-axum v0.1.0 (/Users/you/rust-book-backend/code/topics/hello-axum)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.97s
     Running `target/debug/hello-axum`
Listening on http://127.0.0.1:3000
```

(If this is the first Axum project you've built, Cargo first downloads and compiles Axum, Tokio and
the crates they use, so a long list of `Compiling …` lines comes before these.) In your own folder,
`cargo run` prints the same, with your project's name:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/my-first-server`
Listening on http://127.0.0.1:3000
```

The terminal now stays busy: the server is running, and waiting. Leave it alone. Open a **second**
terminal, and send it three requests. The book keeps them in a small script file,
`code/topics/hello-axum/http/01-first-requests.sh`:

```bash
{{#include ../../code/topics/hello-axum/http/01-first-requests.sh}}
```

The first line starts with `#`, so it's a **comment**: your terminal ignores it. It's a note for the
book's checking tool, which says which server to start before the requests (here, the same
`-p hello-axum` you typed). The other three lines are what you type, one at a time, pressing
Enter after each. Every lesson from here on shows its requests this way.

This is what the server answers. Every `$` line is a command you typed (don't type the `$`
yourself: it stands for your terminal's prompt), and the lines under it are what curl printed:

```text
{{#include ../../code/topics/hello-axum/http/01-first-requests.out}}
```

Two small differences from your own terminal, so they don't surprise you:

- **Your terminal also shows a `date: …` line**, right after `content-length`, such as
  `date: Wed, 30 Sep 2026 16:07:52 GMT`. The server adds the current time to every response. The
  book hides that line because it changes every second: the book's checker re-sends these requests
  whenever the code changes and compares the answers with this page, and a line that changes every
  second could never match.
- **Your next prompt may appear right after the body**, on the same line, such as `…app</h1>%` or
  `…app</h1>$`, depending on your terminal. The body ends exactly where the server's text ends,
  with no line break after it. The book starts each command on a fresh line.

Now, every line, one response at a time.

**`GET /`, the HTML page:**
- `HTTP/1.1 200 OK` is the **status line**: the HTTP version, then the
  [status code](../glossary.md#status-code) `200` and its name, `OK`. The page was found.
- `content-type: text/html; charset=utf-8` is the `Html` wrapper at work: it tells the client that
  the body is a web page (`text/html`), written in UTF-8, the usual way of storing text as bytes.
- `content-length: 37` is the body's size in bytes. Count
  `<h1>Welcome to my first Axum app</h1>`: 37 characters, one byte each.
- The **empty line** means "headers are over, the body starts".
- `<h1>Welcome to my first Axum app</h1>` is the body, exactly the text `home` returned. curl
  prints the tags as they are; a browser would draw a heading.

**`GET /about`, the plain-text page:**
- `HTTP/1.1 200 OK`: found, again.
- `content-type: text/plain; charset=utf-8`: **plain text**. The only difference from the home page
  is that `about` returned its text without `Html(…)`.
- `content-length: 31` and the body, `This server is written in Rust.`: 31 characters.

**`GET /contact`, a page that doesn't exist:**
- `HTTP/1.1 404 Not Found`: no route matched `/contact`, so Axum answered on its own, with `404`.
- `content-length: 0`: the body is **empty**. There's no page, so there's nothing to send.
- There's **no `content-type`** line: with no body, there's no type of body to describe.

When you're done, go back to the first terminal and press **`Ctrl+C`** to stop the server.

- ✅ If you see `200 OK` with `text/html` for `/`, `200 OK` with `text/plain` for `/about`, and
  `404 Not Found` for `/contact`, you're right: you've built a server with two pages, and read its
  answers line by line.
- ✅ Open `http://127.0.0.1:3000/` in a web browser while the server runs: you see a big heading,
  not tags. Open `/about`, and you see the sentence in plain type.
- ❌ If curl says `Failed to connect to 127.0.0.1 port 3000 … Couldn't connect to server`, the
  server isn't running. Check the first terminal shows `Listening on http://127.0.0.1:3000`.
- ❌ If `cargo run` stops with `port 3000 is busy: stop the other program using it, or change the
  port`, another server is still running. See *Common mistakes*.

## You might be wondering…

**"Why does `main` block forever?"**
Because a server's job is to wait for the next request, forever. `axum::serve(…)` is a loop that
only ends if something breaks, and `.await` waits for it to end. So the program never reaches the
closing `}` of `main`, and the terminal stays busy. That's why you need a second terminal for curl.
Stop the server with `Ctrl+C`.

**"Why `127.0.0.1` and not `localhost`?"**
`localhost` is a *name* for this computer, and it can mean two addresses: `127.0.0.1` (the older
IPv4 kind) and `::1` (the newer IPv6 kind). Our server listens on `127.0.0.1` only. When you type
`curl http://localhost:3000/`, curl tries `::1` first, finds nobody there (`Connection refused`),
then tries `127.0.0.1` and gets through. So `localhost` works with curl and browsers, but some
tools try only `::1`, and then fail. Writing `127.0.0.1` everywhere means the address you type is
exactly the one the server opened, with no guessing.

**"I changed the code, but curl still shows the old text."**
A running program never picks up changes to its source. Stop the server with `Ctrl+C`, then
`cargo run` again: Cargo rebuilds it with your change, then starts it.

**"Do I need `cargo add` every time?"**
No: once per crate, per project. `cargo add` only writes a line into `Cargo.toml`. After that,
`cargo build` and `cargo run` download and compile whatever that file lists, by themselves. You can
also type the line into `Cargo.toml` by hand; `cargo add` saves you looking up the newest version
and the right spelling. And inside the book's `code/` folder you never need it: every version the
book uses is already in `code/Cargo.toml`.

**"Why is `/contact` a 404 with an empty body?"**
Because the Router has no rule for `/contact`. When nothing matches, Axum answers for you with
`404 Not Found` and no body at all: the status code says everything a program needs to know. Other
frameworks send a small "Not Found" page instead (see the next section). You can give Axum your own
"not found" answer with `Router::fallback`, but the empty 404 is what you get by default.

## Coming from another language?

Every web framework has the same three pieces you wrote today: a project file listing dependencies,
handlers, and routes. Here's the home page (and, where the default differs, the about page) in
each one. Each snippet leaves out the setup lines (creating `app` or `r`, and starting the
server); Tour of the stack shows them for Express, Flask and Go.

**Express (Node.js).** `npm init` and `npm install express` match `cargo new` and `cargo add`, and
`package.json` matches `Cargo.toml`:

```js
app.get("/", (req, res) => res.send("<h1>Welcome to my first Express app</h1>"));
app.get("/about", (req, res) => res.type("text/plain").send("This server is written in JavaScript."));
```

Notice the default is the other way round: Express sends a string as `text/html` unless you say
otherwise, while Axum sends it as `text/plain` unless you wrap it in `Html`.

**Flask and FastAPI (Python).** `pip install flask` matches `cargo add`. Flask, like Express, sends
a returned string as `text/html`:

```python
@app.route("/")
def home():
    return "<h1>Welcome to my first Flask app</h1>"
```

FastAPI sends a returned string as JSON by default; you ask for HTML with a response class, which
is close to Axum's `Html` wrapper:

```python
from fastapi.responses import HTMLResponse, PlainTextResponse

@app.get("/", response_class=HTMLResponse)
def home():
    return "<h1>Welcome to my first FastAPI app</h1>"

@app.get("/about", response_class=PlainTextResponse)
def about():
    return "This server is written in Python."
```

Tokio's features are like Python's *extras*: `pip install "fastapi[standard]"` switches on optional
parts of a package, as `--features tokio/macros` does.

**Spring Boot (Java).** Dependencies go in `pom.xml` or `build.gradle`. A `@RestController` sends a
`String` as plain text; `produces` asks for HTML:

```java
@GetMapping(value = "/", produces = MediaType.TEXT_HTML_VALUE)
public String home() {
    return "<h1>Welcome to my first Spring app</h1>";
}
```

**Go (`net/http` and Gin).** `go mod init` and `go get` match `cargo new` and `cargo add`. The
standard library guesses the content type from the first bytes of the body, so text that starts
with `<h1>` goes out as `text/html`. Gin makes you choose: `c.String(200, …)` is plain text, and
`c.Data(200, "text/html; charset=utf-8", []byte(…))` is HTML:

```go
r.GET("/", func(c *gin.Context) {
    c.Data(200, "text/html; charset=utf-8", []byte("<h1>Welcome to my first Gin app</h1>"))
})
r.GET("/about", func(c *gin.Context) {
    c.String(200, "This server is written in Go.")
})
```

And the 404 for `/contact`? Express sends a small HTML page saying `Cannot GET /contact`; Flask, an
HTML "Not Found" page; FastAPI, the JSON `{"detail":"Not Found"}`; Spring Boot, a JSON error object
(to curl; a browser gets its "Whitelabel Error Page"); Go and Gin, the text `404 page not found`. Axum sends the status and an empty body.

## Common mistakes

The file paths in these messages start at your own project (`src/main.rs`). Run from the book's
`code/` folder, the same messages say `topics/hello-axum/src/main.rs`.

**Using `Html` without importing it.**
The handler is right, but the `use` line at the top still has only `Router` and `get`, as in Tour
of the stack:

```rust,noplayground,ignore
use axum::{Router, routing::get};

async fn home() -> Html<&'static str> {
    Html("<h1>Welcome to my first Axum app</h1>")
}
```

```text
error[E0425]: cannot find type `Html` in this scope
 --> src/main.rs:3:20
  |
3 | async fn home() -> Html<&'static str> {
  |                    ^^^^ not found in this scope

error[E0425]: cannot find function, tuple struct or tuple variant `Html` in this scope
 --> src/main.rs:4:5
  |
4 |     Html("<h1>Welcome to my first Axum app</h1>")
  |     ^^^^ not found in this scope

error[E0277]: the trait bound `fn() -> impl Future<Output = {type error}> {home}: Handler<_, _>` is not satisfied
   --> src/main.rs:13:25
    |
 13 |         .route("/", get(home))
    |                     --- ^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn() -> impl Future<Output = {type error}> {home}`
…
error: could not compile `my-first-server` (bin "my-first-server") due to 3 previous errors
```

Three errors, one cause. The first two say Rust doesn't know the name `Html`, once in the return
type and once where the text is wrapped. The third is a knock-on effect: because `home`'s return
type is broken (`{type error}`), Axum can't accept `home` as a handler. **Read the first error
first**: it's usually the real one, and fixing it often makes the rest disappear. **Fix:** add
`response::Html` to the `use` line, as in Step 3: `use axum::{Router, response::Html, routing::get};`.

**Writing a handler, but forgetting its route.**
This Router has lost its `/about` line; the `about` handler is still in the file:

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/forgot-route.rs:app}}
```

It compiles, but the compiler gives you a hint:

```text
warning: function `about` is never used
 --> src/main.rs:7:10
  |
7 | async fn about() -> &'static str {
  |          ^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
```

"Never used" means nothing calls `about`: no route points at it. Run the server anyway
(`cargo run -p hello-axum --example forgot-route`), and send two requests:

```bash
{{#include ../../code/topics/hello-axum/http/70-forgot-route.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/70-forgot-route.out}}
```

`/` still works, but `/about` is now the same empty `404` as `/contact` was: the Router has no
rule for it, so the handler is never reached. **Fix:** add `.route("/about", get(about))` back. When
you see `is never used` on a handler, look for its missing route. (The book's copy of this file,
`examples/forgot-route.rs`, has an `#[allow(dead_code)]` line above `about` that silences the
warning, because the book's checks treat every warning as an error.)

**Running two servers at once.**
Start the server in one terminal, forget it's there, and run `cargo run -p hello-axum` in another:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/hello-axum`

thread 'main' (12781086) panicked at topics/hello-axum/src/main.rs:28:10:
port 3000 is busy: stop the other program using it, or change the port: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Only one program at a time can own a port. The second server's `bind` fails, and our `.expect`
message says so, followed by the operating system's reason, `Address already in use`. (The code is
48 on macOS and 98 on Linux, and the number after `'main'` changes every run.) This also happens
when you move on to an example (`--example …`) while the lesson's main server is still running.
**Fix:** find the terminal with the first server and press `Ctrl+C`. If you can't find it,
`lsof -i :3000` shows which program holds the port, as in
[Your toolbox](../part-0-start/your-toolbox.md).

## More examples

Each example is a complete program in `code/topics/hello-axum/examples/`. Stop any running server
with `Ctrl+C`, start the example with the command shown, and send its requests from your second
terminal. To try one in `my-first-server`, copy the lines shown into your `src/main.rs`.

### A home page with a link

HTML's `<a>` tag makes a link. This home page links to `/about`
(`cargo run -p hello-axum --example link-page`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/link-page.rs:home}}
```

```bash
{{#include ../../code/topics/hello-axum/http/50-link-page.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/50-link-page.out}}
```

`<a href="/about">About this server</a>` shows the words *About this server*; clicking them makes
the browser send `GET /about`. The text contains double quotes, so it's written as a **raw
string**, `r#"…"#`: everything between `r#"` and `"#` is taken exactly as written, quotes included.
In an ordinary `"…"` string, each inner `"` would need a backslash in front (`\"`). Open
`http://127.0.0.1:3000/` in a browser, and click the link.

### Text built with `format!`

When the text isn't fixed, build it with `format!`, and return a `String`
(`cargo run -p hello-axum --example format-string`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/format-string.rs:about}}
```

```bash
{{#include ../../code/topics/hello-axum/http/51-format-string.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/51-format-string.out}}
```

`format!` fills each `{language}` and `{pages}` with the variable of that name
([Rust for Humans: formatting](https://open-source-bd.github.io/rustbook-for-human/language-basics/formatting-with-format.html)),
and makes a new `String`. The text is built while the program runs, so it can't be a
`&'static str`; a `String` owns its text instead. Axum sends a `String` exactly like a
`&'static str`: `text/plain`.

### A third route

One more page is one more handler and one more `.route`
(`cargo run -p hello-axum --example third-route`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/third-route.rs:hello}}

{{#include ../../code/topics/hello-axum/examples/third-route.rs:app}}
```

```bash
{{#include ../../code/topics/hello-axum/http/52-third-route.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/52-third-route.out}}
```

The order of the `.route` lines doesn't matter: Axum matches each request by its whole path, not
by trying the rules from top to bottom.

### An HTML page built while the program runs

`Html` can hold a `String` as well as a `&'static str`. This handler builds a list from an array
(`cargo run -p hello-axum --example html-string`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/html-string.rs:menu}}
```

```bash
{{#include ../../code/topics/hello-axum/http/53-html-string.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/53-html-string.out}}
```

`<ul>` is an HTML list, and each `<li>` is one item in it. The `for` loop adds one `<li>…</li>` per
dish to `items` (`push_str` adds text to the end of a `String`), then `format!` puts the whole list
under a heading. The return type is `Html<String>` this time, and the `content-type` is still
`text/html`: the wrapper decides the type, not what's inside it.

## Your turn

Each solution is also a complete program in `code/topics/hello-axum/examples/`, with a test inside.

### 🟢 Guided

Give the home page a heading of your own. In `src/main.rs`, change the text between `<h1>` and
`</h1>` in `home`. Stop the server, run it again, and curl `/`. Which header line changes, and by
how much?

<details><summary>Solution</summary>

Here, for someone called Ada (`cargo run -p hello-axum --example guided-heading`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/guided-heading.rs:home}}
```

```bash
{{#include ../../code/topics/hello-axum/http/90-guided-heading.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/90-guided-heading.out}}
```

`content-length` changes from 37 to 32: `<h1>Ada's corner of the web</h1>` is 32 characters. Your
number depends on your heading. If you changed `src/main.rs` in the book's copy, the test
`home_is_an_html_page` at the bottom of the file now fails when you run `cargo test -p hello-axum`,
because it checks for the old heading. Change the text in the test too, and it passes again.

</details>

### 🟡 Tweak

Add a third page, `GET /contact`, that answers with your email address as plain text.

<details><summary>Solution</summary>

A handler with no wrapper (plain text), and one more route
(`cargo run -p hello-axum --example contact`):

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/contact.rs:contact}}

{{#include ../../code/topics/hello-axum/examples/contact.rs:app}}
```

```bash
{{#include ../../code/topics/hello-axum/http/91-contact.sh}}
```

```text
{{#include ../../code/topics/hello-axum/http/91-contact.out}}
```

The `404` from Step 7 is now `200 OK`, with `text/plain`. (`example.com` is a web address set aside
for examples; use your own.) In the book's copy, the test `unknown_path_is_404` at the bottom of
`src/main.rs` uses `/contact` as its "page that doesn't exist", so it now fails. Change its path to
one that really doesn't exist, such as `/nope`.

</details>

### 🔴 From scratch

Add `GET /time`, an **HTML** page that shows the number of whole seconds since the Unix epoch
(midnight, UTC, on 1 January 1970, the moment most computers count time from). Use
`std::time::SystemTime` from Rust's standard library: nothing new goes into `Cargo.toml`. Tour of
the stack's 🔴 exercise returned the number as plain text; this time, wrap it in a sentence inside
a `<p>` (paragraph) tag.

<details><summary>Solution</summary>

`cargo run -p hello-axum --example time-page`:

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/time-page.rs:use_time}}

{{#include ../../code/topics/hello-axum/examples/time-page.rs:time}}

{{#include ../../code/topics/hello-axum/examples/time-page.rs:app}}
```

- `use std::time::{SystemTime, UNIX_EPOCH};` goes at the top of the file, under the `use axum…`
  line. It brings in the clock type, and the constant for 1 January 1970.
- `SystemTime::now().duration_since(UNIX_EPOCH)` measures the time from 1970 until now. It fails
  only if the computer's clock is set before 1970, which `.expect(…)` handles with a message.
- `.as_secs()` turns that into whole seconds, a number.
- `format!` puts the number into the sentence, and `Html(…)` marks it as a web page, so the return
  type is `Html<String>`, as in the menu example above.

`curl -i http://127.0.0.1:3000/time` answers `200 OK` with `content-type: text/html; charset=utf-8`,
and a body like `<p>Seconds since 1970: ` followed by a ten-digit number and `</p>`. Run it twice,
a second apart, and the number goes up. That's also why the book can't show you this transcript:
it would be different every second. The example's test checks only the parts that stay the same,
the status and the `content-type`:

```rust,noplayground
{{#include ../../code/topics/hello-axum/examples/time-page.rs:test}}
```

Run it from `code/` with `cargo test -p hello-axum --example time-page`:

```text
running 1 test
test tests::time_is_an_html_page ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

</details>

## Quick check

<div class="quiz" data-topic="hello-axum"></div>

## Remember this

- `cargo new <name>` makes a project; `cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`
  adds the two dependencies an Axum server needs. The book's crates write `.workspace = true`
  instead, because their versions live once, in `code/Cargo.toml`.
- A handler that returns `&'static str` or `String` sends `text/plain`; wrap the text in `Html(…)`
  and it sends `text/html`, which a browser draws as a page.
- One page is one handler plus one `.route(path, get(handler))`. A path with no route gets
  `404 Not Found` with an empty body.
- Run the server in one terminal, curl it from a second, and `Ctrl+C` to stop. Your terminal also
  shows a `date:` line, which the book hides.

## Go deeper

- [Rust for Humans: Cargo basics](https://open-source-bd.github.io/rustbook-for-human/start-here/cargo-basics.html)
- [Rust for Humans: Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)
- [Axum docs](https://docs.rs/axum/0.8.9/axum/) — Official API reference.
- [axum::response::Html](https://docs.rs/axum/0.8.9/axum/response/struct.Html.html) — The wrapper that turns text into an HTML response.
- [The Cargo Book: cargo add](https://doc.rust-lang.org/cargo/commands/cargo-add.html) — Every option of `cargo add`, including `--features` and `--dev`.

<!-- next:start -->

**Next:**

- [Routes and HTTP methods](../a2-axum/routes-and-methods.md)

<!-- next:end -->
