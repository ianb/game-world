// The 3D view: one place as a lit diorama. The ground is the 2D renderer's painting laid on
// a plane, walls are raised from it, and objects, trees and villagers are upright LPC sprites
// that turn to face the camera. Light follows the clock. Loaded only when switched on.
import * as THREE from "three";
import type { AnimalView, Calendar, ClockView, Emitted, FieldView, ObjectView, PlaceView, Rect, RoofView, RoomView, Sim, TrainView, VillagerView, Walling, WorldView } from "@game-world/protocol";
import { type Dressing, SOUTHERN_WOODS, dressingFor, paintDressing, treeAt } from "./dressing";
import { type Facing, type Materials, type Tileset, groundNames, isTreeCell, materials, objectTileAt, paintGround, trainCars } from "./render";

/** Wall height, in tiles: the LPC tall walls are three tiles of art. */
const WALL_H = 3;
/** What is left of a wall cut away so the camera can see past it. */
const STUB_H = 0.3;
/** Doorways are open to this height, with wall above. */
const DOOR_H = 2;
/** Closer than this the camera is looking at someone, and only walls in the way are cut;
 * further out every wall with a room behind it is cut, as in a dollhouse. */
const CLOSE = 16;
/** Objects drawn flat on the floor rather than standing up. */
const LYING = new Set(["bed", "bed_double", "platform"]);
/** Objects people stand at the front of, set back so whoever is on their tile shows. */
const BACKDROPS = new Set(["stall"]);
/** Objects that glow after dark. */
const HEARTHS = new Set(["oven", "stove", "hearth", "kiln", "copper"]);
/** How long a speech mark stays up, in real milliseconds. */
const SPEECH_MS = 3500;

export interface DioramaOptions {
  tiles: Tileset;
  places: PlaceView[];
  objects: ObjectView[];
  /** Inline buildings' footprints, lit from inside after dark. */
  roofs: RoofView[];
  /** Rooms, for their floors and the material of their walls. */
  rooms: RoomView[];
  /** The whole world view, for what dresses the ground. */
  world: WorldView;
  name: (id: number) => string;
  /** A tile was clicked (not dragged). */
  pick: (place: number, x: number, y: number) => void;
}

/** Where the camera looks from: a point on the ground, a turn about it, a tilt and a distance. */
interface View {
  x: number;
  z: number;
  yaw: number;
  pitch: number;
  dist: number;
}

/** A villager on screen, eased between the tiles the sim reports. */
interface Walker {
  mesh: THREE.Mesh;
  uv: THREE.BufferAttribute;
  label: HTMLDivElement;
  speech: HTMLDivElement;
  speechUntil: number;
  from: { x: number; z: number };
  to: { x: number; z: number };
  /** Game tick the current step was seen to start, and how long steps have been taking. */
  stepAt: number;
  stepTicks: number;
  shown: { x: number; z: number };
  heading: number;
  steps: number;
  frame: number;
  place: number;
}

const FACING_ROW: Record<Facing, number> = { up: 0, left: 1, down: 2, right: 3 };
const ROW_OF = [FACING_ROW.down, FACING_ROW.right, FACING_ROW.up, FACING_ROW.left];

/** "SmallTalk" as "small talk". */
const words = (s: string): string => s.replace(/([a-z])([A-Z])/g, "$1 $2").toLowerCase();

export class Diorama {
  readonly el: HTMLDivElement;
  private readonly renderer: THREE.WebGLRenderer;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(30, 1, 0.1, 400);
  private readonly atlas: THREE.Texture;
  private readonly spriteMaterial: THREE.MeshStandardMaterial;
  private readonly spriteDepth: THREE.MeshDepthMaterial;
  private readonly labels: HTMLDivElement;
  private readonly sun = new THREE.DirectionalLight(0xffffff, 2);
  private readonly sky = new THREE.HemisphereLight(0xbfd8ff, 0x4a3b2a, 1);
  private placeGroup = new THREE.Group();
  private sprites: THREE.Mesh[] = [];
  /** Trees, which step aside when they stand between the camera and what it looks at. */
  private trees: Trees[] = [];
  /** Where trees stand while a place is being built, before they become one mesh. */
  private treeSpots = new Map<string, { x: number; z: number }[]>();
  private dressing: Dressing | null = null;
  private calendar: Calendar | null = null;
  private readonly beasts = new Map<number, Beast>();
  private trainMeshes: THREE.Mesh[] = [];
  private hearths: THREE.PointLight[] = [];
  /** Lamps inside buildings, lit in the evening. */
  private lamps: THREE.PointLight[] = [];
  private walls: Walls | null = null;
  private ground: { canvas: HTMLCanvasElement; texture: THREE.CanvasTexture; painted: number } | null = null;
  private place: PlaceView | null = null;
  private mat: Materials | null = null;
  private readonly walkers = new Map<string, Walker>();
  private view: View = { x: 0, z: 0, yaw: 0, pitch: 0.75, dist: 40 };
  private goal: View = { ...this.view };
  /** While set, the camera keeps this villager in the middle. */
  private follow: string | null = null;
  private night = 0;

  constructor(private readonly o: DioramaOptions) {
    this.el = document.createElement("div");
    this.el.className = "diorama";
    this.renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.el.append(this.renderer.domElement);
    this.labels = document.createElement("div");
    this.labels.className = "labels";
    this.el.append(this.labels);

    this.atlas = new THREE.Texture(o.tiles.image);
    this.atlas.magFilter = THREE.NearestFilter;
    this.atlas.minFilter = THREE.NearestFilter;
    this.atlas.generateMipmaps = false;
    this.atlas.colorSpace = THREE.SRGBColorSpace;
    this.atlas.needsUpdate = true;
    this.spriteMaterial = new THREE.MeshStandardMaterial({ map: this.atlas, alphaTest: 0.5, side: THREE.DoubleSide, roughness: 1 });
    this.spriteDepth = new THREE.MeshDepthMaterial({ depthPacking: THREE.RGBADepthPacking, map: this.atlas, alphaTest: 0.5 });

    this.sun.castShadow = true;
    this.sun.shadow.mapSize.set(2048, 2048);
    this.sun.shadow.bias = -0.0005;
    this.sun.shadow.normalBias = 0.02;
    this.scene.add(this.sun, this.sun.target, this.sky, this.placeGroup);
    this.controls();
  }

  /** Puts a place on the stage, building its ground, walls and objects. */
  show(sim: Sim, placeId: number): void {
    if (this.place?.id === placeId) return;
    const place = this.o.places.find((p) => p.id === placeId);
    if (!place) return;
    this.place = place;
    this.mat = materials(place, this.o.rooms, this.o.roofs);
    this.scene.remove(this.placeGroup);
    this.placeGroup.traverse((o) => {
      if (o instanceof THREE.Mesh && o.geometry) o.geometry.dispose();
    });
    this.placeGroup = new THREE.Group();
    this.scene.add(this.placeGroup);
    this.walls?.dispose();
    this.walls = null;
    this.sprites = [];
    this.trees = [];
    this.treeSpots = new Map();
    this.beasts.clear();
    this.trainMeshes = [];
    this.dressing = dressingFor(this.o.world, place);
    this.calendar = sim.clock().calendar;
    this.hearths = [];
    this.lamps = [];
    this.ground?.texture.dispose();
    this.ground = null;
    for (const w of this.walkers.values()) {
      w.mesh.removeFromParent();
      w.label.remove();
      w.speech.remove();
    }
    this.walkers.clear();
    const names = groundNames(sim, place);
    this.buildGround(place, names);
    this.buildWalls(place, names);
    this.buildObjects(place);
    this.buildTrees(place, names);
    this.buildLamps(place);
    this.buildWoods(place);
    this.trees = this.makeTrees();
    this.syncAnimals(sim);
    // The sun's shadow covers the whole place.
    const s = this.sun.shadow.camera;
    const r = Math.hypot(place.width, place.height) / 2 + 4;
    s.left = -r;
    s.right = r;
    s.top = r;
    s.bottom = -r;
    s.near = 1;
    s.far = 4 * r + 40;
    s.updateProjectionMatrix();
    this.fit();
  }

