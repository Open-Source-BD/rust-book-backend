// http.mjs — pure helpers for tools/http-check.mjs (no I/O).
export function parseScript(text) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const serveLine = lines.find((l) => /^#\s*serve:/.test(l));
  if (!serveLine) throw new Error('missing "# serve: <cargo run args>" header line');
  const serve = serveLine.replace(/^#\s*serve:\s*/, "").trim().split(/\s+/).filter(Boolean);
  if (!serve.includes("-p")) throw new Error('"# serve:" must name a package with -p');
  const commands = lines.filter((l) => l.trim() && !l.trim().startsWith("#"));
  return { serve, commands };
}
export function stripVolatile(text) {
  return text.replace(/\r\n/g, "\n").split("\n").filter((l) => !/^date:\s/i.test(l)).join("\n");
}
export function transcript(commands, outputs) {
  return commands.map((c, i) => `$ ${c}\n${outputs[i].replace(/\n?$/, "\n")}`).join("\n");
}
// Turns a spawnSync result into { ok, reason }. A command that could not start, timed out, was killed
// by a signal, or exited non-zero fails the script (`curl -i` exits 0 on 4xx/5xx, so lessons never
// need a failing command).
export function commandResult(r) {
  if (r.error) return { ok: false, reason: `could not run: ${r.error.message}` };
  if (r.signal) return { ok: false, reason: `killed by signal ${r.signal}` };
  if (r.status !== 0) return { ok: false, reason: `exited with status ${r.status}` };
  return { ok: true, reason: "" };
}
// Package names asked for on the command line that have no http/ scripts.
export function unknownPackages(requested, known) {
  return requested.filter((p) => !known.includes(p));
}
// The last `n` non-empty lines of some captured output, for error messages.
export function lastLines(text, n = 20) {
  return text.split("\n").filter((l) => l.trim()).slice(-n).join("\n");
}
