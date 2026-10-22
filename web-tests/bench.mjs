// The performance benchmark in a browser (M8 step C4; our own tooling).
//
// Serves dist/web, opens the game client with `?mode=game&bench=SPEC` in a
// Playwright browser at the benchmark's size, picks the install folder
// through the folder input, waits for the page to report the benchmark done
// (`data-glasnost-bench`), and writes the summary and the per-frame rows.
// The install stays local: it is read from DIR, never copied or committed.
//
// Usage: node bench.mjs --install DIR [--browser chromium|firefox] [--backend webgl2|webgpu|auto]
//                       [--spec SPEC] [--gpu] [--headed] [--dist ../dist/web] [--out DIR] [--timeout MS]
//
// SPEC is the benchmark's key=value words (docs/development.md, "Performance (M8 C4)"); the
// default is "default". Without --gpu the browser renders in software (SwiftShader in Chromium,
// the browser's own fallback in Firefox), as the smoke lane does; with --gpu it uses the
// machine's GPU where the browser allows. Writes <out>/<label>.txt and .csv (label: the spec's,
// else web-<browser>). Exit code 0 when the benchmark finished.

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
if (typeof args.install !== "string") {
  console.error("usage: node bench.mjs --install DIR [--browser chromium|firefox] [--spec SPEC] [--gpu]");
  process.exit(2);
}
const browserName = args.browser ?? "chromium";
const backend = args.backend ?? "webgl2";
const dist = path.resolve(here, args.dist ?? "../dist/web");
const outDir = path.resolve(here, args.out ?? "../local-lane/m8/c4/web");
const timeoutMs = Number(args.timeout ?? 3_600_000);
const gpu = args.gpu === true;
let spec = typeof args.spec === "string" ? args.spec : "default";
if (!/(^|[ ,;])label=/.test(spec)) spec += ` label=web-${browserName}${gpu ? "-gpu" : ""}`;
const labelOf = (s) => (s.match(/(?:^|[ ,;])label=([^ ,;]+)/) ?? [null, "web"])[1];
const size = (spec.match(/(?:^|[ ,;])size=(\d+)x(\d+)/) ?? [null, "1920", "1080"]).slice(1).map(Number);

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
const query = new URLSearchParams({ backend, mode: "game", bench: spec });
const url = `http://127.0.0.1:${server.address().port}/?${query}`;

const chromiumArgs = gpu
  ? ["--ignore-gpu-blocklist", "--enable-gpu-rasterization", "--disable-gpu-vsync", "--disable-frame-rate-limit"]
  : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"];
const browser = await playwright[browserName].launch({
  headless: !args.headed,
  ...(browserName === "chromium" ? { args: chromiumArgs } : {}),
  ...(browserName === "firefox"
    ? {
        firefoxUserPrefs: {
          "webgl.force-enabled": true,
          // Frames as fast as the page draws them, and a clock finer than Firefox's 1 ms default.
          "layout.frame_rate": 0,
          "privacy.reduceTimerPrecision": false,
          "dom.webgpu.enabled": backend !== "webgl2",
        },
      }
    : {}),
});
let ok = false;
try {
  const page = await browser.newPage({ viewport: { width: size[0], height: size[1] }, deviceScaleFactor: 1, acceptDownloads: true });
  // Every console line goes to <out>/<label>.console.txt; the benchmark's own lines also here.
  fs.mkdirSync(outDir, { recursive: true });
  const consoleLog = fs.createWriteStream(path.join(outDir, `${labelOf(spec)}.console.txt`));
  page.on("console", (m) => {
    const t = m.text();
    consoleLog.write(`[${m.type()}] ${t}\n`);
    if (/bench|error|panic/i.test(t)) console.log(`[${browserName}] ${t.split("\n")[0]}`);
  });
  page.on("pageerror", (e) => console.log(`[${browserName}] pageerror: ${e.message}`));
  await page.goto(url);
  await page.waitForFunction(() => ["presented", "error"].includes(document.body.dataset.glasnostStatus), null, {
    timeout: 120_000,
  });
  const status = await page.evaluate(() => ({ ...document.body.dataset }));
  if (status.glasnostStatus === "error") throw new Error(`app reported an error: ${status.glasnostDetail}`);
  console.log(`[${browserName}] presented with ${status.glasnostBackend}; picking the install folder`);
  await page.locator("#glasnost-folder").setInputFiles(path.resolve(args.install));
  // Wait for the end; every minute, a screenshot of the page shows how far it got.
  const started = Date.now();
  let data;
  for (;;) {
    data = await page.evaluate(() => ({ ...document.body.dataset }));
    if (["done", "failed"].includes(data.glasnostBench)) break;
    if (Date.now() - started > timeoutMs) throw new Error(`no result within ${timeoutMs} ms`);
    await page.waitForTimeout(60_000);
    await page.screenshot({ path: path.join(outDir, `${labelOf(spec)}.progress.png`) }).catch(() => {});
    console.log(`[${browserName}] ${Math.round((Date.now() - started) / 1000)} s, status ${data.glasnostStatus}`);
  }
  const label = labelOf(spec);
  fs.writeFileSync(path.join(outDir, `${label}.txt`), data.glasnostBenchSummary ?? "");
  fs.writeFileSync(path.join(outDir, `${label}.csv`), data.glasnostBenchCsv ?? "");
  console.log(data.glasnostBenchSummary ?? "(no summary)");
  console.log(`[${browserName}] benchmark ${data.glasnostBench}; wrote ${path.join(outDir, label)}.txt and .csv`);
  ok = data.glasnostBench === "done";
} catch (e) {
  console.error(`FAIL [${browserName}]: ${e.message}`);
} finally {
  await browser.close();
  server.close();
}
process.exit(ok ? 0 : 1);
