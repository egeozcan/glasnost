---
id: Q-2782
title: The scatter from damage against the arena records M-0021, M-0001 and M-0003
status: open
related: [S-0215, S-0208, S-0309]
answered-by: []
---

## Need

The M8 build of S-0215 claim 19 (the scatter from damage) and S-0208 claim 15 (request A of a
hit) is faithful to the claims as written, but the arena now fails rows that its records grade
*exact* or unanimous. These rows passed before the build, when a hit sent no scatter request:

- M-0021 part B:
  - "c ≥ 3 in every judged shot" fails in 3 shots: F on side B, seeds 1001 (two shots) and
    1002;
  - "hits a judged shot owns" fails in the same 3 shots (2 or 1 fragment hits in place of 3);
  - the reading-points row "some part B deaths have a shot of the destroyed vehicle at t, each
    one's hit in a later tick naming no attacker";
  - the side-left and dead count row fails on main too, but one run's G-dead count moves.
- M-0001: the pairings LTNK-APOC and LTNK-E1 now fail; LTNK-E1's winner flips.
- M-0003: the pairing MTNK-APOC now fails.

In our engine, every arena house is a test-only computer-controlled player house (S-0215 Terms).
It takes the original's computer-controlled-owner rules, so it meets case 2's last condition.

The code is `damage::damage_scatter_sent` and `damage::reactions`
(`crates/deterministic/sim/src/combat/damage.rs`), with `acquire::retaliate`
(`crates/deterministic/sim/src/combat/acquire.rs`).

## What our runs show (game and object facts from our engine)

I probed three games with a temporary local probe.

**M-0021 part B, F on side B, seed 1001.**
- 22 requests are sent: 19 to G vehicles and 3 to F vehicles, all by case 2 of claim 19.
- Every receiver is in *hunt* and has neither a target nor a destination when hit. Its current
  behaviour has the scatter property, it has no tether link, its locomotor is not moving, and
  its owner is computer-controlled.
- 21 of the hits name a known attacker. All 21 are answered *may not retaliate* by one
  condition only: S-0215 claim 18's "only while the current behaviour has the retaliate
  property", as the installed data gives that property. One of them also has an allied
  attacker (S-0309 claim 18).
- The 22nd hit names no attacker.
- In both judged shots of this run with *c* < 3, the target *X* is a G vehicle that a hit had
  sent the request 6 ticks before the first of the two shots. It took the next cell toward the F
  group as its destination, and by the second shot it stood there.
- Two of the G vehicles next to *X* had been sent the request too: one tick before *X*, and one
  tick after the first shot. Each took a cell next to its own as destination, the same cell for
  both.
- At the first shot only 2 cells of rings 1 to 3 around *X* held a G vehicle and no F vehicle;
  at the second shot, 1.

**M-0001 LTNK-E1, side A, seed 0.** 13 requests, all case 2: 11 to the tanks, 2 to E1 soldiers.
- 11 of them are answered *may not retaliate* by the same claim 18 condition: 10 tanks and 1
  soldier.
- 1 tank has an allied attacker.
- 1 soldier is hit by a hit that names no attacker.
- The first difference from the run without the build is a tank in *hunt*. It dropped its target
  6 ticks before an E1 hit it, and it had no destination; the scatter gives it one, and it takes
  a new target 3 ticks later.

**M-0003 MTNK-APOC, side A, seed 0.** 5 requests, all case 2 and all to APOC vehicles.
- 4 are refused retaliation on the same condition.
- 1 comes from a hit that names no attacker.

The claims involved are:
- S-0215 claim 19 case 2 and its Original-only note;
- S-0215 claim 18 and S-0309 claim 18, for *may retaliate*;
- S-0208 claim 15, the second case of request A, and S-0208 Terms *may retaliate*;
- S-0215 Terms *Test-only computer-controlled player house*.

## Questions

1. In the original, is a vehicle of a computer player house that is in *hunt*, with no target
   and no destination, sent the scatter from damage when a hit by an enemy it may not retaliate
   against lands on it? The enemy is one it may not retaliate against under claim 18's
   behaviour condition. Does it then move to a nearby cell?
2. If it is sent the request, do the original's vehicles in *hunt* in these fights ever stand
   with neither a target nor a destination when hit? M-0021's record has *c* ≥ 3 in every
   judged shot. Is the difference then in how long our hunting vehicles stand free between one
   target and the next?
3. Does a hit from a vehicle of the same house, or of an allied house, send case 2? Does a hit
   that names no attacker send it to a computer player house's vehicle?
4. Which reading is right for the arena houses:
   - (a) the claims as built, with these rows failing for another reason (question 2);
   - (b) case 2 does not reach a computer player house's vehicle in *hunt*;
   - (c) something else?

## Possible observation

Use M-0021 part B's own recordings (F on side B, seed 1001), or a new run of that setup. For each
hit on a G vehicle, record whether the vehicle had a target and a destination when hit, and
whether it changed cell in the next ticks without a target. Do the same around M-0001 LTNK-E1's
first tank hit while it has no target. A tiny map would also do: two computer houses, a
vehicle in *hunt* with nothing in sight, hit by a weapon from beyond its sight, then record its
destination and cell each tick.

## Current workaround

The claims as built: case 2 is sent to the arena houses' units, and the arena rows above fail. No
switch was added and nothing was tuned toward the records.
