// The tile renderer: one canvas per place, drawn from the sim's views with the placeholder
// tileset in public/tiles.png. Pixel art at native size, scaled up by CSS at an integer
// factor. The text views stay the source of truth for tests; this is for eyes.
import type { Cell, ObjectView, PlaceView, Sim, Surface, VillagerView } from "@game-world/protocol";

interface Atlas {
  tile: number;
  columns: number;
  tiles: Record<string, { col: number; row: number }>;
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

  draw(ctx: CanvasRenderingContext2D, name: string, x: number, y: number): void {
    const t = this.atlas.tiles[name] ?? this.atlas.tiles["obj_generic"];
    if (!t) return;
    const s = this.atlas.tile;
    ctx.drawImage(this.image, t.col * s, t.row * s, s, s, x * s, y * s, s, s);
  }
}

const CELL_TILE: Record<Cell, string> = {
  Ground: "grass",
  Path: "path",
  Blocked: "tree",
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

function objectTile(kind: string): string {
  switch (kind) {
    case "bed":
    case "table":
    case "counter":
    case "oven":
    case "stove":
    case "shelf":
      return `obj_${kind}`;
    default:
      return "obj_generic";
  }
}

/** A canvas showing one place. `draw` repaints it from the current sim state. */
export class PlaceCanvas {
  readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly objects: ObjectView[];
  private readonly outdoors: boolean;
  /** Villager key -> sprite variant, in first-seen order. */
  private readonly variants = new Map<string, number>();

  constructor(
    private readonly tiles: Tileset,
    private readonly place: PlaceView,
    objects: ObjectView[],
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
    this.outdoors = place.kind === "Zone";
  }

  /** Integer upscale so the place fits the given width, at least 1, at most `max`. */
  fit(width: number, max = 3): void {
    const scale = Math.max(1, Math.min(max, Math.floor(width / this.canvas.width)));
    this.canvas.style.width = `${this.canvas.width * scale}px`;
    this.canvas.style.height = `${this.canvas.height * scale}px`;
  }

  draw(sim: Sim, villagers: VillagerView[]): void {
    const { ctx, tiles, place } = this;
    const terrain = this.outdoors ? sim.terrain(place.key).tiles : null;
    for (let y = 0; y < place.height; y++) {
      for (let x = 0; x < place.width; x++) {
        const i = y * place.width + x;
        const surface = terrain?.[i] ?? null;
        const cell = place.cells[i] ?? "Void";
        tiles.draw(ctx, surface ? SURFACE_TILE[surface] : CELL_TILE[cell], x, y);
      }
    }
    for (const o of this.objects) tiles.draw(ctx, objectTile(o.kind), o.pos.x, o.pos.y);
    for (const v of villagers) {
      if (v.place !== place.id) continue;
      let variant = this.variants.get(v.key);
      if (variant === undefined) {
        variant = this.variants.size % 4;
        this.variants.set(v.key, variant);
      }
      tiles.draw(ctx, `villager_${variant}`, v.pos.x, v.pos.y);
    }
    // Name labels last, so they sit over everything.
    ctx.font = "7px ui-monospace, monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "bottom";
    for (const v of villagers) {
      if (v.place !== place.id) continue;
      const s = tiles.size;
      const cx = v.pos.x * s + s / 2;
      const top = v.pos.y * s - 1;
      const w = ctx.measureText(v.name).width + 2;
      ctx.fillStyle = "rgba(0,0,0,0.55)";
      ctx.fillRect(cx - w / 2, top - 8, w, 8);
      ctx.fillStyle = "#fff";
      ctx.fillText(v.name, cx, top);
    }
  }
}
