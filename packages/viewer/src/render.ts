// The tile renderer: one canvas per place, drawn from the sim's views with the LPC tileset
// in public/tiles.png (built by scripts/make-tileset.mjs). Pixel art at native size, scaled
// by CSS at an integer factor. The text views stay the source of truth for tests; this is
// for eyes.
import type { AnimalView, Cell, ObjectView, PlaceView, RoofView, Sim, Surface, TrainView, VillagerView, Walling, WorldView } from "@game-world/protocol";
import { type Dressing, SOUTHERN_WOODS, dressingFor, paintDressing, treeAt } from "./dressing";

interface AtlasTile {
  col: number;
  row: number;
  /** Size in tiles, for sprites bigger than one. */
  w?: number;
  h?: number;
  /** Where the sprite sits relative to its cell, in pixels. */
  ox?: number;
  oy?: number;
  /** Plain variants in consecutive columns, picked by position. */
  variants?: number;
}

interface Atlas {
  tile: number;
  columns: number;
  tiles: Record<string, AtlasTile>;
  /** Each villager's walk sheet: `frames` frames of `frame` px; rows face up, left, down,
   * right; at `left`, `top` in the atlas. */
  villagers: { frame: number; frames: number; top: Record<string, number>; left?: Record<string, number> };
}

export class Tileset {
  private constructor(
    readonly image: HTMLImageElement,
    readonly atlas: Atlas,
  ) {}

  static async load(base: string): Promise<Tileset> {
    const atlas = (await (await fetch(new URL("tiles.json", base))).json()) as Atlas;
    const image = new Image();
    image.src = new URL("tiles.png", base).href;
    await image.decode();
    return new Tileset(image, atlas);
  }

  get size(): number {
    return this.atlas.tile;
  }

  has(name: string): boolean {
    return name in this.atlas.tiles;
  }

  /** Draws a tile or sprite at a cell. A sprite taller than a tile with no set offset
   * hangs down from its cell, or rises from it when `rise` (there is no room below). */
  draw(ctx: CanvasRenderingContext2D, name: string, x: number, y: number, rise = false): void {
    const t = this.atlas.tiles[name] ?? this.atlas.tiles["obj_generic"];
    if (!t) return;
    const s = this.atlas.tile;
    const col = t.variants ? t.col + variantAt(x, y, t.variants) : t.col;
    const w = (t.w ?? 1) * s;
    const h = (t.h ?? 1) * s;
    const oy = t.oy ?? (rise ? s - h : 0);
    ctx.drawImage(this.image, col * s, t.row * s, w, h, x * s + (t.ox ?? 0), y * s + oy, w, h);
  }

  /** The cells a sprite drawn at (x, y) covers, as an inclusive tile rect. */
  covers(name: string, x: number, y: number): { x0: number; y0: number; x1: number; y1: number } {
    const t = this.atlas.tiles[name] ?? this.atlas.tiles["obj_generic"];
    const s = this.atlas.tile;
    if (!t) return { x0: x, y0: y, x1: x, y1: y };
    const px = x * s + (t.ox ?? 0);
    const py = y * s + (t.oy ?? 0);
    const w = (t.w ?? 1) * s;
    const h = (t.h ?? 1) * s;
    return { x0: Math.floor(px / s), y0: Math.floor(py / s), x1: Math.floor((px + w - 1) / s), y1: Math.floor((py + h - 1) / s) };
  }

  /** Draws a tile at a fractional position, for things that slide between cells. */
  drawAt(ctx: CanvasRenderingContext2D, name: string, x: number, y: number): void {
    const t = this.atlas.tiles[name];
    if (!t) return;
    const s = this.atlas.tile;
    const w = (t.w ?? 1) * s;
    const h = (t.h ?? 1) * s;
    ctx.drawImage(this.image, t.col * s, t.row * s, w, h, Math.round(x * s + (t.ox ?? 0)), Math.round(y * s + (t.oy ?? 0)), w, h);
  }

