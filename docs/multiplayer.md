# Playing Glasnost over the network

This guide is for whoever sets up and runs a network game: build and run the relay, open it to the internet, host and join a 2v2, and collect evidence when something goes wrong. It assumes you know the game and Linux, not our network code. The design behind it is in [development.md](development.md), "Multiplayer (M9)"; each section below names the part that holds the details.

## How a network game works, in one paragraph

Every player runs the whole game on their own machine, from the same start values, and the players send each other only their orders. Every few ticks they compare a hash of the game state; if two hashes differ, the games have gone apart (a *desync*) and each player writes a report. All players meet through a **relay**, a small server program (`glasnost-relay`) that makes rooms, hands out room codes and passes each player's messages on to the others. The relay never runs the game and never sees your game files. Optionally the players also open **direct links** (WebRTC) between themselves, which only make orders arrive sooner; the relay always carries every message too, so a direct link that fails changes nothing about the game.

What you need:

- One machine to run the relay. It needs no copy of the game, a few MB of memory and one open TCP port. The 2v2 in the lead's exit run needs nothing more.
- For every player: the same Glasnost **build** and the same game install (the same rules and map files). The lobby refuses a player whose rules or map differ, and names the difference. See the build note below.

**The build id names the commit.** The lobby compares a build id such as `glasnost 0.0.0 (a1b2c3d4e)`: the version and the short hash of the commit the program was built from. A tree with uncommitted changes to tracked files reads `(a1b2c3d4e+dirty)`, and a build outside a git checkout `(unknown)`; such builds are not known to match anything, so build every player's client from a clean checkout of one commit. A player on another commit is refused when they join, and the screen names both ids ("That room runs X; this is Y."). The multiplayer screen shows your own id at its foot, `glasnost-relay --version` and `glasnost-headless --version` print theirs, and the relay prints its own on its second line. Desync and pathing reports carry the id too. The id is read when the program is built, and Cargo reads it again when the commit or the index changes; if you only edited files without staging them after a build, run `cargo clean -p glasnost-net` to force a fresh read. Setting `GLASNOST_COMMIT=NAME` while building names the commit yourself, for a build outside git.

## 1. Build and run the relay

From the repository root:

```sh
cargo build --release -p glasnost-relay
target/release/glasnost-relay --listen 0.0.0.0:8787
```

The first line it prints is `glasnost-relay listening on ws://ADDR/`, the second `build glasnost 0.0.0 (a1b2c3d4e)`. Stop it with Ctrl-C. After that it prints one line of counts every minute (below) and nothing else: there is no access log, and the line holds no room codes, names or addresses. The operating system shows who is connected (for example `ss -tn | grep :8787`).

The line looks like this (rooms, connections and games running now; the rest counted since the last line, whose length in seconds ends it):

```
stats rooms=2 connections=7 games=1 in=14210msg/3011200B out=39874msg/8452110B refused=3 drops=0 rejoins=0 over=60s
```

`in` and `out` are WebSocket messages and their bytes (`out` counts each receiver); `refused` is requests the relay refused (a wrong build, a full room, a wrong code, or the connection limit); `drops` are players lost from a running game; `rejoins` are reconnects begun. A relay with nothing happening still prints the line with zeros, so a missing line means a stuck relay.

| Flag | Default | Meaning |
|---|---|---|
| `--listen ADDR` | `127.0.0.1:8787` | Address and port to listen on. The default accepts only the same machine; use `0.0.0.0:PORT` to accept other machines. A bad address exits with status 1. |
| `--max-rooms N` | 256 | Most rooms at once; more are refused. |
| `--max-connections N` | 1024 | Most connections at once. |
| `--grace SECONDS` | 60 | How long a dropped player has to come back before their house surrenders. See section 5. |
| `--stats SECONDS` | 60 | The time between the counts lines above; 0 prints none. |
| `--version` | | Prints the build id and stops. |
| `--help` | | Prints the usage. An unknown flag exits with status 2. |

