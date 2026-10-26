// The words tab: every topic (from the content) and every gesture (from the sim), how often
// each has come up, and for one of them the times it did. Built from the events the sim
// returns as it steps, so it covers the run so far.
import type { Emitted, GestureView, Topic } from "@game-world/protocol";

export interface TopicDef {
  id: string;
  gloss: string;
  trouble?: string | null;
}

/** One time a word came up: when, and a line saying what happened. */
interface Use {
  tick: number;
  text: string;
}

/** Most recent uses kept per word. */
const KEEP = 60;

class Tally {
  count = 0;
  readonly by = new Map<string, number>();
  readonly uses: Use[] = [];
  add(tick: number, text: string, who?: string): void {
    this.count += 1;
    if (who) this.by.set(who, (this.by.get(who) ?? 0) + 1);
    this.uses.push({ tick, text });
    if (this.uses.length > KEEP) this.uses.shift();
  }
}

export type Word = { kind: "topic"; key: string } | { kind: "gesture"; name: string };

export interface WordsOptions {
  el: HTMLElement;
  topics: TopicDef[];
  gestures: GestureView[];
  name: (id: number) => string;
  topicText: (t: Topic) => string;
  timeText: (tick: number) => string;
  /** Replays the town to a moment. */
  replay: (tick: number) => void;
}

/** "SmallTalk" as "small talk". */
const words = (s: string): string => s.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();

export class Words {
  private readonly topics = new Map<string, Tally>();
  private readonly gestures = new Map<string, Tally>();
  /** For each gesture done, what it was taken as when that differed. */
  private readonly takenAs = new Map<string, Map<string, number>>();
  private readonly convos = new Map<number, [number, number]>();
  private indexedTo = -1;
  private open: Word | null = null;
  private shown = "";

  constructor(private readonly o: WordsOptions) {
    for (const t of o.topics) this.topics.set(t.id, new Tally());
    for (const g of o.gestures) this.gestures.set(g.gesture, new Tally());
  }

  /** Indexes events from a step up to `upTo`. Steps over the past (after a rewind) add
   * nothing, so nothing is counted twice. */
  add(events: Emitted[], upTo: number): void {
    const from = this.indexedTo;
    if (upTo <= from) return;
    const n = this.o.name;
    for (const e of events) {
      if (e.tick <= from) continue;
      switch (e.kind) {
        case "ConversationStarted":
          this.convos.set(e.convo, [e.who, e.with]);
          this.topic(e.topic, e.tick, `${n(e.who)} and ${n(e.with)} start ${words(e.frame)} about ${this.o.topicText(e.topic)}`, n(e.who));
          break;
        case "Beat": {
          const pair = this.convos.get(e.convo);
          const to = pair ? n(pair[0] === e.who ? pair[1] : pair[0]) : "?";
          if ("Gesture" in e.did) {
            const { gesture, topic } = e.did.Gesture;
            const line = `${n(e.who)} to ${to}: ${words(gesture)} "${this.o.topicText(topic)}"`;
            this.gestures.get(gesture)?.add(e.tick, line, n(e.who));
            this.topic(topic, e.tick, line, n(e.who));
          } else if ("Bid" in e.did) {
            const { frame, topic } = e.did.Bid;
            this.topic(topic, e.tick, `${n(e.who)} to ${to}: bids for ${words(frame)} about "${this.o.topicText(topic)}"`, n(e.who));
          }
          break;
        }
        case "TookUp":
          if (e.taken_as !== e.did) {
            const m = this.takenAs.get(e.did) ?? new Map<string, number>();
            m.set(e.taken_as, (m.get(e.taken_as) ?? 0) + 1);
            this.takenAs.set(e.did, m);
          }
          if (e.caught !== "Addressed") {
            this.gestures
              .get(e.did)
              ?.add(e.tick, `${n(e.who)} overheard ${n(e.speaker)} (${words(e.did)}${e.taken_as !== e.did ? `, took it as ${words(e.taken_as)}` : ""}), caught ${words(e.caught)}`);
          }
          break;
        case "Thought": {
          const t = e.thought;
          this.topic(t.about, e.tick, `${n(e.who)} thinks${t.aloud ? " aloud" : ""}: ${this.o.topicText(t.about)}`, n(e.who));
          break;
        }
        default:
          break;
      }
    }
    this.indexedTo = upTo;
  }

  private topic(t: Topic, tick: number, text: string, who?: string): void {
    this.topics.get(t.key)?.add(tick, text, who);
  }

  show(word: Word | null): void {
    this.open = word;
    this.shown = "";
    this.render(true);
  }

