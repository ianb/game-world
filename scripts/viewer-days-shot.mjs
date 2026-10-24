// Screenshots the days panel after running the sim forward, then replays the first unusual
// line of the latest day and screenshots that moment. Usage: node scripts/viewer-days-shot.mjs [days]
import { existsSync } from "node:fs";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

const days = Number(process.argv[2] ?? 8);
const port = 4400 + Math.floor(Math.random() * 100);
const server = spawn("pnpm", ["exec", "vite", "preview", "--port", String(port), "--strictPort"], { cwd: "packages/viewer", stdio: "ignore" });
const preinstalled = "/opt/pw-browsers/chromium";
const executablePath = process.env.CHROMIUM_PATH || (existsSync(preinstalled) ? preinstalled : undefined);
try {
  const browser = await chromium.launch(executablePath ? { executablePath } : {});
  const page = await browser.newPage({ viewport: { width: 1100, height: 1000 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  for (let t = 0; ; t++) { try { await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle" }); break; } catch (e) { if (t > 20) throw e; await new Promise((r) => setTimeout(r, 250)); } }
  await page.waitForFunction(() => typeof window.__scene === "function", undefined, { timeout: 15_000 });
  await page.evaluate((d) => window.__advance(d), days);
  await page.waitForTimeout(300);
  await page.screenshot({ path: "target/viewer-days.png" });
  const first = page.locator("#days > details[open] button.moment").first();
  const label = await first.textContent();
  await first.click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: "target/viewer-days-replay.png" });
  const scene = await page.evaluate(() => window.__scene());
  console.log(`clicked: ${label}`);
  console.log(`now: ${scene.clock}; days summarised: ${scene.days}`);
  console.log(await page.locator("#inspect").textContent());
  if (errors.length) console.error("page errors:", errors);
  await browser.close();
} finally {
  server.kill();
}
