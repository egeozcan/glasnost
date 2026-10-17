---
id: S-0606
title: Vehicle bunker
topic: structures
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: []
supersedes: []
---

# S-0606: Vehicle bunker

## Summary

This spec covers vehicle eligibility, approach, entry tracks and wall effects, occupancy and release
from a tank bunker in skirmish. Combat modifiers belong to the specs cited in claim 9; network games
and campaigns are out of scope. Entry permission and timing below describe a bunker without
another special docking, repair, reload, grinding or absorption role; combinations of those roles
have not been fully traced.

## Terms

- **Tank bunker, bunkered:** as in the glossary. Here *bunker* always means a tank bunker: a
  building whose type has the bunker flag (Import mapping).
- **Bunker link.** The occupancy relation between a bunker and one vehicle. A
  vehicle is *bunkered* exactly while it holds a bunker link, and a bunker is *occupied* exactly
  while it holds one. A bunker holds at most one bunker link (claim 3). The link is set only at the
  last entry stage (claim 6). Claims 12 to 14 give the established releases.
- **Bunker-eligible vehicle.** A vehicle that passes claim 1's test.
- **Entry stage.** The bunker's progress through one vehicle's entry: one of *clearing*,
  *waiting for clear cells*, *turning in*, *driving in*, *turning to face south*, *raising walls*
  and *occupied* (table S1). A bunker that is not taking in a vehicle is at *clearing*.
- **Reference point.** S-0204 claim 4's reference point of the bunker: the centre of its
  foundation.
- **Docking cell.** The cell that holds the vehicle's docking point (claim 4).
- **Entry track, release track.** The special drive tracks of tables T1 and T2. A vehicle on one
  of them moves under S-0207's track rules, with the speed rules of claim 6. *Track*, *track
  target*, ρ and ρₜ are S-0207's and S-0417's (claim 6).
- **Walls-up effects, walls-down effects.** The bunker's animation and sound changes when a
  vehicle becomes bunkered (claim 6, *raising walls* and *occupied*) and when it leaves in place
  or by its own *unload* (claims 12 and 14). *Special-1* to *special-4* are S-0108's animation
  slots. Each is started in its normal or damaged variant: the damaged variant when the bunker's
  health ratio is at or below the yellow condition ratio (Import mapping), as in S-0108.
- **Contact, passenger, garrison occupant, bunkered vehicle.** Four distinct relations:
  - A bunkered vehicle is also the bunker's *contact* (glossary). It keeps that contact from its
    entry request until a release ends it.
  - It is not a *passenger*: it stays on the map, on a cell of the bunker's foundation, drawn and
    able to fire.
  - It is not a garrison occupant (S-0508). It is not an absorbed unit.
  - A vehicle that is still entering holds contact but no bunker link. It is not yet bunkered.

## Behaviour

### Who may enter

1. **Bunker-eligible vehicle.** A vehicle is bunker-eligible when all of these hold:
   - its type has the bunkerable flag (Import mapping; absent-key default: on for vehicle types,
     off for every other kind),
   - its type has the turret flag,
   - it has a turret weapon: its *current weapon* (glossary) names a weapon type. For a type with
     several turrets this is the weapon of its current turret index; otherwise it is the first
     weapon. Elite replacement is per slot under S-0308 claims 1 and 2,
   - its locomotion class is not *hover* (S-0211's locomotion classes),
   - no parasite is inside it (S-0614).

   A vehicle that is not bunker-eligible never becomes bunkered. The flag is off by default for
   infantry, aircraft and building types. If data sets it on another kind, claim 2 may still grant
   permission, but such an object is never taken in: claim 6's *no vehicle* row sends the bunker
   back to *clearing*.
   - Backing: `measurable` (scenario K2). Permission for non-vehicle kinds is `unmeasured`:
     no normal order route establishing their entry request has been traced.
   - Confidence: high

2. **Permission.** A bunker answers an actor's permission query (S-0314 claim 16's vocabulary) as
   follows, first matching row wins:

   | Row | Condition | Answer |
   |---|---|---|
   | P1 | the bunker's house is not allied with the actor's house | no answer (the actor is not permitted) |
   | P2 | the bunker's current behaviour is *construction* or *selling* | refuses |
   | P3 | the bunker has not finished its build-up | refuses |
   | P4 | outside scenario initialisation, the bunker has no free contact place for the actor (contact places are max(*d*, 1) for the imported docking-place count *d*, S-0314 claim 17) | refuses |
   | P5 | S-0504 claim 7's naval-match test fails | refuses |
   | P6 | the actor's type has the balloon-hover flag | refuses |
   | P7 | the bunker is *switched off* (S-0405 Terms) | refuses |
   | P8 | the actor is not bunker-eligible (claim 1) | refuses |
   | P9 | the actor stands on a cell of this bunker's foundation | refuses |
   | P10 | otherwise | accepts |

   An unpowered bunker that is switched on still accepts. Low power alone is not a refusal.
   - Backing: `measurable` (scenarios K2 and K3); `unmeasured` for P4's initialisation
     exemption, because no player order can query permission during map loading, and for
     P3 independently of P2, because no unfinished build-up outside construction or selling
     has been established.
   - Confidence: high (rows P1 to P3, P6 to P10); medium (P4 and P5)

