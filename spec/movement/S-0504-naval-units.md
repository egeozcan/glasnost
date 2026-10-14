---
id: S-0504
title: Naval units (ship locomotion, wakes, sinking, underwater vessels, docking at yards)
topic: movement
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0504: Naval units (ship locomotion, wakes, sinking, underwater vessels, docking at yards)

## Summary

How vessels move and what is particular to them once they are on the water: the ship locomotor
(which reuses the driving rules of S-0207 with three differences), the wakes a moving vessel
leaves, what a sinking vessel does between its killing hit and leaving play, what the underwater
flag does and does not change, and the naval rules of docking at a building, including the
distance at which a repair yard starts on a vessel.

This spec does not repeat what other specs own. Entering water, the land-type and zone rules and
ship substitutions are S-0211 (claims 10 to 12) and S-0209 (claim 5f). The naval weapon choice is
S-0308 (claim 11 and table N). Release of a new vessel from a yard is S-0404 (claim 13) and S-0417
(claim 9). The sinking condition at a killing hit is S-0312 (claim 10). A vessel's crush handling
is S-0210 (claim 9). Cloaking, cloak timing and detection are S-0516; this spec only says which
naval states those rules produce (Terms, *submerged*). The squid's grappling is S-0614.

**Carrier and missile vessels.** A vessel that launches spawned aircraft (a carrier) or spawned
missiles (a dreadnought) uses the spawn rules of S-0501 (spawned aircraft: launch, sortie, return,
replacement) and S-0506 (spawned missiles: availability, launch, flight). The only naval-specific
rule for them is the launching hold of claim 3; the firing check's *busy* answer while launching is
S-0301 (claim 9, rank 2).

## Terms

- **Ship locomotor.** The locomotor whose class identifier is the "ship" row of S-0207 table T3.
  Every stock vessel that is not a hovering craft has it. A *ship* in this spec is a vehicle with the
  ship locomotor.
- **Naval vehicle, underwater type.** As S-0303 (Terms): a vehicle whose type has the naval flag,
  and a type with the underwater flag.
- **Submerged.** An underwater-type vehicle whose cloak state is cloaking, cloaked or uncloaking
  (the same states S-0308 table N tests). Submerging and surfacing are therefore the cloak
  transitions of S-0516; this spec adds no state of its own. An underwater type without the
  ability to cloak is never submerged (Edge cases).
- **Wake.** An instance of the game-wide wake animation (Import mapping) created by a vessel on the
  water (claims 2 and 4). It uses ordinary animation rules (S-0109); these claims add no
  wake-specific effect on other objects.
- **Sinking.** As S-0207 (Terms): the state of a killed vessel that goes down in the water while it
  stays in play. S-0312 (claim 10) gives the condition at a killing hit; claims 4 and 5 here give
  what happens while it lasts.
- **Sinking descent.** The lowering of a sinking vessel by a fixed amount each tick until it leaves
  play (claim 4).
- **Docking answer.** As S-0214 (Terms): the answer a building gives a unit that asks whether it
  may enter or dock.

## Behaviour

### Ship locomotion

1. **A ship moves by the driving rules.** A ship moves, turns and stops exactly as a drive vehicle
   of S-0207 with the same type fields on ordinary travel tracks, except for claims 2 and 3 and
   the ship crush handling of S-0210 claim 9. This does not assert equivalence for special exit
   tracks. These rules cover skirmish; network games and campaigns are out of scope. In particular these S-0207 rules
   hold for it unchanged: the speed and progress per tick (claims 1 to 4), height and facing on a
   track (claim 4), turning in place before a step (claim 6, with S-0206 claim 12), the choice of
   track from tables T1 and T2 (claim 7), no chaining of curves for ordinary vehicles (claim 8),
   the target fraction and the speed-fraction update with the same cases and constants (claims 9
   and 10), stopping (claim 11), ticks and track boundaries (claim 12), the speed power-up (claim 13),
   the outcomes of the step check (claim 15, with the crush mode of S-0210 claim 9 in place of the
   driving one), scatter requests (claim 16), the lead speed (claim 18), the approach to a target
   (claim 19) and cell-tag arrival timing (claim 22). A ship's *heading point* (S-0207 Terms) is the
   driving one: the track target of the track it is on, from the tick the track begins until the
   tick it ends, and none while it stands still, turns in place, waits or is between tracks.
   A ship stays in the ground layer, or the bridge-deck layer on a deck, by S-0207's rules; an
   underwater type is in the same layer as a surface ship (claim 6). Its body tilts on slopes as
   S-0106 claim 5 says for naval locomotion.
   - Backing: `measurable` (N1 and N2) for ordinary progress, turning, stopping, heading-point
     scatter and approach. The remaining shared-rule assertions are `unmeasured` here: those
     scenarios do not isolate power-ups, lead speed, cell-tag timing, slope tilt or every entry
     outcome; the cited specs' scenarios need ship-specific counterparts.
   - Confidence: high for shared ordinary rules and tables; medium for lead speed, approach
     and cell-tag timing

