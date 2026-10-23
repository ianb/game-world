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
  /* Layout: a sticky clock bar, then the town's text views in a wrapping grid. */
  :root {
    --bg: #f4f2ec; --fg: #1d1b16; --muted: #6b6659; --bar: #e6e2d6; --rule: #cfc9b8; --accent: #8a5a2b;
    --mono: ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace;
  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; color-scheme: dark; }
  }
  :root[data-theme="dark"] { --bg: #121311; --fg: #dcd8cc; --muted: #9a9484; --bar: #1e201c; --rule: #33362f; --accent: #d9a35e; color-scheme: dark; }
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
  h2 { font-size: 13px; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; color: var(--muted); margin: 8px 0 4px; }
  button { font: inherit; min-height: 44px; min-width: 64px; background: var(--bg); color: var(--fg); border: 1px solid var(--rule); border-radius: 4px; }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  @media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto; } }
</style>
${script}
${body.trim()}
`;
writeFileSync(join(out, "index.html"), page);
for (const f of readdirSync(join(dist, "assets"))) cpSync(join(dist, "assets", f), join(out, "assets", f));
cpSync(join(dist, "content.json"), join(out, "content.json"));
console.log(`wrote ${out}/index.html and ${readdirSync(join(out, "assets")).length} assets`);