  /** An animal's walking frame: four facings by four frames. Chickens fill a tile; sheep
   * are two tiles wide, centred on theirs and standing on its lower edge. */
  drawAnimal(ctx: CanvasRenderingContext2D, a: AnimalView, facing: Facing, frame: number): void {
    const t = this.atlas.tiles[`animal_${a.kind}`];
    if (!t) return;
    const s = this.atlas.tile;
    const size = a.kind === "sheep" ? 2 * s : s;
    const row = { up: 0, left: 1, down: 2, right: 3 }[facing];
    const col = frame === 0 ? 0 : frame % 4;
    const dx = a.pos.x * s + (s - size) / 2;
    const dy = (a.pos.y + 1) * s - size + (a.kind === "sheep" ? 8 : 0);
    ctx.drawImage(this.image, t.col * s + col * size, t.row * s + row * size, size, size, dx, dy, size, size);
  }

  /** Draws one quadrant (qx, qy in 0..1) of a shape tile into the same quadrant of (x, y). */
  drawQuadrant(ctx: CanvasRenderingContext2D, name: string, qx: number, qy: number, x: number, y: number): void {
    const t = this.atlas.tiles[name];
    if (!t) return;
    const s = this.atlas.tile;
    const h = s / 2;
    ctx.drawImage(this.image, t.col * s + qx * h, t.row * s + qy * h, h, h, x * s + qx * h, y * s + qy * h, h, h);
  }

  /** A villager standing (frame 0) or mid-step, feet on the bottom of their cell, shifted
   * `dx` pixels when sharing the cell. */
  drawVillager(ctx: CanvasRenderingContext2D, key: string, facing: Facing, frame: number, x: number, y: number, dx = 0): void {
    const v = this.atlas.villagers;
    const top = v.top[key] ?? Object.values(v.top)[0];
    if (top === undefined) return;
    const s = this.atlas.tile;
    const f = v.frame;
    const sx = (v.left?.[key] ?? 0) + (frame % v.frames) * f;
    const sy = top + FACING_ROW[facing] * f;
    ctx.drawImage(this.image, sx, sy, f, f, x * s + s / 2 - f / 2 + dx, (y + 1) * s - f + FEET, f, f);
  }
}

/** People on the same cell (at a counter, say) stand side by side: pixel shifts by key. */
function spread(villagers: VillagerView[], tile: number): Map<string, number> {
  const cells = new Map<string, VillagerView[]>();
  for (const v of villagers) {
    if (v.activity === "sleep") continue;
    const k = `${v.pos.x},${v.pos.y}`;
    cells.set(k, [...(cells.get(k) ?? []), v]);
  }
  const shift = new Map<string, number>();
  const step = tile * 0.6;
  for (const group of cells.values()) {
    group.sort((a, b) => (a.key < b.key ? -1 : 1));
    group.forEach((v, i) => shift.set(v.key, (i - (group.length - 1) / 2) * step));
  }
  return shift;
}

/** Someone talking faces the nearest other person who is talking. */
function facePartner(v: VillagerView, here: VillagerView[]): Facing | null {
  let best: VillagerView | null = null;
  let bestD = Infinity;
  for (const o of here) {
    if (o.key === v.key || o.activity !== "talk") continue;
    const d = Math.max(Math.abs(o.pos.x - v.pos.x), Math.abs(o.pos.y - v.pos.y));
    if (d < bestD) {
      best = o;
      bestD = d;
    }
  }
  if (!best || bestD > 3) return null;
  const dx = best.pos.x - v.pos.x;
  const dy = best.pos.y - v.pos.y;
  // Side by side on one cell: the one on the left faces right.
  if (dx === 0 && dy === 0) return v.key < best.key ? "right" : "left";
  if (Math.abs(dx) >= Math.abs(dy)) return dx > 0 ? "right" : "left";
  return dy > 0 ? "down" : "up";
}

/** How often the still layer (ground, trees, furniture) is repainted, in milliseconds. */
const LAYER_MS = 3000;

/** LPC frames leave a little room under the feet. */
const FEET = 2;

