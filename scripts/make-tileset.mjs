// Builds the viewer's tileset from the LPC art in art/lpc: 32px tiles composed from the
// source sheets into packages/viewer/public/tiles.png, with an atlas in tiles.json. The tile
// names are the contract with the renderer; this script decides what each looks like.
// Usage: node scripts/make-tileset.mjs
import { readFileSync, writeFileSync } from "node:fs";
import { PNG } from "pngjs";

const T = 32;
const H = T / 2;
const ART = "art/lpc";

class Img {
  constructor(w, h, data) {
    this.w = w;
    this.h = h;
    this.data = data ?? Buffer.alloc(w * h * 4);
  }
  static load(path) {
    const p = PNG.sync.read(readFileSync(`${ART}/${path}`));
    return new Img(p.width, p.height, p.data);
  }
  /** Alpha-composites `src`'s (sx, sy, w, h) at (dx, dy). */
  draw(src, sx, sy, w, h, dx, dy) {
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const tx = dx + x, ty = dy + y, fx = sx + x, fy = sy + y;
        if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
        if (fx < 0 || fy < 0 || fx >= src.w || fy >= src.h) continue;
        const s = (fy * src.w + fx) * 4;
        const a = src.data[s + 3] / 255;
        if (a === 0) continue;
        const d = (ty * this.w + tx) * 4;
        const da = this.data[d + 3] / 255;
        const oa = a + da * (1 - a);
        for (let c = 0; c < 3; c++) {
          this.data[d + c] = Math.round((src.data[s + c] * a + this.data[d + c] * da * (1 - a)) / oa);
        }
        this.data[d + 3] = Math.round(oa * 255);
      }
    }
  }
  fill(x0, y0, w, h, rgba) {
    for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) this.data.set(rgba, (y * this.w + x) * 4);
  }
}

const sheets = new Map();
const sheet = (path) => {
  if (!sheets.has(path)) sheets.set(path, Img.load(path));
  return sheets.get(path);
};

/** A tile-sized (or w x h tiles) image, built by drawing onto it. */
const blank = (w = 1, h = 1) => new Img(w * T, h * T);
/** A copy of whole tiles from a sheet. */
function tiles(path, col, row, w = 1, h = 1) {
  const t = blank(w, h);
  t.draw(sheet(path), col * T, row * T, w * T, h * T, 0, 0);
  return t;
}
/** A copy of a pixel rectangle from a sheet, placed at (dx, dy) in a w x h tile image. */
function pixels(path, sx, sy, sw, sh, w = 1, h = 1, dx = 0, dy = 0) {
  const t = blank(w, h);
  t.draw(sheet(path), sx, sy, sw, sh, dx, dy);
  return t;
}

// What goes in the atlas: name -> { img, ox, oy } where ox/oy place a sprite larger than a
// tile relative to its cell (negative oy rises above it).
const out = {};
const put = (name, img, ox = 0, oy = 0, variants = undefined) => {
  out[name] = { img, ox, oy, variants };
};

// ---- Terrain: LPC blocks are 3 tiles wide and 6 tall: a scrap tile and a 2x2 of inner
// corners on top, a 3x3 blob of edges in the middle, three plain fills at the bottom. The
// renderer draws a shaped feature one quadrant at a time from five shape tiles, so each
// shape tile is assembled here from the matching quadrants of the block.
function shapes(name, path, bx, by, fullFromFill = false) {
  const q = (shape, pick) => {
    const t = blank();
    for (const [qx, qy] of [[0, 0], [1, 0], [0, 1], [1, 1]]) {
      const [c, r] = pick(qx, qy);
      t.draw(sheet(path), c * T + qx * H, r * T + qy * H, H, H, qx * H, qy * H);
    }
    put(`${name}_${shape}`, t);
  };
  // Some blocks' blob centre is flat; their textured fill row reads better in the middle.
  q("full", () => (fullFromFill ? [bx, by + 5] : [bx + 1, by + 3]));
  q("out", (qx, qy) => [bx + 2 * qx, by + 2 + 2 * qy]);
  q("h", (qx, qy) => [bx + 1, by + 2 + 2 * qy]);
  q("v", (qx) => [bx + 2 * qx, by + 3]);
  q("in", (qx, qy) => [bx + 2 - qx, by + 1 - qy]);
  put(name, out[`${name}_full`].img);
}

