---
id: S-0402
title: Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners)
topic: economy
revision: 24
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0140, Q-0141, Q-0142, Q-0143, Q-0144, Q-0145, Q-0194, Q-0197, Q-0270, Q-0371, Q-0411, Q-0412, Q-0413, Q-0414, Q-0441, Q-0442, Q-0500, Q-0550, Q-0580, Q-0590]
supersedes: []
---

# S-0402: Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners)

## Summary

A harvester drives to a resource field, loads bales one at a time, carries its load to a refinery
of its owner, docks, and is paid for each kind of resource it carries; then it goes back for more.
This spec covers the harvester's capacity, the timing of a load step, how it chooses a field and a
refinery, when it waits or queues, docking and unloading, the credits a house gets and when, what
happens when fields are exhausted or no refinery is left, and the variants: the slave miner, whose
infantry slaves gather for it, and the chrono miner, which teleports; resource cells themselves
(stages, bales, growth) are S-0401's. This revision covers skirmish only; network
games and campaigns are out of scope.

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
- **Remembered cell.** A cell a harvester keeps so that it can drive back to its field after
  unloading (claims 6 and 12).
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
  at construction completion in play or during placement at game set-up.
- **Start price.** The price of a produced item at the moment its production starts (S-0404,
  claim 2), which the original records on the produced object; claim 18 reads it.