export type Facing = "up" | "left" | "down" | "right";
const FACING_ROW: Record<Facing, number> = { up: 0, left: 1, down: 2, right: 3 };

/** A stable pseudo-random pick for a cell, so the ground does not repeat in a grid. */
export function variantAt(x: number, y: number, n: number): number {
  let h = Math.imul(x, 374761393) + Math.imul(y, 668265263);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) % n;
}

/** Features drawn over a background with auto-shaped edges, and which of them join up. */
const FEATURE_GROUP: Record<string, string> = {
  water: "water",
  canopy: "canopy",
  path: "path",
  weedy_path: "path",
  overgrown_path: "path",
  track: "path",
  cowpath: "path",
  tall_grass: "grass",
  brush: "grass",
  wall: "wall",
  door: "wall",
};

/** Groups whose features run off the edge of the map, like the border forest. Walls stop. */
const RUNS_OFF_MAP = new Set(["water", "canopy", "path", "grass"]);

type Shape = "full" | "h" | "v" | "out" | "in";

/** The shape of one quadrant given whether the feature continues to its two cardinal
 * neighbours and its diagonal. */
function quadrantShape(side: boolean, vertical: boolean, diagonal: boolean): Shape {
  if (side && vertical) return diagonal ? "full" : "in";
  if (vertical) return "v";
  if (side) return "h";
  return "out";
}

const CELL_TILE: Record<Cell, string> = {
  Ground: "grass",
  Path: "path",
  Blocked: "canopy",
  Water: "water",
  Wall: "wall",
  Floor: "floor",
  Opening: "opening",
  Door: "door",
  Void: "void",
};

const SURFACE_TILE: Record<Surface, string> = {
  Grass: "grass",
  TallGrass: "tall_grass",
  Brush: "brush",
  Track: "track",
  Cowpath: "cowpath",
  Path: "path",
  WeedyPath: "weedy_path",
  OvergrownPath: "overgrown_path",
};

/** What a wall shows from the south: the room's panelling, the street side, or its top. */
function wallFace(below: string | undefined, walls: Walling): string | null {
  if (below === "floor" || below === "opening") return `wall_face_${walls}`;
  if (below === undefined || below === "void" || FEATURE_GROUP[below] === "wall") return null;
  return `wall_outside_${walls}`;
}

/** Object kinds with a sprite of their own; anything else is drawn as a crate. */
const OBJECT_TILES = new Set([
  "bed", "table", "counter", "oven", "stove", "shelf", "pantry", "barrel", "cart", "privy", "well", "bench", "chair",
  "anvil", "sacks", "millstone", "altar", "workbench", "desk", "loom", "lectern", "bookshelf", "chest", "weaponrack",
  "clock", "hearth", "barrels", "haystack", "firewood", "logs", "bucket", "trough", "churn", "wheel", "grave", "cross",
  "scarecrow", "banner", "lantern", "stall", "hive", "clamp", "fence", "plant", "coop", "crates", "platform",
]);

/** Kinds drawn with another kind's sprite. */
const OBJECT_ALIASES: Record<string, string> = {
  woodpile: "firewood",
  block: "logs",
  kiln: "hearth",
  washtub: "trough",
  pen: "fence",
  strongbox: "chest",
  jars: "sacks",
  copper: "hearth",
};

export function objectTile(kind: string): string {
  const k = OBJECT_ALIASES[kind] ?? kind;
  return OBJECT_TILES.has(k) ? `obj_${k}` : "obj_generic";
}

/** The sprite for an object as placed among its neighbours: a bed for two is drawn once,
 * from its left half; the right half draws nothing. */
export function objectTileAt(o: ObjectView, all: ObjectView[]): string | null {
  if (o.kind !== "bed_double") return objectTile(o.kind);
  const left = all.some((p) => p.kind === "bed_double" && p.place === o.place && p.pos.y === o.pos.y && p.pos.x === o.pos.x - 1);
  return left ? null : "obj_bed_double";
}

/** Where each car of a train is drawn, in tile units, engine first. The track runs left to
 * right; a train going toward its end faces right. */