Limits that are not flags (fixed in the relay): a message may be at most 64 KiB; a connection may send about 200 messages a second (400 at once); a connection that is in no room for 2 minutes, or silent for 15 seconds, is closed; at most 32 refused requests per connection (this stops anyone guessing room codes); chat is limited to a line a second. None of these should matter to a 2v2.

Leave it running under `tmux` or as a service; if it stops, every game on it drops (players can reconnect only to a relay that still holds their game, so a relay restart ends the games on it).

The relay speaks plain WebSocket (`ws://`) and no TLS itself. Browser players on an `https://` page need a TLS proxy in front of it (section 2), and native players can then use the same `wss://` address.

## 2. Open the relay to the internet

Skip this section for a game on one LAN: use the relay machine's LAN address.

1. **Forward the port.** On the router in front of the relay machine, forward TCP port 8787 (or the port you chose) to the relay machine. If you use the proxy below, forward TCP 443 to the proxy machine instead, and keep the relay itself closed to the outside.
2. **Allow it in the machine's firewall.** For example, with firewalld (Fedora): `sudo firewall-cmd --add-port=8787/tcp` (add `--permanent` to keep it). With ufw: `sudo ufw allow 8787/tcp`.
3. **Give players the address.** `ws://YOUR.HOST.NAME:8787/` (the trailing slash matters little, but use it). A name that follows your home IP (dynamic DNS) is easier than a number that changes.
4. **Try it from outside** before the game: from a machine on another network, `curl -i http://YOUR.HOST.NAME:8787/` should end at once with `curl: (52) Empty reply from server`. The relay accepts the connection and closes it, because a plain HTTP request is not a WebSocket; that proves the port is open. "Connection refused" or a hang means it is not reachable.

Plain `ws://` over the internet means the room code, the names and the chat can be read by anyone on the path. Nothing secret is sent (the game files and rules never cross the network), but do not treat a room as private.

### Browser players: wss:// through a reverse proxy

A page served over `https://` may only open `wss://` connections. The relay does not do TLS, so put a proxy that does in front of it. **The native client speaks `wss://` too** (rustls; it trusts the operating system's certificate roots, so a certificate from Let's Encrypt or any public authority works, and a self-signed one is refused with "cannot reach the relay at ...", naming the certificate). Everyone, native and browser, can therefore use the one `wss://relay.example.org/` address, and the relay can bind to `127.0.0.1:8787` with only the proxy exposed. (A machine whose certificate store is empty, such as a minimal container, cannot verify any certificate; use `ws://` there.) The relay's own `ws://` port still works for a LAN.

These are examples; none of them is part of Glasnost, and they were written from the proxies' documentation, not run here. Replace `relay.example.org` with a name that points at the proxy machine.

Caddy (gets and renews the certificate by itself; a WebSocket needs no special setting):

```
relay.example.org {
    reverse_proxy 127.0.0.1:8787
}
```

nginx (certificate paths are yours; the two `Upgrade` lines are required, the long timeout is a safety margin because the relay itself sends a keepalive every 2 seconds):

```
server {
    listen 443 ssl;
    server_name relay.example.org;
    ssl_certificate     /etc/letsencrypt/live/relay.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/relay.example.org/privkey.pem;
    location / {
        proxy_pass http://127.0.0.1:8787;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 3600s;
    }
}
```

Browser players also need the game page itself. `cargo xtask web --release` builds it into `dist/web/`; serve that folder over https with any static file server (the proxy can do it). Each browser player picks their own install folder on the main menu; nothing is uploaded.

### Pointing the client at the relay

- **Native:** `--relay ws://HOST:8787/` on the command line, or type the address into the Multiplayer screen's *Relay* field (it is remembered). Without either it is `ws://127.0.0.1:8787/`.
- **Browser:** add `?relay=wss://relay.example.org/` to the page address, for example `https://play.example.org/?mode=game&relay=wss://relay.example.org/`. Or type it into the field. A page served from `http://localhost` may use `ws://`.

## 3. Direct links, STUN and TURN

Direct links are on by default and need nothing from you: each pair of human players tries one, the relay carries the offers and answers, and if it works orders arrive sooner (in the soak runs on a simulated bad network, about three quarters of frames arrived the direct way first and the game needed less input delay). If it does not work, the game uses the relay alone and is exactly as correct.

