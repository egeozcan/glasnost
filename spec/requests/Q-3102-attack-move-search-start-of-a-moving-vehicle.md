---
id: Q-3102
title: When the attack a moving vehicle's attack-move search queues becomes current
status: open
related: [S-0309, S-0215]
answered-by: []
---

## Need
S-0309 claim 23 (a) says that an attack-move search that finds a target "queues *attack* with an
immediate start and counts as engaged". S-0215 claim 7 says that "with an immediate start"
always means the readiness-dependent start, and claim 11 says that a vehicle in *move* whose
locomotor is moving it, with a heading point and a positive integer movement step, is not ready.
Nothing in claim 23 (a), S-0213 claim 6 or S-0215 claim 22 stops the vehicle or changes its
destination when the search finds a target. Read together, a vehicle attack-moving to a cell keeps
driving to that cell with *attack* queued, and *attack* becomes current only on arrival.

S-0309 scenario A6 part g expects otherwise: "after the first successful search the unit stays
engaged across tick starts and attacks the stationary enemy", and "when the enemy is removed and
attack processing falls back, it resumes the remembered destination". S-0215 claim 33's paragraph
on actors with an attack-move memory and scenario B27 (e) say only that the attack is first
processed on a later tick than the search, which fits either reading.

A trace on synthetic data (an armed vehicle attack-moving 16 cells, an enemy 3 cells off the route
and within weapon range), with the readiness-dependent start built: the search finds the enemy and
the vehicle is engaged with *attack* queued; it drives on with *move* current, the enemy leaves its
reach, its target is dropped by a later search, and *attack* becomes current only at the
destination, with no target. It never fires at the enemy. With *attack* made current at once, it
attacks the enemy as part g expects. (Issue #229.)

## Questions
1. When an attack-move search of a vehicle that is moving in *move* finds a target, when does
   *attack* become current: at once whether or not the vehicle is ready, at its next start when it
   is ready, or by some other rule that stops it first (a stop request, a destination change)?
2. If some rule stops the vehicle or changes its destination at that search, what is it, at which
   step of the tick does it act, and what does it do to the locomotor's destination and route?
3. Does the answer differ for infantry, for aircraft or for other locomotors (jumpjet, hover,
   ship)?
4. Do the attack-move *resumption* starts of claim 23 (b) (*move* or *attack* queued with an
   immediate start at a fall-back) depend on readiness as claim 7 says? (The engine now makes them
   readiness-dependent.)

## Possible observation
S-0309 scenario A6 part g: record current and queued behaviour, destination, position and target
every tick while the vehicle passes the enemy. The tick on which *attack* becomes current, and
whether the vehicle is still moving then, answer question 1.

## Current workaround
The attack-move search starts *attack* at once, whether or not the actor is ready
(`attack_move_search` in `crates/deterministic/sim/src/combat/acquire.rs`), as before issue #229.
The resumption at a fall-back uses the readiness-dependent start of S-0215 claim 7. The two tests of
the readiness-dependent search start in `crates/deterministic/sim/tests/claims_s0309.rs` are
ignored until this request is answered.
