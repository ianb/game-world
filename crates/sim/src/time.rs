//! Ticks and the calendar. One tick is one game-second. The calendar is arithmetic over the
//! tick, so there is no second clock to drift.
use serde::{Deserialize, Serialize};
use std::fmt;

pub const TICKS_PER_MINUTE: u32 = 60;
pub const TICKS_PER_HOUR: u32 = 60 * TICKS_PER_MINUTE;
pub const TICKS_PER_DAY: u32 = 24 * TICKS_PER_HOUR;
pub const DAYS_PER_SEASON: u32 = 28;
pub const SEASONS_PER_YEAR: u32 = 4;
pub const DAYS_PER_YEAR: u32 = DAYS_PER_SEASON * SEASONS_PER_YEAR;

/// Absolute simulation time in game-seconds since midnight before day 1 of year 1.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Tick(pub u32);

impl Tick {
    pub const ZERO: Tick = Tick(0);

    #[must_use]
    pub fn plus(self, ticks: u32) -> Tick {
        Tick(self.0.checked_add(ticks).expect("tick overflow"))
    }

    #[must_use]
    pub fn minus(self, ticks: u32) -> Tick {
        Tick(self.0.saturating_sub(ticks))
    }

    #[must_use]
    pub fn day_index(self) -> u32 {
        self.0 / TICKS_PER_DAY
    }

    #[must_use]
    pub fn seconds_into_day(self) -> u32 {
        self.0 % TICKS_PER_DAY
    }

    /// Midnight at the start of the day this tick falls in.
    #[must_use]
    pub fn day_start(self) -> Tick {
        Tick(self.day_index() * TICKS_PER_DAY)
    }

    /// Midnight at the start of the next day.
    #[must_use]
    pub fn next_day_start(self) -> Tick {
        self.day_start().plus(TICKS_PER_DAY)
    }

    /// The tick for `hh:mm` on the same day as `self`.
    #[must_use]
    pub fn at_time(self, hour: u32, minute: u32) -> Tick {
        self.day_start()
            .plus(TimeOfDay::new(hour, minute, 0).seconds())
    }

    #[must_use]
    pub fn calendar(self) -> Calendar {
        Calendar::from_tick(self)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const ALL: [Season; 4] = [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    pub const ALL: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        }
    }

    #[must_use]
    pub fn is_weekend(self) -> bool {
        matches!(self, Weekday::Sat | Weekday::Sun)
    }
}

/// Wall-clock time within a day.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl TimeOfDay {
    #[must_use]
    pub fn new(hour: u32, minute: u32, second: u32) -> TimeOfDay {
        assert!(
            hour < 24 && minute < 60 && second < 60,
            "invalid time of day"
        );
        TimeOfDay {
            hour: u8::try_from(hour).expect("hour < 24"),
            minute: u8::try_from(minute).expect("minute < 60"),
            second: u8::try_from(second).expect("second < 60"),
        }
    }

    #[must_use]
    pub fn from_seconds(seconds_into_day: u32) -> TimeOfDay {
        let s = seconds_into_day % TICKS_PER_DAY;
        TimeOfDay::new(
            s / TICKS_PER_HOUR,
            (s % TICKS_PER_HOUR) / TICKS_PER_MINUTE,
            s % 60,
        )
    }

    #[must_use]
    pub fn seconds(self) -> u32 {
        u32::from(self.hour) * TICKS_PER_HOUR
            + u32::from(self.minute) * TICKS_PER_MINUTE
            + u32::from(self.second)
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Everything a schedule might key on, derived from one tick.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Calendar {
    /// Days since the start, from 0.
    pub day_index: u32,
    /// From 1.
    pub year: u32,
    pub season: Season,
    /// From 1 to `DAYS_PER_SEASON`.
    pub day_of_season: u32,
    pub weekday: Weekday,
    pub time: TimeOfDay,
}

impl Calendar {
    #[must_use]
    pub fn from_tick(tick: Tick) -> Calendar {
        let day_index = tick.day_index();
        let day_of_year = day_index % DAYS_PER_YEAR;
        Calendar {
            day_index,
            year: day_index / DAYS_PER_YEAR + 1,
            season: Season::ALL[(day_of_year / DAYS_PER_SEASON) as usize],
            day_of_season: day_of_year % DAYS_PER_SEASON + 1,
            weekday: Weekday::ALL[(day_index % 7) as usize],
            time: TimeOfDay::from_seconds(tick.seconds_into_day()),
        }
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Y{} {} {} ({}) {}",
            self.year,
            self.season.name(),
            self.day_of_season,
            self.weekday.name(),
            self.time
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_zero_is_spring_1_monday_midnight() {
        let c = Tick::ZERO.calendar();
        assert_eq!(c.year, 1);
        assert_eq!(c.season, Season::Spring);
        assert_eq!(c.day_of_season, 1);
        assert_eq!(c.weekday, Weekday::Mon);
        assert_eq!(c.time, TimeOfDay::new(0, 0, 0));
        assert_eq!(c.to_string(), "Y1 Spring 1 (Mon) 00:00");
    }

    #[test]
    fn seasons_and_years_roll() {
        let last_spring = Tick(27 * TICKS_PER_DAY).calendar();
        assert_eq!(
            (last_spring.season, last_spring.day_of_season),
            (Season::Spring, 28)
        );
        let first_summer = Tick(28 * TICKS_PER_DAY).calendar();
        assert_eq!(
            (first_summer.season, first_summer.day_of_season),
            (Season::Summer, 1)
        );
        let year_two = Tick(DAYS_PER_YEAR * TICKS_PER_DAY).calendar();
        assert_eq!(
            (year_two.year, year_two.season, year_two.day_of_season),
            (2, Season::Spring, 1)
        );
    }

    #[test]
    fn weekday_cycles_every_seven_days() {
        assert_eq!(Tick(6 * TICKS_PER_DAY).calendar().weekday, Weekday::Sun);
        assert_eq!(Tick(7 * TICKS_PER_DAY).calendar().weekday, Weekday::Mon);
    }

    #[test]
    fn at_time_and_time_of_day_round_trip() {
        let t = Tick(3 * TICKS_PER_DAY).at_time(7, 15);
        assert_eq!(t.calendar().time.to_string(), "07:15");
        assert_eq!(t.day_index(), 3);
        assert_eq!(t.next_day_start(), Tick(4 * TICKS_PER_DAY));
        for s in [0, 1, 59, 60, 3599, 3600, 86_399] {
            assert_eq!(TimeOfDay::from_seconds(s).seconds(), s);
        }
    }
}
