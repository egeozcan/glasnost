---
id: Q-2035
title: How a seized vehicle of a computer-controlled house in hunt gets its own movement back after a drop
status: answered
related: [S-0607, S-0215, S-0309, S-0210, S-0502]
answered-by: [S-0607]
---

## Need

M-0015 watches the arena games of TELE against BFRT (M-0001, side A, seeds 0 and 1). There the
Magnetrons start many holds on the fortresses, and most end in a landing; the fortresses also
crush Magnetrons (M-0015). By S-0607
claim 3 a unit that is already seized cannot be seized again. By claim 10 a seized unit cannot
fire. So in the stock games each fortress gets its own movement back (S-0607 claim 20) after
most of its landings.

Our engine follows the approved specs, and it never gives a fortress its movement back. We see
this in all twelve of our games (both sides, seeds 0 to 5; `glasnost-headless arena --trace`, local
lane):

- At tick 1 each fortress is seized (S-0607 claim 5). Its house is a test-only
  computer-controlled player house (glossary), so it queues *hunt* (claim 6). *Hunt* is current
  from tick 2.
- Each lifter ends its grip by accepting a destination (claim 12, row 2). Each fortress is then
  dropped over clear land (claim 13), descends and is set down unhurt (claim 17, the *not
  blocked* row). It is still seized.
- Claim 20 restores a seized unit only when it falls back to rest (S-0215 claim 13). It says: "A
  set-down unit whose current behaviour stays *guard* ... and which never falls back to rest
  stays seized indefinitely."
- Each *hunt* processing runs the normal search (S-0215 claim 27). That search rejects every
  candidate at S-0309 table C's *firing check* row, because a seized unit's firing check answers
  *illegal* for every weapon and target (S-0607 claim 10). The hunter therefore has no target,
  and with no target *hunt* "only attempts its idle action". A vehicle has none (S-0107 claim 14 gives idle actions to infantry only), and the
  processing does not fall back to rest. No other rule we know of makes it fall back to rest:
  - a scatter request without a threat point queues no behaviour (S-0207 claim 16);
  - claim 9 refuses every new destination.
- So from tick 1 to the frame limit our fortresses stay seized. They take no target and fire
  no shot. Every later Magnetron hit meets a seized fortress and seizes nothing (claims 3 and 12).
  In each game we count 4 holds and about 3 100 such fire events. The record has 56 and 52 holds,
  and only 22 and 20 fire events at a fortress that was free and took no hold. Both groups end
  with all their units.

A synthetic test reproduces the built chain: `m0015_a_set_down_computer_unit_in_hunt_stays_seized_and_never_fires`
(`crates/deterministic/sim/tests/claims_s0607_m0015.rs`). In it, a computer-house unit is
dropped and set down. It has *hunt* current and an enemy in range. It stays seized for 600 ticks
and fires no shot. The same unit, never seized, fires.

S-0607 scenario L11 part b records the *hunt* that is queued at the seize. It records nothing
after the touchdown. Part c leaves a unit alone after touchdown and expects it to stay seized,
but it does not say which behaviour that unit has.

## Questions

1. A vehicle of a computer-controlled house (not passive, not a resource gatherer) is dropped by
   magnetic lift over a clear cell. It is set down unhurt (S-0607 claim 17), and *hunt* is current
   (claim 6). When does it get its own movement back? For example:
   - at the touchdown itself;
   - at a later *hunt* processing that finds no target;
   - after a fixed time;
   - only through some event that makes it fall back to rest. If so, which one?
2. Before that moment, does the *hunt* search of a seized unit find targets? In other words, is
   claim 10's *illegal* applied at S-0309 table C's firing-check row for a seized searcher, or not?
   If the search does find a target, what does the unit do with it while it is still seized?
3. Does the answer to question 1 depend on the behaviour? Does a seized set-down unit in *guard*
   (claim 20, scenario L11 part c) stay seized while one in *hunt* does not? Does it depend on the
   house being computer-controlled?
4. Once it has its own movement back, is it an ordinary hunter from then on? We would then
   expect it to search, fire, and make the attack-target crushing approach of S-0210 claim 11. M-0015
   counts the hits and the crushes on Magnetrons. Or does it first do something else, such as
   staying in place or queuing another behaviour?

## Possible observation

The M-0015 games, recorded per tick for each fortress from each landing to its next hold:
whether its own movement is back (for example, whether a move order would move it), its
current and queued behaviour, and its target. A smaller scenario would also settle it. Extend
S-0607 scenario L11 part b: drop an isolated vehicle of the computer opponent over clear land,
first with no enemy in reach and then with an armed enemy in its weapon's range. Leave it alone
for several hundred ticks, and record the tick at which it first moves or fires and its
behaviour at that tick. Run a human-owned control in *guard* (part c) beside it.

## Current workaround

The built rules: our engine restores a set-down seized unit only at a fall-back to rest
(claim 20), and a unit in *hunt* with no target never falls back to rest. We built no interim
rule. Any rule we picked would be fitted to M-0015 alone. The arena's TELE against BFRT pairing
is reported as failing for this reason (`docs/m8-claims/a1-arena-spread.md`, rank 5;
`docs/m8-claims/intake40-M-0015.md`). Q-1712 question 2 is answered in part by M-0015. Q-2005
(how a computer-controlled crusher chooses an enemy to drive over) may also bear on how a
restored fortress comes to crush a Magnetron.