  /** Frames the whole place from the usual angle. */
  fit(): void {
    const p = this.place;
    if (!p) return;
    this.follow = null;
    const size = Math.max(p.width, p.height * 1.3);
    this.goal = { x: p.width / 2, z: p.height / 2 + 1, yaw: 0, pitch: 0.75, dist: size * 1.9 };
  }

  /** Keeps a villager in the middle of the view, or stops following. */
  followVillager(key: string | null): void {
    if (key !== this.follow && key) this.goal.dist = Math.min(this.goal.dist, 24);
    this.follow = key;
  }

  /** A close shot of some villagers from the side, low, as if someone were standing there. */
  shot(keys: string[]): void {
    const view = this.shotView(keys);
    if (!view) return;
    this.follow = null;
    this.goal = view;
  }

  /** A photograph of a moment: the shot of `keys` at `width` by `height`, as a JPEG data
   * URL. Leaves the live camera where it was. Call after `show` and `sync(..., true)`. */
  photo(keys: string[], width: number, height: number, clock: ClockView): string | null {
    const view = this.shotView(keys);
    if (!view) return null;
    const saved = this.view;
    const size = this.renderer.getSize(new THREE.Vector2());
    this.view = view;
    this.renderer.setSize(width, height, false);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.light(clock);
    this.placeWalkers(Number.MAX_SAFE_INTEGER);
    this.aim();
    this.renderer.render(this.scene, this.camera);
    const url = this.renderer.domElement.toDataURL("image/jpeg", 0.85);
    this.view = saved;
    this.renderer.setSize(size.x, size.y, false);
    this.camera.aspect = size.x / Math.max(1, size.y);
    this.camera.updateProjectionMatrix();
    return url;
  }

  private shotView(keys: string[]): View | null {
    const ws = keys.map((k) => this.walkers.get(k)).filter((w): w is Walker => !!w && w.mesh.visible);
    if (!ws.length) return null;
    const x = ws.reduce((a, w) => a + w.to.x, 0) / ws.length;
    const z = ws.reduce((a, w) => a + w.to.z, 0) / ws.length;
    let yaw = 0;
    const [a, b] = ws;
    if (a && b) {
      // Look across the line between them, from whichever side is nearer the south.
      yaw = Math.atan2(a.to.z - b.to.z, b.to.x - a.to.x);
      if (yaw > Math.PI / 2) yaw -= Math.PI;
      if (yaw < -Math.PI / 2) yaw += Math.PI;
    }
    // From the other side only if much more stands in the way from the south, since from
    // the north people are seen from behind. Never end on, which would stand one in front
    // of the other. What is left in the way is hidden in `aim`.
    const view = { x, z, yaw, pitch: 0.42, dist: 13 };
    let best = view;
    let fewest = Infinity;
    for (const turn of [0, Math.PI]) {
      const v = { ...view, yaw: yaw + turn };
      const n = this.occluders(v).length + this.trees.reduce((k, t) => k + this.treesInTheWay(v, t).size, 0) + (turn === 0 ? 0 : 2.5);
      if (n < fewest) {
        best = v;
        fewest = n;
      }
    }
    return best;
  }

  /** Objects that would hide the middle of a close view. */
  private occluders(v: View): THREE.Mesh[] {
    return this.sprites.filter((m) => inTheWay(v, m.position.x, m.position.z, Number(m.userData.w ?? 1), Number(m.userData.h ?? 1)));
  }

  /** Trees of one kind, by index, that would hide the middle of a close view. */
  private treesInTheWay(v: View, t: Trees): Set<number> {
    const out = new Set<number>();
    t.at.forEach((p, i) => {
      if (inTheWay(v, p.x, p.z, t.w, t.h)) out.add(i);
    });
    return out;
  }

  rotate(by: number): void {
    this.goal.yaw += by;
  }

  zoomBy(f: number): void {
    this.goal.dist = Math.min(160, Math.max(4, this.goal.dist * f));
    this.goal.pitch = Math.min(1.2, Math.max(0.22, 0.25 + (this.goal.dist / 60) * 0.7));
  }

  /** Jumps the camera to where it was heading, for screenshots. */
  settle(): void {
    this.view = { ...this.goal };
  }

  /** Speech marks for what was said: the gesture and topic over the speaker's head. */
  hear(events: Emitted[], upTo: number, topicText: (e: Emitted) => string | null): void {
    const now = performance.now();
    for (const e of events) {
      // After a jump forward, only what was said in the last few minutes.
      if (e.tick < upTo - 5 * 60 * 60) continue;
      if (e.kind !== "Beat" && e.kind !== "Thought") continue;
      const key = this.keyOf(e.who);
      const w = key ? this.walkers.get(key) : undefined;
      if (!w) continue;
      let text: string | null = null;
      if (e.kind === "Beat") {
        if ("Gesture" in e.did) text = words(e.did.Gesture.gesture);
        else if ("Bid" in e.did) text = `bids: ${words(e.did.Bid.frame)}`;
      } else if (e.thought.aloud) {
        text = "mutters";
      }
      if (!text) continue;
      const topic = topicText(e);
      w.speech.textContent = topic ? `${text} · ${topic}` : text;
      w.speechUntil = now + SPEECH_MS;
    }
  }

  private keyOf(id: number): string | null {
    for (const [k, w] of this.walkers) if (Number(w.mesh.userData.id) === id) return k;
    return null;
  }

