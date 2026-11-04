---
id: Q-2426
title: Which answers part (e) passes over with quirks.moment_reason_attack_cursor off
status: answered
related: [S-0214, S-0301, S-0607, S-0516, S-0405, S-0506]
answered-by: [S-0214]
---

## Need

S-0214 r78, Edge cases, *Attack offered over a reason of the moment*: with
`quirks.moment_reason_attack_cursor` off, "part (e) asks only whether a reason that gives
*illegal* holds, at any rank, passing over every reason of the moment (the *rearming* answers of
ranks 4 and 5, out of ammunition, *cloaked* and the lower ranks)". Claim 6 (e) names four early
*rearming* answers of rank 4 (a materialising firer, a spawner weapon none of whose spawned
aircraft is alive, a target the firer's temporal beam holds, an own building's pending delayed
shot). S-0301 claim 9's rank 4 has two more *rearming* answers that S-0214 does not name: the
firer's own gripped unit (S-0607 claim 11) and the magnetic-beam firer with a gripped unit and a
live wave against another target that is not a building. Ranks 1 to 4 also hold answers that are
neither *illegal* nor *rearming*: *busy* (rank 2), and the *impossible* answers of rank 4 (the
hidden-target refusal of S-0516 claim 24, the held spawner of S-0506 claim 10, the other weapon's
live effect of S-0301 claim 5, a building's own *impossible* of S-0405 table T1). Rank 11 has
*must deploy*.

The words "only whether a reason that gives *illegal* holds, at any rank" read as: every answer
that is not *illegal* is passed over, so that an *illegal* reason ranked below it still fails part
(e). The parenthesis lists only *rearming*, out of ammunition, *cloaked* and the lower ranks. The
two readings differ, with the switch off, for:

- a lifter over its own gripped unit held in the air, whose weapon cannot hit a target in the air
  (the grip *rearming* hides the projectile refusal in the on position);
- a target hidden from the actor's house that a later test rules illegal, for example one its
  warhead's versus value of zero cannot affect (S-0214 claim 6 (e) says the hidden target's
  *impossible* is not *illegal*);
- a vehicle that is launching spawned aircraft (*busy*), and a deploy-to-fire healer vehicle
  whose cell fails the deploy test (*must deploy* above rank 12's healer *illegal*).

Where: `combat::check::illegal_at_any_rank` (S-0214 claim 6 (e) with the switch off), called from
`targeting::attack_rule`.

## Questions

1. With `quirks.moment_reason_attack_cursor` off, does part (e) pass over the gripped-unit
   *rearming* and the magnetic-beam *rearming* of rank 4, so that a later *illegal* reason (for
   example the projectile refusal against the gripped unit in the air) fails part (e)?
2. With the switch off, does part (e) pass over the *impossible* answers of rank 4 (the hidden
   target, the held spawner, the other weapon's live effect, a building's own *impossible*), the
   *busy* of rank 2 and the *must deploy* of rank 11, so that a later *illegal* reason fails part
   (e)?

## Possible observation

None: the off position is not the original's behaviour; this asks what the switch covers.

## Current workaround

Interim: the literal reading. With the switch off, part (e) fails when any reason that gives
*illegal* holds, at any rank, and passes over every other answer: *busy*, every *rearming* of
ranks 4 and 5 (the grip and magnetic-beam answers included), every *impossible* of rank 4, out of
ammunition, *cloaked*, *must deploy* and the lower ranks. Two cases stop short: with no weapon in
the slot nothing further is asked (part (e) already holds for an actor with no weapon), and the
range test is not asked (part (d) is the range part). The on position (the default) is
unaffected.
