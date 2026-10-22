// The soak's wasm client in a real browser (M9 N8c): the game client built
// for wasm32 (crates/headless-web), played by our AI as a person, in a
// Chromium or Firefox page driven by Playwright. Unlike the node driver
// (crates/headless-web/node/net-client.mjs) the page has RTCPeerConnection,
// so the client opens direct links through the browser's WebRTC, as the web
// client does, and `--relay-only` keeps it on the relay.
//
// It takes the options of `glasnost-headless net-client` (plus `--browser
// chromium|firefox|webkit`), prints the same status lines on standard
// output and writes the same hash log, report and code file; the page's
// console goes to standard error. Synthetic install only: the page has no
// install folder. `cargo run -p glasnost-tools --bin glasnost-headless --
// soak --browser-row R:chromium --browser-script web-tests/net-client.mjs`
// runs it as a soak row.
//
// Build the package first (docs/development.md, "Network lab (N8c)"):
//   cargo build --release --target wasm32-unknown-unknown -p glasnost-headless-web
//   wasm-bindgen --target web --out-dir target/headless-web-page \
//     target/wasm32-unknown-unknown/release/glasnost_headless_web.wasm
// GLASNOST_WASM_WEB_PKG names another package folder.
//
// Host candidates are offered as addresses, not mDNS names (a headless
// machine may not resolve those): a flag for Chromium, a pref for Firefox,
// as in rtc.mjs. Our own harness; no spec applies.

import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as playwright from "playwright";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkg = path.resolve(process.env.GLASNOST_WASM_WEB_PKG ?? path.join(here, "../target/headless-web-page"));

const all = process.argv.slice(2);
const bi = all.indexOf("--browser");
const browserName = bi >= 0 ? all[bi + 1] : "chromium";
const args = bi >= 0 ? [...all.slice(0, bi), ...all.slice(bi + 2)] : all;
const opt = (name) => {
  const i = args.indexOf(name);
  return i >= 0 && i + 1 < args.length ? args[i + 1] : undefined;
};
if (args.includes("--game-dir")) {
  process.stdout.write("failed the browser driver runs the synthetic install only\n");
  process.exit(1);
}
if (!args.includes("--synthetic")) args.push("--synthetic");
const hashLog = opt("--hash-log");
const report = opt("--report");
const codeFile = opt("--code-file");

const LOBBY_TIMEOUT_MS = 900_000; // our own harness: as the native client
const LINGER_MS = 3_000; // our own harness: as the native client

const page_html = `<!doctype html><meta charset="utf-8"><title>net-client</title>
<script type="module">
import init, { NetClient } from "./glasnost_headless_web.js";
globalThis.glasnostReadAt = () => { throw new Error("no install in the page"); };
globalThis.glasnostSaveReport = (name, bytes) => {
  globalThis.glasnostReport(name, Array.from(bytes));
  return "page:" + name;
};
globalThis.glasnostRun = async (args, lobbyTimeout, linger) => {
  await init();
  const out = (o) => globalThis.glasnostOut(o);
  const t0 = performance.now();
  let client;
  try {
    client = new NetClient(args, [], []);
  } catch (e) {
    await out({ lines: ["failed " + (e.message ?? e)] });
    return 1;
  }
  await out({ lines: ["mounted " + Math.round(performance.now() - t0) + " ms"] });
  const clock = performance.now();
  const now = () => performance.now() - clock;
  // A yield that is not clamped like a nested setTimeout.
  const ch = new MessageChannel();
  const waiting = [];
  ch.port1.onmessage = () => waiting.shift()?.();
  const yieldNow = () => new Promise((r) => { waiting.push(r); ch.port2.postMessage(0); });
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  let code = undefined;
  let last = performance.now();
  while (!client.done()) {
    const lines = client.step(now());
    const hashes = client.hash_lines();
    if (code === undefined) code = client.code();
    if (lines.length > 0 || hashes.length > 0 || (code !== undefined && !out.sent)) {
      await out({ lines, hashes, code });
      if (code !== undefined) out.sent = true;
    }
    if (!client.in_game() && now() > lobbyTimeout) {
      await out({ lines: ["failed no game after " + lobbyTimeout + " ms: " + client.status()] });
      return 1;
    }
    // Let the socket's and the data channels' events in: a short sleep
    // now and then, a bare yield otherwise.
    if (performance.now() - last > 4) { await sleep(1); last = performance.now(); } else { await yieldNow(); }
  }
  await out({ hashes: client.hash_lines() });
  const end = performance.now();
  while (performance.now() - end < linger && client.networked()) {
    client.linger(now());
    await sleep(2);
  }
  const why = client.failure();
  await out({ report: client.report(), lines: [why !== undefined ? "failed " + why : client.summary_line()] });
  return why !== undefined ? 1 : 0;
};
globalThis.glasnostReady = true;
</script>`;

const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm" };
const server = http.createServer((req, res) => {
  const rel = decodeURIComponent(new URL(req.url, "http://x").pathname).replace(/^\/+/, "") || "index.html";
  if (rel === "index.html") {
    res.writeHead(200, { "Content-Type": "text/html" }).end(page_html);
    return;
  }
  const file = path.join(pkg, rel);
  if (!file.startsWith(pkg + path.sep) || !fs.existsSync(file)) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "Content-Type": types[path.extname(file)] ?? "application/octet-stream" });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

const browser = await playwright[browserName].launch({
  ...(browserName === "chromium" ? { args: ["--disable-features=WebRtcHideLocalIpsWithMdns"] } : {}),
  ...(browserName === "firefox"
    ? { firefoxUserPrefs: { "media.peerconnection.ice.obfuscate_host_addresses": false } }
    : {}),
});
let code = 1;
try {
  const page = await browser.newPage();
  page.on("console", (m) => process.stderr.write(`[console.${m.type()}] ${m.text()}\n`));
  page.on("pageerror", (e) => process.stderr.write(`[pageerror] ${e.message}\n`));
  let wroteCode = false;
  await page.exposeFunction("glasnostOut", (o) => {
    for (const l of o.lines ?? []) process.stdout.write(l + "\n");
    if (hashLog !== undefined && o.hashes) fs.appendFileSync(hashLog, o.hashes);
    if (!wroteCode && codeFile !== undefined && o.code) {
      fs.writeFileSync(codeFile, o.code);
      wroteCode = true;
    }
    if (report !== undefined && o.report !== undefined) fs.writeFileSync(report, o.report);
  });
  await page.exposeFunction("glasnostReport", (name, bytes) => {
    try {
      fs.mkdirSync("desync-reports", { recursive: true });
      fs.writeFileSync(path.join("desync-reports", path.basename(name)), Buffer.from(bytes));
    } catch (e) {
      process.stderr.write(`cannot store the report: ${e}\n`);
    }
  });
  await page.goto(`${base}/index.html`);
  await page.waitForFunction(() => globalThis.glasnostReady === true);
  code = await page.evaluate(([a, t, l]) => globalThis.glasnostRun(a, t, l), [args, LOBBY_TIMEOUT_MS, LINGER_MS]);
} catch (e) {
  process.stdout.write(`failed the page: ${e.message ?? e}\n`);
  code = 1;
} finally {
  await browser.close().catch(() => {});
  server.close();
}
process.exit(code);
