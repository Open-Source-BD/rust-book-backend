# Handlers and IntoResponse

> **Beginner** · Part A2 · Axum

## By the end of this lesson

- You know which Rust values Axum can turn into a response and what each one sends.
- You can set the status code and headers of a response.
- You can build a response by hand when nothing else fits.

## What & why

What a handler can return — text, HTML, status codes, headers, or a hand-built Response — and how `IntoResponse` makes it work.

Go back to the restaurant from
[How a web backend works](../part-0-start/how-a-web-backend-works.md): the kitchen cooks, and the
waiter carries the plate out to the customer. But the kitchen doesn't always hand over a plate.
Soup comes in a bowl, a set meal comes on a tray, and an order to go comes in a takeaway box. The
waiter doesn't need a separate lesson for each one: every container the kitchen uses is one the
waiter **knows how to carry**.

In Axum, your [handler](../glossary.md#handler) is the cook, and the value it returns is the
container. So far you've returned four kinds: a `&'static str` and an `Html` in
[Hello, Axum](hello-axum.md), and a `(StatusCode, &'static str)` pair and a lone `StatusCode` in
[Routes and HTTP methods](routes-and-methods.md). Axum carried each one out as a proper HTTP
[response](../glossary.md#response), with a status line, [headers](../glossary.md#header) (the
`name: value` lines of extra information) and a body, because each of those types *knows how to
become a response*.

That ability has a name: **`IntoResponse`**. It's a
[**trait**](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html),
Rust's word for a set of abilities a type promises to have (Rust for Humans teaches traits in
full). A type that implements `IntoResponse` has one ability: "turn me into an HTTP response".
Axum accepts **any** return type with that ability, and refuses, at compile time, any type
without it.

This lesson walks through the containers you'll use most: three kinds of text, a chosen
[status code](../glossary.md#status-code), extra headers, and, for the rare case nothing else fits,
a response you build by hand, piece by piece.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/handlers-and-into-response`. Its `Cargo.toml` is the same as in
Routes and HTTP methods, with a different name:

```toml
{{#include ../../code/topics/handlers-and-into-response/Cargo.toml}}
```

### Line by line

`[package]` · `name = "handlers-and-into-response"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p handlers-and-into-response`.
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
- **Why:** the same two crates every Axum server needs. Everything this lesson uses, including
  `IntoResponse`, comes with Axum: no new dependency.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the response bodies, without
  opening a port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the same dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, and the test crates
from `cargo add --dev tower http-body-util --features tower/util`.

### Step 2: the `use` line

This lesson needs more names from Axum than the last two, so `cargo fmt` (Rust's formatter) puts
one per line:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:imports}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`use axum::{` · `};`
- **What:** brings seven names from Axum into this file.
- **Why:** so the code can write `StatusCode` instead of `axum::http::StatusCode`.
- **How:** the braces list several names from the same crate. Inside, `http::{StatusCode, header}`
  is a second pair of braces: two names from Axum's `http` module in one go.
- **Remove it and…** none of the seven names is known, and the build stops with many errors.

`Router`
- **What:** Axum's table of [routes](../glossary.md#route).
- **Why:** Step 8 builds one.
- **How:** the same `Router` as in every Axum lesson.
- **Remove it and…** ``cannot find type `Router` in this scope``.

`http::{StatusCode, header}`
- **What:** `StatusCode` is the list of every status code, such as
  `StatusCode::CREATED` for `201`. `header` is a **module** (a named group of code) holding a
  constant for every standard header name, such as `header::CACHE_CONTROL` for `cache-control`.
- **Why:** Step 4 chooses a status code, and Step 5 adds a header.
- **How:** you met `StatusCode` in Routes and HTTP methods. `header` is new: you write
  `header::` and then the name you want, in capitals, with `_` for `-`.
- **Remove it and…** (`header`) ``cannot find module or crate `header` in this scope``, pointing
  at `header::CACHE_CONTROL`.

`response::{Html, IntoResponse, Response}`
- **What:** three names from Axum's `response` module: the `Html` wrapper, the `IntoResponse`
  trait, and `Response`, the type of a finished HTTP response.
- **Why:** `page` returns `Html`, `no_cache` promises "something that implements `IntoResponse`",
  and `by_hand` returns a `Response` it builds itself.
- **How:** `Html` is the same wrapper as in Hello, Axum. The other two are new, and Steps 5 and 7
  explain them.
- **Remove it and…** (`IntoResponse`) ``cannot find trait `IntoResponse` in this scope``. Without
  `Response`, three errors: two ``cannot find type `Response` in this scope``, and a knock-on error
  saying `by_hand` isn't a valid handler.

`routing::get`
- **What:** the function that makes a route answer `GET`.
- **Why:** every route in this lesson is a `GET`, so you can open each one in a browser.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** ``cannot find function `get` in this scope``.

### Step 3: three kinds of text

Here are three handlers that all send text. Each returns a different type:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:simple}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`async fn text() -> &'static str {` · `"plain text"`
- **What:** returns a fixed piece of text.
- **Why:** the simplest response there is: `200 OK`, plain text.
- **How:** `&'static str` is text written into your program, as in Hello, Axum's `about`. Axum
  turns it into a response with `content-type: text/plain; charset=utf-8` and the text as the
  body.
- **Remove it and…** (the `-> &'static str`) the function returns nothing, and the compiler stops
  with `mismatched types`: the body says text, the signature says nothing.

`async fn sum() -> String {` · `format!("{} + {} = {}", 2, 3, 2 + 3)`
- **What:** builds the text `2 + 3 = 5` while the program runs, and returns it.
- **Why:** real answers are rarely fixed. They're built from numbers, names, database rows.
- **How:** `format!` fills each `{}` with the next value after the comma, in order: `2`, `3`, then
  the result of `2 + 3`. It makes a new `String`, which owns its text. A `String` goes out exactly
  like a `&'static str`: `200 OK`, `text/plain`. (Hello, Axum's
  [More examples](hello-axum.md#text-built-with-format) used `format!` the same way.)
- **Remove it and…** (writing `-> &'static str` instead of `-> String`) the compiler stops with
  `mismatched types`, ``expected `&str`, found `String` ``: text built while the program runs
  can't be a `&'static str`.

`async fn page() -> Html<&'static str> {` · `Html("<p>an HTML page</p>")`
- **What:** returns text wrapped in `Html`.
- **Why:** the wrapper changes one header: `content-type: text/html` instead of `text/plain`, so a
  browser draws the page instead of showing the tags.
- **How:** the body is the same kind of text as in `text`. Only the wrapper, and so the
  `content-type`, is different.
- **Remove it and…** (the `Html(…)` and the `Html<…>`) the tags go out as `text/plain`, and a
  browser shows `<p>an HTML page</p>` as it is.

So the **type** you return decides the `content-type`. You never write that header yourself: each
type knows which one it needs when it becomes a response.

### Step 4: choose the status code

Routes and HTTP methods returned a status code and a body together. Here it is again, now that you
know why it works:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:status}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`async fn created() -> (StatusCode, &'static str) {`
- **What:** the return type is a **tuple**: two values in round brackets, a `StatusCode` and a
  `&'static str`, in that order.
- **Why:** a handler returns **one** value, but a response needs a status and a body. A tuple packs
  both into one value.
- **How:** Axum implements `IntoResponse` for this pair: "use the first item as the status, and turn
  the last item into the body".
- **Remove it and…** (the `StatusCode, ` part of the type, and of the value) you're back to a plain
  `&'static str`, sent with the usual `200 OK`.

`(StatusCode::CREATED, "made it")`
- **What:** the value: status `201 Created`, and the body `made it`.
- **Why:** `201` is the code for "a new thing was made", which is what this handler pretends to do.
- **How:** **status first, body last.** The order isn't a style choice: Axum only knows how to turn
  a tuple into a response when the body is its **last** item, and the status, if there is one, its
  **first**. *Common mistakes* shows the error you get the other way round.
- **Remove it and…** (swap the order, `("made it", StatusCode::CREATED)`) the program no longer
  compiles.

### Step 5: add a header

A status and a body cover most answers. Sometimes you also need to say something **about** the
answer: how long it may be kept, what file name to save it as, which page to go to next. That's
what headers are for:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:headers}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`async fn no_cache() -> impl IntoResponse {`
- **What:** the return type says "some type that implements `IntoResponse`", without naming it.
- **Why:** the real type of the value below is long:
  `(StatusCode, [(HeaderName, &'static str); 1], &'static str)`, where `; 1` means "an array of
  one". `impl IntoResponse` saves you writing it, and says the one thing that matters: Axum can turn it into a response.
- **How:** `impl Trait` in a return type means "a value of one type that implements this trait;
  the compiler works out which from the function's body". It's still one exact type, checked at
  compile time. You don't have to spell it out.
- **Remove it and…** (the word `impl`, leaving `-> IntoResponse`) ``error[E0782]: expected a type,
  found a trait``: a trait is a list of abilities, not a type, so Rust needs `impl` to mean "some
  type with these abilities". (Writing the long type out in full instead works too.)

`(` · `)`
- **What:** a tuple with **three** items this time.
- **Why:** status, headers and body: the three parts of every HTTP response.
- **How:** `cargo fmt` puts each item on its own line because together they're too long for one.
  The commas between them are what matter, not the line breaks.
- **Remove it and…** (the brackets) three values with nothing holding them together aren't one
  return value, and the build stops.

`StatusCode::OK,`
- **What:** status `200 OK`.
- **Why:** it's the normal status; it's written here so you can see where a status goes in a
  three-item tuple.
- **How:** first, as in Step 4.
- **Remove it and…** the tuple becomes `([…], "fresh every time")`, which works too: with no status
  in the tuple, Axum uses `200`. The 🔴 exercise uses that shape.

`[(header::CACHE_CONTROL, "no-store")],`
- **What:** the headers to add: an **array** (a list in square brackets) of pairs, each pair a
  header name and its value. There's one pair here.
- **Why:** `cache-control: no-store` tells the browser, and anything in between, not to keep a copy
  of this answer: ask the server again every time. It's what you'd send for a bank balance.
- **How:** `header::CACHE_CONTROL` is the constant for the name `cache-control`, spelled right and
  checked by the compiler. For two headers, put two pairs in the array, separated by a comma. The
  array goes **between** the status and the body.
- **Remove it and…** (the whole line) the answer has no `cache-control` header: a `(status, body)`
  pair, like `created`.

`"fresh every time",`
- **What:** the body.
- **Why:** every response here sends some text, so you can see which handler answered.
- **How:** last, as always. It's a `&'static str`, so it sets `content-type: text/plain`.
- **Remove it and…** the last item becomes the header array, which Axum then sends as a response
  with no body: the tuple still compiles, but the text is gone.

### Step 6: the shape of a response tuple

Steps 4 and 5 used two shapes of tuple. They're both the same pattern, with optional parts:

```text
(  StatusCode::OK,   [(header::CACHE_CONTROL, "no-store")],   "fresh every time"  )
   └ 1. status        └ 2. headers                              └ 3. body
     optional            optional, any number                     always there, always last
```

### Line by line

`(` … `)`
- **What:** one tuple, one return value.
- **Why:** a handler returns one value; the tuple holds the pieces of the response.
- **How:** the items are separated by commas, and read left to right.
- **Remove it and…** you're down to one item: a body on its own, such as `text` in Step 3.

`1. status, optional`
- **What:** a `StatusCode`, if you want one other than `200 OK`.
- **Why:** so `201`, `404` or `418` are one word away.
- **How:** if it's there, it's **first**. Leave it out, and Axum uses `200`.
- **Remove it and…** the response is `200 OK`.

`2. headers, optional, any number`
- **What:** things to add to the response: here, an array of header pairs.
- **Why:** so you can add headers without building the response yourself.
- **How:** they sit **between** status and body. You can have several, one after the other
  (Axum accepts up to 16 in one tuple), each one something that can add to a response. A header
  array is the most common kind. Axum calls these things *response parts*.
- **Remove it and…** the response carries only the headers its body type sets, such as
  `content-type`.

`3. body, always there, always last`
- **What:** anything that implements `IntoResponse`: `&'static str`, `String`, `Html`, and more.
- **Why:** it's the part the client came for.
- **How:** Axum turns it into a response **first** (which sets its `content-type`), then puts your
  headers on top, then sets your status. That's why it must be last: every item before it is a
  change made *to* it.
- **Remove it and…** (with a status and headers left) Axum sends those with an empty body.

### Step 7: build a response by hand

A tuple covers almost everything. When it doesn't, you can build the `Response` yourself, one
piece at a time:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:by_hand}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`async fn by_hand() -> Response {`
- **What:** returns a `Response`: a finished HTTP response, the very thing every other return type
  gets turned into.
- **Why:** when you build it yourself, there's nothing left for Axum to convert.
- **How:** `Response` implements `IntoResponse` too, the easy way: it's already a response, so Axum
  sends it as it is.
- **Remove it and…** (the `-> Response`) the function returns nothing, and the compiler stops with
  `mismatched types`.

`Response::builder()`
- **What:** starts an empty response **builder**: a blank form you fill in, one field per method
  call, then hand in.
- **Why:** a response has several parts, and a builder lets you set only the ones you need.
- **How:** each call below returns the builder with one more field filled in, the same chaining
  style as `.route(…).route(…)` on a Router.
- **Remove it and…** there's nothing to call `.status(…)` on.

`.status(StatusCode::ACCEPTED)`
- **What:** sets the status to `202 Accepted`.
- **Why:** `202` means "I got your request, and I'll deal with it later": the job is queued, not
  done yet.
- **How:** the same `StatusCode` names as in a tuple.
- **Remove it and…** the builder uses `200 OK`.

`.header("x-note", "built by hand")`
- **What:** adds the header `x-note: built by hand`.
- **Why:** to show a header you made up. Starting a made-up name with `x-` is a common way to say
  "this isn't a standard header".
- **How:** there's no `header::` constant for a name you invented, so the name is plain text. Call
  `.header` again for each extra header.
- **Remove it and…** the response has no `x-note` line.

`.body("queued".into())`
- **What:** sets the body to `queued`, and **finishes** the builder.
- **Why:** a response needs a body, even an empty one, so this is always the last step.
- **How:** a `Response` holds its body as Axum's `Body` type, not as a `&str`. `.into()` converts
  the text into a `Body`. You don't name the type: Rust works it out from the return type,
  `Response`.
- **Remove it and…** (the `.into()`) `mismatched types`: ``expected `Response<Body>`, found
  `Response<&str>` ``.

`.unwrap()`
- **What:** takes the finished `Response` out of the `Result` that `.body(…)` returns.
- **Why:** `.body(…)` is where the builder checks everything you gave it. A header name with a
  space in it, for example, isn't allowed, so building **can** fail, and `.body(…)` returns
  `Ok(response)` or an error.
- **How:** `.unwrap()` means "I'm sure it's `Ok`: give me the response, and if it's an error, stop
  (panic)". It's safe **here** because the status and the header are fixed, valid values: they're
  the same every time, so if they work once, they always work. *You might be wondering…* shows what
  happens when they're not.
- **Remove it and…** `mismatched types`: ``expected `Response<Body>`, found `Result<Response<_>,
  Error>` ``.

### Step 8: the Router

One route per handler, all `GET`:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:app}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router, starting from an empty one.
- **Why:** a separate function, so `main` and the tests use the same Router.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** there's no Router to add routes to.

`.route("/text", get(text))` · `.route("/sum", get(sum))` · `.route("/page", get(page))`
- **What:** the three text handlers from Step 3.
- **Why:** one path each, so you can compare their answers side by side.
- **How:** notice that `get(…)` takes all three, although they return three different types. It
  accepts any `async fn` whose return type implements `IntoResponse`.
- **Remove it and…** (one of them) that path gets Axum's empty `404`.

`.route("/created", get(created))` · `.route("/no-cache", get(no_cache))` · `.route("/by-hand", get(by_hand))`
- **What:** the status, header and hand-built handlers from Steps 4, 5 and 7.
- **Why:** same reason: one path each.
- **How:** `get(created)` is fine with a tuple, `get(no_cache)` with an `impl IntoResponse`, and
  `get(by_hand)` with a `Response`. They all have the one ability `get` asks for.
- **Remove it and…** that path gets `404`.

### Step 9: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/src/main.rs:main}}
```

📁 Full code: code/topics/handlers-and-into-response · ▶ Run it: `cd code && cargo run -p handlers-and-into-response`

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
- **How:** for every request, Axum finds the handler, awaits it, and calls `into_response()` on
  whatever it returned. That call is the one ability `IntoResponse` promises.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p handlers-and-into-response
```

```text
   Compiling handlers-and-into-response v0.1.0 (/Users/you/rust-book-backend/code/topics/handlers-and-into-response)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
     Running `target/debug/handlers-and-into-response`
Listening on http://127.0.0.1:3000
```

In the second terminal, ask each of the six paths:

```bash
{{#include ../../code/topics/handlers-and-into-response/http/01-returns.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/01-returns.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides. Reading
it one response at a time:

- **`/text`:** `200 OK`, `content-type: text/plain; charset=utf-8`, and `content-length: 10`, the
  ten characters of `plain text`. A `&'static str` became a plain-text response.
- **`/sum`:** the same status and `content-type`, with the body `2 + 3 = 5` (9 characters). A
  `String` is sent exactly like a `&'static str`; only where the text came from is different.
- **`/page`:** `content-type: text/html; charset=utf-8`. Same kind of text, but wrapped in `Html`,
  so the type of the body changed.
- **`/created`:** `HTTP/1.1 201 Created`: the tuple's status replaced `200`. The body, `made it`,
  is still plain text, because the body in the tuple is a `&'static str`.
- **`/no-cache`:** `200 OK`, and a new line, `cache-control: no-store`, from the header array. It
  comes **after** `content-type`: the body set `content-type` first, then your header was added on
  top, in the order Step 6 described. (The order of headers doesn't change their meaning.)
- **`/by-hand`:** `HTTP/1.1 202 Accepted` and `x-note: built by hand`, both from the builder, and
  the body `queued`. And one line is **missing**: there's no `content-type`. A `&'static str`
  returned on its own knows it's plain text, and says so; a `Body` built by hand is only bytes, so
  when you build a response by hand, every header is yours to add, `content-type` included.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/page` says `text/html` while `/text` and `/sum` say `text/plain`, you've seen the return
  type choose the `content-type`.
- ✅ If `/created` says `201 Created`, `/no-cache` has `cache-control: no-store`, and `/by-hand`
  says `202 Accepted` with `x-note: built by hand`, you've set a status and headers three ways.
- ❌ If `cargo run` stops before `Listening…` with ``the trait bound `…: Handler<_, _>` is not
  satisfied``, a handler returns a type Axum can't send. See *Common mistakes*.

Here is everything the six handlers returned, and what each one sent:

| The handler returns | Status | `content-type` | Other headers |
|---|---|---|---|
| `&'static str` | `200 OK` | `text/plain; charset=utf-8` | |
| `String` | `200 OK` | `text/plain; charset=utf-8` | |
| `Html<&'static str>` | `200 OK` | `text/html; charset=utf-8` | |
| `(StatusCode, &'static str)` | yours: `201 Created` | `text/plain; charset=utf-8` | |
| `(StatusCode, [(name, value)], &'static str)` | yours: `200 OK` | `text/plain; charset=utf-8` | yours: `cache-control` |
| `Response` (built by hand) | yours: `202 Accepted` | none, unless you add it | yours: `x-note` |

## You might be wondering…

**"What is a trait?"**
A named set of abilities that a type can promise to have. `IntoResponse` is a trait with one
ability, `into_response()`: "turn this value into an HTTP response". Axum's authors wrote that
ability for `&'static str`, `String`, `Html`, `StatusCode`, tuples, and many more types, and
Axum's `get(…)` accepts any handler whose return type has it. Rust for Humans'
[Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
lesson teaches traits from the start, including how to give your own type an ability. You'll do
that for `IntoResponse` in *Error handling in Axum*.

**"What does `impl IntoResponse` mean?"**
"This function returns a value of **some** type that implements `IntoResponse`; I'm not writing
out which." The compiler still knows the exact type, from the function's body, and checks it; you
don't have to type it. Use it when the real type is long, such as a tuple with a header array in
it. Two limits: every path through the function must return the **same** type (*Common mistakes*
shows what happens if not), and the caller only knows it can call `into_response()` on it, which is
all Axum needs.

**"Why is status first in the tuple?"**
Because Axum only implements `IntoResponse` for tuples in that order: an optional status first, then
any headers, then the body last. It mirrors an HTTP response as it travels, top to bottom: status
line, headers, body. And the body has to be last for a practical reason: Axum turns the body into a
response first, then applies each item before it on top (Step 6). Flask, in Python, uses the
opposite order, `return "made it", 201`, so if you've used Flask, this one will catch you.

**"When would I build a Response by hand?"**
Rarely, and that's the point of all the other types. Reach for the builder when the parts of the
response aren't known until the program runs and don't fit a tuple well, such as adding a header
only in some cases, or when you're passing on a response from somewhere else. Most handlers in this
book return text, `Html`, JSON (in [JSON with serde](json-and-serde.md)), or a tuple. Remember the cost: a hand-built
response has only the headers you add, as `/by-hand`'s missing `content-type` showed.

**"Is `.unwrap()` dangerous in `by_hand`?"**
Only if the values can be wrong. Here's what happens if the header name has a space in it,
`.header("x note", "built by hand")`, which HTTP doesn't allow. It compiles, because a name is
text, and the compiler doesn't check what's in text. Then the server starts, and `curl -i
http://127.0.0.1:3000/by-hand` prints:

```text
curl: (52) Empty reply from server
```

The server's terminal shows why (this was captured in a scratch project called `my-responses`, so
the line number is from that file):

```text
Listening on http://127.0.0.1:3000

thread 'tokio-rt-worker' (12919639) panicked at src/main.rs:37:10:
called `Result::unwrap()` on an `Err` value: http::Error(InvalidHeaderName)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`.body(…)` returned an error, `InvalidHeaderName`, and `.unwrap()` stopped that request with a
panic, so the client got no answer at all. Axum runs each request on its own, so the server keeps
going: `/text` still answers. With fixed, correct values, as in `by_hand`, this can't happen. When
the values come from outside (a user, a file), don't `.unwrap()`: handle the error, which
*Error handling in Axum* shows how to do.

## Coming from another language?

Every framework has the same three choices: what the body is, what the status is, and which headers
to add. They differ in how much the framework decides for you.

**Express (Node.js).** You call methods on the response object `res`, then send:

```js
app.get("/created", (req, res) => res.status(201).send("made it"));
app.get("/no-cache", (req, res) => {
  res.set("Cache-Control", "no-store");
  res.send("fresh every time");
});
```

One difference: `res.send` with a string sends `content-type: text/html` by default. Axum sends a
string as `text/plain`, and makes you ask for HTML with `Html`.

**Flask and FastAPI (Python).** Flask lets you return a tuple too, in the **opposite** order to Axum:
body first, then status, then headers.

```python
@app.get("/created")
def created(): return "made it", 201

@app.get("/no-cache")
def no_cache(): return "fresh every time", 200, {"Cache-Control": "no-store"}
```

Flask also sends a string as `text/html`. FastAPI turns a returned string into **JSON** (the text
in double quotes), so for plain text you return `PlainTextResponse("made it", status_code=201)`,
and for HTML an `HTMLResponse`. Those response classes play the part of Axum's `Html` wrapper and
tuples.

**Spring Boot (Java).** `ResponseEntity` is a builder, very close to Axum's hand-built `Response`:

```java
@GetMapping("/by-hand")
public ResponseEntity<String> byHand() {
    return ResponseEntity.status(HttpStatus.ACCEPTED)
        .header("x-note", "built by hand")
        .body("queued");
}
```

Returning a plain `String` from a `@RestController` method is the equivalent of Axum returning a
`&'static str`: `200 OK` with the text as the body.

**Go (`net/http` and Gin).** You write to the response in the same order as Axum's tuple: headers,
then status, then body. Headers set after `WriteHeader` are ignored:

```go
func noCache(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Cache-Control", "no-store")
    w.WriteHeader(http.StatusOK)
    w.Write([]byte("fresh every time"))
}
```

If you don't set `Content-Type`, Go guesses it from the first bytes of the body. Gin has one method
per kind of body, much like Axum's types: `c.String(http.StatusCreated, "made it")`,
`c.Data(…)` for bytes, and `c.Header("Cache-Control", "no-store")` for headers.

## Common mistakes

The three errors below were captured in a scratch copy of this lesson's code, a project called
`my-responses`, so their line numbers are from that file, not from the book's.

**Returning a number.**
You want a route that answers how many books there are:

```rust,noplayground,ignore
async fn book_count() -> i32 {
    42
}

// …and in app():
        .route("/count", get(book_count))
```

```text
error[E0277]: the trait bound `fn() -> impl Future<Output = i32> {book_count}: Handler<_, _>` is not satisfied
   --> src/main.rs:52:30
    |
 52 |         .route("/count", get(book_count))
    |                          --- ^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn() -> impl Future<Output = i32> {book_count}`
    |                          |
    |                          required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
…
```

The message points at the **route**, not at the handler, and doesn't mention `IntoResponse`. It
says `book_count` isn't a `Handler`: Axum's name for "a function `get` can use". (The rest of the
message points inside Axum's own code; the book cuts it at `…`.) You'll see this same error every
time a handler's return type isn't something Axum can send.

The `note:` line has the way to find out why: `#[axum::debug_handler]`. It needs Axum's `macros`
feature (`cargo add axum --features macros` in your own project). Write `#[axum::debug_handler]` on
the line above `async fn book_count`, build again, and the first error now names the real problem:

```text
error[E0277]: the trait bound `i32: IntoResponse` is not satisfied
  --> src/main.rs:13:26
   |
13 | async fn book_count() -> i32 {
   |                          ^^^ the trait `IntoResponse` is not implemented for `i32`
   |
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
…
```

Read it as: "`i32` doesn't have the `IntoResponse` ability". And that's on purpose: should `42` be
sent as text, as JSON, or as 4 raw bytes? Axum refuses to guess. The `help:` list shows some of the
types that do have it (`&'static str` is there). **Fix:** turn the number into text yourself:
return a `String`, with `42.to_string()` or `format!("{} books", 42)`.

**Putting the body before the status.**
The Flask habit, `("made it", StatusCode::CREATED)`:

```rust,noplayground,ignore
async fn created() -> (&'static str, StatusCode) {
    ("made it", StatusCode::CREATED)
}
```

Without `debug_handler`, it's the same `Handler<_, _>` error as above, this time naming
``Output = (&'static str, StatusCode)``. With `#[axum::debug_handler]` above the handler:

```text
error[E0277]: the trait bound `&'static str: IntoResponseParts` is not satisfied
  --> src/main.rs:21:24
   |
21 | async fn created() -> (&'static str, StatusCode) {
   |                        ^ the trait `IntoResponseParts` is not implemented for `&'static str`
   |
   = help: the following other types implement trait `IntoResponseParts`:
             ()
             (T1, T2)
             (T1, T2, T3)
             (T1, T2, T3, T4)
             (T1, T2, T3, T4, T5)
             (T1, T2, T3, T4, T5, T6)
             (T1, T2, T3, T4, T5, T6, T7)
             (T1, T2, T3, T4, T5, T6, T7, T8)
           and 15 others
```

(The same error is printed twice, then the `Handler` error follows.) `IntoResponseParts` is the
trait for Step 6's *response parts*, the things that sit before the body and add to it, such as
a header array. Axum read your tuple the only way it knows: everything but the last item is a part,
and the last item is the body. So it tried to use `"made it"` as a part, and text can't be one.
**Fix:** status first, body last: `(StatusCode::CREATED, "made it")`, and the same order in the
return type.

**Returning two different types from an `if`.**
A handler that answers `409 Conflict` when the shelf is full, and `201 Created` otherwise:

```rust,noplayground,ignore
fn shelf_is_full() -> bool {
    false
}

async fn add_book() -> impl IntoResponse {
    if shelf_is_full() {
        StatusCode::CONFLICT
    } else {
        (StatusCode::CREATED, "book added")
    }
}
```

```text
error[E0308]: `if` and `else` have incompatible types
  --> src/main.rs:32:9
   |
29 | /     if shelf_is_full() {
30 | |         StatusCode::CONFLICT
   | |         -------------------- expected because of this
31 | |     } else {
32 | |         (StatusCode::CREATED, "book added")
   | |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `StatusCode`, found `(StatusCode, &str)`
33 | |     }
   | |_____- `if` and `else` have incompatible types
   |
   = note: expected struct `StatusCode`
               found tuple `(StatusCode, &str)`
help: you could change the return type to be a boxed trait object
   |
28 - async fn add_book() -> impl IntoResponse {
28 + async fn add_book() -> Box<dyn IntoResponse> {
   |
help: if you change the return type to expect trait objects, box the returned expressions
   |
30 ~         Box::new(StatusCode::CONFLICT)
31 |     } else {
32 ~         Box::new((StatusCode::CREATED, "book added"))
   |
```

Both branches have the `IntoResponse` ability, so why not? Because `impl IntoResponse` means **one**
type that has it, not "any of several". The `if` branch is a `StatusCode`, the `else` branch is a
tuple, and a function can only return one type. Don't follow the two `help:` lines here: they
compile the `if`, but a `Box<dyn IntoResponse>` isn't something Axum can send, so you're back to the
`Handler<_, _>` error (tried in the same scratch project).

**Fix:** turn each branch into a `Response` yourself, with `.into_response()`, the one ability
`IntoResponse` gives. Now both branches are the same type, `Response`, and so is the function
(`cargo run -p handlers-and-into-response --example handlers-if-else`):

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-if-else.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-if-else.rs:add_book}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/70-if-else.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/70-if-else.out}}
```

The shelf isn't full, so the `else` branch ran: `201 Created`, `book added`. `.into_response()`
is a method of the `IntoResponse` trait, and a trait's methods can only be called when the trait
is in the `use` line, which is why `IntoResponse` is imported here even though no return type
mentions it. `shelf_is_full` always says `false`: it stands in for a real check, such as asking a
database, which comes in Part A3.

## More examples

Each example is a complete program in `code/topics/handlers-and-into-response/examples/`. Stop any
running server with `Ctrl+C`, start the example with the command shown, and send its requests from
your second terminal.

### `Result<String, StatusCode>`: an answer or an error code

A preview of *Error handling in Axum*. `Result` holds either a success (`Ok`) or an error (`Err`),
and Axum implements `IntoResponse` for it when both sides have the ability
(`cargo run -p handlers-and-into-response --example handlers-result`):

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-result.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-result.rs:find_book}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-result.rs:app}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/50-result.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/50-result.out}}
```

`find_book(1)` returns `Ok` with a `String`, sent as `200 OK` and plain text. `find_book(9)` returns
`Err(StatusCode::NOT_FOUND)`, sent as `404 Not Found` with an empty body (`content-length: 0`).
Axum sends whichever side it got. The two routes have fixed paths for now: reading the `1` or `9`
from one `/books/{id}` route is the next lesson, [Path and Query extractors](path-and-query-extractors.md).

### `StatusCode` alone: nothing to say

Routes and HTTP methods' `delete_book`, again: a status code on its own is a whole response
(`cargo run -p handlers-and-into-response --example handlers-no-content`):

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-no-content.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-no-content.rs:delete_book}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-no-content.rs:app}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/51-no-content.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/51-no-content.out}}
```

`204 No Content` and nothing else: no body, so no headers describing one. The route starts with
`delete(…)` this time, so `delete` is the function in the `use` line.

### `Vec<u8>`: raw bytes

Not every body is text. Images, PDFs and zip files are bytes, and `Vec<u8>` is Rust's list of bytes
(`cargo run -p handlers-and-into-response --example handlers-bytes`):

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-bytes.rs:bytes}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-bytes.rs:app}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/52-bytes.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/52-bytes.out}}
```

`vec![104, 105, 33]` is three bytes. Axum doesn't know what they mean, so it says
`content-type: application/octet-stream`, which is HTTP for "bytes; I don't know what kind". curl
prints them as text anyway, and they happen to be the codes for `h`, `i` and `!`. For a real image
you'd add the right `content-type` with a header array, such as `image/png`.

### `Redirect::to`: go to another address

When a page moves, the old address should send visitors to the new one
(`cargo run -p handlers-and-into-response --example handlers-redirect`):

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-redirect.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-redirect.rs:handlers}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-redirect.rs:app}}
```

