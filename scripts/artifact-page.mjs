// Turns packages/viewer/dist into a page for a host that supplies its own document
// skeleton (claude.ai artifacts): strips the outer tags, swaps in theme tokens, and
// copies the assets and content next to it. Usage: node scripts/artifact-page.mjs <outdir>
// Its stylesheet is a themed copy of the one in packages/viewer/index.html: a layout change
// there has to be made here too.
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
  /* Layout: a clock bar, then the map (one place at a time) beside a tabbed sidebar that
     scrolls on its own; narrow screens stack them. */
  :root {
    --bg: #f4f2ec; --fg: #1d1b16; --muted: #6b6659; --bar: #e6e2d6; --rule: #cfc9b8; --accent: #8a5a2b;
    --mono: ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace;
    --c-ground: #4f8a3c; --c-tall: #3f7f2e; --c-brush: #2f5a2a; --c-weedy: #7a7230; --c-path: #9a7a2e; --c-track: #b8923e; --c-water: #3f6fa8; --c-wall: #8a8578; --c-object: #9b4fb0; --c-villager: #0b7f8a;
  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; --c-ground: #5f9a4a; --c-tall: #7fbf5a; --c-brush: #3f7a3a; --c-weedy: #8f8a4a; --c-path: #b8923e; --c-track: #e8c76a; --c-water: #4e7fb8; --c-wall: #777; --c-object: #c77dd6; --c-villager: #5fe0e0; color-scheme: dark; }
  }
  :root[data-theme="dark"] { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; --c-ground: #5f9a4a; --c-tall: #7fbf5a; --c-brush: #3f7a3a; --c-weedy: #8f8a4a; --c-path: #b8923e; --c-track: #e8c76a; --c-water: #4e7fb8; --c-wall: #777; --c-object: #c77dd6; --c-villager: #5fe0e0; color-scheme: dark; }
  html, body { height: 100%; }
  body { margin: 0; background: var(--bg); color: var(--fg); font: 14px/1.3 var(--mono); font-variant-numeric: tabular-nums; display: flex; flex-direction: column; height: 100dvh; overflow: hidden; }
  header {
    display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center;
    padding-block: 8px; padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px));
    background: var(--bar); border-bottom: 1px solid var(--rule);
  }
  body::before { content: ""; position: fixed; inset: 0 0 auto; height: env(safe-area-inset-top, 0px); z-index: 10; background: var(--bar); }
  header strong { color: var(--accent); }
  label.speed, .zoom { display: inline-flex; gap: 6px; align-items: center; color: var(--muted); }
  .zoom button { min-width: 44px; }
  select { font: inherit; min-height: 44px; background: var(--bg); color: var(--fg); border: 1px solid var(--rule); border-radius: 4px; }
  #clock { display: inline-block; min-width: 27ch; }
  #pause { width: 10ch; }
  #app { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 460px; }
  #stage, #side { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  #side { border-left: 1px solid var(--rule); padding-right: env(safe-area-inset-right, 0px); }
  #stage { padding-left: env(safe-area-inset-left, 0px); }
  nav.tabs { display: flex; flex-wrap: wrap; gap: 4px; padding: 6px 8px; border-bottom: 1px solid var(--rule); }
  nav.tabs button { min-height: 36px; padding: 4px 10px; }
  nav.tabs button[aria-selected="true"] { outline: 2px solid var(--accent); }
  #map { flex: 1; min-height: 320px; position: relative; overflow: hidden; touch-action: none; cursor: grab; background: var(--bar); }
  #map.dragging { cursor: grabbing; }
  #map canvas.place { position: absolute; left: 0; top: 0; transform-origin: 0 0; }
  .tab { flex: 1; min-height: 0; overflow: auto; padding: 10px 12px; display: grid; gap: 12px; align-content: start; }
  .tab[hidden], section[hidden] { display: none; }
  pre { margin: 0; white-space: pre; overflow-x: auto; }
  canvas.place { display: block; image-rendering: pixelated; image-rendering: crisp-edges; }
  #legend { overflow-x: auto; }
  .daynav { display: grid; grid-template-columns: auto 1fr auto; gap: 8px; align-items: center; }
  .daynav strong { text-align: center; }
  #days { display: grid; gap: 4px; align-content: start; }
  #days details { border-top: 1px solid var(--rule); padding-top: 4px; }
  #days summary { cursor: pointer; min-height: 32px; display: flex; align-items: center; }
  button.moment { display: grid; grid-template-columns: 6em 1fr; gap: 8px; width: 100%; min-height: 32px; text-align: left; background: none; color: inherit; border: none; border-radius: 4px; padding: 4px 6px; }
  button.moment:hover { background: var(--bar); }
  button.moment .when { color: var(--muted); text-align: right; }
  #clock.past { color: var(--accent); }
  .muted { color: var(--muted); }
  #everyone { display: grid; grid-template-columns: max-content 22ch minmax(0, 1fr); gap: 4px 12px; align-items: center; align-content: start; }
  #everyone > span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  #everyone .muted { color: var(--muted); }
  button.name { min-height: 32px; min-width: 0; padding: 4px 10px; text-align: left; }
  button.name[aria-pressed="true"] { outline: 2px solid var(--accent); }
  #inspect { white-space: pre-wrap; overflow-wrap: anywhere; min-height: 12.5em; padding: 8px; border: 1px solid var(--rule); border-radius: 4px; font-variant-numeric: tabular-nums; }
  canvas.legend { display: block; image-rendering: pixelated; image-rendering: crisp-edges; }
  a { color: var(--accent); }
  button { font: inherit; min-height: 44px; min-width: 64px; background: var(--bg); color: var(--fg); border: 1px solid var(--rule); border-radius: 4px; }
  button:disabled { opacity: 0.4; }
  #words { display: grid; gap: 8px; align-content: start; }
  #words h2 { font-size: 13px; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; color: var(--muted); margin: 8px 0 0; }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; }
  button.word { min-height: 32px; min-width: 0; padding: 2px 8px; }
  #inspect button.word { min-height: 0; padding: 0 4px; font: inherit; }
  .word-head { white-space: pre-wrap; }
  #words button.moment { grid-template-columns: 11ch 1fr; }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  @media (max-width: 900px) {
    body { height: auto; overflow: auto; }
    #app { grid-template-columns: minmax(0, 1fr); }
    #map { flex: none; height: 60vh; }
    #side { border-left: none; border-top: 1px solid var(--rule); }
    .tab { max-height: 75vh; }
  }
  @media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto; } }
</style>
${script}
${body.trim()}
`;
writeFileSync(join(out, "index.html"), page);
for (const f of readdirSync(join(dist, "assets"))) cpSync(join(dist, "assets", f), join(out, "assets", f));
for (const f of ["content.json", "tiles.png", "tiles.json"]) cpSync(join(dist, f), join(out, f));
console.log(`wrote ${out}/index.html and ${readdirSync(join(out, "assets")).length} assets`);
