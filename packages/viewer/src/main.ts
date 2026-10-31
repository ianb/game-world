// The viewer: the sim advancing in the browser, drawn as tiles, with the text views the
// tools print behind a toggle (and always in `window.__scene()` for the smoke test).
import "@fontsource-variable/nunito";
import * as wasm from "@game-world/sim-wasm";
import { LocalSim, TICKS_PER_MINUTE, at, type Cause, type ClockView, type Emitted, type DaySummary, type MindView, type ObjectView, type Sim, type StockView, type SummaryItem, type Topic, type TrainTimeView, type VillagerView, type FieldView, type FieldTileView } from "@game-world/protocol";
import { PlaceCanvas, Tileset, legend, objectTile, trainCars, type View } from "./render";
import { Words, type TopicDef, type Word } from "./words";
import type { Diorama } from "./diorama";

const $ = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;
const clockEl = $("clock");
const everyoneEl = $("everyone");

/** The world to open: `?world=<id>`, else the last one opened, else the first built. */
function chooseWorld(): string {
  const built = wasm.worlds();
  const asked = new URLSearchParams(location.search).get("world");
  let last: string | null = null;
  try {
    last = localStorage.getItem("world");
  } catch {
    // storage can be blocked; fall through to the default
  }
  const id = [asked, last].find((w): w is string => w !== null && built.includes(w)) ?? built[0]!;
  try {
    localStorage.setItem("world", id);
  } catch {
    // as above
  }
  return id;
}

const worldId = chooseWorld();

async function loadContent(): Promise<string> {
  const res = await fetch(new URL(`worlds/${worldId}/content.json`, document.baseURI));
  if (!res.ok) throw new Error(`worlds/${worldId}/content.json: HTTP ${res.status}`);
  return res.text();
}

let sim: Sim;
// The 3D view's state; it is built in setView3d.
let diorama: Diorama | null = null;
let view3d = false;
let followedIn3d: string | null = null;
let pendingShot: string[] | null = null;
let shownClock: ClockView | null = null;
/** Topic glosses by id, from the content. */
const glosses = new Map<string, string>();
let topicDefs: TopicDef[] = [];
/** The railway's timetable and its station objects, from the content, for the station's
 * card. When the trains come is asked of the sim. */
interface RailwaySpec {
  platform: string;
  shed: string;
  runs: { days: string; arrive: string; depart?: string; stops?: boolean }[];
}
let railway: RailwaySpec | null = null;
try {
  const contentJson = await loadContent();
  sim = LocalSim.create(wasm, worldId, 1, contentJson);
  const parsed = JSON.parse(contentJson) as { topics?: TopicDef[]; zones?: { railway?: RailwaySpec | null }[] };
  topicDefs = parsed.topics ?? [];
  railway = parsed.zones?.find((z) => z.railway)?.railway ?? null;
  for (const t of topicDefs) glosses.set(t.id, t.gloss);
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
const mapCanvas = $<HTMLCanvasElement>("map-canvas");
const mapCtx = mapCanvas.getContext("2d") ?? (() => { throw new Error("no 2d canvas"); })();
const placeTabsEl = $("place-tabs");
const placeTabs = new Map<number, HTMLButtonElement>();
/** A place's key as a person would say it: "ashby_barn/loft" is "Ashby barn loft". */
const placeName = (key: string): string => {
  const t = key.replace(/[_/]+/g, " ");
  return t.charAt(0).toUpperCase() + t.slice(1);
};
let activePlace = (world.places.find((p) => p.key === "town") ?? world.places[0])?.id ?? 0;
for (const place of world.places) {
  canvases.push(new PlaceCanvas(tileset, place, world));
  const tab = document.createElement("button");
  tab.type = "button";
  tab.setAttribute("role", "tab");
  tab.textContent = placeName(place.key);
  tab.addEventListener("click", () => {
    // Choosing a place stops following someone who is elsewhere.
    const p = pinned;
    if (p?.kind === "villager" && sim.villagers().find((v) => v.key === p.key)?.place !== place.id) pinned = null;
    showPlace(place.id);
  });
  placeTabsEl.append(tab);
  placeTabs.set(place.id, tab);
}

/** The tile of the shown place under a pointer, or null. */
function tileUnder(e: MouseEvent): { canvas: PlaceCanvas; x: number; y: number } | null {
  const canvas = activeCanvas();
  const view = currentView();
  if (!canvas || !view) return null;
  const r = mapEl.getBoundingClientRect();
  const t = canvas.tileAt(e.clientX - r.left, e.clientY - r.top, view);
  return t ? { canvas, ...t } : null;
}
mapCanvas.addEventListener("pointermove", (e) => {
  const t = tileUnder(e);
  const next: Focus | null = t ? { kind: "tile", place: t.canvas.id, x: t.x, y: t.y } : null;
  if (next && hover && samePin(hover, next)) return;
  hover = next;
  drawnTick = -1;
});
mapCanvas.addEventListener("pointerleave", () => {
  hover = null;
  drawnTick = -1;
});
mapCanvas.addEventListener("click", (e) => {
  const t = tileUnder(e);
  if (!t) return;
  // Clicking a villager pins the person, an object (anywhere on its sprite) the object,
  // anything else the tile. Clicking what is already pinned lets it go.
  const v = sim.villagers().find((v) => v.place === t.canvas.id && v.pos.x === t.x && v.pos.y === t.y);
  const o = v ? undefined : objectAt(t.canvas.id, t.x, t.y);
  const next: Focus = v ? { kind: "villager", key: v.key } : o ? { kind: "object", key: o.key } : { kind: "tile", place: t.canvas.id, x: t.x, y: t.y };
  pinned = samePin(pinned, next) ? null : next;
  drawnTick = -1;
});

// What the inspector shows: the pinned thing, else whatever the pointer is over.
type Focus = { kind: "tile"; place: number; x: number; y: number } | { kind: "villager"; key: string } | { kind: "object"; key: string };
let hover: Focus | null = null;
let pinned: Focus | null = null;

function samePin(a: Focus | null, b: Focus): boolean {
  if (!a || a.kind !== b.kind) return false;
  if (a.kind === "tile" && b.kind === "tile") return a.place === b.place && a.x === b.x && a.y === b.y;
  return "key" in a && "key" in b && a.key === b.key;
}

/** The object whose sprite covers a tile: one standing on it first, else the nearest in
 * front. */
function objectAt(place: number, x: number, y: number): ObjectView | undefined {
  let best: ObjectView | undefined;
  for (const o of world.objects) {
    if (o.place !== place) continue;
    if (o.pos.x === x && o.pos.y === y) return o;
    const r = tileset.covers(objectTile(o.kind), o.pos.x, o.pos.y);
    if (x >= r.x0 && x <= r.x1 && y >= r.y0 && y <= r.y1 && (!best || o.pos.y > best.pos.y)) best = o;
  }
  return best;
}

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && pinned) {
    pinned = null;
    drawnTick = -1;
  }
  // Space pauses, unless it is typing into a field or pressing a button.
  const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement || e.target instanceof HTMLButtonElement;
  if (e.key === " " && !typing) {
    e.preventDefault();
    paused = !paused;
  }
  if (e.target instanceof HTMLInputElement) return;
  const n = Number(e.key);
  if (n >= 1 && n <= SPEEDS.length) setSpeed(n - 1);
  if (e.key === "j" || e.key === "J") setJournal(document.body.classList.contains("journal-closed"));
});
const inspectEl = $("inspect");
const inspectorEl = $("inspector");
const hintEl = $("hint");
let hintDone = false;
if (window.matchMedia("(pointer: coarse)").matches) hintEl.textContent = "tap a person or a thing to follow it";

