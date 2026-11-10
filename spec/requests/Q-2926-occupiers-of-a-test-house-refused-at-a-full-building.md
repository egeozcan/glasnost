---
id: Q-2926
title: Occupiers of a computer-controlled house refused at a full building, and the hunt they lose
status: answered
related: [S-0215, S-0309, S-0508, M-0001, M-0002]
answered-by: [S-0508]
---

## Need

M8 step A6b built S-0309 claim 25 part (e)'s occupation alternative and the infantry rule of a
computer-controlled owner in S-0215 claims 27 (first infantry rule) and 23 (second row of the
infantry table). Both arena houses are test-only computer-controlled player houses (S-0215
Terms), so they now follow these original-only rules: an infantry unit of an occupier type finds
a building of the passive house that it may occupy (S-0508 claim 1's test), and when it processes
*hunt* or *attack* with that building as target it takes the building as destination and is
forced into *capture*.

In the arena games whose infantry starts beside the map's neutral garrisonable buildings (side B
of the pairings against E1 and E2), many infantry units go for the same few buildings at once.
The buildings fill, and the units that arrive later are refused (S-0508 claim 4: destination
cleared, scatter from the building's centre). As we read the specs, those units then never hunt
again:

- In *capture*, claim 28 row 4 (a scatter destination, no target) changes nothing; once the
  scatter cell is reached, row 2 (no destination) falls back to rest. Claim 28's hunt rule, which
  would queue *hunt* again for a computer-controlled owner, excludes occupier infantry types.
- The fall-back to rest of claim 14 gives a computer-controlled owner's armed infantry *area guard*
  (row 9) or *guard* (row 7).
- The *hunt* that the arena's placing step queued (S-0216 claim 8, S-0009 claim 41) was cleared by
  the forced *capture* (claim 9: the queued slot becomes none).

So these units rest in *area guard* near the buildings for the rest of the game. Where the other
group cannot or does not come to them, the game reaches the tick limit.

What we see, ours over seeds 0 to 63 after the change, against the record's unanimous side:

- M-0001 TELE against E2, side B: the record's column wins on all six seeds; ours ends with no
  winner on all 64 (before the change, ours matched the record). The E2 units stand in *area
  guard* or walk to their scatter cells; the TELE units hunt with no target.
- M-0002 INIT against E1, YURI against E1 and YURI against E2, side B, and M-0001 YTNK against E1,
  side B, moved away from the record's outcome at the same change.

The specs give one answer, and we built it. But the record's outcomes suggest that, in the
original, these infantry units keep fighting. Either the original's occupiers do not all go for
the buildings, or a refused occupier does get back to *hunt*. We cannot tell which from the specs,
so we ask before changing anything.

## Questions

1. When an occupier of a computer-controlled owner is refused at a building that has filled since
   it set out (S-0508 claim 4), does it return to *hunt* in the original? Claim 28's hunt rule
   excludes occupier types: does that exclusion hold for a unit whose entry was refused, or only
   for one that has entered?
2. Does the forced *capture* of claim 27's first infantry rule (and claim 23's second row) clear a
   *hunt* that is current (and one that is queued) so that it does not come back, as claim 9 reads?
3. Does S-0309 claim 25 part (e) let an occupier of a computer-controlled owner pick a building
   that other units of its house are already bound for, when the occupants and the units bound for
   it together exceed its capacity? Claim 1 of S-0508 counts only the occupants inside.
4. Is the search in *area guard* (S-0215 claim 26, S-0309 claim 1) the one that first picks the
   building for a unit whose *hunt* is still queued and not yet current, as happens in our arena
   at the first processing after placement?

## Possible observation

M-0001's set-up, TELE against E2, side B, one seed: for every E2 unit, record the tick it takes a
building as its target, the tick it enters one or is refused at it, and its behaviour 60 ticks
after a refusal (*hunt*, *area guard* or *guard*). The same for M-0002 YURI against E1, side B.

## Current workaround

None needed in code: the rules are built as S-0215 claims 9, 23, 27 and 28 and S-0309 claim 25 part (e) give
them (`process::computer_taker_target`, `acquire::candidate`), with tests in
`crates/deterministic/sim/tests/claims_s0309_c25e_occupy.rs`. The affected arena rows are listed
in `docs/m8-claims/a6b-arena-triage.md` as class (b), waiting on this request.
