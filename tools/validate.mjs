// validate.mjs — checks every published page and question bank.
// Run: node tools/validate.mjs   (exits 1 on any ERROR; warnings don't fail)
import fs from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import topics from "./topics.data.js";
import { pagePath, checkPage, checkBank, checkTopics, checkLinks, checkComingSoon, glossaryIds, quizzable } from "./lib.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
let errors = 0, warns = 0;
const report = (who, r) => {
  r.errors.forEach((e) => { console.log(`ERROR  ${who}: ${e}`); errors++; });
  r.warnings.forEach((w) => { console.log(`warn   ${who}: ${w}`); warns++; });
};

const topicErrors = checkTopics(topics);
report("topics.data.js", { errors: topicErrors, warnings: [] });
if (topicErrors.length) { console.log("\nfix topics.data.js first — pages can't be located until it is valid"); process.exit(1); }

const glossaryFile = join(ROOT, "src", "glossary.md");
const glossary = fs.existsSync(glossaryFile) ? glossaryIds(fs.readFileSync(glossaryFile, "utf8")) : new Set();
if (!fs.existsSync(glossaryFile)) report("glossary.md", { errors: ["src/glossary.md missing"], warnings: [] });

const io = { exists: (p) => fs.existsSync(p), read: (p) => fs.readFileSync(p, "utf8") };
const published = topics.filter((t) => t.status === "published");
// Pages a link may point at: every published topic page plus the fixed front/back matter.
const FIXED = ["introduction.md", "glossary.md", "review.md"];
const publishedPaths = new Set([...FIXED.map((f) => join(ROOT, "src", f)), ...published.map((t) => join(ROOT, "src", pagePath(t)))]);
const publishedTitles = new Set(published.map((t) => t.title));
for (const f of FIXED) {
  const file = join(ROOT, "src", f);
  if (!fs.existsSync(file)) continue;
  const md = fs.readFileSync(file, "utf8");
  report(f, { errors: [...checkLinks({ md, mdFile: file, exists: io.exists, publishedPaths }), ...checkComingSoon(md, publishedTitles)], warnings: [] });
}
for (const t of published) {
  const mdFile = join(ROOT, "src", pagePath(t));
  if (!fs.existsSync(mdFile)) { report(t.slug, { errors: [`page missing: src/${pagePath(t)}`], warnings: [] }); continue; }
  report(t.slug, checkPage({ topic: t, md: fs.readFileSync(mdFile, "utf8"), mdFile, root: ROOT, glossary, publishedPaths, publishedTitles, ...io }));
  if (!quizzable(t)) continue;
  const qFile = join(ROOT, "questions", `${t.slug}.json`);
  try { report(t.slug, checkBank(t, JSON.parse(fs.readFileSync(qFile, "utf8")))); }
  catch (e) { report(t.slug, { errors: [`questions JSON missing or invalid — ${e.message}`], warnings: [] }); }
}

console.log(`\n${published.length} published pages checked (${topics.length - published.length} drafts skipped) — ${errors} error(s), ${warns} warning(s)`);
process.exit(errors ? 1 : 0);
