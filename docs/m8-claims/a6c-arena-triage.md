# M8 step A6c: triage of the arena rows that fail after sim issues S20 to S22

Step A6c of [docs/m8-plan.md](../m8-plan.md) (*Exit*: "The arena failures fall, with every change
made against specs and records only"). On main e2c2efe (sim issues S20, S21 and S22 and render
perf RP5 merged) the arena fails 27 rows: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019
0 of 6, M-0020 0 of 25 and M-0021 10 of 50. This file gives each failing row (or group of rows) its
class, the cause we found, the claims that give it, what we did, and its verdict before and after.
Where A6b's class ([a6b-arena-triage.md](a6b-arena-triage.md)) still holds, the row says so in a
line. No stock value is copied here: causes are described by mechanism, and the numbers are our own
seed counts and the records' published outcomes (row, col or none).

The step's two issue fixes (#284 and #367, [sim-issues.md](sim-issues.md), *Batch A6c*) change no
arena game: every output file of the arena is byte-identical before and after, so *before* and
*after* agree in every row below. No row is class (a): in each group the first behaviour of ours
that decides the games is a claim as built.

Classes, as the step's brief defines them: **(a)** our departure from an approved claim (fixed,
with a test); **(b)** the specs are silent or ambiguous (request filed or named, behaviour kept);
**(c)** a harness or set-up difference (the request or record that covers it is named); **(d)**
within noise or the known spread gap (A1); **(e)** a record that a new oracle run would settle (a
wishlist entry in [oracle-wishlist.md](oracle-wishlist.md)).

## Method

As A6b's: `arena --seed-range ROW,COL,A|B,0-63` for every failing side, played on this step's build
(b1c588e), and `arena --watch` and `--trace` for the groups. *p* is our share of the stock's 6-of-6
outcome over seeds 0 to 63; the side is within noise when *p*⁶ ≥ 0.01, the records' α, that is 30
of 64 or more. A side below that is systematic.

## The neutral-bunker group (S-0309's open question from Q-2987)

Eight sides of six pairings are systematic and turn on the neutral bunkers of the arena ground: M-0001
CAOS–E1 (both sides) and CAOS–E2 (side B), M-0002 YURI–E1 and YURI–E2 (side B), as Q-2987 named
them, and M-0002 INIT–E1 and INIT–E2 (side A), which became systematic since A6b and intake 124.
Every arena house is a test-only computer-controlled player house with no building (S-0215 and
S-0309 *Terms*), so its infantry may take a passive house's building it may occupy as a target (S-0309
claim 25 (e)) and go for it in *capture* (S-0215 claim 27), and each of its checks gives the
house-wide hunt order, which puts its occupiers out with *hunt* next (S-0217 claims 1, 2 and 4;
S-0508 claim 13).

**Traces (seed 0).** M-0001 CAOS–E1, side B: Q-2987's chain is unchanged. E1 A#438 takes bunker
#258 as its target in its first *hunt* (tick 1), fires at it, is in *capture* from tick 16, goes
inside at tick 89, is put out at its house's check (tick 111), takes #258 again at its next *hunt*
(tick 112), and is in *capture* toward it when the CAOS gas turns it berserk (tick 154). M-0002
INIT–E2, side A, the largest move: every INIT takes bunker #258 or #259 as its target in its first
*hunt* (tick 1) and fires at it; at the next *hunt* processing (ticks 15 to 17) S-0215 claim 27,
judged on the target held when the processing begins, forces *capture*. The bunkers fill (by tick
73 for #259), and the INIT left outside drop the target. At their house's check (tick 110) the
INIT, inside and outside, get *hunt*, and the search takes the bunker again (tick 111), which
starts the round again. Meanwhile the E2 close in, and they win at tick 912 with some of their
own inside other bunkers. M-0002 INIT–E1, side A, plays the same way (*hunt* with #259 at ticks 1
and 110, *capture* at 15 and 124, again at 240 to 257).

**Claims.** Every step is a claim as built. The shots at the neutral bunker: no row of the firing
check refuses a passive house's building (S-0306 claim 23; S-0304 claim 2's drops are for illegal
and allied targets, which this is not). S-0309 (the spec that answers Q-2987) settles its question 1 (the occupiers do
come out at the checks) and keeps its question 2 open: *Claim 25, part e, after an eviction* says
such a unit is expected not to take the building it has just left as its first target after its
house's first checks (low), and that "claims 9, 17 and 25 do not give it". So the behaviour stays as
the claims give it; nothing was built from the expectation, and nothing was tuned toward the
records. The INIT rows are the same step, on the side whose occupiers are the row type. They are
named in a new wishlist entry, **A6c-1**, beside A6b-1 and A6b-3.

## The failing rows

Stock is the record's outcome on the failing side (6 of 6 there). *Ours, 64 seeds* counts our games
with that outcome on that side.

