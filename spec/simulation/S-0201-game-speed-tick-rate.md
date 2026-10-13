---
id: S-0201
title: Game speed and tick rate
topic: simulation
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: []
supersedes: []
---

# S-0201: Game speed and tick rate

## Summary

How many ticks the original runs per second of real time at each game speed, how it paces ticks
against the host clock, where a skirmish's and a campaign mission's game speed comes from, and when
no tick runs (lost focus, the options menu, a freeze) or the player cannot give orders although
ticks run (the first ticks of a game, a chrono flash). It owns exception 2 of glossary claim G3,
the game-speed dependence of a chrono flash that overlaps a freeze, and that exception's part of the
shared switch `quirks.speed_normalised_delays`; what happens inside one tick is S-0203.
This revision covers skirmishes only. Network games and campaigns are out of scope; the retained
campaign passages record earlier findings for a future campaign spec, not requirements for this
revision's implementation or measurement.

## Terms

- **Tick number.** As in the glossary: the number of ticks a game has completed. It is 0 while the
  first tick of a game runs and grows by 1 at the end of every tick that does not end the game
  (claim 1). Timings of game state counted in ticks are measured against it (S-0203, claim 9).
- **Pacing unit.** 16 milliseconds of host time. The original paces against a clock whose reading
  is *u*(*t*) = ⌊*t* / 16⌋ pacing units at host time *t*, where *t* is the reading of the host's
  millisecond clock, a non-negative integer. The reading advances by 1 on every 16 ms boundary.
  62.5 pacing units make one second.
- **Cycle.** As in the glossary: the unit of real time in which the original runs a game. Cycles
  follow one another without overlap. In a single-player game every cycle makes one drawn frame (claim 6). A cycle
  that is not frozen takes at most one input event from the player's waiting inputs, in the order
  they were made, and then runs one tick (S-0203). An input event is a key or mouse button going
  down or coming back up. A frozen cycle (claim 10) takes no input and runs no tick. S-0213,
  claim 1, owns the input-event limit, its confidence and its backing (scenario O9); claim 2 here
  owns pacing, not that limit. The cycle lasts at least its paced period (none at
  index 0, claim 5), measured from its start (claim 2). During a stand-still there are no cycles at
  all.
- **Stand-still.** A stretch of real time with no cycles, as in the glossary's *cycle* entry: the
  game window has lost the input focus (claim 9), or the options menu is open (claim 8). "The game
  stands still" means the same. No tick runs and no drawn frame of the game is made during it. Where
  a rule holds for one of the two causes only, the spec names the cause.
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
  movie not yet ended, the game is under an untimed freeze (claim 11).
- **Movie length *M*.** In our engine, the number of cycles a radar movie occupies, fixed when the
  movie starts (claim 11, Formulas F5).

## Behaviour

1. **Tick number.** At game start the tick number is 0. It grows by exactly 1 at the end of every
   tick that runs and does not end the game, and nothing else changes it during a game. How a game
   ends decides what it holds afterwards:
   - **Ended by a rule of game state.** When victory or defeat is decided during a tick, that tick
     runs in full but does not add 1, so afterwards the tick number is the number of the last tick
     that ran. A resignation counts here: it makes the player's house lose in a later tick.
   - **Left from the options menu.** When the player leaves the game through an options-menu choice
     that ends it at once, the game ends between two cycles and no further tick runs. The tick
     number stays at the count of ticks run, one more than the number of the last tick.
   - Which of the menu's choices ends the game at once and which one resigns is not traced (Open
     questions). Our engine reproduces both rules.
   - Backing: `measurable` (scenario T1, part A) for the start at 0 and the step of 1; `measurable`
     (scenario T1, parts C and D) for the two ways a game ends
   - Confidence: high for the start and the step; medium for the two ways a game ends; low for
     which menu choice is which

