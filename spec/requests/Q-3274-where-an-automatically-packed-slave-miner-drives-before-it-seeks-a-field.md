---
id: Q-3274
title: Where an automatically packed slave miner drives before it seeks a field
status: open
related: [S-0402, S-0415]
answered-by: []
---

## Need

An issue report (#451) says that in a native game on an imported install, a deployed slave miner whose nearby ore has run out never moves on to another field. The lead expects it to pack up, drive to a site near another field and deploy there.

Our trace on synthetic data (claim test `claim_25_issue_451_automatic_pack_up_redeploys_one_cell_on` in `crates/deterministic/sim/tests/claims_s0402_slaves.rs`) finds that the engine follows S-0402 claims 22, 24, 25 and F7 and S-0415 claims 9 and 10 as written, and that together they never take the miner to the other field. The setup: a 3 × 3 deployed master on flat open ground, its near field used up, a reachable field beyond its short scan and within its long scan. Every link up to the pack-up works:

1. At a group update the field check finds no short-scan field (claim 22). The long scan finds the far field, the site search near it finds a site (claim 24), and F7 requests packing. The building's guard spot becomes its top-left cell *A* (claims 22, 25 and 28). In the trace this is tick 721.
2. The build-down ends and the vehicle appears at the return cell *A* + (1, 1), with *A* as its destination and *move* queued (S-0415 claim 10). Tick 759.
3. Its first group update recalls the slaves and enters field seeking, with no search (claims 22 and 25). Tick 761.
4. At the following update, tick 771, the vehicle is still driving the one diagonal cell to *A*: at the synthetic vehicle speed it arrives at tick 820. Claim 25 says "an existing destination is kept and makes the group travelling", so it does not seek a field.
5. On arrival it has no destination, and its next group update attempts deployment (claim 22; S-0415 claims 2 to 6). A 3 × 3 building deployed from cell *A* has its top-left cell at *A* − (1, 1).
6. The new building's field check again finds no short-scan field and packs up again (step 1).

Each cycle takes about 180 ticks and moves the miner one cell along map −*x* and −*y*, whatever the direction of the other field. Once no field is left, claim 25's comparison with cell (0, 0) still requests packing, so the cycle never ends. A player sees a miner that packs up and redeploys almost in place, again and again. In the trace, a much faster synthetic vehicle (it reaches *A* within 10 ticks of the second update) instead fails to deploy at *A* twice, enters field seeking and drives to the far field. So the outcome depends on whether the vehicle still holds *A* at its second group update.

S-0402 claim 25 says "A large foundation's return cell can differ from *A*, allowing movement towards *A* before field seeking", and its timing backing is medium ("conversion-to-movement timing and later group-update interaction"). It is not clear whether the original really redeploys at *A* when the drive there outlasts the first two group updates, or whether some rule we do not have takes it to the other field.

## Questions

1. After an automatic pack-up of a deployed slave miner whose foundation is larger than 2 × 2, does the vehicle drive to the old top-left cell *A* at all? If it does, and it is still on its way at the group update after the one that entered field seeking, does it keep travelling to *A*, as claim 25 says, or does it seek a field then?
2. When it reaches *A* without having sought a field, does it deploy there (claim 22's travelling row)? If so, is the new building's top-left cell *A* − (1, 1), one cell along −*x* and −*y* from the old one?
3. In an ordinary game, when a deployed miner's near ore runs out and another field lies within its long scan, does it end up deployed near that field? Through which states, and at which group updates does it pass?
4. When no field is left anywhere within the long scan, does a deployed miner keep packing up and redeploying (claim 25's comparison with cell (0, 0)), or does it stay deployed?

## Possible observation

S-0402 scenario H19 extended past the conversion: a deployed 3 × 3 slave miner whose near field is used up, with one field within its long scan in a direction other than map −*x*, −*y*, at a known vehicle speed. Record per tick the master's kind, its cell and top-left cell, its destination, its group state at each group update, and the slaves' positions, from the field check that requests packing until the miner has deployed twice more or 3000 ticks have passed. Repeat with no field left on the map.

## Current workaround

None in the code: the engine follows S-0402 claims 22, 24, 25 and F7 and S-0415 claims 9 and 10 as written, so such a miner redeploys one cell along −*x* and −*y* in each cycle. We change nothing until this is answered.
