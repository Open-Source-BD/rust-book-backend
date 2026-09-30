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
  const files = { "/r/code/x/src/main.rs": "", "/r/src/a2-axum/x.md": "# X\n" }; // the linked lesson exists
  const r = checkPage({ ...ctx(md, files), topic: t });
  assert.equal(r.errors.length, 1, r.errors.join("\n"));
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

test("stripFences handles a nested inner fence inside an outer 4-backtick fence", () => {
  const md = "````md\n```rust\nfn a(){}\n```\n## Your turn\n````\nafter\n";
  const out = stripFences(md);
  assert.equal(out.split("\n").length, md.split("\n").length);
  assert.ok(!out.includes("## Your turn"));
  assert.ok(!out.includes("fn a(){}"));
  assert.ok(out.includes("after"));
});

test("a ~~~ fence is not closed by a ``` line", () => {
  const md = "~~~\n```\n## should stay hidden\n~~~\nafter\n";
  const out = stripFences(md);
  assert.equal(out.split("\n").length, md.split("\n").length);
  assert.ok(!out.includes("## should stay hidden"));
  assert.ok(out.includes("after"));
});

test("checkPage: heading whose only occurrence is inside a nested fence still counts as missing", () => {
  const nested = "\n````md\n```rust\nfn a(){}\n```\n## Common mistakes\n````\n";
  const md = good().replace("\n## Common mistakes\n", nested);
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes('missing section "## Common mistakes"')), r.errors.join("\n"));
});

// ---------------------------------------------------------------------------
// Final fix wave (I1-I5 + minors)

import { regenerateNext, checkLinks } from "../lib.mjs";

const linkCtx = (md, extra = {}) => {
  const files = {
    "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n",
    "/r/src/part-0-start/other.md": "# Other\n",
    "/r/src/a1-postgres/draft.md": "# Draft\n",
    "/r/src/introduction.md": "# Introduction\n",
    ...(extra.files || {}),
  };
  return {
    ...ctx(md, files),
    publishedPaths: new Set(["/r/src/part-0-start/other.md", "/r/src/introduction.md"]),
    publishedTitles: new Set(["Other"]),
    ...extra,
    files: undefined,
  };
};

test("I1: link to a missing .md file is an error", () => {
  const r = checkPage(linkCtx(good() + "\nSee [gone](gone.md).\n"));
  assert.ok(r.errors.some((e) => e.includes("gone.md") && e.includes("not found")), r.errors.join("\n"));
});

test("I1: link to a draft page is an error", () => {
  const r = checkPage(linkCtx(good() + "\nSee [d](../a1-postgres/draft.md).\n"));
  assert.ok(r.errors.some((e) => e.includes("draft.md") && e.includes("not published")), r.errors.join("\n"));
});

test("I1: valid links (plain, #fragment, external, pure #frag, in code) pass", () => {
  const md = good() + "\nSee [o](other.md), [o2](other.md#some-part), [i](../introduction.md#x), " +
    "[w](https://example.com/x.md), [m](mailto:a@b.c), [f](#local).\n\n`[c](nope.md)`\n\n```md\n[c](nope2.md)\n```\n";
  const r = checkPage(linkCtx(md));
  assert.deepEqual(r.errors, []);
});

test("I1: link with #fragment to a missing file is still an error", () => {
  const r = checkPage(linkCtx(good() + "\nSee [g](gone.md#part).\n"));
  assert.ok(r.errors.some((e) => e.includes("gone.md")), r.errors.join("\n"));
});

test("I1: checkLinks is usable on non-topic pages", () => {
  const errs = checkLinks({ md: "[a](part-0-start/other.md) [b](a1-postgres/draft.md)", mdFile: "/r/src/introduction.md",
    exists: (p) => p === "/r/src/part-0-start/other.md" || p === "/r/src/a1-postgres/draft.md",
    publishedPaths: new Set(["/r/src/part-0-start/other.md"]), publishedTitles: new Set() });
  assert.equal(errs.length, 1);
  assert.ok(errs[0].includes("draft.md"));
});

test("I2: stub wraps the Next block in markers", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const md = stubFor(a, map([a, b]));
  assert.match(md, /<!-- next:start -->\n\n\*\*Next:\*\*\n\n- B \(coming soon\)\n\n<!-- next:end -->/);
});

