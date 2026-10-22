// The soak's wasm client under node (M9 N6): the game client built for
// wasm32 (crates/headless-web), played by our AI as a person. It takes the
// options of `glasnost-headless net-client`, prints the same status lines on
// standard output and writes the same hash log, report and code file; the
// client's own log goes to standard error. Node's WebSocket is the
// browser's API, which the client's web link uses.
//
// Build the package first (docs/development.md, "The soak (N6)"):
//   cargo build --release --target wasm32-unknown-unknown -p glasnost-headless-web
//   wasm-bindgen --target nodejs --out-dir target/headless-web \
//     target/wasm32-unknown-unknown/release/glasnost_headless_web.wasm
// GLASNOST_WASM_PKG names another package folder. Our own harness.

import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkg = process.env.GLASNOST_WASM_PKG ?? path.join(here, "../../../target/headless-web");

// The client's log (console) goes to standard error; standard output
// carries only the status lines.
for (const k of ["log", "info", "debug", "warn", "error", "trace"]) {
  console[k] = (...a) => process.stderr.write(a.map(String).join(" ") + "\n");
}

const args = process.argv.slice(2);
const opt = (name) => {
  const i = args.indexOf(name);
  return i >= 0 && i + 1 < args.length ? args[i + 1] : undefined;
};
const gameDir = args.includes("--synthetic")
  ? undefined
  : opt("--game-dir") ?? process.env.GLASNOST_GAME_DIR;
const hashLog = opt("--hash-log");
const report = opt("--report");
const codeFile = opt("--code-file");

// The install folder's files: names and sizes; archives are read on demand.
const names = [];
const lens = [];
if (gameDir !== undefined) {
  for (const e of fs.readdirSync(gameDir, { withFileTypes: true })) {
    if (!e.isFile()) continue;
    names.push(e.name);
    lens.push(BigInt(fs.statSync(path.join(gameDir, e.name)).size));
  }
  if (!args.includes("--game-dir")) args.push("--game-dir", gameDir);
}
const fds = new Map();
globalThis.glasnostReadAt = (name, offset, len) => {
  let fd = fds.get(name);
  if (fd === undefined) {
    fd = fs.openSync(path.join(gameDir, name), "r");
    fds.set(name, fd);
  }
  const buf = new Uint8Array(len);
  let got = 0;
  while (got < len) {
    const n = fs.readSync(fd, buf, got, len - got, Number(offset) + got);
    if (n === 0) break;
    got += n;
  }
  return got === len ? buf : buf.subarray(0, got);
};
globalThis.glasnostSaveReport = (name, bytes) => {
  try {
    fs.mkdirSync("desync-reports", { recursive: true });
    const p = path.join("desync-reports", name);
    fs.writeFileSync(p, bytes);
    return p;
  } catch (e) {
    process.stderr.write(`cannot store the report: ${e}\n`);
    return "";
  }
};

const require = createRequire(import.meta.url);
const { NetClient } = require(path.join(pkg, "glasnost_headless_web.js"));

const LOBBY_TIMEOUT_MS = 900_000; // our own harness: as the native client
const LINGER_MS = 3_000; // our own harness: as the native client

const t0 = performance.now();
let client;
try {
  client = new NetClient(args, names, lens);
} catch (e) {
  process.stdout.write(`failed ${e.message ?? e}\n`);
  process.exit(1);
}
process.stdout.write(`mounted ${Math.round(performance.now() - t0)} ms\n`);
const clock = performance.now();
const now = () => performance.now() - clock;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let wroteCode = false;

const flushHashes = () => {
  const lines = client.hash_lines();
  if (hashLog !== undefined && lines.length > 0) fs.appendFileSync(hashLog, lines);
};

while (!client.done()) {
  for (const line of client.step(now())) process.stdout.write(line + "\n");
  if (!wroteCode && codeFile !== undefined) {
    const code = client.code();
    if (code !== undefined) {
      fs.writeFileSync(codeFile, code);
      wroteCode = true;
    }
  }
  if (!client.in_game() && now() > LOBBY_TIMEOUT_MS) {
    process.stdout.write(`failed no game after ${LOBBY_TIMEOUT_MS} ms: ${client.status()}\n`);
    process.exit(1);
  }
  flushHashes();
  // Let the socket's events in.
  await sleep(1);
}
flushHashes();
const end = performance.now();
while (performance.now() - end < LINGER_MS && client.networked()) {
  client.linger(now());
  await sleep(2);
}
if (report !== undefined) fs.writeFileSync(report, client.report());
const why = client.failure();
if (why !== undefined) {
  process.stdout.write(`failed ${why}\n`);
  process.exit(1);
}
process.stdout.write(client.summary_line() + "\n");
process.exit(0);
