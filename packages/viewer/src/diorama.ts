// The 3D view: one place as a lit diorama. The ground is the 2D renderer's painting laid on
// a plane, walls are raised from it, and objects, trees and villagers are upright LPC sprites
// that turn to face the camera. Light follows the clock. Loaded only when switched on.
import * as THREE from "three";
import type { ClockView, Emitted, ObjectView, PlaceView, RoofView, Sim, VillagerView } from "@game-world/protocol";
import { type Facing, type Tileset, groundNames, isTreeCell, objectTile, paintGround } from "./render";

/** Wall height, in tiles. Low enough to see into rooms from the usual angle. */
const WALL_H = 1;
/** Objects drawn flat on the floor rather than standing up. */
const LYING = new Set(["bed"]);
/** Objects that glow after dark. */
const HEARTHS = new Set(["oven", "stove"]);
/** How long a speech mark stays up, in real milliseconds. */
const SPEECH_MS = 3500;

export interface DioramaOptions {
  tiles: Tileset;
  places: PlaceView[];
  objects: ObjectView[];
  /** Inline buildings' footprints, lit from inside after dark. */
  roofs: RoofView[];
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
  private trees: THREE.Mesh[] = [];
  private hearths: THREE.PointLight[] = [];
  /** Lamps inside buildings, lit in the evening. */
  private lamps: THREE.PointLight[] = [];
  private ground: { canvas: HTMLCanvasElement; texture: THREE.CanvasTexture; painted: number } | null = null;
  private place: PlaceView | null = null;
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
    this.scene.remove(this.placeGroup);
    this.placeGroup.traverse((o) => {
      if (o instanceof THREE.Mesh && o.geometry) o.geometry.dispose();
    });
    this.placeGroup = new THREE.Group();
    this.scene.add(this.placeGroup);
    this.sprites = [];
    this.trees = [];
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
    return { x, z, yaw, pitch: 0.42, dist: 13 };
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
    if (place.kind === "Zone" && performance.now() - (this.ground?.painted ?? 0) > 3000) {
      this.repaintGround(sim);
    }
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
    // Toward the camera along the ground, and across.
    const back = { x: Math.sin(v.yaw), z: Math.cos(v.yaw) };
    const reach = v.dist * Math.cos(v.pitch);
    for (const t of this.trees) {
      t.rotation.y = v.yaw;
      const dx = t.position.x - v.x;
      const dz = t.position.z - v.z;
      const along = dx * back.x + dz * back.z;
      const across = Math.abs(dx * back.z - dz * back.x);
      // A tree four tiles tall hides what is behind it from a low camera.
      t.visible = !(along > 0.8 && along < reach && across < 2.5 && 4 / along > Math.tan(v.pitch));
    }
    for (const w of this.walkers.values()) w.mesh.rotation.y = v.yaw;
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
    this.paint(this.place, groundNames(sim, this.place));
  }

  private paint(place: PlaceView, names: string[]): void {
    const g = this.ground;
    if (!g) return;
    const ctx = g.canvas.getContext("2d");
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;
    paintGround(ctx, this.o.tiles, place, names, true);
    // Things that lie flat, like beds, are part of the floor.
    for (const o of this.o.objects) {
      if (o.place === place.id && LYING.has(o.kind)) this.o.tiles.draw(ctx, objectTile(o.kind), o.pos.x, o.pos.y);
    }
    g.texture.needsUpdate = true;
    g.painted = performance.now();
  }

  /** Walls as one mesh of side faces from the atlas, topped with the ground's own painting
   * of the wall so the tops join up the way the 2D view shapes them. */
  private buildWalls(place: PlaceView, names: string[]): void {
    const w = place.width;
    const h = place.height;
    const isWall = (x: number, y: number): boolean => names[y * w + x] === "wall";
    const at = (x: number, y: number): string | undefined => (x < 0 || y < 0 || x >= w || y >= h ? undefined : names[y * w + x]);
    const sides: number[] = [];
    const sideUv: number[] = [];
    const tops: number[] = [];
    const topUv: number[] = [];
    const quad = (out: number[], uvOut: number[], a: number[], b: number[], c: number[], d: number[], uv: [number, number, number, number]): void => {
      // a b on top, c d below, wound to face outward.
      out.push(...a, ...c, ...b, ...b, ...c, ...d);
      const [u0, v0, u1, v1] = uv;
      uvOut.push(u0, v1, u0, v0, u1, v1, u1, v1, u0, v0, u1, v0);
    };
    const tile = (name: string): [number, number, number, number] => {
      const t = this.o.tiles.atlas.tiles[name];
      const iw = this.o.tiles.image.width;
      const ih = this.o.tiles.image.height;
      const s = this.o.tiles.size;
      if (!t) return [0, 0, 0, 0];
      return [(t.col * s) / iw, 1 - ((t.row + 1) * s) / ih, ((t.col + 1) * s) / iw, 1 - (t.row * s) / ih];
    };
    const faceFor = (n: string | undefined): [number, number, number, number] => tile(n === "floor" || n === "opening" ? "wall_face" : "wall_outside");
    const H = WALL_H;
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        if (!isWall(x, y)) continue;
        // Top: this cell of the ground painting, raised.
        const u0 = x / w, u1 = (x + 1) / w, v0 = 1 - (y + 1) / h, v1 = 1 - y / h;
        tops.push(x, H, y, x, H, y + 1, x + 1, H, y, x + 1, H, y, x, H, y + 1, x + 1, H, y + 1);
        topUv.push(u0, v1, u0, v0, u1, v1, u1, v1, u0, v0, u1, v0);
        const south = at(x, y + 1);
        if (south !== "wall") quad(sides, sideUv, [x, H, y + 1], [x + 1, H, y + 1], [x, 0, y + 1], [x + 1, 0, y + 1], faceFor(south));
        const north = at(x, y - 1);
        if (north !== "wall") quad(sides, sideUv, [x + 1, H, y], [x, H, y], [x + 1, 0, y], [x, 0, y], faceFor(north));
        const east = at(x + 1, y);
        if (east !== "wall") quad(sides, sideUv, [x + 1, H, y + 1], [x + 1, H, y], [x + 1, 0, y + 1], [x + 1, 0, y], faceFor(east));
        const west = at(x - 1, y);
        if (west !== "wall") quad(sides, sideUv, [x, H, y], [x, H, y + 1], [x, 0, y], [x, 0, y + 1], faceFor(west));
      }
    }
    const mesh = (pos: number[], uv: number[], material: THREE.Material): void => {
      if (!pos.length) return;
      const g = new THREE.BufferGeometry();
      g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
      g.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2));
      g.computeVertexNormals();
      const m = new THREE.Mesh(g, material);
      m.castShadow = true;
      m.receiveShadow = true;
      this.placeGroup.add(m);
    };
    const wallMaterial = new THREE.MeshStandardMaterial({ map: this.atlas, roughness: 1, side: THREE.DoubleSide });
    mesh(sides, sideUv, wallMaterial);
    if (this.ground) mesh(tops, topUv, new THREE.MeshStandardMaterial({ map: this.ground.texture, roughness: 1 }));
  }

  private buildObjects(place: PlaceView): void {
    for (const o of this.o.objects) {
      if (o.place !== place.id || LYING.has(o.kind)) continue;
      const m = this.sprite(objectTile(o.kind), o.pos.x, o.pos.y);
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
        if (!isTreeCell(names, place.width, x, y)) continue;
        const m = this.sprite("tree", x, y);
        if (m) this.trees.push(m);
      }
    }
  }

  /** A lamp in each inline building, over its middle. */
  private buildLamps(place: PlaceView): void {
    for (const r of this.o.roofs) {
      if (r.place !== place.id) continue;
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
        const m = this.sprite("tree", x, y);
        if (m) this.trees.push(m);
      }
    }
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
    const sx = (frame % vs.frames) * f;
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
    this.sky.intensity = 1.1 - this.night * 0.6;
    const bg = new THREE.Color(0x9fc4e8).lerp(new THREE.Color(0x141a30), this.night).lerp(new THREE.Color(0xe8a070), dusk * 0.5);
    this.scene.background = bg;
    const d = this.view.dist;
    this.scene.fog = new THREE.Fog(bg, d * 1.1, d * 2.6);
    const flicker = 0.9 + 0.1 * Math.sin(performance.now() / 130) * Math.sin(performance.now() / 370);
    for (const lamp of this.hearths) lamp.intensity = (0.6 + this.night * 5) * flicker;
    // Lamps go on at dusk and out late at night.
    const lit = hour >= 18.5 || hour < 0.5 ? Math.min(1, this.night * 1.5) : 0;
    for (const lamp of this.lamps) lamp.intensity = lit * 6;
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