test("I2: regenerateNext reflects a newly published topic and leaves the rest byte-identical", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const bDraft = pub({ slug: "b", title: "B", status: "draft", part: "A1" });
  const before = "# A\r\nhuman text ✅\n\n## Go deeper\n\n- [RFH](https://x)\n\n<!-- next:start -->\n\n**Next:**\n\n- B (coming soon)\n\n<!-- next:end -->\n\ntrailing human text\n";
  assert.equal(regenerateNext(before, a, map([a, bDraft])), before, "idempotent when nothing changed");
  const bPub = { ...bDraft, status: "published" };
  const after = regenerateNext(before, a, map([a, bPub]));
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
  assert.ok(!after.includes("coming soon"));
  const [pre, post] = [before.split("<!-- next:start -->"), before.split("<!-- next:end -->")];
  assert.ok(after.startsWith(pre[0] + "<!-- next:start -->"));
  assert.ok(after.endsWith("<!-- next:end -->" + post[1]));
});

test("I2: regenerateNext leaves a page without markers untouched", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const md = "# A\n\n## Go deeper\n\n**Next:**\n\n- B (coming soon)\n";
  assert.equal(regenerateNext(md, a, map([a, b])), md);
});

test("I2: validator catches a stale (coming soon) naming a published topic", () => {
  const md = good() + "\n- Other (coming soon)\n- Still Draft (coming soon)\n";
  const r = checkPage(linkCtx(md));
  const stale = r.errors.filter((e) => e.includes("coming soon"));
  assert.equal(stale.length, 1, r.errors.join("\n"));
  assert.ok(stale[0].includes("Other"));
});

test("I3: rustdoc_include and playground are validated like include", () => {
  const md = good() + "\n```rust,noplayground\n{{#rustdoc_include ../../code/x/src/main.rs:nope}}\n```\n\n" +
    "```rust,editable\n{{#playground ../../code/x/src/missing.rs editable}}\n```\n\n" +
    "```rust,noplayground\n{{#rustdoc_include ../../code/x/src/main.rs:app}}\n```\n";
  const r = checkPage(ctx(md));
  assert.ok(r.errors.some((e) => e.includes('anchor "nope"')), r.errors.join("\n"));
  assert.ok(r.errors.some((e) => e.includes("not found: ../../code/x/src/missing.rs")), r.errors.join("\n"));
  assert.equal(r.errors.length, 2, r.errors.join("\n"));
});

test("I3: playground with an anchor is an error", () => {
  const md = good() + "\n```rust,editable\n{{#playground ../../code/x/src/main.rs:app}}\n```\n";
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("playground")));
});

test("I4: anchor renamed — include asks for app, file only has app-state", () => {
  const r = checkPage(ctx(good(), { "/r/code/x/src/main.rs": "// ANCHOR: app-state\nfn a(){}\n// ANCHOR_END: app-state\n" }));
  assert.ok(r.errors.some((e) => e.includes('anchor "app"')), r.errors.join("\n"));
});

test("I5: a hand-typed rust fence is an error; editable, ignore and include fences pass", () => {
  const bad = good() + "\n```rust,noplayground\nfn typed() {}\n```\n\n```rust\nfn typed2() {}\n```\n\n  ```rust,noplayground\n  fn indented() {}\n  ```\n";
  const r = checkPage(ctx(bad));
  assert.equal(r.errors.filter((e) => e.includes("hand-typed Rust")).length, 3, r.errors.join("\n"));
  const ok = good() + "\n```rust,editable\nfn main() {}\n```\n\n```rust,noplayground,ignore\nfn broken( {}\n```\n\n```text\nfn not_rust() {}\n```\n";
  assert.deepEqual(checkPage(ctx(ok)).errors, []);
});

test("minor: CRLF page passes like its LF twin", () => {
  assert.deepEqual(checkPage(ctx(good().replace(/\n/g, "\r\n"))).errors, []);
});

test("minor: a heading with a trailing space still counts", () => {
  const md = good().replace("## Your turn\n", "## Your turn \n").replace("### Line by line\n", "### Line by line  \n");
  assert.deepEqual(checkPage(ctx(md)).errors, []);
});

test("minor: file::N is a line range — warn, don't look up an empty anchor", () => {
  const r = checkPage(ctx(good().replace(":app}}", "::3}}")));
  assert.ok(r.warnings.some((w) => w.includes("use an ANCHOR")), r.warnings.join("\n"));
  assert.ok(!r.errors.some((e) => e.includes("anchor")), r.errors.join("\n"));
});

