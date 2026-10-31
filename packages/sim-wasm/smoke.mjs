// Runs the wasm build of the sim in Node: a week of town life, then checks the hash against
// the native golden. Usage: node packages/sim-wasm/smoke.mjs <world> <content.json> <golden-hash>
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const wasm = require("./pkg-node/sim_wasm.js");

const [world, contentPath, golden] = process.argv.slice(2);
if (!/^[0-9a-f]{16}$/.test(golden ?? "")) {
  console.error(`expected a 16-hex golden hash as the second argument, got ${JSON.stringify(golden)}`);
  process.exit(2);
}
const content = readFileSync(contentPath, "utf8");
const TICKS_PER_DAY = 86_400;

console.log(`protocol v${wasm.protocol_version()}`);
const sim = new wasm.Sim(world, 2026, 0, content);
sim.apply_input({ tick: 3 * TICKS_PER_DAY + 12 * 3600, source: { Player: 0 }, kind: { Marker: "midweek" }, seq: 0 });
const events = sim.step_until(3 * TICKS_PER_DAY + 14 * 3600);
console.log(`${events.length} events by day 4 14:00; last: ${JSON.stringify(events.at(-1))}`);
console.log(sim.where_is_everyone());
console.log(sim.render_place("house/upstairs"));
sim.step_until(7 * TICKS_PER_DAY);
const hash = sim.hash();
console.log(`hash after a week: ${hash} (parts ${sim.hash_parts().join(" ")})`);
if (hash !== golden) {
  console.error(`MISMATCH: native golden is ${golden}`);
  process.exit(1);
}
console.log("ok");
