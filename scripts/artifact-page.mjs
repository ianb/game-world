// Turns packages/viewer/dist into a page for a host that supplies its own document
// skeleton (claude.ai artifacts): strips the outer tags, swaps in theme tokens, and
// copies the assets and content next to it. Usage: node scripts/artifact-page.mjs <outdir>
import { cpSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const dist = "packages/viewer/dist";
const out = process.argv[2] ?? "target/artifact";
mkdirSync(join(out, "assets"), { recursive: true });
const html = readFileSync(join(dist, "index.html"), "utf8");
const script = html.match(/<script type="module"[^>]*><\/script>/)?.[0];
const body = html.match(/<body>([\s\S]*)<\/body>/)?.[1];
if (!script || !body) throw new Error("unexpected dist/index.html shape");

const page = `<title>Game World</title>
<meta name="claude:viewport-fit-sides" content="cover">
<style>
  /* Layout: a sticky clock bar, then the town's places as tile maps in a wrapping grid. */
  :root {
    --bg: #f4f2ec; --fg: #1d1b16; --muted: #6b6659; --bar: #e6e2d6; --rule: #cfc9b8; --accent: #8a5a2b;
    --mono: ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace;
    --c-ground: #4f8a3c; --c-tall: #3f7f2e; --c-brush: #2f5a2a; --c-weedy: #7a7230; --c-path: #9a7a2e; --c-track: #b8923e; --c-water: #3f6fa8; --c-wall: #8a8578; --c-object: #9b4fb0; --c-villager: #0b7f8a;
  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; --c-ground: #5f9a4a; --c-tall: #7fbf5a; --c-brush: #3f7a3a; --c-weedy: #8f8a4a; --c-path: #b8923e; --c-track: #e8c76a; --c-water: #4e7fb8; --c-wall: #777; --c-object: #c77dd6; --c-villager: #5fe0e0; color-scheme: dark; }
  }
  :root[data-theme="dark"] { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; --c-ground: #5f9a4a; --c-tall: #7fbf5a; --c-brush: #3f7a3a; --c-weedy: #8f8a4a; --c-path: #b8923e; --c-track: #e8c76a; --c-water: #4e7fb8; --c-wall: #777; --c-object: #c77dd6; --c-villager: #5fe0e0; color-scheme: dark; }
  body { margin: 0; background: var(--bg); color: var(--fg); font: 14px/1.3 var(--mono); font-variant-numeric: tabular-nums; }
  header {
    position: sticky; top: env(safe-area-inset-top, 0px); z-index: 2;
    display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center;
    padding-block: 8px; padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px));
    background: var(--bar); border-bottom: 1px solid var(--rule);
  }
  body::before { content: ""; position: fixed; inset: 0 0 auto; height: env(safe-area-inset-top, 0px); z-index: 10; background: var(--bar); }
  header strong { color: var(--accent); }
  #speed { color: var(--muted); }
  main {
    display: grid; gap: 16px; grid-template-columns: repeat(auto-fit, minmax(min(100%, 360px), 1fr));
    padding-block: 12px; padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px));
  }
  main > * { min-width: 0; }
  pre { margin: 0; white-space: pre; overflow-x: auto; }
  canvas.place { display: block; image-rendering: pixelated; image-rendering: crisp-edges; }
  #places { grid-column: 1 / -1; }
  #legend { overflow-x: auto; }
  #everyone { display: flex; flex-wrap: wrap; gap: 6px; }
  button.name { min-height: 32px; min-width: 0; padding: 4px 8px; text-align: left; }
  button.name[aria-pressed="true"] { outline: 2px solid var(--accent); }
  #inspect { white-space: pre-wrap; min-height: 4.5em; padding: 8px; border: 1px solid var(--rule); border-radius: 4px; }
  canvas.legend { display: block; image-rendering: pixelated; image-rendering: crisp-edges; }
  h2 { font-size: 13px; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; color: var(--muted); margin: 8px 0 4px; }
  button { font: inherit; min-height: 44px; min-width: 64px; background: var(--bg); color: var(--fg); border: 1px solid var(--rule); border-radius: 4px; }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .g-ground { color: var(--c-ground); } .g-tall { color: var(--c-tall); font-weight: bold; } .g-brush { color: var(--c-brush); } .g-weedy { color: var(--c-weedy); } .g-path { color: var(--c-path); } .g-track { color: var(--c-track); }
  .g-cowpath { color: var(--c-track); font-weight: bold; } .g-blocked { color: var(--c-water); } .g-wall { color: var(--c-wall); }
  .g-door { color: var(--fg); font-weight: bold; } .g-object { color: var(--c-object); } .g-villager { color: var(--c-villager); font-weight: bold; }
  @media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto; } }
</style>
${script}
${body.trim()}
`;
writeFileSync(join(out, "index.html"), page);
for (const f of readdirSync(join(dist, "assets"))) cpSync(join(dist, "assets", f), join(out, "assets", f));
for (const f of ["content.json", "tiles.png", "tiles.json"]) cpSync(join(dist, f), join(out, f));
console.log(`wrote ${out}/index.html and ${readdirSync(join(out, "assets")).length} assets`);