const rules = sim.rules();

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

/** A moment in the coming week: "today 3:00pm", "tomorrow 10:00am", "Fri 10:00am". */
function whenText(tick: number): string {
  const day = 24 * 60 * TICKS_PER_MINUTE;
  const days = Math.floor(tick / day) - Math.floor(sim.tick / day);
  const weekday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][Math.floor(tick / day) % 7] ?? "";
  return `${days === 0 ? "today" : days === 1 ? "tomorrow" : weekday} ${sinceText(tick)}`;
}

function sinceText(tick: number): string {
  const t = (tick % (24 * 60 * TICKS_PER_MINUTE)) / TICKS_PER_MINUTE;
  return twelveHour(Math.floor(t / 60), Math.floor(t % 60));
}

const signed = (n: number): string => (n > 0 ? `+${n}` : `${n}`);
/** "SmallTalk" as "small talk". */
const words = (name: string): string => name.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();
/** "ashby_barn" as "Ashby barn". */
const title = (s: string): string => {
  const t = s.replace(/[_/-]+/g, " ").trim();
  return t.charAt(0).toUpperCase() + t.slice(1);
};

/** The inspector shows a card: a picture, a name, a line under it, and blocks of detail.
 * Text can carry words (topics, gestures) that open the words panel. */
type Seg = string | { word: Word; label: string };
type Block =
  | { kind: "text"; segs: Seg[]; muted?: boolean }
  | { kind: "meter"; label: string; value: number; level: number; note: string }
  | { kind: "pips"; label: string; level: number; max: number; good: boolean; name: string; note: string }
  | { kind: "stats"; items: { label: string; value: string }[] }
  | { kind: "head"; text: string };
interface Card {
  picture: { villager: string } | { tile: string } | null;
  title: string;
  sub: string;
  blocks: Block[];
}
const text = (...segs: Seg[]): Block => ({ kind: "text", segs });
const note = (...segs: Seg[]): Block => ({ kind: "text", segs, muted: true });
const head = (t: string): Block => ({ kind: "head", text: t });

/** A topic chip from a label as views show it ("money (Mara)"): the topic whose gloss it
 * starts with. */
function topicSeg(label: string): Seg {
  let best: string | null = null;
  for (const [id, gloss] of glosses) {
    if (label.startsWith(gloss) && (best === null || gloss.length > (glosses.get(best)?.length ?? 0))) best = id;
  }
  return best ? { word: { kind: "topic", key: best }, label } : label;
}

/** One need as a meter: 0 fine, 1 some, 2 pressing (they act on it), 3 urgent. */
function needMeter(label: string, value: number, perHour: number): Block {
  const level = value >= rules.need_urgent ? 3 : value >= rules.need_pressing ? 2 : value >= 40 ? 1 : 0;
  return { kind: "meter", label, value, level, note: `+${perHour}/h` };
}

function describeNeeds(v: VillagerView): Block[] {
  const n = v.needs;
  return [
    head("Needs"),
    needMeter("Hunger", n.hunger, rules.hunger_per_hour),
    needMeter("Thirst", n.thirst, rules.thirst_per_hour),
    needMeter("Sleep", n.sleep, rules.sleep_per_hour),
    needMeter("Bladder", n.bladder, rules.bladder_per_hour),
    note(`0 to ${rules.need_max}: they act at ${rules.need_pressing}, urgent at ${rules.need_urgent}. Rises per hour awake; asleep, sleep falls ${rules.sleep_recovery_per_hour}/h.`),
  ];
}

/** Persona, mood, how they see the people they have talked to, and recent memory. */
function describeMind(m: MindView, villagers: VillagerView[]): Block[] {
  const name = (id: number): string => villagers[id]?.name ?? "?";
  const blocks: Block[] = [
    head("Character"),
    {
      kind: "stats",
      items: [
        { label: "sociable", value: signed(m.sociable) },
        { label: "warmth", value: signed(m.warmth) },
        { label: "charity", value: signed(m.charity) },
        { label: "skill", value: `${m.skill}/10` },
        { label: "touchy", value: `${m.rejection_sensitivity}/10` },
      ],
    },
  ];
  if (m.worry) blocks.push(text("On their mind: ", topicSeg(m.worry)));
  if (m.regard.length) blocks.push(head("How they see people"));
  for (const r of m.regard) {
    blocks.push(text(`${name(r.with)}: feeling ${signed(r.feeling)} after ${r.familiarity} talks; seemed ${signed(r.last_seemed)} last time`));
  }
  if (m.episodes.length) blocks.push(head("Recent talks"));
  for (const e of m.episodes) {
    const notes = e.notes.length ? ` (${e.notes.join("; ")})` : "";
    blocks.push(text(`${sinceText(e.tick)} · ${name(e.with)}, ${words(e.frame)}: peak ${signed(e.peak)}, end ${signed(e.end)}${notes}`));
  }
  if (m.thoughts.length) blocks.push(head("Thoughts"));
  for (const { tick, thought: t } of m.thoughts) {
    const blame =
      t.blame === "Me" ? "blames themself" : t.blame === "World" ? "blames the world" : t.blame === "Luck" ? "puts it down to luck" : `blames ${name(t.blame.Them)}`;
    const span = [t.lasting ? "for good" : "", t.everywhere ? "for everything" : ""].filter(Boolean).join(" and ");
    const about = topicText(t.about, name);
    blocks.push(text(`${sinceText(tick)} · ${blame}${span ? `, ${span}` : ""}: `, { word: { kind: "topic", key: t.about.key }, label: about }, t.aloud ? ", aloud" : ""));
  }
  if (m.heard.length) blocks.push(head("Heard"));
  for (const h of m.heard) {
    const to = h.caught === "Addressed" ? "" : h.to === null ? ", overheard from afar" : ` to ${name(h.to)}, overheard`;
    const what: Seg[] = h.topic === null ? [] : [" \u201c", topicSeg(h.topic), "\u201d"];
    blocks.push(text(`${sinceText(h.tick)} · ${name(h.speaker)}${to}: `, { word: { kind: "gesture", name: h.gesture }, label: words(h.gesture) }, ...what, h.about_me ? " (about me)" : ""));
  }
  return blocks;
}

