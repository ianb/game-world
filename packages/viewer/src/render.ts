// The tile renderer: one canvas per place, drawn from the sim's views with the LPC tileset
// in public/tiles.png (built by scripts/make-tileset.mjs). Pixel art at native size, scaled
// by CSS at an integer factor. The text views stay the source of truth for tests; this is
// for eyes.
import type { Cell, ObjectView, PlaceView, RoofView, Sim, Surface, VillagerView } from "@game-world/protocol";

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
  /** Each villager's walk sheet: `frames` frames of `frame` px; rows face up, left, down, right. */
  villagers: { frame: number; frames: number; top: Record<string, number> };
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

  /** Draws one quadrant (qx, qy in 0..1) of a shape tile into the same quadrant of (x, y). */
  drawQuadrant(ctx: CanvasRenderingContext2D, name: string, qx: number, qy: number, x: number, y: number): void {
    const t = this.atlas.tiles[name];
    if (!t) return;
    const s = this.atlas.tile;
    const h = s / 2;
    ctx.drawImage(this.image, t.col * s + qx * h, t.row * s + qy * h, h, h, x * s + qx * h, y * s + qy * h, h, h);
  }

  /** A villager standing (frame 0) or mid-step, feet on the bottom of their cell. */
  drawVillager(ctx: CanvasRenderingContext2D, key: string, facing: Facing, frame: number, x: number, y: number): void {
    const v = this.atlas.villagers;
    const top = v.top[key] ?? Object.values(v.top)[0];
    if (top === undefined) return;
    const s = this.atlas.tile;
    const f = v.frame;
    const sx = (frame % v.frames) * f;
    const sy = top + FACING_ROW[facing] * f;
    ctx.drawImage(this.image, sx, sy, f, f, x * s + s / 2 - f / 2, (y + 1) * s - f + FEET, f, f);
  }
}

/** LPC frames leave a little room under the feet. */
const FEET = 2;

export type Facing = "up" | "left" | "down" | "right";
const FACING_ROW: Record<Facing, number> = { up: 0, left: 1, down: 2, right: 3 };

/** A stable pseudo-random pick for a cell, so the ground does not repeat in a grid. */
function variantAt(x: number, y: number, n: number): number {
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
function wallFace(below: string | undefined): string | null {
  if (below === "floor" || below === "opening") return "wall_face";
  if (below === undefined || below === "void" || FEATURE_GROUP[below] === "wall") return null;
  return "wall_outside";
}

function objectTile(kind: string): string {
  switch (kind) {
    case "bed":
    case "table":
    case "counter":
    case "oven":
    case "stove":
    case "shelf":
    case "pantry":
    case "barrel":
    case "cart":
    case "privy":
    case "well":
    case "bench":
    case "chair":
      return `obj_${kind}`;
    default:
      return "obj_generic";
  }
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
  private readonly outdoors: boolean;
  private readonly gaits = new Map<string, Gait>();

  constructor(
    private readonly tiles: Tileset,
    private readonly place: PlaceView,
    objects: ObjectView[],
    roofs: RoofView[] = [],
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
    this.objects = objects.filter((o) => o.place === place.id);
    this.roofs = roofs.filter((r) => r.place === place.id);
    this.outdoors = place.kind === "Zone";
  }

  /** Integer upscale so the place fits the given width, at most `max`. Only the width is
   * set; the height follows, so tiles stay square when the page is narrower than 1x. */
  fit(width: number, max = 3): void {
    const scale = Math.max(1, Math.min(max, Math.floor(width / this.canvas.width)));
    this.canvas.style.width = `${this.canvas.width * scale}px`;
    this.canvas.style.maxWidth = "100%";
    this.canvas.style.height = "auto";
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
  private gait(v: VillagerView): { facing: Facing; frame: number } {
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

  draw(sim: Sim, villagers: VillagerView[], highlight: { x: number; y: number } | null = null): void {
    const { ctx, tiles, place } = this;
    const terrain = this.outdoors ? sim.terrain(place.key).tiles : null;
    const background = this.outdoors ? "grass" : "floor";
    const w = place.width;
    const h = place.height;
    // The tile name for every cell, then a second pass shapes the auto-tiled ones.
    const names: string[] = new Array<string>(w * h);
    for (let i = 0; i < w * h; i++) {
      const surface = terrain?.[i] ?? null;
      names[i] = surface ? SURFACE_TILE[surface] : CELL_TILE[place.cells[i] ?? "Void"];
    }
    const at = (x: number, y: number): string | undefined =>
      x < 0 || y < 0 || x >= w || y >= h ? undefined : names[y * w + x];
    const group = (x: number, y: number, self: string): boolean => {
      const n = at(x, y);
      if (n === undefined) return RUNS_OFF_MAP.has(self);
      return FEATURE_GROUP[n] === self;
    };
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const name = names[y * w + x] ?? "void";
        const g = FEATURE_GROUP[name];
        if (g === "wall") {
          const face = name === "door" ? null : wallFace(at(x, y + 1));
          if (face) {
            tiles.draw(ctx, face, x, y);
            continue;
          }
        }
        if (!g || !tiles.has(`${g === "wall" ? "wall" : name}_out`)) {
          tiles.draw(ctx, name, x, y);
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
    // Objects and people back to front, so whoever is lower on the map is in front.
    const sprites: { y: number; draw: () => void }[] = [];
    // Trees stand in the forest, scattered so they do not line up.
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        if (names[y * w + x] === "canopy" && (x + y) % 2 === 0 && variantAt(x, y, 3) === 0) {
          sprites.push({ y, draw: () => tiles.draw(ctx, "tree", x, y) });
        }
      }
    }
    for (const o of this.objects) {
      const below = at(o.pos.x, o.pos.y + 1);
      const rise = below !== "floor" && below !== "opening";
      sprites.push({ y: o.pos.y, draw: () => tiles.draw(ctx, objectTile(o.kind), o.pos.x, o.pos.y, rise) });
    }
    for (const v of villagers) {
      if (v.place !== place.id) continue;
      const { facing, frame } = this.gait(v);
      // LPC has no lying-down frames: a sleeper is in their bed, shown by their name.
      if (v.activity === "sleep") continue;
      sprites.push({ y: v.pos.y + 0.5, draw: () => tiles.drawVillager(ctx, v.key, facing, frame, v.pos.x, v.pos.y) });
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
    for (const v of villagers) {
      if (v.place !== place.id) continue;
      const s = tiles.size;
      const cx = v.pos.x * s + s / 2;
      const top = (v.pos.y + 1) * s - 50;
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