### Approach and entry

3. **Entry requests and capacity.** A vehicle in *enter* with a bunker as its target makes S-0215
   claim 29's repeated entry request. The bunker answers it as follows:
   - **Switched off:** refuses. S-0215 claim 29 then ends the vehicle's contact unless it is
     tethered.
   - **Already in contact with the actor:** refuses if the actor is no longer bunker-eligible,
     otherwise accepts.
   - **Not yet in contact, a contact place free:** the bunker takes the actor as a contact and
     accepts.
   - **Not yet in contact, no contact place free:** it still answers acceptance, but does not
     establish contact. This is not permission to enter and does not replace an existing contact.

   When a vehicle in *enter* that holds no contact is given a bunker as its destination:
   - if the bunker already holds any contact, the vehicle's destination is cleared and it does not
     move toward the bunker,
   - otherwise it makes one entry request at once. On acceptance the bunker stays its destination.
     On refusal it ends contact and its destination is cleared.

   Through this normal destination-setting route, a bunker has at most one vehicle approaching
   or inside, even when its docking-place count gives it more than one contact place. A second
   vehicle cannot displace the first. There is no waiting queue.
   - Backing: `measurable` (scenario K3); `unmeasured` for an entry request without contact
     when all places are full, because normal destination-setting prevents that request
   - Confidence: high (one approaching or bunkered vehicle); medium (the destination-clearing rule
     with more than one contact place)

4. **Docking point.** A vehicle that is approaching a bunker drives toward the docking point. Take
   the direction of the vehicle from the bunker's reference point (S-0204 claim 8, F3) and reduce
   it to a quadrant (Formulas F1). The docking point is the reference point plus the quadrant's
   displacement of ±128 leptons on each axis. For a 2 × 2 foundation this is the centre of the foundation
   cell on the vehicle's side. The direction is taken from the vehicle's position at the time the
   point is asked for, so the docking cell can change while the vehicle drives around the bunker.
   Which foundation cells may be entered is S-0209's (claim 5, row VP12).
   - Backing: `measurable` (scenario K1, approaches from four sides)
   - Confidence: high (formula); medium (when during the approach the point is re-asked)

5. **Arrival.** A vehicle arrives when all of these hold:
   - it is in *enter* (or in *patrol*, which the same test also accepts),
   - its contact is the bunker and the bunker is also its destination,
   - its cell is the docking cell.

   At arrival it ends its route, stops, and makes S-0215 claim 29's completion report. The bunker
   accepts the report unless its current behaviour is *selling*. On acceptance it sets its ready
   mark and queues *repair*, without an immediate start. The vehicle stays in *enter*: unlike at a
   repair depot it does not queue *sleep*.

   S-0215 claim 29's earlier guard transition also applies to a bunker. A contacted actor in
   *enter* that stands still within 64 leptons of the reference point makes a bunker in *guard*
   queue *repair*. A vehicle at a docking cell is about 181 leptons from the reference point, so
   this route does not apply to it there.
   - Backing: `measurable` (scenario K1); `unmeasured` for the earlier guard route, because
     normal arrival is outside its distance threshold and no separate reachable setup is known
   - Confidence: high (arrival conditions, the repair queueing); medium (the earlier guard route
     for bunkers)

6. **Entry stages.** While a bunker is in *repair*, each processing of that behaviour carries out
   at most one row of table S1 and chooses a delay of 15 ticks, with no draw, for a bunker
   without another repair, reload or absorption role. S-0215 claim 2 governs skipped turns and
   claim 3 governs a timer reset caused by a behaviour start. Uninterrupted entry needs at
   least six processings, plus whatever turns and drives the vehicle must finish in between. The
   vehicle being taken in is the bunker's linked vehicle if any, otherwise its first contact.
   - **No vehicle:** if there is no such vehicle, or it is not a vehicle, the bunker returns to
     *clearing* and queues *guard*, without an immediate start.
   - **Clearing cells:** the scatter requests of the *clearing* row go to the object nearest the
     centre of each foundation cell (the bunker itself and the entering vehicle excepted). Each
     request is forced and not move-anywhere, with the bunker's position as threat point. S-0215
     claim 17 owns how they are carried out.
   - **Turning in:** the desired body facing is (*d* + 32767) mod 65536, where *d* is the
     direction of the vehicle from the reference point (S-0204 claim 8, F3). This is nearly,
     but not exactly, a half-turn reversal. At a docking-cell centre its rounded facing selects
     the inward diagonal track of T1.
   - **Driving in:** the entry track is chosen from the vehicle's body facing rounded to 1/256
     turns (F2, table T1). It is forced toward the reference point as its track target. It sets
     both ρₜ and ρ to 1.0.
   - **Occupied:** the vehicle becomes bunkered (claim 7).
   - Backing: `measurable` (scenario K1, including K1b for the nearest-object quirk)
   - Confidence: high (stage order, the delay, the facings, the track choice, the effects), and
     medium (only the object nearest each cell centre is examined, Edge cases)

