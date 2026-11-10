// The display scale in a browser at device pixel ratio 2 (design: readable
// text and display scale (lead's request 2026-11-10); our own tooling, local
// lane).
//
// Serves dist/web, opens the game client with `?mode=game&probe=1` in a
// Chromium window of 1280 x 800 CSS pixels at a device scale factor of 2
// (`--force-device-scale-factor`, a real one: under Playwright's emulated
// deviceScaleFactor headless Chromium reports the canvas's device-pixel box
// at its CSS size), picks the install folder through the folder input,
// starts a skirmish by keyboard, and checks through the probe
// (`data-glasnost-display`, `data-glasnost-view`, `data-glasnost-selected`)
// that:
//   - the canvas's backing store is the CSS size times the ratio, and the
//     client sees that size, a factor of 2 (the Auto setting) and a view of
//     the CSS size (backing / factor);
//   - a mouse click near the battlefield's middle in CSS pixels selects the
//     base unit (the view starts on it), and does so again after the
//     selection is given up: the click landed on the object, not at twice
//     its place;
//   - a pixel wheel of 120 CSS pixels over the battlefield pans the view by
//     120 view pixels (one CSS pixel is one view pixel at 2x);
//   - when the ratio changes to 1 at run time with a new CSS size (as a zoom
//     does; emulated, so the backing keeps 2 device pixels per CSS pixel),
//     the client follows: a factor of 1 and a view of the new backing size.
// The install stays local: it is read from DIR, never copied or committed.
//
// Usage: node hidpi.mjs --install DIR [--dist ../dist/web] [--out out] [--headed]
// Exit code 0 when every check passes. Writes <out>/hidpi-*.png.

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
  console.error("usage: node hidpi.mjs --install DIR [--dist ../dist/web] [--out out] [--headed]");
  process.exit(2);
}
const dist = path.resolve(here, args.dist ?? "../dist/web");
const outDir = path.resolve(here, args.out ?? "out");
const size = { width: 1280, height: 800 };
const ratio = 2;

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
const url = `http://127.0.0.1:${server.address().port}/?backend=webgl2&mode=game&probe=1`;

