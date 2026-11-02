---
id: S-0610
title: Psychic weapons
topic: combat
revision: 12
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0661, Q-1166, Q-1180, Q-1526]
supersedes: []
---

# S-0610: Psychic weapons

## Summary

The psychic side of combat: the two psychic immunities that types carry, how psychic area pulses
are delivered, how the berserk state starts, what it forbids and how it ends, the psychic sensor
that shows the intent of hostile units near it, and the psychic reveal superweapon. The chaos
drone's berserk weapon is covered here rather than in a unit spec of its own. These claims cover
skirmish only; network games and campaigns are out of scope.

**What this spec does not restate.**
- **Damage resolution of psychic and berserk hits:** S-0302. The psychic-immunity row and the
  berserk row of claim 8's table, the berserk duration *T*, its re-hit reset and its exceptions
  (claim 10) are S-0302's.
- **Firing while berserk:** the halved rearm time is S-0301 claim 8.
- **Searches while berserk:** S-0309 claims 7 and 9 to 12, including the friendly-row exceptions
  and the permanent-berserk mark.
- **Orders to a berserk unit:** S-0214 claim 15 refuses them.
- **Firing from the deployed state** (the pulse of a deployed psychic infantry): S-0517 claim 16.
  Ordinary area damage of the pulse: S-0303.
- **The warning marker** that psychic detection shows for a nuclear strike: S-0513 claim 11.
- **Cloak sensors and disguise detectors**, a different coverage with different rules: S-0516
  claims 21 to 24 and 40.
- **Granting, charging, readiness and the aiming interface** of the psychic reveal: S-0512 claims 1
  to 8 and 16 to 18, and S-0513 claims 1 to 3 for aiming and the record.
- **The area reveal itself** (cells, radius cap, explore and see, gap fields): S-0409 claim 17.
- **Gas clouds** that carry a berserk warhead: S-0613 claims 4 and 13 cover their creation and
  hits when the data supplies them; each hit then follows this spec and S-0302 claim 10.
  Whether an area berserk weapon creates gas particles depends only on its imported data (claim 8).
- **Permanent and temporary mind control** use the psionics immunity of claim 1.
  Their eligibility rules are S-0601 claims 3 and 13. Psychic-domination eligibility is
  S-0612 claim 13.
- **Healing itself:** S-0314 (healers, self-healing, house-wide totals, repair depot, hospital,
  building repair), and the heal-all crate, which is S-0520 claim 14. Claim 6 says only how a
  berserk unit stands in them.

## Terms

- **Psionics immunity.** A type's flag that protects its objects from control and berserk: mind
  control, permanent control, psychic domination and berserk hits all leave them alone (claim 1).
- **Psychic-weapon immunity.** A separate type flag that protects only from the damage of
  psychic-damage warheads (claim 1). The two flags are independent.
- **Psychic-damage warhead.** A warhead whose hits a psychic-weapon-immune target ignores (S-0302
  claim 8).
- **Berserk-causing warhead.** A warhead whose hits put the target into the berserk state instead
  of damaging it (S-0302 claim 10).
- **Berserk selector, berserk mask.** The colour-adjustment table position that the rules choose
  for berserk units, and the packed colour mask made from that position's triple (claim 7).
- **Berserk-friendly.** A type's flag: a berserk unit may not fire at objects of this type
  (claim 6).
- **Psychic sensor, psychic-detection radius.** A building whose type has a positive
  psychic-detection radius, in whole cells. A house's *psychic sensors* are the buildings it owns
  with such a type (claim 9).
- **Hostile-intent line.** The line a psychic sensor lets its owner see from a hostile unit to
  what that unit is attacking or where it is going (claims 10 and 11).
- **Psychic reveal.** The superweapon kind that reveals an area of the map to the launching house
  (claims 13 to 15).
- **Late start.** The second ready-and-start chance of an infantry unit or vehicle in its update.
  A unit's readiness there counts the movement it made in that same update. The start makes current a
  queued behaviour of a unit that is then ready (S-0215 claim 11), which also defines the start that
  comes first in the update; the late start is separate from it.
- **Countdown update.** An update of a berserk unit on which its remaining berserk time goes down by
  one: any update that is in play and is neither an excluded nor a warp-only update (claim 5).
- **Unlimited controller.** A mind controller whose primary weapon gives unlimited control, so that its
  capacity test never refuses a target and it suffers overload pulses (S-0601, Terms and claims 1 and
  16). Its overload update is not a countdown update: it does not need the controller to be berserk.
- **Excluded update, warp-only update.** An update of a berserk unit that is in play but on which the
  countdown does not run. An excluded update is one that S-0215 claim 2 excludes for an infantry unit
  or vehicle (travelling through a tunnel, being erased, or in the leaving stage of a chrono move, in the sense of S-0513 claim 39: from the member's first
  update after launch until departure, including the update where its wait expires),
  or a landed aircraft's update while it is being erased (S-0215 claim 2 excludes its behaviour
  processing; the countdown part of that exclusion is this spec's choice, claim 5), or an update of
  a superweapon-warp member phasing in with a temporal beam attached that S-0215 claim 2 excludes
  (S-0513 claim 39a; claim 5). A warp-only update is one of a superweapon-warp member on which
  S-0513 claim 51 suppresses its ordinary work (claim 5).

## Behaviour

### Immunities and pulses

1. **Two immunities.** Every infantry, vehicle, aircraft and building type has a psionics immunity
   and a psychic-weapon immunity (Import mapping). Both are on by default for building types and
   off by default for infantry, vehicle and aircraft types. These are initial defaults when
   no rules layer supplies the field, not resets when a later layer omits it (S-0006 claim 2).
   An otherwise damageable building with the default psychic-weapon immunity ignores a
   psychic-damage hit. A building with the default psionics immunity fails the temporary-control
   eligibility test (S-0601 claim 3) and the berserk-weapon firing check (claim 3).
   Buildings are independently excluded from berserk hits (S-0302 claim 10) and permanent
   control (S-0601 claim 13), even when their psionics immunity is explicitly off.
   - Psionics immunity is consulted by: berserk hits (S-0302 claim 10), the firing check of a
     weapon whose warhead is berserk-causing (claim 3), temporary and permanent control
     (S-0601 claims 3 and 13), and psychic domination (S-0612 claim 13).
   - Psychic-weapon immunity is consulted only by damage resolution (S-0302 claim 8). It adds no
     firing refusal and no search exclusion: a unit may fire a psychic-damage weapon at a
     psychic-weapon-immune target, and the hit does nothing.
   - Backing: `measurable` (P1 for psychic damage, P4 for berserk immunity, P8 for
     absent-key defaults, temporary-control immunity and immunity-specific firing checks).
     Permanent-control eligibility is backed by S-0601 claim 13's scenarios. Psychic-domination
     immunity is `measurable` under S-0612 claim 13 (scenario PD10 part a).
   - Confidence: high (defaults and consumers); medium (that nothing else reads the
     psychic-weapon immunity)

