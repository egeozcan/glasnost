---
id: Q-2586
title: Where S-0215 claim 16's no-threat fall-back falls among the other reactions to the same hit, and whether a heal triggers it
status: answered
related: [S-0215, S-0302, S-0309, S-0208, S-0107, S-0607]
answered-by: [S-0215]
---

## Need

S-0215 r104 claim 16: "When a mobile actor that is not a team member, with the no-threat property
but not the zombie property, takes damage from a known attacker and survives, it falls back to
rest at once (claims 13 to 15), which normally ends the no-threat behaviour." The confidence line
rates "the exact damage outcomes that trigger the fall-back: any damage that changes health and
leaves the actor alive" medium.

The same hit can set off other reactions of the actor: retaliation (S-0215 claim 18, S-0309
claim 18), the scatter from damage (S-0215 claim 19 for vehicles and aircraft, S-0208 claim 15
for infantry), fear (S-0107 claim 16) and anger (S-0309 claim 16), and for a vehicle the crush
response (S-0215 claim 39). Claim 39 says the crush response "follows every other effect of the
same hit on the vehicle (S-0302)", so the fall-back comes before it. No claim says where the
fall-back stands against the others, and the order changes the outcome:

- **Retaliation.** Retaliation is an override to *attack* (claim 10). If it comes first, *attack*
  is current when claim 16 is tested, so an actor that retaliates does not fall back (unless
  *attack* has the no-threat property), keeps the no-threat behaviour suspended, and returns to it
  at the revert. If the fall-back comes first, the rest behaviour is queued (claims 14 and 15
  queue without an immediate start), and the override then puts aside the queued behaviour with
  `quirks.override_suspends_queued` on (claim 10, Edge cases) while it stays queued, so the queued
  rest behaviour becomes current at the next per-tick start at which the actor is ready (claim 11)
  and ends the *attack* override early.
- **Scatter from damage.** Case 1 of claim 19 and S-0208 claim 15 need an actor with neither a
  target nor a destination. An infantry fall-back can give a destination (claim 13, guard-spot
  row), so whether the request is sent can depend on whether the fall-back came first.
- **A heal.** A negative hit (S-0302 claims 1 and 11) from a known attacker raises health and
  reports *unaffected*. "Takes damage" reads as a loss of health; "any damage that changes health"
  could include a raise.

Code: `combat::damage::reactions` (crates/deterministic/sim/src/combat/damage.rs).

## Questions

1. In one hit that an actor in a no-threat behaviour (not zombie) survives, and whose attacker it
   may retaliate against (S-0215 claim 18), does it fall back to rest (claim 16) before or after
   the retaliation override? If after, is claim 16 then judged on the behaviour current after the
   override?
2. Does the fall-back come before or after the scatter from damage of the same hit (S-0215 claim
   19 cases 1 and 2, S-0208 claim 15 requests A and B), and before or after fear (S-0107 claim
   16) and anger (S-0309 claim 16)?
3. Does a negative hit (a heal) from a known attacker that raises the health of an actor in a
   no-threat behaviour make it fall back to rest? Does a hit from a known attacker that lowers no
   health (*unaffected*, for example a 0 amount or an immune target) never do so?

## Possible observation

S-0215 scenario B5 with the *harmless* test file also giving *harmless* the retaliate property
and the subject armed, the damage coming from an enemy it may retaliate against (computer-
controlled owner, so the behaviour list of claim 18 does not stop it). Record the subject's three
slots and its target on the tick of the hit and the next ticks: after-retaliation order gives
*attack* current with *harmless* suspended and nothing queued; fall-back-first gives *attack*
current with the rest behaviour queued, then current at the next ready start. For question 3,
the same map with a healing weapon fired at the harmless unit after it has lost health.

## Current workaround

Interim rule (M8 seam S-0215 claim 16, `docs/m8-claims/seam-s0215-c16.md`): the fall-back comes
after the hit's S-0302 effects, anger and fear, and before retaliation; it is judged on the
behaviour current when the hit lands (as claim 16 reads, "with the no-threat property ... takes
damage"); retaliation then judges the state the fall-back leaves, with its own rules. The crush
response stays last (claim 39). Only a hit that lowers health and leaves the actor with health
above 0 triggers it; a heal does not. The scatter from damage (claim 19 for vehicles and
aircraft, S-0208 claim 15 for infantry) is not built in our engine's hit reactions, so question
2's scatter part has no effect yet.
