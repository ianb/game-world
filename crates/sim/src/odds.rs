//! Named chances a test can force. Every roll that decides whether something happens goes
//! through `Odds::chance` with a label, so a test can set up a scene and say "Kit dawdles
//! today" instead of searching for a seed where he does. In a normal run nothing is forced
//! and each call rolls as it would. The forced outcomes live in `SimState`, so a snapshot
//! carries them.
use crate::ids::VillagerId;
use rand::RngExt;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// A forced outcome for `label`, for one villager or (`who: None`) for everyone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Forced {
    pub label: String,
    pub who: Option<VillagerId>,
    pub happens: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Odds {
    forced: Vec<Forced>,
}

impl Odds {
    /// Forces `label` for everyone. A later call for the same label replaces it.
    pub fn force(&mut self, label: &str, happens: bool) {
        self.set(label, None, happens);
    }

    /// Forces `label` for `who` only; this wins over a force for everyone.
    pub fn force_for(&mut self, label: &str, who: VillagerId, happens: bool) {
        self.set(label, Some(who), happens);
    }

    /// Goes back to rolling for `label`, for everyone and every villager.
    pub fn release(&mut self, label: &str) {
        self.forced.retain(|f| f.label != label);
    }

    fn set(&mut self, label: &str, who: Option<VillagerId>, happens: bool) {
        self.forced.retain(|f| !(f.label == label && f.who == who));
        self.forced.push(Forced {
            label: label.to_owned(),
            who,
            happens,
        });
    }

    /// The forced outcome of `label` for `who`, if a test set one.
    #[must_use]
    pub fn forced(&self, label: &str, who: VillagerId) -> Option<bool> {
        let find = |w: Option<VillagerId>| {
            self.forced
                .iter()
                .find(|f| f.label == label && f.who == w)
                .map(|f| f.happens)
        };
        find(Some(who)).or_else(|| find(None))
    }

    /// Whether something with a `percent` chance happens to `who`. A forced outcome is
    /// returned without drawing from `rng`.
    pub fn chance(&self, rng: &mut Pcg32, who: VillagerId, label: &str, percent: i16) -> bool {
        self.chance_in(rng, who, label, 100, i32::from(percent))
    }

    /// A chance that belongs to no villager (a hen laying): only a force for everyone
    /// applies.
    pub fn chance_any(&self, rng: &mut Pcg32, label: &str, percent: i16) -> bool {
        match self
            .forced
            .iter()
            .find(|f| f.label == label && f.who.is_none())
        {
            Some(f) => f.happens,
            None => i32::try_from(rng.random_range(0..100u32)).unwrap_or(0) < i32::from(percent),
        }
    }

    /// `chance` with odds out of `scale` (1000 for per-mille).
    pub fn chance_in(
        &self,
        rng: &mut Pcg32,
        who: VillagerId,
        label: &str,
        scale: u32,
        odds: i32,
    ) -> bool {
        match self.forced(label, who) {
            Some(happens) => happens,
            None => i64::from(rng.random_range(0..scale)) < i64::from(odds),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn forced_outcomes_win_and_do_not_draw() {
        let mut odds = Odds::default();
        let a = VillagerId(0);
        let b = VillagerId(1);
        odds.force("trips", true);
        odds.force_for("trips", b, false);
        let mut rng = Pcg32::seed_from_u64(1);
        let before = rng.clone();
        assert!(odds.chance(&mut rng, a, "trips", 0));
        assert!(!odds.chance(&mut rng, b, "trips", 100));
        assert_eq!(rng, before);
        odds.release("trips");
        assert!(!odds.chance(&mut rng, a, "trips", 0));
        assert!(odds.chance(&mut rng, a, "trips", 100));
    }
}