2. **Psychic pulses need no delivery of their own.** A psychic area pulse, such as the one a
   deployed psychic infantry fires, is an ordinary weapon whose warhead is a psychic-damage
   warhead. It is fired under S-0517 claim 16 (or as an ordinary shot), spreads as ordinary area
   damage (S-0303, including the friendly-damage rules of claim 14 there), and each hit is resolved
   by S-0302. The psychic-damage property changes nothing about which objects the pulse reaches:
   it only makes psychic-weapon-immune objects ignore the hit.
   - Backing: `measurable` (scenario P1)
   - Confidence: medium

### Berserk

3. **Firing refusals for berserk weapons.** The firing check (S-0301 claim 9) answers *illegal*,
   at rank 4 of its table, when the weapon's warhead is berserk-causing and either
   - the target's type has psionics immunity, or
   - the target is a vehicle inside a tank bunker, or a tank bunker that holds a vehicle.
   So a berserk weapon is never fired at a building whose type keeps the default immunity, nor at
   a bunkered vehicle. Its area may still reach such objects when it is fired at something else,
   and S-0302 claim 10 then decides.
   - Backing: `measurable` for the immunity refusal (P8 c, using named test data).
     The bunker case is `unmeasured` here: no named single-target bunker fixture is supplied.
   - Confidence: high (psionics immunity); medium (the bunker case)

4. **Going berserk.** When a berserk hit takes effect (S-0302 claim 10) on a unit that is not
   berserk, all of the following happen at once:
   - its remaining berserk time becomes *T* and the berserk state starts,
   - it drops its target,
   - *hunt* is queued as its next behaviour (S-0215), replacing whatever was queued; it becomes
     current at the unit's next behaviour change, not within the hit,
   - in the original, a unit that belongs to a team leaves that team (see *team member* —
     the original's team system is out of scope).
   A berserk hit on a unit that is already berserk only sets its remaining time to the new *T*
   (S-0302 claim 10). Nothing else changes: its target and behaviour stay. Both rules hold for the
   berserk effect only. When the same hit gives a vehicle the crush response of S-0215 claim 39,
   that response follows the berserk entry and queues *move* in place of the *hunt* queued above,
   and on a vehicle that is already berserk it still gives the vehicle a destination and queues
   *move* (S-0215 claim 39).
   - Backing: `measurable` (P2 for entry, target loss, queued behaviour and re-hit).
     Team departure is `unmeasured` here because the original's team system is out of scope.
     The crush-response cases are backed as S-0215 claim 39 backs them (S-0210 scenario K15).
   - Confidence: high, except the crush-response cases, which take S-0215 claim 39's levels (medium
     for the replacement of the queued *hunt*)

