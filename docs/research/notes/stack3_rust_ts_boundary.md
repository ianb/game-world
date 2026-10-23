# Stack 3: Keeping a Rust-to-TypeScript (wasm) boundary tight and consistent

Researched 2026-10-23. Scope: a deterministic discrete-event sim crate compiled to wasm32-unknown-unknown, consumed by a TS viewer in the browser, developed by an AI coding agent in a headless Linux container, built in CI. Mobile deployment and renderer libraries are out of scope.

Version snapshot (crates.io / npm registry APIs queried 2026-10-23; dates are registry `created_at`):

| Tool | Latest | Released | Note |
|---|---|---|---|
| wasm-bindgen / wasm-bindgen-cli | 0.2.129 | 2026-09-25 | 0.2.128 was 2026-09-04, 0.2.127 2026-08-08; roughly monthly cadence in 2026 |
| wasm-pack (crate and npm) | 0.15.0 | 2026-05-15 | 0.14.0 2026-01-20; 0.13.1 was 2024-10-29 (15-month gap before the move) |
| serde-wasm-bindgen | 0.6.5 | 2024-02-27 | no release in 2.5 years |
| ts-rs | 12.0.1 | 2026-01-31 | 11.1.0 2025-10-14; MSRV 1.88 |
| tsify | 0.5.8 | 2026-08-23 | madonoharu/tsify, co-owner siefkenj (author of tsify-next) |
| typeshare / typeshare-cli | 1.0.5 / 1.13.4 | 2026-01-02 / 2025-12-11 | |
| specta / specta-typescript / tauri-specta | 2.0.0-rc.27 / 0.0.14 / 2.0.0-rc.26 | 2026-10-21 | still release candidates |
| schemars | 1.2.3 | 2026-10-12 | |
| postcard | 1.1.3 | 2025-07-24 | |
| rmp-serde | 1.3.1 | 2025-12-23 | |
| bincode | 3.0.0 | 2025-12-16 | same day as the RUSTSEC unmaintained advisory; see encoding section |
| bitcode | 0.6.9 | 2025-12-18 | |
| ron | 0.12.2 | 2026-06-22 | 0.10.1 Apr 2025, 0.11.0 Aug 2025, 0.12.0 Nov 2025 |
| insta | 1.49.0 | 2026-10-03 | |
| proptest | 1.12.0 | 2026-10-10 | |
| trunk | 0.22.0 | 2026-10-20 | |
| just | 1.59.0 | 2026-10-15 | |
| cargo-make | 0.37.24 | 2025-01-18 | |
| indexmap | 2.14.2 | 2026-09-05 | |
| vite-plugin-wasm (npm) | 3.6.0 | 2026-03-15 | |
| vite-plugin-top-level-await (npm) | 1.6.0 | 2025-07-17 | |
| vitest (npm) | 5.0.5 | 2026-10-15 | |
| vite (npm) | 8.3.5 | 2026-10-21 | |
| typescript (npm) | 7.0.2 | 2026-07-08 | |
| zod (npm) | 4.6.7 | 2026-10-12 | |
| @msgpack/msgpack / msgpackr (npm) | 3.1.3 / 2.1.2 | 2025-12-26 / 2026-10-20 | |
| json-schema-to-typescript / json-schema-to-zod (npm) | 16.0.0 / 2.8.1 | 2026-08-28 / 2026-04-01 | |

