# S-0108 r106: claim ledger, M8 step A3 intake 62

Built to r102 by intake 53; claim 26's slot-play order against M-0014 by intake 55b
([ledger](intake55b-S-0314-S-0202-S-0601-S-0612-S-0614-M-0007-M-0008-M-0014.md)). r102 to r106
changes wording and backing in claims 13 and 28 (no behaviour), adds claim 26's *Tick of the first
advance* (answers **Q-2186**), the B22 part (p) scenario, and a note for S-0203 claim 3. No
revision adds a quirk ("This claim adds no switch"); the rendering quirk test is unchanged and green.
This is client work plus one presentation field: nothing in the simulation's state, save format,
AI game format or goldens changes (`Effect` is neither saved nor hashed), so the sweeps, the arena
and the AI lanes are not needed and were not run.

## Built

- `Effect::Captured::after_slots` (presentation only) and `ownership::change_by` /
  `mind::set_owner_by`: the change says whether the object that made it stands after the
  building's slots in the update order. A slot's animation stands directly after a map-placed
  building, so the test is the actor's position against the building's in `World::update.order`.
  Actors: the capturing engineer, the arriving unit of a takeover, the controller of a mind-control
  takeover and release. A trigger's change and the occupation rules' (the building's own turn)
  are before the slots. `change` and `set_owner` (no actor) keep the earlier rule: an
  announcing cause after the slots (`// design: Q-2186`, for callers that name no actor).
- `mapview::live` keeps a building in `turn_captures` only when `after_slots`; `State::change_after_slots`
  (renamed from `captured_in_turn`) selects `Slots::enablings` (after the slots' turns, first
  advance next tick) or `early_enablings` (before them, first advance in the tick of the change).

## Claims

| claim | status |
|---|---|
| 26 Tick of the first advance, trigger's change | Tested: `m0014_trigger_owner_change_timing` (slot_play), M-0014 |
| 26 engineer that stands after the slots (trained, left a closed transport, starting engineer) | Tested: `m0014_capture_releases_in_the_next_tick`; `s0108_claim_26_the_change_says_whether_it_follows_the_slots` (sim, an engineer after the building) |
| 26 engineer the map places (before the building, so before its slots) | Tested: `a_capture_by_an_engineer_before_the_slots_releases_them_one_tick_earlier` (live, drawing) and the sim test above (an engineer before the building). That the sim's load puts map infantry before buildings is S-0009 claim 43 (`objects.rs`), existing |
| 26 starting engineer after the slots | Partly: it stands after every map building in the update order (S-0203 claim 3, `load.rs`), so the id test gives "after". Rated low and `unmeasured` by the spec |
| 26 takeover of a vehicle-like building, mind-control takeover and release | Tested at the sim level by the same order test (the actor is the arriving unit or the controller); no live test. Release by a dead controller still gives "after" through the actor's absence |
| 26 garrison takeover and return (the building's own turn, before its slots) | Tested: the occupation cause gives `after_slots == false` (sim test) |
| 26 slots restarted after the capturer entered | Seam: the client does not track when a slot's animation entered play, only the building's position. A slot started later than the building (a restart) would stand after a newer engineer; not modelled |
| 26 a counting removal earlier in the pass costs a slot its turn (S-0203 claim 10) | Seam: the client's slot turns do not model skipped turns (existing seam of intake 53) |
| 13, 28 wording and backing | No behaviour; nothing to build |
| B22 part (p) | Not run (a measurement scenario for the lead) |
| S-0203 claim 3 note (slots after their building) | Spec change for S-0203; nothing to build in the sim (it has no slot animations) |

## Requests

None. Q-2221 to Q-2225 are unused.

## C7

No quirk, no switch, no play fix.
