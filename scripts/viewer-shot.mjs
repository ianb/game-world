// Screenshots one place from the built viewer after running the sim forward, at the tiles'
// native size (read from its canvas), so a person or the agent's image reader can look at
// the tiles. Serves dist itself.
// Usage: node scripts/viewer-shot.mjs [--place town] [--days 12] [--out target/town-tiles.png]
import { existsSync, writeFileSync } from "node:fs";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

const args = Object.fromEntries(process.argv.slice(2).map((a, i, all) => (a.startsWith("--") ? [a.slice(2), all[i + 1]] : [])).filter((e) => e.length));
const placeKey = args.place ?? "town";
const days = Number(args.days ?? 12);
const out = args.out ?? `target/${placeKey.replace(/\//g, "-")}-tiles.png`;
const port = 4190 + Math.floor(Math.random() * 100);

const server = spawn("pnpm", ["exec", "vite", "preview", "--port", String(port), "--strictPort"], { cwd: "packages/viewer", stdio: "ignore" });
const preinstalled = "/opt/pw-browsers/chromium";
const executablePath = process.env.CHROMIUM_PATH || (existsSync(preinstalled) ? preinstalled : undefined);
try {
  const browser = await chromium.launch(executablePath ? { executablePath } : {});
  const page = await browser.newPage({ viewport: { width: 1400, height: 1000 }, deviceScaleFactor: 2 });
  for (let tries = 0; ; tries++) {
    try { await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle" }); break; }
    catch (e) { if (tries > 20) throw e; await new Promise((r) => setTimeout(r, 250)); }
  }
  await page.waitForFunction(() => typeof window.__scene === "function", undefined, { timeout: 15_000 });
  await page.evaluate((d) => window.__advance(d), days);
  const scene = await page.evaluate(() => window.__scene());
  sim_place_index(scene.places, placeKey);
  const url = await page.evaluate((k) => window.__show(k), placeKey);
  writeFileSync(out, Buffer.from(url.slice(url.indexOf(",") + 1), "base64"));
  await browser.close();
  console.log(`${scene.clock}: wrote ${out}`);
} finally {
  server.kill();
}

function sim_place_index(places, key) {
  const i = places.findIndex((text) => text.startsWith(`${key} (`));
  if (i < 0) throw new Error(`no place ${key}; have ${places.map((p) => p.split(" ")[0]).join(", ")}`);
  return i;
}
