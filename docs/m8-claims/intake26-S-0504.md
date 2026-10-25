# S-0504 r26: claim ledger, M8 step A3 intake 26

This intake takes in S-0504 revision 26 (spec commit fa10c82, from revision 20 at fcfca56), which
answers Q-1871: a sinking vessel leans fore and aft as it goes down. It replaces the interim of
M8 A4 note 20 ([a4-play-notes.md](a4-play-notes.md)), our own design that listed a vessel by
depth with an id-hashed roll to one side and a tilted shadow. That interim is deleted
(`mapview::sinking`, `mapview::sinking_tests` rewritten, `voxel::vehicle_parts_posed`, every
`// design: Q-1871 interim`).

- **S-0504 revision 26.** New claims 15 (the pitch step while sinking) and 16 (how the lean is
  drawn, and what reads it), F4 (the step's arithmetic) and F5 (the rocking body transform for
  the driving and ship locomotors). New Q-1871 summary block, Terms *Pitch and roll*, claim 10's
  pointer to claim 15, five Constants rows, three State and ordering rows and the timing
  paragraph, the original-versus-fixed note (three oddities not copied, no switch), the Import
  mapping sentence (no imported key), scenario N16 with parts a to h, N5 wording, Changes for
  other specs (S-0210's drawn sense of the body tilt; a pointer to F5 from S-0106 claim 9 and
  S-0210), and Open questions (the return of a rocked body to level; Q-1871's open details:
  hovering and jumpjet composition, lift and teleport carriers, clipping).

**Formats.** Save format 78 to 79, AI game format 40 to 41: `NavalState` gains `pitch` and
`lean`. The goldens move for that state only: `duel-naval.txt` (16 hash lines); the others are
unchanged. The oracle arena CSVs and the M6c, M7a and M7b seed sweeps 1 to 16 are identical
before and after.

**Code.**
- `glasnost_sim::naval`: `Lean` (`BowDown`, `SternDown`); `NavalState::pitch` (`F32`, soft
  float, default 0) and `NavalState::lean` (`Option<Lean>`, set at the first step); `lean_of`
  (claim 15's 8-direction groups); `pitch_step` (F4: the step formed in double precision and
  narrowed to single under the rules' rounding mode, clamped at ±*L*); `pitch` (the reader).
  `sinking_turn` takes one step after each descent of a sinking vehicle, the direction from the
  body facing's value for that tick at the first step and kept.
- `glasnost_render::voxel`: `Rocking` (pitch, roll and F5's *W* and *H* from a section's grid
  size), `ROCKING_THRESHOLD` (0.005, S-0106 Terms), `vehicle_parts_rocking` (F5's *D*, with the
  turret and barrel on it); `vehicle_parts_posed` is gone.
- `glasnost_app::mapview`: `Attitude::Sinking { pitch, roll, drawn }` from `live::attitude`
  (the simulation's pitch bits; roll 0; `drawn` for a type locomotor of driving, ship, hovering
  or jumpjet); `Tilt::Sinking`; a sinking vehicle draws no shadow; a body rocking at or above the
  threshold is posed by F5 and kept in the `rocked` cache keyed by type and exact pitch and roll
  bits (the `listed` cache is gone); a settled one uses the ordinary cache.
- Lane: `sinking-list` in `crates/app/tests/m7b_lane.rs`, redone (frames in
  `local-lane/m8/a3-26/`).

**Design rules of our own**, each with a `// design:` comment:
- Every sinking vehicle takes the pitch steps, not only one drawn as a voxel model: the
  simulation does not know which bodies are voxels, and only drawing reads the pitch (claim 16),
  so a sprite body's unread steps change nothing seen (claim 15 rates "no pitch steps" for such
  a vehicle `unmeasured` for that reason).
- F5 is used for the hovering and jumpjet locomotors too. S-0504 leaves their order of roll and
  pitch and their lift and shift open (Open questions); our roll is always level, so the order is
  moot, and the lift and shift follow the driving and ship form.
- The locomotor that decides `drawn` is the type's own (S-0207 claim 14), also for a vehicle
  carried by a lift or a teleport (open in S-0504).
- No clipping below a fixed screen row (open in S-0504).

