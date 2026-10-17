---
id: Q-1153
title: S-0616 (particle systems of the other kinds) is referenced but not in spec/
status: open
related: [S-0301, S-0302, S-0613, S-0004, S-0616]
answered-by: []
---

## Need

Several approved revisions now point at S-0616 for behaviour our engine needs, but no S-0616 is in
`spec/`, so we cannot implement or cite it:

- S-0301 revision 21, claim 5: the liveness of an effect weapon's particle system rests on S-0616.
- S-0302 revision 28, claim 13: the damage effect is "marked" until its particle system leaves play
  (S-0616), no longer removed at once.
- S-0613 revision 6, claim 4: the smoke, fire, spark and railgun kinds emit and move their particles
  by S-0616 claims 5 to 18; claim 17: removal of a system's owner object marks it to end (S-0616
  claim 4); S-0616 claim 22 lists the other kinds' marks; the empty-named held type is S-0616
  claim 21.
- S-0004 revision 18, claim 13: the empty-named particle type is registered by reference
  (S-0616 claim 21).

In our engine the smoke, fire, spark and railgun systems are kept and take turns, but their first
particle is not kept (Q-1243, `gas::warhead_system`); a system has no owner object; effect weapons
and S-0302 claim 13's effect do not depend on a particle system.

## Questions

1. Can S-0616 be approved into `spec/`, or should the claims above be read without it until it is?
2. Until then, should S-0301 claim 5 and S-0302 claim 13 keep the behaviour of their previous
   revisions (no dependence on a particle system's life), as our engine does now?

## Possible observation

None needed: this is a request for an existing spec to be released.

## Current workaround

The rows that rest on S-0616 are classed *not built (S-0616 absent)* in
`docs/m7-claims/step1g-m7a.md`. The engine keeps its current behaviour: non-gas systems are inert
apart from their turns and lifetime (Q-1243), no system has an owner object, and S-0301 claim 5 and
S-0302 claim 13 behave as in S-0301 revision 20 and S-0302 revision 27. The importer's part that
needs no S-0616 behaviour is built: a system section read without `HoldsWhat` holds the empty-named
type (S-0613 Import mapping), already registered by `layers.rs`.
