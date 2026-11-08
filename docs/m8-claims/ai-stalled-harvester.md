# Own AI: a harvester stalled in *unload* (M8)

Our own AI (PLAN §4.7), so design work, not a spec build. Found by intake 113 ([intake113.md](intake113.md)): in the AI quick lane (seed 9, map 0, sides 1,2) house 0's harvester stood in *unload* from about tick 79 557 to the end of the game with no contact, no destination and no dock, and kept its load. That is S-0402 claim 11, *Contact lost otherwise while unloading*, under `quirks.unload_without_contact_stalls` (on in the reference ruleset, off in the play-fixes layer): its refinery was destroyed under it, and the harvester stays in *unload* "until an order gives it a behaviour". A person sees a harvester standing still where no refinery is and clicks it; our AI gave no order, and the house sat at 0 credits.

## Why the AI missed it

The plan's harvester rule (`Script::think_with`, step 4) sends a harvester to harvest when it rests (no behaviour, *guard*, *area guard* or *stop*) or is idle (`scripted::idle`). *Unload* is neither, so a harvester stalled in it was never ordered. No other part of the brain orders harvesters.

## What changed

- `scripted::stalled_unload`: a harvester whose *unload* has lasted more than `UNLOAD_STALL` ticks (150, `// design:`), that is out of the unloading state, has no contact, no destination and nothing queued. That is exactly claim 11's stall (row 1 taken, nothing queued). A harvester really unloading holds its refinery's contact (every row but the first needs one), and an ordered one has a destination or a queued behaviour, so neither counts.
- The harvester rule sends such a harvester to harvest (the same order, with no destination, that it gives a resting harvester), under the same condition that the house owns a refinery.
- The wait counts from the start of *unload*, the only clock the harvester shows; the docking is never later than the lost contact. A stalled harvester stays for good, so the wait need only pass the few ticks between a lost contact and the next processing, in which a harvester whose contact ended another way (or under the play-fixes layer) moves on by itself. Ten think periods is about what a person takes to notice the harvester and click it; it costs one harvester a short pause, against the tens of thousands of ticks of the lane game.
- The order is the one a person gives, through `orders::issue`; the AI reads only its own harvester's state (as a person sees it standing), draws no random value and adds no saved state.
- AI game format 82 → 83, past intake 114's 82 (decisions change; first 81 → 82 on the earlier base 024af18). The simulation's save format stays at 117: the brain is saved by `Game::save` beside the world, and no world state changed.
- With the play-fixes layer on (the app's default) the quirk is off, the harvester moves on by itself, and the rule never acts. It matters for the reference ruleset (the box *Pathing and other fixes* unticked, and the lanes without `GLASNOST_PLAY_FIXES`).

## Tests

| claim | status | test names or reason |
|---|---|---|
| S-0402 claim 11, *Contact lost otherwise while unloading* (the stall the AI answers) | tested | `sends_a_harvester_stalled_in_unload_to_harvest` (`crates/deterministic/ai/tests/economy.rs`): on the synthetic skirmish with a second refinery, a harvester unloading in contact gets no order even with its unload made 10 000 ticks long; its refinery is destroyed (`combat::death::kill`), it survives, stays in *unload* with its load, no contact, destination or queue for 300 ticks; the next think gives it exactly the harvest order, `orders::issue` takes it, and 30 ticks later it has left *unload*. Fails without the rule (the think gives no order). |

## Other units left in a behaviour until an order

Searched: `grep -n 'until an order'` and the like over `spec/`, and the quirk list of the play-fixes layer (`crates/deterministic/rules/src/layers.rs`). None was clearly the same gap reached by our AI, so none was changed:

- **Absorber stale contact** (S-0605 claim 17, `absorber_stale_contact_kept`): already answered, `absorber::stalled` clicks the building again (M7a step 1g).
- **Depot lost under a docked vehicle** (S-0314 claim 22 and S-0504 claim 14, `depot_loss_leaves_sleep`, on in both layers): a vehicle in *sleep* after its arrival stays asleep until ordered, and `scripted::idle` does not count *sleep*, so a vehicle our AI sent for repair (`Brain::send_for_repair`) would never rejoin a wave. The closest candidate. Not changed: a contact within S-0312 claim 17's distance of the destroyed depot is destroyed with it, and a vehicle docked on a depot stands on it, so the case needs a vehicle carried off the depot after its arrival or a docking point farther away (a naval yard's, depending on the data). The AI fixture has no depot, and in the lane dumps (36 games, every 10 000 ticks) one vehicle was asleep at one sample only, damaged (most likely under repair), never at two (below). A follow-up can count such vehicles in the lane before it adds a rule.
- **Hover landing request** (S-0503 claim 19, `hover_landing_request_stands`, off in the play-fixes layer): a landing-first in-place hover keeps *unload* current and never deploys. The deployers module would not re-order it. Whether a stock type reaches it is not traced; see the lane note below.
- **Aircraft enter stall on the ground** (S-0501 claim 24, `aircraft_grounded_enter_stall`) and **a non-pad-bound aircraft asleep away from contact** (S-0501, restored *sleep*): the AI flies aircraft from pads, which are pad-bound; not seen in the lane.
- **Vehicle chargers do not break off** (S-0518, `vehicle_charge_no_break_off`): the unit keeps firing at its own building, so it is busy, not standing; the base defence sends only units not already fighting. Not the same gap.
- **Stranded hover** (S-0314, `depot_loss_strands_hover`), **zero load period** (S-0402 claim 5, `zero_load_period_stalls`) and **a refinery that is not a dock-unload refinery** (S-0402 claims 8 and 10, `non_unloading_refinery_accepts`): an order does not free the unit (it never moves again, or it loads or enters the same way again), so no order of a person helps.
- **Units left at rest until an order** (a harvester at rest on a docking cell after a give-up, S-0212 claim 23; a depot's docked vehicle in *guard*, S-0309): resting, so the existing rules (`idle`, the harvester rule) already order them.
- **A unit a factory has not sent out when the factory changes owner** (S-0309 claim 31, S-0417 claim 2): it stays at its exit point in contact with the factory, now another house's. It reaches our AI only through a capture or mind control of its factory; not traced, listed as a seam.

## Lanes

Base: main at 024af18 (save format 116, AI game format 81). After: 9162f71 on that base (AI game format 82). Raw outputs git-ignored under `local-lane/m8/aih/`.

**AI quick lane** (`GLASNOST_AI_QUICK=1 glasnost-headless ai-sides 120000 <seed> 0 2 8`, seeds 7 to 10, 36 games): every game ends with a winner, before and after. 32 games are identical. Four change, each first at a think that sends a harvester stalled in *unload* to harvest:

| Seed, map, sides | Before | After | First differing tick and object (`GLASNOST_AI_HASH`, `GLASNOST_AI_OBJS`) |
| --- | --- | --- | --- |
| 7, 8, 0 and 1 | house 0, 34 112 | house 0, 49 658 | 24 188: house 1's harvester 453, in *unload* since 24 027 with no contact, gets *harvest* queued. In the base game it is gone before the 30 000 sample. |
| 9, 0, 0 and 1 | house 0, 38 695 | house 0, 37 026 | 28 853: house 1's harvester 1150, in *unload* since 28 696 with no contact. In the base game it still stands there at the 30 000 sample. |
| 9, 0, 1 and 2 | house 0, 113 455 | **house 1**, 106 239 | 79 711: house 0's harvester 1158, in *unload* since 79 557 with no contact (the stall found by intake 113). In the base game it stands there to the end. |
| 10, 2, 0 and 2 | house 1, 69 105 | house 1, 47 791 | 23 281: house 0's harvester 774 (the other harvester type), in *unload* since 23 115 with no contact. In the base game it is gone before the 30 000 sample. |

Each first difference is the rule of this change acting on claim 11's stall, 154 to 166 ticks after the docking (the first think past `UNLOAD_STALL`); nothing else differs at that tick. The one changed winner (seed 9, map 0, sides 1 and 2) follows from it: in the base game house 0's harvester stood loaded for the last 34 000 ticks while house 0 sat near 0 credits. No game was tuned toward.

Stalls left: in the after dumps (every 10 000 ticks), no harvester stands in *unload* at the same cell at two samples, and none of the four stalls above recurs. In the base dumps only the seed 9 stall showed at two samples; the other three were found by the traces.

**Seed sweeps** (`GLASNOST_SWEEP=1-16`, M6c, M7a and M7b `seed_sweep`): output identical before and after, apart from timings, so the rule never acted there. Smallest minimum: M6c 0, M7a 1, M7b drain 3.

Other units in *unload* in the dumps (for the list above): besides harvesters at refineries and factories releasing units, non-harvester vehicles stand in *unload* at single samples. One object (seed 8, map 2, sides 1 and 2, the same in both builds) stands in *unload* at the same cell at two samples. An in-place deploying vehicle keeps *unload* while deployed (S-0517), which fits. It was not traced further, so the hover landing request (S-0503 claim 19) is not ruled out for it.
