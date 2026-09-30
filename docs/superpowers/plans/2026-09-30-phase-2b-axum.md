# Phase 2b — CI-checked HTTP transcripts and Part A2 (Axum): Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish Part A2 of "Rust Backend for Humans": 12 Axum lessons, the guided "Build it: a Todo API" project, and the Axum cheat sheet. Every Rust listing is compiled and tested in CI, and every `curl` transcript shown to readers is re-run by CI against the real server.

**Architecture:**
- Each lesson has one crate in `code/topics/<slug>/`. The lesson `{{#include}}`s the crate's anchored code.
- Each lesson's `curl` sessions live in `code/topics/<slug>/http/NN-name.sh`, next to their real output in `NN-name.out`.
- A new runner, `tools/http-check.mjs`, reproduces those outputs. It is the HTTP twin of `sql-check.mjs`: it starts the lesson's server, runs each command, drops the always-changing `date:` header, and compares the result with the committed `.out`.
- The project lives in `code/projects/a2-build-todo-api/`.

**Tech Stack:**
- axum 0.8.9, tokio 1.53.1, tower 0.5.3, tower-http 0.7.1, serde 1.0.229, serde_json 1.0.151
- thiserror 2.0.21, validator 0.21.0, tracing-subscriber 0.3.23, http-body-util 0.1.5
- Node 24, mdBook 0.5.4

**Spec:** `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md`. Phase 2 is A1 + A2; 2a shipped A1. This plan is **2b**.

**Reference bundle (verified code):** `docs/superpowers/plans/2026-09-30-phase-2b-reference/`

Everything in the bundle was built on 2026-09-30 in a scratch workspace against the pinned versions, and passes:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets`

`http-check.mjs` was run for real against two of the crates, twice, with identical output.

When a task says "copy X from the bundle", copy the file byte for byte. Only then add what the task asks for (examples, exercises, http scripts).

**Read first:** `CLAUDE.md` (hard rules, publishing, SQL listings), and the published lessons `src/part-0-start/tour-of-the-stack.md` and `src/a1-postgres/crud-in-sql.md` for house style.

## Global Constraints

- Everything in `CLAUDE.md`'s "Hard rules" and "Global Constraints" applies unchanged:
  - the 11 lesson headings in exact order
  - three tiers, each with a `<details>` solution
  - `### Line by line` after **every** listing in "The idea, slowly", including diagrams
  - `### Run it` with ✅/❌
  - no banned words
  - glossary: "First used in" points at the earliest page, and each term is linked once, at its first use
  - ≥4 quiz questions
  - the Co-Authored-By trailer
