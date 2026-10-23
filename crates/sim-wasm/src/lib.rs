//! The wasm facade: one `Sim` handle, typed events out, typed inputs in, bytes for
//! snapshots. The TypeScript side never reaches past this file.
//!
//! Numbers: ticks and ids are `u32`/`u16` so they cross as plain JS numbers. The seed is
//! two `u32` halves to avoid `BigInt`. Hashes cross as hex strings.
use characters::Town;
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

#[wasm_bindgen]
#[derive(Debug)]
pub struct Sim {
    inner: sim::Sim<Town>,
}

#[wasm_bindgen]
impl Sim {
    /// A new town from content JSON (the output of `sim-tools bundle`).
    #[wasm_bindgen(constructor)]
    pub fn new(seed_lo: u32, seed_hi: u32, content_json: &str) -> Result<Sim, JsError> {
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
            inner: sim::Sim::new(seed, content),
        })
    }

    pub fn restore(bytes: &[u8], content_json: &str) -> Result<Sim, JsError> {
        console_error_panic_hook::set_once();
        let content = content::load_str(content_json).map_err(|e| JsError::new(&e.to_string()))?;
        let inner = sim::Sim::restore(bytes, content).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Sim { inner })
    }

    #[must_use]
    pub fn tick(&self) -> u32 {
        self.inner.tick().0
    }

    /// Runs to `tick` and returns the emitted events as an array of tagged objects.
    pub fn step_until(&mut self, tick: u32) -> Result<JsValue, JsError> {
        let events = self
            .inner
            .step_until(Tick(tick))
            .map_err(|e| JsError::new(&e.to_string()))?;
        to_js(&events)
    }

    /// Queues an input object (`{tick, source, kind}`) for its tick.
    pub fn apply_input(&mut self, input: JsValue) -> Result<(), JsError> {
        let input: Input =
            serde_wasm_bindgen::from_value(input).map_err(|e| JsError::new(&e.to_string()))?;
        self.inner
            .apply_input(input)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        self.inner.snapshot()
    }

    /// Combined state hash as 16 hex characters.
    #[must_use]
    pub fn hash(&self) -> String {
        self.inner.hash().hex()
    }

    /// Per-subsystem hashes: tick, rng, queue, world, inputs.
    #[must_use]
    pub fn hash_parts(&self) -> Vec<String> {
        let h = self.inner.hash();
        [h.tick, h.rng, h.queue, h.world, h.inputs]
            .iter()
            .map(|p| format!("{p:016x}"))
            .collect()
    }

    #[must_use]
    pub fn place_ids(&self) -> Vec<String> {
        self.inner
            .world()
            .places
            .iter()
            .map(|p| p.id.clone())
            .collect()
    }

    /// ASCII render of a place with villagers drawn in, or an empty string for an unknown id.
    #[must_use]
    pub fn render_place(&self, id: &str) -> String {
        match self.inner.world().place_index.get(id) {
            Some(&i) => views::render_place_now(&self.inner, i),
            None => String::new(),
        }
    }

    /// The living ground of a place (surfaces row-major), or an error for an unknown id or
    /// an indoor place.
    pub fn terrain(&self, id: &str) -> Result<JsValue, JsError> {
        let world = self.inner.world();
        let Some(&i) = world.place_index.get(id) else {
            return Err(JsError::new("unknown place"));
        };
        to_js(&self.inner.terrain().view(world, i))
    }

    /// Goods on objects as `StockView[]`.
    pub fn economy(&self) -> Result<JsValue, JsError> {
        to_js(&sim::protocol::stock_views(&self.inner))
    }

    #[must_use]
    pub fn where_is_everyone(&self) -> String {
        views::where_is_everyone(&self.inner)
    }

    /// The compiled world (places, objects, doors, roofs) as a `WorldView`.
    pub fn world(&self) -> Result<JsValue, JsError> {
        to_js(&sim::protocol::world_view(&self.inner))
    }

    /// Current villagers as `VillagerView[]`.
    pub fn villagers(&self) -> Result<JsValue, JsError> {
        to_js(&sim::protocol::villager_views(&self.inner))
    }

    /// The clock as a `ClockView`.
    pub fn clock(&self) -> Result<JsValue, JsError> {
        to_js(&sim::protocol::clock_view(&self.inner))
    }
}
