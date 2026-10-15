---
id: S-0508
title: Garrisons (occupying buildings, ownership of civilian garrisons, evacuation and clearing)
topic: structures
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0508: Garrisons (occupying buildings, ownership of civilian garrisons, evacuation and clearing)

## Summary

How infantry occupy a building whose type can be garrisoned: which units may enter which building,
when, in what order and up to what number. It also covers how a civilian garrison building changes
hands with its occupants, and how occupants leave or are lost: an evacuation order, the building
being badly damaged, destroyed, sold or erased, a map trigger, or an assault. Skirmish only; network
games and campaigns are out of scope.

Neighbouring specs own the rest:
- the order and cursor that send an infantry unit to garrison (S-0214 row I8) and the *capture*
  behaviour while it walks there (S-0215 claim 28),
- the arrival moment (S-0208 claim 22) and scatter requests (S-0208 claims 11 to 13),
- what a garrisoned building's attack does: weapon rotation (S-0308 claim 13), muzzle index
  (S-0313 claim 6), rate factor (S-0301 claim 7), damage factor (S-0302 claim 2) and range
  (S-0307 claim 4),
- when a sale puts the occupants out (S-0408 claim 9) and how a destroyed building's contacts and
  survivors leave (S-0312 claims 17 and 18),
- the effects of a building ownership change (planned S-0509 Terms, claim 12),
- engineers and capture (planned S-0509), transports (S-0507) and special attacks (planned S-0519),
- the drawing of occupied buildings (S-0108 claim 2) and the announcements (S-0701 claim 28).

Cross-spec changes required at hand-off: S-0701 claim 28 must use claim 6's entering-unit owner
for garrisoned feedback and claim 8's building-turn abandonment gate, rather than a generic
local-building occupancy change. S-0516 must remove its garrison-protection hit rejection, which
conflicts with claim 16. S-0011 needs claim 13's two action mappings (Import mapping below).

## Terms

- **Garrisonable type.** A building type whose occupiable flag is set (Import mapping). S-0214 and
  planned S-0509 use the same word. A *garrisoned building* (glossary) is a building of such a type with at
  least one occupant.
- **Occupant.** An infantry unit inside a garrisonable building: S-0213's *garrison occupants*
  (Terms) are the occupants of one building. It is off the map and out of play
  (glossary, *in play*), keeps its own owner, and is neither destroyed nor removed. A building's
  occupants form an ordered list: the *first occupant* is the one that has been inside longest.
- **Occupant capacity.** A garrisonable type's maximum number of occupants (Import mapping).
- **Occupier type, assaulter type.** Infantry types marked as able to garrison, and as able to clear
  an enemy garrison, respectively (Import mapping). A type with both marks behaves as an occupier
  type only.
- **Civilian garrison building.** A garrisonable building whose type's tech level is exactly −1
  (S-0407 claim 5). Claims 7 to 9 apply only to these.
- **Civilian house.** The first house, in house order (S-0009 claim 19), whose country belongs to
  the civilian side (S-0414 claim 12). It is normally the neutral house.
- **Eviction.** Putting all occupants of a building out at once (claims 10 to 12). An eviction has
  two options: *hunting* (claim 11) and *inside fallback* (claim 12).
- **Exit ring.** For a building whose foundation rectangle has top-left cell (x₀, y₀) and size
  W × H (glossary, *foundation*): the cells with x₀ − 1 ≤ x ≤ x₀ + W and y₀ − 1 ≤ y ≤ y₀ + H that lie
  outside the rectangle.
- **Inside fallback cell.** The foundation rectangle's cell (x₀ + W − 1, y₀ + H − 1): the cell with
  the largest x and y.

## Behaviour

### Entering

1. **Occupation test.** An infantry unit *may garrison* a building when all of these hold:
   - the building's type is garrisonable,
   - the building is not being built up and is not being sold (S-0408),
   - the building's centre lies in the usable area (glossary),
   - the building is not being erased (glossary),
   - the unit's type is an occupier type,
   - the building belongs to the unit's own house, or to a passive house (glossary, S-0009 Terms),
   - the building has fewer occupants than its occupant capacity,
   - the building is not badly damaged (S-0108 Terms, compared as S-0108 states, with health above
     zero),
   - the unit is not mind-controlled (S-0214 Terms).

   Alliance plays no part: an allied player's infantry may not garrison your building, and you may
   not garrison an enemy's. A capacity of zero lets no unit in. This test is the "can garrison" test
   of S-0214 row I8 and the occupiable-building test that S-0215 names in its Terms.
   - Backing: `measurable` (scenarios G1 and G2) for owner, type, capacity, damage, sale and
     mind control. The build-up, unusable-centre and erasure gates are `unmeasured`: no data-only
     fixture is established that delivers an already ordered entrant during those states, rather
     than merely having the cursor reject its order.
   - Confidence: high (owner, capacity, badly damaged, sale, mind control); medium (built up,
     usable area, being erased)

