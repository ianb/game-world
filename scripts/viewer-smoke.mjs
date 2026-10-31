// Loads the built viewer in headless Chromium, waits for the sim to advance, and prints the
// scene dump. Usage: node scripts/viewer-smoke.mjs <url> [place to draw in 3D]
import { existsSync } from "node:fs";
import { chromium } from "playwright";

const url = process.argv[2] ?? "http://localhost:4173/";
const place3d = process.argv[3] ?? "town";
// Prefer an explicit CHROMIUM_PATH, then the cloud container's preinstalled Chromium, else
// whatever `playwright install chromium` provided.
const preinstalled = "/opt/pw-browsers/chromium";
const executablePath = process.env.CHROMIUM_PATH || (existsSync(preinstalled) ? preinstalled : undefined);
// SwiftShader gives headless Chromium WebGL for the 3D view.
const args = ["--enable-unsafe-swiftshader"];
const browser = await chromium.launch(executablePath ? { executablePath, args } : { args });
const page = await browser.newPage({ viewport: { width: 1000, height: 800 } });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
await page.goto(url, { waitUntil: "networkidle" });
try {
  await page.waitForFunction(() => typeof window.__scene === "function", undefined, { timeout: 15_000 });
} catch (e) {
  console.error("page never exposed __scene;", errors.length ? `errors: ${errors.join("; ")}` : String(e));
  await browser.close();
  process.exit(1);
}
const first = await page.evaluate(() => window.__scene());
await page.waitForTimeout(1500);
const second = await page.evaluate(() => window.__scene());
await page.screenshot({ path: `target/viewer-smoke-${place3d}.png` });
// A day on, replay a moment from the first day (its snapshot was taken before the first
// step), then draw the town in 3D.
await page.evaluate(() => window.__advance(1));
await page.getByRole("tab", { name: "days" }).click();
await page.locator("#days button.moment").first().click();
await page.waitForTimeout(300);
const replayed = await page.evaluate(() => window.__scene());
const shot = await page.evaluate((key) => window.__show3d(key), place3d);
await browser.close();
console.log(`first:  ${first.clock} tick ${first.tick}`);
console.log(`second: ${second.clock} tick ${second.tick}`);
console.log(second.everyone);
if (errors.length) { console.error("page errors:", errors); process.exit(1); }
if (second.tick <= first.tick) { console.error("sim did not advance"); process.exit(1); }
if (!second.canvases || second.canvases !== second.places.length) { console.error(`expected ${second.places.length} canvases, got ${second.canvases}`); process.exit(1); }
if (!replayed.clock.includes("Spring 1 ")) { console.error(`replay did not go back to the first day: ${replayed.clock}`); process.exit(1); }
if (!shot || shot.length < 10_000) { console.error("3D view drew nothing"); process.exit(1); }
console.log(`replayed: ${replayed.clock}; 3D view ${Math.round(shot.length / 1024)} KB`);
console.log("ok");
