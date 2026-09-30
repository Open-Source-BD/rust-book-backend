# CLAUDE.md

This file gives an agent (or a human contributor) what it needs to work on this repo without
re-deriving it from scratch. This is "Rust Backend for Humans": an mdBook site + a Cargo workspace
of real, tested code that the pages `{{#include}}` from. Read the full design spec before making
structural changes: `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md`.

## Local dev loop

```bash
npm test                  # node --test "tools/test/*.test.mjs" — keep the quotes: Node 24 rejects
                           # a bare directory argument (`node --test tools/test` fails)
node tools/generate.mjs   # writes missing page stubs; regenerates src/SUMMARY.md + theme/questions.data.js
node tools/validate.mjs   # checks every published page; exits non-zero on error
mdbook build               # or `mdbook serve --open` while iterating
```

## Running the book's code

```bash
cp .env.example .env
docker compose up -d --wait   # Postgres 18 on host port 5433
cd code && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --all-targets
```

`--all-targets` matters: it also runs the tests inside `examples/*.rs` (plain `cargo test
--workspace` skips them). CI (`code.yml`) runs the same command.

- Postgres listens on host port **5433**, not 5432, so it doesn't clash with a locally installed
  Postgres. If `docker compose up` fails with "port is already allocated", something else on the
  host already owns 5433 — change the left-hand port number in `docker-compose.yml` (e.g.
  `"5434:5432"`) and the port in `DATABASE_URL` (`.env` / `.env.example`) to match. Do not just
  retry.
- `docker compose up -d --wait` blocks until Postgres is truly ready because the healthcheck in
  `docker-compose.yml` runs `pg_isready -h 127.0.0.1 -U postgres -d rbh` — a TCP check. A socket
  check would report healthy too early, while Postgres's temporary first-start server is still
  only listening on the Unix socket.

## SQL listings

Part A1 (PostgreSQL & SQL) lessons show real `psql` output, checked by CI instead of hand-typed.

- **File layout and numbering:** `code/sql/<slug>/NN-name.sql`, a two-digit prefix, run in name
  order inside one database. Ranges: `01`–`49` for "The idea, slowly", `50`–`69` for "More
  examples", `70`–`79` for "Common mistakes", `90`–`99` for "Your turn" solutions.
- **One database per lesson:** the database name is the slug with `-` → `_` (e.g. `crud-in-sql` →
  `crud_in_sql`, via `dbNameFor` in `tools/sql.mjs`). Readers create it once with `docker compose
  exec db createdb -U postgres <name>` and run a file with `docker compose exec -T db psql -U
  postgres -d <name> < code/sql/<slug>/NN-name.sql`.
- **Rerunnable preamble:** a file that creates tables starts with `SET client_min_messages =
  warning;` and `DROP TABLE IF EXISTS …;`, so running it twice gives identical output, not
  "relation already exists".
- **No nondeterministic SQL:** no `now()`, `random()`, `gen_random_uuid()`, `\timing` or
  `version()` — use fixed date literals instead. `EXPLAIN` always uses `(COSTS OFF)`, or
  `(ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)`.
- **`--check-twice` re-runs the lesson in the SAME database, not a fresh one.** After the normal
  (fresh-database) pass, it runs every `NN-*.sql` file again from `01` in that same database, with
  no `DROP`/`CREATE` in between, and fails if the output differs. This is exactly what a reader
  does when they rerun a lesson, so it proves two things at once: the SQL is deterministic, *and*
  `01` actually resets its own tables (the rerunnable-preamble rule above) — a lesson missing that
  preamble fails `--check-twice` even though its single-run output is correct. CI always passes
  `--check-twice`.