export function trainCars(t: TrainView, track: { x: number; y: number }[]): { tile: string; x: number; y: number }[] {
  const first = track[0];
  if (!first) return [];
  const cars = ["engine", "wagon", "wagon", "van"].slice(0, Math.max(1, Math.floor(t.length / 2)));
  const head = t.head / 16;
  return cars.map((kind, k) => {
    // Each car is two tiles; the engine's front is at the head.
    const front = t.toward_end ? head - 2 * k : head + 2 * k;
    const left = t.toward_end ? front - 2 : front;
    const tile = kind === "engine" ? (t.toward_end ? "train_engine_e" : "train_engine_w") : kind === "van" ? "train_van" : "train_wagon";
    return { tile, x: first.x + left, y: first.y };
  });
}

/** What each cell of a place is made of: the floor tile of the room it is in, and the
 * material of the building whose wall it is. */
export interface Materials {
  floor: (string | undefined)[];
  walls: (Walling | undefined)[];
}

export function materials(place: PlaceView, rooms: WorldView["rooms"], roofs: RoofView[]): Materials {
  const w = place.width;
  const n = w * place.height;
  const floor = new Array<string | undefined>(n).fill(undefined);
  const walls = new Array<Walling | undefined>(n).fill(undefined);
  const byBuilding = new Map(roofs.map((r) => [r.building, r.walls]));
  const inside = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < w && y < place.height;
  for (const r of rooms) {
    if (r.place !== place.id) continue;
    const material = byBuilding.get(r.building) ?? "timber";
    // The room's own cells, and its walls one tile out.
    for (let y = r.rect.y - 1; y <= r.rect.y + r.rect.h; y++) {
      for (let x = r.rect.x - 1; x <= r.rect.x + r.rect.w; x++) {
        if (!inside(x, y)) continue;
        const i = y * w + x;
        const edge = x < r.rect.x || y < r.rect.y || x >= r.rect.x + r.rect.w || y >= r.rect.y + r.rect.h;
        if (!edge) floor[i] = `floor_${r.flooring}`;
        else walls[i] ??= material;
      }
    }
  }
  // Openings and doors take the floor of a room beside them.
  for (let i = 0; i < n; i++) {
    const c = place.cells[i];
    if (floor[i] || (c !== "Opening" && c !== "Door")) continue;
    const x = i % w, y = Math.floor(i / w);
    for (const [dx, dy] of [[0, 1], [0, -1], [1, 0], [-1, 0]] as const) {
      if (inside(x + dx, y + dy) && floor[(y + dy) * w + x + dx]) {
        floor[i] = floor[(y + dy) * w + x + dx];
        break;
      }
    }
  }
  return { floor, walls };
}

/** The tile name for every cell of a place: its living ground outdoors, else its cells. */
export function groundNames(sim: Sim, place: PlaceView): string[] {
  const terrain = place.kind === "Zone" ? sim.terrain(place.key).tiles : null;
  const names: string[] = new Array<string>(place.width * place.height);
  for (let i = 0; i < names.length; i++) {
    const surface = terrain?.[i] ?? null;
    names[i] = surface ? SURFACE_TILE[surface] : CELL_TILE[place.cells[i] ?? "Void"];
  }
  return names;
}

/** Paints a place's ground: surfaces, water, paths and walls, auto-shaped at their edges.
 * With `flatWalls` a wall shows only its top and a door is open floor, for a view that
 * raises the walls itself. */
