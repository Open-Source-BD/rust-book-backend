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