The second request uses `-L` (short for `--location`), which tells curl to follow a redirect:

```bash
{{#include ../../code/topics/handlers-and-into-response/http/53-redirect.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/53-redirect.out}}
```

- `HTTP/1.1 303 See Other`: `Redirect::to` always sends `303`, which means "the answer is at another
  address: go and `GET` it there".
- `location: /text` is the header that says **where**. It's the only thing a redirect really
  carries; the body is empty (`content-length: 0`, then the blank line that ends the headers, and
  no text).
- With `-L`, curl read the `location:` header and sent a second request, to `/text`. You see both
  answers: the `303`, then `200 OK` with `plain text`. A browser does the same thing on its own,
  so a visitor who types `/old-text` lands on `/text` and sees the new address in the address bar.

Axum has two more: `Redirect::temporary` (`307`) and `Redirect::permanent` (`308`, "this moved for
good": browsers and search engines remember it). `Redirect::to` is the usual one after a form is
sent.

## Your turn

Each solution is also a complete program in `code/topics/handlers-and-into-response/examples/`,
with a test inside.

### 🟢 Guided

`202 Accepted` means "I got it, I'll do it later". Change `created` so it answers `202` instead of
`201`, with the same body. You change one word; find it in Step 4.

<details><summary>Solution</summary>

`cargo run -p handlers-and-into-response --example handlers-accepted`:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-accepted.rs:created}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/90-accepted.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/90-accepted.out}}
```

`StatusCode::CREATED` became `StatusCode::ACCEPTED`, and the status line followed. The return type
didn't change: it's still a `StatusCode` and a `&'static str`. If you change `src/main.rs` itself,
the test `every_return_type_becomes_a_response` still expects `201` for `/created`, so update it
too.

