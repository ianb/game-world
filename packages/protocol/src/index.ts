// The only way TypeScript talks to the sim. Wraps the wasm-bindgen handle with the generated
// types, asserts the protocol version at construction, and keeps bytes opaque.
import type { Emitted } from "./generated/Emitted";
import type { Input } from "./generated/Input";
import type { VillagerView } from "./generated/VillagerView";
import type { WorldView } from "./generated/WorldView";
import type { ClockView } from "./generated/ClockView";
import type { TerrainView } from "./generated/TerrainView";
import type { StockView } from "./generated/StockView";
import type { RulesView } from "./generated/RulesView";
import type { DaySummary } from "./generated/DaySummary";
import type { MindView } from "./generated/MindView";

export type { Emitted, Input, VillagerView, WorldView, ClockView, TerrainView, StockView, RulesView, DaySummary, MindView };
export type { RegardView } from "./generated/RegardView";
export type { EpisodeView } from "./generated/EpisodeView";
export type { HeardView } from "./generated/HeardView";
export type { Frame } from "./generated/Frame";
export type { Gesture } from "./generated/Gesture";
export type { Topic } from "./generated/Topic";
export type { Thought } from "./generated/Thought";
export type { ThoughtView } from "./generated/ThoughtView";
export type { Blame } from "./generated/Blame";
export type { Mishap } from "./generated/Mishap";
export type { SummaryItem } from "./generated/SummaryItem";
export type { SummaryKind } from "./generated/SummaryKind";
export type { Cause } from "./generated/Cause";
export type { NeedsView } from "./generated/NeedsView";
export type { Need } from "./generated/Need";
export type { Surface } from "./generated/Surface";
export type { InputKind } from "./generated/InputKind";
export type { Source } from "./generated/Source";
export type { Calendar } from "./generated/Calendar";
export type { Season } from "./generated/Season";
export type { Weekday } from "./generated/Weekday";
export type { TimeOfDay } from "./generated/TimeOfDay";
export type { Cell } from "./generated/Cell";
export type { PlaceKind } from "./generated/PlaceKind";
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
export const PROTOCOL_VERSION = 14;

export const TICKS_PER_MINUTE = 60;
export const TICKS_PER_HOUR = 3600;
export const TICKS_PER_DAY = 86_400;

/** The wasm-bindgen module as this wrapper uses it. `Sim.restore` is a static method. */
export interface WasmModule {
  protocol_version(): number;
  Sim: {
    new (seedLo: number, seedHi: number, contentJson: string): WasmSim;
    restore(bytes: Uint8Array, contentJson: string): WasmSim;
  };
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
  terrain(id: string): unknown;
  economy(): unknown;
  clock(): unknown;
  rules(): unknown;
  summaries(): unknown;
  mind(key: string): unknown;
  frontier(): number;
  rewind(tick: number): void;
  back_to_now(): void;
  free(): void;
}

export class ProtocolMismatch extends Error {
  constructor(found: number) {
    super(`sim protocol version ${found}, wrapper expects ${PROTOCOL_VERSION}`);
    this.name = "ProtocolMismatch";
  }
}

/** The wasm instance trapped (a Rust panic). Its memory may be torn; make a new one. */
export class SimDead extends Error {
  constructor(cause: unknown) {
    super(`sim instance is dead after a trap: ${String(cause)}`);
    this.name = "SimDead";
  }
}

/** The interface the viewer depends on. A remote (server-backed) implementation later
 * satisfies the same shape; only the local one can `load` synchronously. */
export interface Sim {
  readonly tick: number;
  stepUntil(tick: number): Emitted[];
  applyInput(input: Input): void;
  snapshot(): Uint8Array;
  load(bytes: Uint8Array): void;
  hash(): string;
  hashParts(): string[];
  world(): WorldView;
  villagers(): VillagerView[];
  /** The living ground of an outdoor place. Throws for an indoor or unknown place. */
  terrain(id: string): TerrainView;
  /** Goods on objects. */
  economy(): StockView[];
  clock(): ClockView;
  /** The constants behind the needs. */
  rules(): RulesView;
  /** What happened on each finished day, deviations from routine first. */
  summaries(): DaySummary[];
  /** A villager's persona, mood, view of others and recent memory; null for one with no
   * program of their own. Throws for an unknown key. */
  mind(key: string): MindView | null;
  /** The furthest tick ever reached. */
  readonly frontier: number;
  /** Replays the past up to `tick`; inputs are refused until `backToNow`. */
  rewind(tick: number): void;
  backToNow(): void;
  renderPlace(id: string): string;
  whereIsEveryone(): string;
  placeIds(): string[];
}

