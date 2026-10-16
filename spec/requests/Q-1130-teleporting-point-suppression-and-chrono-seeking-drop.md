---
id: Q-1130
title: How pending point suppression meets a teleporting destination, and when a teleporting chrono miner "starts seeking"
status: answered
related: [S-0207, S-0502, S-0402, S-0417]
answered-by: [S-0402]
---

## Need

S-0207 revision 34 claim 27 (*The restoring assignment's destination*) says that restoration "does not
exempt this application from suppression already pending before the assignment", so a one-application
movement-point suppression can meet a teleporting locomotor taking a destination. The same claim says
suppression "applies equally to claim 25's driving point and to a jumpjet movement point". It does not
list a teleporting landing point. It also does not say how suppression and claim 20 (a)'s refusal of a
destination given while phasing in or held are ordered. A restored predecessor can still be phasing in
(claim 21 keeps that state). The engine has to place the test in `teleport::take_destination`
(`crates/deterministic/sim/src/teleport.rs`).

S-0402 revision 27 claim 17 says a chrono miner whose active locomotion is teleporting, "when it starts
seeking with a destination", drops that destination and runs the ring search from where it stands. The
remembered cell it adopts at that moment counts too. S-0417 revision 20 claim 16 confirms that the test
reads the active locomotion. Claim 3's phase table has no "start" event inside *seeking*. A harvester is
processed in *seeking* several times: when it enters *harvest*, after a ring search that sent it to
another cell, and while it travels with a destination. The engine cannot tell which of these processings
drop the destination (`harvest::seeking` in `crates/deterministic/sim/src/econ/harvest.rs`).

## Questions

1. Does a pending one-application point suppression skip the landing-point fixing of every teleporting
   vehicle's nonempty destination (S-0207 claim 20 a)? Or does it skip only the assignment that restores a
   suspended predecessor under claim 27? Is the suppression consumed when it skips a teleporting point?
2. A teleporting vehicle is phasing in or held, so claim 20 (a) drops a destination given to it. Suppression
   is pending. Which happens: the destination is dropped and the suppression stays pending, or the
   suppression is consumed and the order-level destination stays without a landing point?
3. For a chrono miner with active teleporting locomotion, which *seeking* processings drop a destination
   (or a remembered cell) and run the ring search instead? Only the first processing after it enters
   *harvest*? Every *seeking* processing that finds a destination? Or another event?
   Does the drop clear the order-level destination, the fixed landing point, or both?

## Possible observation

Question 3: S-0402 H8 with an active teleporting chrono miner. Give it a *harvest* order on an ore cell
several cells away while other ore lies nearer. Record its destination, its landing point and its cell per
tick from the order through its first loading. Repeat after an unloading, when the miner seeks again with a
remembered cell. Questions 1 and 2 have no ordinary stock fixture that we know of (S-0207 D32).

## Current workaround

Interim rules, noted in `docs/development.md` (section *Simulation (M6c: destination refresh and the dock
fallback)*):
- Questions 1 and 2: pending suppression skips any teleporting vehicle's nonempty landing-point fixing
  and is consumed by it. The order-level destination stays. The test comes after claim 20 (a)'s
  phasing-in and held refusal, so a refused destination is not an application and the suppression stays
  pending.
- Question 3: no seeking destination is dropped. A chrono miner with teleporting locomotion keeps its
  destination as any harvester does (claim 3's "Remains seeking" row). This is unchanged from before.
