# M8 A3 intake 117: S-0506 r40, S-0411 r60

Brief: `.claude/worktrees/M8-A3-INTAKE117-BRIEF.md`. Answered requests: **Q-2811** (S-0506 r40
claim 11: a controller target lost while a pending target waits) and **Q-2831** (S-0411 r60 claim
23: the marks of a tip entry's text). No new request. One quirk is new,
`launching_spawner_resets_after_target_loss`, with its switch at the spec default (on) and a C7
verdict in [c7-play-fixes.md](c7-play-fixes.md): not admitted to the play-fixes layer. S-0411 r60
adds two display-only departures without a switch. Both interims (`design: Q-2811` in
`spawn::target_gone`, `design: Q-2831` in `sidebar_tip::fill`) are gone. S-0506's Open question on
Q-2821 (S-0501 claim 18's missile-type rows) stays open, as the brief says.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0506 r40 claim 11 (Q-2811): the controller target is cleared at the loss also while a pending target waits, at each loss event alike (destroying hit, going inside, removal, cloaking), and the loss gives no reset | built, tested | `spawn::target_gone`; `claims_s0506_r40.rs` `r7_f_a_loss_while_a_pending_target_waits_clears_then_the_next_pass_resets` (all four events), `claims_s0506_r34.rs` `no_reset_when_a_pending_target_waits` (rewritten: target cleared, pending kept) |
| S-0506 r40 claims 5 and 11, Controller table: at the next pass a launching controller with no controller target resets (preparing missiles released with no target, so to the F8 cell, slot empty with a regeneration countdown, pending cleared); no launch at the pending target in that pass; a released rocket keeps its aim point | built, tested | `spawn::controller_step` (launching arm); the same test |
| S-0506 r40 claim 11: the next launch needs a later shot and an idle pass | built, tested | `r7_f_the_next_launch_needs_a_later_shot_and_an_idle_pass` |
| S-0506 r40 claim 11: the pending target lost as well: cleared with no reset; the launching controller resets at its next pass; no launch at either object | built, tested (both switch positions) | `r7_f_both_lost_the_next_pass_resets_and_launches_at_neither` |
| S-0506 r40 claims 3, 5 and 11: a non-missile slot whose object is preparing adopts the pending target before the controller's check, and the controller goes on launching (an ordinary carrier and a flagged non-rocket carrier) | built, tested (both switch positions) | `slot_step` order in `spawn::pass` (unchanged); `r7_f_a_preparing_non_missile_slot_adopts_and_the_controller_goes_on` |
| S-0506 r40 claim 11: a cooling-down controller keeps the pending target until an attacking or returning slot adopts it, or its first idle pass | built (already), tested | `controller_step`, `slot_step`; `a_cooling_down_controller_keeps_the_pending_target_for_an_attacking_slot` (the idle-pass adoption is the existing claim 5 test path) |
| S-0506 r40 Terms and Slot table: a non-missile slot's preparing rows move its object whether flagged or not (re-sent to the step direction 4 neighbour, or returning with no controller target) | built, tested | `spawn::slot_step` (the flagged branch removed); `the_preparing_rows_move_a_flagged_non_rocket_object`; `claims_s0506.rs` `flagged_non_rocket_slot` (rewritten: re-sent flagged or not) |
| S-0506 r40 claim 11: after a double loss a flagged non-rocket preparing object has turned returning, so the reset does not release it | built, tested | `both_lost_a_flagged_non_rocket_object_returns_and_is_not_released` |
| S-0506 r40 Terms and claim 11: a reset releases a preparing flagged non-missile slot's object; a preparing object without the flag is not affected | built (already), tested | `spawn::reset`; `a_reset_releases_a_preparing_flagged_non_rocket_object_only` |
| S-0506 r40 claim 4: claim 8's release detaches a flagged non-missile slot's object | built (already), tested | `spawn::release`; `claims_s0506.rs` `flagged_non_rocket_slot` |
| S-0506 r40 claim 14: with a pending target waiting, the leaving defers to claim 11's next pass | built, tested | `spawn::target_leaves_play`; `no_reset_when_a_pending_target_waits`, the r40 test's `Inside` run |
| S-0506 r40 quirk `launching_spawner_resets_after_target_loss` (default on); off, the launching controller adopts the pending target at that check with no range test and goes on launching, its preparing missiles waiting for claim 8's release with that target | built, tested both ways | `quirks.rs`, `ruleset.schema.json`; `quirk_off_the_launching_controller_adopts_the_pending_target`, `the_switch_defaults_on`, the two both-position tests above; C7: not admitted |
| S-0411 r60 claim 23, *Marks*: flags, width, precision and size marks in that order; s, the number letters, a doubled percent sign; marks shown as they stand (c C B e E f g G n S Z, s or a number letter after I64, s after h, any asterisk mark, also at the end of the text); others dropped for their ending character; an unended mark dropped | built, tested | `sidebar_tip::{read_mark, end_of, fill}`; `claim_23_marks`, `terms_insertion_points` |
| S-0411 r60 claim 23, *Which value each point takes*: each point the next value of its own kind; a point with none left writes nothing (its width and flags too); leftover values unwritten | built, tested | `fill`; `claim_23_values` |
| S-0411 r60 claim 23, *Forms*: text precision; d and i signed; u, o, x, X as u32; p as X with precision 8; precision as least digits, 0 with value 0 writes none; h the low 16 bits; l w F L N change nothing | built, tested | `sidebar_tip::{text_point, number_point}`; `claim_23_forms` |
| S-0411 r60 claim 23, *Flags and width*: plus and space; hash for o, x, X, p; width with sign and prefix; minus; zero padding between sign or prefix and digits, spaces with a precision; zeros pad a text point | built, tested | `number_point`, `text_point`, `Out::padded`; `claim_23_flags_and_width` |
| S-0411 r60 claim 23: the money-format tip turns a width's spaces into line feeds; the power tip keeps them | built, tested | `icon_text`, `power_text`; `claim_23_flags_and_width` |
| S-0411 r60 scenario S16 part j, texts (1) to (8) | tested, with invented names and values | `claim_23_s16_part_j_texts` |
| S-0411 r60 Edge cases: a point given a value of another kind; marks with a defined original result shown as they stand; the power-meter long text shown as the name tip's | built, tested | `fill`; `name_tip::cut` (unchanged); `claim_23_a_large_width_is_cut_as_a_long_text`; C7 rows, no switch |
| Our reading: p with the zero flag | built, tested | claim 23 gives p "always a precision of 8", and "a number point with a precision gets spaces", so `%012p` pads with spaces (`claim_23_flags_and_width`). Read as stated; no request |
| Our bound on long output | built, tested | `fill` writes no more than `name_tip::TEXT_LIMIT` (256) units, the cut every tip text passes, and widths and precisions saturate; nothing shown changes |

## Render lane

`r8_render_lane.rs` `i115_the_sidebar_tips` was re-taken on this branch. All eight screens are
byte-identical to intake 115's `after` screens, and the printed boxes and line counts are the same.
The stock entries have one text point and then one number point (money format), or two number
points (power), and claim 23 fills those as before. The lead sees no change, so no a4-play-notes
row is added.

## Lanes

Base: 1c88f74 (save 117, AI game 83). Main moved to 7006a96 during the intake: S-0501 r172 (spec
only) under the same S-0402 merge, so 7006a96's code equals 1c88f74's (they differ only in `spec/`
and `docs/`), and the base figures hold for it. "After" is this branch before the rebase, whose
code equals the rebased branch's.

- Oracle arena (`--record all --threads 2`): the output files are byte-identical before and after.
  Failing rows as main: M-0001 12 of 96, M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 1 of
  25, M-0021 11 of 50.
- Seed sweeps 1–16: M7b, M7a and M6c print the same lines before and after (M7b smallest drain
  minimum 3; M7a smallest minimum 1; M6c smallest minimum 0, as on main).
- `glasnost-ai` with ignored tests, release: 264 pass before and after.
- AI quick lane (seeds 7–10, maps 0, 2 and 8, three side pairs, 36 games): every game ends at the
  same tick with the same winner. No winner changes, so there is nothing to trace: no lane game
  loses a missile spawner's controller target while a pending target waits in a way that reaches
  the result.

## Formats

Behaviour changes what a saved game and an AI game replay to, and the rules gain a quirk field, so
the save format goes to 119 and the AI game format to 85, one past intake 116's 118 and 84 on
main bdb91f1. No hashed state is added. The determinism gate's goldens were regenerated on the
branch rebased onto bdb91f1 and none moved: no gate scenario loses a missile spawner's
controller target while a pending target waits.
