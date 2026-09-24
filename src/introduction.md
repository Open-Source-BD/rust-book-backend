# Rust Backend for Humans

You will start this book not knowing what a [server](glossary.md#server) even is, and finish it
having built and deployed a working e-commerce [backend](glossary.md#backend) that real people
could use. Every line of
code in every lesson is explained — what it does, why it is there, and what breaks if you remove
it. Nothing is assumed: if a word might be new to you, it is defined the moment it appears, and
linked in the [Glossary](glossary.md) so you can look it up again later.

## What you'll build

Along the way you'll build several small **APIs** of your own. An API is a set of addresses your
program answers, so that other programs — a website, a phone app, another service — can ask it for
data or ask it to do something. By the end of Part A, you will have built:

- **Library database** — design tables and write real SQL queries for a small library system,
  straight in Postgres.
- **Todo API** — an in-memory [REST](glossary.md#rest) API for a to-do list, with a
  [route](glossary.md#route) for every basic operation: add, view, change, remove.
- **Blog data layer** — the data side of a blog (users, posts and comments), built with SeaORM
  instead of raw SQL.
- **Notes API** — a full API backed by a real Postgres database, combining everything Axum and
  SeaORM taught you.
- **URL shortener** — a REST API you design and build yourself from a written description of what
  it must do, with hints to help if you get stuck.

Then comes the capstone — the big project that pulls everything together: **ShopRS**, a
production-style e-commerce backend with products, a shopping cart,
checkout, payments and an admin area. It is the same kind of backend a real online shop would run,
and you build and deploy every part of it yourself.

## Is this book for me?

This book does not re-teach the Rust language — it assumes you already know basic Rust. You're
ready if you've finished *Rust for Humans*, or if you're already comfortable with these four ideas:

- [Rust for Humans: Ownership](https://open-source-bd.github.io/rustbook-for-human/ownership/ownership.html)
- [Rust for Humans: Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
- [Rust for Humans: Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
- [Rust for Humans: Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)

Coming from JS/Python/Java/Go? Good — every lesson has a box translating to what you know.

## How every lesson works

Every topic lesson (Part A onward) follows the same eleven sections in the same order, so you
always know where to look for what you need:

| # | Heading | Contents |
|---|---|---|
| 1 | `## By the end of this lesson` | 2–3 concrete, checkable outcomes |
| 2 | `## What & why` | Real-world problem solved + plain-language analogy |
| 3 | `## The idea, slowly` | Small steps. After each listing: `### Line by line` and `### Run it` |
| 4 | `## You might be wondering…` | Pre-emptive FAQ at the point of confusion |
| 5 | `## Coming from another language?` | Express / Flask / Spring / Go equivalents |
| 6 | `## Common mistakes` | Real compiler/runtime errors, meaning, fix |
| 7 | `## More examples` | 4 scenario subsections (`### title`, one-line hook, code) |
| 8 | `## Your turn` | Three tiered exercises: 🟢 Guided, 🟡 Tweak, 🔴 From scratch, each with a hidden solution |
| 9 | `## Quick check` | A short quiz on this lesson only |
| 10 | `## Remember this` | 3–5 takeaways |
| 11 | `## Go deeper` | Official docs + Rust for Humans links |

The section you'll see most often is **Line by line**: after almost every piece of code, each
non-trivial line gets its own What / Why / How / "remove it and…" breakdown. Here is what one looks
like:

`let app = Router::new().route("/hello", get(hello));`
- **What:** creates the app's route table and adds one rule.
- **Why:** Axum has to know which function answers which address.
- **How:** `Router::new()` starts an empty table; `.route(path, method_router)` adds a rule;
  `get(hello)` means "only for GET, call `hello`".
- **Remove it and…** every request gets `404 Not Found`.

You never have to guess what a line does or why it is there — if it matters, it gets one of these.

## The path

- **Part 0 · Before you start** — set up your tools and learn the ideas every later lesson leans on.
- **Part A1 · PostgreSQL & SQL** — design tables and write queries directly against Postgres.
- **Part A2 · Axum** — build web APIs: routes, handlers, JSON, error handling and testing.
- **Part A3 · SeaORM** — talk to Postgres from Rust code instead of writing raw SQL by hand.
- **Part A4 · Putting it together** — configuration, logging and project layout, then two
  build-it-yourself APIs and a readiness check.
- **Part B · Capstone: ShopRS** — build and ship a full e-commerce backend end to end.

Each part ends with a project you build yourself, and the guidance shrinks every time you reach
one: the first projects are **guided** step by step with checkpoints, the next is **half-guided**
(the steps are named, but you work the code out from hints), and the last is **independent** — a
written description of what to build, plus hints — no walkthrough until you've tried it yourself.
A **readiness check** then lists everything the capstone needs, with each item linking back to the
lesson that teaches it, so any gaps get filled before you start the **capstone** — the ShopRS
project itself.

## How to not forget

Every lesson ends with a `## Quick check` — a handful of questions on that lesson alone. Cards you
find tricky come back more often on the [Review & flashcards](review.md) page, which collects the
key question from every lesson you've read into one shuffled deck. Everything is saved in your own
browser, so you can dip in and out without losing progress.

## Start here

Ready? Start with [How to use this book](part-0-start/how-to-use-this-book.md).
