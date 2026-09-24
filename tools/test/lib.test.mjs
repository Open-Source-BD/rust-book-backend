import { test } from "node:test";
import assert from "node:assert/strict";
import { PARTS, HEADINGS, TIERS, pagePath, stubFor, summaryFor, bundleFor, questionStub } from "../lib.mjs";

const pub = (o) => ({ kind: "lesson", level: "Beginner", status: "published", part: "0",
  outcomes: ["a", "b"], summary: "s", prereq: [], next: [], rfhLinks: [], links: [], ...o });
const map = (ts) => new Map(ts.map((t) => [t.slug, t]));

test("pagePath uses the part directory", () => {
  assert.equal(pagePath(pub({ slug: "x", part: "A2" })), "a2-axum/x.md");
  assert.equal(PARTS.map((p) => p.id).join(","), "0,A1,A2,A3,A4,B");
});

test("lesson stub has every lesson heading in order and the tiers", () => {
  const t = pub({ slug: "hello", title: "Hello" });
  const md = stubFor(t, map([t]));
  let at = -1;
  for (const h of HEADINGS.lesson) {
    const i = md.indexOf(`\n${h}\n`);
    assert.ok(i > at, `heading ${h} missing or out of order`);
    at = i;
  }
  for (const tier of TIERS) assert.ok(md.includes(tier), tier);
  assert.ok(md.includes('<div class="quiz" data-topic="hello"></div>'));
  assert.ok(md.includes("AUTHORING:"));
});

test("next link to a draft renders as coming-soon text, not a link", () => {
  const a = pub({ slug: "a", title: "A", next: ["b", "c"] });
  const b = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const c = pub({ slug: "c", title: "C", part: "A1" });
  const md = stubFor(a, map([a, b, c]));
  assert.ok(md.includes("- B (coming soon)"));
  assert.ok(!md.includes("](../a1-postgres/b.md)"));
  assert.ok(md.includes("- [C](../a1-postgres/c.md)"));
});

test("project stub uses project headings", () => {
  const t = pub({ slug: "p", title: "P", kind: "project" });
  const md = stubFor(t, map([t]));
  for (const h of HEADINGS.project) assert.ok(md.includes(`\n${h}\n`), h);
  assert.ok(!md.includes("## Quick check"));
});

test("summary links published pages and shows drafts as draft chapters", () => {
  const a = pub({ slug: "a", title: "A" });
  const b = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const s = summaryFor([a, b]);
  assert.ok(s.startsWith("# Summary\n\n[Introduction](introduction.md)\n[Glossary](glossary.md)\n"));
  assert.ok(s.includes("# Part 0 · Before you start\n\n- [A](part-0-start/a.md)\n"));
  assert.ok(s.includes("# Part A1 · PostgreSQL & SQL\n\n- [B]()\n"));
  assert.ok(s.trimEnd().endsWith("[Review & flashcards](review.md)"));
});

test("bundle includes only published lessons", () => {
  const a = pub({ slug: "a", title: "A" });
  const p = pub({ slug: "p", title: "P", kind: "project" });
  const d = pub({ slug: "d", title: "D", status: "draft" });
  const out = bundleFor([a, p, d], { a: { quiz: [1] } });
  assert.ok(out.startsWith("// AUTO-GENERATED"));
  assert.ok(out.includes('window.RUST_QUESTIONS = {"a":{"quiz":[1]}};'));
  assert.ok(out.includes('window.RUST_TOPIC_ORDER = [{"slug":"a","title":"A","category":"Part 0 · Before you start"}];'));
});

test("questionStub is valid JSON with empty banks", () => {
  const q = JSON.parse(questionStub(pub({ slug: "a", title: "A" })));
  assert.deepEqual(q, { topic: "a", title: "A", quiz: [], flashcards: [] });
});

import { stripFences, headingId, glossaryIds, checkPage, checkBank, checkTopics } from "../lib.mjs";

const T = pub({ slug: "hello", title: "Hello" });
const good = () => stubFor(T, map([T]))
  .replace(/<!-- AUTHORING:[^>]*-->/g, "text")
  .replace("## The idea, slowly\n\ntext", "## The idea, slowly\n\n```rust\n{{#include ../../code/x/src/main.rs:app}}\n```\n\n### Line by line\n\nok");
