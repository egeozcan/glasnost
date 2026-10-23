---
id: S-0201
title: Game speed and tick rate
topic: simulation
revision: 23
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-23
measurements: []
requests: [Q-1806]
supersedes: []
---

# S-0201: Game speed and tick rate

## Summary

How many ticks the original runs per second of real time at each game speed, how it paces ticks
against the host clock, where a skirmish's game speed comes from, and when
no tick runs (lost focus, the options menu, a freeze) or the player cannot give orders although
ticks run (the first ticks of a game, a chrono flash). It owns exception 2 of glossary claim G3,
the game-speed dependence of a chrono flash that overlaps a freeze, and that exception's part of the
shared switch `quirks.speed_normalised_delays`; what happens inside one tick is S-0203.
This revision covers skirmishes. Of network games it covers four points only: what the options
menu, lost focus and the options menu's speed control do in a network game, and whether the active
clock stops there (claims 15 to 18, and claim 19 for games played online), with the part of the agreed tick rate that the speed control
reaches (claim 17). The rest of network play and campaigns are out of scope (claim 13). Campaign material retained below records earlier findings for a future campaign spec; it is historical context only, is not a claim of this revision, and is not a requirement or measurement for this revision. Both radar-movie actions do nothing in skirmish. This revision defines no movie timeline for our engine: the earlier cycle-counted movie proposal is deferred, not an additional path from game speed to skirmish game state.

## Terms

- **Tick number.** As in the glossary: the number of ticks a game has completed. It is 0 while the
  first tick of a game runs and grows by 1 at the end of every tick except the final tick
  (claim 1). A victory or defeat decision can precede that final tick by an end delay (S-0414,
  claim 8). Timings of game state counted in ticks are measured against it (S-0203, claim 9).
- **Pacing unit.** 16 milliseconds of host time. The original paces against a clock whose reading
  is *u*(*t*) = ⌊*t* / 16⌋ pacing units at host time *t*, where *t* is the reading of the host's
  millisecond clock, a non-negative integer. The reading advances by 1 on every 16 ms boundary.
  62.5 pacing units make one second.
- **Cycle.** As in the glossary: the unit of real time in which the original runs a game. Cycles
  follow one another without overlap. In a single-player game every cycle makes one drawn frame (claim 6), and a cycle
  that is not frozen takes at most one input event from the player's waiting inputs, in the order
  they were made, and then runs one tick (S-0203). In a network game a cycle runs a tick only once
  every player's orders for that tick have arrived (Terms, *Lockstep*). An input event is a key or mouse button going
  down or coming back up. A frozen cycle (claim 10) takes no input and runs no tick. S-0213,
  claim 1, owns the input-event limit, its confidence and its backing (scenario O9); claim 2 here
  owns pacing, not that limit. The cycle lasts at least its paced period (none at
  index 0, claim 5), measured from its start (claim 2). During a stand-still there are no cycles at
  all. In a network game the paced period is that of claim 17 instead.
- **Stand-still.** A stretch of real time with no cycles, as in the glossary's *cycle* entry: in a
  single-player game, the game window has lost the input focus (claim 9), or the options menu is
  open (claim 8). "The game stands still" means the same. No tick runs and no drawn frame of the
  game is made during it. Where a rule holds for one of the two causes only, the spec names the
  cause. A network game has no stand-still: neither cause stops its cycles (claims 15 and 16).
- **Paced period.** The shortest real time a cycle may take at a given game speed (claim 2).
- **Freeze.** A stretch of real time during which cycles keep making drawn frames but no tick runs
  and the player cannot give orders (claim 10). A cycle during a freeze is a *frozen cycle*. A freeze
  is *timed* when it ends after a set time (claim 10) and *untimed* when something else ends it
  (claim 11).
- **Hold.** An unbroken stretch of frozen cycles, during which the tick number stays at one value.
  Freezes that overlap, or follow one another with no tick between them, make one hold (claim 11,
  Formulas F4).
- **Order block.** A state in which the game accepts no orders from the player and hides the mouse
  pointer, although ticks may run (claims 11 and 14). A freeze also blocks orders.
- **Chrono flash.** The glossary's *chrono flash*, one of its *global effects*. S-0104 owns its
  lighting. As the glossary entry allows, this spec uses the term for the whole effect, which it
  owns apart from the lighting: its steps, its order block and its freeze (claim 11). It is not the
  chronosphere superweapon's jump.
- **Single-player game.** As in the glossary: a skirmish or a campaign mission, as opposed to a
  network game (LAN or online).
- **Network game.** As in the glossary (*multiplayer game, network game*): a game played by several
  machines together. Each player's machine runs the whole game. *Machine* means one player's
  computer with its copy of the game; *host* keeps its meaning of claim 4, the machine's own clock,
  sleep and speed.
- **Lockstep.** The way the machines of a network game stay together: every machine runs the same
  ticks with the same orders, and a machine does not run a tick before the orders of every player
  for that tick have reached it. A machine that runs its cycles more slowly than the others
  therefore holds every machine back to its own rate.
- **Agreed tick rate.** *A*, in ticks per second: the rate that every machine of a network game
  paces its cycles by (claim 17).
- **Timing order.** In a network game, the event that changes the agreed tick rate (claim 17)
  and the order delay.
  What a recorder sees of it: the rate changes on every machine in the same tick; that tick
  follows, by the order delay, a tick whose number is a multiple of 256; and the rate it sets is
  min(cap(*s*), *C*) (claim 17), so after a speed change the rate settles there (in a game on a local network; claim 19 for an online game's first timing order). A timing order is
  not a player's order to their objects. Besides the agreed tick rate, it changes the order delay
  (below). Which machine sends it has no
  observable effect, since the game-speed index it reads is the same on every machine and *C* is
  set by the slowest machine, not by the sender (claim 17),
  and our engine sends no timing orders (claim 17, Our engine).
- **Order delay.** *W*, the number of ticks from the tick in which a player's order is sent to the
  tick in which every machine carries it out, in a network game (claim 17). At any moment one
  value applies to the orders of every player, on every machine. In the original it is not a
  constant: it changes on every machine at once, in the tick in which a timing order is carried
  out, so it depends on the network and can change during a game (`unmeasured`, confidence medium: scenario
  T8 reads it in each run but varies no network condition). In our engine *W* is its own order
  delay for a network game, which the future multiplayer spec owns (claim 13).
- **Active clock.** The clock of elapsed time whose total S-0414 claim 23 displays at the end of a
  game. It reaches no rule of game state.
- **Options menu.** The in-game menu the player opens during a game, with the game-speed control
  among its settings.
- **Terms of S-0011.** *Number slot*, *parameter kind* and *value* of a trigger action (the
  action's integer parameter, read as S-0011's Import mapping, table I2, says; for kind 0 the
  value is an integer that fills the number slot), *one-time* (repeat type 0 of a tag) and
  *elapsed-time trigger* (a trigger whose only event is the elapsed-time event, which fires in tick
  15 × *v* for the event's value *v*, so 20 gives tick 300) are S-0011's terms, used here as
  defined there.
- **Pause action, win action.** Two of the map's trigger actions, whose type codes Import mapping
  gives. The pause action starts a timed freeze (claim 10); the win action makes a house win or lose
  (scenario T1, part C).
- **Radar movie.** A movie that a trigger action gives to the sidebar's radar panel to play. A
  *freezing* movie is one given by the freezing radar-movie action: while it is the earliest-given
  movie not yet ended, the game is under an untimed freeze in the historical campaign finding only (claim 11). Neither action plays a movie or freezes a skirmish.

## Behaviour

1. **Tick number.** At game start the tick number is 0. It grows by exactly 1 at the end of every
   tick that runs except the final tick, and nothing else changes it during a game. How a game
   becomes over decides what it holds afterwards:
   - **Ended by a rule of game state.** A victory or defeat decision does not itself make its
     tick the final tick. Play continues through the end delay (S-0414, claim 8), and the tick
     number advances after each tick before the final one. The final tick runs in full but does
     not add 1, so afterwards the tick number is the number of the last tick that ran (S-0203,
     State and ordering). A resignation counts here: it makes the player's house lose in a later
     tick, followed by the end delay, rather than ending play immediately.
   - **Left from the options menu.** When the player leaves the game through an options-menu choice
     that ends it at once, the game ends between two cycles and no further tick runs. The tick
     number stays at the count of ticks run, one more than the number of the last tick.
   - Which of the menu's choices ends the game at once and which one resigns is not established (Open
     questions). Our engine reproduces both rules.
   - Backing: `measurable` (scenario T1, part A) for the start at 0 and the step of 1; `measurable`
     (scenario T1, parts C and D) for the two ways play becomes over; S-0414 claim 8 owns the
     end delay's length and backing; `unmeasured` for the exact
     menu-choice mapping, since no button-to-result mapping has been established
   - Confidence: high for the start and the step; medium for the two ways play becomes over; low for
     which menu choice is which

2. **Pacing at game speeds 1 to 6.** In a single-player game at game-speed index *s* from 1 to 6,
   a cycle does not end before the pacing clock has advanced by *s* pacing units since the cycle
   began. The cycle's own work (input, the drawn frame and the tick) counts inside that wait, so the
   wait is only what is left of it. When the work alone takes longer, there is no wait at all. The
   next cycle begins as soon as the wait ends. With a host that keeps up, cycles therefore end on
   16 ms boundaries and the mean tick rate is *R*(*s*) = 62.5 / *s* ticks per second (Formulas F1,
   Constants). This is not 60 / *s*: the pacing unit is 16 ms, not 1/60 s.
   The input work processes at most one waiting input event in an unfrozen cycle, as defined in
   Terms, *Cycle*. S-0213 claim 1 owns that limit, with medium confidence and backing
   `measurable` (its scenario O9); this claim owns the pacing rule only.
   - Backing: `measurable` (scenario T1, part B)
   - Confidence: high for the rule and the 16 ms unit; medium for the measured rate on a given
     host, which also depends on how finely the host's millisecond clock and its sleep resolve
     (claim 4)

3. **No catch-up.** Each cycle measures its paced period from its own start. A cycle that took
   longer than the paced period (a slow host, or a long tick) is not made up later: the ticks lost
   to it are never run faster afterwards. The same holds after a stretch without ticks (claims 8 to
   10).
   - Backing: `measurable` (scenario T4) after a stretch without cycles (lost focus); `unmeasured`
     for a single slow cycle: the measurement setup has no way to make one cycle slow without
     changing the game, and the rule is the same one, a paced period measured from each cycle's own
     start
   - Confidence: high

4. **Our engine's pacing (departure).** The original's real tick rate depends on the host: on the
   resolution of its millisecond clock, on how long its sleep really lasts, and on how fast it
   computes and draws a cycle. Our engine replaces this with fixed pacing: at game speed *s* from 1
   to 6, measured from the cycle's start, each cycle lasts exactly 16 × *s* ms, or as long as its
   work takes if that is longer. There is no catch-up, as in claim 3. The paced period of index 0
   is set in claim 5. This is a *departure* with no switch.
   - **What reaches game state.** No rule of game state in our engine reads real time, so neither
     the host nor a slow cycle changes a result of game state. The paced period itself does: it
     sets the number of frozen cycles of a timed freeze (claim 10), and through a chrono flash
     in force that number changes how many ticks run before
     the chrono flash's own freeze (claim 11). So the game speed, and the choice of the paced period
     of index 0 (claim 5), reach game state there, as the game speed does in the original. In the
     original the host reaches game state on that path as well (claims 10 and 11); our engine
     removes only that. Within this spec's skirmish scope, that timed-freeze interaction is the
     only path from real time or from the paced period to game state; the rest is counted in ticks. The one switch
     `quirks.speed_normalised_delays` (glossary claim G3) also removes the game-speed dependence
     of this path (claim 11, Edge cases and quirks).
   - Backing: `unmeasured` (a decision of our engine, not a statement about the original)
   - Confidence: high

5. **Game speed 0.** Index 0 adds no wait of its own: a cycle at index 0 ends as soon as its work is
   done, so the original's tick rate at index 0 is limited only by the host and the display path.
   This agrees with glossary claim G2, part (b′). Our engine paces index 0 at a fixed period of 8 ms
   (125 ticks per second), as a departure with no switch; this period
   also fixes the frozen cycles of a timed freeze at index 0 (claim 10).
   - Backing: `unmeasured` for the absence of a wait, for the reason glossary claim G2 gives for
     part (b′); scenario T1 (part B) shows its consequence *N*₀ ≥ 0.99 × *N*₁
   - Confidence: high

6. **One drawn frame per tick.** In a single-player game every cycle makes exactly one drawn frame.
   In an unfrozen cycle after a completed tick, that drawn frame shows the state at the end of
   the previous tick, including the orders that took effect at its end (S-0203, claim 7).
   The tick of its own cycle does not show in it yet. During a freeze no new tick state appears,
   although the chrono flash can advance. This holds
   at every game speed, so at index *s* ≥ 1 the original draws at most 62.5 / *s* drawn frames per
   second. During a stand-still, no drawn frame of the game is made either (claims 8 and 9). This is
   the rule the glossary's *drawn frame* entry points to for single-player games: rendering rules that say "per drawn frame" and "per tick" coincide there, except during a
   freeze, which makes drawn frames without ticks (claim 10).
   - Backing: `measurable` (scenario T3, part A) for one drawn frame per tick; `measurable`
     (scenario T3, part B) for no drawn frame while focus is lost or the menu is open, and
     `unmeasured` for which tick's state a drawn frame shows, which is internal: its observable
     consequence is the order latency of S-0203, claim 7
   - Confidence: high

7. **Where a skirmish's game speed comes from.** In a skirmish the game speed is the position of
   the skirmish lobby's speed control: its seven positions, from the slowest to the fastest, set
   indices 6, 5, …, 0 (glossary claim G2, part c). When the lobby opens, the control starts at the
   index saved in the settings file's skirmish section. When that key is absent, it starts at the
   rules' lobby default, and when that key is absent too, at the absent-key default 6 (Import
   mapping). At skirmish start the chosen index replaces the general game-speed value held
   for the session (the settings file's own value is not read again), so the general value the
   settings file held before does not decide the speed of a skirmish started from the lobby, and
   the options menu, in the skirmish and after it, shows the lobby's index. When the settings file's general game-speed key is
   absent, the general value is 3 (the middle position of the speed control). In single-player
   scope that value decides no game speed: a skirmish replaces it at its start and a campaign
   mission sets it aside (claim 12). It shows only in the speed control of an options menu that is
   offered outside a mission, after a campaign mission or at the start of a session.
   - Backing: `measurable` (scenario T2, parts A, B and E) for the lobby's starting position and the
     replacement; `unmeasured` for the absent-key general value 3: skirmish start replaces it,
     and no reachable outside-game speed control has been established for T2 part E or the
     historical T2 part D. That value remains a low-confidence import default, not a skirmish speed.
   - Confidence: medium for the lobby's initial value (it rests on the examination of the original; whether the original keeps the
     index in range when the saved value lies outside 0 to 6 is not); low for the absent-key
     value 3, which no game in the scope of this spec uses