function describeVillager(v: VillagerView): Card {
  const place = world.places.find((p) => p.id === v.place);
  const mind = sim.mind(v.key);
  const blocks: Block[] = [
    text(`Because: ${causeText(v.cause, sim.villagers())}`),
    note(`${title(place?.key ?? "?")}, at ${v.pos.x}, ${v.pos.y}`),
    {
      kind: "stats",
      items: [
        { label: "coins", value: String(v.coins) },
        ...(mind ? [{ label: "spirits", value: `${mind.spirits} (${signed(mind.mood)})` }] : []),
      ],
    },
    ...describeNeeds(v),
  ];
  if (mind && mind.moods.length) {
    blocks.push(head("Moods"));
    for (const m of mind.moods) {
      blocks.push({
        kind: "pips",
        label: words(m.mood),
        level: m.level,
        max: m.max,
        good: m.good,
        name: m.name,
        note: m.level > 0 ? `${m.cause}, since ${sinceText(m.since)}` : "",
      });
    }
    blocks.push(note("Each mood runs 0 to 3. Spirits add the good moods, take away the bad ones and the body's discomfort, -3 to 3."));
  }
  if (mind) blocks.push(...describeMind(mind, sim.villagers()));
  return {
    picture: { villager: v.key },
    title: v.name,
    sub: `${v.activity}${v.heading_for ? ` → ${v.heading_for}` : ""} since ${sinceText(v.since)}`,
    blocks,
  };
}

function describeObject(o: ObjectView, villagers: VillagerView[], stock: StockView[]): Card {
  const place = world.places.find((p) => p.id === o.place);
  const owner = o.owner ? villagers.find((v) => v.key === o.owner) : undefined;
  const blocks: Block[] = [];
  if (owner) blocks.push(text(`Belongs to ${owner.name}`));
  if (o.activity) blocks.push(text(`For: ${o.activity}`));
  const goods = stock.filter((s) => s.key === o.key && s.count > 0);
  if (goods.length) blocks.push(head("Holds"), { kind: "stats", items: goods.map((g) => ({ label: g.good, value: String(g.count) })) });
  if (railway && (o.key === railway.platform || o.key === railway.shed)) {
    blocks.push(head("Timetable"));
    for (const r of railway.runs) {
      const stops = r.stops ?? true;
      blocks.push(text(`${title(r.days)}: ${stops ? `stops ${r.arrive}${r.depart ? ` to ${r.depart}` : ""}` : `passes through at ${r.arrive}`}`));
    }
    const next = sim.upcomingTrains()[0];
    if (next) blocks.push(note(`Next: ${whenText(next.arrive)}${next.stops ? "" : ", passing through"}. The ⏭ menu skips to it.`));
  }
  const near = villagers.filter((v) => v.place === o.place && Math.abs(v.pos.x - o.pos.x) <= 1 && Math.abs(v.pos.y - o.pos.y) <= 1);
  if (near.length) blocks.push(head("Here"));
  for (const v of near) blocks.push(text(`${v.name}: ${v.activity}`));
  blocks.push(note(o.key));
  return {
    picture: { tile: objectTile(o.kind) },
    title: title(words(o.kind)),
    sub: `${o.room ? `${title(o.building)}, ${o.room}` : title(place?.key ?? "?")} · ${o.pos.x}, ${o.pos.y}`,
    blocks,
  };
}

/** A named level for a value on a 0..max scale: the first name for 0, the last for max,
 * the rest spread between. */
function level(value: number, max: number, names: string[]): string {
  if (value <= 0) return names[0] ?? "";
  if (value >= max) return names[names.length - 1] ?? "";
  const inner = names.length - 2;
  return names[1 + Math.min(inner - 1, Math.floor(((value - 1) / Math.max(1, max - 1)) * inner))] ?? "";
}

/** The field tile at a spot, from the fields as they were at the current tick. */
let fieldsAt: { tick: number; fields: FieldView[] } | null = null;
function fieldAt(place: number, x: number, y: number): { plot: FieldView; tile: FieldTileView } | null {
  if (!fieldsAt || fieldsAt.tick !== sim.tick) fieldsAt = { tick: sim.tick, fields: sim.fields() };
  for (const plot of fieldsAt.fields) {
    const r = plot.rect;
    if (plot.place !== place || x < r.x || y < r.y || x >= r.x + r.w || y >= r.y + r.h) continue;
    const tile = plot.tiles[(y - r.y) * r.w + (x - r.x)];
    if (tile) return { plot, tile };
  }
  return null;
}

function describeTile(f: { place: number; x: number; y: number }, villagers: VillagerView[], stock: StockView[]): Card {
  const place = world.places.find((p) => p.id === f.place);
  if (!place) return { picture: null, title: "?", sub: "", blocks: [] };
  const i = f.y * place.width + f.x;
  const cell = place.cells[i] ?? "Void";
  const blocks: Block[] = [];
  let ground: string = cell;
  if (place.kind === "Zone") {
    const t = sim.terrain(place.key);
    const surface = t.tiles[i];
    const cost = t.cost[i] ?? 0;
    if (surface) ground = surface;
    blocks.push({
      kind: "stats",
      items: [
        { label: "step cost", value: cost ? String(cost) : "impassable" },
        ...(surface ? [{ label: "wear", value: String(t.wear[i] ?? 0) }, { label: "growth", value: String(t.growth[i] ?? 0) }] : []),
      ],
    });
  }
  const field = fieldAt(f.place, f.x, f.y);
  if (field) {
    const { plot, tile } = field;
    ground = `${plot.crop}, ${words(tile.stage)}`;
    blocks.push(head(title(plot.id)));
    blocks.push({
      kind: "stats",
      items: [
        { label: "grown", value: `${Math.min(tile.growth, plot.grow)} of ${plot.grow}` },
        { label: "water", value: `${level(tile.water, plot.water_full, ["dry", "drying", "damp", "wet"])} (${tile.water} of ${plot.water_full})` },
        { label: "weeds", value: `${level(tile.weeds, plot.weeds_max, ["clean", "a few", "weedy", "choked"])} (${tile.weeds} of ${plot.weeds_max})` },
        { label: "soil", value: `${level(tile.fertility, plot.fertility_max, ["spent", "poor", "fair", "rich"])} (${tile.fertility} of ${plot.fertility_max})` },
      ],
    });
    blocks.push(note("A watered, weeded tile grows 2 a day, a weedy one 1, a dry one wilts back 1. Ripe crops spoil after 5 days."));
  }
  for (const o of world.objects) {
    if (o.place !== f.place || o.pos.x !== f.x || o.pos.y !== f.y) continue;
    const goods = stock.filter((s) => s.key === o.key).map((s) => `${s.good} ${s.count}`);
    blocks.push(text(`${title(words(o.kind))}${o.room ? ` in ${title(o.building)}, ${o.room}` : ""}${goods.length ? `: ${goods.join(", ")}` : ""}`));
  }
  const here = villagers.filter((v) => v.place === f.place && v.pos.x === f.x && v.pos.y === f.y);
  for (const v of here) blocks.push(text(`${v.name}: ${v.activity}`));
  if (here.length) blocks.push(note("Click to follow."));
  return { picture: null, title: title(words(ground)), sub: `${title(place.key)} · ${f.x}, ${f.y}`, blocks };
}

function inspect(villagers: VillagerView[]): void {
  const focus = pinned ?? hover;
  inspectorEl.hidden = !focus;
  hintEl.hidden = !!focus || hintDone;
  if (!focus) {
    shownInspect = "";
    return;
  }
  // The hint has done its job once something has been pinned.
  if (pinned) hintDone = true;
  const stock = sim.economy();
  let card: Card;
  if (focus.kind === "villager") {
    const v = villagers.find((x) => x.key === focus.key);
    card = v ? describeVillager(v) : { picture: null, title: "?", sub: "", blocks: [] };
  } else if (focus.kind === "object") {
    const o = world.objects.find((x) => x.key === focus.key);
    card = o ? describeObject(o, villagers, stock) : { picture: null, title: "?", sub: "", blocks: [] };
  } else {
    card = describeTile(focus, villagers, stock);
  }
  // Rebuilt only when something on it changes, so a word can be clicked between redraws.
  const key = `${pinned ? "pinned" : "hover"}${JSON.stringify(card)}`;
  if (key === shownInspect) return;
  shownInspect = key;
  renderCard(card);
}
let shownInspect = "";

