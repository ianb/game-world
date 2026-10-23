# Blocked and unverified sources

Sources the October 2026 research could not read directly. Each entry says
what it would settle and which note cites it. When one is checked, fold what
it says into the note (and the report, if it changes a conclusion) and tick
it here.

Notes referenced: `shipped` = shipped_games_living_worlds.md, `social` =
social_simulation_research.md, `npc-ai` = npc_ai_architectures_and_authoring.md,
`llm` = llm_driven_npcs.md, `stack` = engine_and_stack_options.md.

## Login-walled (GDC Vault)

GDC Vault was in maintenance during research and its recordings need a
login anyway. All of these were cited from abstracts or secondary write-ups.

- [ ] Richard Evans, "Modeling Individual Personalities in The Sims 3" (GDC 2010).
  Would settle: the Sims 3 trait and social-rule system as designed.
  https://gdcvault.com/play/1012294/Modeling-Individual-Personalities-in-The (shipped)
- [ ] Ubisoft, Watch Dogs: Legion "Census" system talk (GDC 2021).
  Would settle: the two-layer profile system and affinity propagation details. (shipped)
- [ ] IO Interactive, Hitman 2 crowd and NPC scalability talk (GDC 2019).
  Would settle: crowd FSM vs. full NPC split, situations, LOD thresholds. (shipped)
- [ ] Warhorse, Kingdom Come: Deliverance II AI talk (GDC 2026).
  Would settle: how GOAP and behavior trees divide daily-life NPC work, AI LOD at ~2,400 NPCs. (shipped, npc-ai)
- [ ] Dave Mark, Infinite Axis Utility System talks (GDC 2013, 2015).
  Would settle: the compensation factor used when multiplying many considerations. (npc-ai)
- [ ] Jeff Orkin, "Three States and a Plan" (GDC 2006). (npc-ai)
- [ ] Guerrilla, "Beyond Killzone" (GDC 2018) and "HTN Planning in Decima" (GDC 2024).
  Would settle: Horizon's HTN plus utility split. (npc-ai)
- [ ] Wildermyth narrative-system talk.
  Would settle: hooks, aspects and role casting as built. (social)

## Anti-bot or 403 on direct fetch

Findings from these rest on search-engine snippets and are marked
"(snippet)" in the notes.

- [ ] Creation Kit wiki, Package (Form), Procedure, and Bethesda Tutorial Packages.
  Would settle: schedule tab fields, package stack evaluation order and
  re-evaluation frequency, Must Complete and Interrupt Override flags,
  procedure tree node types.
  https://ck.uesp.net/wiki/Package_(Form)
  https://ck.uesp.net/wiki/Procedure
  https://ck.uesp.net/wiki/Bethesda_Tutorial_Packages (shipped, npc-ai)
- [ ] Valve Developer Community, Response System.
  Would settle: criteria scoring and the Left 4 Dead rewrite. The Alien Swarm
  SDK source is the fallback primary.
  https://developer.valvesoftware.com/wiki/Response_System (npc-ai)
- [ ] ColePowered, Shadows of Doubt DevBlog #8 "Simulating a City" and the
  Wireframe magazine feature.
  Would settle: the precomputed-schedule representation, the 23 traits, how
  killers plan.
  https://colepowered.com/?p=34142 (shipped)
- [ ] AI and Games, "The Genius AI Behind The Sims" full transcript.
  Would settle: Will Wright's own description of advertisements and motive scoring.
  https://amara.org/subtitles/cVoJS4OdmVql/en/1/download/The%20Genius%20AI%20Behind%20The%20Sims.en.txt (shipped)
- [ ] Sims fandom wiki pages on Sims 1 motives and scoring. (shipped)
- [ ] Phaser 4 release post (phaser.io returned 404) and the GameDev.net
  article (403).
  Would settle: renderer internals, TypeScript typing quality, bundle size,
  v3-to-v4 migration list, Tiled loader changes, Phaser 3 end of maintenance.
  https://gamedev.net/news/2759-phaser-4-released/ (stack)
