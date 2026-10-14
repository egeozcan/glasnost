---
id: S-0402
title: Harvest cycle (loading, field and refinery choice, docking, unloading, income, slave and chrono miners)
topic: economy
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0140, Q-0141, Q-0142, Q-0143, Q-0144, Q-0145, Q-0194, Q-0197]
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
(stages, bales, growth) are S-0401's.

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
- **Grand opening.** S-0108's term (claim 11): the moment a building first completes, at the end of
  its build-up.
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

## Behaviour

### The cycle

1. **Capacity and the load.** A harvester carries at most *C* bales. Each bale it loads is added
   to the amount of the tiberium of the cell it came from. A harvester type with *C* = 0 is bad
   data: its fill ratio is 0 / 0, which is not a number, and the cycle then runs without ever
   carrying a bale (Edge cases). The load
   survives everything in this spec except unloading; what happens to it when the harvester dies
   is S-0312's.
   - Backing: `measurable` (scenario H1; its variant d for *C* = 0)
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
   Shroud does not matter in a skirmish. The search takes no draws. Because worth grows with the
   value per bale, a harvester prefers a tiberium of greater value per bale to one of smaller value
   at the same ring distance, and a denser cell to a sparser one.
   - Backing: `measurable` (scenario H2; variants b and c for the entry test and the movement zone)
   - Confidence: high; medium for the entry test (which entry outcomes count as "could enter")

5. **Load steps and their timing.** In the *loading* phase the harvester is processed every tick.
   A load step is made at a processing only when the *step counter* it sees is at least 9. The
   counter rises by 1 on a schedule set by its *pacing period* *p* and the tick in which that period
   was last started: every *p* ticks after the start for *p* > 0, in every tick from the start on for
   *p* < 0, and never for *p* = 0 (`quirks.zero_load_period_stalls`, Edge cases). Starting a period
   replaces the schedule of the one before. A rise in tick *t* first counts at the processing in tick
   *t* + 1, and it counts there even when the processing in tick *t* set the counter to 0 (Formulas
   F6). With nothing interrupting, the first bale is therefore loaded 19 ticks after loading starts
   and each further bale 9*L* + 1 ticks after the previous one (Formulas F1); the "+ 1" and the 19
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

### Refineries