2. **Pacing at game speeds 1 to 6.** In a single-player game at game-speed index *s* from 1 to 6,
   a cycle does not end before the pacing clock has advanced by *s* pacing units since the cycle
   began. The cycle's own work (input, the drawn frame and the tick) counts inside that wait, so the
   wait is only what is left of it. When the work alone takes longer, there is no wait at all. The
   next cycle begins as soon as the wait ends. With a host that keeps up, cycles therefore end on
   16 ms boundaries and the mean tick rate is *R*(*s*) = 62.5 / *s* ticks per second (Formulas F1,
   Constants). This is not 60 / *s*: the pacing unit is 16 ms, not 1/60 s.
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
     sets the number of frozen cycles of a timed freeze (claim 10) and of a freezing radar movie
     (claim 11), and through a chrono flash in force that number changes how many ticks run before
     the chrono flash's own freeze (claim 11). So the game speed, and the choice of the paced period
     of index 0 (claim 5), reach game state there, as the game speed does in the original. In the
     original the host reaches game state on that path as well (claims 10 and 11); our engine
     removes only that. These two freezes are the only paths from real time or from the paced
     period to game state; the rest of the spec is counted in ticks. The one switch
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
   That drawn frame shows the state at the end of the previous tick, including the orders that took
   effect at its end (S-0203, claim 7). The tick of its own cycle does not show in it yet. This holds
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
     replacement; `measurable` (scenario T2, part D) for the absent-key value 3, which is
     observable only if such an options menu can be opened, otherwise the part is inconclusive and
     this value stays untested
   - Confidence: medium for the lobby's initial value (it is traced; whether the original keeps the
     index in range when the saved value lies outside 0 to 6 is not); low for the absent-key
     value 3, which no game in the scope of this spec uses

8. **Changing speed during a skirmish.** The options menu's speed control sets the index to
   6 − *p* for control position *p* (0 slowest to 6 fastest), so the same seven indices as in the
   lobby. The new index takes effect from the next cycle. With it come the paced period of claim 2,
   the speed-normalised delays that start from then on (glossary claim G3) and, in our engine, the
   number of frozen cycles of a timed freeze that starts from then on (claim 10). While the options
   menu is open, no cycle runs: no tick, no drawn frame of the game. Closing it resumes the cycles
   without catching up (claim 3).
   - Backing: `measurable` (scenario T5; scenario T3, part B, for the drawn frames)
   - Confidence: medium

9. **No ticks while the window lacks focus.** In a single-player game, while the game window does
   not have the input focus, no cycle runs and the game stands still. It resumes, without catching
   up (claim 3), when the window regains focus. Our engine also runs no cycle while its window
   lacks focus, and a stand-still uses up no freeze (claim 10). Window focus is not part of game
   state: no rule of game state in our engine reads real time (claim 4), so the stand-still changes
   no result, and it is a rule of presentation. In our engine a radar movie's length is counted in
   cycles too, so a stand-still does not use it up either (claim 11).
   - Backing: `measurable` (scenario T4) for single-player games; network games are out of
     scope (claim 13)
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
    - **Radar movies in the original.** In a campaign mission, the plain radar-movie action and the
      freezing radar-movie action (Import mapping) each give the sidebar's radar panel a movie to
      play. The panel plays these movies one at a time, in the order in which their actions ran, so
      a movie starts only once every movie given before it has ended. While the earliest-given
      movie that has not yet ended is one given by the freezing action, the game is under an
      untimed freeze, which ends when that movie ends, and a chrono flash in force takes one step in
      each of its frozen cycles as well. A movie given by the plain action freezes nothing. A
      parameter value outside the game's movie list gives no movie, and in a skirmish both actions
      do nothing. A movie ends when its playback ends, in real time, so in the original the
      freeze's length depends on the host (compare claim 10, at a host that does not keep up). Which
      movie a value names and how it is drawn are presentation, not covered by this revision.
    - **Radar movies in our engine (departure).** The number of frozen cycles of a freezing movie
      must not depend on real time, so our engine counts a movie's length in cycles. Each movie has
      a playing time *T*, an integer number of milliseconds, a property of the movie's own data (the
      movie's format spec owns where it comes from; Open questions). A movie given in the tick of
      cycle *c* starts at cycle *c* + 1 if no earlier-given movie is still playing, and otherwise in
      the cycle after the last cycle of the one before it. It occupies *M* = max(1, ⌈*T* / *P*⌉)
      consecutive cycles (Formulas F5), *P* being the paced period at the game speed in force when
      it starts. Frozen and unfrozen cycles count alike, and a stand-still counts for none. If it
      is a freezing movie, each of its *M* cycles is a frozen cycle, so *F* = *M* in F4. The drawn
      movie follows this timeline, not the other way round. This is a *departure* with no switch,
      as claim 4: it replaces a count that depends on the host with a count fixed when the movie
      starts, and the game speed still reaches game state through it, exactly as through a timed
      freeze (claim 4, Edge cases and quirks).
    - **Our engine** counts the chrono flash's steps per cycle as the original does by default
      (`quirks.speed_normalised_delays` on, the original), with cycles paced as in claim 4 and
      timed freezes counted in cycles as in claim 10. With the switch off, a chrono flash steps in
      a frozen cycle only while its own freeze is in force (Edge cases and quirks; F4).
    - Backing: `measurable` (scenario T6, parts B, G, H and I) for the chrono flash, including the
      game-speed dependence (part G); `measurable` (scenario T6, parts J, L and M) for the radar
      movies in the original, which state only what the original does (the freeze during a
      freezing movie, nothing in a skirmish, no movie for a value outside the list); `unmeasured`
      for our engine's movie timeline, a decision of our engine as in claim 4
    - Confidence: medium for the chrono flash's step counts and the game-speed dependence; low for
      the radar movies