7. **Bunkered state.** On reaching *occupied*:
   - **Link:** the bunker link is set.
   - **Selection:** the vehicle leaves any control group it belonged to and is removed from the
     selection.
   - **Behaviour:** the vehicle queues *guard* with an immediate start.
   - **Sound:** the imported walls-up sound plays at the bunker's position, if one is set.

   While bunkered:
   - **Position:** the vehicle stands at the reference point, on a foundation cell, with body
     facing 32768 (map south) unless a later rule turns it.
   - **Behaviour:** it keeps *guard*. S-0215 claim 15 row 12 and claim 26 row 2 return it to
     *guard*.
   - **The bunker:** it stays in *repair*, with no further occupancy-stage effect, until a
     release. Claim 6's ordinary processing delay applies.
   - **Contact:** the vehicle stays the bunker's contact.
   - **Map presence:** it stays on the map, in play, and is not a passenger.
   - **Foundation:** every foundation cell of the occupied bunker is a blocking cell (S-0209 Terms,
     row VP12).
   - Backing: `measurable` (scenarios K1 and K6)
   - Confidence: high

8. **Clear report.** When a vehicle in contact with a bunker reports clear (S-0417 claim 7's
   report), the bunker answers as follows:
   - **Within 384 leptons:** if the spatial distance from the reference point to the vehicle is
     below 384 leptons, the bunker gives the accepting answer and contact is kept. The distance is
     computed in floating point and truncated toward zero, not with the table square root.
   - **Otherwise:** contact ends and the bunker gives the run-away reply (S-0417 claim 8's reply,
     with its destination rules).
   - Backing: `unmeasured`: no setup is known in which a vehicle in contact with a bunker, and not
     in *enter* or *unload*, makes a clear report
   - Confidence: medium

### While bunkered

9. **Firing and targeting from inside.** Being bunkered adds no exclusion or delay to the
   vehicle's ordinary per-tick firing step (S-0310 claim 9). Other inactivity rules still apply.
   Gattling spin follows S-0609 claim 5 with the same firing-check conditions as on open ground,
   not an extra bunker-specific update or a guarantee of a shot every tick. The
   vehicle acquires and attacks targets through its *guard* behaviour (S-0215). The other effects of
   being bunkered are owned elsewhere and not restated:
   - **Rate of fire:** S-0301 claim 7.
   - **Damage:** S-0302 claims 2 and 8.
   - **Range:** S-0307 claim 5.
   - **Turret rest:** S-0206 claim 9.
   - **Target redirection:** S-0309 claim 24. Attacks on the vehicle may be redirected to the
     bunker.
   - **Foundation passability:** S-0209 claim 5 row VP12.
   - **Deploy and unload exceptions:** S-0415 claim 1 and S-0517 claim 1.
   - **No parasite latch:** S-0614 claims 2 and 4.
   - **Death of a building contact:** S-0312 claim 17, with claim 13's exception.
   - **Erasure:** S-0514 claim 12.
   - **Behaviour returns:** S-0215 claims 15, 17 and 26.
   - **Ordinary mind control:** S-0601 claim 3 excludes a bunkered vehicle. The Psychic
     Dominator instead follows S-0612 claim 13: a reached, otherwise eligible vehicle is
     taken over without ending the bunker link (claim 15).
   - **Magnetic lift:** S-0607 claims 1 to 3. Locomotor-warhead attacks on a bunkered vehicle are redirected to
     the bunker (S-0309 claim 24).
   - **Drain of the bunker building:** planned S-0608; no drain rule is claimed here.
   - Backing: `measurable` (scenario K6)
   - Confidence: high

10. **No second bonus.** This spec adds no rate-of-fire, damage, range or armour rule of its own.
    Every numeric effect of being bunkered is the cited spec's.
    - Backing: `unmeasured`: a statement of scope, with nothing to observe
    - Confidence: high

11. **Selection and orders.** While the bunker link holds, the bunkered vehicle and occupied
    bunker are excluded from ordinary band-box selection (S-0214 claim 19). This is not a
    universal selection refusal: the bunker link alone does not exclude ordinary click selection.
    Other click-selection conditions still apply (S-0214 claim 12). The vehicle is not
    *movable* (S-0214 Terms), which does not by itself forbid selection or *deploy*.
    S-0214 claim 4 owns the action tables, including the bunkered vehicle's deploy action.
    An end-to-end player-input route from pointing at the occupant through selection to
    claim 12's *unload* has not been established here. Band-box exclusion is not evidence
    that no such route exists.
    - Backing: `measurable` (scenario K6); `unmeasured` for end-to-end unload-input
      reachability, pending the separate click and deploy checks in K6
    - Confidence: high (ordinary band-box exclusion and no link-only click exclusion),
      low (end-to-end unload-input reachability)

### Leaving

