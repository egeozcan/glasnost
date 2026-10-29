# Own AI of M8 Track B

These sections of M8's Track B (steps B1 to B7, the own AI's tempo, margins and the dominator) moved here from [development.md](development.md) on 2026-10-25, when that file reached the repository's 1 MiB file limit again; Track B is done (docs/m8-plan.md, *Decisions of 2026-10-25*). They are kept as they were; links inside are relative to `docs/`.

## Own AI (M8 steps B1 and B2)

Expansion in full games (B1) and superweapon saving without losing tempo (B2), both in `Brain::economy_first` and `Brain::expand` (`crates/deterministic/ai/src/brain.rs`); AI game format 30. Lane outputs are git-ignored under `local-lane/m8/b12/` (counts and ticks only). Seeds 7 to 10, maps 8, 2 and 0, Yuri (house 1) against Allied and against Soviet: the quick lane (`GLASNOST_AI_QUICK=1`, 24 games) and the six full games (seed 7, quick unset: replay and save checks; a full game plays the same game as the quick game of its seed, so the full set adds only those checks). Every binary below differs from the one before it only in the brain; *base* is main at 6fe1b84 with this branch's two expansion-probe commits (a read-only probe and lane output, no change to play).

**The expansion probe** (`GLASNOST_AI_EXPROBE=<ticks>`, `Brain::expansion_probe`, `ExpansionProbe::blocked_by`): every so many ticks the lane prints, per AI house, what `expand` sees (yards, yard vehicles, the line, whether the yard vehicle is buildable and, if not, its fewest unmet prerequisites and the building that would clear them, credits against cost, the far field, harvesters, intruders, selling, paying, outnumbered, losing, saving), and the report line *expansion per house* gives the most yards at once, the first tick with two, the first tick the yard vehicle was buildable (sampled every 100 ticks), and the probe samples by failed condition; *superweapons per house* gives the first tick each kind was granted and ready.

**What the probe found (base, six full games).** No yard vehicle was built in any of them, for two reasons; saving was not one, since `expand` had no saving test (the round-3 note said otherwise; saving acted only by keeping the credits low).
- The Allied and Soviet houses never had the yard vehicle on the sidebar: it waits on one or two prerequisites (S-0407 claim 8), and a building that clears them was buildable and placeable all game. No step built it: the tech climb (`Script::climb`) targets army types only, `build_superweapon` only superweapon givers, and the yard vehicle has no role.
- The Yuri house could build it from ticks 11 600 to 22 400 in three games, and was then refused by the wealth test (credits at least twice its cost): its credits stayed near 0 to a few hundred, since every line draws them down as they come in. In the probe samples where only wealth could block, the credits were below a quarter of the cost in nearly all.

