# M9 plan: multiplayer

Drafted by the lead session on 2026-10-21, under the lead's direction to work autonomously through M9.
PLAN.md §4.8 and §5 give the scope: a relay (also the lobby), lockstep, desync reports, then WebRTC.
The exit check is a 2v2 across networks for 30+ minutes without desync, and a forced desync that
produces a readable diff.

Multiplayer here is our own engine's networking. It is not compatible with the original's clients (PLAN
§6.1 tier c), so the protocol, the lobby and the desync tooling are our own design and need no spec. Game
rules that only network games have are another matter. S-0412 leaves network games out, so where a rule
is the original game's behaviour (for example, alliances changed during play), a step files a request
and does not guess.

## Decisions

- **Hosting:** the relay is built and tested on this machine (loopback, several clients, AI players over
  the network, simulated loss and delay). The lead runs the real cross-network 2v2 on a host of their
  choice, from the guide written in step N8.
- **Sim boundary:** the simulation crates stay free of I/O (CLAUDE.md). Networking lives in `glasnost-net`
  (protocol, lockstep, no sockets), a new `glasnost-relay` server binary, and the client's transports
  (native WebSocket, browser WebSocket, later WebRTC).
- **Our AI in network games:** a computer player's orders come from `glasnost_ai`, which is deterministic
  inside the tick. Every peer runs it, and its orders never cross the network.
- **A player who leaves or drops:** the house surrenders by an order that every peer applies on the same
  tick: S-0414 claim 11's surrender, as in skirmish. A reconnect within the grace period (N5) cancels the drop
  before that tick. This is our own network rule, not a claim about the original.
- **Versions:** peers must have the same build, the same `sim_rules_hash` and the same map content hash
  (`session::content_hash`). The lobby refuses a mismatch and names it. Replays record the build.
- **Dependencies:** only GPL-3-compatible crates (`deny.toml`), for example tokio and tungstenite (MIT),
  and matchbox (MIT or Apache-2.0) for WebRTC.

## Request block

Q-1800 to Q-1819 (network-only game rules, if any).

## Steps

| Step | Work | Exit check |
|---|---|---|
| N1 | **Lockstep core** (`glasnost-net`, no I/O). Frames: each peer sends its orders for tick N + delay, and an empty frame when it has none. Tick N runs only when every peer's frame for N is in, merged in slot order. Adaptive delay from measured round trips. A state hash every K ticks. Session start: build, rules hash, map hash, seed and slots. An in-memory transport with seeded latency, jitter, loss and reordering for tests. | Unit and property tests: 2 to 8 peers under bad links reach identical per-tick hashes; a stalled peer stalls all and none run ahead; the delay adapts both ways. |
| N2 | **Relay and lobby** (`crates/relay`, a native binary). WebSocket rooms with a room code, player slots, the setup values of S-0412 (map, countries, colours, teams, starts, computer players), ready states and start. It forwards frames, sends keepalives, and reports a drop to every peer. | Integration tests on loopback: create, join, leave, start, forward, drop. A mismatched build or map is refused with a reason. |
| N3 | **Client integration.** A *Multiplayer* entry on the main menu: host (create a room) and join (a room code), a lobby screen built on the skirmish setup, then a game paced by lockstep (the `Pacer` waits for frames; a "waiting for player" notice after a short stall). Native and browser WebSocket transports. Chat in the lobby and in the game. | `game::client` tests on a synthetic install with an in-process relay; a local-lane run of two native clients on loopback. |
| N4 | **Desync reports.** On a hash mismatch, every peer keeps the last frames and a snapshot from before the mismatch and sends its hash to the relay. A `glasnost-headless desync` command replays both sides from the snapshot and prints a readable per-object diff (the first differing tick, object and field). A debug switch perturbs one client, for tests only. | A forced desync test produces a diff that names the perturbed object and field. |
| N5 | **Reconnect and late join** from a snapshot (save/load already round-trips the state hash): the relay asks a peer for a snapshot, the rejoining client loads it and replays the frames since. A drop grace period comes before the surrender order. | Tests: drop and rejoin mid-game with identical hashes after; past the grace period, the house surrenders on every peer on the same tick. |
| N6 | **Soak harness.** A headless client (`glasnost-headless net-client`) that plays our AI as a "human" player over the relay. A local 2v2 of four such clients plays 30+ minutes of game time under simulated loss and delay, natively and with at least one wasm client (node or a headless browser). | Four-client 2v2 to a winner or 30+ minutes, with no desync, in the local lane. The determinism gate is unchanged. |
| N7 | **WebRTC** through matchbox, with the relay as signalling server and as the fallback when a direct link fails. | Tests with the fallback forced and not forced; the soak of N6 also over WebRTC on loopback. |
| N8 | **Guide and exit run.** `docs/multiplayer.md`: build and run the relay, open it to the internet, play a 2v2 across networks, collect a desync report. The lead runs the cross-network game. | The lead's 2v2 runs 30+ minutes without desync (the lead's run; the "Needs you" list tracks it). |

N1 comes first. N2 and N4 can follow in parallel. N3 needs N1 and N2. N5 and N6 need N3. N7 comes after N6.
