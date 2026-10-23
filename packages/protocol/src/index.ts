// The only way TypeScript talks to the sim. Wraps the wasm-bindgen handle with the generated
// types, asserts the protocol version at construction, and keeps bytes opaque.
import type { Emitted } from "./generated/Emitted";
import type { Input } from "./generated/Input";
import type { VillagerView } from "./generated/VillagerView";
import type { WorldView } from "./generated/WorldView";
import type { ClockView } from "./generated/ClockView";

export type { Emitted, Input, VillagerView, WorldView, ClockView };
export type { InputKind } from "./generated/InputKind";
export type { Source } from "./generated/Source";
export type { Calendar } from "./generated/Calendar";
export type { Cell } from "./generated/Cell";
export type { PlaceView } from "./generated/PlaceView";
export type { ObjectView } from "./generated/ObjectView";
export type { DoorView } from "./generated/DoorView";
export type { RoofView } from "./generated/RoofView";
export type { Pos } from "./generated/Pos";
export type { Rect } from "./generated/Rect";
export type { Tick } from "./generated/Tick";
export type { PlaceId } from "./generated/PlaceId";
export type { VillagerId } from "./generated/VillagerId";

/** Must equal `PROTOCOL_VERSION` in crates/sim/src/protocol.rs. */
export const PROTOCOL_VERSION = 3;

export const TICKS_PER_MINUTE = 60;
export const TICKS_PER_HOUR = 3600;
export const TICKS_PER_DAY = 86_400;

/** The subset of the wasm-bindgen module this wrapper needs. */
export interface WasmModule {
  protocol_version(): number;
  Sim: new (seedLo: number, seedHi: number, contentJson: string) => WasmSim;
  // Static method on the class is exposed as a plain function by wasm-bindgen.
  // Called as `Sim.restore` in the bundler/nodejs output.
}

export interface WasmSim {
  tick(): number;
  step_until(tick: number): unknown;
  apply_input(input: unknown): void;
  snapshot(): Uint8Array;
  hash(): string;
  hash_parts(): string[];
  place_ids(): string[];
  render_place(id: string): string;
  where_is_everyone(): string;
  world(): unknown;
  villagers(): unknown;
  clock(): unknown;
  free(): void;
}

export class ProtocolMismatch extends Error {
  constructor(found: number) {
    super(`sim protocol version ${found}, wrapper expects ${PROTOCOL_VERSION}`);
  }
}

/** The interface the viewer depends on. A remote (server-backed) implementation later
 * satisfies the same shape. */
export interface Sim {
  readonly tick: number;
  stepUntil(tick: number): Emitted[];
  applyInput(input: Input): void;
  snapshot(): Uint8Array;
  hash(): string;
  hashParts(): string[];
  world(): WorldView;
  villagers(): VillagerView[];
  clock(): ClockView;
  renderPlace(id: string): string;
  whereIsEveryone(): string;
  placeIds(): string[];
}

export class LocalSim implements Sim {
  private readonly inner: WasmSim;

  constructor(wasm: WasmModule, seed: number, contentJson: string) {
    const found = wasm.protocol_version();
    if (found !== PROTOCOL_VERSION) throw new ProtocolMismatch(found);
    const seedLo = seed >>> 0;
    const seedHi = Math.floor(seed / 2 ** 32) >>> 0;
    this.inner = new wasm.Sim(seedLo, seedHi, contentJson);
  }

  get tick(): number {
    return this.inner.tick();
  }
  stepUntil(tick: number): Emitted[] {
    return this.inner.step_until(tick) as Emitted[];
  }
  applyInput(input: Input): void {
    this.inner.apply_input(input);
  }
  snapshot(): Uint8Array {
    return this.inner.snapshot();
  }
  hash(): string {
    return this.inner.hash();
  }
  hashParts(): string[] {
    return this.inner.hash_parts();
  }
  world(): WorldView {
    return this.inner.world() as WorldView;
  }
  villagers(): VillagerView[] {
    return this.inner.villagers() as VillagerView[];
  }
  clock(): ClockView {
    return this.inner.clock() as ClockView;
  }
  renderPlace(id: string): string {
    return this.inner.render_place(id);
  }
  whereIsEveryone(): string {
    return this.inner.where_is_everyone();
  }
  placeIds(): string[] {
    return this.inner.place_ids();
  }
  free(): void {
    this.inner.free();
  }
}

/** Ticks for `hh:mm` on day `day` (0-based). */
export function at(day: number, hour: number, minute = 0): number {
  return day * TICKS_PER_DAY + hour * TICKS_PER_HOUR + minute * TICKS_PER_MINUTE;
}