12. **Campaign missions.** During a campaign mission the game speed is index 2, whatever the
    player has set, and the options menu does not offer the speed control. The player's own
    setting is put aside when the mission starts and restored when it ends. The speed-normalised
    delays of glossary claim G3, and the frozen cycles of claim 10, therefore use index 2 in a
    campaign mission. The original lifts this only through a developer start-up option, which is not
    one of our engine's inputs, so our engine always runs campaign missions at index 2.
    - Backing: `measurable` (scenario T2, part C)
    - Confidence: high

13. **Network games (out of scope).** This spec covers single-player games only: a skirmish or a
    campaign mission. Network games are out of scope; a future multiplayer spec owns them,
    including the agreed tick rate, the way a speed change is sent, how losing focus and the pause action behave, and the draws that network games take for the synchronisation
    checksum (S-0202, claim 12, states the draws that those games take from the scenario random
    stream). Nothing else in this spec holds for network games,
    except where it says so.
    - Backing: `unmeasured` (network games are outside the measurement setup of this revision)
    - Confidence: low

14. **No orders in the first ticks of a game.** When a single-player game starts, orders are
    blocked during the cycles of ticks 0, 1, 2 and 3. The block is released at the end of tick 3,
    so from the cycle of tick 4 on orders are accepted: the earliest order takes effect at the end
    of tick 4 (S-0203, claim 7) and objects first act on it in tick 5. The drawn frames of those
    cycles are made as usual. If a map's own trigger actions have already blocked orders at that
    moment, this rule adds nothing. A chrono flash (claim 11) that starts during ticks 0 to 3
    cancels the release at the end of tick 3, and because orders were blocked when it started, they
    stay blocked after it ends (Edge cases and quirks). Our engine reproduces this; it depends on
    ticks only, not on real time.
    - Backing: `measurable` (scenario T7; scenario T6, part I, for the chrono-flash case)
    - Confidence: medium for the count of ticks and the chrono-flash case; low for which inputs are
      blocked: orders and the pointer are, and whether the player can still scroll the view during
      an order block is not traced (Open questions; scenario T6, part B, records it)

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

**F4 Chrono flash with a timed freeze.** (A freezing radar movie is a freeze like a timed one,
with *F* = *M* in our engine, F5. Several freezes that overlap or follow one another make one stretch of frozen cycles, and *F* is the length of the stretch that begins in the cycle after tick *n*; see the rule after the table.) A timed freeze with *F* frozen cycles starts in tick *n*,
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

