// Contract test: the wasm build, driven through the wrapper, reproduces the committed
// fixture that crates/sim/tests/fixtures.rs checks natively.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { describe, expect, it } from "vitest";
import { LocalSim, PROTOCOL_VERSION, TICKS_PER_DAY, at, type Emitted, type WasmModule } from "../src/index";

const require = createRequire(import.meta.url);
const wasm = require("@game-world/sim-wasm/node") as WasmModule;
const root = new URL("../../../", import.meta.url);
const content = readFileSync(new URL("target/content-bundle.json", root), "utf8");

interface Fixture {
  seed: number;
  day: number;
  protocol_version: number;
  hash: string;
  events: Emitted[];
}
const fixture = JSON.parse(readFileSync(new URL("fixtures/day0-seed1.json", root), "utf8")) as Fixture;

describe("wasm sim against the native fixture", () => {
  it("has the protocol version the wrapper expects", () => {
    expect(wasm.protocol_version()).toBe(PROTOCOL_VERSION);
    expect(fixture.protocol_version).toBe(PROTOCOL_VERSION);
  });

  it("emits the same day-0 events and ends on the same hash", () => {
    const sim = new LocalSim(wasm, fixture.seed, content);
    sim.stepUntil(fixture.day * TICKS_PER_DAY);
    const events = sim.stepUntil((fixture.day + 1) * TICKS_PER_DAY);
    expect(events.length).toBe(fixture.events.length);
    expect(events).toEqual(fixture.events);
    expect(sim.hash()).toBe(fixture.hash);
  });

  it("exposes a typed world and villagers", () => {
    const sim = new LocalSim(wasm, 1, content);
    const world = sim.world();
    expect(world.places.map((p) => p.key)).toEqual(["town", "house/upstairs"]);
    const town = world.places[0]!;
    expect(town.cells.length).toBe(town.width * town.height);
    sim.stepUntil(at(0, 9, 30));
    const mara = sim.villagers().find((v) => v.key === "mara")!;
    expect(mara.activity).toBe("tend-shop");
    expect(sim.clock().time).toEqual({ hour: 9, minute: 30, second: 0 });
    expect(sim.renderPlace("house/upstairs")).toContain("#########");
  });

  it("rejects inputs in the past and applies markers", () => {
    const sim = new LocalSim(wasm, 1, content);
    sim.stepUntil(100);
    expect(() => sim.applyInput({ tick: 5, source: "System", kind: "Noop" })).toThrow();
    sim.applyInput({ tick: 200, source: { Player: 1 }, kind: { Marker: "hello" } });
    const events = sim.stepUntil(200);
    expect(events.some((e) => e.kind === "InputApplied" && e.summary.includes("hello"))).toBe(true);
  });
});