2. **When an entry is attempted.** An infantry unit of an occupier type or an assaulter type tries
   to enter when it arrives at the end of a walking step (S-0208 claim 22) in a cell that holds a
   building, while its behaviour is *capture* and that building is its *destination*. Its target does
   not count, and *area guard* or *patrol* do not count: an occupier type arriving in those cases
   follows planned S-0509 claims 1 and 2 instead and does not garrison. When that general building-entry
   test is met, it can still give one general entry raise; that is not a garrison entry raise.
   Any building destination counts, garrisonable or not: one that fails the test is refused (claim 4), even a vehicle-like building (S-0214 Terms)
   that planned S-0509 claim 1 would otherwise hand to a hijack. Planned S-0509 has no outcome at that same arrival,
   so its general entry raise does not happen for it.
   - Backing: `measurable` (scenarios G1 and G3) for ordered capture arrivals. The target-only,
     area-guard, patrol and vehicle-like cases are `unmeasured`: no data-only fixture is established
     that gives an occupier that behaviour and building reference at arrival. Walking through a
     foundation alone is not such a fixture.
   - Confidence: high (capture behaviour and destination); low (the vehicle-like case, which stock
     orders do not give an occupier type)

3. **The garrison entry raise.** When the attempt of claim 2 passes the test of claim 1 (for an
   occupier type) or of claim 14 (for an assaulter type), the building's tag receives one entry
   raise with the entering unit as causing object (S-0524 claim 12). Effects of that raise can
   observe the entrant still outside and the occupants not yet cleared. A refused attempt raises
   nothing. So a garrison entry raises the tag exactly once.
   - Backing: `measurable` (scenarios G3 and G10)
   - Confidence: high

4. **Refusal.** When the attempt fails the test, the unit has its destination cleared and receives a
   scatter request whose threat point is the building's centre, forced and move-anywhere (S-0208
   claims 11 to 13), as an engineer refusal does (planned S-0509 claim 4). It stays on the map.
   - Backing: `measurable` (scenarios G1 and G2)
   - Confidence: high

5. **Going inside.** An occupier type that passes the test leaves the map at once, in the same tick,
   and is added at the end of the building's occupant list. It is not used up: it keeps its owner,
   health and experience, and comes out again by an eviction (claim 11). The building's owner does
   not change at this moment (claim 7).
   - Backing: `measurable` (scenarios G1, G5 and G9)
   - Confidence: high (leaves the map, end of the list, owner kept); medium (health and experience
     kept)

6. **Garrisoned announcement.** When an entry makes the occupant count 1, and the *entering unit's*
   owner is the local player (glossary), the "structure garrisoned" announcement and the
   garrisoned sound play, the sound at the entering unit's last position. The building's owner
   plays no part, so the announcement plays for a civilian-owned building. Later entries are
   silent. S-0701 claim 28 owns how announcements are queued.
   - Backing: `measurable` (scenario G4) for local entry feedback. The absence of feedback for
     a non-local entrant is `unmeasured`: no validated fixture here orders another house's unit
     into the building while the local player only watches.
   - Confidence: high

### Ownership of civilian garrison buildings

7. **Owner follows the first occupant.** At each of its own turns, a civilian garrison building that
   has at least one occupant and belongs to the civilian house changes owner to the first
   occupant's house. This is a building ownership change (planned S-0509 Terms). It happens at the
   building's turn, not at the entry: in the tick of the entry the building may still belong to
   the civilian house (scenario G5). A garrisonable building of any other tech level never changes
   owner through this spec.
   - Backing: `measurable` (scenario G5) for a single-house garrison and owner-change timing.
     Choosing between occupants of different houses is `unmeasured`: no validated data-only fixture
     gives two houses' units the required arrivals between consecutive building turns.
   - Confidence: high (who becomes owner); medium (which turn)

8. **Abandoned buildings return to the civilian house.** At each of its own turns, a civilian
   garrison building that has no occupants and does not belong to the civilian house changes owner to
   the civilian house. Abandonment feedback belongs to the owner losing the building: when that
   owner is the local player, in that same turn:
   - the abandoned sound plays, not placed at a position and not throttled,
   - a *garrison abandoned* radar event is requested at the building's top-left cell (S-0701 F7,
     S-0410 claim 11),
   - only when that request creates an event, the "structure abandoned" announcement plays.

   This applies however the building was emptied and however it came to belong to that house, so an
   empty civilian garrison building that the map gives to a player, or that a player captures,
   returns to the civilian house at its first turn.
   - Backing: `measurable` (scenarios G4 and G6)
   - Confidence: high (the change and the gate on the owner); medium (the radar event throttling the
     announcement)