- **Docking, payment and unloading animations.** Three animations of the building in the
  neighbouring cell (claim 11): the *docking animation* (the building's pre-production animation,
  played when unloading starts), the *payment animation* (its special animation, played at a
  payment unless it is already playing) and the *unloading animation* (its production animation,
  played, by a refinery-flag building only, when the harvester's load is empty). Each is played in its damaged form when the
  building's health ratio is at or below the yellow health threshold (S-0003, claim C9), and
  only if the building's art data gives that animation; a building without it plays nothing.
  Only the unloading animation delays the harvester (claim 11).
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
- **Chrono miner.** A harvester whose type has the teleporter flag: it moves by teleporting.
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

    **Visibility conditions.** While eligible, the sprite is suppressed during movement
    or body turning, while an erasing weapon is removing the vehicle, and during its
    teleport disappearance or reappearance state. For a ground-driving harvester the
    movement test means either a turning body or an active track with a positive
    integer per-tick progress step *s* (S-0207 claim 2), not merely a positive
    double-precision speed fraction *ρ* (S-0207 Terms). The step is
    *s* = trunc(*s*₂ × *ρ*), truncated toward zero after the floating-point
    multiplication, with *s*₂ as defined in S-0207 claim 2. On an active track
    with *ρ* > 0 but *s* = 0, the sprite is shown if the body is not turning and
    the other visibility conditions permit it. Conversely, *s* > 0 suppresses
    it even when the progress budget buys no track point and the position does
    not change that tick (S-0207 claim 3). A destination alone does not suppress
    it. For a teleporting harvester it means a teleport journey still in progress.
    Ordinary object visibility still applies.
    Thus driving to another resource cell suppresses the sprite without restarting its
    frame cycle, and it can reappear on stopping before another successful load step.

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
    - Backing: `measurable` (H20)
    - Confidence: high for the absence of a docking-cell exception in the harvest rules

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

   A chrono miner always moves in the last case, even at 768 leptons or less (it teleports,
   claim 17).
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
    | 2 | it does not stand on the docking cell | its destination is set to the docking cell, again at each such processing (an ordinary move: S-0212 finds the path, and the harvester drives forwards) |
    | 3 | it stands on the docking cell, and its body facing is not exactly 16384 (map east) | it and the refinery become tethered (glossary, *tether link*), and it turns its body on the spot towards 16384 |
    | 4 | it stands on the docking cell, facing exactly 16384, and not moving | it and the refinery are tethered (as in row 3), and the refinery queues *unload* on it, without an immediate start, unless the refinery is being sold; *unload* becomes current by S-0215's start rules (claim 11) |
    | 5 | otherwise (on the docking cell, facing 16384, still moving) | tethered as in row 3; nothing else |

    A tethered harvester whose refinery refuses (row 1's refusal while tethered) keeps its contact
    and goes on to the other rows.

    **Tag notifications.** Ordinary refinery arrival and docking in *enter* add neither an
    entered-by raise on the refinery's tag nor a destroyed-by-anything raise on the
    harvester's own tag (S-0524 claim 14). Cell tags encountered on the approach are
    separate; docking does not consume the harvester.

    So there is no backing in: the harvester arrives with whatever facing its path gives, and the
    turn of row 3 happens on the docking cell. The turn of row 3 aims at exactly 16384, so row 4
    holds at the first processing after the turn has ended; claim 11's facing range then holds at
    once. Under the default quirk, a refinery that is not a dock-unload refinery and has
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
      and the non-dock-unload contact stall,
      H14 for the second route, its ordering and draw count; S-0524 claim 14,
      scenario C16 for the absence of docking tag raises). The selling-state
      arrival refusal and its destination-absent scatter case are `unmeasured`:
      H14 proposes timing controls but establishes no reliable overlap before
      the sale independently releases contact and tether.
    - Confidence: medium; high for the docking cell at (3, 1) from the top-left cell; low
      for being erased and for the refinery that is not a dock-unload refinery; high for the
      second route's track-completion event, the absence of facing and stopped tests and its
      scatter-request options; medium for its ordering and retained state

10a. **A blocked docking cell does not expire the reservation (Q-0371).** Holding
    contact reserves a dock slot, not exclusive occupancy of the docking cell.
    The docking exchange of claim 10 directs the contacting harvester, not an
    unrelated friendly harvester standing on that cell. It does not request that
    blocker to scatter. While the refinery continues accepting the contacting
    harvester, failure to reach the cell does not by itself release contact,
    send it to a queueing cell or select another refinery. There is no elapsed-tick
    limit in this docking rule: claim 10 row 2 keeps offering the same docking
    destination at each *enter* processing. Other movement, orders, loss of contact
    and refusal rules can still end or change this situation.

    A harvester resting there without contact does not unload merely by occupying
    the cell; claims 10 and 11 still require contact. This spec supplies no automatic
    eviction of that resting harvester. General occupancy and scatter rules remain
    separate, and a vehicle factory's apron clearing is S-0417 claim 13, not a
    refinery-wide docking rule.
    - Backing: `measurable` (H20)
    - Confidence: high for the docking exchange, retained reservation and absence of
      a docking timeout; medium for the resting-occupant case pending movement tests

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
    | Unloading, not finishing, a building there, counter ≥ 900 × *D*, load not empty | the building plays its payment animation unless that is already playing; the harvester is paid (claim 12) for the tiberium it holds with the lowest registration index, the whole amount at once; counter set to 0 (its period is not restarted) | 1 tick |
    | Unloading, not finishing, a building there, counter ≥ 900 × *D*, load empty | becomes *finishing*; the building's payment animation, if playing, is stopped, and a refinery-flag building in the neighbouring cell plays its unloading animation; the counter is left as it is | 1 tick |
    | Finishing, and the building in the neighbouring cell is a refinery-flag building that is still playing its unloading animation | nothing | 1 tick |
    | Finishing, with a destination and a queued behaviour other than *harvest* | leaves the unloading state; stops if moving; starts the queued behaviour if it is ready to change; contact is kept | standard delay of the behaviour then current (one draw) |
    | Finishing, otherwise | leaves the unloading state; queues *harvest* without an immediate start; if it is ready to change, breaks off contact and starts the queued behaviour | standard delay of the behaviour then current (one draw) |

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
    building's unloading animation (none when the building has no such animation data, so the
    harvester leaves in the tick after it became *finishing*). The docking and payment animations
    never delay the harvester. Back in *harvest*, the harvester starts seeking and drives to its
    remembered cell (claim 3).

    *Unloading state.* From the start row until it leaves (the rows above, or a scatter), the
    harvester is in the unloading state, and a vehicle in that state is not ready to change (S-0215,
    claim 11, the "letting out" transition), so no queued behaviour starts by S-0215's per-tick start.

    *Refinery sold while unloading.* At the first processing of a sale of the refinery, the refinery
    tells every unit docked with it to leave (S-0408, claim 6). A harvester in the unloading state leaves it at
    once, receives a forced vehicle scatter request without a threat point and not move-anywhere
    (S-0207 claim 16), queues *harvest* and starts it if it is ready to change
    (it now is), keeping the part of its load that has not been paid; the sale then breaks off the
    contact. A harvester still in *enter* at that moment (tethered on or towards the docking cell)
    queues *guard* instead and receives the same kind of vehicle scatter request, subject to
    the usual refusals (a turning body or an existing destination can prevent movement).

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
    once. The harvester keeps the part of its load that has not been paid; it is not paid for it
    later unless it unloads again. The original also has a route by which an order ends unloading
    while contact is kept (the harvester then becomes *finishing* with its load kept, and the row
    for a destination and a queued behaviour applies), but orders of S-0213 never reach it, since
    only a dock-unload refinery queues *unload* and an order breaks off that contact first.
    - Backing: `measurable` (scenario H1; its variant e for the three animations; H12 for the sale,
      contact lost while unloading, an order while unloading and the free dock slot).
      The order-ending route with contact kept is `unmeasured`: the orders in S-0213
      break this contact before that route could be observed, and no other reachable
      input is known to preserve it.
    - Confidence: high for the payment per tiberium and the dump threshold; medium for the exact
      tick of each payment and the 1-tick intervals, for the extra interval before *finishing* and
      for the one-tick difference of the first payment; medium for the unloading animation being
      the one the harvester waits for and the other two not delaying it; low for the damaged forms
      and for which building types play the docking animation (this claim says any building in the
      neighbouring cell); medium for the stall after contact is lost, for the order's route and for
      the delays of the two *finishing* rows; medium for the sale's run-away and the *guard* of a
      harvester still in *enter*; low for the route with contact kept

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
      means no slave has been sent out since the group last came together. The original's
      computer-controlled condition does not apply to player houses in our engine; it may apply
      to the neutral and special houses. The initial value of *s*<sub>g</sub> is 0 ticks,
      not the miner's creation tick (claim 21). The short-scan test is the ring search from the
      miner with the slave-miner short scan range.

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
      leave again (claim 21).
    - **Lost slaves.** A slave that is killed or lost starts the slave regeneration field's
      delay in ticks; a replacement is created inside the miner at the first group update after
      that delay has elapsed (claims 19 and 21).
    - The miner's initial slaves and their emergence are claims 19 to 21. Claim 22 gives the
      supported harvest-order and automatic-deploy outcomes. Claim 24 fixes site selection, and
      claim 25 gives relocation-failure inputs and the immediate post-pack-up destination.
    - Backing: `measurable` (scenario H7; its variant b for the kick wrap and its switch, variant e
      for the group's rest tick, variant f for the deployed miner and the slaves' health, and
      S-0215 scenario B31 for the separate area-guard effect)
    - Confidence: medium for the slave cycle and the wake-up rule; low for the miner's own
      movement

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
      (claim 9), which it reaches by teleporting,
    - while it is moving in the *returning* phase and a refinery would accept it (the normal
      choice of claim 8 finds one), it drops its movement, so that its next processing reserves it,
    - when it starts seeking with a destination (for example after a *harvest* order on a cell, or
      the remembered cell it adopts as its destination at that moment, claim 3), it drops that
      destination and runs the ring search from where it stands. This rule holds for a harvester
      whose current locomotor is the teleport locomotor, whether or not its type has the teleporter
      flag, and not for one whose type has the flag but that moves by another locomotor; the two go
      together in normal data,
    - in *guard*, a chrono miner queues *harvest* again (1-tick interval, ending the processing as
      a case of row 2 of S-0215 claim 24's vehicle table; the claim 13 note on row 3 does not
      apply to it) when one of its eight
      neighbouring cells holds a building of its owner whose type has the refinery flag (Import
      mapping; the dock list is not consulted here), or when it is full and still moving. In the
      original this rule is reached only for a human-controlled owner (a computer-controlled owner
      gets claim 13's rule instead); in our engine it applies to every player house.
    How it teleports is S-0207 claim 20's. A vehicle whose type has the harvester flag does not
    phase in after a jump (S-0207 claim 20 f).
    - Backing: `measurable` (scenario H8; S-0207 scenario D22 run 2 for no phasing in; H8 part g
      for the "full and still moving" trigger, `unmeasured` if no run reaches it)
    - Confidence: medium; high for dropping a destination when seeking and for no phasing in.
      Low for the "full and still moving" trigger

### A building's free unit

18. **When and where a free unit appears.** A building whose type has a free unit (Terms) gets
    one at its grand opening (Terms), subject to the exclusions below. For a building constructed
    in play, this is at the end of its build-up, in the building's own turn. It gets none:
    - when it is captured (a capture is not a grand opening, and a building has one grand opening
      at most),
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

    A free unit that is placed queues *harvest* with an immediate start, whatever its type, so it
    starts the harvest cycle at once (S-0215 claim 15's placement fall-back is not involved).
    Creating it takes the object creation draw (S-0202, claim 10); the placement and the searches
    take none, except the draws that putting a unit of that type on the map takes in general (none
    for a harvester). The building's other grand-opening effects (its slot animations, S-0108 claim
    11, and any aircraft its type comes with) are other specs'.
    - Backing: `measurable` (scenario H13, parts a to g)
    - Confidence: high for the timing at the grand opening, for no unit after a capture and for the
      first place and its facing; medium for the two searches, the refund, the start of *harvest*
      and the start-price condition; high for suppression of the set-up free-unit grant

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
    The transferred slaves are not replaced by a second working group.

    Each initial or replacement infantry creation takes the ordinary object-creation draw
    (S-0202 claim 10). Keeping it inside takes no placement search or placement draw.
    - Backing: `measurable` (scenario H15)
    - Confidence: high for creation, count, ownership, hidden state and no payment,
      medium for the creation-source and draw controls

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
    With a site, it moves there and enters travelling. A working vehicle that is no longer
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
    still on resource land, no longer in *harvest*, the slave re-enters searching.
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
    *guard*, no longer runs while *move* remains current. No automatic return
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
> *X*ₐ = trunc(RZ₅₃(*X* + RZ₅₃(30 × cosT θ))),
> *Y*ₐ = trunc(RZ₅₃(*Y* − RZ₅₃(30 × sinT θ))), and *Z*ₐ = *Z*.

The angle is in radians, the 30 in leptons, and both coordinate conversions truncate
toward zero. Here sinT and cosT are the table sine and cosine required by S-0205
claim 6, including the table's negative-angle quirk; they are not exact mathematical
sines and cosines or library evaluations. Table results are single-precision values
widened exactly for the double-precision products and sums, which round toward zero.
The cosine contributes to *X* and the sine to *Y*, not the reverse. Project
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

| Phase | Condition at a processing | Result | Next processing |
|---|---|---|---|
| any | owner owns no building of a dock type | queues *guard* | 1 tick |
| seeking | full | → returning | 1 tick |
| seeking | not full, has a remembered cell | drives there and forgets it; no search in this processing (chrono miner: claim 17) | standard delay |
| seeking | not full, no remembered cell, has a destination | stays seeking; no search | standard delay |
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
| Rules, vehicle type section, `Harvester` | harvester (Terms); eligibility for the vehicle harvesting sprite (claim 5a) | boolean | no |
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
objects, which the void rule exempts there. Named test data is the map's override sections; values
are invented. Tick
numbers, behaviours and phases, loads (pips, or a recorder that reads them), cell stages, the balance
and the scenario stream's draw count come from a recorder (a read-only memory recorder, glossary
*recorder*), and orders are given by scripted input (glossary, *measurement setup*), as the project
lead allowed. Unless a scenario says otherwise, the harvester is the human
house's, the refinery has stock art, there is one harvester, and resource cells do not grow (growth
and spread fractions 0). Unless stated otherwise, use game-speed index 2 at 800 × 600,
with director and mods off except for the scenario's named test data.

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
  separated from a failure to wake; it no longer implies an unknown initial last-rest tick.
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
- *Runs:* 3 for the main run, 1 for each variant.