  /** Updates villagers' positions from the sim. Call whenever the sim has stepped. */
  sync(sim: Sim, villagers: VillagerView[], focus: string | null, snap = false): void {
    const place = this.place;
    if (!place) return;
    const tick = sim.tick;
    for (const v of villagers) {
      let w = this.walkers.get(v.key);
      const here = v.place === place.id && v.activity !== "sleep";
      if (!here) {
        if (w) w.mesh.visible = false;
        if (w) w.label.hidden = w.speech.hidden = true;
        continue;
      }
      const to = { x: v.pos.x + 0.5, z: v.pos.y + 0.5 };
      if (!w) {
        w = this.walker(v, to);
        this.walkers.set(v.key, w);
      }
      w.mesh.visible = true;
      w.label.hidden = false;
      w.label.classList.toggle("focus", v.key === focus);
      if (w.to.x !== to.x || w.to.z !== to.z || w.place !== v.place) {
        const jump = snap || Math.abs(to.x - w.to.x) + Math.abs(to.z - w.to.z) > 2 || w.place !== v.place;
        const took = tick - w.stepAt;
        if (took > 0 && took < 1200) w.stepTicks = took;
        w.from = jump ? to : { ...w.shown };
        w.heading = Math.atan2(to.x - w.to.x, to.z - w.to.z);
        w.to = to;
        w.stepAt = tick;
        w.steps += 1;
        w.place = v.place;
      }
      if (v.activity === "talk") {
        // Turn toward the nearest other talker.
        let best: VillagerView | null = null;
        let d = 4;
        for (const o of villagers) {
          if (o.key === v.key || o.activity !== "talk" || o.place !== v.place) continue;
          const od = Math.max(Math.abs(o.pos.x - v.pos.x), Math.abs(o.pos.y - v.pos.y));
          if (od < d) {
            best = o;
            d = od;
          }
        }
        if (best && d > 0) w.heading = Math.atan2(best.pos.x - v.pos.x, best.pos.y - v.pos.y);
      }
    }
    this.syncAnimals(sim);
    this.syncTrain(sim.train());
    if (place.kind === "Zone" && performance.now() - (this.ground?.painted ?? 0) > 3000) {
      this.calendar = sim.clock().calendar;
      this.repaintGround(sim);
    }
  }

  /** Animals as sprites, easing toward their tiles; roosting ones are hidden. */
  private syncAnimals(sim: Sim): void {
    const place = this.place;
    if (!place) return;
    for (const a of sim.animals()) {
      let b = this.beasts.get(a.id);
      const here = a.place === place.id && !a.roosting;
      if (!here) {
        if (b) b.mesh.visible = false;
        continue;
      }
      const to = { x: a.pos.x + 0.5, z: a.pos.y + 0.5 };
      if (!b) {
        b = this.beast(a, to);
        this.beasts.set(a.id, b);
      }
      b.mesh.visible = true;
      if (b.to.x !== to.x || b.to.z !== to.z) {
        b.heading = Math.atan2(to.x - b.to.x, to.z - b.to.z);
        b.to = to;
        b.steps += 1;
      }
    }
  }

  private beast(a: AnimalView, at: { x: number; z: number }): Beast {
    const size = a.kind === "sheep" ? 2 : 1;
    const g = new THREE.PlaneGeometry(size, size);
    g.translate(0, size / 2 - (a.kind === "sheep" ? 0.3 : 0.05), 0);
    const uv = g.getAttribute("uv") as THREE.BufferAttribute;
    const mesh = new THREE.Mesh(g, this.spriteMaterial);
    mesh.customDepthMaterial = this.spriteDepth;
    mesh.castShadow = true;
    this.placeGroup.add(mesh);
    const b: Beast = { mesh, uv, kind: a.kind, to: at, shown: { ...at }, heading: 0, steps: 0, frame: -1 };
    this.beastFrame(b, true);
    return b;
  }

  private beastFrame(b: Beast, force = false): void {
    const t = this.o.tiles.atlas.tiles[`animal_${b.kind}`];
    if (!t) return;
    const rel = b.heading - this.view.yaw;
    const quarter = ((Math.round(rel / (Math.PI / 2)) % 4) + 4) % 4;
    const row = ROW_OF[quarter] ?? FACING_ROW.down;
    const moving = Math.abs(b.shown.x - b.to.x) + Math.abs(b.shown.z - b.to.z) > 0.05;
    const col = moving ? (b.steps + Math.floor(performance.now() / 150)) % 4 : 0;
    const code = row * 4 + col;
    if (!force && code === b.frame) return;
    b.frame = code;
    const s = this.o.tiles.size;
    const f = (b.kind === "sheep" ? 2 : 1) * s;
    const iw = this.o.tiles.image.width;
    const ih = this.o.tiles.image.height;
    const sx = t.col * s + col * f;
    const sy = t.row * s + row * f;
    b.uv.set([sx / iw, 1 - sy / ih, (sx + f) / iw, 1 - sy / ih, sx / iw, 1 - (sy + f) / ih, (sx + f) / iw, 1 - (sy + f) / ih]);
    b.uv.needsUpdate = true;
  }

  /** The train's cars as upright sprites along the track. */
  private syncTrain(t: TrainView | null): void {
    const rail = this.o.world.rail;
    const cars = t && rail && this.place && rail.place === this.place.id ? trainCars(t, rail.tiles) : [];
    while (this.trainMeshes.length < cars.length) {
      const m = this.sprite("train_wagon", 0, 0);
      if (!m) break;
      this.trainMeshes.push(m);
    }
    this.trainMeshes.forEach((m, i) => {
      const c = cars[i];
      m.visible = c !== undefined;
      if (!c) return;
      const t = this.o.tiles.atlas.tiles[c.tile];
      if (!t) return;
      const s = this.o.tiles.size;
      const iw = this.o.tiles.image.width;
      const ih = this.o.tiles.image.height;
      const [u0, u1] = [(t.col * s) / iw, ((t.col + 2) * s) / iw];
      const [vb, vt] = [1 - ((t.row + 2) * s) / ih, 1 - (t.row * s) / ih];
      const attr = m.geometry.getAttribute("uv") as THREE.BufferAttribute;
      attr.set([u0, vt, u1, vt, u0, vb, u1, vb]);
      attr.needsUpdate = true;
      m.position.set(c.x + 1, 0, c.y + 0.55);
    });
  }

  /** Draws a frame. `tick` may carry a fraction for smooth walking. */
  render(tick: number, clock: ClockView): void {
    const { width, height } = this.el.getBoundingClientRect();
    if (width === 0 || height === 0) return;
    const size = this.renderer.getSize(new THREE.Vector2());
    if (size.x !== Math.round(width) || size.y !== Math.round(height)) {
      this.renderer.setSize(width, height, true);
      this.camera.aspect = width / height;
      this.camera.updateProjectionMatrix();
    }
    this.light(clock);
    this.placeWalkers(tick);
    for (const b of this.beasts.values()) {
      if (!b.mesh.visible) continue;
      b.shown.x += (b.to.x - b.shown.x) * 0.15;
      b.shown.z += (b.to.z - b.shown.z) * 0.15;
      b.mesh.position.set(b.shown.x, 0, b.shown.z - 0.1);
      this.beastFrame(b);
    }
    if (this.follow) {
      const w = this.walkers.get(this.follow);
      if (w?.mesh.visible) {
        this.goal.x = w.shown.x;
        this.goal.z = w.shown.z;
      }
    }
    // The camera eases toward where it is going.
    const k = 0.12;
    const v = this.view;
    const g = this.goal;
    v.x += (g.x - v.x) * k;
    v.z += (g.z - v.z) * k;
    v.yaw += (g.yaw - v.yaw) * k;
    v.pitch += (g.pitch - v.pitch) * k;
    v.dist += (g.dist - v.dist) * k;
    this.aim();
    this.renderer.render(this.scene, this.camera);
    this.placeLabels();
  }

