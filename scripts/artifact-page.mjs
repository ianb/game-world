// Turns packages/viewer/dist into a page for claude.ai artifacts: the viewer's own
// stylesheet and body in a plain document, with the assets and content copied next to it.
// Usage: node scripts/artifact-page.mjs <outdir>
import { cpSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const dist = "packages/viewer/dist";
const out = process.argv[2] ?? "target/artifact";
mkdirSync(join(out, "assets"), { recursive: true });
const html = readFileSync(join(dist, "index.html"), "utf8");
const script = html.match(/<script type="module"[^>]*><\/script>/)?.[0];
const links = html.match(/<link rel="stylesheet"[^>]*>/g) ?? [];
const style = html.match(/<style>[\s\S]*?<\/style>/)?.[0];
const body = html.match(/<body>([\s\S]*)<\/body>/)?.[1];
if (!script || !style || !body) throw new Error("unexpected dist/index.html shape");

// A whole document: the viewer is one dark look by design, and its stylesheet sets every
// colour itself.
const page = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Game World</title>
${links.join("\n")}
${style}
${script}
</head>
<body>
${body.trim()}
</body>
</html>
`;
writeFileSync(join(out, "index.html"), page);
for (const f of readdirSync(join(dist, "assets"))) cpSync(join(dist, "assets", f), join(out, "assets", f));
for (const f of ["worlds", "tiles.png", "tiles.json"]) cpSync(join(dist, f), join(out, f), { recursive: true });
console.log(`wrote ${out}/index.html and ${readdirSync(join(out, "assets")).length} assets`);
