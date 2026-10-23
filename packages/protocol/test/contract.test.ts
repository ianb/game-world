// Contract test: the wasm build, driven through the wrapper, reproduces the committed
// fixture that crates/characters/tests/town.rs checks natively.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { describe, expect, it } from "vitest";
import { LocalSim, PROTOCOL_VERSION, TICKS_PER_DAY, at, splitSeed, type Emitted, type WasmModule } from "../src/index";

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
    const sim = LocalSim.create(wasm, fixture.seed, content);
    const start = fixture.day * TICKS_PER_DAY;
    const events = sim.stepUntil(start).filter((e) => e.tick >= start);
    events.push(...sim.stepUntil(start + TICKS_PER_DAY));
    expect(events.length).toBe(fixture.events.length);
    expect(events).toStrictEqual(fixture.events);
    expect(sim.hash()).toBe(fixture.hash);
  });

  it("exposes a typed world and villagers, with Option as null", () => {
    const sim = LocalSim.create(wasm, 1, content);
    const world = sim.world();
    expect(world.places.map((p) => p.key)).toStrictEqual(["town", "house/upstairs"]);
    const town = world.places[0]!;
    expect(town.cells.length).toBe(town.width * town.height);
    const asleep = sim.villagers().find((v) => v.key === "mara")!;
    expect(asleep.heading_for).toBeNull();
    expect(asleep.next_tile).toBeNull();
    sim.stepUntil(at(0, 9, 0) + 30);
    const walking = sim.villagers().find((v) => v.key === "mara")!;
    expect(walking.heading_for).toBe("tend-shop");
    expect(walking.next_tile).not.toBeNull();
    sim.stepUntil(at(0, 9, 30));
    const mara = sim.villagers().find((v) => v.key === "mara")!;
    expect(mara.activity).toBe("tend-shop");
    expect(sim.clock().time).toStrictEqual({ hour: 9, minute: 30, second: 0 });
    expect(sim.renderPlace("house/upstairs")).toContain("█████████");
  });

  it("applies inputs from players, actors and the system, and rejects bad ones", () => {
    const sim = LocalSim.create(wasm, 1, content);
    sim.stepUntil(100);
    expect(() => sim.applyInput({ tick: 5, source: "System", kind: "Noop", seq: 0 })).toThrow(/before the current tick/);
    sim.applyInput({ tick: 200, source: { Player: 1 }, kind: { Marker: "hello" }, seq: 0 });
    sim.applyInput({ tick: 200, source: { Actor: "director" }, kind: { Marker: "from an actor" }, seq: 0 });
    sim.applyInput({ tick: 200, source: { Player: 1 }, kind: { Teleport: { who: 1, place: 0, x: 20, y: 20 } }, seq: 0 });
    sim.applyInput({ tick: 200, source: "System", kind: { Teleport: { who: 1, place: 0, x: 4, y: 4 } }, seq: 0 });
    const events = sim.stepUntil(200);
    const applied = events.filter((e) => e.kind === "InputApplied").map((e) => (e as { summary: string }).summary);
    expect(applied).toStrictEqual(['marker "hello"', 'marker "from an actor"']);
    const rejected = events.filter((e) => e.kind === "InputRejected").map((e) => (e as { reason: string }).reason);
    expect(rejected).toHaveLength(2);
    expect(rejected[0]).toMatch(/may not teleport/);
    expect(rejected[1]).toMatch(/not a passable tile/);
  });

  it("round-trips a snapshot through restore and load", () => {
    const live = LocalSim.create(wasm, 9, content);
    live.stepUntil(at(0, 8));
    const bytes = live.snapshot();
    const restored = LocalSim.restore(wasm, bytes, content);
    expect(restored.hash()).toBe(live.hash());
    const a = live.stepUntil(at(0, 20));
    const b = restored.stepUntil(at(0, 20));
    expect(b).toStrictEqual(a);
    const reloaded = LocalSim.create(wasm, 1, content);
    reloaded.load(bytes);
    reloaded.stepUntil(at(0, 20));
    expect(reloaded.hash()).toBe(live.hash());
  });

  it("validates seeds", () => {
    expect(splitSeed(0)).toStrictEqual([0, 0]);
    expect(splitSeed(2 ** 32 + 5)).toStrictEqual([5, 1]);
    expect(() => splitSeed(-1)).toThrow(RangeError);
    expect(() => splitSeed(1.5)).toThrow(RangeError);
    expect(() => splitSeed(2 ** 53)).toThrow(RangeError);
  });
});
