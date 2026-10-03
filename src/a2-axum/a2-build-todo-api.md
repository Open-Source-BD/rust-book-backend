# Build it: a Todo API

> **Intermediate** · Part A2 · Axum

## What you'll build

You've learned Part A2 one idea at a time: [routes](../glossary.md#route),
[handlers](../glossary.md#handler), [JSON](../glossary.md#json), [state](../glossary.md#state),
errors, [validation](../glossary.md#validation), [modules](../glossary.md#module),
[tests](../glossary.md#test). Now you'll put all of them together and build something real: a small
[API](../glossary.md#api) for a todo list, the "hello world" of backend projects. A program (a phone
app, a web page, `curl`) can ask it to **create** a todo, **list** them all, **show** one, **update**
one (change its title, or tick it off as done) and **delete** one. Those five jobs are the
[CRUD](../glossary.md#crud) you met in SQL, now spoken over HTTP.

By the end of this page:

- You built a complete REST API for todos from an empty folder.
- You combined routing, JSON, state, errors, validation and tests in one app.
- You tested it by hand and with automated tests.

This is a conversation with the finished API, sent for real with `curl`. Start the server with
`cd code && cargo run -p a2-build-todo-api`, then, in a second terminal:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/06-final.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/06-final.out}}
```

Top to bottom: the server says it's alive; two todos are **created** (`201 Created`, each with its
own `id`); the **list** shows both; the first one is **ticked off** (`"done":true`); the second is
**deleted** (`204 No Content`: done, nothing to send back); asking for it again gets a `404` with a
JSON message; and a todo with an empty title is **refused** with `422` and the reason. Every answer
on this page comes from the real server, and the book's automatic checks re-run each one on every
change. (Your terminal also shows a `date:` line in each response, which the book hides.)

This project is **fully guided**. The spec tells you what to build. Each step gives you a goal, a
hint you can open if you're stuck, the book's own code for that step, and a ✅ checkpoint with the
exact output you should see. A full reference solution, explained line by line, comes after the
steps. Try each step yourself first: the feeling of "I wrote that, and it works" is the whole point.

## What you need to know

Everything here comes from Part A2. If a step feels unfamiliar, the lesson in brackets is where it
was taught:

- [Hello, Axum](hello-axum.md): `cargo new`, `cargo add`, a `Router`, `main`, and the two-terminal
  routine for running a server and talking to it with `curl`.
- [Routes and HTTP methods](routes-and-methods.md): one path with several methods,
  `get(…).post(…)`, and why a missing method answers `405`.
- [Handlers and IntoResponse](handlers-and-into-response.md): returning a `(StatusCode, body)` pair,
  and `204 No Content`.
- [Path and Query extractors](path-and-query-extractors.md): `Path(id)` for `/todos/{id}`, and
  `Query` (in the stretch goals).
- [JSON with serde](json-and-serde.md): `#[derive(Serialize, Deserialize)]`, `Json` in and out, one
  struct for each direction, and `Option` fields.
- [Shared state](shared-state.md): `Arc<Mutex<…>>`, `State`, `.with_state`, and locking.
- [Error handling in Axum](error-handling-in-axum.md): an `AppError` enum with `thiserror`, its
  `IntoResponse`, `Result` from a handler, and `.ok_or(…)?`.
- [Middleware and Tower layers](middleware-and-tower-layers.md): not used in the steps, but the
  idea that a Router is a Tower service is what makes Step 6's tests work.
- [Nesting and modular routers](nesting-and-modular-routers.md): one file per area, `mod`, `pub`,
  `crate::`, and `.nest`.
- [Custom extractors](custom-extractors.md): how `FromRequest` lets you write your own extractor.
- [Input validation](input-validation.md): `#[derive(Validate)]`, length rules, and the
  `ValidatedJson` extractor, which you'll reuse word for word.
- [Testing handlers](testing-handlers.md): the `lib.rs`/`main.rs` split, `tests/`, and `oneshot`.

Three small things are new, and each is explained where it appears: `iter_mut`, `if let Some`, and
`retain` (all in Step 4).

## The spec

A **spec** (short for *specification*) is a description of what to build, written before anyone
builds it. Real projects start with one. This is yours.

### What a todo is

A todo has three fields: a number that names it (`id`), what to do (`title`), and whether it's
finished (`done`). As JSON, the format every answer uses, one todo looks like
`{"id":1,"title":"buy milk","done":false}`, and a list of todos is a JSON array of those, in the
order they were created.

### The endpoints

An [endpoint](../glossary.md#endpoint) is one method plus one path. Following the
[REST](../glossary.md#rest) plan from [How a web backend works](../part-0-start/how-a-web-backend-works.md),
the API has **two paths and five methods** for todos, plus a health check. Each row gives the
[status code](../glossary.md#status-code) a good request gets, and the ones a bad request can get:

| Method | Path | Request body | Success | Errors |
|---|---|---|---|---|
| `GET` | `/health` | none | `200`, the text `ok` | none |
| `GET` | `/api/todos` | none | `200`, every todo as a JSON array | none |
| `POST` | `/api/todos` | `{"title":"…"}` | `201`, the new todo | `422` if the title breaks rule 1 |
| `GET` | `/api/todos/{id}` | none | `200`, that todo | `404` if there's no todo with that `id` |
| `PATCH` | `/api/todos/{id}` | `{"title":"…"}`, `{"done":true}`, or both | `200`, the changed todo | `404` as above, `422` if the title breaks rule 1 |
| `DELETE` | `/api/todos/{id}` | none | `204`, empty body | `404` as above |

`{id}` stands for a number, such as `/api/todos/1`. Any other method on these paths answers `405`,
and any other path answers `404`; Axum does both for you. Broken JSON, or a body that isn't JSON at
all, gets Axum's own `400`, `415` or `422` from the `Json` extractor, as in
[JSON with serde](json-and-serde.md).

While you build (Steps 1 to 5), the todo paths are shorter, `/todos` and `/todos/{id}`. Step 6 moves
them under `/api`.

### The rules

1. **A title is 1 to 100 characters long.** An empty title, or one over 100 characters, is refused
   with `422` and validator's list of what went wrong, both when creating and when a `PATCH`
   changes the title.
2. **Ids start at 1, go up by one, and are never reused.** Not even after a delete: if todo 2 is
   deleted, no later todo is ever called 2.
3. **`PATCH` changes only the fields it's given.** `{"done":true}` ticks a todo off and leaves its
   title alone; `{"title":"…"}` renames it and leaves `done` alone.
4. **A new todo is never done.** The client sends only a title; the server sets `done` to `false`.
5. **"Not found" is JSON**: `{"error":"todo 7 not found"}`, with status `404`.

The todos live in the server's memory. Stop the server, and they're gone: Part A3, *SeaORM*, moves
them into Postgres.

### Acceptance checks

**Acceptance checks** are the tests that say "done". Yours are:

- ✅ Your finished server answers the conversation in *What you'll build* exactly.
- ✅ `cargo test` runs the two tests in `tests/api.rs`, and both pass.
- ✅ After a delete, the next new todo gets a **new** number (Step 4's checkpoint shows it).

## Build it, step by step

Each step adds one or two ideas to a program that runs. You'll grow one `src/main.rs` through
Steps 1 to 5, then split it into files in Step 6.

**Where to put your code.** Make your own project, outside the book's `code/` folder (your home
folder is fine), with every crate the project will need:

```bash
cargo new todo-api
cd todo-api
cargo add axum tokio serde serde_json thiserror validator --features tokio/macros,tokio/rt-multi-thread,tokio/net,serde/derive,validator/derive
cargo add --dev tower http-body-util --features tower/util
```

That's the same `cargo new` and `cargo add` as in [Hello, Axum](hello-axum.md), with more crates in
one go: the first `cargo add` adds six, each feature named after its crate (`serde/derive` is
`serde`'s `derive` feature), and `--dev` puts the two test-only crates under
`[dev-dependencies]`. The *Reference solution* explains every line they write into `Cargo.toml`.
`cargo add` picks each crate's newest version; when this book was written, those were the same
versions the book's own `code/Cargo.toml` uses (Axum 0.8.9, Tokio 1.53.1, and so on). To pin one
exactly, add `@` and the version, as in `cargo add axum@0.8.9`
([Hello, Axum](hello-axum.md) has the details).

**The book's version of each step** is a full program in
`code/projects/a2-build-todo-api/examples/`, named `step1.rs` to `step5.rs`. Each one has a test at
the bottom too. If you're stuck, run the book's step from the book's `code/` folder, compare, and
carry on from there.

**Every checkpoint** uses the two-terminal routine from
[Hello, Axum, Step 7](hello-axum.md#step-7-start-the-server-then-talk-to-it): start your server in
the first terminal with `cargo run` (inside `todo-api`), type the checkpoint's `curl` commands in the
second, and press **`Ctrl+C`** in the first when you're done. **Restart the server before each
checkpoint**, so it starts with an empty list, as the book's did.

### Step 1: a server that says `ok`

**Goal:** `GET /health` answers `200` with the text `ok`. Nothing else yet.

<details><summary>Hint</summary>

This is [Hello, Axum](hello-axum.md) again, with one route. The handler can be an `async fn` that
returns `&'static str`. Keep the usual `main`: bind `127.0.0.1:3000`, print a line, `axum::serve`.

</details>

<details><summary>The book's code for Step 1</summary>

`code/projects/a2-build-todo-api/examples/step1.rs`:

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/step1.rs:all}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example step1`

</details>

**✅ Checkpoint.** With your server running, in the second terminal:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/01-step1.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/01-step1.out}}
```

A **health check** like this is the first route of almost every real service: monitoring tools ask
"are you alive?" every few seconds, and a `200` means yes.

- ✅ If you see `200 OK` and `ok`, you're right.
- ❌ If `curl` says `Failed to connect to 127.0.0.1 port 3000`, the server isn't running: start it
  in the first terminal, and wait for its `listening` line.
- ❌ If the server stops at once with `port 3000 is busy`, another server (perhaps an earlier step)
  is still running. Press `Ctrl+C` in its terminal.

### Step 2: list and create

**Goal:** `GET /todos` lists every todo as JSON, and `POST /todos` with `{"title":"…"}` creates one,
answering `201` with the new todo. A new todo's `id` is "how many todos there are, plus one", and
`done` is `false`.

<details><summary>Hint 1: the types</summary>

Two structs, one for each direction, as in [JSON with serde](json-and-serde.md): a `Todo` the server
sends (`Serialize`, and `Clone`, so you can keep one copy and send another), and a `NewTodo` the
client sends (`Deserialize`, with only a `title`). The `id` is a `u64`, a whole number that's never
negative.

</details>

<details><summary>Hint 2: the state</summary>

This is [Shared state](shared-state.md) with todos instead of books: an `AppState` holding an
`Arc<Mutex<Vec<Todo>>>`, `#[derive(Clone, Default)]`, and `.with_state(AppState::default())` on the
Router. One path, two methods: `.route("/todos", get(list).post(create))`. In `create`, the body
[extractor](../glossary.md#extractor) `Json(input): Json<NewTodo>` comes **last**, after `State`.
For the id, `todos.len()` counts the todos; it's a `usize`, so `todos.len() as u64 + 1` turns it
into the id's type first, as `as u32` did in [Testing handlers](testing-handlers.md).

</details>

<details><summary>The book's code for Step 2</summary>

`code/projects/a2-build-todo-api/examples/step2.rs`:

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/step2.rs:all}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example step2`

</details>

**✅ Checkpoint.** Restart your server, then:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/02-step2.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/02-step2.out}}
```

An empty list is `[]`. Each `POST` answers `201 Created` with the todo the server made, `id` and
all, so the client learns the number it was given. The last list holds both, in the order they were
created.

- ✅ If you see `[]`, two `201`s with ids `1` and `2`, then both todos in the list, you're right.
- ❌ If a `POST` answers `415 Unsupported Media Type`, the `-H 'content-type: application/json'`
  part is missing from your `curl` command: `Json` refuses a body that doesn't say it's JSON.
- ❌ If a `POST` answers `422` with ``missing field `title` ``, the JSON's key is spelled differently
  from the struct's field.
- ❌ If the list is still `[]` after a `POST`, each request is getting its own empty list: the list
  must live in the state, behind `Arc`, not be created inside the handler.

### Step 3: real ids, and one todo by its id

**Goal:** `GET /todos/{id}` answers `200` with that todo, or `404` with
`{"error":"todo 99 not found"}` when there's none. And ids now come from a counter that only goes up,
so rule 2 holds even once todos can be deleted.

Why change the ids? "Count plus one" works while todos are only ever added. But Step 4 adds
**delete**, and then it breaks. Follow it through: you create todo 1 and todo 2. You delete todo 1.
Now the list holds one todo, so the next new one gets "count plus one" = **2**. There are now **two**
todos called 2. `GET /todos/2` finds whichever comes first, and `DELETE /todos/2` deletes both.
Worse, a client that saved "my todo is number 2" now points at someone else's todo. A counter that
only ever goes up never hands out a number twice.

<details><summary>Hint 1: the counter</summary>

Keep the counter **next to** the list, behind the **same** lock: a `Store` struct with
`next_id: u64` and `todos: Vec<Todo>`, and `AppState` holding `Arc<Mutex<Store>>`. In `create`, add 1
to `next_id`, then use it as the new id. With `#[derive(Default)]`, `next_id` starts at 0, so the
first todo is 1.

</details>

<details><summary>Hint 2: the 404</summary>

This is [Error handling in Axum](error-handling-in-axum.md): an `AppError` enum with a
`NotFound(u64)` variant and `#[error("todo {0} not found")]`, an `IntoResponse` that sends `404` and
`{"error": …}`, and a handler that returns `Result<Json<Todo>, AppError>`. To find the todo:
`.iter().find(|t| t.id == id)`, then `.cloned()`, then `.ok_or(AppError::NotFound(id))?`.

</details>

<details><summary>The book's code for Step 3</summary>

`code/projects/a2-build-todo-api/examples/step3.rs`:

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/step3.rs:all}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example step3`

</details>

**✅ Checkpoint.** Restart your server, then:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/03-step3.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/03-step3.out}}
```

Todo 2 comes back on its own. There's no todo 99, so the answer is `404 Not Found`, and the body is
JSON (`content-type: application/json`) that a program can read, not an empty page.

- ✅ If you see two `201`s, then todo 2, then the `404` with `todo 99 not found`, you're right.
- ❌ If `GET /todos/2` answers `404` with an **empty** body, the route `/todos/{id}` is missing: the
  `404` came from Axum, not from your `AppError`.
- ❌ If the build fails with `` expected `Todo`, found `&Todo` ``, the `.cloned()` is missing.
  `.find` gives back a **reference** into the list, and the list stays behind the lock; `.cloned()`
  makes a copy you can send:

  ```text
  error[E0308]: mismatched types
    --> projects/a2-build-todo-api/examples/step3.rs:90:13
     |
  90 |     Ok(Json(todo))
     |        ---- ^^^^ expected `Todo`, found `&Todo`
  ```

  (That's the first lines of the real error; the rest suggests `todo.clone()`, which also works.
  The errors on this page were captured from the book's files, so they name paths such as
  `projects/a2-build-todo-api/examples/step3.rs`. In your project the path is `src/main.rs`, and
  the line numbers are different; the message is the same.)

### Step 4: change and delete

**Goal:** `PATCH /todos/{id}` with `{"done":true}`, `{"title":"…"}` or both changes **only** the
fields that were sent and answers `200` with the changed todo. `DELETE /todos/{id}` answers `204`
with no body. Both answer `404` for a todo that doesn't exist.

<details><summary>Hint 1: a body where every field is optional</summary>

A third struct, `UpdateTodo`, with `title: Option<String>` and `done: Option<bool>`. A field the
client leaves out becomes `None` (serde does that for every `Option` field), so `{"done":true}`
gives `title: None, done: Some(true)`. Then, for each field: if it's `Some(value)`, copy the value
into the todo. Rust has a short way to write "if this is `Some`, take what's inside":

```rust,editable
fn main() {
    let new_title: Option<String> = Some("buy oat milk".to_string());
    if let Some(title) = new_title {
        println!("change the title to {title}");
    }
    let new_done: Option<bool> = None;
    if let Some(done) = new_done {
        println!("never printed: there's nothing inside, so this block is skipped ({done})");
    }
}
```

`if let Some(title) = new_title { … }` runs the block only when `new_title` is `Some`, and inside it,
`title` is the value that was inside. (Rust for Humans:
[Pattern matching](https://open-source-bd.github.io/rustbook-for-human/language-basics/pattern-matching.html).)

</details>

<details><summary>Hint 2: changing a todo in place</summary>

To change a todo that's **in** the list, you need a way in that allows changes. `.iter()` hands out
read-only references (`&Todo`); `.iter_mut()` hands out **mutable** ones (`&mut Todo`), through
which you can write `todo.done = done;`. The lock must be `let mut store = …`, because changing
something inside it counts as changing it.

</details>

<details><summary>Hint 3: deleting</summary>

`store.todos.retain(|t| t.id != id)` keeps every todo whose id is **not** `id`, and drops the rest.
It doesn't say whether it dropped anything, so compare `store.todos.len()` before and after: the
same length means there was no such todo, so answer `404`. The success answer is the status alone:
return `StatusCode::NO_CONTENT`, as in [Handlers and IntoResponse](handlers-and-into-response.md).
One path, three methods: `.route("/todos/{id}", get(show).patch(update).delete(remove))`.

</details>

<details><summary>The book's code for Step 4</summary>

`code/projects/a2-build-todo-api/examples/step4.rs`:

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/step4.rs:all}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example step4`

</details>

**✅ Checkpoint.** Restart your server, then:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/04-step4.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/04-step4.out}}
```

Read it as a story. Two todos are made. `{"done":true}` ticks off todo 1 and leaves its title as it
was (rule 3). The rename changes todo 2's title and leaves its `done` alone. The first `DELETE`
answers `204 No Content`, with an empty body. The second finds nothing to delete: `404`. Then the
key moment, **rule 2**: the new todo is number **3**, not 2, even though only one todo was left. The
final list shows todos 2 and 3.

The book's step also has two tests at its bottom, which check the same story without a server.
From the book's `code/` folder:

```bash
cargo test -p a2-build-todo-api --example step4
```

```text
   Compiling a2-build-todo-api v0.1.0 (/Users/you/rust-book-backend/code/projects/a2-build-todo-api)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests examples/step4.rs (target/debug/examples/step4-ee49118f9deeb21a)

running 2 tests
test tests::patch_changes_only_what_you_send ... ok
test tests::delete_then_ids_are_never_reused ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`--example step4` runs the tests inside that one example file. Every step's file has tests like
these; Step 6 explains how they're written.

- ✅ If the new todo is `"id":3`, rule 2 holds.
- ❌ If the new todo is `"id":2`, your `create` still uses "count plus one": go back to Step 3.
- ❌ If the build fails with `` cannot borrow `store` as mutable ``, the lock in `update` is
  `let store`; make it `let mut store`, as the compiler's `help:` says:

  ```text
  error[E0596]: cannot borrow `store` as mutable, as it is not declared as mutable
     --> projects/a2-build-todo-api/examples/step4.rs:110:16
      |
  110 |     let todo = store
      |                ^^^^^ cannot borrow as mutable
      |
  help: consider changing this to be mutable
      |
  109 |     let mut store = state.store.lock().unwrap();
      |         +++
  ```

- ❌ If `PATCH` with only `{"done":true}` answers `422` with ``missing field `title` ``, a field of
  `UpdateTodo` isn't an `Option`.

### Step 5: refuse bad titles

**Goal:** rule 1. Creating a todo with an empty title, or one over 100 characters, answers `422` with
validator's list of what's wrong. A `PATCH` that changes the title follows the same rule; a `PATCH`
without a title is still fine.

<details><summary>Hint</summary>

This is [Input validation](input-validation.md), almost word for word. Add `Validate` to the
`derive` of `NewTodo` and `UpdateTodo`, and `#[validate(length(min = 1, max = 100))]` above each
`title`. Copy the lesson's `ValidatedJson` extractor, including its `serde_json::to_value` line, and
use `ValidatedJson(input): ValidatedJson<NewTodo>` in place of `Json(input): Json<NewTodo>` (and the
same for `UpdateTodo`). On `UpdateTodo`, the rule sits on an `Option<String>`, so it checks the
title only when one is sent: exactly what you want here (that lesson's *Common mistakes* shows why).

</details>

<details><summary>The book's code for Step 5</summary>

`code/projects/a2-build-todo-api/examples/step5.rs`:

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/step5.rs:all}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example step5`

</details>

**✅ Checkpoint.** Restart your server, then:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/05-step5.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/05-step5.out}}
```

The empty title is refused before the handler runs, so no todo is made, and no id is used up: the
next good todo is still number 1. validator's answer names the field (`title`), the rule that
failed (`length`), the rule's limits (`min` 1, `max` 100), and the value it got (`""`). The `PATCH`
with an empty title gets the same `422`; the `PATCH` with only `done` passes, because there's no
title to check.

- ✅ If you see `422`, `201` with id `1`, `422`, then `200` with `"done":true`, you're right.
- ❌ If the empty title gets `201`, the handler still takes `Json<NewTodo>`: plain `Json` never runs
  the rules.
- ❌ If the keys inside `params` come out in a different order each time you restart the server,
  the `serde_json::to_value` line is missing from your `ValidatedJson` (Input validation, Step 4,
  explains why).

### Step 6: files, `/api`, and tests

**Goal:** the same API, split into files the way a real project is, with the todo routes moved
under `/api/todos`, and two automated tests that check the whole lifecycle without a server. This
step has no new example file: the result **is** the project's `src/` and `tests/` folders.

Split `src/main.rs` into these files, in the same `todo-api` folder:

- `src/lib.rs`: the library, with one `pub fn app()` that builds the whole Router. `GET /health`
  stays here, and the todo routes come in with `.nest("/api/todos", todos::router())`.
- `src/error.rs`: `AppError` and its `IntoResponse`.
- `src/validated.rs`: `ValidatedJson`.
- `src/todos.rs`: the types, the store, the five handlers, and a `pub fn router()`.
- `src/main.rs`: a thin `main` that serves `todo_api::app()`.
- `tests/api.rs`: tests that drive `app()` with `oneshot`.

<details><summary>Hint 1: modules that use each other</summary>

As in [Nesting and modular routers](nesting-and-modular-routers.md), `mod error;`, `mod todos;` and
`mod validated;` in `lib.rs` add the files to the library. `todos.rs` then names the other two by
their path from the top of the library: `use crate::{error::AppError, validated::ValidatedJson};`.
For that to work, `AppError`, `ValidatedJson` **and** its field must be `pub`, and so must
`todos::router`. The *Reference solution* shows the exact error for each missing `pub`.

</details>

<details><summary>Hint 2: the paths inside the todos router</summary>

`.nest("/api/todos", …)` puts `/api/todos` in front of every path in the nested Router. So inside
`todos.rs`, the list is `"/"` and one todo is `"/{id}"`: together they make `/api/todos` and
`/api/todos/{id}`.

End `router()` with `.with_state(AppState::default())`, so it gives back a plain, ready `Router`.
Then `app()` never sees the store: it nests a finished Router. (If you leave `.with_state` in
`app()` instead, `router()` returns a `Router<AppState>`, which `app()` must name, so `AppState`
has to become `pub`, and the type errors pile up.)

</details>

<details><summary>Hint 3: the tests</summary>

This is [Testing handlers](testing-handlers.md). Your library's name is your package's name with
`-` turned into `_`: the book's package is `a2-build-todo-api`, so its tests write
`use a2_build_todo_api::app;`; yours is `todo-api`, so write `use todo_api::app;` (and
`todo_api::app()` in `main.rs`). One `send` helper that takes a method, a path and an optional JSON
body keeps each check to a line or two. To keep state between requests, build `app()` once and send
`app.clone()` each time.

</details>

The book's files are in `code/projects/a2-build-todo-api/`, and each one is shown and explained in
the *Reference solution* below.

**✅ Checkpoint 1: the tests.** In your `todo-api` folder, run `cargo test`. The book's own, from
the book's `code/` folder:

```bash
cargo test -p a2-build-todo-api
```

```text
   Compiling a2-build-todo-api v0.1.0 (/Users/you/rust-book-backend/code/projects/a2-build-todo-api)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.02s
     Running unittests src/lib.rs (target/debug/deps/a2_build_todo_api-07ecaa971d06a922)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/a2_build_todo_api-1da7372c13de5a2c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/api.rs (target/debug/deps/api-06b4dc6f9e91090d)

running 2 tests
test full_lifecycle ... ok
test validation_and_rejections ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests a2_build_todo_api

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

As in [Testing handlers](testing-handlers.md#run-it): the library and the program have no tests of
their own (`running 0 tests`), and `tests/api.rs` has two, which both pass. Between them, they send
eleven requests through the whole app, and need no server, no port and no `curl`. (A plain
`cargo test` doesn't run the tests inside `examples/`; `--example step4` in Step 4 did that for one
file.)

**✅ Checkpoint 2: the real server.** Start your server with `cargo run` (the book's:
`cd code && cargo run -p a2-build-todo-api`). The first terminal shows:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/a2-build-todo-api`
Todo API listening on http://127.0.0.1:3000
```

Then, in the second terminal, the conversation from *What you'll build*:

```bash
{{#include ../../code/projects/a2-build-todo-api/http/06-final.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/06-final.out}}
```

Same answers as Steps 1 to 5, now at `/api/todos`. That's your first acceptance check, and with the
tests passing, your second.

- ✅ If you see `2 passed` and this conversation, you're done. You built it.
- ❌ If every `/api/todos` request answers `404` with an empty body, check the paths inside
  `todos.rs`: they must be `"/"` and `"/{id}"`, not `"/todos"` (which would make
  `/api/todos/todos`).
- ✅ If `curl -i http://127.0.0.1:3000/api/todos/` (with a `/` at the end) answers `404`, that's
  right too: Axum matches paths exactly ([Nesting and modular routers](nesting-and-modular-routers.md)
  explains the trailing slash).
- ❌ If the build fails with `is private`, a `pub` is missing; the *Reference solution* shows each
  case.

## Reference solution, line by line

These are the book's files: the book's automatic checks build them, run `tests/api.rs`, and replay
every checkpoint against them on every change, so the outputs you've seen are exactly what they
produce. Lines you've seen in earlier lessons get a short note with a link; the ones that are new
**in combination** get the full treatment.

### The files

```text
code/projects/a2-build-todo-api/
├── Cargo.toml
├── src/
│   ├── lib.rs         app(): /health, and the todos nested at /api/todos
│   ├── error.rs       AppError → 404 + JSON
│   ├── validated.rs   ValidatedJson: Json + validator rules → 422
│   ├── todos.rs       Todo, the store, five handlers, router()
│   └── main.rs        serve a2_build_todo_api::app() on port 3000
├── tests/
│   └── api.rs         two tests that drive app() with oneshot
├── examples/          step1.rs … step5.rs, and the stretch-goal solutions
└── http/              the checkpoints' curl scripts and their real output
```

### Line by line

`Cargo.toml`
- **What:** the project's settings: its name and its crates.
- **Why:** Cargo needs it to build anything.
- **How:** shown in full in the next section.
- **Remove it and…** Cargo doesn't know this folder is a project.

`src/lib.rs  app(): /health, and the todos nested at /api/todos`
- **What:** the top of the library: it names the other three modules and builds the whole Router.
- **Why:** the tests and `main.rs` need `app()`, so it lives in the library, as in
  [Testing handlers](testing-handlers.md).
- **How:** Cargo builds a library from `src/lib.rs` without being told.
- **Remove it and…** there's nothing for `main.rs` or the tests to import.

`src/error.rs  AppError → 404 + JSON`
- **What:** the app's error type and how it turns into a response.
- **Why:** one place decides what every error looks like, so all errors look alike.
- **How:** `mod error;` in `lib.rs` adds it to the library.
- **Remove it and…** `todos.rs` has no `AppError` to return.

`src/validated.rs  ValidatedJson: Json + validator rules → 422`
- **What:** the extractor from Input validation.
- **Why:** it's not about todos; any future area of the app (users, lists) can use it as it is. Its
  own file says so.
- **How:** `mod validated;` in `lib.rs`.
- **Remove it and…** `todos.rs` can't find `ValidatedJson` (see `lib.rs` below for the error).

`src/todos.rs  Todo, the store, five handlers, router()`
- **What:** everything about todos: types, storage, handlers and routes.
- **Why:** one file per area, as in [Nesting and modular routers](nesting-and-modular-routers.md).
  A second area later (say, users) would get its own file and its own `router()`.
- **How:** it shares nothing with the rest of the app except `pub fn router()`.
- **Remove it and…** the API has only `/health`.

`src/main.rs  serve a2_build_todo_api::app() on port 3000`
- **What:** the program you start with `cargo run`.
- **Why:** a library can't run by itself; something must open the port.
- **How:** when a package has both `src/lib.rs` and `src/main.rs`, Cargo builds both.
- **Remove it and…** `cargo run -p a2-build-todo-api` has no program to run; the tests still pass.

`tests/  api.rs  two tests that drive app() with oneshot`
- **What:** the [integration tests](../glossary.md#integration-test).
- **Why:** they check the app from the outside, through `app()`, as a real client would.
- **How:** Cargo finds a folder called `tests` next to `src` on its own.
- **Remove it and…** `cargo test` finds no tests to run.

`examples/` · `http/`
- **What:** the book's version of each step and stretch goal, and the scripts behind every
  checkpoint.
- **Why:** so you can run any step, and so the book's checks can replay every transcript on this page.
- **How:** `cargo run -p a2-build-todo-api --example step3` runs `examples/step3.rs`.
- **Remove it and…** the app is the same; you lose the steps to compare with.

### `Cargo.toml`

```toml
{{#include ../../code/projects/a2-build-todo-api/Cargo.toml}}
```

The versions and features are written once, in the workspace's `code/Cargo.toml`:

```toml
{{#include ../../code/Cargo.toml:axum}}
{{#include ../../code/Cargo.toml:tokio}}
{{#include ../../code/Cargo.toml:test_deps}}
{{#include ../../code/Cargo.toml:serde}}
{{#include ../../code/Cargo.toml:serde_json}}
{{#include ../../code/Cargo.toml:thiserror}}
{{#include ../../code/Cargo.toml:validator}}
```

### Line by line

`[package]` · `name = "a2-build-todo-api"` · `version = "0.1.0"`
- **What:** the project's name and version.
- **Why:** the name is what you type after `-p`, and the library's name, `a2_build_todo_api`, comes
  from it.
- **How:** named after this page, as every crate in the book is.
- **Remove it and…** (`name`) Cargo stops with an error saying the name is missing.

`edition.workspace = true` · `publish.workspace = true`
- **What:** "take these two settings from the workspace's `code/Cargo.toml`".
- **Why:** every project in the book uses `edition = "2024"` and `publish = false`.
- **How:** [Hello, Axum, Step 2](hello-axum.md#step-2-the-books-copy-of-the-same-project) explains
  `.workspace = true`. This project sits in `code/projects/`, not `code/topics/`, and is still part
  of the workspace: `code/Cargo.toml` lists both folders as `members`.
- **Remove it and…** (`edition`) Cargo falls back to edition 2015, where `async fn` isn't allowed.

`axum.workspace = true` · `axum = "0.8.9"`
- **What:** Axum 0.8.9: the Router, the extractors, `Json`, `StatusCode`.
- **Why:** it's the web framework the whole app is built on.
- **How:** `"0.8.9"` allows any later 0.8 fix release, never 0.9.
- **Remove it and…** every `use axum::…` line fails with ``unresolved import `axum` ``.

`tokio.workspace = true` · `tokio = { version = "1.53.1", features = [ … ] }`
- **What:** the async runtime, with the features `main` needs.
- **Why:** `#[tokio::main]` and `#[tokio::test]` come from `macros`, the runtime from
  `rt-multi-thread`, and `TcpListener` from `net`. (`time` is there for other lessons.)
- **How:** [Tour of the stack](../part-0-start/tour-of-the-stack.md) explains each feature.
- **Remove it and…** `main` can't start, and the tests can't be `async`.

`serde.workspace = true` · `serde = { version = "1.0.229", features = ["derive"] }`
- **What:** serde, with the `derive` feature.
- **Why:** `#[derive(Serialize, Deserialize)]` turns todos into JSON and request bodies into structs.
- **How:** as in [JSON with serde](json-and-serde.md).
- **Remove it and…** ``unresolved import `serde` `` in `todos.rs` and `validated.rs`.

`serde_json.workspace = true` · `serde_json = "1.0.151"`
- **What:** the crate that reads and writes JSON text.
- **Why:** the app's own code calls it twice: `json!(…)` builds the error body in `error.rs`, and
  `serde_json::to_value` sorts validator's errors in `validated.rs`.
- **How:** a normal dependency, not a dev-dependency, because the server uses it, not only the
  tests.
- **Remove it and…** ``unresolved import `serde_json` `` in `error.rs`, and
  ``cannot find module or crate `serde_json` in this scope`` in `validated.rs`.

`thiserror.workspace = true` · `thiserror = "2.0.21"`
- **What:** the crate that writes an error type's message for you.
- **Why:** `#[error("todo {0} not found")]` on `AppError` is all it takes to get that message.
- **How:** as in [Error handling in Axum](error-handling-in-axum.md).
- **Remove it and…** ``cannot find module or crate `thiserror` in this scope``, on `AppError`'s
  `derive`.

`validator.workspace = true` · `validator = { version = "0.21.0", features = ["derive"] }`
- **What:** the validation crate, with its `derive` feature.
- **Why:** rule 1, the 1-to-100-character title, is a `#[validate(length(…))]` line.
- **How:** as in [Input validation](input-validation.md).
- **Remove it and…** ``unresolved import `validator` ``.

`[dev-dependencies]` · `tower.workspace = true` · `http-body-util.workspace = true`
- **What:** the two test-only crates: Tower for `oneshot`, `http-body-util` for reading a body.
- **Why:** only `tests/api.rs` and the examples' tests use them.
- **How:** `[dev-dependencies]` are built for `cargo test`, never for the server you `cargo run`.
- **Remove it and…** `cargo run` still works; the tests fail with ``unresolved import `tower` ``.

### `src/lib.rs`

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/lib.rs:lib}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api`

### Line by line

`mod error;` · `mod todos;` · `mod validated;`
- **What:** adds the three files to the library, as the modules `error`, `todos` and `validated`.
- **Why:** a file in `src/` is compiled only if a `mod` line names it.
- **How:** `mod error;` tells Cargo "compile `src/error.rs`, and call it `error`". Without `pub`,
  the modules are private to the library: code inside it (such as `todos.rs`) can reach into them,
  and code outside it (`main.rs`, the tests) can't. That's what we want: outsiders only need `app()`.
- **Remove it and…** (`mod validated;`) the library no longer has a `validated` module, and the
  `use` line in `todos.rs` fails:

  ```text
  error[E0432]: unresolved import `crate::validated`
   --> projects/a2-build-todo-api/src/todos.rs:2:30
    |
  2 | use crate::{error::AppError, validated::ValidatedJson};
    |                              ^^^^^^^^^ could not find `validated` in the crate root
  ```

`use axum::Router;`
- **What:** the one Axum name this file uses by its short name.
- **Why:** `app()` returns a `Router`.
- **How:** the `get` below is written as `axum::routing::get`, its full path, so it needs no `use`
  line. Both ways work; a full path is handy for a name used once.
- **Remove it and…** ``cannot find type `Router` in this scope``.

`pub fn app() -> Router {`
- **What:** the library's one public door: a function that builds the whole app.
- **Why:** `main.rs` serves it; the tests send requests to it.
- **How:** `pub` is what lets code outside the library call it.
- **Remove it and…** (`pub`) ``error[E0603]: function `app` is private``, in `main.rs` and in
  `tests/api.rs`, as in [Testing handlers](testing-handlers.md).

`Router::new()` · `.route("/health", axum::routing::get(|| async { "ok" }))`
- **What:** `GET /health` answers `ok`.
- **Why:** the health check from Step 1. It's about the whole app, not about todos, so it stays at
  the top.
- **How:** the handler is a closure written in place, as in Testing handlers: it takes nothing and
  returns `"ok"`.
- **Remove it and…** `GET /health` answers `404`, and the last check in
  `validation_and_rejections` fails.

`.nest("/api/todos", todos::router())` · `}`
- **What:** mounts the todos' own Router under `/api/todos`.
- **Why:** this is where the whole todos area joins the app, in one line. `/api` at the front is a
  common habit: it keeps the API's paths apart from anything else the server might serve later,
  such as web pages.
- **How:** as in [Nesting and modular routers](nesting-and-modular-routers.md): the prefix is put in
  front of every path in the nested Router, so its `"/"` becomes `/api/todos` and its `"/{id}"`
  becomes `/api/todos/{id}`. `todos::router()` already has its state (it called `.with_state`), so
  `app()` never needs to know the todos have a store at all.
- **Remove it and…** every `/api/todos` request answers `404`, and the tests fail.

### `src/error.rs`

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/error.rs:imports}}

{{#include ../../code/projects/a2-build-todo-api/src/error.rs:error}}

{{#include ../../code/projects/a2-build-todo-api/src/error.rs:into_response}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api`

### Line by line

`use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};` · `use serde_json::json;`
- **What:** the names the error needs: `Json` for the body, `StatusCode` for `404`, `IntoResponse`
  and `Response` for turning an error into an answer, and the `json!` macro.
- **Why:** this file is Error handling in Axum's error type, in a file of its own.
- **How:** as in [Error handling in Axum](error-handling-in-axum.md).
- **Remove it and…** (`json`) ``cannot find macro `json` in this scope``.

`#[derive(Debug, thiserror::Error)]` · `pub enum AppError {`
- **What:** the app's error type, with its message written by thiserror.
- **Why:** `pub` because `todos.rs`, a **different** module, returns it. A module's items are
  private to that module unless marked `pub`; and since the module `error` itself is private to the
  library, `pub` here means "anywhere in this library", not "anywhere in the world".
- **How:** `Debug` and `thiserror::Error` together make it a proper Rust error, as in Error handling
  in Axum.
- **Remove it and…** (`pub`) the `use` line in `todos.rs` can't reach it:

  ```text
  error[E0603]: enum `AppError` is private
    --> projects/a2-build-todo-api/src/todos.rs:2:20
     |
   2 | use crate::{error::AppError, validated::ValidatedJson};
     |                    ^^^^^^^^ private enum
     |
  note: the enum `AppError` is defined here
    --> projects/a2-build-todo-api/src/error.rs:12:1
     |
  12 | enum AppError {
     | ^^^^^^^^^^^^^
  ```

`#[error("todo {0} not found")]` · `NotFound(u64),` · `}`
- **What:** the one thing that can go wrong: no todo with this id. It carries the id.
- **Why:** rule 5: the message names the todo, `todo 7 not found`.
- **How:** `{0}` is replaced by the variant's first value, the `u64`. The id is a `u64`, like
  `Todo::id`.
- **Remove it and…** (`#[error(…)]`) thiserror writes no message, and the build fails with
  ``error[E0277]: `AppError` doesn't implement `std::fmt::Display` ``.

`impl IntoResponse for AppError {` · `fn into_response(self) -> Response {`
- **What:** teaches Axum how to send an `AppError`.
- **Why:** handlers return `Result<…, AppError>`, and Axum must turn the `Err` into an HTTP answer.
- **How:** as in Error handling in Axum.
- **Remove it and…** no handler that returns `Result<_, AppError>` counts as a handler, and the
  Router won't compile.

`let status = match self {` · `AppError::NotFound(_) => StatusCode::NOT_FOUND,` · `};`
- **What:** picks the status for each kind of error.
- **Why:** one variant today, but a `match` is ready for more: add a variant, and the compiler
  points here until you give it a status.
- **How:** `_` means "whatever id is inside; we don't need it here".
- **Remove it and…** there's no `status` for the next line.

`(status, Json(json!({ "error": self.to_string() }))).into_response()` · `}` · `}`
- **What:** the answer: the status, and a JSON body like `{"error":"todo 2 not found"}`.
- **Why:** a program reading the API gets JSON for errors too, the same format as everything else.
- **How:** `self.to_string()` is thiserror's message; `json!` builds the JSON object; `Json(…)`
  sends it with `content-type: application/json`.
- **Remove it and…** (the `Json(…)` part) the body would be empty, and a client couldn't tell why.

### `src/validated.rs`

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/validated.rs:imports}}

{{#include ../../code/projects/a2-build-todo-api/src/validated.rs:extractor}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api`

### Line by line

`use axum::{…};` · `use serde::de::DeserializeOwned;` · `use validator::Validate;`
- **What:** the names the extractor needs: `FromRequest` to be an extractor, `Json` and
  `JsonRejection` to reuse Axum's JSON reading, `StatusCode` for `422`, and the two traits a body
  type must have.
- **Why:** this is [Input validation, Step 4](input-validation.md#step-4-the-validatedjson-extractor)
  in a file of its own.
- **How:** that lesson explains each one.
- **Remove it and…** (`Validate`) the extractor can't ask for `T: Validate`.

`pub struct ValidatedJson<T>(pub T);`
- **What:** the extractor: a wrapper around a body that was parsed **and** passed its rules.
- **Why:** **two** `pub`s. The first lets `todos.rs` name the type. The second, on the field, lets
  `todos.rs` take the value out with the pattern `ValidatedJson(input)`. In Input validation it was
  all one file, so neither was needed.
- **How:** a tuple struct's field is private like any other unless marked `pub`. A pattern that
  unpacks it has to see the field.
- **Remove it and…** (the field's `pub`) both handlers that use it fail:

  ```text
  error[E0532]: cannot match against a tuple struct which contains private fields
    --> projects/a2-build-todo-api/src/todos.rs:58:5
     |
  58 |     ValidatedJson(input): ValidatedJson<NewTodo>,
     |     ^^^^^^^^^^^^^
  ```

  (the first lines of the real error, which then shows the same for `update`), and the compiler's
  `help:` suggests exactly the fix: `pub struct ValidatedJson<T>(pub T);`.

`impl<T, S> FromRequest<S> for ValidatedJson<T>` · `where` · … · `type Rejection = Response;`
- **What:** makes `ValidatedJson<T>` an extractor that reads the body, for any `T` that can be
  read from JSON and has rules.
- **Why:** one extractor serves both `NewTodo` and `UpdateTodo`.
- **How:** word for word Input validation's: [Custom extractors](custom-extractors.md) explains
  `FromRequest`, and Input validation explains the `where` lines.
- **Remove it and…** `ValidatedJson<NewTodo>` isn't an extractor, and `create` isn't a handler.

`let Json(value) = Json::<T>::from_request(request, state)` · `.await` · `.map_err(|rejection| rejection.into_response())?;`
- **What:** first, read the body as JSON, exactly as `Json` would.
- **Why:** broken JSON, a missing `content-type` or a missing field still gets Axum's own answer
  (`400`, `415` or `422`), unchanged.
- **How:** as in Input validation.
- **Remove it and…** there's no `value` to check.

`value.validate().map_err(|errors| {` · `let sorted = serde_json::to_value(errors).expect(…);` · `(StatusCode::UNPROCESSABLE_ENTITY, Json(sorted)).into_response()` · `})?;`
- **What:** then, check the rules; if any fail, answer `422` with the list of failures, its keys
  sorted.
- **Why:** rule 1. And the sorting makes the answer the same, byte for byte, on every run: validator
  keeps its errors (and each error's `params`) in a `HashMap`, whose order is shuffled each time the
  program starts. Even Step 5's one-field answer has a `params` map with three keys (`max`, `min`,
  `value`), which would come out in a different order from run to run.
- **How:** [Input validation](input-validation.md) explains `serde_json::to_value` line by line.
- **Remove it and…** (the `to_value` line, writing `Json(errors)` below) it still works, but the
  `params` keys move around between runs, and the checkpoints in this book couldn't be checked.

`Ok(ValidatedJson(value))`
- **What:** both checks passed: hand the value to the handler.
- **Why:** from here on, the handler knows the title is 1 to 100 characters.
- **How:** wraps the value in the extractor's type.
- **Remove it and…** the function returns nothing, and won't compile.

### `src/todos.rs`

The biggest file. It's shown in pieces, top to bottom, in the order it's written.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:imports}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api`

### Line by line

`use crate::{error::AppError, validated::ValidatedJson};`
- **What:** brings in the two items this file takes from the rest of the library.
- **Why:** the handlers return `AppError` and take `ValidatedJson`.
- **How:** `crate` means "the top of this library", `lib.rs`. From there, `error::AppError` is the
  path to the enum. Both items are `pub`, so a sibling module can use them.
- **Remove it and…** ``cannot find type `AppError` in this scope``, once per handler.

`use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};`
- **What:** the Axum names the handlers and the router use.
- **Why:** `Path` for the id, `State` for the store, `StatusCode` for `201` and `204`, `get` to
  start each route.
- **How:** `.post`, `.patch` and `.delete` are methods on what `get` returns, so they need no
  `use` line.
- **Remove it and…** (`Path`) ``cannot find tuple struct or tuple variant `Path` in this scope``.

`use serde::{Deserialize, Serialize};` · `use std::sync::{Arc, Mutex};` · `use validator::Validate;`
- **What:** serde's derive names, the shared-store types, and validator's derive name.
- **Why:** todos go to and from JSON, the store is shared, and the inputs have rules.
- **How:** as in JSON with serde, Shared state and Input validation.
- **Remove it and…** (`Validate`) ``cannot find derive macro `Validate` in this scope``.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:types}}
```

### Line by line

`#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]` · `pub struct Todo {` · `pub id: u64,` · `pub title: String,` · `pub done: bool,` · `}`
- **What:** one todo, as the server stores it and sends it.
- **Why:** the server needs `Clone` (one copy in the list, one in the answer) and `Serialize` (to
  send it as JSON). `Deserialize`, `Debug` and `PartialEq` are unused today. `tests/api.rs` can't
  name `Todo` (the module `todos` is private), so it compares JSON text instead. They're there for
  a unit test inside `todos.rs`, which could read a todo back from JSON and compare it with
  `assert_eq!`, as in Testing handlers.
- **How:** `u64` is a whole number from 0 up, big enough that the counter never runs out. The
  `pub`s change nothing today: the module `todos` is private to the library, and only this file
  uses `Todo`. They're a habit for a type that **is** the API's shape, ready for the day the module
  becomes public.
- **Remove it and…** (`Serialize`) `Json<Todo>` can't be sent, and the handlers stop being
  handlers. (the `pub`s) everything still compiles and works.

`#[derive(Deserialize, Validate)]` · `pub struct NewTodo {` · `#[validate(length(min = 1, max = 100))]` · `pub title: String,` · `}`
- **What:** what a client sends to create a todo: only the title.
- **Why:** rule 4: the client never chooses the `id` or `done`. With no such fields here, there's
  no way to send them: serde ignores unknown keys, so `{"title":"x","done":true}` makes a todo
  that's **not** done.
- **How:** `length(min = 1, max = 100)` counts characters, as in Input validation.
- **Remove it and…** (the `#[validate(…)]` line) an empty title is accepted with `201`.

`#[derive(Deserialize, Validate)]` · `pub struct UpdateTodo {` · `pub title: Option<String>,` · `pub done: Option<bool>,` · `}`
- **What:** what a client sends to change a todo: a title, a `done`, or both.
- **Why:** rule 3. A missing key becomes `None`, so the handler can tell "not sent" apart from "sent".
- **How:** serde treats an `Option` field as optional: no key, no error, `None`. The length rule on
  an `Option<String>` checks only a title that's there (the behaviour from Input validation's
  *Common mistakes*, wanted this time).
- **Remove it and…** (the `Option` on `title`) `{"done":true}` answers `422` with
  ``missing field `title` ``, and you'd have to send the title every time.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:store}}
```

### Line by line

`#[derive(Default)]` · `struct Store {` · `next_id: u64,` · `todos: Vec<Todo>,` · `}`
- **What:** everything the API remembers: the last id handed out, and the todos.
- **Why:** rule 2. `next_id` only ever goes up, so no number is handed out twice, whatever is
  deleted.
- **How:** `Default` makes `Store::default()`: `next_id` 0, an empty list. No `pub`: nothing outside
  this file touches the store.
- **Remove it and…** (`next_id`) you're back to "count plus one", and Step 3's duplicate ids.

`#[derive(Clone, Default)]` · `struct AppState {` · `store: Arc<Mutex<Store>>,` · `}`
- **What:** the state every handler receives: one shared, lockable `Store`.
- **Why:** **one** lock around **both** fields. `create` reads the counter, adds one, and pushes the
  todo. If the counter had its own lock, two requests at the same moment could each read 4, and
  both make a todo 5. With one lock, the second request waits until the first has finished all
  three steps.
- **How:** as in [Shared state](shared-state.md): `Clone` copies the `Arc` (a pointer), so every
  request shares one store; `Default` starts it empty.
- **Remove it and…** (`Clone`) a pile of ``the trait bound `AppState: Clone` is not satisfied``
  errors: Axum hands each request its own clone.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:list}}

{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:create}}
```

### Line by line

`async fn list(State(state): State<AppState>) -> Json<Vec<Todo>> {` · `Json(state.store.lock().unwrap().todos.clone())` · `}`
- **What:** `GET /api/todos`: lock the store, copy the list, send the copy as a JSON array.
- **Why:** the copy lets the lock go at once, before the JSON is written.
- **How:** `.lock()` waits for the lock and gives back a `Result`; `.unwrap()` takes the guard out
  (it fails only if an earlier request crashed while holding the lock, as
  [Error handling in Axum](error-handling-in-axum.md) showed). The guard is a temporary value, so
  the lock is released at the end of this one line.
- **Remove it and…** (`.clone()`) ``error[E0507]: cannot move out of dereference of
  `std::sync::MutexGuard<'_, Store>` ``: the list belongs to the store, and can't be carried off
  from behind the lock.

`async fn create(` · `State(state): State<AppState>,` · `ValidatedJson(input): ValidatedJson<NewTodo>,` · `) -> (StatusCode, Json<Todo>) {`
- **What:** `POST /api/todos`: receives the store and a body that has already passed rule 1.
- **Why:** the handler doesn't check the title at all; by the time it runs, the extractor has.
- **How:** the body extractor is last, as always.
- **Remove it and…** (`ValidatedJson`, back to `Json`) empty titles get in.

`let mut store = state.store.lock().unwrap();` · `store.next_id += 1;`
- **What:** lock the store for changing, and move the counter on by one.
- **Why:** the new todo's id is the counter's **new** value: 1 for the first todo, because it started
  at 0.
- **How:** `mut` because the counter and the list change; `+= 1` adds one to it in place.
- **Remove it and…** (`mut`) ``cannot borrow `store` as mutable``.

`let todo = Todo {` · `id: store.next_id,` · `title: input.title,` · `done: false,` · `};`
- **What:** builds the new todo.
- **Why:** rule 4: `done` is always `false` at first.
- **How:** `input.title` moves the title out of the request into the todo; no copy needed.
- **Remove it and…** (`done: false`) ``missing field `done` in initializer of `Todo` ``.

`store.todos.push(todo.clone());` · `(StatusCode::CREATED, Json(todo))` · `}`
- **What:** store a copy, send the original back with `201`.
- **Why:** the client learns the id it was given.
- **How:** the lock is released at the closing `}`, when `store` goes away.
- **Remove it and…** (`.clone()`) ``use of moved value: `todo` ``.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:show}}
```

### Line by line

`async fn show(State(state): State<AppState>, Path(id): Path<u64>) -> Result<Json<Todo>, AppError> {`
- **What:** `GET /api/todos/{id}`: either the todo, or an `AppError`.
- **Why:** "not found" is a normal answer, so the handler says so in its type.
- **How:** `Path(id): Path<u64>` reads the number from the path, as in
  [Path and Query extractors](path-and-query-extractors.md). `/api/todos/abc` isn't a `u64`, so Axum
  answers `400` before the handler runs.
- **Remove it and…** (the `Result`) there's no way to answer `404`.

`let store = state.store.lock().unwrap();`
- **What:** lock the store, read-only this time.
- **Why:** nothing changes, so no `mut`.
- **How:** as in `list`.
- **Remove it and…** there's no list to search.

`let todo = store` · `.todos` · `.iter()` · `.find(|t| t.id == id)` · `.cloned()` · `.ok_or(AppError::NotFound(id))?;` · `Ok(Json(todo))`
- **What:** find the todo with this id, copy it, or stop with `NotFound`.
- **Why:** one chain, read top to bottom: walk the list, take the first match, copy it out from behind
  the lock, turn "none" into the error.
- **How:** `.find` gives `Option<&Todo>`, a reference into the list. `.cloned()` makes it
  `Option<Todo>`, a copy. `.ok_or(…)` makes it `Result<Todo, AppError>`, and `?` returns the error
  from the handler right there, as in Error handling in Axum. Axum then calls
  `AppError::into_response`, and you get the `404` and its JSON.
- **Remove it and…** (`.cloned()`) `` expected `Todo`, found `&Todo` ``, the error from Step 3.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:update}}
```

### Line by line

`async fn update(` · `State(state): State<AppState>,` · `Path(id): Path<u64>,` · `ValidatedJson(input): ValidatedJson<UpdateTodo>,` · `) -> Result<Json<Todo>, AppError> {`
- **What:** `PATCH /api/todos/{id}`: the store, the id, and a checked body.
- **Why:** three extractors, the body last.
- **How:** `ValidatedJson` checks the title only if one was sent (the rule is on an `Option`).
- **Remove it and…** (`Path`) the handler can't tell which todo to change.

`let mut store = state.store.lock().unwrap();`
- **What:** lock the store for changing.
- **Why:** a todo inside it is about to change.
- **How:** `mut`, as in `create`.
- **Remove it and…** `` cannot borrow `store` as mutable ``, the error from Step 4.

`let todo = store` · `.todos` · `.iter_mut()` · `.find(|t| t.id == id)` · `.ok_or(AppError::NotFound(id))?;`
- **What:** find the todo, this time as something you can **change** in place.
- **Why:** the change must happen to the todo **in the list**, not to a copy.
- **How:** **new:** `.iter_mut()` walks the list handing out `&mut Todo`, mutable references, where
  `.iter()` handed out read-only ones. So `todo` is a `&mut Todo` pointing into the list, and
  writing through it changes the stored todo. No `.cloned()` here: a copy would be changed and then
  thrown away.
- **Remove it and…** (`_mut`, writing `.iter()`) the assignments below fail with
  `` cannot assign to `todo.title`, which is behind a `&` reference ``.

`if let Some(title) = input.title {` · `todo.title = title;` · `}`
- **What:** if the client sent a title, put it in the todo.
- **Why:** rule 3: no title sent, title unchanged.
- **How:** **new:** `if let Some(title) = input.title` runs the block only when `input.title` is
  `Some`, and names what's inside `title`. When it's `None`, the block is skipped. It's a short
  `match` with one arm you care about (Rust for Humans:
  [Pattern matching](https://open-source-bd.github.io/rustbook-for-human/language-basics/pattern-matching.html)).
- **Remove it and…** (the `if let`, writing `todo.title = input.title;`) ``mismatched types``:
  ``expected `String`, found `Option<String>` ``. Rust won't let a "maybe a title" become a title by
  accident.

`if let Some(done) = input.done {` · `todo.done = done;` · `}`
- **What:** the same for `done`.
- **Why:** `{"title":"…"}` alone must not reset `done`.
- **How:** two separate `if let`s, so each field is handled on its own. `{}` changes nothing and
  answers the todo as it is.
- **Remove it and…** `PATCH` can rename, but never tick anything off.

`Ok(Json(todo.clone()))` · `}`
- **What:** answer `200` with the todo as it is now.
- **Why:** the client sees the result of its change, with both fields.
- **How:** `todo` is a reference into the list; `.clone()` makes an owned copy to send. The lock is
  released at the `}`.
- **Remove it and…** (`.clone()`) `` expected `Todo`, found `&mut Todo` ``.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:remove}}
```

### Line by line

`async fn remove(` · `State(state): State<AppState>,` · `Path(id): Path<u64>,` · `) -> Result<StatusCode, AppError> {`
- **What:** `DELETE /api/todos/{id}`: answers with a status alone, or an `AppError`.
- **Why:** a successful delete has nothing to send back, so the success type is `StatusCode`.
- **How:** called `remove`, not `delete`, so it doesn't look like the `delete` method on the route.
- **Remove it and…** (the `Result`) no way to answer `404`.

`let mut store = state.store.lock().unwrap();` · `let before = store.todos.len();`
- **What:** lock the store, and count the todos.
- **Why:** the count is how we'll know whether anything was deleted.
- **How:** `len()` is the number of items in the `Vec`.
- **Remove it and…** (`before`) there's nothing to compare with below.

`store.todos.retain(|t| t.id != id);`
- **What:** keeps every todo whose id is **not** `id`. The one that matches is dropped.
- **Why:** "delete the todo with this id", in one line.
- **How:** **new:** `retain` ("keep") walks the `Vec` and calls the closure on each item, keeping
  those for which it returns `true`. The list closes up behind the removed one, so the order of the
  rest is kept.
- **Remove it and…** nothing is ever deleted.

`if store.todos.len() == before {` · `return Err(AppError::NotFound(id));` · `}`
- **What:** if the count didn't change, nothing matched: `404`.
- **Why:** deleting a todo that isn't there is an error the client should hear about, as in the
  second `DELETE` of Step 4.
- **How:** `return` leaves the handler early with the error.
- **Remove it and…** every `DELETE` answers `204`, even for a todo that never existed.

`Ok(StatusCode::NO_CONTENT)` · `}`
- **What:** `204 No Content`: done, and nothing to send.
- **Why:** the usual answer to a successful `DELETE`.
- **How:** a `StatusCode` on its own is a complete response, with an empty body.
- **Remove it and…** the function returns nothing, and won't compile.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/todos.rs:router}}
```

### Line by line

`pub fn router() -> Router {`
- **What:** the todos area's one door: its own Router, ready to be nested.
- **Why:** `lib.rs` calls it, from another module, so it must be `pub`.
- **How:** it returns a plain `Router` (a `Router<()>`), because it fills in its own state below.
- **Remove it and…** (`pub`) `lib.rs` can't call it:

  ```text
  error[E0603]: function `router` is private
     --> projects/a2-build-todo-api/src/lib.rs:11:36
      |
   11 |         .nest("/api/todos", todos::router())
      |                                    ^^^^^^ private function
      |
  note: the function `router` is defined here
     --> projects/a2-build-todo-api/src/todos.rs:123:1
      |
  123 | fn router() -> Router {
      | ^^^^^^^^^^^^^^^^^^^^^
  ```

`Router::new()` · `.route("/", get(list).post(create))`
- **What:** the list path: `GET` lists, `POST` creates.
- **Why:** two of the five endpoints share a path.
- **How:** `"/"` is relative to the prefix, so it becomes `/api/todos`.
- **Remove it and…** `GET /api/todos` answers `404`.

`.route("/{id}", get(show).patch(update).delete(remove))`
- **What:** one todo: `GET` shows, `PATCH` changes, `DELETE` deletes.
- **Why:** the other three endpoints share a path.
- **How:** `{id}` captures the number for `Path`. Any other method (such as `PUT`) answers `405`,
  as [Routes and HTTP methods](routes-and-methods.md) showed; `validation_and_rejections` checks
  it.
- **Remove it and…** (`.patch(update)`) `PATCH` answers `405 Method Not Allowed`.

`.with_state(AppState::default())` · `}`
- **What:** gives the todos' Router its state: a brand-new, empty store.
- **Why:** each call to `router()`, so each call to `app()`, starts with no todos and `next_id` 0.
  That's what lets every test start from scratch.
- **How:** `.with_state` turns a Router that **needs** an `AppState` into one that has it, so it can
  be nested into an `app()` that knows nothing about stores.
- **Remove it and…** ``mismatched types``: the function promised a ready `Router`, and this one
  still waits for its state.

### `src/main.rs`

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/src/main.rs:main}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api`

### Line by line

`#[tokio::main]` · `async fn main() {` · `let listener = …bind("127.0.0.1:3000")…;` · `println!(…);`
- **What:** start the runtime, claim port 3000, print one line.
- **Why:** the same start as every Axum program in this book.
- **How:** [Hello, Axum, Step 6](hello-axum.md#step-6-main) explains every line.
- **Remove it and…** (`#[tokio::main]`) ``error[E0752]: `main` function is not allowed to be `async` ``.

`axum::serve(listener, a2_build_todo_api::app())` · `.await` · `.expect(…);` · `}`
- **What:** serve the library's app.
- **Why:** the one line that names the library: `a2_build_todo_api`, the package name with `_` for
  `-`. In your project, it's `todo_api::app()`.
- **How:** as in [Testing handlers, Step 4](testing-handlers.md#step-4-the-program-srcmainrs).
- **Remove it and…** (writing `app()` alone) ``cannot find function `app` in this scope``.

### `tests/api.rs`

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/tests/api.rs:helpers}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo test -p a2-build-todo-api`

### Line by line

`use a2_build_todo_api::app;` · `use axum::{Router, body::Body, http::{Request, StatusCode}};` · `use http_body_util::BodyExt;` · `use tower::ServiceExt;`
- **What:** the app, and the tools from [Testing handlers, Step 5](testing-handlers.md#step-5-the-test-helpers).
- **Why:** `app` is the only item the tests take from the library; everything else is reached over
  HTTP, as a client would.
- **How:** `BodyExt` gives `.collect()`, `ServiceExt` gives `.oneshot()`.
- **Remove it and…** (`ServiceExt`) ``no method named `oneshot` found``.

`async fn send(app: Router, method: &str, uri: &str, body: Option<&str>) -> (StatusCode, String) {`
- **What:** one helper for every request in this file: give it the app, a method such as
  `"PATCH"`, a path, and maybe a JSON body; get back the status and the body as text.
- **Why:** this API has five methods, some with a body and some without. One helper that takes the
  method as text keeps every check to a single call.
- **How:** `Option<&str>`: `Some(r#"{"done":true}"#)` for a body, `None` for none.
- **Remove it and…** each test builds every request by hand, a dozen lines each.

`let mut builder = Request::builder().method(method).uri(uri);`
- **What:** starts building a request with this method and path.
- **Why:** the header and the body come next, and only some requests have them.
- **How:** `Request::builder()` makes a *builder*, a value you add parts to step by step. `mut`
  because the next lines may replace it.
- **Remove it and…** (`mut`) `` cannot assign twice to immutable variable `builder` ``.

`let body = match body {` · `Some(json) => {` · `builder = builder.header("content-type", "application/json");` · `Body::from(json.to_owned())` · `}` · `None => Body::empty(),` · `};`
- **What:** with a JSON body, add the `content-type` header and use the JSON as the body; without
  one, an empty body.
- **Why:** `Json` refuses a body without `content-type: application/json` (the `415` from JSON with
  serde), so the header must come with every JSON body.
- **How:** a `match` that also gives back a value: whichever arm runs, its last line becomes `body`.
  `.header(…)` gives back a new builder, which replaces the old one. `json.to_owned()` turns the
  `&str` into an owned `String`, which a `Body` can keep.
- **Remove it and…** (the `.header` line) every `POST` and `PATCH` answers `415`.

`let response = app.oneshot(builder.body(body).unwrap()).await.unwrap();` · `let status = …;` · `let bytes = …;` · `(status, String::from_utf8(bytes.to_vec()).unwrap())` · `}`
- **What:** finish the request, send it through the app, read the answer.
- **Why:** the test bench from [Testing handlers](testing-handlers.md): no server, no port.
- **How:** `.body(body)` finishes the request (it fails only for a malformed method or path, so
  `.unwrap()`); the rest is Testing handlers' `send`, line for line.
- **Remove it and…** there's nothing to check.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/tests/api.rs:lifecycle}}
```

### Line by line

`#[tokio::test]` · `async fn full_lifecycle() {` · `let app = app();`
- **What:** a test that walks one todo through its whole life, in one app.
- **Why:** the steps depend on each other (you can only delete what you created), so they share one
  app, and one store.
- **How:** `app()` once; then `app.clone()` for each request, because `oneshot` uses up the Router
  it's given. Clones share the same `Arc`'d store, as in Testing handlers, Step 8.
- **Remove it and…** (sending a fresh `app()` each time) each request sees an empty store, and the
  `PATCH` answers `404`.

`assert_eq!(` · `send(app.clone(), "GET", "/api/todos", None).await,` · `(StatusCode::OK, "[]".into())` · `);`
- **What:** a new app starts with an empty list.
- **Why:** checks the starting point before anything changes.
- **How:** `send` gives back a `(StatusCode, String)`, so the expected value must be one too:
  `"[]".into()` turns the `&str` into a `String`, and Rust works out which type from the other side
  of the comparison.
- **Remove it and…** (`.into()`) ``mismatched types``: a `&str` and a `String` aren't the same type.

`let (s, b) = send(app.clone(), "POST", "/api/todos", Some(r#"{"title":"buy milk"}"#)).await;` · `assert_eq!((s, b.as_str()), (StatusCode::CREATED, r#"{"id":1,"title":"buy milk","done":false}"#));`
- **What:** create a todo; expect `201` and the exact JSON, with id 1.
- **Why:** checks rule 4 (`done` is `false`) and the first id.
- **How:** the other way to compare a `String` with a `&str`: `b.as_str()` borrows the `String` as a
  `&str`. Unpacking into `(s, b)` first gives each half a short name.
- **Remove it and…** a `create` that answered `200`, or forgot `done`, would go unnoticed.

`send(app.clone(), "PATCH", "/api/todos/1", Some(r#"{"done":true}"#))` · `(StatusCode::OK, r#"{"id":1,"title":"buy milk","done":true}"#)`
- **What:** tick it off; expect `done: true` **and** the title unchanged.
- **Why:** rule 3, checked on the whole body: a `PATCH` that wiped the title would fail here.
- **How:** as above.
- **Remove it and…** a broken partial update would go unnoticed.

`assert_eq!(send(app.clone(), "DELETE", "/api/todos/1", None).await.0, StatusCode::NO_CONTENT);` · `send(app.clone(), "GET", "/api/todos/1", None).await` · `(StatusCode::NOT_FOUND, r#"{"error":"todo 1 not found"}"#.into())`
- **What:** delete it (`204`), then ask for it: `404` with the JSON message.
- **Why:** checks `remove`, `show`'s error path, and rule 5's exact message.
- **How:** `.0` takes the first half of the pair, the status, when the body doesn't matter.
- **Remove it and…** a `DELETE` that did nothing would go unnoticed.

`let (s, _) = send(…, "POST", "/api/todos", Some(r#"{"title":"again"}"#)).await;` · `assert_eq!(s, StatusCode::CREATED);` · `assert_eq!(send(app, "GET", "/api/todos/2", None).await.0, StatusCode::OK);` · `}`
- **What:** create another todo, then check that it's number **2**.
- **Why:** rule 2. The list is empty again, so "count plus one" would call it 1, and
  `GET /api/todos/2` would answer `404`. This line is the test that catches Step 3's bug.
- **How:** `_` means "ignore the body". The last call uses `app` itself, not a clone: nothing comes
  after it.
- **Remove it and…** ids could be reused, and no test would notice.

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/tests/api.rs:validation}}
```

### Line by line

`let (s, b) = send(app(), "POST", "/api/todos", Some(r#"{"title":""}"#)).await;` · `assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);` · `assert!(b.contains("\"title\""));`
- **What:** an empty title gets `422`, and the answer mentions `title`.
- **Why:** rule 1. The test checks the field's name, not validator's whole answer, so a later
  validator version that words its answer differently doesn't break it.
- **How:** each check here needs no earlier state, so each uses its own fresh `app()`. `"\"title\""`
  is the text `"title"`, quotes included; `\"` is a quote inside a normal string.
- **Remove it and…** an API that accepted empty titles would pass.

`send(app(), "DELETE", "/api/todos/9", None).await.0` · `StatusCode::NOT_FOUND`
- **What:** deleting a todo that doesn't exist is `404`.
- **Why:** checks `remove`'s error path.
- **How:** a fresh app has no todo 9.
- **Remove it and…** a `remove` that always said `204` would pass.

`send(app(), "PUT", "/api/todos/1", None).await.0` · `StatusCode::METHOD_NOT_ALLOWED`
- **What:** `PUT` isn't one of this path's methods: `405`.
- **Why:** the spec has `PATCH`, not `PUT`. This checks that nobody adds one by accident.
- **How:** Axum answers `405` by itself.
- **Remove it and…** nothing breaks; one fewer promise is checked.

`send(app(), "GET", "/health", None).await` · `(StatusCode::OK, "ok".into())` · `}`
- **What:** the health check still answers `ok`, outside `/api`.
- **Why:** checks that `lib.rs`'s own route survived the move to modules.
- **How:** as in the first check of `full_lifecycle`.
- **Remove it and…** a lost `/health` would go unnoticed until monitoring complained.

## Stretch goals

Each goal adds one feature to the finished API. The book's solutions are full programs in
`examples/`, built from Step 5 with the todo routes nested at `/api/todos`; each has a test at the
bottom. Try them in your own `todo-api`, in `src/todos.rs`.

### Only the finished ones: `?done=true`

`GET /api/todos?done=true` lists only the finished todos, and `?done=false` only the open ones.
Without `?done`, the list is complete, as before. A value that isn't `true` or `false` is refused.

<details><summary>Hint</summary>

A [query string](../glossary.md#query-string) struct, as in
[Path and Query extractors](path-and-query-extractors.md), with one `Option<bool>` field: `None`
when there's no `?done`. Add `Query(query): Query<ListQuery>` to `list`, and keep only the todos that
match. Rust's `Option` has a method for "is it `None`, or does the value inside pass this test?":
`query.done.is_none_or(|done| t.done == done)`.

</details>

<details><summary>Solution</summary>

`code/projects/a2-build-todo-api/examples/todo-filter-done.rs` (only `list` changed, and `Query` was
added to the `use axum::{…}` line):

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/todo-filter-done.rs:list}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example todo-filter-done`

```bash
{{#include ../../code/projects/a2-build-todo-api/http/90-filter-done.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/90-filter-done.out}}
```

`.filter(…)` keeps the todos for which the closure says `true`, `.cloned()` copies them, and
`.collect()` gathers them into a new `Vec<Todo>`; Rust knows it's a `Vec<Todo>` because that's what
`Json(todos)` must send. `is_none_or` says `true` when there's no `?done` (so everything is kept),
and otherwise compares. `?done=maybe` never reaches the handler: `Query` can't read `maybe` as a
`bool`, and answers `400` with the reason. The URLs are in double quotes because `?` means
something special to your terminal's shell (*Path and Query extractors* explains).

</details>

### Clear the finished ones: `DELETE /api/todos?done=true`

A "clear completed" button: `DELETE /api/todos?done=true` deletes every finished todo at once, and
answers `200` with how many it removed, as `{"removed":1}`. `DELETE /api/todos` with no `?done` must
**not** delete everything: refuse it.

<details><summary>Hint</summary>

A new handler on the **list** path: `.route("/", get(list).post(create).delete(clear))`. Its query
struct has `done: bool`, **not** `Option<bool>`, so a missing `?done` is refused by `Query` with
`400` before your handler runs. `retain` keeps the todos whose `done` is **not** the one asked for;
the difference in `len()` is the count.

</details>

<details><summary>Solution</summary>

`code/projects/a2-build-todo-api/examples/todo-clear-done.rs` (a new handler, one more method on the
`"/"` route, and `Query` and `serde_json::Value` added to the `use` lines):

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/todo-clear-done.rs:clear}}
```

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/todo-clear-done.rs:route}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example todo-clear-done`

```bash
{{#include ../../code/projects/a2-build-todo-api/http/91-clear-done.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/91-clear-done.out}}
```

One todo was done, so one is removed, and the list keeps the other. The last request is the safety
net: with no `?done`, `Query` can't fill the `done: bool` field, so it answers `400`
``missing field `done` ``, and nothing is deleted. That's a deliberate choice: a typo that forgets
`?done=true` should never wipe the whole list. (`?done=false` would clear the **open** ones; the
rule is the same, the other way round.) The answer is `200` with a body, not `204`, because the
count is worth sending: the client can show "1 todo cleared". `Json<Value>` means "any JSON"; `json!`
builds it, as in `error.rs`.

</details>

### Newest first: `?order=newest`

`GET /api/todos?order=newest` lists the todos newest first, and `?order=oldest` (or no `?order`)
oldest first, as before. Any other value is refused.

<details><summary>Hint</summary>

The list is already in creation order, oldest first, because `create` always pushes at the end. So
"newest first" is the same list, reversed: `Vec` has a `.reverse()` method. For the query, an enum
with two variants, `Oldest` and `Newest`, read from lowercase text with
`#[serde(rename_all = "lowercase")]` (from [JSON with serde](json-and-serde.md)), and
`#[serde(default)]` on the field so a missing `?order` means `Oldest`.

</details>

<details><summary>Solution</summary>

`code/projects/a2-build-todo-api/examples/todo-newest-first.rs` (only `list` changed, and `Query`
was added to the `use axum::{…}` line):

```rust,noplayground
{{#include ../../code/projects/a2-build-todo-api/examples/todo-newest-first.rs:list}}
```

📁 Full code: code/projects/a2-build-todo-api · ▶ Run it: `cd code && cargo run -p a2-build-todo-api --example todo-newest-first`

```bash
{{#include ../../code/projects/a2-build-todo-api/http/92-newest-first.sh}}
```

```text
{{#include ../../code/projects/a2-build-todo-api/http/92-newest-first.out}}
```

`#[derive(Default)]` on an enum needs to know which variant is the default: `#[default]` marks
`Oldest`. `#[serde(default)]` on the field says "if `?order` is missing, use `Order::default()`".
`if let Order::Newest = query.order` is `if let` with an enum variant: the block runs only for
`Newest`. Copy the list out first (`.clone()`, which also lets the lock go), then reverse the copy:
reversing the stored list would flip it for every later request too. `?order=random` isn't one of
the two names, so `Query` answers `400`, and its message lists the names it would accept.

</details>

## Remember this

- **Start from a spec:** the endpoints (method, path, body, success and error statuses) and the
  rules, written down before any code. Each rule becomes a line of code and a test.
- **Ids come from a counter that only goes up**, behind the same lock as the list. "Count plus
  one" reuses numbers after a delete.
- **Partial update = `Option` fields + `if let Some(…)`**, applied through `.iter_mut()` to the todo
  **in** the list. Delete with `retain`, and compare lengths to know whether anything matched.
- **One file per job:** `error.rs`, `validated.rs`, `todos.rs` behind `mod` lines in `lib.rs`;
  `pub` on exactly what a sibling module uses; `pub fn app()` as the only door out; `.nest("/api/todos", …)`
  to mount an area.
- **Test the whole lifecycle** through `app()` with `oneshot`, sharing one app with `.clone()`, and
  check the real server by hand with `curl -i`.

## Go deeper

- [Rust for Humans: Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
- [Rust for Humans: Modules and crates](https://open-source-bd.github.io/rustbook-for-human/language-basics/modules-and-crates.html)
- [axum::Router::nest](https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.nest) — Official reference: how a nested router's paths join the prefix, and what happens to its state.
- [Axum's todos example](https://github.com/tokio-rs/axum/tree/main/examples/todos) — Official example: another in-memory todo API, with pagination and a request timeout.

<!-- next:start -->

**Next:**

- [Cheat sheet: Axum](../a2-axum/cheatsheet-axum.md)
- What is an ORM? (coming soon)

<!-- next:end -->