5. **Countdown and the end of berserk.** In each countdown update a berserk unit's remaining time
   goes down by 1. In the update where it reaches 0 or less, the berserk state ends, the remaining
   time is set to 0, the unit drops its target and a behaviour is queued:
   - *guard* when its owner is human-controlled,
   - *hunt* otherwise, in the original. In our engine an AI house follows the human-controlled
     rule (*guard*), and only units of the neutral and special houses queue *hunt* (glossary,
     *AI house*).
   For *T* ≥ 1, expiry occurs on the *T*-th countdown update following application of the hit.
   The first such update may occur in the hit tick if the hit precedes that unit's update
   (S-0203 claims 1 and 3); an elapsed interval of exactly *T* ticks from the hit is not guaranteed.
   A hit with *T* ≤ 0 still has all the effects of claim 4, and the state ends at the next
   countdown update (Edge cases, `quirks.berserk_zero_duration`).

   **Which updates are countdown updates.** A countdown update is any update of a berserk unit that
   is in play and is neither excluded nor warp-only. The countdown is one of the few counters that
   step once per turn, as S-0203 claim 9 allows, and this claim is the owner that says so. It does
   not depend on the unit's behaviour timer being due, on what the unit is doing, on its owner or on
   whether it has a target. The countdown does *not* run in any of these:
   - **Out of play.** A unit that is out of play takes no updates (S-0203 claims 2 and 3): a
     passenger of a transport that is not open-topped, an occupant of a building, and any other unit
     held inside another object. Its berserk state and remaining time are *kept unchanged* while it
     is inside. The approved specs give no change to the berserk state when a unit boards or leaves
     (S-0507, S-0508 and S-0605 do not mention berserk), so none is assumed here. The countdown
     resumes at the first update after the unit is back in play, with the time it had. A passenger
     of an open-topped transport stays in play and does take updates (S-0203 claim 3), so its
     countdown continues.
   - **Excluded updates.** The updates of S-0215 claim 2 on which the unit is travelling through a
     tunnel, is being erased by a temporal weapon or is in the leaving stage of a chrono move
     (S-0513 claim 39), and the updates in which a superweapon-warp member phases in that S-0215
     claim 2 excludes: those of a vehicle or landed aircraft member with a temporal beam attached,
     being erased or not, and those of an infantry member while it is being erased or, with S-0513
     `quirks.phase_in_beam_spares_infantry` off, while a beam is attached to it (S-0513 claim 39a).
     Such an update takes no countdown step; the remaining time is kept. For a landed aircraft being
     erased, and so for a landed aircraft member phasing in with a beam attached, the countdown is
     not established: S-0514 claim 17 excludes its movement, turning and firing, and S-0215 claim 2
     excludes its behaviour processing, but neither mentions the countdown. Until a measurement
     settles it, this spec takes no countdown step there (Open questions). No part of P9 tests that
     case, and no setup is known for a berserk aircraft (claim 7), so it is `unmeasured`.
   - **Warp-only updates** of a superweapon-warp member of a kind that S-0513 claim 51 names
     (vehicles, teleporter infantry, landed aircraft) (S-0513 claim 51; S-0203 claim 4). In the
     original the leaving-stage updates are the excluded ones above. In our engine the countdown
     joins the ordinary work that claim 51 suppresses from the first pending update through
     arrival effects: no step is taken, none is made up later, and the remaining time is kept. In
     arrival hold it runs in each update after that update's hold-end test, whether the hold is
     still holding or expiring, as S-0513 claim 51 gives the other ordinary work, except in those
     excluded phasing-in updates, which take no countdown step and keep the remaining time. This is
     a departure of design scope: for the updates before arrival hold, the original's continuation
     is unresolved (S-0513 claim 52), so it may run the countdown in them, and our engine does not.
     It has no quirk switch. An unlimited controller (Terms) has an overload update
     (S-0601 claim 16) in the same set of updates as this countdown, read without the berserk
     condition: every update that is in play and is neither excluded nor warp-only. In the warp-only
     updates the overload update is suppressed and its timing is kept, as the countdown is. S-0601
     claim 16 owns its rules and its backing.

   **Effects in the update where the countdown ends.** The expiry shows in that same tick, in the
   unit's firing, its target searches and its behaviour. The observable consequences are these.
   The unit's firing and target searches in that update already treat it as ordinary (the first
   bullet below). Whether a queued behaviour that the expiry leaves can become current at the end of
   that update, through the unit's late start (S-0215 claim 11), is open (see Open questions). For an
   infantry unit that is moving and ready, S-0215 B46 is the proposed measurement of that case; for every
   other ready unit no measurement exists yet. The queued-behaviour bullet below states what follows either way:
   - **No berserk shot in the expiry update.** The expiry therefore clears the target before any
     shot of that update. The last shot fired as
     berserk (halved rearm time, S-0301 claim 8; friendly targets allowed, S-0309) is in the
     update before the expiry update, or earlier. After the expiry update's step the unit is an
     ordinary unit for the rest of that update: a target it acquires later in the same update
     follows the ordinary rules (no berserk exceptions, ordinary rearm) and may be fired at in that
     same update. A shot already in flight is not affected.
   - **The queued behaviour is not yet in charge.** The *guard* or *hunt* queued by expiry is only
     queued when the countdown ends. If the expiry is already in effect when the unit's late start is
     judged (see Effects), a unit that is then ready to change may have the queued behaviour made current
     by that late start at the end of the same update (S-0215 claim 11). Either way, no guard-mode search,
     guard hold or processing of the queued behaviour happens in the expiry update, and its first
     processing is in the unit's next update. A hunt move that the unit is carrying out as its current
     behaviour continues through the expiry update. On the expiry tick, the behaviour that was
     current before the expiry still runs when its delay is due, now without the target it had and
     with ordinary, non-berserk searches. A vehicle that is moving under hunt is
     not ready (S-0215 claim 11), so it keeps hunting until it stops; an infantry unit under hunt is
     ready, so, if the expiry is already in effect at its late start, that late start may make the queued
     behaviour current at the end of the expiry update, without processing it (for an infantry unit that is
     moving and ready, the proposed measurement is S-0215 B46; other cases are unmeasured, see Open
     questions).
   - **A queued behaviour that never started.** Expiry queues *guard* or *hunt* by replacing
     whatever is queued, as claim 4 does. A unit that was never ready after the hit still has the
     hit's *hunt* queued; expiry replaces it by *guard* (human-controlled owner, AI house) and the
     unit never hunts. Where expiry queues the behaviour that is already current and nothing else
     is queued (a neutral-house unit under hunt), nothing changes.
   - Backing: `measurable` (P2; for *T* ≤ 0, P3 and S-0302 scenario D17 part i's named zero- and
     negative-versus test setup; P9 part a for the shot, and P9 part b for the behaviour order except
     the start order (for an infantry unit that is moving and ready on the expiry tick, S-0215 B46; otherwise
     unmeasured, see Open questions); P9 part d for
     the erasing exclusion; P9 part e for the leaving stage of a chrono move, by its leaving-stage
     expectation; P9 part c for the replaced queue if a stock or map-placed setup keeps the vehicle
     moving, as that part says, otherwise `unmeasured`). The tunnel and landed-aircraft exclusions are
     `unmeasured`: no setup is known that puts a berserk unit in a tunnel, and no setup is known for a
     berserk aircraft that could be landed and erased (claim 7). The out-of-play rule is
     `unmeasured`: no ordinary way is known to bring a berserk unit inside another object (a berserk
     unit takes no orders, S-0214 claim 15; a hit on a unit inside needs it to be on the map,
     S-0303 claim 9). No scenario creates the branch, and whether ordinary play reaches it is not
     established, since claim 4 does not say what a berserk hit does to a current behaviour that
     takes a unit inside. The non-leaving warp-only rule is a design rule of our engine, checked by
     replaying P9 part e on our engine; the original's leaving stage is `measurable` (P9 part e).
     The phasing-in exclusion of a superweapon-warp member with a temporal beam attached is
     `unmeasured`: no scenario makes a berserk warp member phase in with a beam attached. The off
     setting of S-0513 `quirks.phase_in_beam_spares_infantry` is design.
   - Confidence: high (countdown and expiry effects); medium (order of the step and the firing check,
     pending P9 parts a and b; the start of the queued behaviour in the expiry update, pending S-0215 B46 (see
     Open questions); the erasing and leaving-stage exclusions of the countdown, pending P9
     parts d and e; tunnel exclusion, unmeasured; the open-topped passenger case); medium
     (the *T* ≤ 0 case; the phasing-in exclusion of a superweapon-warp member with a temporal beam
     attached, unmeasured, except for a landed aircraft member); low (landed aircraft exclusion,
     unmeasured, and the phasing-in exclusion of a landed aircraft member, as for that
     exclusion; that the original takes the step in the non-leaving warp updates, which our engine
     suppresses by design; the out-of-play rule: derived from the update order, unmeasured, and not
     known to be reachable in ordinary play)

6. **While berserk.** A berserk unit takes no orders (S-0214 claim 15), fires with the halved rearm
   time (S-0301 claim 8) and searches for targets with the friendly-row exceptions of S-0309. In
   addition the firing check answers *illegal*, at rank 4 of S-0301 claim 9, when the firer is
   berserk and the target's type is berserk-friendly. So a berserk unit never fires at, and never
   acquires through a search that asks the firing check, an object of a berserk-friendly type.
   - The original's rule that a unit of a computer-controlled owner drops a target allied with its
     owner in each update does not apply to a berserk unit, so a berserk unit of such an owner
     keeps attacking its allies. In our engine that rule holds only for units of the neutral and
     special houses (glossary, *AI house*). S-0215 claim 33a owns the clearing rule and
     its other exceptions.
   - **Healing is unaffected.** A berserk unit regains health exactly as an ordinary unit does:
     self-healing, the house-wide healing totals, healers, a repair depot and a hospital are
     S-0314's, and the heal-all crate is S-0520 claim 14's. None of their conditions asks whether
     the unit is berserk. A healing hit from a warhead that is not berserk-causing leaves the
     remaining time unchanged. A hit with a negative amount from a berserk-causing warhead is still
     a berserk hit (S-0302 claim 10): it changes no health, and its effects are those of claims 4
     and 5. Where such a hit falls relative to the countdown step is open (Open questions): it depends
     on the update in which the hit is applied (for a projectile, its own turn, S-0203 claim 5) and on
     the target's update (S-0203 claims 1 and 3). Backing: `measurable` (scenario P3: its
     negative-versus repeat and its hit timing on either side of the target's update). Confidence: low.
     **Self-healing against the countdown.** Self-healing reads no berserk state and the countdown
     reads no health, so for a berserk unit with no overload pulse, the order of self-healing and the
     countdown step has no observable effect: an engine may run them in either order. For an unlimited
     controller, the place of the countdown step against its overload pulse and its heal is S-0601
     claim 16's; this spec adds no order of its own. In a given update a damaged berserk unit that heals
     itself and has no overload pulse gains the same health and its remaining time falls by the same step
     whichever comes first. Backing: `unmeasured` for units without an overload pulse (no observable
     depends on the order); for an unlimited controller, as S-0601 claim 16 gives it.
     Confidence: high for units without an overload pulse; for an unlimited controller, as S-0601
     claim 16 gives it.
     Healing backing: `unmeasured`: no scenario here records the health of a berserk unit, and no
     setup covers the healers, the repair depot, the hospital or the crates.
     Healing confidence: medium.
   - Backing: `measurable` (scenario P4)
   - Confidence: high (berserk-friendly refusal); medium (the exception to the computer-owner rule)