12. **Leaving by unload.** When a bunkered vehicle carries out *unload*:
    - the walls-down effects happen: *special-1* and *special-2* stop, the imported walls-down
      sound plays at the bunker's position if one is set, and *special-3* and *special-4* start,
    - the vehicle's link ends, and it is put on the release track (table T2) toward the point
      (−128, +128, 0) leptons from the reference point, with ρₜ and ρ set to 1.0, after
      switching its locomotion back on if it was off,
    - it gets as destination the result of S-0212 claim 25's nearby-cell search, centred on the
      cell one cell along −*x* and one along +*y* from the bunker's top-left foundation cell, with
      the vehicle's locomotion class, movement zone *normal*, no reach region, never in deck mode,
      bridge cells allowed and no other condition (the same settings as S-0212 T5's unit crate
      placement fallback row), and queues *move*, without an immediate start,
    - the bunker's link ends, it returns to *clearing*, queues *guard* without an immediate start,
      and ends contact with the vehicle.

    The release applies to the bunker whose foundation holds the vehicle's cell, which is its own
    bunker. A vehicle in *unload* that is not bunkered is not affected by this claim.
    - Backing: `unmeasured`: no input is known that gives a bunkered vehicle *unload* (claim 11)
    - Confidence: high (the effects, as behaviour of the original); low (reachability)

13. **Let out: bunker destroyed, sold or erased.** An occupied bunker lets its vehicle out at
    these moments:
    - **Destroyed:** within the hit that destroys it.
    - **Sold:** at the first processing of its *selling* behaviour (S-0408 claim 6).
    - **Erased:** at the completion of its erasure (S-0514 claim 12).

    Letting out:
    - puts the vehicle on the release track toward the point (−128, +128, 0) from the reference
      point, with ρₜ and ρ set to 1.0, after switching its locomotion back on if it was off,
    - ends the bunker link on both sides, and the bunker ends contact with the vehicle.

    It has no walls-down effects. The vehicle gets no new behaviour or destination. It keeps
    *guard* and stops at the release track's end on the bunker's former foundation.

    **Destruction ordering.** The contacts that S-0312 claim 17 treats are those the bunker held
    just before the destroying hit, less the bunkered vehicle. So the bunkered vehicle is let out
    and is neither destroyed nor told to run away by that claim. It is not immune to other
    damage from the same attack or from death effects. Any other contact is treated under
    claim 17. For a 2 × 2 bunker on flat ground, a vehicle still entering at a docking-cell
    centre (contact, no bunker link) is within 256 leptons and receives that claim's demolition
    damage, ordinarily killing it.
    - Backing: `measurable` (scenarios K4 and K5; S-0514 claim 12, scenario E9 for bunker
      erasure)
    - Confidence: high (destruction and sale); medium (erasure)

14. **Released in place: vehicle dies, is erased or is warped.** When a bunkered vehicle is destroyed, completes erasure (S-0514 claim 12), or joins a warp group (S-0513 claim 34), its link and contact have ended, the bunker is back at *clearing* with *guard* queued, and the walls-down effects have happened by the time the vehicle's other death effects are resolved:
    - the walls-down effects of claim 12 happen,
    - the bunker link ends on both sides, and the bunker ends contact with the vehicle,
    - the bunker returns to *clearing* and queues *guard*, without an immediate start.

    The vehicle is not moved by this rule.
    - Backing: `measurable` (scenario K5, vehicle death; S-0513 claim 34, scenario SW12 for
      warp); `unmeasured` for vehicle erasure, because ordinary temporal fire is redirected
      to the bunker (S-0514 claim 12)
    - Confidence: high (death); medium (erasure, warp)

15. **Ownership and other removal routes.**
    - **Either side changes owner:** the link holds. After a bunker capture, the vehicle stays
      bunkered under its own house in a bunker of another house.
    - **Removal by another route:** no additional reachable skirmish release route has been
      established. Behaviour of other removals is untraced, not a guarantee that a stale
      occupancy relation persists (Open questions).
    - **Other influences:** ordinary mind control cannot take over a bunkered vehicle
      (S-0601 claim 3), and a parasite cannot latch (S-0614 claim 2). The Psychic Dominator
      can take over a vehicle that its sweep reaches and that passes its remaining eligibility
      conditions (S-0612 claim 13). In a 2 × 2 bunker, the occupant is reached: its position
      coincides with the foundation centre, and the tied within-cell distance does not let the
      bunker hide it from the sweep. The vehicle changes owner without leaving the bunker or
      ending the link. Other foundation sizes retain S-0612 claim 13's conditional reachability.
    - Backing: `measurable` for either side's ownership change (scenario K7 and S-0612
      claim 13, scenario PD10); `unmeasured` for other removal routes, because no reachable
      independent setup has been established
    - Confidence: medium

## Formulas

**F1. Docking quadrant** (claim 4). With *d* the direction of the vehicle from the reference
point, an integer 0 … 65535 (S-0204 F3, computed in double-precision floating point there and
truncated toward zero):

> *q* = ⌊(⌊*d* / 128⌋ + 1) / 2⌋ mod 256 (integer, floor division of non-negative values)

