# M8 step A1: arena-spread audit

Step A1 of [docs/m8-plan.md](../m8-plan.md). Our oracle-arena games vary much less with the seed than
the stock games of M-0001 to M-0003 (*Arena mode (M7b)* in [development.md](../development.md)). This
file holds (a) where the arena's seed goes, (b) the table of random draws an arena game reaches,
(c) the failing pairings ranked by how far our spread falls short, (d) the draw built at this step and
(e) the requests filed. Nothing is tuned against the records: no stock value of the records is copied
here, and the ranking gives only ranks, our own numbers and counts.

Tool: `glasnost-headless arena --spread ROW,COL,A|B` (`oracle::spread`) plays the six seeds of one
side and prints, per seed, the tick at which the game state first differs from seed 0's and what
differs (positions, facings, health, targets, missions, timers). The draw counts below were taken once
with a throwaway per-site counter on the random stream (not committed: a counter in the simulation
would change it).

## (a) Where the seed goes

| stream (S-0202) | seeded from | in the arena |
|---|---|---|
| scenario stream (claim 10) | `Setup::seed` (`load::load`, `Rng::new(setup.seed)`) | the arena passes seed 0 to 5 per side (`oracle::setup`, `setup::skirmish(.., 0, seed)`). Every simulation draw of the table below comes from it |
| map generator stream (claim 13) | 0, always | the map's own draws (ore glints, claim 14; tiberium growth); the same for every seed, as specified |
| unsynchronised stream (claim 11) | presentation only | no simulation draw; none in the headless arena |
| lobby stream | not used | countries, colours and starts are fixed by M-0001's *Setup*, so nothing in the arena draws from it |

Placement, facing and the hunt order are fixed by the records' *Setup* (units created through
`testkit::spawn` and `objects::create`, which takes the creation draw of S-0202 claim 10; the hunt
starts with its timer expired, S-0215 claim 3). So the seed reaches the games only through the
scenario stream, and it does reach them: with `--spread`, our games differ from seed 0's

