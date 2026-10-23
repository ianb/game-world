// The viewer: the sim advancing in the browser, drawn as tiles, with the text views the
// tools print behind a toggle (and always in `window.__scene()` for the smoke test).
import * as wasm from "@game-world/sim-wasm";
import { LocalSim, TICKS_PER_MINUTE, at, type Sim, type StockView, type VillagerView } from "@game-world/protocol";
import { PlaceCanvas, Tileset, legend } from "./render";

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

let tileset: Tileset;
try {
  tileset = await Tileset.load(document.baseURI);
} catch (e) {
  clockEl.textContent = `failed to load tiles: ${String(e)}`;
  throw e;
}

const world = sim.world();
const pres = new Map<string, HTMLPreElement>();
const canvases: PlaceCanvas[] = [];
let showText = false;
for (const place of world.places) {
  const section = document.createElement("section");
  const h = document.createElement("h2");
  h.textContent = place.key;
  const canvas = new PlaceCanvas(tileset, place, world.objects, world.roofs);
  const pre = document.createElement("pre");
  pre.hidden = !showText;
  section.append(h, canvas.canvas, pre);
  placesEl.appendChild(section);
  pres.set(place.key, pre);
  canvases.push(canvas);
  canvas.canvas.addEventListener("pointermove", (e) => {
    const t = canvas.tileAt(e.clientX, e.clientY);
    hover = t ? { kind: "tile", place: canvas.id, x: t.x, y: t.y } : null;
    drawnTick = -1;
  });
  canvas.canvas.addEventListener("pointerleave", () => {
    hover = null;
    drawnTick = -1;
  });
  canvas.canvas.addEventListener("click", (e) => {
    const t = canvas.tileAt(e.clientX, e.clientY);
    if (!t) return;
    // Clicking a villager pins the person; clicking anything else pins the tile.
    const v = sim.villagers().find((v) => v.place === canvas.id && v.pos.x === t.x && v.pos.y === t.y);
    pinned = v ? { kind: "villager", key: v.key } : { kind: "tile", place: canvas.id, x: t.x, y: t.y };
    drawnTick = -1;
  });
}

// What the inspector shows: the pinned thing, else whatever the pointer is over.
type Focus = { kind: "tile"; place: number; x: number; y: number } | { kind: "villager"; key: string };
let hover: Focus | null = null;
let pinned: Focus | null = null;
const inspectEl = $("inspect");

function describeNeeds(v: VillagerView): string {
  const n = v.needs;
  const word = (x: number): string => (x >= 90 ? "urgent" : x >= 70 ? "pressing" : x >= 40 ? "some" : "fine");
  return `hunger ${n.hunger} (${word(n.hunger)}), thirst ${n.thirst} (${word(n.thirst)}), sleep ${n.sleep} (${word(n.sleep)}), bladder ${n.bladder} (${word(n.bladder)})`;
}

function sinceText(tick: number): string {
  const t = (tick % (24 * 60 * TICKS_PER_MINUTE)) / TICKS_PER_MINUTE;
  return twelveHour(Math.floor(t / 60), Math.floor(t % 60));
}

function describeVillager(v: VillagerView, stock: StockView[]): string[] {
  const place = world.places.find((p) => p.id === v.place);
  const lines = [
    `${v.name}`,
    `${v.activity}${v.heading_for ? ` (heading for ${v.heading_for})` : ""} since ${sinceText(v.since)}`,
    `at ${place?.key ?? "?"} (${v.pos.x}, ${v.pos.y})`,
    `${v.coins} coins`,
    describeNeeds(v),
  ];
  void stock;
  return lines;
}

function describeTile(f: { place: number; x: number; y: number }, villagers: VillagerView[], stock: StockView[]): string[] {
  const place = world.places.find((p) => p.id === f.place);
  if (!place) return ["?"];
  const i = f.y * place.width + f.x;
  const cell = place.cells[i] ?? "Void";
  const lines = [`${place.key} (${f.x}, ${f.y})`];
  if (place.kind === "Zone") {
    const t = sim.terrain(place.key);
    const surface = t.tiles[i];
    const cost = t.cost[i] ?? 0;
    lines.push(`${surface ?? cell}${cost ? `, step cost ${cost}` : ", impassable"}`);
    if (surface) lines.push(`wear ${t.wear[i] ?? 0}, growth ${t.growth[i] ?? 0}`);
  } else {
    lines.push(cell);
  }
  for (const o of world.objects) {
    if (o.place !== f.place || o.pos.x !== f.x || o.pos.y !== f.y) continue;
    const goods = stock.filter((s) => s.key === o.key).map((s) => `${s.good} ${s.count}`);
    lines.push(`${o.kind} ${o.key}${o.room ? ` in ${o.building}/${o.room}` : ""}${goods.length ? `: ${goods.join(", ")}` : ""}`);
  }
  for (const v of villagers) {
    if (v.place === f.place && v.pos.x === f.x && v.pos.y === f.y) lines.push(...describeVillager(v, stock));
  }
  return lines;
}

