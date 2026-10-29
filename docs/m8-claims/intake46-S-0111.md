# S-0111 r21: claim ledger, M8 step A3 intake 46

This intake takes in **S-0111 revision 21** (spec commit e07cde2, approved 2026-10-29), which
answers **Q-2040** (how the control-group number looks). Our engine built revision 19 (c285b54,
[intake 2](intake2-S-0616-S-0111.md)). The diff r19 to r21 rewrites claim 17, adds a sentence to
claim 2, adds Formulas F6, two Edge cases, two Import mapping rows and scenario OV12's parts. The
pip scale of claims 9 to 16 is unchanged since r19 but was not drawn before this intake; it is
built here too, so the whole status-extras stack of claims 9 to 17 is now drawn.

## Code

- `app::mapview::pip_scale` (new, pure): the pip-scale kind (`pip_kind`, the `PipScale` words of
  the Import mapping, read through `value::named`), the pip image frames (`pip_frame`, the `Pip`
  and `OccupyPip` words with their defaults), claim 10's places (`place`, `places`), T2's row
  lengths (`row_length`), claim 12's passenger row (`passenger_row`, with the size blocks, the
  clipping and the gunner gap `passenger_place`; `absorbed_row`), claim 11's spawn and occupant
  rows, claim 13's mind-control row, claim 14's resource row (F2), claim 15's ammunition row (F3)
  and claim 16's self-heal marker (`self_heal`).
- `app::mapview::live`: `Live::pip_rows` draws, for every object whose pip scale claim 9 shows,
  the spawn row, the occupant row, the passenger or by-kind row and the self-heal marker, in that
  order, after the health bar and before the control-group number. `pip_scale_shown` now reads the
  kind through `pip_scale::pip_kind`. Missing `PIPS.SHP` or `PIPS2.SHP` give the notes
  `PIPS_MISSING` and `PIPS2_MISSING` (Import mapping; the rows using that sprite are left out).
- `app::mapview::combat`: `PIP2_SPRITE` and `Sprite::Pips2`; `Pip` gains `shadow` (claim 16's
  blink paints the marker as a shadow) and the constructor `Pip::image`; the control-group cache
  holds `GroupImage`s (built in ac3455a: `GroupFont`, `group_number_image`, F6).