**B1, what the brain does now.**
- *Unblocker:* when the yard vehicle is not buildable and the house would expand otherwise, it builds `Script::unblocker` of the cheapest yard vehicle type (the tech climb's search, S-0407 claim 8), unless this think already orders a place or a building other than the climb's (it replaces the climb's building, like a superweapon building does). Counter `Uses::expansion_unblocks`.
- *Rich* (`Brain::rich`): twice the cost in hand, as before, or the economy whole (`Script::economy_kept`) and no superweapon building on its lines. A person finishes the superweapon first, then expands, and buys the yard vehicle step by step like any item (S-0404 claim 4: no money is needed to start one).
- *Safe:* besides harvesters kept and no intruder, the house is not outnumbered by the enemy units it sees and not losing a fight (`Brain::losing`, below).
- *Held while saving:* a yard vehicle on the line while the house has a yard is held with the army while it saves (it draws on the superweapon's money); one bought with no yard (`rebuild_yard`) runs on.
- The 2-yard stop (`EXPANSION_YARDS`) is unchanged: it counts yards, yard vehicles and a yard vehicle on the line, and stops the buying only. A yard vehicle the house did not buy (a crate's unit, S-0520, or one taken over) is still driven to a field and deployed, as a person would: in 6 of 120 quick games after the change a house had 3 yards at once, and once 4, each time with one expansion build.

**B2, what the brain does now.** Saving keeps round 3's trigger: a superweapon-giving building runs on a line owing more than the money in hand (S-0404 Terms, balance owed), and the house is safe. With less than the balance in hand every credit the army spends is one the building still needs, so the hold shortens the time to the superweapon. Changed: only army items that still owe money are held; *safe* (`Brain::safe`) now also needs no fight being lost; and `Brain::saving` is shared by the probe and `Brain::rich`. `Brain::losing`: seen enemy army units shoot at the house's objects, its own army units stand within `FIGHT` cells of the shooters, and the seen enemy army units there are worth at least as much by cost. Shots with no army near them are a raid, which the base and economy defences answer: counted as a lost fight they ended the saving in 928 of 2 458 probe samples (about two in five), and the first version of the rule did so.

**Tried and dropped (B2).** A hold only while the building is short now (its clock stands, or the money in hand does not pay its next step, S-0404 claim 4), with a held item resumed once the money pays both next steps (lane *v2*). The army spent what the building's later steps needed: the house held and resumed on most thinks (up to 442 holds in a quick game against 60 in base) and every superweapon came later (Yuri's force shield in all 16 games where both readied it, by a median 1 936 ticks; its mutator in 12 of 12, by a median 4 436).

**Quick lane, 24 games.** *nosave*: saving forced off. *B2 alone*: the final B2 rule with the B1 changes stubbed off. *B1, nosave*: the final binary with saving forced off.

| Binary | Yuri wins vs Allied | vs Soviet | No winner | Yuri superweapon holds, median / most | Expansion builds (house 0, house 1) | Moves | Unblocks | Games with 2 yards (house 0, 1) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| base | 4 / 12 | 10 / 12 | 0 | 32.5 / 60 | 0, 7 | 8, 56 | 0, 0 | 1, 3 |
| nosave | 3 / 12 | 9 / 12 | 0 | 0 / 0 | 0, 4 | 2, 24 | 0, 0 | 0, 6 |
| v2 (dropped rule, with B1) | 4 / 12 | 10 / 12 | 0 | 81.5 / 442 | 13, 16 | 220, 73 | 16, 14 | 12, 12 |
| B2 alone | 3 / 12 | 10 / 12 | 0 | 33.5 / 88 | 0, 2 | 6, 18 | 0, 0 | 2, 6 |
| B1, nosave | 3 / 12 | 10 / 12 | 1 | 0 / 0 | 7, 9 | 34, 40 | 11, 8 | 3, 9 |
| **final (B1 + B2)** | 2 / 12 | 10 / 12 | 1 | 33.5 / 139 | 15, 11 | 44, 83 | 16, 5 | 9, 9 |

Yuri's superweapons, first tick ready, against base (median difference over the games where both binaries readied the kind; games in which it was later / games compared):

| Binary | Force shield | Mutator | Dominator | Psychic reveal |
| --- | --- | --- | --- | --- |
| nosave | +3 241 (19 / 19) | +7 589 (9 / 9) | +9 288 (3 / 3) | +27.5 (15 / 24) |
| B2 alone | 0 (10 / 22) | +175 (9 / 12) | −145.5 (3 / 6) | 0 (0 / 24) |
| final | 0 (10 / 22) | +618 (11 / 12) | +1 841.5 (5 / 8) | 0 (0 / 24) |

**Full games, seed 7 (base → final).** Every game replays to the same state and survives its save and load in both sets.

| Opponent, map | Winner, tick | Yuri holds | Expansion builds / moves / unblocks, house 0 | house 1 | Most yards (0, 1) | Yuri ready: force shield, mutator, dominator |
| --- | --- | --- | --- | --- | --- | --- |
| Allied, 0 | Allied 42 765 → Allied 40 146 | 21 → 9 | 0 / 0 / 0 → 0 / 0 / 0 | 0 → 0 | 1, 1 → 1, 1 | 13 041, 16 922, – → 13 156, 16 926, – |
| Allied, 2 | Yuri 39 544 → none at 120 000 | 54 → 139 | 0 → 1 / 7 / 1 | 0 → 0 | 1, 1 → 2, 1 | 10 710, 14 379, 35 349 → 10 710, 14 292, 111 483 |
| Allied, 8 | Allied 36 075 → Allied 29 234 | 36 → 28 | 0 → 1 / 5 / 1 | 0 → 0 | 1, 1 → 2, 1 | 13 764, 17 294, – → 14 063, –, – |
| Soviet, 0 | Yuri 57 992 → Soviet 74 888 | 9 → 73 | 0 → 1 / 0 / 1 | 0 → 0 | 1, 1 → 1, 1 | 13 877, –, – → 13 923, 22 215, – |
| Soviet, 2 | Yuri 42 938 → Yuri 28 189 | 46 → 22 | 0 → 0 | 0 → 0 | 1, 1 → 1, 1 | 10 710, 13 886, 29 783 → 10 710, 14 292, – |
| Soviet, 8 | Yuri 28 300 → Yuri 37 182 | 41 → 60 | 0 → 1 / 5 / 1 | 0 → 1 / 12 / 1 | 1, 2 → 2, 2 | 13 790, 16 674, 25 997 → 14 799, 18 636, 30 514 |

**B1 exit, house by house** (full games, final binary): the yard vehicle was bought and deployed by the Allied house on maps 2 and 8 and by both houses on Soviet map 8. Where a house could build it and did not, the probe names why:
- Neither house could ever build it on Soviet map 2 (the game ended at 28 189), nor the Allied house on map 0.
- Yuri against Allied, maps 0, 2 and 8: no free field on its half (55 of the 62 samples where it was buildable on map 2), and outnumbered or with enemies in its base most of the time it could build it; on maps 0 and 8 it was buildable in only 1 and 4 samples, under attack in each, and lost its yard on map 0 at about tick 27 000.
- Yuri against Soviet, map 0: a superweapon building stood on its line from about tick 22 500 to 46 500 (no expansion before the superweapon is finished), and it lost its yard by tick 48 000.
- The Soviet house on map 0: the yard vehicle was on its line from tick 52 500 to the end (74 888) without being finished; the house's credits stayed below 500 in every sample (broke, no saving: a person would not have bought it either; the brain had nothing else to stop it).

**Exit B2: one game short.** The fair comparison is the final binary against the same binary with saving forced off (*B1, nosave*), the only pair whose one difference is the saving with B1 on. Saving on wins one game fewer: 12 against 13 on seeds 7 to 10, and 12 against 12 on seeds 11 to 14 (a second quick lane of 24 games run for this check), so 24 against 25 of 48. The other pairs mix in B1's effect on the opponents (the Allied and Soviet houses expand only since the unblocker): B2 alone wins 13 against nosave's 12, and on the same main base and nosave already differ by 2 (14 against 12), so one game in 48 is within the lane's spread, but the exit as written (no fewer wins) is not met. The tempo half holds: against *B1, nosave* the final binary readies Yuri's force shield earlier in every game where both ready it (median 2 977.5 ticks earlier over 20 games on seeds 7 to 10, 3 911 over 16 on seeds 11 to 14) and its mutator earlier in every game (7 106.5 over 12, 8 051.5 over 10). Against the round-3 hold (base) the force shield is ready at the same tick (median 0) and the mutator later by a median 175 ticks with B2 alone and 618 with B1 (the expansion's spending between superweapons).

**Tests** (synthetic data). `tests/economy.rs`: `holds_the_army_only_while_the_balance_is_not_in_hand`, `holds_an_expansion_yard_vehicle_while_saving`, `keeps_training_while_losing_a_fight` (and round 3's three saving tests). `tests/expansion.rs`: `a_house_with_its_economy_whole_builds_a_yard_vehicle_step_by_step`, `a_poor_house_saving_for_a_superweapon_builds_no_yard_vehicle`, `builds_what_the_yard_vehicle_waits_for` (a fixture building as the yard vehicle's prerequisite: built, placed, then the yard vehicle), and round 3's three.

**Arena and duel sweeps.** The oracle arena (no brain) prints the same rows before and after (M-0001 57 of 96 rows pass, M-0002 8 of 24, M-0003 28 of 35), and the M6c, M7a and M7b duel seed sweeps 1 to 16 print the same counters for every seed. The sim-gate goldens changed from their first line because every state hash includes the brain's saved counters, which gained a field (format 30); play in the gate's own scenarios (seed 20261018) was not compared apart from that.

## Own AI (M8 step B3: the dominator takes units)

Claim ledger: [m8-claims/S-0612.md](m8-claims/S-0612.md). Code: `glasnost_ai::dominator`; the estimates `glasnost_sim::combat::damage::hit_estimate` and `glasnost_sim::combat::area::object_distances` (read-only, no state); the lane lines in `crates/tools/src/skirmish_lane.rs`. Tests: `crates/deterministic/ai/tests/dominator.rs` (16), `the_hit_estimate_is_the_detonation_loss` (`claims_s0303.rs`), `counts_dominator_launches_takeovers_and_damage` (tally).

- **Who counts as a take.** The candidate test was already S-0612 claim 13's (S-0601 claim 13's conditions: building, psionics immunity, protection, balloon hover, in the air; `mind::dominable`), and the spec settles takeability, so no request was filed. What the 4d rerun had missed was claim 10 and the time to the sweep: the lane (below) showed predicted takes walking out of the reach, and others destroyed by the sweep's own damage. A takeable unit is now one that passes those tests **and stands still** (no movement destination; the sweep comes some time after the launch, claim 4). A takeable unit inside the area damage is counted as damage, not as a take, when the damage is expected to destroy it (claim 10), or to destroy any object the house sees: a destroyed object's death weapon goes off in the middle of the detonation (S-0303 claim 11). In the lane a building the sweep destroyed took the vehicle beside it, whose own death weapon then destroyed three more buildings.
- **The damage estimate** is now the simulation's own: the detonation's distance (S-0303 claims 7 and 10, a building per foundation cell, claim 8), the defences and the modified amount (S-0302 claims 5 and 6). The old estimate (falloff and versus of the raw amount at the 2-D distance to the position) counted no defences and measured buildings to their position.
- **Where it aims.** `best_take` searches every explored cell from which the swept cells (`domination::swept`, claim 11) reach a seen takeable unit (the ring metric is symmetric, S-0303 Terms), for the most value taken less own and allied losses, at least one enemy unit taken. Only when no cell takes any does `best_damage` (the cells of the seen enemy objects) aim for the damage alone, and that shot is held until the weapon has been ready for `HOLD_PERCENT` (50, own tuning) of its charge time: a person waits a while for a better target, but never sits on a full charge for nothing.
- **Lane lines.** The psychic-domination line counts sweeps and sweeps that changed an owner; each launch prints `dominator launch tick … cell …: preview takes N, of other houses M` with each predicted take of another house (owner, cell, health, destination, behaviour, expected damage), each sweep prints what became of them, and an object killed or gone before the sweep is reported. The probe (`GLASNOST_AI_SWPROBE`) counts dominable units on the move.

**Full games** (`ai-sides 120000 7 <map>`, full mode, `GLASNOST_AI_SWPROBE=900`; outputs git-ignored under `local-lane/m8/b3/{rbefore,final}-f-*`; before: main at 54554a5 with only this branch's lane counter, after: this branch on the same main). Yuri is house 1. Launches / sweeps that changed an owner / objects taken / killed by the sweeps:

| Opponent, map | Winner, tick before | Before | Winner, tick after | After |
| --- | --- | --- | --- | --- |
| Allied, 2 | Yuri, 42 619 | 2 / 0 / 0 / 11 | Yuri, 45 840 | 2 / 1 / 1 / 7 |
| Allied, 8 | Allied, 30 920 | no launch | Allied, 30 920 | no launch |
| Allied, 0 | Yuri, 46 938 | 1 / 0 / 0 / 1 | Yuri, 57 798 | 1 / 1 / 1 / 0 |
| Soviet, 2 | Yuri, 25 679 | no launch | Yuri, 25 679 | no launch |
| Soviet, 8 | Yuri, 38 553 | 1 / 0 / 0 / 3 | Yuri, 38 659 | 1 / 0 / 0 / 2 |
| Soviet, 0 | Yuri, 57 999 | no launch | Yuri, 57 999 | no launch |

Exit not met on this main: 2 of the 4 launches in the full games took at least one object (before: 0 of 4), half and not most. Both launches that aimed at a predicted take took; the other two were shots for damage alone after the hold, with no takeable unit in sight for the whole hold (the probe: the seen units refused as immune or in the air, or none). On the earlier main (6fe1b84, before steps B1 and B2) the same code took in 3 of 4 launches (before: 0 of 4); B1 and B2 changed the games, not the aim.

**Quick lane** (seeds 7 to 10, 24 games; `{rbefore,final}-q-*`): launches 15 → 10, sweeps that changed an owner 1 → 5, objects taken 1 → 7 (owner changes 1 → 6), killed by the sweeps 52 → 29. Yuri's wins: 3 of 12 against Allied and 10 of 12 against Soviet, before and after. Of the 10 launches, 5 aimed at a predicted take and all 5 took; the other 5 were shots for damage alone after the hold. On the earlier main the steps on the way were (launches / sweeps changing an owner / owner changes; before 23 / 0 / 0): the take-first aim alone 20 / 9 / 12; with standing units only 18 / 10 / 19; with the simulation's estimate 16 / 9 / 14; with the wreck rule 17 / 9 / 13; final (buildings by foundation cell) 18 / 9 / 13. There 12 of the 18 launches aimed at a predicted take and 9 took. The 3 lost takes: twice the predicted units (two walkers, one vehicle) were destroyed in the sweep though no seen object was expected to be destroyed (a chain the house could not see, or damage the estimate misses: not traced further), and once a walker in the move behaviour with no destination stepped one cell and was not taken.

**Gate and arena.** The `duel-m7b` golden is regenerated (on main 54554a5); no save or AI game format change (no new state). Seed sweeps 1 to 16 on main 54554a5, min / median: M6c and M7a unchanged. M7b, before → after: dominator launches 8/8 → 6/7, objects taken 12/20 → 11/20, mutator launches 5/7 → 5/7, infantry mutated 15/22 → 15/20, berserk orders 20/30 → 15/27, units made berserk 12/21 → 12/21, takeover orders 0/3 → 0/1, takeovers 0/1 → 0/1, absorber entries 11/22 → 13/18, absorbed 5/9 → 4/8, grinder entries 4/9 → 6/9, ground 4/7 → 5/8, bunker builds 4/4 → 3/4, slave buildings 4/12 → 4/12, slaves 3 413/3 682 → 3 413/3 693; the drain counters unchanged (smallest drain minimum 3). The oracle arena attaches no brain: its CSVs are byte-identical before and after (M-0001: 34 of 96 pairings failing any test, 148 of 172 unanimous outcomes reproduced; M-0002: 16 of 24, 43 of 47; M-0003: 3 of 35, 61 of 64).

## Own AI (M8 step B5: tempo after expansion)

Follow-up of B1 and B2 (above), in `Brain::expand`, `Brain::rich` and `Brain::economy_first` (`crates/deterministic/ai/src/brain.rs`). Lane outputs are git-ignored under `local-lane/m8/b5/` (counts and ticks only). The quick lane is seeds 7 to 14, maps 8, 2 and 0, Yuri (house 1) against Allied and against Soviet (48 games); the six full games are seed 7 (quick unset). Binaries: *main* is main without this branch; *round 3* is the same main with B1's and B2's brain changes (4b050ea) reversed, so the round-3 saving rebuilt on that main, not the binary B2 measured; *nosave* is a binary with both hold branches of `Brain::economy_first` forced off (scratch only, nothing of it is in the tree). The findings below were made on main f3aab5d; the results were measured twice, on f3aab5d and again after the rebase onto d38748f (step A5's animation playback, which changes play). Six quick games of seed 7 replay the same on 33b9844 as on d38748f.

### What the probes found (main f3aab5d)

- **No stall on main.** The Allied, map 2, seed 7 game with no winner at 120 000 ticks was B2's final binary on main 6fe1b84. On f3aab5d (B3 in) the six full games all have winners, as in B3's table above (Yuri 5 of 6), and so do the 48 quick games of seeds 7 to 14; seeds 7 to 10 play the same 24 games as B3's final quick lane. What the old game showed (B2's binary rerun with `GLASNOST_AI_ECON`): from about tick 60 000 the map was mined out and both houses lived on tech-building income (Allied 5 000 to 13 000 credits per 3 000 ticks, Yuri 1 500 to 5 000), feeding their armies in piece by piece. The Allied house was in drain blackout from tick 42 000 or earlier to the end (power output 0), its superweapons and defences dark, under one Yuri drainer in the air that no unit of the house targeted. It had built one anti-air type: `antiair::build_type` builds while the house has fewer objects able to hit the drainer than drainers, and `antiair::able` counted the house's anti-air defence, which stood dark (no power) and about 12 cells from the drainer. The same count shows on main in Allied houses drained for long (02-0-10: 114 027 drained-building ticks in a 69 793-tick game, 3 anti-air builds or fewer). Left as a finding: no main game stalls, and a change there plays the opponents differently, which would mix into the tempo comparison. The fix would count only a defence the firing check lets fire (`power::firing_answer` none: in operation, not drained) and within reach of a drainer.
- **The yard vehicle's prerequisite before the superweapon building.** The B1 unblocker (`Script::unblocker` of the yard vehicle) is ordered when the house would expand, and an order for a building whose line is busy queues behind the line's item (S-0404 claim 3). In 12-8-7 (Soviet, map 8, seed 7) it queued behind a power plant and started when the plant ended (tick 7 441); the next superweapon building, which round 3 started on the same line at tick 7 471, started at 8 641. Every later Yuri superweapon of the game is 1 000 ticks or more behind round 3 (force shield 13 790 → 14 799, mutator 16 732 → 19 419, dominator 29 445 → 34 032); 12-8-8 and 12-8-10 show the same.
- **The yard vehicle bought between superweapon buildings.** `Brain::rich` lets a house with its economy whole buy the yard vehicle with no money in hand when no superweapon building is on its lines (`superweapon_owed` none). That test misses the think in which `build_superweapon` orders the next building: the order is applied after the think, so `expand` in the same think sees no such building. In 12-8-7 (a probe every 15 ticks) the unblocker ended at 8 641, the yard vehicle became buildable at the think of tick 8 640, the superweapon building and the yard vehicle were ordered in that think, and the probe of tick 8 655 already read "superweapon building". The same test passes while a superweapon building is done but not placed, on hold, or not yet buildable while its own prerequisite is built. Over the 48 quick games, in 70 of the 177 probe samples (every 1 500 ticks) with a yard vehicle on Yuri's line, a superweapon building was on that line too.
- **What the yard vehicle then costs.** B2 holds it with the army while the house saves; saving stops whenever the house is not safe, and the vehicle draws money again. In 12-8-7 the yard vehicle was queued behind an army vehicle at 8 640 and was the Ground line's item from about 15 000, 17 of 54 steps done at 19 500; about a third of its price was paid before the mutator and the rest before the dominator. In 12-8-8 the whole yard vehicle was bought between the mutator and the dominator, which came 4 978 ticks after round 3's. And a held item keeps its line: the queue does not advance while the line is held (S-0404 claim 5), so a held yard vehicle stops every army vehicle queued behind it. A cancel (S-0404 claim 6) frees the line and refunds what was paid (at an unchanged price the refund is everything paid).
- **The yard vehicle bought while broke.** In 02-0-7 (Allied, map 0, seed 7) Yuri's superweapons were all built by tick 37 500 and the yard vehicle went on the line then; income was 275 to 600 credits per 1 500 ticks from 40 500 on, credits stayed under 30 in every sample, and the yard vehicle stood on the Ground line from 43 500 to the end (57 798) and never passed step 23 of 54, with army vehicles queued behind it. `rich` passed because the house kept every harvester it wanted (`Script::economy_kept`), which says nothing of what they earn on a nearly mined-out map. Of the 177 Yuri samples with a yard vehicle on the line, 172 had credits under 500 and 46 income under 1 500 per 1 500 ticks (02-0-7, 12-0-10 and 02-0-13 most; for the Allied and Soviet houses 27 of 129, 02-8-10, 12-0-8 and 12-8-14 most).
- **Where Yuri's tempo goes, by spending.** Credits spent by category between Yuri's superweapons first ready (`GLASNOST_AI_ECON`, main against round 3 on the same seeds): in 12-8-7, less than half of round 3's spending on superweapon buildings before the psychic reveal, the unblocker taking the difference; in 12-0-8 (Soviet, map 0), about eleven times round 3's spending on other buildings between force shield and mutator (the tech climb, not the expansion: the house bought no yard vehicle there, and its game differs from round 3's because the Soviet house expanded); in 02-0-7, spending on other buildings and on the yard vehicle between force shield and mutator, none in round 3.
- **The hold released while losing a fight.** B2 lifts the hold whenever the house is not safe, so that the army can answer the fight. Map 2's small delays come from there. In 12-2-7 the house was losing a fight from tick 8 100 to 8 340. The held items resumed, an engineer took 23 steps and a tank 8, and the mutator came 406 ticks after round 3's. Round 3 had no such release.

### What the brain does now (B5)

- **No yard vehicle before a superweapon building** (`Brain::rich`). The second wealth test (economy whole) now also needs no superweapon building on the lines *or left to go for* (`Brain::superweapon_next`, the search `build_superweapon` uses, split out of it). The test sees the next building in the think that orders it, and while it is done but not placed, on hold or waiting on its own prerequisite. The unblocker follows `rich`, so it waits too.
- **Income, not harvesters, for the second test.** The economy watch (`Script::watch_income`) adds up what the house earned: the change of its credits plus what its lines paid since the last think (S-0404 claim 4). A finished item pays the rest of its balance. A cancelled item's payments are refunded (S-0404 claim 6). A credit fall that no payment explains is not income. Every `INCOME_WINDOW` (3 000) ticks the sum becomes `Script::income`. The second test needs an income of one yard vehicle's cost per window (`EXPANSION_EARNING`). The first test (twice the cost in hand) now counts the money in hand beyond what the superweapon building still owes.
- **A broke house drops the yard vehicle** (`Brain::drop_expansion`). The house has a yard and a yard vehicle on a line or queued. If its income per window is under half the cost (`EXPANSION_BROKE`) and its credits do not cover what the yard vehicle and the superweapon building still owe, it sends the cancel-all order (S-0404 claim 6), counted as `Uses::expansion_cancels`. The threshold is below the buying threshold, so a house at the edge does not buy and cancel in turn. A yard vehicle bought with no yard (`rebuild_yard`) runs on. In 12-0-10 on f3aab5d, Yuri's yard vehicle stood on the line in 29 probe samples without the cancel (lane *a*, below) and in 5 with it.
- **Non-fighters stay held while losing a fight** (`Brain::economy_first`). Saving keeps B2's trigger. While the house is not safe and the building is still short, only army items that can join the fight resume. A yard vehicle and an engineer stay held.
  - **Deviation from B2's rule.** The plan said "never while the house is losing a fight", and this narrows it.
  - **Benefit not shown.** In 12-2-7 the engineer stayed held, but the tank line took the freed credits (the earlier line is paid first, S-0404 claim 1). The mutator stayed at 14 292. It is also the first step down in wins on f3aab5d (*eng*, below: 26 against main's 28). It is the first piece to revert if the plan's rule is wanted as written.
- **AI game format 31** (the watch's new fields, serde default). All six duel goldens of the sim gate are regenerated on the rebased branch: they change from their first line because the state hash includes the brain.

**Tried on the way (B5, on f3aab5d).** Same quick lane; medians against round 3.
- *v1*: the `rich` guard and the income test, plus a cancel of the yard vehicle whenever the superweapon building was short. Force shield 0, mutator +116, dominator −496. Yuri won 25. The mutator came later than round 3.
- *eng*: the non-fighter hold alone. Force shield 0, mutator 0, dominator +353.5. Yuri won 26.
- *a*: *eng* plus the `rich` guard (no income test, no cancel). Force shield 0, mutator −14.5, dominator +425. Yuri won 25. In 12-0-10 the yard vehicle bought while broke stood on the line in 29 samples.
- *Final*: *a* plus the income test and the broke cancel. Its superweapon tempo is nearly the same as *a*'s.

### Results on main d38748f

Yuri's superweapons, first tick ready, against round 3: the median difference, with games later / games compared.

| Binary | Force shield | Mutator | Dominator | Psychic reveal | Yuri wins vs Allied / Soviet | No winner |
| --- | --- | --- | --- | --- | --- | --- |
| round 3 | – | – | – | – | 3 / 18 (21) | 0 |
| main | +57.5 (23 / 44) | +145 (17 / 29) | +1 147 (12 / 16) | 0 (3 / 48) | 9 / 19 (28) | 0 |
| **final** | 0 (20 / 44) | +43.5 (16 / 28) | +2 380 (9 / 13) | 0 (0 / 48) | 7 / 17 (24) | 0 |
| final, nosave | +3 347 (35 / 36) | +7 496 (21 / 21) | +6 043 (3 / 3) | +22 (27 / 48) | 5 / 20 (25) | 1 |

- **Final against main:** force shield 0 (4 later of 44), mutator −29 (8 of 35), dominator +320.5 (10 of 18), reveal 0.
- **Final against final, nosave:** every superweapon is earlier with saving. Force shield −3 300 (1 later of 36), mutator −7 375 (0 of 23), dominator −5 759 (0 of 4), reveal −22. The dominator is ready in 20 games with saving and in 6 without.
- **Dominator against round 3**, the 13 games where both ready it, sorted (final minus round 3): −15 857, −2 558, −1 270, −230, +1 052, +2 145, +2 380, +3 058, +3 728, +5 645, +5 932, +8 282, +61 093. Ten of the 13 are Soviet, map 2 or 8. Only round 3 readies it in 6 games (Yuri lost 3 of them in final), and only final in 7.

**Brain counters, quick lane** (house 0, house 1):

| Binary | Expansion builds | Unblocks | Cancels | Superweapon holds | Games where Yuri had 2 yards | Dominator launches / changed an owner / objects taken |
| --- | --- | --- | --- | --- | --- | --- |
| round 3 | 0, 2 | 0, 0 | 0, 0 | 1 798, 1 422 | 8 | 27 / 19 / 23 |
| main | 23, 25 | 27, 16 | 0, 0 | 2 106, 2 113 | 17 | 31 / 12 / 19 |
| final | 19, 28 | 21, 7 | 3, 1 | 1 879, 2 097 | 15 | 20 / 8 / 11 |

**Full games, seed 7, main → final.** Every game has a winner. All replay to the same state and survive save and load. Yuri wins 5 of 6 in both. Yuri's superweapons are given as first tick ready.

| Opponent, map | Winner, tick | Force shield | Mutator | Dominator |
| --- | --- | --- | --- | --- |
| Allied, 0 | Yuri 43 035 → Yuri 52 169 | 15 107 → 15 107 | – → – | – → – |
| Allied, 2 | Yuri 42 692 → Yuri 61 146 | 10 710 → 10 710 | 14 263 → 14 495 | 30 582 → 59 962 |
| Allied, 8 | Allied 29 203 → Allied 32 673 | 15 603 → 14 211 | – → 18 527 | – → – |
| Soviet, 0 | Yuri 51 486 → Yuri 32 108 | 15 241 → 15 241 | 18 727 → 18 727 | 30 783 → 30 029 |
| Soviet, 2 | Yuri 47 145 → Yuri 28 796 | 10 710 → 10 710 | 14 060 → 13 886 | 24 180 → 25 941 |
| Soviet, 8 | Yuri 44 122 → Yuri 35 985 | 15 285 → 13 919 | 20 752 → 16 968 | 34 826 → 28 385 |

Soviet, map 8 is again at round 3's tempo (round 3's quick game: mutator 16 975).

**Dominator in the full games.** Main launched 6 times and 3 launches took a unit; final launched once (Soviet, map 8, tick 28 386) and it took a unit. In three of final's games the game ended 1 184 to 2 855 ticks after the dominator was ready, Yuri winning each (Soviet, maps 0 and 2, about 19 000 ticks sooner than on main; Allied, map 2).

**Exit B5 on d38748f: missed.**
- Met: force shield 0 and psychic reveal 0 against round 3; all six full games have a winner.
- Missed:
  - Mutator +43.5 and dominator +2 380 against round 3.
  - Wins against saving off: 24 against 25 (saving off has one game with no winner).
  - Full-game launches that took a unit: 1 (of 1), against B3's 2 of 4 and main's 3 of 6.
- Lower than main in both measurements: Yuri wins 24 against main's 28 (on f3aab5d and on d38748f). Quick-lane dominator launches 20 against 31.
- The tempo medians against round 3 do not hold from one base to the other: on f3aab5d final had mutator −14.5 and dominator +425, and main +58 and −1.5; one game now swings by 61 093 ticks. Medians of 13 to 28 compared games cannot settle differences of this size. What held on both: the mechanism fix of 12-8-7, and the four wins fewer than main.

### Results on main f3aab5d (same code, earlier main)

| Binary | Force shield | Mutator | Dominator | Psychic reveal | Yuri wins vs Allied / Soviet | No winner |
| --- | --- | --- | --- | --- | --- | --- |
| round 3 | – | – | – | – | 7 / 16 (23) | 0 |
| main | 0 (17 / 45) | +58 (17 / 27) | −1.5 (8 / 16) | 0 (2 / 48) | 7 / 21 (28) | 0 |
| main, nosave | +4 190 (33 / 34) | +7 043 (17 / 17) | +9 567 (1 / 1) | +22 (30 / 48) | 4 / 19 (23) | 1 |
| final | 0 (13 / 45) | −14.5 (12 / 26) | +425 (9 / 16) | 0 (0 / 48) | 7 / 17 (24) | 0 |
| final, nosave | +3 697 (33 / 34) | +7 135.5 (18 / 18) | +5 314 (1 / 1) | +22 (30 / 48) | 4 / 20 (24) | 1 |

Full games there: all six with a winner, Yuri 4 of 6 (main 5 of 6); dominator 4 launches, 2 took a unit (main 4 and 2); Soviet, map 8 at round 3's tempo (13 790, 16 471, 29 569). Quick-lane dominator launches / changed an owner / objects taken: round 3 41 / 28 / 42, main 31 / 14 / 17, final 27 / 16 / 18.

**Tests** (synthetic data).
- `tests/economy.rs`: `keeps_a_yard_vehicle_and_an_engineer_held_while_losing_a_fight`, `cancels_an_expansion_yard_vehicle_while_broke`, `reads_income_off_the_credits_and_the_lines`. The last is ignored in debug builds and passes in release.
- `tests/expansion.rs`: `a_house_earning_a_yard_vehicle_per_window_buys_one_step_by_step` and `a_house_with_a_superweapon_to_climb_to_buys_no_yard_vehicle`.

**Arena and duel sweeps (d38748f).** The oracle arena attaches no brain, and its CSVs are byte-identical to step A5's after-run: M-0001 33 of 96 pairings failing, 151 of 172 reproduced; M-0002 13 of 24, 43 of 47; M-0003 3 of 35, 61 of 64. The M6c, M7a and M7b seed sweeps 1 to 16 print the same min and median with and without this branch for every counter. For M7b: dominator launches 7/7, objects taken 14/18, mutator 5/7, infantry mutated 15/23, berserk orders 9/29, drain orders 4/6, drain links 3/3. (On f3aab5d likewise the same before and after.)

## Own AI (M8 step B4: gate margins)

The duel gates assert that each counter of their full run is above zero at one seed (the gate seed). B4 measured how far each counter stands from zero over seeds 1 to 32, with the default quirk switches and with each switch the duel reaches flipped. Then it fixed the zeros through the duel's setup, or through the own AI where the AI was at fault (D14: orders a person could give, no scripted choices). Lane outputs are git-ignored under `local-lane/m8/b4/` (counts and ticks only).

**Method** (`crates/sim-gate/tests/margins.rs`, local-only, every test ignored and doing nothing unless its variable is set).
- `margins_sweep` (`GLASNOST_SWEEP=1-32`, `GLASNOST_DUELS`, `GLASNOST_QUIRKS=default,<switch>,…`) plays each duel's full gate run per seed and setting and prints min, median, max and the seeds at zero per counter. "Asserted" counters are those the gate's full run asserts on; the others are reported only (for example the plain duels' rocket and aircraft flight, and M7b's view of M7a's counters).
- `quirk_reach` (`GLASNOST_REACH_SEEDS=gate`) decides reach: a switch is reached by a duel when flipping that switch alone changes the per-tick state hash after some tick of the full run at the duel's gate seed (duel and duel-naval 20 261 018, duel-diagonal 8, M6c 1, M7a 92, M7b 11), or panics (none did). The candidates are the 423 switches of `quirks.rs` that some file of the sim or AI crate reads, plus two unread controls (`chrono_flash_keeps_order_block`, `crate_phantom_place`), which no duel reached. The other 44 switches are read nowhere in those crates and cannot change a duel.
- Reached: duel 37, duel-diagonal 40, duel-naval 45, M6c 46, M7a 39, M7b 35. Each reached switch was then flipped for seeds 1 to 32 (32 games each).
- One-game diagnostics: `watch` (`GLASNOST_WATCH=<duel>,<seed>,<every>[,<switch>]`, with `GLASNOST_WATCH_TYPE=<type>,…` to trace objects), `hits` (`GLASNOST_HITS`) and `losses` (`GLASNOST_LOSSES`, `GLASNOST_LOSSES_NONE`, `GLASNOST_LOSSES_SHOTS`), with `GLASNOST_QUIRK` flipping one switch. `GLASNOST_THREADS` sets the workers and `GLASNOST_SWEEP_OUT` the lane CSV.

**Bases.** *Before* is main fd7e38a without this branch. The plain duels' *after* sweeps ran on this branch before the M6c, M7a and grinder changes (binary a173533); those changes do not reach the plain duels, whose goldens did not move with them. M6c and M7a *after* ran on the final fixtures and AI; M7b *after* ran with the grinder change, the only one of the later changes M7b reaches (the WARD and sensor setup is M7a's `margins`, which M7b's `base_ruleset` does not call). The rebased final binary gives the same per-counter min, median and max over seeds 1 to 16 for M6c, M7a and M7b (*Arena and duel sweeps* below).

### What made the zeros, and the fixes

- **Plain duels, no money** (duel seeds 7 and 29, duel-naval seed 7: no load). Both houses spent their last credit by about tick 6 800; a transport bought in place of a gun vehicle lost a house the game before it loaded a passenger. The plain duels now set the lobby's credits to 20 000 (S-0412 claim 2; `plain::setup`).
- **The ferry behind a busy line** (duel-naval seed 1: no load, no unload). The brain ordered a ferry only when the vehicle line stood idle, and a house short of money kept it busy at every think. The ferry now joins the line's queue when the queue has room (S-0404 claim 3), as a person queues it on the sidebar. Test `a_ferry_is_queued_behind_a_busy_vehicle_line`.
- **M6c's decoy** (seed 5: no jumpjet built). The strike aircraft killed the islet DECOY early, or a radiation deployer of its house deployed among its own army for it. The DECOY now has armour class special 1, at a versus value of zero for every warhead but the jumpjets' own JETW (S-0304 claim 1).
- **M6c's prism support reach** (with the fixes above alone: support beams at zero in 7 flipped games, default min 9; on fd7e38a default min 0). Support reach is the master's slot-1 weapon range (S-0518, Terms). The brain placed its prisms 10 to 18 cells apart, beyond the reach of 10. The reach is now 24 cells. Default support beams: min 72, median 101 (min 9, median 40 with the fixes above alone; min 0, median 56 on fd7e38a).
- **The charger's reach** (`linked::CHARGE_REACH`, own tuning; M6c seed 18 with `path_no_reopen` flipped: no charger order all game). The chargers built later stood idle at the barracks door 15 to 16 cells from the nearest prism and then left with the waves. A charger now looks 24 cells for an own coil. Test `sends_a_charger_at_a_coil_across_the_base`.
- **M7a's start TAKER** (seed 20: no takeover order). It now begins two cells toward the middle (`START_INFANTRY_AHEAD`), within the brain's control reach of the nearest PREY. This thinned the bunker margin (default vehicles bunkered min 1), which the WARD fix below restored.
- **M7a's WARD** (flipped seeds 12 and 13 bunkered no vehicle; from the B4 notes, the raw CSV of that run was overwritten). A berserk gattling of its own house or the enemy's killed it before the bunkers stood, or a controller took it. WARD now has armour class special 1, at a versus value of zero for every warhead (S-0304 claim 1). Default vehicles bunkered: min 8, median 11.
- **M7a's sensor radius** (after the WARD fix: default seed 10 saw hostile intent in 40 ticks, 5 221 without that fix, and eight flipped games in none). The sensor sees a unit whose target or destination lies within its radius (S-0610 claim 10); at 12 cells that was only raids into its own base. EYE now has a radius of 24 cells, beyond a start's 22 cells to the middle. The radius changes what the sensor shows, not play: the probed games (default seeds 10 and 22, and `absorber_stale_contact_kept` seeds 10 and 22) have every other counter the same.
- **The grinder's sides** (M7a seed 4 with `corridor_split_probe` flipped: no unit ground). The brain moved a far unit to the grinder's side facing it; the move never started, and every think clicked the same cell again. S-0212 claim 13a and its edge cases expect such failures at fewer alignments with the probe off, so this is no spec gap. As a person would, the brain now clicks the next nearest of the eight cells around the grinder once its order memory marks a side failed from where the unit stands. No new saved state: the AI game format stays 31. Test `a_far_unit_that_does_not_move_tries_another_side`.

**Tried on the way.**
- *Bait for the prisms* (M6c): a sturdy unarmed unit of steel armour beside the DECOY, which only the prism shot's own warhead touched, so that the prisms always had a target. Support beams fell to min 0, median 1 over the default seeds, with zeros at 11 of 32 seeds: all prisms attacked the bait together, and a building whose behaviour is *attack* may not support (S-0518 claim 2). Dropped; the reach of 24 alone gave min 72.

**Findings left as they are.**
- **`antiair::able`** (B5's finding: it counts an anti-air defence that stands dark or far from the drainer). It explains no zero or stall here: no plain-duel game stalled, and M7b's drain counters keep their margin (drain links min 3). Not changed.
- **Self-irradiation** (M6c). Hits with no attacker are radiation (S-0525; `radiation.rs` damage every 15 ticks). A radiation deployer deploying near an enemy inside its own army irradiates that army. This happens in default games too and does not by itself explain a zero, and a change there would move the radiation-exits margin. Not changed.
- **Thinned margins.** Over seeds 1 to 32 with the default switches, 18 asserted counters have a lower minimum than on fd7e38a, none at zero: M7a berserk weapon orders 30 to 7 (median 82 to 27) and units made berserk 61 to 16 (97 to 47), psychic sensor orders 2 to 1, hostile intent 2 336 to 388, psychic reveal orders and reveals fired 12 to 6, takeovers 6 to 4, grinder entry orders 5 to 4, lifter build orders 4 to 3, lift orders 6 to 4, vehicles seized 11 to 7, seized units dropped 10 to 5; M6c deploy orders 44 to 37, changes of deployed form 4 to 3, radiation exits 8 to 7, erasures begun 6 to 3; duel superweapon charging ticks 3 600 to 2 700 and launches 3 to 2. The M7a falls came with its setup changes (start TAKER, WARD); which change thinned which counter was not followed.
- **Residual zeros** (below): each is a game in which one house collapsed early or one rare event did not happen.

### Results

Seeds 1 to 32 per duel and setting. A median of an even count is the mean of the two middle values (the sweep's own text output prints the upper middle value instead). Reach: the switches each duel reaches, as decided above.

#### Default switches

**duel**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| transport loads (S-0507) | yes | 5 / 19 | 14 / 29 | none |
| transport unloads (S-0507) | yes | 1 / 18 | 10 / 26 | none |
| superweapon charging ticks (S-0512 claims 3, 5) | yes | 3 600 / 8 996.5 | 2 700 / 11 919 | none |
| superweapon launches (S-0513 claim 5) | yes | 3 / 12 | 2 / 13.5 | none |
| rocket flight ticks (S-0506) | no | 0 / 1 336 | 564 / 2 077 | none |
| aircraft flight ticks (S-0501) | no | 228 / 4 217.5 | 771 / 9 265.5 | none |

**duel-diagonal**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| transport loads (S-0507) | yes | 0 / 11 | 7 / 16 | none |
| transport unloads (S-0507) | yes | 0 / 10 | 7 / 16 | none |
| superweapon charging ticks (S-0512 claims 3, 5) | no | 0 / 4 239 | 981 / 10 097 | none |
| superweapon launches (S-0513 claim 5) | no | 0 / 4 | 1 / 16 | none |
| rocket flight ticks (S-0506) | no | 0 / 594.5 | 0 / 1 690 | 3, 23 |
| aircraft flight ticks (S-0501) | no | 0 / 459.5 | 0 / 6 461 | 3, 14 |

**duel-naval**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| transport loads (S-0507) | yes | 0 / 14.5 | 15 / 23.5 | none |
| transport unloads (S-0507) | yes | 0 / 14.5 | 15 / 21 | none |
| superweapon charging ticks (S-0512 claims 3, 5) | no | 0 / 9 692.5 | 3 119 / 12 891 | none |
| superweapon launches (S-0513 claim 5) | no | 0 / 11.5 | 3 / 15 | none |
| rocket flight ticks (S-0506) | no | 0 / 1 455 | 656 / 1 864.5 | none |
| aircraft flight ticks (S-0501) | no | 0 / 3 022 | 0 / 9 573.5 | 13 |

**duel-m6c**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| garrison orders (S-0508) | yes | 12 / 34 | 12 / 29.5 | none |
| garrison occupants inside (S-0508) | yes | 1 / 5.5 | 5 / 7 | none |
| engineer orders (S-0509) | yes | 5 / 6 | 6 / 6 | none |
| buildings captured (S-0509) | yes | 3 / 4 | 4 / 4 | none |
| spy orders (S-0510) | yes | 6 / 6 | 6 / 6 | none |
| infiltrations (S-0510) | yes | 4 / 6 | 6 / 6 | none |
| tech building orders (S-0515) | yes | 4 / 4 | 4 / 4 | none |
| tech building income cycles (S-0515) | yes | 888 / 2 343 | 2 307 / 2 343 | none |
| jumpjet orders (S-0502) | yes | 0 / 53.5 | 10 / 52.5 | none |
| jumpjet flight (S-0502) | yes | 0 / 8 673 | 9 107 / 9 108 | none |
| jumpjet units built and flown (S-0417 claim 15) | yes | 0 / 11 | 8 / 9.5 | none |
| deploy orders (S-0517) | yes | 44 / 80 | 37 / 82 | none |
| changes of deployed form (S-0517) | yes | 4 / 7.5 | 3 / 7 | none |
| radiation exits (S-0525) | yes | 8 / 42.5 | 7 / 36 | none |
| radiation in the field (S-0525) | yes | 1 802 / 5 548 | 2 015 / 5 038 | none |
| chrono erasure orders (S-0514) | yes | 20 / 31 | 20 / 33 | none |
| erasures begun (S-0514) | yes | 6 / 11.5 | 3 / 11.5 | none |
| prism buildings ordered (S-0518) | yes | 7 / 12 | 10 / 11 | none |
| charger orders (S-0518) | yes | 0 / 14 | 3 / 19.5 | none |
| support beams (S-0518) | yes | 0 / 56 | 72 / 101 | none |
| bomb planter orders (S-0519) | yes | 32 / 48 | 36 / 53 | none |
| bombs planted (S-0519) | yes | 14 / 29.5 | 18 / 29.5 | none |
| disarm orders (S-0519) | yes | 7 / 11.5 | 7 / 12 | none |
| suicide orders (S-0519) | yes | 7 / 22 | 11 / 21 | none |
| sabotage orders (S-0519) | yes | 2 / 18 | 9 / 18.5 | none |
| air-strike orders (S-0519) | yes | 2 / 7.5 | 3 / 14 | none |
| air strikes called (S-0519) | yes | 4 / 7 | 7 / 13 | none |
| crate orders (S-0520) | yes | 46 / 155.5 | 82 / 174.5 | none |
| crates picked up (S-0520) | yes | 2 / 8.5 | 4 / 13 | none |

**duel-m7a**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| berserk weapon orders (S-0610) | yes | 30 / 82 | 7 / 27 | none |
| units made berserk (S-0610 claim 4) | yes | 61 / 97 | 16 / 47 | none |
| psychic sensor orders (S-0610 claim 9) | yes | 2 / 2 | 1 / 2 | none |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | yes | 2 336 / 4 442 | 388 / 4 693 | none |
| psychic reveal orders (S-0610 claim 13) | yes | 12 / 12 | 6 / 11 | none |
| psychic reveals fired (S-0610 claims 13 and 14) | yes | 12 / 12 | 6 / 11 | none |
| slave-miner buildings placed or deployed (S-0402 claim 14) | yes | 12 / 13 | 12 / 13 | none |
| slaves at work (S-0402 claim 14) | yes | 2 287 / 7 413 | 3 489 / 7 255 | none |
| gattling stage rises (S-0609 claim 2) | yes | 6 / 17 | 8 / 21 | none |
| gattling at its top stage (S-0609 claim 2) | yes | 960 / 3 201 | 1 746 / 4 751.5 | none |
| takeover orders (S-0601) | yes | 0 / 2 | 1 / 6.5 | none |
| takeovers (S-0601 claim 6) | yes | 6 / 17 | 4 / 13 | none |
| gas clouds (S-0613 claims 2 to 4) | yes | 82 / 257.5 | 158 / 307.5 | none |
| gas hits (S-0613 claim 13) | yes | 310 / 2 084 | 426 / 1 261 | none |
| absorber entry orders (S-0605) | yes | 3 / 13 | 8 / 15.5 | none |
| infantry absorbed (S-0605 claims 1 and 7) | yes | 2 / 6.5 | 3 / 8 | none |
| grinder entry orders (S-0603) | yes | 5 / 10 | 4 / 9 | none |
| units ground (S-0603 claims 5 and 8) | yes | 3 / 5 | 3 / 6 | none |
| tank bunker build orders (S-0606) | yes | 4 / 4 | 4 / 4 | none |
| bunker entry orders (S-0606 claim 3) | yes | 4 / 9 | 9 / 13 | none |
| vehicles bunkered (S-0606 claim 7) | yes | 3 / 7 | 8 / 11 | none |
| bunker exit orders (S-0606 claim 12) | yes | 2 / 7 | 8 / 11 | none |
| lifter build orders (S-0607) | yes | 4 / 4 | 3 / 4 | none |
| lift orders (S-0607) | yes | 6 / 12 | 4 / 11 | none |
| vehicles seized (S-0607 claim 5) | yes | 11 / 16.5 | 7 / 16 | none |
| seized units dropped (S-0607 claim 13) | yes | 10 / 16 | 5 / 15 | none |

**duel-m7b**

| Counter | Asserted | Before: min / median | After: min / median | After: seeds at 0 |
|---|---|---|---|---|
| drain orders (S-0608) | yes | 4 / 6 | 4 / 6 | none |
| drain links formed (S-0608 claims 2 and 5) | yes | 3 / 3 | 3 / 3 | none |
| credits drained (S-0608 claims 7 to 10) | yes | 7 280 / 7 700 | 7 280 / 7 700 | none |
| drained-building ticks (S-0608 claim 1) | yes | 19 794 / 20 232.5 | 19 794 / 20 232.5 | none |
| mutator launches (S-0611 claim 2) | yes | 5 / 7 | 5 / 7 | none |
| infantry mutated (S-0611 claims 9 to 12) | yes | 15 / 23 | 15 / 23 | none |
| dominator launches (S-0612 claim 1) | yes | 6 / 7 | 6 / 7 | none |
| objects taken over (S-0612 claim 14) | yes | 13 / 19 | 13 / 19 | none |
| berserk weapon orders (S-0610) | no | 9 / 28.5 | 9 / 29 | none |
| units made berserk (S-0610 claim 4) | no | 11 / 21.5 | 11 / 21.5 | none |
| psychic sensor orders (S-0610 claim 9) | no | 1 / 1 | 1 / 1 | none |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | no | 0 / 0 | 0 / 0 | 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 32 |
| psychic reveal orders (S-0610 claim 13) | no | 1 / 1 | 1 / 1 | none |
| psychic reveals fired (S-0610 claims 13 and 14) | no | 1 / 1 | 1 / 1 | none |
| slave-miner buildings placed or deployed (S-0402 claim 14) | no | 3 / 12 | 3 / 12 | none |
| slaves at work (S-0402 claim 14) | no | 3 053 / 3 682.5 | 3 053 / 3 682.5 | none |
| gattling stage rises (S-0609 claim 2) | no | 0 / 0 | 0 / 0 | all |
| gattling at its top stage (S-0609 claim 2) | no | 0 / 0 | 0 / 0 | all |
| takeover orders (S-0601) | no | 0 / 1 | 0 / 1 | 2, 6, 9, 12, 13, 20, 25, 26, 27, 30, 32 |
| takeovers (S-0601 claim 6) | no | 0 / 2 | 0 / 2 | 3, 7, 8, 9, 14, 20, 23, 26, 32 |
| gas clouds (S-0613 claims 2 to 4) | no | 0 / 0 | 0 / 0 | all |
| gas hits (S-0613 claim 13) | no | 0 / 0 | 0 / 0 | all |
| absorber entry orders (S-0605) | no | 12 / 21.5 | 12 / 21 | none |
| infantry absorbed (S-0605 claims 1 and 7) | no | 3 / 7 | 3 / 7 | none |
| grinder entry orders (S-0603) | no | 4 / 10 | 4 / 10 | none |
| units ground (S-0603 claims 5 and 8) | no | 4 / 8 | 4 / 8 | none |
| tank bunker build orders (S-0606) | no | 3 / 4 | 3 / 4 | none |
| bunker entry orders (S-0606 claim 3) | no | 0 / 0 | 0 / 0 | 1, 4, 5, 6, 7, 8, 9, 11, 15, 17, 21, 22, 28, 29, 30, 31, 32 |
| vehicles bunkered (S-0606 claim 7) | no | 0 / 0 | 0 / 0 | 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32 |
| bunker exit orders (S-0606 claim 12) | no | 0 / 0 | 0 / 0 | all |
| lifter build orders (S-0607) | no | 1 / 1 | 1 / 1 | none |
| lift orders (S-0607) | no | 0 / 0 | 0 / 0 | all |
| vehicles seized (S-0607 claim 5) | no | 0 / 0 | 0 / 0 | all |
| seized units dropped (S-0607 claim 13) | no | 0 / 0 | 0 / 0 | all |

#### Each reached switch flipped

Per counter: the smallest value over all flipped games of the duel, with the switch and seed that gave it, the median over those games, and how many of them are at zero.

**duel** (1184 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| transport loads (S-0507) | yes | 8 (`stored_target_point`, 23) | 29 | 0 |
| transport unloads (S-0507) | yes | 8 (`corridor_split_probe`, 17) | 26 | 0 |
| superweapon charging ticks (S-0512 claims 3, 5) | yes | 1 614 (`spread_mark_kept_on_last_neighbour`, 30) | 11 700 | 0 |
| superweapon launches (S-0513 claim 5) | yes | 1 (`building_sight_top_left`, 2) | 14 | 0 |
| rocket flight ticks (S-0506) | no | 0 (`drive_leftover_kept_at_rest`, 30) | 2 212 | 3 |
| aircraft flight ticks (S-0501) | no | 0 (`building_sight_top_left`, 2) | 8 923.5 | 16 |

**duel-diagonal** (1280 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| transport loads (S-0507) | yes | 3 (`area_damage_per_foundation_cell`, 21) | 17 | 0 |
| transport unloads (S-0507) | yes | 2 (`drive_brake_floor_raises`, 1) | 16 | 0 |
| superweapon charging ticks (S-0512 claims 3, 5) | no | 363 (`detour_mirrored_reference`, 15) | 10 169 | 0 |
| superweapon launches (S-0513 claim 5) | no | 0 (`detour_mirrored_reference`, 15) | 14 | 4 |
| rocket flight ticks (S-0506) | no | 0 (`aircraft_smoke_scenario_draw`, 3) | 1 764.5 | 70 |
| aircraft flight ticks (S-0501) | no | 0 (`aircraft_smoke_scenario_draw`, 3) | 6 674 | 66 |

**duel-naval** (1440 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| transport loads (S-0507) | yes | 9 (`corridor_split_probe`, 17) | 25 | 0 |
| transport unloads (S-0507) | yes | 6 (`corridor_split_probe`, 18) | 23 | 0 |
| superweapon charging ticks (S-0512 claims 3, 5) | no | 1 207 (`last_bale_lost`, 25) | 12 382.5 | 0 |
| superweapon launches (S-0513 claim 5) | no | 1 (`building_sight_top_left`, 2) | 14 | 0 |
| rocket flight ticks (S-0506) | no | 0 (`drive_leftover_kept_at_rest`, 30) | 1 853 | 3 |
| aircraft flight ticks (S-0501) | no | 0 (`aircraft_smoke_scenario_draw`, 13) | 8 751 | 64 |

**duel-m6c** (1472 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| garrison orders (S-0508) | yes | 8 (`corridor_split_probe`, 6) | 28 | 0 |
| garrison occupants inside (S-0508) | yes | 4 (`approach_alternative_walk`, 3) | 7 | 0 |
| engineer orders (S-0509) | yes | 5 (`approach_alternative_walk`, 23) | 6 | 0 |
| buildings captured (S-0509) | yes | 4 (`airstrike_recharge_same_target_only`, 1) | 4 | 0 |
| spy orders (S-0510) | yes | 5 (`chop_rounding`, 5) | 6 | 0 |
| infiltrations (S-0510) | yes | 5 (`chop_rounding`, 5) | 6 | 0 |
| tech building orders (S-0515) | yes | 4 (`airstrike_recharge_same_target_only`, 1) | 4 | 0 |
| tech building income cycles (S-0515) | yes | 1 101 (`path_no_reopen`, 15) | 2 343 | 0 |
| jumpjet orders (S-0502) | yes | 1 (`approach_alternative_walk`, 5) | 48 | 0 |
| jumpjet flight (S-0502) | yes | 3 132 (`path_no_reopen`, 15) | 9 108 | 0 |
| jumpjet units built and flown (S-0417 claim 15) | yes | 1 (`flatten_straight_run_v`, 23) | 9 | 0 |
| deploy orders (S-0517) | yes | 30 (`chop_rounding`, 21) | 78 | 0 |
| changes of deployed form (S-0517) | yes | 3 (`approach_alternative_walk`, 26) | 7 | 0 |
| radiation exits (S-0525) | yes | 1 (`path_refresh_keeps_old_step`, 11) | 36 | 0 |
| radiation in the field (S-0525) | yes | 1 801 (`jumpjet_turn_slowdown_one_sided`, 21) | 5 024 | 0 |
| chrono erasure orders (S-0514) | yes | 9 (`update_order_removal_skip`, 23) | 33 | 0 |
| erasures begun (S-0514) | yes | 3 (`area_damage_per_foundation_cell`, 23) | 11 | 0 |
| prism buildings ordered (S-0518) | yes | 7 (`approach_alternative_walk`, 5) | 11 | 0 |
| charger orders (S-0518) | yes | 0 (`flatten_straight_run_v`, 23) | 18 | 1 |
| support beams (S-0518) | yes | 0 (`detour_mirrored_reference`, 23) | 101 | 4 |
| bomb planter orders (S-0519) | yes | 27 (`corridor_retry_no_fallback`, 25) | 51 | 0 |
| bombs planted (S-0519) | yes | 13 (`path_refresh_keeps_old_step`, 5) | 30 | 0 |
| disarm orders (S-0519) | yes | 5 (`airstrike_recharge_same_target_only`, 2) | 12 | 0 |
| suicide orders (S-0519) | yes | 4 (`path_refresh_keeps_old_step`, 29) | 20 | 0 |
| sabotage orders (S-0519) | yes | 1 (`detour_mirrored_reference`, 23) | 17 | 0 |
| air-strike orders (S-0519) | yes | 1 (`building_sight_top_left`, 32) | 14 | 0 |
| air strikes called (S-0519) | yes | 4 (`spread_attempts_count_failures`, 3) | 13 | 0 |
| crate orders (S-0520) | yes | 38 (`flatten_straight_run_v`, 23) | 159 | 0 |
| crates picked up (S-0520) | yes | 2 (`building_sight_top_left`, 22) | 13 | 0 |

**duel-m7a** (1248 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| berserk weapon orders (S-0610) | yes | 2 (`foundation_exit_lists`, 14) | 29 | 0 |
| units made berserk (S-0610 claim 4) | yes | 13 (`spread_mark_kept_on_last_neighbour`, 12) | 51 | 0 |
| psychic sensor orders (S-0610 claim 9) | yes | 1 (`absorber_stale_contact_kept`, 10) | 2 | 0 |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | yes | 0 (`corridor_retry_no_fallback`, 22) | 5 075 | 3 |
| psychic reveal orders (S-0610 claim 13) | yes | 6 (`absorber_stale_contact_kept`, 10) | 11 | 0 |
| psychic reveals fired (S-0610 claims 13 and 14) | yes | 6 (`absorber_stale_contact_kept`, 10) | 11 | 0 |
| slave-miner buildings placed or deployed (S-0402 claim 14) | yes | 5 (`jumpjet_step_truncation`, 31) | 13 | 0 |
| slaves at work (S-0402 claim 14) | yes | 2 549 (`corridor_split_probe`, 24) | 7 313 | 0 |
| gattling stage rises (S-0609 claim 2) | yes | 7 (`flatten_straight_run_v`, 14) | 20 | 0 |
| gattling at its top stage (S-0609 claim 2) | yes | 1 661 (`corridor_retry_no_fallback`, 5) | 4 420 | 0 |
| takeover orders (S-0601) | yes | 1 (`absorber_stale_contact_kept`, 16) | 6 | 0 |
| takeovers (S-0601 claim 6) | yes | 4 (`absorber_stale_contact_kept`, 15) | 15 | 0 |
| gas clouds (S-0613 claims 2 to 4) | yes | 136 (`corridor_retry_no_fallback`, 5) | 289 | 0 |
| gas hits (S-0613 claim 13) | yes | 426 (`approach_alternative_walk`, 23) | 1 166.5 | 0 |
| absorber entry orders (S-0605) | yes | 3 (`detour_mirrored_reference`, 1) | 15 | 0 |
| infantry absorbed (S-0605 claims 1 and 7) | yes | 0 (`flatten_straight_run_v`, 27) | 8 | 1 |
| grinder entry orders (S-0603) | yes | 4 (`absorber_stale_contact_kept`, 15) | 10 | 0 |
| units ground (S-0603 claims 5 and 8) | yes | 2 (`corridor_split_probe`, 16) | 6 | 0 |
| tank bunker build orders (S-0606) | yes | 2 (`spread_mark_kept_on_last_neighbour`, 21) | 4 | 0 |
| bunker entry orders (S-0606 claim 3) | yes | 4 (`spread_stage_from_index`, 31) | 13 | 0 |
| vehicles bunkered (S-0606 claim 7) | yes | 4 (`spread_stage_from_index`, 31) | 11 | 0 |
| bunker exit orders (S-0606 claim 12) | yes | 3 (`spread_stage_from_index`, 31) | 11 | 0 |
| lifter build orders (S-0607) | yes | 2 (`jumpjet_step_truncation`, 11) | 4 | 0 |
| lift orders (S-0607) | yes | 3 (`absorber_stale_contact_kept`, 26) | 12 | 0 |
| vehicles seized (S-0607 claim 5) | yes | 3 (`spread_mark_kept_on_last_neighbour`, 21) | 16 | 0 |
| seized units dropped (S-0607 claim 13) | yes | 3 (`corridor_split_probe`, 13) | 15 | 0 |

**duel-m7b** (1120 flipped games)

| Counter | Asserted | Min (switch, seed) | Median | Games at 0 |
|---|---|---|---:|---:|
| drain orders (S-0608) | yes | 3 (`chop_rounding`, 12) | 6 | 0 |
| drain links formed (S-0608 claims 2 and 5) | yes | 3 (`absorber_stale_contact_kept`, 2) | 3 | 0 |
| credits drained (S-0608 claims 7 to 10) | yes | 7 240 (`spread_stage_from_index`, 3) | 7 960 | 0 |
| drained-building ticks (S-0608 claim 1) | yes | 19 341 (`spread_stage_from_index`, 31) | 20 402 | 0 |
| mutator launches (S-0611 claim 2) | yes | 4 (`chop_rounding`, 4) | 7 | 0 |
| infantry mutated (S-0611 claims 9 to 12) | yes | 10 (`jumpjet_step_truncation`, 25) | 22 | 0 |
| dominator launches (S-0612 claim 1) | yes | 6 (`absorber_stale_contact_kept`, 2) | 7 | 0 |
| objects taken over (S-0612 claim 14) | yes | 7 (`detour_mirrored_reference`, 2) | 19 | 0 |
| berserk weapon orders (S-0610) | no | 9 (`absorber_stale_contact_kept`, 1) | 28 | 0 |
| units made berserk (S-0610 claim 4) | no | 9 (`jumpjet_step_truncation`, 13) | 22 | 0 |
| psychic sensor orders (S-0610 claim 9) | no | 1 (`absorber_stale_contact_kept`, 1) | 1 | 0 |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1062 |
| psychic reveal orders (S-0610 claim 13) | no | 0 (`absorber_stale_contact_kept`, 25) | 1 | 17 |
| psychic reveals fired (S-0610 claims 13 and 14) | no | 0 (`absorber_stale_contact_kept`, 25) | 1 | 17 |
| slave-miner buildings placed or deployed (S-0402 claim 14) | no | 2 (`drive_brake_floor_raises`, 30) | 12 | 0 |
| slaves at work (S-0402 claim 14) | no | 2 713 (`table_direction`, 20) | 3 683 | 0 |
| gattling stage rises (S-0609 claim 2) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1119 |
| gattling at its top stage (S-0609 claim 2) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1119 |
| takeover orders (S-0601) | no | 0 (`absorber_stale_contact_kept`, 6) | 1 | 395 |
| takeovers (S-0601 claim 6) | no | 0 (`absorber_stale_contact_kept`, 3) | 2 | 317 |
| gas clouds (S-0613 claims 2 to 4) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1119 |
| gas hits (S-0613 claim 13) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1119 |
| absorber entry orders (S-0605) | no | 10 (`corridor_split_probe`, 3) | 21 | 0 |
| infantry absorbed (S-0605 claims 1 and 7) | no | 0 (`spread_mark_kept_on_last_neighbour`, 2) | 8 | 1 |
| grinder entry orders (S-0603) | no | 0 (`spread_attempts_count_failures`, 2) | 9 | 1 |
| units ground (S-0603 claims 5 and 8) | no | 0 (`absorber_stale_contact_kept`, 25) | 8 | 11 |
| tank bunker build orders (S-0606) | no | 3 (`absorber_stale_contact_kept`, 2) | 4 | 0 |
| bunker entry orders (S-0606 claim 3) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 579 |
| vehicles bunkered (S-0606 claim 7) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1011 |
| bunker exit orders (S-0606 claim 12) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1110 |
| lifter build orders (S-0607) | no | 1 (`absorber_stale_contact_kept`, 1) | 1 | 0 |
| lift orders (S-0607) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1120 |
| vehicles seized (S-0607 claim 5) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1120 |
| seized units dropped (S-0607 claim 13) | no | 0 (`absorber_stale_contact_kept`, 1) | 0 | 1120 |

#### Smallest asserted counter per switch and duel

"–": the duel does not reach the switch.

| Switch flipped | duel | duel-diagonal | duel-naval | duel-m6c | duel-m7a | duel-m7b |
|---|---:|---:|---:|---:|---:|---:|
| `absorber_stale_contact_kept` | – | – | – | – | 1 | 3 |
| `aircraft_smoke_scenario_draw` | – | 7 | 15 | – | – | – |
| `aircraft_unit_step_stall` | – | 7 | – | – | – | – |
| `airstrike_recharge_same_target_only` | – | – | – | 4 | – | – |
| `anim_damage_starts_at_one` | – | – | – | – | – | 3 |
| `approach_alternative_walk` | – | 7 | 9 | 1 | 1 | – |
| `area_damage_per_foundation_cell` | 2 | 3 | 14 | 3 | 1 | 3 |
| `building_pitch_term` | 2 | 7 | 15 | – | – | – |
| `building_sight_top_left` | 1 | 7 | 13 | 1 | 1 | 3 |
| `building_smudge_discarded_draws` | 2 | 6 | 10 | 3 | – | – |
| `cash_cycle_one_short` | – | – | – | 3 | – | – |
| `chop_rounding` | 3 | 7 | 11 | 4 | 1 | 3 |
| `corridor_retry_no_fallback` | 2 | 9 | 9 | 2 | **0** (hostile intent seen by a sensor, seed 22) | 3 |
| `corridor_split_probe` | 3 | 10 | 6 | 2 | 1 | 3 |
| `crate_count_humans_only` | 2 | 7 | 15 | – | 1 | 3 |
| `detour_mirrored_reference` | 2 | 3 | 13 | **0** (support beams, seed 23) | 1 | 3 |
| `dominator_marks_own_objects` | – | – | – | – | – | 3 |
| `drive_brake_floor_raises` | 3 | 2 | 14 | 1 | 1 | 3 |
| `drive_duplicate_scatter_request` | – | – | 15 | 3 | – | – |
| `drive_leftover_kept_at_rest` | 2 | 7 | 14 | 3 | 1 | 3 |
| `drive_subpoint_compounding` | 2 | 7 | 13 | 3 | 1 | 3 |
| `erasure_double_experience` | – | – | – | 3 | – | – |
| `fire_reveal_expires` | 2 | 6 | 11 | 3 | 1 | – |
| `flatten_straight_run_v` | 2 | 6 | 12 | **0** (charger orders, seed 23; support beams, seed 23) | **0** (infantry absorbed, seed 27) | 3 |
| `flying_listed_double_hit` | – | – | – | 3 | 1 | – |
| `foundation_exit_lists` | 2 | 6 | 12 | 3 | 1 | 3 |
| `full_cell_no_regrowth` | 2 | 8 | 16 | 3 | 1 | – |
| `ground_detonation_misses_aircraft` | – | – | 14 | 3 | – | – |
| `infantry_fixed_turn_rate` | 2 | 7 | 15 | 3 | 1 | 3 |
| `jumpjet_step_truncation` | – | – | – | 2 | 1 | 3 |
| `jumpjet_turn_slowdown_one_sided` | – | – | – | 3 | 1 | 3 |
| `last_bale_lost` | 2 | 8 | 11 | 3 | 1 | 3 |
| `lift_impact_removes_survivor` | – | – | – | – | 1 | – |
| `ownership_gain_counts_as_kill` | – | – | – | 3 | 1 | 3 |
| `path_congestion_toggle` | 5 | 7 | – | – | 1 | – |
| `path_no_reopen` | 2 | 4 | 16 | **0** (support beams, seed 15) | 1 | 3 |
| `path_refresh_keeps_old_step` | 2 | 5 | 14 | **0** (support beams, seed 29) | 1 | 3 |
| `placed_buildings_not_ever_built` | 2 | 7 | 15 | 3 | 1 | 3 |
| `placed_buildup_twice` | 2 | 7 | 15 | 3 | 1 | 3 |
| `prism_count_survives_dropped_shot` | – | – | – | 3 | – | – |
| `rail_missile_vanishes_with_spawner` | – | – | 15 | – | – | – |
| `resume_restarts_step` | – | 7 | 15 | – | – | – |
| `rocket_aim_at_heading_point` | 2 | 6 | 13 | 3 | – | – |
| `search_one_object_per_cell` | 3 | 3 | 13 | 3 | 1 | – |
| `set_facing_keeps_target` | 2 | – | 15 | 3 | – | – |
| `sidebar_tie_newest_first` | 2 | 7 | 15 | 3 | 1 | 3 |
| `sight_bonus_wrap` | – | – | – | – | – | 3 |
| `sight_centre_off_map` | 2 | 7 | 15 | – | – | – |
| `spread_attempts_count_failures` | 2 | 3 | 15 | 1 | – | 3 |
| `spread_mark_kept_on_last_neighbour` | 1 | 4 | 16 | 3 | **0** (hostile intent seen by a sensor, seed 21) | 3 |
| `spread_stage_from_index` | 2 | 5 | 10 | 3 | 1 | 3 |
| `stale_suspended_revert` | – | – | 15 | 2 | – | – |
| `stored_target_point` | 2 | 7 | 8 | 2 | 1 | – |
| `table_direction` | 2 | 4 | 13 | 1 | 1 | 3 |
| `table_root_distance` | 3 | 7 | 15 | 2 | 1 | 3 |
| `turret_reaim_after_turn` | 2 | 7 | 14 | 2 | 1 | 3 |
| `unit_changes_no_recheck` | – | – | 15 | 3 | – | 3 |
| `unload_facing_gap_no_wrap` | – | 5 | 15 | – | – | – |
| `update_order_removal_skip` | 2 | 4 | 15 | 1 | **0** (hostile intent seen by a sensor, seed 29) | 3 |
| `uphill_arc_45` | – | – | – | 3 | 1 | – |
| `vehicle_buildings_keep_nobody_alive` | – | – | – | – | 1 | 3 |
| `zero_load_period_stalls` | 2 | – | 15 | – | – | – |

Reached switches per duel: duel 37, duel-diagonal 40, duel-naval 45, duel-m6c 46, duel-m7a 39, duel-m7b 35.

### Residual zeros (flipped switches)

Nine of the 7 744 flipped games have an asserted counter at zero; no default game has one.
- **M6c support beams**, 4 games. `path_no_reopen` seed 15, `path_refresh_keeps_old_step` seed 29 and `flatten_straight_run_v` seed 23 end with a winner at ticks 6 024, 7 032 and 6 580. `detour_mirrored_reference` seed 23 runs its 12 000 ticks, but the losing house keeps only two strike aircraft from early on, and the winner's three prisms stand on guard all game with nothing to fire at.
- **M6c charger orders**, 1 game: `flatten_straight_run_v` seed 23, the same early end.
- **M7a hostile intent seen by a sensor**, 3 games: `corridor_retry_no_fallback` seed 22, `spread_mark_kept_on_last_neighbour` seed 21, `update_order_removal_skip` seed 29. In the first (followed with `watch`), one house's sensor stood from about tick 6 900 to the end, the other house built none, and no enemy unit aimed at or moved to a point within 24 cells of it. The other two were not followed.
- **M7a infantry absorbed**, 1 game: `flatten_straight_run_v` seed 27. Four absorbers stood from early on and the brain gave 16 entry orders, and no infantry unit got in. Not followed further.

M6c's flipped zero games went from 10 (before the prism and charger reaches) to 7 (prism reach) to 5. Some came back in every round (seed 23 with `detour_mirrored_reference` and with `flatten_straight_run_v`, seed 15 with `path_no_reopen`): games in which one house collapses early, where a counter that needs fighting near the prisms cannot rise. Keeping a losing house alive would change the duel's balance and was not done.

### Exit B4: met for the default switches, missed for the flipped switches

- Met: with the default switches, every asserted counter of all six duels is above zero at every seed of 1 to 32 (the tables above). With any reached switch flipped, the three plain duels and M7b have no asserted zero.
- Missed: 9 of 7 744 flipped games (M6c 5 of 1 472, M7a 4 of 1 248), above.

**Formats and goldens.** The AI game format stays 31: no change adds saved state (the grinder's sides use the existing order memory). The goldens of duel, duel-diagonal and duel-naval were regenerated for the 20 000 credits and the queued ferry; duel-m6c for the decoy (first step), then for the two reaches of 24 (from about tick 5 620); duel-m7a for the start TAKER, then for WARD's armour and the grinder's sides (from about tick 1 640); duel-m7b for the grinder's sides only (from about tick 7 370). The sensor radius changes no play. The gate seeds' full runs still exercise every asserted counter.

**Tests** (synthetic data). `a_ferry_is_queued_behind_a_busy_vehicle_line` (`tests/ai.rs`), `a_far_unit_that_does_not_move_tries_another_side` (`tests/grinder.rs`), `sends_a_charger_at_a_coil_across_the_base` (`tests/linked.rs`).

**Arena and duel sweeps (310dba1, on main 63c1df4).** The oracle arena attaches no brain, and its CSVs are byte-identical to the before-run on fd7e38a: M-0001 33 of 96 pairings failing, 151 of 172 reproduced; M-0002 13 of 24, 43 of 47; M-0003 3 of 35, 61 of 64. The duels' own seed sweeps 1 to 16 (`m6c_seed_sweep`, `m7a_seed_sweep`, `m7b_seed_sweep`; upper-middle medians), before (fd7e38a) and after:
- M6c: smallest minimum 0 before (support beams, charger orders, jumpjet orders, flight and units, sabotage orders), 3 after (air-strike orders, 3 before as well; M7a's units ground is also 3). Over these 16 seeds the final binary gives the same min, median and max per counter as the frozen binaries' CSVs behind the 32-seed tables (M6c 29, M7a 26 and M7b 34 counters). Support beams 0/54 to 78/101, charger orders 0/11 to 10/22, jumpjet units built and flown 0/12 to 8/10.
- M7a: smallest minimum 1 before and after (takeover orders, median 2 to 7). Vehicles bunkered 3/7 to 8/10, bunker exit orders 2/7 to 8/10, hostile intent 2 709/4 524 to 2 239/4 611. Berserk weapon orders fell 32/84 to 7/25 and units made berserk 63/108 to 26/48 (see *Thinned margins* above).
- M7b: every drain, mutator and dominator counter the same (drain links min 3); of M7a's reported-only counters, seed 12 gives one bunker entry order where it gave none.

## Own AI (M8 step B6: the anti-air count and the tempo misses)

Follow-up of B5 (above) and of B5's open finding, in `glasnost_ai::antiair`, the placement of a finished building (`Script::think`, `Brain::place_done`, `Script::place_near`) and the lane (`crates/tools/src/skirmish_lane.rs`). Lane outputs are git-ignored under `local-lane/m8/b6/` (counts and ticks only). The lanes are B5's: the quick lane is seeds 7 to 14, maps 8, 2 and 0, Yuri (house 1) against Allied and against Soviet (48 games); the six full games are seed 7 (quick unset: a full game plays the same game as the quick game of its seed and adds the replay and save checks). Binaries, each rebuilt on the tree it is compared with: *final* is the tree as it stands; *round 3* is the same tree with B1's, B2's and B5's brain changes reversed (B5's patch, rebased); *nosave* is the same tree with both hold branches of `Brain::economy_first` forced off (scratch only). The findings were made on main 8dcc429 (*main*); the branch was then rebased onto 7118390 (tests only changed) and measured there, and rebased again onto main 2dc52af (the map flags, the tick speed-ups, aircraft tracking) and measured again.

**The anti-air probe** (`GLASNOST_AI_AAPROBE=<ticks>`, `antiair::probe`, `Brain::antiair_probe`, `Game::antiair_probes`): every so many ticks the lane prints, per house that sees an enemy drainer, the drainers in the air and what answers each (kind, type index, cell, distance, behaviour, destination, the firing check's answer with range, the weapon's range, both heights, whether it targets the drainer), the drainers linked to the house's buildings that are not in the air, the answers on its lines, the type `build_type` would order, the candidate types by test passed (hits, lit when placed, startable, line free), the army units and defences of a hitting type by test passed, and the first unit the firing check refuses (type, slot, answer).

### What the probes found (main 8dcc429)

- **Defences that cannot fire, units that cannot get there** (B5's open finding). `antiair::able` counted every unit, defence and line item of a type that can hit the drainer, wherever it stood and whether or not it was in operation. The quick lane's one game without a winner on main (02-0-10) is that stall. With the first fix below, the probe shows the drainer over the Allied base unanswered from tick 17 250: the one anti-air unit near by passes the firing check but has no cell within its range in its reach region, and the anti-air items on the lines never finish (credits about 4, income 0, power 0; the Yuri house is broke too).
- **A type whose anti-air weapon is never fired at the air.** `antiair::hits_air` asked whether any weapon slot of a type, elite slots included, has an anti-air projectile, a versus value above zero and damage above zero. The weapon choice fires slot 0 for a type whose secondary has the never-use flag (S-0308 claim 4, table P row 3) and for an infantry type with the deploy-fire flag until it deploys (claim 8), and a produced object is not elite (S-0311 claims 6 and 11). In 12-2-10 and 02-2-12 (after the first fix) the house's army units of a "hitting" type stood near the drainer and the firing check refused them as *illegal* with slot 0: in 34 probe samples, 29 were a type whose anti-air weapon is a never-use secondary, 2 a deploy-fire infantry type, 3 a type whose anti-air primary the check refuses for its warhead. The never-use type was the cheapest "anti-air" type, so `build_type` chose it, and `lined` counted it on the lines as an answer.
- **A unit standing off the drainer.** In 02-2-12 an anti-air vehicle was counted as the drainer's answer from tick 17 000 to 24 000 from the same cell, 8 to 11 cells from the drainer: it held the drainer as its target, its weapon range is 6 cells, the firing check answered *out of range*, it was under no attack order and its destination was its own cell. Nothing more was built while the drainer drained on. The base defence leaves such a unit alone too (`Brain::defend`: a unit whose target is a threat is busy).
- **Why the dominator is behind round 3 (main).** The delay is in the grant (the building done), not in the charge. From the mutator granted to the dominator granted (40 games), Yuri earned 1 642 credits per 1 000 ticks on main, 2 086 on round 3 and 2 610 with saving off, and lost 245 refineries and 216 harvesters on main against 130 and 117 on round 3. Three mechanisms showed in the probe games:
  - *The line held by an engineer.* B5 keeps a non-fighter held while the house is short and not safe; a held item keeps its line and the queue does not advance (S-0404 claim 5), and the script orders nothing for a busy line. In 12-2-13 the engineer stood at step 44 of 54 from tick 8 400 to 19 666, in 02-2-8 from 6 110 to 19 846; round 3 finished it at 9 031 and 8 881 and built ten or more infantry on that line in the meantime.
  - *The dominator item lost with its prerequisite.* When a building a type needs is lost, the type leaves the offer and its item is cancelled with refund (S-0407 claim 13, S-0404 claim 9). In 12-2-13 and 02-2-8 on main the dominator item was cancelled at step 32 and step 42 of 54 (ticks 19 606 and 19 861) when its prerequisite was lost; the house rebuilt the prerequisite (1 380 and 2 580 ticks) and started the dominator again from step 0. Round 3 lost neither.
  - *Saving stops when the house is not safe.* Of 268 probe samples (every 1 500 ticks) in which a superweapon building of Yuri was short, the house saved in 128; the others were not safe: intruders in 106, outnumbered in 71, losing a fight in 64 (a sample can have several reasons), losing alone in 12. While it does not save, the army lines take the money: in 02-2-11, Yuri spent 16 767 credits on army vehicles in the window on main against 2 225 on round 3. And raids took the economy: in 12-2-13 a slave miner was lost at about tick 12 000, then a refinery and a power plant to infantry at 13 500.
- **Why the dominator took less in the full games (B5): not on this main.** On main the six full games launched the dominator 11 times, 7 sweeps changed an owner and 9 objects were taken (B5's final on d38748f: 1 launch, 1 taken). The quick lane: main 41 launches, 27 sweeps changing an owner, 31 objects taken; round 3 36, 25, 33.
- **The win against saving off: not lost on this main.** Yuri wins 28 of 48 quick games on main (9 against Allied, 19 against Soviet, one game without a winner), 27 with saving off (9, 18) and 24 on round 3 (6, 18).

**Tried and dropped: B5's first revert.** B5 named the non-fighter hold (an engineer and a yard vehicle held while losing a fight) as the first piece to revert. Main with it reverted (*q2*): Yuri wins 26 (7 against Allied, 19 against Soviet) against main's 28, two games without a winner (02-8-8, 12-8-8: both houses broke, the Allied house drained, its drainer unanswered); dominator ready +1 335 against round 3 (13 later of 18 compared; only round 3 readies it in 7 games, only q2 in 1), against main −284 (7 later of 17; only main readies it in 8 games, only q2 in 2); 35 dominator launches, 12 sweeps changing an owner, 21 objects taken (main 41, 27, 31). The engineer no longer blocks the line, but the line's other items take the money and the dominator is readied in fewer games. The hold stays as B5 left it.

### What the brain does now (B6)

- **A defence answers only when it can fire and reach** (`antiair::defence_answers`): its own answer of the firing check is none (S-0405 claim 9, table T1: in operation, not drained, not building or selling), the firing check without the range test is neither *illegal* nor *impossible* (S-0301 claim 9; the anti-air and versus tests of S-0306 claim 23 and S-0304 claim 1 among them), and the drainer or the building it drains or is after stands in range (S-0307; S-0608 claim 2).
- **A unit answers only when it can get there** (`antiair::unit_answers`): it stands within `RECALL` cells of home or of the drainer, is not berserk (S-0610 claim 6), passes the firing check, flies (an aircraft or a jumpjet) or its movement zone's reach region holds a cell within its range of the drainer (S-0212 claim 4; S-0307 F1's air range bonus), and does not stand off the drainer (`antiair::stands_off`: it holds the drainer as its target, the check with range answers *out of range*, S-0301 claim 9 rank 9, under no attack order and going nowhere; nocite, what a person sees).
- **Each answer counts once**, for the first drainer in threat order it answers (`antiair::answers`).
- **A type counts by the weapon a new object would choose** (`antiair::hits_air`, `antiair::air_slots`): its ordinary slots only (S-0311 claims 6 and 11, S-0308 claim 2); slot 0 for a deploy-fire infantry type (S-0308 claim 8) and for an empty or never-use secondary (claim 4, row 3); otherwise table P rows 6, 15, 16, 19 and 20 against an object in the air (`quirks.secondary_wins_against_air` read). A switching or gattling type (rows 1 and 5) and a draining secondary (row 8) choose by the object's own state, so every ordinary slot counts and the firing check decides once the object is on the map. The same test chooses the type to build and counts the answers on the lines.
- **A defence type the blackout would keep dark** is neither counted on a line nor chosen (`antiair::lit_when_placed`: S-0405 claim 8's powered and no-blackout rows, claim 5's blackout; table T1's pulse cannon, occupiable and needs-engineer types).
- **A finished anti-air defence goes beside the drainer it answers** (`antiair::place_goal`, `Script::place_near`, in both placement paths): the first legal cell of a square spiral around the drained building or the drainer, as a person drops a defence beside the building under attack.
- No new brain state: the AI game format stays at main's 37.

**Left open.** A house too broke to finish its anti-air items keeps counting them on its lines (02-0-10: two or three items for thousands of ticks at credits under 20); a person could do no more. A type whose anti-air primary the firing check refuses for its warhead against the drainer (the 3 samples above) still counts when not on the map. The base defence (`Brain::defend`) still leaves a standing-off unit alone; only the anti-air count no longer takes it for an answer.

### Results on main 2dc52af

Yuri's superweapons, first tick ready, against round 3 on the same tree: the median difference, with games later / games compared. *Main* is main 2dc52af as it stands, compared with the same round 3.

| Binary | Force shield | Mutator | Dominator | Psychic reveal | Yuri wins vs Allied / Soviet | No winner |
| --- | --- | --- | --- | --- | --- | --- |
| round 3 | – | – | – | – | 7 / 19 (26) | 0 |
| main | 0 (15 / 43) | 0 (14 / 30) | +1 296 (16 / 23) | 0 (0 / 48) | 9 / 19 (28) | 1 |
| **final** | 0 (15 / 43) | 0 (14 / 30) | +1 202 (14 / 21) | 0 (0 / 48) | 7 / 20 (27) | 1 |
| final, nosave | +3 677 (37 / 37) | +7 642 (20 / 22) | +7 646 (3 / 3) | +22 (27 / 48) | 8 / 19 (27) | 1 |

- **Dominator against round 3:** only round 3 readies it in 5 games, only final in 3 (main: 3 and 2). On main 8dcc429, against its own round 3, main was +1 827 (15 later of 21; only round 3 4 games, only main 4).
- **Final against main:** force shield, mutator and reveal ready at the same tick in every game; dominator 0 (1 later of 23; only main readies it in 2 games, only final in 1). The anti-air fixes change the end tick or the winner of 29 of the 48 games and the superweapon ticks almost nowhere.
- **Winners changed against main:** Allied now wins 02-2-12 and 02-8-9, Yuri now wins 12-2-9. The game without a winner is still 02-0-10, main's stall (both houses broke; see *Left open*). The nosave game without a winner is 02-2-10.
- **Final against final, nosave:** every superweapon is earlier with saving: force shield −3 357.5 (0 later of 36), mutator −7 584 (0 of 22), dominator −2 481 (0 of 3), reveal −22. The dominator is ready in 24 games with saving and in 5 without.
- **Dominator launches / sweeps that changed an owner / objects taken**, quick lane: round 3 43 / 28 / 32, main 39 / 26 / 29, final 44 / 25 / 32, nosave 8 / 7 / 8.
- **On 7118390** (before main's map flags, tick speed-ups and aircraft tracking) every lane played the same games but one: 12-0-7 ended at 81 370 there and at 41 773 here (main likewise, 81 370 and 41 764), and in nosave 12-0-11 at 62 237 and 60 138. Every median and win count above is the same on both bases; the launch counts there were final 46 / 26 / 34 and main (8dcc429) 41 / 27 / 31.

**Full games, seed 7, main 8dcc429 → final on 2dc52af.** Every game has a winner. All replay to the same state and survive save and load. Yuri wins 4 of 6 in both, the same games. Every superweapon is ready at the same tick as on main but the dominator of Soviet, map 0 (58 ticks sooner); three games end later, one sooner.

| Opponent, map | Winner, tick | Force shield | Mutator | Dominator |
| --- | --- | --- | --- | --- |
| Allied, 0 | Allied 24 139 → Allied 24 139 | 13 485 | – | – |
| Allied, 2 | Yuri 51 495 → Yuri 52 637 | 10 710 | 14 089 | 27 656 |
| Allied, 8 | Allied 32 594 → Allied 32 594 | 14 211 | – | – |
| Soviet, 0 | Yuri 81 370 → Yuri 41 773 | 15 163 | 18 631 | 31 911 → 31 853 |
| Soviet, 2 | Yuri 66 572 → Yuri 119 145 | 10 710 | 13 828 | 24 257 |
| Soviet, 8 | Yuri 50 054 → Yuri 59 697 | 13 896 | 17 910 | 28 504 |

Soviet, map 0 is the one game aircraft tracking changed (main 2dc52af ends it at 41 764 too). Soviet, map 2 ends 855 ticks before the 120 000-tick cap (on 7118390 likewise; the quick game of seed 7 is the same game). The dominator launched 13 times in the six games, 11 sweeps changed an owner and 12 objects were taken (on 7118390 15 / 12 / 14; main 8dcc429 11 / 7 / 9; B5's final on d38748f 1 launch, 1 taken).

**Exit B6: missed on the dominator's tempo.**
- Met: wins with saving 27, with saving off 27. Force shield, mutator and psychic reveal ready no later than round 3 (median 0). All six full games have a winner (one by 855 ticks before the cap). B3's dominator numbers do not fall below B5's: 13 launches and 12 objects taken in the full games against B5's 1 and 1.
- Missed: the dominator is ready +1 202 ticks behind round 3 (14 later of 21). The mechanisms are above (the held engineer, the item lost with its prerequisite, saving that stops while the house is not safe); B6 changes none of them, and B5's first revert was tried and dropped (above).
- 02-0-10 still has no winner: the anti-air count there is now true (the drainer is unanswered), but neither house can pay for an answer.

**Tests** (synthetic data, through the real tick), `crates/deterministic/ai/tests/antiair.rs`, 12 in all. New: `a_type_that_hits_the_air_only_when_elite_is_not_built`, `a_never_use_anti_air_secondary_is_not_built`, `a_deploy_fire_infantry_type_does_not_hit_the_air_with_its_secondary`, `a_dark_defence_does_not_answer_and_no_dark_defence_is_chosen`, `a_lit_defence_in_reach_answers_and_nothing_more_is_built`, `a_defence_out_of_reach_does_not_answer_and_the_new_one_goes_beside_the_drainer`, `an_anti_air_unit_near_home_answers`, `an_anti_air_unit_standing_off_the_drainer_does_not_answer`. Kept: `builds_an_anti_air_unit_against_a_drainer_in_the_air`, `builds_none_with_one_able_unit_per_drainer`, `builds_nothing_without_an_anti_air_type`, `only_the_anti_air_type_hits_the_disc`. Goldens unchanged.

**Arena and duel sweeps.** The oracle arena attaches no brain; on 7118390 its CSVs are byte-identical to main 8dcc429's: M-0001 31 of 96 pairings failing, 152 of 172 reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64. The M6c, M7a and M7b seed sweeps 1 to 16 print the same min and median before and after for every counter, on 7118390 against 8dcc429 and on 2dc52af against main 2dc52af, with the same values on both bases: smallest minimum M6c 4, M7a 1 (takeover orders), M7b drain 3; for M7b, dominator launches 7/7, objects taken 14/18, mutator 5/7, infantry mutated 15/23, berserk orders 9/29, drain orders 4/6, drain links 3/3. B4's 32-seed default margins sweep writes a byte-identical CSV before and after on both bases, with no asserted counter at zero.

**Load-sensitive tests.** On 2dc52af the whole `cargo test --workspace` passed, `two_clients_meet_chat_and_play_in_lockstep` (glasnost-app) and `three_persons_and_a_computer_play_over_the_relay_and_survive_a_drop` (glasnost-relay) among them. On 7118390 the first failed under load and passed alone.

## Own AI (M8 step B7: the dominator's grant and the think tick)

Follow-up of B6 (above): the think tick of an eight-house game, and the three mechanisms B6 found behind the dominator's delay, in our own AI only (`glasnost_ai::brain`, `scripted`, `deployers`, `special`). Lane outputs are git-ignored under `local-lane/m8/b7/` (counts and ticks only). The lanes and binaries are B6's: the quick lane is seeds 7 to 14, maps 8, 2 and 0, Yuri (house 1) against Allied and against Soviet (48 games); the full games are the six of seed 7; *round 3* is the final tree with B1's, B2's and B5's brain changes reversed and *nosave* the final tree with both hold branches of `Brain::economy_first` forced off, each rebuilt on the tree it is compared with. Measured on main bfa4575.

### The think tick

Every brain thought in the ticks that are multiples of `THINK_PERIOD` (15). C4 part 2's benchmark (`glasnost-headless tick-bench`, map 9, seed 7, eight houses, 11 399 ticks; three interleaved runs of each binary) on main: all 760 of those ticks over a 16.7 ms frame, median 53.7 / 53.0 / 56.3 ms, 1 358 / 1 345 / 1 376 ticks over the frame in all, mean 9.02 / 8.93 / 9.61 ms, worst 90.6 / 92.8 / 117.3 ms.

Now each house's brain plans in the ticks `t ≡ phase (mod THINK_PERIOD)`, `scripted::think_phase`: phases 0, 7, 11, 3, 13, 5, 10, 2 by house number (nocite: our own AI's schedule). House 0 keeps phase 0; the table leaves out 4, 9 and 14, where C4 part 2 found the vehicle movement's cost peaking, and puts a duel's two houses half a period apart. A phase is a function of the house number alone: it needs no saving, is the same on every peer and stays put when a house leaves. The reactions still run in every tick. `Script::watch_income` reads an income window at the first think at or after its end, whatever the phase. With the spread: 139 / 212 / 217 of the phase-0 ticks over the frame, median about 15 ms; phases 3 and 7 a mean of about 11 to 12 ms; 951 / 1 047 / 1 119 ticks over the frame in all; mean 8.63 / 8.95 / 9.24 ms. The worst tick, 123.1 / 129.1 / 138.1 ms, is now a movement tick (phase 4), not a think tick. The benchmark plays a different game with the spread (another end hash), the same one on every run. On the final tree (cause 1 and the deployer fixes below; two runs, main rerun alongside at 1 359 over, mean 9.03 ms): the benchmark plays another game again. In that game 18 / 25 of the phase-0 ticks are over the frame (median 13.5 / 13.4 ms), and 125 / 132 ticks are over the frame in all. The mean is 4.31 ms and the worst tick 70.6 / 72.0 ms. The movement peaks at phases 4 and 9 are mostly gone in this game; B7 did not look into why.

### The dominator's grant

- **The line held by an engineer** (cause 1). B5 held a non-fighter (an engineer, a yard vehicle) while the house was short of a superweapon building's cost and not safe. A held item keeps its line, and the queue does not advance (S-0404 claim 5). Now `Brain::economy_first` cancels such an item with a refund (S-0404 claims 6 and 9; `Uses::non_fighter_cancels`), and the think orders no new non-fighter while the house is short. A person cancels the engineer to free the line, not keeps it paused. The army items are held as before. Cancels per house and game in the quick lane (96): none in 43, 1 in 33, 2 in 12, 3 in 3, 4 in 2, 6 in 3.
- **The dominator item lost with its prerequisite** (cause 2). In the final lane, 6 dominator items ended before step 54 of 54, mostly in Yuri's games against Allied on map 8, while the base was being destroyed. That is not a cheap loss to avoid, and B7 changes nothing there.
- **Saving that stops while the house is not safe** (cause 3). The final lane saved in 580 of 1 263 probe samples in which a superweapon building of Yuri was short (B6 on main: 128 of 268). *Tried and dropped:* not counting a fight lost far from home (more than `RECALL` cells) as an end to the saving (`Brain::losing_at_home`, kept for the probe). The army lines then waited while an attack abroad went badly. Against the same round 3 (the first tree, without the deployer fix below): wins 24 (7 against Allied, 17 against Soviet) against cause 1 alone's 29 (10, 19); dominator ready in 19 games against 26; dominator gap +3 356 against +1 167; two games without a winner. A lost fight anywhere still ends the saving.
- **Deployed riflemen that never moved again.** With cause 1, the quick game 02-0-7 had no winner. The Allied riflemen had been deployed out of range of the enemy's last buildings and stood so from tick 37 500 to the end. Deployed infantry drops a destination (S-0215 claim 22) and takes no step towards its target (S-0517 claim 16), so every wave's order was dropped, and `deployers` packed no infantry up. Now:
  - A firing infantry unit is deployed only when a seen enemy it can attack stands in the range of the weapon it would use (S-0215 claim 37 b, S-0307), and packed up when none does.
  - The brain gives a unit it deploys or packs up no behaviour order in the same think: S-0213 claims 3 and 10 say the deploy order queues unload, and a behaviour order of the same tick for the same actor would determine that state.
  - `Script::idle` does not count a deployed infantry unit as idle.
- **The radiation-immune deployer** (a follow-up fix). The in-range rule also caught a radiation-immune deploy-fire infantry unit, which fires its secondary at its own cell when deployed (S-0215 claim 31 rows 3 to 5) and does not use the deployed-firing rule. It packed up at once and deployed only in its rifle's range. M6c's radiation exits fell from a median of 41 to 3, and B4's margins sweep had a required counter at zero (radiation exits, M6c). Now such a unit deploys when a seen enemy it can attack is within `NEAR` and packs up when no seen enemy is within `GONE`, as a deployed in-place vehicle does. The quick and full lanes play every game the same with and without this fix.
- AI game format 38 (`Uses::non_fighter_cancels`). The probe line gains the intruders by kind and `losing at home`; the uses line gains the non-fighter cancels.

### Results

Yuri's superweapons, first tick ready, against round 3 on the same tree: the median difference, with games later / games compared.

| Binary | Force shield | Mutator | Dominator | Psychic reveal | Yuri wins vs Allied / Soviet | No winner |
| --- | --- | --- | --- | --- | --- | --- |
| round 3 | – | – | – | – | 8 / 18 (26) | 1 |
| **final** | 0 (16 / 45) | +87 (18 / 29) | +1 167 (10 / 14) | 0 (0 / 48) | 9 / 19 (28) | 1 |
| final, nosave | +3 032 (38 / 39) | +7 523 (27 / 28) | +19 395 (5 / 5) | +22 (29 / 48) | 9 / 21 (30) | 0 |

- **Dominator against round 3:** final readies it in 24 games, round 3 in 19 (only round 3 in 5, only final in 10). B6's final readied it in 24 games too, against its round 3's 26.
- **From the mutator granted to the dominator granted** (summed over the games with the mutator granted): Yuri earned 1 663 credits per 1 000 ticks against round 3's 1 856, and lost 243 refineries and 226 harvesters against 198 and 175.
- **Final against nosave:** with saving, every superweapon is sooner (force shield −2 930.5, mutator −7 627, dominator −8 980, reveal −22), and the dominator is ready in 24 games against 7.
- **The games without a winner:** final 02-2-13 is a live game at the 120 000-tick cap, with both houses building (23 and 18 buildings) and the map mined out. Round 3's 02-0-7 also reaches the cap.
- **Dominator launches / sweeps that changed an owner / objects taken**, quick lane: round 3 34 / 20 / 25, final 33 / 21 / 28, nosave 9 / 5 / 7.

**Full games, seed 7.** Every game has a winner. All six replay to the same state and survive save and load. Yuri wins 5 of 6 (B6: 4 of 6).

| Opponent, map | Winner, tick | Force shield | Mutator | Dominator |
| --- | --- | --- | --- | --- |
| Allied, 0 | Yuri 52 061 | 13 558 | 21 793 | – |
| Allied, 2 | Yuri 37 873 | 10 702 | 14 864 | 32 651 |
| Allied, 8 | Allied 28 414 | 13 886 | 18 813 | – |
| Soviet, 0 | Yuri 69 439 | 14 442 | 18 022 | 37 269 |
| Soviet, 2 | Yuri 43 126 | 10 702 | 13 820 | 25 641 |
| Soviet, 8 | Yuri 44 208 | 13 925 | 16 830 | 29 658 |

In the six games the dominator launched 7 times, 4 sweeps changed an owner and 4 objects were taken (B6: 13 / 11 / 12, counted the same way from the games' final reports). Four of the six games end sooner than in B6, and the six together play 275 121 ticks against B6's 330 285. Whether the shorter games account for the drop was not checked.

**Exit B7: missed on the dominator's tempo and, narrowly, on the wins.**
- Met: all six full games have a winner. Force shield and psychic reveal are at round 3's tempo (median 0). The think tick is no longer over the frame in every period (above).
- Missed: the dominator is +1 167 ticks behind round 3 (10 later of 14), though it is ready in 24 games against round 3's 19. The mutator is +87 (18 later of 29). Wins with saving are 28, with saving off 30. Cause 1 is fixed. Cause 3's change was tried and played worse. Cause 2 happens while the base is being destroyed.

**Tests** (synthetic data, through the real tick).
- `crates/deterministic/ai/tests/game.rs`: new `the_brains_plan_in_different_ticks` and `a_save_between_the_houses_thinks_plays_on_alike`.
- `economy.rs`, 17 in all. New: `a_house_thinking_off_the_window_ticks_reads_its_income_too`, `cancels_a_yard_vehicle_and_an_engineer_while_losing_a_fight`, `a_cancelled_engineer_leaves_its_line_and_is_not_started_again` and `stops_saving_while_losing_a_fight_far_from_home`.
- `deployers.rs`, 7 in all. New: `does_not_deploy_a_rifleman_out_of_range`, `packs_up_a_deployed_rifleman_with_no_enemy_in_range` and `deploys_a_radiation_immune_unit_near_an_enemy_and_packs_it_up_without_one`. `deploys_deploy_fire_vehicles_and_firing_infantry` now puts the enemy in the rifle's range.
- Goldens: all six duel goldens were regenerated once. The AI game format is in the hashed state from the first tick, and duel-diagonal ends about 1 300 ticks later. duel-m6c was regenerated again for the radiation-immune deployer, with the hashes differing from line 41 and the same length.

**B6's anti-air count and C4's memos.** The anti-air tests changed only to wait for Beta's think phase (`scripted::thinks_at`) instead of a multiple of 15, with no assertion changed, and all 12 pass. `Brain::think_with` opens the placement memo (`Script::begin_think`) at its top in every tick and closes it at its end, whether or not the plan runs, so a memo never outlives a tick. `the_kept_and_once_asked_placement_answers_match_the_three_passes` passes.

**Arena and duel sweeps.** The oracle arena attaches no brain, and its CSVs are byte-identical to main's. M-0001: 31 of 96 pairings failing, 152 of 172 reproduced. M-0002: 13 of 24, 43 of 47. M-0003: 2 of 35, 61 of 64. The M6c, M7a and M7b seed sweeps 1 to 16 move with the AI's play. The smallest minimum stays the same: M6c 4, M7a 1 and M7b drain 3.
- M6c: radiation exits 6 / 41 → 5 / 19 (min / median), radiation in the field 2 300 / 4 646 → 4 936 / 7 216, deploy orders 49 / 82 → 52 / 122.
- M7b: dominator launches 7 / 7 → 6 / 7, objects taken 14 / 18 → 13 / 17, mutator launches 5 / 7 → 5 / 8, drain orders 4 / 6 → 3 / 3.

In B4's 32-seed default margins sweep, no required counter is at zero. The smallest required minimums are duel 3, diagonal 4, naval 14, M6c 4, M7a 1 and M7b 3 (B6: 2, 5, 12, 1, 1, 3). Four duels now reach the 40 000-tick cap without a result: duel seeds 2 and 30 and naval seeds 2 and 28. B6 had none, and diagonal still has none. Bisected, the spread alone gives all four, and main's brain ends them. No AI code depends on the think phase beyond `thinks_at` and the income window. In duel seed 2, both houses are broke and mined out by tick 35 000. House 0 then destroys house 1's last buildings one jet sortie at a time from its one helipad (a power plant between 39 000 and 40 000) and is not done by the cap.

**Left open.** The dominator's tempo gap. The long, broke endgames of the four duels above. The full-game dominator count is down (7 launches against B6's 13). No requests.

## Own AI (M8 step B8: the economy with driving chrono miners)

Follow-up of S-0207 revision 82 (claim 28: the chrono miner drives, except on docking). The step looks at the Allied AI's economy in our own AI only (`glasnost_ai::scripted`, `brain`, `power`). Lane outputs are git-ignored under `local-lane/m8/b8/` (counts and ticks only). The quick lane is seed 7, maps 8, 2 and 0, 120 000 ticks at most, `GLASNOST_AI_QUICK=1`, with reference rules (*f0*) and with `GLASNOST_PLAY_FIXES=1` (*f1*). It plays the side pairs Allied–Soviet, Allied–Yuri, Soviet–Yuri, Allied–Allied and Soviet–Soviet (30 games). The Yuri mirror is left out: the lane registers one Yuri player country, so house 1 of a Yuri mirror gets nothing and the game has no winner, which is a limit of the lane and not of the AI. *Main* is main 7aabc33 with the probe commit; *after* is the branch. Both were rebuilt on the same tree. *e20* is the credits a house earned up to tick 20 000, summed from the lane's economy lines (`GLASNOST_AI_ECON=5000`).

**The economy probe** (`GLASNOST_AI_ECOPROBE=<ticks>`, `Brain::economy_probe`). Every so many ticks it prints per AI house: the refineries with the explored resource near each, each harvester's cell, behaviour, phase, load and distances, the role the plan builds next, the refinery types it can build and place, every item on its lines with line kind, role, progress, balance owed and hold, and the economy-first and selling states. At the end it prints each house's harvester cycles: ticks out, loading, back and unloading, and the distance from the loading cell to the nearest refinery.

### What the probe found (main)

- **A lost refinery that was never rebuilt.** Allied against Yuri on map 0 lost its only refinery at about tick 24 500. The plan wanted a refinery, but the building line held a power plant, and the script orders nothing for a busy line. With credits near 0, `sell_when_broke` then sold the construction yard first, the costliest building. The two chrono miners stood for the rest of the game (S-0402 claim 13: a harvester whose owner has no refinery queues *guard*).
- **The refinery put behind power, over and over.** With the refinery wanted first (the first fix below), the probe showed a loop. `power::plant_first` replaced the refinery's build order with a power plant, since the refinery's drain would leave the house short. At the next think `economy_first` cancelled the plant for the refinery. Credits stayed at 428 and the refinery was never built.
- **The cycle of a driving chrono miner.** In the Allied games, a cycle's median was 340 to 650 ticks out, 550 to 665 loading, about 50 back (the chrono jump) and 32 unloading. The median distance from the loading cell to the nearest refinery was 10 to 16 cells, and 30 in Allied–Soviet on map 2. Since claim 28 the miner drives out and jumps back, so the drive out is a third of each cycle. The refinery stood where the yard's placement spiral first fit it, not by the field. The Soviet war miner drives both ways: a median of 1 110 to 1 320 ticks loading and 330 to 600 back.
- **The home field mined down.** With the refinery beside the home field (the third fix below), the miners on maps 2 and 8 loaded a median of 15 to 34 cells from their only refinery once that field was mined down. On map 2 the home field is about 24 cells.
- In the main lane, f0 and f1 end on the same tick in 13 of 15 games (only Allied–Yuri on maps 0 and 2 differ, after tick 20 000), with the same winner in all 15.

### What the brain does now (B8)

- **A lost refinery is rebuilt first** (`Script::refinery_lost`). This applies to a house that keeps harvesters (not slave miners, which are their slaves' refinery, S-0402 claim 14) and owns no refinery, either because it lost them all or because it started with a harvester and has built none. The house wants the cheapest refinery type it can build and place, before power and also while selling out (`Script::wanted_role`, `Script::think`). `power::plant_first` leaves that order alone (its new `before_power` argument). `Brain::paying` is true while the refinery is not on the lines, so `economy_first` holds the rest of the sidebar for it. The item on the refinery's line is cancelled with a refund, not held (S-0404 claim 6: a held item keeps its line, claim 5). While selling out, `sell_when_broke` keeps the construction yard that builds the refinery and sells the rest first.
- **A refinery goes beside the field** (`Script::place_refinery`, in both placement paths, `Script::think` and `Brain::place_done`). The target is the explored resource cell nearest home with at least `CELLS_PER_HARVESTER` explored resource cells within `FIELD_REACH`, which is a field that keeps a harvester at work (`Script::ore_goal`), or with none such, the explored resource cell nearest home. The refinery takes the legal cell of the wider spiral around the yard whose foundation centre is nearest that target, with no resource under its foundation and the gap narrowing as in `place_cell`. A harvester leaves its refinery for the field after every load, so this shortens every trip, as a person places it. If no resource is explored, it falls back to the old placement.
- **A refinery beside the next field once the home field runs dry** (`Script::field_refinery`). The rule applies to a house with harvesters and a refinery when fewer than `CELLS_PER_HARVESTER` explored resource cells lie within `FIELD_REACH` of its refineries, the `ore_goal` field lies beyond that reach of every refinery, no refinery is on its lines, and a refinery fits with its foundation within that reach of the field. It wants a refinery when nothing else is wanted. A building that comes with a free unit gives a harvester with it (S-0402 claim 18). There is one per field, since the new refinery brings that field within reach.
- **A slave miner's building is left out** of all three (`keeps_slaves`: a building type whose objects keep slaves, S-0402 claim 19). `Script::slave_miners` asks about the harvester types the house can build now, so before its first war factory a Yuri-style house counted as a house with harvesters. In the first after-run, `place_refinery` put its slave-miner building beside the field. M7a's sweep showed it: slave-miner buildings placed or deployed fell from 13 / 13 to 4 / 4 (min / median), and in the lane Yuri earned 13.7 % less to tick 20 000. Now the three rules leave out every refinery type that keeps slaves, so for a house whose only refinery type is a slave-miner building none of them orders, cancels or places anything; the slave-miner rules play such a house as on main.
- The economy probe lists every line item.
- The AI game format stays 45: no new saved state.

**Tried and dropped.**
- *The lost refinery first without the power exemption.* This was the loop above: the refinery was never built.
- *Fixes 1 and 2 without the placement.* In the f0 lane, Allied won 2 of the 6 games against Soviet and Yuri, the same as main. The synthetic `plays_to_the_last_side` (ai `skirmish.rs`) failed: both houses ended broke in a mined-out endgame with 2 explored resource cells, not deadlocked. It passes with the placement.
- *The placement without the field refinery* (fixes 1 to 3). Allied won 3 of 6 against Soviet and Yuri. The median loading distance fell on map 0 (13 to 6–8 cells, ticks out 450 to 280–310), was the same on map 8, where the field is unexplored when the first refinery is placed, and rose on map 2 (10 → 33, 12 → 19, 8 → 18) once the small home field was mined down. That led to the field refinery, which is kept: Allied e20 rises 0.9 % against fixes 1 to 3, Soviet 5 to 11 %, and the map-8 loading distances fall (below).

### Results

Quick lane, f0, main → after. Winner and tick, then e20 of house 0 / house 1.

| Sides, map | Main | After |
| --- | --- | --- |
| Allied–Soviet, 0 | Soviet 63 247; 59 614 / 60 023 | Allied 41 832; 58 553 / 49 221 |
| Allied–Soviet, 2 | Allied 48 233; 69 903 / 71 616 | Soviet 49 453; 77 556 / 83 549 |
| Allied–Soviet, 8 | Allied 32 598; 52 776 / 54 196 | Allied 35 239; 54 495 / 63 642 |
| Allied–Yuri, 0 | Yuri 40 533; 51 367 / 51 949 | Yuri 49 662; 53 134 / 43 992 |
| Allied–Yuri, 2 | Yuri 42 163; 62 531 / 61 981 | Yuri 35 431; 58 516 / 62 163 |
| Allied–Yuri, 8 | Yuri 47 051; 46 111 / 45 180 | Yuri 65 367; 51 198 / 53 270 |
| Soviet–Yuri, 0 | Yuri 45 431; 57 579 / 70 665 | Soviet 76 380; 64 073 / 41 152 |
| Soviet–Yuri, 2 | Yuri 19 475; 49 241 / 48 412 | Yuri 33 678; 53 984 / 57 670 |
| Soviet–Yuri, 8 | Yuri 39 216; 63 173 / 59 887 | Yuri 42 605; 83 573 / 54 072 |
| Allied–Allied, 0 | house 1 25 546; 63 515 / 70 392 | house 1 36 940; 76 062 / 58 106 |
| Allied–Allied, 2 | house 1 90 498; 89 769 / 71 095 | house 1 21 777; 78 958 / 88 276 |
| Allied–Allied, 8 | house 0 93 024; 60 016 / 59 132 | house 0 32 959; 63 429 / 59 273 |
| Soviet–Soviet, 0 | house 1 51 666; 58 797 / 49 770 | house 1 59 394; 58 347 / 51 693 |
| Soviet–Soviet, 2 | house 1 65 116; 69 539 / 73 945 | house 1 23 916; 74 349 / 86 452 |
| Soviet–Soviet, 8 | house 1 55 262; 56 619 / 48 024 | house 0 24 328; 93 443 / 49 826 |

- **e20 by side over the nine games of two sides**: Allied 342 302 → 353 452 (+3.3 %), Soviet 355 828 → 398 042 (+11.9 %), Yuri 338 074 → 312 319 (−7.6 %).
- **Wins**: Allied against Soviet 2 of 3 in both. Allied against Yuri 0 of 3 in both. Soviet against Yuri 0 → 1 of 3. Every game has a winner, every run exits 0, and there are no panics.
- **Game length**: the 15 games take 759 059 → 628 961 ticks. The mirrors on maps 2 and 8 end at 21 777 to 32 959 instead of 55 262 to 93 024.
- **f1** ends on the same tick as f0 in 11 of 15 games after (Allied–Allied on maps 0 and 8, Allied–Yuri on map 8 and Soviet–Yuri on map 0 differ, after tick 20 000), with the same winner in all 15. Its e20 is the same as f0's in every game.
- **Yuri gets worse, through its opponents.** None of the B8 rules orders anything for the Yuri house (above), but its results fall. Against Allied, Yuri's e20 is flat (159 110 → 159 425) and it wins all three games in both. Against Soviet, Yuri's e20 falls from 178 964 to 152 894 while Soviet's rises from 169 993 to 201 630, and Yuri's wins fall from three to two. The lead should judge this against the step's bar that Soviet and Yuri must not get worse. The loss comes from a stronger Soviet economy and not from a change to Yuri's play. Yuri still wins five of its six games.

Harvester cycles (economy probe, the same games): the median distance from the loading cell to the nearest refinery and the completed cycles of the first Allied house, main → after. Both are over the whole game, so a shorter game has fewer cycles.

| Sides, map | Distance (cells) | Cycles |
| --- | --- | --- |
| Allied–Soviet, 0 | 13 → 8 | 90 → 86 |
| Allied–Soviet, 2 | 30 → 27 | 71 → 65 |
| Allied–Soviet, 8 | 16 → 12 | 47 → 62 |
| Allied–Yuri, 0 | 13 → 8 | 32 → 58 |
| Allied–Yuri, 2 | 12 → 16 | 41 → 21 |
| Allied–Yuri, 8 | 16 → 13 | 46 → 103 |
| Allied–Allied, 0 / 2 / 8 (house 0) | 13 / 10 / 15 → 7 / 18 / 12 | 32 / 30 / 135 → 34 / 25 / 58 |

The median ticks out of an Allied cycle fall to 285 to 400 in five of the six games of two sides (main 360 to 646; Allied–Soviet on map 2 rises from 646 to 785). On map 2 the home field is small, and the miners still load 16 to 27 cells out. On main every Allied and Soviet house kept one refinery all game. After, the first house has two in six of the nine games where it is Allied, and other houses reach three to five (Soviet in Allied–Soviet and house 1 of the Allied mirror, both on map 2: five).

**Tests** (synthetic data, through the real tick).
- `crates/deterministic/ai/tests/economy.rs`, new: `rebuilds_a_lost_refinery_before_power`, `keeps_the_yard_while_selling_for_a_lost_refinery` and `places_the_refinery_beside_the_field` (which fails without the placement).
- `expansion.rs`, new: `builds_a_refinery_by_the_far_field_once_the_home_field_runs_dry` (which fails without the field refinery; it also checks that none is built there while the home field is rich). `a_rich_house_with_a_far_field_builds_a_yard_vehicle` checks the case without a far field on the plain map, because on the far-field map the harvesters now explore that field themselves once the home field is mined down.
- `power.rs` passes `None` for the new argument.
- `skirmish.rs`: the naval game's seed moves from 6 to 2. Over seeds 1 to 32, `naval_seed_sweep` passes 31 on main and 18 after. Each failing seed ends with a land win between ticks 14 495 and 17 934, before the ships meet. The passing seeds run from 19 850 ticks to the 40 000-tick cap. On main, 12 of the 32 seeds reached the cap and the others ended at 21 345 to 38 425, except seed 16 at 13 770.
- **Goldens**: seven of nine were regenerated. Six differ from tick 1 960 to 2 000 on, when the first refinery is placed: duel, duel-diagonal, duel-naval, duel-m6c, duel-m7b and skirmish. The duel now ends at about tick 23 900 (main 19 870), duel-diagonal at 18 210 (14 800) and duel-naval at 19 290 (24 270); the others keep their length. duel-m7a differs from tick 60: in the M7a fixture Beta starts with a harvester and no refinery, so it now wants its refinery first. combat and scripted are unchanged.

**Seed sweeps M6c, M7a and M7b, 1 to 16** (min / median, main → after; f0 and f1 alike).
- M6c: smallest minimum 3 → 4. Radiation exits 3 / 28 → 9 / 25, crates picked up 6 / 16 → 11 / 20, support beams 64 / 103 → 51 / 109, chrono erasure orders 27 / 31 → 22 / 32.
- M7a: smallest minimum 0 → 0. Seeds with a zero counter were 6 and 13 (infantry absorbed) on main, and are 7 and 13 (hostile intent seen by a sensor) and 9 (infantry absorbed) after. Units made berserk 30 / 50 → 13 / 41, takeovers 14 / 25 → 5 / 23, slave-miner buildings 13 / 13 → 12 / 13. The gate seed 17 has every counter above zero.
- M7b: smallest drain minimum 3 → 3. Credits drained 7 280 / 7 320 → 6 960 / 7 440, infantry absorbed 3 / 7 → 7 / 10, objects taken over 10 / 17 → 12 / 17.

**Left open.**
- `ore_goal` does not ask whether a harvester can reach the field (its reach region). A refinery can be pulled toward ore across water or a cliff. The probe showed no such case.
- On map 2 the miners still drive far once the small home field is mined out and no other field is explored near home.
- A broke house whose lost refinery stands at 51 of 54, with only the yard left, stays there (Allied–Yuri, map 0, from tick 31 000; a person could do no more).
- The naval synthetic games end before the ships meet in 14 of 32 seeds.
- No requests.

## Own AI (M8 step B9: allies, the far-field refinery, the island naval game)

B9 follows up B8. It changes the own AI (`glasnost_ai::scripted`, `brain`, `fixture`), the harvest lane (`crates/app/tests/harvest_lane.rs`) and the lane's economy probe (`crates/tools/src/skirmish_lane.rs`). Lane outputs are git-ignored under `local-lane/m8/b9/` and hold counts and ticks only. The quick lane is B8's: seed 7, maps 8, 2 and 0, at most 120 000 ticks, `GLASNOST_AI_QUICK=1`, f0 and f1, 30 games. *Before* is the branch with the alliance fix and the two probe commits. The alliance fix changes no game without allies, so before plays exactly as B8's after: every count below matches B8's table. *After* is the branch. Both binaries were built on main b1ab906, before intake 42, so they share one base. Intake 42 changes ore spreading next to trees on real maps, so a lane on today's main can differ from both. The goldens, sweeps and tests were first run on the branch before its rebase onto main 5c1d621; the arena and the final checks ran after it, and there the goldens and the naval sweep pass again unchanged.

### The alliance bug: the lane was at fault

Intake 41's harvest lane (`far_slave_miners_of_a_person_and_a_computer`, map 8) allied every house by hand, and the computer's units still hit the person's slave miner (hit at +1 557, gone by +2 882). The lane's `peace` pushed each house onto the others' `House::allies` lists unsorted. `House::allied_with`, the sim's alliance test, does a binary search on a sorted list, so for house 0's list `[0, 3, 1]` it found no ally but itself. The sim's acquisition (S-0309 claim 18) therefore saw an enemy. No real setup writes alliances like that:
- the loader sorts and dedups the setup's alliances;
- lobby teams (`start_alliances`, S-0412 claim 8) and the in-game alliance order (`victory::alliance`, S-0414 claims 12 and 13) insert in order, rebuild the threat records and update the shroud.

A two-player game cannot ally in play: S-0414 claim 12 refuses the order when exactly one other contender is left unallied.

- **The lane** now puts both slots on lobby team 1 (`SetupHit::Team`, S-0412 claim 15) and asserts that the sim sees every player allied. It also asserts that the person's miner is never hit and that no object ever targets an ally's object. Rerun on map 8 (`local-lane/m8/b9/lane`): no hit, no ally targeted, and the game goes on.
- **The AI**. Two things were wrong here, though neither caused the lane's hit.
  - Its own alliance tests used `allies.contains`, which reads a list differently from the sim's test. They now use `House::allied_with`, as the sim does (`seen_enemies`, special, mutator, dominator, brain).
  - Its wave fallback (`Script::enemy_starts`) sent waves to every other start, an ally's base included. It now leaves out a start where a building of an allied player house stands on an explored cell within `ALLY_BASE` (10, nocite) cells. Allies share sight (S-0409 claim 2), so this is what a person sees.
  - Both are read at every think, so an alliance made during the game counts from the next think.
- **Test** `allies.rs` `allied_ai_houses_never_go_for_each_other`: two fixture AI houses on lobby team 1 over 14 000 ticks. No object targets or hits an ally's, units move, and no move order goes within 5 cells of the ally's start. Without the filter it fails: 76 331 and 72 253 unit-ticks were sent to the ally's start. A mid-game alliance test would need three houses (claim 12), and the fixture map has two starts.

### What the probe found (before)

The economy probe now prints, per house, the spells in which `Brain::wants_field_refinery` holds (sampled every think period) and the stalls within them. A stall is a sample where the want holds and the house has no item on a building line, so nothing is being built for it. An empty line ceases to exist (S-0404 claim 1), so the test is for the absence of a building-line item.

- **The long spells are not stalls.** Five of the 15 f0 games have a spell over 3 000 ticks (f1 the same): Allied–Soviet on maps 0 (16 875–20 595) and 2 (28 935–31 965), Allied–Yuri on maps 0 (15 465–23 985) and 2 (25 965–30 045), Soviet–Yuri on map 0 (39 150–43 605). In every one the house is broke (credits 0 to about 300) with five or six lines running. The building line holds the item the refinery waits for:
  - a power plant that `power::plant_first` put in place of the refinery order, because the refinery's drain would leave the house short (Allied–Yuri map 0: plant started at 15 466, refinery at 23 986; Soviet–Yuri map 0: 41 251 and 43 606; Allied–Soviet map 2: 28 921 and 32 296);
  - a power plant already there (Allied–Soviet map 0: refinery at 20 596);
  - or the Allied superweapon on the defence line taking the money (Allied–Yuri map 2).

  The refinery starts as soon as the line is free. Over the 30 games the probe found stalls only as lone samples, each on the think when a line had just emptied and the next order was going out. No two stall samples were ever consecutive (longest stall 0 ticks). B8's reading was that the want "stays true while the refinery cannot be built and pushes aside the tech climb". That was the length of these spells read as a stall. The climb could not have used the busy building line either.
- **One real stall was possible.** `field_refinery` asked `refinery_site` (the wider spiral, within reach of the field), while `Script::build` orders a type only when `placeable` finds it a cell in the build spiral. A refinery that fit by the field but not in the spiral was wanted with nothing ever ordered, and everything behind it on the plan waited. The lane did not hit this; the synthetic test below builds it.
- **Lost refineries waited on held money.** In 21 probe samples, over 13 of the 30 games, a house with no refinery had its refinery on the building line. Often the money it needed sat in held items. For example, in Soviet–Yuri on map 2 the Soviet house had 5 credits, a held defence building with about 840 paid, a held infantry item, and a refinery at 11 to 21 of 54 for more than 5 000 ticks. B8's `economy_first` holds the rest of the sidebar while it pays for the economy. A held item keeps what was paid into it (S-0404 claim 5), so with no income that money never reached the refinery. This is B8's "stuck at 51 of 54" case. B8's own example (Allied mirror, map 0) shows held army items owing 494 and 428 beside a refinery owing 76 with 30 credits in hand.

### What the brain does now (B9)

- **The field want holds only while it can be ordered** (`Script::field_refinery`, `// design:`). A refinery type must also be `placeable`, the same test `Script::build` uses. The rule has no memory, so the want returns as soon as a cell opens, a new field is explored or a harvester reaches one. The brief asked for a way to give up, "for example after a number of thinks with no placement, or when no legal cell exists by the field". The second is what this does. A count of thinks would only ever fire in the broke spells above, where a person keeps the refinery first too. It would also need saved AI state, so it was not added. The AI game format stays 45.
- **The refinery goes by a field a harvester reaches** (`Script::ore_goal`, `Script::harvester_reach`). The candidate resource cells are limited to the reach regions (S-0212 claim 4) of the house's harvesters, by each one's movement zone. Slave miners, which go by their own rules, and flying harvesters are left out. With no harvester on the map, the regions used are those of the cells within `HOME_RING` (6, nocite) of home, for the zone of each harvester type of the plan, buildable now or not. A house with no harvester has no war factory either, and its first harvester may come later or with a refinery (S-0404 claim 2). This is the pathing query a person's order gives, on cells the house has explored. With nothing known, no cell is left out.
- **A house paying for a lost refinery cancels a held item for its refund** (`Brain::refund_for_refinery`, in `economy_first`). The conditions:
  - its harvesters wait for a lost refinery (`refinery_lost`);
  - it pays for its economy first;
  - the money in hand is less than the refinery still owes (its whole price before it is on a line).

  It then cancels, one item per think, the item that refunds the most. That item is not of the economy, not done, not on the refinery's line, and its refund is above 0 (S-0404 claim 6: *P*′ − *B*, or the price at the start with `quirks.refund_at_current_price` off). A person clears the sidebar the same way to finish the refinery. A broke house with only the yard and nothing paid into any item still waits; no order a person has would pay for it. Like the other B8 rules, this leaves out slave-miner houses.
- The economy probe prints the field-refinery spells and stalls (`fieldspell house h: ...`).

### Results

Quick lane, f0, before → after. Winner and tick, then e20 of house 0 / house 1. Rows that differ:

| Sides, map | Before | After |
| --- | --- | --- |
| Allied–Allied, 0 | house 1 36 940; 76 062 / 58 106 | house 0 48 224; 83 092 / 57 782 |
| Allied–Allied, 2 | house 1 21 777; 78 958 / 88 276 | house 1 26 963; 94 421 / 93 526 |
| Allied–Soviet, 2 | Soviet 49 453 | Soviet 50 621 (e20 the same) |
| Allied–Yuri, 0 | Yuri 49 662 | Allied 82 337 (e20 the same) |
| Soviet–Soviet, 0 | house 1 59 394 | house 1 82 949 (e20 the same) |
| Soviet–Soviet, 2 | house 1 23 916; 74 349 / 86 452 | house 1 22 993; 76 206 / 88 311 |
| Soviet–Yuri, 2 | Yuri 33 678; 53 984 / 57 670 | Yuri 37 285; 66 644 / 57 256 |
| Soviet–Yuri, 8 | Yuri 42 605 | Yuri 42 576 (e20 the same) |

The other seven games are identical. Seven of the eight games that differ split on the refund rule: built without it (`quick/norefund`, the eight f0 games), they play exactly as before. The eighth, Allied–Soviet on map 2, ends 1 168 ticks later with the same winner and the same e20. It splits after tick 40 000, while the Allied house rebuilds a lost refinery with 13 078 credits in hand, so the change comes from the reach filter or the field-want guard (not separated). Allied–Yuri on map 0 splits at about tick 35 000. Before, the Allied refinery stood at 53 of 54 waiting for money, and Yuri won at 49 662. After, the refinery stands again by tick 35 000, and the Allied house recovers and wins at 82 337.

- **Field-refinery spells over 3 000 ticks**: 5 → 6 games in f0 and in f1. The five are the same spells to the tick. The sixth is Soviet–Soviet on map 0, house 1, 69 375–72 885. That game now runs to 82 949. The house is broke with a superweapon building of the climb on the building line (credits 5, building at 24 of 54 owing 1 102), the same kind of spell as above. Stall samples: none consecutive in either lane; lone samples 79 → 91, over the 41 → 46 house-runs with a spell.
- **Lost refineries waiting**: probe samples of a house with no refinery and its refinery on the line, 21 → 12 over the 30 games. The ones left are a Yuri-side house (no B8 or B9 rule acts for slave miners), a house with no harvester left (nothing waits for a refinery), and houses with the money in hand or nothing paid into any item.
- **e20 by side over the nine games of two sides**: Allied 353 452 → 353 452, Soviet 398 042 → 410 702 (+3.2 %), Yuri 312 319 → 311 905 (−0.13 %, all in Soviet–Yuri on map 2, −414).
- **Wins** (f0 = f1): Allied 5 → 6, Soviet 5 → 5, Yuri 5 → 4. Allied against Soviet 2 of 3 in both. Allied against Yuri 0 → 1 of 3. Soviet against Yuri 1 of 3 in both. Every game has a winner, every run exits 0, and there are no panics.
- **The Yuri bar is missed by the letter: one win and 414 credits of e20.** No B9 rule orders anything for the Yuri house. The win changed sides because the Allied house rebuilt its lost refinery with the refund of its held items, as a person does, and came back. The e20 loss is in one game, where the Soviet house did the same before tick 20 000 (Soviet e20 there 53 984 → 66 644). Under D14 a change that makes one side's AI play better is not held back to keep another side's count. The lead should judge this as B8's Yuri bar was judged.
- **Game length**: 628 961 → 705 484 ticks (f0), 646 904 → 730 974 (f1). A house that rebuilds its refinery lasts longer.
- **f1** ends on the same tick as f0 in 10 of 15 games after (11 before), with the same winner in all 15. Its e20 equals f0's in every game.

**The naval game on islands** (`fixture::map_islands`). This is `map_naval` with `map_strait`'s strait from the shore to the south edge, so each start stands on its own island. The sea and the strait join, and no ground unit walks between the bases. Air units and transports still cross. `navy_game` plays on it, and the seed is back to the plain game's seed 5 (`SEED`), not one picked for passing. Over seeds 1 to 32, `naval_seed_sweep` passes 32 on the island map, against 18 on `map_naval` after B8 (31 before B8). The ground war no longer ends the game before the ships meet.

**Tests** (synthetic data, through the real tick).
- `economy.rs`:
  - `places_the_refinery_beside_a_field_a_harvester_reaches`: a moated home field and an open field farther off, both in sight. Without the reach filter the refinery goes toward the moated one.
  - `cancels_a_held_item_to_pay_for_a_lost_refinery`: a held soldier with steps paid is cancelled while the lost refinery cannot be paid, and nothing is cancelled with the balance in hand. It fails without the rule.
- `expansion.rs`, ignored in debug: `wants_no_far_field_refinery_it_could_not_order`. Soldiers fill the build spiral, and a power plant beyond it lets a refinery fit by the far field. The want must be off. It fails without the `placeable` guard.
- `allies.rs`, ignored in debug: `allied_ai_houses_never_go_for_each_other` (above).
- `skirmish.rs`: `navy_game` and `naval_seed_sweep` play the island map, the game at seed 5.

**Goldens**: one of nine was regenerated. duel-diagonal differs from tick 14 860 on and ends at about tick 18 520 (B8 18 210). Built without the refund rule it is byte-identical to the old file, so that rule moves it. The other eight are unchanged.

**Seed sweeps M6c, M7a and M7b, 1 to 16, and the gate seed 17** (f0 and f1 alike; `local-lane/m8/b9/sweeps`): M6c, M7b and every seed-17 run are identical to before. In M7a only seed 5 differs, again from the refund rule (without it seed 5's counters are as before). No minimum moves; the smallest minimum stays 0, at the same seeds 7, 9 and 13. Units made berserk median 41 → 42, gattling at its top stage maximum 7 389 → 7 196, lifter build orders maximum 6 → 8.

**Arena**: byte-identical. The branch rebased on main 5c1d621 writes the same three CSVs as intake 42's after-run on that main (`local-lane/m8/b9/arena-after`). The arena attaches no brain of ours, and B9 changes no simulation code.

**Left open.**
- The long broke spells stay. The field refinery waits behind the power plant that `plant_first` puts first, or behind the climb's superweapon building, while the house has no money. Holding the army to pay for it, as the superweapon saving does, is a balance of fights against income that this step did not try.
- A broke house with no refinery, only its yard, and nothing paid into any item still waits. A person has no order that pays for it.
- `harvester_reach` takes each harvester's zone from its type and reads reach regions only. A field the house's harvester could reach by a bridge that a person knows to be destroyed reads the same as one it cannot reach.
- No requests.

## Own AI (M8 step B10: a broke house recovers)

B10 takes up B9's three open points. It changes the own AI (`glasnost_ai::scripted`, `brain`) and the lane's economy probe (`crates/tools/src/skirmish_lane.rs`). Lane outputs are git-ignored under `local-lane/m8/b10/` and hold counts and ticks only. The quick lane is seeds 7 to 10, maps 8, 2 and 0, at most 120 000 ticks, `GLASNOST_AI_QUICK=1`, with reference rules (*f0*) and with `GLASNOST_PLAY_FIXES=1` (*f1*). It plays Allied–Soviet, Allied–Yuri, Soviet–Yuri, Allied–Allied and Soviet–Soviet: 60 games per rule set. *Before* is main 0eb1690 with the two probe commits; *after* is the branch on the same main. Both were built on that tree. The branch was later rebased onto main 962941b, whose crates differ from 0eb1690 only in the arena oracle, so the lane, the goldens and the sweeps carry over; the gate and the final checks ran again there. The lane was first run on main a970624 and again on e375cf6 (intake 45). Intake 45 changed the lane's games (Soviet cross wins 4 → 7, Yuri 19 → 14). 0eb1690, the S-0501 claim 40 fix that lets the AI skirmish end again, changed none of them: e375cf6 and 0eb1690 give the same 120 games to the tick, with and without B10. *e20* is the credits a house earned up to tick 20 000 (`GLASNOST_AI_ECON=5000`).

**The economy probe** (`GLASNOST_AI_ECOPROBE=<ticks>`, `Brain::broke_probe`, `Game::broke_probes`). Every 60 ticks it samples each AI house. At the end it prints three things:
- the house's *broke spells*: runs of samples in which the house waits for an economy item (a lost refinery, a refinery by a new field, a harvester) with less money in hand than the item owes. For each spell over 3 000 ticks it gives what the house waited behind: a power plant on its lines, a superweapon building running unheld, both or neither. It also gives how many samples the house was saving for the superweapon, paying for its economy first or selling out, and its credits and income;
- its *yard-only spells*: samples with only a construction yard and nothing on any line;
- the map's bridge links whenever one changes (tick, links, enabled).

The probe reads and changes nothing: the 60 f0 games with the probe on match the plain lane to the tick.

### What the probe found (before)

Over the 60 f0 games there were 637 broke spells, 1 104 420 ticks in all, and 124 of them lasted over 3 000 ticks:

| Waits for | Behind | Spells | Ticks | Saving samples | Paying samples |
| --- | --- | --- | --- | --- | --- |
| refinery by a new field | superweapon building | 75 | 424 020 | 2 463 | 26 |
| refinery by a new field | neither | 39 | 217 380 | 15 | 190 |
| refinery by a new field | both | 2 | 27 600 | 112 | 0 |
| harvester | neither | 6 | 21 840 | 0 | 370 |
| lost refinery | neither | 2 | 11 460 | 0 | 193 |

- **The power plant was never the main blocker.** In no long spell was a plant on the lines in most samples. The spells come from the refinery by a new field (116 of 124, 669 000 ticks). The house had a refinery, so the B8 rules did not count it as paying for its economy. Its money went first to a superweapon building on the defence line (the earlier line is paid first, S-0404 claim 1; the house was saving for it in 2 463 samples) or to its army lines. The house earned meanwhile, typically 2 000 to 5 000 credits per 3 000 ticks. A few early Soviet spells on map 8 earned 0 to 500.
- **Yard-only spells: none after the start** in any of the 60 games, before or after.
- **Bridges**: link changes in 3 of 60 games (map 0), each a destroyed bridge (links 4 → 3, and 3 → 2 in one game).

### What the brain does now (B10)

- **A broke house pays for its far-field refinery first** (`Brain::field_refinery_short`, `// design:`). The rule applies when:
  - a refinery item of a house that still has a refinery runs unheld on its lines, owing more than the money in hand;
  - the house is safe (`Brain::safe`), not selling out, and not a slave-miner house.

  The house then pays for its economy first (`Brain::paying`). B8's `economy_first` holds every other running item, the superweapon building included (S-0404 claim 5), and resumes them once the refinery is paid or stands. This is what a person does: pause the sidebar to get the income back. A house that is not safe keeps training, as the superweapon saving does.
- **Nor does that refinery wait behind a power plant while the house cannot pay for both** (`plant_first`'s `before_power`, `Script::field_refinery_type`). With less than the plant's and the refinery's prices together in hand, the plant would take what the refinery needs for its income. B8 made the same exception for a lost refinery.
- **A house selling out with no refinery and no harvester left rebuilds the refinery that brings a harvester** (`Script::refinery_lost`). Before, `refinery_lost` needed a harvester waiting. A house whose refinery and harvesters were all gone ordered nothing while selling out, whatever its credits: selling out latches until the house earns again (`Script::sell_when_broke`), and with no harvester it never earns. It also sold its yard first, the costliest building. Now, while it sells out, a refinery type that comes with a free harvester (S-0404 claim 2, S-0402 claim 18) counts as lost. The house keeps the yard (B8's rule), sells the rest and rebuilds that refinery. A house that is not selling out, the opening included, keeps the plan's order. The ever-built count (S-0407 claim 10) was tried first and is not used: with `quirks.placed_buildings_not_ever_built` on, as in the fixture, a placed building raises no count.
- **Item 2 as briefed did not occur.** No yard-only house with nothing on its lines appeared after the start in any lane game. The fix above is the case the synthetic test found behind it: the selling-out latch with no harvester. A yard-only house with no credits and nothing paid into any item still waits. No order a person has pays for it, except selling the yard, which ends its building.
- **Item 3 needed no change.** A destroyed bridge already cuts a field off. Naming a bridge cell destroyed (`bridges::high::name_cell`, S-0212 claim 3) disables the links within 3 cells and marks the regions dirty. The rebuild joins two regions only through an enabled link (S-0212 claims 3 and 5). `Script::harvester_reach` reads those regions, which is the pathing query a person's move order learns. The test below pins it. Left open: a harvester standing on a bridge deck reads no region of its own.
- No new saved state: AI game format 47 and save format 85 stay. No order-issuing path for moves or attacks was touched.

**What was tried and dropped.** An income test beside the credits (pay first only when the credits and the last window's income together fall short) never fired where it was meant to: the spells behind a superweapon building went 75 → 76 and broke ticks 1 104 420 → 1 160 100. The house did earn, and the earlier line took the income first. It was committed and then reverted on the branch.

### Results

Quick lane, before → after. Every game has a winner except Soviet–Yuri on map 0, seed 10 (f0 and f1, before and after alike), which runs to 120 000 ticks. That game first appears on main with intake 45 and is not B10's. Every run exits 0 and there are no panics. f0 and f1 have the same e20 in every game.

| | Before | After |
| --- | --- | --- |
| e20, Allied (cross games) | 1 583 745 | 1 605 827 (+1.4 %) |
| e20, Soviet | 1 562 250 | 1 527 357 (−2.2 %) |
| e20, Yuri | 1 224 756 | 1 229 766 (+0.4 %) |
| Wins f0, Allied / Soviet / Yuri (cross games) | 14 / 7 / 14 | 16 / 6 / 13 |
| Wins f1 | 14 / 7 / 14 | 16 / 7 / 12 |
| Allied–Soviet (f0) | Allied 9, Soviet 3 | Allied 10, Soviet 2 |
| Allied–Yuri (f0) | Allied 5, Yuri 7 | Allied 6, Yuri 6 |
| Soviet–Yuri (f0) | Soviet 4, Yuri 7 | Soviet 4, Yuri 7 |
| Allied–Allied (f0), house 0 / 1 | 8 / 4 | 7 / 5 |
| Soviet–Soviet (f0), house 0 / 1 | 4 / 8 | 5 / 7 |
| Game length, f0 / f1 | 2 604 732 / 2 597 267 | 2 635 832 / 2 658 156 |

The probe, f0, before → after:
- broke ticks 1 104 420 → 907 020 (−17.9 %);
- spells over 3 000 ticks 124 → 92;
- spells behind a superweapon building 75 (424 020 ticks) → 49 (251 640);
- behind neither 39 (217 380) → 37 (169 440);
- behind both 2 → 0;
- harvester spells 6 → 4;
- lost-refinery spells 2 → 2 (11 460 → 14 760 ticks).

Economy holds over the 120 house-games went 733 → 3 229, at most 122 in one house-game against B2's 442 flicker. The largest count, 276, is the same game before and after.

**Attribution** (f0, each rule off in turn on main e375cf6, which plays as 0eb1690):

| Build | e20 Allied / Soviet / Yuri | Wins A / S / Y | Games differing from after |
| --- | --- | --- | --- |
| before | 1 583 745 / 1 562 250 / 1 224 756 | 14 / 7 / 14 | |
| after | 1 605 827 / 1 527 357 / 1 229 766 | 16 / 6 / 13 | |
| without the pay-first rule | 1 592 788 / 1 551 258 / 1 233 388 | 16 / 5 / 14 | most |
| without the plant exemption | 1 607 210 / 1 533 804 / 1 239 483 | 15 / 8 / 12 | most |
| without the selling refinery rule | 1 605 961 / 1 525 149 / 1 229 766 | 16 / 6 / 13 | 2 |
| with the income test | 1 577 620 / 1 547 091 / 1 233 388 | 16 / 5 / 14 | most |

A small change moves most games here: once one order differs, the game after it differs. The builds above are resamples of almost the same AI. Across them Soviet cross wins range from 5 to 8 and Soviet e20 lies 0.7 % to 2.4 % under before. Each build is below before, the one without the pay-first rule as well, so the Soviet e20 dip is not that rule's alone. The rule's own share against the build without it is about −1.5 % Soviet e20 and +1 Soviet win. **The Soviet bar is a judgment call for the lead,** like B9's Yuri bar: −1 Soviet win in f0 (7 → 7 in f1) and −2.2 % Soviet e20, inside the spread of the resamples, against a broke time cut by 18 % for every side. Under D14 a change that makes the AI recover as a person does is not held back to keep one count.

**Tests** (synthetic data, through the real tick):
- `economy.rs`:
  - `pays_first_for_a_refinery_by_a_new_field`: a brain that has watched two income windows holds its army item while the far-field refinery owes more than the money in hand, earning or not. It holds nothing with the balance in hand or with an enemy near the base. It fails without the rule.
  - `a_house_with_no_harvester_rebuilds_the_refinery_that_brings_one`: a rules variant whose refinery brings a free harvester, with sellable buildings and a refinery cheaper than the refunds. The house's refinery, harvesters and army go, and it is broke. It must sell out, keep the yard and stand with a refinery and a harvester within 8 000 ticks. Without the rule it sells the yard first.
  - `a_destroyed_bridge_cuts_a_field_off`: a moated home field joined to open ground by a bridge link. With the link enabled the refinery goes toward the moated field; with the bridge destroyed before the refinery is placed, it goes toward the open one. It passes on main, which is item 3's finding.
- `expansion.rs`, ignored in debug: `a_broke_house_orders_the_far_field_refinery_before_power`. With power to spare but less than the refinery drains, and one credit short of the plant and the refinery together, the house orders the refinery and no plant. With the money for both it orders the plant first. It fails without the exemption.

**Goldens**: one of nine moved. The straight duel differs from tick 21 220 on: at tick 21 225 house 0 has 42 credits and orders the refinery by a new field, short of power after it, where `plant_first` used to put a power plant in its place. With the plant exemption off, all goldens are byte-identical to main's. The pay-first rule and the selling refinery rule move none.

**Seed sweeps M6c, M7a and M7b, 1 to 16, and the gate seed 17** (f0 and f1; `local-lane/m8/b10/sweeps`): all twelve outputs are identical to before. The smallest minimum stays 0 in M6c and M7a; at seed 17 it stays 4 (M6c) and 2 (M7a).

**Arena**: byte-identical. The arena attaches no brain of ours, and B10 changes no simulation code. On main 0eb1690 the three CSVs of the branch equal those of before (`local-lane/m8/b10/arena-before3`, `arena-after4`). The branch was then rebased onto main 962941b. That main holds the same simulation and AI code as 0eb1690, and intake 47 changes only the oracle's scoring. There too the branch writes the same three CSVs as main (`arena-main962`, `arena-after5`). With intake 47's scoring, the scored pairings failing are M-0001 11 of 84, M-0002 2 of 24 and M-0003 4 of 24. The stock outcomes reproduced are M-0001 139 of 152, M-0002 45 of 47 and M-0003 38 of 42.

**Left open.**
- 49 spells behind a superweapon building remain. The house paid first in 626 of their 4 243 samples (26 of 7 142 before). In the other samples a condition of the rule did not hold, for example the refinery was wanted but not yet on a line, or the house was not safe. The probe does not tell these apart.
- A yard-only house with no credits and nothing paid still waits. It did not occur in the lane.
- A harvester on a bridge deck reads no region for `harvester_reach`.
- No requests.
