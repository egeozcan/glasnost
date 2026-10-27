# S-0314 r77: claim ledger, M8 step A3 intake 18

This intake takes in one spec revision, which carries one of the lead's rulings of 2026-10-24
([m8-plan.md](../m8-plan.md), *Lead rulings*).

- **S-0314 revision 77** (spec commit 0f8d429, merged in 567fb3c). It covers r74 to r77 against r73,
  which intake 6 built (`git diff 4862549 0f8d429 -- spec/combat/S-0314-healing-repair.md`).
  - One behaviour is added: the switch `quirks.docked_hover_water_loss`. On (the default), a hovering
    vehicle docked at a naval yard on water is lost as before. Off, a spin end that comes while the
    vehicle has depot contact or is settled makes no water test (claim 21, *Over water, at a naval
    yard*; the new Edge case *Hovering vehicle lost at a naval yard*).
  - The Overview, *Enabling while settled*, the water-cell exception of *Disabling while docked*,
    and Changes for other specs (S-0503 claims 10 and 11) now name the switch. Two passages are
    only rewrapped, and the *Questions for the project lead* section is gone.
  - No Import mapping row, formula, constant or scenario changes. H4 part i run 6 now says that it
    tests the default loss.

**Against the ruling.** The ruling says: "a docked hover unit lost at a naval yard gets
`quirks.docked_hover_water_loss`, default the original behaviour." r77 matches it. The switch is
named in claim 21 and in the Edge cases, and its default is the original's loss. The behaviour with
the switch off is marked as our engine's rule (`unmeasured`, design, confidence medium).

The switch was already on main (14a0149). It is now consulted, and its doc comment (in `quirks.rs`
and the ruleset schema) no longer says "Not consulted yet".

