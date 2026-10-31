// What dresses a place beyond its cells: crops in their plots, the railway, bridges where
// a path crosses water, sills in doorways, ladders up to lofts, and the flowers, stones and
// reeds scattered over the ground. All of it lies flat, so the 2D map and the 3D ground
// share it. Also which tree stands where.
import type { Calendar, DoorView, FieldTileView, FieldView, PlaceView, PlotView, WorldView } from "@game-world/protocol";
import { type Materials, type TileRect, type Tileset, variantAt, within } from "./render";

export interface Dressing {
  plots: PlotView[];
  /** Rail tiles in this place. */
  rails: { x: number; y: number }[];
  /** Cells at either end of a ladder. */
  ladders: Set<number>;
  /** Whether lily pads float on still water here. */
  lilies: boolean;
}

export function dressingFor(world: WorldView, place: PlaceView): Dressing {
  const w = place.width;
  const ladders = new Set<number>();
  const ends = (d: DoorView): [number, { x: number; y: number }][] => [
    [d.from_place, d.from],
    [d.to_place, d.to],
  ];
  for (const d of world.doors) {
    if (!d.key.startsWith("ladder")) continue;
    for (const [p, pos] of ends(d)) if (p === place.id) ladders.add(pos.y * w + pos.x);
  }
  return {
    plots: world.plots.filter((p) => p.place === place.id),
    rails: world.rail && world.rail.place === place.id && world.rail.vehicle === "train" ? world.rail.tiles : [],
    ladders,
    // Decks mean open water: no lilies, which belong to the valley's ponds and river.
    lilies: !place.cells.includes("Deck"),
  };
}

/** How far along a crop is on this day of the year. */
type Stage = "bare" | "sprout" | "growing" | "ready";
function stage(c: Calendar): Stage {
  const d = c.day_of_season;
  switch (c.season) {
    case "Spring":
      return d < 10 ? "sprout" : "growing";
    case "Summer":
      return d < 15 ? "growing" : "ready";
    case "Autumn":
      return d < 15 ? "ready" : "bare";
    case "Winter":
      return "bare";
  }
}

/** The tile drawn over the soil for a crop at a stage, or null for bare soil. A crop that
 * fills its ground (wheat) replaces the soil instead. */
function cropTile(crop: string, s: Stage, x: number, y: number): { tile: string; covers: boolean } | null {
  if (s === "bare") return null;
  if (s === "sprout") return { tile: "crop_sprout", covers: false };
    if (crop === "wheat") return { tile: s === "ready" ? "crop_wheat_ready" : "crop_wheat_growing", covers: true };
  // Rows: every other row of a bed holds plants, so the soil shows between them.
  if (y % 2 === 1 && crop !== "flowers") return null;
  if (crop === "corn" && x % 2 === 1) return null;
  return { tile: s === "ready" ? `crop_${crop}` : `crop_${crop}_growing`, covers: crop === "flowers" };
}

/** A field tile: bare, tilled or overgrown ground, or its crop at the stage it has
 * reached, with a straw tint for stubble, a brown one for a crop spoiling, a pale one for a
 * crop gone dry, and weeds showing through when they are thick. */
function paintFieldTile(ctx: CanvasRenderingContext2D, tiles: Tileset, f: FieldView, t: FieldTileView, x: number, y: number): void {
  const s = tiles.size;
  const tint = (colour: string): void => {
    ctx.fillStyle = colour;
    ctx.fillRect(x * s, y * s, s, s);
  };
  if (t.stage === "Fallow") {
    if (t.weeds >= 8) tiles.draw(ctx, "tall_grass_full", x, y);
    else {
      tiles.draw(ctx, "soil", x, y);
      if (t.weeds >= 3) tiles.draw(ctx, "decor_fern", x, y);
    }
    return;
  }
  const stage: Stage = t.stage === "Sprouting" ? "sprout" : t.stage === "Growing" ? "growing" : t.stage === "Ripe" || t.stage === "Spoiling" ? "ready" : "bare";
  const c = cropTile(f.crop, stage, x, y);
  if (!c?.covers) tiles.draw(ctx, "soil", x, y);
  if (c) tiles.draw(ctx, c.tile, x, y);
  if (t.stage === "Stubble") tint("rgba(196, 160, 90, 0.45)");
  if (t.stage === "Spoiling") tint("rgba(80, 52, 24, 0.45)");
  const planted = t.stage === "Sown" || t.stage === "Sprouting" || t.stage === "Growing";
  if (planted && t.water === 0) tint("rgba(214, 186, 110, 0.35)");
  if (t.weeds >= 5 && t.stage !== "Stubble") tiles.draw(ctx, "decor_fern", x, y);
}