- **A failed setup is fatal, never silent.** Before running a lesson's files, the runner drops and
  recreates its database with `\set ON_ERROR_STOP 1` — so if that `DROP`/`CREATE` itself fails
  (e.g. the database can't be dropped), `sql-check.mjs` exits 2 immediately with a message naming
  the setup step and showing psql's real error, instead of quietly running the lesson's files
  against a stale database and reporting a false "all SQL outputs match". Lesson files themselves
  never get `ON_ERROR_STOP`: a Common-mistakes file's job is to run to completion and capture its
  own real error in its `.out`.
- **`.out` files are never hand-edited.** They come only from `node tools/sql-check.mjs --update
  <slug>`, which drops and recreates that lesson's database, pipes each `NN-*.sql` file into
  `psql` for real, and writes the normalized output next to it as `NN-name.out`. Lessons show it
  via `{{#include ../../code/sql/<slug>/NN-name.out}}` in a ` ```text ` fence.
- **Running the checker:** `npm run sql` (= `node tools/sql-check.mjs`) checks every lesson;
  `node tools/sql-check.mjs <slug>` checks one; `--update <slug>` regenerates that lesson's `.out`
  files; `--check-twice` also runs the same-database rerun check above. `RBH_PSQL` is a shell
  command prefix that runs `psql` — the runner appends the database name and pipes SQL on stdin.
  It defaults to `docker compose exec -T db sh -c 'psql -X -U postgres -d "$0" 2>&1'` (the book's
  own `docker-compose.yml` on port 5433). Point it at a different Postgres — a scratch instance on
  another port, or CI's
  service container — by exporting `RBH_PSQL` yourself, e.g. `export RBH_PSQL="docker compose -p
  rbh-sql -f /path/to/docker-compose.yml exec -T db sh -c 'psql -X -U postgres -d \"\$0\" 2>&1'"`.
- **Validator's SQL-fence rule:** every ` ```sql ` fence in a lesson must be an
  `{{#include ../../code/sql/<slug>/NN-name.sql}}`, never hand-typed SQL — the same "comes from
  real, checked code" rule as Rust listings, so `validate.mjs` errors on a hand-typed one and CI
  actually runs what the page shows. The exception is ` ```sql,ignore `, for deliberately broken
  SQL shown next to its real error (mirrors `rust,noplayground,ignore`). The fence's include must
  also resolve under `code/sql/` (the only folder sql-check runs); `validate.mjs` errors otherwise.
- **SQL that Rust generates (Part A2+):** SQL that sqlx/SeaORM builds for you, such as a logged
  query, is shown in a ` ```text ` fence holding real captured output — never a ` ```sql ` fence,
  because ` ```sql ` fences are for executed `code/sql/` listing includes only.
- **Orphan `.out` files:** `--update` never deletes files, so a `.out` whose `.sql` was renamed or
  removed is reported as `FAIL <slug>/NN-name.out: no matching .sql` and `--update` still exits 1.
  That's deliberate: the stale output would otherwise linger (and could still be included by a
  page). `git rm` the orphan, then rerun.

## HTTP transcripts

Part A2+ lessons show real `curl` output from a running Axum server, checked by CI instead of
hand-typed. The runner is `tools/http-check.mjs`.

- **Script format:** `code/topics/<slug>/http/NN-name.sh`. The first line is
  `# serve: -p <package> [--example <name>]` (the `cargo run` arguments; `-p` is required). Every
  other non-comment, non-blank line is one command the reader types, usually `curl -i …`. JSON
  requests: `curl -i -X POST http://127.0.0.1:3000/<path> -H 'content-type: application/json' -d '<json>'`.
- **NN ranges:** same as SQL: `01`–`49` The idea, slowly; `50`–`69` More examples; `70`–`79`
  Common mistakes; `90`–`99` Your turn.
- **Port 3000:** every lesson server binds `127.0.0.1:3000`. The runner refuses to start a script
  (a FAIL line, exit 1) if something already listens on 3000, rather than test the wrong server,
  and it stops its own server after every script, even when a command fails.
- **Volatile output:** the runner drops every `date:` response header (the only line that changes
  each second) and normalizes CRLF. curl runs with a private `.curlrc` (`no-progress-meter`), so no
  progress meter reaches a `.out`.
- **`--update` and `--check-twice`:** `node tools/http-check.mjs --update <package>` writes each
  `NN-name.out`. `--check-twice` runs every script twice, each time against a fresh server, and
  fails if the output differs. `npm run http` = `node tools/http-check.mjs`; CI passes
  `--check-twice`. Orphan `.out` files are reported as `no matching .sh`.
