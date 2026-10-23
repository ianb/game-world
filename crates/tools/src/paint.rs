//! ANSI colour for the text views, by glyph. Plain text stays the source of truth; this is
//! a coat of paint for terminals.
use std::fmt::Write as _;

const RESET: &str = "\x1b[0m";

fn style(ch: char) -> Option<&'static str> {
    Some(match ch {
        '·' => "\x1b[32m",                           // grass: green
        '"' => "\x1b[32;1m",                         // tall grass: bold green
        '♠' => "\x1b[32;2m",                         // brush, overgrown path: dim green
        '┈' => "\x1b[33;2m",                         // weedy path: dim yellow
        '░' => "\x1b[33m",                           // path: yellow
        '∙' => "\x1b[93m",                           // track: bright yellow
        '▒' => "\x1b[93;1m",                         // worn path: bold bright yellow
        '♣' => "\x1b[34m",                           // blocked: blue
        '█' => "\x1b[90m",                           // wall: grey
        '▫' | '▪' => "\x1b[37;1m",                   // openings and doors: bold white
        c if c.is_ascii_lowercase() => "\x1b[35m",   // objects: magenta
        c if c.is_ascii_uppercase() => "\x1b[96;1m", // villagers: bold cyan
        _ => return None,
    })
}

/// Colours a rendered place. Header and row-number columns are left alone: only the map
/// body (after the four-character row prefix) is painted.
#[must_use]
pub fn paint(text: &str) -> String {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if i < 2 || line.len() < 4 {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let (prefix, body) = line.split_at(4);
        out.push_str(prefix);
        // Runs of one style share a single escape, so the output stays readable in a pager.
        let mut current: Option<&'static str> = None;
        for ch in body.chars() {
            let wanted = style(ch);
            if wanted != current {
                if current.is_some() {
                    out.push_str(RESET);
                }
                if let Some(code) = wanted {
                    let _ = write!(out, "{code}");
                }
                current = wanted;
            }
            out.push(ch);
        }
        if current.is_some() {
            out.push_str(RESET);
        }
        out.push('\n');
    }
    out
}

/// Whether to colour: `--color` forces it, otherwise only on a terminal.
#[must_use]
pub fn wanted(flag: bool) -> bool {
    use std::io::IsTerminal as _;
    flag || std::io::stdout().is_terminal()
}
