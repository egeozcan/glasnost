# Intake 73: S-0701 r36 (claim 17, scenario D3)

Client-only presentation. No simulation, AI, save, wire or golden change; the sim and AI lanes are skipped because nothing they run changed. The behaviour was already built by the S-0214 claim 32 work (`Play::type_tap` pushes one entry onto `selected`, `expand_types` and `type_drag` push none); this intake adds the tests that pin r36.

| Claim part | Status | Where |
| --- | --- | --- |
| Claim 17: a type-select tap gives one response, the first added object's | tested | `claim_32_a_tap_widens_from_the_screen_to_the_map`, `claim_17_a_tap_adding_several_objects_gives_one_response` (play_tests.rs); one entry in `Play::selected` is one `Audio::selected` call, so one choice draw (`selection_response_draw`, events_tests.rs) |
| Claim 17: a tap that adds nothing gives no response and no draw | tested | same two tests (empty `selected`) |
| Claim 17: held type-select click (select, toggle add, toggle remove) is silent | tested | `claim_17_a_held_type_select_click_is_silent` |
| Claim 17: held type-select drag is silent | tested | `claim_32_a_held_drag_...` assertion "the drag is silent" in play_tests.rs (`type_drag`) |
| Claim 17: the rest (first-object rule, re-click, modifier deselect, passive and other houses, empty lists, unsynchronised stream) | unchanged | existing tests |
| Scenario D3 (original's per-object responses to a tap) | seam | oracle-side observation, not our engine's rule |
