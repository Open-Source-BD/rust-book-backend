import fs from "node:fs";
import net from "node:net";
import { spawn, spawnSync } from "node:child_process";
import { dirname, join, relative } from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { normalizeOutput, diffLines, planRun } from "./sql.mjs";
import { parseScript, stripVolatile, transcript, commandResult, unknownPackages, lastLines } from "./http.mjs";

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
  const server = spawn("cargo", ["run", "-q", ...serve], { cwd: CODE, detached: true, stdio: ["ignore", "ignore", "pipe"] });
  let stderr = "";
  let exited = null;
  server.stderr.on("data", (d) => { stderr = (stderr + d).slice(-20000); });
  server.on("exit", (code, signal) => { exited = { code, signal }; });
  server.on("error", (e) => { exited = { code: `spawn error: ${e.message}`, signal: null }; });
  try {
    // Wait for the port; give up at once if the server dies first (120 s cap as a backstop).
    let up = false;
    for (let t = 0; t < 120000 && !up; t += 100) {
      if (await portOpen()) { up = true; break; }
      if (exited) {
        await sleep(50); // let the last stderr chunks arrive
        const tail = lastLines(stderr);
        throw new Error(`server exited (code ${exited.code ?? exited.signal}) before opening port ${PORT}${tail ? `\n${tail}` : ""}`);
      }
      await sleep(100);
    }
    if (!up) throw new Error(`server (${serve.join(" ")}) never opened port ${PORT}`);
    const outputs = [];
    for (const c of commands) {
      const r = spawnSync("sh", ["-c", `${c} 2>&1`], { cwd: ROOT, encoding: "utf8", timeout: 30000, env: ENV });
      const res = commandResult(r);
      if (!res.ok) throw new Error(`command failed (${res.reason}): ${c}`);
      outputs.push(stripVolatile(r.stdout || ""));
    }
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
const b = spawnSync("cargo", ["build", "-q", "--workspace", "--bins", "--examples"], { cwd: CODE, stdio: "inherit" });
if (b.status !== 0) process.exit(2);
const scripts = [];
const known = [];
for (const kind of ["topics", "projects"]) {
  const dir = join(CODE, kind);
  if (!fs.existsSync(dir)) continue;
  for (const pkg of fs.readdirSync(dir).sort()) {
    const http = join(dir, pkg, "http");
    if (!fs.existsSync(http)) continue;
    known.push(pkg);
    if (only.length && !only.includes(pkg)) continue;
    let plan;
    try { plan = planRun(fs.readdirSync(http), ".sh"); } catch (e) {
      console.log(`FAIL  ${pkg}: ${e.message}`);
      failed++;
      continue;
    }
    const { sql: files, orphans } = plan;
    for (const o of orphans) { console.log(`FAIL  ${relative(ROOT, join(http, o))}: no matching .sh`); failed++; }
    for (const f of files) scripts.push(join(http, f));
  }
}
for (const name of unknownPackages(only, known)) { console.log(`FAIL  ${name}: no http/ scripts for this package`); failed++; }
for (const file of scripts) {
  const rel = relative(ROOT, file);
  const outFile = file.replace(/\.sh$/, ".out");
  let out;
  try { out = await runScript(file); } catch (e) { console.log(`FAIL  ${rel}: ${e.message}`); failed++; continue; }
  if (TWICE) {
    let again;
    try { again = await runScript(file); } catch (e) { console.log(`FAIL  ${rel}: second run failed: ${e.message}`); failed++; continue; }
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
