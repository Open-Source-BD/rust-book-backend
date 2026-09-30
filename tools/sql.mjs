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

export function planRun(entries, ext = ".sql") {
  const sql = entries.filter((e) => e.endsWith(ext)).sort();
  for (const f of sql) if (!/^\d{2}-/.test(f)) throw new Error(`${f}: listing files must start with a two-digit NN- prefix`);
  const bases = new Set(sql.map((f) => f.slice(0, -ext.length)));
  const orphans = entries.filter((e) => e.endsWith(".out") && !bases.has(e.slice(0, -4))).sort();
  return { sql, orphans };
}

// The DROP/CREATE setup script run once per lesson, before any of its NN-*.sql files. It — and
// only it — sets ON_ERROR_STOP: without it psql exits 0 even when a statement fails (e.g. the
// target database can't be dropped), so a broken setup would silently run the lesson's files
// against a stale database and "pass". Lesson files themselves must NOT get ON_ERROR_STOP: a
// Common-mistakes file's whole point is to run to completion and capture its own real error.
export function setupScript(db) {
  return `\\set ON_ERROR_STOP 1\nDROP DATABASE IF EXISTS ${db} WITH (FORCE);\nCREATE DATABASE ${db};\n`;
}
