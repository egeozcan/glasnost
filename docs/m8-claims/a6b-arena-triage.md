# M8 step A6b: triage of the arena rows that still fail

Step A6b of [docs/m8-plan.md](../m8-plan.md) (*Exit*: "The arena failures fall, with every change
made against specs and records only"). On main e79d86f (intakes up to 122) the arena fails 28 rows:
M-0001 12 of 96, M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 1 of 25 and M-0021 11 of 50.
This file gives each failing row (or group of rows) its class, the cause we found, the claims that
give it, what we did, and its verdict before and after. A6 ([a6-arena-triage.md](a6-arena-triage.md))
triaged 17 pairings at intake 47; where its class still holds, the row says so in a line. No stock
value is copied here: causes are described by mechanism, and the numbers are our own seed counts
and the records' published outcomes.

Classes, as the step's brief defines them: **(a)** our departure from an approved claim (fixed,
with a test); **(b)** the specs are silent or ambiguous (request filed or named, behaviour kept);
**(c)** a harness or set-up difference (the request or record that covers it is named); **(d)**
within noise or the known spread gap (A1); **(e)** a record that a new oracle run would settle (a
wishlist entry in [oracle-wishlist.md](oracle-wishlist.md)).

**Since intake 123** ([intake123.md](intake123.md)): S-0508 r19 answered Q-2926 and Q-2927. It
confirms the chain of rows 9, 10, 18, 20 and 21 (and row 4) as the original's, so for the
systematic rows (10, 18, 20) the record and the spec now disagree; the measurement wins, and
**Q-2951** names them. Intake 123's changes (the assaulter's "may occupy" and capture's hunt rule)
leave every arena game as it was; the 64-seed counts of those six rows are unchanged.

## Method

As A6's ([a6-arena-triage.md](a6-arena-triage.md), *Method*): `arena --watch` and `--trace-summary`
in the local lane, throwaway prints in the simulation where the trace did not show the deciding rule
(never committed), and `arena --seed-range ROW,COL,A|B,0-63` for every side-outcome failure. *p* is
our share of the stock's 6-of-6 outcome over seeds 0 to 63; the row is within noise when *p*⁶ ≥ 0.01,
the records' α. The seed ranges were played on this branch's build (after the fix below). The
verdicts *before* are main e79d86f's arena; *after* is this branch's.

## The fix (class a)

The houses of the arena are test-only computer-controlled player houses (S-0215 *Terms*, *Test-only
computer-controlled player house*; S-0309 *Terms* likewise): they follow every original-only row
and condition for computer-controlled owners. Two such rows for infantry that can take a building
were not built for them:

- **S-0309 claim 25 part (e)**, the house exception of the candidate test: an infantry candidate of
  a computer-controlled owner may take a building of a passive house as its target when it may
  capture it **or occupy it**. Ours had the capture part only. The occupation alternative now uses
  the garrison entry test (`garrison::may_enter`, S-0508 claim 1) in `acquire::candidate`.
