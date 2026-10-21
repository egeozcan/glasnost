# S-0202 revision 33 claim ledger, M8 step A3 intake 1

S-0202 revision 32 to 33 (spec commit e28d1b9, answering Q-1562): new claim 17 (how a particle type's
colour list is read), its quirk `quirks.colour_list_item_triples`, the `ColorList` row of the Import
mapping, scenario R16, and digit grouping in the Formulas table of P and Q and in R2 and R9 (no behaviour change).

Code: `glasnost_convert::sim` (`colour_list`, `colour_list_triples`, `colour_list_groups`,
`colour_component`, the importer warning in `Builder::particle`; the switch is
`SimQuirks::colour_list_item_triples`, listed in `rules::quirks::EXTERNAL_QUIRKS`). The reading
happens in the importer only: the simulation reads the entries it is given (`color_count` for claim
15's gate, `color_list` for S-0616 claim 29 and F16), so no simulation state, save format or golden
changes. Tests: `crates/convert/src/sim_tests.rs` (`colour_list_item_triples_table`,
`colour_list_components_decimal_modulo_256`, `colour_list_511_byte_cut`,
`colour_list_parenthesis_aware_reading`, `colour_list_unclean_warning`,
`colour_list_switch_in_the_import`, and the older `particle_fields`, `particle_look_fields`) and
`crates/tools/tests/colour_list.rs` (`colour_draw_gate_reads_the_entries`,
`colour_steps_follow_the_reading`: imported text through `GasRules` to the colour draw and the colour
steps).

| claim | status | test names or reason |
|---|---|---|
| 17, the text | tested | The value is S-0003 claim 1's text with a 511-byte limit and the cut's trailing whitespace removed (`value::list`): `colour_list_511_byte_cut` (a cut ending in `(4,5,6 ` gives blue 0; a clean 511-byte list is not cut). |
| 17, items and entries | tested | Split at every comma, empty items dropped, none trimmed, parentheses ordinary bytes; consecutive threes, leftovers give nothing: `colour_list_item_triples_table` (every row of the claim's table). |
| 17, components | tested | Red without the first byte, green whole, blue without the last byte; S-0003 claim 3's decimal form only, claim 4's wrapping, modulo 256: `colour_list_components_decimal_modulo_256` (R16's `(1Fh,$20,-1)` and `(300,-56,4294967340)`), `particle_look_fields`. Our earlier reading used S-0003's full integer form, with the `$` and `h` hexadecimal forms; they are gone. |
| 17, empty-list case | tested | A reading with no entry is an empty list, so claim 15 takes no colour draw: `colour_list_switch_in_the_import`, `colour_draw_gate_reads_the_entries` (R16 part b's three lists, in our engine). |
| 17, importer warning | tested | (a) to (d) and the cut, the same findings with either setting, the result unchanged: `colour_list_unclean_warning` (`(1,2,00,4)` warns under (d) alone; clean lists, including ones with whitespace beside the numbers, do not warn), `colour_list_511_byte_cut`, `colour_list_switch_in_the_import`. The cut warns when a byte past the 511th is not whitespace, as S-0003 claim 1's cut warning does elsewhere (`decimal_prefix_list_notes`). |
| Edge cases, `quirks.colour_list_item_triples` | tested | On (default, the original): claim 17's reading. Off (our own): groups from an opening parenthesis to the first closing one, a group with another opening parenthesis inside gives nothing, text outside groups and an unclosed final group give nothing, empty parts kept, a missing or digitless part 0, parts after the third ignored: `colour_list_parenthesis_aware_reading`, `colour_list_switch_in_the_import`, both tests of `crates/tools/tests/colour_list.rs`. Owner: the importer (`EXTERNAL_QUIRKS`; `every_spec_quirk_has_a_switch`). |
| Import mapping, `ColorList` | tested | Read by claim 17 with the switch; empty when omitted or empty in a later layer (the last section read decides, as before): `particle_fields`, `colour_list_switch_in_the_import`. "A list with at least one entry enables the conditional starting-colour draw": `colour_draw_gate_reads_the_entries`. |
| 15, the colour draw's gate | tested | Unchanged in the simulation (`rules::particle_draw_rules`: `color_count` > 0 and a nonblack start colour; `claim_15_particle_groups` in `claims_s0202_middle_event.rs`); the count is now claim 17's entry count, with both settings: `colour_draw_gate_reads_the_entries`. |
| S-0616 claim 29 and F16, through claim 17 | tested | The colour steps walk claim 17's entries: `colour_steps_follow_the_reading` plays R16 part a in our engine (prefix, test text, filler and sentinel against the clean control, colour speed 2.0, one entry per update), with both settings. The spark or railgun short-list warning counts claim 17's entries: `colour_list_switch_in_the_import`. |
| R16 | not implemented (measurable) | An oracle scenario; no M-record yet. Our engine's side of parts a and b is in the tests above. Proposed for the oracle list (step A2). |
| Formulas, R2, R9 text | no change | Digit grouping only (`3 131 664 519`, `100 000`); the values are those the code already cites. |

Requests: Q-1562 answered (its interim rule, "a component that is missing or not an integer is 0", is
gone from `convert::sim` and the docs). None filed.