export function paintGround(
  ctx: CanvasRenderingContext2D,
  tiles: Tileset,
  place: PlaceView,
  names: string[],
  flatWalls: boolean,
  mat: Materials,
): void {
  const background = place.kind === "Zone" ? "grass" : "floor";
  const w = place.width;
  const h = place.height;
  const at = (x: number, y: number): string | undefined =>
    x < 0 || y < 0 || x >= w || y >= h ? undefined : names[y * w + x];
  const group = (x: number, y: number, self: string): boolean => {
    const n = at(x, y);
    if (n === undefined) return RUNS_OFF_MAP.has(self);
    return FEATURE_GROUP[n] === self;
  };
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      let name = names[y * w + x] ?? "void";
      if (flatWalls && name === "door") name = place.kind === "Zone" ? "path" : "floor";
      const g = FEATURE_GROUP[name];
      const i = y * w + x;
      if (g === "wall" && !flatWalls) {
        const face = name === "door" ? null : wallFace(at(x, y + 1), mat.walls[i] ?? "timber");
        if (face) {
          tiles.draw(ctx, face, x, y);
          continue;
        }
      }
      if (!g || !tiles.has(`${g === "wall" ? "wall" : name}_out`)) {
        const floor = mat.floor[i];
        tiles.draw(ctx, floor && (name === "floor" || name === "opening") ? floor : name, x, y);
        continue;
      }
      const shapeName = g === "wall" ? "wall" : name;
      if (g !== "wall") tiles.draw(ctx, background, x, y);
      const n = group(x, y - 1, g), s = group(x, y + 1, g);
      const e = group(x + 1, y, g), wst = group(x - 1, y, g);
      const quads: [number, number, boolean, boolean, boolean][] = [
        [0, 0, n, wst, group(x - 1, y - 1, g)],
        [1, 0, n, e, group(x + 1, y - 1, g)],
        [0, 1, s, wst, group(x - 1, y + 1, g)],
        [1, 1, s, e, group(x + 1, y + 1, g)],
      ];
      for (const [qx, qy, vert, side, diag] of quads) {
        tiles.drawQuadrant(ctx, `${shapeName}_${quadrantShape(side, vert, diag)}`, qx, qy, x, y);
      }
      if (name === "door") tiles.draw(ctx, "door", x, y);
    }
  }
}

/** Forest cells that get a tree drawn on them, scattered so they do not line up. */
export function isTreeCell(names: string[], w: number, x: number, y: number): boolean {
  return names[y * w + x] === "canopy" && (x + y) % 2 === 0 && variantAt(x, y, 3) === 0;
}

/** Where a villager was last drawn and which way they were going. */
interface Gait {
  x: number;
  y: number;
  facing: Facing;
  steps: number;
}

/** A canvas showing one place. `draw` repaints it from the current sim state. */
export class PlaceCanvas {
  readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly objects: ObjectView[];
  /** Inline buildings' footprints in this place, for their name labels. */
  private readonly roofs: RoofView[];
  private readonly mat: Materials;
  private readonly dressing: Dressing;
  private readonly rail: WorldView["rail"];
  private readonly gaits = new Map<string, Gait>();

  constructor(
    private readonly tiles: Tileset,
    private readonly place: PlaceView,
    world: WorldView,
  ) {
    this.canvas = document.createElement("canvas");
    this.canvas.width = place.width * tiles.size;
    this.canvas.height = place.height * tiles.size;
    this.canvas.className = "place";
    this.canvas.setAttribute("role", "img");
    this.canvas.setAttribute("aria-label", `map of ${place.key}`);
    const ctx = this.canvas.getContext("2d");
    if (!ctx) throw new Error("no 2d canvas");
    ctx.imageSmoothingEnabled = false;
    this.ctx = ctx;
    this.objects = world.objects.filter((o) => o.place === place.id);
    this.roofs = world.roofs.filter((r) => r.place === place.id);
    this.mat = materials(place, world.rooms, world.roofs);
    this.dressing = dressingFor(world, place);
    this.rail = world.rail && world.rail.place === place.id ? world.rail : null;
  }

  /** The tile under a mouse or touch position on the canvas, or null when off it. */
  tileAt(clientX: number, clientY: number): { x: number; y: number } | null {
    const r = this.canvas.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return null;
    const x = Math.floor(((clientX - r.left) / r.width) * this.place.width);
    const y = Math.floor(((clientY - r.top) / r.height) * this.place.height);
    if (x < 0 || y < 0 || x >= this.place.width || y >= this.place.height) return null;
    return { x, y };
  }

  get key(): string {
    return this.place.key;
  }

  get id(): number {
    return this.place.id;
  }