</details>

### 🟡 Tweak

Add a `/teapot` route that answers `418 I'm a teapot` with a short body of your choice. (`418` is a
joke from a 1998 April Fools' document about coffee pots, and it's in `StatusCode` for real.) Look
for its name among `StatusCode`'s constants: they're the status's name in capitals, with `_`
between words.

<details><summary>Solution</summary>

`cargo run -p handlers-and-into-response --example handlers-teapot`:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-teapot.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-teapot.rs:teapot}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-teapot.rs:app}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/91-teapot.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/91-teapot.out}}
```

`StatusCode::IM_A_TEAPOT` is `418`, and the status line shows its official name. The `'` in the
body is fine inside double quotes. `content-length: 33` is the 33 characters of the body.

</details>

### 🔴 From scratch

Add a `/download` route that sends some text as a **file to save**, named `notes.txt`. The header
that does it is `content-disposition: attachment; filename="notes.txt"` (`attachment` means "save
it, don't show it"). You need no status: `200` is right. Hint: the header name has a constant in
`header::`, and the value contains double quotes, so each one needs a `\` in front of it inside a
Rust string.

<details><summary>Solution</summary>

`cargo run -p handlers-and-into-response --example handlers-download`:

```rust,noplayground
{{#include ../../code/topics/handlers-and-into-response/examples/handlers-download.rs:imports}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-download.rs:download}}

{{#include ../../code/topics/handlers-and-into-response/examples/handlers-download.rs:app}}
```

```bash
{{#include ../../code/topics/handlers-and-into-response/http/92-download.sh}}
```

```text
{{#include ../../code/topics/handlers-and-into-response/http/92-download.out}}
```

A two-item tuple, `(headers, body)`, with no status, so the answer is `200 OK` (Step 6's shape,
without part 1). `\"` puts a double quote into the string without ending it. curl prints the text
because `-i` shows everything; open `http://127.0.0.1:3000/download` in a browser instead, and it
saves a file called `notes.txt`.

</details>

## Quick check

<div class="quiz" data-topic="handlers-and-into-response"></div>

## Remember this

- A handler can return anything that implements `IntoResponse`: `&'static str` and `String` (plain
  text), `Html` (a web page), `StatusCode` (no body), `Vec<u8>` (bytes), `Redirect`, a `Result`, or
  a tuple. The type decides the `content-type`.
- A response tuple is `(status, headers…, body)`: the status first and optional, the body always
  last. `(StatusCode::CREATED, "made it")`, never the other way round.
- `impl IntoResponse` means "one type with that ability, the compiler knows which". Two branches
  with different types need `.into_response()` on each, and `-> Response`.
- `Response::builder()` builds a response piece by piece. You add every header yourself, even
  `content-type`, and `.unwrap()` is only safe with fixed, valid values.
- A `Handler<_, _>` error on a route usually means the handler's return type can't become a
  response. `#[axum::debug_handler]` names the real cause.

## Go deeper

- [Rust for Humans: Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
- [axum::response](https://docs.rs/axum/0.8.9/axum/response/index.html) — Official reference: every type that implements IntoResponse, and how tuples build a response.
- [MDN: HTTP headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers) — What each standard header means, including `Cache-Control`, `Content-Disposition` and `Location`.
- [MDN: HTTP redirections](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections) — `303`, `307`, `308` and the others, and when to use each.

<!-- next:start -->

**Next:**

- [Path and Query extractors](../a2-axum/path-and-query-extractors.md)

<!-- next:end -->
