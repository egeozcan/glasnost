---
id: Q-0018
title: Map-loading draws and terrain rules that no spec states yet
status: answered
related: [S-0009, S-0010, S-0202, S-0203, S-0211]
answered-by: [S-0009, S-0010, S-0202, S-0203, S-0211]
---

## Need
`glasnost_sim::load` runs a map's loading on the scenario random stream in
the order of S-0202's table "Draws during map loading". Several groups of
that table, and two terrain details, point to specs that do not exist yet.
Every draw they take shifts every later draw of the game, so the oracle's
random outcomes cannot match until they are known.

## Questions
1. **Random start cells (S-0009 claim 18).** The "nearby free-area search"
   for an 8 × 8 area clear for tracked vehicles: in which order does it look
   at cells around the candidate, what does "clear" test, and does it take
   draws? What do starting units consist of, where are they placed, and what
   draws do their placements take?
2. **Low bridges (S-0202 claim 10).** Which three cells does an end marker
   write as its end row, along which line does it look for its partner, and
   up to what distance?
3. **Veins (S-0009 claim 14, S-0202 claim 10).** The vein placement test and
   the vein connection pattern *c*.
4. **Animations at load (S-0202 table).** Which creation draws does a tile
   animation (S-0010 claim 19) and an overlay's cell animation take when it
   starts during loading?
5. **Map edges in terrain recalculation.** In ground and ramp smoothing
   (S-0010 claim 10) and in the cliff-back rule (S-0211 claim 8), what does
   a neighbour outside the map count as?
6. **Terrain objects that take turns (S-0203 claim 2).** Which field of a
   terrain-object type makes it "spawn resources", so that it enters the
   update order?

## Possible observation
S-0202's state recorder at the first tick of maps built to isolate each
group (one bridge, one vein seed, one animated tile, one random start).

## Current workaround
A random start cell is not searched: a game that needs one refuses to start
with a message naming this request. Starting units are not created (the
late phase still takes its draws as S-0009 states). Bridge end markers
complete no bridge and take no draw. Vein seeds are cleared and not rebuilt,
with no draw. Animations take no creation draw. Off-map neighbours read as
blank cells (S-0009 Terms): tile 65535, level 0, flat. No terrain object
takes turns. All marked provisional.
