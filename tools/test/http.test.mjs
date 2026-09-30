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
