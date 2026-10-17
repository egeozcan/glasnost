---
id: S-0522
title: Parachutes and drops
topic: movement
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0736, Q-0750, Q-0751, Q-0752, Q-0753, Q-0754]
supersedes: []
---

# S-0522: Parachutes and drops

## Summary

Paradrop delivery in skirmish: plane arrival, passes, passenger release, parachutes, landing and
losses. Charging and firing belong to S-0512 claims 5 and 8; target choice and paradrop dispatch
belong to S-0513 claims 1 and 42. Network games and campaigns are out of scope.

Pointers:
- S-0513 claim 43 reuses claims 4 and 5 here for the spy plane's arrival edge, cell and draws.
- S-0107 claims 6, 17 and 21 and its state table give the *parachute* sequence and how deaths of
  falling infantry are shown.
- S-0214 (Terms and claims 6 to 9) gives what a falling unit cannot do.
- S-0312 claims 5 and 16 give the death of an aircraft and of its passengers.
- S-0011 claim 36 gives the drop-zone animation; it is not part of delivery.
- S-0501 gives aircraft flight. Its claim 20 makes a shot of a carrying aircraft a drop (claim 11
  here), and its table T2 leaves *paradrop approach* and *paradrop overfly* unchanged on a
  fall-back to rest.
- S-0511 claim 10 and its Formula F3 give falling and the fall rate.

Cross-spec coordination for this hand-off: S-0501 claims 13 to 15 and 20 and S-0301
claims 1, 4, 10 and 15 need claim 11's drop-shot exception. S-0513 claim 43 needs claim 2's
compact-list clarification. S-0109 claim 25 and S-0511 claim 10 need claim 13's absent-type
fall-rate distinction; S-0103 claim 8 needs its corresponding absent-type default and
open-question update. S-0215 claims 4 and 11 can use claim 17's fresh-paratrooper fixture
for their previously unmeasured readiness cases. These are coordinated corrections, not
alternative rules to apply alongside this spec.

## Terms

- **Paradrop plane.** An aircraft of the paradrop plane type (Import mapping) sent by a launch
  (claim 3). It is a *visiting aircraft* (S-0501) and carries its paratroopers as passengers
  (S-0501 claim 1, *passenger carrier*).
- **Paratrooper.** A passenger of a paradrop plane, and the same unit after it has been dropped,
  until it lands.
- **Payload list.** A pair of lists from the rules: a list of infantry types and a list of counts.
  Entry *i* means "one plane carrying *n*ᵢ units of type *t*ᵢ". There are four payload lists
  (Import mapping), one per side group and one for the American paradrop.
