//! The fields in the running sim: laid out from content, changed every night (with rain
//! some nights), and worked tile by tile by whoever stands on them.
use super::Sim;
use crate::brain::CharacterSet;
use crate::fields::{Fields, Plot, crop_rules, starting_tiles, work};
use crate::ids::{ObjectId, PlaceId, VillagerId};
use crate::protocol::{Emitted, FieldWork, GoodsLine};
use crate::time::Season;

use crate::fields::RAINS;

/// Chance, in percent, of rain on a night in each season.
#[must_use]
pub fn rain_percent(season: Season) -> i16 {
    match season {
        Season::Spring => 40,
        Season::Summer => 25,
        Season::Autumn => 35,
        Season::Winter => 30,
    }
}

impl<C: CharacterSet> Sim<C> {
    /// Every plot in content, sown and some way along.
    pub(super) fn lay_out_fields(&self) -> Fields {
        let mut plots = Vec::new();
        for z in &self.content.zones {
            let place = PlaceId(u16::try_from(self.world.place_index[&z.id]).expect("few places"));
            for p in &z.plots {
                let store = p.store.as_ref().and_then(|s| {
                    self.world
                        .objects
                        .iter()
                        .position(|o| &o.id == s)
                        .map(|i| ObjectId(u16::try_from(i).expect("few objects")))
                });
                plots.push(Plot {
                    id: p.id.clone(),
                    place,
                    rect: p.rect,
                    crop: p.crop,
                    store,
                    gathered: 0,
                    tiles: starting_tiles(plots.len(), p.rect, p.crop),
                });
            }
        }
        Fields { plots }
    }

    /// The night's change on every field, and whether it rained.
    pub(super) fn fields_new_day(&mut self, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let season = tick.calendar().season;
        let rained =
            self.state
                .odds
                .chance_any(&mut self.state.rng.weather, RAINS, rain_percent(season));
        if rained {
            out.push(Emitted::Rained { tick });
        }
        self.moods_weather(rained, out);
        let night = self.state.fields.new_day(season, rained);
        let id = |p: u16| self.state.fields.plots[usize::from(p)].id.clone();
        for (p, tiles) in night.ripened {
            out.push(Emitted::CropsRipened {
                tick,
                plot: id(p),
                tiles,
            });
        }
        for (p, tiles) in night.spoiled {
            out.push(Emitted::CropsSpoiled {
                tick,
                plot: id(p),
                tiles,
            });
        }
    }

    /// `who` does `job` to the field tile they stand on.
    pub(super) fn tend(&mut self, who: VillagerId, job: FieldWork, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let v = &self.state.villagers[who.index()];
        let (place, pos) = (v.place, v.pos);
        let Some((p, i)) = self
            .state
            .fields
            .plots
            .iter()
            .enumerate()
            .find_map(|(p, plot)| {
                (plot.place == place)
                    .then(|| plot.index_at(pos).map(|i| (p, i)))
                    .flatten()
            })
        else {
            return;
        };
        let plot = &mut self.state.fields.plots[p];
        let crop = plot.crop;
        if !work(&mut plot.tiles[i], crop, job) {
            return;
        }
        let mut stored = None;
        if job == FieldWork::Harvest {
            let rules = crop_rules(plot.crop);
            plot.gathered += 1;
            if plot.gathered >= rules.tiles_per_unit {
                plot.gathered -= rules.tiles_per_unit;
                if let Some(store) = plot.store {
                    let n = self
                        .state
                        .stock
                        .entry(store.0)
                        .or_default()
                        .entry(rules.good.to_string())
                        .or_insert(0);
                    *n = n.saturating_add(1);
                    stored = Some(GoodsLine {
                        good: rules.good.to_string(),
                        count: 1,
                    });
                }
            }
        }
        let plot = &self.state.fields.plots[p];
        out.push(Emitted::Tended {
            tick,
            who,
            plot: plot.id.clone(),
            x: pos.x,
            y: pos.y,
            work: job,
            stored,
            cause: self.cause.clone(),
        });
    }
}
