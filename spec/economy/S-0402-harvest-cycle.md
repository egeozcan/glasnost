---
id: S-0402
title: Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners)
topic: economy
revision: 84
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-13
measurements: []
requests: [Q-0140, Q-0141, Q-0142, Q-0143, Q-0144, Q-0145, Q-0194, Q-0197, Q-0270, Q-0371, Q-0411, Q-0412, Q-0413, Q-0414, Q-0441, Q-0442, Q-0500, Q-0550, Q-0580, Q-0590, Q-0900, Q-1130, Q-1759, Q-1870, Q-1990, Q-3062, Q-3063]
supersedes: []
---

# S-0402: Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners)

## Summary

A harvester drives to a resource field, loads bales one at a time, carries its load to a refinery
of its owner, docks, and is paid for each kind of resource it carries; then it goes back for more.
This spec covers the harvester's capacity, the timing of a load step, how it chooses a field and a
refinery, when it waits or queues, docking and unloading, the credits a house gets and when, what
happens when fields are exhausted or no refinery is left, and the variants: the slave miner, whose
infantry slaves gather for it, and the chrono miner, which teleports to dock and at a sale of its refinery; resource cells
themselves (stages, bales, growth) are S-0401's. This spec covers skirmish only; network
games and campaigns are out of scope. It answers Q-1870 in part: that a harvester left on a docking cell by a failed
departure stays there, except a chrono miner (claim 3a), and what a second harvester does at
that occupied docking cell (claim 10a). Whether the path computation gives up (the request's
first question) is S-0212's. It answers Q-1990: a slave miner in *guard* that rests far from
any field and not on tiberium land wakes by itself only for a computer-controlled owner, and
our AI gets that behaviour by ordering the miner to harvest (claim 14). The separate area-guard
effect of claim 14 wakes a resting group in *area guard* whatever the owner.

## Terms

Glossary terms are used as defined there, in particular *house*, *country*, *credits*, *vehicle*,
*infantry*, *building*, *behaviour*, *contact*, *tether link*, *cell*, *usable area*, *movement
zone*, *reference point*, *step direction*, *facing*, *tick*, *scenario random stream*, *AI house*,
*human-controlled*, *computer-controlled*, *neutral house*, *special house*, *departure*, *importer warning*, *quirk switch*,
*round-toward-zero mode*, *switched on, switched off* (a building's power switch, S-0405, claim 10),
*being erased* and, in the scenarios, *bystander computer player*. S-0205's notation applies (trunc,
RZ₂₄, RZ₅₃, *single*, *double*), and S-0215's *standard delay*, *processing*, *start tick* and queueing
wording (S-0215, Terms and claim 6). S-0401's
terms *resource cell*, *the tiberium of a cell*, *stage*, *bale*, *value per bale V* and the
*worth* of a cell (S-0401, claim 2) are used as S-0401 defines them. In addition:

- **Harvester.** A vehicle whose type has the harvester flag. A *weeder* (the weeder flag) gathers
  the weeds land type instead; weeders are left to a later revision (Edge cases).
- **Turn.** Used in S-0203's sense: an object's share of a tick (S-0203, Terms). A rotation is
  written with facing words.
- **Dock types.** A harvester type's list of building types it may unload at (its dock list).
  A *refinery* here is any building of a dock type of the harvester in question (glossary,
  *refinery, dock types*). The *refinery flag* is a building type's refinery flag (Import mapping),
  and a *refinery-flag building* is a building whose type has it, whoever owns it. Claims 3, 11, 13
  and 17 use the refinery-flag building where they say so. Every other "refinery" in this spec is
  the first set, the dock-type buildings.
- **Capacity *C*.** A harvester type's storage field: the number of bales it can carry, an integer.
  With `quirks.zero_capacity_harvester_cycles` off, a vehicle harvester's imported capacity
  of 0 instead gives effective *C* = 1 bale throughout this spec. Slave capacity is unchanged.
- **Load.** The bales a harvester carries, held as one amount per tiberium. Amounts are whole
  numbers of bales. The *total* is the sum of the amounts. For *C* ≥ 1, the harvester is *full*
  when the total equals *C*. For *C* = 0, it is treated as not full while seeking and at a load
  step, and as full in the check after a failed load step; it never carries a bale (Edge cases).
- **Harvest phase.** The part of the *harvest* behaviour a harvester is in: *seeking*, *loading*,
  *returning*, *requesting* or *idle* (claim 3). A harvester enters *harvest* in the seeking phase.
- **Destination.** Unless qualified otherwise, the order-level destination, not the
  locomotor destination point or the heading point of S-0207 claims 24 to 26.
  In particular, an absent locomotor destination point does not mean that a harvester
  has no destination for the seeking test of claim 3.
- **Remembered cell.** A cell a harvester keeps so that it can drive back to its field after
  unloading (claims 6 and 12). An owner change forgets it (S-0309 claim 31).
- **Landing point, pending jump, vehicle-present mark.** S-0207's terms (claims 20, 21 and 27): the
  fixed landing point of a teleporting object's destination, the jump still to be made to it, and the
  vehicle-present mark that S-0207 sets on the cell of a vehicle's landing point. Claim 17 calls that
  mark the "reservation".
- **Scan range.** A number of cells, taken from one of the rules' scan fields given in leptons
  (S-0003 lepton form): the field divided by 256, truncated toward zero. The *long scan* and *short
  scan* ranges come from the resource scan fields (Import mapping).
- **Ring search** (claim 4). The search for a field cell around a harvester.
- **Top-left cell.** The cell of a building's position: the top-left cell of its foundation, the
  one with the smallest *x* and *y*; not the reference point, which is the foundation's centre.
- **Dock-unload refinery.** A refinery whose type has the dock-unload flag (Import mapping). Only
  such a refinery directs a harvester to its docking cell (claim 10).
- **Docking cell.** For a dock-unload refinery with top-left cell (*x*₀, *y*₀), the cell
  (*x*₀ + 3, *y*₀ + 1), whatever the size of its foundation; the art does not move it (claim 10).
  A harvester unloads standing on it, and claim 11 looks for the refinery in the neighbouring cell
  in step direction 6 (map west) of the harvester's cell.
- **Queueing cell.** A cell near a refinery where a harvester that has not reserved it waits: the
  refinery's top-left cell moved by the queueing shift of the refinery's art (Import mapping).
  The harvester drives to the result of a nearby-cell search around it (claim 9).
- **Dock slots.** The number of harvesters that a refinery can hold in contact at once: the
  number of docks of its art, at least 1 (Import mapping).
- **Load step period *L*.** The rules' harvester load-rate field, in ticks.
- **Dump rate *D*.** The rules' harvester dump-rate field, in game minutes: the value read from
  data is a single widened to double; the absent-key default is a double (Import mapping).
- **Income factor *M*.** The income factor of the house's country, a single-precision number.
- **Step counter.** A harvester's count, in whole steps, that paces its load steps (claim 5) and
  its unloading payments (claim 11). It goes up by 1 when its *pacing period* has passed (claim 5)
  and is set to 0 by the events those claims list.
- **Pacing period.** A whole number of ticks, possibly 0 or negative, that sets when the step
  counter rises (claim 5, Formulas F6).
- **Holding order.** The order of a house's buildings in which the harvest rules take them (claim 8):
  a building joins the end of its owner's holding order when it is put on the map for that owner,
  and leaves it when it ceases to exist; the buildings that remain keep their relative order. A
  produced building (S-0404) joins when it is placed, not when its production starts. The same
  holds for a building that a deploying vehicle creates and for a building that the map pre-places,
  so the buildings that exist at the start of a skirmish join in the order in which the set-up puts
  them on the map (S-0009 for pre-placed buildings, S-0412 for the start). A building that changes
  owner (capture) leaves its old owner's order at that moment and joins the end of its new owner's
  order. Everything a harvester can observe of the holding order is claim 8's tie rule and its
  primary-building rule.
- **Free unit.** S-0408's term (Terms): the vehicle type that a building type's free-unit field
  names. Claim 18 says when one is created.
- **Grand opening.** S-0108's term (claim 11): a building's once-only entry into service,
  at construction completion in play, during placement at game set-up, when a trigger action
  creates the building in play, or at an owner change of a building that has not had it (S-0309
  claim 31).
- **Start price.** The price of a produced item at the moment its production starts (S-0404,
  claim 2), which the original records on the produced object; claim 18 reads it.
- **Docking, payment and unloading animations.** Three animations of the building in the
  neighbouring cell (claim 11): the *docking animation* (the building's pre-production animation,
  played when unloading starts), the *payment animation* (its special animation, played at a
  payment unless it is already playing) and the *unloading animation* (its production animation,
  played, by a refinery-flag building only, when the harvester's load is empty). They are the
  building's *pre-production*, *special-1* and *production* slots of S-0108, and each is started
  as S-0108 starts a slot (Terms there), with no occupied variant allowed: with its damaged
  variant when the building's health ratio is at or below the yellow health threshold (S-0003,
  claim C9), with its normal variant otherwise. The type's art read fills an empty damaged name
  with the normal name (S-0108 Import mapping, **Names**), so a damaged building whose art gives
  only the normal name plays that normal animation, for each of the three alike. A start plays
  nothing only when the chosen variant's name is empty or names no animation type: the art gives
  neither name, or it gives only the damaged name and the building is not damaged. Only the
  unloading animation delays the harvester (claim 11).
- **Primary building.** A building marked as its owner's primary factory (S-0213, claim 13, owns
  how the mark is switched). Any building that carries the mark counts, including a refinery of a
  type that is also a factory type.
- **Purifier count *n*<sub>p</sub>.** The number of ore-purifier buildings (the building type's
  ore-purifier flag) that the owner of the refinery being paid has, counted from when each is complete (outside its construction
  state) until it is removed or changes owner; power does not matter.
- **Purifier bonus *b*.** The rules' purifier-bonus field, a single.
- **Slave miner, slave.** A vehicle type that both gathers and processes resources and controls a
  group of infantry *slaves* of the type its enslaves field names (S-0215 calls them the slave
  master and its harvesting slaves). It can deploy into a building that keeps controlling them.
- **Chrono miner.** A harvester whose type has the teleporter flag, which is the flag S-0207 calls
  the teleport-capable flag (the same imported key). It drives everywhere except on its jump to a
  refinery's docking cell during *enter*, and on its jump when that refinery is sold while it
  still holds the contact: to the scatter cell, or, for a miner that was unloading and is not
  full, to the field cell that its first seeking processing finds (claims 11 and 17). This holds
  outside S-0207 claim 28's exclusions (a warp-pending immobilization, being gripped, being
  seized: claim 28's paragraph *The rule*). The rule is that, outside those exclusions, every
  destination other than a qualified docking assignment installs temporary driving: one given
  while the miner is in contact with a building that permits dock unloading, as a cell
  destination, with no ground-layer vehicle in that cell (S-0207 claims 27 and 28). In ordinary
  play such assignments include the docking-cell offer, its repeated dock test, a sale's
  scatter request (claim 11) and, for a miner that was unloading at the sale, the cells that its
  first seeking processing assigns while the sale's contact holds (claim 17, case (f)). A
  vehicle-factory-built miner also starts with temporary driving locomotion for its first field
  leg (S-0417 claim 16).
- **Group update.** The slave group's periodic opportunity to regenerate, restore or release
  slaves and to reconsider the master's movement (claim 21). Its group states are *at rest*,
  *field seeking*, *travelling*, *retrying deployment* and *working* (claim 22). *Working*
  permits ready slaves to emerge; it is not the master's current behaviour. A deployed group
  also has *checking the field* and *packing* states (claims 22 and 27).
- **Slave place.** A persistent place in the group, numbered by initial creation order.
  Replacing a slave keeps its place; replacement creation order does not reorder the group.

## Behaviour

### The cycle

1. **Capacity and the load.** A harvester carries at most *C* bales. Each bale it loads is added
   to the amount of the tiberium of the cell it came from. A harvester type with *C* = 0 is bad
   data: under `quirks.zero_capacity_harvester_cycles` (default: original), its fill ratio
   is 0 / 0, which is not a number, and the cycle runs without ever carrying a bale
   (Edge cases). With the switch off, imported capacity 0 uses effective capacity
   1 bale and the ordinary positive-capacity cycle. The importer warns in either setting.
   The load
   survives everything in this spec except unloading; what happens to it when the harvester dies
   is S-0312's.
   - Backing: `measurable` (scenario H1; its variant d for imported capacity 0)
   - Confidence: high; low for *C* = 0

2. **The harvester behaviours.** The cycle uses three behaviours: *harvest* for seeking, loading
   and returning to a refinery (claims 3 to 10), *enter* for docking at the refinery (claim 10) and
   *unload* for unloading (claim 11). A *harvest* order on a cell (S-0214) gives *harvest* with the
   cell as destination; an *enter* order on a refinery starts docking directly. The *return*
   behaviour is not used by the cycle as far as known (Open questions). Stopping a harvester in
   *harvest* or *return* gives it *guard* at once (S-0213, table T1). A harvester's resting
   behaviour is S-0215's (claim 15, row 4, and claim 24's vehicle rows).
   - Backing: `measurable` (scenarios H1 for the three behaviours, H9 for the stop order and for
     *return* not occurring)
   - Confidence: high for the three behaviours; low for *return* not being used

3. **Harvest phases.** Within *harvest* a harvester moves through the phases of the table in
   **State and ordering**. The phase outcomes are:

   | Phase | Condition | Observable result |
   |---|---|---|
   | Seeking | Full, except for the *C* = 0 treatment in Terms | Changes to *returning*. |
   | Seeking | Not full and has a remembered cell | Drives to that cell and forgets it. |
   | Seeking | Not full, no remembered cell and has a destination | Remains seeking. |
   | Seeking | Not full, no remembered cell or destination; long-range ring search finds its own cell | Starts *loading*. |
   | Seeking | Not full, no remembered cell or destination; search finds another cell | Drives to that cell and remains seeking. |
   | Seeking | Not full, no remembered cell or destination; search finds nothing | Goes *idle*. |
   | Loading | — | Takes load steps under claim 5 until full or the cell is exhausted; claims 6 and 7 give the resulting transitions. |
   | Returning | — | Chooses and reserves a refinery, or drives towards one, under claims 8 and 9. |
   | Requesting | Next processing | Queues *enter* under claim 10. |
   | Idle | Next processing | Steps off a refinery-flag building's cell if standing on one, then queues *guard* under claim 13. |

   In every phase, a harvester whose owner owns no building of any of its dock types queues *guard*
   at its processing, and nothing else of its phase happens (claim 13). The processing interval is 1 tick in *loading*, after
   seeking finds it full, after seeking starts loading and in *requesting*; 105 ticks after it goes
   *idle* for lack of a field; and the standard delay of *harvest* (S-0215, claim 6, one draw)
   otherwise, including every processing in *returning* and the change from *returning* to
   *requesting* (State and ordering).
   - Backing: `measurable` (scenarios H1, H2, H5)
   - Confidence: high for the phases; medium for the intervals

4. **Ring search for a field cell.** The ring search with range *R* from a harvester's cell
   (*x*, *y*) returns:
   - the harvester's own cell, when its land type is *tiberium* (S-0211), whatever the cell holds,
   - otherwise, for rings *k* = 1, 2, …, *R* − 1 in turn, the cells at Chebyshev distance *k*, and
     it stops at the first ring that holds at least one *acceptable* cell. In that ring it returns
     the acceptable cell of greatest worth (S-0401, claim 2: *V* × (*d* + 1)). Ties go to the cell
     with the smallest *tie key*. A ring-*k* cell displaced by (*a*, *b*) from (*x*, *y*) lies on one
     edge of the ring, or on two if it is a corner, and each edge gives it a pair: the top edge
     (*b* = −*k*) gives (*a*, 0), the bottom edge (*b* = *k*) gives (*a*, 1), the left edge
     (*a* = −*k*) gives (*b*, 2) and the right edge (*a* = *k*) gives (*b*, 3). Its tie key is the
     smaller of its pairs, compared by the first number and then by the second,
   - nothing, when no ring up to *R* − 1 holds an acceptable cell (also when *R* ≤ 1).

   A cell is acceptable when it is in the usable area (height-aware form, S-0009 claim 5), the
   harvester's movement zone connects it with the harvester's cell (S-0212), its land type is
   *tiberium*, and the harvester could enter it now (the entry outcome *clear* for a ground
   vehicle mover, S-0209 claim 5).
   Shroud does not matter in a skirmish. The search takes no draws. Within the selected ring, preference is by the product of value per bale and bales
   present, not by either factor alone. At equal density, greater value per bale wins.
   For the same tiberium, a denser cell wins.
   - Backing: `measurable` (scenario H2; variants b and c for the entry test and the movement zone)
   - Confidence: high; medium for the entry test (which entry outcomes count as "could enter")

5. **Load steps and their timing.** In the *loading* phase the harvester is processed every tick.
   A load step is made at a processing only when the *step counter* it sees is at least 9. The
   counter rises by 1 on a schedule set by its *pacing period* *p* and the tick in which that period
   was last started: every *p* ticks after the start for *p* > 0, in every tick from the start on for
   *p* < 0, and never for *p* = 0 (`quirks.zero_load_period_stalls`, Edge cases). Starting a period
   replaces the schedule of the one before. A rise in tick *t* first counts at the processing in tick
   *t* + 1, and it counts there even when the processing in tick *t* set the counter to 0 (Formulas
   F6). With nothing interrupting and *C* ≥ 1, the first bale is therefore loaded 19 ticks after
   loading starts and, for *L* > 0, each further bale 9*L* + 1 ticks after the previous one (Formulas F1); the "+ 1" and the 19
   come from that one-tick lag.

   A load step takes one bale from the harvester's cell (S-0401, claim 3) when the harvester has
   no destination, is not full and stands on a cell of land type *tiberium*. It *succeeds* when a
   bale is delivered. The counter and period change as follows:

   | Event | Step counter | Pacing period, started now |
   |---|---|---|
   | Loading starts (seeking finds the harvester's own cell) | set to 0 | 2 ticks (engine constant) |
   | A load step succeeds | set to 0 | *L* |
   | A load step fails because the harvester is full or its cell is not of land type *tiberium* | set to 0 | 0 until the harvester's next processing in *loading*, which starts the period *L* |
   | A load step fails only because the cell's last bale gave nothing (S-0401, claim 3) | unchanged | unchanged |
   | A load step fails because the capacity is 0 (it takes no bale, as the room left is 0) | unchanged | unchanged; the check after the failed step then counts the harvester as full (claim 6, Edge cases) |
   | The harvester has a destination: no step is made, and it stays in *loading* | unchanged | unchanged |

   The harvester discovers that it is full only at a load step, so after the bale that fills it,
   it stays in *loading* until the counter has reached 9 again and the step fails (F1). A harvester
   that drives on to another cell of the field within *loading* (claim 7) keeps counting while it
   drives. If its counter has reached 9 when it arrives, it loads its next bale at its first
   processing there, without a new wait. With *L* = 0 the period after the first bale is 0, so
   with the original's rule no second bale is ever loaded; with *L* < 0 the counter goes up every
   tick and bales follow 9 ticks apart (Edge cases).
   - Backing: `measurable` (scenario H1; H3 for the drive-on case)
   - Confidence: high for the 9-count, *L* and the counter resets; medium for the "+ 1" and for the
     19 ticks, which depend on the one-tick lag of the counter; medium for the period of 0 and the
     negative periods (scenario H1, variant c)

5a. **The vehicle's harvesting display (Q-0550, Q-0580, Q-0590).** A vehicle harvester shows a separate
    harvesting sprite while loading, subject to the visibility conditions below. It is the
    same fixed game-wide sprite for every harvester type and every resource kind, including
    ore and gems. Neither the vehicle's rules nor its art names a harvesting animation.
    This is an extra image drawn with the animation palette, centred at the projected point
    of F8 and composited before both the vehicle's shadow and its body. A passing shadow
    pixel can darken it, and the body can cover it. It has no house-colour remap.
    Its brightness is the vehicle's brightness in per-mille, using S-0104 claim 13's
    vehicle rule: the unscaled ground light of the vehicle's cell plus the extra unit
    light, with that rule's bridge-deck addition or shadowed-cell penalty where applicable.
    The displaced anchor does not select a different cell for lighting. This is not
    full brightness and is not an animation-type lighting option. Depth compositing
    belongs to S-0102 claim 13, not to the playback rules of an animation.
    It is not a separately existing animation:
    S-0109 claim 1's animation stage and its playback fields do not pace it, and it takes
    no random draws. Its image frame is given by F8, with eight facing groups of 15 frames
    each, advancing every tick and repeating after 15 ticks when the vehicle's
    walk-animation counter stays constant.

    **Start and stop.** The display becomes eligible in the processing that starts loading
    (claim 5), not on mere entry into the cell and not on the first successful bale.
    With uninterrupted loading, it is therefore already eligible during the 19-tick wait
    before that bale. It remains eligible between load steps and while full until the
    failing load step changes the phase to returning (claim 6). After an exhausted cell,
    finding another cell keeps it eligible, including before the next bale at that cell.
    Finding no cell and returning with a partial load ends it. Seeking, waiting for a
    field, returning, docking and unloading do not themselves show this sprite.
    Leaving *harvest* ends it by the vehicle's next turn; merely queueing a replacement
    behaviour does not end loading. It is not replayed from image frame 0 at loading
    start, at a bale or after movement: F8 uses the current tick and walk-animation counter.

    **Visibility conditions.** While eligible, the sprite is suppressed during movement or body
    turning, while an erasing weapon is removing the vehicle, and during its teleport disappearance
    or reappearance state. For a ground-driving harvester the movement test means either a turning
    body or an active track with a positive integer per-tick progress step *s* (S-0207 claim 2), not
    merely a positive double-precision speed fraction *ρ* (S-0207 Terms). The step is *s* =
    trunc(*s*₂ × *ρ*), truncated toward zero after the floating-point multiplication, with *s*₂ as
    defined in S-0207 claim 2. On an active track with *ρ* > 0 but *s* = 0, the sprite is shown if
    the body is not turning and the other visibility conditions permit it. Conversely, *s* > 0
    suppresses it even when the progress budget buys no track point and the position does not change
    that tick (S-0207 claim 3). A destination alone does not suppress it. For a teleporting
    harvester it means a teleport journey still in progress. A chrono miner drives on every trip
    except its docking jump and its jump at a sale (S-0207 claim 28; claims 11 and 17), so for it
    only those jumps count as teleport journeys, and its driving trips count as ordinary movement.
    Ordinary object visibility still applies. Thus driving to another resource cell suppresses the
    sprite without restarting its frame cycle, and it can reappear on stopping before another
    successful load step.

    **Body and variants.** Loading selects no different body model, voxel section, HVA
    frame sequence, turret or barrel motion, or vehicle-sprite sequence. Ordinary body
    and turret drawing continues, including S-0105 claim 16's frame rules; loading's
    step counter is not the walk-animation counter. The separate harvesting sprite is
    unchanged for a harvester that can deploy or teleport. A slave miner taking claim
    22's group-directed route does not load itself and does not gain this display merely
    because its slaves are working. Slaves' own harvesting sequence belongs to S-0107,
    and a deployed master's building images are not a vehicle harvesting sprite.
    Load pips are independently S-0111 claim 14's, already answering that part of the
    display before Q-0550 was filed. The refinery animations remain claim 11's and do
    not substitute for this field-loading display.
    - Backing: `measurable` (H26). The bridge-deck brightness case is `unmeasured`:
      H26 establishes no legal-input setup with the display eligible on a bridge deck.
      The inherited deck-light modifier alone does not establish that overlap.
    - Confidence: high for the fixed sprite, frame selection, phase boundaries and no
      loading-specific body change, table trigonometry, brightness and order before the
      shadow and the integer-step movement test; medium for placement and the other
      visibility boundaries

6. **When the harvester is full.** When a load step fails because the harvester is full (or
   because its capacity is 0, Edge cases), it runs the ring search with the short scan range and remembers the result (or forgets any remembered
   cell when there is none), and goes *returning*.
   - Backing: `measurable` (scenario H1: after unloading it drives back to a cell near where it
     stopped)
   - Confidence: high

7. **When the cell is exhausted.** When a load step fails and the harvester is not full (its cell
   was emptied, or its last bale gave nothing, S-0401 claim 3), it runs the ring search with the
   short scan range from its cell. If it finds a cell, it drives there (or stays, for its own cell)
   and keeps *loading*. If it finds none and has no destination, it forgets any remembered cell and
   goes *returning* with whatever it carries: a partial load is unloaded like a full one.
   - Backing: `measurable` (scenario H3)
   - Confidence: high

3a. **Harvesting from a docking cell (Q-0371).** For an ordinary, non-teleporting
    harvester, standing on a refinery's docking cell adds no exception to claim 3's
    seeking rules. A harvest order can give it a field destination while another
    harvester holds that refinery's dock. Dock ownership is not a prerequisite for
    leaving this cell to harvest, and the harvest rules do not replace that destination
    with *guard* merely because the departure cell is a docking cell. Whether movement
    can enter and persist on the cell and find a departure route is S-0212 claim 27a
    (scenario P27). Failure classification and route retry delay are S-0212 claim 23,
    not a special harvest retry timer. S-0215 claim 11a fixes give-up versus
    retained-seeking outcomes and subsequent behaviour-start and processing timing.
    If seeking survives with no destination or remembered cell, claim 3's search
    applies at its next processing; a queued replacement behaviour can instead start
    before another harvest processing. Ending *harvest* does not itself resume it.
    Claim 13 governs rest and the original's computer-only resumption rule.

    So after a failed departure that gives up (S-0212 claim 23), a harvester of a player
    house other than a chrono miner rests on the docking cell in *guard* (Q-1870), as it does
    after a stop order there. It does not try its field again by itself: the route retry
    delay running out restores no destination (S-0212 claim 27a), no rule of the refinery
    moves it, and claim 13's step off a refinery-flag building's cell belongs to going idle
    for lack of a field, not to a give-up. This spec adds no other way out of the cell than an
    order or a scatter request that it accepts, such as the one a second harvester's approach can send (claim
    10a). A chrono miner is the exception: in *guard* it queues *harvest* again by claim 17
    when one of its eight neighbouring cells holds a refinery-flag building of its owner, as
    the cells round a refinery's docking cell do when they belong to that refinery, and then
    claim 3's seeking applies to it.
    - Backing: `measurable` (H20's first run for the absence of a docking-cell exception, part
      e for the rest after a stop order), and `unmeasured` for the rest after a give-up: no setup
      is known that is sure to give a departure from the docking cell that gives up (S-0212
      claim 23), and without one a rest cannot be told from a retry that never comes. The
      chrono-miner exception is claim 17's (its backing there).
    - Confidence: high for the absence of a docking-cell exception in the harvest
      rules; medium for the rest after a stop order, pending H20 part e; medium for the rest
      after a give-up

3b. **Seeking after the final field track (Q-0900).** For an ordinary driving
    harvester, claim 3 tests the order-level destination alone. It does not test the
    locomotor destination point, either alone or in combination with that destination.
    Clearing the locomotor point at a qualifying track completion therefore does not
    itself permit a seeking search or start loading.

    Consider an uninterrupted arrival at the field cell named by the order-level
    destination: the harvester is not full, has no remembered cell or approach
    destinations, remains in *harvest*, owns an available dock-type building, and
    the arrival cell remains resource land. Let *T* be the final track's completion
    tick. S-0207 claims 25 and 26 retain that cell destination through completion
    and remove it in the following tick's locomotion update. Due harvest processing
    sees the destination before that tick's locomotion can remove it. The observable
    consequences are:

    | Processing time | Seeking result |
    |---|---|
    | In tick *T*, if due | No loading start; the destination is still present before completion. |
    | In tick *T* + 1, if due | Remains seeking, takes no field search and chooses one standard delay of *harvest*, including its one delay draw. The destination is removed later in this tick, not by this processing. |
    | First due processing in tick *T* + 2 or later | Finds its own resource cell and starts loading; the next processing is 1 tick later. |

    Thus the first processing strictly after arrival starts loading if it is in
    tick *T* + 2 or later. If that first processing is in tick *T* + 1, only the
    following processing starts loading. Arrival neither expires nor restarts the
    processing timer and causes no extra harvest processing. It is not an
    unconditional extra standard-delay wait on every arrival.

    F1a gives the ticks from completion to loading and the first load step. The
    first processing of the loading phase is 1 tick after loading starts, but
    it takes no load step: the first load-step-bearing processing is 19 ticks
    after loading starts (claim 5). These two events must not be conflated.
    - Backing: `measurable` (H27). Arrival-to-first-bale histogram frequencies
      are `unmeasured`: no arrival sample has been recorded, and distances,
      order times and shared-stream histories select different populations.
      F1a specifies conditional timings, not an assumed uniform distribution.
    - Confidence: high for the destination test and retained seeking; medium for
      the composed tick timing, pending H27

### Refineries

8. **Choosing a refinery.** A harvester chooses among the buildings its owner owns that are of one
   of its dock types, are on the map, are connected to it by its movement zone, and accept
   it (claim 10). With `quirks.non_unloading_refinery_accepts` off, buildings without
   the dock-unload flag are excluded from both the normal and waiting choices.
   No other eligibility, distance, primary-building or tie rule changes. *Connected* means that two cells lie in the same reach region (S-0212, claim 4)
   for the movement zone of the harvester's type, read for the deck when the harvester is on a
   bridge deck: the cell of the harvester's heading point (S-0212, Terms; its own position when it
   is not moving) and the cell of the refinery's reference point. A harvester whose heading-point
   cell lies outside the usable area counts as connected to every refinery. It takes the nearest candidate. The distance is measured as the square of the 2-D
   distance between the two reference points, an integer of leptons squared, *dx*² + *dy*², computed
   as a signed 32-bit integer that wraps by default (Formulas F5).
   `quirks.refinery_distance_wraps` defaults to this original rule. With it off, the squared
   distance is the exact non-negative integer *dx*² + *dy*², without wrapping or rounding.
   Only this distance changes: both choice levels, eligibility, primary-building overrides
   and tie rules below remain the same in either setting. A primary building can override
   the nearest rule. The choice is made at two levels by
   the same rule:
   - within each dock type, among the owner's candidates of that type, ordered by holding order
     (Terms); this gives the *best of the type*, when the type has a candidate,
   - across types, among the bests of the types, ordered by the dock list; the best of a type
     counts as a primary building when it is one.

   At each level: when no candidate is a primary building, the choice is the nearest, and a tie
   goes to the earliest. Otherwise let *P* be the last primary building of the level (for the
   across-types level, the last type whose best is a primary building, by that best); the choice is
   the nearest among *P* and everything after it, and a tie goes to the earliest of them, so *P*
   wins ties. The levels do not combine into one flat list: a type's best counts as a primary
   building only if it is one itself, also when the type has a primary building and a nearer
   candidate after it is the type's best (scenario H4 part i). In the normal choice a refinery whose dock slots are all held by
   other harvesters does not accept; in the *waiting choice* (claim 9) slots are not looked at.
   - Backing: `measurable` (scenario H4 parts a, c and i; part c's placing-order runs and part j
     for the holding order of produced and captured buildings; part e for the wrap, `unmeasured` if
     the original cannot load a map that large; part k for connected versus disconnected
     refineries). The heading-point/deck-specific connectivity cases and the outside-usable-area
     exception are `unmeasured`: part k does not distinguish them from a test at the current
     ground cell, and no legal-input setup reliably places a seeking harvester's heading point
     outside the usable area. These cases need a separate reachable boundary setup.
     Joining at placement rather than production start is also `unmeasured`:
     H4's same-type runs preserve both orders, while different-type ties are decided
     by dock-list order and cannot distinguish those joining times.
   - Confidence: high for nearest-first and the slot test; medium for the two-level rule, the
     primary-building rule, the holding order (Terms) and the tie order that rests on it, and for
     the connectivity cells; low for the usable-area exception

