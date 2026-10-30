//! Trains on the timetable: each day's stops are queued at midnight; a stopping train
//! leaves its deliveries in the shed, and when it leaves it takes what was consigned and
//! pays for it.
use super::Sim;
use crate::brain::CharacterSet;
use crate::event::Event;
use crate::ids::{ObjectId, VillagerId};
use crate::protocol::{Cause, Emitted, GoodsLine, Shipment};
use crate::railway::{Consignment, Timing, head_at, timings};

impl<C: CharacterSet> Sim<C> {
    fn railway_spec(&self) -> Option<&content::RailwaySpec> {
        self.content.zones.iter().find_map(|z| z.railway.as_ref())
    }

    /// Today's trains, in timetable order.
    #[must_use]
    pub fn trains_today(&self) -> Vec<Timing> {
        let (Some(spec), Some(rail)) = (self.railway_spec(), self.world.rail.as_ref()) else {
            return Vec::new();
        };
        let tick = self.state.tick;
        timings(spec, rail, tick.day_start(), tick.calendar().weekday)
    }

    /// The train on the track now, with where its front is, if there is one.
    #[must_use]
    pub fn train_now(&self) -> Option<(Timing, i32)> {
        let rail = self.world.rail.as_ref()?;
        let tick = self.state.tick;
        self.trains_today()
            .into_iter()
            .find_map(|t| head_at(&t, rail, tick).map(|h| (t, h)))
    }

    /// The trains still to come: one on the track now, then every run over the next `days`
    /// days, in order.
    #[must_use]
    pub fn upcoming_trains(&self, days: u32) -> Vec<Timing> {
        let (Some(spec), Some(rail)) = (self.railway_spec(), self.world.rail.as_ref()) else {
            return Vec::new();
        };
        let now = self.state.tick;
        (0..=days)
            .flat_map(|d| {
                let day = now.day_start().plus(d * crate::time::TICKS_PER_DAY);
                timings(spec, rail, day, day.calendar().weekday)
            })
            .filter(|t| t.exit > now)
            .collect()
    }

    /// Queues today's arrivals and departures. Called at the start of each day.
    pub(super) fn schedule_trains(&mut self) {
        let now = self.state.tick;
        for (i, t) in self.trains_today().into_iter().enumerate() {
            let run = u16::try_from(i).expect("few trains");
            if t.arrive >= now {
                self.state
                    .queue
                    .schedule(t.arrive, Event::TrainArrives { run });
            }
            if t.stops && t.depart >= now {
                self.state
                    .queue
                    .schedule(t.depart, Event::TrainDeparts { run });
            }
        }
    }

    pub(super) fn train_arrives(&mut self, run: u16, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let Some(t) = self.trains_today().get(usize::from(run)).copied() else {
            return;
        };
        if !t.stops {
            out.push(Emitted::TrainPassed { tick });
            return;
        }
        let (Some(spec), Some(rail)) = (self.railway_spec(), self.world.rail.as_ref()) else {
            return;
        };
        let shed = u16::try_from(rail.shed).expect("few objects");
        let delivers = spec.delivers.clone();
        let goods = self.state.stock.entry(shed).or_default();
        let mut delivered = Vec::new();
        for d in delivers {
            let n = goods.entry(d.good.clone()).or_insert(0);
            if *n < d.count {
                delivered.push(GoodsLine {
                    good: d.good,
                    count: d.count - *n,
                });
                *n = d.count;
            }
        }
        out.push(Emitted::TrainArrived { tick, delivered });
    }

    pub(super) fn train_departs(&mut self, _run: u16, out: &mut Vec<Emitted>) {
        let tick = self.state.tick;
        let (Some(spec), Some(rail)) = (self.railway_spec(), self.world.rail.as_ref()) else {
            return;
        };
        let platform = u16::try_from(rail.platform).expect("few objects");
        let prices: Vec<(String, u32)> = spec
            .buys
            .iter()
            .map(|b| (b.good.clone(), b.price))
            .collect();
        let consigned = std::mem::take(&mut self.state.railway.consigned);
        let mut shipped = Vec::new();
        for c in consigned {
            let price = prices
                .iter()
                .find(|(g, _)| *g == c.good)
                .map_or(0, |(_, p)| *p);
            let paid = price * u32::from(c.count);
            if let Some(goods) = self.state.stock.get_mut(&platform) {
                if let Some(n) = goods.get_mut(&c.good) {
                    *n = n.saturating_sub(c.count);
                }
                goods.retain(|_, n| *n > 0);
            }
            let v = &mut self.state.villagers[c.who.index()];
            v.coins = v.coins.saturating_add(paid);
            out.push(Emitted::Paid {
                tick,
                who: c.who,
                from: None,
                to: Some(c.who),
                amount: paid,
                what: format!("{} {} by rail", c.count, c.good),
                payer_left: 0,
                cause: Cause::Sim,
            });
            shipped.push(Shipment {
                who: c.who,
                good: c.good,
                count: c.count,
                paid,
            });
        }
        out.push(Emitted::TrainLeft { tick, shipped });
    }

    /// `who` leaves goods from `from` on the platform for the next train.
    pub(super) fn consign(
        &mut self,
        who: VillagerId,
        from: ObjectId,
        good: &str,
        count: u16,
        out: &mut Vec<Emitted>,
    ) {
        let tick = self.state.tick;
        let (Some(spec), Some(rail)) = (self.railway_spec(), self.world.rail.as_ref()) else {
            return;
        };
        if !spec.buys.iter().any(|b| b.good == good) {
            out.push(Emitted::Decided {
                tick,
                who,
                text: format!("the railway does not take {good}"),
            });
            return;
        }
        let platform = u16::try_from(rail.platform).expect("few objects");
        let have = self
            .state
            .stock
            .get(&from.0)
            .and_then(|g| g.get(good))
            .copied()
            .unwrap_or(0);
        let n = have.min(count);
        if n == 0 {
            return;
        }
        if let Some(goods) = self.state.stock.get_mut(&from.0) {
            if let Some(c) = goods.get_mut(good) {
                *c -= n;
            }
            goods.retain(|_, c| *c > 0);
        }
        let on = self.state.stock.entry(platform).or_default();
        let c = on.entry(good.to_string()).or_insert(0);
        *c = c.saturating_add(n);
        self.state.railway.consigned.push(Consignment {
            who,
            good: good.to_string(),
            count: n,
        });
        out.push(Emitted::Consigned {
            tick,
            who,
            good: good.to_string(),
            count: n,
            cause: self.cause.clone(),
        });
    }
}