8. **Changing speed during a skirmish.** The options menu's speed control sets the index to
   6 − *p* for control position *p* (0 slowest to 6 fastest), so the same seven indices as in the
   lobby. The new index takes effect from the next cycle. With it come the paced period of claim 2,
   the speed-normalised delays that start from then on (glossary claim G3) and, in our engine, the
   number of frozen cycles of a timed freeze that starts from then on (claim 10). In a
   single-player game, while the options menu is open, no cycle runs: no tick, no drawn frame of the
   game. Closing it resumes the cycles without catching up (claim 3). In a network game the menu
   stops no cycle and a speed change travels as an order (claims 15 and 17). The menu does not
   always offer the control: in a network game it shows none once the player's own house is
   defeated, and in our engine neither does a menu opened while the person watches a skirmish on
   after the human player's defeat (claim 17).
   - Backing: `measurable` (scenario T5; scenario T3, part B, for the drawn frames), with
     `unmeasured` for the exact first cycle affected and for delays started after the change:
     T5 measures a sustained rate, not the transition cycle, and does not start a speed-normalised delay
   - Confidence: medium

9. **No ticks while the window lacks focus.** In a single-player game, while the game window does
   not have the input focus, no cycle runs and the game stands still. It resumes, without catching
   up (claim 3), when the window regains focus. Our engine also runs no cycle while its window
   lacks focus in a single-player game, and a stand-still uses up no freeze (claim 10). Window
   focus is not part of game state: no rule of game state in our engine reads real time (claim 4),
   so the stand-still changes no result, and it is a rule of presentation. Claim 16 covers network
   games, where lost focus stops no cycle.
   - Backing: `measurable` (scenario T4) for single-player games
   - Confidence: high

10. **Freeze.** The pause action, a trigger action of the map, with a parameter of *N* seconds,
    starts a timed freeze in a single-player game. During it, cycles go on making drawn frames, but
    no tick runs, the tick number does not change and no order can be given. If orders were not
    already blocked when the freeze began, the mouse pointer is hidden during it, and orders are
    accepted again once it ends; if they were blocked, they stay blocked after it as before. The
    tick in which the action runs completes, so the tick number stands at that tick's number plus 1
    during the freeze.
    - **Length in the original.** The freeze ends at the first cycle that begins once the pacing
      clock has advanced by 60 × *N* pacing units since the action ran, that is 0.96 × *N* seconds
      of real time (Formulas F2), not *N* seconds; that cycle runs a tick again unless another
      freeze is still in force. At a host that keeps up, the first tick after the freeze therefore
      begins 960 × *N* ms after the start of the cycle in which the action ran, and the freeze has
      60 × *N* / *s* − 1 frozen cycles at index *s* from 1 to 6. The number of frozen cycles thus
      depends on the game speed; at index 0, and at a host that does not keep up, it depends on the
      host. The length runs in real time: time during a stand-still for lost focus (claim 9)
      counts towards it. Whether the options menu can be opened during a freeze is open (Open
      questions).
    - **Several actions in one tick.** No trigger action runs during a freeze, so a second pause action can only meet a timed freeze in force when both run in the same tick. It
      does not add a second freeze: it restarts the one in force with its own *N*, so the last such
      action of the tick with *N* ≠ 0 decides the length. Parameters of 0 or below are bad data
      and covered in Edge cases and quirks.
    - **Our engine.** It runs no tick during a freeze, as the original, and counts the length in
      cycles instead of real time: a timed freeze with *N* ≥ 1 has *F* = 960 × *N* / *P* − 1
      frozen cycles after the cycle in which the action ran, where *P* is our engine's paced period
      at the game speed in force when the action ran (claims 4 and 5, Formulas F2). At a host that
      keeps up this is the original's count at every index from 1 to 6, so *F* depends on the game
      speed as the original's count does. It is a departure only in what it counts: real time in
      the original, cycles in our engine, so a stand-still does not use up a freeze and the host
      does not change *F*. The count matters to game state through a chrono flash in force
      (claim 11).
    - Backing: `measurable` (scenario T6, parts A, C and F; part K for orders that were already
      blocked)
    - Confidence: medium

11. **Chrono flash.** The trigger action that starts a chrono flash, with a parameter *D* (a
    positive count), plays it in a single-player game. From the moment the action runs, orders are
    blocked, but ticks go on.
    - **Steps.** The chrono flash advances by one step in every cycle from the action on: in a
      cycle that runs a tick it takes the chrono flash's step of that tick (S-0203, row 4), and in a
      frozen cycle it steps as well. A stand-still (claims 8 and 9) does not advance it.
      It has 2*D* + 45 steps. Its step *D* + 44 starts an untimed freeze from the next cycle, and
      its last step ends that freeze, so the cycle after the last step runs a tick again unless
      another freeze is still in force.
    - **Without another freeze.** If no other freeze falls within the chrono flash, steps 1 to
      *D* + 44 fall in *D* + 44 consecutive ticks and the freeze has *D* + 1 frozen cycles, so at a
      host that keeps up the first tick after the freeze begins (*D* + 2) × *P*(*s*) after the start
      of the cycle of the last tick before it (Formulas F1, F3). An action of a trigger checked at
      the start of the tick (an elapsed-time trigger, S-0203, row 1) takes its first step in its own
      tick, so its own tick is the first of the *D* + 44; an action set off later in a tick, after
      row 4, takes its first step in the next tick.
    - **With a timed freeze.** The chrono flash's freeze adds to a timed one: the game stays frozen
      until both have ended. A timed freeze that falls within the chrono flash's first *D* + 44
      steps uses up one step in each of its frozen cycles, so fewer ticks run before the chrono
      flash's own freeze. If the timed freeze has at least as many frozen cycles as steps remain
      up to and including step *D* + 44 (*F* ≥ *D* + 44 − *k*, with *k* the steps taken by the end
      of the tick in which the timed freeze starts), the chrono flash's freeze starts inside the
      timed one and the two make one hold, with no tick between them (Formulas F4).
    - **Game speed reaches game state here.** The timed freeze's frozen cycles are 60 × *N* / *s* − 1
      in the original at a host that keeps up, and *F* in our engine (claim 10), so the number of
      ticks that run before the chrono flash's own freeze depends on the game speed. That is not a
      speed-normalised delay: it is exception 2 of glossary claim G3. For example, with *D* = 30 and
      both actions in one tick, a timed freeze with *N* = 1 lets 64 ticks run after the actions' own
      tick and before the chrono flash's freeze at index 6, 14 at index 1, and none at index 0 in
      our engine, where the two freezes make one hold (scenario T6, part G). Every rule of game
      state that waits on the chrono flash's freeze, or on the end of its order block, inherits the
      dependence. Switch `quirks.speed_normalised_delays`, default original (Edge cases
      and quirks); with it off the chrono flash takes a step in a frozen cycle only while its own
      freeze is in force, so it takes none in a frozen cycle of a timed freeze before its own
      freeze, and the dependence is gone.
    - **Orders afterwards.** When the chrono flash ends, orders are accepted again unless they were
      already blocked when the action ran (consequences in Edge cases and quirks).
    - **Repeated action.** If the action runs again while the chrono flash is running, it starts
      over from its first step with the new *D*. With *D* of 0 or below the action does nothing,
      not even to a chrono flash already running.
    - **Radar movies in the original.** *Skirmish rule:* in a skirmish both actions do nothing. *Historical campaign finding (out of scope; retained for a future campaign spec only, not a claim or measurement of this revision):* in a campaign mission, the plain radar-movie action and the freezing radar-movie action (Import mapping) each give the sidebar's radar panel a movie to play. The panel plays these movies one at a time, in the order in which their actions ran, so a movie starts only once every movie given before it has ended. While the earliest-given movie that has not yet ended is one given by the freezing action, the game is under an untimed freeze, which ends when that movie ends, and a chrono flash in force takes one step in each of its frozen cycles as well. A movie given by the plain action freezes nothing. A parameter value outside the game's movie list gives no movie. A movie ends when its playback ends, in real time, so in the original the freeze's length depends on the host (compare claim 10, at a host that does not keep up). Which movie a value names and how it is drawn are presentation, not covered by this revision.
    - **Radar movies in our engine.** Both actions do nothing in skirmish, as in the original.
      Movie timing, queueing and any movie-related freeze in our engine are deferred to a future
      campaign spec. They impose no requirement here and are not an exception to glossary G3.
    - **Our engine** counts the chrono flash's steps per cycle as the original does by default
      (`quirks.speed_normalised_delays` on, the original), with cycles paced as in claim 4 and
      timed freezes counted in cycles as in claim 10. With the switch off, a chrono flash steps in
      a frozen cycle only while its own freeze is in force (Edge cases and quirks; F4).
    - Backing: `measurable` (scenario T6, parts B, G, H and I) for the chrono flash, including the
      game-speed dependence (part G) and non-positive parameters (part N); `measurable` (scenario T6, part L) for the radar movies'
      skirmish behaviour (both actions do nothing); historical finding only for the campaign
      radar-movie observations, not measured or measurable as claims of this revision (the earlier
      proposed measurements are retained in T6, parts J and M, as historical context only)
    - Confidence: medium for the chrono flash's step counts and the game-speed dependence; low for
      the radar movies

12. **Campaign missions (historical finding; out of scope).** The following records an earlier finding for a future campaign spec; it is not a behavioural claim or implementation requirement of this skirmish-only revision. During a campaign mission the game speed is index 2, whatever the player has set, and the options menu does not offer the speed control. The player's own setting is put aside when the mission starts and restored when it ends. The speed-normalised delays of glossary claim G3, and the frozen cycles of claim 10, therefore use index 2 in a campaign mission. The original lifts this only through a developer start-up option, which is not one of our engine's inputs, so our engine always runs campaign missions at index 2.
    - Backing: historical finding only; not measured or measurable as a claim of this revision (the earlier proposed measurement is retained in T2, parts C and D, as historical context only)
    - Confidence: high for the recorded finding; it is outside this revision's scope

