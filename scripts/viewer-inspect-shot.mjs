// Screenshots the page with the inspector pinned on a villager and the pointer over a
// tile, for checking the inspector by eye. Usage: node scripts/viewer-inspect-shot.mjs
import { existsSync } from "node:fs";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

const port = 4300 + Math.floor(Math.random() * 100);
const server = spawn("pnpm", ["exec", "vite", "preview", "--port", String(port), "--strictPort"], { cwd: "packages/viewer", stdio: "ignore" });
const preinstalled = "/opt/pw-browsers/chromium";
const executablePath = process.env.CHROMIUM_PATH || (existsSync(preinstalled) ? preinstalled : undefined);
try {
  const browser = await chromium.launch(executablePath ? { executablePath } : {});
  const page = await browser.newPage({ viewport: { width: 1100, height: 900 }, deviceScaleFactor: 1 });
  for (let t = 0; ; t++) { try { await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle" }); break; } catch (e) { if (t > 20) throw e; await new Promise((r) => setTimeout(r, 250)); } }
  await page.waitForFunction(() => typeof window.__scene === "function", undefined, { timeout: 15_000 });
  await page.evaluate(() => window.__advance(3));
  await page.locator("button.name").first().click();
  await page.waitForTimeout(200);
  await page.screenshot({ path: "target/viewer-inspect-villager.png" });
  const canvas = page.locator("canvas.place").first();
  const box = await canvas.boundingBox();
  await page.locator("button.name").first().click(); // unpin
  await page.mouse.move(box.x + box.width * (31 / 48), box.y + box.height * (21 / 36)); // the pond
  await page.waitForTimeout(200);
  await page.screenshot({ path: "target/viewer-inspect-tile.png" });
  console.log(await page.locator("#inspect").textContent());
  await browser.close();
} finally {
  server.kill();
}