7. **Berserk tint (Q-1526).** This is presentation only; no game state depends on it.
   - **Selected colour.** The *berserk selector* is the rules' berserk-colour field, a zero-based
     position in the colour-adjustment table (S-0006 claim 2), position 0 when absent. Its RGB
     triple becomes the *berserk mask* by S-0102 F9, exactly as a protection selector's triple
     does. The bytes are used as they are, unscaled, packed once in the surface's layout. The original
     sometimes combines several layouts' masks into one, which adds stray colour bits. That is
     the drawing defect S-0102 claim 34 corrects without a switch; it is corrected here in the same way.
   - **Composition.** On each qualifying opaque pixel the berserk mask joins the packed pixel by
     bitwise union after palette conversion and shade selection, as S-0102 claim 34 and F9
     specify for protection masks. This is neither a per-channel addition, saturating or not, nor a
     multiplication. Transparent pixels and pixels rejected by depth stay untouched.
   - **Drawings that take it.**

     | Berserk object, drawing | Berserk mask |
     |---|---|
     | Vehicle drawn as voxels: body and voxel turret/barrel | yes |
     | Vehicle drawn as a sprite, including a voxel turret on a sprite body | none |
     | Infantry sprite body | yes, unless the centre of the infantry's cell is shrouded for the viewing player (then none in that frame) |
     | Aircraft | none |
     | Shadows of any of these | none: ordinary shadow darkening only |
     | Animations attached to the object | none; attachment grants no recolouring (S-0102 claim 34, animation rule) |

     Buildings cannot go berserk.
   - **Other masks and their combination.** A vehicle drawn as voxels can carry two masks: the
     curtain mask (S-0102 claim 34) and the berserk mask. When both apply, the combined mask is
     their bitwise union, and that union joins the pixel as above. Neither replaces the other,
     and their components are not summed. An infantry unit has no other mask: curtain protection
     kills ordinary infantry (S-0513 claim 23). Air-strike marking applies only to a marked building
     (S-0519 claims 17 and 26). No other colour mask applies to a vehicle or infantry.
   - **No pulse.** The berserk mask is the same in every drawn frame while the unit is berserk. It
     is not scaled by brightness, and the berserk state changes no brightness. The curtain's
     brightness envelope (S-0513 claim 29) and deactivation darkening (S-0104 claim 13) act on brightness
     only, so they change which shaded pixel the mask is joined to, never the mask. The first frame
     drawn after expiry (claim 5) has no berserk mask.
   - Backing: `measurable` (P4 c). The aircraft case is `unmeasured`: no stock-data setup puts a
     susceptible aircraft in the berserk state. The absent-field default is `unmeasured` because
     the scenarios set the field rather than remove it.
   - Confidence: high for the selector, the union composition, the voxel-vehicle, sprite-vehicle
     and infantry rows, the union with the curtain mask and the absence of a pulse; medium for
     the shrouded-cell exception, shadows and the aircraft row

8. **Area berserk weapons (chaos drone).** No rule in this spec or in S-0302 names a particular
   unit. A vehicle type that combines a weapon with a berserk-causing warhead and an area spread,
   fired at an area through the vehicle's deploying behaviour (S-0517 claim 1), with psionics immunity and
   the berserk-friendly flag behaves as claims 3 to 7 give: its own hits cannot make it berserk,
   and units made berserk by it never fire at it or at others of its type. Every berserk hit takes
   *T* at distance 0 (S-0302 claim 10), so equally susceptible units with the same type,
   posture and damage modifiers get the same initial duration from one pulse. This does not
   promise simultaneous expiry: units can have different numbers of countdown updates in the hit
   tick. Whether such a weapon also creates gas particles depends only on its imported data.
   If the data supplies gas particles with a berserk-causing warhead, their delivery
   follows S-0613 claims 4 and 13, and their later hits can reset individual durations under
   S-0302 claim 10.
   - Backing: `measurable` (scenarios P4 part a and P5)
   - Confidence: medium

### Psychic sensor

9. **Membership.** A building whose type has a psychic-detection radius above 0 becomes one of its
   owner's psychic sensors when it is placed on the map. When it is captured, it stops being one
   of the old owner's and becomes one of the new owner's. The same set of buildings feeds the
   warning marker of S-0513 claim 11. A sensor counts only while it is in operation (S-0405), as
   in S-0513 claim 11.
   - Backing: `measurable` (scenario P6)
   - Confidence: high (placement and capture); medium (operation requirement)

10. **Detection conditions.** A viewing player sees the hostile-intent line of a unit when all of
    these hold:
    - the unit is an infantry, vehicle or aircraft and is in play,
    - it is not owned by the viewer's house, and the viewer's house does not count the unit's
      owner as an ally (the one-way alliance is taken from the viewer's side),
    - at least one psychic sensor of the viewer's house that is in operation is within reach of
      the unit's intent: (a) the unit's target, an object or a cell, or (b) the unit's movement
      destination, an object or a cell, lies within that sensor's radius. The destination is the
      last waypoint of its queued path when it has one, and its current destination otherwise.
    Distance is the point-to-point spatial distance of F1 from the sensor's reference point
    (as in S-0513 claim 11) to the target's or destination's reference point. No building-size
    subtraction applies, even when the target is a building. The point is within reach when the
    distance is at most the radius in cells times 256 leptons (Formulas F1).
    Visibility plays no part: the line is shown for units that are cloaked, disguised or under the
    shroud for the viewer (Edge cases, `quirks.psychic_intent_ignores_visibility`). Eligibility is
    tested afresh when drawing. There is at most one line per unit: a target takes precedence
    over movement. In particular, a destination inside the radius can make the unit eligible
    while its line points to an attack target outside the radius. With no target, the line
    uses the destination, choosing the final queued waypoint when available. It is the intent's
    endpoint, not the unit's position, that must be in reach.
    - Backing: `measurable` (P6, including parts e to h).
      The table-root versus exact-root distinction is `unmeasured`: no legal reachable
      boundary separating the two has yet been identified for this sensor.
    - Confidence: high (one line and target precedence); medium (other conditions)

11. **The line's look.** A hostile-intent line runs from the unit to its target (from its firing
    point to the target's aim point) or from the unit to its destination (raised by a bridge's
    height when the destination cell carries a bridge). Both ends carry a small square of 3 by 3
    pixels, drawn in the unit owner's house colour. The line has a dash pattern animated with
    real time rather than ticks. Presentation only; exact line-colour animation is not specified.
    - Backing: `measurable` (P6 c)
    - Confidence: medium

12. **Range ring.** A psychic sensor of a type that shows a range ring when selected (Import
    mapping) shows a ring of its psychic-detection radius when that radius is at least 1, in
    preference to any cloak-sensor radius it also has. Presentation only.
    - Backing: `measurable` (scenario P6 part d)
    - Confidence: medium

### Psychic reveal

