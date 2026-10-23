# Rust and other compiled stacks for the sim core, versus TypeScript

Researched 2026-10-23. Scope: a deterministic discrete-event sim for a 2D top-down pixel-art town game, built by an AI coding agent in a headless Linux container, web first, desktop/Steam second. TypeScript is the baseline. JS renderers, Unity, Godot, Defold and GameMaker are covered by other researchers; they appear here only where they could host a Rust core.

A note on sourcing: many GitHub pages were fetched directly, but GitHub's rendered pages do not show commit dates or release dates in the fetched text, so "last activity" is usually taken from crates.io or from a dated blog post instead. Where a figure comes from an aggregator or a syndicated article rather than the project, the finding says so.

## Bevy (0.18 through 0.20): fit, headless, tilemap, pixel art, wasm, compile times, churn, testing, Foundation

### Takeaway
Bevy ships a new 0.x every four to five months (0.18 on 2026-01-13, 0.19 on 2026-06-19, 0.20 on 2026-10-08), each with a migration guide on the order of 75 sections, and it is a schedule/system-driven ECS rather than an event-queue sim kernel. It has the best-documented pixel-art and wasm paths of the Rust engines, a real headless mode (`MinimalPlugins`), and a maintained tilemap crate, but it is heavy for a few-hundred-entity sim and its Foundation is explicitly underfunded.

