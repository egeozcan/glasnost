# M8 seam sweep 5a: rows marked partly or seam in intakes 51 to 72

Brief: `.claude/worktrees/M8-SEAMS-5A-BRIEF.md`. Method and row form follow [seams-3.md](seams-3.md)
and [seams-4.md](seams-4.md). Sweep 5b covers intakes 73 to 95.

**Scope.** Rows read:
- `intake51-*.md` to `intake72.md` (51, 52a, 52b, 53 to 72).
- The capture seam ledger [seam-S-0215-claim28.md](seam-S-0215-claim28.md).
- The step-ledger rows *Seam (trace: …)* and *Seam (trace 2: …)* in `docs/step-ledger-m8.md`.

A row is in scope when it says partly, not tested, untested, by construction, by inspection, built
with no test, not built, or seam.

**Counting rule.** Two kinds of item are counted:
- Table rows, matched case-insensitively on those words.
- Every bullet of a ledger's *Seams* section.

A bullet that repeats a table row gets that row's outcome and is counted again as a bullet. The
capture ledger's open items and the two trace rows' open items are counted one per item.

A row tested in part is counted once, under Tested. Its untested part is listed under Left but not
counted there. Rows are named by ledger line (`intake52b:130` is line 130 of the intake 52b ledger).

**Result.**

| | Table rows | Seam bullets | Capture and trace items | Total |
|---|---|---|---|---|
| Read | 159 | 53 | 10 | 222 |
| Closed earlier | 15 | 5 | 7 | 27 |
| Claim gone (current revision) | 2 | 1 | 0 | 3 |
| Not a seam row (tested at its intake, or a note; the word is only in its text) | 8 | 2 | 0 | 10 |
| Blocked by an open request | 1 | 2 | 0 | 3 |
| Tested here | 12 | 3 | 1 | 16 |
| Built here | 0 | 0 | 0 | 0 |
| Left | 121 | 40 | 2 | 163 |

- 12 test functions in 9 files.
- No test found the code differing from an approved claim, so no simulation code changed.
- No hashed state was added, so there is no format bump (save 98, AI game 61, wire 12).
- Goldens and lanes are not affected; lanes were not run, since the simulation is unchanged.
- No spec request was filed; Q-2556 to Q-2565 are unused.

## Built

Nothing was built. Most in-scope "not built" rows name a feature our engine lacks: tunnels,
underground travel, patrol processing, aircraft sinking, trigger action 32, a command bar.

One row is specified and not built: S-0215 r104 claim 16's damage-triggered fall-back for a
no-threat behaviour (intake57:185). It was left out because it changes play and so needs its own
intake with lanes (see Left).

## Tested

