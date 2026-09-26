// generate.mjs — writes page stubs, questions/<slug>.json stubs, src/SUMMARY.md and
// theme/questions.data.js from tools/topics.data.js.
// Also rewrites the Next block between <!-- next:start --> / <!-- next:end --> in every existing
// published page, so "(coming soon)" turns into a link as soon as the target is published.
// Run:  node tools/generate.mjs          (safe: never overwrites existing pages outside the markers)
//       node tools/generate.mjs --force  (overwrite page stubs)
import fs from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import topics from "./topics.data.js";
import { pagePath, stubFor, summaryFor, bundleFor, questionStub, quizzable, checkTopics, regenerateNext } from "./lib.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const FORCE = process.argv.includes("--force");
const topicErrors = checkTopics(topics);
if (topicErrors.length) {
  topicErrors.forEach((e) => console.log(`ERROR  topics.data.js: ${e}`));
  process.exit(1);
}
const bySlug = new Map(topics.map((t) => [t.slug, t]));
let wrote = 0, kept = 0, nexts = 0;

for (const t of topics.filter((t) => t.status === "published")) {
  const file = join(ROOT, "src", pagePath(t));
  fs.mkdirSync(dirname(file), { recursive: true });
  if (FORCE || !fs.existsSync(file)) { fs.writeFileSync(file, stubFor(t, bySlug)); wrote++; }
  else {
    kept++;
    const md = fs.readFileSync(file, "utf8");
    const out = regenerateNext(md, t, bySlug);
    if (out !== md) { fs.writeFileSync(file, out); nexts++; }
  }
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

console.log(`pages: ${wrote} written, ${kept} kept (${nexts} Next block(s) updated) | SUMMARY.md + questions.data.js regenerated`);