/** Fills the inspector with a card. */
function renderCard(card: Card): void {
  const el = (tag: string, cls: string, content?: string): HTMLElement => {
    const e = document.createElement(tag);
    e.className = cls;
    if (content !== undefined) e.textContent = content;
    return e;
  };
  const top = el("header", "card-top");
  const pic = document.createElement("canvas");
  pic.className = "pic";
  if (card.picture) {
    const src = "villager" in card.picture ? tileset.portrait(card.picture.villager) : tileset.icon(card.picture.tile);
    pic.width = src.width;
    pic.height = src.height;
    pic.getContext("2d")?.drawImage(src, 0, 0);
  } else {
    pic.hidden = true;
  }
  const names = el("div", "names");
  names.append(el("strong", "title", card.title), el("span", "sub", card.sub));
  top.append(pic, names);
  if (pinned) {
    const close = el("button", "close", "×");
    close.setAttribute("aria-label", "let go");
    close.title = "let go (Escape)";
    close.addEventListener("click", () => {
      pinned = null;
      drawnTick = -1;
    });
    top.append(close);
  }
  const body = el("div", "card-body");
  for (const b of card.blocks) {
    if (b.kind === "head") body.append(el("h3", "", b.text));
    else if (b.kind === "stats") {
      const row = el("div", "stats");
      for (const it of b.items) {
        const s = el("span", "stat");
        s.append(el("b", "", it.value), ` ${it.label}`);
        row.append(s);
      }
      body.append(row);
    } else if (b.kind === "pips") {
      const row = el("div", `pips ${b.good ? "good" : "bad"}`);
      const pips = el("span", "dots");
      for (let i = 0; i < b.max; i++) pips.append(el("span", i < b.level ? "on" : ""));
      row.append(el("span", "label", b.label), pips, el("span", "name", b.level > 0 ? b.name : "—"));
      body.append(row);
      if (b.note) body.append(el("p", "muted cause", b.note));
    } else if (b.kind === "meter") {
      const row = el("div", `meter level${b.level}`);
      const bar = el("span", "bar");
      const fill = el("span", "fill");
      fill.style.width = `${Math.min(100, (b.value / rules.need_max) * 100)}%`;
      bar.append(fill);
      row.append(el("span", "label", b.label), bar, el("span", "value", String(b.value)), el("span", "rate", b.note));
      body.append(row);
    } else {
      const p = el("p", b.muted ? "muted" : "");
      for (const s of b.segs) {
        if (typeof s === "string") p.append(s);
        else {
          const chip = wordsPanel.chip(s.word, s.label);
          chip.addEventListener("click", () => selectSideTab("words"));
          p.append(chip);
        }
      }
      body.append(p);
    }
  }
  inspectEl.replaceChildren(top, body);
}

/** Where to draw the highlight on a place, from the focus. */
function highlightFor(placeId: number, villagers: VillagerView[]): { x: number; y: number } | null {
  const focus = pinned ?? hover;
  if (!focus) return null;
  if (focus.kind === "tile") return focus.place === placeId ? { x: focus.x, y: focus.y } : null;
  if (focus.kind === "object") {
    const o = world.objects.find((x) => x.key === focus.key);
    return o && o.place === placeId ? { x: o.pos.x, y: o.pos.y } : null;
  }
  const v = villagers.find((x) => x.key === focus.key);
  return v && v.place === placeId ? { x: v.pos.x, y: v.pos.y } : null;
}

/** The who-is-where list as buttons, one per villager, so a name can be clicked. */
/** One row per villager, made once and updated in place so nothing jumps: a name button,
 * then the activity and the place in fixed columns. */
const rows = new Map<string, { face: HTMLCanvasElement; button: HTMLButtonElement; activity: HTMLSpanElement; place: HTMLSpanElement }>();
let filter = "";
const filterEl = $<HTMLInputElement>("people-filter");
filterEl.addEventListener("input", () => {
  filter = filterEl.value.trim().toLowerCase();
  drawnTick = -1;
});
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
      const face = document.createElement("canvas");
      face.className = "face";
      const src = tileset.portrait(v.key);
      face.width = src.width;
      face.height = src.height;
      face.getContext("2d")?.drawImage(src, 0, 0);
      const activity = document.createElement("span");
      const place = document.createElement("span");
      place.className = "where";
      everyoneEl.append(face, button, activity, place);
      row = { face, button, activity, place };
      rows.set(v.key, row);
    }
    const where = placeName(world.places.find((p) => p.id === v.place)?.key ?? "?");
    const activity = `${v.activity}${v.heading_for ? ` → ${v.heading_for}` : ""}`;
    if (row.activity.textContent !== activity) row.activity.textContent = activity;
    if (row.place.textContent !== where) row.place.textContent = where;
    const hide = filter !== "" && !`${v.name} ${activity} ${where}`.toLowerCase().includes(filter);
    if (row.button.hidden !== hide) row.face.hidden = row.button.hidden = row.activity.hidden = row.place.hidden = hide;
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
  for (const [k, tab] of placeTabs) tab.setAttribute("aria-selected", String(k === id));
  applyCamera();
  drawnTick = -1;
}

function activeCanvas(): PlaceCanvas | undefined {
  return canvases.find((c) => c.id === activePlace);
}

// The camera: each place is drawn at its pixel size and moved and scaled inside the map
// pane. Two-finger scroll or a drag pans; pinch, ctrl+scroll or the buttons zoom. Until
// someone pans or zooms, a place is fitted to the pane.
interface Camera {
  x: number;
  y: number;
  z: number;
  fitted: boolean;
}
const cameras = new Map<number, Camera>();
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 6;
const clampZoom = (z: number): number => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));

function paneSize(): { w: number; h: number } {
  return { w: mapEl.clientWidth || window.innerWidth, h: mapEl.clientHeight || window.innerHeight };
}

function fitted(c: PlaceCanvas): Camera {
  const { w, h } = paneSize();
  const z = clampZoom(Math.min(w / c.width, h / c.height));
  return { x: (w - c.width * z) / 2, y: (h - c.height * z) / 2, z, fitted: true };
}

function camera(): Camera | undefined {
  const c = activeCanvas();
  if (!c) return undefined;
  let cam = cameras.get(c.id);
  if (!cam || cam.fitted) {
    cam = fitted(c);
    cameras.set(c.id, cam);
  }
  return cam;
}

/** Keeps the middle of the pane over the place, so it cannot be lost off screen. */
function applyCamera(): void {
  const c = activeCanvas();
  const cam = camera();
  if (!c || !cam) return;
  const { w, h } = paneSize();
  const cw = c.width * cam.z;
  const ch = c.height * cam.z;
  cam.x = Math.min(w / 2, Math.max(w / 2 - cw, cam.x));
  cam.y = Math.min(h / 2, Math.max(h / 2 - ch, cam.y));
  mapDirty = true;
}