What decides whether a pair links:

- **On one LAN or one machine** it works with no setup.
- **Across the internet, behind home routers**, most pairs need at least a **STUN** server, which tells each player their outside address. Pairs behind symmetric NAT or a strict firewall also need a **TURN** server, which relays the traffic. Nothing is opened on a router for this: the links use outgoing UDP, and the answer comes back through the hole that opens.
- **What the network lab and an internet run showed** (docs/development.md, "Network lab (N8c)"):
  - Two players behind ordinary home routers (a "cone" NAT, which keeps a player's port for every destination) link with a STUN server, in under a second.
  - If either player of a pair is behind a symmetric NAT (a new port for every destination), STUN is not enough. That pair links only through a TURN server, in about 2 seconds. Without TURN it stays on the relay.
  - With UDP blocked altogether, no pair links, and TURN does not help (it runs over UDP here). The game plays on the relay, correctly, just with more input delay on a bad network.
  - A pair that loses its link (UDP cut for a while) falls back to the relay and links again at the next attempt after UDP is back: the attempts come 15 seconds after the fallback, then 30, doubling up to 2 minutes apart.
  - Without any ICE server, in the lab where every player was behind a NAT, no pair linked.
- **No STUN or TURN server is configured by default**, on purpose. Without one, a pair across the internet will often fail to link and use the relay. The client contacts only the relay and the other players of its game. If you name a server, that server learns the player's addresses; use one you run (for example coturn) or one you trust. We do not recommend or supply one.

To name servers, give every player the same kind of setting:

| Where | How |
|---|---|
| Native, command line | `--ice stun:HOST:3478` (repeat `--ice` for more servers). For a TURN login the words go in the same flag list: `--ice turn:HOST:3478 --ice username=NAME --ice credential=WORD`. One login serves all the servers named. |
| Native, kept | A line in the settings file, `~/.config/glasnost/settings.txt`: `ice=stun:HOST:3478` or `ice=turn:HOST:3478 username=NAME credential=WORD` (space-separated; the password is kept as typed). The command line wins for that run. |
| Browser | `?ice=stun:HOST:3478` in the page address (space-separated URLs, `%20` for the space), with the same `username=` and `credential=` words. |

**Running your own TURN server.** Use a real one, such as coturn; the repository's `glasnost-turn` is a test tool for the lab and is not meant to be deployed. A TURN server needs its listening port open for UDP (usually 3478) and its whole range of relay ports open for UDP too. Each player's client allocates a relay port for every attempt at a link, even one that then links directly, and again for every retry; a port stays taken until the allocation's lifetime runs out (10 minutes by default). So the range needs room for every attempt of the last ten minutes, across all the games using the server: in the runs, three 4-player games inside ten minutes held 35 ports at once. The TURN login is shared by every player who names the server.

**Relay-only switch.** To keep a player entirely on the relay: `--relay-only` (native), `?direct=off` (browser), or the settings line `direct=0`. Use it to rule direct links out when you suspect them.

**What the player sees.** Nothing on screen: there is no notice when a pair links or falls back, by design. A player who wants to look reads the client's log (native: the terminal it was started from, at the default log level; browser: the developer console). The lines are:

- `direct link to slot N open after MS ms`: that pair linked.
- `direct link to slot N not made (WHY)`: the attempt failed (it gives up after 10 seconds); the offering end tries again after 15 seconds, then 30, doubling up to 2 minutes.
- `direct link to slot N lost (WHY); the relay carries its frames`: an open link went silent for 3 seconds or closed; the pair is on the relay again and tries a new link.
- `direct link to slot N not made (the peer makes no direct links)`: that player is relay-only. Their client turns the offer down, and no more attempts are made to them while they play.
- `direct links: off; the relay carries every message`: relay-only is on.

A fallback never causes a desync and the game never waits for a link. If two players' orders ever arrive twice and differ, the chat area shows "X's orders came twice and differed"; that is a fault to report (section 7).

The native client also logs a few dozen lines per attempt from the WebRTC library (ICE and mDNS messages, including warnings such as "could not listen udp fe80::..."). They are harmless.

## 4. Host and join a 2v2

Every player starts the client (native shown; a browser player opens the page of section 2):

```sh
cargo run --release -p glasnost-platform-native -- --game --game-dir DIR --relay ws://HOST:8787/
```

or the built binary `target/release/glasnost`. `DIR` is the player's own install.

**Host:**

1. Main menu: **Multiplayer**. Type a name, check the relay address, click **Host**.
2. You are now in the room, which looks like the skirmish setup with the room code in its frame title. Read the code to the other players (eight characters; case, dashes and spaces do not matter when typing it).
3. Pick a map with at least four start positions. Click the first button of rows 2 to 4 until it reads **Open** (a place for a person); set unused rows to **Closed** (or **AI** for a computer player). A row's button cycles through Open, AI and Closed. The host sets the map, the game options (credits, units, flags, speed) and every row nobody has taken.
4. Set your own row: country, colour, team, start position.

**Guests:** Main menu, **Multiplayer**, type a name, type the room code, click **Join**. The guest takes the first open row and sets its own country, colour, team and start. For a 2v2 the teams are, for example, host and one guest on Team 1, the other two on Team 2. Teams are set by each player on their own row (the host can change AI rows).

**Ready and start.** Every change anyone makes, including a join or a leave, clears everybody's ready mark. When the setup is final each guest clicks **Ready** (it reads **Not ready** to undo). The host's button **Start game** is offered once every person's row is taken, every guest is ready and the setup has no problem; a note under the rows says what the room is still waiting for ("Waiting for everyone to be ready.", or that you lack the room's map). The relay chooses the random seed and sends everyone the start.