| # | Row | Spec | Claim or part | Test and result |
|---|---|---|---|---|
| 1 | intake68:82 | S-0309 | claim 29, Answers: the rank-5 keep for an infantry holder | `claims_s0309_r70::p2_the_rank_5_keep_for_an_infantry_holder`. The rearm countdown keeps the target to the first check tick after it ends; the control with no countdown drops at the next check tick. Code agrees. |
| 2 | intake68:87 | S-0309 | claim 29, a vehicle launching spawned aircraft (*busy*) keeps its target | `claims_s0309_r70::a_launching_vehicle_keeps_an_illegal_target_through_check_ticks`. Three check ticks are kept, and the control without the spawner drops. The launching count is forced through the spawn controller, as the r80 test does, and is not reached by real spawning. Code agrees. |
| 3 | intake52b:132 | S-0309 | claim 27, the holder's rearm countdown runs on after the drop | `claims_s0309_r51::claim_27_the_holders_rearm_countdown_runs_on`. A vehicle holder and a building holder are tested for the kill, inside and remove cases. Each has no target, `rearm_until` is unchanged and there is no shot over 5 ticks. Code agrees. |
| 4 | intake52b:130 | S-0309 | claim 27, a crashing object is dropped at its killing hit | `claims_s0502_crash::claim_27_a_crashing_jumpjet_is_dropped_at_its_killing_hit`. The holders are a vehicle and a building of the killer's house and a vehicle of another house. While the jumpjet crashes in play, every holder has no target and makes no shot. Code agrees. Not tested: a sinking vessel, and a building awaiting removal. |
| 5 | intake55a:122, bullet 169 | S-0312 | claim 13, a *moving blocker* refuses the crew | `claims_s0312::claim_13_crew_refused_by_a_moving_blocker`. With a moving friendly infantry unit, the cell gives `MovingBlocker` and no crew is placed. With all three still, it gives `StationaryFriendly`; an empty cell gives `Clear` and one crew. Code agrees. |
| 6 | intake55a:127, bullet 169 | S-0312 | claim 18, no entry test against another house's infantry | `claims_s0312_survivors::claim_18_no_entry_test_against_another_houses_infantry` (seeds 1 to 16). The probe there is not clear, yet the survivors are placed as the draws give. Code agrees. |
| 7 | intake55a:131, bullet 171 | S-0312 | claim 19 (g), a high-bridge cell reads the deck record | `claims_s0312_survivors::claim_19g_a_high_bridge_cell_reads_the_deck_record`. The deck record decides under a deck, the ground record elsewhere, and water under a deck is clear. The draws match the replay. Code agrees. |
| 8 | intake52b:140, bullet 164 | S-0405 | claim 13, the hover's building exception reads the cell it stands in | `claims_s0405_r47_hover::claim_13_the_hovers_building_exception_reads_the_cell_it_stands_in`. The vehicle stays enabled in its building's cell, though it heads for a cell without one, and is disabled once it stands outside. Code agrees. The refused choice is left (below). |
| 9 | intake63:103 | S-0213 | claim 16c, a second loop closed in the same tick changes too | `claims_s0213_r84::claim_16c_a_second_loop_closed_in_the_same_tick_changes_too`. Two loop-close orders are given in one tick, and both routes change. Code agrees. |
| 10 | intake63:115 | S-0213 | claim 16d, an aircraft or building in the route-holder limit | `claims_s0213_r84::claim_16d_a_building_and_an_aircraft_count_against_the_holder_limit`. At the limit, the click is refused with `MSG:PlannerMaximum`. Below it, the first point add raises the count. Code agrees. |
| 11 | intake60:92, intake71:77 | S-0513 | claim 39 with S-0309 claim 29, a building holder of a warp member | `claims_s0513_r62::claim39_a_building_holder_drops_the_member_at_the_first_check_tick`. The building is tested in guard and in attack, with its processing held back. It keeps the target to the first check tick after the launch and drops it there, with no draw and no shot. Code agrees. Aircraft holders are left. |
| 12 | capture: units inside | S-0215, S-0203 | claim 2 (processing only in the actor's own turn) with S-0203 claim 3 (a unit inside a building or a closed transport takes no turns) | `claims_s0215_capture::claim_28_a_unit_inside_is_not_processed_until_it_is_back`. A unit in *capture* with nothing is taken inside by `objects::take_inside`. It is not processed while inside, and gets row 2 (*guard*) at its first processing back on the map. Code agrees. `take_inside` is the path that garrison entry (`structures/garrison.rs` `go_inside`, S-0508 claim 5) and transport boarding (`transport::board`) both use. It takes the unit out of the update order. Boarding an open-topped transport puts it back at the end (`w.update.enter`, S-0203 claim 3), so such a passenger takes turns; that case is not tested here. The capture seam's question is answered by the spec text, so no request is needed. |

## Closed earlier

| Row | Closed by |
|---|---|
| intake52a:143 (S-0216 claim 7), intake52b:149 (M-0006) | Intake 71: S-0216 r37 claim 8 is the set-up, and M-0006 is its acceptance check. |
| intake52b bullet 162 (warp departure draw, Q-2156) | Intake 60, S-0513 r43 (the bullet says so). |
| intake54:84, bullet 116 (two points at one place, Q-2166) | Intake 56, S-0213 r74. |
| intake54:88, bullet 114 (S-0214 claim 6 (e), a falling vehicle, V4) | Tested at intake 85. |
| intake55a bullet 167 (Q-2181) | S-0301 r63, intake 63. |
| intake56:69, :73, :77 (Q-2191, Q-2192, Q-2193) | S-0213 r84, intake 63. |
| intake57:126 (more listed aircraft than pads) | Tested by S-0501's own tests (the row says so). |
| intake57 bullet 221 (S-0502 claim 24, part e's clearing) | Intake 74 (S-0215 r104 claim 37 part e: no effective clear and no jumpjet stop), tested by `claims_s0215_r104::part_e_clearing_is_no_effective_clear_and_no_stop_follows`. |
| intake60:79 (Q-2211 interim for a fate's member) | Intake 71: S-0513 r62 answers Q-2211, and question 3's interim stays under Q-2276 (sweep 5b's). |
| intake63:88 (Q-2227), :119 (Q-2226) | S-0213 r89, intake 66. |
| intake68:95 (Q-2261, place in the turn) | Intake 74, S-0309 r94. |
| intake68:101 (Q-2264, materialising holders) | Intakes 74, 75 and 79. |
| intake68:102 (warp members) | Tested since intake 71 (the row says so). |
| intake71:78 (Q-2263, neutral and special holders) | Intake 74, S-0309 r94 (`a_neutral_holder_makes_the_check_and_no_processing_drop`, `p9a_a_special_holder_…`). |
| capture row 2b (Q-2251), part 4 (Q-2252) | Intake 76, S-0501 r93 claims 49 and 50. |
| capture: the golden run left unverified | Superseded. Later merges regenerated and checked the goldens with the capture code in, for example the trace row (9242973): `duel`, `duel-diagonal`, `duel-naval`, `skirmish`, `m6c`, `m7a` and `m7b`. |
| trace 1: the overfly's approach heading | Trace 2 (the run lock suppresses body guidance, and our code matches). Q-2291 was answered at intake 76 (claim 51). |
| trace 2: `stale_guard_target`, the crate on-its-way test | *Seam (AI: stale guard target, crate on-its-way)*, 09cbd2c. |
| trace 2: the fly-back at the edge (Q-2291) | Intake 76, S-0501 r93 claim 51. |

## Claim gone

| Row | Current revision |
|---|---|
| intake57:130 (T12 run 8 d), :132, bullet 219 (S-0411 claim 3, an icon darkened while S-0407 claim 16's test refuses its type, *D* ≥ *T*) | S-0411 r49 claim 3 lists the causes of the darkening: a building line with an item, no switched-on factory, and the build limit (S-0407 claim 9's answer). It adds that dock occupancy plays no part. S-0407 claim 16 now says its pad test is distinct from claim 9's answer, and that a refusal at issue is S-0411 claim 4's row that gives no order. The darkening that intake 57 waited for is no longer claimed. Our icon darkens on claim 9's answer only, as the claim now reads. |

## Not seam rows

These rows are marked tested at their intake, or are notes; one of the words appears only in
their text.

| Rows | What the word names |
|---|---|
| intake52a:108 | Deployed and bunkered vehicles share the tested path. |
| intake53:53, :55, :57 | Claim 12's direct rocking is a seam; rocking is left (below). |
| intake56:75 | — |
| intake57:180 | — |
| intake63:94 | — |
| intake72:48 | — |
| intake55a bullet 172 | A note: nothing depends on intake 55b's or 56's specs. |
| intake69 bullet 91 | A lane note: the before shots predate the scroll-button shots. |

## Blocked

| Row | Request |
|---|---|
| intake51:75, bullet 93 (part) | Q-2111 question 3, open. |
| intake52b bullet 158 (DISK–FV jumpjet targets) | Q-2112, open. |

## Left (no test written)

**Not reached** means the row could be tested and no approved claim stands in the way, but this
sweep wrote no test for it. Each such row is still open for a later sweep.

| Rows | Spec | Why left |
|---|---|---|
| intake53:61, :62, :63, :64, :68, :69, :71, :75, :76; bullets 80, 82, 84, 87, 88; intake55a:142, :144, bullet 168; intake55b:144, :152, bullets 188, 189; intake65:69; intake67:59 | S-0113, S-0303, S-0612, S-0614, S-0306 | Rocking is the client's, not game state (S-0113 claim 19 seam since intake 53). There are no fire points on the angles and no rolling-over destruction. Direct rocking is a selection only. |
| intake53 bullet 91 | S-0113 claim 17 | The dominator ring's formula is open in the spec. |
| intake53:35, :37, :38, :46, bullet 92 | S-0108 | The view reads the simulation's record (S-0512's, intake 54). Slot creation draws are the client's. The reset row is `unmeasured`. |
| intake51:71, :73 | S-0306 | No outcome can tell. The walk reads only cells, and a 1-lepton raise need not change the rounded Ω distance. |
| intake51:84, bullet 93 (part) | S-0313 | An open question in the spec; the interim design stands. |
| intake51 bullets 89, 90, 92 | S-0306, S-0506 | Design choice (`rocket::place`). A fire point under a deck is tested through `start_point` only (not reached through a firer). P43 is a measurement, not code. |
| intake52a:100, :106 | S-0215 claim 39 | No observable: a killed vehicle has nothing to respond, and no house has teams. |
| intake52a:113, :115 | S-0215, S-0402 | The harvester's return is not specified. The note for AI designers is not a rule. |
| intake52b:126 | S-0309 r53 claim 22 | Not reached (S-0207 claim 15's give-up in front of a stationary friend). |
| intake52b:127, bullet 157 | S-0309 claim 22 | The spec does not examine the hovering and ship locomotors. |
| intake52b:128 | S-0309 claim 27 | Each listed origin goes through `take_inside` or `remove`, whose drops are tested (`claims_s0309_r51`). The single origins (transport, bunker, fall, trigger erasure, map edge, engineer) are not reached. |
| intake52b:130 (part) | S-0309 claim 27 | Not reached: a sinking vessel, and a building awaiting removal. The crash part is tested (#4). |
| intake52b:138, bullet 161 | S-0309 claim 28 | No underground locomotor. |
| intake52b:140 (part), bullet 164 (part) | S-0405 claim 13 | Not reached: a refused heading-point choice. The building exception is tested (#8). |
| intake52b bullets 155, 160 | S-0309, S-0210 | Draw order between two specs, open in their Open questions. |
| intake52b bullet 163 | S-0306 claim 3 | No rule of ours reaches an aim without a target. |
| intake54:81, bullet 112; intake54:89 | S-0213 claim 16c, S-0214 claim 6 (e) | Not reached: `unmeasured` with no scenario; the infiltrator exception's path is unchanged. |
| intake54:91 | S-0214 claim 32 | Client gesture. `Play::type_drag` is tested, and the session's drag branch is not driven. |
| intake54 bullet 115; intake56:85, :86, bullets 115, 119 | S-0213 claim 16g, S-0701 | Our art, our sound (`Effect::Route` unvoiced). |
| intake55a:108 | S-0301 claim 10 | Not reached: a SAM building with a finite magazine. |
| intake55a:121 | S-0312 claim 13 | No observable: the driving locomotor keeps a vehicle at the surface height. |
| intake55a:132, :134 | S-0312, S-0408 | By construction, with one class for every type. The sale tests pass. |
| intake55b:103, :105, :107, :117, :120; bullets 182, 183, 184 | S-0314 | Not stated, or open in the spec. Ours as the row says. |
| intake55b:114, :119, :121 | S-0314 | `measurable`, not measured, or by construction. A naval yard's destruction is not reached. |
| intake55b:169, :170, bullet 190 | S-0405, M-0014 | Observations. Pulses do not reach the simulation. |
| intake56:70 | S-0213 claim 16b | Not reached: a point held twice, beyond part w (8). |
| intake56:72, bullet 113 | S-0213, S-0214 GC3 | Patrol processing (S-0215) is not built. |
| intake56:81, bullet 116 | S-0213 claim 16h | Not reached: a tool press through the session (client). |
| intake56:84, bullet 114; intake63:120 | S-0213 claim 16h | No command bar; the spec leaves the button open. |
| intake56 bullet 117 | S-0213 claim 16h | `unmeasured`, built as given. |
| intake57:120, :125, :129, :143, :183, :186, :196, :204 | S-0407, S-0215, S-0607, S-0408 | By construction, a description that binds nothing, or not our rule. Each row gives its path. |
| intake57:159, :160, :162, bullet 225 | S-0215 claim 37 (a1) | No zone-less unit, no tunnel travel state, no loaner or team member. |
| intake57:161 | S-0215 claim 37 (a1) | Not reached: the scatter reference point and heading point of a1. |
| intake57:179, bullet 227 | S-0607 claim 9 | r33's open question. |
| intake57:185, bullet 223 | S-0215 claim 16, S-0607 claim 9 | S-0215 r104 claim 16 does specify the fall-back: a mobile actor that is not a team member, with the no-threat property and not the zombie property, falls back to rest at once when it survives damage from a known attacker (high; medium for which damage triggers it). It is still not built: the code has only the search exclusion (`combat::acquire`). Building it changes play, so it needs lanes and goldens. Its place among the other damage responses (claims 18 and 39) was not checked here. This is a build item for an S-0215 intake, not this test sweep. |
| intake57 bullet 229 | S-0214 | Viewer prune only, not the game's selection. |
| intake58:12, :15, :16, :17, :20 | S-0102 | Client view wiring. The rule is tested. The simulation keeps no tunnel or melee-link answer. |
| intake59:35 | S-0311, S-0011 | Trigger action 32 is not built (S-0011 is not in scope). |
| intake60:71 | S-0513 | Not reached: an object killed earlier in the landing cell's listing. |
| intake60:92 (part), intake71:77 (part) | S-0513 claim 39 | Not reached: aircraft holders. Vehicle, infantry and building holders are tested. |
| intake61:104, :107, :144, :145 | S-0306 | `unmeasured` repeats of tested paths (a search, a landed aircraft, a led void shot, a repeated scatter). Not reached. |
| intake61:126, :148 | S-0306, S-0405 | By construction; nothing to build. |
| intake61:127, bullet 224 | S-0306 Terms | A cloaking firer is not built (low; S-0302's cloak case is not built either). |
| intake61:132, bullet 225 | S-0306 T1 | Laser-fence section states are not modelled. |
| intake61:150, bullets 226, 227 | S-0306 P44 a | The whole scenario is the arena's. ROBO–HTK is an arena row (Q-2216, answered). |
| intake62:33, :36, :37 | S-0108 claim 26 | The client's slot turns: low, `unmeasured` or the intake 53 seam. |
| intake63:83, :99, :104, :109, :111 | S-0213 | Not reached: end-to-end clicks (water shore, a join before the loop, a dock return, a building's *wait*). |
| intake63:92, :93 | S-0213 claim 16b | No building undeploy; a building's *move* and *deploy* as built. |
| intake63:141, :142 | S-0301, S-0616 | `unmeasured`, or left open by S-0616 claim 4. |
| intake66:59, :63, :64; intake70:64 | S-0213 r89 and r94 | `unmeasured`; by construction. |
| intake67:66, :80, :84, :87 | S-0501, S-0109 | By construction; no game-wide smoke entry imported; draws through `anims::spawn`. |
| intake67:88 | S-0501 claim 9 | Aircraft sinking is not built. |
| intake68:86, :90, :98, :100, :105, :106, :109, :111, :114, :115 | S-0309 claim 29 and 4 | By construction, or `unmeasured`. Each row names the shared check. |
| intake68:99 | S-0309 claim 29 | No tunnel, vehicle death frames or materialising building. |
| intake69 bullets 82, 88 | S-0411, S-0005 | Client file lookup notes, not claims. |
| intake70:50 | S-0301 | By construction. |
| intake71:76 | S-0513 claim 39 | By construction. |
| intake71:102 | S-0216 claim 8 | Aircraft groups: low and open in the spec. No record has one (intake 77 keeps it). |
| intake72:58 | S-0301 | The original's let-go end, replaced by our fix (lead decision). |
| intake72:63 | S-0616 P1 b | Not reached: the gas and fire systems' trigger marks. |
| capture row 5 | S-0215 claim 28 | The hunt rule is the claim's own "In our engine" divergence. |
| capture: the seed 10 engineer that stands still | S-0215 claim 28 row 3 | Row 3 gives "no change". What keeps it still is untraced, and no claim says otherwise. |
