# S-0411 r57 and S-0111 r57: claim ledger, M8 step A3 intake 115

- **S-0411** r49 to r57 (commit 15a3cc9, from the r49 that intake 78 built, d87d8d6). r57 answers
  Q-2791 (the tips of the sidebar's build icons). It adds claims 20 to 22 (when an icon tip shows,
  its text, place and look, and the tips of the other controls), the Terms *Sidebar tip*, *Slot*,
  *Tip area* and *Insertion point*, two Constants (the icon-tip overlap 5 and the wrap margin 8),
  the State and ordering rows, five Edge cases (all display-only or departures, none with a
  switch), the Import mapping of the string-table entries, scenario S16, Open questions 10 and 11
  and the Changes for S-0111. No quirk is added.
- **S-0111** r49 to r57 (commit 2a7f785, from the r49 that intake 111 built, 05ade81). r57 adds
  claim 20's exception over the build-icon slots (S-0411 claim 20 applies there), says the name
  tip is not shown over the sidebar because the sidebar's own tips are S-0411's, restates the
  state table's exception, adds two Edge cases (*A name tip back under a resting pointer* and *The
  movement that leaves the build-icon slots*, both display-only, no switch), and widens NT1's
  falsifier for the first. No quirk is added.

App only: `crates/app/src/game/{name_tip, sidebar_tip, session}.rs` and `crates/ui/src/hud.rs`.
The simulation is untouched; the session reads the price (`econ::production::price`, S-0404
claim 2's *P*) and the power totals (`econ::power::totals`) through functions that existed.

Request filed: [Q-2831](../../spec/requests/Q-2831-sidebar-tip-entries-with-other-insertion-points.md)
(an entry whose insertion points differ from Import mapping's, S-0411 Open question 11), with the
interim in `sidebar_tip::fill`. Block Q-2832 to Q-2840 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**; **design**
for a choice the spec leaves to the implementer.

## Built

### One tip state for the name tip and the sidebar tips (S-0411 claim 20, S-0111 claim 20)

- `name_tip::NameTip` keeps the rest, the shown tip and two new fields:
  - `armed`: a drawn frame has passed with the pointer on a slot (set in each `update` from the
    pointer's place, cleared by a movement that ends off the slots);
  - `at_once`: the time of a movement over the slots that shows its tip at once.
- `moved(now, on_slot)`: armed and on a slot, the movement hides the tip and marks it to show at
  once; any other movement is S-0111 claim 20's ordinary one (hide, wait again).
- `update(now, on_slot, blocked, choose)`: an `at_once` tip shows in that frame with its lifetime
  from the movement, or nothing while blocked; the rest is the name tip's wait, lifetime and
  choice in every drawn frame, unchanged.
- `button()` and `off()` also clear `at_once`; `armed` stays, so after a click the next movement
  over the slots shows the tip at once.
- `Session::update_name_tip` picks the blocks by the pointer's place (`TipPlace`): over the sidebar
  only the order block (the hidden cursor); over the view the moving view, the held view button and
  the order block as before.

### Places, texts and boxes (`game::sidebar_tip`)

- `place_at`: a shown slot of the strip (`hud::Layout::slot_at`, tiles of the icon cell plus its
  gaps, so a gap between icons is on the slots), a control of claim 22's table, our power-mode
  button, or nothing (radar, credits, shell, gaps between controls).
- `fill`: Terms' insertion points (`%s` the next text, `%d` the next number, decimal, a minus sign,
  no grouping), with Q-2831's interim for other layouts.
- `icon_text`: the money-format entry filled with the display name and *P*, every space a line feed;
  a missing entry shows the placeholder alone. A superweapon icon shows its display name alone,
  spaces kept (`Session::icon_text`, `Kind::Superweapons`).
- `power_text`: the power entry with *O* and *D*; `with_disabled`: the disabled line of a tab
  without icons and of the scroll buttons while all icons fit.
- `slot_place`: claim 21's *l* and *m*; `wrap_width`: *A*<sub>w</sub> − 8. A control's tip is
  placed by `name_tip::place` at the pointer with the tip area's right and bottom edges.
- `name_tip::make` lays out and sizes a text as claim 24 gives and places it by a closure;
  `Shown::clip` is the view for a name tip and the sidebar for a sidebar tip, and `draw` cuts at it.

### The old sidebar tooltip removed (`hud.rs`)

- The interim tooltip of the hovered icon (name and price or time left), the tab and mode buttons'
  hover names and the power bar's output/drain tooltip (`hud::tooltip`, `tooltip_lines`,
  `power_lines`, `CameoView::seconds`, `HudModel::focus_cameo`) are gone; claims 20 to 22 replace
  them. The superweapon icon's tip now shows its name only, with no time left (claim 20).
- The keyboard focus on an icon keeps a tip (design, below).

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0411 20: the option off, no sidebar tip | tested | `client::sidebar_tip_tests::claim_20_buttons_and_blocks` (last step) |
| 20: the movement onto the slots waits 1000 ms; the tip after the wait, lifetime from then | tested | `claims_20_and_21_an_icon_tip`; `name_tip::tests::s0411_claim_20_over_the_slots_no_wait` |
| 20: after one drawn frame on a slot, a movement ending on a slot shows its tip at once | tested | the same two tests |
| 20: a second movement before a drawn frame has passed is ordinary | tested | `claim_20_two_movements_in_one_frame`; `s0411_claim_20_one_drawn_frame_first` |
| 20: a movement within one icon keeps the tip, the lifetime started again | tested | `claims_20_and_21_an_icon_tip` |
| 20: a movement onto another icon shows its tip at once; onto an empty slot hides it | tested | the same test (whichever the synthetic strip holds in slot 1); `s0411_claim_20_over_the_slots_no_wait` for both |
| 20: a movement ending off the slots is ordinary; the tip of the place where it rests | tested | `claims_20_and_21_an_icon_tip` (onto the view); `claim_22_the_controls` (a control after the wait) |
| 20: Lifetime 10000 ms; the next movement over the slots shows it again at once | tested | `claims_20_and_21_an_icon_tip` |
| 20: a press or release hides it; the next movement over the slots shows it at once | tested | `claim_20_buttons_and_blocks` (middle and right buttons); `s0411_claim_20_buttons_and_blocks` |
| 20: a held button does not stop a movement over the slots | tested | `claim_20_buttons_and_blocks` (right button held) |
| 20: the order block (hidden cursor) blocks sidebar tips; its end brings nothing back | tested | `claim_20_buttons_and_blocks` |
| 20: a moving battlefield view blocks no sidebar tip | tested | `claim_20_buttons_and_blocks` |
| 20: the options menu hides it and stops its wait | tested | `claim_20_buttons_and_blocks` |
| 20: Text: the money format, display name and *P*, every space a line feed | tested | `sidebar_tip::tests::claim_20_icon_text`; `claims_20_and_21_an_icon_tip` (synthetic table) |
| 20: *P* in its form in parts for a building whose cost pays for units that come with it | partly | read through `econ::production::price`, S-0404 claim 2's own function; no synthetic type here pays for a unit that comes with it |
| 20: a superweapon icon: its display name alone, spaces kept | partly | `Session::icon_text`; the synthetic house has no superweapon icon, so no test reaches it |
| 20: an empty slot has no text and no tip | tested | `claim_20_chosen_again` (the slot emptied); `claims_20_and_21_an_icon_tip` when slot 1 is empty |
| 20: every state the same (darkened, building, on hold, done, placing, superweapon states) | partly | the text reads only the type and the price, never the item's state; no test sets each state |
| 20: chosen again in every drawn frame: another text replaces it, lifetime kept, placed again | tested | `claim_20_chosen_again` |
| 20: the slot becomes empty: hidden, nothing until the pointer moves | tested | `claim_20_chosen_again` |
| 20: the price follows a cost bonus gained or lost | partly | `price` is read in every drawn frame; no test changes a bonus |
| S-0411 21: laid out, sized and drawn as S-0111 claims 24 and 25, in the tip colour | tested | `name_tip::make` as `tip` uses it; `claims_24_and_25_the_drawn_tip` |
| 21: wrap width *A*<sub>w</sub> − 8 | tested | `sidebar_tip::tests::claim_21_slot_place` (the width); `claim_20_chosen_again` (a long text) |
| 21: placed by the slot: *l*₀, *l*₁, *l*, *m*₀, *m* | tested | `claim_21_slot_place` (all four branches and both corrections); `claims_20_and_21_an_icon_tip`, `claim_21_at_another_scale` |
| 21: no correction at the area's bottom edge | tested | `claim_21_slot_place` |
| 21: drawn over the sidebar, cut at its edges, never over the view | tested | `name_tip::tests::s0411_claim_21_cut_at_the_sidebar`; `Shown::clip` checked in the client tests |
| 21: the tip area (Terms) | design | our layout's matching part: the sidebar's width from the row below the radar panel's frame to the bar's bottom (`hud::Layout::tip_area`), at the interface scale, inside the screen sidebar. It holds the credits, power meter and mode buttons too, as they lie below our radar |
| 21: the slot's rectangle | design | the icon's cell (`Layout::cameo`), in screen pixels |
| S-0411 22: the controls' timing: S-0111 claim 20's wait, lifetime, press and release, with the hidden cursor and the options menu as blocks | tested | `claim_22_the_controls` (the wait); the timing is the shared `NameTip`'s |
| 22: placed by the pointer as S-0111 claim 24, with the tip area | tested | `claim_22_the_controls` (the power meter) |
| 22: the spaces kept | tested | `claim_22_the_controls` |
| 22: the tab labels; the disabled line for a tab without icons | tested | `claim_22_the_controls`; `sidebar_tip::tests::claim_22_disabled_line` |
| 22: the scroll labels; the disabled line while all icons fit | tested | `claim_22_the_controls` (scroll-up) |
| 22: the repair-mode, sell-mode and options labels | tested | `claim_22_the_controls` |
| 22: the power meter: *O* and *D* | tested | `claim_22_the_controls` |
| 22: the diplomacy button | design | our sidebar has none, so it has no tip |
| 22: our power-mode button | design | its tip is "Power on/off", the name our earlier hover showed (`sidebar_tip::POWER_MODE_TIP`) |
| 22: nothing elsewhere on the sidebar (radar, credits, shell, gaps) | tested | `claim_22_the_controls` (radar, credits); `sidebar_tip::tests::claim_22_places` |
| Terms: insertion points | tested | `sidebar_tip::tests::terms_insertion_points` |
| Import mapping: the entries and labels; a missing entry shows the placeholder | tested | `claim_22_the_controls` (sell label, power entry); `claim_20_icon_text` (money format) |
| Open question 11: other insertion-point layouts | seam | Q-2831 filed; interim in `sidebar_tip::fill` (`design: Q-2831`) |
| S-0111 r57 20: the exception over the build-icon slots | tested | the S-0411 claim 20 rows above |
| S-0111 20: over the sidebar no name tip, its tips being S-0411's | tested | `claim_22_the_controls` (radar, credits: nothing) |
| S-0111 Edge cases: a name tip back under a resting pointer | no change | display-only, no switch: ours shows nothing until the pointer moves and rests again, as before |
| S-0111 Edge cases: the movement that leaves the build-icon slots | tested | `claims_20_and_21_an_icon_tip` (ours waits) |
| S-0411 Edge cases: icon tips of a full tab (75 or 76 icons) | no change | ours shows them, by the same path as any tab |
| S-0411 Edge cases: the movement that leaves the slots | tested | as the S-0111 row |
| S-0411 Edge cases: a slot filled again under a resting pointer | tested | `claim_20_chosen_again` (nothing until the pointer moves) |
| S-0411 Edge cases: icon-tip text of 66 text units or more | tested | `name_tip::cut` through `make`, as the name tip (`claim_25_look_and_the_text_limit`) |
| S-0411 Edge cases: boxes placed with a smaller size | tested | ours places each box with the size it draws (`claim_21_slot_place`) |
| design: the keyboard focus on an icon | design | while the focus shows on a shown icon, no pointer tip shows and no order block holds, that icon's tip shows, placed by its slot (`Session::choose_focus_tip`); `design_the_keyboard_focus_tip` |
| design: watch mode | design | the watch panel in the sidebar's place shows no sidebar tip |
| design: no string table (a game built without an install) | design | stand-ins "%s %d" and "%d %d" |
