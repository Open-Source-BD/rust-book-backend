// validate.mjs — checks every published page and question bank.
// Run: node tools/validate.mjs   (exits 1 on any ERROR; warnings don't fail)
import fs from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import topics from "./topics.data.js";
import { pagePath, checkPage, checkBank, checkTopics, glossaryIds, quizzable } from "./lib.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
let errors = 0, warns = 0;
const report = (who, r) => {
  r.errors.forEach((e) => { console.log(`ERROR  ${who}: ${e}`); errors++; });
  r.warnings.forEach((w) => { console.log(`warn   ${who}: ${w}`); warns++; });
};

report("topics.data.js", { errors: checkTopics(topics), warnings: [] });

const glossaryFile = join(ROOT, "src", "glossary.md");
const glossary = fs.existsSync(glossaryFile) ? glossaryIds(fs.readFileSync(glossaryFile, "utf8")) : new Set();
if (!fs.existsSync(glossaryFile)) report("glossary.md", { errors: ["src/glossary.md missing"], warnings: [] });

const io = { exists: (p) => fs.existsSync(p), read: (p) => fs.readFileSync(p, "utf8") };
const published = topics.filter((t) => t.status === "published");
for (const t of published) {
  const mdFile = join(ROOT, "src", pagePath(t));
  if (!fs.existsSync(mdFile)) { report(t.slug, { errors: [`page missing: src/${pagePath(t)}`], warnings: [] }); continue; }
  report(t.slug, checkPage({ topic: t, md: fs.readFileSync(mdFile, "utf8"), mdFile, root: ROOT, glossary, ...io }));
  if (!quizzable(t)) continue;
  const qFile = join(ROOT, "questions", `${t.slug}.json`);
  try { report(t.slug, checkBank(t, JSON.parse(fs.readFileSync(qFile, "utf8")))); }
  catch (e) { report(t.slug, { errors: [`questions JSON missing or invalid — ${e.message}`], warnings: [] }); }
}

console.log(`\n${published.length} published pages checked (${topics.length - published.length} drafts skipped) — ${errors} error(s), ${warns} warning(s)`);
process.exit(errors ? 1 : 0);
