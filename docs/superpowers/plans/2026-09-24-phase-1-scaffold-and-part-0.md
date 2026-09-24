# Phase 1 — Scaffold, Tooling, CI & Part 0 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a deployable "Rust Backend for Humans" mdBook site with the full generator/validator
tooling, a tested Cargo workspace wired into the book through `{{#include}}`, CI, the introduction,
the glossary, the review page, and the four Part 0 lessons. The whole 62-page roadmap is visible in
the sidebar, with unwritten pages shown as drafts.

**Architecture:** A single source of truth (`tools/topics.data.js`) drives a generator that writes
page stubs, `SUMMARY.md` and the quiz bundle. Pure functions live in `tools/lib.mjs`, unit-tested
with `node:test`. A validator enforces the spec's page formats. Every Rust listing lives in a real
crate under `code/` and reaches the page via mdBook `{{#include path:anchor}}`, so CI compiling and
testing `code/` proves the book's code works.

**Tech Stack:** mdBook 0.5.4, Node 24 (ESM, `node:test`), Rust 1.96 (edition 2024), Axum 0.8.9,
Tokio 1.53, Postgres 18 (Docker), GitHub Actions + GitHub Pages.

**Spec:** `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md`

**Out of this plan (later phase plans):** Parts A1–A4 and B content, the capstone chapter format
(decided in the Phase 4 plan), SeaORM crates, Postgres-backed tests.

## Global Constraints

- Title: "Rust Backend for Humans". Sequel to Rust for Humans (RFH), base URL
  `https://open-source-bd.github.io/rustbook-for-human/` (RFH lesson URL = `<base><category-dir>/<slug>.html`, e.g. `ownership/borrowing.html`).
- Core rule: answer the question before the reader gets confused. No undefined words (first use
  defines + links to `glossary.md`), no unexplained magic, nothing hidden (show `use` lines and
  `Cargo.toml` lines), max 3 new ideas per lesson.
- Banned prose words (validator warns): simply, just, obviously, trivially.
- Lesson headings, exact text and order: `## By the end of this lesson`, `## What & why`,
  `## The idea, slowly`, `## You might be wondering…`, `## Coming from another language?`,
  `## Common mistakes`, `## More examples`, `## Your turn`, `## Quick check`, `## Remember this`,
  `## Go deeper`. (`…` is the single Unicode ellipsis character U+2026.)
- `## Your turn` contains, in order: `### 🟢 Guided`, `### 🟡 Tweak`, `### 🔴 From scratch`, each with a `<details>` solution.
- Every listing in `## The idea, slowly` is followed by `### Line by line` (What / Why / How / Remove it and…) and, if it runs, `### Run it` (command, real output, ✅/❌ lines).
- Axum/SeaORM listings use ` ```rust ` (no Run button) + `📁 Full code: code/<path>` + run command. Only pure-std snippets may use ` ```rust,editable `.
- Pinned versions: axum 0.8.9, tokio 1.53.1, tower 0.5.3, http-body-util 0.1.5, sea-orm 2.0.3 (later phases), Postgres image `postgres:18`, mdBook 0.5.4, Node 24.
- Quiz: ≥4 questions per lesson (validator warns below 4). localStorage prefix `rbh:v1:`.
- Commits end with `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`.

## Review Focus

1. **Renamed/missing include anchor** — an author renames `// ANCHOR: app` in Rust; the reader should never see an empty code box. `validate.mjs` must error (test in Task 3).
2. **Heading-like lines inside code fences** — a bash `# comment` or a Markdown sample containing `## Your turn` inside a fence must not satisfy or break the heading check (test in Task 3).
3. **Same-origin storage collision with RFH** — both books on `open-source-bd.github.io`; quiz progress must not mix. Prefix `rbh:v1:` (check in Task 1).
4. **Links to draft pages** — a published page's "Next" pointing at a draft must render as plain "(coming soon)" text, never a dead link (test in Task 2).
5. **Reader already runs Postgres on 5432** (Homebrew, like this machine) — `docker compose up` must not fail with "port is already allocated". Map host port **5433** (Task 4), and the toolbox lesson explains it.

---

## File structure

| Path | Responsibility |
|---|---|
| `package.json` | ESM mode + `npm test` / `gen` / `validate` scripts |
| `book.toml` | mdBook config |
| `theme/retention.js`, `theme/retention.css` | Quiz/flashcard widget copied from RFH, prefix `rbh:v1:` |
| `tools/topics.data.js` | All 62 pages + 4 cheat sheets (`status: "published" \| "draft"`) |
| `tools/lib.mjs` | Pure functions: parts, paths, stubs, SUMMARY, bundle, page/bank checks |
| `tools/generate.mjs` | Thin I/O wrapper over `lib.mjs` |
| `tools/validate.mjs` | Thin I/O wrapper over `lib.mjs` |
| `tools/test/lib.test.mjs` | Unit tests for `lib.mjs` |
| `code/Cargo.toml`, `code/Cargo.lock` | Workspace + pinned deps |
| `code/topics/tour-of-the-stack/` | First crate, included by the Part 0 tour lesson |
| `docker-compose.yml`, `.env.example` | Postgres 18 on host port 5433 |
| `.github/workflows/deploy.yml`, `code.yml` | Book CI/deploy, Rust CI |
| `src/introduction.md`, `src/glossary.md`, `src/review.md` | Non-lesson pages |
| `src/part-0-start/*.md`, `questions/*.json` | Part 0 content |
| `README.md`, `CLAUDE.md` | Contributor docs |

---

### Task 1: Repo scaffold, theme and topic data

**Files:**
- Create: `package.json`, `.gitignore`, `book.toml`, `theme/retention.js`, `theme/retention.css`, `tools/topics.data.js`
- Modify: `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md` (Postgres 17 → 18; host port 5433; note sea-orm 2.0.3)

**Interfaces:**
- Produces: `tools/topics.data.js` default-exports an array of
  `{ slug, title, part, kind, level, status, outcomes?, summary?, prereq?, next?, rfhLinks?, links?, codeDir? }`
  where `part ∈ {"0","A1","A2","A3","A4","B"}`, `kind ∈ {"lesson","project","cheatsheet","checklist"}`,
  `level ∈ {"Beginner","Intermediate"}`, `status ∈ {"published","draft"}`.

- [ ] **Step 1: Create `package.json`**

```json
{
  "name": "rust-backend-for-humans",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "node --test tools/test/",
    "gen": "node tools/generate.mjs",
    "validate": "node tools/validate.mjs"
  }
}
```

- [ ] **Step 2: Create `.gitignore`**

```
book/
node_modules/
.DS_Store
code/target/
.env
```

- [ ] **Step 3: Create `book.toml`**

```toml
[book]
title = "Rust Backend for Humans"
description = "Learn to build real web backends with Rust, Axum, SeaORM and PostgreSQL — every line explained, from your first route to a production e-commerce API."
authors = ["Md Shamirul Islam"]
language = "en"
src = "src"

[output.html]
default-theme = "light"
preferred-dark-theme = "navy"
git-repository-url = ""
additional-css = ["theme/retention.css"]
additional-js = ["theme/questions.data.js", "theme/retention.js"]

[output.html.playground]
editable = true
copyable = true
line-numbers = false

[output.html.search]
enable = true
limit-results = 30
use-boolean-and = true
boost-title = 2

[output.html.fold]
enable = true
level = 1
```

- [ ] **Step 4: Copy the widget and change its storage prefix**

```bash
mkdir -p theme
cp ../rustbook-for-human/theme/retention.js ../rustbook-for-human/theme/retention.css theme/
sed -i '' 's/var KEY = "rfh:v1:";/var KEY = "rbh:v1:";/' theme/retention.js
sed -i '' '1s/Rust for Humans/Rust Backend for Humans/' theme/retention.js theme/retention.css
grep -n 'var KEY' theme/retention.js
```

Expected: `16:  var KEY = "rbh:v1:"; // bump to reset everyone's saved state`

- [ ] **Step 5: Create `tools/topics.data.js`** with this exact content (Part 0 published, everything else draft):