**Chat.** In the room there is a chat box with a **Say** field. In the game press Enter, type, press Enter to send; Escape drops the line. The last lines show over the battlefield for 10 seconds. A line is cleaned to 200 characters, and lines faster than one a second on average (five at once) are refused. Notices from the game (a player left, a desync) appear in the same place.

**What the banners mean.**

- "Waiting for the other players to load..." right after the start: some player's machine is still loading the map.
- "Waiting for NAME..." (several names are comma-separated) appears when a tick cannot run because that player's orders have not arrived for a full second. Everybody waits, since the game cannot go on without them. Usually it clears in a moment (a lost packet or a hiccup). If it stays, see the table in section 7. A player who is silent for 15 seconds is treated as dropped (section 5).
- "Waiting for the relay..." / "Connecting to the relay..." on the Multiplayer screen: the connection is being made.

**Speed.** The game speed is the room's, set by the host; nobody can pause or change speed alone in a network game, and the options menu does not stop the game.

**Leaving.** **Leave** in the room takes you out of it and back to the Multiplayer screen. **Leave** (shown as **End** in the game) during a game is a drop: the others see the grace-period notice and the player's house surrenders when it runs out. A player who leaves on purpose does not get a **Reconnect** button.

## 5. When a player drops, and reconnecting

If a player's connection fails (or the client is closed or killed), everyone else sees "NAME left the game; they have 60 seconds to reconnect." and the game goes on without them; their house stands idle.

