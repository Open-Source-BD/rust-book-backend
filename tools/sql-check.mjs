// sql-check.mjs — runs every code/sql/<slug>/NN-*.sql against a fresh database named after the
// slug and compares psql's real output with the committed NN-*.out.
// Run:  node tools/sql-check.mjs                 (check all)
//       node tools/sql-check.mjs --update crud-in-sql   (rewrite .out for one lesson)
//       node tools/sql-check.mjs --check-twice   (also re-run each lesson's files a second time,
//                                                  in the SAME database with no reset in between:
//                                                  proves both determinism and that 01 resets its
//                                                  own tables, so a reader can rerun the lesson)
// RBH_PSQL: shell prefix that runs psql; the database name is appended and SQL comes on stdin.
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { dbNameFor, normalizeOutput, diffLines, planRun, setupScript } from "./sql.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SQL_DIR = join(ROOT, "code", "sql");
const PSQL = process.env.RBH_PSQL || `docker compose exec -T db sh -c 'psql -X -U postgres -d "$0" 2>&1'`;
const args = process.argv.slice(2);
const UPDATE = args.includes("--update");
const TWICE = args.includes("--check-twice");
const only = args.filter((a) => !a.startsWith("--"));

// `step`, when given, names what this particular call is doing (e.g. "setting up database X for
// lesson Y (DROP/CREATE)") so a setup failure is unmistakable — never confused with a lesson
// file's own (possibly intentional) error — and the message shows psql's real error text instead
// of only a generic connectivity hint.
function psql(db, input, step) {
  const r = spawnSync("sh", ["-c", `${PSQL} ${db}`], { input, encoding: "utf8", cwd: ROOT });
  if (r.status !== 0) {
    console.error(`psql failed (exit ${r.status}) ${step ? `while ${step}` : `for database ${db}`}:\n${r.stdout}${r.stderr}`);
    console.error(step
      ? "Setup must succeed before any SQL file runs — fix the error above (or the target database) and re-run."
      : "Is Postgres running? (docker compose up -d --wait) — or set RBH_PSQL.");
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
  // ON_ERROR_STOP (inside setupScript) makes a failed DROP/CREATE fatal instead of silently
  // leaving the previous database in place. Lesson files run WITHOUT it: see psql()'s doc comment.
  psql("postgres", setupScript(db), `setting up database ${db} for lesson ${slug} (DROP/CREATE)`);
  const runFiles = () => sql.map((f) => normalizeOutput(psql(db, fs.readFileSync(join(dir, f), "utf8"))));
  const outs = runFiles();
  if (TWICE) {
    // Re-run every file from 01, in the SAME database, with no DROP/CREATE in between — proves
    // determinism AND that 01 actually resets its own tables (a reader reruns the lesson this way
    // too, not against a fresh database).
    const again = runFiles();
    sql.forEach((f, i) => {
      if (again[i] !== outs[i]) {
        console.log(`FAIL  ${slug}/${f}: output differs on a second run in the same database (nondeterministic SQL, or 01 doesn't reset its own tables)`);
        failed++;
      }
    });
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
