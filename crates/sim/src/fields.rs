//! Fields. Every tile of a plot carries its own state: what stage it is at, how far its
//! crop has grown, and its water, weeds and fertility. Each day the land changes on its
//! own: soil dries unless it rains, weeds come up, crops grow (fast: one to two weeks from
//! sowing to ripe), ripe crops spoil if nobody takes them, and spoiled or stubble ground
//! goes fallow. Farmers change it by working one tile at a time (`Work`). All of it is in
//! the state and the hash, so a field's history replays.
use crate::ids::{ObjectId, PlaceId};
use crate::protocol::{FieldStage as Stage, FieldWork as Work};
use crate::time::Season;
use content::{Crop, Pos, Rect};
use serde::{Deserialize, Serialize};

/// Label of the nightly chance of rain, for tests to force.
pub const RAINS: &str = "rains";
/// Water a tile holds after rain or watering, in days of drying.
pub const WATER_FULL: u8 = 6;
/// Weeds run from 0 to this.
pub const WEEDS_MAX: u8 = 10;
/// Weeds at which growth slows, and at which a farmer bothers to weed.
pub const WEEDY: u8 = 5;
/// Fertility runs from 0 to this; a new field starts at `FERTILITY_START`.
pub const FERTILITY_MAX: u8 = 10;
pub const FERTILITY_START: u8 = 6;
/// Days a ripe crop stands before it starts to spoil, and days spoiling before it is lost.
pub const RIPE_DAYS: u8 = 5;
pub const SPOIL_DAYS: u8 = 3;
/// Days stubble stands before it is just fallow ground.
pub const STUBBLE_DAYS: u8 = 4;
/// Days fallow for each point of fertility the ground gets back.
pub const FALLOW_DAYS_PER_FERTILITY: u8 = 4;
/// Fertility at or below which growth slows, and below which a farmer rests the ground
/// rather than tilling it.
pub const POOR: u8 = 2;

/// What a crop needs and gives.
#[must_use]
#[deny(clippy::wildcard_enum_match_arm)]
pub fn crop_rules(crop: Crop) -> CropRules {
    let (grow, good, tiles_per_unit, feeds) = match crop {
        Crop::Wheat => (24, "wheat", 4, false),
        Crop::Corn => (24, "corn", 3, false),
        Crop::Potatoes => (20, "potatoes", 3, false),
        Crop::Carrots => (16, "carrots", 2, false),
        Crop::Cabbages => (14, "cabbages", 2, false),
        Crop::Beans => (14, "beans", 2, true),
        Crop::Flowers => (12, "flowers", 2, false),
    };
    CropRules {
        grow,
        good,
        tiles_per_unit,
        feeds,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CropRules {
    /// Growth points from sowing to ripe. A well-watered, weeded tile gains 2 a day.
    pub grow: u8,
    /// The good a harvest yields.
    pub good: &'static str,
    /// Tiles harvested for one unit of the good.
    pub tiles_per_unit: u8,
    /// Whether a harvest leaves the ground richer (beans) rather than poorer.
    pub feeds: bool,
}

/// One tile of a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldTile {
    pub stage: Stage,
    /// Growth points so far, toward the crop's `grow`.
    pub growth: u8,
    /// Days in the current stage, for ripe, spoiling, stubble and fallow ground.
    pub days: u8,
    /// 0 is dry; `WATER_FULL` after rain or watering.
    pub water: u8,
    pub weeds: u8,
    pub fertility: u8,
}

impl FieldTile {
    /// Whether a crop is in the ground.
    #[must_use]
    pub fn planted(&self) -> bool {
        matches!(
            self.stage,
            Stage::Sown | Stage::Sprouting | Stage::Growing | Stage::Ripe | Stage::Spoiling
        )
    }
}

/// A plot: a rectangle of tiles under one crop.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plot {
    pub id: String,
    pub place: PlaceId,
    pub rect: Rect,
    pub crop: Crop,
    /// Where the harvest goes, if anywhere; without one it is eaten at home.
    pub store: Option<ObjectId>,
    /// Harvested tiles not yet making up a whole unit of the good.
    pub gathered: u8,
    /// Row by row across `rect`.
    pub tiles: Vec<FieldTile>,
}