- [ ] Inworld June 2026 price-cut press release (403).
  https://www.morningstar.com/news/business-wire/20260610968386/inworld-cuts-prices-to-take-down-the-biggest-wall-in-consumer-ai-cost (llm)
- [ ] OpenAI main pricing page (403; the developer-docs mirror was used). (llm)
- [ ] GamesRadar interview with Bruce Nesmith on Skyrim's NPC system (read via an inkl mirror). (shipped)

## Not found, empty download, or server error

- [ ] Tarn Adams, "Characterization and Emergent Narrative in Dwarf Fortress",
  chapter in *Procedural Storytelling in Game Design* (Routledge). The PDF
  link returned 404. Also his "Simulation Principles from Dwarf Fortress" in
  Game AI Pro 2.
  Would settle: the needs and facets design rationale as he states it. (shipped, social)
- [ ] James Ryan, dissertation "Curating Simulated Storyworlds" (UCSC; PDF
  download came back empty); "Toward Characters Who Observe, Tell,
  Misremember, and Lie" (EXAG 2015); the Bad News ICIDS 2016 paper; "A simple
  method for evolving large character social networks" (2016).
  Would settle: the belief-model and gossip details, the routine generator,
  the narrativized trace format. (social)
- [ ] Ensemble engine FDG 2015 paper, and a complete JSON example of a
  volition or trigger rule from the Ensemble repo. Table I of the CiF 2014
  paper (example influence rules) did not survive PDF text extraction. (social)
- [ ] Kismet ICCC 2020 workshop paper (only the informal Google Doc spec was read). (social)
- [ ] Caves of Qud "Mythic Biographies" paper. (social)
- [ ] Azad and Martens, taxonomy of social simulation. (social)
- [ ] Max Kreminski, storylet survey and "Select the Unexpected" (cited secondhand). (social)
- [ ] Ultima VI SCHEDULE.DAT byte layout. Nuvie's loader source is the primary reference. (shipped)
- [ ] CK3 script schema for `opinion_modifiers` (decay, months, stacking fields). (npc-ai)
- [ ] Yarn Spinner 3 changelog (exact release date; whether try.yarnspinner.dev
  runs a JS or WASM runtime). (npc-ai)
- [ ] articy JSON export schema from articy's own help pages (only a
  third-party importer was read). (npc-ai)
- [ ] Guerrilla's published HTN PDF. (npc-ai)

## Not reached in research budget

Nothing blocked these; the researcher ran out of time.

- [ ] Godot: write-ups on hand-writing `.tscn`/`.tres` text files as a
  code-only workflow; C#/.NET web export status in 4.6/4.7; exact gdUnit4 and
  GUT headless CLI flags. (stack)
- [ ] Bevy official docs on headless (`MinimalPlugins`) and wasm in 2026
  (WebGL2 vs WebGPU default, threads on web, bundle size). (stack)
- [ ] Defold, Unity, Unreal, Zig/C web and headless status. (stack)
- [ ] headless-gl and node-canvas maintenance status in 2026. (stack)
- [ ] Bundle size and cold-load figures for each JS engine. (stack)
- [ ] Cloudflare Pages `_headers` configuration for COOP/COEP (only needed for
  Godot threaded or love.js builds). (stack)
- [ ] Tiled 1.11/1.12 release specifics. (stack)
- [ ] Prom Week's CiF papers in full (the dev-blog summary was used in npc-ai;
  the social note did read the 2014 TCIAIG paper). (npc-ai)
- [ ] Ceptre follow-ups (Ceptre Editor 2019). (social)
- [ ] FAtiMA, PsychSim, Thespian primary papers (summaries only). (social)
- [ ] Any developer source on Yakuza's crowd system or Kenshi's faction
  simulation; any ConcernedApe interview on schedule design. (shipped)
- [ ] Altera's status after Project Sid; Emily Short's position on LLM NPCs
  (only a 2023 lecture listing was found). (llm)