13. **Launch.** A psychic reveal is launched like every superweapon: its record must be ready and
    is spent under S-0512 (claims 1 to 8), and it may be aimed at any cell, by the tactical view or
    by the radar (S-0513 claims 1 to 3). It needs no power unless its data says so (S-0512). At
    the launch, in the same tick:
    - the area reveal of claim 14 is made,
    - the reveal's activation sound plays at the target cell's centre, positionally, for every
      player,
    - for the launching player the aiming mode ends and that superweapon's ready announcement is
      cancelled if it is still pending.
    There is no launch announcement for any house and no radar event.
    - Backing: `measurable` (scenario P7)
    - Confidence: high

14. **Area and visibility.** The reveal is an area reveal of S-0409 claim 17 originating at the
    target cell's centre. Its grid centre includes S-0409 claim 5's height shift; it is not
    necessarily the unshifted target cell. Its radius in cells is the rules' psychic-reveal
    radius (3 when absent). As that claim says, a radius above 10 counts as 10 and a radius of 0 reveals nothing.
    Unlike the other reveals of S-0409 claim 17, high ground hides nothing: the cell test of
    S-0409 claim 7 is not made, so every legal cell of the shape (S-0409 claims 5 and 6) is
    explored and seen whatever the reveal-by-height flag says. It is made for the launching house. In the original it changes only the local player's map,
    and so reveals the local map only when the launching house is the local player or its
    reveal is shared with that player under S-0409 claim 2. Our engine applies it to the launching house's own map state
    (S-0409 claim 1) and to the houses that S-0409 claim 2 lets share it; this scope is a design
    rule, as for the reveals named in S-0409 claim 17.
    - Backing: `measurable` (scenario P7); the default radius 3 is `unmeasured`, because stock
      data sets the field and scenarios use stock data only
    - Confidence: high (centre, radius and no high-ground test); medium (who receives it)

15. **Duration and detection.** The reveal has no duration: it explores and sees the cells once and
    adds no lasting sight, so the cells stay explored and are re-covered by the shroud's normal
    rules (S-0409 claim 17, including the cells inside hostile gap fields). It gives no sensor
    coverage and no disguise detection: cloaked units in the area stay hidden and disguised units
    stay disguised (S-0516). It deals no damage and changes no ownership or orders.
    - Backing: `measurable` (scenario P7 part c)
    - Confidence: high

16. **Unused berserk setting.** The rules' berserk-allowed flag in the combat-damage section
    is read but has no effect on the berserk hits or countdown described here.
    - Backing: `unmeasured`: stock data fixes the flag, so the stock-only scenarios do not
      compare its two settings.
    - Confidence: medium

## Formulas

> **F1. Sensor reach.** A point *p* is within reach of a psychic sensor *s* with radius *R* (whole
> cells) when trunc(*r*(Δ*x*² + Δ*y*² + Δ*z*²)) ≤ 256 × *R*. Coordinate differences are
> integer leptons between the sensor's reference point and *p*, heights included. Squares and
> sum are evaluated in floating point (S-0205). The root *r* is S-0204 claim 5's table root,
> with its rounding and `quirks.table_root_distance` alternative. Its non-negative result is
> truncated toward zero to integer leptons. Radius scaling and comparison are integer.
> Unlike S-0204 claim 6's object-to-object distance, no target-building size is subtracted.

> **F2. Berserk countdown.** With remaining time *t* after the hit (*t* = *T*), each countdown update of the
> unit sets *t* ← *t* − 1 (integer); berserk ends in the countdown update where *t* ≤ 0. For *T* ≥ 1 that is
> the *T*-th countdown update following application of the hit; for *T* ≤ 0 it is the first.
> The first countdown update may be in the same tick as the hit (claim 5). Out-of-play time, excluded updates and warp-only updates are not countdown updates and leave *t* unchanged (claim 5). A berserk shot in a countdown update is allowed only when *t* > 0 after its step (claim 5).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Psychic-reveal radius default | 3 | cells | when the rules' field is absent (claim 14) |
| Berserk selector default | 0 | zero-based table position | colour-adjustment table position (claim 7) |
| Psychic-detection radius default | 0 | cells | no sensor (claim 9) |
| Sensor reach scale | 256 | leptons per cell | F1 |
| Intent-line end square | 3 × 3 | pixels | claim 11 |
| Berserk countdown step | 1 | per countdown update | F2 |

## State and ordering

| State | Event | Result |
|---|---|---|
| not berserk | berserk hit takes effect (S-0302 claim 10) | berserk with *t* = *T*; target dropped; *hunt* queued; leaves its team (original) |
| berserk | berserk hit takes effect | *t* = new *T*; nothing else |
| berserk | countdown update, *t* − 1 > 0 | *t* decreases by 1 |
| berserk | out of play (no update), excluded update (tunnel, erasing, leaving stage) or warp-only update | unchanged: still berserk, *t* kept (claim 5) |
| berserk | countdown update, *t* − 1 ≤ 0 | not berserk, *t* = 0; target dropped; *guard* queued (human-controlled owner, AI house) or *hunt* (otherwise) |

- Entry clears the old target immediately. This does not forbid acquiring that object again
  through a later search, even in the same tick. The queued *hunt* becomes current at the next
  behaviour change (claim 4).
- Every effective re-hit replaces the remaining duration rather than adding to it (claim 4).
  A countdown update between two hits can therefore affect expiry in the same tick.
- A psychic reveal applies in the launch tick (claims 13 and 14). A re-cover sweep later
  in that tick can already hide a cell in a hostile gap field again (S-0409 claim 17).

## Edge cases and quirks

- **`quirks.berserk_zero_duration`** (default on, the original). A berserk hit whose *T* is 0 or
  less (for example on a target whose versus value for the warhead is 0, or any berserk hit in an
  inert game)
  still makes the target drop its target and queue *hunt*, and the unit is berserk until its next
  countdown update, when expiry queues *guard* or *hunt* (claim 5). Its effect is that the target loses its
  target and its queued behaviour. Off: a berserk hit with *T* ≤ 0 has no effect on the target.
- **`quirks.psychic_intent_ignores_visibility`** (default on, the original). Hostile-intent lines
  are shown for units the viewer cannot see, including cloaked units, disguised units and units
  under the shroud, so a sensor betrays where they are (claim 10). Off: a line is shown only for a
  unit that is visible to the viewer's house (not hidden under S-0516 claim 18, and on a cell the
  viewer's house sees).
- **Sensor capture.** A captured sensor moves to the new owner at once (claim 9); there is no
  delay.
- **Berserk-friendly applies only to firing.** A berserk unit's area weapon may still hit a
  berserk-friendly object near its target. When that weapon's warhead is itself berserk-causing,
  the object can go berserk only if all exceptions of S-0302 claim 10 permit it, including
  the immunity, building and attacking-house exceptions. For other warheads, resolve the hit
  normally under S-0302; berserk-friendly itself supplies no damage immunity.

