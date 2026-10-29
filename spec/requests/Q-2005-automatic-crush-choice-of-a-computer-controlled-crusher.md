---
id: Q-2005
title: When a computer-controlled crusher chooses a crushable enemy by itself, and whether a berserk one does
status: open
related: [S-0210, S-0215, S-0309, S-0610, S-0302]
answered-by: []
---

## Need

S-0210 revision 22, claim 11, first list: a crusher may choose a crushable enemy as a target to
drive over "on the occasions on which the behaviour specs (S-0215) let a unit pick a target of
its own". For the original's computer-controlled owner, the table row allows it when the house
skill level is at least the auto-crush threshold and the difficulty index is not 2. When the
target is chosen, the crusher is given a move to the target's cell. Claim 11 then says: "Which
behaviours give the crusher the chance to choose, and so what counts as being busy with another
order, belongs to the behaviour specs."

S-0215 revision 76 does not name these occasions. Its only mention of crushing (claim 37, around
its line 2987) is about the crushing approaches for a target the unit already holds: the
original's attack-target variant and the omni-crusher variant. We build those two
(`crushing_approach` in `crates/deterministic/sim/src/approach.rs`). We do not build the
automatic choice of the first list for any house (`general.player_auto_crush` is read but not
used; `docs/m6-claims/step2-movement.md`, claim 11 row).

M-0011 needs that choice. It watches the arena game of CAOS against BFRT (side A, seeds 0 and 1).
There every drone but one per seed leaves play "with no damage event, while a grounded fortress that
had the drone as its target stood on the drone's position in the tick before". The record reads
this as crushing under S-0210 claim 4. Yet the same fortresses are berserk from ticks 50 to 58 to
the end, and "the fortresses' own fire goes at each other while they are berserk".

Our engine plays these games by the approved specs:

- From the first gas (ticks 51 to 58 in ours), every fortress is berserk. Its search covers the
  whole map. The search rejects every drone at S-0309 table C's *firing check* row, because a
  berserk unit never fires at a berserk-friendly type and never takes one through a search that
  asks the firing check (S-0610 claims 6 and 8). The drone type is berserk-friendly. So the
  fortresses take each other as targets, as the record says.
- A berserk hit sets off no reactions (S-0302 claim 10), so no fortress takes a drone as target by
  retaliation.
- In 6080 ticks no fortress takes a drone as target while berserk, so neither crushing approach
  ever applies. No drone leaves play, and both games run to the frame limit. In the record,
  BFRT wins at ticks 608 and 912.
- Before the gas (ticks 1 to 50), our fortresses do hold drones as targets, from their ordinary
  searches. They make the attack-target crushing approach (destination: the drone), but they
  crush none before they turn berserk.

A synthetic test of ours shows that once a berserk computer-controlled crusher holds a
berserk-friendly crushable target, the built attack-target variant drives it over the target,
with no shot at it. What is missing is how the fortress comes to hold the drone as its target
while berserk. The only route the specs name is claim 11's automatic choice, and its occasions
are not specified.

## Questions

1. For a crusher of a computer-controlled house that passes claim 11's table row, in the *hunt*
   behaviour (S-0215), on which of its turns may it make the automatic crushing choice? Is it in
   each turn in which it searches for a target (S-0309), or also in turns in which it already
   holds a target or a move destination? What counts as being busy with another order?
2. On such a turn, does the crushing choice come before the ordinary threat search, after it, or
   only when that search finds nothing? If both find something, which one wins?
3. When more than one crushable enemy is within the crush range, which one does it choose?
   (For example, the nearest, the first found in some fixed order, or the best by S-0309's score.)
4. Does the choice make the chosen object the crusher's target (glossary *target*), or does it
   only give the move to the object's cell, as claim 11 says? M-0011 sees the fortress "had the
   drone as its target".
5. Does a berserk crusher make this choice? If it does, can it choose a unit of a berserk-friendly
   type, which S-0610 claims 6 and 8 keep out of its firing and out of searches that ask the
   firing check? Can it also choose a unit of its own house, or only units of other houses?
6. While it drives to the chosen cell, does the crusher fire at anything, and does a later search
   in that time replace the choice?

## Possible observation

The M-0011 games (CAOS against BFRT, side A, seeds 0 and 1), recorded per tick for each
fortress: target, move destination and behaviour, in the 30 ticks before each drone leaves play.
This shows at which tick the drone became the fortress's target, whether a fortress dropped
another fortress as target for it, and whether the fortress had a destination then. A smaller
scenario in the style of S-0210 scenario K7 would also settle it: one computer-controlled crusher
made berserk by a gas weapon, with one crushable enemy of a berserk-friendly type within the
crush range and one enemy of another type further away.

## Current workaround

The built rules: no automatic crushing choice for any house. Both crushing approaches of claim 11
apply only to a target the crusher already holds. We built no interim rule. Each candidate rule
would be fitted to M-0011 alone. The arena's CAOS against BFRT pairing is reported as failing for
this reason (`docs/m8-claims/a1-arena-spread.md`, rank 7; `docs/m8-claims/intake38-M-0011.md`).
Our tests: `m0011_a_berserk_crusher_never_searches_out_a_berserk_friendly_enemy` and
`m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it`
(`crates/deterministic/sim/tests/claims_s0610.rs`).

Q-1712 question 3 (how the BFRT group destroys CAOS vehicles) is answered in part by M-0011.
This request asks for the rule behind it.