```js
// Single source of truth for every page in the book.
// status "draft"  -> shown greyed-out in the sidebar, no stub written, not validated.
// status "published" -> stub generated (if missing), linked in sidebar, fully validated.
const RFH = "https://open-source-bd.github.io/rustbook-for-human/";
const rfh = (label, path) => ({ label: `Rust for Humans: ${label}`, href: RFH + path });

const draft = (slug, title, part, kind = "lesson", level = "Intermediate") => ({
  slug, title, part, kind, level, status: "draft",
});

export default [
  // ---- Part 0 · Before you start ----
  {
    slug: "how-to-use-this-book",
    title: "How to use this book",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You know what you will be able to build after this book.",
      "You can check that you know enough Rust to start.",
      "You know how every lesson is laid out and how to use it.",
    ],
    summary: "What this book teaches, what Rust you need first, and how each lesson page works.",
    prereq: [],
    next: ["your-toolbox"],
    rfhLinks: [rfh("Ownership", "ownership/ownership.html"), rfh("Result and Option", "abstractions/result-and-option.html"), rfh("Async basics", "runtime-and-ecosystem/async-basics.html")],
    links: [{ label: "The Rust Book", href: "https://doc.rust-lang.org/book/", note: "The official Rust guide." }],
  },
  {
    slug: "your-toolbox",
    title: "Your toolbox",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "Rust, Docker and an HTTP client are installed and checked.",
      "A Postgres database is running in Docker on your computer.",
      "You can connect to it and run your first SQL query.",
    ],
    summary: "Install and check every tool the book uses, and start your own Postgres database.",
    prereq: ["how-to-use-this-book"],
    next: ["how-a-web-backend-works"],
    rfhLinks: [rfh("Install Rust", "start-here/install-rust.html"), rfh("Cargo basics", "start-here/cargo-basics.html")],
    links: [
      { label: "Docker Desktop", href: "https://docs.docker.com/get-started/get-docker/", note: "Install Docker." },
      { label: "Postgres Docker image", href: "https://hub.docker.com/_/postgres", note: "Options for the postgres image." },
    ],
  },
  {
    slug: "how-a-web-backend-works",
    title: "How a web backend works",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You can explain what happens between typing a URL and seeing a result.",
      "You can read an HTTP request and response line by line.",
      "You know what JSON, REST and status codes are, and can send a request with curl.",
    ],
    summary: "Clients, servers, HTTP requests and responses, JSON, REST and status codes — the ideas every later lesson uses.",
    prereq: ["your-toolbox"],
    next: ["tour-of-the-stack"],
    rfhLinks: [rfh("Serde and JSON", "runtime-and-ecosystem/serde-and-json.html")],
    links: [
      { label: "MDN: An overview of HTTP", href: "https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Overview", note: "A friendly deep dive." },
      { label: "MDN: HTTP status codes", href: "https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status", note: "Every code explained." },
    ],
  },
  {
    slug: "tour-of-the-stack",
    title: "Tour of the stack",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You know what Tokio, Axum, SeaORM and Postgres each do.",
      "You can follow one request through all four layers.",
      "You have run your first Axum server and seen it answer.",
    ],
    summary: "Meet the four tools of this book, see how one request flows through them, and run a 10-line Axum server.",
    prereq: ["how-a-web-backend-works"],
    next: ["what-is-a-database"],
    codeDir: "code/topics/tour-of-the-stack",
    rfhLinks: [rfh("Async basics", "runtime-and-ecosystem/async-basics.html"), rfh("Tokio runtime and tasks", "runtime-and-ecosystem/tokio-runtime-and-tasks.html"), rfh("Web services", "runtime-and-ecosystem/web-services.html")],
    links: [
      { label: "Axum docs", href: "https://docs.rs/axum/0.8.9/axum/", note: "Official API reference." },
      { label: "Tokio tutorial", href: "https://tokio.rs/tokio/tutorial", note: "How the async runtime works." },
      { label: "SeaORM docs", href: "https://www.sea-ql.org/SeaORM/docs/index/", note: "Official guide." },
    ],
  },

  // ---- Part A1 · PostgreSQL & SQL ----
  draft("what-is-a-database", "What is a database?", "A1", "lesson", "Beginner"),
  draft("tables-rows-and-psql", "Tables, rows and psql", "A1", "lesson", "Beginner"),
  draft("postgres-data-types", "Postgres data types", "A1", "lesson", "Beginner"),
  draft("crud-in-sql", "CRUD in SQL", "A1", "lesson", "Beginner"),
  draft("keys-and-relations", "Keys and relations", "A1", "lesson", "Beginner"),
  draft("indexes", "Indexes", "A1", "lesson", "Beginner"),
  draft("sql-transactions", "SQL transactions", "A1", "lesson", "Beginner"),
  draft("a1-build-library-schema", "Build it: a library database", "A1", "project", "Beginner"),
  draft("cheatsheet-sql", "Cheat sheet: SQL", "A1", "cheatsheet", "Beginner"),

  // ---- Part A2 · Axum ----
  draft("hello-axum", "Hello, Axum", "A2", "lesson", "Beginner"),
  draft("routes-and-methods", "Routes and HTTP methods", "A2", "lesson", "Beginner"),
  draft("handlers-and-into-response", "Handlers and IntoResponse", "A2", "lesson", "Beginner"),
  draft("path-and-query-extractors", "Path and Query extractors", "A2", "lesson", "Beginner"),
  draft("json-and-serde", "JSON with serde", "A2", "lesson", "Beginner"),
  draft("shared-state", "Shared state", "A2"),
  draft("error-handling-in-axum", "Error handling in Axum", "A2"),
  draft("middleware-and-tower-layers", "Middleware and Tower layers", "A2"),
  draft("nesting-and-modular-routers", "Nesting and modular routers", "A2"),
  draft("custom-extractors", "Custom extractors", "A2"),
  draft("input-validation", "Input validation", "A2"),
  draft("testing-handlers", "Testing handlers", "A2"),
  draft("a2-build-todo-api", "Build it: a Todo API", "A2", "project", "Beginner"),
  draft("cheatsheet-axum", "Cheat sheet: Axum", "A2", "cheatsheet"),

  // ---- Part A3 · SeaORM ----
  draft("what-is-an-orm", "What is an ORM?", "A3", "lesson", "Beginner"),
  draft("connecting-to-postgres", "Connecting to Postgres", "A3", "lesson", "Beginner"),
  draft("migrations", "Migrations", "A3"),
  draft("generating-entities", "Generating entities", "A3"),
  draft("entity-model-activemodel-column", "Entity, Model, ActiveModel and Column", "A3"),
  draft("inserting-rows", "Inserting rows", "A3"),
  draft("selecting-rows", "Selecting rows", "A3"),
  draft("update-and-delete", "Update and delete", "A3"),
  draft("relations-and-loading", "Relations and loading", "A3"),
  draft("seaorm-transactions", "Transactions in SeaORM", "A3"),
  draft("raw-sql-and-custom-selects", "Raw SQL and custom selects", "A3"),
  draft("testing-with-seaorm", "Testing with SeaORM", "A3"),
  draft("a3-build-blog-data", "Build it: a blog data layer", "A3", "project"),
  draft("cheatsheet-seaorm", "Cheat sheet: SeaORM", "A3", "cheatsheet"),

  // ---- Part A4 · Putting it together ----
  draft("config-and-env", "Config and .env", "A4"),
  draft("logging-with-tracing", "Logging with tracing", "A4"),
  draft("project-layout", "Project layout", "A4"),
  draft("mini-notes-api", "Build it: a Notes API", "A4", "project"),
  draft("a4-build-url-shortener", "Build it yourself: a URL shortener", "A4", "project"),
  draft("ready-for-the-capstone", "Ready for the capstone?", "A4", "checklist"),
  draft("cheatsheet-project-patterns", "Cheat sheet: project patterns", "A4", "cheatsheet"),

  // ---- Part B · Capstone: ShopRS ----
  // kind for capstone chapters is decided in the Phase 4 plan; drafts are not validated.
  draft("shop-01-plan", "ShopRS 1: Plan the shop", "B"),
  draft("shop-02-skeleton", "ShopRS 2: Project skeleton", "B"),
  draft("shop-03-schema", "ShopRS 3: Database schema", "B"),
  draft("shop-04-errors-and-responses", "ShopRS 4: Errors and responses", "B"),
  draft("shop-05-register", "ShopRS 5: Register users", "B"),
  draft("shop-06-login-jwt", "ShopRS 6: Log in with JWT", "B"),
  draft("shop-07-roles", "ShopRS 7: Admin roles", "B"),
  draft("shop-08-categories", "ShopRS 8: Categories", "B"),
  draft("shop-09-products", "ShopRS 9: Products", "B"),
  draft("shop-10-product-listing", "ShopRS 10: Listing, filters and search", "B"),
  draft("shop-11-cart", "ShopRS 11: Shopping cart", "B"),
  draft("shop-12-checkout", "ShopRS 12: Checkout", "B"),
  draft("shop-13-payments", "ShopRS 13: Payments", "B"),
  draft("shop-14-orders", "ShopRS 14: Orders", "B"),
  draft("shop-15-hardening", "ShopRS 15: Production hardening", "B"),
  draft("shop-16-integration-tests", "ShopRS 16: Integration tests", "B"),
  draft("shop-17-openapi", "ShopRS 17: API docs with OpenAPI", "B"),
  draft("shop-18-ship-it", "ShopRS 18: Ship it", "B"),
];
```

- [ ] **Step 6: Verify the count**

Run: `node -e "import('./tools/topics.data.js').then(m=>{const t=m.default;console.log(t.length, t.filter(x=>x.kind!=='cheatsheet').length)})"`
Expected: `66 62`

- [ ] **Step 7: Update the spec** — in `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md` replace every `Postgres 17` with `Postgres 18`, and after the `### Versions` paragraph add:
`Verified 2026-09-24: Node 24 in CI (Node 20 is deprecated), axum 0.8.9, tokio 1.53.1, tower 0.5.3, sea-orm / sea-orm-migration 2.0.3, tower-http 0.7.1, tracing 0.1.44. docker-compose maps Postgres to host port 5433 so it never clashes with a locally installed Postgres on 5432.`

- [ ] **Step 8: Commit**

```bash
git add package.json .gitignore book.toml theme tools/topics.data.js docs/superpowers/specs
git commit -m "chore: scaffold book config, widget theme and topic roadmap

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: Generator — stubs, SUMMARY and quiz bundle

**Files:**
- Create: `tools/lib.mjs`, `tools/generate.mjs`, `tools/test/lib.test.mjs`, `src/review.md`

**Interfaces:**
- Consumes: `tools/topics.data.js` (Task 1).
- Produces (exported from `tools/lib.mjs`):
  - `PARTS: Array<{ id: string, title: string, dir: string }>`
  - `HEADINGS: { lesson: string[], project: string[], cheatsheet: string[], checklist: string[] }`
  - `TIERS: string[]` — the three `### ` tier headings
  - `pagePath(topic) -> string` — e.g. `"part-0-start/tour-of-the-stack.md"`
  - `stubFor(topic, bySlug: Map<string, topic>) -> string`
  - `summaryFor(topics) -> string`
  - `bundleFor(topics, banks: Record<string, object>) -> string`
  - `questionStub(topic) -> string`

- [ ] **Step 1: Write the failing tests** — create `tools/test/lib.test.mjs`:

