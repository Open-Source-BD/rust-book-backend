import { test } from "node:test";
import assert from "node:assert/strict";
import { parseScript, stripVolatile, transcript } from "../http.mjs";

test("parseScript reads the serve header and the commands", () => {
  const s = parseScript("# serve: -p hello-axum --example about\n# a comment\n\ncurl -i http://127.0.0.1:3000/\ncurl -i http://127.0.0.1:3000/about\n");
  assert.deepEqual(s.serve, ["-p", "hello-axum", "--example", "about"]);
  assert.deepEqual(s.commands, ["curl -i http://127.0.0.1:3000/", "curl -i http://127.0.0.1:3000/about"]);
});

test("parseScript rejects a script without a usable serve header", () => {
  assert.throws(() => parseScript("curl -i http://127.0.0.1:3000/\n"), /serve/);
  assert.throws(() => parseScript("# serve: --example x\ncurl x\n"), /-p/);
});

test("stripVolatile drops only the date header", () => {
  const raw = "HTTP/1.1 200 OK\r\ncontent-length: 2\r\ndate: Wed, 30 Sep 2026 15:38:54 GMT\r\nDate: again\r\n\r\nok";
  assert.equal(stripVolatile(raw), "HTTP/1.1 200 OK\ncontent-length: 2\n\nok");
});

test("transcript prefixes each command with $ and separates blocks", () => {
  assert.equal(transcript(["curl a", "curl b"], ["A", "B\n"]), "$ curl a\nA\n\n$ curl b\nB\n");
});

import { commandResult, unknownPackages, lastLines } from "../http.mjs";

test("commandResult fails on error, signal or non-zero status", () => {
  assert.deepEqual(commandResult({ status: 0 }), { ok: true, reason: "" });
  assert.match(commandResult({ error: new Error("boom"), status: null }).reason, /boom/);
  assert.match(commandResult({ signal: "SIGTERM", status: null }).reason, /SIGTERM/);
  const bad = commandResult({ status: 7 });
  assert.equal(bad.ok, false);
  assert.match(bad.reason, /status 7/);
});

test("unknownPackages lists requested names with no http/ scripts", () => {
  assert.deepEqual(unknownPackages(["a", "nosuch"], ["a", "b"]), ["nosuch"]);
  assert.deepEqual(unknownPackages([], ["a"]), []);
});

test("lastLines keeps only the tail, skipping blanks", () => {
  assert.equal(lastLines("a\n\nb\nc\n", 2), "b\nc");
});