9. **Reserving, or driving to the queue.** In the *returning* phase, while the harvester has no
   destination, it makes the normal choice (claim 8). If there is a choice within the too-far
   distance, meaning a truncated distance of at most the limit, equality included, it asks that
   refinery for contact. The limit is the harvester-too-far field for ordinary harvesters and the
   chrono-harvester field for chrono miners, in cells, × 256 leptons; the distance is the 3-D
   distance between the reference points by S-0204's table square root
   (`quirks.table_root_distance`), truncated. If the refinery agrees, the harvester holds contact
   with it and goes *requesting*. Otherwise (no choice, too far, or refused) it makes the waiting choice:
   - none: it stays and tries again at its next processing,
   - an ordinary harvester at most 768 leptons (3 cells, equality included, by the same 3-D
     distance) from that refinery stays where it is and tries again at its next processing,
   - otherwise it drives to the cell that a nearby-cell search around the queueing cell gives, and
     tries again when it arrives; when the search gives no cell, its destination becomes the
     refinery building itself, as for a move onto that building.

   The search is S-0212's nearby-cell search (claim 25) with these settings: the centre is the
   queueing cell; the locomotion class is *wheel* and the movement zone *normal*, whatever the
   harvester's own; no reach region; never in deck mode; no height condition; no further test; and
   the reference cell is the refinery's top-left cell (without the queueing shift). So the result
   is the preferred collected cell nearest the top-left cell, the first collected on equal distance,
   and the queueing cell itself is used when it qualifies (it is ring 0). "Free" therefore means
   S-0212's qualification: inside the usable area, enterable by class for *wheel* with *normal*,
   and an occupancy record that is empty as a whole; it is not the entry outcome *clear*. With the
   default shift (0, 0) the queueing cell is a foundation cell of the refinery itself, which never
   qualifies, so the result comes from ring 1 or beyond: on open level ground, the cell
   (*x*₀, *y*₀ − 1) above the top-left cell.

   A chrono miner always moves in the last case, even at 768 leptons or less. It drives
   there (claim 17; S-0207 claim 28).
   While a harvester is driving in this phase it does not re-choose, except a chrono miner (claim
   17). The phase takes no draws other than the standard delay's; the search takes none.
   - Backing: `measurable` (scenario H4 parts a, b and f, H8 part f for the chrono miner, and H11
     parts a to c for the search settings and the fallback to the refinery)
   - Confidence: high for the too-far test and the queueing cell; medium for the 768-lepton wait,
     for the equality at both limits and for the search settings; low for the fallback to the
     refinery building

10. **Contact and docking.** A refinery accepts a harvester (agrees to contact, and counts as
    accepting in claim 8) only when it is not in its construction state, is not being sold, has a
    dock slot free or already held by that harvester (except in the waiting choice), and is not
    switched off (S-0405, claim 10). `quirks.non_unloading_refinery_accepts` defaults to
    the original: absence of the dock-unload flag adds no refusal to this refinery
    exchange. With the switch off, that absence also refuses the harvester's refinery
    contact request, including one made through an explicit enter order. This does not
    change other unit-entry exchanges, such as repair or absorption. Whether a refinery that is being erased also refuses is not
    determined (Open questions). Further refusals for other kinds of units are the structures
    specs'. A refinery whose art gives no dock count has one dock slot, so one
    harvester at a time holds contact with it and the others queue. The house's power level does
    not matter. In *requesting* the harvester
    queues *enter*. Each processing of *enter* is followed by the next after the standard delay of
    *enter* (S-0215, claim 6: one draw per processing). At each processing the harvester, in contact
    with a dock-unload refinery, asks to dock, and the result is the first matching row:

    | # | Situation at a processing of *enter* | Result |
    |---|---|---|
    | 1 | the refinery refuses (it does when it is switched off) and the harvester is not tethered | the harvester breaks off contact and falls back to rest (S-0215, claim 15) |
    | 2 | it does not stand on the docking cell | the docking cell is offered as its destination again at each such processing; ordinary driving uses S-0212's path and moves forwards, subject to the assignment qualification below |
    | 3 | it stands on the docking cell, and its body facing is not exactly 16384 (map east) | it and the refinery become tethered (glossary, *tether link*), and it turns its body on the spot towards 16384 |
    | 4 | it stands on the docking cell, facing exactly 16384, and not moving | it and the refinery are tethered (as in row 3), and the refinery queues *unload* on it, without an immediate start, unless the refinery is being sold; *unload* becomes current by S-0215's start rules (claim 11) |
    | 5 | otherwise (on the docking cell, facing 16384, still moving) | tethered as in row 3; nothing else |

    A tethered harvester whose refinery refuses (row 1's refusal while tethered) keeps its contact
    and goes on to the other rows.

    **Repeated destination offers.** Row 2 offers a destination, not an unconditional
    new movement point. S-0207 claim 27 owns the teleport/dock test: only an otherwise
    eligible destination assignment reaches it, when the destination changes or pending
    destination refresh permits an unchanged assignment. An unchanged offer without
    refresh is ignored before that test, even when contact or cell occupancy has changed, for
    every harvester. The dock test of the next sentence is not such an offer, so this does not
    stop it. A chrono miner also gets S-0207 claim 28's repeated dock test: at each processing at which the
    refinery accepts it, or at which it is tethered, its destination, if it has one, is tested,
    changed or not. So a chrono miner whose docking cell was occupied tests the cell again at each
    such processing, and, if its suspended predecessor is its teleporting locomotor, jumps once the
    cell is clear and it stands still (S-0207 claim 28, the occupied docking cell). Acquiring
    contact alone does not trigger the test. For a qualified chrono miner
    currently using non-teleporting locomotion, that claim
    determines restoration of a suspended predecessor versus fallback. On qualified successful
    restoration of teleporting locomotion, the restored movement receives that same assignment's
    otherwise due point, fixing its landing point under S-0207 claim 20. Eligibility uses the
    driving points already present, not the offered new point. Unless already-pending point
    suppression or that claim's disabling states prevent it, the miner jumps at its next teleporting
    movement update, possibly in the same tick. Restoration itself requires neither a preliminary
    driving leg nor a second offer, and enables neither refresh nor point suppression. The jump's
    own removal of the destination makes driving active again from the jump tick on (S-0207 claim
    28), so the miner unloads and leaves with driving locomotion.
    The non-restorable fallback stops locomotion, clears the current behaviour and queues *enter*
    without an immediate start. Normally that same assignment installs the order-level cell
    destination but suppresses its locomotor movement point, consuming point suppression while
    leaving newly enabled refresh pending. A later otherwise eligible equal offer can therefore test
    again: there is no once-per-entry limit. This fallback does not itself turn driving into
    teleporting locomotion.

    **Tag notifications.** Ordinary refinery arrival and docking in *enter* add neither an
    entered-by raise on the refinery's tag nor a destroyed-by-anything raise on the
    harvester's own tag (S-0524 claim 14). Cell tags encountered on the approach are
    separate; docking does not consume the harvester.

    So there is no backing in: the harvester arrives with whatever facing its path gives, and the
    turn of row 3 happens on the docking cell. The turn of row 3 aims at exactly 16384, so row 4
    holds at the first processing after the turn has ended; claim 11's facing range then holds at
    once. Through row 4, the *unload* is queued only then, once the body has stopped turning, so it does not wait for a
    turn; had the body still been turning, it would count as moving and the start would wait for the turn
    to end under S-0215 claim 11's tick-based readiness rule. Under the default quirk, a refinery that is not a dock-unload refinery and has
    no other applicable unit-entry exchange can agree to contact but never directs the
    harvester: it stays in *enter*, processed at every standard delay, until something
    else ends it (Edge cases).

    **Second route to unloading.** In the tick in which a driving harvester reaches its track
    target, a tethered harvester currently in *enter*, in contact with a building that occupies
    the neighbour in step direction 0 (map north) of its arrival cell, reports arrival to that
    building. Merely crossing into a cell during a track does not count. The intermediate cell
    of a two-cell curve does not count, nor does changing curves before the track target is reached.
    The arrival position has already been applied and the finished track cleared. The report's
    result is visible before any next track begins in that tick (S-0207 claim 12), and before
    the cell-centre crushing of S-0210 claims 3 and 8. The cell-boundary crush-slowdown effects
    of S-0210 claim 9 can therefore have occurred earlier, while the track was still running.

    For a dock-unload refinery without another building-type entry exchange taking precedence,
    the report queues *unload* without an immediate start, unless the refinery is currently in
    *selling*. This route requires neither facing exactly 16384 nor a stopped locomotor, nor
    being on the docking cell of Terms. It does not repeat claim 8's acceptance test: switching
    the refinery off after contact forms does not by itself refuse this report. Starting the
    queued *unload* still follows S-0215's start rules, and claim 11 still turns a harvester
    that is not facing map east before unloading starts. Building types that also absorb,
    repair, reload or otherwise take units inside belong to their entry-exchange specs, not
    to this refinery-only result.

    If the arrival report is refused, the harvester receives a vehicle scatter request
    (S-0207 claim 16), forced, without a threat point, and not move-anywhere. Neither the
    arrival report nor that scatter request takes a random draw: in particular there is no
    infantry direction draw. The usual vehicle refusals still apply, so a harvester with a
    destination keeps it and does not scatter. An accepting harvester takes the nearby-cell
    search result around its own arrival cell, since its track has ended; if that search finds
    no cell it gets no new destination. This refusal rule does not break contact or the tether,
    change the current *enter* behaviour or queue a behaviour. Other arrival effects and the
    refinery's sale can independently change those states. No behaviour processing or new
    standard delay is caused by this report.
    - Backing: `measurable` (scenario H4 parts a and d for one-at-a-time, the power switch and
      low power, part g for the construction state and for being sold, part h for being erased,
      H10 for the docking cell, the approach, the turn, the delays, the start of *unload*
      and the non-dock-unload contact stall, plus
      H14 for the second route, its ordering and draw count; S-0524 claim 14,
      scenario C16 for the absence of docking tag raises). The qualified chrono-miner
      docking assignment itself is reached by S-0207 scenario D34 parts c and e (below); the
      destination-refresh and point-suppression states it passes through are `unmeasured`: D34
      does not read them, and no recorder field shows them; H10 supplies conditional
      diagnostics for them, not a claim of unreachability. The selling-state
      arrival refusal and its destination-absent scatter case are `unmeasured`:
      H14 proposes timing controls but establishes no reliable overlap before
      the sale independently releases contact and tether.
    - Confidence: medium; high for the docking cell at (3, 1) from the top-left cell; low
      for being erased and for the refinery that is not a dock-unload refinery; high for the
      second route's track-completion event, the absence of facing and stopped tests and its
      scatter-request options, and for the refinery's repeated-offer rule (an unchanged offer
      without refresh is ignored); medium for same-assignment point suppression, which arises
      only on the qualified chrono-miner assignment path, whose suppression state is `unmeasured`
      (above); medium for its ordering and retained state.
    - The chrono miner's repeated dock test in *enter* and its driving from the jump tick on
      are S-0207 claim 28's, with the backing and confidence that claim gives them (medium).

10a. **A blocked docking cell does not expire the reservation (Q-0371).** Holding
    contact reserves a dock slot, not exclusive occupancy of the docking cell.
    The docking exchange of claim 10 directs the contacting harvester, not an
    unrelated friendly harvester standing on that cell. The exchange itself does not
    request that blocker to scatter; the contacting harvester's driving can (below). While the refinery continues accepting the contacting
    harvester, failure to reach the cell does not by itself release contact,
    send it to a queueing cell or select another refinery. There is no elapsed-tick
    limit in this docking rule: claim 10 row 2 governs the repeated offers of the
    docking destination, and this rule adds no limit to them. Other movement, orders, loss of contact
    and refusal rules can still end or change this situation.

    A harvester resting there without contact does not unload merely by occupying
    the cell; claims 10 and 11 still require contact. This spec supplies no automatic
    eviction of that resting harvester. General occupancy and scatter rules remain
    separate, and a vehicle factory's apron clearing is S-0417 claim 13, not a
    refinery-wide docking rule.

    **A second harvester at an occupied docking cell (Q-1870).** Those general rules decide what
    a contacting harvester does when a harvester rests on the docking cell. A chrono miner follows
    them too, except where a substitute is adopted (*A chrono miner*, below). While the
    cell gives that harvester the outcome *stationary friendly* (S-0209 claim 5), each offer
    of the cell by claim 10 row 2 that claim 10 does not ignore as unchanged leads to a path
    computation towards it, which waits for a running retry delay like any fresh search (S-0212,
    State and ordering), and S-0212 claim 24 decides by its distance *D*ᵤ to the cell (S-0212
    F6a):

    | Situation at the path computation | Result |
    |---|---|
    | *D*ᵤ is above the close-enough distance and a substitute is adopted | its destination becomes a free cell near the docking cell, the one nearest itself (S-0212 claims 24 and 25), and it drives there; the next offer of the docking cell is a changed destination and is tested again |
    | *D*ᵤ is at most the close-enough distance, or no substitute is adopted | when a search finds a path, its route leads onto the docking cell. Whenever a fresh search gives it a path whose first step enters the docking cell, the look at that first cell (S-0207 claim 15) sends the occupant a forced, move-anywhere scatter request with no threat point, and, because the harvester holds contact, that look never gives up the move. Such a search is the one made on the offer when the path begins with that step, or the one a first step check of that step makes on finding *stationary friendly*. The step check of that step follows (below). When a search finds no path, S-0212 claim 23 decides instead: the destination is cleared, and that claim's rows say whether the harvester also gives up |

    What follows the request depends on the occupant (S-0207 claim 16):

    | The occupant | Result for the contacting harvester |
    |---|---|
    | accepts, and its nearby-cell search gives it a cell | the occupant then has a destination and counts as moving (S-0212 claim 27). The step check that follows the look (on that tick when the harvester faces the step, otherwise once it has turned) takes the outcome S-0209 gives for that occupant (rows VP21 to VP23), and S-0212 claim 22 acts on it; if that gives up the move, the next offer repeats the sequence, as in the next row. Either way the harvester keeps its contact and reservation and docks by claim 10 once the cell is clear |
    | refuses, or accepts but its search gives no cell, so that it stays and the cell still gives *stationary friendly* | the step check that follows (on that tick when the harvester faces the step, otherwise once it has turned) makes the repeated check on its tick (after a first check, whose fresh search repeats the look and its request). When the give-up test (S-0207 F7, which asks nothing about contact at a step check) holds, the harvester gives up the move there: its destination is removed, and the next processing of *enter* offers the docking cell again, a changed destination, so the sequence repeats. When the test fails, it sends the request of S-0207 table T4 and waits. Either way it stays next to the docking cell with its contact and reservation and docks only once the occupant has left |

    A resting occupant in *guard*, standing still with no destination, accepts the request
    unless another of S-0207 claim 16's refusals holds (its type is a train type, its body is
    turning, its locomotor is teleporting or switched off, or it is deployed, deploying or
    undeploying), and drives to the nearby-cell result round its own cell. It stays in
    *guard* there and does not
    resume harvesting unless claim 13 or 17 makes it. An occupant that refuses includes a chrono
    miner whose teleporting locomotion is active (S-0207 claim 16, case b); such an occupant in
    *guard* leaves by itself when the refinery is its owner's, because claim 17 then queues
    *harvest* for it (claim 3a) and claim 3's search gives it a destination. When that search
    finds nothing, it goes *idle* instead, and leaves by itself only by claim 13's step off a
    refinery-flag building's cell, when it stands on one.

    *D*ᵤ counts the harvester's own world height in full, against the docking cell's centre at
    height 0 (`quirks.substitution_distance_flat_destination`). On a base raised so high that
    *D*ᵤ stays above the close-enough distance even at the cells next to the docking cell, which
    the harvester's world height alone exceeding that distance guarantees, every offer is
    substituted again, unless no substitute is adopted at it (then the second row of the
    first table applies). A harvester whose occupancy record holds it (S-0212 claim 24) then
    moves from one free cell next to the docking cell to another at its processings of *enter*,
    never gets a path onto the docking cell and so
    never sends the request, and keeps its contact and its reservation until the occupant is moved by other
    means. With the switch off the true distance decides, so height alone does not keep the
    harvester from the first table's second row. Neither case adds a docking timeout or another refinery choice.

    *A chrono miner.* An offer of the occupied cell is not a qualified docking assignment, so a
    chrono miner outside S-0207 claim 28's exclusions drives, with its teleporting locomotor
    suspended. A substitute that a path computation adopts in the first table's first row is
    different: it is a free cell given to the miner while it holds contact with the refinery, so
    it is a qualified docking assignment (S-0207 claims 27 and 28). It comes while the miner has a
    movement report, as its locomotor still holds the point of the docking cell it was just given,
    so it gets claim 27's fallback. The miner stops, finishing the track it is on, *enter* is
    queued again with refresh pending, and the substitute is not driven to. At the next processing
    of *enter* at which it stands still, the refinery offers the docking cell again, and a path
    computation that adopts a substitute stops it again. So:
    - while the cell stays occupied and each path computation adopts a substitute, the miner
      stands where the first one stopped it, which from far away is near where it was first
      offered the cell. It sends no request and keeps its contact and its reservation. On a base
      raised as above, it stands there until the occupant leaves, instead of moving between the
      free cells next to the docking cell.
    - where a path computation adopts no substitute (the first table's second row, including
      within the close-enough distance), it drives on the route it gets, and the look, the
      request, the step check and the give-up follow as for other harvesters (second table).
    - once the cell is clear, it jumps to the docking cell at the first processing of *enter* at
      which it stands still (S-0207 claim 28, *An occupied docking cell*), unless it has already
      reached the cell by driving.
    - Backing: `measurable` (H20; part e for the request, the occupant's acceptance, the kept
      contact and reservation and the docking, part f for the substituted offers on a raised base,
      part g for a chrono miner's stop at an adopted substitute, which is composed from S-0212
      claim 24's substitution and S-0207 claim 27's dock test and fallback),
      the step-check outcome after an acceptance resting on S-0209 rows VP21 to VP23,
      and `unmeasured` for the give-up and the repeated offers after a refusing or staying
      occupant: the only refusing resting occupant known is a chrono miner with teleporting
      active, which leaves the cell by claim 17, and no setup is known that keeps another
      refusal of S-0207 claim 16, or a failed search for the occupant, in force while the
      contacting harvester approaches.
    - Confidence: high for the docking exchange, retained reservation and absence of
      a docking timeout; medium for the resting-occupant case pending movement tests,
      and low for the request and for both of its results for the contacting harvester
      (the step-check outcome after an acceptance as S-0209 rows VP21 to VP23 give it), which
      rest on the look at the first cell of a new path, rated low in S-0207 claim 15
      (composed from the movement rules cited, not observed); medium for the substituted
      offers on a raised base, and for a chrono miner's stop at an adopted substitute and the
      standing that follows, until H20 part g runs

11. **Unloading.** A harvester counts as facing map east when its facing *f* (glossary) satisfies
    16256 ≤ *f* ≤ 16511. "The neighbouring cell" is the cell in step direction 6 (map west) of the
    harvester's cell. While it is in *unload* the harvester is in one of the situations below at
    each processing. *Unloading* means that the start row has been done; *finishing* is the state
    entered by the empty-load row. The animations are those of Terms.

    | Situation at a processing of *unload* | Result | Next processing |
    |---|---|---|
    | Not in contact with anything | falls back to rest (S-0215, claim 15, which queues nothing while *unload* is current); leaves the unloading state; stops if it is moving; then starts its queued behaviour by a start-next if it has one and is ready to change (S-0215, claim 11) | 1 tick |
    | In contact, not unloading, not facing map east | turns its body towards map east | 5 ticks |
    | In contact, not unloading, facing map east | starts unloading: step counter set to 0 and a pacing period of 1 tick started now; the building in the neighbouring cell, if any, plays its docking animation | standard delay of *unload* (S-0215, claim 6, one draw) |
    | Unloading, not finishing, and no building stands in the neighbouring cell | breaks off contact and queues *harvest* with an immediate start | — |
    | Unloading, not finishing, a building there, counter < 900 × *D* (the integer counter compared with the double product) | nothing | 1 tick |
    | Unloading, not finishing, a building there, counter ≥ 900 × *D*, load not empty | in this order: the smoke event (**Refinery smoke**, below); the building plays its payment animation unless that is already playing; the harvester is paid (claim 12) for the tiberium it holds with the lowest registration index, the whole amount at once; counter set to 0 (its period is not restarted) | 1 tick |
    | Unloading, not finishing, a building there, counter ≥ 900 × *D*, load empty | in this order: the smoke event; the building plays its payment animation unless that is already playing, as in the payment row; a refinery-flag building in the neighbouring cell plays its unloading animation; the building's payment animation is stopped. It becomes *finishing*; the counter is left as it is | 1 tick |
    | Finishing, and the building in the neighbouring cell is a refinery-flag building that is still playing its unloading animation | nothing | 1 tick |
    | Finishing, with a destination and a queued behaviour other than *harvest* | leaves the unloading state; stops if moving; starts the queued behaviour if it is ready to change; contact is kept | standard delay of the behaviour then current (one draw) |
    | Finishing, otherwise | leaves the unloading state; queues *harvest* without an immediate start; if it is ready to change, breaks off contact and starts the queued behaviour | standard delay of the behaviour then current (one draw) |

    **Refinery smoke.** Each payment row and the empty-load row that starts *finishing*
    produces one smoke event on the building in the neighbouring cell, as defined by
    S-0616 claim 1. The smoke event comes first in its row: before the row's animations start or
    stop and before payment is applied at that processing. Thus an uninterrupted
    unloading of *k* different tiberiums produces *k* + 1 events, including one when docking
    empty. Whether an event creates smoke systems is S-0616 claim 1's rule, independent of
    the docking, payment and unloading animations above.

    **Order within the two rows.** The objects that the smoke event and the row's own starts
    create in a payment row or the empty-load row enter play in this order: the smoke event's
    particle systems (S-0616 claim 1), then the payment animation if it was not already playing,
    then, in the empty-load row, the unloading animation. A start made in the condition opposite
    to the building's slot condition first makes S-0108 claim 12's swap of every running slot,
    whose new animations and draws come before that start's own, at that claim's ratings (the
    order of the swap's restarts and draws is medium there). Making the smoke systems takes no draw (S-0616 claim 1); each animation takes its
    creation draws (S-0109) at its start, so the payment animation's draws come before the
    unloading animation's. In the
    empty-load row the payment animation is always stopped last, so a payment animation that was
    not playing when that row came is started and removed in the same processing: it enters play
    with its creation draws and leaves play before it is ever processed or drawn. A building whose
    payment animation's chosen variant names an animation (Terms) plays it at a docking with an
    empty load in this way too.
    Stopping a slot takes no draw (S-0108 Terms).

    *The harvester's image while unloading.* While a harvester is in the unloading state (below),
    its body is drawn with the image of its type's deployed image type (S-0517 claim 12; the same
    imported field) when its type names one, and with its own image otherwise. The image changes
    with the start row's processing and changes back when the harvester leaves the unloading state
    (*Unloading state*, below). As in
    S-0517 claim 12, the deployed image type is used only for drawing the body: the harvester
    keeps its own type for every rule, and its own position, facing (map east from the start
    row), owner and light. A body drawn as a voxel model takes its frame from the harvester's own
    walk-animation counter (S-0105 claim 16), as any vehicle body does; the image type does not
    restart it. The harvester's selection box, health bar and other overlays are not part of the
    body and keep its own type. Unloading adds nothing else to the harvester's display: no
    sequence or frame change of its own, and no facing change once the turning row has brought it
    to map east; the harvesting sprite of claim 5a is not shown while unloading.

    While the harvester unloads, the step counter follows the schedule of claim 5 (Formulas F6)
    with a period of 1 tick. Unloading starts the period at its processing in tick *u*₀, so the
    counter rises in every tick from *u*₀ + 1 on, and before the first payment a processing in tick
    *t* sees the counter *t* − *u*₀ − 1. A payment sets the counter to 0 without restarting the
    period, so the rises go on in every tick, the payment's own tick included, and after a payment
    in tick *u* a processing in tick *t* sees *t* − *u* (Formulas F2).
    The processing after the start row comes *d* ticks later, *d* being the standard delay drawn at
    the start row; from that processing on the harvester is processed in every tick, so each
    payment, and the change to *finishing*, comes in the first tick, from then on, in which the
    counter has reached 900 × *D*. A harvester with an empty load becomes *finishing* only at a processing at which the
    counter has reached 900 × *D*, as for a payment, so after its last payment it waits
    max(1, ⌈900 × *D*⌉) ticks before it becomes *finishing*.

    So unloading takes one payment per tiberium carried, each max(1, ⌈900 × *D*⌉) ticks after the
    previous one, then one further such interval before *finishing*, and then the wait for the
    building's unloading animation (the finishing row above). That wait depends on whether a
    refinery-flag building in the neighbouring cell is playing an unloading animation when the
    *finishing* harvester is processed, not on whether the empty-load row started one: a start
    whose chosen variant names nothing leaves a running unloading animation in place (S-0108), so
    the harvester then waits for that one. When none is playing, the harvester takes its next
    *finishing* row in the tick after it became *finishing*. The docking and payment animations
    never delay the harvester. Back in *harvest*, the harvester starts seeking and drives to its
    remembered cell (claim 3).

    *Turning body.* In every row above that starts a queued behaviour "if ready to change", a body
    that is still turning counts as moving (S-0215, claim 11), unless one of that claim's exceptions
    applies: the queued behaviour is *enter*; the current behaviour is *guard*, or *attack* without a
    target; or the queued *guard* is one of the two unload rules that S-0215 gives to other specs.
    Where no exception applies, the unfinished turn blocks the start. For an unchanged body turn
    begun in tick *t*₀ with integer duration Θ ticks (S-0206 claim 1), turning has ended by tick
    number *t*₀ + Θ, including at that tick's first per-tick start. If a behaviour is already queued
    and no other readiness blocker or update exclusion remains, it becomes current at that first
    start and is processed in that same tick (S-0215 claim 11), not only at a late start or in the
    next tick. If another blocker clears later, S-0215 claim 11's ordinary start rules apply:
    turn expiry alone neither ends the unloading state nor bypasses that blocker. The same rule
    governs the starts in *Order while unloading* and *Refinery sold while unloading*.
    Backing: `measurable` by S-0215 claim 11's proposed scenario B47 for the isolated turning
    condition and same-tick processing. Confidence: high for that inherited rule. This spec
    proposes no separate scenario isolating an unloading start after a turn.

    *Unloading state.* From the start row until it leaves (the rows above, or a scatter), the
    harvester is in the unloading state, and a vehicle in that state is not ready to change (S-0215,
    claim 11, the "letting out" transition), so no queued behaviour starts by S-0215's per-tick start.

    *Refinery sold while unloading.* At the first processing of a sale of the refinery, the refinery
    tells every unit docked with it to leave (S-0408, claim 6). A harvester in the unloading state leaves it at
    once, queues *harvest* and starts it if it is ready to change
    (subject to any remaining readiness blocker, including an unfinished body turn; see
    *Turning body* for first-start eligibility on the turn's expiry tick), keeping the part of its load that has not been paid.
    In the same processing it receives a forced vehicle scatter request without a threat point
    (S-0207 claim 16), made only while it has no destination. If *harvest* could not start,
    the request is refused, since S-0207 claim 16 refuses a vehicle whose behaviour is *unload*,
    and the harvester gets no scatter destination. The sale then breaks off the
    contact at its next processing, 1 tick after the first, whether or not the harvester is still
    tethered (a docked unit holds back only the release and the build-down: Changes for other
    specs, S-0408 claim 6), so
    the scatter destination is assigned while the contact still holds, and no processing of the
    harvester after the sale's second one sees that contact. A harvester
    still in *enter* at that moment, in contact with the refinery (on the docking cell, or driving
    towards it under row 2 of claim 10), queues *guard*
    instead. If its destination is the refinery building itself, rather than a cell (a
    destination naming a building, S-0207 claim 26), that destination is removed first. Then,
    only if it has no destination, it receives the same kind of vehicle scatter request, which a
    turning body still refuses (S-0207 claim 16). A harvester whose destination is a cell, such
    as the docking cell, keeps that destination and receives no request. For a chrono miner such a
    scatter destination can be a qualified docking assignment, which restores its teleporting
    locomotion with the jump to the scatter cell pending (S-0207 claim 28, *A sale of the
    refinery*). An unloading chrono miner's *harvest*, started by the sale, is processed in its
    next turn, before its locomotion and while the sale's contact still holds, and that seeking
    processing decides whether the jump is made, and where a miner that is not full goes
    instead (claim 17, case (f)).

    *Refinery changing owner while unloading.* A harvester docked at a refinery that changes owner
    follows it (S-0509 claim 14) and has its orders reset (S-0309 claim 31). In the unloading state
    it is not ready to change, so the *guard* request of that change stays queued, and it goes on
    unloading by the rows above; each payment after the change goes to the new owner, and at
    *finishing* it queues *harvest* in place of *guard*. It has forgotten its remembered cell, so
    back in *harvest* it seeks from where it stands (claim 3). A harvester in *unload* that has not
    yet started unloading starts *guard* at the change, or once its body turn ends, and stays
    docked with its load unpaid (claim 13's rules for a harvester in *guard*).

    *Contact lost otherwise while unloading.* When the refinery leaves play without a sale (it is
    destroyed, for example) or otherwise breaks off contact with no run-away, the next processing
    takes the first row. With nothing queued, the fall-back queues nothing (*unload* is current)
    and nothing starts, so the harvester stays in *unload*, keeps its load and is processed every
    tick, taking the first row each time, until an order or another rule queues a behaviour
    (`quirks.unload_without_contact_stalls`, Edge cases). Such a loss of contact queues nothing on
    the harvester. This holds for every owner.

    *Order while unloading.* An order (a destination and a queued behaviour) given to a harvester
    that unloads at a dock-unload refinery breaks off its contact and its tether at once (S-0213,
    table T3, part b), so the refinery's dock slot is free in the same tick: another harvester may
    take contact with it from then on. The ordered behaviour does not start then, because the
    harvester is not ready to change. At the harvester's next processing of *unload*, which comes
    when the delay of its previous processing ends (1 tick while it is paid), the first row applies:
    the fall-back's row for a vehicle with a destination keeps the destination and queues nothing,
    the harvester leaves the unloading state and stops, and the ordered behaviour becomes current at
    once, unless its body is still turning and the behaviour is not *enter*; then it becomes current at
    the first eligible start under *Turning body*, including the first per-tick start on the
    turn's expiry tick when no other blocker remains. The harvester keeps the part of its load that has not been paid; it is not paid for it
    later unless it unloads again. The original also has a route by which an order ends unloading
    while contact is kept (the harvester then becomes *finishing* with its load kept, playing the
    unloading animation and stopping the payment animation as the empty-load row does but with no
    smoke event and no payment-animation start, and the row for a destination and a queued
    behaviour applies), but orders of S-0213 never reach it, since
    only a dock-unload refinery queues *unload* and an order breaks off that contact first.
    - Backing: `measurable` (scenario H1; its variant e for the three animations; H12 for the sale,
      its part a2 for the end of contact at the sale's second processing, contact lost while
      unloading, an order while unloading and the free dock slot; for
      refinery smoke, S-0616 claim 1, scenario P20).
      The order-ending route with contact kept is `unmeasured`: the orders in S-0213
      break this contact before that route could be observed, and no other reachable
      input is known to preserve it. The sale's scatter request refused when *harvest* could
      not start is `unmeasured`: in H12 part a2 *harvest* starts at once, and no set-up is known
      that holds a readiness blocker other than a body turn, which refuses the request itself.
      A cell destination kept with no request is `measurable` through H12 part a4. The removal of
      a destination naming the refinery is `unmeasured`: an explicit enter order gives a
      harvester that destination, but no set-up is known that keeps it while the harvester holds
      contact, since row 2 of claim 10 replaces it with the docking cell at the processings of
      *enter* that follow; H12 part a4 records it if a run shows it. A chrono miner's jump at the
      sale, to the scatter cell or to a field cell, is S-0207 claim 28's and claim 17's case (f)
      (S-0207 scenario D34 part h). A change of the refinery's owner while unloading is
      `measurable` by S-0309 scenario A19 part e; for a harvester that has not yet started
      unloading it is `unmeasured`, as S-0309 claim 31 gives. The order of the objects created in
      the payment and empty-load rows, the transient payment animation, the harvester's body image
      while unloading, and the normal animation of a damaged building whose art gives only the
      normal name are `measurable` by scenario H28. The frame of a body drawn as a voxel model and
      the overlays, the selection box and health bar included, are `unmeasured`: H28 records
      neither the harvester's walk-animation counter nor any overlay, and its deployed image type
      differs from the harvester's type only in its image, so a run could not tell from which of the
      two types an overlay is drawn. That the three starts allow no occupied variant (Terms) is
      `unmeasured`: no part here garrisons the building, so no run tells such a start from one that
      allows the occupied variant. The damaged forms when the art gives both names are
      `measurable` by H1 variant e's health control (distinguishable damaged versions of all three
      animations, falsified by any other selection than the damaged one exactly in the runs at or
      below the yellow threshold). The wait for an unloading animation that a start whose chosen
      variant names nothing left running is `unmeasured`: no part here keeps one unloading
      animation running into a later empty-load row. The payment animation's leaving play in the
      same processing, rather than before it is first drawn, is `unmeasured`: H28 part b captures
      frames and lists the objects that enter play, not those that leave it.
    - Confidence: high for the payment per tiberium, the dump threshold and the smoke events.
      Medium for the exact
      tick of each payment and the 1-tick intervals, for the extra interval before *finishing* and
      for the one-tick difference of the first payment; medium for the unloading animation being
      the one the harvester waits for and the other two not delaying it; high for the order of the
      smoke event and the animations within the payment and empty-load rows, for the transient
      payment animation's start, its creation draws and its never being drawn, for the damaged
      forms when the art gives both names (H1 variant e) and for the normal animation played when
      only the normal name is given (H28 part d); medium for the payment animation's leaving play
      in the same processing, for the three starts allowing no occupied variant and for the wait
      for an unloading animation left running (each `unmeasured`, above); high for the harvester's body being drawn with its type's deployed
      image type while unloading, medium for the drawing properties that come from that type and
      for the overlays keeping the harvester's own type; low for which building types play the
      docking animation (this claim says any building in the neighbouring cell); medium for the stall after contact is lost, for the order's route and for
      the delays of the two *finishing* rows; medium for the sale's run-away and the *guard* of a
      harvester still in *enter*; medium, until H12 part a2 runs, for the end of contact at the
      sale's second processing; medium, until H12 part a4 runs, for a cell destination kept with
      no request; medium for a change of the refinery's owner while unloading; medium for the removal of a destination naming the refinery (`unmeasured`,
      above); medium for the sale's
      scatter request being refused when
      *harvest* could not start; low for the route with contact kept, its animation starts and
      stops included