- **On the page:** show the script in a ` ```bash ` fence with
  `{{#include ../../code/topics/<slug>/http/NN-name.sh}}` and the result in a ` ```text ` fence with
  `{{#include ../../code/topics/<slug>/http/NN-name.out}}`. **`.out` files are never hand-edited**;
  they come only from `--update`.
- **Rust listings** follow the Hard rules above: ` ```rust,noplayground ` fence holding an include
  from `code/topics/<slug>/src/` (by anchor), then `📁 Full code: code/topics/<slug>` and
  `cd code && cargo run -p <slug>`. Variations and exercise solutions are full programs in
  `examples/<name>.rs` with a `#[cfg(test)]` test (run them with `cargo run -p <slug> --example <name>`).

## Publishing a draft page

1. Set `status: "published"` for that page's entry in `tools/topics.data.js`.
2. Run `node tools/generate.mjs` (writes the stub if missing; regenerates SUMMARY.md and
   questions.data.js; rewrites the Next block between `<!-- next:start -->` and
   `<!-- next:end -->` in every published page, so pages whose Next said "X (coming soon)" now
   link to X).
3. Fill in the stub. Keep the two `next:` markers in `## Go deeper`: the generator owns what's
   between them (edit `next` in `topics.data.js` instead); the RFH/official links above them are
   yours.
4. Run `node tools/validate.mjs` and fix everything it reports (it errors on a stale
   "(coming soon)", a link to a missing or draft page, and hand-typed Rust).
5. Commit the page together with the other pages whose Next block `generate.mjs` updated.

## Hard rules

- **Never hand-edit `src/SUMMARY.md` or `theme/questions.data.js`.** Both are generated by
  `tools/generate.mjs` from `tools/topics.data.js`; a manual edit is silently overwritten (and
  drifts from the source of truth) the next time `generate.mjs` runs. Edit `topics.data.js`
  instead and regenerate.
- **Every Rust listing anywhere in a lesson must come from compiled code under `code/` via
  `{{#include path:anchor}}` or `{{#rustdoc_include path:anchor}}`** — including `use` lines, and
  including `## More examples` and `## Your turn` solutions. Never hand-type a Rust snippet into a
  lesson's Markdown: it will drift from the real, tested code. Mark the source with
  `// ANCHOR: name` / `// ANCHOR_END: name` and include it by anchor. Example: the
  tour-of-the-stack crate (`code/topics/tour-of-the-stack/src/main.rs`) defines anchors `imports`,
  `handler`, `app` and `main`; the lesson includes all four separately instead of the whole file.
  Variations (an extra route, an exercise solution) are full small programs in
  `code/topics/<slug>/examples/<name>.rs`, with anchors around the part the lesson shows; a
  solution's test goes in that file under `#[cfg(test)]` (run by `--all-targets`). Only two
  exceptions, and `validate.mjs` errors on anything else:
  - ` ```rust,editable ` — pure-std code that runs on the Playground as it is.
  - ` ```rust,noplayground,ignore ` — deliberately broken code under `## Common mistakes`, always
    next to the real compiler error it produces.
- **Paste only real command output.** Run the command yourself and paste what it actually printed
  — never a plausible-looking reconstruction. The only edits allowed to pasted output: replace a
  personal username/path with `you`, and replace a public IP address with an RFC 5737 example
  address (e.g. `203.0.113.7`).
- CI Actions are pinned: `actions/checkout@v7`, `actions/setup-node@v7`,
  `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`. Don't bump these without checking
  they still work with `mdbook 0.5.4` / Node 24.

## Global Constraints

(Verbatim from the plan's constraints doc, with one amendment noted inline: the fence rule below
was corrected from plain `` ```rust `` to `` ```rust,noplayground `` during the build — see the
note attached to that bullet.)

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
- **Amended:** Axum/SeaORM listings use ` ```rust,noplayground ` (not plain ` ```rust `) + `📁 Full code: code/<path>` + run command. `book.toml` sets `[output.html.playground] editable = true`, so a plain ` ```rust ` fence gets a Run button — which fails for a bare snippet with no `Cargo.toml`, no dependencies and no `main`. Only pure-std runnable snippets may use ` ```rust,editable `.
- Pinned versions: axum 0.8.9, tokio 1.53.1, tower 0.5.3, http-body-util 0.1.5, sea-orm 2.0.3 (later phases), Postgres image `postgres:18`, mdBook 0.5.4, Node 24.
- Quiz: ≥4 questions per lesson (validator warns below 4). localStorage prefix `rbh:v1:`.
- Commits end with `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`.

### Review Focus

1. **Renamed/missing include anchor** — an author renames `// ANCHOR: app` in Rust; the reader should never see an empty code box. `validate.mjs` must error (test in Task 3).
2. **Heading-like lines inside code fences** — a bash `# comment` or a Markdown sample containing `## Your turn` inside a fence must not satisfy or break the heading check (test in Task 3).
3. **Same-origin storage collision with RFH** — both books on `open-source-bd.github.io`; quiz progress must not mix. Prefix `rbh:v1:` (check in Task 1).
4. **Links to draft pages** — a published page's "Next" pointing at a draft must render as plain "(coming soon)" text, never a dead link (test in Task 2).
5. **Reader already runs Postgres on 5432** (Homebrew, like this machine) — `docker compose up` must not fail with "port is already allocated". Map host port **5433** (Task 4), and the toolbox lesson explains it.
