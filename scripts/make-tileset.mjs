// Draws the placeholder tileset: 16px pixel tiles, procedurally, from a fixed seed, into
// packages/viewer/public/tiles.png with an atlas in tiles.json (name -> column, row).
// Generated art from the asset pipeline replaces the PNG; the names are the contract.
// Usage: node scripts/make-tileset.mjs
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";

const T = 16;
const COLS = 8;

// A small deterministic PRNG so the committed PNG is reproducible.
let seed = 0x9e3779b9;
function rnd() {
  seed = (Math.imul(seed ^ (seed >>> 15), 0x2c1b3c6d) >>> 0);
  seed = (Math.imul(seed ^ (seed >>> 12), 0x297a2d39) >>> 0);
  return ((seed ^ (seed >>> 15)) >>> 0) / 4294967296;
}
const hex = (h) => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16), 255];
const CLEAR = [0, 0, 0, 0];

class Tile {
  constructor() { this.px = Array.from({ length: T * T }, () => CLEAR); }
  set(x, y, c) { if (x >= 0 && y >= 0 && x < T && y < T) this.px[y * T + x] = c; }
  fill(c) { for (let i = 0; i < T * T; i++) this.px[i] = c; }
  rect(x, y, w, h, c) { for (let j = y; j < y + h; j++) for (let i = x; i < x + w; i++) this.set(i, j, c); }
  speckle(c, n) { for (let k = 0; k < n; k++) this.set(Math.floor(rnd() * T), Math.floor(rnd() * T), c); }
  blades(c, n, len = 3) {
    for (let k = 0; k < n; k++) {
      const x = Math.floor(rnd() * T), y = Math.floor(rnd() * T);
      for (let j = 0; j < len; j++) this.set(x, y + j, c);
    }
  }
  blob(cx, cy, r, c) { for (let j = -r; j <= r; j++) for (let i = -r; i <= r; i++) if (i * i + j * j <= r * r + 1) this.set(cx + i, cy + j, c); }
}

const grass = ["#5e9a48", "#6ba854", "#528a40"];
const dirt = ["#a68a5a", "#b89a66", "#8f7649"];
const stone = ["#8d8a80", "#a3a096", "#6e6b63"];
const wood = ["#9a7046", "#b08456", "#7a5836"];

function grassBase(t) {
  t.fill(hex(grass[0]));
  t.speckle(hex(grass[1]), 18);
  t.speckle(hex(grass[2]), 14);
}
function pathBase(t) {
  t.fill(hex(dirt[0]));
  t.speckle(hex(dirt[1]), 22);
  t.speckle(hex(dirt[2]), 10);
}
function floorBase(t) {
  t.fill(hex(wood[0]));
  for (let y = 0; y < T; y += 4) t.rect(0, y, T, 1, hex(wood[2]));
  for (let y = 0; y < T; y += 4) t.rect((y / 4) % 2 ? 11 : 4, y + 1, 1, 3, hex(wood[2]));
  t.speckle(hex(wood[1]), 10);
}

