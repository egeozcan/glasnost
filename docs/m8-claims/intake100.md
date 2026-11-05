# S-0302 r41: claim ledger, M8 step A3 intake 100

Last built revision: r40 (intake 79, `docs/m8-claims/intake79.md`). Delta read in full
(`git diff 0af260a~1 0af260a -- spec/`): claim 10's berserk bullet and its backing and confidence, one
sentence of claim 12 (a berserk hit has none of the reactions *below*), D23 part f's wording, and
five Changes entries (S-0215 claim 16, S-0215 claim 39, S-0208 claim 15, S-0107 claim 16, S-0701 claim
28). No quirk is added or changed; `requests:` is unchanged (Q-0061, Q-0075, Q-0081, Q-0170, Q-0648), so
no request is answered by this revision. The delta restates which reactions a berserk hit makes.

Before r41 our berserk branch of `damage::apply_hit` returned `Unchanged` with no reaction at all.

| Claim part | Status | Code | Test |
|---|---|---|---|
| Claim 10: a berserk hit changes no health, reports *unchanged*, sets off no anger change, retaliation, scatter from damage, trigger event, damage effect or disguise loss | tested (held already) | `damage::apply_hit` berserk branch returns before `reactions` | `claim_10_a_berserk_hit_causes_no_retaliation`, `claims_s0610` and D34 tests |
| Claim 10: the no-threat fall-back (S-0215 claim 16) follows a berserk hit from a known attacker, replacing the hunt the entry queued | tested | `process::no_threat_applies` judged before the entry, `process::fall_back` after it (`damage::berserk_reactions`) | `claim_10_a_berserk_hit_makes_a_no_threat_infantry_unit_fall_back`, `claim_10_no_fall_back_without_a_known_attacker`, `claim_10_no_fall_back_without_the_no_threat_property`, `claim_10_a_renewing_berserk_hit_also_makes_the_actor_fall_back` |
| Claim 10: the change of fear of an infantry unit (S-0107 claim 16), after the fall-back, with the health ratio as it stands, no attacker needed for the plain increment | tested | `fear::damaged` from the berserk branch | `claim_10_a_berserk_hit_changes_the_fear_of_an_infantry_unit`, `claim_10_a_berserk_hit_without_an_attacker_raises_fear_by_the_increment` |
| Claim 10: the crush response of a vehicle (S-0215 claim 39) | tested (held already) | `damage::apply` | S-0215 claim 39 tests |
| Claim 10: the harvester-under-attack announcement (S-0701 claim 28) | tested | `Sound::HarvesterAttacked` event, `audio::events::harvester_attacked` | `claim_10_a_berserk_hit_gives_the_harvester_announcement_event`, `a_berserk_hit_on_a_local_harvester_is_announced` (audio) |
| Claim 10 with `quirks.berserk_zero_duration` (S-0610, default on): a hit with T = 0 | tested, both positions; interim rule: with the switch off the hit "has no effect on the target" (S-0610 Edge cases), so no reaction follows (the spec says no more) | the reactions are gated on `psychic::enter`'s return | `claim_10_a_zero_duration_berserk_hit_follows_the_switch` |
| Claim 10: the three excepted cases report *unaffected*, nothing happens (no fall-back, no fear, no event) | tested | the early return | `claim_10_an_allied_attacker_house_stops_the_hit_before_every_reaction` |
| Claim 10: a team member has left its team | seam | no house of our engine has teams (S-0215 Terms) | none |
| Claim 10 / S-0208 claim 15: request B (the threat-point scatter request) after a berserk hit with a known attacker; S-0215 claim 19 request A is not sent | seam | the scatter from damage is built for no hit (`docs/m8-claims/seam-s0215-c16.md`); owner: the intake that builds S-0208 claim 15 | none |
| Claim 12: "a berserk hit has none of the reactions below" | partly | as above (no damage effect, no disguise loss: the branch returns first; no test lands a berserk hit on a disguised spy, scenario D23 f is `unmeasured` here and has no test) | the health, retaliation and fall-back tests of claim 10 |
| Sound reactions to a berserk hit other than the harvester announcement (the damage sound of S-0701 claim 20, the ally-under-attack announcement) | request | **Q-2626**; interim rule: none (the behaviour before r41) | none |
| Changes entries for S-0215 claims 16 and 39, S-0208 claim 15, S-0107 claim 16, S-0701 claim 28 | pointers | owners build their claim; the berserk trigger is built here as above | |
| Older ledgers (`intake38-M-0011.md` lines 27 and 46: "a berserk hit sets off no reactions") | describe behaviour before r41; the M-0011 tests still pass | | |
| Quirks | none added | | `every_spec_quirk_has_a_switch` unaffected |
