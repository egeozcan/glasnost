---
id: S-0607
title: Magnetic lift (seizing a vehicle's locomotor, lifting, dropping and the fall)
topic: combat
revision: 38
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-1320, Q-1321, Q-1322, Q-1323, Q-1324, Q-1325, Q-1326, Q-1327, Q-1328, Q-1329, Q-1330,
Q-1331, Q-1332, Q-1333, Q-1334, Q-1335, Q-1336, Q-1337, Q-1338, Q-1339, Q-1340, Q-1341, Q-1342,
Q-1343, Q-1344, Q-1522, Q-2035, Q-2113, Q-2151, Q-2160]
supersedes: []
---

# S-0607: Magnetic lift (seizing a vehicle's locomotor, lifting, dropping and the fall)

## Summary

A shot whose warhead seizes locomotors replaces an eligible vehicle's own movement with jumpjet
movement and links it to the firer, which pulls it in. This spec owns:

- which hits seize and which targets are immune,
- the lift and what a seized unit can and cannot do,
- every way the link ends, and the difference between a unit dropped from the air and one set
  down,
- the fall, the impact on what lies underneath, and the falling damage and its credit,
- the outcomes on water, beach, clear and blocked cells,
- when the unit gets its own locomotor back,
- the magnetic wave's lifetime and how it interacts with a second, building-attacking weapon,
- how the magnetic wave is drawn and where it falls in the draw order (presentation only),
- what a seizing warhead does when it names a locomotor other than the jumpjet's, or none.

Skirmish only. Network games and campaigns are out of scope.

Rules owned elsewhere and not restated here:

- **Firing cycle.** The shared sonic and magnetic wave slot is S-0301 claim 5. The early *rearming*
  answer against the firer's own gripped unit is S-0301 claim 9, rank 4. The weapon chosen
  against a building is S-0308 claim 3, table P row 7.
- **Protection.** Temporary invulnerability refuses a seize: S-0513 claim 27.
- **Movement of a seized unit as a jumpjet.** S-0502 owns it (for another stand-in locomotor, claim
  23 here names the owning specs):
  - claim 10: body facing,
  - claim 12: the arrival row,
  - claim 16: turning a destination into a movement point (claim 7 here),
  - claims 13 to 15: hovering, water and beach, the landing check,
  - claim 18: the crash fall, no spin, zero speeds and the growing crash rate,
  - claim 19: sinking,
  - claim 29: registration as a tracked flying object,
  - claim 30: the height test for being in the air for combat.
- **Other rules that test the seized state:**
  - the approach step: S-0215 claim 37 c and d (claim 11 here for the gripped unit),
  - deployed vehicles: S-0517 claims 9 and 11,
  - cloaking: S-0516 claim 4,
  - spawners: S-0506 claims 6 and 13 (claim 20 here gives what follows for a seized spawner),
  - parasites: S-0614 claim 12,
  - crash start of a released slot aircraft: S-0501 claim 36,
  - turning: S-0206 claim 14,
  - restoration of a teleporting locomotor: S-0207 claim 21,
  - the falling-back table: S-0215 claim 13,
  - a building lifter that stops being in operation: S-0405 claim 9.

## Terms

From the glossary: *jumpjet state* (grounded, ascending, hovering, cruising, descending, crashing,
crashed), *hover slot*, *height above the surface*, *render layer*, *contact*, *team member*,
*temporarily invulnerable*, *locomotor*.
- **Projectile damage:** S-0302 (Terms).
- **Demolition warhead, crushing warhead:** S-0303 (Import mapping).
- **Game-wide death weapon:** S-0312 claim 6.
- **Magnetic-beam weapon, magnetic wave:** S-0301 (Terms).
- **Passive house:** S-0009.
- **Tracked flying object:** S-0303 (Terms), with jumpjet registration owned by S-0502 claim 29.
- **In the air for combat:** S-0502 claim 30. This is distinct from the positive height that
  makes a release a drop (claim 13), and from the render-layer test for seize eligibility
  (claim 3).

New terms:

- **Seizing warhead.** A warhead with the imported locomotor flag. It names a locomotor that it
  installs on its target as a temporary replacement for the target's own. This spec covers the
  case where that locomotor is the jumpjet locomotor.
  Claim 22 says what a hit does when the warhead names another locomotor, or none, and how a unit
  seized with another locomotor moves and whether it is restored.
- **Stand-in locomotor.** The locomotor a seize installs in place of the unit's own: the
  jumpjet locomotor in claims 5 to 21, or another one under claim 22.
- **Movement point:** S-0502 (Terms): the point the jumpjet locomotor flies to, distinct from the
  unit's order-level destination.
- **Heading point** of a mobile object: the point its own movement is heading for at that moment
  (its next movement point, or its position when it has none), as in S-0212 claim 25, table T5.
- **Seized.** A unit is *seized* from a successful seize (claim 5) until a restoration clears it
  (claim 20; for other stand-ins, claim 24). The state survives the end of the grip and a drop.
  - Other specs say "held by magnetic lift" (S-0502), "held by a magnetic beam" (S-0501, S-0506,
    S-0516), "restrained by a magnetic beam" (S-0506), "magnetic grip" (S-0516) and "carried by
    another object's locomotion" (S-0517). All of these mean *seized*.
  - It is distinct from the glossary's *held* (the parasite countdown).
- **Grip, lifter, gripped unit.** The link between a firer and the unit it has seized. The firer is
  the *lifter* and the seized unit its *gripped unit*. An object grips at most one unit, and a unit
  has at most one lifter.
  - A gripped unit is always seized. A seized unit is not always gripped.
  - S-0301's *held target* and S-0405's *held unit* are the lifter's gripped unit.
- **Drop, dropped unit, dropper.** A grip that ends while the gripped unit's height above the
  surface is 1 lepton or more *drops* it (claim 13).
  - The dropped unit records the lifter as its *dropper*, and the lifter's house as the
    *dropper house*.
  - A unit stays *dropped* until it touches down on a clear cell (claim 17) or its fall ends at
    the impact (claim 18).
- **Wave ends, first end, second end.** The two world points between which a magnetic wave is
  drawn (claim 25). The *first end* is the one its ripples spread from.
- **Wave band.** The flat four-cornered area of the battlefield picture that a magnetic wave
  changes (claim 25).
- **Shift direction.** One of the four screen directions right, left, up and down, fixed when a
  magnetic wave is created, along which the wave band displaces the picture (claim 25).
- **Set down.** A grip that ends while the gripped unit's height above the surface is 0 or less
  *sets it down*. A seized unit is also set down when it touches down at the end of a *descending* flight.
  Being set down is what lets a seized unit get its own locomotor back (claim 20).

## Behaviour

### Seizing

1. **A seizing hit does no damage.** When a projectile with a seizing warhead detonates, the
   seize of claims 2 to 5 takes the place of ordinary damage. Neither the target nor anything
   near it takes damage from that hit.
   - A takeover warhead suppresses the seize (S-0601 claim 5).
   - Effects that S-0303 claim 1 lists as independent of this selection still happen.
   - Backing: `measurable` (scenario L2: the target's health before and after the hit)
   - Confidence: high

2. **When a hit seizes nothing.**
   - The hit does nothing at all when any of these holds:
     - the firer has left play,
     - the firer already grips this very target,
     - the firer is itself seized.
   - Otherwise, before the target is tested, the firer's current grip ends as claim 13 says. A
     dropped unit falls. The firer keeps its target. This happens even when the target then
     turns out to be immune.
   - Backing: `measurable` (scenario L9 parts c and e)
   - Confidence: high

3. **Which targets can be seized.** The target is seized only when all of these hold:
   - it is a vehicle or an aircraft. Infantry and buildings are never seized,
   - it is in the ground render layer (S-0102). An aircraft in flight and a jumpjet in the air
     layer are immune; a landed aircraft is not,
   - it is not temporarily invulnerable (S-0513 claim 27),
   - it is not already seized,
   - it is not protected by its factory. A vehicle is protected when it is in contact with a
     building of a war-factory type and that building covers the vehicle's cell,
   - its type's size is strictly less than the projectile damage (F1).
   - Backing: `measurable` (scenario L1)
   - Confidence: high (each condition); medium (the factory-contact condition, whose window is
     short in play)
   - **A hold starts only with a shot.** The conditions above, and claim 2's, are tested when
     the lifter's seizing projectile detonates, against that projectile's target. No range test
     is made then: neither the weapon's range nor its minimum range is tested at the detonation.
     So a unit that is outside the minimum range when the shot is fired and comes inside it
     before the projectile detonates, or that is within the weapon's range at the shot and goes
     beyond it before the detonation, is seized at the detonation when the conditions above hold
     then. The shot itself needs the firing check, whose range test includes the weapon's
     minimum range (S-0307 claim 9). So a mobile lifter whose weapon has a minimum range and is
     not of unlimited range (S-0307 claim 2) does not fire at a unit inside it, and starts no
     grip on it from there. Before the lifter grips the unit, its approach step against such a
     target is the ordinary one (S-0215 claim 37 parts d and f): claim 11's exception covers
     only a gripped unit that is the lifter's target. A unit that arrives at its lifter (claim
     8) is no longer the lifter's target (claim 12, first row). Once it is restored (claim 20),
     the lifter can take it as target again, by an order or by a search, and it is then an
     ordinary target: a later shot can seize it again under this claim.
     - Backing: `measurable` (scenario L19 part a for the refusal inside the minimum range, the
       approach step, the cleared target after an arrival and a second seize after an order
       given once the unit is restored; scenario L19 part e for the seize of a unit that came
       inside the minimum range during the projectile's flight; scenario L19 part f for the seize
       of a unit that went beyond the weapon's range during the projectile's flight; scenario L2
       for the arrival row of claim 12). No range test at the detonation against a landed
       aircraft target is `unmeasured`: no part of L19 fires at a landed aircraft. A target that
       the lifter takes again by a search is `unmeasured`: no scenario here gives the lifter a
       search after an arrival, and the shot that follows meets the same rules as after an
       order.
     - Confidence: high for the refusal inside the minimum range, for the cleared target after
       an arrival, for the second seize after an order, for no minimum-range test at the
       detonation against a vehicle target (L19 part e) and for no test of the weapon's range at
       the detonation against a vehicle target (L19 part f); medium for no range test at the
       detonation against a landed aircraft, for the ordinary approach step before a grip and
       for a target that the lifter takes again by a search

4. **Resource gatherers already on a temporary locomotor.** A target that is a resource-gathering
   vehicle has a special case when its own locomotor is already temporarily replaced and it is not
   gripped. An example is a teleporting harvester while it drives.
   - If that replacement may end at this moment (claim 20's condition), it ends: the vehicle gets
     its own locomotor back and is not seized.
   - Otherwise the vehicle stops and its current behaviour is cleared. It queues *stop* and is
     not seized.
   - Both cases still lose spawned objects and release a naval parasite as claim 5 describes.
   - **Whatever the warhead names.** This case is decided before the warhead's locomotor is
     considered, so it applies in the same way whichever locomotor the warhead names (claim 22),
     whether or not that locomotor can stand in, and also when the warhead names an unknown
     identifier: such a harvester gets this claim's outcome, not claim 22's design outcome.
     Claim 22's outcomes apply only to targets that this claim does not catch.
   - Backing: `measurable` (scenario L1 part f, and L13 part d for other locomotors)
   - Confidence: medium (the two outcomes); high (that this case comes before the warhead's
     locomotor)

5. **The seize.** A seized unit:
   - stops: its route is cleared, it gives up the occupancy mark of its cell, and it stands
     still,
   - starts *guard* with no target and no destination,
   - loses its spawned objects as S-0506 claim 13 says, and its naval parasite as S-0614 claim
     12 says.

   Its jumpjet locomotor is then linked, with the jumpjet fields of the unit's own type. For a
   warhead naming another locomotor, claims 22 and 23 replace this and the destination bullet
   below. If the link succeeds:
   - the firer's previous grip ends (claim 13) without clearing the firer's target,
   - a grip the unit itself held ends (claim 13), clearing the unit's own target,
   - the grip is made, and the unit is given the lifter as its destination. Its jumpjet movement
     turns this at once into a fixed movement point, and its order-level destination becomes the
     cell of that point, not the lifter (claim 7),
   - the unit is deselected. While it is seized it cannot be selected at all: a click on it, a
     band box around it and its control group's command all pass it over, also while it falls
     after its grip has ended (claims 13 and 15), and it can be selected again only once it is no
     longer seized (claim 20). With `quirks.lift_standin_stays_seized` at its default, a unit
     seized with the driving, walking, ship or teleporting stand-in, which claim 24 never
     restores, and a unit seized with the drop-pod stand-in, which gets its own locomotor back
     at touchdown but stays seized, are never selectable again (claim 24),
   - it breaks off a contact with a building,
   - it queues *guard*.

   Ordering is observable: the firer's old gripped unit is released, and starts its fall, in the
   same tick in which the new unit is seized.
   - In the original it also stops being a team member. The original's team system is outside
     this spec's implementation scope.
   - Backing: `measurable` (scenario L2, including contact and cleanup variants; scenario L18
     for the control group's refusal during the grip and for the unit being selectable after
     restoration). That the refusal lasts until the unit is no longer seized, and not only until
     its set-down, is `unmeasured`: L18 makes no probe between the set-down and restoration. Team
     removal is `unmeasured` here: it concerns the original's out-of-scope team system. The
     click and band-box refusals during the grip, and the click refusal during the fall, are
     `unmeasured`: L18's click and band box can falsify them, by selecting the victim, but
     cannot confirm them, since nothing L18 records shows that the click pointed at the gripped
     or falling victim or that the box took it in (S-0214 claim 19 claims no reference point
     for a lifted or falling state). The control group's command does not depend on a screen
     point. The control group's refusal during the fall is `unmeasured`: L18 presses the group
     key only during the grip. The selection refusal for a unit seized with another stand-in
     than the jumpjet is `unmeasured`: no scenario here tries to select such a unit.
   - Confidence: high (stop, guard, grip, destination, deselection; for the jumpjet stand-in,
     the control group's refusal during the grip); medium (the unit being selectable after
     restoration, which is a selection click's outcome and so is rated as S-0214 claim 12 rates
     selection clicks; contact, the click and band-box refusals during the grip, the
     selection refusal during the fall, the control group's refusal during the fall, the refusal
     lasting until the unit is no longer seized,
     the selection refusal with another stand-in)

6. **Seized units of computer-controlled houses.** When the seized unit belongs to a
   computer-controlled house that is not a passive house, it queues *hunt* instead. A
   resource-gathering vehicle is the exception. This describes the original only. Our AI houses
   use the human-house rule; our AI may reproduce the later hunt through ordinary orders after
   restoration. The neutral and special houses retain the original's non-human rule. The queued
   hunt cannot make a seized unit move under its own power or fire (claims 9 and 10). It is
   still current when the unit is restored (claim 20), and the unit then hunts as any hunter.
   - Backing: `measurable` (scenario L11 parts b and e)
   - Confidence: medium

### The lift

7. **The pull.** A gripped unit flies with jumpjet movement (S-0502), with the jumpjet fields and
   absent-key defaults of its own type (S-0502 Import mapping).
   - **Its movement point is fixed at the seize.** The unit rises and cruises towards the point
     produced by applying S-0502 claim 16 to the lifter's point at the moment of seizure:
     - the lifter's point is its heading point when the lifter is mobile (a lifter standing
       still gives its own position), and the centre of its foundation when it is a building.
       A building gives its docking point instead only when its type has the pad flag, the
       repair-depot flag or the vehicle-bunker flag (Import mapping). Every other building,
       a refinery included, gives the centre of its foundation.
     - **The docking point** is the one the building would give the seized unit itself, chosen
       in this order of precedence:
       - with the obsolete-unloading flag, the fixed point of S-0314 claim 18 (the centre of
         the cell displaced by (2, 1) cells from the top-left foundation cell),
       - else with the resource-unloading flag, the foundation centre displaced by +128
         leptons along *x*,
       - else with the vehicle-bunker flag, the bunker's quadrant point on the seized unit's
         side (S-0606 claim 4), from the seized unit's position at the seize,
       - else the docking-place rule of S-0314 claim 18 and S-0404 claim 15: with a
         docking-place count of 0, the foundation centre; with a count of 1, the centre plus
         the sole docking displacement; with a greater count, the centre plus the
         displacement of the place that the seized unit holds through a contact with that
         building, and the foundation centre when it holds none. The seize breaks the seized
         unit's building contact only after this point is taken (claim 5), so a unit docked at
         its own lifter uses its own place.
     - **Height.** The foundation centre carries the building's world height, and a docking
       displacement adds its height. With the jumpjet stand-in only the point's cell matters:
       the search below replaces the point by a cell centre at that cell's terrain height. For
       another stand-in the point is passed on with its height (claim 23).
     - S-0502 claim 16 then takes the centre of a cell found by the nearby-cell search around
       that point's cell (S-0212 claim 25, table T5, the jumpjet movement-point row for a
       vehicle that is not balloon-hover). That search rejects a cell whose occupancy record is
       not empty, so the lifter's own cell, which the lifter marks, normally does not qualify:
       the movement point is normally the centre of a free cell next to the lifter's cell,
       chosen by that claim's visibility preference and tick rule. Bridge cells never qualify.
     - when the search finds no cell, the movement point is the lifter's point itself, with S-0502
       claim 16's no-result consequences.
   - The movement point is not refreshed while the grip lasts. When the lifter moves, the gripped
     unit keeps flying to the point fixed at the seize. A mobile lifter that accepts a new
     destination ends the grip anyway (claim 12).
   - It keeps the body facing it had when seized (S-0502 claim 10).
   - The hovering, water/beach and landing exceptions of S-0502 claims 13 to 15 apply to it
     because it is seized.
   - **Flying-object registration.** When the seize gives it a movement commitment, its first
     eligible jumpjet update takes off and makes it a tracked flying object, even before its
     first rise. S-0502 claim 29 owns the registration's boundaries and exceptions. Ending
     the grip does not end registration: a clear-cell drop stays registered through its
     descent until touchdown; a blocked crash ends registration before its impact effects.
     Leaving the map also ends registration. If the seize's search finds no cell, it has no
     movement commitment and stays grounded (S-0502 claim 16); there is no take-off registration.
   - **In the air for combat.** While on the map it counts as in the air exactly when its
     signed integer height above the surface is at least 208 leptons (S-0502 claim 30), an
     exact comparison with no rounding. This does not depend on its flight state, its own
     type, whether it is gripped, dropped or set down, or its flying-object registration.
     S-0502 claim 30 supplies the bridge-height interpretation. A registered unit below
     that height is not in the air for combat, although the flying-object search can find it.
   - Backing: `measurable` (scenario L2 for the path, facing and the movement point beside the
     lifter; scenario L12 for a lifter that moves during the pull; scenario L15 for registration
     and the combat height test). The no-cell take-off exception is `unmeasured`: no ordinary
     setup establishing that initial search failure is supplied here.
   - Confidence: high (the fixed point, the search, no refresh, which building types give a
     docking point and its precedence, registration and the combat height test),
     medium (the observed cell for a building lifter)

8. **Arrival drops the unit beside its lifter.** When a gripped unit arrives (S-0502 claim 12: its
   horizontal distance from its movement point is strictly less than 20 leptons), these happen in
   the same update:
   - it is placed exactly on its movement point,
   - its target height becomes 0,
   - it enters *descending*,
   - its lifter's grip ends as claim 13 says.

   Normally its height above the surface is still positive at that moment, so it is dropped
   over the cell of its movement point, normally a free cell next to the lifter's: claims 15 to 17 normally find
   that cell clear, and the unit descends and is set down unhurt. The unit falls onto the lifter
   only when the lifter is in that cell at the impact, for example because it has driven there.
   - Backing: `measurable` (scenarios L2 and L12)
   - Confidence: high

### What a seized unit can do