  /** Which way a villager faces and which step they are on, from how they moved since the
   * last draw. Standing still shows the standing frame, facing the way they last went. */
  private gait(v: { key: string; pos: { x: number; y: number } }): { facing: Facing; frame: number } {
    const g = this.gaits.get(v.key) ?? { x: v.pos.x, y: v.pos.y, facing: "down" as Facing, steps: 0 };
    const dx = v.pos.x - g.x;
    const dy = v.pos.y - g.y;
    let frame = 0;
    if (dx !== 0 || dy !== 0) {
      if (Math.abs(dx) > Math.abs(dy)) g.facing = dx > 0 ? "right" : "left";
      else g.facing = dy > 0 ? "down" : "up";
      g.steps += 1;
      frame = 1 + (g.steps % 8);
    }
    g.x = v.pos.x;
    g.y = v.pos.y;
    this.gaits.set(v.key, g);
    return { facing: g.facing, frame };
  }

  /** The ground, trees, furniture and building names: everything that does not move,
   * painted into its own canvas and repainted every few seconds as the ground changes. */
  private layer: { canvas: HTMLCanvasElement; at: number } | null = null;

  private paintLayer(sim: Sim): HTMLCanvasElement {
    const now = performance.now();
    if (this.layer && now - this.layer.at < LAYER_MS) return this.layer.canvas;
    const canvas = this.layer?.canvas ?? document.createElement("canvas");
    canvas.width = this.canvas.width;
    canvas.height = this.canvas.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("no 2d canvas");
    ctx.imageSmoothingEnabled = false;
    const { tiles, place } = this;
    const w = place.width;
    const h = place.height;
    const names = groundNames(sim, place);
    const at = (x: number, y: number): string | undefined =>
      x < 0 || y < 0 || x >= w || y >= h ? undefined : names[y * w + x];
    paintGround(ctx, tiles, place, names, false, this.mat);
    paintDressing(ctx, tiles, place, names, this.mat, this.dressing, sim.clock().calendar);
    // Trees and furniture back to front, so whatever is lower on the map is in front.
    const sprites: { y: number; draw: () => void }[] = [];
    // Trees stand in the forest, scattered so they do not line up.
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        if (isTreeCell(names, w, x, y)) {
          const tree = treeAt(x, y, y > h * SOUTHERN_WOODS);
          sprites.push({ y, draw: () => tiles.draw(ctx, tree, x, y) });
        }
      }
    }
    for (const o of this.objects) {
      const below = at(o.pos.x, o.pos.y + 1);
      const rise = below !== "floor" && below !== "opening";
      const tile = objectTileAt(o, this.objects);
      if (tile) sprites.push({ y: o.pos.y, draw: () => tiles.draw(ctx, tile, o.pos.x, o.pos.y, rise) });
    }
    sprites.sort((a, b) => a.y - b.y);
    for (const s of sprites) s.draw();
    // Building names, on the top wall of each footprint.
    ctx.font = "bold 12px ui-monospace, monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "top";
    for (const r of this.roofs) {
      const s = tiles.size;
      const label = r.building.replace(/[_-]+/g, " ");
      const cx = (r.rect.x + r.rect.w / 2) * s;
      const top = r.rect.y * s + 3;
      const lw = ctx.measureText(label).width + 8;
      ctx.fillStyle = "rgba(20, 16, 10, 0.7)";
      ctx.fillRect(cx - lw / 2, top - 2, lw, 16);
      ctx.fillStyle = "#f4e8c8";
      ctx.fillText(label, cx, top);
    }
    this.layer = { canvas, at: now };
    return canvas;
  }

  draw(sim: Sim, villagers: VillagerView[], highlight: { x: number; y: number } | null = null): void {
    const { ctx, tiles, place } = this;
    ctx.drawImage(this.paintLayer(sim), 0, 0);
    // Villagers, animals and the train over the still layer, back to front.
    const here = villagers.filter((v) => v.place === place.id);
    const shift = spread(here, tiles.size);
    const sprites: { y: number; draw: () => void }[] = [];
    for (const a of sim.animals()) {
      if (a.place !== place.id || a.roosting) continue;
      const g = this.gait({ key: `animal-${a.id}`, pos: a.pos });
      sprites.push({ y: a.pos.y + 0.4, draw: () => tiles.drawAnimal(ctx, a, g.facing, g.frame) });
    }
    const train = this.rail ? sim.train() : null;
    if (train && this.rail) {
      for (const c of trainCars(train, this.rail.tiles)) {
        sprites.push({ y: c.y + 0.6, draw: () => tiles.drawAt(ctx, c.tile, c.x, c.y) });
      }
    }
    for (const v of here) {
      const gait = this.gait(v);
      // LPC has no lying-down frames: a sleeper is in their bed, shown by their name.
      if (v.activity === "sleep") continue;
      const facing = v.activity === "talk" ? (facePartner(v, here) ?? gait.facing) : gait.facing;
      const dx = shift.get(v.key) ?? 0;
      sprites.push({ y: v.pos.y + 0.5, draw: () => tiles.drawVillager(ctx, v.key, facing, gait.frame, v.pos.x, v.pos.y, dx) });
    }
    sprites.sort((a, b) => a.y - b.y);
    for (const s of sprites) s.draw();
    if (highlight) {
      const s = tiles.size;
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 1;
      ctx.strokeRect(highlight.x * s + 0.5, highlight.y * s + 0.5, s - 1, s - 1);
      ctx.strokeStyle = "#000000";
      ctx.strokeRect(highlight.x * s + 1.5, highlight.y * s + 1.5, s - 3, s - 3);
    }
    // Name labels last, so they sit over everything: above the head.
    ctx.font = "10px ui-monospace, monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    // Names on a shared cell stack upward, since they are wider than the shift between them.
    const stacked = new Map<string, number>();
    for (const v of [...here].sort((a, b) => (a.key < b.key ? -1 : 1))) {
      const s = tiles.size;
      const cell = `${v.pos.x},${v.pos.y}`;
      const row = stacked.get(cell) ?? 0;
      stacked.set(cell, row + 1);
      const cx = v.pos.x * s + s / 2 + (shift.get(v.key) ?? 0);
      const top = (v.pos.y + 1) * s - 50 - row * 12;
      const lw = ctx.measureText(v.name).width + 4;
      ctx.fillStyle = "rgba(0,0,0,0.55)";
      ctx.fillRect(cx - lw / 2, top - 11, lw, 11);
      ctx.fillStyle = "#fff";
      ctx.fillText(v.name, cx, top);
    }
  }
}

