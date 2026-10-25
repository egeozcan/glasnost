# S-0109 r41: claim ledger, M8 step A3 intake 22

This intake takes in one spec revision. It answers two requests, so two interims are retired:
Q-1716 (the order of an advance's damage and its middle event) and Q-1717 (the warhead of the
built-in invisibility-animation type's damage).

- **S-0109 revision 41** (spec commit 62081c1; Q-1716 answered in 63d7407, Q-1717 in 9751b4d).
  It covers r33 to r41 against r32, which step A5 built. Most of the diff is rewrapping. The
  changes in substance:
  - State and ordering, claims 14 and 17 (Q-1716): within one advance the damage detonation
    comes first, with all its draws and kills, then the middle event (particles, then the
    scorch-or-crater choice and any crater's resource removal), then the turn and pass tests
    (a new pass with its loop-delay draw, the follow-up switch with its delay draw and start
    event, or the end). The same holds on the ending advance and on the advance before a
    follow-up switch.
  - State and ordering, new: *An animation that leaves play in its own detonation*. Nothing
    more happens to it in that update. The original's result is undefined; our engine stops.
  - Open questions, new: an attached animation whose own detonation removes its owner goes on
    with the rest of that update. Our engine uses the owner's last position (design).
  - Claim 17 and Import mapping (Q-1717): the warhead is chosen on each delivery by comparing
    the current type's stored name, byte for byte, with the built-in invisibility-animation
    type's name. That type takes `C4Warhead`; every other type takes `FlameDamage2`. A type
    first registered in other letter case never matches, and with no type of exactly that name
    `C4Warhead` is never chosen.
  - New scenarios AN28 (damage before the middle event, seen through a crater's fit) and AN29
    (the warhead choice, part b the letter case). Confidence and backing lines are updated.
  - Changes for other specs: "None". The joint hand-off is now a plain hand-off paragraph.
  - The four quirk switches (`anim_damage_starts_at_one`, `anim_fast_rate_freezes`,
    `anim_reverse_raw_loop_end`, `paratrooper_any_animation_slow_fall`) are only reworded.
  - The drawing's claims (19 to 21, 26, 27) are only rewrapped.

**The lead's ruling** (docs/m8-plan.md, S-0109: "the Import mapping may name the stock
invisibility animation literally"). r41 does so, and only there. In our code the name appears
once, in `rules::INVISIBILITY_ANIM_NAME`, cited to Import mapping. Every other place names the
type by its role (`AnimRules::invisibility`, `anims::damage_warhead`).

**Code** (simulation and importer coverage; no new hashed state).
- `rules.rs`: `AnimRules::invisibility`, set at rules build when the type's stored name (its
  registry spelling) equals `INVISIBILITY_ANIM_NAME` byte for byte.
- `anims.rs`:
  - `damage_warhead(rules, type)`: `C4Warhead` for the invisibility-animation type, otherwise
    `FlameDamage2`. Used by `AnimEnv::damage` and by the burning walker's expiry damage
    (`walker::advance_damage`). Because the choice reads the current type, a follow-up switch
    changes it from the follow-up's first delivery on.
  - `AnimWorld::current` now also holds the role of the animation taking its turn, and
    `with_role` sees that animation while it has not been removed. Before, a drain link that ended during the drain
    animation's own detonation could not find the animation (it is out of the map during its
    turn), so it was never removed.
  - `AnimEnv::advanced` returns after the damage when the animation left play (`left_play`),
    before the middle event.
- `anim_clock.rs`: `Env::left_play` (default false). `Player::advance` returns after
  `env.advanced` when it is true, so there is no turn or pass test, follow-up switch or end.
- Q-1716 and Q-1717's interim comments in `anims.rs` and `walker.rs` are replaced by citations.
  The two rows in `docs/development.md` are removed.
- `convert/src/sim_combat.rs`: the `[CombatDamage] C4Warhead` coverage row also cites S-0109.
- `tools/src/local_lane.rs`: three counts (types named exactly as the invisibility-animation
  type, those with damage above 0, and whether `C4Warhead` names a warhead in play).

**Tests.**
- `crates/deterministic/sim/tests/claims_s0109_r41.rs` (new):
  `claim17_the_invisibility_type_takes_the_demolition_warhead`,
  `claim17_a_type_first_registered_in_other_case_never_matches`,
  `claim17_a_follow_up_switch_changes_the_warhead`,
  `state_and_ordering_the_detonation_comes_before_the_middle_event`,
  `state_and_ordering_the_middle_event_comes_before_the_end`,
  `state_and_ordering_the_middle_event_comes_before_a_follow_up_switch`,
  `open_question_owner_removed_by_its_own_detonation`.
- `claims_s0608.rs`: `the_drain_animation_leaves_play_in_its_own_detonation`.
- `convert/src/sim_special_tests.rs`: `an_animation_keeps_its_first_registered_spelling`.

| Claim or item | Status | Notes |
|---|---|---|
| Summary, hand-off paragraph (was the joint hand-off) | followed | Spec-side text; nothing to build. Its statements (creation-time sound and chain reaction, the crater's 6 bales from stage 6) were built at A5 and S-0202's intake. |
| Claims 1 to 13, 15, 16, 18 to 29: rewrapped only | no change | Checked line by line in the word diff. |
| Claim 14, the middle event after the same advance's damage detonation | built, tested (new) | Already in this order (`AnimEnv::advanced`); Q-1716's interim retired. `state_and_ordering_the_detonation_comes_before_the_middle_event`: the detonation's kill takes a list pick, and the scorch-or-crater choice must come from the state after it. Seeds are checked where the two orders give different choices. |
| Claim 14, before the end and before a follow-up switch | built, tested (new) | `state_and_ordering_the_middle_event_comes_before_the_end` (one choice draw on the ending advance, then removal), `state_and_ordering_the_middle_event_comes_before_a_follow_up_switch` (the choice draw, then the follow-up's delay draw: checked against the reverse order on discriminating seeds). |
| Claim 17, the warhead by the current type's name, compared exactly | built, tested (new) | `AnimRules::invisibility`, `anims::damage_warhead`. `claim17_the_invisibility_type_takes_the_demolition_warhead` (100 % against 0 %). |
| Claim 17, a follow-up switch changes the warhead from its first delivery | built, tested (new) | `claim17_a_follow_up_switch_changes_the_warhead` (a follow-up named in lower case finds the type registered in capitals). |
| Claim 17, the burning walker's expiry damage | built | `walker::advance_damage` uses the same choice. Not tested separately. |
| State and ordering, the playing row and *Observable order within one update* | built, tested (new) | As claim 14. The order of draws inside the detonation itself is S-0302's and S-0303's, unchanged. |
| State and ordering, *An animation that leaves play in its own detonation* | built, tested (new) | Our engine's rule, as the spec states. `left_play` in `AnimEnv` and `Player::advance`. `the_drain_animation_leaves_play_in_its_own_detonation`: the drain animation's damage kills its drainer, the link ends, the animation is removed in its own turn and takes no follow-up switch or delay draw. The spec's known case (a tank bunker's *special-1* and *special-2* slots) is a seam: building slot animations are not simulated (claim 23). |
| Open questions, an owner removed by the animation's own detonation | built, tested (new) | Our design, as the spec states: the update goes on (middle event, turn and pass tests) at the owner's last position, and the animation is deleted at its next update (claim 22). `open_question_owner_removed_by_its_own_detonation`. |
| Import mapping, the invisibility-animation type's name; first-registered spelling | built, tested (new) | `rules::INVISIBILITY_ANIM_NAME`; the importer keeps a type's first spelling (S-0004 claims 2 and 7, already so). `claim17_a_type_first_registered_in_other_case_never_matches` (sim) and `an_animation_keeps_its_first_registered_spelling` (convert). |
| Import mapping, `C4Warhead` of `[CombatDamage]` | built, tested | Already converted (S-0302, S-0303); the coverage row now names S-0109 too. |
| Import mapping, with no type of exactly that name `C4Warhead` is never chosen | built, tested (new) | No type has the flag; `claim17_a_type_first_registered_in_other_case_never_matches`. |
| Edge cases, the four quirk switches (reworded only) | no change | `anim_damage_starts_at_one` (rules quirks; both ways in `claims_s0109_playback.rs`), `anim_fast_rate_freezes` (rules quirks and mapview; both ways in `claims_s0202.rs`), `anim_reverse_raw_loop_end` (`anim_clock::AnimQuirks` and `mapview::anims`; the simulation plays with `AnimQuirks::default()`, a seam since A5; the off position in `mapview/anims.rs`), `paratrooper_any_animation_slow_fall` (rules quirks; both ways in `claims_s0522.rs`). |
| Importer warnings | no change | None added or changed. |
| AN28 | tested on synthetic data, partly | Smudges are not modelled, so the crater's fit, which is what AN28 observes, cannot be reproduced. The stage outcome is the same under either order (the scenario says so), so our tests check the order through the draws instead. The lead's runs are theirs. |
| AN29 | tested on synthetic data | Main run, variant and part b, on a vehicle in place of a stock building. The lead's runs are theirs. |
| Confidence and backing lines | noted | Spec-side. |
| Changes for other specs ("None") | followed | Nothing to build. |
| Open questions, Q-1716 and Q-1717 answered | followed | Interims retired (above). |

## Requests

None filed (block Q-1900 to Q-1904 unused). r41 settles both of its open cases as our engine's
design, and the code follows them; the only known case of an animation leaving play in its own
detonation (the tank bunker's slots) is not simulated.

## Checks

- The only new state, `AnimWorld::current`'s role, is not saved (`#[serde(skip)]`, empty
  between turns), so the save format stays 78 and the AI game format 40.
- Oracle arena: before and after on base d4a79ba, again on base a7478bb after a rebase, and an
  after run on 9916bba matching a7478bb's before (intake 23 then recorded its own arena and
  sweeps identical, so these carry over to the final base f3bbe51). The CSVs are byte-identical (M-0001 31 of 96
  pairings failing, 151 of 172 unanimous outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003
  2 of 35, 61 of 64). The M6c, M7a and M7b seed sweeps 1 to 16 print the same lines (smallest
  minimum M6c 2, M7a 1, M7b drain 3). The sim-gate goldens hold without the update flag; none
  was regenerated.
- Local lane (`local-lane/m8/a3-22/lane.txt`, counts only): of 621 animation types registered,
  1 is named exactly as the invisibility-animation type, and it has damage above 0;
  `C4Warhead` names a warhead in play. So the installed data now takes the demolition warhead
  for that type's damage wherever a game creates it.