12. **What the house gains, and when.** Each payment for an amount *a* of tiberium *T* (value per
    bale *V*) adds to the balance of the refinery's owner:
    - the main payment: trunc(RZ₅₃(RZ₅₃(RZ₅₃(*V* × *M*) × *a*) + *B*)) − *B*, where *B* is the
      balance before (Formulas F3),
    - then, when *x* = RZ₂₄(RZ₅₃(*n*<sub>p</sub> × *b*) × *a*) is above 0, a bonus payment computed
      the same way with *x* in place of *a*.

    Credits go straight into the balance: no storage building is needed and none caps them. They
    arrive during the harvester's turn, so production lines see them in the same tick (S-0203,
    claim 8; its scenario O7 part A observes this, and it is not repeated here). The purifier count
    *n*<sub>p</sub> is that of the refinery's owner. In the original, a refinery owner that is not
    human-controlled also counts, in *n*<sub>p</sub>, a number of virtual purifiers taken from the
    rules' list at the position that its difficulty index selects. When the list is missing, or has no entry at
    that position, the original's result is undefined; our engine counts 0 virtual purifiers then
    (departure, importer warning). In our engine that rule never holds for an
    AI house (glossary, *AI house*); it does hold for the neutral house and the special house, which
    keep the original's rules. Those two houses have difficulty index 1 (S-0412, Terms) in every
    skirmish (S-0412, claim 5), so they
    take the list's second entry, whatever the lobby's computer difficulty; no difficulty row is
    applied to them. **Original only (note for AI designers):** the original's computer player
    earns that extra bonus; our AI can only match it by owning purifiers.
    - Backing: `measurable` (scenario H6; part c for the virtual purifiers of the original's
      computer player, part f for those of the neutral house, part e for the absence of a cap),
      `unmeasured` for the missing-list departure, whose original result is undefined
    - Confidence: high for the formulas and the absence of a cap; medium for when a purifier starts
      to count and for the list position of the neutral and special houses (it rests on S-0412
      claim 5's index, which is medium there)

### Exhausted fields, lost refineries and rest

13. **No field, no refinery.** A harvester in *harvest* whose owner owns no building of any of its
    dock types queues *guard* (with a 1-tick processing interval). A harvester that finds no
    acceptable cell within the long scan range, has no destination and no remembered cell goes
    *idle*: 105 ticks later, it moves off the cell of any refinery-flag building (or weed-flag
    building, Import mapping) it stands on and queues *guard*. Its destination is the result of
    S-0212's nearby-cell search with these settings: the centre is the cell of that building's
    reference point; the locomotion class is the harvester's own, *winged* counted as *track*; the
    movement zone is its type's own; the reach region is that of the centre cell for that zone,
    read for the deck when the harvester is on a bridge deck; deck mode when the harvester is on a
    bridge deck; no height condition; no further test; no reference cell, so the result is the
    preferred cell at position (*N* mod *n*) of S-0212 claim 25. When there is no result,
    its destination is nevertheless replaced: it becomes the off-map cell (0, 0), not the
    building, its current cell, or another search result (`quirks.harvester_failed_step_off_origin`,
    Edge cases). The search takes no random draws, including on failure. It still queues
    *guard* without an immediate start, and the next processing is scheduled with the standard
    delay of the current *harvest* behaviour, taking its one ranged draw (S-0215 claim 6).
    The failed search and this destination assignment do not themselves change body facing,
    contact or tether. The house's no-field record was already set when it went idle and
    is not changed by the step-off result. Subsequent movement and behaviour processing follow
    their ordinary rules; the failed search is not retried by this idle processing.
    In *guard*, the outcomes are:

    | Owner/rule condition | Outcome of this claim's guard rule |
    |---|---|
    | Human-controlled owner (including an AI house in our engine) | Remains where it is and does not resume harvesting by itself; the chrono-miner exception is claim 17. |
    | Computer-controlled owner in the original; owner has a dock-type refinery; its no-field record is unset | Queues *harvest* at each processing, with a 1-tick interval; this ends the processing as row 2 of S-0215 claim 24. |
    | Computer-controlled owner in the original; no dock-type refinery, or its no-field record is set | This claim ends nothing; processing continues under the general rules of S-0215 claim 24. |
    | Weeder | The only additive case, row 3 of S-0215 claim 24, belongs to the later weeder revision (Edge cases). |

    The no-field record belongs to the house. In the original it is set when any harvester of that
    house goes idle for lack of a field, regardless of owner, and is never cleared during the game.
    Once set, no harvester of that computer-controlled house resumes by itself, even if resources
    later appear. Our AI gets the resuming behaviour by ordering idle harvesters to harvest. In our
    engine, the neutral and special houses retain the original computer-controlled rule.
    - Backing: `measurable` (scenario H5; its parts C and D for the no-field record; H11 part d for
      the step-off search; H11 part e for failure, the destination replacement, the queued
      behaviour, retained state and draws). The no-field record suppressing another
      harvester's resumption is `unmeasured`: H5 D establishes no reliable second
      resting harvester after that record is set, without computer intervention.
    - Confidence: high for the guard results and the failed step-off destination, queueing and
      draw rules; medium for the 105-tick idle interval, for the record never being cleared,
      for the step-off search settings and for the retained state

### Variants

14. **Slave miner and slaves.** A slave miner does not load bales itself; its slaves do, and it
    acts as their refinery, whether it is a vehicle or deployed as a building.
    - **Wake-up rule** (owned here for S-0215: claim 24's vehicle row 1, the slave miner in *guard*,
      and claim 26's row 1, in *area guard*; both use the same rule). At a processing of *guard* or
      *area guard*, the slaves wake exactly when

      > The slaves wake under this rule only when the current tick is later than the sum of the kick delay and the current behaviour's start tick, and the group is at rest, and at least one of these alternatives holds: the owner is computer-controlled; the miner stands on tiberium land; or the current tick is later than the sum of the kick delay and the group's last-rest tick and the short-scan search finds a cell.

      Here *k* is the kick delay in ticks, *s*<sub>m</sub> is the current behaviour's start tick,
      *s*<sub>g</sub> is the group's last-rest tick, and *t* is the tick number. “Group at rest”
      means no slave has been sent out since the group last came together. The initial value
      of *s*<sub>g</sub> is 0 ticks, not the miner's creation tick (claim 21). The short-scan
      test is the ring search from the miner with the slave-miner short scan range.

      **Who takes the owner alternative (Q-1990).** In the original, an owner counts as
      computer-controlled when its house is not played by a person: every computer player of a
      skirmish, and, by the same rule, the neutral and special houses (`unmeasured`, low: no
      scenario here places a slave miner of those houses). For such an owner the wake needs only the
      kick test on *s*<sub>m</sub> and a group at rest: no rest-tick test, no scan and no land
      test. While the miner stays in the same *guard*, every later processing of *guard* that
      finds the group at rest wakes it again, so a computer player's miner far from any field
      wakes, seeks a field with the long scan and moves to a site (claim 22). A person's miner
      that rests away from tiberium land with no cell in its short scan never wakes through this
      rule and does not move without an order. Nothing else wakes it in *guard*: claim 13's
      guard rows do not apply to it (below), and the separate area-guard effect below needs
      *area guard*, which an idle miner without a destination does not get (claim 22).

      **Claim 13's guard rows and the slave miner.** At a processing of *guard*, this wake-up
      rule is tested first, and when it wakes the slaves the processing ends there. Claim 13's
      guard rows are tested only after a processing that did not wake, and only for a type with
      the harvester or the weeder flag (Terms): a slave miner is a type with an enslaves field
      and the two resource fields (Import mapping), and the harvester flag is a separate field.
      So for a slave miner whose type has neither flag, no row of claim 13 applies, for either
      owner: a computer-controlled owner's processing that does not wake the slaves queues
      nothing under claim 13, so claim 13 leaves its *guard* and *s*<sub>m</sub> as they were
      until the wake. A type that
      has both an enslaves field and the harvester flag gets claim 13's rows after each
      processing that did not wake, so for a computer-controlled owner with a dock-type refinery
      and an unset no-field record such a processing queues *harvest* (claim 13, row 2), and
      claim 22 then applies to that *harvest*.

      In our engine the owner alternative applies to units of the neutral and special houses
      and of a test-only computer-controlled player house. An AI house takes the
      human-controlled alternatives (glossary, *AI house*): the miner on tiberium land, or the
      rest-tick test with the short scan. So in a normal game no player house's idle
      miner wakes far from a field. DECIDED (autonomy, 2026-10-28): our AI reproduces the
      original computer player's behaviour with an ordinary order, a harvest order on a field
      cell, which finds a site, travels and deploys under claim 22. The person's miner keeps
      the original's behaviour.

      The sums are signed 32-bit integers that wrap around (Formulas F5): with a very large kick
      delay, such as the absent-key default 2147483647, any start tick of 1 or more makes the sum
      negative and the test pass, while a start tick of 0 leaves the sum at 2147483647 and the test
      fails. `quirks.slave_kick_delay_wraps` (default: the original). With the switch off, the sums
      are exact. This wake-up rule always requires *k* + *s*<sub>m</sub> < *t*, so with a kick
      delay near 2³¹ it then never wakes the slaves through this rule.

      **Separate area-guard effect.** S-0215 claim 26's “wakes its slaves anyway” effect requires
      only that the slave group be at rest: every other condition above is skipped, including
      both kick-delay tests, the owner condition, the miner's land type and the short scan.
      S-0215 claim 26 owns which outcomes receive that effect; it does not occur at every
      processing of *area guard*. Its scenario B31 distinguishes it from the wake-up rule above.
    - **Slaves.** A woken slave searches (ring search with the slave scan range, from itself),
      walks to the cell found, and loads one bale (S-0401, claim 3) at each successful processing of its
      *harvest*, whose loading interval is *h* ticks, *h* being its type's harvest-rate field.
      Claim 26 distinguishes this from group updates while seeking and returning. Its capacity
      is its type's storage field. When full, it remembers its cell and walks back to the miner
      (for a deployed miner, to the foundation cell (*W* − 1, ⌊*H*/2⌋) of it, counted from its
      top-left cell). When its cell is exhausted it searches again; when a search finds nothing it
      walks back with what it has.
    - **Delivery.** At a group update, a returning slave in the unloading cell with no destination
      is paid as in claim 12, to the miner's owner, for
      each tiberium it carries in turn (lowest index first, main payment then bonus), goes inside
      the miner, and starts the slave reload field's delay in ticks. At the first group update
      after that delay has elapsed, its health is restored to its strength and it is ready to
      leave again (claim 21). After all payments of a delivery, the master receives one
      refinery smoke event under S-0616 claim 1, only when at least one payment was for a
      positive amount of resource. It is one event per delivery, not per tiberium. A deployed
      master's building type determines whether smoke systems appear; an undeployed vehicle
      master makes no refinery smoke.
    - **Lost slaves.** A slave that is killed or lost starts the slave regeneration field's
      delay in ticks; a replacement is created inside the miner at the first group update after
      that delay has elapsed (claims 19 and 21).
    - The miner's initial slaves and their emergence are claims 19 to 21. Claim 22 gives the
      supported harvest-order and automatic-deploy outcomes. Claim 24 fixes site selection, and
      claim 25 gives relocation-failure inputs and the immediate post-pack-up destination.
    - Backing: `measurable` (scenario H7; its variant b for the kick wrap and its switch, variant e
      for the group's rest tick, variant f for the deployed miner and the slaves' health,
      variants d and g for a person's and a computer player's miner far from a field, part g
      also for claim 13's rows not applying to a miner type with neither flag, and
      S-0215 scenario B31 for the separate area-guard effect; S-0616 claim 1, scenario P20
      for refinery smoke). `unmeasured` for the owner alternative of the neutral and special
      houses: no scenario here gives a slave miner to those houses. `unmeasured` for a type with
      both an enslaves field and the harvester flag: no scenario here uses such a type, and what
      follows the queued *harvest* rests on claim 22's path for that type
    - Confidence: high for the smoke events. Medium for the slave cycle and the wake-up rule,
      including the owner alternative for a skirmish computer player, until H7 parts (d) and
      (g) run. Low for the owner alternative of the neutral
      and special houses (`unmeasured`, above). Low for the miner's own movement. Medium, as
      for the wake-up rule until H7 (g) runs, that the wake-up rule is tested first and that
      claim 13's rows do not apply to a slave miner type with neither the harvester nor the
      weeder flag. Medium for a type with both an enslaves field and the harvester flag
      (`unmeasured`, above)

15. **Loads of several tiberiums.** A harvester may carry bales of several tiberiums; each is
    paid separately (claim 11). Only the tiberiums with registration indices 0 to 3 can be held. In the
    original, a load step on a cell of a tiberium with registration index 4 or more has an
    undefined result. Departure: our engine holds an amount for every tiberium, and the
    importer warns when the rules create more than four tiberiums.
    - Backing: `measurable` for two tiberiums (scenario H6); `unmeasured` for the departure, whose
      original behaviour is undefined
    - Confidence: high

16. **Fullness is counted in whole bales.** The amounts of a load are whole numbers of bales
    (Terms), so its total is their exact sum, and every load step takes exactly one bale while the
    harvester is not full. The harvester is full when its fill ratio total / *C*, a quotient in
    double precision, equals 1.0 (Formulas F4). For *C* ≥ 1 that is exactly when the total equals
    *C*.
    - Backing: `measurable` (scenario H1)
    - Confidence: high

17. **Chrono miner.** A chrono miner follows the cycle with these differences:
    - its too-far distance is the chrono-harvester field (claim 9), and when it does not reserve a
      refinery and the waiting choice finds one, it always moves to that refinery's queueing cell
      (claim 9), and it drives there (S-0207 claim 28),
    - while it is moving in the *returning* phase and a refinery would accept it (the normal
      choice of claim 8 finds one), it drops its movement, so that its next processing reserves it,
    - at a seeking processing it drops its destination and runs the ring search from where it
      stands (the *seeking drop*, below). This rule holds for a harvester
      whose current locomotor is the teleport locomotor, whether or not its type has the teleporter
      flag, and not for one whose type has the flag but that moves by another locomotor. A chrono
      miner reaches it only while its teleporting locomotion is still active at the seeking
      test (*Who reaches the drop*, below),
    - in *guard*, a chrono miner queues *harvest* again (1-tick interval, ending the processing as
      a case of row 2 of S-0215 claim 24's vehicle table; the claim 13 note on row 3 does not
      apply to it) when one of its eight
      neighbouring cells holds a building of its owner whose type has the refinery flag (Import
      mapping; the dock list is not consulted here), or when it is full and still moving. In the
      original this rule is reached only for a human-controlled owner (a computer-controlled owner
      gets claim 13's rule instead); in our engine it applies to every player house.

    **The seeking drop (Q-1130).** Here and in the rest of this claim's drop rules, "the miner"
    means a harvester of a type with the teleporting locomotor and without the teleporter flag,
    such as H8's type *Q*: the jumps these rules describe are its jumps only. A chrono miner
    meets the drop only in the cases of *Who reaches the drop*, below. The drop is not tied to
    the first processing after the miner enters *harvest*; it applies at every seeking processing that gets past the *full* test
    (claim 3's first seeking row). At such a processing the destination the miner had before the
    ring search is removed, whatever its origin. A remembered cell counts as such a destination: it
    is adopted by an ordinary destination assignment (which can fix a landing point under S-0207
    claim 20 a, unless that claim's disabling states or pending point suppression (S-0207 claim 27)
    prevent it) and removed at once, so the miner never jumps to it. Removing a destination stops the
    teleporting locomotion, which cancels the pending jump and the moving state. The ring search of
    claim 4 then runs from the miner's own cell. Any destination that search sets is jumped to in the
    same turn (claim 3b; S-0207 claim 20 d), so the miner has no destination after the processing
    unless that jump is blocked.

    The drop therefore acts at each of these events:

    | Seeking event | Dropped? | Consequence |
    |---|---|---|
    | full | no | goes *returning* (the returning drop is the second bullet above) |
    | not full, remembered cell | yes, the adopted cell | the miner never jumps to the remembered cell; it searches from where it stands |
    | not full, no remembered cell, destination present at the processing | yes | the pending jump, if the miner has not yet made it, is cancelled; it searches from where it stands |
    | not full, nothing remembered, no destination | nothing to drop | the search runs |
    | a ring search sent it to another cell at an earlier processing | not at that processing: the destination is new | see below: the next processing finds no destination if the jump was made |

    The drop clears the **order-level destination** and cancels the **pending jump**. It does
    **not** clear the **fixed landing point** or its occupancy reservation: whatever landing point
    S-0207 claim 20 a had fixed (for the adopted cell, or for a harvest order's cell) stays
    remembered, and the vehicle-present reservation stays at that point's cell until the
    locomotor takes its next destination (S-0207 claim 20 a: clearing an earlier reservation uses
    the remembered point). A miner that then starts loading at its own cell therefore keeps a
    reservation at a cell it has not moved to. When no landing point was fixed by the
    assignment (the miner was disabled or suppression was pending), there is none to retain.

    Within one miner's turn, a due behaviour processing sees the destination state before that
    turn's locomotion changes it (claim 3b), and a destination that a behaviour processing sets is
    jumped to in the same turn (S-0207 claim 20 d). Hence, for the miner as defined above:
    - a destination the ring search sets is jumped to in that same turn and removed by the jump,
      so the next seeking processing, a standard delay later, finds none; a search that sent the
      miner away produces a drop at the next processing only if the jump was blocked (the miner
      disabled by an electromagnetic pulse, *held*, or *being erased*), in which case the pending
      jump is cancelled and the search runs again from where it stands.
    - the remembered-cell event occurs at the first seeking processing after each unloading in which
      the miner holds a remembered cell (the short-scan result of claim 6), and that adopted cell is
      always dropped.
    - the "destination present" event is reached for a harvest order on a cell when the
      order's destination is still unfulfilled at the miner's first seeking processing, that is
      when no locomotor update jumped it first. Whether the order's destination is dropped before
      or after the first jump depends on whether the order starts *harvest* so that a processing is
      due in the first turn after the order (settled by scenario H8 part h; see the confidence below),
      with the
      processing first, the miner never jumps and loads at the cell the search finds from where it
      stands (as in part d); with the jump first, the destination is already gone.
    The "remains seeking" row of claim 3 (a destination present, no search) is therefore not
    reached by a miner with active teleporting locomotion in ordinary play, except through the
    blocked-jump case. It remains the rule for every harvester with other active locomotion,
    including a chrono miner in ordinary play (S-0207 claim 28) and a vehicle-factory-built
    miner on its temporary driving leg.
    - Backing: `measurable` (scenario H8 parts d, h and i, with the flag-off type *Q*); a chrono
      miner's cases have the backing given under *Who reaches the drop*, below
    - Confidence: high that, for a
      harvester whose teleporting locomotion is active, every seeking processing past the *full*
      test drops a destination; medium that the order-level destination is cleared and the
      pending jump cancelled (S-0207 claim 20's rule for an empty assignment while a jump is
      pending, which that claim rates medium) and for the retained landing point and reservation
      (it composes this with S-0207 claim 20 a and the locomotor's stop); low for the
      order-to-first-processing timing of a harvest order

    **Who reaches the drop (Q-1759).** The seeking drop applies when teleporting locomotion
    is still active at the seeking test. Which locomotor is active follows from the destination
    assignments, under S-0207 claim 28. A chrono miner's docking jump installs driving, which
    stays active through the docking and the unloading. Outside the contact of a sale (case (f)
    below), adopting a remembered cell, a harvest order and every other destination assignment
    install driving before the drop's test. A seeking processing begins with teleporting
    locomotion active in seven cases, listed below: a harvester type without the teleporter flag,
    and six chrono-miner cases, (a) to (f). The table after the list gives, for each of (a) to
    (f), the destination held at the seeking processing and the outcome.
    - Only case (e), warp-pending immobilization, meets the drop as the flag-off type does, with
      teleporting locomotion active at the test and after it.
    - In cases (a) to (d), adopting a remembered cell installs driving before the test, so the
      cell is not dropped. A held destination, possible only in case (a), is removed by the
      drop's empty assignment, which installs driving first. Otherwise there is nothing to drop.
    - In case (f) the sale's contact still holds at the test, so adopting a remembered cell is a
      qualified docking assignment and teleporting stays active. The drop removes the held cell
      by an empty assignment, which installs driving, and a destination that the ring search sets,
      again a qualified docking assignment while the contact holds, restores teleporting: the
      chrono miner jumps to that cell in the same turn. When a ground-layer vehicle stands in the
      remembered cell, adopting it is not qualified and installs driving, so nothing is dropped,
      no ring search follows, and the miner drives to its remembered cell.
    - Of the six, cases (c) and (f) have known ordinary-play fixtures. In case (c) the chrono
      miner holds no destination.

    So a chrono miner in ordinary play either drives at a seeking processing or has teleporting
    active there with no destination, except in case (f), where it holds the scatter cell with
    the jump pending. In both cases it drives to the cell the processing sets, except in case
    (f), where it jumps to that cell while the sale's contact holds.
    The seven cases are:
    - the flag-off type: a harvester type with the teleporting locomotor and without the teleporter flag, to which
      S-0207 claim 28 does not apply,
    - (a) *temporary driving ended*: a chrono miner whose temporary driving S-0607 claim 4 ended (a magnetic-beam hit while it
      drives, which seizes nothing), before its next destination assignment (S-0207 claim 28,
      the window after that restoration),
    - (b) *at rest after a seizure*: a chrono miner at rest after the restoration that ends its seizure (S-0607 claim 20),
      before its next destination assignment; in the original a computer-controlled house's
      restored miner then queues *harvest* (S-0607's restoration paragraph), so it can reach a
      seeking processing in that window (S-0607 says our AI houses use the human-house rule
      there, claim 20),
    - (c) *at rest after a fall-back*: a chrono miner at rest after S-0215 claim 13's restoration, before its next destination
      assignment (S-0207 claim 28, the at-rest window). A fall-back to rest restores
      teleporting when the chrono miner stands still with no destination and driving installed
      over its teleporting locomotor, and S-0417 claim 5's restoration conditions hold (no
      restoration lock, not seized, no driving movement report). This happens in ordinary play
      once a player's move order has ended, and after its docking jump when the refinery refuses
      it or the contact ends before it docks. After a move order the fall-back queues *harvest*
      or *guard* (S-0215 claim 15 row 4), and its *guard* rule above then queues *harvest* when an
      own refinery-flag building is next to it, so it reaches a seeking processing in this window
      (S-0207 scenario D34 part i). A refinery switched off after the docking jump
      and before the chrono miner's next processing of *enter*, while it is not yet tethered,
      refuses its dock request at that processing (claim 10 row 1; S-0207 claim 28
      and its scenario D34 part g), so that fall-back is reachable. A sale of the refinery
      leads to this window too, but only after the sale's contact has ended: while the contact
      holds, the sale's scatter request (claim 11) is a qualified docking assignment, and a
      chrono miner standing still has teleporting restored with the jump to the scatter cell
      pending. One still in *enter* makes that jump, which installs driving again (S-0207 claim
      28, *A sale of the refinery*). A chrono miner still in *enter* has *guard*
      queued by the sale (claim 11), whatever came of the sale's scatter request. It falls back
      to rest at a processing of *enter* if one comes before that *guard* starts: at the first
      one if it is not tethered, since a refinery being sold refuses it (claim 10 row 1), or
      otherwise at the first one after the sale has broken off its tether and contact. That
      fall-back restores teleporting when it stands still with no destination. The queued
      *guard* starts at the first per-tick start at which the miner is ready to change (S-0215
      claim 11), for example after its jump, or once a refused turning body's turn has ended. A
      processing of *enter* comes only when its processing timer is due (S-0215 claim 2), so
      which of the two comes first depends on that timer. If the *guard* starts first, the miner
      makes no fall-back from *enter*: its behaviour becomes *guard*, driving stays installed,
      and this window is not reached that way. S-0207 scenario D34 part h records which comes
      first. A miner that was unloading has *harvest* current instead and does not fall back:
      it is case (f),
    - (d) *placed with no destination assignment*: a chrono miner whose placement made no destination assignment, before its first
      destination assignment (S-0207 claim 28, the third window). A refinery's free miner is
      not one: its placement fall-back clears its destination before it is given *harvest*
      (claim 18), and that clear installs driving (S-0207 claim 28, the placement bullet),
    - (e) *warp-pending*: a chrono miner with warp-pending immobilization, one of S-0207 claim 28's exclusions,
      whose current locomotor is teleporting,
    - (f) *unloading at a sale*: a chrono miner that was unloading when its refinery was sold, at
      the first seeking processing of the *harvest* that the sale started (claim 11), when the
      sale's scatter request has restored teleporting with the jump to the scatter cell pending.
      (When the request was refused or assigned nothing, driving stays installed and this is not
      a case of the list. The cells its processings set then follow S-0207 claim 28's rule, under
      which they too are qualified docking assignments while the sale's contact holds.) Since a
      behaviour start sets its processing timer to expire at once (S-0215 claim 3), that
      processing comes in the miner's next turn, before its locomotion (S-0207 claim 20 d): in the
      sale's tick when the miner's turn follows the refinery's, otherwise in the next tick, before
      the refinery's turn. Either way it comes before the sale's next processing, 1 tick after the
      first (S-0408 claim 6). The sale breaks off the contact at that next processing (claim 11),
      so the contact still holds at the miner's seeking processing and has ended by its next
      processing.

    The other two of S-0207 claim 28's exclusions, a gripped and a seized chrono miner, are one
    case here, since a gripped vehicle is always seized (glossary). A chrono miner whose own
    locomotor is the teleporting one has, while it drives, that locomotor temporarily replaced by
    driving (S-0207 claim 28), so a magnetic-beam hit on it while it
    is not gripped is S-0607 claim 4's case if its type is a resource-gathering vehicle (S-0607
    claim 4's condition): that claim ends its temporary driving or stops it, it is not seized,
    and when its temporary driving ends, the window above follows. For such a type, a hit seizes it only where S-0607 claim 4
    does not apply, which here means while its teleporting locomotion is active, in one of the
    windows above. A chrono miner seized with the jumpjet stand-in has that as its
    current locomotor (S-0607 claim 5; a seize whose warhead names another locomotor, S-0607
    claims 22 and 23, is open: Open questions), so it is not teleporting. The restoration that
    ends its seizure brings back its teleporting locomotor at rest (the bullet above).

    A chrono miner whose own locomotor is the driving one (so nothing is suspended) has nothing
    installed or replaced at any assignment (S-0207 claim 28: when the current locomotor is
    already driving, nothing changes).
    None of the windows above arises for it, and its qualified docking assignments get S-0207 claim
    27's fallback, as for a vehicle with no suspended predecessor (S-0207 claim 28, an occupied
    docking cell). A magnetic-beam hit on it is not S-0607 claim 4's case, since nothing is
    temporarily replaced; S-0607's other claims decide it.

    Cases (a) to (f) meet a seeking processing as follows. The drop removes a destination by an
    empty destination assignment.

    | Case | Destination held at the seeking processing | Outcome |
    |---|---|---|
    | (a) temporary driving ended by S-0607 claim 4 | may still hold the destination of the trip it was driving (S-0607 claim 4 does not clear it) | a held destination: the drop's empty assignment installs driving first (S-0207 claim 28: empty assignments included), and driving receives it, so the miner stops there; held or not, it then goes on as in the next row |
    | (b) at rest after the restoration that ends its seizure | none | the ring search's destination is an ordinary assignment, which installs driving first (S-0207 claim 28): it drives there and is a driving chrono miner from then on |
    | (c) at rest after S-0215 claim 13's restoration (after a move order, a refusal on the docking cell, or the end of its contact, a sale's included) | none | as the row above |
    | (d) placed with no destination assignment | none | as the row above |
    | (e) warp-pending immobilization | may hold one | a held destination, or an adopted remembered cell, is removed at once as for the flag-off type *Q* (H8), which stops its teleporting locomotion; claim 28 installs nothing for it, so the ring search's destination goes to its teleporting locomotion, and what that locomotion then does with it is open |
    | (f) unloading at a sale | the scatter cell, with the jump to it pending | full: the *full* row of claim 3 comes first, so nothing is dropped or assigned; the pending jump to the scatter cell is made in that turn, and the miner goes *returning*. Not full: a remembered cell is adopted and kept by teleporting (a qualified docking assignment while the contact holds), unless a ground-layer vehicle is in it, in which case driving takes it, which by our engine's rule (S-0207 claim 21) cancels the pending scatter jump, nothing is dropped or searched and the miner drives there; otherwise the drop's empty assignment then installs driving, which by the same rule cancels it, so no jump to the scatter cell or the remembered cell is made; a destination that the ring search sets restores teleporting and the miner jumps to it in that turn, unless a ground-layer vehicle is in that cell, in which case driving takes it and the miner drives there; when the search finds nothing the miner goes *idle* with driving installed and no destination |

    In cases (a) to (d) the chrono miner makes no jump to a held destination before driving is
    installed: a restored teleporting locomotion has no pending jump, since by our engine's rule
    installing driving cancels a pending one (S-0207 claim 21; claim 28, the window after S-0607
    claim 4's ending).
    A remembered cell's adoption is an ordinary assignment too and
    installs driving before the drop's test, so in those cases it is not dropped and the chrono
    miner drives to it. Installing driving leaves the chrono miner's behaviour as it is: the
    reset of the behaviour that S-0215 claim 9 gives a change of locomotor is not applied to it.
    This rule is S-0207 claim 28's, `measurable` at the docking jump in S-0207 scenario D34
    part c and rated medium there. S-0215 claim 9 lacks this exception; S-0207 lists the
    amendment under its Cross-spec changes (its Q-1759 entry on
    S-0215 claim 9).

    In case (e) the drop of the seeking events table above applies unchanged, the adopted
    remembered cell included: only what the immobilized teleporting locomotion does with the
    ring search's destination, before or after the superweapon warp, is not established (Open
    questions).

    A chrono miner seized with the jumpjet stand-in is not among those cases. A seized unit is
    put in *guard* (S-0607 claim 5), and a computer-controlled house's seized resource gatherer
    gets no *hunt* (S-0607 claim 6), which changes nothing here. Whether the chrono miner's *guard* rule above (an own
    refinery-flag building next to it, or full and still moving), or, in the original under a
    computer-controlled owner, claim 13's *guard* rule can queue *harvest* while the chrono miner is
    seized is open (Open questions). If a seeking processing runs while it is seized, that processing gives it no empty destination: the drop
    needs teleporting locomotion, and the ring search and a remembered cell give only cells. It
    ignores those, as a seized unit ignores every new destination (S-0607 claim 9; the one
    exception there is a rule that gives a gripped unit no destination), so the processing
    neither ends its grip nor makes it travel.

    Outside those cases, a chrono miner that is driving takes claim 3's rows for other active
    locomotion: it drives to a remembered cell, to a harvest order's cell and to the cells the
    ring search sets.
    - Backing: `measurable` (S-0207 scenario D34, with H8 part j) for a chrono miner driving at a
      seeking processing and to every cell this claim sends it to. `measurable` through S-0207
      scenario D34 part i for the rest after S-0215 claim 13's restoration that follows a move
      order: teleporting active at rest, then a driven trip to the cell its seeking processing
      sets. `measurable` through D34 part h for the jump to a sale's scatter cell of a miner in
      *enter* and the fall-back that follows the sale's end of contact, for the queued *guard*
      that starts before any fall-back from *enter*, and for case (f): run 1 for a miner that is
      not full, which jumps to a field cell, and run 4 for a full one, which jumps to the scatter
      cell. `unmeasured` for case (f) with a ground-layer vehicle in its remembered cell or in the
      ring search's cell, or with no cell found, and for what a later restoration of its
      teleporting locomotion does after that: D34's map has a free field in range, and no
      recorder field shows the suspended teleporting locomotion. `unmeasured` for the other four
      chrono-miner cases, their consequences, the chrono miner whose own locomotor is driving and
      the seized chrono miner's seeking processing: no ordinary-play fixture is known that brings a chrono
      miner to a seeking processing in one of those states, and H8 tests the drop with the
      flag-off type *Q*.
    - Confidence: medium for a chrono miner driving at, or driving from, a seeking processing in
      ordinary play (S-0207 claim 28's rating until D34 runs), the at-rest case after S-0215
      claim 13's restoration and the sale's scatter jump of a miner in *enter* included; medium,
      until D34 part h runs, for case (f)'s jump to a field cell and to the scatter cell, and for
      the timing that puts its seeking processing inside the sale's contact; low for case (f) with
      a vehicle in its remembered cell or in the search's cell, or no cell found, and for what
      follows; medium for the sale's
      fall-back and for the queued *guard* that can start before it; low for the other four cases,
      their consequences, the chrono miner whose own locomotor is driving and the seized chrono
      miner.

    **Temporary driving after production.** A vehicle-factory-built chrono miner still using
    temporary driving drives its first leg to ore found in another cell. The ore destination
    replaces its retained factory-exit point. Queueing or starting *harvest*, arrival at that
    first ore cell, and starting loading do not themselves restore teleporting locomotion.
    S-0417 claim 16 owns the exit point's lifetime, its replacement and clearing, and later
    eligible restoration. A qualified refinery docking assignment can separately restore
    teleporting locomotion and supply that same offer to it (claim 10; S-0207 claim 27).
    A refinery's free unit follows claim 18's distinct release path, not the vehicle-factory
    exit; its travel is not evidence that a factory-built miner skips this first driving leg.
    A free chrono miner has driving installed at its placement (S-0207 claim 28), so its trips
    are driven too. Whether or not a miner has docked since it was made, in ordinary play every
    trip other than its docking jump and its jump at a sale (claim 11; case (f) above) is driven,
    outside S-0207 claim 28's exclusions: only a qualified docking assignment (S-0207 claim 27)
    installs no driving.

    How it teleports while teleporting locomotion is active is S-0207 claim 20's.
    A vehicle whose type has the harvester flag does not
    phase in after a jump (S-0207 claim 20 f).
    - Backing: `measurable` (scenario H8, parts d and j and the queueing-cell runs, and S-0207
      scenario D34 for driving on every trip but the docking jump; S-0207 scenario D22 run 2
      for no phasing in; H8 part g
      for the "full and still moving" trigger, `unmeasured` if no run reaches it, and
      S-0417 claim 16, scenario U15 for the factory-built first field leg, point replacement,
      clearing and later restoration; claim 10's conditional H10 trace for docking restoration)
    - Confidence: medium, including the composed factory-built first field leg and later
      restoration; high for the active-locomotion seeking test, harvest queueing not itself
      restoring locomotion, and no phasing in. Medium for S-0207 claim 28's rule at an assignment,
      the level that claim gives it until its scenario D34 runs; medium for that claim's composed
      ordinary-play consequences, among them that a chrono miner drives on every trip except its
      docking jump and its jump at a sale. Low for the "full and still moving" trigger

### A building's free unit

18. **When and where a free unit appears.** A building whose type has a free unit (Terms) gets
    one at its grand opening (Terms), subject to the exclusions below. For a building constructed
    in play, this is at the end of its build-up, in the building's own turn. It gets none:
    - when it changes owner (an owner change gives a building that has not had its grand opening
      that opening, with no free unit, and a building has one grand opening at most, so none
      follows at the end of its build-up, S-0309 claim 31),
    - when the map pre-places it: the grand opening runs during set-up placement, but its free-unit
      grant is suppressed then (S-0009 claim 40). This does not suppress the opening itself or its
      readiness effect. Set-up queues *guard*, and it becomes current at the first per-tick start
      (S-0215 claim 1),
    - when its owner is a player house, the building was produced, and its start price (Terms) is
      not 0 and not above the adjusted cost of its type (S-0408, claim 2, *C*), both integers in
      credits. A building that was not produced has no start price, and this condition does not
      apply to it. The price of a building type with a free unit includes the price of the free unit
      (S-0404, claim 2: *P* = own part + pad term + *P*ₑ, the own part being formed from *C*), so
      with neutral factors the start price is above *C* by *P*ₑ and the unit appears. The
      condition withholds it only when the house's factors or cost bonuses bring the whole price
      down to *C* or below, for example a vehicle cost factor so low that *P*ₑ is 0. In the original the
      condition applies only to a human-controlled owner; in our engine it applies to every player
      house, AI houses included (glossary, *AI house*), and never to the neutral and special houses.

    For this condition, a produced building retains its integer start price through completion,
    placement and build-up. A later gain or loss of a cost-bonus building does not replace that
    price with the current price. This record is required regardless of the cancellation-refund
    setting in S-0404. The comparison uses the type's adjusted cost, not the current production
    price or the amount paid so far.

    A building that a deploying vehicle creates goes through its build-up and gets its free unit at
    the grand opening, with no start price.

    The free unit is a new vehicle of the free unit's type, owned by the building's owner. It is put
    on the map at the first of these places that accepts it:
    1. the centre of the cell next to the cell of the building's reference point in step direction 4
       (map south), with body facing 49152 (map west),
    2. the cell given by S-0212's nearby-cell search with the settings below and the further
       condition that the cell holds no overlay of any kind (glossary, *overlay*: resources included), with body facing 40960 (map
       south-west),
    3. the cell given by the same search without the overlay condition, facing 40960.

    The settings of both searches: the centre is the building's top-left cell (Terms); the
    locomotion class is *wheel*, whatever the free unit's own; the movement zone is that of the free
    unit's type; the reach region is that of the centre cell for that zone, read for the ground,
    never in deck mode; the height condition applies; no further test; bridge cells never qualify,
    and the reference cell is the top-left cell, so the result is the nearest preferred cell to it.
    A place accepts the unit when the unit's entry outcome for that cell is *clear* (S-0209); a
    search with no result offers no place. When no place accepts it, the unit is removed again and
    the owner receives the refund of S-0408 claim 1 for the free unit's type with the refund
    percentage taken as 1 (that is, the amount *P*, or the refund-override form, of that claim).

    The free unit is placed before it is given any behaviour, so its placement fall-back (S-0215
    claim 15) finds no contact, no destination and nothing current or queued. For a harvester
    type that is row 4: target and destination are cleared and *harvest* is queued (for a chrono
    miner the clear installs driving, S-0207 claim 28). Once placed, the free unit is given
    *harvest* with an immediate start, whatever its type, so it starts the harvest cycle at once,
    in *harvest* from its first recorded tick. For a harvester type, that *harvest* replaces the
    one the fall-back queued; both are the same behaviour, so the replacement itself shows
    nothing, and an implementation may equally start the fall-back's own.
    Creating it takes the object creation draw (S-0202, claim 10); the placement and the searches
    take none, except the draws that putting a unit of that type on the map takes in general (none
    for a harvester). The building's other grand-opening effects (its slot animations, S-0108 claim
    11, and any aircraft its type comes with) are other specs'.
    - Backing: `measurable` (scenario H13, parts a to g; for the placement fall-back coming before
      the free unit's *harvest*, S-0207 scenario D34 part f, whose free chrono miner drives from
      its first recorded tick only if that fall-back cleared its destination). For every other
      harvester type the fall-back is `unmeasured`: the free unit has no
      target or destination for it to clear, and the *harvest* it queues is replaced at once, so
      it leaves nothing that a recorder can read; for such a type it changes nothing observable.
      The immediate start of *harvest* is `measurable` (H13; S-0207 D34 part f, *harvest* from the
      tick the free miner appears); its replacement of the fall-back's queued *harvest* is
      `unmeasured`, since it shows nothing, as stated above.
    - Confidence: high for the timing at the grand opening and for the first place and its
      facing, high for no unit after a capture of a building that has had its grand opening (H13
      part e), medium for no unit after an owner change before its grand opening (`measurable`,
      S-0309 scenario A19 part m, run m2), and medium for the two searches, the refund, the start of *harvest*
      and the start-price condition; medium, until S-0207 scenario D34 part f runs, for the
      placement fall-back before *harvest* for a chrono miner, the type that part tests; medium
      for it for other harvester types (`unmeasured`, above); high for suppression of the
      set-up free-unit grant

### Creating and directing a slave group

19. **Existing slaves from creation (Q-0270 part 1).** A newly created object whose type
    names a slave infantry type creates its group as part of its own creation, using its type's
    slave count and regeneration and reload delays. For a positive count *N*, it creates *N*
    infantry of the named type, owned by the miner's house. A count of 0 or less creates none.
    They start ready, inside and off the map, with that miner as master. They initially occupy
    no cells. No payment is deducted for initial or replacement slaves. Creation is not
    postponed until harvesting, deployment or a player's order.

    A produced miner therefore gets its slaves at production start, when its object is created
    (S-0404 claim 11), not on factory release. Starting units, map objects and a refinery's
    free unit get them when those sources create the miner. A newly created deployed building
    with a slave type also gets a hidden group. Conversion preserves the previous working
    group: deploying and packing up already transfer its slaves (S-0415 claims 6 and 10).
    The transferred slaves are not replaced by a second working group. The fate of a group
    created at production start whose item is then cancelled is S-0312 claim 31 (*A cancelled
    production master*).

    Each initial or replacement infantry creation takes the ordinary object-creation draw
    (S-0202 claim 10). Each initial slave also takes one creation number after its own draw.
    The initial slaves' draws and numbers follow the master's own draw and number and those
    of any initial spawn-slot objects made in the same creation, in slave-place order.
    Nested creation sets complete before the next slot or place, as specified by S-0611
    claim 14 and F5. Keeping a slave inside takes no placement search or placement draw.

    **Bad-data type cycles.** S-0611 claim 14 and F5 override the ordinary creation rule
    when a chain of spawned and slave types leads back to a type already in that same
    chain. Our engine creates the cycle-closing object with its own creation draw and
    number, but gives it neither a slave group nor a spawn controller. It has no slave
    places or slots, not empty ones, so it cannot later replace slaves or regenerate
    slot objects. The importer warns. This documented departure applies to that object,
    not to every object of its type; the original has no defined result for such cycles.
    - Backing: `measurable` (scenario H15; S-0611 scenario MU12 for creation-set ordering).
      The cycle exception is `unmeasured`, design: the original has no defined outcome
      to compare; S-0611 scenario MU14 checks our engine's rule
    - Confidence: high for creation, count, ownership, hidden state, no payment and the
      cycle design rule; medium for the creation-source and draw controls

19a. **Pack-up creation draws precede old-group release (Q-0441).** A pack-up
    creates the return vehicle and its own initial hidden slaves under claim 19 before
    attempting vehicle placement, on both success and refusal. Each new slave takes
    its ordinary creation draw. The building's old group still exists at this time.
    These creations therefore precede any destruction or release of the old slaves
    caused by removing the building.

    Successful placement preserves the old working group, but destroys and removes
    the vehicle's newly created hidden slaves before transferring that old group.
    The discarded slaves' creation draws are not undone. Their removal follows
    S-0312 claim 31's hidden-slave destruction notifications, not ordinary death
    effects or attacker credit. This is not a second lasting working group.
    Refused placement skips transfer and retains the new hidden group under
    S-0415 claim 11's default quirk; removing the building ends only its old group.
    With `quirks.failed_packup_keeps_vehicle` off, the already-created vehicle and its new group
    are removed on refusal: their creation draws still occur, rather than being
    suppressed in advance. This alternative is our engine's rule, not another
    outcome observable in the original.
    - Backing: `measurable` (H24 for original creation and removal ordering).
      Temporary-hidden-slave tag notifications are `unmeasured`: no established
      data-only setup attaches a tag to a generated slave. S-0011 claim 19 and
      conditional scenario T25 B cover that separate test if a fixture is found.
      Disappearance and losses alone do not test tag evaluation
    - Confidence: high for creation before placement and old-group removal,
      medium for temporary-slave destruction notifications

20. **Created is not factory-built (Q-0270 part 2).** Initial slaves, their emergence and
    replacement slaves do not raise their type's ever-built count or the end-screen built total.
    They are not factory releases (S-0407 claim 10 and S-0414 claim 20 already distinguish
    creations from releases). Ordinary ownership counts follow S-0407 claim 9 with its type-flag
    exclusions; being inside does not make an existing slave an uncreated object. Later kills
    and losses follow S-0414 claim 21, not a special slave-built statistic.
    - Backing: `measurable` (scenario H15)
    - Confidence: high for no ever-built increment; medium for ownership counts

21. **Group updates and emergence.** A new group is at rest, with last-rest tick 0. Its first
    group update becomes due 10 ticks after creation. Subsequent uninterrupted group updates
    are 10 ticks apart during the master's turns, separate from the slaves' own harvesting
    processings. A failed automatic deployment starts a 30-tick wait before the next group
    update (claim 22). Regeneration and reload timers can expire between updates: their effects
    wait for an update. S-0203 claim 12 places the update relative to the master's
    operation-state effects, animation advancement, paid repair and vehicle movement.
    For a non-negative delay *r* started in tick *a*, completion is observed
    at the first eligible later group update in tick *t* with *t* − *a* ≥ *r*. These are exact
    integer ticks, without rounding. Normal suspension or loss of a master's turns can postpone
    the update further.

    A working group offers each ready hidden slave a place at each group update. A vehicle
    master offers its own cell; a building offers the foundation cell (*W* − 1, ⌊*H*/2⌋)
    counted from its top-left cell, for positive integer width *W* and height *H* in cells.
    It seeks a free infantry standing position there. If none exists, or placement fails, the
    slave stays ready inside until a later update, not on a neighbouring cell. On successful
    emergence its target is the master and it starts the resource-search part of claim 14.
    When only one place accepts an emerging slave, the slave in the latest initial place emerges first. Replacements keep their places. Emergence takes no new object-creation draw.
    The desired emergence point is the offered cell's centre, at its terrain surface height.
    The standing-position choice is S-0208 claim 3's centre-class row, not its
    occupant-ignoring slave-return rule in claim 4. Thus, unless the vehicle-present or
    object-present refusal applies, each ready slave's offer takes one ranged scenario draw
    over 0 to 3, even when all side positions are taken and no position is found. The draw
    selects the corresponding row of S-0208 table O; the first free side position in that
    row wins. The centre and top positions are never selected. When multiple ready slaves
    are offered places at one update, their observable offers and associated draws occur in
    reverse slave-place order. Every successful emergence starts with facing 0 (map north),
    independent of the master's facing; subsequent walking can turn the slave. A vehicle master that still occupies the offered cell
    can therefore block emergence. There is no draw when the initial occupancy refusal
    applies, and no second draw merely because the subsequent placement fails.
    A replacement created, or a hidden slave made ready by reload completion, can emerge
    in that very update if the master's state permits emergence and a position accepts it.
    A slave taken inside in that update cannot finish reload until a later update, even
    with reload delay 0. Slave-place order fixes delivery and replacement ordering (claim 26).
    - Backing: `measurable` (scenarios H15, H16 and H21)
    - Confidence: high for initial rest tick, periods, emergence cell, facing and slave-place
      order; medium for placement refusal and timer boundaries

22. **Harvest directs the miner (Q-0270 parts 4 and 5).** A miner with both resource-gatherer
    and resource-destination fields and a slave group processes *harvest* through its group rule,
    not the ordinary refinery-dependent cycle of claims 3 to 10. Its order still supplies a
    destination and queues *harvest*, recalling the slaves to guard it as already specified by
    S-0213 claim 5 and table T3 part (h). The miner does not itself load a bale there.

    At each processing of *harvest*, a vehicle with a destination searches for a deployment
    site near that destination, moves to a found site with *move* queued, and puts the group
    into travelling. If the site search fails, it queues *guard*, puts the group at rest and
    records the current tick as last-rest. A vehicle without a destination instead queues
    *guard* and records rest in the same way, without a field or site search. In every case
    the next processing delay is the standard delay of *harvest* (S-0215 claim 6), including
    its one ranged draw, not 1 tick. The queued behaviour follows the usual start rules.
    A new harvest order that becomes current therefore searches using its new destination,
    even during an older journey. Ordinary travelling group updates do not search again
    near that destination. If *harvest* processes again before the queued *move* starts,
    however, it searches again: this is not a once-per-order restriction.
    A travelling vehicle with no destination attempts deployment at its next group update,
    under S-0415 claims 2 to 6. Success puts the group into working. Failure enters a retry state and
    starts the 30-tick wait of claim 21. The next update retries deployment; another failure
    puts the group into field seeking. A travelling master that has become a building instead
    puts the group at rest.

    Waking under claim 14 starts field seeking, not immediate emergence of all hidden slaves.
    A seeking vehicle already with a destination enters travelling without replacing it.
    Otherwise it seeks a field with the miner's long scan range and searches for a site near
    the result. No field or no site puts it at rest with the current tick as last-rest.
    With a site, it moves there and enters travelling. A working vehicle that has stopped
    deploying and is in *guard* re-enters travelling, permitting another deployment attempt.

    An at-rest deployed group enters checking the field at an update that sees neither
    *construction* nor *selling* current; its first field check is at a later update.
    A working group whose master has become a building enters checking the field once the
    building is outside its construction state, without checking or emergence in that
    transition update. Claim 27 gives the busy-state boundaries. A short-scan field keeps it where it is and permits
    release of ready slaves. With no short-scan field, the long scan seeks another field.
    The scan correction is an improvement margin in whole cells: a prospective site must
    improve the truncated cell-distance to the selected field by more than that margin before
    this rule requests automatic packing-up. These checks apply to human-controlled owners
    too; AI houses have the same rules. The original's computer-only waking shortcut remains
    claim 14's. Actual conversion and slave transfer are S-0415 claims 6 and 10.

    Site selection is claim 24. When both the long scan and site search succeed, the relocation
    comparison is F7: equality does not request packing-up. The remembered cell for this
    automatic pack-up is the building's own top-left cell, not the selected field or site.
    A packing group recalls its slaves and enters field seeking at the first group update
    that sees the vehicle. That update does not also seek a field: the next update does.
    Claim 25 gives the vehicle's earlier destination. Manual packing is not an unconditional
    entry to this packing group state (claim 28).
    - Backing: `measurable` (scenarios H16 and H23; S-0213 claim 5, scenario O4 for recalling the slaves)
    - Confidence: high for the harvest override and deployment retry,
      medium for field-seeking and relocation gates

23. **Ownership when the master changes house (Q-0270 part 3).** Initial and replacement
    slaves take the master's owner at their own creation (claim 19). Changing a surviving
    master's house, by engineer capture of the building or mind control of the vehicle,
    does not itself change the owners of its existing slaves, hidden or on the map.
    Their master remains that object. Emergence and reload do not replace the slave's
    owner either. A replacement created afterwards belongs to the master's new house,
    so one group can contain slaves of different houses. Delivery pays the master's
    current house under claim 14, not the delivering slave's house.

    Deploying and packing up transfer the master relationship without recreating these
    slaves (S-0415 claims 6 and 10). Ordinary killing-hit outcomes are S-0312 claim 30.
    true-sale slave fate is S-0408 claim 10a. Neither is capture of a surviving master.
    Erasure without a killing hit, environmental sinking and other destruction paths
    remain unresolved here, as does the exact sale-cleanup tick. Capture and
    mind-control-return variants retain H17's ownership measurement.
    - Backing: `measurable` (scenario H17)
    - Confidence: medium; capture does not imply propagation, but hidden-slave and
      mind-control-return variants need the same ownership test

24. **Choosing a slave-miner deployment site.** The centre and reference cell are the
    desired destination cell for a directed miner, or the long-scan resource cell for
    field seeking and relocation. The search uses the ring order, visibility preference,
    collection limit and minimum-distance tie rule of S-0212 claim 25, with no random draw.
    Unlike its single-cell uses, a candidate is the top-left cell of a rectangular footprint.
    Width and height are the master's building dimensions when deployed, or those of its
    deploys-into building type when mobile; a vehicle without a conversion type uses 1 × 1.

    The locomotion class is *foot*, movement zone *normal*, and reach region is that of
    the master's own ground cell for *normal*. There is no deck mode, height condition
    or burrowing condition. The top-left candidate must be in the height-aware usable area
    and must not be a bridge cell. Every cell of the width-by-height rectangle must pass
    the class, reach-region and empty-occupancy tests of S-0212 claim 25 and contain no
    overlay. Every rectangle cell must additionally have passability category *open*
    (S-0211 claim 11), no terrain object or building, and no slope. All four rectangle
    corners must pass the height-aware usable-area test. This is a site search, not the final
    deploy legality test: S-0415 claims 2 to 6 still decide whether deployment succeeds.

    Let the chosen top-left cell be (*x*, *y*), width *W* and height *H*, integer cells.
    The vehicle destination returned by the search is
    (*x* + 1 when *W* > 2, otherwise *x*,
    *y* + 1 when *H* > 2, otherwise *y*).
    These are exact integer additions, independently on the two axes, not a geometric
    centre calculation. Distance ties are decided on the unshifted top-left candidates.
    A failed search gives no site, with the directed and seeking outcomes of claim 22.
    - Backing: `measurable` (scenario H18)
    - Confidence: high for dimensions, axis shifts and absence of draws; medium for
      footprint qualification and the shared search tie rule

25. **Relocation failures still compare distances (Q-0411).** A deployed group checking
    the field uses F7 even when the long scan finds no field or the site search finds no
    site. A missing field supplies cell (0, 0) as *F*, and the site search of claim 24
    is still made around it. A missing site supplies cell (0, 0) as *S*, without the
    foundation-size shifts. These are comparison points, not usable destinations.
    With neither result, packing is requested when *c* < *d*(*A*, (0, 0)); with a field
    but no site it is requested when *c* + *d*((0, 0), *F*) < *d*(*A*, *F*).
    Equality does not request packing. There is no unconditional stay or unconditional
    packing on failure. `quirks.slave_relocation_failed_search_origin` defaults to
    this original rule; off, either failure leaves the miner deployed without requesting
    packing. No importer warning: a field can run out or a valid site become occupied.

    Without a packing request, ready slaves are still offered emergence in that update.
    The field check repeats at the next ordinary group update, normally 10 ticks later.
    Slaves' independent field searches can send them back empty. With a packing request,
    no emergence is offered in that update.

    Automatic packing remembers *A*, the old top-left cell. Successful conversion gives
    the vehicle that cell as its immediate destination and queues *move* under S-0415
    claim 10, even before the next group update. It is not sent to the distant field or
    tested site. A large foundation's return cell can differ from *A*, allowing movement
    towards *A* before field seeking. The first vehicle group update only recalls slaves
    and enters field seeking (claim 22). At the following update, an existing destination
    is kept and makes the group travelling; otherwise the long scan and site search apply.
    Later arrival and deployment use claim 22.
    - Backing: `measurable` (H19)
    - Confidence: high for zero-cell comparison inputs and strict comparison; medium for
      conversion-to-movement timing and later group-update interaction

26. **Slave work is paced by two clocks (Q-0412).** At a group update, each slave place whose loss timer has expired receives a replacement attempt; the attempt is not limited to one place per update. When effects for several places are due in the same update, the observable effects occur in increasing slave-place order: reload completions, returning deliveries, and outside slaves' searches and transitions all follow that order rather than being grouped into separate passes. Thus a delivery from an earlier place is observed before replacement creation for a later place. These effects are observed before the master's field, travel, deployment or relocation decision. Replacement-creation draws are observed in increasing place order; emergence offers and their draws are observed in reverse place order (claim 21).

    Seeking, checking resource-cell arrival, detecting a full or exhausted load, and
    accepting delivery are group-update effects. A slave actually loading in *harvest*
    takes load steps in its own turn, at its type's interval *h*. A processing on a
    non-resource cell or while full queues *guard* and has a 1-tick next-processing delay.
    Seeking and walking back do not run a separate *harvest* processing every *h* ticks.
    A newly created slave with capacity 0 or less loads nothing, but is not full and
    does not immediately return merely because of that capacity. Unlike a vehicle
    harvester, a zero-capacity slave has fill ratio 0, not a non-number. Its *harvest*
    processing queues *guard* with a 1-tick delay. At a group update checking a slave
    still on resource land, out of *harvest*, the slave re-enters searching.
    Arrival at a selected resource cell queues *harvest* again. It can therefore
    alternate searching and attempted harvesting without taking resources.

    A negative-capacity slave starting empty has fill ratio 0 as well. On resource
    land its *harvest* processing leaves the resource cell and load unchanged,
    remains in *harvest* and schedules the next processing after *h* ticks. Its
    requested bale amount is trunc(min(1, single(*C*) − 0)), using floating-point
    arithmetic and round-toward-zero conversion to integer bales (S-0205). This
    amount is negative; a non-positive take delivers nothing and changes no resource
    stage (S-0401 claim 3). The group does not send it back for fullness: it can
    remain harvesting an unchanged cell indefinitely. Exhaustion by another unit
    still permits searching and an eventual empty return when no field is found.
    `quirks.slave_negative_capacity_stalls` defaults to this original rule; off,
    negative slave capacity uses the zero-capacity treatment above.

    A slave's field search uses claim 4's rings, worth and ties, but requires its own
    infantry entry outcome (S-0209 claim 4) to be *clear* and its own movement-zone
    connection from its heading-point cell. A partly occupied infantry cell qualifies
    when that outcome is clear; another slave or unit is not a blanket refusal. As in
    claim 4, tiberium land at its own cell selects that cell without the candidate tests.
    The search takes no draws.

    A deployed master's short and long scans centre on the cell of its reference point,
    not its top-left or unloading cell. They use the same rings, worth and ties, and
    immediately accept tiberium land at the centre. Other candidates need only tiberium
    land: no infantry-entry, occupancy, reach-region, shroud or usable-area test applies.
    Resources in legal cells outside the usable area can therefore be selected. This
    promises neither a deployable site nor a reachable slave destination. Site tests
    remain claim 24's separate rule.
    - Backing: `measurable` (H21). The heading-point-specific slave connectivity
      test is `unmeasured`: H21's disconnected-field variant does not distinguish
      the heading-point cell from the slave's current cell, and no separate reachable
      boundary setup is established here.
    - Confidence: high for place ordering, clocks, non-positive capacity and deployed-scan
      qualification; medium for slave connectivity and scan-centre boundaries

27. **Busy masters do not suspend all slave work (Q-0413).** Loss replacement, reload,
    outside-slave searches and delivery continue at due group updates during a building's
    construction, selling or packing. Returning slaves in the unloading cell without a
    destination still pay and go inside. The table fixes master decisions and emergence.
    There is no separate all-group busy-state pause.

    | Group state and master | Due update result |
    |---|---|
    | At rest, building currently in *construction* or *selling* | Remains at rest; no field check or emergence. |
    | At rest, building in neither behaviour | Enters checking the field; no check or emergence in this update. |
    | Field seeking, building | Uses the long scan and site search of claim 22. Failure puts the group at rest and records this tick as last-rest. Success offers the site under claim 27a, requests *move* without an immediate start, and enters travelling. No emergence or field-check transition occurs in this update. |
    | Travelling or retrying deployment, building | Enters at rest and records this tick as last-rest, without field search, deployment attempt or emergence. |
    | Working, building still in its construction state | Remains working; no check or emergence. |
    | Working, building outside its construction state | Enters checking the field; no check or emergence in this update. |
    | Checking the field, building | Field and relocation rule; emergence unless packing is requested. This state has no further construction or selling exclusion. |
    | Packing, still a building | Waits for conversion; no field check or emergence. |
    | Packing, now a vehicle | Recalls slaves and enters field seeking; no check or emergence in this update. |

    Removal without a killing hit ends a remaining, untransferred group under S-0312
    claim 31. That rule fixes the group's fate at removal, not when a sale, erasure or
    failed conversion reaches removal. S-0415 claims 6 and 10 own successful group
    transfer and the failed-conversion boundary; this claim does not apply the true-sale
    result to every failed pack-up.
    Field seeking is therefore not equivalent to travelling after manual deployment.
    When its site search succeeds, the next update normally sees travelling and puts
    the building's group at rest; a subsequent eligible update enters checking the
    field, and only a later update checks it. Construction or selling does not
    suppress the field-seeking or retrying rows themselves. A selling building can
    refuse the destination and queued behaviour under claim 27a without
    preventing the group's change to travelling.
    - Backing: `measurable` (H22, H25). Checking-the-field during current
      *selling* is `unmeasured`: H22 has no established fixture that preserves
      that overlap rather than entering packing or removing the master.
    - Confidence: high for absence of a group-wide pause and the state gates; medium for
      busy-state delivery and reachable selling-state variants

27a. **The field-seeking building's site offer (Q-0500).** Claim 27's successful
    search offers the shifted site cell of claim 24, not the unshifted footprint corner.
    Its destination result is the first matching row:

    | Building at the offer | Destination result |
    |---|---|
    | Currently in *selling* | Keeps its earlier guard spot, including an absent one. |
    | Not in *selling*, and its type is a factory of infantry or vehicles, a cloning building, a repair depot or a construction yard | Replaces its guard spot with the site cell. |
    | Any other building | Keeps its earlier guard spot, including an absent one. |

    There is no separate movement destination on the building. Merely having an
    undeploys-into type or a slave group does not qualify it for the replacement.
    Construction state, power, owner and the construction-yard redeploy setting
    add no exclusion to this offer. The retained or replaced guard spot is the
    same remembered destination of claim 28, read by S-0415 claims 9 and 10 and
    S-0408 claim 17. Consequently a later successful pack-up uses that spot,
    not a hidden second copy of the offered site. This is not the guard-spot
    order of S-0213 claim 20: that order can directly change the guard spot and
    is not subject to this site's offer conditions.

    Independently of destination acceptance, the group requests *move* without
    an immediate start. Current *selling* refuses that request and preserves
    the earlier queued behaviour (S-0215 claim 7). Otherwise ordinary queueing
    applies even when the destination offer changed nothing. The group enters
    travelling in either case. At its next update while still a building,
    claim 27 puts it at rest; that transition does not clear the guard spot or
    cancel the queued or current *move*.

    An accepted *move* can become current through S-0215 claim 11's building
    starts before that next group update, when readiness permits. Its building
    processing does nothing, schedules the next processing 450 ticks later
    and takes no draw (S-0215 claim 4 and table T1). It does not move the
    building, request packing or clear its guard spot. Becoming current does
    replace the previous behaviour, so its previous processing, such as
    *guard*, does not run while *move* remains current. No automatic return
    to *guard* accompanies the group's return to rest.
    - Backing: `measurable` (H25, site-offer variants). Selling-state variants remain
      `unmeasured` until a run reaches field seeking during current *selling* before
      conversion; the proposed timing test alone does not establish that overlap.
    - Confidence: high for the destination gates, retained state and no-op
      processing; medium for reachable selling-state timing

28. **Manual packing and order boundaries (Q-0414).** Stop and ordinary behaviour-order
    recall belong to S-0213 claim 8 and claim 5. They do not supply a building with the
    vehicle-only recall effect. *Attack* is already excluded from behaviour-order recall.
    Do not extend recall to all move and attack orders alike.

    At the first processing of selling or packing, a slave-master building invokes the
    directed site rule only when its remembered destination is a resource-land cell other
    than its own top-left cell. A found site replaces that remembered destination, recalls
    its slaves and puts the group into packing. A failed site search leaves the previous
    remembered destination and group state unchanged. Without that qualifying destination,
    this rule does nothing to the group. Automatic relocation remembers the building's
    own cell and therefore skips this additional site search.

    **One remembered destination (Q-0442).** The automatic old-cell memory of
    claim 25 and the building's guard spot read by S-0415 claim 9 and S-0408
    claim 17 are the same destination, not independent memories. Automatic
    relocation replaces any earlier guard spot with the old top-left cell.
    A later move or harvest order that changes that guard spot replaces the
    automatic cell in turn. Claim 28's site rule reads whichever destination
    is then present, with no automatic-over-manual priority. A later sell order
    does not itself clear this destination: subject to the other pack-up
    conditions of S-0415 claim 9, it packs towards the retained old cell rather
    than becoming a true sale merely because that cell was remembered automatically.

    Manual packing does not always recall and enter field seeking: it does so when the
    group is packing. A group still checking the field instead becomes at rest when its
    update sees a vehicle, records that update as last-rest and does not recall by that
    transition. An at-rest group stays at rest. Subsequent wake-up is claim 14's; inherited
    destination and *move* are S-0415 claim 10's. Travelling arrival does not deploy at
    once: the attempt is at a due group update seeing no destination (claim 22).
    - Backing: `measurable` (H23, H25)
    - Confidence: high for shared destination, site-rule gates, group transitions and group-update deployment,
      medium for manual packing scenarios and remembered-destination handover

## Formulas

**F1. Load timing** (claim 5). With loading started at tick *t*₀, *L* the load step period, and
nothing interrupting, bale *j* (*j* = 1, 2, …) is loaded at tick

> *t*₀ + 19 + (*j* − 1)(9*L* + 1)

integers, ticks, for effective capacity *C* ≥ 1 and *L* > 0. A harvester of capacity *C*
that starts empty on a field that never runs out has
its last bale at tick *t*₀ + 19 + (*C* − 1)(9*L* + 1). It is full from then on, but the phase ends
only at the next load step, in tick *t*₀ + 19 + *C*(9*L* + 1), when that step fails because the
harvester is full. That processing changes it to *returning*; refinery selection occurs no
earlier than its following processing. For *L* ≤ 0 see Edge cases.

**F1a. Arrival-to-load timing** (claim 3b). Use claim 3b's uninterrupted
arrival conditions, effective capacity at least 1 bale, a cell that can deliver
its first bale, and a positive harvest rate delay *r* in integer ticks
(S-0215 claim 6). Let *P* be the first due harvest processing tick strictly
later than track completion tick *T*, and *S* the loading-start tick.

| Condition | *S* | First loading-phase processing | First load-step-bearing processing and first bale |
|---|---|---|---|
| *P* = *T* + 1 | *T* + 1 + *r* + *u*, where *u* is that processing's delay draw from 0 through 2 | *S* + 1 | *T* + 20 + *r* + *u* |
| *P* ≥ *T* + 2 | *P* | *P* + 1 | *P* + 19 |

All arithmetic is integer addition, without rounding; units are ticks. Draw
selection is S-0202 claim 4's, not a new arrival draw. A processing in *T*
still uses its ordinary standard delay, which can set *P*. Given continuous
eligible processing and these conditions, the first bale is no earlier than
*T* + 21 and no later than *T* + *r* + 22: the longest residual delay is
*r* + 2, or a processing in *T* + 1 can choose a further *r* + 2 ticks.
These are bounds, not a claim that every interval occurs equally often or
that every distance admits every interval. The measured distribution is the
histogram of *S* − *T* + 19 over the chosen arrivals. No sample-independent
frequency follows from the load timer alone. Nonpositive rate delays and
turns excluded from processing instead follow S-0215 claims 2 and 6; the
positive-rate bounds above do not apply to them.

**F2. Unload timing** (claim 11). Let unloading start at the processing in tick *u*₀ (facing east,
in contact). The counter seen by a processing in tick *t* is *c* = *t* − *u*₀ − 1 until the first
payment, and *c* = *t* − *u* after a payment in tick *u*. The processing after the start row comes
in tick *u*₀ + *d*, *d* being the standard delay drawn at the start row, and every tick after it
has a processing, so the first payment comes in the first tick *t* ≥ *u*₀ + *d* with

> *t* − *u*₀ − 1 ≥ 900 × *D*, that is *t* = max(*u*₀ + *d*, *u*₀ + 1 + ⌈900 × *D*⌉),

and each later payment in the first tick *t* with *t* − *u* ≥ 900 × *D*, that is *t* = *u* +
max(1, ⌈900 × *D*⌉), where *u* is the tick of the previous payment. The minimum of 1 tick
reflects the next processing: no second payment occurs in the same processing, including
when the dump rate is zero or negative. When the load is empty at such a
processing, the harvester becomes *finishing* instead of being paid. 900 × *D* is a double product
of the double nearest 900 and *D*, exact for single *D* (S-0205, claim 9); the left side is an
integer, and ⌈ ⌉ rounds the double up to an integer. For the absent-key default, a double, the
product is not exact, but it lies near 14.4, far from an integer, so the ceiling is 15 under
either rounding mode. Units: ticks.

**F3. Payment** (claim 12). For an amount *a* of tiberium *T*:

> *B*′ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(*V* × *M*) × *a*) + *B*))

where *V* is an integer, *M* a single, *a* a whole number held as a single, and *B* the balance (an
integer). The products and the sum are double operations under the round-toward-zero mode, and
trunc is toward zero (S-0205, claim 5; `quirks.chop_rounding`). The bonus amount is

> *x* = RZ₂₄(RZ₅₃(*n*<sub>p</sub> × *b*) × *a*)

(a single result), and when *x* > 0 a second payment *B*″ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(*V* × *M*) × *x*) + *B*′))
follows. Units: credits. Example with invented values: *V* = 30, *M* = 1, *a* = 10, *n*<sub>p</sub>
= 1, *b* = 0.25 (exact in single): 300 credits, then *x* = 2.5 and a bonus of trunc(75) = 75.