**H8. Chrono miner (claim 17).**
- *Setup:* a chrono miner far (40 cells) from its refinery, chrono too-far field 50.
- *Observable:* its cell per tick (a teleport is a jump of more than 2 cells in one tick), its
  behaviour and phase.
- *Expected:* when full it reserves the refinery and teleports to dock. With the field set to 10, it
  teleports to the queueing cell. Standing next to its owner's refinery in *guard*, it queues
  *harvest* by itself. Part (d): ordered to harvest a cell 10 cells away while a richer cell lies
  next to it, it drops the order's destination and loads at the cell the ring search from where it
  stands finds. Part (e): while it is moving in *returning* and a refinery would accept it, its
  movement stops and it reserves that refinery on its next processing. Part (f): with the
  refinery busy and the chrono miner 2 cells (512 leptons) from it, too-far field 1: it moves
  (teleports) to the queueing cell, although an ordinary harvester stays there (H4 part b).
  Part (g), the "full and still moving" trigger: a full chrono miner is returning to its only
  refinery over a long way (40 cells), and that refinery is sold while the miner is moving, so
  claim 13 puts it into *guard* with its locomotor still moving. Expected: at every processing of
  *guard* at which the recorder shows the locomotor moving, the behaviour one tick later is
  *harvest* (which falls back to *guard* at once, claim 13, as the house owns no refinery); once
  the locomotor has stopped, it stays in *guard*. If no processing of *guard* ever shows the
  locomotor moving (the miner stops at once when the refinery goes), the part shows nothing and
  the trigger stays `unmeasured` for that reason.