impl Plot {
    #[must_use]
    pub fn pos(&self, index: usize) -> Pos {
        let w = usize::from(self.rect.w.max(1));
        Pos {
            x: self.rect.x + u16::try_from(index % w).unwrap_or(0),
            y: self.rect.y + u16::try_from(index / w).unwrap_or(0),
        }
    }

    #[must_use]
    pub fn index_at(&self, pos: Pos) -> Option<usize> {
        self.rect.contains(pos.x, pos.y).then(|| {
            usize::from(pos.y - self.rect.y) * usize::from(self.rect.w)
                + usize::from(pos.x - self.rect.x)
        })
    }
}

/// Every plot in the world, in content order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fields {
    pub plots: Vec<Plot>,
}

/// What happened to the fields overnight, for the events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overnight {
    /// Plots that lost tiles of crop to spoiling, with how many.
    pub spoiled: Vec<(u16, u16)>,
    /// Plots with tiles that ripened, with how many.
    pub ripened: Vec<(u16, u16)>,
}

/// A season in which crops grow and can be sown.
#[must_use]
pub fn growing_season(season: Season) -> bool {
    !matches!(season, Season::Winter)
}

impl Fields {
    /// One day's change on every tile. `rained` soaks every field.
    pub fn new_day(&mut self, season: Season, rained: bool) -> Overnight {
        let mut night = Overnight::default();
        for (p, plot) in self.plots.iter_mut().enumerate() {
            let rules = crop_rules(plot.crop);
            let (mut spoiled, mut ripened) = (0u16, 0u16);
            for t in &mut plot.tiles {
                t.water = if rained {
                    WATER_FULL
                } else {
                    t.water.saturating_sub(1)
                };
                if growing_season(season) {
                    t.weeds = (t.weeds + 1).min(WEEDS_MAX);
                }
                t.days = t.days.saturating_add(1);
                match t.stage {
                    Stage::Sown | Stage::Sprouting | Stage::Growing => {
                        if growing_season(season) {
                            let gain = match (t.water, t.weeds >= WEEDY || t.fertility <= POOR) {
                                (0, _) => 0,
                                (_, true) => 1,
                                (_, false) => 2,
                            };
                            // A dry tile wilts back.
                            t.growth = if gain == 0 {
                                t.growth.saturating_sub(1)
                            } else {
                                t.growth.saturating_add(gain)
                            };
                        }
                        let stage = grown_stage(t.growth, rules.grow);
                        if stage != t.stage {
                            if stage == Stage::Ripe {
                                ripened += 1;
                            }
                            t.stage = stage;
                            t.days = 0;
                        }
                    }
                    Stage::Ripe if t.days >= RIPE_DAYS => {
                        t.stage = Stage::Spoiling;
                        t.days = 0;
                    }
                    Stage::Spoiling if t.days >= SPOIL_DAYS => {
                        spoiled += 1;
                        t.stage = Stage::Fallow;
                        t.growth = 0;
                        t.days = 0;
                    }
                    Stage::Stubble if t.days >= STUBBLE_DAYS => {
                        t.stage = Stage::Fallow;
                        t.days = 0;
                    }
                    Stage::Fallow if t.days >= FALLOW_DAYS_PER_FERTILITY => {
                        t.fertility = (t.fertility + 1).min(FERTILITY_MAX);
                        t.days = 0;
                    }
                    Stage::Ripe
                    | Stage::Spoiling
                    | Stage::Stubble
                    | Stage::Fallow
                    | Stage::Tilled => {}
                }
            }
            let p = u16::try_from(p).unwrap_or(u16::MAX);
            if spoiled > 0 {
                night.spoiled.push((p, spoiled));
            }
            if ripened > 0 {
                night.ripened.push((p, ripened));
            }
        }
        night
    }
}