| claim | status | test names or reason |
|---|---|---|
| Front matter (revision 26, Q-1871 in requests) | no change | Metadata. |
| Q-1871 summary block | tested | Summarises claims 15 and 16; see their rows. |
| Terms, *Pitch and roll* (single precision; a positive pitch lowers the bow under *Ry*; the roll about *x*; the rates belong to S-0210, S-0303 claim 20, S-0614) | partly | The pitch is held as `F32` bits and drawn about the model's *y* axis with S-0105's convention (`f5_lift_and_shifts`: at +*L* the bow end goes down and keeps its edge). Rocking rates are not modelled (seam, below), so the roll stays 0. |
| Claim 10, pointer to claim 15 (the lean is a fore-and-aft pitch set at the first update; a continuing turn does not change it; no sideways list) | tested | `claim_15_a_continuing_turn_keeps_the_lean`. The interim's sideways list is gone. |
| Claim 15, a step per own update towards ±π/4, stopping at it and holding | tested | `f4_the_lean_reaches_the_full_lean_at_the_79th_step`, `claim_15_the_lean_grows_with_each_descent_and_takes_no_draw`. |
| Claim 15, *Direction* (8-directions 1 to 5 bow down, 0, 6, 7 stern down; kept; no draw) | tested | `claim_15_the_end_that_goes_down_by_eight_direction` (boundaries 4095/4096, 45055/45056, 61439/61440), `claim_15_the_lean_grows_with_each_descent_and_takes_no_draw` (facing 0 stern, 16384 bow; the random state equals claim 4's wake draws alone). |
| Claim 15, *Which facing* (the body facing's value for the tick of the first step, also mid-turn) | tested | `claim_15_a_continuing_turn_keeps_the_lean` (facing 40960 turning to 0). |
| Claim 15, *Course of the lean* and F4 (exact single values; 79 steps) | tested | `f4_the_lean_reaches_the_full_lean_at_the_79th_step` (RZ and RN 78th values, the 79th at *L*). |
| Claim 15, *Timing* (same update as the descent; complete at the 79th, held at the 80th, removed at the 81st) | tested | `claim_15_the_lean_grows_with_each_descent_and_takes_no_draw` (80 recorded pitches, [77] short of *L*, [78] and [79] at *L*, then out of play). Starts below or above height 0 follow from the same step per descent. |
| Claim 15, *Starting value* (from the pitch at the start; the opposite sense passes through level) | partly | `claim_15_a_pitch_in_the_opposite_sense_passes_through_level` (F4 recurrence). In play the start is always level: rocking is not modelled, so no pitch is left over (seam). |
| Claim 15, *The pitch belongs to the lean; the roll is left free* | partly | No forward rate is added and the pitch never returns to level (only `pitch_step` writes it). A rocking hit during descent cannot roll the hull: S-0303 claim 20's rocking is not built (seam; N16 parts b, c and h cannot be reproduced). |
| Claim 15, *Turning* (a continuing turn into the other group keeps the lean) | tested | `claim_15_a_continuing_turn_keeps_the_lean`. |
| Claim 15, *The original's drawing* (three oddities not copied, no switch) | tested | The fixed rule is built: clamp at *L* (`f4_…`), kept direction (`claim_15_a_continuing_turn_keeps_the_lean`); the held roll is moot with no rocking. No switch added. |
| Claim 15, *Causes and types* (the sinking state alone; underwater types too; no steps for a non-voxel body) | partly | Keyed on the sinking state in `sinking_turn`; `claim_15_a_second_start_keeps_the_lean` (claim 13 restart), lane (an underwater type set sinking leans the same). Design: every sinking vehicle steps, voxel or not (unobservable, above). |
| Claim 15, afloat | tested | `claim_15_a_vessel_afloat_is_level`. |
| Claim 15 with claims 4 and 5, save and load | tested | `sinking_save_load_round_trip` (the naval state, pitch included, survives; the hash matches). |
| Claim 16, *Drawn transform* (F5 for driving and ship; turret and barrel on *D*) | tested | `f5_lift_and_shifts`, `rocking_parts_ride_the_body` (render); `a_leaning_vessel_has_images_of_its_own_and_no_shadow` (app). Slope tilt *T* is not in our F5: a sinking vessel is on water, and the land rocking that would combine with a slope is not modelled (seam). |
| Claim 16, hovering and jumpjet drawn as rocking | partly | Drawn with F5 (design, above). |
| Claim 16, *Image cache* (S-0105 claim 26 and S-0106 claim 13: rocking drawn fresh) | tested | `a_leaning_vessel_has_images_of_its_own_and_no_shadow` (a leaning image depends on its angles alone, apart from the settled ones; below 0.005 the settled image). The `rocked` cache is a memory bound with the pixels of a fresh draw, as for a rocket's pitches. |
| Claim 16, *Which locomotors show it* (driving, ship, hovering, jumpjet; others step but are not drawn) | tested | `only_the_rocking_locomotors_draw_the_lean`, `a_body_that_draws_no_rocking_is_drawn_settled_without_a_shadow`. |
| Claim 16, *Shadow* (none from the first drawn frame after sinking begins) | tested | `a_leaning_vessel_has_images_of_its_own_and_no_shadow` (at pitch 0 too), `a_body_that_draws_no_rocking_is_drawn_settled_without_a_shadow`. |
| Claim 16, *What reads it* (drawing only; the vessel cannot fire) | tested | Only `live::attitude` reads `naval::pitch`; `the_view_reads_the_pitch_the_simulation_steps` (reading changes nothing). |
| Claim 16, *Passengers* (an open-topped transport's passengers fire during descent from the fire point on the leaned body, F5's lift and shifts as leptons) | seam | The firing check's sinking test is the firer's own (`combat::check`), so passengers are not stopped. S-0313 claim 5's fire point is built from the heading alone (`claim1_point`, Rz only), so their launch positions do not take the lean (seam; N16 part g). |
| Claim 16, the angles may be presentation state | no change | We hold them in simulation state (hashed and saved), which the claim also allows; they advance once per own update. |
| F4 | tested | As claim 15, *Course of the lean*. |
| F5 (lift, *s*ₓ, *s*ᵧ; *W*, *H* from the first section of the main body model; table sin and cos; RZ double arithmetic) | tested | `f5_lift_and_shifts` (grid 20 × 12 × 9: *W* 6.5, *H* 10; at ±*L* (7, ±3, 0); roll ±0.5 (3, 0, ∓1)). The size comes from the main body also while the alternative body is drawn. |
| Constants rows (step, limit, size factors, 79 steps, bow-down 8-directions) | tested | Cited at their use (`naval.rs`, `voxel.rs`); tests as above. |
| State and ordering rows (two own-update rows, the drawn-frame row) and the timing paragraph | tested | As claim 15, *Timing*, and claim 16, *Shadow*. |
| Original-versus-fixed note (no switch; the 5/3 split is the original's) | no change | No switch added; `every_spec_quirk_has_a_switch` and `every_rendering_spec_quirk_has_a_switch` green. |
| Import mapping (no imported key) | no change | The step, limit and groups are engine constants with spec citations. |
| N5 wording ("Record" for "Confirm") | no change | Wording. |
| N16 and its parts a to h | partly | Main run, part a, part e (cull: `start_sinking` path) and the absence of a shadow are covered by the tests and the lane. Parts b, c and h need rocking (not built); part f needs a magnetic-lift drop drawn with the type's locomotor (design); part g needs claim 5's fire point (seam); part d is unmeasured. |
| Changes for other specs, S-0210's drawn sense of the body tilt | no change | Spec-side. We draw no crush tilt, so nothing of ours depends on the words. |
| Changes for other specs, a pointer to F5 from S-0106 claim 9 and S-0210 | no change | Spec-side. |
| Open questions (a rocked body's return to level; hovering and jumpjet composition; lift and teleport carriers; clipping) | no change | Our choices are the design rules above; no request filed. |

**Seams named here.** Rocking (S-0303 claim 20, S-0614 claim 10, S-0210's rates) is not
modelled, so the roll is always level and a sinking vessel always starts from level; slope tilt
*T* is not in F5; S-0313 claim 5's passenger fire point does not take the body transform.

**Requests.** None filed (block Q-1930 to Q-1934 unused): the open details are S-0504's own Open
questions, and each has a design choice above.

**Checks.** `cargo test --workspace`, `cargo xtask check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, both quirk-switch tests, the sim-gate
goldens with the ignored ones; the oracle arena and the M6c, M7a and M7b seed sweeps 1 to 16
before (at 1d7f987) and after, both before and after the rebase onto main 5c34005 (identical).
Lane `sinking-list` (map 3, a turretless voxel ship type, ids 363 and 367 at facings 0 and 16384,
an underwater type id 371 at 40960): level with no lean at the hit; after descent 1 pitch ∓0.01;
after 40 ∓0.3999997; after 79 and 80 ∓0.7853982 (bits 0xbf490fdb and 0x3f490fdb), the facing-0
vessel stern down, the others bow down; every locomotor drawn; the vessels gone 82 ticks after the
hit, after 80 descents.
