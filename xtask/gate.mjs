// Runs the determinism gate's wasm32 module under node and compares its
// output with the committed golden file (called by `cargo xtask gate`).
// usage: node xtask/gate.mjs <glasnost_sim_gate.wasm> <scenario index> <golden file>
import { readFileSync } from "node:fs";

const [wasmPath, scenario, goldenPath] = process.argv.slice(2);
const golden = readFileSync(goldenPath, "utf8");
const ticks = golden.split("\n").filter((l) => l && !l.startsWith("#")).length;
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const len = instance.exports.gate_run(Number(scenario), ticks);
const ptr = instance.exports.gate_ptr();
const got = new TextDecoder().decode(new Uint8Array(instance.exports.memory.buffer, ptr, len));
if (got === golden) {
  console.log(`wasm32 (${goldenPath}): ${ticks} ticks match the golden hashes`);
} else {
  const a = got.split("\n");
  const b = golden.split("\n");
  const i = a.findIndex((l, k) => l !== b[k]);
  console.error(`wasm32 (${goldenPath}): line ${i + 1} differs (tick ${i - 1}): got ${a[i]}, want ${b[i]}`);
  process.exit(1);
}