- **Rust listings:**
  - Every listing is `{{#include ../../code/topics/<slug>/src/<file>.rs:<anchor>}}` in a ` ```rust,noplayground ` fence, followed by `📁 Full code: code/topics/<slug>` and the run command `cd code && cargo run -p <slug>`.
  - More examples and Your-turn solutions are full programs in `code/topics/<slug>/examples/<name>.rs`, with anchors and a `#[cfg(test)]` test. Run them with `cargo run -p <slug> --example <name>`.
  - Deliberately broken code goes in ` ```rust,noplayground,ignore `, next to the real compiler error. Produce that error in a scratch copy, never in the committed crate.
- **HTTP transcripts:**
  - Every Run-it that talks to a running server is an `http/NN-name.sh` file. Its first line is `# serve: -p <package> [--example <name>]`, and every other non-comment line is one command the reader types (usually `curl -i …`).
  - Pages show the script with ` ```bash ` + `{{#include ../../code/topics/<slug>/http/NN-name.sh}}`, and the result with ` ```text ` + `{{#include ../../code/topics/<slug>/http/NN-name.out}}`.
  - `.out` files come only from `node tools/http-check.mjs --update <package>`. Never hand-edit them.
  - The runner drops the `date:` response header, the only line that changes every second. The first lesson that shows a transcript (hello-axum) tells the reader that their own output will also have a `date:` line.
- **Numbering:** the same NN ranges as SQL: `01`–`49` The idea, slowly; `50`–`69` More examples; `70`–`79` Common mistakes; `90`–`99` Your turn.
- **Port:** every lesson server binds `127.0.0.1:3000`. The runner refuses to start if 3000 is already taken.
- **JSON requests** in scripts use `curl -i -X POST http://127.0.0.1:3000/<path> -H 'content-type: application/json' -d '<json>'`.
- **Logs:** server-side log lines (Middleware lesson) cannot come from the runner. Hand-capture them for real, following the allowed-output rule in CLAUDE.md.
- **Glossary:** RFH (Rust for Humans) already teaches the Rust language. On first use of `Arc`, `Mutex`, traits, generics, `?`, closures, modules and tests, link the matching RFH lesson instead of re-teaching it:
  - `runtime-and-ecosystem/shared-state-mutex-and-arc.html`
  - `abstractions/traits-basics.html`
  - `abstractions/generics.html`
  - `abstractions/the-question-mark-operator.html`
  - `abstractions/closures.html`
  - `language-basics/modules-and-crates.html`
  - `runtime-and-ecosystem/unit-testing.html`
  - `runtime-and-ecosystem/integration-testing.html`
  - `runtime-and-ecosystem/serde-and-json.html`
  - `abstractions/custom-error-types.html`
  - `abstractions/error-crates-thiserror-and-anyhow.html`
  - `runtime-and-ecosystem/logging-and-tracing.html`

## Review Focus

1. **Transcript drift.** A handler's text changes but its `.out` doesn't. `http-check.mjs` must exit 1 with a line diff (test in Task 2 via the smoke crate).
2. **Nondeterministic transcripts.** Dates, curl progress meters and timings must never reach a `.out`. The runner strips `date:`, gives curl a private `.curlrc` with `no-progress-meter`, and `--check-twice` catches anything else. Task 2 runs `--check-twice` on the smoke crate.
3. **A stale or foreign server on port 3000.** The runner must refuse to start rather than test the wrong server, and must free the port after every script, even when a command fails. Test in Task 2: start a listener on 3000, and the runner reports the port-in-use failure.
4. **Rust code shown on the page but not compiled.** It is already blocked by the validator's hand-typed-Rust rule. Each lesson's reviewer confirms that every `rust,noplayground` fence is an include.
5. **A reader copying a lesson crate into a fresh `cargo new` project.** They must see which `Cargo.toml` lines and features they need. Every lesson shows its `Cargo.toml` (with `.workspace = true` explained once, in hello-axum) and the equivalent `cargo add` command.

---

## File structure

| Path | Responsibility |
|---|---|
| `tools/sql.mjs` | `planRun(entries, ext)` gains an extension parameter and friendly errors (carry-over) |
| `tools/lib.mjs` | Windows-safe `code/sql` include-root check (carry-over) |
| `tools/http.mjs`, `tools/http-check.mjs` | HTTP transcript runner (copy from the bundle, then add tests) |
| `tools/test/http.test.mjs` | Unit tests for `http.mjs` |
| `code/Cargo.toml` | Workspace: members `topics/*` + `projects/*`, and the new workspace dependencies |
| `code/topics/<slug>/` | One crate per A2 lesson (from the bundle), plus `examples/` and `http/` |
| `code/projects/a2-build-todo-api/` | The project crate (from the bundle), plus step examples and `http/` |
| `code/topics/cheatsheet-axum/` | A compiled crate holding every cheat-sheet pattern |
| `.github/workflows/code.yml` | The `rust` job also runs `node tools/http-check.mjs --check-twice` |
| `src/a2-axum/*.md`, `questions/*.json`, `src/glossary.md`, `tools/topics.data.js` | Pages |

---

### Task 1: Carry-over fixes and `planRun(entries, ext)`

**Files:** Modify `tools/sql.mjs`, `tools/sql-check.mjs`, `tools/lib.mjs`, `tools/test/sql.test.mjs`, `tools/test/lib.test.mjs`

**Interfaces:**
- Produces: `planRun(entries: string[], ext = ".sql") -> { sql: string[], orphans: string[] }`.
  - `sql` holds the files ending in `ext`, sorted.
  - `orphans` holds `.out` files with no matching `ext` file.
  - It throws `Error("<file>: listing files must start with a two-digit NN- prefix")`.
  - Task 2 calls `planRun(entries, ".sh")`.

- [ ] **Step 1: Write the failing tests.** Append to `tools/test/sql.test.mjs`:

```js
test("planRun works for any listing extension", () => {
  const r = planRun(["02-b.sh", "01-a.sh", "01-a.out", "07-old.out", "notes.md"], ".sh");
  assert.deepEqual(r.sql, ["01-a.sh", "02-b.sh"]);
  assert.deepEqual(r.orphans, ["07-old.out"]);
  assert.throws(() => planRun(["curl.sh"], ".sh"), /two-digit NN- prefix/);
});
```

Append to `tools/test/lib.test.mjs` (uses the existing `good`/`ctx` helpers):

```js
test("carry: sql include root check uses path segments, not '/' strings", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/sqlx/hello/01-a.sql": "SELECT 1;\n" };
  const md = good().replace("### Line by line", "```sql\n{{#include ../../code/sqlx/hello/01-a.sql}}\n```\n\n### Line by line");
  assert.ok(checkPage(ctx(md, files)).errors.some((e) => e.includes("code/sql/")), "code/sqlx must not pass as code/sql");
});
```

- [ ] **Step 2: Run and see them fail.** Run `npm test`. Expected:
  - The planRun test fails: `.sh` files are ignored, and the message says "SQL files".
  - The root test fails if the check is a plain `startsWith(".../code/sql")` string test. If it already passes, keep the test and say so in the report.

- [ ] **Step 3: Implement.**
  1. In `tools/sql.mjs`, give `planRun` an `ext = ".sql"` parameter. Filter by `ext`, slice `ext.length`, and use the neutral message `listing files must start with a two-digit NN- prefix`.
  2. In `tools/sql-check.mjs`, wrap the `planRun` call in the same try/catch style as `dbNameFor`. Print `FAIL  <slug>: <message>` and count it as a failure, never a stack trace.
  3. In `tools/lib.mjs`, make the SQL include-root check segment-safe: `const rel = path.relative(join(root, "code", "sql"), resolved)`. The include passes only if `rel && !rel.startsWith("..") && !path.isAbsolute(rel)`.

- [ ] **Step 4: Run and see them pass.** Run `npm test`, then `node tools/validate.mjs` (0/0). With a scratch Postgres (see "Local Postgres" in the Task 3–14 shared rules), run `node tools/sql-check.mjs` and expect "all SQL outputs match".

- [ ] **Step 5: Commit** `fix(tools): planRun takes an extension; friendly NN- error; path-safe sql include root`.

---

### Task 2: HTTP transcript runner, workspace dependencies, CI, docs

**Files:**
- Create: `tools/http.mjs`, `tools/http-check.mjs` (copy both from the bundle's `tools/`), `tools/test/http.test.mjs`
- Modify: `code/Cargo.toml`, `package.json`, `.github/workflows/code.yml`, `CLAUDE.md`, the spec (§4 "Code ↔ book link"), `README.md`

**Interfaces:**
- Consumes: `planRun(entries, ".sh")`, `normalizeOutput`, `diffLines` from `tools/sql.mjs` (Task 1).
- Produces:
  - `parseScript(text) -> { serve: string[], commands: string[] }`. It throws on a missing `# serve:` header, or on a header with no `-p`.
  - `stripVolatile(text) -> string`. It removes every `date: …` line, case-insensitive, and normalizes CRLF.
  - `transcript(commands: string[], outputs: string[]) -> string`. It returns `"$ <cmd>\n<output>\n"` blocks joined by a blank line.
  - CLI: `node tools/http-check.mjs [--update] [--check-twice] [package …]`.
  - npm script: `"http": "node tools/http-check.mjs"`.

- [ ] **Step 1: Write the failing tests.** Create `tools/test/http.test.mjs`:

```js
import { test } from "node:test";
import assert from "node:assert/strict";
import { parseScript, stripVolatile, transcript } from "../http.mjs";

test("parseScript reads the serve header and the commands", () => {
  const s = parseScript("# serve: -p hello-axum --example about\n# a comment\n\ncurl -i http://127.0.0.1:3000/\ncurl -i http://127.0.0.1:3000/about\n");
  assert.deepEqual(s.serve, ["-p", "hello-axum", "--example", "about"]);
  assert.deepEqual(s.commands, ["curl -i http://127.0.0.1:3000/", "curl -i http://127.0.0.1:3000/about"]);
});

test("parseScript rejects a script without a usable serve header", () => {
  assert.throws(() => parseScript("curl -i http://127.0.0.1:3000/\n"), /serve/);
  assert.throws(() => parseScript("# serve: --example x\ncurl x\n"), /-p/);
});

test("stripVolatile drops only the date header", () => {
  const raw = "HTTP/1.1 200 OK\r\ncontent-length: 2\r\ndate: Wed, 30 Sep 2026 15:38:54 GMT\r\nDate: again\r\n\r\nok";
  assert.equal(stripVolatile(raw), "HTTP/1.1 200 OK\ncontent-length: 2\n\nok");
});

test("transcript prefixes each command with $ and separates blocks", () => {
  assert.equal(transcript(["curl a", "curl b"], ["A", "B\n"]), "$ curl a\nA\n\n$ curl b\nB\n");
});
```

- [ ] **Step 2: Run and see them fail.** Run `npm test`. Expected: `Cannot find module '…/tools/http.mjs'`.

- [ ] **Step 3: Copy the runner.** Copy `tools/http.mjs` and `tools/http-check.mjs` from the bundle. Its `planRun(…, ".sh")` call needs Task 1. Run `npm test`: all pass.

- [ ] **Step 4: Update the workspace.** Replace `code/Cargo.toml` with the bundle's `workspace-Cargo.toml`. It keeps `tour-of-the-stack` working, adds `members = ["topics/*", "projects/*"]`, adds tokio's `time` feature, and adds serde, serde_json, thiserror, tower-http, tracing, tracing-subscriber and validator. Then run `cd code && cargo test --workspace --all-targets` (the existing tour crate stays green). Commit `Cargo.lock`.

- [ ] **Step 5: Try it for real.** Add a throwaway crate `code/topics/zz-smoke/` (copy the bundle's `topics/hello-axum/`, rename the package to `zz-smoke`) with:
  - `http/01-first.sh`: `# serve: -p zz-smoke`, then `curl -i http://127.0.0.1:3000/about`.
  - `http/02-twice.sh`: `# serve: -p zz-smoke`, then `curl -i http://127.0.0.1:3000/`.

  Then run and record each result:

```bash
node tools/http-check.mjs zz-smoke; echo "exit=$?"              # expected: FAIL missing .out, exit=1
node tools/http-check.mjs --update zz-smoke && cat code/topics/zz-smoke/http/01-first.out
node tools/http-check.mjs --check-twice zz-smoke; echo "exit=$?" # expected: ok ×2, exit=0
# drift: edit zz-smoke's about() text, then:
node tools/http-check.mjs zz-smoke; echo "exit=$?"              # expected: FAIL output changed + diff, exit=1
# port guard: in another shell `python3 -m http.server 3000 --bind 127.0.0.1`, then:
node tools/http-check.mjs zz-smoke; echo "exit=$?"              # expected: FAIL … port 3000 is already in use, exit=1
```

  Expected `01-first.out`:

```text
$ curl -i http://127.0.0.1:3000/about
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 31

This server is written in Rust.
```

  Afterwards, stop the python listener, delete `code/topics/zz-smoke`, and remove its lines from `Cargo.lock` (`cargo metadata` regenerates them). Record every command and its output in the report.

- [ ] **Step 6: Add CI.** In `.github/workflows/code.yml`, add these steps to the `rust` job after `cargo test`:

```yaml
      - uses: actions/setup-node@v7
        with:
          node-version: "24"
      - name: Every curl transcript on the site matches the real server (twice)
        working-directory: .
        run: node tools/http-check.mjs --check-twice
```

  Also add `"http": "node tools/http-check.mjs"` to `package.json`. Validate the YAML with `ruby -ryaml -e 'YAML.load_file(".github/workflows/code.yml")'`.

- [ ] **Step 7: Document it.** Add a `## HTTP transcripts` section to CLAUDE.md. It covers:
  - the script format and the `# serve:` header
  - the NN ranges
  - port 3000 and the refusal to run when the port is busy
  - `date:` stripping and the private `.curlrc`
  - `--update` and `--check-twice` (each script gets a fresh server, twice)
  - how pages include the `.sh` in ` ```bash ` and the `.out` in ` ```text `
  - the rule that `.out` is never hand-edited
  - the Rust-listing rules in Global Constraints above (`rust,noplayground`, examples/, the `📁 Full code` line)

  Add one paragraph to spec §4 and one line to the README dev loop.

- [ ] **Step 8: Verify and commit.** Run `npm test && node tools/validate.mjs`. Run `cd code && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`. Run `node tools/http-check.mjs`, which prints "all HTTP outputs match" with no scripts yet. Commit `feat(tools): CI-checked HTTP transcripts — http-check runner, workspace deps, CI step`.

---

### Tasks 3–14: A2 lessons — shared rules

Each lesson is one task and one commit (`docs(a2): <slug> lesson`). For every lesson:

1. **Topic entry.** In `tools/topics.data.js`, replace its `draft(…)` with a full entry: `status: "published"`, plus the `outcomes`, `summary`, `prereq`, `next`, `level`, `rfhLinks` and `links` from the brief, and `codeDir: "code/topics/<slug>"`. Run `node tools/generate.mjs`.
2. **Crate.** Copy `topics/<slug>/` from the reference bundle **byte for byte**: `Cargo.toml`, `src/`, and `tests/` if present. Then add:
   - `examples/*.rs` for More examples and Your-turn solutions. Each is a full program with anchors and at least one `#[cfg(test)]` test.
   - `http/NN-name.sh` for every Run it the brief lists, plus one per example or solution that runs a server.

   Then run `cd code && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets`, then `node tools/http-check.mjs --update <slug>`, then `node tools/http-check.mjs --check-twice <slug>`. Read every `.out`. If it disagrees with what the prose will say, fix the prose, or fix the code and re-run.
3. **Page.** Write `src/a2-axum/<slug>.md` (see Global Constraints):
   - Show each crate's `Cargo.toml` once, with a Line by line entry per dependency line, and give the equivalent `cargo add` command.
   - "Coming from another language?" compares with Express (Node), Flask/FastAPI (Python), Spring Boot (Java) and Go `net/http`/Gin. These snippets are illustrative prose-code and must be accurate.
   - Common mistakes show real compiler errors (from a scratch copy) or real `.out` transcripts.
4. **Glossary.** Add each new term the brief lists in A–Z order, with "First used in" pointing at the earliest page that uses it; grep `src/` first. Link it once, at its first use.
5. **Quiz.** Write `questions/<slug>.json` with exactly the brief's quiz and flashcards. Run `node tools/generate.mjs`.
6. **Verify.** All of these must pass:
   - `npm test`
   - `node tools/validate.mjs` (0 errors, 0 warnings)
   - `mdbook build 2>&1 | grep -E ' (ERROR|WARN) '` (no output)
   - `node tools/http-check.mjs --check-twice <slug>`
   - the cargo chain above

   Then do a confused-beginner read-through.
7. **Commit** the page, crate, examples, `http/` files, quiz, `topics.data.js`, glossary, `Cargo.lock`, and every page whose Next block `generate.mjs` rewrote.

**Next chain (set `next` accordingly):**
`cheatsheet-sql → hello-axum → routes-and-methods → handlers-and-into-response → path-and-query-extractors → json-and-serde → shared-state → error-handling-in-axum → middleware-and-tower-layers → nesting-and-modular-routers → custom-extractors → input-validation → testing-handlers → a2-build-todo-api → cheatsheet-axum → what-is-an-orm`
- Each lesson's `next` is the following item.
- `testing-handlers` also lists `cheatsheet-axum`.
- `what-is-an-orm` stays a draft, so it shows "(coming soon)".

**Local Postgres (for Task 1 only):** port 5433 belongs to the unrelated container `inkwell-db-1`; never touch it. Run a scratch copy of `docker-compose.yml` on port 55433 (`-p rbh-sql`) and export `RBH_PSQL` as documented in CLAUDE.md. Tear it down after Task 1.

---

### Task 3: `hello-axum` (Beginner)

- **Bundle:** `topics/hello-axum/`. Anchors: `imports`, `handlers`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can start a new Axum project with cargo and add the right dependencies.", "You can serve an HTML page and a plain-text page.", "You can read every line of a raw HTTP response from your own server."
  - summary: "Build your own Axum server from an empty folder: dependencies, handlers, routes, and a first look at real responses."
  - prereq `["cheatsheet-sql"]`; next `["routes-and-methods"]`
  - rfhLinks: cargo-basics `start-here/cargo-basics.html`, async-basics
  - links: axum docs `https://docs.rs/axum/0.8.9/axum/`, `https://docs.rs/axum/0.8.9/axum/response/struct.Html.html`
- **Brief:**
  - *What & why:* tour-of-the-stack showed a server you downloaded; now you build one yourself. Analogy: the tour was a test drive; this is getting your licence.
  - *The idea, slowly:*
    1. `cargo new my-first-server` and `cargo add axum tokio --features tokio/macros,tokio/rt-multi-thread,tokio/net`, shown as ` ```bash `. Show the resulting dependency lines. Explain that the book's crate says `.workspace = true` because its versions live in `code/Cargo.toml`, and that this is the same versions in one shared place. This is hand-captured real output in a scratch folder.
    2. The `Cargo.toml` include.
    3. `imports`: explain each `use` item.
    4. `handlers`: `Html`, why `&'static str`, and why two handlers.
    5. `app`: two `.route` calls.
    6. `main` (link back to tour for the parts already explained).
    7. `### Run it` with `http/01-first-requests.sh`:
       ```
       # serve: -p hello-axum
       curl -i http://127.0.0.1:3000/
       curl -i http://127.0.0.1:3000/about
       curl -i http://127.0.0.1:3000/contact
       ```
       Explain every response line. `content-type: text/html` versus `text/plain` shows the `Html` wrapper at work. The 404 has an empty body and no content-type. Explain the omitted `date:` line: "your terminal also shows `date: …`; the book hides it because it changes every second".
  - *You might be wondering…:* "Why does main block forever?"; "Why `127.0.0.1` and not `localhost`?"; "Do I need `cargo add` every time?"; "Why is `/contact` a 404 with an empty body?"
  - *Common mistakes:* `Html(...)` without `use axum::response::Html` (real compiler error); forgetting `.route` for a handler (real 404 transcript `70-forgot-route.sh` via an example that omits `/about`); running two servers at once (the real `expect` panic text).
  - *More examples:* each is an example file with a test and an http script:
    - a page with a link (`<a href="/about">`)
    - a `String` built with `format!`
    - a third route `/hello`
    - `Html(String)` built at runtime
  - *Your turn:* 🟢 change the home page heading; 🟡 add `/contact` returning your email as text; 🔴 add `/time` returning HTML with seconds since the Unix epoch. For 🔴, the test checks status and content-type only, and the transcript would not be deterministic, so there is no http script; show it in prose.
  - *Glossary terms:* `dependency`, `HTML`. `Cargo.toml` and `feature` can be explained inline.
  - *Quiz:*
    1. q: "What does `Html(\"<h1>Hi</h1>\")` change about the response?"; options: ["Nothing", "It sets content-type to text/html so the browser renders it", "It compresses the text", "It makes the server faster"]; answer 1; explain: "The Html wrapper tells Axum to send `content-type: text/html; charset=utf-8`, so a browser renders the tags instead of showing them."
    2. q: "Which command adds Axum to a new project?"; options: ["cargo install axum", "cargo add axum", "cargo new axum", "npm install axum"]; answer 1; explain: "`cargo add` writes the dependency line into Cargo.toml for you."
    3. q: "Your server returns 404 for `/contact`. Most likely cause?"; options: ["The port is wrong", "No `.route(\"/contact\", …)` was added", "Tokio is missing", "The handler is too slow"]; answer 1; explain: "Axum answers 404 for any path the Router has no rule for."
    4. q: "Why does the book's crate write `axum.workspace = true`?"; options: ["It's required by Axum", "The version is set once in code/Cargo.toml and shared by every lesson crate", "It disables features", "It downloads the latest version"]; answer 1; explain: "A Cargo workspace lets many crates share one list of dependency versions."
  - *Flashcards:* "Serve HTML from a handler?" → "Return `Html(\"…\")`."; "Add a dependency?" → "`cargo add <crate>` (optionally `--features …`)."

### Task 4: `routes-and-methods` (Beginner)

- **Bundle:** `topics/routes-and-methods/`. Anchors: `imports`, `handlers`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can send different HTTP methods on one path to different handlers.", "You know what 404 and 405 mean and when Axum sends each.", "You can give unknown paths your own fallback response."
  - summary: "One path, many methods: GET, POST, PUT and DELETE on the same URL, plus 404, 405 and a custom fallback."
  - prereq `["hello-axum"]`; next `["handlers-and-into-response"]`
  - links: `https://docs.rs/axum/0.8.9/axum/routing/index.html`, MDN HTTP methods `https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Methods`
- **Brief:**
  - *What & why:* a library desk with one counter and different forms: borrow, return, renew. The same place does different jobs depending on the form.
  - *The idea, slowly:* `handlers` (the five book handlers + `not_found`); `app` (method chaining `get(…).post(…)`); `{id}` as a placeholder that matches any segment (reading its value comes in *Path and Query extractors*); `.fallback`.
  - *Run it:* `http/01-methods.sh` runs `curl -i` GET /books, `-X POST` /books, GET /books/7, `-X PUT` /books/7, `-X DELETE` /books/7. `http/02-wrong-method.sh` runs `-X PATCH` /books (405, and explain the real `allow:` header) and GET /magazines (the fallback's 404 and body).
  - *You might be wondering…:* "Why not one handler with an if on the method?"; "What's the difference between 404 and 405?"; "Does `{id}` check that it's a number?" (no, not yet); "Can two routes share a handler?"
  - *Common mistakes:* using `:id` (the pre-0.8 Axum syntax). Show the real panic message when the app starts, captured from a scratch copy, as ` ```rust,noplayground,ignore ` plus the real panic. Also: two `.route` calls for the same path (the real "Overlapping method route" panic).
  - *More examples:* `any(…)`; `.route("/", get(…))` plus `head` behavior (GET routes answer HEAD automatically; show the transcript); a fallback returning HTML; `.post` only, so GET gives 405.
  - *Your turn:* 🟢 add `.patch(...)` to `/books/{id}`; 🟡 make the fallback say which path was missing (needs `Uri` extractor: `async fn not_found(uri: Uri)`). Introduce it in the solution with one sentence. 🔴 build `/authors` with GET and POST and `/authors/{id}` with GET and DELETE.
  - *Glossary:* `HTTP method` (grep: earlier pages say "method", so point to the earliest page). `fallback` goes inline.
  - *Quiz:*
    1. q: "`.route(\"/books\", get(list).post(add))` — what runs for `POST /books`?"; options: ["list", "add", "both", "neither"]; answer 1; explain: "Method chaining attaches a different handler per method on the same path."
    2. q: "Axum returns 405 when…"; options: ["the path doesn't exist", "the path exists but not for that method", "the server is down", "the body is invalid"]; answer 1; explain: "405 Method Not Allowed: the route exists, but nothing handles that verb. 404 means no route matched at all."
    3. q: "In Axum 0.8, a path placeholder is written…"; options: [":id", "{id}", "<id>", "*id"]; answer 1; explain: "Axum 0.8 uses braces. The old `:id` style now panics at startup."
    4. q: "What does `.fallback(not_found)` handle?"; options: ["Every request", "Requests whose path matched no route", "Only 500 errors", "Only POST requests"]; answer 1; explain: "The fallback runs when the path matches none of the routes."
  - *Flashcards:* "Same path, two methods?" → "`.route(\"/x\", get(a).post(b))`"; "404 vs 405?" → "404: no route for the path. 405: route exists, method not handled."

### Task 5: `handlers-and-into-response` (Beginner)

- **Bundle:** `topics/handlers-and-into-response/`. Anchors: `imports`, `simple`, `status`, `headers`, `by_hand`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You know which Rust values Axum can turn into a response and what each one sends.", "You can set the status code and headers of a response.", "You can build a response by hand when nothing else fits."
  - summary: "What a handler can return — text, HTML, status codes, headers, or a hand-built Response — and how `IntoResponse` makes it work."
  - prereq `["routes-and-methods"]`; next `["path-and-query-extractors"]`
  - rfhLinks: traits-basics
  - links: `https://docs.rs/axum/0.8.9/axum/response/index.html`
- **Brief:**
  - *What & why:* the kitchen hands the waiter a plate; the plate can be a bowl, a tray or a takeaway box, and the waiter knows how to carry each. `IntoResponse` is "knows how to become a response".
  - *The idea, slowly:* `simple` (&str vs String vs Html); `status` (tuple order: status first); `headers` (`impl IntoResponse`, array of header pairs); `by_hand` (the builder, `.into()` for the body, why `.unwrap()` is safe here but worth explaining).
  - *Run it:* `http/01-returns.sh` requests all six routes with `curl -i`. Line by line covers content-type per type, 201, `cache-control`, `x-note`, and 202.
  - *You might be wondering…:* "What is a trait?" (RFH link); "What does `impl IntoResponse` mean?"; "Why is status first in the tuple?"; "When would I build a Response by hand?"
  - *Common mistakes:*
    - Returning an `i32` (the real compiler error "the trait bound `i32: IntoResponse` is not satisfied" from a scratch copy).
    - Headers after the body in a tuple: `("text", StatusCode::CREATED)` gives a real error.
    - Two different return types from `if` branches without `.into_response()`: show the real "`if` and `else` have incompatible types" error, and the fix.
  - *More examples:* `Result<String, StatusCode>`, as a preview; `StatusCode` alone (204); a `Vec<u8>` body; a redirect with `Redirect::to("/text")` (show 303 and `location:`).
  - *Your turn:* 🟢 change `created` to 202; 🟡 add a `/teapot` route returning 418 with a body; 🔴 add `/download` returning text with `content-disposition: attachment; filename="notes.txt"`.
  - *Glossary:* `header` (grep first; how-a-web-backend-works may use it), `trait` → RFH link inline, `IntoResponse` inline.
  - *Quiz:*
    1. q: "Which return type makes the browser render HTML?"; options: ["&'static str", "String", "Html<&'static str>", "StatusCode"]; answer 2; explain: "Html sets content-type text/html; plain strings are sent as text/plain."
    2. q: "How do you return status 201 with a body?"; options: ["(\"made\", StatusCode::CREATED)", "(StatusCode::CREATED, \"made\")", "StatusCode::CREATED + \"made\"", "Created(\"made\")"]; answer 1; explain: "In a response tuple the status comes first, then headers, then the body."
    3. q: "What does `impl IntoResponse` as a return type mean?"; options: ["The handler returns nothing", "Some type Axum can turn into a response — you don't have to name it", "An HTTP header", "A compile error"]; answer 1; explain: "`impl Trait` says 'a value of some type that implements this trait'."
    4. q: "Why does returning an `i32` from a handler fail to compile?"; options: ["Numbers are too big", "i32 does not implement IntoResponse", "Handlers can't be async", "Axum only returns JSON"]; answer 1; explain: "Axum only accepts return types that implement IntoResponse, and a bare number doesn't: which content-type would it be?"
  - *Flashcards:* "Status + headers + body in one return?" → "`(StatusCode::OK, [(header::X, \"v\")], \"body\")`"; "Return 'no content'?" → "`StatusCode::NO_CONTENT`"

### Task 6: `path-and-query-extractors` (Beginner)

- **Bundle:** `topics/path-and-query-extractors/`. Anchors: `imports`, `path`, `query`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can read values from the URL path into typed Rust variables.", "You can read query-string options, including optional ones.", "You know what Axum answers when the URL can't be parsed."
  - summary: "Extractors pull data out of the request: `Path` for /books/42 and `Query` for ?q=rust&page=2 — typed, checked, and rejected with 400 when wrong."
  - prereq `["handlers-and-into-response"]`; next `["json-and-serde"]`
  - rfhLinks: serde-and-json, pattern-matching `language-basics/pattern-matching.html`
  - links: `https://docs.rs/axum/0.8.9/axum/extract/index.html`
- **Brief:**
  - *What & why:* a form at the counter: the clerk reads the book number off the form for you. Extractors are clerks.
  - *The idea, slowly:*
    - `path`: `Path(id): Path<u32>`, the destructuring pattern explained slowly (link RFH pattern matching); the tuple form for two values.
    - `query`: `#[derive(Deserialize)]`, `Option<u32>`, `unwrap_or`, and `{:?}` printing quotes.
    - `app`.
    - The `serde` Cargo.toml line, with `features = ["derive"]` explained.
  - *Run it:*
    - `http/01-path.sh`: /books/42, /books/3/chapters/7, /books/dune. The last is a real 400 body; explain it.
    - `http/02-query.sh`: /search?q=rust, ?q=rust&page=2, /search (400 missing field), `?q=hello%20world` (percent-encoding explained).
  - *You might be wondering…:* "Why wrap it in `Path(...)` and then unwrap it?"; "What if the number is negative?" (u32 rejects it; show it in More examples); "Path or query — which should I use?"; "Can I have both in one handler?" (yes; More examples).
  - *Common mistakes:*
    - A handler with `Path<(u32, u32)>` on a route with one placeholder (the real 500 transcript "Wrong number of path arguments…"). Capture via an example file.
    - Forgetting `#[derive(Deserialize)]` (the real compiler error).
    - A query field name that doesn't match (`?query=rust` gives the real 400 "missing field `q`").
  - *More examples:* `Path` + `Query` together; a `HashMap<String, String>` query; `Path<String>` for slugs; negative number rejection.
  - *Your turn:* 🟢 add `/authors/{name}` returning a greeting; 🟡 add `sort: Option<String>` to Search, defaulting to "title"; 🔴 `/range?from=1&to=5` returning the numbers joined by commas, and 400 if from > to. Use a `(StatusCode, String)` return.
  - *Glossary:* `extractor`, `query string` (grep: how-a-web-backend-works defines it, so point there), `deserialize`.
  - *Quiz:*
    1. q: "For `/books/{id}` with `Path(id): Path<u32>`, what happens on `/books/dune`?"; options: ["id is 0", "400 Bad Request — 'dune' isn't a u32", "404", "The server crashes"]; answer 1; explain: "The Path extractor fails to parse and Axum answers 400 before your handler runs."
    2. q: "How do you make a query parameter optional?"; options: ["Give it a default in the URL", "Make the field `Option<T>`", "Use Path instead", "It's always optional"]; answer 1; explain: "An `Option` field is None when the parameter is missing."
    3. q: "What does `#[derive(Deserialize)]` give your struct?"; options: ["A database table", "The ability to be built from query-string or JSON data", "A web route", "Faster code"]; answer 1; explain: "serde's Deserialize lets extractors like Query and Json fill your struct from the request."
    4. q: "What does `Path((book, chapter)): Path<(u32, u32)>` do?"; options: ["Reads two path placeholders in order", "Reads the query string", "Creates a route", "Nothing"]; answer 0; explain: "A tuple reads the placeholders left to right."
  - *Flashcards:* "Read /books/42?" → "`Path(id): Path<u32>` on route `/books/{id}`"; "Optional query param?" → "`#[derive(Deserialize)] struct P { page: Option<u32> }` + `Query(p): Query<P>`"

### Task 7: `json-and-serde` (Beginner)

- **Bundle:** `topics/json-and-serde/`. Anchors: `imports`, `types`, `handlers`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can send a Rust struct as JSON and receive JSON into a struct.", "You can make a JSON field optional with a default.", "You know the three ways a JSON request can be rejected (400, 415, 422)."
  - summary: "`Json<T>` in and out: serde turns structs into JSON and back, and Axum rejects bad bodies before your handler runs."
  - prereq `["path-and-query-extractors"]`; next `["shared-state"]`
  - rfhLinks: serde-and-json, derive-traits `language-basics/derive-traits.html`
  - links: `https://serde.rs/`, `https://docs.rs/axum/0.8.9/axum/struct.Json.html`
- **Brief:**
  - *What & why:* JSON (from how-a-web-backend-works) is the language apps speak. serde is the translator between Rust structs and JSON text.
  - *The idea, slowly:* `types` (Serialize vs Deserialize, and why separate `Book` and `NewBook`, i.e. the client doesn't choose the id; `#[serde(default)]`); `handlers` (`Json(...)` out, `Json(input)` in).
  - *Run it:*
    - `http/01-json-out.sh`: GET /books/sample. Explain `content-type: application/json`.
    - `http/02-json-in.sh`: POST with the full body, then without `in_stock` (the default shows false).
    - `http/03-rejections.sh`: missing field (422), broken JSON (400), no content-type header (415). Show the real bodies and explain the difference between 400, 415 and 422.
  - *You might be wondering…:* "Why two structs?"; "Why snake_case in JSON?" (and `#[serde(rename_all = "camelCase")]` in More examples); "Does field order matter?"; "What about extra fields?" (ignored by default; `deny_unknown_fields` in More examples).
  - *Common mistakes:* `Json` as the first of two extractors when a `Path` comes after it (the real compiler error: the body extractor must be last). Capture it in a scratch copy with `Path` + `Json` swapped. Also: Serialize on the input struct only (the real compiler error when returning it).
  - *More examples:* `rename_all = "camelCase"`; `deny_unknown_fields` (a real 422 transcript); a nested struct (an author inside a book); `Vec<Book>` as JSON.
  - *Your turn:* 🟢 add `pages: u32` to Book and NewBook; 🟡 make `in_stock` default to true with a function `#[serde(default = "yes")]`; 🔴 POST /orders accepting `{"items":[{"book_id":1,"qty":2}]}` and returning the total quantity as JSON `{"total_items": 2}`.
  - *Glossary:* `serialize`, `serde`. `deserialize` was added in Task 6, so only link it.
  - *Quiz:*
    1. q: "Which derive lets a struct be sent back as JSON?"; options: ["Deserialize", "Serialize", "Debug", "Clone"]; answer 1; explain: "Serialize turns a Rust value into JSON; Deserialize goes the other way."
    2. q: "A POST has valid JSON but is missing a required field. Axum answers…"; options: ["400", "415", "422", "500"]; answer 2; explain: "422 Unprocessable Entity: the JSON parsed, but it doesn't match your struct."
    3. q: "A client forgets `content-type: application/json`. Axum answers…"; options: ["200", "400", "415", "404"]; answer 2; explain: "415 Unsupported Media Type: the Json extractor only accepts JSON bodies."
    4. q: "Why use a separate `NewBook` input struct?"; options: ["serde requires it", "The client shouldn't send fields the server decides, like id", "It's faster", "JSON can't have ids"]; answer 1; explain: "Input and output often differ: the server assigns the id, so the input type leaves it out."
  - *Flashcards:* "Receive JSON?" → "`Json(input): Json<NewBook>` (last extractor)"; "400 vs 415 vs 422 for JSON?" → "400 broken JSON · 415 wrong content-type · 422 JSON doesn't fit the struct"

### Task 8: `shared-state` (Intermediate)

- **Bundle:** `topics/shared-state/`. Anchors: `imports`, `types`, `state`, `handlers`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can give every handler access to shared data with `State`.", "You know why the data is wrapped in `Arc<Mutex<…>>`.", "You know this memory is lost on restart — and why that's the database's job."
  - summary: "Share data between requests with `State<T>`, `Arc` and `Mutex`: a books list every handler can read and change."
  - prereq `["json-and-serde"]`; next `["error-handling-in-axum"]`
  - rfhLinks: shared-state-mutex-and-arc, smart-pointers `runtime-and-ecosystem/smart-pointers.html`
  - links: `https://docs.rs/axum/0.8.9/axum/extract/struct.State.html`
- **Brief:**
  - *What & why:* each request is served by a different worker at the same time; they need one shared whiteboard. `Arc` is "many hands can hold it" and `Mutex` is "one writer at a time".
  - *The idea, slowly:* `state` (`#[derive(Clone, Default)]` and why Clone is cheap for Arc); `handlers` (`State(state)`, `.lock().unwrap()`, `books.clone()` for the response, `len + 1` ids); `app` (`.with_state`).
  - *Run it:* `http/01-remembers.sh`: GET /books (`[]`); POST Dune; POST Emma; GET /books (both). Every script starts a fresh server, so the output is deterministic. Also include a hand-captured step: restart the server, then GET gives `[]`. It's real, captured, and the runner can't express a restart mid-script, so explain "memory dies with the process — the database (A3) fixes that".
  - *You might be wondering…:* "Why not a global variable?"; "What does `.unwrap()` on lock mean here?" (poisoning, one sentence; it's revisited in the errors lesson); "Is Mutex slow?"; "Why must AppState be Clone?"
  - *Common mistakes:* forgetting `.with_state` (the real compiler error `Router<AppState>` vs `Router<()>` from a scratch copy); `Rc` instead of `Arc` (the real `Send` error); holding the lock across `.await` (describe it plus the real compiler error "future cannot be sent between threads safely" from a scratch example with `tokio::time::sleep` while holding the guard).
  - *More examples:* a visit counter with `AtomicU64` (no Mutex needed); two state fields; `RwLock` for read-heavy data; state holding config (`app_name`).
  - *Your turn:* 🟢 add GET /books/count returning the number; 🟡 add DELETE /books (clear all, 204); 🔴 add `/likes/{id}` POST that increments a per-book like count stored in a `HashMap<u32, u32>` in state, and GET returning it.
  - *Glossary:* `state`; `Arc`/`Mutex` get RFH links, not glossary entries.
  - *Quiz:*
    1. q: "Why is the list wrapped in `Arc<Mutex<…>>`?"; options: ["To make it JSON", "Many requests share it (Arc) and only one changes it at a time (Mutex)", "To save it to disk", "Axum requires Vec to be wrapped"]; answer 1; explain: "Arc shares ownership across tasks; Mutex stops two writers colliding."
    2. q: "What happens to the books when you restart the server?"; options: ["They're saved", "They're gone — they lived only in memory", "They move to a file", "Axum reloads them"]; answer 1; explain: "State is ordinary memory. Keeping data across restarts is the database's job (Part A3)."
    3. q: "How does a handler get the state?"; options: ["A global variable", "The `State(state): State<AppState>` extractor", "A function argument named state", "It reads a file"]; answer 1; explain: "The State extractor hands each handler a clone of what you passed to `.with_state`."
    4. q: "Why must AppState implement Clone?"; options: ["Axum hands every request its own copy (a cheap Arc clone)", "To print it", "To serialize it", "It doesn't need to"]; answer 0; explain: "Each handler call gets a clone; cloning an Arc copies a pointer, not the data."
  - *Flashcards:* "Share data with handlers?" → "`Router::new()….with_state(state)` + `State(s): State<AppState>`"; "Arc vs Mutex?" → "Arc: many owners. Mutex: one at a time may change it."

### Task 9: `error-handling-in-axum` (Intermediate)

- **Bundle:** `topics/error-handling-in-axum/`. Anchors: `imports`, `types`, `error`, `into_response`, `handler`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can define one error type for your whole app.", "You can turn each error into the right status code and a JSON body.", "You can use `?` in handlers so errors flow out automatically."
  - summary: "One `AppError` enum, `IntoResponse` for it, and `?` in handlers: clean, consistent JSON errors instead of unwraps."
  - prereq `["shared-state"]`; next `["middleware-and-tower-layers"]`
  - rfhLinks: result-and-option, the-question-mark-operator, custom-error-types, error-crates-thiserror-and-anyhow
  - links: `https://docs.rs/thiserror/2.0.21/thiserror/`, `https://docs.rs/axum/0.8.9/axum/error_handling/index.html`
- **Brief:**
  - *What & why:* the waiter shouldn't shout "KITCHEN EXCEPTION LINE 42"; they should say "sorry, we're out of fish" (404) or "the kitchen is closed" (500). One error type lets every handler speak the same polite language.
  - *The idea, slowly:*
    - `error`: thiserror's `#[error("…")]` and `{0}`.
    - `into_response`: `match` to a status, the `json!` macro, the tuple.
    - `handler`: `Result<Json<Book>, AppError>`; `map_err` on the lock (what poisoning is, in one careful paragraph); `.iter().find().cloned().ok_or(...)`, each step explained; `?`.
    - The Cargo.toml lines for serde_json and thiserror.
  - *Run it:* `http/01-found-and-missing.sh`: GET /books/2 (200 JSON), GET /books/9 (404 with a JSON error), GET /books/abc (Axum's own 400 plain text). Point out that it isn't JSON; the fix is in More examples/exercises.
  - *You might be wondering…:* "Why not return `(StatusCode, String)` everywhere?"; "What is `thiserror` doing for me?"; "Should the error message be shown to users?" (be careful with 500 details; log them instead, see Middleware); "What about anyhow?"
  - *Common mistakes:* `.unwrap()` on a missing book (show the real 500 and the server-side panic line, hand-captured, using an example `70-unwrap-panics` whose transcript shows the 500); forgetting `impl IntoResponse for AppError` (the real compiler error); a wrong status mapping (every error 500 hides the 404).
  - *More examples:* an `AppError::BadInput(String)` variant for 422; `From<JsonRejection> for AppError`, so JSON errors become JSON too (show the real transcript); logging 500s with `eprintln!` before responding; an error with an `id` field in the JSON.
  - *Your turn:* 🟢 add `AppError::Gone(u32)` → 410 for book 13; 🟡 make the JSON include `"code": "not_found"`; 🔴 add DELETE /books/{id} returning 204, or 404 via AppError.
  - *Glossary:* `error type` inline; `enum` gets an RFH link (`language-basics/enums.html`).
  - *Quiz:*
    1. q: "What does `?` do inside a handler returning `Result<_, AppError>`?"; options: ["Ignores the error", "Returns the error early; Axum turns it into a response", "Retries", "Panics"]; answer 1; explain: "`?` stops the handler and returns the Err; AppError's IntoResponse makes the HTTP answer."
    2. q: "Why implement `IntoResponse` for `AppError`?"; options: ["To print it", "So Axum knows which status and body each error becomes", "To make it Clone", "thiserror requires it"]; answer 1; explain: "Axum can only send types that implement IntoResponse."
    3. q: "What does `#[error(\"book {0} not found\")]` generate?"; options: ["A route", "The text shown by `to_string()` for that variant", "A log line", "An HTTP header"]; answer 1; explain: "thiserror writes the Display implementation for you."
    4. q: "Why avoid `.unwrap()` for 'book not found'?"; options: ["It's slower", "It panics and the client gets a 500 instead of a clear 404", "It doesn't compile", "It deletes the book"]; answer 1; explain: "A missing book is an expected case; handle it with an error value, not a crash."
  - *Flashcards:* "App-wide error pattern?" → "enum AppError + `impl IntoResponse` + handlers return `Result<T, AppError>` and use `?`"; "Option to error?" → "`.ok_or(AppError::NotFound(id))?`"

### Task 10: `middleware-and-tower-layers` (Intermediate)

- **Bundle:** `topics/middleware-and-tower-layers/`. Anchors: `imports`, `handlers`, `middleware`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can run code before and after every request with middleware.", "You can add ready-made layers: request logging, CORS and timeouts.", "You know the order layers run in."
  - summary: "Middleware wraps every request: write your own with `from_fn`, and add tower-http's TraceLayer, CorsLayer and TimeoutLayer."
  - prereq `["error-handling-in-axum"]`; next `["nesting-and-modular-routers"]`
  - rfhLinks: logging-and-tracing, closures
  - links: `https://docs.rs/tower-http/0.7.1/tower_http/`, `https://docs.rs/axum/0.8.9/axum/middleware/index.html`, MDN CORS `https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS`
- **Brief:**
  - *What & why:* airport security. Every passenger passes the same checks on the way in and the way out, whatever their destination.
  - *The idea, slowly:*
    - `middleware`: `Request`, `Next`, `next.run(request).await`, and changing the response.
    - `app`: `.layer` order. Include a ` ```text ` onion diagram with Line by line: the last `.layer` is the outermost, so a request passes Trace → Cors → Timeout → from_fn → handler and the response goes back out.
    - `main`: `tracing_subscriber` and the `with_env_filter` string.
    - The tower-http features line in Cargo.toml.
  - *Run it:*
    - `http/01-header.sh`: GET / shows `x-powered-by`; GET /missing still has it.
    - `http/02-cors.sh`: `curl -i -H 'origin: https://example.com' …` shows `access-control-allow-origin: *`.
    - `http/03-timeout.sh`: /slow gives 408 after 1 s (verified in planning).
    - Logs (hand-captured, real): the server terminal output for two requests.
  - *You might be wondering…:* "Middleware or a handler?"; "Is `CorsLayer::permissive()` safe?" (no, not for production; show a specific-origin version in More examples); "Why does order matter?"; "What is tower?"
  - *Common mistakes:*
    - Forgetting `.await` on `next.run` (the real compiler error).
    - Adding `.layer` before the routes it should cover. Routes added after a `.layer` are not wrapped; show it with a real transcript from an example.
    - Expecting the timeout to cancel a blocking `std::thread::sleep` (describe it; the real transcript shows it hangs). Make it a real example with a 3 s blocking sleep behind the 1 s timeout; the transcript shows the timeout doesn't fire until the blocking work ends. **Only if it is deterministic.** Verify with --check-twice; if not, describe it in prose.
  - *More examples:* a timing header `x-response-time-ms` (nondeterministic, so show it in prose only, no transcript); `CorsLayer::new().allow_origin("https://myapp.com".parse::<HeaderValue>().unwrap())`; `route_layer` to protect only some routes; `from_fn` that rejects requests without a header.
  - *Your turn:* 🟢 change the header value; 🟡 add a second middleware that adds `x-request-count` using an AtomicU64 in a static; 🔴 middleware that answers 503 with "maintenance" for every path except `/health` when an env var `MAINTENANCE=1` is set. The transcript uses a `# serve:` with no env, so show the maintenance case in the test only.
  - *Glossary:* `middleware`, `layer`, `CORS`, `timeout`.
  - *Quiz:*
    1. q: "What does `next.run(request).await` do in middleware?"; options: ["Stops the request", "Passes the request on to the handler (and inner layers) and gets the response back", "Logs the request", "Restarts the server"]; answer 1; explain: "Middleware wraps the rest of the app; `next.run` calls inward and returns the response."
    2. q: "Which layer is outermost?"; options: ["The first `.layer` call", "The last `.layer` call", "They run in random order", "TraceLayer always"]; answer 1; explain: "Each `.layer` wraps everything added before it, so the last one sees the request first."
    3. q: "What does CORS control?"; options: ["Which databases can connect", "Which websites' JavaScript may call your API from a browser", "Server speed", "JSON format"]; answer 1; explain: "CORS headers tell browsers which other origins may read your responses."
    4. q: "The handler takes 3 s and TimeoutLayer allows 1 s. The client gets…"; options: ["200 after 3 s", "408 after about 1 s", "500 immediately", "Nothing"]; answer 1; explain: "The timeout layer gives up and answers with the status you chose."
  - *Flashcards:* "Custom middleware?" → "`async fn m(req: Request, next: Next) -> Response` + `.layer(middleware::from_fn(m))`"; "Request logging?" → "`.layer(TraceLayer::new_for_http())` + a tracing_subscriber"

### Task 11: `nesting-and-modular-routers` (Intermediate)

- **Bundle:** `topics/nesting-and-modular-routers/`, with files `main.rs` (anchors `modules`, `imports`, `app`, `main`), `health.rs` (anchor `health`) and `books.rs` (anchor `books`).
- **Topic entry:**
  - outcomes: "You can split routes into modules, one file per area.", "You can mount a group of routes under a prefix with `nest`.", "You can combine routers with `merge`."
  - summary: "Grow beyond one file: routers in modules, `merge` to combine them, `nest` to mount them under /api/books."
  - prereq `["middleware-and-tower-layers"]`; next `["custom-extractors"]`
  - rfhLinks: modules-and-crates, visibility-and-privacy `language-basics/visibility-and-privacy.html`
  - links: `https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.nest`
- **Brief:**
  - *What & why:* a department store has floors; each floor manages its own shelves, and the lobby map points to floors.
  - *The idea, slowly:* the folder tree as a ` ```text ` diagram with Line by line; `health` (`pub fn router`); `books` (routes relative to the mount point); `modules` (`mod`); `app` (`merge` vs `nest`).
  - *Run it:* `http/01-routes.sh`: /health, /api/books, /api/books/7, /books (404), /api/books/ (trailing slash gives 404; verified).
  - *You might be wondering…:* "merge vs nest?"; "Why is the books route `/` and not `/api/books`?"; "What about the trailing slash?" (one explicit route or a redirect; More examples); "How do nested routers share state?" (More examples: `Router<AppState>` + `.with_state` once at the top).
  - *Common mistakes:* forgetting `pub` on `router` (the real error "function `router` is private"); forgetting `mod books;` (the real "failed to resolve"); nesting at `"/api/books/"` with a trailing slash (the real panic, captured).
  - *More examples:* nested routers sharing `AppState` (`fn router() -> Router<AppState>`); two nesting levels (`/api` → `/v1`); a redirect for the trailing slash; a middleware layer applied to one nested router only.
  - *Your turn:* 🟢 add `/api/books/{id}/reviews` in books.rs; 🟡 add an `authors.rs` module mounted at `/api/authors`; 🔴 split the shared-state lesson's app into `state.rs` + `books.rs` modules with `Router<AppState>`, and keep its tests passing.
  - *Glossary:* `module` gets an RFH link; `nest` and `merge` are explained inline.
  - *Quiz:*
    1. q: "`.nest(\"/api/books\", books::router())` with a route `/{id}` inside — which URL matches?"; options: ["/{id}", "/api/books/7", "/books/7", "/api/7"]; answer 1; explain: "nest adds the prefix to every route of the inner router."
    2. q: "What's the difference between `merge` and `nest`?"; options: ["None", "merge combines routes as-is; nest mounts them under a prefix", "nest is faster", "merge only works with state"]; answer 1; explain: "merge keeps paths unchanged; nest prefixes them."
    3. q: "Why must `router()` be `pub`?"; options: ["Axum needs it", "main.rs is a different module and can only call public functions", "For speed", "It doesn't"]; answer 1; explain: "Items are private to their module unless marked pub."
    4. q: "Does `/api/books/` (trailing slash) match a nested `/` route?"; options: ["Yes, always", "No — Axum treats it as a different path and answers 404", "It redirects automatically", "It returns 405"]; answer 1; explain: "Paths match exactly; add a route or a redirect if you want both."
  - *Flashcards:* "Mount routes under a prefix?" → "`.nest(\"/api/books\", books::router())`"; "Combine two routers as-is?" → "`.merge(other)`"

### Task 12: `custom-extractors` (Intermediate)

- **Bundle:** `topics/custom-extractors/`. Anchors: `imports`, `extractor`, `handlers`, `app`, `main`.
- **Topic entry:**
  - outcomes: "You can write your own extractor with `FromRequestParts`.", "You can reject a request with your own status and message.", "You can protect a route by adding one argument to its handler."
  - summary: "Write an `ApiKey` extractor with `FromRequestParts`: check a header once, reuse it on any handler, reject with 401 or 403."
  - prereq `["nesting-and-modular-routers"]`; next `["input-validation"]`
  - rfhLinks: traits-basics, generics
  - links: `https://docs.rs/axum/0.8.9/axum/extract/trait.FromRequestParts.html`
- **Brief:**
  - *What & why:* a bouncer at the door: every protected room asks for the same wristband, so write the check once.
  - *The idea, slowly:* `extractor`, line by line:
    - `impl<S> … where S: Send + Sync` (generics in one gentle paragraph; RFH link)
    - `type Rejection`
    - `async fn from_request_parts`
    - `parts.headers.get`, `.and_then`, `.to_str().ok()`, `.ok_or(...)?`
    - the key comparison

    Then `handlers` (the handler that takes `ApiKey`); `app`.
  - *Run it:* `http/01-keys.sh`: /public (200); /secret without a header (401); `-H 'x-api-key: guess'` (403); `-H 'x-api-key: letmein'` (200).
  - *You might be wondering…:* "FromRequestParts vs FromRequest?" (parts = headers/URL; FromRequest can read the body, as the next lesson shows); "401 vs 403?" (link back to how-a-web-backend-works); "Should the key be hard-coded?" (no: config and env vars in A4, italic mention); "Is this real authentication?" (a first step; JWT in the capstone, italic mention).
  - *Common mistakes:* forgetting `S: Send + Sync` (the real compiler error); putting a body extractor (`Json`) before the custom one. This is not an error here, since it's parts, so choose a real mistake instead: implementing `FromRequest` when you only need headers, then using it alongside `Json` (the real error: two body extractors). Also: comparing keys with `==` is a timing-attack risk. Mention it briefly for honesty and point to `constant_time_eq`, without adding a dependency.
  - *More examples:* an extractor returning `AppError`-style JSON rejections; extracting the `user-agent`; an `Option<ApiKey>` via `OptionalFromRequestParts`. **Check the axum 0.8.9 API first. Only include it if it compiles; otherwise use `Result<ApiKey, …>` as the argument.** Also: a key checked against a list in state (`FromRequestParts<AppState>`).
  - *Your turn:* 🟢 change the key; 🟡 accept keys from either `x-api-key` or a `?key=` query parameter; 🔴 an `AdminKey` extractor that requires the key AND `x-role: admin`, answering 403 "admins only" otherwise.
  - *Glossary:* `API key`, `rejection`; `generics` gets an RFH link.
  - *Quiz:*
    1. q: "Why write an extractor instead of checking the header in each handler?"; options: ["It's required", "Write the check once and reuse it by adding one argument", "It's faster", "Headers can't be read in handlers"]; answer 1; explain: "An extractor is reusable: any handler that takes ApiKey is protected."
    2. q: "What does `type Rejection` decide?"; options: ["The route", "What response the client gets when extraction fails", "The key value", "The port"]; answer 1; explain: "If from_request_parts returns Err, Axum sends the Rejection as the response."
    3. q: "Missing header → 401, wrong key → 403. Why different?"; options: ["Random choice", "401: we don't know who you are; 403: we know, but you're not allowed", "403 is for servers only", "They mean the same"]; answer 1; explain: "Same distinction as in How a web backend works."
    4. q: "`FromRequestParts` can read…"; options: ["The body", "Headers, URL and method — not the body", "Only cookies", "The database"]; answer 1; explain: "Parts are everything except the body; FromRequest is for body extractors."
  - *Flashcards:* "Own extractor from headers?" → "`impl<S: Send + Sync> FromRequestParts<S> for X { type Rejection = …; async fn from_request_parts(…) }`"; "Protect a route?" → "Add the extractor as a handler argument."

### Task 13: `input-validation` (Intermediate)

- **Bundle:** `topics/input-validation/`. Anchors: `imports`, `input`, `extractor`, `handler`, `main`.
- **Topic entry:**
  - outcomes: "You can declare rules on input fields (length, email, range).", "You can build a `ValidatedJson` extractor that checks them before your handler runs.", "You can return every problem at once as a 422 JSON response."
  - summary: "Parse, then validate: the validator crate plus a `ValidatedJson<T>` extractor gives clear 422 errors listing every bad field."
  - prereq `["custom-extractors"]`; next `["testing-handlers"]`
  - rfhLinks: generics, traits-basics
  - links: `https://docs.rs/validator/0.21.0/validator/`, `https://docs.rs/axum/0.8.9/axum/extract/trait.FromRequest.html`
- **Brief:**
  - *What & why:* JSON that parses can still be nonsense: a 2-letter username, an email "nope", age 9. It's like a form that's filled in (so it parses) but filled in wrongly. Validation checks meaning, not shape.
  - *The idea, slowly:*
    - `input`: each `#[validate(…)]` rule.
    - `extractor`: `FromRequest` vs `FromRequestParts` (it reads the body); the where-clause read aloud; the two-step Json-then-validate; `Json(errors)` serializing the error map.
    - `handler`.
    - The validator line in Cargo.toml, with `features = ["derive"]`.
  - *Run it:* `http/01-good.sh` (201); `http/02-bad-values.sh` (the real 422 JSON listing username, email and age, verified shape `{"username":[{"code":"length",…}],…}`; explain it field by field); `http/03-bad-json.sh` (a missing field gives Axum's 422 text; broken JSON gives 400. Two different 422s: parse vs rules).
  - *You might be wondering…:* "Why not check in the handler with if statements?"; "Why are all errors returned at once?"; "Can I have custom messages?" (More examples: `#[validate(length(min = 3, message = "…"))]`); "Is the database a second line of defence?" (yes: CHECK and UNIQUE from A1; link keys-and-relations).
  - *Common mistakes:* forgetting `derive(Validate)` (the real compiler error at `ValidatedJson<SignUp>`); forgetting `features = ["derive"]` (the real error "cannot find derive macro `Validate`", from a scratch copy); validating an `Option` field incorrectly. Only include that last one if the real behavior is instructive; verify it.
  - *More examples:* custom messages; a custom validation function (`#[validate(custom(function = "no_spaces"))]`); nested validation (`#[validate(nested)]`); `must_match` for password confirmation.
  - *Your turn:* 🟢 raise the minimum age to 16; 🟡 add `website: Option<String>` with `#[validate(url)]`; 🔴 a `CreateBook { title: 1..=200 chars, pages: 1..=5000, isbn: exactly 13 digits via regex or a custom fn }`. Prefer the custom fn, to avoid adding a regex dependency.
  - *Glossary:* `validation`.
  - *Quiz:*
    1. q: "What's the difference between parsing and validating?"; options: ["None", "Parsing checks the JSON shape; validating checks the values make sense", "Validating is faster", "Parsing checks emails"]; answer 1; explain: "Json<T> only proves the shape; rules like 'email must look like an email' are validation."
    2. q: "Why does `ValidatedJson` implement `FromRequest` rather than `FromRequestParts`?"; options: ["Style", "It needs to read the request body", "It's newer", "FromRequestParts is deprecated"]; answer 1; explain: "Only FromRequest extractors can consume the body."
    3. q: "Which status fits 'the JSON is fine but the email is invalid'?"; options: ["400", "404", "422", "500"]; answer 2; explain: "422 Unprocessable Entity: understood, but the content breaks the rules."
    4. q: "Why return all errors at once?"; options: ["It's required", "So the user fixes every field in one go instead of one per request", "It's faster to compute", "Browsers need it"]; answer 1; explain: "Reporting every problem together is kinder to the person filling in the form."
  - *Flashcards:* "Declare a rule?" → "`#[derive(Validate)]` + `#[validate(length(min = 3))]`"; "Validate automatically?" → "A `ValidatedJson<T>` FromRequest extractor: Json first, then `.validate()`, 422 on errors."

### Task 14: `testing-handlers` (Intermediate)

- **Bundle:** `topics/testing-handlers/`, with files `src/lib.rs` (anchor `lib`), `src/main.rs` (anchor `main`), and `tests/api.rs` (anchors `helpers`, `first_test`, `json_test`, `state_test`, `error_test`).
- **Topic entry:**
  - outcomes: "You can test an Axum app without starting a server.", "You can split an app into a library and a thin `main` so tests can import it.", "You can test JSON, shared state and error responses."
  - summary: "Test handlers with `tower::ServiceExt::oneshot`: a lib/main split, a tests/ folder, and tests for JSON, state and errors."
  - prereq `["input-validation"]`; next `["a2-build-todo-api", "cheatsheet-axum"]`
  - rfhLinks: unit-testing, integration-testing
  - links: `https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot`, axum testing example `https://github.com/tokio-rs/axum/tree/main/examples/testing`
- **Brief:**
  - *What & why:* you checked every lesson by hand with curl. Tests do the same checks in milliseconds, every time you change code, and CI runs them on every push. By now you've been reading the `#[cfg(test)]` blocks in every crate; this lesson explains them.
  - *The idea, slowly:*
    - A ` ```text ` folder diagram (lib.rs, main.rs, tests/api.rs) with Line by line.
    - `lib` (`pub` items, `pub fn app()`).
    - `main` (`testing_handlers::app()`, crate names with `_`).
    - `helpers`: `oneshot` explained slowly (the Router as a Service; one request in, one response out; no network); collecting the body.
    - `first_test`, `json_test` (`serde_json::from_str` into `Note`, why `PartialEq` and `Debug` are derived), `state_test` (`app.clone()`: why cloning keeps the shared state), `error_test`.
    - The dev-dependencies section of Cargo.toml.
  - *Run it:* `cd code && cargo test -p testing-handlers`. This is a hand-captured real terminal output, showing the test names and "test result: ok. 4 passed". It's not an http script. Plus `http/01-by-hand.sh` doing the same as the tests with curl, for comparison.
  - *You might be wondering…:* "Unit tests or tests/ folder?"; "Does oneshot start a server?" (no); "Why `.clone()` the app?"; "How do I test with a database?" (italic pointer to A3's testing lesson).
  - *Common mistakes:* testing a private function from tests/ (the real error); forgetting `#[tokio::test]` on an async test (the real error); reusing the app after `oneshot` without clone (the real "use of moved value" error).
  - *More examples:* testing headers; a test helper returning parsed JSON (`serde_json::Value`); a table-driven test over several URLs; testing that a slow route times out (reusing the middleware idea).
  - *Your turn:* 🟢 add a test that GET /notes starts empty; 🟡 add DELETE /notes (clear) to lib.rs with a test; 🔴 write tests for the input-validation lesson's rules. Put them in that crate's tests, which needs the lib split there; accept moving it into an example crate if cleaner, and document the choice.
  - *Glossary:* `test` (RFH link), `integration test`, `oneshot` inline.
  - *Quiz:*
    1. q: "What does `app().oneshot(request)` do?"; options: ["Starts the server on port 3000", "Sends one request straight into the router and returns the response — no network", "Deploys the app", "Runs every test"]; answer 1; explain: "oneshot drives the Router as a Service, entirely in memory."
    2. q: "Why split the app into lib.rs and main.rs?"; options: ["Axum requires it", "So tests in tests/ can import `app()` from the library", "It's faster", "To hide main"]; answer 1; explain: "Integration tests can only use a crate's public library items."
    3. q: "In the state test, why `app.clone()` before each request?"; options: ["oneshot consumes the app; clones share the same Arc'd state", "To reset the data", "To make it faster", "It's a mistake"]; answer 0; explain: "Each clone points at the same shared list, so later requests see earlier changes."
    4. q: "What does `#[tokio::test]` do?"; options: ["Runs the test on a Tokio runtime so it can `.await`", "Marks slow tests", "Starts the server", "Skips the test"]; answer 0; explain: "Async tests need a runtime, just like an async main."
  - *Flashcards:* "Test a handler without a server?" → "`app().oneshot(Request::get(\"/x\").body(Body::empty())?).await`"; "Share state across test requests?" → "Build the app once, `.clone()` it per request."

### Task 15: Project `a2-build-todo-api` (Beginner → Intermediate, fully guided)

**Files:** Create `code/projects/a2-build-todo-api/` (copy the bundle's `projects/a2-build-todo-api/` byte for byte; it passes `tests/api.rs`), `examples/step1.rs` … `step5.rs`, `http/*.sh`, `src/a2-axum/a2-build-todo-api.md`; modify `tools/topics.data.js`.

**Topic entry:**
- `kind: "project"`, `level: "Intermediate"`
- outcomes: "You built a complete REST API for todos from an empty folder.", "You combined routing, JSON, state, errors, validation and tests in one app.", "You tested it by hand and with automated tests."
- summary: "Build a complete Todo REST API — create, list, show, update, delete — using everything from Part A2."
- prereq `["testing-handlers"]`; next `["cheatsheet-axum", "what-is-an-orm"]`
- codeDir `code/projects/a2-build-todo-api`

**The final code** is the bundle, with these modules:
- `lib.rs`: `app()` with `/health` and `nest("/api/todos")`
- `error.rs`: `AppError::NotFound` → 404 JSON
- `validated.rs`: `ValidatedJson`
- `todos.rs`: the `Todo`/`NewTodo`/`UpdateTodo` types, a `Store { next_id, todos }` behind `Arc<Mutex<…>>`, and the handlers `list`/`create`/`show`/`update` (PATCH, partial)/`remove` (204 or 404)
- `main.rs`
- `tests/api.rs` (lifecycle + validation)

**Guided steps.** Each step is a full program in `examples/stepN.rs` that the reader types. Run it with `cargo run -p a2-build-todo-api --example stepN`. Each has an http script checkpoint (`http/0N-stepN.sh` with `# serve: -p a2-build-todo-api --example stepN`) and at least one test:
1. step1: `/health` only (hello-axum).
2. step2: GET/POST `/todos` with in-memory state and JSON (json + shared-state), no ids beyond `len+1`.
3. step3: `next_id` counter + GET `/todos/{id}` with `AppError` 404 (errors; explain why `len+1` breaks after deletes).
4. step4: PATCH (partial update with `Option` fields) + DELETE 204/404 (routes-and-methods).
5. step5: `ValidatedJson` on create/update (input-validation) and the real 422 transcript for an empty title.
6. step 6 is not an example file. The reader moves the code into modules and `nest("/api/todos")` (nesting), which is exactly the bundle's `src/` layout plus `tests/api.rs`. Its checkpoints:
   - `cargo test -p a2-build-todo-api`, shown as real hand-captured terminal output;
   - the final transcript `http/06-final.sh` with `# serve: -p a2-build-todo-api` (the real binary), running the full lifecycle: create 2, list, patch done, delete, 404, 422.

   So the examples are `step1.rs`–`step5.rs` only.

**Page:** the project format (What you'll build → What you need to know (links to all 12 lessons) → The spec (endpoints table: method, path, body, success status, error statuses; the rules: title 1–100 chars, ids never reused, PATCH changes only the given fields) → Build it, step by step (6 steps, each with a goal, a hint in `<details>`, and a ✅ checkpoint showing the script include + `.out` include) → Reference solution, line by line (the final `src/` files by anchor; add anchors to the copied bundle files as needed, comment lines only) → Stretch goals (each a solution example with a test: `?done=true` filter via Query; `DELETE /todos?done=true` clears finished items; a `created_order` sort) → Remember this → Go deeper). It has no quiz.

### Task 16: `cheatsheet-axum` + phase check

- **Files:** Create `code/topics/cheatsheet-axum/` (one compiled crate: `src/main.rs` with anchors per pattern and a test per pattern), `src/a2-axum/cheatsheet-axum.md`; modify `tools/topics.data.js`.
- **Topic entry:** `kind: "cheatsheet"`, summary "Every Axum pattern from Part A2 on one page, each linked to the lesson that explains it.", prereq `["testing-handlers"]`, next `["what-is-an-orm"]`, codeDir `code/topics/cheatsheet-axum`.
- **Patterns** (one `## How do I…?` item each; each item includes its anchor and links to its lesson):
  - start a server
  - route by method
  - return status + headers
  - read a path value
  - read the query string
  - receive and send JSON
  - share state
  - return errors with `?`
  - add middleware
  - add CORS/timeout/logging layers
  - split into modules and nest
  - write a custom extractor
  - validate input
  - test a handler with oneshot
- **Link sweep:** grep `src/` for italic or plain mentions of A2 page titles now published, and turn them into links. Example: A1 pages or the tour may mention *Hello, Axum*. Leave A3/A4/capstone mentions in italics.
- **Phase check:**

```bash
npm test && node tools/generate.mjs && git status --short   # clean after commits
node tools/validate.mjs                                      # 0 errors, 0 warnings; 27 published pages
mdbook build 2>&1 | grep -E ' (ERROR|WARN) ' ; echo "grep-exit=$?"
(cd code && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets)
node tools/http-check.mjs --check-twice                      # all HTTP outputs match
RBH_PSQL=… node tools/sql-check.mjs --check-twice            # unchanged, all match (scratch Postgres)
```

  Commit `docs(a2): Axum cheat sheet` and `docs(a2): link now-published lessons`.