const ctx = (md, files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n" }) => ({
  topic: T, md, mdFile: "/r/src/part-0-start/hello.md", root: "/r",
  glossary: new Set(["handler"]),
  exists: (p) => p in files || p === "/r/code/x",
  read: (p) => files[p],
});

test("a complete lesson passes", () => {
  const r = checkPage(ctx(good()));
  assert.deepEqual(r.errors, []);
});

test("missing anchor is an error", () => {
  const r = checkPage(ctx(good(), { "/r/code/x/src/main.rs": "fn a(){}\n" }));
  assert.ok(r.errors.some((e) => e.includes('anchor "app"')), r.errors.join("\n"));
});

test("missing include file is an error", () => {
  const r = checkPage(ctx(good(), {}));
  assert.ok(r.errors.some((e) => e.includes("include file not found")));
});

test("line-range include is a warning", () => {
  const md = good().replace(":app}}", ":1:3}}");
  assert.ok(checkPage(ctx(md)).warnings.some((w) => w.includes("use an ANCHOR")));
});

test("headings inside code fences do not count", () => {
  const md = good().replace("\n## Common mistakes\n", "\n```md\n## Common mistakes\n```\n");
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes('"## Common mistakes"')), r.errors.join("\n"));
});

test("out-of-order headings are an error", () => {
  const md = good().replace("## What & why", "## TEMP").replace("## Common mistakes", "## What & why").replace("## TEMP", "## Common mistakes");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("order")));
});

test("missing tier and missing Line by line are errors", () => {
  const md = good().replace("### 🟡 Tweak", "### Tweak").replace("### Line by line", "### Notes");
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes("🟡 Tweak")));
  assert.ok(r.errors.some((e) => e.includes("Line by line")));
});

test("AUTHORING placeholders are errors; banned words in prose warn, not in code", () => {
  const md = good() + "\n<!-- AUTHORING: x -->\nYou simply run it.\n```bash\n# just a comment\n```\n`just`\n";
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes("AUTHORING")));
  assert.equal(r.warnings.filter((w) => w.includes("banned word")).length, 1);
  assert.ok(r.warnings.some((w) => w.includes('"simply"')));
});

test("unknown glossary anchor warns", () => {
  const md = good() + "\nA [router](../glossary.md#router) and a [handler](../glossary.md#handler).\n";
  const w = checkPage(ctx(md)).warnings.filter((x) => x.includes("glossary"));
  assert.equal(w.length, 1);
  assert.ok(w[0].includes("#router"));
});

test("checklist items must link", () => {
  const t = pub({ slug: "ready", title: "R", kind: "checklist" });
  const md = "# R\n\n- [ ] I can add a route ([lesson](../a2-axum/x.md))\n- [ ] I can write a migration\n";
  const r = checkPage({ ...ctx(md), topic: t });
  assert.equal(r.errors.length, 1);
  assert.ok(r.errors[0].includes("I can write a migration"));
});

test("missing codeDir is an error", () => {
  const r = checkPage({ ...ctx(good()), topic: { ...T, codeDir: "code/nope" } });
  assert.ok(r.errors.some((e) => e.includes("codeDir")));
});

test("headingId matches mdBook", () => {
  assert.equal(headingId("Status code"), "status-code");
  assert.equal(headingId("ORM (Object-Relational Mapper)"), "orm-object-relational-mapper");
  assert.deepEqual([...glossaryIds("# Glossary\n\n### Handler\n\n### Status code\n")], ["handler", "status-code"]);
});

test("stripFences keeps line count", () => {
  const md = "a\n```\nb\nc\n```\nd";
  assert.equal(stripFences(md).split("\n").length, md.split("\n").length);
  assert.ok(!stripFences(md).includes("b"));
});

test("checkBank: bad answer index errors, <4 questions warns", () => {
  const r = checkBank(T, { quiz: [{ q: "q", options: ["a", "b"], answer: 2, explain: "e" }], flashcards: [{ front: "f", back: "b" }] });
  assert.ok(r.errors.some((e) => e.includes("answer index")));
  assert.ok(r.warnings.some((w) => w.includes("fewer than 4")));
});

test("checkTopics catches duplicate slugs and bad parts", () => {
  const errs = checkTopics([pub({ slug: "a", title: "A" }), pub({ slug: "a", title: "A2" }), pub({ slug: "z", title: "Z", part: "X" })]);
  assert.ok(errs.some((e) => e.includes("duplicate slug a")));
  assert.ok(errs.some((e) => e.includes("unknown part X")));
});
