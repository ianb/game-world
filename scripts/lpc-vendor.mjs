// Copies what the villagers' outfits (art/lpc/characters.json) need from a checkout of the
// Universal LPC Spritesheet Character Generator into art/lpc/characters: each layer's walk
// sheet, the palettes to recolor it, and the credits for every file used. Run it again after
// changing an outfit, then `just tileset`.
// Usage: node scripts/lpc-vendor.mjs <generator checkout>
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const gen = process.argv[2];
if (!gen) throw new Error("usage: node scripts/lpc-vendor.mjs <generator checkout>");
const out = "art/lpc/characters";
const read = (p) => JSON.parse(readFileSync(join(gen, p), "utf8"));
const outfits = JSON.parse(readFileSync("art/lpc/characters.json", "utf8")).villagers;

const ANIMATION = "walk";
const layers = {};
const files = new Set();
const credits = new Map();
const palettes = {};

/** Every layer of one item, for one body type: the file, its z, and how to recolor it. */
function resolve(defPath, body, color, skin) {
  const def = read(join("sheet_definitions", defPath));
  const found = [];
  for (const [key, layer] of Object.entries(def)) {
    if (!key.startsWith("layer_") || !layer[body]) continue;
    const base = `${layer[body]}${ANIMATION}`;
    const variants = def.variants ?? [];
    let file;
    let recolor = null;
    if (variants.length) {
      if (!variants.includes(color)) throw new Error(`${defPath}: no variant ${color} (${variants.join(", ")})`);
      file = `${base}/${color}.png`;
    } else {
      file = `${base}.png`;
      const r = def.recolors?.color_1 ?? def.recolors;
      if (def.match_body_color || r?.material === "body") recolor = { material: "body", color: skin };
      else if (r?.material && color) recolor = { material: r.material, color };
    }
    found.push({ file, z: layer.zPos, recolor });
    for (const c of def.credits ?? []) {
      if (file.startsWith(c.file)) credits.set(c.file, c);
    }
  }
  if (!found.length) throw new Error(`${defPath}: nothing for body ${body}`);
  return found;
}

for (const [who, o] of Object.entries(outfits)) {
  const all = resolve("body/body.json", o.body, null, o.skin);
  for (const [def, color] of o.items) all.push(...resolve(def, o.body, color ?? null, o.skin));
  all.sort((a, b) => a.z - b.z);
  layers[who] = all;
  for (const l of all) {
    files.add(l.file);
    if (l.recolor) (palettes[l.recolor.material] ??= new Set()).add(l.recolor.color);
  }
}

// The checkout may be sparse: ask git for exactly these files.
const wanted = [...files].map((f) => `spritesheets/${f}`);
execFileSync("git", ["-C", gen, "sparse-checkout", "add", "--no-cone", ...wanted], { stdio: "ignore" });

rmSync(out, { recursive: true, force: true });
for (const f of files) {
  const src = join(gen, "spritesheets", f);
  if (!existsSync(src)) throw new Error(`missing ${src}`);
  mkdirSync(dirname(join(out, f)), { recursive: true });
  copyFileSync(src, join(out, f));
}

// Each material's base palette (what the sheets are drawn in) and the colors used.
const palettesOut = {};
for (const [material, colors] of Object.entries(palettes)) {
  const meta = read(`palette_definitions/${material}/meta_${material}.json`);
  const table = read(`palette_definitions/${material}/${material}_ulpc.json`);
  const entry = { base: table[meta.base], colors: {} };
  for (const c of colors) {
    if (!table[c]) throw new Error(`${material}: no color ${c}`);
    entry.colors[c] = table[c];
  }
  palettesOut[material] = entry;
}

writeFileSync(join(out, "layers.json"), `${JSON.stringify({ animation: ANIMATION, villagers: layers, palettes: palettesOut }, null, 1)}\n`);
writeFileSync(join(out, "credits.json"), `${JSON.stringify([...credits.values()], null, 1)}\n`);
const md = [...credits.values()].map(
  (c) => `- \`${c.file}\`: ${c.authors.join(", ")}. ${c.licenses.join(" / ")}.${c.notes ? ` ${c.notes}.` : ""}${c.urls?.length ? `\n  ${c.urls.join("\n  ")}` : ""}`,
);
writeFileSync(join(out, "CREDITS.md"), `# Villager sprite credits\n\nThe layers in this folder, from the Universal LPC Spritesheet Character Generator.\n\n${md.join("\n")}\n`);
console.log(`${files.size} layer files, ${credits.size} credit entries, palettes: ${Object.keys(palettesOut).join(", ")}`);
