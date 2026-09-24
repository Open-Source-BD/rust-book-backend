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