  /** Walkers ease from tile to tile over the time their steps have been taking. People on
   * one tile stand side by side, across the view. */
  private placeWalkers(tick: number): void {
    const shared = new Map<string, string[]>();
    for (const [key, w] of this.walkers) {
      if (!w.mesh.visible) continue;
      const cell = `${w.to.x},${w.to.z}`;
      shared.set(cell, [...(shared.get(cell) ?? []), key].sort());
    }
    const across = { x: Math.cos(this.view.yaw), z: -Math.sin(this.view.yaw) };
    for (const [key, w] of this.walkers) {
      if (!w.mesh.visible) continue;
      const t = Math.min(1, Math.max(0, (tick - w.stepAt) / w.stepTicks));
      const group = shared.get(`${w.to.x},${w.to.z}`) ?? [key];
      const off = (group.indexOf(key) - (group.length - 1) / 2) * 0.55;
      w.shown = { x: w.from.x + (w.to.x - w.from.x) * t + across.x * off, z: w.from.z + (w.to.z - w.from.z) * t + across.z * off };
      w.mesh.position.set(w.shown.x, 0, w.shown.z - 0.2);
      const moving = t < 1 && (w.from.x !== w.to.x || w.from.z !== w.to.z);
      this.setFrame(w, moving ? 1 + (Math.floor(t * 4 + w.steps * 4) % 8) : 0);
    }
  }

  /** Points the camera from the current view and turns the sprites to face it. */
  private aim(): void {
    const v = this.view;
    const lookY = Math.min(1, 12 / v.dist);
    this.camera.position.set(
      v.x + v.dist * Math.sin(v.yaw) * Math.cos(v.pitch),
      lookY + v.dist * Math.sin(v.pitch),
      v.z + v.dist * Math.cos(v.yaw) * Math.cos(v.pitch),
    );
    this.camera.lookAt(v.x, lookY, v.z);
    for (const s of this.sprites) s.rotation.y = v.yaw;
    // Close up, whatever stands between the camera and what it looks at steps aside.
    for (const s of this.sprites) s.visible = true;
    if (v.dist < CLOSE) for (const m of this.occluders(v)) m.visible = false;
    for (const t of this.trees) {
      const hidden = v.dist < CLOSE ? this.treesInTheWay(v, t) : new Set<number>();
      const o = new THREE.Object3D();
      o.rotation.y = v.yaw;
      t.at.forEach((p, i) => {
        o.position.set(p.x, 0, p.z);
        o.scale.setScalar(hidden.has(i) ? 0 : 1);
        o.updateMatrix();
        t.mesh.setMatrixAt(i, o.matrix);
      });
      t.mesh.instanceMatrix.needsUpdate = true;
    }
    for (const w of this.walkers.values()) w.mesh.rotation.y = v.yaw;
    for (const b of this.beasts.values()) b.mesh.rotation.y = v.yaw;
    for (const m of this.trainMeshes) m.rotation.y = v.yaw;
    this.walls?.cutFor(v);
  }

  /** The canvas as a PNG, drawn fresh. */
  snapshot(): string {
    return this.renderer.domElement.toDataURL("image/png");
  }

  // Building the place.

  private buildGround(place: PlaceView, names: string[]): void {
    const s = this.o.tiles.size;
    const canvas = document.createElement("canvas");
    canvas.width = place.width * s;
    canvas.height = place.height * s;
    const texture = new THREE.CanvasTexture(canvas);
    texture.magFilter = THREE.NearestFilter;
    texture.minFilter = THREE.LinearMipmapLinearFilter;
    texture.anisotropy = this.renderer.capabilities.getMaxAnisotropy();
    texture.colorSpace = THREE.SRGBColorSpace;
    this.ground = { canvas, texture, painted: 0 };
    this.paint(place, names);
    const plane = new THREE.Mesh(new THREE.PlaneGeometry(place.width, place.height), new THREE.MeshStandardMaterial({ map: texture, roughness: 1 }));
    plane.rotation.x = -Math.PI / 2;
    plane.position.set(place.width / 2, 0, place.height / 2);
    plane.receiveShadow = true;
    plane.userData.ground = true;
    this.placeGroup.add(plane);
    // Under the edges, so the place does not float in the sky colour.
    const skirt = new THREE.Mesh(new THREE.PlaneGeometry(place.width * 6, place.height * 6), new THREE.MeshStandardMaterial({ color: place.kind === "Zone" ? 0x2f4a24 : 0x1a1612, roughness: 1 }));
    skirt.rotation.x = -Math.PI / 2;
    skirt.position.set(place.width / 2, -0.02, place.height / 2);
    skirt.receiveShadow = true;
    this.placeGroup.add(skirt);
  }

  private repaintGround(sim: Sim): void {
    if (!this.place) return;
    const id = this.place.id;
    this.fields = sim.fields().filter((f) => f.place === id);
    this.paint(this.place, groundNames(sim, this.place));
  }

  /** The fields of the place shown, as of the last repaint. */
  private fields: FieldView[] | null = null;

  private paint(place: PlaceView, names: string[]): void {
    const g = this.ground;
    if (!g) return;
    const ctx = g.canvas.getContext("2d");
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;
    const mat = this.mat ?? materials(place, this.o.rooms, this.o.roofs);
    paintGround(ctx, this.o.tiles, place, names, true, mat);
    if (this.dressing) paintDressing(ctx, this.o.tiles, place, names, mat, this.dressing, this.calendar, null, this.fields);
    // Things that lie flat, like beds, are part of the floor.
    for (const o of this.o.objects) {
      if (o.place !== place.id || !LYING.has(o.kind)) continue;
      const tile = objectTileAt(o, this.o.objects);
      if (tile) this.o.tiles.draw(ctx, tile, o.pos.x, o.pos.y);
    }
    g.texture.needsUpdate = true;
    g.painted = performance.now();
  }

  private buildWalls(place: PlaceView, names: string[]): void {
    if (!this.ground) return;
    // The buildings the camera can look into: inline ones by their footprint, or the whole
    // place when it is an interior.
    const rooms = place.kind === "Zone" ? this.o.roofs.filter((r) => r.place === place.id).map((r) => r.rect) : [{ x: 0, y: 0, w: place.width, h: place.height }];
    const walls = (this.mat ?? materials(place, this.o.rooms, this.o.roofs)).walls;
    this.walls = new Walls(this.o.tiles, this.atlas, this.ground.texture, place, names, rooms, walls);
    this.placeGroup.add(this.walls.group);
  }

  private buildObjects(place: PlaceView): void {
    for (const o of this.o.objects) {
      if (o.place !== place.id || LYING.has(o.kind)) continue;
      const tile = objectTileAt(o, this.o.objects);
      const m = tile ? this.sprite(tile, o.pos.x, o.pos.y) : null;
      if (m && BACKDROPS.has(o.kind)) m.position.z -= 0.6;
      if (m) this.sprites.push(m);
      if (HEARTHS.has(o.kind)) {
        const lamp = new THREE.PointLight(0xffa04a, 0, 9, 1.6);
        lamp.position.set(o.pos.x + 0.5, 1.2, o.pos.y + 0.9);
        this.placeGroup.add(lamp);
        this.hearths.push(lamp);
      }
    }
  }

  private buildTrees(place: PlaceView, names: string[]): void {
    for (let y = 0; y < place.height; y++) {
      for (let x = 0; x < place.width; x++) {
        if (isTreeCell(names, place.width, x, y)) this.addTree(treeAt(x, y, place.kind === "Zone" && y > place.height * SOUTHERN_WOODS), x, y);
      }
    }
  }