- **The grace period is game time**: 60 seconds of play at the room's speed by default, set with the relay's `--grace`. It is the same whatever the machines' speed.
- **The dropped player** returns to the Multiplayer screen with "Reconnect within about N seconds to play on, or your house surrenders." A button **Reconnect to game CODE** is shown under **Host** and **Join**. It is also shown after restarting the client (the game's room, slot and a private ticket are kept in the settings file while a game runs), as long as the relay field is not empty and the relay is still the same one. Click it.
- While it works the banner reads "Loading the game...", "Waiting for the game...", "Receiving the game (k of n parts)...", "Catching up with the game (tick N)...", then "You are back in the game." The other players see "NAME is reconnecting; sending them the game." (the lowest-numbered live player sends the game, a few MB) and then "NAME reconnected." Expect it to take roughly 5 to 15 seconds on a good link; the loopback runs took about 4 seconds.
- **Past the grace period** the house surrenders on every machine on the same tick ("NAME did not come back in time: their house surrendered."), and a rejoin is refused ("too late to rejoin..."). The client then forgets the game.
- Only one player can be rejoining at a time; a second one retries after a moment.
- If the relay itself restarts, the game is gone and cannot be rejoined.

## 6. Reports: desyncs

A desync means two machines computed different games. Nothing stops the game when it happens, but it is the one failure this milestone must not have, so each player who sees it saves a report.

**What you see.** "The games went apart at tick N (a desync)." and, a few ticks later, "Desync at tick N; report saved to PLACE (digest XXXXXXXXXXXX)." and, for every other player's report, "NAME saved a desync report of tick N (digest XXXXXXXXXXXX)."

**Where the file lands.** `desync-reports/desync-CODE-tickN-slotK.gdsync` under the folder the native client was started from (the folder is made if needed); in a browser the file goes to the downloads. A file holds the whole game state at that point, typically 3 to 10 MB on the stock maps, so do not email a pile of them; do not commit them either. Keep them out of git.

**Collect two** reports from the two sides of the mismatch (the notices name the slots). To check a file is the one a player's game wrote, compare the first 12 hex digits of `sha256sum FILE` with the digest in the notice.

**Compare them** on a machine that has the same install the game ran on:

```sh
cargo build --release -p glasnost-tools --bin glasnost-headless
GLASNOST_GAME_DIR=DIR target/release/glasnost-headless desync A.gdsync B.gdsync
```

`GLASNOST_GAME_DIR` is the install folder. Without it only reports of the synthetic test rules can be read. The exit status is 0 when the two reports agree, 1 when they differ, 2 when they cannot be compared (a file that is not a report, an install without that map or with other rules). The text starts with each file's facts, then "The first differing tick: N.", then whether each side's replay reproduces what that player's machine recorded (if one does not, that machine's run changed outside the shared rules: another build, a platform difference, a hardware fault), then whether the merged orders differ (they should never), then a field-by-field diff of objects, houses and game state. Paste the whole output when you report.

**Test desyncs.** A debug build of the native client reads `GLASNOST_PERTURB` (for example `credits:300:0:100`, or `none` for the other player) to force a desync. Only for tests; such a client names a different build, so it plays only with other test clients.

**Pathing reports** are a separate tool for "a unit took a strange route". In the game press **M** (mark) as it happens, then **P** (save pathing report) within 20 seconds. The file lands in `path-reports/path-TICK-N.gpath` under the working folder (a browser download). Read it with `target/release/glasnost-headless path-report FILE` (see [development.md](development.md), "Play-note fixes (M8 step A4, part 1)" and the README). Both files hold a save of the game: keep them out of git.

**What to send to the main session** (without game files): the room code and the time; the build id each player's screen shows (the commit, if the id says `unknown`); the names and tick of the notices; both `.gdsync` files (or their digests and a note saying you have them) and the full output of `glasnost-headless desync`; each player's terminal log; the relay's single line and the time it was started; whether direct links were on, and any `ice=` setting.

## 7. When something goes wrong