- *Falsified if:* it drives, stays in *guard* next to its refinery, goes to the ordered cell in part
  (d), keeps moving in part (e), or stays where it is in part (f); in part (g), a processing of
  *guard* that shows the locomotor moving and is followed by a behaviour other than *harvest*, or
  *harvest* after the locomotor has stopped.
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
- *Runs:* 2 per approach side and foundation size, 3 without the flag; 1 per engine-only control.

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
  stationary units. Use the recorder to confirm that no cell qualifies; the harvester must
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
  3 freed runs for e. Use game-speed index 2, 800 × 600, director and mods off for e,
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
  the sale's first processing, with the unpaid rest of its load, and drives off (to another
  refinery of the house when there is one, part a2 having a second refinery). Part a3: sold while a
  harvester is in *enter* on the docking cell, still turning: it is in *guard* soon after and does
  not dock. Part b (no sale): the first harvester is given a move order in the tick after its first
  payment: the second harvester takes contact with the refinery from the next of its own
  processings on (the slot was freed in the order's tick); the first harvester starts moving at its
  next processing of *unload* (1 tick later) and keeps its unpaid load; ordered back to harvest, it
  docks again and is paid for that load.
- *Falsified if:* in part a, the first harvester leaves *unload* without an order, or is paid; in
  part a2, it stays in *unload* or loses its load; in part a3, it unloads at the sold refinery; in
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
  director and mods off. A named test map/art variant gives the refinery a rectangular
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
  initial observation. Director and mods are off apart from these named test overrides.
- *Procedure:* produce a miner and record from production start, before release. Repeat with a
  map-created miner, a starting miner, and a grand-opening free unit whose type is the miner.
  Separately create a deployed building of the slave-miner type. Record creation counts,
  owners, inside/on-map state, last-rest tick, credits, per-type owned and ever-built counts,
  built total and creation draws. Controls use counts 0 and 1 and a miner without a slave type.
- *Expected:* 3 hidden ready infantry exist at the master's creation, owned by its house,
  with no occupied infantry cells and no additional credit debit. Last-rest initially reads
  0 even for a miner created later in the game. Each slave takes one creation draw. Their
  creations increment eligible owned counts but never the infantry ever-built count or built
  total, before or after release of the master. Counts 0 and 1 produce 0 and 1 initial slaves.
  A deploy/pack-up pair retains the existing working slaves under S-0415 claims 6 and 10.
- *Falsified if:* initial creation waits for release or wake-up, uses another owner, places the
  initial slaves on cells, deducts a slave price, records creation tick as initial last-rest,
  or raises an infantry ever-built count.
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

**H20. Friendly harvester on the docking cell (claims 3a and 10a; Q-0371).**
- *Setup:* tiny flat skirmish map, stock refinery and two ordinary harvesters of one
  human house, reachable ore field, speed index 2, remote bystander; director and
  mods off. Keep the test refinery powered on. Use the normal measurement setup.
- *Procedure:* while B unloads, move A to the docking cell and stop it after B leaves.
  Order B to enter the refinery again. Record for 2000 ticks, then give A a harvest
  order. Repeat three times; repeat with the field reachable only past the refinery,
  and with a second available refinery. A computer-owner run records the original's
  own orders separately rather than assuming human input reaches that owner.
- *Observable:* both units' cells, current and queued behaviours, destinations,
  contact and tether, dock occupancy, and any movement or scatter request, tick by tick.
- *Expected:* merely occupying the cell does not give A contact or unloading. B's
  docking exchanges keep directing B to that cell without a docking timeout,
  alternative-refinery choice or a request to scatter A. A's harvest seeking can
  select the reachable ore destination despite B's reservation. Any later route
  failure is logged separately from the harvest destination assignment.
- *Falsified if:* the docking exchange evicts A, expires B's reservation because
  time passed, queues B elsewhere or chooses another refinery, or harvest seeking
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

**H21. Slave-place ordering, two clocks, facing and scan tests (claims 21, 26).**
- *Setup:* common skirmish setup, speed 2, 800 × 600, director and mods off. Named map
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
- *Setup:* common setup, speed 2, stock slave miner near a field, director and mods off.
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
  data lengthens the harvest behaviour's standard delay; no engine add-ons, director or
  other mods. Observe its destinations, current/queued behaviours, group state and slaves.
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
- *Setup:* a tiny stock-data skirmish map, game speed 2, director and mods off,
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
- *Setup:* a tiny purpose-made stock-data skirmish map at speed 2, director and
  mods off, human-owned slave miners and one distant bystander computer player.
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
  speed index 2, 800 × 600, director and mods off. Run each vehicle separately so its
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
  target it with an erasing weapon, and record a chrono miner's journey. The extra
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

## Open questions

- Q-0550 is answered by claim 5a, F8 and H26. S-0111 claim 14 already covered load
  pips. The fixed harvesting sprite is not an animation instance, so its frame cycle
  does not require a new S-0109 playback claim.
- Q-0580 questions 1, 2 and 4 were already answered by claim 5a, corrected F8
  and H26. S-0205 claim 6 already required table trigonometry before the request.
  The prior F8 omitted that precision rule and also reversed its coordinate
  contributions. Question 3 is already answered by S-0102 claim 13 and scenario
  D43. Question 5 is answered by claim 5a's integer-step test and H26's zero-step
  movement control; S-0207 claim 2 already defines the step and its rounding.
- Cross-spec coverage: S-0215 claim 4 already covers the reachable 450-tick no-op building *move* described in claim 27a and cites H25. S-0415 claim 9 already recognizes that claim 27a's site offer can supply a guard spot, including with lobby redeploy off; pack-up legality remains S-0213 claim 12's, unchanged here.
- Q-0500 is answered by claim 27a. Claim 28 and S-0415 claims 9 and 10
  already identify the shared guard spot and its packing consequences.
  S-0215 claims 7 and 11 already fix queue refusal and building starts.
  The site-offer gates and destination retention are now explicit here.
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
  with H19 to confirm them. No failure is assumed to mean stay-in-place.
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

## Questions for the project lead

None. The nine quirk switches of this spec (`quirks.harvester_failed_step_off_origin`,
`quirks.refinery_distance_wraps`,
`quirks.slave_kick_delay_wraps`, `quirks.zero_load_period_stalls` and
`quirks.unload_without_contact_stalls`, `quirks.slave_relocation_failed_search_origin`
and `quirks.slave_negative_capacity_stalls`, `quirks.zero_capacity_harvester_cycles`
and `quirks.non_unloading_refinery_accepts`, each default original) follow the project's quirk
convention.