- **Berserk selector outside the table** (departure). The original does not check the berserk
  selector against the table's 16 positions, so a value below 0 or above 15 picks up bytes that are
  not a colour-adjustment entry. Ours: the importer warns, and a berserk unit is drawn with no
  berserk mask. Stock data never does this, so no switch is offered. Backing: `unmeasured`: the
  original's result depends on bytes outside the table and gives no stable value to compare; engine
  fixtures check the warning and the absent mask. Confidence: high that the original does no range
  check.

## Import mapping

| Key | Section | Parse | Absent | Feeds |
|---|---|---|---|---|
| `ImmuneToPsionics` | infantry, vehicle, aircraft or building type | boolean | yes for building types, no for others | psionics immunity (claims 1, 3) |
| `ImmuneToPsionicWeapons` | same | boolean | yes for building types, no for others | psychic-weapon immunity (claim 1; S-0302 claim 8) |
| `BerserkFriendly` | infantry, vehicle, aircraft or building type | boolean | no | berserk-friendly (claim 6) |
| `Psychedelic` | warhead | boolean | no | berserk-causing warhead (claims 3, 4; S-0302) |
| `PsychicDamage` | warhead | boolean | no | psychic-damage warhead (claim 2; S-0302) |
| `PsychicDetectionRadius` | building type | integer, cells | 0 | psychic-detection radius (claims 9 to 12) |
| `HasRadialIndicator` | building type | boolean | no | shows a range ring when selected (claim 12) |
| `BerserkColor` | `[AudioVisual]` | integer, zero-based position in the colour-adjustment table; not a colour-definition name | 0; absent in a later layer retains the held selector | berserk selector (claim 7); a value outside 0 to 15 is the departure under Edge cases |
| `PsychicRevealActivateSound` | `[AudioVisual]` | sound name | none | activation sound (claim 13) |
| `PsychicRevealRadius` | `[CombatDamage]` | integer, cells | 3 | reveal radius (claim 14) |
| `Type` | superweapon section | kind name | (S-0512) | the psychic-reveal kind of S-0512's kind table (claims 13 to 15) |
| `EVA_PsychicRevealReady` | announcements | announcement name | none | the ready announcement cancelled at launch (claim 13) |
| `BerzerkAllowed` | `[CombatDamage]` | boolean | no | read, no effect (claim 16) |

## Measurement scenarios

All scenarios: stock data except the explicitly named test data below, no engine add-on and no mods (the named test files of a scenario excepted)
apart from that named test data, a small purpose-made skirmish map, normal game
speed, 800 × 600 presentation. Use scripted input and a read-only recorder when needed to deliver and observe the stated setup on the stock executable. Record the seed rather
than assuming it can be set. Repeat each deterministic part three times. Include one bystander
computer player, kept outside the test area under the glossary's void rule. Record tick counts, health and orders using read-only observation, and capture frames for presentation.
Unless a part gives a narrower falsifier, any observed departure from its expected outcome
falsifies that part. A setup not achievable with stock data or the scenario's explicitly named
test data is not a successful measurement.

Psychic-domination immunity in claim 1 uses S-0612 claim 13's measurement coverage:
scenario PD10 part a isolates takeovers with named zero-damage test data and expects the
psionics-immune subject to keep its owner. A takeover of that subject falsifies the immunity.

- **P1. Immunities.** A deployed psychic infantry fires its pulse with an enemy building, an
  enemy vehicle, an enemy infantry and an enemy infantry of a psychic-weapon-immune type inside
  the pulse's area. Choose the building and the immune infantry by reading the stock rules (a
  building type that does not set the psychic-weapon immunity field, an infantry type that sets
  it); copy no value. Record the health of each before and after. Expected: the building and the
  immune infantry are unchanged, the vehicle and the other infantry take damage. Falsified if the
  building or the immune infantry is damaged.
- **P2. Entry and expiry.** A human-owned tank attacking a durable enemy target stands inside
  an enemy chaos drone's pulse area. Have the drone fire once, then withdraw it. Void a run
  with any further berserk hit during the isolated interval. Record hit and countdown-update ticks,
  target loss, queued and current behaviour, tint and acceptance of subsequent orders. Compute
  *T* under S-0302 claim 10. Expected: the initial hit clears the target and queues *hunt*,
  without immediately replacing the current behaviour. Expiry clears the target and queues
  *guard* on the *T*-th countdown update following application of the hit, counting an update
  in the hit tick when applicable. The unit then accepts orders. Repeat with a neutral-house
  tank: expiry queues *hunt*. Repeat with map-placed tanks on either side of the drone in the
  update order (S-0203 claim 3). Re-hit halfway through: remaining time becomes the new *T*,
  not the sum, without another target clear or behaviour change. Falsifiers include immediate
  behaviour replacement, an additive re-hit, or expiry at the wrong update. This tests a
  queued behaviour, not a guarantee that its later search chooses a different enemy.
- **P3. Zero duration.** A berserk hit with *T* ≤ 0: either a target whose armour has a versus
  value of 0 for the berserk warhead (choose by reading the stock rules; not measurable if none
  has), or any target in an inert game (S-0302). The target is attacking something before the hit.
  Record target, queued behaviour and berserk state around the hit and next countdown update,
  not just tint: entry and expiry can both precede a drawn frame. Expected: target cleared and
  *hunt* queued at entry, then expiry and *guard* queued at the first countdown update.
  Falsified by no entry effects, no expiry at that update, or a different queued behaviour.
  For a guaranteed susceptible zero-duration fixture, use S-0302 scenario D17 part i's
  named test map D17-zero-duration, including its negative-versus repeat and hit timing on
  either side of the target's update. This is an explicit named-test-data exception to the
  common stock-data setup.
