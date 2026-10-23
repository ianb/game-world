# Editor-centric engines (Unity, Godot, Defold, GameMaker, Unreal) driven headless by a coding agent

Researched 2026-10-23. Scope: a 2D top-down pixel-art town game (Stardew / Ultima VI style), built entirely by an AI coding agent in a headless Linux container, with a deterministic discrete-event simulation that may live in a separate package. Web first, desktop/Steam second. JS renderers and Rust/Bevy stacks are out of scope.

Where a claim comes from a secondary site (news aggregator, Docker Hub mirror, forum post) it is marked. Where I could not find a source the item is in Gaps.

---

## Unity: licensing and cost in 2026

### Takeaway
Unity Personal is free for anyone under $200K revenue/funding, the splash screen is optional on Unity 6, and the Runtime Fee is dead. Pro ($2,310/seat/yr from 12 Jan 2026) is only required above $200K; there is no longer any Plus tier mentioned on the pricing page.

### Cited Findings
- The Runtime Fee was cancelled 12 Sept 2024 "effective immediately" and never applied to Unity 6 — [Unity pricing updates](https://unity.com/products/pricing-updates)
- Personal: free, for customers with up to $200,000 annual revenue and funding; the $200K threshold took effect with Unity 6 (17 Oct 2024); the splash screen is optional with Unity 6 — [Unity pricing updates](https://unity.com/products/pricing-updates)
- Pro: required above $200K revenue or funding in the last 12 months; 2026 price $2,310/seat/yr prepaid or $210/seat/mo; Pro and Enterprise prices rose 5% on 12 Jan 2026; Enterprise required above $25M, custom pricing — [Unity pricing updates](https://unity.com/products/pricing-updates)
- Using Unity 6 requires accepting the 10 Oct 2024 Editor Software Terms; Havok Physics is no longer bundled with Pro/Enterprise from Unity 6.3 — [Unity pricing updates](https://unity.com/products/pricing-updates)
- The pricing page does not mention a Plus plan at all — [Unity pricing updates](https://unity.com/products/pricing-updates)

### Inferences
- For a solo/agent project under $200K, cost is $0; the only licensing cost is the activation friction described in the next section.

### Gaps
- I did not find an explicit statement that Plus was retired (it is simply absent from the 2026 page).

---

## Unity: installing and running the Editor headless on Linux, and license activation without a GUI

### Takeaway
Two viable routes exist in 2026: (a) the new standalone `unity` CLI (beta, Linux Ubuntu 22.04+/RHEL 9, glibc 2.34+) which installs editors and modules without the Hub and has `build`, `test`, `run`, `license` commands; (b) GameCI `unityci/editor` Docker images (Unity 6000.3.27f1 `ubuntu-...-webgl-3` is 7.56 GB compressed, published 2026-10-21). Activation is the real obstacle: Personal licenses have no serial and the documented path is to obtain a `.ulf` file from Unity Hub on a GUI machine, then inject it (plus email/password) as secrets; Unity disabled manual `.alf` upload for Personal in Aug 2023.

### Cited Findings
- Unity CLI: a standalone binary, no Hub needed; "works in CI pipelines"; Linux support is RHEL 9 and Ubuntu 22.04+ with glibc 2.34+ (Debian 11, CentOS 8, Amazon Linux 2 unsupported); install with `curl -fsSL https://unity.com/install.sh | bash` to `~/.local/bin/unity`; also `.deb`/`.rpm` repos (beta in `unstable` suite), Homebrew cask `unity-cli`, AppImage — [Use the Unity CLI](https://docs.unity.com/en-us/unity-cli/use-unity-cli.md)
- `unity install lts`, `unity install 6000.3.7f1`, `unity install lts -m ios android webgl`, `unity install-modules -e 6000.3.7f1 -m android ios`, `unity install-modules -e <ver> -l` lists installable modules — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- Module ID for WebGL is `webgl` (Hub CLI examples `-m appletv webgl`); availability "depends on the Editor version and platform" — [Hub CLI reference](https://docs.unity.com/en-us/hub/hub-cli-reference)
- CLI commands: `build` ("Builds a project in batch mode with CI-friendly flags... `--list-targets`, `--list-profiles`, `--create-profile <target>`"), `run` ("Runs a project in batch mode or runs a registered Editor command in headless mode with `--command`"), `test` ("Runs a project's Edit Mode and Play Mode tests through the Editor test runner and write an NUnit report"), `recompile`, `watch`, `license` ("Lists, activates, and returns Unity licenses on this machine"), `mcp` — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- CLI exit codes: 0 ok, 2 usage, 3 auth, 6 primary operation failed (incl. "an unavailable license", compile error, crash), 7 service unreachable (retryable), 8 `unity test` had failing tests, 130/143 signals — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- Unattended sign-in: set `UNITY_SERVICE_ACCOUNT_ID` and `UNITY_SERVICE_ACCOUNT_SECRET`; `unity auth login` otherwise opens a browser flow; `UNITY_NON_INTERACTIVE`, `UNITY_NO_CLOUD`, `UNITY_FORMAT=json|ndjson|github` — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- The Unity CLI docs explicitly point Claude Code/Codex users at Unity's "Unity plugin" and the `unity-cli` skill — [Use the Unity CLI](https://docs.unity.com/en-us/unity-cli/use-unity-cli.md)
- Older Hub CLI: Hub AppImage `--headless install --version <v>` is experimental and requires the Hub itself — [Hub CLI](https://docs.unity3d.com/hub/manual/HubCLI)
- Editor CLI flags: `-batchmode` (no dialogs, exits 1 on exception), `-nographics` (skips GPU init; "doesn't allow GI baking"; logs off unless `-logFile`), `-quit`, `-executeMethod Class.Method` (static, in an Editor folder; exceptions or `EditorApplication.Exit(nonzero)` fail the run), `-projectPath`, `-logFile -` for console, `-buildTarget` accepts `webgl`, `linux64`, `win64`, etc. — [Editor command line arguments (Unity 6.6)](https://docs.unity3d.com/Manual/EditorCommandLineArguments.html)
- License flags: `-serial` (paid/serial or named-user), `-username`, `-password`, `-createManualActivationFile`, `-manualLicenseFile <file.ulf|.xml>`, `-returnlicense`; `-serial` requires `-batchmode` — [Editor command line arguments](https://docs.unity3d.com/Manual/EditorCommandLineArguments.html)
- Unity's manual: Personal licenses "have no serial number, so the command-line method doesn't apply and the Unity Hub is used to activate Personal licenses" — [Managing your Unity license](https://docs.unity3d.com/cn/2023.2/Manual/ManagingYourUnityLicense.html)
- GameCI Personal flow: install Hub on a desktop, log in, Preferences > Licenses > Add > "Get a free personal license", then copy `~/.local/share/unity3d/Unity/Unity_lic.ulf` (Linux path) into secret `UNITY_LICENSE` plus `UNITY_EMAIL`/`UNITY_PASSWORD`; "Licenses are not tied to a specific Unity version or platform"; Pro uses `UNITY_SERIAL` instead — [GameCI activation](https://game.ci/docs/github/activation)
- Unity stopped supporting manual `.alf` -> `.ulf` activation for Personal licenses in Aug 2023, breaking the old GameCI headless path — [Unity Discussions, Aug 2023](https://discussions.unity.com/t/unity-no-longer-supports-manual-activation-of-personal-licenses/926760); a DOM-editing hack to re-expose the Personal option on the activation page was published Aug/Sep 2023 and the author hoped it would not last — [dev.to workaround](https://dev.to/ankursheel/workaround-for-unity-personal-license-manual-activation-not-supported-49j2)
- GameCI Docker images: `unityci/editor` on Docker Hub, Ubuntu or Windows base (no macOS), new editor versions added automatically "within a few hours"; each OS used in a build consumes one concurrent seat — [GameCI docker images](https://game.ci/docs/docker/docker-images)
- Docker Hub API sizes for Unity 6000.3.27f1 (all last updated 2026-10-21, compressed): `ubuntu-6000.3.27f1-webgl-3` 7.56 GB, `ubuntu-6000.3.27f1-linux-il2cpp-3` 5.79 GB, `ubuntu-6000.3.27f1-android-3` 8.75 GB, `-ios-3` 5.8 GB; 1,881 tags match `6000.3` — [Docker Hub API query](https://hub.docker.com/v2/repositories/unityci/editor/tags?name=6000.3)
- Unity only supports WebGL builds with the Linux Editor on distributions with glibc 2.27+ (2021.2+) — [Unity issue tracker](https://issuetracker.unity3d.com/issues/give-error-when-building-webgl-on-centos7); Unity provides Linux Editor support only for Ubuntu — [Unity Linux manual](https://docs.unity.com/en-us/engine/6000.3/manual/platform-specific/linux.md)
- Community (secondary): a 2025 Docker fork lists `ubuntu-6000.3.2f1-webgl-runner` at 12.18 GB and other Ubuntu builds at 10-13.6 GB; a beta `ubuntu-6000.3.0b5-webgl-3.1.0` at 6.9 GB — [tyroonis/unityci-editor tags](https://hub.docker.com/r/tyroonis/unityci-editor/tags), [pitougames/unityci-editor](https://hub.docker.com/r/pitougames/unityci-editor)
- Headless-ish Unity MCP servers exist: "Union" can launch Unity with `-batchmode`/`-nographics` for CI and warns against `-logFile` because it reads stdout — [Union MCP server](https://glama.ai/mcp/servers/@nurture-tech/unity-mcp-server); `headless-unity-mcp` runs a build/test/screenshot loop in batchmode but is "macOS only, for now"; most other Unity MCP bridges (incl. Unity's first-party) are editor-embedded and need a live editor — [search summary of MCP servers](https://glama.ai/mcp/servers/oo90ns6thr)
- Unity's official AI/MCP open beta (announced 11 May 2026, Unity 6, clients Claude Code/Cursor/VS Code) connects agents to the running Editor; nothing found says it runs headless — [gamedev.net repost](https://gamedev.net/news/4376-unity-ai-open-beta-how-to-get-started-with-mcp/)

### Inferences
- A Personal license can be used in a container, but only after a one-time human step on a GUI machine to mint the `.ulf`; the agent cannot bootstrap activation on its own. A Pro serial (`-serial`) is the only fully headless activation path, at $2,310/yr.
- The existence of official `ubuntu-6000.3.x-webgl-3` GameCI tags, plus Unity's own glibc-2.27 statement, confirms WebGL building from the Linux Editor is a supported, packaged path.
- Container footprint is roughly 7.5 GB compressed (expect ~15-20 GB on disk) for the WebGL image alone; add ~5.8 GB for a Linux desktop (IL2CPP) image or use the CLI to add both modules to one install.

### Gaps
- Whether the new `unity license` command can activate a Personal license non-interactively on Linux: the reference page only lists the command; no option table was published. Unverified.
- No current (2025-2026) first-hand report of a Personal `.ulf` working long-term inside Unity 6 GameCI on Linux; the thread titled "Best practice for CI account activation" (last activity June 2026) was listed but its contents could not be fetched.
- Build times for a small 2D WebGL project on Linux in CI: no sourced figure.

---

## Unity: building Web and desktop from the CLI on Linux, running tests, output size, mobile web

### Takeaway
`Unity -batchmode -nographics -quit -projectPath . -buildTarget webgl -executeMethod Build.Web` (or `unity build` via the new CLI) is the standard pattern; Test Framework runs with `-runTests -testPlatform EditMode|PlayMode -testResults out.xml`. An empty Unity 6 URP Web build is ~3.3-5 MB; iOS Safari 15+ and Android Chrome 58+ are officially supported for Unity Web in Unity 6.

### Cited Findings
- `-buildTarget webgl|linux64|win64|...`, `-executeMethod`, `-batchmode`, `-nographics`, `-quit` semantics as above; `-quit` "causes the Editor to exit before in-progress tests finish when used with `-runTests`" — [Editor command line arguments](https://docs.unity3d.com/Manual/EditorCommandLineArguments.html)
- Test Framework 1.4 CLI: `-runTests`, `-testPlatform EditMode` (default) | `PlayMode` | any `BuildTarget` enum value (runs PlayMode tests on a built player), `-testResults <path>` (NUnit XML), `-testFilter`, `-testCategory`, `-assemblyNames`, `-testSettingsFile`, `-playerHeartbeatTimeout`, `-forgetProjectPath`, `-runSynchronously`, `-retry`, `-repeat`; "no common definition for exit codes"; example `Unity.exe -runTests -batchmode -projectPath P -testResults results.xml -testPlatform PS4` — [Test Framework command line](https://docs.unity3d.com/Packages/com.unity.test-framework@1.4/manual/reference-command-line.html)
- New CLI `unity test` writes an NUnit report and exits 8 on failures, 6 if the run did not finish; `unity build --list-targets/--list-profiles/--create-profile` for Unity 6 build profiles; `UNITY_BUILD_TIMEOUT` — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- Unity Pipeline package (Unity 6.0+): "Control the Unity Editor through a local HTTP API to automate builds, tests, and development workflows" via `unity pipeline install`, `unity command` (listing `eval`, `eval_file` etc.); install steps assume an open Editor — [Unity Pipeline package](https://docs.unity.com/en-us/unity-cli/unity-pipeline/unity-pipeline-package.md)
- Web build size (forum, Unity 6 URP): "current automated minimum build size is 3.32 MiB"; another user lists 1.8 MB legacy / 3.5 MB URP; enable release build and disk-size optimization — [Unity Discussions: smallest WebGL build](https://discussions.unity.com/t/smallest-possible-webgl-build-size/917019); third-party guide for 6.1+: ~12 MB unstripped, ~5 MB default-stripped, ~7 MB "minimal", recommends the Web Stripping Tool package — [FRVR Unity web optimization](https://docs.frvr.com/engines/unity/optimization/)
- Unity 6 Web browser compatibility: iOS Safari 15+ and Android Chrome 58+ listed as supported mobile browsers; Safari lacks WebGL 2 before 15 and lacks IndexedDB in iframes — [Unity Web browser compatibility (6.3/6.4)](https://docs.unity3d.com/Manual/webgl-browsercompatibility); mobile browser support announced for Unity 6 — [Unity blog](https://unity.com/blog/engine-platform/web-runtime-updates-enhance-browser-experience)
- Known Linux WebGL build bugs were fixed in 2021.1 (gzip -> bundled 7za) and 2021.3.14f1/2022.1.23f1 (Emscripten script) — [issue tracker](https://issuetracker.unity3d.com/issues/linux-webgl-build-always-fails-and-throws-a-filenotfoundexception)
- Old report (date not shown): "Unity Hub installed Linux version of Unity Editor will not build projects from command line" — [Unity on Linux release notes thread](https://discussions.unity.com/t/unity-on-linux-release-notes-and-known-issues/595006)

### Inferences
- A small pixel-art 2D game with compressed audio should land roughly 5-15 MB gzipped on Unity 6 Web; this is an estimate from the empty-build figures, not a measured number.
- PlayMode tests in `-batchmode -nographics` run in the Editor without a GPU; tests that render or read pixels will not work (see cross-cutting section).

### Gaps
- No sourced measured Web build size or load time for a specific small 2D pixel-art Unity 6 game.
- No sourced Linux-container WebGL build time.

---

## Unity: authoring scenes/prefabs/tilemaps/assets as text from code; 2D pixel-art features

### Takeaway
Unity's own docs say "Avoid generating or editing Unity YAML files directly"; the format is a restricted YAML dialect with class IDs, fileIDs and GUID `.meta` files, so the practical code-only route is Editor scripts run with `-executeMethod` that build scenes/prefabs/tilemaps through the Editor API and let Unity serialize them. The 2D feature set (Tilemap, Pixel Perfect Camera, URP 2D Renderer with Custom Axis transparency sort, 2D lights) is complete and well documented, but every feature assumes the Editor is in the loop for serialization.

### Cited Findings
- "Avoid generating or editing Unity YAML files directly. Consider using existing Unity API to update relevant YAML fields" (Unity 6.6 manual); older manuals said files "cannot be produced externally" — [UnityYAML](https://docs.unity3d.com/Manual/UnityYAML.html), [2019.4 UnityYAML](https://docs.unity3d.com/ru/2019.4/Manual/UnityYAML.html)
- UnityYAML does not support full YAML: comments, tags, multi-document input unsupported; a scene must start with `%YAML 1.1` and `%TAG !u! tag:unity3d.com,2011:`; import "is designed to be tolerant of omissions" and supplies defaults — [YAML scene example](https://docs.unity3d.com/Manual/YAMLSceneExample.html), [Unity YAML (7000.0 docs)](https://docs.unity.com/en-us/engine/7000.0/manual/working-with-scenes/text-scene-format/unity-yaml)
- Unity-maintained agent skills (`npx skills add Unity-Technologies/skills`) "control your open Unity Editor directly rather than editing scene and asset files by hand"; "Many skills need your Unity Editor to be open" — [Unity-Technologies/skills](https://github.com/Unity-Technologies/skills/blob/main/README.md)
- URP 2D Renderer Data: Transparency Sort Mode is a General property; "Custom Axis" with Y=1 renders objects higher on screen behind lower ones; Sort Axis field appears only in Custom Axis mode — [2D Renderer Data (Unity 6)](https://docs.unity3d.com/6/Documentation/Manual/urp/2DRendererData-overview.html); community confirms "set Transparency Sort Mode to Custom, Axis to 0,1,0 within your URP asset" — [Adventure Creator forum](https://adventurecreator.org/forum/discussion/comment/61460)
- Pixel Perfect Camera component: Asset Pixels Per Unit, Reference Resolution, Upscale Render Texture, Pixel Snapping (only when upscale RT is off; does not alter transforms), Crop Frame X/Y, Stretch Fill; sprites need identical PPU, Filter Mode Point, Compression None, pixel pivots; conflicts with Cinemachine unless the Cinemachine Pixel Perfect extension is used — [com.unity.2d.pixel-perfect 5.0 manual](https://docs.unity3d.com/Packages/com.unity.2d.pixel-perfect@5.0/manual/index.html) (URP ships its own copy under `urp/2d-pixelperfect`; that page redirected in a loop and could not be fetched)
- Tilemap component (Unity 6.6 manual): Animation Frame Rate, Color, Tile Anchor (default cell center), Orientation XY/XZ/..., Info; Tilemap Renderer settings (Chunk vs Individual mode) are on a separate reference page — [Tilemap reference](https://docs.unity3d.com/Manual/tilemaps/work-with-tilemaps/tilemap-reference.html)
- An Editor-only "Hierarchy Exporter" tool exports scenes to JSON/YAML/Markdown; nothing found goes the other way — [Unity Discussions](https://discussions.unity.com/t/hierarchy-exporter-export-unity-scene-hierarchies-to-tree-json-yaml-markdown/1718469)

### Inferences
- An agent can author Unity content code-only by (1) writing C# Editor scripts that create GameObjects, Tilemaps (`Tilemap.SetTile`), Tile assets and prefabs, and (2) running them with `-batchmode -nographics -executeMethod`. Hand-writing `.unity`/`.prefab`/`.asset` YAML is possible for simple cases (the importer tolerates omissions) but Unity recommends against it, and every referenced asset needs a stable GUID in a `.meta` file, which the agent must also generate.
- Y-sorting for a top-down town: URP 2D Renderer Custom Axis (0,1,0) plus Tilemap Renderer in Individual mode for per-tile sorting of tall objects; roof hiding needs custom scripting (no built-in feature found).
- I found no public report of an agent or CI-only pipeline authoring Unity scenes without an Editor; Unity's own tooling direction (skills, MCP, Pipeline package) assumes a running Editor, headless or not.

### Gaps
- Tilemap Renderer Chunk/Individual mode and Y-sort interaction page not fetched (redirect).
- No source on roof-hiding or occlusion patterns specific to Unity 2D.
- No source on URP 2D lights in batchmode (they are runtime features; nothing suggests a problem).

---

## Godot 4.x: current versions, headless CLI, export templates, sizes

### Takeaway
Godot 4.7 is current (June 2026, secondary source), with 4.6 (26 Jan 2026) and 4.5 (17 Sept 2025) before it. The editor binary itself is the CLI: `godot --headless --import` then `godot --headless --export-release "Web" build/index.html`, with export templates installed for the exact version. `--headless` is "required on machines without GPU access, such as CI".

### Cited Findings
- Godot 4.5 stable published 2025-09-17; 4.5.2 dated 19 Mar 2026 — [Godot 4.5 release](https://godotengine.org/releases/4.5/)
- Godot 4.6 released 26 Jan 2026 (Jolt default for 3D, LibGodot embedding, new editor theme) — [GamingOnLinux](https://www.gamingonlinux.com/2026/01/the-free-and-open-source-godot-engine-4-6-is-out-now-with-major-upgrades); download archive lists `linux.x86_64.zip`, `mono_linux_x86_64.zip`, `export_templates.tpz`, `mono_export_templates.tpz` dated 26 Jan 2026 (no sizes shown) — [Godot 4.6 download archive](https://godotengine.org/download/archive/4.6-stable/)
- Godot 4.7: release page describes a new export template downloader that fetches templates "for individual platforms or specific architectures, instead of all platforms at once"; 2D additions include a 2D Scene Paint Mode and one-way collision direction on `CollisionShape2D` — [Godot 4.7 release page](https://godotengine.org/releases/4.7/); secondary coverage gives 19 June 2026, "1,265 fixes", wasm64 web export option lifting the 4 GB limit, C# hot reload default — [cinevva (secondary)](https://app.cinevva.com/de/news/2026-06-19-godot-4-7-released)
- CLI (4.7 docs): `--headless` = `--display-driver headless --audio-driver Dummy`; `--path <dir>`; `--quit`, `--quit-after <n>`; `-s/--script <file>` (must extend `SceneTree` or `MainLoop`); `--check-only`; `--export-release <preset> <path>`, `--export-debug` (implies `--import`), `--export-pack` (`.pck`/`.zip`, implies `--import`), `--export-patch`; `--main-pack`; `--write-movie <file.avi|.png>` with `--fixed-fps`; "Unknown arguments are silently ignored"; "`--headless` is required on machines without GPU access, such as CI"; relative export paths resolve from the `project.godot` directory; output directory must exist — [Command line tutorial (4.7)](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Export must be run with an editor binary, not a template; export templates must be installed for the editor version (or a custom template defined in the preset); preset names come from `export_presets.cfg` — [Command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Templates are a `.tpz` (renamed ZIP) installed via the editor; export-only binaries are "smaller in size, more optimized" — [Exporting projects](https://docs.godotengine.org/en/3.6/tutorials/export/exporting_projects.html)
- Community CI example: `godot -v --export-release --headless "Web"` — [godot_template commit](https://git.d4m13n.dev/damien/godot_template/commit/3a3e4017af97e78ae1f3b5b7ef93cf6a1af02856)

### Inferences
- The whole toolchain is a single ~zip of the editor plus a `.tpz`; both are downloadable with `curl` and need no license or account. Install templates by unzipping the `.tpz` into `~/.local/share/godot/export_templates/<version>.stable/`.

### Gaps
- Exact byte sizes of the 4.6/4.7 Linux editor zip and `export_templates.tpz`: GitHub API/HTML were blocked from this session and the download page shows no sizes. From memory the editor zip is ~50-60 MB and the full `.tpz` is roughly 1 GB (hence the 4.7 per-platform downloader), but this is unsourced.

---

## Godot: code-only workflow (hand-written .tscn/.tres, runtime-built scenes), and reports

### Takeaway
`.tscn`/`.tres` are documented plain-text formats (`[gd_scene format=3 uid=...]`, `[ext_resource ...]`, `[sub_resource ...]`, `[node name= type= parent=]`) that an agent can generate; the editor tolerates comments and default-valued properties but strips them on save, and 4.6+ adds `unique_id` on nodes. Building the whole tree in GDScript (`Node.new()`, `add_child`) with no scene files is equally valid. Reports of agents editing `.tscn` directly exist (Cline in a forum post) but no systematic write-up.

### Cited Findings
- Format: `[gd_scene format=3 uid="uid://..."]`; `[ext_resource type="Texture2D" uid="uid://..." path="res://x.tres" id="2_eorut"]` referenced as `ExtResource("2_eorut")`; `[sub_resource type="CapsuleShape3D" id="CapsuleShape3D_fdxgg"]` referenced as `SubResource(...)` and must appear before use; `[node name="Ball" type="RigidBody3D" unique_id=...]`, children use `parent="."` or `parent="Arm/Hand"`; keywords `instance`, `instance_placeholder`, `index`, `owner`, `groups`, `node_paths`; property syntax like `transform = Transform3D(...)`, `Color(...)`, `NodePath("..")`, `PackedStringArray(...)`; `.tres` uses `[gd_resource type=... format=3 uid=...]` with a `[resource]` section; `unique_id` only in files saved with 4.6+; `load_steps` deprecated and ignored; `;` comments and default-valued properties are discarded when the editor saves — [TSCN file format (4.x)](https://docs.godotengine.org/en/stable/engine_details/file_formats/tscn.html)
- UIDs let the engine track moved files "even while the editor is closed" — [TSCN file format (ru mirror)](https://docs.godotengine.org/ru/4.x/engine_details/file_formats/tscn.html); a community asset claims stale `uid=` tokens are stripped and regenerated on next import and that hard-coded paths break with "Scene file appears to be invalid/corrupt" after moves — [RepairScene asset (community claim)](https://godotengine.org/asset-library/asset/edit/21196)
- Runtime scene construction: `PackedScene.instantiate()` + `add_child()`; nodes must be in the tree before `get_node()` (use `_ready()`); freeing the top node frees the branch — [Nodes and scene instances](https://docs.godotengine.org/en/4.4/tutorials/scripting/nodes_and_scene_instances.html)
- Forum report: "unlike with Unity tools, Cline can edit the .tscn file and set up the entire scene"; another user found AI-generated GDScript unreliable — [Godot forum, AI mini-games](https://forum.godotengine.org/t/anyone-else-using-ai-to-create-mini-games-one-shot-prompt/99856)
- Tooling: `gdscene` (PyPI, WIP) generates scene files programmatically; "Godot Assistant" editor plugin generates `.tscn` from text and "sanitizes the output to fix common AI formatting mistakes"; several Godot MCP servers create scenes via the running editor — [gdscene](https://libraries.io/pypi/gdscene), [Godot AI Assistant](https://store.godotengine.org/asset/champ-gaming/godot-ai-assistant/), [godot-mcp-server create_scene](https://glama.ai/mcp/servers/ricky-yosh/godot-mcp-server/tools/godot_create_scene)
- Headless import: `--export-debug`/`--export-pack` imply `--import`; `--headless` with `--script` is the documented way to run tooling scripts — [Command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)

### Inferences
- Recommended agent pattern: write `project.godot`, `export_presets.cfg`, `.tscn`/`.tres` and `.gd` as text; run `godot --headless --import` so `.godot/` (imported textures, UIDs) is generated; validate with `godot --headless --check-only -s file.gd` and `godot --headless --quit-after 2 scene.tscn`. Omit `uid=` on hand-written ext_resources and let the importer resolve by `path`, which avoids the UID mismatch class of errors.
- A tilemap-heavy town can be generated entirely in code: `TileSet.new()` with `TileSetAtlasSource`, `TileMapLayer.set_cell()` / `set_cells_terrain_connect()`; this sidesteps the verbose `.tres` encoding of TileSets.

### Gaps
- No long-form blog or postmortem of a fully editor-less Godot project (only forum anecdotes and tooling READMEs).
- `TileMapLayer.set_cell` / `TileSet` code APIs were not fetched (class reference); the tilemap tutorial page defers to it.

---

## Godot: running tests headless (GUT, gdUnit4)

### Takeaway
GUT runs headless with `godot --headless -s addons/gut/gut_cmdln.gd -gdir=res://test -gexit` and exits 1 on failure with JUnit XML output. gdUnit4 (v6.2.x for Godot 4.5-4.7.1, v7 for 4.7/4.8) ships a CLI runner and a GitHub Action; its .NET flavour has a documented Linux headless problem that forced an xvfb workaround "too slow" on GPU-less CI.

### Cited Findings
- GUT CLI: `godot -d -s --path "$PWD" addons/gut/gut_cmdln.gd`; options `-gdir`, `-ginclude_subdirs`, `-gtest`, `-gprefix` (default `test_`), `-gsuffix`, `-gconfig` (`res://.gutconfig.json` by default), `-glog=1`, `-gexit`, `-gselect`, `-gh` help, `-gpo` prints effective options, `-gjunit_xml_file`, `-gjunit_xml_timestamp`; exit 0 if all pass, 1 if any fail; CLI overrides gutconfig — [GUT Command Line](https://gut.readthedocs.io/en/latest/Command-Line.html)
- gdUnit4 version matrix: master v7.0.0 for Godot 4.7/4.7.1/4.8.dev7; v6.2.x for 4.5-4.7.1; v6.1.x for 4.5-4.6.3; v5.x for 4.3-4.4.1; features "Run tests outside the Godot editor for CI/CD pipelines", HTML and JUnit XML reports, `gdunit4-action` on the Marketplace — [gdUnit4 README](https://raw.githubusercontent.com/MikeSchulze/gdUnit4/master/README.md)
- gdunit4-action inputs: `godot-version` (required), `godot-status` (stable/rc/dev), `godot-net`, `paths` (`res://tests`), `arguments` (e.g. `--verbose --fail-fast`), `timeout` (default 10 min), `report-name`, `retries`, `warnings-as-errors`, `dotnet-version` (net8.0 default); caches Godot binaries and project cache — [gdUnit4-action README](https://raw.githubusercontent.com/MikeSchulze/gdUnit4-action/master/README.md)
- gdUnit4Net (C#): `.runsettings` `Parameters` takes extra Godot args such as `"--verbose --headless"`; a user reports headless via runsettings "does not work in Linux", that headless "has issues... due to input handling", and that the action's xvfb workaround "was too slow on their CI runners" — [gdUnit4Net discussion #350](https://github.com/godot-gdunit-labs/gdUnit4Net/discussions/350), [gdUnit4.test.adapter on NuGet](https://feed.nuget.org/packages/gdUnit4.test.adapter/)

### Inferences
- For a GDScript project, GUT is the lower-friction headless option; gdUnit4's richer assertions/mocking come with more moving parts. For a sim that lives in a separate package (TypeScript/Rust), engine tests should be limited to integration/smoke tests and the sim's own test runner does the heavy lifting.

### Gaps
- The gdUnit4 CLI docs page (`runtest.sh`, `--ignoreHeadlessMode`, `-a`/`-i`/`-c` flags) returned 404 at the URLs tried, so exact GDScript-runner flags are unverified here; use `runtest.sh --help` in the addon.
- No benchmark of GUT/gdUnit4 headless run time on CI.

---

## Godot: 2D pixel-art and tilemap features (TileMapLayer, terrains, Y-sort, pixel snapping, integer scaling)

### Takeaway
Godot's documented pixel-art setup is: base window size e.g. 640x360, `display/window/stretch/mode = viewport`, `stretch/aspect = keep|expand`, `stretch/scale_mode = integer`, nearest texture filter, and `rendering/2d/snap/snap_2d_transforms_to_pixel = true` (the vertex-snap option is deprecated). TileMapLayer nodes (one per layer) support terrains (autotile), per-tile Y-sort origin, and physics/navigation; Y-sort is the CanvasItem `y_sort_enabled` flag.

### Cited Findings
- Stretch modes: `disabled`, `canvas_items` (renders at target res; "2D sprite pixels no longer map 1:1 to screen pixels"), `viewport` (renders at base size then scales; "gives the low-resolution look pixel art needs"); aspect `keep`/`expand`/`keep_width`/`keep_height`/`ignore`; scale mode `integer` rounds the scale factor down and adds black bars (640x360 base scales cleanly to 720p/1080p/1440p/4K); use Exclusive Fullscreen because Fullscreen's 1-px reduction can drop an integer step; use `canvas_items` only if sub-pixel sprite movement is needed — [Multiple resolutions](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html)
- `rendering/2d/snap/snap_2d_transforms_to_pixel` supersedes 3.x GPU Pixel Snap; "Snap 2D Vertices to Pixel" was an early 4.0 experiment slated for deprecation — [Godot forum](https://forum.godotengine.org/t/difference-between-snap-2d-vertices-to-pixel-and-snap-2d-transforms-to-pixel/78642); enabling it fixed sprite stretching between pixels in a viewport-stretch project — [Godot forum](https://forum.godotengine.org/t/pixel-stretching-issue/127623)
- 4.4 regression report: `canvas_items` + integer scale behaved differently than 4.3 — [Godot forum](https://forum.godotengine.org/t/canvas-items-scaling-mode-broken-in-godot-4-4-works-in-4-3/105205)
- TileMapLayer: each layer is its own node (reorder by dragging); Y Sort Origin is a per-tile pixel offset that only applies when CanvasItem Y Sort Enabled is true; X Draw Order Reversed also needs Y-sort; "Rendering quadrant size doesn't apply to Y-sorted layers"; terrains need a terrain set, Connect vs Path mode; TileSet can be saved as an external resource and shares patterns; physics (Collision Enabled, Use Kinematic Bodies) and navigation flags; built-in TileMap navigation has performance limits, bake a NavigationRegion2D instead — [Using TileMaps](https://docs.godotengine.org/en/stable/tutorials/2d/using_tilemaps.html)
- Godot 4.5 added "chunk tilemap physics" — [Godot 4.5 release](https://godotengine.org/releases/4.5/)

### Inferences
- Roof hiding for a top-down town is not a built-in feature; the standard approach is a separate "roof" TileMapLayer whose `modulate` alpha is tweened when the player enters a building's Area2D. Y-sorting between characters and tall tiles works by enabling `y_sort_enabled` on both the layer and the parent node holding characters.

### Gaps
- The official page listing `snap_2d_transforms_to_pixel` (ProjectSettings class reference) was not fetched; property name is from forum posts quoting the docs.
- No source on how `--headless` interacts with `snap` settings (irrelevant at runtime; no rendering).

---

## Godot: web export caveats, C# vs GDScript on web, GDExtension and godot-rust for a Rust sim core

### Takeaway
Godot 4 web export runs only on the Compatibility (WebGL 2) renderer; the single-threaded export (default since 4.3) needs no COOP/COEP headers and works on macOS/iOS; the threaded export needs `SharedArrayBuffer` and cross-origin isolation. C# projects still cannot export to web as of 4.7 (the PR #106125 prototype from May 2025 has not landed; an unofficial build and a NuGet package exist). GDExtensions can run on web but must be compiled for wasm and need the "Extensions Support" flag; godot-rust wasm support is explicitly experimental.

### Cited Findings
- Web export (4.7 docs): requires WebAssembly + WebGL 2.0; only the Compatibility renderer; WebGPU "not yet supported"; single-threaded is default and preferred since 4.3 ("needs no special server headers", works on itch.io/Poki/CrazyGames, "works well on macOS and iOS"); "Thread Support" needs a secure context, `SharedArrayBuffer`, `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`, and rules out ads/third-party embeds; Safari has WebGL 2 issues so Chromium/Firefox recommended; "Godot 4 C# projects cannot currently be exported to the web"; GDExtension needs "Extensions Support" + wasm-compiled extension + cross-origin isolation headers; `.wasm` gzips to ~1/4; serve `.wasm` as `application/wasm`; audio defaults to Web Audio "Sample" mode (no effects) with "Stream" available at higher latency; PWA option; background tabs pause `_process()`; `user://` needs IndexedDB — [Exporting for the Web (4.7)](https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_web.html)
- A user on 4.7 stable mono still sees "Exporting to the web is currently not supported in Godot 4 when using C#/.NET" — [Godot forum, Can't export to web](https://forum.godotengine.org/t/cant-export-to-web/141491)
- Timeline: GodotCon Boston May 2025 prototype (PR godotengine/godot #106125 by raulsntos, draft); Dec 2025 "not a 4.6 feature... C# GDExtension first, expect 4.7 or more likely 4.8"; Mar 2026 blog quote "cannot commit to a specific timeline"; unofficial `ComplexRobot/godot-dotnet-web-export` builds exist; last reply 17 Apr 2026 still blocked — [Godot forum thread](https://forum.godotengine.org/t/is-there-an-update-on-exporting-c-projects-to-web/128821)
- Unofficial NuGet `2dog.browser-wasm.release` 4.7.1.17 (July 2026) statically links Godot into a .NET browser-wasm publish, net10.0+, small download count — [NuGet](https://www.nuget.org/packages/2dog.browser-wasm.release)
- godot-rust (gdext): wasm requires the `experimental-wasm` feature; "work-in-progress... not yet well tested"; threads on by default require `-C link-args=-pthread` for `wasm32-unknown-unknown` and must match Godot's threading setting, or use `experimental-wasm-nothreads`; an older note said it might not work in Firefox — [docs.rs godot crate](https://docs.rs/godot)
- Community (4.5 era): official web export templates lack GDExtension support enabled, so custom templates may be needed — [Godot forum](https://forum.godotengine.org/t/what-is-the-status-of-web-export-if-my-game-uses-c-from-gdextension/125375)
- C# docs: Android/iOS C# experimental since 4.2; hot reload loses non-exported state; assemblies must be rebuilt to see new exports; .NET 8 runtime in 4.5 docs — [C# basics (4.5)](https://docs.godotengine.org/en/4.5/tutorials/scripting/c_sharp/c_sharp_basics.html)

### Inferences
- For web-first, GDScript is the only safe scripting choice in Godot in 2026. A Rust sim core via godot-rust on web is possible but stacks two experimental things (gdext wasm + GDExtension-on-web with custom templates); a TypeScript sim is better reached via `JavaScriptBridge` in the browser, with a different transport (sockets/stdio) on desktop.
- The single-threaded default means a Godot web build is friendlier to itch.io and ad-supported portals than a Unity Web build only if Unity also avoids threads; both are WebGL 2 / wasm.

### Gaps
- No measured Godot web export size for a small 2D project (docs say only that `.wasm` gzips to ~25%).
- `JavaScriptBridge` docs were not fetched; its existence and use for JS interop is from prior knowledge.

---

## Defold

### Takeaway
Defold is free (Defold License 1.0, Apache-2.0-derived, royalty-free, source available, only restriction is reselling it as an engine), Lua-scripted, and the most CLI-native of the four: `java -jar bob.jar --archive --platform wasm-web resolve build bundle` (OpenJDK 25) builds HTML5 without the editor, `--variant headless` builds a GPU-less engine for tests, and all project files are text protobuf. Empty HTML5 build is 1.06 MB (1.12.4, May 2026). Current version is 1.13.x (June 2026). Community is small but shipping on Steam/Switch/Poki.

### Cited Findings
- License: "completely free to use", no subscription/royalties; Defold License v1.0 (May 2020) derived from Apache 2.0; Section 4(a) bars commercializing it "as a Game Engine Product"; owned by the Defold Foundation; attribution notice required; no OSI claim made — [Defold license](https://defold.com/license/)
- bob.jar: download from GitHub releases; "OpenJDK 25 is required"; commands `resolve`, `clean`, `distclean`, `build`, `bundle` chainable; `-a/--archive`, `-p/--platform`, `-bo/--bundle-output`, `--variant debug|release|headless`, `--settings <file>` (repeatable, applied left to right), `-tc/--texture-compression` (switch), `--build-report-html/json`, `--build-server` (default `https://build.defold.com` for native extensions), `-e/--email -u/--auth`, `--max-cpu-threads`; platforms `x86_64-linux`, `arm64-linux`, `x86_64-win32`, `x86_64-macos`, `arm64-macos`, `arm64-ios`, `armv7-android`, `wasm-web` (default web) and `wasm_pthread-web`; example `java -jar bob.jar --archive --platform x86_64-macos resolve distclean build bundle` — [Bob manual](https://defold.com/manuals/bob/)
- HTML5: `wasm-web` (non-threaded) and `wasm_pthread-web` (needs SharedArrayBuffer + COOP/COEP); bundle can include both with runtime fallback; heap size default 256 MB; scale modes Fit/Downscale Fit/Stretch/No Scale; Mustache custom HTML template; `graphics.webgl_version_hint` defaults to WebGL 2; hot reload does not work in HTML5; must be served over HTTP with `application/wasm` — [HTML5 manual](https://defold.com/manuals/html5/)
- 1.12.4 (4 May 2026): "the empty project is now 1.06 MB" for HTML5 — [Defold 1.12.4](https://defold.com/2026/05/04/Defold-1-12-4/); 1.12.1 posted 4 Feb 2026 — [Defold 1.12.1](https://defold.com/2026/02/04/Defold-1-12-1/); 1.13.0 stable 29 June 2026 (Wikipedia), 1.13.1 since; 1.11.0 was 8 Sept 2025, 1.11.2 3 Nov 2025 — [Wikipedia Defold](https://en.wikipedia.org/wiki/Defold), [defold-docker tags](https://gitlab.com/mattpwest/defold-docker/-/tags)
- Automated testing manual: use `--settings test/test.settings` to point `[bootstrap] main_collection` at a test collection in CI without editing the repo; build a headless test bundle with `java -jar bob.jar --root . --settings test/test.settings --platform x86_64-linux --variant headless --archive --bundle-output build/test-bundle resolve build bundle`; run it under a process controller with timeout and parse `TEST {json}` lines; browser tests via Playwright/Puppeteer etc. with a JS bridge (`html5.run()`); visual checks via runtime screenshots and image diff; DefTest (Telescope-based) for Lua unit tests — [Automated testing manual](https://defold.com/manuals/automated-testing/); earlier forum guidance: run unit tests with `dmengine_headless` from d.defold.com for Jenkins/TeamCity — [Defold forum](https://forum.defold.com/t/testing-with-dmengine-headless/1772)
- File formats: source files are text protobuf; a `.go` is written as `embedded_components { id: "model" type: "model" data: "mesh: \"...\"\n..." }` with the component's own protobuf text escaped inside `data`; `.collection` is a flat list where objects reference children by unique ids; editor scripting (1.10.4, July 2025) adds `editor.create_resources()`, `editor.transact()`, `editor.tx.add` — [Editor scripting update](https://www.defold.com/2025/07/11/editor-scripting-update); protobuf text explained by the team, no published spec — [Defold forum](https://forum.defold.com/t/whats-the-text-format-used-to-serialize-collections/72707/5); npm `defold-parser` encodes/decodes `.go`/`.collection` — [defold-parser](https://www.npmjs.com/package/defold-parser); per-example source files published for LLM use — [defold.com/llms/examples](https://defold.com/llms/examples/)
- Tilemap component: layers, tile source, brush flip/rotate, `tilemap.set_tile("/level#map", "ground", x, y, 4)` / `tilemap.get_tile()`, material/blend mode/tint; Tiled and Tilesetter export to Defold — [Tilemap manual](https://defold.com/manuals/tilemap/)
- Pixel art: set Default Texture Min/Max Filter to nearest in `game.project` (default linear); use a render script with `use_fixed_projection` and an integer zoom; disable Subpixels so sprites snap to whole pixels; GUI renders separately at high res — [Defold forum: pixel art blurry](https://forum.defold.com/t/pixel-art-blurry/64332), [Defold forum: source pixel art size](https://forum.defold.com/t/source-pixel-art-size-scaling-etc/68185), [defold/doc PR 571](https://github.com/defold/doc/pull/571.patch)
- Shipped games (showcase): Sector 781 (Switch/Steam/Poki, 2020), Tiny Racing (Steam/Switch, June 2025), Extinction Zone (Steam, 2026), Sprint City (Steam, Mar 2026), Kaiju Control Force (Steam/web, Nov 2024), Void Scrappers (Steam/Switch, 2022), Fates of Ort (Switch/desktop, 2020), Warnament (Steam/HTML5, Aug 2025), Craftomation 101 (Steam/Poki, Feb 2024), SuperWEIRD (2026), Hill Climb Racing Lite on Poki (2025) — [Defold showcase](https://defold.com/showcase/)

### Inferences
- Defold is the only engine here where "no editor ever" is the officially documented CI story (bob + headless variant + settings overlays). The cost is the `.go`/`.collection` double-escaped protobuf text, which is tedious but mechanical for an agent, and the lack of a formal spec.
- Native extensions build on Defold's cloud build server by default (`build.defold.com`), so a Rust/C++ sim core as a native extension needs network access or a self-hosted Extender.
- The ~1 MB empty web build is roughly 3-5x smaller than Unity's and likely smaller than Godot's.

### Gaps
- Community size numbers (forum users, Discord) not found.
- No source on Y-sorting in Defold tilemaps/sprites (Defold uses explicit z per component; the Y-sort pattern would be scripted). Not verified.
- Whether `--variant headless` dmengine renders anything (it does not, by name) is inferred, not quoted.

---

## GameMaker (2024/2025/LTS 2026)

### Takeaway
GameMaker is free for non-commercial use on non-console platforms, $99.99 one-time Professional for commercial release (incl. HTML5/GX.games and "Igor/CI builds"), Enterprise subscription for consoles. Igor is a real CLI (`Igor.exe /project=... /rp=... /lf=... -- <Platform> <Command>`, `dotnet Igor.dll` on macOS) with HTML5 and GX.games targets and a `Tests RunTests` mode, but it requires a runtime installed from the GameMaker feed, a `licence.plist` from an IDE login or access key, and the documentation gives no Linux host paths; the Linux IDE is a 2021 Ubuntu beta with no newer news. The project format is JSON-ish (trailing commas, big ints) with third-party parsers.

### Cited Findings
- Pricing (page copyright 2026): Free "for non-commercial use" with GX Games, desktop, web (HTML5) and mobile export; Professional one-time $99.99 adds commercial licence and "Igor/CI builds"; Enterprise subscription needed for consoles and runtime source; the upcoming GMRT runtime "will be free for non-commercial use", Professional for commercial — [GameMaker get page](https://gamemaker.io/en/get); Nov 2023 change reported widely: free non-commercial, $99.99 commercial, Enterprise $79.99/mo ($799.99/yr) for consoles — [gamefromscratch](https://gamefromscratch.com/gamemaker-announces-major-pricing-changes/), [TechRadar](https://www.techradar.com/gaming/consoles-pc/gamemaker-engine-transforms-its-business-model-and-introduces-a-one-time-fee)
- Igor: "Building executables via command line requires you to have logged in to the IDE" or a `licence.plist` from `runtime FetchLicense -ak <key> -of <file>`; default Igor path `C:\ProgramData\GameMakerStudio2\Cache\runtimes\runtime-[version]\bin\igor\<OS>\<arch>\` (Windows) or `/Users/Shared/GameMakerStudio2/Cache/runtimes/...` (macOS); options `/lf`, `/uf`, `/rp`, `/project`, `/cache`, `/temp`, `/of`, `/tf`, `/config`, `/runtime=YYC|VM`, `/j`, `/df`, `/device`, `/launch`, `/packagetype`, `/v`, `/jsonErrors`; form `Igor.exe [options] -- <Platform> <Command>`; platforms incl. GX.games (Compile/Clean/Run/Package, package types `OperaGXPackage_Zip|Gamestrip|Wallpaper`), HTML5 (Compile/Clean/folder build with `/tf`), Linux (Compile/Clean/Package, "use `/device` when building from a PC"); `Runtime Install` with `/modules` such as `html5`; tests: `Igor.exe Tests RunTests <dir|file>` with `/timeout` (120 s), `/waittime` (30 s), `/target PLATFORM|DEVICE`, `tests.json`; a test "passes if the runner is still running after the wait time" — [Building via Command Line (manual mirror on GitHub)](https://raw.githubusercontent.com/YoYoGames/GameMaker-Manual/develop/Manual/contents/Settings/Building_via_Command_Line.htm) (canonical page [manual.gamemaker.io](https://manual.gamemaker.io/monthly/en/Settings/Building_via_Command_Line.htm) was Cloudflare-blocked from this session)
- Project format: `.yyp` lists resources, each asset has a `.yy`; "JSON-ish, but don't follow the JSON spec" (trailing commas, very large integers); standard JSON libraries may fail or reformat; `*.resource_order` alongside — [GameMaker Project Format](https://manual.gamemaker.io/lts/en/Additional_Information/Project_Format.htm), [yal.cc formats](https://yal.cc/gamemaker-formats/); `@bscotch/yy` reads/validates/writes `.yy`/`.yyp` for recent versions — [@bscotch/yy](https://www.npmjs.com/package/@bscotch/yy)
- LTS 2026.0 (21 May 2026): no sign-in required to use the IDE (accounts still needed to export); GMRT Beta via Package Manager, "certain platforms are still being worked on"; GX.games target can export the WASM build as a ZIP; Switch 2 support; GMS2 runtime frozen to SDK/critical fixes until at least Q1 2028; no Linux IDE mention — [GameMaker LTS 2026](https://gamemaker.io/blog/lts-2026-release); GMRT on desktop "today" with source-available access, 30 Apr 2026 — [Opera press](https://press.opera.com/?p=3924); mobile/console GMRT later in 2026 — [GameMaker Spring 2026 update](https://gamemaker.io/blog/update-spring-2026)
- HTML5 runtime open-sourced 14 Nov 2022 at YoYoGames/GameMaker-HTML5 — [GameMaker blog](https://gamemaker.io/en/blog/html5-open-source); GX.games remains an export target; its paid-game purchasing "is being phased out as of September 2026" — [Wikipedia GameMaker](https://en.wikipedia.org/wiki/GameMaker); GX.games export yields "a WebAssembly runner plus a flat set of static files" — [Wavedash docs](https://docs.wavedash.com/engines/gamemaker)
- Linux IDE: Ubuntu editor beta shipped in GMS 2.3.4 (Aug/Sept 2021) with a non-working debugger and Ubuntu-only builds; no 2025-2026 news found — [GamingOnLinux 2021](https://gamingonlinux.com/2021/09/gamemaker-studio-2-update-released-to-bring-forth-the-ubuntu-linux-editor-beta); itch.io users describe Linux builds as hard to set up with incomplete library docs — [itch.io post](https://itch.io/post/16246532)

### Inferences
- An editor-less GameMaker pipeline on Linux is the weakest of the four: the manual documents Igor only on Windows/macOS hosts, licence provisioning needs an IDE login or website access key, and the only headless "test" mode is "did the runner survive N seconds". GML has no first-class way to host a foreign-language sim on web beyond JS extensions.
- The format being JSON-ish is agent-friendly in principle, but GameMaker rewrites files on open, so round-tripping without the IDE risks drift.

### Gaps
- Whether Igor binaries ship for a Linux host in the 2025/2026 runtime feed (the manual's `<OS>` placeholder suggests yes, but no Linux path or example was found).
- Enterprise price on the current page (not stated; 2023 reports say $79.99/mo).

---

## Unreal (one paragraph)

### Takeaway
Not practical here: the prebuilt Linux editor was a ~20 GB download / ~60 GB install in 2022 and source builds of 5.4 reach ~190 GiB installed, Paper2D has been maintenance-only per Epic for years, and nothing in UE's toolchain targets a lightweight web build (UE5 has no official HTML5 target).

### Cited Findings
- Epic on Paper2D: "We are not actively working on Paper2D, we will continue to maintain it, but no feature development is occurring" (UE4-era thread) — [Unreal forums: Paper2D dead?](https://forums.unrealengine.com/t/paper2d-dead/121200); current docs still list Paper 2D as the plugin for sprites, flipbooks and tile sets — [2D in Unreal Engine](https://dev.epicgames.com/documentation/unreal-engine/2d-in-unreal-engine)
- Linux editor download ~20 GB, ~60 GB installed, EULA + Epic account required (2022) — [GamingOnLinux](https://www.gamingonlinux.com/2022/07/unreal-engine-5-editor-quietly-gets-a-proper-linux-version/); UE 5.4 source build "~190 GiB" installed, 340+ GiB to build — [Arch wiki](https://wiki.archlinux.org/title/Unreal_Engine_5)

### Inferences
- Even ignoring web, the container cost alone (tens of GB plus hours of cook time) disqualifies it for an agent loop; `UnrealEditor-Cmd -nullrhi` headless commandlets exist but were not researched further.

### Gaps
- No 2025-2026 statement on Paper2D's status or current Linux download size (Epic's Linux page returned 403).

---

## Cross-cutting: running a separate deterministic sim, hot reload, and how an agent verifies visuals headless

### Takeaway
None of the editors render in their headless modes (Unity `-nographics` skips GPU init; Godot `--headless` uses the headless display server and Dummy audio; Defold `--variant headless`), so headless screenshots from the engine itself are not available; the documented alternatives are xvfb/virtual framebuffer for the engine, or (for web builds) a browser automation tool (Playwright/Puppeteer) that loads the exported game and screenshots it. A separate-language sim reaches the engine via JS interop on web (Unity .jslib/`SendMessage`, Godot `JavaScriptBridge`, Defold `html5.run()`) and via sockets/stdio or a native extension on desktop.

### Cited Findings
- Unity `-nographics`: "skips graphics device initialization, so you can run automated workflows on machines without a GPU... doesn't allow GI baking. Simulated input only works while a window is in focus" — [Editor command line arguments](https://docs.unity3d.com/Manual/EditorCommandLineArguments.html); a Linux-server Docker guide runs Unity under `xvfb-run` with `-batchmode -nographics` — [Edgegap Unity server in Docker](https://docs.edgegap.com/docs/tools-and-integrations/unity-server-in-docker)
- Godot `--headless` = headless display server + Dummy audio; dedicated-server export can "Strip Visuals" (textures replaced by placeholders storing only image size) — [Command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html), [Exporting for dedicated servers](https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_dedicated_servers.html); a Godot MCP screenshot tool notes it "requires a display server (not headless mode)" and suggests Xvfb on Linux — [godot-mcp-rts take_screenshot](https://glama.ai/mcp/servers/mikeumus/godot-mcp-rts/tools/take_screenshot); `--write-movie` exists for frame capture when rendering is available — [Command line tutorial](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- Defold's automated testing manual: runtime screenshots come from "a running build"; for HTML5, use Playwright/Puppeteer/Selenium to load the bundle, send input, collect console errors and screenshots; "Multimodal evaluation can flag issues... but it is a supplementary signal, not a replacement for deterministic checks"; Defold calls JS with `html5.run()` — [Automated testing manual](https://defold.com/manuals/automated-testing/)
- Godot web: GDExtension on web requires wasm-compiled extensions, Extensions Support, and cross-origin isolation headers; networking on web limited to HTTP/WebSocket/WebRTC (no low-level sockets) — [Exporting for the Web](https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_web.html)
- Defold native extensions compile on `build.defold.com` by default or a self-hosted Extender — [Bob manual](https://defold.com/manuals/bob/)
- Hot reload: Defold has Hot Reload but it "doesn't work in HTML5 builds" — [HTML5 manual](https://defold.com/manuals/html5/); Godot C# hot reload loses non-exported state — [C# basics](https://docs.godotengine.org/en/4.5/tutorials/scripting/c_sharp/c_sharp_basics.html); Unity CLI `recompile` and `watch` re-run commands on file change against a running Editor — [Unity CLI reference](https://docs.unity.com/en-us/unity-cli/unity-cli-reference.md)
- Unity Web <-> JS interop via .jslib plugins and `SendMessage` is standard; the manual page `web-interacting-browser-scripting.html` returned 404 at the URL tried, so no quote.

### Inferences
- Best agent verification loop across engines: (1) deterministic sim tests in the sim's own language, (2) engine smoke tests headless (GUT / DefTest / Unity EditMode), (3) a web export served locally and screenshotted with Playwright (works identically for Unity Web, Godot Web and Defold HTML5 and needs only a headless Chromium in the container, not a GPU), (4) optionally xvfb + the desktop build for engine-native captures.
- Keeping the sim in TypeScript and the engine as a renderer is cleanest for web-first: Godot via `JavaScriptBridge`, Defold via `html5.run()`, Unity via `.jslib`. On desktop the same sim would need a Node sidecar over WebSocket (all three engines have WebSocket clients) or a port to Rust/C# as a native extension/plugin. Compiling the sim to WebAssembly and loading it inside a Unity or Godot web build is possible in principle (both are Emscripten hosts) but no documentation of that pattern was found.

### Gaps
- No primary documentation fetched for Unity jslib interop or Godot `JavaScriptBridge` (both known APIs; only the Defold equivalent was quoted).
- No published example of an agent-run visual regression loop for any of these engines beyond Defold's manual.
