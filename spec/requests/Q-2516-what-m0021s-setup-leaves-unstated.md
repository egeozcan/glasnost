---
id: Q-2516
title: What M-0021's test pairing set up that the record does not state
status: answered
related: [S-0216, S-0308, S-0318, S-0609]
answered-by: [M-0021]
---

## Need

The arena's M-0021 harness (`crates/tools/src/oracle_m0021.rs`, M8 step A3 intake 93) plays the
record's test pairing in our engine and judges each row of its Result. Three parts of the setup
are not in the record, so the harness makes its own choice for each. The record's Notes say the
per-shot rows and the stage rows do not depend on fields kept from the copied stock types. The
first-shot distances and part B's outcome follow how the vehicles move, so these choices may
change those rows.

## Questions

1. **Creation order.** In each run, which group was created and placed first: F's group, G's
   group, or side A's group whichever type it holds? Within a group, were the units created in
   increasing index on M-0001's cells, as M-0001's groups are?
2. **The gattling type's numbered weapons.** G has two stages, a stage 0 weapon and a stage 1
   weapon. Which weapon did the record's file put in each numbered weapon slot of G (slots 0 to
   3)? Were G's primary and secondary weapons set as well, and to which weapon?
3. **The observer.** The human player is a British observer that owns one building. Which
   building type, and on which cell? Did the observer take a start position? Did any other object
   of the observer stand on the map at tick 0?
4. **The run's end.** Procedure step 2 gives a firer's last shot a window "to the run's end". In
   our engine a shot fired a few ticks before a part's tick limit can still have fragments in
   flight when the run stops, so its window holds fewer than 3 fragment hits. Did the recorder
   keep logging hits after the tick limit until the shots in flight landed? Or did every judged
   last shot in the record's runs have all its hits before the limit? Or was a last shot left out
   when its fragments had not landed?

## Possible observation

The record's own files answer questions 1 and 2. Question 3 can be read from the record's
recording at tick 0. Question 4 can be read from the recording's last tick and the ticks of the
hits the last shots own.

## Current workaround

- Question 1: F's group is created and placed first, on whichever side it stands
  (`oracle_m0021::F_FIRST`). This follows the oracle arena's rule that the row type goes first
  (S-0216 claim 8, *Order*).
- Question 2: G's numbered slots 0 to 3 hold the stage 0, stage 0, stage 1 and stage 1 weapons
  (stage *s* fires slot 2*s*, S-0308 claim 7). G's primary and secondary weapons are both the
  stage 0 weapon.
- Question 3: the human player is the arena's idle human player house at start 3, with its base
  unit (S-0216 claim 8, *Game start*), not an observer with one building.
- Question 4: the run stops at the tick limit. A firer's last shot is not judged when it falls in
  the run's last 2 ticks, or when a projectile of that firer is still in flight at the end
  (`oracle_m0021::Findings::cut_short`). On the install, this leaves out the control's last
  shots that failed the hit rows.

The harness reports the first-shot distances and part B's outcome with this caveat.