- **P4. While berserk.** (a) A berserk tank of house A beside a chaos drone and an infantry, both
  of house A. Expected: it attacks the infantry, never the drone. (b) A tank of the neutral house
  made berserk next to another neutral-house unit: it attacks that unit and keeps attacking it.
  Also place an enemy chaos drone in the pulse area: it must not go berserk from the hit.
  (c) Berserk mask (claim 7). This part uses named test rules. First read the stock curtain,
  force-shield, air-strike-marking and berserk selectors (copy no value). If another selector shares
  the berserk selector's position, move the berserk selector to an unused valid position in the test
  rules. Then change only the triple at the berserk position, to invented (1, 0, 0), (0, 3, 0),
  (0, 0, 1), then (255, 255, 255), each in a fresh session, and keep every other table position.
  On flat ground, in full view, make three units berserk: a voxel vehicle with a voxel turret, a
  sprite-drawn vehicle (choose one by reading the stock rules; if none exists, a named test vehicle
  type with sprite art) and an infantry unit. A berserk unit hunts and refuses orders, so isolate
  each unit where its hunt finds nothing reachable, or use named test types with no speed and no
  weapon. Use no damage flash and lossless captures. Capture each unit before the hit, on at least
  30 drawn frames while berserk, and on the first frames after expiry. Compare each berserk frame
  with a non-berserk frame of the same unit at the same facing, image frame and brightness, using
  S-0102 F9 and claim 34. Include pixels whose shaded channels already hold the mask's bits, and
  black shade-row pixels. Expected: the voxel body and turret, and the infantry body, show the
  union. The sprite vehicle, every shadow and any attached animation are unchanged. Berserk frames
  that share facing, image frame and brightness are identical, and the mask is gone on the first
  frame after expiry. Original 5-6-5 captures may show the stray extra-layout bits of claim 7 and
  are not acceptance values for that correction; the engine's rendered pixels must match F9 exactly. A per-channel
  sum or clamp, scaling of the mask, variation between matching berserk frames, a mask on the
  sprite vehicle or a shadow, or no mask on the voxel vehicle or infantry falsifies the matching row.
  Then make the voxel vehicle berserk first and curtain it afterwards (an invulnerable target cannot
  go berserk, S-0302 claim 10). Record that it is still berserk after the curtain arrives; if it is
  not, this route cannot measure the combination row. With the curtain and berserk selectors at
  different positions, use test triples whose bits overlap, then triples whose bits are disjoint.
  Expected: the union of both masks on each pixel, compared with curtain-only frames of equal
  brightness. A sum, a carry between channels, or one mask replacing the other falsifies the
  combination row. Last, map-place an infantry unit in a cell whose centre is shrouded for the
  viewer and which borders visible cells. Make it berserk, and capture the frames after the hit and
  any frames in which its hunt carries it across the shroud edge. Expected: no berserk mask in a
  frame where its cell centre is shrouded, and the mask in a frame where it is not.
  Falsifiers of parts a and b: a shot aimed at a berserk-friendly drone, losing the allied target
  solely because it is allied, or the immune drone going berserk.
- **P5. Equal initial duration over the area.** Several enemy tanks of one type, rank and
  damage modifiers at different distances inside a chaos drone's area, all on the same side
  of the drone in update order.
  Record the first hit and expiry for each, and record whether any gas particles appear and compare with the drone's imported data.
  Void runs with further berserk hits. Expected: gas particles appear only if the imported data
  supplies them, equal initial *T* under S-0302 claim 10, and expiry after equal numbers of countdown updates.
  A duration difference caused by distance, or a gas particle created where the data supplies none,
  falsifies the claim. A later reset does not falsify the equal-initial-duration claim.
- **P6. Psychic sensor.** The viewer owns a psychic sensor (choose the type by reading the stock
  rules). Enemy tank 1 is ordered from outside the radius to move to a cell inside it, enemy tank
  2 to attack an object inside it, enemy tank 3 to move to a cell outside it. Expected: lines for
  tanks 1 and 2 only, in the enemy house colour, while the sensor is powered, and none while it is
  unpowered (S-0405). An allied unit's order inside the radius shows no line. A cloaked enemy unit
  ordered into the radius shows a line. Capture the sensor with an enemy engineer: from then on
  the viewer no longer sees lines from that sensor. In a separate run the viewer captures an
  enemy sensor and begins seeing lines from it. No player-view switching is assumed.
  (c) Screenshot the ends: 3 × 3
  squares; record their house colour. Compare dash motion across equal tick intervals at two
  game speeds: animation follows elapsed real time rather than tick count. Also order a unit
  onto a bridge and verify the movement endpoint is raised to deck height.
  (d) Select the sensor: its ring has the psychic-detection radius, not its cloak-sensor radius.
  (e) Keep a tank with both an attack target and a movement destination in reach, using an
  attack while moving where legal. Expected: exactly one line, to its target. Move the target
  outside reach while keeping its destination inside: the line still points to the target.
  (f) Queue movement waypoint points whose first point is inside and last point outside the radius,
  then reverse them. With no target, eligibility and the endpoint follow the last waypoint.
  Keep the unit itself outside reach in both tests. (g) Use a disguised enemy and a unit on a
  shrouded cell whose endpoint is in reach: lines still appear. For alliance direction, use a
  purpose-made map with a one-way alliance and keep the reverse relation unchanged: it is the
  viewer's relation that suppresses the line. (h) Place endpoints at the integer-lepton reach
  boundary of F1, then just outside it, using stock movement and object positions where possible.
  Repeat with a height difference and with a building target whose centre is outside reach
  but foundation edge inside it. Expected: height counts, and building size does not extend
  reach. Record actual reference-point coordinates rather than just cell labels. Falsifiers:
  two simultaneous lines, eligibility following the first waypoint or the unit's position,
  hidden-unit suppression, the reverse alliance deciding, or a boundary result unlike F1.
  The table-root versus exact-root distinction is unmeasured unless a legal reachable boundary
  separates their results.
- **P7. Psychic reveal.** Grant the reveal to the viewer and launch it at a shrouded cell far from
  any own unit, next to a cliff of several height levels. Expected: the shape of S-0409 claim 6 for
  radius min(imported radius, 10), with the height-shifted centre of S-0409 claim 5, is explored
  in the launch tick, including
  cells behind the high ground that an ordinary reveal would leave out (S-0409 claim 7), the
  activation sound plays positionally and no launch announcement or radar event appears.
  Record sound, targeting mode, the ready record and revealed cells at tick boundaries.
  Expected: readiness is spent, targeting ends for the launching viewer, and the reveal has
  no persistent sight contribution. Units become hidden again when S-0409's normal coverage
  rules require it, not necessarily in the next tick. Compare an area with and without a hostile
  gap field: exploration persists outside it and is re-covered under S-0409 claim 17 inside it.
  (b) Launch from a bystander house first without, then with the sight-sharing conditions of
  S-0409 claim 2. In the original only the latter reveals the viewer's map. Our engine's
  per-house scope is a design rule, not an observable of the original.
  (c) A cloaked enemy stays hidden and a disguised spy stays disguised. Record their health,
  ownership and orders too: the reveal changes none of them. Falsifiers include height-blocked
  cells missing from the shape, persistent sight without a covering object, detection of the
  cloaked unit, loss of disguise, or a launch announcement or radar event.

- **P8. Per-kind defaults and building consumers (Q-0661).** Named test map
  P8-psychic-defaults registers new infantry, vehicle, aircraft and building types so no lower
  rules layer has supplied either immunity. For each kind provide three otherwise identical
  variants: both immunities omitted, both explicitly on, both explicitly off. Give them no
  other protection, and keep aircraft landed. Use separate undamaged targets for each part.
  (a) Deliver a positive psychic-damage hit with a named test weapon that can hit each kind,
  with positive versus for each target's armour. Record health before and after. Expected:
  omitted buildings match the on variant (no health loss), while omitted infantry, vehicles
  and aircraft match the off variant (health loss). All explicit-on variants ignore the hit
  and all explicit-off variants lose health.
  (b) Try temporary control with a controller that has free capacity and whose test weapon
  permits every target kind. Use enemy targets satisfying all other conditions of S-0601
  claim 3, and record ownership and control state. Expected: omitted buildings resist like
  the on variant; omitted nonbuildings are taken like the off variant. Explicit-on targets
  resist and explicit-off targets are taken, including buildings.
  (c) Give a ready, armed firer two otherwise identical, single-target test weapons, one
  psychic-damage and one berserk-causing, each with positive versus and sufficient range.
  Against enemy building variants, record whether an attack order produces a shot and whether
  the hit changes health or berserk state. Expected: the psychic-damage weapon can fire at
  all variants, but damages only the explicit-off building. The berserk weapon does not fire
  at omitted or explicit-on buildings; it can fire at the explicit-off building, which still
  cannot become berserk. Keep other firing and order conditions legal (S-0301 claim 9).
  Void a run in which another eligibility condition prevents the explicit-off control case.
  Any omitted variant matching the wrong explicit variant, refusal of the psychic shot solely
  for its immunity, or a berserk shot at an immune building falsifies the corresponding part.
  Remove the named test data after the runs.