## Asset generation track (added 2026-10-22)

Notes referenced: `assets-google` = assets_google_image_models.md,
`assets-alpha` = assets_transparency_and_reference_images.md,
`assets-pipe` = assets_pipelines_and_pixel_art_tools.md.

- [ ] OpenAI per-image price table for gpt-image-2 / 2.5 and the OpenAI
  Business Terms (403). Third-party figure of about $0.05 per 2K image was
  used instead. (assets-alpha)
- [ ] Ideogram 4.x per-image dollar prices (rendered client-side; use the
  API's `dry_run=true` PriceQuote instead). (assets-alpha)
- [ ] Google Agent Platform (ex-Vertex) per-image prices for the Nano Banana
  models, and the exact text of the Google Cloud indemnity conditions. (assets-google)
- [ ] Google docs on mask inputs for Nano Banana editing (only "semantic
  masking" by prompt was found); exact per-tier image rate limits. (assets-google)
- [ ] Phil Schmid's sticker-generation post and two Medium write-ups on Nano
  Banana sprites (404/403). (assets-google)
- [ ] Adobe Firefly API indemnity coverage; Krea API docs; remove.bg pricing. (assets-alpha)
- [ ] Retro Diffusion and PixelLab per-tier license text (official pages
  unreachable; the FAQ claim that outputs belong to the user is vendor-stated). (assets-pipe)
- [ ] Any independent quality review or benchmark of PixelLab or Retro
  Diffusion output; any background-remover test on pixel art; whether
  SynthID survives a 16x nearest-neighbour downscale. These are gaps in the
  world, not blocked pages; the "first experiments" in the asset report are
  designed to fill them. (assets-google, assets-alpha, assets-pipe)
- [ ] Rosebud, DragonBones, and the Tuxemon / Sprout Lands / LimeZu asset
  licences were not covered. (assets-pipe)

## World layout track (added 2026-10-23)

Notes referenced: `layout-games` = layout_world_structure_in_shipped_games.md,
`layout-tools` = layout_tooling_formats_and_generation.md,
`layout-present` = layout_presenting_interiors_and_floors.md.

- [ ] Exult forum threads and Exult Studio HTML docs, and Ultima Codex IFIX
  (fixed-object) format details (Cloudflare blocks). Would confirm how upper
  storeys and roofs are stored and the modders' account of empty upper
  floors. (layout-games, layout-present)
- [ ] ColePowered, Shadows of Doubt DevBlog #13 on building generation (body
  not extractable). (layout-games, layout-tools)
- [ ] Botea, Müller, Schaeffer, "Near Optimal Hierarchical Path-Finding"
  (HPA*) PDF (503). (layout-games)
- [ ] A Link to the Past entrance and room tables. (layout-games)
- [ ] The Indie Stone's original Build 42 post on the height-limit change. (layout-games)
- [ ] Merrell et al. 2011 "Interactive Furniture Layout Using Interior Design
  Guidelines" and Lopes et al. 2010 floorplan generation PDFs (503). (layout-tools)
- [ ] Brogue blueprint/vault source (404). (layout-tools)
- [ ] Tiled's current `main.cpp`, to confirm the exact CLI flags
  (`--export-formats`, `--project`). LDtk Super Simple Export `data.json`
  fields and the multi-world folder layout. (layout-tools)
- [ ] Any primary source on The Sims' wall-on-edge representation (FreeSO's
  reimplementation was used). (layout-games, layout-tools)
- [ ] Exult official FAQ, Bulbapedia, TV Tropes, Prima guides, the Sims
  fandom wiki and the Dwarf Fortress wiki (403/429/503); claims from them
  are marked "(excerpt)". (layout-present)
- [ ] Any ConcernedApe statement on choosing separate interior maps or on
  villager map markers; any technical source on Harvest Moon, Graveyard
  Keeper or Moonlighter scene structure; any GDC talk comparing scene swap
  with seamless interiors. These are gaps in the world rather than blocked
  pages. (layout-games, layout-present)