13. **Network games (mostly out of scope).** Of network games this revision covers only the
    options menu (claim 15), lost focus (claim 16), the options menu's speed control with the cap
    that the game speed puts on the agreed tick rate (claim 17) and the active clock (claim 18).
    A future multiplayer spec owns the rest, including the full negotiation of the agreed tick rate
    (the part that depends on the machines' speed, and when the first timing order comes), how the
    pause action and the chrono flash behave in a network game, the drawn frames made while a
    machine waits for the others' orders, and the draws that network games take for the
    synchronisation checksum (S-0202, claim 12, states the draws that those games take from the
    scenario random stream). Nothing else in this spec holds for network games, except where it
    says so. Claim 18 makes one narrow statement about the pause action and the chrono flash in a
    network game: neither stops the active clock. Claims 15 to 18 are stated for network
    games played on a local network; claim 19 says how far each holds for games played online.
    - Backing: `unmeasured` for the parts left to the future spec (they are outside the
      measurement setup of this revision); claims 15 to 18 carry their own backing
    - Confidence: low for the parts left to the future spec

14. **No orders in the first ticks of a game.** When a single-player game starts, orders are
    blocked during the cycles of ticks 0, 1, 2 and 3. The block is released at the end of tick 3,
    so from the cycle of tick 4 on orders are accepted: the earliest order takes effect at the end
    of tick 4 (S-0203, claim 7) and objects first act on it in tick 5. The drawn frames of those
    cycles are made as usual. If a map's own trigger actions have already blocked orders at that
    moment, this rule adds nothing. A chrono flash (claim 11) that starts during ticks 0 to 3
    cancels the release at the end of tick 3, and because orders were blocked when it started, they
    stay blocked after it ends (Edge cases and quirks). Our engine reproduces this; it depends on
    ticks only, not on real time.
    - **Scrolling is separate from orders.** S-0101 claims 12 to 15 own edge and unmodified
      arrow scrolling. During an order block, edge scrolling is suppressed, but an arrow
      already held before the block continues scrolling in unfrozen cycles. Neither scrolls
      during frozen cycles or a stand-still, and a freeze does not itself clear edge momentum
      or held-arrow state. This is presentation behaviour, not another game-speed path to
      game state. S-0213 claim 23 owns arrow-input acceptance: with
      `quirks.blocked_arrow_events` on (default original), presses and releases made during
      the block are discarded, previously accepted contributions remain unchanged, and
      unblocking does not synthesize a press. Thus a blocked release does not stop an
      accepted contribution, and an arrow first pressed during the block contributes
      nothing even if still physically held afterwards. This concerns events made during
      the block with no older waiting input; it does not cancel input already waiting
      before the block. With the switch off, view-arrow presses and releases remain
      accepted during the block, but frozen cycles still do not scroll (S-0213 claim 23).
    - Backing: `measurable` (scenario T7; scenario T6, part I, for the chrono-flash case).
      Scrolling inherits S-0101 claim 15's `measurable` backing (its scenario M7f), and
      arrow-input acceptance inherits S-0213 claim 23's `measurable` backing (its Scenario O22)
    - Confidence: medium for the count of ticks and the chrono-flash case; high for scrolling
      suppression and retained state, medium for visible resumption (S-0101 claim 15), and
      high for arrow-input acceptance (S-0213 claim 23)

15. **Network games: the options menu (Q-1806, question 1).** In a network game, opening the
    options menu stops no machine's game. Every machine, the opener's included, goes on running
    cycles and ticks at the agreed tick rate (claim 17), and the other players are not told that the
    menu is open. While it is open, the opener's machine takes no input for the game, so that
    player gives no orders, and it draws no new frame of the battlefield. Orders the player gave
    before opening it are carried out as usual, and the player's objects go on acting in every
    tick. Closing the menu brings back input and drawing; no tick was missed, so there is nothing
    to catch up.
    - **When the menu opens.** In a skirmish and in a network game alike, the menu does not open
      from the end of the game on, and stays refused through the end delay. The house state this
      rule turns on is S-0414's: the end of the game comes from its end test (S-0414 claim 6),
      from that tick the player's own house has its result, a win that is pending or a loss
      (S-0414 claims 7 and 9), and the end delay is S-0414 claim 8's. After the player has
      resigned through the menu, it does not open again until the player's house is defeated; what
      a resignation does to the house, and when the house is defeated, are S-0414's (claims 11
      and 5). A defeated player who goes on watching a network game can open it. The refusal of
      the menu in each of these states is this claim's rule; S-0414 does not state it. S-0414
      covers skirmishes only; in a network game these states are those that the future
      multiplayer spec is to define (claim 13).
    - **A surrender during the end delay.** In a skirmish no tick runs while the menu is open
      (claim 8), so a skirmish menu is never open when the game ends, and a skirmish player cannot
      surrender during the end delay. In a network game ticks go on while the menu is open, so a
      menu opened before the end can still be open during the end delay; choosing surrender in it
      then still sends the surrender order, and S-0414 claim 11 says what that order does (nothing,
      for a house whose result is pending). So in the original the surrender of S-0414 claim 11's
      pending-result case can be given only in a network game (Changes for other specs).
    - **Our engine.** Reproduces this. Q-1806's interim rule (the menu never stops the ticks)
      agrees with it. A surrender order given by other means (S-0213) is not limited by the menu.
    - Backing: `measurable` (scenario T8, parts A, F and H, part A also for the orders given before
      the menu opened, the objects acting and the other players not being told; part D for a
      defeated player opening the menu; scenario T9 for the skirmish end delay); `unmeasured` that
      the opener's machine takes no input while the menu is open, since no part offers input to a
      machine whose menu is open and the menu itself takes the clicks; `unmeasured`
      that the menu is refused from the end of a network game on, since no network part tries to
      open it then; `unmeasured` that a surrender chosen in the end delay is sent rather than dropped: part H
      sees only that it has no effect; `unmeasured` for the menu after resigning in a skirmish:
      in a skirmish the defeat that follows a
      resignation is also the end of the game (S-0414 claims 6 and 11), from which the menu is
      refused anyway, so no reopening can be seen there; only part F, a network run, shows it
    - Confidence: medium, until part A runs, for everything part A tests: that ticks go on, for the
      opener and the others, that the opener's machine draws no new frame of the battlefield while
      the menu is open, the earlier orders, the objects and the notice; medium for the input
      (unmeasured); medium for when the menu
      opens, and for the surrender from a menu open across the end of a network game; medium for
      the menu after resigning in a skirmish

16. **Network games: lost focus (Q-1806, question 2).** In a network game a machine whose window
    lacks the input focus goes on running cycles and ticks, so the game goes on for its player and
    for the others; there is no stand-still. While the window lacks focus, that machine takes no
    input for the game and makes no drawn frame. Its player's earlier orders are carried out as
    usual.
    - **The original slows every machine.** Each cycle of the unfocused machine takes at least
      10 ms of real time beyond its paced period (claim 17), and more on a host whose sleep is
      coarser. Through the lockstep that machine holds back every machine, so while any player's
      window lacks focus the whole game runs at about 1000 / (⌊1000 / *A*⌋ + 10) ticks per second
      or less, against 1000 / ⌊1000 / *A*⌋ with every window focused (Formulas F5). At *A* = 30
      that is about 23 instead of 30.3 ticks per second. The other players see their own game run
      slower, and they keep their input and drawing.
    - **Our engine (departure).** Keeps running its cycles at its own network pacing (claim 17, Our engine) while its window
      lacks focus in a network game, and adds no extra wait, so one player's lost focus slows no
      one. How the client keeps its pace when the host slows a window in the background is a matter
      of implementation; the requirement is that the ticks go on. The slowdown is real-time
      behaviour: it changes no rule of game state, so under glossary *quirk switch* it takes no
      switch; the project lead is asked to confirm. Q-1806's interim rule agrees.
    - Backing: `measurable` (scenario T8, part B); `unmeasured` that the unfocused machine takes
      no input for the game: a window without the focus gets no ordinary input from its player,
      so part B offers it none to refuse; `unmeasured` that its player's earlier orders are carried
      out as usual and that the other players keep their input and drawing: part B gives no order
      and records drawn frames on machine 2 only
    - Confidence: medium, until part B runs, that ticks go on and that the unfocused machine draws
      nothing; medium that
      it takes no input, that its player's earlier orders are carried out and that the other
      players keep their input and drawing (unmeasured); medium for the slowdown of every machine; low for its size on a given host