const tiles = {
  grass: (t) => grassBase(t),
  tall_grass: (t) => { t.fill(hex("#4f8a3e")); t.speckle(hex("#5e9a48"), 20); t.blades(hex("#7fbf5a"), 16, 4); t.blades(hex("#3f7030"), 8, 3); },
  brush: (t) => { t.fill(hex("#3f6e36")); t.blob(5, 6, 3, hex("#4f8a3e")); t.blob(11, 9, 3, hex("#4a803a")); t.blob(7, 12, 2, hex("#5e9a48")); t.speckle(hex("#2f5a2a"), 16); t.blades(hex("#6ba854"), 6, 3); },
  track: (t) => { grassBase(t); for (let y = 0; y < T; y++) for (let x = 5; x < 11; x++) if (rnd() < 0.55) t.set(x, y, hex(dirt[Math.floor(rnd() * 3)])); },
  cowpath: (t) => { pathBase(t); t.rect(0, 0, 2, T, hex(grass[0])); t.rect(14, 0, 2, T, hex(grass[0])); t.speckle(hex(grass[2]), 8); },
  path: (t) => { pathBase(t); t.speckle(hex("#c9ad78"), 6); },
  weedy_path: (t) => { pathBase(t); t.blades(hex("#6ba854"), 10, 2); t.speckle(hex("#5e9a48"), 8); },
  overgrown_path: (t) => { pathBase(t); t.speckle(hex("#4f8a3e"), 60); t.blades(hex("#7fbf5a"), 10, 3); },
  tree: (t) => { grassBase(t); t.blob(8, 7, 6, hex("#2f6a30")); t.blob(6, 6, 3, hex("#3f8a3c")); t.blob(10, 9, 2, hex("#3f8a3c")); t.rect(7, 12, 2, 3, hex("#5a4028")); },
  water: (t) => { t.fill(hex("#3a6fa8")); t.speckle(hex("#4a84c0"), 14); for (let k = 0; k < 4; k++) { const x = Math.floor(rnd() * 12), y = Math.floor(rnd() * T); t.rect(x, y, 4, 1, hex("#7fb0e0")); } },
  wall: (t) => { t.fill(hex(stone[0])); for (let y = 0; y < T; y += 4) { t.rect(0, y, T, 1, hex(stone[2])); const off = (y / 4) % 2 ? 4 : 0; for (let x = off; x < T; x += 8) t.rect(x, y, 1, 4, hex(stone[2])); } t.speckle(hex(stone[1]), 12); },
  floor: (t) => floorBase(t),
  opening: (t) => { floorBase(t); t.rect(0, 7, T, 2, hex("#5a4028")); },
  door: (t) => { floorBase(t); t.rect(3, 1, 10, 14, hex("#6b4a2c")); t.rect(4, 2, 8, 12, hex("#8a6238")); t.rect(7, 2, 1, 12, hex("#6b4a2c")); t.set(10, 8, hex("#e0c060")); },
  void: (t) => t.fill(hex("#101010")),
  obj_bed: (t) => { t.rect(1, 1, 14, 14, hex("#6b4a2c")); t.rect(2, 2, 12, 12, hex("#e8e4d8")); t.rect(2, 2, 12, 4, hex("#f4f1ea")); t.rect(2, 7, 12, 7, hex("#a84a5a")); },
  obj_table: (t) => { t.rect(2, 3, 12, 9, hex(wood[1])); t.rect(2, 3, 12, 1, hex("#c9a070")); t.rect(2, 12, 2, 3, hex(wood[2])); t.rect(12, 12, 2, 3, hex(wood[2])); },
  obj_counter: (t) => { t.rect(0, 4, T, 9, hex(wood[1])); t.rect(0, 4, T, 2, hex("#c9a070")); t.rect(0, 12, T, 1, hex(wood[2])); t.speckle(hex(wood[2]), 6); },
  obj_oven: (t) => { t.rect(1, 1, 14, 14, hex("#4a4a4a")); t.rect(2, 2, 12, 12, hex("#5e5e5e")); t.rect(4, 8, 8, 5, hex("#2a2a2a")); t.rect(5, 9, 6, 3, hex("#e07a2a")); t.set(7, 10, hex("#f6c04a")); },
  obj_stove: (t) => { t.rect(1, 1, 14, 14, hex("#3a3a3a")); t.rect(2, 2, 12, 12, hex("#555555")); t.blob(5, 5, 2, hex("#222222")); t.blob(10, 5, 2, hex("#222222")); t.blob(5, 10, 2, hex("#222222")); t.blob(10, 10, 2, hex("#222222")); },
  obj_shelf: (t) => { t.rect(1, 0, 14, 16, hex(wood[2])); t.rect(2, 1, 12, 14, hex(wood[0])); for (let y = 2; y < 15; y += 5) { t.rect(2, y + 3, 12, 1, hex(wood[2])); t.rect(3, y, 2, 3, hex("#c04a4a")); t.rect(7, y, 2, 3, hex("#4a7ac0")); t.rect(11, y, 2, 3, hex("#d0b040")); } },
  obj_generic: (t) => { t.rect(2, 2, 12, 12, hex("#9b4fb0")); t.rect(3, 3, 10, 10, hex("#c77dd6")); },
};

// Villagers: a simple front-facing figure, four palettes.
const outfits = [["#d8a070", "#3a2a1a", "#c04a4a"], ["#e0b080", "#6a4a2a", "#4a7ac0"], ["#c89060", "#1a1a1a", "#d0b040"], ["#e8c8a0", "#8a6a3a", "#4aa070"]];
outfits.forEach(([skin, hair, shirt], i) => {
  tiles[`villager_${i}`] = (t) => {
    t.rect(5, 2, 6, 5, hex(skin));
    t.rect(5, 1, 6, 2, hex(hair));
    t.set(5, 3, hex(hair)); t.set(10, 3, hex(hair));
    t.set(7, 4, hex("#1a1a1a")); t.set(9, 4, hex("#1a1a1a"));
    t.rect(4, 7, 8, 5, hex(shirt));
    t.rect(3, 8, 1, 3, hex(skin)); t.rect(12, 8, 1, 3, hex(skin));
    t.rect(5, 12, 2, 3, hex("#3a3a5a")); t.rect(9, 12, 2, 3, hex("#3a3a5a"));
    t.rect(5, 15, 2, 1, hex("#2a2a2a")); t.rect(9, 15, 2, 1, hex("#2a2a2a"));
  };
});

const names = Object.keys(tiles);
const rows = Math.ceil(names.length / COLS);
const W = COLS * T, H = rows * T;
const img = new Uint8Array(W * H * 4);
const atlas = {};
names.forEach((name, n) => {
  const t = new Tile();
  tiles[name](t);
  const col = n % COLS, row = Math.floor(n / COLS);
  atlas[name] = { col, row };
  for (let y = 0; y < T; y++) for (let x = 0; x < T; x++) {
    const c = t.px[y * T + x];
    const o = ((row * T + y) * W + col * T + x) * 4;
    img[o] = c[0]; img[o + 1] = c[1]; img[o + 2] = c[2]; img[o + 3] = c[3];
  }
});

// Minimal PNG writer: RGBA, no filtering.
function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) { c ^= b; for (let k = 0; k < 8; k++) c = c & 1 ? (c >>> 1) ^ 0xedb88320 : c >>> 1; }
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}
const raw = Buffer.alloc((W * 4 + 1) * H);
for (let y = 0; y < H; y++) { raw[y * (W * 4 + 1)] = 0; img.subarray(y * W * 4, (y + 1) * W * 4).forEach((v, i) => { raw[y * (W * 4 + 1) + 1 + i] = v; }); }
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4); ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw, { level: 9 })), chunk("IEND", Buffer.alloc(0)),
]);
mkdirSync("packages/viewer/public", { recursive: true });
writeFileSync("packages/viewer/public/tiles.png", png);
writeFileSync("packages/viewer/public/tiles.json", JSON.stringify({ tile: T, columns: COLS, tiles: atlas }, null, 2) + "\n");
console.log(`wrote tiles.png (${W}x${H}, ${names.length} tiles) and tiles.json`);