**Code.**
- `hover::sink_test` (S-0503 claim 11's water test, called by `hover::spin` at a spin's end): with
  the switch off, it returns before the water and height test when the vehicle has depot contact
  (`depot::link`, the vehicle's own contact with a depot) or is settled (`depot::settled`). The test
  is judged at the spin's end, so a spin that ends after the contact and being settled have ended is
  tested as usual. The two draws, the spin count and the rotation are untouched, so the random
  stream is the same in both positions.
- With the switch on, nothing changes. No new state: save format 78, AI game format 40 (taken by intake 15 part 1).

**Tests.** `crates/deterministic/sim/tests/claims_s0314_naval_hover.rs`, new. It uses a synthetic
lake and these synthetic types:
- SKIMMER: a hover of the plain amphibious zone, with the powered-unit flag;
- NAVYARD: a 2 × 2 naval vehicle factory with the repair flag, standing on the water, its docking
  cell (2, 1) a water cell;
- PLANT and CENTRE: the power plant and SKIMMER's control centre.

**Goldens, arena and sweeps.** These were run twice: first with *before* = main 14a0149 and *after* =
this branch, then again after the rebase, with *before* = main ea5c2bb (intake 15 part 1) and
*after* = the rebased branch. Main a32dbdc changes no simulation code against ea5c2bb, so the
second run holds for it too. Both runs give the same answer: with the switch on (the default) the
change is byte-identical.
- Goldens: unchanged. `cargo xtask gate` passes on every golden, none regenerated.
- Oracle arena (local lane): the CSVs are byte-identical before and after in both runs. On
  ea5c2bb:
  - M-0001: 31 of 96 pairings failing; 151 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M6c, M7a and M7b sweeps 1 to 16: every counter has the same minimum, median and maximum before
  and after, in both runs, and intake 15 part 1 changed none of them.
  - M6c's smallest minimum is 2 (jumpjet orders 2/51, charger orders 2/18).
  - M7a's smallest minimum is 1 (takeover orders 1/6).
  - M7b's smallest drain minimum is 3 (drain orders 3/3).

| Claim or item | Status | Notes |
|---|---|---|
| Overview, the switch named | followed | — |
| Claim 21, *Over water, at a naval yard*, the default loss (unchanged since r73) | tested (new) | `claim_21_docked_hover_lost_at_a_naval_yard_by_default`, end to end. A person's click docks SKIMMER at NAVYARD on water and it is repaired. Then come a disabling, an enabling (it rises) and a disabling above the water (S-0405 claim 20, by the control centre). That last disabling takes the two draws and starts a spin, and the spin's end gives the bomb mark. SKIMMER falls and is lost, and the yard holds no contact. This closes intake 6's "not tested at a naval yard". |
| Claim 21, a building leaves its cells' land type unchanged | tested | `docked` asserts that SKIMMER at the yard's docking cell stands on a water cell. |
| Claim 21, with the switch off: no water test while in contact or settled | built, tested (new) | `sink_test`. `quirk_docked_hover_water_loss_off_spares_the_docked_hover` uses the same scene as the default test. The same draws are taken. At the spin's end the vehicle is in contact or settled, it gets no bomb mark and does not fall, it rests on the water and stays in contact with the yard, it survives after the enabling, and its state survives a save and load. |
| Claim 21, off, contact alone (Terms: contact can exist before the building is reached) | tested (new) | `quirk_off_contact_without_settled_is_spared` (both positions): contact by `depot::request`, not settled, a spin ending 50 leptons above the open water. |
| Claim 21, off, settled alone (contact ended, claim 22) | tested (new) | `quirk_off_settled_without_contact_is_spared` (both positions). |
| Claim 21, off, "once its contact has ended and it is not settled, a later spin end is tested as usual" | tested (new) | `quirk_off_settled_ended_before_the_spin_end_is_tested` (by the test hook): settled when the spin starts, no longer settled at its end, so it is marked. In play: `quirk_off_a_spin_outliving_the_disabling_is_tested`. A control-centre loss over open water disables SKIMMER and starts a spin, and the centre is powered on again before the spin ends. The enabling ends being settled, so the spin's end is tested and marked. |
| Edge cases, "everywhere else S-0503 claim 11's water test is unchanged" | tested (new); Q-1876 answered, see [intake30-S-0314.md](intake30-S-0314.md) | `quirk_off_open_water_still_sinks` uses the test hook `econ::power::apply_disabled`, which settles nothing: no contact, not settled, switch off, so it is marked and falls. In play every switch-off of a hover is a depot's or a disabling's, and both leave it settled. So with the switch off, under the Q-1876 interim, only a spin that outlives being settled is tested (`quirk_off_a_spin_outliving_the_disabling_is_tested`). The S-0503 tests are unchanged and pass. |
| Claim 21, a switch-off that takes no draws starts no spin and makes no test | tested (new) | `no_draw_switch_off_makes_no_test_in_either_position` (*sleep*, both positions). |
| Claim 21, whether an ordinary repair visit reaches the loss (open in the spec) | followed, no request | The code keeps to the stated rule. In the end-to-end scene the ordinary visit starts no spin: the arrival's switch-off comes in *enter* and the later ones in *sleep* (`docked` asserts it). The spec leaves the behaviour during those ticks open (H4 part i run 1), and the code does not need the answer, so no request is filed. |
| Terms, being settled from a disabling, read with the switch off | interim, Q-1876 (retired at intake 30: r88 answers it, a hover disabled away from any depot is tested in both positions; the test below was replaced, see [intake30-S-0314.md](intake30-S-0314.md)) | `depot::disabled` makes every disabled vehicle settled, as intake 6 built from the Terms list. So with the switch off, a hover disabled by the loss of its control centre over open water, far from any depot, is spared too: `q1876_interim_disabled_far_from_a_depot` (on: lost; off: rests). Q-1876 asks whether the Terms entry, or the switch-off rule, covers a vehicle with no depot contact. |
| Claim 21, *Enabling while settled*, "by default loses the vehicle (unless … off)" | tested | In the end-to-end scene's enabling, SKIMMER is switched on and rises (`spin_at_the_yard`). The loss in each position, from the second disabling's spin, is covered by the two end-to-end tests. A later settling switch-off that takes the draws after an enabling is not set up on its own; it goes through the same `sink_test`. |
| Claim 21, *Disabling while docked*, the water-cell exception names the switch | tested | As above: in our scene the second disabling takes the draws (it is above the water), then loses the vehicle (on) or keeps it serviced (off). |
| Claim 21, *After the settling step* (rewrapped only) | no change | — |
| Edge cases, *Hovering vehicle lost at a naval yard* | built, tested (new) | The rows above. |
| Changes for other specs, S-0503 claim 10 ("ends by default in the vehicle's loss") | followed | `sink_test`. |
| Changes for other specs, S-0503 claim 11 (the added sentence with the switch off) | built, tested (new) | `sink_test`; the rows above. S-0503 itself is not revised yet. |
| Scenario H4 part i run 6 (now: tests the default loss at a naval yard) | tested on synthetic data | `claim_21_docked_hover_lost_at_a_naval_yard_by_default` follows run 6's shape: a disabling at a naval yard while the vehicle is above the water, with `no_draw_switch_off_makes_no_test_in_either_position` as the no-draw control. The land-depot control is S-0503's own over-land test. The stock run uses a named test type and is the lead's to record. |
| *Questions for the project lead* (removed; answered by the ruling) | followed | — |
| Intake 6's rows "21, the hover switch-off …: not tested at a naval yard", "H4 part i run 6: not implemented as a test" and the seam "The naval-yard water spin at a switch-off is not tested" | updated | Now tested (above). Updated in [intake6-S-0318-S-0314-S-0108-S-0701.md](intake6-S-0318-S-0314-S-0108-S-0701.md). |

## Requests

Q-1876 was filed (block Q-1876 to Q-1879). It asks whether a disabling makes a vehicle with no depot
contact settled, which decides, with the switch off, whether a hover disabled over open water far
from any depot is spared. Interim: the built Terms reading, so it is spared (`sink_test`,
`depot::disabled`). Q-1877 to Q-1879 are unused. S-0314 r88 answered Q-1876 and intake 30 retired the interim.
