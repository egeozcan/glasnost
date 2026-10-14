---
id: S-0401
title: Ore and gems (resource cells, bales and value, growth, spread, ore mines)
topic: economy
revision: 7
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0046]
supersedes: []
---

# S-0401: Ore and gems (resource cells, bales and value, growth, spread, ore mines)

## Summary

Ore and gems lie on the map as resource cells: overlays of a tiberium (the resource type kind),
whose overlay data value is the cell's stage. This spec covers which tiberium a cell holds, how many
bales it holds and what they are worth, what taking bales out of a cell does, how cells grow denser
and spread into empty neighbours on per-tiberium timers with random choices, where new resource may
appear, how ore mines make new resource cells, and how all of this is ordered within a tick and
draws from the scenario random stream. The harvesting units that take the bales, and the credits a
house gets for them, are S-0402's.

**Joint hand-off.** At sign-off, check claim 20 and E17 against S-0109 claim 13 and
S-0011 claim 36 and T21: creation-time resource chain reaction is enabled, and only
later start events of a trigger-played animation suppress it. Hand off these aligned
revisions together. S-0202 claims 10 and 15 and R14 own the middle-event
draw composition during loading and play, including later playback events; subsequent
particle updates remain outside that established scope. Check claim 16c and E13 against
S-0202 claim 15 and R14: taking 6 bales leaves a stage-6 cell at stage 0, not empty.

## Terms

Glossary terms are used as defined there, in particular *tiberium* (the type kind of the resource
types; glossary *type kind*), *overlay*, *overlay data value*, *cell*, *legal cell*, *usable area*,
*step direction*, *slope*, *land type*, *tick*, *tick number*, *game start*, *session*, *rules
layer*, *map layer*, *scenario random stream*, *departure*, *importer warning*, *quirk switch*,
*recorder*, *measurement setup* and *round-toward-zero mode*. S-0205's notation applies: trunc,
RZ₅₃, *single* and *double*. S-0202's notation applies to draws: a *raw draw* is one value *r* of
the scenario random stream, a signed 32-bit integer, and *U*(*a*, *b*) is a ranged draw (S-0202,
claim 4), which can take more than one raw draw. In addition:

- **Resource cell.** A cell whose overlay type has the tiberium flag (S-0112, Terms, *tiberium
  overlay*). Every other cell holds no resource.
- **The tiberium of a cell.** For a resource cell, the tiberium of its overlay as S-0112 (Terms,
  *the tiberium of an overlay*) defines it: the first tiberium, in registration order, whose flat or
  sloped image run contains the overlay type's list position, or the first tiberium when none does.
  The same rule decides the tiberium for every economic purpose: value, growth, spread and what a
  harvester loads. This answers S-0004's open question 6.
- **Stage *d*.** The overlay data value of a resource cell, an integer from 0 to 255. Stage 0 is the
  sparsest. The *top stage* is 11 (Constants).
- **Bale.** The unit in which resource is held and harvested. A resource cell of stage *d* holds
  *d* + 1 bales (claim 2).
- **Value per bale *V*.** A tiberium's value field, an integer number of credits.
- **Growth step, spread step.** The two kinds of change that make resource denser or wider: a
  growth step raises one cell's stage by 1 (claim 7); a spread step makes one empty neighbouring
  cell a resource cell (claim 9).
- **Growth field *G*, spread field *S*.** A tiberium's growth and spread fields, integers. They set
  the periods of its growth and spread updates (Formulas F3 and F4).
- **Growth fraction *g*, spread fraction *s*.** A tiberium's growth-percentage and
  spread-percentage fields, decimals (S-0003, claims 6 and 7) held as doubles. They set how many
  cells an update may treat (Formulas F5 and F6). A fraction at most the double nearest 0.00001
  (written 10⁻⁵ below) stops the matching update; a fraction below it also stops the eligibility
  tests of claims 7 and 8. The two differ only for a fraction exactly equal to that double, which a
  plain decimal (a single widened to double) can never be and a percent form (S-0003, claim 7)
  could reach only by coincidence.
- **Growth list, spread list.** Each tiberium keeps two lists of pending cells, its growth list and
  its spread list. An entry names a cell and carries a key, a single-precision number. A spread list
  also marks the cells it holds; a cell is *marked* in it while the list considers it present (a
  growth list has no marks that any rule consults). Claims 6 to 9 and 13 say when entries are added,
  taken and dropped. Within a list, entries are taken smallest key first; entries with equal keys
  are taken in the order they were added (claim 14).
- **Entry count *E* of a list.** The number of entries added to a list since its last rebuilding
  (claim 13), counting the entries the rebuilding itself added. Taking an entry does not lower it.
  It decides when a list is rebuilt during play (Formulas F7).
- **Grid size *C*.** 2 × *W* × (*H* + 4) cells, where *W* and *H* are the map's width and height
  (S-0009, Import mapping, the map size). It appears only in the rebuilding rule (F7).
- **Taking an entry.** Removing the entry that comes first in a list's order (smallest key, then
  earliest added). Taking does not look at the key's size: an update takes entries whatever their
  keys are, even entries whose key lies in the future.
- **Resource settings.** Three on/off settings of a game: the *grows setting* and the *spreads
  setting*, both always on in a skirmish (claim 4), and the *map growth switch* (claim 4).
- **Germinable cell.** A cell on which new resource may appear (claim 10).
- **Ore mine.** A terrain object whose type has the spawns-resources flag (S-0203, claim 2, and
  S-0104, which calls it an ore-spawning terrain object). Its *animated flag*, *animation rate R* (ticks)
  and *animation probability p* (a single-precision number) are fields of its terrain type
  (Import mapping).
- **Turn.** In this spec "turn" means an object's share of one tick, in S-0203's sense (S-0203,
  Terms), as in "an ore mine takes turns", "the mine's turn in tick *t*₀" and "harvesters' turns".
  No rule here uses "turn" for an amount of rotation.

## Behaviour

### Resource cells, bales and value

1. **A resource cell holds the resource of one tiberium, at a stage.** The tiberium of a cell
   (Terms) is decided by the overlay type the map or a later change stored in the cell, and the
   stage is the cell's overlay data value. A tiberium that has never been given an image number
   (S-0112, claim 12) cannot appear on any new cell (claim 11), and a tiberium's stages for growth
   end at the top stage 11, whatever image number it has (S-0112, claim 10: every image number
   gives a run of 12 pictures). A map may store any stage from 0 to 255; a stage above 11 never
   grows (claim 7) but holds bales like any other (claim 2).
   - Backing: `measurable` (scenario E1)
   - Confidence: high