/** The camera as the renderer takes it. */
function currentView(): View | null {
  const cam = camera();
  if (!cam) return null;
  const { w, h } = paneSize();
  return { x: cam.x, y: cam.y, z: cam.z, w, h, dpr: window.devicePixelRatio || 1 };
}

/** Set when the map needs drawing again: the camera moved, the pane resized, or the sim or
 * the focus changed. */
let mapDirty = true;
let lastVillagers: VillagerView[] = [];
function paintMap(): void {
  const c = activeCanvas();
  const view = currentView();
  if (!c || !view) return;
  const pw = Math.round(view.w * view.dpr);
  const ph = Math.round(view.h * view.dpr);
  if (mapCanvas.width !== pw || mapCanvas.height !== ph) {
    mapCanvas.width = pw;
    mapCanvas.height = ph;
  }
  c.draw(mapCtx, view, sim, lastVillagers, highlightFor(c.id, lastVillagers));
  mapDirty = false;
}

// Building names are painted into the cached map with whatever font is ready; paint them
// again once the page's own has loaded.
void document.fonts.ready.then(() => {
  for (const c of canvases) c.invalidate();
  mapDirty = true;
});

/** Zooms by `f` keeping the pane point (px, py) where it is. */
function zoomAt(f: number, px: number, py: number): void {
  const cam = camera();
  if (!cam) return;
  const z = clampZoom(cam.z * f);
  const k = z / cam.z;
  cam.x = px - (px - cam.x) * k;
  cam.y = py - (py - cam.y) * k;
  cam.z = z;
  cam.fitted = false;
  applyCamera();
}

function panBy(dx: number, dy: number): void {
  const cam = camera();
  if (!cam) return;
  // Moving the map yourself stops it following a train.
  if (watching) watching.follow = false;
  cam.x += dx;
  cam.y += dy;
  cam.fitted = false;
  applyCamera();
}

const paneCentre = (): [number, number] => {
  const { w, h } = paneSize();
  return [w / 2, h / 2];
};
$("zoom-in").addEventListener("click", () => (diorama && view3d ? diorama.zoomBy(1 / 1.5) : zoomAt(1.5, ...paneCentre())));
$("zoom-out").addEventListener("click", () => (diorama && view3d ? diorama.zoomBy(1.5) : zoomAt(1 / 1.5, ...paneCentre())));
$("zoom-fit").addEventListener("click", () => {
  if (diorama && view3d) {
    diorama.fit();
    return;
  }
  const c = activeCanvas();
  if (c) cameras.set(c.id, fitted(c));
  applyCamera();
});

// Trackpads send two-finger scrolls as wheel events, and pinches as wheel events with
// ctrlKey (Chrome, Firefox) or as gesture events (Safari).
mapEl.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    const r = mapEl.getBoundingClientRect();
    if (e.ctrlKey || e.metaKey) {
      // A trackpad pinch sends small deltas, a mouse wheel's notches large ones.
      const f = Math.min(1.5, Math.max(1 / 1.5, Math.exp(-e.deltaY * 0.01)));
      zoomAt(f, e.clientX - r.left, e.clientY - r.top);
    } else {
      const k = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? r.height : 1;
      panBy(-e.deltaX * k, -e.deltaY * k);
    }
  },
  { passive: false },
);
let gestureScale = 1;
mapEl.addEventListener("gesturestart", (e) => {
  e.preventDefault();
  gestureScale = 1;
});
mapEl.addEventListener("gesturechange", (e) => {
  e.preventDefault();
  const g = e as unknown as { scale: number; clientX: number; clientY: number };
  const r = mapEl.getBoundingClientRect();
  zoomAt(g.scale / gestureScale, g.clientX - r.left, g.clientY - r.top);
  gestureScale = g.scale;
});

// Dragging pans (a mouse, or one finger); two fingers on a touch screen pinch. A drag is
// not a click, so it does not pin what it started on.
const pointers = new Map<number, { x: number; y: number }>();
let dragged = false;
let pressed: { x: number; y: number } | null = null;
mapEl.addEventListener("pointerdown", (e) => {
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
  if (pointers.size === 1) {
    pressed = { x: e.clientX, y: e.clientY };
    dragged = false;
  }
});
mapEl.addEventListener("pointermove", (e) => {
  const prev = pointers.get(e.pointerId);
  if (!prev) return;
  const now = { x: e.clientX, y: e.clientY };
  if (pointers.size === 1) {
    if (!dragged && pressed && Math.hypot(now.x - pressed.x, now.y - pressed.y) > 4) {
      dragged = true;
      mapEl.setPointerCapture(e.pointerId);
      mapEl.classList.add("dragging");
    }
    if (dragged) panBy(now.x - prev.x, now.y - prev.y);
  } else if (pointers.size === 2) {
    const other = [...pointers].find(([id]) => id !== e.pointerId)?.[1];
    if (other) {
      const r = mapEl.getBoundingClientRect();
      const before = Math.hypot(prev.x - other.x, prev.y - other.y);
      const after = Math.hypot(now.x - other.x, now.y - other.y);
      if (before > 0) zoomAt(after / before, (now.x + other.x) / 2 - r.left, (now.y + other.y) / 2 - r.top);
      panBy((now.x - prev.x) / 2, (now.y - prev.y) / 2);
      dragged = true;
    }
  }
  pointers.set(e.pointerId, now);
});
const release = (e: PointerEvent): void => {
  pointers.delete(e.pointerId);
  if (pointers.size === 0) mapEl.classList.remove("dragging");
};
mapEl.addEventListener("pointerup", release);
mapEl.addEventListener("pointercancel", release);
mapEl.addEventListener(
  "click",
  (e) => {
    if (!dragged) return;
    dragged = false;
    e.stopPropagation();
    e.preventDefault();
  },
  true,
);

for (const [k, tab] of placeTabs) tab.setAttribute("aria-selected", String(k === activePlace));
applyCamera();
window.addEventListener("resize", applyCamera);
new ResizeObserver(applyCamera).observe(mapEl);

/** Keeps a followed villager in view: moves the camera when they get near the edge. */
function keepInView(v: VillagerView): void {
  const c = activeCanvas();
  const cam = camera();
  if (!c || !cam || c.id !== v.place) return;
  const { w, h } = paneSize();
  const s = tileset.size * cam.z;
  const px = cam.x + (v.pos.x + 0.5) * s;
  const py = cam.y + (v.pos.y + 0.5) * s;
  const margin = Math.min(3 * s, w / 4, h / 4);
  let moved = false;
  if (px < margin || px > w - margin) {
    cam.x = w / 2 - (v.pos.x + 0.5) * s;
    moved = true;
  }
  if (py < margin || py > h - margin) {
    cam.y = h / 2 - (v.pos.y + 0.5) * s;
    moved = true;
  }
  if (moved) applyCamera();
}

/** Place tabs show how many people are in each place. */
function labelPlaces(villagers: VillagerView[]): void {
  for (const place of world.places) {
    const n = villagers.filter((v) => v.place === place.id).length;
    const tab = placeTabs.get(place.id);
    if (!tab) continue;
    const text = `${placeName(place.key)}${n ? "*" : ""}`;
    if (tab.textContent !== text) {
      tab.textContent = text;
      tab.title = n ? `${n} ${n === 1 ? "person" : "people"} here` : "nobody here";
    }
  }
}