| *q* | Displacement (*x*, *y*) from the reference point, leptons |
|---|---|
| 0 … 63 | (+128, −128) |
| 64 … 127 | (+128, +128) |
| 128 … 191 | (−128, +128) |
| 192 … 255 | (−128, −128) |

**F2. Entry track choice** (claim 6, *driving in*). With *f* the vehicle's current body facing
(0 … 65535):

> *r* = 2 × (⌊(⌊*f* / 128⌋ + 1) / 2⌋ mod 256), integer, with floor division throughout.

*r* / 2 is the facing in 1/256 turns, rounded half up. Table T1's rows are chosen by *r* / 2:
32, 96 and 224 select rows E1, E2 and E4. Every other value selects E3.

**F3. Turning-in facing** (claim 6): (*d* + 32767) mod 65536, integer, *d* as in F1 but measured
when the stage is reached.

**F4. Release point** (claims 12 and 13): the reference point plus (−128, +128, 0) leptons. Height
is the reference point's own. Coordinate additions in F1, F4 and T1/T2 are exact integer-lepton
operations, without fractional rounding.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Bunker entry processing delay | 15 | ticks | claim 6, no draw |
| Docking displacement | 128 | leptons per axis | F1 |
| Turning-in addend | 32767 | facing units | F3, exact (not 32768) |
| Bunkered body facing | 32768 | facing units | claim 7, map south |
| Clear-report keep distance | 384 | leptons | claim 8, strict, floating point truncated |
| Earlier guard distance | 64 | leptons | claim 5, from S-0215 claim 29 |
| Entry and release speed fraction | 1.0 | fraction | ρₜ and ρ, claims 6, 12 and 13 |
| Release point displacement | (−128, +128, 0) | leptons | F4 |
| Exit search centre displacement | (−1, +1) | cells | claim 12, from the top-left foundation cell |

**T1. Entry tracks.** The base list *B* has 16 points. Point *n* (*n* = 1 … 16) is
(−128 + 8*n*, 128 − 8*n*) leptons from the track target, with facing 32 (1/256 turns): from
(−120, 120) to (0, 0) in steps of (+8, −8). Each row transforms *B* by S-0207's transforms:

| Row | Body facing *r* / 2 | Transform of each point (*x*, *y*, facing *φ*) | First point | Facing on track |
|---|---|---|---|---|
| E1 | 32 | none | (−120, 120) | 32 |
| E2 | 96 | (*x*, −*y*), *φ* → (−*φ* − 128) mod 256 | (−120, −120) | 96 |
| E3 | otherwise (160 in practice) | (*y*, *x*), *φ* → (−*φ* − 64) mod 256 | (120, −120) | 160 |
| E4 | 224 | (−*x*, *y*), *φ* → (−*φ*) mod 256 | (120, 120) | 224 |

The track target is the reference point. No chain point and no early release point.

**T2. Release track.** 17 points, (*x*, *y*) leptons from the track target (F4), facing in 1/256
turns, untransformed:

| *n* | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| *x* | 128 | 124 | 119 | 115 | 111 | 106 | 101 | 96 | 91 |
| *y* | −128 | −112 | −96 | −80 | −64 | −57 | −50 | −44 | −37 |
| facing | 128 | 132 | 136 | 140 | 144 | 148 | 152 | 156 | 160 |

| *n* | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 |
|---|---|---|---|---|---|---|---|---|
| *x* | 84 | 77 | 70 | 64 | 48 | 32 | 16 | 0 |
| *y* | −32 | −27 | −22 | −17 | −12 | −8 | −4 | 0 |
| facing | 164 | 168 | 172 | 176 | 180 | 184 | 188 | 192 |

The first point is the reference point itself. The track ends facing 192 (map west). No chain
point and no early release point.

## State and ordering

**S1. Entry stages of a bunker** (claim 6). One row at most per processing of the bunker's
*repair* behaviour. A row whose event does not hold leaves the stage unchanged.

| Stage | Event | Next stage | Effect |
|---|---|---|---|
| clearing | the vehicle stands on a foundation cell of this bunker and is not moving | waiting for clear cells | scatter requests to the object nearest each foundation cell's centre, unless it is the entering vehicle (claim 6) |
| waiting for clear cells | the object nearest each foundation cell's centre is absent or is the entering vehicle | turning in | desired body facing F3 |
| turning in | the body is not turning | driving in | entry track by F2 and T1 toward the reference point; ρₜ = ρ = 1.0 |
| driving in | the vehicle stands on a foundation cell of this bunker and is not moving | turning to face south | desired body facing 32768 |
| turning to face south | the body is not turning | raising walls | *special-1* and *special-2* start (normal or damaged variant) |
| raising walls | always | occupied | bunker link set; group and selection cleared; vehicle queues *guard* with an immediate start; walls-up sound |
| occupied | any processing | occupied | nothing |
| any | no vehicle to take in, or it is not a vehicle | clearing | bunker queues *guard*, no immediate start |

**S2. Ending the bunker link.**