17. **Network games: the speed control and the agreed tick rate (Q-1806, question 3).**
    - **The control is offered.** In a network game the options menu offers the speed control, at
      the positions of claim 8, as long as the player's own house has not been defeated. Once it
      has, the menu (claim 15) shows no speed control in a network game. In a skirmish the
      original's menu does not open after the human player's house is defeated: that defeat ends
      the game (S-0414 claim 6), and the menu does not open from the end of the game on (claim 15).
      Where our engine lets the person go on watching a skirmish after that defeat, a menu opened
      then likewise shows no speed control; this is our engine's rule, not a claim about the
      original.
    - **A change travels as an order.** Setting the control to a position whose index differs
      from the index in force does not change the index on the player's own machine at once.
      The change is sent, when the menu is closed (State and ordering), to every machine as an
      order of that player, with the same delay as that
      player's other orders, and is carried out in the same tick on every machine: tick *t* + *W*,
      where *t* is the tick in which the menu closes and *W* is the order delay in force (Terms).
      In our engine *W* is its own order delay for a network game, the one every other order of
      the game has, so no further value is needed here; in the original *W* depends on the network
      and has no fixed figure to reproduce (Terms), and scenario T8 reads it from each run. In that tick every machine sets the game-speed index to the new value,
      whoever's machine sent it, and every machine but the sender's shows a message naming the
      house whose player changed it. Every player whose menu offers the control can change the
      speed, and the last change carried out holds. Setting the index already in force sends nothing
      (these two points: `unmeasured`, below).
    - **Game state follows at once.** Speed-normalised delays started after the speed order is carried
      out use the new index (glossary claim G3, as in a
      skirmish). This is game state, and it stays in step on every machine because every machine
      changes the index in the same tick.
    - **The tick rate follows later.** In a network game the paced period is not that of claim 2.
      Each cycle lasts at least ⌊1000 / *A*⌋ ms, measured in whole milliseconds from the cycle's
      start, with no catch-up, where *A* is the agreed tick rate (Formulas F5). *A* is 30 when a
      game on a local network starts, whatever its speed (`unmeasured` at starting indices other
      than 3, below). Only a timing order changes it, and it does so on
      every machine in the tick in which the order is carried out. In a game on a local network a
      timing order is sent only in a tick whose number is a multiple of 256, and is carried out one
      order delay later; not every such tick sends one (which ones is left to the future multiplayer
      spec, claim 13). The order sets *A* = min(cap(*s*), *C*). Here *s* is the game-speed index in force in the
      tick in which the order is sent, the same on every machine, and cap(*s*) is 60, 45, 30, 20, 15, 12 and 10 ticks per
      second for indices 0 to 6 (Constants). *C* is a rate set by the slowest
      machine, which depends on the hosts and is left to the future multiplayer spec (claim 13).
      So a speed change reaches the tick rate only when a timing order sent after it is carried
      out: at the earliest from the first tick after the change whose number is a multiple of 256,
      plus the order delay, and never above *C*. These rates
      differ from the single-player rates of F1: index 6 gives 10 ticks per second in a network
      game, against 10.416… in a skirmish.
    - **Room speed.** The game speed in force when a network game starts is the one set for the
      game before the start, the same on every machine.
    - **Our engine.** Q-1806's interim rule (the control does nothing, every machine keeps the
      speed set before the start) diverges from the original in game state, not only in pacing: in
      the original the index changes on every machine, and with it the speed-normalised delays.
      This revision proposes that our engine sends a speed change as an order and carries it out as
      above. Its pacing is a departure with no switch, in the manner of claim 4: each cycle of a
      network game lasts ⌊1000 / cap(*s*)⌋ ms for the index *s* in force, or as long as its work
      and the lockstep's wait take if that is longer, from the start of the game at the index set
      before the start, and from the cycle after a speed order is carried out at the new index.
      At a host that keeps up this is the original's settled rate at every index (F5). The start
      at 30, the timing order's 256-tick schedule and *C* are host behaviour, not reproduced
      (Questions for the project lead).
    - Backing: `measurable` (scenario T8: part C for the order, the message and the cap of index 6,
      and for the change landing in the same tick on both machines, later than the close (part C
      reads *W* from that change, so it does not test *t* + *W* against the delay of other orders); part D for the control hidden after defeat in a network game; part C's further runs for the caps of indices 0, 1, 2, 4 and 5; part G for the start
      at 30 at index 3); the use of the new index by delays that start from the tick of the change is
      `unmeasured`: no part of T8 times such a delay; that the last of several changes holds, and
      that setting the index already in force sends nothing, are `unmeasured`: no part of T8
      makes two changes and judges which one holds, and a change that sends nothing leaves nothing to observe on the other machine
      beyond the absence of the message, which no part checks; that the change is sent when the menu closes, and not when the control is set, is `unmeasured`: part C sets the control and closes the menu together and reads *W* from that close; the room speed and the start at 30 are `measurable` at
      index 3 only (part G), and at other starting indices `unmeasured`: T8 creates every game
      at index 3; that the speed order has the same delay as the player's other orders is
      `unmeasured`, since T8 reads *W* from the speed order itself and times no other order; the
      hidden control in a skirmish after the human's defeat is our engine's rule (`unmeasured`:
      the original has no such state to observe; design); the
      original's order delay as a function of the network is `unmeasured` (Terms),
      and the schedule's choice of which multiples of 256 carry a timing order, and *C*, are
      `unmeasured` here (left to the future multiplayer spec, claim 13)
    - Confidence: medium, until part C runs, that the control is offered and that a change is
      carried out as an order on every machine; medium for the hidden control in a skirmish after defeat (our engine's rule); medium for the control hidden after defeat in a network game, until part D runs; medium for the delays' use of the new index, the cap table, the start at 30
      and the interval of 256 ticks between the ticks that can carry a timing order, until T8 runs
      (parts C and G see the rate change, not the send: a change seen a short time after a
      multiple of 256 places the send at that multiple only to within the order delay in force
      when the timing order is sent, which may differ from the run's *W*, because a timing order
      carried out between the reading of *W* and that send changes the delay (Terms, *Order
      delay*); so the parts show that changes come only just after multiples of 256, not the send
      tick exactly); medium for
      the speed order sharing the delay of the player's other orders, for the send at the menu's close, for the last change holding, for an unchanged index sending nothing, and for the room speed at
      indices other than 3; low for which
      multiples of 256 carry a timing order, and for *C*

18. **Network games: the active clock (Q-1806, question 4).** In a network game the active clock
    does not stop while a player's options menu is open, while a window lacks the focus, or for the
    pause action or a chrono flash; only leaving play for the result screen stops it, as S-0414
    claim 23 describes. Its total therefore includes every stretch of claims 15 and 16.
    - **The clock in a skirmish.** In a skirmish the original stops the active clock while the
      options menu is open and during every hold (a timed freeze, a chrono flash's freeze), and
      does not stop it during a stand-still for lost focus. That differs from S-0414 claim 23 as
      approved, which pauses it for lost focus and counts frozen cycles (Changes for other specs).
      S-0414 owns the clock; this claim records the finding for that spec's next revision.
    - Backing: `measurable` (scenario T8, part E, for the options menu and lost focus in a network
      game; for the skirmish, `unmeasured` until S-0414 scenario V19 is revised as Changes for other specs
      says, since no scenario here sets up a hold or a lost-focus stretch in a skirmish).
      That the pause action and a chrono flash do not stop the clock in a network game is
      `unmeasured`: T8 uses neither, and their other effects in a network game are left to the
      future multiplayer spec (claim 13), so no scenario here sets them up
    - Confidence: medium for the options menu and lost focus in a network game, until T8 part E
      runs; medium for the pause action and the chrono flash in a network game, which rest on the
      examination of the original alone; medium for the skirmish clock, until S-0414 scenario V19
      runs as revised

19. **Network games played online (Q-1806).** Claims 15 to 18 are stated for games on a local
    network. In the original a network game played online follows the same rules, with the
    differences below.
    - **The same as on a local network.** The options menu (claim 15), lost focus (claim 16), the
      speed control and its order (claim 17: offered until the player's house is defeated, carried
      out in one tick on every machine with the message, the speed-normalised delays following at
      once) and the active clock (claim 18) hold for an online game as stated: the original makes
      no difference between the two kinds of network game for them.
    - **The agreed tick rate at the start.** In an online game a timing order is sent at the start
      of the game. It sets *A* to cap(*s*) for the game
      speed set before the start, without the limit *C*, and is carried out one order delay later.
      So an online game runs at 30 ticks per second only until that order is carried out, not, as
      on a local network (scenario T8, part G), until a multiple of 256, and then at its starting
      index's cap at a host that keeps up. Later timing orders follow claim 17's rule.
    - **Extra waits.** An online game can also lengthen a machine's cycles while another machine
      has fallen behind. So the slowdown of claim 16 can be larger online.
    - **Our engine.** Makes no difference between a game on a local network and an online game:
      the rules of claims 15 to 18, including claim 17's pacing (Our engine), apply to both. The
      original's start rule and extra waits online are host and network behaviour, not reproduced,
      like claim 17's start at 30 (design).
    - Backing: `unmeasured` for every part: the original's own online service can no longer be
      reached, so no scenario can set up an online game with the stock program; our engine's rule
      is design
    - Confidence: medium that claims 15, 16 and 18, and claim 17's speed control and its order,
      hold online as stated; low for the start rule and the extra waits online

### Evidence boundaries

These qualifications apply to the claims above and their restatements in Formulas, Constants,
State and ordering, Edge cases and quirks, and Import mapping.

- Claims 2 and 10: the exact within-cycle clock boundary, and a freeze's count on an arbitrarily
  slow host, are `unmeasured` (confidence: high for pacing, medium for the freeze). T1 and T6
  test sustained keep-up rates and selected holds, not each clock phase or host workload.
- Claim 3: no catch-up after the menu or a freeze is `unmeasured` (confidence: high). T4 isolates
  lost focus only; T5 and T6 do not bound the rate immediately after resumption.
- Claim 11: stepping after a late-in-tick action is `measurable` by S-0203 scenario O6 part B,
  with that spec's confidence. Stepping across a stand-still is `unmeasured` (confidence: medium):
  T4 and T6 part F do not include a chrono flash across focus loss.
- Claim 14's interaction with a separate map-created order block, and claim 11's eventual release
  by a map action, are `unmeasured` (confidence: medium). T6 tests repeated flashes and the
  initial block, not a separately created and lifted map block.
- The non-positive pause cases are `measurable` by T6 parts D and E for cancelling a timed
  freeze and for a standalone permanent freeze (confidence: medium). The additional cases with
  an untimed freeze, with no earlier freeze for a negative parameter, or with a timed freeze
  followed by a zero parameter are `unmeasured` (confidence: medium): those combinations have
  no separate scenario here. Their stated rules are not confirmed by D or E.
- Both switches' off states, importer warnings and our engine's cycle-counted departures are
  `unmeasured` design rules (confidence: high): the original cannot exercise our alternatives.
  F4's original equality boundary and a pause begun in the tick of the flash's own freeze are
  `unmeasured` (confidence: medium): G1–G3 do not isolate those boundary cases.
- Claims 15 to 18 were established for network games played on a local network. Games played
  online are claim 19's, `unmeasured` throughout (no scenario here can play online).
- Import mapping's nonzero parameter kinds and the empty number-slot default for these actions
  are `unmeasured` (confidence: medium): T6 uses kind 0 with explicit values and supplies no
  cross-kind fixtures. Parameter-kind parsing otherwise belongs to S-0011.

## Formulas

**F1 Mean tick rate** at game-speed index *s* from 1 to 6, for a host that finishes every cycle's
work in less than 16 × *s* ms:

> *P*(*s*) = 16 × *s* ms, *R*(*s*) = 1000 / *P*(*s*) = 62.5 / *s* ticks per second

Exact rational values; no rounding applies to the rule itself. The original's wait ends at the first
host millisecond *t* with ⌊*t* / 16⌋ − ⌊*t*₀ / 16⌋ ≥ *s*, where *t*₀ is the host millisecond at which
the cycle began. Number domain: *t* and *t*₀ are non-negative integers in milliseconds, and ⌊ ⌋
rounds down. No floating point is involved in the pacing and freeze rules, so the round-toward-zero
mode of S-0205 (claim 5) does not apply.
A single cycle can therefore last anywhere from just over 16 × (*s* − 1) ms to 16 × *s* ms, but in
steady state every cycle begins just after a 16 ms boundary and lasts 16 × *s* ms. Our engine uses
*P*(*s*) exactly (claim 4), and *P*(0) = 8 ms (claim 5).

| *s* | *P*(*s*) (ms) | *R*(*s*) (ticks per second) | Ticks per 60 s | Real time of 900 ticks (s) |
|---|---|---|---|---|
| 1 | 16 | 62.5 | 3750 | 14.4 |
| 2 | 32 | 31.25 | 1875 | 28.8 |
| 3 | 48 | 20.833… | 1250 | 43.2 |
| 4 | 64 | 15.625 | 937.5 | 57.6 |
| 5 | 80 | 12.5 | 750 | 72.0 |
| 6 | 96 | 10.416… | 625 | 86.4 |

So a *game minute* of 900 ticks (glossary) lasts 57.6 s of real time at index 4, and at no index
from 1 to 6 does it last exactly one minute.

**F2 Freeze length** for a pause action with parameter *N* (seconds, integer,
*N* ≥ 1):

> *L* = 60 × *N* pacing units = 960 × *N* ms (the original)
>
> *F* = 960 × *N* / *P* − 1 frozen cycles (our engine; *P* in ms, the paced period at the game
> speed in force when the action ran)

Integer arithmetic, exact: 960 × *N* / *P* is an integer for every paced period of our engine
(8 ms and 16 × *s* ms for *s* from 1 to 6), so no rounding applies. In the original the freeze ends
on the first cycle at which the pacing clock has advanced by at least *L* units since the action
ran, so its real length is *L* rounded up to the next cycle boundary; at a host that keeps up it
has 60 × *N* / *s* − 1 frozen cycles, which equals *F* for *s* from 1 to 6. At index 0 our engine
gives *F* = 120 × *N* − 1.

**F3 Chrono flash** with parameter *D* (integer, *D* ≥ 1), when no other freeze falls within it:

> ticks from the first step to the chrono flash's freeze = *D* + 44, frozen cycles = *D* + 1,
> steps in all = 2*D* + 45

Integer counts, exact.

**F4 Chrono flash with a timed freeze.** A timed freeze with *F* frozen cycles starts in tick *n*,
in which a chrono flash with parameter *D* has taken *k* steps by the end of the tick
(0 ≤ *k* ≤ *D* + 44). *k* is 0 when the chrono flash's action is set off in tick *n* after the
chrono flash's step (S-0203, row 4), so its first step falls in the next cycle. After
tick *n*, the tick number goes through the stretches of the matching row in order, from left to
right:

| `quirks.speed_normalised_delays` | Case | First hold | Ticks that run between the holds | Second hold |
|---|---|---|---|---|
| on (original) | *k* + *F* < *D* + 44 | at *n* + 1, *F* frozen cycles | *D* + 44 − *k* − *F* | at *n* + 1 + *D* + 44 − *k* − *F*, *D* + 1 frozen cycles |
| on (original) | *k* + *F* ≥ *D* + 44 | one hold at *n* + 1, max(*F*, 2*D* + 45 − *k*) frozen cycles | — | — |
| off | *k* < *D* + 44 | at *n* + 1, *F* frozen cycles | *D* + 44 − *k* | at *n* + 1 + *D* + 44 − *k*, *D* + 1 frozen cycles |
| off | *k* = *D* + 44 | one hold at *n* + 1, max(*F*, *D* + 1) frozen cycles | — | — |

Integer counts, exact. *F* is 60 × *N* / *s* − 1 for the original at a host that keeps up, and F2's
*F* for our engine. Example (scenario T6, part G): *n* = 300, *k* = 1, *D* = 30, *N* = 1 gives a
second hold at 365 at index 6 (*F* = 9) and at 315 at index 1 (*F* = 59); with *N* = 2 at index 1
(*F* = 119) there is one hold of 119 frozen cycles at 301.

Movie freezes and movie lengths are outside this table and this revision's implementation scope.
Both radar-movie actions do nothing in skirmish (claim 11). The movie-length proposal of earlier
revisions is withdrawn pending a campaign spec, including its fallback length and warning.

**F5 Network pacing** (claims 16 and 17), for a network game played on a local network (claim 19
for online games):

> *P*<sub>net</sub> = ⌊1000 / *A*⌋ ms, *R*<sub>net</sub> = 1000 / *P*<sub>net</sub> ticks per second
>
> *A* = min(cap(*s*), *C*), set only when a timing order is carried out, with *s* the index in force when it was sent (claim 17); cap(0) = 60, cap(1) = 45, cap(*s*) = 60 / *s* for *s* from 2 to 6
>
> unfocused machine: each cycle ≥ *P*<sub>net</sub> + 10 ms

Integer arithmetic: ⌊ ⌋ rounds down, and 60 / *s* is exact for *s* from 2 to 6 (30, 20, 15, 12,
10). *A* is 30 until the first timing order is carried out. *C*, the rate that the slowest
machine reports it can sustain, is left to the future multiplayer spec (claim 13); the table
below assumes hosts fast enough that *C* is not the smaller term. The extra 10 ms of an
unfocused machine is a lower bound: the host's sleep can make it longer. No floating point is
involved.

| *s* | cap(*s*) | *P*<sub>net</sub> at *A* = cap(*s*) (ms) | *R*<sub>net</sub> (ticks per second) | With one unfocused machine, at most (ticks per second) |
|---|---|---|---|---|
| 0 | 60 | 16 | 62.5 | 38.46… |
| 1 | 45 | 22 | 45.45… | 31.25 |
| 2 | 30 | 33 | 30.30… | 23.25… |
| 3 | 20 | 50 | 20 | 16.66… |
| 4 | 15 | 66 | 15.15… | 13.15… |
| 5 | 12 | 83 | 12.04… | 10.75… |
| 6 | 10 | 100 | 10 | 9.09… |

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Pacing unit | 16 | ms | Claim 2; the host's millisecond clock read in whole 16 ms units |
| Pacing units per second | 62.5 | pacing units per s | 1000 / 16 |
| Paced period | 16 × *s* | ms | Index *s* from 1 to 6 (claim 2, F1) |
| Paced period at index 0 (original) | none (no wait) | ms | No wait (claim 5) |
| Paced period at index 0 (our engine) | 8 | ms | Departure, claim 5 |
| Campaign game speed (historical context only) | 2 | game-speed index | Claim 12, out of scope |
| Speed-control positions | 7 | positions | Index = 6 − position (claims 7 and 8) |
| Freeze length per second of parameter | 60 | pacing units | 960 ms (claim 10, F2) |
| Tick number at game start | 0 | ticks | Claim 1 |
| Chrono flash, steps before its freeze | *D* + 44 | steps (one per cycle) | Claim 11, F3; *D* is the action's parameter |
| Chrono flash, frozen part | *D* + 1 | cycles | Claim 11, F3 |
| Ticks without orders at game start | 4 | ticks | Ticks 0 to 3 (claim 14) |
| Absent-key default, settings file general game speed | 3 | game-speed index | Import mapping; unmeasured, claim 7 |
| Absent-key default, rules lobby game speed | 6 | game-speed index | Import mapping |
| Agreed tick rate at network game start (local network) | 30 | ticks per second | Claim 17, F5 |
| Cap of the agreed tick rate, indices 0 to 6 | 60, 45, 30, 20, 15, 12, 10 | ticks per second | Claim 17, F5 |
| Interval of timing orders | 256 | ticks | Sent only in a tick whose number is a multiple of 256, not necessarily every one (claim 17); medium confidence for the 256 (T8 parts C and G would show the rate changing only just after a multiple of 256, plus the order delay; they see the change, not the send, and the delay in force at the send may differ from the run's *W*, claim 17's confidence), low for which multiples |
| Extra wait per cycle of an unfocused machine in a network game | at least 10 | ms | Claim 16, F5 |

## State and ordering

**Whether a cycle runs a tick** (skirmish; campaign findings above impose no requirements; network
games in the second table):

| State | Event | Result |
|---|---|---|
| Running | Cycle begins | One drawn frame and one tick; the cycle lasts at least its paced period (none at index 0, claim 5; claim 2) |
| Running | Window loses focus | Stand-still (focus lost): no cycles (claim 9) |
| Running | Options menu opened | Stand-still (menu open): no cycles (claim 8); in our engine, after the human player's defeat, the menu shows no speed control (claim 17) |
| Stand-still (menu open) | Options menu closed | Running from the next cycle, at the index then set, with no catch-up (claims 3 and 8) |
| Running | Pause action runs with *N* ≥ 1 | Frozen from the next cycle (claim 10) |
| Running | Chrono flash takes its step *D* + 44 | Frozen from the next cycle, untimed (claim 11) |
| Frozen | Cycle begins | One drawn frame, no tick, no input; the tick number stays; a chrono flash in force steps as claim 11 specifies, subject to the shared switch |
| Frozen | Window loses focus | Stand-still (focus lost) with the freeze in force. Original: a timed freeze's real time keeps running and can end meanwhile; a chrono flash takes no step. Our engine: the frozen cycles left do not change (claim 10) |
| Stand-still (focus lost) | Window regains focus | Frozen if a freeze is still in force, otherwise Running; no catch-up |
| Frozen | Every freeze has ended | Running from the next cycle |
| Game starts | Cycles of ticks 0 to 3 | Running, but orders blocked (claim 14) |

**One machine of a network game, in the original** (claims 15 to 17; the pause action and the chrono flash are out
of scope, claim 13):

| State | Event | Result |
|---|---|---|
| Running | Cycle begins | Takes input, runs one tick once every player's orders for it have arrived, lasts at least ⌊1000 / *A*⌋ ms (F5) |
| Running | Options menu opened | Menu open: cycles and ticks go on; this machine takes no input and draws no new frame of the battlefield; nothing is sent (claim 15); no speed control once this machine's house is defeated (claims 8 and 17) |
| Menu open | Options menu closed, speed unchanged | Running |
| Menu open | Options menu closed with a new speed | Running; a speed order is sent, and every machine sets the new index in the tick in which it is carried out (claim 17) |
| Running or menu open | Window loses focus | Unfocused: cycles and ticks go on; no input, no drawn frame; each cycle at least 10 ms longer in the original, holding every machine back (claim 16) |
| Unfocused | Window regains focus | Running, or menu open if it was open; no catch-up |
| Any | Timing order carried out | *A* changes on every machine from the next cycle (claim 17); the order delay *W* changes on every machine in that tick (Terms) |

**Ordering consequences.** In consecutive unfrozen cycles, the drawn frame of cycle *k* shows
what the tick of cycle *k* − 1 left, including its orders, before tick *k* runs. During frozen
cycles no new tick state appears, although the chrono flash can advance. In a single-player game, the game
speed set during cycle *k* governs the paced period from cycle *k* + 1 on (claim 17 for a network game). A
trigger action that starts a freeze does not stop the rest of its own tick, so the tick number
during a freeze is one more than the number of the tick in which the action ran.

## Edge cases and quirks

- **The 16 ms unit.** At a host that keeps up, the original's paced period at index *s* from 1 to 6
  is 16 × *s* ms, a whole number of 16 ms pacing units, not a multiple of 1/60 s. Our engine
  reproduces the 16 ms unit exactly (claim 4). No switch: how ticks are spread over real time is
  real-time behaviour, which under glossary *quirk switch* takes no switch. The pacing reaches game
  state only through a chrono flash's steps during a timed freeze, and that path
  is covered by the shared switch below.
- **Freezes are 4 % short.** A freeze of *N* "seconds" lasts 0.96 × *N* s (claim 10). Our engine
  keeps the original's length at a host that keeps up, as the count *F* of frozen cycles. The real
  length itself takes no switch: it is real-time behaviour. A count based on whole seconds is not
  offered either: whatever a second is taken to be, the count still depends on the game speed, so
  it would change game state without removing the dependence. The switch
  `quirks.speed_normalised_delays` removes the dependence itself (below).
- **Host-dependent rate.** Departure of claim 4, and the fixed index-0 period of claim 5.
- **A freeze runs on during a stand-still (departure).** In the original a timed freeze is measured
  in real time, so a stand-still for lost focus uses it up, and the number of its frozen cycles
  depends on the host. Our engine counts frozen cycles (claim 10, F2), fixed when the action
  runs. No switch: real-time behaviour. Campaign movie timing is deferred.
- **The game speed decides a chrono flash's ticks during a timed freeze.** In the original a chrono
  flash takes one step in every frozen cycle of a timed freeze, so the
  game speed changes how many ticks run before its own freeze, and whether there are two holds or
  one (claim 11, F4); in our engine through *F* of claim 10. This is exception 2 of glossary claim G3. Switch
  `quirks.speed_normalised_delays`, the single switch through which game speed reaches game state
  in our engine, default on (the original behaviour); this spec has no switch of its own for the
  path. With it off, a chrono flash takes a step in a frozen cycle only while its own freeze is in
  force:
  - A timed freeze with no chrono flash in force is unchanged.
  - When a timed freeze starts before the chrono flash's step *D* + 44, the chrono flash waits
    through it and still takes *D* + 44 − *k* further ticks before its own freeze, so there are
    always two holds and the count of ticks between them is the same at every game speed (F4).
  - When the timed freeze starts in the tick of step *D* + 44, the two freezes make one hold, as
    with the switch on.
- **One unfocused player slows a network game (departure).** In the original, a machine whose
  window lacks the focus lengthens each of its cycles, and the lockstep passes the slowdown on to
  every machine (claim 16, F5). Our engine keeps the agreed pace. No switch: it is real-time
  behaviour and reaches no rule of game state.
- **A speed change does not show on the changer's machine at once.** In a network game the
  changer's own index changes only when the speed order is carried out (claim 17), like everyone
  else's. Reproduced; it is what keeps the machines in step.
- **Pause parameter below 0.** In the original such an action ends the timed freeze in force, if
  any, at the next cycle: no frozen cycle of that freeze follows, and that cycle runs a tick unless
  an untimed freeze is also in force (for example a chrono flash's freeze starting from that cycle),
  which keeps the game frozen until it ends. With no freeze in force, or with only an untimed one,
  the action has no lasting effect: it adds no frozen cycle and blocks orders in no cycle. Our
  engine reproduces this. The importer warns about such an action (glossary *importer warning*), as
  a likely data error. No switch: it is reproduced bad-data behaviour, which the importer reports,
  and a mod gains nothing from it that *N* = 0 or no action does not give.
- **Pause parameter 0 (departure).** The original adds an untimed freeze that nothing ends, so the
  game stays frozen for good. That holds also when a timed freeze is in force: the timed one ends on
  time, and the game stays frozen. Our engine treats *N* = 0 as no action at all: it starts no
  freeze, leaves a freeze in force (timed or untimed) to end as it would have, and changes neither
  the order block nor the mouse pointer. The importer warns about such an action. This replaces a
  game that never resumes.
- **Chrono flash parameter 0 or below.** The action does nothing at all (claim 11), in the original
  and in our engine. No departure and no switch, because there is no behaviour to switch between.
  The importer warns about such an action, as a
  likely data error.
- **Chrono flash leaves orders blocked.** The chrono flash restores the order block that was in
  force when its action ran (claim 11). Two consequences follow in the original: if the action runs
  again while the chrono flash is running, the block of the first run counts as "already blocked",
  so orders stay blocked after the second run ends; and if the chrono flash starts during ticks 0 to
  3, the game-start block counts as "already blocked" and its release at the end of tick 3 is
  cancelled (claim 14), so orders stay blocked after the chrono flash ends. In both cases they stay
  blocked until a trigger action of the map lifts the block. This is a quirk of its own, not tied
  to the game speed: a mod that wants orders back after a chrono flash, in particular one that
  starts at game start, would turn it off. Switch `quirks.chrono_flash_keeps_order_block`, default
  on (the original behaviour). With it off, when
  the chrono flash ends orders are accepted again unless a block set by the map's own trigger
  actions is still in force; a block from an earlier run of the chrono flash or from the start of
  the game does not outlast it.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Settings file, section `[Skirmish]`, key `GameSpeed` | starting position of the skirmish lobby's speed control, as index *s* (claim 7) | integer; 0 to 6 covered | the rules' lobby default below |
| Rules, section `[MultiplayerDialogSettings]`, key `GameSpeed` | the lobby default index (claim 7) | integer; 0 to 6 covered | 6 (engine constant) |
| Settings file, section `[Options]`, key `GameSpeed` | the general game-speed value of the session; replaced at skirmish start by the lobby's index (claim 7), set aside during a campaign mission (claim 12) | integer; 0 to 6 covered | 3 (engine constant) |
| Map, trigger action of type 110 (the pause action) | starts a timed freeze; *N* in seconds is the action's number slot (claim 10) | integer, from the number slot as S-0011 (Import mapping, table I2) fills it for the action's parameter kind (for kind 0, the integer value); *N* < 0 reproduced as in the original, *N* = 0 gives no action (departure); both give an importer warning (Edge cases and quirks) | a parameter kind that fills no number slot leaves it at 0 (engine constant), so the action gives no action and an importer warning |
| Map, trigger action of type 127 (chrono flash) | starts the chrono flash of claim 11; *D* is the action's number slot | integer, from the number slot as for type 110; *D* ≤ 0 does nothing and gives an importer warning (Edge cases and quirks) | 0, as for type 110, so the action does nothing |
| Map, trigger actions of types 100 (the plain radar-movie action) and 117 (the freezing radar-movie action) | neither action has any effect in skirmish (claim 11); campaign semantics are historical context only | integer number slot, ignored in skirmish | no effect |
| Map, trigger action of type 1 (the win action) | used by scenario T1, part C, only, which states the assumption it relies on; its parameter is the action's number slot | integer, from the number slot as for type 110 | 0, as for type 110 |
| Rules, section `[General]`, key `GameSpeedBias` | not a game-speed input: a movement multiplier that the movement specs own | decimal | — |

No key of the user's data feeds claims 15 to 18. A network game's starting speed is the one set in
its lobby before the start (claim 17).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup*, with a *recorder* that reads the
tick number and, where stated, counts drawn frames as the game hands them to the display path,
reads a vehicle's position, and captures the screen. The game window keeps the input focus
throughout, except where a scenario moves it. The record names the host machine and the display
path, and names any display compatibility layer generically. Input is scripted (mouse and keyboard), and where a scenario needs an order to be
accepted, the scripted player selects a vehicle of their own and clicks a destination; the observable is
whether the vehicle's destination changes.

**Scenario T1: tick number and tick rate per game speed (claims 1, 2, 5).**
- *Setup:* a small skirmish map, one human player and one computer player, no orders given unless a
  part says so. The speed is set through the lobby's speed control. A display path that does not
  wait for the monitor's refresh, because a 60 Hz wait would hold index 1 (62.5 ticks per second)
  below its rate.
- *Part A (claim 1):* index 1, run as the second game of a session, so that the first game leaves a
  tick number other than 0. The recorder reads the tick number at least once every 4 ms, from before
  the second game's map starts loading until 60 s after its map is visible; with 16 ms cycles it sees
  every value. *Observable:* the values read, in order. *Runs:* 2. *Expected:* the value drops once
  to 0 while the map loads; from then on every value equals the one before it or exceeds it by
  exactly 1, and the values read in the 60 s span 3750 ± 1 % ticks. *Falsified if:* the value after
  the drop is not 0, or the value later falls or grows by 2 or more between two successive
  readings.
- *Part B (claims 2, 5):* start the game, and once the map is visible count ticks over 300 s of real
  time. *Observable:* *N*ₛ, ticks counted in 300 s at index *s*. *Runs:* 2 per index from 0 to 6.
  *Expected:* *N*ₛ within 1 % of 18750 / *s* for *s* from 1 to 6 (18750, 9375, 6250, 4687.5, 3750,
  3125), and *N*₀ ≥ 0.99 × *N*₁ (1 % allows for timing noise). *Falsified if:* any *N*ₛ for *s*
  from 1 to 6 lies outside 1 % of 18750 / *s*; or the counts lie within 1 % of 18000 / *s* (a
  1/60 s unit); or *N*₀ < 0.99 × *N*₁. A run whose host cannot keep up (shown by *N*₁ below the
  expectation while *N*₂ to *N*₆ meet theirs) is repeated on a faster host and is not a
  falsification of claim 2.
- *Setup of parts C and D:* a purpose-made skirmish map, index 6, with the human player's vehicle
  ordered, as soon as orders are accepted (claim 14), to a destination at least 40 cells away along
  open ground, so that it is still moving in the final tick after the end delay. The recorder reads the tick number and
  the vehicle's position at least once every 4 ms, until the vehicle can no longer be read. A run in
  which the vehicle stopped before the game ended is void and repeated. A run counts only if the
  vehicle's position is still read at least 100 ms (more than one cycle of index 6) after the tick
  number last changed; otherwise it is inconclusive and repeated. Under every reading that late
  position is the final state of the game.
- *Part C (claim 1, end by game state):* one one-time trigger whose only event is an elapsed time of
  20 (tick 300, S-0011, claim 13) and whose action is the win action (Import mapping), with a
  parameter value equal to the registration index of the human player's country. The part assumes
  that this action makes the local player win when its value is the registration index of the local
  player's country, and lose otherwise; a run in which the human player does not win is void and
  the map is checked. *Observable:* the tick-number sequence from tick 300 through the end delay,
  the final tick number *v*; the first position read while the tick number stood at *v*, and the
  last position read. *Runs:* 3. *Expected:* the tick number advances beyond 300, continuing in
  steps of 1 through the end delay, whose length is S-0414 claim 8's. In every run the last
  position read differs from the first position read at *v*: the final tick ran while the tick
  number stood at *v* and added nothing. *Falsified if:* the tick number stops at 300 despite
  the win fixture succeeding, or a resolved tick-number transition during the end delay differs
  from 1; or, in every run, the last position read equals the first position read at *v*, while
  the readings at *v* − 1 show the vehicle moving (the final tick added 1, so no tick ran at *v*).
  A run that does not resolve the first position at *v* before that tick's movement is inconclusive
  for the final increment and is repeated; the recorder must resolve that boundary, not merely
  sample the settled final value.
- *Part D (claim 1, leaving from the menu):* no trigger. After at least 30 s, open the options menu
  and leave the game, trying each menu choice that leaves the game in its own runs. *Observable:*
  the tick number
  and the position read while the menu is open; then whether either changes after the menu closes,
  and the final values. *Runs:* 2 per choice. *Expected:* at least one choice ends the game with
  neither value changing after the menu closes (no further tick; the final tick number is the one
  read while the menu was open). A choice that resigns lets ticks run on through the defeat and
  its end delay, and its final tick then behaves as in part C. *Falsified if:* every choice that leaves the game lets at least
  one more tick run after the menu closes and ends as in part C.

**Scenario T2: source of the game speed (claims 7, 12).**
- *Setup:* parts A and B as T1 (a small skirmish map, one human player and one computer player, and
  a display path that does not wait for the monitor's refresh); part C as given there. The tick
  counts are taken as in T1, part B.
- *Part A (skirmish):* the settings file's general game-speed key set to 6 and its skirmish
  game-speed key set to 1. Open the skirmish lobby, leave the speed control as it starts, and start.
  During the skirmish open the options menu once, read the speed control's position and close it.
  *Expected:* the control starts at the second-fastest position, the count is within 1 % of 18750
  (index 1), and the menu's control stands at position 5, that of index 1. *Falsified if:* the
  control starts at another position, or the count is not within 1 % of 18750 (in particular, if it
  is that of index 6), or the menu's control stands at position 0, that of the general key's
  index 6.
- *Part B (skirmish, keys absent):* the skirmish game-speed key removed from the settings file and
  the general key set to 1. Two runs with a named test copy of the rules file: in B1 the lobby
  default key is removed, in B2 it is set to 3. *Expected:* B1: the control starts at the slowest
  position (index 6); B2: at the middle position (index 3); in both, the count matches that index
  by T1. *Falsified if:* either control starts at the second-fastest position (the general key) or
  at any position other than the expected one.
- *Historical campaign observations (out of scope; not measurement scenarios or falsification tests for this revision).* The following earlier proposed procedures and expected observations are retained for a future campaign spec only. They do not support a claim of this revision.
  - *Part C (campaign):* the general game-speed key set to 6, then in a second run to 0; start the first mission of a campaign. During the mission open the options menu once and close it.
    *Expected:* both runs within 1 % of 9375 (index 2); the options menu shows no speed control; after the mission the options menu shows the player's setting again. *Falsified if:* either run differs from 9375 by more than 1 %, or the menu offers the speed control during the mission, or after the mission the options menu does not show the player's setting.
  - *Part D (absent general key, claim 7):* the general game-speed key removed from the settings file, and a campaign mission started as in part C. After the mission, open the options menu where it offers the speed control. *Expected:* the control stands at the middle position (index 3). If the control cannot be reached outside a game, the part is inconclusive and the absent-key value stays untested. *Falsified if:* the control stands at another position.
- *Part E (claim 7, the replacement outlasts the skirmish):* as part A (general key 6, skirmish key
  1). Play the skirmish for at least 10 s and leave it, then, without leaving the program, open the
  options menu where it offers the speed control. *Expected:* the control stands at position 5, that
  of index 1, the lobby's index. If the control cannot be reached outside a game, the part is
  inconclusive. *Falsified if:* the control stands at position 0, the saved general key's index.
- *Observable:* the lobby control's starting position (A, B), whether the options menu shows the
  speed control (C), the control's position during the skirmish (A) and after it or after the
  mission (C, D, E), and ticks counted over 300 s after the map is visible. *Runs:* 1 per part and
  value.

**Scenario T3: drawn frames (claims 6, 8, 9).**
- *Part A (claim 6):* as T1 at indices 1 and 6, with a recorder that counts both ticks and the drawn
  frames the game hands to the display path. *Observable:* ticks and drawn frames over 60 s.
  *Runs:* 2 per index. *Expected:* the two counts differ by at most 1. *Falsified if:* they differ
  by more than 1.
- *Part B (claims 6, 8, 9):* as part A at index 1. Move the input focus to another window for 10 s,
  then back; later open the options menu for 10 s and close it. *Observable:* drawn frames of the
  game handed to the display path during each 10 s stretch, counted as the game's own completed
  drawn frames, not as repaints that a display compatibility layer makes on its own or as drawing of the
  menu itself. *Runs:* 2. *Expected:* at most 1 drawn frame of the game in each stretch.
  *Falsified if:* more than 1. If the recorder cannot tell the game's drawn frames from the menu's own
  drawing, the menu stretch is recorded as inconclusive.

**Scenario T4: focus loss and no catch-up (claims 3, 9).**
- *Setup:* as T1 at index 1.
- *Procedure:* after 60 s, move the input focus to another window for 30 s, then back.
- *Observable:* ticks during the 30 s without focus, and ticks during the 10 s right after focus
  returns.
- *Runs:* 3.
- *Expected:* at most 1 tick without focus; 625 ± 2 % ticks in the 10 s after.
- *Falsified if:* more than 1 tick runs without focus, or the 10 s after focus returns hold more
  than 2 % above 625 (a catch-up burst).

**Scenario T5: options menu and in-game speed change (claim 8).**
- *Setup:* as T1 at index 1.
- *Procedure:* open the options menu for 30 s and close it. Later, open it, move the speed control
  to the slowest position, close it, and count ticks over the next 60 s.
- *Observable:* ticks while the menu is open; ticks over the 60 s after the change.
- *Runs:* 2.
- *Expected:* at most 1 tick while the menu is open; 625 ± 1 % after the change.
- *Falsified if:* more than 1 tick runs while the menu is open, or the count after the change is not
  within 1 % of 625 (index 6).

**Scenario T6: freezes and the chrono flash (claims 10, 11, 14).**
- *Setup:* a purpose-made skirmish map with one human vehicle in the open and one-time triggers
  whose only event is an elapsed time; by S-0011 (claim 13) an elapsed-time value of 20 makes the
  action run in tick 300 and a value of 0 in tick 0. A trigger's actions take effect in the order of
  its action list (S-0011, claim 11). The recorder reads the tick number as in T1 part A and
  captures the screen and counts drawn frames handed to the display path, as T3 does.
- *Observable (every part):* the value at which the tick number stands still; the real time from
  the moment the tick number reaches that value to the moment it reaches the next value (the
  *gap*); whether the mouse pointer is visible during the *hold*, the stretch in which the tick
  number stays at that value; whether an order is accepted during it and after it.
- *Runs:* 3 per part, and per index where a part names several.
- *Part A (claim 10):* elapsed time 20, the pause action with *N* = 10, index 1. *Expected:* the tick
  number stands at 301; the gap is 9.6 s within ± 0.1 s (one cycle of index 1 is 16 ms; screen
  capture adds its own resolution); the pointer is hidden and no order is accepted during the
  hold; orders are accepted after it. There are 599 frozen cycles, each with one drawn frame.
  *Falsified if:* the gap lies outside 9.5 to 9.7 s (in particular, 10.0 s), the tick number
  changes during the hold, the pointer is visible, or an order is accepted during it or refused
  afterwards. A frozen-frame count other than 599 falsifies the keep-up cycle count only when
  the recorder resolves both boundaries and the host sustains index 1; otherwise that count is inconclusive.
- *Part B (claim 11):* elapsed time 20, the chrono-flash action with *D* = 30, index 6. *Expected:*
  orders are blocked and the pointer hidden from tick 300 on; the tick number stands at 374 (steps 1
  to 74 fall in ticks 300 to 373); the gap is 32 cycles of index 6, that is 3.072 s within
  ± 0.1 s; orders are accepted after the hold. *Falsified if:* the tick number stands at a
  value other than 374 (375 alone is consistent with the step counts if the action's own tick takes
  no step, against the order of claim 11; any other value falsifies the step counts), the gap lies
  outside 2.97 to 3.17 s, ticks run during the hold, an order is accepted before the hold
  ends, or orders remain refused after it. For scrolling during the block and hold, use
  S-0101 scenario M7f: a previously held arrow continues before the hold, edge scrolling is
  suppressed throughout the block, and both stop during the hold. For arrow-input
  acceptance (claim 14), use S-0213 Scenario O22: blocked presses and releases change
  no accepted contribution, and unblocking does not start an arrow first pressed during
  the block. O22 supplies the release/re-press tests, controls and falsifiers.
- *Part C (claim 10, several actions):* elapsed time 20, one trigger with two pause actions, *N* = 10 then *N* = 2, index 1. *Expected:* the tick number stands at 301 with a gap of
  1.92 s ± 0.1 s. *Falsified if:* the gap is about 9.6 s (first wins) or about 11.5 s (the freezes
  add).
- *Part D (Edge cases, *N* < 0):* as part C with *N* = 10 then *N* = −1. *Expected:* no hold:
  the gap after 301 is one cycle (16 ms, within the capture's resolution). *Falsified if:* the gap
  is about 9.6 s.
- *Part E (Edge cases, *N* = 0):* elapsed time 20, the pause action with *N* = 0, index 1; wait
  120 s, then abort the game. *Expected:* the tick number stands at 301 for the whole 120 s.
  *Falsified if:* ticks run again within the 120 s.
- *Part F (claim 10, stand-still during a freeze):* as part A, but 1 s after the hold begins,
  move the input focus to another window for 15 s, then back. *Expected:* the first tick after the
  hold runs within 0.1 s of the focus returning, not 8.6 s later. *Falsified if:* the gap
  after the focus returns is about 8.6 s.
- *Part G (claim 11 and F4, with a timed freeze, at several game speeds):* elapsed time 20, one
  trigger with the chrono-flash action (*D* = 30) and then the pause action. Only the tick numbers
  and gaps are judged; whether orders are accepted between the two holds is not (Open questions).
  - *G1:* *N* = 1, index 6 (9 frozen cycles at a host that keeps up). *Expected:* the tick number
    first stands at 301 for about 0.96 s, then runs and stands at 365 ± 1 (steps 1 to 74 fall in
    tick 300, 9 frozen cycles and ticks 301 to 364), with a gap of 3.072 s ± 0.1 s. *Falsified
    if:* either hold starts outside its expected tick-number range, either gap differs by more
    than 0.1 s from its expected value, or a further hold appears within the following 60 s.
    In particular, a second hold at 374 ± 1 would mean frozen cycles took no step.
  - *G2:* *N* = 1, index 1 (59 frozen cycles). *Expected:* the tick number first stands at 301 for
    about 0.96 s, then runs and stands at 315 ± 1 (steps 2 to 60 in the frozen cycles, 61 to 74 in
    ticks 301 to 314), with a gap of 32 cycles of index 1, 0.512 s ± 0.1 s. *Falsified if:* the
    either hold starts outside its expected tick-number range, either gap differs by more than
    0.1 s from its expected value, or a further hold appears within the following 60 s.
    In particular, a second hold at 365 ± 1 would mean the frozen-cycle count does not depend on
    game speed, and one at 374 ± 1 would mean frozen cycles take no step.
  - *G3:* *N* = 2, index 1 (119 frozen cycles, more than the 73 steps, 2 to 74, left up to and
    including step 74).
    *Expected:* one hold at 301 with a gap of 1.92 s ± 0.1 s, and no second hold in the 60 s that
    follow. *Falsified if:* a second hold appears (in particular, at about 374), or the gap lies
    outside 1.82 to 2.02 s.
- *Part H (claim 11, repeated action):* two one-time triggers, elapsed time 20 and elapsed time 21
  (tick 315), each with the chrono-flash action, *D* = 30, index 6. *Expected:* the tick number
  stands at 389 (the chrono flash starts over in tick 315, steps 1 to 74 in ticks 315 to 388),
  and after the hold orders stay blocked for the 60 s observed. *Falsified if:* it stands at 374 (no
  restart), or an order is accepted after the hold.
- *Part I (claims 11, 14, chrono flash at game start):* elapsed time 0, the chrono-flash action
  with *D* = 10, index 6. *Expected:* the tick number stands at 54 (steps 1 to 54 in ticks 0 to
  53), and no order is accepted at any time, during the 60 s observed after the hold as well.
  *Falsified if:* an order is accepted at any time, in particular from tick 4.
- *Historical campaign observation (out of scope; not a measurement scenario or falsification test for this revision).* The following earlier proposed procedure and expected observations are retained for a future campaign spec only; they do not support a claim of this revision. *Part J (claim 11, radar movies):* a named test copy of a campaign mission map with two added one-time triggers whose only event is an elapsed time: elapsed time 20 with the plain radar-movie action and elapsed time 40 (tick 600) with the freezing radar-movie action (Import mapping), both with parameter value 0, the first entry of the movie list. A second test copy adds the chrono-flash action (*D* = 30) to the trigger of the freezing radar-movie action.
  *Observable:* in addition to the common ones, whether the radar panel shows a movie, and from when to when. *Expected:* while the first movie plays the tick number goes on growing. Once the second movie starts, the tick number stands still for as long as it plays and runs again once it ends; how soon after tick 600 the movie starts, and how long it plays, remain questions for the future campaign spec, so only the hold's coincidence with the movie would be judged. With the chrono flash, its own freeze begins earlier than tick 674 by the number of frozen cycles of the movie's hold that fell before its step 74, or merges with the movie's hold when that hold lasts long enough (a possible campaign extension of F4, not a requirement here). *Falsified if:* the tick number stands still during the first movie, or runs on during the second; or, with the chrono flash, its own freeze begins at 674 although the movie's hold had frozen cycles before it.

- *Part K (claim 10, orders already blocked):* two one-time triggers: elapsed time 20 with the
  chrono-flash action (*D* = 30) and elapsed time 21 (tick 315) with the pause action, *N* = 1,
  index 6 (9 frozen cycles). Scripted clicks at a destination cell run throughout, as in T7, from
  tick 300 to the second hold. *Expected:* the first hold stands at 316 for about 0.96 s and the
  second at 365 ± 1 (F4, *n* = 315, *k* = 16); orders are refused and the pointer is hidden in the
  ticks between the two holds, not only during them. *Falsified if:* an order is accepted or the
  pointer is visible between the holds (the pause's end unblocked orders although the chrono
  flash's block was in force).
- *Part N (claim 11, non-positive chrono parameter):* as part B, first with *D* = 0, then
  *D* = −1, with no preceding flash in each case. Also repeat part H with its second action's
  parameter replaced by each value. For the standalone cases, select the vehicle before tick 300.
  *Observable:* holds and order acceptance. *Expected:* the standalone actions cause no hold or
  order block; with a flash already running, its sole hold still begins at 374 and orders are
  accepted afterwards. *Falsified if:* a standalone action blocks orders or freezes, or the
  second action changes the hold or leaves orders blocked. Three runs per case.
- *Part L (claim 11, radar movies in a skirmish):* a purpose-made skirmish map with one one-time
  trigger, elapsed time 20, with the freezing radar-movie action, parameter value 0; index 6.
  *Expected:* the radar panel shows no movie and the tick number stands still at no value for
  longer than one cycle in the 60 s after tick 300. *Falsified if:* the tick number stands still
  for longer than one cycle, or a movie plays. A host stall makes the run inconclusive and it
  is repeated. Repeat with the plain action: no movie and no hold are expected, and either
  falsifies the claim.
- *Historical campaign observation (out of scope; not a measurement scenario or falsification test for this revision).* The following earlier proposed procedure and expected observations are retained for a future campaign spec only; they do not support a claim of this revision. *Part M (claim 11, a value outside the movie list):* the named test copy of a campaign mission map of part J, with the freezing radar-movie action given the parameter value −1 in one run and 99999 in a second. *Expected:* no movie plays and the tick number stands still at no value for longer than one cycle. *Falsified if:* a movie plays, or the game holds for more than 0.5 s.

**Scenario T7: no orders in the first ticks (claim 14).**
- *Setup:* a purpose-made skirmish map with one human vehicle in the open, index 6, and a recorder
  that reads the tick number and the vehicle's destination once per tick and captures the screen.
- *Procedure:* before the map appears, hold the pointer over a cell 5 cells from the vehicle and
  click repeatedly (about 20 clicks per second, so at least one click falls in every cycle of
  96 ms) from the moment the map is shown; the click rate is recorded. Setup 1: the vehicle is
  selected by the map at game start, so every click is a move order. Setup 2: no selection; the
  pointer starts over the vehicle, the first accepted click selects it, and the pointer is then
  moved to the destination cell while clicking goes on.
- *Observable:* the first tick at whose end the vehicle's destination has changed; whether the
  mouse pointer is visible in the screen captures of the first cycles.
- *Runs:* 5 per setup.
- *Expected:* setup 1: tick 4 in every run; setup 2: tick 5 at the earliest (the selection is
  accepted in the cycle of tick 4, the order in a later cycle). The pointer is hidden in the
  captures before the cycle of tick 4 and visible after.
- *Falsified if:* setup 1 changes the destination at the end of tick 3 or earlier, or first at the
  end of tick 6 or later although clicks arrived in every cycle (tick 5 in setup 1 means a click
  missed the cycle of tick 4; such a run is repeated); setup 2 changes it at the end of tick 4 or
  earlier.

**Scenario T8: a network game on a local network (claims 15 to 18).**
- *Setup:* unlike T1 to T7, two machines on one local network, each with the stock game and the
  recorder of the measurement setup, reading only, on both: the tick number at least once every
  4 ms, the game-speed index once per tick, the drawn frames handed to the display path, and the
  screen. No helper that changes rules, houses or computer players during the game is active on
  either machine. A small purpose-made map for two human players and one computer player, each
  player's base far from the others, no orders given unless a part says so. The game is created on
  machine 1 at the middle position of the lobby's speed control (index 3), and machine 1's player
  is listed first. Both windows keep the focus except where a
  part moves it. Every run of a part is a game of its own, except that part D follows part F in
  the same game. Parts A, B, C and F start at least 600 s of real time after the map is visible,
  so that timing orders have been carried out; parts E, G and H follow their own timing. Both machines read their tick rate over the 30 s just
  before the part as the *base rate* (expected 20 ticks per second, F5, unless *C* is lower).
  *The order delay.* In the original its value depends on the network (Terms), so every run reads it as *W*, in ticks: in
  part C, *W* = *t*₁ − *t*₀; in a run with no speed change of its own (parts A and G), once the
  part's observation is over, machine 2's player opens the menu, sets the control one position
  slower and closes it, and *W* is the tick at which the index changes less the tick at which
  the menu closed. Every expectation below that names the order delay uses that run's *W*. One
  delay applies at a time to the orders of every player on every machine (Terms), so a timing
  order is carried out *W* ticks after it is sent unless a timing order carried
  out in between has changed the delay; the falsifiers below do not rest on *W*, which only places
  an expected change.
  *Runs:* 2 per part.
- *Part A (claim 15):* just before opening the menu, machine 2's player orders one of their
  vehicles to move to a clear cell about 20 cells away; machine 2's player then opens the options
  menu for 30 s and closes it. *Observable:* ticks on both machines during the 30 s; drawn frames
  of the battlefield on machine 2 during it; the vehicle's cell, read on machine 1 once per
  second; machine 1's screen (any message or notice). *Expected:* both tick numbers grow at the
  base rate within 2 % throughout, and agree with each other within *W* ticks; machine 2 hands no
  new battlefield frame to the display path; the vehicle moves towards its destination while
  the menu is open, until it arrives; machine 1 shows no message or notice about machine 2's menu. *Falsified if:* either
  tick number stands still for more than 1 s, or the rate on either machine falls more than 2 %
  below its base rate, or the vehicle stops short of its destination while the menu is open, or machine 1 shows a notice
  of the menu.
- *Part B (claim 16):* machine 2's window is put in the background for 30 s, then brought back.
  *Observable:* ticks on both machines during the 30 s; drawn frames on machine 2. *Expected:*
  ticks run on both machines; on both, the rate during the 30 s is at most 1000 / (⌊1000 / *A*⌋ +
  10) ticks per second plus 2 %, with *A* the agreed tick rate of F5 in force during the part
  (*A* = 20 at index 3 unless *C* is lower, which gives a bound of about 16.7 ticks per second.
  The base rate read before the part is expected to be ⌊1000 / *A*⌋ ms per tick, and a run in
  which it is not is inconclusive), and
  the two machines agree within 2 %; machine 2 makes no drawn frame. *Falsified if:* either tick
  number stands still for more than 1 s (a stand-still); or the rate on either machine stays within
  2 % of its base rate (no slowdown); or the two rates differ by more than 2 %. A rate between
  the expected bound and 2 % below the base rate (at *A* = 20, between about 17.0 and 19.6 ticks
  per second) shows a slowdown but an extra wait shorter than 10 ms per cycle, and falsifies
  claim 16's 10 ms, though not the slowdown itself.
- *Part C (claim 17):* machine 2's player opens the options menu, sets the speed control to the
  slowest position (index 6) and closes it; the tick *t*₀ at which the menu closes is recorded on
  machine 2. *Observable:* the tick at which the index read changes, on each machine; whether
  machine 1 shows a message naming machine 2's house; the tick rate on both machines, tick by tick,
  from *t*₀ until 600 ticks later. *Expected:* the index changes from 3 to 6 in the same tick *t*₁
  on both machines, with *t*₁ > *t*₀ on machine 2 (not at once); machine 1 shows the message and
  machine 2 does not; the rate stays at the base rate until the first tick after *t*₁ whose
  number is a multiple of 256, plus *W* ticks, and is 10 ticks per second within 2 % from
  then on. *Falsified if:* the index changes in different ticks on the two machines, or on
  machine 2 in the tick of *t*₀; or the rate changes before that multiple of 256, or settles
  anywhere but 10 ticks per second within 2 % (10.416… would be the single-player rate). A settled
  rate below 10 is inconclusive (*C* may be the smaller term), as in the further runs below. A run in
  which the rate first changes after a later multiple of 256 still judges the settled rate, and
  records which multiple it was, as evidence on the schedule that claim 13 leaves open. A run in
  which the rate has not changed 600 ticks after *t*₀ is inconclusive. A repeat
  with machine 1 changing the speed checks that a change made on the machine listed first works
  the same way. *Further runs (the caps):* the same, with the control set from index 3 to each of indices
  0, 1, 2, 4 and 5 in turn, one run each, every run in a new game. *Expected:* the settled rate is
  62.5, 45.45…, 30.30…, 15.15… and 12.04… ticks per second within 2 % (F5's table). A settled rate
  below the expected one is inconclusive (*C* may be the smaller term); any other settled rate falsifies the
  cap.
- *Part D (claim 17, control hidden after defeat):* follows part F in the same run, while the
  other two houses play on: it reads the options menu that part F's scripted input opened on
  machine 2 after machine 2's house was defeated, and does not open another. *Observable:* whether
  that menu shows the speed control. *Expected:* it shows no speed control.
  *Falsified if:* the control is shown.
- *Part E (claim 18):* a run of its own, with a test copy of the map that adds one one-time
  trigger whose only event is an elapsed time of 600 (tick 9000) and whose action is the win action
  with the registration index of machine 1's player's country, under the assumption that T1 part C
  states. From 120 s after the map is visible, machine 2's menu is open for 30 s, and later its
  window is in the background for 30 s; the game then runs on to its end and the result screen.
  *Observable:* real time from the map becoming visible
  to the active clock's stop on leaving play for the result screen, measured by the recorder on machine 2, and the duration machine 2 displays.
  *Expected:* the displayed total equals that real time divided by 0.96 s, truncated, within 1 s
  (S-0414 claim 23's 960 ms second), with both 30 s stretches counted. *Falsified if:* the displayed
  total is short by about 31 s or 62 s (a stretch not counted). If machine 2's result screen
  shows no duration in a network game, the part is inconclusive.
- *Part F (claim 15, the menu after resigning):* machine 2's player resigns through the options
  menu, and the scripted input tries to open the menu once every 100 ms until it opens.
  *Observable:* the tick at which the menu first opens, and the tick at which machine 2's house is
  defeated. *Expected:* the menu does not open before the defeat and opens after it. *Falsified
  if:* it opens before the defeat, or never opens after the defeat. A run in which machine 2's house is not defeated within 120 s
  of the resignation (S-0414 claim 11 allows a later defeat, or none) is void and repeated.
- *Part G (claim 17, the start at 30 and the room speed):* a run of its own, at index 3.
  *Observable:* the game-speed index read on both machines in tick 0, and both machines'
  tick rate from the tick the map is visible, tick by tick, until tick 2048. *Expected:* index 3,
  the lobby's value, on both machines in tick 0; about
  30.30… ticks per second within 2 % from the start, then 20 ticks per second within 2 % from a
  tick just after a multiple of 256 (plus *W* ticks) on. *Falsified if:* the rate is 20 from
  the start, or changes to 20 other than just after a multiple of 256, or the index in tick 0
  differs from 3 on either machine. A run in which the rate is
  still 30.30… at tick 2048 is inconclusive on the schedule but still shows the start at 30.
- *Part H (claim 15, a menu open across the end):* a run of its own, with the end delay raised by
  a test rules file as in S-0414 scenario V8, part b. The two human players are allied in the
  lobby and the computer player, their only enemy, has a single object left, which machine 1's
  forces are attacking. Machine 2's player opens the options menu before that object is destroyed
  and keeps it open; 2 s of real time after the computer player's defeat (the end of the game,
  logged by machine 1's recorder), machine 2's player chooses surrender. *Observable:*
  whether machine 2's objects are destroyed, machine 2's result, and the tick at which the game is
  over. *Expected:* the surrender is offered and chosen, nothing of machine 2's is destroyed, its
  result is a win, and the game is over at the tick it would be without the surrender (S-0414
  claim 11's pending-result case). *Falsified if:* the menu closes at the end of the game, or its
  surrender choice cannot be made, or machine 2's objects are destroyed.

**Scenario T9: the options menu during the end delay of a skirmish (claim 15).**
- *Setup:* a skirmish as in S-0414 scenario V8, part b: the end delay raised by a test rules file,
  the human allied with computer player A against computer player B, and B destroyed by the human's
  forces. No helper that changes rules, houses or computer players during the game is active. The
  recorder reads the tick number and whether the options menu is shown.
- *Procedure:* from the tick B is defeated, the scripted input presses the options-menu key once
  every 100 ms until the game is over.
- *Observable:* whether the menu is ever shown between B's defeat and the game being over.
- *Expected:* it is never shown, so no surrender can be given in the end delay of a skirmish.
- *Falsified if:* the menu opens in the end delay. A control run presses the key once, 5 s before
  B is defeated, and must open the menu (otherwise the run is void); the menu stops the cycles
  (claim 8), so the scripted input closes it again 2 s later, play resumes, and the run then goes on
  as the main run, pressing the key from the tick of B's defeat.

## Open questions

- **Network games.** Beyond claims 15 to 18 they are out of scope (claim 13). A future multiplayer
  spec should own the agreed tick rate's capability term *C* and when the first timing order comes,
  the extra drawn frames made while waiting for the other machines, our engine's order delay *W*, the
  original's start rule and extra waits in an online game (claim 19), and the pause
  action and the chrono flash in a network game. Not established here, and not specified: in a
  network game the pause action appears to block orders while ticks go on, and the block does not
  lift by itself (confidence: low; `unmeasured`
  here: no scenario of this spec sets up the pause action in a network game, which is left to the
  future multiplayer spec, claim 13).
- In a network game, a speed change is one of the player's orders (claim 17), so the limit on
  pending orders of S-0213 claim 2 is believed to apply to it: issued while that list is full, it
  would be dropped without notice. S-0213 claim 2 leaves open how orders enter a network game, so
  this is not established (confidence: low; `unmeasured` here: no scenario of this spec fills the
  list in a network game, which is left to the future multiplayer spec, claim 13).
- Whether the original keeps the lobby's starting index in range when the saved value lies outside
  0 to 6 is not established (claim 7).
- Which of the options menu's choices ends a game at once and which one resigns is not established
  (claim 1); scenario T1, part D, tries each.
- Radar-movie timing and its source in movie data are deferred to the future campaign spec.
  No fallback length or importer warning for a missing movie duration is required by this revision.
- When a timed freeze and a chrono flash overlap, each, when it ends, leaves orders blocked or
  accepted as they were when it started, so whether orders are accepted in the ticks between the two holds can depend on which
  started first (scenario T6, part G, judges tick numbers only). The full rule for overlapping
  order blocks is not covered.
- Whether a test map can select a vehicle at game start, as setup 1 of scenario T7 needs, is not
  settled; setup 2 does without it.
- A freeze that starts during ticks 0 to 3 is not covered: whether the release of the order block
  at the end of tick 3 (claim 14) still happens then, and whether orders are accepted after the
  freeze. Only a chrono flash is covered (claim 14, scenario T6, part I).
- Whether the options menu can be opened during a freeze is not covered. If it can, the state table
  lacks that case, and a speed change made during a timed freeze would change the original's number
  of frozen cycles left (claim 10 counts real time), while our engine's *F* stays as fixed when the
  action ran (claims 8 and 10).
- When two pause actions run in the same tick (claim 10), whether orders are accepted
  after the freeze is not covered: the second action finds orders blocked by the first. Scenario
  T6, part C, judges tick numbers and the gap only.
- The original's behaviour for very large parameters is not covered: a pause action's *N* or a
  chrono flash's *D* so large that the counts of claims 10 and 11 would exceed the range of the
  original's integers. Formulas F2 to F4 treat *N* and *D* as unbounded integers; the scenarios use
  small values only.

## Changes for other specs

- **S-0414 (approved, revision 15), claim 23, network games.** The duration paragraph can add: in
  a network game the active clock does not stop while a player's options menu is open, while a
  window lacks the focus, or for the pause action or a chrono flash; only leaving play stops it
  (S-0201 claim 18).
- **S-0414, claim 23, skirmish (correction).** "pauses that clock when the game is paused for the
  options menu or lost focus … Frozen cycles and the end delay are included" disagrees with the
  original. The original stops the clock while the options menu is open and during every hold (a
  timed freeze, a chrono flash's freeze), and restarts it when the last freeze in force ends. A
  stand-still for lost focus does not stop it. The sentence can read: the clock pauses while the
  options menu is open and while the game is frozen; time without the input focus and the end
  delay are included (S-0201 claim 18). This agrees with S-0201 claim 10, by which a timed freeze's
  real time keeps running through a stand-still for lost focus.
- **S-0414, claim 11 and scenario V8, part b (the surrender during the end delay).** In the
  original the options menu is refused from the end of the game on (S-0201 claim 15), and in a
  skirmish no tick runs while it is open, so a skirmish player cannot surrender in the end delay.
  V8 part b's input, "the human surrenders in the delay", therefore cannot be given in a skirmish,
  and claim 11's sentence "In the original only the local player can surrender, through the
  options menu" can add "and not from the end of the game on (S-0201 claim 15)". The pending-result
  case can be given in the original only in a network game, from a menu opened before the end
  (S-0201 scenario T8, part H); in a skirmish it is a rule of our engine, where the surrender order
  is legal at any time (S-0213), and its backing there is `unmeasured`. V8 part b can either point
  to T8 part H or be marked as a run of our engine only; S-0201 scenario T9 shows that the
  original's menu stays closed in the skirmish end delay.
- **S-0414, scenario V19.** "menu and focus pauses add none, freezes and the end delay add active
  time" can read "the menu pause and the frozen stretch add none; the focus loss and the end delay
  add active time", for a skirmish. A network run can point to S-0201 scenario T8, part E.
- **Glossary, *cycle*.** "While the game stands still (the game window has lost the input focus,
  or the options menu is open) there are no cycles" can begin "In a single-player game, while …",
  with a pointer to S-0201 claims 15 and 16 for network games, where neither stops the cycles.
- **Glossary, *multiplayer game, network game*.** "How the original runs network games (the
  agreed tick rate, the synchronisation draws) is left by S-0201 (claim 13) …" can add that S-0201
  claims 15 to 19 cover the options menu, lost focus, the speed control with the cap of the agreed
  tick rate, and the active clock in a network game.
- **Glossary, Open questions.** The item "How game speed works in network games is left by S-0201
  (claim 13) to a future network-game spec" can be narrowed to the agreed tick rate's capability
  term and the timing order's schedule (S-0201 claims 13 and 17). Claim G3's speed-normalised
  delays in a network game use the index in force, which a speed order changes on every machine in
  one tick (S-0201 claim 17); G3 can say so, or keep its skirmish scope and point there.

## Questions for the project lead

- **Scope (Q-1806).** This revision brings four points of network games into S-0201 (claims 15 to
  18, with claim 19 for games played online) while the glossary still states that network games are out of scope. Confirm that S-0201 is
  the place for them until a multiplayer spec exists, and that claim 17's cap table may be stated
  now while the rest of the agreed tick rate stays deferred.
- **Lost focus in a network game (claim 16).** The original slows every machine while any player's
  window lacks the focus. This revision proposes that our engine does not, as a departure without a
  switch, since it is real-time behaviour. Confirm, or ask for the slowdown to be reproduced.
- **The speed control in a network game (claim 17).** Q-1806's interim rule ignores the control.
  In the original a change is an order that changes the game-speed index, and with it the
  speed-normalised delays, on every machine in one tick, so ignoring it changes game state. This
  revision proposes reproducing the order and the index change, and pacing at ⌊1000 / cap(*s*)⌋ ms
  from the cycle after the change, as a departure (the original follows only at the next timing
  order, sent only in ticks whose number is a multiple of 256, capped by the slowest machine,
  and starts every game on a local network at 30 ticks per second; online, claim 19). Our engine
  makes no difference between a game on a local network and an online game (claim 19). Confirm.
- **The speed control after defeat in a skirmish (claims 8 and 17).** Where our engine lets the
  person watch a skirmish on after the human player's defeat (the original ends the game there),
  a menu opened then shows no speed control, as the original's network-game menu does after the
  player's defeat. This is a design decision of our engine with no original behaviour to follow.
  Confirm it.
- **The active clock in a skirmish (claim 18).** The original stops it while frozen and keeps it
  running without focus, against S-0414 claim 23 as approved (Changes for other specs). Confirm
  that S-0414 takes the correction in its next revision, with scenario V19's expectations.

- **Cycle wording, for hand-off.** Terms, *Cycle*, and claim 2 incorporate S-0213 claim 1's
  limit of at most one input event per unfrozen cycle. S-0213 owns the limit and its scenario O9.
  Claim 2 here owns pacing. Confirm this wording together with the S-0213 revision handed off,
  including its citation to this spec's Terms rather than to claim 2 for input timing. Check the
  compatible definitions and claims in the latest same-batch revisions at sign-off, not against
  an earlier batch's revision list.
- **Tick-order boundaries, for sign-off.** The current S-0203 rows 1 and 4 retain trigger
  polling and the chrono-flash step: an action from row 1 takes its first step in that tick,
  while an action after row 4 first steps in the next tick. Its claim 7 still places orders
  after the turn pass and before the next tick's turns; claim 9 still counts timings against
  the tick number, not turns. These agree with claims 6, 11 and 14 and F4 here, and with
  S-0213 claim 1's input and order boundaries. Reconfirm those labels and boundaries in
  the actual revisions handed off together. Production-start draw assignment belongs to
  S-0202, S-0203 and S-0404; it changes none of these pacing or chrono-flash rules.
- **Companion pointers in S-0011, for sign-off.** The current S-0011 companion table after I4
  points here for the pause and chrono-flash actions, the no-effect skirmish radar-movie actions,
  and T1 part C's win-action fixture assumption. These pointers agree with this revision's
  claims 10 and 11, Import mapping and scenario T1 part C. The win-action row is a fixture
  assumption only, not a general victory rule. Confirm the actual revisions handed off together:
  the pointers neither extend S-0011's supported action subset nor specify campaign movies.
- **Citations, for sign-off.** The current glossary's claim G3 holds exception 2 and gives
  `quirks.speed_normalised_delays` the scope used here. S-0011, S-0202, S-0203 and S-0205 are
  approved specs cited by number and claim. The glossary's *departure* entry lists the pause
  action with *N* = 0, the paced period of index 0 and the real-time length of a timed freeze as
  departures of this spec.