9. **Forced evacuation when badly damaged.** At each of its own turns, a civilian garrison building
   that is badly damaged (as in claim 1) evicts its occupants (claim 10) without hunting and
   without inside fallback. A building emptied this way returns to the civilian house in that
   same turn under claim 8, with feedback for the owner it had before returning. While it stays badly damaged,
   claim 1 refuses every new occupant. A garrisonable building of any other tech level is never
   evacuated this way.
   - Backing: `measurable` (scenario G6)
   - Confidence: high

### Eviction

10. **Choosing the exit cell.** An eviction looks for one exit cell for all occupants. A cell
    *qualifies* when it gives the first occupant the infantry entry outcome *clear* (S-0209),
    judged as S-0208 claim 19's entry check is, with initial reference level −1. The exit cell is
    the qualifying cell of the exit ring with the highest rank. The ranking takes the four sides of
    the ring in this priority:

    | Priority | Side | Its cells | Within the side, first |
    |---|---|---|---|
    | 1 | x = x₀ + W | both of its corners included | larger y |
    | 2 | y = y₀ + H | x from x₀ − 1 to x₀ + W − 1 | larger x |
    | 3 | y = y₀ − 1 | x from x₀ to x₀ + W − 1 | smaller x |
    | 4 | x = x₀ − 1 | y from y₀ to y₀ + H − 1 | smaller y |

    The corner cell (x₀ − 1, y₀ − 1) has no rank and is never used (quirk, Edge cases). So the far
    corner (x₀ + W, y₀ + H) has the highest priority. Only the first occupant's view of a cell matters; the
    other occupants are not consulted.
    - Backing: `measurable` (scenarios G7 and G8) for ranking, skipped corner and first-occupant
      choice. The exact initial reference level is `unmeasured`: no fixture here separates −1
      from another reference level while keeping the other entry-check inputs identical.
    - Confidence: high (order and the skipped corner); medium (the entry outcome used)

11. **Putting occupants out.** With an exit cell, the occupants are put on the map in that one cell,
    from the last occupant to the first, within the same tick. Each faces facing 0 and takes a
    standing position in the cell as S-0208 claim 3 chooses for a unit placed at the cell's centre.
    An occupant that cannot be placed is removed (not destroyed). Each placed occupant then, before
    the next one is placed:
    - has its target cleared,
    - receives a scatter request whose threat point is the building's centre, forced and
      move-anywhere (S-0208 claims 11 to 13),
    - when the eviction is hunting, leaves its team and is given the *hunt* behaviour (S-0215) as
      its next behaviour.

    The building then has no occupants. Its muzzle index returns to 0 (S-0313 claim 6).
    - Backing: `measurable` (scenarios G7, G9 and G13) except team departure, which is
      `unmeasured`: no fixture is established that retains a team member inside a garrison and
      makes its post-eviction team membership observable. Removal on placement failure is also
      `unmeasured`: G7 tests crowding, but cannot guarantee a qualifying cell followed by a failed
      placement. The conditional rule does not assert that crowding necessarily causes failure.
    - Confidence: high (order of placement, target cleared, scatter); medium (standing positions,
      the removal of an occupant that cannot be placed); low (team and hunting)

12. **When no cell qualifies.** If no cell of the exit ring qualifies:
    - with inside fallback, the inside fallback cell is the exit cell and claim 11 applies (this is
      the inside position of S-0408 claim 9),
    - without it, every occupant is removed (not destroyed: no death, no kill, no survivors) and the
      building has no occupants (quirk, Edge cases).
    - Backing: `measurable` (scenario G8)
    - Confidence: high

13. **What causes an eviction.** Each of these events evicts the building's occupants with the
    options shown:

    | Event | Hunting | Inside fallback | Owner of the event's timing |
    |---|---|---|---|
    | The owner gives the building a deploy order (S-0213, the deploy rule) | no | no | S-0213 |
    | Badly damaged civilian garrison building, at its turn | no | no | claim 9 |
    | The building is destroyed | no | no | S-0312 claim 17 |
    | The building is sold | no | yes | S-0408 claim 9 |
    | A map trigger action that evicts a building's occupants | no | no | S-0011 |
    | A house-wide order that sends all its units hunting, including map action code 6 | yes | no | the action's firing tick |

    - Backing: `measurable` (scenarios G6, G7, G8 and G9 for the first four rows; G13 for the last two)
    - Confidence: high (first four rows); medium (trigger action); low (house-wide hunt order)

### Clearing and loss of occupants