2. **Bales and value of a cell.** A resource cell of stage *d* holds *d* + 1 bales and is worth
   *V* × (*d* + 1) credits, where *V* is the value per bale of the cell's tiberium (Formulas F1). A
   cell that holds no resource is worth 0. This worth is the measure by which harvesters compare
   cells (S-0402); what a house is paid for a load is S-0402's.
   - Backing: `measurable` (scenario E2, through S-0402's field choice and payment)
   - Confidence: high

3. **Taking bales out of a cell, and the lost last bale.** When *n* ≥ 1 bales are taken from a
   resource cell of stage *d*:
   - if *n* ≤ *d*, the stage falls to *d* − *n* and all *n* bales are delivered,
   - if *n* ≥ *d* + 1, the cell is emptied (claim 5) and only *d* bales are delivered, one fewer
     than the cell held.

   Harvesters take one bale at a time (S-0402), so a harvester gets *d* bales from a cell of stage
   *d*: the cell's last bale (stage 0) yields nothing and empties the cell. Taking 0 or fewer bales,
   or taking from a cell without resource, changes nothing and delivers nothing.
   `quirks.last_bale_lost` (default: the original). With the switch off, emptying a cell delivers
   all *d* + 1 bales.
   - Backing: `measurable` (scenario E2)
   - Confidence: high

4. **Resource settings in a skirmish.** Three settings gate growth and spread:
   - The **grows setting** and the **spreads setting** are always on in a skirmish, whatever the
     map, the game-mode file or the rules data say. The rules' lobby default for resource growth and
     the map's own grows and spreads keys have no effect in a skirmish (Import mapping). The grows
     setting does not switch growth on or off: it only shortens the growth period (claim 6, Formulas
     F3). The spreads setting is needed by every spread step except an ore mine's (claims 8, 9 and
     15).
   - The **map growth switch** is the map's own growth key (Import mapping), on unless the map turns
     it off. While it is off, no tiberium runs growth or spread updates (claim 6), no cell grows
     (claim 7), and the rebuilding of the growth lists at game start leaves them empty (claim 13).
     A trigger action of the original can change it during play (action code 86, owned by the
     trigger specs; S-0011's subset does not include it).

   In a campaign mission of the original the map's grows and spreads keys do apply; campaigns are
   out of scope.
   - Backing: `measurable` (scenario E3)
   - Confidence: high for the skirmish values and for the effect of the grows setting on the
     period; medium for the map growth switch's effect on the lists at game start

5. **Emptying a cell.** After a cell is emptied, its overlay is absent, its stage is 0, its land type is recalculated (S-0211, claim 5), and it is unmarked in every tiberium's spread list. Consider its eight neighbours in step-direction order 0 through 7. Each legal neighbour not already marked in the emptied cell's tiberium's spread list is offered to that list. The offer outcomes, their draws, and the resulting order among equal keys follow that direction order (claim 14). A neighbour of another tiberium is still offered to the emptied cell's tiberium's list; if it later spreads, the new resource is of its own tiberium (claim 9).
   `quirks.spread_offer_by_emptied_type` (default: the original). With the switch off, each
   neighbour is offered to the spread list of its own tiberium.
   - Backing: `measurable` (scenario E5 for new resource growing back into emptied cells, E16 for
     the cross-tiberium case: two tiberiums with very different spread periods make the list that
     took the neighbour visible as the time at which new resource appears)
   - Confidence: high

### Growth and spread updates

6. **Per-tiberium timers, and the order within a tick.** Each tiberium has a growth timer and a
   spread timer. In any tick when the map growth switch is on and both update kinds occur, the resulting cell stages, list entries, and scenario-stream draws are those of all tiberiums' growth updates followed by all tiberiums' spread updates. Within each kind, the observable results and draws follow tiberium registration order. Thus each spread update reflects the stage changes and list entries produced by that tick's growth updates, and its draws follow the growth-update draws. This is row 9 of S-0203 (growth first),
   which comes after the polling of map triggers and before planted bombs. A tiberium's update
   runs in a tick when its timer has run out: when the tick number minus the tick of its last
   update is at least its period. After an update the timer
   restarts from that tick with the period then in force:
   - growth period *P*<sub>g</sub> = trunc(RZ₅₃(*G* × *k*)), with *k* = 0.3 while the grows setting
     is on and 1.0 while it is off (Formulas F3), so in a skirmish *k* = 0.3,
   - spread period *P*<sub>s</sub> = *S* (Formulas F4).

   A period of 0 or less makes the update run in every tick. A growth update does nothing when the
   tiberium's growth list is empty or its growth fraction is at most 10⁻⁵, and a spread update
   likewise with the spread list and the spread fraction; the timer restarts all the same. While
   the map growth switch is off the timers do not restart, so the first tick after the switch comes
   back on runs every update whose period has passed.
   - Backing: `measurable` (scenario E4 for the periods and their rounding, E3 for the map growth
     switch); the place in the tick relative to other drawing parts is S-0203's
     (claim 1 there and its scenarios)
   - Confidence: high

7. **Growth list and growth update.** A cell joins a tiberium's growth list when it is offered to
   it with a stage below 11; the offer adds an entry with key *t* + (|*r*| mod 50), where *t* is the
   tick number and *r* one raw draw (Formulas F2). An offer of a cell at stage 11 or above does
   nothing and takes no draw. A cell is offered to the growth list of its tiberium when it becomes a
   resource cell (claim 11) and at the game-start rebuilding (claim 13). Taking bales out of a cell
   never offers it (claim 18). The offer tests the cell's stage only: not its tiberium, slope or the
   growth fraction. A growth update also adds a re-entry for an entry it has taken (table below). A
   re-entry is not an offer, so no stage test other than the table's applies to it. A cell can be
   in the list twice (an offer for a cell whose earlier entry is still pending, for example after
   the cell was emptied and then received new resource).

   A growth update of a tiberium *T* with *n* ≥ 1 entries in its growth list takes one raw draw and
   works out *m* (Formulas F5): *m* is between 1 and 50 and grows with *n* × *g*. It takes up to *m*
   entries, one after another, fewer when the list is empty. Each entry taken has the result that
   the table gives for its cell at that moment. A cell of *T* gets a *growth step* when all of these
   hold: the map growth switch is on, the cell is flat (slope shape 0), its stage is below 11, *T*
   has been given an image number (S-0112, claim 12) and *T*'s growth fraction is at least 10⁻⁵. A
   growth step raises the stage by 1. A spread offer in a row judges the cell at its stage after
   that row's growth step, if any. A spread offer takes a draw only when it adds an entry (claim 8),
   and a new growth entry always takes one (F2).

   | Cell of the entry taken | Result |
   |---|---|
   | holds no resource, or its tiberium is not *T* | no change to the cell; no new entry (the entry is dropped) |
   | resource of *T* at stage *d*, gets a growth step, *d* + 1 below 11 | stage becomes *d* + 1; offered to *T*'s spread list; a new growth entry for the cell |
   | resource of *T* at stage 10, gets a growth step | stage becomes 11; offered to *T*'s spread list; no new growth entry |
   | resource of *T* below stage 11, no growth step | no change of stage; a new growth entry for the cell; then offered to *T*'s spread list |
   | resource of *T* at stage 11 or above | no change of stage; no new growth entry |

   The draws of a growth update therefore come in this order: the draw for *m*, then for each entry
   taken the draws of its row in the order the row lists them (the spread offer's draw only when it
   adds an entry, the new growth entry's draw only when the row has one).

   A new growth entry has a key of at least *t*, and taking does not look at keys, so a re-entry is
   taken again in the same update whenever it is first in the list's order: a lone cell grows by up to
   *m* stages in one update.

   The count *m* is below 1 only for the one raw value of `quirks.random_magnitude_overflow` (S-0202,
   Edge cases), whose remainder is negative (Formulas F2 and F5). Such an update makes the
   rebuilding test of Formulas F7 with that *m*, and its only other effect is that the entry first
   in the list's order (after any rebuilding) is removed without a result: no further draw, no
   growth step, no re-entry. With the switch off, *m* is always at least 1.
   - Backing: `measurable` (scenario E4 for the number of steps per update and the stage changes,
     the random-state recorder for the draw count per update; E10 for a lone cell taken again); the
     *m* < 1 case is `unmeasured`: it occurs for one raw value in 2³², too rare to provoke
   - Confidence: high; medium for the *m* < 1 case (inferred, never seen in play)

8. **Spread list.** A cell offered to a tiberium's spread list joins it, with key *t* + (|*r*| mod
   50) from one raw draw (Formulas F2), when it is not already marked in that list and all of these
   hold for the cell, judged with the cell's own tiberium: the spreads setting is on; the cell is a
   resource cell; its stage is above ⌊*i*/2⌋, where *i* is the registration index of its tiberium
   (claim 17); it is flat; the spread fraction of its tiberium is at least 10⁻⁵; and no object of
   any kind is listed in the cell's ground layer (S-0209, claim 13; for example infantry, a
   vehicle, a building or a terrain object; objects on a bridge above the cell do not count).
   Otherwise the offer does
   nothing and takes no draw. Cells are offered to the spread list when a growth update takes them
   (claim 7, the rows of its table that say so), when a neighbour is emptied (claim 5) and at the
   game-start rebuilding (claim 13).
   - Backing: `measurable` (scenario E5, and E6 for the stage test)
   - Confidence: high

9. **Spread update.** A spread update of a tiberium with *n* entries in its spread list takes one
   raw draw and works out *m* (Formulas F6): *m* is between 1 and 25 and grows with *n* × *s*. It
   then takes entries one after another until it has made *m* attempts or the list is empty. For a selected entry, let *c* be the number of its cell's eight germinable neighbours at selection, before any spread step. If *c* = 0, the cell becomes unmarked, the entry is discarded, and no attempt is used. If *c* = 1, one attempt is used and a spread step is tried; no replacement entry is made and the cell remains marked. If *c* ≥ 2, one attempt is used and a spread step is tried; the cell remains marked and can be selected again ahead of positive-key entries, subject to the equal-key ordering rule. In both attempted cases, the attempt is used even if the cell fails the spread conditions. With `quirks.spread_attempts_count_failures` off, a selected cell that fails those conditions is instead unmarked and discarded without using an attempt. With `quirks.spread_mark_kept_on_last_neighbour` off, a cell discarded with one germinable neighbour is unmarked.

   Because a cell put back with key 0 goes ahead of every entry with a positive key and, among key-0
   entries, behind those added earlier (claim 14), it is taken again in the same update when no
   other entry has a key of 0 or less: a single spreading cell can use several
   of the update's *m* attempts and spread into several neighbours in one update, until fewer than
   two germinable neighbours are left.

   **Spread step from a cell.** The cell spreads only when the conditions listed in claim 8 hold for
   it at that moment, judged with its own tiberium and not counting the mark (the cell is still
   marked when it is taken). If they do, one ranged draw *U*(0, 7) gives a starting step
   direction *j*, and the neighbours in step directions *j*, *j* + 1, …, *j* + 7 (modulo 8) are
   examined in that order; the first that is germinable becomes a new resource cell of the
   spreading cell's tiberium at stage 3 (claim 11). If the conditions do not hold, no draw is taken
   and nothing happens, but the attempt still counts.

   The count *m* is below 1 only for the one raw value of `quirks.random_magnitude_overflow`
   (S-0202, Edge cases). Such an update makes the rebuilding test of Formulas F7, and its only
   other effect is that the entry first in the list's order (after any rebuilding) is removed
   without a result: no attempt, no further draw, and its cell keeps its mark. With the switch off,
   *m* is always at least 1. This case is `unmeasured` (one raw value in 2³²) and of medium confidence.

   Two consequences are behaviour of the original, each with a switch:
   - An entry whose cell can no longer spread (it was harvested empty, a unit stands on it, its
     stage fell too low) but that still has two or more germinable neighbours keeps returning to
     the front of the list with key 0 and uses up one attempt in every spread update.
     `quirks.spread_attempts_count_failures` (default: the original). With the switch off, an entry
     whose cell fails the conditions listed in claim 8 (not counting the mark) is unmarked and
     dropped and counts no attempt.
   - A cell dropped with *c* = 1 stays marked, so no later offer can bring it back into that spread
     list (claim 8 refuses marked cells); it can spread again only after a rebuilding of that
     list (claim 13: at the next game start, or during play under F7).
     `quirks.spread_mark_kept_on_last_neighbour` (default: the original). With the switch off, the
     cell is unmarked when it is dropped.
   - Backing: `measurable` (scenario E5 for the spread step and the new stage, E7 for the two
     consequences); the choice among several germinable neighbours is `measurable` too (E5, with
     the random-state recorder: one ranged draw over 0 to 7 per successful attempt, and the new cell
     is the first germinable neighbour from that direction)
   - Confidence: high for the rules; medium for the consequences' size in normal play

