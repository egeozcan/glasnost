# M8 step A3, intake 27: S-0105 r39 (voxel rendering; Q-1885, the barrel of a sprite-body vehicle)

S-0105 revision 39 answers Q-1885, which intake 19 filed. Q-1885 asked four things about the voxel
barrel of a vehicle drawn as a sprite: its depth row, its depth above height 0, its place in the
drawing order beside the sprite turret, and its HVA frame. We had built revision 35; this record
covers `git diff c2ee154 52a739b -- spec/rendering/S-0105-voxel-rendering.md`. The revision
also proposes changes to S-0102 claims 13, 14 and 22. That S-0102 revision has not landed, and
this intake does not take it in.

**Code.**
- `crates/render/src/voxel.rs`:
  - `sprite_body_barrel_shift` gives claim 12's *d = trunc(δ / 8)*.
  - `sprite_body_barrel_pose` places the barrel by claim 8's transform with that shift. It turns
    the barrel by claim 4's relative angle and pitches it by claim 6's elevation about claim 10's
    axis.
  - The docs of `VoxelQuirks::barrel_frame_zero` and `barrel_frame_with_turret` now say the
    switch covers barrels drawn with a voxel turret only. The sprite-body barrel stays in frame 0.
- `crates/app/src/mapview/build.rs`:
  - With the turret flag, `sprite_vehicle` draws its shadow first, then one composed item.
    Without the flag it is unchanged.
  - `sprite_body_barrel` draws the barrel part in frame 0 with the new pose, keyed in frame 0.
  - `sprite_turret_parts` orders the parts by claim 11's 4-step turret facing.
  - `compose_parts` paints the parts into the exact union of their rectangles.
  - `composed_item` caches each distinct parts list once (`Painter::composed`). It gives the
    composed image the voxel-vehicle row *U*, upright, a reader.

**No new simulation state.** Drawing code only: save format 78 and AI game format 40 stay, and
`crates/deterministic` and `crates/sim-gate` are untouched. No new switch.

**Tests.**
- `voxel::tests::sprite_body_barrel_pose_shifts_in_eighths` (render).
- `mapview::live_tests::a_sprite_body_is_drawn_as_one_composed_image` replaces intake 19's
  `a_sprite_body_draws_its_single_voxel_barrel`. It runs on synthetic SHP, VXL, HVA and VPL data.
- `mapview::live_tests::composed_parts_fill_their_exact_union_in_order`.

## Claims

