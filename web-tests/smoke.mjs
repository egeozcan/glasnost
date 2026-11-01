// Browser smoke test for the web build (PLAN.md 6.2, layer 3).
//
// Serves dist/web, opens it in a Playwright browser with a forced backend,
// waits for the app to report its first presented frame, and checks that the
// canvas shows the palette-indexed checkerboard (many distinct colours in
// the middle, the clear colour in the corner).
//
// Usage: node smoke.mjs [--browser chromium|firefox|webkit] [--backend webgl2|webgpu|auto]
//                       [--webgpu] [--expect WebGL2|WebGPU] [--dist ../dist/web] [--out out] [--headed]
//                       [--viewer [--install DIR] | --game]
//
// --viewer opens the asset viewer (?mode=viewer) instead of the demo and
// imports DIR (default: the synthetic MIX fixtures) through its folder input.
// --game opens the game client (?mode=game) and checks that its main menu
// draws.
//
// --webgpu turns on Chromium's WebGPU flags (implied by --backend webgpu), so
// `--backend auto --webgpu --expect WebGPU` checks the runtime selection.
// Exit code 0 on success. Writes a screenshot to <out>/<browser>-<backend>.png.

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
const backend = args.backend ?? "webgl2";
const dist = path.resolve(here, args.dist ?? "../dist/web");
const outDir = path.resolve(here, args.out ?? "out");
const timeoutMs = Number(args.timeout ?? 60000);
const webgpuFlags = backend === "webgpu" || args.webgpu === true;
const expectedBackend = args.expect ?? { webgl2: "WebGL2", webgpu: "WebGPU" }[backend];

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
const viewer = args.viewer === true;
const game = args.game === true;
const mode = viewer ? "&mode=viewer" : game ? "&mode=game" : "";
const suffix = viewer ? "-viewer" : game ? "-game" : "";
const url = `http://127.0.0.1:${server.address().port}/?backend=${backend}${mode}`;
// Synthetic install for the viewer lane: our own fixture archives, never game data.
const syntheticInstall = path.resolve(here, args.install ?? "../tests/fixtures/synthetic/mix");

// Software rendering so the lane runs on GPU-less CI machines.
const launchOptions = {
  headless: !args.headed,
  chromium: {
    args: webgpuFlags
      ? ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"]
      : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
  },
  firefox: { firefoxUserPrefs: { "webgl.force-enabled": true, "dom.webgpu.enabled": webgpuFlags } },
  webkit: {},
};

let failed = false;
const fail = (msg) => {
  console.error(`FAIL [${browserName}/${backend}]: ${msg}`);
  failed = true;
};

const browser = await playwright[browserName].launch({
  headless: launchOptions.headless,
  ...(browserName === "chromium" ? { args: launchOptions.chromium.args } : {}),
  ...(browserName === "firefox" ? launchOptions.firefox : {}),
});
try {
  const page = await browser.newPage({ viewport: { width: 640, height: 480 } });
  const logs = [];
  page.on("console", (m) => logs.push(`[console.${m.type()}] ${m.text()}`));
  page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));
  await page.goto(url);

  const handle = await page.waitForFunction(
    () => ["presented", "error"].includes(document.body.dataset.glasnostStatus) && { ...document.body.dataset },
    null,
    { timeout: timeoutMs },
  ).catch(() => null);
  const data = handle ? await handle.jsonValue() : null;

  if (!data) fail(`no status within ${timeoutMs} ms`);
  else if (data.glasnostStatus === "error") fail(`app reported an error: ${data.glasnostDetail}`);
  else {
    if (expectedBackend && data.glasnostBackend !== expectedBackend) {
      fail(`backend is ${data.glasnostBackend}, expected ${expectedBackend}`);
    }
    console.log(`[${browserName}/${backend}] presented with ${data.glasnostBackend} (${data.glasnostDetail})`);

    if (viewer) {
      // Import a synthetic folder through the folder input, as a user would.
      //
      // Playwright's Firefox backend sometimes never answers a directory
      // upload (one stall of 30 s on CI in 12 runs; 70 local runs, alone and
      // under load, never stalled). The page is not the cause the log can
      // show: the wait below proves its main thread still serves frames. The
      // upload is a protocol call the app does not own, so it gets a short
      // timeout and is retried; a retry that lands after the first one did
      // work only remounts the same folder.
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
      for (let attempt = 1; ; attempt++) {
        try {
          await page.locator("#glasnost-folder").setInputFiles(syntheticInstall, { timeout: 10000 });
          break;
        } catch (e) {
          if (attempt === 3) throw e;
          console.log(`[${browserName}/${backend}] folder upload attempt ${attempt} stalled; retrying`);
        }
      }
      const mount = await page
        .waitForFunction(() => document.body.dataset.glasnostMount, null, { timeout: timeoutMs })
        .then((h) => h.jsonValue())
        .catch(() => null);
      console.log(`[${browserName}/${backend}] viewer mount: ${mount}`);
      if (!mount || !mount.startsWith("mounted:")) fail(`viewer did not mount the synthetic folder (${mount})`);
    }

    // Let a few frames render, then screenshot just the canvas.
    await page.waitForTimeout(500);
    fs.mkdirSync(outDir, { recursive: true });
    const png = await page.locator("canvas").screenshot({ path: path.join(outDir, `${browserName}-${backend}${suffix}.png`) });

    // Decode the PNG in a blank page and measure it.
    const probe = await browser.newPage();
    const stats = await probe.evaluate(async (b64) => {
      const blob = await (await fetch(`data:image/png;base64,${b64}`)).blob();
      const bmp = await createImageBitmap(blob);
      const c = new OffscreenCanvas(bmp.width, bmp.height);
      const ctx = c.getContext("2d");
      ctx.drawImage(bmp, 0, 0);
      const px = (x, y) => Array.from(ctx.getImageData(x, y, 1, 1).data.slice(0, 3));
      const w = bmp.width, h = bmp.height;
      const box = ctx.getImageData(Math.floor(w * 0.4), Math.floor(h * 0.4), Math.floor(w * 0.2), Math.floor(h * 0.2)).data;
      const colours = new Set();
      for (let i = 0; i < box.length; i += 4) colours.add((box[i] << 16) | (box[i + 1] << 8) | box[i + 2]);
      return { w, h, corner: px(2, 2), centre: px(Math.floor(w / 2), Math.floor(h / 2)), distinct: colours.size };
    }, png.toString("base64"));
    await probe.close();

    console.log(`[${browserName}/${backend}] canvas ${stats.w}x${stats.h}, centre ${stats.centre}, corner ${stats.corner}, ${stats.distinct} colours in the middle`);
    if (game) {
      if (stats.distinct < 2) fail("the game client's menu did not draw");
    } else if (!viewer) {
      if (stats.distinct < 8) fail(`only ${stats.distinct} distinct colours in the middle; the checkerboard did not render`);
      if (stats.corner.some((c) => c > 64)) fail(`corner ${stats.corner} is not the dark clear colour`);
    } else if (stats.distinct < 2) {
      fail("the viewer canvas is blank");
    }
  }
  if (failed) console.error(logs.join("\n"));
} finally {
  await browser.close();
  server.close();
}
process.exit(failed ? 1 : 0);