/** Plain fills from the bottom row of a block, as variants the renderer picks by position. */
function fills(name, path, bx, by, n = 3) {
  const t = blank(n, 1);
  t.draw(sheet(path), bx * T, (by + 5) * T, n * T, T, 0, 0);
  put(name, t, 0, 0, n);
}

fills("grass", "tiles/base/grass.png", 0, 0);
shapes("water", "tiles/base/water.png", 0, 0, true);
shapes("path", "tiles/base/dirt.png", 0, 0);
shapes("track", "tiles/base/dirt2.png", 0, 0);
shapes("cowpath", "tiles/atlas/terrain_atlas.png", 3, 26);
shapes("tall_grass", "tiles/atlas/terrain_atlas.png", 21, 6, true);
shapes("brush", "tiles/atlas/terrain_atlas.png", 21, 0, true);
shapes("canopy", "tiles/atlas/terrain_atlas.png", 6, 24, true);

// Weeds on a path: the dirt shapes with grass and weeds laid over, only where there is
// dirt so the edges stay clean. Overlays are [sheet, sx, sy, w, h, dx, dy] in pixels.
const ATLAS = "tiles/atlas/terrain_atlas.png";
const BLADES = [ATLAS, 0, 31 * T, T, T, 0, 0];
const clump = (dx, dy) => ["tiles/base/grass.png", 9, T + 10, 15, 13, dx, dy];
const weed = (dx, dy) => [ATLAS, 9 * T + 7, 25 * T + 14, 18, 15, dx, dy];
const sprout = (dx, dy) => [ATLAS, 11 * T + 10, 24 * T + 15, 9, 13, dx, dy];
function weedy(name, from, overlays) {
  for (const shape of ["full", "out", "h", "v", "in"]) {
    const base = out[`${from}_${shape}`].img;
    const layer = blank();
    for (const [path, sx, sy, w, h, dx, dy] of overlays) layer.draw(sheet(path), sx, sy, w, h, dx, dy);
    for (let i = 3; i < layer.data.length; i += 4) if (base.data[i] === 0) layer.data[i] = 0;
    const t = blank();
    t.draw(base, 0, 0, T, T, 0, 0);
    t.draw(layer, 0, 0, T, T, 0, 0);
    put(`${name}_${shape}`, t);
  }
  put(name, out[`${name}_full`].img);
}
weedy("weedy_path", "path", [BLADES, sprout(18, 3)]);
weedy("overgrown_path", "track", [BLADES, clump(2, 3), clump(16, 17), weed(13, 1), sprout(3, 18)]);

// A broadleaf tree, crown over trunk, standing on the bottom of its cell.
put("tree", (() => {
  const t = blank(3, 4);
  t.draw(sheet("tiles/base/trunk.png"), 0, 0, 3 * T, 3 * T, 0, T);
  t.draw(sheet("tiles/base/treetop.png"), 0, 0, 3 * T, 3 * T, 0, 0);
  return t;
})(), -T, -3 * T);