  /** A lamp in each inline building, over its middle. */
  private buildLamps(place: PlaceView): void {
    const rooms = place.kind === "Zone" ? this.o.roofs.filter((r) => r.place === place.id) : [{ rect: { x: 0, y: 0, w: place.width, h: place.height } }];
    for (const r of rooms) {
      const lamp = new THREE.PointLight(0xffc27a, 0, Math.max(r.rect.w, r.rect.h) * 1.1, 1.4);
      lamp.position.set(r.rect.x + r.rect.w / 2, 2.2, r.rect.y + r.rect.h / 2);
      this.placeGroup.add(lamp);
      this.lamps.push(lamp);
    }
  }

  /** Woods around an outdoor place, so it sits in a landscape rather than on a board. */
  private buildWoods(place: PlaceView): void {
    if (place.kind !== "Zone") return;
    const W = place.width;
    const H = place.height;
    const band = 14;
    for (let y = -band; y < H + band; y++) {
      for (let x = -band; x < W + band; x++) {
        const inside = x >= 0 && y >= 0 && x < W && y < H;
        if (inside) continue;
        const edge = Math.max(-x, x - W + 1, -y, y - H + 1);
        let h = Math.imul(x + 1000, 374761393) + Math.imul(y + 1000, 668265263);
        h = Math.imul(h ^ (h >>> 13), 1274126177);
        const r = ((h ^ (h >>> 16)) >>> 0) / 4294967296;
        // Thicker further out; none right on the edge, where the canopy tiles are.
        if (edge < 2 || r > 0.12 + edge * 0.02) continue;
        this.addTree(treeAt(x, y, false), x, y);
      }
    }
  }

  private addTree(kind: string, x: number, y: number): void {
    const spots = this.treeSpots.get(kind) ?? [];
    spots.push(this.spriteFoot(kind, x, y));
    this.treeSpots.set(kind, spots);
  }

  /** Where a sprite's foot stands, in tiles, as the 2D view places it. */
  private spriteFoot(name: string, x: number, y: number): { x: number; z: number } {
    const t = this.o.tiles.atlas.tiles[name] ?? this.o.tiles.atlas.tiles["obj_generic"];
    const s = this.o.tiles.size;
    const tw = t?.w ?? 1;
    const th = t?.h ?? 1;
    const ox = t?.ox ?? 0;
    const oy = t?.oy ?? (th > 1 ? s - th * s : 0);
    return { x: x + ox / s + tw / 2, z: y + (oy + th * s) / s - 0.45 };
  }

  /** Every tree as one instanced mesh per kind, each turned to the camera in `aim`. */
  private makeTrees(): Trees[] {
    const out: Trees[] = [];
    for (const [kind, spots] of this.treeSpots) {
      const t = this.o.tiles.atlas.tiles[kind];
      if (!t || !spots.length) continue;
      const s = this.o.tiles.size;
      const tw = t.w ?? 1;
      const th = t.h ?? 1;
      const iw = this.o.tiles.image.width;
      const ih = this.o.tiles.image.height;
      const g = new THREE.PlaneGeometry(tw, th);
      g.translate(0, th / 2, 0);
      const u0 = (t.col * s) / iw, u1 = ((t.col + tw) * s) / iw;
      const v0 = 1 - ((t.row + th) * s) / ih, v1 = 1 - (t.row * s) / ih;
      g.setAttribute("uv", new THREE.Float32BufferAttribute([u0, v1, u1, v1, u0, v0, u1, v0], 2));
      const mesh = new THREE.InstancedMesh(g, this.spriteMaterial, spots.length);
      mesh.customDepthMaterial = this.spriteDepth;
      mesh.castShadow = true;
      this.placeGroup.add(mesh);
      out.push({ mesh, at: spots, w: tw, h: th });
    }
    return out;
  }

  /** An upright sprite standing where the 2D view would draw it, its foot in its cell. */
  private sprite(name: string, x: number, y: number): THREE.Mesh | null {
    const t = this.o.tiles.atlas.tiles[name] ?? this.o.tiles.atlas.tiles["obj_generic"];
    if (!t) return null;
    const s = this.o.tiles.size;
    const tw = t.w ?? 1;
    const th = t.h ?? 1;
    const ox = t.ox ?? 0;
    const oy = t.oy ?? (th > 1 ? s - th * s : 0);
    const iw = this.o.tiles.image.width;
    const ih = this.o.tiles.image.height;
    const g = new THREE.PlaneGeometry(tw, th);
    g.translate(0, th / 2, 0);
    const u0 = (t.col * s) / iw, u1 = ((t.col + tw) * s) / iw;
    const v0 = 1 - ((t.row + th) * s) / ih, v1 = 1 - (t.row * s) / ih;
    g.setAttribute("uv", new THREE.Float32BufferAttribute([u0, v1, u1, v1, u0, v0, u1, v0], 2));
    const m = new THREE.Mesh(g, this.spriteMaterial);
    m.customDepthMaterial = this.spriteDepth;
    m.castShadow = true;
    // Centre and foot as the 2D view places them, in tiles.
    const cx = x + ox / s + tw / 2;
    const foot = y + (oy + th * s) / s;
    m.position.set(cx, 0, foot - 0.45);
    m.userData.w = tw;
    m.userData.h = th;
    this.placeGroup.add(m);
    return m;
  }

  private walker(v: VillagerView, at: { x: number; z: number }): Walker {
    const vs = this.o.tiles.atlas.villagers;
    const fs = vs.frame / this.o.tiles.size;
    const g = new THREE.PlaneGeometry(fs, fs);
    g.translate(0, fs / 2 - 0.06, 0);
    const uv = g.getAttribute("uv") as THREE.BufferAttribute;
    const mesh = new THREE.Mesh(g, this.spriteMaterial);
    mesh.customDepthMaterial = this.spriteDepth;
    mesh.castShadow = true;
    mesh.userData.id = v.id;
    mesh.userData.key = v.key;
    this.placeGroup.add(mesh);
    const label = document.createElement("div");
    label.className = "name";
    label.textContent = v.name;
    const speech = document.createElement("div");
    speech.className = "speech";
    speech.hidden = true;
    this.labels.append(label, speech);
    const w: Walker = { mesh, uv, label, speech, speechUntil: 0, from: at, to: at, stepAt: 0, stepTicks: 60, shown: at, heading: 0, steps: 0, frame: -1, place: v.place };
    this.setFrame(w, 0, true);
    return w;
  }

  /** Points a walker's quad at a frame of their sheet, facing relative to the camera. */
  private setFrame(w: Walker, frame: number, force = false): void {
    const vs = this.o.tiles.atlas.villagers;
    const top = vs.top[String(w.mesh.userData.key)] ?? 0;
    // Which of the four drawn directions is nearest the heading as the camera sees it.
    const rel = w.heading - this.view.yaw;
    const quarter = ((Math.round(rel / (Math.PI / 2)) % 4) + 4) % 4;
    const row = ROW_OF[quarter] ?? FACING_ROW.down;
    const code = frame * 4 + row;
    if (!force && code === w.frame) return;
    w.frame = code;
    const iw = this.o.tiles.image.width;
    const ih = this.o.tiles.image.height;
    const f = vs.frame;
    const sx = (vs.left?.[String(w.mesh.userData.key)] ?? 0) + (frame % vs.frames) * f;
    const sy = top + row * f;
    const u0 = sx / iw, u1 = (sx + f) / iw, v0 = 1 - (sy + f) / ih, v1 = 1 - sy / ih;
    w.uv.set([u0, v1, u1, v1, u0, v0, u1, v0]);
    w.uv.needsUpdate = true;
  }

