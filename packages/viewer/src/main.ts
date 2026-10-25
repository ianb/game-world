// The viewer: the sim advancing in the browser, drawn as tiles, with the text views the
// tools print behind a toggle (and always in `window.__scene()` for the smoke test).
import * as wasm from "@game-world/sim-wasm";
import { LocalSim, TICKS_PER_MINUTE, at, type Cause, type DaySummary, type MindView, type Sim, type StockView, type SummaryItem, type Topic, type VillagerView } from "@game-world/protocol";
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
/** Topic glosses by id, from the content. */
const glosses = new Map<string, string>();
try {
  const contentJson = await loadContent();
  sim = LocalSim.create(wasm, 1, contentJson);
  for (const t of (JSON.parse(contentJson) as { topics?: { id: string; gloss: string }[] }).topics ?? []) glosses.set(t.id, t.gloss);
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
const canvases: PlaceCanvas[] = [];
// One place on screen at a time, picked by a tab or by following whoever is pinned.
const mapEl = $("map");
const placeTabsEl = $("place-tabs");
const sections = new Map<number, HTMLElement>();
const placeTabs = new Map<number, HTMLButtonElement>();
let activePlace = (world.places.find((p) => p.key === "town") ?? world.places[0])?.id ?? 0;
for (const place of world.places) {
  const section = document.createElement("section");
  const canvas = new PlaceCanvas(tileset, place, world.objects, world.roofs);
  section.append(canvas.canvas);
  section.hidden = place.id !== activePlace;
  placesEl.appendChild(section);
  sections.set(place.id, section);
  canvases.push(canvas);
  const tab = document.createElement("button");
  tab.type = "button";
  tab.setAttribute("role", "tab");
  tab.textContent = place.key;
  tab.addEventListener("click", () => {
    // Choosing a place stops following someone who is elsewhere.
    const p = pinned;
    if (p?.kind === "villager" && sim.villagers().find((v) => v.key === p.key)?.place !== place.id) pinned = null;
    showPlace(place.id);
  });
  placeTabsEl.append(tab);
  placeTabs.set(place.id, tab);
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

const rules = sim.rules();

/** One need as a bar: name, ten blocks, the value, and what the sim makes of it. */
function needBar(name: string, value: number, perHour: number): string {
  const filled = Math.round((value / rules.need_max) * 10);
  const bar = "▮".repeat(filled) + "▯".repeat(10 - filled);
  const word = value >= rules.need_urgent ? "urgent" : value >= rules.need_pressing ? "pressing" : value >= 40 ? "some" : "fine";
  return `${name.padEnd(8)} ${bar} ${String(value).padStart(3)} ${word.padEnd(8)} +${perHour}/h`;
}

function describeNeeds(v: VillagerView): string[] {
  const n = v.needs;
  return [
    needBar("hunger", n.hunger, rules.hunger_per_hour),
    needBar("thirst", n.thirst, rules.thirst_per_hour),
    needBar("sleep", n.sleep, rules.sleep_per_hour),
    needBar("bladder", n.bladder, rules.bladder_per_hour),
    `scale 0 to ${rules.need_max}; acts at ${rules.need_pressing}, urgent at ${rules.need_urgent}`,
    `+/h is per hour awake, roughly; asleep, sleep repays ${rules.sleep_recovery_per_hour}/h`,
  ];
}

/** A topic's gloss and who it concerns: "money (Mara)", "sweet on (Hal, Mara)". */
function topicText(t: Topic, name: (id: number) => string): string {
  const who = [t.about, t.with].filter((v): v is number => v !== null).map(name);
  return `${glosses.get(t.key) ?? t.key}${who.length ? ` (${who.join(", ")})` : ""}`;
}

/** Why someone is doing something, in a few words. */
function causeText(cause: Cause | null, villagers: VillagerView[]): string {
  if (cause === null) return "";
  if (cause === "Schedule") return "the schedule";
  if (cause === "Plan") return "their own plan for the day";
  if (cause === "Return") return "back from an errand";
  if (cause === "Sim") return "the town's rules";
  if ("Reaction" in cause) return `reacting to: ${cause.Reaction.trigger}`;
  if ("Knew" in cause) {
    const { from, topic, heard_at } = cause.Knew;
    const name = (id: number): string => villagers[id]?.name ?? "?";
    return `${name(from)} said ${topicText(topic, name)} at ${sinceText(heard_at)}`;
  }
  return `outside input #${cause.Input.seq}`;
}

function sinceText(tick: number): string {
  const t = (tick % (24 * 60 * TICKS_PER_MINUTE)) / TICKS_PER_MINUTE;
  return twelveHour(Math.floor(t / 60), Math.floor(t % 60));
}

const signed = (n: number): string => (n > 0 ? `+${n}` : `${n}`);
/** "SmallTalk" as "small talk". */
const words = (name: string): string => name.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();

/** Persona, mood, how they see the people they have talked to, and recent memory. */
function describeMind(m: MindView, villagers: VillagerView[]): string[] {
  const name = (id: number): string => villagers[id]?.name ?? "?";
  const lines = [
    "",
    `sociable ${signed(m.sociable)}, warmth ${signed(m.warmth)}, charity ${signed(m.charity)}`,
    `skill ${m.skill}/10, rejection sensitivity ${m.rejection_sensitivity}/10`,
    `mood ${signed(m.mood)}${m.worry ? `; on their mind: ${m.worry}` : ""}`,
  ];
  for (const r of m.regard) {
    lines.push(`sees ${name(r.with)}: ${r.familiarity} talks, feeling ${signed(r.feeling)}, seemed ${signed(r.last_seemed)} last time`);
  }
  if (m.episodes.length) lines.push("recent talks:");
  for (const e of m.episodes) {
    const notes = e.notes.length ? ` (${e.notes.join("; ")})` : "";
    lines.push(`  ${sinceText(e.tick)} ${name(e.with)}, ${words(e.frame)}: peak ${signed(e.peak)}, end ${signed(e.end)}${notes}`);
  }
  if (m.thoughts.length) lines.push("thought:");
  for (const { tick, thought: t } of m.thoughts) {
    const blame =
      t.blame === "Me" ? "blames themself" : t.blame === "World" ? "blames the world" : t.blame === "Luck" ? "puts it down to luck" : `blames ${name(t.blame.Them)}`;
    const span = [t.lasting ? "for good" : "", t.everywhere ? "for everything" : ""].filter(Boolean).join(" and ");
    const about = topicText(t.about, name);
    lines.push(`  ${sinceText(tick)} ${blame}${span ? `, ${span}` : ""}: ${about}${t.aloud ? ", aloud" : ""}`);
  }
  if (m.heard.length) lines.push("heard:");
  for (const h of m.heard) {
    const to = h.caught === "Addressed" ? "" : h.to === null ? ", overheard from afar" : ` to ${name(h.to)}, overheard`;
    const what = h.topic === null ? "" : ` "${h.topic}"`;
    lines.push(`  ${sinceText(h.tick)} ${name(h.speaker)}${to}: ${words(h.gesture)}${what}${h.about_me ? " (about me)" : ""}`);
  }
  return lines;
}

function describeVillager(v: VillagerView, stock: StockView[]): string[] {
  const place = world.places.find((p) => p.id === v.place);
  const lines = [
    `${v.name}`,
    `${v.activity}${v.heading_for ? ` (heading for ${v.heading_for})` : ""} since ${sinceText(v.since)}`,
    `because: ${causeText(v.cause, sim.villagers())}`,
    `at ${place?.key ?? "?"} (${v.pos.x}, ${v.pos.y})`,
    `${v.coins} coins`,
    ...describeNeeds(v),
  ];
  const mind = sim.mind(v.key);
  if (mind) lines.push(...describeMind(mind, sim.villagers()));
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
/** One row per villager, made once and updated in place so nothing jumps: a name button,
 * then the activity and the place in fixed columns. */
const rows = new Map<string, { button: HTMLButtonElement; activity: HTMLSpanElement; place: HTMLSpanElement }>();
function listEveryone(villagers: VillagerView[]): void {
  for (const v of villagers) {
    let row = rows.get(v.key);
    if (!row) {
      const button = document.createElement("button");
      button.className = "name";
      button.type = "button";
      button.textContent = v.name;
      button.addEventListener("click", () => {
        pinned = pinned?.kind === "villager" && pinned.key === v.key ? null : { kind: "villager", key: v.key };
        drawnTick = -1;
      });
      const activity = document.createElement("span");
      const place = document.createElement("span");
      place.className = "muted";
      everyoneEl.append(button, activity, place);
      row = { button, activity, place };
      rows.set(v.key, row);
    }
    const placeName = world.places.find((p) => p.id === v.place)?.key ?? "?";
    const activity = `${v.activity}${v.heading_for ? ` → ${v.heading_for}` : ""}`;
    if (row.activity.textContent !== activity) row.activity.textContent = activity;
    if (row.place.textContent !== placeName) row.place.textContent = placeName;
    const pressed = pinned?.kind === "villager" && pinned.key === v.key ? "true" : "false";
    if (row.button.getAttribute("aria-pressed") !== pressed) row.button.setAttribute("aria-pressed", pressed);
  }
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

function showPlace(id: number): void {
  activePlace = id;
  for (const [k, section] of sections) section.hidden = k !== id;
  for (const [k, tab] of placeTabs) tab.setAttribute("aria-selected", String(k === id));
  applyZoom();
  drawnTick = -1;
}

// Zoom: the steps keep pixels even; `null` fits the whole place into the map pane.
const ZOOMS = [0.5, 1, 2, 3];
let zoom: number | null = null;
const narrow = window.matchMedia("(max-width: 900px)");
function fitZoom(c: PlaceCanvas): number {
  const w = mapEl.clientWidth || window.innerWidth;
  // Stacked on a narrow screen, the map pane takes its height from the map: fit the width.
  if (narrow.matches) return Math.min(3, w / c.canvas.width);
  const h = mapEl.clientHeight || window.innerHeight;
  return Math.min(3, w / c.canvas.width, h / c.canvas.height);
}
function activeCanvas(): PlaceCanvas | undefined {
  return canvases.find((c) => c.id === activePlace);
}
function currentZoom(): number {
  const c = activeCanvas();
  return zoom ?? (c ? fitZoom(c) : 1);
}
function applyZoom(): void {
  for (const c of canvases) c.scale(zoom ?? fitZoom(c));
}
$("zoom-in").addEventListener("click", () => {
  zoom = ZOOMS.find((z) => z > currentZoom()) ?? ZOOMS[ZOOMS.length - 1]!;
  applyZoom();
});
$("zoom-out").addEventListener("click", () => {
  zoom = [...ZOOMS].reverse().find((z) => z < currentZoom()) ?? ZOOMS[0]!;
  applyZoom();
});
$("zoom-fit").addEventListener("click", () => {
  zoom = null;
  applyZoom();
});
for (const [k, tab] of placeTabs) tab.setAttribute("aria-selected", String(k === activePlace));
applyZoom();
window.addEventListener("resize", applyZoom);

/** Keeps a followed villager in view: scrolls the map when they get near its edge. */
function keepInView(v: VillagerView): void {
  const c = activeCanvas();
  if (!c || c.id !== v.place) return;
  const z = c.canvas.clientWidth / c.canvas.width;
  const px = (v.pos.x + 0.5) * tileset.size * z;
  const py = (v.pos.y + 0.5) * tileset.size * z;
  const margin = 3 * tileset.size * z;
  const { scrollLeft, scrollTop, clientWidth, clientHeight } = mapEl;
  if (px < scrollLeft + margin || px > scrollLeft + clientWidth - margin) mapEl.scrollLeft = px - clientWidth / 2;
  if (py < scrollTop + margin || py > scrollTop + clientHeight - margin) mapEl.scrollTop = py - clientHeight / 2;
}

/** Place tabs show how many people are in each place. */
function labelPlaces(villagers: VillagerView[]): void {
  for (const place of world.places) {
    const n = villagers.filter((v) => v.place === place.id).length;
    const text = n ? `${place.key} (${n})` : place.key;
    const tab = placeTabs.get(place.id);
    if (tab && tab.textContent !== text) tab.textContent = text;
  }
}

// The sidebar's tabs.
const sideTabs = Array.from(document.querySelectorAll<HTMLButtonElement>("#side-tabs button"));
for (const b of sideTabs) {
  b.addEventListener("click", () => {
    for (const o of sideTabs) {
      const on = o === b;
      o.setAttribute("aria-selected", String(on));
      $(`tab-${o.dataset.tab ?? ""}`).hidden = !on;
    }
  });
}

// Game-minutes per real second. 6 means a day takes four real minutes.
const SPEEDS = [1, 2, 6, 12, 30, 60, 120, 240];
let speedIndex = 2;
const speedSelect = speedEl as HTMLSelectElement;
for (const [i, s] of SPEEDS.entries()) {
  const day = (24 * 60) / s;
  const length = day >= 60 ? `${day / 60}m` : `${day}s`;
  speedSelect.add(new Option(`${s} min/s, a day in ${length}`, String(i), i === speedIndex, i === speedIndex));
}
let paused = false;
let gameTick = at(0, 6);
sim.stepUntil(gameTick);
let drawnTick = -1;
let drawnPaused = !paused;

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
  pauseEl.textContent = paused ? "resume" : "pause";
  const past = sim.tick < sim.frontier;
  backEl.hidden = !past;
  clockEl.classList.toggle("past", past);
  listDays();
  const villagers = sim.villagers();
  listEveryone(villagers);
  labelPlaces(villagers);
  // Follow whoever is pinned, into whatever place they go.
  const p = pinned;
  const followed = p?.kind === "villager" ? villagers.find((v) => v.key === p.key) : undefined;
  if (followed && followed.place !== activePlace) showPlace(followed.place);
  for (const c of canvases) if (c.id === activePlace) c.draw(sim, villagers, highlightFor(c.id, villagers));
  if (followed) keepInView(followed);
  inspect(villagers);
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
const pauseEl = $("pause");
pauseEl.addEventListener("click", () => {
  paused = !paused;
});
speedSelect.addEventListener("change", () => {
  speedIndex = Number(speedSelect.value);
  drawnTick = -1;
});
// Skips to the next 6am: later today if it is still night, else tomorrow.
$("morning").addEventListener("click", () => {
  const day = 24 * 60 * TICKS_PER_MINUTE;
  let target = Math.floor(sim.tick / day) * day + 6 * 60 * TICKS_PER_MINUTE;
  if (target <= sim.tick) target += day;
  gameTick = target;
  sim.stepUntil(gameTick);
  paused = false;
  drawnTick = -1;
});

// The days panel: one finished day at a time, with buttons to page through, news first and
// routine folded away. Clicking a line replays the town to that moment (from the day's
// snapshot) and pins whoever it is about.
const daysEl = $("days");
const dayTitleEl = $("day-title");
const dayPrev = $("day-prev") as HTMLButtonElement;
const dayNext = $("day-next") as HTMLButtonElement;
const backEl = $("back") as HTMLButtonElement;
/** The day shown, by index; `null` keeps up with the latest. */
let dayIndex: number | null = null;
let shownDays = "";
function pageDays(step: number): void {
  const n = sim.summaries().length;
  if (n === 0) return;
  const i = Math.min(n - 1, Math.max(0, (dayIndex ?? n - 1) + step));
  dayIndex = i === n - 1 ? null : i;
  listDays();
}
dayPrev.addEventListener("click", () => pageDays(-1));
dayNext.addEventListener("click", () => pageDays(1));

function dayTitle(d: DaySummary): string {
  const c = d.calendar;
  return `Y${c.year} ${c.season} ${c.day_of_season} (${c.weekday})`;
}

function itemButton(item: SummaryItem): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = "moment";
  const t = (item.tick % (24 * 60 * TICKS_PER_MINUTE)) / TICKS_PER_MINUTE;
  const when = document.createElement("span");
  when.className = "when";
  when.textContent = twelveHour(Math.floor(t / 60), Math.floor(t % 60));
  const what = document.createElement("span");
  what.textContent = item.text;
  b.append(when, what);
  const why = causeText(item.cause, sim.villagers());
  b.title = why ? `because: ${why}. Click to replay this moment.` : "Click to replay this moment.";
  b.addEventListener("click", () => goTo(item));
  return b;
}

function goTo(item: SummaryItem): void {
  sim.rewind(item.tick);
  gameTick = item.tick;
  paused = true;
  if (item.who !== null) {
    const v = sim.villagers().find((x) => x.id === item.who);
    if (v) pinned = { kind: "villager", key: v.key };
  }
  drawnTick = -1;
}

function listDays(): void {
  const days = sim.summaries();
  const n = days.length;
  const i = dayIndex ?? n - 1;
  const key = `${n}:${i}`;
  if (key === shownDays) return;
  shownDays = key;
  dayPrev.disabled = i <= 0;
  dayNext.disabled = i >= n - 1;
  const d = days[i];
  if (!d) {
    dayTitleEl.textContent = "no days yet";
    const p = document.createElement("p");
    p.className = "muted";
    p.textContent = "A summary appears when the first day ends (try to 6am).";
    daysEl.replaceChildren(p);
    return;
  }
  dayTitleEl.textContent = `${dayTitle(d)}, day ${i + 1} of ${n}`;
  const frag = document.createDocumentFragment();
  const head = document.createElement("p");
  head.className = "muted";
  head.textContent = d.news.length ? `${d.news.length} unusual` : "nothing unusual";
  frag.append(head, ...d.news.map(itemButton));
  if (d.routine.length > 0) {
    const more = document.createElement("details");
    const label = document.createElement("summary");
    label.textContent = `${d.routine.length} routine`;
    more.append(label, ...d.routine.map(itemButton));
    frag.append(more);
  }
  daysEl.replaceChildren(frag);
  daysEl.parentElement?.scrollTo({ top: 0 });
}

backEl.addEventListener("click", () => {
  sim.backToNow();
  gameTick = sim.tick;
  drawnTick = -1;
});

// For Playwright and the agent: a structured dump of what is on screen.
declare global {
  interface Window {
    __scene: () => { tick: number; clock: string; everyone: string; hash: string; places: string[]; canvases: number; days: number };
    /** Runs the sim forward by whole days and redraws, for screenshots. */
    __advance: (days: number) => void;
    /** Puts a place on screen and redraws; returns its canvas as a PNG data URL. */
    __show: (key: string) => string | null;
  }
}
window.__advance = (days: number) => {
  paused = true;
  gameTick += Math.max(0, Math.floor(days)) * 24 * 60 * TICKS_PER_MINUTE;
  sim.stepUntil(gameTick);
  drawnTick = -1;
  draw();
};
window.__show = (key: string) => {
  const place = world.places.find((p) => p.key === key);
  if (!place) return null;
  pinned = null;
  showPlace(place.id);
  draw();
  return activeCanvas()?.canvas.toDataURL("image/png") ?? null;
};
window.__scene = () => ({
  tick: sim.tick,
  clock: clockEl.textContent ?? "",
  everyone: sim.whereIsEveryone(),
  hash: sim.hash(),
  places: sim.placeIds().map((id) => sim.renderPlace(id)),
  canvases: canvases.filter((c) => c.canvas.width > 0).length,
  days: sim.summaries().length,
});

draw();
requestAnimationFrame(frame);
