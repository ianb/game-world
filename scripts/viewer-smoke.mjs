// Loads the built viewer in headless Chromium, waits for the sim to advance, and prints the
// scene dump. Usage: node scripts/viewer-smoke.mjs <url>
import { chromium } from "playwright";

const url = process.argv[2] ?? "http://localhost:4173/";
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || undefined });
const page = await browser.newPage({ viewport: { width: 1000, height: 800 } });
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
await page.goto(url, { waitUntil: "networkidle" });
await page.waitForFunction(() => typeof window.__scene === "function");
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