// The sidebar's tabs.
const sideTabs = Array.from(document.querySelectorAll<HTMLButtonElement>("#side-tabs button"));
function selectSideTab(name: string): void {
  for (const o of sideTabs) {
    const on = o.dataset.tab === name;
    o.setAttribute("aria-selected", String(on));
    $(`tab-${o.dataset.tab ?? ""}`).hidden = !on;
  }
  if (name === "words") wordsPanel.render(true);
}
for (const b of sideTabs) {
  b.addEventListener("click", () => {
    const closed = document.body.classList.contains("journal-closed");
    // On a phone the open tab, tapped again, folds the sheet away.
    if (!closed && b.getAttribute("aria-selected") === "true" && window.matchMedia("(max-width: 760px)").matches) {
      setJournal(false);
      return;
    }
    selectSideTab(b.dataset.tab ?? "people");
    if (closed) setJournal(true);
  });
}

// Game-minutes per real second, one button each, as arrows: 6 means a day takes four
// real minutes. Keys 1 to 4 pick them.
const SPEEDS = [2, 6, 30, 120];
let speedIndex = 1;
const speedButtons = SPEEDS.map((s, i) => {
  const day = (24 * 60) / s;
  const length = day >= 60 ? `${day / 60} minutes` : `${day} seconds`;
  const b = document.createElement("button");
  b.type = "button";
  b.textContent = "▶".repeat(i + 1);
  b.title = `${s} game minutes a second: a day takes ${length} (${i + 1})`;
  b.setAttribute("aria-label", `speed ${i + 1}`);
  b.addEventListener("click", () => setSpeed(i));
  $("speed-buttons").append(b);
  return b;
});
/** Picks a speed, and resumes if paused. Picking one stops watching a train. */
function setSpeed(i: number): void {
  speedIndex = i;
  paused = false;
  watching = null;
  drawnTick = -1;
}

/** A train being watched: time runs slowly while it moves, at the chosen speed while it
 * stands at the station, and the map follows it until someone moves the map. */
let watching: { train: TrainTimeView; follow: boolean } | null = null;
/** Game minutes per real second while a watched train is moving: about four tiles a second. */
const WATCH_SPEED = 0.25;
/** How long before a train reaches the station to skip to, in ticks. */
const WATCH_LEAD = 90;
function currentSpeed(): number {
  const w = watching?.train;
  if (w && (sim.tick < w.arrive || sim.tick >= w.depart)) return WATCH_SPEED;
  return SPEEDS[speedIndex]!;
}
function watchTrain(t: TrainTimeView): void {
  pinned = null;
  if (view3d) void setView3d(false);
  const rail = world.rail;
  if (rail && rail.place !== activePlace) showPlace(rail.place);
  skipTo(Math.max(t.enter + 8, t.arrive - WATCH_LEAD));
  watching = { train: t, follow: true };
  // Close enough to see the train, on the station to start with.
  const cam = camera();
  const stop = rail?.tiles[rail.stop];
  if (cam && stop) {
    const { w, h } = paneSize();
    cam.z = Math.max(cam.z, 0.75);
    cam.x = w / 2 - (stop.x + 0.5) * tileset.size * cam.z;
    cam.y = h / 2 - (stop.y + 0.5) * tileset.size * cam.z;
    cam.fitted = false;
    applyCamera();
  }
}
/** Keeps a followed train in the middle of the map, and ends the watch once it is gone. */
function followTrain(): void {
  if (!watching) return;
  if (sim.tick >= watching.train.exit) {
    watching = null;
    drawnTick = -1;
    return;
  }
  const train = sim.train();
  const rail = world.rail;
  const cam = camera();
  if (!watching.follow || !train || !rail || !cam || rail.place !== activePlace || view3d) return;
  const engine = trainCars(train, rail.tiles)[0];
  if (!engine) return;
  const { w, h } = paneSize();
  cam.x = w / 2 - (engine.x + 0.5) * tileset.size * cam.z;
  cam.y = h / 2 - (engine.y + 0.5) * tileset.size * cam.z;
  applyCamera();
}
let paused = false;
// The words tab, fed by every step the sim takes.
const DAY_TICKS = 24 * 60 * TICKS_PER_MINUTE;
const villagerName = (id: number): string => sim.villagers()[id]?.name ?? "?";
const wordsPanel = new Words({
  el: $("words"),
  topics: topicDefs,
  gestures: sim.gestures(),
  name: villagerName,
  topicText: (t) => topicText(t, villagerName),
  timeText: (tick) => `day ${Math.floor(tick / DAY_TICKS) + 1} ${sinceText(tick)}`,
  replay: (tick) => replayTo(tick, null),
});
let wordsDrawn = 0;

let gameTick = at(0, 6);
/** Steps the sim to `gameTick`, indexing what happened. */
function step(): void {
  const events = sim.stepUntil(gameTick);
  wordsPanel.add(events, gameTick);
  if (diorama && view3d) diorama.hear(events, gameTick, speechTopic);
}

/** The topic a speech mark shows: its gloss alone, since the speaker is plain to see. */
function speechTopic(e: Emitted): string | null {
  if (e.kind === "Beat") {
    const t = "Gesture" in e.did ? e.did.Gesture.topic : "Bid" in e.did ? e.did.Bid.topic : null;
    return t ? topicText(t, villagerName) : null;
  }
  if (e.kind === "Thought") return topicText(e.thought.about, villagerName);
  return null;
}
step();
let drawnTick = -1;
let drawnPaused = !paused;

function twelveHour(hour: number, minute: number): string {
  const h = hour % 12 === 0 ? 12 : hour % 12;
  return `${h}:${String(minute).padStart(2, "0")}${hour < 12 ? "am" : "pm"}`;
}

/** The clock's dial: the sky by the hour, with the sun crossing it by day and the moon by
 * night, over a strip of ground. */
const dialEl = document.getElementById("dial");
let dialHour = -1;
function drawDial(hour: number): void {
  if (!dialEl || Math.abs(hour - dialHour) < 0.1) return;
  dialHour = hour;
  const day = hour >= 6 && hour < 18;
  // Where along the arc from the east (left) to the west (right), 0 to 1.
  const t = day ? (hour - 6) / 12 : ((hour + 6) % 24) / 12;
  const a = Math.PI * (1 - t);
  const x = 23 + 15 * Math.cos(a);
  const y = 30 - 15 * Math.sin(a);
  const sky = day ? (hour < 7.5 || hour >= 16.5 ? "#e8a160" : "#8cc4e8") : hour >= 18 && hour < 19.5 ? "#4a3a6a" : "#1c2340";
  const body = day ? '<circle r="5" fill="#ffd968" stroke="#fff3b0" stroke-width="1.5"/>' : '<circle r="4.5" fill="#efe8d0"/><circle r="4" cx="2" cy="-1.5" fill="' + sky + '"/>';
  dialEl.innerHTML =
    `<clipPath id="dial-clip"><circle cx="23" cy="23" r="21"/></clipPath>` +
    `<g clip-path="url(#dial-clip)"><rect width="46" height="46" fill="${sky}"/>` +
    `<g transform="translate(${x.toFixed(1)} ${y.toFixed(1)})">${body}</g>` +
    `<rect y="30" width="46" height="16" fill="#4f8a3c"/><rect y="30" width="46" height="2" fill="#6aa850"/></g>`;
}