| # | record | pairing | fails | stock (failing side) | ours, 64 seeds | class | cause | claims | fix, request or wishlist | verdict before | verdict after |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | M-0001 | LTNK–E1 | side outcome, both sides | col | A col 52 (*p*⁶ = 0.29); B col 47 (*p*⁶ = 0.16) | (d) | A6b row 2 (side A): within noise, still. Side B now misses on seeds 0 to 5 too, within noise | S-0215 claim 19 case 2 (A6b row 1) | none | fail | fail |
| 2 | M-0001 | MIND–E1 | side outcome, both sides | row | A row 59 (*p*⁶ = 0.61); B row 63 (*p*⁶ = 0.91) | (d) | Intake 124's side A verdict (within noise) holds; side B within noise too | – | none | fail | fail |
| 3 | M-0001 | TELE–ROBO | side outcome, side B | none | none 46 (*p*⁶ = 0.14) | (d) | A6b row 5: within noise, still | – | none | fail | fail |
| 4 | M-0001 | TELE–BFRT | side outcome, side A | none | none 54 (*p*⁶ = 0.36) | (d) | A6b row 6: within noise, still | – | none | fail | fail |
| 5 | M-0001 | TELE–APOC | side outcome, side A | col | col 58 (*p*⁶ = 0.55) | (d) | A6b row 7: within noise, still | – | none | fail | fail |
| 6 | M-0001 | TELE–V3 | side outcome, side A | col | col 61 (*p*⁶ = 0.76) | (d) | **Cause changed.** A6b row 8 had side A systematic (col 0 of 64, class e). Sim issues S7 (#213: a seized unit carries a jumpjet state, S-0607 claim 5) and S10 moved the pairing to the record's outcome; side B passes, and side A misses only on seeds 0 to 5. Scored against M-0016's stock rows | S-0607 claims 5, 8, 12, 16 to 18 | A6b-4 annotated: no longer decides an arena row | fail | fail |
| 7 | M-0001 | CAOS–TTNK | side outcome, both sides | none | A none 63 (*p*⁶ = 0.91); B none 46 (*p*⁶ = 0.14) | (d) | A6b row 12 (side B) and intake 124 (both sides): within noise, still. Side A's one other outcome of 64 falls in seeds 0 to 5 | – | none | fail | fail |
| 8 | M-0001 | CAOS–E1 | win test; side outcome, side B | col | col 8 (row 56) | (b), (e) | The neutral-bunker group (above): put out at the checks, the E1 take the bunker again and meet the CAOS gas outside | S-0309 claim 25 (e); S-0215 claims 9, 27, 28; S-0508 claims 1, 4, 13; S-0217 claims 1, 2, 4 | S-0309's open question *Claim 25, part e, after an eviction* (Q-2987 question 2); wishlist A6b-1 | fail | fail |
| 9 | M-0001 | CAOS–E1 | side outcome, side A | col | col 5 (row 59) | (b), (e) | A6b row 14 and Q-2987: systematic, still; the same group | as row 8 | as row 8; wishlist A6b-3 | fail | fail |
| 10 | M-0001 | CAOS–E2 | win test; side outcome, side B | col | col 12 (row 50, none 2) | (b), (e) | As row 8 | as row 8 | as row 8 | fail | fail |
| 11 | M-0001 | CAOS–E2 | side outcome, side A | col | col 53 (*p*⁶ = 0.33) | (d) | A6b row 16: within noise, still. The pairing's win test fails with side B (row 10) | – | none | fail | fail |
| 12 | M-0001 | DISK–FV | win test; side outcome, both sides | col | A col 7 (row 57); B col 0 (row 64) | (b), (e) | A6b row 17's class and cause hold: the FV do not find a hovering disc again after their first targets die. Side B is now row in all 64 | S-0309 claims 7, 22; S-0215 claim 18 | **Q-2112** question 1 (open in S-0309); wishlist A6b-2 | fail | fail |
| 13 | M-0002 | INIT–E1 | side outcome, side A | row | row 20 (col 40, none 4) | (b), (e) | **Cause changed.** Intake 124 had row 38 of 64 (within noise; named in Q-2987); now systematic. The neutral-bunker group on the INIT side (trace above) | as row 8 | S-0309's open question, as row 8; wishlist **A6c-1** | fail | fail |
| 14 | M-0002 | INIT–E2 | side outcome, side A | row | row 2 (col 62) | (b), (e) | **Cause changed.** A6b row 19 had row 55 of 64 (d); now systematic. The neutral-bunker group on the INIT side (trace above) | as row 8 | as row 13; wishlist **A6c-1** | fail | fail |
| 15 | M-0002 | YURI–E1 | side outcome, side B | col | col 7 (row 57) | (b), (e) | A6b row 20 and Q-2987: systematic, still; the same group (the occupiers come out at tick 110) | as row 8; S-0508 claim 9 | as row 8 | fail | fail |
| 16 | M-0002 | YURI–E2 | side outcome, side B | col | col 28 (row 36; *p*⁶ = 0.007) | (b), (e) | Q-2987: systematic, at the edge (28 of 64 is below the bound of 30); the same group | as row 15 | as row 8 | fail | fail |
| 17 | M-0003 | MGTK–APOC | side outcome, side A | col | col 62 (*p*⁶ = 0.83) | (d) | New since A6b; side A misses on one of seeds 0 to 5 and on one other seed of 64 | – | none | pass | fail |
| 18 | M-0003 | ROBO–HTK | side outcome, both sides | col | A col 55 (*p*⁶ = 0.40); B col 33 (*p*⁶ = 0.019, near the edge) | (d) | A6b row 24 (side B): within noise, still; side A, new, within noise too | – | none | fail | fail |
| 19 | M-0003 | ROBO–V3 | side outcome, both sides | col | A col 59 (*p*⁶ = 0.61); B col 56 (*p*⁶ = 0.45) | (d) | **Cause changed.** Failing since sim issues S8 (#191: the rockets' aim at a hovering tank's heading point), Q-3066; S-0506 r43's nose-point test (S10) brought it back toward the record, and it is now within noise. Scored against M-0016's stock rows | S-0506 claims 14, 19 | Q-3066, answered by S-0506 r43 | fail | fail |
| 20 | M-0021 | part B: *c* ≥ 3 in every judged shot; the hits a judged shot owns; the reading-points row on a part B death's shot whose later hit names no attacker (3 rows) | rows | – | – | (b) | A6b row 26: S-0215's open question *Arena records and the scatter from damage* (low, from Q-2782), still | S-0215 claim 19, B6 f and g; S-0208 claim 15 | S-0215's open question | fail | fail |
| 21 | M-0021 | part B: the side left and the dead per run; five order-of-deaths rows (6 rows) | rows | – | – | (c) | A6b row 27: M-0021 *Reproducing it* (the test types, movement, speed and house factors kept local). One of A6b's six order-of-deaths rows now passes | S-0413 claim 1; S-0301; S-0302 | covered by M-0021's *Reproducing it* | fail | fail |
| 22 | M-0021 | seeds: every vehicle's first shot falls before the first difference | row | – | – | (c) | A6b row 28: Q-2597 was declined at M-0021's approval and stays open on purpose | – | **Q-2597** (open) | fail | fail |

Rows 1 to 19 cover 17 pairings, 25 failing sides: rows 8 and 9 are the two sides of CAOS–E1, and
rows 10 and 11 those of CAOS–E2. Rows 20 to 22 hold 3, 6 and 1 of M-0021's rows. That makes the 27
failing arena rows (M-0001 10, M-0002 4, M-0003 3, M-0021 10), before and after.

**Count per class** (ledger rows; arena rows in brackets): **(a)** 0 [0]; **(b)** 10 [6 pairings,
of which CAOS–E2 also has a (d) side, and 3 M-0021 rows], of which 9 rows (the neutral-bunker group
and DISK–FV) also have an **(e)** wishlist entry; **(c)** 2 [7 M-0021 rows]; **(d)** 11 [11
pairings, CAOS–E2's side A among them]. Ten pairings fail only within noise (rows 1 to 7, 17 to 19).

## Requests

None is filed: every gap found is already an open question of an approved spec (S-0309's *Claim 25,
part e, after an eviction* and *Q-2112 question 1*; S-0215's *Arena records and the scatter from
damage*) or a declined request kept open (Q-2597). Q-3170 to Q-3175 are unused.

Status of the requests the brief names: **Q-2926** and **Q-2927** are answered (S-0508 r19).
**Q-2951** and **Q-2987** carry `status: open` with `answered-by` filled in (S-0217 and S-0508; S-0309):
both are answered in the specs, and their status field is the lead's to set. **Q-3066** (row 19)
is the same: `status: open`, answered by S-0506 (r43). Q-2597 stays open on
purpose (row 22), and Q-2097 is on hold.

## Checks

- Base: main e2c2efe; this step's build b1c588e (the commits after it change docs only).
- Arena (all six records): every output file byte-identical before and after; 27 failing rows.
- Seed sweeps 1 to 16: M6c, M7a and M7b print the same lines as base (smallest minimum 0, 2 and
  the drain minimum 3).
- `glasnost-ai` with ignored tests, release: 362 passed before and after.
- AI quick lane (seeds 7 to 10, maps 0, 2 and 8, 36 games): every game ends on the same tick with
  the same winner as base, so none needed a trace.
- The sim gate with ignored tests: 17 passed before and after; no golden moved.
- No format change: save 150, AI game 127 and wire 12.
