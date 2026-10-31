# M-0011: claim ledger, M8 step A3 intake 38

This intake takes in one measurement record, **M-0011** (Unit arena, CAOS against BFRT, the stock
replay watched per unit; measurement commit 275e576). It answers part of Q-1712 question 3. We
compared our CAOS against BFRT games (M-0001, side A, seeds 0 and 1) with it fact by fact. To do
that we added a per-unit trace to the arena tool. No spec revision comes with the record. The
specs it touches were read as they stand: S-0302 r39 claim 10, S-0610 r11 claims 4, 5, 6 and 8,
S-0309 r38 table C, S-0210 r22 claims 2 to 4 and 11, and S-0215 r76 claims 10, 27 and 37.

**Result: no departure from an approved claim was found, so no simulation code changed.** Where we
differ from the record, the difference comes from a rule the specs leave open. We filed a request
for it (Q-2005). A side finding has its own request (Q-2006).

## Fact by fact

Ours: the oracle arena as built (`glasnost-headless arena --trace`, local lane). It reads state
after each tick, as the record's tick convention does. The record's numbers come from M-0011.

| M-0011 fact | stock | ours | verdict |
|---|---|---|---|
| tick of the first gas on each fortress | 50 to 58 | 51 to 58 | agrees |
| remaining berserk time at the end of the first gas tick, the same for all fortresses | the same for every fortress | the same for every fortress | agrees (S-0302 claim 10 at distance 0) |
| later gas events that leave the remaining time at the full value | all but three; every one; 3 lower amounts not explained | every later berserk hit sets it to the full value | agrees with claim 10; the record leaves the 3 lower amounts unexplained, so we do not chase them |
| a further hit sets the time anew, longer or shorter | 1 shorter, 2 longer | tested synthetically, both ways | agrees (test below) |
| countdown with no gas event | falls by 1 each tick; 5 held ticks in seed 1 not explained | falls by 1 each tick | agrees (S-0610 claim 5); the record leaves the held ticks unexplained |
| health taken by a gas event | 0 | 0 | agrees (S-0302 claim 10: a berserk hit changes no health) |
| gas damage events on fortresses | 140 and 120 | no direct counterpart: our berserk hits make no hit event | not compared; we count the berserk entries and resets to the end tick instead: at least 249 in seed 0 and 304 in seed 1. That is more than the record, because our drones never leave play |
| fortresses that leave the berserk state | local | none | compared in the local lane only; follows from the drones row (below): our drones keep gassing the fortresses |
| fire of one fortress at another | 198; 275 | 249 by tick 608; 318 by tick 912 | agrees that it happens; the counts follow from the drones row |
| fortresses destroyed by that fire | local | compared in the local lane | the same |
| the fortresses' fire goes only at each other while they are berserk | yes | yes: no shot at a drone while berserk | agrees (S-0610 claims 6 and 8) |
| drones that leave play | every drone | none in 6080 ticks | **differs**: cause below, Q-2005 |
| of those, crushed: no damage event, a grounded fortress that had the drone as its target on its position | all but one | none: no berserk fortress ever takes a drone as target | **differs**: Q-2005 |
| of those, one "Super" event with no attacker | the other one | none | left alone: the record does not identify its source |
| end of game | BFRT wins at 608 and 912 | frame limit (6080), both seeds; no drone leaves play | follows from the drones row |

## Cause

At its first gas each fortress turns berserk and drops its target (S-0610 claim 4). From then on
it searches the whole map. The search rejects
every drone at the *firing check* row of S-0309 table C. A berserk unit never fires at a
berserk-friendly type, and never takes one through a search that asks the firing check (S-0610
claims 6 and 8). The drone type is berserk-friendly. This is the approved rule, and the record
agrees with it: the fortresses fire only at each other.

A berserk hit sets off no reactions (S-0302 claim 10), so no fortress takes a drone by
retaliation either. Before the gas (ticks 1 to 50) the fortresses do hold drones as targets and
make the attack-target crushing approach (S-0210 claim 11), but they crush none before they turn
berserk.

The only rule in the specs that would let a berserk fortress hold a drone as target without the
firing check is S-0210 claim 11's automatic crushing choice (its first list, under the
computer-controlled table row, which the arena houses pass). Its occasions "belong to the behaviour
specs", and S-0215 does not name them. Also open: which enemy it chooses, whether the choice sets
the target or only a move, and whether a berserk crusher makes it and can choose a
berserk-friendly enemy.

