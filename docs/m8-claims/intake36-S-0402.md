# S-0402 r76: claim ledger, M8 step A3 intake 36

This intake takes in one spec revision and answers the lead's round 5 notes 23, 24 and 30
(slave miners, the computer's miners, ore drills; [a4-play-notes.md](a4-play-notes.md)).

- **S-0402 revision 76** (spec commit 34dd574, with the Q-1870 answer 6020b95). The diff against
  the revision our engine built (8fcdefd) is mostly wording. The rule changes:
  - Claim 11, *Refinery sold while unloading*: the unloading harvester queues and starts
    *harvest* first, and only then gets the forced scatter request, only while it has no
    destination (refused while *unload* is still current). A harvester in *enter* loses a
    destination that names the refinery building; a cell destination, such as the docking cell,
    is kept with no request. The contact ends at the sale's second processing, 1 tick after the
    first, not at the first.
  - Changes for other specs (S-0408 claim 6): a tethered unit holds back only the releases and
    the build-down, not the end of its contact; a contact without a tether holds back nothing.
  - Claim 10a (Q-1870): a second harvester at an occupied docking cell, the close-enough
    distance, the look's scatter request, the raised base under
    `quirks.substitution_distance_flat_destination`. These state rules our engine already had.
  - Claim 18: the free unit's placement fall-back comes before its *harvest*.
  - Terms, claims 5a, 9, 10, 17 (cases (a) to (f), *Who reaches the drop*), 18's chrono part
    and scenario H8: the chrono miner's driving and teleporting, which rest on S-0207 r82 claim 28
    (bdd7fa8, not built).
  - Wording only: claims 3a, 9 (the chrono miner drives), 22, 26, 27a, the seeking table of State
    and ordering, scenarios H1, H4, H12, H13, H20 and the Open questions.

The cross-specs it names were read as they stand: S-0207 claims 15, 16, 27 and 28, S-0212 claims
23 to 25 and F6a, S-0215 claims 6, 11 and 15, S-0408 claim 6, S-0209 claim 5. None was revised for
this intake. S-0212 r83 (5d1e9f5), its Q-1975 answer (9cdf2a6), M-0011 and the Q-1712 partial
answer reach this branch through main (intake 37 built r83); this intake builds none of them.

**Code.**
- `econ::harvest::refinery_sold` (the sale's first processing): the new order for an unloading
  harvester (leave unloading, queue and start *harvest*, then the scatter request) and for one in
  *enter* (queue *guard*, remove a destination naming the refinery, then the request). It no
  longer breaks the contact.
- `econ::harvest::sale_breaks_contact` (new), called by `econ::sell::process_selling` at every
  processing in `Stage::Begun` before the docked test: it breaks every contact with the refinery
  and says whether a tethered harvester was docked, which keeps the sale waiting one processing.
- Import (S-0401, note 30): `convert::sim` fills a terrain object's `image_frames` from its
  sprite's frame count. Before, the count was 0, so an ore mine that started its animation never
  stopped it and grew no more ore.
- Our opponent (`ai::absorber`), found and **not** changed: a slave resting idle by its master
  between trips counts as spare infantry and is sent into the house's bio reactor, where it never
  mines again (M7a seed 1 with the play fixes: slave 797 sent at tick 7813, inside from tick 8084
  to the end). Keeping slaves in `free_infantry` (`|| slaves::is_slave(w, id)`, as `grinder`
  already does) fixes it: the same slave mines to tick 12000. But in the M7a duel almost every
  absorbed unit was a slave: with the change, infantry absorbed per seed of 1 to 16 fell from 6 to
  13 to 0 to 2, and the gate seed 92 fails (absorber orders 2, absorbed 0). Left for the lead
  (re-pick the M7a seed or give the duel spare infantry, then take the change).
- No new saved or hashed state; save format 83 and AI game format 45 are unchanged.

**Design.** The spec does not say whether the sale's docked test reads the contacts before or
after the second processing breaks them. We read them before: otherwise no tethered harvester
could ever hold back the build-down, as the change for S-0408 claim 6 says it does. A sale with a
tethered harvester docked therefore starts its build-down one processing later than before
(S-0408 claim 6: "with no unit docked … the second processing").

| Claim or item | Status | Notes |
|---|---|---|
| 11 (sale: harvest first, then the scatter request while no destination; contact to the second processing) | tested | `claim_11_sale_breaks_contact_at_its_second_processing` (H12 a2: *harvest* current, a scatter cell, contact kept at the first processing; broken 1 tick later; the tethered harvester holds the build-down), `claim_11_refinery_sold_while_unloading` (unchanged). |
| 11 (request refused when *harvest* could not start) | tested | `claim_11_sale_scatter_refused_when_harvest_cannot_start` (a turning body: *unload* current, *harvest* queued, no destination). |
| 11 (in *enter*: guard; a destination naming the refinery removed; a turning body refuses) | tested | `claim_11_sale_while_entering_on_the_docking_cell_still_turning` (H12 a3: never docks, in *guard* after the turn). |
| 11 (in *enter*, driving to the docking cell: the cell kept, no request) | tested | `claim_11_sale_while_driving_to_the_docking_cell_keeps_it` (H12 a4: never docks). |
| 11 (chrono miner: the scatter cell as a qualified docking assignment) | waits | S-0207 r82 claim 28 intake. Today a teleporting chrono miner refuses the request (S-0207 claim 16 b). |
| Changes for other specs (S-0408 claim 6: tether, not contact, holds the sale) | tested | `claim_11_untethered_contact_does_not_hold_the_sale` (build-down at the second processing). |
| 10a (occupied docking cell within the close-enough distance) | tested | `claim_10a_occupant_scattered_and_the_contacting_harvester_docks`. |
| 10a (raised base; `substitution_distance_flat_destination`) | tested | `claim_10a_raised_base_substitutes_every_offer` (on: 600 ticks with no dock, the occupant stays, contact and reservation kept; off: it docks). |
| 10a (chrono miner at an occupied cell) | waits | S-0207 r82 claim 28. |
| 18 (placement fall-back before *harvest*) | tested (later) | `objects::place_with` already runs the placement fall-back (S-0215 Terms) before `grand_opening` queues *harvest*. Its only visible effect, the chrono miner's driving, was built at intake 39 and is tested by `claim_28_placement_installs_driving` ([intake39-S-0207.md](intake39-S-0207.md)); stale "waits" closed at seams sweep 2. |
| 3a (no docking-cell exception) | by construction | Wording; `claims_s0212_docking.rs` covers it. Chrono exception: claim 17's, unchanged. |
| Terms, 5a, 9, 10 (repeated dock test), 17 (a) to (f), 18 (chrono), H8 | built at intake 39 | S-0207 r82 claim 28 intake (driving installed at placement, adoption and the sale): see [intake39-S-0207.md](intake39-S-0207.md), whose rows carry the tests; stale "waits" closed at seams sweep 2. |
| 14 (wake-up rule) | unchanged | Note 23: Q-1990. |
| 22, 26, 27a, State and ordering, scenarios, Open questions | wording | No rule change. |

**Notes 23, 24 and 30** (local lane `crates/app/tests/harvest_lane.rs`, map 0, outputs in the
git-ignored `local-lane/m8/a3-36/`).
- 30: fixed by the import. Before: frame count 0, all 14 mines animating for good, 35 of 286
  emptied cells near the mines had ore again at tick 9000 (from field spread only). After: the
  sprite's frame count, the mines start and stop, 123 of 286 cells by tick 9000. A 5 × 5 control patch far
  from any mine stays empty in both. Seam: the client does not draw a mine's animation frames.
- 23: a slave miner vehicle with no destination, off tiberium land and with no field within its
  short scan, never wakes its slaves (claim 14; our player houses never take the
  computer-controlled alternative). Ordered to a field cell, it travels, deploys and works.
  Near a field it waits about the kick delay, then works. A placed slave-miner building, the
  plain harvester and the chrono miner work for both houses. Request Q-1990.
- 24: not reproduced. The computer's miners keep cycling on map 0. On map 8 (headless, 22000
  ticks) its two slave masters pack up at tick 20400 and redeploy at real sites, not at (0, 0). The
  absorber finding above is the one cause found of the computer's slaves going missing.

**Gate and margins** (run on the build before the rebase, main 21651b9 with this intake).
- Sweeps M6c, M7a and M7b, seeds 1 to 16, reference and with the play-fixes layer: every line is
  the same as before. The M7a gate (seed 92) passes.
- Oracle arena (local lane; exit 101 as before). Before: M-0001 30 of 96 pairings failing, 151 of
  172 unanimous outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64. After:
  M-0001 31 of 96, 150 of 172; M-0002 12 of 24, 42 of 47; M-0003 2 of 35, 60 of 64. Bisected: with
  the sale change alone every arena game is the same; the import change moves them. The arena
  plays on the record's stock map, which holds 12 animated ore mines (checked with a local-lane
  probe of the loaded map): with the frame count filled, a mine's animation now ends, so it draws
  again and the random stream shifts. The same binary run twice gives the same games.
- Quick skirmish (`glasnost-headless ai-sides 120000 7 <map>` with `GLASNOST_AI_QUICK=1`; maps 8,
  2 and 0; Yuri as house 1 against Allied and against Soviet; reference and with the play-fixes
  layer; 12 games, `local-lane/m8/a3-36/quick/`): every game is over with a winner, between
  23 922 and 57 159 ticks, and none panicked. Incomes (house 0 / house 1): map 8 Allied 57 979 /
  29 394, Soviet 30 240 / 49 844; map 2 Allied 93 673 / 87 238, Soviet 19 117 / 29 565; map 0
  Allied 75 951 / 29 038 (play fixes 47 054 / 55 962), Soviet 38 955 / 53 987. Yuri's bio
  reactors held 3 to 15 passengers at once in these games (the absorber finding above).
- Goldens (after the rebase onto intakes 35 and 37): all pass unchanged, and so do the M6c, M7a and M7b
  gate games.
