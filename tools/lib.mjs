// lib.mjs — pure functions shared by generate.mjs and validate.mjs (no file I/O here).
import { dirname as _dirname, resolve as _resolve, relative as _relative, isAbsolute as _isAbsolute } from "node:path";

export const PARTS = [
  { id: "0", title: "Part 0 · Before you start", dir: "part-0-start" },
  { id: "A1", title: "Part A1 · PostgreSQL & SQL", dir: "a1-postgres" },
  { id: "A2", title: "Part A2 · Axum", dir: "a2-axum" },
  { id: "A3", title: "Part A3 · SeaORM", dir: "a3-seaorm" },
  { id: "A4", title: "Part A4 · Putting it together", dir: "a4-together" },
  { id: "B", title: "Part B · Capstone: ShopRS", dir: "b-capstone" },
];
const partOf = (t) => {
  const p = PARTS.find((p) => p.id === t.part);
  if (!p) throw new Error(`${t.slug}: unknown part ${t.part}`);
  return p;
};

export const HEADINGS = {
  lesson: [
    "## By the end of this lesson",
    "## What & why",
    "## The idea, slowly",
    "## You might be wondering…",
    "## Coming from another language?",
    "## Common mistakes",
    "## More examples",
    "## Your turn",
    "## Quick check",
    "## Remember this",
    "## Go deeper",
  ],
  project: [
    "## What you'll build",
    "## What you need to know",
    "## The spec",
    "## Build it, step by step",
    "## Reference solution, line by line",
    "## Stretch goals",
    "## Remember this",
  ],
  cheatsheet: [],
  checklist: [],
};

export const TIERS = ["### 🟢 Guided", "### 🟡 Tweak", "### 🔴 From scratch"];

export const pagePath = (t) => `${partOf(t).dir}/${t.slug}.md`;

const bullets = (xs) => (xs || []).map((x) => `- ${x}`).join("\n");
const authoring = (what) => `<!-- AUTHORING: ${what} -->`;

function linkTo(slug, bySlug) {
  const t = bySlug.get(slug);
  if (!t) return null;
  return t.status === "published" ? `- [${t.title}](../${pagePath(t)})` : `- ${t.title} (coming soon)`;
}

// The Next block is generator-owned: it lives between these markers inside "## Go deeper" and
// is rewritten on every `generate.mjs` run, so "(coming soon)" never goes stale. Everything
// outside the markers (the RFH/official links) is human-owned.
export const NEXT_START = "<!-- next:start -->";
export const NEXT_END = "<!-- next:end -->";

function nextBlock(t, bySlug) {
  const next = (t.next || []).map((s) => linkTo(s, bySlug)).filter(Boolean);
  const inner = next.length ? `\n\n**Next:**\n\n${next.join("\n")}\n\n` : "\n";
  return `${NEXT_START}${inner}${NEXT_END}`;
}

// Rewrites only the marked Next block; a page without both markers is returned unchanged.
// The markers are located on lines outside any fence (via stripFences on a CRLF-normalized copy,
// which preserves line count 1:1 with `md`), so example markers inside a ```md fence are never
// mistaken for the real block. Line indices are then mapped back to character offsets in the
// original, UNTOUCHED `md` string (offset arithmetic and CRLF detection both use the raw text),
// so a CRLF page keeps its CRLF line endings in the freshly generated block.
export function regenerateNext(md, t, bySlug) {
  const mdLines = md.split("\n");
  const strippedLines = stripFences(md.replace(/\r\n/g, "\n")).split("\n");
  const offsetOfLine = (idx) => {
    let off = 0;
    for (let k = 0; k < idx; k++) off += mdLines[k].length + 1; // +1 for the split-on "\n"
    return off;
  };

  let start = -1;
  for (let i = 0; i < strippedLines.length && start < 0; i++) {
    const col = strippedLines[i].indexOf(NEXT_START);
    if (col >= 0) start = offsetOfLine(i) + col;
  }
  if (start < 0) return md;

  let end = -1;
  for (let i = 0; i < strippedLines.length && end < 0; i++) {
    const col = strippedLines[i].indexOf(NEXT_END);
    if (col < 0) continue;
    const off = offsetOfLine(i) + col;
    if (off > start) end = off;
  }
  if (end < 0) return md;

  let block = nextBlock(t, bySlug);
  if (md[start + NEXT_START.length] === "\r") block = block.replace(/\n/g, "\r\n");
  return md.slice(0, start) + block + md.slice(end + NEXT_END.length);
}