14. **Assault.** An assaulter type is an engine feature that stock data does not use. For an
    infantry unit of an assaulter type and not an occupier type, the complete assault eligibility
    test is:
    - the building's type is garrisonable,
    - the building is neither being built up nor being sold,
    - the building's centre lies in the usable area,
    - the building is not being erased,
    - the building has at least one occupant,
    - the building's owner does not count the unit's house as allied (a one-way test from the
      building owner's side, including own-house alliance).

    No other occupation gate of claim 1 applies to an assault. In particular, the building need
    not be own-house or passive-owned, its capacity need not have a free place, badly damaged
    positive-health buildings are not excluded, and mind control of the assaulter does not exclude
    it. A type with both marks instead uses claim 1 alone and never assaults.

    On a passing attempt, the raise of claim 3 sees the occupants still inside. Every occupant
    is then removed, from the last to the first, each with the
    assaulter weapon's occupant animation at the muzzle point of its position in the list (S-0313
    Terms). The assaulter does not go inside: it has its destination cleared and scatters as in
    claim 4. The building, now empty, follows claim 8 at its next turn when it is a civilian
    garrison building.
    - Backing: `measurable` (scenario G10, named test data) for occupied/enemy/allied eligibility,
      capacity, damage, mind control, both-mark precedence and clearing. The one-way alliance
      distinction and the build-up, sale, unusable-centre and erasure gates are `unmeasured`: G10
      establishes neither an asymmetric-alliance arrival nor an already ordered assaulter arriving
      during those building states. The exact animation position and reverse removal order are
      also `unmeasured`: simultaneous identical animations do not identify which removed occupant
      produced which animation, and no discriminating fixture is established.
    - Confidence: medium (whether a removal counts as a kill: Open questions)

15. **Erasure.** When a building with occupants is erased (glossary, *being erased*), its occupants
    are removed with it, without death and without being put out.
    - Backing: `measurable` (scenario G11)
    - Confidence: medium

16. **Hits on the building do not pass through to occupants.** Weapon and warhead hits do not
    directly damage, remove or evict individual occupants. They can still cause the building-level
    evacuation, destruction or erasure of claims 9, 13 and 15. Occupants otherwise leave or are lost
    only through claims 10 to 14 and their causes. No hit is rejected because the target is garrisoned.
    - Backing: `measurable` (scenario G12) for its tested weapons. The universal absence of any
      other occupant-damage path is `unmeasured`: a finite weapon sample cannot exclude every
      combination of imported weapon and warhead fields.
    - Confidence: medium

17. **Occupants keep their own house.** A building ownership change, by any cause, leaves the
    occupants and their owners unchanged. After the change of claim 7, occupants of another house
    stay inside. An occupant's own house is the one that claims 7 and 11 use.
    - Backing: `measurable` (scenarios G5 and G10) for the single-house civilian ownership change
      and a trigger transfer from A to B while A's occupants remain inside. Mixed garrisons and
      other ownership-change causes are `unmeasured`: no validated fixture establishes mixed-house
      arrivals or another transfer cause while the building stays occupied.
    - Confidence: medium

## Formulas

> Exit ring size = 2 × (W + H) + 4 cells. Ranked distinct candidates = 2 × (W + H) + 3 cells
> (all but one corner). Exact integer arithmetic, no rounding; positive W and H in cells.
> Cell coordinates are integers (glossary, *foundation*).

> Inside fallback cell = (x₀ + W − 1, y₀ + H − 1). Integer cell coordinates.

> A garrisonable building accepts an occupier while occupants < capacity. Integer comparison.

Badly damaged is S-0108's comparison (binary64), with health above zero.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tech level of a civilian garrison building | −1 | level | claims 7 to 9 |
| Facing of a put-out occupant | 0 | facing unit | claim 11 |
| Reference level of the exit-cell check | −1 | height level | claim 10 |
| Absent occupant capacity | 0 | occupants | Import mapping |

## State and ordering

| State | Event | Result |
|---|---|---|
| occupier walking in *capture*, destination a garrisonable building | arrival in a cell of that building, test passed | one entry raise; unit inside at the end of the list; announcement when it is the first |
| same | arrival, test failed | no raise; destination cleared; scatter from the building centre |
| civilian garrison building, civilian house owner, occupants ≥ 1 | building's turn | owner becomes the first occupant's house |
| civilian garrison building, other owner, no occupants | building's turn | abandoned sound and radar event for the local player; owner becomes the civilian house |
| civilian garrison building, badly damaged, occupants ≥ 1 | building's turn | eviction, then (now empty) return to the civilian house, same turn |
| garrisoned building | deploy order, destruction, sale, trigger, hunt order | eviction with the options of claim 13 |
| garrisoned building | erasure completes | occupants removed with it |

Ordering as observed:
- A civilian garrison building emptied by forced evacuation returns to civilian ownership in
  that same turn and gives abandonment feedback to the owner losing it. A building still occupied
  can instead follow its first occupant's owner. Claims 7 and 8 cannot both change its owner in
  one turn.
- Within one eviction: the exit cell is chosen once; occupants are placed from the last to the
  first, and each gets its scatter request before the next one is placed. This can distinguish
  the standing positions assigned to identifiable occupants and the order of their scatter draws.
  It does not guarantee which unit completes a walking step first.
- At a sale, the occupants leave after the absorbed passengers and before the sale survivors
  (S-0408 claim 9).

## Edge cases and quirks