Sources for the table: [crates.io](https://crates.io/) per-crate pages (e.g. [wasm-bindgen](https://crates.io/crates/wasm-bindgen), [wasm-pack](https://crates.io/crates/wasm-pack), [ts-rs](https://crates.io/crates/ts-rs), [ron](https://crates.io/crates/ron)) and [registry.npmjs.org](https://registry.npmjs.org/) per-package metadata, queried via their JSON APIs.

## 1. Type generation from Rust to TypeScript

### Takeaway
For serde-shaped event/snapshot data, ts-rs 12 (derive + `cargo test` writes `bindings/*.ts`, committed and diffed in CI) is the most mature, most widely used and most agent-legible option; tsify 0.5.8 is the right choice if the types also cross the wasm-bindgen boundary directly because it writes into wasm-bindgen's own `.d.ts`. wasm-bindgen alone cannot describe data-carrying enums; typeshare is alive but slow and multi-language-oriented; specta is still on release candidates; schemars-to-JSON-Schema is the best "schema-first" bridge when you also want runtime validation (zod) or an editor schema.

### Cited Findings

**ts-rs (12.0.1)**
- Current version 12.0.1, MIT; MSRV 1.88.0. Derive `TS` and add `#[ts(export)]`; "When running `cargo test` or `cargo test export_bindings`" the type is written to `bindings/User.ts` as `export type User = { user_id: number, first_name: string, last_name: string, };`. Non-test alternatives: `TS::export_all`, `TS::export`, `TS::export_to_string`. — [docs.rs ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
- Env vars: `TS_RS_EXPORT_DIR` (default `./bindings`), `TS_RS_IMPORT_EXTENSION` (extension in generated `import` statements), `TS_RS_LARGE_INT` (binding for i64/u64/i128/u128, default `bigint`). Docs suggest setting them in `.cargo/config.toml` under `[env]`, e.g. `TS_RS_LARGE_INT = "number"`. — [docs.rs ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
- serde-compat (default feature) honours `rename`, `rename-all`, `rename-all-fields`, `tag`, `content`, `untagged`, `skip`, `skip_serializing`, `skip_serializing_if`, `flatten`, `default`. "skip_serializing and skip_serializing_if only have an effect when used together with #[serde(default)]". `skip_deserializing` is ignored. Unsupported serde attributes produce a build warning unless `no-serde-warnings` is enabled. — [docs.rs ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
- Feature flags: `serde-compat`, `format` (format generated TS, extra deps), `no-serde-warnings`, and library integrations `serde-json-impl`, `chrono-impl`, `bigdecimal-impl`, `url-impl`, `uuid-impl`, `bson-uuid-impl`, `bytes-impl`, `indexmap-impl`, `ordered-float-impl`, `heapless-impl`, `semver-impl`, `smol_str-impl`, `tokio-impl`, `jiff-impl`, `arrayvec-impl`. — [docs.rs ts-rs](https://docs.rs/ts-rs/latest/ts_rs/)
- Container attributes: `export`, `export_to` ("Specifies where the type should be exported to. Defaults to `<name>.ts`", relative to `TS_RS_EXPORT_DIR`, trailing `/` means directory, requires `export`), `as`, `type`, `rename`, `rename_all`, `concrete` (e.g. `#[ts(concrete(I = std::vec::IntoIter<String>))]` "will always generate `type SearchResult = Array<String>`"), `bound`, `crate`. Struct: `tag`, `optional_fields` (`t?: T`, or `= nullable` for `t?: T | null`). Field: `type`, `as` (with `_` meaning the field's own type, e.g. `Option<_>`), `rename`, `inline`, `skip`, `optional`, `flatten`. Enum: `tag`, `content`, `untagged`, `rename_all`, `rename_all_fields`, `repr(enum)` ("Exports the enum as a TypeScript enum instead of type union", discriminants kept; `repr(enum = name)` emits `VariantName = "VariantName"`). Variant: `rename`, `skip`, `untagged`, `rename_all`. `Option<T>` fields render as `T | null` by default. — [docs.rs ts-rs trait TS](https://docs.rs/ts-rs/latest/ts_rs/trait.TS.html)
- Release cadence: 11.0.0 2025-06-03, 11.0.1 2025-06-05, 11.1.0 2025-10-14, 12.0.0 and 12.0.1 2026-01-31; ~19.7M downloads. — [crates.io ts-rs](https://crates.io/crates/ts-rs)
- The ts-rs README and docs do not document a built-in "check" mode or CI drift detection; they only say "We recommend doing this in your tests." — [GitHub Aleph-Alpha/ts-rs](https://github.com/Aleph-Alpha/ts-rs)

**Drift-check tooling that does exist**
- ts-typegen (1.2.0 published 2026-08-10; 1.3.x also listed): `#[derive(Ts)]` "expands to nothing"; a `build.rs` calling `ts_typegen_build::Config::new().output("bindings").run()` parses `src/**/*.rs` with `syn` and writes one `.ts` per type plus an `index.ts` barrel. "`TS_TYPEGEN_CHECK=1 cargo build` generates the bindings in memory, compares them to the files on disk, and fails with a diff rather than writing." Reads serde `rename`, `rename_all`, `rename_all_fields`, `skip`, `skip_serializing`, `skip_serializing_if`, `flatten`, `transparent`, `tag`, `content`, `untagged`; `with`/`serialize_with` are build errors because the wire format cannot be inferred; all four serde enum representations supported, output follows `serde_json`. Because it sees source text only, aliases and foreign types need explicit mappings. — [docs.rs ts-typegen](https://docs.rs/crate/ts-typegen/1.2.0)
- rspyts-cli 0.2.0 has a `check` command that regenerates bindings from the host cdylib and compares against committed output using the same features/profile/lockfile policy as the build. — [docs.rs rspyts-cli](https://docs.rs/crate/rspyts-cli/0.2.0)
- typebridge-cli ("typewriter") has `check --ci` that exits non-zero on drift, configured by `typewriter.toml`. — [docs.rs typebridge-cli](https://docs.rs/typebridge-cli/)

**tsify (0.5.8, 2026-08-23)**
- "Tsify is a library for generating TypeScript definitions from rust code"; repo madonoharu/tsify, owners madonoharu and siefkenj. Features: `json` (default, via serde_json) and `js` (via serde-wasm-bindgen, "will be the default in future versions"). — [docs.rs tsify](https://docs.rs/crate/tsify/latest)
- `into_wasm_abi`/`from_wasm_abi` are **deprecated**: when deserialization fails, `wasm_bindgen::throw_str` skips destructors and leaks; use the `Ts<T>` wrapper (`#[repr(transparent)]` over `JsValue`) on parameters/returns and call `.to_rust()` to get an ordinary `Result`. `Ts<Vec<T>>` is unsupported; use `Vec<Ts<T>>`. Serializer options as attributes: `missing_as_null`, `hashmap_as_object`, `large_number_types_as_bigints`. Container attrs `namespace`, `type`, `type_params`, `type_prefix`/`type_suffix` (warning: partial application yields references to undeclared names), `rename`; field attrs `type`, `type_params`, `optional`. — [docs.rs tsify](https://docs.rs/crate/tsify/latest)
- Example output. Struct `Point { x: i32, y: i32 }` gives `export interface Point { x: number; y: number; }` plus `export function from_js(point: Point): void; export function into_js(): Point;`. Enum with unit, tuple and struct variants gives `export type Color = | "Red" | "Blue" | "Green" | { Rgb: [number, number, number] } | { Hsv: { hue: number; saturation: number; value: number } };`. — [docs.rs tsify](https://docs.rs/crate/tsify/latest)
- tsify-next's README said the original Tsify "appears to be in hibernation mode" and that it would maintain updates until upstream returned; lib.rs shows tsify 0.5.5 (Apr 2025) and 0.5.6 (2025-10-14), and an aggregator says upstream has caught up and tsify-next's authors now recommend it (aggregator claim, verify). — [lib.rs tsify](https://lib.rs/crates/tsify); [openapps tsify-next](https://openapps.pro/packages/tsify-next)
- Real-world rule set (Bitwarden SDK): default to tsify; use wasm-bindgen only when "the web-side needs to call functions" or hold live Rust objects. wasm-bindgen objects are "references to Rust-owned memory" and classes include `free`; tsify records are copied into plain JS objects typed as interfaces. External types with no tsify support (Uuid, DateTime<Utc>) are declared manually: `#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)] const TS_CUSTOM_TYPES: &'static str = r#"export type Uuid = string;..."#;`. `getter_with_clone` is required for `String` fields on exported structs. — [Bitwarden SDK interoperability](https://contributing.bitwarden.com/architecture/sdk/web/interoperability)

**wasm-bindgen's own .d.ts**
- wasm-bindgen "has limitations when it comes to handling more complex Rust types such as enums with values or deeply nested structures"; tsify "is generally better for handling complex data types". — [Hackernoon guide](https://sia.hackernoon.com/how-to-share-rust-types-with-typescript-for-webassembly-in-30-seconds-a-quick-guide); same framing in [Bitwarden](https://contributing.bitwarden.com/architecture/sdk/web/interoperability)
- String enums got TS types via PR #4147 ("users might have worked around the lack of string enum types with typescript_custom_section before"); const enums were rejected because they do not allow string literals directly. — [wasm-bindgen PR 4147](https://github.com/rustwasm/wasm-bindgen/pull/4147)
- Changelog: 0.2.107 includes doc comments in TS class definitions and sorts custom sections alphabetically; 0.2.114 `unchecked_optional_param_type`; 0.2.121 string enums and dynamic unions emit `export type`; 0.2.128 adds `--ts-typed-array-buffers` (needs TypeScript 5.7+), declares `initSync` for `no-modules`, and types `Vec` of C-style enums as `EnumName[]` instead of `any[]`. Unreleased: number arrays typed as typed array or plain array; struct fields get getter/setter pairs; 64-bit arrays typed `BigInt64Array | bigint[]`. — [wasm-bindgen CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)
- Non-determinism of generated `.d.ts`: "custom TypeScript sections come out in encounter order" (issue tracked as wasm-bindgen#4738); the changelog shows 0.2.106 emitting declarations alphabetically and 0.2.107 sorting custom sections across codegen units. — [harbor task wasm-bindgen-4738](https://hub.harborframework.com/tasks/abundant/wasm-bindgen__wasm-bindgen-4738); [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)

**typeshare (1Password)**
- Open-sourced Nov 2022; generates Swift, Kotlin, Scala, TypeScript (Go and Python "experimental"). README shows `#[serde(tag = "type", content = "content")]` enums becoming a TS union such as `{ type: "MyOtherVariant", content: undefined }`. Repo: 3.0k stars, 73 open issues, 24 PRs, 590 commits; no archive banner. — [GitHub 1Password/typeshare](https://github.com/1Password/typeshare)
- typeshare-cli releases: 1.13.1 2024-11-20, 1.13.2 2024-11-21, 1.13.3 2025-06-10, 1.13.4 2025-12-11; `typeshare` 1.0.5 2026-01-02. — [crates.io typeshare-cli](https://crates.io/crates/typeshare-cli)
- 1Password uses typeshare to pass requests from browser-extension TS to a Rust core compiled to wasm; annotations must sit on in-tree type definitions, which pushed them toward a hard fork. — [1Password passkey crates](https://1password.com/blog/passkey-crates)

**specta / tauri-specta**
- specta 2.0.0-rc.27, specta-typescript 0.0.14, tauri-specta 2.0.0-rc.26, all pushed 2026-10-21. — [crates.io specta](https://crates.io/crates/specta)
- TypeScript and Swift exporters "Stable"; JSON Schema, Zod and others "Experimental" ("Pre-1.0 and still evolving"). Types collected explicitly (`Types::default().register::<TypeOne>()`) or via `specta::export()` for all `#[derive(Type)]` types (exclude with `#[specta(collect = false)]`). Export: `Typescript::default().export_to("./bindings.ts", &types).unwrap();`. Unit enum `pub enum MyEnum { A, B, C }` -> `export type MyEnum = "A" | "B" | "C";`. Usable without Tauri. — [GitHub specta-rs/specta](https://github.com/specta-rs/specta)
- docs.rs for tauri-specta still shows the older `ts::export`/`collect_types!` API, which does not match 2.x RC; check the repo README. — [docs.rs tauri-specta](https://docs.rs/tauri-specta)

**schemars route**
- schemars 1.2.3 (2026-10-12), MSRV 1.74. 1.0 changed schemas to plain `serde_json::Value`s; project policy: "the exact structure of generated schemas may change between versions of schemars - this is not considered a breaking change". — [docs.rs schemars](https://docs.rs/schemars/latest); [Zed migration commit](https://git.secluded.site/zed/commit/5fafab6e52f87cd5d6235628b12e6c62a2db0fd4)
- json-schema-to-zod (npm 2.8.1): "runtime package and CLI tool to convert JSON schema (draft 4+) objects or files into Zod schemas"; n8n maintains a fork `@n8n/json-schema-to-zod`. json-schema-to-typescript 16.0.0 (2026-08-28). A Rust crate `vld-ts` converts JSON Schema `serde_json::Value` to Zod code strings. — [npm json-schema-to-zod](https://www.npmjs.com/package/json-schema-to-zod); [docs.rs vld-ts](https://docs.rs/crate/vld-ts/0.2.0)

### Inferences
- For an event protocol that is pure data (ticks, entity ids, enum payloads), ts-rs's output is a plain `.ts` file under version control: a diff of generated TS is the drift signal, and `cargo test export_bindings && git diff --exit-code bindings/` is the whole CI check. This is simpler than the schema-first path and gives the agent a single mechanical loop.
- ts-rs's `TS_RS_LARGE_INT` default of `bigint` is correct for a 64-bit sim hash but awkward for everything else; keep u64 confined to `hash()`/seed fields and set `#[ts(as = "..")]`/`type = "string"` or accept bigint only where it matters.
- tsify is preferable to ts-rs only if you want wasm-bindgen function signatures (`step_until(tick: number): Event[]`) typed with the same types. Its deprecation of `into_wasm_abi` in favour of `Ts<T>` is recent (0.5.x), and the Bitwarden page still shows the older style, so expect agent training data to be out of date here.
- schemars is worth adding alongside ts-rs even if TS types come from ts-rs: the JSON Schema serves content-file validation and an optional zod runtime check at the boundary (see section 8). Because schemars warns its output structure may change between versions, pin it exactly and commit the generated schema.
- typeshare's two-stage design (scans source text, so annotations must be in-tree) is the same constraint ts-typegen has; both reject `serialize_with`. Any custom serde serializer on a boundary type breaks every source-scanning generator; prefer `#[serde(with)]`-free boundary types.

### Gaps
- Could not confirm whether specta 2.0 final or tauri-specta 2.0 final has shipped (only RCs on crates.io as of 2026-10-21).
- Could not retrieve a GitHub "last commit" date for typeshare (API blocked in this session); crates.io shows releases in Dec 2025/Jan 2026, so it is maintained but on a slow cadence. No source discusses a "typeshare 2.0".
- ts-rs's exact rendered shapes for adjacently/internally tagged enums and `HashMap<K,V>` were not visible in the fetched docs pages (the trait page truncates). The serde-compat section confirms the attributes are honoured.

## 2. Data encoding across the wasm boundary

### Takeaway
Three viable encodings, in order of recommended default: (1) `serde-wasm-bindgen` (0.6.5) producing JS values directly, fastest common path and what wasm-bindgen officially prefers, but with `Map`-vs-object, `None`-vs-`undefined` and 64-bit-int pitfalls that must be pinned by configuration; (2) a compact binary (postcard 1.1.3 or MessagePack via rmp-serde 1.3.1 + `@msgpack/msgpack`/`msgpackr`) returned as `Vec<u8>`/`Uint8Array`, best for replays/snapshots and for hashing; (3) raw linear-memory views (`*const T` + `#[repr(C)]`) as sandspiel does, for per-tick bulk state. bincode is now officially unmaintained and should not be chosen for new work.

### Cited Findings
- serde-wasm-bindgen 0.6.5 default mapping: `()`/`None` -> `undefined`; `bool`; `f32`/`f64` -> `number`; `u8..i32` -> `number`; `u64`/`i64`/`usize`/`isize` -> `number` "if within the safe integer range" (otherwise fail unless bigint option); `u128`/`i128` -> `bigint`; `String` -> string; structs -> plain objects; `HashMap`/`BTreeMap` -> `Map`; tuples/`Vec`/`HashSet` -> arrays; `serde_bytes` buffers -> `Uint8Array`. `from_value` also accepts `null` for `None`, and `ArrayBuffer`/`Array` for bytes. — [docs.rs serde-wasm-bindgen](https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/)
- Serializer options (all default false): `.serialize_missing_as_null(true)`, `.serialize_maps_as_objects(true)`, `.serialize_large_number_types_as_bigints(true)`, `.serialize_bytes_as_arrays(true)`; `Serializer::json_compatible()` preset enables missing-as-null, maps-as-objects, bytes-as-arrays. `#[serde(with = "serde_wasm_bindgen::preserve")]` passes `Date`/`RegExp`/`JsValue` through. "provides much faster serialization/deserialization as well" than JSON "in most common cases, with much smaller code size overhead"; the wasm-bindgen project "now officially prefers it". The crate does not generate TS types and recommends tsify with the `js` feature. — [docs.rs serde-wasm-bindgen](https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/)
- Precision trap when mixing libraries: gloo-utils converts u64 to number and loses precision (12815844532926915242 came out as 12815844532926915000), while serde-wasm-bindgen can keep bigint. — [gloo issue 310](https://github.com/rustwasm/gloo/issues/310)
- Webpack 4 failed on `__wbindgen_bigint_from_u64` "Non-JS-compatible Func Signature (i64 as parameter)"; fixed in newer webpack, not backported. — [webpack PR 16339 reference](https://github-redirect.dependabot.com/webpack/webpack/pull/16339)
- wasm-bindgen changelog (Unreleased): `debugString` handles `bigint`; 64-bit number arrays typed `BigInt64Array | bigint[]`. 0.2.128: `--ts-typed-array-buffers`. — [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)
- bincode: RUSTSEC-2025-0141 "Bincode is unmaintained", published 2025-12-16, type INFO Unmaintained, no patched versions; reason given is a doxxing and harassment incident after which the team "decided to cease development permanently"; 1.3.3 considered complete; suggested replacements wincode, postcard, bitcode, rkyv. — [RUSTSEC-2025-0141](https://rustsec.org/advisories/RUSTSEC-2025-0141); bincode 3.0.0 was published the same day, 2025-12-16 — [crates.io bincode](https://crates.io/crates/bincode)
- cargo-audit flags crates depending on unmaintained bincode (Mozilla bug 2011119); wincode is only compatible with bitcode v2+. — [Mozilla bug 2011119](https://bugzilla.mozilla.org/show_bug.cgi?id=2011119)
- postcard vs bincode wire differences: bincode 2 "allows configuration of whether (fixed) array lengths are included on wire, which is also the default, while Postcard never includes it for arrays (of known length)"; `bincode::encode_to_vec` and `bincode::serde::encode_to_vec` are different, incompatible functions. — [users.rust-lang.org bincode vs postcard](https://users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740)
- One team rejected postcard as "a focus on embedded usage which is not a good fit for us" when replacing bincode. — [umwelt-info issue 174](https://gitlab.opencode.de/umwelt-info/metadaten/-/issues/174)
- Linear-memory pattern: "when you declare memory in WebAssembly, it behaves like an ArrayBuffer of a defined byte size"; a typed array over `memory.buffer` lets JS read/write directly; mismatched element sizes corrupt data. Shared (threaded) memory needs extra target features and SharedArrayBuffer headers. — [KodeKloud notes](https://notes.kodekloud.com/docs/Rust-Programming/WebAssembly-with-Rust/Accessing-WebAssembly-Memory-from-JavaScript); [Tweag wasm threads](https://tweag.io/blog/2022-11-24-wasm-threads-and-messages/)
- sandspiel (falling-sand game, Rust via wasm + WebGL + JS): `#[wasm_bindgen] pub struct Universe { cells: Vec<Cell>, rng: SplitMix64, ... }` with exported `reset`, `tick`, `paint(x, y, size, species)`, `width`, `height`, `cells(&self) -> *const Cell` (`self.cells.as_ptr()`), `winds`, `burns`, `push_undo`/`pop_undo`/`flush_undos`, `new(width, height)`. `Cell` and `Wind` are `#[repr(C)]` + `#[wasm_bindgen]`; cells indexed `(x * self.height + y)`. RNG seeded from a constant `SeedableRng::seed_from_u64(0x734f6b89de5f83cc)` but `Cell::new` uses `js_sys::Math::random()`. Built with `cd crate && wasm-pack build`, webpack bundler, `cargo watch -s 'wasm-pack build'` for dev. JS does `import { Universe } from "../crate/pkg"; const universe = Universe.new(n, n); universe.tick();` and hands `universe` to render/fluid modules. — [sandspiel lib.rs](https://raw.githubusercontent.com/MaxBittker/sandspiel/master/crate/src/lib.rs); [sandspiel js/index.js](https://raw.githubusercontent.com/MaxBittker/sandspiel/master/js/index.js); [GitHub MaxBittker/sandspiel](https://github.com/MaxBittker/sandspiel)
- rust_serialization_benchmark (Jan 2026, rustc 1.94 nightly) puts bitcode at the top on speed and size; postcard/rmp-serde figures not visible in the snippet. — [Claude artifact summary, unverified](https://claude.ai/public/artifacts/60d3f2e3-bd79-4eb3-ad28-364bad047220)

### Inferences
- For "a few hundred entities emitting an event stream per tick", any of the three encodings is fast enough; the choice should be driven by what else the bytes are used for. Events that only feed the viewer: serde-wasm-bindgen with `json_compatible()` (so TS sees plain objects, `null`, arrays, and the shape matches ts-rs/serde_json output exactly). Snapshots and replays that must be hashed, stored, and compared native-vs-wasm: a byte format (postcard is schema-less and compact; MessagePack is self-describing and has mature TS decoders), returned as `Vec<u8>` -> `Uint8Array`.
- Pin serde-wasm-bindgen's options in one Rust helper (`fn to_js<T: Serialize>(v: &T) -> JsValue { v.serialize(&Serializer::json_compatible()) }`) and never call bare `to_value` at the boundary, so `Map` and `undefined` cannot leak into TS. ts-rs's `Option<T>` -> `T | null` default matches `serialize_missing_as_null(true)` and not the crate default `undefined`; this is the most likely silent mismatch.
- u64 at the boundary: either keep `hash()` returning a hex `String`, or return `u64` through wasm-bindgen directly (wasm-bindgen maps u64 to JS `bigint`), and type it in ts-rs with `TS_RS_LARGE_INT=bigint`. Avoid u64 inside serde structs that cross via serde-wasm-bindgen unless `serialize_large_number_types_as_bigints(true)` is set everywhere.
- sandspiel's raw-pointer approach is the classic bulk-state path but is a hazard for an agent: the pointer returned by `cells()` is only valid until the next wasm memory growth, so views must be rebuilt each frame; nothing in TS types encodes this. For a DES with an event stream rather than a grid, prefer events over memory views.
- bincode is removed from consideration both because of the advisory (cargo-audit in CI will fail) and because its `serde` vs native encoders are incompatible, a trap for a TS decoder.

### Gaps
- No TS decoder for postcard with meaningful adoption was found in this research; if postcard is chosen, decoding in TS means hand-writing a varint/zigzag decoder from the postcard wire spec, or decoding inside wasm and re-emitting JS values. MessagePack avoids this.
- No measured numbers found for serde-wasm-bindgen vs JSON.stringify/parse vs MessagePack at the "few hundred entities per tick" scale; the only performance claim is the crate's own qualitative one.
- Veloren web experiments and OpenFrontIO's encoding choices were not found in reachable sources; sandspiel is the only game example with code inspected.

## 3. API shaping (thin facade vs rich objects, errors, versioning)

### Takeaway
Expose one opaque `#[wasm_bindgen]` handle (or none, using a module-level singleton) with a handful of data-in/data-out functions; keep every payload a serde type; make `Result<T, E>` cross as a JS exception with a serde-typed error; export a `PROTOCOL_VERSION` constant from Rust and assert it in TS at init.

### Cited Findings
- An exported Rust struct becomes a JS class holding `this.ptr`; associated functions become `static`, methods become instance methods passing `this.ptr`; `free()` does `const ptr = this.ptr; this.ptr = 0; wasm.__wbg_foo_free(ptr);` and "future usage of this object should trigger a panic in Rust" (`assert_not_null`). The design doc describes manual memory management as exposed to JS. — [wasm-bindgen design: exporting a Rust struct](https://wasm-bindgen.github.io/wasm-bindgen/contributing/design/exporting-rust-struct.html)
- Bitwarden: "You can usually ignore `free`, since memory is cleaned up automatically" (FinalizationRegistry-based cleanup in modern wasm-bindgen output); their rule is to use tsify records by default and wasm-bindgen objects only for live Rust state. `getter_with_clone` needed for `String` fields. — [Bitwarden SDK interoperability](https://contributing.bitwarden.com/architecture/sdk/web/interoperability)
- tsify deprecates `into_wasm_abi`/`from_wasm_abi` because a deserialization failure throws via `wasm_bindgen::throw_str`, which skips destructors and leaks; `Ts<T>` + `.to_rust()` lets the function return an ordinary `Result`. — [docs.rs tsify](https://docs.rs/crate/tsify/latest)
- wasm-bindgen changelog: 0.2.109 "`catch` is implemented in Wasm via `WebAssembly.JSTag` when Wasm exception handling is available"; 0.2.122 `-Cpanic=unwind` emits modern exnref exception handling by default and needs Node.js 22.22.3+; 0.2.123 `JsOption<T>` treats only `undefined` as empty; 0.2.109 `Closure::new()` requires `UnwindSafe` under `panic=unwind` with `_aborting` variants added. — [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)
- sandspiel's facade is exactly the "handle + a few methods" shape: `Universe::new`, `tick`, `paint`, `reset`, pointer getters, undo stack. — [sandspiel lib.rs](https://raw.githubusercontent.com/MaxBittker/sandspiel/master/crate/src/lib.rs)

### Inferences
- A facade of `new Sim(contentBytes, seed)`, `apply_input(bytes)`, `step_until(tick) -> JsValue (Event[])`, `snapshot() -> Uint8Array`, `load(bytes)`, `hash() -> bigint`, `tick() -> number` keeps exactly one `free()`-bearing class in TS, which is manageable (wrap it in a TS class whose `dispose()` calls `free()`, and rely on FinalizationRegistry as a backstop). Returning `*const T` pointers (sandspiel style) should be avoided unless profiling demands it.
- Errors: wasm-bindgen turns `Result<T, JsValue>`/`Result<T, E: Into<JsValue>>` into a thrown JS exception. Make the error a serde enum (`SimError`) exported via ts-rs, convert it with the same `to_js` helper, and in TS wrap every call in a helper that narrows `unknown` to `SimError`. Since Rust 1.8x + panic=unwind changes landed in 0.2.122, keep `panic = "abort"` in the wasm profile (the default for wasm32) so panics stay as traps and are not confused with typed errors.
- Protocol versioning: export `pub const PROTOCOL_VERSION: u32` via `#[wasm_bindgen] pub fn protocol_version() -> u32`, put the same constant in the ts-rs bindings output (a `#[ts(export)]` type or a generated `const`), and have TS throw at init if `protocol_version() !== PROTOCOL_VERSION`. Include `protocolVersion` and `contentVersion` in snapshot and replay headers so golden files self-describe.

### Gaps
- The design doc fetched predates the weak-refs/FinalizationRegistry change; its current behaviour is only indirectly attested (Bitwarden's "cleaned up automatically").
- No published guide specifically on Result-to-JS error typing with ts-rs was found; the above is inference from the crate docs.

## 4. Testing across the boundary

### Takeaway
Use insta (1.49.0) `assert_json_snapshot!`/`assert_ron_snapshot!` for Rust-side goldens, vitest (5.x) snapshots for the TS side, and a shared `fixtures/` directory of committed event-stream and snapshot files that both suites load; run the real wasm in Node by building a `--target nodejs` (CommonJS) or `experimental-nodejs-module` package for tests; add a determinism test that compares `hash()` after N ticks native vs wasm; eliminate `std::collections::HashMap` iteration and relaxed-SIMD from the sim path.

### Cited Findings
- insta 1.49.0: `assert_snapshot!` for string output; `assert_json_snapshot!`/`assert_ron_snapshot!`/`assert_yaml_snapshot!` for `Serialize` values (features `json`, `ron`, `yaml`). Snapshots live "Right next to your test in a folder called `snapshots` as individual `.snap` files"; new snapshots get `.new`, reviewed with `cargo insta review`. `INSTA_UPDATE` default `auto` is "`no` for CI environments or `new` otherwise"; other modes `new`, `always`, `unseen`, `no`, `force`. Redactions: `dynamic_redaction`, `rounded_redaction`, `sorted_redaction`. Config in `.config/insta.yaml`/`insta.yaml` with `behavior` options `require_full_match`, `force_pass`, `output`, `update`. — [docs.rs insta](https://docs.rs/insta/latest/insta/)
- Running wasm in Node tests: one project runs `wasm-pack build --target nodejs` then a Jest suite comparing the Rust/wasm implementation to the original JS library. — [rust-and-wasm-from-js](https://github.com/ivanalejandro0/rust-and-wasm-from-js)
- Vitest gotcha: the default (bundler/browser) target `package.json` has a `module` field but no `main`/`type`/`exports`, which can stop Node tools such as Vitest loading the package; fix was a post-build script adding those fields. — [sigma-rust issue 632](https://redirect.github.com/ergoplatform/sigma-rust/issues/632); [sigma-rust PR 637](https://redirect.github.com/ergoplatform/sigma-rust/pull/637)
- wasm-pack targets: `nodejs` "Outputs JS that uses CommonJS modules, for use with a `require` statement. `main` key in `package.json`"; `bundler` (default) sets `module` and `sideEffects: false`; `web` requires manual instantiation; `--dev`/`--profiling`/`--release` map to cargo profiles (release default). — [wasm-pack build docs](https://wasm-bindgen.github.io/wasm-pack/book/commands/build.html)
- wasm-bindgen 0.2.107: `experimental-nodejs-module` emits one JS entry file; 0.2.109 Atomics builds gain Node `worker_threads` support; 0.2.122 exnref output needs Node.js 22.22.3+. — [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)
- WebAssembly nondeterminism (spec design doc): NaN payload bits ("there is nondeterminism in determining the specific bits of the NaN"), NaN sign bit, "The relaxed SIMD instructions have nondeterministic results", threads/shared memory, resource exhaustion, environment values. Standard (non-relaxed) SIMD has no listed nondeterminism. — [WebAssembly Nondeterminism.md](https://github.com/WebAssembly/design/blob/main/Nondeterminism.md)
- HashMap order: "Iteration order on a HashMap (or a HashSet) is dependent on the internal hash seed of the hasher"; a fixed-seed Rust routine in Qiskit produced varying float sums until `hashbrown::HashMap` was swapped for `indexmap::IndexMap`. — [Qiskit SabreSwap fix](https://cdn05042023.gitlink.org.cn/scnc/qiskit/commit/48cfab601e16517eca1a5249a45cc43726bf7d61)
- Wasmtime/Cranelift wraps HashMap/HashSet in `StableMap`/`StableSet` that "prevent accidentally observing the non-deterministic iteration order"; Materialize bans std hash collections and recommends BTree types which "guarantee a stable iteration order, based on the Ord relation". — [cranelift_isle stablemapset](https://docs.wasmtime.dev/api/cranelift_isle/stablemapset/index.html); [Materialize mz_ore hash.rs](https://dev.materialize.com/api/rust/src/mz_ore/collections/hash.rs.html)
- wasm-bindgen's own output was non-deterministic between identical builds due to unordered-map iteration; fix was sorted export ordering (0.2.106 alphabetical declarations, 0.2.116 walrus 0.26.1 deterministic type section). — [harbor task wasm-bindgen-4813](https://hub.harborframework.com/tasks/abundant/wasm-bindgen__wasm-bindgen-4813); [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)
- proptest 1.12.0 (2026-10-10). — [crates.io proptest](https://crates.io/crates/proptest)
- sandspiel seeds `SplitMix64` from a constant but also calls `js_sys::Math::random()` in `Cell::new`, so its state is not reproducible across runs. — [sandspiel lib.rs](https://raw.githubusercontent.com/MaxBittker/sandspiel/master/crate/src/lib.rs)

### Inferences
- Contract test shape that works with these tools: `fixtures/<case>/input.json` (seed, content version, scripted inputs) plus `fixtures/<case>/events.json` and `fixtures/<case>/hash.txt` committed. Rust test: run the sim natively, `assert_json_snapshot!` the events with the snapshot path pointed at the fixture (insta `Settings::set_snapshot_path`), and assert the hash. Vitest test: load the `--target nodejs` package, run the same input, `expect(events).toEqual(JSON.parse(fixture))` and `expect(hash).toBe(...)`. The TS side also type-checks the fixture against the ts-rs-generated `Event` type, so a protocol change fails `tsc` before it fails a runtime test.
- Because insta's `INSTA_UPDATE=auto` becomes `no` under CI, a missing or changed snapshot fails in CI without extra flags; locally the agent must run `cargo insta accept` deliberately, which is the right friction. Set `INSTA_FORCE_PASS` unset and `require_full_match: true` in `insta.yaml`.
- Determinism checklist derived from the sources: no `std::collections::HashMap`/`HashSet` in sim state or iteration (use `BTreeMap`, `IndexMap`, or `HashMap` with a fixed `BuildHasher` and never iterate it); seed all RNG from the input; never call `js_sys`/`Math.random`/`Date` from the crate; do not enable `relaxed-simd` in `-Ctarget-feature`; canonicalize NaN before hashing or make NaN unreachable (f32/f64 arithmetic apart from NaN bits is IEEE-deterministic in wasm); avoid `mul_add` unless both native and wasm paths use it identically; compare native vs wasm `hash()` in CI as the test that proves the above.
- The `nodejs` target's CommonJS output is the lowest-friction way to load wasm in vitest 5; building two packages (`pkg/` bundler for Vite, `pkg-node/` for vitest) from the same crate is cheap and avoids the `package.json` `exports` problem. Alternatively use `--target web` and call `initSync({ module: fs.readFileSync(...) })` in a vitest setup file.

### Gaps
- No source found that documents vitest 5 + wasm-bindgen configuration end to end (the sigma-rust thread is the closest and predates vitest 5).
- No source found measuring native-vs-wasm hash equivalence for a Rust sim; the determinism points are from the wasm spec and generic Rust HashMap incidents.
- Whether Rust's wasm32 codegen ever contracts `a*b+c` into FMA (which would differ from native aarch64) was not verified; LLVM does not contract by default in Rust, but this is from memory, not a fetched source.

## 5. Build and toolchain pinning

### Takeaway
Pin Rust with `rust-toolchain.toml` (`channel = "1.xx.y"`, `targets = ["wasm32-unknown-unknown"]`), pin `wasm-bindgen = "=0.2.129"` in Cargo.toml and install the identical `wasm-bindgen-cli` version in CI (0.2.128 reintroduced a hard schema match), and drive everything from one `justfile`. wasm-pack survived the rustwasm sunset (now `wasm-bindgen/wasm-pack`, 0.15.0, May 2026) but a plain `cargo build --target wasm32-unknown-unknown` + `wasm-bindgen` CLI script is smaller and more predictable for an agent.

### Cited Findings
- rustwasm sunset (2025-07-21): wasm-bindgen moved to a new `wasm-bindgen` GitHub org described as a neutral home; wasm-pack, gloo, twiggy, walrus, weedle "Will be archived, unless an existing maintainer asks to take over"; the rustwasm org fully archived September 2025; wasm-bindgen "won't depend on unmaintained crates" (walrus/weedle may be inlined). — [Rust blog: Sunsetting the rustwasm GitHub org](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org)
- Follow-up (Aug 2025): wasm-pack's doc site 404'd for a while; it was transferred to maintainer drager; "neither wasm-bindgen, web-sys, wasm-pack, nor gloo will be deprecated because of the sunset". — [users.rust-lang.org Future of Rust WASM](https://users.rust-lang.org/t/future-of-rust-wasm/133089)
- wasm-pack now lives at github.com/wasm-bindgen/wasm-pack ("started by ashleygwilliams and is maintained by drager"), docs at wasm-bindgen.github.io/wasm-pack/, 332 open issues, 57 open PRs, not archived; drager/wasm-pack is a redirect stub. — [GitHub wasm-bindgen/wasm-pack](https://github.com/wasm-bindgen/wasm-pack); [GitHub drager/wasm-pack](https://github.com/drager/wasm-pack)
- wasm-pack releases: 0.12.1 2023-06-27, 0.13.0 2024-07-01, 0.13.1 2024-10-29, 0.14.0 2026-01-20, 0.15.0 2026-05-15. — [crates.io wasm-pack](https://crates.io/crates/wasm-pack)
- wasm-bindgen 0.2.128 (2026-09-04): "Requires matching `wasm-bindgen` and CLI versions (schema bump)"; 0.2.129 (2026-09-25) improved the schema mismatch error message. MSRV: 0.2.106 policy and bump to 1.71; 0.2.118 CLI 1.76 / API 1.86 (as written in the changelog; the 0.2.129 entry says library MSRV 1.77 -> 1.81, and the two figures conflict); 0.2.115 added `--target emscripten`; 0.2.122 `js_namespace` must be repeated on every `impl` block (breaking); 0.2.128 `js_namespace` on both `extern "C"` block and item is a hard error. — [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md); dates from [crates.io wasm-bindgen](https://crates.io/crates/wasm-bindgen)
- rust-toolchain.toml: mandatory `[toolchain]` table; `channel` may be `stable`/`beta`/`nightly` or `<major.minor>`/`<major.minor.patch>` optionally with a date; `components`, `targets` (additive, host included), `profile` (`minimal`/`default`/`complete`). Example: `[toolchain]\nchannel = "nightly-2020-07-10"\ncomponents = [ "rustfmt", "rustc-dev" ]\ntargets = [ "wasm32-unknown-unknown", "thumbv2-none-eabi" ]\nprofile = "minimal"`. When pinning a release, "`Cargo.lock` should probably be tracked too". — [rustup overrides](https://rust-lang.github.io/rustup/overrides.html)
- CI caching: `Swatinem/rust-cache@v2` caches `~/.cargo` (bins, registry, git deps) and `./target` (dependency artifacts); key includes rustc version/host, `RUSTFLAGS`-style env, hashes of `Cargo.lock`/`Cargo.toml`, `rust-toolchain` files and `.cargo/config.toml`; options `shared-key`, `key`, `workspaces` (`$workspace -> $target`), `cache-on-failure` (default false), `save-if`, `cache-bin`; GitHub cache limit 10 GB; run `rustup toolchain install` before the cache step "as the cache uses the current rustc version as its cache key". Example: `- uses: Swatinem/rust-cache@v2\n  with:\n    shared-key: ""\n    save-if: ${{ github.ref == 'refs/heads/master' }}`. — [GitHub Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- Vite integration: vite-plugin-wasm README pairs `wasm()` with `topLevelAwait()` from vite-plugin-top-level-await, needed "unless you target very modern browsers only" (`build.target: 'esnext'`); workers need both plugins in `worker.plugins`; README says Vite 2.x-6.x supported (an index claims up to 8.x; unverified). vite-plugin-wasm 3.6.0 (2026-03-15), Vite 8.3.5 (2026-10-21). — [vite-plugin-wasm README](https://unpkg.com/vite-plugin-wasm@3.4.1/README.md); [npm vite-plugin-wasm](https://www.npmjs.com/package/vite-plugin-wasm)
- wasm-pack `--mode no-install` builds bindings "without installing `wasm-bindgen`"; `normal` uses an installed `wasm-bindgen`; wasm-opt gets `--enable-memory64` for wasm64 and `--enable-exception-handling` for `--panic-unwind`. — [wasm-pack build docs](https://wasm-bindgen.github.io/wasm-pack/book/commands/build.html)
- The `wasm-opt` crate (Rust bindings to binaryen) is at 0.116.1, last published 2024-03-31. — [crates.io wasm-opt](https://crates.io/crates/wasm-opt)
- Task runners: just 1.59.0 (2026-10-15), cargo-make 0.37.24 (2025-01-18), trunk 0.22.0 (2026-10-20). — [crates.io just](https://crates.io/crates/just); [crates.io cargo-make](https://crates.io/crates/cargo-make); [crates.io trunk](https://crates.io/crates/trunk)

### Inferences
- Minimal pinned build without wasm-pack, which an agent can read in full:
  ```
  # rust-toolchain.toml
  [toolchain]
  channel = "1.91.0"          # whatever stable is pinned; >= 1.88 for ts-rs 12
  components = ["rustfmt", "clippy"]
  targets = ["wasm32-unknown-unknown"]
  profile = "minimal"
  ```
  ```
  # crates/sim-wasm/Cargo.toml
  wasm-bindgen = "=0.2.129"
  serde-wasm-bindgen = "=0.6.5"
  ```
  ```
  # justfile
  wasm-bindgen-version := "0.2.129"
  build-wasm:
      cargo build -p sim-wasm --release --target wasm32-unknown-unknown
      wasm-bindgen --target bundler --typescript --out-dir packages/sim-wasm/pkg target/wasm32-unknown-unknown/release/sim_wasm.wasm
      wasm-bindgen --target nodejs    --typescript --out-dir packages/sim-wasm/pkg-node target/wasm32-unknown-unknown/release/sim_wasm.wasm
  check: fmt clippy test-rust gen-types check-types build-wasm test-ts lint-ts typecheck-ts
  ```
  CI installs the CLI with `cargo install wasm-bindgen-cli --version 0.2.129 --locked` (or a prebuilt binary via `taiki-e/install-action`); because 0.2.128+ hard-fails on schema mismatch, a mismatch is loud rather than subtle, which is what you want.
- wasm-pack is still fine (it resolves and caches the matching CLI for you), but its 332 open issues and the `package.json` shape problems in Node make the two-line wasm-bindgen invocation preferable for a repo an agent maintains. Trunk is aimed at whole-app Rust frontends (Yew/Leptos) and is a poor fit when the viewer is TS/Vite.
- wasm-opt: binaryen is not on crates.io at a current version; install `binaryen` via the distro package or `taiki-e/install-action` in CI and run `wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int` only in the release job, with a size budget asserted by a script (`stat -c%s pkg/*.wasm` compared against a committed number). Track `[profile.release] opt-level = "z"|"s", lto = true, codegen-units = 1, panic = "abort"`.
- Build-time expectation for a small crate: cold CI build of a few-thousand-line sim with serde + wasm-bindgen is dominated by dependency compilation; rust-cache cuts that to the crate itself. No source gives numbers; treat as unmeasured.

### Gaps
- No source states the Rust 1.82/1.87 wasm32 C-ABI change and its wasm-bindgen minimum; the changelog portion fetched did not mention it. Anyone pinning an old wasm-bindgen with a new Rust should verify this separately.
- No source documents how wasm-pack 0.15 resolves the wasm-bindgen-cli version (reads Cargo.lock and downloads a matching binary is the historical behaviour, not verified for 0.15).
- No benchmark of wasm build times or binaryen version pins was found.

## 6. Repo layout for mixed Rust/TS monorepos

### Takeaway
The pattern seen in real repos is `crates/` (cargo workspace) beside `packages/` (pnpm workspace), the wasm crate's generated `pkg/` consumed as a workspace package, and generated TS types committed in a dedicated directory so diffs show drift. sandspiel uses the older single-crate `crate/` + `js/` + root `package.json` layout.

### Cited Findings
- sandspiel layout: `assets/`, `crate/` (Rust), `functions/`, `js/`, root `package.json`, `webpack.config.js`, `.babelrc`, both `pnpm-lock.yaml` and `yarn.lock`; JS imports the generated package relatively: `import { Universe } from "../crate/pkg";`. — [GitHub MaxBittker/sandspiel](https://github.com/MaxBittker/sandspiel); [sandspiel js/index.js](https://raw.githubusercontent.com/MaxBittker/sandspiel/master/js/index.js)
- Bitwarden SDK keeps a dedicated crate `bitwarden-wasm-internal` with `src/custom_types.rs` for hand-declared TS types, and wraps wasm-bindgen/tsify behind its own `bitwarden_ffi` macros (`wasm_export`, `wasm_object`, `wasm_record`). — [Bitwarden SDK interoperability](https://contributing.bitwarden.com/architecture/sdk/web/interoperability)
- ts-rs writes to `./bindings` by default, overridable via `TS_RS_EXPORT_DIR` set in `.cargo/config.toml` `[env]`, with `#[ts(export_to = "dir/")]` for per-type placement and imports generated between files. — [docs.rs ts-rs](https://docs.rs/ts-rs/latest/ts_rs/); [trait TS](https://docs.rs/ts-rs/latest/ts_rs/trait.TS.html)
- ts-typegen writes one `.ts` per type plus `index.ts` into the configured output and expects the files committed so `TS_TYPEGEN_CHECK=1` can diff. — [docs.rs ts-typegen](https://docs.rs/crate/ts-typegen/1.2.0)
- Swatinem/rust-cache `workspaces` input accepts `$workspace -> $target` entries for non-root cargo workspaces. — [GitHub Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)
- wasm-pack `bundler` target emits `package.json` with `module` and `sideEffects: false`; `nodejs` target emits `main`. — [wasm-pack build docs](https://wasm-bindgen.github.io/wasm-pack/book/commands/build.html)

### Inferences
- Suggested layout consistent with the above:
  ```
  Cargo.toml                 # [workspace] members = ["crates/*"]
  rust-toolchain.toml
  .cargo/config.toml         # [env] TS_RS_EXPORT_DIR = { value = "packages/protocol/src/generated", relative = true }
  justfile
  crates/sim/                # pure Rust, no wasm deps, insta + proptest tests
  crates/sim-wasm/           # thin #[wasm_bindgen] facade over crates/sim
  packages/protocol/         # committed ts-rs output + hand-written index.ts, zod schemas if any
  packages/sim-wasm/         # pkg/ (bundler) and pkg-node/ (nodejs) from wasm-bindgen; package.json "name": "@game/sim-wasm"
  packages/viewer/           # Vite app, depends on @game/protocol and @game/sim-wasm (workspace:*)
  fixtures/                  # golden event streams, snapshots, hashes, read by both test suites
  pnpm-workspace.yaml        # packages: ["packages/*"]
  ```
  Commit `packages/protocol/src/generated/**` (so drift is a diff) and gitignore `packages/sim-wasm/pkg*/` (binary output rebuilt by `just build-wasm`; CI builds it before `tsc`).
- Keep `crates/sim` free of `wasm-bindgen` so `cargo test` runs natively and fast; only `crates/sim-wasm` depends on wasm-bindgen/serde-wasm-bindgen. This also makes the native-vs-wasm hash comparison trivial.
- Workspace versioning: one `PROTOCOL_VERSION` in `crates/sim`, re-exported through the facade and into generated TS; semver on npm/cargo packages is irrelevant inside a private monorepo.

### Gaps
- No Tauri/Dioxus/Yew repo was inspected directly in this research for a `crates/` + `packages/` layout; the layout above is a synthesis of tool defaults and the two inspected projects, not a documented convention.

## 7. Agent-specific practice for mixed-language repos

### Takeaway
No published, citable post-mortems from 2025-2026 about AI coding agents working specifically across Rust/TS wasm boundaries were found; GitHub code search for CLAUDE.md/AGENTS.md files mentioning wasm-bindgen was unavailable in this session. What exists is tooling whose design matches agent needs: committed generated types, single check commands, CI-fails-on-drift modes.

### Cited Findings
- Prompt templates and "skills" for Rust-to-wasm exist (e.g. a skill that triggers on "WebAssembly, WASM, WASI, wasm-bindgen, Rust to WASM, wasm-pack"), with small adoption (one listing shows 61 all-time installs); they contain generic advice (strip debug info, wasm-opt, LTO; "Wasm modules operate on a flat byte array that grows but never shrinks"), not outcomes. — [agentskills discover-wasm](https://agentskills.so/skills/rand-cc-polymath-discover-wasm); [skillselion rust-wasm](https://skillselion.com/skills/pluginagentmarketplace/custom-plugin-rust/rust-wasm)
- Drift-check modes designed for CI exist in ts-typegen (`TS_TYPEGEN_CHECK=1`), rspyts-cli (`check`), typebridge-cli (`check --ci`). — [docs.rs ts-typegen](https://docs.rs/crate/ts-typegen/1.2.0); [docs.rs rspyts-cli](https://docs.rs/crate/rspyts-cli/0.2.0); [docs.rs typebridge-cli](https://docs.rs/typebridge-cli/)
- insta defaults to `INSTA_UPDATE=no` under CI, so snapshot changes must be accepted explicitly by a human or agent locally. — [docs.rs insta](https://docs.rs/insta/latest/insta/)
- wasm-bindgen 0.2.128+ hard-fails on crate/CLI schema mismatch (0.2.129 improved the message). — [CHANGELOG](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)

### Inferences
- Guardrails implied by the tooling, suitable for a CLAUDE.md/AGENTS.md in this repo:
  1. "Run `just check` before committing; it runs fmt, clippy, cargo test (which regenerates `packages/protocol/src/generated`), `git diff --exit-code` on that directory, the wasm build, vitest against the real wasm, eslint and tsc." One command, deterministic order, fails fast on the boundary.
  2. "Never hand-edit `packages/protocol/src/generated/**`; change the Rust type and re-run." Committed generated files make drift visible in the diff the agent and reviewer both see.
  3. "Boundary types live in `crates/sim/src/protocol.rs`; they derive `Serialize, Deserialize, TS, JsonSchema` and must not use `#[serde(with)]`/`serialize_with`" (source-scanning generators cannot follow custom serializers).
  4. "If a snapshot or fixture changes, say why in the commit; run `cargo insta accept` only after reading the diff." insta's CI default enforces the second half automatically.
  5. "Pinned versions: Rust in `rust-toolchain.toml`, `wasm-bindgen` crate and CLI must be identical (0.2.129); do not bump one without the other."
  6. "Determinism rules: no `std::collections::HashMap` iteration in `crates/sim`, no `js_sys`/time/random in `crates/sim`, seed from input only; `just check` compares native and wasm `hash()` on the fixtures."
- The agent-friendliness ranking for type generation follows from how much is visible as text: ts-rs/ts-typegen (committed `.ts`, diffable) > schemars JSON (committed, diffable, also validates content) > tsify (types appear inside the build-generated `.d.ts`, usually gitignored, so drift surfaces only as a `tsc` error after a rebuild) > typeshare (extra CLI and config, multi-language focus).

### Gaps
- No first-hand reports from 2025-2026 on agents working across Rust/TS wasm boundaries were found through web search; GitHub code search (for CLAUDE.md/AGENTS.md mentioning wasm-bindgen or ts-rs) returned 403 in this session, so no real-world agent instruction files were sampled.

## 8. Content data: serde formats, RON status, schemars for validation

### Takeaway
RON is actively maintained again (0.10.1 Apr 2025 through 0.12.2 Jun 2026) but has no official schema language; the practical path is RON or JSON for authoring, `schemars` 1.2.x to emit JSON Schema from the same Rust content structs, and `json-schema-to-typescript`/`json-schema-to-zod` or editor JSON-Schema support for TS/agent-side validation. Validation at load happens in Rust via serde; schema output is for tooling and for authoring-time feedback.

### Cited Findings
- `ron` releases: 0.8.1 2023-08-17 (then a long gap, 0.9.0-alpha.0 Mar 2024), 0.10.1 2025-04-08, 0.11.0 2025-08-27, 0.12.0 2025-11-12, 0.12.1 2026-03-30, 0.12.2 2026-06-22; ~122.8M downloads. — [crates.io ron](https://crates.io/crates/ron); [lib.rs ron](https://www.lib.rs/crates/ron)
- Third-party RON schema tooling: `ron-schema-cli` 0.10.0 ("RON has no equivalent of JSON Schema. This project fills that gap") using a custom `.ronschema` format, "Schema parser, RON parser, validator, and CLI are all functional with test coverage". — [docs.rs ron-schema-cli](https://docs.rs/ron-schema-cli/)
- `ron2` is a separate parser (not the official crate): releases 0.1.0 2026-01-20 to 0.3.0 2026-01-25; ecosystem includes derive macros for FromRon/ToRon and schema generation, a language server and a doc generator from schema files. — [lib.rs ron2](https://lib.rs/crates/ron2)
- insta supports `assert_ron_snapshot!` natively (feature `ron`), so RON is also a convenient snapshot format. — [docs.rs insta](https://docs.rs/insta/latest/insta/)
- schemars 1.2.3 (2026-10-12), MSRV 1.74; 1.0 moved to plain `serde_json::Value` schemas; generated-schema structure may change between versions and that is "not considered a breaking change". — [crates.io schemars](https://crates.io/crates/schemars); [docs.rs schemars](https://docs.rs/schemars/latest); [Zed migration commit](https://git.secluded.site/zed/commit/5fafab6e52f87cd5d6235628b12e6c62a2db0fd4)
- json-schema-to-typescript 16.0.0 (2026-08-28); json-schema-to-zod 2.8.1 (2026-04-01), draft 4+; zod 4.6.7. — [npm json-schema-to-typescript](https://www.npmjs.com/package/json-schema-to-typescript); [npm json-schema-to-zod](https://www.npmjs.com/package/json-schema-to-zod)
- specta lists a JSON Schema exporter as "Experimental". — [GitHub specta-rs/specta](https://github.com/specta-rs/specta)

### Inferences
- If content files are RON, there is no mainstream editor/agent validator for them; the agent's validation loop is `cargo run --bin validate-content` (serde deserialize + domain checks) with good error messages. If content files are JSON or YAML, a committed `schemas/content.schema.json` from schemars gives VS Code/agents inline validation and lets a TS tool (e.g. a content browser in the viewer) validate with zod generated from the same schema. For an agent-authored project, JSON/YAML + schemars is the lower-friction choice; RON's advantage (enums and tuples look like Rust) matters mostly for human authors.
- Keep content structs and protocol structs separate but both `#[derive(JsonSchema)]`; commit the generated schema and add `git diff --exit-code schemas/` to `just check` so schema changes are reviewed like generated TS.
- Because schemars does not guarantee stable output across versions, pin it exactly (`schemars = "=1.2.3"`) or the drift check will fire on dependency bumps.

### Gaps
- No source describes RON's own plans for a schema language; the only schema tools are third-party (`ron-schema-cli`, `ron2`), both small.
- No source found that validates the same schemars schema in both Rust (at load) and TS; the Rust side validates by deserializing, not by schema, and that asymmetry is inherent.