function goDeeper(t, bySlug) {
  const refs = [...(t.rfhLinks || []), ...(t.links || [])]
    .map((l) => `- [${l.label}](${l.href})${l.note ? ` — ${l.note}` : ""}`);
  const head = refs.length ? refs.join("\n") : authoring("official docs + Rust for Humans links");
  return `${head}\n\n${nextBlock(t, bySlug)}`;
}

function lessonStub(t, bySlug) {
  const tier = (h) => `${h}\n\n${authoring("exercise")}\n\n<details><summary>Solution</summary>\n\n${authoring("solution")}\n\n</details>`;
  const body = {
    "## By the end of this lesson": bullets(t.outcomes) || authoring("2–3 concrete outcomes"),
    "## What & why": `${t.summary || ""}\n\n${authoring("real-world problem + plain-language analogy")}`,
    "## The idea, slowly": `${authoring("small steps; after each listing add ### Line by line and ### Run it")}`,
    "## You might be wondering…": authoring("pre-emptive FAQ: the questions a confused beginner asks here"),
    "## Coming from another language?": authoring("Express / Flask / Spring / Go equivalents"),
    "## Common mistakes": authoring("real errors, what they mean, the fix"),
    "## More examples": authoring("4 × (### title, one-line hook, code)"),
    "## Your turn": TIERS.map(tier).join("\n\n"),
    "## Quick check": `<div class="quiz" data-topic="${t.slug}"></div>`,
    "## Remember this": authoring("3–5 takeaways"),
    "## Go deeper": goDeeper(t, bySlug),
  };
  return HEADINGS.lesson.map((h) => `${h}\n\n${body[h]}`).join("\n\n");
}

export function stubFor(t, bySlug) {
  const head = `# ${t.title}\n\n> **${t.level}** · ${partOf(t).title}`;
  let body;
  if (t.kind === "lesson") body = lessonStub(t, bySlug);
  else if (t.kind === "project") body = HEADINGS.project.map((h) => `${h}\n\n${authoring(h.slice(3))}`).join("\n\n");
  else if (t.kind === "checklist") body = `${authoring("- [ ] items, each linking to the lesson that teaches it")}`;
  else body = `${authoring("one-page How do I…? list, each item links to its lesson")}`;
  return `${head}\n\n${body}\n`;
}

export function summaryFor(topics) {
  let s = "# Summary\n\n[Introduction](introduction.md)\n[Glossary](glossary.md)\n";
  for (const p of PARTS) {
    const inPart = topics.filter((t) => t.part === p.id);
    if (!inPart.length) continue;
    s += `\n# ${p.title}\n\n`;
    for (const t of inPart) s += t.status === "published" ? `- [${t.title}](${pagePath(t)})\n` : `- [${t.title}]()\n`;
  }
  return s + "\n---\n\n[Review & flashcards](review.md)\n";
}

const quizzable = (t) => t.status === "published" && t.kind === "lesson";

export function bundleFor(topics, banks) {
  const live = topics.filter(quizzable);
  const q = Object.fromEntries(live.filter((t) => banks[t.slug]).map((t) => [t.slug, banks[t.slug]]));
  const order = live.map((t) => ({ slug: t.slug, title: t.title, category: partOf(t).title }));
  return (
    "// AUTO-GENERATED by tools/generate.mjs — do not edit by hand.\n" +
    `window.RUST_QUESTIONS = ${JSON.stringify(q)};\n` +
    `window.RUST_TOPIC_ORDER = ${JSON.stringify(order)};\n`
  );
}

