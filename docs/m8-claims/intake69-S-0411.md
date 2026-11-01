# S-0411 r28: claim ledger, M8 step A3 intake 69

This intake takes in one spec revision:

- **S-0411 revision 28** (sidebar behaviour): answers **Q-2238** (A4 round 7, note 41: whether
  the command and tab buttons take images from the install's data, and which).

Our engine built S-0411 r20. The diff from r20 to r28 was read in full (`git diff 2cd6884 926f5ae
-- spec/ui/S-0411-sidebar-behaviour.md`): the Summary's two sentences on the install's tab and
button images, claim 11's lit artwork, the new claim 19 with its frame table, *Mode buttons*,
*Mode lighting*, *Power-mode button* and *Layout*, five Import mapping rows, scenario S15, the Open
questions and the Questions for the project lead. No other claim changed.

**Client only.** No simulation code, rules, saved or hashed state changes: save format and AI game
format stay, and no sweep is due. **No quirk switch**: claim 19 calls the original's mode-lighting
difference a display bug without gameplay effect, so our engine uses its own rows (no C7 judgement
is due; `c7-play-fixes.md` is unchanged). No goldens or file formats change.

## What changed

- `game::sidebar_art` (new, `crates/app`): the eight file names and `SIDEBAR.PAL` of the Import
  mapping; `read` resolves each through the search order as it stands and decodes the frames claim
  19 uses on the file's canvas; `palette_rgba` turns the 768-byte palette into the painter's RGBA
  by (4·*c*) mod 256; `mount_side` mounts the local player's side group; `button_art` gives the
  sidebar model the frames the painter took.
- `Session::start`: after the game is assembled (its rules, map and art already read with the
  importer's side group), the side group of the local player's side is mounted (S-0005 claims 13
  and 13a: the country the local house plays, or registration index 0 for a watched game), then the
  images and palette are read. `register_art` registers the palette as a painter row and each frame
  as a sprite (`BUTTON_SPRITES` + file index); a frame the painter refuses counts as missing.
- `hud` (`crates/ui`): `tab_frame`, `mode_frame` and `scroll_frame` are claim 19's first table.
  `ButtonArt` carries each image's sprite, canvas and frames; `view` draws the image's frame for the
  button's state and falls back to our symbol (round 7's) for a missing file or frame, state by
  state. The power button keeps our symbol. Hit targets, tooltips and keyboard focus are unchanged.
  `HudModel::scroll_pressed` names the scroll button whose press goes on with the pointer over it
  (`Session::scroll_pressed`: the press began on it and the pointer is on it, exactly when a
  release pages, claim 18).
- `Layout::with_buttons` (design): a button with an image takes its file's canvas as its cell, so
  the image is drawn whole and unscaled; each row is centred in the sidebar, 4 pixels between cells,
  wrapping to more lines when too wide, a cell wider than the sidebar narrowed to it. The power
  button keeps 44 pixels, narrowed to what the row leaves, and takes a line of its own when fewer
  than 16 are left. `Session::layout` uses it for drawing, hits, the wheel and paging, so claim 18's
  row count *R* is the one shown. Without images every rectangle is the earlier layout's.

## Claims

| Item | Status | Tests / notes |
|---|---|---|
| Summary: drawing the install's images is optional; when drawn, claim 19 fixes file, frame and palette | built | our client draws them when the palette and a file resolve |
| Claim 11: the lit artwork is ours or the install's lit tab frames | built | lit frames 3 and 4 drawn when the tab images resolve; our look otherwise. `hud::tests::claim19_the_view_draws_each_buttons_frame`; `tab_flash_follows_its_start_tick` unchanged |
| Claim 19: one file per button, named in Import mapping; the tab files numbered in claim 1's tab order | tested | `sidebar_art::tests::the_button_images_are_read_with_the_frames_claim_19_uses` (synthetic install: a file's canvas and frames, a missing file, a third tab's fifth frame) |
| Claim 19: each file resolved by the ordinary precedence with the local player's side group mounted (S-0005 claims 13 and 13a) | tested | `sidebar_art::tests::the_images_come_from_the_local_players_side_group` (synthetic `sidec01.mix` and `sidec02.mix` with different sell images: side 0 the first, sides 1 and 2 the second, back to side 0 the first; a file only in the main archive found for every side) |
| Claim 19: every image drawn with the sidebar palette; the same file name for every side, the copy the ordinary precedence finds; S-0005 claim 22's discarding at each side-group mount | tested | `sidebar_art::read` resolves `SIDEBAR.PAL` after the side-group mount of each game; `Session::register_art` draws every frame with that row only (not the icon palette, not the 16-bit step). Same test as above |
| Import mapping: `SIDEBAR.PAL` 768 bytes, file byte *c* gives (4·*c*) mod 256 | tested | `sidebar_art::tests::the_sidebar_palette_converts_each_byte_by_four` (64 and 255 wrap; 767 and 769 bytes refused) |
| Import mapping: unresolved palette, every button our own symbol | tested | `the_button_images_are_read_with_the_frames_claim_19_uses` (no palette, and one of the wrong size: no images) |
| Claim 19, first table, tab rows (unavailable 2; lit and shown 4; lit 3; shown 1; else 0) | tested | `hud::tests::claim19_the_frame_tables` (every row and the order), `claim19_the_view_draws_each_buttons_frame` (shown, lit, lit and shown, unavailable in the drawn sidebar) |
| Claim 19, first table, sell and repair rows (mode active 1, else 0) | tested | same two tests; `play::tests::s0411_claim19_a_mode_buttons_click_ends_its_active_mode` (the model's frame through clicks) |
| Claim 19, first table, scroll rows (disabled 2, even pressed; press in progress over it 1; else 0) | tested | `claim19_the_frame_tables`, `claim19_the_view_draws_each_buttons_frame`; `client::play_notes_tests::a_scroll_buttons_press_frame_follows_the_press_over_it` (press on a button, off it, over the other, back over it, release pages; a release off it does not) |
| Claim 19: a held press has no frame of its own on a tab, sell or repair button; a resting pointer changes no frame | tested | `hud::tests::claim19_hover_changes_no_frame` (hover over every button: the same sprites and frames); the model has no press state for these buttons. `image` adds no hover or press styling over an image |
| Claim 19: a scroll button's frame while pressed with the pointer off it | design | frame 0 (the claim's engine design) |
| Claim 19, *Mode buttons*: a click on the active mode's button ends it, as a click while inactive enters it (S-0214 claim 13) | tested | `s0411_claim19_a_mode_buttons_click_ends_its_active_mode` (`Play::mode_click` already toggled) |
| Claim 19, *Mode buttons*: a click with no building owned or during placement | not claimed | no change |
| Claim 19, *Mode lighting*: our rows, frame 1 whenever the mode is active however it was entered | tested | `s0411_claim19_a_mode_buttons_click_ends_its_active_mode` (a mode set directly, as another way of entering it would, shows frame 1). The original's rows are a display bug: no quirk switch |
| Claim 19, *Power-mode button*: our design, no install image | built | `ButtonArt::mode(Mode::Power)` is `None`; the round 7 symbol stays (`claim19_the_view_draws_each_buttons_frame`: no sprite for it) |
| Claim 19, *Layout*: positions, spacing, cells and the image's fit are design; a missing file or frame draws our symbol for that button | tested | `claim19_layout_without_images_is_our_earlier_layout` (every rectangle of the earlier layout at three window sizes), `claim19_layout_with_images_takes_their_canvases` (synthetic canvases from small to wider than the sidebar: inside the bar, no overlap, wrapping, the power button's room, whole grid rows), `claim19_a_missing_frame_falls_back_to_the_symbol` (a missing frame gives the symbol for that state only; no images gives no sprite) |
| Claim 19: whether the pictures are kept for the session | not claimed | our engine reads them at each game's start (design), which also follows S-0005 claim 22 for the palette |
| Scenario S15 part a (files, frames, palette per side) | local lane | `crates/app/tests/intake69_lane.rs` `s15_a_the_button_files_per_side` (output not tracked) |
| Scenario S15 part b (frames per state in play) | local lane | `s15_b_the_sidebar_buttons_in_play`: our engine's frames per state, shot for each side; a comparison with the original is the lead's record |