- **`quirks.garrison_exit_skips_corner`** (default: on). The exit search never tries the corner
  cell (x₀ − 1, y₀ − 1). With every other exit ring cell blocked and that corner free, the
  occupants are lost (or put inside, with inside fallback). Fixed behaviour: the corner is tried
  last, after the cells of claim 10.
- **`quirks.garrison_no_exit_removes`** (default: on). An eviction without inside fallback and with
  no qualifying exit cell removes every occupant (claim 12). With claim 9, a civilian garrison
  building surrounded by blockers loses its occupants when it becomes badly damaged and returns to
  the civilian house in the same turn. Fixed behaviour: the eviction does nothing; the occupants
  stay inside, and a later eviction tries again.
- **Mixed garrison.** Because the owner change of claim 7 waits for the building's turn, infantry of
  two houses can both enter a civilian-owned building in the same tick, each passing the passive
  owner condition. The building then goes to the first occupant's house, and the other house's
  occupants stay inside under their own owner (claim 17). No switch: this is the original's rule.
- **Building ownership change on a garrisoned building.** A capture or trigger that hands a
  garrisoned building to another house leaves the occupants as they are (claim 17).
- **No civilian house.** With stock data this cannot happen in a skirmish: the neutral house's
  country belongs to the civilian side, and S-0009 claim 28 never removes a passive house. With data
  in which no house of the game has a country of the civilian side, claim 8 hands the emptied
  building to no house, which the original does not handle (Questions for the project lead).
- **Both type marks.** An infantry type marked both occupier and assaulter is an occupier only: it
  never assaults.

## Import mapping

| Key (file, section) | Concept | Parse form | Absent default |
|---|---|---|---|
| `CanBeOccupied` (rules, building type) | garrisonable type | boolean (S-0003) | no |
| `MaxNumberOccupants` (rules, building type) | occupant capacity | integer | 0 |
| `TechLevel` (rules, building type) | civilian garrison building when exactly −1 | integer | 255 (S-0407 Import mapping) |
| `Occupier` (rules, infantry type) | occupier type | boolean | no |
| `Assaulter` (rules, infantry type) | assaulter type | boolean | no |
| `MultiplayPassive` (rules, country) | passive house (claim 1) | boolean | as S-0009 |
| `Side` (rules, country), side named `Civilian` | civilian side, civilian house | as S-0414 | as S-0414 |
| `ConditionRed` (rules, `[AudioVisual]`) | red ratio for badly damaged | as S-0108 | as S-0108 |
| `BuildingGarrisonedSound` (rules, `[AudioVisual]`) | garrisoned sound (claim 6) | sound name | none: no sound |
| `BuildingAbandonedSound` (rules, `[AudioVisual]`) | abandoned sound (claim 8); stock data does not set it | sound name | none: no sound |
| `[EVA_StructureGarrisoned]`, `[EVA_StructureAbandoned]` (EVA file) | the two announcements | as S-0701 | as S-0701 |
| `OccupantAnim` (rules, weapon) | assault animation (claim 14); also drawing of occupant fire | animation name | none |
| `MuzzleFlash`*n* (art, building) | muzzle points (claim 14) | as S-0313 | as S-0313 |

The initial absent defaults above have high confidence and `unmeasured` backing here: no fixture
removes the supplying stock layer and distinguishes an initial default from a retained earlier
value. Referenced defaults inherit their owning specs' backing.

For claim 13's map actions, code 111 selects the first eligible building attached to the firing
trigger and evicts without hunting or inside fallback. Code 6 reads its number slot as a house
value under S-0011 claim 23 and sends that house's units hunting, including its garrison evictions.
Other action encoding belongs to S-0011. These two action mappings share G13's measurable backing
and medium confidence. S-0011 needs corresponding action-table entries in the same hand-off.

## Measurement scenarios

All scenarios use the glossary's measurement setup: the stock executable and stock data, no
engine add-ons, director disabled and no mods except the named test maps below. Use game-speed
index 2, a tiny flat test area, the local player's house A, and one bystander computer player B
isolated from that area. Name B's test units explicitly where used; other interference voids the
run under the glossary's bystander rule. Keep both contenders alive throughout. "Civilian building"
means a pre-placed garrisonable building of tech level −1 owned by the neutral house.

The named test maps are `garrison-entry.map` (G1 to G4), `garrison-owner.map` (G5 and G6),
`garrison-exit.map` (G7 to G9), `garrison-assault.map` (G10), `garrison-loss.map` (G11 and G12) and
`garrison-trigger.map` (G13). Only the overrides explicitly described below differ from stock data.
Remove all test data after the runs. Use scripted mouse/keyboard input and a read-only recorder
for tick, behaviour, destination, target, owner, health, experience, occupant order and placement
observations; record video/audio for feedback. State observations are game quantities, not program
layouts. Unless stated otherwise, perform three fresh runs of every subcase. Each expected result
must hold in every valid run; any differing count, eligibility, owner, placement, state or tick
falsifies that part. No scenario here is an already completed measurement.