- at tick 1 in the ROBO pairings (S-0503 claim 10: a disabled hovering unit's spin draws),
- at tick 2 in the infantry pairings (S-0208 claim 3: the standing-row draw of a walking infantry),
- at about tick 30 to 90 in the vehicle pairings, when the groups first fire (S-0301, S-0306).

The outcomes often do not follow: many pairings give the same winner and the same shares in all six
seeds although the states differ from early on. The shortfall is therefore mostly not a seeding gap
(the seed reaches the stream) but either a fight that one side wins decisively in ours (nothing left
for the draws to vary, see (c)) or a variation before first contact that the specs do not give us
(Q-1711).

## (b) Draws an arena game reaches

Every draw site the counter saw in the 1860 games of the three records, grouped by spec. "Reached"
gives the records in which the site took at least one draw (1 = M-0001, 2 = M-0002, 3 = M-0003);
"many" means more than about 10⁵ draws over the record. All draws are from the scenario stream.

| spec and claim | decides | status | code | reached |
|---|---|---|---|---|
| S-0215 F2, F0 | the per-object processing delay U(0, 2) | built | `process.rs` | 1, 2, 3 (many) |
| S-0215 claim 32 | the hunt behaviour's next search delay | built | `combat/behave.rs` | 1, 2, 3 (many) |
| S-0215 claim 26 | the mission delay U(1, 5) | built | `process.rs` | 1, 2, 3 |
| S-0215 claim 33, S-0309 claim 4 | the targeting timer's restart U(0, 2) after a search | built | `combat/acquire.rs` | 1, 2, 3 (many) |
| S-0215 claim 38, F11 | the base-guard displacement and direction | built | `combat/base.rs` | 1, 2, 3 |
| **S-0309 claim 25 (g)** | a computer-owner searcher seeing through a blinking disguise | **built at this step** (was the seam `cloak::disguise::neutral_sees_through`) | `combat/acquire.rs` (`admit`) | 1, 3 (the MGTK pairings, where the arena houses search at a mirage tank) |
| S-0516 claim 33 (c) | the disguise a mirage tank takes | built | `cloak/disguise.rs` | 1, 3 |
| S-0301 F1 | the full reload delay | built | `combat/fire.rs` | 1, 2, 3 (many) |
| S-0301 claim 4 | R(3, 5) | built | `combat/fire.rs` | 1, 3 |
| S-0306 claim 4, F5 | projectile scatter distance and direction | built | `combat/projectile.rs` | 1, 3 |
| S-0306 claim 9 | invisible flak scatter | built | `combat/projectile.rs` | 1 |
| S-0306 claim 21 | the post-detonation delay and nearby point | built | `combat/projectile.rs` | 1, 2, 3 (many) |
| S-0303 claim 21 | an invisible projectile's nearby point | built | `combat/projectile.rs` | 1, 2, 3 (many) |
| S-0303 F6 | debris count | built | `combat/projectile.rs` | 2 |
| S-0303 F4 | the area-damage threshold draw | built | `combat/area.rs` | 1 |
| S-0312 F1, Edge cases | debris and metallic debris on death | built | `combat/death.rs` | 1, 2, 3 |
| S-0312 claims 9, 16 | explosion and destruction list picks | built | `combat/death.rs`, `jump/fly.rs` | 1, 2, 3 |
| S-0312 claim 19 | building-death smudge and fire draws | built | `combat/death.rs` | 1, 2 |
| S-0312 claim 26 | smudges | not modelled (the draws of claim 19 are taken; the smudge itself is not placed) | — | — |
| S-0107 F4, claim 14 | infantry idle timer, idle row and facing | built | `infantry.rs` | 1, 2, 3 (many) |
| **S-0107 claim 17, row 3 and the corpse pick** | a type's own death animation; the corpse a completed death sequence leaves (one raw value modulo the list length) | **built at this step** (was the seam `death_anims_row`, and the corpse lists were not imported) | `combat/death.rs` (`infantry_death`, `corpse`) | 1, 2 (built after the count: the build changed 37 games of M-0001 and 37 of M-0002, none of M-0003) |
| S-0208 claim 3 | the standing row of a walking infantry | built | `walk.rs` | 1, 2 |
| S-0208 claims 12, 13 | the scatter direction | built | `scatter.rs` | 1, 2, 3 |
| S-0615 claim 6 | the walker's fallback destination | built | `walker.rs` | 2 |
| S-0503 claim 10 | a disabled hover unit's spin | built | `hover.rs` | 1, 3 |
| S-0502 claim 21 | a jumpjet's landing neighbour | built | `jump.rs` | 1 |
| S-0601 claims 8, 16 | mind control's chance, sparks and rocking | built | `mind.rs` | 1, 2 |
| S-0613 F2 | gas cloud movement | built | `gas.rs` | 2 |
| S-0202 claim 15 | smoke and particle variations | built | `gas.rs` | 1, 2, 3 |
| S-0616 claims 1, 6, 9, 15, 16, F4, F10 | particle systems and emitters | built | `particles.rs`, `emitters.rs` | 1, 2, 3 (many) |
| S-0202 claim 10 | the creation draw | built | `objects.rs` | 1, 2, 3 |
| S-0202 Constants | animation bounce draws | built | `load.rs` | 1, 2, 3 |
| S-0202 claim 14 | ore glints (map stream at load; scenario stream in play) | built | `load.rs` | 1, 2, 3 (many) |
| S-0401 claim 15 | ore growth and spread | built | `econ/resource.rs` | 1, 2, 3 (many) |
| S-0009 claim 20 | the final draw after the starting units | built | `starting.rs` | 1, 2, 3 |
| S-0011 F2 | trigger timers | built | `triggers.rs` | 1, 2, 3 |
| S-0109 claims 11, 14 (μ > 0), 15, 17, 28, 29 | an animation's between-pass loop delay, its follow-up switch (and the follow-up's creation and start-event draws), damage over time | seam: the simulation does not play animations (claim 28's seam, *Animation playback (M6b)* in development.md). A diagnostic note (`load::anim_created`) shows that the loop and follow-up parts are created in nearly every pairing; an animation with damage over time is created in none | `load.rs` | notes in 1, 2, 3 |

The table lists only built sites, since the counter sees only those. The other direction (draws the
specs name that the code does not take) was checked with a sweep of the claims of S-0107, S-0109,
S-02xx, S-03xx, S-05xx and S-06xx whose text names a draw, against the claim ledgers and the code's
citations. Reachable in the arena and not built, besides the S-0109 row above:

| spec and claim | decides | status | reached |
|---|---|---|---|
| S-0312 claim 18 | survivors of a destroyed building | seam (no sides in the simulation's rules, so the count is 0; `docs/m4-claims/S-0312.md`) | 1, 2 (the few building deaths: S-0312 claim 19 draws were taken) |
| S-0312 claims 13 and F4, rows 2 to 5 | the crew type by side | partly: the crew type is always F4's first row, with no draw (no sides) | not reached: the crew chance draw of claim 13 was taken in no arena game |
| S-0312 claims 22, 23, 26 | spilled resources, barrels set off, the smudge type | seams | 22 and 23 not reached (no harvester dies, and the exploding-overlay draws of S-0303 claim 19 were taken in no game); 26 takes no draw while no smudge type fits |

The draws of S-0107 claim 17 were the one specified gap the arena reaches at every infantry death;
they are built at this step (d). The animation
playback is a large piece of architecture (the simulation would run every animation's frames) and is
not attempted at this step; it is proposed as its own step.

## (c) Failing pairings ranked by spread shortfall

The shortfall of a pairing is the sum, over the row-win indicator and the two shares left, of the
stock games' seed-to-seed standard deviation (within a side, averaged over the two sides) less ours,
each term clipped at 0 (scratch script, not committed). Computed in the local lane only (`local-lane/m8/a1/before-spread.txt`). The worst ten failing
pairings, before this step, with what our games show:

| rank | record, pairing | ours (all six seeds of each side) | candidate cause |
|---|---|---|---|
| 1 | M-0001 LTNK against ROBO | the ROBO are disabled (no shots) and the LTNK destroy all of them in every game; in the stock games a seed-dependent part survives | Q-1710 |
| 2 | M-0001 YTNK against SREF | YTNK win by the first sample at full value; the SREF fire few shots; the front column of YTNK all take equal damage | Q-1714; since M8 A3 intake 6 S-0318's shrapnel fragments answer it in part and the pairing passes (ours 9/3 against stock 8/4; row left 0.183 against 0.122) |
| 3 | M-0001 MIND against E2 | the column side wins every game | the open mind-control item of *Arena mode (M7b)*; Q-1713 for E2 on side A |
| 4 | M-0001 YTNK against E2 | side B ends `none` in every game | Q-1713 |
| 5 | M-0001 TELE against BFRT | the BFRT take no target and fire no shot in 6080 frames; the TELE fire thousands of shots at one BFRT with no effect; both shares stay 1 | Q-1712; answered in part by M-0015 (M8 A3 intake 40, [ledger](intake40-M-0015.md)): in ours every fortress is seized at tick 1, dropped and set down unhurt, and then stays seized for good (in *hunt* it never falls back to rest, so S-0607 claim 20 never restores it), so it is never held again and never fires; in the stock games the fortresses are held again 52 to 56 times, fire at the Magnetrons and crush them; what restores them is open, Q-2035 |
| 6 | M-0003 MGTK against APOC | the MGTK side wins every game; the APOC stop firing at a disguised MGTK | S-0309 claim 25 (g), built at this step: the pairing now passes, see (d) |
| 7 | M-0001 CAOS against BFRT | always `none`, the CAOS group untouched | Q-1712; answered in part by M-0011 (M8 A3 intake 38, [ledger](intake38-M-0011.md)): in the stock games the berserk fortresses crush the drones, in ours they never take a drone as target, so no drone leaves play; the rule that would let them choose one is open, Q-2005 |
| 8 | M-0002 INIT against E2 | INIT on side A: the five INIT on the cliff cells survive, always `none` | Q-1713 |
| 9 | M-0003 ROBO against APOC | as rank 1: disabled ROBO, all destroyed | Q-1710 |
| 10 | M-0003 ROBO against HTNK | as rank 1 | Q-1710 |

Next in the ranking: M-0001 CAOS against E2 and MIND against E1 (the CAOS and mind-control items of
*Arena mode (M7b)*). M-0003 MGTK against TTNK ranks fourth by shortfall but passes every test.

Overall, before this step, the stock games give the same result in all six seeds on 135 of the 310
sides, ours on 178. Most of the worst shortfalls are gaps in the mean as much as in the spread: our
game is won (or stalls) so clearly that the draws we take have nothing to vary.

## (d) Built at this step

| claim | status | tests or reason |
|---|---|---|
| S-0107 claim 17, row 3 and the corpse pick | tested | The importer reads `DeadBodies` (infantry types and `[General]`) and `DeathAnims` (Import mapping). Row 3: a type with its own death-animation list shows entry *w* (entry 0 when *w* is negative or not less than the list's length; no-type and empty items are left out by the importer, S-0107 revision 22), with no sequence, and is removed at once, ahead of rows 4 to 8 (`combat::death::infantry_death`, `death_anim`). The corpse: when die-1 or die-2 completes (and for a death request refused with `quirks.refused_death_keeps_unit` off), the type's own list when not empty, else the general list unless the type is "not human"; one raw value modulo the length picks the entry (`combat::death::corpse`); wet-die leaves none. No-type entries: Q-1715, answered by S-0107 revision 22 and built at M8 step A3 intake 5. Tests in `claims_s0615.rs`: `s0107_claim_17_corpse_from_the_general_list_by_one_draw`, `s0107_claim_17_own_corpse_list_not_human_and_wet`, `s0107_claim_17_refused_death_with_the_quirk_off_leaves_a_corpse`, `s0107_claim_17_row_3_own_death_animations`; importer `corpse_and_death_animation_lists` |
| S-0309 claim 25 (g) | tested | `acquire::admit` and the `Verdict` of `acquire::candidate`: for a searcher of a computer-owner house (not a player house with `human` set, S-0309 Terms), a target that appears disguised is rejected without a draw unless its disguise is blinking (S-0516 claim 39); when it blinks and every row outside the six of part g lets the target through, one R(0, 99) draw is taken (after the targeting timer's draw), and the target is let through when the draw is at most the house's entry of the disguise-detection list, selected by its difficulty index (a marked test house reads its index 0). A target that the capture, engineer, storage, bridge-layers, healer or score row rejects has still taken the draw. No such entry: nothing is let through (Import mapping). Tests in `claims_s0309.rs`: `neutral_searcher_sees_through_a_blinking_disguise_by_one_draw`, `player_searcher_never_takes_the_disguise_draw`, `disguise_draw_entry_by_difficulty_index`, `disguise_draw_only_for_an_object_the_other_rows_let_through` |

The draw is reached in the arena's MGTK pairings: both arena houses are marked test houses (computer
owners), so a searcher of either side now sees through a mirage tank whose disguise blinks with the
draw. Only MGTK pairings changed: M-0003 MGTK against APOC (rank 6) and M-0001 YTNK against MGTK now
pass, and four more side-unanimous stock outcomes in M-0001 and two in M-0003 are reproduced (MIND
against MGTK still fails, on another test).

Effect on play outside the arena: the sim gate's goldens and the three seed sweeps are unchanged
(no golden or sweep scenario has a mirage tank meeting a computer-owner searcher, and their
synthetic rules give no corpse or death-animation list). In a game on real rules data both builds
are reachable: claim 25 (g) for a searcher of the neutral or special house at a blinking disguise
(our AI's houses are player houses with `human` set, so they never take it), and the corpse pick
at every infantry death sequence that completes when the rules give a corpse list. No state was
added (a corpse is an ordinary animation), so the save and AI formats stay as they were.

## (e) Requests

| request | gap | interim |
|---|---|---|
| [Q-1710](../../spec/requests/Q-1710-hunters-and-disabled-vehicles.md) | which disabled vehicles a hunting group leaves alone (ROBO pairings) | the built rules |
| [Q-1711](../../spec/requests/Q-1711-variation-at-the-start-of-an-arena-game.md) | what varies from seed to seed before first contact (first move, search and shot, facings, the infantry standing point, paths, whether the harness used the map's entries) | answered in part by S-0216 (questions 2 and 3), built at M8 step A3 intake 21, [intake 21 ledger](intake21-S-0216.md); question 5 open, so the oracle arena keeps the records' *Setup* as read, infantry at the cell's centre point |
| [Q-1712](../../spec/requests/Q-1712-battle-fortress-group-in-the-arena.md) | what a BFRT group without passengers does (TELE and CAOS against BFRT) | the built rules |
| [Q-1713](../../spec/requests/Q-1713-infantry-on-cliff-cells-after-q1502.md) | how the games with infantry on the cliff cells end, after Q-1502 | answered in part by S-0216 claim 6 (questions 2 and 3, already as built), M8 step A3 intake 21, [intake 21 ledger](intake21-S-0216.md); question 1 open: S-0309 revision 21 and S-0208 revision 34 as built |
| [Q-1714](../../spec/requests/Q-1714-ytnk-against-sref-opening.md) | how YTNK against SREF unfolds | answered in part by S-0318 (shrapnel fragments), built at M8 step A3 intake 6: the pairing passes, [intake 6 ledger](intake6-S-0318-S-0314-S-0108-S-0701.md) |
| [Q-1715](../../spec/requests/Q-1715-none-entries-in-corpse-and-death-animation-lists.md) | what a no-type entry does in a corpse list or a death-animation list (S-0107 claim 17, built at this step) | answered by S-0107 revision 22 (spec 98bcb77), built at M8 step A3 intake 5: no-type and empty items are left out of both lists |

## Arena and sweeps, before and after

Oracle arena (release, 8 threads; outputs in `local-lane/m8/a1/before/` and `after/`, never committed):

| record | when | fail win | fail row-left KS | fail column-left KS | pairings failing any test | side-unanimous stock outcomes reproduced |
|---|---|---|---|---|---|---|
| M-0001 | before | 6 | 26 | 16 | 35 of 96 | 145 of 172 |
| M-0001 | with claim 25 (g) | 5 | 25 | 15 | 34 of 96 | 149 of 172 |
| M-0001 | after (with the corpse pick too) | 5 | 25 | 14 | 34 of 96 | 148 of 172 |
| M-0002 | before and after | 4 | 9 | 3 | 16 of 24 | 43 of 47 |
| M-0003 | before | 1 | 3 | 2 | 4 of 35 | 59 of 64 |
| M-0003 | after (both builds) | 0 | 2 | 1 | 3 of 35 | 61 of 64 |

Claim 25 (g) moved only MGTK pairings (above). The corpse pick of S-0107 claim 17 changed 37 of
M-0001's 1152 games (5 pairings) and 37 of M-0002's 288 (7 pairings), none of M-0003's: it is taken
only when an infantry death sequence completes. In M-0001 one column-left KS test now passes, and
CAOS against E1 (still failing) no longer reproduces one side-unanimous stock outcome.

Sides on which all six of our games end alike: 178 of 310 before, 179 after (the stock games: 135).

Seed sweeps 1 to 16 (`m6c_seed_sweep`, `m7a_seed_sweep`, `m7b_seed_sweep`): every counter's minimum
and median are the same before, with claim 25 (g) and after both builds (the smallest minimums: 0 in M6c, 1 in M7a, 3 for the
drain in M7b).

## After M-0017 (M8 step A3)

The arena's report gives its totals twice: for the pairings whose stock rows M-0017 confirms, and for the pairings with SREF or V3 on either side ("not confirmed by M-0017"; still judged). The 7 crashed MIND against E2 games are left out, so that pairing runs on n = 5. See [intake-M-0017.md](intake-M-0017.md).
