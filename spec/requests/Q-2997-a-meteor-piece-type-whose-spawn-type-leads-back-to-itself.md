---
id: Q-2997
title: What does a meteor debris piece type do whose spawn type leads back to itself?
status: open
related: [S-0317, S-0203]
---

## Need

S-0317 claim 12 makes an ending meteor piece create *n* = R(0, *c*) + R(0, *c*) pieces of its
spawn type, and claim 6 puts them in play at once, so they take their first turn in the same
pass (S-0203 claim 4). The spec does not say what happens when the spawn type is the type itself,
or when two types spawn each other. With a short lifetime each piece ends in its first turn and
makes up to 2*c* more in the same pass, so in our engine one tick never ended (GitHub issue
#174). No shipped type is known to do this; a mod or a mistake in a custom layer can.

## Questions

1. With `Spawns` naming the piece type itself, how many pieces does one piece lead to, and over
   how many ticks?
2. The same for two types that name each other.
3. Does the game refuse or change such data when it loads the rules?

## Possible observation

A test rules layer gives an invented meteor voxel animation type X `Spawns=X`, `SpawnCount=2` and
a short `Duration`, shown by a warhead's debris list. Fire it at an empty cell and record, per
tick, the number of pieces drawn (or whether the game stops responding) for 30 ticks.

## Current workaround

At rules build our engine marks every spawn link that lies on a loop of piece types. A piece made
through a marked link carries a saved mark (`pieces::Piece::looped`) and makes no pieces through a
marked link when it ends: it takes neither count draw for it. So one piece leads to at most 2*c*
pieces of the loop. Links on no loop are unchanged. The importer warns about each loop.
`// nocite: our own input validation (#174)`, in `pieces::end`, with tests in
`crates/deterministic/sim/tests/spawn_cycle.rs`.