function draw(): void {
  if (sim.tick === drawnTick && paused === drawnPaused) return;
  drawnTick = sim.tick;
  drawnPaused = paused;
  const clock = sim.clock();
  shownClock = clock;
  const cal = clock.calendar;
  clockEl.textContent = `${cal.weekday} ${twelveHour(clock.time.hour, clock.time.minute)}`;
  dateEl.textContent = watching ? "watching the train" : `${cal.season} ${cal.day_of_season}, year ${cal.year}`;
  pauseEl.setAttribute("aria-label", paused ? "resume" : "pause");
  pauseEl.setAttribute("aria-pressed", String(paused));
  speedButtons.forEach((b, i) => b.setAttribute("aria-pressed", String(!paused && i === speedIndex)));
  drawDial(clock.time.hour + clock.time.minute / 60);
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
  if (diorama && view3d) {
    diorama.show(sim, activePlace);
    diorama.sync(sim, villagers, followed?.key ?? null);
    const key = followed?.key ?? null;
    if (key !== followedIn3d) {
      followedIn3d = key;
      diorama.followVillager(key);
    }
    if (pendingShot) {
      diorama.shot(pendingShot);
      pendingShot = null;
    }
  } else {
    lastVillagers = villagers;
    mapDirty = true;
    if (followed) keepInView(followed);
    followTrain();
  }
  // The words tab, at most once a second while it is open.
  if (!$("tab-words").hidden && performance.now() - wordsDrawn > 1000) {
    wordsDrawn = performance.now();
    wordsPanel.render();
  }
  inspect(villagers);
}

// Ticks of catch-up allowed per frame: a hidden tab resumes from where it left off.
const MAX_TICKS_PER_FRAME = 10 * TICKS_PER_MINUTE;
let last = performance.now();
let carry = 0;
function frame(now: number): void {
  if (photographing) {
    last = now;
    requestAnimationFrame(frame);
    return;
  }
  const dt = Math.min((now - last) / 1000, 1);
  last = now;
  if (!paused) {
    carry += dt * currentSpeed() * TICKS_PER_MINUTE;
    const whole = Math.min(Math.floor(carry), MAX_TICKS_PER_FRAME);
    if (whole > 0) {
      carry -= whole;
      gameTick += whole;
      step();
    }
  }
  draw();
  if (diorama && view3d && shownClock) diorama.render(sim.tick + carry, shownClock);
  else if (mapDirty) paintMap();
  requestAnimationFrame(frame);
}

document.addEventListener("visibilitychange", () => {
  if (!document.hidden) {
    last = performance.now();
    carry = 0;
  }
});
const pauseEl = $("pause");
const dateEl = $("date");
pauseEl.addEventListener("click", () => {
  paused = !paused;
});

// Skips to the next 6am: later today if it is still night, else tomorrow.
// The skip menu: the next 6am, and the next trains. Skipping to a train jumps to just
// before it reaches the station, puts the map on it, and slows time while it moves so it
// can be watched coming in and going out.
const skipEl = $("morning");
const skipMenu = $("skip-menu");
function nextMorning(): number {
  const day = 24 * 60 * TICKS_PER_MINUTE;
  let target = Math.floor(sim.tick / day) * day + 6 * 60 * TICKS_PER_MINUTE;
  if (target <= sim.tick) target += day;
  return target;
}
function skipTo(tick: number): void {
  if (tick > sim.tick) {
    gameTick = tick;
    step();
  }
  paused = false;
  drawnTick = -1;
}
function openSkipMenu(): void {
  const trains = sim.upcomingTrains();
  const stopping = trains.find((t) => t.stops);
  const passing = trains.find((t) => !t.stops);
  const item = (label: string, when: string, act: () => void): HTMLButtonElement => {
    const b = document.createElement("button");
    b.type = "button";
    b.setAttribute("role", "menuitem");
    const l = document.createElement("strong");
    l.textContent = label;
    const w = document.createElement("span");
    w.textContent = when;
    b.append(l, w);
    b.addEventListener("click", () => {
      closeSkipMenu();
      act();
    });
    return b;
  };
  const items = [item("Morning", whenText(nextMorning()), () => skipTo(nextMorning()))];
  const onTrack = trains[0] && trains[0].enter <= sim.tick ? trains[0] : undefined;
  if (onTrack) items.push(item("The train now", onTrack.stops && sim.tick < onTrack.depart ? "at the station" : "on the line", () => watchTrain(onTrack)));
  if (stopping && stopping !== onTrack) items.push(item("Train stopping", whenText(stopping.arrive), () => watchTrain(stopping)));
  if (passing && passing !== onTrack) items.push(item("Train passing through", whenText(passing.arrive), () => watchTrain(passing)));
  skipMenu.replaceChildren(...items);
  skipMenu.hidden = false;
  // Under the button, kept inside the map.
  const r = skipEl.getBoundingClientRect();
  const stage = $("stage").getBoundingClientRect();
  const left = Math.max(8, Math.min(r.left - stage.left, stage.width - skipMenu.offsetWidth - 8));
  skipMenu.style.left = `${left}px`;
  skipMenu.style.top = `${r.bottom - stage.top + 8}px`;
  skipEl.setAttribute("aria-expanded", "true");
  items[0]?.focus();
}
function closeSkipMenu(): void {
  skipMenu.hidden = true;
  skipEl.setAttribute("aria-expanded", "false");
}
skipEl.addEventListener("click", (e) => {
  e.stopPropagation();
  if (skipMenu.hidden) openSkipMenu();
  else closeSkipMenu();
});
document.addEventListener("click", (e) => {
  if (!skipMenu.hidden && !skipMenu.contains(e.target as Node)) closeSkipMenu();
});
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !skipMenu.hidden) closeSkipMenu();
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
  // A run of lines from one moment shares a picture; it goes on the first.
  const photo = photos.get(photoKey(item));
  const repeat = photo === lastPhoto;
  lastPhoto = photo;
  if (photo && !repeat) {
    const img = document.createElement("img");
    img.className = "photo";
    img.src = photo;
    img.alt = "";
    b.append(img);
  }
  b.append(when, what);
  const why = causeText(item.cause, sim.villagers());
  b.title = why ? `because: ${why}. Click to replay this moment.` : "Click to replay this moment.";
  b.addEventListener("click", () => goTo(item));
  return b;
}

function goTo(item: SummaryItem): void {
  replayTo(item.tick, item.who);
}

/** Replays the town to `tick`, paused, pinning `who` if given. */
function replayTo(tick: number, who: number | null): void {
  sim.rewind(tick);
  gameTick = tick;
  paused = true;
  if (who !== null) {
    const v = sim.villagers().find((x) => x.id === who);
    if (v) pinned = { kind: "villager", key: v.key };
  }
  pendingShot = shotOf(tick, who);
  drawnTick = -1;
}

/** Who a replayed moment is about, for the 3D camera: the person and whoever they are
 * talking with, else everyone talking near the moment's speaker. */