10. **Where new resource may appear (germinable cells).** A cell is germinable when all of these
    hold:
    - it is in the usable area, by the height-aware form of the test (S-0009, claims 4 and 5),
    - it is not part of a bridge (neither a high bridge's cell nor any other bridge part),
    - it holds no living building, other than one of a type that is invisible or invisible in
      game; only the first building among the objects of the cell's ground layer is looked at,
    - its first terrain object, if any, is not an ore mine (other terrain objects, such as trees,
      do not prevent it),
    - its land type is buildable (the buildable flag of S-0211, claim 2, and S-0010, claim 16),
    - it holds no overlay,
    - it is flat (slope shape 0),
    - its tile's tileset has the allow-tiberium flag, or its tile number lies outside the theater's
      tiles (S-0010, claim 16).

    Infantry, vehicles and aircraft in the cell do not prevent it. The building and terrain-object
    conditions apply only while a game is running; before play starts (while a map is being
    loaded) they are not checked. S-0010 claim 16 states the last four conditions; this claim adds
    the others.
    - Backing: `measurable` (scenario E8)
    - Confidence: high; medium for "only the first building and the first terrain object are
      looked at", which matters only when a cell holds two of them

11. **A new resource cell.** When a germinable cell becomes a resource cell of tiberium *T* at
    stage *a* (a spread step gives *a* = 3, claim 9, and so does an ore mine, claim 15):
    - its overlay type is chosen in *T*'s image run (S-0112, Terms) by one ranged draw: on a flat
      cell *U*(0, 11), the type at position *P* + the draw; on a sloped cell of slope shape σ (1 to
      4), *U*(0, 1), the type at position *P* + *F* + 2(σ − 1) + the draw (*P*, *F* as in S-0112).
      Germinable cells are flat (claim 10), so in play only the first form occurs. What is drawn
      for the cell is S-0112's (claim 4), and it does not depend on the stored type,
    - the cell is offered to *T*'s growth list (claim 7), which takes one raw draw when it adds an
      entry (see Open questions for the stage this offer sees),
    - its stage becomes *a*.

    The draws come in that order: the overlay draw, then the growth-list draw. No cell becomes a
    resource cell when *a* is 12 or more (the run's 12 pictures), and a tiberium that has never
    been given an image number has a run of no pictures, so it never makes a new cell (S-0112,
    claim 12).

    A new resource cell is **not** offered to any spread list. It joins a spread list only when it
    later gets a growth step (claim 7), when a neighbour is emptied (claim 5) or at a rebuilding
    (claim 13). So, as long as none of these happens, a cell made by a spread step or an ore mine
    does not spread itself. With a growth fraction below 10⁻⁵ new cells never get a growth step, so
    they spread only after a neighbour is emptied or a rebuilding (as in scenario E5); with the map
    growth switch off no spread update runs (claim 6), so they never spread while it stays off.
    - Backing: `measurable` (scenario E5 for the stage and for new cells not spreading, the
      random-state recorder for the two draws)
    - Confidence: high

12. **Sloped resource never grows or spreads.** Resource on a cell of slope shape 1 to 4 (the only
    slopes that keep resource, S-0211 claim 7) never gets a growth step (claim 7), never spreads
    (claim 8) and never receives new resource (claim 10). It can be harvested like flat resource.
    - Backing: `measurable` (scenario E6)
    - Confidence: high

13. **Rebuilding the lists.** At every game start each tiberium's growth list and spread list are
    rebuilt, and during play one list is rebuilt under the rule of Formulas F7. A rebuilt list
    holds nothing it held before; what it holds afterwards depends only on the cells at that moment:
    - the growth list holds one entry for each resource cell of that tiberium that is flat and below
      stage 11, when the map growth switch is on and the growth fraction is at least 10⁻⁵, and no
      entry otherwise,
    - the spread list holds one entry for each resource cell of that tiberium for which the
      conditions of claim 8 hold, and exactly those cells are marked in it,
    - every entry has key 0, and the entries stand in the visiting order of the legal cells
      (S-0009, claim 3, including its narrow-map case), so that by claim 14 they are taken in that
      order,
    - the entry count *E* equals the number of entries the rebuilding made.

    A rebuilding takes no draw. The game-start rebuilding judges the cells with the map's overlays
    and terrain objects in place but none of its vehicles, aircraft, infantry or buildings, so the
    only objects it sees in cells are terrain objects: a cell under one of the map's units or
    buildings still joins the spread list, and while the object stays it is treated like any entry
    whose cell cannot spread (claim 9). Because *E* counts every entry ever added since the last
    rebuilding, including the entries that updates put back (claims 7 and 9), a list that stays
    short can still be rebuilt during a long game. A rebuilding during play gives every eligible cell key 0 and restores cells that the
    original had left out (claims 9 and 18).
    - Backing: `measurable` (scenario E4: growth starts on map ore at the first growth update,
      and scenario E12 for a rebuilding during play)
    - Confidence: high for the game-start rebuilding and its conditions; medium for the rule's
      frequency in normal play

14. **Order among equal keys.** Entries with equal keys are common: every entry made at the
    game-start rebuilding has key 0, entries offered in the same tick get keys from the same 50
    values, and claim 9 puts entries back with key 0. In a list, entries with equal keys are taken
    in the order they were added: the entry added first comes first. This is a **departure** (no
    switch) from the original, by the project lead's ruling on pathfinding ties (S-0212, D7).
    - What the original does instead: the order among equal keys depends on the
      whole history of adds and takes. The observable difference is easiest to see in one case.
      Entries 1 to *N* added to an empty list in that order with equal keys, then *m* taken in one
      update, each taken entry being re-entered with a larger key: the original takes entry 1,
      then entry *N*, then entry 2 (for *m* up to 3, when no re-entry key is 0), where this spec
      takes entries 1, 2, 3.
    - Which cells grow or spread when keys tie therefore differs from the original, and through the
      draws of later entries so does everything after. The counts per update, the periods, the
      conditions and the draw counts per update stay exact.
    - Backing: `measurable` (scenario E15 measures the original's choice, so the departure's size can
      be stated)
    - Confidence: high for the rule of this spec (it is a definition); medium for the description of
      the original

15. **Ore mines.** An ore mine takes turns (S-0203, claim 2). A mine whose type lacks the animated
    flag never draws and never makes resource. A mine whose type has it is, in each of its turns,
    either *idle* or *animating*. *R* is its animation rate and ⌊*N*/2⌋ the number of body image
    frames of its sprite (for a sprite of *N* image frames, S-0112 Terms).

    | The mine's turn | Result |
    |---|---|
    | idle | one raw draw *r*; an animation starts when (\|*r*\| mod 1000000) × 10⁻⁶ < *p* (Formulas F8). With *R* = 0 a started animation never advances and the mine stays idle, so it draws in every turn and never makes resource |
    | animating (*R* ≠ 0), animation started in the turn in tick *t*₀ | no draw; the mine shows image frame *k* from its turn in tick *t*₀ + *kR* |
    | animating, the turn in tick *t*₀ + *R* × ⌊*N*/2⌋, when ⌊*N*/2⌋ ≥ 1 | the animation stops, the mine is idle from its next turn, and it makes a spread step from its own cell (below) |

    A mine whose sprite has fewer than 2 image frames (⌊*N*/2⌋ = 0) never makes resource: once
    started, its animation never stops (its image frame only grows past 0).

    The mine's spread step differs from claim 9 in two ways. None of the conditions of claim 8 is
    checked, so the spreads setting does not matter, and neither does the map growth switch. And
    the tiberium is that of the mine's cell, or the first tiberium in registration order when the
    cell holds no resource, which is the normal case because the mine's own cell is never
    germinable (claim 10). So every ore mine makes resource of the first tiberium in registration
    order, at stage 3, in a germinable neighbour, unless a map put resource of another tiberium
    under the mine. The ranged draw *U*(0, 7) of the spread step is taken even when no neighbour is
    germinable; nothing appears then.
    - Backing: `measurable` (scenario E9)
    - Confidence: high for the spread step and the tiberium; medium for the exact tick
      *t*₀ + *R* × ⌊*N*/2⌋, which assumes the mine takes a turn in every tick

16. **Other ways resource is removed.** Besides harvesting (claim 3), resource leaves cells through
    the following, and no other path is known. Each removal that takes a number of bales follows the
    taking rule of claim 3, and each that empties a cell does what claim 5 says.
    - (a) **Detonations** (S-0303, claim 17): the resource-flag and chain-reaction rule of area
      damage.
    - (b) **The start event of an animation** (S-0109, claim 13): an animation whose type has the
      chain-reaction flag takes every bale of the cell it is in, so a resource cell there is emptied.
      Claim 20 gives the accompanying effects and draws, including during map loading.
    - (c) **The middle event of an animation** (S-0109, claim 14; the craters of its claim 18):
      when the animation's height above the terrain is below 30 leptons, and its type has the
      crater flag and either no scorch flag or the scorch flag with a draw that does not choose the
      scorch mark (S-0109 gives the draw), 6 bales are taken from the cell the animation is in.
      This is independent of whether a crater is then drawn or created. Under claim 3, a cell
      at stage 6 or above retains its resource overlay at stage *d* − 6; a cell at stage 0
      through 5 is emptied. In particular, stage 6 becomes stage 0, not empty. These integer
      stage changes involve no rounding. A type with the crater flag and the large-crater
      flag takes the same 6 bales.
    - (d) **Steep slopes** (S-0211, claim 7): a cell whose slope no longer allows resource loses it
      when its land type is worked out again.
    - (e) **Laser-fence lines** (S-0202, Terms, *fence line*): when a line of fence sections is
      built between two matching posts, every cell strictly between the two posts is cleared of
      resource, a cell holding 12 bales or fewer being emptied, one holding more losing 12 bales.
      The lines are built at game start for posts on the map and when a post is put into play.
    - (f) **The trigger action that thins resource around a waypoint** (map triggers, action code
      59; not in S-0011's subset). The original takes one bale, 109 times in a row, from the
      waypoint's own cell: once for every displacement (*dx*, *dy*) with |*dx*|, |*dy*| ≤ 5 and
      *dx*² + *dy*² ≤ 35 (the displacements whose Euclidean length, truncated to a whole number, is at
      most 5). The displacement only decides how many takes there are: every take hits the waypoint's
      cell, none hits a neighbour. A cell of stage *d* therefore ends emptied when *d* ≤ 108 and at
      stage *d* − 109 otherwise; a waypoint cell without resource, and every other cell, is not
      touched. It can occur on a skirmish map that carries triggers. `quirks.thinning_hits_waypoint_only`
      (default: the original). With the switch off, each cell of the disc of 109 displacements around the
      waypoint cell loses one bale, as the action's purpose suggests.

    Building placement is not a removal. S-0406 (claim 3, part c) refuses to place a building on
    any cell that holds an overlay, so a resource cell never takes a placed building. A building
    that the map file itself puts on a resource cell takes no resource away: the cell keeps its
    resource, under the building: it can still grow (claim 7 does not look at buildings), cannot spread
    while the building stands (claim 8: no object may be in the cell) and cannot receive new resource
    (it holds an overlay, claim 10). Low confidence for "nothing removes the resource": no removal
    of it is known.
    - Backing: `measurable` through the owning specs' scenarios for parts a, b and d: S-0303
      scenario A10 (detonations), S-0109 scenario AN21 (the chain reaction, which is claim 13's
      start event), S-0211 scenario L3 (steep slopes); part c through scenario E13 below, which S-0109
      does not have (its scenario AN10 uses a scorch flag with no crater flag); parts e and f
      through scenario E14
    - Confidence: high for part b; medium for parts a, c and d (the exact scorch-or-crater draw
      belongs to S-0109); medium for part f (the odd rule that every take hits the waypoint's own
      cell is unexpected, so scenario E14 part B is the check) and low for part e. That the list
      is complete is inferred, not measured; other overlay removals (a bridge, a wall) are not
      resource removals

17. **The stage needed to spread depends on the tiberium's index.** The spread condition of claim
    8 compares the stage with ⌊*i*/2⌋ for the registration index *i* of the cell's tiberium, not
    with any field of the tiberium. So tiberiums at indices 0 and 1 need stage 1 or more, those at indices 2
    and 3 stage 2 or more, and a tiberium at index 10 stage 6. `quirks.spread_stage_from_index` (default: the original). With the switch off, the
    condition is a stage above ⌊11/2⌋ = 5 for every tiberium (half the top stage).
    - Backing: `measurable` (scenario E6)
    - Confidence: high for the comparison; the "fixed" threshold is a proposal

18. **A harvested full cell never grows again.** Taking bales out of a cell never adds a growth
    entry for it, whatever its stage, and so takes no draw for a growth list.
    A cell at stage 11 is in no growth list (it is dropped when it reaches stage 11 and left out at
    the rebuilding; the rare cell in a list twice, claim 7, can still have its other entry pending),
    so after such a harvest it gets no further growth step, apart from one through such a pending
    entry, until the next rebuilding of its tiberium's growth list (claim 13), which puts it back with key 0; a rebuilding
    during play needs the entry count of F7, so in a short game or on a large map the cell may never
    grow again. This holds for every take of claims 3 and 16. Until then, ore regrows in a harvested
    field only through cells that were below stage 11 when they joined a growth list and through
    spread into emptied cells. `quirks.full_cell_no_regrowth` (default: the original). With the
    switch off, a take that leaves a cell which was at stage 11 as a resource cell offers it to the
    growth list once its stage has fallen (claim 7: one draw), so it grows again.
    - Backing: `measurable` (scenario E10)
    - Confidence: high

19. **Timers carry over between games in one session.** The timers of claim 6 belong to the
    tiberiums, which carry over between the games of a session (S-0006, claim 10), while the tick
    number starts again at 0 in every game. In the first game of a session the first growth and
    spread updates run in the first tick (tick number 0). In a later game of the same session a
    tiberium's first update runs only once the tick number reaches the tick of its last update in
    the earlier game plus the period then in force. `quirks.resource_timers_carry_over` (default:
    the original). With the switch off, every tiberium's timers restart at each game start, so that
    its first updates run in the first tick. This is a separate switch from S-0006's
    `quirks.persist_tiberium_overrides`, which concerns the tiberiums' fields.
    - Backing: `measurable` (scenario E11)
    - Confidence: medium (that nothing restarts the timers at game start is inferred, not observed; the
      first game's tick-0 update depends on the tick number when the tiberiums are created)

20. **Animation start-event resource chain reaction (Q-0046 Q2a).** The chain reaction is
    enabled by the animation type's resource-chain-reaction flag. For a trigger-played animation,
    it is suppressed at later start events, but not at the creation-time start event
    (S-0011 claim 36; S-0109 claim 13). It acts on the cell containing the animation's
    position, not necessarily the cell whose overlay created the animation (S-0009 claim 12).
    The following applies during map loading as well as during play.

    When that cell holds no resource at the time of the chain reaction, there is no resource
    change, debris trial, debris selection or detonation, and the chain reaction takes **zero
    draws**. This does not suppress the animation's other creation or middle-event draws.
    When a start event also produces a middle event because μ = 0, that middle event's effects
    and draws precede the chain reaction. Thus a crater that has already emptied the cell can
    make the chain reaction take no draws (S-0109 claim 14 and claim 16c here).

    When the cell holds resource of tiberium *T*, it is emptied as in claim 5. The draws of any
    neighbour offers that add spread entries occur in step-direction order 0 to 7, before the
    debris trial. There is no growth-list draw for the emptied cell (claim 18).
    With *L* the length of *T*'s resource-debris animation list:
    - *L* = 0: no debris trial, selection or debris-animation creation, hence no draws for
      any of those. Emptying still makes its eligible neighbour offers, and the detonation
      still occurs. Thus an empty debris list does not imply zero total chain-reaction draws.
    - *L* > 0: one raw scenario-stream draw *r* tests whether |*r*| mod 3 = 0. The arithmetic
      is signed 32-bit integer arithmetic, with remainder after division truncated toward zero,
      as in F2 and S-0202's `quirks.random_magnitude_overflow`. For *r* = −2³¹ the remainder
      is −2 with that switch on and 2 with it off; neither passes the trial.
    - A passing trial takes one ranged scenario-stream draw *U*(0, *L* − 1), choosing that
      list entry, then creates one animation of the chosen type at the source animation's
      position with world height raised by 10 leptons. Its creation has no loop delay, repeat
      factor 1 and no request for reversed play. Its creation draws and immediate start-event
      effects are those of S-0202 claim 10 and S-0109; they come after the selection and before
      the detonation below. A singleton list's selection takes no raw draw (S-0202 claim 4).
      A failed trial takes no selection draw and creates nothing.

    Whether or not debris was created, the emptied-cell case produces a detonation at the
    source animation's position with the game-wide resource-explosion damage, the demolition
    warhead, no source object and no source house. This detonation does **not** allow the
    scanned-cell resource removal of S-0303 claim 17, so it makes no neighbour spread-offer
    draws through that rule. S-0303 owns its other effects and conditional draws. There is
    no separate chance draw for that detonation and no projectile explosion-animation selection
    (S-0303 claim 21 belongs to projectiles).

    After the detonation's effects, the cell containing the source animation's position
    undergoes a terrain recalculation (S-0010 claims 8 and 19; S-0211 claim 5), even when
    the detonation changes no terrain. Its land type and slope reflect the contents then
    present. Any tile animation newly started by this recalculation takes its creation and
    immediate start-event draws after the detonation's applicable draws. A cell that has
    already started a tile animation since the second load pass starts no second one under
    S-0010 claim 19's once-only rule, so this recalculation adds no animation-creation draws
    there. This post-detonation recalculation is absent when the reaction is suppressed or
    finds no resource. It is separate from the recalculation caused by emptying the cell
    before the debris trial (claim 5).

    In observable scenario-stream order, any draws for eligible neighbour offers precede the optional debris trial; a passing trial is followed by a selection draw only when the debris list has more than one entry, then by any draws caused by creating and starting the selected debris animation; applicable detonation draws follow, then any tile-animation draws from the post-detonation recalculation. This is not a fixed raw-draw count: ranged rejection and downstream effects can add draws. A failed debris trial skips selection and debris-animation creation, not the later detonation. Suppression at a later start event for a trigger-played animation skips the whole chain reaction, including its emptying, neighbour offers, debris trial, detonation and post-detonation recalculation, with zero draws from the reaction itself. Its creation-time start event is not suppressed and follows the normal chain-reaction rules above. This specifies observable stream order, not an internal call structure.
    - Backing: `measurable` (scenario E17)
    - Confidence: high for the no-resource gate, creation/later suppression distinction and direct draw conditions; medium for the
      complete composed sequence during map loading until E17 is run

## Formulas

**F1. Worth of a cell.** worth = *V* × (*d* + 1) for a resource cell, 0 otherwise. Integers, exact.
Unit: credits. Bales held = *d* + 1; bales delivered when *n* bales are taken = *n* when *n* ≤ *d*,
else *d* (claim 3).

**F2. Key of a new entry.** key = *t* + (|*r*| mod 50), where *t* is the tick number and *r* the raw
draw. |*r*| is the absolute value of the signed 32-bit integer, and mod is the remainder of the
division truncated toward zero, so for the one value *r* = −2³¹, whose absolute value does not fit,
the remainder is −48 (S-0202's |*r*| mod *k* form). The re-entry that a growth update makes for a
cell it has just taken (claim 7) instead takes the absolute value of the remainder,
|*r* mod 50|, which equals |*r*| mod 50 except for *r* = −2³¹, where it gives 48. The key is stored in single precision; it is
exact while *t* + 49 < 2²⁴, and beyond that the conversion to single follows S-0205. Unit: ticks (used only to order entries).

**F3. Growth period.** *P*<sub>g</sub> = trunc(RZ₅₃(*G* × *k*)), *k* the double nearest 0.3 while
the grows setting is on, 1.0 while it is off. The product is a double product rounded under the
round-toward-zero mode (S-0205, claim 5; `quirks.chop_rounding`). Because the double nearest 0.3 is
slightly below 0.3, the original gives one tick less than the exact product whenever *G* × 0.3 is a
whole number: for an invented field *G* = 1000, *P*<sub>g</sub> = 299, not 300. Unit: ticks.

**F4. Spread period.** *P*<sub>s</sub> = *S*. Integer, ticks.

**F5. Growth count.** *k*<sub>g</sub> = trunc(RZ₅₃(*n* × *g*)), clamped to 5 … 50 (a result below 5
gives 5, above 50 gives 50); *m* = (|*r*| mod *k*<sub>g</sub>) + 1. *n* is the number of entries in
the growth list (counting any cell twice that is in it twice), *g* the growth fraction, a double
(from a given key, a single value widened to double, S-0003; when the key is absent, the double
nearest 0.1, Import mapping), *r* one raw draw. *m* is 1 … *k*<sub>g</sub> (so at most 50),
except for the one raw value *r* = −2³¹ under `quirks.random_magnitude_overflow`: the remainder is
−(2³¹ mod *k*<sub>g</sub>), so *m* = 1 − (2³¹ mod *k*<sub>g</sub>), which is below 1 unless *k*<sub>g</sub> is
8, 16 or 32 (claim 7 says what the update does then). With the switch off, *m* = (2³¹ mod
*k*<sub>g</sub>) + 1. Unit: entries taken.

**F6. Spread count.** *k*<sub>s</sub> = trunc(RZ₅₃(*n* × *s*)), clamped to 5 … 25; *m* = (|*r*| mod
*k*<sub>s</sub>) + 1, as in F5 with the spread list and fraction. *m* is 1 … *k*<sub>s</sub> (so at most 25), with the same
exception for *r* = −2³¹ (claim 9). Unit: attempts.

**F7. Rebuild during play.** With *C* = 2 × *W* × (*H* + 4) the grid size (Terms), *E* a list's
entry count and *n* the number of entries now in it:
- an offer to the growth list that will add an entry (a cell below stage 11) rebuilds the list
  first when *E* > *C* − 10; a growth update rebuilds it, after its draw for *m* and before taking
  entries, when *n* > *C* − 2*m*,
- an offer to the spread list that will add an entry (claim 8's conditions hold and the cell is not
  marked) rebuilds it first when *E* ≥ *C* − 20; a spread update rebuilds it, after its draw for
  *m*, when *n* > *C* − 20,
- the entries that updates put back (growth re-entries, claim 7; spread entries with key 0,
  claim 9) add 1 to *E* each without any test.

Integers, exact. A rebuild takes no draw. The new entry of the offer that caused a rebuild is
added after it, with its draw. In the original, an entry count *E* above *C* gives an undefined
result; reaching it needs about *C* entries put back by updates with no checked offer in between.
Departure: our engine has no such limit and applies only the tests above.

**F8. Ore mine start.** start when (|*r*| mod 1000000) × 10⁻⁶ < *p*, the product a double product of
an integer and the double nearest 10⁻⁶, rounded under the round-toward-zero mode (S-0205, claim 5, and
`quirks.chop_rounding`), compared with *p* widened from single. For *r* = −2³¹ under
`quirks.random_magnitude_overflow` the remainder is −483648, the product is negative and the
animation starts for every *p* ≥ 0 (with the switch off the remainder is 483648). Unit: per turn.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Top stage | 11 | stage | claims 1, 7, 18; hard-coded, not taken from the image run |
| Pictures per image run | 12 | image frames | S-0112 claim 10; a stage reached by germination must be below it (claim 11) |
| Stage of a new resource cell from a spread step or an ore mine | 3 | stage | claims 9, 11, 15 |
| Growth step | 1 | stage | claim 7 |
| Growth period factor, grows setting on | the double nearest 0.3 | factor | F3 |
| Growth period factor, grows setting off | 1.0 | factor | F3 |
| Key spread | 0 to 49 | ticks | F2 |
| Growth count bounds | 5 to 50 | entries | F5 (before the draw) |
| Spread count bounds | 5 to 25 | attempts | F6 (before the draw) |
| Fraction threshold | the double nearest 10⁻⁵ | fraction | claims 6 to 8 |
| Ore mine draw modulus | 1000000 | raw-draw units | F8 |
| Ore mine draw scale | the double nearest 10⁻⁶ | probability per raw-draw unit | F8 |
| Rebuild margins | 10, 2*m*, 20 | entries | F7 |
| Grid size | 2 × *W* × (*H* + 4) | cells | F7; *W*, *H* the map size |
| Spread stage divisor | 2 | registration indices per stage | claim 17 |
| Bales a crater takes | 6 | bales | claim 16c |
| Crater height limit | 30 | leptons | claim 16c |
| Bales a laser-fence line takes per cell | 12 | bales | claim 16e |
| Thinning action: takes | 109 (displacements with *dx*² + *dy*² ≤ 35, each of *dx*, *dy* at most 5 in absolute value) | takes of 1 bale | claim 16f |

## State and ordering

**Resource cell states** (one cell, one tiberium):

| State | Event | Result |
|---|---|---|
| empty, germinable | spread step from a neighbour, or ore mine | resource cell at stage 3; one overlay draw; offered to the growth list (one draw when it joins, claim 11) |
| resource, stage *d* < 11, in growth list | taken in a growth update, switch on, flat | stage *d* + 1; offered to spread list; if still below 11, new growth entry (one draw) |
| resource, stage 11 | taken in a growth update | dropped from growth list |
| resource, stage 11 | *n* bales taken | no growth entry (claim 18); stage 11 − *n* or emptied |
| resource, stage *d* | *n* ≤ *d* bales taken | stage *d* − *n*; *n* delivered |
| resource, stage *d* | *n* > *d* bales taken | emptied (claim 5); *d* delivered |
| resource on slope shape 1 to 4 | any growth or spread event | no change (claim 12) |

**Within one tick** (S-0203 row 9): every growth update of the tick takes effect before any of its
spread updates, each kind in the registration order of the tiberiums (claim 6), so a spread update
sees what the tick's growth updates did and its draws come after theirs; each update whose timer
has run out takes its draws in the order of claims 7 and 9. Ore mines draw and spread in their own turns (S-0203 row 5, the turn pass),
later in the tick. Harvesting, which takes bales, happens in harvesters' turns (S-0402).

## Edge cases and quirks

- `quirks.last_bale_lost` (claim 3): the last bale of a cell yields nothing.
- `quirks.spread_offer_by_emptied_type` (claim 5): neighbours of an emptied cell join the emptied
  cell's tiberium's spread list.
- `quirks.spread_attempts_count_failures` (claim 9): failed attempts count and return to the front.
- `quirks.spread_mark_kept_on_last_neighbour` (claim 9): a cell dropped with one free neighbour can
  rejoin only through a rebuilding.
- `quirks.spread_stage_from_index` (claim 17): the spread stage depends on the tiberium's index.
- `quirks.full_cell_no_regrowth` (claim 18): harvesting a full cell takes it out of growth until a
  rebuilding.
- `quirks.resource_timers_carry_over` (claim 19): timers carry over between games of a session.
- `quirks.chop_rounding` (S-0205) covers F3, F5, F6 and F8; no switch of this spec.
- `quirks.random_magnitude_overflow` (S-0202) covers F2, F5, F6 and F8 for the one raw value
  −2³¹ (claims 7 and 9 say what an update with *m* below 1 does).
- `quirks.thinning_hits_waypoint_only` (claim 16, part f): the waypoint-thinning action takes all
  109 bales from the waypoint's own cell.
- **The grows setting only changes the period** (claims 4 and 6). In a skirmish it is always on, so
  this has no effect there; it is stated so that the original's campaign behaviour (growth at a
  third of the speed, not off, when a map turns the setting off) is not mistaken for a skirmish
  rule.
- **A tiberium of index 4 or more.** S-0402 (claim 15) gives what a harvester's load does with it.
- **A stage above 11 from map data** never grows, is drawn as S-0112 (claim 9) says, and yields *d*
  bales to a harvester.
- **A cell in a growth list twice** (claim 7) is grown once per entry taken; this needs two offers
  of the same cell before it is taken and is rare.
- **Entry count beyond the grid size** (F7): undefined in the original; departure, no limit in our
  engine.
- **Equal keys** (claim 14): departure, no switch. Entries with equal keys are taken in the order
  they were added; the original's order depends on the whole history of adds and takes.
- **A cell taken again in the same spread update** (claim 9) follows from the key-0 re-entry; it is
  not a separate rule and has no switch of its own (`quirks.spread_attempts_count_failures` and
  `quirks.spread_mark_kept_on_last_neighbour` do not change it).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, `[Tiberiums]` list | the tiberiums and their registration order (S-0004, claim 10) | integer key = name | S-0004 |
| Rules, tiberium section, `Value` | value per bale *V* (claims 2, 3) | integer | 0 |
| Rules, tiberium section, `Growth` | growth field *G* (F3) | integer | 0 |
| Rules, tiberium section, `GrowthPercentage` | growth fraction *g* (F5) | decimal (S-0003, claims 6 and 7), a single widened to double | the double nearest 0.1 |
| Rules, tiberium section, `Spread` | spread field *S* (F4) | integer | 0 |
| Rules, tiberium section, `SpreadPercentage` | spread fraction *s* (F6) | decimal, as above | the double nearest 0.1 |
| Rules, tiberium section, `Image` | image number (S-0112, claim 10) | integer | owned by S-0112 |
| Rules, tiberium section, `Debris` | resource-debris animation list (claim 20) | ordered list of animation names | empty list |
| Rules, tiberium section, `Power`, `Color` | not used by the resource amounts or direct draws in this spec | — | — |
| Art, animation section, `TiberiumChainReaction` | resource-chain-reaction flag (claim 20; S-0109) | boolean | no |
| Rules, `[CombatDamage]`, `TiberiumExplosionDamage` | resource-explosion damage (claim 20) | integer, damage units | 100 |
| Rules, `[CombatDamage]`, `C4Warhead` | demolition warhead (claim 20; S-0303) | warhead name | none |
| Rules, overlay section, `Tiberium` | tiberium flag of an overlay type (Terms, *resource cell*) | boolean | no |
| Map, `[Basic]`, `TiberiumGrowthEnabled` | map growth switch of claim 4 | boolean | yes; a layer without the key keeps the value in force |
| Map, `[SpecialFlags]`, `TiberiumGrows`, `TiberiumSpreads` | grows and spreads settings in a campaign mission only; ignored in a skirmish (claim 4) | boolean | — |
| Rules, `[MultiplayerDialogSettings]`, `TiberiumGrows` | nothing: read, but never used in a skirmish (claim 4) | boolean | — |
| Rules, `[General]`, `GrowthRate` | nothing: read, but never used (growth timing comes from each tiberium's growth field, F3) | decimal | — |
| Rules, `[General]`, `TiberiumGrows`, `TiberiumSpreads` | nothing: never read | boolean | — |
| Rules, terrain type section, `SpawnsTiberium` | ore mine (claim 15; S-0203 claim 2) | boolean | no |
| Rules, terrain type section, `IsAnimated` | animated flag of an ore mine (claim 15) | boolean | no |
| Rules, terrain type section, `AnimationRate` | animation rate *R* (claim 15) | integer, ticks | 0 |
| Rules, terrain type section, `AnimationProbability` | animation probability *p* (F8) | decimal, single | 0 |
| Theater data, tileset `AllowTiberium` | allow-tiberium flag (claim 10; S-0010, claim 16) | boolean | no (S-0010) |
| Rules, land-type sections, `Buildable` | buildable flag (claim 10; S-0211) | boolean | no (S-0211) |

The absent-key defaults of the tiberium keys are the values a tiberium has when it is created; for a
tiberium that already exists, a layer without the key keeps the value in force (S-0006, claim 10).
The terrain-type defaults are those of a newly created terrain type; the rules for terrain-type
sections in later layers are S-0006's. Not all of the absent-key defaults of the terrain-type keys
were traced; a scenario that relies on one sets the key.

## Measurement scenarios

All scenarios run under the measurement setup on purpose-made skirmish maps with one human house and
one bystander computer player (observed area: the cells a scenario sets up or observes, with their
neighbours); named test data is a map's override sections unless a scenario says
otherwise. Cells and stages are read from the picture (S-0112: the image frame is the stage) or from
a recorder that reads cell contents; the tick number from a recorder. Each scenario starts a fresh
program unless it says otherwise (claim 19). Values below are invented test values, not stock data.

**E1. The tiberium of a cell and the top stage (claim 1).** *Setup:* override two tiberiums with
different value fields; a map whose overlays use an overlay type from the second tiberium's run
on some cells and a stage of 20 on one cell of the first tiberium; growth fraction 1.0, growth field
30. *Observable:* after 600 ticks, the stages of all cells; the credits a harvester delivers per bale
from each kind (S-0402 payment). *Expected:* cells of both runs grow up to stage 11 and stop; the
stage-20 cell stays 20; the payment per bale matches each run's tiberium. *Falsified if:* a cell
passes 11, the stage-20 cell changes, or payments follow the stored type instead of the run.

**E2. Bales, worth and the last bale (claims 2, 3).** *Setup:* one isolated resource cell of
stage *d* = 4 (growth and spread fractions 0), a harvester of the human house with capacity of at
least 10 bales next to it, a refinery. *Procedure:* order the harvester to harvest that cell, let it
unload. *Observable:* credits gained (S-0402, Formulas) and the bales loaded (pips or recorder).
*Expected:* 4 bales, worth 4 × *V* × the house's income factor; the cell is empty afterwards. With
*d* = 0: no bale, the cell is emptied. *Falsified if:* 5 bales (all of *d* + 1), or the cell not
emptied.

**E3. Settings (claims 4, 6).** *Setup:* maps of at least 40 by 40 cells, with resource fields
small enough that no list is rebuilt during play (F7; in part C every new cell adds to the growth
list's entry count). *Part A:* a skirmish map whose special-flags section turns grows and
spreads off. *Expected:* growth and spread proceed as on a control map without those keys (same
periods). *Part B:* the map growth switch off, no ore mine on the map. *Expected:* no stage change
of any cell and no new cell for 3000 ticks. *Part B2:* as part B with an ore mine whose type has
the animated flag. *Expected:* the mine still makes resource, and only next to the mine. *Part C:*
the switch off at start and turned on by a trigger at tick 300. *Expected:* no stage change of any
cell present at the start (their growth lists were left empty) in the 3000 ticks after tick 300,
but new cells do appear (spread runs), and the new cells grow. *Falsified if:* part A differs from
the control, part B or B2 shows a stage change or a new cell away from the mine, or part C shows a
stage change of a cell present at the start, a stage change before tick 300, or no new cell after
tick 300.

**E4. Growth period and count (claims 6, 7, 13; F3, F5).** *Setup:* one tiberium with growth field
*G* = 100 (expected period 29 ticks in the original, 30 under round-to-nearest), growth fraction 0
for a second tiberium, spread fraction 0; a field of 40 flat cells at stage 0. *Observable:* the
ticks at which any stage changes; the number of cells that change per update. *Expected:* changes only in
ticks 0, 29, 58, … (a period of 29 ticks runs the next update once 29 ticks have passed since the
last one); between 1 and *k*<sub>g</sub> cells change per update, with *k*<sub>g</sub> = 5 for
*n* = 40 and *g* = 0.1.
*Falsified if:* the second update comes at tick 30 (that would falsify the round-toward-zero
product and bear on `quirks.chop_rounding`), or more than *k*<sub>g</sub> cells change in one update.
*Runs:* 3 seeds; with the random-state recorder, one raw draw for *m* per update plus the entry
draws of claim 7.

**E5. Spread and germination (claims 5, 8, 9, 11).** *Setup:* a single ore cell at stage 11
surrounded by at least two rings of germinable clear cells; spread field 10, spread fraction 1.0,
growth fraction 0. *Observable:* the ticks at which new cells appear, their positions and stages, and,
with the random-state recorder, the direction draw of each spread step. *Expected:* new cells
appear only in the eight neighbours of the centre, each at stage 3, and several can appear in one
spread update (claim 9); each new cell is the first germinable neighbour counted from the drawn
direction; none appears in the second ring before the centre is emptied, because new cells are not
offered to the spread list and with growth fraction 0 never grow (claim 11). Then harvest the
centre cell empty. *Expected:* the emptying offers the stage-3 neighbours to the spread list
(claim 5); new cells now appear in the second ring, and in at least one of 5 seeds the emptied
centre receives new resource within 3000 ticks. *Falsified if:* new cells appear at another stage,
a new cell is not the first germinable one from the drawn direction, a second-ring cell receives
resource before the centre is emptied, or no new cell appears beyond the first ring in the 3000
ticks after the emptying in any of the 5 seeds.

**E6. Stage needed to spread, slopes (claims 8, 12, 17).** *Run 1:* four isolated cells, one of
each of the tiberiums at indices 0 to 3 (override sections give all four the same spread and
growth fields, spread fraction 1.0 and growth fraction 0), at stage 1, each with several free
neighbours. *Expected:* the cells of indices 0 and 1 spread; those of indices 2 and 3 do not (repeat
with stage 2: all four spread). *Run 2:* one tiberium with growth fraction 1.0 and spread fraction
1.0; a cell on a slope of shape 2 at stage 5 with free flat neighbours, and a flat control cell at
stage 5 elsewhere, each isolated from the other. *Expected:* the control cell grows and spreads, and
the sloped cell neither grows nor spreads. *Falsified if:* in run 1 index 2 or 3 spreads at stage 1
or index 0 or 1 fails to spread at stage 1, in run 2 the sloped cell changes stage or spreads or
the control cell does not.

**E7. Spread-list consequences (claim 9).** *Setup:* maps of at least 40 by 40 cells (no
rebuilding, F7); spread fraction 1.0; growth fraction 0. *Part A:* a single stage-11 cell with
exactly one free germinable neighbour, one neighbour holding a neutral building that the human
player can destroy, and the remaining six neighbours non-buildable land. *Procedure:* at tick 600
destroy the building. *Expected:* the cell spreads once into the free neighbour in the first
spread update that reaches it; after the building is destroyed the cell never spreads again in
3000 ticks. *Falsified if:* the original cell spreads a second time. *Part B:* a single stage-11
cell with two or more free germinable neighbours and an infantry unit of the human house standing
on it from the start until tick 600. *Expected:* no new cell appears before tick 600; with the
random-state recorder, every spread update before tick 600 takes exactly one raw draw (the one for
*m*) and no direction draw; after the unit leaves, new cells appear within a few spread periods.
*Falsified if:* a new cell appears before tick 600, a direction draw appears while the unit stands
there, or no new cell appears after the unit has left (the entry was dropped, which would mean
failed attempts do not return to the list).

**E8. Germinable cells (claim 10).** *Setup:* around a spreading stage-11 cell: a cell with an
infantry unit, a cell with a tree, a cell with an ore mine, a cell with a living building, a cell of
a high bridge, a cell of non-buildable land, a cell holding a wall. *Observable:* which cells
receive new resource over 6000 ticks, 5 runs. *Expected:* the infantry and tree cells can receive
it; the others never. *Falsified if:* any of the others receives resource, or the infantry or tree
cell never does over the runs while free cells do.

**E9. Ore mines (claim 15).** *Setup:* an ore mine type with the animated flag, *p* = 0.01, *R* = 5,
a sprite of 2*B* image frames; growth fraction 0 for all tiberiums; part B: a gem cell placed under
the mine by the map; part C: the map growth switch off; part D: *R* = 0. *Observable:* ticks
between animation start (first image frame change) and the appearance of a new cell; the new
cell's tiberium and stage; in part D the random-state recorder. *Expected:* a new cell of the first
tiberium at stage 3, about 5*B* ticks after the animation starts; part B: a new cell of the
tiberium of the cell under the mine (gems); part C: still produced; part D: a raw draw in every
tick (the mine never has a running animation) and no new cell, and the image never leaves frame
0. *Falsified if:* another tiberium appears in part A, the first tiberium appears in part B, the
stage differs, nothing appears in part C, or part D shows a new cell or a tick without a draw.

**E10. No regrowth of a harvested full cell (claims 7, 18).** *Setup:* a map of at least 40 by
40 cells (no rebuilding); two isolated cells of one tiberium, one at stage 11 (called A) and one at
stage 5 (B), spread fraction 0 (so no new cell), growth field 3000 (period 899 ticks), growth
fraction 1.0, a harvester with capacity 3 next to them. *Procedure:* B is the only entry in the
growth list, so the first update (tick 0) takes it again and again until its *m* attempts are used
(claim 7) and B is at stage 5 + *m*, with *m* from 1 to 5 (read *m* from the recorder). Before
tick 899, let the harvester take 3 bales from each cell (two loads; order it to each cell).
*Observable:* the stages over the next 9000 ticks. *Expected:* A stays at stage 8; B rises by
*m*′ stages at each growth update (*m*′ from the recorder, up to the stage 11) until it reaches 11.
*Falsified if:* A grows above 8, or B does not rise, or B's first update does not raise it by *m*
stages (which would falsify the re-entry rule of claim 7).

**E11. Timer carry-over (claim 19).** *Procedure:* in one session, play a first skirmish with
growth field 3000 (period 899) on a field of stage-0 cells for 20000 ticks; return to the menu and
start a second skirmish on the same map. *Observable:* the tick of the first stage change in each
game. *Expected:* game 1: tick 0; game 2: no change before the tick number reaches the last update
tick of game 1 plus 899 (about 20000). Control: a fresh program start for game 2 gives tick 0.
*Falsified if:* game 2 grows at tick 0.

**E12. Rebuilding during play (claims 9, 13, 18; F7).** *Setup:* a very small map (for example
*W* = *H* = 16, so *C* = 640), one tiberium with growth field 10 (period 2 ticks), growth fraction
1.0, spread field 2, spread fraction 1.0, a spreading field that keeps making new cells, and next to
it one stage-11 cell that the harvester of scenario E10 cuts to stage 8. *Observable:* the stage of
the harvested cell over 20000 ticks; with the random-state recorder, the draws per growth update.
*Expected:* the harvested cell stays at 8 for a while and then grows again at a later growth
update, after a rebuilding of the growth list (F7) has put it back with key 0; the draws of every
growth update follow the order of claim 7, and the entries the rebuilding made (key 0) are taken,
in visiting order, before any entry added after the rebuilding (all of which have positive keys).
*Falsified if:* the harvested cell never grows again while new cells keep appearing over the whole
run (no rebuilding during play), or it grows again at once. *Runs:* 3 seeds; a recorder of the
list sizes, if available, confirms the rebuilding tick.

**E13. Craters take bales (claim 16, part c).** *Setup:* a test copy of the art database gives one
explosion animation type the crater flag only, another both the crater and scorch flags, and a third
the scorch flag only; three weapons use them, each fired at its own isolated stage-8 resource cell
on flat ground with no building. *Observable:* the stage of each target cell per tick, and with
the random-state recorder the draws at the middle event. *Expected:* the crater-only target falls
to stage 2 at the middle event; the crater-and-scorch target falls to stage 2 when the draw
gives a crater and stays at 8 when it gives a scorch mark (over 20 seeds both occur); the
scorch-only target stays at 8. Repeat the crater-only case at incoming stages 0, 5, 6 and 7:
stages 0 and 5 lose their resource overlay, stage 6 retains it at stage 0, and stage 7 retains
it at stage 1. Record overlay presence as well as stage, since an emptied cell also has stage
0. A fourth run from a high bridge deck (height 30 or more above the terrain) leaves the stage
unchanged. *Falsified if:* the stage falls by any other amount, a boundary case differs in stage
or overlay presence, the crater-and-scorch case always removes or never removes, or the
scorch-only target changes.

**E14. Fence lines, the waypoint trigger and a building on a resource cell (claim 16, parts e, f
and the paragraph after the list).** Parts A and B use growth and spread fractions 0 for every
tiberium, so that no other change reaches the observed cells. *Part A:* two matching fence posts on
a test map, five cells apart in one line, with stage-8 resource in each of the four cells between
them and in one cell beside the line. *Expected:* at the tick the line is built the four cells between the posts are
empty and the side cell is unchanged. *Part B:* a skirmish map whose trigger fires action code 59
at a waypoint over a field of stage-8 cells (at least 11 by 11 cells around the waypoint) and,
in a second run, a waypoint cell at stage 200. *Expected:* the waypoint cell is empty (second run:
stage 91) and every other cell of the field stays at stage 8. *Falsified if:* a cell between the
posts keeps its resource, any cell other than the waypoint's changes, or the waypoint cell is not
emptied (stage 8 at the waypoint after the action) or does not drop by 109 stages (second run).
A field in which every cell of the disc of 109 displacements is at stage 7 falsifies this rule and
shows the behaviour of `quirks.thinning_hits_waypoint_only` off. *Part C:* a map that
places a building on a stage-8 resource cell, spread fraction 1.0, growth fraction 1.0. *Expected:*
the cell keeps its resource under the building (stage readable by a recorder), grows, and does
not spread while the building stands. *Falsified if:* the resource is gone, or the cell spreads
while the building stands.

**E15. Order among equal keys (claim 14).** *Setup:* a map of at least 40 by 40 cells; one
tiberium, growth field 100, growth fraction 0.1, spread fraction 0; 40 isolated flat stage-0 cells
in a row, numbered 1 to 40 in the visiting order of S-0009 claim 3. *Procedure:* run only the
update of tick 0 (n = 40, so *k*<sub>g</sub> = 5); read *m* and the re-entry draws from the
random-state recorder, and the changed cells after tick 0 from the recorder of cell contents.
*Expected* (original): for *m* = 1 cell 1 changed; *m* = 2 cells 1 and 40; *m* = 3 cells 1, 40 and
2, in every run where no re-entry draw is a multiple of 50. This spec's rule would give cells 1, 2
and 3. *Runs:* enough seeds to see *m* = 2 and 3 at least 5 times each. *Falsified if:* for *m* =
2 the changed cells are 1 and 2 (the original then orders ties as this spec does and the
departure can be dropped), or any other pair.

**E16. The list that takes a neighbour of an emptied cell (claim 5).** *Setup:* a map of at least
40 by 40 cells; two tiberiums, A with spread field 10 and spread fraction 1.0, B with spread field
2000 and spread fraction 1.0, both with growth fraction 0; one A cell at stage 0, adjacent to it one
B cell at stage 11 whose other seven neighbours are non-buildable land; a harvester of the human
house next to the A cell. *Procedure:* at tick 600 let the harvester take the A cell (one bale
empties it). *Observable:* the tick at which the emptied cell becomes a B cell of stage 3.
*Expected:* within 10 ticks of the emptying (the cell joins A's list, period 10; an update takes
entries whatever their keys). With `quirks.spread_offer_by_emptied_type` off the cell would join B's
list and appear only at B's next update (about tick 2000). *Falsified if:* the new cell appears only near tick 2000.

**E17. Start-event chain-reaction draws (claim 20; Q-0046 Q2a).** *Setup:* a purpose-made
skirmish map, game speed 2, the usual human and bystander houses, director and mods off. Named
art test data gives a tile animation the resource-chain-reaction flag, no particles, crater or
scorch flags, no bouncing, and a sprite with μ > 0. Named map overrides set growth and spread
fractions to 0 and the resource-explosion damage to 0, isolating the direct draws. Use the
random-state recorder during map loading and a cell-content recorder at the first tick.
*Procedure:* create that tile animation over (A) an empty cell, (B) a resource cell whose
resource-debris list is empty, (C) the same resource cell with a singleton debris list, and
(D) a two-entry list. Debris types have μ > 0 and no immediate effects. Account separately for
each animation's stage-delay draw (S-0202). The base cases use target tiles that
name no animation, so neither the emptying nor the post-detonation recalculation adds
tile-animation draws. Repeat C and D until both passing and failing trials
occur, at least 10 of each. *Expected:* A takes no chain-reaction draws or detonation and leaves
the cell empty; B empties its cell and takes no direct chain-reaction draw; C and D empty the
cell and take exactly one raw debris trial. A trial with |*r*| mod 3 ≠ 0 creates nothing. A
passing C trial creates the sole debris animation, with no raw selection draw; D takes the
ranged selection before its debris creation draws. Debris starts 10 leptons above the source.
*Falsified if:* A advances the stream beyond the animation's independent creation draws, B takes
a debris trial, or C or D deviates from those conditions or ordering. Repeat B to D with
trigger-played creations at waypoint markers (S-0011 T21): their creation-time start events
must empty the resource and follow the same reaction draw rules.

*Later-start suppression control:* give the test animation a positive loop-delay range and
enough repeats to observe a later start event. Enable normal spreading from nearby resource
so that the emptied target can receive resource again during the delay. Record the target's
resource and stream state immediately before and after that later start. Count only runs in
which resource is present before the event and no unrelated removal occurs. Obtain at least
10 such events for both a trigger-played animation and an ordinary-animation control. Expected:
the trigger-played later start leaves the resource unchanged and takes no reaction draws,
even with a nonempty debris list; the ordinary later start empties it and follows the normal
reaction rules. Account separately for loop-delay, middle-event and resource-update draws.
A suppressed creation reaction, or a reaction at the trigger-played later start, falsifies
the creation/later distinction. Inability to refill the target in time is inconclusive.

*Variants:* repeat during play using a weapon explosion animation with the same settings,
subtracting its projectile and explosion-selection draws. Enable spreading on unmarked eligible
neighbours in this variant: their claim-5 entry draws must precede the debris trial. Restore a
nonzero resource-explosion damage and put a wall in its scanned cells: its S-0303 wall draws
must follow debris creation. Repeat this nonzero-damage variant with an empty debris list
and with a failed debris trial: the detonation and its applicable draws must still occur,
following any emptying draws in the empty-list case and following the trial in the failed-trial
case. Finally use μ = 0 and a crater-only animation over a stage-0
resource cell: the middle event empties the cell and the later chain reaction has no debris trial
or detonation. Repeat the load cases with an overlay cell animation whose position, rather than
its overlay cell, is over the test resource (S-0009 M6e). Any differing gate or draw order
falsifies claim 20; unrelated middle-event draws are accounted for by their owning specs.

*Post-detonation terrain variant:* use named theater test data and a resource overlay whose
land type makes its cell the overlay case of S-0010 claim 8. Before removal its tile A names
no animation and remains unsmoothed. Choose the transition settings and neighbours so that
emptying recalculates the cell from tile A to tile B under S-0010 claim 10. Tile B names a
non-bouncing animation with μ > 0, a non-singleton random-rate range and no immediate effects.
Its attaches-to value matches the target's sub-tile. Use a displaced source animation in a
separate cell so that creating it does not recalculate the target first. Keep the target
clear of grounded objects and use a detonation that changes no tile. Record tile, land type,
animation starts and stream state through the reaction during loading. Expected: emptying
leaves tile B but no tile animation (that recalculation began with A); after the detonation,
B's animation starts and takes its stage-delay draw after all applicable detonation draws.
The land type then reflects B rather than A. Repeat with a tile animation already started
on the target since the second load pass: the post-detonation recalculation takes no further
animation-creation draw. Include empty-debris and failed-trial cases, which still recalculate,
and the no-resource and trigger-played later-start controls, which do not. A trigger-played
creation over resource must still recalculate normally. A missing or premature start,
a duplicate start despite the once-only rule, or a different stream order falsifies the claim.

## Open questions

- Whether the tick number is 0 when the tiberiums are created in the first game of a session, which
  decides whether the first updates fall in tick 0 (claim 19). Scenario E11's game 1 settles it.
- Whether any removal outside those listed in claim 16 takes bales from cells.
- A germinable cell keeps whatever overlay data value it had; when a map stores a value of 11 or
  more on a cell without an overlay, the growth-list offer of claim 11 (made before the new stage is
  set) may see that old value and add no entry. Whether placing the new overlay resets the value
  first was not established (low confidence; a map with such a cell next to a spreading cell
  settles it with the random-state recorder).
- How often a rebuilding during play (claim 13, F7) happens on stock-sized maps over a long game:
  scenario E12 measures the rule on a small map.
- The absent-key defaults of the ore-mine terrain-type keys (Import mapping).

## Questions for the project lead

None open. Claim 14 applies the lead's ruling of 2026-10-10 on pathfinding ties (S-0212, D7) to the
growth and spread lists: equal keys are taken in the order the entries were added. If the lead
prefers the original's tie order instead, the spec would have to state that order exactly; it
depends on the whole history of adds and takes, so no
shorter statement keeps its meaning.