**G1. Type, owner and capacity (claims 1, 4 and 5).** Setup: a civilian building of imported capacity
*c* and *c* + 1 stock occupier infantry, plus a non-occupier infantry and a non-garrisonable building.
Procedure: order occupiers into the civilian building one at a time, recording each arrival. Test
orders to an own, an enemy and a passive-owned copy, and test the non-occupier and non-garrisonable
pairings. Expected: own and passive-owned eligible buildings accept up to *c* occupants; an enemy's
building and the two wrong-type pairings offer no garrison order. The last ordered unit at capacity
is refused, remains in play, loses its destination and receives forced move-anywhere scatter with
the building centre as threat point. For a guaranteed arrival refusal, send the last two toward the
last free place together and retain runs where both had valid orders before the first arrived.
Observable: offered orders, occupant counts, entry tick, destination and scatter destination checked
against S-0208 claims 11 to 13. First entry removes the unit from play in its arrival tick, not from
the game. A named map variant with occupant capacity zero offers no garrison order. Cursor results
alone do not establish how an otherwise unreachable failed arrival would behave.

**G2. Alliance, sale, damage and mind control (claims 1 and 4).** Setup: G1's map with a B-owned
non-civilian garrisonable building, a sellable A-owned garrisonable building, an A mind controller,
an A occupier and a B occupier. Use separate allied-B and enemy-B runs. Procedure: (a) try to send
A's occupier into B's building in both alliance variants; (b) order A in while its own building is eligible, then sell before
arrival. (c) Compare positive-health buildings just above and at the imported red-ratio comparison.
(d) A mind-controls B's occupier, then tries to send it into A's building. Use a named map
variant with a red ratio exactly representable at an integer health value for the equality case.
Observable: cursor eligibility, health, occupant count, arrival tick, destination and scatter.
Expected: A cannot garrison B's building in either alliance variant in (a); a unit whose existing
order survives until arrival
during sale is refused and scatters in (b). Only the above-ratio building is eligible in (c), and
(d) is ineligible. For damage-at-arrival refusal, order while healthy then damage before arrival.
An accepted arrival during sale or at the red ratio, or an eligible mind-controlled unit, falsifies
the respective gate. Do not count a cancelled order as an arrival refusal.

**G3. Entry raises (claims 2 and 3).** Setup: a civilian building carrying a repeating entered-by
trigger with a calibrated visible counter effect, as in S-0524 claim 12. Procedure: (a) order one
unit in; (b) arrange G1's capacity race so a validly ordered unit is refused at arrival. Observable:
counter increments, order tick, arrival tick and occupant count. At the counter effect, record
whether the entrant is still in play and the current occupant count. Expected: exactly one increment
at successful arrival while the entrant is still outside and before the count increases, none on
issuing the order and none on the refused arrival.
Any other count or an increment before arrival falsifies this. Area-guard and patrol arrivals are
not test parts: no fixture establishes their required building destination or target. Claim 2
states the conditional distinction: they never garrison, but a qualifying general entry can raise
under planned S-0509 claims 1 and 2.

**G4. Feedback (claims 6 and 8).** Setup: two nearby civilian buildings and A's infantry.
Procedure: enter with one
unit, then a second, wait for ownership, and evacuate. Repeat evacuation on the nearby building
inside, then outside, the suppression window and radius of S-0701's garrison-abandoned radar event.
Observable: audio, radar events and their cells, entry positions and building owners. Expected:
first local entry alone requests the garrisoned announcement and positional sound, even while the
building is still neutral. Local abandonment plays its non-positional sound independently of radar
suppression; the abandoned announcement requires a new radar event at the top-left cell. Attribute
this feedback to the local owner losing the building, and record it in the turn the owner returns
to the civilian house. Feedback gated on the new civilian owner, or delayed to a later turn,
falsifies that timing and owner rule.
Use a named map variant binding both feedback sounds to
distinct existing sound definitions, since stock data need not bind both; use the stock variant as
the control. An extra later-entry sound or an abandoned announcement without a new radar event
falsifies the applicable rule. Queue announcements far enough apart that
ordinary audio queuing cannot hide a request.

**G5. Ownership timing (claims 5, 7 and 17).** Setup: a civilian building and A's occupiers, with
recorded entry health and experience. Procedure: send one unit in, record owner at arrival and at
the next building turn, then send a second unit in and evacuate. Take event-time observations,
not just tick-boundary snapshots. Expected: owner remains neutral at entry, then changes to A at
the next building turn. That turn may be later in the same tick, not necessarily in the next tick.
Both occupants retain A as owner, and their health and experience, inside and after evacuation.
Falsified by ownership changing at entry, a change delayed past the next building turn, or an
occupant owner/state reset. A named non-civilian-tech-level copy must retain its original owner
despite an eligible entry. This does not validate the mixed-house consequence: obtaining two valid
arrivals between consecutive building turns remains a fixture problem, not a promised test part.

