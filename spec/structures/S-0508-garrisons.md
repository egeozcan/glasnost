---
id: S-0508
title: Garrisons (occupying buildings, ownership of civilian garrisons, evacuation and clearing)
topic: structures
revision: 39
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
measurements: []
requests: [Q-0660, Q-0972, Q-0973, Q-0970, Q-0971, Q-0990, Q-2712, Q-2926, Q-2927, Q-2951]
supersedes: []
---

# S-0508: Garrisons (occupying buildings, ownership of civilian garrisons, evacuation and clearing)

## Summary

How infantry occupy a building whose type can be garrisoned: which units may enter which building,
when, in what order and up to what number. It also covers how a civilian garrison building changes
hands with its occupants, and how occupants leave or are lost: an evacuation order, the building
being badly damaged, destroyed, sold or erased, a map trigger, the house-wide hunt order of a map
action or of the computer player's periodic checks (S-0217), an assault, or an occupant's own house
being defeated. Skirmish only; network games and campaigns are out of scope.

Neighbouring specs own the rest:
- the order and cursor that send an infantry unit to garrison (S-0214 row I8) and the *capture*
  behaviour while it walks there (S-0215 claim 28),
- the arrival moment (S-0208 claim 22) and scatter requests (S-0208 claims 11 to 13),
- what a garrisoned building's attack does: weapon rotation (S-0308 claim 13), muzzle index
  (S-0313 claim 6), rate factor (S-0301 claim 7), damage factor (S-0302 claim 2) and range
  (S-0307 claim 4),
- when a sale puts the occupants out (S-0408 claim 9) and how a destroyed building's contacts and
  survivors leave (S-0312 claims 17 and 18),
- the effects of a building ownership change (S-0509 claim 12),
- the house-wide hunt order and the computer player's checks that give it (S-0217 claims 1 to 4),
- engineers and capture (S-0509), transports (S-0507) and special attacks (S-0519),
- the drawing of occupied buildings (S-0108 claim 2) and the announcements (S-0701 claim 28).

## Terms

- **Garrisonable type.** A building type whose occupiable flag is set (Import mapping). S-0214 and
  S-0509 use the same word. A *garrisoned building* (glossary) is a building of such a type with at
  least one occupant.
- **Occupant.** An infantry unit inside a garrisonable building: S-0213's *garrison occupants*
  (Terms) are the occupants of one building. It is off the map and out of play
  (glossary, *in play*), keeps its own owner, and is neither destroyed nor removed. A building's
  occupants form an ordered list: the *first occupant* is the one that has been inside longest.
  List positions are integers numbered from 0. A *dead occupant place* is a position retained
  after claim 19's defeat hit kills its occupant in the original (claim 20). Our engine removes
  that place under claim 23; it is not a living unit.
- **Occupant capacity.** A garrisonable type's imported occupant limit (Import mapping), tested
  for equality as claim 1 specifies, not as a general upper bound.
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
   - the building's occupant count is not equal to its occupant capacity,
   - the building does not have both positive health and a badly damaged health ratio (S-0108
     Terms, compared as S-0108 states),
   - the unit is not mind-controlled (S-0214 Terms).

   Alliance plays no part: an allied player's infantry may not garrison your building, and you may
   not garrison an enemy's. An empty building with capacity zero lets no unit in. The capacity
   test is equality, not an upper bound: a negative capacity does not stop entries, and a count
   already above capacity does not stop them either (quirk, Edge cases). The damage test itself
   adds no positive-health requirement when the building has zero or negative health, though
   destruction normally prevents such an arrival.
   This test is the "can garrison" test of S-0214 row I8 and the occupiable-building test that
   S-0215 names in its Terms.

   **Units on their way count for nothing.** The count compared is that of the occupants inside.
   No place is held for a unit that has the building as target or destination, so any number of
   units may hold the same building at once, and those that arrive after it is full are refused
   (claim 4). This holds wherever another spec applies this test: S-0309 claim 25 part (e), and
   S-0215 claims 23 and 27.

   **"May occupy" in other specs.** Where S-0215 claims 23 and 27 ask whether an infantry unit
   "may occupy" a building, the test is this claim's for a unit of an occupier type, and claim
   14's assault eligibility for a unit of an assaulter type that is not an occupier type. It is
   never met by a type with neither mark. The condition belongs to the occupier and assaulter
   alternatives of those claims: an infiltrator type takes the target as destination and is forced
   into *capture* whatever this test gives. S-0309 claim 25 part (e) asks it only of a searcher of
   an occupier type, so an assaulter type that is not an occupier type never passes that
   alternative. Part (e) has no behaviour condition: whichever of S-0309's searches runs first, in
   whatever behaviour, may pick the building as target. A queued behaviour runs no search, so for
   a unit whose *hunt* is queued and not yet current, the search that picks the building is one of
   its current behaviour. In *area guard* that is area guard's own search in its processing
   (S-0215 claim 26), or the attack-move search when the unit carries that memory (S-0215 claim
   33); the opportunity search of S-0215 claim 33 does not run in *area guard*.
   - Backing: `measurable` (scenarios G1 and G2) for owner, type, capacity, damage, sale and mind
     control, and (scenario G21) for units on their way. The meaning of "may occupy" for an
     assaulter type and the infiltrator alternative are `unmeasured`: no scenario here gives a
     computer-controlled assaulter or infiltrator an attack or hunt on a building that fails this
     test. Which search picks the building for a unit with a queued *hunt* is `unmeasured` here: no
     scenario of this spec gives a unit in *area guard* a queued *hunt*, and S-0215 claims 26 and 33
     carry their own backing for the searches they name. The build-up, unusable-centre and erasure
     gates are `unmeasured`: no data-only fixture is established that delivers an already ordered
     entrant during those states, rather than merely having the cursor reject its order.
     Over-capacity and non-positive-health arrivals are also `unmeasured`: no validated fixture
     establishes either arrival state. G1's negative-capacity variant distinguishes equality from an
     upper-bound test.
   - Confidence: high (owner, capacity, badly damaged, sale, mind control); medium (built up,
     usable area, being erased, units on their way, "may occupy" in other specs, the search for a
     unit with a queued *hunt*)