## Requests

- [Q-2266](../../spec/requests/Q-2266-the-sidebar-button-images-transparent-index-and-display-colour.md):
  whether palette index 0 of these images is transparent, whether other indices are transparent or
  shadow, and whether their colours go through S-0103 claim 9's 16-bit step. Interim: index 0
  transparent and the rest opaque, as for every sprite our painter draws; the converted colour at
  full 8-bit precision.

Q-2267 to Q-2270 are unused.

## Seams

- **Icons from the importer's side group.** `glasnost_convert::game::load` reads the rules and art,
  the sidebar icons among them, with the side group of the first country that has a side (it is
  called with no country). Only the button images are read after the local player's side group is
  mounted. A local player of another side group sees icons found through the first group where a
  side archive holds its own copy. This was already so before this intake; S-0411 claim 19 does not
  cover icons.
- **Files read after the start see the local side group.** The game's voice and sound files read
  after `Session::start` (the client's start job) now resolve with the local player's side group
  mounted, as S-0005 claim 13 gives, not the importer's.
- **Before shots with the scroll buttons disabled only.** The before lane predates the short-window
  shots; the enabled and pressed scroll buttons were shot after only (their look before was round
  7's plain buttons).

## Lanes

Local lane `crates/app/tests/intake69_lane.rs` (ignored; needs a real install). Before: the lane
run before the client changes (round 7's symbols); after: this branch. Shots under
`local-lane/m8/intake69/before` and `after` (not tracked), the sidebar column enlarged three times:
start, deployed, sell and repair on, off, hovered and held, each tab shown, a lit tab shown and
not shown, and the scroll buttons enabled, hovered, held, and held with the pointer off them in a
short window.

| Lane | Result |
|---|---|
| S15 part a, each side | the palette and all eight files resolve for each side; sides 1 and 2 share their images (S-0005 claim 13a) |
| S15 part b, after | each side's sidebar draws the install's images; frames per state as the table gives; the power button our symbol |

## Checks

Rebased onto 7f7ce87 (main with intake 67); one conflict in note 40's row of
`a4-play-notes.md`, resolved by keeping main's row 40 and this intake's row 41. On the rebased
tree:

- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`: clean.
- `cargo xtask check`: deps, determinism, cite and canary ok; stock-guard 105 problems, all in
  `measurements/`, as on main.
- `cargo test -p glasnost-ui --lib hud`: 19 passed; the new `glasnost-app` tests (`sidebar_art`,
  `s0411_claim19_…`, `a_scroll_buttons_press_frame_…`): 5 passed.
- Not run on the rebased tree: `cargo test --release --workspace` (stopped by the host for low
  memory, not a failure), `cargo xtask gate` and `cargo xtask web --release` (left out while memory
  stayed short). No simulation code changed, so no sweep is due.