| Cause | Walls-down effects | Vehicle moved | Vehicle's new behaviour | Bunker afterwards |
|---|---|---|---|---|
| vehicle's *unload* (claim 12) | yes | release track, then *move* to the nearby-cell result | *move* | *clearing*, *guard* |
| bunker destroyed, sold or erased (claim 13) | no | release track | none (keeps *guard*) | being removed |
| vehicle destroyed, erased or warped (claim 14) | yes | no | n/a | *clearing*, *guard* |
| bunker changes owner (claim 15) | n/a: the link holds | no | no | still occupied |

Ordering consequences:
- In a destroying hit on the bunker, the let-out comes before S-0312 claim 17's treatment of
  contacts, which uses the contacts as they were before the hit, less the bunkered vehicle.
- A destroying hit on the vehicle leaves the bunker empty with walls-down effects and *guard*
  queued, even if the vehicle's death effects then damage the bunker. Queuing does not guarantee
  that *guard* is current in that tick (S-0215 claims 8 and 11).
- Within a processing of *repair*, a stage change takes effect at that processing. The next row
  can run only at a later processing, normally 15 ticks later absent an interruption or restart.

## Edge cases and quirks

- **Only the nearest object per cell is examined** (`quirks.bunker_clear_nearest_only`). The
  clearing and waiting rows look at the single object nearest each foundation cell's centre. When
  the entering vehicle is that object, a second object in the same cell is neither asked to scatter
  nor waited for, and stays on the foundation while the vehicle drives in. With the switch off,
  every object other than the bunker and the entering vehicle on each foundation cell gets the
  scatter request, and the waiting row waits until none is left.
  - Backing: `measurable` (scenario K1b)
  - Confidence: medium
  - Default: original behaviour (switch on).
- **A turreted vehicle without a turret weapon cannot enter**, even though its type is
  bunkerable (claim 1). This is by design of the test, not a bug, and has no switch.
- **Ordinary band-box exclusion** (claim 11) does not establish click-selection refusal or
  unload-input unreachability. Claims 13 and 14 are the established reachable releases here.
  Claim 12's end-to-end input route remains open. No selection quirk is introduced.
- **Foundations other than 2 × 2.** F1's ±128 displacement and T1/T2 assume a 2 × 2 foundation. For
  other sizes the docking point and release point need not be cell centres. The rules are still
  applied as written. The importer warns when a bunker type's foundation is not 2 × 2.
  - Backing: `measurable` (K1c for the coordinates); the warning is an importer design choice.
  - Confidence: high (fixed displacements); medium (completion on other foundations)
- **Unexpected removal.** No additional skirmish removal route has been established. This spec
  does not prescribe either persistence or cleanup for an untraced route.
- **Release with no nearby cell found** (claim 12): what destination the vehicle gets is not
  established (Open questions).
- **Release under a pulse** (claims 12 and 13). Letting out switches the vehicle's locomotion back
  on without checking for a pulse (S-0405 Terms). Whether other inactivity rules allow movement
  while disabled, or leave locomotion on until the pulse ends, was not traced (Open questions).
  No movement-while-disabled rule or quirk is prescribed here.
  - Backing: `unmeasured`: no independently traced pulse-and-release setup
  - Confidence: low

## Import mapping

| Source | Key | Feeds | Parse | Absent |
|---|---|---|---|---|
| Rules, building type sections | `Bunker` | the bunker flag (Terms) | boolean (S-0003) | off |
| Rules, actor type sections | `Bunkerable` | the bunkerable flag (claim 1) | boolean | on for vehicle types; off for infantry, aircraft and building types |
| Rules, vehicle type sections | `Turret` | the turret flag (claim 1) | boolean | off |
| Rules, vehicle type sections | `SpeedType` | locomotion class *hover* excludes (claim 1) | S-0211's list | S-0211's default |
| Rules, vehicle type sections | `BalloonHover` | balloon-hover flag (claim 2, row P6) | boolean | off |
| Rules, building type sections | `NumberOfDocks` | docking-place count *d* (claim 2, row P4) | integer | S-0314 claim 17's |
| Rules, building and vehicle type sections | `Naval` | naval-match test (claim 2, row P5) | boolean | S-0504's |
| Rules, `[AudioVisual]` | `BunkerWallsUpSound` | walls-up sound (claim 7) | sound name | none |
| Rules, `[AudioVisual]` | `BunkerWallsDownSound` | walls-down sound (claims 12 and 14) | sound name | none |
| Rules, `[AudioVisual]` | `ConditionYellow` | yellow condition ratio (Terms, walls effects) | as S-0108 | as S-0108 |
| Art, building image sections | `SpecialAnim`, `SpecialAnimTwo`, `SpecialAnimThree`, `SpecialAnimFour` and their `…Damaged` forms | *special-1* to *special-4* (S-0108) | as S-0108 | none |

Weapon slots and elite replacement use S-0308 claims 1 and 2's import mapping. The rate-of-fire,
damage and range keys of being bunkered are imported by the specs of claim 9.

## Measurement scenarios

