// Browser check of the direct links' web adapter (M9 N7).
//
// Serves dist/web, opens `?mode=probe` (the page loads the module and runs
// nothing) and calls the module's `rtc_probe`: two RTCPeerConnections in
// one page, signalled to each other, must open and carry a packet each
// way with no ICE server at all, and an attempt fed an offer that does
// not read must end without a panic. Node has no RTCPeerConnection, so
// this is the only lane that runs the adapter.
//
// Usage: node rtc.mjs [--browser chromium|firefox|webkit] [--dist ../dist/web] [--timeout MS]
//
// Host candidates only: Chromium and Firefox hide local addresses behind
// mDNS names, which a headless machine may not resolve, so the lane turns
// that off (a flag, a pref). Exit code 0 on success.

import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as playwright from "playwright";

const here = path.dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(
  process.argv.slice(2).reduce((acc, a, i, all) => {
    if (a.startsWith("--")) acc.push([a.slice(2), all[i + 1]?.startsWith("--") ? true : all[i + 1] ?? true]);
    return acc;
  }, []),
);
const browserName = args.browser ?? "chromium";
const dist = path.resolve(here, args.dist ?? "../dist/web");
const timeoutMs = Number(args.timeout ?? 10000);

const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm" };
const server = http.createServer((req, res) => {
  const rel = decodeURIComponent(new URL(req.url, "http://x").pathname).replace(/^\/+/, "") || "index.html";
  const file = path.join(dist, rel);
  if (!file.startsWith(dist) || !fs.existsSync(file)) {
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
let failed = false;
try {
  const page = await browser.newPage();
  const logs = [];
  page.on("console", (m) => logs.push(`[console.${m.type()}] ${m.text()}`));
  page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));
  await page.goto(`${base}/?mode=probe`);
  const line = await page.evaluate(async (t) => {
    const m = await import("./glasnost_platform_web.js");
    // The page's own script may still be loading the module: initialise it
    // here too (on `?mode=probe` the start function does nothing).
    await m.default();
    return await m.rtc_probe(t);
  }, timeoutMs);
  console.log(`[${browserName}] ${line}`);
  const words = Object.fromEntries(line.split(" ").map((w) => w.split("=")));
  if (!("open" in words)) failed = true;
  if (words.echo !== "yes") failed = true;
  if (words.hostile !== "closed") failed = true;
  if (logs.some((l) => l.includes("panicked"))) {
    failed = true;
    console.error(logs.join("\n"));
  }
} catch (e) {
  console.error(`FAIL [${browserName}]: ${e.message}`);
  failed = true;
} finally {
  await browser.close();
  server.close();
}
if (failed) {
  console.error(`FAIL [${browserName}]: the direct link did not open, carry or refuse as it should`);
  process.exit(1);
}
console.log(`[${browserName}] direct links: ok`);
