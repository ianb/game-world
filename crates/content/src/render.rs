//! ASCII views of compiled places. Generated, never edited; the agent's primary way to see.
use crate::compile::{Cell, World};
use crate::model::Pos;
use std::fmt::Write as _;

/// Legend: `.` ground, `^` blocked, `#` wall, `,` floor, `+` opening, `>` door tile,
/// lowercase letter = object (first letter of its kind), overlay glyphs win.
#[must_use]
pub fn render_place(world: &World, place: usize, overlay: &[(Pos, char)]) -> String {
    let p = &world.places[place];
    let mut rows: Vec<Vec<char>> = (0..p.height)
        .map(|y| (0..p.width).map(|x| p.cell(x, y).glyph()).collect())
        .collect();
    for o in world
        .objects
        .iter()
        .filter(|o| o.place == place && p.in_bounds(o.at.x, o.at.y))
    {
        let glyph = o.kind.chars().next().unwrap_or('?').to_ascii_lowercase();
        rows[usize::from(o.at.y)][usize::from(o.at.x)] = glyph;
    }
    for (pos, glyph) in overlay {
        if p.in_bounds(pos.x, pos.y) {
            rows[usize::from(pos.y)][usize::from(pos.x)] = *glyph;
        }
    }
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} ({}x{}) arrival ({}, {})",
        p.id, p.width, p.height, p.arrival.x, p.arrival.y
    );
    out.push_str("    ");
    for x in 0..p.width {
        out.push(char::from_digit(u32::from(x % 10), 10).unwrap_or('?'));
    }
    out.push('\n');
    for (y, row) in rows.iter().enumerate() {
        let _ = write!(out, "{y:3} ");
        out.extend(row.iter());
        out.push('\n');
    }
    out
}

/// One line per object in a place, for cross-checking the render.
#[must_use]
pub fn list_objects(world: &World, place: usize) -> String {
    let mut lines: Vec<String> = world
        .objects
        .iter()
        .filter(|o| o.place == place)
        .map(|o| {
            let mut s = format!(
                "{} {} at ({}, {}) in {}/{}",
                o.id, o.kind, o.at.x, o.at.y, o.building, o.room
            );
            if let Some(owner) = &o.owner {
                let _ = write!(s, " owner={owner}");
            }
            if let Some(c) = o.capacity {
                let _ = write!(s, " capacity={c}");
            }
            if let Some(a) = &o.activity {
                let _ = write!(s, " activity={a}");
            }
            s
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

/// One line per door, both ends in place coordinates.
#[must_use]
pub fn list_doors(world: &World) -> String {
    let mut lines: Vec<String> = world
        .doors
        .iter()
        .map(|d| {
            let arrow = if d.one_way { "->" } else { "<->" };
            let mut s = format!(
                "{} ({}): {} ({}, {}) {arrow} {} ({}, {})",
                d.id,
                d.building,
                world.places[d.from_place].id,
                d.from.x,
                d.from.y,
                world.places[d.to_place].id,
                d.to.x,
                d.to.y
            );
            if let Some(h) = &d.hours {
                let _ = write!(s, " hours={h}");
            }
            if let Some(o) = &d.owner {
                let _ = write!(s, " owner={o}");
            }
            s
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

/// Marks every cell, for debugging the compiler.
#[must_use]
pub fn cell_name(c: Cell) -> &'static str {
    match c {
        Cell::Ground => "ground",
        Cell::Blocked => "blocked",
        Cell::Wall => "wall",
        Cell::Floor => "floor",
        Cell::Opening => "opening",
        Cell::Door => "door",
        Cell::Void => "void",
    }
}
