---
id: Q-3112
title: Which deaths the crash death-weapon switch reaches when it is off
status: open
related: [S-0312, S-0502]
answered-by: []
---

## Need

S-0312 Edge cases give `quirks.crash_death_weapon_twice`: on, an exploding
non-rocket aircraft fires its death weapon at death and again at the impact
of its crash (claim 16); off, it fires only at the impact. The engine now
reads the switch (sim batch S13, issue 252) for an exploding non-rocket
aircraft that starts to crash. Two cases the switch's text does not settle:

- An exploding aircraft killed with no height above the terrain does not
  crash: claim 16 has it leave play at once, so it has no impact. With the
  switch off, read literally ("only at the impact"), it would fire no death
  weapon at all.
- A crashable vehicle using jumpjet movement crashes as well (claim 16 and
  S-0502 claim 18). The switch names aircraft only.

The code is `death::kill_in`, step 2 (the death weapon).

## Questions

1. With the switch off, does an exploding aircraft killed on the ground
   (no crash, it leaves play at once) fire its death weapon at its death, or
   not at all?
2. Does the switch reach a crashable vehicle using jumpjet movement that is
   killed in the air and crashes? If so, with the switch off, does it fire its
   death weapon only at the impact?

## Possible observation

None needed: the switch's off state is a design choice of ours, so the spec
side can settle it by stating the intended scope.

## Current workaround

With the switch off, only an exploding non-rocket aircraft that starts to
crash skips the death weapon at death. An aircraft that dies without
crashing, and every crashable vehicle, keep the switch-on behaviour.
