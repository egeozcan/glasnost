# M8 seam sweep 4: rows marked partly or seam since sweep 3

Brief: `.claude/worktrees/M8-SEAMS-4-BRIEF.md`. Method and row form follow [seams-3.md](seams-3.md).
Rows read: `intake49-S-0607.md`, `a7-S-0312-survivors.md`,
`intake50-S-0213-S-0214-S-0512-S-0522.md`, `c7-pathing*.md`, and sweep 3's *Left* rows.

**Result.** One importer warning built, one flaky test fixed, one test written. No test found the
code differing from an approved claim. No simulation change: no hashed state, no format bump
(save 88, AI game 50), goldens and seed sweeps unaffected. No spec request filed (Q-2141 to Q-2145
unused).

## Built

| Item | What changed |
|---|---|
| S-0408 Edge cases, the importer warning for a side crew or technician type that is an engineer type (A7: "not built") | `sim_combat.rs` warns for `[General] AlliedCrew`, `SovietCrew`, `ThirdCrew` and `Technician` naming an infantry type with `Engineer=yes`. The approved text (Edge cases, *A crew choice that always gives an engineer*, and Terms) says that the importer warns; it gives no wording, so the message is ours. |

## Tested

| # | Spec | Claim or scenario part | State before | Test and result |
|---|---|---|---|---|
| 1 | S-0408 | Edge cases, the engineer-crew warning | not built | `engineer_crew_types_warn` (`sim_combat_tests.rs`): quiet for non-engineer crews, a warning naming the key for a side crew and for the technician. |
| 2 | S-0701 | claim 24 with S-0213 claim 16 (F8), route-mode refusal cue once per completed tick | partly | `route_mode_refusals_share_one_cue_per_tick` (`client_play_notes_tests.rs`): two refusals in one tick give one cue, a refusal after a completed tick gives a new one. Code agrees. |

## Flaky test

`client_net_tests::two_clients_meet_chat_and_play_in_lockstep` raced on timing, not in the
lockstep code. It recorded a state hash only for ticks it saw in its own loop, but the ticks start
running while the start `pump` waits for both peers' phase, and under load a peer runs several
ahead of the other there, so the early ticks (tick 1 first) were never recorded on that side and
the final comparison saw a missing hash. Fix: `pump_seen` records the hash after every update from
the first one on (`keep_hash`), and the main loop uses the same helper. The comparison still covers
every tick 1 to 300 on both peers. Checked with 8 busy loops running: 6 of 6 runs pass.

## Left (no test written)

| Spec | Claim or scenario part | Why left |
|---|---|---|
| S-0312 (sweep 3 *Left*, survivors F4, F5) | claim 18 | Closed by A7 (`a7-S-0312-survivors.md`); sweep 3's reason no longer holds. |
| S-0312 | F5 no-crew map flag, claim 18 free position, claim 19 (g) cell categories | Open Q-2131, Q-2132, Q-2133. |
| S-0312 | claim 19 (g) and claim 26, the smudge | Smudge types are not modelled. |
| S-0214, S-0215 | GC3 patrol on a route point; claim 13 patrol begun and the approach row | Patrol processing (S-0215) is not built; Q-2138. |
| S-0213 | claim 16g look, claim 22 group bindings, route-mode texts and sound | Our art; no string table; spec's Open question. |
| S-0410 | radar miniature arrow in route mode | The radar takes no command clicks yet. |
| S-0214, S-0512 | claim 32 drag pool, claim 25 announcement, Terms group numbers | Q-2140; maps and teams load no group numbers. |
| S-0607 | claim 10 seized unit at rest deploys; early-ending turns | As sweep 3: no order route reaches it; design rows. |
| S-0504, S-0313, S-0303 and others | Rocking rates, fire point | Q-2081 answered by S-0113: rates built in intake 53; the fire point and game state stay a seam ([intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md)). |
| c7 pathing | any row | The ledger has no partly or seam rows. |
