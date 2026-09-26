# Phase 2a — Tooling carry-overs, CI-checked SQL, Part A1 (PostgreSQL & SQL) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish Part A1 of "Rust Backend for Humans": 7 PostgreSQL lessons, the "Build it: a library database" project and the SQL cheat sheet. Every SQL listing and every pasted psql output is checked by CI against a real Postgres 18.

**Architecture:** SQL listings live in `code/sql/<slug>/NN-name.sql`. The new runner `tools/sql-check.mjs` executes them in order against a fresh database per lesson and compares psql's real output with the committed `NN-name.out` files. `--update` rewrites those files. Lessons `{{#include}}` both files, so readers see exactly what CI verified. The validator gains a rule that ` ```sql ` fences must be includes, just like Rust.

**Tech Stack:** Node 24 (`node:test`, `child_process.spawnSync`), psql 18 inside the `postgres:18` image (via `docker compose exec` locally, `docker run --network host` in CI), mdBook 0.5.4, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-24-rust-backend-book-design.md` (Phase 2 = A1 + A2; this plan is **2a**: carry-overs + A1. Plan 2b covers A2 Axum.)

**Read first:** `CLAUDE.md` (hard rules, publishing steps, local dev loop) and one finished lesson, `src/part-0-start/your-toolbox.md`, for house style.

## Global Constraints

- Everything in `CLAUDE.md` "Hard rules" and "Global Constraints" applies unchanged: 11 lesson headings in exact order, the three tiers each with a `<details>` solution, Line by line (What / Why / How / Remove it and…), Run it with ✅/❌, no banned words (simply, just, obviously, trivially), glossary links on first use, real output only, ≥4 quiz questions, and the commit trailer `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`.
- **SQL files:** `code/sql/<slug>/NN-name.sql`, two-digit prefix, run in name order in one database per lesson. Numbering: `01`–`49` for "The idea, slowly", `50`–`69` for "More examples", `70`–`79` for "Common mistakes", `90`–`99` for "Your turn" solutions.
- **SQL outputs:** `code/sql/<slug>/NN-name.out` is written only by `node tools/sql-check.mjs --update <slug>` and never hand-edited. Lessons show it in a ` ```text ` fence via `{{#include ../../code/sql/<slug>/NN-name.out}}`.
- **One database per lesson:** the database name is the slug with `-` → `_` (e.g. `crud_in_sql`). Readers create it once with `docker compose exec db createdb -U postgres <name>` and run a file with `docker compose exec -T db psql -U postgres -d <name> < code/sql/<slug>/NN-name.sql`. Every lesson's first "Run it" teaches this.
- **Rerunnable files:** a file that creates tables starts with `SET client_min_messages = warning;` and `DROP TABLE IF EXISTS …;`, so a reader can run it twice and get the same output.
- **No nondeterministic SQL:** no `now()`, `random()`, `gen_random_uuid()`, `\timing` or `version()`. Use fixed date literals. `EXPLAIN` always uses `(COSTS OFF)`, or `(ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)`.
- **Local Postgres on this machine:** host port 5433 is held by the unrelated container `inkwell-db-1`; never stop or modify it. Run the book's Postgres from a scratch copy of `docker-compose.yml` with host port `55433` and project `-p rbh-sql`, and point the runner at it with `RBH_PSQL` (exact value in Task 2).
- Every page change is published by the CLAUDE.md "Publishing a draft page" steps: set `status: "published"` and fill in `outcomes`, `summary`, `prereq`, `next`, `rfhLinks`, `links` and `codeDir: "code/sql/<slug>"` in `tools/topics.data.js`, run `node tools/generate.mjs`, write the page, then run `node tools/validate.mjs`.

## Review Focus

1. **Output drift.** If a `.sql` file is edited but its `.out` is not regenerated, the lesson must never show stale output. `sql-check.mjs` exits 1 with a line diff (test in Task 2).
2. **Nondeterministic output.** A query whose output changes run to run would make CI flaky. The runner merges stderr into stdout *inside* the container (verified stable 12/12 runs), and Task 2 adds a double-run determinism check (`--check-twice`) used in CI.
3. **Reader reruns a lesson file.** Running `01-*.sql` twice must give identical output, not "relation already exists". The preamble rule is enforced by the runner's second pass (test in Task 2) and reviewed per lesson.
4. **Lessons sharing one database.** Lesson B's `DROP TABLE books` must not collide with lesson A's tables. One database per lesson, created from the slug; `dbNameFor` is unit-tested (Task 2).
5. **A hand-typed SQL block** would never be run by CI. The validator errors on a ` ```sql ` fence that is not an include (unless tagged `ignore`) (test in Task 2).

---

## File structure

| Path | Responsibility |
|---|---|
| `tools/lib.mjs` | Validator: carry-over fixes (Task 1), and the SQL fence rule (Task 2) |
| `tools/sql.mjs` | Pure helpers for the runner: `dbNameFor`, `normalizeOutput`, `diffLines`, `planRun` |
| `tools/sql-check.mjs` | CLI runner: drop/create the DB, run files, compare or `--update` |
| `tools/test/lib.test.mjs`, `tools/test/sql.test.mjs` | Unit tests |
| `.github/workflows/code.yml` | New `sql` job |
| `code/sql/<slug>/*.sql`, `*.out` | Listings and their real outputs |
| `src/a1-postgres/*.md`, `questions/*.json` | Pages and quizzes |
| `src/glossary.md` | New A1 terms |
| `CLAUDE.md`, spec §4 | SQL listing rules |

---

### Task 1: Phase 1 carry-over fixes (tooling + one sentence)

**Files:**
- Modify: `tools/lib.mjs`, `tools/test/lib.test.mjs`, `src/part-0-start/tour-of-the-stack.md:179`

**Interfaces:**
- Consumes: the existing `stripFences`, `fencedBlocks`, `checkLinks`, `checkComingSoon`, `regenerateNext`, `checkPage` and test helpers `pub`, `map`, `T`, `good`, `ctx` in `tools/test/lib.test.mjs`.
- Produces: the same exported names and signatures (behaviour widened only). A new exported constant: `FENCE_PREFIX = /^[ \t>]*/`.

- [ ] **Step 1: Write the failing tests.** Append the following to `tools/test/lib.test.mjs`. Import `fencedBlocks` and `checkComingSoon` if they are not imported yet; `regenerateNext` and `checkLinks` are already imported at line 201.

```js
import { fencedBlocks, checkComingSoon } from "../lib.mjs";

test("carry: a fence indented 4+ spaces inside a list item hides its headings", () => {
  const md = good().replace("\n## Common mistakes\n", "\n- item:\n\n      ```md\n      ## Common mistakes\n      ```\n");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes('"## Common mistakes"')));
});

test("carry: a fence inside a blockquote is a fence", () => {
  const md = "> ```rust\n> fn main() {}\n> ```\n";
  const blocks = fencedBlocks(md);
  assert.equal(blocks.length, 1);
  assert.equal(blocks[0].info, "rust");
});

test("carry: hand-typed rust inside an indented list fence is caught", () => {
  const md = good().replace("## Common mistakes\n\ntext", "## Common mistakes\n\n1. step:\n\n    ```rust,noplayground\n    fn main() {}\n    ```");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("hand-typed Rust")));
});

test("carry: reference-style and <a href> links to missing pages are errors", () => {
  const md = "See [the toolbox][tb] and <a href=\"../part-0-start/nope.md\">this</a>.\n\n[tb]: ../part-0-start/your-tolbox.md\n";
  const errs = checkLinks({ md, mdFile: "/r/src/a1-postgres/x.md", exists: () => false, publishedPaths: new Set() });
  assert.equal(errs.length, 2);
});

test("carry: '* ' and '+ ' bullets count for stale coming-soon", () => {
  const titles = new Set(["Indexes"]);
  assert.equal(checkComingSoon("* Indexes (coming soon)\n", titles).length, 1);
  assert.equal(checkComingSoon("+ Indexes (coming soon)\n", titles).length, 1);
});

test("carry: a rust fence mixing an include with hand-typed lines is an error", () => {
  const md = good().replace("{{#include ../../code/x/src/main.rs:app}}", "{{#include ../../code/x/src/main.rs:app}}\nfn extra() {}");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("mixes")));
});

test("carry: a rust fence holding only {{#playground}} counts as included", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/x/examples/p.rs": "fn main(){}\n" };
  const md = good().replace("### Line by line", "```rust\n{{#playground ../../code/x/examples/p.rs}}\n```\n\n### Line by line");
  assert.ok(!checkPage(ctx(md, files)).errors.some((e) => e.includes("hand-typed")));
});

test("carry: regenerateNext keeps CRLF line endings in a CRLF page", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const before = "# A\r\n\r\n<!-- next:start -->\r\n<!-- next:end -->\r\n";
  const after = regenerateNext(before, a, map([a, b]));
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
  assert.ok(!/[^\r]\n/.test(after), "every \\n is preceded by \\r");
});

