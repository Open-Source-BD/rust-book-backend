# Input validation

> **Intermediate** · Part A2 · Axum

## By the end of this lesson

- You can declare rules on input fields (length, email, range).
- You can build a `ValidatedJson` extractor that checks them before your handler runs.
- You can return every problem at once as a 422 JSON response.

## What & why

Parse, then validate: the validator crate plus a `ValidatedJson<T>` extractor gives clear 422 errors listing every bad field.

Picture a paper sign-up form handed in at a counter. The clerk looks at it twice. The first look
asks "is this our form, with every box filled in?" The second look reads what's **in** the boxes:
a username of two letters, an email address that says `nope`, an age of 9. The form is complete,
so it passed the first look, but it's filled in wrongly. A good clerk circles **every** wrong box
in one go and hands the form back. A bad clerk points at the first mistake, waits for you to fix
it, then points at the next one.

Your API has the same two looks. In [JSON and serde](json-and-serde.md), `Json<NewBook>` did the
first one: it refuses a body that isn't JSON (`400`) or doesn't fit the struct (`422`). That is
**parsing**: it checks the **shape**. But `{"username":"al","email":"nope","age":9}` has a
perfect shape for a sign-up, and `Json` lets it straight through. Checking that the values make
sense is the second look, called **[validation](../glossary.md#validation)**. You could write it as
`if` statements at the top of every handler. This lesson writes it once, as rules on the struct,
and checks them in an extractor, before the handler runs.

Three new ideas:

1. **Rules on fields.** The validator [crate](../glossary.md#crate) lets you write a rule above
   each field, such as `#[validate(length(min = 3, max = 20))]`, and `#[derive(Validate)]` turns
   those rules into a `.validate()` method.
2. **`ValidatedJson<T>`: parse, then validate.** An [extractor](../glossary.md#extractor) of
   your own, like the `ApiKey` of [Custom extractors](custom-extractors.md), but it reads the
   **body**. It lets `Json` parse first, then calls `.validate()`. A handler that takes a
   `ValidatedJson<SignUp>` only runs for a sign-up that passed every rule.
3. **Every problem, in one `422`.** When rules fail, the client gets `422 Unprocessable Entity` and a
   JSON body naming **every** bad field, with the rule each one broke: the circled boxes. It's the
   answer [How a web backend works](../part-0-start/how-a-web-backend-works.md#bad-input-422-with-an-error-body)
   sketched by hand, made real.

## The idea, slowly

### Step 1: the crate's `Cargo.toml`

The code lives in `code/topics/input-validation`. Two lines are new: `serde_json` and `validator`.

```toml
{{#include ../../code/topics/input-validation/Cargo.toml}}
```

Their versions and features are written once, in the workspace's `code/Cargo.toml`:

```toml
{{#include ../../code/Cargo.toml:serde_json}}
{{#include ../../code/Cargo.toml:validator}}
```

### Line by line

`[package]` · `name = "input-validation"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`: `cargo run -p input-validation`.
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
- **Why:** Axum gives `Json`, `FromRequest` and `StatusCode`; Tokio runs the server; serde gives
  `#[derive(Deserialize)]`.
- **How:** the versions are written once, in `code/Cargo.toml`.
- **Remove it and…** (`serde`) ``error[E0432]: unresolved import `serde` ``.

`serde_json.workspace = true` · `serde_json = "1.0.151"`
- **What:** the crate that reads and writes JSON text, version 1.0.151.
- **Why:** Step 4 calls one of its functions by name, `serde_json::to_value`, to put the error list
  in a fixed order.
- **How:** in JSON and serde, Axum's `Json` used serde_json for you, and you didn't need this line.
  Axum brings serde_json in for **itself**; to write `serde_json::…` in **your** code, your project
  must list it too.
- **Remove it and…** ``error[E0433]: cannot find module or crate `serde_json` in this scope``.

`validator.workspace = true` · `validator = { version = "0.21.0", features = ["derive"] }`
- **What:** the validator crate, version 0.21.0, with its `derive` feature switched on.
- **Why:** validator knows how to check a length, an email address, a range of numbers, a web
  address and more. The `derive` feature adds `#[derive(Validate)]`, which writes the checking code
  for you from the rules above your fields.
- **How:** like serde, validator keeps its `#[derive(…)]` shortcut behind a feature, so projects
  that don't use it don't have to build it.
- **Remove it and…** (`features = ["derive"]`) the build fails with ``cannot find derive macro
  `Validate` in this scope``. *Common mistakes* shows the whole error.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** two crates for the tests at the bottom of `src/main.rs`.
- **Why:** the tests send pretend requests to the Router and read the answers, without opening a
  port.
- **How:** Cargo builds `[dev-dependencies]` only for `cargo test`.
- **Remove it and…** `cargo run` still works, but `cargo test` fails to compile.

In your own project, the same lines come from
`cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, then
`cargo add serde --features derive`, `cargo add serde_json` and
`cargo add validator --features derive`. The test crates come from
`cargo add --dev tower http-body-util --features tower/util`.

### Step 2: the `use` lines

```rust,noplayground
{{#include ../../code/topics/input-validation/src/main.rs:imports}}
```

📁 Full code: code/topics/input-validation · ▶ Run it: `cd code && cargo run -p input-validation`

### Line by line

`use axum::{` · `Json, Router,`
- **What:** opens the list of names this file takes from Axum: `Json` and `Router`.
- **Why:** `Json` does the parsing, and turns the error list into a JSON body; `Router` holds the
  route.
- **How:** the braces let one `use` line bring in several names.
- **Remove it and…** (`Json`) ``cannot find type `Json` in this scope``.

`extract::{FromRequest, Request, rejection::JsonRejection},`
- **What:** three names from Axum's `extract` module. `FromRequest` is the
  [trait](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html) for
  extractors that read the **body**. `Request` is a whole request, head and body. `JsonRejection` is
  the type of `Json`'s "no", its [rejection](../glossary.md#rejection): the `400`, `415` or `422` it
  sends.
- **Why:** Step 4 implements `FromRequest`, receives a `Request`, and names `JsonRejection`.
- **How:** `rejection::JsonRejection` reaches into the `rejection` module inside `extract`, where
  every built-in extractor's rejection type lives.
- **Remove it and…** (`FromRequest`) ``error[E0405]: cannot find trait `FromRequest` in this scope``.

`http::StatusCode,`
- **What:** the type for status codes.
- **Why:** for `StatusCode::UNPROCESSABLE_ENTITY` (`422`) and `StatusCode::CREATED` (`201`).
- **How:** as in [Handlers and IntoResponse](handlers-and-into-response.md).
- **Remove it and…** ``cannot find type `StatusCode` in this scope``.

`response::{IntoResponse, Response},`
- **What:** `Response`, a finished response ready to send, and `IntoResponse`, the trait with
  `.into_response()`, which turns a value into one.
- **Why:** Step 4's extractor can refuse a request in two different ways, and turns both into a
  `Response`.
- **How:** every type a handler may return implements `IntoResponse`, as you saw in Handlers and
  IntoResponse.
- **Remove it and…** (`IntoResponse`) every `.into_response()` fails with ``no method named
  `into_response` found``.

`routing::post,` · `};`
- **What:** `post`, to attach a handler to `POST` requests; then the list closes.
- **Why:** a sign-up sends data, so it's a `POST`.
- **How:** as in [Routes and HTTP methods](routes-and-methods.md).
- **Remove it and…** ``cannot find function `post` in this scope``.

`use serde::{Deserialize, de::DeserializeOwned};`
- **What:** `Deserialize`, which you know from JSON and serde, and `DeserializeOwned`, from serde's
  `de` (deserialize) module.
- **Why:** `Deserialize` lets JSON become a `SignUp`. `DeserializeOwned` is how Step 4 says "any
  type that can be built from JSON".
- **How:** `DeserializeOwned` means "can be built from JSON **and** owns all its data", with no
  `&str` borrowed from the request's text. The body is thrown away once it's read, so the value
  has to stand on its own. A struct of `String`s and numbers, like `SignUp`, is always fine.
- **Remove it and…** (`de::DeserializeOwned`) ``cannot find trait `DeserializeOwned` in this
  scope``.

`use validator::Validate;`
- **What:** the name `Validate` from the validator crate.
- **Why:** it's two things with one name: the **trait** that gives `.validate()`, and, thanks to the
  `derive` feature, the **derive macro** that writes that method for you. This one line brings in
  both.
- **How:** the same trick as serde's `Deserialize`, which is also a trait and a derive macro.
- **Remove it and…** ``error[E0405]: cannot find trait `Validate` in this scope``, plus a ``cannot
  find derive macro `Validate` `` and one ``cannot find attribute `validate` `` per rule.

### Step 3: the rules

Here is the sign-up form, with its rules written above each field:

```rust,noplayground
{{#include ../../code/topics/input-validation/src/main.rs:input}}
```

📁 Full code: code/topics/input-validation · ▶ Run it: `cd code && cargo run -p input-validation`

### Line by line

`#[derive(Deserialize, Validate)]`
- **What:** asks for two pieces of code to be written for you: `Deserialize` (JSON becomes a
  `SignUp`) and `Validate` (a `.validate()` method that checks every rule below).
- **Why:** you write **what** the rules are; the derive writes the code that checks them, field by
  field.
- **How:** `#[derive(Validate)]` reads every `#[validate(…)]` line in the struct. Each rule becomes
  a check inside `.validate()`, which runs them **all** and gathers every failure, instead of
  stopping at the first.
- **Remove it and…** (`Validate`) the `#[validate(…)]` lines mean nothing to the compiler, and the
  build fails. *Common mistakes* shows the real error.

`struct SignUp {`
- **What:** the shape of a sign-up: three fields.
- **Why:** it's both the parsing target (what `Json` builds) and the thing being validated.
- **How:** an ordinary struct, as in JSON and serde.
- **Remove it and…** there's no sign-up to build.

`#[validate(length(min = 3, max = 20))]` · `username: String,`
- **What:** the username must be between 3 and 20 characters long, both ends included.
- **Why:** `al` is too short to tell people apart, and a 500-character name breaks every page that
  shows it.
- **How:** a rule is written **above** the field it checks. `length` counts characters, not bytes,
  so `clé` is 3 long even though `é` takes two bytes. You can give only `min`, only `max`, or both.
- **Remove it and…** (the rule) `al`, and an empty username, are accepted.

`#[validate(email)]` · `email: String,`
- **What:** the text must look like an email address.
- **Why:** `nope` can't receive an email.
- **How:** it checks the **form**: something, then `@`, then a valid domain. It doesn't check that
  the mailbox exists (*You might be wondering…* says more).
- **Remove it and…** `nope` is accepted.

`#[validate(range(min = 13))]` · `age: u32,` · `}`
- **What:** the age must be 13 or more.
- **Why:** many online services only accept people aged 13 or older.
- **How:** `range` works on numbers, with `min`, `max`, or both. It never sees a **negative** age:
  a `u32` can't hold one, so `Json` refuses `-5` at the parsing step, with its own `422`. The type
  does part of the checking, and the rule does the rest.
- **Remove it and…** a 9-year-old can sign up.

### Step 4: the `ValidatedJson` extractor

This is the clerk's two looks in code. It's the longest listing in the lesson, so the Line by line
goes one or two lines at a time. If `FromRequestParts` from
[Custom extractors](custom-extractors.md#step-3-the-extractor) is fresh in your mind, much of it
will look familiar.

```rust,noplayground
{{#include ../../code/topics/input-validation/src/main.rs:extractor}}
```

📁 Full code: code/topics/input-validation · ▶ Run it: `cd code && cargo run -p input-validation`

### Line by line

`struct ValidatedJson<T>(T);`
- **What:** a new type: a box holding one value of **any** type `T`.
- **Why:** `Json<T>` is a box that means "this was parsed". `ValidatedJson<T>` is a box that means
  "this was parsed **and** passed its rules". The handler asks for the box it needs.
- **How:** the `<T>` is
  [generics](https://open-source-bd.github.io/rustbook-for-human/abstractions/generics.html): the
  same box works for `SignUp`, or a `CreateBook`, or any type you validate. It's a tuple struct, like
  `ApiKey`, so you open it with the pattern `ValidatedJson(input)`.
- **Remove it and…** there's no extractor for the handler to ask for.

`impl<T, S> FromRequest<S> for ValidatedJson<T>`
- **What:** "here's how to build a `ValidatedJson<T>` from a request".
- **Why:** it's `FromRequest`, not `FromRequestParts` as in Custom extractors, because we need the
  **body**. `FromRequestParts` only gets the head (method, address, headers), and the JSON is in the
  body.
- **How:** two generic names this time. `T` is the type inside the box; `S` is the app's
  [state](../glossary.md#state), whatever it is, as in Custom extractors. Like `Json`, an extractor
  that reads the body must be the **last** argument of a handler, because a body can be read only
  once.
- **Remove it and…** (the whole `impl`) the handler is refused with the `Handler<_, _>` error you've
  met before: `ValidatedJson<SignUp>` isn't an extractor.

`where`
- **What:** the start of the conditions. This `impl` only applies when the three lines below are
  all true.
- **Why:** the code inside uses `Json::<T>` and `.validate()`, and Rust only lets you use what the
  conditions promise.
- **How:** read the whole `where` aloud: "for any `T` and `S`, as long as `T` can be built from JSON
  and has rules, `S` can cross threads, and `Json<T>` is a body extractor whose 'no' is a
  `JsonRejection`."
- **Remove it and…** (all three lines) the build fails: nothing promises that a `T` can be parsed
  or validated.

`T: DeserializeOwned + Validate,`
- **What:** `T` must be buildable from JSON **and** have a `.validate()` method.
- **Why:** step one parses a `T` from JSON; step two calls `.validate()` on it.
- **How:** `+` means "both". `SignUp` derives `Deserialize` (which gives `DeserializeOwned` for a
  struct like this) and `Validate`, so it qualifies.
- **Remove it and…** (`+ Validate`) ``error[E0599]: no method named `validate` found for type
  parameter `T` in the current scope``: nothing promised that `T` has rules. Remove only
  `DeserializeOwned`, and it still builds, because the third line already promises that `Json<T>`
  works, which needs it. Keeping it says plainly what `T` must be.

`S: Send + Sync,`
- **What:** the state type must be safe to move to, and share between, threads.
- **Why:** for the same reason as in Custom extractors: Tokio may move a waiting request to another
  thread, and `from_request` holds a `&S` while it waits.
- **How:** every state a Router accepts is both already, so this costs nothing.
- **Remove it and…** ``error: future cannot be sent between threads safely``, as in
  [Custom extractors](custom-extractors.md#common-mistakes).

`Json<T>: FromRequest<S, Rejection = JsonRejection>,`
- **What:** "`Json<T>` is a body extractor in an app with state `S`, and when it fails, it fails
  with a `JsonRejection`".
- **Why:** step one hands the request to `Json`, and the line after that converts `Json`'s "no".
  This line is the promise that both are possible.
- **How:** a condition can be about any type, not only `T` or `S`. Axum's own `validator` example
  (for Axum 0.8.9) writes the same three conditions, with `Form` in place of `Json`.
- **Remove it and…** it still builds, with a warning that `JsonRejection` is now unused: the first
  two conditions are enough for Rust to work this out. Either way of saying it works; this one
  names the rejection type the code relies on.

`{` · `type Rejection = Response;`
- **What:** when this extractor says no, it answers with a `Response`.
- **Why:** it can say no in **two** ways: `Json`'s own `400`/`415`/`422`, a `JsonRejection`, and our
  `422` with the error list, which is a different type. `type Rejection` allows one type, so we
  turn each into a finished `Response`, the type both can become.
- **How:** in Custom extractors the rejection was a `(StatusCode, &'static str)`. A `Response` is
  the most general choice: anything with `.into_response()` fits.
- **Remove it and…** ``error[E0046]: not all trait items implemented, missing: `Rejection` ``.

`async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {`
- **What:** the one function the trait asks for. Axum calls it before the handler, with the whole
  request.
- **Why:** this is where both looks happen.
- **How:** it receives `request` **by value**, not as `&mut Parts`: reading a body uses it up, so
  the extractor takes the request for itself. That's why there can only be one body extractor per
  handler. It returns `Ok(ValidatedJson(…))` or a `Response` to send instead.
- **Remove it and…** (`async`) ``error[E0728]: `await` is only allowed inside `async` functions
  and blocks``, plus ``is not a future``, as in Custom extractors.

`let Json(value) = Json::<T>::from_request(request, state)` · `.await`
- **What:** look one: let `Json` parse the body into a `T`, exactly as it does in any handler.
- **Why:** there's no point checking rules on text that isn't even a sign-up. `Json` already knows
  how to refuse a wrong `content-type`, broken JSON and a missing field, so we reuse it, all of it.
- **How:** an extractor may call another extractor. `Json::<T>` says which `Json`: the one for our
  `T`. The `::<T>` is called the *turbofish*, a way to fill in a generic type by hand. `let
  Json(value) = …` opens the box, so `value` is the `T` itself. `.await` waits for the body to
  arrive.
- **Remove it and…** there's no `value` to validate.

`.map_err(|rejection| rejection.into_response())?;`
- **What:** if parsing failed, turn `Json`'s rejection into a `Response`, and stop here.
- **Why:** our `Rejection` type is `Response`, so `Json`'s `JsonRejection` must be converted before
  it can leave the function. The client gets Axum's usual plain-text answer, unchanged.
- **How:** `.map_err` changes the error inside a `Result` and leaves `Ok` alone; the
  [closure](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html)
  `|rejection| rejection.into_response()` says how. Then
  [`?`](https://open-source-bd.github.io/rustbook-for-human/abstractions/the-question-mark-operator.html)
  returns the error at once, and the handler never runs.
- **Remove it and…** (the `.map_err(…)`, keeping `?`) ``error[E0277]: `?` couldn't convert the
  error to `Response<Body>` ``: `?` can't turn a `JsonRejection` into a `Response` by itself.

`value.validate().map_err(|errors| {`
- **What:** look two: run every rule on the parsed value.
- **Why:** this is the line that turns "a sign-up" into "a **correct** sign-up".
- **How:** `.validate()` returns `Ok(())` if every rule passed, or `Err(errors)`, where `errors` (a
  `ValidationErrors`) lists every failure, field by field. The `.map_err` closure, which spans the
  next three lines, decides what the client sees.
- **Remove it and…** nobody calls the rules, and every sign-up is accepted. (*Common mistakes*
  shows what that looks like.)

`let sorted = serde_json::to_value(errors).expect("errors always convert to JSON");`
- **What:** turns the error list into a JSON value whose keys are in alphabetical order.
- **Why:** validator keeps the errors in a `HashMap`, a table whose order is **shuffled** every time
  the program starts, on purpose (it makes certain attacks harder). Sent as it is, one run of the
  server answers `username`, `email`, `age`, and the next `age`, `username`, `email`. A
  `serde_json::Value` keeps its keys sorted, so the same request always gets the same answer, byte
  for byte. That's kinder to the programs and tests that read it, and it lets this book check its
  transcripts.
- **How:** `serde_json::to_value` turns anything serde can [serialize](../glossary.md#serialize)
  into a `serde_json::Value`, a JSON value held in memory. It returns a `Result`, because some
  Rust values can't be JSON: a JSON key must be text, so a map whose keys are pairs, such as
  `(1, 2)`, fails with `key must be a string`. An error list's keys are always text, so
  `.expect(…)` takes the value out, and would stop the program with that message if the impossible
  ever happened.
- **Remove it and…** (the whole line, writing `Json(errors)` below) it still works, but the order
  of the fields, and of the params inside them, changes from one run of the server to the next.
  Remove only `.expect(…)`, and the next line fails: a `Json<Result<…>>` isn't a response.

`(StatusCode::UNPROCESSABLE_ENTITY, Json(sorted)).into_response()` · `})?;`
- **What:** the answer for broken rules: status `422`, with the sorted list as a JSON body. Then
  `?` sends it if `.validate()` failed.
- **Why:** `422 Unprocessable Entity` means "I understood what you sent, but its content breaks the
  rules". `Json(sorted)` sets `content-type: application/json` for you.
- **How:** the same `(status, body)` pair as in Handlers and IntoResponse, made into a `Response`
  with `.into_response()`, because that's our `Rejection` type. `})` closes the closure and the
  `.map_err`, and `?` returns the error from the function, so the handler never runs.
- **Remove it and…** (the `?`) it **builds**, with only ``warning: unused `Result` that must be
  used``, and every request is accepted, rules or no rules. That warning is worth reading. Remove
  `.into_response()` instead, and it's ``error[E0277]: `?` couldn't convert the error to
  `Response<Body>` ``.

`Ok(ValidatedJson(value))` · `}` · `}`
- **What:** both looks passed: put the value in the box and hand it to Axum.
- **Why:** `Ok` means "extraction worked"; Axum calls the handler with this `ValidatedJson`.
- **How:** the two `}` close the function and the `impl`.
- **Remove it and…** (writing `Ok(value)`) ``error[E0308]: mismatched types``, with the compiler's
  help: ``try wrapping the expression in `ValidatedJson` ``.

### Step 5: what happens to one sign-up

Here are the two looks, for one request, as a picture. The first three checks are `Json`'s, exactly
as in [JSON and serde, Step 5](json-and-serde.md#step-5-what-happens-to-a-json-body); the fourth is
new:

```text
POST /sign-up  (a body, and a content-type header)
  │  1. labelled JSON?             no → 415, plain text  ┐
  │  2. well-formed JSON?          no → 400, plain text  │ parsing (Json)
  │  3. fits a SignUp?             no → 422, plain text  ┘
  ▼
SignUp { username: "al", email: "nope", age: 9 }
  │  4. every rule passes?         no → 422, JSON list of every problem   validation
  ▼
sign_up(ValidatedJson(SignUp { … }))  runs → 201 Created
```

### Line by line

`POST /sign-up  (a body, and a content-type header)`
- **What:** a sign-up arrives, and the Router picks `sign_up`.
- **Why:** its argument is a `ValidatedJson<SignUp>`, so Axum calls `ValidatedJson`'s
  `from_request` first.
- **How:** `from_request` hands the request straight to `Json::<SignUp>::from_request`.
- **Remove it and…** no request, nothing happens.

`1. labelled JSON?` · `2. well-formed JSON?` · `3. fits a SignUp?` · `parsing (Json)`
- **What:** `Json`'s three checks, in order: the `content-type` label (`415`), the JSON grammar
  (`400`), and the shape of the struct (`422`).
- **Why:** each catches a different kind of broken request, as in JSON and serde.
- **How:** any failure comes out of the `.map_err(|rejection| …)?` line as Axum's own plain-text
  answer. The rules are never looked at: there's no `SignUp` to look at.
- **Remove it and…** (these checks) there'd be no `SignUp` for the rules to check.

`SignUp { username: "al", email: "nope", age: 9 }`
- **What:** parsing worked: a real `SignUp`, with the values the client sent.
- **Why:** it has the right shape. Whether the values make sense is the next question.
- **How:** this is `value` in Step 4.
- **Remove it and…** nothing to validate.

`4. every rule passes?  no → 422, JSON list of every problem   validation`
- **What:** `.validate()` runs **all** the rules. If any failed, the client gets `422` and a JSON
  body listing each one.
- **Why:** this is the new check, the one `Json` can't do.
- **How:** notice that this `422` is a different answer from step 3's `422`: plain text for "wrong
  shape", JSON for "broken rules". *Run it* shows both.
- **Remove it and…** `al`, `nope` and `9` reach your handler.

`sign_up(ValidatedJson(SignUp { … }))  runs → 201 Created`
- **What:** every check passed, and the handler runs.
- **Why:** this is the only way into `sign_up`: no `ValidatedJson`, no call.
- **How:** the handler's answer, here `201 Created`, is the response.
- **Remove it and…** nobody could sign up.

### Step 6: the handler and the Router

```rust,noplayground
{{#include ../../code/topics/input-validation/src/main.rs:handler}}
```

📁 Full code: code/topics/input-validation · ▶ Run it: `cd code && cargo run -p input-validation`

### Line by line

`async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {`
- **What:** a handler that takes a `ValidatedJson<SignUp>`, and opens the box, so `input` is the
  `SignUp`.
- **Why:** **this argument is the guarantee.** `sign_up` can only run with a `SignUp` that parsed
  and passed every rule, so there's no `if` here, and nothing to forget.
- **How:** exactly like `Json(input): Json<NewBook>` in JSON and serde, with a different box. It
  returns a status and a text, as in Handlers and IntoResponse.
- **Remove it and…** (writing `Json<SignUp>` instead) it builds and runs, but nothing checks the
  rules. *Common mistakes* shows it.

`(` · `StatusCode::CREATED,` · `format!("welcome, {} <{}>", input.username, input.email),` · `)` · `}`
- **What:** answers `201 Created` with `welcome, ada <ada@example.com>`.
- **Why:** `201` means "a new thing was made", here a new account. The text proves the handler got
  the values.
- **How:** a real app would save the account here (Part A3 teaches databases). The two `{}` are
  filled in order.
- **Remove it and…** the function returns nothing, and fails with ``mismatched types``.

`fn app() -> Router {` · `Router::new().route("/sign-up", post(sign_up))` · `}`
- **What:** `POST /sign-up` runs `sign_up`.
- **Why:** as in every lesson, a separate function, so `main` and the tests use the same Router.
- **How:** notice there's nothing about validation here. The handler's argument says it all.
- **Remove it and…** `/sign-up` answers `404`.

### Step 7: `main`

```rust,noplayground
{{#include ../../code/topics/input-validation/src/main.rs:main}}
```

📁 Full code: code/topics/input-validation · ▶ Run it: `cd code && cargo run -p input-validation`

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
- **How:** word for word the `main` of every Axum lesson.
- **Remove it and…** there's no port for curl to reach.

### Run it

Use the two-terminal routine from [Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it).
In the first terminal:

```bash
cd code && cargo run -p input-validation
```

In the second terminal, sign up properly first:

```bash
{{#include ../../code/topics/input-validation/http/01-good.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/01-good.out}}
```

As before, your terminal also shows a `date:` line in each response, which the book hides.
`201 Created` and `welcome, ada <ada@example.com>`: both looks passed, and the handler ran.

Now the form from *What & why*: complete, but filled in wrongly. Three fields, three broken rules:

```bash
{{#include ../../code/topics/input-validation/http/02-bad-values.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/02-bad-values.out}}
```

Read it from the top:

- **`422 Unprocessable Entity`.** The rules said no; the handler never ran. (How a web backend works
  called `422` *Unprocessable Content*: that's its newer official name. Same code, same meaning;
  Axum prints the older one.)
- **`content-type: application/json`.** This answer came from our `Json(sorted)`, so it's JSON, not
  plain text.
- **The body is one JSON object, with one key per bad field:** `age`, `email` and `username`. All
  three problems, in one answer. They're in alphabetical order, not the struct's order, because of
  the `serde_json::to_value` line in Step 4.
- **Each field holds a list `[…]`.** A field can break more than one rule (*More examples* has a
  username with two), and each broken rule gets one entry in its field's list.
- **`"code":"length"`, `"email"`, `"range"`.** The name of the rule that failed. It's for programs:
  a web page can check `code` and show its own text, in its own language.
- **`"message":null`.** A sentence for humans, if you wrote one. We didn't, so it's `null`, JSON's
  "nothing". *More examples* adds messages.
- **`"params"`.** The rule's settings, plus what was sent: for `username`, `"max":20,"min":3`
  (the limits) and `"value":"al"` (the value that broke them); for `age`, `"min":13` and
  `"value":9`; for `email`, only `"value":"nope"`, since that rule has no settings.

Last, two requests that never reach the rules. The first leaves out `age`; the second is missing
its closing `}`:

```bash
{{#include ../../code/topics/input-validation/http/03-bad-json.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/03-bad-json.out}}
```

These are Axum's own answers, from the `.map_err(|rejection| …)?` line, word for word what
[JSON and serde](json-and-serde.md#run-it) showed:

- **No `age`: `422`, plain text, ``missing field `age` ``.** A `SignUp` must have an age, so `Json`
  couldn't build one. That's a **shape** problem, found while parsing.
- **No closing `}`: `400 Bad Request`.** It isn't JSON at all.

So there are **two different `422`s**, and the `content-type` tells them apart:

| Answer | Means | Found by |
|---|---|---|
| `422`, `text/plain`, `Failed to deserialize the JSON body…` | the JSON doesn't fit the struct: a field is missing, or has the wrong type | parsing, `Json` |
| `422`, `application/json`, `{"field":[{"code":…}]}` | the JSON fits, but the values break your rules | validation, `.validate()` |

When you're done, press **`Ctrl+C`** in the first terminal to stop the server.

- ✅ If you see `201`, then a `422` with `age`, `email` and `username` in one JSON body, then a plain
  `422` and a `400`, the two looks work, in the right order.
- ❌ If the bad values get `201`, check that the handler takes `ValidatedJson<SignUp>`, not
  `Json<SignUp>` (*Common mistakes*).
- ❌ If your fields come out in a different order on each run, the `serde_json::to_value` line is
  missing.

## You might be wondering…

**"Why not check in the handler with `if` statements?"**
You can, and for one handler it works. But the checks then live in the handler, mixed with the
real work, and every handler that takes a `SignUp` must repeat them. Forget them in one, and that
route accepts anything. With rules on the struct, the rules travel with the type: any handler that
asks for a `ValidatedJson<SignUp>` gets the same checks, and can't run without them. It's the
bouncer from Custom extractors again: one set of rules, at the door.

**"Why return every problem at once?"**
Because a person is usually on the other end. If the server reports one problem per request, they
fix the username, send, see the email problem, fix it, send, see the age problem… Three trips for
one form. `.validate()` runs every rule before answering, so one `422` circles every wrong box.

**"Can I have my own messages?"**
Yes: add `message = "…"` to a rule, such as
`#[validate(length(min = 3, max = 20, message = "must be 3 to 20 characters long"))]`. It fills in
the `"message"` that was `null`. *More examples* shows it on all three rules.

**"Is the database a second line of defence?"**
Yes, and a good app uses both. In
[Keys and relations](../a1-postgres/keys-and-relations.md#a-rule-you-write-yourself-check), Postgres
refused rows that broke a `CHECK` rule, and duplicates that broke `UNIQUE`, whichever program sent
them. Validation in Axum gives the **person** a friendly, complete answer before anything touches
the database. The database's [constraints](../glossary.md#constraint) protect the **data**, even
from a bug in your code, or a script that never goes through your API. And some questions only the
database can answer: "is this username already taken?" depends on the rows already stored, which
validator never sees.

**"Why is Axum's `422` plain text, but mine JSON? Can they match?"**
They come from different places: Axum wrote one, you wrote the other. A real API should answer in
one style. [Error handling in Axum](error-handling-in-axum.md#axums-rejections-as-json-too) shows how
to turn Axum's rejections into JSON. In `ValidatedJson`, the place to do it is the
`.map_err(|rejection| …)` line: build any `Response` you like from the rejection.

**"Does `email` check that the address really exists?"**
No. It only checks the form, and even `a@b` passes. The only way to know an address works is to
send it an email, which is why sites send you a link to click. Validation catches typos and
nonsense, not lies.

## Coming from another language?

Every framework has the same two looks; most libraries do both in one call, and differ in which
status code they use.

**Express (Node.js).** Express has no validation of its own; a popular choice is the zod library,
which describes the shape and the rules together:

```js
const { z } = require("zod");

const SignUp = z.object({
  username: z.string().min(3).max(20),
  email: z.string().email(),
  age: z.number().int().min(13),
});

app.post("/sign-up", (req, res) => {
  const result = SignUp.safeParse(req.body);
  if (!result.success) {
    return res.status(422).json(result.error.flatten().fieldErrors);
  }
  const { username, email } = result.data;
  res.status(201).send(`welcome, ${username} <${email}>`);
});
```

`safeParse` checks the shape and the rules in one go and, like `.validate()`, collects every
problem. The difference: you must remember to call it in each handler. Nothing stops a handler from
using `req.body` unchecked.

**FastAPI and Flask (Python).** FastAPI uses pydantic models, where the rules sit on the fields:

```python
from fastapi import FastAPI
from pydantic import BaseModel, EmailStr, Field

app = FastAPI()

class SignUp(BaseModel):
    username: str = Field(min_length=3, max_length=20)
    email: EmailStr
    age: int = Field(ge=13)

@app.post("/sign-up", status_code=201)
def sign_up(input: SignUp):
    return f"welcome, {input.username} <{input.email}>"
```

This is the closest match to `ValidatedJson<SignUp>`: the handler can't run with a bad `SignUp`,
and every problem comes back in one `422`. (`EmailStr` needs the extra `email-validator` package:
`pip install "pydantic[email]"`.) FastAPI uses the same JSON `422` for a missing field and for a
broken rule; Axum keeps them apart. Flask has nothing built in: you call a library such as
marshmallow or pydantic yourself, at the top of each view.

**Spring Boot (Java).** Bean Validation puts the rules on the fields, and `@Valid` checks them
before the method runs:

```java
record SignUp(
    @NotNull @Size(min = 3, max = 20) String username,
    @NotNull @Email String email,
    @Min(13) int age) {}

@PostMapping("/sign-up")
@ResponseStatus(HttpStatus.CREATED)
public String signUp(@Valid @RequestBody SignUp input) {
    return "welcome, " + input.username() + " <" + input.email() + ">";
}
```

It needs the `spring-boot-starter-validation` dependency. Two differences: a failed `@Valid` answers
`400 Bad Request` by default, not `422`; and `@Size` and `@Email` accept a missing value, which is
why `@NotNull` is there. That's the same trap as validator's rules on an `Option` (*Common
mistakes*).

**Go (`net/http` and Gin).** Gin reads `binding` tags, checked by the go-playground validator:

```go
type SignUp struct {
    Username string `json:"username" binding:"required,min=3,max=20"`
    Email    string `json:"email" binding:"required,email"`
    Age      int    `json:"age" binding:"required,gte=13"`
}

r.POST("/sign-up", func(c *gin.Context) {
    var input SignUp
    if err := c.ShouldBindJSON(&input); err != nil {
        c.JSON(http.StatusUnprocessableEntity, gin.H{"error": err.Error()})
        return
    }
    c.String(http.StatusCreated, "welcome, %s <%s>", input.Username, input.Email)
})
```

`ShouldBindJSON` parses and validates, and its error lists every failed field. As in Express, each
handler must call it. With plain `net/http`, you decode with `encoding/json` and then call the same
validator library yourself.

## Common mistakes

The compiler errors below were captured in a scratch copy of this lesson's code, called
`my-validation`, so their line numbers are from that copy.

**Forgetting `derive(Validate)`.**
You write the rules, but derive only `Deserialize`:

```rust,noplayground,ignore
#[derive(Deserialize)]
struct SignUp {
    #[validate(length(min = 3, max = 20))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(range(min = 13))]
    age: u32,
}
```

```text
error: cannot find attribute `validate` in this scope
  --> src/main.rs:17:7
   |
17 |     #[validate(range(min = 13))]
   |       ^^^^^^^^
   |
   = note: `validate` is an attribute that can be used by the derive macro `Validate`, you might be missing a `derive` attribute
…
error[E0277]: the trait bound `fn(ValidatedJson<SignUp>) -> ... {sign_up}: Handler<_, _>` is not satisfied
   --> src/main.rs:51:42
    |
 51 |     Router::new().route("/sign-up", post(sign_up))
    |                                     ---- ^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(ValidatedJson<SignUp>) -> impl Future<Output = ...> {sign_up}`
    |                                     |
    |                                     required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
…
```

The same `cannot find attribute` error appears once for each of the three `#[validate(…)]` lines.
`#[validate]` isn't part of Rust: it only means something inside a struct that derives `Validate`,
and the `note:` says exactly that: ``you might be missing a `derive` attribute``. The second error
follows from the first: `SignUp` has no `.validate()`, so it breaks the `T: Validate` condition of
Step 4, so `ValidatedJson<SignUp>` isn't an extractor, so `sign_up` isn't a handler. Add
`#[axum::debug_handler]` above `sign_up` (it needs Axum's `macros` feature, as in
[Handlers and IntoResponse](handlers-and-into-response.md#common-mistakes)), and that error points
at the argument and at our `where` clause instead:

```text
error[E0277]: the trait bound `ValidatedJson<SignUp>: FromRequest<(), _>` is not satisfied
  --> src/main.rs:44:40
   |
44 | async fn sign_up(ValidatedJson(input): ValidatedJson<SignUp>) -> (StatusCode, String) {
   |                                        ^^^^^^^^^^^^^ unsatisfied trait bound
   |
…
help: the trait `FromRequest<S>` is implemented for `ValidatedJson<T>`
  --> src/main.rs:23:1
   |
23 | / impl<T, S> FromRequest<S> for ValidatedJson<T>
24 | | where
25 | |     T: DeserializeOwned + Validate,
26 | |     S: Send + Sync,
27 | |     Json<T>: FromRequest<S, Rejection = JsonRejection>,
   | |_______________________________________________________^
…
```

"It's implemented, **where** these conditions hold", and `T: Validate` doesn't. **Fix:**
`#[derive(Deserialize, Validate)]`, as in Step 3.

**Forgetting `features = ["derive"]`.**
In your own project, `cargo add validator` without `--features derive` gives the line
`validator = "0.21.0"`. The code is right, but:

```text
error: cannot find derive macro `Validate` in this scope
  --> src/main.rs:11:23
   |
11 | #[derive(Deserialize, Validate)]
   |                       ^^^^^^^^
   |
note: `Validate` is imported here, but it is only a trait, without a derive macro
  --> src/main.rs:9:5
   |
 9 | use validator::Validate;
   |     ^^^^^^^^^^^^^^^^^^^

error: cannot find attribute `validate` in this scope
  --> src/main.rs:13:7
   |
13 |     #[validate(length(min = 3, max = 20))]
   |       ^^^^^^^^
…
```

The `note:` is the clue: ``it is only a trait, without a derive macro``. Without the feature,
validator gives you the `Validate` **trait**, but not the `#[derive(Validate)]` that writes it for
you. The `cannot find attribute` errors (one per rule) and the `Handler<_, _>` error follow from
that. **Fix:** `validator = { version = "0.21.0", features = ["derive"] }`, or
`cargo add validator --features derive`.

**Thinking a rule makes an `Option` field required.**
You want an email, so you add `#[validate(email)]`, but the field is an `Option<String>`:

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-option-mistake.rs:input}}

{{#include ../../code/topics/input-validation/examples/validate-option-mistake.rs:handler}}
```

`cargo run -p input-validation --example validate-option-mistake`:

```bash
{{#include ../../code/topics/input-validation/http/70-option-skipped.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/70-option-skipped.out}}
```

No email at all: `201`, `welcome, ada <no email>`. A rule on an `Option` checks the value **when
there is one**, and a missing field is `None`, so there's nothing to check. Send `"nope"`, and the
rule does its job. **Fix:** decide what you mean. If every sign-up needs an email, use `String`, not
`Option<String>`: then a missing email is refused by `Json` at the parsing step, as `age` was in
*Run it*. If the email is truly optional, but must be valid when given, this code is right. (*Your
turn* uses that on purpose. validator also has a `required` rule for `Option` fields, if you need
one.)

**Declaring rules, then taking plain `Json`.**
The rules are on the struct and `derive(Validate)` is there, but the handler says `Json<SignUp>`:

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-plain-json.rs:handler}}
```

`cargo run -p input-validation --example validate-plain-json`:

```bash
{{#include ../../code/topics/input-validation/http/71-plain-json.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/71-plain-json.out}}
```

`welcome, al <nope>`, with no error and no warning. `#[derive(Validate)]` only **writes** the
`.validate()` method; nothing calls it unless someone asks. `Json` parses and stops there.
**Fix:** take `ValidatedJson<SignUp>`, the extractor that calls `.validate()` for you.

## More examples

Each example is a complete program in `code/topics/input-validation/examples/`, with the same
`ValidatedJson` extractor as Step 4. Stop any running server with `Ctrl+C`, start the example with
the command shown, and send its requests from your second terminal.

### Your own messages

`message = "…"` gives each rule a sentence for humans
(`cargo run -p input-validation --example validate-messages`):

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-messages.rs:input}}
```

```bash
{{#include ../../code/topics/input-validation/http/51-messages.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/51-messages.out}}
```

Same request as *Run it*, same codes and params, and now every `"message"` holds your sentence
instead of `null`. For `email`, which has no settings, the message goes in brackets:
`email(message = "…")`. A web page can show `message` as it is, or use `code` to show its own text.

### A rule you write yourself

No built-in rule says "no spaces". Write it as an ordinary function, and point a rule at it with
`custom(function = "…")` (`cargo run -p input-validation --example validate-custom-fn`):

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-custom-fn.rs:imports}}

{{#include ../../code/topics/input-validation/examples/validate-custom-fn.rs:rule}}

{{#include ../../code/topics/input-validation/examples/validate-custom-fn.rs:input}}
```

```bash
{{#include ../../code/topics/input-validation/http/52-custom-fn.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/52-custom-fn.out}}
```

The function receives the field's value as a `&str` (a `&String` works too) and returns
`Result<(), ValidationError>`: `Ok(())` for "fine", or an `Err` with a code of your choosing,
`ValidationError::new("no_spaces")`. `.with_message(…)` adds the human sentence; `.into()` turns
the `&str` into the type it stores. validator adds the `"value"` param for you. One field can have
several rules, separated by commas: `length(…), custom(…)`. `ada lovelace` is long enough, so only
`no_spaces` fails. The third name is too long **and** has spaces, so `username`'s list holds two
entries, one per broken rule, in the order the rules are written.

### A struct inside a struct

A sign-up with an address, which has rules of its own. `#[validate(nested)]` says "validate this
field's own rules too" (`cargo run -p input-validation --example validate-nested`):

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-nested.rs:input}}
```

```bash
{{#include ../../code/topics/input-validation/http/53-nested.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/53-nested.out}}
```

`Address` derives `Validate` too, with its own rules: a `city` that isn't empty, and a `postcode`
of exactly 4 characters (`length(equal = 4)`). The errors keep the same shape as the data:
`"address"` holds an object with **its** bad fields, `city` and `postcode`, next to `username` at
the top. Without `#[validate(nested)]`, the address is parsed but its rules are never checked.

### "Type your password twice"

`must_match` compares two fields (`cargo run -p input-validation --example validate-must-match`):

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-must-match.rs:input}}
```

```bash
{{#include ../../code/topics/input-validation/http/54-must-match.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/54-must-match.out}}
```

The rule sits on `password_again`, and `other = "password"` names the field it must equal. Look at
`params`, though: `"other":"correct horse","value":"correct hose"`. The error repeats **both
passwords** back. That's harmless on your own machine, but a real app shouldn't send passwords
around, or let them land in logs: before answering, it would drop `params` from the list, and keep
`code` and `message`.

## Your turn

Each solution is a complete program in `code/topics/input-validation/examples/`, with a test inside.

### 🟢 Guided

The service changes its rules: you must now be **16** or older. Change one line of Step 3, then
check that 15 is refused and 16 accepted.

<details><summary>Solution</summary>

`cargo run -p input-validation --example validate-min-age`:

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-min-age.rs:input}}
```

```bash
{{#include ../../code/topics/input-validation/http/90-min-age.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/90-min-age.out}}
```

One number: `range(min = 16)`. `"min":16` in the answer shows the new rule, and 16 itself passes,
because `min` includes its own value. The extractor, the handler and the Router didn't change.

</details>

### 🟡 Tweak

Add an optional `website: Option<String>` to `SignUp`. A sign-up without one is fine; but if one is
sent, it must be a real web address. validator has a rule called `url`. Make the welcome message say
`(website: none)` when there isn't one.

<details><summary>Solution</summary>

`cargo run -p input-validation --example validate-website`:

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-website.rs:input}}

{{#include ../../code/topics/input-validation/examples/validate-website.rs:handler}}
```

```bash
{{#include ../../code/topics/input-validation/http/91-website.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/91-website.out}}
```

Here the `Option` behaviour from *Common mistakes* is exactly what you want: no `website`, no check;
a `website`, checked. `my site` isn't an address, so it's `422` with `"code":"url"`. The rule wants
a full address with its scheme: `https://example.com` passes, but `example.com` alone doesn't.
`input.website.as_deref()` turns the `Option<String>` into an `Option<&str>`, so
`.unwrap_or("none")` can give a `&str` fallback.

</details>

### 🔴 From scratch

Write `POST /books`, taking a `CreateBook` with:

- `title`: 1 to 200 characters;
- `pages`: 1 to 5000;
- `isbn`: exactly 13 digits, nothing else (`9780441013593` is fine; `978-0441013593` isn't).

Answer `201` with `added "Dune", 412 pages, ISBN 9780441013593`. Hint: no built-in rule says
"13 digits", so write one, as in *More examples*. Two methods help: `.len()` gives a text's length
in bytes, and `.bytes().all(|byte| byte.is_ascii_digit())` is true when every byte is `0` to `9`.

<details><summary>Solution</summary>

`cargo run -p input-validation --example validate-create-book`. The `ValidatedJson` extractor and
`main` are the same as in the lesson:

```rust,noplayground
{{#include ../../code/topics/input-validation/examples/validate-create-book.rs:rule}}

{{#include ../../code/topics/input-validation/examples/validate-create-book.rs:input}}

{{#include ../../code/topics/input-validation/examples/validate-create-book.rs:handler}}
```

```bash
{{#include ../../code/topics/input-validation/http/92-create-book.sh}}
```

```text
{{#include ../../code/topics/input-validation/http/92-create-book.out}}
```

`thirteen_digits` checks both things at once: 13 bytes long, and every byte a digit. Counting bytes
is safe here, because a digit is always one byte: if every byte is a digit, 13 bytes means 13
digits. `&&` stops early, so for a 12-character ISBN the digits aren't even looked at. The bad
request breaks all three rules, and gets all three back in one answer: `isbn` with your own code and
message, `pages` and `title` with the built-in ones. The handler's `{:?}` prints the title with
quotes around it.

</details>

## Quick check

<div class="quiz" data-topic="input-validation"></div>

## Remember this

- **Parsing checks the shape; validation checks the values.** `Json<T>` does the first. Rules on the
  struct do the second.
- Write rules above fields with `#[derive(Validate)]` and `#[validate(…)]`: `length`, `email`,
  `range`, `url`, `must_match`, `nested`, and `custom(function = "…")` for your own. The derive
  needs `features = ["derive"]`.
- A `ValidatedJson<T>` extractor implements `FromRequest` (it reads the body): `Json` first, then
  `.validate()`, then `422` with every problem as JSON. A handler that takes it can't see bad input.
- Two different `422`s: plain text from `Json` (wrong shape), JSON from your rules (wrong values).
- A rule on an `Option` checks only when a value is there. The database's constraints are the second
  line of defence.

## Go deeper

- [Rust for Humans: Generics](https://open-source-bd.github.io/rustbook-for-human/abstractions/generics.html)
- [Rust for Humans: Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
- [validator 0.21](https://docs.rs/validator/0.21.0/validator/) — Official reference: every built-in rule (length, email, range, url, must_match…), custom functions and nested structs.
- [axum::extract::FromRequest](https://docs.rs/axum/0.8.9/axum/extract/trait.FromRequest.html) — Official reference: the trait for extractors that read the request body.

<!-- next:start -->

**Next:**

- [Testing handlers](../a2-axum/testing-handlers.md)

<!-- next:end -->