General setup: stock data, no engine add-on and no mods (the named test files of a scenario excepted), except for the explicitly named test-map
fixtures below under the glossary's measurement-setup allowance. Use a tiny flat clear map, game
speed index 2, 800 × 600, one human house and one bystander computer player. The observed area
is the bunker and its approach lanes. Apply the glossary's bystander void rule, not an assumption
that the computer player gives no orders. Run three recorded seeds per case using scripted mouse and keyboard input and a read-only recorder of ticks, positions, facings, behaviours, contacts, occupancy, selection, animation and random-stream state. Record seeds, not fixed random-state writes.

- **K1. Entry timeline (claims 4 to 7, S1, T1).** Order one eligible tank into an own bunker
  from each of four quadrants, in separate runs. Put an own infantry unit on another foundation
  cell for a clearing run, and repeat without it. Give the tank a control group and select it
  before arrival. Record docking-cell changes, current and queued behaviours, turns, track
  target and traversed points, wall effects, sound, contact, group and selection.
  *Expected:* F1 docking point, arrival only with matching contact, destination and cell,
  *repair* queued on completion while the tank stays in *enter*, and the six transitions in
  S1's order, at most one per processing. In an uninterrupted run successive processings are
  15 ticks apart. The inward facing follows F3 and the entry points T1, ending at the reference
  point facing 32768. No occupancy relation exists before the last transition. The tank loses
  selection and group membership at that transition. The bunker remains in *repair*, the tank
  in *guard* and contact, without becoming a passenger. Repeat with a damaged bunker to test
  the damaged animation variants. Any missing effect, early occupancy, wrong point or facing,
  different spacing without a restart, or premature behaviour change falsifies the relevant
  part. Compare the recorded stream around stage transitions with a matched no-scatter control
  to distinguish scatter draws from the entry delay's no-draw claim.
- **K1b. Clearing overlap (claim 6, nearest-object quirk).** Arrange an entering tank nearer a
  foundation-cell centre than an infantry unit sharing that cell. Verify both positions with
  the recorder before the clearing processing. Compare with the infantry nearer the centre.
  *Expected with the quirk on:* the farther infantry is neither asked to scatter nor waited
  for when the tank is nearer. With infantry nearer, it is asked to scatter and entry waits
  while it remains nearest. Scattering or waiting for the farther infantry falsifies the
  nearest-only rule. If normal movement cannot establish the overlap, record this subclaim
  as unmeasured rather than inferring it from ordinary clearing.
- **K1c. Other foundations (Edge cases).** In `bunker-tests.map`, repeat K1 with a 1 × 1 and
  a 3 × 3 foundation, recording the reference point, docking point and any release point.
  *Expected:* the same fixed F1/F4 displacements, not displacements scaled with foundation
  size. Any size-dependent scaling falsifies the fixed-coordinate rule. Record entry failure
  separately: this spec does not promise successful entry on every foundation.
- **K2. Eligibility and permission (claims 1 and 2).** Compare an eligible turreted tank with
  stock non-turreted, hover and infantry actors, and a tank with an attached parasite. Also
  compare own, allied and enemy bunkers, a bunker during construction, one being sold, one
  switched off, and one merely in a low-power house but switched on. Try *enter* while the
  actor already stands on the foundation. Record cursor, permission answer where reached,
  behaviour, destination and eventual occupancy, rather than treating a cursor as the only
  evidence. For independent conditions lacking stock witnesses, the named fixture
  `bunker-tests.map` uses copies of the eligible type, each differing only in bunker eligibility,
  turret presence, presence of a turret weapon, hover movement, balloon-hover status or naval
  status. The same fixture supplies a bunker with a usable power switch. Include a vehicle
  type whose eligibility flag is omitted to test the default, an elite-only first weapon tested
  below and at elite rank, and a numbered-weapon type with present and absent current turret
  weapons. *Expected:* claim 1's conjunction
  and the first matching refusal in P1 to P9, with acceptance in P10, including low power alone.
  Acceptance with an isolated disqualifier, refusal of the baseline or low-power-only case,
  or occupancy by a disqualified vehicle falsifies the corresponding claim. Cases where no
  input reaches the permission query remain unmeasured, not confirmed refusals.
- **K3. Capacity (claims 2 and 3).** Order two eligible tanks toward one empty bunker in the
  same tick, then a third toward the occupied bunker. Repeat in `bunker-tests.map` with three
  imported docking places, and repeat with a bunker switched off between contact and arrival.
  Record destinations and contacts, not just final occupancy. *Expected:* the normal
  destination-setting route admits one approach, clears a newcomer's destination while any
  contact exists, and supplies no queue. Switching off refuses subsequent requests and ends
  untethered contact. Two approaches admitted by that route, replacement of the first tank,
  a retained waiting destination or continued acceptance while off falsifies the relevant part.
- **K4. Sale (claim 13, S2, T2).** Sell an occupied own bunker. Record the first selling
  processing, occupancy and contact changes, positions, behaviour, destination, wall animation
  and sound. *Expected:* release starts at that processing, follows T2 toward F4 and adds no
  new vehicle behaviour or destination. There are no walls-down effects. A different release
  tick or path, a newly assigned movement order, or walls-down effects falsifies the claim.
