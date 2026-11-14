---
id: Q-3100
title: How our engine identifies the infantry type singled out by name in claim 30's area-fire unload rule
status: open
related: [S-0215]
answered-by: []
---

## Need
S-0215 claim 30 gives an undeployed infantry deployer in *unload* whose deploy weapon has the
area-fire flag two outcomes: a unit of one type singled out by name aims at nothing new, and its
delay is the frame count of its *deploy* sequence + 1 when its undeploy delay is negative; a unit
of any other type takes its own cell as its target. `quirks.named_deploy_area_fire` (default on)
keeps the named type's rule; off, no type is singled out (Edge cases).

S-0215 Import mapping says the type is identified by a name fixed in the original's program,
compared without regard to case, and leaves the literal name out. Our rules format has no field
that marks the type, and the importer cannot mark a type whose name it is not given. So with the
switch on the engine cannot tell the named type from any other (`process_unload_infantry` in
`crates/deterministic/sim/src/process.rs`, issue #232).

## Questions
1. How does our engine identify the singled-out type with `quirks.named_deploy_area_fire` on: by
   a type name the spec gives (and which name), by an infantry-type flag that the importer sets
   from that name (and what the flag is called in our format), or in some other way?
2. If it is a flag, is it set only by the importer from the fixed name, so that a mod that renames
   the type loses the rule as Edge cases says, or may a ruleset of ours set it on any type?

## Possible observation
None needed for the mechanism: the rows themselves are already backed by S-0215 scenario B12
case c. This is a question of how our format carries the identification.

## Current workaround
No type is singled out, in both positions of the switch: every undeployed deployer whose deploy
weapon has the area-fire flag takes its own cell as its target in *unload*, with the delay of the
"any other type" rows. The named type's two rows (no new target; *deploy* frame count + 1) are
not built.
