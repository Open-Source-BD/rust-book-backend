# Design: "Rust Backend for Humans" — Axum + SeaORM + PostgreSQL book

Date: 2026-09-24
Status: Draft, awaiting review

## 1. Intent

**Outcome:** A free website/book that takes a learner who knows basic Rust from zero backend knowledge
to building and deploying a production-grade e-commerce REST API with Axum, SeaORM and PostgreSQL.

**Audience:** Beginner → intermediate. Assume the reader is new to backend work, possibly new to
programming in general or coming from JS/Python/Java/Go. They must never have to guess *why*
something is there or *what they get* from it.

**Core rule of the book:** answer the question before the reader gets confused.

**Relationship to Rust for Humans:** This is a sequel to `rustbook-for-human` (sibling repo). It does
not re-teach the Rust language. When a Rust concept appears (ownership, traits, `Result`, `?`,
async, `Arc`), the lesson gives a one-sentence reminder and links to the matching Rust for Humans lesson.

**Success criteria:**
- A reader who completes the book has a working, tested, deployed ShopRS API.
- Every code listing in the book compiles and passes tests in CI (no drift between book and code).
  Every Rust listing anywhere in a lesson (including `## More examples` and `## Your turn`
  solutions) is `{{#include}}`/`{{#rustdoc_include}}`d from compiled code under `code/`. Two
  exceptions only: ` ```rust,editable ` blocks of pure-std code that run on the Playground, and
  fences tagged ` ```rust,noplayground,ignore ` holding deliberately broken code under Common
  mistakes, each next to its real compiler error. `validate.mjs` errors on any other Rust fence.
- Every line of every listing is explained (What / Why / How / Remove it and…).
- Every lesson states its outcomes up front and passes `tools/validate.mjs`.
- Before Part B, the reader has built 5 working projects of their own (library schema, Todo API,
  blog data layer, Notes API, URL shortener), the last one from a spec with no step-by-step guide.

**Decisions made with the user:**
| Decision | Choice |
|---|---|
| Scope | Sequel to Rust for Humans |
| Structure | Hybrid: topic lessons (Part A), then a capstone project (Part B) |
| Capstone | Backend REST API only (no frontend), production-grade e-commerce, intermediate level |
| Site tech | Reuse the Rust for Humans setup (mdBook, generator, quizzes, flashcards, GitHub Pages) |
| Code correctness | Real Cargo crates under `code/`, pulled into lessons via mdBook `{{#include}}` |

**Assumption (change if wrong):** the working title is "Rust Backend for Humans".

## 2. Table of contents (62 lessons + 4 cheat sheets)

Each lesson has a level badge (Beginner / Intermediate). Every capstone chapter opens with a
"New in this chapter" box listing at most 3 new ideas.

### 2.1 Confidence ladder (Part A goal)

Part A's job is not only to explain topics. By the end of Part A the reader should feel able to
build *any* small backend on their own, before starting the capstone. Four mechanisms get them there:

1. **A "Build it" project at the end of each part.** It uses only what that part taught, so the
   reader gets an early win:
   - A1 → `a1-build-library-schema`: design and query a library database in plain SQL
   - A2 → `a2-build-todo-api`: an in-memory Todo REST API with Axum
   - A3 → `a3-build-blog-data`: a blog data layer (users → posts → comments) with SeaORM
   - A4 → `mini-notes-api` (already listed): a full CRUD API with Axum + SeaORM + Postgres
2. **Less hand-holding each time.** Build-it projects move from guided to independent:
   A1/A2 are fully guided (step by step, with checkpoints), A3 is half-guided (steps named but
   the code comes out of hint ladders), and the final project is independent (spec + hints only).
