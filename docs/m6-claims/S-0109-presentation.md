# S-0109 animation playback claim ledger (M6b, presentation)

Superseded for the simulation's side by M8 step A5: the simulation now plays every ordinary animation (claims 5 to 17, 22, 28 and 29), and its rows below that say the simulation has no playback clock are replaced by [docs/m8-claims/S-0109-playback.md](../m8-claims/S-0109-playback.md). The drawing's rows (claims 19 to 21, 24, 26, 27) stand.

Revision 28 (taken in at M6c step 1b, [step1b-m6c.md](../m6-claims/step1b-m6c.md)): the in-place deploy presentation is created in the vehicle's turn with no initial loop delay and a reverse request for an undeploy (`inplace::start_presentation`, `Effect::Anim` with `reverse`; the viewer's `mapview::live::create_anim` and `mapview::anims::Player::new` create it with no loop delay, repeat factor 1, the first update in the creation tick and its first image advance after the playback delay), its playback delay is the viewer's own and the vehicle's completion clock is S-0517 claim 7's base-delay count in the simulation. No code change in the viewer.

S-0109 played in the drawing: the animations the simulation reports (`Effect::Anim`) and the
move-target marker (S-0214 claim 30). The simulation keeps no playback state; it takes the
creation draws and carries the drawn creation delay to the drawing in `Effect::Anim::delay`.

Tests: `crates/app/src/mapview/anims.rs` (the stage clock, scenarios AN1 to AN11 on invented
types), `animation_events_play_their_art_and_end` in `crates/app/src/mapview/live_tests.rs`
(an animation event drawn with its art and removed at its end, a type without a sprite, no
marker in the game client, the move-feedback animation), `crates/app/src/mapview/slots.rs`
and the slot-animation tests of `crates/app/src/mapview/tests.rs` (the drawing code path the
played animations share). Local lane: `crates/app/tests/anim_lane.rs`.
Code: `crates/app/src/mapview/anims.rs`, `live.rs` (`create_anim`, `update_anims`,
`take_effects`, `move_marker`, `redraw`), `build.rs` (`Builder::live_anim`, `anim_items`,
`anim_files`), `slots.rs` (the imported fields, `translucency`), and
`crates/deterministic/sim/src/load.rs` (`anim_created_at` returns the drawn delay).

