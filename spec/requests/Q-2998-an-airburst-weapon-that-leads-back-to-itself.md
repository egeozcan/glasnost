---
id: Q-2998
title: What does an airburst weapon do whose airburst projectile leads back to the same weapon?
status: open
related: [S-0306, S-0203]
---

## Need

S-0306 claim 21 makes an airburst projectile detonate once and launch nine projectiles of its
airburst weapon, which enter the update order and take their turns in the same pass. The spec
does not say what happens when that weapon's projectile is itself an airburst whose airburst
weapon is the first weapon again (a weapon W whose projectile names W, or two weapons that name
each other). In our engine each generation launched nine more, so the pass never ended or the
projectiles grew nine-fold per generation (GitHub issue #203). No shipped weapon is known to do
this; a mod or a mistake in a custom layer can.

## Questions

1. With an airburst projectile whose airburst weapon fires that same projectile, how many
   projectiles does one shot lead to, and over how many ticks?
2. The same for a loop of two weapons.
3. Does the game refuse or change such data when it loads the rules?

## Possible observation

A test rules layer gives an invented weapon W an airburst projectile P with `AirburstWeapon=W`.
Fire W once at an empty cell and record, per tick, the number of projectiles in flight (or whether
the game stops responding) for 60 ticks.

## Current workaround

At rules build our engine marks every airburst link (a weapon to the airburst weapon of its
projectile) that lies on a loop of weapons. A projectile launched through a marked link carries a
saved mark (`Projectile::looped`); when it detonates it launches no children through a marked
link and takes no draw for them. So one shot leads to at most nine projectiles of the loop.
Links on no loop are unchanged. The importer warns about each loop.
`// nocite: our own input validation (#203)`, in `combat::projectile`, with tests in
`crates/deterministic/sim/tests/airburst_cycle.rs`.
