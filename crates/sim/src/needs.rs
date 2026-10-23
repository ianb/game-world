//! Bodies. Four needs per villager rise through the day at a slightly random rate and
//! are met by what the villager is doing: eating, drinking, sleeping, or a visit to the
//! privy. When a need gets pressing the character is asked what to do about it.
use rand::RngExt;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

pub use crate::protocol::{Need, NeedsView};

/// A need at or above this asks the character once; at `URGENT` it asks again.
pub const PRESSING: u8 = 70;
pub const URGENT: u8 = 90;

/// Base rise per hour awake, in need points (0..=100).
pub const RATE_HUNGER: u8 = 6;
pub const RATE_THIRST: u8 = 9;
pub const RATE_SLEEP: u8 = 5;
pub const RATE_BLADDER: u8 = 12;
/// Sleep repaid per hour asleep.
pub const SLEEP_RECOVERY: u8 = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Needs {
    pub hunger: u8,
    pub thirst: u8,
    pub sleep: u8,
    pub bladder: u8,
}

impl Needs {
    /// A villager at midnight on day one: fed, watered and asleep, with some jitter so
    /// nobody's needs march in step.
    pub fn starting(rng: &mut Pcg32) -> Needs {
        Needs {
            hunger: rng.random_range(25..45),
            thirst: rng.random_range(20..40),
            sleep: rng.random_range(10..30),
            bladder: rng.random_range(10..40),
        }
    }

    #[must_use]
    pub fn get(&self, need: Need) -> u8 {
        match need {
            Need::Hunger => self.hunger,
            Need::Thirst => self.thirst,
            Need::Sleep => self.sleep,
            Need::Bladder => self.bladder,
        }
    }

    /// An hour passes. Returns the needs that just crossed a threshold, pressing first.
    pub fn hour(&mut self, asleep: bool, rng: &mut Pcg32) -> Vec<Need> {
        let mut crossed = Vec::new();
        let mut rise = |value: &mut u8, base: u8, need: Need| {
            // A little randomness: anywhere from a slow hour to a hungry one.
            let jitter = rng.random_range(0..=5u8);
            let mut delta = (base + jitter).saturating_sub(2);
            if asleep {
                delta /= 2;
            }
            let before = *value;
            *value = value.saturating_add(delta).min(100);
            // Ask at the two thresholds, then keep asking every hour while urgent, so a
            // need that got urgent overnight is dealt with in the morning.
            let crossing =
                (before < PRESSING && *value >= PRESSING) || (before < URGENT && *value >= URGENT);
            let nagging = *value >= URGENT && need != Need::Sleep;
            if crossing || nagging {
                crossed.push(need);
            }
        };
        rise(&mut self.hunger, RATE_HUNGER, Need::Hunger);
        rise(&mut self.thirst, RATE_THIRST, Need::Thirst);
        rise(&mut self.bladder, RATE_BLADDER, Need::Bladder);
        if asleep {
            self.sleep = self.sleep.saturating_sub(SLEEP_RECOVERY);
        } else {
            rise(&mut self.sleep, RATE_SLEEP, Need::Sleep);
        }
        crossed
    }

    /// What an activity does for the body, applied when the activity starts.
    pub fn on_activity(&mut self, activity: &str) {
        match activity {
            "eat" => {
                self.hunger = 0;
                self.thirst = self.thirst.saturating_sub(30);
            }
            "drink" | "drink-water" => {
                self.thirst = 0;
                self.bladder = self.bladder.saturating_add(10).min(100);
            }
            "relieve" => self.bladder = 0,
            _ => {}
        }
    }

    #[must_use]
    pub fn view(&self) -> NeedsView {
        NeedsView {
            hunger: self.hunger,
            thirst: self.thirst,
            sleep: self.sleep,
            bladder: self.bladder,
        }
    }
}

impl Need {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Need::Hunger => "hungry",
            Need::Thirst => "thirsty",
            Need::Sleep => "sleepy",
            Need::Bladder => "needs the privy",
        }
    }
}