**G6. Damage evacuation and empty return (claims 8, 9 and 13).** Setup: an A-garrisoned civilian
building with free exits, a non-civilian-tech-level garrisonable copy, and an empty civilian
building assigned to A by the map. Procedure: damage each garrisoned building to just above, then
at or below, the red comparison while keeping health positive; record the first building turn after
each hit. Observable: occupant count, exit positions, owner, turn and feedback. Expected: the civilian
one stays occupied above the threshold, but evicts without hunting at the first turn at or below
it and returns to the civilian house in that same turn. The non-civilian copy does not evacuate or
change owner. The initially empty civilian building returns at its first turn with claim 8's
local feedback. Repeat with all exit-ring cells blocked: the damaged civilian one loses its
occupants without deaths and returns in that same turn. A delayed return, surviving garrison with
no exit, or damage evacuation of the non-civilian copy falsifies the corresponding rule.

**G7. Exit ranking and placement (claims 10 and 11).** Setup: a garrisoned W × H building on clear
level ground, with impassable blockers at chosen ring cells. Procedure: deploy to evacuate. In
separate subcases leave each ranked cell and all lower-ranked cells clear, blocking all higher-ranked
ones; this tests every rank of claim 10, not just the first two. Observable: first placement cell,
facing, standing positions and tick. Expected: the highest-ranked clear cell is used for all
occupants in that tick, facing 0, with positions chosen under S-0208 claim 3. Repeat at full imported
capacity. A named map variant changes one occupier's movement permissions so the two occupier types
have different entry outcomes under S-0209 on a chosen land-type test cell. Enter that type first,
then last in separate runs, with all higher-ranked alternatives blocked. The choice must follow the first occupant's entry
outcome alone. A lower-ranked cell, selection based on a later occupant, or different placement
facing falsifies that part. Record any excess-occupant losses without assuming crowding must cause
placement failure. The initial reference level is `unmeasured` by this flat-ground part; see the
backing qualification in claim 10.

**G8. Missing corner and blocked exits (claims 12 and 13).** Setup: G7, with every ring cell blocked
except (x₀ − 1, y₀ − 1); identify occupants individually. Procedure in separate runs: (a) deploy,
(b) sell; (c) destroy. Repeat with the last corner also blocked. Observable: identified occupants,
first placement cells, death effects, destroyed-event counters on occupant tags, and kill/loss
counts, separated from the building's own destruction and any building survivors. Expected: (a)
and (c) remove all occupants with no occupant death, kill or survivor; (b) uses the inside
fallback cell, subject to claim 11's placement success. Falsified by a corner exit, an occupant
remaining inside after an original-mode eviction, or an occupant death registration. Sale survivors
are not evidence that garrison occupants escaped. Also replay with each quirk disabled on our
engine: the corner becomes the last choice with the corner quirk off; a fully blocked eviction
without fallback leaves the occupants inside with the removal quirk off. These last replays test
the specified alternatives, not the original.

**G9. Reverse placement and retained state (claims 5 and 11).** Setup: three identifiable occupants
entered in a known order and with distinct health and experience. Give the infantry a target before
entry, and let the building attack a separate target. Procedure: evacuate by deploy. Observable:
placement order, standing positions, facing, target, health, experience, behaviour and scatter
requests before later automatic acquisition. A read-only random-state record may distinguish the
scatter draws in the order specified by S-0208 claims 11 to 13. Expected: reverse entry-order
placement in one chosen cell, each unit's target cleared and scatter requested before the next
placement, preserved health/experience, and no hunt requested. The muzzle index is 0 immediately
after eviction; re-garrison and check the first muzzle under S-0313 claim 6 as a visible control.
Any retained target, reset state or non-reversed assignment falsifies that part. Which unit first
finishes a later walking step is not a test of placement order.