- **S-0215 claim 27, first infantry rule** (and claim 23, the infantry table's row 2 for *attack*):
  a computer-controlled infantry actor whose target is a building it may infiltrate, occupy or
  assault takes the building as its destination and makes *capture* current (forced), with the
  first *capture* processing 1 tick later, no targeting-timer restart and no draw. Ours went on
  hunting or attacking. `process::computer_taker_target` and `process::go_for_building` now do it
  in `process_hunt` and `process_attack`.

Our own AI's houses are human-controlled (S-0215 *Terms*), so neither row reaches them; the neutral
and special houses do get them, as the Terms say. Tests:
`crates/deterministic/sim/tests/claims_s0309_c25e_occupy.rs` (8 tests: the computer occupier finds
a passive garrisonable building and a human one does not; a computer searcher without an occupation
is still refused; in *hunt* and in *attack* the computer occupier goes for the building and enters
it, the human one keeps hunting or attacking).

**What it moved.** In M-0001 CAOS–E1 and CAOS–E2, side B, our E1 and E2 now enter the neutral
bunkers of the arena ground, and those sides give the record's published outcome; MIND–E1 side B
passes too. But six rows
of E1 and E2 against other types now fail on side B (TELE–E1, TELE–E2, YTNK–E1; INIT–E1, YURI–E1,
YURI–E2): the arena goes from 28 to **33** failing rows. In the watched games (TELE–E2 and YURI–E1,
side B, seed 0) the mechanism is one chain of approved rules: the queued *hunt* starts, its search
takes a bunker as target, claim 27 forces *capture* (which clears the queued slot, claim 9), the
bunker is already full so the entry is refused with a scatter (S-0508 claim 4), the *capture*
processing falls back to rest (claim 28 row 4, then row 2; claim 14 row 9 queues *area guard*), and
claim 28's hunt rule leaves occupier types out, so the unit never hunts again. The specs do not say
whether that is what the original does at a full building: **Q-2926** asks (below). No rule was
changed toward the lane; the rows stay as the claims give them.

## The failing rows

Stock is the record's outcome on the failing side (6 of 6 there). *Ours, 64 seeds* counts our games
with that outcome on that side, on this branch.

| # | record | pairing | fails | stock (failing side) | ours, 64 seeds | class | cause | claims | fix, request or wishlist | verdict before | verdict after |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | M-0001 | LTNK–APOC | side outcome, side A | row | row 63 of 64 (*p*⁶ = 0.91) | (d) | New since the scatter-from-damage build; Q-2782's answer kept it, and S-0215's open question *Arena records and the scatter from damage* (low) names it | S-0215 claim 19 case 2 | none | fail | fail |
| 2 | M-0001 | LTNK–E1 | side outcome, side A | col | col 57 of 64 (*p*⁶ = 0.50) | (d) | As row 1 | as row 1 | none | fail | fail |
| 3 | M-0001 | MIND–E1 | side outcome, side B | row | – | (d) | A6 row 7: within noise. Passes with the fix | – | none | fail | **pass** |
| 4 | M-0001 | YTNK–E1 | side outcome, side B | row | row 41 of 64 (*p*⁶ = 0.069) | (d) | New with the fix (the E1 go for the bunkers); within noise | S-0309 claim 25 (e), S-0215 claim 27 | none | pass | fail |
| 5 | M-0001 | TELE–ROBO | side outcome, side B | none | none 51 of 64 (*p*⁶ = 0.26) | (d) | – | – | none | fail | fail |
| 6 | M-0001 | TELE–BFRT | side outcome, side A | none | none 53 of 64 (*p*⁶ = 0.32) | (d) | A6 row 4 was (b), Q-2113, answered and built (intakes 49 and 52a); what is left is noise | – | none | fail | fail |
| 7 | M-0001 | TELE–APOC | side outcome, side A | col | col 52 of 64 (*p*⁶ = 0.29) | (d) | A6 row 5: within noise, still | – | none | fail | fail |
| 8 | M-0001 | TELE–V3 | side outcome, both sides | col | side A col 0 of 64 (none 62); side B col 38 of 64 (none 26; *p*⁶ = 0.044) | (e) | Side B is within noise; side A is systematic. The V3s, set down and restored (S-0607 r14), are seized again and again, and most games reach the sample limit without a winner ([arena.md](../dev/arena.md), intakes 45 and 49: not yet explained). Scored against M-0016's stock rows. No approved claim was found to be departed from; a hold record for this pairing would show which step differs | S-0607 claims 8, 12, 16 to 18; S-0215 claim 37 | wishlist A6b-4: M-0015's hold recording for TELE against V3 | fail | fail |
| 9 | M-0001 | TELE–E1 | side outcome, side B | col | col 31 of 64 (*p*⁶ = 0.013) | (d) | New with the fix; at the edge of the test (31 of 64). The E1 that find a full bunker stop hunting, as row 10 | S-0309 claim 25 (e), S-0215 claims 27, 28 | Q-2926 (related) | pass | fail |
| 10 | M-0001 | TELE–E2 | side outcome, side B | col | col 0 of 64 (none 64) | (b) | New with the fix. The E2 take the full bunkers as targets, are refused at entry and scatter, fall back to rest and never hunt again (*The fix*, above); every game ends without a winner | S-0309 claim 25 (e); S-0215 claims 9, 14, 27, 28; S-0508 claim 4 | **Q-2926**; wishlist A6b-1: S-0215 B11 (b) at a full building with S-0508 G1's refusal | pass | fail |
| 11 | M-0001 | CAOS–APOC | side outcome, side B | none | none 62 of 64 (*p*⁶ = 0.83) | (d) | – | – | none | fail | fail |
| 12 | M-0001 | CAOS–TTNK | side outcome, side B | none | none 51 of 64 (*p*⁶ = 0.26) | (d) | – | – | none | fail | fail |
| 13 | M-0001 | CAOS–E1 | side outcome, side B | col | – | (a) | Ours had neither the occupation alternative of the house exception nor claim 27's infantry rule for the test houses, so the E1 never went for the bunkers (M-0001 and M-0017 put four neutral bunkers in play from tick 0). With the rows built, side B gives the record's published outcome | S-0309 claim 25 (e); S-0215 claims 23, 27 | built, `claims_s0309_c25e_occupy.rs` | fail | side B passes; the pairing still fails on side A (row 14) |
| 14 | M-0001 | CAOS–E1 | side outcome, side A | col | col 21 of 64 (row 32, none 11; *p*⁶ = 0.001) | (e) | Systematic, before and after the fix. In the watched game (seed 0) the E1 lose nearly all their units in the first sample period and CAOS wins; we found no step of ours that departs from an approved claim. Whether the stock E1 on this side enter a building, and when, is what a trace would show | S-0309 claim 25 (e); S-0215 claim 27; S-0508 claims 1, 4 | wishlist A6b-3: the garrison trace of row 10, on CAOS–E1 side A | fail | fail |
| 15 | M-0001 | CAOS–E2 | side outcome, side B | col | – | (a) | As row 13 | as row 13 | built, as row 13 | fail | side B passes; the pairing still fails on side A (row 16) |
| 16 | M-0001 | CAOS–E2 | side outcome, side A | col | col 51 of 64 (*p*⁶ = 0.26) | (d) | – | – | none | fail | fail |
| 17 | M-0001 | DISK–FV | win test, both sides | col | ours row 11 of 12 on seeds 0 to 5 | (b), (e) | A6 row 2: the FV never find a hovering disc again after their first targets die. S-0309 r53 answered Q-2112 in part; which occasion gives the stock hunters their targets is S-0309's open question *Q-2112 question 1* (low), which names the re-read that would decide it | S-0309 claims 7, 22; S-0215 claim 18 | **Q-2112** question 1 (open in S-0309); wishlist A6b-2: the re-read S-0309 names | fail | fail |
| 18 | M-0002 | INIT–E1 | win test, side outcome, side B | row | row 0 of 64 (col 54, none 10) | (b) | New with the fix. The E1 now go for the bunkers and win from inside; which of Q-2926's points decides it (whether the stock units go for the building at all before the queued *hunt* starts, and what a refused one does) is open | as row 10 | **Q-2926** | pass | fail |
| 19 | M-0002 | INIT–E2 | side outcome, side A | row | row 55 of 64 (*p*⁶ = 0.40) | (d) | A6 row 12 (then *p*⁶ = 0.044); within noise, further from the edge now | – | none | fail | fail |
| 20 | M-0002 | YURI–E1 | side outcome, side B | col | col 16 of 64 (row 46; *p*⁶ < 0.001) | (b) | New with the fix. In the watched game (seed 0) the E1 go for the bunkers; those that fill them are taken by mind control and evicted (S-0508 claim 9), the rest are refused and stop hunting as in row 10 | as row 10; S-0508 claim 9 | **Q-2926**; wishlist A6b-1 | pass | fail |
| 21 | M-0002 | YURI–E2 | side outcome, side B | col | col 36 of 64 (*p*⁶ = 0.032) | (d) | New with the fix (as row 20); within noise | as row 10 | Q-2926 (related) | pass | fail |
| 22 | M-0002 | YURI–SHK | side outcome, side A | row | row 63 of 64 (*p*⁶ = 0.91) | (d) | – | – | none | fail | fail |
| 23 | M-0003 | MTNK–APOC | side outcome, side A | row | row 58 of 64 (*p*⁶ = 0.55) | (d) | As row 1 (Q-2782's open question names it) | as row 1 | none | fail | fail |
| 24 | M-0003 | ROBO–HTK | side outcome, side B | col | col 36 of 64 (*p*⁶ = 0.032) | (d) | A6 row 18 was (b), Q-2216, answered as built (intake 65); within noise now | – | none | fail | fail |
| 25 | M-0020 | part 2, YURI–SHK arm S | 12 games' winners | row 12 of 12 | row 11 of 12 (seeds 3 and 4 differ from the archived games) | (d) | Arm S is M-0002's YURI–SHK side A at seeds 0 to 11 (row 22): ours gives row in 63 of 64 games | – | none | fail | fail |
| 26 | M-0021 | part B: *c* ≥ 3 in every judged shot; the hits a judged shot owns; the reading-points row on a part B death's shot whose later hit names no attacker (3 rows) | rows | – | – | (b) | Since the scatter-from-damage build: case 2 of claim 19 reaches the test houses' free hunting vehicles. S-0215 r137 answered Q-2782 with that reading and left what else makes these rows differ open (*Arena records and the scatter from damage*, low) | S-0215 claim 19, B6 f and g; S-0208 claim 15 | S-0215's open question (from Q-2782) | fail | fail |
| 27 | M-0021 | part B: the side left and the dead per run; the six order-of-deaths rows (7 rows) | rows | – | – | (c) | These rows depend on how the vehicles move and which target each takes. M-0021 *Reproducing it* says a comparison builds the test types on a tank type of its own choice, and that part B's outcome depends on the movement, the countries' speed factors and the houses' factors, all kept local ([intake101.md](intake101.md): no request) | S-0413 claim 1; S-0301; S-0302 | covered by M-0021's *Reproducing it* | fail | fail |
| 28 | M-0021 | seeds: every vehicle's first shot falls before the first difference | row | – | – | (c) | Q-2597 (the record's two readings) was declined at M-0021's approval and stays open on purpose; the harness shows reading (b) passing beside it | – | **Q-2597** (open) | fail | fail |

Rows 1 to 25 cover 23 arena rows: rows 13 and 14 are the two sides of one pairing (CAOS–E1), and
so are rows 15 and 16 (CAOS–E2). Rows 26 to 28 hold 3, 7 and 1 of M-0021's rows. That makes 34
arena rows: the 33 that fail after the fix, and MIND–E1, which passes. Before: 28 failing rows; after: 33 (M-0001 12 → 14, M-0002 2 → 5, M-0003 2 → 2, M-0019 0 → 0,
M-0020 1 → 1, M-0021 11 → 11).

**Count per class** (ledger rows; arena rows in brackets): **(a)** 2 [two sides that now give the stock's outcome; both pairings still fail on side A];
**(b)** 5 [4 pairing rows and 3 M-0021 rows]; **(c)** 2 [8 M-0021 rows]; **(d)** 17 [17, of which
1 now passes and 3 are new with the fix]; **(e)** 2 [2], and DISK–FV (row 17) has a wishlist entry
beside its class (b).

## Requests

| request | question | interim |
|---|---|---|
| [Q-2927](../../spec/requests/Q-2927-may-occupy-for-an-assaulter-that-is-not-an-occupier.md) | What "may occupy this one" (S-0215 claims 23 and 27, S-0309 claim 25 (e)) means for an assaulter type that is not an occupier type: S-0508 claim 1's test (never true for it), claim 14's assault eligibility, or claim 1 without the unit-type condition | **answered** by S-0508 r19 claim 1 (claim 14's assault eligibility for claims 23 and 27; part (e) asks only occupier types); built at intake 123 ([intake123.md](intake123.md)) |
| [Q-2926](../../spec/requests/Q-2926-occupiers-of-a-test-house-refused-at-a-full-building.md) | Whether a computer-controlled occupier refused at a full building returns to *hunt*; whether forced *capture* clears the queued *hunt* for good; whether claim 25 (e) counts units already bound for the building; whether the *area guard* search takes the building before the queued *hunt* starts | **answered** by S-0508 r19 claims 1 and 4: the chain as built is the original's (no place held, no *hunt* after a refusal for an occupier type). The rows still fail; intake 123 filed **Q-2951** naming them ([intake123.md](intake123.md)) |

Q-2928 to Q-2935 are unused.

## Seams found on the way

S-0215 *Terms* make the arena houses follow the original-only rows. These original-only rows are
still not built for computer-controlled owners; none of them was shown to decide a failing row, so
they were left:

- **S-0215 claim 27, second infantry rule** (a deployed deployer infantry unit in *hunt* with no
  target undeploys; scenario B11 (c)).
- **S-0215 claim 28, the hunt rule** (an actor of a computer-controlled owner other than an
  occupier type, in *capture*, *sabotage* or *eaten*, has its destination cleared and queues
  *hunt*). The claim's own text says it never applies in our engine, which the *Test-only
  computer-controlled player house* term overrides for the arena houses (and the neutral and
  special houses keep it). `process::process_capture` cites it as never applying. It does not
  reach the E1 and E2 of rows 10, 18 and 20 (occupier types), so it would not move them.
  *Built at intake 123* (`process::capture_hunt_rule`; [intake123.md](intake123.md)); the arena
  did not move.
- **S-0215 claim 24, vehicle row 4** (`process.rs`, the *attack* vehicle rows) and **claim 26
  row 4** (deploying into a construction yard), original-only rows for computer-controlled owners.
- **S-0215's original-only automatic deploy row** for computer-controlled deployers.

## Checks

- Base: main e79d86f. The branch is rebased onto origin/main a71647f; the commits between them
  change tests, docs and the app's client only, so the base build and its arena stand for the
  rebased base. The final checks below ran on the rebased tree.
- Save format 125 → 126 and AI game format 91 → 92: the occupier rows change what a saved game
  replays to. The sim-gate goldens pass unchanged (none was regenerated).
- Arena (all six records): 28 failing rows before, 33 after (table above).
- Seed sweeps 1 to 16: M6c, M7a and M7b output identical to the base's (smallest minimum 0, 1 and
  the drain minimum 3).
- `glasnost-ai` with ignored tests, release: 264 passed, as on main.
- AI quick lane (seeds 7 to 10, maps 0, 2 and 8, 36 games): every winner and every end tick
  unchanged, so no game needed a trace. Our AI houses are human-controlled, so the rows reach only
  neutral and special houses' infantry there.
- Workspace tests, clippy, fmt, the wasm32 build, `glasnost-tools` (release), `cargo xtask check`
  (the 137 stock-guard problems, all in measurement records) and `cargo xtask gate`: as on main,
  with this step's 8 new tests.