function inspect(villagers: VillagerView[]): void {
  const focus = pinned ?? hover;
  if (!focus) {
    inspectEl.textContent = "hover or click a tile; click a name for the person";
    return;
  }
  const stock = sim.economy();
  const lines =
    focus.kind === "villager"
      ? (() => {
          const v = villagers.find((x) => x.key === focus.key);
          return v ? describeVillager(v, stock) : ["?"];
        })()
      : describeTile(focus, villagers, stock);
  inspectEl.textContent = (pinned ? "pinned: " : "") + lines.join("\n");
}

/** Where to draw the highlight on a place, from the focus. */
function highlightFor(placeId: number, villagers: VillagerView[]): { x: number; y: number } | null {
  const focus = pinned ?? hover;
  if (!focus) return null;
  if (focus.kind === "tile") return focus.place === placeId ? { x: focus.x, y: focus.y } : null;
  const v = villagers.find((x) => x.key === focus.key);
  return v && v.place === placeId ? { x: v.pos.x, y: v.pos.y } : null;
}

/** The who-is-where list as buttons, one per villager, so a name can be clicked. */
function listEveryone(villagers: VillagerView[]): void {
  const frag = document.createDocumentFragment();
  for (const v of villagers) {
    const b = document.createElement("button");
    b.className = "name";
    b.type = "button";
    const place = world.places.find((p) => p.id === v.place);
    b.textContent = `${v.name}: ${v.activity}${v.heading_for ? ` → ${v.heading_for}` : ""} at ${place?.key ?? "?"}`;
    if (pinned?.kind === "villager" && pinned.key === v.key) b.setAttribute("aria-pressed", "true");
    b.addEventListener("click", () => {
      pinned = pinned?.kind === "villager" && pinned.key === v.key ? null : { kind: "villager", key: v.key };
      drawnTick = -1;
    });
    frag.append(b);
  }
  everyoneEl.replaceChildren(frag);
}
// What the ground can look like, so cowpaths and overgrowth are recognisable when they
// appear. Grass wears to a track and a cowpath where villagers walk; untrodden grass goes
// tall and then to brush; a laid path goes weedy and then overgrown.
const legendEl = $("legend");
legendEl.append(
  legend(tileset, [
    ["grass", "grass"],
    ["track", "track"],
    ["cowpath", "cowpath"],
    ["path", "road"],
    ["weedy_path", "weedy"],
    ["overgrown_path", "overgrown"],
    ["tall_grass", "tall grass"],
    ["brush", "brush"],
    ["water", "water"],
  ]),
);

function fitCanvases(): void {
  const width = placesEl.clientWidth || window.innerWidth;
  for (const c of canvases) c.fit(width - 8);
}
fitCanvases();
window.addEventListener("resize", fitCanvases);

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
  const villagers = sim.villagers();
  listEveryone(villagers);
  for (const c of canvases) c.draw(sim, villagers, highlightFor(c.id, villagers));
  inspect(villagers);
  if (showText) for (const [id, pre] of pres) paint(pre, sim.renderPlace(id));
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
$("day").addEventListener("click", () => {
  window.__advance(1);
  paused = false;
  drawnTick = -1;
});
$("text").addEventListener("click", () => {
  showText = !showText;
  for (const pre of pres.values()) pre.hidden = !showText;
  drawnTick = -1;
});

// For Playwright and the agent: a structured dump of what is on screen.
declare global {
  interface Window {
    __scene: () => { tick: number; clock: string; everyone: string; hash: string; places: string[]; canvases: number };
    /** Runs the sim forward by whole days and redraws, for screenshots. */
    __advance: (days: number) => void;
  }
}
window.__advance = (days: number) => {
  paused = true;
  gameTick += Math.max(0, Math.floor(days)) * 24 * 60 * TICKS_PER_MINUTE;
  sim.stepUntil(gameTick);
  drawnTick = -1;
  draw();
};
window.__scene = () => ({
  tick: sim.tick,
  clock: clockEl.textContent ?? "",
  everyone: sim.whereIsEveryone(),
  hash: sim.hash(),
  places: sim.placeIds().map((id) => sim.renderPlace(id)),
  canvases: canvases.filter((c) => c.canvas.width > 0).length,
});

draw();
requestAnimationFrame(frame);