8. **Choosing a refinery.** A harvester chooses among the buildings its owner owns that are of one
   of its dock types, are on the map, are connected to it by its movement zone, and accept
   it (claim 10). *Connected* means that two cells lie in the same reach region (S-0212, claim 4)
   for the movement zone of the harvester's type, read for the deck when the harvester is on a
   bridge deck: the cell of the harvester's heading point (S-0212, Terms; its own position when it
   is not moving) and the cell of the refinery's reference point. A harvester whose heading-point
   cell lies outside the usable area counts as connected to every refinery. It takes the nearest candidate. The distance is measured as the square of the 2-D
   distance between the two reference points, an integer of leptons squared, *dx*² + *dy*², computed
   as a signed 32-bit integer that wraps (Formulas F5, `quirks.refinery_distance_wraps`; Edge
   cases). A primary building can override the nearest rule. The choice is made at two levels by
   the same rule:
   - within each dock type, among the owner's candidates of that type, ordered by holding order
     (Terms); this gives the *best of the type*, when the type has a candidate; -
   across types, among the bests of the types, ordered by the dock list; the best of a type
     counts as a primary building when it is one.

   At each level: when no candidate is a primary building, the choice is the nearest, and a tie
   goes to the earliest. Otherwise let *P* be the last primary building of the level (for the
   across-types level, the last type whose best is a primary building, by that best); the choice is
   the nearest among *P* and everything after it, and a tie goes to the earliest of them, so *P*
   wins ties. The levels do not combine into one flat list: a type's best counts as a primary
   building only if it is one itself, also when the type has a primary building and a nearer
   candidate after it is the type's best (scenario H4 part i). In the normal choice a refinery whose dock slots are all held by
   other harvesters does not accept; in the *waiting choice* (claim 9) slots are not looked at.
   - Backing: `measurable` (scenario H4 parts a, c and i; part c's production-order run and part j
     for the holding order of produced and captured buildings; part e for the wrap, `unmeasured` if
     the original cannot load a map that large; part k for the connectivity cells)
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
    switched off (S-0405, claim 10). Whether a refinery that is being erased also refuses is not
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

    So there is no backing in: the harvester arrives with whatever facing its path gives, and the
    turn of row 3 happens on the docking cell. The turn of row 3 aims at exactly 16384, so row 4
    holds at the first processing after the turn has ended; claim 11's facing range then holds at
    once. A refinery that is not a dock-unload refinery agrees to contact but never directs the
    harvester: the harvester stays in *enter*, processed at every standard delay, until something
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
      H10 for the docking cell, the approach, the turn, the delays and the start of *unload*,
      H14 for the second route, its ordering, refusal request, retained state and draw count)
    - Confidence: medium; high for the docking cell at (3, 1) from the top-left cell; low
      for being erased and for the refinery that is not a dock-unload refinery; high for the
      second route's track-completion event, the absence of facing and stopped tests and its
      scatter-request options; medium for its ordering and retained state

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
    counter has reached 900 × *D*, as for a payment, so after its last payment it waits one more
    interval of 900 × *D* before it becomes *finishing*.

    So unloading takes one payment per tiberium carried, each ⌈900 × *D*⌉ ticks after the
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
      contact lost while unloading, an order while unloading and the free dock slot)
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
      behaviour, retained state and draws)
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
      to the neutral and special houses. The initial value of *s*<sub>g</sub>, before any slave has
      gone out, is undetermined (Open questions). The short-scan test is the ring search from the
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
      walks to the cell found, and loads one bale (S-0401, claim 3) at each processing of its
      *harvest*, which comes every *h* ticks, *h* being its type's harvest-rate field; its capacity
      is its type's storage field. When full, it remembers its cell and walks back to the miner
      (for a deployed miner, to the foundation cell (*W* − 1, ⌊*H*/2⌋) of it, counted from its
      top-left cell). When its cell is exhausted it searches again; when a search finds nothing it
      walks back with what it has.
    - **Delivery.** A slave that reaches the miner is paid as in claim 12, to the miner's owner, for
      each tiberium it carries in turn (lowest index first, main payment then bonus), goes inside
      the miner, and stays there for the slave reload field's number of ticks, after which its
      health is restored to its strength and it is ready to be woken again.
    - **Lost slaves.** A slave that is killed or lost is replaced, inside the miner, after the slave
      regeneration field's number of ticks.
    - Where the miner deploys, when it undeploys and moves on, and its own scan fields (the long
      scan and the scan correction) are not specified here (Open questions).
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
    one at its grand opening (Terms), so at the end of its build-up, in the building's own turn. It
    gets none:
    - when it is captured (a capture is not a grand opening, and a building has one grand opening
      at most),
    - when the map pre-places it (it has no grand opening, S-0009 claim 40; also when the original
      reaches that step during the set-up, the free unit is skipped then),
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
    - Backing: `measurable` (scenario H13, parts a to f)
    - Confidence: high for the timing at the grand opening, for no unit after a capture and for the
      first place and its facing; medium for the two searches, the refund, the start of *harvest*
      and the start-price condition; medium for the pre-placed building (S-0009's)

## Formulas

**F1. Load timing** (claim 5). With loading started at tick *t*₀, *L* the load step period, and
nothing interrupting, bale *j* (*j* = 1, 2, …) is loaded at tick

> *t*₀ + 19 + (*j* − 1)(9*L* + 1)

integers, ticks. A harvester of capacity *C* that starts empty on a field that never runs out has
its last bale at tick *t*₀ + 19 + (*C* − 1)(9*L* + 1). It is full from then on, but the phase ends
only at the next load step, in tick *t*₀ + 19 + *C*(9*L* + 1), when that step fails because the
harvester is full; it is *returning* from the processing after that. For *L* ≤ 0 see Edge cases.

**F2. Unload timing** (claim 11). Let unloading start at the processing in tick *u*₀ (facing east,
in contact). The counter seen by a processing in tick *t* is *c* = *t* − *u*₀ − 1 until the first
payment, and *c* = *t* − *u* after a payment in tick *u*. The processing after the start row comes
in tick *u*₀ + *d*, *d* being the standard delay drawn at the start row, and every tick after it
has a processing, so the first payment comes in the first tick *t* ≥ *u*₀ + *d* with

> *t* − *u*₀ − 1 ≥ 900 × *D*, that is *t* = max(*u*₀ + *d*, *u*₀ + 1 + ⌈900 × *D*⌉),

and each later payment in the first tick *t* with *t* − *u* ≥ 900 × *D*, that is *t* = *u* +
⌈900 × *D*⌉, where *u* is the tick of the previous payment (every such tick has a processing). When the load is empty at such a
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
integers, with the true result *r* replaced by its residue modulo 2³² in the range −2³¹ to 2³¹ − 1 (that
is, by *r* − 2³² when *r* ≥ 2³¹):
- the kick tests *k* + *s* < *t* (claim 14), integers, ticks, and
- the refinery distance *dx*² + *dy*² (claim 8), integers, leptons squared; it wraps when the true
  value is at least 2³¹, which needs a distance of at least about 46341 leptons (about 181 cells).
  Two such distances are compared as signed numbers.
Each wrap has its own quirk switch, in claims 8 and 14.

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

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
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

**Ordering.** Load steps, unloading payments and slave deliveries all happen in the harvester's
(or slave's) turn, in the turn pass (S-0203, row 5); a payment in tick *n* is seen by production in
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
  and returns to harvest; it never carries a bale, is never paid, and repeats the cycle. Our
  engine does the same (no switch; the importer warns). Confidence low for both outcomes, which
  scenario H1 variant d checks.
- **More than four tiberiums** (claim 15): departure, importer warning.
- **The refinery distance wraps** (claim 8, F5) for a distance of about 46341 leptons (181 cells)
  or more between the reference points, which a large map's diagonal can reach.
  `quirks.refinery_distance_wraps` (default: the original): the squared distance is a signed 32-bit
  integer that wraps, so a refinery that far away can look nearer than one close by. With the
  switch off, the square is exact (64-bit) and the nearest
  refinery wins as claim 8 reads without the wrap. No importer warning: the data is valid.
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
- **A refinery that is not a dock-unload refinery** (claim 10). It agrees to contact but never
  directs the harvester to a docking cell, so the harvester stays in *enter* and never unloads,
  until an order or another rule ends it. Our engine does the same (no switch; the importer warns
  when a building type in a harvester's dock list lacks the dock-unload flag). Confidence low.
- **Harvester truce.** Out of scope: the original's lobby has a harvester-truce setting (harvesters
  immune to damage), but its skirmish start writes it off in every skirmish (S-0412, claim 1), so
  harvesters take damage normally and nothing in this spec depends on it.
- `quirks.table_root_distance` (S-0204) covers the too-far and wait distances of claim 9. Default: the original's tabulated square root, truncated to an integer. With the switch off, use the exact integer square root, ⌊√*n*⌋, of the squared distance instead.
- `quirks.chop_rounding` (S-0205) covers F3.
- `quirks.slave_kick_delay_wraps` (claim 14), `quirks.refinery_distance_wraps` (claim 8),
  `quirks.zero_load_period_stalls` (claim 5) and `quirks.unload_without_contact_stalls` (claim 11)
  together with `quirks.harvester_failed_step_off_origin` (claim 13), are this spec's own switches.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, vehicle type section, `Harvester` | harvester (Terms) | boolean | no |
| Rules, vehicle type section, `Weeder` | weeder (Edge cases) | boolean | no |
| Rules, type section, `Storage` | capacity *C* (claim 1) | integer | 0 |
| Rules, type section, `Dock` | dock types (Terms, claim 8) | list of building type names | empty |
| Rules, type section, `Teleporter` | chrono miner (claim 17) | boolean | no |
| Rules, type section, `ResourceGatherer`, `ResourceDestination` | together with an enslaves field, a slave miner (claim 14) | boolean | no |
| Rules, type section, `Enslaves`, `SlavesNumber`, `SlaveRegenRate`, `SlaveReloadRate` | slave type, number of slaves, regeneration and reload delays in ticks (claim 14) | type name, integers | none, 0 |
| Rules, infantry type section, `HarvestRate` | a slave's interval between bales *h* (claim 14) | integer, ticks | 1 |
| Rules, `[General]`, `HarvesterLoadRate` | load step period *L* (claim 5) | integer, ticks | 2 |
| Rules, `[General]`, `HarvesterDumpRate` | dump rate *D* (claim 11) | decimal (S-0003), game minutes | the double nearest 0.016 |
| Rules, `[General]`, `TiberiumLongScan`, `TiberiumShortScan` | long and short scan ranges (Terms, claims 3 to 7) | lepton form: decimal cells × 256 | 8192 and 1536 leptons (32 and 6 cells) |
| Rules, `[General]`, `HarvesterTooFarDistance` | too-far distance of ordinary harvesters (claim 9) | integer, cells | 5 |
| Rules, `[General]`, `ChronoHarvTooFarDistance` | too-far distance of chrono miners (claims 9, 17) | integer, cells | 50 |
| Rules, `[General]`, `SlaveMinerShortScan`, `SlaveMinerSlaveScan` | slave-miner short scan and slave scan ranges (claim 14) | lepton form | 1280 and 4096 leptons (5 and 16 cells) |
| Rules, `[General]`, `SlaveMinerLongScan`, `SlaveMinerScanCorrection` | the miner's own movement (not specified, claim 14) | lepton form | 20480 and 768 leptons |
| Rules, `[General]`, `SlaveMinerKickFrameDelay` | kick delay *k* of the wake-up rule (claim 14) | integer, ticks | 2147483647, which wraps (claim 14, F5) |
| Rules, `[General]`, `PurifierBonus` | purifier bonus *b* (claim 12) | decimal, single | the single nearest 0.25 |
| Rules, `[General]`, `AIVirtualPurifiers` | virtual purifiers per difficulty index, original only, and for the neutral and special houses (entry 2, claim 12) | decimal-prefix signed 32-bit integer list of purifier counts (S-0003 claim 23, row C21), 511-byte text limit; no scalar hexadecimal forms; unreadable non-empty items retain a place with 0, empty items are skipped; missing or empty keeps the whole list in force, present non-empty replaces it; item conversion, wrapping and importer warnings are S-0003's | initially empty list (original: undefined result; engine: 0 virtual purifiers, departure, importer warning; claim 12) |
| Rules, building type section, `OrePurifier` | ore-purifier flag (Terms) | boolean | no |
| Rules, building type section, `Refinery` | refinery flag: the refinery-flag building rules of claims 3, 11, 13 and 17 (the idle step-off, the unloading animation and the harvester's wait for it, the chrono miner's guard rule) | boolean | no |
| Rules, building type section, `DockUnload` | dock-unload refinery (Terms, claim 10): directs a harvester to its docking cell, and queues unloading on an accepted track-completion report | boolean | no |
| Rules, mobile type section, `SpeedType`, `MovementZone` | locomotion class and movement zone used by the idle step-off (claim 13); scatter uses the vehicle search settings of S-0207 claim 16 | vocabulary forms owned by S-0211 | S-0211 |
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
and spread fractions 0).

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
  apart (no oracle part). Falsified if *L* = 0 loads a second bale in the oracle. (d) Capacity 0
  (the data layer may refuse it; then it is `unmeasured` for the original, with that reason): the
  harvester goes to the field and starts loading; expected: no bale at any tick, the harvester
  *returning* 19 ticks after loading starts (the tick of its first load step) or at the
  processing after it, no balance change in 3000 ticks, and the cycle repeating (it docks and
  returns to the field again); falsified if a bale is loaded, if it stays in *loading* for more
  than 21 ticks after loading starts, or if it is paid. (e) The three animations: the refinery's
  art gets a docking animation of 30 ticks, a payment animation of 200 ticks and an unloading
  animation of 60 ticks (animation types of known length). Run (i) with these, (ii) with a
  docking animation of 400 ticks, (iii) without an unloading animation. Expected, with *f* the
  tick of the change to *finishing* and *e* the tick at which the unloading animation ends: (i)
  contact is broken in tick *e* or *e* + 1, and not before *f* + 60; (ii) the same tick as (i)
  relative to *f*; (iii) contact is broken in tick *f* + 1. Falsified if run (ii) differs from run (i), if
  in run (i) contact is broken before the unloading animation has ended or more than one tick after
  it, or if in (iii) it is broken later than *f* + 1.
- *Runs:* 3 for the main run, 1 for each variant.

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
  - *Production-order run:* two refinery test types, R1 of line kind building and R2 a copy whose
    test data makes it a *defence* (S-0403's line kinds), so that both can be in production at
    once; both are dock types of the harvester. R1's production is started first and R2's second.
    When both are ready, R2 is placed first and R1 second, at exactly equal distances, neither
    primary. *Expected:* the harvester docks at R2 (placement decides). *Falsified if:* it docks at
    R1 (production start would decide).
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
- *Part e (distance wrap, original default).* A map large enough for two refineries, A about 150
  cells and B about 190 cells from the harvester along one axis; too-far field 50. Expected: the
  harvester drives to B's queueing cell (B's wrapped distance is negative). Falsified if it goes to A.
  If the original cannot load a map that large, this part is dropped and the wrap stays
  `unmeasured` (reason: it needs a map beyond the original's size).
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
  harvester is given a stop order (S-0213, table T1). *Observable:* its behaviour per tick.
  *Expected:* it is in *guard* one tick after the order and in *harvest* again within 3 ticks, and
  it harvests without further orders. *Falsified if:* it is still in *guard* 10 ticks after the
  order.
- *Part D (original only, record set and never cleared):* the same bystander house, its harvester
  on an isolated field so that it exhausts the field and goes *idle* (the house's no-field
  record is now set). A second harvester of the same house, standing in *guard* next to the
  refinery, and a new ore cell placed within its scan range by a trigger later. *Observable:* the
  behaviour of both harvesters per tick over 3000 ticks after the first went idle. *Expected:*
  neither harvester leaves *guard* by itself (the first stays in *guard* after its 105 ticks; the
  second, given a stop order after the record is set, stays in *guard* too), also after the new
  cell appears. *Falsified if:* either harvester is in *harvest* again without an order.
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
  no bonus, or a bonus that follows the lobby setting.
- *Runs:* 3 for each of a and b, 1 for each of c, d and e, 1 per lobby setting for f.

**H7. Slave miner (claim 14).**
- *Setup:* a slave miner of a human house with 3 slaves of capacity 2 and harvest rate 40, kick
  delay 300, the slave-miner short scan set by the test data to 2048 leptons (8 cells), an ore
  field 5 cells away, the miner on a cell that is not ore. The miner is given a stop order at
  tick 100, which forces *guard* (S-0213, table T1); the recorder logs the start tick of that
  *guard*, *s*<sub>m</sub> (S-0215, claim 3), and the first processing of *guard* follows at once.
- *Observable:* the tick at which each slave leaves, *T*₁ for the first; the ticks between a
  slave's bales; the payments to the house on delivery; the time slaves spend inside before leaving
  again; a damaged slave's health after its reload; the replacement time of a killed slave.
- *Expected:* the slaves first leave at a processing of *guard* with *t* > *k* + *s*<sub>m</sub>
  and *t* > *k* + *s*<sub>g</sub>, so *T*₁ ≥ *k* + *s*<sub>m</sub> + 1 = 401 and, when the group's
  rest tick before any slave has gone out is at most *s*<sub>m</sub>, within one standard delay of
  the first processing after tick 400, and 40 ticks between a slave's bales, one payment per
  delivery, reload and regeneration times equal to their fields; a damaged slave's health at its full
  strength after reload.
- *Falsified if:* the slaves leave at or before tick 400, or the intervals differ. Not leaving
  within 3000 ticks does not falsify claim 14: it says the group's rest tick before any slave has
  gone out is large, and *T*₁ − *k* − 1 then gives its value (Open questions).
- *Variants:* (b) the wrap: the miner stands on an ore cell (so the group's rest tick plays no
  part) and the kick key is absent (the default 2147483647); the stop order is given at tick 100
  as above. Expected: the slaves leave within 10 standard delays of *s*<sub>m</sub>, because the
  kick sum wraps negative for *s*<sub>m</sub> ≥ 1; falsified if they have not left by then. If
  the recorder shows *s*<sub>m</sub> = 0 for a miner that stands in *guard* from the start of the
  map, that run expects the slaves never to leave, since 2147483647 + 0 does not wrap. On our engine with
  `quirks.slave_kick_delay_wraps` off, the slaves of the *s*<sub>m</sub> = 100 run never leave
  (no oracle part). (c) The miner standing on an ore cell, with no field within the short scan,
  kick delay 300: the slaves still leave once *t* > *k* + *s*<sub>m</sub>; falsified if they stay.
  (d) The miner on a non-ore cell with the field 9 cells away (beyond the short scan): the slaves
  do not leave in 3000 ticks; falsified if they do. (e) The group's rest tick: after the slaves
  of the main run have all returned at tick *r* and the group is at rest again, with the miner
  still in *guard* on a non-ore cell: the next leave comes at a tick after *r* + *k*; falsified if
  slaves leave at or before *r* + *k*. (f) The deployed miner: the miner is ordered to deploy next
  to the field; the cell where the slaves reach the miner to deliver is the foundation cell of
  claim 14 (*W* − 1, ⌊*H*/2⌋) counted from its top-left cell; falsified if they deliver at another
  cell.
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
  flag off.
- *Observable:* per tick, from the tick the harvester goes *requesting* to its first payment: its
  cell, its body facing, whether its locomotor moves, its behaviour, and the tick of each
  processing of *enter* (recorder).
- *Expected:* the harvester drives forwards to the cell (*x*₀ + 3, *y*₀ + 1) from the refinery's
  top-left cell, for both foundation sizes; on that cell it turns on the spot until its facing is
  exactly 16384; *unload* becomes current after the first processing of *enter* that sees it on the
  cell, facing 16384 and not moving, unless the separately recorded track-completion route
  of H14 queued it earlier; the processings of *enter* are spaced by values the standard
  delay of *enter* allows (S-0215, claim 6). The run without the dock-unload flag: the harvester stays
  in *enter* for 3000 ticks and is never paid. Repeat from several approach sides (the harvester
  starting north, south, east and west of the refinery): the arrival facing varies, the final facing
  does not, and the harvester never reverses onto the cell.
- *Falsified if:* another docking cell, a cell that depends on the foundation size, a reversing
  approach, *unload* before the turn has ended through the processing route (the second route
  is distinguished by H14), or a final facing other than 16384; in the last run,
  a payment.
- *Runs:* 2 per approach side and foundation size, 1 without the flag.

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
  queueing cell; the test data's dump rate is slow enough that unloading lasts several payments.
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
  no harvester appears. Part f: the test data gives the house's country a vehicle cost factor of
  0 and a building cost factor of 1, so the refinery's price at its production start is its
  adjusted cost *C* (S-0404 claim 2, with *P*ₑ = 0; *C* above 0): no harvester appears. With the
  vehicle cost factor at 1 the price is *C* + *P*ₑ and it appears.
- *Falsified if:* another tick, cell or facing; a harvester in part d, e or the first run of f; no
  refund or another amount in part d.
- *Runs:* 1 for each part and run.

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

## Open questions

- Whether *return* is ever given to a harvester in a skirmish (claim 2); scenario H9 logs it.
- The slave miner's own movement: when and where it deploys and undeploys (claim 14), and the value
  of the group's rest tick before any slave has gone out (claim 14; scenario H7's main run gives it
  through *T*₁).
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

None. The five quirk switches of this spec (`quirks.harvester_failed_step_off_origin`,
`quirks.refinery_distance_wraps`,
`quirks.slave_kick_delay_wraps`, `quirks.zero_load_period_stalls` and
`quirks.unload_without_contact_stalls`, each default original) follow the project's quirk
convention.
