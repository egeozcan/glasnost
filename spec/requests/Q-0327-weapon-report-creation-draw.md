---
id: Q-0327
title: How the presentation learns an object's creation draw for its weapon report variant
status: answered
related: [S-0701, S-0202]
answered-by: [S-0202]
---

## Need

S-0701 claims 21 and 32 fix an object's weapon report variant for its whole life. The variant is
entry *r* mod *n*, where *r* is the low 16 bits of the object's creation draw (S-0202 claim 10).

Our simulation takes that raw value and then discards it (`glasnost_sim::objects::create`), so the
presentation cannot see it. Keeping the value on the object would add game state. That changes
saves, the state hash and the determinism goldens. The task brief for S-0701 forbids that change
without the lead's decision.

## Questions

1. Which should our engine do?
   - The simulation keeps the low 16 bits of each object's creation draw as object state, saved
     and hashed.
   - The simulation reports the value through a presentation-only effect when the object is
     created, and the presentation remembers it per object.
2. Which of the two is expected for objects created before the presentation starts listening
   (pre-placed and starting units)?

## Current workaround

Every object uses report entry 0 of its weapon's list and takes no draw. So claim 21's "firing
takes no draw" holds, but the variant does not follow the creation draw. Documented in
docs/development.md, "Audio".
