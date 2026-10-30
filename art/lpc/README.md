# LPC art

The viewer's tiles, villagers and animals are built from Liberated Pixel Cup (LPC) art:
pixel art that hundreds of artists have contributed to OpenGameArt since the
2012 contest. Most of it is CC-BY-SA 3.0 and GPL 3.0, some also OGA-BY or
CC-BY. Each file's authors and licenses are in the credits next to it.

`packages/viewer/public/tiles.png` is made from these files, so it is under
the same terms: CC-BY-SA 3.0 (or GPL 3.0), with the credits below.

## What is here

| Folder | Source | Credits |
| --- | --- | --- |
| `tiles/base` | [LPC base assets](https://opengameart.org/content/liberated-pixel-cup-lpc-base-assets-sprites-map-tiles) (Lanea Zimmerman and others) | `tiles/base/CREDITS.TXT` |
| `tiles/atlas` | [LPC tile atlas](https://opengameart.org/content/lpc-tile-atlas) and [atlas 2](https://opengameart.org/content/lpc-tile-atlas2) | `tiles/atlas/Attribution.txt`, `Attribution2.txt` |
| `tiles/house-interior` | [House interior and decorations](https://opengameart.org/content/lpc-house-interior-and-decorations) (Reemax and others) | `tiles/house-interior/credits.txt` |
| `tiles/walls` | [[LPC] Walls](https://opengameart.org/content/lpc-walls) (bluecarrot16 and others) | `tiles/walls/CREDITS-walls.txt` |
| `tiles/windows-doors` | [[LPC] Windows & Doors](https://opengameart.org/content/lpc-windows-doors) | `tiles/windows-doors/CREDITS-windows-doors.txt` |
| `tiles/decorations-medieval` | [[LPC] Medieval village decorations](https://opengameart.org/content/lpc-medieval-village-decorations) | `tiles/decorations-medieval/CREDITS-decorations-medieval.txt` |
| `animals` | [LPC style farm animals](https://opengameart.org/content/lpc-style-farm-animals) (Daniel Eddeland) | `animals/CREDITS.md` |
| `characters` | layers from the [Universal LPC Spritesheet Character Generator](https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator) | `characters/CREDITS.md` |

The source sheets are kept whole, so a different tile can be picked from
them without fetching anything.

## Rebuilding

- `just tileset` composes `tiles.png` and `tiles.json` from these files.
  Which part of which sheet becomes which tile is in
  `scripts/make-tileset.mjs`.
- What each villager wears is in `characters.json`, as generator items and
  colors. After changing it, run
  `node scripts/lpc-vendor.mjs <generator checkout>` to copy the layers it
  needs (a sparse clone of the generator is enough; the script fetches the
  files it uses), then `just tileset`.