// ---- Buildings. Wall tops are drawn from the LPC wall-top beam: a dark fill with a beam
// along every side that faces away from the wall, assembled per quadrant like terrain.
const WALL = "tiles/walls/walls.png";
const BEAM = 9;
const dark = (() => {
  const s = sheet(WALL);
  const i = (40 * s.w + 48) * 4;
  return [s.data[i], s.data[i + 1], s.data[i + 2], 255];
})();
function wallShape(shape) {
  const t = blank();
  for (const [qx, qy] of [[0, 0], [1, 0], [0, 1], [1, 1]]) {
    const x0 = qx * H, y0 = qy * H;
    t.fill(x0, y0, H, H, dark);
    const edgeY = shape === "h" || shape === "out";
    const edgeX = shape === "v" || shape === "out";
    // Horizontal beam from the top of the horizontal wall-top tile, vertical from the left
    // of the vertical one.
    if (edgeY) t.draw(sheet(WALL), 64 + x0, 0, H, BEAM, x0, qy === 0 ? 0 : T - BEAM);
    if (edgeX) t.draw(sheet(WALL), 0, y0, BEAM, H, qx === 0 ? 0 : T - BEAM, y0);
    if (shape === "in") t.draw(sheet(WALL), 64, 0, BEAM, BEAM, qx === 0 ? 0 : T - BEAM, qy === 0 ? 0 : T - BEAM);
  }
  put(`wall_${shape}`, t);
}
for (const s of ["full", "out", "h", "v", "in"]) wallShape(s);
put("wall", out.wall_full.img);

// A wall seen from the room below it: a strip of top, then the panelled face.
const face = (col, row) => {
  const t = blank();
  t.draw(sheet(WALL), col * T, row * T + BEAM, T, T - BEAM, 0, BEAM);
  t.draw(sheet(WALL), 64, 0, T, BEAM, 0, 0);
  return t;
};
put("wall_face", face(1, 61));
// The outside of a building's south wall, seen from the street.
put("wall_outside", face(53, 67));

// Full-height walls for the 3D view, three tiles tall: wood panelling inside, half-timbered
// brick outside.
put("wall_tall_inside", tiles(WALL, 1, 59, 1, 3));
put("wall_tall_outside", tiles(WALL, 53, 65, 1, 3));

// The same per building material (`Walling` in content): the column and top row of a
// three-tile block in walls.png, outside and inside. Timber buildings are panelled inside,
// brick ones too; stone and board ones show the same material both sides.
const WALLINGS = {
  timber: { out: [53, 65], in: [1, 59] },
  stone: { out: [1, 20], in: [1, 20] },
  brick: { out: [1, 23], in: [1, 59] },
  boards: { out: [17, 59], in: [17, 59] },
};
for (const [name, w] of Object.entries(WALLINGS)) {
  put(`wall_outside_${name}`, face(w.out[0], w.out[1] + 2));
  put(`wall_face_${name}`, face(w.in[0], w.in[1] + 2));
  put(`wall_tall_outside_${name}`, tiles(WALL, w.out[0], w.out[1], 1, 3));
  put(`wall_tall_inside_${name}`, tiles(WALL, w.in[0], w.in[1], 1, 3));
}
// A window for outside walls, and its dark panes alone, which the 3D view lights at night.
put("window", tiles("tiles/windows-doors/windows-doors.png", 21, 6));
put("window_glow", (() => {
  const t = tiles("tiles/windows-doors/windows-doors.png", 21, 6);
  for (let i = 0; i < t.data.length; i += 4) {
    const dark = t.data[i + 3] > 0 && t.data[i] + t.data[i + 1] + t.data[i + 2] < 90;
    t.data.set(dark ? [255, 196, 120, 255] : [0, 0, 0, 0], i);
  }
  return t;
})());

put("floor", tiles("tiles/house-interior/interior.png", 0, 3));
// Room floors by material (`Flooring` in content).
put("floor_planks", tiles("tiles/house-interior/interior.png", 0, 3));
put("floor_darkplanks", tiles("tiles/base/inside.png", 0, 4));
put("floor_flagstone", tiles("tiles/atlas/build_atlas.png", 1, 3));
put("floor_cobbles", tiles("tiles/house-interior/interior.png", 1, 4));
put("floor_checker", tiles("tiles/house-interior/interior.png", 0, 4));
put("floor_earth", tiles("tiles/base/dirt.png", 1, 3));
put("opening", tiles("tiles/house-interior/interior.png", 0, 3));
put("door", tiles("tiles/windows-doors/windows-doors.png", 4, 26));
put("void", (() => {
  const t = blank();
  t.fill(0, 0, T, T, [24, 20, 24, 255]);
  return t;
})());