  // Light.

  /** Sun, sky and hearths from the time of day. */
  private light(clock: ClockView): void {
    const hour = clock.time.hour + clock.time.minute / 60;
    // The sun is up from 6 to 20; its height follows a sine over that span.
    const day = (hour - 6) / 14;
    const up = day > 0 && day < 1 ? Math.sin(day * Math.PI) : 0;
    const dusk = day > 0 && day < 1 ? Math.max(0, 1 - Math.min(day, 1 - day) * 6) : 0;
    // From the east in the morning, round by the south, to the west.
    const az = Math.PI * (1 - Math.min(1, Math.max(0, day)));
    const p = this.place;
    const cx = p ? p.width / 2 : 0;
    const cz = p ? p.height / 2 : 0;
    const r = 60;
    this.sun.position.set(cx + Math.cos(az) * r * 0.9, 8 + up * r, cz + Math.sin(az) * r * 0.55 + 10);
    this.sun.target.position.set(cx, 0, cz);
    const sunColor = new THREE.Color(0xfff1d6).lerp(new THREE.Color(0xff9a4a), dusk);
    this.sun.color.copy(sunColor);
    this.night = 1 - Math.min(1, up * 3);
    // After dark the same light is the moon: dim, blue and high.
    if (this.night > 0.5) {
      this.sun.position.set(cx - 20, 50, cz + 25);
      this.sun.color.set(0x9db4ff);
    }
    this.sun.intensity = Math.max(2.6 * Math.min(1, up * 2.2), this.night * 0.55);
    const skyDay = new THREE.Color(0xbfd8ff);
    const skyNight = new THREE.Color(0x24305a);
    this.sky.color.copy(skyDay.clone().lerp(skyNight, this.night).lerp(new THREE.Color(0xffb37a), dusk * 0.4));
    this.sky.groundColor.set(0x4a3b2a).lerp(new THREE.Color(0x10121c), this.night);
    // Interiors are never as dark as the night outside.
    this.sky.intensity = Math.max(1.1 - this.night * 0.6, p && p.kind !== "Zone" ? 0.75 : 0);
    const bg = new THREE.Color(0x9fc4e8).lerp(new THREE.Color(0x141a30), this.night).lerp(new THREE.Color(0xe8a070), dusk * 0.5);
    this.scene.background = bg;
    const d = this.view.dist;
    this.scene.fog = new THREE.Fog(bg, d * 1.1, d * 2.6);
    const flicker = 0.9 + 0.1 * Math.sin(performance.now() / 130) * Math.sin(performance.now() / 370);
    for (const lamp of this.hearths) lamp.intensity = (0.6 + this.night * 5) * flicker;
    // Lamps go on at dusk and out late at night.
    const lit = hour >= 18.5 || hour < 0.5 ? Math.min(1, this.night * 1.5) : 0;
    for (const lamp of this.lamps) lamp.intensity = lit * 6;
    this.walls?.glow(lit);
  }

  // Labels.

  private placeLabels(): void {
    const { width, height } = this.el.getBoundingClientRect();
    const now = performance.now();
    const p = new THREE.Vector3();
    // Names and speech marks move up until they clear the ones already placed.
    const used: { l: number; r: number; t: number; b: number }[] = [];
    const place = (cx: number, bottom: number, el: HTMLElement): number => {
      const half = el.offsetWidth / 2 + 2;
      const tall = el.offsetHeight + 2;
      let b = bottom;
      for (let i = 0; i < 20; i++) {
        const hit = used.find((u) => cx - half < u.r && cx + half > u.l && b - tall < u.b && b > u.t);
        if (!hit) break;
        b = hit.t;
      }
      used.push({ l: cx - half, r: cx + half, t: b - tall, b });
      el.style.transform = `translate(${Math.round(cx)}px, ${Math.round(b)}px) translate(-50%, -100%)`;
      return b - tall;
    };
    const shown = [...this.walkers.values()].filter((w) => w.mesh.visible);
    // Nearer the camera first, so the front ones keep their places.
    const depth = new Map(shown.map((w) => [w, p.set(w.shown.x, 1.7, w.shown.z - 0.2).project(this.camera).z]));
    shown.sort((a, b) => (depth.get(a) ?? 0) - (depth.get(b) ?? 0));
    for (const w of shown) {
      p.set(w.shown.x, 1.7, w.shown.z - 0.2).project(this.camera);
      const x = (p.x * 0.5 + 0.5) * width;
      const y = (-p.y * 0.5 + 0.5) * height;
      const top = place(x, y, w.label);
      const talking = now < w.speechUntil;
      w.speech.hidden = !talking;
      if (talking) {
        place(x, top - 2, w.speech);
        w.speech.style.opacity = String(Math.min(1, (w.speechUntil - now) / 600));
      }
    }
  }

  // Input: two-finger scroll or a drag pans, pinch or ctrl+scroll zooms, a right-drag or
  // shift-drag turns, a click picks a tile.

  private controls(): void {
    const el = this.renderer.domElement;
    el.style.touchAction = "none";
    const pan = (dx: number, dy: number): void => {
      const k = this.goal.dist / 900;
      const c = Math.cos(this.goal.yaw), s = Math.sin(this.goal.yaw);
      this.goal.x -= (dx * c + dy * s) * k;
      this.goal.z -= (-dx * s + dy * c) * k / Math.max(0.35, Math.sin(this.goal.pitch));
      this.follow = null;
    };
    // Closer means lower, like leaning in.
    const zoom = (f: number): void => this.zoomBy(f);
    el.addEventListener(
      "wheel",
      (e) => {
        e.preventDefault();
        if (e.ctrlKey || e.metaKey) zoom(Math.min(1.5, Math.max(1 / 1.5, Math.exp(e.deltaY * 0.01))));
        else {
          const k = e.deltaMode === 1 ? 16 : 1;
          if (e.shiftKey) this.goal.yaw += e.deltaX * k * 0.004;
          else pan(-e.deltaX * k, -e.deltaY * k);
        }
      },
      { passive: false },
    );
    let gs = 1;
    el.addEventListener("gesturestart", (e) => {
      e.preventDefault();
      gs = 1;
    });
    el.addEventListener("gesturechange", (e) => {
      e.preventDefault();
      const g = e as unknown as { scale: number; rotation: number };
      zoom(gs / g.scale);
      gs = g.scale;
    });
    const pointers = new Map<number, { x: number; y: number }>();
    let down: { x: number; y: number; turn: boolean } | null = null;
    let dragged = false;
    el.addEventListener("contextmenu", (e) => e.preventDefault());
    el.addEventListener("pointerdown", (e) => {
      pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
      if (pointers.size === 1) {
        down = { x: e.clientX, y: e.clientY, turn: e.button === 2 || e.shiftKey };
        dragged = false;
      }
    });
    el.addEventListener("pointermove", (e) => {
      const prev = pointers.get(e.pointerId);
      if (!prev || !down) return;
      const now = { x: e.clientX, y: e.clientY };
      if (!dragged && Math.hypot(now.x - down.x, now.y - down.y) > 4) {
        dragged = true;
        el.setPointerCapture(e.pointerId);
      }
      if (dragged && pointers.size === 1) {
        if (down.turn) {
          this.goal.yaw -= (now.x - prev.x) * 0.006;
          this.goal.pitch = Math.min(1.35, Math.max(0.15, this.goal.pitch + (now.y - prev.y) * 0.004));
        } else pan(now.x - prev.x, now.y - prev.y);
      } else if (pointers.size === 2) {
        const other = [...pointers].find(([id]) => id !== e.pointerId)?.[1];
        if (other) {
          const before = Math.hypot(prev.x - other.x, prev.y - other.y);
          const after = Math.hypot(now.x - other.x, now.y - other.y);
          if (after > 0) zoom(before / after);
        }
      }
      pointers.set(e.pointerId, now);
    });
    const up = (e: PointerEvent): void => {
      pointers.delete(e.pointerId);
      if (pointers.size === 0 && down && !dragged) this.click(e.clientX, e.clientY);
      if (pointers.size === 0) down = null;
    };
    el.addEventListener("pointerup", up);
    el.addEventListener("pointercancel", (e) => {
      pointers.delete(e.pointerId);
      down = null;
    });
  }