```js
import { test } from "node:test";
import assert from "node:assert/strict";
import { PARTS, HEADINGS, TIERS, pagePath, stubFor, summaryFor, bundleFor, questionStub } from "../lib.mjs";

const pub = (o) => ({ kind: "lesson", level: "Beginner", status: "published", part: "0",
  outcomes: ["a", "b"], summary: "s", prereq: [], next: [], rfhLinks: [], links: [], ...o });
const map = (ts) => new Map(ts.map((t) => [t.slug, t]));

test("pagePath uses the part directory", () => {
  assert.equal(pagePath(pub({ slug: "x", part: "A2" })), "a2-axum/x.md");
  assert.equal(PARTS.map((p) => p.id).join(","), "0,A1,A2,A3,A4,B");
});

test("lesson stub has every lesson heading in order and the tiers", () => {
  const t = pub({ slug: "hello", title: "Hello" });
  const md = stubFor(t, map([t]));
  let at = -1;
  for (const h of HEADINGS.lesson) {
    const i = md.indexOf(`\n${h}\n`);
    assert.ok(i > at, `heading ${h} missing or out of order`);
    at = i;
  }
  for (const tier of TIERS) assert.ok(md.includes(tier), tier);
  assert.ok(md.includes('<div class="quiz" data-topic="hello"></div>'));
  assert.ok(md.includes("AUTHORING:"));
});

test("next link to a draft renders as coming-soon text, not a link", () => {
  const a = pub({ slug: "a", title: "A", next: ["b", "c"] });
  const b = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const c = pub({ slug: "c", title: "C", part: "A1" });
  const md = stubFor(a, map([a, b, c]));
  assert.ok(md.includes("- B (coming soon)"));
  assert.ok(!md.includes("](../a1-postgres/b.md)"));
  assert.ok(md.includes("- [C](../a1-postgres/c.md)"));
});

test("project stub uses project headings", () => {
  const t = pub({ slug: "p", title: "P", kind: "project" });
  const md = stubFor(t, map([t]));
  for (const h of HEADINGS.project) assert.ok(md.includes(`\n${h}\n`), h);
  assert.ok(!md.includes("## Quick check"));
});

test("summary links published pages and shows drafts as draft chapters", () => {
  const a = pub({ slug: "a", title: "A" });
  const b = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const s = summaryFor([a, b]);
  assert.ok(s.startsWith("# Summary\n\n[Introduction](introduction.md)\n[Glossary](glossary.md)\n"));
  assert.ok(s.includes("# Part 0 · Before you start\n\n- [A](part-0-start/a.md)\n"));
  assert.ok(s.includes("# Part A1 · PostgreSQL & SQL\n\n- [B]()\n"));
  assert.ok(s.trimEnd().endsWith("[Review & flashcards](review.md)"));
});

test("bundle includes only published lessons", () => {
  const a = pub({ slug: "a", title: "A" });
  const p = pub({ slug: "p", title: "P", kind: "project" });
  const d = pub({ slug: "d", title: "D", status: "draft" });
  const out = bundleFor([a, p, d], { a: { quiz: [1] } });
  assert.ok(out.startsWith("// AUTO-GENERATED"));
  assert.ok(out.includes('window.RUST_QUESTIONS = {"a":{"quiz":[1]}};'));
  assert.ok(out.includes('window.RUST_TOPIC_ORDER = [{"slug":"a","title":"A","category":"Part 0 · Before you start"}];'));
});

test("questionStub is valid JSON with empty banks", () => {
  const q = JSON.parse(questionStub(pub({ slug: "a", title: "A" })));
  assert.deepEqual(q, { topic: "a", title: "A", quiz: [], flashcards: [] });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm test`
Expected: FAIL — `Cannot find module '.../tools/lib.mjs'`

- [ ] **Step 3: Write `tools/lib.mjs` (generator half)**

```js
// lib.mjs — pure functions shared by generate.mjs and validate.mjs (no file I/O here).

export const PARTS = [
  { id: "0", title: "Part 0 · Before you start", dir: "part-0-start" },
  { id: "A1", title: "Part A1 · PostgreSQL & SQL", dir: "a1-postgres" },
  { id: "A2", title: "Part A2 · Axum", dir: "a2-axum" },
  { id: "A3", title: "Part A3 · SeaORM", dir: "a3-seaorm" },
  { id: "A4", title: "Part A4 · Putting it together", dir: "a4-together" },
  { id: "B", title: "Part B · Capstone: ShopRS", dir: "b-capstone" },
];
const partOf = (t) => PARTS.find((p) => p.id === t.part);

export const HEADINGS = {
  lesson: [
    "## By the end of this lesson",
    "## What & why",
    "## The idea, slowly",
    "## You might be wondering…",
    "## Coming from another language?",
    "## Common mistakes",
    "## More examples",
    "## Your turn",
    "## Quick check",
    "## Remember this",
    "## Go deeper",
  ],
  project: [
    "## What you'll build",
    "## What you need to know",
    "## The spec",
    "## Build it, step by step",
    "## Reference solution, line by line",
    "## Stretch goals",
    "## Remember this",
  ],
  cheatsheet: [],
  checklist: [],
};

export const TIERS = ["### 🟢 Guided", "### 🟡 Tweak", "### 🔴 From scratch"];

export const pagePath = (t) => `${partOf(t).dir}/${t.slug}.md`;

const bullets = (xs) => (xs || []).map((x) => `- ${x}`).join("\n");
const authoring = (what) => `<!-- AUTHORING: ${what} -->`;

function linkTo(slug, bySlug) {
  const t = bySlug.get(slug);
  if (!t) return null;
  return t.status === "published" ? `- [${t.title}](../${pagePath(t)})` : `- ${t.title} (coming soon)`;
}

function goDeeper(t, bySlug) {
  const parts = [];
  const refs = [...(t.rfhLinks || []), ...(t.links || [])]
    .map((l) => `- [${l.label}](${l.href})${l.note ? ` — ${l.note}` : ""}`);
  if (refs.length) parts.push(refs.join("\n"));
  const next = (t.next || []).map((s) => linkTo(s, bySlug)).filter(Boolean);
  if (next.length) parts.push(`**Next:**\n\n${next.join("\n")}`);
  return parts.join("\n\n") || authoring("official docs + Rust for Humans links");
}

function lessonStub(t, bySlug) {
  const tier = (h) => `${h}\n\n${authoring("exercise")}\n\n<details><summary>Solution</summary>\n\n${authoring("solution")}\n\n</details>`;
  const body = {
    "## By the end of this lesson": bullets(t.outcomes) || authoring("2–3 concrete outcomes"),
    "## What & why": `${t.summary || ""}\n\n${authoring("real-world problem + plain-language analogy")}`,
    "## The idea, slowly": `${authoring("small steps; after each listing add ### Line by line and ### Run it")}`,
    "## You might be wondering…": authoring("pre-emptive FAQ: the questions a confused beginner asks here"),
    "## Coming from another language?": authoring("Express / Flask / Spring / Go equivalents"),
    "## Common mistakes": authoring("real errors, what they mean, the fix"),
    "## More examples": authoring("4 × (### title, one-line hook, code)"),
    "## Your turn": TIERS.map(tier).join("\n\n"),
    "## Quick check": `<div class="quiz" data-topic="${t.slug}"></div>`,
    "## Remember this": authoring("3–5 takeaways"),
    "## Go deeper": goDeeper(t, bySlug),
  };
  return HEADINGS.lesson.map((h) => `${h}\n\n${body[h]}`).join("\n\n");
}

export function stubFor(t, bySlug) {
  const head = `# ${t.title}\n\n> **${t.level}** · ${partOf(t).title}`;
  let body;
  if (t.kind === "lesson") body = lessonStub(t, bySlug);
  else if (t.kind === "project") body = HEADINGS.project.map((h) => `${h}\n\n${authoring(h.slice(3))}`).join("\n\n");
  else if (t.kind === "checklist") body = `${authoring("- [ ] items, each linking to the lesson that teaches it")}`;
  else body = `${authoring("one-page How do I…? list, each item links to its lesson")}`;
  return `${head}\n\n${body}\n`;
}

export function summaryFor(topics) {
  let s = "# Summary\n\n[Introduction](introduction.md)\n[Glossary](glossary.md)\n";
  for (const p of PARTS) {
    const inPart = topics.filter((t) => t.part === p.id);
    if (!inPart.length) continue;
    s += `\n# ${p.title}\n\n`;
    for (const t of inPart) s += t.status === "published" ? `- [${t.title}](${pagePath(t)})\n` : `- [${t.title}]()\n`;
  }
  return s + "\n---\n\n[Review & flashcards](review.md)\n";
}

const quizzable = (t) => t.status === "published" && t.kind === "lesson";

export function bundleFor(topics, banks) {
  const live = topics.filter(quizzable);
  const q = Object.fromEntries(live.filter((t) => banks[t.slug]).map((t) => [t.slug, banks[t.slug]]));
  const order = live.map((t) => ({ slug: t.slug, title: t.title, category: partOf(t).title }));
  return (
    "// AUTO-GENERATED by tools/generate.mjs — do not edit by hand.\n" +
    `window.RUST_QUESTIONS = ${JSON.stringify(q)};\n` +
    `window.RUST_TOPIC_ORDER = ${JSON.stringify(order)};\n`
  );
}

export const questionStub = (t) =>
  JSON.stringify({ topic: t.slug, title: t.title, quiz: [], flashcards: [] }, null, 2) + "\n";

export { quizzable };
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test`
Expected: PASS, 7 tests.

- [ ] **Step 5: Write `tools/generate.mjs`**

```js
// generate.mjs — writes page stubs, questions/<slug>.json stubs, src/SUMMARY.md and
// theme/questions.data.js from tools/topics.data.js.
// Run:  node tools/generate.mjs          (safe: never overwrites existing pages)
//       node tools/generate.mjs --force  (overwrite page stubs)
import fs from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import topics from "./topics.data.js";
import { pagePath, stubFor, summaryFor, bundleFor, questionStub, quizzable } from "./lib.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const FORCE = process.argv.includes("--force");
const bySlug = new Map(topics.map((t) => [t.slug, t]));
let wrote = 0, kept = 0;