/** A strip of named tiles, for a legend: each tile at 2x with its label underneath. */
export function legend(tiles: Tileset, entries: [name: string, label: string][]): HTMLCanvasElement {
  const s = tiles.size;
  const cellW = 84;
  const canvas = document.createElement("canvas");
  canvas.width = entries.length * cellW;
  canvas.height = s * 2 + 14;
  canvas.className = "legend";
  canvas.setAttribute("role", "img");
  canvas.setAttribute("aria-label", `legend: ${entries.map(([, l]) => l).join(", ")}`);
  const ctx = canvas.getContext("2d");
  if (!ctx) return canvas;
  ctx.imageSmoothingEnabled = false;
  ctx.font = "10px ui-monospace, monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  entries.forEach(([name, label], i) => {
    const x = i * cellW + (cellW - s * 2) / 2;
    // Grass behind, so shaped features sit on their background like they do on the map.
    const t = tiles.atlas.tiles["grass"];
    if (t) ctx.drawImage(tiles.image, t.col * s, t.row * s, s, s, x, 0, s * 2, s * 2);
    const full = tiles.has(`${name}_full`) ? `${name}_full` : name;
    const f = tiles.atlas.tiles[full];
    if (f) ctx.drawImage(tiles.image, f.col * s, f.row * s, s, s, x, 0, s * 2, s * 2);
    ctx.fillStyle = "currentColor";
    ctx.fillText(label, i * cellW + cellW / 2, s * 2 + 2);
  });
  return canvas;
}
