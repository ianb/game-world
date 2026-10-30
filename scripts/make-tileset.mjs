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
put("obj_firewood", tiles(DEC, 12, 20, 2, 2), -H, -T);
put("obj_logs", tiles(DEC, 13, 22));
put("obj_bucket", tiles(DEC, 2, 14));
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

// ---- The valley outdoors and its furnishings, second round.
const TERR = "tiles/atlas/terrain_atlas.png";
const BLD = "tiles/atlas/build_atlas.png";
const WD = "tiles/windows-doors/windows-doors.png";
/** `img` mirrored left to right. */
function flipH(img) {
  const t = new Img(img.w, img.h);
  for (let y = 0; y < img.h; y++) {
    for (let x = 0; x < img.w; x++) t.data.set(img.data.subarray((y * img.w + x) * 4, (y * img.w + x) * 4 + 4), (y * img.w + (img.w - 1 - x)) * 4);
  }
  return t;
}
const rgba = (h, a = 255) => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16), a];

// Corrections: the sacks were half of a two-tile drawing; banners are gone for plants; the
// barn's hay is one bale and the trough a real trough.
put("obj_sacks", tiles(OBJ, 1, 15));
put("obj_plant", tiles(BLD, 7, 5, 1, 2), 0, -T);
put("obj_haystack", tiles(TERR, 0, 26, 1, 2), 0, -T);
put("obj_trough", tiles(DEC, 14, 18, 2, 1), -H, 0);
put("obj_coop", tiles(DEC, 9, 22, 2, 2), -H, -T);
put("obj_crates", tiles(TERR, 21, 18, 2, 1), -H, 0);
put("obj_platform", tiles(TERR, 16, 20, 3, 1), -T, 0);
put("ladder", tiles(TERR, 9, 5));
// Two beds side by side as one: the left bed's left part and the right bed's right part,
// one frame between them.
put("obj_bed_double", (() => {
  const single = (() => {
    const b = blank(1, 2);
    b.draw(sheet(INT), 14 * T, 0, T, T, 0, 0);
    b.draw(sheet(INT), 14 * T, 2 * T, T, T, 0, T);
    return b;
  })();
  const t = blank(2, 2);
  t.draw(single, 0, 0, 29, 2 * T, 3, 0);
  t.draw(single, 3, 0, 29, 2 * T, 32, 0);
  return t;
})());
// Doorways: a sill across an opening, and the red doors of a barn standing over a wide one.
put("threshold_wood", tiles(TERR, 16, 20));
put("threshold_stone", tiles(TERR, 17, 26));
put("barn_door", tiles(TERR, 17, 21, 2, 2), 0, -T);
put("barn_door_l", tiles(TERR, 17, 21, 1, 2), 0, -T);
put("barn_door_r", tiles(TERR, 18, 21, 1, 2), 0, -T);

// Trees of several kinds; a cell picks one by position. Each stands on its cell.
/** `img` with its leaves recoloured: each green pixel's green and red swapped toward
 * `mix` ([r, g, b] multipliers on the pixel's green), for fir and autumn variants. */
function retint(img, mix) {
  const t = new Img(img.w, img.h, Buffer.from(img.data));
  for (let i = 0; i < t.data.length; i += 4) {
    const [r, g, b, a] = [t.data[i], t.data[i + 1], t.data[i + 2], t.data[i + 3]];
    if (a === 0 || g <= r || g <= b) continue;
    t.data[i] = Math.min(255, Math.round(g * mix[0]));
    t.data[i + 1] = Math.min(255, Math.round(g * mix[1]));
    t.data[i + 2] = Math.min(255, Math.round(g * mix[2]));
  }
  return t;
}
const pine = pixels(TERR, 962, 0, 62, 160, 3, 5, 20, 0);
const oak = tiles(TERR, 29, 28, 3, 4);
put("tree_pine", pine, -T, -4 * T);
put("tree_fir", retint(pine, [0.35, 0.8, 0.75]), -T, -4 * T);
put("tree_oak", oak, -T, -3 * T);
put("tree_young", tiles(TERR, 27, 29, 2, 3), -H, -2 * T);
put("tree_autumn", retint(oak, [1.15, 0.6, 0.2]), -T, -3 * T);

// Things in the grass, on the banks and in the water.
put("grass_flowers", tiles(TERR, 21, 11, 3, 1), 0, 0, 3);
put("decor_rock", tiles(TERR, 22, 21));
put("decor_stones", tiles(TERR, 24, 21));
put("decor_mushrooms", tiles(TERR, 27, 28));
put("decor_toadstool", tiles(TERR, 26, 31));
put("decor_bush", tiles(TERR, 9, 14));
put("decor_fern", tiles(TERR, 13, 14));
put("decor_reeds", tiles(TERR, 26, 29, 1, 2), 0, -T);
put("decor_lily", tiles(TERR, 6, 30, 3, 1), 0, 0, 3);
put("decor_lily_flower", tiles(TERR, 6, 31, 3, 1), 0, 0, 3);

