# M8 step A6: triage of the arena's failing scored pairings

Step A6 of [docs/m8-plan.md](../m8-plan.md). Under intake 47's scoring
([intake47-arena-restated.md](intake47-arena-restated.md)), 17 of the 132 scored pairings of
M-0001 to M-0003 fail. This file gives, for each one, its class, the cause we found, the claims
that give it, what we did, and its verdict before and after. No stock value is copied here: the
causes are described by mechanism, and the numbers are our own seed counts and the records'
published outcomes.

Classes, as the step's brief defines them: **(a)** our departure from an approved claim (fixed,
with a test); **(b)** the specs are silent or ambiguous (request filed, behaviour kept); **(c)** a
harness or setup difference that Q-2096 to Q-2099 cover; **(d)** within noise.

## Method

- **Watching.** `arena --watch`, `--trace` and `--trace-summary` in the local lane, with
  throwaway probes in the simulation (environment-gated prints, never committed) where the trace
  did not show the deciding rule.
- **Noise.** A scored pairing fails its *side outcome* when the stock games of one side all gave
  one outcome (6 of 6) and ours did not. `arena --seed-range ROW,COL,A|B,0-63` (new,
  `oracle::seed_range`) plays that side over seeds 0 to 63 (seeds 0 to 5 are the scored games;
  every one of them reproduced the arena's per-seed outcome). Let *p* be our share of the stock's
  outcome over the 64 seeds. If the stock's true share were *p*, it would give 6 of 6 with
  probability *p*⁶. We call the failure **within noise** when *p*⁶ ≥ 0.01, the records' α: then
  the stock's 6 of 6 is no evidence, at that level, that the stock differs from ours. When
  *p*⁶ < 0.01 the difference is systematic and we look for its cause.
- **Win test failures** (the two-proportion test on row wins) are all systematic here: ours gives
  the same winner on all 64 seeds of the failing sides.

## The failing pairings

Stock and ours on seeds 0 to 5 are from the arena table. *Ours, 64 seeds* counts our games with
the stock's outcome on the failing side.

| # | record | pairing | fails | stock (failing side) | ours, 64 seeds | class | cause | claims | fix or request | verdict before | verdict after |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | M-0001 | DISK–APOC | win test, both sides | col 6 of 6 on each side | col 0 of 64 on each side (row 64) | (b) | Every missile the column fires at a hovering disc detonates at its launcher in its first turn, so the discs take no damage. The traced fire point is the launcher's own position at the surface (S-0313 claim 1 with a 0, 0, 0 muzzle position, the Import mapping's value for an absent key), and a steering projectile at height 0 detonates at the start of its turn (S-0306 T1, row *h* ≤ 0) | S-0313 claim 1 and Import mapping; S-0306 claim 19, T1 | **Q-2111**, answered by S-0306 r45 claim 8 (a start point not above the terrain is raised 1 lepton above it) and S-0313 r37 claim 1a; built in intake 51 ([intake51-S-0306-S-0313.md](intake51-S-0306-S-0313.md)) | fail | fail (A6); pass after intake 51: col 64 of 64 on each side, every game won by the column at frame 912 |
| 2 | M-0001 | DISK–FV | win test, both sides | col 6 of 6 on each side | col 6 (A) and 0 (B) of 64 | (b) | The FV take targets at tick 1 while the discs stand on the ground and hold them as the discs rise. After a target dies, no FV takes another one: the map-wide search never finds a hovering jumpjet, and retaliation does not happen for these hunters | S-0309 claim 7 (`quirks.hunt_ground_layer_only`); S-0215 claim 18 | **Q-2112** (the open item "hunting ground units against hovering jumpjets") | fail | fail |
| 3 | M-0001 | CAOS–MGTK | side outcome, both sides | none 6 of 6 on each side | none 63 (A) and 61 (B) of 64 | (d) | Seeds 0 to 5 hold 3 of our 4 row wins in 128 games: berserk MGTKs destroy each other to the last one. *p*⁶ = 0.91 and 0.75 | – | none | fail | fail |
| 4 | M-0001 | TELE–BFRT | side outcome, side A | none 6 of 6 | none 16 of 64 (*p*⁶ < 0.001) | (b) | Each gripped fortress comes inside the minimum range of its Magnetron's weapon before it arrives. The Magnetron's approach step (range test with minimum range) then searches an approach position, finds one about its weapon's range away, accepts it, and so ends the grip. The Magnetrons back away at every hold and reach the lower ground beside the plateau. There, fortresses are dropped on cells with a tree (claim 16's hit does not hurt it, S-0302 claim 20) or rock land, find the cell blocked and are killed by the falling damage. M-0015 records no such landing damage | S-0215 claim 37 (b), (d), (f); S-0307 claims 9, 15, 16; S-0607 claims 12 (row 2), 13, 16 to 18; S-0302 claim 20 | **Q-2113**, answered by S-0607 r18 (intake 49) and S-0215 r88 claim 37 (c) (intake 52a) | fail | fail |
| 5 | M-0001 | TELE–APOC | side outcome, side A | col 6 of 6 | col 45 of 64 (*p*⁶ = 0.12) | (d) | – | – | none | fail | fail |
| 6 | M-0001 | LTNK–APOC | side outcome, side B | col 6 of 6 | col 51 of 64 (*p*⁶ = 0.26) | (d) | – | – | none | fail | fail; **pass** after intake 61 (ours 6/6/0 as the stock; [ledger](intake61-S-0607-S-0306.md)) |
| 7 | M-0001 | MIND–E1 | side outcome, side B | row 6 of 6 | row 62 of 64 (*p*⁶ = 0.83) | (d) | – | – | none | fail | fail |
| 8 | M-0001 | CAOS–GGI | side outcome, side A | none 6 of 6 | none 61 of 64 (*p*⁶ = 0.75) | (d) | – | – | none | fail | fail |
| 9 | M-0001 | CAOS–SHK | side outcome, side B | none 6 of 6 | none 61 of 64 (*p*⁶ = 0.75) | (d) | – | – | none | fail | fail |
| 10 | M-0001 | CAOS–E1 | side outcome, side B | col 6 of 6 | col 61 of 64 (*p*⁶ = 0.75) | (d) | – | – | none | fail | fail |
| 11 | M-0001 | LTNK–ROBO | win test | row 5 of 12, the rest none | row 64 of 64 on each side | (b) | The ROBO are disabled (no buildings) and our hunters destroy all of them in every game; in the stock games a seed-dependent part survives | S-0405 claim 13; S-0309; S-0215 | **Q-1710**, answered by S-0405 r47 (claims 13 and 20) and M-0005: both claims were already as built and are tested in intake 52b ([ledger](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md)); M-0005 shows that every stock robot tank is disabled too, and that the stock games end in a stall whose shells land outside every robot tank's damage reach. The stall is open: **Q-2157** (where the shells land) and **Q-2158** (hunters that fire without a projectile) | fail | fail (A6); fail after intake 52b: ours 12 row wins of 12, stock 5 and 7 without a winner; **Q-2157 and Q-2158 answered by S-0306 r53** (intake 61, [ledger](intake61-S-0607-S-0306.md): a switched-off hover is led, and a shell with a firer flies on below the ground): **pass** after intake 61, ours 12 of 12 games without a winner (stock 5 row and 7 none), *p* = 0.012 |
| 12 | M-0002 | INIT–E2 | side outcome, side A | row 6 of 6 | row 38 of 64 (*p*⁶ = 0.044) | (d) | Near the edge of the test. 11 of the side-A INIT are placed off their stated cells (Q-2096, Q-2097), which may move the share; we do not count it as (c). The placement is S-0216 r37 claim 8's since intake 71 | – | none | fail | fail |
| 13 | M-0002 | YURI–GGI | side outcome, side B | row 6 of 6 | row 63 of 64 (*p*⁶ = 0.91) | (d) | – | – | none | fail | fail |
| 14 | M-0003 | MTNK–APOC | side outcome, side B | col 6 of 6 | col 51 of 64 (*p*⁶ = 0.26) | (d) | – | – | none | fail | fail; **pass** after intake 61 (ours 6/6/0 as the stock; [ledger](intake61-S-0607-S-0306.md)) |
| 15 | M-0003 | MTNK–HTNK | side outcome, side B | col 6 of 6 | col 63 of 64 (*p*⁶ = 0.91) | (d) | – | – | none | fail | fail |
| 16 | M-0003 | MGTK–APOC | side outcome, side A | col 6 of 6 | col 61 of 64 (*p*⁶ = 0.75) | (d) | – | – | none | fail | fail |
| 17 | M-0003 | ROBO–APOC | side outcome, side A | none 6 of 6 | none 0 of 64 (col 64) | (b) | As row 11: the disabled ROBO are all destroyed | as row 11 | **Q-1710** (answered, as row 11); the stall: **Q-2157** | fail | fail (A6); fail after intake 52b: ours col 12 of 12 games (stock none 11 of 12); **pass** after intake 61 (Q-2157 answered, as row 11): ours none 12 of 12 |
| 18 | M-0003 | ROBO–HTK | side outcome, both sides (seeds 0 to 5: A none 1, B none 2) | col 6 of 6 on each side | side B col 23 of 64 (none 41), side A col 59 of 64 | (b) | New after intake 61 (S-0306 r53): the HTK leave the disabled robot tanks of the rear column standing | S-0306 claim 3 b, T1 | Q-2216 (answered by S-0306 r62, intake 65: as built; P45 part a is the lead's record) | pass | fail after intake 61 ([ledger](intake61-S-0607-S-0306.md)); unchanged after intake 65 ([ledger](intake65-S-0306.md)) |

Totals: (a) 0, (b) 5 (four requests, one open request reused), (c) 0, (d) 12.

## Found on the way

- **A crushed unit stays a target** (**Q-2114**). In TELE–BFRT, side A, seed 0, a fortress crushes
  a Magnetron (S-0210 claim 4). Two other fortresses keep it as their target for some more ticks,
  and one fires at it: the shot detonates at the world origin. A kill clears the targets of every
  holder at once (`combat::death::mark_killed`); a removal without death does that only for
  aircraft (S-0501 claim 32). No approved claim we found covers ground holders. No verdict
  depends on it. Since intake 52b this is S-0309 r51 claim 27's (Q-2006): every holder drops a
  crushed object inside the crush, with the timer draw, and fires nothing at it in that tick
  ([intake 52b ledger](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md)).

## Requests

| request | question | interim |
|---|---|---|
| [Q-2111](../../spec/requests/Q-2111-steering-missile-fired-from-a-muzzle-at-the-surface.md) | does a steering projectile fired from a muzzle at the surface detonate in its first turn; is an absent muzzle position the zero triple | answered by S-0306 r45 and S-0313 r37, built in intake 51: a projectile starts 1 lepton above the terrain when placed at or below it; an absent elite muzzle position takes the same slot's non-elite one (S-0313 claim 1a) |
| [Q-2112](../../spec/requests/Q-2112-hunting-ground-units-against-hovering-jumpjets.md) | how ground hunters find a hovering jumpjet again after losing their target | answered in part by S-0309 r53, built in [intake 52b](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md): the occasions are a jumpjet picked in the ground layer and still held, the limited-scan mark after a failed approach (claim 22, now set), and a give-up of the move, which now drops the target (claim 7). Which of them operates in the DISK–FV games is open (S-0309 Open questions, Q-2112 question 1) |
| [Q-2113](../../spec/requests/Q-2113-a-lifter-that-pulls-its-unit-inside-its-minimum-range.md) | does a lifter take the approach step against its own gripped unit inside its minimum range; what ends the stock holds | answered by S-0607 r18 (built in [intake 49](intake49-S-0607.md)) and, on the behaviour side, S-0215 r88 claim 37 part c (built in [intake 52a](intake52a-S-0210-S-0215-S-0216.md): a seized unit whose target is its lifter now takes the step; Q-2151 follows from it) |
| [Q-2114](../../spec/requests/Q-2114-targets-of-a-crushed-unit.md) | what happens to the targets of objects whose target is crushed | answered by S-0309 r53 claim 27 and S-0210 r31 claim 4, built in intake 52b: every holder drops it inside the crush, in creation order, with each removed object's draws before the next one's, and a crushed vehicle's two animation picks before its holders' draws. The request file is the lead's to close |

Q-2115 is unused.

## Checks

- No simulation change: this step commits only the `--seed-range` option of the oracle tooling,
  the requests and these notes. The save format stays 85 and the AI game format 47; no golden was
  regenerated.
- Arena before (main 99bae28) and after (this branch on main 138e768): 17 of 132 scored pairings
  failing in both, the same pairings. See the step's report for the table and the seed sweeps.