3. **A final independent project + readiness check** before the capstone:
   - `a4-build-url-shortener`: the reader builds a URL shortener API from a written spec
     (requirements, endpoints, table design, acceptance tests given as curl commands). They get
     three-level hint ladders (nudge → approach → code), and a reference solution with a full
     line-by-line walkthrough comes after the reader's own attempt.
   - `ready-for-the-capstone`: a self-check list ("I can add a route… write a migration…
     return a proper error…"). Each item links back to the lesson that teaches it, so gaps get
     filled before Part B.
4. **Tiered "Your turn" exercises in every topic lesson:** 🟢 *Guided* (fill in the blank), 🟡 *Tweak*
   (change working code to do something new), 🔴 *From scratch* (write it from a one-line spec).
   Each tier has a hidden solution.

**Cheat sheets** (one reference page per part, not lessons): `cheatsheet-sql`, `cheatsheet-axum`,
`cheatsheet-seaorm`, `cheatsheet-project-patterns`. Each is a one-page "How do I…?" list of the
patterns from that part, with a link to the lesson that explains each one. Readers keep them open
while building their own projects.

### 2.2 Project page format (Build-it projects)

Build-it pages use their own section order instead of the 11-section lesson format:
`## What you'll build` (outcome + final demo output) → `## What you need to know` (links to the
lessons used) → `## The spec` (requirements, endpoints/tables, acceptance checks) → `## Build it,
step by step` (each step has a goal, hints in `<details>`, and a ✅ checkpoint command with expected
output) → `## Reference solution, line by line` → `## Stretch goals` → `## Remember this`.
The code of a Rust project lives in `code/projects/<slug>/` and is included and tested like all
other code. A SQL-only project (such as A1's `a1-build-library-schema`) has no Rust, so it keeps its
code in `code/sql/<slug>/` like a SQL lesson: `NN-name.sql` files whose `.out` output is written and
checked by `tools/sql-check.mjs` (see *Code ↔ book link*).

### Part 0 · Before you start (4)
1. `how-to-use-this-book` — prerequisites, links to Rust for Humans lessons
2. `your-toolbox` — Rust, Docker + Postgres, `sea-orm-cli`, curl/Bruno
3. `how-a-web-backend-works` — HTTP, request/response, JSON, REST, status codes
4. `tour-of-the-stack` — what Tokio, Axum, SeaORM and Postgres each do, and how they connect

### Part A1 · PostgreSQL & SQL (7 + 1 project)
`what-is-a-database` · `tables-rows-and-psql` · `postgres-data-types` · `crud-in-sql` ·
`keys-and-relations` · `indexes` · `sql-transactions` · 🛠 `a1-build-library-schema` · 📄 `cheatsheet-sql`

### Part A2 · Axum (12 + 1 project)
`hello-axum` (`#[tokio::main]`, `Router`, `TcpListener`) · `routes-and-methods` ·
`handlers-and-into-response` · `path-and-query-extractors` · `json-and-serde` · `shared-state` ·
`error-handling-in-axum` · `middleware-and-tower-layers` · `nesting-and-modular-routers` ·
`custom-extractors` · `input-validation` · `testing-handlers` · 🛠 `a2-build-todo-api` · 📄 `cheatsheet-axum`

### Part A3 · SeaORM (12 + 1 project)
`what-is-an-orm` · `connecting-to-postgres` · `migrations` · `generating-entities` ·
`entity-model-activemodel-column` · `inserting-rows` · `selecting-rows` (filter, order, paginate) ·
`update-and-delete` · `relations-and-loading` · `seaorm-transactions` · `raw-sql-and-custom-selects` ·
`testing-with-seaorm` (MockDatabase + real test DB) · 🛠 `a3-build-blog-data` · 📄 `cheatsheet-seaorm`

### Part A4 · Putting it together (3 + 2 projects + readiness check)
`config-and-env` · `logging-with-tracing` · `project-layout` (routes → handlers → services → DB) ·
🛠 `mini-notes-api` (guided: full CRUD API combining A2 + A3) ·
🛠 `a4-build-url-shortener` (independent: build from spec) · ✅ `ready-for-the-capstone` ·
📄 `cheatsheet-project-patterns`

### Part B · Capstone: ShopRS e-commerce API (18)
1. `shop-01-plan` — features, ER diagram, API design, folder layout
2. `shop-02-skeleton` — config, tracing, `/health`, docker-compose
3. `shop-03-schema` — migrations + entities: users, categories, products, carts, cart_items, orders, order_items, payments
4. `shop-04-errors-and-responses` — error type and consistent JSON response format
5. `shop-05-register` — argon2 password hashing
6. `shop-06-login-jwt` — JWT + `AuthUser` extractor
7. `shop-07-roles` — admin-only routes
8. `shop-08-categories` — categories CRUD
9. `shop-09-products` — products CRUD + validation
10. `shop-10-product-listing` — pagination, filters, search, sort
11. `shop-11-cart`
12. `shop-12-checkout` — order in a transaction, stock deduction, price snapshot, row locking
13. `shop-13-payments` — `PaymentProvider` trait, Stripe test mode, idempotent webhook
14. `shop-14-orders` — order history + admin order-status workflow (state machine)
15. `shop-15-hardening` — rate limits, CORS, request IDs, timeouts, graceful shutdown
16. `shop-16-integration-tests` — against a real Postgres
17. `shop-17-openapi` — API docs with `utoipa`
18. `shop-18-ship-it` — multi-stage Dockerfile, migrations on startup, CI, deploy

Plus non-lesson pages: `introduction`, `glossary`, `review` (flashcards).

## 3. Lesson page format

Fixed section order, enforced by `validate.mjs`:

| # | Heading | Contents |
|---|---|---|
| 1 | `## By the end of this lesson` | 2–3 concrete, checkable outcomes |
| 2 | `## What & why` | Real-world problem solved + plain-language analogy |
| 3 | `## The idea, slowly` | Small steps. After each listing: `### Line by line` and `### Run it` |
| 4 | `## You might be wondering…` | Pre-emptive FAQ at the point of confusion |
| 5 | `## Coming from another language?` | Express / Flask / Spring / Go equivalents |
| 6 | `## Common mistakes` | Real compiler/runtime errors, meaning, fix |
| 7 | `## More examples` | 4 scenario subsections (`### title`, one-line hook, code) |
| 8 | `## Your turn` | Three tiered exercises: 🟢 Guided, 🟡 Tweak, 🔴 From scratch, each with a hidden solution (`<details>`) |
| 9 | `## Quick check` | `<div class="quiz" data-topic="<slug>"></div>` |
| 10 | `## Remember this` | 3–5 takeaways |
| 11 | `## Go deeper` | Official docs + Rust for Humans links |

Lessons that are purely conceptual (e.g. `how-a-web-backend-works`) may have no Rust listing; there
`### Line by line` applies to SQL/HTTP/shell listings instead. Validation requires at least one
`### Line by line` per lesson that contains a code fence.

**Line-by-line entry format:**

```markdown
`let app = Router::new().route("/hello", get(hello));`
- **What:** creates the app's route table and adds one rule.
- **Why:** Axum has to know which function answers which URL.
- **How:** `Router::new()` starts an empty table; `.route(path, method_router)` adds a rule; `get(hello)` means "only for GET, call `hello`".
- **Remove it and…** every request gets `404 Not Found`.
```

Trivial lines (a closing brace, a blank line) are not listed. Repeated boilerplate already explained
earlier in the same lesson is referenced, not repeated.

**Run it format:** exact command, exact output copied from a real run, then
"✅ If you see this, you're right" / "❌ If you see *error*, do *fix*".

**Writing rules (every page):**
- No undefined words: every new term is defined in one sentence on first use and links to `glossary.md`.
- No magic: macros and attributes (`#[tokio::main]`, `#[derive(DeriveEntityModel)]`, `.await?`) are explained, including what they do behind the scenes.
- Nothing hidden: listings show their `use` lines; each new crate appears with its `Cargo.toml` line and a reason.
- At most 3 new ideas per lesson.
- Banned words in prose: "simply", "just", "obviously", "trivially" (validator warns).

## 4. Repository layout

```
rust-book-backend/
├── book.toml
├── src/
│   ├── SUMMARY.md                 # AUTO-GENERATED
│   ├── introduction.md  glossary.md  review.md
│   └── part-0-start/ a1-postgres/ a2-axum/ a3-seaorm/ a4-together/ b-capstone/
├── questions/<slug>.json
├── theme/
│   ├── retention.js  retention.css   # copied from rustbook-for-human; storage prefix changed to "rbh:v1:"
│   ├── head.hbs
│   └── questions.data.js             # AUTO-GENERATED
├── tools/
│   ├── topics.data.js                # single source of truth for lessons
│   ├── generate.mjs
│   └── validate.mjs
├── code/                             # Cargo workspace
│   ├── Cargo.toml                    # [workspace] + [workspace.dependencies] with pinned versions
│   ├── topics/<slug>/                # one small crate per Part A lesson that needs code
│   ├── projects/<slug>/              # reference solutions for the Build-it projects
│   └── shop/step-NN/                 # complete ShopRS snapshot after each capstone chapter
├── docker-compose.yml                # Postgres 18
└── .github/workflows/
    ├── deploy.yml                    # generate → mdbook build → GitHub Pages
    └── code.yml                      # Postgres service → fmt --check, clippy -D warnings, test
```

### topics.data.js entry shape
`{ slug, title, part, kind, level, outcomes[], summary, prereq[], next[], rfhLinks[{label, href}], links[{label, href, note}], codeDir? }`

`kind` is one of `lesson` (11-section format), `project` (Build-it format, §2.2), `cheatsheet`, or
`checklist`. The generator and validator pick the required headings by `kind`.

### generate.mjs (adapted from Rust for Humans)
- Writes a stub per topic with the headings for its `kind` (skips existing files; `--force` overwrites).
- Writes an empty `questions/<slug>.json` if missing.
- Always regenerates `src/SUMMARY.md` grouped by part, and `theme/questions.data.js`.

### validate.mjs
Errors (non-zero exit): missing/out-of-order required headings for the page's `kind`; a `lesson`
whose `## Your turn` lacks the three tiers; a `ready-for-the-capstone` item without a lesson link; lesson with code fences but no
`### Line by line`; `{{#include}}` path or anchor that does not resolve; leftover `AUTHORING:`
placeholders; malformed question JSON; a `codeDir` in `topics.data.js` that does not exist.
Warnings: fewer than 4 quiz questions; banned words; glossary terms linked but missing from `glossary.md`.

### Code ↔ book link
Source files mark snippets with `// ANCHOR: name` / `// ANCHOR_END: name`. Lessons include them with
`{{#include ../../code/<path>:name}}`. Axum/SeaORM listings use ` ```rust,noplayground ` fences (no
Run button) followed by `📁 Full code: code/<path>` and the `cargo run -p <crate>` command — plain
` ```rust ` would get a Run button (`book.toml` sets `playground.editable = true`) that fails on a
bare snippet with no `Cargo.toml`.

The rule has no other way out: every Rust listing anywhere in a lesson — `## The idea, slowly`,
`## More examples`, `## Your turn` solutions — comes from compiled code via `{{#include}}` or
`{{#rustdoc_include}}`. Variations of a lesson's crate (an extra route, a solution) live as full
small programs in `code/topics/<slug>/examples/<name>.rs` with anchors around the part shown; a
solution's test goes in that file under `#[cfg(test)]` (CI runs `cargo test --workspace
--all-targets`, which includes examples). The only two exceptions, which `validate.mjs` allows and
nothing else:
- ` ```rust,editable ` — pure-std code that runs on the Playground as it is.
- ` ```rust,noplayground,ignore ` — deliberately broken code under `## Common mistakes`, always
  shown next to the real compiler error it produces.

The same rule covers SQL listings in Part A1: every ` ```sql ` fence must be
`{{#include ../../code/sql/<slug>/NN-name.sql}}`, and its output is
`{{#include ../../code/sql/<slug>/NN-name.out}}` in a ` ```text ` fence — never hand-typed SQL or a
pasted-in result. `tools/sql-check.mjs` is the real runner: for each `code/sql/<slug>/` it drops
and recreates a database named after the slug (`dbNameFor`, e.g. `crud-in-sql` → `crud_in_sql`),
pipes every `NN-*.sql` file into `psql` in name order, and diffs the normalized output against the
committed `NN-name.out`; `.out` files are written only by `node tools/sql-check.mjs --update
<slug>`, never hand-edited. `--check-twice` re-runs each lesson a second time and fails if the
output differs, catching nondeterministic SQL (`now()`, `random()`, `gen_random_uuid()`) before it
reaches CI. `validate.mjs` errors on a hand-typed ` ```sql ` fence; the only exception is
` ```sql,ignore ` for deliberately broken SQL shown next to its real error, mirroring
`rust,noplayground,ignore`. CI's `sql` job (`.github/workflows/code.yml`) runs
`node tools/sql-check.mjs --check-twice` against a fresh `postgres:18` service container on every
push and pull request.

### Capstone snapshots
`code/shop/step-NN/` is a full, independent crate (`shop-step-NN`). Each step is created by copying
the previous one and applying that chapter's changes, so readers can start from any chapter.
Lessons show the changed parts only, and each chapter says which files changed.

### Versions
All crate versions pinned once in `[workspace.dependencies]`, set to the latest stable releases
(Axum, Tokio, SeaORM + sea-orm-migration, serde, tower-http, tracing, argon2, jsonwebtoken, utoipa,
etc.), verified against crates.io when the plan is written. Postgres 18. MSRV = current stable Rust.
Verified 2026-09-24: Node 24 in CI (Node 20 is deprecated), axum 0.8.9, tokio 1.53.1, tower 0.5.3, sea-orm / sea-orm-migration 2.0.3, tower-http 0.7.1, tracing 0.1.44. docker-compose maps Postgres to host port 5433 so it never clashes with a locally installed Postgres on 5432.

## 5. Quality bar & testing

**A lesson is done when:**
1. All 11 sections are present and in order (validator).
2. Every non-trivial line of every listing has a Line-by-line entry.
3. Its code compiles and its tests pass in `code.yml` CI.
4. Expected output was copied from a real run.
5. The quiz has at least 4 questions and the flashcards are filled in.
6. It passed a "confused beginner" read-through: no undefined terms, no unexplained magic, no banned words.

**CI:**
- `deploy.yml`: Node 24, mdBook, `generate.mjs`, `validate.mjs`, `mdbook build` (fails on any
  mdBook `ERROR` line, since mdBook exits 0 on a broken include), publish.
- `code.yml`: Postgres 18 service container; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace --all-targets`.

## 6. Rollout phases

Each phase ends with a deployable site.
1. **Scaffold:** tooling, theme, CI, glossary, introduction, Part 0
2. **A1 Postgres + A2 Axum** (including their Build-it projects and cheat sheets)
3. **A3 SeaORM + A4** (including Notes API, URL shortener, readiness check, cheat sheets)
4. **Capstone B1–B9**
5. **Capstone B10–B18**

## 7. Out of scope
- Frontend / HTML storefront
- Re-teaching core Rust (covered by Rust for Humans)
- Translations (English only for now)
- Running Axum/SeaORM code in the browser