/// The stage a crop has reached with `growth` points of the `grow` it needs.
#[must_use]
pub fn grown_stage(growth: u8, grow: u8) -> Stage {
    if growth >= grow {
        Stage::Ripe
    } else if growth * 2 >= grow {
        Stage::Growing
    } else if growth * 4 >= grow && growth > 0 {
        Stage::Sprouting
    } else {
        Stage::Sown
    }
}

/// The work a tile wants next, most pressing first: bring in what is ripe, then keep
/// the weeds and the drought off what is growing, then sow, then till. Nothing is
/// started out of season.
#[must_use]
pub fn wants(t: &FieldTile, season: Season) -> Option<Work> {
    match t.stage {
        Stage::Ripe | Stage::Spoiling => Some(Work::Harvest),
        Stage::Sown | Stage::Sprouting | Stage::Growing if t.weeds >= WEEDY => Some(Work::Weed),
        Stage::Sown | Stage::Sprouting | Stage::Growing if t.water <= 1 => Some(Work::Water),
        Stage::Tilled if growing_season(season) => Some(Work::Sow),
        // Spent ground is left to rest until it recovers.
        Stage::Fallow | Stage::Stubble if growing_season(season) && t.fertility > POOR => {
            Some(Work::Till)
        }
        Stage::Sown
        | Stage::Sprouting
        | Stage::Growing
        | Stage::Tilled
        | Stage::Fallow
        | Stage::Stubble => None,
    }
}

/// Minutes one piece of work takes on one tile.
#[must_use]
#[deny(clippy::wildcard_enum_match_arm)]
pub fn minutes(work: Work) -> u32 {
    match work {
        Work::Till => 4,
        Work::Sow | Work::Weed => 2,
        Work::Water => 1,
        Work::Harvest => 3,
    }
}

/// Does `work` to a tile of `crop`. Returns false when the tile does not take that work
/// now. A harvest is counted by the caller.
pub fn work(t: &mut FieldTile, crop: Crop, work: Work) -> bool {
    let ok = match (work, t.stage) {
        (Work::Till, Stage::Fallow | Stage::Stubble) => {
            t.stage = Stage::Tilled;
            t.weeds = 0;
            true
        }
        (Work::Sow, Stage::Tilled) => {
            t.stage = Stage::Sown;
            t.growth = 0;
            true
        }
        (Work::Water, Stage::Sown | Stage::Sprouting | Stage::Growing | Stage::Tilled) => {
            t.water = WATER_FULL;
            true
        }
        (Work::Weed, s) if s != Stage::Fallow => {
            t.weeds = 0;
            true
        }
        (Work::Harvest, Stage::Ripe | Stage::Spoiling) => {
            t.stage = Stage::Stubble;
            t.growth = 0;
            t.fertility = if crop_rules(crop).feeds {
                (t.fertility + 1).min(FERTILITY_MAX)
            } else {
                t.fertility.saturating_sub(1)
            };
            true
        }
        _ => false,
    };
    if ok {
        t.days = 0;
    }
    ok
}