9. **Orders and destinations.** A seized unit ignores every new destination, from orders, approach
   steps or any other rule.
   - A rule that gives a gripped unit *no* destination while it has one (an effective clear in
     S-0502 claim 24's sense) is the exception. That ends its grip without a drop, and the
     unit is set down where it is, at whatever height. Claim 13's jumpjet stop applies to it,
     so when that stop finds a cell the unit flies to the found point and descends and lands
     there. Claim 13's design rule for no cell does not apply: here the grip
     has already ended when the stop runs, and S-0502 claim 17's self-destruction, with its
     quirk switch and its lack of credit, applies unchanged.
   - **A gripped unit with no destination.** A gripped vehicle that has no destination gets
     none again from such a rule: the assignment leaves its destination unchanged, so it is not
     the exception, and the grip holds. Its grip then ends only by another event of claim 12.
     This holds whatever its stand-in: what ends the grip is a change of the unit's own
     destination to none, which its movement does not bear on. For the jumpjet stand-in it is
     also S-0502 claim 24's rule for an unchanged destination.
   - **A balloon-hover type.** S-0502 claim 28's retention is tested for a seized vehicle as for
     any other, before a seized unit's refusals. So when a rule would give a gripped vehicle of
     a balloon-hover type no destination while it holds both a destination and a target, and a
     row of that claim holds, the assignment is rejected: it is not the exception, the grip
     holds, the destination is kept, and the unit's behaviour becomes *attack* as that claim
     says. *Attack* processing then takes the approach step (S-0215 claims 23 and 37 part a),
     with the outcomes below. The step's drop and its failed search clear the target before
     they give no destination, so S-0502 claim 28 cannot retain the destination there, and
     part e's clearing is no assignment at all. How a gripped unit comes to hold a target is
     given below.
   - **The approach step (Q-2151).** A gripped unit takes S-0215 claim 37's approach step as any
     actor does, also when its target is its own lifter (that claim's part c). The step's
     outcomes for it:
     - **A drop** (part d: the target dropped, with the destination) gives the unit no
       destination. While the unit has one, this is the exception above: in that step its
       grip ends, the unit is set down where it is, its own target is cleared and the lifter
       keeps its target (claim 12).
     - **Part e's clearing** (the destination too far from the target) is not the exception and
       not an effective clear. Unlike part d's drop, which assigns the unit an empty
       destination, part e removes the destination directly: it is not an assignment of an
       empty destination, as S-0502 claim 24 says of the factory ascent report's direct
       discard. So that claim's effective-clear rule, with its reroute (S-0502 claim 17), does
       not apply, and neither does this claim's exception. Part e removes the destination and
       nothing else: the grip holds, whatever the unit's stand-in, since the removal is not a
       change by assignment and so not this claim's exception. With the jumpjet stand-in, unless
       the seize's search found no cell (claim 7), the jumpjet movement keeps its movement point
       and commitment (claim 7), so the unit is still pulled in and dropped on arrival (claim 8)
       or by another event of claim 12. How a unit with
       another stand-in then moves is open (Open questions). When that grip ends, a drop still runs
       claim 13's jumpjet stop, which the stop on dropping gives directly, but a set-down does not,
       since its assignment of no destination is then redundant (claim 13).
     - **Part f.** A gripped unit in the air goes on to part f even when part e keeps its
       destination (S-0215 claim 37 part e). A destination that part f gives (a searched cell,
       an approach destination or the target itself) is ignored, as above, and an approach
       destination it takes still leaves the vehicle's approach destinations (S-0215 claim 37
       part f). A failed search that drops the target and
       finds no nearby cell (S-0307 claim 17) gives no destination. While the unit still has
       its destination this is the exception, as for a drop. After part e's clearing, in the
       same step or a later one, it is a redundant assignment and the grip holds.
     - So once part e has cleared a gripped unit's destination, no later drop or failed search
       ends its grip.
   - **How a gripped unit comes to hold a target.** The step needs a target (S-0215 claim 37 part
     c). The seize leaves the unit with none (claim 5), its S-0309 searches find it no new target
     (claim 20), it never retaliates, since its firing check answers *illegal* (claim 10) and
     retaliation needs another answer (S-0309 claim 18), and it cannot be selected (claim 5), so
     no order given after the seize reaches it. These can still give it a target:
     - **An order given in the seize's tick.** An order given to it in the tick of its seize,
       while it was still selectable, is applied after every object's update of that tick (S-0213
       claim 1), so after the seize. That order is applied as to any unit: it sets what S-0213
       table T3 gives that order, subject to this claim's destination rules, and queues the
       behaviour that table gives it, if any (a stop queues none, S-0213 claim 8). A
       destination it gives is ignored, as above. An order that gives the unit no destination
       (for example an attack order, whose destination becomes nothing, S-0213 claim 21) is this
       claim's exception instead: while the unit has its destination, it ends the grip and
       the unit is set down where it is. For a balloon-hover type that assignment can be rejected
       instead (above), since the order sets the target first (S-0502 claim 28).
     - **An attack-move memory.** The seize does not remove an attack-move memory the unit holds
       (S-0213 claim 6): it is not a behaviour order. S-0309 claim 23 part a applies to it in the
       grip at each expiry of the unit's targeting timer, as to any object, and governs what the
       memory does there. The range searches of that part find no new target in the grip (claim 20),
       so a memory that holds only a target can make the memory's target the unit's target again,
       and a memory with a destination finds the unit no target of its own. A target the unit
       already holds, though (from the seize-tick order or the charging scan below), is kept by
       that part's search when the unit is not engaged: the unit then queues *attack* with an
       immediate start and counts as engaged (S-0309 claim 23 part a). Once that *attack* is
       current, its processing takes the approach step (S-0215 claims 23 and 37 part a), with
       the outcomes above. When the unit is engaged, that part's engaged rows apply as for any
       object. While the unit's behaviour is *guard* and it still holds the grip's destination, a
       target the memory gives does not lead to the approach step: *guard* processing does not
       take the step (S-0215 claim 37 part a), a destination that processing gives is ignored,
       as above, and the per-tick rule of S-0215 claim 37 part a needs a unit with no
       destination.
     - **A gripped unit with no destination.** A gripped unit can be left with no destination
       while its grip holds: after part e's clearing (above), or in claim 23's no-cell case. Such
       a unit meets S-0215 claim 37 part a's per-tick rule as any unit does, in every behaviour,
       *guard* included: on a tick on which it holds a target whose cell it cannot reach, it
       takes the approach step, with the outcomes above for a unit with no destination, and it
       drops the target when that part's conditions for a drop hold.
     - **The charging scan in *guard*.** S-0518 claim 14's charging scan makes no firing check,
       and neither it nor *guard* processing makes an exception for a seized unit. So a gripped
       unit in *guard* that has no target and charges buildings (S-0518 claim 14) runs the scan
       as any unit does, over the cells next to the cell it is over. A found building of its own
       house becomes its target, its charging mark is set and *attack* is queued without an
       immediate start (S-0215 claim 24). Whether that *attack* becomes current in the grip turns
       on S-0215 claim 11. Once it is current, its processing takes the approach step (S-0215
       claims 23 and 37 part a), with the outcomes above.
     - **A fall-back to rest in the grip.** *Guard* processing itself gives no fall-back to rest
       (S-0215 claim 24). Neither the damage response nor the fall-back makes an exception for
       a seized unit, though. So when *guard* has the no-threat property but not the zombie
       property (rules data) and the gripped unit survives damage from a known attacker, it
       falls back to rest at once
       (S-0215 claim 16). The effects of S-0215 claims 13 and 15 and of S-0309 claim 23 part b
       apply to it as to any vehicle, the grip's destination counting as its destination. It
       does not get its own locomotor back there, since it is neither set down nor freed
       (claim 20). The outcome depends on its attack-move memory:
       - **A memory that holds only a target.** The attack move is resumed: *attack* is queued
         with an immediate start, and the memory's target becomes the unit's target. Once
         *attack* is current, its processing takes the approach step (S-0215 claims 23 and 37
         part a), with the outcomes above.
       - **A memory that holds a destination.** *Move* is queued with an immediate start, and
         the memory's destination is ignored, as above.
       - **No memory.** The unit queues *move* without an immediate start (S-0215 claim 15,
         row 3, for the grip's destination).

       *Move* processing does not take the approach step (S-0215 claim 37 part a), and neither
       does the per-tick rule while the grip's destination is held.

     Outside the original's team system, which is out of this spec's scope, the ways traced here
     by which a gripped unit takes the approach step are six: the order given in the seize's
     tick; when *guard* has the no-threat property but not the zombie property, a
     damage-triggered fall-back that resumes a memory holding only a target; for a unit that
     charges buildings, the charging scan in *guard* once its queued *attack* is current; for a
     unit with a memory that holds a destination and that already holds a target, the *attack*
     that the memory's search queues with an immediate start; for a balloon-hover type that
     holds a target, a rejected assignment of no destination; and, for a gripped unit with no
     destination that holds a target, S-0215 claim 37 part a's per-tick rule (above). Our
     engine follows the same rules, and the outcomes above govern any way by which a gripped
     unit holds a target. The count describes the ways traced here and binds nothing.
   - **No cell found, with `quirks.jumpjet_no_cell_self_destruct` off (design).** The unit is
     not destroyed, and it never comes down onto a cell that the stop's search has just
     rejected:
     - **In the air** (height above the surface 1 lepton or more): it re-enters *hovering* over
       its current cell (S-0502 Edge cases). It keeps no movement point and its jumpjet movement
       is no longer moving, so it neither descends nor flies anywhere. It stays set down and
       seized. Claim 20 does not restore it while it hovers. At the end of each of its turns it
       repeats the stop's search instead (claim 20).
     - **On the ground** (height above the surface 0 or less): it stays where it is and its
       flight state becomes *grounded*, even if it was *ascending* at height 0. It keeps no
       movement point and is no longer moving. Claim 20 then restores it as any grounded
       set-down unit, at the end of its first turn that comes after the stop.
     - With the switch on the unit is destroyed, and none of this arises.
   - With the jumpjet stand-in, its own locomotor does not return until claim 20 allows it; other stand-ins follow claims 23 and 24.
   - Backing: `unmeasured`: a seized unit cannot be selected (claim 5), so only an order given in
     the seize's own tick reaches it, and no scenario here times an order to that tick: a
     recorder cannot see in which tick an input was taken (S-0213 claim 1). So the destination
     refusals and the no-destination exception have no scenario that reaches them. Scenario L9
     part a is a conditional protocol, not established backing: it cannot tell which of its
     clicks was taken in the seize's own tick, so an order that reaches the unit can falsify
     the seize-tick order bullet but cannot confirm it.
     The switch-off rule for no cell is `unmeasured`: it is a design rule, and the original has
     only the switch-on behaviour.
     The approach-step outcomes (the drop, part e's clearing and the hold after it, the ignored
     part f destinations, the failed search with no nearby cell), the redundant no-destination
     rule, the hold after part e and after a redundant assignment whatever the stand-in, and the
     order applied after the seize are `unmeasured` for the same reason: each
     needs a gripped unit that takes the approach step, which no scenario here makes it take.
     The seize-tick order cannot be timed, and whether the step is then taken in the grip also
     turns on when that order's behaviour becomes current (S-0215 claim 11), which no scenario
     here establishes. The damage-triggered fall-back in the grip, with its three outcomes, is
     `unmeasured`: no scenario here gives *guard* the no-threat property and damages a gripped
     unit. The ways traced to the approach step outside the team system are `unmeasured`: no
     scenario here reaches any of them, as this backing gives for each. The charging
     scan in the grip is `unmeasured`: no scenario here grips a unit that charges buildings
     next to a building of its own house. The *attack* that a memory with a destination queues
     for a target the unit already holds is `unmeasured`: it needs a gripped unit that holds a
     target from the seize-tick order or the charging scan, which no scenario here gives it. The
     balloon-hover retention in the grip, the order in which the step's drop and its failed
     search clear the target and then give no destination (so that S-0502 claim 28 cannot keep
     the destination there), part e's direct removal and its difference from part d are
     `unmeasured` for the same reason as the approach-step outcomes: each needs a gripped
     unit that holds a target, which no scenario here gives it. The per-tick rule for a gripped
     unit with no destination is `unmeasured` for the same reason, and because no scenario here
     leaves a gripped unit with no destination. What tells part d from part e
     is stated as behaviour: part d's drop gives the empty destination as any order does, and
     part e's clearing removes it without an assignment, so only part d is an effective clear
     (S-0502 claim 24). That the
     seize keeps an attack-move memory, the target the memory gives in the grip, and that this
     target does not lead to the approach step while the unit is in *guard* and holds the grip's
     destination, are `measurable`
     through conditional scenario L18 part b, when its victim's target changes in the grip and
     *guard* lacks the no-threat property or has it with the zombie property. In the grip the
     unit's S-0309 searches find no new target (claim 20), and the charging scan finds only a
     building of the unit's own house, so a target there that becomes the bystander's object comes
     from the memory. The target the victim holds after restoration is a check only: an
     ordinary *guard* search can also find that object (S-0309 claim 8), so it cannot confirm
     the kept memory, though a restored victim that never holds it, when no fall-back in the
     grip removed the memory, falsifies it. The
     rest of the paragraph on how a
     gripped unit comes to hold a target cites claims 5, 10 and 20 and S-0309 claim 18, and is
     backed there.
   - Confidence: medium (ignored destinations, the no-destination exception, the redundant
     no-destination rule, the approach step's drop, part e's clearing and the ignored part f
     destinations, the failed search with no nearby cell, the order applied after the seize,
     the damage-triggered fall-back in the grip and its three outcomes, the charging scan in the
     grip, the *attack* a memory with a destination queues for a held target, the ways traced to
     the approach step outside the team system, the per-tick rule for a gripped unit with no
     destination, the balloon-hover retention in the grip, the order
     of the step's target and destination clearing, part e's direct removal and its difference from
     part d, the hold after part e and after a redundant assignment whatever the stand-in, the
     attack-move memory kept by the seize, the target it gives in the grip and that this target does
     not lead to the approach step in *guard* while the grip's destination is held)

10. **No firing.** A seized unit gets *illegal* for every weapon and target, at S-0301 claim 9's
    rank 4 (a reason about the firer's state). This lasts from the seize until it gets its own
    locomotor back, including while it falls.
    - **Deploying and unloading are not refused.** Being seized adds no refusal of its own to
      deploying, undeploying or unloading passengers beyond what S-0517 claims 9 and 11 already
      say, while gripped or while set down before restoration. Those actions go through their ordinary checks (S-0517 claims 9 and 11, and
      the transport specs), which can still refuse them for other reasons, such as height.
    - Backing: `measurable` (scenario L9 part b; part g for deploying and unloading).
      The exact refusal rank is `unmeasured`: these scenarios observe the refusal, not
      precedence between hidden refusal reasons that prevent the same shot.
    - Confidence: high (the refusal); medium (the rank); low (deploying and unloading)

11. **The lifter's own weapons while it grips.**
    - **Against its gripped unit.** A firing check by the lifter against its own gripped unit
      answers *rearming*, with any weapon. This answer comes ahead of every test of the target's
      layer, height, kind or armour and of every projectile test, so a gripped unit that is in the
      air never gets *illegal* or *impossible* from its lifter through those tests. Of S-0301 claim
      9's rank-4 answers it is preceded only by the materialising firer's *rearming*, by the refusal
      of a firer that is bound to a slave master and, for an infantry lifter, by the answers that an
      infantry firer is asked first (S-0301 claim 9, *Grip refusals within rank 4*): the infantry
      healer's *illegal* (S-0310 claim 11), the keep-acting *illegal* against a vehicle that is
      melee-linked to an object other than the lifter, and any answer that S-0310's open question on
      dying and falling sequences settles as asked first. A lifter that gets none of those keeps the
      gripped unit as its target through the pull: neither that check nor any other rule of this
      spec drops it, though S-0215 claim 37 part a's per-tick drop can (below). An infantry lifter
      that gets one of those *illegal* answers against its gripped unit drops it as its target as
      S-0301 claim 9 gives for that firer, at S-0309 claim 29's check or through its firing step's
      reply. The grip stays, as claim 12 has no row for a target change. In every case the grip
      lasts until an event of claim 12.
    - **No approach against its gripped unit.** A mobile lifter takes no approach step against
      the unit it grips while that unit is its target, in every behaviour, unless one of the
      crushing or balloon-hover approaches of S-0215 claim 37 part h applies to the lifter. Those
      come first and are not excepted: such an approach gives the lifter its gripped unit as its
      destination, which ends the grip (claim 12). Otherwise the range test, minimum range
      included, gives the lifter no destination towards or away from its gripped unit, so the
      approach step never ends a grip through the gripped unit (S-0215 claim 37, part c). Q-2113
      question 2 therefore does not arise: outside part h's approaches, no approach step against
      the gripped unit gives the lifter a destination.
    - **The per-tick drop still applies.** The per-tick rule of S-0215 claim 37 part a applies
      unchanged, with every condition and trigger that part states, and makes its own range test
      after the approach step. So a lifter whose target is its gripped unit drops it as its
      target when part a's conditions hold, among them: the lifter has no destination, it cannot
      reach the gripped unit's cell, and the gripped unit is out of its range, minimum range
      included. The grip stays (claim 12 has no row for a target change) and the wave ends
      (claim 21). Its next search may give it a new target, and an approach step against that
      target can give it a destination, which ends the grip.
    - A computer-controlled lifter in *hunt* whose gripped unit stays reachable or in range
      therefore takes no approach step against it while the unit comes in, makes no fire event
      against it
      (*rearming*, above) and keeps the grip until the unit arrives (claim 8) or another event of
      claim 12 happens.
    - A lifter with a magnetic-beam weapon gets *rearming* when all of these hold:
      - it grips a unit,
      - its magnetic wave is alive (claim 21),
      - the target is not its gripped unit,
      - the target is not a building.
    - Against a building, a weapon is not refused for this reason. S-0308 claim 3,
      table P row 7 governs secondary-weapon selection against a building.
    - When the lifter turns to a building its target changes. That ends the wave (claim 21) but
      not the grip: the gripped unit keeps being pulled towards the lifter, and the lifter is
      still refused against other non-building targets until the wave's fade-out ends.
    - A magnetic-beam weapon is not otherwise held back by its own live wave. Apart from S-0301
      claim 5's shared slot, a live wave refuses only the cases above.
    - This *rearming* belongs to S-0301 claim 9's rank 4, before the ordinary countdown and
      own-effect gates of rank 5. It comes after the target's layer and projectile tests, so
      against an airborne target the beam cannot hit, that refusal wins.
    - Backing: `measurable` (scenario L9 parts c, d and f) for firing, target retention and
      wave/grip outcomes. The absence of an approach step is `measurable` (scenario L17) and not yet
      measured: M-0015 records neither destinations nor what ended a hold. The part h exception and
      the per-tick drop are `unmeasured` here: they apply S-0215 claim 37 parts h and a unchanged
      (backed there, part a by its scenario B32 case d), and no scenario of this spec gives a lifter
      a part h approach or makes its gripped unit's cell unreachable while the unit is out of range.
      Exact precedence between refusal reasons with identical observable effects is `unmeasured`:
      those scenarios do not distinguish their rank. An infantry lifter's answers ahead of the
      gripped-unit *rearming* and the drop they cause are `unmeasured`: no scenario of this spec
      gives a lifter an infantry type.
    - Confidence: high (the conditions, the gripped-unit answer and its precedence); low (an
      infantry lifter's answers ahead of the gripped-unit *rearming* and the drop they cause, as
      S-0301 claim 9 rates them); medium (no
      approach against the gripped unit, the part h exception, the per-tick drop, the
      building-attack sequence)

### Ending the grip

12. **What ends a grip.** Each event in this table ends the grip at once.

    | Event | Lifter's target cleared? |
    |---|---|
    | The gripped unit arrives at its lifter (claim 8) | yes |
    | The lifter is a mobile unit and accepts a non-empty destination (from an order, an approach step or another rule; an approach step against its gripped unit gives none, claim 11, except S-0215 claim 37 part h's) | yes |
    | The lifter is a vehicle that falls back to rest, unless an override is in effect (S-0215, Terms) or its current behaviour is *wait* | yes |
    | The lifter is a building that stops being in operation (S-0405 claim 9) | yes |
    | The lifter is a building that is destroyed | yes |
    | The lifter's seizing projectile detonates again at any target (claim 2) | no |
    | The lifter seizes another unit (claim 5) | no |
    | The lifter leaves play | not applicable |
    | The gripped unit leaves play | yes |
    | The gripped unit is given no destination (claim 9) | no |
    | The gripped unit's drop-pod stand-in ends (claim 23) | yes |

    Ending a grip never clears the *seized* state. Restoration clears it under claim 20
    (claim 24 for other stand-ins).
    - A lifter whose grip ended without clearing its target may fire at the same unit again. That
      unit is still seized, so the hit seizes nothing (claim 3) and does no damage (claim 1).
    - Backing: `measurable` (L2 arrival, L3 accepted destinations and rest, L7 leaving play,
      L8 building rows, L9 new hits, L13 part c for the drop-pod row in the original). The
      no-destination row retains claim 9's `unmeasured` backing for orders; L13 part b observes
      it through the teleporting stand-in's jump (claim 23).
    - Confidence: high (arrival, orders, a new seizing hit, leaving play); medium (falling back
      to rest, building rows)

13. **Drop or set down.** When a grip ends:
    - **Height above the surface 1 lepton or more:** the unit is dropped.
      - It is marked as crashing, with the lifter as its dropper and the lifter's house as its
        dropper house.
      - Its orders stop (the **stop on dropping**): its destination becomes none, its route is
        cleared, its target is cleared, it breaks off any contact and it is deselected.
      - This is not a crash start in the sense of S-0312 claim 16: its passengers stay in, and no
        tumbling draws are taken.
    - **Height 0 or less:** the unit is set down and its destination becomes none, except as
      below for a balloon-hover type.
    - **The jumpjet stop.** In the same update S-0502 claim 17's stopped-movement rule runs on
      the unit's jumpjet movement:
      - **On a drop it always runs.** Besides assigning the unit no destination, the stop on
        dropping gives its jumpjet movement a direct instruction to stop. That instruction is not
        a destination assignment, so S-0502 claim 24's rule for redundant assignments does not
        bear on it: the stop runs whether or not the unit still had a destination.
      - **On a set-down it runs only through the destination's clearing.** A set-down gives no
        direct instruction: it only assigns the unit no destination. That runs the stop only when
        it is an effective order-level clear (S-0502 claim 24), which it is whenever the unit
        still has a destination, except as below for a balloon-hover type. A set-down unit that
        already has none takes no stop: the assignment is redundant, and its jumpjet movement
        keeps its movement point and its movement commitment. With the jumpjet stand-in, a
        gripped unit is left with no destination only by S-0215 claim 37 part e's clearing in the
        grip (claim 9). It is then, unless the seize's search found no cell (claim 7), a set-down
        unit that is still moving (below), and its kept point is the one its pull was heading to
        (claim 7). Claim 23's no-cell case also leaves a
        gripped unit with no destination, but with the teleporting stand-in: that unit has no
        jumpjet movement, so this stop does not bear on it, and it stays where it is (claim 23).
      - **A balloon-hover type.** A set-down gives the unit no destination only after its grip
        has ended, so a rejection of that assignment keeps no grip (unlike claim 9's rejection
        in the grip). A set-down vehicle of a balloon-hover type that still holds a destination
        and a target, when a row of S-0502 claim 28 holds, has the set-down's assignment
        rejected under that claim. It then keeps its destination, movement point and
        commitment, takes no stop, and its behaviour becomes *attack* as that claim says. It is
        then a set-down unit that is still moving (below).
      - **A dropped balloon-hover vehicle.** The stop on dropping gives the unit no destination
        while it still holds its target, and S-0502 claim 28 can reject that assignment as
        above. It then clears the target and gives the unit no destination again, which that
        claim cannot reject. So a dropped one still ends with no destination and takes the
        direct stop. As with S-0502 claim 28's stop, the target's clearing does not undo the
        rejection's change of behaviour: a dropped one whose assignment made while it held its
        target was rejected has *attack* current with no target, and *attack* processing with
        no target falls back to rest (S-0215 claim 23).
      The stop's results for a seized unit:
      - **Not moving.** If the jumpjet movement is not *moving*, nothing of that rule happens.
        A gripped unit is not moving only when the seize's movement-point search found no cell
        (claim 7); it is then still grounded, so its grip ends with a set-down, not a drop.
      - **A cell is found.** The unit's movement point becomes the point that rule produces
        (its re-order through S-0502 claim 16, whose search takes no draw). Its flight state,
        speeds and target height are not changed by it, its order-level destination stays none,
        and no reservation is taken. The unit keeps its movement commitment (S-0502's term): its
        jumpjet movement stays *moving*.
        - **Where a dropped unit falls.** The point decides nothing about where it falls
          horizontally: off its cell centre, claim 15's detour replaces it before the unit is
          steered by it; over its cell centre (always so after claim 8's arrival, which places
          the unit on a cell centre) the fall begins with zero horizontal speed and the point
          is kept.
        - **How high it is held during the fall.** A kept point in another cell is not
          inert. While the unit is crashing over a cell other than its point's cell, S-0502
          claim 9's look-ahead ground reference applies even at zero horizontal speed. Only
          its own cell's clearance (as S-0502 claim 9 defines it) contributes, not a
          neighbour's. Its ordinary height step is taken towards that clearance rather than
          the direct ground reference, before S-0502 claim 18's extra crash drop. Its
          vertical motion can therefore differ from a fall whose point lies in its own cell,
          for example when another object is listed in its own cell. A dropped unit with no
          movement point (no cell found, below) uses the direct ground reference throughout.
      - **No cell is found, grip ended by its lifter (design).** This covers every event of claim
        12 except a gripped unit given no destination (claim 9, below). S-0502 claim 17's
        self-destruction does not happen, whatever `quirks.jumpjet_no_cell_self_destruct` says.
        In both cases the unit keeps no movement point (the one fixed at the seize is cleared)
        and loses its movement commitment: its jumpjet movement is no longer moving.
        - **A set-down unit** stays where it is. Claim 20 restores it at the end of its own turn
          in that tick when that turn comes after the grip's end in the update order (S-0203),
          and otherwise at its next test (claim 20).
        - **A dropped unit** falls as claim 15 says. Over its cell centre it falls with no
          movement point, so S-0502 claim 9's direct ground reference applies to its height,
          not the look-ahead. Off its cell centre, claim 15's detour gives it that centre as its
          point, which lies in its own cell. Losing its commitment does not hold back its fall:
          it takes its jumpjet update every tick until its fall ends, also in a tick in which
          it is *hovering*, so a unit dropped while hovering still starts claim 15's fall in
          its next update.
        In the original the self-destruction kills the unit while its lifter is still letting go
        of it, and the letting go then fails: the game stops with a fatal error. No credit
        question arises, because the original never gets past that point.
      - **No cell is found, no destination (claim 9).** There the grip has ended before the stop
        runs, and S-0502 claim 17's self-destruction applies unchanged, with its quirk switch and
        no credit. With the switch off, claim 9 says how the unit then hovers or stays grounded.
      - **A set-down unit that is still moving.** A unit set down while its jumpjet movement is
        still moving (for example released in the tick of its seize, before its first jumpjet
        update) keeps the found point and its movement commitment; being set down clears
        neither. It is a committed grounded jumpjet under S-0502 claims 5 and 24: its next
        eligible jumpjet update takes off, and it then flies to the point and lands there by
        S-0502's ordinary flight and arrival rules. It stays set down throughout, and claim 20
        restores it only after that commitment has ended.
    - Backing: `measurable` (scenario L3 runs a, b and e) for the drop, the set-down and the
      found point's lack of effect on where the unit falls. The still-moving set-down and the
      kept point's effect on the fall's height are `measurable` through conditional scenarios
      (L3 runs b and f, which use S-0502 claims 5, 9 and 24). Whether ordinary play can release a
      unit before its first jumpjet update, or keep a point in another cell over a cell with
      an allowance, is not established; a run that does not reach those conditions is
      inconclusive. The no-cell rule is `unmeasured`: it is a design choice, and the original
      cannot be observed past it. That the stop runs on a drop and on a set-down from an
      effective clearing is backed with the drop and the set-down above. That a drop runs it
      whatever the destination, and that a set-down after part e's clearing takes none, are
      `unmeasured`: both differ from the backed cases only for a gripped unit that part e has
      left with no destination, which needs a gripped unit that takes the approach step (claim
      9's backing gives why no scenario here sets that up). The balloon-hover set-down and drop,
      the drop's two assignments and its *attack* left current, are `unmeasured`: the
      balloon-hover cases need a gripped unit that holds a target (claim 9's backing). The
      order of the drop's assignments, the first while the target is held and the second after
      its clearing, is `unmeasured` with them.
    - Confidence: high (drop, set-down, which stop runs, that the original fails with no cell,
      take-off and kept commitment of a still-moving set-down unit, the zero-speed look-ahead),
      medium (the found point's lack of effect on where the unit falls, which rests on the
      detour and zero speed; that a drop runs the stop with no destination left; that a set-down
      with no destination left takes none; the direct instruction on a drop; the balloon-hover
      set-down and drop, with the drop's two assignments, their order and its *attack* left current)

14. **A dropper that leaves play.** When a dropper leaves play before its dropped unit lands, the
    unit no longer has a dropper, but keeps its dropper house. Credit under claims 16 to 18 then
    goes to that house.
    - A mobile lifter that leaves play while it grips drops the unit (claim 12) just before.
    - Backing: `measurable` (scenario L7)
    - Confidence: medium

### The fall and the impact

15. **The fall.** A dropped unit falls in two stages.
    - **To its cell centre.** If it is not exactly over the centre of its current cell when its
      crash would begin, its destination becomes that centre first and it keeps flying under
      S-0502's cruising rules. Its crash begins only once it is over the centre. The arrival snap
      of S-0502 claim 12 puts it there.
    - **The fall itself.** From there it falls by S-0502 claim 18 (no spin, speeds 0, a growing
      crash rate).
      - **Starting rate.** The crash rate starts at the unit's own type's jumpjet crash value
        (S-0502 Import mapping), narrowed to single precision. It starts afresh at every seize, so
        a unit seized a second time does not keep the rate it reached before.
      - **Each update.** S-0502 claim 18's extra crash drop uses the current rate (the resulting
        height truncated toward zero), and only after that drop does the rate grow by 1.0, the
        new rate narrowed to single precision. The first update's extra drop is therefore the
        starting rate itself.
      - **Target height.** While a seized unit is crashing, its jumpjet target height is 0 in
        every update.
    - **When the fall ends.** It ends when its height above the surface falls below 1 lepton, or
      when its fall crosses a bridge deck from at or above to below. Claims 16 and 17 then apply,
      in that order, in the same update.
    - Backing: `measurable` (scenario L4)
    - Confidence: medium (the detour, which is the carried S-0502 item); high (the hand-over to
      S-0502 claim 18, the starting rate, the order of drop and growth)

16. **What lies underneath.** When a dropped unit's fall ends, the first object listed in its
    cell's ground layer, in S-0209 claim 13's listing order, other than the unit itself decides
    the result. The ground layer is read even when the fall ended on a bridge deck (quirk below):

    | First other object | Result |
    |---|---|
    | none | The cell is not blocked by an occupant; claim 17 decides. |
    | a vehicle or a building | No damage from this rule. The cell is blocked and claim 17's blocked row on land applies, even on water: a unit dropped onto a vessel crashes rather than sinks. |
    | an infantry unit | That infantry unit and every object after it in the cell's list are removed at once, without death effects or death animations. The crush sound of each one's type plays at the dropped unit's position. Each removal is credited as a kill to the dropper, else to the dropper house, else to the dropped unit. Claim 17 then decides. |
    | anything else (for example a landed aircraft or a tree) | It takes damage equal to its current health with the crushing warhead, ignoring defences. The attacker is the dropper; with no dropper, the attacker is the dropped unit and the house is the dropper house. Claim 17 then decides. |

    - Backing: `measurable` (scenario L5)
    - Confidence: high (infantry, other objects, the credit), medium (vehicles and buildings,
      the listing order in mixed cells, the bridge-deck case). Removal of later non-infantry
      objects has high confidence as a rule but `unmeasured` backing for reachability: no legal
      setup establishing such a mixed list at impact is known. L5 gives a conditional test.

17. **Water, beach, clear and blocked.** After claim 16, unless a vehicle or building made the
    cell blocked:
    - **Water and beach.** The cell counts as water when its land type is water and the unit did
      not land on a bridge deck. It counts as beach likewise.
    - **Blocked.** The cell is blocked when the entry outcome that S-0209 gives the dropped unit
      for that cell (claim 5's table for a vehicle) is anything other than *clear*: *cloaked
      blocker*, *moving blocker*, *closed friendly gate*, *friendly obstacle*, *enemy obstacle*,
      *stationary friendly* and *blocked* all count. A crushable occupant whose row gives *clear*
      (rows VR2 and VR3 of S-0209 claim 5) does not block. Since claim 16 has already dealt with
      the first other object, this test decides in practice only for a cell whose list was empty,
      held a tree or a landed aircraft, or was emptied by claim 16's erasure, and for a deck.
      A dropped aircraft or jumpjet type also faces an extra clearance test:
      use the bridge-deck occupancy record when the cell has a high bridge, otherwise the
      ground record (S-0209 claim 1). A building-present mark or a wall overlay fails this
      test; infantry and vehicle/object marks do not. Away from a high bridge, a zero wheeled
      terrain-speed multiplier also fails it (S-0211 claim 2). A failed test counts as blocked.

    | Cell | Result |
    |---|---|
    | blocked, water and not beach | The unit is set sinking (S-0504 claims 4 and 5) and stunned. The stun repeats claim 13's stop on dropping, so it adds nothing observable beyond S-0504's sinking. Its slaves are freed by the death of their controller, credited to the dropper. Its own coming death is credited to the dropper, else to the dropper house. It does not take claim 18's falling damage. |
    | blocked, on land or beach | The unit ends its fall in *crashed*, releases its hover slot, and claim 18 applies. |
    | not blocked | The unit releases its hover slot and enters *descending* with no destination. At touchdown it is set down and is no longer dropped or crashing. It survives unhurt (claim 20 then restores it). |

    - Backing: `measurable` (scenario L3 for clear land, L6 for water, L5 and L4 for blocked)
    - Confidence: high (clear land and water, the blocked outcomes); medium (beach, decks,
      aircraft)

18. **Falling damage.** A dropped unit that ends its fall in *crashed* receives one impact
    notice.
    - **Vehicle types that can crash.** If its type is crashable, the ordinary crash end of S-0312 claim 16
      applies instead and none of this claim does.
    - **Otherwise,** while it is seized, its impact has these effects:
      - The impact produces two detonations using the projectile of the game-wide death weapon
        and damage *D*ꜰ (F2). The source is the dropper, or the falling unit with no dropper.
        The first is at ground level (the unit's world height less its height above the surface)
        with the crushing warhead. The second is at the unit's own position with the demolition
        warhead. Ordinary area damage applies (S-0303 claims 6 to 12), so the falling unit,
        its lifter if underneath, and neighbours can all be hurt.
      - **No death weapon (design).** The original reads the game-wide death weapon without
        checking that one is configured, and stops with a fatal error when none is. Our engine
        warns at import, and each of the two detonations becomes a plain area detonation of its
        warhead at its point, with the same amount *D*ꜰ, source and house, and no projectile
        effects.
      - Impact destruction is credited to the dropper, else to the retained dropper house.
      - The falling unit leaves play after these effects even if both detonations left it
        alive (quirk below).
    - **Ordering.** A neighbour destroyed by the first detonation is absent for the second. Both
      use the same pre-impact damage amount even if the first changes the falling unit's health.
    - **Credit when a detonation kills the falling unit** (`quirks.lift_impact_double_credit`,
      default on). The impact credit is given whether or not the unit is already dead. A falling
      unit killed by either detonation is therefore credited twice: once to the detonation's
      source under the ordinary rules, and once more by the impact credit. Both credits count in
      the kill and loss tallies, the score and the experience of the credited side. A survivor
      removed under `quirks.lift_impact_removes_survivor` is credited once. Fixed behaviour:
      the impact credit is skipped for a unit that a detonation has already killed.
    - **Passengers.** A dropped transport that a detonation kills loses its passengers under the
      ordinary rules for a dying object (S-0312 claim 5). A transport that survives both
      detonations and is then removed loses its passengers without attacker credit (S-0507
      claim 16).
    - Backing: `measurable` (scenario L4 and its test-data variants); the missing-weapon
      departure is `unmeasured` because it is a project design choice, not an observation of the
      original.
    - Confidence: high (the two detonations, shared amount, removal, the double credit and the
      passenger rule); medium (the credit of a survivor's removal).

19. **Dropped aircraft.** A seized aircraft that ends its fall in *crashed* takes the same falling
    damage, but its first detonation uses the demolition warhead. Claim 18's crashable-type
    vehicle exception does not apply to this aircraft impact.
    - Backing: `measurable` (scenario L4 part d, named test data)
    - Confidence: medium

### Getting its own locomotor back

20. **Restoration.** A seized unit's temporary jumpjet locomotor ends, and its own locomotor
    returns, at the first restoration test at which all of these hold:
    - its jumpjet movement is not moving (S-0502's *moving*, not *moving now*),
    - it is set down, or it is no longer seized,
    - it is not a seized set-down unit hovering with no movement point and no movement
      commitment after S-0502 claim 17's no-cell outcome with
      `quirks.jumpjet_no_cell_self_destruct` off.

    **When restoration is tested.** The test is made at the end of each of the unit's turns
    (S-0203): after the point at which it may
    process its behaviour (S-0215 claim 2), whether or not its behaviour is processed in that
    turn, and after its movement. A turn in which its processing timer has not expired is
    tested like any other. A tick in which the unit takes no turn (a turn lost to a removal,
    S-0203 claim 10) has no test for it, and the next test is at the end of its next turn. The
    test is made whatever the unit's current behaviour is and whichever house owns it.
    Whether some other states keep the test from being made is open (Open questions). The
    same test is also made whenever the unit falls back to rest (S-0215 claim 13). Both ask
    the same question, so what can be observed is the first test at which the conditions hold.
    - **At touchdown.** A unit that touches down on a clear cell (claim 17's *not blocked* row)
      stops moving and is set down in its own movement in its own turn, so it is restored at
      the end of that same turn. A unit set down by a grip that ends at height 0 or less with
      no movement commitment is restored at the end of its own turn in that tick when that turn
      comes after the set-down in the update order (S-0203), and otherwise at its next test.
    - **Behaviour and house make no difference.** A set-down unit in *guard* (claim 5) and one in
      *hunt* (claim 6) are restored at the same tick, for every house. The house matters only
      for the queue that restoration adds for a gatherer (below).
    - **Units being erased.** A seized infantry unit or vehicle is not restored while it is
      being erased (glossary) or is in the leaving stage of a chrono move (S-0215 claim 2): it
      stays seized for as long as either holds, even when it is set down and not moving. One
      that survives is restored at the end of its first turn after that ends at which the
      conditions hold. For a landed aircraft being erased, which S-0215 claim 2 already keeps
      from ordinary movement, whether the test is made is not stated (Open questions).

    **The hovering set-down unit (design).** Every seized set-down unit in that switch-off
    outcome follows this rule, whatever route brought it there, not only claim 9's
    no-destination set-down. Restoration is not tested for it, neither at the end of a tick nor
    at a fall-back to rest. At the end of each of its turns the unit repeats the stop's
    search instead: S-0502 claim 17's nearby-cell search around its current cell, for its type's
    own locomotion class and movement zone, with no draw.
    This includes a set-down unit that kept its movement commitment, flew to its point, and
    subsequently reached that no-cell outcome through a failed landing or another stop.
    S-0502's switch-off height test applies: height above the surface 1 lepton or more hovers.
    Height 0 or less stays put and becomes *grounded*, even from *ascending*.
    - **A cell is found.** It re-orders itself to that cell's centre through S-0502 claim 16,
      exactly as claim 13's stop does. It is then claim 13's set-down unit that is still moving:
      it flies to its point, descends and lands there, and is restored at the end of the turn in
      which it lands.
    - **No cell is found.** It keeps hovering and is not restored. It is not destroyed: the
      switch is off. It searches again at the next test, so it lands once a cell becomes free.

    Restoration clears *seized*, *set down* and the dropper. A seized resource gatherer of a
    computer-controlled house then queues *harvest* in the original. Our AI houses use the
    human-house rule and may issue an ordinary harvest order after restoration; the neutral
    and special houses retain the original's non-human rule.

    So a gripped unit hovering over its lifter is never restored. Subject to the hovering
    exclusion above, a unit set down by claims 9, 13 or 17 is restored at the end of the first
    of its turns after it is set down at which its jumpjet movement is not moving.
    - **Set down does not pre-empt movement.** Being set down, or being grounded, does not
      by itself restore a unit whose jumpjet movement still has a movement commitment (claim
      13's still-moving set-down). That unit takes off and flies to its point first (S-0502
      claims 5 and 24), and is restored at the end of the turn in which the commitment ends.
    - **Before restoration.** Until it is restored, the unit's S-0309 searches for a target find
      it no new target: its firing check answers *illegal* (claim 10) at S-0309 table C's
      firing-check row, so a *hunt* processing in the grip, in the fall or earlier in the
      touchdown turn takes no new target. Keeping a target the unit already holds, and an
      attack-move memory, follow S-0309 claim 23 as claim 9 gives. S-0518 claim 14's charging scan
      makes no firing check and still runs in *guard* (claim 9).
    - **After restoration.** Restoration queues and starts no behaviour of its own, apart from
      the gatherer's *harvest* above. A unit in *hunt* keeps it and is an ordinary hunter from
      its next *hunt* processing (S-0215 claim 27): it searches, takes targets, fires and
      approaches them under the combat specs. It can be seized again (claim 3). Restoration does
      not itself remove an attack-move memory: a memory the seize kept (claim 9) and that no
      fall-back in the grip removed (S-0309 claim 23 part b) is still held, and acts as S-0309
      claim 23 gives for any object.
    - **A spawner while seized and after restoration (Q-2160).** A seized spawner neither fires
      (claim 10) nor launches (S-0506 claim 6) from the seize until restoration: in the grip, in the
      fall and while set down. The seize clears its slots (claim 5), with S-0506 claim 13's
      effects per slot state: a docked or reloading rocket is removed, a preparing rocket is
      released and flies on to its aim point, and a rocket that has already left its slot is not
      affected. A rocket still on the rail vanishes with no detonation while
      `quirks.rail_missile_vanishes_with_spawner` is on (the default). With it off, it is released
      as a preparing rocket is (S-0506 claim 13). The spawner is healthy and in play, so each
      slot the seize empties gets a regeneration countdown of 0 and is refilled at its next pass
      (S-0506 claim 9), seized or not. A slot that was already empty at the seize keeps its own
      countdown. Clearing the slots does not change
      the controller target, and the controller's passes go on while the spawner is seized. With
      `quirks.spawner_pending_target_survives_idle_clear` on, the seize's own-target clear empties
      the pending target only when the spawner's own target was not empty and the controller
      target is not empty (S-0506 claim 25). So, with
      that switch on, a spawner seized after a shot and before the release of the salvo that shot
      asked for keeps a target for its controller: its controller target, or a pending target that
      the controller adopts at a later pass while the spawner is seized (S-0506 claim 5, with the
      range test made from where the spawner then is). While the controller is idle, each of its
      passes repeats that range test from where the spawner then is (S-0506 claim 5), so a reset
      of S-0506 claim 11 can come while the spawner is lifted away. Unless such a reset clears
      that target, the spawner launches at it once it is restored and not moving, with no new
      shot.
      A kept target that leaves play while the spawner is seized is lost as S-0506 claim 25
      gives for any spawner: a lost pending target is cleared at once, and a lost controller
      target is cleared, with a reset when no pending target waits (S-0506 claim 11). No launch
      at a lost target follows restoration.
      - Backing: `measurable` (scenario L19: part b for the kept controller target and the
        launch after restoration, and for a kept target that leaves play while the spawner is
        seized, part d for the kept pending target and an adoption while
        seized, part c for the on-rail and left-slot rocket outcomes and the refill, part g for
        the preparing and docked rocket outcomes). No committed S-0506 claim states
        that the slot clearing keeps the controller target or that the controller's passes go on
        while the spawner is seized: the kept controller target rests on L19 parts b and d only,
        and the passes while seized on L19 parts c, d and g only. The rail outcome with
        `quirks.rail_missile_vanishes_with_spawner` off is `unmeasured` here: the stock game runs
        only the on side. It is S-0506 claim 13's.
      - Confidence: high for no fire and no launch while seized, and medium for the rocket
        outcomes with either switch setting (S-0506 claim 13 rates its list of events medium),
        the regeneration countdown of a slot already empty at the seize, the refill while seized,
        the controller's passes while seized, the kept controller or pending target, the
        launch after restoration with no new shot and the loss of a kept target while seized
    - **The first order after touchdown.** A move order given in any tick after a clear-cell
      touchdown meets a restored unit, so it moves the unit with its own locomotor.
    - **Readiness while seized.** Wherever S-0215 claim 11 asks whether a vehicle is moving on
      this tick, it asks the unit's current locomotor. While seized that is the temporary
      jumpjet movement, so the question is S-0502's *moving now* (every state except *grounded*
      and *hovering*). This holds also for a unit that was driving on a temporary locomotor of
      its own when it was seized: the driving locomotor's moving test no longer applies.
    - Backing: `measurable` (scenarios L3 and L11) for restoration, its timing, the behaviour and
      house independence and the first order. That restoration does not itself remove an
      attack-move memory is `unmeasured`: L18 part b's target after restoration can falsify it
      but cannot confirm it, since an ordinary *guard* search can find the same object (claim
      9's backing), and its part in the grip shows only that the seize kept the memory.
      The charging scan before restoration is claim 9's, `unmeasured` there. That a kept
      movement commitment delays
      restoration is `measurable` through conditional scenario L3 run b (S-0502 claims 5 and
      24); whether ordinary play reaches that case is not established. The being-erased case
      is `unmeasured`: it needs an erasure that starts before a set-down unit's first test,
      which ordinary play is not known to produce. Conditional scenario L11 part f tests it
      when a run reaches that state. The hovering exclusion, retry for
      every route and height-based grounded outcome are `unmeasured`: they are design rules for
      a non-default switch setting, which the original does not have. Scenario L14 checks
      these rules on our engine only.
    - Confidence: high (restoration conditions, no restoration while committed, the
      route-independent switch-off design rule and its height boundary), medium (the test at
      the end of every turn whatever the behaviour and house, restoration in the touchdown
      turn, no target found by an S-0309 search before restoration, the charging scan still
      running before it, an ordinary hunter after restoration, the
      set-down tick of a height-0 grip end, the kept attack-move memory through restoration),
      low (units being
      erased)

### The magnetic wave

21. **The wave's lifetime.**
    - **Creation.** A magnetic-beam weapon's shot creates a magnetic wave only when its firer has
      no live wave (sonic or magnetic, S-0301 claim 5). The wave is not created when the
      horizontal distance between the shot's firing and impact points is under 240 leptons
      (F3). Either way the shot
      itself is fired, so a seize can happen with no wave.
    - **Ending.** The wave has no fixed duration. It ends when its firer or its target leaves
      play, or when its firer's target is no longer the wave's target. The wave's distance limit
      applies to sonic waves only. Leaving play includes going inside a transport, a bunker or a
      building: a target or firer that does so ends the wave at the wave's next update, even
      though it is not destroyed.
    - **When it updates.** The wave joins the update order when it is created, appended after
      every object already in play (S-0203 claims 2 and 3). It has one update at creation; the
      same tick's pass may reach it again under S-0203's rules. Each later tick it updates in its
      own turn, after its firer's turn. Its end test reads its firer's target as it stands at
      that turn, so a retarget made in the firer's turn of the same tick is seen at once.
    - **Its points.** F3 is evaluated once, at creation, from the shot's firing and impact points
      as computed when the shot is fired. Later movement of firer or target does not re-test it.
    - **Fade-out.** After it ends it fades out (F4) and stays alive until the fade-out finishes,
      about 20 ticks after a full fade-in. While alive it counts for S-0301 claim 5 and claim 11.
    - **No damage.** A magnetic wave deals no damage of its own.
    - **Not tied to the grip.** The grip does not depend on the wave. The wave can end while the
      grip lasts (claim 11), and a grip can begin with no wave.
    - Backing: `measurable` (scenario L10) for creation, lifetime, update turn, damage and fade
      timing. The exact accumulator precision and table-root choice in F3 and F4 are `unmeasured`:
      no setup here distinguishes alternative arithmetic with the same visible thresholds.
    - Confidence: high (no damage, end conditions, numeric rules, update turn); medium
      (observable fade timing)

### Other locomotors

22. **A seizing warhead that names another locomotor, or none.** Claims 1 to 4 decide the seize
    the same way whatever locomotor the warhead names: claim 4's resource-gatherer case comes
    first, and what follows here applies only to targets it does not catch. What follows depends
    on that locomotor:
    - **One that cannot stand in temporarily** (hovering, mechanical walking, flying, rocket or
      tunnelling movement; *mechanical walking* is the walking-vehicle locomotor of S-0207 table
      T3, distinct from the infantry walking locomotor below): the hit still has claim 5's other effects on an eligible target (it
      stops, is put in *guard*, loses its spawned objects and its parasite), and still releases the
      firer's previous victim (claim 2). But the target is not seized: no locomotor is installed,
      it keeps moving with its own, and it can be hit again.
    - **One that can stand in temporarily** (driving, walking, ship, teleporting or drop-pod
      movement; *walking* is the infantry walking locomotor of S-0207 table T3): the target is seized as in claim 5, with that locomotor installed as its stand-in
      in place of its own. Claims 23 and 24 say how it then moves and whether it is restored.
    - **No locomotor named.** A warhead whose locomotor key is absent names the locomotor in force
      from an earlier rules layer, else the teleporting locomotor (Import mapping). With no earlier
      layer, a hit therefore seizes with the teleporting stand-in.
    - **An empty value.** A line whose value is empty makes no entry (S-0001 claim 8), so it
      reads as an absent key, as in the previous bullet, and gives no warning.
    - **A malformed value (bad data).** The value is read as S-0207 claim 14 reads a type's
      locomotor field: leading and trailing blanks are removed, and a non-empty value that does
      not have the form of a class identifier (a bare name, or digits without braces) is ignored
      as if the key were absent. The original does this silently. Our importer does the same and,
      on a seizing warhead, gives an importer warning (glossary *importer warning*); on a warhead
      without the seizing flag the value is unused and does not warn. So with no earlier layer,
      such a warhead seizes with the teleporting stand-in.
    - **An unknown identifier (design).** A value of the right form that names no locomotor: the
      original stops with a fatal error when such a hit reaches an eligible target. Our importer
      warns, and a hit is treated as with a locomotor that cannot stand in temporarily.
    - Backing: `measurable` (scenario L13) for the first five bullets; the importer warning and
      the unknown-identifier departure are `unmeasured` because they are project design choices,
      not observations of the original.
    - Confidence: high (which locomotors can stand in, the effects without a seize, the parse of
      an empty or malformed value); medium (the teleporting result with no locomotor named, which
      rests on the read rather than an observation)

23. **How a unit seized with another stand-in moves.** At the seize the unit is given the lifter
    as its order-level destination, exactly as in claim 5, before the seized state begins. The
    stand-in receives the lifter's point of claim 7 (the heading point or position of a mobile
    lifter, the foundation centre of a building, or a docking point) as its movement instruction.
    - **No movement point of its own.** The nearby-cell search and the fixed movement point of
      claim 7 belong to the jumpjet's movement only. With another stand-in the order-level
      destination stays the lifter itself, as an object destination, not the cell of a point.
    - **Occupancy** (`quirks.lift_standin_unmarked_cell`, default on). The unit gives up the
      vehicle-present mark of its cell at the seize (claim 5), and installing the stand-in marks
      no cell:
      - **Driving and ship.** Receiving the lifter's point marks nothing. The unit holds no
        vehicle-present mark until its stand-in starts a step into a neighbouring cell; from then
        on it marks and clears cells as S-0209 claim 1 says for a driving vehicle. A unit that
        has moved and stops keeps the mark of its last cell, as any vehicle at rest does. A unit
        that never moves under its stand-in stands unmarked for good, also after its grip ends.
      - **Teleporting.** Fixing the landing point at the seize moves the mark at once to the
        landing cell (S-0207 claim 20, part a). The unit's own cell stays unmarked until the
        jump, and the landing cell stays marked after it.
      - **Walking.** The unit does not move (below), so it stays unmarked.
      - **Drop-pod.** The unit marks its cell again as its own locomotor returns (below).
      - **What the missing mark changes.** The unit stays listed in its cell, and a vehicle's
        entry outcome (S-0209 claim 5) reads the listed occupants first, so the missing mark
        does not let another vehicle in. While the seized unit has the lifter as its destination
        it counts as moving (S-0212 claim 27) and, before its first step, is between steps
        (S-0209 Terms): an allied mover meets *moving blocker* (row VP22), or *blocked* in
        S-0212 claim 27's close case. Once its destination is gone it no longer counts as moving,
        and an allied mover meets *stationary friendly* (row VP20). An enemy mover meets the
        seized unit's enemy row; a crushable seized unit gives *clear* to a crusher with or
        without the mark (rows VR2 and VR3). The missing mark is seen only by rules that read
        the occupancy record alone, such as the nearby-cell search of S-0212 claim 25: claim 7's
        movement-point search for another seize can choose the seized unit's cell.
      - **Fixed behaviour.** At the seize a unit seized with the driving, walking or ship stand-in
        keeps the vehicle-present mark of its own cell, as a vehicle at rest does. Its stand-in's
        movement then clears and marks cells as usual. The teleporting stand-in is unchanged.
    - **Which spec moves it.** From then on the unit moves under its stand-in's ordinary rules,
      with that destination:
      - driving: S-0207, including claims 25 to 27 for the locomotor destination point, the
        removal of an object destination and refresh,
      - teleporting: S-0207 claim 20,
      - ship: S-0504's ship locomotor.
    - **Walking (not settled, design).** How a vehicle moves with the walking locomotor is covered
      by no movement spec. Our engine keeps such a unit where it is: it does not move or turn
      under that stand-in while it is seized.
    - **Drop-pod (partly settled, design).** In the original the drop-pod stand-in takes the
      lifter's point as the target of a drop-pod descent. The unit is moved up into the air near
      that point, without leaving play, and falls to the ground. Throughout the descent the grip
      lasts: the lifter keeps the unit as its target, gets *rearming* against it (claim 11), and
      its magnetic wave stays alive (claim 21). At touchdown the unit is taken off the map and put
      back at the landing point, the stand-in ends itself, and there are landing effects of its
      own. Taking the unit off the map ends the grip as claim 12's row for a gripped unit leaving
      play does: the unit is set down (claim 13, height 0) and the lifter's target is cleared,
      which also ends the wave at its next update (claim 21). No spec owns the descent itself.
      Our engine instead ends the drop-pod stand-in at once at the seize, with the original's
      touchdown result: the unit stays where it is, gets its own locomotor back, and the grip
      ends with the unit set down and the lifter's target cleared (claim 12's drop-pod row). The
      lifter is then no longer gripping, so claim 11's refusals stop at once. The unit marks its
      cell again as its own locomotor returns. At touchdown the original also makes objects of
      other houses that have the unit as their target drop it, subject to a visibility
      condition that is not pinned down (medium confidence); each object that drops it may take
      one draw of its retargeting delay, from 4 to 8 ticks. Objects of the unit's own house keep
      it as their target. Our engine reproduces none of this, draws included: the unit never
      leaves the map, so every object other than the lifter keeps its target. Claim 24 says what remains of the seized state.
    - **Arrival.** Claim 8's arrival drop is part of the jumpjet's movement and does not happen.
      The grip ends only by an event of claim 12. Its row for a gripped unit given no destination
      applies when the stand-in's own rules remove the unit's order-level destination through the
      ordinary destination-setting rule: the grip ends, the unit is set down and the lifter keeps
      its target. Such removals are:
      - **The teleporting jump.** S-0207 claim 20 removes the destination when the jump is made,
        so the grip ends on the tick of the jump, with the unit set down at its landing point.
        The jump also makes every object that targets the unit drop it (S-0207 claim 20,
        part e), so in this case the lifter's target is cleared after all, in the same tick.
        With no destination, and refusing every new one (claim 9), the unit makes no further jump.
      - **No jump.** When the landing point equals the unit's own position, S-0207 claim 20
        removes the destination at its first movement update without a jump: the grip ends the
        same way, in that update, and the lifter keeps its target. When the landing search finds
        no cell, the destination is removed while the seize itself assigns it, before the seized
        state begins. That removal does not end the grip: the unit stays where it is with no
        destination, never jumps, and the grip lasts until another event of claim 12.
      - **Driving and ship at an exact point.** S-0207 claim 26's guard row removes the
        destination when the unit, at rest in *guard* (claim 5), stands exactly on its locomotor
        destination point in all three coordinates. The lifter's point is normally the heading
        point or position of a mobile lifter, which the lifter itself occupies, or a building's
        foundation centre or docking point with that building's height, so this is rare. When it
        happens, the grip ends and the lifter keeps its target.
      - S-0207 claim 25's give-up row is another example.
    - **A landed aircraft** (claim 3) seized with a ground stand-in receives the lifter's point
      in the same way.
      - **Teleporting.** S-0207 claim 20 covers aircraft. The aircraft's own entry outcome is
        *clear* (S-0209 claim 14), so its landing point is normally the centre of the cell of
        the lifter's point, even when the lifter stands there. It lands at terrain height (or on
        a deck) and the grip ends at the jump as above.
      - **Driving, walking and ship (not settled, design).** No movement spec covers an aircraft
        moving with these locomotors. Our engine keeps it where it stands: it does not move or
        turn under that stand-in, and its grip ends only by a lifter-side event of claim 12.
      - **Drop-pod.** As for any unit, above.
      - **Taking off.** With `quirks.lift_standin_stays_seized` on, such an aircraft never takes
        off again: the driving, walking, ship and teleporting stand-ins never end, and after the
        drop-pod stand-in ends the aircraft is still seized and refuses every destination
        (claim 9). With the fixed behaviour of claim 24 it is restored and can then take off on
        a later order.
    - Claims 9 and 10 apply unchanged: later destinations are refused and it cannot fire. With a
      ground stand-in its height above the surface is normally 0, so a grip that ends sets it down
      (claim 13) rather than dropping it.
    - Backing: `measurable` (scenario L13 parts b, c and e) for the lifter destination, the
      movement towards the lifter, the missing arrival drop, the grip's end at the teleporting
      jump and the lifter's target, the original's drop-pod descent and touchdown, and the
      teleporting aircraft. The occupancy rule is `measurable` (scenario L13 part f). Not
      reproducing the touchdown's target drop for other objects is `unmeasured` as a design
      choice. The original's same-house target retention is `measurable` (L13 part c).
      Its other-house visibility condition and the retargeting draw are `unmeasured`: that
      condition is not pinned down, and L13 part c does not isolate the draw or its trigger.
      The guard-row ending is `measurable` only conditionally (L13 part b, depot
      variant): no layout is established in which a driving unit stops exactly on a docking
      point. The no-cell case of the teleporting stand-in is `unmeasured`: no setup is
      established in which the search around a lifter finds no cell. The walking rule, the
      aircraft rule for driving, walking and ship, and the drop-pod rule in our engine are
      `unmeasured` as design choices.
    - Confidence: medium (the instruction and destination, read rather than observed, the
      occupancy marks, the teleporting jump's end of the grip, the original's drop-pod
      touchdown, the aircraft's teleporting landing); low (the composed course of the drive
      towards an occupied lifter cell, which those specs decide, when a stand-in next marks a
      cell, the composed entry outcomes, the no-cell case and the guard-row case)

24. **Restoration with another stand-in** (`quirks.lift_standin_stays_seized`, default on).
    - With the driving, walking, ship or teleporting stand-in, the unit is never restored. Each of
      these stand-ins refuses to end while its unit is seized, and only the end of the jumpjet
      stand-in clears *seized*. The unit keeps the stand-in until it leaves play: it ignores every
      new destination (claim 9), never fires (claim 10) and can never be seized again (claim 3).
    - With the drop-pod stand-in, the unit gets its own locomotor back when its descent ends
      (claim 23; at once in our engine). It stays *seized*, though, so claims 9 and 10 go on
      applying to it indefinitely, and claim 3 keeps it from being seized again.
    - Fixed behaviour: claim 20 applies to every stand-in, with that stand-in's own moving test in
      place of the jumpjet's, and the restoration clears *seized*, *set down* and the dropper.
    - Backing: `measurable` (scenario L13 part c) for the original's retained seized state.
      The fixed alternative is `unmeasured`: it is a project design choice.
    - Confidence: high (the refusal of the four stand-ins and that only the jumpjet's end clears
      the state, the drop-pod stand-in ending itself without clearing it)

### How the magnetic wave is drawn

25. **The picture of a magnetic wave.** This is presentation only: the picture changes no game
    state, and no game rule reads it.
    - **Its ends.** Every update of a wave that has not ended (claim 21), including its update at
      creation, sets the wave's two ends from current positions:
      - When the wave's target is a vehicle, the first end is the target's centre, and the second
        end is the firer's fire point (glossary) for its first weapon, raised by 50 leptons.
      - For any other target, the first end is that fire point and the second end is the target's
        centre, raised by 50 leptons.

      The fire point is the one of the firer's first weapon even when the magnetic-beam weapon is
      its second. Against a vehicle, the usual case, the ripples therefore spread from the seized
      vehicle towards the lifter.
    - **After the end.** Once the wave has ended, its ends are no longer updated. Through the
      fade-out (F4) the band stays where it last was, fixed in the world, while its ripples go on
      moving.
    - **The band.** The wave band is a flat four-cornered area, 100 leptons wide. Its corners lie
      50 leptons to either side of each end, measured horizontally at right angles to the
      horizontal line from the first end to the second. The two corners at an end have that
      end's height. The band is the quadrilateral between the four corners' screen points
      (S-0101 F1). F5 gives the geometry.
    - **No colour of its own.** The band paints no colour and draws no outline. Each band pixel
      takes the colour of a *source pixel* 0, 1 or 2 pixels away along the shift direction. Its
      blue and red are then boosted and its green is kept (F5). Source pixels are read from the
      picture as it stood before the wave drew, so the boosts never compound and a displaced
      pixel never shows another band pixel's result.
    - **Ripples.** How strongly a band pixel is displaced and boosted depends on its screen
      distance from the first end's screen point and on the wave's visual countdown (F4). The
      result is concentric rings around the first end, about 31 pixels apart, clipped to the band.
      Most of each ring gets the 2-pixel displacement and the strongest boost; only thin rings
      near the zero crossings keep the pixel in place.
    - **Ring motion.** The countdown falls by 1 per tick, so the rings move outward by 1 pixel per
      tick. The countdown starts at 100. When an update finds it at 19 it restarts at 64 (F4), so
      the next drawn value is 63. The ring phase therefore jumps back by 44 steps 82 ticks after
      creation and every 45 ticks after that: the pattern drawn 82 ticks after creation is the one
      drawn 38 ticks after it, and from then on it repeats every 45 ticks. Because the rings repeat
      about every 31 pixels, the jump looks like a step of about 13 pixels inward. S-0203's extra update in the creation tick can
      shift this count by one tick.
    - **Shift direction.** The shift direction is chosen once, when the wave is created, from the
      shot's firing and impact points. It is right or left when the band's screen line from the
      firing point is within 45° of horizontal, and up or down otherwise (F5). It does not follow
      later movement, not even the change of ends in the first update.
    - **No fade.** The fade values *a* and *b* of F4 do not affect the picture. A wave is drawn at
      full strength from its first drawn frame until it is removed, and then vanishes at once.
      The visual countdown sets only the ring phase.
    - **Depth.** A band pixel is drawn only where it passes the depth test as a reader (S-0102
      claim 10), with the flat profile and the depth bias −*L(z)* − 2, where *z* is the world
      height of the first end (S-0102 F2). The wave writes no depth. Because the profile is flat,
      each band pixel is compared as a point at the first end's height above the ground under that
      pixel: the band behaves as a level sheet at that height, hidden where a depth writer
      (terrain, overlays, trees, buildings) lies in front of it. Against a lifted vehicle the first
      end is high, so the sheet is high and is hidden less.
    - **View edge.** The band is not drawn within 3 pixels of the battlefield view's edges. The
      original used a 2-pixel margin in some frames instead. That is a drawing artefact; we use 3
      throughout.
    - **Fog.** With fog of war on, a wave is not drawn when both of its ends lie in cells that are
      explored but unseen for the local player (glossary *explored, seen*).
    - **Number domains.** The original computes the band's corners in floating point, with table
      trigonometry, and truncates them to whole leptons. That can leave a corner 1 lepton short of
      the exact value. Our engine uses the exact corners of F5, rounded toward zero, because the
      difference is not visible. The ripple values are integers derived once from table functions
      (F5). The colour arithmetic is exact integer arithmetic on the 16-bit picture (S-0103
      claim 9).
    - **Fixed artefacts.** The original's ripple tables cover only screen distances below 300 pixels
      on each axis and ring phases below 496; beyond them it reads unrelated memory. Our engine
      evaluates F5 for every distance. When the two ends coincide horizontally, the original's
      band direction is undefined; our engine then takes the band's length direction along the
      world *x* axis.
    - Backing: `measurable` (scenario L16) for the ends and their swap for a vehicle target, the
      freeze after the end, the band's width and length, the colour rule, the ring spacing, motion
      and jump, the shift direction, the absence of fade, the depth test, the view-edge margin and
      the fog rule. Exact corner rounding, the table-derived ripple values pixel by pixel and the
      fill rule on the band's edge pixels are `unmeasured`: a capture cannot separate them from
      the screen projection's own rounding. The fixed artefacts are `unmeasured` because they are
      design choices.
    - Confidence: high (the band's shape and width, the colour rule, the ripple formula, the
      absence of fade, the shift direction rule, the depth bias); medium (the swap of ends for a
      vehicle target and the first-weapon fire point, read from argument order rather than
      observed; the fog rule; which pixels on the band's boundary are filled)

26. **Where the magnetic wave falls in the draw order.** This is also presentation only.
    - A magnetic wave is an object of the air layer (S-0102 claim 3). It enters the air layer's
      list once, at its creation, as the list's last object (S-0102 claim 7). Moving its ends
      does not re-enter it, so it keeps its place in the list until it is removed.
    - Because the wave changes the picture already drawn (claim 25), it displaces and tints:
      - the terrain picture,
      - every object of the underground, surface and ground layers,
      - the air-layer objects that entered the list before it,
      - ion-cannon blast effects, which are drawn before the layers (S-0102 claim 2).
    - Everything drawn after it appears unchanged on top of the band:
      - air-layer objects that entered the list after it,
      - the top layer,
      - status extras,
      - the effects S-0102 claim 2 draws after the layers (light flashes, the laser, electric-bolt
        and radiation beams, range circles and map beacons).

      The magnetic wave is not one of those post-layer beam effects.
    - For example, while a seized vehicle is in the ground layer the band tints and displaces it.
      Once it enters the air layer after the wave was created, it is drawn over the band
      untouched. When that happens depends on S-0102 claim 4's re-evaluation moments.
    - Backing: `measurable` (scenario L16 part c)
    - Confidence: high (air layer, list order, the effect on earlier drawings only); medium (that
      moving the ends never re-enters the list; the seized-vehicle example, because of S-0102
      claim 4's open moments)

## Formulas

**F1 Size test.** The target can be seized only when

> *s* < *D*

where *s* is the target type's size (a decimal held in double precision) and *D* is the projectile
damage when it detonates (an integer, widened exactly). The comparison is exact and strict, so a
size equal to the damage is immune.

**F2 Falling damage.**

> *D*ꜰ = max(0, trunc(RZ₅₃(*B* × *m*)))

where:
- *m* is the falling-damage multiplier, held in single precision,
- *B* is the unit's current health when the current-strength switch is on, else its type's
  strength (both integers),
- the product is one arithmetic result under S-0205 claims 4 and 5,
- trunc is the common-form conversion (S-0205 claim 7).

A negative result becomes 0. With the engine defaults (*m* = 1.0, switch on), *D*ꜰ is the unit's
current health.

**F3 Wave distance.** With the horizontal integer lepton coordinates of the firing and impact
points of the shot (S-0313 claim 1 and S-0306 claim 1):

> *n* = (*x*ₜ − *x*ꜰ)² + (*y*ₜ − *y*ꜰ)²
> *d* = *r*(*n*)

The differences, squares and sum use signed 32-bit integer arithmetic, wrapping as S-0205
claim 1 states. For non-negative *n*, *r* is S-0204 claim 5's table square root, without integer
truncation of its result. For a sum that wraps negative, *r*(*n*) = *r*(−*n*): the root's input
is made positive first. No wave is created when *d* < 240 leptons. This distance includes no height or building-size term.

**F4 Wave fade.** The two dimensionless values *a* and *b* start at 0 and are held in double
precision. Let *q* = 13421773 / 268435456, the exact single-precision engine constant close to
0.05, widened exactly to double precision. Each addition and multiplication below uses RZ₅₃
(S-0205 claims 4 and 5); no single-precision narrowing occurs in these accumulators.

While the wave travels, each update with *a* < 1 gives *a* the value RZ₅₃(*a* + *q*). A result
above the double-precision engine threshold 8827055269646172 / 9007199254740992 (approximately
0.98) becomes exactly 1. Once the wave has ended, each update with
*b* ≤ RZ₅₃(*a* + RZ₅₃(*q* × 0.5)) gives *b* the value RZ₅₃(*b* + *q*). It is removed in that
update if the new *b* ≥ *a*. This also applies if the wave ends during fade-in: *a* stops growing
and *b* catches up to that retained value. There is one update at creation as well as the
subsequent per-tick updates.

A visual integer countdown starts at 100 ticks, decreases by 1 each tick and is reset to 64
when an update finds it at trunc(1 / *q*) = 19 ticks. It never ends a magnetic wave. It sets
the ripple phase of the picture (claim 25, F5); *a* and *b* have no effect on the picture.

**F5 Wave picture (claim 25).**

*Geometry.* Let the ends be *E*₁ = (*x*₁, *y*₁, *z*₁) and *E*₂ = (*x*₂, *y*₂, *z*₂), in leptons, with
*E*₂ already raised by 50 leptons (claim 25). Let *h* = √((*x*₂ − *x*₁)² + (*y*₂ − *y*₁)²). When
*h* > 0, let *u* = ((*x*₂ − *x*₁) / *h*, (*y*₂ − *y*₁) / *h*); when *h* = 0, let *u* = (1, 0). Let
*n* = (−*u*ᵧ, *u*ₓ). The four corners are

> *E*₁ − 50*n* and *E*₁ + 50*n*, both at height *z*₁,
> *E*₂ − 50*n* and *E*₂ + 50*n*, both at height *z*₂,

each component rounded toward zero to whole leptons. The band is the convex quadrilateral whose
vertices are the corners' screen points (S-0101 F1), in the order *E*₂ − 50*n*, *E*₂ + 50*n*,
*E*₁ + 50*n*, *E*₁ − 50*n*.

*Shift direction* (once, at creation). Take *E*₁ as the shot's firing point and *E*₂ as its impact
point raised by 50 leptons, as given to the wave at creation and before any update. Let *S* be the
screen point of *E*₁ and *P* the screen point of the corner *E*₂ + 50*n*. Let Δ*x* = *P*ₓ − *S*ₓ
and Δ*y* = *S*ᵧ − *P*ᵧ (positive when *P* is higher on the screen). If Δ*x* ≠ 0 and
−1 ≤ Δ*y* / Δ*x* < 1, the shift direction is right when Δ*x* > 0 and left when Δ*x* < 0. Otherwise
it is up when Δ*y* ≥ 1 and down when Δ*y* ≤ 0. The ratio is a floating-point division in the
original; only its comparison with ±1 matters, and that comparison is exact for integer operands.

*Ripple strength.* For a band pixel at screen point (*x*, *y*), with (*x*₀, *y*₀) the screen
point of *E*₁ and *c* the visual countdown of F4 at the time of drawing:

> *D* = trunc(*r*((*x* − *x*₀)² + (*y* − *y*₀)²)),
> *W*(*i*) = trunc(RZ₅₃(RZ₅₃(12 × sinₜ(RZ₅₃(*i* × 0.1))) + 0.49)),
> *j* = min(|*W*(*D* + *c*)|, 9),

where *r* is the table square root of S-0204 claim 5, sinₜ the table sine of S-0105 F1, 0.1 and
0.49 the double-precision values nearest to those decimals, RZ₅₃ the rounding of S-0205 claims 4 and
5, and trunc truncation toward zero. *W* takes the integer values −11 to 12. *D* and *W* are
integers; the original builds them once into tables.

*Displacement and boost.* The displacement is *s* = 0 pixels for *j* ≤ 1, 1 pixel for 2 ≤ *j* ≤ 6
and 2 pixels for 7 ≤ *j* ≤ 9, along the shift direction. The boost factor is *k* = 110 + 8*j*. The
source pixel is the pixel *s* pixels from (*x*, *y*) along the shift direction, with its colour as
before the wave drew. Widen its 16-bit colour to 8-bit components *R*, *G*, *B* (each with zeros in
its discarded low bits, S-0103 claim 9). The band pixel becomes

> *R*′ = min(255, *R* + ⌊(*k* / 2) × *R* / 256⌋),
> *G*′ = *G*,
> *B*′ = min(255, *B* + ⌊4*k* × *B* / 256⌋),

packed with S-0103 claim 9's 16-bit step. All of this is exact integer arithmetic; *k* is even,
so *k* / 2 is exact. For a source pixel with *B* = 0 or *R* = 0 that component stays 0: the wave
brightens blue and red where they exist, and adds no colour to pure black or pure green.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| arrival distance | 20 | leptons | S-0502 claim 12 (owned there) |
| drop height threshold | 1 | lepton above the surface | claim 13: non-positive height sets down, 1 or more drops |
| minimum wave distance | 240 | leptons, horizontal | F3 |
| wave fade step | 13421773 / 268435456 | per update, dimensionless | F4; single-precision constant, double-precision accumulator |
| wave full-strength threshold | 8827055269646172 / 9007199254740992 | dimensionless | F4; approximately 0.98 |
| wave fade-out margin | half the wave fade step | dimensionless | F4 |
| wave visual counter start, reset trigger, reset value | 100, 19, 64 | ticks | F4; no game effect, sets the ring phase (F5) |
| wave second-end lift | 50 | leptons, vertical | claim 25, F5 |
| wave band half-width | 50 | leptons, horizontal | claim 25, F5 |
| ripple phase step | 0.1 | radians per pixel of distance and per countdown step | F5; double precision |
| ripple amplitude, added constant | 12, 0.49 | dimensionless | F5; double precision |
| ripple strength cap | 9 | — | F5 |
| displacement by strength | 0 for 0–1, 1 for 2–6, 2 for 7–9 | pixels | F5 |
| boost factor base, step | 110, 8 | 1/256 units, per strength step | F5; red uses half the factor, blue four times it |
| wave view-edge margin | 3 | pixels | claim 25 |
| wave depth constant | −2 | depth units | claim 25; added to −*L(z)* |
| falling-damage multiplier absent-key default | 1.0 | — | F2 |
| current-strength switch absent-key default | on | — | F2 |
| magnetic-lift crash growth | 1.0 | leptons per tick, per tick | S-0502 claim 18 (owned there) |

## State and ordering

Life of a seized unit:

| State | Event | Next state | Effect |
|---|---|---|---|
| ordinary | eligible seizing hit (claims 1 to 5) | gripped (seized) | stops, guard, deselected and unselectable while seized, movement point fixed (claim 7) |
| gripped | arrives at its movement point (claim 8) | dropped if height ≥ 1 lepton, otherwise set down | release over that cell, normally beside the lifter |
| gripped | grip ends at height ≥ 1 (claim 12) | dropped | crashing mark, dropper recorded |
| gripped | grip ends at height ≤ 0, or no destination while it had one (claims 9, 12) | set down | destination none, except for a balloon-hover type (claim 13) |
| gripped, balloon-hover type holding a destination and a target | grip ends at height ≤ 0 and a row of S-0502 claim 28 holds (claim 13) | set down, movement commitment kept | the set-down's assignment is rejected after the grip has ended; destination, point and commitment kept; *attack* current |
| gripped | its approach step clears its destination for being too far from its target (claim 9) | gripped, no destination | grip holds, any stand-in; with the jumpjet stand-in its movement point and commitment are kept, so the pull goes on unless the seize's search found no cell (claims 7 and 9); a later rule giving no destination changes nothing |
| gripped, balloon-hover type holding a destination and a target | a rule gives it no destination and a row of S-0502 claim 28 holds (claim 9) | gripped | destination kept; *attack* current as that claim says |
| dropped | not over its cell centre | dropped | flies to the cell centre (claim 15) |
| dropped | fall ends, cell clear (claim 17) | descending, then set down at touchdown | survives |
| dropped | fall ends, blocked on water (claim 17) | sinking | death credited to the dropper |
| dropped | fall ends, blocked on land or beach | crashed, then removed | claim 18, unless crashable |
| set down, movement commitment kept (claim 13) | next eligible jumpjet update | set down, ascending | flies to its point and lands there (S-0502 claims 5 and 24); not restored meanwhile |
| set down, outside the hovering exclusion of claim 20 | end of each of its turns or a fall-back to rest, jumpjet not moving (claim 20) | ordinary | own locomotor returns; in the touchdown tick for a clear-cell touchdown |
| seized and set down, hovering with no point or commitment after no cell with the self-destruction switch off (claim 20, any route) | end of each of its turns (claim 20) | set down and moving if the repeated search finds a cell, otherwise unchanged | not restored; with a cell, flies there and lands |
| gripped with the teleporting stand-in | jump (claim 23) | set down, seized | grip ends; the jump clears the lifter's target |
| gripped with another ground stand-in | its own rules remove its destination (claim 23) | set down, seized | grip ends; the lifter keeps its target |
| gripped with the drop-pod stand-in | the seize, in our engine (touchdown in the original) (claim 23) | set down, seized, own locomotor back | grip ends; the lifter's target is cleared |
| seized with another stand-in | any restoration test (claim 24) | stays seized | the driving, walking, ship or teleporting stand-in is never restored; with the drop-pod stand-in the unit gets its own locomotor back at touchdown but stays seized |

**Ordering consequences.**
- One hit can drop the old gripped unit and seize the new one in the same tick (claim 5).
- Within one seizing hit, the firer's previous gripped unit is released even when the new target
  is immune. A resource gatherer made immune by claim 3 keeps its temporary locomotor. A
  resource gatherer that reaches claim 4's special case loses its spawned objects and releases
  its naval parasite in either outcome of that case.
- Claim 16's effects on what lies underneath happen before claim 17's sinking or landing.
- A unit that touches down on a clear cell is restored at the end of its turn in that tick,
  after the point at which it may process its behaviour, so a *hunt* processing in the
  touchdown turn still finds no target; the first search that can find one is its next *hunt*
  processing (claim 20).
- A neighbour destroyed by claim 18's first detonation is absent for the second; both
  detonations nevertheless use the damage computed from the falling unit before either.

## Edge cases and quirks

- **Erasing the rest of the cell** (`quirks.lift_drop_erases_cell_tail`, default on).
  - When the first other object under a dropped unit is an infantry unit, every object after it
    in the cell's list is also removed (claim 16), whatever its kind. That includes vehicles and,
    in principle, the dropped unit itself when it is listed after the infantry.
  - Fixed: only infantry units in the cell are removed, and the dropped unit never is.
- **Removing a survivor** (`quirks.lift_impact_removes_survivor`, default on).
  - Claim 18 removes the unit even when the falling damage did not kill it. That can happen
    with armour against the crushing and demolition warheads, a multiplier below 1, or curtain
    protection gained after the seize.
  - Fixed (design): a unit that survives both detonations stays in play. It is placed on the
    surface below its impact point, or on the deck if its fall ended crossing a deck, grounded
    and set down, and is then restored at the end of the turn in which its fall ended (claim 20).
- **Double impact credit** (`quirks.lift_impact_double_credit`, default on). A falling unit
  killed by either impact detonation is credited a second time by the impact credit (claim 18).
  Fixed: it is credited once.
- **A deck landing reads the ground layer** (`quirks.lift_deck_impact_reads_ground`, default on).
  - When a dropped unit's fall ends on a bridge deck, claim 16 judges the objects listed in the
    ground layer under the bridge, not those on the deck. Infantry under the bridge are erased
    and a vehicle under it blocks.
  - Fixed: claim 16 reads the layer the unit landed in.
- **A non-jumpjet stand-in never lets go** (`quirks.lift_standin_stays_seized`, default on).
  - A unit seized with the driving, walking, ship or teleporting stand-in is never restored, and
    one seized with the drop-pod stand-in gets its own locomotor back but stays seized (claim 24).
    Either way it ignores destinations and cannot fire until it leaves play.
  - Fixed: claim 20 applies to every stand-in with its own moving test, and restoration clears
    the seized state.
- **A stand-in that leaves its unit's cell unmarked** (`quirks.lift_standin_unmarked_cell`,
  default on).
  - A unit seized with the driving, walking or ship stand-in holds no vehicle-present mark until
    its stand-in's own movement sets one, and one that never moves stands unmarked for good
    (claim 23). Its listed presence still decides vehicle entry outcomes; rules that read only
    the occupancy record, such as the nearby-cell search, see its cell as free.
  - Fixed: it keeps the mark of its own cell at the seize, as a vehicle at rest does.
- **No cell for the stop when a lifter lets go** (claim 13, design). The original fails with a
  fatal error when S-0502 claim 17's search finds no cell for a unit whose lifter is letting go
  of it; our engine skips that claim's self-destruction there and clears the unit's movement
  point and commitment, lets a dropped unit fall with the direct ground reference and stops a
  set-down one where it is. No quirk switch: there is no original behaviour to keep. A grip
  ended by claim 9's no-destination exception is not covered: the original survives it, and
  S-0502 claim 17 applies unchanged.
- **No cell for a seized set-down unit, switch off** (claims 9 and 20, design). After
  S-0502 claim 17's switch-off outcome, every seized set-down unit hovering with no point or
  commitment retries the search at the end of each of its turns instead of being
  restored, whatever route brought it there. Height above the surface decides the outcome:
  1 lepton or more hovers; 0 or less stays where it is and becomes *grounded*, even from
  *ascending*, and is eligible for ordinary restoration. Claim 13's lifter-ended safety
  departure is unchanged.
- **Wave picture artefacts** (claim 25). The original's ripples read unrelated memory beyond
  its table range, its view-edge margin varies between 2 and 3 pixels, and its band direction
  is undefined when the ends coincide horizontally. These are drawing artefacts with no gameplay
  effect, so they get no switch; claim 25 states the fixed behaviour.
- **A harvester driving temporarily is immune** (claim 4). This is not a quirk: it is the only way
  a resource gatherer on a temporary locomotor is treated.
- **An infantry kill credited to the dropped unit** (claim 16) when there is neither a dropper nor a
  dropper house. This happens only if the dropper house no longer exists; the original behaviour
  is kept.

## Import mapping

Claims 9 and 20's switch-off design rules add no imported field or absent-key default.
No imported key changes how a magnetic wave looks or where it is drawn (claims 25 and 26): its
shape, colours, ripples and depth are engine constants. Only the magnetic-beam weapon flag below
decides that a shot makes such a wave.
Their search uses the type's locomotion class and movement zone through S-0502 claim 17.
S-0502 owns the corresponding import mapping and the shared self-destruction quirk.

| Source | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| rules, weapon | `IsMagBeam` | magnetic-beam weapon (claims 11, 21, 25, 26; S-0301 owns the term) | boolean | no |
| rules, warhead | `IsLocomotor` | seizing warhead (Terms, claim 1) | boolean | no |
| rules, warhead | `Locomotor` | the stand-in locomotor that the seize installs (Terms, claims 22 to 24) | locomotor identifier, read as S-0207 claim 14 (blanks trimmed; a malformed non-empty value is ignored, and warns on a seizing warhead; an empty value makes no entry, S-0001 claim 8, and reads as absent without a warning) | the locomotor in force from an earlier rules layer, else the teleporting locomotor (claim 22); an unknown identifier warns (claim 22) |
| rules, infantry or vehicle or aircraft type | `Size` | *s* (F1) | decimal | 1.0 (S-0507) |
| rules, object type | `Strength` | *B* when the current-strength switch is off (F2) | integer | as S-0302 |
| rules, type | the jumpjet type keys of S-0502 (Import mapping) | the lifted unit's jumpjet movement (claim 7) and its crash value, the starting crash rate (claim 15) | as S-0502 | as S-0502 |
| rules, `[CombatDamage]` | `FallingDamageMultiplier` | *m* (F2) | decimal, single precision | 1.0 |
| rules, `[CombatDamage]` | `CurrentStrengthDamage` | current-strength switch, F2 | boolean | yes |
| rules, `[CombatDamage]` | `CrushWarhead` | crushing warhead (claims 16, 18) | warhead name | as S-0303 |
| rules, `[CombatDamage]` | `C4Warhead` | demolition warhead (claims 18, 19) | warhead name | as S-0303 |
| rules, `[CombatDamage]` | `DeathWeapon` | game-wide death weapon, whose projectile claim 18 uses | weapon name | as S-0312; none configured: claim 18's design rule |
| rules, building type | `WeaponsFactory` | war-factory type (claim 3); does not select a docking point (claim 7) | boolean | no |
| rules, building type | `Helipad` | pad flag: a building lifter gives its docking point (claim 7; S-0404 owns the flag) | boolean | no |
| rules, building type | `UnitRepair` | repair-depot flag: a building lifter gives its docking point (claim 7; S-0314 owns the flag) | boolean | no |
| rules, building type | `Bunker` | vehicle-bunker flag: a building lifter gives its docking point, the quadrant point (claim 7; S-0606 owns the flag) | boolean | no |
| rules, building type | `Weeder` | obsolete-unloading flag: the fixed docking point, first in claim 7's precedence (S-0314 owns the flag) | boolean | no |
| rules, building type | `Refinery` | resource-unloading flag: second in claim 7's precedence; alone it gives no docking point | boolean | no |
| rules, building type; art, building type | `NumberOfDocks`; `DockingOffset0`, `DockingOffset1` and later numbered keys | docking-place count and displacements (claim 7) | as S-0314 and S-0404 | as S-0314 and S-0404 |
| rules, vehicle type | `ResourceGatherer` | resource-gathering vehicle (claims 4, 6, 20) | boolean | no |
| rules, country | `MultiplayPassive` | passive house (claim 6) | boolean | as S-0009 |
| rules, type | `Crashable` | crashable vehicle type (claim 18) | boolean | no |
| rules, type | `JumpJet` | jumpjet-type extra landing-clearance test (claim 17) | boolean | no |
| rules, type | `CrushSound` | crush sound (claim 16) | sound name | none: no sound |

## Measurement scenarios

Use a purpose-made small skirmish map with clear land, water, beach, a bridge and a tree cell,
with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), speed index 2 and 800×600 pixels. One human house owns the magnetic
lifter and, unless a test says otherwise, its victims: force-fire supplies friendly hits without
a second human player. One bystander computer player stays outside the observed test area.
Apply the glossary's bystander void rule. Named test map overrides below are removed afterwards.
They change data only, never the executable. Run each deterministic variant three times and
record the initial random state, rather than writing a fixed seed into the game.

Follow the glossary's measurement setup. Observe position, height, health, facing, current
and queued behaviour, destination, owner, kill counts, selection and passenger counts at each
tick. Record random-stream positions for L3's draw comparison. Do not change game state to
establish a qualifying condition; a run lacking the required observations is inconclusive.

- **L1 Eligibility (claims 3, 4).**
  - Fire at each of: an infantry unit, a building, a light and a heavy vehicle, a landed aircraft,
    an aircraft in flight, a vehicle under the iron curtain, a vehicle just leaving a war factory,
    and an already-seized vehicle.
  - Named test data: give one vehicle type a size just below and one equal to the weapon's damage.
  - Part f: a teleporting harvester while it drives back to its refinery.
  - Named test map L1 also supplies a seizing weapon able to hit each target class; otherwise a
    refusal to fire cannot test immunity. Keep its damage above the tested size except for the
    equal-size variant. Test a grounded and an airborne jumpjet vehicle separately.
  - Expected: each otherwise eligible ground-layer vehicle or aircraft lifts; infantry,
    buildings, air-layer units, invulnerable units, already-seized units, equal-size units and
    factory-protected vehicles do not. Merely being heavy is not immunity.
  - Record the harvester's movement, queued order and restoration: a replacement able to end
    is restored without a lift; otherwise it stops and queues stop without a lift. For the
    factory case, the observations must confirm contact and foundation coverage at impact; without
    that state this part is void, not a confirmation.
  - Falsified by: any immune target lifting, an eligible one not lifting, or a harvester outcome
    contrary to its confirmed temporary-movement state. Part f's two states are conditional if
    ordinary driving never supplies them; do not manufacture either by writing game state.
- **L2 Pull and arrival drop (claims 1, 5, 7, 8, 16, 17, 18).**
  - Seize a vehicle at long range and let the lifter stand still.
  - Record:
    - the target's health at the hit, which should not change,
    - its body facing during the pull, which should stay constant,
    - the movement point fixed at the seize (its cell) and the tick of arrival there,
    - the drop over that cell: the fall, and either a harmless descent and touchdown (clear
      cell) or both detonations and removal (blocked cell),
    - the health of the lifter and of the dropped unit,
    - whose kill count rises.
  - Also record the stopped route, forced guard, cleared target, initial destination and
    deselection. Put a separate undamaged unit nearby to detect area damage at the seizing hit.
    For cleanup, use a spawned-aircraft carrier and, separately, a parasitised vessel.
    Expect spawned objects lost and the naval parasite released at seizure. If a contacted
    factory-protected target is immune, it cannot test post-seize contact breaking; record
    contact breaking only when a different reachable building contact is confirmed at seizure.
  - Expected: no damage from the seizing hit, unchanged body facing, a path to a free cell
    next to the lifter's cell, a drop at arrival within the strict 20-lepton threshold of that
    cell's centre, and, with that cell clear, a harmless descent and touchdown beside the lifter.
    A named L2 map may vary the victim's jumpjet speed or climb field one at a time, its trajectory must follow its own fields, not the lifter's.
  - Falsified by: any missing seizure effect above, damage at the hit, a turning body, trajectory
    following the lifter's fields, a grip surviving arrival, an arrival over the lifter's own
    cell while a free neighbour cell existed, damage after a clear-cell drop, or survival after
    a blocked impact.
- **L3 Release by order over clear land (claims 9, 12, 13, 17, 20).** This is the first scenario
  to run, because it decides whether a release is harmless.
  - Run a: mid-pull, at full cruise height over clear land, order the lifter to move one cell.
    - Expected: the unit is dropped; it flies to its cell centre, falls with a growing rate,
      finds the cell clear, descends, touches down undamaged, is not crashing, and is restored
      at the end of the touchdown tick: a move order in the next tick drives it normally.
    - Record the ticks from the order to touchdown and the health at touchdown.
    - Falsified by: damage or removal on clear land.
  - Run b (conditional; S-0502's J27 a): give the order in the tick of the seize, at height 0,
    varying the order tick around the seize over 20 recorded seeds, with the surrounding cells
    free. Record the movement point, movement commitment, flight state and height per update,
    and the restoration tick.
    - Qualify only a run whose observations confirm height 0, *grounded* and a movement
      commitment at release, before the unit's first jumpjet update. If no run qualifies,
      the test is inconclusive, not falsified.
    - Expected: set down with no fall. The point and commitment are kept; the next eligible
      jumpjet update starts its ascent, the unit flies to the point and lands there, and
      it cannot be restored before the commitment has ended. Expect restoration at the end of
      the tick in which its landing ends the commitment, with no order given.
    - Falsified by: a fall or damage; staying grounded through that eligible update; losing the
      commitment despite a found point; restoration while committed; failure to restore by
      the end of the landing tick.
  - Run a also falsifies claim 15 if a non-centred release falls vertically without the
    cell-centre detour.
  - Run c: use a transport with passengers. After a clear-land release, verify all passengers
    remain. Compare its random-stream trace with an empty transport under the same recorded
    conditions: no crash-start tumbling draws should occur. Passenger loss at drop or those
    extra draws falsifies claim 13.
  - Run e (claim 13's found point): release mid-pull, off the cell centre, over a cell that has
    a vehicle in one neighbour and clear cells around; separately let a pull end by arrival.
    Record the per-tick horizontal position from the release to the impact or touchdown.
    - Expected: the unit moves only towards its own cell centre (as in run a), never towards
      another cell, and falls over that cell.
    - Falsified by: any horizontal movement towards a neighbouring cell after the release.
    - Do not set up a release with no cell for the unit's own movement within the search
      reach of S-0212 claim 25 (for example a land vehicle far out over open water): the
      original stops with a fatal error there.
  - Run f (conditional; S-0502's J27 b): let a pull end by arrival, after placing a parked
    ground vehicle in the unit's arrival cell after the seize's initial point search, varying
    the placement tick so the stop's found point lies in another cell. Qualify only a run with
    that kept point, zero crash speed and another object listed in the unit's own cell. Record
    the height at every update from release to impact.
    - Expected: each height step follows S-0502 claim 9's look-ahead with own-cell clearance,
      then claim 18's extra crash drop; moving a neighbouring occupant does not change it.
    - Falsified by: steps that follow bare terrain where the predictions differ, a reference
      taken from a neighbour at zero speed, or any other height mismatch. If no run qualifies,
      the test is inconclusive.
  - Run d: order stop on the lifter while gripping, with no override or wait behaviour. Record
    its return to rest and the victim's release. A retained grip after that return falsifies
    claim 12's rest row. Repeat with wait current or an override; that rest event alone must
    not release the victim.
- **L4 Blocked landing (claims 15, 16, 18, 19; crash rate, credit, passengers).**
  - Release over a cell holding another vehicle. Record the cell-centre detour, the crash rate per
    tick, both detonations, the health of the vehicle underneath, and the dropped unit's removal.
  - Crash rate: record the height per tick of the fall. Each tick's fall is S-0502's ordinary
    height step toward target height 0, using claim 13's clearance reference when applicable,
    plus the extra crash drop. Expect the extra drop (the fall
    beyond the ordinary climb-rate descent) to equal the lifted type's crash value in the first
    crashing update and to grow by 1 lepton per update after that, so that while the ordinary
    descent is steady, successive per-tick falls differ by exactly 1, with heights truncated.
    Seize and drop the same unit twice (it must survive the first drop over a clear cell, then be
    seized again): the second fall must again start from the type's crash value. A named L4 map
    may give the lifted type a non-integral crash value to expose the truncation. A seized
    crashing unit must never aim below the surface (target height 0, not −5).
  - Double credit: give the dropper a kill counter and the lifted unit low health and no armour
    against the impact warheads, so the first detonation kills it. Expect the dropper's kill
    tally and its house's tallies and score to rise by two for that one unit, and its
    experience to grow by twice the unit's value. A survivor removed by
    `quirks.lift_impact_removes_survivor` (variant (a)) must add one.
  - Passengers: drop a loaded transport onto a blocked cell. Expect its passengers to die
    credited to the attacker when a detonation kills it, and without attacker credit when it
    survives and is removed.
  - Named test-data variants:
    - (a) a multiplier of 0.5,
    - (b) the current-strength switch off,
    - (c) a crashable lifted type, which should give the ordinary crash end,
    - (d) a lifted landed aircraft, using a test weapon whose damage exceeds its size.
  - Name the overrides L4 impact map. Give test bystanders enough health and simple armour
    factors to distinguish damage from the two impact warheads without killing them. Compare
    every health delta with F2 and the ordinary area-damage rules. In (b), injure the victim
    before seizure so maximum and current health predict different amounts. Add a negative
    multiplier and an absent-key variant, and a non-integral product to distinguish truncation
    from rounding to nearest. Both detonations must use the amount computed before either.
  - In (d), give the two impact warheads distinct data-only damage modifiers. The aircraft must
    use the demolition modifier in both detonations, unlike a vehicle. Repeat with its
    crashable-type flag toggled: that flag must not replace the aircraft's magnetic impact.
  - Expected under (a): removal even when both defended detonations leave the victim alive.
    Under (b), the uninjured type strength supplies the base; negative damage becomes 0.
    Variant (c) follows the ordinary vehicle crash end, not the two magnetic detonations.
  - Falsified by: a wrong F2 delta, a crash rate that does not start at the type's value or
    grows before the first drop, a single credit for a detonation kill, recomputing damage after
    the first detonation, survivor retention, a crashable vehicle using magnetic damage, or an aircraft using crushing damage
    in its first detonation. Repeat a release above a bridge: stopping the fall anywhere other
    than the downward deck crossing or surface contact falsifies claim 15.
- **L5 Dropping on infantry and objects (claim 16).**
  - Variants: release over a cell holding infantry; infantry plus a vehicle listed after them; a
    tree; a landed aircraft.
  - Record which objects vanish without death animations, the kill counts, and whether the
    dropped unit survives (clear cell after erasure).
  - Expected: infantry erased without death effects, with each type's crush sound if present,
    and kills credited to the dropper (else its house). A tree or landed aircraft receives its
    own health as defence-ignoring crushing damage; subsequent landing depends on occupancy.
    A vehicle or building underneath takes no direct crushing damage from claim 16, though
    the later impact detonations may hurt it. Also drop onto a vessel: expect a blocked crash,
    not sinking.
  - Use named L5 map data with conspicuous death effects to distinguish erasure from ordinary
    damage, and distinct crush sounds. Repeat over a bridge with infantry and then a vehicle
    below the deck; their ground-layer presence, not deck occupants, decides claim 16.
  - The infantry-plus-vehicle-tail test is conditional: run it only if ordinary placement or
    movement establishes that list order without writing state. Otherwise its reachability
    remains unmeasured. Reverse entry order wherever legal to test the first-occupant rule.
  - Falsified by: infantry death effects, missing credited kills or configured crush sounds,
    direct damage to a first vehicle/building, the deck layer deciding instead of ground, or
    sinking after landing on a vessel. A later non-infantry occupant surviving the confirmed
    infantry-first list falsifies the cell-tail rule.
- **L6 Water, beach, bridge (claim 17).**
  - Release over deep water, over beach, and over a bridge deck above water.
  - Expected for an ordinary land vehicle:
    - blocked water: sinking and stun, no magnetic impact detonations, death credit to dropper
      or its retained house,
    - beach: clear occupancy gives harmless descent; blocked occupancy gives crashed impact,
    - clear deck: descent and touchdown on the deck, not sinking.
  - Repeat with an amphibious vehicle on water that it can occupy: expect harmless descent,
    not sinking merely because the land type is water. Named L6 map data gives a vehicle
    amphibious movement and toggles its jumpjet-type flag: the extra wheeled-clearance test
    may turn an otherwise clear water landing into sinking. Confirm both clearance results
    from observed occupancy and terrain; a clear result despite failed extra clearance falsifies claim 17. Repeat with a slave-bearing vehicle
    and record freed slaves and ownership under S-0312 claim 30's controller-death rule.
  - Falsified by: sinking on a clear legal water landing or on a deck, magnetic falling damage
    on the sinking branch, beach always causing sinking, or an ownership/credit result that
    differs from these rules.
- **L7 Lifter killed mid-lift (claims 12, 14).**
  - Destroy the lifter by friendly force-fire while it grips a unit over clear land and,
    separately, over infantry. Repeat by destroying the gripped unit itself. For the latter,
    observe that the surviving lifter loses its target. Separately destroy the dropper only
    after an ordinary move-order drop but before impact.
  - Expected: lifter removal drops its airborne victim; gripped-unit removal ends the link and
    clears the lifter's target. A surviving dropped unit retains the former dropper house for
    impact credit after the dropper disappears.
  - Falsified by: a grip surviving either participant's removal, no fall after lifter removal,
    a retained lifter target after victim removal, or impact kills assigned to the killer of
    the lifter rather than the former dropper house.
- **L8 Building lifter (claim 12).**
  - Named test data: a building with a magnetic-beam weapon and a seizing warhead. Seize a
    vehicle, then cut power (S-0405) and, separately, destroy the building.
  - Expected: a drop and cleared building target in both runs; record release in the same tick
    as the loss of operation/removal. A retained grip or target, or release only when the wave
    subsequently disappears, falsifies claim 12.
- **L9 Refusals and the building attack (claims 2, 9, 10, 11).**
  - Part a is conditional: give the friendly victim move or stop orders, one click after
    another, while the seizing projectile flies, so that one may be given in the seize's own
    tick (claim 9). Record whether an order actually reaches destination handling after the
    seize. If one does, a non-empty destination must be ignored; clearing it must unlink
    without a crash drop and allow ordinary descent. Horizontal movement from its own move
    order, or a crash drop on destination clearing, falsifies claim 9. Deselection blocking the
    order is not confirmation of destination handling.
  - Part b: use an armed friendly victim with enemies in range and record shots from seizure
    through fall, touchdown and restoration. No shot is allowed while seized; any such shot
    falsifies claim 10. Absence of a target is not a firing-refusal test.
  - Part c: while gripping, force-fire at a second vehicle.
    - Expected: rearming until the wave's fade-out ends.
    - After that: a seize of the second vehicle and a drop of the first in the same tick.
    - Falsified by: firing at the second vehicle while that gate applies, retaining the old
      grip after the new seize, or release and seizure occurring in different ticks.
  - Part d: while gripping, attack an enemy building (or force-fire at a friendly building).
    - Expected: the secondary weapon fires, the wave fades, and the gripped unit keeps being
      pulled in. Once other firing gates permit it, a new magnetic shot while an old wave is
      alive creates no second wave. A named L9 map may shorten the rearm delay to expose this.
    - Falsified by: a building refused solely by the grip/wave gate, the grip ending merely on
      retargeting, a second wave, or non-building fire before the live-wave grip gate ends.
  - Part f: while gripping, keep the lifter's target on its gripped unit with a non-magnetic
    weapon too (named L9 map data gives the lifter such a secondary weapon). Keep the gripped
    unit's cell reachable for the lifter, or the unit within range, on every tick of the part,
    and record both on every tick. Expect no shot at the gripped unit from any weapon, and the
    lifter's target to stay on it until arrival or another event of claim 12. A shot at the
    gripped unit falsifies claim 11, and so does a dropped target without such an event on a
    tick on which the cell was reachable or the unit in range. A drop on a tick on which the
    cell could not be reached and the unit was out of range is claim 11's per-tick drop, and
    makes the part void.
  - Part g: seize a deployable vehicle and, separately, a loaded transport. A clear-cell
    touchdown restores the unit in the same tick (claim 20), so try deploy, undeploy and
    unload through the harness while it is gripped at height 0, or while it is set down and
    still committed (L3 run b's route). Whether any order reaches a seized unit gripped at
    height 0 or set down is open (Open questions; claim 9's order given in the seize's own tick
    is not one of these), so this part is conditional. When an order reaches the unit, record
    whether each starts. Expect the ordinary result for a grounded unit; a refusal explained only by
    the seized state falsifies claim 10's last bullet. When no order reaches the unit, the part is
    inconclusive and confirms nothing.
  - Part e: named L9 map data supplies a slow seizing projectile. Launch it at an immune
    target while a grip exists, after any wave refusal has ended; expect release of the old
    victim without seizure of the immune one. Repeat with the firer destroyed, seized, or
    still gripping that very target when its delayed projectile arrives: expect no new seize
    and no release from that hit. Only use states reached through ordinary play. A contrary
    release/seize with the recorded guards falsifies claim 2; an unreachable variant remains
    unmeasured rather than confirmed.
- **L10 Wave lifetime (claim 21, F3).**
  - Named L10 map: remove the weapon's minimum range. Record actual firing/impact coordinates,
    not unit-centre distances. Test point separations below, exactly at and above 240 leptons
    according to F3, including a diagonal separation at which the table and exact roots differ.
    Expect seizure even without a wave at a sub-threshold separation.
  - Let a wave reach full strength, retarget and count its remaining updates: expect 20 fade
    updates by F4. Repeat retargeting after several different fade-in lengths, including the
    first available tick; F4 predicts the shorter lifetime from the retained strength.
    Separately remove target and firer to test both ending conditions. With stationary
    participants and unchanged targeting, observe well past 100 ticks: no fixed expiry.
  - Update turn: retarget the firer and record the tick in which the wave begins fading; expect
    the same tick. Separately, let the wave's target enter a transport or a bunker; expect the
    wave to begin fading in the next update, while the target is unharmed. A named L10 map may
    place the firer and target so that the squared horizontal distance wraps negative; expect
    creation exactly as F3's mirrored root predicts, if such a separation is reachable.
  - For no-damage testing, keep a non-seizing magnetic shot's ordinary impact outside nearby
    test units and record their health along the wave path: the wave alone must not hurt them.
  - Falsified by: wave creation below F3's threshold, missing creation at/above it with a free
    slot, failure to seize solely because no wave exists, a lifetime inconsistent with F4,
    fixed expiry with unchanged targeting, or health loss attributable only to the wave.
- **L11 Restoration and computer houses (claims 6, 20; parts c to f for claim 20's later
  bullets).**
  - Part a: after L3, record the tick the unit first accepts a move order. Expect the first
    tick after touchdown.
  - Part b: repeat L3 against an isolated unit of the computer opponent, excluding resource
    gatherers and passive countries. Record its queued hunt at seizure, rather than inferring
    it from later AI movement. A second run uses a resource gatherer and records its harvest
    queue at restoration; a human-owned control must not receive either computer-only queue.
    A passive-country control must not receive the seizure hunt.
  - Part c: a human-owned unit in *guard* after a clear-cell touchdown. Record per tick whether
    it is seized. Expect restoration at the end of the touchdown tick. One move order given in
    the next tick moves it with its own locomotor.
  - Part d: seize a vehicle while it is mid-turn or mid-step and give it an order that waits for
    readiness; expect readiness to follow the jumpjet's *moving now* state, not its earlier
    driving state.
  - Part e (computer hunter after a drop; extends part b). Use the common measurement setup.
    A magnetic lifter of the human house seizes an isolated armed vehicle of the computer
    opponent (not a gatherer, not passive). A second lifter of the human house stands by,
    idle, outside the vehicle's weapon range. Order the first lifter to move one cell
    mid-pull, so the vehicle is dropped over clear land. That move order clears the first
    lifter's target (claim 12). To keep it from taking the vehicle as a new target, order it
    on in the tick after the drop to a cell beyond its weapon's range of the vehicle and
    outside the vehicle's sight. Leave the vehicle alone for 600 ticks. Record per tick its
    seized and set-down state, current and queued behaviour, target, position, and every fire
    event. A run in which the first lifter fires at the vehicle again is void. In every run of
    this part, a vehicle that holds an attack-move memory makes the run void, since that memory
    can give it a target before restoration (claim 9).
    - Run 1, second lifter withdrawn as the first, no armed object of the human house in the
      vehicle's reach, and an unarmed vehicle of the human house placed 10 cells from the
      vehicle's touchdown cell as the human's nearest object: record the full 600 ticks, and
      the first tick at which the vehicle takes a target, moves or fires, with that target and
      its behaviour. Expected: no target before restoration; its first target at its first
      *hunt* processing after the touchdown tick (an object that its hunt search reaches,
      normally the unarmed vehicle), with *hunt* current, then an ordinary approach and fire.
    - Run 1b: as run 1, but the second lifter stays and at touchdown + k ticks (k = 1, 5 and
      30, one run each) fires at the vehicle. A seize can succeed only on a restored unit
      (claim 3). Expected: the seize succeeds for every k. After that second seize claims 5
      and 6 apply again.
    - Run 2, an armed enemy vehicle placed within the dropped vehicle's weapon range of its
      touchdown cell, with the second lifter withdrawn: record the first tick it takes a
      target, fires or moves, the target, and its behaviour at that tick. Expected: no fire
      during the grip, the fall or the touchdown tick; its first target at its first *hunt*
      processing after the touchdown tick, chosen by the ordinary threat rules (S-0309), and
      its first shot under the ordinary firing rules (S-0301), with *hunt* current. Record
      whether the enemy destroys it first; such a run is void for the first-shot tick.
    - Control: a human-owned vehicle of the same type, dropped beside it in *guard* (part c).
      Expect restoration in its touchdown tick too, and in run 2 the ordinary *guard* response.
    - Falsified by, outside a run made void by an attack-move memory: a vehicle still seized
      after its touchdown tick, any shot or target before restoration, a different restoration tick
      for the hunter and the guard control, or a behaviour other than *hunt* at the first shot.
  - Part f (units being erased; conditional): drop a vehicle over clear land and have an
    eraser lock onto it at touchdown, then remove the eraser before the erasure completes.
    Record per tick its seized state and its being-erased state. Expected: still seized while
    it is being erased, restored at the end of its first turn after it stops being erased.
    Falsified by restoration while it is being erased. Inconclusive if no lock starts before
    the end of the touchdown tick.
  - Expected: no restoration while gripped, no accepted own movement before restoration,
    restoration at the end of the touchdown tick (or the end of a still-committed movement)
    whatever the behaviour and house, and the queues above.
    Falsified by earlier restoration, a later restoration of a resting set-down unit, or a
    queue contrary to the owner/type conditions. Keep incidental AI orders out of the observed
    interval, or mark the run void.
- **L12 Seizing from a moving lifter (claims 7, 8).**
  - Let the lifter drive across clear land and fire at a vehicle as it goes (named L12 map data
    may give a mobile lifter a moving-fire ability or a short rearm delay); separately, use a
    building lifter.
  - Record the lifter's heading point at the seize, the gripped unit's movement point and its
    arrival cell, and the lifter's position at arrival.
  - Expected: the movement point is a free cell beside the lifter's heading point at the seize
    (the foundation centre or docking point for a building) and does not follow the lifter
    afterwards; the unit drops over that cell and lands unharmed when it is clear.
  - Falsified by: a movement point that follows the moving lifter, an arrival over the lifter's
    current position after it moved, or selection of an occupied movement-point cell when
    a qualifying free neighbour existed at the seize. Occupancy gained after the point was
    fixed does not falsify the claim.
  - Docking variant (claim 7). Named L12 map data gives a seizing magnetic-beam weapon to a
    repair-depot type with one docking place, to the same type with several docking places, to
    a vehicle-bunker type and to a refinery type. Seize a vehicle from several sides with each,
    and record the cell of the fixed movement point against the building's cells and its docking
    places.
    - Expected, in each case the cell that claim 7's search returns around the named point's
      cell (that cell itself when it qualifies): for the depot with one place, around that
      place's point; for the one with several places and a vehicle not in contact with it,
      around the foundation centre; for the bunker, around the quadrant point on the vehicle's
      side, changing with that side; for the refinery, around its foundation centre, the same
      from every side.
    - Falsified by: a movement point that follows a docking place for the refinery, or that
      ignores the side for the bunker.
    - Extend the named L12 map with a pad type and a depot with no docking places, and with
      depot types combining the obsolete-unloading, resource-unloading and bunker flags.
      With multiple places, also attempt a seize while the victim has a confirmed docking
      contact with its lifter. Compare the fixed point's cell with claim 7's precedence,
      including its building-height and docking-height inputs before cell-centre replacement.
      Qualify only layouts where the candidate points predict different searched cells.
      Expected: the selected cell follows claim 7; a different cell falsifies it. A docking
      contact that cannot be reached without factory protection is inconclusive, not backing.
- **L13 Other locomotors (claims 12, 22 to 24).**
  - Part a. Named L13 map data: seizing warheads naming hovering, mechanical walking, flying,
    rocket, tunnelling, driving, walking, ship and teleporting movement, one with the locomotor key absent, one with
    an empty value and one with a malformed value (a bare name, and separately digits without
    braces), each fired at an eligible vehicle.
    - Expected: with hovering, mechanical walking, flying, rocket and tunnelling, the target stops, enters *guard*,
      loses spawned objects, is not lifted and keeps its own movement, and can be hit again with
      the same result; with driving, walking, ship and teleporting, and with the absent, empty and
      malformed values (teleporting), it is seized: it ignores orders and cannot fire. The firer's previous
      victim is dropped in all of them.
    - Falsified by: a seize with hovering, mechanical walking, flying, rocket or tunnelling, a missing stop, or no
      seize with a stand-in locomotor or with the absent, empty or malformed value.
  - Part b. With the driving and the teleporting warheads, seize a vehicle at long range from a
    lifter that stands still, and separately from a building lifter. Record per tick the victim's
    order-level destination, its path and position, its height, the tick of a jump, and the
    lifter's target, its shots and its refusal state.
    - Expected with driving: the victim drives towards the lifter under S-0207's rules, is never
      lifted, gets no arrival drop near the lifter, and its grip lasts until an event of claim 12.
    - Expected with teleporting: one jump to a landing point near the lifter, on the movement
      update after the seize; the grip ends in the jump tick with the victim set down there, the
      lifter's target is cleared in that tick and its wave begins fading at its next update; no
      further jump follows.
    - Depot variant (conditional): named L13 map data gives a seizing magnetic-beam weapon to a
      repair-depot type with one docking place. Qualify only a run in which the driving victim
      comes to rest in *guard* exactly on the docking point in all three coordinates. Expected:
      the grip ends in that tick, the victim is set down and the lifter keeps its target. Any
      later shot by the lifter at the victim seizes nothing and does no damage. If no run
      qualifies, this variant is inconclusive.
    - Falsified by: a lift, a drop or impact damage; a driving grip ended by arrival in a cell
      alone; a teleporting grip that survives the jump, or a second jump; a lifter target kept
      after the jump; or, in a qualifying depot run, a grip that survives the exact stop.
  - Part c. After part b's grip has ended, leave the victim alone until its wave has ended
    (claim 21) and, with the drop-pod warhead, it has touched down. Then try to give it two move
    orders at different ticks, and let enemies come into its range. A seized unit cannot be
    selected (claim 5), so record for each attempt whether an order reaches the victim's
    destination handling. Repeat with walking, ship and drop-pod warheads; use navigable water
    for the ship variant.
    - Expected: with driving, walking, ship and teleporting, no order that reaches it is
      accepted, it never fires and a second seizing hit does not seize it again; with drop-pod,
      the original's descent near the lifter is recorded (its path and timing have no expected
      result here), its own locomotor returns at touchdown, and it still ignores any order that
      reaches it and never fires. When no order reaches the victim, as when the attempts find
      it unselectable, the order part is inconclusive and confirms nothing; the firing and
      re-seizing parts still stand.
    - Drop-pod grip: record per tick the lifter's target, its shots and refusal state and its
      wave from the seize until well after touchdown. Expected: during the descent the lifter
      keeps the victim as its target, does not fire at it, and its wave stays alive; in the
      touchdown tick the grip ends and the lifter's target is cleared; the wave begins fading at
      its next update. Give the victim two attackers, one of its own house and one of another
      house, both targeting it during the descent, and record whether each still has it as its
      target after touchdown. Expected in the original: the other-house attacker drops it, the
      same-house attacker keeps it (our engine keeps both by design). Record visibility as
      well; the other-house result is diagnostic until the visibility condition in claim 23
      is established, not a confirmation from house ownership alone.
    - Falsified by: an accepted order or a shot after the grip ends, a restoration that
      clears the seized state, a grip or lifter target that ends before touchdown, or a lifter
      target kept after touchdown. A same-house attacker losing its target solely at
      touchdown falsifies the same-house retention rule.
  - Part d (claim 4 with other locomotors). With the hovering, the driving and an
    unknown-identifier warhead of part a, fire at a teleporting harvester while it drives back to
    its refinery (L1 part f's state, confirmed by the observations). The unknown-identifier warhead
    must reach no other eligible target: the original stops with a fatal error on one.
    - Expected: L1 part f's outcome with each warhead, and no fatal error with the unknown
      identifier.
    - Falsified by: a stop with *guard* and no restoration (claim 22's outcome), or a seize.
  - Part e (claim 23, landed aircraft). Seize a landed aircraft with the teleporting warhead from
    a lifter that stands still. Record its landing point and height, the grip's end, its owner's
    later move orders and whether it ever takes off. Expected: one jump to a landing point at
    terrain height, normally in the cell of the lifter's point, the grip ending at the jump, and
    no take-off afterwards. Repeat with the driving warhead and record what the original does
    (no expected result: claim 23 states a design rule there). Falsified by: a flight, a grip
    that survives the jump, or a later take-off.
  - Part f (claim 23, occupancy). Seize a friendly stationary vehicle with the driving warhead
    from a lifter far away, before the victim's first step. In the following ticks, order a
    second friendly vehicle into the victim's cell; repeat after the grip has ended. Then, with a
    second lifter whose cell has the unmoved seized vehicle as its only free-looking neighbour
    (all other neighbours occupied or impassable), seize a third vehicle with the jumpjet
    warhead and record the cell of its movement point and what happens at its arrival drop.
    Repeat with a victim that has driven some cells and come to rest, and with the teleporting
    warhead for the cells around its landing point before the jump.
    - Expected: the second vehicle waits as before a moving blocker while the victim has its
      destination, and stops as before a stationary friendly after the grip has ended; the
      jumpjet victim's movement point can be the unmoved seized vehicle's cell, and its drop
      there meets a vehicle (claim 16). A seized vehicle that has driven and come to rest, and
      the teleporting landing cell, are never chosen.
    - Falsified by: an unmoved seized vehicle's cell refused by the movement-point search when
      it was the only candidate, a cell the victim has driven into chosen, or the second
      vehicle entering the seized vehicle's cell.
- **L14 No-cell switch-off design checks (claims 9 and 20; our engine only).**
  - Use a small skirmish test map at speed index 2, a lifter and a seized vehicle, with the
    self-destruction quirk off. These are design checks, not oracle measurements.
  - Compare a no-destination set-down with a set-down unit that kept its commitment, flew
    to its point and then reached a no-cell stop after a failed landing. Qualify the latter
    only if that route actually reaches the stated outcome; the seized landing exceptions
    can prevent it. In both runs record per tick seized/set-down state, height, flight state,
    movement point, commitment and each search.
  - Expected at height 1 lepton or more with no point or commitment: no restoration, and
    another search with no draw at the end of each of its turns. With no cell still
    available it keeps hovering; after a qualifying cell becomes available it flies there
    and lands, and is restored at the end of its landing tick. The route into the outcome
    makes no difference.
  - Separately stop at height 0 while *ascending*, and at height 0 while *grounded*, with no
    cell. Both stay put, become *grounded*, clear point and commitment, and are restored at the
    end of its turn in that tick or the next. Compare height 1: it must hover instead.
  - A restoration while hovering, a route-dependent retry, or hovering at height 0 fails the
    design check. A run that never reaches the required state is inconclusive.
- **L15 Registration and combat height (claim 7).**
  - Use the common measurement setup above, with a magnetic lifter seizing a ground vehicle
    over clear land. Record its first jumpjet update, flight state, signed integer height
    above the surface, grip, registration and touchdown each tick with a read-only recorder.
    Propose recorder support to the user if absent; do not write game state.
  - Repeat with a mid-pull release over a parked vehicle to obtain a blocked crash. Observe
    registration immediately before the impact effects, not only after removal. In both runs,
    record the combat in-the-air answer during ascent and descent or crashing, including
    observations below and at or above 208 leptons. If the trajectory skips the exact
    boundary, report that limitation rather than claiming an exact-boundary measurement.
    Named L15 map overrides may vary the victim's jumpjet climb field to obtain observations
    at 207 and 208 leptons; only a trace reaching both heights tests the exact boundary.
  - In a separate run, destroy the seized unit during ascent by force-fire. Record its
    departure from the map and registration ending; registration must not survive that event.
    A retained registration after departure falsifies the map-exit boundary.
  - Expected: registration starts in the take-off update, before the first rise. Release
    leaves it registered. The clear-cell run stays registered until touchdown; the blocked
    run unregisters before its impact effects. In every observed state the combat answer is
    true exactly at heights of at least 208 leptons, not merely whenever registered.
  - Falsified by: registration starting only at a positive height, release unregistering it,
    loss of registration during clear-cell descent before touchdown, registration surviving
    into blocked-impact effects, or a combat answer inconsistent with the height comparison.
    Run each variant three times. A missing read-only observation is inconclusive.
- **L16 Wave picture and draw order (claims 25, 26, F5).** Presentation: capture the picture
  rather than game state.
  - Setup: the common setup above, at the game's own resolution, unscaled and unfiltered, with the
    display path recorded as S-0103's palette scenarios require. Use a flat stretch of clear land
    whose terrain colour is uniform over at least 40 × 40 pixels. Capture one frame per tick from
    the shot until the wave is gone, with the view still.
  - (a) Vehicle target. A lifter seizes a parked vehicle whose screen line from the lifter is
    nearly horizontal; repeat with a nearly vertical line. Expected: a band 100 leptons wide under
    S-0101 F1, with no outline and no colour of its own. Its rings are centred on the vehicle, about
    31 pixels apart, and move outward 1 pixel per tick. The pattern 82 ticks after the shot (±1)
    matches the one 38 ticks after it, and it repeats every 45 ticks from then on. Inside the band the terrain is displaced by 0, 1
    or 2 pixels: sideways in the horizontal run and up or down in the vertical one. On terrain of
    known colour, the band pixels match F5 for some strength from 0 to 9, with green unchanged.
    While the vehicle is pulled and lifted, the band follows it tick by tick.
  - (b) End and freeze. Retarget the lifter mid-grip and keep capturing. Expected: the band stops
    following, keeps its last place and full strength while the rings keep moving, then vanishes
    in one frame, the tick F4's fade-out ends (claim 21). It never fades gradually.
  - (c) Order and depth. Repeat (a) with a building between the view and the band's far part,
    with a laser-armed defence firing across the band, and through the victim's lift past the
    air-layer height of S-0102 claim 3. Expected: the building hides the part of the band behind
    it. The laser beam and every health bar are drawn untinted over the band. The victim is
    tinted while in the ground layer and drawn untinted over the band once it is in the air
    layer.
  - (d) Fog. With fog of war on, let a computer house's lifter seize a vehicle in an area the
    human player has explored but cannot see. Then move a human unit until one end is seen.
    Expected: no band while both ends are unseen; a band from the first frame in which one end
    is seen.
  - (e) Another target. If a magnetic-beam weapon can be force-fired at a cell or at a non-vehicle
    target, repeat (a) that way. Expected: the rings are centred on the lifter's first-weapon fire
    point. If no such shot is possible, record that the case is unreachable in stock data.
  - Falsified by: a drawn colour or outline of its own, rings centred on the lifter in (a) or on
    the target in (e), a gradual fade, ring spacing or motion inconsistent with F5, a band that
    keeps following after the end, a later air-layer object, a laser or a health bar tinted by
    the band, or a band drawn while both ends are unseen under fog.
- **S-0502 carry items.** L2, L3 and L4 together give the held body facing (S-0502 claim 10), the
  hovering, landing and arrival exclusions (claims 12 to 15), the growing crash rate (claim 18),
  the cell-centre detour (claim 15 here), and the sinking-only-after-a-drop case (S-0502 claim 19,
  via L6).

- **L17 No approach against the gripped unit (claim 11; Q-2113).** Replay M-0015's two games, or a
  purpose-made game of computer-controlled lifters in *hunt* against a group of heavy enemy
  vehicles on a flat plateau, with the recorder. The lifters are a test copy of the lifter type
  (named test rules file) whose weapon has a non-zero minimum range and which has none of S-0215
  claim 37 part h's approaches.
  - For each hold (from the first tick the victim has the lifter as its holder to the release,
    the tick of an event of claim 12 that ends the grip),
    record on every tick: the lifter's destination, its target and the gripped unit's target; the
    spatial distance from the lifter's range-test source point to the gripped unit (S-0215 claim
    37 part b's test), whether it is below the weapon's minimum range or above its range, and
    whether the lifter can reach the gripped unit's cell. Record each order given to the lifter
    and every damage event on a holding lifter. At the release, record whether the gripped unit
    had arrived (claim 8) or which other event of claim 12 happened. Record each landing cell by
    role only: its land type and whether a terrain object stands on it.
  - A hold counts for the claim only if the gripped unit is inside minimum range on at least one
    tick on which the lifter processes its behaviour. Holds that never reach this are
    inconclusive for the claim and are reported separately. The release event is recorded for
    every hold, counted or not (Q-2113 question 3).
  - Expected: in a counted hold, no new destination on any tick on which the lifter's target is
    the gripped unit and no order is given to it. The target stays the gripped unit, except a drop
    by the per-tick rule on a tick on which the gripped unit's cell cannot be reached and it is
    out of range. Destinations that follow an order, or come after the target changed to another
    object, are listed apart and are not part of this expectation. The record gives each landing
    cell's land type and terrain object, with no expected value (Open questions).
  - Falsified by any new destination in a counted hold, at whatever distance and on whatever
    tick, while the lifter's target is the gripped unit and no order is given, or by a target
    drop on a tick before the release while the gripped unit's cell can be reached. The release
    events of claim 12, arrival among them, are not drops.
- **L18 Selecting a seized unit (claim 5), and its attack-move memory (claims 9 and 20).** A
  purpose-made flat map of clear land. The human house owns the lifter, the victim and a second
  vehicle of its own, as the common setup says. *Emptying the selection* is a right-button click
  on the battlefield with no scroll drag and no tool mode armed (S-0214 claim 26): it clears the
  selection and gives no order.
  - Part a. Before the seize, put the victim and the second vehicle in one control group. Select
    the lifter alone and force-fire it at the victim. Then, before the shot seizes the victim,
    press the control group's select key, so that the victim and the second vehicle are
    selected and the lifter is not. A run whose seize comes before that key press is
    inconclusive for the deselection in the seize's tick.
    - During the grip, one after another: empty the selection and click on the victim; drag a
      band box that covers both the victim and the second vehicle; empty the selection and press
      the control group's select key. Let the victim arrive (claim 8); empty the selection and
      click on it while it falls (claim 15). After it has landed and is restored (claim 20),
      empty the selection and click on it once more. So no click on the victim is made with an
      object selected.
    - Record on every tick the selection, whether the victim is seized, its flight state and
      every order given. For each input, record when it was made and whether the victim was
      seized then. An input that changes the selection (the group key's press, which selects at
      least the second vehicle into an empty selection, and the click after restoration) also
      shows the tick in which it was applied. An input that changes nothing
      leaves no trace of its tick, so it is placed only by when it was made.
    - Expected: the victim leaves the selection in the tick of its seize. While it is seized no
      input selects it: the click selects nothing, and the band box and the group key leave it
      out of the selection (the box may also select the lifter). The click and the band box
      during the grip, and the click during the fall, are checks only, since nothing recorded
      shows that the click pointed at the victim or that the box took it in; the group key is
      not. After restoration the click selects it. Falsified by a
      seized victim in the selection after its seize's tick, falling or not, or by a restored
      victim that the click does not select.
    - A grip probe made when the victim is no longer seized is void, and so is any probe that
      gives an order; the run is inconclusive for that input. A run whose fall click is made
      after the restoration is inconclusive for the fall.
  - Part b, a separate run. The bystander has an object visible to the human house, at a
    distance kept local that is beyond the victim's reach (S-0309 claim 23), and no enemy object
    comes within the victim's reach during the run.
    - Select the victim alone and give it an attack-move order on that object (S-0214 claim 5:
      use a victim whose type is eligible). Then select the lifter alone and force-fire it at
      the victim so that it is seized. A run in which the victim's target comes within its reach
      before the seize is void, and so is a run in which the victim takes a building of its own
      house as its target (claim 9's charging scan). Give the victim no further order.
    - Record on every tick the victim's target, its current and queued behaviour, whether it is
      seized, its flight state and every order given, until it holds the bystander's object as
      its target after restoration, or for a length kept local.
    - Expected: from the seize, the victim's target is nothing or the bystander's object. After
      restoration it comes to hold the bystander's object as its target, in *guard*, or in
      *attack* if a fall-back to rest resumed the attack move (S-0309 claim 23 part b). This
      after-restoration target is a check only: an ordinary *guard* search can find the object
      too (S-0309 claim 8), so it does not confirm the kept memory. Falsified by a restored
      victim that never holds that object as its target, in a run that lasts at least one full
      targeting interval of the victim after restoration (S-0309) and in which no fall-back in
      the grip removed the memory; a shorter run is inconclusive.
    - Conditional, when the victim's target becomes the bystander's object in the grip, and
      *guard* lacks the no-threat property, or has it with the zombie property, in the rules data
      used (S-0215 claim 16; a test rules
      file may clear it, as a test value): its behaviour stays *guard* while it is gripped, and
      it is not set down before arrival (claim 8) by a drop of its target. Falsified by
      *attack* current in the grip, or by a grip that ends before arrival with the victim's
      target dropped. A run with no such change in the grip is inconclusive for this part.
      When *guard* has the no-threat property but not the zombie property, a run in which the
      victim survives damage from a known attacker while gripped is void for this part (claim
      9's fall-back in the grip).
- **L19 Minimum range and spawners (claims 3 and 20; Q-2160).** A named test rules file gives a
  test copy of the lifter type a seizing weapon with a non-zero minimum range and none of S-0215
  claim 37 part h's approaches. It gives a test missile-launcher vehicle one rocket slot, long
  pause and tilt durations and a size the seize accepts. All of these are test values. Both
  belong to the human house. Record on every tick, for both: position, the spatial distance from
  the lifter's range-test source point to the launcher, target, destination, fire events, the
  seized state and the grip. For the lifter record also each detonation of its seizing projectile
  with its tick. For the launcher record also its controller target, its pending target, its slot
  states, each launch and each rocket's position.
  - (a) The launcher stands inside the minimum range of the lifter, which is ordered to fire at
    it (force-fire). Expected: no fire event while the distance is below the minimum range, an
    approach destination for the lifter, and a first shot and a seize once the distance has
    reached the minimum range. Let the run go on until the grip ends by arrival (claim 8) and
    the launcher is restored (claim 20), then order the lifter to fire at the launcher again.
    Expected: the lifter's target is empty after the arrival, and the new order gives a shot,
    once the distance is at least the minimum range, and a second seize. Falsified by a fire
    event at a distance below the minimum range at the tick of that fire event, by a seize whose
    shot was fired at such a distance, by a lifter target kept after the arrival, or by no second
    seize after a shot at a restored launcher that meets claim 3's conditions.
  - (b) The launcher force-fires at a named high-health test structure of the human house in
    range. The lifter's force-fire is timed, varied across runs, so that the seize falls after the
    controller's adopting pass and before the salvo's release. Count only such runs. Expected in a
    counted run: no fire event and no launch until restoration; the controller target stays the
    test structure through the seize unless a reset of S-0506 claim 11 is recorded; after
    restoration, with no order given, the launcher launches at the test structure once it is not
    moving. A run in which the launcher fires before its first launch after restoration is
    inconclusive for this part. Falsified by a launch while seized, by an empty controller target
    after the seize with no reset recorded, or by no launch after restoration while the kept
    target is in range. In other counted runs, the human house sells the test structure while
    the launcher is seized. Expected then: the controller target is empty from the tick the
    structure leaves play, with a reset recorded, and no launch follows restoration. Falsified
    by a controller target kept after the structure has left play, or by any launch after
    restoration with no new fire event.
  - (c) Time the seize while a rocket is on the rail and, in other runs, after it has left its
    slot. Expected in a run with the rocket on the rail: the rocket vanishes with no detonation,
    and its slot, emptied by the seize, is refilled at the controller's next pass, even when the
    launcher is still seized then. Falsified by any other outcome for that rocket, or by a refill
    of that slot that waits for restoration or comes later than that pass. Expected in a run
    after the rocket has left its slot: the rocket flies on and detonates at its aim point, and
    the seize does not change when the slot, already empty, is refilled: the slot is refilled at
    the same pass as in a run with no seize. Falsified by a rocket outcome other than that, or by
    a refill of that slot at a different pass from the no-seize run.
  - (d) As part b, but with the seize timed after the launcher's fire event and before the
    controller's adopting pass. Count only such runs. Expected in a counted run: the pending
    target survives the seize, the controller adopts it at a pass while the launcher is seized
    (or resets if the test structure is out of range from where the launcher then is, with a
    reset recorded), and with no reset the launcher launches at the test structure once restored
    and not moving, with no new fire event. A run in which the launcher fires before that launch
    is inconclusive for this part. Falsified by an empty pending target after the seize and
    before the controller's next pass, by an adoption that waits for restoration, or by no
    launch after restoration with no reset recorded.
  - (e) A second test copy of the lifter has a seizing weapon whose projectile is slow (its speed
    a test value), so that it detonates some ticks after the shot. The launcher is ordered to
    move towards that lifter, and the lifter's force-fire at it is timed, varied across runs, so
    that the shot falls while the distance is at least the minimum range and the detonation falls
    after the distance has dropped below it. Count only such runs in which claim 3's conditions
    and claim 2's hold at the detonation. Expected in a counted run: the launcher is seized at
    the detonation. Falsified by a counted run with no seize at the detonation.
  - (f) As part e, with the same slow test projectile, but the launcher is ordered to move away
    from the lifter, and the force-fire is timed, varied across runs, so that the shot falls
    while the distance is at most the weapon's range and the detonation falls after the distance
    has passed it. Count only such runs in which that shot's detonation is recorded and claim 3's
    conditions and claim 2's hold at it. Expected in a counted run: the launcher is seized at
    the detonation. Falsified by a counted run with no seize at the detonation.
  - (g) As part b, with a second test copy of the missile launcher that has two rocket slots (a
    test value) and otherwise the first copy's test values. The lifter's force-fire is timed,
    varied across runs, so that the seize falls after the first slot's launch and before the
    second slot's launch. Count only such runs. Expected in a counted run: the first slot's
    rocket, preparing at the seize, flies on and detonates at its aim point, and the second
    slot's object, docked at the seize, is removed with no launch and no detonation. Both slots
    are empty after the seize and are refilled at the controller's next pass, seized or not.
    Falsified by a vanished or stopped first rocket, by a launch from the second slot while the
    launcher is seized, or by a refill of either slot that waits for restoration or comes later
    than that pass.
- **L20 Launchers against lifters on side A (Open questions; Q-2160 question 1).** Replay the
  games of M-0016 in which a lifter group starting on side A meets a missile-launcher group, with
  the recorder. For each launcher record every tick on which it is seized, gripped, dropped, set
  down and restored, and every fire event and launch with its tick. For each lifter record every
  hold start, the spatial distance from its range-test source point to the gripped unit at that
  start, and the event of claim 12 that ends the hold. No expected value: the record answers the
  open question.

## Open questions

- Whether stock data lets a magnetic-beam weapon make a wave whose target is not a vehicle
  (claim 25's second case; scenario L16 part e). The picture for that case is stated either way.
- Whether a magnetic wave is still drawn when its first end is off the view while its band
  crosses the view (claim 25). The layer pass's own visibility test for the wave was not read.
- Which pixels on the boundary of the wave band are filled (claim 25). The band's interior is
  stated; the edge rule of the original's fill is not pinned down.

- (Q-2160 question 1.) Whether the launchers are seized at all in the games of M-0016 in which a
  lifter group starting on side A meets a missile-launcher group, and how often a restored
  launcher is seized again, is not established: M-0016 records outcomes only. Scenario L20
  records it.
- (Q-2113 question 3.) What ends each hold in M-0015's games is not settled: M-0015 does not
  record it. Scenario L17 records the release event of every hold; claim 11 expects arrival
  (claim 8) or another event of claim 12.
- Whether the cell of claim 7's movement point, where a gripped unit is set down, can be a cell
  with a tree or other terrain object, or a rock cell (Q-2113). Scenario L17 records each
  landing cell by role.
- How a gripped unit with a stand-in other than the jumpjet moves after the approach step's
  part e has cleared its destination. That its grip holds is claim 9's.
- How the factory-contact protection of claim 3 can be reached in play.
- Whether a squared sum that wraps negative in F3 is reachable on a playable map. F3 gives its
  result if it is.
- How a vehicle moves with the walking locomotor installed as a stand-in, and the details of the
  original's drop-pod descent (its height, its horizontal displacement, its speed and its landing effects). No movement spec
  covers them; claim 23 gives our engine's safe results.
- How a landed aircraft moves in the original with the driving, walking or ship stand-in
  (claim 23 gives our engine's rule; scenario L13 part e records the driving case).
- Whether a driving seized unit can stop exactly on a building lifter's docking point, and
  whether a lifter's surroundings can leave the teleporting landing search with no cell
  (claim 23). The outcomes are stated.
- Whether ordinary play can release a seized unit before its first jumpjet update, or keep a
  found point in another cell while its own cell holds another object (claim 13). The
  outcomes are stated; scenario L3 runs b and f are inconclusive until a run reaches them.
- Deploying, undeploying and unloading while seized (claim 10's last bullet) are allowed because
  no seized-state test refuses them; no refusal is claimed. Low confidence until scenario L9 part g
  gets an order through to a seized unit gripped at height 0 or set down; whether the harness
  or the player can do so is open.
- Whether any state other than erasure and the leaving stage keeps claim 20's test from being
  made at the end of a turn: a deactivated or paralysed vehicle (S-0215 Terms, S-0214), or a
  unit phasing in after a chrono move. Claim 20 states the test for every other turn.
- Whether a landed aircraft that is being erased is tested at the end of its turns (claim 20,
  *Units being erased*).

## Changes for other specs

- **S-0506, claim 13 (Q-2160).** Claim 13's slot clearing does not change the controller target,
  and the controller's passes go on while the spawner is seized. The pending target then follows
  claim 25 alone. Claim 13 could say so, and could point to claim 20 here for the launch that can
  follow restoration with no new shot. Medium, `measurable` (scenario L19 parts b and d here for
  the kept controller target, parts c and d for the passes while seized).
- **S-0215, claim 37, part c (the approach step's nothing-to-do case).** Part c makes the step do
  nothing when the actor's target is the object that holds the actor
  with a magnetic beam. By claim 11, and as scenario L17 expects, the step does nothing when the
  actor grips a unit and its target is that unit. The reverse case needs no exception, since a
  seized unit ignores destinations (claim 9). Part c should state the gripping case instead,
  after part h, with backing that cites scenario L17 here. Part a should say that its per-tick
  drop still runs for a lifter whose gripped unit is its target: the empty approach step gives
  no destination, and an unreachable gripped unit out of range, minimum range included, is
  dropped as target while the grip stays (claim 11).
- **S-0215, claim 37, parts c and e (Q-2151).** Part c's last sentence could point to claim 9
  here for the step's drop and failed search, which end a grip while the gripped unit has a
  destination. Part e could say that its clearing removes the destination directly, not by an
  assignment of an empty destination, so it is no effective clear in S-0502 claim 24's sense
  and no jumpjet stop follows it (claim 9).
- **S-0502, claims 24 and 28.** Claim 24's direct destination discards could list S-0215 claim
  37 part e's clearing beside the factory ascent report's. Claim 28 could say that its
  retention applies to a seized vehicle too, before the seized unit's destination refusals
  (S-0607 claims 9 and 13).
- **S-0214, Terms, *Can be selected*.** A seized unit (S-0607 Terms) cannot be selected by any
  selecting input until it is no longer seized (claim 5). The term could list it among the
  exclusions.
- **S-0215, claim 13, first row (temporary locomotor).** The row gives a unit its own locomotor
  back when it falls back to rest. For a seized unit's temporary jumpjet locomotor, the same
  test is also made at the end of each of the unit's turns (S-0203), after its movement and
  after the point at which it may process its behaviour, whether or not its behaviour is
  processed in that turn, whatever its behaviour, and not while it is being erased or in the
  leaving stage (S-0607 claim 20). The row should say so for that locomotor, or point to
  claim 20.
- **Questions for other specs (not rules).** The end-of-turn test of claim 20 is shared by every
  infantry unit, vehicle and aircraft, but only the seized unit's jumpjet locomotor was traced
  through it. Each of these should check, against its own conditions for ending a temporary
  locomotor, whether that locomotor can also end at the end of a turn without a fall-back to
  rest: S-0417 claim 5 (temporary driving during factory exit), S-0207 claims 21 and 28 (the
  restored teleporting state, and driving that stays active through the docking and the
  unloading), S-0402's chrono-miner rest cases after a seizure or a fall-back, and request
  Q-0233, which rests on S-0215 claim 13's first row.
- **S-0502, Edge cases (*No landing slot*) and scenario J27 part (c).** Both say that a seized
  hovering set-down unit, with `quirks.jumpjet_no_cell_self_destruct` off, retries the stop's
  search at each fall-back to rest. Claim 20 sets the cadence to the end of each of the unit's
  turns (S-0203), and not at a fall-back to rest. Replacement for the Edge cases sentence:
  "while hovering it is not restored and retries the search at the end of each of its turns
  (S-0203), not at a fall-back to rest". Replacement for J27 (c): "retries the search at the
  end of each of its turns under S-0607 claims 9 and 20", and in its falsifier "a
  route-dependent retry, a turn that ends without a retry, or a retry at a fall-back to rest
  outside that test". Expected retry ticks: the search runs at the end of the unit's turn in
  every tick from the tick of the no-cell outcome on in which it takes a turn. A cell freed in
  a tick is found at the end of the unit's turn in that tick when the freeing comes before
  that turn in the update order, and otherwise at its next retry. Both
  are design rules for the switch-off setting; the original destroys the unit instead.