**F4. Fill ratio** (claim 16). total = the sum of the held amounts (at most four tiberiums can be
held in the original, every tiberium in our engine, claim 15): whole numbers, so the sum is exact.
ratio = total / *C*, a double quotient. The harvester is full when ratio = 1.0, which for integers
*C* ≥ 1 below 2³¹ holds exactly when total = *C*, under either rounding mode. For *C* = 0 the
quotient is 0 / 0, not a number: see Edge cases. Units: bales; the ratio has none.

**F5. Wrapping sums and squares** (claims 8 and 14). The following are computed as signed 32-bit
integers, with the true result *r* replaced by its residue modulo 2³² in the range −2³¹ to 2³¹ − 1:
*r* − 2³² × ⌊(*r* + 2³¹) / 2³²⌋. This is exact integer arithmetic; ⌊ ⌋ is floor division.
The affected expressions are:
- the kick tests *k* + *s* < *t* (claim 14), integers, ticks, and
- the refinery distance *dx*² + *dy*² (claim 8), integers, leptons squared; it wraps when the true
  value is at least 2³¹, which needs a distance of at least about 46341 leptons (about 181 cells).
  Two such distances are compared as signed numbers.
Each wrap has its own quirk switch, in claims 8 and 14. With
`quirks.refinery_distance_wraps` off, compare exact non-negative integer squared distances
instead of their signed residues; there is no rounding. This does not remove claim 8's
primary-building overrides or change either tie rule.