**G10. Assault (claim 14).** Setup: named test-map overrides of a stock occupier infantry: variant
A has only the assaulter mark, variant B has both marks. Both also have the engineer capability,
so normal capture/repair orders can supply a building destination in *capture* behaviour. The
garrison outcome of claim 2 replaces the engineer outcome. Prepare each occupied test building
by sending A's ordinary occupiers into a civilian building, then use a timed change-owner trigger
(S-0011 claim 24), attached only to that building, to transfer it to B. Record that the occupants
remain inside under owner A and the building is B-owned before issuing the assault order.
An occupant loss or owner change during this transfer falsifies claim 17; do not silently discard
such a run as an assault setup failure. Use separate
mutually allied-B and enemy-B runs, and an empty enemy copy. Make the buildings capturable and
damage the allied copy enough to offer a repair order while preserving positive health and staying
above the red ratio. Keep the assaulter out of the occupants' firing range until its order is given.
Use an existing weapon with a visible occupant animation. Procedure: give each variant the normal
capture or repair order, verify *capture* behaviour and building destination, then record arrival.
Test the enemy occupied building, allied occupied building and empty enemy building separately.
Repeat the enemy occupied case with every imported occupant place filled. Observable: entered-by counter, occupant count, animation points, assaulter
position/destination/scatter, owner at the next building turn, kills and experience. Expected:
variant A clears only the occupied non-allied building, with one entry raise, remains outside and
scatters, and that civilian building returns to the civilian house at its next turn. At the
entered-by counter effect the occupants must still be inside, before clearing. The allied and
empty attempts are refused without a garrison raise; filling the capacity does not prevent assault. Variant B never clears an enemy building as an
assault; on an eligible own building with a free occupant place it occupies normally. For the
damage subcase, give variant A its order while the enemy garrison is healthy, then inflict a
nonlethal hit below the red ratio before arrival. Retain only runs where the hit and arrival fall
between consecutive building turns, so claim 9 has not already emptied it; record this boundary.
For mind control, pre-place a B-owned variant A and have A's controller take it, then give it the
normal capture order to a separately prepared B-owned occupied building. Verify its current owner
is A and it remains mind-controlled at arrival. These two arrivals clear the garrison despite the
damage and mind control. They do not waive the shared building gates listed in claim 14.
Falsified by an allied/empty assault, a full-capacity refusal, consumption of the assaulter, a raise
whose counter effect sees occupants already cleared, or an extra red-ratio or mind-control assault
gate. A damage run whose garrison already evacuated is void, not an assault refusal. Kill credit, experience and
destroyed-event results are recorded to settle the open question, not assigned an expected value.
Exact animation-to-occupant correspondence is not established by this scenario.

**G11. Erasure (claim 15).** Setup: a garrisoned civilian building, an enemy stock erasing unit,
and a control empty copy. Give each occupant a destroyed-event counter and note starting kill/loss
counts. Procedure: erase the garrisoned building completely. Observable: occupants in play or
inside, death effects, occupant tag counters and score deltas distinct from the building's loss.
Expected: completed erasure removes the occupants without placing them outside or registering
occupant deaths. An escaping occupant, retained off-map occupant after completion, or occupant
death registration falsifies the rule. A separate interrupted-erasure run may record when occupants
are lost, but that timing has no predicted result here and cannot confirm the claim.

**G12. Hits do not pass through (claim 16).** Setup: one garrisoned building per tested stock weapon
class (bullets, flame, gas, radiation and explosives), occupants with recorded health, and an empty
control building for each weapon. Procedure: land nonlethal hits without reaching the forced-
evacuation threshold; record damage to each building, then evacuate. Observable: hit acceptance,
occupant count and occupant health. Expected: building hits are accepted as on the control, count
is unchanged, and every occupant has its entry health. Any garrison-specific hit rejection,
occupant loss or occupant health decrease falsifies the corresponding part. These tests do not
prove the universal absence of a path in every possible weapon-field combination.

**G13. Trigger eviction and house-wide hunt (claims 11 and 13).** Setup: a civilian building with
identified occupants and free exits, one nearby enemy target, and a timed trigger. For part (a),
attach only this building to the trigger and give it eviction action code 111. For part (b), use
house-wide hunt action code 6 with the building owner's house as recipient. Procedure: garrison,
wait for ownership to settle, and let the timed action fire; repeat each part with every ring cell
blocked. Observable: action tick, occupant count, placement, target/behaviour after scatter, and
occupant death counters. Expected with free exits: both actions empty the building in the firing
tick, using claims 10 and 11; only (b) queues hunt, so its released units subsequently seek the enemy
without a player attack order. With blocked exits both remove occupants without deaths, rather
than using inside fallback. A retained garrison, fallback placement, hunt requested by (a), or no
hunt requested by (b) falsifies that part. Observe queued behaviour directly to distinguish hunt
from ordinary nearby auto-acquisition. Team-membership changes remain outside this fixture.

## Open questions

- Whether the assault removal of claim 14 counts as a kill for the assaulter's house (score,
  experience) and raises the occupants' destroyed events. Test in G10.
- Whether an exit ring cell outside the map or outside the usable area can qualify (claim 10).
- Whether occupants beyond the number of free standing positions in the exit cell are removed
  (claim 11), or always placed. Test with a full-capacity garrison in G7.
- Whether the owner change of claims 7 and 8 plays the "building captured" announcement or any
  other effect that an engineer capture has and this change does not (planned S-0509 claim 12).
- Whether any skirmish cause other than map action code 6 and the original's computer player
  sends the house-wide hunt order of claim 13.

## Questions for the project lead

- With no civilian house (Edge cases), adopt leaving the owner unchanged and skipping abandoned
  feedback? The unsafe original outcome is `unmeasured`, confidence low: no deliberate unsafe-data
  run is proposed. This is not reachable with stock skirmish data.
