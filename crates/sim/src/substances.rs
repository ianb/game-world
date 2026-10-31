//! Substances: what people take for its effect. A dose lifts spirits for some hours and
//! moves the moods it names. A habit deepens with each dose and fades without, and between
//! doses a craving grows at a rate the habit sets. Every portion belongs to a batch, so a
//! bad batch can be followed through the people it reached. The rules are declared in
//! content (`content::SubstanceDef`); design in `docs/worlds/welling/plan.md`.
use crate::ids::VillagerId;
use crate::time::Tick;
use content::SubstanceDef;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Craving at which someone wants a dose and goes for one.
pub const WANTS: u8 = 30;
/// Craving at which it is all they think of, and it lowers their spirits.
pub const DESPERATE: u8 = 70;
pub const MAX: u8 = 100;
/// Hours without a dose for the habit to ease by one.
pub const EASES_EVERY: u16 = 12;

/// The chance label for a batch being bad when it is made.
pub const BAD_BATCH: &str = "bad-batch";
/// The chance label for a portion having an effect nobody planned.
pub const ODD_PORTION: &str = "odd-portion";
/// The chance label for that effect being for the better.
pub const ODD_FOR_THE_BETTER: &str = "odd-for-the-better";

/// One dose working on someone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dose {
    /// Index into the content's substances.
    pub substance: u16,
    pub batch: Option<u32>,
    pub lift: i8,
    pub hours_left: u8,
}

/// Someone's relation to one substance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Habit {
    pub substance: u16,
    /// How hooked they are, 0 to 100.
    pub hooked: u8,
    /// How much they want a dose now, 0 to 100.
    pub craving: u8,
    /// Hours since the last dose.
    pub clean_hours: u16,
}

impl Habit {
    /// 0 below `WANTS`, 1 wanting, 2 desperate.
    #[must_use]
    pub fn level(&self) -> u8 {
        match self.craving {
            c if c >= DESPERATE => 2,
            c if c >= WANTS => 1,
            _ => 0,
        }
    }
}

/// One person's doses and habits.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intake {
    pub doses: Vec<Dose>,
    pub habits: Vec<Habit>,
}

impl Intake {
    /// What their doses do to their spirits now, less one if a craving is desperate.
    #[must_use]
    pub fn lift(&self) -> i8 {
        let doses: i32 = self.doses.iter().map(|d| i32::from(d.lift)).sum();
        let want = i32::from(self.habits.iter().any(|h| h.craving >= DESPERATE));
        i8::try_from((doses - want).clamp(-3, 3)).unwrap_or(0)
    }

    fn dosed(&self, substance: u16) -> bool {
        self.doses.iter().any(|d| d.substance == substance)
    }

    /// An hour passes. Doses wear off; each habit without a dose working eases a little
    /// every `EASES_EVERY` hours, and while awake its craving grows by a fifth of how
    /// hooked they are. Returns the habits whose craving reached a new level, with it.
    pub fn hour(&mut self, awake: bool) -> Vec<(u16, u8)> {
        for d in &mut self.doses {
            d.hours_left = d.hours_left.saturating_sub(1);
        }
        self.doses.retain(|d| d.hours_left > 0);
        let mut crossed = Vec::new();
        let dosed: Vec<bool> = self
            .habits
            .iter()
            .map(|h| self.dosed(h.substance))
            .collect();
        for (h, dosed) in self.habits.iter_mut().zip(dosed) {
            if dosed {
                continue;
            }
            h.clean_hours = h.clean_hours.saturating_add(1);
            if h.clean_hours.is_multiple_of(EASES_EVERY) {
                h.hooked = h.hooked.saturating_sub(1);
            }
            if awake && h.hooked > 0 {
                let was = h.level();
                h.craving = h.craving.saturating_add((h.hooked / 5).max(1)).min(MAX);
                if h.level() > was {
                    crossed.push((h.substance, h.level()));
                }
            }
        }
        crossed
    }

    /// A dose of `substance` (index `index`) with `lift`, from `batch`: it works for the
    /// substance's hours, deepens the habit and stills the craving.
    pub fn take(
        &mut self,
        def: &SubstanceDef,
        index: u16,
        batch: Option<u32>,
        lift: i8,
        hours: u8,
    ) {
        if hours > 0 {
            self.doses.push(Dose {
                substance: index,
                batch,
                lift,
                hours_left: hours,
            });
        }
        let habit = match self.habits.iter_mut().find(|h| h.substance == index) {
            Some(h) => h,
            None if def.habit > 0 => {
                self.habits.push(Habit {
                    substance: index,
                    hooked: 0,
                    craving: 0,
                    clean_hours: 0,
                });
                self.habits.last_mut().expect("just pushed")
            }
            None => return,
        };
        habit.hooked = habit.hooked.saturating_add(def.habit).min(MAX);
        habit.craving = 0;
        habit.clean_hours = 0;
    }
}

/// Portions made together: by whom, when, and whether the batch is bad.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batch {
    pub substance: u16,
    pub made: Tick,
    pub by: Option<VillagerId>,
    pub bad: bool,
}

