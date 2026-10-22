# Research

Produced October 2026, before any design or code decisions.

- `living-world-game-research.md` is the synthesized report on the simulation. Read this first.
- `generated-game-assets.md` is the report on generating 2D assets with Google's image models, rival APIs and pixel-art tools, with a recommended pipeline and first experiments.
- `notes/` holds the raw research notes it was written from, one per track:
  - `shipped_games_living_worlds.md`: how shipped games built NPC schedules,
    needs, relationships and world simulation, with postmortem lessons.
  - `social_simulation_research.md`: academic and experimental social
    simulation and interactive drama (Facade, CiF, Versu, Talk of the Town,
    Kismet, Ceptre, storylets, drama managers).
  - `npc_ai_architectures_and_authoring.md`: utility AI, GOAP, HTN, behavior
    trees, schedule systems, smart objects, dialogue rule systems, pathfinding,
    simulation testing, and authoring tools (ink, Yarn Spinner, Twine, articy,
    storylet libraries).
  - `llm_driven_npcs.md`: generative agents, AI Town, Project Sid, middleware,
    shipped LLM games, hybrid designs, cost and latency, authoring-time use.
  - `engine_and_stack_options.md`: JS/TS engines, Godot, Bevy, Ebitengine and
    others; map and asset tooling; headless testing; deployment.
  - `assets_google_image_models.md`: Google's Nano Banana and Imagen lineup,
    reference images, transparency, resolution, pricing, terms.
  - `assets_transparency_and_reference_images.md`: non-Google providers on
    native alpha, reference conditioning, LoRA training, hosting, licensing.
  - `assets_pipelines_and_pixel_art_tools.md`: Retro Diffusion, PixelLab,
    tilesets and autotile formats, Node post-processing, open-asset hybrids.

Each note section has a takeaway, cited findings with inline links and dates,
inferences marked as such, and gaps. Several primary sources were blocked
during research (GDC Vault, the Creation Kit wiki, the Valve developer wiki,
some UCSC paper PDFs, the Phaser 4 release post), so some findings rest on
abstracts or secondary write-ups and are labeled that way in the notes.

The notes are unedited research output. The reports were written to the
project's prose preferences; the notes were not.
