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