- **Arrival edge.** One of four map edges, numbered 0 north, 1 east, 2 south, 3 west, belonging to
  a house. Its paradrop planes enter from it (claim 4), and S-0501 claim 34's *retreat* leaves by
  it (S-0501 claim 34 calls it the house's starting map edge).
- **Local point.** A point (*a*, *b*) of integers in the local area's own frame: *a* counts cells
  across the screen from the local area's left side, *b* counts half-rows down from its top
  (Formula F2).
- **Pass counter.** A paradrop plane's count of passes it may still start without a successful
  drop. It starts at 5 (claims 6 to 8).
- **Drop attempt.** One attempt to release the first passenger of a paradrop plane (claim 9).
- **Drop point.** The point beside the plane at which a drop attempt is judged (Formula F4).
- **Falling** has the meaning of S-0107 (Terms); *fall rate* is S-0511's term.
- **Construction yard,** in this spec, has two senses, kept apart:
  - a *building factory*: a building whose type produces buildings (claim 5's trigger),
  - a *yard-flag building*: a building whose type has the construction-yard flag (claim 5's scan).
- **Water-role tile.** A cell whose tile is one of the theater's water-role tiles (S-0010 claim 5).
- **R(a, b).** One scenario-stream ranged draw over the inclusive integer range a … b,
  following S-0202 claim 4. A ranged draw can consume zero, one or several raw values.

## Behaviour

### Launch and planes

1. **Target cell.** A launch at a cell that is not a legal map cell sends nothing. When the target
   cell is a water-role tile, the target is moved once, before any plane is sent, by the nearby-cell
   search of S-0212 claim 25 with these settings: centre the target cell, locomotion class *foot*,
   movement zone *normal*, no reach region, never in deck mode, no height condition, no further
   test, no no-overlay condition, bridge cells allowed, and no reference cell (so the pick uses the
   launch tick's number). The search's result replaces the target only when there is one and it is
   not itself a water-role tile; otherwise the original target cell stays. This search takes no
   random draw.
   - Backing: `measurable` (scenario P1) for water adjustment; `unmeasured` for an illegal-cell
     launch, because no normal-input launch fixture bypassing target legality is established
   - Confidence: high

2. **Payload by side.** The payload list used depends on the launch kind and the launching
   house's side, by the side's position in the rules' side list (glossary *side*):
   - the American paradrop always uses the American payload list,
   - otherwise position 0 uses the first side group's list, position 2 the third side group's list,
     and any other position the second side group's list.

   For each entry *i* of the infantry list, in list order, one paradrop plane is sent (claim 3)
   carrying *n*ᵢ units of type *t*ᵢ. When the two lists of the payload have different lengths:
   - for the American list and the first and third side groups' lists, no plane is sent at all,
   - for the second side group's list, the original reads past the end of a shorter count list.
     Our engine sends planes for the entries that have a count and ignores the rest; the importer
     warns (documented departure, design). A count list longer than the infantry list has its extra
     counts ignored (the original does the same).

   Payload names are creating references of the infantry kind, not a filter against the
   infantry type list (S-0004 claims 2, 4, 6 and 13). An unknown ordinary name creates an
   infantry type of that name. A name otherwise used only by a vehicle or aircraft likewise
   creates a separate infantry type; it does not carry that vehicle or aircraft. Its entry
   sends one plane with its paired count of that infantry type, whose properties follow the
   ordinary type-reading rules. There is no fallback infantry type. A lack of suitable art
   or usable properties is not a reason to discard that payload entry at import.

   Empty items and reserved no-type names are omitted from the infantry reference list.
   The integer count list is independent: pairing uses the resulting compact infantry list,
   not original text positions. Thus omitting a middle no-type item shifts later infantry
   against the counts and can trigger the length rejection above. An unknown ordinary name
   is retained, so it causes no shift. S-0513 claim 43's spy-plane count uses this resolved
   list's length and its length-equality test, not the number of infantry names explicitly
   listed in the rules.

   When the rules define no paradrop plane type, no plane is sent.
   - Backing: `measurable` (scenario P2) for list choice, reference resolution, count pairing
     and one plane per retained entry; `unmeasured`
     for the short count list, which is a design rule
   - Confidence: high

3. **Sending one plane.** Each plane is a visiting aircraft, placed at the centre of its arrival
   cell, facing 0 (map north), at cruise height by S-0501 claim 1. It starts *paradrop approach*
   immediately, with the launch target and initially no destination. Its passengers leave in
   reverse creation order: the last paratrooper created is the first passenger released, as in
   S-0507 claim 3. Creation and arrival draw order is claim 16's.

   If placement fails, the plane is removed and no paratroopers are created for it. The next
   payload entry is still processed. An entry with a count of 0 sends a plane
   with no passengers: it flies the whole approach and retreats at its first overfly processing
   (claims 7 and 8). A negative count is treated as 0, with an
   importer warning (design: the original would create a practically unbounded number).
   - Backing: `measurable` (scenario P2, with the random-state recorder for the draw order),
     `unmeasured` for negative counts, a design rule, and for failed plane placement, for which
     no normal-input failure fixture is established
   - Confidence: high

4. **Arrival cell.** The plane's arrival cell lies on its house's arrival edge (claim 5). In the
   local area of size *L*w × *L*h (S-0009 claim 4), the candidates are local points (Formula F2).
   A candidate is acceptable when its cell is outside the usable area by S-0009's height-aware test.
   For north, draw *r* = R(1, *L*w). The candidate sequence is the north-edge points
   ((*i* + *r* − 1) mod *L*w, 0), for integer *i* from 0 through *L*w* − 1. The first acceptable point in this cyclic order is selected.
   For east, draw *r* = R(1, 2*L*h). The candidate sequence is (*L*w*, ((*i* + *r* − 1) mod 2*L*h)) for integer *i* from 0 through 2*L*h* − 1; the first acceptable point in that cyclic order is selected.
   For west, draw *r* = R(0, 2*L*h), and let *b*₀ = *r* − 1. The candidate sequence is (0, rem(*b*₀ + *i*, 2*L*h)) for integer *i* from 0 through 2*L*h* − 1, where `rem` has the dividend's sign. The first acceptable point in that sequence is selected; in particular, when *r* = 0 its first row is −1.
   For the south edge, the draw sequence has two members. The first is R(1, *L*w*), whose value does not affect the selected cell. For each column *a* in 0 … *L*w* − 1, define its retained candidate as the acceptable point (*a*, 2*L*h* + *j*) with the smallest *j* in 0 … 14; if the column has no acceptable point in that range, it contributes no candidate. Order the retained candidates by increasing *a*. Column 0 always contributes: its retained row is the one of 2*L*h* and 2*L*h* + 1 for which *b* + *L*y* is even. Therefore the candidate count *n* is at least 1. The second draw is R(0, *n* − 1), and its value selects that zero-based position in the ordered candidate list.
   - **No acceptable candidate** on the north, east or west edge: the arrival cell is the cell of
     local point (1, ⌊*L*w / 2⌋). On a local area with positive width and height this does not arise
     on the east and west edges: by F2 and S-0009 claim 4, every point of column *L*w, and every point
     (0, *b*) with *b* + *L*y even, is outside the usable area at any height. On the north edge, row 0
     is outside unless its cell lies at a negative height level below −*L*y.

   So a plane takes one draw for its arrival cell, or two on the south edge. These draws are ranged draws on the scenario stream (S-0202). No occupant,
   terrain or passability test is made: an aircraft may arrive over any cell.
   - Backing: `measurable` (scenario P3, with the random-state recorder) for edge candidates
     and draws; `unmeasured` for the no-acceptable-candidate fallback, because no skirmish
     map fixture exhausting the north candidates has been established
   - Confidence: high

5. **Arrival edge.** A house's arrival edge is decided as follows.
   - When the house's map section names an edge (Import mapping), that edge is used.
   - Otherwise its *computed edge* is used. It starts as north (0). It is computed again each time
     a building factory of the house is marked primary (S-0213, *primary*). The reference cell is
     the cell of that building's centre. Should that building not be primary at that moment, the house's buildings that are on the map are scanned instead: the
     primary yard-flag building, otherwise the last yard-flag building in the house's building
     order, otherwise the first other building. When even that finds no building, the edge keeps its
     value.
   - The new computed edge is the edge whose midpoint (Formula F3) is nearest to the reference
     cell, with distances from Formula F3; on equal distances the lower edge number wins.

   The edge is read when each plane is sent; a later change of the computed edge does not move a
   plane already sent.
   - Backing: `measurable` (scenario P3) for the computed edge and tie rule; `unmeasured` for
     the fallback scan, because no skirmish input reaching it has been established, and for the
     map-section edge on player houses, whose loading remains unresolved
   - Confidence: high (the formula and tie rule); medium (fallback scan); low (whether a player
     house reads a map-section edge)

### Approach, passes and exit

6. **Approach.** In *paradrop approach* a paradrop plane flies toward its target cell (flight:
   S-0501). At each processing:
   - with no target, it loses its destination and its next behaviour is *retreat* (claim 8),
   - with no destination, the target becomes its destination,
   - otherwise, when its planar distance to the target (S-0204 claim 6) is at most the paradrop
     radius (Import mapping), its next behaviour is *paradrop overfly* and its pass counter goes
     down by 1.

   Each processing ends with a delay of 3 ticks (no random draw).
   - Backing: `measurable` (scenario P4); `unmeasured` for the no-target case, because a normal
     launched plane targets a persistent cell and no normal-input way to remove it is established
   - Confidence: high

7. **Overfly.** In *paradrop overfly* the plane holds the run lock (S-0501) while it is inside the
   radius. At each processing:
   - with no target or no passenger left, it releases the run lock, loses its target and
     destination and its next behaviour is *retreat*,
   - outside the paradrop radius, it releases the run lock; with a pass counter above 0 its next
     behaviour is *paradrop approach* (another pass at the same target), and otherwise it loses its
     target and destination and its next behaviour is *retreat*,
   - inside the radius, when the cell of its own position is inside the usable area (height-aware
     test), it makes one drop attempt (claim 9); outside the usable area it makes none.

   Each processing ends with a delay of 5 ticks (no random draw). So over the target, drop attempts
   come every 5 ticks, each from the plane's position at that moment, and at most one paratrooper
   leaves per attempt. The destination stays the target cell throughout, so the plane flies over
   it, turns and comes back as S-0501's flight gives.
   - Backing: `measurable` (scenario P4); `unmeasured` for no target, for claim 6's reason
   - Confidence: high

8. **Giving up and leaving.** A successful drop resets the pass counter to 5. So a plane makes at
   most five consecutive passes with no successful drop; when it leaves the radius after the fifth,
   it *retreats* with the paratroopers it still carries. A plane in *paradrop overfly* with no passenger
   left retreats at its next processing. In *retreat* it flies to a cell on its arrival edge and is removed when it
   leaves the usable area (S-0501 claims 10 and 34). The exit destination uses the house's edge
   when that destination is chosen, not an edge remembered at launch. Paratroopers still on board
   leave play with the plane without a death, loss count, kill credit or experience award.
   - Backing: `measurable` (scenarios P3 and P5)
   - Confidence: high

### Drops

9. **A drop attempt.** A drop attempt takes the plane's first passenger and decides its side and
   its drop point (Formula F4): the plane's ammunition (glossary *ammunition*) goes down by 1, and
   the new value's parity picks the side, odd to the left of the plane's body facing and even to the
   right. A drop succeeds exactly when the following conditions all hold for cell *C* of the drop point and the standing position selected there: *C* gives the paratrooper entry outcome *clear* under S-0209, using the landing-entry rules of S-0208 claim 19 (initial reference level −1, deck objects and record on a high-bridge cell, and the bypass at terrain level −6 or lower); S-0208 claim 3 finds a standing position using the drop point as the desired point and the cell's ground occupancy record, including on a bridge cell; that position lies inside the usable area; a bridge cell has the deck-entry mark; and the cell meets S-0212 claim 25's nearby-cell qualification for the paratrooper's locomotion class and movement zone (the entire applicable occupancy record is empty and the cell is enterable by class, using the deck for a bridge cell). The entry-outcome refusal suppresses standing-position selection entirely. Once entry is clear, selection precedes the usable-area, deck-entry-mark and whole-record/class qualification refusals. Consequently the centre-class row's R(0, 3) draw is allocated as follows; every row assumes a desired point of that class, and later refusals assume earlier conditions passed.

   | Reason the attempt is refused | Standing-position draw |
   |---|---|
   | entry outcome is not clear, including an occupant or class restriction detected there | none |
   | selected standing position is outside the usable area | one R(0, 3) |
   | high-bridge cell lacks the deck-entry mark | one R(0, 3) |
   | final whole-record/class qualification fails | one R(0, 3) |

   A class restriction normally refuses the initial entry test and consumes no standing-position
   draw. If an entry bypass such as the negative-level case lets it reach final qualification,
   it does consume the centre-class draw. Other standing-position rows consume none. No
   standing-position result also refuses the drop, without reaching the later conditions.

   On success the paratrooper appears at the standing position, at the plane's own world height,
   facing 32768 (map south), falling (claim 12), with a parachute (claim 13). The parachute sound
   (Import mapping) plays at the plane's position. A paratrooper whose cell is a bridge cell is on
   the bridge deck. For an ordinary drop from above the deck, its standing-position mark is
   in the deck occupancy record throughout descent and after landing, not the ground record
   used to choose the position. Selection alone does not reserve a ground position. More
   generally the marking test uses the unit's actual world height: at or above the local
   terrain surface plus 416 leptons on a high-bridge cell it marks the deck, below that it
   marks ground. These are exact integer-lepton comparisons and addition, without rounding.
   Landing at deck height retains a deck mark.

   Release adds any mobile sensor coverage immediately at the release cell, without an
   adjacency-reveal test, under S-0516 claim 22. Descent does not continuously relocate
   that coverage.

   On failure the paratrooper stays in the plane as its first passenger, and the ammunition goes
   back up by 1, so the next attempt uses the same side. A failed attempt that reaches the
   standing-position table's centre-class row has still taken that row's draw.
   - Backing: `measurable` (scenarios P4 and P6) for ordinary above-deck releases and the
     listed refusal/draw cases. Release sensor effects use S-0516 claim 22's scenarios.
     `unmeasured` for actual-height marking below or exactly at
     the bridge-height boundary, because those scenarios do not establish a release at
     those heights
   - Confidence: high

10. **Consequences of the drop rules.** These follow from claim 9 and are stated for clarity:
    - Successive drop points alternate sides, approximately half a cell from the plane by F4.
      The released unit appears at the selected standing position, not necessarily that point.
    - An ordinary drop from above the landing surface occupies its destination cell's landing
      layer already, so another drop into that layer is refused until the cell is freed.
      Unusually low releases use claim 9's actual-height marking rule. Standing infantry, vehicles and
      buildings on that layer also refuse a drop. This does not limit the cell to one infantry
      unit after ordinary ground movement, which follows S-0209 claim 1.
    - Over water, cliffs and other cells its class cannot enter, nothing is dropped; the plane keeps
      its paratroopers and tries again 5 ticks later from wherever it is.
    - Over a bridge, a paratrooper lands on the deck when the deck is free.
    - The paratrooper does not move sideways while it falls, so it lands at the standing position.
    - Backing: `measurable` (scenario P6) for ordinary releases; the unusually low-release
      marking exception is `unmeasured` for claim 9's reason
    - Confidence: high

11. **A drop by a shot.** A shot of an aircraft that carries passengers is a drop attempt (S-0501
    claim 20), exactly as in claim 9, with no projectile. It has no usable-area condition of its
    own; attempts occur when the aircraft fires (S-0501 claim 20).

    | Shot effect | Refused drop | Successful drop |
    |---|---|---|
    | rearm countdown | unchanged | expired now |
    | ordinary full-delay or in-burst delay, including its random draw | none | none |
    | burst-counter advance | none | none |
    | weapon report, muzzle animation and spray draw | none | none |
    | projectile | none | none |
    | immediate ammunition change | none | minus one, as claim 9 |
    | pass counter | unchanged | reset to 5 |
    | parachute sound and applicable animation draws | none | claims 9 and 13 |

    The absence of a projectile is not an ordinary void shot under S-0301 claim 15: it neither
    reverses a successful release nor causes ordinary-shot effects. No ordinary delay is set
    and then cleared. This exception applies only while a first passenger exists.

    Attack-run bookkeeping is separate. A firing pass that reaches this attempt still makes
    a round pending under S-0501 claim 15, even if every drop in that pass is refused. That
    pending round is spent at the ordinary run boundary. Thus a successful drop spends its
    immediate round in addition to the run's later pending round; a refused drop can still
    belong to a run that spends that later round. The pass's flight/behaviour delay is not
    the rearm countdown and remains governed by S-0501 claims 13 and 14.
    - Backing: `measurable` (scenario P7)
    - Confidence: high

12. **Falling.** A dropped paratrooper falls by S-0511 claim 10 and Formula F3, starting at the
    plane's world height with the fall rate it already holds (0 for a fresh unit). While an
    animation is attached to it (its parachute), its rate follows the parachute recurrence of S-0511
    F3; with none attached, the no-parachute recurrence. It lands when its height above the surface
    (the deck on a bridge cell) is less than 1 lepton after a fall step: its height becomes 0 and
    falling ends. Landing then supplies S-0516 claim 22's cell-arrival update, including
    a same-cell update, with sensor-disc relocation and adjacency reveal before any
    remaining parachute is told to end (claim 13). There is no fall damage: a paratrooper
    is not hurt by landing. While it falls it
    shows the *parachute* sequence and, on landing, asks for *stand* (S-0107 claim 21 and its state
    table); what it may not do while falling is S-0214's.
    - Backing: `measurable` (scenario P8) for parachute attachment, expiry and landing,
      and S-0516 claim 22's scenarios for landing sensor effects; `unmeasured` for a non-parachute animation selecting the slow recurrence, because no
      fixture retaining such an attachment during descent has been established
    - Confidence: high

13. **The parachute.** At a successful drop, the parachute animation (Import mapping), when
    defined, is attached at the paratrooper's position, so it follows the unit down. It has no
    creation loop delay and uses repeat factor 1, not a fixed one-pass lifetime: the animation
    type's loop count determines its passes by S-0109 claim 7. It can therefore end before landing
    if its playback is finite and short enough. If it is still present on landing, its remaining
    passes become 0, making the current pass its last; normal end and follow-up behaviour is
    S-0109 claims 15 and 16. Its creation draws are S-0202 claim 10's animation draws, after any
    standing-position draw of the same drop.

    A genuinely absent parachute type does not refuse release, play a visible parachute,
    or take animation-creation random draws. For a fresh paratrooper released during its
    plane's turn, with no other attached animation and the default attached-animation
    fall-rate quirk, its release-tick fall update uses the parachute recurrence once.
    From the next tick it uses the no-parachute recurrence. With
    `quirks.paratrooper_any_animation_slow_fall` off, an absent type instead uses the
    no-parachute recurrence from its first update. There is no durable parachute attachment.
    Both recurrences and their number domains
    and rounding are S-0511 claim 10 and Formula F3. A null-type release therefore must not
    be equated with starting immediately with the no-parachute recurrence.

    A non-reserved unlisted animation name is different: it creates an animation type by
    S-0004 claims 2, 4, 6 and 13. Release then uses that type normally, including any
    creation draws, playback, attachment and expiry. It can acquire art without being
    explicitly listed. If no sprite resolves, invisible playback and the unsafe negative
    playback-limit case are S-0109 claim 25 and its missing-sprite rule, not an absent-type
    fallback. An absent or empty key in a higher rules layer retains the previous type.
    The absent-type case requires no type to be held, not merely omission from the map.
    - Backing: `measurable` (scenario P8)
    - Confidence: high for release and creating references; medium for the absent-type
      release-tick rate transition pending the discriminating P8 observation

### Deaths and losses

14. **Death in descent.** A falling paratrooper can be hit and killed like any infantry unit. How
    its death shows is S-0107 claim 17: a unit not marked "not human" is removed at once with no
    death sequence; the outcome for a "not human" type is S-0107's open question. A paratrooper
    removed this way no longer occupies its cell, so a later drop can use that cell.
    - Backing: `measurable` (scenario P9) for ordinary human-type infantry,
      `unmeasured` for the not-human-type outcome, which remains unresolved in S-0107 claim 17
    - Confidence: medium

15. **Loss of the plane.** A paradrop plane that is killed and starts to crash loses all its
    passengers, killed and credited to the attacker (S-0312 claim 16); a plane that explodes kills
    them by S-0312 claim 5. Paratroopers already dropped
    are not affected by what happens to the plane.
    - Backing: `measurable` (scenario P9)
    - Confidence: high

16. **Random draws of a paradrop, in observable stream order.** The scenario-stream observations have these consequences:
    - For each payload entry, in payload-list order, the plane-creation draw precedes that plane's arrival-cell draw or draws; any placement draws precede the creation draws for that plane's passengers. Thus, for two entries, the observed stream order is plane 1 creation, plane 1 arrival, plane 1 placement if any, plane 1 passengers, plane 2 creation, plane 2 arrival, plane 2 placement if any, plane 2 passengers.
    - A drop attempt whose standing-position choice reaches the centre-class row of S-0208 claim 3's table consumes one R(0, 3), including when the attempt fails. Other rows consume none.
    - A successful drop with a defined parachute type is followed by that animation's
      applicable creation draws (claim 13). An absent type takes none.

    Target adjustment (claim 1), arrival-edge computation (claim 5), and the processing delays of claims 6 and 7 do not advance the scenario stream. Flight draws remain those of S-0501 claim 37. These statements constrain the observable stream state and its advancement, not the order of internal operations.
    - Backing: `measurable` (scenarios P1, P2, P3, P4 and P8, with the random-state recorder)
    - Confidence: high

### Orders and presentation

17. **The released infantry's standing order.** A fresh paratrooper receives the ordinary
    placement fall-back to rest at release, not a special guard order at landing. Immediately
    after placement its current behaviour is *none*, with *guard* queued, or *area guard*
    queued when S-0215 claim 14's type/veterancy rule selects it. It has no target and no
    destination. Falling makes it not ready to start that queued behaviour (S-0215 claim 11).
    Landing does not issue, replace or clear an order, target or destination. The queued rest
    behaviour becomes current when ordinary readiness next permits, and then makes ordinary
    automatic searches by S-0215 claims 24 and 26 and S-0309. Thus nearby eligible enemies
    can be acquired without a player order; landing grants neither an immediate target nor
    an unconditional shot.

    Player move clicks while falling are refused because the unit is not movable (S-0214
    claims 7 to 9 and its movable definition). They are not saved as moves to execute after
    landing. This does not erase a pre-existing order on a non-fresh passenger: release uses
    the ordinary placement fall-back, which respects existing state by S-0215 claims 13 and 14.
    - Backing: `measurable` (scenario P10) for fresh payload infantry and player orders,
      `unmeasured` for retention combinations on a previously ordered passenger, because
      no fixture here establishes all of those prior states
    - Confidence: high

18. **Parachute drawing.** The parachute and an ordinary walking paratrooper are both in the
    ground render layer. Attachment is not a special instruction to paint behind the unit.
    Their relative order is S-0102 claims 5 and 6's ground ordering: the parachute adds its
    imported animation sort adjustment to the unit's position sum. A negative adjustment
    puts it behind the unit; zero puts it after the already-placed unit, in front where their
    opaque pixels overlap; a positive adjustment also puts it after the unit. Height does
    not change that comparison. Ordinary depth acceptance still applies (S-0102 claim 10).

    Palette and brightness are already specified by S-0103 claim 8, row 3, and S-0104 claim
    13: an ordinary untiled parachute uses the carried infantry's house-colour unit palette,
    not the ordinary animation palette. Its brightness is the unscaled ground light of the
    infantry's cell sampled at parachute creation, without extra unit light or airborne-height
    addition. S-0103 claim 8 owns the normal-light override, special palette precedence and
    tiled-copy exception. No additional palette rule is introduced here.
    - Backing: `measurable` (scenario P11) for sort, palette, light retention and the named
      light/palette controls; `unmeasured` for distinguishing exact creation-tick brightness
      sampling from sampling at first visible opening, because P11 establishes no separate
      opening event. Palette and light also use the owning claims' scenarios
    - Confidence: high for sort and palette; medium for creation-time brightness sampling

## Formulas

**F1. Drop cadence.** Over the target, attempts are 5 ticks apart: an attempt in tick *t* is
followed by the next processing in tick *t* + 5 (claim 7). The approach checks every 3 ticks
(claim 6). Tick arithmetic is exact integer addition. Real-time tick pacing is S-0201 claim 4.

**F2. Local point to cell.** With the local area's origin (*L*x, *L*y) and the map width *W*
(S-0009), a local point (*a*, *b*) has *X* = *a* + *L*x and *Y* = *b* + *L*y, and its cell is

> *x* = ⌊(*Y* + 1) / 2⌋ + *X*, *y* = *W* + ⌊*Y* / 2⌋ − *X*

with floor division (toward −∞), in integers. Equivalently, in S-0009's terms, *s* = *x* + *y* =
*W* + *Y* and *d* = *x* − *y* = 2*X* − *W* + (*Y* mod 2). The centre of cell (*x*, *y*) is
(256*x* + 128, 256*y* + 128) leptons. All values are exact integers.

**F3. Arrival-edge distances.** The four edge midpoints are the local points

| Edge | Local point |
|---|---|
| 0 north | (⌊*L*w / 2⌋, 1) |
| 1 east | (*L*w, *L*h) |
| 2 south | (⌊*L*w / 2⌋, 2*L*h) |
| 3 west | (0, *L*h) |

each turned into a cell (*x*ₑ, *y*ₑ) by F2. With the reference cell (*x*, *y*), the distance is

> *D*ₑ = trunc(*r*((*x* − *x*ₑ)² + (*y* − *y*ₑ)²)), in cells,

where *r* is S-0204's table root (claim 5, `quirks.table_root_distance`) of the exact integer sum
of squares. The coordinate differences are integers; their squares and sum are evaluated in
floating point (double precision), exact for legal map cells. Trunc is truncation toward zero
of the root's floating-point value. The edge is
the one with the smallest *D*ₑ, ties to the smaller edge number.

**F4. Drop point.** Let (*x*, *y*) be the plane's position in leptons and *f* its body facing.
The side facing is

> *f*′ = *f* − 16383 when the new ammunition is odd, *f*′ = *f* + 16383 when it is even,

wrapped to a signed 16-bit value (−32768 … 32767). The drop point is the plane's position moved
128 leptons in direction *f*′, by S-0501 Formula F3's horizontal step with step length *s* = 128:

> θ = (*f*′ − 16383) × *K*, *x*′ = trunc(*x* + cos_T(θ) × 128), *y*′ = trunc(*y* − sin_T(θ) × 128)

with *K*, cos_T and sin_T as in S-0501 F3 (table trigonometry). θ and the products are computed in
double precision, and trunc truncates toward zero. For a plane facing map north the odd side is
map west (the plane's left). The drop point's height is not used for the cell; the paratrooper
appears at the plane's height (claim 9).

**F5. Fall.** S-0511 Formula F3, with the parachute recurrence while an animation is attached
(claim 12).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Pass counter start and reset | 5 | passes | claims 6 to 8 |
| Approach processing delay | 3 | ticks | claim 6, no draw |
| Overfly processing delay | 5 | ticks | claim 7, no draw |
| Drop-point distance | 128 | leptons | F4, half a cell |
| Side turn of the drop direction | 16383 | facing units | F4 |
| Paradrop radius, absent-key default | 1024 | leptons | claims 6 and 7 |
| South scan depth | 15 | local rows | claim 4, rows 2*L*h to 2*L*h + 14 |
| Initial facing of a plane | 0 | facing units | claim 3, map north |
| Facing of a dropped paratrooper | 32768 | facing units | claim 9, map south |
| Landing threshold | 1 | lepton | claim 12, height below 1 lands |
| Initial computed arrival edge | 0 | edge number | claim 5, north |

## State and ordering

**Paradrop plane behaviours.** Within each behaviour, the first applicable row governs.

| State | Event | Result |
|---|---|---|
| *paradrop approach* | no target | destination cleared; next *retreat*; 3 ticks |
| *paradrop approach* | no destination | target becomes destination; 3 ticks |
| *paradrop approach* | planar distance ≤ radius | next *paradrop overfly*; pass counter − 1; 3 ticks |
| *paradrop approach* | otherwise | keeps flying; 3 ticks |
| *paradrop overfly* | no target, or no passenger | run lock off; target and destination cleared; next *retreat*; 5 ticks |
| *paradrop overfly* | distance > radius, pass counter > 0 | run lock off; next *paradrop approach*; 5 ticks |
| *paradrop overfly* | distance > radius, pass counter = 0 | run lock off; target and destination cleared; next *retreat*; 5 ticks |
| *paradrop overfly* | distance ≤ radius, plane's cell in the usable area | run lock on; one drop attempt; 5 ticks |
| *paradrop overfly* | distance ≤ radius, plane's cell outside the usable area | run lock on; 5 ticks |
| any | drop attempt succeeds | pass counter becomes 5 |
| *retreat* | leaves the usable area | removed with any passengers (S-0501 claims 10 and 34) |

How a queued behaviour becomes current is S-0501's (claim 31's three points and T2).

**Paratrooper.**

| State | Event | Result |
|---|---|---|
| passenger | drop attempt succeeds | falling at the plane's height; parachute as claim 13; placement rest as claim 17 |
| passenger | drop attempt fails | stays first passenger |
| passenger | plane starts to crash | dies, credited to the attacker |
| passenger | plane removed by leaving | leaves play without a death or loss count (claim 8) |
| falling | height above the surface < 1 after a fall step | lands; any remaining parachute stops looping; asks for *stand* |
| falling | killed | S-0107 claim 17 |

**Ordering consequences.**
- Planes of one launch are sent in payload-list order; each plane's arrival draw comes after its
  own creation draw and before its paratroopers' creation draws, so a payload of two entries draws
  plane 1, arrival 1, troops 1, plane 2, arrival 2, troops 2.
- Paratroopers leave in reverse creation order, one per successful attempt (claim 3). A failed
  attempt keeps the same passenger first, so it does not reorder later successful releases.

## Edge cases and quirks

- **Arrival fallback coordinate** (`quirks.paradrop_fallback_point`, default on: use the original fallback local point (1, ⌊*L*w* / 2⌋), whose coordinates are reversed from the north midpoint; off: use (⌊*L*w* / 2⌋, 1)). It is rarely used on positive-size local areas.
- **West edge start row** (`quirks.paradrop_west_edge_start_row`, default on): the west scan's
  first candidate is local row *r* − 1, so when *r* = 0 it is row −1 (claim 4). With the switch
  off, the first candidate is row *r* modulo 2*L*h; subsequent candidates continue in cyclic
  order modulo 2*L*h. The draw range and number of draws are unchanged.
- **The falling mark of a failed drop.** In the original, a drop attempt that fails after the usable-area
  test leaves the passenger marked as falling while it waits in the plane. This has no observable
  effect; our engine does not mark a passenger that stays aboard.
- **Attached-animation fall rate** (`quirks.paratrooper_any_animation_slow_fall`, default on: use the parachute recurrence while any animation is attached to the falling object; off: use it only while the parachute animation is attached).
- **No paradrop fall damage.** The falling-damage multiplier is not applied to paradrops
  (claim 12, Import mapping); magnetic-lift impact uses it (S-0607 claim 18 and F2).

## Import mapping

| Section | Key | Feeds | Parse form | Absent |
|---|---|---|---|---|
| aircraft type list | the original's fixed paradrop-plane type name (not an INI key) | paradrop plane type (claims 2, 3) | the aircraft type with this name (S-0004 claim 4) | no such type: no plane is sent |
| `[General]` | `AllyParaDropInf`, `AllyParaDropNum` | payload of side position 0 | list of infantry type IDs; list of integers | empty lists: no plane |
| `[General]` | `SovParaDropInf`, `SovParaDropNum` | payload of every side position other than 0 and 2 | as above | as above |
| `[General]` | `YuriParaDropInf`, `YuriParaDropNum` | payload of side position 2 | as above | as above |
| `[General]` | `AmerParaDropInf`, `AmerParaDropNum` | payload of the American paradrop | as above | as above |
| `[General]` | `ParadropRadius` | paradrop radius (claims 6, 7) | integer, leptons | 1024 |
| `[General]` | `Parachute` | parachute animation type (claim 13) | creating animation reference, case-insensitive; ordinary unlisted names create a type; `none` and `<none>` clear it (S-0004 claims 2 and 13) | initially none; omitted or empty layered value retains the held type |
| `[General]`, payload infantry lists above | `AllyParaDropInf`, `SovParaDropInf`, `YuriParaDropInf`, `AmerParaDropInf` | claim 2's compact infantry references | creating infantry reference list, 127-byte text limit, S-0004 claim 13; empty items and `none`/`<none>` omitted; other names retained or created, even names of other type kinds | omitted or empty value retains the held list, initially empty |
| parachute animation's art section | `YSortAdjust` | relative ground-layer order (claim 18) | integer, leptons; S-0102 claim 5 | 0 |
| parachute animation's art section | `UseNormalLight`, `Tiled`, `AltPalette` | claim 18's light and palette exceptions | booleans, S-0103 claim 8 | no (false) for each of the three keys |
| infantry type and veterancy lists | `DefaultToGuardArea`, `VeteranAbilities`, `EliteAbilities` | release-time resting behaviour (claim 17) | boolean and ability lists, respectively; S-0215 claim 14 and S-0311 claim 3 own their full mapping | no and empty lists, respectively |
| aircraft weapon | `ROF` | ordinary rearm bypassed by claim 11 | integer ticks, S-0301 Import mapping | 0 |
| aircraft weapon | `Burst` | ordinary burst advancement bypassed by claim 11 | integer shot count, S-0301 Import mapping | 1 |
| aircraft weapon | `Report` | weapon report bypassed by claim 11 | sound-reference list, S-0701 claim 21 and Import mapping | held list, initially empty |
| aircraft weapon | `Anim` | muzzle animation bypassed by claim 11 | creating animation-reference list, S-0004 claims 2 and 13 and Import mapping | held list, initially empty |
| `[AudioVisual]` | `ChuteSound` | parachute sound (claim 9) | sound ID | none: no sound |
| `[General]` | `ParachuteMaxFallRate`, `NoParachuteMaxFallRate` | fall-rate floors (S-0511 Import mapping) | integer, leptons per update | S-0511's |
| `[CombatDamage]` | `FallingDamageMultiplier` | not applied to paradrops; magnetic-lift impact uses it (S-0607 claim 18 and F2) | see S-0607 Import mapping | see S-0607 Import mapping |
| aircraft type | `Ammo` | the plane's starting ammunition, whose parity picks the first drop side (claim 9, F4) | integer | −1 (S-0301 claim 10 and its Import mapping) |
| map house section | `Edge` | the house's named arrival edge (claim 5) | one of `North`, `East`, `South`, `West`, `Air`, compared case-insensitively; `Air` and any other word leave the edge unnamed | unnamed |
| superweapon type | `Type` | which launch kind (paradrop or American paradrop) is delivered; dispatch: S-0513 claim 42 | word | no kind |

The superweapon kind mapping is given in S-0512's Import mapping. Charging and firing follow
S-0512 claims 5 and 8; targeting and dispatch follow S-0513 claims 1 and 42. The original fixes
the paradrop-plane type name; the user's rules must define an aircraft type with that name
for paradrops to work. Whether a skirmish player's house reads the map's house section for `Edge`
is an open question; a house without a named edge uses its computed edge (claim 5).

## Measurement scenarios

Common setup: the glossary's measurement setup, stock data as the baseline, no engine add-on and no mods (the named test files of a scenario excepted), skirmish, a small purpose-made map, speed index 1, bench at 800×600. Use one bystander computer
player isolated from the observed area, with the glossary's void rule. Run each case at three
recorded seeds and record launch ticks. Give the launching player access to the paradrop and
wait for readiness. Named test-map overrides below are explicit exceptions to the stock baseline,
not engine changes. Use screen capture and a proposed read-only recorder for ticks, positions,
heights, passenger creation order and random state where needed; any recorder extension is a
proposal, not a change to the harness.

- **P1 Target over water (claim 1).** Launch at a water cell a few cells from a shore; repeat at
  different launch ticks. Record the cell the planes fly to, as shown by their flight and
  the drop locations. Expected: a nearby land cell, chosen among the first ring of
  land cells by the launch tick's number; no paratrooper lands in water. Falsified by a drop
  attempt pattern centred on the water cell when the search predicts a different cell. Also
  test water farther than the search limit from enterable land: expect the original target to
  remain. A different retained target or a random draw from adjustment falsifies claim 1.
- **P2 Payload and creation order (claims 2, 3, 16).** Launch the paradrop as a house of side
  position 0, 1 and 2, and the American paradrop. Record the number of planes and each plane's
  paratrooper count and types (from the units that land) and the draw count around the launch.
  Expected: one plane per payload entry, with the entry's count of the entry's type; draws as in
  claim 16. Record initial plane position, facing, height, target, destination and behaviour
  before its first flight update: expect the arrival-cell centre, map north, cruise height,
  the launch target, no destination and *paradrop approach*. Any difference falsifies claim 3.
  Record passenger identity by creation event: releases must be last-created first,
  including after a refused drop. Any other payload, draw order or release order falsifies the
  claims. Named test-map variants give a country side position other than 0, 1 or 2 and give
  distinct payload lists: expect the second side group's payload. Also launch the American
  paradrop from each side: expect the American payload regardless of side. A different list
  falsifies claim 2. Mismatched lists for each checked payload give no planes.
  Excess counts in the second side group's list are ignored; count 0 gives an empty plane that
  approaches and then retreats; a missing plane type sends none. A contrary result falsifies
  the corresponding rule. Do not run the original with negative counts or a short second-group
  count list; those safe replacements are design rules, not oracle predictions.
  Reference variants: supply three infantry names and counts 1, 2 and 3. Replace the middle
  name first with an unlisted ordinary name supplied with a valid infantry rules section and
  art, then with a name listed only as a vehicle or aircraft but also supplied with usable
  infantry properties. Expect three planes carrying 1, 2 and 3 infantry, with the middle
  passengers of the separately created infantry type, not the other object kind. A missing
  middle plane or shifted third count falsifies claim 2. Replace the middle item with each
  reserved no-type spelling, then an empty item: expect a compact two-type list and length
  rejection for checked payloads. With a matching two-count list, expect the second retained
  type to use the second count. Repeat spy-plane launches: S-0513 claim 43 predicts three
  planes for the unlisted-name variants and none for the mismatched compact list. Record
  imported type identities and passenger counts, not just visible sprites.
- **P3 Arrival edge and cell (claims 4, 5, 16).** Place the construction yard near each of the four
  edges in turn (four maps or four starts). Launch; record the cell where the plane first appears
  and the draws taken. Expected: the plane appears just outside the usable area on the edge nearest
  the yard by F3, one draw (two for the south edge) after the plane's creation draw; the position
  along the edge follows claim 4 from the recorded draw. Repeat with a second yard made primary on
  another side: the edge follows the new primary building factory. Use a midpoint-tie placement
  to test the lower-edge-number rule, and record launch before any yard exists to test initial
  north. A different cell, edge, tie result or stream advancement falsifies claims 4, 5 or 16.
  Named map variants give an explicit edge, then change primary yards between launch and retreat:
  arrival stays fixed but a newly chosen retreat destination uses the then-current edge. Whether
  a skirmish player reads the explicit map edge remains an open question.
  West-edge variant: use an even local top coordinate and record both row −1 and row 0's
  candidate cells and acceptability. Make up to 200 launches, retaining the draws and candidates,
  include an *r* = 0 case if one occurs. Expected with the default quirk: candidates begin
  at −1 and continue through 2*L*h − 2, with claim 4's first acceptable point selected.
  A different arrival from that prediction falsifies the original rule. If no *r* = 0
  occurs, report that subcase inconclusive rather than confirmed. On our engine only,
  replay the same draw with the switch off: expect the cyclic sequence starting at row 0.
  Repeat the ordinary edge tests with obstacles and occupants beneath the predicted arrival
  cells: the cell and draw count must remain unchanged; a change falsifies claim 4's
  absence of occupancy and passability tests. The no-candidate fallback remains unmeasured.
- **P4 Passes and cadence (claims 6, 7, 9).** Launch at an open land target with one plane of
  several paratroopers. Log per tick the plane's position, behaviour and each paratrooper's
  appearance tick and cell. Expected: approach checks 3 ticks apart, overfly checks and attempts
  5 ticks apart, radius transitions as in claims 6 and 7. Predict the drop point from F4 and the
  standing position from claim 9: the unit need not appear exactly 128 leptons from the plane.
  Successive successful attempts alternate sides. A different cadence, transition distance,
  side or standing position falsifies the corresponding claim. Include a refused attempt:
  the next attempt must retain its passenger and side. Compare its random-state change with
  whether the standing-position table reached its centre-class row. Expect one R(0, 3)
  exactly when that row is reached, even on refusal; a different draw count falsifies claim 9.
  Record ammunition and the released unit's initial facing and height: expect one ammunition
  decrement per success, no net decrement on failure, map south and the plane's world height.
  Record the parachute sound's playback position: expect the plane's position on success and
  no such sound on refusal. A difference falsifies claim 9. Compare random state across
  approach and overfly checks without other draw-producing events: expect no processing-delay
  draws; a draw attributable to either delay falsifies claims 6, 7 and 16. Observe the initial
  approach without a destination: its first check assigns the target as destination rather
  than immediately entering overfly. A launch near the usable-area boundary tests that an
  in-radius plane outside it makes no attempt. A contrary transition or outside-area attempt
  falsifies claims 6 or 7. The targetless cases remain unmeasured: this setup cannot delete a
  target cell.
- **P5 Five passes then leave (claim 8).** Launch at a one-cell land islet (not a water-role
  tile, so claim 1 does not move the target) with a unit standing on it, whose surroundings out
  to the radius plus 128 leptons are water. Count the plane's passes over the
  target. Expected: exactly five passes, then *retreat* and removal outside the usable area.
  No paratrooper appears. More or fewer passes or retained off-map planes falsify the claims.
  A second case allows one successful drop after several failed passes, then blocks the target:
  expect five further unsuccessful passes, testing reset rather than just initial budget.
  Record passenger loss counts, kill credits and enemy experience across removal: expect no
  increase. An increase attributable to the departing passengers falsifies claim 8.
- **P6 Blocked and bridge cells (claims 9, 10).** Target a strip with a bridge, a row of single
  infantry units on every other cell, and a cliff. Record which cells receive paratroopers and their
  layer. Expected: no drop into a cell already holding any unit, into a cliff cell, or into water,
  a drop on a free bridge deck lands on the deck. Named test-map variants place terrain at
  level −6 and just above it, vary ground occupancy beneath a free bridge deck, and put the
  desired standing position at the usable-area boundary. Log the drop point, entry outcome,
  standing position, deck-entry mark and whole-record occupancy. With centre-class desired
  points, isolate each claim-9 refusal and record random state: a non-clear entry consumes
  no standing-position draw; usable-area, missing-deck-mark and final-qualification refusals
  consume one R(0, 3) after a clear entry. In particular compare ordinary class refusal with
  the same class restriction under the negative-level entry bypass. A different stream
  advancement falsifies claim 9's conditional allocation. Observe both occupancy records
  from a successful above-deck release until after landing: the chosen mark must be in deck
  only, with no newly reserved ground position. Any ground-only or duplicate ground mark
  attributable to that release falsifies the mark rule. Predict each success or
  refusal with claim 9 and the cited entry and standing-position rules; any disagreement
  falsifies claim 9, including a release outside the usable area. A release on a refused landing layer, a second
  release into a layer already occupied by a falling paratrooper, or a ground landing on the
  bridge falsifies the rules. Record horizontal position throughout descent: drift before
  landing falsifies claim 10. Move landed troops away and repeat to distinguish occupancy from
  a permanent one-unit-per-cell restriction.
- **P7 Drop by a shot (claim 11).** With a test aircraft type that is armed and carries passengers
  (named test-map override), order it to attack while loaded. Use one free-land run and one
  water/occupied-cell refusal run, with enough passengers that the observed shots all still
  have a first passenger. Record each attempt tick, passenger count, rearm countdown, burst
  counter, ammunition, pending round, pass counter, projectile and report/muzzle effects,
  plus both random streams. Expect claim 11's table: no ordinary delay, delay draw, spray
  draw, burst advance, projectile or weapon report/muzzle animation; failure retains rearm
  countdown, success expires it and resets passes to 5. Separate parachute sound and
  animation draws from weapon effects. Observe the following run boundary: one pending
  round is spent even for an entirely refused run, in addition to each successful drop's
  immediate round. Compare a no-passenger aircraft as the ordinary-shot control. Any
  contrary counter change, draw, effect or missing pending-round spending falsifies claim 11.
  Do not conflate the attack behaviour's processing delay with the rearm countdown.
- **P8 Fall (claims 12, 13).** Record a paratrooper's height per tick from its drop to its landing.
  Expected: S-0511 F3's parachute recurrence while an animation is attached, starting at the
  plane's world height; landing below 1 lepton above the landing surface, with no health loss.
  Record the infantry sequence while falling and on landing: expect *parachute*, then a request
  for *stand*. A different sequence request falsifies claim 12. Record the attached animation's
  lifetime and passes. Named test-map variants use a finite,
  short-lived parachute and an endless one, without follow-up effects, and omit the parachute.
  The finite one can finish before landing, after which the no-parachute recurrence applies.
  The endless one stops looping on landing. In fresh program runs, a named test rules file
  removes the parachute reference from every applied layer, then a comparison file explicitly
  clears it with a reserved no-type value. Expect successful release and no animation draws,
  one release-tick update using the parachute recurrence, then the no-parachute recurrence
  from the next tick under the default attached-animation quirk. On our engine only, repeat
  with that quirk off: expect no-parachute recurrence from the first update. Use distinct
  test fall-rate floors, including a parachute floor of 0
  leptons per update and a negative no-parachute floor, to distinguish this transient from
  starting immediately without an attachment. Log heights and rates from the release tick,
  not only the first displayed image. Either an immediate no-parachute first rate update or
  a lasting slow rate falsifies claim 13. Omission only in a map override is a retention
  control, not the absent-type case.
  Another named test file uses an unlisted ordinary animation name with valid supplied art:
  expect ordinary attachment, draws and playback, not the no-type result. A spriteless
  variant supplies explicit non-negative playback limits and tests S-0109 claim 25's invisible
  expiry; do not exercise its known unsafe negative-limit case on the oracle. Rejection of
  either unlisted name merely because it was not listed falsifies the creating-reference rule.
  A rate, landing tick, pass count or health change differing from the defined-animation
  predictions falsifies claims 12 or 13. Check animation-creation draws against S-0202 claim 10:
  they must follow any standing-position draw of that attempt. A different count or ordering
  falsifies claims 13 and 16. The non-parachute-attachment subcase remains unmeasured.
- **P9 Deaths (claims 14, 15).** Shoot down a loaded paradrop plane with anti-air units; record the
  attacker's kill count and experience. Use named aircraft-type test variants selecting a crash
  and an immediate explosion, and record passenger disappearance against crash-start or
  explosion ticks, respectively. Expected: every passenger dies at that event, credited to
  the attacker; passengers surviving until later crash impact falsify the crash-start rule.
  Kill ordinary human-type falling paratroopers: they disappear at once with no death sequence,
  and the cell becomes available for another drop. Kill the plane after several releases:
  already released units survive unless separately hit. A missing passenger credit, retained
  ordinary falling corpse, blocked vacated cell or remote death of a released unit falsifies
  claims 14 or 15. The not-human-type exception retains S-0107 claim 17's unresolved outcome.

- **P10 Rest and orders (claim 17).** Use the common setup with fresh human-owned payload
  infantry and an isolated enemy within the predicted landing cell's weapon range. Record
  current and queued behaviours, target and destination from release until 60 ticks after
  landing. Expect current *none*, queued ordinary rest and no target/destination at release,
  followed by the readiness-dependent start of guard or area guard and ordinary acquisition
  of the eligible enemy. Repeat with no enemy, and a named type override selecting area guard.
  With scripted input, select a falling paratrooper and click a distinct free move cell before
  landing. Record the order application and post-landing destination. Expect refusal and no
  deferred move. A different initial order, landing-generated guard replacement, accepted
  falling move, or a move emerging after landing without another order falsifies claim 17.
  Guard-acquisition latency is predicted from S-0215 and S-0309, not fixed to the landing tick.
- **P11 Parachute overlap and colour (claim 18).** Use the common setup and a named test art
  file whose parachute contains opaque pixels overlapping the infantry and remap-range pixels.
  Test animation sort adjustments below, at and above 0 in three runs each. Screenshot
  mid-descent and record both keys. Expect the negative case behind, zero and positive in
  front of the infantry wherever depth accepts both. Use two house colours: an ordinary
  untiled parachute must show its carried infantry's colour and unit-palette colours, not
  animation-palette colours. Change ambient light after release and screenshot before landing:
  under S-0103 claim 8's original setting the parachute retains its creation brightness.
  Add normal-light, alternative-palette and tiled-copy controls using that claim's predicted
  precedence and exceptions. For each of those three flags, compare a new test animation
  type whose art omits the flag with one explicitly setting it false: expect identical
  results. A difference falsifies the corresponding absent-key default in Import mapping.
  Exact creation-tick versus first-visible-opening sampling remains unmeasured. Wrong
  overlap order, remap colour, palette or brightness falsifies claim 18 or its named owner.
  These are named test-art exceptions, not claims about stock art's overlap pixels.

## Open questions

1. Whether a skirmish player's house can receive a named arrival edge from the map (Import
   mapping, claim 5).
2. Oracle confirmation of claim 13's transient absent-type fall-rate change is wanted (P8).
   Release and the distinction between absent and unlisted names are settled; the rate
   transition has medium confidence, not a completed measurement.