  /** Redraws when what it would show has changed; `force` redraws anyway. */
  render(force = false): void {
    const key = this.open ? `${this.open.kind}:${this.open.kind === "topic" ? this.open.key : this.open.name}:${this.count(this.open)}` : [...this.topics.values(), ...this.gestures.values()].map((t) => t.count).join(",");
    if (!force && key === this.shown) return;
    this.shown = key;
    this.o.el.replaceChildren(this.open ? this.detail(this.open) : this.list());
  }

  private count(w: Word): number {
    return (w.kind === "topic" ? this.topics.get(w.key) : this.gestures.get(w.name))?.count ?? 0;
  }

  private list(): DocumentFragment {
    const frag = document.createDocumentFragment();
    const section = (title: string, entries: [Word, string, number][]): void => {
      const h = document.createElement("h2");
      h.textContent = title;
      const box = document.createElement("div");
      box.className = "chips";
      for (const [w, label, n] of entries.sort((a, b) => b[2] - a[2])) {
        box.append(this.chip(w, n ? `${label} ×${n}` : label, n === 0));
      }
      frag.append(h, box);
    };
    section(
      "topics",
      this.o.topics.map((t) => [{ kind: "topic", key: t.id }, t.trouble ? `${t.gloss} (trouble)` : t.gloss, this.topics.get(t.id)?.count ?? 0]),
    );
    section(
      "gestures",
      this.o.gestures.map((g) => [{ kind: "gesture", name: g.gesture }, words(g.gesture), this.gestures.get(g.gesture)?.count ?? 0]),
    );
    return frag;
  }

  /** A button that opens a word. */
  chip(w: Word, label: string, quiet = false): HTMLButtonElement {
    const b = document.createElement("button");
    b.type = "button";
    b.className = quiet ? "word muted" : "word";
    b.textContent = label;
    b.addEventListener("click", () => this.show(w));
    return b;
  }

  private detail(w: Word): DocumentFragment {
    const frag = document.createDocumentFragment();
    const back = document.createElement("button");
    back.type = "button";
    back.textContent = "◀ all words";
    back.addEventListener("click", () => this.show(null));
    const lines: string[] = [];
    let tally: Tally | undefined;
    if (w.kind === "topic") {
      const def = this.o.topics.find((t) => t.id === w.key);
      tally = this.topics.get(w.key);
      const gloss = def?.gloss ?? w.key;
      lines.push(`topic: ${gloss}${gloss === w.key ? "" : ` (${w.key})`}`);
      if (def?.trouble) lines.push(`marks trouble: ${def.trouble}`);
    } else {
      const g = this.o.gestures.find((x) => x.gesture === w.name);
      tally = this.gestures.get(w.name);
      lines.push(`gesture: ${words(w.name)}`);
      if (g) {
        const reach = [`words ${g.words}`, `gist ${g.gist}`, g.noise ? `raised voice ${g.noise}` : null].filter(Boolean).join(", ");
        lines.push(`carries (tiles): ${reach}`);
        lines.push(`warmth: ${signed(g.warmth)} by convention; ${g.by.map((b) => `${this.o.name(b.who)} ${signed(b.warmth)}`).join(", ")}`);
      }
      const taken = this.takenAs.get(w.name);
      if (taken?.size) lines.push(`taken as: ${[...taken].sort((a, b) => b[1] - a[1]).map(([t, n]) => `${words(t)} ×${n}`).join(", ")}`);
    }
    lines.push(`came up ${tally?.count ?? 0} times`);
    if (tally?.by.size) lines.push(`by: ${[...tally.by].sort((a, b) => b[1] - a[1]).map(([who, n]) => `${who} ×${n}`).join(", ")}`);
    const head = document.createElement("pre");
    head.className = "word-head";
    head.textContent = lines.join("\n");
    frag.append(back, head);
    const uses = [...(tally?.uses ?? [])].reverse();
    if (uses.length) {
      const h = document.createElement("h2");
      h.textContent = uses.length < (tally?.count ?? 0) ? `the last ${uses.length}` : "each time";
      frag.append(h);
    }
    for (const u of uses) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "moment";
      const when = document.createElement("span");
      when.className = "when";
      when.textContent = this.o.timeText(u.tick);
      const what = document.createElement("span");
      what.textContent = u.text;
      b.append(when, what);
      b.title = "Click to replay this moment.";
      b.addEventListener("click", () => this.o.replay(u.tick));
      frag.append(b);
    }
    return frag;
  }
}

const signed = (n: number): string => (n > 0 ? `+${n}` : `${n}`);
