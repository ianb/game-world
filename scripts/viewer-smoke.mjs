// Loads the built viewer in headless Chromium, waits for the sim to advance, and prints the
// scene dump. Usage: node scripts/viewer-smoke.mjs <url>
import { existsSync } from "node:fs";
import { chromium } from "playwright";

const url = process.argv[2] ?? "http://localhost:4173/";
// Prefer an explicit CHROMIUM_PATH, then the cloud container's preinstalled Chromium, else
// whatever `playwright install chromium` provided.
const preinstalled = "/opt/pw-browsers/chromium";
const executablePath = process.env.CHROMIUM_PATH || (existsSync(preinstalled) ? preinstalled : undefined);
const browser = await chromium.launch(executablePath ? { executablePath } : {});
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
await page.screenshot({ path: "target/viewer-smoke.png" });
await browser.close();
console.log(`first:  ${first.clock} tick ${first.tick}`);
console.log(`second: ${second.clock} tick ${second.tick}`);
console.log(second.everyone);
if (errors.length) { console.error("page errors:", errors); process.exit(1); }
if (second.tick <= first.tick) { console.error("sim did not advance"); process.exit(1); }
console.log("ok");