**F6. Step counter** (claims 5 and 11). A pacing period *p* started in tick *a* holds until the
next period is started, in tick *a*′ (or for good). It makes the counter rise in the ticks *τ*
with *a* ≤ *τ* < *a*′ such that

> *τ* = *a* + *kp* for an integer *k* ≥ 1, when *p* > 0; every such *τ*, when *p* < 0; none, when
> *p* = 0.

A processing in tick *t* sees the counter

> *c*(*t*) = the number of rise ticks *τ* with *r* ≤ *τ* ≤ *t* − 1,

where *r* is the tick of the latest processing before *t* that set the counter to 0. Integers,
exact; unit: counter steps. With loading started in tick *t*₀ (*p* = 2, *r* = *a* = *t*₀), the
counter seen is ⌊(*t* − 1 − *t*₀) / 2⌋, which first reaches 9 in tick *t*₀ + 19. After a successful
load step in tick *s* (*p* = *L* > 0, *r* = *a* = *s*), it is ⌊(*t* − 1 − *s*) / *L*⌋, which first
reaches 9 in tick *s* + 9*L* + 1 (F1). For *L* < 0 it is *t* − *s*, which reaches 9 in tick *s* + 9.
For unloading, see F2.

**F7. Slave-miner relocation comparison** (claims 22, 24 and 25). Let *F* be the selected
resource cell, *S* the shifted vehicle destination from claim 24, *A* the deployed
master's top-left cell, and *c* = trunc(scan correction in leptons / 256), integer cells.
For cell points *P*, *Q*, define

> *d*(*P*, *Q*) = signed₁₆(trunc(*r*(Δx² + Δy²)))

where each coordinate difference is reduced modulo 65536 to the signed range
−32768 to 32767 before squaring. Products and sum are double floating-point operations
under S-0205's round-toward-zero mode. The table root *r* is S-0204's
`quirks.table_root_distance`; its result is truncated toward zero and reduced modulo
65536 to the same signed range. Units: cells. Packing-up is requested exactly when

> *c* + *d*(*S*, *F*) < *d*(*A*, *F*).

The final addition is signed 32-bit wrapping arithmetic as in F5, with no further
rounding. On normal-sized maps no narrowing or addition wrap occurs. This formula
uses claim 25's cell (0, 0) substitution for missing results under the default quirk.

**F8. Harvesting sprite** (claim 5a). Let *q* be the body's 8-step facing, an integer
from 0 to 7 by S-0204 claim 10; *t* the current tick number; and *ν* the vehicle's
walk-animation counter of S-0105 claim 16. The image frame is

> 15(7 − *q*) + ((*t* + *ν*) mod 15).

This is integer arithmetic with a non-negative remainder, in image frames. Each facing
has 15 image frames. The loading step counter and the load period do not enter this formula.
For the anchor, let (*X*, *Y*, *Z*) be the vehicle's world position in integer leptons,
and let *f̂* be 8192*q* reduced to the signed facing range −32768 to 32767. Then

> θ = RZ₅₃((*f̂* − 16383) × *c*), where *c* is the binary64 value nearest −2π / 65534,
> *X*ₐ = trunc(RZ₅₃(*X* + RZ₅₃(30 × *C*ₜ(θ)))),
> *Y*ₐ = trunc(RZ₅₃(*Y* − RZ₅₃(30 × *S*ₜ(θ)))), and *Z*ₐ = *Z*.

The angle is in radians, the 30 in leptons, and both coordinate conversions truncate
toward zero. Let *S*ₜ(θ) and *C*ₜ(θ) denote the table sine and cosine required by S-0205
claim 6, including the table's negative-angle quirk; they are not exact mathematical
sines and cosines or library evaluations. Their results are single-precision values
widened exactly for the double-precision products and sums, which round toward zero.
*C*ₜ(θ) contributes to *X* and *S*ₜ(θ) to *Y*, not the reverse. Project
(*X*ₐ, *Y*ₐ, *Z*ₐ) by S-0101 claim 1 and centre the harvesting image there.
S-0204 claim 8's `quirks.table_direction` covers this reverse short-angle scale too:
with it off, replace 16383 by 16384 and *c* by the binary64 value nearest −2π / 65536.
The image-frame formula and the visibility conditions are unchanged.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Harvesting sprite facing groups | 8 | facing groups | claim 5a, F8 |
| Harvesting sprite frames per facing | 15 | image frames | claim 5a, F8 |
| Harvesting sprite anchor displacement | 30 | leptons | claim 5a, F8 |
| Step counter threshold for a load step | 9 | counter steps | claim 5 |
| Pacing period started when loading starts | 2 | ticks | claim 5 |
| Pacing period while unloading | 1 | ticks | claim 11 |
| Bales per load step | 1 | bales | claims 5, 16 |
| Ticks per game minute (dump rate) | 900 | ticks per game minute | claim 11, F2 |
| Idle interval after no field is found | 105 | ticks | claims 3, 13 |
| Processing interval while turning to dock | 5 | ticks | claim 11 |
| Processing interval while unloading, except at the start and when the building is missing | 1 | ticks | claim 11 |
| Facing counted as map east | 16256 to 16511 | facing units | claim 11 |
| Docked refinery position | step direction 6 (map west) of the docking cell | step direction | Terms, claim 11 |
| Docking cell, counted from the refinery's top-left cell | (3, 1) | cells | Terms, claim 10; independent of the foundation size and the art |
| Facing aimed at on the docking cell | 16384 (exactly) | facing units | claim 10 |
| Processing interval of *enter* | the standard delay of *enter* | ticks | claim 10; one draw per processing |
| Processing interval of *unload* without contact | 1 | ticks | claim 11 |
| Free unit's first place | the cell in step direction 4 (map south) of the cell of the building's reference point | step direction | claim 18 |
| Free unit's facing at the first place, and at a searched place | 49152 (map west), 40960 (map south-west) | facing units | claim 18 |
| Queueing-cell search: locomotion class and movement zone | *wheel*, *normal* | — | claim 9 |
| Free-unit search: locomotion class | *wheel* | — | claim 18 |
| Wait distance near a refinery | 768 | leptons | claim 9 |
| Leptons per cell for the too-far fields and scan ranges | 256 | leptons | claims 4, 9 |
| Held tiberiums in a load | 4 | tiberiums | claim 15; the original's limit. Our engine holds every tiberium (claim 15, departure) |
| Wrap modulus of the kick sums and the refinery distance | 2³² | integer values | claims 8, 14, F5 |
| Minimum dock slots | 1 | harvesters | Terms |

## State and ordering

**Harvest phases** (claim 3). "Next" means the harvester's next processing.
Destination means the order-level destination (Terms). Claim 3b owns the
post-arrival window: a due seeking processing in the tick after final track
completion still sees that destination; its removal later that tick does not
cause a second processing.

