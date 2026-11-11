# Multiplayer development notes

Moved from [development.md](../development.md) on 2026-11-07.

## Multiplayer (M9)

M9 adds multiplayer by lockstep: every peer runs the whole game from the same start values, and the peers exchange only the persons' orders, their delay proposals and state hashes. Rules data and game state never cross the network. Our AI plays the computer slots inside the tick on every peer, so a computer slot's orders never cross the network either. The protocol, the frame rule, the tick rule and the delay rule below are our own netcode design; no spec governs them, and their constants are marked `// nocite: our own netcode`. Nothing in the game rules changes, so the step needs no spec request. Plan: [m9-plan.md](../m9-plan.md).

### The lockstep core (N1)

The crate `glasnost-net` holds the protocol and one peer's lockstep session. It does no I/O: it reads no clock, opens no socket and starts no thread. The caller passes its clock in milliseconds into every call and moves the bytes. `crates/net/clippy.toml` forbids clocks, threads, locks, sockets, files and hashed containers in the crate, and `xtask deps` forbids it every socket or async runtime crate (the list `FORBIDDEN_EXTERNAL`, which now also names the WebSocket, WebRTC and HTTP crates). The simulation crates' determinism lint now also forbids `std::net::TcpListener`, with a canary.

**Wire format** (`wire.rs`). A message is a 4-byte header, then a postcard body. The header holds the magic `GN`, the protocol version (`WIRE_VERSION`, now 5: N2 reserved the sender id 255, `wire::RELAY`, for the relay, N3 added chat lines to the lobby messages that share the version, N4 added the desync report's digest to the frames message, and N5 added `Body::Serve`, `Body::Resume`, `Frame.resume`, the grace period in the start values, and the lobby's `Rejoin`, `Snapshot`, `CaughtUp`, `Rejoined`, `Begin.ticket` and new refusals) and the sender's slot. A receiver reports a message with another version as a refusal on the field *protocol*. Trailing bytes, a short header or a body that does not decode are reported as `Event::BadMessage`, and never cause a panic (`hostile_bytes_never_panic`). The body is one of three kinds:

- `Start`: the sender's start values.
- `Frames`: its frames not yet acknowledged by every live peer, and its checkpoint hashes not yet acknowledged. The message also carries the ticks it has run, its acknowledgements of every other slot's frames and hashes (how many it holds from 0 without a gap), its time stamp, an echo of every other live peer's latest stamp with the time it held that stamp (since N7 only a live peer's: a dropped slot's stamps belong to a process that may be gone), and, once it has written one, its desync report's tick and digest (N4).
- `Dropped { slot, kept }`: the relay's report that a slot left (N2). A session acts on it only from the sender `RELAY`; from any other sender it is a bad message, as is any other body from `RELAY`.

Each message carries the whole unacknowledged window, so a lost message costs nothing once a later one arrives. A receiver keeps the first copy of each (slot, frame) and ignores duplicates. It also ignores frames beyond the window: two largest delays past its own tick.

**Session start** (`start.rs`). The start values are the build id, the rules hash, the map hash, the seed, the slot table, the hash of the whole `GameSetup`, and the lockstep parameters: the initial, smallest and largest delay, and the checkpoint period. The slot table lists the player houses in house order, each marked person or computer. The caller supplies the build id (N3: the client's version string) and the map hash (the `content_hash` string of the map file, `glasnost_net::start::content_hash` since N4). The setup hash is SHA-256 over the setup's postcard bytes. Each peer resends its start until it has heard frames from every other peer. It starts running once every other person's start matches its own. On the first difference it stops, with `Phase::Refused` and `Event::Refused { slot, field }`. The protocol version is checked first, in the header; the start's fields are then compared in a fixed order (build, rules hash, map hash, seed, slots, setup, parameters), so the reported field is the first that differs. A start from a slot that is not another person's slot is refused on the field *slot* (`a_mismatched_start_is_refused_with_the_field_named`).

**Frames and the tick rule** (`session.rs`). Frame *n* of a person's slot is that person's input for tick *n*. After *k* ticks with delay *D*, a peer has made its frames 0 to *k* + *D* − 1. The person's orders since the last frame go into the first new frame, and the other new frames are empty. A frame never changes once made. Tick *n* runs only when frame *n* of every live person's slot is in. Its orders are issued slot by slot in slot order, and in the order the person gave them within a slot; then the game steps. So whatever order the frames arrive in, every peer issues the same orders in the same order (`merged_orders_apply_in_slot_order_whatever_the_arrival_order`). No peer can run more than the delay ahead of the slowest peer, and a stalled peer stalls everyone (`a_stalled_peer_stalls_all_and_none_runs_ahead`). The session's `step` runs at most one tick when the caller's pacer says one is due. It returns the tick's merged orders (`Tick`), and for each order whether the game took it.

**The delay** (`delay.rs`). Each peer measures the round trip to every other peer from the echoes of its own stamps, minus the time the other peer held each stamp. The smoothed estimate and its variation are integers, using the RFC 6298 estimator. A sample below the estimate counts an eighth as much in the variation, so a link that gets faster does not raise the timeout. Only an echo newer than the last one sampled is used, and only if its round trip is plausible (at most 60 s); since N7 an implausible one moves nothing, so an echo of a restarted process's old stamps cannot make every later echo look old. From the largest timeout (smoothed + 4 × variation), the peer works out its *need*: ⌈timeout / (2 × tick period)⌉ + 1, a one-way trip and a tick to spare. It writes a proposal into every frame it makes, as follows:

- The need raises the proposal at once.
- A wait for a frame since the last change took effect raises the proposal to at least the delay + 1, once per wait (since N7: at the largest delay the raise changes nothing, and a wait that counted again on every proposal held the delay there to the end of the game).
- A need below the peer's own last proposal lowers it half the way down (at least one step). This happens only after `calm_ticks` ticks with no wait, no change and no lowering of its own; until then the peer repeats its last proposal.
- The proposal is clamped to the smallest and largest delay.

The peer repeats its own proposal, not the session's delay. Otherwise another peer's one-off proposal would be echoed back one delay later, and that echo would keep changing the delay and never let it fall. A wait counts only when the waiting peer is not ahead of the others: `Session::ahead` estimates the lead over the slowest peer from its reported ticks, the time since and half a round trip, and a leading peer always waits. The session's delay from tick *n* + 1 on is the largest proposal among the frames of tick *n*. Every peer holds exactly those frames when it runs *n*, so every peer switches on the same tick without any further message (`the_delay_adapts_both_ways_and_switches_on_the_same_tick_everywhere`, `late_frames_raise_the_delay`).

**State hashes.** After every `hash_every` ticks a peer takes the game's state hash as a checkpoint, and sends it until every peer acknowledges it. Each peer compares the others' checkpoints with its own. A difference is reported as `Event::Desync { tick, mine, theirs }`, with the slots and both hashes (`a_desync_is_reported_with_tick_slots_and_hashes`).

**Seams for the later steps.**

- `Transport` and `pump` connect a session to a network. `simnet::SimNet` is the seeded in-memory network for tests, with per-link latency, jitter, loss, duplication, reordering, stalls and cuts.
- `Session::drop_slot` and `Body::Dropped { slot, kept }` handle a departure. The relay (N2) chooses `kept`: the number of that slot's frames it forwarded to everyone. Every peer then treats that slot's frames from `kept` on as empty. Since N2 the session honours a `Dropped` only from the sender `RELAY` (255), and the relay never forwards a client's `Dropped` or a message whose sender byte is not the client's own slot, so only the relay can report a drop (N7 must keep this over WebRTC, see *The relay and lobby (N2)*). A drop that contradicts a tick already run is reported as a bad message.
- `Event::Silent` and `Event::Heard` report a peer that has gone unheard for `silent_ms` and its return (for N3's notice and N5's rules). `Session::waiting_for` names the slots the next tick waits on, for the "waiting for player" notice.
- `Session::ahead` is for N3's pacer, which may stretch its tick while this peer leads.
- `Event::Desync` is for N4's desync report (done: see *Desync reports (N4)*).
- `sim::Simulation` is what the session drives. It is implemented for `glasnost_ai::game::Game`.

**Tests.** `crates/net/tests/lockstep.rs` covers the protocol on a toy game whose state is a hash chain of every order and step, so any difference in what was applied shows. One sweep runs 24 seeds of 2 to 8 persons and 0 to 2 computer slots, in shuffled slot order, on bad links (loss, duplication, reordering, jitter); every tick's hash matches a single-process run fed the same orders. `crates/net/tests/games.rs` does the same with real games built on the synthetic fixture rules: 2, 4 and 8 persons, with computer slots and persons played by our own AI's brain or by a script, all on bad links. Each peer's hash after every tick must equal the hash of one game fed the merged orders, and the games' replays must agree.

### The relay and lobby (N2)

The relay is the native server players meet through: they make or join a room by its code, agree on the game setup in the room and start, and then the relay forwards each person's lockstep messages to the others. It never runs the game and holds no game state. The protocol, the room rules, the drop rule and the limits are our own netcode design (S-0412 does not cover network games); the setup's choice rules cite S-0412 and S-0009 as the setup screen does, and the house order follows S-0009 claim 19, with random colours drawn first (below; Q-1805, answered by S-0009 revision 44).

**Crates.** The protocol and the room state machine are in `glasnost-net` (`crates/net/src/lobby/`), with no I/O, so that N3's clients share them. The server is the new crate `glasnost-relay` (`crates/relay`): `hub.rs` is its core with no I/O (connections, rooms, forwarding, drops, limits; the caller's clock in milliseconds goes into every call, host randomness comes from an `Entropy`), and `server.rs` is the tokio shell (tokio, tokio-tungstenite, futures-util, getrandom). The relay is native only. `xtask deps` forbids every simulation crate and `glasnost-net` the relay and its runtime crates (the futures crates joined `FORBIDDEN_EXTERNAL`), and forbids the client crates (`glasnost-app`, `glasnost-platform-native`, `glasnost-platform-web`) the relay; the test `only_the_relay_reaches_the_async_runtime` checks the real graph. CI's wasm32 clippy has an explicit `-p` list without the relay and net.

**Messages.** One WebSocket binary message is one of our messages, told apart by its magic. Lobby messages (`lobby::wire`) have a 3-byte header (`GL` and `WIRE_VERSION`, shared with the lockstep wire) and a postcard body; lockstep messages are N1's (`GN`). A client sends `ToRelay`:

- `Create { name, build, rules_hash, offered, map, setup }`: a new room with the host's build, rules hash, the values its sliders offer (`Offered`: countries, credit and unit ranges), its map (`MapChoice`: hash, label, start positions) and its setup (the setup screen's `Setup`).
- `Peek { code }`: the room's `RoomInfo` (build, rules hash, map, persons, open rows, playing), before joining.
- `Join { code, name, build, rules_hash, map_hash }`, `Leave`.
- `Edit(Edit)`: the map; a row's kind; a row's choices (country, colour, team, start); the game-wide values (credits, units, flags, speed).
- `Ready { ready, map_hash }`, `Start`.

The relay answers with `FromRelay`: `Room(RoomView)` to every member after every change (the setup, the members with their rows, names and ready marks, the host, the receiver's own id), `Info`, `Refused(Refusal)`, `Begin` at the start and `Closing(reason)` just before it closes a connection. A refusal names its reason: `Mismatch(field)` names the start field (`Field::Build`, `RulesHash`, `MapHash`, `Protocol`) whose value differs from the room's; the others are no room, room full, too many rooms, playing, not in a room, already in one, not the host, not yours (row), seated (row), no such row, a bad value (`Value`), a setup problem (`Problem`), an open row and a person not ready.

A room code is 8 characters of Crockford's base 32 (`CODE_ALPHABET`, 40 bits) drawn from the relay's host randomness and drawn again on a collision; codes are read loosely (case, dashes and spaces, I and L for 1, O for 0; `normalise_code`).

**Room rules** (`lobby::room`, documented at the top of `room.rs`). The setup's rows are its eight slots; a person's row is taken by one member or open; only the map's rows are used. The host takes the first person's row, a joiner the first open one, so the number of players in a room is capped by the setup's eight rows (and the map's start positions), not by a separate limit. A join is refused, naming the field, when its build, rules hash or map hash differs from the room's, in that order. The host edits the map, the kinds of untaken rows, the computer rows' choices and the game-wide values; each person edits its own row's choices; the room enforces the setup screen's choice rules. Every edit, join and leave clears every ready mark. The host starts when every person's row is taken, every other member is ready for the room's map hash and the setup has no problem (open rows are reported first). A member who leaves before the start opens its row with its choices on random; the host's role passes to the member on the lowest row; an empty room is gone. A room that has begun refuses joins (`Playing`).

**The start.** The relay draws the seed from its host randomness and sends every member the same `Begin` (build, rules hash, map, seed, setup, slot table, rows, names, lockstep parameters) with its own slot (`you`). The slot table lists the occupied rows in house order (`Setup::house_order`, S-0009 claim 19: persons by increasing colour slot, the earlier row first on a tie, then computers in row order). Before the order is fixed, `Setup::resolve_colours` gives every row on random a free colour slot, persons and computers alike, in increasing row order (S-0412 claim 7; the order is our design, S-0009 claim 19). A person on random is then placed by its given slot. The resolved setup is the one sent in `Begin`, and the room's own setup keeps its choices on random. Each client builds its game from `Begin`: `Begin::local_setup` with its own index of the map, `Setup::lobby`, `glasnost_ui::setup::game_setup` with the seed as both seeds, then `Begin::start_info`, which refuses a game setup whose slot table or seed is not the room's; then N1's `Session::new`.

**Forwarding.** Once its room has begun, a person's lockstep message goes byte for byte to every other live person of the room, in the order it came. The relay discards, and counts (`Stats`), a lockstep message from a connection outside a game, one whose sender byte is not the connection's slot (a client speaking for another or as `RELAY`), and any `Dropped` from a client. It does not read further into the message.