test("minor: each Your turn tier needs a <details> solution", () => {
  const i = good().indexOf("### 🟡 Tweak");
  const j = good().indexOf("### 🔴 From scratch");
  const md = good().slice(0, i) + "### 🟡 Tweak\n\nno solution here\n\n" + good().slice(j);
  const r = checkPage(ctx(md));
  const e = r.errors.filter((x) => x.includes("<details>"));
  assert.equal(e.length, 1, r.errors.join("\n"));
  assert.ok(e[0].includes("🟡 Tweak"));
});

test("minor: checkTopics catches unknown next/prereq slugs", () => {
  const errs = checkTopics([pub({ slug: "a", title: "A", next: ["ghost"], prereq: ["b"] }), pub({ slug: "b", title: "B", prereq: ["phantom"] })]);
  assert.ok(errs.some((e) => e.includes("a: next") && e.includes("ghost")), errs.join("\n"));
  assert.ok(errs.some((e) => e.includes("b: prereq") && e.includes("phantom")), errs.join("\n"));
  assert.equal(errs.length, 2, errs.join("\n"));
});

test("I2: a lesson without Next markers warns", () => {
  const md = good().replace(/<!-- next:(start|end) -->/g, "");
  assert.ok(checkPage(ctx(md)).warnings.some((w) => w.includes("next:start")));
  assert.ok(!checkPage(ctx(good())).warnings.some((w) => w.includes("next:start")));
});

// ---------------------------------------------------------------------------
// Phase 1 carry-over fixes

import { fencedBlocks, checkComingSoon } from "../lib.mjs";

test("carry: a fence indented 4+ spaces inside a list item hides its headings", () => {
  const md = good().replace("\n## Common mistakes\n", "\n- item:\n\n      ```md\n      ## Common mistakes\n      ```\n");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes('"## Common mistakes"')));
});

test("carry: a fence inside a blockquote is a fence", () => {
  const md = "> ```rust\n> fn main() {}\n> ```\n";
  const blocks = fencedBlocks(md);
  assert.equal(blocks.length, 1);
  assert.equal(blocks[0].info, "rust");
});

test("carry: hand-typed rust inside an indented list fence is caught", () => {
  const md = good().replace("## Common mistakes\n\ntext", "## Common mistakes\n\n1. step:\n\n    ```rust,noplayground\n    fn main() {}\n    ```");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("hand-typed Rust")));
});

test("carry: reference-style and <a href> links to missing pages are errors", () => {
  const md = "See [the toolbox][tb] and <a href=\"../part-0-start/nope.md\">this</a>.\n\n[tb]: ../part-0-start/your-tolbox.md\n";
  const errs = checkLinks({ md, mdFile: "/r/src/a1-postgres/x.md", exists: () => false, publishedPaths: new Set() });
  assert.equal(errs.length, 2);
});

test("carry: '* ' and '+ ' bullets count for stale coming-soon", () => {
  const titles = new Set(["Indexes"]);
  assert.equal(checkComingSoon("* Indexes (coming soon)\n", titles).length, 1);
  assert.equal(checkComingSoon("+ Indexes (coming soon)\n", titles).length, 1);
});

test("carry: a rust fence mixing an include with hand-typed lines is an error", () => {
  const md = good().replace("{{#include ../../code/x/src/main.rs:app}}", "{{#include ../../code/x/src/main.rs:app}}\nfn extra() {}");
  assert.ok(checkPage(ctx(md)).errors.some((e) => e.includes("mixes")));
});

test("carry: a rust fence holding only {{#playground}} counts as included", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/x/examples/p.rs": "fn main(){}\n" };
  const md = good().replace("### Line by line", "```rust\n{{#playground ../../code/x/examples/p.rs}}\n```\n\n### Line by line");
  assert.ok(!checkPage(ctx(md, files)).errors.some((e) => e.includes("hand-typed")));
});

test("carry: regenerateNext keeps CRLF line endings in a CRLF page", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const before = "# A\r\n\r\n<!-- next:start -->\r\n<!-- next:end -->\r\n";
  const after = regenerateNext(before, a, map([a, b]));
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
  assert.ok(!/[^\r]\n/.test(after), "every \\n is preceded by \\r");
});