export const questionStub = (t) =>
  JSON.stringify({ topic: t.slug, title: t.title, quiz: [], flashcards: [] }, null, 2) + "\n";

export { quizzable };

// ---------------------------------------------------------------------------
// Validation

// CommonMark fence rule, widened: an opener is a run of leading spaces/tabs/
// blockquote markers (so a fence nested in a list item's indented content or
// inside a blockquote is still recognized), then a run of >=3 backticks or
// tildes (an info string may follow). A line only closes it if — after the
// same prefix-stripping — it is a run of the SAME char with length >= the
// opener's run, followed by nothing but whitespace. Anything else inside the
// fence is just content (including lines that look like a different/shorter
// fence).
export const FENCE_PREFIX = /^[ \t>]*/;
const FENCE_OPEN = /^[ \t>]*(`{3,}|~{3,})/;

export function stripFences(md) {
  let fence = null; // { char, len } while inside an open fence
  return md.split("\n").map((line) => {
    // A trailing \r survives `line.split("\n")` on raw (non-normalized) CRLF text; strip it only
    // for the opener/closer tests below — the line returned for a non-fenced line is untouched.
    const stripped = line.replace(FENCE_PREFIX, "").replace(/\r$/, "");
    if (fence) {
      const close = new RegExp(`^${fence.char}{${fence.len},}\\s*$`);
      if (close.test(stripped)) fence = null;
      return "";
    }
    const m = /^(`{3,}|~{3,})(.*)$/.exec(stripped);
    if (m) { fence = { char: m[1][0], len: m[1].length }; return ""; }
    return line;
  }).join("\n");
}

export function headingId(text) {
  let id = "";
  for (const ch of text) {
    if (/[\p{L}\p{N}_-]/u.test(ch)) id += ch.toLowerCase();
    else if (/\s/.test(ch)) id += "-";
  }
  return id;
}