### Cited Findings
- Release cadence and size: Bevy 0.18 was published 2026-01-13 (174 contributors, 659 PRs); 0.19 on 2026-06-19 (261 contributors, 1,185 PRs); 0.20 on 2026-10-08 (227 contributors, 817 PRs). Bevy 0.17 was 2025-09-30, 0.16 was 2025-04-24. — [Bevy news index](https://bevy.org/news/)
- 0.18 highlights: `2d`, `3d`, `ui` cargo feature collections (plus mid-level ones like `2d_api`) to compile only what you need; `PanCamera` 2D pan/zoom controller in `bevy_camera_controller`; `get_components_mut` for safe multi-component mutable access; `remove_systems_in_set`; `EasyScreenshotPlugin`/`EasyScreenRecordPlugin`. The post says nothing about wasm, compile times, or headless. The editor is mentioned only as a future goal. — [Bevy 0.18](https://bevy.org/news/bevy-0-18/)
- 0.19 highlights: BSN (Bevy Scene Notation) via `bsn!`; resources are now stored as components on singleton entities; observers accept run conditions; `commands.delayed().secs(...)`; `contiguous_iter` exposing table slices (10,000-entity bulk update 4.88 µs vs 5.58 µs for normal iteration, 1.87 µs with AVX2); the render graph was replaced by ECS schedules; text moved from `cosmic_text` to `parley`. Wasm: dropped tasks now cancel on wasm via the `web-task` crate, so "Task drop semantics are finally identical on all platforms." No editor shipped; the post says the editor will be a Bevy app built on BSN. — [Bevy 0.19](https://bevy.org/news/bevy-0-19/)
- 0.20 highlights relevant to 2D: the sprite render backend was rebuilt on the 3D infrastructure; custom sprite shaders via `MaterialExtension2d`/`SpriteMaterial`; `ExtendedMaterial2d`. ECS: `chain_weak()`/`before_weak()`/`after_weak()` keep ordering only between systems whose data accesses conflict; `ScheduleBuildSettings::shuffle_seed` (debug feature) randomizes schedule order to surface hidden ordering bugs; panics in systems, commands and observers now become errors routed to the fallback error handler; `despawn_all` is 1.30x to 1.70x faster than individual despawns. Shaders moved from the custom WGSL dialect to WESL. Mesh shaders are not supported on web. Nothing on pixel art, tilemaps, camera scaling, wasm bundle size, or compile times. — [Bevy 0.20](https://bevy.org/news/bevy-0-20/)
- Migration churn, 0.19 to 0.20: roughly 75 sections. Sprite-relevant items: sprites moved to `Mesh2d` + `SpriteMaterial` and draw order for sprites sharing a Z level may change, so overlapping sprites need explicit Z; `SpriteMaterial` renamed `SpriteMeshMaterial`; `Camera2d` now defaults to `Tonemapping::Linear`; observers lose the `B: Bundle` generic (`On<Add, A>` becomes `On<Add<A>>`); `iter_many` now yields `Result`s; several built-in schedules switched to weak ordering, so code relying on implicit ordering may need explicit `.after(...)`; renames include `set_if_neq` to `set_if_different` and `Pointer<Press>` to `PointerPress`; GLSL support and `naga_oil` preprocessing removed. — [0.19 to 0.20 migration guide](https://bevy.org/learn/migration-guides/0-19-to-0-20/)
- Headless: `MinimalPlugins` is the official plugin group for "minimal, headless programs" and includes a schedule runner that stands in for the windowed event loop. — [MinimalPlugins docs](https://dev-docs.bevyengine.org/bevy/struct.MinimalPlugins.html)
- Headless testing pattern: replace `DefaultPlugins` with `MinimalPlugins`; window and input resources are then missing and have to be added by hand, which lets tests fake input; splitting the project into plugins lets each be tested in isolation. — [taintedcoders headless mode](https://taintedcoders.com/bevy/how-to/headless-mode); the Bevy repo also carries a `tests/how_to_test_apps.rs` example — [mirror](https://git.adamlamers.com/PublicArchive/bevy/raw/branch/create-pull-request/patch/tests/how_to_test_apps.rs)
- Pixel art, first-party example: `pixel_grid_snap` renders the world at 160x90 into an `Image::new_target_texture(...)` from an `InGameCamera` with `order: -1`, draws that texture as a `Sprite` on a second `RenderLayers` layer with an `OuterCamera`, uses `ImagePlugin::default_nearest()` for nearest sampling, `Msaa::Off` on both cameras, and a `fit_canvas` system that on `WindowResized` sets `projection.scale = 1. / h_scale.min(v_scale).floor()` to get integer scaling. It does not use `ScalingMode`. — [pixel_grid_snap.rs](https://raw.githubusercontent.com/bevyengine/bevy/main/examples/2d/pixel_grid_snap.rs)
- Camera scaling modes available in 0.20.0: `WindowSize`, `Fixed { width, height }`, `AutoMin { min_width, min_height }`, `AutoMax { max_width, max_height }`, `FixedVertical { viewport_height }`, `FixedHorizontal { viewport_width }`. — [ScalingMode docs, bevy 0.20.0](https://docs.rs/bevy/latest/bevy/camera/enum.ScalingMode.html)
- First-party 2D examples list includes Pixel Grid Snapping, Sprite Sheet, Sprite Slice, Sprite Tile, Texture Atlas, Tilemap Chunk, and Tilemap Chunk with Orientations (so there is now a first-party tilemap chunk example in the repo, separate from `bevy_ecs_tilemap`). — [examples README](https://raw.githubusercontent.com/bevyengine/bevy/main/examples/README.md)
- `bevy_ecs_tilemap`: compatibility table tops out at `bevy_ecs_tilemap 0.20` for Bevy 0.20, with `main` and `bevy-track` branches; about 1.3k stars, 1,096 commits, 67 open issues, 20 open PRs; chunked rendering (each chunk its own mesh), entity-per-tile, layers, sparse maps, GPU animations, isometric and hex maps, Tiled and LDtk integration examples; web examples run with WebGL2 by default and WebGPU behind `bevy/webgpu`. Y-sorting appears only in the `3d_iso` example, not as a listed feature. The table has a gap at Bevy 0.13. — [bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap)
- Wasm build path: `rustup target add wasm32-unknown-unknown`, `cargo install wasm-bindgen-cli`, `cargo build --release --example X --target wasm32-unknown-unknown`, run `wasm-bindgen`, serve the directory. WebGL2 is the default; `webgpu` feature overrides it and the result will not run in browsers without WebGPU. "Everything is single threaded" in the browser, which can cause audio stutter. Size guidance: build with `--profile wasm-release`, then `wasm-opt -Oz`. The README's only size numbers are from July 2022 for a small 3D project: 13.0 MB default without wasm-opt down to 4.8 MB with wasm-opt, `opt-level="z"`, fat LTO and `codegen-units=1`. — [examples README](https://raw.githubusercontent.com/bevyengine/bevy/main/examples/README.md)
- The official setup page's wasm advice: a `[profile.wasm-release]` inheriting `release` with `opt-level = "s"` and `strip = "debuginfo"`, then optional `wasm-opt -Os`. — [Bevy setup](https://bevy.org/learn/quick-start/getting-started/setup/)
- Compile-time mitigations from the same page: `opt-level = 1` for the project and `opt-level = 3` for dependencies in dev; `bevy/dynamic_linking` is called "the most impactful compilation time decrease" but must not ship; LLD is "much faster than the default linker"; mold is "up to 5x faster than LLD" on Linux; Cranelift (nightly) compiles "about 30% faster than LLVM" but "Wasm builds don't work yet" and macOS can crash; "The first full build of Bevy is slow. Later builds are fast." No absolute seconds are given. — [Bevy setup](https://bevy.org/learn/quick-start/getting-started/setup/)
- Bevy's own metrics site tracks compile stats on an AMD Ryzen 9 9950X / RTX 3060 / 64 GB machine, but the numbers are loaded dynamically and could not be read in this research. — [metrics.bevy.org](https://metrics.bevy.org)
- Foundation and funding: the Bevy Foundation was announced 2024-03-11 as a Washington State non-profit and was approved as a 501(c)(3) public charity on 2024-09-25. — [Bevy news index](https://bevy.org/news/)
- The donate page shows $11,621 per month from 216 members and 2 sponsors. — [Bevy donate](https://bevy.org/donate/)
- The Sixth Birthday post (2026-08-10) says the Foundation is "drastically underfunded for our ambitions", that existing paid developers are paid about 54% below the baseline salary the project wants to pay them, that a baseline editor "has not landed upstream" and is the next overriding priority, and gives August 2026 stats: 1,527 unique contributors (up from 1,291), 47,529 GitHub stars (up from 40,900), 13,543 commits, 6,656,483 crates.io downloads (up from 2,753,203). It does not mention 1.0 or a stability guarantee. — [Bevy's Sixth Birthday](https://bevy.org/news/bevys-sixth-birthday/)
- A critical practitioner view: LogLog Games (two-person studio, 100k+ lines of Rust, several shipped games) left Rust gamedev in April 2024, arguing "ECS is a tool, a very specific tool that solves very specific problems, and that does not come for free", that Bevy's parallel scheduling "doesn't maintain consistent ordering even across frames" so ordering constraints have to be added anyway, that runtime borrow panics (overlapping queries, `RefCell`) show up after refactors, that `hot-lib-reloader` was unreliable, and that "paying 8 second to compile 1600 lines of code" (a serde-heavy module) is never reasonable. — [Leaving Rust gamedev after 3 years](https://loglog.games/blog/leaving-rust-gamedev/)

### Inferences
- Bevy's model is a schedule of systems that run every tick over archetype storage. A discrete-event sim is a priority queue of timestamped events popped in order. Nothing in the 0.18 to 0.20 notes adds an event-queue runner; `commands.delayed().secs(...)` (0.19) is the closest thing and it is wall-clock based. The natural fit is to keep the DES as a plain Rust struct (own event heap, own tick function) and let Bevy be a viewer that reads snapshots, which is the same split as the "wasm sim plus separate viewer" architecture. That makes Bevy's ECS mostly irrelevant to the sim and relevant only to rendering.
- The 0.20 move to weak ordering plus `shuffle_seed` is the project admitting the LogLog point above; it is good for catching ordering bugs but it is an argument against putting deterministic sim logic in Bevy systems at all.
- Every ~5 months the agent will spend a session on a ~75-item migration guide. For a viewer-only Bevy dependency that cost is bounded; for a sim written in Bevy ECS it touches the core.
- The pixel-art path is well-trodden (low-res render target plus integer upscale) and bevy_ecs_tilemap tracks every release, so Bevy can render this genre. The open question is whether a 4.8 to 13 MB wasm download (2022 figure, 3D) is acceptable for a web-first game; no 2026 2D measurement was found.

### Gaps
- No 2025 or 2026 measurement of a 2D Bevy app's wasm size with WebGL2 versus WebGPU was found; metrics.bevy.org has the data but it was not readable via fetch.
- No absolute clean or incremental compile-time figures for current Bevy on a known machine were found in official sources.
- Wasm multithreading status in 0.20 beyond "everything is single threaded" was not found.
- Bevy's own test harness docs (beyond `MinimalPlugins` and the `how_to_test_apps` example) were not fetched.

## Macroquad/miniquad, Comfy, ggez, Fyrox, pixels+winit, egui

### Takeaway
Macroquad and miniquad are alive (releases in May, June and July 2026), small, and have the simplest wasm story in Rust; Comfy is archived (2025-11-24); ggez has no official web target; Fyrox hit 1.0.0 on 2026-03-24 with an export CLI that targets WebAssembly; `pixels` is a framebuffer-on-wgpu crate whose WebGL2 path is still labeled work in progress.

### Cited Findings
- Macroquad releases: 0.4.16 on 2026-07-30, 0.4.15 on 2026-05-20, 0.4.14 on 2025-03-20, 0.4.13 on 2024-08-26 (so a 14-month gap in 2024 to 2025, then two releases in 2026). — [crates.io API](https://crates.io/api/v1/crates/macroquad/versions)
- Miniquad releases: 0.4.11 on 2026-06-03, 0.4.10 and 0.4.9 on 2026-05-20, 0.4.8 on 2025-03-20. — [crates.io API](https://crates.io/api/v1/crates/miniquad/versions)
- Macroquad repo: 4.7k stars, 752 commits, 262 open issues, 88 open PRs. Wasm: `cargo build --target wasm32-unknown-unknown`, then an HTML page loads `mq_js_bundle.js` and calls `load("CRATENAME.wasm")`; no wasm-bindgen step. The README claims a clean build "takes only 16s on x230" (an old ThinkPad) and recommends `opt-level = 3` for dependencies in dev. Bundle size and texture filter mode are not stated on the page. — [macroquad](https://github.com/not-fl3/macroquad)
- Comfy: archived by the owner 2025-11-24, read-only; README says "Comfy is now archived until further notice." Timeline: 2024-09-08 maintainer announced a switch to C++ citing that frequent breaking changes in wgpu, winit and egui made maintenance a poor use of time; 2025-10-24 maintainer said they now use their own language (Rock, compiles to Lua) and Comfy's chances of returning are "essentially zero". 736 stars. — [darthdeus/comfy](https://github.com/darthdeus/comfy)
- ggez: `ggez = "0.10.0"`, wgpu-based, 4.7k stars, 2,813 commits; web, Android and iOS are "not officially supported" ("might work anyway"); the README points to good-web-game for running ggez up to 0.7 on wasm. — [ggez](https://github.com/ggez/ggez)
- Fyrox: 1.0.0 released 2026-03-24 (RC1 2025-10-21, RC2 2026-02-09), all posts by mrDIMAS. — [Fyrox blog](https://fyrox.rs/blog/). Repo: about 9.6k stars, 8,546 commits, 59 open issues, 1 open PR; includes `editor`, `editor-standalone`, `project-manager` and a `fyrox-autotile` crate; the README states the engine is written without AI/LLM use. — [FyroxEngine/Fyrox](https://github.com/FyroxEngine/Fyrox). A secondary review says 1.0 adds an export CLI "targeting PC, WebAssembly, or Android" and that every project now includes an export-cli crate. — [i-programmer](https://www.i-programmer.info/news/144-graphics-and-games/19015-fyrox-game-engine-reaches-version-100.html)
- pixels: "A tiny hardware-accelerated pixel frame buffer" on wgpu with pixel-boundary scaling and custom shaders (CRT etc.); you manage the window, event loop and input yourself (examples with winit, tao, fltk, egui, imgui); "DirectX 11, WebGL2, and WebGPU support are a work in progress"; a `minimal-web` WebGL2 example and `run-wasm` exist; 2.1k stars, 293 commits, 23 open issues. — [parasyte/pixels](https://github.com/parasyte/pixels)

### Inferences
- For a pixel-art viewer the three realistic Rust choices are: Bevy (heaviest, best docs), macroquad (lightest, no wasm-bindgen, its own JS loader, immediate-mode drawing), and pixels+winit (you own the loop, you blit a CPU framebuffer). Macroquad's own JS loader (`mq_js_bundle.js`) means it does not compose naturally with a TypeScript viewer; it wants to own the canvas.
- Comfy's archive notice is the clearest evidence on the "0.x churn" cost in this ecosystem: a maintainer with a shipped-games background gave up on tracking wgpu/winit/egui breakage. Any Rust renderer choice inherits that churn; a wasm sim with a TS renderer does not.
- Fyrox 1.0 is the only Rust engine with a stability promise and a single-maintainer bus factor; its 2D tooling (autotile) exists but was not evaluated here.

### Gaps
- egui status in 2026 was not researched; it is referenced only indirectly (Comfy's complaint about its breaking changes; pixels has an egui example).
- Macroquad's nearest-neighbor filter API and wasm bundle size were not confirmed from the README (the page omits them).
- Fyrox's actual wasm export quality and bundle size were not found.

## Architecture: Rust sim compiled to wasm, consumed by a TypeScript/Canvas viewer

### Takeaway
The toolchain is stable and maintained (wasm-bindgen 0.2.129 on 2026-09-25; wasm-pack under the wasm-bindgen org, maintained by drager), the state-passing options are well understood (serde-wasm-bindgen for structured snapshots, flat typed arrays or a view over wasm memory for hot paths), and several small open-source projects do exactly this split. The real costs are the boundary (serialization, two type systems, debugging across the line) and the fact that a sim this small (a few hundred entities) gives wasm no performance reason to exist.

### Cited Findings
- wasm-bindgen releases: 0.2.129 (Sep 25, 2026): experimental JSPI on emscripten, library MSRV 1.81. 0.2.128 (Sep 5): breaking, export symbols are hash-mangled per crate and "the library and CLI versions must match"; adds `--split-debug-info` and `--ts-typed-array-buffers`. 0.2.127 (Aug 8): WebGPU bindings updated to the August 2026 spec. 0.2.123 (Jun 8): breaking, `JsOption<T>` treats only `undefined` as empty. 0.2.122 (May 22): threading on recent nightlies needs `-Clink-arg=--export=__heap_base`; `-Cpanic=unwind` now emits exnref and needs Node 22.22.3+. 0.2.120 (Apr 28): `wasm64-unknown-unknown` (memory64). — [wasm-bindgen releases](https://github.com/rustwasm/wasm-bindgen/releases)
- rustwasm org sunset: a blog excerpt dated 2025-07-21 said the org "might be sunsetting this September", that "all major repos have been taken on by other maintainers", and that `wasm-bindgen`, `web-sys`, `wasm-pack` and `gloo` "will not be deprecated". Forum replies confirm wasm-pack moved to drager and web-sys lives inside the wasm-bindgen repo; the OP's claim that wasm-pack is being archived was not confirmed. — [users.rust-lang.org thread](https://users.rust-lang.org/t/future-of-rust-wasm/133089)
- wasm-pack today: hosted under the wasm-bindgen GitHub org, README says it "was started by ashleygwilliams and is maintained by drager", 7.3k stars, 1,433 commits, 332 open issues, no archived notice. — [wasm-bindgen/wasm-pack](https://github.com/wasm-bindgen/wasm-pack). Debian has 0.15.0 in testing as of June 2026 (search snippet, lower confidence). — [search result](https://tracker.debian.org/pkg/wasm-pack)
- serde-wasm-bindgen 0.6 is "the officially preferred approach" over wasm-bindgen's JSON-based serde support, with "much smaller code size overhead than JSON" and faster in most cases (no numbers given). Defaults that bite a TS consumer: `HashMap`/`BTreeMap` become JS `Map` unless `serialize_maps_as_objects(true)`; `u64`/`i64`/`usize`/`isize` become `number` only inside the safe-integer range and otherwise fail unless `serialize_large_number_types_as_bigints(true)`; `None` and unit become `undefined` unless `serialize_missing_as_null(true)`; bytes become `Uint8Array`. `Serializer::json_compatible()` flips the first three. — [serde-wasm-bindgen](https://github.com/RReverser/serde-wasm-bindgen)
- Generated TS types: ts-rs 12.0 derives a `TS` trait, `#[ts(export)]` writes `bindings/*.ts` during `cargo test`, honors serde `rename`/`tag`/`content`/`untagged`/`skip`/`flatten`/`default`, 64-bit ints default to `bigint` (`TS_RS_LARGE_INT`), MSRV 1.88. Known limits: `skip_serializing_if` only works with `#[serde(default)]`, `skip_deserializing` is ignored. — [ts-rs](https://github.com/Aleph-Alpha/ts-rs). typeshare (1Password): `#[typeshare]` attribute plus `typeshare --lang=typescript` CLI, targets TypeScript, Swift, Kotlin, Scala, with Go and Python experimental; about 3k stars, 590 commits, 73 open issues; tagged enums map to TS unions. — [typeshare](https://github.com/1Password/typeshare)
- Real project, pure-ABI style: caputchin-replay-rs compiles a deterministic sim to a freestanding `wasm32-unknown-unknown` cdylib with no wasm-bindgen glue and no imports; the replay ABI is `cap_alloc`/`cap_run` over `fn([u32; 4], &[i32], &[u8]) -> Verdict` (four u32 seed, i32 config slice, byte trace, verdict as three little-endian i32s); a `caputchin_live!` macro emits `live_new`/`live_step`/`live_state`/`live_trace`/`live_free` for a browser renderer so "the renderer drives the identical wasm the server replays" and float results match. The sim must avoid the clock, any RNG beyond the seed, and host calls. — [caputchin-replay-rs](https://docs.rs/caputchin-replay-rs)
- Real project, sandspiel: Rust sim in `crate/` built with `wasm-pack build`, JS in `js/` "glues things together", WebGL for display, fluid sim adapted from WebGL-Fluid-Simulation; 3.2k stars, 346 commits, 100 open issues; dev loop is `cargo watch -s 'wasm-pack build'` plus webpack. — [MaxBittker/sandspiel](https://github.com/MaxBittker/sandspiel)
- Real project, anguis (ECS snake): fixed-step sim at 64 Hz, "All gameplay randomness funnels through a seeded GameRng; verified by tests/determinism.rs. Calling rand::thread_rng() will silently break the determinism check"; web build via wasm-bindgen and wasm-opt. — [anguis](https://zff.dev/ARKye03/anguis)
- Real project, jugar-web: "Rust/WASM game logic with minimal JavaScript computation", JS limited to input and Canvas2D drawing, with a trace module for deterministic replay. — [jugar-web](https://docs.rs/jugar-web)
- Boundary-cost warning: the wator-wasm author notes that making 5,000 calls per frame back into JavaScript for rendering is inefficient and suggests batching draw calls or sharing a buffer. — [wator-wasm](https://github.com/csf/wator-wasm)
- Shared-memory pattern: tsoding's rust-browser-game has the wasm module render frames into its own memory and JavaScript copies them to the canvas. — [rust-browser-game](https://github.com/tsoding/rust-browser-game). deciduously/dots: "logic implemented in Rust targeting WebAssembly, with the rendering handled by JavaScript to a canvas element". — [dots](https://github.com/deciduously/dots)
- Host-import caveat: Ochagavía's Rocket port had to pass some float math functions in from JavaScript, so trig results came from the browser's `Math` library. — [Rocket on wasm](https://ochagavia.nl/blog/rocket-a-rust-game-running-on-wasm/)
- Wasm determinism: the spec says when an arithmetic operator returns NaN "there is nondeterminism in determining the specific bits of the NaN"; relaxed SIMD and some approximation operators may vary by platform. — [webassembly.org nondeterminism](https://www.webassembly.org/docs/nondeterminism/). Wasmtime can canonicalize NaNs and run relaxed SIMD deterministically at a performance cost. — [Wasmtime deterministic execution](https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html). DFINITY disables threads and SIMD and instruments wasm to normalize NaNs. — [DFINITY forum](https://forum.dfinity.org/t/how-is-deterministic-execution-of-webassembly-ensured/6335)
- Size reference point from a non-Bevy project: Vello's WebGL backend PR reports skipping the wgpu dependency removes about 3 MB from a wasm binary. — [vello commit](https://skia.googlesource.com/external/github.com/linebender/vello/+/refs/heads/ajakubowicz-fix-wgsl-source%5E)

### Inferences
- Three boundary styles, in increasing coupling:
  1. Opaque bytes with a hand-rolled ABI (caputchin style). No wasm-bindgen, smallest binary, identical module on server and browser, but every field layout is maintained twice by hand.
  2. wasm-bindgen plus serde-wasm-bindgen snapshot per tick, with ts-rs generating the TS types from the same Rust structs. Simplest for an agent; one `#[derive(Serialize, TS)]` keeps both sides in sync. Cost: one allocation and a JS object walk per snapshot. For a few hundred entities at a sim tick rate well below 60 Hz this is negligible.
  3. A flat `Vec<f32>`/`Vec<u32>` entity table exposed as a `Float32Array` view over wasm memory, refreshed each tick. Zero-copy, but the view is invalidated whenever wasm memory grows, and the viewer has to know the column layout.
- An event-stream API (the sim emits `Vec<Event>` per step and the viewer applies them) is strictly better for a DES than a full snapshot: the event log is the replay file, the viewer's interpolation needs only deltas, and the same stream can be written to a test fixture. A snapshot export is still worth having for save/load and for `insta` tests.
- Illustrative boundary (style 2), based on the wasm-bindgen, serde-wasm-bindgen and ts-rs findings above; not copied from any project:

  ```rust
  // sim/src/lib.rs
  use wasm_bindgen::prelude::*;
  use serde::Serialize;
  use ts_rs::TS;

  #[derive(Serialize, TS, Clone)]
  #[ts(export)]
  pub enum SimEvent {
      Moved { id: u32, x: i32, y: i32, at: u64 },
      StartedTask { id: u32, task: String, at: u64 },
  }

  #[wasm_bindgen]
  pub struct World { inner: sim::World }

  #[wasm_bindgen]
  impl World {
      #[wasm_bindgen(constructor)]
      pub fn new(seed: u64) -> World { World { inner: sim::World::new(seed) } }

      /// Advance to `until` (sim ticks) and return the events that fired.
      pub fn step_until(&mut self, until: u64) -> Result<JsValue, JsValue> {
          let events: Vec<SimEvent> = self.inner.run_until(until);
          let ser = serde_wasm_bindgen::Serializer::json_compatible();
          events.serialize(&ser).map_err(|e| e.into())
      }

      pub fn snapshot_bytes(&self) -> Vec<u8> { self.inner.to_bytes() }
  }
  ```

  ```ts
  // viewer/src/sim.ts
  import init, { World } from "../../sim/pkg/sim.js";
  import type { SimEvent } from "../../sim/bindings/SimEvent"; // from ts-rs
  await init();
  const world = new World(42n);
  const events = world.step_until(600n) as SimEvent[];
  ```

  Note the `bigint` for `u64` (ts-rs default, and serde-wasm-bindgen's safe-integer rule); using `u32` tick counters avoids that friction.
- Debugging across the boundary: panics in wasm surface as opaque `RuntimeError: unreachable` unless `console_error_panic_hook` is installed; the 0.2.128 `--split-debug-info` and the `--ts-typed-array-buffers` flags are the current tooling levers. (Inference from release notes; no 2026 debugging write-up was found.)
- Hot reload: the Rust side has no hot reload in the browser; the dev loop is `cargo watch -s 'wasm-pack build'` (sandspiel) and a full page reload. With the sim stateless-by-construction (seed plus event log), reload cost is "re-run the log to tick N", which is cheap for a few hundred entities. The TS viewer keeps Vite HMR.
- Performance is not an argument for this architecture at this scale. A few hundred entities with an event heap is well within TypeScript's budget. The arguments that remain are: type-level determinism guarantees (integers, no implicit float coercion, no undefined), Rust's testing tools (`insta`, `proptest`), and a native build for Steam that does not need a browser.

### Gaps
- No published benchmark of serde-wasm-bindgen versus `JSON.parse(serde_json)` with numbers was found; the project README only says "faster in most cases".
- No 2024 to 2026 postmortem of a shipped commercial game using a Rust wasm sim with a TypeScript renderer was found; the examples above are small open-source projects and one unpublished-scale framework (caputchin).
- Bundle size for a non-graphics Rust sim crate (no wgpu) compiled to wasm was not found; the Vello 3 MB figure is for wgpu's share, not a floor for a sim crate.

## Alternative: Rust sim as a native process with a web viewer over WebSocket, then wasm for deployment

### Takeaway
No source describing this exact pattern for a game was found, so the following is inference from the toolchain facts above. It is attractive for the headless dev loop (native `cargo test`, no browser in the container) and the same event-stream API serves both transports, but it adds a third surface (the socket protocol) to keep in sync with the Rust types and the TS types.

### Cited Findings
- caputchin-replay-rs demonstrates the inverse arrangement: the same wasm module runs on a server (replay) and in a browser (live stepping), so the "one artifact, two hosts" property is achievable with wasm alone. — [caputchin-replay-rs](https://docs.rs/caputchin-replay-rs)
- Ebitengine 2.10 ships "application virtualization": the game runs as a guest process while a host process injects input, advances a set number of ticks and captures rendering and audio, aimed at "automated testing by programs and AI agents"; the guest needs no window or GPU, but the host still needs a graphics environment and "there is no software renderer for CI environments without a GPU". — [Ebitengine 2.10 notes](https://ebitengine.org/en/documents/2.10.html)
- The research found no Rust-specific equivalent of Ebitengine's host/guest harness.

### Inferences
- If the sim exposes `step_until(tick) -> Vec<SimEvent>` and `snapshot() -> bytes`, a 50-line axum/tungstenite server that pushes the event vector over a WebSocket is enough for a browser viewer to render against a native process. The viewer code that applies events is identical whether events arrive from `world.step_until()` in wasm or from `socket.onmessage`. The protocol can reuse the ts-rs-generated types if the native process serializes with `serde_json`.
- For an agent in a headless container the win is that the entire sim test loop (`cargo test`, `insta`, `proptest`) never touches wasm or a browser; wasm becomes a deploy target checked once in CI (build plus a Playwright smoke test), not a per-iteration cost.
- The cost is a third thing to keep consistent (socket framing, reconnect, tick pacing) that exists only for development. Whether that is cheaper than a `wasm-pack build` per iteration depends on how slow the wasm build is for the sim crate; no measurement was found.

### Gaps
- No write-up of a game project using a native sim over WebSocket for development and wasm for shipping was found.
- No measurement of `wasm-pack build` incremental time for a mid-size sim crate was found.

## Determinism and replay: Rust versus TypeScript, and testing tools

### Takeaway
Rust makes the sources of nondeterminism explicit and mostly compile-visible (randomized `HashMap` order, non-portable `StdRng`/`SmallRng`, platform-dependent float distributions), and has first-class snapshot and property testing (`insta`, `proptest`). TypeScript has no equivalent traps in the language runtime but also no equivalent guard rails; its advantage is that one JS engine runs both the dev tests and the shipped game.

### Cited Findings
- `std::collections::HashMap` randomizes per `RandomState`; the stdlib source says "exposing a deterministic iteration order allows a form of DOS attack. To counter that we increment one of the seeds on every RandomState creation". — [std hash/random.rs](https://doc.rust-lang.org/stable/src/std/hash/random.rs.html)
- Mitigation options named in practice: a fixed-seed or custom deterministic hasher, or an order-preserving map such as indexmap. — [search summary, GoudEngine issue](https://github.com/aram-devdocs/GoudEngine/issues/802)
- The `rand` book: "all public items not marked non-portable are expected to be portable"; `SmallRng` and `StdRng` are explicitly non-portable and "may produce different results across platforms and library versions"; minor releases may make value-breaking changes to portable items (noted in the CHANGELOG); `rand_distr` distributions using `f32`/`f64` "may not be portable, because floating-point rounding and transcendental function implementations vary by platform", and the project prefers `libm` over `std` for transcendental functions. — [rand reproducibility](https://rust-random.github.io/book/crate-reprod.html)
- Deterministic-simulation frameworks in Rust intercept `getrandom`, `rand` and std `HashMap` seeding to make async runs reproducible (refault), and the general recipe is single-threaded execution, every RNG seeded, no physical clocks. — [refault](https://docs.rs/crate/refault/latest); [S2 DST post](https://s2.dev/blog/dst)
- Wasm float semantics: IEEE 754 except NaN bit patterns and relaxed SIMD (see the architecture section). — [webassembly.org nondeterminism](https://www.webassembly.org/docs/nondeterminism/)
- `insta`: snapshot testing with `assert_yaml_snapshot!` (JSON, YAML, TOML, CSV formats), inline or external snapshot files, redactions for values that change between runs, and `cargo-insta` interactive review with terminal diffs. — [insta.rs](https://insta.rs/)
- `proptest`: QuickCheck-family property testing inspired by Hypothesis; generation and shrinking defined per value rather than per type; failing inputs are reduced to a minimal case; has a "Failure Persistence" chapter (regression seed files). — [proptest intro](https://proptest-rs.github.io/proptest/intro.html)
- The anguis project's pattern: a `tests/determinism.rs` that runs the same seed for N ticks twice and compares state; "Calling rand::thread_rng() will silently break the determinism check". — [anguis](https://zff.dev/ARKye03/anguis)
- Bevy 0.20's `ScheduleBuildSettings::shuffle_seed` exists specifically to surface hidden system-ordering bugs. — [Bevy 0.20](https://bevy.org/news/bevy-0-20/)

### Inferences
- What Rust makes easy: integer-only sim math is natural (`i32`/`u32` tile coordinates, `u64` ticks); `BTreeMap`/`BTreeSet` give ordered iteration by type choice; `rand_chacha`/`rand_pcg` are portable by the book's rule; a `#[derive(Serialize)]` world plus `insta::assert_yaml_snapshot!` gives a reviewable golden file per scenario in one line; `proptest` can generate random event sequences and shrink the one that breaks an invariant.
- What Rust makes hard: `HashMap` is the default collection in examples and in model output, so an agent will reach for it; the fix is a clippy lint or a `type Map<K,V> = BTreeMap<K,V>` alias in the sim crate. Float use in `rand_distr` is a portable-looking trap.
- What TypeScript makes easy: `Map` iteration is insertion-ordered by spec, all numbers are f64 with IEEE semantics in every engine, `vitest` has `toMatchFileSnapshot`, and a seeded PRNG is a 10-line function. There is no compiler-enforced separation between sim code and anything that touches `Math.random()` or `Date.now()`; that discipline has to come from lint rules and code review.
- What TypeScript makes hard: integer math requires `| 0` or `Math.trunc` discipline and there is no `u32` type to make overflow explicit; a structural type system cannot forbid `number` from carrying a fractional value into tile math.
- Replay and lockstep across native and wasm builds in Rust: integer-only sim logic sidesteps the NaN and transcendental issues entirely; if floats are needed, keep them out of state that influences branching, or use fixed-point.

### Gaps
- No controlled comparison of determinism bug rates in Rust versus TypeScript sims was found; the above is a comparison of what each language enforces, not of outcomes.
- Fixed-point crates for Rust (e.g. `fixed`) and their 2026 status were not researched.

## Other compiled stacks: Ebitengine, MonoGame/FNA/KNI, HaxeFlixel, Zig, Odin, Nim, and hosting a Rust core in Godot/Unity

### Takeaway
Ebitengine (Go) is the strongest non-Rust compiled option for an agent-driven, web-first project: one-command wasm builds, pure-Go desktop builds since 2.10 (2026-09-08), and a new host/guest "virtualization" harness aimed at automated testing by AI agents (though it still needs a GPU on the host). MonoGame has no official web target (the issue was closed "not planned"); the KNI fork adds a Blazor WebAssembly target. HaxeFlixel 6.x is active and HTML5-native. Zig (Mach) still lists web as "soon", Odin+raylib web builds work via emscripten with caveats, and Nim's main raylib wrapper (naylib) was archived 2026-08-07.

### Cited Findings
- Ebitengine v2.10.0 released 2026-09-08, patches to v2.10.5 on 2026-10-10; requires Go 1.25+; macOS and Linux/BSD builds no longer need Cgo ("can now be built with Go alone", OS APIs via PureGo); mobile and some consoles still need Cgo. — [Ebitengine 2.10 notes](https://ebitengine.org/en/documents/2.10.html)
- Ebitengine VM mode: opt in with the `ebitenginevmguest` build tag or `ebiten.RunGameOptions.VMGuestEndpoint`; `EBITENGINE_VM_ENDPOINT` names the host; host uses `exp/vmhost`; host and guest must be the same Ebitengine version; host injects input, advances N ticks, captures rendering and audio; guest has no window or GPU, but "the host still needs a graphics environment" and "there is no software renderer for CI environments without a GPU". Also new: `exp/shaderprecomp` for shader precompilation. — [Ebitengine 2.10 notes](https://ebitengine.org/en/documents/2.10.html); [v2.10.0 blog](https://ebitengine.org/en/blog/v2.10.0.html)
- Ebitengine web: `GOOS=js GOARCH=wasm go build -o yourgame.wasm`, copy `$(go env GOROOT)/lib/wasm/wasm_exec.js`, or `go run github.com/hajimehoshi/wasmserve@latest ./path` for dev; embed in an iframe with `allow="autoplay"` for Chrome audio. The page gives no binary size. — [Ebitengine WebAssembly](https://ebitengine.org/en/documents/webassembly.html)
- Ebitengine wasm size, indirect: the piweb project's README says its Snake game is 3 MB (0.9 MB gzipped) "instead of 10 MB", presumably versus a standard Ebitengine build, but it does not name Ebitengine directly. — [piweb](https://pkg.go.dev/github.com/elgopher/piweb)
- MonoGame: "doesn't officially support web builds"; the issue requesting web support was closed as "not planned"; community routes are KNI (BlazorGL target) or Blazor WebAssembly with MonoGame 3.8.2+. — [Wavedash MonoGame page](https://docs.wavedash.com/engines/monogame.md)
- KNI: "a MonoGame-compatible fork that adds a Blazor WebAssembly target (`BlazorGL`)"; `Game`/`Initialize`/`Update`/`Draw` code unchanged; .NET 8 SDK or newer; `dotnet publish -c Release` produces `wwwroot`. No bundle size, load time or runtime size is given. — [Wavedash KNI page](https://docs.wavedash.com/engines/kni.md)
- FNA: no source was found saying FNA runs in a browser; a shader-compiler package notes its FNA (D3D9) output is for desktop downloads because "a browser cannot render DXBC or D3D9 bytecode". — [ShadowDusk.Wasm](https://packages.nuget.org/packages/ShadowDusk.Wasm/)
- HaxeFlixel: 6.0.0 released 2025-01-31; 6.1.0 (2025-04-28) fixed a "flickering issue when sprites have varied colors in the html5 renderer"; 6.2.0 is the newest (about 3 months before 2026-10-23, so around July 2026) with a sound refactor, `FlxMatrixSprite` and tilemap ray methods. — [flixel 6.1.0 changelog](https://development-lib.haxe.org/p/flixel/6.1.0/changelog); [flixel versions](https://lib.haxe.org/p/flixel/versions/)
- Odin: karl-zylinski/odin-raylib-web targets browser builds (145 stars, last push 2026-04-17 per a tracker); atomicptr's starter requires the latest emsdk; the original proof-of-concept notes the default vendor raylib bindings do not work for wasm because they depend on `core:c`; a forum PSA warns to test the wasm export often because stack size is a common wasm-only failure. Odin's September 2026 newsletter covers compiler work (inline asm templates), not web. — [odin-raylib-web](https://gittrend.io/repo/karl-zylinski/odin-raylib-web); [raylib_wasm_odin](https://proxy-ga.blitzz.co/proxy/123456/github.com/Caedo/raylib_wasm_odin/tree/master); [Odin forum PSA](https://forum.odin-lang.org/t/psa-having-new-weird-wasm-issues-but-not-with-your-desktop-binary/1229); [Odin newsletter 2026-09](https://odin-lang.org/news/newsletter-2026-09/)
- Zig/Mach: the site lists desktop support with "WebAssembly (soon) and Mobile (future)", "In development since 2021", "We're not there yet"; it requires a nominated Zig version; current branch 0.4. — [machengine.org](https://machengine.org/); [Mach v0.4 page](https://machengine.org/v0.4)
- Nim: naylib ("Yet another raylib Nim wrapper", 364 stars) lists WebAssembly as "Supported, Tested" but the repository "was archived by the owner on Aug 7, 2026". — [planetis-m/naylib](https://github.com/planetis-m/naylib)
- Hosting a Rust core in Godot: godot-rust's docs say wasm export support "is still a work-in-progress and is not yet well tested", opt-in, threads on by default with an `experimental-wasm-nothreads` feature; a Godot forum thread says official export templates do not enable GDExtension for web, so custom templates are needed. — [godot crate docs](https://docs.rs/godot); [Godot forum](https://forum.godotengine.org/t/what-is-the-status-of-web-export-if-my-game-uses-c-from-gdextension/125375)
- Comparative anecdote on Go versus Rust loop speed (AI-generated article, cited people are real): `go build` about 1 to 2 s and `go test` 1 to 3 s with caching; Rust single-file type-checking "takes milliseconds in practice" but no full-build figure; `tsc --noEmit` "fast". — [yage.ai language choice](https://yage.ai/share/ai-language-choice-en-20260528.html)

### Inferences
- Ebitengine is the only stack in this list that offers all three of: a single-command wasm build, a pure-Go headless-friendly desktop build, and an official automation harness for agents. Its remaining gap is the same as Bevy's: no software renderer, so pixel-level screenshot tests still need a GPU or a browser. Go's compile speed is a real advantage for an agent loop over Rust, and Go has no equivalent of Bevy's 0.x churn (Ebitengine is on 2.x with semver patches).
- C#/MonoGame is what Stardew used, but for a web-first project it means the KNI fork, Blazor, and a .NET runtime download whose size no source quantified. It is the weakest fit for "web primary".
- HaxeFlixel is HTML5-native and active, but its agent-friendliness was not evaluated and Haxe is a much smaller training-data language than Rust, Go or TypeScript (inference; no benchmark includes Haxe).
- Zig, Odin and Nim are not ready for a web-first project in 2026 by the evidence here: Mach has no shipped web target, Odin needs emsdk and custom bindings, naylib is archived.
- Unity could host a Rust core only as a native plugin on desktop; for web it would need the core compiled into the Unity wasm build, which no source here covers. Godot via godot-rust works on desktop and is experimental on web.

### Gaps
- Ebitengine wasm binary size from an official source; the only number is from a competitor's README.
- KNI's web runtime download size and load time.
- HaxeFlixel 6.0.0's full changelog and any current HTML5 caveats beyond the 6.1.0 flicker fix.
- Any 2026 Mach release note confirming or denying shipped web support.
- Unity native-plugin plus wasm path for a Rust core.

## Evidence on AI coding agents writing Rust versus TypeScript

### Takeaway
On repository-level issue benchmarks the picture is mixed and benchmark-dependent: SWE-bench Multilingual (300 tasks) had Rust as the best-resolved language (58%) and JS/TS near the bottom (35%) for SWE-agent + Claude 3.7, while Multi-SWE-bench (NeurIPS 2025) had every non-Python language in single digits to mid-teens with Rust (max 15.9%) above TS (max 11.6%). The strongest 2026 practitioner evidence (Bun's 535k-line Zig to Rust port by 64 Claude agents in 11 days) supports "the compiler is the agent's reviewer", and the strongest counter-evidence is the same project's criticism (unsafe-block count, "unreviewed slop"). No controlled study of full agentic loops by language was found.

### Cited Findings
- SWE-bench Multilingual: 300 tasks from 42 repos across 9 languages; SWE-agent + Claude 3.7 Sonnet resolved 128/300 (42.67%) at a $2.50 per-task cost limit. Per language: Rust 25/43 (58.14%), Java 23/43 (53.49%), PHP 21/43 (48.84%), Ruby 19/44 (43.18%), JavaScript/TypeScript combined 15/43 (34.88%), Go 13/42 (30.95%), C/C++ 12/42 (28.57%). The same model scored 63% on SWE-bench Verified (Python). — [swebench.com/multilingual](https://swebench.com/multilingual)
- Multi-SWE-bench (arXiv 2504.02605, NeurIPS 2025): instances per language TypeScript 224, JavaScript 356, Go 428, Rust 239, Java 128, C 128, C++ 129. Best Rust result: Claude 3.7 Sonnet with OpenHands 15.90%; best TS result: Claude 3.5 Sonnet with OpenHands 11.61% (Claude 3.7 with MSWE-agent 11.16%). Python reference for Claude 3.7 + OpenHands: 52.20%. The paper states "TS and JS consistently yield the lowest resolved rates", that Go and Rust "generally beat TS and JS but trail Java", and attributes TS/JS difficulty to dynamic typing and async behavior and Rust/Go/C/C++ difficulty to memory management and build systems. Hard-level issues resolve near zero in every language. — [Multi-SWE-bench](https://arxiv.org/html/2504.02605)
- Aggregator snapshots of SWE-bench Multilingual with newer models (lower confidence, self-reported lab scores): a February 2026 list has Gemini 3 Flash 72.7%, Claude 4.6 Opus 72.0%, Claude 4.5 Opus 70.7%, Gemini 3 Pro 68.7%, GPT-5.2 (high) 66.7%; a July 2026 list has Claude Opus 5 at 89.5%, Claude Opus 4.8 at 84.4%, Claude Sonnet 5 at 78.3%. Neither shows a per-language split. — [llmrun](https://llmrun.dev/benchmark/swe-bench-multilingual?show=all); [BenchLM](https://benchlm.ai/benchmarks/swe-bench-multilingual)
- RustEvo² (arXiv 2503.16922, March 2025): 588 tasks from real API changes across Rust 1.71.0 to 1.84.0 (stabilizations, signature changes, behavioral changes, deprecations); Claude 3.7 Sonnet topped 10 models at 65.3% Pass@1, the rest 40.5% to 57.5%; RAG improved success by 13.5% on average for APIs released after model training. — [RustEvo²](https://arxiv.org/abs/2503.16922v1)
- RustAssistant (Microsoft Research, ICSE 2025): LLM repair of Rust compilation errors, including borrow-checker errors, peaking at 74% accuracy on real-world errors. — [ICSE 2025 entry](https://conf.researchr.org/details/icse-2025/icse-2025-research-track/9/RustAssistant-Using-LLMs-to-Fix-Compilation-Errors-in-Rust-Code); [arXiv 2308.05177](https://arxiv.org/pdf/2308.05177)
- RustRepoTrans (arXiv 2411.13990): in repository-level translation to Rust, compilation errors make up most failures and a single translation can produce many errors at once. — [RustRepoTrans](https://arxiv.org/pdf/2411.13990)
- AdaTrans (C to Rust) argues LLMs "often ignore compiler feedback and sometimes add `unsafe` blocks to silence the borrow checker". — [AdaTrans](https://arxiv.org/pdf/2606.31706)
- Aider's polyglot benchmark covers 225 Exercism exercises in C++, Go, Java, JavaScript, Python and Rust (no TypeScript); top scores gpt-5 (high) 88.0% on 2025-08-23, o3-pro 84.9%, gemini-2.5-pro 83.1%; no per-language breakdown is published. — [Aider leaderboards](https://aider.chat/docs/leaderboards/)
- Bun's Zig to Rust port (The Pragmatic Engineer, 2026-07-16): 535,496 lines of Zig; 11 days from planning to merge (about 3 hours prep writing a 600-line PORTING.md, about 2 days parallel porting, about 12 hours crate-by-crate compiler fixes, about 2 days local tests, about 3 days CI); 64 Claude agents in 4 worktree shards of 16; about 6,500 commits; about $165,000 at API prices (5.9B uncached input tokens, 690M output, 72B cached reads); every commit checked by two adversarial reviewer agents; merged only after the full test suite passed. Jarred Sumner: "Compiler errors are a better feedback loop than a style guide." Caveats the article lists: needs an engineer who knows the codebase, a very robust test suite, and heavy token spend. — [Pragmatic Engineer](https://blog.pragmaticengineer.com/the-pulse-what-can-we-learn-from-buns-rapid-rust-rewrite-with-ai/)
- Criticism of the same port: Zig's creator Andrew Kelley called it "unreviewed slop" (devclass, 2026-07-15); a secondary source counts 13,044 `unsafe` blocks in the generated Rust versus 73 in comparable hand-written Rust (unverified). — [devclass](https://www.devclass.com/ai-ml/2026/07/15/zig-creator-calls-buns-claude-rust-rewrite-unreviewed-slop/5271749); [search summary](https://www.theregister.com/a/5297549)
- Practitioner account (Tigran Bayburtsyan): the Rust compiler acts as an "automatic expert reviewer"; errors carry stable codes (E0106, E0373, E0502) with line/column and suggestions, so the agent loop is write, `cargo check`, read, restructure, re-check, "measured in seconds"; async code with mutexes "might need several iterations"; C++ template backtraces are harder for agents to parse; no numbers, no compile-time discussion, TypeScript not mentioned. — [Coding Rust with Claude Code and Codex](https://dev.to/tigranbs/coding-rust-with-claude-code-and-codex-2c4j)
- Practitioner account (Tarek Ziadé, 2025-12-20): built a Rust project (rustnn) mainly with Claude Code; credits CLAUDE.md with constraining the agent. — [all the code are belong to claude](https://blog.ziade.org/ai/2025/12/20/all-the-code-are-belong-to-claude/)
- Secondary claim: Steve Klabnik reportedly used Claude Code to write about 100,000 lines of Rust in 11 days; MultiPL-E (2022) found static typing had almost no effect on one-shot accuracy and Rust scored significantly below Python and JavaScript; the article estimates roughly 60 to 70% of language-performance variance is training-data volume. The article is labeled 100% AI-generated. — [yage.ai](https://yage.ai/share/ai-language-choice-en-20260528.html)
- Compile-time cost on the agent loop, Rust side: see the Bevy setup page (dynamic linking, mold, cranelift) and the LogLog "8 seconds for 1,600 lines" complaint above. — [Bevy setup](https://bevy.org/learn/quick-start/getting-started/setup/); [LogLog Games](https://loglog.games/blog/leaving-rust-gamedev/)

### Inferences
- The two issue benchmarks disagree on magnitude but agree on ordering: Rust is at or above TS/JS for the same model and scaffold. The absolute numbers are low everywhere outside Python and are from Claude 3.5/3.7-era models; 2026 aggregate scores are two to three times higher but have no language split, so the 2026 ordering is unknown.
- The strongest signal for this project is structural, not benchmark-derived: a sim crate with no graphics dependencies is the easy end of Rust (no async, no lifetimes across threads, mostly owned data and integers), and it is exactly the code where `cargo check` plus `insta` gives the agent a tight loop. The hard end of Rust (wgpu/winit/async) is the renderer, which this architecture leaves in TypeScript.
- Compile time matters more than benchmarks for an agent: a sim crate with `serde`, `rand_chacha`, `ts-rs` and no engine should check in well under the Bevy-scale times, but no measurement was found; the agent should measure it in the first session and treat Bevy's "slow first build" as a renderer-only cost.
- Bun's result shows that scale is reachable with adversarial review agents and a test suite, and its criticism shows the failure mode: compiling is not the same as reviewed. For a sim whose correctness is checked by determinism tests and snapshot fixtures, the test suite is the review.

### Gaps
- No benchmark splits TypeScript from JavaScript on SWE-bench Multilingual, and none publishes 2026 per-language scores for current models.
- No controlled study of agent success in a full edit-compile-test loop by language (the "Generative Compilation" July 2026 preprint proposes compiler feedback during decoding but its results were not verified). — [emergentmind listing](https://www.emergentmind.com/papers/2607.13921)
- No measurement of agent iteration count or wall-clock per task in Rust versus TypeScript for game-sim code specifically.
- The 13,044-unsafe-blocks figure for Bun's port could not be traced to a primary source.

## Cost of two languages in one repo for an agent, and mitigations

### Takeaway
The concrete costs are a second toolchain (`rustup` target, `wasm-bindgen-cli` pinned to the crate's `wasm-bindgen` version since 0.2.128, `wasm-opt`), a type definition that would otherwise be duplicated, and a boundary where errors are opaque. ts-rs or typeshare removes the duplication by generating the TS types from the serde-annotated Rust structs; the toolchain cost is one `package.json` script; the boundary cost is permanent.

### Cited Findings
- Since wasm-bindgen 0.2.128 "the library and CLI versions must match" (schema bump), so the CLI has to be pinned alongside the crate. — [wasm-bindgen releases](https://github.com/rustwasm/wasm-bindgen/releases)
- ts-rs: `#[ts(export)]` emits `bindings/<Type>.ts` on `cargo test`, honoring serde attributes, with `TS_RS_EXPORT_DIR` to choose the output directory; MSRV 1.88. — [ts-rs](https://github.com/Aleph-Alpha/ts-rs)
- typeshare: single CLI pass over the crate, `--lang=typescript --output-file=...`; also emits Swift/Kotlin/Scala, useful if a native mobile viewer is ever wanted. — [typeshare](https://github.com/1Password/typeshare)
- serde-wasm-bindgen's defaults (JS `Map`, `undefined` for `None`, `bigint` thresholds) differ from JSON; `json_compatible()` aligns them with what ts-rs types describe. — [serde-wasm-bindgen](https://github.com/RReverser/serde-wasm-bindgen)
- Comfy's maintainer cited "frequent breaking changes in wgpu, winit, and egui" as a reason to stop; those crates are renderer dependencies, not sim dependencies. — [darthdeus/comfy](https://github.com/darthdeus/comfy)
- Multi-SWE-bench attributes Rust/Go/C/C++ difficulty partly to "complex build systems". — [Multi-SWE-bench](https://arxiv.org/html/2504.02605)

### Inferences
- Costs specific to an agent: (1) every cross-boundary change touches a Rust file, a generated TS file and a TS consumer, so a `pnpm check` that runs `cargo test` (to regenerate bindings) before `tsc` is needed to keep the agent from reading stale types; (2) context switching is real but bounded if the repo has a hard rule that the sim crate has no DOM, no wasm-bindgen types in its public API except in a thin `wasm/` adapter crate, and no renderer concerns; (3) debugging a wrong number on screen requires deciding which side is wrong, which a snapshot test on the Rust side and a fixture-driven render test on the TS side can answer without stepping across the boundary.
- Mitigations in order of value: ts-rs (or typeshare) for types; an event-log fixture format shared by `insta` on the Rust side and vitest file snapshots on the TS side; a single `just`/`pnpm` entry point that builds wasm, regenerates bindings and typechecks; pinning `wasm-bindgen` and `wasm-bindgen-cli` to the same version in one place.
- If the renderer also moves to Rust (Bevy or macroquad), the two-language cost disappears but the churn cost from the Comfy/LogLog findings arrives, and the TypeScript projects the developer already has stop sharing code with this one.

### Gaps
- No published case study measuring agent productivity in a mixed Rust/TypeScript repo versus a single-language one.
- Whether ts-rs 12 handles the specific enum shapes of a DES event log (tagged enums with tuple variants) without manual `#[ts(type = "...")]` overrides was not verified.
