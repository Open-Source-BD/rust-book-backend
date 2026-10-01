# JSON with serde

> **Beginner** · Part A2 · Axum

## By the end of this lesson

- You can send a Rust struct as JSON and receive JSON into a struct.
- You can make a JSON field optional with a default.
- You know the three ways a JSON request can be rejected (400, 415, 422).

## What & why

`Json<T>` in and out: serde turns structs into JSON and back, and Axum rejects bad bodies before your handler runs.

Apps talk to backends in [JSON](../glossary.md#json), the text format you met in
[How a web backend works, Step 5](../part-0-start/how-a-web-backend-works.md#step-5-json-the-language-of-the-body):
objects in braces, `"name": value` pairs, strings in double quotes. A phone app that wants a book
sends `GET /books/sample` and expects an answer like `{"id":1,"title":"Dune","in_stock":true}`. A
form that adds a book sends a `POST` with a body like `{"title":"Emma"}`.

Your Rust code doesn't think in JSON text, though. It thinks in **structs**: a `Book` with an `id`
that is a number, a `title` that is a `String`, and an `in_stock` that is `true` or `false`. So
every JSON backend needs a **translator** between the two, like an interpreter at a meeting who
turns each sentence from one language into the other, both ways, without changing its meaning.

In Rust, that translator is [**serde**](../glossary.md#serde), the crate you met in
[Path and Query extractors](path-and-query-extractors.md). It does both directions:

- **Struct → JSON** is to [**serialize**](../glossary.md#serialize). You ask for it with
  `#[derive(Serialize)]`.
- **JSON → struct** is to [**deserialize**](../glossary.md#deserialize). You ask for it with
  `#[derive(Deserialize)]`, as `Query` already did.

Axum puts serde to work through one type, **`Json<T>`**. This lesson has three ideas:

1. **Return** `Json(book)` from a [handler](../glossary.md#handler), and the struct goes out as JSON.
2. **Take** `Json(input)` as a handler parameter, an [extractor](../glossary.md#extractor), and a
   JSON body comes in as a struct, with a default for any field you mark optional.
3. When the body is wrong, Axum answers for you, **before** your handler runs, with one of three
   [status codes](../glossary.md#status-code): `400`, `415` or `422`. Each means something
   different, and you'll see all three for real.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/json-and-serde`. Its `Cargo.toml` has the same lines as in
Path and Query extractors: nothing new to install.

```toml
{{#include ../../code/topics/json-and-serde/Cargo.toml}}
```

### Line by line

`[package]` · `name = "json-and-serde"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p json-and-serde`.
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
- **Why:** the two crates every Axum server needs. `Json` comes with Axum.
- **How:** the versions are written once, in `code/Cargo.toml`. Axum's `Json` reads and writes JSON
  text with the help of a crate called `serde_json`. Axum brings that crate in itself, through a
  feature it switches on by default, so this list doesn't need a `serde_json` line.
- **Remove it and…** (`axum`) every `use axum::…` line fails with ``unresolved import `axum` ``.

`serde.workspace = true`
- **What:** serde, with its `derive` feature, from the workspace
  ([Path and Query extractors, Step 2](path-and-query-extractors.md#step-2-the-serde-line-in-the-workspace)
  shows that line).
- **Why:** `#[derive(Serialize)]` and `#[derive(Deserialize)]` come from serde, not from Axum.
- **How:** the `derive` feature adds the two `#[derive(…)]` shortcuts this lesson uses.
- **Remove it and…** ``error[E0432]: unresolved import `serde` ``, pointing at the
  `use serde::…;` line.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the response bodies, without
  opening a port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the dependencies come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, and the test crates from
`cargo add --dev tower http-body-util --features tower/util`. There's no `cargo add serde_json`:
`axum` already brings it.

### Step 2: the `use` lines

```rust,noplayground
{{#include ../../code/topics/json-and-serde/src/main.rs:imports}}
```

📁 Full code: code/topics/json-and-serde · ▶ Run it: `cd code && cargo run -p json-and-serde`

### Line by line

`use axum::{` · `};`
- **What:** brings four names from Axum into this file.
- **Why:** so the code can write `Json` instead of `axum::Json`.
- **How:** the braces list several names from one crate; `cargo fmt` puts them over several lines.
- **Remove it and…** none of the four names is known, and the build stops with many errors.

`Json`
- **What:** Axum's JSON type: a box that holds one value of yours, and knows how to carry it as
  JSON.
- **Why:** it's the whole lesson. The same type works in both directions: as a return value, it
  **sends** JSON; as a parameter, it **receives** JSON.
- **How:** `Json` sits at the top of Axum, `axum::Json`, because it's a response as well as an
  extractor. (Axum also lists it in `axum::extract`, next to `Path` and `Query`: both names mean
  the same type.)
- **Remove it and…** ``cannot find type `Json` in this scope``, and the same for the function
  `Json(…)`.

`Router` · `routing::{get, post}`
- **What:** Axum's table of [routes](../glossary.md#route), and the functions that make a route
  answer `GET` or `POST`.
- **Why:** this lesson has one route of each: `GET` to read a book, `POST` to send one.
- **How:** exactly as in [Routes and HTTP methods](routes-and-methods.md).
- **Remove it and…** (`post`) ``cannot find function `post` in this scope``.

`use serde::{Deserialize, Serialize};`
- **What:** brings in serde's two abilities, each with its `#[derive(…)]` shortcut.
- **Why:** Step 3 puts `Serialize` on one struct and `Deserialize` on the other.
- **How:** a separate `use` line, because it comes from a different crate: serde, not Axum.
- **Remove it and…** ``cannot find derive macro `Serialize` in this scope``, and the same for
  `Deserialize`.

### Step 3: two structs, one for each direction

A book that goes **out** to the client, and a new book that comes **in** from the client:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/src/main.rs:types}}
```

📁 Full code: code/topics/json-and-serde · ▶ Run it: `cd code && cargo run -p json-and-serde`

### Line by line

`#[derive(Serialize)]`
- **What:** asks serde to write the code that turns a `Book` into JSON text.
- **Why:** a `Book` is what the server **sends**, so it must be able to become JSON.
- **How:** as with `Deserialize` in the last lesson, `#[derive(…)]` means "write this ability for
  me". `Serialize` is a
  [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html), a
  named ability; derive writes it by looking at the fields, and each field becomes one
  `"name": value` pair in the JSON object. Rust for Humans'
  [Derive traits](https://open-source-bd.github.io/rustbook-for-human/language-basics/derive-traits.html)
  lesson covers `derive` in full.
- **Remove it and…** the build stops at both routes with the ``Handler<_, _>`` error you met in
  [Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes), plus a warning
  ``unused import: `Serialize` ``. *Common mistakes* below shows the same error, and how to get a
  clearer one.

`struct Book {` · `id: u32,` · `title: String,` · `in_stock: bool,` · `}`
- **What:** a book as the server shows it: a number, a piece of text, and yes-or-no.
- **Why:** these are the three facts a client needs about a book.
- **How:** each Rust type becomes the matching JSON type: `u32` becomes a JSON number (`1`),
  `String` becomes a JSON string (`"Dune"`), and `bool` becomes `true` or `false`. The JSON names
  are the field names, exactly as written: `in_stock`, with its underscore.
- **Remove it and…** (a field, such as `in_stock`) the JSON has no `in_stock` pair, and every
  client that reads it gets nothing there.

`#[derive(Deserialize)]`
- **What:** asks serde to write the code that builds a `NewBook` out of JSON text.
- **Why:** a `NewBook` is what the server **receives**, so it must be able to be built from JSON.
- **How:** the same `Deserialize` as for `Query` in the last lesson: the JSON's names must match
  the field names.
- **Remove it and…** the first error is ``cannot find attribute `serde` in this scope``, pointing at
  `#[serde(default)]` below: a `#[serde(…)]` line only means something to serde's derive, so
  without the derive, Rust doesn't know the word. Then comes the ``Handler<_, _>`` error at
  `post(create)`: `Json` can only fill a type that knows how to be deserialized.

`struct NewBook {` · `title: String,` · `}`
- **What:** a book as the client sends it: a title, and maybe `in_stock`. There's **no `id`**.
- **Why:** the server decides the id, not the client. If `NewBook` had an `id` field, any client
  could pick its own number, or take one that's already used. Leaving the field out makes that
  impossible: there's nowhere for an `id` to go.
- **How:** `title: String` is **required**: a JSON body without `"title"` is refused, as Run it
  shows.
- **Remove it and…** (`title`) `input.title` in Step 4 stops the build with
  ``no field `title` on type `NewBook` ``.

`#[serde(default)]` · `in_stock: bool,`
- **What:** makes `in_stock` **optional** in the JSON. When it's missing, serde uses the type's
  default value, which for a `bool` is `false`.
- **Why:** a client adding a book it hasn't got yet shouldn't have to say `"in_stock": false`.
- **How:** `#[serde(…)]` is an instruction to serde about the line below it, here the field
  `in_stock`. `default` means "if this name isn't in the JSON, fill it with the default". Every
  basic type has one: `false` for `bool`, `0` for numbers, an empty `String` for text.
- **Remove it and…** `in_stock` becomes required: `{"title":"Emma"}` gets `422`, with
  ``Failed to deserialize the JSON body into the target type: missing field `in_stock` at line 1
  column 16`` (tried in a scratch copy).

### Step 4: the handlers, `Json` out and `Json` in

```rust,noplayground
{{#include ../../code/topics/json-and-serde/src/main.rs:handlers}}
```

📁 Full code: code/topics/json-and-serde · ▶ Run it: `cd code && cargo run -p json-and-serde`

### Line by line

`async fn sample() -> Json<Book> {`
- **What:** a handler that returns a `Book`, wrapped in `Json`.
- **Why:** `Json<Book>` says "answer with this book, as JSON".
- **How:** `Json<Book>` has the `IntoResponse` ability from
  [Handlers and IntoResponse](handlers-and-into-response.md), as long as `Book` has `Serialize`.
  Axum serializes the book into the body and sets the header `content-type: application/json`.
- **Remove it and…** (write `-> Book`) the build stops with the ``Handler<_, _>`` error: a bare
  `Book` doesn't know how to become a response. `Json` is what gives it that.

`Json(Book {` · `id: 1,` · `title: "Dune".to_string(),` · `in_stock: true,` · `})`
- **What:** builds a `Book` and puts it in the `Json` box.
- **Why:** this is the one sample book the route always answers with.
- **How:** `Json(…)` wraps any value, the same box-and-contents idea as `Path(42)`. `"Dune"` is a
  `&'static str`, and `.to_string()` turns it into the `String` the field wants.
- **Remove it and…** (the `Json(` … `)` around it) ``mismatched types``: the function promised a
  `Json<Book>` and found a plain `Book`.

`async fn create(Json(input): Json<NewBook>) -> Json<Book> {`
- **What:** a handler that **receives** a `NewBook` as JSON and answers with a `Book` as JSON.
- **Why:** this is how a client sends data to your server: in the body of a `POST`.
- **How:** read it in the two halves from Path and Query extractors. The type, `Json<NewBook>`, asks
  Axum for the JSON extractor, which reads the request's **body** and builds a `NewBook`. The
  pattern, `Json(input)`, opens the box, so `input` **is** the `NewBook`.
- **Remove it and…** (the parameter) nothing reads the body, and `input` doesn't exist.

`Json(Book {` · `id: 2,` · `title: input.title,` · `in_stock: input.in_stock,` · `})`
- **What:** builds the new `Book` from what the client sent, plus an id the server chose.
- **Why:** it shows the struct arrived, field by field, and it shows the server adding the id.
- **How:** `input.title` and `input.in_stock` take the two fields out of the `NewBook`. The id is
  always `2` for now, because there's nowhere to store books yet. The next lesson,
  [Shared state](shared-state.md), keeps a list of books in memory; *Part A3* lets Postgres choose
  each id.
- **Remove it and…** (`title: input.title,`) the build stops with
  ``missing field `title` in initializer of `Book` ``: a Rust struct needs every field.

### Step 5: what happens to a JSON body

Before `create` runs, the `Json` extractor checks the body **three** times, in this order. Each
check that fails has its own status code:

```text
POST /books  (a body, and a content-type header)
  │  1. is the body labelled JSON?       no → 415 Unsupported Media Type
  ▼
the body, as text
  │  2. is the text well-formed JSON?    no → 400 Bad Request
  ▼
a JSON object
  │  3. does it fit a NewBook?           no → 422 Unprocessable Entity
  ▼
create(Json(NewBook { … }))
  │  4. your handler runs, and returns Json(Book { … })
  ▼
{"id":2,"title":"Emma","in_stock":false}   5. serialized, sent as 200 OK
```

### Line by line

`POST /books` · `1. is the body labelled JSON?` · `415`
- **What:** the first check reads the request's `content-type` header, not the body.
- **Why:** a body can be anything: JSON, a web form, a photo. The header is where the client says
  which it is, and the `Json` extractor only accepts `application/json`.
- **How:** if the header is missing or names another format, Axum answers `415 Unsupported Media
  Type`: "I don't accept that kind of body". It doesn't even look at the text.
- **Remove it and…** (this check) a photo, or a web form, would be read as if it were JSON.

`the body, as text` · `2. is the text well-formed JSON?` · `400`
- **What:** the second check reads the text and checks the JSON **grammar**: braces closed,
  strings in double quotes, commas between pairs.
- **Why:** text that isn't JSON at all can't be understood, whatever your struct looks like.
- **How:** if the grammar is broken, Axum answers `400 Bad Request`, the same code as a bad path
  value in the last lesson: "your request is broken".
- **Remove it and…** there would be nothing to fit into a struct.

`a JSON object` · `3. does it fit a NewBook?` · `422`
- **What:** the third check compares the JSON with your struct: is every required name there, and
  is every value the right type?
- **Why:** `{"price":5}` is perfect JSON, but it isn't a new book.
- **How:** if the JSON doesn't fit, Axum answers `422 Unprocessable Entity`: "I understood what you
  sent, but I can't use it". This is where `#[serde(default)]` matters: a missing `in_stock` is
  filled in, not refused.
- **Remove it and…** your handler could receive a book with no title.

`create(Json(NewBook { … }))` · `4. your handler runs`
- **What:** only now does your code run, with a real `NewBook`.
- **Why:** every check passed, so the handler can trust `input` completely.
- **How:** Axum calls `create` with the filled-in `Json` box.
- **Remove it and…** (any of the checks) your handler would have to do that checking itself.

`{"id":2,…}` · `5. serialized, sent as 200 OK`
- **What:** the `Book` the handler returned, turned into JSON text.
- **Why:** this is the other direction: `Serialize` at work.
- **How:** `Json<Book>` becomes a response with status `200 OK`, the header
  `content-type: application/json`, and the JSON as its body.
- **Remove it and…** the client gets no answer.

### Step 6: the Router

```rust,noplayground
{{#include ../../code/topics/json-and-serde/src/main.rs:app}}
```

📁 Full code: code/topics/json-and-serde · ▶ Run it: `cd code && cargo run -p json-and-serde`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router, starting from an empty one.
- **Why:** a separate function, so `main` and the tests use the same Router.
- **How:** exactly as in Hello, Axum.
- **Remove it and…** there's no Router to add routes to.

`.route("/books/sample", get(sample))`
- **What:** `GET /books/sample` answers with the sample book, as JSON.
- **Why:** a `GET` only reads, and has no body, so `sample` takes no parameters.
- **How:** `get(…)` makes the route answer the `GET` method.
- **Remove it and…** `/books/sample` gets `404`.

`.route("/books", post(create))`
- **What:** `POST /books` receives a new book, as JSON.
- **Why:** `POST` is the method for "here's something new", and its body carries the JSON.
- **How:** `post(…)` makes the route answer the `POST` method. A `GET /books` gets
  `405 Method Not Allowed`, as in [Routes and HTTP methods](routes-and-methods.md).
- **Remove it and…** `POST /books` gets `404`.

### Step 7: `main`, and run it

`main` is the same as in every Axum lesson:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/src/main.rs:main}}
```

📁 Full code: code/topics/json-and-serde · ▶ Run it: `cd code && cargo run -p json-and-serde`

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
- **How:** for every request, Axum finds the route, runs the handler's extractors, and then, only
  if they all succeed, calls the handler.
- **Remove it and…** the program prints `Listening…` and exits at once.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p json-and-serde
```

```text
   Compiling json-and-serde v0.1.0 (/Users/you/rust-book-backend/code/topics/json-and-serde)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.98s
     Running `target/debug/json-and-serde`
Listening on http://127.0.0.1:3000
```

In the second terminal, ask for the sample book:

```bash
{{#include ../../code/topics/json-and-serde/http/01-json-out.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/01-json-out.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`content-type: application/json`:** Axum set this header for you, because the handler returned
  a `Json`. It's the label from How a web backend works: "this body is JSON". A browser or an app
  reads it and knows to parse the body as JSON instead of showing it as text. Compare
  `text/plain; charset=utf-8`, which you got for a `String`.
- **`content-length: 39`:** the body is 39 bytes long.
- **The body:** `{"id":1,"title":"Dune","in_stock":true}`. The `Book`, serialized: one pair per
  field, in the order the struct lists them, names exactly as written. It's compact, with no spaces
  or line breaks: machines don't need them, and every byte saved is a byte not sent.

Now send a book in. `-X POST` picks the method, `-H` adds the header that labels the body as JSON,
and `-d` is the body. The JSON is inside **single** quotes, so the shell passes its double quotes
to curl untouched.

```bash
{{#include ../../code/topics/json-and-serde/http/02-json-in.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/02-json-in.out}}
```

- **The full body:** `{"id":2,"title":"Emma","in_stock":true}`. Your JSON became a `NewBook`,
  `create` built a `Book` from it with the server's id, and the `Book` went back out as JSON.
- **Without `in_stock`:** `"in_stock":false`. The field was missing, so `#[serde(default)]` filled
  in `false`, and the request was accepted. (The body is one byte longer: `false` has one letter
  more than `true`.)

Now break it, three ways. Each request below fails a different one of the three checks from Step 5:

```bash
{{#include ../../code/topics/json-and-serde/http/03-rejections.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/03-rejections.out}}
```

Every answer here came from Axum; `create` never ran. They're plain text, so the header is
`content-type: text/plain; charset=utf-8`. Read them from the top:

- **`{"in_stock":true}`: `422 Unprocessable Entity`.** Perfect JSON, but no `"title"`, and a
  `NewBook` must have one. Word by word: `Failed to deserialize the JSON body into the target
  type` means "I couldn't build your struct (the *target type*, `NewBook`) from this JSON"; ``missing
  field `title` `` is the reason; `at line 1 column 17` is where serde was when it noticed. The body
  is 17 characters long, so column 17 is the closing `}`: serde only knows a field is missing once
  the object has ended without it.
- **`{"title":"Emma"`: `400 Bad Request`.** The closing `}` is missing, so it isn't JSON at all.
  `Failed to parse the request body as JSON` means the grammar check failed. `EOF` stands for
  *end of file*, meaning "the end of the text": serde ran out of text `while parsing an object`,
  still waiting for the `}`. Column 15 is the last character, the closing `"` of `"Emma"`.
- **No `-H`: `415 Unsupported Media Type`.** The JSON is fine, but the request doesn't say it's
  JSON: ``Expected request with `Content-Type: application/json` ``. Without `-H`, curl's `-d`
  labels the body `application/x-www-form-urlencoded`, the format of an old-style web form. The
  `Json` extractor refuses any label but JSON, and gives the same `415` when there's no
  `content-type` at all.

So the three codes are the three checks, in order: **415**, the wrong label; **400**, broken JSON;
**422**, good JSON with the wrong shape. The first two usually mean a bug in the client's code.
The third often means the person filling in a form left something out, so a real app shows it to
them.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If `/books/sample` answers with `content-type: application/json` and the JSON book, `Json` is
  serializing your struct.
- ✅ If the `POST` without `in_stock` answers `"in_stock":false`, your default works.
- ✅ If the three broken requests answer `422`, `400` and `415`, in that order, you've seen every
  way a JSON body can be refused.
- ❌ If a `POST` you expected to work answers `415`, check the `-H 'content-type:
  application/json'`: without it, or with a typo in it, Axum refuses the body.
- ❌ If a `POST` you typed yourself answers `400 Bad Request`, compare its JSON with the script's,
  character by character: a missing `}` or `"`, or single quotes inside the JSON, are the usual
  causes. *Common mistakes* shows the single-quote one.

## You might be wondering…

**"Why two structs? Couldn't `Book` do both jobs?"**
It could, with `#[derive(Serialize, Deserialize)]`, but then a client could send its own `"id"`.
The two structs say what each side is allowed to decide: the client chooses the title and the
stock, the server chooses the id. *More examples* shows what happens today when a client sends
`"id": 99` anyway: it's ignored, and the answer has the server's id. Separate input and output
types are the normal pattern in real APIs, and you'll see more fields differ as the book goes on,
such as a creation date only the server knows.

**"Why is it `in_stock` with an underscore in the JSON?"**
Because serde uses the field name exactly as you wrote it, and Rust names fields in *snake_case*,
lower case with underscores. Many JSON APIs, especially ones used from JavaScript, prefer
*camelCase* instead: `inStock`. You don't have to rename your fields for that: one line,
`#[serde(rename_all = "camelCase")]`, changes the JSON names and leaves the Rust names alone.
*More examples* shows it.

**"Does the order of the fields matter?"**
Not when you receive JSON: a JSON object is a set of named pairs, and serde finds each by its name.
`{"in_stock":true,"title":"Emma"}` works exactly like `{"title":"Emma","in_stock":true}`; *More
examples* shows it for real. When you send JSON, serde writes the pairs in the order your struct
lists its fields, so the output is always the same, which makes it easy to read and to test.

**"What happens to fields I didn't ask for?"**
They're ignored, like unknown query options in the last lesson. `{"title":"Emma","author":"Jane
Austen"}` is accepted, and the author goes nowhere. That keeps old servers working when newer
clients send more. If you'd rather refuse unknown fields, so that a misspelt name like `"titel"`
is caught, serde has `#[serde(deny_unknown_fields)]`: *More examples* shows the `422` it gives.

**"What's the difference between `#[serde(default)]` and `Option`?"**
Both make a field optional. With `#[serde(default)]`, a missing `in_stock` becomes `false`, and
your code can't tell whether the client said `false` or said nothing. An `Option<bool>` field is
`None` when it's missing, so you can tell the two apart. Use `default` when a missing value really
means the default, and `Option` when "not given" means something different.

**"Why are Axum's error answers plain text, not JSON?"**
Because Axum doesn't know how your API writes its errors, so its built-in rejections use the
simplest format there is. Most JSON APIs answer errors in JSON too, such as
`{"error":"missing field title"}`. [Error handling in Axum](error-handling-in-axum.md) shows how to turn every rejection into
the error format you choose.

## Coming from another language?

Every web framework turns JSON into objects and back. The differences are what gets checked, and
who answers when the check fails.

**Express (Node.js).** `express.json()` parses the body; `res.json()` sends JSON:

```js
app.use(express.json());

app.post("/books", (req, res) => {
  const { title, in_stock = false } = req.body; // nothing checks that title is there
  res.json({ id: 2, title, in_stock });
});
```

Broken JSON gets a `400` from `express.json()`. A body without the JSON `content-type` isn't parsed
at all: `req.body` isn't filled in, and your handler runs anyway. A missing field is `undefined`, and
it's yours to notice. Axum does all three checks before your handler runs.

**Flask and FastAPI (Python).** In Flask, `request.get_json()` answers `415` for a non-JSON
`content-type` and `400` for broken JSON, the same as Axum, but it gives you a dictionary, so
checking the fields is yours. Returning a `dict` sends it as JSON. FastAPI is the closest to Axum:

```python
from pydantic import BaseModel

class NewBook(BaseModel):
    title: str
    in_stock: bool = False

@app.post("/books")
def create(book: NewBook):
    return {"id": 2, "title": book.title, "in_stock": book.in_stock}
```

The class is FastAPI's `NewBook` struct, `= False` is its `#[serde(default)]`, and a missing
`title` gets a `422`, as in Axum, but with a JSON body describing the problem.

**Spring Boot (Java).** `@RequestBody` reads JSON into a class, using a library called Jackson,
and a returned object is sent as JSON:

```java
record NewBook(String title, boolean inStock) {}
record Book(int id, String title, boolean inStock) {}

@PostMapping("/books")
public Book create(@RequestBody NewBook input) {
    return new Book(2, input.title(), input.inStock());
}
```

Jackson uses the Java names, which are camelCase by convention, so here the JSON says `"inStock"`,
not `"in_stock"`. A wrong `content-type` gets `415` and broken JSON gets `400`, as in Axum. But a
missing `title` isn't an error: it arrives as `null`, and a missing `inStock` as `false`, unless
you add [validation](../glossary.md#validation) annotations. Spring Boot ignores unknown fields,
like serde does by default.

**Go (`net/http` and Gin).** The standard library's `encoding/json` fills a struct, with **tags**
naming the JSON keys:

```go
type NewBook struct {
    Title   string `json:"title"`
    InStock bool   `json:"in_stock"`
}

func create(w http.ResponseWriter, r *http.Request) {
    var input NewBook
    if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
        http.Error(w, err.Error(), http.StatusBadRequest)
        return
    }
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(map[string]any{"id": 2, "title": input.Title, "in_stock": input.InStock})
}
```

A missing field quietly gets its zero value (`""` or `false`), and the `content-type` isn't
checked unless you check it. `decoder.DisallowUnknownFields()` is Go's `deny_unknown_fields`. Gin's
`c.ShouldBindJSON(&input)` fills a struct, and a `binding:"required"` tag makes a field required;
`c.JSON(200, book)` sends one.

## Common mistakes

**Putting `Json` before `Path`.**
You write a handler that updates book 7: the id from the path, the new details from the body. It
seems natural to write the body first:

```rust,noplayground,ignore
async fn update(Json(input): Json<NewBook>, Path(id): Path<u32>) -> Json<Book> {
```

This error was captured in a scratch copy of this lesson's code, a project called `my-json`, with
this handler added on the route `.route("/books/{id}", put(update))`, so its line numbers are from
that file:

```text
error[E0277]: the trait bound `fn(Json<NewBook>, Path<u32>) -> ... {update}: Handler<_, _>` is not satisfied
   --> src/main.rs:50:35
    |
 50 |         .route("/books/{id}", put(update))
    |                               --- ^^^^^^ unsatisfied trait bound
    |                               |
    |                               required by a bound introduced by this call
    |
    = help: the trait `Handler<_, _>` is not implemented for fn item `fn(Json<NewBook>, axum::extract::Path<u32>) -> impl Future<Output = Json<Book>> {update}`
    = note: Consider using `#[axum::debug_handler]` to improve the error message
…
```

The same `Handler<_, _>` error as in the last two lessons: it says *that* `update` isn't a valid
handler, not *why*. Follow the `note:` line. Add `#[axum::debug_handler]` on the line above
`async fn update` (it needs Axum's `macros` feature, as in
[Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes)), and a new error comes
first:

```text
error: `Json<_>` consumes the request body and thus must be the last argument to the handler function
  --> src/main.rs:39:30
   |
39 | async fn update(Json(input): Json<NewBook>, Path(id): Path<u32>) -> Json<Book> {
   |                              ^^^^
```

That's the rule, in Axum's own words. A request's **body** arrives as a stream of bytes, and it can
be read only **once**: after `Json` has read it, it's gone. So Axum splits extractors into two
kinds. Most, like `Path` and `Query`, only look at the request's **head**, the method, the address
and the headers, which any number of extractors can read. A body extractor like `Json` **uses up**
the body, so there can be only one, and it must be the **last** parameter, after every extractor
that reads the head. **Fix:** put `Path` first and `Json` last
(`cargo run -p json-and-serde --example json-path-then-body`):

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-path-then-body.rs:update}}

{{#include ../../code/topics/json-and-serde/examples/json-path-then-body.rs:app}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/72-path-then-body.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/72-path-then-body.out}}
```

The `7` came from the path, and the rest from the body. `PUT` is the method for "replace this
thing", from [Routes and HTTP methods](routes-and-methods.md).

**Returning a struct that can only be received.**
You want to send back exactly what the client sent, so you return the `NewBook` itself:

```rust,noplayground,ignore
async fn echo(Json(input): Json<NewBook>) -> Json<NewBook> {
    Json(input)
}
```

In the same scratch copy, on the route `.route("/echo", post(echo))`, it fails with the
`Handler<_, _>` error again, this time pointing at `post(echo)` and naming
``Output = Json<NewBook>``. With `#[axum::debug_handler]` above `async fn echo`, the first error
names the real problem:

```text
error[E0277]: the trait bound `NewBook: serde::Serialize` is not satisfied
  --> src/main.rs:47:46
   |
47 | async fn echo(Json(input): Json<NewBook>) -> Json<NewBook> {
   |                                              ^^^^ unsatisfied trait bound
   |
help: the trait `Serialize` is not implemented for `NewBook`
  --> src/main.rs:16:1
   |
16 | struct NewBook {
   | ^^^^^^^^^^^^^^
   = note: for local types consider adding `#[derive(serde::Serialize)]` to your `NewBook` type
…
```

`NewBook` has `Deserialize`, so it can come **in**, but not `Serialize`, so it can't go **out**.
The two directions are two separate abilities, and each struct has only the ones you derive. Its
`note:` even gives the fix. **Fix:** return a `Book`, as `create` does, or, if you really want to
send a `NewBook` back, derive both: `#[derive(Serialize, Deserialize)]`.

**Sending a value of the wrong type.**
The client sends `"yes"` where `in_stock` expects `true` or `false`:

```bash
{{#include ../../code/topics/json-and-serde/http/70-wrong-type.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/70-wrong-type.out}}
```

A `422`: the JSON is fine, and so is the name, but a `bool` only accepts `true` or `false`. This
time the reason starts with the field's name, `in_stock:`, so you know exactly which value was
wrong, then says ``invalid type: string "yes", expected a boolean``. Numbers are the same: `"42"`,
in quotes, is a string, and a `u32` field refuses it. **Fix:** send the JSON type your field
expects: `"in_stock":true`.

**Writing JSON with single quotes.**
Python dictionaries and JavaScript both accept `'Emma'`, so it's easy to type:

```bash
{{#include ../../code/topics/json-and-serde/http/71-single-quotes.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/71-single-quotes.out}}
```

A `400`: this isn't JSON. As How a web backend works said, JSON strings, and names, always use
double quotes. ``key must be a string`` means "after `{`, I expected a name in double quotes",
and column 2 is the first `'`. **Fix:** double quotes inside the JSON, and single quotes around the
whole `-d` value, as in every other script in this lesson.

## More examples

Each example is a complete program in `code/topics/json-and-serde/examples/`. Stop any running
server with `Ctrl+C`, start the example with the command shown, and send its requests from your
second terminal.

### `rename_all = "camelCase"`: JSON names in a different style

The same two structs, with one extra line each
(`cargo run -p json-and-serde --example json-camel-case`). The handlers and routes are the same as
in the lesson's code:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-camel-case.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-camel-case.rs:types}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/50-camel-case.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/50-camel-case.out}}
```

`#[serde(rename_all = "camelCase")]` sits on the **struct**, so it applies to every field: `in_stock`
is `inStock` in the JSON, in both directions, while your Rust code still says `in_stock`. `id` and
`title` are one word, so they don't change. Look at the third request: it sent `in_stock`, the old
name, which is now an unknown field, so it was ignored, and the default made it `false`. After a
rename, clients must use the new names.

### Extra fields, field order, and `deny_unknown_fields`

First, the lesson's own server (`cargo run -p json-and-serde`), sent fields in a different order,
a field it doesn't know, and an `id`:

```bash
{{#include ../../code/topics/json-and-serde/http/51-order-and-extras.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/51-order-and-extras.out}}
```

All three are accepted. The reversed order makes no difference. `"author"` is ignored. And
`"id":99` is ignored too, because `NewBook` has no `id` field: the answer has the server's id, `2`.
That's the two-struct design working.

Now the same `NewBook` with `#[serde(deny_unknown_fields)]` on it
(`cargo run -p json-and-serde --example json-deny-unknown`):

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-deny-unknown.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-deny-unknown.rs:new_book}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/52-deny-unknown.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/52-deny-unknown.out}}
```

Now `"author"` is a `422`. The reason names the field (`author:`), says what went wrong
(``unknown field `author` ``), and lists what this struct does accept
(``expected `title` or `in_stock` ``), which is exactly what a client needs to fix a typo. A body
with only known fields still works. Choose it for inputs where a misspelt field would otherwise be
silently lost.

### A struct inside a struct

A book has an author, and an author has facts of their own
(`cargo run -p json-and-serde --example json-nested`):

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-nested.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-nested.rs:types}}

{{#include ../../code/topics/json-and-serde/examples/json-nested.rs:sample}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/53-nested.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/53-nested.out}}
```

A struct field becomes a JSON **object** inside the object: `"author":{"name":…,"born":1920}`.
`Author` needs its own `#[derive(Serialize)]`, because serde serializes a `Book` by asking each
field to serialize itself. This works for any depth, and in the other direction too, with
`Deserialize` on both structs.

### `Vec<Book>`: a JSON array

A list of books (`cargo run -p json-and-serde --example json-list`):

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-list.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-list.rs:list}}

{{#include ../../code/topics/json-and-serde/examples/json-list.rs:app}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/54-list.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/54-list.out}}
```

A `Vec`, Rust's growable list, becomes a JSON **array**: `[` … `]`, with one object per book, in
the list's order. `vec![…]` builds a `Vec` from the values inside the brackets. `Json<Vec<Book>>`
works because `Book` has `Serialize`, and serde already knows how to serialize a `Vec` of anything
that has it. This is the shape a "list all books" endpoint usually answers with.

## Your turn

Each solution is also a complete program in `code/topics/json-and-serde/examples/`, with a test
inside.

### 🟢 Guided

Add a `pages: u32` field to both `Book` and `NewBook`. The sample book, Dune, has `412` pages, and
`POST /books` with `{"title":"Emma","pages":474}` should answer with `"pages":474`. You need one new
line in each struct, and one in each place a `Book` is built.

<details><summary>Solution</summary>

`cargo run -p json-and-serde --example json-pages`:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-pages.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-pages.rs:types}}

{{#include ../../code/topics/json-and-serde/examples/json-pages.rs:handlers}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/90-pages.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/90-pages.out}}
```

`pages` goes out last in the JSON because it's the last field of `Book`. In `NewBook` it has no
`#[serde(default)]`, so it's required: a body without `"pages"` now gets a `422`, which the
solution's second test checks. If you forgot to add `pages:` in `sample` or `create`, the compiler
stopped you with ``missing field `pages` in initializer of `Book` ``.

</details>

### 🟡 Tweak

Make `in_stock` default to `true` instead of `false`. `#[serde(default)]` always uses the type's
default, and a `bool`'s is `false`, so you need its other form, `#[serde(default = "yes")]`: it
names a function, `yes`, that serde calls to get the value. Write that function. It takes nothing
and returns a `bool`.

<details><summary>Solution</summary>

`cargo run -p json-and-serde --example json-default-true`:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-default-true.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-default-true.rs:new_book}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/91-default-true.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/91-default-true.out}}
```

A missing `in_stock` is now `true`, and one the client sends, even `false`, is kept. The function's
name is written as **text**, in quotes, inside the attribute: serde looks up the function by that
name. `fn yes() -> bool { true }` has no `return`: the last expression in a function's body, `true`
here, is what it returns.

</details>

### 🔴 From scratch

Write `POST /orders`. It receives an order with a list of items, such as
`{"items":[{"book_id":1,"qty":2}]}`, and answers with the total number of books ordered, as JSON:
`{"total_items":2}`. You need three structs: one item, one order holding a `Vec` of items, and one
answer. Hints: a JSON array fills a `Vec`, and a JSON object inside it fills a struct, as in the
nested example; `for item in &order.items` visits each item.

<details><summary>Solution</summary>

`cargo run -p json-and-serde --example json-orders`:

```rust,noplayground
{{#include ../../code/topics/json-and-serde/examples/json-orders.rs:imports}}

{{#include ../../code/topics/json-and-serde/examples/json-orders.rs:types}}

{{#include ../../code/topics/json-and-serde/examples/json-orders.rs:handler}}

{{#include ../../code/topics/json-and-serde/examples/json-orders.rs:app}}
```

```bash
{{#include ../../code/topics/json-and-serde/http/92-orders.sh}}
```

```text
{{#include ../../code/topics/json-and-serde/http/92-orders.out}}
```

Two structs come **in** (`Deserialize`) and one goes **out** (`Serialize`), each with only the
ability it needs. `items: Vec<OrderItem>` reads the JSON array, one `OrderItem` per object. The
loop adds up each `qty`; the `&` in `&order.items` borrows the list, because the loop only needs
to read it.
`Json(OrderSummary { total_items })` is short for `total_items: total_items`: when a variable has
the field's name, you can write it once.

One line needs a word: `#[allow(dead_code)]`. Without it, the compiler warns
``field `book_id` is never read``: serde fills it in, but no code reads it yet. The attribute tells
the compiler that's on purpose. The field still documents what the client sends, and a real shop
would use it to look up each book's price. (You could also leave `book_id` out of the struct
altogether: serde would ignore it, like any unknown field.)

</details>

## Quick check

<div class="quiz" data-topic="json-and-serde"></div>

## Remember this

- `#[derive(Serialize)]` lets a struct go **out** as JSON; `#[derive(Deserialize)]` lets one come
  **in**. A struct has only the directions you derive.
- Return `Json(value)` to answer with JSON and `content-type: application/json`. Take
  `Json(input): Json<T>` to read a JSON body, always as the **last** extractor.
- Use a separate input struct, without the fields the server decides, like `id`.
- `#[serde(default)]` makes a field optional; `#[serde(default = "fn_name")]` picks the default.
- **415** wrong `content-type` · **400** broken JSON · **422** JSON that doesn't fit your struct.
  Your handler never runs for any of them.

## Go deeper

- [Rust for Humans: Serde and JSON](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/serde-and-json.html)
- [Rust for Humans: Derive traits](https://open-source-bd.github.io/rustbook-for-human/language-basics/derive-traits.html)
- [serde.rs](https://serde.rs/) — serde's own guide: every attribute, such as rename_all, default and deny_unknown_fields.
- [axum::Json](https://docs.rs/axum/0.8.9/axum/struct.Json.html) — Official reference: the Json extractor and response, and its rejections.

<!-- next:start -->

**Next:**

- [Shared state](../a2-axum/shared-state.md)

<!-- next:end -->