*Where the table does not apply: a freeze that begins after the stretch has begun.* A freezing movie
can start in a cycle that is already frozen (it was given behind another movie, and the earlier
movie ends during a freeze or while the chrono flash's own freeze is in force). Then the frozen
cycles are the union of the freezes' cycles, and the tick number stands still through all of them
as one hold. The chrono flash's own freeze lies at fixed cycles in either case. With the switch on
it is the *D* + 1 cycles after the chrono flash's cycle of step *D* + 44, whatever else freezes,
because the chrono flash steps in every cycle. With the switch off it begins in the cycle after the
tick that took step *D* + 44, and no freeze that begins later moves it. A movie that begins inside
the chrono flash's own freeze adds only the cycles that extend beyond that freeze's last cycle. This
follows from claim 11's per-cycle rule and is `unmeasured` as a separate case.

**F5 Radar movie length (our engine).** For a movie of playing time *T* ms (integer, *T* ≥ 0) that
starts when the paced period is *P* ms:

> *M* = max(1, ⌈*T* / *P*⌉) cycles
>
> *M* = 1 for a movie with no stated *T*, as for *T* = 0 (the importer warns)

A movie of the list for which no playing time is known thus holds the game for one frozen cycle in
our engine, a deterministic stand-in until the movie's format spec owns *T*. Integer arithmetic: the ceiling of an exact quotient of integers, so no floating point and no
rounding other than the stated ceiling. At index 0, *P* = 8 ms (claim 5). A freezing movie has
*F* = *M* frozen cycles.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Pacing unit | 16 | ms | Claim 2; the host's millisecond clock read in whole 16 ms units |
| Pacing units per second | 62.5 | pacing units per s | 1000 / 16 |
| Paced period | 16 × *s* | ms | Index *s* from 1 to 6 (claim 2, F1) |
| Paced period at index 0 (original) | none (no wait) | ms | No wait (claim 5) |
| Paced period at index 0 (our engine) | 8 | ms | Departure, claim 5 |
| Campaign game speed | 2 | game-speed index | Claim 12 |
| Speed-control positions | 7 | positions | Index = 6 − position (claims 7 and 8) |
| Freeze length per second of parameter | 60 | pacing units | 960 ms (claim 10, F2) |
| Tick number at game start | 0 | ticks | Claim 1 |
| Chrono flash, steps before its freeze | *D* + 44 | steps (one per cycle) | Claim 11, F3; *D* is the action's parameter |
| Chrono flash, frozen part | *D* + 1 | cycles | Claim 11, F3 |
| Radar movie length (our engine) | max(1, ⌈*T* / *P*⌉) | cycles | Claim 11, F5; *T* is the movie's playing time in ms, owned by the movie's format |
| Ticks without orders at game start | 4 | ticks | Ticks 0 to 3 (claim 14) |
| Absent-key default, settings file general game speed | 3 | game-speed index | Import mapping |
| Absent-key default, rules lobby game speed | 6 | game-speed index | Import mapping |

## State and ordering

**Whether a cycle runs a tick** (single-player games):

| State | Event | Result |
|---|---|---|
| Running | Cycle begins | One drawn frame and one tick; the cycle lasts at least its paced period (none at index 0, claim 5; claim 2) |
| Running | Window loses focus | Stand-still (focus lost): no cycles (claim 9) |
| Running | Options menu opened | Stand-still (menu open): no cycles (claim 8) |
| Stand-still (menu open) | Options menu closed | Running from the next cycle, at the index then set, with no catch-up (claims 3 and 8) |
| Running | Pause action runs with *N* ≥ 1 | Frozen from the next cycle (claim 10) |
| Running | A freezing radar movie becomes the earliest-given movie not yet ended (campaign only) | Frozen from that movie's first cycle, untimed (claim 11) |
| Running | Chrono flash takes its step *D* + 44 | Frozen from the next cycle, untimed (claim 11) |
| Frozen | Cycle begins | One drawn frame, no tick, no input; the tick number stays; a chrono flash in force takes one step |
| Frozen | Window loses focus | Stand-still (focus lost) with the freeze in force. Original: a timed freeze's real time keeps running and can end meanwhile; a chrono flash takes no step. Our engine: the frozen cycles left do not change, nor do the cycles a radar movie has left (claims 10 and 11) |
| Stand-still (focus lost) | Window regains focus | Frozen if a freeze is still in force, otherwise Running; no catch-up |
| Frozen | Every freeze has ended | Running from the next cycle |
| Game starts | Cycles of ticks 0 to 3 | Running, but orders blocked (claim 14) |

**Ordering consequences.** A drawn frame of cycle *k* shows the state left by the tick of cycle
*k* − 1. The game speed set during cycle *k* governs the paced period from cycle *k* + 1 on. A
trigger action that starts a freeze does not stop the rest of its own tick, so the tick number
during a freeze is one more than the number of the tick in which the action ran.

## Edge cases and quirks

- **The 16 ms unit.** At a host that keeps up, the original's paced period at index *s* from 1 to 6
  is 16 × *s* ms, a whole number of 16 ms pacing units, not a multiple of 1/60 s. Our engine
  reproduces the 16 ms unit exactly (claim 4). No switch: how ticks are spread over real time is
  real-time behaviour, which under glossary *quirk switch* takes no switch. The pacing reaches game
  state only through the frozen cycles of a timed freeze and of a freezing radar movie, and that path
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
  depends on the host; a freezing radar movie ends when its playback ends, also in real time. Our
  engine counts frozen cycles (claim 10, F2), fixed when the action runs, and a movie's cycles
  (claim 11, F5), fixed when it starts. No switch: real-time behaviour.
- **The game speed decides a chrono flash's ticks during a timed freeze.** In the original a chrono
  flash takes one step in every frozen cycle of a timed freeze (and of a radar-movie freeze), so the
  game speed changes how many ticks run before its own freeze, and whether there are two holds or
  one (claim 11, F4); in our engine through *F* of claim 10 and *M* of claim 11. This is exception 2 of glossary claim G3. Switch
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
| Map, trigger actions of types 100 (the plain radar-movie action) and 117 (the freezing radar-movie action) | a movie for the radar panel to play (claim 11); a movie given by type 117 freezes the game while it plays, one given by type 100 does not; the number slot is the movie's position in the game's movie list | integer, from the number slot as for type 110; a value outside the list gives no movie | 0, as for type 110 |
| Map, trigger action of type 1 (the win action) | used by scenario T1, part C, only, which states the assumption it relies on; its parameter is the action's number slot | integer, from the number slot as for type 110 | 0, as for type 110 |
| Rules, section `[General]`, key `GameSpeedBias` | not a game-speed input: a movement multiplier that the movement specs own | decimal | — |

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
  open ground, so that it is still moving when the game ends. The recorder reads the tick number and
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
  the map is checked. *Observable:* the final tick number *v*; the first position read while
  the tick number stood at *v*, and the last position read. *Runs:* 3. *Expected:* in every run the
  last position read differs from the first position read at *v*: the last tick ran while the tick
  number stood at *v* and added nothing. *Falsified if:* in every run the last position read equals
  the first position read at *v*, while the readings at *v* − 1 show the vehicle moving (the last
  tick added 1, so no tick ran at *v*).
- *Part D (claim 1, leaving from the menu):* no trigger. After at least 30 s, open the options menu
  and leave the game, trying each menu choice that leaves the game in its own runs. *Observable:*
  the tick number
  and the position read while the menu is open; then whether either changes after the menu closes,
  and the final values. *Runs:* 2 per choice. *Expected:* at least one choice ends the game with
  neither value changing after the menu closes (no further tick; the final tick number is the one
  read while the menu was open). A choice that resigns lets ticks run on until the defeat, and its
  end then behaves as in part C. *Falsified if:* every choice that leaves the game lets at least
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
- *Part C (campaign):* the general game-speed key set to 6, then in a second run to 0; start the
  first mission of a campaign. During the mission open the options menu once and close it.
  *Expected:* both runs within 1 % of 9375 (index 2); the options menu shows no speed control; after
  the mission the options menu shows the player's setting again. *Falsified if:* either run differs
  from 9375 by more than 1 %, or the menu offers the speed control during the mission, or after the
  mission the options menu does not show the player's setting.
- *Part D (absent general key, claim 7):* the general game-speed key removed from the settings
  file, and a campaign mission started as in part C. After the mission, open the options menu where
  it offers the speed control. *Expected:* the control stands at the middle position (index 3). If
  the control cannot be reached outside a game, the part is inconclusive and the absent-key value
  stays untested. *Falsified if:* the control stands at another position.
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
  captures the screen.
- *Observable (every part):* the value at which the tick number stands still; the real time from
  the moment the tick number reaches that value to the moment it reaches the next value (the
  *gap*); whether the mouse pointer is visible during the *hold*, the stretch in which the tick
  number stays at that value; whether an order is accepted during it and after it.
- *Runs:* 3 per part, and per index where a part names several.
- *Part A (claim 10):* elapsed time 20, the pause action with *N* = 10, index 1. *Expected:* the tick
  number stands at 301; the gap is 9.6 s within ± 0.1 s (one cycle of index 1 is 16 ms; screen
  capture adds its own resolution); the pointer is hidden and no order is accepted during the
  hold; orders are accepted after it. *Falsified if:* the gap lies outside 9.5 to 9.7 s (in
  particular, 10.0 s), the tick number changes during the hold, or an order is accepted
  during it.
- *Part B (claim 11):* elapsed time 20, the chrono-flash action with *D* = 30, index 6. *Expected:*
  orders are blocked and the pointer hidden from tick 300 on; the tick number stands at 374 (steps 1
  to 74 fall in ticks 300 to 373); the gap is 32 cycles of index 6, that is 3.072 s within
  ± 0.1 s; orders are accepted after the hold. *Falsified if:* the tick number stands at a
  value other than 374 (375 alone is consistent with the step counts if the action's own tick takes
  no step, against the order of claim 11; any other value falsifies the step counts), the gap lies
  outside 2.97 to 3.17 s, ticks run during the hold, or an order is accepted before the hold
  ends. *Also recorded, not judged (claim 14, Open questions):* whether the keyboard's scroll keys
  and the pointer at the screen edge move the view during the ticks before the hold, while orders
  are blocked.
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
    if:* the second hold begins at 374 ± 1, as it would if frozen cycles took no step.
  - *G2:* *N* = 1, index 1 (59 frozen cycles). *Expected:* the tick number first stands at 301 for
    about 0.96 s, then runs and stands at 315 ± 1 (steps 2 to 60 in the frozen cycles, 61 to 74 in
    ticks 301 to 314), with a gap of 32 cycles of index 1, 0.512 s ± 0.1 s. *Falsified if:* the
    second hold begins at 365 ± 1 (the count of frozen cycles does not depend on the game speed)
    or at 374 ± 1 (frozen cycles take no step).
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
- *Part J (claim 11, radar movies):* a named test copy of a campaign mission map with two
  added one-time triggers whose only event is an elapsed time: elapsed time 20 with the plain
  radar-movie action and elapsed time 40 (tick 600) with the freezing radar-movie action (Import
  mapping), both with parameter value 0, the first entry of the movie list. A second test copy adds
  the chrono-flash action (*D* = 30) to the trigger of the freezing radar-movie action.
  *Observable:* in addition to the common ones, whether the radar panel shows a movie, and from when
  to when. *Expected:* while the first movie plays the tick number goes on
  growing. Once the second movie starts, the tick number stands still for as long as it plays and
  runs again once it ends; how soon after tick 600 the movie starts, and how long it plays, are the original's and are not
  what our engine's timeline of claim 11 reproduces (it fixes them by *T*), so only the hold's
  coincidence with the movie is judged. With the chrono flash, its own freeze begins earlier
  than tick 674 by the number of frozen cycles of the movie's hold that fell before its step 74, or
  merges with the movie's hold when that hold lasts long enough (F4, with *F* the frozen cycles of
  the movie's hold). *Falsified if:* the tick number stands still during the first movie, or runs
  on during the second; or, with the chrono flash, its own freeze begins at 674 although the
  movie's hold had frozen cycles before it.

- *Part K (claim 10, orders already blocked):* two one-time triggers: elapsed time 20 with the
  chrono-flash action (*D* = 30) and elapsed time 21 (tick 315) with the pause action, *N* = 1,
  index 6 (9 frozen cycles). Scripted clicks at a destination cell run throughout, as in T7, from
  tick 300 to the second hold. *Expected:* the first hold stands at 316 for about 0.96 s and the
  second at 365 ± 1 (F4, *n* = 315, *k* = 16); orders are refused and the pointer is hidden in the
  ticks between the two holds, not only during them. *Falsified if:* an order is accepted or the
  pointer is visible between the holds (the pause's end unblocked orders although the chrono
  flash's block was in force).
- *Part L (claim 11, radar movies in a skirmish):* a purpose-made skirmish map with one one-time
  trigger, elapsed time 20, with the freezing radar-movie action, parameter value 0; index 6.
  *Expected:* the radar panel shows no movie and the tick number stands still at no value for
  longer than one cycle in the 60 s after tick 300. *Falsified if:* the tick number stands still
  for more than 0.5 s, or a movie plays. The same with the plain action must show no movie.
- *Part M (claim 11, a value outside the movie list):* the named test copy of a campaign
  mission map of part J, with the freezing radar-movie action given the parameter value −1 in one
  run and 99999 in a second. *Expected:* no movie plays and the tick number stands still at no value
  for longer than one cycle. *Falsified if:* a movie plays, or the game holds for more than 0.5 s.

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

## Open questions

- **Network games.** Out of scope for this spec (claim 13). A future multiplayer spec should own the
  agreed tick rate, its negotiation, the extra drawn frames made while waiting, the order that
  changes speed and the behaviour of focus loss and of the pause action.
- Whether the original keeps the lobby's starting index in range when the saved value lies outside
  0 to 6 is not traced (claim 7).
- Which of the options menu's choices ends a game at once and which one resigns is not traced
  (claim 1); scenario T1, part D, tries each.
- The playing time *T* of each radar-panel movie (claim 11, F5) is not given here. Which spec of the
  movie data owns it, and whether it should come from the movie's frame count and playback rate, is
  open; until then a movie with no stated *T* is given *M* = 1 cycle (F5) and an importer warning, so the
  chrono flash's steps never depend on an undefined length.
- When a timed freeze and a chrono flash overlap, each, when it ends, leaves orders blocked or
  accepted as they were when it started, so whether orders are accepted in the ticks between the two holds can depend on which
  started first (scenario T6, part G, judges tick numbers only). The full rule for overlapping
  order blocks is not covered.
- Whether the player can scroll the view during an order block (the first ticks of a game, or a
  chrono flash before its freeze) is not traced (claim 14). Scenario T6, part B, records it without
  an expectation; a later revision should state the rule.
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

## Questions for the project lead

- **Trigger actions outside S-0011's subset, for sign-off.** This spec states the parameter
  reading of the win action, the two radar-movie actions, the pause action and the chrono-flash
  action (Import mapping, with their type codes), which S-0011 parses but leaves outside its subset.
  S-0011's table of action codes should point here for them, or take them over in a later revision.
- **Citations, for sign-off.** The current glossary's claim G3 holds exception 2 and gives
  `quirks.speed_normalised_delays` the scope used here. S-0011, S-0202, S-0203 and S-0205 are
  approved specs cited by number and claim. The glossary's *departure* entry lists the pause
  action with *N* = 0, the paced period of index 0 and the real-time length of a timed freeze as
  departures of this spec.
