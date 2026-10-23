// A text viewer: the sim advancing in the browser, as the same views the tools print.
// The renderer comes later and consumes the same `Sim` interface.
import * as wasm from "@game-world/sim-wasm";
import { LocalSim, TICKS_PER_MINUTE, at, type Sim } from "@game-world/protocol";

const content = await (await fetch("/content.json")).text();
const sim: Sim = new LocalSim(wasm, 1, content);

const $ = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;
const clockEl = $("clock");
const everyoneEl = $("everyone");
const placesEl = $("places");
const speedEl = $("speed");

const pres = new Map<string, HTMLPreElement>();
for (const id of sim.placeIds()) {
  const pre = document.createElement("pre");
  placesEl.appendChild(pre);
  pres.set(id, pre);
}

// Game-minutes per real second. 1 means a day takes 24 real minutes.
let speed = 6;
let paused = false;
let gameTick = at(0, 6);
sim.stepUntil(gameTick);

function draw(): void {
  const clock = sim.clock();
  const cal = clock.calendar;
  clockEl.textContent = `Y${cal.year} ${cal.season} ${cal.day_of_season} (${cal.weekday}) ${String(clock.time.hour).padStart(2, "0")}:${String(clock.time.minute).padStart(2, "0")}`;
  speedEl.textContent = paused ? "paused" : `${speed} game-min/s`;
  everyoneEl.textContent = sim.whereIsEveryone();
  for (const [id, pre] of pres) pre.textContent = sim.renderPlace(id);
}

let last = performance.now();
let carry = 0;
function frame(now: number): void {
  // Cap the catch-up after a hidden tab so a long absence does not freeze the page.
  const dt = Math.min((now - last) / 1000, 1);
  last = now;
  if (!paused) {
    carry += dt * speed * TICKS_PER_MINUTE;
    const whole = Math.floor(carry);
    if (whole > 0) {
      carry -= whole;
      gameTick += whole;
      sim.stepUntil(gameTick);
    }
  }
  draw();
  requestAnimationFrame(frame);
}

$("pause").addEventListener("click", () => { paused = !paused; });
$("faster").addEventListener("click", () => { speed = Math.min(speed * 2, 240); });
$("slower").addEventListener("click", () => { speed = Math.max(speed / 2, 1); });

// For Playwright and the agent: a structured dump of what is on screen.
declare global {
  interface Window { __scene: () => { tick: number; clock: string; everyone: string; hash: string } }
}
window.__scene = () => ({ tick: sim.tick, clock: clockEl.textContent ?? "", everyone: sim.whereIsEveryone(), hash: sim.hash() });

draw();
requestAnimationFrame(frame);
