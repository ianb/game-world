//! A PNG of a place: cells coloured by kind, objects as dark squares, roofs as outlines,
//! villagers as bright squares with a per-villager outline. For a person, or the agent's
//! image reader, to glance at.
use characters::Town;
use content::{Cell, Pos, World};
use image::{Rgb, RgbImage};
use sim::Sim;
use sim::terrain::Surface;

const TILE: u32 = 8;

fn cell_colour(c: Cell) -> Rgb<u8> {
    match c {
        Cell::Ground => Rgb([96, 140, 72]),
        Cell::Path => Rgb([176, 150, 96]),
        Cell::Blocked => Rgb([40, 70, 110]),
        Cell::Wall => Rgb([70, 60, 55]),
        Cell::Floor => Rgb([190, 160, 120]),
        Cell::Opening => Rgb([220, 200, 150]),
        Cell::Door => Rgb([230, 190, 90]),
        Cell::Void => Rgb([20, 20, 20]),
    }
}

fn surface_colour(s: Surface) -> Rgb<u8> {
    match s {
        Surface::Grass => Rgb([96, 140, 72]),
        Surface::TallGrass => Rgb([76, 118, 58]),
        Surface::Brush => Rgb([58, 92, 50]),
        Surface::Track => Rgb([136, 150, 84]),
        Surface::Cowpath => Rgb([168, 150, 96]),
        Surface::Path => Rgb([176, 150, 96]),
        Surface::WeedyPath => Rgb([140, 140, 80]),
        Surface::OvergrownPath => Rgb([96, 120, 60]),
    }
}

fn fill(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, colour: Rgb<u8>) {
    for yy in y..(y + h).min(img.height()) {
        for xx in x..(x + w).min(img.width()) {
            img.put_pixel(xx, yy, colour);
        }
    }
}

fn outline(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, colour: Rgb<u8>) {
    fill(img, x, y, w, 1, colour);
    fill(img, x, y + h - 1, w, 1, colour);
    fill(img, x, y, 1, h, colour);
    fill(img, x + w - 1, y, 1, h, colour);
}

/// `surface(x, y)` is the living ground of an outdoor tile, None indoors.
#[must_use]
pub fn render_place(
    world: &World,
    place: usize,
    villagers: &[(usize, content::Pos, usize)],
    surface: &dyn Fn(u16, u16) -> Option<Surface>,
) -> RgbImage {
    let p = &world.places[place];
    let mut img = RgbImage::new(u32::from(p.width) * TILE, u32::from(p.height) * TILE);
    for y in 0..p.height {
        for x in 0..p.width {
            fill(
                &mut img,
                u32::from(x) * TILE,
                u32::from(y) * TILE,
                TILE,
                TILE,
                surface(x, y).map_or_else(|| cell_colour(p.cell(x, y)), surface_colour),
            );
        }
    }
    for o in world.objects.iter().filter(|o| o.place == place) {
        let (x, y) = (u32::from(o.at.x) * TILE, u32::from(o.at.y) * TILE);
        let colour = match o.kind.as_str() {
            "bed" => Rgb([150, 60, 90]),
            "table" | "counter" => Rgb([120, 80, 40]),
            "oven" | "stove" => Rgb([60, 60, 60]),
            _ => Rgb([100, 100, 60]),
        };
        fill(&mut img, x + 1, y + 1, TILE - 2, TILE - 2, colour);
    }
    for r in world.roofs.iter().filter(|r| r.place == place) {
        outline(
            &mut img,
            u32::from(r.rect.x) * TILE,
            u32::from(r.rect.y) * TILE,
            u32::from(r.rect.w) * TILE,
            u32::from(r.rect.h) * TILE,
            Rgb([255, 255, 255]),
        );
    }
    let palette = [
        Rgb([255, 80, 80]),
        Rgb([80, 160, 255]),
        Rgb([255, 220, 60]),
        Rgb([200, 90, 255]),
        Rgb([60, 230, 200]),
    ];
    for &(vp, pos, index) in villagers {
        if vp != place {
            continue;
        }
        let (x, y) = (u32::from(pos.x) * TILE, u32::from(pos.y) * TILE);
        let colour = palette[index % palette.len()];
        fill(
            &mut img,
            x + 1,
            y + 1,
            TILE - 2,
            TILE - 2,
            Rgb([255, 255, 255]),
        );
        outline(&mut img, x, y, TILE, TILE, colour);
        fill(&mut img, x + 3, y + 3, TILE - 6, TILE - 6, colour);
    }
    img
}

/// Every place stacked vertically with a gap, villagers drawn in.
#[must_use]
pub fn render_town(sim: &Sim<Town>) -> RgbImage {
    let world = sim.world();
    let villagers: Vec<(usize, content::Pos, usize)> = sim
        .state()
        .villagers
        .iter()
        .enumerate()
        .map(|(i, v)| (v.place.index(), v.pos, i))
        .collect();
    let images: Vec<RgbImage> = (0..world.places.len())
        .map(|i| render_place(world, i, &villagers, &|x, y| sim.surface(i, Pos { x, y })))
        .collect();
    let gap = TILE;
    let width = images.iter().map(RgbImage::width).max().unwrap_or(1);
    let height: u32 = images.iter().map(|i| i.height() + gap).sum();
    let mut out = RgbImage::from_pixel(width, height, Rgb([20, 20, 20]));
    let mut y = 0;
    for img in &images {
        image::imageops::replace(&mut out, img, 0, i64::from(y));
        y += img.height() + gap;
    }
    out
}
