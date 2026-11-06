# S-0015 and S-0111 r40: claim ledger, M8 step A3 intake 109

- **S-0015** r4 (commit 72e6747, new): the string table file layout and lookup results. It answers
  Q-2686: the file (`ra2md.csf`, claim 1), the header and its refusals (claim 2), the entries
  (claim 3), the decoding (claim 4), the entry labels (claim 5), several texts or none (claim 6),
  the extra value (claim 7), the missing-label placeholder (claim 8) and a table that cannot be
  decoded (claim 9). It has no quirk and no switch.
- **S-0111** r34 to r40 (commit a13d785, from the r34 that round 8b built). r40 answers Q-2687, the
  name tip's open points: claims 20, 24, 25 and 26, the state table, the Edge cases and scenarios
  NT1, NT2, NT6, NT7 and NT8. Every departure it names is display-only with no switch, so no
  quirk is owed (`every_spec_quirk_has_a_switch` and `every_rendering_spec_quirk_has_a_switch`
  were not changed and pass); [c7-play-fixes.md](c7-play-fixes.md) holds no row for it.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

Request filed: **Q-2736** (a left or right button pressed in the route mode and held: S-0111's Open
questions leave it undecided; the interim is that it blocks like any held button, marked
`design: Q-2736` in `Session::pointer_down`). Q-2737 to Q-2740 unused. Nothing else in either delta
is unclear or contradicts another spec.

## S-0015

| Claim | Status | Where and how |
|---|---|---|
| 1 which file | partly | `strings::FILE`, `strings::read` resolves `ra2md.csf` through the vfs (S-0005 claim 9); `ra2.csf` is never asked for. `install::prepare` reads it once when the program starts, and a table that cannot be resolved stops the install from preparing (the menu shows "the program cannot start"; S-0005 claim 16). Lane: `i109_the_install_string_table_reads`. Seam: it is resolved after the whole startup mount (`prepare` runs after `Vfs::mount_startup`), not "while only the two language archives are mounted"; the language archives rank first in the search order, so the result differs only when no loose file and no language archive holds `ra2md.csf` but a later archive does: the original then refuses to start and ours starts. The refusal has no unit test of its own (`prepare` needs a mounted install; the lane covers the read). |
| 2 header | tested | `csf::StringFile::read`: refusals for a short file, other identifying bytes, a count of 0 or below; version, unused value and language code ignored. `s0015_header_checks`. The refusal reaches the start-up through `strings::read` and `install::prepare` (see claim 1). |
| 3 entries | tested | `read_entry`: entry and text identifying bytes, signed counts, plain and extended texts, one to three trailing bytes ignored. `s0015_reads_entries`. |
| 4 decoding | tested | complement `65535 − stored`, ends before the first stored 0 or 65535, unit count 0 gives empty text; the space normalisation runs at lookup (`strings::normalised`, the same result as once at reading). `s0015_reads_entries`, `claim_17_normalisation`. |
| 5 labels | tested | cut before a zero byte (`s0015_reads_entries`); 32 bytes or more match nothing and warn (`StringTable::new`, `s0015_placeholder_and_reading`); case-only and equal labels: the first in the file, with a warning (`claim_17_matching`). |
| 6 several texts or none | tested | first text kept, later ones read and dropped; an entry with no texts gives empty text, not the placeholder (`s0015_placeholder_and_reading`, `claim_21_which_objects_get_a_tip` for an empty own name). |
| 7 extra value | tested | read and discarded (`s0015_reads_entries`). |
| 8 placeholder | tested | `strings::placeholder`: marker word, colon, apostrophe, the label's bytes as units, apostrophe, spaces kept; `resolve` uses it for a missing entry. `s0015_placeholder_and_reading`, `claim_17_labels_and_resolving`, `claim_21_which_objects_get_a_tip`. |
| 9 damaged table | tested | the reading stops at the first damaged entry (cases i to iv), at the header's entry count and at the header's text count, keeps the entries read whole, and warns with the byte; fewer than the header's counts warn. `s0015_damage_keeps_whole_entries`, `s0015_placeholder_and_reading`. The warnings go to the log once, when the program starts (they carry labels, so a lane never prints them). |

State and ordering: the table is read once, at start-up (`install::prepare`), kept on the `Install`
and handed to each game by the client (`Done::Started`); no game or network start reads it again.

Seam: only the name tip and the fixed labels resolve through the table. The sidebar's icon names and
the superweapon countdown rows (S-0512 claim 18) still show the type's name, as before; their
specs' display-name routing is not part of this delta.

## S-0111 r40

| Claim | Status | Where and how |
|---|---|---|
| 20 wait: any movement over the window, the window's edge not an event | tested | `Session::pointer_left` no longer stops the wait; the wait that ends with the pointer outside shows nothing and is used up. `claim_20_the_pointer_leaving_the_window`. |
| 20 where the pointer rests | no change | `choose_name_tip` chooses only over the battlefield view (`hud_at == Map`); the message lines are no target. `claim_21_which_objects_get_a_tip` and round 8b's tests. |
| 20 lifetime not started again | tested | `NameTip::update` keeps `until` when a text is chosen again. `claim_26_chosen_again`, `claim_26_the_text_is_chosen_again`. |
| 20 blocked: held left or right button pressed over the view | tested | `Session::view_button_held`, set by `pointer_down` over the view, cleared by `pointer_up` and `pointer_left`; a middle button blocks nothing. `claim_20_a_held_button_blocks`. |
| 20 blocked: hidden cursor, locked input | tested | an order block (`world.tick <= orders_blocked_through`, S-0201 claims 11 and 14) blocks. `claim_20_the_order_block`. Seam: the test names no other source of a lock, as the spec says none is known. |
| 20 blocked: moving view | no change | the view centre changed since the last frame (round 8b). |
| 20 options menu | tested | opening and closing the in-game options menu turn the tip's state off (`open_options`, `close_options`, `update_name_tip`); nothing shows until the pointer moves and rests again. `claim_20_the_options_menu`. |
| 24 margins 8 and 4, wrap width W − 8, wrapping at the last space, space dropped, no break at a leading space, a line break forgets earlier spaces, the first unit kept | tested | `name_tip::{EXTRA_W, EXTRA_H, WRAP_ALLOWANCE, lines, size}`. `claim_24_layout`. |
| 24 flip: last row on the pointer's row | tested | `name_tip::place` (`y − h + 1`). `claim_24_placement`. |
| 24 the box keeps the pointer position; placed again for a text chosen again | tested | the choice runs at the pointer's place, which does not change while the tip shows (a movement hides it). `claim_26_the_text_is_chosen_again`. |
| 25 text origin (4, 2) | tested | `name_tip::INSET`, `draw`. `claims_24_and_25_the_drawn_tip`. |
| 26 text chosen again in every drawn frame | tested | `NameTip::update` with `choose(Some(tip))`: no text or blocked hides it and nothing shows until a rest; another text replaces it; the same text leaves it. `claim_26_chosen_again`, `claim_26_the_text_is_chosen_again`. |
| Edge cases: the original's display bugs (bottom row, wider than the view, wrap after a line break, leading space, smaller text) | no change | the spec gives them as the original's results with our engine's rule; ours is the rule in each. |
| Hand off with S-0004 scenario L2 (margin 4) | no change | a measurement scenario of the lead's; nothing to build. |

Removed `// design:` interims for Q-2686 and Q-2687: the string table's decoding, the placeholder's
stand-in, the text inset, the wrap rule, the pointer leaving the window, the hidden cursor and locked
input line, and the options menu line.