- **K5. Destruction and warp (claims 13 and 14).** (a) Destroy an occupied bunker with an
  attack whose damage and death effects do not reach the vehicle. Record all health changes,
  contact and occupancy changes and the release path. *Expected:* the tank survives and starts
  T2 in the destroying tick without contact-demolition damage. (b) Destroy a flat 2 × 2 bunker
  while its entering tank is at a docking-cell centre but not yet bunkered. *Expected:* that
  contact instead receives S-0312 claim 17's demolition hit. (c) Kill the bunkered vehicle
  using a bunker-penetrating warhead (S-0302 claim 8), recording walls, sound and queued
  behaviour. *Expected:* same-tick walls-down effects and cleared occupancy/contact, with
  *guard* queued for the bunker, and no release track for the dead tank. A new tank can then
  enter. (d) Use S-0513 claim 34, scenario SW12's bunkered-vehicle warp case and record the
  same walls-down effects and vacated bunker, without a release track before the warp.
  Demolition damage in (a), its absence in (b), retained occupancy/contact, missing walls-down
  effects in (c)/(d), or a release track in (c)/(d) falsifies the relevant part.
- **K6. Inside (claims 7, 9 and 11).** Place an enemy target in range of a bunkered eligible
  gattling tank, with an open-ground control of the same tank type. Record firing-check
  outcomes, spin value and stage each tick, and shot ticks separately. Compare only intervals
  with equal firing-check outcomes. *Expected:* S-0609 claim 5's spin change per eligible tick,
  not a shot every tick. Bunker-only missing spin updates falsify claim 9. Attempt click,
  band-box, type selection and group recall for both bunker and vehicle, plus movement and
  deploy input, recording the pointed object separately from the selected object and issued
  order. *Expected:* an ordinary band box adds neither, and the occupant is not movable.
  A click must not be refused solely because of the bunker link. Addition by an ordinary
  box or a link-only click refusal falsifies claim 11. For any successfully clicked occupant,
  try a self-click and the deploy command, recording the resolved action, queued behaviour
  and any claim 12 release. This settles unload-input reachability rather than presuming
  failure from the band-box test. If unload is reached, also record T2, the nearby-cell
  destination, walls-down effects and both links/contact to test claim 12. Record
  guard behaviour and map presence, and compare routes through the empty and occupied
  foundation as in S-0209 claim 5's scenario C23: occupied blocking cells must cause a bypass.
  Missing map presence, a passenger relation or a route through an occupied blocking cell
  falsifies the corresponding part of claim 7.
- **K7. Ownership change (claim 15).** Use a stock capturable bunker if available, otherwise
  `bunker-tests.map` changes only its engineer-capture eligibility. Have another house capture
  it while occupied, recording both owners and occupancy/contact state. *Expected:* the
  vehicle retains its owner and occupancy relation with no release. Changed vehicle ownership,
  a release or lost occupancy falsifies the claim. This tests a bunker owner change only.
  For vehicle ownership change, use S-0612 claim 13's scenario PD10 with a 2 × 2 bunker
  and an otherwise eligible occupant surviving the sweep damage. Record both owners,
  permanent control and both links/contact at the sweep. *Expected:* the vehicle changes
  owner, the bunker does not, and occupancy/contact persist with no release. A reached,
  eligible survivor keeping its former owner or losing the link falsifies the relevant claim.

## Open questions

1. **Input reachability (claim 11).** Ordinary band-box exclusion is established, not a
   universal selection ban. K6 must test pointing, ordinary click selection and deploy
   separately to establish an end-to-end player-input route to claim 12's unload. Other
   hotkeys, pre-existing selections and queued orders have not been exhaustively checked.
2. **No nearby cell (claim 12).** What destination the vehicle gets when the nearby-cell search
   finds no cell was not established.
3. **Pulse and release (Edge cases).** Can the vehicle actually advance on its release track
   while disabled, despite other inactivity rules? What happens to locomotion power after
   the track and at pulse expiry?
4. **Interruption during entry.** If a vehicle is given another order before *occupied* but keeps
   its contact, the bunker may keep waiting at its stage and refuse every other vehicle (claim 3).
   Not traced.
5. **Psychic Dominator (resolved).** S-0612 claim 13 establishes conditional sweep eligibility
   and reachability for a vehicle in a 2 × 2 bunker. Claim 15 records the retained link.
6. **Re-asking the docking point (claim 4).** Whether the docking point is re-asked on every
   route refresh or only when the route is built was not traced. The two give different docking
   cells for a vehicle that drives around the bunker.
7. **Other removals (claim 15).** No exhaustive trace establishes the occupancy outcome of
   every other removal route. Do not infer a stale relation merely from an absent release here.

## Questions for the project lead

- If K6 fails to establish the original's click-and-deploy exit route, should glasnost offer an
  explicit exit order invoking claim 12? This spec does not authorise an input departure.