| What you see | Meaning and what to do |
|---|---|
| "The relay refused: the build differs from the room's." | The player's build id (version and commit) differs from the host's; the screen shows both ("That room runs X; this is Y."). Everybody must run the same commit, from a clean tree (a `+dirty` id never matches a clean build). |
| "...the rules hash differs..." | The player's install produces different rules from the host's: a different game version, or a modified install. Everybody needs the same files. |
| "...the map hash differs..." / "you lack the room's map" | The player does not have the map the host chose, or has a different copy. The host picks a map all of them have. |
| "...the protocol differs..." | The players' clients speak different versions of the network protocol: mixed commits. Update to the same commit. |
| Room is full / playing / no such room / too many rooms | The room has no open row, has begun (a started game takes only a reconnect), the code is wrong (check the letters; the relay closes a connection after 32 wrong guesses) or the relay is at its room limit. |
| "cannot reach the relay at ADDR: ..." | Nothing answers there. Check the address and port, that the relay is running (`ss -ltn` shows its port), the firewall and the forwarded port. From outside, `curl -i http://HOST:8787/` should say "Empty reply from server" (reachable); a refusal or a hang means not. For `wss://`: check the proxy and that its certificate is valid for that name (a native client refuses one the system does not trust). |
| "cannot reach the relay at wss://...", naming a certificate | The proxy's certificate is self-signed, expired, for another name or from an authority the operating system does not trust. Fix the certificate; there is no switch to skip the check. |
| Browser page cannot connect | A page served over `https://` can only open a `wss://` relay (section 2). Check the address in `?relay=` and the proxy. |
| "The game cannot start: NAME's map hash differs from yours." | The start-time check found a mismatch after loading; same remedy as above. |
| "Waiting for NAME..." for a long time | That player's machine or link has stalled. Wait up to 15 seconds: the relay then drops them (section 5). If it happens to many players, note the time and who was waiting; try `--relay-only` or a different relay machine. |
| Constant small "Waiting" with high ping | The game adapts its input delay to the slowest round trip. Very bad links lift the delay and make orders feel slow; this is not a fault. Note the ping. |
| Client says its link closed and shows Reconnect | Section 5. |
| "Desync at tick N" | Section 6. Keep the reports. |
| "NAME's orders came twice and differed" | A direct link and the relay gave different copies of the same orders. Report it with the reports of section 6; rerun with `--relay-only` to rule direct links out. |
| Direct links never open | Not an error. See section 3: add a STUN server, or accept the relay. |

## 8. The lead's exit run: a 2v2 across networks

Goal: a 2v2 of four people (or fewer people and AI), on at least two different home networks, 30 or more minutes of play, no desync. This is the lead's run; the milestone cannot be closed without it.

**Before**

- [ ] One machine runs the relay: `target/release/glasnost-relay --listen 0.0.0.0:8787` (and the proxy if anyone is in a browser). Check from outside with `curl -i http://HOST:8787/` ("Empty reply from server" means it is reachable).
- [ ] Every player builds the same commit (the multiplayer screen shows the same build id for all; the lobby refuses another). Every player has the same install.
- [ ] Decide: direct links on (default; STUN named with `--ice` if you have one) or `--relay-only`. Ideally run once each, noting which.
- [ ] Each player starts the client from a folder they will keep (the reports land there), with the terminal left open so the log is kept: `target/release/glasnost --game --game-dir DIR --relay ws://HOST:8787/ 2>&1 | tee client.log`.

**Who does what**

- Host (you or one player): Multiplayer, Host, read out the code, set the map (four start positions), rows 2 to 4 Open, own team.
- Others: Join with the code; set country, colour, team (two and two); Ready.
- Host: Start game. Everyone writes down the **start time** and the game speed.
- During the game: everyone plays normally. Say in chat when something odd happens, with the time. Every 10 minutes or so, each player says their rough ping (any tool) and whether they have seen "Waiting".
- At 30 minutes the game can be left running to a win or surrender; the check is that no desync notice appeared.

**Optional, to exercise the rest** (do these in a second run if you want a clean first one): one player closes their client or pulls their network for 10 to 20 seconds mid-game and uses **Reconnect** (section 5); one player on a browser (section 2).

**What to note**

- Start time, end time, map, speed; who was on which network, rough ping each (to the relay and between players).
- Every "Waiting for NAME..." (who, how long, roughly when).
- Every drop and reconnect: who, how long, whether it worked.
- Any desync notice (tick, digest).
- Whether direct links opened (each client's log: `direct link to slot N open`, `lost`, `not made`).
- Any refusal in the lobby, in the words shown.

**Files to keep**

- Each player's `client.log` and any `desync-reports/` and `path-reports/` folder.
- The relay's start line and the time (it prints nothing more).
- Your notes above.

**Pass:** the game ran 30 or more minutes of game time with no desync notice from any player, and the notes show no unexplained stop. A desync, or a stall you cannot explain, is not a failed trip: send the section 6 package to the main session.