test("carry: marker text inside a code fence is not treated as the Next block", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const before = "```md\n<!-- next:start -->\nexample\n<!-- next:end -->\n```\n\n<!-- next:start -->\n<!-- next:end -->\n";
  const after = regenerateNext(before, a, map([a, b]));
  assert.ok(after.startsWith("```md\n<!-- next:start -->\nexample\n<!-- next:end -->\n```\n"), "fenced example untouched");
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
});
```

- [ ] **Step 2: Run the tests and see them fail.** Run `npm test`. Expected: the 9 new tests FAIL, and the 44 existing tests still pass.

- [ ] **Step 3: Implement the fixes in `tools/lib.mjs`.**
  1. **Fences.** Export `const FENCE_PREFIX = /^[ \t>]*/`. In `stripFences` and `fencedBlocks`, strip that prefix from each line before applying the opener and closer tests. The opener becomes `/^(`{3,}|~{3,})(.*)$/` on the stripped line, and the closer becomes `^${char}{${len},}\s*$`. Keep `FENCE_OPEN` for the "page has code" check, but make it `/^[ \t>]*(`{3,}|~{3,})/`. The body lines pushed by `fencedBlocks` are the prefix-stripped lines.
  2. **checkLinks.** Also scan these, with the same resolve, exists and published logic as inline links:
     - reference definitions: `/^\s*\[[^\]]+\]:\s*(?![a-z][a-z0-9+.-]*:)(\S+?\.md)(#\S*)?\s*$/gim`
     - HTML anchors: `/<a\s[^>]*href="(?![a-z][a-z0-9+.-]*:)([^"#]+\.md)(#[^"]*)?"/gi`
  3. **checkComingSoon.** Change the regex to `/^[-*+] (.+?) \(coming soon\)\s*$/gm`.
  4. **Rust rule (step 5 of checkPage).** Compute `const inc = b.body.filter((l) => /\{\{#(include|rustdoc_include|playground)\s/.test(l))` and `const other = b.body.filter((l) => l.trim() && !inc.includes(l))`. Then:
     - if `inc.length && other.length`: push the error `` line ${b.line}: fence mixes an {{#include}} with hand-typed lines — put all of it in code/ ``;
     - else if `!inc.length`: keep the existing hand-typed error;
     - otherwise the fence passes.
  5. **regenerateNext.**
     - Find the markers only on lines that are outside fences. Use `stripFences(md)` (which preserves line count) to locate the line index of `NEXT_START` and `NEXT_END`, then map back to character offsets in the original string.
     - Build the block with `\n`. If the original page contains `\r\n`, convert the new block's `\n` to `\r\n` before splicing.
     - A page without both unfenced markers is returned unchanged.

- [ ] **Step 4: Run the tests and see them pass.** Run `npm test`. Expected: all 53 pass. Also run `node tools/generate.mjs && git status --short`: expected empty (idempotent), and `node tools/validate.mjs` reports 0 errors and 0 warnings.

- [ ] **Step 5: Fix the carry-over sentence.** In `src/part-0-start/tour-of-the-stack.md:179`, replace
  `- **Why:** one \`cargo test --workspace\` from \`code/\` builds and tests every project in the book.`
  with
  `- **Why:** one \`cargo test --workspace --all-targets\` from \`code/\` builds and tests every project in the book — \`--all-targets\` also runs the tests inside each project's \`examples/\` folder, which plain \`cargo test --workspace\` skips.`
  Then re-run `node tools/validate.mjs` (0/0).

- [ ] **Step 6: Commit.**

```bash
git add tools/lib.mjs tools/test/lib.test.mjs src/part-0-start/tour-of-the-stack.md
git commit -m "fix(tools): phase-1 carry-overs — nested/quoted fences, ref links, mixed fences, CRLF-safe Next

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: CI-checked SQL — runner, validator rule, CI job, docs

**Files:**
- Create: `tools/sql.mjs`, `tools/sql-check.mjs`, `tools/test/sql.test.mjs`
- Modify: `tools/lib.mjs` (SQL fence rule), `tools/test/lib.test.mjs`, `package.json` (script), `.github/workflows/code.yml` (new job), `CLAUDE.md`, the spec §4 "Code ↔ book link"

**Interfaces:**
- Produces (from `tools/sql.mjs`):
  - `dbNameFor(slug: string) -> string`: lowercase, `-` → `_`. Throws on anything other than `[a-z0-9-]`.
  - `normalizeOutput(text: string) -> string`: CRLF → LF, trailing whitespace stripped per line, trailing blank lines removed, exactly one final `\n`. An empty string stays `""`.
  - `diffLines(expected: string, actual: string) -> Array<{ line: number, expected: string|undefined, actual: string|undefined }>`: 1-based, lists every differing line.
  - `planRun(entries: string[]) -> { sql: string[], orphans: string[] }`: `sql` holds the `NN-*.sql` names sorted; `orphans` holds `.out` files with no matching `.sql`. It throws if a `.sql` name lacks the two-digit `NN-` prefix.
- Produces (CLI): `node tools/sql-check.mjs [--update] [--check-twice] [slug …]`. The env var `RBH_PSQL` is a shell command prefix. The runner appends the database name and pipes SQL on stdin. The default is `docker compose exec -T db sh -c 'psql -X -U postgres -d "$0" 2>&1'`.
- Produces (npm): `"sql": "node tools/sql-check.mjs"` in `package.json`.

- [ ] **Step 1: Write the failing tests.** Create `tools/test/sql.test.mjs`:

```js
import { test } from "node:test";
import assert from "node:assert/strict";
import { dbNameFor, normalizeOutput, diffLines, planRun } from "../sql.mjs";

test("dbNameFor turns a slug into a database name", () => {
  assert.equal(dbNameFor("crud-in-sql"), "crud_in_sql");
  assert.equal(dbNameFor("a1-build-library-schema"), "a1_build_library_schema");
  assert.throws(() => dbNameFor("bad; DROP DATABASE x"));
});

test("normalizeOutput strips trailing spaces and blank lines, keeps one final newline", () => {
  assert.equal(normalizeOutput(" id | name \r\n----+------\n  1 | Ada  \n\n\n"), " id | name\n----+------\n  1 | Ada\n");
  assert.equal(normalizeOutput(""), "");
});

test("diffLines reports changed, added and removed lines (1-based)", () => {
  assert.deepEqual(diffLines("a\nb\n", "a\nb\n"), []);
  assert.deepEqual(diffLines("a\nb\n", "a\nc\n"), [{ line: 2, expected: "b", actual: "c" }]);
  assert.deepEqual(diffLines("a\n", "a\nb\n"), [{ line: 2, expected: undefined, actual: "b" }]);
});

test("planRun sorts .sql files and finds orphan .out files", () => {
  const r = planRun(["02-select.sql", "01-create.sql", "01-create.out", "09-old.out", "README.md"]);
  assert.deepEqual(r.sql, ["01-create.sql", "02-select.sql"]);
  assert.deepEqual(r.orphans, ["09-old.out"]);
  assert.throws(() => planRun(["select.sql"]), /NN-/);
});
```

Append to `tools/test/lib.test.mjs`:

```js
test("sql: a hand-typed sql fence is an error; include and ignore pass", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/sql/hello/01-a.sql": "SELECT 1;\n" };
  const withFence = (f) => good().replace("### Line by line", `${f}\n\n### Line by line`);
  const bad = checkPage(ctx(withFence("```sql\nSELECT 1;\n```"), files));
  assert.ok(bad.errors.some((e) => e.includes("hand-typed SQL")));
  const inc = checkPage(ctx(withFence("```sql\n{{#include ../../code/sql/hello/01-a.sql}}\n```"), files));
  assert.ok(!inc.errors.some((e) => e.includes("hand-typed")));
  const ign = checkPage(ctx(withFence("```sql,ignore\nSELEC 1;\n```"), files));
  assert.ok(!ign.errors.some((e) => e.includes("hand-typed")));
});
```

- [ ] **Step 2: Run the tests and see them fail.** Run `npm test`. Expected: FAIL with `Cannot find module '…/tools/sql.mjs'`, and the SQL-fence test fails.

- [ ] **Step 3: Implement `tools/sql.mjs`.**

```js
// sql.mjs — pure helpers for tools/sql-check.mjs (no I/O).

export function dbNameFor(slug) {
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error(`not a slug: ${slug}`);
  return slug.replace(/-/g, "_");
}

export function normalizeOutput(text) {
  const lines = text.replace(/\r\n/g, "\n").split("\n").map((l) => l.replace(/\s+$/, ""));
  while (lines.length && lines[lines.length - 1] === "") lines.pop();
  return lines.length ? lines.join("\n") + "\n" : "";
}

export function diffLines(expected, actual) {
  const a = expected.split("\n"), b = actual.split("\n");
  if (a[a.length - 1] === "") a.pop();
  if (b[b.length - 1] === "") b.pop();
  const out = [];
  for (let i = 0; i < Math.max(a.length, b.length); i++)
    if (a[i] !== b[i]) out.push({ line: i + 1, expected: a[i], actual: b[i] });
  return out;
}

export function planRun(entries) {
  const sql = entries.filter((e) => e.endsWith(".sql")).sort();
  for (const f of sql) if (!/^\d{2}-/.test(f)) throw new Error(`${f}: SQL files must start with a two-digit NN- prefix`);
  const bases = new Set(sql.map((f) => f.slice(0, -4)));
  const orphans = entries.filter((e) => e.endsWith(".out") && !bases.has(e.slice(0, -4))).sort();
  return { sql, orphans };
}
```

- [ ] **Step 4: Implement `tools/sql-check.mjs`.**

```js
// sql-check.mjs — runs every code/sql/<slug>/NN-*.sql against a fresh database named after the
// slug and compares psql's real output with the committed NN-*.out.
// Run:  node tools/sql-check.mjs                 (check all)
//       node tools/sql-check.mjs --update crud-in-sql   (rewrite .out for one lesson)
//       node tools/sql-check.mjs --check-twice   (also run each lesson a second time: output must not change)
// RBH_PSQL: shell prefix that runs psql; the database name is appended and SQL comes on stdin.
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { dbNameFor, normalizeOutput, diffLines, planRun } from "./sql.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SQL_DIR = join(ROOT, "code", "sql");
const PSQL = process.env.RBH_PSQL || `docker compose exec -T db sh -c 'psql -X -U postgres -d "$0" 2>&1'`;
const args = process.argv.slice(2);
const UPDATE = args.includes("--update");
const TWICE = args.includes("--check-twice");
const only = args.filter((a) => !a.startsWith("--"));

function psql(db, input) {
  const r = spawnSync("sh", ["-c", `${PSQL} ${db}`], { input, encoding: "utf8", cwd: ROOT });
  if (r.status !== 0) {
    console.error(`psql failed (exit ${r.status}) for database ${db}:\n${r.stdout}${r.stderr}`);
    console.error("Is Postgres running? (docker compose up -d --wait) — or set RBH_PSQL.");
    process.exit(2);
  }
  return r.stdout;
}

let failed = 0;
const slugs = fs.existsSync(SQL_DIR) ? fs.readdirSync(SQL_DIR).filter((d) => fs.statSync(join(SQL_DIR, d)).isDirectory()).sort() : [];
for (const slug of slugs.filter((s) => !only.length || only.includes(s))) {
  const dir = join(SQL_DIR, slug);
  const { sql, orphans } = planRun(fs.readdirSync(dir));
  for (const o of orphans) { console.log(`FAIL  ${slug}/${o}: no matching .sql`); failed++; }
  const db = dbNameFor(slug);
  const runOnce = () => {
    psql("postgres", `DROP DATABASE IF EXISTS ${db} WITH (FORCE);\nCREATE DATABASE ${db};\n`);
    return sql.map((f) => normalizeOutput(psql(db, fs.readFileSync(join(dir, f), "utf8"))));
  };
  const outs = runOnce();
  if (TWICE) {
    const again = runOnce();
    sql.forEach((f, i) => { if (again[i] !== outs[i]) { console.log(`FAIL  ${slug}/${f}: output differs between two runs (nondeterministic SQL?)`); failed++; } });
  }
  sql.forEach((f, i) => {
    const outFile = join(dir, f.replace(/\.sql$/, ".out"));
    if (UPDATE) { fs.writeFileSync(outFile, outs[i]); console.log(`wrote ${slug}/${f.replace(/\.sql$/, ".out")}`); return; }
    if (!fs.existsSync(outFile)) { console.log(`FAIL  ${slug}/${f}: missing .out — run: node tools/sql-check.mjs --update ${slug}`); failed++; return; }
    const d = diffLines(fs.readFileSync(outFile, "utf8"), outs[i]);
    if (d.length) {
      failed++;
      console.log(`FAIL  ${slug}/${f}: output changed — run: node tools/sql-check.mjs --update ${slug}`);
      for (const x of d.slice(0, 10)) console.log(`  line ${x.line}\n    - ${x.expected ?? "(none)"}\n    + ${x.actual ?? "(none)"}`);
    } else console.log(`ok    ${slug}/${f}`);
  });
}
console.log(failed ? `\n${failed} SQL check(s) failed` : "\nall SQL outputs match");
process.exit(failed ? 1 : 0);
```

- [ ] **Step 5: Add the SQL fence rule to `checkPage` in `tools/lib.mjs`.** This is a new sub-step next to the Rust rule. For every `fencedBlocks(md)` block whose `info` starts with `sql`:
  - skip it if the info tags include `ignore`;
  - otherwise, if the body has no `{{#include` line, push `` line ${b.line}: hand-typed SQL in a ```${b.info} fence — put it in code/sql/<slug>/NN-name.sql and {{#include}} it ``;
  - apply the same "mixes" rule as Rust.

- [ ] **Step 6: Run the tests and see them pass.** Run `npm test`. Expected: all pass (53 + 5 = 58).

- [ ] **Step 7: Try the runner for real on a throwaway lesson.** Start the scratch database:

```bash
S=/private/tmp/rbh-sql; mkdir -p $S && sed 's/"5433:5432"/"55433:5432"/' docker-compose.yml > $S/docker-compose.yml
docker compose -p rbh-sql -f $S/docker-compose.yml up -d --wait
export RBH_PSQL="docker compose -p rbh-sql -f $S/docker-compose.yml exec -T db sh -c 'psql -X -U postgres -d \"\$0\" 2>&1'"
mkdir -p code/sql/zz-smoke && printf "SELECT 2 + 3 AS answer;\n" > code/sql/zz-smoke/01-add.sql
node tools/sql-check.mjs zz-smoke; echo "exit=$?"          # expected: FAIL missing .out, exit=1
node tools/sql-check.mjs --update zz-smoke && cat code/sql/zz-smoke/01-add.out
node tools/sql-check.mjs --check-twice zz-smoke; echo "exit=$?"   # expected: ok, exit=0
printf "SELECT 2 + 4 AS answer;\n" > code/sql/zz-smoke/01-add.sql
node tools/sql-check.mjs zz-smoke; echo "exit=$?"          # expected: FAIL output changed with the line diff, exit=1
rm -rf code/sql/zz-smoke
```

Expected `01-add.out`:

```text
 answer
--------
      5
(1 row)
```

Record every command and its output in the report. Stop the scratch database only at the end of the whole plan; later tasks reuse it.

- [ ] **Step 8: Add the CI job.** Append this job to `.github/workflows/code.yml`, at the same indent level as `rust:`:

```yaml
  sql:
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
          --health-cmd "pg_isready -h 127.0.0.1 -U postgres -d rbh"
          --health-interval 2s --health-timeout 3s --health-retries 15
    env:
      # psql runs inside the postgres:18 image (same version as the server); stderr is merged into
      # stdout INSIDE the container so error lines keep their place in the output.
      RBH_PSQL: docker run --rm -i --network host -e PGPASSWORD=postgres postgres:18 sh -c 'psql -X -h 127.0.0.1 -p 5433 -U postgres -d "$0" 2>&1'
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: "24"
      - name: Pull psql image once
        run: docker pull postgres:18
      - name: Every SQL listing runs and matches its committed output (twice)
        run: node tools/sql-check.mjs --check-twice
```

Also add `"sql": "node tools/sql-check.mjs"` to `package.json` scripts.

- [ ] **Step 9: Document the rules.**
  - **CLAUDE.md:** add a `## SQL listings` section. It covers:
    - the file layout and numbering;
    - one database per lesson;
    - the rerunnable preamble;
    - the no-nondeterminism rule;
    - `.out` files come only from `node tools/sql-check.mjs --update <slug>`;
    - how to point `RBH_PSQL` at a non-default Postgres;
    - the validator's SQL-fence rule, and the `sql,ignore` exception for deliberately broken code shown next to its real error.
  - **Spec §4 "Code ↔ book link":** add one paragraph with the same rule.
  - **README.md:** add `npm run sql` to the dev loop.

- [ ] **Step 10: Verify and commit.** Run `npm test && node tools/validate.mjs`. With `RBH_PSQL` set, `node tools/sql-check.mjs` must print `all SQL outputs match` (no lessons yet).

```bash
git add tools package.json .github/workflows/code.yml CLAUDE.md README.md docs/superpowers/specs
git commit -m "feat(tools): CI-checked SQL listings — sql-check runner, sql fence rule, CI job

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>"
```

---

### Tasks 3–9: A1 lessons — shared rules

Each lesson is one task and one commit (`docs(a1): <slug> lesson`). For every lesson:

1. **Topic entry.** In `tools/topics.data.js`, replace its `draft(…)` with a full entry: `status: "published"`, the `outcomes`/`summary`/`prereq`/`next` given in the brief, `codeDir: "code/sql/<slug>"`, `rfhLinks` (if any) and `links` (official docs given in the brief). Run `node tools/generate.mjs`.
2. **SQL files.** Create `code/sql/<slug>/` with the exact SQL files in the brief (verified against Postgres 18.6 when this plan was written). Add the More examples (`50`–`69`), Common-mistakes (`70`–`79`) and Your-turn solution (`90`–`99`) files the brief describes. Then run `node tools/sql-check.mjs --update <slug>` and `node tools/sql-check.mjs --check-twice <slug>` (must pass). Read every `.out`: if an output is not what the lesson's prose will claim, fix the SQL, not the prose.
3. **Page.** Write `src/a1-postgres/<slug>.md`:
   - Every SQL listing is ` ```sql\n{{#include ../../code/sql/<slug>/NN-name.sql}}\n``` `, followed by `### Line by line` (every non-trivial line or clause: What / Why / How / Remove it and…) and `### Run it`.
   - `### Run it` gives the reader's command `docker compose exec -T db psql -U postgres -d <db> < code/sql/<slug>/NN-name.sql`, then ` ```text\n{{#include ../../code/sql/<slug>/NN-name.out}}\n``` `, then the ✅/❌ lines.
   - The first Run it of the lesson also teaches `docker compose exec db createdb -U postgres <db>`, including what `createdb: error: database "<db>" already exists` means on a second try (harmless).
   - Explain psql's command tags (`CREATE TABLE`, `INSERT 0 2`, `(2 rows)`) the first time each appears in this lesson.
   - Common-mistakes errors come from real `70`–`79` files. Show the broken statement via the included `.sql` (it runs in CI and produces the error), and its included `.out`.
   - "Coming from another language?" compares with an ORM or query builder the reader may know (Prisma/Sequelize, SQLAlchemy/Django ORM, JPA/Hibernate, GORM), plus spreadsheets.
4. **Glossary.** Add the brief's new terms to `src/glossary.md` in A–Z order, with "**First used in:**" pointing at this lesson. Link each term's first use in the lesson.
5. **Quiz.** Write `questions/<slug>.json` with exactly the brief's quiz and flashcards. Run `node tools/generate.mjs`.
6. **Verify.** `node tools/validate.mjs` (0 errors, 0 warnings), `mdbook build 2>&1 | grep -E ' (ERROR|WARN) '` (no output), `node tools/sql-check.mjs --check-twice <slug>` (pass). Then do a confused-beginner read-through: no undefined term, no unexplained line, and no command whose output isn't shown.
7. **Commit** the page, SQL + `.out`, quiz, `topics.data.js`, `glossary.md`, and every page whose Next block `generate.mjs` rewrote.

The pages link to each other in order: `tour-of-the-stack → what-is-a-database → tables-rows-and-psql → postgres-data-types → crud-in-sql → keys-and-relations → indexes → sql-transactions → a1-build-library-schema → cheatsheet-sql → hello-axum`. Set each entry's `next` accordingly: lesson N's `next` is lesson N+1. The last lesson also lists the cheat sheet.

---

### Task 3: Lesson `what-is-a-database` (db `what_is_a_database`)

**Files:**
- Create: `code/sql/what-is-a-database/*`, `src/a1-postgres/what-is-a-database.md`, `questions/what-is-a-database.json`
- Modify: `tools/topics.data.js`, `src/glossary.md`

**Topic entry.**
- outcomes:
  - "You can say what a database is and why apps don't keep their data in plain files."
  - "You know what a table, a row and a column are."
  - "You have run your first two SQL queries against your own Postgres."
- summary: "Why apps store data in a database, what Postgres and SQL are, and your first queries."
- prereq `["tour-of-the-stack"]`; next `["tables-rows-and-psql"]`
- links:
  - PostgreSQL tutorial `https://www.postgresql.org/docs/18/tutorial.html`
  - "SQL Language" `https://www.postgresql.org/docs/18/sql.html`

**SQL files (exact):**

`01-first-query.sql`
```sql
SELECT 'Hello, Postgres!' AS greeting;
SELECT 2 + 3 AS answer;
```

`02-first-table.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS friends;
CREATE TABLE friends (
    name text,
    city text
);
INSERT INTO friends (name, city) VALUES ('Ada', 'London'), ('Linus', 'Helsinki');
SELECT * FROM friends;
```

**Brief.**
- **What & why:** a notes app that keeps its data in a text file. Show what breaks: two users saving at once, finding one row in a million, a half-written file after a crash, and a typo like "12,99" in a price column. A database is a program whose only job is to keep data safe, correct and fast to find. Analogy: a library with a librarian, versus a pile of papers on the floor. Postgres is the database we use; SQL is the language we talk to it in.
- **The idea, slowly:**
  1. The shape of data as a ` ```text ` drawing of a table (columns across, rows down), compared with a spreadsheet.
  2. `01-first-query.sql`: `SELECT`, the string in single quotes, `AS` names the column, `;` ends a statement.
  3. `02-first-table.sql`: explain the preamble lines as "start clean so you can rerun this file", then `CREATE TABLE`, the column names and the `text` type (types come in 2 lessons), `INSERT … VALUES` with two rows, and `SELECT *`.
  4. A ` ```text ` diagram: your command → psql → Postgres server (in Docker) → data on disk.
- **You might be wondering…:**
  - "Is Postgres the same as SQL?"
  - "Why not MySQL/SQLite/MongoDB?" (short, fair answer: Postgres is free, strict about data and very common in Rust backends)
  - "Where is my data actually stored?" (the Docker volume from Your toolbox)
  - "Do I have to type uppercase?"
- **Common mistakes** (`70-mistakes.sql`):
  - a double-quoted string `SELECT "Hello";` (real error: column "Hello" does not exist)
  - a missing `;` between two statements in one file (real syntax error)
- **More examples** (4, files `50`–`53`): arithmetic and text joining (`||`); `SELECT` several values at once; a second table `pets (name text, kind text)` with 3 rows; `SELECT name FROM friends` (one column).
- **Your turn:**
  - 🟢 run both files in your own db;
  - 🟡 add yourself to `friends` (solution file `90-add-me.sql`);
  - 🔴 create a `books (title text, author text)` table with two books and select them (solution file `91-books.sql`).
- **Glossary terms to add:** `table`, `row`, `column`, `query`, `psql` (heading `### psql`).
- **Quiz (exact):**
  1. q: "What is a database?"; options: ["A program whose job is to store data safely and find it fast", "A spreadsheet file", "A folder of text files", "A web server"]; answer 0; explain: "A database server like Postgres stores data, keeps it correct, and finds it fast — even with many users at once."
  2. q: "In SQL, how do you write the text Hello?"; options: ["\"Hello\"", "'Hello'", "`Hello`", "Hello"]; answer 1; explain: "Text values use single quotes. Double quotes mean a column or table name, which is why \"Hello\" gives 'column does not exist'."
  3. q: "What does `AS answer` do in `SELECT 2 + 3 AS answer;`?"; options: ["Stores 5 in a variable", "Names the result column 'answer'", "Creates a table", "Nothing"]; answer 1; explain: "AS gives the output column a name you can read."
  4. q: "What is a row?"; options: ["One column's type", "One record in a table, like one friend", "A SQL command", "A database"]; answer 1; explain: "Each row is one thing you stored — one friend, one book. Columns are the fields every row has."
- **Flashcards:**
  - front "Table vs row vs column?"; back "Table = the whole list. Row = one record. Column = one field every record has."
  - front "Run a SQL file against a lesson database?"; back "`docker compose exec -T db psql -U postgres -d <db> < file.sql`"

### Task 4: Lesson `tables-rows-and-psql` (db `tables_rows_and_psql`)

**Topic entry.**
- outcomes:
  - "You can create a table with named, typed columns."
  - "You can insert rows and select exactly the columns you want."
  - "You can find your way around with psql's backslash commands."
- summary: "Create tables, add and read rows, and explore your database with psql's backslash commands."
- prereq `["what-is-a-database"]`; next `["postgres-data-types"]`
- links:
  - psql docs `https://www.postgresql.org/docs/18/app-psql.html`
  - CREATE TABLE `https://www.postgresql.org/docs/18/sql-createtable.html`

**SQL files (exact):**

`01-create-members.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS members;
CREATE TABLE members (
    id     integer,
    name   text,
    email  text,
    joined date
);
\d members
```

`02-rows.sql`
```sql
INSERT INTO members (id, name, email, joined) VALUES
    (1, 'Ada Lovelace', 'ada@example.com', '2026-01-15'),
    (2, 'Alan Turing', 'alan@example.com', '2026-02-01');
SELECT * FROM members;
SELECT name, joined FROM members;
\dt
```

`70-mistakes.sql`
```sql
SELEC * FROM members;
SELECT * FROM member;
SELECT nickname FROM members;
```

**Brief.**
- **What & why:** a table is a promise about shape: every member has these four fields. psql is your window into the database. Analogy: a form with fixed boxes.
- **The idea, slowly:**
  - interactive psql first: `docker compose exec db psql -U postgres -d tables_rows_and_psql`, the prompt `tables_rows_and_psql=#`, and `\q` to leave, shown as a ` ```text ` transcript;
  - then `01`, where `\d members` shows the table's shape, and each output column (Column/Type/Collation/Nullable/Default) is explained;
  - then `02`: the multi-row INSERT, `INSERT 0 2` (the 0 is a historical leftover, always 0), selecting columns by name, and `\dt`.
- **You might be wondering…:**
  - "Why does `\d` have no semicolon?" (backslash commands belong to psql, not SQL)
  - "Can I change a table later?" (yes, ALTER TABLE, shown in More examples)
  - "What's `id` for if nothing checks it?" (keys come in Keys and relations)
  - "Why is the date in quotes?"
- **Common mistakes:** the three errors of `70-mistakes.sql`, each with its real message, the `^` caret explained, and the fix.
- **More examples** (`50`–`53`):
  - `ALTER TABLE members ADD COLUMN phone text;` + `\d members`
  - `SELECT` with a column alias
  - `\l` to list databases (**output varies per machine**, so do NOT include `\l` in a checked file; show it as a described command only, in prose, no fence)
  - `\?` described in prose only
  - `DROP TABLE` in a throwaway `scratch` table
- **Your turn:**
  - 🟢 run `\d members` interactively;
  - 🟡 add a third member (`90`);
  - 🔴 create `events (id integer, title text, happens_on date)` with 2 rows and show only titles (`91`).
- **Glossary:** `backslash command`.
- **Quiz (exact):**
  1. q: "What does `\\d members` show?"; options: ["The rows in members", "The columns and types of members", "All databases", "It deletes members"]; answer 1; explain: "\\d describes a table's shape: its columns, their types and rules."
  2. q: "Why doesn't `\\dt` end with a semicolon?"; options: ["It's a typo", "It's a psql command, not SQL", "Semicolons are optional everywhere", "It only works in scripts"]; answer 1; explain: "Backslash commands are handled by psql itself and end at the end of the line."
  3. q: "`INSERT 0 2` means…"; options: ["Error in row 0", "2 rows were inserted", "0 rows were inserted", "The table has 2 columns"]; answer 1; explain: "The last number is how many rows were inserted; the 0 is a historical leftover that is always 0."
  4. q: "`ERROR: relation \"member\" does not exist` most likely means…"; options: ["Postgres is down", "You misspelled the table name", "The table is empty", "You need a semicolon"]; answer 1; explain: "Relation means table here. Check the spelling with \\dt."
- **Flashcards:**
  - "psql: list tables / describe one / quit?" → "`\\dt` / `\\d name` / `\\q`"
  - "Pick columns instead of all?" → "`SELECT name, joined FROM members;`"

### Task 5: Lesson `postgres-data-types` (db `postgres_data_types`)

**Topic entry.**
- outcomes:
  - "You can choose a sensible type for numbers, money, text, true/false, dates and IDs."
  - "You know why money is stored as whole cents."
  - "You know what NULL means and how to test for it."
- summary: "Integers, exact numbers, text, booleans, dates, timestamps, UUIDs and NULL — and how to choose."
- prereq `["tables-rows-and-psql"]`; next `["crud-in-sql"]`
- links: data types `https://www.postgresql.org/docs/18/datatype.html`

**SQL files (exact):**

`01-numbers.sql`
```sql
SELECT 7 / 2 AS whole_numbers, 7 / 2.0 AS with_decimals;
SELECT 0.1::double precision + 0.2::double precision AS floating_point,
       0.1::numeric + 0.2::numeric AS exact_numeric;
SELECT 2147483647 + 1 AS too_big_for_integer;
SELECT 2147483647::bigint + 1 AS fits_in_bigint;
```

`02-text-bool-dates.sql`
```sql
SELECT 'Rust' || ' ' || 'book' AS joined_text, length('héllo') AS letters;
SELECT true AS in_stock, DATE '2026-09-24' AS published, DATE '2026-09-24' + 14 AS due_back;
SELECT TIMESTAMPTZ '2026-09-24 10:30:00+06' AS dhaka_time_shown_in_utc;
```

`03-typed-table.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS products;
CREATE TABLE products (
    id          bigint,
    name        text,
    price_cents integer,
    in_stock    boolean,
    added_on    date,
    sku         uuid
);
INSERT INTO products VALUES (1, 'Mug', 1299, true, '2026-09-01', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11');
INSERT INTO products VALUES (2, 'Poster', 'cheap', true, '2026-09-02', NULL);
INSERT INTO products VALUES (3, 'Pen', 199, NULL, NULL, NULL);
SELECT * FROM products;
SELECT name FROM products WHERE in_stock IS NULL;
SELECT name FROM products WHERE in_stock = NULL;
```

The expected highlights, verified:
- `7 / 2` = 3
- `0.30000000000000004` vs `0.3`
- `ERROR:  integer out of range`
- `due_back` 2026-10-08
- the Dhaka time shows as `2026-09-24 04:30:00+00`
- the 'cheap' insert fails with a caret
- `= NULL` returns `(0 rows)`

**Brief.**
- **What & why:** a type is a promise the database enforces. Link back to the "price as cents" answer in How a web backend works.
- **The idea, slowly:** one listing per file above, and a ` ```text ` "which type do I pick?" table (whole numbers → integer/bigint, money → integer cents or numeric, text → text, yes/no → boolean, calendar day → date, moment in time → timestamptz, public IDs → uuid).
- `::` is the cast operator: explain it where it appears.
- Mention `varchar(n)` vs `text` in the FAQ (use text; add a CHECK when you need a limit).
- **You might be wondering…:**
  - "Why does `7/2` give 3?"
  - "timestamp vs timestamptz?" (always timestamptz; it stores a moment and shows it in the session time zone, UTC here)
  - "Why not float for money?"
  - "What is NULL — zero? empty string?" (neither: unknown/missing)
- **Common mistakes** (`70`–`71`): comparing `= NULL` (use `IS NULL`); a date in the wrong format `'24/09/2026'` (real error text from the file).
- **More examples** (`50`–`53`):
  - `numeric(10,2)` rounding
  - `upper()`/`lower()`
  - date subtraction giving a number of days
  - `COALESCE(in_stock, false)`
- **Your turn:**
  - 🟢 run `01` and explain each result;
  - 🟡 add a `weight_grams integer` column and set it (`90`);
  - 🔴 design `events (id bigint, title text, starts_at timestamptz, is_free boolean, price_cents integer)` with 2 rows (`91`).
- **Glossary:** `data type`, `NULL`, `cast`.
- **Quiz (exact):**
  1. q: "Best type for a price like 12.99 dollars?"; options: ["double precision", "integer holding 1299 cents", "text", "boolean"]; answer 1; explain: "Whole cents in an integer are exact. Floating-point types can't store 0.1 exactly."
  2. q: "`SELECT 7 / 2;` returns…"; options: ["3.5", "3", "4", "an error"]; answer 1; explain: "Both sides are integers, so Postgres does whole-number division. Use 7 / 2.0 for 3.5."
  3. q: "How do you find rows where in_stock is unknown?"; options: ["WHERE in_stock = NULL", "WHERE in_stock IS NULL", "WHERE in_stock = ''", "WHERE in_stock = 0"]; answer 1; explain: "NULL means unknown; comparing with = never matches. Use IS NULL."
  4. q: "Which type stores a moment in time correctly across time zones?"; options: ["date", "text", "timestamptz", "integer"]; answer 2; explain: "timestamptz stores the exact moment and displays it in your session's time zone."
- **Flashcards:**
  - "integer vs bigint?" → "integer goes up to about 2.1 billion; bigint to about 9.2 quintillion. Use bigint for IDs that may grow large."
  - "NULL means…?" → "Unknown or missing — not 0 and not ''. Test with IS NULL / IS NOT NULL."

### Task 6: Lesson `crud-in-sql` (db `crud_in_sql`)

**Topic entry.**
- outcomes:
  - "You can create, read, update and delete rows."
  - "You can filter, sort and limit results."
  - "You know why an UPDATE or DELETE without WHERE is dangerous."
- summary: "Create, Read, Update, Delete — the four things every backend does to data, in SQL."
- prereq `["postgres-data-types"]`; next `["keys-and-relations"]`
- links: SELECT `https://www.postgresql.org/docs/18/sql-select.html`, UPDATE `https://www.postgresql.org/docs/18/sql-update.html`, DELETE `https://www.postgresql.org/docs/18/sql-delete.html`

**SQL files (exact):**

`01-create-and-insert.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS books;
CREATE TABLE books (
    id     integer,
    title  text,
    author text,
    year   integer,
    copies integer
);
INSERT INTO books (id, title, author, year, copies) VALUES
    (1, 'Dune', 'Frank Herbert', 1965, 3),
    (2, 'Emma', 'Jane Austen', 1815, 1),
    (3, 'Neuromancer', 'William Gibson', 1984, 2),
    (4, 'Persuasion', 'Jane Austen', 1817, 0);
```

`02-select.sql`
```sql
SELECT title, year FROM books;
SELECT title FROM books WHERE author = 'Jane Austen';
SELECT title, year FROM books WHERE year > 1900 ORDER BY year DESC;
SELECT title FROM books ORDER BY title LIMIT 2;
SELECT count(*) AS austen_books FROM books WHERE author = 'Jane Austen';
```

`03-update.sql`
```sql
UPDATE books SET copies = copies + 1 WHERE id = 4;
SELECT id, title, copies FROM books WHERE id = 4;
```

`04-delete.sql`
```sql
DELETE FROM books WHERE id = 2;
SELECT id, title FROM books ORDER BY id;
```

`70-forgot-where.sql`
```sql
BEGIN;
UPDATE books SET copies = 0;
SELECT title, copies FROM books ORDER BY id;
ROLLBACK;
SELECT title, copies FROM books ORDER BY id;
```

**Brief.**
- **What & why:** every screen of every app is CRUD: a shop's product page (Read), "add to cart" (Create), "change quantity" (Update), "remove" (Delete). Show the HTTP ↔ SQL mapping from How a web backend works: POST→INSERT, GET→SELECT, PUT→UPDATE, DELETE→DELETE.
- **The idea, slowly:** one section per file, with every clause (WHERE, ORDER BY … DESC, LIMIT, count(*)) explained. Note that without ORDER BY, row order is not guaranteed.
- **You might be wondering…:**
  - "Why does DELETE print `DELETE 1`?"
  - "Can I undo a DELETE?" (only inside a transaction; the next lessons)
  - "Is there a way to get the new row back after INSERT?" (RETURNING, in More examples)
  - "Why not SELECT * everywhere?"
- **Common mistakes:**
  - `70-forgot-where.sql`: an UPDATE without WHERE changes every row. It is wrapped in BEGIN/ROLLBACK here so it's safe, and you'll learn those in SQL transactions.
  - `71`: string comparison is case-sensitive (`WHERE author = 'jane austen'` returns 0 rows); ILIKE is the fix.
- **More examples** (`50`–`53`): `INSERT … RETURNING id, title`; `WHERE … AND …`; `LIKE 'Per%'`; `UPDATE … SET a = …, b = …` two columns.
- **Your turn:**
  - 🟢 select the books from after 1950;
  - 🟡 add a book and give it 5 copies (`90`);
  - 🔴 delete every book with 0 copies, then show what's left (`91`).
- **Glossary:** `CRUD`, `WHERE clause` (heading `### WHERE`).
- **Quiz (exact):**
  1. q: "Which SQL statement matches an HTTP POST that creates something?"; options: ["SELECT", "INSERT", "UPDATE", "DELETE"]; answer 1; explain: "POST creates, INSERT adds a new row."
  2. q: "What happens with `UPDATE books SET copies = 0;` (no WHERE)?"; options: ["Error", "Only the first row changes", "Every row changes", "Nothing"]; answer 2; explain: "Without WHERE, UPDATE and DELETE apply to every row in the table."
  3. q: "Without ORDER BY, the order of rows is…"; options: ["Alphabetical", "By id", "Not guaranteed", "Newest first"]; answer 2; explain: "Postgres returns rows in whatever order is fastest. Always ORDER BY when order matters."
  4. q: "`SELECT title FROM books ORDER BY title LIMIT 2;` returns…"; options: ["The last 2 titles", "The first 2 titles alphabetically", "2 random titles", "Titles with 2 letters"]; answer 1; explain: "ORDER BY sorts A→Z, LIMIT keeps the first 2."
- **Flashcards:**
  - "CRUD in SQL?" → "Create = INSERT, Read = SELECT, Update = UPDATE, Delete = DELETE."
  - "Newest first?" → "`ORDER BY year DESC`"

### Task 7: Lesson `keys-and-relations` (db `keys_and_relations`)

**Topic entry.**
- outcomes:
  - "You can give every row a unique ID the database manages."
  - "You can link tables with foreign keys and join them back together."
  - "You can model one-to-many and many-to-many relationships."
- summary: "Primary keys, unique rules, foreign keys, JOIN, and modelling one-to-many and many-to-many."
- prereq `["crud-in-sql"]`; next `["indexes"]`
- links: constraints `https://www.postgresql.org/docs/18/ddl-constraints.html`, joins tutorial `https://www.postgresql.org/docs/18/tutorial-join.html`

**SQL files (exact):**

`01-primary-keys.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS book_tags, tags, books, authors;
CREATE TABLE authors (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert');
SELECT * FROM authors;
INSERT INTO authors (name) VALUES ('Jane Austen');
INSERT INTO authors (id, name) VALUES (7, 'Someone');
```

`02-foreign-keys.sql`
```sql
CREATE TABLE books (
    id        integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title     text NOT NULL,
    author_id integer NOT NULL REFERENCES authors (id)
);
INSERT INTO books (title, author_id) VALUES ('Emma', 1), ('Persuasion', 1), ('Dune', 2);
INSERT INTO books (title, author_id) VALUES ('Ghost book', 99);
SELECT * FROM books;
```

`03-join.sql`
```sql
SELECT books.title, authors.name AS author
FROM books
JOIN authors ON authors.id = books.author_id
ORDER BY books.id;
```

`04-many-to-many.sql`
```sql
CREATE TABLE tags (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
CREATE TABLE book_tags (
    book_id integer NOT NULL REFERENCES books (id) ON DELETE CASCADE,
    tag_id  integer NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);
INSERT INTO tags (name) VALUES ('classic'), ('sci-fi'), ('romance');
INSERT INTO book_tags (book_id, tag_id) VALUES (1, 1), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2);
SELECT books.title, tags.name AS tag
FROM book_tags
JOIN books ON books.id = book_tags.book_id
JOIN tags  ON tags.id  = book_tags.tag_id
WHERE tags.name = 'classic'
ORDER BY books.title;
```

`05-on-delete.sql`
```sql
DELETE FROM authors WHERE name = 'Frank Herbert';
DELETE FROM books WHERE title = 'Dune';
SELECT count(*) AS dune_tag_rows FROM book_tags WHERE book_id = 3;
DELETE FROM authors WHERE name = 'Frank Herbert';
```

`50-count-per-author.sql` (More example: why LEFT JOIN)
```sql
INSERT INTO authors (name) VALUES ('Ursula K. Le Guin');
SELECT authors.name, count(books.id) AS books
FROM authors
LEFT JOIN books ON books.author_id = authors.id
GROUP BY authors.name
ORDER BY authors.name;
```

The expected highlights, verified:
- the unique violation `authors_name_key`
- `cannot insert a non-DEFAULT value into column "id"` with its HINT
- the FK violation `books_author_id_fkey`
- the RESTRICT error on deleting Frank Herbert
- the cascade leaving `dune_tag_rows` = 0
- `Ursula K. Le Guin | 0`

**Brief.**
- **What & why:** two members named "Alan Turing" means you need an ID. Copying the author's name into every book row means a typo in one row, so it should be stored once and pointed to. Analogy: a phone's contact list, where messages point to a contact rather than copying the phone number.
- **The idea, slowly:** one section per file.
  - "GENERATED ALWAYS AS IDENTITY" = "Postgres numbers the rows for you".
  - `PRIMARY KEY` = unique + not null + the row's identity. `UNIQUE`, `NOT NULL` and `REFERENCES` are each explained.
  - A ` ```text ` diagram for one-to-many (authors 1 → many books) and many-to-many (books ↔ book_tags ↔ tags).
  - `JOIN … ON` read aloud in English.
  - `ON DELETE CASCADE` vs the default (refuse).
- **You might be wondering…:**
  - "serial vs identity?" (identity is the modern standard)
  - "UUID or integer IDs?" (integers here; UUIDs in the capstone for public IDs)
  - "Why is `book_tags`' primary key two columns?"
  - "Should I always CASCADE?" (no: pick deliberately)
- **Common mistakes** (`70`–`71`):
  - joining without ON, in a `sql,ignore` fence: show the real syntax error by running `SELECT * FROM books JOIN authors;` in `70`
  - an ambiguous column `SELECT id FROM books JOIN authors ON …` (`71`: the real "column reference "id" is ambiguous")
- **More examples** (`50`–`53`): `50` above; LEFT JOIN vs JOIN; aliases `b`/`a`; which tags does Emma have.
- **Your turn:**
  - 🟢 join books and authors yourself;
  - 🟡 add a tag 'favourite' to Persuasion (`90`);
  - 🔴 create `reviews (id identity pk, book_id FK, stars integer CHECK 1–5, body text)` and join them to titles (`91`).
- **Glossary:** `primary key`, `foreign key`, `constraint`, `join`.
- **Quiz (exact):**
  1. q: "What does a foreign key guarantee?"; options: ["The column is unique", "The value points at a row that really exists in another table", "The column is indexed", "The value is never NULL"]; answer 1; explain: "REFERENCES makes Postgres refuse values that don't exist in the other table."
  2. q: "How do you model books ↔ tags (many-to-many)?"; options: ["A tags text column in books", "A third table holding (book_id, tag_id) pairs", "A foreign key from tags to books", "It's impossible in SQL"]; answer 1; explain: "A join table like book_tags stores one row per pairing."
  3. q: "`GENERATED ALWAYS AS IDENTITY` means…"; options: ["You must type every id", "Postgres picks the id and you can't override it by accident", "The id is random", "The id is text"]; answer 1; explain: "Postgres numbers rows itself; inserting your own id is refused unless you say OVERRIDING SYSTEM VALUE."
  4. q: "Deleting an author who still has books fails because…"; options: ["Authors can't be deleted", "The foreign key's default rule refuses to leave books pointing at nothing", "You need CASCADE on authors", "The table is locked"]; answer 1; explain: "By default a foreign key blocks deleting a row that others still reference."
- **Flashcards:**
  - "One-to-many in SQL?" → "Put a foreign key column on the 'many' side (books.author_id → authors.id)."
  - "JOIN vs LEFT JOIN?" → "JOIN keeps only matches; LEFT JOIN keeps every left row, with NULLs where nothing matches."

### Task 8: Lesson `indexes` (db `indexes`)

**Topic entry.**
- outcomes:
  - "You can explain what an index is with the back-of-the-book analogy."
  - "You can read EXPLAIN to see whether Postgres scanned every row."
  - "You know when an index helps and when it doesn't."
- summary: "Make lookups fast with indexes, read EXPLAIN, and learn when an index isn't worth it."
- prereq `["keys-and-relations"]`; next `["sql-transactions"]`
- links: indexes `https://www.postgresql.org/docs/18/indexes.html`, EXPLAIN `https://www.postgresql.org/docs/18/using-explain.html`

**SQL files (exact):**

`01-setup.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS orders;
CREATE TABLE orders (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    customer_id integer NOT NULL,
    status      text NOT NULL,
    total_cents integer NOT NULL
);
INSERT INTO orders (customer_id, status, total_cents)
SELECT n % 5000,
       CASE WHEN n % 10 = 0 THEN 'refunded' ELSE 'paid' END,
       (n * 37) % 10000
FROM generate_series(1, 100000) AS n;
ANALYZE orders;
SELECT count(*) AS orders FROM orders;
```

`02-without-index.sql`
```sql
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE customer_id = 42;
```

`03-with-index.sql`
```sql
CREATE INDEX orders_customer_id_idx ON orders (customer_id);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF)
SELECT * FROM orders WHERE customer_id = 42;
```

`04-when-not-to-index.sql`
```sql
CREATE INDEX orders_status_idx ON orders (status);
ANALYZE orders;
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'paid';
EXPLAIN (COSTS OFF) SELECT * FROM orders WHERE status = 'refunded';
\di
```

The expected highlights, verified:
- `Seq Scan … Rows Removed by Filter: 99980`
- after the index, `Bitmap Heap Scan … Bitmap Index Scan on orders_customer_id_idx … Index Searches: 1`
- 'paid' (90% of rows) still uses `Seq Scan`, while 'refunded' uses the index

**Brief.**
- **What & why:** finding one customer's orders in 100,000 rows by reading every row is like finding "Postgres" in a book by reading every page. An index is the book's index at the back.
- **The idea, slowly:**
  - `generate_series` and `CASE` are introduced as "a way to make lots of fake rows": explain enough to read the file, without teaching them in depth;
  - `ANALYZE` = "let Postgres count what's in the table so it can plan well";
  - EXPLAIN line by line: node names, `Filter`, `Rows Removed by Filter`, `Bitmap Index Scan`, `Recheck Cond`, `Heap Blocks`, `Index Searches`. The `(ANALYZE, COSTS OFF, …)` options mean "run it for real, but hide the numbers that change every run";
  - why the planner ignores the status index for 'paid' (reading 90% of the table via an index is slower);
  - primary keys and UNIQUE get an index automatically (visible in `\di`);
  - the cost: indexes use disk and slow down writes.
- **You might be wondering…:**
  - "Why not index every column?"
  - "Does the primary key already have an index?" (yes, orders_pkey)
  - "Should foreign key columns be indexed?" (usually yes; Postgres doesn't do it automatically)
  - "Why show 'actual rows' but no times?"
- **Common mistakes** (`70`–`71`):
  - an index on a function-wrapped column not being used: `WHERE lower(status) = 'refunded'` with EXPLAIN (COSTS OFF). The plain `status` index can't serve `lower(status)`, so the real output shows a Seq Scan, while `status = 'refunded'` used the index. The fix, an expression index `ON orders (lower(status))`, goes in the same file with a second EXPLAIN;
  - expecting an index to help `LIKE '%foo'` (EXPLAIN shows Seq Scan).
- **More examples** (`50`–`53`):
  - a multi-column index `(customer_id, status)`
  - a UNIQUE index enforcing unique emails on a small `users` table, with the duplicate insert error
  - `DROP INDEX`
  - a partial index `WHERE status = 'refunded'`
- **Your turn:**
  - 🟢 run `02` and `03` and compare "Rows Removed";
  - 🟡 index `total_cents` and EXPLAIN `WHERE total_cents = 1234` (`90`);
  - 🔴 decide whether `status` deserves an index, prove it with EXPLAIN, and explain in the solution (`91`).
- **Glossary:** `index`, `EXPLAIN`, `query planner`.
- **Quiz (exact):**
  1. q: "What problem does an index solve?"; options: ["Storing more data", "Finding matching rows without reading the whole table", "Backing up data", "Enforcing types"]; answer 1; explain: "Like a book's index, it points straight at the rows you want."
  2. q: "In EXPLAIN output, `Seq Scan` with `Rows Removed by Filter: 99980` means…"; options: ["An index was used", "Postgres read every row and threw away 99,980", "99,980 rows were deleted", "The query failed"]; answer 1; explain: "A sequential scan reads the whole table; the filter discards non-matching rows."
  3. q: "Why might Postgres ignore an index on `status` for `status = 'paid'`?"; options: ["The index is broken", "90% of rows match, so reading the table directly is faster", "Text columns can't be indexed", "You forgot ANALYZE"]; answer 1; explain: "Indexes win when few rows match. For most of the table, a plain scan is cheaper."
  4. q: "What's a cost of adding an index?"; options: ["Queries can't use WHERE", "Extra disk space and slower INSERT/UPDATE", "The table becomes read-only", "None"]; answer 1; explain: "Every write must also update the index, and the index takes space."
- **Flashcards:**
  - "Create an index?" → "`CREATE INDEX orders_customer_id_idx ON orders (customer_id);`"
  - "See how Postgres runs a query?" → "`EXPLAIN (COSTS OFF) SELECT …;` — add ANALYZE to really run it."

### Task 9: Lesson `sql-transactions` (db `sql_transactions`)

**Topic entry.**
- outcomes:
  - "You can group changes so they all happen or none do."
  - "You can undo work with ROLLBACK."
  - "You know what an aborted transaction is and how to recover."
- summary: "BEGIN, COMMIT and ROLLBACK: all-or-nothing changes, and why a bank transfer needs them."
- prereq `["indexes"]`; next `["a1-build-library-schema", "cheatsheet-sql"]`
- links: tutorial `https://www.postgresql.org/docs/18/tutorial-transactions.html`, BEGIN `https://www.postgresql.org/docs/18/sql-begin.html`

**SQL files (exact):**

`01-setup.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS accounts;
CREATE TABLE accounts (
    id            integer PRIMARY KEY,
    owner         text NOT NULL,
    balance_cents integer NOT NULL CHECK (balance_cents >= 0)
);
INSERT INTO accounts (id, owner, balance_cents) VALUES (1, 'Ada', 10000), (2, 'Linus', 5000);
SELECT * FROM accounts ORDER BY id;
```

`02-without-transaction.sql`
```sql
UPDATE accounts SET balance_cents = balance_cents + 20000 WHERE id = 2;
UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
UPDATE accounts SET balance_cents = 5000 WHERE id = 2;
```

`03-transfer-commit.sql`
```sql
BEGIN;
UPDATE accounts SET balance_cents = balance_cents - 2500 WHERE id = 1;
UPDATE accounts SET balance_cents = balance_cents + 2500 WHERE id = 2;
COMMIT;
SELECT * FROM accounts ORDER BY id;
```

`04-rollback.sql`
```sql
BEGIN;
UPDATE accounts SET balance_cents = 0 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
ROLLBACK;
SELECT * FROM accounts ORDER BY id;
```

`05-failure-inside.sql`
```sql
BEGIN;
UPDATE accounts SET balance_cents = balance_cents + 20000 WHERE id = 2;
UPDATE accounts SET balance_cents = balance_cents - 20000 WHERE id = 1;
SELECT * FROM accounts ORDER BY id;
COMMIT;
SELECT * FROM accounts ORDER BY id;
```

The expected highlights, verified:
- `02` creates money: Linus shows 25000 while Ada keeps 10000, because the second UPDATE failed on the CHECK. The last line repairs the data.
- `03` commits 7500 / 7500.
- `04` shows 0 inside and 7500 after ROLLBACK.
- `05` fails with the CHECK error, then `current transaction is aborted…`, and `COMMIT` answers `ROLLBACK`: balances are unchanged.

**Brief.**
- **What & why:** a transfer is two changes. If the app crashes between them, money appears or vanishes. Show the real `02` output first (the "why"). Analogy: a shop checkout, where the goods and the payment happen together or not at all.
- **The idea, slowly:**
  - one section per file, with CHECK constraints explained where `01` introduces them;
  - ACID in plain words: all-or-nothing, rules always hold, others don't see half-done work, once committed it stays;
  - a ` ```text ` timeline diagram of BEGIN → changes → COMMIT/ROLLBACK;
  - forward pointer: SeaORM transactions (A3) and checkout in the capstone use exactly this.
- **You might be wondering…:**
  - "Does every single statement run in a transaction?" (yes: autocommit, one-statement transactions)
  - "What do other connections see mid-transaction?" (nothing until COMMIT)
  - "Why did COMMIT print ROLLBACK?"
  - "Can transactions be too long?" (yes, they hold locks; keep them short)
- **Common mistakes** (`70`–`71`):
  - forgetting COMMIT: show that `BEGIN; UPDATE …;` then the file ends (psql session ends) means the change is gone. `70` is BEGIN + UPDATE with no COMMIT, and `71` selects afterwards to prove it.
  - carrying on after an error: the aborted state from `05`, and the fix is ROLLBACK.
- **More examples** (`50`–`53`):
  - SAVEPOINT and ROLLBACK TO in a 3-step change
  - a transaction that inserts an order and its items (tiny two tables)
  - a transaction around a DELETE for safety (select before and after, then ROLLBACK)
  - `BEGIN; … COMMIT;` with a count check
- **Your turn:**
  - 🟢 run `03` and `04`;
  - 🟡 transfer 1000 cents from Linus to Ada in a transaction (`90`);
  - 🔴 write a transfer that would overdraw, run it, and show the balances are untouched (`91`).
- **Glossary:** `transaction`, `commit`, `rollback`, `ACID`.
- **Quiz (exact):**
  1. q: "Why wrap a money transfer in a transaction?"; options: ["It's faster", "So both updates happen or neither does", "To lock the database forever", "Postgres requires it"]; answer 1; explain: "A transaction makes the two changes all-or-nothing, so money never appears or disappears."
  2. q: "What does ROLLBACK do?"; options: ["Saves the changes", "Undoes every change since BEGIN", "Deletes the table", "Restarts Postgres"]; answer 1; explain: "ROLLBACK throws away all changes made inside the current transaction."
  3. q: "After an error inside BEGIN, the next statements say `current transaction is aborted`. What now?"; options: ["Keep going", "ROLLBACK (or COMMIT, which rolls back) and start again", "Restart the computer", "DROP the table"]; answer 1; explain: "Once a statement fails, the transaction can only be ended. Postgres ignores everything until then."
  4. q: "Without BEGIN, each statement…"; options: ["Is never saved", "Is its own tiny transaction, saved immediately", "Waits for COMMIT", "Is rolled back"]; answer 1; explain: "By default Postgres autocommits each statement on its own."
- **Flashcards:**
  - "All-or-nothing changes?" → "`BEGIN; …; COMMIT;` — or `ROLLBACK;` to undo."
  - "ACID in one line?" → "All-or-nothing, rules always hold, others don't see half-done work, committed means saved."

### Task 10: Project `a1-build-library-schema` (db `a1_build_library_schema`)

**Files:**
- Create: `code/sql/a1-build-library-schema/*`, `src/a1-postgres/a1-build-library-schema.md`
- Modify: `tools/topics.data.js`

**Topic entry:**
- `kind: "project"`, `level: "Beginner"`, `status: "published"`
- outcomes: ["You designed a real schema of four related tables.", "You wrote the queries a library app needs.", "You changed data safely with transactions."]
- summary: "Design and query a library database — authors, books, members and loans — using everything from Part A1."
- prereq `["sql-transactions"]`; next `["cheatsheet-sql", "hello-axum"]`
- codeDir `code/sql/a1-build-library-schema`
- The project has no quiz (projects are not quizzable).

**Reference solution SQL (exact, verified):**

`01-schema.sql`
```sql
SET client_min_messages = warning;
DROP TABLE IF EXISTS loans, books, members, authors;
CREATE TABLE authors (
    id   integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text NOT NULL UNIQUE
);
CREATE TABLE books (
    id               integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title            text    NOT NULL,
    author_id        integer NOT NULL REFERENCES authors (id),
    copies_total     integer NOT NULL CHECK (copies_total >= 0),
    copies_available integer NOT NULL CHECK (copies_available BETWEEN 0 AND copies_total)
);
CREATE TABLE members (
    id        integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name      text NOT NULL,
    email     text NOT NULL UNIQUE,
    joined_on date NOT NULL
);
CREATE TABLE loans (
    id          integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    book_id     integer NOT NULL REFERENCES books (id),
    member_id   integer NOT NULL REFERENCES members (id),
    loaned_on   date    NOT NULL,
    due_on      date    NOT NULL CHECK (due_on > loaned_on),
    returned_on date             CHECK (returned_on >= loaned_on)
);
CREATE INDEX loans_book_id_idx   ON loans (book_id);
CREATE INDEX loans_member_id_idx ON loans (member_id);
\dt
```

`02-seed.sql`
```sql
INSERT INTO authors (name) VALUES ('Jane Austen'), ('Frank Herbert'), ('Ursula K. Le Guin');
INSERT INTO books (title, author_id, copies_total, copies_available) VALUES
    ('Emma',                     1, 2, 1),
    ('Persuasion',               1, 1, 1),
    ('Dune',                     2, 3, 1),
    ('A Wizard of Earthsea',     3, 2, 2),
    ('The Left Hand of Darkness', 3, 1, 1);
INSERT INTO members (name, email, joined_on) VALUES
    ('Ada Lovelace', 'ada@example.com',  '2026-01-15'),
    ('Alan Turing',  'alan@example.com', '2026-02-01'),
    ('Grace Hopper', 'grace@example.com', '2026-03-10');
INSERT INTO loans (book_id, member_id, loaned_on, due_on, returned_on) VALUES
    (1, 1, '2026-09-01', '2026-09-15', NULL),
    (3, 1, '2026-09-10', '2026-09-24', NULL),
    (3, 2, '2026-09-20', '2026-10-04', NULL),
    (2, 3, '2026-08-01', '2026-08-15', '2026-08-14'),
    (3, 3, '2026-07-01', '2026-07-15', '2026-07-20');
```

`03-queries.sql`
```sql
-- The catalogue: every book with its author.
SELECT books.title, authors.name AS author, books.copies_available
FROM books
JOIN authors ON authors.id = books.author_id
ORDER BY authors.name, books.title;

-- Who has what right now (not returned yet)?
SELECT members.name AS member, books.title, loans.due_on
FROM loans
JOIN members ON members.id = loans.member_id
JOIN books   ON books.id   = loans.book_id
WHERE loans.returned_on IS NULL
ORDER BY loans.due_on;

-- Overdue on 2026-09-24: not returned and the due date has passed.
SELECT members.name AS member, books.title, loans.due_on
FROM loans
JOIN members ON members.id = loans.member_id
JOIN books   ON books.id   = loans.book_id
WHERE loans.returned_on IS NULL
  AND loans.due_on < DATE '2026-09-24'
ORDER BY loans.due_on;

-- How many times has each book been borrowed?
SELECT books.title, count(loans.id) AS times_borrowed
FROM books
LEFT JOIN loans ON loans.book_id = books.id
GROUP BY books.title
ORDER BY times_borrowed DESC, books.title;
```

`04-borrow-and-return.sql`
```sql
-- Grace borrows "A Wizard of Earthsea" (book 4): both changes or neither.
BEGIN;
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (4, 3, '2026-09-24', '2026-10-08');
UPDATE books SET copies_available = copies_available - 1 WHERE id = 4;
COMMIT;

-- Ada returns "Emma" (loan 1): both changes or neither.
BEGIN;
UPDATE loans SET returned_on = '2026-09-24' WHERE id = 1;
UPDATE books SET copies_available = copies_available + 1 WHERE id = 1;
COMMIT;

SELECT id, title, copies_available FROM books WHERE id IN (1, 4) ORDER BY id;
```

`05-the-rules-hold.sql`
```sql
-- Borrow "Persuasion" twice: there is only one copy.
BEGIN;
UPDATE books SET copies_available = copies_available - 1 WHERE id = 2;
UPDATE books SET copies_available = copies_available - 1 WHERE id = 2;
COMMIT;

-- A second member with Ada's email.
INSERT INTO members (name, email, joined_on) VALUES ('Ada Again', 'ada@example.com', '2026-09-24');

-- A loan due before it starts.
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (5, 2, '2026-09-24', '2026-09-01');

-- A loan for a book that does not exist.
INSERT INTO loans (book_id, member_id, loaned_on, due_on) VALUES (99, 2, '2026-09-24', '2026-10-08');

SELECT title, copies_available FROM books WHERE id = 2;
```

**Page** (project format: What you'll build → What you need to know → The spec → Build it, step by step → Reference solution, line by line → Stretch goals → Remember this). This is the **fully guided** level of the confidence ladder.
- **What you'll build:** the final `03-queries.out` as the demo output, plus one sentence on what a real library app does with it.
- **What you need to know:** links to all 7 A1 lessons.
- **The spec:**
  - requirements in plain English: authors, books with total/available copies, members with unique emails, and loans with loaned/due/returned dates;
  - the rules: available between 0 and total; due after loaned; returned not before loaned; a loan must point at a real book and member;
  - the four questions the app must answer;
  - borrowing and returning must be all-or-nothing;
  - acceptance checks: "run `05-the-rules-hold.sql` and see 4 refusals and Persuasion still at 1".
  - Draw the tables as a ` ```text ` ER diagram.
- **Build it, step by step:** 5 steps matching the 5 files. Each step has a goal, a hint in `<details>`, and a ✅ checkpoint: the reader's command and the included `.out`.
- **Reference solution, line by line:** include each `.sql`, with Line by line entries for the parts that are new in combination (`BETWEEN 0 AND copies_total`, the three-table JOIN, `LEFT JOIN … GROUP BY` counting zeros, and why borrow and return are transactions).
- **Stretch goals** (each with a solution file `90`–`92` run by CI):
  - a `max 3 open loans per member` rule, shown as a query that finds members over the limit;
  - the member who borrowed most;
  - books never borrowed.
- **Remember this:** 4 bullets.

### Task 11: Cheat sheet `cheatsheet-sql` + phase check

**Files:**
- Create: `code/sql/cheatsheet-sql/01-cheatsheet.sql`, `src/a1-postgres/cheatsheet-sql.md`
- Modify: `tools/topics.data.js`, `src/part-0-start/tour-of-the-stack.md` (only via generate: its Next now links what-is-a-database)

- [ ] **Step 1: Write one self-contained SQL file.** Make `01-cheatsheet.sql` rerunnable, with `-- ANCHOR: <name>` / `-- ANCHOR_END: <name>` around each pattern:
  - `create-table`
  - `insert`
  - `select-filter-sort`
  - `update`
  - `delete`
  - `primary-foreign-keys`
  - `join`
  - `left-join-count`
  - `index`
  - `explain`
  - `transaction`
  - `null-checks`

  Use a tiny `shop_items`/`shop_orders` schema, so every snippet runs in order. Generate its `.out` (it is checked by CI but not shown).
- [ ] **Step 2: Write the page.** Topic entry: `kind: "cheatsheet"`, `status: "published"`, `prereq: ["sql-transactions"]`, `next: ["hello-axum"]`. Set summary to "Every SQL pattern from Part A1 on one page, each linked to the lesson that explains it." The page has one `## How do I…?` section per pattern. Each is a short task-phrased heading ("…find rows that match?"), then a ` ```sql ` include by anchor (`{{#include ../../code/sql/cheatsheet-sql/01-cheatsheet.sql:join}}`), then "Explained in: [Keys and relations](keys-and-relations.md)". Add a short psql-commands table (`\dt`, `\d`, `\di`, `\q`, `\?`) in prose/table form (no fence).
- [ ] **Step 3: Run the phase check.**

```bash
npm test && node tools/generate.mjs && git status --short   # generate must leave nothing new after commits
node tools/validate.mjs                                      # 0 error(s), 0 warning(s); 13 published pages
mdbook build 2>&1 | grep -E ' (ERROR|WARN) ' ; echo "grep-exit=$?"   # grep-exit=1 (none)
node tools/sql-check.mjs --check-twice                       # all SQL outputs match
(cd code && cargo test --workspace --all-targets)            # unchanged, green
```

- [ ] **Step 4: Tear down the scratch database** (`docker compose -p rbh-sql -f /private/tmp/rbh-sql/docker-compose.yml down -v`) and commit (`docs(a1): SQL cheat sheet`).
