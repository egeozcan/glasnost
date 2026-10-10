---
id: Q-0020
title: Absent-key defaults, bridge decks, wall ownership and gate state for the M3 simulation
status: open
related: [S-0207, S-0209, S-0210, S-0211, S-0215]
answered-by: []
---

## Need
`glasnost_sim` needs a few values and states that the approved specs point
to but do not give. Each one is marked in the code with this request's ID.

## Questions
1. **Precision of two engine defaults (S-0207 Constants).** The absent-key
   defaults of the acceleration factor (0.03) and the deceleration factor
   (0.002) are given as decimals. Is each one the double nearest to that
   decimal, the single-precision value widened to double, or another
   value? (S-0207's import mapping says that the imported factor is a
   decimal widened to double. The engine default may be held in a
   different precision, as happens with the default behaviour rate in
   S-0215 F1.) The code uses the nearest double.
2. **Absent-key defaults of the area-guard threshold and the guard stray
   distance (S-0215 Import mapping).** What does the game use for
   `[IQ]` `GuardArea` (claims 14 and 15) and `[General]` `GuardModeStray`
   (F5) when no rules layer gives them? S-0215 lists the stray default as
   open. The code uses a threshold of 0 and, without a stray distance, the
   leash of F5.
3. **Overlay damage-stage count and wall strength defaults (S-0210 claim 8,
   F3).** What are the absent-key defaults of an overlay type's
   `DamageLevels` and `Strength`? S-0210 says that the overlay specs own
   them, and those specs are not written. The code uses 1 and 0.
4. **Bridge decks (S-0209 Terms; S-0207 claim 9; S-0208 Terms).** Which map
   cells carry a bridge deck, and which are high-bridge cells? The specs
   use the bridge-deck layer, the deck height of 4 levels and the
   high-bridge cell in several rules, but no approved spec says how a
   deck is identified from the map and the overlays. The code has no
   decks.
5. **Owner of a map's walls (S-0209 claims 4 and 5).** The entry-outcome
   rows for walls compare the wall's owner with the mover's house. Who
   owns a wall overlay that the map places? The code treats every wall as
   owned by no house, so it counts as "another house".
6. **Gate state (S-0209 claim 5, row WP11).** Is a gate building open or
   closed when it is placed, and when does it open and close? The code
   reads every gate as open.

## Possible observation
- Questions 1 and 2: a rules file without the keys, a vehicle that
  accelerates from rest (with the state recorder reading its speed
  fraction), and a unit in area guard led away from its guard spot.
- Question 5: a tank of each house next to a map-placed wall, ordered
  through it, with the outcome noted for each.
- Question 6: a map-placed gate with a friendly unit ordered through it
  from the first tick.

## Current workaround
Each value above is marked "Q-0020" at its place in the code. None of
them is a quirk switch, because none is known to be the original's
behaviour.