| claim | status | test names or reason |
|---|---|---|
| 1 (image frame a + k) | tested | `plain_playback_an1`, `end_past_the_sprite_an1_t3`; `animation_events_play_their_art_and_end` |
| 2 (stage delay from the rate) | tested | `plain_playback_an1`, `fast_rate_freezes_an2` |
| 3 (random rate) | tested | `random_rate_an3` (the simulation's drawn value; equal ends take no draw), `follow_up_redraws_its_delay`. Partly: a follow-up's draw comes from the drawing's cosmetic stream, not the scenario stream claim 28 requires (playback is not in the simulation) |
| 4 (game-speed normalisation) | tested | `game_speed_an4` |
| 5 (when the stage advances) | tested | `first_frame_and_lifetime_an1_t5_t6`, `first_update_after_creation`. Partly: every reported animation takes its first update in its creation tick, claim 29's case for creation before or during the turn pass; the update lost to a removal in the update order and creation after the pass are not distinguished, because playback is not in the simulation |
| 6 (passes and end limits) | tested | `plain_playback_an1`, `looped_an5`, `loop_quirks_off` |
| 7 (remaining passes, endless) | tested | `looped_an5`, `loop_quirks_off` (`anim_loop_count_wrap`) |
| 8 (reversed play) | tested | `reversed_an6` (with `anim_reverse_raw_loop_end` off) |
| 9 (ping-pong play) | tested | `ping_pong_an7` (with `anim_pingpong_lower_turn` off) |
| 10 (shadow types) | tested | `shadow_pass_end_an8`; the shadow frame drawing in the slot tests of `mapview::tests` |
| 11 (loop delays) | tested | `loop_delay_an9`. Partly: the between-pass draw comes from the cosmetic stream, not claim 28's scenario stream; no creator asks for a loop delay at creation |
| 12 (no random starting frame) | tested | every test starts at frame *a* |
| 13 (start event) | not implemented | the start event's effects are simulation state (S-0202 claim 15, taken at creation by `load::anim_created_at`); the drawing plays no event |
| 14 (middle event) | not implemented | simulation state; μ > 0 is not modelled in the simulation (docs/development.md, *Animation start events*) |
| 15 (follow-up type) | tested | `follow_up_an11` (with `anim_followup_start_doubled` and `anim_followup_keeps_step` off), `follow_up_redraws_its_delay`, `unknown_follow_up_ends` |
| 16 (end of an animation) | tested | `first_frame_and_lifetime_an1_t5_t6`, `animation_events_play_their_art_and_end` (removed at its end). Infantry-making ends are S-0611 claims 9 to 12 (M7b step 3.4): the simulation plays those animations on the same stage clock (`glasnost_sim::anim_clock`, `mutation::turn`), tested in `claims_s0611.rs` (`claim_10_new_unit`, `claim_11_placement_refused`). A burning walker's end is S-0615 claim 11 (M7b step 3.2: `glasnost_sim::walker`, `expiry_lifetime_and_damage` in `claims_s0615.rs`) |
| 17 (damage over time) | partly | Built for the burning walker at M7b step 3.2 (S-0615 claim 11, `walker::advance_damage`): F5 with the accumulator, `quirks.anim_damage_starts_at_one` both ways in `expiry_lifetime_and_damage` (`claims_s0615.rs`). Other animations have no playback clock in the simulation, so their damage over time is not delivered |
| 18 (timing reaches game state) | not implemented | follows from 13, 14, 15 and 17 in the simulation, which has no playback clock |
| 28 (playback draws are simulation effects) | not implemented | the simulation has no playback clock; the drawing's cosmetic stream takes these draws (seam `AnimEnv::draw`) |
| 29 (creators and first-update timing) | partly | `first_update_after_creation`: an animation reported in a tick's effects is updated first in that tick; the in-pass placement and lost updates need playback in the simulation |
| 19 (detail level) | tested | slot tests of `mapview::tests` (`Builder::live_anim` uses the same `shown_at`) |
| 20 (translucency) | partly | `translucency_codes`, `translucent_fade_by_progress` (F4 at *t* = 0). Translucency levels *t* > 0 apply only to building slot animations, which are not played: `anim_translucency_levels`, `anim_hide_at_level_15`, `anim_low_detail_shows_cloaked` are in NOT_IMPLEMENTED |
| 21 (erasure) | not implemented | no building is erased yet |
| 22 (attached animations) | not implemented | the simulation reports no owner for the animations it creates (damage fires at the cell centre, S-0302 claim 13; trailers and wakes at a position), so every played animation is unattached |
| 23 (held animations) | not implemented | only building slot animations are held, and they are not played |
| 24 (fog, deferred) | not applicable | fog is off in a skirmish (S-0409 claim 19) |
| 25 (sprite of the derived fields) | tested | `loop_quirks_off` (`anim_loop_end_from_sprite`); the importer's sprite-found moment (S-0007) |
| 26 (draw point, vertical shift) | tested | slot tests of `mapview::tests`, shared through `Builder::anim_items` |
| 27 (tiled copies) | tested | slot tests of `mapview::tests`, shared through `Builder::anim_items` |

Requests: Q-0770 and Q-0771 are answered by revision 24 (claims 28 and 29); both need playback in
the simulation, which is not built.

Also: S-0214 claim 30's move-target marker plays `[General] MoveFlash` unattached, created between
ticks (first update in the next tick, claim 5), with its draws from the cosmetic stream (S-0214
claim 14); `animation_events_play_their_art_and_end` checks it.
