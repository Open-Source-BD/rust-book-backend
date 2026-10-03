# Custom extractors

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can write your own extractor with `FromRequestParts`.
- You can reject a request with your own status and message.
- You can protect a route by adding one argument to its handler.

## What & why

Write an `ApiKey` extractor with `FromRequestParts`: check a header once, reuse it on any handler, reject with 401 or 403.

Picture a club with a bouncer at the door. Some rooms are open to everyone; the others ask for the
same wristband. You could put a guard inside every private room, and teach each of them the same
three rules: no wristband, go away; a fake wristband, go away; a real one, come in. Or you put **one**
bouncer at the door who knows the rules, and every private room says only "wristband holders
only". Change the rules once, and every room follows.

Your API has the same need. Some routes should answer only programs that send a secret, an
[API key](../glossary.md#api-key): a password for a program, sent in a
[header](../glossary.md#header) such as `x-api-key: letmein`. Checking that header at the top of
every protected handler means copying the same lines again and again, and one day forgetting them
in one handler. In
[Middleware and Tower layers](middleware-and-tower-layers.md#a-middleware-that-refuses-requests-without-a-key)
you saw one way out: a [middleware](../glossary.md#middleware) that guards a whole group of
routes. This lesson shows the other way, the one Axum uses for everything a handler receives: an
[extractor](../glossary.md#extractor), like `Path`, `Query` and `Json`, but one that you write.

Three new ideas:

1. **`FromRequestParts`: your type, built from the request.** It's the
   [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
   behind `Path` and `Query`: "this type can be made from the request's head", its method, address
   and headers. Implement it for your own `ApiKey` type, and Axum can fill it in, the same way it
   fills a `Path<u32>`.
2. **`type Rejection`: your own "no".** When the check fails, your extractor answers instead of the
   handler, with the status and message you choose. That answer is called a
   **[rejection](../glossary.md#rejection)**: `401` when the key is missing, `403` when it's wrong.
3. **One argument protects a route.** `async fn secret(ApiKey(key): ApiKey)` can only run if a
   valid key came with the request. Any handler that takes an `ApiKey` is protected; any handler
   that doesn't, isn't.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/custom-extractors`. Its `Cargo.toml` has nothing new: writing an
extractor needs only Axum itself.

```toml
{{#include ../../code/topics/custom-extractors/Cargo.toml}}
```

### Line by line

`[package]` · `name = "custom-extractors"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p custom-extractors`.
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
- **Why:** Axum gives `Router`, the `FromRequestParts` trait and `StatusCode`; Tokio runs the server.
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

### Step 2: the `use` lines

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:imports}}
```

📁 Full code: code/topics/custom-extractors · ▶ Run it: `cd code && cargo run -p custom-extractors`

### Line by line

`use axum::{` · `Router,`
- **What:** opens the list of names this file takes from Axum, starting with `Router`.
- **Why:** `app()` builds a Router, as in every lesson.
- **How:** the braces let one `use` line bring in several names.
- **Remove it and…** (`Router`) ``cannot find type `Router` in this scope``.

`extract::FromRequestParts,`
- **What:** the trait we'll implement, from Axum's `extract` module, where `Path`, `Query` and the
  other extractors live.
- **Why:** to say "`ApiKey` can be built from a request", we have to name the trait that means it.
- **How:** a trait must be in scope (named by a `use`) before you can write `impl … for` it.
- **Remove it and…** ``error[E0405]: cannot find trait `FromRequestParts` in this scope``.

`http::{StatusCode, request::Parts},`
- **What:** two names from Axum's `http` module: `StatusCode` (`401`, `403`…) and `Parts`, the
  request's head.
- **Why:** the rejection carries a `StatusCode`, and the function that checks the key receives a
  `Parts`.
- **How:** `request::Parts` is the type `Parts`, inside `http`'s `request` module. A request is
  made of two pieces: its **parts** (method, address, headers) and its **body**. Step 3 reads the
  parts only.
- **Remove it and…** (`request::Parts`) ``cannot find type `Parts` in this scope``.

`routing::get,` · `};`
- **What:** `get`, to attach handlers to `GET` requests; then the list closes.
- **Why:** both routes in this lesson answer `GET`.
- **How:** as in [Routes and HTTP methods](routes-and-methods.md).
- **Remove it and…** ``cannot find function `get` in this scope``.

### Step 3: the extractor

Here's the bouncer. It's the longest listing in the lesson, so the Line by line goes slowly,
one or two lines at a time.

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:extractor}}
```

📁 Full code: code/topics/custom-extractors · ▶ Run it: `cd code && cargo run -p custom-extractors`

### Line by line

`struct ApiKey(String);`
- **What:** a new type, `ApiKey`, holding one `String`: the key that was sent.
- **Why:** the extractor needs a type of its own, so a handler can ask for it by name. Keeping the
  key inside lets the handler know **which** key came in, which matters once there are several
  (*More examples* has a list of keys).
- **How:** a *tuple struct*: a struct whose one field has no name, only a position. You open it
  with a pattern, `ApiKey(key)`, the same way you open `Path(id)`.
- **Remove it and…** every other line of the listing fails: there's no `ApiKey` to build.

`impl<S> FromRequestParts<S> for ApiKey`
- **What:** "here's how to build an `ApiKey` from a request's parts".
- **Why:** this line is what turns `ApiKey` into an extractor. When Axum sees a handler argument of
  type `ApiKey`, it looks for exactly this, and calls the function inside.
- **How:** a trait is a list of things a type promises it can do; `impl Trait for Type` keeps the
  promise for one type. `Path`, `Query` and `HeaderMap` all have an `impl FromRequestParts` inside
  Axum; yours is one more. The `<S>` is
  [generics](https://open-source-bd.github.io/rustbook-for-human/abstractions/generics.html):
  `S` stands for "the app's [state](../glossary.md#state) type, whatever it is". Some extractors,
  such as `State`, need to know that type. Ours doesn't use the state at all, so we write the code
  once **for every possible `S`**: the same `ApiKey` then works in a Router with no state, or with
  the `AppState` of [Shared state](shared-state.md), or any other.
- **Remove it and…** (the whole `impl`) the handler that takes `ApiKey` is rejected with the
  `Handler<_, _>` error you've met before: `ApiKey` isn't an extractor.

`where` · `S: Send + Sync,`
- **What:** a condition on `S`: "only for state types that are `Send` and `Sync`", that is, types
  that can safely be moved to, and shared between, threads.
- **Why:** Tokio runs requests on several threads, and may move a half-finished request from one
  to another. The function below receives `&S`, a reference to the state, and that reference may
  only cross threads if `S` is `Sync`. Without this line, Rust can't promise that, and stops.
- **How:** `where` lists conditions after the `impl` line. Every state a Router accepts is already
  `Send` and `Sync`, so this costs nothing in practice. *Common mistakes* shows the real error you
  get without it.
- **Remove it and…** ``error: future cannot be sent between threads safely``.

`type Rejection = (StatusCode, &'static str);`
- **What:** the type of the answer sent when the check fails: a status code and a message.
- **Why:** the trait asks "if this goes wrong, what do I send back?". Here, the same pair you've
  returned from handlers since
  [Handlers and IntoResponse](handlers-and-into-response.md): a tuple becomes a response with that
  status and that text.
- **How:** any type that implements `IntoResponse` can be a rejection. *More examples* uses an
  `AppError` instead, for JSON errors.
- **Remove it and…** ``error[E0046]: not all trait items implemented, missing: `Rejection` ``.

`async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {`
- **What:** the one function the trait asks for. Axum calls it **before** your handler, once per
  request.
- **Why:** this is where the check happens. It receives the request's head, `parts`, and the app's
  state, and returns either an `ApiKey` (`Ok`) or a rejection (`Err`).
- **How:** `Self` means "the type this `impl` is for", `ApiKey`, and `Self::Rejection` is the type
  chosen on the line above. `parts` is `&mut` because some extractors take things out of the head;
  ours only reads. `_state` starts with `_` because we don't use it: the underscore tells Rust that
  on purpose, so it doesn't warn. The function is `async`, like a handler, so an extractor may wait
  for something slow, such as a database.
- **Remove it and…** (`async`) ``error[E0277]: `Result<ApiKey, (StatusCode, &'static str)>` is not
  a future``: the trait expects a function that can be awaited.

`let value = parts` · `.headers` · `.get("x-api-key")`
- **What:** looks up the `x-api-key` header among the request's headers.
- **Why:** that's where clients put the key.
- **How:** `parts.headers` is every header of the request, and `.get(name)` gives `Some(value)` if
  that header was sent, or `None` if it wasn't. The name is matched without caring about upper or
  lower case, as for every header.
- **Remove it and…** there's nothing to check.

`.and_then(|value| value.to_str().ok())`
- **What:** turns the header's value into text, a `&str`.
- **Why:** a header value is raw bytes, and may hold characters that aren't plain text. We want to
  compare it with `"letmein"`, and keep it as a `String`.
- **How:** `.to_str()` succeeds only if the value is plain visible ASCII, and `.ok()` turns its
  `Result` into an `Option`: `Some(text)`, or `None` on failure. `.and_then` runs that only if there
  was a header, so we end with **one** `Option`: `Some("guess")`, or `None` if the header was
  missing **or** unreadable. A key such as `clé`, with an accent, gets the same `401` as no key at
  all. `|value| …` is a
  [closure](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html), a small
  function written in place.
- **Remove it and…** `value` stays a header value, not text, and `ApiKey(value.to_owned())` below
  fails with ``mismatched types``.

`.ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;`
- **What:** no key? Then stop here, and answer `401 Unauthorized` with `missing x-api-key header`.
- **Why:** `401` means "I don't know who you are", which is exactly the situation: no key, no idea
  who's calling.
- **How:** `.ok_or(error)` turns `Some(text)` into `Ok(text)`, and `None` into `Err(error)`. Then
  [`?`](https://open-source-bd.github.io/rustbook-for-human/abstractions/the-question-mark-operator.html)
  does its usual job: on `Ok`, it unwraps the text into `value`; on `Err`, it returns the error from
  the function at once. The error is our `Rejection`, so Axum sends it, and the handler never runs.
- **Remove it and…** `value` stays an `Option<&str>`, and the comparison below fails with
  ``mismatched types``. Remove only the `?`, and it's a `Result`, with the same error.

`if value == "letmein" {` · `Ok(ApiKey(value.to_owned()))`
- **What:** the right key? Then build the `ApiKey`, holding a copy of the key, and hand it over.
- **Why:** `Ok(…)` means "extraction worked": Axum takes the `ApiKey` out and calls the handler
  with it.
- **How:** `value` is a `&str` borrowed from the request, and `.to_owned()` makes a `String` the
  `ApiKey` owns. The key is written into the code to keep this lesson small; *You might be
  wondering…* says where it belongs in a real app, and *Common mistakes* says why `==` isn't the
  last word on comparing secrets.
- **Remove it and…** (`.to_owned()`) ``mismatched types``: `ApiKey` holds a `String`, not a
  borrowed `&str`.

`} else {` · `Err((StatusCode::FORBIDDEN, "wrong API key"))` · `}` · `}` · `}`
- **What:** a key came, but it's the wrong one: answer `403 Forbidden` with `wrong API key`.
- **Why:** `403` means "I see what you sent, and the answer is no". Sending the same key again
  won't help.
- **How:** `Err(…)` is the other way to finish: Axum turns the rejection into the response. The
  three `}` close the `if`/`else`, the function and the `impl`.
- **Remove it and…** (the `else` branch) the `if` has no answer for a wrong key, and the build fails
  with ``error[E0317]: `if` may be missing an `else` clause``: every path through the function must
  return a `Result`.

### Step 4: what happens to one request

Here's the bouncer's job, for one request to `/secret`, as a picture:

```text
GET /secret
  │
  ├─ no x-api-key header ──────► 401 missing x-api-key header   (handler not called)
  ├─ x-api-key: guess ─────────► 403 wrong API key              (handler not called)
  └─ x-api-key: letmein ───────► secret(ApiKey("letmein")) runs, 200
```

### Line by line

`GET /secret`
- **What:** a request arrives, and the Router picks the `secret` handler.
- **Why:** before calling it, Axum must fill in its arguments.
- **How:** the argument is an `ApiKey`, so Axum calls `ApiKey::from_request_parts`.
- **Remove it and…** no request, nothing happens.

`├─ no x-api-key header ──────► 401 missing x-api-key header   (handler not called)`
- **What:** the `.ok_or(…)?` line returns the `401` rejection.
- **Why:** with no key, there's no reason to run the handler at all.
- **How:** Axum sends the rejection as the response, and `secret` never runs.
- **Remove it and…** (the check) anyone could read the secret.

`├─ x-api-key: guess ─────────► 403 wrong API key              (handler not called)`
- **What:** the `else` branch returns the `403` rejection.
- **Why:** a key came, so the client is known to be trying, but this key opens nothing.
- **How:** the same as above: the rejection is the response.
- **Remove it and…** (the comparison) any key at all would get in.

`└─ x-api-key: letmein ───────► secret(ApiKey("letmein")) runs, 200`
- **What:** `Ok(ApiKey(…))`: Axum calls the handler with the filled-in `ApiKey`.
- **Why:** this is the only way into `secret`: no `ApiKey` value, no call.
- **How:** the handler's answer is the response, `200 OK` as usual.
- **Remove it and…** (the `Ok`) nobody could get in, not even with the right key.

### Step 5: the handlers

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:handlers}}
```

📁 Full code: code/topics/custom-extractors · ▶ Run it: `cd code && cargo run -p custom-extractors`

### Line by line

`async fn public() -> &'static str {` · `"anyone can read this"` · `}`
- **What:** a handler with no arguments, answering `anyone can read this`.
- **Why:** to show that only routes that **ask** for a key are protected. This one doesn't ask.
- **How:** the same kind of handler as in [Hello, Axum](hello-axum.md).
- **Remove it and…** `get(public)` in `app()` fails with ``cannot find value `public` in this
  scope``.

`async fn secret(ApiKey(key): ApiKey) -> String {`
- **What:** a handler that takes one argument, an `ApiKey`, and opens it with the pattern
  `ApiKey(key)`, so `key` is the `String` inside.
- **Why:** **this argument is the protection.** Axum can't call `secret` without an `ApiKey`, and
  the only way to get one is through `from_request_parts`, so `secret` runs only after the check
  passed. There's no `if` in the handler, and nothing to forget.
- **How:** exactly like `Path(id): Path<u32>`. To protect another handler, give it an
  `ApiKey` argument too. If the handler doesn't need the key's value, write `_key` instead of `key`,
  and the check still runs.
- **Remove it and…** (the argument) `/secret` answers everyone, with no key.

`format!("welcome, holder of key {key:?}")` · `}`
- **What:** the answer: `welcome, holder of key "letmein"`.
- **Why:** to prove the handler received the key the extractor checked.
- **How:** `{key:?}` prints the text with quotes around it, the `Debug` way.
- **Remove it and…** the function has no return value, and fails with ``mismatched types``.

### Step 6: the Router

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:app}}
```

📁 Full code: code/topics/custom-extractors · ▶ Run it: `cd code && cargo run -p custom-extractors`

### Line by line

`fn app() -> Router {` · `Router::new()`
- **What:** builds the app's Router.
- **Why:** as in every lesson, a separate function, so `main` and the test use the same Router.
- **How:** an ordinary function, not `async`: building a Router doesn't wait for anything.
- **Remove it and…** there's nothing to serve.

`.route("/public", get(public))`
- **What:** `GET /public` runs `public`.
- **Why:** the open room.
- **How:** nothing special; it has no `ApiKey` argument.
- **Remove it and…** `/public` answers `404`.

`.route("/secret", get(secret))` · `}`
- **What:** `GET /secret` runs `secret`.
- **Why:** the private room.
- **How:** notice what's **not** here: no `.layer`, no guard. The Router doesn't know `/secret` is
  protected; the handler's argument says so. That's the difference from the middleware in
  [Middleware and Tower layers](middleware-and-tower-layers.md#a-middleware-that-refuses-requests-without-a-key),
  which was added to the Router.
- **Remove it and…** `/secret` answers `404`.

### Step 7: `main`

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:main}}
```

📁 Full code: code/topics/custom-extractors · ▶ Run it: `cd code && cargo run -p custom-extractors`

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
- **How:** word for word the `main` of every Axum lesson. The extractor changed one handler, and
  nothing here.
- **Remove it and…** there's no port for curl to reach.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p custom-extractors
```

In the second terminal, visit the open room, then knock on the private one three times: with no
key, a wrong key, and the right key. `-H 'name: value'` makes curl send a header:

```bash
{{#include ../../code/topics/custom-extractors/http/01-keys.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/01-keys.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.

- **`/public`: `200`, `anyone can read this`.** No argument, no check.
- **`/secret`, no header: `401 Unauthorized`, `missing x-api-key header`.** The `.ok_or(…)?` line
  answered, and `secret` never ran.
- **`/secret`, `x-api-key: guess`: `403 Forbidden`, `wrong API key`.** The `else` branch answered.
- **`/secret`, `x-api-key: letmein`: `200 OK`, `welcome, holder of key "letmein"`.** The
  extractor said `Ok`, and the handler ran with the key.

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If you see `200`, `401`, `403` and `200`, in that order, your extractor checks the key and
  your handler is protected.
- ❌ If the build fails with `future cannot be sent between threads safely`, the `where S: Send + Sync`
  lines are missing (*Common mistakes*).
- ❌ If `/secret` answers `200` with no key, check that `secret` really takes an `ApiKey` argument.

## You might be wondering…

**"`FromRequestParts` or `FromRequest`: what's the difference?"**
`FromRequestParts` gets the request's **parts**, its head: the method, the address (path and query
string) and the headers. Any number of these extractors can read them, in any order. `FromRequest`
gets the **whole** request, body included. A body can be read only once, so a handler has at most
one `FromRequest` extractor, and it must be the **last** argument, as you saw with `Json` in
[JSON with serde](json-and-serde.md#common-mistakes). Rule of thumb: if you only need headers or the
address, use `FromRequestParts`. [Input validation](input-validation.md), the next lesson, writes a `FromRequest`
extractor, because it reads the body.

**"Why 401 for a missing key, but 403 for a wrong one?"**
It's the difference from
[How a web backend works](../part-0-start/how-a-web-backend-works.md#you-might-be-wondering):
`401` means "who are you?", and sending a key can fix it. `403` means "I know what you sent, and
the answer is no", and sending the same thing again won't help. A client program can act on the
difference: on `401`, ask the user for a key; on `403`, tell them that this key isn't allowed in.

**"Extractor or middleware: which one, when?"**
Both can refuse a request before the handler runs. A **middleware** (the `require_key` of
[Middleware and Tower layers](middleware-and-tower-layers.md#a-middleware-that-refuses-requests-without-a-key))
is put on the **Router**, and guards everything under it, even paths that don't exist. It's
the right tool when a whole area must be closed, such as everything under `/api/admin`, and the
handlers don't need anything from the check. An **extractor** is put on the **handler**: you can
see in its signature that it's protected, and it **hands the handler a value**, here the key, or,
in a real app, the user it belongs to. When a handler needs to know who's calling, use an
extractor. The two can work together, too: a nested Router with a layer for the area, and
extractors for the details.

**"Does the header name's case matter? And the key's?"**
Run the server again, and send the header with capitals, then the key with capitals
(`code/topics/custom-extractors/http/02-header-case.sh`):

```bash
{{#include ../../code/topics/custom-extractors/http/02-header-case.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/02-header-case.out}}
```

Header **names** never care about case, so `X-API-Key` finds the same header as `x-api-key`. The
**value** is your data, and `==` compares it exactly: `LetMeIn` is not `letmein`, so it's `403`.

**"Should the key be written in the code?"**
No. Anyone who can read the code can read the key, and changing it means building the program
again. A real app reads its secrets when it starts, from settings outside the code, usually
[environment variables](../glossary.md#environment-variable). *Config and .env*, in Part A4, shows
how. Until then, *More examples* keeps the allowed keys in the app's state, which is where values
read at startup end up.

**"Is this real authentication?"**
It's the first step. An API key says "this program may call me", and it's common for one program
calling another. Logging in **people** needs more: passwords stored safely, and a token the server
can check on every request without asking the database each time. The capstone does that in
*ShopRS 6: Log in with JWT*. The server checks a signed token on every request there, and an
extractor like this one is the natural place for that check.

## Coming from another language?

Every framework lets you run a check before a handler. What differs is how the handler gets the
result, and whether the compiler makes sure the check ran.

**Express (Node.js).** A middleware function, put on one route:

```js
function requireKey(req, res, next) {
  const key = req.get("x-api-key");
  if (!key) return res.status(401).send("missing x-api-key header");
  if (key !== "letmein") return res.status(403).send("wrong API key");
  req.apiKey = key;
  next();
}

app.get("/secret", requireKey, (req, res) => {
  res.send(`welcome, holder of key ${req.apiKey}`);
});
```

`requireKey` is `from_request_parts`, and `req.apiKey = key` is `Ok(ApiKey(…))`. The difference: in
Express, the handler **trusts** that `requireKey` ran. If you forget it on a route, `req.apiKey` is
`undefined`, and nothing warns you. In Axum, a handler that takes `ApiKey` can't run without the
check.

**FastAPI and Flask (Python).** FastAPI's *dependencies* are the closest thing to extractors:

```python
from typing import Annotated
from fastapi import Depends, FastAPI, Header, HTTPException

app = FastAPI()

def api_key(x_api_key: Annotated[str | None, Header()] = None) -> str:
    if x_api_key is None:
        raise HTTPException(status_code=401, detail="missing x-api-key header")
    if x_api_key != "letmein":
        raise HTTPException(status_code=403, detail="wrong API key")
    return x_api_key

@app.get("/secret")
def secret(key: Annotated[str, Depends(api_key)]):
    return f"welcome, holder of key {key}"
```

`Depends(api_key)` in the handler's parameters is `ApiKey` in its arguments, and raising
`HTTPException` is returning a rejection. (FastAPI turns the parameter name `x_api_key` into the
header name `x-api-key` by itself.) Flask has no dependencies; the usual way is a decorator that
reads `request.headers.get("x-api-key")` and calls `abort(401)` before the view runs.

**Spring Boot (Java).** The exact match is a `HandlerMethodArgumentResolver`: a class that tells
Spring how to fill a parameter of a type you made:

```java
class ApiKeyResolver implements HandlerMethodArgumentResolver {
    public boolean supportsParameter(MethodParameter p) {
        return p.getParameterType() == ApiKey.class;
    }
    public Object resolveArgument(MethodParameter p, ModelAndViewContainer m,
                                  NativeWebRequest request, WebDataBinderFactory f) {
        String key = request.getHeader("x-api-key");
        if (key == null)
            throw new ResponseStatusException(HttpStatus.UNAUTHORIZED, "missing x-api-key header");
        if (!key.equals("letmein"))
            throw new ResponseStatusException(HttpStatus.FORBIDDEN, "wrong API key");
        return new ApiKey(key);
    }
}
```

After you register it in a `WebMvcConfigurer`, any controller method with an `ApiKey key` parameter
gets one. That's `FromRequestParts` exactly, with more ceremony. In practice, most Spring apps check
keys with Spring Security's filters, which work like Axum middleware.

**Go (`net/http` and Gin).** Gin uses a middleware, and passes the result through the context:

```go
func RequireKey(c *gin.Context) {
    key := c.GetHeader("x-api-key")
    if key == "" {
        c.String(http.StatusUnauthorized, "missing x-api-key header")
        c.Abort()
        return
    }
    if key != "letmein" {
        c.String(http.StatusForbidden, "wrong API key")
        c.Abort()
        return
    }
    c.Set("apiKey", key)
    c.Next()
}

r.GET("/secret", RequireKey, func(c *gin.Context) {
    c.String(http.StatusOK, "welcome, holder of key %s", c.GetString("apiKey"))
})
```

`c.Set("apiKey", key)` stores the key under a name, and `c.GetString("apiKey")` reads it back: if
the middleware didn't run, you get an empty string, not an error. With plain `net/http`, you'd
call a helper such as `apiKey(r)` at the top of each handler, which is the "check in every handler"
this lesson started from.

## Common mistakes

The compiler errors below were captured in scratch copies of this lesson's code, so their line
numbers are from those copies.

**Forgetting `S: Send + Sync`.**

```rust,noplayground,ignore
impl<S> FromRequestParts<S> for ApiKey {
```

```text
error: future cannot be sent between threads safely
  --> src/main.rs:13:67
   |
13 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future returned by `from_request_parts` is not `Send`
   |
note: captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
  --> src/main.rs:13:52
   |
13 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                    ^^^^^^ has type `&S` which is not `Send`, because `S` is not `Sync`
note: required by a bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
  --> /Users/you/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/axum-core-0.5.6/src/extract/mod.rs:62:64
   |
62 |     ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
   |                                                                ^^^^ required by this bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
help: consider restricting type parameter `S` with trait `Sync`
   |
10 | impl<S: std::marker::Sync> FromRequestParts<S> for ApiKey {
   |       +++++++++++++++++++
```

Read it from the bottom of the story up. Axum's trait (the `note:` shows its real line) promises
that the work `from_request_parts` does is `Send`: Tokio may move it to another thread while it
waits. That work holds `_state`, a `&S`, and a reference can only go to another thread if the
thing it points at is `Sync`. With no condition on `S`, `S` could be anything, so Rust refuses. The
`help:` suggests `S: Sync`, and that alone does compile. Axum's own extractors, and its
documentation, write `S: Send + Sync`, and every state a Router accepts is both anyway, so use
that. **Fix:**

```rust,noplayground
{{#include ../../code/topics/custom-extractors/src/main.rs:extractor}}
```

**Using `FromRequest` when you only need headers.**
`FromRequest` sounds like the general one, so you write the same check with it. Here it is in a
scratch project called `my-body`, with a handler that also reads a JSON body:

```rust,noplayground,ignore
impl<S> FromRequest<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let value = req
            .headers()
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
        if value == "letmein" {
            Ok(ApiKey(value.to_owned()))
        } else {
            Err((StatusCode::FORBIDDEN, "wrong API key"))
        }
    }
}

async fn create_book(ApiKey(key): ApiKey, Json(book): Json<NewBook>) -> String {
    format!("{key:?} added {:?}", book.title)
}
```

The extractor compiles. The route doesn't:

```text
error[E0277]: the trait bound `fn(ApiKey, Json<NewBook>) -> impl Future<Output = String> {create_book}: Handler<_, _>` is not satisfied
   --> src/main.rs:41:40
    |
 41 |     Router::new().route("/books", post(create_book))
    |                                   ---- ^^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(ApiKey, Json<NewBook>) -> impl Future<Output = String> {create_book}`
    |                                   |
    |                                   required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
…
```

The usual `Handler<_, _>` error: *that*, not *why*. Add `#[axum::debug_handler]` above
`async fn create_book` (it needs Axum's `macros` feature, as in
[Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes)), and the first error
names the argument:

```text
error[E0277]: the trait bound `ApiKey: FromRequestParts<()>` is not satisfied
  --> src/main.rs:37:35
   |
37 | async fn create_book(ApiKey(key): ApiKey, Json(book): Json<NewBook>) -> String {
   |                                   ^^^^^^ unsatisfied trait bound
   |
help: the trait `FromRequestParts<()>` is not implemented for `ApiKey`
  --> src/main.rs:9:1
   |
 9 | struct ApiKey(String);
   | ^^^^^^^^^^^^^
   = note: Function argument is not a valid axum extractor. 
           See `https://docs.rs/axum/0.8/axum/extract/index.html` for details
…
```

That's Axum's rule for arguments: every argument **except the last** must be a `FromRequestParts`
extractor, because only the last one may read the body. `ApiKey` is first, and it's only a
`FromRequest`, so it isn't allowed there. (The `()` is this app's state type: a Router with no
state has the state `()`, "nothing".) Swap the two arguments, and `Json` is the one out of place:

```text
error: `Json<_>` consumes the request body and thus must be the last argument to the handler function
  --> src/main.rs:37:34
   |
37 | async fn create_book(Json(book): Json<NewBook>, ApiKey(key): ApiKey) -> String {
   |                                  ^^^^
```

Either way, there are two arguments that each claim the body, and a body can be read only once.
`ApiKey` never even looks at the body, but by implementing `FromRequest`, it said it might. **Fix:**
implement `FromRequestParts`, as in Step 3. A `FromRequestParts` extractor can go anywhere in the
argument list, even last, so `ApiKey` and `Json` fit in the same handler, with `Json` last.

**Comparing secrets with `==`.**
This one compiles, runs and passes every test, so it's worth a short, honest note. `==` on text
stops at the **first** character that differs. A wrong key that starts with the right letters takes
a tiny bit longer to reject than one that doesn't, and an attacker who sends many requests and
measures the time very precisely could, in principle, guess a key one character at a time. This is
called a **timing attack**. Over a real network the differences are hard to measure, but code that
checks secrets shouldn't leak them at all. The fix is a **constant-time** comparison, which always
looks at every byte. The `constant_time_eq` crate has one:
`constant_time_eq(value.as_bytes(), b"letmein")`. This lesson keeps `==`, to stay small and use only
Axum; when you check real secrets, use a constant-time comparison.

## More examples

Each example is a complete program. Stop any running server with `Ctrl+C`, start the example with
the command shown, and send its requests from your second terminal.

### A rejection in the app's JSON error format

In [Error handling in Axum](error-handling-in-axum.md), every error became a JSON body such as
`{"error":"book 9 not found"}`, through an `AppError` that implements `IntoResponse`. An extractor's
rejection can be that same `AppError`, so a refused request looks like every other error. This
example uses serde_json and thiserror, so it lives in that lesson's project
(`cargo run -p error-handling-in-axum --example errors-key-extractor`):

```rust,noplayground
{{#include ../../code/topics/error-handling-in-axum/examples/errors-key-extractor.rs:imports}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-key-extractor.rs:error}}

{{#include ../../code/topics/error-handling-in-axum/examples/errors-key-extractor.rs:extractor}}
```

```bash
{{#include ../../code/topics/error-handling-in-axum/http/55-key-extractor.sh}}
```

```text
{{#include ../../code/topics/error-handling-in-axum/http/55-key-extractor.out}}
```

Two lines changed in the extractor: `type Rejection = AppError;`, and the two errors are
`AppError::MissingKey` and `AppError::WrongKey` instead of tuples. The rest, the status codes and
the JSON, is `AppError`'s job, in its `IntoResponse`. Now the client gets
`content-type: application/json` for every error, from the extractor or from a handler.

### An extractor that never fails: the `user-agent`

Not every extractor checks something. This one reads the `user-agent` header, the name a client
gives itself, and uses `"unknown"` when there's none
(`cargo run -p custom-extractors --example extract-user-agent`):

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-user-agent.rs:imports}}

{{#include ../../code/topics/custom-extractors/examples/extract-user-agent.rs:extractor}}

{{#include ../../code/topics/custom-extractors/examples/extract-user-agent.rs:handler}}
```

```bash
{{#include ../../code/topics/custom-extractors/http/51-user-agent.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/51-user-agent.out}}
```

curl normally sends its own name and version, which differs from computer to computer, so the first
request sets the header itself, and the second, `-H 'user-agent:'` with nothing after the colon,
tells curl to send no `user-agent` at all. `.unwrap_or("unknown")` gives the fallback, and the
function always returns `Ok`. Its `Rejection` is `Infallible`, from the standard library: a type
with **no** possible values, which tells Axum, and every reader, "this can't fail".

### `Option<ApiKey>`: guests welcome, wrong keys not

Some pages show more to members, but still answer guests. For that, the handler takes an
`Option<ApiKey>`: `None` when no key was sent. In Axum 0.8, you choose what `Option<YourType>`
means by implementing a second trait, `OptionalFromRequestParts`
(`cargo run -p custom-extractors --example extract-optional-key`):

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-optional-key.rs:imports}}

{{#include ../../code/topics/custom-extractors/examples/extract-optional-key.rs:optional}}

{{#include ../../code/topics/custom-extractors/examples/extract-optional-key.rs:handler}}
```

The `ApiKey` and its `FromRequestParts` are the same as in Step 3.

```bash
{{#include ../../code/topics/custom-extractors/http/52-optional-key.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/52-optional-key.out}}
```

No header: `Ok(None)`, and the guest gets an answer. The right key: the normal check runs and gives
`Some(ApiKey(…))`. A **wrong** key is still `403`: we decided that a wrong key is a mistake worth
reporting, not a guest. That decision is the whole point of the trait; another app could treat a
wrong key as `None`. Two details:

- **`<ApiKey as FromRequestParts<S>>::from_request_parts(parts, state)`** calls the Step 3 check.
  `ApiKey` now has **two** functions called `from_request_parts`, one from each trait, so plain
  `ApiKey::from_request_parts` fails with ``error[E0034]: multiple applicable items in scope``. The
  `<Type as Trait>::` form says which one.
- **Without this `impl`**, a handler taking `Option<ApiKey>` is refused with the `Handler<_, _>`
  error. (In Axum 0.7, any extractor worked inside `Option`, and every failure quietly became
  `None`. Axum 0.8 makes you say what `None` means.)

If you'd rather see the rejection inside the handler, `Result<ApiKey, (StatusCode, &'static str)>`
also works as an argument, with no extra `impl`: Axum hands you `Ok(key)` or the rejection as `Err`,
and the handler decides what to do.

### The allowed keys in the app's state

Real apps have more than one key, and don't write them into the extractor. Here the keys live in
the app's state, and the extractor reads them from there
(`cargo run -p custom-extractors --example extract-keys-in-state`):

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-keys-in-state.rs:imports}}

{{#include ../../code/topics/custom-extractors/examples/extract-keys-in-state.rs:state}}

{{#include ../../code/topics/custom-extractors/examples/extract-keys-in-state.rs:extractor}}

{{#include ../../code/topics/custom-extractors/examples/extract-keys-in-state.rs:app}}
```

```bash
{{#include ../../code/topics/custom-extractors/http/53-keys-in-state.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/53-keys-in-state.out}}
```

Both listed keys get in, and the handler's answer shows which one was used. The change is in the
`impl` line: `FromRequestParts<AppState>` instead of `<S> FromRequestParts<S>`. This extractor
works only in a Router whose state is `AppState`, and in exchange, `state` is a real `&AppState`,
so `state.keys` can be read. There's no `<S>` and no `where` any more: `AppState` is a concrete
type, and Rust can see for itself that it's `Send` and `Sync`.
`state.keys.iter().any(|key| key == value)` asks "is any of the listed keys equal to the one sent?". The keys are in an `Arc`, as in
[Shared state](shared-state.md), so every copy of the state that Axum makes shares one list. In
*Config and .env*, in Part A4, that list will come from the environment, not from the code.

## Your turn

Each solution is a complete program in `code/topics/custom-extractors/examples/`, with a test
inside.

### 🟢 Guided

Change the key from `letmein` to `open-sesame`. Which line of Step 3 do you change? Then check that
the old key now gets `403`, and the new one `200`.

<details><summary>Solution</summary>

`cargo run -p custom-extractors --example extract-new-key`:

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-new-key.rs:extractor}}
```

```bash
{{#include ../../code/topics/custom-extractors/http/90-new-key.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/90-new-key.out}}
```

One line: `if value == "open-sesame"`. The handlers and the Router didn't change at all. That's the
bouncer's whole point: the rules live in one place.

</details>

### 🟡 Tweak

Some clients can't set headers easily, such as a link someone clicks. Accept the key from the
`x-api-key` header **or** from a `?key=` query parameter: `/secret?key=letmein` gets in too. If
neither is there, answer `401` with `missing API key`. Hint: `parts.uri` is the request's address,
and `Query::<HashMap<String, String>>::try_from_uri(&parts.uri)` reads its query string into a map.

<details><summary>Solution</summary>

`cargo run -p custom-extractors --example extract-key-or-query`:

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-key-or-query.rs:imports}}

{{#include ../../code/topics/custom-extractors/examples/extract-key-or-query.rs:extractor}}
```

```bash
{{#include ../../code/topics/custom-extractors/http/91-key-or-query.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/91-key-or-query.out}}
```

Both places now end as an `Option<String>`. `from_header` is the Step 3 chain, with
`.map(|value| value.to_owned())` to make the text owned. `from_query` asks `Query` to read the query
string into a `HashMap`; `.ok()` turns a failure into `None`, and `params.remove("key")` takes the
`key` out, if it's there. `from_header.or(from_query)` keeps the header if there is one, and the
query value otherwise. A word of warning: a key in the address ends up in browser history and in
server logs, so headers are the safer place. Offer `?key=` only when you must.

</details>

### 🔴 From scratch

Write a second extractor, `AdminKey`, for a new route `GET /admin`. It needs a valid API key (the
same `401`/`403` rules as `ApiKey`) **and** the header `x-role: admin`. With a valid key but no
`x-role: admin`, answer `403` with `admins only`. Keep `/secret` working with `ApiKey`. Hint: an
extractor may call another extractor.

<details><summary>Solution</summary>

`cargo run -p custom-extractors --example extract-admin-key`. The `ApiKey` extractor is the same as
in Step 3; here is the new one, and the handlers:

```rust,noplayground
{{#include ../../code/topics/custom-extractors/examples/extract-admin-key.rs:admin}}

{{#include ../../code/topics/custom-extractors/examples/extract-admin-key.rs:handlers}}
```

```bash
{{#include ../../code/topics/custom-extractors/http/92-admin-key.sh}}
```

```text
{{#include ../../code/topics/custom-extractors/http/92-admin-key.out}}
```

`ApiKey::from_request_parts(parts, state).await?` runs the whole key check first. If it fails, its
`401` or `403` comes out through `?` unchanged, which is why the third request, with a role but no
key, gets `missing x-api-key header`. The two rejection types match, `(StatusCode, &'static str)`,
so `?` can pass one on as the other. Only then does `AdminKey` look at `x-role`:
`role == Some("admin")` compares the `Option` in one go. `/secret` still only needs the key.

One honest note: in a real app, the role must come from the **server's** records (the key's entry
in the database, say), never from a header the client chooses: anyone can send `x-role: admin`. The
exercise is about combining checks; *ShopRS 6: Log in with JWT* gets the role from a token the
server signed.

</details>

## Quick check

<div class="quiz" data-topic="custom-extractors"></div>

## Remember this

- An extractor is a type with `impl<S> FromRequestParts<S> for YourType where S: Send + Sync`.
  Axum calls its `from_request_parts` before the handler.
- `type Rejection` is what the client gets when the check fails. Return `Err(…)` and the handler
  never runs: `401` for "who are you?", `403` for "not you".
- To protect a route, add the extractor as a handler argument. No layer, no `if` in the handler.
- `FromRequestParts` reads the head (headers, address, method) and can go in any position;
  `FromRequest` reads the body and must be last. Use `FromRequestParts` when you only need headers.
- Keys belong in settings, not in code; compare secrets in constant time.

## Go deeper

- [Rust for Humans: Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
- [Rust for Humans: Generics](https://open-source-bd.github.io/rustbook-for-human/abstractions/generics.html)
- [axum::extract::FromRequestParts](https://docs.rs/axum/0.8.9/axum/extract/trait.FromRequestParts.html) — Official reference: the trait every head-only extractor implements, with its Rejection type.

<!-- next:start -->

**Next:**

- [Input validation](../a2-axum/input-validation.md)

<!-- next:end -->