- `app::mapview::build`: `PIPS2.SHP` joins the overlay assets.
- `app::game::session`: the Q-2040 design note on the group-number colour is replaced by the
  claim 17 citation (the owner's house display RGB of S-0103 claim 15).
- `glasnost_convert::sim_combat`: `pip_scale_checks` warns on an unknown `PipScale` word, on
  passengers with a pip-scale kind other than passengers, on the mind-control kind without a
  primary weapon, on the resource kind with storage 0, on a gunner transport whose row length is
  0 or less, on an absorbing building whose capacity exceeds its row, on an infantry or vehicle
  `Size` below 1 and on unknown `Pip` or `OccupyPip` words. The `SelfHeal*Frames` warning now
  names claim 16's blink.
- Commit ac3455a (already on this branch) built claim 17, F6 and the object order of the status
  extras (`Live::status_order`).
- No simulation change: no save format, AI game format, golden or arena change.

## Claims

| Claim | Status | Tests |
| --- | --- | --- |
| 1 (culling) | built (seams commit): status extras, the bomb icon and wrench included, are drawn only while the status anchor lies within the battlefield view widened by 360 px left and right and 180 px above and below, equality included; fog does not hide them. The cull is read at each status redraw, from `Live::battlefield` (no view known: no cull) | `claim_1_culling_margins` (OV1's shape: each of the four edges, at the margin and one pixel beyond) |
| 2 (shading: brackets, insignia, wrench, bomb sprite) | built (seams commit): the bracket image, the insignia, the corner brackets, the wrench and the bomb icon are `Paint::Lit` through the untinted shade table of their palette (overlay or cursor) at neutral brightness 1000, the path of S-0104 claim 15; segments, pips, the group number and its box stay `Paint::Row` | `claim_2_shading_of_the_bracket_and_the_insignia` (OV2's shape: a synthetic light-alpha 0 darkens bracket and insignia, not the segments), `repair_wrench_claim19`, `bomb_icon_claim24` |
| 2 (the control-group number and its box unshaded) | built | `the_group_number_follows_the_pip_scale` (the number is painted through its colour row, never shaded) |
| 8 (corner brackets) | built (seams commit); which corners is our design (see below) | `claim_8_corner_brackets`, `claim_8_a_selected_building_in_the_live_view` |
| 9 (who sees the pip scale) | built (r19 text; the gate now also gates the rows and the number) | `pip_rows_and_the_self_heal_marker` |
| 10 (placement) | built | `claim_10_places`, `pip_rows_and_the_self_heal_marker` |
| 11 (rows, order, covering) | built; spawn states and occupant order are design (Q-2072) | `claim_11_spawn_and_occupant_rows`, `a_building_occupant_row` |
| 12 (passenger pips: blocks, total size, clipping, gunner gap, absorbing buildings) | built | `claim_12_blocks_read_in_order_of_entry`, `claim_12_total_size_with_a_negative_size`, `claim_12_clipping_and_other_passengers`, `claim_12_gunner_gap_and_absorbing_buildings`, `pip_rows_and_the_self_heal_marker` |
| 13 (mind-control pips, overload pip) | built | `claim_13_mind_control_row` |
| 14 (resource pips) and F2 | built | `claim_14_resource_row` |
| 15 (ammunition pips) and F3 | built; a negative wrap reads as no wrap (Q-2071) | `claim_15_ammo_row`, `pip_rows_and_the_self_heal_marker` |
| 16 (self-heal marker, blink, T = 0) | built | `claim_16_self_heal_marker`, `pip_rows_and_the_self_heal_marker` |
| 17 glyph, rows past the pitch, missing glyph, box, colour | built (ac3455a) | `claim_17_glyph_and_box_follow_f6`, `claim_17_rows_past_the_pitch_paint`, `claim_17_no_box_at_a_small_pitch`, `claim_17_a_missing_digit_takes_the_fallback` |
| 17 placement and the pip-scale gate | built (ac3455a) | `the_group_number_follows_the_pip_scale`, `a_building_group_number_and_the_resource_switch` |
| 17 object order | built (ac3455a) | `group_numbers_follow_the_object_order` |
| 17 view edges | satisfied by the compositor's cut (the original's edge-line defect is not reproduced, as the claim allows) | none new |
| 17 missing or unusable font | built: no number | `group_numbers_follow_the_object_order` (no font: neither digit nor box) |
| F6 | built (ac3455a) | as claim 17 |
| T1, T2 | built | `t2_row_lengths`, `named_values` |
| Edge cases (new in r21: missing font, missing glyph) | built | as claim 17 |
| Import mapping (`PipScale`, `Pip`, `OccupyPip`, `ShowOccupantPips`, `Storage`, sprites, `GAME.FNT`) | built; importer warnings | `pip_scale_warnings`, `a_missing_pip_sprite_leaves_its_rows_out` (`PIPS_MISSING`, `PIPS2_MISSING`, the rows left out); `GAME.FNT` as claim 17's missing font |
| OV6 to OV12 | not run; measurable by the lead | local lane `crates/app/tests/pip_scale_lane.rs` gives pictures |
| Hand-offs (S-0014 font, S-0213 claim 25, S-0102 claim 1 object order) | read; S-0102's order used through `Live::status_order` | none new |

## Requests

- **Q-2071** (new): an unknown `PipScale` word, and a negative pip wrap. Interim: an unknown word
  gives no pip scale (the importer warns); a negative wrap reads as no wrap.
- **Q-2072** (new): which spawn slot states show a full pip, and in which order occupant pips
  fill. Interim: the docked and the reloading states show frame 1 at the slot's own index;
  occupant *k* fills pip *k* in S-0508's occupant order.
- **Q-2040**: answered by r21. Its interim is retired: our 3 × 5 digits, the white fallback for
  an object without an owner and session.rs's design note are gone.

## Design and seams

- The resource row's storage comes from the harvest rules (`econ.harvest`) for harvesters and
  slave miners; other types read the import `Storage` key (absent 0, which shows no pips, as
  claim 14 gives for C = 0).
- The importer's gunner check needs passengers above 0 and a row length of 0 or less (an absent
  `Ammo` defaults to −1). The size warning covers only an explicit `Size` below 1 on infantry and
  vehicles.
- Built in the seams commit: claim 1's culling, claim 2's shading and claim 8's corner brackets
  (rows above). The original's view-edge line defect and its fallback colour defect are not
  reproduced, as the spec allows.
- Claim 8 corners (our design, 2026-10-25 decision; the spec's confidence on which corners are
  drawn is low and gives nothing to choose from, so no request): the four corners of the box's top
  face and the three base corners its silhouette shows (left, front, right); the hidden back base
  corner is left out. At each, one line along each of the three edges that meet there (x, y and
  vertical), a quarter of the edge's length (256 W, 256 D, V leptons) rounded toward zero. Index 15,
  or 12 when the building's z is more than 4 leptons below its cell's level height (our reading of
  "height above terrain"; unreachable in play, as the spec notes). The line is one pixel wide
  (Bresenham). Selected buildings no longer get the viewer's egui placeholder (`bracket_drawn`).
- Culling seam: the scene is rebuilt per tick and on selection change, not per scroll step, so a
  pure scroll between redraws is covered only by the 360/180 margin until the next redraw.
- Not done: no by-eye lane shots added for these three.

## By eye

The local lane (`crates/app/tests/pip_scale_lane.rs`) writes `pips-<map>-group.png` and
`pips-<map>-building.png` (two maps by default) with the house's building, nearest vehicle and
infantry, and its harvester if any, in groups 1 to 4, the vehicle at half strength and its owner
given unit self-heal. The game is not run on, so the pictures show one moment only: the group
digits in the game font in their black boxes with the house-colour ring, the health bars, and the
vehicle's self-heal marker at the map's first tick, which falls in the blink (t mod |T| < 6), so
the marker is drawn as a shadow. In the first run neither map had a harvester, and the lane picks
no transport or ammunition unit, so the passenger, ammunition, resource, spawn, occupant and
mind-control rows and the marker's plain phase still need a look in a played game (or OV6 to
OV11).
