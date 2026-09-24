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
