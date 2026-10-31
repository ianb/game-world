//! The wasm facade: one `Sim` handle, typed events out, typed inputs in, bytes for
//! snapshots. The TypeScript side never reaches past this file.
//!
//! Numbers: ticks and ids are `u32`/`u16` so they cross as plain JS numbers. The seed is
//! two `u32` halves to avoid `BigInt`. Hashes cross as hex strings.
//!
//! Every world in the repository is built in; the world id given at construction picks
//! which characters run (`docs/worlds/README.md`).
use sim::history::History;
use sim::input::Input;
use sim::time::Tick;
use sim::views;
use wasm_bindgen::prelude::*;

/// Protocol version asserted by the TypeScript wrapper at construction.
#[wasm_bindgen]
#[must_use]
pub fn protocol_version() -> u32 {
    sim::protocol::PROTOCOL_VERSION
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsError> {
    let ser = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&ser)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// The world ids this build can run.
#[wasm_bindgen]
#[must_use]
pub fn worlds() -> Vec<String> {
    vec!["valley".into()]
}

#[derive(Debug)]
enum World {
    Valley(History<valley::Town>),
}

/// Evaluates `$body` with `$h` bound to the world's history, whichever world it is.
macro_rules! each {
    ($world:expr, $h:ident => $body:expr) => {
        match $world {
            World::Valley($h) => $body,
        }
    };
}

impl World {
    fn new(id: &str, seed: u64, content: content::Content) -> Result<World, JsError> {
        match id {
            "valley" => Ok(World::Valley(History::new(seed, content))),
            _ => Err(JsError::new(&format!("unknown world {id:?}"))),
        }
    }

    fn restore(id: &str, bytes: &[u8], content: content::Content) -> Result<World, JsError> {
        let fail = |e: sim::SimError| JsError::new(&e.to_string());
        match id {
            "valley" => Ok(World::Valley(
                History::restore(bytes, content).map_err(fail)?,
            )),
            _ => Err(JsError::new(&format!("unknown world {id:?}"))),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Sim {
    world: World,
}

#[wasm_bindgen]
impl Sim {
    /// A new run of the world `world` from its content JSON (the output of
    /// `sim-tools --world <id> bundle`).
    #[wasm_bindgen(constructor)]
    pub fn new(
        world: &str,
        seed_lo: u32,
        seed_hi: u32,
        content_json: &str,
    ) -> Result<Sim, JsError> {
        console_error_panic_hook::set_once();
        let content = content::load_str(content_json).map_err(|e| JsError::new(&e.to_string()))?;
        let problems = content::validate(&content);
        if problems
            .iter()
            .any(|p| p.severity == content::validate::Severity::Error)
        {
            let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
            return Err(JsError::new(&text.join("\n")));
        }
        let seed = (u64::from(seed_hi) << 32) | u64::from(seed_lo);
        Ok(Sim {
            world: World::new(world, seed, content)?,
        })
    }

    pub fn restore(world: &str, bytes: &[u8], content_json: &str) -> Result<Sim, JsError> {
        console_error_panic_hook::set_once();
        let content = content::load_str(content_json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Sim {
            world: World::restore(world, bytes, content)?,
        })
    }

    #[must_use]
    pub fn tick(&self) -> u32 {
        each!(&self.world, h => {
            h.sim().tick().0
        })
    }

    /// Runs to `tick` and returns the emitted events as an array of tagged objects.
    pub fn step_until(&mut self, tick: u32) -> Result<JsValue, JsError> {
        let events = each!(&mut self.world, h => h.step_until(Tick(tick)))
            .map_err(|e| JsError::new(&e.to_string()))?;
        to_js(&events)
    }

    /// Queues an input object (`{tick, source, kind}`) for its tick.
    pub fn apply_input(&mut self, input: JsValue) -> Result<(), JsError> {
        each!(&mut self.world, h => {
            let input: Input =
                serde_wasm_bindgen::from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
            h
                .apply_input(input)
                .map_err(|e| JsError::new(&e.to_string()))
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        each!(&self.world, h => {
            h.sim().snapshot()
        })
    }

    /// Combined state hash as 16 hex characters.
    #[must_use]
    pub fn hash(&self) -> String {
        each!(&self.world, h => {
            h.sim().hash().hex()
        })
    }

    /// Per-subsystem hashes: tick, rng, queue, world, inputs.
    #[must_use]
    pub fn hash_parts(&self) -> Vec<String> {
        each!(&self.world, h => {
            let h = h.sim().hash();
            [h.tick, h.rng, h.queue, h.world, h.inputs]
                .iter()
                .map(|p| format!("{p:016x}"))
                .collect()
        })
    }

    #[must_use]
    pub fn place_ids(&self) -> Vec<String> {
        each!(&self.world, h => {
            h
                .sim()
                .world()
                .places
                .iter()
                .map(|p| p.id.clone())
                .collect()
        })
    }

    /// ASCII render of a place with villagers drawn in, or an empty string for an unknown id.
    #[must_use]
    pub fn render_place(&self, id: &str) -> String {
        each!(&self.world, h => {
            match h.sim().world().place_index.get(id) {
                Some(&i) => views::render_place_now(h.sim(), i),
                None => String::new(),
            }
        })
    }

    /// The living ground of a place (surfaces row-major), or an error for an unknown id or
    /// an indoor place.
    pub fn terrain(&self, id: &str) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            let world = h.sim().world();
            let Some(&i) = world.place_index.get(id) else {
                return Err(JsError::new("unknown place"));
            };
            to_js(&h.sim().terrain().view(world, i))
        })
    }

    /// Goods on objects as `StockView[]`.
    pub fn economy(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::stock_views(h.sim()))
        })
    }

    #[must_use]
    pub fn where_is_everyone(&self) -> String {
        each!(&self.world, h => {
            views::where_is_everyone(h.sim())
        })
    }

    /// The compiled world (places, objects, doors, roofs) as a `WorldView`.
    pub fn world(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::world_view(h.sim()))
        })
    }

    /// Current villagers as `VillagerView[]`.
    pub fn villagers(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::villager_views(h.sim()))
        })
    }

    /// Animals as `AnimalView[]`.
    pub fn animals(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::animal_views(h.sim()))
        })
    }

    /// The train on the track now as a `TrainView`, or null.
    pub fn train(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::train_view(h.sim()))
        })
    }

    /// Every field tile as `FieldView[]`, plot by plot.
    pub fn fields(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::field_views(h.sim()))
        })
    }

    /// The trains still to come this week as `TrainTimeView[]`, in order.
    pub fn upcoming_trains(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::upcoming_train_views(h.sim(), 7))
        })
    }

    /// A villager's inner side as a `MindView`, or null for one with no program, by
    /// villager key.
    pub fn mind(&self, key: &str) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            let sim = h.sim();
            let Some(i) = sim.state().villagers.iter().position(|v| v.id == key) else {
                return Err(JsError::new("unknown villager"));
            };
            let who = sim::ids::VillagerId(u16::try_from(i).expect("few villagers"));
            to_js(&sim::protocol::mind_view(sim, who))
        })
    }

    /// Every gesture as a `GestureView[]`: its reach and how warm each villager takes it.
    pub fn gestures(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::gesture_views(h.sim()))
        })
    }

    /// The clock as a `ClockView`.
    pub fn clock(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&sim::protocol::clock_view(h.sim()))
        })
    }

    /// The constants behind the needs, as a `RulesView`.
    pub fn rules(&self) -> Result<JsValue, JsError> {
        to_js(&sim::protocol::rules_view())
    }

    /// Summaries of every finished day, oldest first, as `DaySummary[]`.
    pub fn summaries(&self) -> Result<JsValue, JsError> {
        each!(&self.world, h => {
            to_js(&h.summaries())
        })
    }

    /// The furthest tick ever reached.
    #[must_use]
    pub fn frontier(&self) -> u32 {
        each!(&self.world, h => {
            h.frontier().0
        })
    }

    /// Goes back to `tick` (at or before the frontier) by replaying from the nearest day
    /// snapshot. Inputs are refused until `back_to_now`.
    pub fn rewind(&mut self, tick: u32) -> Result<(), JsError> {
        each!(&mut self.world, h => {
            h
                .rewind(Tick(tick))
                .map_err(|e| JsError::new(&e.to_string()))
        })
    }

    pub fn back_to_now(&mut self) -> Result<(), JsError> {
        each!(&mut self.world, h => {
            h
                .back_to_now()
                .map_err(|e| JsError::new(&e.to_string()))
        })
    }
}