2. **Wakes behind a moving ship.** In each tick in which a ship is *moving now*, and the tick number
   (glossary) is divisible by 8, the ship creates one wake at its own position, provided all of these
   hold: its type is not an underwater type, it is not on a bridge deck, the land type of its cell is
   *water* (S-0211), and the game-wide wake animation is defined. A ship is *moving now* when its
   body is turning, or when it has a heading point and its current speed is above 0. A wake takes no
   draw of its own beyond those its animation type takes when it is created (S-0109 claim 3).
   - Backing: `measurable` (N3, including its named no-animation variant)
   - Confidence: high

3. **Launching hold.** A ship that is launching (S-0301 Terms *launching*: one of its spawned aircraft
   or missiles is still being launched, S-0501 and S-0506) does not begin its next step: between
   tracks it keeps waiting, with its destination kept, until it is no longer launching, and then goes
   on by claim 1. A ship already on a track finishes that track first. A carrier given a move order
   while it launches therefore leaves only after the launch. The driving locomotor holds in the same
   way.
   - Backing: `measurable` (scenario N4)
   - Confidence: high for the ordinary hold; interaction with concurrent waits remains open

### Sinking

4. **Sinking descent and leaving play.** While a vessel is sinking, each of its own updates (once per
   tick, S-0203) first lowers it by 5 leptons. If its height above the terrain is then below
   −400 leptons (strictly), it leaves play at once (S-0312 claim 15 says what that implies). Its
   destruction was already counted at its killing hit (S-0312). Otherwise, in a tick whose tick
   number is divisible by 4, it takes two ranged draws over [−170, 170] from the scenario random
   stream (S-0202 claim 4) and creates one wake at the displaced point of F2, at the water surface.
   A vessel that starts sinking at height 0 therefore leaves play at its 81st descent (F1), 80 ticks
   after its first descent, and creates 20 wakes in all with two draws each, since its 80 descents
   that leave it in play fall on 80 consecutive tick numbers. Its first descent comes in its own
   update after the killing hit: in the tick of the hit when the vessel's turn (S-0203) comes after
   the hit, otherwise in the next tick.
   When no wake animation is defined, our engine still takes the two draws on such a tick and
   creates no wake, and the importer warns (a design choice: the original's behaviour without a
   wake animation was not established, and stock data always defines one).
   - Backing: `measurable` (scenario N5; the draws with the random-state recorder)
   - Confidence: high for the descent, the threshold, the draws and their order; medium for the tick
     of the first descent, which follows the update order rather than a separate reading

