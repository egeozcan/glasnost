---
id: Q-2868
title: How a kamikaze aircraft is removed after its ordinary shot
status: answered
related: [S-0501, S-0506]
answered-by: [S-0501]
---

## Need

S-0501 r172 claim 20 says: "A kamikaze slot aircraft (S-0506) is removed after its ordinary shot,
also when that shot creates no projectile. Every flagged object that a carrier releases is a
kamikaze (S-0506 claim 8, Terms *kamikaze*), a flagged type that is not a rocket type included, so
such an aircraft makes a single ordinary shot and no run end after it (Q-2821)."

Our engine has never built this removal: an aircraft of a flagged type that is not a rocket type,
released by a carrier (`spawn::release_object`, enrolled for S-0506 claim 26's re-orders), flies
by S-0501's ordinary flight and fires ordinary shots (`combat::fire`) like any aircraft, and
nothing takes it out of play after its first shot. (Rocket types do not reach this: they fly and
end by S-0506's own rocket rules.) Neither claim 20 nor S-0506 says what the removal consists of,
so it cannot be built without guessing. Checked: claim 20's backing, S-0522 claim 11 (scenario P7),
is the drop by a shot of an aircraft carrying passengers and says nothing of a kamikaze; S-0506
Terms (*Kamikaze*: "it does not by itself mean that the aircraft is dead"), claim 9 and its slot
table (a kamikaze's slot empties at a loss notification or removal from play) and claim 12 (the
release cell and the re-orders every 30 ticks) say when the slot empties, not what removes the
aircraft after its shot.

## Questions

1. Is the kamikaze removed from play without dying (as claim 10's removal on leaving the map is,
   "without dying"), or is it destroyed? If destroyed: with or without its death effects (debris,
   explosion animation, death weapon, S-0312 claim 16's crash), and is the kill credited to anyone?
2. When in the shot does it happen: at once after the shot's own effects (projectile creation,
   ammunition spent, burst counter, report and animations), in the same tick, or later? Does a
   burst of more than one shot fire only its first shot?
3. Does its removal raise the loss notification of S-0506 claim 9 (so its slot empties with the
   regeneration countdown), and the destruction events of S-0011?
4. With "also when that shot creates no projectile": which shots count (a shot whose projectile
   creation fails, a shot whose weapon has no projectile)? Is a firing attempt the firing check
   refuses a shot here?

## Possible observation

S-0506 scenario R21's carrier with a test slot type that has the missile-spawn flag, is not a rocket
type and has an ordinary weapon (test values): record per tick the released aircraft's existence,
health, position and ammunition, the target's damage, the slot's state and the score, from the
release to 300 ticks after its first shot. A variant whose weapon's projectile cannot be created
answers question 4.

## Current workaround

Not built, as before r172: a released flagged aircraft that is not a rocket type stays in play after
its shot and goes on by S-0506 claim 26's re-orders and S-0501's ordinary rules. No interim cite is
added in the code, since no code path makes the removal; the claim part is listed as blocked in
the intake 118 ledger.
