# Rust deterministic discrete-event simulation toolkit for a living-town game (researched 2026-10-23)

Scope: the Rust ecosystem and design patterns for a deterministic, integer-tick, discrete-event
simulation core (villagers, schedules, door-graph + grid pathfinding, smart objects, input log as
save/replay, state hashes, serde content, text views, wasm export). Crate versions and release
dates below were pulled from the crates.io API on 2026-10-23 (cited as the crate's crates.io page).
"recent" means downloads in the trailing 90 days as reported by crates.io.

Version table (crates.io, 2026-10-23). Sorted by topic; "stale" flags are my reading of the dates.

| Topic | Crate | Latest | Released | 90-day downloads | Note |
|---|---|---|---|---|---|
| DES queue | `priority-queue` | 2.7.0 | 2025-09-27 | 7.2M | indexed heap, change-priority |
| DES queue | `keyed_priority_queue` | 0.4.2 | 2023-11-15 | 4.4M | stale but widely used |
| DES framework | `nexosim` (ex-`asynchronix`) | 1.0.0 | 2026-02-03 | 5.3k | async actor DES, overkill here |
| DES framework | `desim` | 0.4.0 | 2023-11-25 | 69 | effectively dead |
| DES framework | `sim` | 0.13.1 | 2025-04-26 | 183 | niche |
| RNG | `rand` | 0.10.3 | 2026-09-20 | 543M | 0.10 line since 2026-02 |
| RNG | `rand_pcg` | 0.10.2 | 2026-04-11 | 90M | portable |
| RNG | `rand_xoshiro` | 0.8.1 | 2026-05-16 | 39M | portable |
| RNG | `rand_chacha` | 0.10.0 | 2026-02-02 | 356M | portable; rand's StdRng now uses `chacha20` instead |
| Math | `fixed` | 1.31.0 | 2026-03-20 | 3.0M | MSRV 1.93, no trig built in |
| Math | `ordered-float` | 5.5.0 | 2026-08-21 | 118M | only if floats are kept |
| Hash | `rustc-hash` | 2.1.3 | 2026-07-02 | 262M | FxHasher, unseeded, deterministic |
| Hash | `foldhash` | 0.2.0 | 2025-08-23 | 281M | fixed-seed mode available |
| Hash | `ahash` | 0.8.12 | 2025-05-08 | 172M | explicitly NOT stable across machines/versions |
| Hash | `fxhash` | 0.2.1 | 2017-07-29 | 27M | abandoned; use `rustc-hash` |
| Hash | `xxhash-rust` | 0.8.19 | 2026-09-28 | 39M | xxh3/xxh64, const fn, versions track xxHash releases |
| Hash | `twox-hash` | 2.1.5 | 2026-10-04 | 67M | alternative xxhash impl |
| Hash | `blake3` | 1.8.7 | 2026-08-20 | 54M | cryptographic, portable |
| Hash | `crc32fast` | 1.5.3 | 2026-10-20 | 201M | CRC32 (Factorio-style) |
| Hash | `seahash` | 4.1.0 | 2021-01-16 | 36M | used by bevy_ggrs checksums |
| Hash | `siphasher` | 1.0.4 | 2026-09-25 | 175M | SipHash with explicit keys |
| Arena | `slotmap` | 1.1.1 | 2025-12-06 | 32M | free-list not serialized (issue #104 open) |
| Arena | `thunderdome` | 0.6.1 | 2023-06-25 | 221k | stale |
| Arena | `generational-arena` | 0.2.9 | 2023-05-22 | 1.2M | stale |
| Ids | `typed-index-collections` | 3.5.0 | 2026-01-18 | 2.3M | `TiVec<K, V>` typed-index Vec |
| ECS | `hecs` | 0.11.2 | 2026-10-04 | 169k | alive; serialize via helper traits |
| ECS | `bevy_ecs` | 0.20.0 | 2026-10-08 | 2.3M | standalone possible, heavy |
| ECS | `legion` | 0.4.0 | 2021-02-25 | 17k | dead |
| Grid | `grid` | 1.0.1 | 2026-04-20 | 6.0M | 2D Vec wrapper |
| Grid | `ndarray` | 0.17.2 | 2026-01-10 | 32M | heavier than needed |
| Graph | `petgraph` | 0.8.3 | 2025-09-30 | 128M | optional for door graph |
| Pathfinding | `pathfinding` | 4.16.0 | 2026-09-07 | 596k | A*, Dijkstra, BFS, grid; MSRV 1.88 |
| Pathfinding | `hierarchical_pathfinding` | 0.5.0 | 2021-08-08 | 1.0k | HPA*, stale |
| Pathfinding | `bevy_northstar` | 0.7.0 | 2026-07-08 | n/a | HPA* for Bevy only |
| Maps | `indexmap` | 2.14.2 | 2026-09-05 | 424M | order independent of hasher |
| Serde | `serde` | 1.0.229 | 2026-07-18 | 377M | |
| Serde | `serde_json` | 1.0.154 | 2026-10-11 | 377M | |
| Serde | `ron` | 0.12.2 | 2026-06-22 | 22M | |
| Serde | `toml` | 0.9.8 (1.1.8 pre) | 2025-10-09 | 272M | 1.x prerelease in progress |
| Binary | `postcard` | 1.1.3 | 2025-07-24 | 33M | compact, no_std |
| Binary | `bincode` | 3.0.0 | 2025-12-16 | 72M | ORIGINAL DISCONTINUED; see below |
| Binary | `bincode-next` | 3.1.1 | 2026-06-20 | n/a | community fork (Apich Org) |
| Binary | `rmp-serde` | 1.3.1 | 2025-12-23 | 29M | MessagePack |
| Binary | `bitcode` | 0.6.9 | 2025-12-18 | 3.1M | smallest/fastest in benchmarks |
| Binary | `savefile` | 0.20.6 | 2026-10-19 | 13k | versioned schemas built in |
| Schema | `schemars` | 1.2.3 | 2026-10-12 | 213M | JSON Schema from serde types |
| Validation | `validator` | 0.21.0 | 2026-07-27 | 15M | |
| Validation | `garde` | 0.23.0 | 2026-05-23 | 2.7M | |
| Test | `insta` / `cargo-insta` | 1.49.0 | 2026-10-03 | 34M | |
| Test | `proptest` | 1.12.0 | 2026-10-10 | 62M | |
| Test | `quickcheck` | 1.1.0 | 2026-02-10 | 14M | |
| Test | `rstest` | 0.28.0 | 2026-10-18 | 33M | |
| Test | `criterion` | 0.8.2 | 2026-02-04 | 70M | |
| Test | `divan` | 0.1.21 | 2025-04-10 | 4.3M | lighter bench alternative |
| Test | `cargo-nextest` | 0.9.149 | 2026-10-20 | 1.8M | |
| Test | `datatest-stable` | 0.3.3 | 2025-09-29 | 888k | file-driven tests, nextest-friendly |
| Test | `cargo-mutants` | 27.1.0 | 2026-06-02 | 361k | |
| Test | `pretty_assertions` | 1.4.1 | 2024-09-15 | 43M | |
| Test | `similar-asserts` | 2.0.0 | 2026-04-01 | 6.6M | |
| Fuzz | `cargo-fuzz` / `arbitrary` | 0.13.2 / 1.5.0 | 2026-06-09 / 2026-10-06 | 1.8M / 50M | |
| Text | `comfy-table` | 8.0.1 | 2026-09-25 | 22M | |
| Text | `tabled` | 0.22.0 | 2026-09-05 | 12M | |
| Text | `unicode-width` | 0.2.2 | 2025-10-06 | 197M | |
| Text | `ratatui` | 0.30.2 | 2026-06-19 | 23M | only if a TUI is wanted |
| CLI | `clap` | 4.6.7 | 2026-09-14 | 263M | |
| Log | `tracing` / `tracing-subscriber` | 0.1.44 / 0.3.23 | 2025-12-18 / 2026-03-13 | 233M / 169M | |
| Log | `tracing-test` | 0.2.6 | 2026-02-07 | 12M | |
| Log | `test-log` | 0.2.21 | 2026-05-30 | 11M | |
| Image | `image` | 0.25.10 | 2026-03-10 | 66M | PNG encode, CPU |
| Image | `tiny-skia` | 0.12.0 | 2026-02-02 | 21M | CPU 2D raster |
| Wasm | `wasm-bindgen` | 0.2.129 | 2026-09-25 | 155M | |
| Wasm | `getrandom` | 0.4.3 | 2026-06-17 | 691M | avoid entirely in the sim crate |
| Wasm | `console_error_panic_hook` | 0.1.7 | 2021-10-11 | 18M | stale but tiny and standard |
| Wasm | `wasm-opt` (Rust wrapper) | 0.116.1 | 2024-03-31 | 976k | prefer binaryen binary |
| Rollback | `ggrs` / `bevy_ggrs` | 0.13.0 / 0.22.0 | 2026-06-26 | 15k / 5.4k | SyncTestSession pattern |
| Supply chain | `cargo-deny` | 0.20.2 | 2026-07-09 | 2.0M | |

Sources for the table: crates.io crate pages, e.g. [pathfinding](https://crates.io/crates/pathfinding),
[priority-queue](https://crates.io/crates/priority-queue), [keyed_priority_queue](https://crates.io/crates/keyed_priority_queue),
[nexosim](https://crates.io/crates/nexosim), [rand](https://crates.io/crates/rand), [rand_pcg](https://crates.io/crates/rand_pcg),
[rand_xoshiro](https://crates.io/crates/rand_xoshiro), [fixed](https://crates.io/crates/fixed), [ahash](https://crates.io/crates/ahash),
[rustc-hash](https://crates.io/crates/rustc-hash), [xxhash-rust](https://crates.io/crates/xxhash-rust), [blake3](https://crates.io/crates/blake3),
[slotmap](https://crates.io/crates/slotmap), [hecs](https://crates.io/crates/hecs), [bevy_ecs](https://crates.io/crates/bevy_ecs),
[bincode](https://crates.io/crates/bincode), [bincode-next](https://crates.io/crates/bincode-next), [postcard](https://crates.io/crates/postcard),
[insta](https://crates.io/crates/insta), [proptest](https://crates.io/crates/proptest), [rstest](https://crates.io/crates/rstest),
[wasm-bindgen](https://crates.io/crates/wasm-bindgen), [getrandom](https://crates.io/crates/getrandom), [ggrs](https://crates.io/crates/ggrs).

---

## 1. Discrete-event scheduling in Rust

### Takeaway
Hand-roll the event queue: `std::collections::BinaryHeap<Reverse<(tick, seq, Event)>>` plus a
monotonically increasing `seq` gives deterministic tie-breaking in ~50 lines and is trivially
serde-serializable. The DES frameworks on crates.io are either dead (`desim`) or built for a very
different problem (`nexosim` is an async, multi-threaded actor framework for cyber-physical
systems). Use an indexed queue (`priority-queue`) only if you need O(log n) cancellation by key;
tombstoning (cancelled-flag or generation check on pop) is simpler and keeps the queue a plain heap.

### Cited Findings
- `nexosim` 1.0.0 (2026-02-03) describes itself as "a developer-friendly, highly optimized discrete-event simulation framework written in Rust"; models are actors exchanging async closures via bounded MPSC mailboxes, run by "a custom multi-threaded executor rather than a general-purpose runtime like Tokio"; the 1.0 announcement lists "simulation save/restore"; its README does not describe determinism or wasm support, and its motivation is "simulating large cyberphysical systems", drawing "from experience on spacecraft real-time simulators" — [nexosim README](https://github.com/asynchronics/nexosim); versions from [crates.io](https://crates.io/crates/nexosim). Its predecessor `asynchronix` last released 0.2.4 on 2024-11-16 and has 73 downloads in 90 days — [crates.io](https://crates.io/crates/asynchronix).
- `desim` 0.4.0 was last released 2023-11-25 and has 69 downloads in the trailing 90 days; `sim` 0.13.1 (2025-04-26) has 183 — [desim](https://crates.io/crates/desim), [sim](https://crates.io/crates/sim).
- `keyed_priority_queue` (0.4.2, 2023-11-15): push/pop/remove-by-key/`set_priority` are O(log n), lookup is O(1), an Entry API lets you look up and update with one hash lookup; it "uses HashMap and own implementation of binary heap"; its borrowing iterator is unordered and the consuming iterator yields in decreasing priority; the docs do not mention serde or a pluggable hasher; its only dependency listed is `indexmap ^2.1.0` — [docs.rs](https://docs.rs/keyed_priority_queue/latest/keyed_priority_queue/).
- `priority-queue` 2.7.0 (2025-09-27) is the most-downloaded indexed priority queue (7.2M in 90 days) — [crates.io](https://crates.io/crates/priority-queue).
- The 1.0 generation of `nexosim` depends on `serde` and its example model derives `Serialize`/`Deserialize` — [nexosim README](https://github.com/asynchronics/nexosim).
- OpenTTD's desync doc lists "undefined behaviour such as unstable sort ordering for equal keys" as a known desync cause — [OpenTTD docs/desync.md](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md).

### Inferences
- `std::collections::BinaryHeap` is a max-heap with no ordering guarantee among equal elements, so the key must be a total order that never ties: `(tick: u64, seq: u64)` with `seq` from a counter stored in the sim state (so it survives snapshots). Wrap in `std::cmp::Reverse` for min-first. Sketch:

  ```rust
  #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
  struct Scheduled { tick: Tick, seq: u64, event: Event }   // derive Ord: tick, then seq, then event

  #[derive(Serialize, Deserialize, Default)]
  pub struct Queue { heap: BinaryHeap<Reverse<Scheduled>>, next_seq: u64 }

  impl Queue {
      pub fn schedule(&mut self, tick: Tick, event: Event) -> EventId {
          let seq = self.next_seq; self.next_seq += 1;
          self.heap.push(Reverse(Scheduled { tick, seq, event }));
          EventId(seq)
      }
      pub fn pop_due(&mut self, now: Tick) -> Option<Scheduled> {
          match self.heap.peek() { Some(Reverse(s)) if s.tick <= now => self.heap.pop().map(|r| r.0), _ => None }
      }
  }
  ```
  `Event` must itself derive `Ord` (or you must exclude it from the key with a manual `Ord` impl comparing only `(tick, seq)`); deriving `Ord` on an enum is deterministic (variant order, then fields).
- Cancellation/rescheduling without an indexed heap: keep per-villager (or per-object) a `current_plan_generation: u32`; store the generation inside the event; on pop, drop events whose generation no longer matches. This "lazy deletion" is standard in DES and costs O(1) per cancel. It also keeps the queue a plain `BinaryHeap`, which serde serializes out of the box (`BinaryHeap<T: Serialize + Ord>` is supported by serde; the heap array order after a round-trip may differ, but pop order is identical because the key is a total order).
- Event enum over trait objects: with serde + snapshots + replay, a closed `enum Event { WakeUp(VillagerId), ArriveAt(VillagerId, Tile), ObjectFree(ObjectId), DayRollover, ... }` is the only option that derives `Serialize`/`Ord`/`Hash` and can be hashed into the per-tick state hash. `Box<dyn FnOnce>` closures cannot be serialized or hashed; `typetag`-style trait-object serde adds a registry for little gain with a few dozen variants.
- `keyed_priority_queue` is unsuitable as-is for the deterministic core: its internal `HashMap` uses an unspecified hasher, it has no serde support in its docs, and the crate has not been released since 2023. `priority-queue` is maintained and generic over the hasher but adds a map you must also serialize; only worth it if cancellation volume is large (it will not be: a few dozen villagers, a handful of pending events each).
- Calendar: keep a single `tick: u64` as truth and derive date/weekday/season/year arithmetically (`const TICKS_PER_DAY: u64 = 86_400` for one game-second ticks). Scheduling "tomorrow 06:00" is `day_start(tick) + TICKS_PER_DAY + 6*3600`. No separate clock state means nothing to desync.

### Gaps
- I found no benchmark or writeup comparing `BinaryHeap` vs `priority-queue` for DES workloads in Rust; the choice above rests on asymptotics and serde ergonomics.
- `nexosim`'s determinism guarantees under its multi-threaded executor were not documented on the README; I did not read its source.

---

## 2. Determinism: RNG, collections, math, time, sorting, and lints

### Takeaway
Use a portable PRNG from the `rand_*` family (`rand_pcg` or `rand_xoshiro`), never `StdRng`/`SmallRng`
(explicitly non-portable per the rand book), and keep RNG state in the snapshot. Replace `HashMap`
with `BTreeMap`/`IndexMap` or a fixed-seed hasher (`rustc-hash`, `foldhash` fixed state), never `ahash`
default or `RandomState`. Use integers (and `fixed` if fractions are needed) rather than floats;
basic IEEE ops are deterministic across x86/arm/wasm but transcendental functions and NaN payloads are
not. Enforce it with `clippy.toml` `disallowed-types`/`disallowed-methods` and a "run twice, compare
hashes" test.

### Cited Findings
- rand book, reproducibility: "Non-portable (no reproducibility guarantee): `rand::rngs::StdRng` and `rand::rngs::SmallRng`"; all other public API parts "including PRNGs, distributions, and other stochastic algorithms, are expected to give results that are reproducible across platforms and patch releases"; "`usize` and `isize` are non-portable because their size varies by target"; "Distributions in `rand_distr` that use `f32` or `f64` may not be portable" because of platform-specific transcendental functions; value-breaking changes "are permitted in minor versions but must be well motivated and noted in the CHANGELOG" — [The Rust Rand Book: Reproducibility](https://rust-random.github.io/book/crate-reprod.html).
- rand CHANGELOG: 0.9.0 (2025-01-27) changed `SmallRng::seed_from_u64`, `UniformFloat` upper bound, `shuffle`/`choose`/`sample_floyd` algorithms, and switched `Uniform` to Canon's/Lemire's methods ("breaks value stability"); `Uniform` for `usize` made portable via `UniformUsize`. 0.10.0 (2026-02-08): `StdRng` moved from `rand_chacha` to the `chacha20` crate, "the output remains the same"; `Xoshiro128PlusPlus`/`Xoshiro256PlusPlus` publicly exported; `small_rng` feature removed; `StdRng` no longer `Clone`; 0.10.1 includes a soundness fix (#1763) — [rand CHANGELOG](https://github.com/rust-random/rand/blob/master/CHANGELOG.md).
- rand 0.10.3 features: default = `std`, `std_rng`, `sys_rng`, `thread_rng`; `sys_rng` pulls in `getrandom`; non-default: `chacha`, `log`, `serde`, `simd_support`, `unbiased` — [docs.rs features](https://docs.rs/crate/rand/latest/features).
- `rand_pcg` 0.10.2 (2026-04-11) and `rand_xoshiro` 0.8.1 (2026-05-16) are current and maintained under rust-random/rngs — [rand_pcg](https://crates.io/crates/rand_pcg), [rand_xoshiro](https://crates.io/crates/rand_xoshiro).
- aHash README: aHash "does not have a fixed standard for its output"; "different computers or computers on different versions of the code will observe different hash values"; "aHash is not intended for network use or in applications which persist hashed values"; its default `runtime-rng` feature seeds from the OS via `getrandom` — [aHash README](https://github.com/tkaitchuck/aHash/blob/master/README.md).
- IndexMap: "a hash table where the iteration order of the key-value pairs is independent of the hash values of the keys"; the `serde` feature adds `Serialize`/`Deserialize`, and `map::serde_seq` serializes as an ordered sequence — [docs.rs indexmap](https://docs.rs/indexmap/latest/indexmap/).
- Egregoria determinism issue: recommends `BTreeMap`/`BTreeSet` over `HashMap` "at a modest runtime cost"; warns `indexmap::IndexMap` is not guaranteed to keep order through serialization; recommends systems run sequentially and dependencies be kept minimal "so they can be audited for nondeterminism"; recommends "hashing instead of RNG to avoid order dependencies" — [Egregoria issue #90](https://github.com/uriopass/egregoria/issues/90).
- Qiskit's SabreSwap fix: summing floating-point distances while iterating a `HashMap` depended on iteration order; fix was insertion-ordered `IndexMap` — [commit](https://cdn05042023.gitlink.org.cn/scnc/qiskit/commit/48cfab601e16517eca1a5249a45cc43726bf7d61).
- Rust `f32` docs: Rust "does not currently guarantee that the bit patterns of NaN are preserved over arithmetic operations, and they are not guaranteed to be portable or even fully deterministic"; running the same NaN-producing operation twice can yield different bit patterns — [core::primitive::f32 docs](https://doc.rust-lang.org/nightly/core/primitive.f32.html) (via kernel.org mirror in search).
- WebAssembly spec (as quoted on LLVM Discourse): for most float operators the NaN sign is nondeterministic and the payload is canonical only if all inputs were canonical — [LLVM Discourse](https://discourse.llvm.org/t/semantics-of-nan/66729?page=2). Wasmtime's fuzzing harness masks NaN payload bits to zero to canonicalize — [wasmtime_fuzzing source](https://docs.wasmtime.dev/api/src/wasmtime_fuzzing/generators/single_inst_module.rs.html).
- `alice-det-math` crate docs claim IEEE 754 `+ - * / sqrt` are bit-identical across SSE2, aarch64 and wasm32, while libm `sin/exp/ln` differ in the last ulp and `mul_add` fuses on FMA hardware (library documentation, not an authoritative source) — [docs.rs alice-det-math](https://docs.rs/alice-det-math).
- Rapier: "locally deterministic by default" but may diverge across computers; cross-platform determinism is achieved mainly by using nalgebra's `ComplexField::sin` rather than `f64::sin`; the wasm/JS build "is fully cross-platform deterministic across browsers, OSes, and processors", confirmed by matching MD5 hashes of world snapshots — [rapier determinism guide](https://rapier.rs/docs/user_guides/rust/determinism/).
- bugnet.io desync guide: causes are floating point (FMA, fast-math), uninitialized memory, hash iteration order, wall-clock and locale, RNG misuse ("use a separate RNG for cosmetic effects"), pointer-based sorting ("sort by a stable key such as entity ID"), thread scheduling; recommends fixed-point (their example: 64-bit with 32 fractional bits, 128-bit intermediate on multiply) and a CI job that replays recorded sessions on multiple platforms — [bugnet.io](https://bugnet.io/blog/how-to-debug-desync-in-deterministic-lockstep-games).
- `fixed` 1.31.0 (2026-03-20): types `FixedI<n>/FixedU<n>` with `Frac` parameter and aliases like `I20F12`, `I32F32`; `sqrt` available; "No algebraic functions (such as `pow`), trigonometric functions, or other transcendental functions" (points to the `cordic` crate); optional `serde` feature; `no_std`; MSRV 1.93 — [docs.rs fixed](https://docs.rs/fixed/latest/fixed/).
- Clippy config: `disallowed-methods`, `disallowed-types`, `disallowed-macros` each take a list of entries with `path` (required), `reason`, `replacement`, and `allow-invalid` (default false; when true "it will ignore this entry if the path doesn't exist, instead of emitting an error") — [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html). The `disallowed_methods` docs use `std::time::Instant::now` as their example of a banned call; `disallowed_types` uses `BTreeMap` as the example — search summaries of [Klocwork mirror of disallowed_methods](https://help.klocwork.com/2026.1/en-us/reference/rs.clippy.disallowed_methods.htm) and [disallowed_types](https://help.klocwork.com/2026.1/en-us/reference/rs.clippy.disallowed_types.htm).
- Real-world `clippy.toml` example (schneems, 2025-11-19): entries like `{ path = "std::fs::canonicalize", replacement = "fs_err::canonicalize" }` and a `path = "std::env::set_current_dir"` entry with a `reason`; CI runs `cargo clippy --all-targets --all-features -- --deny warnings`; gotchas: rules "won't block your `cargo build`", usage in tests slips through without `--all-targets`, escape hatch is `#[allow(clippy::disallowed_methods)]` — [schneems.com](https://schneems.com/2025/11/19/find-accidental-code-usage-with-a-custom-clippytoml/). Zed's repo uses the same mechanism to ban `std::process::Command::spawn/output/status` — [zed clippy.toml](https://git.secluded.site/zed/blob/linux-ci/clippy.toml).
- Nautilus Trader DST docs: AHash "randomizes per process", so insertion-order iteration is needed where order affects output — [nautilustrader.io](https://nautilustrader.io/docs/latest/concepts/dst/).
- `renew` engine (Rust, 0.1.1, updated 2026-09-21): "bit-deterministic", fixed-timestep loop in Q47.16 fixed point; CI replays on five targets and "a digest mismatch fails the build" — [renew-engine org](https://github.com/renew-engine).
- Orion (Rust RTS): Q16.16 fixed point, input-only mutation, FNV checksums every 24 ticks, "Two peers stepping the same commands are bit-identical" — [orion wiki v0.1.0](https://github.com/wachtelhund/orion/wiki/v0.1.0).

### Inferences
- RNG recipe: `rand_pcg::Pcg32` (64-bit state, 32-bit output, 16 bytes serialized) or `rand_xoshiro::Xoshiro256PlusPlus` with the `serde` feature; store one RNG per subsystem (e.g. `schedule_jitter`, `idle_choice`) inside the sim state so that adding a cosmetic random call cannot shift gameplay streams, mirroring bugnet's "separate RNG for cosmetic effects". Seed via `SeedableRng::seed_from_u64(seed)` on the portable RNG (not on `SmallRng`, whose `seed_from_u64` changed in 0.9). Depend on `rand = { version = "0.10", default-features = false }` so `sys_rng`/`thread_rng` (and `getrandom`) are not even compiled into the sim crate; this also avoids the wasm `getrandom` backend dance in Section 9.
- Range sampling: `Rng::random_range(0..n)` on `u32`/`u64` is portable per the book; avoid `usize` ranges (cast to `u32` first) and avoid all `rand_distr` float distributions. For "pick one of N options" use `u32` indexes.
- Collections: `BTreeMap<VillagerId, _>` for anything iterated during simulation; `Vec` indexed by typed ids for hot paths; `IndexMap` only when insertion order is semantically wanted (and remember `swap_remove` reorders; use `shift_remove` or avoid removal). If a hash map is unavoidable, `HashMap<K, V, rustc_hash::FxBuildHasher>` is unseeded and deterministic across runs and platforms, but iteration order still depends on insertion history and capacity, so sort before any order-sensitive use. Never iterate a hash map while mutating sim state or summing anything.
- Math: ticks, tile coordinates, durations, and costs are all integers in this game; path costs are integer tile counts (or 10/14 for 4/8-direction weights). Fixed point is only needed if sub-tile movement interpolation is simulated rather than rendered; if so `fixed::types::I32F32` or `I16F16` with `serde` feature, and `cordic` for trig. Keep `f32` strictly in the renderer/TS side.
- Time: the sim exposes `step(tick_inputs)` and never calls `std::time::Instant::now`/`SystemTime::now`; tooling binaries may time themselves outside the sim crate. Enforce with the `clippy.toml` below at the sim crate root (clippy.toml is per-package/dir, so the `tools` crate can have a laxer one):

  ```toml
  # sim/clippy.toml
  disallowed-types = [
    { path = "std::collections::HashMap", reason = "unordered iteration; use BTreeMap, IndexMap or FxHashMap + sort" },
    { path = "std::collections::HashSet", reason = "unordered iteration; use BTreeSet" },
    { path = "std::collections::hash_map::RandomState", reason = "randomly seeded" },
    { path = "std::time::Instant", reason = "wall clock; sim time is `Tick`" },
    { path = "std::time::SystemTime", reason = "wall clock" },
    { path = "rand::rngs::StdRng", reason = "non-portable per rand book", allow-invalid = true },
    { path = "rand::rngs::SmallRng", reason = "non-portable per rand book", allow-invalid = true },
    { path = "ahash::AHashMap", reason = "per-process random seed", allow-invalid = true },
  ]
  disallowed-methods = [
    { path = "std::time::Instant::now", reason = "wall clock" },
    { path = "std::time::SystemTime::now", reason = "wall clock" },
    { path = "rand::rng", reason = "ambient randomness (thread_rng)", allow-invalid = true },
    { path = "rand::random", reason = "ambient randomness", allow-invalid = true },
    { path = "core::slice::sort_unstable_by", reason = "unstable sort with non-total key can desync; use sort_by with a total key", allow-invalid = true },
  ]
  ```
  Use `allow-invalid = true` on paths from optional dependencies so clippy does not error when the crate is not in the graph (the field exists for exactly this). These lints are in the default `style` group, so `cargo clippy --all-targets -- -D warnings` fails CI. Floats cannot be banned by clippy config; a grep-based test (`rg -n 'f32|f64' sim/src` must return nothing outside an allowlist) is the pragmatic guard, and `#![deny(clippy::float_arithmetic)]` (a `restriction` lint) at the crate root catches any arithmetic on floats that does appear.
- Sorting: `sort_unstable` is deterministic for a total order (same input, same output) but equal keys may land in any order, so any sort key that can tie needs a tiebreaker (`(priority, id)`); `sort_by_key` stable sort preserves input order for ties, which is fine only if the input order is itself deterministic. Ban `sort_by(|a, b| a.partial_cmp(b).unwrap())` patterns by banning floats.
- Catching nondeterminism: (1) a unit test that constructs two sims from the same seed+content, feeds the same input log, and asserts equal per-tick hashes for N days (cheap, run always); (2) a test that snapshots at tick t, restores into a fresh sim, runs both to t+k and compares hashes (catches "state not in snapshot" bugs, OpenTTD's "incomplete savegame" class); (3) CI matrix running (1) on x86_64 Linux, aarch64 (macOS runner) and wasm32 (via `wasm-bindgen-test` or a node runner) and diffing the hash log as an artifact, which is the renew engine's five-target digest check. `loom` is for concurrent memory-model exploration and is irrelevant to a single-threaded sim. `cargo-fuzz` with `arbitrary`-derived input logs is useful for crash-finding and for invariant checks (no two villagers in one tile), not for determinism per se.

### Gaps
- I did not verify that `path = "std::collections::HashMap"` is accepted as-is by current clippy (type aliases vs the underlying `std::collections::hash::map::HashMap` path); the `allow-invalid` field exists so a wrong path is caught at lint time, and the Klocwork mirror documents `disallowed_types` with std collection examples.
- No source quantified the BTreeMap-vs-FxHashMap cost for maps of a few dozen entries; at this scale it is immaterial.
- The rand book does not name `rand_pcg`/`rand_xoshiro` as portable individually; portability follows from its "everything except StdRng/SmallRng" policy and these crates' own test vectors.

---

## 3. Hashing state for desync detection

### Takeaway
Hash a canonical byte serialization (postcard/bitcode of the state) or implement a manual
`StateHash` trait that feeds fields in a fixed order into a stable, non-cryptographic 64-bit hasher
(`xxhash-rust` xxh3 or `rustc-hash`), never `ahash` or `std::hash::DefaultHasher`/`RandomState`.
Keep per-subsystem hashes (villagers, objects, queue, RNG, calendar) and a combined hash, record them
every N ticks in the input log, and bisect by subsystem first, following Factorio and bugnet's method.

### Cited Findings
- Factorio FFF #47: a special mode "makes a CRC from the whole map every tick" and replay compares, "since this is the first desynchronization tick the difference is usually very small, often a single variable" — [FFF #47](https://www.factorio.com/blog/post/fff-47). FFF #55: multiplayer uses a lighter checksum covering "player states, active entity counts, and production statistics", compared "for all the players in the game for every tick (with a communication latency)"; a special build adds human-readable tags to saves to locate the divergent code — [FFF #55](https://factorio.com/blog/post/fff-55). Players use the `/heavy-mode` debugging tool to isolate mod desyncs — [Factorio forums](https://forums.factorio.com/viewtopic.php?p=686391).
- OpenTTD: checks a checksum of the game-logic RNG state at network frames; a mismatch disconnects with a Desync error, but "because the check only catches a desync after it has grown large enough to alter the RNG, it cannot show when the desync began"; `-d desync=2` recomputes cached values every tick and logs differences, `desync=3` also saves periodic `dmp_cmds_*.sav`; replay compares checksums and triggers `NOT_REACHED` on mismatch; savegames can be exported to JSON and diffed; known causes: incomplete savegame, stale caches, unstable sort ordering for equal keys, architecture-dependent results, and commands whose test-run mutates state — [OpenTTD docs/desync.md](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md).
- bugnet.io: hash all mutable sim state every tick (positions, health, state ids, cooldowns, RNG state, tick number); "CRC32 or a simple XOR hash is fast enough", their example is FNV-1a from `0x811c9dc5`; hash subsystems separately and find "the subsystem that diverged first" because divergence cascades (physics error at tick 4,712 shows up as a combat divergence at 4,720); also hash each tick's inputs so simulation bugs can be told from input-delivery bugs; state dumps as one line per variable so the first differing line in a diff marks the origin — [bugnet.io](https://bugnet.io/blog/how-to-debug-desync-in-deterministic-lockstep-games).
- bevy_ggrs ships `ChecksumPlugin`, `ComponentChecksumPlugin`, `ChecksumFlag`, `ChecksumPart` and a `checksum_hasher` built on seahash "for portable checksums"; `RollbackOrdered` keeps "a stable, insertion-ordered list of rollback IDs" used for deterministic checksums — [docs.rs bevy_ggrs](https://docs.rs/bevy_ggrs/). ggrs `SyncTestSession` "simulates a rollback every frame and resimulates the last n states" and compares checksums — [docs.rs ggrs](https://docs.rs/ggrs/0.10.2/ggrs).
- aHash explicitly: "different computers or computers on different versions of the code will observe different hash values" — [aHash README](https://github.com/tkaitchuck/aHash/blob/master/README.md).
- `xxhash-rust` 0.8.19: features `xxh32`, `xxh64`, `xxh3`, `const_xxh3` (all off by default); "Version corresponds to xxHash releases"; wasm SIMD128 via `-Ctarget-feature=+simd128` — [docs.rs xxhash-rust](https://docs.rs/xxhash-rust/latest/xxhash_rust/).
- Tickwise 0.2.x (early, pre-1.0): records `.rec` files with inputs and per-tick hashes, uses a "light-hash" per tick confirmed by a "full hash" later, `tickwise compare` finds the first divergent tick and hash kind, `tickwise diff` does a structural diff of two `.dump` files classifying differences as `Structural`, `Exact` or `SubEpsilonFloat`; a "self-check" replays one recording twice — [docs.rs tickwise](https://docs.rs/crate/tickwise/0.1.0).
- Orion checksums every 24 ticks with FNV — [orion wiki](https://github.com/wachtelhund/orion/wiki/v0.1.0); renew fails CI on a digest mismatch across five targets — [renew-engine](https://github.com/renew-engine).

### Inferences
- Two viable strategies. (a) Serialize-then-hash: `postcard::to_allocvec(&state)` then `xxh3_64(&bytes)`; simplest, guaranteed to cover every field that is in the snapshot (and therefore doubles as a "snapshot completeness" check), but costs a full serialization per hash; for a few dozen villagers that is microseconds (Section 6 benchmark: postcard serializes a ~700 KB log dataset in ~415 µs), so hashing every tick is affordable and hashing every 60 ticks is free. (b) Manual `trait StateHash { fn hash_into(&self, h: &mut impl Hasher); }` implemented per struct in declared field order; faster and lets you skip derived caches, but every new field is a chance to forget one. A hybrid is to use (a) for the periodic "full hash" and a per-subsystem (b) for the "light hash", which is exactly Tickwise's design.
- Do not derive `std::hash::Hash` and feed `DefaultHasher`: `DefaultHasher::new()` is SipHash with fixed keys today but std documents its algorithm as unspecified and subject to change between Rust versions, so stored hashes would go stale across toolchain upgrades. `rustc_hash::FxHasher` (unseeded) and `xxhash_rust::xxh3::Xxh3` (which implements `core::hash::Hasher`, seedable) are stable algorithms with fixed output; `blake3` is the choice if hashes are ever used as proof (e.g. server-validated replays, as in `caputchin-replay-rs`), at the cost of being slower than xxh3 on tiny inputs. `crc32fast` works for Factorio-style CRCs but a 32-bit space makes accidental collisions across millions of ticks plausible; prefer 64 bits.
- Floats: there should be none in sim state (Section 2). If any remain, hash `f32::to_bits()` after canonicalizing (`if x.is_nan() { f32::NAN.to_bits() } else if x == 0.0 { 0 } else { x.to_bits() }`) so -0.0/+0.0 and NaN payloads do not create false desyncs.
- Hash-derive strategy that survives serde round-trips: hash only what is serialized, in the same order serde emits it, and exclude anything rebuilt on load (`#[serde(skip)]` caches, door-distance tables derived from content). Strategy (a) above enforces this by construction. Add a test: `hash(state) == hash(deserialize(serialize(state)))` at several ticks.
- Per-subsystem layout for this game: `calendar` (tick only), `rng` (each stream), `queue` (the heap contents sorted by `(tick, seq)`, plus `next_seq`), `villagers` (position, current activity, path index, needs), `objects` (occupancy), `input_log_cursor`. Record `[tick, combined, sub[..]]` every N ticks into the replay file; on mismatch in a replay, the first differing subsystem at the first differing tick is the lead, per bugnet's method.

### Gaps
- `xxhash-rust` docs do not state a test-vector guarantee against the C reference; "version corresponds to xxHash releases" is the only stability statement found. `twox-hash` is an alternative implementation if cross-checking is wanted.
- I did not find Rust-specific numbers for xxh3 vs FxHash vs blake3 on small (sub-KB) inputs.

---

## 4. Pathfinding: `pathfinding` crate, grids, hierarchy, caching, flow fields

### Takeaway
`pathfinding` 4.16.0 (2026-09-07, MSRV 1.88) is the maintained general-purpose crate: `astar`,
`dijkstra`, `bfs` are generic over a successors closure, so they work on both the door graph and the
tile grid with no adapter types. Its tie-breaking is unspecified, so deterministic ordering comes
from your successors closure returning neighbours in a fixed order and from integer costs. The
hierarchical layer (door graph with precomputed door-to-door distances, then grid A* inside a place)
is small enough to hand-roll; `hierarchical_pathfinding` (2021) is stale and `bevy_northstar` is Bevy-only.

### Cited Findings
- `pathfinding::directed::astar::astar` signature: `pub fn astar<N, C, FN, IN, FH, FS>(start: &N, successors: FN, heuristic: FH, success: FS) -> Option<(Vec<N>, C)> where N: Eq + Hash + Clone, C: Zero + Ord + Copy, FN: FnMut(&N) -> IN, IN: IntoIterator<Item = (N, C)>, FH: FnMut(&N) -> C, FS: FnMut(&N) -> bool`; costs "must be non-negative"; heuristic "must not be greater than the real cost, or a wrong shortest path may be returned"; the returned path includes start and end; the docs do not say which path is returned when several share the same cost — [docs.rs astar](https://docs.rs/pathfinding/latest/pathfinding/directed/astar/fn.astar.html).
- The crate's modules: `directed` (A*, BFS, bidirectional, DFS, Dijkstra, Edmonds-Karp, Floyd, Fringe, IDA*, IDDFS, path counting, SCC, topological sort, Yen), `undirected` (connected components, Kruskal, Prim, cliques), `grid` ("Rectangular grid in which vertices can be added or removed, with or without diagonal links"), `matrix`, `kuhn_munkres`; edge weights must implement `Ord` (wrap floats with `ordered-float`); MSRV 1.88.0; dev-deps include `criterion` and `iai-callgrind` — [docs.rs pathfinding](https://docs.rs/pathfinding/latest/pathfinding/).
- `pathfinding` has 596k downloads in 90 days and released 4.16.0 on 2026-09-07 — [crates.io](https://crates.io/crates/pathfinding).
- `hierarchical_pathfinding` 0.5.0 (2021-08-08) implements HPA* and Hierarchical Dijkstra, "always correct: a path is found if and only if it exists", "paths are slightly worse than optimal", chunk size configurable — [docs.rs](https://docs.rs/crate/hierarchical_pathfinding/latest); 1.0k downloads in 90 days — [crates.io](https://crates.io/crates/hierarchical_pathfinding).
- `bevy_northstar` 0.7.0 (2026-07-08) offers HPA*, Any-Angle HPA* and Theta*; has a `parallel` feature on by default "which can be disabled for single-threaded grid builds, for example on WASM" — [docs.rs bevy_northstar](https://docs.rs/bevy_northstar/); version from [crates.io](https://crates.io/crates/bevy_northstar).
- HPA* paper (Botea, Müller, Schaeffer): "up to a 10-fold speed improvement in exchange for a 1% degradation in path quality" on BioWare maps — [paper PDF](https://webdocs.cs.ualberta.ca/~mmueller/ps/hpastar.pdf).
- `grid` 1.0.1 (2026-04-20) is a plain 2D `Vec` wrapper, 6.0M downloads/90d; `ndarray` 0.17.2 — [grid](https://crates.io/crates/grid), [ndarray](https://crates.io/crates/ndarray). `petgraph` 0.8.3 (2025-09-30) — [crates.io](https://crates.io/crates/petgraph).

### Inferences
- Determinism with `pathfinding::astar`: the crate uses a `BinaryHeap` and a hash-based `indexmap`-style parent table internally (not verified against source for 4.16, see Gaps), but the output depends only on the order in which your `successors` closure yields neighbours and on cost ties. Yield neighbours in a fixed order (e.g. N, E, S, W) and make `N` a plain `(u16, u16)` or `TileId(u32)`; with integer costs and a consistent heuristic (Manhattan for 4-connected) the result is a function of inputs only. To make equal-cost paths visually stable across content edits, add a tiny deterministic tiebreak into the cost (e.g. cost in units of 10 with +1 for turns) or post-process.
- Hierarchical design that fits "places joined by doors": (1) content load builds, per place, a `Grid` of walkable tiles and runs BFS from every door to every other door in that place once (`pathfinding::directed::bfs` or `dijkstra_all`), producing a `door_dist: BTreeMap<(DoorId, DoorId), u32>` per place; (2) a door graph whose nodes are `DoorId` and edges are (a) same-place door-to-door with the precomputed distance and (b) the door's own crossing edge to its twin door in the adjacent place (cost 1); (3) a trip from tile A in place P to tile B in place Q = BFS from A to each door of P (one multi-source BFS gives all distances), `dijkstra` over the door graph, BFS from B to each door of Q, pick the minimum sum; (4) then grid A* only within the current place, segment by segment, as the villager walks. With a few dozen places and a few hundred doors, the door Dijkstra is microseconds; cache the door-to-door all-pairs matrix (Floyd-Warshall via `pathfinding::directed::floyd_warshall` or repeated Dijkstra) at content-load time and store it as derived data (`#[serde(skip)]`, rebuilt on load) so it never enters the snapshot or the hash. This matrix is also directly the "travel-time matrix" text view.
- Grid sizes of a few thousand tiles per place: plain A* on a `Vec<u8>` walkability grid with `(x, y)` nodes is sub-millisecond per query in release builds on this scale; hierarchical grid methods (HPA*) are for maps of 10^5+ tiles and are unnecessary inside a single place. The door graph is the hierarchy.
- Flow fields (one Dijkstra from the destination over the grid, then every agent descends the gradient) suit "many agents, few destinations" (festival square, church bell) and are deterministic by construction; for a few dozen villagers with individual destinations, per-agent A* is simpler. Keep flow fields as an optional optimization for crowd events.
- Collision between villagers: if "no two in one tile" is a required invariant, resolve at the tick level with a deterministic priority (lower `VillagerId` moves first, the other waits one tick or re-plans), and property-test it (Section 7).

### Gaps
- I did not read `pathfinding` 4.16 source to confirm its internal data structures; its docs do not discuss tie-breaking or determinism at all.
- No Rust-specific benchmark of `pathfinding::astar` on ~5k-tile grids was found; the "sub-millisecond" statement is an estimate from algorithmic cost, not a measurement.

---

## 5. Entities and state: arenas, typed ids, or ECS

### Takeaway
For a few dozen villagers with schedule-driven logic, snapshot-and-replay, and per-tick hashing,
plain structs in `Vec`s indexed by typed-id newtypes (or `typed-index-collections::TiVec`) beat both
generational arenas and ECS: the whole state is one `#[derive(Serialize, Deserialize)]` struct,
iteration order is index order, and nothing hides allocator state. `slotmap` has an open bug where
the free list is not serialized, which breaks deterministic replay after a snapshot; `hecs` needs
purpose-built serialization helpers; `bevy_ecs` standalone is heavy; `legion` is dead.

### Cited Findings
- `slotmap` issue #104 (opened July 2023, still open, "accepted" label, moved to the 2.0 milestone in December 2025): "SlotMap's free list isn't preserved across serialization. Running operations, serializing, deserializing, and continuing produces different keys than running the same operations without the round trip"; maintainer orlp recommended forking for now; Uriopass published `slotmapd` and had to add deserialization validation because "accepting an arbitrary serialized free list without checks could allow out-of-bounds memory access" — [slotmap #104](https://github.com/orlp/slotmap/issues/104).
- Egregoria (Aug 2023) removed `hecs` "in favor of simple 'entity lists' for the map ... This allows me to get a better grip on determinism and serialization. The tradeoff is less cache-friendly iteration"; it also noted hecs did not serialize entity state (Ralith/hecs#332) — [Egregoria #90](https://github.com/uriopass/egregoria/issues/90).
- `hecs` 0.11.2 (2026-10-04) `serialize` module: "Components are not necessarily serializable, so we cannot directly implement" `Serialize` for `World`; helpers "allow serialization and deserialization based on purpose-defined traits"; `column` ("Fast column-major") and `row` ("Human-friendly row-major") submodules behind `column-serialize`/`row-serialize` features — [docs.rs hecs::serialize](https://docs.rs/hecs/latest/hecs/serialize/index.html); version from [crates.io](https://crates.io/crates/hecs).
- `bevy_ecs` 0.20.0 (2026-10-08), 2.3M downloads/90d; `legion` 0.4.0 last released 2021-02-25 — [bevy_ecs](https://crates.io/crates/bevy_ecs), [legion](https://crates.io/crates/legion).
- `thunderdome` 0.6.1 (2023-06-25) and `generational-arena` 0.2.9 (2023-05-22) have had no releases in over three years — [thunderdome](https://crates.io/crates/thunderdome), [generational-arena](https://crates.io/crates/generational-arena). `typed-index-collections` 3.5.0 (2026-01-18, 4.0.0-alpha.2 in progress) — [crates.io](https://crates.io/crates/typed-index-collections).
- An AI-generated-Rust retrospective reports agents "tend to over-engineer for flexibility" and recommends "concrete types over generics unless the abstraction is strictly necessary" (anonymous blog, treat as anecdote) — [aiindigo.com](https://aiindigo.com/blog/ai-driven-development-lessons-from-100k-lines-of-ai-generated-rust).

### Inferences
- Layout: content ids (`PlaceId`, `DoorId`, `ObjectKindId`, `VillagerId`) are dense `u16`/`u32` newtypes assigned at content-load in a sorted, documented order (sort by string id), so the same content files always produce the same numeric ids and the same `Vec` order. Runtime entities are not created or destroyed in this design (villagers are fixed by content; objects are fixed by content), so no arena or generation counter is needed at all. If dynamic entities appear later (dropped items, visitors), a `Vec<Option<T>>` with an explicit, serialized `free: Vec<u32>` free list and a `generation: u32` per slot is ~40 lines and avoids the slotmap #104 class of bug by making the allocator state part of the snapshot.
- A `#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)] pub struct VillagerId(pub u16);` plus `impl Index<VillagerId> for Vec<Villager>` (or `TiVec<VillagerId, Villager>`) gives type-safety without an ECS. `serde(transparent)` keeps ids compact in files.
- ECS mismatch: ECS shines when many systems iterate many entities each frame; this sim is event-driven (most villagers are idle between scheduled events) and the logic is "what does villager X do at this event". An ECS scheduler's system ordering and parallelism are also an extra determinism surface (Egregoria had to make systems run sequentially). `bevy_ecs` does serialize via `bevy_reflect`/scenes, but it is a large dependency and slower to compile, which matters for the agent feedback loop (Section 10).

### Gaps
- I did not find a Rust project that hashes an ECS world per tick for lockstep other than bevy_ggrs (which does, via `ChecksumPlugin` + `RollbackOrdered`), so the "ECS makes hashing harder" point is an inference from the ordering problem bevy_ggrs had to solve.
- Whether `slotmap` 1.1.1 (2025-12-06) changed any serialization behaviour was not checked; the issue is still open so presumably not.

---

## 6. Serialization, snapshots, input log, content files

### Takeaway
Use `serde` with `postcard` (1.1.3, maintained, no_std, compact) for snapshots and the binary input
log, `serde_json` or `ron` for human-readable dumps and diffs, and a one-struct snapshot with an
explicit `version: u32` field. Treat the original `bincode` as discontinued: the `bincode` 3.0.0 on
crates.io (2025-12-16, published by a now-deleted account) is a dead end per docs.rs, and the live fork
is `bincode-next` 3.1.1. For content, RON or JSON with `schemars`-generated JSON Schema plus a custom
cross-reference validation pass; `validator`/`garde` cover field-level checks only.

### Cited Findings
- docs.rs for the original `bincode`: "development on bincode has ceased. No further releases will be published on crates.io"; the docs.rs build for 3.0.0 failed and the last successful build is 2.0.1 — [docs.rs bincode](https://docs.rs/bincode/*/). crates.io shows `bincode` 3.0.0 published 2025-12-16 by user `ghost_3672` (2.0.0/2.0.1 by the same account; earlier RCs by VictorKoenders) — [crates.io bincode](https://crates.io/crates/bincode) (API query 2026-10-23).
- `bincode-next` (Apich Organization): v3.0.0-rc.1 announced 2026-03-29, "apart from the MSRV change, the api of the original part remains the same as bincode v2"; v3.1.1 announced 2026-06-20 with bit packing, zero-copy, CBOR, static-size and schema fingerprinting stabilized, serde compatibility kept, MSRV 1.90, data format promised stable with regression tests, a separate v2 branch for fixes; migrate with `package = "bincode-next"` — [users.rust-lang.org rc.1](https://users.rust-lang.org/t/releasing-bincode-next-v3-0-0-rc-1/138466), [v3.1.1 post](https://users.rust-lang.org/t/releasing-bincode-next-v3-1-1-v3-stabilization-release/140842); versions on [crates.io bincode-next](https://crates.io/crates/bincode-next).
- rust_serialization_benchmark (updated 2026-10-05), `minecraft_savedata` dataset: postcard 1.1.3 serialize 433 µs / deserialize 2.16 ms / 367,489 bytes; rmp-serde 1.3.1 1.44 ms / 3.00 ms / 424,533 B; bitcode 0.6.9 128 µs / 1.28 ms / 327,688 B; serde_json 1.0.151 3.54 ms / 7.20 ms / 1,623,191 B; ron 0.12.2 8.54 ms / 28.0 ms / 1,465,223 B; savefile 0.20.5 219 µs / 1.92 ms / 566,991 B. `bincode` does not appear in the current tables — [rust_serialization_benchmark](https://github.com/djkoloski/rust_serialization_benchmark).
- `postcard` 1.1.3 (2025-07-24, 33M/90d), `rmp-serde` 1.3.1 (2025-12-23), `bitcode` 0.6.9 (2025-12-18), `savefile` 0.20.6 (2026-10-19), `ron` 0.12.2 (2026-06-22), `toml` 0.9.8 stable with 1.1.8 prerelease, `serde_json` 1.0.154 (2026-10-11), `schemars` 1.2.3 (2026-10-12), `validator` 0.21.0 (2026-07-27), `garde` 0.23.0 (2026-05-23) — crates.io pages ([postcard](https://crates.io/crates/postcard), [rmp-serde](https://crates.io/crates/rmp-serde), [bitcode](https://crates.io/crates/bitcode), [savefile](https://crates.io/crates/savefile), [ron](https://crates.io/crates/ron), [toml](https://crates.io/crates/toml), [schemars](https://crates.io/crates/schemars), [validator](https://crates.io/crates/validator), [garde](https://crates.io/crates/garde)).
- Veloren's `veloren_common_assets` exposes `load_ron` and a `MultiRon` wrapper that "combines several RON files from multiple sources" (filesystem and plugins); the docs show no validation step beyond deserialization — [docs.veloren.net](https://docs.veloren.net/veloren_common_assets/index.html).
- `bevy_asset_ron` registers RON files as Bevy assets (one extension per type) — [docs.rs](https://docs.rs/bevy_asset_ron).
- Egregoria: "Determinism must hold through serialization cycles, since the game state is serialized and sent mid-replay" — [Egregoria #90](https://github.com/uriopass/egregoria/issues/90). OpenTTD: "incomplete savegame: state that affects progression is not saved or loaded, or leftover data from a previous game is not reset" is a leading desync cause; savegames exported to JSON and diffed to find the first divergence — [OpenTTD desync.md](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md).
- `fixed` has a `serde` feature (and `serde-str` for string form in human-readable formats) — [docs.rs fixed](https://docs.rs/fixed/latest/fixed/). `indexmap`'s `map::serde_seq` serializes as an ordered sequence — [docs.rs indexmap](https://docs.rs/indexmap/latest/indexmap/).

### Inferences
- Snapshot = `struct Snapshot { version: u32, content_hash: u64, seed: u64, state: SimState }` serialized with postcard; `SimState` is the whole deterministic state (calendar tick, RNG streams, event queue, villagers, objects, `next_seq`). `content_hash` (xxh3 of the canonical content) guards against replaying against edited content. Versioning: bump `version` and keep a `fn migrate(v: u32, bytes) -> SimState` chain, or simply refuse old snapshots during early development; serde `#[serde(default)]` handles additive fields in self-describing formats (JSON/RON) but not in postcard (positional), so human-readable snapshots are the ones that migrate gracefully and postcard ones are the ones that are fast; emit both when in doubt.
- Input log = the replay file: `header { version, content_hash, seed, start_snapshot: Option<Snapshot> }` then a stream of `Record` entries: `Input { tick, event: PlayerInput }` and `Hash { tick, combined, parts: [u64; N] }` (every N ticks). Store as length-prefixed postcard frames for compactness, with a `--json` tool flag to dump as newline-delimited JSON for diffing. Because the sim's state at any tick is a pure function of (content, seed, inputs up to that tick), "save game" is just the input log, and "quick load" is the log plus the most recent periodic snapshot to avoid replaying from day 1. This is the OpenTTD `commands.log` + `dmp_cmds` design.
- Replay testing: `replay(log) -> Vec<Hash>` compared against the hashes stored in the log is a complete regression test for determinism and for "did my refactor change behaviour". Store a few canonical logs (one week of the default town) in `tests/replays/` and treat a hash change as a snapshot-test failure requiring review (Section 7).
- Content format: RON reads better for Rust-shaped data (enums, tuples) and is what Veloren uses; JSON gets schema tooling (`schemars` → JSON Schema → editor autocompletion, TS types for the other side of the wasm boundary). A reasonable split: author in RON or JSON (one choice; JSON if any content will be edited from TypeScript tooling), and always run a `content::validate(&Content) -> Result<(), Vec<ContentError>>` pass after deserialization that checks cross-references (every door's `to_place` exists, both sides of a door are walkable tiles, every schedule entry names an existing place/object, every villager has a bed, every place's walkable region is connected to at least one door, no two objects share a tile). `validator`/`garde` only check scalar constraints (ranges, lengths), so the referential checks are hand-written either way; `garde` is the more modern derive-based option, `validator` the more downloaded.
- Give every content error a stable, line-oriented message with the file and id (`villagers/mara.ron: schedule[2].place "bakery" not found`) so a coding agent can fix it from the test output.

### Gaps
- No source compared RON vs JSON vs TOML for game content specifically beyond Veloren's RON choice; TOML's nested arrays-of-tables are awkward for schedules, which is my judgement, not a cited finding.
- Snapshot sizes for a town sim were not found; the benchmark numbers above are for unrelated datasets and are indicative of per-byte throughput only.
- I did not verify whether `bincode` 3.0.0 on crates.io is a functional release or a tombstone; docs.rs says the build failed and development ceased, which is reason enough to avoid it.

---

## 7. Testing: insta, proptest/quickcheck, rstest, criterion, nextest, golden day replays

### Takeaway
`insta` 1.49 handles golden "day replay" outputs (itineraries, ASCII renders, hash tables) as file
snapshots; in CI set nothing (its `auto` mode becomes `no` when CI is detected) or run
`cargo insta test --unreferenced=reject` so stale snapshots fail; let the agent accept snapshots
locally with `cargo insta accept` after reviewing the diff. `proptest` 1.12 generates seeds/input
logs/content perturbations for invariants (every villager reaches a bed, no two in one tile,
replay hash equality). `rstest` 0.28 supplies fixtures (a loaded default town), `criterion` 0.8 or
`divan` benches, `cargo-nextest` runs tests in separate processes with per-test timeouts.

### Cited Findings
- insta `INSTA_UPDATE` modes: `auto` (default; "`no` in CI environments, otherwise `new`"), `always`, `unseen`, `new` (writes `.snap.new`), `no`; `INSTA_FORCE_PASS=1` makes a test pass so all its snapshots are created in one run, but the docs recommend `cargo insta test --review` instead; `INSTA_OUTPUT` = `diff|summary|minimal|none`; `INSTA_GLOB_FILTER`; `with_settings!` supports `description` and `info`; `allow_duplicates!`; redactions for nondeterministic data — [insta advanced docs](https://insta.rs/docs/advanced/).
- cargo-insta CLI: `cargo insta test` "forces snapshot tests to pass so they can be collected" with `--review`, `--accept`, `--accept-unseen`, `--no-force-pass`, `--keep-pending`, `--force-update-snapshots`, `--unreferenced <ignore|warn|reject|delete|auto>` (default `ignore`; `auto` = `reject` in CI, `delete` otherwise), `--glob-filter`, `--test-runner <cargo-test|nextest>`; `cargo insta review/accept/reject/pending-snapshots [--as-json]`; install via `curl -LsSf https://insta.rs/install.sh | sh` or `cargo install cargo-insta` — [insta CLI docs](https://insta.rs/docs/cli/).
- `insta` and `cargo-insta` 1.49.0 released 2026-10-03; `proptest` 1.12.0 2026-10-10; `quickcheck` 1.1.0 2026-02-10; `rstest` 0.28.0 2026-10-18; `criterion` 0.8.2 2026-02-04; `divan` 0.1.21 2025-04-10; `cargo-nextest` 0.9.149 2026-10-20; `datatest-stable` 0.3.3 2025-09-29; `cargo-mutants` 27.1.0 2026-06-02; `pretty_assertions` 1.4.1 2024-09-15; `similar-asserts` 2.0.0 2026-04-01 — crates.io pages ([insta](https://crates.io/crates/insta), [proptest](https://crates.io/crates/proptest), [quickcheck](https://crates.io/crates/quickcheck), [rstest](https://crates.io/crates/rstest), [criterion](https://crates.io/crates/criterion), [divan](https://crates.io/crates/divan), [cargo-nextest](https://crates.io/crates/cargo-nextest), [datatest-stable](https://crates.io/crates/datatest-stable), [cargo-mutants](https://crates.io/crates/cargo-mutants)).
- proptest: generation and shrinking are defined per value (strategies) rather than per type as in QuickCheck, "which makes composition more flexible"; failures are persisted and re-checked — [lambdaclass blog](https://blog.lambdaclass.com/what-is-property-based-testing/); a 2026 writeup of an agent adding properties to a falling-sand sim proposed "determinism under a fixed seed and rewind round-trip fidelity" and a "cell is not processed twice in one tick" invariant — [cyfrin.io](https://www.cyfrin.io/blog/testing-cygent-against-a-rust-codebase-a-developer-s-perspec). A crate's testing notes argue property tests should "assert durable invariants rather than only checking that a function does not panic" — [delaunay docs](https://docs.rs/crate/delaunay/0.8.0/source/docs/property_testing_summary.md).
- ggrs `SyncTestSession` "simulates a rollback every frame and resimulates the last n states ... The resimulated checksums will be compared with the original checksums and report if there was a mismatch" — [docs.rs ggrs](https://docs.rs/ggrs/0.10.2/ggrs).
- Factorio replays a save with per-tick CRC and throws a desync exception on the first differing tick — [FFF #47](https://www.factorio.com/blog/post/fff-47). The bugnet guide recommends a CI job that replays recorded sessions on every commit and compares checksums — [bugnet.io](https://bugnet.io/blog/how-to-debug-desync-in-deterministic-lockstep-games).
- Egregoria recommends "functional tests that replay games and check determinism and serialization consistency ... as regression guards" — [Egregoria #90](https://github.com/uriopass/egregoria/issues/90).

### Inferences
- insta policy for an agent-driven repo: snapshots are reviewed by a human in the diff, not by insta's TUI. Workflow: agent runs `cargo insta test --accept` (or `INSTA_UPDATE=always cargo test`) locally, inspects `git diff` of `*.snap`, and commits; CI runs `cargo insta test --unreferenced=reject` with `CI=true` so any mismatch or stale file fails. Prefer file snapshots (`assert_snapshot!(name, text)`) for multi-line ASCII renders and itineraries (readable diffs, one file per view) and inline snapshots (`@"..."`) for single hashes or short tables. Use `assert_ron_snapshot!`/`assert_yaml_snapshot!` (insta `ron`/`yaml` features) for structured state with `insta::Settings::sort_maps(true)`. Redactions are unnecessary if the sim is deterministic; needing one is a smell.
- Golden day replays: `tests/replays/default_town_week1.replay` + a test that replays and asserts (a) hash sequence equals the recorded one, (b) `assert_snapshot!` of each villager's day-1 itinerary and of the travel-time matrix. When a behaviour change is intended, re-record with a tool binary (`sim-tools record --days 7`) and accept the snapshots; the hash-sequence check is deliberately the thing that fails whenever behaviour changes, so intent becomes a reviewed diff.
- proptest invariants for this sim (use `prop_oneof!`/`vec()` strategies over seeds `u64`, input logs `Vec<(tick, Input)>`, and small content mutations): `run_two_sims_same_hashes(seed, inputs)`; `snapshot_restore_same_hashes(seed, inputs, cut_tick)`; `every_villager_in_bed_at(seed, day, 23*3600+...)` (or more realistically "within X ticks of bedtime"); `no_two_villagers_share_tile(seed, tick)`; `all_paths_valid(seed)` (each step moves to an adjacent walkable tile or crosses a door); `queue_never_contains_past_events`. Set `ProptestConfig { cases: 32, .. }` for sim-long tests and higher for cheap ones; the persisted `proptest-regressions/` files are committed so found seeds are retested.
- `rstest` fixtures: `#[fixture] fn town() -> Content { load_default() }`, `#[rstest] #[case(0)] #[case(42)] fn determinism(town: Content, #[case] seed: u64)`. `cargo-nextest` gives per-test process isolation and `slow-timeout` so a runaway sim loop fails instead of hanging CI; insta supports it via `--test-runner nextest`.
- Benchmarks: a `criterion` bench for "simulate one day" and "A* within largest place" establishes a budget; `divan` is lighter to compile if build time matters. Keep benches out of the default `cargo test` run.
- `cargo-mutants` is a cheap way to check the determinism tests actually bite (a mutant that changes a tiebreak should fail the replay test).

### Gaps
- No source documented a specific CI recipe combining `--unreferenced=reject` with `--accept` policies for agent-driven repos; the policy above is a synthesis of the documented flags.
- I did not find published proptest suites for town/colony sims; the invariant list is derived from the project's own requirements.

---

## 8. Text views and tools: ASCII grids, tables, CLI, logging, PNG

### Takeaway
`comfy-table` 8.0.1 or `tabled` 0.22 for itineraries and travel-time matrices, `unicode-width` 0.2.2
when cells may contain non-ASCII, `clap` 4.6 for the `sim-tools` binary, `tracing` 0.1.44 with a
`tracing-subscriber` fmt layer configured `.without_time()` writing to a buffer (or `tracing-test`)
for deterministic test logs, and `image` 0.25.10 (PNG encode, pure CPU) or `tiny-skia` 0.12 for
occasional PNG renders without a GPU.

### Cited Findings
- Versions and release dates: `comfy-table` 8.0.1 (2026-09-25), `tabled` 0.22.0 (2026-09-05), `unicode-width` 0.2.2 (2025-10-06), `clap` 4.6.7 (2026-09-14), `tracing` 0.1.44 (2025-12-18), `tracing-subscriber` 0.3.23 (2026-03-13), `tracing-test` 0.2.6 (2026-02-07), `test-log` 0.2.21 (2026-05-30), `image` 0.25.10 (2026-03-10), `tiny-skia` 0.12.0 (2026-02-02), `ratatui` 0.30.2 (2026-06-19) — crates.io pages ([comfy-table](https://crates.io/crates/comfy-table), [tabled](https://crates.io/crates/tabled), [unicode-width](https://crates.io/crates/unicode-width), [clap](https://crates.io/crates/clap), [tracing](https://crates.io/crates/tracing), [tracing-subscriber](https://crates.io/crates/tracing-subscriber), [tracing-test](https://crates.io/crates/tracing-test), [test-log](https://crates.io/crates/test-log), [image](https://crates.io/crates/image), [tiny-skia](https://crates.io/crates/tiny-skia), [ratatui](https://crates.io/crates/ratatui)).
- The renew engine's stated design: "Each capability is a library with a command-line interface and schema-versioned JSON output, so developers, scripts, CI, and AI agents can drive it the same way" — [renew-engine](https://github.com/renew-engine).
- bugnet.io recommends state dumps as "a structured text format with one line per variable, containing subsystem, entity ID, field name, and value", so the first differing line in a diff marks the desync origin — [bugnet.io](https://bugnet.io/blog/how-to-debug-desync-in-deterministic-lockstep-games). OpenTTD exports savegames to JSON to diff — [OpenTTD desync.md](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md).

### Inferences
- ASCII place render: write it by hand (a `Vec<Vec<char>>` the size of the place, fill walkable/wall/door glyphs, overlay object glyphs then villager initials, join rows) and keep it in the `sim` crate behind a `text` feature so wasm builds can expose it to the browser too; no crate is needed for a fixed-width grid of ASCII. Only pull `unicode-width` if glyphs go beyond ASCII (box drawing is single-width, emoji are not).
- Tables: `comfy-table` has `ContentArrangement::Dynamic` wrapping and presets (`UTF8_FULL`, `ASCII_MARKDOWN`); use the markdown preset for outputs that end up in docs or issues. `tabled` is derive-based (`#[derive(Tabled)]` on a row struct), convenient for itineraries.
- `sim-tools` subcommands (clap derive): `validate <content-dir>`, `render-place <place> [--tick]`, `itinerary <villager> --day N`, `travel-matrix`, `record --days N --out x.replay`, `replay x.replay [--verify-hashes] [--dump-tick T --json]`, `diff-dumps a.json b.json`, `hash-trace --every 60`. All output as text or JSON with `--json`, following the renew pattern so an agent can parse it.
- Deterministic logs: `tracing_subscriber::fmt().without_time().with_ansi(false).with_target(false).with_writer(buf)` captured per test, or `tracing-test`'s `#[traced_test]` with `logs_contain`. Never include wall-clock or thread ids in snapshot-captured logs. Log at `debug` with the tick as a structured field (`tracing::debug!(tick, villager = %id, "arrive")`) so greps on `tick=` work.
- PNG: `image::RgbaImage::from_fn(w, h, |x, y| ...)` with one pixel (or 4x4 block) per tile and `.save("out.png")` is ~20 lines and has no system dependencies; `tiny-skia` only if anti-aliased lines/text are wanted (it has no text; combine with `fontdue`/`ab_glyph` for labels). Keep these in the `tools` crate so the sim and wasm crates never compile `image`.

### Gaps
- No source compares `comfy-table` vs `tabled` in depth; both are maintained and the choice is stylistic.

---

## 9. Wasm export basics for the sim crate

### Takeaway
Build the sim as `rlib` and a thin `sim-wasm` crate as `cdylib` with `wasm-bindgen` 0.2.129; keep
`getrandom` out of the dependency graph entirely (`rand` with `default-features = false`), which
sidesteps the `wasm_js` backend/cfg dance; set `panic = "abort"`, `opt-level = "s"` or `"z"`,
`lto = true`, `codegen-units = 1` in the release profile and run `wasm-opt -Oz` (roughly 15-20%
smaller per the rustwasm book). Verify the hash sequence from the wasm build equals the native one
in CI.

### Cited Findings
- getrandom 0.3 on `wasm32-unknown-unknown` requires both the `wasm_js` feature and a cfg flag, recommended per-target in `.cargo/config.toml`: `[target.wasm32-unknown-unknown] rustflags = ['--cfg', 'getrandom_backend="wasm_js"']`; enabling `wasm_js` "will bloat Cargo.lock on all platforms and can cause build issues on some non-web WASM platforms"; multiple getrandom versions in one graph (0.1/0.2 use a different `js` feature) each need their own fix; the flag is also needed for docs.rs — [getrandom docs](https://docs.rs/getrandom/0.3/), [users.rust-lang.org thread](https://users.rust-lang.org/t/getrandom-version-conflict-error-building-a-wasm/132355). getrandom's current release is 0.4.3 (2026-06-17) — [crates.io](https://crates.io/crates/getrandom).
- rand 0.10.3 default features are `std`, `std_rng`, `sys_rng`, `thread_rng`; `sys_rng` pulls in `getrandom` — [docs.rs rand features](https://docs.rs/crate/rand/latest/features). aHash's default `runtime-rng` feature also seeds via `getrandom` — [aHash README](https://github.com/tkaitchuck/aHash/blob/master/README.md).
- rustwasm book, code size: `[profile.release] lto = true` ("shrinks the .wasm and speeds it up, at the cost of longer compile times"), `opt-level = 's'` or `'z'` ("'s' can sometimes produce a smaller binary than 'z'. Always measure!"); `wasm-opt -Os/-Oz` after LLVM "can save roughly 15-20% on code size, often with runtime speedups"; `wasm-opt` strips the names section unless `-g`; `twiggy top` for size profiling (names section was 9,158 bytes / 19.65% in their example); avoid `format!`/`to_string`, avoid panics (`get(i)`, `checked_div`), the default dlmalloc allocator is about 10 KB; `wasm-snip` + `wasm-opt --dce` — [rustwasm book: Shrinking .wasm size](https://rustwasm.github.io/docs/book/reference/code-size.html).
- Rapier's wasm/JS build is "fully cross-platform deterministic across browsers, OSes, and processors", verified by MD5 of world snapshots — [rapier determinism](https://rapier.rs/docs/user_guides/rust/determinism/).
- Wasm NaN nondeterminism: sign nondeterministic, payload canonical only if inputs canonical — [LLVM Discourse quoting the Wasm spec](https://discourse.llvm.org/t/semantics-of-nan/66729?page=2). `alice-det-math` claims `+ - * / sqrt` are bit-identical across SSE2/aarch64/wasm32 while libm transcendental functions differ (library docs) — [docs.rs alice-det-math](https://docs.rs/alice-det-math).
- `bevy_northstar` disables its `parallel` feature "for single-threaded grid builds, for example on WASM" — [docs.rs bevy_northstar](https://docs.rs/bevy_northstar/).
- `caputchin-replay-rs`: generates C-ABI exports for a wasm replay module, requires the run to "avoid clocks, RNG beyond the seed, and host calls", and builds browser and server from "the same sim into one WASM module" so both replay identical code — [docs.rs caputchin-replay-rs](https://docs.rs/caputchin-replay-rs).
- `wasm-bindgen` 0.2.129 (2026-09-25), `console_error_panic_hook` 0.1.7 (2021-10-11), `wasm-opt` crate wrapper 0.116.1 (2024-03-31) — crates.io pages ([wasm-bindgen](https://crates.io/crates/wasm-bindgen), [console_error_panic_hook](https://crates.io/crates/console_error_panic_hook), [wasm-opt](https://crates.io/crates/wasm-opt)).

### Inferences
- Crate shape: `sim` is a plain `[lib]` (rlib) with `#![cfg_attr(not(feature = "std"), no_std)]` optional but not required (wasm32-unknown-unknown has `std` minus threads, fs, time, net); `sim-wasm` has `[lib] crate-type = ["cdylib", "rlib"]`, depends on `sim` and `wasm-bindgen`, and exposes `new(content_bytes, seed) -> SimHandle`, `step(n)`, `apply_input(bytes)`, `snapshot() -> Vec<u8>`, `hash() -> u64` (as `BigInt` or two `u32`s, since `u64` crosses the boundary as `BigInt`), `render_place(id) -> String`. CLI-only deps (`clap`, `comfy-table`, `image`) live in `tools`, never in `sim`; if some text view must exist in both, gate it behind a `text` feature on `sim` with no extra deps.
- No-`getrandom` check: `cargo tree -p sim-wasm --target wasm32-unknown-unknown -e normal | grep -E 'getrandom|ahash|instant|web-time'` must be empty; add it as a CI step. Any crate that pulls `std::time::Instant` on wasm panics at runtime (`Instant::now()` is unimplemented on wasm32-unknown-unknown), so this check is also a runtime-crash guard.
- Threads: none on wasm32-unknown-unknown without the atomics/shared-memory flags; `rayon` must not appear in the sim graph (it would be a determinism hazard anyway).
- Profiles:

  ```toml
  [profile.release]
  opt-level = "s"      # measure against "z"
  lto = true
  codegen-units = 1
  panic = "abort"
  strip = true
  [profile.release.package.sim-wasm]
  opt-level = "z"
  ```
  then `wasm-opt -Oz --enable-bulk-memory -o out.wasm in.wasm` (binaryen binary from GitHub releases or `wasm-opt` crate; the crate wrapper is pinned at binaryen 116 from 2024, so prefer a current binaryen binary in CI). `panic = "abort"` also removes unwinding tables; keep `console_error_panic_hook` in debug builds only.
- Compile times: the sim crate's own dependency list is tiny (serde, postcard, rand_pcg, pathfinding, xxhash-rust), so an incremental debug build should be seconds; the wasm release build with `lto` + `codegen-units = 1` is the slow one and belongs in CI, not the agent's inner loop. Use `cargo check --target wasm32-unknown-unknown -p sim-wasm` in the inner loop to catch target-specific compile errors cheaply.
- Determinism across native and wasm is expected to hold because the sim uses only integer arithmetic (no NaN or libm issue applies); still assert it in CI by running the same replay under node (`wasm-bindgen-test` or a tiny JS harness, which is the other researcher's area) and comparing the hash sequence to the native run.

### Gaps
- `getrandom` 0.4's wasm configuration may differ from the 0.3 docs quoted above; irrelevant if getrandom is excluded, which is the recommendation.
- No source gave wasm binary-size numbers for a small Rust sim with serde + pathfinding; expect a few hundred KB before `wasm-opt`, but this is unverified.
- Rust's `wasm32-unknown-unknown` ABI change (C ABI fix, 2025) and `wasm-bindgen` minimum versions were not researched; pin a 2026 `wasm-bindgen` and matching CLI.

---

## 10. Project layout, toolchain, lints, and agent-driven Rust practice

### Takeaway
A four-crate workspace (`sim`, `content`, `tools`, `sim-wasm`) with `[workspace.lints]` shared
clippy settings (`-D warnings`, a chosen `pedantic` subset), a `rust-toolchain.toml` pinning a stable
channel and the `wasm32-unknown-unknown` target, `rustfmt.toml`, `cargo-deny` for licenses and
duplicate versions, and a `clippy.toml` in `sim/` banning clocks and hash maps. Keep the sim crate's
dependency count small: agent reports consistently cite the compiler error loop and fast tests as the
main leverage, and over-abstraction as the main failure mode.

### Cited Findings
- HackerNoon on Claude Code/Codex with Rust: error messages give "the exact source location, an explanation of the violated rule, fix suggestions where possible, and links to documentation"; "the feedback loop is measured in seconds"; recommends a repository-level CLAUDE.md that documents "how your crates are organized" — [hackernoon](https://hackernoon.com/coding-rust-with-claude-code-and-codex).
- Michael Yuan (AI Engineer 2025): Rust's type system and diagnostics make it suited to AI generation because "compilation provides a tight, verifiable feedback loop"; RustCoder feeds compiler output back into generation — [ai.engineer talk](https://ai.engineer/talks/rust-is-the-language-of-the-agi).
- RustConf 2026 session on "vibe-coded" contributions: code that is "syntactically correct but architecturally un-idiomatic", examining LLM patterns against ownership, the type system and "Rust's crate design" — [RustConf 2026 schedule](https://rustconf2026.sched.com/event/2KHsj).
- Anonymous 100k-line retrospective: agents "tend to over-engineer for flexibility"; prefer "concrete types over generics unless the abstraction is strictly necessary" (unverified blog) — [aiindigo.com](https://aiindigo.com/blog/ai-driven-development-lessons-from-100k-lines-of-ai-generated-rust).
- Clippy config fields for `disallowed-*` including `allow-invalid` — [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html); CI should run `cargo clippy --all-targets` or test-only usage slips through — [schneems.com](https://schneems.com/2025/11/19/find-accidental-code-usage-with-a-custom-clippytoml/).
- `cargo-deny` 0.20.2 (2026-07-09) — [crates.io](https://crates.io/crates/cargo-deny). `cargo-nextest` 0.9.149 (2026-10-20) — [crates.io](https://crates.io/crates/cargo-nextest).
- renew engine: 29 modular crates, every capability "a library with a command-line interface and schema-versioned JSON output" so "CI, and AI agents can drive it the same way" — [renew-engine](https://github.com/renew-engine).
- Egregoria: "Only the game state in the `egregoria` crate needs to be deterministic. The UI can be nondeterministic" — [Egregoria #90](https://github.com/uriopass/egregoria/issues/90).

### Inferences
- Workspace:

  ```
  Cargo.toml            # [workspace] members, [workspace.dependencies], [workspace.lints]
  rust-toolchain.toml   # channel = "1.9x", components = ["clippy","rustfmt"], targets = ["wasm32-unknown-unknown"]
  rustfmt.toml          # edition = "2024", max_width = 100 (keep defaults otherwise)
  deny.toml             # licenses allow-list, bans.multiple-versions = "warn", advisories
  crates/content/       # serde types for buildings/rooms/doors/villagers/schedules + validate(); no sim logic
  crates/sim/           # the deterministic core; depends on content; clippy.toml with the bans
  crates/tools/         # bin: sim-tools (clap, comfy-table, image, tracing-subscriber)
  crates/sim-wasm/      # cdylib + rlib, wasm-bindgen only
  content/              # the data files (RON/JSON), schemas/ generated by schemars
  tests/replays/        # golden replay logs
  ```
  Splitting `content` from `sim` lets `tools` validate and render content without the sim, keeps the content types' serde derives compiling fast, and gives `schemars` a single crate to generate schemas from. Resist splitting further until compile times demand it; each crate boundary is an API an agent must keep coherent.
- `[workspace.lints.rust] unsafe_code = "forbid"` (there is no reason for unsafe in this sim) and `[workspace.lints.clippy] all = "deny"`, `pedantic = "warn"` with explicit allows for the noisy ones (`module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `cast_possible_truncation` if tile math casts are common), plus `float_arithmetic = "deny"` and `disallowed_types`/`disallowed_methods` (default-on, configured via `sim/clippy.toml`). Crates inherit via `[lints] workspace = true`. Treat warnings as errors in CI (`RUSTFLAGS=-Dwarnings` or `-D warnings` on the clippy invocation) so the agent sees a non-zero exit, which is the signal it acts on.
- Feedback loop for the agent: `cargo check` (fast) → `cargo nextest run -p sim` (unit + replay tests, seconds) → `cargo clippy --all-targets -- -D warnings` → `cargo insta test --unreferenced=reject` for the golden views; a `justfile`/`Makefile` target `check` that runs all four in order is worth writing on day one, and `cargo check --target wasm32-unknown-unknown -p sim-wasm` belongs in it. Keep test names and assertion messages descriptive, because the agent reads the failing test's output, not the test's intent.
- Guardrails against over-engineering: no traits for things with one implementation, no generics over the id type, no plugin registries; the sim's public API is a handful of functions on `Sim`. Put this in the repo's agent instructions alongside the determinism rules (no clocks, no HashMap, no floats, no threads, every new state field goes in the snapshot and the hash).

### Gaps
- The exact pedantic subset is a judgement; no source surveyed which clippy lints are productive for game sims.
- I found no quantitative study of agent productivity vs crate granularity; the "small sim crate, fast tests" guidance is consistent across the (anecdotal) sources but not measured.

---

## 11. Precedents: Rust games/sims with deterministic lockstep or replay

### Takeaway
The clearest Rust precedents are Egregoria (city sim that abandoned hecs and forked slotmap over
determinism), the `ggrs`/`bevy_ggrs` rollback stack (`SyncTestSession` and per-component checksums),
Rapier's cross-platform determinism mode, the renew engine (Q47.16 fixed point, five-target digest
CI), and the small RTS Orion (Q16.16, FNV checksums every 24 ticks). Veloren is not deterministic.
Reported bug classes match Factorio/OpenTTD: hash-map order, float/trig, unsaved state, unstable
sorts, allocator state (slotmap free list) not round-tripping through serialization.

### Cited Findings
- Egregoria (Rust city builder, lockstep multiplayer and replays): identified hash-map iteration order, slotmap free lists not surviving serialization, hecs not serializing entity state, floating point/trig, and parallel system ordering; removed hecs in favour of entity lists (Aug 2023); forked slotmap as `slotmapd` with a serialized, validated free list; issue still open — [Egregoria #90](https://github.com/uriopass/egregoria/issues/90).
- `ggrs` 0.13.0 / `bevy_ggrs` 0.22.0 (both 2026-06-26): `DesyncDetection` compares checksums between peers; `SyncTestSession` resimulates the last n frames every tick and compares checksums; bevy_ggrs `ChecksumPlugin`, `RollbackOrdered`, seahash-based `checksum_hasher`; the BoxGame example uses f32 math that "may produce different results across different CPU architectures or operating systems, leading to desyncs" — [docs.rs ggrs](https://docs.rs/ggrs/0.10.2/ggrs), [docs.rs bevy_ggrs](https://docs.rs/bevy_ggrs/); versions on [crates.io ggrs](https://crates.io/crates/ggrs), [bevy_ggrs](https://crates.io/crates/bevy_ggrs).
- Rapier: locally deterministic by default, cross-platform with `ComplexField::sin` instead of `f64::sin`; wasm build cross-platform deterministic, verified by MD5 of snapshots — [rapier.rs](https://rapier.rs/docs/user_guides/rust/determinism/).
- renew engine (Rust, Apache-2.0, v0.1.1, 28 stars, updated 2026-09-21): bit-deterministic fixed-timestep Q47.16 simulation, CI digest check on five targets — [renew-engine](https://github.com/renew-engine).
- Orion (Rust RTS): Q16.16 fixed point, input-only mutation, FNV checksums every 24 ticks, WebSocket lockstep via Cloudflare Worker relay — [orion wiki](https://github.com/wachtelhund/orion/wiki/v0.1.0).
- Tickwise (Rust, MIT, early): engine-agnostic record/replay/desync toolkit, `.rec` with inputs + per-tick hashes, `compare`/`diff`/`inspect` CLI, self-check replays twice — [docs.rs tickwise 0.2.0](https://docs.rs/crate/tickwise/0.2.0).
- caputchin-replay-rs: server re-runs a recorded trace in wasm and "trusts only the replayed result"; same sim compiled into one wasm module for browser and server — [docs.rs](https://docs.rs/caputchin-replay-rs).
- FantasyCresendoBevy lists cross-platform determinism for rollback netplay as a goal, experimental — [GitHub](https://github.com/HouraiTeahouse/FantasyCresendoBevy).
- Factorio: whole-map CRC per tick in debug mode, replay throws on first differing tick; the Egregoria issue relays that Factorio's custom trig functions made float determinism a non-issue for them (secondhand) — [FFF #47](https://www.factorio.com/blog/post/fff-47), [Egregoria #90](https://github.com/uriopass/egregoria/issues/90). OpenTTD: RNG-state checksum at network frames, command logging and replay, known-cause taxonomy — [OpenTTD desync.md](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md).
- Veloren's asset layer (`load_ron`, `MultiRon`) is a content-loading precedent, not a determinism one — [docs.veloren.net](https://docs.veloren.net/veloren_common_assets/index.html).
- No crate named "anguis" was found on docs.rs/crates.io in this search; other deterministic-simulation-testing crates that surfaced (`anapao`, `dsim`, `navian-dst`) target distributed-systems testing rather than games — search results via [docs.rs](https://docs.rs/crate/anapao/0.2.0), [dsim](https://docs.rs/crate/dsim/latest), [navian-dst](https://docs.rs/navian-dst).

### Inferences
- The lesson that transfers most directly to this project is Egregoria's: generic game-dev crates (ECS, arenas) optimize for iteration speed and treat allocator/ordering state as an implementation detail, which is exactly the state a replay needs to be explicit. A town sim with a few dozen fixed entities does not need those crates, so it can avoid the forks.
- Nobody in these sources reports a determinism bug from `BinaryHeap` or from integer math; all reported bugs are floats, hashing/ordering, unsaved state, or parallelism. An integer-only, single-threaded, BTreeMap/Vec-based core with the whole state in one serde struct removes every reported class except "forgot to put it in the snapshot", which the snapshot-restore-and-compare test (Section 2) catches.

### Gaps
- Egregoria's current determinism status (which recommendations were implemented after 2023) is not visible from the issue page.
- I did not find a Dwarf-Fortress-like or Stardew-like Rust project that documents deterministic replay; the precedents are RTS/rollback/physics/city-sim.
- "caputchin-replay-rs" maturity and adoption are unverified beyond its docs.rs page.
