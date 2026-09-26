# Rust Backend for Humans

A free book/website that takes a reader who knows basic Rust from zero backend knowledge to
building and deploying a production-grade e-commerce REST API with Axum, SeaORM and PostgreSQL.
Every code listing is a real, tested Cargo crate pulled into the page — nothing is hand-typed or
untested. See the [design spec](docs/superpowers/specs/2026-09-24-rust-backend-book-design.md) for
the full plan (table of contents, page format, rollout phases).

## Develop locally

Install the tools:

```bash
cargo install mdbook --locked --version 0.5.4   # or: brew install mdbook
```

Then, from the repo root:

```bash
npm test                  # runs the generator/validator unit tests
node tools/generate.mjs   # writes any missing page stubs, regenerates SUMMARY.md and questions.data.js
node tools/validate.mjs   # checks every published page against the format rules
mdbook serve --open       # builds the book and opens it in your browser, rebuilding on save
```

`npm test` runs `node --test "tools/test/*.test.mjs"` (a quoted glob). Node 24 rejects a bare
directory argument (`node --test tools/test`), so don't drop the quotes or the glob.

## Run the book's code

The Axum/SeaORM listings in the book are real crates under `code/`, compiled and tested in CI. To
run them yourself:

```bash
cp .env.example .env
docker compose up -d --wait   # starts Postgres 18 on host port 5433
cd code && cargo test --workspace --all-targets   # --all-targets also runs examples/*.rs tests
```

`docker compose up -d --wait` only returns once Postgres is actually ready to accept connections:
the healthcheck in `docker-compose.yml` runs `pg_isready -h 127.0.0.1 …` (a TCP check), not a
socket check, because Postgres's temporary first-start server listens on the socket before it's
ready to accept real TCP connections.

Postgres is mapped to host port **5433** (not 5432), so it doesn't clash with a Postgres you may
already have installed locally. If something else on your machine is already using 5433, `docker
compose up` will fail with "port is already allocated" — fix it by changing the left-hand port
number in `docker-compose.yml` (e.g. `"5434:5432"`) and the port in `DATABASE_URL` in `.env` to
match.

## Publish a draft page

Every page lives in `tools/topics.data.js` with a `status` of `"draft"` or `"published"`. Draft
pages show up in the sidebar greyed out (no link) and are skipped by the validator. To publish one:

1. Set `status: "published"` for that page's entry in `tools/topics.data.js`.
2. Run `node tools/generate.mjs` — this writes a stub file with the required headings (if the page
   doesn't already exist), regenerates `src/SUMMARY.md` and `theme/questions.data.js`, and
   rewrites the Next block between `<!-- next:start -->` and `<!-- next:end -->` in every
   published page — so a page whose Next said "X (coming soon)" now links to X.
3. Fill in the stub. Leave the two `next:` markers in `## Go deeper` alone: the generator owns
   what's between them (change `next` in `topics.data.js` instead); the links above them are yours.
4. Run `node tools/validate.mjs` and fix anything it reports. It errors on a stale
   "(coming soon)", a link to a missing or draft page, and hand-typed Rust.
5. Commit the new page together with any pages whose Next block `generate.mjs` updated.

Never hand-edit `src/SUMMARY.md` or `theme/questions.data.js` — both are generated and any manual
edit will be overwritten (and drift from `topics.data.js`) the next time `generate.mjs` runs.

## Include/anchor rules

Every Rust listing anywhere in a lesson — including `use` lines, `## More examples` and
`## Your turn` solutions — comes from a real, compiled file under `code/` via mdBook's
`{{#include path:anchor}}` (or `{{#rustdoc_include}}`), using `// ANCHOR: name` /
`// ANCHOR_END: name` markers in the source. Variations and exercise solutions live as small full
programs in `code/topics/<slug>/examples/<name>.rs`. Nothing is hand-typed into the Markdown. This
keeps the book and the code from drifting apart: `validate.mjs` errors if an include's path or
anchor doesn't resolve, and on any Rust fence with no include, except the two below.

Fence choice matters:

- Axum/SeaORM listings use ` ```rust,noplayground ` (not plain ` ```rust `). `book.toml` sets
  `[output.html.playground] editable = true`, so a plain ` ```rust ` fence gets an in-browser Run
  button — which would try to compile a bare snippet with no `Cargo.toml`, no dependencies and no
  `main`, and fail. `noplayground` shows the code without the Run button.
- Pure-std runnable snippets (no external crates) may use ` ```rust,editable ` instead, since they
  really can run standalone in the playground.
- Deliberately broken code shown under `## Common mistakes` uses ` ```rust,noplayground,ignore `,
  always next to the real compiler error it produces.

## Deploy

The site deploys via GitHub Actions (`.github/workflows/deploy.yml`) on every push to `main`. In
the repo's GitHub settings: **Settings → Pages → Source: GitHub Actions**. The workflow runs
`generate.mjs`, `validate.mjs` and `mdbook build` (failing on any mdBook `ERROR` line, because
mdBook itself exits 0 on a broken include), then publishes `book/` to GitHub Pages.