// Crops: tilled soil, then each crop as it grows: sprouts, growing, ready.
put("soil", tiles(TERR, 4, 28));
put("crop_sprout", tiles(TERR, 9, 24, 3, 1), 0, 0, 3);
put("crop_wheat_growing", tiles(TERR, 9, 18));
put("crop_wheat_ready", tiles(TERR, 1, 29));
put("crop_cabbages", tiles(TERR, 10, 28));
put("crop_cabbages_growing", tiles(TERR, 10, 26));
put("crop_carrots", tiles(TERR, 10, 30));
put("crop_carrots_growing", tiles(TERR, 11, 25));
put("crop_potatoes", tiles(TERR, 9, 30));
put("crop_potatoes_growing", tiles(TERR, 9, 27));
put("crop_beans", tiles(TERR, 12, 28));
put("crop_beans_growing", tiles(TERR, 12, 26));
put("crop_corn", tiles(TERR, 15, 26, 1, 3), 0, -2 * T);
put("crop_corn_growing", tiles(TERR, 15, 25));
put("crop_flowers", tiles(TERR, 21, 11, 3, 1), 0, 0, 3);
put("crop_flowers_growing", tiles(TERR, 14, 25));

// Bridges: boards across the water with a rail along each open side.
put("bridge", tiles(TERR, 16, 20));
/** A wooden rail along one edge of a bridge tile: posts and a top bar. */
function bridgeRail(top) {
  const t = blank();
  const y = top ? 0 : T - 9;
  t.fill(0, y + 1, T, 3, rgba("#6b4426"));
  t.fill(0, y + 1, T, 1, rgba("#9a6a3c"));
  for (const x of [2, 18]) t.fill(x, y, 4, 9, rgba("#4e301a"));
  return t;
}
put("bridge_rail_n", bridgeRail(true));
put("bridge_rail_s", bridgeRail(false));

// The railway, drawn here: sleepers and two rails, across and down, and the train.
put("rail_h", (() => {
  const t = blank();
  for (let x = 2; x < T; x += 8) t.fill(x, 6, 5, 20, rgba("#5b3f28"));
  t.fill(0, 9, T, 3, rgba("#8f969c"));
  t.fill(0, 20, T, 3, rgba("#8f969c"));
  t.fill(0, 9, T, 1, rgba("#c7ccd0"));
  t.fill(0, 20, T, 1, rgba("#c7ccd0"));
  return t;
})());
put("rail_v", (() => {
  const t = blank();
  for (let y = 2; y < T; y += 8) t.fill(6, y, 20, 5, rgba("#5b3f28"));
  t.fill(9, 0, 3, T, rgba("#8f969c"));
  t.fill(20, 0, 3, T, rgba("#8f969c"));
  return t;
})());
/** A train car seen from the side, two tiles long, facing east (right). */
function car(kind) {
  const t = blank(2, 2);
  const wheel = (x) => {
    t.fill(x, 52, 10, 10, rgba("#2a2a2e"));
    t.fill(x + 2, 54, 6, 6, rgba("#a33a2c"));
    t.fill(x + 4, 56, 2, 2, rgba("#e0c06a"));
  };
  if (kind === "engine") {
    t.fill(4, 44, 58, 8, rgba("#1d1d22"));
    t.fill(18, 30, 36, 16, rgba("#2c3f35"));
    t.fill(18, 32, 36, 2, rgba("#47614f"));
    for (const x of [26, 38]) t.fill(x, 30, 2, 16, rgba("#c49a3a"));
    t.fill(46, 18, 7, 13, rgba("#1d1d22"));
    t.fill(44, 16, 11, 3, rgba("#1d1d22"));
    t.fill(54, 38, 6, 8, rgba("#1d1d22"));
    t.fill(2, 22, 18, 24, rgba("#5a1f1a"));
    t.fill(0, 20, 22, 3, rgba("#1d1d22"));
    t.fill(6, 26, 10, 8, rgba("#e7d9a0"));
    t.fill(56, 40, 6, 3, rgba("#e7d9a0"));
    for (const x of [8, 24, 40]) wheel(x);
  } else {
    const body = kind === "wagon" ? "#7a5232" : "#5d6a73";
    t.fill(2, 26, 60, 22, rgba(body));
    for (let x = 2; x < 62; x += 6) t.fill(x, 26, 1, 22, rgba("#3e2a1a"));
    t.fill(2, 24, 60, 3, rgba("#3e2a1a"));
    t.fill(2, 46, 60, 3, rgba("#1d1d22"));
    t.fill(0, 44, 2, 3, rgba("#1d1d22"));
    t.fill(62, 44, 2, 3, rgba("#1d1d22"));
    for (const x of [8, 46]) wheel(x);
  }
  return t;
}
put("train_engine_e", car("engine"), 0, -T);
put("train_engine_w", flipH(car("engine")), 0, -T);
put("train_wagon", car("wagon"), 0, -T);
put("train_van", car("van"), 0, -T);

// Farm animals: four facings (up, left, down, right) by four walking frames. Chickens are a
// tile a frame; sheep are cropped to two tiles from their 128px frames.
put("animal_chicken", (() => {
  const t = blank(4, 4);
  t.draw(sheet("animals/chicken_walk.png"), 0, 0, 128, 128, 0, 0);
  return t;
})());
put("animal_sheep", (() => {
  const t = blank(8, 8);
  for (let r = 0; r < 4; r++) for (let c = 0; c < 4; c++) t.draw(sheet("animals/sheep_walk.png"), c * 128 + 32, r * 128 + 40, 64, 64, c * 64, r * 64);
  return t;
})());

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
