# M6b step 6b (warp whole-turn exclusion, placement acceptance, deck flag, cloak warp suppression): spec revisions since 090046f

Buckets as in [step5.md](step5.md): **(a)** implemented with a test; **(a, already done)**; **(b)** seam;
**(c)** editorial; **(e)** not built. Front matter changes are (c).

| Spec | Revisions |
|---|---|
| S-0516 Cloak and stealth | 8 to 9 |
| S-0513 Superweapon effects | 13 to 14 |
| S-0501 Aircraft | 15 to 16 |
| S-0614 Parasites | 6 to 7 |

Requests. **Answered:** Q-0870 (S-0513 claims 51 and 52), Q-0871 (claim 50), Q-0872 (S-0501 claim 44). Interim rows for
Q-0870 and Q-0871 are retired. The Q-0872 row and the `air.rs` marker are left to the merge: the deck-flag code
(`air.rs`), the `tick.rs` early-outs and `fire::reload_elapse` belong to the parallel session specs-r5a (an
untracked `overlap-r5a.patch` in the worktree root holds my version of them). **Filed:** none.

Saved state and schema. `warp::Member` gains `entry_ok` (`serde(default)`, saved state). The rules schema is unchanged.
The pending r5a parts add `AirState::mark_layer` if they follow my patch.

Tests marked `#[ignore]` below pass only with r5a's parts; remove the ignore after merging.

## S-0513 (rev 13 to 14)

| Claim | Bucket | Code, tests |
|---|---|---|
| 50 placement acceptance (Q-0871) | (a) | `warp::placement_accepted`, `enterable_now`, `Member::entry_ok`. Tests in `claims_s0513_placement.rs`: building, protected object, vehicle that arrived first, two members for one cell, infantry standing positions, terrain change vs unchanged fatal terrain. Full recheck on the next update was already the `Blocked` stage. An infantry unit whose standing position stays taken is refused indefinitely (the spec allows waiting forever). |
| 51 whole member turn (Q-0870) | (a, already done) for behaviour start, cloak, parasite, economy and power, depot retry, firing frame; (a) for reload elapse, expected-health bookkeeping and protection phases | `warp::turn` returns the whole turn; `claims_warp_turn.rs` (depot, firing frame); reload, expected-health and protection tests are `#[ignore]` until r5a's `tick.rs`/`fire.rs` parts merge. |
| 52 original-side gaps | (c) | Evidence table only. |
| Import mapping rows | (c) | Engine design, no data key. |

## S-0516 (rev 8 to 9)

| Claim | Bucket | Code, tests |
|---|---|---|
| 20 backing text (type-radius change unmeasured) | (a, already done) | No runtime type-radius change exists; `claim_20_rows_without_a_phase_change`. |
| 22 table blank line, F5 rewording, CS test text | (c) | F5 is the same recurrence (`claim_19_field_shape`). |
| 42 ordering preface, explicit causes need no ordinary update | (a, already done) | `claim_12_hits_uncloak`, `claim_7_durations`. |
| 42 warp suppression | (a, already done) | Stamp-based fade; `s0516_claim_42_no_fade_step_through_arrival_effects_and_one_stage_in_the_hold`, `s0516_claim_42_a_hit_and_the_arrival_sensor_reveal_are_not_postponed` (`claims_warp_turn.rs`). A leaving member takes no hit (being erased). |

## S-0501 (rev 15 to 16)

| Claim | Bucket | Code, tests |
|---|---|---|
| 44 deck-rest flag (Q-0872) | (a), owned by specs-r5a | Tests `claim_44_*` in `claims_s0501.rs`, `#[ignore]` until merged. |

## S-0614 (rev 6 to 7)

| Claim | Bucket | Code, tests |
|---|---|---|
| 18 warp exclusion of the parasite step | (a, already done) | `s0614_claim_18_an_attached_parasite_makes_no_ordinary_step_before_the_hold`. |
| Open question removal | (c) | |

Related: S-0301 claims 11 and 13 and S-0203 claim 4 carry the same exclusion: tests `s0301_claim_11_*`, `s0301_claim_13_*`.
