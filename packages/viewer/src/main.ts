// A text viewer: the sim advancing in the browser, as the same views the tools print.
// The renderer comes later and consumes the same `Sim` interface.
import * as wasm from "@game-world/sim-wasm";
import { LocalSim, TICKS_PER_MINUTE, at, type Sim } from "@game-world/protocol";

const $ = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;
const clockEl = $("clock");
const everyoneEl = $("everyone");
const placesEl = $("places");
const speedEl = $("speed");

async function loadContent(): Promise<string> {
  const res = await fetch(new URL("content.json", document.baseURI));
  if (!res.ok) throw new Error(`content.json: HTTP ${res.status}`);
  return res.text();
}

let sim: Sim;
try {
  sim = LocalSim.create(wasm, 1, await loadContent());
} catch (e) {
  clockEl.textContent = `failed to start: ${String(e)}`;
  throw e;
}

const pres = new Map<string, HTMLPreElement>();
for (const id of sim.placeIds()) {
  const section = document.createElement("section");
  const h = document.createElement("h2");
  h.textContent = id;
  const pre = document.createElement("pre");
  section.append(h, pre);
  placesEl.appendChild(section);
  pres.set(id, pre);
}

// Game-minutes per real second. 6 means a day takes four real minutes.
const SPEEDS = [1, 2, 6, 12, 30, 60, 120, 240];
let speedIndex = 2;
let paused = false;
let gameTick = at(0, 6);
sim.stepUntil(gameTick);
let drawnTick = -1;
let drawnPaused = !paused;

// CSS class per map glyph, matching the legend in crates/content/src/render.rs.
function glyphClass(ch: string): string | null {
  switch (ch) {
    case "·": return "g-ground";
    case "\"": return "g-tall";
    case "♠": return "g-brush";
    case "┈": return "g-weedy";
    case "░": return "g-path";
    case "∙": return "g-track";
    case "▒": return "g-cowpath";
    case "♣": return "g-blocked";
    case "█": return "g-wall";
    case "▫":
    case "▪": return "g-door";
    default:
      if (ch >= "a" && ch <= "z") return "g-object";
      if (ch >= "A" && ch <= "Z") return "g-villager";
      return null;
  }
}

/** Fills a <pre> with the rendered place, one span per coloured glyph. */
function paint(pre: HTMLPreElement, text: string): void {
  const frag = document.createDocumentFragment();
  const lines = text.split("\n");
  lines.forEach((line, i) => {
    if (i < 2 || line.length < 4) {
      frag.append(line, "\n");
      return;
    }
    frag.append(line.slice(0, 4));
    let run = "";
    let runClass: string | null = null;
    const flush = (): void => {
      if (!run) return;
      if (runClass) {
        const span = document.createElement("span");
        span.className = runClass;
        span.textContent = run;
        frag.append(span);
      } else {
        frag.append(run);
      }
      run = "";
    };
    for (const ch of line.slice(4)) {
      const cls = glyphClass(ch);
      if (cls !== runClass) {
        flush();
        runClass = cls;
      }
      run += ch;
    }
    flush();
    frag.append("\n");
  });
  pre.replaceChildren(frag);
}

function twelveHour(hour: number, minute: number): string {
  const h = hour % 12 === 0 ? 12 : hour % 12;
  return `${h}:${String(minute).padStart(2, "0")}${hour < 12 ? "am" : "pm"}`;
}

function draw(): void {
  if (sim.tick === drawnTick && paused === drawnPaused) return;
  drawnTick = sim.tick;
  drawnPaused = paused;
  const clock = sim.clock();
  const cal = clock.calendar;
  clockEl.textContent = `Y${cal.year} ${cal.season} ${cal.day_of_season} (${cal.weekday}) ${twelveHour(clock.time.hour, clock.time.minute)}`;
  speedEl.textContent = paused ? "paused" : `${SPEEDS[speedIndex]} game-min/s`;
  everyoneEl.textContent = sim.whereIsEveryone();
  for (const [id, pre] of pres) paint(pre, sim.renderPlace(id));
}

// Ticks of catch-up allowed per frame: a hidden tab resumes from where it left off.
const MAX_TICKS_PER_FRAME = 10 * TICKS_PER_MINUTE;
let last = performance.now();
let carry = 0;
function frame(now: number): void {
  const dt = Math.min((now - last) / 1000, 1);
  last = now;
  if (!paused) {
    carry += dt * SPEEDS[speedIndex]! * TICKS_PER_MINUTE;
    const whole = Math.min(Math.floor(carry), MAX_TICKS_PER_FRAME);
    if (whole > 0) {
      carry -= whole;
      gameTick += whole;
      sim.stepUntil(gameTick);
    }
  }
  draw();
  requestAnimationFrame(frame);
}

document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    last = performance.now();
    carry = 0;
  }
});
$("pause").addEventListener("click", () => {
  paused = !paused;
});
$("faster").addEventListener("click", () => {
  speedIndex = Math.min(speedIndex + 1, SPEEDS.length - 1);
  drawnTick = -1;
});
$("slower").addEventListener("click", () => {
  speedIndex = Math.max(speedIndex - 1, 0);
  drawnTick = -1;
});

// For Playwright and the agent: a structured dump of what is on screen.
declare global {
  interface Window {
    __scene: () => { tick: number; clock: string; everyone: string; hash: string };
  }
}
window.__scene = () => ({ tick: sim.tick, clock: clockEl.textContent ?? "", everyone: sim.whereIsEveryone(), hash: sim.hash() });

draw();
requestAnimationFrame(frame);