function assertProtocol(wasm: WasmModule): void {
  const found = wasm.protocol_version();
  if (found !== PROTOCOL_VERSION) throw new ProtocolMismatch(found);
}

/** Splits a non-negative safe integer seed into the two u32 halves the facade takes. */
export function splitSeed(seed: number): [lo: number, hi: number] {
  if (!Number.isSafeInteger(seed) || seed < 0) {
    throw new RangeError(`seed must be a non-negative safe integer, got ${seed}`);
  }
  return [seed % 2 ** 32, Math.floor(seed / 2 ** 32)];
}

export class LocalSim implements Sim {
  private inner: WasmSim;
  private dead: SimDead | null = null;

  private constructor(
    private readonly wasm: WasmModule,
    private readonly contentJson: string,
    inner: WasmSim,
  ) {
    this.inner = inner;
  }

  static create(wasm: WasmModule, seed: number, contentJson: string): LocalSim {
    assertProtocol(wasm);
    const [lo, hi] = splitSeed(seed);
    return new LocalSim(wasm, contentJson, new wasm.Sim(lo, hi, contentJson));
  }

  static restore(wasm: WasmModule, bytes: Uint8Array, contentJson: string): LocalSim {
    assertProtocol(wasm);
    return new LocalSim(wasm, contentJson, wasm.Sim.restore(bytes, contentJson));
  }

  /** Runs a facade call; a wasm trap marks this instance dead for every later call. */
  private guard<T>(f: () => T): T {
    if (this.dead) throw this.dead;
    try {
      return f();
    } catch (e) {
      if (e instanceof WebAssembly.RuntimeError) {
        this.dead = new SimDead(e);
        throw this.dead;
      }
      throw e;
    }
  }

  get tick(): number {
    return this.guard(() => this.inner.tick());
  }
  stepUntil(tick: number): Emitted[] {
    return this.guard(() => this.inner.step_until(tick) as Emitted[]);
  }
  applyInput(input: Input): void {
    this.guard(() => this.inner.apply_input(input));
  }
  snapshot(): Uint8Array {
    return this.guard(() => this.inner.snapshot());
  }
  /** Replaces this instance's state with a snapshot made from the same content. */
  load(bytes: Uint8Array): void {
    const next = this.wasm.Sim.restore(bytes, this.contentJson);
    this.inner.free();
    this.inner = next;
    this.dead = null;
  }
  hash(): string {
    return this.guard(() => this.inner.hash());
  }
  hashParts(): string[] {
    return this.guard(() => this.inner.hash_parts());
  }
  world(): WorldView {
    return this.guard(() => this.inner.world() as WorldView);
  }
  villagers(): VillagerView[] {
    return this.guard(() => this.inner.villagers() as VillagerView[]);
  }
  terrain(id: string): TerrainView {
    return this.guard(() => this.inner.terrain(id) as TerrainView);
  }
  economy(): StockView[] {
    return this.guard(() => this.inner.economy() as StockView[]);
  }
  clock(): ClockView {
    return this.guard(() => this.inner.clock() as ClockView);
  }
  rules(): RulesView {
    return this.guard(() => this.inner.rules() as RulesView);
  }
  summaries(): DaySummary[] {
    return this.guard(() => this.inner.summaries() as DaySummary[]);
  }
  mind(key: string): MindView | null {
    return this.guard(() => (this.inner.mind(key) as MindView | undefined) ?? null);
  }
  get frontier(): number {
    return this.guard(() => this.inner.frontier());
  }
  rewind(tick: number): void {
    this.guard(() => this.inner.rewind(tick));
  }
  backToNow(): void {
    this.guard(() => this.inner.back_to_now());
  }
  renderPlace(id: string): string {
    return this.guard(() => this.inner.render_place(id));
  }
  whereIsEveryone(): string {
    return this.guard(() => this.inner.where_is_everyone());
  }
  placeIds(): string[] {
    return this.guard(() => this.inner.place_ids());
  }
  free(): void {
    this.inner.free();
  }
}

/** Ticks for `hh:mm` on day `day` (0-based). */
export function at(day: number, hour: number, minute = 0): number {
  return day * TICKS_PER_DAY + hour * TICKS_PER_HOUR + minute * TICKS_PER_MINUTE;
}