| Phase | Condition at a processing | Result | Next processing |
|---|---|---|---|
| any | owner owns no building of a dock type | queues *guard* | 1 tick |
| seeking | full | → returning | 1 tick |
| seeking | not full, has a remembered cell | drives there and forgets it; no search in this processing (a harvester whose teleporting locomotion is still active at the drop's test: the seeking drop of claim 17 removes the adopted cell, and the processing then takes the ring-search rows below; a chrono miner in cases (a) to (d) of claim 17 (*Who reaches the drop*) is not such a harvester, since adopting the remembered cell installs driving before the drop's test, S-0207 claim 28; the drop holds for a chrono miner with warp-pending immobilization, whose open case is only the ring search's destination, and for one unloading at a sale, case (f), whose adoption keeps teleporting while the sale's contact holds, claim 17) | standard delay; for such a harvester, the next processing is that of the row the ring search then reaches |
| seeking | not full, no remembered cell, has a destination | stays seeking; no search (a harvester whose teleporting locomotion is still active, as in the row above: the seeking drop removes the destination, and the processing then takes the ring-search rows below; for a chrono miner whose temporary driving S-0607 claim 4 ended, the drop's empty assignment installs driving first, claim 17) | standard delay; for such a harvester, the next processing is that of the row the ring search then reaches |
| seeking | not full, no remembered cell, no destination, ring search (long scan) gives its own cell | step counter 0, period 2 started; → loading | 1 tick |
| seeking | not full, no remembered cell, no destination, ring search gives another cell | drives there; stays seeking | standard delay |
| seeking | not full, no remembered cell, no destination, nothing found | → idle | 105 ticks |
| loading | counter < 9 | waits | 1 tick |
| loading | counter ≥ 9, has a destination | nothing | 1 tick |
| loading | counter ≥ 9, load step succeeds | counter 0, period *L* started | 1 tick |
| loading | step fails, full | counter 0, period 0; remembers a short-scan cell; → returning | 1 tick |
| loading | step fails, capacity 0 | counter and period unchanged; counted as full: remembers a short-scan cell; → returning | 1 tick |
| loading | step fails, not full, short-scan cell found | drives there or stays; stays loading | 1 tick |
| loading | step fails, not full, nothing found, no destination | forgets; → returning | 1 tick |
| returning | has a destination | keeps driving (chrono miner: claim 17) | standard delay |
| returning | normal choice within too-far distance, accepted | contact; → requesting | standard delay |
| returning | otherwise | waiting choice: stay, or drive to the queueing cell (claim 9) | standard delay |
| requesting | — | queues *enter* | 1 tick |
| idle | — | attempts to move off a refinery-flag or weed-flag building's cell; destination (0, 0) if that search fails; queues *guard* | standard delay of *harvest* |

**Docking** (claim 10, which has the full table): at each processing of *enter*, one standard
delay apart: refused and not tethered → break off contact, rest; not on the docking cell →
destination set to it; on it, facing not exactly 16384 → tethered, turn on the spot; on it, facing
16384, not moving → tethered, *unload* queued (it starts by S-0215's start rules).

**Unloading** (claim 11, which has the full table): not in contact → rest (nothing queued while
*unload* is current), stop, start a queued behaviour if any, every tick; turning → every 5 ticks,
facing east → counter and period start, docking animation, then a standard delay; afterwards a
processing in every tick: pay one tiberium (payment animation) when the counter allows; empty and
the counter allows → *finishing* (unloading animation); *finishing* and the building's unloading
animation ended → queue *harvest*, break off contact, start it, then a standard delay. An order
breaks off contact at once (S-0213); the next processing takes the first row and starts the order.

**Ordering.** Harvester load steps and payments happen in the harvester's turn; slave load
steps happen in the slave's turn, but slave deliveries happen at group updates in the master's
turn (claim 26), in the turn pass (S-0203 claim 1, row 5); a payment in tick *n* is seen by production in
tick *n* (S-0203, claim 8). The idle step-off and the second-route arrival report take no draws of their own. Their
standard-delay processings, when due, still draw normally. Scatters use the vehicle rules of
S-0207 claim 16, never the infantry direction draw of S-0208. A bale taken from a cell changes the cell at once (S-0401).
When two harvesters load from the same cell during one tick, the later turn observes the cell
after the earlier turn's take, as required by S-0203's update order. This spec does not otherwise
select which harvester takes the earlier turn.

## Edge cases and quirks

- **Failed idle step-off aims off-map** (claim 13).
  `quirks.harvester_failed_step_off_origin` (default: original) replaces the destination with
  off-map cell (0, 0) when no step-off cell qualifies. It does not preserve a previous
  destination or mean no destination. With the switch off, that failure clears the destination
  instead. Both settings queue *guard*, use the same standard delay and take the same draws.
  Neither performs another search. No importer warning: occupancy, not bad data, causes this.
- **Capacity 0** (claim 1, bad data, importer warning). The fill ratio is 0 / 0, which is not a
  number. Such a harvester counts as not full when it seeks and when it makes a load step, and as
  full in the check after a failed load step. So the harvester seeks, goes to
  a field cell, starts loading, and 19 ticks later its first load step takes no bale (the room
  left is 0) and fails without touching the counter or the period (claim 5's table); the check
  after it counts the harvester as full, it runs claim 6's short-scan search and goes *returning*
  with an empty load, is docked, unloads nothing (becoming *finishing* once the counter allows)
  and returns to harvest; it never carries a bale, is never paid, and repeats the cycle.
  `quirks.zero_capacity_harvester_cycles` (default: original) retains this empty cycle.
  With the switch off, imported vehicle capacity 0 uses effective capacity 1 bale:
  fill ratio, load limit, full checks and load steps all use that positive capacity,
  so it loads, returns and earns payment normally. This changes neither slave capacity
  nor any other imported capacity. The importer warns about vehicle capacity 0 in
  both settings. Confidence is low for the original outcome, measurable by H1 variant d.
  The off setting is a design rule, tested only on our engine.
- **Non-positive slave capacity** (claim 26). Neither zero nor negative capacity
  makes an empty slave full. Zero capacity alternates guard and group-driven searches
  without loading; negative capacity can stay harvesting unchanged resource land
  indefinitely. `quirks.slave_negative_capacity_stalls` (default: original) retains
  the latter outcome. Off, negative slave capacity uses the zero-capacity guard and
  rescan treatment. This is not the vehicle's capacity-0 cycle above.
- **More than four tiberiums** (claim 15): departure, importer warning.
- **The refinery distance wraps** (claim 8, F5) for a distance of about 46341 leptons (181 cells)
  or more between the reference points, which a large map's diagonal can reach.
  `quirks.refinery_distance_wraps` (default: the original): the squared distance is a signed 32-bit
  integer that wraps, so a refinery that far away can look nearer than one close by. With the
  switch off, *dx*² + *dy*² is an exact non-negative integer, without wrapping or rounding,
  at both levels of claim 8's choice. Eligibility, holding order, dock-list order,
  primary-building overrides and ties are unchanged; a primary building can therefore still
  defeat a nearer one. H4 part e includes an engine-only off-setting check.
  No importer warning: the data is valid.
- **The last bale** of a cell yields nothing (S-0401, `quirks.last_bale_lost`), so a harvester
  that empties cells one after another loses one bale per cell.
- **A load period of 0** (claim 5). With *L* = 0 the pacing period after a load step is 0, and a
  period of 0 keeps the counter where it is, so the harvester loads its first bale (the period
  before it is 2) and then no more: it stays in *loading* for good, never full and never returning.
  `quirks.zero_load_period_stalls` (default: the original). With the switch off, a period of 0 is
  treated like a negative one: the counter goes up in every tick and bales follow 9 ticks apart.
  No importer warning: the data is valid.
- **A negative load period** (claim 5). With *L* < 0 the counter goes up in every tick, so bales
  follow 9 ticks apart, in both modes of the switch above.
- **No virtual-purifier list, or a list shorter than the position** (claim 12): departure with an
  importer warning, 0 virtual purifiers; the original's result is undefined.
- **Weeders** (the weeder flag) use the same behaviours with different rules (weed land, weed
  storage, a slower step, no money); they are left to a later revision. S-0215 claim 24's row 3 (additive, with no delay of its own; in the original it is the weeder's
  rule in *guard*) belongs to that revision. Claims 13 and 17 add nothing of row 3: their guard
  rules are row 2 cases.
- **Unload without contact stalls** (claim 11). A harvester whose refinery breaks off contact while
  it unloads without a sale (the refinery is destroyed, for example), and that has nothing queued, stays in *unload*
  for good, processed every tick, until an order gives it a behaviour.
  `quirks.unload_without_contact_stalls` (default: the original). With the switch off, the first
  row of claim 11 falls back to rest as if *unload* were not current, so the harvester queues what
  S-0215 claim 15 gives it (row 4: *harvest*, or *guard* for a human-controlled harvester off its
  resource's land type) and starts it at once. No importer warning: no data is involved.
- **A refinery that is not a dock-unload refinery** (claims 8 and 10).
  `quirks.non_unloading_refinery_accepts` (default: original) allows refinery contact
  without that flag. With no other applicable unit-entry exchange, the building never
  directs the harvester to a docking cell: it stays in *enter* without unloading until
  an order or another rule ends it. With the switch off, the building is excluded
  from both refinery choices and refuses refinery contact with that harvester, also
  through an explicit enter order. A valid dock-unload candidate can therefore be
  chosen instead; with none, claim 9's no-choice waiting applies. Other unit-entry
  exchanges are unchanged. The importer warns about a dock-list building lacking
  the flag in either setting. Confidence low for the original stall, measurable by H10.
  The off setting is a design rule, tested only on our engine.
- **Harvester truce.** Out of scope: the original's lobby has a harvester-truce setting (harvesters
  immune to damage), but its skirmish start writes it off in every skirmish (S-0412, claim 1), so
  harvesters take damage normally and nothing in this spec depends on it.
- `quirks.table_root_distance` (S-0204) covers the too-far and wait distances of claim 9. Default: the original's tabulated square root, truncated to an integer. With the switch off, use the exact integer square root, ⌊√*n*⌋, of the squared distance instead.
- `quirks.chop_rounding` (S-0205) covers F3 and F7. S-0204's
  `quirks.table_root_distance` also covers F7's cell distances.
- `quirks.slave_kick_delay_wraps` (claim 14), `quirks.refinery_distance_wraps` (claim 8),
  `quirks.zero_load_period_stalls` (claim 5) and `quirks.unload_without_contact_stalls` (claim 11)
  together with `quirks.harvester_failed_step_off_origin` (claim 13) and
  `quirks.slave_relocation_failed_search_origin` (claim 25) and
  `quirks.slave_negative_capacity_stalls` (claim 26),
  `quirks.zero_capacity_harvester_cycles` (claim 1) and
  `quirks.non_unloading_refinery_accepts` (claims 8 and 10), are this spec's own switches.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| No data key | seeking tests the order-level destination, not the locomotor point; final-track arrival adds no processing or timer reset (claim 3b, F1a) | fixed behaviour; seeking's standard delay and its rate/default are imported by S-0215 claim 6; load timing inputs remain claim 5's | fixed |
| Rules, vehicle type section, `Harvester` | harvester (Terms); eligibility for the vehicle harvesting sprite (claim 5a); whether claim 13's guard rows apply to a slave miner (claim 14) | boolean | no |
| Fixed harvesting asset | vehicle harvesting sprite of claim 5a; loaded as a fixed engine asset, not by a rules or art animation name | sprite file; ordinary asset precedence of S-0005 claim 1 | no alternate harvesting asset is selected |
| No data key | harvesting-display phase and movement gates, frame selection, 15-tick stationary repetition and 30-lepton anchor displacement (claim 5a, F8) | engine behaviour and constants; ground movement gate uses S-0207 claim 2's integer step, whose speed and acceleration inputs and absent-key defaults are imported by S-0207; no separate display-speed threshold or per-type or per-resource animation setting | fixed |
| Rules, `[AudioVisual]`, `ExtraUnitLight` | harvesting sprite uses the same extra brightness as its vehicle (claim 5a) | decimal converted to integer per-mille; S-0104 claim 13 owns the vehicle light rule and its cell/bridge inputs | 0 per-mille |
| No data key | harvesting sprite precedes the vehicle's shadow and body; table sine and cosine select its anchor (claim 5a, F8) | engine drawing rules; no animation-type lighting or playback setting | fixed |
| No harvesting-specific body key | loading keeps the ordinary vehicle image and frame rules (claim 5a) | existing vehicle art and model imports; S-0105 claim 16 owns voxel frames | no extra loading image |
| Rules, vehicle type section, `Weeder` | weeder (Edge cases) | boolean | no |
| Rules, type section, `Storage` | harvester capacity *C* (claim 1); slave capacity, including distinct zero and negative treatments (claim 26) | signed integer, bales; zero vehicle capacity warns in either setting and uses effective 1 bale only with claim 1's quirk off; negative slave values are retained under the default quirk | 0 |
| Rules, type section, `Dock` | dock types (Terms, claim 8) | list of building type names | empty |
| Rules, type section, `Teleporter` | chrono miner (claim 17) | boolean | no |
| Rules, type section, `ResourceGatherer`, `ResourceDestination` | together with an enslaves field, a slave miner (claim 14) | boolean | no |
| Rules, type section, `Enslaves`, `SlavesNumber`, `SlaveRegenRate`, `SlaveReloadRate` | slave infantry type, initial count, regeneration and reload delays in ticks (claims 14, 19, 21) | infantry type name, signed integers | none for the type name; 0 for each integer |
| Rules, vehicle type section, `DeploysInto`; building type section, `UndeploysInto` | conversion types for claim 22's deploy and packing-up attempts | type names, S-0415 | none, S-0415 |
| No data key | initial last-rest tick 0, hidden initial state, 10-tick group period and 30-tick failed-deploy wait (claims 19, 21, 22); emergence position draw (claim 21), existing-slave ownership retention (claim 23), footprint search and axis shifts (claim 24) | engine constants and behaviour | fixed |
| Art, building type section, `Foundation` | site-search rectangle width and height (claim 24), from the deployed or deploys-into type | foundation form owned by S-0406 | S-0406 |
| No data key | missing relocation comparison points, emergence and retry after failed improvement, immediate automatic-pack-up destination (claim 25); slave-place ordering, map-north emergence, two-clock work and busy-state gates (claims 21, 26, 27); directed harvest processing and manual packing gates (claims 22, 28) | engine behaviour, not imported settings | fixed, with claim 25's quirk alternative |
| No data key | temporary pack-up slave creation and draw order (claim 19a), field-seeking and retrying building transitions (claim 27), shared automatic memory and guard spot (claim 28) | engine behaviour, not imported settings | fixed; failed-vehicle retention alternative is S-0415 claim 11's quirk |
| Rules, building type section, `Factory`, `Cloning`, `UnitRepair`, `ConstructionYard` | site-offer guard-spot eligibility (claim 27a): infantry or vehicle factory category, or any of the three flags | factory-category form (S-0003 claim 21), then booleans | no factory, then no for each flag |
| No data key | site-offer refusal during selling, absence of a separate building movement destination, independent move queueing and 450-tick no-op building move processing (claim 27a) | engine behaviour, not imported settings | fixed |
| Rules, infantry type section, `HarvestRate` | loading-processing interval *h*, not the seeking or delivery group-update interval (claims 14, 26) | integer, ticks | 1 |
| Rules, `[General]`, `HarvesterLoadRate` | load step period *L* (claim 5) | integer, ticks | 2 |
| Rules, `[General]`, `HarvesterDumpRate` | dump rate *D* (claim 11) | decimal (S-0003), game minutes | the double nearest 0.016 |
| Rules, `[General]`, `TiberiumLongScan`, `TiberiumShortScan` | long and short scan ranges (Terms, claims 3 to 7) | lepton form: decimal cells × 256 | 8192 and 1536 leptons (32 and 6 cells) |
| Rules, `[General]`, `HarvesterTooFarDistance` | too-far distance of ordinary harvesters (claim 9) | integer, cells | 5 |
| Rules, `[General]`, `ChronoHarvTooFarDistance` | too-far distance of chrono miners (claims 9, 17) | integer, cells | 50 |
| Rules, `[General]`, `SlaveMinerShortScan`, `SlaveMinerSlaveScan` | slave-miner short scan and slave scan ranges (claims 14, 26), with the distinct vehicle, deployed-master and infantry scan tests | lepton form | 1280 and 4096 leptons (5 and 16 cells) |
| Rules, `[General]`, `SlaveMinerLongScan`, `SlaveMinerScanCorrection` | field seeking and relocation improvement margin (claim 22); divide by 256 and truncate toward zero to whole cells | lepton form | 20480 and 768 leptons |
| Rules, `[General]`, `SlaveMinerKickFrameDelay` | kick delay *k* of the wake-up rule (claim 14) | integer, ticks | 2147483647, which wraps (claim 14, F5) |
| Rules, `[General]`, `PurifierBonus` | purifier bonus *b* (claim 12) | decimal, single | the single nearest 0.25 |
| Rules, `[General]`, `AIVirtualPurifiers` | virtual purifiers per difficulty index, original only, and for the neutral and special houses (entry 2, claim 12) | decimal-prefix signed 32-bit integer list of purifier counts (S-0003 claim 23, row C21), 511-byte text limit; no scalar hexadecimal forms; unreadable non-empty items retain a place with 0, empty items are skipped; missing or empty keeps the whole list in force, present non-empty replaces it; item conversion, wrapping and importer warnings are S-0003's | initially empty list (original: undefined result; engine: 0 virtual purifiers, departure, importer warning; claim 12) |
| Rules, building type section, `OrePurifier` | ore-purifier flag (Terms) | boolean | no |
| Rules, building type section, `Refinery` | refinery flag: the refinery-flag building rules of claims 3, 11, 13 and 17 (the idle step-off, the unloading animation and the harvester's wait for it, the chrono miner's guard rule) | boolean | no |
| Rules, building type section, `DockUnload` | dock-unload refinery (Terms, claim 10): directs a harvester to its docking cell, and queues unloading on an accepted track-completion report; a dock-list building lacking it warns in either setting and is excluded from refinery choice/contact with claim 10's quirk off | boolean | no |
| Rules, mobile type section, `SpeedType`, `MovementZone` | locomotion class and movement zone used by the idle step-off (claim 13) and slave field-search connectivity and entry (claim 26); scatter uses the vehicle search settings of S-0207 claim 16 | vocabulary forms owned by S-0211 | S-0211 |
| No data key | no docking-cell exception to harvest seeking, no docking-exchange eviction of an unrelated occupant and no docking-reservation timeout (claims 3a, 10a) | engine behaviour, not imported settings | fixed; processing rates are those of claims 3 and 10 |
| No data key | failed step-off destination (0, 0), track-completion report timing, facing-independent arrival acceptance and forced, no-threat-point, not-move-anywhere scatter options (claims 10, 13) | engine behaviour, not imported fields | fixed rules of claims 10 and 13 |
| Rules, building type section, `Weeder` | weed-flag building: the idle step-off of claim 13 (and the weeders' docking, Edge cases) | boolean | no |
| Rules, building type section, `FreeUnit` | free unit (Terms, claim 18; S-0408 owns the field's cost uses) | vehicle type name | none |
| Rules, country section, `IncomeMult` | income factor *M* (claim 12) | decimal, single | 1.0 |
| Art, building section, `QueueingCell` | queueing shift (Terms, claim 9); it does not move the docking cell (claim 10) | two integers *x*, *y*, cells | 0, 0 |
| Art, building section, `NumberOfDocks` | dock slots (Terms, claim 10) | integer; values below 1 give 1 | 1 |
| Art, building section, `PreProductionAnim`, `PreProductionAnimDamaged`, `SpecialAnim`, `SpecialAnimDamaged`, `ProductionAnim`, `ProductionAnimDamaged` | docking animation, payment animation and unloading animation, with their damaged forms (Terms, claim 11) | animation type name | none |
| Rules, vehicle type section, `UnloadingClass` | deployed image type of a harvester type: the image its body is drawn with while unloading (claim 11, *The harvester's image while unloading*; S-0517 claim 12 owns the field and its other use) | vehicle type name | none: the harvester's own image |
| Rules, `[AudioVisual]`, `ConditionYellow` | the yellow health threshold that selects the damaged forms (Terms; owned by S-0003, claim C9) | percentage (S-0003) | S-0003 |
| Rules, `[Unload]`, `[Harvest]`, `[Enter]` behaviour sections (S-0215) | rates of the standard delays of *unload*, *harvest* and *enter* | owned by S-0215 | S-0215 |

The absent-key defaults of the engine-wide rules fields are the executable's own constants. For the
per-type fields (slave counts and delays, queueing shift, dock count, income factor, harvest rate),
the default is the value the original gives a type before any data is read.

## Measurement scenarios

All scenarios run under the measurement setup on purpose-made skirmish maps with one human house and
one bystander computer player. Each scenario's observed area is the region that holds its test
objects (the field, the refineries and the harvesters of the part); the bystander is kept away from
it, and the void rule of the glossary applies. H5 part C and H6 part c use the bystander's own
objects, which the void rule exempts there, and H7 part (g) uses the bystander's slave miner
and its slaves, which the void rule exempts in that part only. Named test data is the map's
override sections; values
are invented. Tick
numbers, behaviours and phases, loads (pips, or a recorder that reads them), cell stages, the balance
and the scenario stream's draw count come from a recorder (a read-only memory recorder, glossary
*recorder*), and orders are given by scripted input (glossary, *measurement setup*), as the project
lead allowed. Unless a scenario says otherwise, the harvester is the human
house's, the refinery has stock art, there is one harvester, and resource cells do not grow (growth
and spread fractions 0). Unless stated otherwise, use game-speed index 2 at 800 × 600,
with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).

**H27. Final field track to the first load step (claim 3b, F1a; Q-0900).**
- *Setup:* a tiny level skirmish map with an isolated resource patch, one ordinary
  stock-data driving harvester of the human house and its dock-type building,
  and the bystander kept outside the observed area. Use stock rules and art,
  game-speed index 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Choose a patch with
  enough resource for a first bale. No approach destinations, teleporting,
  slave group, combat or replacement orders during each observed arrival.
- *Procedure:* from at least six different starting distances, issue harvest
  orders to a named patch cell. Repeat over ten recorded scenario seeds and
  several order ticks. Use scripted input and the read-only recorder. Retain
  at least five arrivals whose first post-completion processing is in *T* + 1
  and five whose first processing is later; expand the distance/order-time
  sample if needed. Void arrivals whose preconditions in claim 3b fail.
- *Observable:* final track completion tick *T*, current and queued behaviour,
  phase, remembered cell and approach destinations, order-level destination
  presence and kind, locomotor point presence, due-processing ticks, selected
  delays, counter and pacing period, loading-start tick *S*, first
  loading-phase processing and first bale tick. Record destinations before
  due processing and after locomotion, not just once per tick. Record the
  scenario random state around each seeking processing. Tabulate the
  arrival-to-first-bale histogram separately by distance and seed.
- *Expected:* completion clears the locomotor point but not the cell order
  destination. If processing is due in *T* + 1, it retains seeking and takes
  one standard-delay draw, then locomotion removes the destination that tick.
  The following processing starts loading. If the first post-arrival
  processing is in *T* + 2 or later, it starts loading at that first processing.
  Loading starts with counter 0 and period 2 ticks. The next processing is
  *S* + 1, the first load step and bale are *S* + 19, and every valid arrival
  satisfies F1a. Compare the histogram with the conditional predictions
  computed from its recorded processing ticks and delays; do not fit a
  uniform arrival-residual distribution.
- *Falsified if:* loading starts while the cell order destination remains at
  the seeking test merely because the locomotor point is absent; a due
  *T* + 1 processing already starts loading, arrival resets the timer,
  a first eligible processing from *T* + 2 onward waits a further interval,
  or a valid first bale differs from *S* + 19. Failure to obtain either
  timing class leaves that class unmeasured, not confirmed.

**H1. One full cycle: load, travel, unload, credits (claims 1, 2, 3, 5, 6, 11, 12, 16; F1, F2).**
- *Setup:* load period *L* = 3, dump rate 0.02 (900 × *D* is just below 18 in single precision, so
  the counter must reach 18), the unload behaviour's rate set so that its standard delay is at
  most 5 ticks, a harvester of capacity 10, a field of 15 ore cells at stage 11, 10 cells from a
  refinery, value per bale 20, income factor 1, no purifier.
- *Procedure:* order the harvester to harvest a field cell and let it run two cycles.
- *Observable:* the tick of each bale (the load rises by one); the tick at which *loading* ends and
  the tick at which it first moves; the behaviour it is in during each part of the cycle (*harvest*, *enter*, *unload*); the tick
  of reaching the docking cell, of facing east, of starting to unload, of each balance change, of
  the change to *finishing* and of breaking off contact; the balance changes; the cell the
  harvester returns to; the scenario stream's draws per tick.
- *Expected:* bales 19 ticks after loading starts and then every 28 ticks (F1); the *loading* phase
  ends 28 ticks after the tenth bale (the failing load step of F1) and the harvester starts
  driving no earlier than the next tick, not at once after its tenth bale; one balance change of 200 credits per
  cycle, exactly 19 ticks after unloading starts (F2, as the standard delay at the start is at most
  5); the change to *finishing* exactly 18 ticks after that payment (the extra interval of claim
  11), and contact broken no earlier than the next tick; it faces map east and the refinery lies map west of
  it while unloading; the three behaviours occur in the order *harvest*, *enter*, *unload*,
  *harvest*; the second cycle starts on a cell within 6 cells of where the first load ended; draws
  occur only in ticks in which the harvester has a standard-delay processing (none during the
  loading ticks and none while unloading after the start).
- *Falsified if:* bales every 27 or 9*L* ticks; the *loading* phase ending in the tick after
  its last bale; the payment split into several changes for one tiberium or a payment other than
  200; the payment coming at any tick other than 19 ticks after unloading starts, or the change to
  *finishing* at any tick other than 18 ticks after the payment; a draw in a loading tick or in an
  unloading tick after the start; a second cycle starting more than 6 cells away.
- *Variants:* (a) *L* = 2 confirms 9*L* + 1 = 19. (b) dump rate 0.04 (900 × *D* is just below 36 in single
  precision): the first payment comes exactly 37 ticks after unloading starts, and the change to
  *finishing* exactly 36 ticks after the payment (the one-tick difference of F2); falsified by any
  other tick. (c) *L* = 0: exactly one bale loads in 3000
  ticks; and *L* = −1 (if the data layer accepts it): bales 19 ticks, then 9 ticks apart. On our
  engine with `quirks.zero_load_period_stalls` off, *L* = 0 gives bales 19 ticks, then 9 ticks
  apart (no oracle part). A second bale with *L* = 0 in the oracle, or any interval
  other than 19 ticks initially and 9 ticks thereafter with *L* = −1, falsifies
  the respective rule. (d) Capacity 0
  (the data layer may refuse it; then it is `unmeasured` for the original, with that reason): the
  harvester goes to the field and starts loading; expected: no bale at any tick, the harvester
  *returning* exactly 19 ticks after loading starts (the tick of its first load step),
  with the counter and period unchanged by that failure, no resource-stage or balance
  change in 3000 ticks, and the cycle repeating (it docks and returns to the field again).
  A different transition tick, counter reset, period restart, resource reduction, loaded
  bale, payment or failure to repeat the cycle falsifies the hypothesis.
  On our engine only, repeat with `quirks.zero_capacity_harvester_cycles` off and
  compare against an otherwise identical capacity-1 control: one bale at tick 19,
  returning 28 ticks later with this setup's *L* = 3, and payment for one bale under F3.
  Any difference from the capacity-1 control falsifies the off-setting rule. Both settings
  must warn at import. These are conformance tests, not oracle measurements. (e) The three animations: the refinery's
  art gets a docking animation of 30 ticks, a payment animation of 200 ticks and an unloading
  animation of 60 ticks (animation types of known length). Run (i) with these, (ii) with a
  docking animation of 400 ticks, (iii) without an unloading animation. Expected, with *f* the
  tick of the change to *finishing* and *e* the tick at which the unloading animation ends: (i)
  contact is broken in tick *e* or *e* + 1, and not before *f* + 60; (ii) the same tick as (i)
  relative to *f*; (iii) contact is broken in tick *f* + 1. Falsified if run (ii) differs from run (i), if
  in run (i) contact is broken before the unloading animation has ended or more than one tick after
  it, or if in (iii) it is broken later than *f* + 1.
  (f) Dump rate 0 and then −0.02, with a mixed load of two tiberiums: the first
  payment occurs at the first processing after the unloading start row, then the
  second payment and the change to *finishing* follow one tick apart. Any two
  payments in one processing, or a longer gap with uninterrupted processing,
  falsifies this boundary of F2.
  For variant e, repeat with health above and at or below the yellow threshold,
  supplying distinguishable damaged versions of all three animations. The damaged
  version must be selected exactly in the at-or-below runs; a different selection
  falsifies the damaged-form rule. Test a dock-unload building without the refinery
  flag separately: its docking and payment animations still play, but it supplies
  no unloading-animation wait. A missing docking animation solely because this
  flag is absent falsifies that part of claim 11.
- *Runs:* 3 for the main run, 1 for each variant and animation/health control.

**H2. Field choice (claim 4).**
- *Setup:* a clear cell *Q* with the harvester starting 4 cells away from it. The harvester is
  ordered to harvest at *Q*, which is not a resource cell. It drives there with a destination, and
  on arrival the destination is gone and no remembered cell exists, so its next seeking
  processing runs the ring search from *Q* (claims 2 and 3). Around *Q*: ring 2 holds an ore cell at
  stage 11 and a gem cell at stage 0 (gems' value per bale more than 12 times ore's in the test
  data); ring 3 holds a gem cell at stage 11; a ring-1 ore cell is blocked by a vehicle of the
  human house.
- *Observable:* the cell at which the harvester starts loading, and its path (cells entered) after
  arriving at *Q*.
- *Expected:* it goes to the ring-2 cell of greater worth, and never to ring 3 while ring 2 has an
  acceptable cell; with the ring-1 cell freed, it goes to ring 1. Repeat with two equal ring-2
  cells at (*x* − 2, *y* − 2) and (*x* + 2, *y* − 2), *Q* = (*x*, *y*): it goes to
  (*x* − 2, *y* − 2).
- *Falsified if:* it picks by distance within the ring, picks the blocked cell, or goes to ring 3.
- *Variants:* (b) the entry test: the ring-1 ore cell is occupied in turn by a stationary vehicle of
  the human house, a moving vehicle of the human house passing through it, and a stationary enemy
  vehicle of the bystander, each with a ring-2 ore cell available as the alternative. Expected: a
  ring-1 cell is chosen exactly when S-0209 claim 5's outcome for the occupant is *clear*.
  Falsified if a cell with any other outcome is chosen, or a *clear* one is skipped. (c) the movement
  zone: a ring-1 and a ring-2 ore cell lie across an impassable stretch (water for a ground vehicle)
  that no path crosses, and a ring-3 ore cell is reachable. Expected: the harvester loads at the
  ring-3 cell; falsified if it picks either unconnected cell.
- *Runs:* 3 for the main run, 2 for each variant.

**H3. Exhausted cell, partial load, driving on (claims 5, 7).**
- *Setup:* capacity 20, the short scan field set by the test data to 1536 leptons (6 cells), a
  single isolated ore cell of stage 5.
- *Observable:* the load per tick, the cell it stands on, the tick it starts driving, the tick of
  the bale after arriving at another cell.
- *Expected:* 5 bales loaded, then it returns and unloads 5 bales' worth. With a second cell
  4 cells away it moves there first and keeps loading; with one 8 cells away (beyond the short scan
  of 6) it returns with 5. With *L* = 1 and the second cell 4 cells away (a drive of more than
  9*L* + 1 = 10 ticks), the first bale there is loaded at the first processing after arrival, not
  10 ticks later.
- *Falsified if:* it waits on the empty cell, ignores the cell 4 cells away, or waits a full
  9*L* + 1 ticks after arriving.
- *Runs:* 3 per variant.

**H4. Refinery choice, queueing, primary building, power switch, distance wrap (claims 8, 9, 10).**
- *Part a (nearest, slots, too-far).*
  - *Setup:* two refineries, A at 8 cells and B at 15 cells from a field; three harvesters,
    and the too-far field set to 5.
  - *Observable:* which refinery each harvester docks at; where the others wait; the tick at which a
    waiting harvester starts docking.
  - *Expected:* only one harvester holds contact with a refinery at a time; a harvester farther than
    5 cells from its nearest refinery drives to that refinery's queueing cell (or the nearest free
    cell) and waits within 3 cells of it; when A is busy and B free and within 5 cells, a harvester
    reserves B.
  - *Falsified if:* two harvesters unload at one refinery at once, or a harvester reserves a
    refinery from farther than 5 cells.
- *Part b (the 768-lepton wait).* A refinery busy with another harvester; the waiting harvester
  stands 2 cells from it (too-far field 1): expected, it stays where it is; stands 4 cells from it:
  expected, it drives to the queueing cell. Falsified if it moves at 2 cells or stays at 4. The
  equality at 3 cells is part f.
- *Part c (primary building, ties and the holding order).*
  - *Setup:* too-far field 50. The refinery type is also made a factory type in the test data.
    Each refinery is produced and placed before the next one's production starts, so the holding
    order (Terms) is the placing order. Refineries
    A (near) and B (farther), both free; the primary order (S-0213, claim 13) is given to B. Two
    runs differ only in the placing order: (i) A first, then B; (ii) B first, then A.
  - *Expected:* (i) the harvester docks at B (B is later than A in the holding order and primary,
    so it replaces A); (ii) it docks at A (A comes after the primary B in the holding order, and
    is nearer than B). With B's mark removed, both runs dock at A.
  - *Falsified if:* in (i) it docks at A, or in (ii) it docks at B, or in either run with the mark
    removed it docks at B. If both runs give the other's result, the holding order is not the
    placing order (the order is then the reverse, or by another key): record it and revise Terms.
  - *Tie setup:* two refineries at exactly equal distance, neither primary, placed in turn: (i) X
    first then Y; (ii) Y first then X. *Tie expected:* in both runs the one placed first. *Tie
    falsified if:* the one placed second.
  - *Cross-type tie run:* two refinery test types, R1 of line kind building and R2 a copy whose
    test data makes it a *defence* (S-0403's line kinds), so both can be in production at
    once. Start R1 first but place R2 first, at exactly equal distances, neither primary.
    With dock list R1 then R2, expect R1; with that list reversed, expect R2.
    Choosing the second dock-list type falsifies the across-type tie rule.
    This does not test when a building joins holding order: the tie is across types,
    not within one type, regardless of production or placement order.
  - *Order after a removal:* three refineries W, X, Y placed in that order, X and Y at equal
    distance and W anywhere that does not decide the choice (farther than both), W then sold:
    expected, the harvester docks at X (the others keep their relative order). Then X is sold and
    a new refinery X′ is placed at X's position; expected, it docks at Y (X′ comes after Y).
    Falsified if it docks at Y before the second step, or at X′ after it.
  - *Falsified, for the primary rule, if:* the primary refinery is passed over although no nearer
    candidate comes after it in the holding order, or a non-primary building farther away than the
    nearest candidate after the primary one is chosen.
- *Part d (power switch).* The nearer refinery is switched off by its owner (S-0213, claim 13,
  the power switch): the harvester docks at the other one; with the refinery switched on again, and separately
  with the house at low power, it docks at the nearer one. Falsified if the switched-off refinery
  is used or low power makes the harvester refuse the nearer one.
- *Part e (distance wrap, original default).* A map large enough for two connected,
  accepting refineries of one dock type, neither primary: A about 150 cells and B about
  190 cells from the harvester along one axis; too-far field 50. Expected: the
  harvester drives to B's queueing cell (B's wrapped distance is negative). Falsified if it goes to A.
  If the original cannot load a map that large, this part is dropped and the wrap stays
  `unmeasured` (reason: it needs a map beyond the original's size). On our engine only,
  repeat with `quirks.refinery_distance_wraps` off: it drives towards A, whose exact
  squared distance is smaller. Choosing B falsifies the off-setting rule. Repeat part c
  and its tie runs with the switch off as well: their primary and tie outcomes must not
  change. These off-setting runs are conformance tests, not oracle measurements.
- *Part f (the equality at both limits).* The refinery's foundation centre lies at a cell centre
  and the harvester stands at a cell centre, on level ground, so the 3-D distance is a whole number
  of cells times 256 leptons with no remainder. (i) Too-far field 5, harvester exactly 5 cells
  (1280 leptons) from a free refinery along a row: expected, it reserves the refinery (contact,
  *requesting*); falsified if it makes the waiting choice instead. At 6 cells: expected, no
  reservation. (ii) Too-far field 1, the refinery busy, harvester exactly 3 cells (768 leptons)
  away along a row: expected, it stays where it is; falsified if it drives to the queueing cell
  (unless S-0204's table root of 589824 is above 768, which the run records). At 4 cells:
  expected, it drives.
- *Part g (construction state and being sold).* The only refinery is a second one placed while the
  harvester is returning: (i) a refinery in its construction state, with an existing refinery
  busy or absent: expected, no contact until the refinery is complete; falsified if the harvester
  holds contact with it earlier. (ii) A sell order given to an idle refinery while a harvester is
  returning to it: expected, no contact is taken during the sale; falsified if one is.
- *Part h (being erased, low confidence).* A refinery that an erasing weapon has locked, with the
  harvester returning to it: record whether it takes contact. Hypothesis: it is refused as in part
  g; falsified if contact is held. Either result is recorded for claim 10.
- *Part i (two dock types).*
  - *Setup:* the harvester's dock list names two refinery types, T1 then T2; T2 is also a factory
    type. Refineries are placed in this order: X of type T1 (near), Q of type T2 (far), then C of
    type T2 (between X and Q in distance, nearer than Q and farther than X); Q is made primary.
    Too-far field 50.
  - *Expected:* run 1 (Q placed before C): it docks at X. Within T2, C comes after the primary Q
    and is nearer, so C is T2's best and is not primary; across types, C is not nearer than X and
    not primary, so X stays. Run 2 (the placing order of Q and C swapped, C before Q): it docks at
    Q. Within T2, Q comes after C and is primary, so Q is T2's best and counts as primary; across
    types, it replaces X.
  - *Falsified if:* in run 1 it docks at C (one flat list with one last primary building would
    choose C) or at Q; in run 2 it docks at X or C.
- *Part j (capture and the holding order).* The human house owns refineries X (placed first) and
  Y at exactly equal distances from its harvester, neither primary. A refinery Z of the bystander,
  at the same distance as X and Y, is captured by an engineer of the human house (Z joins at the
  end). Then X is sold. *Expected:* before the sale the harvester docks at X; after it, at Y (Z
  comes after Y). Repeat with Z captured before Y is placed: after X's sale it docks at Z.
  *Falsified if:* the docking refinery after the sale is the other one in either run.
- *Part k (connectivity cells).* A refinery R on an island that a ground vehicle cannot reach
  (water around it, so its reference-point cell is in another reach region for the harvester's
  movement zone) and a reachable refinery S farther away. *Expected:* the harvester never chooses
  R, even when R is nearer; when the water is replaced by a land bridge, it chooses R. *Falsified
  if:* it chooses R while cut off, or ignores R when connected.
- *Runs:* 3 for a, 2 for each of b to k.

**H5. No field, no refinery (claims 3, 13).**
- *Part A:* a harvester whose field is exhausted with no other resource within the long scan range.
  *Observable:* the harvester's behaviour per tick, its cell. *Expected:* it unloads, finds nothing,
  and 105 ticks after finding nothing it is in *guard*, standing still (off the cell of the refinery-flag building); it
  stays there for 3000 ticks. *Falsified if:* the human's harvester resumes by itself, it is not in
  *guard* within 105 ticks plus one standard delay, or it still stands on the cell of a
  refinery-flag building.
- *Part B:* the house's only refinery is sold while the harvester loads (1-tick processing
  interval). *Observable:* the harvester's behaviour per tick from the tick the refinery has gone.
  *Expected:* *guard* within 2 ticks, and it stays in *guard*. *Falsified if:* it is not in *guard*
  2 ticks after the refinery has gone, or it loads another bale.
- *Part C (original only, no record set):* the bystander computer player owns a refinery and a
  harvester (its own objects, in the observed area), and none of its harvesters has gone idle. The
  map pre-places the harvester in *guard* beside a reachable field. Do not issue
  a human stop order to that house: S-0213 claim 3 does not allow it to take effect.
  *Observable:* its behaviour and due processing ticks, with computer-player
  behaviour changes logged separately. *Expected:* its first qualifying *guard*
  processing queues *harvest* with a 1-tick interval, and harvesting resumes under
  the ordinary start rules. *Falsified if:* that processing queues no *harvest* or
  uses another interval. A computer-player behaviour change before the observation
  makes that run inconclusive, not evidence of automatic resumption.
- *Part D (original only, record set and never cleared):* the same bystander house, its harvester
  on an isolated field so that it exhausts the field and goes *idle* (the house's no-field
  record is now set). A new ore cell is placed within its scan range by a trigger
  later. *Observable:* its behaviour per tick over 3000 ticks after it went idle,
  distinguishing guard processing from independent computer-player behaviour changes.
  *Expected:* after its idle wait it reaches *guard*, and this claim's guard rule
  does not queue *harvest*, even after the new cell appears. *Falsified if:* a
  qualifying guard processing queues *harvest* by this rule. Independent computer
  intervention makes the affected interval inconclusive. A second same-house
  harvester observed naturally in *guard* after the record is set should have
  the same result, but no reliable fixture for that overlap is established here.
  That cross-harvester case remains `unmeasured`. No human stop order or runtime
  write may manufacture the overlap.
- *Runs:* 3 for each part.

**H6. Payments, purifiers, mixed loads and no cap (claims 11, 12, 15; F3).**
- *Setup:* dump rate 0.02 (so the counter must reach 18 between payments), value per bale 30 for
  ore and 70 for gems; one purifier, bonus 0.25; income factor 1.0; a harvester of capacity 10
  loads 4 bales of gems and 6 of ore.
- *Observable:* the balance at the end of every tick, with its tick.
- *Expected:* part a, the main run: the balance rises by 225 (180 for the ore, index 0, and a
  bonus of 45) in the tick of the first payment, and by 350 (280 for the gems and a bonus of 70) in
  the tick 18 ticks later. Part b: ore only, value per bale 33, a country income factor of 1.1,
  6 bales, one purifier: the balance rises by 271 in one tick (217 and then a bonus of 54:
  trunc(33 × single(1.1) × 6) = 217 and trunc(33 × single(1.1) × 1.5) = 54, double products
  under the round-toward-zero mode, F3; a single truncation of the real sum, 272.25, would give
  272).
- *Falsified if:* the ore and the gems are paid in the same tick, or the gems before the ore, or
  other amounts; in part b, a rise of 272 or any other amount.
- *Part c (original only, virtual purifiers):* the bystander computer player's own refinery and a
  harvester of it loading 10 ore bales, no purifier, and the virtual-purifier list set to 2 in every position of
  the test data. *Expected:* a main payment and a bonus payment as F3 gives it for
  *x* = 2 × 0.25 × 10 = 5.
  *Falsified if:* no bonus payment. (The human house never gets one: repeat with the human house
  and a field and expect no bonus.)
- *Part d (when a purifier counts):* a purifier under construction gives no bonus at an unload; the
  same purifier completed does, also while the house is at low power; after it is sold, none.
  *Falsified if:* a bonus is paid during construction or after the sale, or none is paid at low
  power with the purifier complete.
- *Part e (no cap):* the human house has no purifier and no silo or other storage building, starts with 20000
  credits, and its harvester has capacity 100 and value per bale 1000 (on a field of 30 cells at stage
  11), so that every unload pays 100000; three unloads in a row, and a repeat with a storage building added. *Expected:* each
  unload raises the balance by exactly 100000, with and without the storage building, up to a
  balance of 320000 (below 2³¹). *Falsified if:* any payment is cut, delayed or refused for lack
  of storage.
- *Part f (virtual purifiers of the neutral house):* the map pre-places a refinery and a harvester
  owned by the neutral house next to an ore field (the harvester harvests by itself under claim
  13's computer-controlled rule), no purifier; the virtual-purifier list of the test data is
  1, 2, 4 (three distinct entries), bonus 0.25, a load of 8 ore bales, value per bale 30, the
  neutral country's income factor 1.0; the lobby's computer difficulty is varied over its three
  entries in three runs. *Observable:* the neutral house's balance per tick (recorder).
  *Expected:* in every run a main payment of 240 and a bonus payment for *x* = 2 × 0.25 × 8 = 4
  bales, 120 (the second entry). *Falsified if:* a bonus of 60 or 240 (the first or third entry),
  no bonus, or a bonus that follows the lobby setting. Repeat with the same objects
  owned by the special house: the same payments and falsifiers apply.
- *Runs:* 3 for each of a and b, 1 for each of c, d and e,
  1 per lobby setting and non-player house for f.

**H7. Slave miner (claim 14).**
- *Setup:* a slave miner of a human house with 3 slaves of capacity 2 and harvest rate 40, kick
  delay 300, the slave-miner short scan set by the test data to 2048 leptons (8 cells), an ore
  field 5 cells away, the miner on a cell that is not ore. The miner is given a stop order at
  tick 100, which forces *guard* (S-0213, table T1); the recorder logs the start tick of that
  *guard*, *s*<sub>m</sub> (S-0215, claim 3), and the first processing of *guard* follows at once.
- *Observable:* the tick at which each slave leaves, *T*₁ for the first; the ticks between a
  slave's bales; the payments to the house on delivery; the time slaves spend inside before leaving
  again; a damaged slave's health after its reload; the replacement time of a killed slave.
- *Expected:* the group's wake transition occurs at a processing of *guard* with
  *t* > *k* + *s*<sub>m</sub> and *t* > *k* + *s*<sub>g</sub>. After the stop order,
  *s*<sub>g</sub> is the stop tick (the group reset), so it cannot wake before tick 401.
  Record that transition separately from emergence: movement, deployment and claim 21's
  group updates can postpone *T*₁. Once a slave is harvesting, expect 40 ticks between its
  bales, payments under claim 12, timer completions at group updates (claim 21), and full
  health after reload.
- *Falsified if:* the group wakes or slaves emerge at or before tick 400, bale intervals
  differ, or timer completion occurs between group updates. A failure to emerge must be
  separated from a failure to wake; it does not imply an unknown initial last-rest tick.
- *Variants:* (b) the wrap: the miner stands on an ore cell (so the group's rest tick plays no
  part) and the kick key is absent (the default 2147483647); the stop order is given at tick 100
  as above. Expected: the group wakes within 10 standard delays of *s*<sub>m</sub>, because the
  kick sum wraps negative for *s*<sub>m</sub> ≥ 1; falsified if it has not woken by then.
  Emergence is recorded separately under claim 21. If
  the recorder shows *s*<sub>m</sub> = 0 for a miner that stands in *guard* from the start of the
  map, that run expects the slaves never to leave, since 2147483647 + 0 does not wrap. On our engine with
  `quirks.slave_kick_delay_wraps` off, the slaves of the *s*<sub>m</sub> = 100 run never leave
  (no oracle part). (c) The miner standing on an ore cell, with no field within the short scan,
  kick delay 300: the group still wakes once *t* > *k* + *s*<sub>m</sub>; falsified if it
  stays at rest despite a qualifying processing.
  (d) The miner on a non-ore cell with the field 9 cells away (beyond the short scan): the slaves
  do not leave in 3000 ticks; falsified if they do. (e) The group's rest tick: after the slaves
  of the main run have all returned at tick *r* and the group is at rest again, with the miner
  still in *guard* on a non-ore cell: the next leave comes at a tick after *r* + *k*; falsified if
  slaves leave at or before *r* + *k*. (f) The deployed miner: the miner is ordered to deploy next
  to the field; the cell where the slaves reach the miner to deliver is the foundation cell of
  claim 14 (*W* − 1, ⌊*H*/2⌋) counted from its top-left cell; falsified if they deliver at another
  cell. Damage an outside slave without killing it before delivery. At the first group
  update after its reload delay expires, expect full strength, not its delivery health.
  A different health or restoration between group updates falsifies that part of claim 14.
  (g) Q-1990: a skirmish with one bystander computer player. Place a slave miner of the person
  and one of the computer player, each with no destination, kick delay 300, on a clear non-ore
  area more than the short scan from every field and within the long scan of one ore field.
  The person's miner is given a stop order at tick 100, as in the main run. The computer
  player's miner gets no human stop order (S-0213 claim 3 does not let one take effect, as in
  H5 part C): the map pre-places it in *guard* with no destination, and the recorder logs the
  start tick of that *guard* as its *s*<sub>m</sub> (S-0215, claim 3). Record each group's
  first wake, the slaves' first emergence tick and the miner's first move tick, for 3000
  ticks, and log the computer player's behaviour changes and destinations for its miner
  separately. Expected: the computer player's group wakes at the first processing of *guard*
  with *t* > 300 + *s*<sub>m</sub>, with the miner still in the *guard* that started at
  *s*<sub>m</sub>, and its miner then moves towards a site near the field. The person's miner
  and slaves stay put for the whole run, as in part (d). Falsified if the computer player's
  group wakes at or before 300 + *s*<sub>m</sub>, or stays at rest past the first qualifying
  processing with no behaviour change, or if the person's group wakes. A wake counts for this
  rule only when no behaviour change or destination of the computer player's own came first.
  Independent computer intervention makes the affected interval inconclusive (as in H5 part
  D). The miner type used has neither the harvester nor the weeder flag (check the data; a type
  with either is the other case of claim 14 and does not test this part). Claim 13's rows then
  do not apply, so before the wake the recorder shows no *harvest* queued for the computer
  player's miner and no new *guard* start, and *s*<sub>m</sub> stays at its logged value. A
  *harvest* queued in that interval with no order of the computer player's own falsifies
  that part of claim 14. The void rule exempts the computer player's slave miner and its slaves
  in this part and applies to the bystander's other objects. The person's run uses part (d)'s
  expectation, and only the computer player's run tests the owner alternative.
- *Runs:* 3 for the main run, 1 for each variant.

**H8. Chrono miner (claim 17).**
- *Setup:* a chrono miner *H* far (40 cells) from its refinery, chrono too-far field 50. A named
  test harvester type *Q*, a copy of the chrono miner's type with the teleporter flag off.
  *Q* keeps the teleporting locomotor, so S-0207 claim 28 does not apply to it and its
  teleporting locomotion stays active. *Q* is used only for the seeking drop, in parts (d),
  (h) and (i), which test no rule that needs the teleporter flag. The rules keyed to the flag
  (the chrono too-far field, the queueing-cell choice, the returning drop and the *guard*
  rule: the first three expected results and parts (e) to (g)) are tested with *H* only. Record the
  active locomotion of both before each travel or seeking test. Separately use S-0417 claim
  16's U15 to test a vehicle-factory-built miner's first distant ore leg and later
  restoration, and distinguish it from the refinery free-unit path of claim 18.
- *Observable:* the cell per tick (a teleport is a jump of more than 2 cells in one tick), the
  behaviour and phase, and the active movement kind.
- *Expected:*
  - When full, *H* reserves the refinery and teleports to the docking cell. Its active movement
    kind is driving from that jump's tick.
  - With the field set to 10, *H* drives towards the queueing cell.
  - Standing next to its owner's refinery in *guard*, *H* queues *harvest* by itself.
  - Part (d): ordered to harvest a cell 10 cells away while a richer cell lies next to it, *H*
    keeps the order's destination, drives to the ordered cell and loads there. Repeated with
    *Q*: it drops the order's destination by its first seeking processing, with either timing
    outcome of part (h): it loads at the cell that the ring search from where it stands finds,
    or it jumps to the ordered cell once and then searches from there.
  - Part (e): while *H* is moving in *returning* and a refinery would accept it, its movement
    stops and it reserves that refinery on its next processing.
  - Part (f): with the refinery busy, *H* 2 cells (512 leptons) from it and the too-far field
    1, it drives to the queueing cell, although an ordinary harvester stays there (H4 part b).
  - Part (g), the "full and still moving" trigger: a full *H* is returning to its only
    refinery over a long way (40 cells), and that refinery is sold while the miner is moving,
    so claim 13 puts it into *guard* with its locomotor still moving. At every processing of
    *guard* at which the recorder shows the locomotor moving, the behaviour one tick later is
    *harvest*, which falls back to *guard* at once (claim 13, as the house owns no refinery).
    Once the locomotor has stopped, it stays in *guard*. If no processing of *guard* ever shows
    the locomotor moving (the miner stops at once when the refinery goes), the part shows
    nothing, and the trigger stays `unmeasured` for that reason.
  - Part (h), the seeking drop's timing for a harvest order (Q-1130): as part (d) with *Q*, the
    ordered cell 10 cells away and richer ore next to it. Record every tick from the order until
    the first load: the cell, order-level destination, the locomotor's goal, its moving state,
    behaviour and phase, and the time of each processing. The destination is gone within a few
    ticks. Either *Q* loads at the nearby ore (the processing came
    first), or it jumps to the ordered cell once and then searches from there. Both outcomes are
    recorded; neither falsifies the drop itself, and the observed one settles the `unmeasured`
    timing in claim 17.
  - Part (i), the remembered cell and the retained landing point: let *Q* unload at a refinery
    with ore 10 or more cells away from its first loading cell, so that it has a remembered
    cell. Record the same fields over the first seeking processing after the unloading, plus
    the occupancy of the remembered cell (try to move a second vehicle onto it, or read the
    recorder's vehicle-present mark). *Q* does not jump to the remembered cell: its cell does
    not change in the processing's tick unless the ring search from where it stands finds
    another cell. The remembered
    cell's reservation stays marked until *Q*'s next accepted destination. Repeat with *Q*
    disabled (electromagnetic pulse) at the processing: the drop still removes the destination,
    and no landing point or mark appears.
  - Part (j), *H* after unloading: over three loads, its trip to the remembered cell or to the
    cell the ring search finds is driven every time, and its only jumps are to the docking cell.
- *Falsified if any of these holds:*
  - *H* jumps anywhere other than to a refinery's docking cell during *enter*, or does not
    make the docking jump under claim 10's qualified docking conditions.
  - *H* stays in *guard* next to its refinery.
  - in part (d), *H* drops the ordered cell or jumps, or *Q* keeps the ordered cell as its
    destination after its first seeking processing.
  - *H* keeps moving in part (e), or stays where it is in part (f).
  - in part (g), a processing of *guard* that shows the locomotor moving and is followed by a
    behaviour other than *harvest*, or *harvest* after the locomotor has stopped.
  - in part (h), the ordered cell still set as *Q*'s destination after the first seeking
    processing.
  - in part (i), *Q* jumps to the remembered cell, a destination is still set after the seeking
    processing, the reservation moves back to *Q*'s cell at the drop, or the drop happens only
    at the first processing after *harvest* starts (a destination that the search sets at a
    later seeking processing is gone at the next seeking processing, unless the jump was
    blocked).
- *Runs:* 3.

**H9. Stop order and *return* (claim 2).** A harvester in *loading* and another in *returning* are
given a stop order (S-0213). *Observable:* the behaviour one tick later, and the harvester's position
and load over the next 3000 ticks. *Expected:* *guard* at once; they do not resume by themselves,
and the load stays. Across all runs of H1 to H8 the recorder also logs every behaviour the harvesters
are in. *Falsified if:* a stopped harvester is not in *guard* one tick later or resumes harvesting
by itself (human owner), or if *return* appears in any run (then claim 2's low-confidence part
is wrong). *Runs:* 3.

**H10. Docking (claim 10).**
- *Setup:* a human house with one refinery and one harvester on open level ground, no other units.
  The harvester is given a full load by an ore field next to it. Two runs per refinery art: the
  stock refinery, and a test refinery type with the same art except a larger foundation (one
  more cell in each direction). A further run with the test data's refinery having the dock-unload
  flag off and no alternative entry exchange (no weed-unloading, absorption, repair
  or reload role). Retain its place in the harvester's dock list.
- *Observable:* per tick, from returning through payment or 3000 ticks after contact
  in the no-flag run: cell, body facing, locomotor movement, current and queued behaviours,
  destination, contact, load, balance and tick of each processing of *enter* (recorder).
- *Expected:* the harvester drives forwards to the cell (*x*₀ + 3, *y*₀ + 1) from the refinery's
  top-left cell, for both foundation sizes; on that cell it turns on the spot until its facing is
  exactly 16384; *unload* becomes current after the first processing of *enter* that sees it on the
  cell, facing 16384 and not moving, unless the separately recorded track-completion route
  of H14 queued it earlier; the processings of *enter* are spaced by values the standard
  delay of *enter* allows (S-0215, claim 6). The run without the dock-unload flag:
  it takes contact, receives no docking destination from the refinery, remains in
  *enter* for 3000 ticks with the same load and contact, and is never paid. Repeat from several approach sides (the harvester
  starting north, south, east and west of the refinery): the arrival facing varies, the final facing
  does not, and the harvester never reverses onto the cell.
- *Falsified if:* another docking cell, a cell that depends on the foundation size, a reversing
  approach, *unload* before the turn has ended through the processing route (the second route
  is distinguished by H14), or a final facing other than 16384; in the no-flag run,
  refusal of contact, a refinery-directed docking destination, departure from *enter*,
  lost contact, changed load or a payment without an independent order or event.
- *Engine-only off-setting controls:* with `quirks.non_unloading_refinery_accepts`
  off, the no-flag building is never chosen in normal or waiting choice. Add a farther,
  valid dock-unload refinery: the harvester chooses that one, including when it is busy
  and only the waiting choice can use it. Remove that alternative: it stays returning
  without refinery contact, as claim 9's no-choice case requires. Separately give an
  explicit enter order to the no-flag building and record refusal of refinery contact,
  not an unloading payment. Repeat the valid-refinery main run: its outcomes must
  be unchanged. A no-flag choice or accepted refinery contact, or changed valid-refinery
  docking, falsifies the alternative. Import warns about the no-flag dock-list type
  in either setting. These controls are conformance tests, not oracle measurements.
- *Conditional chrono-miner trace:* if an ordinary stock sequence establishes a chrono
  miner with non-teleporting locomotion while contacting the refinery, record every due
  docking-cell offer with the order-level destination, locomotor destination point,
  destination-refresh and point-suppression states, suspended predecessor and restoration
  permission, current and queued behaviours, and the predicates of S-0207 claim 27.
  Compare an unchanged offer without refresh with a changed or refreshed offer, keeping
  the other predicates qualified. The former must not run the dock test by itself. The
  miner's repeated dock test (S-0207 claim 28) runs it at every accepted processing at which
  it has a destination, so record each processing's dock-test outcome whether or not the
  refinery made an offer. For the latter,
  distinguish restoration from non-restorable fallback. On qualified restoration, expect
  the restored teleporting locomotion to receive this same offer's otherwise due movement
  point, using the old driving points for restoration eligibility. With no already-pending
  suppression or S-0207 claim 20 disabling state, expect its landing point fixed by this
  assignment and a jump on its next teleporting movement update, possibly in the same tick.
  A preliminary driving leg or a required second offer in that qualified case falsifies the
  routing rule. Restoration newly enabling either pending state, or ignoring suppression
  already pending, also falsifies it. In the usual fallback with a
  nonempty point otherwise due, the order-level cell destination is installed but its
  movement point is skipped in that assignment; suppression is consumed and newly enabled
  refresh remains. A subsequent eligible equal offer may test again. Contact acquisition
  alone, or changing cell occupancy without an eligible assignment, must not cause a test.
  A dock test caused by the rejected unchanged offer itself (as distinct from S-0207 claim
  28's repeated dock test at the same processing), a movement point installed by the triggering
  fallback assignment, or a once-per-entry limit falsifies these conditional predictions.
  If this state cannot be established with stock data, retain it as unmeasured; do not
  infer that it is unreachable or replace the missing run with altered locomotion data.
- *Runs:* 2 per approach side and foundation size, 3 without the flag; 1 per engine-only control.
  For the conditional trace, retain every qualifying offer and its next equal offer.

**H11. The queueing search and the step-off (claims 9, 13).**
- *Setup:* a human house with one refinery (default queueing shift) and two harvesters, so one
  holds the dock slot and the other is in *returning* more than 3 cells away and waits (claim 9).
- *Observable:* the cell the waiting harvester drives to, and for part d the cell an idle
  harvester steps off to and the tick of that step.
- *Expected:* part a: on open level ground, the cell (*x*₀, *y*₀ − 1). Part b: walls (not units)
  are placed on that cell, then on each further cell the harvester picks in turn, run by run; each
  pick is the qualifying cell nearest the refinery's top-left cell in S-0212's ring order, wherever
  the harvester came from, and a cell held by an infantry unit is not picked (the occupancy record
  must be empty). Part c: every cell the search could reach is blocked (walls in a large square
  around the refinery, leaving the harvester's own path): the harvester drives towards the refinery
  building itself. Part d: a harvester standing on its refinery's cell with no field on the map goes
  *idle*, and 105 ticks later moves to one of the cells around the refinery; over runs started at
  different ticks, the chosen cell varies with the tick as (*N* mod *n*) of S-0212 claim 25 predicts
  for the count *n* of preferred cells.
- *Falsified if:* part a gives another cell; part b picks a cell that is not the nearest qualifying
  one to the top-left cell, or an occupied cell; part c leaves the harvester standing; part d always
  picks the same cell across start ticks, or a cell the prediction rules out.
- *Part e (Q-0194, failed idle step-off):* no resources, an ordinary non-teleporting harvester
  standing on a refinery-flag building's cell, and an owned dock-type building still present.
  Block every cell through the largest search radius of S-0212 claim 25 with walls or
  stationary units. Use the recorder to check that no cell qualifies; the harvester must
  enter idle normally, without setting its state externally. Record its destination, current
  and queued behaviours, body facing, contact, tether, no-field record, next-processing tick
  and random-stream state immediately before and after the processing 105 ticks later.
  Expected: destination cell (0, 0), *guard* queued, *harvest* still current until the ordinary
  start rule runs, and the next-processing interval equal to the standard delay of *harvest*.
  Exactly its delay draw occurs; no search draw. Facing, contact and tether are unchanged by
  this processing; the no-field record remains set. No cell can be reached while blocked.
  A preserved or absent destination, a retry search, no queued *guard*, a different delay or
  an additional draw falsifies this. Repeat with one qualifying cell freed to distinguish
  the failure from an ordinary step-off. On our engine only, the quirk off expects no
  destination instead, with all the other outcomes unchanged.
- *Runs:* 1 per blocking step in b, 2 for each of a and c, 5 start ticks for d; 3 blocked and
  3 freed runs for e. Use game-speed index 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted) for e,
  with only its purpose-made map and any named override sections active.

**H12. Contact lost and orders while unloading (claim 11).**
- *Setup:* a human house with one refinery and two full harvesters, the second waiting at the
  queueing cell. Each carries two resource kinds, so a payment leaves an unpaid load.
  Named map data gives a slow dump rate to leave time for input between those payments.
- *Observable:* per tick, each harvester's behaviour, contact, load, position, and the house's
  balance.
- *Expected:* part a: the refinery is destroyed (test data: a weapon that kills it in one shot)
  after the first payment: the first harvester stays in *unload* with the rest of its load, does
  not move and is not paid, for 3000 ticks; a move order breaks contact at once, and at its next
  processing of *unload* the harvester leaves the unloading state and stops. Part a2: the
  refinery is sold after the first payment instead: the harvester is in *harvest* one tick after
  the sale's first processing, with the unpaid rest of its load; its contact with the refinery
  ends at the sale's second processing, 1 tick after the first; and it drives off, not full, to
  its remembered cell if it has one, otherwise to the sale's scatter cell (claim 3's seeking
  rows). In part a2 the recorder also logs, per tick, its order-level destination and the cell
  of its last load. Part a3: sold while a
  harvester is in *enter* on the docking cell, still turning: it is in *guard* soon after and does
  not dock. Part a4: the first harvester returns to dock as usual with the dock slot free, and the
  refinery is sold while it is in *enter*, in contact with the refinery and still driving to the
  docking cell (row 2 of claim 10 has offered it that cell); the recorder also logs, per tick,
  whether its destination names the refinery building or a cell, and which cell. Expected: its
  destination is still the docking cell in the sale's first processing, with no scatter
  destination. Conditional variant: select the harvester and click the refinery (an explicit
  enter order) in the tick before the sale. If a run shows the destination naming the refinery
  building, with contact held, at the sale, expect that destination gone in the sale's first
  processing, and the harvester then to receive the sale's scatter request (claim 11): its
  destination becomes the cell that the nearby-cell search gives around its vehicle scatter
  centre cell (S-0207 claim 16 and Terms: the cell of its heading point while it has one,
  otherwise its own cell), or it gets no destination when its body is turning or the search
  gives no cell. The recorder also logs, per tick, the cell of its heading point and its body
  facing.
  Runs in which the destination names neither at the sale are void. Part b (no sale): the first harvester is given a move order in the tick after its first
  payment: the second harvester takes contact with the refinery from the next of its own
  processings on (the slot was freed in the order's tick); the first harvester starts moving at its
  next processing of *unload* (1 tick later) and keeps its unpaid load; ordered back to harvest, it
  docks again and is paid for that load.
- *Falsified if:* in part a, the first harvester leaves *unload* without an order, or is paid; in
  part a2, it stays in *unload*, loses its load, keeps its contact after the sale's second
  processing or drives off towards a cell other than those two; in part a3, it unloads at the
  sold refinery; in part a4, a harvester with a cell destination gets a scatter destination or
  loses the cell, or, in a run of the variant that qualifies, a destination naming the refinery survives the sale's
  processing, or the harvester gets a scatter destination while its body is turning; in
  part b, the second harvester cannot take contact until the first has driven off, or the first
  harvester loses its unpaid load or is paid for it before it docks again.
- *Runs:* 2 for each part.

**H13. Free unit (claim 18).**
- *Setup:* a human house with a construction yard and credits, on open level ground; the
  refinery's free unit is the harvester. The recorder logs the tick of the refinery's grand opening,
  the tick, cell and body facing of the free unit when it appears, its first behaviour, and the
  balance per tick.
- *Expected:* part a: the harvester appears in the tick of the grand opening on the cell in step
  direction 4 (map south) of the refinery's reference-point cell, facing 49152, and is in *harvest*
  one tick later. Part b: that cell is blocked by a wall: the harvester appears facing 40960 on the
  cell without overlay nearest the refinery's top-left cell in S-0212's ring order. Part c: the cell of
  part a and every cell near the refinery except ore cells are blocked by walls: the harvester appears on the nearest
  ore cell, facing 40960. Part d: every cell is blocked (the refinery placed on a tiny enclosed
  area): no harvester appears and the balance rises by *P* of S-0408 claim 1 for the harvester's
  type in that tick. Part e: a refinery captured by an engineer, and a refinery the map pre-places:
  no harvester appears. For the pre-placed case use a refinery of the neutral house, with a clear
  exit area. Record the end of set-up and its first turn: expect the opening's slot effects
  (S-0108 claim 11) and readiness, with *guard* queued at the end of set-up and current from the
  first per-tick start (S-0215 claim 1), but no free unit at either observation. This distinguishes
  a suppressed grant from an absent opening. Part f: the test data gives the house's country a vehicle cost factor of
  0 and a building cost factor of 1, so the refinery's price at its production start is its
  adjusted cost *C* (S-0404 claim 2, with *P*ₑ = 0; *C* above 0): no harvester appears. With the
  vehicle cost factor at 1 the price is *C* + *P*ₑ and it appears.
- *Part g (retained start price):* use named map overrides to give a capturable cost-bonus
  building factors for which the refinery's price without that building is above its positive
  adjusted cost *C*, and its price with it is above 0 but at most *C* (S-0404 claim 2).
  Run 1: start refinery production without the bonus, then capture the bonus building before
  completion. Run 2: start with the bonus, then sell the bonus building before completion.
  Repeat both with the gain or loss after completion but before placement, and during build-up.
  Leave the free unit's exit area clear. Record the price at production start, the later current
  price, the bonus ownership change and the grand-opening tick. Expected: the free unit appears
  in every run 1 and in no run 2, despite the opposite prediction from the later current price.
  The default cancellation-refund setting is used; on our engine repeat with that setting
  changed and expect the same free-unit outcomes. No cancellation is ordered in these runs.
- *Falsified if:* another tick, cell or facing; a harvester in part d, e or the first run of f; no
  refund or another amount in part d; in part g, the later price rather than the start price
  determines whether the free unit appears.
- *Runs:* 1 for each part and run, including each timing variant of g.

**H14. Track-completion route to unloading (claim 10; Q-0197).**
- *Setup:* H10's human house and ordinary driving harvester, game-speed index 2 at 800 × 600,
  no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). A named test map/art variant gives the refinery a rectangular
  foundation 3 cells along map x and 1 along map y, retains its dock-unload flag and has no
  absorbing, repair or reload flags. The cell immediately map west of its docking cell is
  then immediately map south of a refinery foundation cell. Lengthen the *enter* standard
  delay in named test data to separate driving from behaviour processing. Use a neighbouring
  ground vehicle factory's apron-clearing request (S-0417) to move a tethered harvester from
  the docking cell onto that southern neighbouring cell before its next *enter* processing.
  Vary the clearing tick and blockers to obtain the needed scatter destination. No orders
  or recorder writes may manufacture a tether. Record whether this actually preserves
  contact and tether.
- *Procedure:* record straight approaches and a two-cell curved approach returning to a cell
  with the contacted refinery map north. Compare the tick of crossing into each cell with
  the tick of reaching the track target. Repeat with the refinery switched off after the
  tether forms, and with scripted sale input near arrival. Vary timing until the sale run
  records *selling* at arrival while contact, tether and *enter* still hold. Runs without
  those preconditions are inconclusive for refusal, not evidence of acceptance.
- *Observable:* per movement event, position, track target, facing, destination, current and
  queued behaviours, contact, tether, refinery behaviour and power switch, and stream state.
  Also record cell-centre crushing if a crushable overlay can be placed in the arrival cell,
  and whether a next track begins in that same tick.
- *Expected:* the report queues *unload* at track completion, never merely at the boundary or
  the intermediate cell of a two-cell curve. It can queue with facing other than 16384 and
  while movement continues, and queues while the refinery is switched off. No arrival draw
  or new processing delay occurs. The queued result precedes cell-centre crushing and any
  next track. A refusal from a refinery in *selling* sends the forced, no-threat-point,
  not-move-anywhere vehicle request: zero draws and no behaviour queued by that request,
  contact and tether retained by the refusal rule. A still-present destination refuses the
  scatter; with no destination and the other vehicle gates passed, the destination is the
  nearby-cell result around the arrival cell. Observe before independent sale or later
  arrival effects release contact or alter behaviour.
- *Falsified if:* a report at a boundary or intermediate cell, an arrival draw, a facing or
  stopped requirement, refusal solely from the power switch, changed contact or behaviour
  from the refusal itself, or a scatter replacing an existing destination. A run that never
  forms the required tethered approach cannot decide the route; record it as inconclusive.
- *Runs:* 3 qualifying runs per approach and power-switch variant; 3 qualifying sale/refusal
  runs, separating destination-present and destination-absent cases when reachable. Use a
  read-only recorder and scripted input; no harness modification is authorised here.

**H15. Initial hidden slaves, ownership, credits and built counts (claims 19 to 21).**
- *Setup:* speed index 2, one human house and the remote bystander of the common setup. Named
  map override sections give a stock slave miner 3 slaves and give the slaves scoring and
  ownership-count flags suitable for observing both counts. Provide a factory and an isolated
  clear test area. Keep the miner stopped without any working-group transition during the
  initial observation. There is no engine add-on and there are no mods (the named test files of a scenario and the purpose-made test maps excepted).
- *Procedure:* produce a miner and record from production start, before release. Repeat with a
  map-created miner, a starting miner, and a grand-opening free unit whose type is the miner.
  Separately create a deployed building of the slave-miner type. Record creation counts,
  owners, inside/on-map state, last-rest tick, credits, per-type owned and ever-built counts,
  built total, creation draws and creation numbers. Controls use counts 0 and 1 and a miner
  without a slave type. For combined spawn-slot and slave creation, use S-0611 scenario MU12's
  creation-set variants.
- *Expected:* 3 hidden ready infantry exist at the master's creation, owned by its house,
  with no occupied infantry cells and no additional credit debit. Last-rest initially reads
  0 even for a miner created later in the game. Each slave takes one creation draw and then
  one creation number, after the master's and any initial spawn-slot objects', in place order
  (S-0611 claim 14 and F5). Their creations increment eligible owned counts but never the infantry ever-built count or built
  total, before or after release of the master. Counts 0 and 1 produce 0 and 1 initial slaves.
  A deploy/pack-up pair retains the existing working slaves under S-0415 claims 6 and 10.
- *Falsified if:* initial creation waits for release or wake-up, uses another owner, places the
  initial slaves on cells, deducts a slave price, records creation tick as initial last-rest,
  raises an infantry ever-built count, or allocates an initial slave's draw or number before
  the master's or before an initial spawn-slot object's, or out of place order.
- *Runs and tool:* 3 per creation source and 1 per count control, using scripted input and a
  read-only recorder. No harness modification is authorised.

**H16. Group emergence, timer boundaries and deployment (claims 21 and 22).**
- *Setup:* speed index 2, common bystander kept away, a human slave miner, a clear resource
  field and named map overrides giving 3 slaves, reload delay 13 ticks and regeneration delay
  17 ticks. Test both a vehicle and a deployed master. Arrange the emergence cell once free,
  once with room for only one slave, and once full of stationary friendly infantry.
- *Procedure:* issue harvest on the field; log the master's destination, behaviour, deploy
  attempts, group state and update ticks, slave creation/emergence ticks and occupied cells.
  Repeat with its deployment footprint obstructed just before arrival, then cleared. Damage
  a delivering slave and kill an emerged slave; log timer start and effect ticks. In another
  run empty the local field while leaving a distant field within the long scan.
- *Expected:* a harvest order recalls the slaves and directs the miner through travelling,
  rather than loading a bale itself or requiring an ordinary dock-type refinery. With a
  clear site it attempts deployment at a group update after its destination clears. A failed
  attempt postpones the next group update by 30 ticks; a second failure resumes field seeking.
  Regular updates are 10 ticks apart, first due 10 ticks after group creation. A ready slave
  emerges only at an update, in the offered cell of claim 21; blocked slaves remain inside.
  With room for one, the ready slave in the latest initial place emerges first. Reload and regeneration
  effects occur at the first later group update at which their timers have expired, not
  necessarily exactly 13 or 17 ticks after their start. A found short-scan field prevents
  automatic relocation. Without one, the master can seek a long-scan field and request
  packing-up, even for the human owner; log the chosen sites and distance comparison for
  claim 24's sites and F7's strict comparison.
- *Falsified if:* emergence itself creates another object, an emerged slave occupies a
  neighbouring rather than offered cell, the first ready slave wins the single-place test,
  timer effects occur between group updates, deployment retry ignores the 30-tick wait, or
  relocation occurs despite a short-scan field. Site choice is tested by H18 and relocation
  failures by H19.
- *Runs and tool:* 3 per placement/obstruction variant and 3 timer runs, scripted input with
  a read-only recorder. Treat interruptions of the master's turns as separate controls.

**H17. Capture without slave-owner propagation (claim 23).**
- *Setup:* speed index 2, common remote bystander, a deployed enemy stock slave miner
  and a capturable building in an isolated area. Give the human house an engineer and
  a stock mind-controlling unit. Named map overrides may make the slave miner vehicle
  susceptible to mind control for the vehicle run; do not use an engine add-on.
- *Procedure:* capture a working deployed miner with some slaves outside and some hidden.
  Record each existing slave's owner and master before capture, immediately afterwards,
  after hidden emergence and after reload. Kill an existing slave and wait for its
  replacement. Repeat on a controlled vehicle and on release from control. Record the
  house whose credits change on delivery. Include a deploy/pack-up control.
- *Expected:* existing slaves keep their individual owner and the surviving object as
  master; replacement slaves use the master's owner at replacement creation. Delivery
  credits the master's current house. Conversion changes the master object, not an
  existing slave's owner.
- *Falsified if:* master capture, control or release propagates its house to an existing
  slave, including on later emergence or reload, or delivery pays the slave's old house.
- *Runs and tool:* 3 per variant, scripted input and read-only tick-labelled ownership,
  master and credits recorder. If the vehicle control cannot be arranged with named test
  data, report that variant as unobserved rather than substituting building capture.

**H18. Emergence draws and deployment-site ties (claims 21 and 24).**
- *Setup:* speed index 2, common remote bystander, flat isolated test area, human slave
  miner and named map overrides for 1 × 1, 2 × 2, 3 × 2, 2 × 3 and 3 × 3 conversion
  foundations. Prepare symmetric empty sites around a field or directed destination.
  Separately prepare the emergence cell with one, several or no free side positions.
- *Procedure:* order harvest, record selected site, vehicle destination and placement.
  Repeat with a resource overlay, ordinary overlay, terrain object, slope, bridge,
  infantry reservation, building or non-open passability cell in the site's rectangle.
  Compare sites tied in distance, including different rings where the visibility
  preference matters. Log emergence offers and scenario-stream draws separately from
  ordinary behaviour-delay and object-creation draws.
- *Expected:* the unshifted candidate sequence and distance ties follow S-0212 claim 25,
  but only footprints passing claim 24 qualify. Each axis shifts independently only
  when its dimension exceeds 2. No site-search draw occurs. Each centre-class emergence
  offer not initially refused takes one ranged draw over 0 to 3, including a no-room
  offer. S-0208 table O predicts its selected free side position, in reverse slave order.
- *Falsified if:* a forbidden footprint is selected, tie order differs after visibility
  preference, either shift follows the other dimension, emergence selects centre or top,
  or draws differ from the offer rule.
- *Runs and tool:* 3 per footprint/obstruction variant and 100 emergence offers, scripted
  input and read-only position, occupancy and random-state recorder, at 800 × 600.

**H19. Relocation failures and pack-up destination (claim 25).**
- *Setup:* speed index 2, common remote bystander, a human deployed stock slave miner
  outside construction and selling. Named map overrides suppress resource growth and
  set known short scan, long scan and correction fields. Provide three separate layouts:
  no field in either range, a long-scan field but no qualifying deployment footprint,
  and a long-scan field with a clear site. Keep the master's turns uninterrupted.
- *Procedure:* exhaust the short-scan area and record every group update, field/site
  search outcome, packing-up request, conversion tick, remembered cell, vehicle
  destination and current/queued behaviour through its first field-seeking update.
  For the clear-site control vary the correction across the F7 equality boundary.
- *Expected:* the clear-site control packs up only for strict F7 improvement and records
  the master's old top-left cell, not the tested candidate site. After conversion the
  first vehicle group update enters field seeking without also searching. Its immediate
  destination is the old top-left cell and *move* is queued. For each failure layout,
  substitute (0, 0) for the missing result and predict packing from F7. With no field
  and no site, use corrections just below, equal to and above the distance to (0, 0):
  only the below case packs. With a field but no site, put the field once nearer (0, 0)
  and once nearer the master and compute both strict comparisons. When the test fails,
  emergence offers continue and field checks repeat every 10 ticks. When packing is
  requested, no emergence occurs in that update. Use a foundation larger than 2 × 2
  for the immediate movement control: return cell and old top-left cell differ.
- *Falsified if:* a failure automatically means stay or packing regardless of F7,
  equality packs, checks cease after failed improvement, emergence stops solely because
  no field/site was found, the first vehicle update also searches, or the immediate
  vehicle destination is the distant field, prospective site or absent. On our engine
  only, the new quirk off expects both failure layouts to stay deployed.
- *Runs and tool:* 5 per layout and 3 per correction boundary, scripted input and a
  read-only tick-labelled recorder. This scenario proposes observation only, not a
  change to the harness or the original.

**H20. Friendly harvester on the docking cell (claims 3a and 10a; Q-0371, Q-1870).**
- *Setup:* tiny flat skirmish map, stock refinery and two ordinary harvesters of one
  human house, reachable ore field, speed index 2, remote bystander; no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Keep the test refinery powered on. Use the normal measurement setup.
- *Procedure:* while B unloads, move A to the docking cell and stop it after B leaves.
  Order B to enter the refinery again from at least 10 cells away. In the first run, give A
  a harvest order once B has contact and while B's distance by S-0212 F6a still exceeds the
  close-enough distance, and record until A leaves the docking cell or 2000 ticks pass. In
  the second run give A no order and record for 2000 ticks. Repeat each run three times, and
  repeat both with the field reachable only past the refinery and with a second available
  refinery. A computer-owner run records the original's
  own orders separately rather than assuming human input reaches that owner.
- *Observable:* both units' cells, current and queued behaviours, destinations,
  contact and tether, dock occupancy, and any movement or scatter request, tick by tick.
- *Expected:* in the first run A's harvest seeking selects the reachable ore destination
  from the docking cell despite B's reservation. Any later route failure is logged
  separately from the harvest destination assignment. In the second run merely occupying
  the cell does not give A contact or unloading. B's docking exchanges keep directing B to
  that cell without a docking timeout or alternative-refinery choice, and the exchanges
  themselves send A no request; B's own driving can (parts e and f).
- *Falsified if:* the docking exchange evicts A other than through B's driving
  (claim 10a), expires B's reservation because time passed, queues B elsewhere or chooses another refinery, or harvest seeking
  refuses A's field destination solely because it starts on the docking cell.
- *Route variants:* S-0212 claim 27a/P27 supplies docking-cell entry, persistence and
  departure tests; claim 23 supplies failure classification and retry delay. Record
  S-0215 claim 11a's distinction between failures retaining seeking and failures
  queueing a replacement behaviour, including the later start and processing ticks.
  A queued replacement need not wait for the old processing or route retry timer.
  General scatter remains owned by the movement specs, not the docking exchange.
- *Tool:* scripted input with video and a read-only tick/state recorder. Reachability
  and route-failure overlaps must actually be observed; this scenario does not
  assume that every movement failure leaves *harvest* current.
- *Part e (Q-1870, low base):* the flat map above. A rests on the docking cell in *guard*
  with no destination, after a stop order or, where one can be obtained, after a failed
  departure that gave up; record A for 2000 ticks before B arrives. Then B arrives loaded
  from at least 10 cells away and takes contact. Record B's destination at every processing
  of *enter*, every scatter request with its sender, options and receivers, A's acceptance
  and new destination, B's contact and reservation, and the tick B docks. Expected: A stays on
  the cell in *guard* until B's request, with no route retry of its own. While B's distance by
  S-0212 F6a exceeds the close-enough distance, B's destination is a free cell next to the
  docking cell; once B is within it, B's route leads onto the cell, B sends A a forced,
  move-anywhere request without a threat point, and A accepts it and drives off to a nearby
  cell, staying in *guard* there. B keeps its contact and reservation throughout and docks
  after A leaves. Falsified if B, within the close-enough distance, keeps moving round the cell
  without sending the request, A, still and without a destination, refuses it, or B loses its
  contact or reservation, is sent to a queueing cell or another refinery, or never docks once
  the cell is clear.
- *Part f (Q-1870, raised base):* as part e, with the refinery on a plateau whose world
  height exceeds the close-enough distance (its height levels named in the run's record) and
  free cells next to the docking cell. Expected: at every processing of *enter* B's
  destination is replaced by a free cell next to the docking cell; B moves between such
  cells, keeps its contact, sends A no request and does not dock for 3000 ticks. Then order
  A off the cell: B docks at a later processing of *enter*. Falsified if B docks or A is
  scattered before that order, except after a recorded processing of *enter* at which no
  substitute was adopted (claim 10a's second row then applies). Repeat on a plateau whose world height is well below the
  close-enough distance: part e's result is expected.
- *Part g (chrono miner):* parts e and f with B a chrono miner whose field is 15 to 40 cells from
  the docking cell, with the test rules file of S-0207 scenario D34 part e. Record B's movement
  kind, its tracks, its destination and its movement report at every processing of *enter*.
  Expected:
  once B is offered the occupied cell, B starts no new track after the track it is on, sends no
  request, keeps its contact and reservation, and does not dock while A stays (on the low base,
  unless a recorded path computation adopted no substitute, after which part e's driving and
  request follow). Then order A off the cell: B jumps to the docking cell at the first processing
  of *enter* at which it stands still. Falsified if B drives to a free cell next to the docking
  cell after a recorded substitution, or jumps anywhere but the docking cell.

**H21. Slave-place ordering, two clocks, facing and scan tests (claims 21, 26).**
- *Setup:* common skirmish setup, speed 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Named map
  overrides give a stock miner three slaves, harvest interval 37 ticks and regeneration
  and reload delays 13 ticks. Use a rectangular deployed foundation whose reference,
  top-left and unloading cells differ. Mark slaves by identity and initial creation order.
- *Procedure:* kill two slaves before the same group update; record all replacements,
  ready states, emergence offers, facings, payments and draws at each update. Arrange
  an earlier-place slave to deliver while a later-place replacement is due. Repeat with
  reload delay 0. Observe seeking, loading and return over 1000 ticks. Rotate the vehicle
  master before emergence. Repeat with slave capacities 0, −1 and −7 as named test
  data, starting each slave empty. Disable resource growth for these capacity runs,
  record resource stages as well as loads, and later exhaust the cell with an
  ordinary harvester to test subsequent searching and empty return.
- *Expected:* every expired place is attempted in increasing place order; replacement
  creation draws are in that order. An earlier-place delivery precedes a later-place
  replacement. Ready replacements and completed reloads may emerge in the same update,
  in reverse place order, facing map north. Newly accepted deliveries cannot finish even
  zero reload until the next update. Loading uses 37-tick intervals in slave turns,
  search, fullness and delivery effects use group updates in master turns. Capacity 0
  never loads and is not full; its harvest queues guard with a 1-tick delay and the
  group subsequently re-enters searching rather than sending it back for fullness.
  Negative capacity never loads or changes the resource stage, remains harvesting
  with 37-tick processings and is not full. It does not return while that cell remains
  resource land. After externally exhausting the field, ordinary searching and
  empty return apply. Repeat negative runs with the quirk off on our engine: they
  use the zero-capacity guard and rescan treatment.
- *Scan variants:* surround slaves with resource cells occupied once by one friendly
  infantry, once by a full infantry record, once by a vehicle, and once disconnected
  by impassable terrain. Predict candidate choice from clear infantry entry and region
  connection, with the own-cell tiberium shortcut recorded separately. For a deployed
  master, use a short range that distinguishes the three scan centres; put the only
  resource inside it relative to the reference cell, then block or disconnect that cell.
  Separately place it in a legal cell outside the usable area but within the scan rings.
  Expected: deployed scans select it irrespective of occupancy, connectivity or usable
  area; slave scans retain their candidate tests. Neither field scan takes a draw.
- *Falsified if:* one expired place per update, fresh creation order replaces place order,
  all replacements precede all deliveries, replacement emergence waits another update,
  facing follows the master, non-positive capacity loads or consumes resources,
  either capacity treatment returns solely for fullness, a clock differs, or a scan differs
  from its specified centre and qualification.
- *Runs/tool:* 3 per timing and capacity variant, 3 per scan layout; scripted input and
  read-only tick/event, facing, ownership, occupancy and random-state recorder. A run
  lacking the arranged overlap is inconclusive for ordering.

**H22. Busy slave masters (claim 27).**
- *Setup:* common setup, speed 2, stock slave miner near a field, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
  Named map overrides lengthen build-up/build-down enough to contain several group
  updates, and set known reload and regeneration delays. Observe all its slaves.
- *Procedure:* deploy with slaves outside and returning, create a building master with
  initial hidden slaves, and sell or manually pack a working master. Record current
  behaviour, construction state, group state, group updates, payments and emergence.
  Keep a short-scan field in one run; exhaust it in another.
- *Expected:* returning slaves still enter and pay, and expired reload or loss timers
  still have their effects during busy states. Newly created at-rest groups and working
  groups in construction do not emerge or check the field. Record the transition-only
  update into checking the field and the later first check. A group already checking
  the field continues checks and emergence even while selling, unless another rule puts
  it into packing; packing buildings release no ready slaves. Compare claim 28's
  directed packing case separately instead of treating every sale as that case.
- *Falsified if:* a blanket busy-state pause suppresses delivery or timers, the transition
  update itself checks or emerges, or emergence disregards the recorded group state.
- *Runs/tool:* 3 per reachable state, scripted input and read-only recorder. Report an
  unreachable checking-during-selling variant as inconclusive, not as a universal gate.

**H23. Harvest processing and manual packing (claims 22, 28).**
- *Setup:* common setup, speed 2, two isolated fields and a human slave miner. Named map
  data lengthens the harvest behaviour's standard delay; no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Observe its destinations, current/queued behaviours, group state and slaves.
- *Procedure:* release a miner whose initial harvest has no destination, then give two
  successive harvest orders on different fields during travel. Record every processing,
  search and random draw. Manually pack a deployed miner by a move click once to clear
  ground and once to a resource cell other than its top-left cell. Repeat the resource
  click with no qualifying site. Record site selection at first selling processing,
  remembered destination at conversion and the first two vehicle group updates.
- *Expected:* destination-free harvest queues guard, records rest and uses the standard
  harvest delay with its one ranged draw. Each processing with a destination searches
  near the destination then current; ordinary travelling updates do not search again.
  Deployment waits for a due group update seeing no destination, not the arrival event.
  Clear-ground manual packing does not force packing group state; a group checking the
  field becomes at rest on its vehicle update. Resource-directed packing with a found
  site records that site, recalls and enters packing, then its first vehicle update
  enters field seeking. Failed site search preserves the old destination and group state.
- *Falsified if:* the no-destination interval is always 1 tick, a newly current harvest
  retains an earlier order's site without searching, every travelling update searches,
  deployment is immediate on arrival, or every manual pack-up unconditionally seeks.
- *Runs/tool:* 3 per order/packing variant, scripted input and read-only tick and stream
  recorder. Factory release supplies destination-free harvest without an injected order.

**H24. Temporary pack-up slaves and draw order (claim 19a).**
- *Setup:* a tiny stock-data skirmish map, game speed 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
  with a human-owned deployed slave miner and one bystander computer player
  kept outside the observed area. Use a control with a free return cell and
  a second run with that cell blocked. Include old slaves both inside and outside.
- *Procedure:* pack up once per run; repeat three times per case. Use a read-only
  state and random-stream recorder across the conversion tick, recording new
  infantry creation, their on-map status, old slave ownership and losses.
  Compare end-screen losses with a control that performs no pack-up.
- *Expected:* the return vehicle creates its type's initial count of hidden slaves
  before placement in both cases, each with its creation draw. On success these
  temporary slaves are destroyed before old-group transfer; on refusal they remain
  with the hidden vehicle, and their creations precede old-group release. No old
  slave is recreated on successful transfer. The discarded temporary slaves count
  as losses. Missing temporary creations, absent losses, undone draws or old-group
  release preceding those creations falsifies the claim.
  Tag notifications remain `unmeasured`: this setup establishes no tag attachment
  to a generated slave. S-0011 claim 19 and conditional scenario T25 B propose
  the notification test if a data-only fixture is found; disappearance and losses
  alone neither confirm nor falsify tag evaluation.
  The quirk-off result is an engine conformance test, not an oracle measurement.

**H25. Deployment-state boundaries and shared guard spot (claims 27, 28).**
- *Setup:* a tiny purpose-made stock-data skirmish map at speed 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), human-owned slave miners and one distant bystander computer player.
  Provide one distant field with a valid deployment site and a separate exhausted
  field that induces automatic relocation. Observe only these miners and slaves.
- *Procedure:* with scripted input and a read-only state recorder, manually deploy
  while field seeking and while retrying deployment. Record three runs of each,
  including group states, last-rest ticks, destinations, behaviours and emergence
  at each due group update. Repeat the seeking case with no acceptable field/site.
  Separately request automatic relocation after first giving a different manual
  guard spot, then sell during build-down. In a control give a new move destination
  after automatic relocation but before conversion, when the order is accepted.
- *Expected:* seeking with a site enters travelling, then at rest at the next
  update; seeking failure and a retrying building enter at rest directly and
  record that update's tick. Neither transition emits ready slaves. Ordinary later
  at-rest/checking rules resume as claim 27 states. Automatic relocation replaces
  the earlier manual guard spot with the old top-left cell; a later accepted
  destination replaces that cell. A sell order alone retains it, and successful
  packing gives the vehicle that destination. Immediate checking from seeking or
  retrying, emergence in those transitions, independent destination memories or
  automatic memory overriding a later accepted order falsifies the claims.
- *Site-offer variants (Q-0500):* repeat the field-seeking success case three
  times with a previously remembered, distinct cell and three times with no
  guard spot. Record the offered shifted site, guard spot, current and queued
  behaviours, processing ticks, group state and random-stream state through
  the next group update, then sell and observe the returned vehicle's destination.
  Use the same speed, houses, recorder and scripted input as above. Add named
  test-map type overrides qualifying the master through exactly one of these
  properties at a time: cloning, construction yard, repair depot, infantry factory
  and vehicle factory. Use controls with none of those properties, including an
  aircraft-factory-only control. Keep the building alive through the test.
  The qualifying non-selling variant replaces the guard spot with the offered
  site and later packs towards it. The control retains its earlier spot or
  absence, yet requests *move*. Repeat during current *selling*, timing the
  search before conversion: both the earlier guard spot and queued behaviour
  remain unchanged, but the group still enters travelling. In a ready,
  non-constructing case, *move* may become current before the next group
  update, without movement or packing. Each processing while it is current
  does nothing and has a 450-tick delay with no draw attributable to that
  processing; separate group-update draws are recorded separately. Returning
  to rest does not cancel *move*. Replacement for a nonqualifying or selling
  building, any separate site memory inherited on packing, packing caused by
  *move* alone, a processing interval other than 450 ticks, a processing draw, or
  move cancellation by the group reset falsifies claim 27a.
  Repeat the qualifying offer during construction, with low power and with the
  power switch off, recording the offer separately from whether *move* can start.
  Repeat the construction-yard variant with lobby redeploy off: destination
  acceptance is unchanged, although conversion still has its separate legality
  checks. A rejected offer solely due to any of these conditions falsifies the
  corresponding no-exclusion statement. Record the bystander's own qualifying
  master separately for an owner control; a refusal solely due to owner falsifies
  that statement. Only named test objects are exempt from the bystander void rule.
  Runs that never reach the required group-state/behaviour overlap do not test
  that variant; its backing remains `unmeasured`, with the unreachable overlap
  recorded as the reason.

**H26. Vehicle harvesting display (claim 5a, F8; Q-0550, Q-0580, Q-0590).**
- *Setup:* a small, level purpose-made skirmish map with a human-owned ordinary miner,
  a chrono miner and a slave miner, their appropriate unloading buildings and separate
  ore and gem fields. Keep the observed vehicles free of attackers. Use stock data,
  speed index 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Run each vehicle separately so its
  images are not covered by other objects. Use a read-only recorder and tick-labelled
  frame capture; log position, body facing, behaviour, harvest phase, step counter,
  walk-animation counter, load and scenario random-stream state alongside each drawn frame.
- *Procedure:* record three full cycles for each ordinary and chrono miner on each
  resource kind, from approach through full load and departure. Record the slave miner
  stationary and deployed while its slaves work. In separate runs, stop an ordinary
  miner during loading, let a small cell exhaust with a second cell nearby, and exhaust
  an isolated field with only a partial load. Include a body turn during an approach.
- *Observable:* the extra sprite's first and last visible ticks, its location and frame
  progression, the body/turret images, and their relation to bales and phase changes.
- *Expected:* the ordinary and chrono miners use the same extra harvesting sprite on
  both resources; it begins on the loading-start processing once stationary, before the
  first bale, and repeats every 15 ticks with a constant walk-animation counter.
  It continues after becoming full until the failed step ends loading. Moving or
  turning hides it, but on stopping within loading its image frame follows F8, not
  a fresh frame 0. Returning with a partial load ends it. No body or turret cycle
  is started by a bale. Slave work alone gives the master no vehicle harvesting sprite.
  A stop that actually ends *harvest* removes it by the next vehicle turn.
- *Falsifiers:* no extra image on a visible stationary loading miner; a sprite specific
  to gems or to the chrono miner; a first display delayed to the first bale; disappearance
  at the filling bale rather than the failed step, an image cycle reset at every bale,
  a loading-selected body section or turret movement; or a master display caused only
  by working slaves.
- *Exact-frame and anchor control:* use a named test copy of the fixed harvesting sprite
  with 120 distinguishable, small numbered image frames and a marked canvas centre.
  Change no playback data. This is test data, removed after the run. Record eight
  stationary loading approaches with different 8-step body facings, and one run with
  a different load period. Compare the visible centre against F8 projected by S-0101
  claim 1, and every frame against 15(7 − *q*) + ((*t* + *ν*) mod 15).
  The changed load period must change the bale times but not this frame rule. A
  different facing group, repeat length, phase or anchor falsifies F8. The named sprite
  also separates absence from ordinary body occlusion. Record integer lepton positions
  and compare all eight centres against table and exact trigonometry predictions,
  as well as against the reversed sine/cosine assignment. Where one-lepton differences
  project to the same pixel, repeat at other within-cell positions until the predicted
  screen centres differ. Agreement only with the exact or reversed form falsifies F8.
- *Brightness control:* on the same stock-data map include a lit field and an unlit
  field; capture the same harvesting frame and facing on each. Compare visible
  sprite colours with animation-palette reference images at the vehicle brightness
  predicted by S-0104 claim 13, not the anchor cell's scaled light. In a separate
  named map-data control change only the extra unit brightness, then restore it.
  The sprite must follow that addition like the vehicle. Unchanged full-brightness
  pixels, or agreement only with bare cell light when the addition is nonzero,
  falsify the claim. No tint or house remap should be added to the animation palette.
  Repeat the same visible frame with two different owner colours: the harvesting
  sprite's palette indices and colours must not change solely with the owner colour.
  A remapped harvesting pixel falsifies the no-remap rule. Repeat beside a tall
  building whose shadowed-cell light penalty covers the field, capturing sprite
  pixels not occluded by its body or the building. Expected brightness includes
  S-0104 claim 13's shadowed-cell penalty; agreement only with unpenalised brightness
  falsifies that case. These controls use the same recorder, game settings and three
  cycle runs as the main scenario.
- *Shadow-order control:* retain stock vehicle art and use a named copy of the
  harvesting sprite with a broad opaque, uniformly coloured patch extending beyond
  the body into its shadow footprint. Capture a stationary loading vehicle with a
  visible shadow at each facing. Compare pixels outside the body but inside the
  shadow footprint with the same patch outside the footprint. Pixels that pass the
  shadow's ordinary depth test must be darkened as in S-0102 claim 26. A patch
  covering those shadow pixels without darkening falsifies the before-shadow order.
  Restore the original sprite after the control; this changes test art only, not
  the shadow rules or the miner's gameplay.
- *Suppression controls:* in separate runs, give the loading miner a movement order,
  target it with an erasing weapon, and record a chrono miner's trips: its driving trips and
  its docking jump, the only one of them that is a teleport journey. The extra
  image is absent in the conditions of claim 5a and can resume when those conditions
  end without restarting its image cycle. Record whether loading is still eligible:
  ending the behaviour is not evidence for a movement-only gate. Presence while an
  erasing weapon is removing an otherwise eligible miner, or during a recorded
  teleport disappearance or reappearance state, falsifies those suppression rules.
  Account for behaviour-delay, creation and resource-event draws using their owning
  claims; any additional scenario draw attributed solely to drawing the harvesting
  sprite falsifies its no-draw rule.
- *Zero-step movement control (Q-0590):* use the exhausted-cell run
  with a second resource cell directly ahead, so the miner keeps loading eligibility
  while starting a straight track without a body turn. Record active track, body-turn
  state, speed fraction *ρ*, integer step *s* from S-0207 claim 2 and progress budget
  with each captured frame. Select ticks with *ρ* > 0 and *s* = 0. Expected: the
  sprite remains visible on those ticks, subject to the other gates. Once *s* > 0
  it is suppressed, even on a tick with no position change because the budget buys
  no track point. Absence at *s* = 0 with every other gate clear, or presence at
  *s* > 0 on an active track, falsifies the integer-step test. Use the marked sprite
  to distinguish suppression from body occlusion. If stock acceleration produces
  no qualifying tick, repeat as a separately labelled map-data control with the
  acceleration flag enabled and a smaller positive acceleration factor, leaving
  the harvest and display settings unchanged; restore stock data afterwards.
  This control must still record loading eligibility, not infer it from lack of
  movement. No writes to runtime state are proposed.
- *Engine-only control:* repeat the marked-centre tests with `quirks.table_direction`
  off and compare against F8's full-scale alternative. This tests our engine, not
  another original-game result.
- *Tool:* tick-labelled screen capture and the read-only recorder; scripted ordinary
  mouse/keyboard orders suffice. No harness change or oracle measurement is performed
  by this spec.

**H28. Animations and smoke at payment, and the harvester's image while unloading (claim 11, Terms; Q-3062, Q-3063).**
- *Setup:* a purpose-made two-player map with a small ore field; stock data otherwise, with the
  director and mods off, at a fixed seed. The human house starts with a construction yard, a war
  factory and enough power and credits, and builds one refinery-flag, dock-unload refinery by a
  place order at the start of the run. A structure that the map places for a human player's house
  is dropped (S-0009 claim 27), so the refinery is not pre-placed. The refinery's
  art gets three purpose-made animation types of known, distinct lengths and distinguishable
  images for its docking, payment and unloading animations; the payment and unloading types each
  have a descending random-rate pair whose converted bounds differ (*d*<sub>1</sub>′ ≠
  *d*<sub>2</sub>′, S-0109 claim 3), so each creation takes exactly one ranged draw. The
  payment animation is longer than the interval from the payment to the change to *finishing*,
  which the run records. The test data gives the refinery's type a refinery smoke type and one
  set smoke point, so that each smoke event makes one system (S-0616 claim 1). A test harvester type names a purpose-made vehicle
  type with a distinguishable image as its deployed image type; a second run uses a test harvester
  type that names none.
- *Procedure:* (a) let a harvester with a load of one bale dock and unload. (b) Give a
  newly produced harvester, its load empty, an explicit enter order to the refinery. (c) Run (a)
  with the harvester type that names a deployed image type, capturing every frame from the docking
  row to its departure. (d) Give the refinery only normal names for the three animations, damage it to at or
  below the yellow threshold, and repeat (a); then give it only damaged names, at full health, and
  repeat (a).
- *Observable:* per tick, the objects that enter play (particle systems and animations) in the
  order the read-only recorder lists them, and the scenario random stream's state before and after
  the harvester's processing, from which the ranged draws taken and their order follow (the raw
  values each ranged draw consumes follow S-0202 claim 4); the animation shown on the refinery per captured frame; the
  harvester's drawn body image per frame; the tick of the change to *finishing* and of the
  contact's end.
- *Expected:* (a) in the payment tick, the smoke system enters play before the payment
  animation, which takes its one draw, and the credits arrive in the same tick; in the empty-load
  tick, the smoke system and then the unloading animation enter play, with the unloading
  animation's one draw, no payment animation is started again, and the payment animation is
  gone at the end of that tick. (b) The empty docking shows the docking animation; in the empty-load
  tick the smoke system, then a payment animation with its draw, then the unloading animation
  with its draw enter play, and the payment animation is not shown on the refinery in any
  captured frame. (c)
  The body is drawn with the deployed image type's image from the start row's tick until the
  harvester leaves the unloading state, and with its own image before and after; with the type
  that names none, its own
  image throughout. (d) With only normal names, the damaged refinery plays the normal animations,
  and the harvester waits for the normal unloading animation; with only damaged names at full
  health, no animation plays and the harvester changes from *finishing* to its next row 1 tick
  later, as in H1 variant e run (iii).
- *Falsified by:* in parts a and c, and in part d's run with only normal names, credits arriving
  in a tick other than that of the payment animation's start; a payment animation entering play
  before the smoke system; a payment animation started again in
  part a's empty-load tick; in part b, the payment animation's draw after the unloading
  animation's, or the payment animation drawn; a body image other than the deployed image type's at any
  frame of the unloading state, or that image outside it; no animation on the damaged refinery
  with only normal names.
- *Tool:* tick-labelled screen capture and the read-only recorder; ordinary orders only. This is
  a proposal for the oracle; no harness change is assumed beyond what the recorder already lists.

## Open questions

- (Claim 17, Q-1759.) Q-1759 questions 1 to 4 are settled in S-0207 claim 28 (its answer
  table); this spec carries their consequences in claim 17 and the Terms. Still unsettled, all
  `unmeasured`:
  - whether a chrono miner reaches a seeking processing with teleporting locomotion active in
    each of claim 17's cases: (a) after S-0607 claim 4 ended its temporary driving; (b) after the
    restoration that ends its seizure; (d) after a placement that made no destination assignment
    (no way of creating one is established); and (e) with warp-pending immobilization. Of these,
    only (e) meets the drop with teleporting active at the test and after it, and only (e), or (a)
    with a held destination, meets the drop at all. Case (c), after a fall-back before docking, is
    measurable through S-0207 scenario D34 parts g, h and i, and case (f), a miner unloading at a
    sale, through D34 part h. Neither is among these.
  - what a chrono miner in case (e) does with the destination that the seeking drop's ring
    search gives its teleporting locomotion (an adopted remembered cell is removed at once,
    claim 17): whether it is dropped, kept for after the superweapon warp, or jumped to.
  - whether the chrono miner's *guard* rule, or claim 13's *guard* rule under a
    computer-controlled owner, can queue *harvest* while it is seized, and that a seized chrono
    miner's seeking processing neither ends its grip nor makes it travel (claim 17).
  - a chrono miner seized with a stand-in other than the jumpjet one, named by the seizing
    warhead (S-0607 claims 22 and 23), is not covered: which of claim 17's rules it meets is
    open.
  - the answers to Q-1759 questions 1 to 4 rest on S-0207 claim 28 and its scenario D34, which
    has not been run.
- Q-1130 question 3 is answered by the seeking drop in claim 17 (every seeking processing past
  the *full* test; order-level destination cleared, pending jump cancelled, landing point and
  reservation retained). Open: whether a harvest order's destination is dropped before the
  miner's first jump (claim 17; H8 part h). Questions 1 and 2 are S-0207's.
- Q-0550 is answered by claim 5a, F8 and H26. S-0111 claim 14 already covered load
  pips. The fixed harvesting sprite is not an animation instance, so its frame cycle
  does not require a new S-0109 playback claim.
- Q-0580 questions 1, 2 and 4 are answered by claim 5a, F8 (with its precision
  rule and its coordinate contributions) and H26. S-0205 claim 6 requires table
  trigonometry before the request. Question 3 is already answered by S-0102 claim 13 and scenario
  D43. Question 5 is answered by claim 5a's integer-step test and H26's zero-step
  movement control; S-0207 claim 2 already defines the step and its rounding.
- Cross-spec coverage: S-0215 claim 4 already covers the reachable 450-tick no-op building *move* described in claim 27a and cites H25. S-0415 claim 9 already recognizes that claim 27a's site offer can supply a guard spot, including with lobby redeploy off; pack-up legality remains S-0213 claim 12's, unchanged here.
- Q-0500 is answered by claim 27a. Claim 28 and S-0415 claims 9 and 10
  already identify the shared guard spot and its packing consequences.
  S-0215 claims 7 and 11 already fix queue refusal and building starts.
  This spec states the site-offer gates and destination retention.
- Q-0441's creation and draw ordering is claim 19a; S-0415 claim 11 owns
  refused-vehicle retention and its quirk. Q-0442's deployed state boundaries and
  shared remembered destination are claims 27 and 28. Q-0440 belongs to S-0408
  claim 10a, applying the common slave teardown of S-0312 claims 30 and 31.
  This spec does not duplicate that sale-owned claim.
- Q-0412 is answered for this spec's scope: claims 21 and 26 already cover replacement
  count and order, emergence facing, the two clocks, slave field qualification and
  deployed scan centre and qualification (questions 2, 3 and 5–7). Claim 26 now
  completes question 4's non-positive capacities, correcting the earlier zero-capacity
  fullness statement. Question 1's master-turn integration is S-0203 claim 12.
  Claims 21 and 26 own the within-group order.
- Q-0411 is answered by claim 25 and F7; Q-0413's busy-state part by claim 27, with
  removal and failed-conversion parts owned by S-0312, S-0408 and S-0415. Q-0414's
  harvesting, manual packing and arrival parts are claims 22 and 28; applied-order
  recall is S-0213 claim 8 and claim 5.
- Whether *return* is ever given to a harvester in a skirmish (claim 2); scenario H9 logs it.
- Q-0270 is listed as answered for this spec's scope: claims 19 to 24 cover initial creation,
  counting, emergence, ownership retention, directed harvesting, sites and successful relocation,
  claim 25 now supplies failure comparison inputs and the immediate vehicle destination,
  with H19 to check them. No failure is assumed to mean stay-in-place.
  Ordinary killing-hit integration is S-0312 claim 30; attacker-less removal of a
  remaining, untransferred group is S-0312 claim 31. True-sale slave fate is S-0408
  claim 10a, whose exact cleanup tick remains open. Earlier group-ending events during
  erasure or environmental sinking, and when other destruction paths reach removal,
  remain unresolved. Surviving-master capture and
  mind-control return are claim 23, pending H17. Deployment and packing-up
  preservation already belong to S-0415 claims 6 and 10.
- Q-0371 is answered here for harvest and docking rules by claims 3a and 10a.
  Claims 3, 9, 10 and 13 already supplied phase intervals, queueing and rest.
  Non-contact docking-cell entry, persistence and departure are S-0212 claim 27a/P27.
  Route-failure classification and retry delay are its claim 23, and later behaviour
  starts and processing are S-0215 claim 11a. General step-aside requests and their
  destinations remain with the movement specs. S-0417 claim 13's
  factory-apron clearing must not be assumed to apply to every refinery.
- The exact tick of the first bale (19 ticks) and the "+ 1" of the load period depend on the order of
  the counter increase and the processing within a turn (claim 5); scenario H1 settles both.
- Further refusals in a refinery's answer to a contact request for harvesters, and whether a
  refinery that is being erased refuses (claim 10; scenario H4 part h).
- What else the building does at each payment besides playing its payment animation (claim 11).
  No observable effect is known, and scenario H1 would show one as a difference in its balance or
  timing runs.
- Weeders and the harvester truce are out of scope (Edge cases).
- Whether H14 can reliably arrange every refusal precondition before the sale independently
  releases contact. The arrival event, acceptance and scatter rules are specified in claim 10.
  Reachability of the rare destination-absent refusal is not yet measured.
- What a refinery that is not a dock-unload refinery does with a harvester in contact (claim 10,
  Edge cases); confidence low.
- That a pre-placed building gets no free unit rests on S-0009 claim 40 (claim 18).

## Changes for other specs

- **S-0408 claim 6, the sale's contact break.** S-0408 claim 6's row "A later processing while
  any unit remains docked: the sale waits" holds back only the releases and the build-down. The
  sale's second processing breaks off every contact with the refinery, whether or not a unit is
  still tethered (claim 11). A contact still held after the first processing by a unit that is
  not tethered does not hold up the release. A tethered unit holds up only the releases and the
  build-down, not the end of its contact.
- **S-0215 claim 24, the vehicle table's backing and the player houses.** Its backing names
  S-0402 scenario H7 "variants b to e" for row 1. Add variants (d) and (g): (d) for a person's
  miner far from a field, (g) for a computer player's and for claim 13's rows not applying to a
  slave miner type with neither the harvester nor the weeder flag (claim 14). Its sentence
  "since no player house is computer-controlled" holds except for the test-only
  computer-controlled player house (glossary), which takes row 2's case and the owner
  alternative of claim 14, like the neutral and special houses.
- **S-0517 claim 12 and its Import mapping, the deployed image type's second use.** The field
  also gives the image a harvester's body is drawn with while it is in the unloading state (S-0402
  claim 11, *The harvester's image while unloading*), with the same rule that the type is used
  only for drawing. Claim 12's second item and its Import mapping row can point there. Level and
  backing: high for the body's image and when it is used, medium for the drawing properties that
  come from that type and for the overlays keeping the harvester's own type; `measurable` by
  S-0402 scenario H28 part c for the body's image, and `unmeasured` for a voxel body's frame and
  for the overlays, the selection box and health bar included, as S-0402 claim 11's backing says.

## Questions for the project lead

None. This spec rests on S-0207 claim 28. S-0207's Cross-spec changes list the hand-off
for scenarios that expect a chrono miner to jump after a restoration (S-0215 scenario B15 part
(e), S-0417 scenarios U6 and U15) and the S-0214 Terms restatement of the self-teleporting type
that this spec's Terms (*chrono miner*) rely on, and its Q-1759 entry on S-0215 claim 9 carries
the amendment that claim 17 relies on (the
guard reset is not applied when driving is installed). The note for S-0408 claim 6 is under
Changes for other specs. The nine quirk switches of this spec
(`quirks.harvester_failed_step_off_origin`,
`quirks.refinery_distance_wraps`,
`quirks.slave_kick_delay_wraps`, `quirks.zero_load_period_stalls` and
`quirks.unload_without_contact_stalls`, `quirks.slave_relocation_failed_search_origin`
and `quirks.slave_negative_capacity_stalls`, `quirks.zero_capacity_harvester_cycles`
and `quirks.non_unloading_refinery_accepts`, each default original) follow the project's quirk
convention.