We have never built this choice (`general.player_auto_crush` is unused;
[step 2 ledger](../m6-claims/step2-movement.md), claim 11 row). We built no interim rule, because
any rule we picked would be fitted to M-0011 alone: **Q-2005**.

Once a berserk computer-controlled crusher holds such a target, our built attack-target variant
drives it over the target with no shot at it. A synthetic test shows this. So building Q-2005's
answer should be enough for the drones to leave play as the record has them.

## Claims

| claim | status | tests or reason |
|---|---|---|
| S-0302 claim 10 (a further berserk hit sets the remaining time anew, longer or shorter) | tested | `m0011_a_further_berserk_hit_sets_the_time_anew_longer_or_shorter` (`claims_s0610.rs`): a long hit, then a short one that shortens the time, then a long one that lengthens it again |
| S-0610 claims 6 and 8, S-0309 table C *firing check* (a berserk crusher's search never takes a berserk-friendly enemy, even one it could crush) | tested | `m0011_a_berserk_crusher_never_searches_out_a_berserk_friendly_enemy` |
| S-0210 claims 3, 4 and 11, the attack-target variant (a berserk computer-controlled crusher holding a berserk-friendly crushable target drives over it: removed, no damage, the crush sound, no shot at it before the crush) | tested | `m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it`. Since intake 52b the test counts the crush tick too, and the crusher fires nothing there (S-0309 claim 27, Q-2006; [intake 52b ledger](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md)) |
| S-0210 claim 11, the automatic choice of the first list (occasions, which enemy, berserk interplay) | answered, built | Q-2005, answered by S-0210 r29 and S-0215 r88 claim 39: the choice is now the *crush response* to a hit, built in intake 52a ([ledger](intake52a-S-0210-S-0215-S-0216.md)) |
| what a ground unit does with a held target that has just left play | built in intake 52b | Q-2006 (below), answered by S-0309 r51 claims 27 and 28, built and tested in [intake 52b](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md) |

**Side finding (Q-2006).** In the third test the crusher crushes its held target at the cell
centre. In the same tick, its firing step still holds the removed object as its target. Our firing
check treats a target that is no longer in play as no target and answers *can fire*, so the
crusher fires one shot. No spec says what a unit does with a held target that has just left play,
or what the firing check answers for it. We filed **Q-2006** and changed nothing. No arena game
reaches this today. S-0309 r51 claim 27 answers Q-2006: every holder drops the object inside
the crush and fires nothing at it in that tick (intake 52b).

## Tool

`glasnost-headless arena --record <ID> --trace ROW,COL,A|B,SEED` (`--trace-summary` for the
per-unit totals only; `crates/tools/src/oracle_trace.rs`). It plays one game as the arena does and
prints each unit's changes tick by tick: health, behaviour, target, destination, berserk entries,
resets and exits, shots at its own or the other group, and how it leaves play (with who stood on
its cell the tick before and the crush sounds of that tick). It is tooling only and reads state
after each tick. The trace files stay in the local lane: they hold the stock berserk duration.

## Arena and sweeps

No simulation code changed, so the arena CSVs before and after are the same byte for byte (local
lane, `local-lane/m8/a3-38/before` and `after`; both played on main 426d38d, before the rebase onto
intake 36):

- M-0001: 30 of 96 pairings fail (win 5, row left 22, column left 13); 151 of 172 side-unanimous
  stock outcomes reproduced.
- M-0002: 13 of 24 fail (4, 6, 3); 43 of 47.
- M-0003: 2 of 35 fail (0, 2, 0); 61 of 64.

CAOS against BFRT still fails, now with its cause pinned to Q-2005. (Intake 52a builds the answer, the crush response of S-0215 claim 39; see [its ledger](intake52a-S-0210-S-0215-S-0216.md) for the arena after it.)

After the rebase onto main badb202, the M6c, M7a and M7b sweeps (seeds 1 to 16) all pass, and with
`GLASNOST_PLAY_FIXES=1` they print the same lines as the reference. The sim gate's goldens are
unchanged. No saved or hashed state was added: save
format 83 and AI game format 45 stay as they were.