  private click(cx: number, cy: number): void {
    const place = this.place;
    if (!place) return;
    const r = this.renderer.domElement.getBoundingClientRect();
    const ndc = new THREE.Vector2(((cx - r.left) / r.width) * 2 - 1, -((cy - r.top) / r.height) * 2 + 1);
    const ray = new THREE.Raycaster();
    ray.setFromCamera(ndc, this.camera);
    // A villager under the pointer wins over the ground behind them.
    const hits = ray.intersectObjects([...this.walkers.values()].filter((w) => w.mesh.visible).map((w) => w.mesh));
    const hit = hits[0];
    if (hit) {
      const key = String(hit.object.userData.key);
      const w = this.walkers.get(key);
      if (w) {
        this.o.pick(place.id, Math.floor(w.to.x), Math.floor(w.to.z));
        return;
      }
    }
    const ground = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
    const p = new THREE.Vector3();
    if (!ray.ray.intersectPlane(ground, p)) return;
    const x = Math.floor(p.x);
    const y = Math.floor(p.z);
    if (x >= 0 && y >= 0 && x < place.width && y < place.height) this.o.pick(place.id, x, y);
  }
}

/** One wall cell: a full wall, or a doorway with wall above it. */
interface WallCell {
  x: number;
  y: number;
  door: boolean;
}

/** Atlas coordinates of a tile: left, bottom, right, top in texture space. */
type Uv = [number, number, number, number];

/** The walls of a place, raised to full height. Walls that stand between the camera and
 * what it looks at are cut down to a stub; the geometry is rebuilt when that set changes. */
class Walls {
  readonly group = new THREE.Group();
  private readonly cells: WallCell[] = [];
  private readonly index = new Map<string, number>();
  private readonly sideMaterial: THREE.MeshStandardMaterial;
  private readonly topMaterial: THREE.MeshStandardMaterial;
  private readonly glowMaterial: THREE.MeshBasicMaterial;
  private readonly windows: { cell: number; meshes: THREE.Mesh[] }[] = [];
  private meshes: THREE.Mesh[] = [];
  private cutKey: string | null = null;
  private cut = new Set<number>();

  constructor(
    private readonly tiles: Tileset,
    atlas: THREE.Texture,
    ground: THREE.Texture,
    private readonly place: PlaceView,
    private readonly names: string[],
    private readonly rooms: Rect[],
    private readonly walling: (Walling | undefined)[],
  ) {
    this.sideMaterial = new THREE.MeshStandardMaterial({ map: atlas, roughness: 1, side: THREE.DoubleSide });
    this.topMaterial = new THREE.MeshStandardMaterial({ map: ground, roughness: 1, side: THREE.DoubleSide });
    this.glowMaterial = new THREE.MeshBasicMaterial({ map: atlas, transparent: true, opacity: 0, blending: THREE.AdditiveBlending, depthWrite: false });
    for (let y = 0; y < place.height; y++) {
      for (let x = 0; x < place.width; x++) {
        const n = names[y * place.width + x];
        if (n !== "wall" && n !== "door") continue;
        this.index.set(`${x},${y}`, this.cells.length);
        this.cells.push({ x, y, door: n === "door" });
      }
    }
    this.buildWindows(atlas);
    this.rebuild();
  }

  private at(x: number, y: number): string | undefined {
    const { width: w, height: h } = this.place;
    return x < 0 || y < 0 || x >= w || y >= h ? undefined : this.names[y * w + x];
  }

  private uv(name: string, rows = 1): Uv {
    const t = this.tiles.atlas.tiles[name];
    const iw = this.tiles.image.width;
    const ih = this.tiles.image.height;
    const s = this.tiles.size;
    if (!t) return [0, 0, 0, 0];
    return [(t.col * s) / iw, 1 - ((t.row + rows) * s) / ih, ((t.col + 1) * s) / iw, 1 - (t.row * s) / ih];
  }

  /** Outside walls get a window on some cells, away from doors. */
  private buildWindows(atlas: THREE.Texture): void {
    const pane = new THREE.MeshStandardMaterial({ map: atlas, alphaTest: 0.5, roughness: 1 });
    const sides: [number, number, number][] = [
      [0, 1, 0],
      [0, -1, Math.PI],
      [1, 0, Math.PI / 2],
      [-1, 0, -Math.PI / 2],
    ];
    this.cells.forEach((c, i) => {
      if (c.door) return;
      for (const [dx, dy, turn] of sides) {
        const out = this.at(c.x + dx, c.y + dy);
        if (out === undefined || out === "wall" || out === "door" || out === "floor" || out === "opening") continue;
        // Not beside a door along the wall, and only on some cells.
        if (this.at(c.x + dy, c.y + dx) === "door" || this.at(c.x - dy, c.y - dx) === "door") continue;
        if ((c.x * 7 + c.y * 13 + dx * 3 + dy * 5) % 3 !== 0) continue;
        const meshes = [pane, this.glowMaterial].map((material, k) => {
          const g = new THREE.PlaneGeometry(1, 1);
          const [u0, v0, u1, v1] = this.uv(k === 0 ? "window" : "window_glow");
          g.setAttribute("uv", new THREE.Float32BufferAttribute([u0, v1, u1, v1, u0, v0, u1, v0], 2));
          const m = new THREE.Mesh(g, material);
          const lift = 0.01 + k * 0.01;
          m.position.set(c.x + 0.5 + dx * (0.5 + lift), 1.6, c.y + 0.5 + dy * (0.5 + lift));
          m.rotation.y = turn;
          if (k === 0) m.receiveShadow = true;
          this.group.add(m);
          return m;
        });
        this.windows.push({ cell: i, meshes });
      }
    });
  }

  /** Lights the windows from inside, 0 to 1. */
  glow(lit: number): void {
    this.glowMaterial.opacity = lit;
  }