2. **When an entry is attempted.** An infantry unit of an occupier type or an assaulter type tries
   to enter when it arrives at the end of a walking step (S-0208 claim 22) in a cell that holds a
   building, while its behaviour is *capture* and that building is its *destination*. Its target does
   not count, and *area guard* or *patrol* do not count: an occupier type arriving in those cases
   follows S-0509 claims 1 and 2 instead and does not garrison. When that general building-entry
   test is met, it can still give one general entry raise; that is not a garrison entry raise.
   Any building destination counts, garrisonable or not: one that fails the test is refused (claim 4), even a vehicle-like building (S-0214 Terms)
   that S-0509 claim 1 would otherwise hand to a hijack. S-0509 has no outcome at that same arrival,
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
   claims 11 to 13), as an engineer refusal does (S-0509 claim 4). It stays on the map. The
   refusal does not change its target.

   **No hunt after a refusal (original only, and test-only computer-controlled player houses).** For
   a unit of a computer-controlled owner, a *hunt* current or queued when *capture* was forced on it
   was removed by that force (S-0215 claim 9). Capture's hunt rule (S-0215 claim 28) leaves out
   every unit of an occupier type by its type alone, whether it was refused, lost its target on the
   way or never set out. An assaulter type that is not an occupier type is not left out. None of
   S-0215's rules for *capture*, *attack* and the fall-back to rest (claims 14, 23 and 28) gives an
   occupier-type unit *hunt*, so a refused occupier-type unit does not go back to *hunt* by any of
   them. The force does not touch a suspended *hunt* (S-0215 claims 9 and 10): a unit whose *hunt*
   an override such as a retaliation had suspended can have it made current again by a later revert.
   A new order, a map action or another spec's rule (a berserk hit, S-0610 claim 4, for example) can
   also give *hunt*. In the original, and for a test-only computer-controlled player house in our
   engine (S-0217), the owner's own periodic checks give it *hunt* too: a check that gives the
   house-wide hunt order (S-0217 claims 1 to 4) queues *hunt* for every unit of the house on the map
   (S-0215 claim 7 says what that changes for a unit that has *hunt* current), so a refused unit of
   a house that owns no building other than occupied civilian buildings (S-0217 Terms) and no
   construction vehicle has *hunt* queued at the next check of its house, at most 112 ticks later,
   if it is still on the map then. A refused unit for which *hunt* is already current at that check,
   by a revert or by any of the other means above, is changed by the check as S-0215 claim 7 gives
   for a unit with *hunt* current. The queued *hunt* becomes current as S-0215 claims 7 and 11 give.
   What a refused unit does after the scatter request and before such an order is not given here
   (Open questions).
   - Backing: `measurable` (scenarios G1 and G2) for the destination, the scatter request and the
     unit staying on the map, and (scenario G21, for units refused before its trigger's tick) for
     the absence of *hunt*. The *hunt* that the owner's checks give takes S-0217 claims 1 to 4's
     backing and levels (S-0217 scenario H1, medium): scenario G21 leaves the *hunt* from its
     trigger's tick on to S-0217 scenario H1. S-0217 scenario H1 judges a check's order only through
     its evictions, so the *hunt* a refused unit gets at a check after tick 0 at which its house
     owns no occupied building is `unmeasured` there (S-0217 claim 4), and so here. The unchanged
     target is `unmeasured`: in a computer-controlled owner's run other rules can change the target
     in the refusal tick (S-0309 claim 25), and no setup here separates them from the refusal.
   - Confidence: high (destination, scatter request, staying on the map); medium (unchanged
     target, no *hunt*)

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
   has at least one occupant and belongs to the civilian house changes owner to the first occupant's
   house. This is a building ownership change (S-0509 claim 12). It happens at the building's turn,
   not at the entry: in the tick of the entry the building may still belong to the civilian house
   (scenario G5). A garrisonable building of any other tech level never changes owner through this
   spec. Within the building's turn this change comes after the building's behaviour processing, its
   per-tick search (S-0215 claim 33) and all of its firing in that turn, the animation-delayed shot
   of S-0301 claim 14 included. The order is the same in every turn, the turn of the entry tick
   included. The change drops the building's target, whoever owns the target (S-0309 claim 30). So
   when the building's turn in the entry tick comes after the entry, its per-tick search there may
   pick an object of the entrant's house, which is not allied with the civilian house. A building
   with no animation-delayed shot pending in that turn does not fire at it: a target found by the
   per-tick search is first acted on in a later tick (S-0215 claim 33, part c), and the change at
   the end of that same turn drops it. The delayed shot of S-0301 claim 14 comes after the per-tick
   search in the building's turn. So a building whose delayed shot was pending when it was emptied
   keeps it pending across its return to the civilian house (claim 8, S-0309 claim 30), and if it is
   entered again and its delay ends in the turn of the change, that shot's firing check uses the
   target that search gave it, if any (S-0309 claim 30), and on *can fire* it fires that shot at
   that object of the entrant's house before the change drops the target. Our engine follows this.
   From its next turn on the building belongs to the entrant's house, and its searches no longer
   take that house's objects or its allies', as far as S-0309 table C's row *friendly* rejects them
   (with that row's conditions and its friendly-row exceptions). When the building's turn in the
   entry tick comes before the entry, the building is still empty in that turn and makes no passive
   search there (S-0309 claim 2). Its next turn, later in the same tick or in a later tick, is then
   the turn of the change, with the same outcome.
   - Backing: `measurable` (scenario G5) for a single-house garrison and owner-change timing, and
     (scenario G20) for the dropped target and, by variant (c)'s runs with a search of the
     building's own in the turn of the change, for the change after the per-tick search. The change
     after the behaviour processing and after the firing, the animation-delayed shot included, is
     `unmeasured`: in G20 the building has no shot and no pending shot in the turn of the change by
     which that order would show, as G20's expected result gives, and no setup here gives a building
     of this claim one in that turn. The delayed shot after the per-tick search, and so the shot at
     the entrant's house's object, is `unmeasured`: no setup here empties a building with its
     delayed shot pending and has it entered again before the delay ends. Choosing between occupants
     of different houses is `unmeasured`: no validated data-only fixture gives two houses' units the
     required arrivals between consecutive building turns.
   - Confidence: high (who becomes owner, the change after the per-tick search in every turn, the
     dropped target); medium (which turn, the change after the behaviour processing and after the
     firing, the animation-delayed shot included, the delayed shot after the per-tick search and the
     shot at the entrant's house's object that follows)

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
   These return and feedback rules require an existing civilian house. Claim 21 describes the
   original's unsafe missing-house case; claim 24 instead requires silent owner retention in
   our engine.
   - Backing: `measurable` (scenarios G4 and G6)
   - Confidence: high (the change and the gate on the owner); medium (the radar event throttling the
     announcement)

9. **Forced evacuation when badly damaged.** At each of its own turns, a civilian garrison building
   that is badly damaged (as in claim 1) evicts its occupants (claim 10) without hunting and
   without inside fallback. A building emptied this way returns to the civilian house in that
   same turn under claim 8, with feedback for the owner it had before returning. While it stays badly damaged,
   claim 1 refuses every new occupant. A garrisonable building of any other tech level is never
   evacuated this way. Evacuation itself does not require a civilian house; if none exists, the
   occupants still leave. The original's subsequent ownership operation is unsafe (claim 21).
   Our engine retains the owner without abandonment feedback (claim 24).
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

    There is no independent legal-cell or usable-area gate in this search. A candidate with no
    map cell is read as the stand-in cell of S-0009 claim 44 and undergoes the same entry check.
    A candidate outside the usable area qualifies exactly when that check is *clear*, not merely
    because its terrain is passable. An ordinary occupant that entered from inside the usable
    area retains that status while inside the building: S-0209 claim 4's outside-area rejection
    applies. Exceptions remain those of the entry check, not a new garrison permission. The
    reference-level bypass of S-0208 claim 19 also applies: terrain level −6 or lower gives
    *clear* before the usable-area and occupancy tests (`quirks.garrison_negative_level_exit`).
    Outside-area cells are thus not
    categorically forbidden; nor is a missing map cell rejected independently of its stand-in
    reading and the occupant's entry rules. S-0009 claim 44's immutable stand-in departure
    still applies in our engine.
    - Backing: `measurable` (scenarios G7, G8 and G16) for ranking, skipped corner,
      first-occupant choice and ordinary outside-area rejection. Qualification of a missing map
      cell with an occupant exempt from outside-area rejection is `unmeasured`: G16 does not
      establish that special occupant state inside a garrison. The negative-level bypass inherits
      S-0208 claim 19's backing; G16 proposes its exit-specific comparison. The exact initial
      reference level remains `unmeasured` until that fixture is validated; flat-ground tests
      do not separate it from other levels.
    - Confidence: high (order, skipped corner and absence of an independent map-boundary gate),
      medium (outside-area consequence and placement of a boundary fixture)

11. **Putting occupants out.** With an exit cell, the occupants are put on the map in that one cell,
    from the last occupant to the first, within the same tick. Each faces facing 0 and is placed
    at the cell's centre on its terrain surface, ignoring standing-position occupancy for this
    placement. Several released occupants can therefore overlap at the centre. There is no
    random draw to choose a standing position, unlike S-0208 claim 3's ordinary position choice.
    An occupant that cannot be placed is removed (not destroyed). Claim 22 gives the placement
    conditions; standing-position crowding is not a failure condition. Each placed occupant then, before
    the next one is placed:
    - has its target cleared,
    - receives a scatter request whose threat point is the building's centre, forced and
      move-anywhere (S-0208 claims 11 to 13),
    - when the eviction is hunting, leaves its team and is given the *hunt* behaviour (S-0215) as
      its next behaviour.

    The building then has no occupants. Its shared occupant/muzzle index is 0 (claim 18 and
    S-0313 claim 6).
    - Backing: `measurable` (scenarios G7, G9 and G13) except team departure, which is
      `unmeasured`: no fixture is established that retains a team member inside a garrison and
      makes its post-eviction team membership observable. Removal on placement failure is also
      `unmeasured`: G7 tests crowding, but cannot guarantee a qualifying cell followed by a failed
      placement. The conditional rule does not assert that crowding necessarily causes failure.
    - Confidence: high (order of placement, centre placement without a position draw, target
      cleared, scatter); medium (the removal of an occupant that cannot be placed); low (team
      and hunting)

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
    | The house-wide hunt order (S-0217 claim 1): map action code 6, or a check of the original's computer player or of a test-only computer-controlled player house (S-0217 claims 3 and 4) | yes | no | the action's firing tick, or the check's tick (S-0217 claims 1 and 2) |

    - Backing: `measurable` (scenarios G6, G7, G8 and G9 for the first four rows; G13 for the last
      two, and S-0217 scenario H1 for the checks of S-0217 claim 4 in the last row; those of S-0217
      claim 3 are `unmeasured` there)
    - Confidence: high (first four rows); medium (the fifth row, a map trigger action that
      evicts); low (the last row's order given by map action code 6); medium (the last row's order
      at the checks of S-0217 claim 4 after tick 0, which S-0217 scenario H1 tests, and at the
      checks of S-0217 claim 3, `unmeasured` here, at that claim's medium); low for the order given
      at a house's check of tick 0 (S-0217 claim 4)

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

    On a passing attempt, the raise of claim 3 sees the occupants still inside. Every occupant is
    removed during that attempt, from the last list position to position 0. Each occupant's
    animation and reversed kill credit occur before its removal; its credit and removal precede
    those of the preceding list position. Destruction-tag effects on the assaulter can therefore
    observe that reverse order through the credited occupants and their retained state.

    For the occupant at list position i, the assaulter weapon's occupant animation uses the
    building's muzzle point i, projected as S-0313 claim 6's pixel shift. This is the position
    in the occupant list, not the shared firing index of claim 18. There is no wrapping by the
    number of configured points: supported point numbers are 0 through 9. Missing points retain
    their imported defaults. An occupant at position 10 or later has no supported
    animation point and no specified safe original result; this is not original wrapping to point 0.
    Claim 25 supplies our engine's safety departure for those positions.
    The assaulter does not go inside: it has its destination cleared and scatters as in
    claim 4. The building, now empty, follows claim 8 at its next turn when it is a civilian
    garrison building.

    **Reversed kill credit.** For each removed occupant, the surviving assaulter is recorded as
    destroyed by that occupant, not the other way round. The assaulter's destruction notifications
    are raised and its house's loss bookkeeping runs; the occupant receives the kill credit and
    any experience, using S-0311 claims 7, 8 and 10 with the assaulter as victim. This does not
    damage or remove the assaulter or cause its normal death animation. The occupants themselves
    are removed without their own destruction notifications. Scoring exemptions still apply
    (quirk, Edge cases).
    - Backing: `measurable` (scenario G10, named test data) for occupied/enemy/allied eligibility,
      capacity, damage, mind control, both-mark precedence, clearing and reversed kill credit.
      Nonzero score and experience for a hostile occupant are `unmeasured`: no validated
      data-only fixture gives a locally ordered assaulter occupants of another house. G10's
      same-house case tests the reversed notifications and counters, not a nonzero gain.
      The one-way alliance distinction and the build-up, sale, unusable-centre and erasure gates
      are `unmeasured`: G10 establishes neither an asymmetric-alliance arrival nor an already
      ordered assaulter arriving during those building states. Reverse removal and credit order
      are `measurable` (G17); animation position is `measurable` for supported point numbers
      (G17's changing-count comparison). Unsupported animation points are `unmeasured`: no safe
      oracle outcome is established, and G17 deliberately stays within the supported numbers.
    - Confidence: high (reverse list order, point number and independence from the firing index),
      medium (other assault rules)

15. **Erasure.** When a building with occupants is erased (glossary, *being erased*), its occupants
    are removed with it, without death and without being put out.
    - Backing: `measurable` (scenario G11)
    - Confidence: medium

16. **Hits on the building do not pass through to occupants.** Weapon and warhead hits do not
    directly damage, remove or evict individual occupants. They can still cause the building-level
    evacuation, destruction or erasure of claims 9, 13 and 15. House defeat is a distinct source
    of damage to the occupants themselves (claim 19), not weapon damage passing through the
    building. No hit is rejected because the target is garrisoned.
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

18. **Occupant changes and the shared firing index.** The shared occupant/muzzle index
    (S-0313 claim 6; S-0308 claim 13) is an integer counted from 0. After a completed eviction
    or clearing action it is 0. This applies to evacuation, failed-exit removal, assault and
    erasure (claims 10 to 15). Reoccupation starts with index 0. Additional entries append
    occupants (claim 5) and retain the index, rather than resetting or advancing it.
    This specifies the completed action, without requiring an order between its index reset
    and individual releases or removals. None of these actions offers selective evacuation.
    Independent death inside has the original-side rule of claim 19 and our engine's safety
    departure of claim 23, not this whole-building reset.
    - Backing: `measurable` (G14) for completed deploy-evacuation/assault reset, reoccupation
      and additional entry. Reset on other eviction causes, including failed-exit removal, is
      `unmeasured` here: G6 to G13 observe departures but do not isolate the firing index across
      each cause. The index of an erased building is `unmeasured`: the building has no subsequent
      shot through which to observe it. Reset timing relative to individual releases/removals is
      also `unmeasured`: no validated fixture interleaves a shot or credited destruction within
      the action. Such timing is not an implementation constraint (Changes for other specs).
    - Confidence: high

19. **Skirmish departure and death inventory.** The following events have different scopes.
    A change to the building is not, by itself, a change to any occupant's own house (claim 17).
    The table gives original outcomes; our engine replaces the independent-death result with
    claim 23's safety rule.

    | Event | Occupants affected | Result and index rule |
    |---|---|---|
    | Building deploy order, civilian red-health evacuation, map eviction or house-wide hunt | all | eviction, claims 9 to 13; completed-action index 0 |
    | Sale | all | eviction with inside fallback, claim 13; completed-action index 0 |
    | Destruction of the building, including a killing weapon hit or a map action's killing hit | all | eviction, claim 13; completed-action index 0 |
    | Assault | all | clearing, claim 14; completed-action index 0 |
    | Completed temporal erasure of the building | all | removal, claim 15; completed-action index 0 |
    | Nonlethal weapon hits without building-level evacuation or erasure | none | no occupant damage or departure, claim 16; no index change from the hit |
    | Engineer capture, mind control or another ownership change of the building | none | occupants and their houses retained, claim 17; no index reset |
    | Superweapon teleport | none | does not select a building or its off-map occupants, S-0513 claim 34 |
    | Packing a building up into a vehicle | whole-building outcome, not selective release | S-0415 claim 9 owns release versus suppression; stock garrisonable types do not pack up |
    | Defeat or surrender of an occupant's own house while the building belongs to a surviving house | that house's occupants only | individual killing hits under S-0414 claim 5; no eviction or index reset |

    The last row is an exception to the proposed absence of selective skirmish death. Occupants
    keep their owners, so a mixed-house garrison can lose one house's occupant while another
    house's occupants survive inside. Even without a mixed garrison, a building transferred
    away from its occupants' house survives that house's defeat while its occupants are hit.
    Defeat does not require a unit to be on the map before delivering its killing hit.

    In the original, such a death retains the numeric firing index: it neither resets it nor reduces it modulo
    a new count. It also does not perform the individual occupancy removal described for
    set-up in S-0311 claim 15. The original can consequently retain a dead occupant among its
    firing choices. Continued firing or experience selection through that dead choice has
    no specified safe original result here. This is not evidence of original survivor compaction.
    Claim 23 supplies our engine's deliberately different removal and index rule.
    - Backing: `measurable` for the completed whole-building departure outcomes through G6 to
      G14, with index-reset coverage limited as in claim 18; ordinary hits through G12,
      ownership transfer through G10, and defeat hits and numeric
      index retention in a foreign-owned building through G15. Mixed-house selective death is
      `unmeasured`: no validated fixture supplies the required differently owned entrants.
      Other ownership-change causes retain claim 17's limitations. Superweapon teleport selection and
      pack-up rules inherit their cited owners' backing. Unsafe subsequent firing/credit is
      `unmeasured`: no unsafe continuation is proposed as a confirming run.
    - Confidence: high (group actions and defeat's off-map inclusion); medium (selective
      mixed-house consequence, retained index and dead firing choice)

20. **Original behaviour: dead places still count; their use is unsafe (Q-0970).** After claim 19's independent
    defeat death, the list length n still includes the dead occupant place. The capacity test
    of claim 1, empty-building test of claim 8, delay quotient of S-0301 claim 7 and firing-index
    modulus of S-0313 claim 6 all use this n, not a count of living occupants. A building with
    only dead places is not empty for abandonment. No death-specific decrement, compaction or
    index repair occurs.

    This does not establish a safe shot from a dead place. The original supplies no next-living
    substitution, dead-place weapon fallback or dead-place experience exemption. Weapon selection
    (S-0308 claim 13) and the occupant-recipient branch of experience selection (S-0311 claim 8)
    require an occupant that still exists. Once it has been removed, which weapon fires, whether
    a launch succeeds and advances the index, and which occupant receives experience have no
    dependable original result. The ordinary muzzle calculation selects numeric point k
    independently of occupant health (S-0313 claim 6); it cannot make an unsafe launch safe.
    A trainable building's own experience branch remains S-0311 claim 8's rule, not a new
    fallback invented for dead occupants.

    A later eviction has no dead-place skip either. It includes the retained places in the exit
    search and release attempt, or in removal when there is no exit. It does not intentionally
    create a replacement unit. Once the occupant no longer exists, neither a successful release
    nor an orderly skip is guaranteed; no placement for that dead place is specified as safe.
    Claim 23 defines the deterministic safety departure for our engine; it is not an original
    rule recovered by choosing a survivor.
    - Backing: `unmeasured` for unsafe original continuation: no safe confirming run or outcome
      is proposed for firing, experience or eviction through a dead place. G15 retains the
      proposed count/non-abandonment observation before unsafe use; that limited observation
      is `measurable`. Delay and modulus use the same original count by their cited claims.
    - Confidence: high (unchanged count and absence of dead-place filtering); medium (the
      boundary beyond which no dependable continuation can be specified)

21. **Original behaviour: no civilian house is not silent retention (Q-0971).** With no civilian house, an empty
    civilian garrison building does not simply keep its owner without feedback. At its turn,
    the original still requests claim 8's local-owner sound and radar event, with the
    announcement conditional on event creation, then attempts an ownership change without a
    receiving house. There is no safe resulting owner or rule for every later turn. The same
    happens after claim 9's forced evacuation: evacuation is not skipped, and emptying leads
    into this unsafe ownership operation in that turn. A still-occupied building with a valid
    owner cannot match a nonexistent civilian house and therefore does not change owner under
    claim 7. Claim 24 requires silent owner retention as a safety departure, not original behaviour.
    - Backing: `unmeasured`: no deliberate missing-house crash run is proposed. A diagnostic
      variant of G6 would remove civilian-side membership from every instantiated country and
      observe the first empty-building turn only, but is not a safe confirming scenario for a
      stable owner. The ordinary existing-house control remains G6.
    - Confidence: high (feedback attempt, evacuation and absence of an ownership guard),
      medium (unsafe ownership outcome, not a specific crash time)

22. **Limits of centre placement (Q-0972).** For a living occupant still off-map inside the
    building, with an ordinary valid centre position during an active game, claim 11's placement
    succeeds. There is no second infantry entry-outcome test at placement, no test of the later
    occupants' movement permissions, and no capacity, terrain-passability or usable-area refusal
    added after choosing the exit cell. A later occupant can thus be placed on terrain which it
    could not itself have selected as an exit.

    Outside those normal conditions, placement can refuse an invalid position, an inactive game,
    an object not in its off-map state, or an object already marked on the map. A failed map-entry
    marking also refuses placement; for ordinary infantry at this site its relevant preconditions
    are that it is off-map and not already map-marked. There is no random centre-position failure.
    An absent or already removed occupant is instead claim 20's unsafe case, not a living unit
    that this rule promises to revive. Existence alone does not guarantee success.
    - Backing: `measurable` (G7, G9 and G16's different-permissions variant) for normal centre
      placement and lack of a later-occupant terrain test. Exceptional state refusals are
      `unmeasured`: no validated data-only fixture supplies a still-listed living occupant in
      those inconsistent states or invokes eviction outside an active game. G16 records any
      normal-state loss as a falsifier, not evidence for an unspecified random failure.
    - Confidence: high

23. **Engine design rule: remove an occupant's place on death (Q-0970).** This is a safety
    departure from claims 19 and 20, not original behaviour, with no quirk switch. When an
    occupant dies inside, from defeat or any other cause, its place is removed in the same
    update. The remaining occupants keep their relative order and are numbered from 0.
    The numeric firing index is retained if it is less than the new count n; otherwise it
    becomes 0, including when n = 0. This is a range check, not reduction modulo n.

    Claims 1 and 8, S-0301 claim 7 and S-0313 claim 6 use the count after removal. No shot,
    experience award or eviction placement uses a removed place. With n > 0, the repaired
    index selects the weapon under S-0308 claim 13 and muzzle under S-0313 claim 6; ordinary
    successful-shot advancement still applies. Experience follows S-0311 claim 8 using the
    remaining occupants and the current index at destruction time, not an unconditional
    no-experience rule. Its trainable-building self-recipient branch remains unchanged.
    With n = 0 there is no occupant weapon or occupant experience recipient, and a civilian
    garrison building is empty for claim 8, subject to claim 24. Later eviction places only
    remaining occupants under claims 10 to 12; it creates nothing for a removed place.
    The distinguishing observable from the original is the reduced count in that death update,
    and, after the last occupant dies, eligibility for abandonment at the building's turn.
    - Backing: `measurable` on our engine (G18); the required result is a design rule, not an
      oracle prediction. Original count retention and unsafe continuation retain G15 and
      claim 20's backing limitations.
    - Confidence: high

24. **Engine design rule: missing civilian house suppresses abandonment (Q-0971).** This is
    a safety departure from claim 21, not original behaviour, with no quirk switch. When no
    civilian house exists, claims 8 and 9 do not change the building's owner. It keeps its
    current owner at every later turn while that condition holds, without the abandoned sound,
    radar event or announcement. Claim 9's forced evacuation still applies and empties the
    building under claims 10 to 12. With a civilian house present, claims 8 and 9 are unchanged.
    Claim 7's occupied-building rule is unchanged: a valid owner cannot equal a nonexistent
    civilian house. The distinguishing observable is the absence of abandonment feedback at
    the first empty-building turn, rather than the original's feedback attempt followed by
    unsafe ownership; stable owner retention is an engine promise only.
    - Backing: `measurable` on our engine (G6's missing-house variant); this is a design rule,
      not an oracle prediction. The original unsafe path remains `unmeasured` under claim 21.
    - Confidence: high

25. **Engine design rule: assault animation beyond point 9 (Q-0990).** For an occupant
    removed by assault at list position i ≥ 10, our engine shows its occupant animation at
    muzzle point j = i mod 10. This is exact non-negative integer remainder, with j in 0
    through 9 and no rounding. The selected point is looked up and projected exactly as
    claim 14 looks up point i for positions 0 through 9, including the imported default
    for a missing point. The modulus is always 10, not the number of configured points
    or the occupant count; the shared firing index does not participate.

    This is an engine design rule, a safety departure rather than original behaviour,
    with no quirk switch. Claim 14's reverse credit/removal order is unchanged, including
    animation and credit before each removal. Positions 0 through 9 are unchanged.
    - Backing: `measurable` on our engine (G19); this is a design-rule check, not an
      oracle prediction. The original result at positions 10 or later remains `unmeasured`:
      no safe outcome is established and no unsafe confirming run is proposed.
    - Confidence: high

## Formulas

> Exit ring size = 2 × (W + H) + 4 cells. Ranked distinct candidates = 2 × (W + H) + 3 cells
> (all but one corner). Exact integer arithmetic, no rounding; positive W and H in cells.
> Cell coordinates are integers (glossary, *foundation*).

> Inside fallback cell = (x₀ + W − 1, y₀ + H − 1). Integer cell coordinates.

> The capacity gate accepts exactly when occupants ≠ capacity. Exact integer comparison.

The damage gate rejects exactly when health > 0 and health / type maximum health ≤ red ratio.
The ratio and comparison use binary64 as in S-0108; no integer rounding is applied. This is a
rejection condition, not a requirement that an accepted building have positive health.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tech level of a civilian garrison building | −1 | level | claims 7 to 9 |
| Facing of a put-out occupant | 0 | facing unit | claim 11 |
| Reference level of the exit-cell check | −1 | height level | claim 10 |
| Absent occupant capacity | 0 | occupants | Import mapping |
| Supported assault muzzle points | 10 | points, numbered 0 through 9 | claim 14; also the engine-only remainder divisor in claim 25 |
| Highest terrain level receiving the exit-check bypass | −6 | height level | claim 10; integer comparison |

## State and ordering

| State | Event | Result |
|---|---|---|
| occupier walking in *capture*, destination a garrisonable building | arrival in a cell of that building, test passed | one entry raise; unit inside at the end of the list; announcement when it is the first |
| same | arrival, test failed | no raise; destination cleared; scatter from the building centre |
| civilian garrison building, civilian house owner, occupants ≥ 1 | building's turn | owner becomes the first occupant's house |
| civilian garrison building, other owner, no occupants, civilian house exists | building's turn | abandoned sound and radar event for the local player; owner becomes the civilian house |
| civilian garrison building, badly damaged, occupants ≥ 1, civilian house exists | building's turn | eviction, then (now empty) return to the civilian house, same turn |
| civilian garrison building, no civilian house, empty or badly damaged | building's turn in our engine | forced eviction when badly damaged; owner retained, no abandonment feedback (claim 24) |
| occupied building | occupant dies inside, in our engine | place removed that update, order retained; index kept when in range, otherwise 0 (claim 23) |
| garrisoned building | deploy order, destruction, sale, trigger, hunt order | eviction with the options of claim 13 |
| garrisoned building | erasure completes | occupants removed with it |

Ordering as observed:
- With a civilian house present, a civilian garrison building emptied by forced evacuation
  returns to civilian ownership in that same turn and gives abandonment feedback to the owner losing it. A building still occupied
  can instead follow its first occupant's owner. Claims 7 and 8 cannot both change its owner in
  one turn.
- In a civilian garrison building's turn, claim 7's owner change comes after its behaviour
  processing, its per-tick search and its firing, and drops its target (S-0309 claim 30), so a
  building with no delayed shot pending never fires at a target that only its civilian ownership
  made a candidate. A delayed shot that was pending when the building was emptied, and that ends in
  the turn of the change after it is entered again, makes its firing check with the target, if any,
  that that turn's per-tick search gave it, and so can be fired at an object of the entrant's house
  (claim 7).
- Within one eviction: the exit cell is chosen once; occupants are placed from the last to the
  first, and each gets its scatter request before the next one is placed. This can distinguish
  the order of identifiable occupants' placement and scatter draws. Placement itself draws no
  random value for a standing position.
  It does not guarantee which unit completes a walking step first.
- At a sale, the occupants leave after the absorbed passengers and before the sale survivors
  (S-0408 claim 9).

## Edge cases and quirks

- **`quirks.garrison_capacity_equality`** (default: on). Claim 1 refuses exactly at capacity,
  rather than at or above it. A negative capacity admits occupants indefinitely, subject to the
  other gates. Fixed behaviour: entry requires occupant count < capacity, so non-positive
  capacity permits no entry.
- **`quirks.assaulter_reversed_kill_credit`** (default: on). Claim 14 records the assaulter as
  destroyed by each occupant even though it survives. Fixed behaviour: each removed occupant
  instead receives destruction notifications and loss bookkeeping, with kill credit and
  experience for the assaulter under the ordinary rules. The assaulter still survives and scatters.
- **`quirks.garrison_negative_level_exit`** (default: on). Claim 10's initial reference-level
  bypass lets an exit candidate at terrain level −6 or lower qualify without the later entry
  tests. With it off, the candidate receives the ordinary terrain, usable-area and occupancy
  tests instead, with no connecting-step height restriction. This switch is scoped to garrison
  exits; it does not change S-0208 claim 19's teleport-specific switch.
- **`quirks.garrison_exit_skips_corner`** (default: on). The exit search never tries the corner
  cell (x₀ − 1, y₀ − 1). With every other exit ring cell blocked and that corner free, the
  occupants are lost (or put inside, with inside fallback). Fixed behaviour: the corner is tried
  last, after the cells of claim 10.
- **`quirks.garrison_no_exit_removes`** (default: on). An eviction without inside fallback and with
  no qualifying exit cell removes every occupant (claim 12). With claim 9, a civilian garrison
  building surrounded by blockers loses its occupants when it becomes badly damaged and, when a
  civilian house exists, returns to that house in the same turn. Fixed behaviour: the eviction does nothing; the occupants
  stay inside, and a later eviction tries again.
- **Mixed garrison.** Because the owner change of claim 7 waits for the building's turn, infantry of
  two houses can both enter a civilian-owned building in the same tick, each passing the passive
  owner condition. The building then goes to the first occupant's house, and the other house's
  occupants stay inside under their own owner (claim 17). No switch: this is the original's rule.
- **Building ownership change on a garrisoned building.** A capture or trigger that hands a
  garrisoned building to another house leaves the occupants as they are (claim 17).
- **No civilian house.** With stock data this cannot happen in a skirmish: the neutral house's
  country belongs to the civilian side, and S-0009 claim 28 never removes a passive house.
  Claim 21 describes the unsafe original result with altered data. Claim 24 requires silent
  owner retention in our engine, without a quirk switch.
- **Death inside.** Claim 20 describes the original's unsafe retained places. Claim 23 requires
  same-update removal and index range repair in our engine, without a quirk switch.
- **Assault beyond point 9.** Claim 14 leaves the unsafe original result unmeasured.
  Claim 25 requires point i mod 10 in our engine, with no quirk switch.
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
| `MuzzleFlash`*n* (art, building image), n from 0 through 9 | pixel pair for assault animation of list position n (claim 14); our engine also uses point i mod 10 for list position i ≥ 10 (claim 25), never the configured-point count or firing index | two integers in pixels, as S-0313 | (0, 0), as S-0313 |
| No user-data field | assault animation safety selection beyond point 9 (claim 25) | exact non-negative integer remainder modulo 10 | engine design rule always applies; no quirk setting |
| No user-data field | shared occupant/muzzle index and action reset (claims 18 and 19) | runtime integer, not imported | initially 0 (S-0313 claim 6) |
| No additional user-data field | defeat hits on occupants (claim 19) | event rule, not imported | S-0414 claim 5 and S-0303 Import mapping own the defeat hit and its warhead |
| No user-data field | original retained dead places (claim 20), replaced in our engine by death removal and index range repair (claim 23) | runtime integer count and index, not imported | engine design rule always applies; no quirk setting |
| No user-data field | reverse assault credit/removal order (claim 14) | integer list position | independent of the firing index |
| No additional user-data field | exit qualification and centre-placement conditions (claims 10 and 22) | runtime spatial/state tests | no independent legal-cell gate; outside-map reads as S-0009 claim 44 |
| No additional user-data field | no-civilian-house case (claims 21 and 24) | house lookup using the country-side mapping above | original unsafe; our engine retains owner without abandonment feedback, always |

The initial absent defaults above have high confidence and `unmeasured` backing here: no fixture
removes the supplying stock layer and distinguishes an initial default from a retained earlier
value. Referenced defaults inherit their owning specs' backing.

For claim 13's map actions, trigger action 111 selects the first eligible building attached to the firing trigger and evicts its occupants without hunting or inside fallback. The house-wide hunt action, trigger action 6, interprets its number slot as a house value under S-0011 claim 23 and sends that house's units hunting, including occupants released by its garrison evictions.
Other action encoding belongs to S-0011. These two action mappings share G13's measurable backing
and medium confidence. S-0011 claim 33 and its Import mapping own the corresponding action entries.

## Measurement scenarios

All scenarios use the glossary's measurement setup: the stock executable and stock data, no
engine add-on and no mods (the named test files of a scenario excepted). Use game-speed
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
A second named variant sets occupant capacity to −1 and orders two occupiers in, one at a time.
Both entries succeed: refusal of either otherwise eligible entry falsifies the equality test.
On our engine with the capacity-equality quirk off, neither entry is allowed. This comparison
uses invented test data, not a stock capacity.

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
under S-0509 claims 1 and 2.

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

**G6 missing-house variant (claims 21 and 24).** Use a named `garrison-no-civilian.map`
variant of G6 in which every instantiated country's side mapping excludes the civilian side.
On our engine, test both an initially empty A-owned civilian garrison building and a garrisoned
one damaged to the red threshold with positive health. Use free exits, bind a distinct abandoned
sound as in G4, and keep the feedback queue clear. Record owner, occupants, placements, sound,
radar events and announcements for the first empty-building turn and ten subsequent turns.
Expected: owner A throughout, no abandoned sound/event/announcement; damage still evacuates
all occupants. Repeat three fresh games per case, plus G6's civilian-house-present control.
Any owner change or abandonment feedback without a civilian house, or skipped forced evacuation,
falsifies claim 24. These are engine design-rule checks only. The original diagnostic would
observe the first empty turn's attempted feedback and unsafe ownership operation, but remains
unmeasured: no deliberate missing-house crash run or stable original-owner prediction is proposed.

**G7. Exit ranking and placement (claims 10 and 11).** Setup: a garrisoned W × H building on clear
level ground, with impassable blockers at chosen ring cells. Procedure: deploy to evacuate. In
separate subcases leave each ranked cell and all lower-ranked cells clear, blocking all higher-ranked
ones; this tests every rank of claim 10, not just the first two. Observable: first placement cell,
facing, standing positions and tick. Expected: the highest-ranked clear cell is used for all
occupants in that tick, facing 0, each initially at the same cell centre, with no standing-position
choice draw. Sample before scatter movement advances them; a later walking position is not the
placement position. Repeat at full imported capacity. A named map variant changes one occupier's movement permissions so the two occupier types
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
are not evidence that garrison occupants escaped. Also replay with each exit quirk disabled on
our engine: the corner becomes the last choice with the corner quirk off; a fully blocked eviction
without fallback leaves the occupants inside with the removal quirk off. These last replays test
the specified alternatives, not the original.

**G9. Reverse placement and retained state (claims 5 and 11).** Setup: three identifiable occupants
entered in a known order and with distinct health and experience. Give the infantry a target before
entry, and let the building attack a separate target. Procedure: evacuate by deploy. Observable:
placement order, standing positions, facing, target, health, experience, behaviour and scatter
requests before later automatic acquisition. A read-only random-state record may distinguish the
scatter draws in the order specified by S-0208 claims 11 to 13. Expected: reverse entry-order
placement at one chosen cell's centre, with no standing-position choice draw, each unit's target
cleared and scatter requested before the next placement, preserved health/experience, and no hunt
requested. An extra position-choice draw or a non-centre initial placement falsifies that part.
The muzzle index is 0 immediately after eviction; re-garrison and check the first muzzle under S-0313 claim 6 as a visible control.
Any retained target, reset state or non-reversed assignment falsifies that part. Which unit first
finishes a later walking step is not a test of placement order.

**G10. Assault (claim 14).** Setup: named test-map overrides of a stock occupier infantry: variant
A has only the assaulter mark, variant B has both marks. Both also have the engineer capability,
so normal capture/repair orders can supply a building destination in *capture* behaviour. The
garrison outcome of claim 2 replaces the engineer outcome. Prepare each occupied test building
by sending A's ordinary occupiers into a civilian building, then use a timed change-owner trigger
(S-0011 claim 24), attached only to that building, to transfer it to B. Record that the occupants
remain inside under owner A and the building is B-owned before issuing the assault order.
For a separate transfer-index subcase, use three occupants, let the garrison fire once and stop
at index 1; record that same index immediately after transfer, voiding this subcase if another
shot intervenes. This index preparation is not required for the other assault subcases.
An occupant loss or owner change during this transfer falsifies claim 17; an index change
falsifies claim 19. Do not silently discard such a result as an assault setup failure. Use separate
mutually allied-B and enemy-B runs, and an empty enemy copy. Make the buildings capturable and
damage the allied copy enough to offer a repair order while preserving positive health and staying
above the red ratio. Keep the assaulter out of the occupants' firing range until its order is given.
Use an existing assaulter weapon with a visible occupant animation. A named map variant makes
the occupants' garrison weapon harmless so that the enemy-owned building cannot kill the entrant
before arrival. Procedure: give each variant the normal capture or repair order, verify *capture* behaviour and building destination, then record arrival.
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
gate. A damage run whose garrison already evacuated is void, not an assault refusal.

For the kill-credit subcase, use scored infantry types, a trainable occupant and an assaulter
without a veterancy-changing effect. Keep A and B mutually hostile, with A's occupants still
inside the transferred B-owned building. Attach distinct repeating destruction counters to the
A-owned assaulter and to each A-owned occupant. Start with one occupant, then repeat with two.
Record counter effects, house kill/loss deltas and assaulter health and experience. Expected:
each clearing raises the assaulter's destruction counter once per removed occupant, but none of
the occupants' destruction counters. A receives both the infantry kill and loss bookkeeping per
occupant, since A owns both the credited occupant and the falsely recorded victim. The assaulter
remains alive at its arrival health and receives no kill experience. Use rookie occupants so
that same-house zero kill value does not trigger a cap reduction of pre-existing experience.
Neither house gains score value for these same-house credits (S-0311 claim 7). Credit to the
assaulter, occupant destruction notifications, an assaulter death or absent assaulter
notifications falsifies the corresponding original-mode prediction. Nonzero experience and
score for hostile occupants remain unmeasured until a validated fixture supplies such occupants
without requiring orders in another player's name. On our engine with reversed
kill credit disabled, the notifications and credit instead identify the occupants as victims
and the assaulter as killer. G17 supplements this scenario for ordered credit/removal and
animation-to-list-position correspondence.

**G11. Erasure (claim 15).** Setup: a garrisoned civilian building, an enemy stock erasing unit,
and a control empty copy. Give each occupant a destroyed-event counter and note starting kill/loss
counts. Procedure: erase the garrisoned building completely. Observable: occupants in play or
inside, death effects, occupant tag counters and score deltas distinct from the building's loss.
Expected: completed erasure removes the occupants without placing them outside or registering
occupant deaths. An escaping occupant, retained off-map occupant after completion, or occupant
death registration falsifies the rule. A separate interrupted-erasure run may record when occupants
are lost, but that timing has no predicted result here and cannot confirm the claim.

**G12. Hits do not pass through (claims 16 and 19).** Setup: one garrisoned building per tested stock weapon
class (bullets, flame, gas, radiation and explosives), occupants with recorded health, and an empty
control building for each weapon. Procedure: land nonlethal hits without reaching the forced-
evacuation threshold; record damage to each building, then evacuate. Before the hits, fire once
from a three-occupant garrison and stop at index 1. Void a run if another shot or whole-building
emptying intervenes before the post-hit index observation. Observable: hit acceptance, occupant
count, occupant health and index before/after the hits. Expected: building hits are accepted as on
the control, count is unchanged, every occupant has its entry health, and the index stays 1.
Any garrison-specific hit rejection, occupant loss, occupant health decrease or index change
falsifies the corresponding part. These tests do not
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

**G14. Firing index across whole-building actions and entries (claim 18).** Use the common
skirmish setup and `garrison-exit.map`, with three identifiable occupants entered one at a time,
free exits and a durable target. Procedure: fire exactly one ordinary shot, then stop the attack
through normal input. Record a nonzero index before ordering evacuation. Record the index after
evacuation, reoccupy in the original order, and record the first shooter and muzzle. Expected:
index 0 after evacuation and the first occupant at muzzle 0 after reoccupation. In a separate
run, begin with two occupants, fire once and stop, then admit the third: the index stays 1,
and the next shot uses the second occupant, not the newcomer or the first. Any different index
or identified shooter falsifies the corresponding rule. Void a run if another shot intervenes.
Repeat each part in three fresh games, recording ticks, occupant identities, index and shot origins
read-only. For clearing, use `garrison-assault.map` and G10's named assaulter fixture: stop the
building at a nonzero index, clear it, record index 0, then reoccupy after its return to civilian
ownership. The first shooter and muzzle must again be the first occupant and 0; another result
falsifies the clearing reset. These observations test the completed action, not an intermediate
single-occupant loss. No test weapon that kills occupants through the building is assumed.

**G15. Defeated occupants in a surviving building (claim 19).** Use the common setup with
`garrison-defeat.map`, adding a third isolated living contender so that defeating A need not
leave only one side. A enters three identifiable occupiers into a civilian building, fires
exactly one ordinary shot at a durable target and stops. Verify index 1. A timed building-only
ownership trigger then transfers the building to B as in G10; verify that occupants still
belong to A. With the building no longer attacking and all occupants still inside, surrender A
through normal input. Record the surrender-processing tick, building health and owner,
occupant health, destruction-counter effects, retained list length n and the numeric index
read-only. Observe during
that processing before later firing or removal can use the dead occupants. Expected: A's
occupants receive defeat's killing hits despite being off-map, the B-owned building is not
hit by A's surrender, no whole-building eviction occurs, n remains 3 including all dead places,
and the index remains 1. If a building turn can be observed without a new dead-place use, it
must not treat these places as empty or request abandonment. A count decrement, abandonment
caused by these deaths alone, a reset or
modulo repair, an eviction caused solely by the occupants' defeat, or exclusion of their hits
because they are off-map falsifies that part. This is an original-only expectation: our engine instead removes the
places in that update (claim 23), so the count distinguishes the two even without another shot.
Repeat three fresh runs. The original's local
end delay does not prevent observing the surrender tick. Do not continue an unsafe attack
as a test of a promised credit recipient. This tests numeric retention without assuming a
validated mixed-house garrison fixture; that selective-survival consequence remains unmeasured.

**G16. Boundary exits and placement permissions (claims 10 and 22).** Use
`garrison-boundary.map`, game speed index 2, stock setup otherwise as above. Place a legal
civilian building with its centre inside the usable area and its exit ring crossing that area's
edge. Keep the first occupant an ordinary stock occupier that entered from inside the usable
area. Block all higher-ranked usable candidates and compare the best otherwise-clear outside
candidate with a lower-ranked usable one. Deploy and record candidate cells, first occupant's
entry outcomes, selected cell and immediate centre placements before scatter. Expected: the
ordinary outside candidate is rejected, and the usable clear one is selected. A selection based
only on outside terrain passability falsifies that gate. Repeat with all usable candidates
blocked: absent another clear candidate, occupants are removed, not placed outside by a special
escape permission.

In a named negative-level variant, make the outside candidate's terrain level −6 and supply a
valid building/entry setup. Record the candidate's terrain height and compare a level −5 control.
Expected: the level −6 candidate passes the entry check's early reference-level bypass, even
outside the usable area; the ordinary outside-area test still rejects the level −5 control.
A blocked level −6 entry outcome falsifies the inherited bypass. In our engine, repeat with
`quirks.garrison_negative_level_exit` off: the outside candidate now undergoes ordinary entry
tests and is rejected by the outside-area gate. This tests the specified alternative, not the
original. Do not equate an editor's refusal to construct this variant with an observed game
outcome; absent a valid imported fixture,
this exit-specific observation remains unmeasured.

For a full-map-edge variant, where the map geometry permits the same legal building/entry setup,
include ring coordinates with no legal map cell. Record their stand-in reading and entry outcome:
selection must agree with that outcome, not an extra nonexistent-cell guard. Ordinary inside-area
occupants with no exit permission are expected to reject the blank level-0 stand-in. This does
not validate hypothetical exempt occupants; no such garrison fixture is supplied here.

For the different-permissions variant, use G7's two named occupier types, first one that can enter
a chosen terrain and then one that cannot. Make that legal cell the highest-ranked clear exit for
the first occupant. Expected: both living off-map occupants are placed at its centre, despite the
second type's entry refusal there. Loss of that second occupant solely for terrain permission, or
any extra standing-position draw, falsifies claim 22. For every valid part use three fresh games,
record health and off-map/map-marked state before release and positions immediately afterwards.
Exceptional inconsistent-state refusals and inactive-game invocation are not claimed tested.

**G17. Assault order and numbered points (claim 14).** Use G10's named assault map and normal
input, retaining its building-only transfer so A's identified occupants remain inside B's building.
Use one, two and three occupants entered individually in known order, each with distinct recorded
health or experience. Give the assaulter a repeating destruction counter with no destructive
side effects. Use a named test-art file with distinguishable small pixel pairs at points 0, 1 and
2. For each count prepare index 0 and, where possible, a different index by ordinary shots before
stopping the attack. After assault, record animation creation positions, destruction-counter
increments and each occupant's existence/removal state at those increments with a read-only
recorder. Tick-end video alone is insufficient for the credit-order part.

Expected: with three occupants, credit and removal identify positions 2, 1, 0 in that order.
at each notification the currently credited occupant still exists, and positions processed
at previous notifications have already been removed. Animation points are 2, 1, 0 respectively,
regardless of the starting firing index. With counts one and two the point sequences are 0 and
1, 0; the count comparison identifies each added last occupant's point without needing different
animation types. Repeat with point 2 omitted in the test art: the third occupant's animation uses
the absent default pair (0, 0), not a wrap to the configured nonzero point 0. Credit still follows
the same reverse list order. Use three fresh runs per variant. Any forward credit/removal order,
a point selected by the firing index, or wrap by configured-point count falsifies that part.
If event-time recording is unavailable, keep the removal/credit observation unmeasured rather
than infer it from simultaneous identical animations. Do not run unsupported point numbers as
a test of a promised safe result. Normal assaulter survival and reversed credit remain G10's
controls.

**G18. Death-removal safety departure (claim 23, our engine only).** Use G15's foreign-owned
building and defeat event, retaining surviving contender B. After A's three occupants die,
expect count 0 and index 0 in that same update, no occupant weapon or occupant experience
recipient, no later placement for those occupants, and claim 8's empty return at the building's
turn when a civilian house exists. This deliberately differs from G15's original retained n.

Also use deterministic engine fixtures with three identified occupants in entry order U, V, W,
index 1, and an independent death of V while inside. Expect remaining order U, W, count 2,
index 1 and W's weapon at muzzle 1 on the next ordinary shot. That successful non-disc-laser
shot advances the index to 0. In a separate fixture start at index 2 and kill V: expect index 0,
not index 1 as modulo reduction would give. Repeat the latter with W dying instead to check
last-place removal. Test both defeat hits and an independently supplied occupant death event.
These engine fixtures do not claim an original mixed-house entry setup. For experience, resolve
a credited destruction while the repaired index names W, before a new launch: the ordinary
occupant-recipient branch gives W the award, and a trainable-building control takes it itself
under S-0311 claim 8. Evict afterwards with free exits: only U and W are placed in reverse order.
Record counts, identities, index, weapon, muzzle, launch success, experience and placements.
Repeat three fresh runs per case. A retained dead place, changed survivor order, out-of-range
index, removed-place shot/award/placement, or incorrect abandonment falsifies the departure.
No unsafe original firing, credit or eviction continuation is proposed as a control.

**G19. Assault animation safety departure (claim 25, our engine only).** Use G10's named
assault fixture on our engine at game-speed index 2, with its isolated bystander, no engine
add-on and no mods (the named test files of a scenario excepted). In `garrison-assault-large.map`, give the building
an invented occupant capacity of 21 and enter identified occupiers individually. Use G17's
named test-art file with distinguishable pixel pairs for all ten points and a visible
occupant animation. Test counts 10, 11, 12 and 21 in three fresh games each. Transfer only
building ownership as in G10 and assault through normal input. Record list identities,
animation creation positions, credit notifications, removals and completed-action firing
index, using event-time records rather than just tick-end video.

Expected: positions 10, 11 and 20 use points 0, 1 and 0 respectively, projected as in claim 14.
Positions 0 through 9 still use their own numbers. Repeat the 12-occupant case with point 0
omitted while point 1 remains nonzero: position 10 uses the absent default pair (0, 0),
not another configured point. Repeat with different starting firing indices prepared by
ordinary shots, stopping fire before assault: these do not change animation positions.
Credit and removal remain in descending list-position order with each animation and credit
before that occupant's removal, and index 0 after completion. No animation at position 10,
a different selected point, dependence on configured-point count or firing index, or changed
credit/removal order falsifies the relevant rule. These are engine-only checks; do not run
positions 10 or later in the original as a purported safe control. G17 remains the supported-
point original comparison.

**G20. Owner change in the building's turn (claim 7, Q-2712).** Setup: a tiny skirmish map, a
civilian garrisonable building *B* of a civilian tech level, with three infantry units of the human
player's house *A* standing in range of an occupant's weapon, at game speed 6. No enemy object is in
range. Procedure: order one more infantry unit of *A*, of an occupier type, into *B*, and repeat
with its arrival falling at different points of *B*'s targeting timer. Record each tick, at the end
of each object's turn where the recorder allows it and otherwise at the end of the tick: *B*'s
owner, target and targeting timer, its fire events, the health of the three units and *B*'s
experience. Variants: (a) as described. (b) the three units belong to a house allied with *A*. (c)
as (a), with an enemy vehicle of raised health also in range. Runs: 10 per variant, and (c) repeats
until at least 5 of its runs have a restart of *B*'s targeting timer (a search of its own) in the
turn in which its owner becomes *A*. Expected: in (a) and (b) *B* never fires at the three units and
its experience does not change. In every variant, at the end of the turn in which *B*'s owner
becomes *A*, *B* holds no target, and in (c) its first fire event at the enemy vehicle comes after a
later search of its own (a restart of its targeting timer) or after a fire event of the vehicle at
*B* (retaliation, S-0309 claim 30). In each run of (c) with a search in the turn of the owner
change, *B* holds no target at the end of that turn, while a change before that search would let the
search, made for *A*, take the enemy vehicle and leave *B* holding it then. Falsified by a shot of
*B* at any of the three units, or by *B* holding a target at the end of the turn of its owner
change.

**G21. Refused computer-controlled occupiers (claims 1 and 4, Q-2926).** Setup: a tiny skirmish map
with one empty civilian building of imported capacity *c* and two test-only computer-controlled
player houses *C* and *D*, enemies of each other, in place of the bystander *B*; *A* only watches,
far from both. *C* starts with no building and no construction vehicle. *C* has *c* + 2 armed units
of one stock occupier type, placed by the map with a behaviour other than *hunt*, side by side a
few cells from the building, and *D* a single armed unit of its own several cells beyond it, on the
far side from *C*, of a type that cannot occupy a building, whose weapons have no berserk effect.
Procedure: a map trigger gives *C* the house-wide hunt action (claim 13) once, at a fixed tick from
tick 1 to tick 105; a later tick leaves more refusals before it, so tick 105 is preferred where the
map allows. Record, each tick and for each unit of *C*: its current, queued and suspended
behaviours, target, destination, position, every order or map action it receives, every hit it
takes, every override and revert (S-0215 claim 10), and its entry or refusal tick; record the
occupant count each tick and at the moment of each refusal, and the building's owner and health. A
run is valid when at least *c* + 1 of *C*'s units have *capture* forced with the building as
destination. A valid run is a race run when it has a tick at which the units of *C* on the map with
the building as destination outnumber its free places (*c* minus the count inside); occupants are
not counted, whatever destination they still record. A run is void when any unit of *C* receives an
order or map action other than the one hunt action and the house-wide hunt orders of *C*'s own
checks (S-0217 claims 2 to 4), when a unit of *C* takes a berserk hit, when the building's health
falls below its value at the start, or when the building's owner becomes a house other than *C* and
the civilian house. *D* is a test-only house too, so its checks send its unit hunting: a unit of
*C* that it destroys does not void the run and is judged only up to its destruction. *C* owns no
building later other than civilian garrison buildings its units occupy, so each of its checks gives
that order (S-0217 claim 4); *C*'s first check is at tick 0 and its second no earlier than tick 106
(S-0217 claim 2), so from tick 1 to tick 105 the trigger's action, at its fixed tick, is the only
house-wide hunt order *C* receives. The falsifier on the count is judged only up to tick 900: after
that, S-0217 claim 3 can give *C*'s occupied buildings a sell order, whose sale puts the occupants
out. Repeat until 10 runs are valid, and on until 3 are race runs or 30 runs are valid. M-0001's
arena set-up (side *B*), as Q-2926 proposes, may stand in for this map when *C*'s house there also
starts with no building and no construction vehicle and its record gives the same quantities under
the same void conditions. Expected: an arrival is refused only while the count inside is *c*. A
race run is the outcome for units on their way: more units are bound for the building than it has
free places, which a place held for each unit on its way would not allow. Race runs alone test that
part; with no race run, it is inconclusive, not falsified. From the tick at which *capture* is
forced on a unit to the tick before the trigger's tick, that unit has *hunt* current or queued
again only when a revert makes a suspended *hunt* current, or the *hunt* is the one that the tick-0
check gave it (that exception serves only a unit on which *capture* was forced in tick 0 before
*C*'s check: a later force removes that *hunt*, S-0215 claim 9), whether it was refused, lost its
target on the way or never arrived. From the trigger's tick on, its action and *C*'s checks give
*C*'s units *hunt* (claim 13, S-0217 claims 1 and 4), which S-0217 scenario H1 tests. Falsified by
a refusal at whose moment the count read is below *c*, or by *hunt* current or queued for a unit of
*C* on the map after *capture* was forced on that unit and before the trigger's tick, other than by
such a revert or by the tick-0 check. A unit refused at or after the trigger's tick is tested only
by the falsifier on the count: the trigger's *hunt* can reach it on its way and stay queued through
its refusal (S-0217 State and ordering). Only refusals before the trigger's tick test the absence
of *hunt* after a refusal, and a valid run with none leaves that part inconclusive, not falsified.
The behaviours after each refusal are recorded for the open question on them; only the falsifier on
*hunt* above tests them, and only before the trigger's tick.

## Open questions

- Q-0660's original index questions are answered by claims 18 and 19; the corresponding
  engine death-removal and index rule is claim 23, rather than original dead-place retention.
- Q-0970 is answered by claims 20 and 23: original unsafe retention remains unmeasured beyond
  G15's limited observation; the engine removes dead places, repairs the index and uses only
  remaining occupants for firing, experience and eviction. No lead decision is pending.
- Q-0971 is answered by claims 21 and 24: original unsafe ownership remains unmeasured, while
  our engine retains the owner silently on every missing-house turn without skipping forced
  evacuation. No lead decision is pending.
- Q-0972 is answered by claims 10 and 22: no independent legal-cell gate, ordinary outside-area
  rejection through the entry check, and no normal centre-placement crowding or terrain failure.
  Exception-state and off-map qualification observations retain their stated unmeasured limits.
- Q-0973 is answered by claim 14: reverse list-order removal/credit and point i for position i,
  without a configured-point-count modulus or participation by the firing index for supported
  original positions. Q-0990 is answered by claim 25's engine-only point i mod 10 for positions
  10 or later. The unsafe original result stays unmeasured; no lead decision is pending.
- Q-2926 is answered by claims 1 and 4 (units on their way, the searches of S-0309 claim 25 part
  (e), the unchanged target and no *hunt* after a refusal from S-0215's *capture*, *attack* and
  rest rules), and Q-2927 by claim 1 ("may occupy"
  in other specs).
- Q-2951 is answered in part, by claim 4 (the owner's checks give a refused unit *hunt*) and claim
  13 (the checks evict with hunting), with S-0217 claims 1 to 4 (r20). Which target each arena unit
  picks first, and so whether and when the units of the named pairings enter (its questions 1 first
  part, 2 and 3), is S-0309's (claims 17 and 25 part e) and is not predicted here; S-0217 scenario
  H1 records it. What a unit with *hunt* then does, and so whether it goes for the other group (its
  question 1), is S-0215's (its *hunt* rules), which this spec does not restate.
- Whether any skirmish cause other than map action code 6 and the original's computer player's
  checks (S-0217) sends the house-wide hunt order of claim 13.
- What a refused unit does after claim 4's scatter request while it keeps the building as target.
  While an accepted request's destination is held, S-0215 claim 28's third row changes nothing. When
  the unit refuses the request, or the request gives it no destination (S-0208 claims 11 to 13), the
  first row gives it the building as destination again at its next *capture* processing. What
  follows once a scatter destination is reached, and whether and how often the unit walks back to
  the building, is not established here. G21 records it.

## Changes for other specs

The current S-0308 claim 13, S-0313 claim 6 and S-0311 claim 15 already agree on completed-action
reset and claim 19's defeat exception. Earlier demands to change their reset ordering are satisfied.
The remaining coordination is:

- **S-0301 claim 7:** distinguish claim 20's original retained count from claim 23's engine
  count after same-update death removal; use the latter in the engine's delay quotient.
- **S-0308 claim 13:** retain claim 20's unsafe original description, but apply claim 23's
  stable death removal and index range repair in our engine. Weapon selection uses the
  repaired index among remaining occupants, with ordinary successful-shot advancement.
  No shot uses a removed place and an empty list supplies no occupant weapon. Update the
  index table and remove the pending-lead question.
- **S-0313 claim 6:** retain original numeric-index retention as claim 20's unsafe original
  case, but apply claim 23's engine removal, survivor order and index rule: keep k when
  k < new n, otherwise 0, including n = 0. Muzzle selection and successful-shot modulus use
  the repaired index and new count. Remove the pending-lead question and add G18's engine check.
- **S-0311 claims 8 and 15 and firing-occupant term:** distinguish claim 20's unsafe original
  recipient from claim 23's engine same-update death removal and index repair. Credit uses
  remaining occupants at the current repaired index under claim 8; no removed place receives
  experience, an empty list has no occupant recipient, and trainable-building self-credit
  is unchanged. Remove the pending-lead question. Claim 9 also needs claim 14's reverse-list
  assault credit/removal order.
- **S-0313 claim 6 and its assault measurement:** retain claim 14's original point i for
  supported positions 0 through 9 and unmeasured unsafe original result for positions 10 or
  later. Add claim 25's engine design rule: position i ≥ 10 uses point i mod 10 with the same
  lookup, projection and missing-point default, independent of configured-point count and
  firing index. Credit/removal order is unchanged; include G19's engine-only check.
- **S-0209 claim 4:** cross-reference claims 10 and 22 for this exit-search use. Do not add a
  blanket missing-cell refusal or a later-occupant terrain test to garrison release.