// ---- Objects. Most sit in their cell; taller ones rise above it, wider ones are centred.
// The bed is three tiles long in LPC; ours keep the head and the foot.
put("obj_bed", (() => {
  const t = blank(1, 2);
  t.draw(sheet("tiles/house-interior/interior.png"), 14 * T, 0, T, T, 0, 0);
  t.draw(sheet("tiles/house-interior/interior.png"), 14 * T, 2 * T, T, T, 0, T);
  return t;
})());
put("obj_table", tiles("tiles/house-interior/interior.png", 13, 2));
put("obj_counter", tiles("tiles/house-interior/interior.png", 3, 3, 1, 2), 0, -T);
put("obj_oven", tiles("tiles/house-interior/interior.png", 11, 4));
put("obj_stove", tiles("tiles/house-interior/interior.png", 11, 8));
put("obj_shelf", tiles("tiles/house-interior/interior.png", 2, 6, 1, 2), 0, -T);
put("obj_pantry", tiles("tiles/house-interior/interior.png", 6, 6));
put("obj_barrel", tiles("tiles/base/barrel.png", 0, 0, 1, 2), 0, -T);
put("obj_cart", tiles("tiles/decorations-medieval/decorations-medieval.png", 7, 20, 2, 2), -H, -T);
put("obj_privy", tiles("tiles/decorations-medieval/decorations-medieval.png", 11, 11, 2, 2), -H, -T);
put("obj_well", tiles("tiles/decorations-medieval/decorations-medieval.png", 0, 13, 2, 2), -H, -T);
put("obj_bench", tiles("tiles/decorations-medieval/decorations-medieval.png", 6, 34, 2, 1), -H, 0);
put("obj_chair", tiles("tiles/base/country.png", 2, 0));
put("obj_generic", tiles("tiles/house-interior/interior.png", 6, 6));
// Workplaces and furnishings.
const INT = "tiles/house-interior/interior.png";
const DEC = "tiles/decorations-medieval/decorations-medieval.png";
const OBJ = "tiles/atlas/obj_misk_atlas.png";
put("obj_anvil", tiles(OBJ, 6, 11));
put("obj_sacks", tiles(OBJ, 4, 13));
put("obj_millstone", tiles(OBJ, 15, 10));
put("obj_altar", tiles("tiles/atlas/build_atlas.png", 13, 12));
put("obj_workbench", tiles(INT, 4, 8));
put("obj_desk", tiles(INT, 4, 9));
put("obj_loom", tiles(INT, 5, 8));
put("obj_lectern", tiles(INT, 7, 6));
put("obj_bookshelf", tiles(INT, 1, 6, 1, 2), 0, -T);
put("obj_chest", tiles(INT, 3, 8, 1, 2), 0, -T);
put("obj_weaponrack", tiles(INT, 6, 8, 1, 2), 0, -T);
put("obj_clock", tiles(INT, 1, 10));
put("obj_hearth", tiles(INT, 12, 4));
put("obj_barrels", tiles(INT, 1, 8, 2, 2), -H, -T);
put("obj_haystack", tiles(DEC, 0, 20, 2, 2), -H, -T);
put("obj_firewood", tiles(DEC, 12, 20, 2, 2), -H, -T);
put("obj_logs", tiles(DEC, 13, 22));
put("obj_bucket", tiles(DEC, 2, 14));
put("obj_trough", tiles(DEC, 4, 5));
put("obj_churn", tiles(DEC, 2, 22));
put("obj_wheel", tiles(DEC, 3, 15));
put("obj_grave", tiles(DEC, 1, 3));
put("obj_cross", tiles(DEC, 0, 0, 1, 2), 0, -T);
put("obj_scarecrow", tiles(DEC, 10, 4, 1, 2), 0, -T);
put("obj_banner", tiles(DEC, 0, 38, 1, 2), 0, -T);
put("obj_lantern", tiles(DEC, 13, 4));
put("obj_stall", tiles(DEC, 6, 25, 2, 3), -H, -2 * T);
put("obj_hive", tiles(DEC, 3, 22));
put("obj_clamp", tiles(DEC, 10, 49));
put("obj_fence", tiles("tiles/atlas/build_atlas.png", 25, 11));