test("carry: marker text inside a code fence is not treated as the Next block", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const before = "```md\n<!-- next:start -->\nexample\n<!-- next:end -->\n```\n\n<!-- next:start -->\n<!-- next:end -->\n";
  const after = regenerateNext(before, a, map([a, b]));
  assert.ok(after.startsWith("```md\n<!-- next:start -->\nexample\n<!-- next:end -->\n```\n"), "fenced example untouched");
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
});

// ---------------------------------------------------------------------------
// Fix round 1 (review findings on Task 1)

test("carry: marker text inside a CRLF code fence is not treated as the Next block", () => {
  const a = pub({ slug: "a", title: "A", next: ["b"] });
  const b = pub({ slug: "b", title: "B", part: "A1" });
  const before = "```md\r\n<!-- next:start -->\r\nexample\r\n<!-- next:end -->\r\n```\r\n\r\n<!-- next:start -->\r\n<!-- next:end -->\r\n";
  const after = regenerateNext(before, a, map([a, b]));
  assert.ok(after.startsWith("```md\r\n<!-- next:start -->\r\nexample\r\n<!-- next:end -->\r\n```\r\n"), "fenced example byte-identical");
  assert.ok(after.includes("- [B](../a1-postgres/b.md)"));
});

test("carry: stripFences is CRLF-safe — a \\r\\n-terminated fence still blanks its content", () => {
  const md = "```md\r\nx\r\n```\r\n";
  const out = stripFences(md);
  assert.equal(out.split("\n").length, md.split("\n").length);
  assert.ok(!out.includes("x"));
});

test("carry: a footnote definition is not a page link; a normal reference definition still is", () => {
  const footnote = checkLinks({ md: "[^1]: ../part-0-start/nope.md\n", mdFile: "/r/src/a1-postgres/x.md", exists: () => false, publishedPaths: new Set() });
  assert.equal(footnote.length, 0, footnote.join("\n"));
  const normal = checkLinks({ md: "[id]: missing.md\n", mdFile: "/r/src/a1-postgres/x.md", exists: () => false, publishedPaths: new Set() });
  assert.equal(normal.length, 1, normal.join("\n"));
});

test("sql: a hand-typed sql fence is an error; include and ignore pass", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/sql/hello/01-a.sql": "SELECT 1;\n" };
  const withFence = (f) => good().replace("### Line by line", `${f}\n\n### Line by line`);
  const bad = checkPage(ctx(withFence("```sql\nSELECT 1;\n```"), files));
  assert.ok(bad.errors.some((e) => e.includes("hand-typed SQL")));
  const inc = checkPage(ctx(withFence("```sql\n{{#include ../../code/sql/hello/01-a.sql}}\n```"), files));
  assert.ok(!inc.errors.some((e) => e.includes("hand-typed")));
  const ign = checkPage(ctx(withFence("```sql,ignore\nSELEC 1;\n```"), files));
  assert.ok(!ign.errors.some((e) => e.includes("hand-typed")));
});

test("sql: a sql fence's include must live under code/sql/ (the only folder sql-check runs)", () => {
  const files = {
    "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n",
    "/r/code/sql/hello/01-a.sql": "SELECT 1;\n",
    "/r/code/topics/x/m.sql": "SELECT 1;\n",
  };
  const withFence = (f) => good().replace("### Line by line", `${f}\n\n### Line by line`);
  const outside = checkPage(ctx(withFence("```sql\n{{#include ../../code/topics/x/m.sql}}\n```"), files));
  assert.ok(outside.errors.some((e) => e.includes("under code/sql/")), outside.errors.join("\n"));
  const inside = checkPage(ctx(withFence("```sql\n{{#include ../../code/sql/hello/01-a.sql}}\n```"), files));
  assert.ok(!inside.errors.some((e) => e.includes("under code/sql/")), inside.errors.join("\n"));
  assert.equal(inside.errors.length, 0, inside.errors.join("\n"));
});

test("carry: sql include root check uses path segments, not '/' strings", () => {
  const files = { "/r/code/x/src/main.rs": "// ANCHOR: app\nfn a(){}\n// ANCHOR_END: app\n", "/r/code/sqlx/hello/01-a.sql": "SELECT 1;\n" };
  const md = good().replace("### Line by line", "```sql\n{{#include ../../code/sqlx/hello/01-a.sql}}\n```\n\n### Line by line");
  assert.ok(checkPage(ctx(md, files)).errors.some((e) => e.includes("code/sql/")), "code/sqlx must not pass as code/sql");
});