5. **A sinking vessel in play.** A sinking vessel:
   - makes no progress by its locomotor, including its locomotor's turning and speed-fraction
     updates (so S-0207 claim 10's sinking case is never reached for it). This does not freeze
     other updates or guarantee that every facing change stops,
   - gets the answer *illegal* from the firing check whenever it would fire, a reason of S-0301
     claim 9 rank 4 (the firer's state),
   - cannot be newly assigned as a target: assigning it leaves no target, subject to the
     redirected-target rules of S-0309 claim 24. This does not itself establish when a target
     already held before sinking is cleared,
   - while it is still asked about as a target, gives the firer the answer *illegal* among the late
     target tests of S-0301 claim 9 rank 9, so a firer whose rearm countdown runs, that is out of
     ammunition or that is cloaked gets that answer instead,
   - retains its horizontal position during ordinary descent. Continued blocking in every
     occupancy test is not established here.
   Other events that set a vehicle sinking (a jumpjet falling into water, a teleport arrival on
   water, a squid's kill) belong to S-0502, S-0207 and S-0614; once set, claims 4 and 5 apply to it
   in the same way.
   - Backing: `measurable` (N6) for firing prevention, target clearing and horizontal position.
     Exact precedence of unsuccessful firing answers is `unmeasured`: no setup is established
     that retains a sinking target through earlier tests and exposes the answer by normal input.
     Unchanged speed fraction is `unmeasured`: sinking prevents independent travel-speed tests.
   - Confidence: high for locomotor suppression, firing checks and target clearing; medium for
     horizontal position

### Underwater vessels

6. **What the underwater flag changes.** An underwater type moves by claims 1 and 3 exactly as a
   surface ship, is in the same layer and occupies cells in the same way, and has no depth of its own.
   Its effects covered here are: it makes no wakes while moving (claim 2), it does not sink
   (S-0312 claim 10), and it is tested by area damage (S-0303 claim 21), the naval weapon choice
   (S-0308 table N, which also tests whether it is submerged) and the threat search (S-0309 F1).
   Whether other objects see it while it is submerged is S-0516's cloak rule.
   - Backing: `measurable` (N1 and N2 repeated with an underwater ship for movement; N3 for
     wakes). Damage, targeting, death and detection inherit the cited claims' backings. Separate
     depth and cell-occupancy equivalence are `unmeasured` here: no scenario isolates height or
     competing occupancy tests.
   - Confidence: high

### Docking at buildings

7. **Naval match.** A building refuses a unit that asks to enter or dock (its docking answer is
   *refuses*) when exactly one of the two, the building and the unit, has the naval flag, unless the
   unit's movement zone is *amphibious*. So a vessel cannot dock at a land repair depot, a land vehicle
   cannot dock at a naval yard, and an amphibious craft can dock at either. The building's other
   reasons to refuse (alliance, construction, power and the like) are unchanged.
   - Backing: `measurable` (scenario N7)
   - Confidence: high

8. **Repair at a yard, and the starting distance.** A naval yard whose type has the repair flag
   repairs vessels by the depot rules of S-0314 (claim 6). Before its settling phase a depot (land
   or naval) with a docked vehicle *waits for it*: at each of its processings (S-0215 claim 2) it
   asks whether the vehicle stands still and measures its spatial distance to the vehicle's
   docking point at the building, not necessarily the building centre (F3). When the vehicle stands still
   and is closer than the starting distance, 200 leptons for a vehicle with the ship or the hovering
   locomotor and 100 leptons for every other locomotor (strictly less in both cases), the depot ends
   waiting and its next processing comes 3 ticks later. At that processing it takes the settling
   step of S-0314 claim 6 if the vehicle is closer than 200 leptons, whatever its locomotor.
   Otherwise its processing delay is the depot's processing delay *D* of S-0314. So the settling
   step comes 3 ticks after the processing that first finds the vehicle at rest within the starting
   distance, not in the tick it comes to rest.
   - Backing: `measurable` (N8) for thresholds, spatial distance and delays. Complete docking-point
     selection is `unmeasured` here: no scenario covers every building and dock-place combination.
   - Confidence: high for thresholds, locomotor test, spatial distance and 3-tick delay,
     medium for *D* while waiting and the exact docking point at each yard

## Formulas

**F1. Sinking descent (claim 4).** Integer leptons. With *h*₀ the vessel's height above the
terrain when it starts sinking, after its *n*-th descent

> *h*ₙ = *h*₀ − 5*n*.

It leaves play at the first *n* with *h*ₙ < −400, that is *n* = ⌊(*h*₀ + 400) / 5⌋ + 1 for
*h*₀ ≥ −400 (floor of a non-negative quotient). For *h*₀ = 0, *n* = 81.

**F2. Sinking wake position (claim 4).** Integer leptons. With (*x*, *y*, *z*) the vessel's position
after the tick's descent and *h* its height above the terrain then, the first draw gives *r*₁ and
the second *r*₂, each a ranged draw over [−170, 170] inclusive:

> wake position = (*x* + *r*₂, *y* + *r*₁, *z* − *h*).

The first draw displaces along the map's *y* axis and the second along *x*. *z* − *h* is the height
of the terrain (the water surface) under the vessel.

**F3. Depot distance (claim 8).** Vehicle position (*x*, *y*, *z*) and its docking point
(*a*, *b*, *c*) have integer-lepton coordinates. The original evaluates squared differences,
their sum and the square root in floating point, then truncates toward zero to integer leptons:

> *d* = trunc(√((*x* − *a*)² + (*y* − *b*)² + (*z* − *c*)²)).

Height counts. This is not the table square root of S-0204. The building may supply different
docking points for different dock places or approach sides; complete point selection is open.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ship wake period | 8 | ticks (tick number divisible by) | Claim 2 |
| Sinking descent | 5 | leptons per tick | Claim 4 |
| Sinking removal height | −400 | leptons (strictly below) | Claim 4 |
| Sinking wake period | 4 | ticks (tick number divisible by) | Claim 4 |
| Sinking wake displacement | [−170, 170] | leptons per axis, inclusive | Claim 4, F2 |
| Depot starting distance, ship and hovering locomotors | 200 | leptons | Claim 8 |
| Depot starting distance, other locomotors | 100 | leptons | Claim 8 |
| Depot settling distance | 200 | leptons | Claim 8 |
| Waiting-to-settling delay | 3 | ticks | Claim 8 |

## State and ordering

| State | Event | Result |
|---|---|---|
| ship, moving now | tick number divisible by 8, conditions of claim 2 hold | one wake at its position |
| ship, between tracks | it is launching | keeps waiting (claim 3) |
| alive vessel | killing hit, S-0312 claim 10 holds | sinking at 1 health point, in play |
| sinking | own update | descends 5 leptons; then leaves play if below −400 leptons, else on a tick number divisible by 4 two draws and one wake |
| sinking | firing check reaches its sinking test | *illegal*, subject to claim 5's earlier answers |
| docked vehicle at a depot | stands still and is closer than the starting distance | settling scheduled 3 ticks later (claim 8) |

The observable consequences within a sinking vessel's update are that its height is 5 leptons lower, and, if it leaves play on that update, it takes no wake-displacement draws on that tick. If it remains in play on a tick divisible by 4, it takes the two wake-displacement draws during its own update. Relative to other objects' draws, these draws follow the update order of S-0203.

## Edge cases and quirks

- **Underwater type without the cloak ability.** It is never submerged, so table N of S-0308 treats
  it as a surfaced underwater target, but it still makes no wakes and does not sink. No stock type
  is like this. Behaviour as stated; no switch.
- **Driving and ship wake cadences differ.** The driving locomotor makes wakes on water every 10th
  tick and does not test the underwater flag (a matter for S-0207), the ship locomotor every 8th
  tick and only for non-underwater types. Both are kept as stated.
- **Concurrent launch and other waits.** Claim 3 specifies the ordinary launch hold only.
  Interaction with other reasons to cancel or delay movement is not established as an observable
  difference between ships and drive vehicles (Open questions).
- **Wake draws without a wake animation.** With no wake animation defined, a sinking vessel still
  takes its two draws every fourth tick and creates no wake (claim 4). This is our engine's choice
  for a case stock data never produces. No switch.

No behaviour of this spec has a quirk switch.

## Import mapping

| Key | Concept | Parse form | Absent |
|---|---|---|---|
| Rules, type section, `Locomotor` | ship locomotor (claim 1; S-0207 claim 14 owns the field) | class identifier | S-0207 |
| Rules, type section, `Naval` | naval flag (claim 7; S-0303 and S-0312 also use it) | boolean | no |
| Rules, type section, `Underwater` | underwater type (claims 2, 6) | boolean | no |
| Rules, type section, `Cloakable` | whether an underwater type can be submerged (Terms; S-0516 owns it) | boolean | no |
| Rules, type section, `MovementZone` | the *amphibious* exception of the naval match (claim 7; S-0211 owns the field) | movement zone name | S-0211 |
| Rules, `[General]`, `Wake` | the game-wide wake animation (claims 2, 4) | animation type name | none (engine default): a ship makes no wakes (claim 2); a sinking vessel takes its draws and makes no wake, with an importer warning (claim 4) |
| Rules, building section, `UnitRepair` | repair flag of a depot or yard (claim 8; S-0314 owns it) | boolean | no |
| Rules, building section, `Naval` | a naval building, for the naval match (claim 7) | boolean | no |
| Rules, type section, `Spawns` | the spawned aircraft or missiles whose launch makes the hold of claim 3 (S-0501, S-0506) | type name | none |
| Rules, `[General]`, `ShipSinkingWeight` | sinking condition (S-0312 claim 10 owns it) | decimal | S-0312 |

## Measurement scenarios

Stock data, director and mods off, a small purpose-made skirmish water map unless stated.
Use game-speed index 2 and one run per deterministic sub-test, with the glossary's isolated
bystander computer player and void rule. Repeat N5 four times to vary first-descent tick modulo 4.
Use tick-labelled frame capture and a read-only object recorder for positions, health, targets
and removal, and the random-state recorder where named. Record actual ticks, not assumed seeds.
N8 uses the processing delay at this speed (S-0215); speed can affect that delay.

**N1. Ship crossing time and path (claim 1).** *Setup:* open water, a stock ship and, on a land strip
of the same length, a drive vehicle; record both types' speed and turn fields from the data.
*Procedure:* order each straight along 10 cells, then along a path with one 45° and one 90° turn.
*Observable:* the tick of each cell arrival and the body facing per tick. *Expected:* the ship's
arrival ticks match S-0207 claims 2 to 12 computed from its own fields, as for the drive vehicle,
the turn in place before a step follows S-0206 claim 12; no tilt on water. *Falsified if:* an
arrival differs by a tick or more from the S-0207 prediction for the ship's own fields.

**N2. Ship scatter and approach (claim 1).** *Setup:* a ship in *guard*, an enemy ship placed
out of range. *Procedure:* (a) issue a scatter on a moving ship mid-track; (b) order an attack.
*Observable:* the cell around which the scatter destination is chosen; the tick the ship stops at
range. *Expected:* (a) the cell of the track target; (b) S-0207 claim 19's stop rule. *Falsified
if:* the scatter centre is the ship's own cell while it is on a track, or the stop tick differs.

**N3. Wake cadence (claims 2, 6).** *Setup:* one surface ship and one underwater ship moving on
open water; a bridge over part of the path. *Procedure:* run 200 ticks of straight movement and a
turn in place. *Observable:* the tick numbers at which wakes appear for each vessel (frame capture
per tick or the object list), and whether one appears while turning in place, under the bridge and
on the deck. *Expected:* surface ship: wakes exactly at tick numbers divisible by 8 while moving or
turning, none on the deck; underwater ship: none. *Falsified if:* any wake at a tick number not
divisible by 8, any missing eligible wake, any wake on a deck, or any wake from the underwater
ship. Wakes below a bridge remain eligible. If no stock ship reaches a deck, use named map test
data placing a ship-locomotor vehicle there; failure to reach a deck is inconclusive. Compare
stream states around wake creation with S-0109 claim 3's animation draws: an additional draw
falsifies claim 2's draw statement. Repeat surface travel with named map test data that leaves
no game-wide wake animation defined: no moving wake should appear, and any such wake falsifies
the missing-animation guard. This variant does not test sinking.

**N4. Launching hold (claim 3).** *Setup:* a stock carrier with its aircraft docked, an enemy
target in range. *Procedure:* order an attack, then while the first aircraft launches order a move
by scripted input. *Observable:* the tick the carrier begins its first track, and the ticks of each
launch. *Expected:* the first track begins only after the last launching aircraft has left.
*Falsified if:* the carrier begins a track while an aircraft is still launching. Repeat with a
missile vessel and with launch initiated during travel: the existing track must finish, and the
next must wait until launch ends. Pausing mid-track or never resuming afterwards falsifies claim 3.
A move order refused by S-0506 claim 10's movable test does not test an accepted move's hold.

**N5. Sinking duration and wakes (claim 4, F1, F2).** *Setup:* a stock surface vessel that meets the
sinking condition, at height 0 on open water; a weapon that kills it in one shot. *Procedure:* kill
it; run the random-state recorder. *Observable:* the tick of the killing hit, the tick it leaves
play (it disappears from the object list), the wakes created and their positions relative to the
vessel, and the number of scenario-stream draws per tick. *Expected:* it leaves play at its 81st
descent; wakes only on tick numbers divisible by 4, each within 170 leptons of the vessel on each axis,
exactly two extra draws on each such tick, the first fixing the *y* displacement. *Falsified if:* the
duration differs, a wake falls outside the box, a draw predicts the wrong axis or position, or
the draw count differs. Also record height each tick: descent other than 5 leptons, removal at
−400 rather than below it, other than 20 wakes, or a first descent not in the vessel's first
update after the hit falsifies claim 4. Exclude animation-owned draws from the two displacement
draws. The no-animation rule is a design choice, not a stock measurement.

**N6. A sinking vessel cannot fire or be fired on (claim 5).** *Setup:* a sinking vessel with an
enemy in range and a second enemy already targeting it before the killing hit. *Procedure:*
kill it, observe through removal, and try to retarget it by scripted input. *Observable:* new shots
from or at it, successful target reassignment and horizontal position. *Expected:* no new
shots pass either sinking test, a new assignment leaves no target by S-0309 claim 24, and
horizontal position stays unchanged. Already airborne projectiles are not new firing attempts. *Falsified
if:* a new shot passes either sinking test, a successful assignment retains the sinking target,
or it travels horizontally. If the click produces no order, that attempt does not test target
assignment; use S-0309 claim 24's guard-search variant instead. This does not test every competing firing answer or every occupancy use.

**N7. Naval match (claim 7).** *Setup:* a land repair depot and a naval yard with the repair flag,
a damaged drive vehicle, a damaged ship and a damaged amphibious hovering craft. *Procedure:* order
each to dock at each building. *Observable:* whether the vehicle docks and is repaired. *Expected:*
drive vehicle at the depot only; ship at the yard only; hovering craft at both. *Falsified if:* any
other pairing docks or a listed pairing is refused.

**N8. Starting distance (claim 8).** *Setup:* a damaged ship at a yard; a damaged amphibious hovering
craft and a damaged drive vehicle at a land depot. *Procedure:* dock each; read positions per tick.
*Observable:* the first tick at which the vehicle stands still within 200 (ship, hovering craft) or
100 leptons (drive vehicle) of its docking point (F3), the waiting-to-settling tick from a read-only recorder, and first repair
(health and credits change). Give its owner ample credits. *Expected:* the first repair comes
3 ticks after the successful waiting gate, provided contact persists and distance remains below
200 leptons; unsuccessful waiting processings recur at *D*. *Falsified if:* the gate accepts a
drive vehicle at 100 leptons or more, refuses a still ship from 100 inclusive to 200 exclusive,
accepts exactly 200, or the gate-to-settling interval differs from 3 ticks. For strict boundaries
and height, use named map test data with docking-point displacements giving rest distances just
below, exactly at and just above each threshold, and repeat with a height difference. A result
inconsistent with F3 falsifies spatial distance. Failure to establish a still docked vehicle at
the intended distance makes that sub-test inconclusive.

## Open questions

- Complete docking-point selection at each naval yard. Claim 8 uses spatial distance to that
  point, not to an assumed building centre.
- Whether the precedence of the launching hold (Edge cases) can be observed in any situation; until
  then either order is acceptable.
- Whether a sinking vessel keeps its cell's occupancy until it leaves play in every respect (claim 5).
- The driving locomotor keeps a few tracks beyond S-0207's tables (for example for a factory exit)
  that the ship locomotor does not have. It is believed that no ship ever selects them, since ships
  leave a yard by S-0417 claim 9; not exhaustively checked.