  /** Chooses which walls to cut for a view, and rebuilds if that changed. */
  cutFor(v: View): void {
    const back = { x: Math.sin(v.yaw), z: Math.cos(v.yaw) };
    const reach = v.dist * Math.cos(v.pitch);
    const tan = Math.tan(v.pitch);
    const cut = new Set<number>();
    const inside = (r: Rect, x: number, y: number): boolean => x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h;
    const close = v.dist < CLOSE;
    const room = close ? this.rooms.find((r) => inside(r, v.x, v.z)) : undefined;
    // One step away from the camera, along whichever axis it mostly looks.
    const away = Math.abs(back.x) > Math.abs(back.z) ? [-Math.sign(back.x), 0] : [0, -Math.sign(back.z)];
    this.cells.forEach((c, i) => {
      const dx = c.x + 0.5 - v.x;
      const dz = c.y + 0.5 - v.z;
      const along = dx * back.x + dz * back.z;
      const across = Math.abs(dx * back.z - dz * back.x);
      if (close) {
        // The near walls of the building in view come away, and any other wall that the
        // line of sight to the middle of the view would have to pass through.
        if (room && inside(room, c.x, c.y) && along > 0.3) cut.add(i);
        else if (along > 0.3 && along < reach && across < 4 && 0.2 + along * tan < WALL_H) cut.add(i);
      } else {
        // A room behind it, looking past walls more than one cell thick.
        for (let k = 1; k <= 3; k++) {
          const behind = this.at(c.x + (away[0] ?? 0) * k, c.y + (away[1] ?? 0) * k);
          if (behind === "wall" || behind === "door") continue;
          if (behind === "floor" || behind === "opening") cut.add(i);
          break;
        }
      }
    });
    const key = [...cut].join(",");
    if (key === this.cutKey) return;
    this.cutKey = key;
    this.cut = cut;
    this.rebuild();
  }

  private rebuild(): void {
    for (const m of this.meshes) {
      m.geometry.dispose();
      m.removeFromParent();
    }
    this.meshes = [];
    const sides: number[] = [];
    const sideUv: number[] = [];
    const tops: number[] = [];
    const topUv: number[] = [];
    const { width: w, height: h } = this.place;
    const span = (i: number): [number, number] => {
      const c = this.cells[i];
      if (!c) return [0, 0];
      return [c.door ? DOOR_H : 0, this.cut.has(i) ? STUB_H : WALL_H];
    };
    // A vertical quad from (x0, z0) to (x1, z1), from height lo to hi, showing that part of
    // a three-tile wall strip.
    const face = (x0: number, z0: number, x1: number, z1: number, lo: number, hi: number, strip: Uv): void => {
      const [u0, vb, u1, vt] = strip;
      const v = (y: number): number => vb + ((vt - vb) * y) / WALL_H;
      sides.push(x0, hi, z0, x0, lo, z0, x1, hi, z1, x1, hi, z1, x0, lo, z0, x1, lo, z1);
      sideUv.push(u0, v(hi), u0, v(lo), u1, v(hi), u1, v(hi), u0, v(lo), u1, v(lo));
    };
    const strips = new Map<string, Uv>();
    const strip = (name: string): Uv => {
      let uv = strips.get(name);
      if (!uv) {
        uv = this.uv(name, 3);
        strips.set(name, uv);
      }
      return uv;
    };
    this.cells.forEach((c, i) => {
      const [lo, hi] = span(i);
      if (hi <= lo) return;
      const { x, y } = c;
      const material = this.walling[y * w + x] ?? "timber";
      const inside = strip(`wall_tall_inside_${material}`);
      const outside = strip(`wall_tall_outside_${material}`);
      // Top: this cell of the ground painting, raised.
      const u0 = x / w, u1 = (x + 1) / w, v0 = 1 - (y + 1) / h, v1 = 1 - y / h;
      tops.push(x, hi, y, x, hi, y + 1, x + 1, hi, y, x + 1, hi, y, x, hi, y + 1, x + 1, hi, y + 1);
      topUv.push(u0, v1, u0, v0, u1, v1, u1, v1, u0, v0, u1, v0);
      if (c.door) {
        // The underside of the lintel.
        tops.push(x, lo, y, x + 1, lo, y, x, lo, y + 1, x + 1, lo, y, x + 1, lo, y + 1, x, lo, y + 1);
        topUv.push(u0, v1, u1, v1, u0, v0, u1, v1, u1, v0, u0, v0);
      }
      const side = (nx: number, ny: number, x0: number, z0: number, x1: number, z1: number): void => {
        const j = this.index.get(`${nx},${ny}`);
        if (j !== undefined) {
          // Against another wall: only what rises above it, or what a doorway leaves open.
          const [nlo, nhi] = span(j);
          if (nlo <= lo && nhi >= hi) return;
        }
        const n = this.at(nx, ny);
        // Rooms and the ends that a cut or a doorway leaves show the inside of the wall.
        face(x0, z0, x1, z1, lo, hi, j !== undefined || n === "floor" || n === "opening" ? inside : outside);
      };
      side(x, y + 1, x, y + 1, x + 1, y + 1);
      side(x, y - 1, x + 1, y, x, y);
      side(x + 1, y, x + 1, y + 1, x + 1, y);
      side(x - 1, y, x, y, x, y + 1);
    });
    const mesh = (pos: number[], uv: number[], material: THREE.Material): void => {
      if (!pos.length) return;
      const g = new THREE.BufferGeometry();
      g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
      g.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2));
      g.computeVertexNormals();
      const m = new THREE.Mesh(g, material);
      m.castShadow = true;
      m.receiveShadow = true;
      this.group.add(m);
      this.meshes.push(m);
    };
    mesh(sides, sideUv, this.sideMaterial);
    mesh(tops, topUv, this.topMaterial);
    for (const win of this.windows) for (const m of win.meshes) m.visible = !this.cut.has(win.cell);
  }

  dispose(): void {
    for (const m of this.meshes) m.geometry.dispose();
    for (const win of this.windows) for (const m of win.meshes) m.geometry.dispose();
    this.group.removeFromParent();
  }
}

/** The trees of a place: one mesh, an instance per tree, all the same size. */
/** An animal's sprite and where it is going. */
interface Beast {
  mesh: THREE.Mesh;
  uv: THREE.BufferAttribute;
  kind: AnimalView["kind"];
  to: { x: number; z: number };
  shown: { x: number; z: number };
  heading: number;
  steps: number;
  frame: number;
}

interface Trees {
  mesh: THREE.InstancedMesh;
  at: { x: number; z: number }[];
  w: number;
  h: number;
}

/** Whether something `w` wide and `h` tall standing at (`x`, `z`) hides the middle of a
 * close view from its camera. */
function inTheWay(v: View, x: number, z: number, w: number, h: number): boolean {
  const back = { x: Math.sin(v.yaw), z: Math.cos(v.yaw) };
  const reach = v.dist * Math.cos(v.pitch);
  const dx = x - v.x;
  const dz = z - v.z;
  const along = dx * back.x + dz * back.z;
  const across = Math.abs(dx * back.z - dz * back.x);
  return along > 0.6 && along < reach && across < w / 2 + 0.8 && h / along > Math.tan(v.pitch);
}