for (const t of topics.filter((t) => t.status === "published")) {
  const file = join(ROOT, "src", pagePath(t));
  fs.mkdirSync(dirname(file), { recursive: true });
  if (FORCE || !fs.existsSync(file)) { fs.writeFileSync(file, stubFor(t, bySlug)); wrote++; } else kept++;
  if (quizzable(t)) {
    const q = join(ROOT, "questions", `${t.slug}.json`);
    fs.mkdirSync(dirname(q), { recursive: true });
    if (!fs.existsSync(q)) fs.writeFileSync(q, questionStub(t));
  }
}

fs.writeFileSync(join(ROOT, "src", "SUMMARY.md"), summaryFor(topics));

const banks = {};
for (const t of topics.filter(quizzable)) {
  try { banks[t.slug] = JSON.parse(fs.readFileSync(join(ROOT, "questions", `${t.slug}.json`), "utf8")); }
  catch { /* validate.mjs reports missing/invalid banks */ }
}
fs.writeFileSync(join(ROOT, "theme", "questions.data.js"), bundleFor(topics, banks));

console.log(`pages: ${wrote} written, ${kept} kept | SUMMARY.md + questions.data.js regenerated`);
```

- [ ] **Step 6: Create `src/review.md`** (copied from RFH, adapted):

```markdown
# Review & flashcards

This is your anti-forgetting page. It pulls the key questions from every lesson into one shuffled
deck. Cards you mark **"Shaky"** come back first next time, so your weak spots get the most
practice. Everything is saved in your browser — no account, no login.

Try to answer each card in your head *before* you flip it. The effort of remembering is what
builds the memory.

<div id="flashcards" data-review="all"></div>
```

- [ ] **Step 7: Run the generator and build**

Run: `node tools/generate.mjs && ls src/part-0-start && head -12 src/SUMMARY.md`
Expected: `pages: 4 written, 0 kept | …`; four `.md` files; SUMMARY starting with `# Summary`, Introduction, Glossary, `# Part 0 · Before you start`.

Create placeholder non-lesson pages so mdBook can build (they are filled in Task 6):
`printf '# Rust Backend for Humans\n' > src/introduction.md; printf '# Glossary\n' > src/glossary.md`

Run: `mdbook build 2>&1 | tail -3`
Expected: build succeeds (`HTML book written to .../book`), no errors. Drafts render greyed out in the sidebar.

- [ ] **Step 8: Commit**

```bash
git add tools/lib.mjs tools/generate.mjs tools/test src questions theme/questions.data.js
git commit -m "feat(tools): generator for page stubs, sidebar and quiz bundle

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: Validator

**Files:**
- Modify: `tools/lib.mjs` (append check functions), `tools/test/lib.test.mjs` (append tests)
- Create: `tools/validate.mjs`

**Interfaces:**
- Consumes: `HEADINGS`, `TIERS`, `pagePath`, `quizzable` from Task 2.
- Produces (exported from `tools/lib.mjs`):
  - `stripFences(md: string) -> string` — fenced blocks replaced by blank lines (line count preserved)
  - `headingId(text: string) -> string` — mdBook's heading id algorithm
  - `glossaryIds(md: string) -> Set<string>`
  - `checkPage({ topic, md, mdFile, root, glossary: Set<string>, exists: (p) => boolean, read: (p) => string }) -> { errors: string[], warnings: string[] }`
  - `checkBank(topic, bank) -> { errors: string[], warnings: string[] }`
  - `checkTopics(topics) -> string[]` (errors)

- [ ] **Step 1: Write the failing tests** — append to `tools/test/lib.test.mjs`:

```js
import { stripFences, headingId, glossaryIds, checkPage, checkBank, checkTopics } from "../lib.mjs";

const T = pub({ slug: "hello", title: "Hello" });
const good = () => stubFor(T, map([T]))
  .replace(/<!-- AUTHORING:[^>]*-->/g, "text")
  .replace("## The idea, slowly\n\ntext", "## The idea, slowly\n\n```rust\n{{#include ../../code/x/src/main.rs:app}}\n```\n\n### Line by line\n\nok");
const ctx = (md, files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n" }) => ({
  topic: T, md, mdFile: "/r/src/part-0-start/hello.md", root: "/r",
  glossary: new Set(["handler"]),
  exists: (p) => p in files || p === "/r/code/x",
  read: (p) => files[p],
});

test("a complete lesson passes", () => {
  const r = checkPage(ctx(good()));
  assert.deepEqual(r.errors, []);
});

test("missing anchor is an error", () => {
  const r = checkPage(ctx(good(), { "/r/code/x/src/main.rs": "fn a(){}\n" }));
  assert.ok(r.errors.some((e) => e.includes('anchor "app"')), r.errors.join("\n"));
});

test("missing include file is an error", () => {
  const r = checkPage(ctx(good(), {}));
  assert.ok(r.errors.some((e) => e.includes("include file not found")));
});

test("line-range include is a warning", () => {
  const md = good().replace(":app}}", ":1:3}}");
  assert.ok(checkPage(ctx(md)).warnings.some((w) => w.includes("use an ANCHOR")));
});

test("headings inside code fences do not count", () => {
  const md = good().replace("\n## Common mistakes\n", "\n```md\n## Common mistakes\n```\n");
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes('"## Common mistakes"')), r.errors.join("\n"));
});

test("out-of-order headings are an error", () => {
  const md = good().replace("## What & why", "## TEMP").replace("## Common mistakes", "## What & why").replace("## TEMP", "## Common mistakes");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("order")));
});

test("missing tier and missing Line by line are errors", () => {
  const md = good().replace("### 🟡 Tweak", "### Tweak").replace("### Line by line", "### Notes");
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes("🟡 Tweak")));
  assert.ok(r.errors.some((e) => e.includes("Line by line")));
});

test("AUTHORING placeholders are errors; banned words in prose warn, not in code", () => {
  const md = good() + "\n<!-- AUTHORING: x -->\nYou simply run it.\n```bash\n# just a comment\n```\n`just`\n";
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes("AUTHORING")));
  assert.equal(r.warnings.filter((w) => w.includes("banned word")).length, 1);
  assert.ok(r.warnings.some((w) => w.includes('"simply"')));
});

test("unknown glossary anchor warns", () => {
  const md = good() + "\nA [router](../glossary.md#router) and a [handler](../glossary.md#handler).\n";
  const w = checkPage(ctx(md)).warnings.filter((x) => x.includes("glossary"));
  assert.equal(w.length, 1);
  assert.ok(w[0].includes("#router"));
});

test("checklist items must link", () => {
  const t = pub({ slug: "ready", title: "R", kind: "checklist" });
  const md = "# R\n\n- [ ] I can add a route ([lesson](../a2-axum/x.md))\n- [ ] I can write a migration\n";
  const r = checkPage({ ...ctx(md), topic: t });
  assert.equal(r.errors.length, 1);
  assert.ok(r.errors[0].includes("I can write a migration"));
});

test("missing codeDir is an error", () => {
  const r = checkPage({ ...ctx(good()), topic: { ...T, codeDir: "code/nope" } });
  assert.ok(r.errors.some((e) => e.includes("codeDir")));
});

test("headingId matches mdBook", () => {
  assert.equal(headingId("Status code"), "status-code");
  assert.equal(headingId("ORM (Object-Relational Mapper)"), "orm-object-relational-mapper");
  assert.deepEqual([...glossaryIds("# Glossary\n\n### Handler\n\n### Status code\n")], ["handler", "status-code"]);
});

test("stripFences keeps line count", () => {
  const md = "a\n```\nb\nc\n```\nd";
  assert.equal(stripFences(md).split("\n").length, md.split("\n").length);
  assert.ok(!stripFences(md).includes("b"));
});

test("checkBank: bad answer index errors, <4 questions warns", () => {
  const r = checkBank(T, { quiz: [{ q: "q", options: ["a", "b"], answer: 2, explain: "e" }], flashcards: [{ front: "f", back: "b" }] });
  assert.ok(r.errors.some((e) => e.includes("answer index")));
  assert.ok(r.warnings.some((w) => w.includes("fewer than 4")));
});

