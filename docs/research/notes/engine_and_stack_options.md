# Engine and stack options for an agent-built, headless-developed 2D top-down web game (as of October 2026)

Scope note: this covers engines, map/asset tooling, architecture, headless testing and deployment. NPC AI and LLM-driven characters are out of scope. Research budget was ~15 tool calls, so several items below are flagged as unverified or gaps rather than guessed.

## Key Question 1: JavaScript/TypeScript 2D engines (Phaser, Excalibur, Kaplay, PixiJS, melonJS)

### Takeaway
All five are alive in 2026, but they differ sharply in fit for a code-first, no-screen workflow. Excalibur is the only one of them whose own release process now uses Playwright-driven visual tests and vitest unit tests, and it ships first-party Tiled and LDtk loaders; Phaser 4 is the largest and newest-rewritten but just six months old (4.0.0 April 2026) with a 403-blocked release post I could not read; Kaplay's next major (4000) has been in alpha for over a year with frequent breaking changes; PixiJS is a renderer only and is the most mature piece of infrastructure here; melonJS is maintained but small.

### Cited Findings

**Phaser**
- Phaser v4.0.0 "Caladan" was released 10 April 2026 and v4.1.0 "Salusa" on 30 April 2026, per Phaser's own v4 download page — [phaser.io/download/phaser4](https://phaser.io/download/phaser4)
- The current stable release listed on Phaser's download page is v4.2.1 "Giedi", 9 July 2026 — [phaser.io/download/stable](https://phaser.io/download/stable)
- v4.2.1 was described as a patch release aimed at runtime bug fixes in the v4 line — [GameDev.net](https://gamedev.net/news/4825-phaser-v421-released/)
- Phaser 4 is described as a major update with a new renderer, filters, lighting and GPU-powered objects — [GameDev.net](https://gamedev.net/news/2759-phaser-4-released/) (summary only; the full article returned 403 when fetched)
- Search results did not show any Phaser release newer than 4.2.1 as of this research; the official Phaser 4 release news post at phaser.io returned 404 when fetched, so renderer internals, TypeScript typing changes, bundle size, and v3 migration specifics are unverified here.

**Excalibur.js**
- Releases listed on GitHub (newest first): v0.32.0 (23 Dec), v0.31.0 (26 Sep), v0.30.3 (02 Jan), v0.30.0 (13 Dec), v0.29.0 (20 Feb) — [GitHub releases](https://github.com/excaliburjs/Excalibur/releases)
- v0.32.0 recording filenames (`recording_20251223_...`) place it on 23 December 2025. Theme: "developer experience and performance". Breaking changes: realistic physics bodies sleep by default, `sleepBias` default 0.9 to 0.5. Event types changed from `type` to `interface` so game code can use declaration merging for custom events. Test suite was split into `unit` and `visual` tests; Playwright upgraded to 1.55.1 then 1.57.0; Vitest UI support for browser testing; Dev Container support added — [v0.32.0 release](https://github.com/excaliburjs/Excalibur/releases/tag/v0.32.0)
- v0.31.0 added camera strategy ordering, a SoundManager for channel-based audio, Query filtering by components and tags, `Clock.clearSchedule(id)`; tests migrated to vitest — [GitHub releases](https://github.com/excaliburjs/Excalibur/releases)
- v0.29.0 was a large breaking release (migration guide) adding `World.query([...])`, `queryTags()`, systems receiving a world in the constructor, scene transitions and loader API, `ex.Debug` drawing helpers, experimental coroutines, antialiasing options for pixel art, and was said to bring the project "closer to v1". No 1.0 release is listed — [GitHub releases](https://github.com/excaliburjs/Excalibur/releases)
- `@excaliburjs/plugin-ldtk` loads LDtk maps via an `LdtkResource` through `ex.Loader`; currently supports only 2D tile maps, targets the latest LDtk release (1.5.3) and warns on older project versions — [Excalibur LDtk plugin docs](https://excaliburjs.com/docs/ldtk-plugin)
- `@excaliburjs/plugin-tiled` (`TiledResource`) supports orthogonal and isometric maps; staggered isometric and hexagonal maps parse but do not render. Older v0.28-and-earlier docs describe a separate package with a different class name — [Excalibur Tiled plugin docs](https://excaliburjs.com/docs/tiled-plugin)
- No 2026 Excalibur release was found; an April 2026 podcast lists Excalibur.js as a sponsor describing itself as "the free and open source friendly TypeScript 2D game engine for the web" — [TypeScript.fm episode](https://podcastaddict.com/typescriptfm-the-friendly-show-for-typescript-developers/episode/221772990)

**Kaplay (formerly Kaboom)**
- Kaboom was created by Tiga Wu at Replit; after Tiga was laid off, Replit stopped development; the community continued under the Kaplay name because Replit retains the Kaboom repo and docs site. The rename was presented as a non-breaking drop-in replacement — [JSLegendDev substack](https://jslegenddev.substack.com/p/kaboomjs-is-now-kaplay)
- Release line on GitHub: 3001.0.19 on 2025-06-15 is the last 3001.x; the 4000 line has been in alpha since at least alpha.20 (2025-06-15) through 4000.0.0-alpha.27.1 (2026-05-12). No stable 4000 release is listed — [Kaplay GitHub releases](https://github.com/kaplayjs/kaplay/releases)
- 4000 alphas include repeated breaking changes: alpha.25 forbids `obj.pos.x += 1` (must assign a new vector or use `moveBy`); alpha.27 changed primitive UVs and removed `SpriteData.tex`; alpha.23 renamed `onShow/onHide`; alpha.22 changed `KAPLAYCtx` generics. alpha.26 moved the build to TypeScript; alpha.22 added typed scene names (`kaplayTypes()`, `strictScenes`); alpha.24 added `maxTimeStep`, `fixedUpdateMode`, `setFixedSpeed()` and broadphase options (Quadtree, sweep and prune); alpha.26 added `floodFill()` and `buildConnectivityMap()` — [Kaplay GitHub releases](https://github.com/kaplayjs/kaplay/releases)
- The releases page mentions `addLevel()` (ASCII-style level definitions) but no Tiled or LDtk import, and nothing about headless mode or testing — [Kaplay GitHub releases](https://github.com/kaplayjs/kaplay/releases)

**PixiJS (renderer only)**
- PixiJS v8 launched in 2024 with WebGPU integrated into the core — [PixiJS blog: v8 launches](https://pixijs.com/blog/pixi-v8-launches)
- v8.20.0 (summer 2026) is a point release focused on the WebGPU path: render bundles, depth-only render targets, WGSL shader overrides, Safari WGSL fallback. Breaking change: `isUniform` replaced by `accessMode` (reading the old property silently yields `undefined`) — [GameDev.net](https://gamedev.net/news/5221-pixijs-v8200-released/)
- A July 2026 article gave 8.19.0 as latest at that time — [issoh.co.jp](https://www.issoh.co.jp/tech/details/9180/)
- No "v9" announcement was found.

**melonJS**
- v19.6.0 (files dated 2026-05-22) focused on WebGL context-loss hardening; a v19.9.0 is listed as the latest download on the SourceForge mirror, date unconfirmed — [SourceForge mirror](https://sourceforge.net/projects/melonjs.mirror/files/19.6.0/)
- A June 19, 2026 overview article describes melonJS as a current browser-based 2D engine — [I Programmer](https://www.i-programmer.info/news/167-javascript/18949-melonjs-browser-based-2d-game-engine.html)

### Inferences
- For an agent working blind, Excalibur's own test setup (vitest unit tests plus Playwright "visual" tests, Dev Container) is the closest thing to an off-the-shelf headless workflow, and its first-party LDtk/Tiled plugins cover the map-import need. Its 0.x versioning and breaking releases (0.29, 0.32 physics defaults) mean pinning a version.
- Phaser 4 is the largest ecosystem but the v4 line is only ~6 months old with patch releases still landing; 4.x-specific tutorials and plugins (pathfinding, Tiled) are thinner than for v3. Phaser's docs site blocking fetches also made it the hardest engine to research programmatically here.
- Kaplay 3001 is stable but frozen; Kaplay 4000 is a moving target with API breaks every alpha. Poor fit for a long-lived codebase unless pinned to 3001.x.
- PixiJS plus a hand-written tilemap/camera layer is a viable "renderer only" path that pairs well with a pure simulation core (see Key Question 5); the OpenFrontIO project cited there does exactly this.

### Gaps
- Phaser 4 specifics (WebGL/WebGPU/Canvas renderer support, TypeScript definition quality, bundle size, v3-to-v4 migration list, Tiled loader changes, Phaser Editor status) could not be verified: phaser.io news page 404 and the GameDev.net article 403.
- No source found describing a headless/Node test mode for Phaser, Kaplay, PixiJS or melonJS.
- Bundle sizes for any of the engines were not found in the sources reached.
- Pathfinding plugin status for each engine (e.g. EasyStar.js compatibility with Phaser 4) was not researched.

## Key Question 2: Godot 4.x for a code-first, editor-less agent

### Takeaway
Godot's CLI is well documented for headless use (`--headless`, `--script`, `--export-release`, `--import`) and the web export default since 4.3 is single-threaded, which removes the SharedArrayBuffer/COOP/COEP header problem. The practical pain points are that CI test runners (GUT, gdUnit4) have reported headless input issues leading people to xvfb, and that import must be run explicitly before export.

### Cited Findings
- `--headless` is shorthand for `--display-driver headless --audio-driver Dummy`, "useful for servers and with --script"; required on machines without GPU access such as CI. The stable docs now target Godot 4.7 — [Godot command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Script execution without the editor: `godot -s sayhello.gd` where the script extends `SceneTree` or `MainLoop`; scripts can carry a `#!/usr/bin/env -S godot -s` shebang; `--check-only` parses for errors and quits — [Godot command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Export from CLI: `godot --headless --export-release "Linux/X11" /var/builds/project`; `--export-debug` same parameters; `--export-pack` for PCK/ZIP only; the `godot` binary must be an editor build with export templates installed; preset name must match `export_presets.cfg`; relative output paths resolve from the `project.godot` directory — [Godot command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- `--import` starts the editor, waits for resources to import, then quits (implies `--editor --quit`) — [Godot command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- `--test` runs engine unit tests only if compiled with `tests=yes` (not for game tests) — [Godot command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Web export: since Godot 4.3 single-threaded export is supported and is now the default; threaded builds depend on SharedArrayBuffer and require Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers; single-threaded is less performant but needs no special headers — [Godot 4.5 web export docs](https://docs.godotengine.org/ja/4.5/tutorials/export/exporting_for_web.html)
- A blank/black screen with "SharedArrayBuffer is not defined" means a threaded export served without the headers; fix is turning Thread Support off in the Web preset — [bugnet.io](https://bugnet.io/blog/fix-godot-export-html5-game-fails-load-shared-array-buffer)
- Godot 4.7 ("Director's Cut") page lists a 2D-only Scene Paint Mode for scattering nodes, `AtlasTexture` in `TextureRect`, `one_way_collision_direction` on `CollisionShape2D`, and a new GDExtensions section in Project Settings; it mentions no TileMap, web export, headless or .NET-web changes — [Godot 4.7 release page](https://godotengine.org/releases/4.7/)
- Release timeline: 4.7 blog post dated 18 June 2026; 4.7.1 on 14 July 2026; 4.7.2 maintenance release dated 18 August 2026; 4.8 dev 3 snapshot 7 August 2026 — [Godot blog index](https://godotengine.org/blog/index.html), [Godot forum maintenance releases](https://forum.godotengine.org/tag/maintenance-release/55)
- 4.6 release date: 27 Jan 2026 per a download mirror; the forum announcement thread is dated 4 February 2026 (sources differ) — [FileHorse](https://mac.filehorse.com/download-godot-engine/old-versions/), [Godot forum announcements](https://forum.godotengine.org/c/announcements/23?page=1)
- gdUnit4 provides command-line tooling for CI with JUnit XML and HTML reports; a marketplace GitHub Action takes Godot version and test paths as parameters — [gdUnit4 asset page](https://store.godotengine.org/asset/mikeschulze/gdunit4-unit-testing-framework/), [gdUnit4 test runner action](https://github.com/marketplace/actions/gdunit4-test-runner-action)
- A gdUnit4Net user reports the action works around headless input problems by running under xvfb, which made tests very slow on CI runners without a GPU, and that `.runsettings` did not pass the headless flag on Linux (one user's report) — [gdUnit4Net discussion #350](https://github.com/godot-gdunit-labs/gdUnit4Net/discussions/350)
- A GUT CI reference shows a GitHub Actions job downloading the Godot 4.6.1 Linux binary directly and running GUT headless with JUnit XML output — [tessl.io GUT CI reference](https://tessl.io/registry/testland/godot-gut-tests/1.4.17/files/references/ci.md)
- A GUT user reported a headless run succeeding locally but behaving differently in CI (GUT 9.0.1, Godot 4.0.2) — [GUT issue #491](https://github.com/bitwes/Gut/issues/491)
- Tiled can export directly to Godot 4 `.tscn` scenes — [Tiled export docs](https://doc.mapeditor.org/en/stable/manual/export/)

### Inferences
- A fully editor-less Godot loop is possible: write `.gd`, `.tscn` and `.tres` as text, run `godot --headless --import` then `godot --headless --export-release Web dist/index.html`, and run GUT/gdUnit4 headless. The agent never needs the GUI, but it also never gets the editor's validation of hand-written `.tscn` text, so a typo surfaces only at import/run time.
- The reported xvfb slowness and flag inconsistencies in gdUnit4/GUT headless runs suggest more CI friction than a Node/vitest loop in a TypeScript engine.
- Godot's single-threaded web export fits a static host without custom headers; the threaded export would need COOP/COEP headers from the host.

### Gaps
- Exact gdUnit4 and GUT CLI flags were not verified against their READMEs; the sources were community skill pages.
- No source found on hand-writing `.tscn/.tres` text files as a workflow (the "code-only Godot" write-ups suggested in the brief were not reached in budget).
- C#/.NET web export status in 4.6/4.7 was not found.
- Godot export template download/install in a container (needed for `--export-release`) was not researched.

## Key Question 3: Non-JS engines (Bevy, Ebitengine, LÖVE, Defold, Unity/Unreal, HaxeFlixel, Zig/C)

### Takeaway
Bevy (Rust) and Ebitengine (Go) both build to wasm from the CLI and both have explicit headless/automation stories, with Ebitengine 2.10 adding an "application virtualization mode for automated testing" in September 2026. LÖVE's web path is still an Emscripten port of 11.5 (LÖVE 12 unreleased), HaxeFlixel's HTML5 target is officially caveated on performance, and no usable sources on Defold HTML5 or Unity/Unreal headless were reached in budget.

### Cited Findings

**Bevy (Rust)**
- Bevy 0.18 was published 13 January 2026 (174-175 contributors). It added Cargo "feature collections" (`2d`, `3d`, `ui`) to compile only what an app needs, a `2d` profile that can swap out the default renderer, `PanCamera` for 2D (WASD panning and zoom), screenshot and screen-recording plugins, and removal of systems from schedules. The release post does not mention wasm, WebGPU/WebGL2, threads or headless mode; breaking changes are in a separate 0.17 to 0.18 migration guide — [Bevy 0.18 release post](https://bevy.org/news/bevy-0-18/)
- Bevy 0.17 shipped in late 2025 with a work-in-progress "headless" UI widget library (a UI concept, not a no-render runtime) — [GameFromScratch on Bevy 0.17](https://gamefromscratch.com/bevy-0-17-released/)
- A 2026 indie guide claims "web builds genuinely work"; the same guide dates 0.18 to March 2026, which conflicts with the official January 13, 2026 post — [Stray Spark blog](https://www.strayspark.studio/blog/bevy-rust-game-engine-2026-indie-guide); contradicted by [Bevy 0.18 release post](https://bevy.org/news/bevy-0-18/)
- A community template repo includes an integration test described as a small test with Bevy in headless mode — [taurr/bevy-template-rs](https://github.com/taurr/bevy-template-rs)
- bevy 0.17.2 exists on crates.io — [deps.rs](https://deps.rs/crate/bevy/0.17.2)

**Ebitengine (Go)**
- Ebitengine v2.10.0 released 2026-09-08, requires Go 1.25+. Headline features: pure-Go builds on desktop, an "application virtualization mode for automated testing", and experimental shader precompilation — [Ebitengine v2.10.0 blog](https://ebitengine.org/en/blog/v2.10.0.html)
- Browser builds use Go's built-in js/wasm target: a single `GOOS=js GOARCH=wasm go build` plus copying `wasm_exec.js` from the Go install; Ebiten creates the canvas and runs the loop around `Update/Draw/Layout` — [Wavedash Ebiten docs](https://docs.wavedash.com/engines/ebiten.md)

**LÖVE (Lua)**
- LÖVE 12.0 ("Bestest Friend") is listed as not released on the official wiki — [love2d.org wiki 12.0](https://www.love2d.org/wiki/12.0)
- The 2dengine love.js player supports LÖVE 11.3, 11.4 and 11.5, must be served from a web server (not file://), and needs specific cross-origin headers — [2dengine/love.js](https://github.com/2dengine/love.js)
- Wavedash describes love.js as an Emscripten build of LÖVE 11.5 — [Wavedash love2d docs](https://docs.wavedash.com/engines/love2d.md)
- A community fork aims to track newer LÖVE and Emscripten versions but no LÖVE 12 target was found — [jiaaro/love.js](https://github.com/jiaaro/love.js)

**HaxeFlixel**
- haxelib lists flixel 6.2.0 ("Sound refactor, FlxMatrixSprite and tilemap ray methods") as the newest version; 6.1.2 changelog is dated 6 December 2025 — [lib.haxe.org flixel versions](https://lib.haxe.org/p/flixel/versions/), [6.1.2 changelog](https://development-lib.haxe.org/p/flixel/6.1.2/changelog)
- Official targets page lists HTML5 via OpenFL with caveats: keep particle count low and do not expect much in terms of performance — [HaxeFlixel targets](https://haxeflixel.com/documentation/haxeflixel-targets/)

### Inferences
- Bevy's 0.x cadence (0.17 late 2025, 0.18 Jan 2026) with a migration guide per release means an agent would spend effort on upgrades; the ECS model, however, maps directly onto the "simulation core separate from renderer" pattern, and `MinimalPlugins`-style headless apps are a known community pattern (one template cited; official docs not reached).
- Ebitengine is the simplest wasm toolchain of the compiled options (one Go command, no bundler) and its new automated-testing virtualization mode is directly aimed at this use case, but it is a library not an engine: tilemaps, cameras, Tiled/LDtk loading and pathfinding are third-party or hand-written.
- LÖVE's web story depends on a third-party Emscripten build pinned to 11.5 that needs COOP/COEP-style headers; weak fit for a web-first target.
- Unity and Unreal were not researched (no budget); they are GUI-centric editors with licensing/installation that is awkward in a headless Linux container, and the brief already marks them as probably impractical. Treat as a gap, not a finding.

### Gaps
- Defold HTML5 export and code-first viability: no sources reached.
- Unity/Unreal headless: not researched.
- Zig/C (raylib, SDL to wasm): not researched.
- Bevy wasm specifics in 2026 (WebGL2 vs WebGPU default, atomics/threads on web, bundle size) were not found in the sources reached.

## Key Question 4: Map and asset tooling (Tiled, LDtk, programmatic maps, free/open art, AI art licensing)

### Takeaway
Both Tiled and LDtk have JSON formats an agent can write or read directly; Tiled has a documented CLI exporter (but needs an X server on Linux, so `xvfb-run`), and LDtk's "Super Simple Export" emits per-level PNG, CSV and small JSON that need almost no parser. Kenney (CC0, 30,000+ assets) is the safest art source; the LPC character generator is extensive but carries CC-BY-SA/GPL attribution and share-alike obligations; AI-generated art faces the US Copyright Office's January 2025 position that purely AI-generated output is not copyrightable.

### Cited Findings

**Tiled**
- Command-line export via `--export-map` and `--export-tileset`. On Linux, Tiled still needs an X server when exporting from the command line; the docs suggest `xvfb-run tiled --export-map ...` for headless environments — [Tiled export manual](https://doc.mapeditor.org/en/stable/manual/export/)
- Built-in export formats include JSON, Lua, CSV, Defold (defold, defoldcollection), GameMaker Studio 1.4 and `.yy`, Godot 4 (`.tscn`), tBIN/tIDE, image export, and custom formats via JavaScript, Python or C++ — [Tiled export manual](https://doc.mapeditor.org/en/stable/manual/export/)

**LDtk**
- The JSON structure documentation is at version 1.5.3 — [LDtk JSON docs](https://ldtk.io/json)
- Super Simple Export (introduced in 1.1.0) writes, per level, a full-level composite PNG, a `data.json` with level info and entities, one PNG per layer, and for IntGrid layers a CSV (e.g. `collisions.csv`) with the same data as plain text. It is enabled in Project settings and built on save — [LDtk game-dev docs](https://ldtk.io/docs/game-dev/), [LDtk 1.1.0 devlog](https://deepnight.itch.io/ldtk/devlog/364343/110-introducing-super-simple-export)
- LDtk can export to Tiled TMX, with documented limitations — [LDtk game-dev docs](https://ldtk.io/docs/game-dev/)
- Excalibur ships loaders for both formats (see Key Question 1) — [Excalibur LDtk plugin](https://excaliburjs.com/docs/ldtk-plugin), [Excalibur Tiled plugin](https://excaliburjs.com/docs/tiled-plugin)

**Free art**
- Kenney's assets are CC0 1.0 Universal: usable in any personal or commercial project, no permission needed, attribution optional — [Kenney license mirror](https://github.com/ETdoFresh/kenney.nl), [OpenGameArt emotes pack](https://opengameart.org/content/emotes-pack)
- Defold's Kenney listing describes the collection as over 30,000 game assets under CC0 1.0 — [defold.com/assets/kenney](https://defold.com/assets/kenney)
- Universal LPC Spritesheet Character Generator: builds character spritesheets from a shared library; artwork in `spritesheets` is under one or more of CC0, CC-BY-SA (credit, no DRM, derivatives under CC-BY-SA 4.0+), CC-BY, OGA-BY (allows DRM), GPL 3.0+; repo license GPL-3.0. Any generated sprite must credit authors except CC0 art; the generator exports a license sheet (CSV/text) and the repo ships `CREDITS.csv`; the README recommends saving the JSON export beside the PNG so credits can be recovered. Built with Vite; Node 22.19+; definitions in `sheet_definitions/` and `palette_definitions/`; CI on Node 24 with Argos visual tests; ~2,353 commits, ~1.9k stars; original author stepped back and others maintain. No documented CLI — [LPC generator GitHub](https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator)
- Tuxemon 16x16 tileset by Buch on OpenGameArt is CC-BY-SA 3.0 with a credit-and-link-back request; comments on the page allege some tiles were taken from another tileset without credit (unverified user comments) — [OpenGameArt Tuxemon tileset](https://opengameart.org/content/tuxemon-tileset)
- Tuxemon monster sprites by Sanglorian are CC-BY-SA 4.0 — [OpenGameArt Tuxemon set 1](https://opengameart.org/content/tuxemon-set-1-154-monsters-front-and-back-sprites-and-menu-animations)
- Tayoko's "Overworld RPG tileset" (top-down) is dual GPL 2.0/3.0 and CC-BY-SA 3.0 — [OpenGameArt node 29210](https://opengameart.org/node/29210)

**AI-generated art**
- Retro Diffusion states its model was trained on licensed pixel art from Astropulse and other artists with consent; pricing is pay-per-generation from about $0.01 per image via an MCP server (vendor claims) — [Retro Diffusion itch page](https://astropulse.itch.io/retrodiffusionai), [Retro Diffusion MCP listing](https://mcpservers.org/de/servers/retro-diffusion/retro-diffusion-mcp)
- An OpenGameArt commenter argues PixelLab's pose/rotation tools keep output tied to the input art's license, while its from-scratch generation is "more questionable" (one user's opinion) — [lpc.opengameart.org comment](https://lpc.opengameart.org/user/131989/comments)
- US Copyright Office, Part 2 report (29 January 2025): works created solely by AI without human intervention are not eligible for copyright; prompts alone are insufficient; protection can extend to outputs where a human contributed sufficient expressive elements (creative arrangement, modification, incorporated human-authored material); no legislative change recommended — [Sidley summary](https://datamatters.sidley.com/2025/02/07/u-s-copyright-office-issues-report-on-artificial-intelligence-and-copyrightability/)

### Inferences
- For an agent with no editor, writing Tiled JSON or LDtk JSON directly from code (or generating maps procedurally in the game's own format and skipping both editors) avoids the Xvfb dependency entirely. Tiled's CLI is only needed to convert formats; LDtk's Super Simple Export is only produced by the LDtk app on save, so the agent would read the LDtk JSON directly rather than rely on the simplified files.
- Kenney CC0 removes licensing bookkeeping; LPC assets give far more top-down RPG character variety but require a credits screen and share-alike on derived art. A mixed approach (CC0 tiles, LPC characters with generated `CREDITS.csv`) is common and workable.
- AI-generated art can fill gaps but, per the Copyright Office position, the game would not own copyright in purely generated sprites; vendor claims about licensed training data reduce but do not remove risk.

### Gaps
- Tiled version number for the "stable" manual was not stated on the page fetched; Tiled 1.11/1.12 release specifics were not found.
- No source found on a documented LDtk CLI or headless export; LDtk appears to be app-only.
- Programmatic Tiled map generation libraries (e.g. Node TMX/JSON writers) were not researched.
- itch.io asset pack licensing norms and OpenGameArt's overall license mix were not researched beyond the entries above.

## Key Question 5: Architecture pattern (pure simulation core, event-sourced state, save/replay)

### Takeaway
Several open projects implement exactly the recommended pattern: a tick-driven, dependency-free simulation with a seeded RNG and no wall-clock access, a renderer that observes but cannot mutate state, and a recorded input stream that replays deterministically. hayao.js and Clockwork Engine are TypeScript examples with headless renderers built in; OpenFrontIO runs its core in a Web Worker separate from PixiJS.

### Cited Findings
- hayao.js is "a headless-native simulation kernel for JavaScript/TypeScript with a Godot-style scene tree, pluggable renderers (SVG / Canvas / headless)"; rendering is a plugin that observes the simulation and cannot change results; randomness goes through a seeded RNG; a canonical state hash supports replay checks — [hayao.js.org](https://hayao.js.org/)
- OpenFrontIO runs its core simulation in a Web Worker separate from PixiJS rendering; the core has no external dependencies and avoids wall-clock time; any randomness must use seeded RNGs; determinism lets clients run their own copy of the logic, enabling replays — [OpenFrontIO architecture docs](https://www.mintlify.com/openfrontio/OpenFrontIO/development/architecture), [OpenFrontIO overview](https://www.mintlify.com/openfrontio/OpenFrontIO/technical/overview)
- Clockwork Engine (TypeScript) has frame-based updates, a seeded PRNG, a recording and playback system, and swappable rendering: PIXI.js for web and a headless platform for testing — [clockwork-engine](https://ithub.global.ssl.fastly.net/hiddentao/clockwork-engine)
- A Crimsonland rewrite (Python) runs a deterministic fixed-timestep update with drawing kept out of the simulation, which enables headless replay verification and ensures rendering bugs cannot affect simulation — [Crimson rewrite game loop docs](https://www.mintlify.com/banteg/crimson/rewrite/architecture/game-loop)
- Replay checksums should cover only gameplay-affecting state; renderer interpolation state can be excluded, simplifying replay verification — [feedbagel: not every byte gets a vote](https://feedbagel.com/post/not-every-byte-gets-a-vote)
- Integer fixed-point arithmetic is used to keep results identical across platforms (tri_grid_sim, Rust) — [docs.rs tri_grid_sim](https://docs.rs/tri_grid_sim)
- Kaplay 4000 alpha.24 added `fixedUpdateMode`, `maxTimeStep` and `setFixedSpeed()` to its loop — [Kaplay GitHub releases](https://github.com/kaplayjs/kaplay/releases)
- Excalibur 0.29 introduced `World.query`, systems receiving a `world`, and 0.32 added ECS system timing stats — [GitHub releases](https://github.com/excaliburjs/Excalibur/releases), [v0.32.0](https://github.com/excaliburjs/Excalibur/releases/tag/v0.32.0)

### Inferences
- The strongest fit for this project is: a pure TypeScript simulation package (tiles, buildings, routes, NPC schedules) tested in Node with tap/vitest and seeds, a thin renderer package (PixiJS, Excalibur or Phaser) that reads a snapshot each frame, and an event log of inputs/decisions that doubles as the save format and the replay. The cited projects show the pattern is proven in TypeScript and works with PixiJS specifically.
- If NPC schedules are driven by integer ticks (e.g. one tick per in-game minute) and all randomness is seeded, the simulation is naturally deterministic in V8 without fixed-point math; fixed-point only matters for cross-platform float divergence, which does not arise when the same JS runs everywhere.
- A worker-hosted core (OpenFrontIO) is also what a later server-side simulation would need: the same package can run in a Cloudflare Worker/Durable Object, since it has no DOM or wall-clock dependencies.

### Gaps
- No source reached on event-sourced save/replay formats specifically for Stardew-style games, or on compaction/snapshot strategies.
- No benchmarks found for simulation tick cost with many scheduled NPCs in JS.

## Key Question 6: Testing and visual verification without a screen

### Takeaway
Playwright screenshots of a WebGL canvas in a GPU-less Linux container work but need explicit SwiftShader flags (`--use-gl=angle --use-angle=swiftshader-webgl --enable-unsafe-swiftshader`), a check that a real WebGL context was created (silent 2D fallback otherwise), baselines captured in the same headless environment, and page-exposed test hooks for deterministic state. Excalibur itself uses Playwright 1.57 for its visual suite, and Ebitengine 2.10 added an automated-testing virtualization mode.

### Cited Findings
- Chromium documents two separate SwiftShader routes: as the OpenGL ES driver (`--use-gl=angle --use-angle=swiftshader`) and as an explicitly opted-in WebGL fallback (`--use-gl=angle --use-angle=swiftshader-webgl`), the latter requiring `--enable-unsafe-swiftshader`, which Chromium warns has lower security guarantees and is for trusted content; legacy SwiftShader GL libraries are being phased out in favor of SwANGLE (ANGLE + SwiftShader Vulkan) — [Chromium SwiftShader docs](https://chromium.googlesource.com/chromium/src.git/+/lkgr-ios-internal/docs/gpu/swiftshader.md)
- `--disable-gpu` breaks WebGL on Chrome 130+; a project switched to `--enable-unsafe-swiftshader`, said to match Playwright's behavior — [chrome-use CLI notes](https://gitea.oimcloud.myaddr.tools/oimwiodev/chrome-use/src/commit/486e1b341fbe1e1a6bbf9bfc885ec0e2d34eb45c/cli)
- On Linux without a display, WebGL can silently degrade to a 2D fallback: the screenshot succeeds and the output is wrong but plausible; a successful launch does not guarantee a WebGL context; check the renderer/GPU status before capturing — [microlink: WebGL without a GPU](https://microlink.io/blog/webgl-without-a-gpu)
- Alpine reports SwiftShader errors in recent headless Chromium that can break WebGL; pin the Chromium version — [Alpine aports issue 13272](https://gitlab.alpinelinux.org/alpine/aports/-/issues/13272)
- Headless and headed rendering differ (GPU availability, scaling, fonts); capture baselines in the same headless CI environment used for comparison; Playwright's `expect(page).toHaveScreenshot()` does baseline comparison; Linux CI can use Xvfb — [createit: headless Chrome WebGL with Playwright](https://www.createit.com/?p=3319)
- A Phaser-focused testing skill recommends exposing a `window.__TEST__` hook so Playwright can toggle modes deterministically, waiting on a ready signal, and using software-rendering flags or skipping visual tests on GPU-limited CI — [phaserjs-oakwoods playwright-testing skill](https://skillselion.com/skills/chongdashu/phaserjs-oakwoods/playwright-testing)
- Excalibur's own suite is split into `unit` and `visual` tests, running on vitest with Playwright 1.57.0 — [Excalibur v0.32.0](https://github.com/excaliburjs/Excalibur/releases/tag/v0.32.0)
- Ebitengine 2.10 added an "application virtualization mode for automated testing" — [Ebitengine v2.10.0 blog](https://ebitengine.org/en/blog/v2.10.0.html)
- Godot `--headless` is required on machines without GPU access; gdUnit4 reports fall back to xvfb for input, which is slow without a GPU — [Godot CLI docs](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html), [gdUnit4Net discussion #350](https://github.com/godot-gdunit-labs/gdUnit4Net/discussions/350)
- hayao.js and Clockwork Engine both ship a headless renderer target for tests — [hayao.js.org](https://hayao.js.org/), [clockwork-engine](https://ithub.global.ssl.fastly.net/hiddentao/clockwork-engine)

### Inferences
- For an agent that cannot see the screen, the highest-value verification is text: an ASCII/JSON dump of the map, NPC positions and schedules from the pure simulation, assertable in tap/vitest. Screenshots are a secondary check for the renderer, and should be gated on a WebGL-context assertion so a SwiftShader failure does not produce a convincing wrong image.
- Choosing a renderer that can fall back to Canvas2D (PixiJS and Phaser historically do; Excalibur is WebGL-based per its visual test setup, not verified here) reduces dependence on SwiftShader in CI.
- Pixel-diff snapshot tests should be few and coarse (whole-scene at fixed seed and tick), because headless rendering differences make fine-grained baselines brittle.

### Gaps
- headless-gl and node-canvas maintenance status in 2026 was not researched.
- No source on whether Playwright's bundled Chromium ships SwiftShader WebGL enabled by default in 2026 (one secondary source claims `--enable-unsafe-swiftshader` matches Playwright's defaults).
- No source found on running PixiJS, Phaser or Excalibur inside Node/jsdom for pure logic tests.

## Key Question 7: Deployment (static hosting, header requirements, later server-side simulation)

### Takeaway
Every JS/TS option and the Godot single-threaded export produce static files that need no special headers; Godot threaded web builds and love.js require COOP/COEP (cross-origin isolation) headers, which a static host must be able to set. A simulation core with no DOM or wall-clock dependencies (Key Question 5) is the prerequisite for running it later in a server runtime.

### Cited Findings
- Godot threaded web builds need Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers; single-threaded (default since 4.3) does not — [Godot 4.5 web export docs](https://docs.godotengine.org/ja/4.5/tutorials/export/exporting_for_web.html)
- love.js must be served from a web server and needs specific cross-origin headers — [2dengine/love.js](https://github.com/2dengine/love.js)
- Ebitengine web builds are a single `GOOS=js GOARCH=wasm` build plus `wasm_exec.js`, i.e. static files — [Wavedash Ebiten docs](https://docs.wavedash.com/engines/ebiten.md)
- OpenFrontIO's core runs in a Web Worker with no external dependencies and no wall-clock access, so the same logic runs on every client — [OpenFrontIO architecture docs](https://www.mintlify.com/openfrontio/OpenFrontIO/development/architecture)

### Inferences
- With the developer's existing Cloudflare Pages/Workers setup, a TypeScript engine bundled by esbuild deploys as static assets with no new infrastructure. Godot single-threaded and Ebitengine wasm also deploy as static assets. Only Godot threaded and love.js would require configuring response headers on the host.
- A later server-side simulation (authoritative town state, multiplayer) would need: the pure simulation package runnable outside the browser, a tick scheduler, and persistent state; the seeded, wall-clock-free design from Key Question 5 is what makes that move possible without a rewrite.

### Gaps
- Cloudflare Pages `_headers` / Workers header configuration for COOP/COEP was not verified in this research.
- Bundle size and cold-load figures for each engine on a static host were not found.

## Key Question 8: Maintenance and community status summary (2025-2026)

### Takeaway
Ranked by evidence of recent stable releases: PixiJS (8.20 summer 2026), Godot (4.7.2 Aug 2026, 4.8 dev), Ebitengine (2.10 Sep 2026), Phaser (4.2.1 Jul 2026), melonJS (19.6 May 2026), Bevy (0.18 Jan 2026), HaxeFlixel (6.2.0, late 2025/2026), Excalibur (0.32 Dec 2025, no 2026 release found), Kaplay (stable 3001.0.19 Jun 2025; 4000 alpha.27.1 May 2026), LÖVE (12.0 unreleased).

### Cited Findings
- Phaser: 4.0.0 10 Apr 2026, 4.1.0 30 Apr 2026, 4.2.1 9 Jul 2026 — [phaser.io/download/phaser4](https://phaser.io/download/phaser4), [phaser.io/download/stable](https://phaser.io/download/stable)
- Excalibur: 0.32.0 23 Dec 2025, 0.31.0 26 Sep 2025 — [GitHub releases](https://github.com/excaliburjs/Excalibur/releases), [v0.32.0](https://github.com/excaliburjs/Excalibur/releases/tag/v0.32.0)
- Kaplay: 3001.0.19 2025-06-15; 4000.0.0-alpha.27.1 2026-05-12 — [Kaplay GitHub releases](https://github.com/kaplayjs/kaplay/releases)
- PixiJS: 8.20.0 summer 2026; 8.19.0 as of July 2026 — [GameDev.net](https://gamedev.net/news/5221-pixijs-v8200-released/), [issoh.co.jp](https://www.issoh.co.jp/tech/details/9180/)
- melonJS: 19.6.0 files dated 2026-05-22; 19.9.0 listed — [SourceForge mirror](https://sourceforge.net/projects/melonjs.mirror/files/19.6.0/)
- Godot: 4.7 18 Jun 2026; 4.7.1 14 Jul 2026; 4.7.2 18 Aug 2026; 4.8 dev 3 7 Aug 2026 — [Godot blog](https://godotengine.org/blog/index.html), [Godot forum](https://forum.godotengine.org/tag/maintenance-release/55)
- Bevy: 0.18 13 Jan 2026; 0.17 late 2025 — [Bevy 0.18](https://bevy.org/news/bevy-0-18/), [GameFromScratch](https://gamefromscratch.com/bevy-0-17-released/)
- Ebitengine: 2.10.0 2026-09-08 — [Ebitengine blog](https://ebitengine.org/en/blog/v2.10.0.html)
- HaxeFlixel: 6.2.0 newest; 6.1.2 6 Dec 2025 — [haxelib](https://lib.haxe.org/p/flixel/versions/)
- LÖVE 12.0: not released — [love2d wiki](https://www.love2d.org/wiki/12.0)
- LPC generator: active CI (Node 24), ~2.3k commits, maintainers changed after original author stepped back — [LPC generator GitHub](https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator)
- LDtk JSON schema at 1.5.3 — [LDtk JSON docs](https://ldtk.io/json)

### Inferences
- Excalibur's lack of a 2026 release (as of October 2026, based on the GitHub releases page which listed v0.32.0 as newest) is worth watching before committing; it may simply be a long cycle, but it is the one TypeScript engine with the best headless/testing fit and the smallest team.
- Phaser 4's rapid 4.0 to 4.2.1 patch cadence within three months suggests the v4 renderer is still stabilizing.

### Gaps
- Exact dates and changelogs for melonJS 19.9.0 and HaxeFlixel 6.2.0 were not confirmed.
- GitHub star/contributor counts and issue-response times were not collected for any engine.
- Phaser's commercial/company status and Phaser 3 end-of-maintenance plans were not reached (release post blocked).