// A real device scale factor, not Playwright's emulated one: under
// emulation headless Chromium reports the canvas's device-pixel box at its
// CSS size, which no real 2x screen does.
const browser = await playwright.chromium.launch({
  headless: !args.headed,
  args: [
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    "--ignore-gpu-blocklist",
    `--force-device-scale-factor=${ratio}`,
    `--window-size=${size.width},${size.height}`,
  ],
});
const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures.push(what);
};
try {
  const context = await browser.newContext({ viewport: null });
  const page = await context.newPage();
  page.on("console", (m) => {
    if (/error|panic/i.test(m.text())) console.log(`[console] ${m.text().split("\n")[0]}`);
  });
  page.on("pageerror", (e) => console.log(`pageerror: ${e.message}`));
  const data = () => page.evaluate(() => ({ ...document.body.dataset }));
  const view = async () => (await data()).glasnostView.split(",").map(Number);
  const selected = async () => Number((await data()).glasnostSelected);
  // `<physical w>x<physical h> <factor> <view w>x<view h> <host scale>`.
  const display = async () => {
    const [physical, factor, v, scale] = ((await data()).glasnostDisplay ?? "").split(" ");
    const pair = (s) => (s ?? "0x0").split("x").map(Number);
    return { physical: pair(physical), factor: Number(factor), view: pair(v), scale: Number(scale) };
  };
  const canvas = () =>
    page.evaluate(() => {
      const c = document.querySelector("canvas");
      return { w: c.width, h: c.height, cssW: c.clientWidth, cssH: c.clientHeight, dpr: devicePixelRatio };
    });
  // What the browser's resize observer reports for the canvas in device
  // pixels (the size the window layer is given).
  const observed = () =>
    page.evaluate(
      () =>
        new Promise((resolve) => {
          const c = document.querySelector("canvas");
          const o = new ResizeObserver((entries) => {
            o.disconnect();
            const s = entries[0].devicePixelContentBoxSize?.[0];
            resolve(s ? `${s.inlineSize}x${s.blockSize}` : "unsupported");
          });
          o.observe(c, { box: "device-pixel-content-box" });
        }),
    );
  // The probe is published with the next drawn frame.
  const settle = () => page.waitForTimeout(400);

  await page.goto(url);
  await page.waitForFunction(() => ["presented", "error"].includes(document.body.dataset.glasnostStatus), null, {
    timeout: 120_000,
  });
  const status = await data();
  if (status.glasnostStatus === "error") throw new Error(`app reported an error: ${status.glasnostDetail}`);
  console.log(`presented with ${status.glasnostBackend}`);
  await settle();

  // The backing store, the client's window and its view.
  const c = await canvas();
  const d = await display();
  console.log(`canvas ${c.w}x${c.h} for ${c.cssW}x${c.cssH} CSS at ${c.dpr}; device-pixel box ${await observed()}; display ${JSON.stringify(d)}`);
  check(c.dpr === ratio && c.w === c.cssW * ratio && c.h === c.cssH * ratio,
    `the canvas's backing store is its CSS size times ${ratio}`);
  check(d.physical[0] === c.w && d.physical[1] === c.h, "the client's window is the backing store");
  check(d.scale === ratio && d.factor === ratio, `the factor is ${ratio} under Auto`);
  check(d.view[0] === Math.floor(c.w / d.factor) && d.view[1] === Math.floor(c.h / d.factor)
    && d.view[0] === c.cssW && d.view[1] === c.cssH, "the view is the backing store over the factor: the CSS size");

  console.log("picking the install folder");
  await page.locator("#glasnost-folder").setInputFiles(path.resolve(args.install));
  await page.waitForTimeout(Number(args.load ?? 30_000));
  fs.mkdirSync(outDir, { recursive: true });
  await page.screenshot({ path: path.join(outDir, "hidpi-menu.png") });
  await page.locator("canvas").focus();
  await page.keyboard.press("Enter");
  await page.keyboard.press("Enter");
  await page.waitForTimeout(2_000);
  await page.screenshot({ path: path.join(outDir, "hidpi-setup.png") });
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Space");
  await page.waitForFunction(() => (document.body.dataset.glasnostView ?? "") !== "", null, {
    timeout: Number(args.start ?? 300_000),
  });
  await page.waitForTimeout(3_000);
  console.log(`in the game, view ${await view()}`);

  // A mouse click on the base unit: the view starts on it, so search
  // outward from the battlefield's middle (the sidebar is on the right) in
  // CSS pixels. Were the click taken as view pixels at twice its place, it
  // would land 555 pixels to the right and 400 down, off the unit.
  const middle = { x: Math.round((c.cssW - 170) / 2), y: Math.round(c.cssH / 2) };
  let unit = null;
  search: for (let r = 0; r <= 60; r += 12) {
    for (let dy = -r; dy <= r; dy += 12) {
      for (let dx = -r; dx <= r; dx += 12) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) !== r) continue;
        const p = { x: middle.x + dx, y: middle.y + dy };
        await page.mouse.click(p.x, p.y);
        await settle();
        if ((await selected()) > 0) {
          unit = p;
          break search;
        }
      }
    }
  }
  check(unit !== null, `a click near the battlefield's middle selects the base unit${unit ? ` (at ${unit.x},${unit.y})` : ""}`);
  if (unit) {
    // A right click on open ground gives the selection up; the same click
    // as before selects the unit again.
    console.log(`view ${await view()}`);
    await page.mouse.click(unit.x - 150, unit.y + 120, { button: "right" });
    await settle();
    check((await selected()) === 0, "a right click on open ground gives the selection up");
    console.log(`view ${await view()}`);
    await page.mouse.click(unit.x, unit.y);
    await settle();
    check((await selected()) > 0, `the same click selects the unit again (view ${await view()})`);
    await page.mouse.click(unit.x - 150, unit.y + 120, { button: "right" });
    await settle();
    console.log(`view ${await view()}`);
  }
  await page.screenshot({ path: path.join(outDir, "hidpi-game.png") });

  // A touchpad swipe of 120 CSS pixels pans by 120 view pixels.
  await page.waitForTimeout(600);
  const before = await view();
  await page.mouse.move(middle.x, middle.y);
  await page.mouse.wheel(120, -60);
  await settle();
  const after = await view();
  check(after[0] - before[0] === 120 && after[1] - before[1] === -60,
    `a pixel wheel of (120, -60) CSS pixels pans by (120, -60) view pixels: ${before} -> ${after}`);

  // The ratio changes to 1 at run time with a new CSS size, as a zoom or a
  // move to another monitor does: the client follows. (Under emulation the
  // ratio alone does not reach the page's resolution media query, so the
  // size changes with it, as it does for a real zoom.)
  const cdp = await context.newCDPSession(page);
  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width: c.cssW - 80,
    height: c.cssH - 40,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await page.waitForTimeout(1_500);
  const c1 = await canvas();
  const d1 = await display();
  const mq = await page.evaluate(() => [1, 2].map((r) => `${r}dppx ${matchMedia(`(resolution: ${r}dppx)`).matches}`).join(", "));
  console.log(`after the change: canvas ${c1.w}x${c1.h} at ${c1.dpr} (${mq}); display ${JSON.stringify(d1)}`);
  check(d1.factor === 1 && d1.physical[0] === c1.w && d1.view[0] === c1.w && d1.view[1] === c1.h,
    "after a change to ratio 1 the factor is 1 and the view is the backing store");
  await page.screenshot({ path: path.join(outDir, "hidpi-ratio-1.png") });
} catch (e) {
  failures.push(e.message);
  console.error(`FAIL: ${e.message}`);
  fs.mkdirSync(outDir, { recursive: true });
  for (const p of browser.contexts().flatMap((c) => c.pages())) {
    await p.screenshot({ path: path.join(outDir, "hidpi-fail.png") }).catch(() => {});
  }
} finally {
  await browser.close();
  server.close();
}
console.log(failures.length === 0 ? "hidpi: all checks passed" : `hidpi: ${failures.length} failed`);
process.exit(failures.length === 0 ? 0 : 1);