- **P9. Countdown placement in the update (Q-1180).** Use the setup of P2 (a human-owned tank; a
  chaos drone fires once and withdraws; compute *T* under S-0302 claim 10) and record, tick by
  tick, the expiry tick (the first tick on which the tint is gone and an order is accepted), every
  shot fired by the tank with its target, the tank's position and destination, and its queued and
  current behaviour where the recorder can read them.
  (a) *Last berserk shot.* Make a durable object of the tank's own house, and nothing else, the only
  target in range. Choose the tank's weapon (reading the stock rules, copying no value) so that
  its rearm time is long enough that the tank fires at most once in any three ticks while
  berserk; if no stock weapon gives that spacing, this part is not measurable. Record each run's
  seed without setting it (measurement setup). A run is valid only if the recorder shows a shot by
  the tank at the own-house object on the tick before the expiry tick; a run without one is void,
  not a pass. Repeat until three valid runs are recorded. Expected: no shot at the own-house
  object on the expiry tick or later (the target is dropped and ordinary rules refuse it); shots
  at it up to the tick before expiry. Falsified by
  one shot at the own-house object on the expiry tick.
  (b) *Queued behaviour.* Put an enemy durable unit far outside the tank's weapon range but in
  the unit's reach, and order the tank to hunt it before the hit (a berserk unit takes no orders, claim 6,
  and the hit only queues *hunt*, claim 4). Expected: at expiry a vehicle keeps
  moving under hunt (it is not ready to change while moving) and stops hunting only when it stops.
  Repeat with an infantry unit: it keeps its hunt move through the expiry tick and takes no
  guard-mode action (no target acquired by a guard scan and no hold) in the expiry tick; the
  earliest guard-mode action is on a later tick. Whether a ready infantry unit shows the queued
  behaviour as current at the end of the expiry tick (S-0215 claim 11) is the proposed measurement S-0215 B46,
  which states its expected value and falsifier. That measurement covers only an infantry unit that is moving
  and ready on the expiry tick; the start order for any other ready unit is unmeasured. A run of either kind is
  valid only if the recorder shows the unit moving on the expiry tick; an infantry run is also valid only under
  the conditions of S-0215 B46. Any other run is void. Falsified by a guard-mode
  acquisition or hold in the expiry tick, an infantry unit that stops its hunt move before the end of
  the expiry tick, or a vehicle that stops moving under hunt, or changes its behaviour, before it
  stops of its own accord.
  (c) *Replaced queue.* Keep a vehicle moving (not ready) from the hit to the expiry by giving it
  a destination through a map-placed script or a path that the hit does not interrupt; if no stock
  setup does that, this part is not measurable. Expected: after expiry the vehicle never hunts
  and holds as *guard* once it stops. Falsified by a hunt after expiry.
  (d) *Excluded updates.* Make a unit berserk with a *T* whose expiry falls inside the unit's erasure:
  the expiry tick comes after the first tick on which it is being erased and before the tick on which
  the erasure removes it. Use a stock berserk warhead if one gives such a *T*; otherwise use the test
  warhead P9-long-berserk, named test data removed after the run: a berserk-causing warhead whose *T*
  under S-0302 claim 10 falls inside that window. Read the removal tick from S-0514's erasure
  measurement scenario, with stock data, and void a run in which removal comes first, since a removed
  unit shows no expiry. Then have a chrono-erasing weapon erase it, recording each tick on which it is
  being erased. (A tunnel variant is not
  measurable: a berserk unit takes no orders, S-0214 claim 15, so no known setup gets it into a
  tunnel while berserk; run it only if a named setup is supplied.) Compare the expiry tick with an unexcluded twin hit in the same tick. Expected:
  the expiry is later than the twin's by the number of excluded updates between the hit and the expiry
  (every update on which the unit is being erased or is in its leaving stage); falsified by equal expiry ticks.
  (e) *Warp.* Make two identical berserk units, one inside a superweapon warp (S-0513 SW23's setup),
  and compare expiry ticks. Expected in the original for the leaving-stage updates: later by that
  many updates. Whether the original takes the step in the other warp updates is unknown and is not
  decided by this part. The engine expectation, later by every warp-only update of S-0513 claim 51,
  is checked by replaying this setup on our engine, not on the stock executable. Falsified by equal
  expiry ticks in the original's leaving stage, or by an engine expiry that is not later by every
  warp-only update.
  (f) *Passenger.* Not measurable in ordinary play (claim 5); do not run unless a setup that
  puts a berserk unit inside a transport is supplied by named test data. Expected then: time
  kept inside, resumed after leaving.

## Open questions
- Whether any ordinary play can put a berserk unit inside another object (claim 5, out of play):
  none is known, so the paused-countdown rule there is derived from the update order alone.
- Whether a queued *guard* or *hunt* that expiry leaves becomes current at the late start of the
  expiry update (claim 5, Effects). S-0215 claim 11 gives that order as an example, and backs it by
  S-0215 B46 only for an infantry unit that is moving and ready on the expiry tick. For any other ready
  unit no measurement yet decides it, so that case stays open.
- Whether the original runs the countdown in the pending, departure, off-map, placement, arrival and hold
  updates of a superweapon warp member (claim 5). Our engine suppresses it in the pending, departure,
  off-map, placement and arrival updates by design, and runs it in hold after each hold-end test (S-0513
  claim 51).
- Whether the original runs the countdown of a landed aircraft being erased. S-0514 claim 17 and
  S-0215 claim 2 exclude its movement, turning, firing and behaviour processing, and say nothing of the
  countdown. Until a measurement settles it, claim 5 takes no countdown step there. The same holds
  for a landed aircraft warp member phasing in with a temporal beam attached (S-0513 claim 39a).
- (Low) Whether a psychic sensor that leaves the map but is not destroyed stays a sensor of its
  owner (claim 9). It cannot be in operation then, so no observable depends on it as far as known.
- Exact line-colour animation, and whether its cosmetic appearance needs a separate rendering
  claim (claim 11).
- Where a negative-amount berserk hit falls relative to the countdown step (claim 6, S-0302 claim 10
  and S-0203 claim 5). Scenario P3 measures it (its negative-versus repeat and hit timing on either side
  of the target's update); until that is run, it stays open.
- Whether a late start falls back to rest (Q-1166, question 1). This spec does not answer it.
- The aircraft row of claim 7 is marked unmeasured. It needs a dedicated setup before it can
  acquire measurement backing.
