// Touch and touchpad scrolling in a browser (design: touch and touchpad
// scrolling (lead's request 2026-11-10); our own tooling, local lane).
//
// Serves dist/web, opens the game client with `?mode=game&probe=1` in a
// Chromium page with touch, picks the install folder through the folder
// input, starts a skirmish by keyboard, and then checks through the probe
// (`data-glasnost-view`, `data-glasnost-selected`) that:
//   - a one-finger tap on the base unit selects it;
//   - a two-finger tap on open ground (a right click) gives the selection up;
//   - a wheel event with a pixel delta over the battlefield pans the view
//     with the content (deltaMode 0, as a touchpad swipe);
//   - a one-finger drag on the battlefield pans the view with the finger.
// The touches are CDP `Input.dispatchTouchEvent`. The install stays local: it
// is read from DIR, never copied or committed.
//
// Usage: node touch.mjs --install DIR [--dist ../dist/web] [--out out] [--headed]
// Exit code 0 when every check passes. Writes <out>/touch.png.

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
  console.error("usage: node touch.mjs --install DIR [--dist ../dist/web] [--out out] [--headed]");
  process.exit(2);
}
const dist = path.resolve(here, args.dist ?? "../dist/web");
const outDir = path.resolve(here, args.out ?? "out");
const size = { width: 1280, height: 800 };

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

const browser = await playwright.chromium.launch({
  headless: !args.headed,
  args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"],
});
const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) failures.push(what);
};
try {
  const context = await browser.newContext({ viewport: size, deviceScaleFactor: 1, hasTouch: true });
  const page = await context.newPage();
  page.on("console", (m) => {
    if (/error|panic/i.test(m.text())) console.log(`[console] ${m.text().split("\n")[0]}`);
  });
  page.on("pageerror", (e) => console.log(`pageerror: ${e.message}`));
  const cdp = await context.newCDPSession(page);
  const touches = (type, points) => cdp.send("Input.dispatchTouchEvent", { type, touchPoints: points });
  const data = () => page.evaluate(() => ({ ...document.body.dataset }));
  const view = async () => (await data()).glasnostView.split(",").map(Number);
  const selected = async () => Number((await data()).glasnostSelected);
  // The probe is published with the next drawn frame.
  const settle = () => page.waitForTimeout(400);

  await page.goto(url);
  await page.waitForFunction(() => ["presented", "error"].includes(document.body.dataset.glasnostStatus), null, {
    timeout: 120_000,
  });
  const status = await data();
  if (status.glasnostStatus === "error") throw new Error(`app reported an error: ${status.glasnostDetail}`);
  console.log(`presented with ${status.glasnostBackend}; picking the install folder`);
  await page.locator("#glasnost-folder").setInputFiles(path.resolve(args.install));
  await page.waitForTimeout(Number(args.load ?? 30_000));
  fs.mkdirSync(outDir, { recursive: true });
  await page.screenshot({ path: path.join(outDir, "touch-menu.png") });
  // Skirmish, then Start (as `start_by_keyboard` in the client's tests).
  await page.locator("canvas").focus();
  await page.keyboard.press("Enter");
  await page.keyboard.press("Enter");
  await page.waitForTimeout(2_000);
  await page.screenshot({ path: path.join(outDir, "touch-setup.png") });
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Space");
  await page.waitForFunction(() => (document.body.dataset.glasnostView ?? "") !== "", null, {
    timeout: Number(args.start ?? 300_000),
  });
  await page.waitForTimeout(3_000);
  console.log(`in the game, view ${await view()}`);

  // A one-finger tap on the base unit: the view starts on it, so search
  // outward from the battlefield's middle (the sidebar is on the right).
  const middle = { x: Math.round((size.width - 170) / 2), y: Math.round(size.height / 2) };
  let unit = null;
  search: for (let r = 0; r <= 60; r += 12) {
    for (let dy = -r; dy <= r; dy += 12) {
      for (let dx = -r; dx <= r; dx += 12) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) !== r) continue;
        const p = { x: middle.x + dx, y: middle.y + dy, id: 1 };
        await touches("touchStart", [p]);
        await touches("touchEnd", []);
        await settle();
        if ((await selected()) > 0) {
          unit = p;
          break search;
        }
      }
    }
  }
  check(unit !== null, `a tap selects the base unit${unit ? ` (at ${unit.x},${unit.y})` : ""}`);

  // A two-finger tap on open ground: a right click there gives the selection up.
  if (unit) {
    if (args.trace) {
      await page.evaluate(() => {
        for (const k of ["pointerdown", "pointermove", "pointerup", "pointercancel"]) {
          document.addEventListener(k, (e) =>
            console.log(`trace ${k} ${e.pointerType} ${e.pointerId} ${e.clientX},${e.clientY} ${Math.round(e.timeStamp)}`), true);
        }
      });
      page.on("console", (m) => m.text().startsWith("trace") && console.log(m.text()));
    }
    const g = { x: unit.x - 150, y: unit.y + 120 };
    // Both fingers down in one event and up in one: each CDP call takes up
    // to 150 ms in a software-rendered page, and the tap has 300 ms.
    await touches("touchStart", [{ x: g.x - 20, y: g.y, id: 2 }, { x: g.x + 20, y: g.y, id: 3 }]);
    await touches("touchEnd", []);
    await settle();
    check((await selected()) === 0, "a two-finger tap on open ground gives the selection up");
  }

  // A touchpad swipe: a pixel-mode wheel over the battlefield. deltaX 120
  // (content moving left) moves the view centre right by 120.
  await page.waitForTimeout(600);
  let before = await view();
  await page.mouse.move(middle.x, middle.y);
  await page.mouse.wheel(120, -60);
  await settle();
  let after = await view();
  check(after[0] - before[0] === 120 && after[1] - before[1] === -60,
    `a pixel wheel over the battlefield pans with the content: ${before} -> ${after}`);

  // A one-finger drag: from the middle 80 left and 50 up; the content
  // follows the finger, so the centre moves right 80 and down 50. The
  // canvas is blurred first, as after the folder picker: the touch itself
  // must give the game the focus its pans need.
  await page.evaluate(() => document.querySelector("canvas").blur());
  await page.waitForTimeout(600);
  before = await view();
  const s = { x: middle.x, y: middle.y, id: 4 };
  await touches("touchStart", [s]);
  for (let k = 1; k <= 8; k++) {
    await touches("touchMove", [{ x: s.x - 10 * k, y: s.y - Math.round(6.25 * k), id: 4 }]);
  }
  await touches("touchEnd", []);
  await settle();
  after = await view();
  check(after[0] - before[0] === 80 && after[1] - before[1] === 50,
    `a one-finger drag pans with the finger: ${before} -> ${after}`);
  check((await selected()) === 0, "the drag clicked nothing");

  fs.mkdirSync(outDir, { recursive: true });
  await page.screenshot({ path: path.join(outDir, "touch.png") });
} catch (e) {
  failures.push(e.message);
  console.error(`FAIL: ${e.message}`);
  fs.mkdirSync(outDir, { recursive: true });
  for (const p of browser.contexts().flatMap((c) => c.pages())) {
    await p.screenshot({ path: path.join(outDir, "touch-fail.png") }).catch(() => {});
  }
} finally {
  await browser.close();
  server.close();
}
console.log(failures.length === 0 ? "touch: all checks passed" : `touch: ${failures.length} failed`);
process.exit(failures.length === 0 ? 0 : 1);