function shotOf(tick: number, who: number | null): string[] | null {
  const vs = sim.villagers();
  let v = who === null ? undefined : vs.find((x) => x.id === who);
  // A moment from the words tab names no one: take whoever was talking then.
  v ??= vs.find((x) => x.activity === "talk" && x.since <= tick);
  if (!v) return null;
  const me = v;
  const near = vs.filter((o) => o.key !== me.key && o.place === me.place && o.activity === "talk" && Math.max(Math.abs(o.pos.x - me.pos.x), Math.abs(o.pos.y - me.pos.y)) <= 3);
  return [me.key, ...near.map((o) => o.key)];
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
  lastPhoto = undefined;
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

// Photographs of a day's unusual moments: each one replayed and shot in the 3D view, then
// the town put back where it was.
const photos = new Map<string, string>();
let lastPhoto: string | undefined;
const photoKey = (item: SummaryItem): string => `${item.tick}:${item.who ?? ""}`;
let photographing = false;
const dayPhotosEl = $("day-photos") as HTMLButtonElement;
async function photographDay(): Promise<void> {
  const days = sim.summaries();
  const d = days[dayIndex ?? days.length - 1];
  if (!d || photographing) return;
  photographing = true;
  dayPhotosEl.disabled = true;
  const items = d.news.filter((item) => item.who !== null && !photos.has(photoKey(item)));
  const back = sim.tick < sim.frontier ? sim.tick : null;
  try {
    const dio = await loadDiorama();
    for (const [i, item] of items.entries()) {
      dayPhotosEl.textContent = `photographing ${i + 1} of ${items.length}`;
      sim.rewind(item.tick);
      const villagers = sim.villagers();
      const v = villagers.find((x) => x.id === item.who);
      if (!v) continue;
      dio.show(sim, v.place);
      dio.sync(sim, villagers, null, true);
      const url = dio.photo(shotOf(item.tick, item.who) ?? [v.key], 480, 300, sim.clock());
      if (url) photos.set(photoKey(item), url);
      // Let the page breathe between shots.
      await new Promise((r) => setTimeout(r, 0));
    }
  } finally {
    if (back === null) sim.backToNow();
    else sim.rewind(back);
    gameTick = sim.tick;
    photographing = false;
    dayPhotosEl.disabled = false;
    dayPhotosEl.textContent = "photograph this day";
    shownDays = "";
    drawnTick = -1;
  }
}
dayPhotosEl.addEventListener("click", () => void photographDay());

backEl.addEventListener("click", () => {
  sim.backToNow();
  gameTick = sim.tick;
  drawnTick = -1;
});

// The 3D view, loaded the first time it is switched on (its state is declared at the top).
const map3dEl = $("map3d");
const view3dEl = $("view3d");
async function loadDiorama(): Promise<Diorama> {
  if (!diorama) {
    const { Diorama } = await import("./diorama");
    diorama = new Diorama({
      tiles: tileset,
      places: world.places,
      objects: world.objects,
      roofs: world.roofs,
      rooms: world.rooms,
      world,
      name: villagerName,
      pick: (place, x, y) => {
        const v = sim.villagers().find((v) => v.place === place && v.pos.x === x && v.pos.y === y);
        const o = v ? undefined : world.objects.find((o) => o.place === place && o.pos.x === x && o.pos.y === y);
        const next: Focus = v ? { kind: "villager", key: v.key } : o ? { kind: "object", key: o.key } : { kind: "tile", place, x, y };
        pinned = samePin(pinned, next) ? null : next;
        drawnTick = -1;
      },
    });
    map3dEl.append(diorama.el);
  }
  return diorama;
}
async function setView3d(on: boolean): Promise<void> {
  if (on) await loadDiorama();
  view3d = on;
  followedIn3d = null;
  view3dEl.setAttribute("aria-pressed", String(on));
  map3dEl.hidden = !on;
  mapEl.hidden = on;
  $("turn").hidden = !on;
  drawnTick = -1;
  if (!on) applyCamera();
}

// The journal (people, days, words, key) folds away to give the map the window. On a
// phone it is a sheet whose tabs stay showing, and choosing a tab opens it.
const journalEl = $("journal");
function setJournal(open: boolean): void {
  document.body.classList.toggle("journal-closed", !open);
  journalEl.setAttribute("aria-pressed", String(open));
  try {
    localStorage.setItem("journal", open ? "open" : "closed");
  } catch {
    // Storage may be unavailable; the journal just starts open next time.
  }
  applyCamera();
}
journalEl.addEventListener("click", () => setJournal(document.body.classList.contains("journal-closed")));
$("journal-close").addEventListener("click", () => setJournal(false));
let journalStart = !window.matchMedia("(max-width: 760px)").matches;
try {
  const saved = localStorage.getItem("journal");
  if (saved) journalStart = saved === "open";
} catch {
  // As above.
}
setJournal(journalStart);
view3dEl.addEventListener("click", () => void setView3d(!view3d));
$("turn-left").addEventListener("click", () => diorama?.rotate(-Math.PI / 4));
$("turn-right").addEventListener("click", () => diorama?.rotate(Math.PI / 4));

// For Playwright and the agent: a structured dump of what is on screen.
declare global {
  interface Window {
    __scene: () => { tick: number; clock: string; everyone: string; hash: string; places: string[]; canvases: number; days: number };
    /** Runs the sim forward by days (to the minute) and redraws, for screenshots. */
    __advance: (days: number) => void;
    /** The 3D view of a place, optionally as a close shot of some villagers, as a PNG data URL. */
    __show3d: (key: string, shot?: string[]) => Promise<string | null>;
    /** Puts a place on screen and redraws; returns its canvas as a PNG data URL. */
    __show: (key: string) => string | null;
  }
}
window.__advance = (days: number) => {
  paused = true;
  gameTick += Math.max(0, Math.round(days * 24 * 60)) * TICKS_PER_MINUTE;
  step();
  drawnTick = -1;
  draw();
};
window.__show = (key: string) => {
  const place = world.places.find((p) => p.key === key);
  if (!place) return null;
  pinned = null;
  showPlace(place.id);
  draw();
  return activeCanvas()?.image(sim, sim.villagers()).toDataURL("image/png") ?? null;
};
window.__show3d = async (key: string, shot?: string[]) => {
  const place = world.places.find((p) => p.key === key);
  if (!place) return null;
  await setView3d(true);
  pinned = null;
  showPlace(place.id);
  drawnTick = -1;
  draw();
  if (!diorama || !shownClock) return null;
  if (shot) diorama.shot(shot);
  else diorama.fit();
  diorama.settle();
  diorama.render(sim.tick, shownClock);
  return diorama.snapshot();
};
window.__scene = () => ({
  tick: sim.tick,
  clock: (() => {
    const c = sim.clock();
    const cal = c.calendar;
    return `Y${cal.year} ${cal.season} ${cal.day_of_season} (${cal.weekday}) ${twelveHour(c.time.hour, c.time.minute)}`;
  })(),
  everyone: sim.whereIsEveryone(),
  hash: sim.hash(),
  places: sim.placeIds().map((id) => sim.renderPlace(id)),
  canvases: canvases.filter((c) => c.width > 0).length,
  days: sim.summaries().length,
});

draw();
requestAnimationFrame(frame);