**Drops.** When a person's connection closes, fails, breaks a limit, goes silent or sends `Leave` during the game, the relay sends `Envelope { from: RELAY, body: Dropped { slot, kept } }` to every remaining person, queued after everything it forwarded from that slot. `kept` is the number of that slot's frames, counted from frame 0 without a gap, that the relay forwarded: each `Frames` message with `first <= kept` raises it to `first + frames.len()`, and so does every `Frames` message of a person with no other live person in the game, past a gap (N5, *A lone stretch*). Every other live person received exactly those messages in order, so every peer holds the same frames and `Session::drop_slot` agrees everywhere (`a_lost_person_is_reported_dropped_with_the_frames_everyone_holds`). When the last person of a game goes, the room goes.

**Limits** (`Limits`, all `// nocite: our own netcode`):

| Limit | Default | When broken |
|---|---|---|
| `max_message` | 64 KiB | closed, "message too large" (tungstenite's message and frame size are set to it) |
| `max_connections` | 1024 | the new connection is closed, "the relay is full" |
| `max_rooms` | 256 | `Refused(TooManyRooms)` |
| `rate`, `burst` | 200 a second, 400 at once (a token bucket on the hub's clock) | closed, "too many messages" |
| `max_refusals` | 32 refused requests (against guessing codes) | closed, "too many refused requests" |
| `idle_ms` | 120 000 in no room | closed, "in no room for too long" |
| `silent_ms` | 15 000 with no message and no answer to a keepalive | closed, "silent for too long" (a drop in a game) |
| `keepalive_ms` | 2 000 with nothing sent | a WebSocket ping |

The keepalive is sent only to a connection the relay has sent nothing to for 2 s, so a client in a game, which hears frames all the time, stays alive only by sending. A client window that draws no frames therefore keeps its link through N3's network pump (*Without frames: the network pump*; the M9 exit run lost a game to a covered window before it).

Bytes that are no message of ours (a text message, another magic, a body that does not decode) close the connection with the reason; another lobby protocol version gets `Refused(Mismatch(Protocol))` first. Nothing a client sends panics the relay (`random_bytes_never_panic_the_hub`, `hostile_bytes_close_the_connection_with_a_reason_and_the_relay_goes_on`). The server adds: a 10 s WebSocket handshake; at most `max_connections` + 64 connections and handshakes at once (more are dropped at accept); a queue of 256 messages per connection, beyond which the connection is closed as "too slow"; a 5 s limit on each message leaving, so a client that does not read cannot hold its connection's task; and 1 s for a closed client to answer the close. A close sends `Closing(reason)` then a close frame with the same reason (at most 123 bytes).

**Running it.** `cargo run -p glasnost-relay -- --listen 127.0.0.1:8787` (the default address; `--max-rooms N`, `--max-connections N`, `--grace SECONDS` since N5, `--help`). Clients connect to `ws://ADDR/`. The relay speaks no TLS: behind the internet it needs a proxy that does (`wss://`); the native client speaks `wss://` too since N8b (rustls with ring, the system's roots). `--stats SECONDS` (default 60, 0 off) prints one line of counts (N8b); `--version` prints the build id, which names the commit (`glasnost 0.0.0 (a1b2c3d4e)`, `+dirty` for changed tracked files, from `crates/net/build.rs`; N8b).

**Tests.** `crates/net/tests/lobby.rs` covers the room state machine and the lobby messages without a relay. `crates/relay/tests/hub.rs` covers the hub with a seeded stand-in for host randomness: create, peek, join, ready and start; leaving and the host's role; refusals naming the field; code guessing; colliding codes; forwarding order; forged senders and drops; `kept` with gaps and resends; the drop after the forwarded frames; every limit; silence, idleness and keepalives; random bytes. `crates/relay/tests/loopback.rs` runs the real server on `127.0.0.1:0` with WebSocket clients: create, peek, join, leave, start and forwarding; a mismatched build or map refused naming the field; a forged drop never delivered and a real one with its `kept`; hostile bytes, rate, rooms and connections; a client that stops reading dropped as silent. `crates/relay/tests/games.rs` plays three persons (two with a chosen colour, one on random) and a computer over the real relay: each client builds its game from `Begin`, runs N1's session on its own `glasnost_ai::game::Game` with scripted orders, and one peer is cut off at tick 120. The survivors run 300 ticks with the same schedule and hashes as each other, as the cut-off peer as far as it ran and as a single-process run of the same orders; both hear one `Dropped` with the same `kept`, no order of the dropped slot runs at or after `kept`, and nobody sees a desync, refusal or bad message.

**Seams for the later steps.**

- N3 (the clients): the lobby screen drives `ToRelay`/`FromRelay` over a WebSocket transport and builds the session from `Begin` as `games.rs` does; `Transport` over the same socket carries the lockstep messages after `Begin`. The relay's keepalives are WebSocket pings, which browsers and tungstenite answer.
- N5 (rejoin, surrender): a rejoin enters at `Room::join`, which refuses a room that is `Playing`; N5 lets a dropped member back into its own slot by slot identity (the slot of the `Dropped` it was reported in), not into an open row, and must decide how the relay replays the frames since `kept`. A peer that was told `Dropped { kept }` but lacks some of that slot's frames below `kept` waits for them; with honest peers this cannot happen (N1's frame window keeps every frame a peer can be sent), but a hostile client that sends frames far beyond the window and then leaves can stall the others this way, which N5's surrender and N3's waiting notice must cover.
- N7 (WebRTC): peer-to-peer data channels carry no relay in between, so anyone can send `from = 255` there. A `Dropped` must be accepted only from the relay's channel (the signalling WebSocket), never from a data channel.

### The client (N3)

The game client plays network games through the relay (`crates/app/src/game/client_net.rs` and `link.rs`, the UI in `crates/ui/src/netmenu.rs` and `chat.rs`). Everything here is our own netcode or our own layout. The only rules of the original it touches are the setup's house order (S-0009 claim 19, as in N2) and the options menu, focus, speed and active-clock rules of S-0201 claims 15 to 19 (Q-1806, answered).

**Screens.** The main menu gains **Multiplayer**, which opens the multiplayer screen (`Screen::Multiplayer`, `ui::netmenu`). It has three text fields (your name, the relay address, a room code), **Host**, **Join** and **Back**, and a status line. A text field takes the focus on a click and gets the keyboard's text while it has it. Enter moves to the next field, Escape lets go and Backspace deletes. **Host** sends `Create` with the install's build, rules hash and map list. **Join** sends `Join` with the code, normalised to Crockford letters; a code that is not one is refused before anything is sent. A refusal shows on the status line, in the error colour, with the field it names (for example "The relay refused: the build differs from the room's."). A link that closes before the game returns to this screen with the reason.

The room (`Screen::Room`) is the skirmish setup in a frame titled with the room code (`ui::setup::view_framed`, `RoomFrame`). It shows each row's member and whether they are ready. The host edits the map, the options and every row; a guest edits only its own row. Every edit is sent as `Edit`, and the screen shows the relay's `RoomView`, never a local guess. A guest's button reads **Ready** or **Not ready**. The host's button reads **Start game** and is offered once every other member is ready. A note below the rows says what the room waits for, or that you lack the room's map. A chat box below the panel shows the last lines and has a **Say** field. **Leave** (and **End** in the game) sends `Leave` and closes the link.

**The start.** On `Begin` the client builds the game as a skirmish would (`Job::Start`), with the room's setup, the relay's seed and `local: Some(the house of its row)`, so each person plays its own house. When the game has loaded, the client checks the build, rules hash and map hash against the room's (`StartInfo`). If they match, it starts a `glasnost_net::Lockstep` at the network period of the room's speed (`setup::net_period_ms`). A mismatch on either side is a notice ("The game cannot start: Bea's map hash differs from yours."), and no ticks run.

**Pacing.** Two things pace a network game's ticks at once:

- the client's own pacer, at ⌊1000 / cap⌋ ms for the game-speed index in force, with the cap 60, 45, 30, 20, 15, 12 or 10 ticks per second for indices 0 to 6 (S-0201 claim 17 and F5: 16 to 100 ms);
- the lockstep session, which runs a tick only when every slot's frame for it has arrived (`Session::step_by`).

The person's orders do not reach the world directly. `Play::give` puts them in `play.outbox`, and `net_tick` submits them to the lockstep, which schedules them after its input delay. A client more than one tick ahead of the slowest other peer (`Lockstep::ahead`) stretches its period by half, so the others catch up without a stall. A client that cannot run a due tick for one second shows a banner naming whom it waits for ("Waiting for Bea...").

In a network game the options menu and lost focus never stop the ticks (S-0201 claims 15 and 16; Q-1806 answered, M8 A3 intake 8). While the menu is open, or the window lacks the focus, the machine draws no new battlefield frame; closing the menu redraws it. The menu is refused from the end of the game on, and after a surrender through the menu until the house is defeated (claim 15). While at least two person-played houses, our own among them, are undefeated it offers **Surrender** (keep watching) and **Surrender and leave** (the client leaves once its own surrender order has run); otherwise only **Leave game**, which sends no order (S-0201 r31, intake 24). Its title says no pause and it offers no settings screen. The speed control is offered until the local house is defeated. A change is sent when the menu closes, as `Order::GameSpeed` through the lockstep with the other orders' delay. Every machine sets `World::game_speed` in the tick the order runs, and every machine but the sender's shows "<name> changed the game speed.". From the next cycle the pacer and the lockstep's tick period follow the new index (`Session::set_tick_ms`). Setting the index already in force sends nothing (claim 17). The active clock runs whenever the game does (claim 18). Seams: the relay's rejoin grace and the lockstep's resend interval stay at the start's period; an unfocused machine adds no wait (claim 16, our engine's departure); the notice names the house by its player's name.

**Transports.** A `Link` (`game::link`) is a message pipe to the relay, with `send`, `recv`, `state` (connecting, open, or closed with a reason) and `close`. `GameOptions.connect` chooses which kind.

- Native: one thread per link with a blocking tungstenite socket. It uses a 2 ms read timeout, so it answers the relay's pings and sends queued messages promptly, and takes messages of at most 64 KiB.
- Web (`crates/platform-web/src/socket.rs`): a `web-sys` WebSocket with array-buffer messages, whose callbacks only queue. Messages sent while it connects wait for it.

The client reads its link once per update, on every screen. It sorts lobby messages (`GL`) from lockstep messages (`GN`, `lobby_wire::is_lobby`). Lockstep messages that arrive after `Begin` but before the game has loaded are kept for the session (at most 4096).

**Without frames: the network pump** (fixed after the M9 exit run, `crates/app/src/game/pump.rs`). The window updates the client once per drawn frame. A window that draws none (covered, as by KDE's screenshot overlay on Wayland, or minimised) or a background browser tab (no animation frames) used to update it never: no pings, acknowledgements or frames left, the relay closed the silent connection after 15 s (`silent_ms`), and the house surrendered after the grace period (the exit run's first game). So while a network link is open (`Client::networked`) the window's loop also wakes on a timer (winit's `about_to_wait` with `ControlFlow::WaitUntil`, `Pump::wait_ms`), and when no frame has updated the client for `STALE_MS` (200 ms) it runs the same `Client::update` a frame runs, without drawing (`Pump::between_frames`), and the job a frame would run after it (natively, so a game that begins in a covered window still loads). In a network game the timer is the time to the next tick (`Client::net_wait_ms`, at least 4 ms); on the network screens and while a game loads, 50 ms. While frames come it only wakes when they would be stale, and does nothing, so camera, scrolling, sound, cosmetic animation and the frame bench run once per frame as before. Without a link the loop waits for events as before, so an idle single-player window does not wake. The benchmark is left alone (`--bench` sets no timer), and `--exit-after-frames` counts presented frames, which the pump never makes.

The pump runs the lockstep's ticks too: a peer makes its frames as it runs ticks, so a covered peer that did not tick would stop the game for everyone, and in a network game neither the menu nor lost focus stops the ticks (S-0201 claims 15 and 16); a window nobody sees is the same case. The pacer still runs at most one tick per update and makes up no lost time (S-0201 claim 17). Natively the timer keeps the full pace. A browser throttles a background tab's timers (Firefox to about once a second), so a hidden tab runs about one tick a second and the others wait for it ("Waiting for ..."), but its connection lives on, well inside the relay's 15 s.

**Chat.** `ToRelay::Chat` and `FromRelay::Chat(ChatLine)` are part of N2's lobby protocol. They carry a line to every member of the room, the sender included, both in the room and during the game; a line never reaches a simulation. They changed the lobby layout, so the shared wire version became 3 (4 since N4).

The relay cleans each line: control characters go, the ends are trimmed, and at most 200 characters are kept. It refuses an empty line as bad text. It also refuses lines that come faster than one a second on average, with a burst of five (`Refusal::ChatTooFast`). Those refusals do not count against the refusal limit. This adds the limits `chat_every_ms` and `chat_burst` and the stats `chats` and `chats_too_fast`.

In the game, Enter opens the chat line, Enter sends and closes it, and Escape drops it. The last lines show over the battlefield for ten seconds. Notices (a refusal, a member who left, a desync, a closed connection) are chat lines without a speaker.

**Settings.** No setting is stored (since N5 the client does store the game it can rejoin; see *Reconnect and surrender (N5)*). The relay address comes from `--relay URL` (native) or `?relay=` (web) and defaults to `ws://127.0.0.1:8787/`, the relay binary's default. The name and code are typed each time. The build string is `glasnost <crate version>`.

**Dependencies.** `cargo xtask deps` lets the client crates (`glasnost-app` and the platforms) reach the native WebSocket (tungstenite, with its rand and getrandom) and the browser's (web-sys, js-sys). It never lets them reach a server runtime (tokio, tokio-tungstenite) or the relay crate (`the_client_reaches_the_socket_but_no_server_runtime`). Edges built only for emscripten are skipped (`only_emscripten_dependencies_are_never_built`). The relay is a dev-dependency of the app's tests only.

**Tests.** `crates/app/src/game/client_net_tests.rs` runs a relay in the test process and two windowless clients on the native link, with the synthetic install (`glasnost_ai::fixture`).

- `two_clients_meet_chat_and_play_in_lockstep`: the host makes a room and adds a computer row; the guest joins by the typed code; a chat line reaches both; the guest says ready and the host starts. They play 300 ticks with scripted orders through the order path, and the state hashes are equal at every tick. When the guest stops, the host shows the waiting banner. When the guest goes, the host shows "Bea left the game." and plays on.
- `a_join_of_another_build_is_refused_on_that_field`.
- `an_unreachable_relay_is_reported`.
- The pump, on a relay with a silence limit of 1 s: `a_network_game_plays_on_while_its_window_draws_no_frames` (one client draws frames, the other none for 4 s and is updated only when its pump asks: it is not dropped, both run their ticks and the hashes agree; while frames come, the pump does nothing) and its control `a_window_that_draws_no_frames_and_is_not_pumped_loses_its_game`. `game::pump`: `without_a_network_link_the_pump_never_wakes_nor_updates`. The winit wiring itself (`GameApp::about_to_wait`) was checked by inspection and by hand only.

The local lane `crates/app/tests/m9_lane.rs` (opt-in, needs `GLASNOST_GAME_DIR`) does the same with real data against the relay binary, which it starts with `cargo run --release -p glasnost-relay` on a free port. It writes each client's room and game screens to `local-lane/m9/n3/`. Run it with `cargo test --release -p glasnost-app --test m9_lane -- --ignored --nocapture`; `GLASNOST_LANE_TICKS` sets how many ticks it plays (default 400).

**Trying it by hand.** Native:

1. Run `cargo run -p glasnost-relay`.
2. In two terminals, run `cargo run --release -p glasnost-platform-native -- --game --game-dir DIR`. Add `--relay ws://HOST:PORT/` to use another relay.
3. In one window, click **Multiplayer**, then **Host**, open row 2 for a person, and read out the room code.
4. In the other window, click **Multiplayer**, type the code and click **Join**.

Web: run `cargo run -p glasnost-relay` and `cargo xtask web --serve`, then open `/?mode=game&relay=ws://127.0.0.1:8787/` in two tabs.

The pump by hand: start a network game, then cover the window with another one (or a screenshot overlay) or minimise it for 30 s, and in the browser switch to another tab for 30 s. The other player's game must play on, with no "left the game" notice; natively at full pace, in a background tab slowly.

**Seams for the later steps.**

- N4 (desync reports): done, see *Desync reports (N4)*.
- N5 (reconnect, surrender, late join): done, see *Reconnect and surrender (N5)*.
- N6 (soak): `m9_lane.rs` takes `GLASNOST_LANE_TICKS`.
- N7 (WebRTC): a data channel is another `Link`. The relay's WebSocket stays the signalling channel, and the only one whose `Dropped` is honoured.

### Desync reports (N4)

When two peers' checkpoint hashes differ, every peer that sees it writes a desync report: a file with what it needs to replay its own game from the last state every peer agreed on to a few ticks past the mismatch. The desync tool reads two peers' reports, replays both, and names the first tick on which they differ and the objects, houses and fields that differ. The evidence and the report are in `crates/net/src/desync.rs`, the tool in `crates/tools/src/desync.rs` and `inspect.rs`, and the client's side in `client_net.rs` and `game/report.rs`. Everything here is our own netcode (`// nocite: our own netcode`). No game rule changes, and the simulation crates gain only a read-only accessor (`glasnost_ai::game::Game::brains`, for the tool's view of our AI's state).

**The evidence** (`desync::Evidence`). Every network game keeps two things in memory:

- *Snapshots.* A snapshot is the game's own save bytes (`Game::save`, which hold the AI houses' brains and the whole replay and load back to the same state hash). There is one of the start, and one after every checkpoint, that is every K = `hash_every` ticks (default 30). A snapshot is taken right after the tick that ends a checkpoint, so its state hash is the checkpoint hash every peer compares.
- *Tick records.* For every tick since the oldest snapshot held: the merged orders (slot, house, order, and whether the game took it), and at a checkpoint the peer's own state hash after the tick.

The client feeds it from `net_tick` (`Evidence::before_tick` and `after_tick` around `Lockstep::step`). It keeps everything from the newest snapshot every peer has agreed on: `Session::agreed()` is the number of leading checkpoints that every live other peer has matched with ours, without a gap. A mismatch is found only when the other peer's hash arrives, at most about two largest delays after its checkpoint, so the evidence never holds more than 3 + ⌈2 × `max_delay` / K⌉ snapshots (5 with the default parameters). Once a mismatch is noted it stops forgetting, and once the report is made it holds nothing more.

**Per-tick hashes.** A state hash serializes the whole world. On the install's maps one takes 4 to 16 ms natively, more than a tick (2 to 4 ms), so the evidence does not take one after every tick while the game runs: a running game pays only the snapshot at each checkpoint (a save, 8 to 29 ms once every K ticks; about 0.2 to 0.7 ms a tick on average). When it makes the report, the peer instead replays its own ticks from the base snapshot, on its own machine and build, and records its state hash after every tick (`TickRecord::live` is false for those). A difference of platform or build shows in those hashes as it would have live. The checkpoints' hashes stay the ones taken live, so a fault the replay does not repeat (a memory fault, a change it did not record) still shows at its checkpoint. Making the report took 0.24 to 0.6 s on the lane's maps, once. `Evidence::with_tick_hashes(true)` takes every tick's hash live instead; the lane measured 4 to 13 ms a tick for it, so only tests use it.

**The report** (`desync::Bundle`). On `Event::Desync` the client notes the mismatch (the earliest counts) and shows "The games went apart at tick N (a desync)." once. Five ticks later (`PAST_TICKS`) the evidence makes the bundle. It holds:

- the build and the start values (`StartInfo`: rules hash, map hash, seed, slots, setup hash, parameters);
- the writer's slot and the mismatch (its tick, the other slot, both hashes);
- the test perturbation it applied, if any (below);
- the base snapshot: the agreed checkpoint before the mismatch;
- the writer's own snapshot at the mismatched checkpoint (`after`);
- the tick records from the base to at least `PAST_TICKS` ticks past the mismatch, each with its state hash (live at the checkpoints, replayed for the others; below).

The file is the magic `GDSY`, the format version (`BUNDLE_VERSION`, 2 since N5 added the grace period to the start values it holds), then a postcard body. The extension is `.gdsync`. A reader refuses another magic, another version and trailing bytes, and never panics on hostile bytes (`hostile_bundle_bytes_never_panic`). Bump the version whenever the layout changes; a change of the save format changes the snapshots inside, which the save format's own version already names.

**Size.** A bundle is two snapshots plus about K + `PAST_TICKS` tick records (small: the persons' orders only), so it is about twice the size of a save of the game at that point. On the synthetic fixture it is about 630 KB. On the lane's three maps of the install a save was 1.5 to 4.6 MB and a bundle 3.0 to 9.1 MB. While a game runs the evidence holds at most 5 snapshots with the default parameters, that is about five saves (up to about 23 MB on the largest of those maps); the lane held 2 at most (3 to 9 MB).

**Only the digest crosses the network.** A bundle holds the whole game state, and it is far larger than the relay's 64 KiB message limit, so it never crosses the network. The peer sends only the bundle's SHA-256 digest: `Session::set_report(tick, digest)` puts `Report { tick, digest }` into every later frames message (`Frames.report`). This changed the frames layout, so `WIRE_VERSION` is now 4 (the lobby shares it). Every other peer gets `Event::Report { slot, tick, digest }` once per report and shows it as a notice.

**In the client.** The client stores the bundle through its `SaveReport` function (`game::report`; `Client::save_report`, `GameOptions.report`):

- Native: a file in `desync-reports/` under the working directory the client was started from, made if need be (`report::save_file`).
- Web: a download through a Blob URL and a clicked anchor (`crates/platform-web/src/download.rs`), which goes into the browser's downloads.

The file name is `desync-<ROOM CODE>-tick<N>-slot<k>.gdsync`, where N is the mismatched checkpoint's last tick and k the writer's slot counted from 1 (`Bundle::file_name`). The notices are:

- "Desync at tick N; report saved to PLACE (digest XXXXXXXXXXXX)." The digest is the first 12 hex digits of the file's SHA-256. PLACE is the file's path, or "your downloads as NAME".
- "Bea saved a desync report of tick N (digest XXXXXXXXXXXX)." for every other peer's report.
- "Desync at tick N; the report was not saved: WHY." when the store fails. The digest is then not sent.

The game goes on after a desync; nothing stops it.

**Collecting and reading a report.**

1. When the notice shows, note the room code and the tick. Each player who saw the desync has a report: natively in `desync-reports/` beside where they started the client, in the browser in their downloads.
2. Collect two reports, one from each side of the mismatch. In a game of three or more, take the one of the slot the notices name and one other. To check that a file is the one its player's game made, compare the first 12 hex digits of `sha256sum FILE` with that player's digest in your notices.
3. Run `GLASNOST_GAME_DIR=DIR cargo run --release -p glasnost-tools --bin glasnost-headless -- desync A.gdsync B.gdsync`, where DIR is the install the game ran on. The tool finds the map by the report's map hash and checks the rules hash. Without `GLASNOST_GAME_DIR` it reads only reports of the synthetic test rules. It exits with 0 when the two agree, 1 when they differ, and 2 when they cannot be compared (a file that is no report, an install without the map or with other rules). Reports of different start values differ (exit code 1), and the tool names the first start value that differs.
4. Read the output from the top:
   - Each file with its digest, slot, build, ticks, mismatch, how many of its hashes were taken live and how many replayed, and any test perturbation.
   - "The first differing tick: N." This is from the hashes each peer recorded while it played.
   - Whether the tool's replay gives the hashes its peer recorded (live or replayed on the peer's machine). A peer whose own run is not reproduced from its snapshot and orders changed outside what this build computes: another build, a platform difference, memory corruption, or a test perturbation it did not record. This is the signal of nondeterminism.
   - Whether the merged orders differ. They never should; if they do, it is a lockstep fault.
   - A field-by-field diff at the first tick where the two games can be compared. Where the replays differ, this is the replayed state at that tick, which is exactly the first differing tick when the bundle records the perturbation that caused it. Otherwise it is each peer's own snapshot at the mismatched checkpoint. The diff lists objects first (id, kind, type name, owner, then each differing field with both values, at most 64 objects and 48 fields each), then houses, the game-wide state, and our AI's state.

**Forcing a desync** (tests and by hand). `desync::Perturb` changes one object's health or one house's credits from outside the rules, before a given tick runs, so the first differing hash is the one after that tick. Only a test client honours it. Natively, a debug build reads `GLASNOST_PERTURB`: `health:TICK:OBJECT[:BY]`, `credits:TICK:HOUSE[:BY]` (BY defaults to 1), or `none`. Any value makes the client a test client (`Client::set_test`), whose build is `glasnost <version> (test)`, so a room or a game of ordinary clients refuses it. To force a desync between two native clients, start both debug clients with `GLASNOST_PERTURB` set, one to a perturbation and the other to `none`. A release build ignores the variable. The bundle records the perturbation it applied, and the tool applies it in the replay.

**Tests.**

- `crates/tools/tests/desync.rs`: peers play the synthetic skirmish over the seeded network (bad links). In `a_perturbed_object_is_named_with_its_field_and_tick`, an object's health changes on one of two peers before tick 100. The tool names tick 100, the object and its health; with the perturbation's record removed, it reports the peer as unreproduced and falls back to the snapshots at the checkpoint; the binary exits with 1. In `a_perturbed_house_is_named_and_the_others_agree`, a house's credits change on the middle one of three peers, with every tick's hash taken live; the two unperturbed peers' bundles compare as the same. In `without_a_perturbation_nothing_differs`, there is no desync; bundles made on demand compare as the same (exit code 0), a junk file gives exit code 2, and the evidence stays bounded.
- `crates/net/tests/lockstep.rs`: `agreed_checkpoints_stop_at_a_desync_and_a_report_reaches_the_others`.
- `crates/net/src/desync.rs`: `perturbations_parse` and `hostile_bundle_bytes_never_panic`.
- `crates/tools/src/inspect.rs`: `differences_are_named_by_path`.
- `crates/app/src/game/client_net_tests.rs`: in `a_desync_is_reported_by_both_with_their_digests`, two test clients play over an in-process relay, and one changes its credits before tick 100. Each shows the desync notice once, stores a report through a test store, and shows the other's digest. The two reports name the same checkpoint and start.

The local lane `lane_a_desync_on_the_installs_maps_is_reported` in `crates/tools/tests/desync.rs` (opt-in, needs `GLASNOST_GAME_DIR`) forces a desync on the first three multiplayer maps of the install. Two persons and computers play there for 400 ticks, slot 0's game changes before tick 300, and the tool runs on the install's rules. On every map both peers wrote a bundle and heard the other's digest, and the tool named tick 300 and the changed object or house, with both replays reproducing their peers' hashes. It prints counts and timings only, and writes them to `local-lane/m9/n4/lane.txt`. Run it with `cargo test --release -p glasnost-tools --test desync -- --ignored --nocapture lane`.

**Seams for the later steps.**

- N5 (reconnect, late join): done, see *Reconnect and surrender (N5)*. A rejoining peer is sent a save made when it asks, not an evidence snapshot, and its evidence starts from that game (`Evidence::resume`).
- N6 (soak): `Evidence::bytes_held` and the lane's timings show what the evidence costs a long game.
- A report on demand (`Evidence::bundle_now`) makes a bundle with no mismatch, for a person who suspects a desync the hashes missed.

### Reconnect and surrender (N5)

A person whose connection or whole client goes during a network game has a grace period to come back. If they come back in time, a live peer sends them the game as it stands, their client runs the ticks they missed and they play on in lockstep. If they do not, their house surrenders on the same tick on every peer. A member who received `Begin` but never ran a tick (a crash while the map loaded) joins late the same way. The rejoin protocol, the transfer, the grace period and its length are our own netcode design (`// nocite: our own netcode`). The only game rule involved is the surrender, which is the existing `Order::Surrender` (S-0414 claim 11), the same order a person gives. No second way for a house to leave is added, and the simulation crates do not change. Code: `crates/net/src/session.rs` (lives, the surrender, `catch_up`, `rejoin`), `crates/net/src/rejoin.rs` (the transfer), `crates/relay/src/hub.rs` (tickets, serving, readmission), `crates/app/src/game/client_net.rs` (the client), `crates/ui/src/netmenu.rs` (the button).

**Tickets.** At the start the relay draws a secret 16-byte ticket for every person's slot from its host randomness, and sends each member only its own, in `Begin.ticket`. A rejoin is `ToRelay::Rejoin { code, slot, ticket, build, rules_hash }`. The relay refuses it with `Refusal::NoGame` when there is no such room, the room has no game or nobody live is left to send it; `Ticket` when the slot is no person's or the ticket is not that slot's; `Mismatch(field)` for another build or rules hash; `Surrendered` when the slot's grace period has run out (the relay knows it from the most ticks any live peer says it has run, `Frames.ran`); and `Rejoining` while another slot is rejoining that game (one at a time per game; this one does not count against the refusal limit). A ticket for a slot that is still live replaces that slot's connection (a client that lost its link before the relay noticed). A connection with the right ticket gets `FromRelay::Rejoined(Begin)`, the start values as at the start.

**Lives and the surrender** (`session::Life`). Each person's slot has the frames of it that count: from `from` (0, or the first frame after a rejoin) to before `until` (set by a drop). The relay's `Dropped { slot, kept }` (N2) sets `until = kept`, the same on every peer. Unless the slot is readmitted first, every peer issues `Order::Surrender` in that house's name on tick `kept + grace_ticks`, at the slot's place in the slot order, so the surrender is part of that tick's orders everywhere. No message asks for it, so no peer can forge or withhold one. `Event::Surrendered { slot, tick }` reports it. `grace_ticks` is a new start value (`Params::grace_ticks`). The relay works it out at the start from its `--grace SECONDS` (default 60 s, `Limits::grace_ms`) and the room's game speed: `grace_ms / period_ms(speed)`, where `glasnost_ui::setup::period_ms` is the paced period of S-0201 claims 2, 4 and 5, which the client's pacer uses too. The grace period is thus game time: a minute of play at the room's speed, whatever the machines' speed.

**Serving.** When a rejoin is accepted, the relay picks the live person of the lowest slot to serve it and sends that peer `Body::Serve { slot, on: true }`. From then on, the relay also sends the rejoining connection every lockstep message it forwards and every drop. The serving peer:

- keeps its frames until the rejoining peer acknowledges them, and sends them to it even when no other peer is live;
- makes a `rejoin::Transfer` once its session runs: its session's view of every person's slot (`Session::catch_up`: lives, the frames that count and those it holds but has not run), the game's save (`Game::save`) and its state hash, after the same ticks;
- sends it as `ToRelay::Snapshot(Part)` parts, one every 25 ms, while it plays on. The relay passes each on as `FromRelay::Snapshot`.

The transfer format is the magic `GRJN`, `TRANSFER_VERSION` 1, then the transfer in postcard. A part holds at most `PART_BYTES` (60 000), so each message stays under the relay's 64 KiB limit. A rejoining peer takes at most `MAX_TRANSFER` (64 MiB), and the relay passes on at most `Limits::max_snapshot` bytes per rejoin (the same 64 MiB by default); beyond that the relay calls the serving off (`Serve { on: false }`) and closes the rejoining connection with "the game is too large to send". `Assembler` takes the parts in order only, refuses a part of another count or past the limit, and never panics on hostile bytes (`hostile_transfer_bytes_never_panic`).

**Catching up.** The rejoining client loads the map from `Rejoined` as at a start, keeps the lockstep messages that come meanwhile (at most 16 MiB), puts the parts together, loads the save (since sim issues S1, `save::load` refuses a save whose tables do not hold together instead of panicking on it, #157; see *Simulation (sim issues S1)* in [simulation.md](simulation.md)), checks the loaded game's state hash against the sender's, and starts its session with `Session::rejoin` from the transfer. It then feeds the kept messages and puts the game in place of the fresh one (`Session::replace_game`). Its desync evidence starts from that game (`Evidence::resume`). It runs the ticks since unpaced, at most 10 per update. Until it is readmitted it makes no frames of its own; it only acknowledges, and the relay sends those acknowledgements to the serving peer only. When it has run every tick it has frames for, it sends `ToRelay::CaughtUp` once.

**Readmission.** On `CaughtUp` the relay asks the serving peer to readmit the slot (`Body::Resume { slot }`). That peer's next frame carries the marker `Frame.resume = Some(slot)`. When the marker's frame is of tick *n*, every peer awaits the slot's frames again from frame `max(n + max_delay + 1, kept)` on, a frame no peer has run when it runs *n*; the surrender is called off if it was not due by tick *n*. The relay counts those frames as forwarded for the slot's next `kept`. A rejoining peer that goes before its readmission is dropped again at that frame, and can rejoin again.

**A lone stretch** (fixed after the M9 exit run). While a person has no other live person in the game, nobody needs its frames: its session's frames messages then start at its next frame and carry none (`first_unacked` has no recipient, and `prune` forgets the frames it ran), so they leave a gap behind the slot's `kept`. Under N2's rule alone `kept` then stopped where the last other person went, and stayed there after a rejoin: the readmission raises `kept` only for the rejoining slot, and the serving peer's frames start at what the rejoining peer acknowledges. When that person later left, every peer was told a `kept` far below the ticks it had run, refused the drop as a bad message ("slot 0 dropped after frame 21803, but tick 29426 already ran with its frames"), and never surrendered the house, so the game had no winner. So the relay now counts a frames message of a person with no other live person past a gap too, up to its end, also while it serves a rejoin. That is sound:

- Nobody else is live to lack a frame below it.
- A rejoining connection is sent every frames message of the serving peer from the moment it is accepted, and the game it is sent holds every frame the serving peer made before `Serve` reached it (the transfer is made after that); the frames messages after `Serve` start at what it acknowledged, without a gap. A frames message the serving peer sent before `Serve` reached it, arriving late, starts at most at the serving peer's next frame then, which the game it is sent also holds.
- Once the rejoined peer is readmitted the serving peer is no longer alone and N2's rule applies again; its frames messages start at the rejoined peer's acknowledgements, which are at most `kept`.
- A drop round (N7) asks from `kept`, so it starts from the right frame too.

Only the relay changes; no message changes, so `WIRE_VERSION` stays. The fix is in the relay rather than in the session because `kept` is the relay's count: the session would otherwise have to keep and resend frames nobody needs for the whole lone stretch.

**What the relay refuses.** `Serve`, `Resume` and `Dropped` come only from the relay (`wire::RELAY`); the relay never forwards one from a client, and a session takes them only from `RELAY` (an `Event::BadMessage` "not from the relay" otherwise). The relay forwards a readmission marker only from the peer it asked, only for the slot it asked about, only in one frame and only after `CaughtUp`, and closes any other sender of one. A rejoining connection that sends anything but frames messages, or speaks for another slot, is discarded and counted. If the serving peer goes before the readmission, the relay closes the rejoining connection with "the player sending you the game left; reconnect to try again". New stats: `rejoins`, `readmitted`, `forged_rejoins`, `snapshot_bytes`.

**What a person sees.**

- When a person goes, every other player sees "Bea left the game; they have 60 seconds to reconnect." (the seconds left of the grace period at the room's speed). The game goes on without them; their house stands idle.
- The person whose link went (closed, or lost in the game) returns to the multiplayer screen. The status line says "Reconnect within about N seconds to play on, or your house surrenders." The screen shows **Reconnect to game CODE** under **Host** and **Join** whenever the client knows a game to rejoin, also after the client restarts, since the room, slot, ticket and relay are kept in the settings store while the game runs (the `rejoin` line; see *Settings (M8 step C2)*). The button is offered while the relay field is not empty.
- After **Reconnect**, a banner over the loading game says "Loading the game...", then "Waiting for the game...", "Receiving the game (k of n parts)..." and "Catching up with the game (tick N)...". When the client is readmitted it says "You are back in the game."
- The serving player sees "Bea is reconnecting; sending them the game.". Everyone sees "Bea reconnected." on readmission.
- When the grace period runs out: "Bea did not come back in time: their house surrendered." A rejoin after that is refused ("too late to rejoin: the grace period ran out and the house surrendered"), and the client forgets the game.
- **Leave** in the game sends `Leave`, which the relay treats as a drop: the others see the grace-period notice and the house surrenders when it runs out. The leaving client forgets the game, so it does not offer **Reconnect**. The stored game is also forgotten when the game ends, and when the relay refuses a rejoin for its ticket, a surrender, no game or a mismatch.

**Tests.**

- `crates/net/tests/lockstep.rs`: `a_dropped_house_surrenders_after_the_grace_period_everywhere_on_one_tick`, over the seeded network. Forged `Serve` and `Resume` from peers are refused as not from the relay, and the surrender runs on tick `kept + grace_ticks` once on each peer.
- `crates/net/src/rejoin.rs`: `parts_go_back_together_in_order_only`, `hostile_transfer_bytes_never_panic`.
- `crates/relay/tests/rejoin.rs`, on the hub with a virtual clock and the synthetic skirmish (`common/skirmish.rs`), each peer on its own `glasnost_ai::game::Game`:
  - `two_persons_drop_and_rejoin_with_identical_hashes` and `three_persons_and_a_computer_drop_and_rejoin_with_identical_hashes`: the rejoined peer's hashes equal the others' at every tick after its readmission, and every peer's game replays to the same hashes.
  - `a_member_gone_before_the_game_ran_joins_late`: a member gone at tick 0 joins late (a transfer of about 320 KB on the fixture, readmitted from frame 111).
  - `past_the_grace_period_the_house_surrenders_on_every_peer_on_one_tick`, `a_rejoin_inside_the_grace_period_cancels_the_surrender`, `a_rejoin_after_the_grace_period_is_too_late`, `a_rejoining_peer_cut_off_before_its_readmission_can_rejoin_again`.
  - `a_rejoin_into_someone_elses_slot_is_refused`, `forged_relay_messages_and_readmission_markers_are_refused`, `only_the_asked_peer_readmits_and_only_after_the_catch_up`.
  - The lone stretch (the M9 exit run's second game): `a_peer_that_played_alone_leaves_and_its_house_surrenders_everywhere` (two persons: slot 1 drops and rejoins after about 75 ticks that slot 0 plays alone, then slot 0 leaves), `a_peer_left_alone_in_a_game_of_three_leaves_and_its_house_surrenders` (the same after slot 2 left for good) and `a_peer_that_played_alone_and_relinked_leaves_through_a_drop_round` (with a direct link the rejoined peer sets up again and slot 0's messages lagging, so its drop goes through N7's round). The peers left take the drop with no bad message and surrender the house on tick `kept + grace_ticks`. Before the fix all three failed with the exit run's message ("slot 0 dropped after frame 63, but tick 249 already ran with its frames"); counting the gap only when a frames message went to nobody, with no rejoining connection, still failed the third. `crates/relay/tests/hub.rs`: `a_lone_person_s_frames_count_past_a_gap`.
- `crates/app/src/game/client_net_tests.rs`: `a_client_that_went_reconnects_and_plays_on_in_lockstep`. Three persons and a computer play over an in-process WebSocket relay. Bea's client goes; a new client on the same settings store presses **Reconnect**; the host serves it and the third player does not. The hashes agree for more than 150 ticks after the readmission, which took about 0.7 s.
- `crates/ui/src/netmenu.rs`: `a_game_to_rejoin_shows_a_reconnect_button`. `crates/app/src/settings_tests.rs`: the `rejoin` line's round trip and bad lines.

**The local lane.** `a_killed_client_process_restarts_and_rejoins` in `crates/app/tests/m9_lane.rs` (opt-in, needs `GLASNOST_GAME_DIR`) runs the guest as a child process of the test (`lane_guest_process`, chosen by `GLASNOST_LANE_ROLE`) with its own settings file. It kills that process with SIGKILL at host tick 300, starts it again on the same settings file, and has it press **Reconnect**. On the install's first offered map:

- The host noticed the drop 13 ms after the kill.
- The restarted guest's rejoin reached the relay about 3 s after the restart (most of it mounting the install). It loaded the game 2.2 s after pressing **Reconnect**: a transfer of 1.6 MB in 27 parts.
- It caught up in about 0.7 s (about 240 ticks a second) and was readmitted 5.9 s after the restart.
- Both clients then played 300 ticks more with equal hashes at every tick.

It writes counts, timings and both clients' screens to `local-lane/m9/n5/`. Run it with `cargo test --release -p glasnost-app --test m9_lane -- --ignored --nocapture a_killed_client_process_restarts_and_rejoins`.

**Seams and limits.**

- A hostile peer that claims to have run more ticks than it has (`Frames.ran`) can make the relay think a slot's grace period is over, and so refuse that slot's rejoin while its house has not yet surrendered. A hostile client that sends frames far ahead and then leaves can still stall the others (N2's seam). A hostile peer left alone can do the same with an empty frames message far ahead, which now raises its `kept` (the lone stretch): the same seam.
- A member who never received `Begin` has no ticket and cannot join late.
- One rejoin at a time per game (`Refusal::Rejoining`).
- A peer whose others have all dropped still sends its frames message, empty of new frames, once every resend interval (by default a tick's period), whether or not it serves anyone: the relay learns from it how far the game has run, and so whether a dropped house has surrendered.
- A lost link takes the person to the multiplayer screen; the client does not reconnect by itself.
- After the game is put in place on a rejoin, some presentation state of the fresh load stays until it next changes: the "Game started." line, the credits counter's roll, the selection.
- N6 (soak): rejoins under load and long games, where the transfer and the catch-up are longer.
- N7 (WebRTC): `Serve`, `Resume`, `Dropped` and the marker checks must stay on the relay's channel; a data channel must never carry them.

### The soak (N6)

A windowless client plays our AI as a person in a network game, and four of them play a 2v2 over the relay on loopback, behind bad networks, with links cut and a process killed mid-game. Everything here is our own harness (`// nocite: our own harness`); the simulation crates do not change. Code: `crates/app/src/game/bot.rs` (the bot, its options and its loop), `crates/tools/src/net_client.rs` (`glasnost-headless net-client`), `crates/tools/src/netem.rs` (the bad network), `crates/tools/src/soak.rs` (`glasnost-headless soak`), `crates/headless-web` (the wasm client) and `crates/tools/tests/soak.rs`.

**The bot** (`glasnost_app::game::bot::Bot`). It drives a real `game::client::Client` through the screens a person uses, by laying out a frame and clicking the client's own targets: **Multiplayer**, then **Host** or **Join** with a room code. In the room, the host steps to the first map (from `--map` on) with a start for every person, sets that many rows to person and closes the rest; every bot picks its own row's team and says ready; the host presses **Start** once every person is in and ready. In the game, a `glasnost_ai::brain::Brain` for the bot's house thinks once per tick the client ran, on the client's world, and its orders go through `Play::issue` into the outbox and the lockstep session, the same path as a person's clicks. When the client loses its game (its link closed), the bot presses **Reconnect** after `reconnect_after_ms` (2 s, plus up to 1 s of seeded jitter so that peers cut together do not come back on one frame) and plays on with a new brain once readmitted. A refused rejoin (`Rejoining`) is tried again after the same wait. With `--resume`, a client started on a settings file that names a game rejoins it (a killed process started again).

Two things differ from a skirmish, both kept in the bot:

- *Orders in flight.* In a network game an order runs a delay after it is given, and the brain, made for orders that run at once, sees no effect and gives the same order again on the next ticks. The bot drops an order equal to one the brain gave at one of the last delay + 2 ticks (`BotConfig::dedupe`, on by default; `--no-dedupe` turns it off). Orders given twice in the same tick pass. In a two-minute trial on the install, the four bots dropped 2 507 repeats and gave 990 orders.
- *A new brain after a rejoin.* The brain is the bot's own, not part of the game, so it is not in the transfer; a rejoined bot starts a new one on the game as it stands.

**The numbers** (`BotStats`, counts and timings only): ticks run and game time, stalls (a tick that came more than `STALL_MS` = 100 ms after its paced period) and the longest gap between two ticks, times the client showed "Waiting for ...", the agreed delay at every change, losses, presses of **Reconnect** and how long each rejoin took from the loss to the readmission, the bytes of game sent to rejoining peers and received, the most `Evidence::bytes_held` reached, the orders given, dropped as repeats and taken (from the replay), the first desync, the winners, the bot's player allies, and every checkpoint hash the client ran. The client keeps its checkpoints for `Client::take_checkpoints` (at most `CHECKPOINTS_KEPT` = 4096 until taken), in the catch-up and in normal play.

**`glasnost-headless net-client`.** Options: `--relay ws://HOST:PORT/`, `--host N` (N persons) with `--map M`, or `--join CODE`; `--name`, `--team T`, `--slot ROW` (the row the client expects: the room gives the first open row, so the rows follow the order of the joins, and a client given another row leaves and fails), `--seed S` (the reconnect jitter), `--ticks N` or `--minutes M` (game time; otherwise until the game ends), `--hash-log FILE` (one `TICK HASH` line per checkpoint, appended as they come), `--report FILE`, `--code-file FILE` (the room code once the room is set up), `--settings FILE` (the settings store, where the rejoin is kept), `--resume`, `--no-ai`, `--no-dedupe`, and `--game-dir DIR` (default `GLASNOST_GAME_DIR`) or `--synthetic` (the synthetic install, `bot::synthetic`). It prints status lines on standard output: `mounted MS ms`, `room CODE row R`, `game slot S house H`, `lost N at tick T`, `readmitted N at tick T after MS ms`, a progress line every 30 s, and `done ticks T desync D` or `failed WHY`. Its log goes to standard error (`GLASNOST_LOG=debug` for more). A desync report goes to `desync-reports/` in the working folder (N4). When the bot is done, the client runs on for `LINGER_MS` (3 s) so that its peers have its frames for their last ticks. Exit code 0 when it finished, 1 when it gave up.

**The bad network** (`netem::Proxy`). A TCP proxy on loopback in front of the relay, one per client, seeded. A WebSocket runs over TCP, which never loses or reorders bytes, so loss is modelled as late delivery and order is always kept:

| What | Default | Constant |
|---|---|---|
| One-way latency | 40 ms | `LATENCY_MS` |
| Jitter, added to each chunk | 0 to 30 ms | `JITTER_MS` |
| Chance a chunk is "lost" | 20 in 1000 | `LOSS_PERMILLE` |
| A lost chunk's resend, and everything behind it waits | +200 ms | `RETRANSMIT_MS` |
| Time between two stalls of one direction | 45 to 150 s | `STALL_EVERY_MS` |
| A stall: nothing delivered, reading goes on | 0.5 to 2.5 s | `STALL_MS` |

Each chunk is delivered at the later of its own time and the time of the chunk ahead of it (head-of-line blocking, as on TCP). The proxy keeps reading while it holds data back, so a stall never pushes back on the sender. Stalls stay well under the session's 5 s silence notice and the relay's 15 s silence limit. `Proxy::cut` closes every connection through the proxy, as a dropped link does. The proxy counts bytes each way, connections, lost chunks and stalls. `Netem::clean` passes bytes on untouched.

**`glasnost-headless soak`.** It starts the relay (`cargo run --release -p glasnost-relay`) on a free port and a proxy per client, then the host (`--host 4`) and the three guests one at a time (each waits for the one before it to have its row), on rows 0 to 3: rows 0 and 2 are team 1, rows 1 and 3 team 2 (`soak::TEAMS`). Each client runs in its own folder `rowN/` under the output folder (default `local-lane/m9/n6/`, git-ignored), with its log, hash log, report, settings file and any desync report. The default trouble (`soak::default_trouble`), for a soak of about S seconds: row 1's link cut at S/6, row 3's at 2S/6, row 2's process killed (SIGKILL) and started again with `--resume` at S/2, the host's link cut at 4S/6. `--trouble 30:1:cut,90:2:kill` sets other trouble, `--no-trouble` none, `--clean` a perfect network; `--latency MS`, `--jitter MS`, `--loss PERMILLE` and `--no-stalls` change one knob of the bad network; `--client-arg ARG` passes an option to every client. `--wasm-row R --script crates/headless-web/node/net-client.mjs` makes row R the wasm client (it is cut, never killed, since it keeps no settings file). At the end it compares every client's checkpoint hashes tick by tick (a rejoined client logs again the ticks it ran again; they must agree too), and writes `soak.txt`. It passes when every client exited with 0, every client saw the game end or reached the limit, none noticed a desync, no two hashes of a tick differ, and the four share checkpoints.

**The wasm client** (`crates/headless-web`, package `glasnost-headless-web`). The same `Client`, bot and options, compiled to `wasm32-unknown-unknown` and bound with `wasm-bindgen --target nodejs`. It reaches the relay through the browser's WebSocket link (`glasnost_platform_web::socket`, the crate without its page entry point: the new default feature `start`), which works under node because node's `WebSocket` is the browser's API. `node/net-client.mjs` runs it: it lists the install folder, reads archive bytes for the client through `globalThis.glasnostReadAt` (synchronous file reads, so the client's jobs finish on the first poll), stores desync reports through `globalThis.glasnostSaveReport`, gives it the clock, and writes the same status lines, hash log, report and code file as the native client. It is the wasm32 build of the client's lockstep, simulation, AI, desync evidence, rejoin and web link; it is not a browser (no page, drawing, audio or folder picker).

Build and run, from the repository root:

```
cargo build --release -p glasnost-tools --bin glasnost-headless
cargo build --release -p glasnost-relay
cargo build --release --target wasm32-unknown-unknown -p glasnost-headless-web
wasm-bindgen --target nodejs --out-dir target/headless-web \
  target/wasm32-unknown-unknown/release/glasnost_headless_web.wasm
GLASNOST_GAME_DIR=<install> target/release/glasnost-headless soak \
  --headless target/release/glasnost-headless --minutes 30 \
  --wasm-row 3 --script crates/headless-web/node/net-client.mjs
```

The ignored test `long_soak_2v2` in `crates/tools/tests/soak.rs` runs the same soak: `GLASNOST_SOAK_MINUTES` (default 30), `GLASNOST_SOAK_WASM=crates/headless-web/node/net-client.mjs` for the wasm row, `GLASNOST_LANE_OUT` for the folder. Run it with `cargo test --release -p glasnost-tools --test soak -- --ignored --nocapture`.

**Tests.**

- `crates/tools/tests/soak.rs`: `four_ai_persons_play_a_2v2_over_bad_links_and_one_rejoins`, in CI. A relay in the test process, four bots on the synthetic install behind bad networks (stalls every 1.5 to 4 s), 450 ticks; row 1's link is cut at tick 150 and it rejoins. Every checkpoint agrees, none notices a desync, each house has one ally, and the rejoined client was sent the game.
- `crates/tools/src/netem.rs`: `bytes_pass_in_order_and_late`, `a_cut_closes_and_a_new_connection_passes`.
- `crates/app/src/game/bot.rs`: `options_parse`, `bad_options_are_refused`.

**The long soak** (local lane, on the install, 2026-10-22; counts and timings only). A limit of 30 minutes of game time (`--minutes 30`); the game ended in a win first, at tick 58 535 (15.6 minutes of game time, 26.5 minutes of wall time). Four persons on map index 30 (`--map 30`; the first maps are small and the AI game there can end in a minute or two), the default bad network on every link, row 3 the wasm client under node, seven troubles: row 1 cut at 120 s and 900 s, row 3 (wasm) cut at 240 s and 1200 s, row 2 killed and restarted at 360 s and 1500 s, the host (row 0) cut at 480 s. Another session's sweep used about 14 of the machine's 32 cores the whole time. It ran a build from before the last harness changes in this section (the pass rule, the network knobs, the transfer counters, the stop on a client that exits early); the lockstep, client and simulation code were the same.

| | |
|---|---|
| Result | pass: 0 mismatches, no desync, every client exited with 0 |
| Wall time | 26.5 min (1 589 s); the game ended before the 30-minute limit |
| Ticks | 58 535 (15.6 min of game time at the room's 16 ms period), on every client |
| Pace | 36.9 ticks/s on average against 62.5 paced: about 50 ticks/s in the first 2.5 min and most of the rest, about 16 ticks/s from tick 4 600 to 12 400, about 40 from 32 500 to 37 600 |
| End | game over at tick 58 535; houses 0 and 2 (team 1) won |
| Checkpoints | 1 925 every client ran, the last at tick 58 529; a rejoined client logs again the ticks it ran again, and they agree too |
| Rejoins | 7 of 7, from the loss to the readmission: row 1 8.0 s and 9.0 s, row 3 (wasm) 10.2 s and 27.8 s, the host 13.2 s; after a kill, from the restarted client finding its game, 2 s wait included: 9.7 s and 12.1 s |
| Games sent | 7 transfers of 2.24 to 2.32 MB (from the logs); the host served 6, row 1 served the host's |
| Delay | 3 for ticks 0 to 3, 30 at tick 4 (the first round-trip sample), then 26 to 30 for the whole game |
| Stalls | 744 to 996 ticks per client came more than 100 ms late; the longest gap between two ticks 2.4 s (the proxy's longest stall 2.49 s); "Waiting for ..." shown about 40 times |
| Network | per client about 30 stalls, 8 300 to 8 600 chunks "lost"; relay traffic per client 21 to 37 MB up (the host's includes the games it served) and 65 to 69 MB down, about 43 KB/s down |
| Evidence | at most 4.64 MB held (`Evidence::bytes_held`) |
| Orders | 3 000 (host), 1 900, 6 (row 2, its last process), 307 (wasm) given; 10 000 and 17 000 repeats dropped by the two native rows that played the whole game |

Shorter runs: 3 minutes native and 3 minutes with the wasm row on the bad network passed (355 common checkpoints, 3 cuts and a kill rejoined in 5.5 to 7.8 s; the wasm client mounted in 3.5 s); 2 400 ticks on the synthetic install with the wasm row passed (4 rejoins, 61 common checkpoints). On the final build, `long_soak_2v2` for 2 minutes with the wasm row passed: 7 500 ticks in 150 s (50 ticks/s), 4 rejoins in 5.0 to 7.9 s, 227 common checkpoints. A first 30-minute try on map index 0 ended in a win at tick 5 001 (80 s), before any trouble, which is why the soak plays a larger map. A milder network (`--latency 20 --jitter 5 --loss 0 --no-stalls`, 3 minutes, default trouble, all native) shows the delay move: 10 to 12 in the first ticks, down to 5 and 6, up to 30 at each rejoin and back down within about 600 ticks; from about tick 4 400 to 10 000 it stayed between 15 and 26 with no trouble, while the four clients ran 35 ticks/s against 62.5 paced, so a slow tick reads as a long round trip.

**Seams and limits.**

- *Orders given again.* Our AI expects an order to run in the tick it gives it; in a network game it runs a delay later, and the brain gives it again until then. The bot drops these repeats (`dedupe`); a person's client does not need it, but an AI playing in a network game for a person would.
- *The brain is not in the transfer.* A rejoined bot starts a new brain on the game as it stands, so it plays on with a different plan than the one it had.
- *The delay stays high on the bad network.* The first round-trip sample sets a timeout of three samples, and with 2 % of the chunks 200 ms late the estimate did not come back down in these runs: the delay sat at 26 to 30 (480 ms at the 16 ms period). On a milder network it moves both ways. Slow ticks raise it too.
- *Pace.* Four clients on one machine ran about 37 ticks/s against 62.5 paced, slower through two spells mid-game; the cause is not isolated (another session's sweep shared the machine).
- *Loss is per chunk.* The proxy draws loss for each 16 KiB read, not for each TCP segment, so a large message is late at most once.
- *A checkpoint can go unlogged.* If the link closes in the same update as a checkpoint, the client may drop that checkpoint before the bot takes it; the soak compares only the ticks it has.
- *Rejoin times after a kill* count from the restarted client finding its game, plus the 2 s wait it skips.
- *Not a browser.* The wasm row is the wasm32 build under node, not a page: no drawing, audio, folder picker or browser download.

### WebRTC (N7)

A network game's peers also send each other their frames on direct WebRTC data channels, beside the relay. The relay stays the game's path and its only authority: every lockstep message still goes through it, and a direct link only lets a frame arrive sooner. Nothing here is a game rule; it is all our own netcode (`// nocite: our own netcode`), and the simulation crates do not change. Code: `crates/net/src/session.rs` (`Via`, `receive_via`, `Paths`, `direct_ok`, the drop round's `Closing` and `Held`), `crates/net/src/lobby/wire.rs` (`Signal`, `Ping`, `Pong`), `crates/relay/src/hub.rs` (signal routing and the drop round), `crates/rtc` (`glasnost-rtc`, the native link), `crates/app/src/game/direct.rs` (the pairs, their timeouts and the shaped network), `crates/platform-web/src/rtc.rs` (the browser's link), `crates/tools/src/soak.rs` (`--transport`) and `web-tests/rtc.mjs`.

**Design.**

- *One data channel per pair of persons.* Each pair of human slots in a game tries one link; the lower slot offers. AI slots have no peer and need none. A peer copies only its own `Frames` messages onto its open links (`wire::is_frames_from`), and still sends everything to the relay.
- *Unordered and never resent.* The channel is negotiated (id 0 on both ends, so neither waits for the other to announce it), unordered, with no retransmits (`maxRetransmits: 0`). The lockstep needs every frame, but it already gets every frame through the relay, over TCP: a datagram the direct path loses costs nothing, the relay's copy comes a moment later, and the lockstep's own resend of unacknowledged frames (N1) goes through the relay too. A reliable, ordered channel would only add head-of-line waits to the path that is meant to be the fast one.
- *Signalling on the relay, as lobby messages.* `ToRelay::Signal { to, attempt, signal }` and `FromRelay::Signal { from, attempt, signal }`, with `Signal::{Offer, Answer, Candidate}` (SDP text, or a candidate as the JSON of `RTCIceCandidate.toJSON()`, at most `MAX_SIGNAL` = 16 KiB, printable ASCII; `Signal::plausible` checks the shape on the relay and again on the receiver). Matchbox's own signalling protocol was not used: it wants its own WebSocket server and peer ids, while the relay already knows the room, its slots and who is live, so the slots are the peer ids and there is no second server and no second room code. The relay passes a signal only between two live persons of one game, from the sender's own slot, with its own budget: one per `signal_every_ms` = 50 ms on average, at most `signal_burst` = 64 at once; the rest is dropped. `ToRelay::Ping { stamp }` / `FromRelay::Pong { stamp }` measure the relay path's round trip (once a second, `RELAY_PING_MS`); a pong spends the general message budget. Lobby wire and lockstep wire version 6.
- *Attempts.* The offering end numbers its attempts. A signal of another attempt than the current one is dropped; an offer of a new attempt replaces the answering end's link. A peer that rejoins is a new process and numbers from 1 again, so the answering end forgets the number while the slot is out, and keeps signals that come before the slot plays on its side (a rejoined peer may be readmitted on its own side a moment earlier) for when it does, at most `MAX_SIGNALS`, none older than the connect timeout.
- *Native.* `glasnost-rtc` wraps `matchbox_socket` 0.14 (webrtc-rs underneath) for one pair: our own signaller in place of matchbox's WebSocket, our ICE servers only (`.ice_server(...)`, so matchbox's default Google STUN server is never used), no reconnects, no keep-alive to a signalling server. Its loop runs on its own thread, `glasnost-rtc`, under `catch_unwind`: matchbox panics on some hostile SDP (an `expect` in its native handshake), and that ends only the attempt. It uses matchbox's non-panicking calls (`try_update_peers`, `try_send`). `cargo xtask deps` keeps the WebRTC stack inside `glasnost-rtc` and out of `glasnost-net`.
- *Browser.* `glasnost-platform-web`'s `rtc` module drives `RTCPeerConnection` through `web-sys`, not matchbox: matchbox's wasm path panics on an empty ICE URL list (which is our default), `expect`s on a hostile SDP, and mishandles a channel that opens after the attempt was given up. The same channel settings; an empty server list is passed as `iceServers: []` (browsers reject an entry with no URLs); candidates either way wait until the remote description is set; a browser without `RTCPeerConnection` closes the attempt at once. A browser and a native client can link (the same SDP and candidate fields), though only browser to browser and native to native were run.
- *Control packets.* Each link carries its own pings: `GP`, version 1, kind 0 (ping) or 1 (pong), a 32-bit stamp, 8 bytes, every `PING_MS`. They give the direct round trip and tell a live link from a dead one: matchbox did not notice a peer that went away within 40 s in a trial.

**Trust.**

- `Dropped`, `Serve`, `Resume`, `Closing`, `Held` and the rejoin markers come only from the relay's WebSocket. A data channel carries frames and nothing else: the session takes from a direct link (`Session::receive_via(.., Via::Direct(slot))`) only `Body::Frames` whose sender byte is the channel's slot, while that slot is live and not leaving and this peer is not catching up (`Session::direct_ok`). Every other byte on a link is refused and counted (`PairStats::refused`). A direct frame that carries a readmission marker (`Frame::resume`), or one more than `Session::lead` (eight largest delays) ahead of what the relay has passed on of that slot (its relay frontier), waits for the relay's copy (`Paths::gated`).
- *`kept` stays authoritative: the drop round.* The relay still sees every frame, but a peer may have run a frame that came the direct way before the relay forwarded it. So when a slot that signalled with others drops, the relay first asks the live peers it signalled with for that slot's frames they hold past what the relay forwarded (`Body::Closing`), takes their answers (`Body::Held`, without a gap, first copy wins) for at most `answer_ms` = 10 s, closes a peer that does not answer by then, sends everyone the frames put together, and only then the `Dropped` whose `kept` counts them. Every peer then drops the slot at the same frame.
- *First copy wins; two stories are reported.* If the relay's copy of a frame differs from the direct copy, the first one ran; the session reports it once per slot (`Event::Conflicting`, a log line and a notice "X's orders came twice and differed"). The checkpoint hashes (N4) then tell whether the peers ran the same.
- *The cost.* A hostile peer can send different frames on its direct links than through the relay (equivocate), or claim false `Held` frames within its last `lead` frames when another slot drops; the conflict notice and the desync check catch it, and relay-only (below) removes it. Bandwidth roughly doubles for a peer's own frames (each goes to the relay and to every open link). A native matchbox panic costs one attempt. webrtc-rs may keep some memory per attempt, and matchbox pulls in `async-tungstenite` and `async-std` that are compiled but unused.

**Fallback.** Nothing for the players to do, and no desync: the relay carries every frame all along, so a pair whose link cannot be made, or dies, simply has its frames come through the relay; the game never waits for a link. A link that dies ends on both sides; the offering end tries again, and a link that comes back is used again. The timeouts (`game::direct`, `Timeouts`):

| What | Default | Constant |
|---|---|---|
| An attempt may take this long to open; then it is given up | 10 s | `CONNECT_MS` |
| An open link may be silent this long; then it is dead | 3 s | `STALE_MS` |
| An open link is pinged every | 0.5 s | `PING_MS` |
| After a failed or dead attempt, the offering end waits | 15 s, doubling with each failure in a row | `RETRY_MS` |
| The longest wait between attempts | 120 s | `RETRY_MAX_MS` |
| The most signals one attempt sends; more ends it | 40 | `MAX_SIGNALS` |
| The relay's signal budget per person | 1 per 50 ms, burst 64 | `Limits::signal_every_ms`, `signal_burst` |
| The relay waits for the drop round's answers | 10 s | `Limits::answer_ms` |

A slot that leaves, or a peer that is catching up, ends its links and resets the back-off; the links come again once it plays.

**Switches.** Direct links are on by default. Relay only: the settings line `direct=0`, `--relay-only` (native), `?direct=off` (web), `--relay-only` (net-client). ICE servers: none by default; the settings line `ice=URL ...`, `--ice URL` (native, repeatable), `?ice=` (web), `--ice "URLS"` (net-client), space-separated `stun:HOST:PORT` or `turn:HOST:PORT` URLs, with `username=NAME` and `credential=WORD` words for a TURN server (kept in the settings file as typed).

**How to run it.**

- Two native clients: `cargo run -p glasnost-relay`, then `cargo run -p glasnost-platform-native -- --game --game-dir DIR` twice; host and join as in N3. The log says `direct link to slot N open` and, on a loss, `direct link to slot N lost (...)`.
- Web: `cargo xtask web --serve` and `/?mode=game&relay=ws://127.0.0.1:8787/` in two tabs (or one tab and a native client).
- The browser check: `cargo xtask web`, then `node web-tests/rtc.mjs --browser chromium` (or `firefox`).
- The soak over direct links: as in N6, with `--transport rtc` (`GLASNOST_SOAK_TRANSPORT=rtc` for `long_soak_2v2`); `--transport relay`, the default, is N6's soak with every client on `--relay-only`.

**Tests.**

- `crates/net/tests/direct.rs` (8): a direct link carries only its slot's frames (another slot's frames, the relay's sender byte, `Dropped`, `Serve`, `Resume` on it change nothing); a relay path a lead behind holds the game back; hostile bytes on a link never panic; the first copy wins and a different one is reported once; a drop round is answered with every frame run or held past the relay's, a round for a slot not live here is answered empty, long answers come in parts; only a peer's own `Frames` messages go on a link.
- `crates/relay/tests/hub.rs` (6 for N7): signals pass between live persons of one game only; signals too fast are dropped, and a ping is answered; a drop of a slot without direct links is reported at once; a drop of a slot with direct links waits for its peers' frames; answers out of turn are refused and a silent peer is closed; an answer with a marker or past a lead ends, and a peer that leaves is not waited for. `crates/relay/tests/rejoin.rs` (3 for N7): a drop with direct links counts every frame a peer ran; a drop and rejoin over a direct link keeps identical hashes; a rejoin while two others play on a direct link. The rejoin harness gives a rejoining peer a new process's clock, far below its slot's old stamps, and every rejoin test checks that it measures its round trips again; `delay.rs`: `a_wait_at_the_largest_delay_does_not_hold_it_there`.
- `crates/rtc/tests/pair.rs` (2, `#[ignore]`: real UDP on loopback): two pairs link (55 ms) and carry packets both ways; a hostile offer ends that attempt (matchbox's panic is caught) and a new attempt still links.
- `crates/app/src/game/client_direct_tests.rs` (3, over an in-process relay and the fake network `game::direct::fake`, with the relay path 40 ms late each way): frames come the direct way and hostile bytes injected on a link change nothing (the fallback not forced); three persons where one pair can never link and a client switched to relay only (`direct=0`), every game agreeing (the fallback forced; a mix of direct and relayed pairs); a link that goes dark mid-game falls back, the relay carries the frames, and once the route is back the pair links again and the link is used.
- `crates/tools/tests/soak.rs`: `four_ai_persons_play_a_2v2_over_direct_links_beside_the_relay`, in CI: N6's short soak with the four on the fake network behind the same bad networks on the direct path; every checkpoint agrees, row 1's cut and rejoin links again, and every pair opened and took frames the direct way first.
- `web-tests/rtc.mjs`: in Chromium and Firefox, two attempts in one page open with no ICE server (33 to 50 ms) and carry a packet each way, and a hostile offer ends its attempt without a panic.
- Unit tests: control packets, the shaped lane, ICE words, settings (`direct_links_are_on_with_no_servers_by_default`), `xtask deps` (`the_direct_links_stack_stays_inside_its_crate`, `versions_of_a_package_keep_their_own_dependencies`).

**The soaks** (local lane, on the install, 2026-10-22; counts and timings only). All runs native, with four persons on map index 30 and `--minutes 30`. The bad network is N6's (40 ms, jitter 0 to 30 ms, 2 % of chunks 200 ms late, stalls of 0.5 to 2.5 s every 45 to 150 s). It is on every relay link and, with `--transport rtc`, on every direct path too. Another session's sweep used about 14 of the machine's 32 cores throughout.

The games are AI games and end when one team wins. With the default trouble (cuts at 300, 600 and 1 200 s, a kill at 900 s), the direct-link game ended after the first cut. A second direct-link run therefore moved the four troubles to 60, 120, 180 (the kill) and 240 s. Delay shares come from each client's delay timeline.

| | rtc, default trouble | rtc, trouble at 60 to 240 s | relay, default trouble |
|---|---|---|---|
| Result | pass: 0 mismatches, no desync, every client exited with 0 | pass | pass |
| Game | won at tick 19 994 (5.3 min of game time), 8.3 min wall, 40 ticks/s | won at tick 52 132 (13.9 min), 21.6 min wall, 40 ticks/s | won at tick 98 793 (26.3 min), 34.4 min wall, 48 ticks/s |
| Rejoins | 1 cut: 12.1 s | cuts 8.4, 8.6 and 10.7 s; the kill 14.7 s | cuts 7.9, 9.2 and 7.9 s; the kill 13.9 s |
| Checkpoints in common | 662 | 1 717 | 3 269 |
| Delay, whole game | 10 to 30: 35 to 37 % at 10 to 18, 42 to 45 % at 19 to 24, 9 % at 25 to 29, 12 % at 30 | 10 to 30: 46 to 49 %, 37 to 38 %, 7 to 8 %, 7 % | 22 to 30: 0 %, 0 %, 12 to 13 %, 87 to 88 % |
| Delay, ticks 0 to 11 900 (before any cut in either default run; the same on each client that covers it) | 33 %, 45 %, 6 %, 16 % | – | 0 %, 0 %, 9 %, 91 % |
| Round trip peer to peer (`session_rtt_ms`, smoothed, all paths) | 251 to 286 ms | 238 to 287 ms | 333 to 440 ms |
| Round trip client to relay (`relay_rtt_ms`, smoothed at the end) | 201 to 279 ms | 199 to 240 ms | 178 to 241 ms |
| Direct link's own round trip (`rtt_ms`, smoothed at the end, over the 12 pairs) | 275 to 310 ms | 274 to 338 ms | – |
| Frames that came the direct way first | 78 % | 76 % | – |
| Pairs | 12 of 12 linked; 17 attempts, 0 failed; 5 fallbacks (the cut slot's links, and twice a link that went silent), every pair linked again | 12 of 12; 18 attempts, 0 failed; 6 fallbacks (the troubled slots' links), every pair linked again | relay only |
| Connect | 186 to 822 ms, median 374 | 155 to 1 585 ms, median 408 | – |
| Held back (`gated`) | 89 frames | 338 | – |
| Late ticks (over 100 ms), per client | 122 to 651 | 136 to 513 | 200 to 225 |
| "Waiting for ..." shown, per client | 0 | 1 or 2 | 38 to 67 |
| Longest gap between two ticks | 0.5 to 0.6 s | 1.6 to 1.8 s | 2.4 to 3.7 s |

On the bad network the direct path gives the peers a shorter round trip, because a lost packet costs only that frame's copy, which the relay's copy covers. The relay path runs over TCP, where a late chunk holds back every chunk after it. With direct links the delay mostly stayed between 10 and 24, and the players almost never saw "Waiting for". On the relay alone it sat at the largest delay, already in the first 11 900 ticks, before any trouble. Paced at the same delay, the relay-only clients ran more ticks per second (48 against 40). A lower delay leaves less room for a stall, so a stall shows as late ticks instead.

*Milder network* (`--latency 20 --jitter 5 --loss 0 --no-stalls`, 3 minutes, the four troubles at 30, 60, 90 (the kill) and 120 s, 11 250 ticks; both runs pass):

| | rtc | relay |
|---|---|---|
| Delay | 3 to 18; 71 to 88 % of ticks at 6 to 8, per client | 3 to 24; 67 to 78 % at 6 to 8 |
| Rejoins | 7.2, 13.1, 11.1 (the kill) and 8.6 s | 7.5, 8.0, 9.4 (the kill) and 11.9 s |
| Direct first | 29 % | – |
| Round trips (smoothed) | session 99 to 107 ms; the direct link's own 122 to 151 ms | session 101 to 113 ms |
| Connect | 54 to 155 ms; 6 fallbacks after the four troubles, all linked again | – |

On this network the shaped direct path is slower than the relay path. That comes from the harness, not the link. The netem proxies on the relay path run on their own threads. The direct path's shape (`game::direct::Shaped`) moves packets only when the client polls it, so each shaped stage waits for the next client update: about 35 ms more per round trip at 62.5 ticks/s. The soak therefore understates what the direct path gains. Unshaped on loopback, a link opens in about 55 ms (`crates/rtc/tests/pair.rs`). The wasm row of the soak was not run in N7 (it stays on the relay; see below).

*A delay fault these soaks found* (not in the direct links; fixed in `glasnost-net`, see "The delay" above). The first long soaks pinned the delay at the largest value: 83 % of ticks at 30 with direct links, 99 % on the relay. The first milder run stuck at 30 from the host's rejoin to the end. Two causes:
- one wait at the largest delay raised every later proposal;
- a peer restarted after a kill took its old process's stamps, still echoed by the others, for its own, and measured no round trip until its new clock passed the old one.

N6's "26 to 30 for the whole game" is very likely the first cause. The numbers above are from the fixed build.

**Seams for N8 and limits.**

- *Routers and firewalls.* With no ICE server a link forms only where the two peers reach each other's host addresses: on one machine or one LAN. Across the internet most pairs need at least a STUN server, and peers behind symmetric NAT or strict firewalls need TURN. Nothing has to be opened on a router for the relay path (outgoing TCP to the relay); a direct link uses outgoing UDP on ephemeral ports, and incoming UDP from the peer once ICE has opened the NAT binding. N8's guide should say which servers to name with `ice=` and that everything still works without them, on the relay.
- *Nothing phones home.* No STUN or TURN server is configured by default, and matchbox's default (a public Google STUN server) is overridden; a client contacts only the relay and the peers of its game. Naming a STUN or TURN server sends that server the client's addresses.
- *Host addresses.* Browsers hide host candidates behind mDNS names (`*.local`); peers on one LAN resolve them (webrtc-rs answers mDNS queries), but a headless browser in a container may not, which is why `rtc.mjs` turns the hiding off (`--disable-features=WebRtcHideLocalIpsWithMdns`, `media.peerconnection.ice.obfuscate_host_addresses`).
- *The wasm soak row stays on the relay.* Node has no `RTCPeerConnection` and no GPL-compatible package already in the repository's tooling provides one, so the wasm client under node keeps `direct = None` (its pairs show as relayed). The browser side runs in `rtc.mjs`. A browser soak row would need the wasm client in a page (Playwright, which `web-tests` already uses), the install's archive reads through a page API instead of `glasnostReadAt`'s synchronous file reads, and the same status lines and files written through the page.
- *Mixed browser and native pairs* use the same SDP and candidate fields but were not run.
- *TURN passwords* are kept in the settings file as typed.
- *Client logs.* webrtc-rs logs ICE, mDNS and connection-state lines at INFO and WARN into the client's log (a few dozen per attempt); they are harmless and could be filtered by target.
- *The soak's trouble after the end.* A trouble timed after the game ended (an AI game can end early) finds no game: a killed client restarted then cannot rejoin, and the soak fails on its exit. It happened once, in a 3-minute run whose game ended at 84 s; the harness could skip trouble once a game is over.
- *Security advisories on an unused branch.* `cargo deny check advisories` (not run in CI) lists RUSTSEC-2025-0052 (async-std unmaintained), RUSTSEC-2025-0134 (rustls-pemfile unmaintained) and seven 2026 advisories on rustls 0.21, rustls-webpki 0.101 and webpki-roots 0.22. They all come in through matchbox_socket's WebSocket signaller (async-tungstenite, async-tls), which matchbox 0.14 compiles on native with no feature to leave it out. glasnost-rtc never builds that path: it signals over the relay. No ignores were added; a later matchbox, or a fork without the signaller, would drop them. The lead accepted them as they are on 2026-10-22: re-check when a matchbox release can leave the signaller out.

### Network lab (N8c)

N7's links were tried only on one machine. N8c runs them behind real NATs and across the internet: a rootless lab of four homes, each behind its own NAT router, with symmetric NATs, a TURN server and blocked UDP; browser rows in the soak; and a game split between this machine and a rented server. Everything here is our own netcode (`// nocite: our own netcode`) or harness (`// nocite: our own harness`); the simulation crates do not change. Code: `crates/tools/src/soak.rs` (split soaks, external relays, browser rows, `soak compare`), `crates/netlab` (`glasnost-netlab`, the binary `glasnost-turn`), `crates/netlab/lab/nat-lab.sh`, `web-tests/net-client.mjs`, `crates/headless-web/src/lib.rs` (direct links in a page) and `crates/app/src/game/direct.rs` (`Signal::Decline`).

**Split soaks.** One game, played by several soaks, each running some of the rows, on a relay none of them started. The options:

- `--relay-url ws://HOST:PORT/` (or `wss://`) names the relay; `--no-proxy` sends the clients straight to it, without the bad-network proxies (so no cut trouble).
- `--rows 0,1` names the rows this soak runs.
- Without row 0, the soak needs the room code: `--code CODE`, or `--code-in FILE`, which it waits for (up to 10 minutes). `--code-out FILE` writes the code once this soak's rows are seated.
- `--row-arg R:ARG` passes an option to one row's client. `--client-arg ARG` passes it to every row.

The room seats a joiner in the next free row, so the parts must join in row order: each part's `--code-in` is the previous part's `--code-out`. `glasnost-headless soak compare [--minutes M | --ticks N] DIR DIR ...` reads the halves' row folders and applies the soak's pass rule to all rows together: every client exited with 0 and saw the end or the limit, no desync, no tick hashed two ways, and the rows share checkpoints.

**Browser rows.** `--browser-row R:chromium` (or `firefox`) with `--browser-script web-tests/net-client.mjs` makes row R the wasm client in a real page, driven by Playwright. The page has `RTCPeerConnection`, so it opens direct links through the browser, as the web client does (N7's node row could not). The driver takes the net-client's options and writes the same status lines, hash log, report and code file. It runs synthetic only (a page has no install folder) and turns off mDNS host names, as `rtc.mjs` does. Build the page package first:

```
cargo build --release --target wasm32-unknown-unknown -p glasnost-headless-web
wasm-bindgen --target web --out-dir target/headless-web-page \
  target/wasm32-unknown-unknown/release/glasnost_headless_web.wasm
```

**The lab's servers.** `glasnost-turn` (crate `glasnost-netlab`, test only) is a STUN and TURN server built on webrtc-rs's `turn` crate, and a NAT probe:

- `glasnost-turn serve --listen IP:PORT [--listen IP:PORT] --relay-ip IP --ports LO-HI --user NAME=PASS [--stats S]` answers STUN binding requests and TURN allocations. It relays from the port range on the relay IP and prints a counts line every S seconds: requests by kind, the allocations alive, and the packets and bytes it relayed. Those counts show whether a pair went through TURN.
- `glasnost-turn probe --server IP:PORT --server IP:PORT` asks two server addresses for its outside address from one socket. It prints `mapping endpoint-independent` when the NAT mapped both to one address and port, and `mapping dependent` (a symmetric NAT) otherwise.

`cargo xtask deps` keeps it out of the game: no client crate, the lockstep crate, the direct links' crate or the relay may depend on it (`the_lab_server_stays_out_of_the_game`). It is not a server to deploy. For a real game, use one you run (for example coturn) or trust (`docs/multiplayer.md`, section 3).

**The NAT lab** (`crates/netlab/lab/nat-lab.sh SCENARIO OUT [MINUTES]`, Linux, no root). The script runs itself again under `unshare -rnm`: a user, a network and a mount namespace. Every router and player is a network namespace inside it, so nothing outside changes, and everything goes when the script ends.

- *Topology.* An "internet" bridge on 203.0.113.0/24 (a documentation range) holds the server at .1 and .2 (the probe needs two addresses). The server runs the relay (:7400), `glasnost-turn` (:3478 on both addresses, relayed ports 49160 to 49200) and, for `wss`, a TLS proxy (socat, :7443) with a certificate from a lab authority made for the run. Home K is a router with WAN 203.0.113.(10+K) and LAN 10.0.K.1, and one player at 10.0.K.2.
- *NATs.* nftables on each router's WAN side. `masquerade` keeps a socket's port for every destination: an endpoint-independent mapping with port-dependent filtering (a port-restricted cone, the common home router). `masquerade fully-random` picks a new port per destination: an endpoint-dependent mapping (a symmetric NAT). Each home's NAT is checked first with `glasnost-turn probe`.
- *A firewall that drops unsolicited UDP.* Each router drops new inbound UDP in its input hook, as a home router does, before conntrack confirms the entry. Without that rule the lab's first run made no direct link at all: a peer's first check reached the router before the player had sent, conntrack confirmed an entry for it on the player's port, and masquerade then gave the player's own send another port. With the drop, a hole opens only by the player's own send, and two players punch through when both send.
- *The game.* One soak per home (home 0 hosts), each with `--rows K --relay-url ... --no-proxy --transport rtc --clean --no-trouble --synthetic`, `--ice` for that scenario, and the code passed down the homes in order. Then `soak compare` over the four.
- *Evidence.* Each router counts the UDP it forwards to each other home (`to_hJ`: that pair's direct path), to the STUN/TURN port (`turn`), to the relayed ports (`turn_ports`: a pair whose path is the other end's TURN allocation) and the TCP to the relay. It also counts the unsolicited UDP it dropped and, while UDP is blocked, the UDP blocked. With the TURN server's counts and the clients' `pairN` lines (attempts, opened, failed, fallbacks, connect time, round trip, frames first by each path), each pair's path is plain.
- `GLASNOST_BIN` names the folder of `glasnost-headless`, `glasnost-relay` and `glasnost-turn` (default `target/release`). `all` runs every scenario. Outputs (counts and timings only) go to `local-lane/m9/n8c/lab/`.

| Scenario | NATs (homes 0 to 3) | ICE servers | What happens |
|---|---|---|---|
| `eim-stun` | all endpoint-independent | STUN | all 12 pair-ends direct, 1 attempt each, connect about 620 ms, round trip 3 ms |
| `sym-one-stun` | home 0 symmetric | STUN | the 3 pairs with home 0 fail 4 attempts each in 3 minutes and stay on the relay; the other 3 pairs direct |
| `sym-both-stun` | all symmetric | STUN | every pair fails (4 attempts each) and stays on the relay |
| `sym-turn` | homes 0 and 1 symmetric | STUN + TURN | all 12 open. Pair 2-3 direct (connect about 1 030 ms); the 5 pairs touching home 0 or 1 through TURN (connect 2 030 to 2 040 ms, round trip 3 ms), 12 allocations, 91 948 packets (13.9 MB) relayed |
| `no-ice` | all endpoint-independent | none | no candidate but the private host address; every pair fails and stays on the relay; no UDP between homes |
| `udp-blocked` | all endpoint-independent, UDP dropped from the start | STUN + TURN | every pair fails (4 attempts each); no allocation (TURN over UDP is blocked too); the game on the relay |
| `udp-flap` (4 min) | all endpoint-independent | STUN | links open (about 620 ms); UDP blocked at 75 s for 40 s: each pair falls back once, the retry inside the block fails, the next opens again (3 attempts, 2 opened, 1 failed, 1 fallback per pair-end) |
| `wss` | all endpoint-independent | STUN | the relay through the TLS proxy (`wss://`, the lab authority in `SSL_CERT_FILE`); all 12 direct; relay round trip 6 to 7 ms smoothed |

Every scenario passed: 0 mismatches over 375 common checkpoints (500 for `udp-flap`), no desync, and no team won within the 3 minutes. The NAT kinds were as set in every scenario (`probe`). From `sym-turn` on, the delay stayed between 2 and 5 and the longest gap between two ticks was at most 60 ms. In `eim-stun`, which ran while a cargo build shared the machine, the delay reached 22 (71 % of ticks at 2), the longest gaps were 147 to 334 ms and each client had 2 to 4 stalls. `sym-one-stun` had delay 2 to 9 and gaps up to 141 ms. These runs used a build of the lab-server commit (`0371d94` on the branch, built before the rebase onto main, so it reports `3b2d11c40+dirty`) with the lab script and the browser driver beside it, uncommitted. The per-pair counters came in from `sym-turn` on. The first three scenarios counted all UDP between homes together, and their paths read from the clients' pair lines.

**Browser runs** (local relay, synthetic, `--transport rtc`, the same build):

- *Rows 1 Chromium and 2 Firefox, rows 0 and 3 native, 2 minutes.* Pass: 250 common checkpoints, 0 mismatches, 7 500 ticks each. All 12 pair-ends open at the first attempt; connect 16 to 265 ms; round trip 0 to 3 ms. Delay 2 to 9 (76 % at 2); longest gap 56 to 91 ms. A browser and a native client link: the mixed pairs N7 did not run.
- *Row 0 Chromium (host), row 1 Firefox `--relay-only`, rows 2 and 3 native, 3 minutes.* Pass: 375 checkpoints, 0 mismatches. The pairs with row 1 stay on the relay; the others open in 9 to 61 ms. Delay 2 to 4. Before the fix below, row 0 made 4 failed attempts to row 1 in the 3 minutes.

**A fault the runs found: offers to a relay-only peer.** A relay-only peer answered no offer, so the offering end tried on its back-off for the whole game, each attempt gathering candidates and, with TURN named, allocating: 4 failed attempts in 3 minutes, about 15 in 30. Now a peer with direct links off answers an offer with `Signal::Decline`, and the offering end makes no more attempts to that slot until the slot plays again (a rejoined process may have direct links on). The log says `direct link to slot N not made (the peer makes no direct links)`. Lobby and lockstep wire version 7. `client_direct_tests.rs`'s relay-only test plays 300 ticks and checks one attempt, one failure (it saw 3 and 3 before the fix). That test is the fix's only check: the browser run that found the fault ran before it, and the server runs had no relay-only peer beside direct ones.

**Across the internet** (a rented server; synthetic; on both ends a build of the per-pair-counters commit: `2a4c004` on the branch, built before the rebase onto main as `f3e854041`, which the binaries report). The server runs the relay and `glasnost-turn` on its public address, with ports open only for the runs. This machine sits behind a home router whose NAT the probe finds endpoint-independent.

Rows 0 and 1 run here, rows 2 and 3 on the server, as a split soak; the clients name the server for STUN and TURN (`--ice "stun:... turn:... username=lab credential=..."`). Run (a) is four rows here on the server's relay. b-rtc's AI game was won after 5.3 minutes, so c-rtc played the 30 minutes with `--no-ai` (each client still sends its frames every tick, with no orders); b-relay ran beside it for about 5 minutes near its start. The server has 2 cores and its public address on its own interface; its two clients used about 40 % of a core each and the relay about 27 %.

| | (a) 4 here, rtc, 10 min | b-rtc: 2 + 2, rtc, AI | c-rtc: 2 + 2, rtc, `--no-ai`, 30 min | b-relay: 2 + 2, relay only, AI |
|---|---|---|---|---|
| Result | pass, 575 common checkpoints, 0 mismatches | pass, 667, 0 | pass, 3 750, 0 | pass, 608, 0 |
| Game | won at tick 17 259 (4.6 min) | won at tick 20 024 (5.3 min) | 112 500 ticks, the full 30 minutes | won at tick 18 265 (4.9 min) |
| Relay round trip (smoothed, range) | 15 to 17 ms (14 to 50) | here 15 to 16 ms (14 to 178); server rows 1 ms | here 15 to 17 ms (13 to 337); server rows 1 ms | here 15 to 16 ms (14 to 51); server rows 1 ms |
| Pairs | 12 of 12 direct, 1 attempt each, connect 26 to 67 ms, round trip 3 ms | 12 of 12, 1 attempt each, no fallback. Pairs inside one side: connect 27 to 57 ms, round trip 3 ms. Pairs across: 347 to 408 ms, 21 to 25 ms | the same: 12 of 12, no fallback in 30 minutes; across, connect 321 to 410 ms, round trip 22 to 25 ms | relay only |
| Frames first by the direct path | over 99.8 % | inside a side: 99.8 % (here), 17 % (server); across: under 0.3 % | inside: 99.9 %, 19 %; across: under 0.3 % | – |
| Delay | 2 to 7, 83 % at 2 | 2 to 16: 63 % at 3, 24 % at 4 | 2 to 24: 57 % at 3, 21 % at 4 | 3 to 9: 42 % at 3, 45 % at 4 |
| Longest gap between two ticks | 39 to 47 ms | 46 to 66 ms | 157 to 188 ms (1 or 2 stalls per client) | 83 to 90 ms |

The home NAT and the server's public address link with STUN at the first attempt: the server's rows answer from their host address, and the home side's checks go out through its endpoint-independent NAT. No pair needed TURN. The TURN server relayed 700 packets in all (the connectivity checks), while it held up to 35 allocations at once of its 41 ports: every attempt allocates when TURN is named, and an allocation stays until its lifetime runs out (TURN's default is 10 minutes), so three games inside ten minutes came close to the range.

The direct links bring the delay down only where they are faster than the relay. With all four players far from the relay (a), the frames came the direct way (3 ms against 15 to 17 ms) and the delay sat at 2. With the relay beside the server's players, a frame between the two sides took the same internet path either way, and the relay's copy came first almost every time. The direct copy goes through the WebRTC stack on both ends; why it loses by a few milliseconds was not measured. The delay then followed the 15 to 25 ms round trip across: mostly 3 and 4, a little lower than relay only (b-relay: 3 and 4 nearly evenly, up to 9). The relay's `drops` counts in the minutes each game ended came from the clients leaving; no client lost its link (`lost=0`, `reconnects=0` everywhere).

Server work: a separate login, ports 7400/tcp, 3478 and 3479/udp and 49160 to 49200/udp opened for the runs and closed after them, remote players synthetic only, every process stopped after each run. Outputs: `local-lane/m9/n8c/server-*` (without the client logs and settings files, which hold addresses).

**A 30-minute synthetic game** (local relay, 4 native, rtc, a build of the NAT-lab commit: `a5a8a39` on the branch, built before the rebase as `d789f8c07`): 112 500 ticks on every row (the full 30 minutes of game time; no team won in this game, while the AI games across the internet were won after 4.6 to 5.3 minutes), 0 mismatches over 3 750 common checkpoints, delay 2 to 7 (86 % at 2), longest gap 49 to 82 ms, no stall.

**The M9 rehearsal** (2026-10-23, approved by the lead as a rehearsal, not the exit run; build `423cf07` on both ends; the rented server again, with a new login whose uid no running process used, ports open only for the runs, the login removed afterwards; this machine shared with two intake agents' builds and tests). Outputs, counts and timings only: `local-lane/m9/rehearsal/`.

| | B: 2 here + 2 on the server, synthetic, rtc, AI | A1: 4 here on the install, relay only through the server, AI, map 30 | A2: the same, `GLASNOST_LOG=debug` |
|---|---|---|---|
| Result | pass, 4 371 common checkpoints, 0 mismatches | pass, 70, 0 | pass, 990, 0 |
| Game | 131 250 ticks: the full 35 minutes of game time, no winner | over at tick 2 126 (34 s): houses 0 and 2 lost with nothing left | over at tick 29 708 (7.9 min): house 2 the last |
| Trouble | row 3 (server) killed at 120 s, readmitted after 4.9 s; row 1 (here) killed at 180 s, readmitted after 4.5 s | kills planned at 600 s and 1 200 s; the game ended first | the same |
| Links | 12 of 12 direct at the first attempt; across the internet about 0.5 s to open | relay only | relay only |
| Delay | 2 to 30, 51 % at 3 | 3 to 30, mostly 10 to 17 | 3 to 30, 52 % at 4 or 5 |

Finding, handed to its own step: in A1 and A2 a house never built a base and lost its starting army without seeing an enemy (A2, house 0: army 11 at tick 0, 5 at tick 1 500 with "seen enemies 0", 0 at 4 500). The clients agreed throughout, so it is no desync; a local game on the same map (no delay, or a simulated 60 ms link) showed no loss in 2 to 3 minutes.

**The finding's cause** (follow-up step, 2026-10-23, local lane on the install, never the server; counts and timings only, in `local-lane/m9/reh-ai/`). It needs no network. `crates/tools/tests/four_houses.rs` (ignored, local lane; options in its header) plays the room's setup (four rows, teams 1, 2, 1, 2, random countries and starts) on map index 30 outside the network, with the AI inside the tick or with one brain per house outside the simulation whose orders run 0 or 8 ticks later behind the bot's repeat filter, and logs when and how each house's objects leave the map. Seeds 1 to 12, each three ways, 3 000 ticks: 26 of 144 houses never got a construction yard, and 17 had nothing left at the end. 20 of the 26 still had their yard vehicle at tick 3 000. 16 of those came from the two starts near cells (110, 25) and (78, 163), where the vehicle has to move before it can deploy, and the other 4 from three other starts. There were two causes:

- *The simulation* (fixed; S-0207 claim 11, S-0415 Terms and claim 2). The AI's deploy step counts a vehicle within two cells of its move goal as idle, so it gives the deploy order while the vehicle still has stored path steps. The order cleared the destination but kept the steps. S-0206's in-progress test saw them, so the vehicle never became still, never made its deploy attempt and stood in *unload* for the rest of the game (seed 1, house 1: the order at tick 196, in *unload* at (107, 25) from tick 199 to 3 000). Now the deploy order drops a driving vehicle's stored path along with its destination, as the stop and behaviour orders already do. A hover follows S-0503 claim 19 (Q-1773, answered by S-0503 r27, built at A3 intake 14).
- *The AI* (fixed). While the house had no yard, the crate step (S-0520) and the deploy-spot step both sent orders to the yard vehicle, and the two orders alternated: the vehicle went back and forth between (64, 162) and (66, 161) for 2 800 ticks (seed 2, house 1). With the simulation fix alone it still did. The crate step no longer picks a vehicle that deploys into a construction yard.

What follows from either cause: the house builds nothing, its army leaves on the opening wave, an enemy wave that reaches the start kills the yard vehicle, and the short-game defeat (S-0414 claims 2 and 5) removes everything else in the same tick. For example, in seed 12 house 3's vehicle was killed at tick 1 574 by house 0's tanks.

With both fixes, on main after intake 7, the same 36 games leave 0 of 144 houses without a yard. The last first yard comes at tick 390, where it was 2 946 before. Houses that had a yard before get it within 2 ticks of the same time; the median rises from 49 to 158 only because the 26 late yards are now counted. 11 houses still have nothing left at tick 3 000. Each of them first lost its yard to an enemy attack, between ticks 1 066 and 1 999: in seed 10 at delay 8, for example, house 3's light tanks reached house 0's base (its start about 60 cells away) at tick 675 and killed the yard at 1 066. That is how the map plays with four AI starts, and nothing was changed for it.

Local soaks (`glasnost-headless soak --map 30 --no-trouble --minutes 1`, `GLASNOST_LOG=debug`; three rooms with `--latency 60 --jitter 30` and one `--clean`; each room draws its own setup). Before the fixes, 3 of the 16 bots (in 2 of the 4 rooms) had "Power: have 0, buildable types 0" at tick 1 500, which is house 0's picture in A2. After the fixes, none of the 16 did. All 8 soaks passed with 0 mismatches. The rehearsal games cannot be replayed, so A1 and A2 are matched to this mechanism by their picture and are not proven. A room whose starts need no deploy move shows nothing, which is why the earlier local probe saw no loss. In A2's status line, "seen enemies" counts live enemy objects on cells the house has explored; where house 0's 6 lost units went is not in its logs.

Checks (main 299ddf4 before and after): the oracle arena's CSVs are byte-identical (31 of 96, 13 of 24 and 2 of 35 pairings failing). The M7a and M7b sweeps 1 to 16 print the same output. In the M6c sweep every seed's smallest counter is the same and some medians move with the games (crate orders 165 → 188, crates picked up 16 → 13, deploy orders 84 → 87). The `duel-diagonal` and `duel-m6c` goldens are regenerated; `duel-m7a` and `duel-m7b` did not move. There is no new state, so the formats are unchanged.

**Tests.**

- `crates/tools/tests/soak.rs`: `a_game_split_in_two_halves_agrees`, in CI: two soaks of two rows on the test's relay, the second joining with the first's code; row 2 is cut and rejoins; each half passes, and the halves together agree on every checkpoint (16 in common).
- `crates/deterministic/sim/tests/claims_s0415.rs`: `claim_2_deploy_order_drops_the_stored_path`, `claim_2_deploy_order_while_moving_deploys` (the rehearsal finding); `crates/deterministic/ai/tests/crates.rs`: `a_yard_vehicle_is_not_sent_at_a_crate`. `crates/tools/tests/four_houses.rs` is local-lane.
- `crates/netlab/tests/turn.rs`: a probe of two of the server's ports on loopback sees one address, and a TURN client allocates, sends a datagram through the relay to a peer and gets its answer, all counted.
- `crates/app/src/game/client_direct_tests.rs`: the relay-only test checks the Decline (one attempt, one failure in 300 ticks). `crates/relay/tests/hub.rs`: a Decline passes between live persons like any signal.
- `crates/tools/src/soak.rs` unit tests: `a_split_half_parses`, `split_options_are_checked`, `browser_rows_are_cut_not_killed`, `halves_compare_as_one_game`.
- The lab, the browser rows and the server runs are local-lane: they need namespaces, browsers or a server, and stay out of CI.

**Seams and limits.**

- *The lab's NATs are Linux's.* Masquerade with port preservation is a good model of a cone NAT and `fully-random` of a symmetric one, but real routers vary (port-preserving symmetric NATs, hairpinning, short UDP timeouts). The lab has no IPv6 and no NAT in front of the server.
- *TURN over UDP only.* With UDP blocked the game falls back to the relay, and TURN does not help: neither the lab server nor the client's ICE list offers TURN over TCP or TLS. A player behind a UDP-blocking firewall plays on the relay, which is correct and only slower.
- *Allocations without need.* With TURN named, every attempt allocates, even for a pair that links directly: 12 allocations in a 4-player game, kept until they expire. The rented server's 41 ports held up to 35 at once; a TURN server's port range needs room for every attempt of the last lifetime.
- *A relay beside some of the players* makes the direct path between those players and the others no faster than the relay's, so the frames come the relay's way first. The links still open and carry the frames; they gain only where the relay is the longer way round.
- *The lab's own fault, fixed.* The script's background jobs were shell functions, so cleanup killed the forked subshell and left each run's relay, TURN server and TLS proxy alive in its abandoned namespaces (one set per run). The jobs now exec into `ip netns exec`, and a 1-minute run leaves nothing behind.
- *The order counts differ by row.* In the browser runs one row gave 453 orders and another 9 in the same game; it was not looked into.
- *A `+dirty` build id* never matches a clean one, so the halves of a split game must come from the same tree state; the lab and browser runs used one dirty build on one machine.