/// Every batch, and where its portions are. A portion taken off an object is carried by
/// whoever took it until they put it down or take it, so a batch follows the goods from
/// the maker to the stall to the person who uses it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batches {
    /// Indexed by batch id.
    pub made: Vec<Batch>,
    /// Portions on objects, by object and good, oldest batch first, as (batch, count).
    pub on: BTreeMap<(u16, String), Vec<(u32, u16)>>,
    /// Portions in someone's hands, by villager, as (good, batch, count), oldest first.
    pub carried: BTreeMap<u16, Vec<(String, u32, u16)>>,
}

/// How many entries anyone's hands keep; the oldest are dropped past it.
const HANDS: usize = 8;

impl Batches {
    pub fn make(&mut self, substance: u16, made: Tick, by: Option<VillagerId>, bad: bool) -> u32 {
        self.made.push(Batch {
            substance,
            made,
            by,
            bad,
        });
        u32::try_from(self.made.len() - 1).expect("few batches")
    }

    /// Puts `count` portions of batch `batch` on the end of `object`'s `good`.
    pub fn put(&mut self, object: u16, good: &str, batch: u32, count: u16) {
        let q = self.on.entry((object, good.to_string())).or_default();
        match q.last_mut() {
            Some((b, n)) if *b == batch => *n = n.saturating_add(count),
            _ => q.push((batch, count)),
        }
    }

    /// Takes `count` portions off `object`'s `good`, oldest first, into `who`'s hands.
    pub fn take_off(&mut self, object: u16, good: &str, count: u16, who: VillagerId) {
        let key = (object, good.to_string());
        let Some(q) = self.on.get_mut(&key) else {
            return;
        };
        let hands = self.carried.entry(who.0).or_default();
        let mut left = count;
        while left > 0 && !q.is_empty() {
            let n = q[0].1.min(left);
            hands.push((good.to_string(), q[0].0, n));
            q[0].1 -= n;
            left -= n;
            if q[0].1 == 0 {
                q.remove(0);
            }
        }
        if q.is_empty() {
            self.on.remove(&key);
        }
        if hands.len() > HANDS {
            hands.drain(..hands.len() - HANDS);
        }
    }

    /// Takes up to `count` portions of `good` out of `who`'s hands, oldest first, as
    /// (batch, count) lots.
    pub fn from_hands(&mut self, who: VillagerId, good: &str, count: u16) -> Vec<(u32, u16)> {
        let Some(hands) = self.carried.get_mut(&who.0) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut left = count;
        for (g, b, n) in hands.iter_mut() {
            if left == 0 {
                break;
            }
            if g == good && *n > 0 {
                let k = (*n).min(left);
                *n -= k;
                left -= k;
                out.push((*b, k));
            }
        }
        hands.retain(|(_, _, n)| *n > 0);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kelp() -> SubstanceDef {
        SubstanceDef {
            id: "kelp".into(),
            lift: 1,
            hours: 3,
            moods: Vec::new(),
            wakes: 0,
            habit: 3,
            wild: None,
        }
    }

    #[test]
    fn a_dose_lifts_while_it_lasts() {
        let mut i = Intake::default();
        i.take(&kelp(), 0, None, 1, 3);
        assert_eq!(i.lift(), 1);
        i.hour(true);
        i.hour(true);
        assert_eq!(i.lift(), 1);
        i.hour(true);
        assert_eq!(i.lift(), 0, "worn off after three hours");
    }

    #[test]
    fn craving_grows_with_the_habit_and_a_dose_stills_it() {
        let mut i = Intake::default();
        i.habits.push(Habit {
            substance: 0,
            hooked: 50,
            craving: 0,
            clean_hours: 0,
        });
        let crossed: Vec<_> = (0..3).flat_map(|_| i.hour(true)).collect();
        assert_eq!(
            crossed,
            vec![(0, 1)],
            "wants it after three hours at 10 an hour"
        );
        let crossed: Vec<_> = (0..4).flat_map(|_| i.hour(true)).collect();
        assert_eq!(crossed, vec![(0, 2)], "desperate at 70");
        assert_eq!(i.lift(), -1, "desperation lowers spirits");
        i.take(&kelp(), 0, None, 1, 3);
        assert_eq!(i.habits[0].craving, 0);
        assert_eq!(i.habits[0].hooked, 53);
    }

    #[test]
    fn asleep_the_craving_waits_and_clean_the_habit_eases() {
        let mut i = Intake::default();
        i.habits.push(Habit {
            substance: 0,
            hooked: 10,
            craving: 0,
            clean_hours: 0,
        });
        for _ in 0..24 {
            i.hour(false);
        }
        assert_eq!(i.habits[0].craving, 0);
        assert_eq!(i.habits[0].hooked, 8);
    }

    #[test]
    fn a_batch_follows_its_portions_through_hands() {
        let mut b = Batches::default();
        let (who, other) = (VillagerId(0), VillagerId(1));
        let first = b.make(0, Tick::ZERO, Some(who), false);
        let second = b.make(0, Tick::ZERO, Some(who), true);
        b.put(1, "gas", first, 2);
        b.put(1, "gas", second, 2);
        b.take_off(1, "gas", 3, other);
        let lots = b.from_hands(other, "gas", 3);
        assert_eq!(lots, vec![(first, 2), (second, 1)], "oldest first");
        assert_eq!(b.on[&(1, "gas".to_string())], vec![(second, 1)]);
    }
}
