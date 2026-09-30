import fs from "node:fs";
import net from "node:net";
import { spawn, spawnSync } from "node:child_process";
import { dirname, join, relative } from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { normalizeOutput, diffLines, planRun } from "./sql.mjs";
import { parseScript, stripVolatile, transcript } from "./http.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const CODE = join(ROOT, "code");
const PORT = 3000;
const args = process.argv.slice(2);
const UPDATE = args.includes("--update");
const TWICE = args.includes("--check-twice");
const only = args.filter((a) => !a.startsWith("--"));

const portOpen = () => new Promise((res) => {
  const s = net.connect(PORT, "127.0.0.1");
  s.once("connect", () => { s.destroy(); res(true); });
  s.once("error", () => res(false));
});
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function waitFor(want, ms) {
  for (let t = 0; t < ms; t += 100) { if ((await portOpen()) === want) return true; await sleep(100); }
  return false;
}
async function runScript(file) {
  const { serve, commands } = parseScript(fs.readFileSync(file, "utf8"));
  if (await portOpen()) throw new Error(`port ${PORT} is already in use — stop whatever is listening there`);
  const server = spawn("cargo", ["run", "-q", ...serve], { cwd: CODE, detached: true, stdio: "ignore" });
  try {
    if (!(await waitFor(true, 120000))) throw new Error(`server (${serve.join(" ")}) never opened port ${PORT}`);
    const outputs = commands.map((c) => {
      const r = spawnSync("sh", ["-c", `${c} 2>&1`], { cwd: ROOT, encoding: "utf8", timeout: 30000, env: ENV });
      return stripVolatile(r.stdout || "");
    });
    return normalizeOutput(transcript(commands, outputs));
  } finally {
    try { process.kill(-server.pid, "SIGTERM"); } catch {}
    await waitFor(false, 15000);
  }
}
// curl prints a progress meter (with timings) when its output is not a terminal. Readers run curl in
// a terminal and never see it, so give curl a private config that turns it off; the command shown on
// the page stays exactly what the reader types.
const CURL_HOME = fs.mkdtempSync(join(os.tmpdir(), "rbh-curl-"));
fs.writeFileSync(join(CURL_HOME, ".curlrc"), "no-progress-meter\n");
const ENV = { ...process.env, CURL_HOME, XDG_CONFIG_HOME: CURL_HOME };
let failed = 0;
const b = spawnSync("cargo", ["build", "-q", "--workspace", "--bins"], { cwd: CODE, stdio: "inherit" });
if (b.status !== 0) process.exit(2);
const scripts = [];
for (const kind of ["topics", "projects"]) {
  const dir = join(CODE, kind);
  if (!fs.existsSync(dir)) continue;
  for (const pkg of fs.readdirSync(dir).sort()) {
    const http = join(dir, pkg, "http");
    if (!fs.existsSync(http) || (only.length && !only.includes(pkg))) continue;
    const { sql: files, orphans } = planRun(fs.readdirSync(http), ".sh");
    for (const o of orphans) { console.log(`FAIL  ${relative(ROOT, join(http, o))}: no matching .sh`); failed++; }
    for (const f of files) scripts.push(join(http, f));
  }
}
for (const file of scripts) {
  const rel = relative(ROOT, file);
  const outFile = file.replace(/\.sh$/, ".out");
  let out;
  try { out = await runScript(file); } catch (e) { console.log(`FAIL  ${rel}: ${e.message}`); failed++; continue; }
  if (TWICE) {
    const again = await runScript(file);
    if (again !== out) { console.log(`FAIL  ${rel}: output differs between two runs`); failed++; continue; }
  }
  if (UPDATE) { fs.writeFileSync(outFile, out); console.log(`wrote ${relative(ROOT, outFile)}`); continue; }
  if (!fs.existsSync(outFile)) { console.log(`FAIL  ${rel}: missing .out — run: node tools/http-check.mjs --update`); failed++; continue; }
  const d = diffLines(fs.readFileSync(outFile, "utf8"), out);
  if (d.length) { failed++; console.log(`FAIL  ${rel}: output changed`); for (const x of d.slice(0, 10)) console.log(`  line ${x.line}\n    - ${x.expected ?? "(none)"}\n    + ${x.actual ?? "(none)"}`); }
  else console.log(`ok    ${rel}`);
}
console.log(failed ? `\n${failed} HTTP check(s) failed` : "\nall HTTP outputs match");
process.exit(failed ? 1 : 0);