/// The tiles a new world starts with: sown, and some way along, varying by plot so the
/// harvests do not all come at once.
#[must_use]
pub fn starting_tiles(index: usize, rect: Rect, crop: Crop) -> Vec<FieldTile> {
    let grow = crop_rules(crop).grow;
    // Plots start between a sixth and two thirds of the way to ripe.
    let sixths = u8::try_from(1 + index % 4).unwrap_or(1);
    let growth = grow * sixths / 6;
    let tile = FieldTile {
        stage: grown_stage(growth, grow),
        growth,
        days: 0,
        water: WATER_FULL / 2,
        weeds: 0,
        fertility: FERTILITY_START,
    };
    vec![tile; usize::from(rect.w) * usize::from(rect.h)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sown() -> FieldTile {
        FieldTile {
            stage: Stage::Sown,
            growth: 0,
            days: 0,
            water: WATER_FULL,
            weeds: 0,
            fertility: FERTILITY_START,
        }
    }

    fn plot(tile: FieldTile) -> Fields {
        Fields {
            plots: vec![Plot {
                id: "test".into(),
                place: PlaceId(0),
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: 1,
                    h: 1,
                },
                crop: Crop::Cabbages,
                store: None,
                gathered: 0,
                tiles: vec![tile],
            }],
        }
    }

    #[test]
    fn a_tended_crop_ripens_in_about_a_week() {
        let mut f = plot(sown());
        let mut days = 0;
        while f.plots[0].tiles[0].stage != Stage::Ripe {
            f.plots[0].tiles[0].water = WATER_FULL;
            f.plots[0].tiles[0].weeds = 0;
            f.new_day(Season::Spring, false);
            days += 1;
            assert!(days < 30, "never ripened");
        }
        assert_eq!(days, 7, "cabbages need 14 points at 2 a day");
    }

    #[test]
    fn a_dry_crop_does_not_grow_and_wilts_back() {
        let mut t = sown();
        t.growth = 6;
        t.stage = Stage::Growing;
        t.water = 0;
        let mut f = plot(t);
        f.new_day(Season::Summer, false);
        assert_eq!(f.plots[0].tiles[0].growth, 5);
    }

    #[test]
    fn rain_soaks_and_weeds_slow_growth() {
        let mut t = sown();
        t.water = 0;
        t.weeds = WEEDY;
        let mut f = plot(t);
        f.new_day(Season::Spring, true);
        assert_eq!(f.plots[0].tiles[0].water, WATER_FULL);
        assert_eq!(f.plots[0].tiles[0].growth, 1);
    }

    #[test]
    fn an_unharvested_crop_spoils_and_the_ground_goes_fallow() {
        let mut t = sown();
        t.stage = Stage::Ripe;
        t.growth = 14;
        let mut f = plot(t);
        let mut lost = 0;
        for _ in 0..(RIPE_DAYS + SPOIL_DAYS) {
            lost += f
                .new_day(Season::Summer, true)
                .spoiled
                .iter()
                .map(|(_, n)| *n)
                .sum::<u16>();
        }
        assert_eq!(f.plots[0].tiles[0].stage, Stage::Fallow);
        assert_eq!(lost, 1);
    }

    #[test]
    fn work_follows_the_tile_round_its_year() {
        let mut t = FieldTile {
            stage: Stage::Fallow,
            ..sown()
        };
        assert_eq!(wants(&t, Season::Spring), Some(Work::Till));
        assert_eq!(wants(&t, Season::Winter), None);
        assert!(work(&mut t, Crop::Cabbages, Work::Till));
        assert_eq!(wants(&t, Season::Spring), Some(Work::Sow));
        assert!(work(&mut t, Crop::Cabbages, Work::Sow));
        assert!(
            !work(&mut t, Crop::Cabbages, Work::Harvest),
            "nothing to harvest yet"
        );
        t.growth = 14;
        t.stage = Stage::Ripe;
        assert_eq!(wants(&t, Season::Spring), Some(Work::Harvest));
        assert!(work(&mut t, Crop::Cabbages, Work::Harvest));
        assert_eq!(t.stage, Stage::Stubble);
        assert_eq!(t.fertility, FERTILITY_START - 1);
    }

    #[test]
    fn spent_ground_rests_and_recovers() {
        let mut t = FieldTile {
            stage: Stage::Stubble,
            fertility: POOR,
            ..sown()
        };
        assert_eq!(wants(&t, Season::Spring), None, "too poor to till");
        t.stage = Stage::Fallow;
        let mut f = plot(t);
        for _ in 0..FALLOW_DAYS_PER_FERTILITY {
            f.new_day(Season::Spring, true);
        }
        let t = f.plots[0].tiles[0];
        assert_eq!(t.fertility, POOR + 1);
        assert_eq!(wants(&t, Season::Spring), Some(Work::Till));
    }

    #[test]
    fn beans_feed_the_ground() {
        let mut t = FieldTile {
            stage: Stage::Ripe,
            ..sown()
        };
        assert!(work(&mut t, Crop::Beans, Work::Harvest));
        assert_eq!(t.fertility, FERTILITY_START + 1);
    }
}