// ---- Villagers: each one's walk sheet composed from their layers (art/lpc/characters),
// recolored from each material's base palette. 9 frames of 64px; rows face up, left, down,
// right.
const FRAME = 64;
const layers = JSON.parse(readFileSync(`${ART}/characters/layers.json`, "utf8"));
const rgb = (h) => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
function recolor(img, material, color) {
  const p = layers.palettes[material];
  const map = new Map(p.base.map((b, i) => [rgb(b).join(","), rgb(p.colors[color][i])]));
  const t = new Img(img.w, img.h, Buffer.from(img.data));
  for (let i = 0; i < t.data.length; i += 4) {
    if (t.data[i + 3] === 0) continue;
    const to = map.get(`${t.data[i]},${t.data[i + 1]},${t.data[i + 2]}`);
    if (to) t.data.set(to, i);
  }
  return t;
}
const villagers = {};
for (const [who, list] of Object.entries(layers.villagers)) {
  const s = new Img(9 * FRAME, 4 * FRAME);
  for (const l of list) {
    let img = Img.load(`characters/${l.file}`);
    if (l.recolor) img = recolor(img, l.recolor.material, l.recolor.color);
    s.draw(img, 0, 0, s.w, s.h, 0, 0);
  }
  villagers[who] = s;
}

// ---- Pack: tiles on shelves of equal height, then one band per villager.
const COLS = 18;
const names = Object.keys(out).sort((a, b) => out[b].img.h - out[a].img.h || a.localeCompare(b));
const placed = {};
let x = 0, y = 0, shelf = 0;
for (const name of names) {
  const { img } = out[name];
  const w = img.w / T, h = img.h / T;
  if (x + w > COLS) {
    x = 0;
    y += shelf;
    shelf = 0;
  }
  placed[name] = { col: x, row: y };
  x += w;
  shelf = Math.max(shelf, h);
}
y += shelf;
// Villagers four to a row, so the sheet stays within the texture size every GPU allows.
const PER_ROW = 4;
const villagerTop = y * T;
const names_v = Object.keys(villagers);
const width = Math.max(COLS * T, PER_ROW * 9 * FRAME);
const atlas = new Img(width, villagerTop + Math.ceil(names_v.length / PER_ROW) * 4 * FRAME);
const entries = {};
for (const name of Object.keys(out).sort()) {
  const { img, ox, oy, variants } = out[name];
  const { col, row } = placed[name];
  atlas.draw(img, 0, 0, img.w, img.h, col * T, row * T);
  const e = { col, row };
  if (variants) e.variants = variants;
  else if (img.w !== T || img.h !== T) Object.assign(e, { w: img.w / T, h: img.h / T });
  if (ox) e.ox = ox;
  if (oy) e.oy = oy;
  entries[name] = e;
}
const villagerRows = {};
const villagerLeft = {};
names_v.forEach((who, i) => {
  const img = villagers[who];
  const x = (i % PER_ROW) * 9 * FRAME;
  const top = villagerTop + Math.floor(i / PER_ROW) * 4 * FRAME;
  atlas.draw(img, 0, 0, img.w, img.h, x, top);
  villagerRows[who] = top;
  villagerLeft[who] = x;
});

const png = new PNG({ width: atlas.w, height: atlas.h });
atlas.data.copy(png.data);
writeFileSync("packages/viewer/public/tiles.png", PNG.sync.write(png));
writeFileSync(
  "packages/viewer/public/tiles.json",
  `${JSON.stringify({ tile: T, columns: COLS, tiles: entries, villagers: { frame: FRAME, frames: 9, top: villagerRows, left: villagerLeft } }, null, 1)}\n`,
);
console.log(`wrote tiles.png (${atlas.w}x${atlas.h}, ${names.length} tiles, ${Object.keys(villagers).length} villagers) and tiles.json`);
