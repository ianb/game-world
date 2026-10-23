# Multiplayer and external actors

Status: design note, 2026-10-23. Not scheduled. Written so the single-player
build does not close doors. Covers live multiuser and the same boundary
used for LLM dialogue and other external actors.

## What the current architecture already gives

The sim is deterministic, steps on integer ticks, takes an ordered input log
and emits typed events. That is the whole substrate for live multiuser. The
three standard shapes all fit on it:

| Shape | How it works | Costs | Fits when |
|---|---|---|---|
| Server-authoritative, event broadcast | One sim runs on the server. Clients send inputs; the server stamps them with a tick, steps, and broadcasts events. Clients run only the viewer. | Bandwidth scales with events; no client prediction, so input latency is visible. | Latency tolerance is high (a town where you tap to walk). Simplest trust model. |
| Lockstep | Every client runs the same sim. The server only orders inputs and relays them. Determinism across clients is required. | Late joiners need a snapshot; a desync is fatal; cheats are trivial without a referee. | Bandwidth is scarce; the native-versus-wasm hash parity check already in CI is the proof needed. |
| Authoritative plus prediction | Server as above; clients also run the sim ahead on their own inputs and reconcile from server snapshots. | The most code; needs rollback or resync. | Latency matters for feel (action games). |

Recommendation for whenever this is built: server-authoritative with event
broadcast, because the game tolerates latency and the viewer already
consumes an event stream. Clients that also run the sim locally (for offline
play, or later for prediction) use the same crate, so the option stays open.

## The invariant to keep now

The viewer depends on an interface, not on the wasm module:

```
interface Sim {
  stepUntil(tick): SimEvent[]      // or: subscribe(events)
  applyInput(input: PlayerInput)   // input carries playerId; tick is assigned by the sim owner
  snapshot(): Uint8Array
  load(bytes: Uint8Array)
  hash(): string
}
```

`LocalSim` wraps the wasm build. `RemoteSim` wraps a WebSocket. The viewer
and its tests do not know which one they have. Keeping this honest in
single-player is the entire preparation cost.

## Hosting sketch

The Rust crate compiles to wasm, so it can run inside a Cloudflare Durable
Object through workers-rs: one object per town, inputs over WebSocket,
events broadcast to connected clients, state in object storage, hibernation
when nobody is connected. This matches the developer's existing Cloudflare
deployments. A native Rust server (axum) on a small VM is the alternative if
Durable Object CPU or memory limits bite. Either way it is the same crate
with a different host.

## Things this changes in single-player design

- A shared town runs at wall-clock pace. Day length and clock speed, decided
  for single-player, become a server setting.
- "Sleep to skip the night" is a single-player convenience. A shared town
  needs something else (nothing, or a vote, or the night is short).
- Villager dialogue with an LLM, if it comes, runs server-side where the
  key lives, and its outputs enter the sim as inputs like everything else.
- Save files are snapshots plus input logs; a server has the same two
  things.

## External actors use the same door

An LLM for dialogue, a drama director, or any future slow or
nondeterministic component is an actor outside the sim: it subscribes to
events and submits inputs, like a player. The sim stays synchronous and
pure, and replay stays deterministic because the actor's responses are in
the input log.

```
            events                          events
  Sim  ───────────────►  Viewer      Sim  ───────────────►  Villager brain (LLM)
   ▲                                  ▲                        │
   └──── inputs (playerId) ───────────┘◄── inputs (actorId) ───┘
```

Protocol consequences:

- Inputs carry provenance: `PlayerInput { source: Player(id) | Actor(id), ... }`.
  The sim can apply different rules per source (a brain may only speak for
  its own villager; a director may only schedule exchanges).
- The sim emits request events when it wants help: `LineNeeded { villager,
  situation, facts, deadlineTick }`. The actor answers with an input. If the
  answer arrives after the deadline or no longer fits, the sim drops it and
  falls back to an authored line.
- The event stream must carry enough context for an actor to act without
  reading sim state directly, which also keeps the wire format complete for
  remote viewers.
- A scripted brain and a model-backed brain are interchangeable behind the
  same actor interface, so "mostly scripts, LLM for high-plot moments" is a
  routing decision, not an architecture change.

## Open decisions, when scheduled

1. Server-authoritative or lockstep (recommendation above).
2. Durable Object or native server.
3. Whether a shared town is persistent across days or sessions are rooms.
4. Identity and presence (who is who in the town, what other players look
   like to villagers).
5. Where external actors run: in the browser beside the viewer (single
   player, key exposure concerns), or on the server beside the sim.
6. Deadline and fallback policy for request events.