| Change | Status | Notes |
|---|---|---|
| Claim 12, *Sprite body*: without a barrel part the body and turret frames are still composed into one image | built, tested (new) | With the turret flag and no `barrel` part, `sprite_vehicle` composes the body frame and the turret frame and gives the image *U*. The bare case of `a_sprite_body_is_drawn_as_one_composed_image` composes two parts with *U*, and their last row is the body frame's. |
| Claim 12, *Sprite body*, Backing and Confidence (the old `unmeasured` pose and overlap notes dropped) | noted | Wording. The pose and overlap now have their own paragraphs, below. |
| Claim 12, *Barrel pose*: claim 8's transform with *d = trunc(δ / 8)* along the body's forward axis, claim 4's relative turn, claim 6's elevation about claim 10's axis | built, tested (new) | `voxel::sprite_body_barrel_pose`. The shift is integer division truncated toward zero. `sprite_body_barrel_pose_shifts_in_eighths` checks MS-33 part k's expected shifts for the pose runs' δ = 0, 200, −100 and −12 (0, 25, −12 and −1). It checks that the shift differs from a voxel vehicle's `vehicle_turret_shift` (the falsifier's δ × ρ), that the pose is the composed transform with its pivot at the shifted point, and that at δ = 0 the pose equals `vehicle_parts`'s barrel. The original's off-axis pitch pivot is not reproduced, as the spec says (Lead-confirmed presentation departures, extended 2026-10-25). |
| Claim 12, *Invisible slot-0 projectile* (original glitch, fixed without a switch) | followed, by construction | The barrel takes the vehicle's elevation (`m.elevation`, claim 6) whatever the slot-0 projectile, so we draw claim 6's elevation. No switch, as the spec decides. |
| Claim 12, *Frame* (Q-1885 question 4): always HVA frame 0, in both states of `quirks.barrel_frame_zero` | built, tested (new) | `sprite_body_barrel` draws frame 0 and keys it by frame 0. The test sets the animation counter to 3: the barrel image is unchanged, and only the frame-0 key is cached. The `barrel_frame_zero` doc comment says the sprite-body barrel stays in frame 0. Intake 19's interim (the frame of a barrel drawn with a voxel turret) is retired. |
| Claim 12, *One image* (Q-1885 question 3): body frame first; at 4-step turret facings 0 and 3 the barrel then the turret frame, at 1 and 2 the turret frame then the barrel; later pixels win, transparent pixels paint nothing; at the body's draw point; shadow first, its own row | built, tested (new) | `sprite_turret_parts` (using `voxel::turret_drawn_first`) and `compose_parts`. The test checks the parts order at facings 0, 16384, 32768 and 49152 (4-step 0 to 3). It checks that the composite equals `compose_parts` of those parts and that the later part wins pixel by pixel. It also checks that the barrel covers body pixels and that the one shadow item comes first with its row *U* − 2, flat (S-0102 claim 22, unchanged). The composed image is drawn at the body's draw point, so on a bridge deck the barrel lifts with the body (lane, below). The test also checks the single barrel at turret indices *none*, 0, 1 and 5 of a multi-turret type (claim 43), as before. |
| Claim 12, *Depth* (Q-1885 questions 1 and 2): one bias and profile for all pixels, *U*, upright, a reader, at every height; rectangle the smallest one holding all parts, its last row F2's anchor; exact rectangle, the original's one-row extension fixed without a switch; S-0102's sprite rows do not apply with the turret flag | built, tested (new) | `composed_item` gives one `Profile { Upright, bias: U }` item, a reader, at every height (the same *U* that a voxel vehicle receives). `compose_parts` takes the exact union, with no extra row or column. `composed_parts_fill_their_exact_union_in_order` checks rectangles where the barrel ends inside, below and to the right of the body. The live test checks that the composite's rectangle is the union of the parts' rectangles. The sprite rows *U* − 2 and −*L(z)* − 2 now apply only without the turret flag (the `body` closure's comment). Intake 19's interim (the sprite row *U* − 2 on each part) is retired. |
| Claim 12, *Depth*, *Split of a tall image* (S-0102 owns it; its second condition is open) | not built: seam, our design | We never split the composed image. One condition is not identified, and S-0102 owns the split, so a partial split would draw a too-big type differently by a rule we cannot finish. **Our design** (decisions of 2026-10-25): no split, *U* upright for every row. It differs from the original only for a type too big to fit under a bridge whose composed image is taller than 16 rows, near a bridge. Our sprite path has no row for that flag either (S-0102 claim 13's *U* − 18 row is not built), so nothing regresses. |
| Claim 12, *No barrel shadow* | built, tested | No shadow item for the barrel. The test checks that the one shadow item is the sprite shadow frame (12 × 4 in the test file, flat). |
| Claim 16: "Barrels drawn together with a voxel turret"; the sprite-body barrel stays in frame 0 in both switch states | followed | The renderer's frame rule for voxel turrets is unchanged. The `barrel_frame_zero` and `barrel_frame_with_turret` docs now say this. |
| Quirks, `quirks.barrel_frame_zero` text (voxel turret only; sprite-body barrel stays in frame 0) | followed | Doc comment only. The switch's default and behaviour are unchanged, and the switch tests are unaffected. |
| Scenario MS-33 part k, sprite-barrel runs (depth, pose, frame, invisible), expected and falsifiers | tested on synthetic data, in part | Pose: the shifts 0, 25, −12 and −1 (render test). Frame: frame 0 at another animation counter (live test). Depth: one item with *U*, a reader, its anchor the composite's last row; the barrel shares the body's depth on every row by construction (one item). Invisible: by construction. The original's runs are the lead's. |
| Scenario wording, rocking freshness ("no longer" → "not") | noted | Wording only. |
| Open questions, *Second condition of the tall-image split* (new) | noted; seam | See the *Split of a tall image* row. |
| Changes for other specs, S-0102 claims 13, 14 and 22 (Q-1885) | noted, not taken in | The S-0102 revision has not landed, so this intake changes no S-0102 row, except the one S-0105 claim 12 now decides directly: the turret-flag sprite vehicle's image takes *U*. The non-turret sprite path is unchanged. Separate seam, not this intake's: it uses *U* − 2 at every height, while S-0102 claim 13 says −*L(z)* − 2 when *h* ≠ 0. |
| "Questions for the project lead" renamed "Decided" | noted | Heading only. |
| Q-1885 | answered | The intake 19 interim is retired: the sprite row *U* − 2 on each part, the voxel-turret barrel frame, and separate items. |

## Requests

None. Block Q-1935 to Q-1939 is unused. The one case r39 leaves open, the second condition of
the tall-image split, has a sound choice (never split; S-0102 owns the split). It is stated above
as our design.

## Checks

Before is main 5c34005, the base of this branch; after is this branch. The branch touches only
`crates/render/src/voxel.rs`, `crates/app/src/mapview` and the lane test.
`git diff main -- crates/deterministic crates/sim-gate` is empty. `glasnost-sim-gate` depends on
neither `glasnost-app` nor `glasnost-render`. The arena and oracle modules of `glasnost-tools`
use neither. So the sweeps and the arena run the same code before and after, and the after runs
give the numbers intake 19 recorded. No goldens regenerated.
- Oracle arena (local lane; it fails while the known discrepancies stand):
  - M-0001: 31 of 96 pairings failing; 151 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M6c, M7a and M7b sweeps 1 to 16 pass. The smallest minima are:
  - M6c 2 (seed 3 charger orders, seed 12 jumpjet orders);
  - M7a 1 (seed 7 psychic sensor orders, seed 9 takeover orders);
  - M7b drain 3.

Lane scene `sprite-barrel` (`crates/app/tests/m7b_lane.rs`; counts, cells and ticks only;
outputs in `local-lane/m8/a3-27/`, git-ignored):
- Turret-flag vehicle types: 18. Sprite-body types with a barrel part: 0. Sprite-body types
  without one: 0. Voxel-body types with a barrel part: 7, of which 0 have a sprite file and 0
  are IFV class (the IFV class has numbered barrels, not a single barrel part).
- So the scene follows MS-33 part k's way in the lane's own copy of the import. It clears the
  first such type's voxel body and borrows the sprite file of a sprite-body vehicle type.
- The vehicle is placed north, west, south and east of a 2 × 2 building. Its turret is turned
  through the four 4-step facings, one screen each (`sprite-barrel-building-0` to `-3`).
- Then it is driven onto a high bridge's deck on map 0, reaching the deck's middle at tick 116,
  at height 416 above the ground. It is turned again there (`sprite-barrel-bridge-0` to `-3`).
- By eye:
  - Behind the building, the barrel and body are hidden together.
  - In front, the barrel is drawn over the body at every facing. At facing 1 it is also drawn
    over the sprite turret.
  - On the deck, the barrel stands lifted with the body at every facing.