test("checkTopics catches duplicate slugs and bad parts", () => {
  const errs = checkTopics([pub({ slug: "a", title: "A" }), pub({ slug: "a", title: "A2" }), pub({ slug: "z", title: "Z", part: "X" })]);
  assert.ok(errs.some((e) => e.includes("duplicate slug a")));
  assert.ok(errs.some((e) => e.includes("unknown part X")));
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm test`
Expected: FAIL — `SyntaxError: The requested module '../lib.mjs' does not provide an export named 'stripFences'`

- [ ] **Step 3: Append the checks to `tools/lib.mjs`**

```js
// ---------------------------------------------------------------------------
// Validation
import { dirname as _dirname, resolve as _resolve } from "node:path";

const FENCE = /^ {0,3}(```|~~~)/;

export function stripFences(md) {
  let inFence = false;
  return md.split("\n").map((line) => {
    if (FENCE.test(line)) { inFence = !inFence; return ""; }
    return inFence ? "" : line;
  }).join("\n");
}

export function headingId(text) {
  let id = "";
  for (const ch of text) {
    if (/[\p{L}\p{N}_-]/u.test(ch)) id += ch.toLowerCase();
    else if (/\s/.test(ch)) id += "-";
  }
  return id;
}

export function glossaryIds(md) {
  const ids = new Set();
  for (const m of stripFences(md).matchAll(/^#{2,4} +(.+)$/gm)) ids.add(headingId(m[1].trim()));
  return ids;
}

const BANNED = ["simply", "just", "obviously", "trivially"];

function sectionBody(prose, heading) {
  const start = prose.indexOf(`\n${heading}\n`);
  if (start < 0) return "";
  const rest = prose.slice(start + heading.length + 2);
  const end = rest.search(/^## /m);
  return end < 0 ? rest : rest.slice(0, end);
}

export function checkPage({ topic: t, md, mdFile, root, glossary, exists, read }) {
  const errors = [], warnings = [];
  const prose = stripFences(md);
  const required = HEADINGS[t.kind] || [];

  // 1. required headings: present, and in order
  const found = [...prose.matchAll(/^## .+$/gm)].map((m) => m[0].trim());
  for (const h of required) if (!found.includes(h)) errors.push(`missing section "${h}"`);
  const seq = found.filter((h) => required.includes(h));
  if (required.every((h) => found.includes(h)) && seq.join("|") !== required.join("|"))
    errors.push(`sections out of order: expected ${required.join(" → ")}`);

  // 2. lesson-only rules
  if (t.kind === "lesson") {
    if (!md.includes(`<div class="quiz" data-topic="${t.slug}"></div>`)) errors.push("quiz mount div missing or wrong slug");
    const turn = sectionBody(prose, "## Your turn");
    let at = -1;
    for (const tier of TIERS) {
      const i = turn.indexOf(tier);
      if (i <= at) errors.push(`"## Your turn" is missing "${tier}" (or it is out of order)`);
      else at = i;
    }
    if (md.split("\n").some((l) => FENCE.test(l)) && !/^### Line by line$/m.test(prose))
      errors.push('page has code but no "### Line by line" section');
  }

  // 3. checklist items must link to a lesson
  if (t.kind === "checklist")
    for (const line of prose.split("\n"))
      if (/^- \[ \]/.test(line) && !line.includes("]("))
        errors.push(`checklist item has no lesson link: ${line.slice(6).trim()}`);
  if (t.kind === "cheatsheet" && !prose.includes("](")) warnings.push("cheat sheet links to no lessons");

  // 4. includes resolve (scan raw md: includes live inside fences)
  for (const m of md.matchAll(/\{\{#include\s+([^}\s]+)\s*\}\}/g)) {
    const [rel, ...rest] = m[1].split(":");
    const file = _resolve(_dirname(mdFile), rel);
    if (!exists(file)) { errors.push(`include file not found: ${rel}`); continue; }
    if (!rest.length) continue;
    if (/^\d/.test(rest[0])) { warnings.push(`include ${m[1]} uses line numbers — use an ANCHOR so edits can't shift it`); continue; }
    const name = rest[0];
    const src = read(file);
    const esc = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    if (!new RegExp(`ANCHOR:\\s*${esc}\\b`).test(src) || !new RegExp(`ANCHOR_END:\\s*${esc}\\b`).test(src))
      errors.push(`anchor "${name}" not found (ANCHOR + ANCHOR_END) in ${rel}`);
  }

  // 5. leftovers, banned words, glossary links, codeDir
  if (md.includes("AUTHORING:")) errors.push("leftover AUTHORING placeholder");
  const plain = prose.replace(/`[^`\n]*`/g, "");
  for (const w of BANNED) {
    const n = (plain.match(new RegExp(`\\b${w}\\b`, "gi")) || []).length;
    if (n) warnings.push(`banned word "${w}" used ${n}×`);
  }
  for (const m of md.matchAll(/glossary\.md#([\w-]+)/g))
    if (!glossary.has(m[1])) warnings.push(`glossary link #${m[1]} has no matching glossary heading`);
  if (t.codeDir && !exists(_resolve(root, t.codeDir))) errors.push(`codeDir does not exist: ${t.codeDir}`);

  return { errors, warnings };
}

export function checkBank(t, bank) {
  const errors = [], warnings = [];
  const quiz = Array.isArray(bank.quiz) ? bank.quiz : [];
  const cards = Array.isArray(bank.flashcards) ? bank.flashcards : [];
  if (quiz.length < 4) warnings.push(`quiz has fewer than 4 questions (${quiz.length})`);
  if (!cards.length) warnings.push("no flashcards");
  quiz.forEach((q, i) => {
    if (!q.q) errors.push(`quiz[${i}] missing question text`);
    if (!Array.isArray(q.options) || q.options.length < 2) errors.push(`quiz[${i}] needs at least 2 options`);
    if (typeof q.answer !== "number" || q.answer < 0 || q.answer >= (q.options || []).length)
      errors.push(`quiz[${i}] answer index out of range`);
    if (!q.explain) warnings.push(`quiz[${i}] has no explanation`);
  });
  cards.forEach((c, i) => { if (!c.front || !c.back) errors.push(`flashcard[${i}] missing front/back`); });
  return { errors, warnings };
}

export function checkTopics(topics) {
  const errors = [], seen = new Set();
  for (const t of topics) {
    if (seen.has(t.slug)) errors.push(`duplicate slug ${t.slug}`);
    seen.add(t.slug);
    if (!PARTS.some((p) => p.id === t.part)) errors.push(`${t.slug}: unknown part ${t.part}`);
    if (!(t.kind in HEADINGS)) errors.push(`${t.slug}: unknown kind ${t.kind}`);
    if (t.status === "published" && t.kind === "lesson" && (!t.outcomes || t.outcomes.length < 2))
      errors.push(`${t.slug}: published lesson needs at least 2 outcomes`);
  }
  return errors;
}
```

Move the `import { dirname as _dirname, resolve as _resolve } from "node:path";` line to the top of `lib.mjs` (ES module imports must be top-level; they are hoisted, but keep them at the top for readers).

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test`
Expected: PASS, 22 tests. If "headings inside code fences" fails, check that `stripFences` is applied before heading matching.

- [ ] **Step 5: Write `tools/validate.mjs`**

```js
// validate.mjs — checks every published page and question bank.
// Run: node tools/validate.mjs   (exits 1 on any ERROR; warnings don't fail)
import fs from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import topics from "./topics.data.js";
import { pagePath, checkPage, checkBank, checkTopics, glossaryIds, quizzable } from "./lib.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
let errors = 0, warns = 0;
const report = (who, r) => {
  r.errors.forEach((e) => { console.log(`ERROR  ${who}: ${e}`); errors++; });
  r.warnings.forEach((w) => { console.log(`warn   ${who}: ${w}`); warns++; });
};

report("topics.data.js", { errors: checkTopics(topics), warnings: [] });

const glossaryFile = join(ROOT, "src", "glossary.md");
const glossary = fs.existsSync(glossaryFile) ? glossaryIds(fs.readFileSync(glossaryFile, "utf8")) : new Set();
if (!fs.existsSync(glossaryFile)) report("glossary.md", { errors: ["src/glossary.md missing"], warnings: [] });

const io = { exists: (p) => fs.existsSync(p), read: (p) => fs.readFileSync(p, "utf8") };
const published = topics.filter((t) => t.status === "published");
for (const t of published) {
  const mdFile = join(ROOT, "src", pagePath(t));
  if (!fs.existsSync(mdFile)) { report(t.slug, { errors: [`page missing: src/${pagePath(t)}`], warnings: [] }); continue; }
  report(t.slug, checkPage({ topic: t, md: fs.readFileSync(mdFile, "utf8"), mdFile, root: ROOT, glossary, ...io }));
  if (!quizzable(t)) continue;
  const qFile = join(ROOT, "questions", `${t.slug}.json`);
  try { report(t.slug, checkBank(t, JSON.parse(fs.readFileSync(qFile, "utf8")))); }
  catch (e) { report(t.slug, { errors: [`questions JSON missing or invalid — ${e.message}`], warnings: [] }); }
}

console.log(`\n${published.length} published pages checked (${topics.length - published.length} drafts skipped) — ${errors} error(s), ${warns} warning(s)`);
process.exit(errors ? 1 : 0);
```

- [ ] **Step 6: Run the validator against the stubs**

Run: `node tools/validate.mjs; echo "exit=$?"`
Expected: ERROR lines for each Part 0 stub (`leftover AUTHORING placeholder`, `codeDir does not exist: code/topics/tour-of-the-stack`), warnings for empty quizzes, and `exit=1`. This is correct: the stubs are not written yet.

- [ ] **Step 7: Commit**

```bash
git add tools
git commit -m "feat(tools): validator for page formats, includes, glossary and quiz banks

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: Cargo workspace, first crate and local Postgres

**Files:**
- Create: `code/Cargo.toml`, `code/topics/tour-of-the-stack/Cargo.toml`, `code/topics/tour-of-the-stack/src/main.rs`, `docker-compose.yml`, `.env.example`

**Interfaces:**
- Produces: anchors `handler`, `app`, `main` in `code/topics/tour-of-the-stack/src/main.rs` (used by the Task 8 lesson); `fn app() -> axum::Router`; server on `127.0.0.1:3000` answering `GET /` with `Hello from Axum!`. Workspace dependency names `axum`, `tokio`, `tower`, `http-body-util`. Postgres on `localhost:5433`, user `postgres`, password `postgres`, database `rbh`.

- [ ] **Step 1: Create the workspace `code/Cargo.toml`**

```toml
[workspace]
resolver = "3"
members = ["topics/*"]

[workspace.package]
edition = "2024"
publish = false

[workspace.dependencies]
axum = "0.8.9"
tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread", "net"] }
tower = { version = "0.5.3", features = ["util"] }
http-body-util = "0.1.5"
```

- [ ] **Step 2: Create `code/topics/tour-of-the-stack/Cargo.toml`**

```toml
[package]
name = "tour-of-the-stack"
version = "0.1.0"
edition.workspace = true
publish.workspace = true

[dependencies]
axum.workspace = true
tokio.workspace = true

[dev-dependencies]
tower.workspace = true
http-body-util.workspace = true
```

- [ ] **Step 3: Write the failing tests first** — create `code/topics/tour-of-the-stack/src/main.rs` with only the tests and an empty `app`:

```rust
use axum::Router;

fn app() -> Router {
    Router::new()
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn root_says_hello() {
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello from Axum!");
    }

    #[tokio::test]
    async fn unknown_path_is_404() {
        let request = Request::builder().uri("/nope").body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
```

- [ ] **Step 4: Run to verify it fails**

Run: `cd code && cargo test -p tour-of-the-stack`
Expected: `root_says_hello` FAILS (`left: 404, right: 200`); `unknown_path_is_404` passes.

- [ ] **Step 5: Write the real program** — replace everything above `#[cfg(test)]` with:

```rust
use axum::{Router, routing::get};

// ANCHOR: handler
async fn hello() -> &'static str {
    "Hello from Axum!"
}
// ANCHOR_END: handler

// ANCHOR: app
fn app() -> Router {
    Router::new().route("/", get(hello))
}
// ANCHOR_END: app

// ANCHOR: main
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("port 3000 is busy: stop the other program using it, or change the port");
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app())
        .await
        .expect("the server stopped because of an error");
}
// ANCHOR_END: main
```

- [ ] **Step 6: Run tests, lints and the server**

Run: `cd code && cargo test -p tour-of-the-stack && cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: `test result: ok. 2 passed`, no fmt diff, no clippy warnings. (If `fmt --check` reports import ordering, run `cargo fmt` and re-check — edition 2024 sorts imports differently from older styles.)

Run: `cd code && (cargo run -q -p tour-of-the-stack & sleep 3; curl -s http://127.0.0.1:3000/; echo; kill %1)`
Expected: `Listening on http://127.0.0.1:3000` then `Hello from Axum!`. Save this exact output — Task 8 pastes it into the lesson.

- [ ] **Step 7: Create `docker-compose.yml`**

```yaml
# Postgres for the whole book. Host port 5433 (not 5432) so it never clashes
# with a Postgres you may already have installed on your computer.
services:
  db:
    image: postgres:18
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: rbh
    ports:
      - "5433:5432"
    volumes:
      # Postgres 18 images keep data under /var/lib/postgresql/18/..., so mount the parent.
      - pgdata:/var/lib/postgresql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d rbh"]
      interval: 2s
      timeout: 3s
      retries: 15

volumes:
  pgdata:
```

- [ ] **Step 8: Create `.env.example`**

```
# Copy to .env (cp .env.example .env). Never commit .env.
DATABASE_URL=postgres://postgres:postgres@localhost:5433/rbh
```

- [ ] **Step 9: Verify Postgres** (requires Docker running; on this machine start OrbStack first)

Run: `docker compose up -d --wait && psql postgres://postgres:postgres@localhost:5433/rbh -c 'select version();' && docker compose down`
Expected: one row starting `PostgreSQL 18.`. Save the exact commands and output for Task 7.

- [ ] **Step 10: Re-run the validator — the codeDir error is gone**

Run: `node tools/validate.mjs | grep codeDir; echo "grep-exit=$?"`
Expected: `grep-exit=1` (no codeDir errors).

- [ ] **Step 11: Commit**

```bash
git add code docker-compose.yml .env.example
git commit -m "feat(code): cargo workspace, tour-of-the-stack crate and postgres compose

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: CI — tests, validation, Rust checks, Pages deploy

**Files:**
- Create: `.github/workflows/deploy.yml`, `.github/workflows/code.yml`

- [ ] **Step 1: Create `.github/workflows/deploy.yml`**

```yaml
name: Deploy book to GitHub Pages

on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: pages-${{ github.ref }}
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      MDBOOK_VERSION: "0.5.4"
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "24"
      - name: Install mdBook
        run: curl -sSL "https://github.com/rust-lang/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar -xz --directory=/usr/local/bin
      - name: Tool tests
        run: npm test
      - name: Generate
        run: node tools/generate.mjs
      - name: Validate
        run: node tools/validate.mjs
      - name: Build book
        run: mdbook build
      - uses: actions/upload-pages-artifact@v3
        if: github.ref == 'refs/heads/main'
        with:
          path: ./book

  deploy:
    if: github.ref == 'refs/heads/main'
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Create `.github/workflows/code.yml`**

```yaml
name: Book code compiles and passes tests

on:
  push:
    branches: [main]
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:18
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: rbh
        ports:
          - 5433:5432
        options: >-
          --health-cmd "pg_isready -U postgres -d rbh"
          --health-interval 2s --health-timeout 3s --health-retries 15
    env:
      DATABASE_URL: postgres://postgres:postgres@localhost:5433/rbh
    defaults:
      run:
        working-directory: code
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: code
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
```

- [ ] **Step 3: Lint the YAML locally**

Run: `node -e "for (const f of ['deploy','code']) { const s=require('fs').readFileSync('.github/workflows/'+f+'.yml','utf8'); if (/\t/.test(s)) throw new Error(f+' has tabs'); console.log(f, 'ok', s.split('\n').length, 'lines') }"`
Expected: `deploy ok …` and `code ok …`. (Full verification happens on the first push; see Task 9.)

- [ ] **Step 4: Commit**

```bash
git add .github
git commit -m "ci: book validate/build/deploy and rust fmt/clippy/test workflows

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 6: Introduction and glossary

**Files:**
- Modify: `src/introduction.md`, `src/glossary.md` (replace Task 2 placeholders)

**Interfaces:**
- Produces: glossary headings (`### Term`) whose ids Part 0 lessons link to as `../glossary.md#<id>`: `backend`, `client`, `server`, `http`, `request`, `response`, `status-code`, `json`, `rest`, `endpoint`, `route`, `handler`, `port`, `localhost`, `crate`, `framework`, `async`, `runtime`, `database`, `sql`, `orm`, `docker`, `container`, `environment-variable`.

- [ ] **Step 1: Write `src/glossary.md`.** Structure:

```markdown
# Glossary

Every technical word in this book, in plain language. Lessons link here the first time they use a
word. Terms are in A–Z order.

### Async

Code that can pause while it waits (for the network, the database…) so the computer can do other
work in the meantime. In Rust you mark such functions `async fn` and wait for them with `.await`.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)
```

Write one entry for each of the 24 ids listed under **Interfaces**, in A–Z order, same shape: a
heading whose text produces that id (e.g. `### Status code`, `### HTTP`, `### Environment variable`),
a definition of 1–2 sentences with no undefined words (define using only everyday words or other
glossary terms, linked as `[route](#route)`), and a **First used in:** link to the Part 0 lesson that
introduces it. Example definitions to hit: *Handler* — "the function that runs when a request
matches a route; it receives the request's data and returns the response"; *Port* — "a numbered
door on a computer; one program listens behind each door, e.g. our server on 3000 and Postgres on
5433"; *ORM* — "a library that lets you work with database rows as normal Rust structs instead of
writing SQL strings by hand".

- [ ] **Step 2: Write `src/introduction.md`.** Required content, in this order:
  1. `# Rust Backend for Humans` + a 3-sentence hook: you'll go from "what is a server?" to a deployed e-commerce API, every line explained, nothing assumed.
  2. `## What you'll build` — the 5 Part A projects (library database, Todo API, blog data layer, Notes API, URL shortener) and the ShopRS capstone (products, cart, checkout, payments, admin), one line each.
  3. `## Is this book for me?` — yes if you finished Rust for Humans or know ownership, `Result`, traits and `async` basics; links to those four RFH lessons (`ownership/ownership.html`, `abstractions/result-and-option.html`, `abstractions/traits-basics.html`, `runtime-and-ecosystem/async-basics.html`). "Coming from JS/Python/Java/Go? Good — every lesson has a box translating to what you know."
  4. `## How every lesson works` — a table of the 11 sections with one line each on how to use it (reuse the spec §3 table wording), then a sample Line-by-line entry (the `Router::new()` example from the spec).
  5. `## The path` — the parts in order with one sentence each, and the confidence ladder (guided → half-guided → independent → readiness check → capstone).
  6. `## How to not forget` — Quick check + the [Review & flashcards](review.md) page.
  7. `## Start here` → link to [How to use this book](part-0-start/how-to-use-this-book.md).

- [ ] **Step 3: Validate glossary ids and build**

Run: `node -e "import('./tools/lib.mjs').then(async l=>{const ids=l.glossaryIds(require('fs').readFileSync('src/glossary.md','utf8'));const need='backend client server http request response status-code json rest endpoint route handler port localhost crate framework async runtime database sql orm docker container environment-variable'.split(' ');const miss=need.filter(i=>!ids.has(i));console.log(miss.length?'MISSING '+miss:'all 24 ids present')})"`

(`node -e` runs as CommonJS even with `"type": "module"`, so `require` and `import()` both work here.)
Expected: `all 24 ids present`

Run: `mdbook build 2>&1 | tail -1`
Expected: build succeeds.

- [ ] **Step 4: Banned-word and undefined-word read-through** — read both pages as a confused beginner; `grep -niwE 'simply|just|obviously|trivially' src/introduction.md src/glossary.md` must print nothing.

- [ ] **Step 5: Commit**

```bash
git add src/introduction.md src/glossary.md
git commit -m "docs: introduction and glossary

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Tasks 7–8: Part 0 lessons — shared rules

Each Part 0 lesson is a task of its own (Task 7: lessons 1–2, Task 8: lessons 3–4, one commit per
lesson). For **every** lesson:

1. Edit the generated stub `src/part-0-start/<slug>.md`: replace every `<!-- AUTHORING: … -->` with real content following the Global Constraints and the brief below.
2. Write `questions/<slug>.json` with **exactly** the quiz in the brief plus the flashcards listed.
3. Every command in `### Run it` must be run for real on this machine and its output pasted exactly (trim long output with a line saying `…`).
4. Verify: `node tools/generate.mjs && node tools/validate.mjs 2>&1 | grep -E "<slug>|checked"` → no `ERROR` lines for the slug, and no `banned word` warnings. `mdbook build` succeeds.
5. Confused-beginner read-through: every technical word either defined in-line or linked to `../glossary.md#id` on first use; every code line explained.
6. Commit: `docs(part-0): <slug> lesson` with the Co-Authored-By trailer.

### Task 7: Lessons "How to use this book" and "Your toolbox"

**Files:**
- Modify: `src/part-0-start/how-to-use-this-book.md`, `src/part-0-start/your-toolbox.md`
- Create/modify: `questions/how-to-use-this-book.json`, `questions/your-toolbox.json`

**Interfaces:**
- Consumes: glossary ids (Task 6), `docker-compose.yml` + `.env.example` (Task 4).

- [ ] **Step 1: Write `how-to-use-this-book`** per this brief:
  - *What & why:* backend = the part of an app you can't see (analogy: restaurant kitchen vs dining room). Why Rust for backends: fast, cheap to run, and the compiler catches bugs before users do.
  - *The idea, slowly:* (a) the Rust you need — a self-check list of 6 items (ownership & borrowing, `struct`/`enum`, `Result` + `?`, traits, closures, `async fn`/`.await`), each linking to its RFH lesson; (b) listing: `rustc --version` and `cargo --version` as a ` ```bash ` block, with `### Line by line` (what `rustc` and `cargo` are, why we check versions, how to read `rustc 1.96.0 (…)`) and `### Run it` with real output and ✅ "any version 1.85 or newer works (edition 2024)", ❌ "`command not found` → install via RFH Install Rust"; (c) the anatomy of a lesson page (the 11 sections).
  - *You might be wondering…:* "Do I need to know SQL?" (no, Part A1 teaches it) · "Do I need a frontend?" (no, we test with curl) · "Why not Node/Python?" · "What if I get stuck?" (Common mistakes + full code in `code/`).
  - *Coming from another language?:* table — Node (Express + Prisma), Python (FastAPI + SQLAlchemy), Java (Spring Boot + JPA), Go (net/http + GORM) → this book (Axum + SeaORM).
  - *Common mistakes:* skipping Part A; copying code without running it; reading without doing the Your turn.
  - *More examples:* 4 × short "how to use a Line-by-line block" / "how to read a Run it block" / "how to use the full-code link" / "how to use the Review page" — each with a tiny illustrative code fence.
  - *Your turn:* 🟢 run the two version commands; 🟡 run `cargo new hello-backend` and `cargo run`; 🔴 write down in one sentence each what you want to build after this book (solution: sample answers).
  - Quiz (write to JSON):
    1. "What is a backend?" options ["The part of an app that runs on a server and handles data and rules", "The CSS of a website", "The bottom half of a web page", "A type of database"], answer 0, explain "The backend runs on a server: it receives requests, applies the rules, talks to the database and sends back responses."
    2. "Which Rust topic should you review before starting this book?" options ["Unsafe Rust", "Ownership and borrowing", "FFI", "Build scripts"], answer 1, explain "Ownership shows up on every page of backend code; unsafe, FFI and build scripts are not used in this book."
    3. "What does the '### Line by line' part of a lesson give you?" options ["A summary video", "What, why and how for each line of the code above it", "Only the output of the program", "A list of links"], answer 1, explain "Each non-trivial line gets What, Why, How and 'Remove it and…' so nothing is magic."
    4. "Where is the full, working code for a lesson?" options ["Nowhere, you type it from the page", "In the code/ folder of the book's repository, linked under each listing", "In a zip you email for", "On the Rust Playground"], answer 1, explain "Every listing is included from a real crate in code/, which CI compiles and tests."
    - Flashcards: front "Which crates does this book use for web and database?" back "Axum (web framework) and SeaORM (ORM), on top of Tokio (async runtime) and PostgreSQL (database)." · front "What are the three exercise tiers?" back "🟢 Guided (fill in), 🟡 Tweak (change working code), 🔴 From scratch (write from a one-line spec)."

- [ ] **Step 2: Verify and commit** `how-to-use-this-book` (shared rules 4–6).

- [ ] **Step 3: Write `your-toolbox`** per this brief:
  - *What & why:* a carpenter's toolbox analogy; list of tools and the job of each: Rust/Cargo (build & run), Docker (runs Postgres in a box so you don't install it into your system), Postgres (the database), `psql` (talk to Postgres), curl (send HTTP requests), Bruno (optional GUI for requests), `sea-orm-cli` (installed later in Part A3 — say so).
  - *The idea, slowly:* listings, each with Line by line + Run it:
    1. ` ```bash ` `docker --version` / `docker compose version`.
    2. ` ```yaml ` include the book's `docker-compose.yml` via `{{#include ../../docker-compose.yml}}`. Line by line for every key: `services`, `db`, `image: postgres:18`, each env var (explain these are the login details, fine for a laptop, never for production), `ports: "5433:5432"` (host:container, and **why 5433** — you may already have Postgres on 5432), `volumes` (data survives restarts; why the parent path for Postgres 18), `healthcheck` (so `--wait` knows when it's ready).
    3. ` ```bash ` `docker compose up -d --wait`, `docker compose ps` — real output.
    4. ` ```bash ` `cp .env.example .env` and include `.env.example` via `{{#include ../../.env.example}}`; Line by line for the URL parts `postgres://USER:PASSWORD@HOST:PORT/DATABASE` as a labelled diagram in a text block.
    5. ` ```bash ` `psql postgres://postgres:postgres@localhost:5433/rbh -c 'select version();'` — real output from Task 4 Step 9. Offer the no-install alternative `docker compose exec db psql -U postgres -d rbh -c 'select version();'`.
    6. ` ```bash ` `curl --version`.
    7. Stopping: `docker compose down` vs `docker compose down -v` (the `-v` deletes your data — warn in bold).
  - *You might be wondering…:* "Why Docker instead of installing Postgres?" · "What is a port?" · "Is the password `postgres` safe?" (for a laptop yes, for a server never — Part B covers secrets) · "Where does my data live?" (the `pgdata` volume).
  - *Coming from another language?:* same tools in every stack; `.env` works like Node's `dotenv` / Python's `python-dotenv`.
  - *Common mistakes:* `Cannot connect to the Docker daemon` → start Docker Desktop/OrbStack; `port is already allocated` → something else on 5433, change the left number and the URL; `psql: command not found` → use the `docker compose exec` form; `password authentication failed` → an old volume with a different password: `docker compose down -v` then up again.
  - *More examples:* view logs `docker compose logs db`; list tables `\dt`; restart; check which program uses a port (`lsof -i :5433`).
  - *Your turn:* 🟢 start the db and run `select 1 + 1;`; 🟡 create a second database `createdb`-style via `psql -c 'create database playground;'` and list with `\l`; 🔴 stop everything, start again, and prove your `playground` database survived (solution explains volumes).
  - Quiz:
    1. "In `ports: \"5433:5432\"`, what is 5433?" options ["The port inside the container", "The port on your computer", "The Postgres version", "The number of connections"], answer 1, explain "The left side is your computer (host), the right side is inside the container. We use 5433 so it can't clash with a Postgres already using 5432."
    2. "Why do we run Postgres in Docker?" options ["Postgres only works in Docker", "It keeps Postgres in an isolated box that is easy to start, stop and delete", "Docker makes queries faster", "Rust requires Docker"], answer 1, explain "Docker gives everyone the same Postgres 18 without installing it into your system, and you can throw it away and start fresh any time."
    3. "What does `docker compose down -v` do that `down` does not?" options ["Shows a verbose log", "Deletes the data volume — your data is gone", "Updates Postgres", "Nothing different"], answer 1, explain "`-v` removes volumes. Use it only when you want a completely fresh database."
    4. "What goes in `.env`?" options ["Your Rust code", "Settings like DATABASE_URL that change between computers", "The docker image", "Test results"], answer 1, explain "`.env` holds environment variables such as the database address. It is never committed to git."
    - Flashcards: front "Shape of a Postgres URL?" back "`postgres://USER:PASSWORD@HOST:PORT/DATABASE`" · front "Start / stop the book's database?" back "`docker compose up -d --wait` / `docker compose down`"

- [ ] **Step 4: Verify and commit** `your-toolbox` (shared rules 4–6).

### Task 8: Lessons "How a web backend works" and "Tour of the stack"

**Files:**
- Modify: `src/part-0-start/how-a-web-backend-works.md`, `src/part-0-start/tour-of-the-stack.md`
- Create/modify: `questions/how-a-web-backend-works.json`, `questions/tour-of-the-stack.json`

**Interfaces:**
- Consumes: anchors `handler`, `app`, `main` in `code/topics/tour-of-the-stack/src/main.rs` (Task 4); glossary ids (Task 6).

- [ ] **Step 1: Write `how-a-web-backend-works`** per this brief:
  - *What & why:* restaurant analogy — client = customer, request = order slip, server = kitchen, handler = the cook for that dish, database = the pantry, response = the plate, status code = the waiter's note ("served", "we don't have that", "kitchen on fire").
  - *The idea, slowly:* listings with Line by line:
    1. ` ```text ` a raw HTTP request: `GET /products/42?currency=usd HTTP/1.1` / `Host: shop.example.com` / `Accept: application/json` / blank line. Line by line: method, path, path segment `42`, query string, version, headers, why the blank line.
    2. ` ```text ` a raw response: `HTTP/1.1 200 OK` / `Content-Type: application/json` / blank / `{"id":42,"name":"Mug","price":1299}`. Line by line incl. why price is cents (integer money, no float rounding).
    3. ` ```bash ` `curl -i https://httpbin.org/get` — Run it with real output (trim), mapping each part back to listing 2. ❌ "no internet → skip; you'll run your own server in the next lesson".
    4. ` ```bash ` `curl -i -X POST https://httpbin.org/post -H 'Content-Type: application/json' -d '{"name":"Mug"}'` — Line by line for `-i`, `-X`, `-H`, `-d`.
    5. JSON: a ` ```json ` product object with a nested array; explain objects, arrays, strings, numbers, booleans, null.
    6. REST: a ` ```text ` table of the ShopRS product endpoints (`GET /products`, `GET /products/{id}`, `POST /products`, `PUT /products/{id}`, `DELETE /products/{id}`) — what each means.
    7. Status codes: table of 200, 201, 204, 400, 401, 403, 404, 409, 422, 500 with a one-line "when" each.
  - *You might be wondering…:* "GET vs POST?" · "Why JSON and not HTML?" · "What's the difference between 401 and 403?" · "Is `?currency=usd` part of the path?"
  - *Coming from another language?:* `fetch()` in JS / `requests` in Python send exactly these requests.
  - *Common mistakes:* sending JSON without `Content-Type: application/json`; using GET to change data; returning 200 for errors; putting secrets in the query string.
  - *More examples:* 4 raw request/response pairs: create (201 + Location), not found (404), bad input (422 with an error JSON body), delete (204, no body).
  - *Your turn:* 🟢 run `curl -i https://httpbin.org/status/404` and name the status; 🟡 send a POST with your own JSON and find it in the echo; 🔴 write (as plain text) the request and response for "add a product to my cart" (solution: `POST /cart/items` + `201` + JSON).
  - Quiz:
    1. "Which part of `GET /products/42?currency=usd` is the query string?" options ["GET", "/products/42", "currency=usd", "HTTP/1.1"], answer 2, explain "Everything after `?` is the query string: optional extra options for the request."
    2. "Which status code means 'created successfully'?" options ["200", "201", "204", "404"], answer 1, explain "201 Created is returned after a POST creates something new."
    3. "A logged-in user tries to open an admin-only page. Which status?" options ["401", "403", "404", "500"], answer 1, explain "403 Forbidden: we know who you are, you're not allowed. 401 means we don't know who you are."
    4. "Why store a price as 1299 instead of 12.99?" options ["It's shorter", "Whole numbers of cents avoid floating-point rounding errors with money", "JSON can't hold decimals", "Postgres requires it"], answer 1, explain "0.1 + 0.2 is not exactly 0.3 in floating point. Counting cents as integers keeps money exact."
    - Flashcards: front "Five parts of an HTTP request?" back "Method, path (+ query string), version, headers, body." · front "401 vs 403?" back "401: not logged in (who are you?). 403: logged in but not allowed."

- [ ] **Step 2: Verify and commit** `how-a-web-backend-works` (shared rules 4–6).

- [ ] **Step 3: Write `tour-of-the-stack`** per this brief:
  - *What & why:* the four tools as a restaurant team — Postgres = pantry (stores data safely), SeaORM = pantry clerk (fetches/stores ingredients for you in Rust terms), Axum = head waiter + order system (routes each request to the right handler), Tokio = the shift manager who lets one cook juggle many orders while waiting (async runtime).
  - *The idea, slowly:*
    1. A ` ```text ` diagram of one request: `curl → TCP port 3000 → Tokio → Axum Router → handler → SeaORM → Postgres` and back, with one line per arrow.
    2. The code's `Cargo.toml` via `{{#include ../../code/topics/tour-of-the-stack/Cargo.toml}}`, Line by line: why each dependency; `.workspace = true` meaning "version is set once in `code/Cargo.toml`"; the `macros`/`rt-multi-thread`/`net` Tokio features.
    3. ` ```rust ` `{{#include ../../code/topics/tour-of-the-stack/src/main.rs:handler}}` — Line by line: `async fn`, `&'static str` (text baked into the program, link RFH lifetimes), why returning a string is enough (Axum turns it into a 200 text response — the `IntoResponse` idea, previewed and linked to A2).
    4. ` ```rust ` `…:app}}` — Line by line: `Router::new()`, `.route("/", get(hello))`, why `hello` is passed without `()` (we hand over the function, Axum calls it later), why it's a separate `fn app()` (so tests can use it).
    5. ` ```rust ` `…:main}}` — Line by line: `#[tokio::main]` (what it expands to: builds a runtime and blocks on your async main), `TcpListener::bind`, `.await`, `.expect(...)` and why our message tells you exactly what to do, `println!`, `axum::serve`.
    6. `### Run it`: `cd code && cargo run -p tour-of-the-stack`, then in a second terminal `curl -i http://127.0.0.1:3000/` — real output (from Task 4 Step 6 plus `-i` headers). ✅/❌: `Address already in use` → the expect message; `Connection refused` → server not running / wrong port.
    7. Where SeaORM + Postgres come in: one paragraph and a ` ```text ` preview of the Part A3 flow — no code yet, say so.
    - Include `📁 Full code: code/topics/tour-of-the-stack` under the listings.
  - *You might be wondering…:* "Why does main need `async`?" · "Why `127.0.0.1` and not `0.0.0.0`?" (only your computer can reach it; Docker/deploy later) · "Why does the program never end?" (a server waits forever; Ctrl+C to stop) · "Why SeaORM and not raw SQL?" (preview; both are taught).
  - *Coming from another language?:* Express `app.get('/', (req,res)=>res.send('Hello'))`; Flask `@app.route('/')`; Go `http.HandleFunc("/", ...)` — side by side with the Axum lines.
  - *Common mistakes:* forgetting `.await` (show the real compiler error text: run it and paste); calling `get(hello())` with parentheses (paste real error); port busy; missing `macros` feature on tokio (paste real error for `#[tokio::main]`). Produce each error by temporarily editing a scratch copy, never the committed crate.
  - *More examples:* 4 × ` ```rust ` snippets (not included, marked "try it in the crate"): a second route `/about`; returning `String` with `format!`; returning a status + text tuple `(StatusCode::CREATED, "made")`; binding to port 8080.
  - *Your turn:* 🟢 change the text to your name and re-run; 🟡 add `GET /health` returning `"ok"` and a test for it (solution includes the test); 🔴 add `GET /time` returning the seconds since the Unix epoch using `std::time::SystemTime` (solution shown).
  - Quiz:
    1. "Which tool decides which function answers `GET /`?" options ["Tokio", "Axum's Router", "SeaORM", "Postgres"], answer 1, explain "The Router matches the method and path to a handler. Tokio runs the async machinery; SeaORM and Postgres handle data."
    2. "What does `#[tokio::main]` do?" options ["Makes the program run faster", "Starts the Tokio async runtime and runs your async main on it", "Connects to the database", "Opens port 3000"], answer 1, explain "`main` can't be async on its own. The attribute builds a runtime and runs your async main inside it."
    3. "Why is it `get(hello)` and not `get(hello())`?" options ["Style preference", "We give Axum the function to call later, not the result of calling it now", "Parentheses are not allowed in Rust", "hello() would call the database"], answer 1, explain "Axum calls `hello` each time a matching request arrives. `hello()` would call it once, right now."
    4. "What does SeaORM do in this stack?" options ["Serves HTTP", "Lets Rust code read and write Postgres rows as structs", "Schedules async tasks", "Stores the data on disk"], answer 1, explain "SeaORM is the ORM: it turns rows into Rust structs and back. Postgres itself stores the data."
    - Flashcards: front "The four layers, request order?" back "Tokio (runtime) → Axum (routing/handlers) → SeaORM (ORM) → Postgres (database)." · front "Minimal Axum server lines?" back "`Router::new().route(\"/\", get(handler))`, `TcpListener::bind(addr).await`, `axum::serve(listener, app).await`."

- [ ] **Step 4: Verify and commit** `tour-of-the-stack` (shared rules 4–6). Additionally run `cd code && cargo test --workspace` to confirm the committed crate is unchanged and green.

---

### Task 9: Contributor docs, full check and first deploy

**Files:**
- Create: `README.md`, `CLAUDE.md`

- [ ] **Step 1: Write `README.md`** — sections: what the book is (2 lines + link to the spec); develop locally (`cargo install mdbook --locked` or brew, `npm test`, `node tools/generate.mjs`, `node tools/validate.mjs`, `mdbook serve --open`); run the book's code (`docker compose up -d --wait`, `cp .env.example .env`, `cd code && cargo test --workspace`); how to publish a draft page (flip `status` to `"published"` in `tools/topics.data.js` → `node tools/generate.mjs` → fill the stub → validate); include/anchor rules; deploy (Settings → Pages → Source: GitHub Actions).

- [ ] **Step 2: Write `CLAUDE.md`** — same facts as README in agent form, plus: the Global Constraints of this plan verbatim; "never hand-edit `src/SUMMARY.md` or `theme/questions.data.js`"; "every Rust listing must be an `{{#include}}` from `code/`"; "paste only real command output"; the spec path.

- [ ] **Step 3: Full local check**

```bash
npm test && node tools/generate.mjs && node tools/validate.mjs && mdbook build \
  && (cd code && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace)
```

Expected: tests pass; validator prints `4 published pages checked (62 drafts skipped) — 0 error(s)` with no banned-word warnings; `mdbook build` succeeds; cargo all green.

- [ ] **Step 4: Visual check** — `mdbook serve --open`; confirm in the browser: sidebar shows all six parts with drafts greyed out; each Part 0 page renders its included code (no empty code boxes); quiz works and persists after reload; Review page shows 8 flashcards; dark theme readable. Check DevTools → Application → Local Storage shows keys starting `rbh:v1:`.

- [ ] **Step 5: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: README and CLAUDE.md for contributors

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: Publish (needs the user's go-ahead — outward-facing).** Ask the user which GitHub repo to use (suggested: `Open-Source-BD/rust-book-backend`). With approval: `gh repo create … --public --source . --push`, then enable Pages (Source: GitHub Actions) and confirm both workflows go green with `gh run list --limit 4`.
