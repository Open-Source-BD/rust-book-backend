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