export function glossaryIds(md) {
  const ids = new Set();
  for (const m of stripFences(md).matchAll(/^#{2,4} +(.+)$/gm)) ids.add(headingId(m[1].trim()));
  return ids;
}

const BANNED = ["simply", "just", "obviously", "trivially"];
const esc = (x) => x.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// Body of a "## …" (or "### …") section: from the heading line (trailing whitespace allowed)
// to the next heading of the same or a higher level.
function sectionBody(prose, heading) {
  const level = heading.match(/^#+/)[0].length;
  const m = new RegExp(`^${esc(heading)}[ \\t]*$`, "m").exec(prose);
  if (!m) return "";
  const rest = prose.slice(m.index + m[0].length);
  const end = rest.search(new RegExp(`^#{1,${level}} `, "m"));
  return end < 0 ? rest : rest.slice(0, end);
}

// Every fenced code block: its info string and body lines (CommonMark rules, as stripFences).
// Body lines have the same leading spaces/tabs/blockquote-marker prefix (and, on raw CRLF text, a
// trailing \r) stripped as the fence markers themselves, so a fence nested in a list item or
// blockquote reads like a top-level one, and CRLF input doesn't hide fences from the opener test.
export function fencedBlocks(md) {
  const blocks = [];
  let open = null;
  md.split("\n").forEach((line, n) => {
    const stripped = line.replace(FENCE_PREFIX, "").replace(/\r$/, "");
    if (open) {
      if (new RegExp(`^${open.char}{${open.len},}\\s*$`).test(stripped)) { blocks.push(open); open = null; }
      else open.body.push(stripped);
      return;
    }
    const m = /^(`{3,}|~{3,})(.*)$/.exec(stripped);
    if (m) open = { char: m[1][0], len: m[1].length, info: m[2].trim(), body: [], line: n + 1 };
  });
  if (open) blocks.push(open);
  return blocks;
}

// Relative links to .md pages must resolve to an existing, published page. Checks inline links
// (`[x](y.md)`), reference-style definitions (`[x]: y.md`) and HTML anchors (`<a href="y.md">`).
// publishedPaths: absolute paths of every published page (+ introduction/glossary/review).
export function checkLinks({ md, mdFile, exists, publishedPaths }) {
  const errors = [];
  const plain = stripFences(md.replace(/\r\n/g, "\n")).replace(/`[^`\n]*`/g, "");
  const LINK_PATTERNS = [
    /\]\((?![a-z][a-z0-9+.-]*:)([^)\s#]+\.md)(#[^)\s]*)?(?:\s+"[^"]*")?\)/gi,
    // Reference-style link definition — but not a footnote definition (`[^1]: ...`), which is
    // not a page link at all.
    /^\s*\[(?!\^)[^\]]+\]:\s*(?![a-z][a-z0-9+.-]*:)(\S+?\.md)(#\S*)?\s*$/gim,
    /<a\s[^>]*href="(?![a-z][a-z0-9+.-]*:)([^"#]+\.md)(#[^"]*)?"/gi,
  ];
  for (const LINK of LINK_PATTERNS) {
    for (const m of plain.matchAll(LINK)) {
      const file = _resolve(_dirname(mdFile), m[1]);
      if (!exists(file)) errors.push(`link target not found: ${m[1]}`);
      else if (publishedPaths && !publishedPaths.has(file)) errors.push(`link to a page that is not published: ${m[1]}`);
    }
  }
  return errors;
}

// "- Title (coming soon)" is stale once Title is published.
export function checkComingSoon(md, publishedTitles) {
  const errors = [];
  for (const m of stripFences(md.replace(/\r\n/g, "\n")).matchAll(/^[-*+] (.+?) \(coming soon\)\s*$/gm))
    if (publishedTitles && publishedTitles.has(m[1]))
      errors.push(`"${m[1]} (coming soon)" is stale: that page is published — link it (run node tools/generate.mjs)`);
  return errors;
}

const INCLUDE = /\{\{#(include|rustdoc_include|playground)\s+([^}\s]+)[^}]*\}\}/g;

export function checkPage({ topic: t, md, mdFile, root, glossary, exists, read, publishedPaths, publishedTitles }) {
  const errors = [], warnings = [];
  md = md.replace(/\r\n/g, "\n");
  const prose = stripFences(md);
  const required = HEADINGS[t.kind] || [];

  // 1. required headings: present, and in order
  const found = [...prose.matchAll(/^## .+$/gm)].map((m) => m[0].trim());
  for (const h of required) if (!found.includes(h)) errors.push(`missing section "${h}"`);
  const seq = found.filter((h) => required.includes(h));
  if (required.every((h) => found.includes(h)) && seq.join("|") !== required.join("|"))
    errors.push(`sections out of order: expected ${required.join(" → ")}`);

  // 2. lesson-only rules
  if (t.kind === "lesson") {
    if (!md.includes(`<div class="quiz" data-topic="${t.slug}"></div>`)) errors.push("quiz mount div missing or wrong slug");
    const turn = sectionBody(prose, "## Your turn");
    let at = -1;
    for (const tier of TIERS) {
      const m = new RegExp(`^${esc(tier)}[ \\t]*$`, "m").exec(turn);
      const i = m ? m.index : -1;
      if (i <= at) { errors.push(`"## Your turn" is missing "${tier}" (or it is out of order)`); continue; }
      at = i;
      if (!sectionBody(turn, tier).includes("<details>"))
        errors.push(`"${tier}" has no <details> solution block`);
    }
    if (md.split("\n").some((l) => FENCE_OPEN.test(l)) && !/^### Line by line[ \t]*$/m.test(prose))
      errors.push('page has code but no "### Line by line" section');
    if (!md.includes(NEXT_START) || !md.includes(NEXT_END))
      warnings.push(`no ${NEXT_START} … ${NEXT_END} markers in "## Go deeper": generate.mjs can't keep the Next block current`);
  }

  // 3. checklist items must link to a lesson
  if (t.kind === "checklist")
    for (const line of prose.split("\n"))
      if (/^- \[ \]/.test(line) && !line.includes("]("))
        errors.push(`checklist item has no lesson link: ${line.slice(6).trim()}`);
  if (t.kind === "cheatsheet" && !prose.includes("](")) warnings.push("cheat sheet links to no lessons");

  // 4. includes resolve (scan raw md: includes live inside fences)
  for (const m of md.matchAll(INCLUDE)) {
    const [kind, arg] = [m[1], m[2]];
    const [rel, ...rest] = arg.split(":");
    const file = _resolve(_dirname(mdFile), rel);
    if (!exists(file)) { errors.push(`${kind} file not found: ${rel}`); continue; }
    if (!rest.length) continue;
    if (kind === "playground") { errors.push(`playground ${arg} takes no anchor or line range — point it at a whole file`); continue; }
    if (rest[0] === "" || /^\d/.test(rest[0])) { warnings.push(`${kind} ${arg} uses line numbers — use an ANCHOR so edits can't shift it`); continue; }
    const name = esc(rest[0]);
    const src = read(file);
    if (!new RegExp(`ANCHOR:\\s*${name}(?![\\w-])`).test(src) || !new RegExp(`ANCHOR_END:\\s*${name}(?![\\w-])`).test(src))
      errors.push(`anchor "${rest[0]}" not found (ANCHOR + ANCHOR_END) in ${rel}`);
  }

  // 5. Rust listings come from compiled code. Exceptions: ```rust,editable (pure-std, runs on
  //    the Playground) and fences tagged `ignore` (deliberately broken Common-mistakes code).
  //    A fence may hold only {{#include}}/{{#rustdoc_include}}/{{#playground}} lines — never a
  //    mix of an include and hand-typed code, which would silently drift from the real source.
  for (const b of fencedBlocks(md)) {
    if (!/^rust\b/.test(b.info)) continue;
    const tags = b.info.split(",").map((x) => x.trim());
    if (tags[1] === "editable" || tags.includes("ignore")) continue;
    const inc = b.body.filter((l) => /\{\{#(include|rustdoc_include|playground)\s/.test(l));
    const other = b.body.filter((l) => l.trim() && !inc.includes(l));
    if (inc.length && other.length)
      errors.push(`line ${b.line}: fence mixes an {{#include}} with hand-typed lines — put all of it in code/`);
    else if (!inc.length)
      errors.push(`line ${b.line}: hand-typed Rust in a \`\`\`${b.info} fence — {{#include}} it from code/, or use rust,editable (pure std) / rust,noplayground,ignore (deliberately broken)`);
  }

  // 5b. SQL listings come from code/sql/<slug>/NN-name.sql, run for real by tools/sql-check.mjs.
  //     Exception: fences tagged `ignore` (deliberately broken Common-mistakes code, shown next to
  //     its real error). Same mixing rule as Rust: a fence may hold only {{#include}} lines, never
  //     an include mixed with hand-typed SQL.
  for (const b of fencedBlocks(md)) {
    if (!/^sql\b/.test(b.info)) continue;
    const tags = b.info.split(",").map((x) => x.trim());
    if (tags.includes("ignore")) continue;
    const inc = b.body.filter((l) => /\{\{#include\s/.test(l));
    const other = b.body.filter((l) => l.trim() && !inc.includes(l));
    if (inc.length && other.length)
      errors.push(`line ${b.line}: fence mixes an {{#include}} with hand-typed lines — put all of it in code/`);
    else if (!inc.length)
      errors.push(`line ${b.line}: hand-typed SQL in a \`\`\`${b.info} fence — put it in code/sql/<slug>/NN-name.sql and {{#include}} it`);
    // Only code/sql/ is executed by sql-check, so a sql fence including anything else would show
    // SQL nobody runs.
    const sqlRoot = _resolve(root, "code/sql");
    for (const l of inc) {
      const m = /\{\{#include\s+([^}\s]+)\s*\}\}/.exec(l);
      if (!m) continue;
      const rel = m[1].split(":")[0];
      const within = _relative(sqlRoot, _resolve(_dirname(mdFile), rel));
      if (!(within && !within.startsWith("..") && !_isAbsolute(within)))
        errors.push(`line ${b.line}: a \`\`\`sql fence must include a file under code/sql/ (only that folder is run by sql-check): ${rel}`);
    }
  }

  // 6. links and stale "coming soon"
  errors.push(...checkLinks({ md, mdFile, exists, publishedPaths }));
  errors.push(...checkComingSoon(md, publishedTitles));

  // 7. leftovers, banned words, glossary links, codeDir
  if (md.includes("AUTHORING:")) errors.push("leftover AUTHORING placeholder");
  const plain = prose.replace(/`[^`\n]*`/g, "");
  for (const w of BANNED) {
    const n = (plain.match(new RegExp(`\\b${w}\\b`, "gi")) || []).length;
    if (n) warnings.push(`banned word "${w}" used ${n}×`);
  }
  for (const m of md.matchAll(/glossary\.md#([\w-]+)/g))
    if (!glossary.has(m[1])) warnings.push(`glossary link #${m[1]} has no matching glossary heading`);
  if (t.codeDir && !exists(_resolve(root, t.codeDir))) errors.push(`codeDir does not exist: ${t.codeDir}`);

  return { errors, warnings };
}

export function checkBank(t, bank) {
  const errors = [], warnings = [];
  const quiz = Array.isArray(bank.quiz) ? bank.quiz : [];
  const cards = Array.isArray(bank.flashcards) ? bank.flashcards : [];
  if (quiz.length < 4) warnings.push(`quiz has fewer than 4 questions (${quiz.length})`);
  if (!cards.length) warnings.push("no flashcards");
  quiz.forEach((q, i) => {
    if (!q.q) errors.push(`quiz[${i}] missing question text`);
    if (!Array.isArray(q.options) || q.options.length < 2) errors.push(`quiz[${i}] needs at least 2 options`);
    if (typeof q.answer !== "number" || q.answer < 0 || q.answer >= (q.options || []).length)
      errors.push(`quiz[${i}] answer index out of range`);
    if (!q.explain) warnings.push(`quiz[${i}] has no explanation`);
  });
  cards.forEach((c, i) => { if (!c.front || !c.back) errors.push(`flashcard[${i}] missing front/back`); });
  return { errors, warnings };
}

export function checkTopics(topics) {
  const errors = [], seen = new Set();
  for (const t of topics) {
    if (seen.has(t.slug)) errors.push(`duplicate slug ${t.slug}`);
    seen.add(t.slug);
    if (!PARTS.some((p) => p.id === t.part)) errors.push(`${t.slug}: unknown part ${t.part}`);
    if (!(t.kind in HEADINGS)) errors.push(`${t.slug}: unknown kind ${t.kind}`);
    if (t.status === "published" && t.kind === "lesson" && (!t.outcomes || t.outcomes.length < 2))
      errors.push(`${t.slug}: published lesson needs at least 2 outcomes`);
  }
  const slugs = new Set(topics.map((t) => t.slug));
  for (const t of topics)
    for (const field of ["next", "prereq"])
      for (const s of t[field] || [])
        if (!slugs.has(s)) errors.push(`${t.slug}: ${field} names unknown slug "${s}"`);
  return errors;
}