const WATER = "water";
const isPath = (n: string | undefined): boolean => n === "path" || n === "weedy_path" || n === "overgrown_path" || n === "track" || n === "cowpath";

/** Paints the flat dressing over a painted ground. */
export function paintDressing(
  ctx: CanvasRenderingContext2D,
  tiles: Tileset,
  place: PlaceView,
  names: string[],
  mat: Materials,
  dressing: Dressing,
  calendar: Calendar | null,
  region: TileRect | null = null,
  fields: FieldView[] | null = null,
): void {
  const w = place.width;
  const h = place.height;
  const r = within(region, w, h);
  const cell = (x: number, y: number): string | undefined => (x < 0 || y < 0 || x >= w || y >= h ? undefined : place.cells[y * w + x]);
  const name = (x: number, y: number): string | undefined => (x < 0 || y < 0 || x >= w || y >= h ? undefined : names[y * w + x]);
  const waterAt = (x: number, y: number): boolean => cell(x, y) === "Water";
  // Bridges: a path cell with water on both sides across it.
  for (let y = r.y0; y < r.y1; y++) {
    for (let x = r.x0; x < r.x1; x++) {
      if (cell(x, y) !== "Path") continue;
      const across = waterAt(x, y - 1) && waterAt(x, y + 1);
      const along = waterAt(x - 1, y) && waterAt(x + 1, y);
      const under = (waterAt(x, y - 1) || waterAt(x, y + 1)) && (waterAt(x - 1, y) || waterAt(x + 1, y) || isPath(name(x - 1, y)));
      if (!across && !along && !under) continue;
      tiles.draw(ctx, "bridge", x, y);
      if (waterAt(x, y - 1)) tiles.draw(ctx, "bridge_rail_n", x, y);
      if (waterAt(x, y + 1)) tiles.draw(ctx, "bridge_rail_s", x, y);
    }
  }
  // Crops: each tile as the sim has it, or by the calendar when there is no sim state.
  if (fields) {
    for (const f of fields) {
      f.tiles.forEach((t, i) => {
        const x = f.rect.x + (i % f.rect.w);
        const y = f.rect.y + Math.floor(i / f.rect.w);
        if (x >= r.x0 && x < r.x1 && y >= r.y0 && y < r.y1) paintFieldTile(ctx, tiles, f, t, x, y);
      });
    }
  } else {
    const s = calendar ? stage(calendar) : "growing";
    for (const p of dressing.plots) {
      for (let y = p.rect.y; y < p.rect.y + p.rect.h; y++) {
        for (let x = p.rect.x; x < p.rect.x + p.rect.w; x++) {
          const c = cropTile(p.crop, s, x, y);
          if (!c?.covers) tiles.draw(ctx, "soil", x, y);
          if (c) tiles.draw(ctx, c.tile, x, y);
        }
      }
    }
  }
  // The railway, on a bridge where it crosses water.
  for (const r of dressing.rails) {
    if (waterAt(r.x, r.y)) {
      tiles.draw(ctx, "bridge", r.x, r.y);
      tiles.draw(ctx, "bridge_rail_n", r.x, r.y);
      tiles.draw(ctx, "bridge_rail_s", r.x, r.y);
    }
    tiles.draw(ctx, "rail_h", r.x, r.y);
  }
  // Doorways: a sill of the building's material, and ladders to the loft.
  const doors: [number, number, string][] = [];
  for (let y = r.y0; y < r.y1; y++) {
    for (let x = r.x0; x < r.x1; x++) {
      const i = y * w + x;
      if (dressing.ladders.has(i)) {
        tiles.draw(ctx, "ladder", x, y);
        continue;
      }
      if (cell(x, y) !== "Opening") continue;
      const material = [mat.walls[i - 1], mat.walls[i + 1], mat.walls[i - w], mat.walls[i + w]].find((m) => m !== undefined);
      tiles.draw(ctx, material === "stone" ? "threshold_stone" : "threshold_wood", x, y);
      // A wide doorway in a board building is a barn's: its doors stand open against the
      // wall on either side.
      if (material === "boards" && cell(x + 1, y) === "Opening" && cell(x - 1, y) === "Wall" && cell(x + 2, y) === "Wall") {
        doors.push([x - 1, y, "barn_door_l"], [x + 2, y, "barn_door_r"]);
      }
    }
  }
  for (const [x, y, t] of doors) tiles.draw(ctx, t, x, y);
  // Things in the grass and the water, scattered by position.
  const plotted = new Set<number>();
  for (const p of dressing.plots) {
    for (let y = p.rect.y; y < p.rect.y + p.rect.h; y++) for (let x = p.rect.x; x < p.rect.x + p.rect.w; x++) plotted.add(y * w + x);
  }
  const railed = new Set(dressing.rails.map((r) => r.y * w + r.x));
  for (let y = r.y0; y < r.y1; y++) {
    for (let x = r.x0; x < r.x1; x++) {
      const i = y * w + x;
      if (plotted.has(i) || railed.has(i)) continue;
      const n = name(x, y);
      const v = variantAt(x * 7 + 3, y * 5 + 1, 120);
      if (n === WATER) {
        const bank = !waterAt(x - 1, y) || !waterAt(x + 1, y) || !waterAt(x, y - 1) || !waterAt(x, y + 1);
        if (dressing.lilies && !bank && v < 6) tiles.draw(ctx, v < 2 ? "decor_lily_flower" : "decor_lily", x, y);
        continue;
      }
      if (n !== "grass") continue;
      const nearWater = waterAt(x - 1, y) || waterAt(x + 1, y) || waterAt(x, y - 1) || waterAt(x, y + 1);
      const nearWood = name(x - 1, y) === "canopy" || name(x + 1, y) === "canopy" || name(x, y - 1) === "canopy" || name(x, y + 1) === "canopy";
      if (nearWater && v < 24) tiles.draw(ctx, "decor_reeds", x, y);
      else if (nearWood && v < 8) tiles.draw(ctx, v < 4 ? "decor_mushrooms" : "decor_toadstool", x, y);
      else if (nearWood && v < 20) tiles.draw(ctx, "decor_fern", x, y);
      else if (v < 22) tiles.draw(ctx, "grass_flowers", x, y);
      else if (v < 26) tiles.draw(ctx, "crop_flowers_growing", x, y);
      else if (v === 26) tiles.draw(ctx, "decor_rock", x, y);
      else if (v === 27) tiles.draw(ctx, "decor_stones", x, y);
      else if (v < 31) tiles.draw(ctx, "decor_bush", x, y);
    }
  }
}

/** Which tree stands on a wooded cell: pines and firs in the southern woods, broadleaves
 * elsewhere, with a few of each mixed in. */
export function treeAt(x: number, y: number, southernWoods: boolean): string {
  const v = variantAt(x, y + 11, 20);
  if (southernWoods) return v < 8 ? "tree_pine" : v < 15 ? "tree_fir" : v < 18 ? "tree" : "tree_oak";
  return v < 6 ? "tree" : v < 11 ? "tree_oak" : v < 14 ? "tree_young" : v < 16 ? "tree_autumn" : v < 18 ? "tree_pine" : "tree_fir";
}

/** The southern woods start this far down a zone, as a fraction of its height. */
export const SOUTHERN_WOODS = 0.72;
