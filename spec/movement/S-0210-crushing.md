---
id: S-0210
title: Crushing (who crushes whom, when, the outcome, walls, warnings, slowdown and automatic crushing)
topic: movement
revision: 22
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0020, Q-0036, Q-0684]
supersedes: []
---

# S-0210: Crushing (who crushes whom, when, the outcome, walls, warnings, slowdown and automatic crushing)

## Summary

Which vehicles can crush, which objects they can crush, the moment at which crushing happens, what
happens to the crushed object, how crushing vehicles flatten crushable overlays and walls, how they
warn infantry ahead of them, when crushing slows a vehicle down, and when a vehicle chooses on its
own to drive over a crushable enemy. The entry outcomes that let a crushing vehicle treat a cell of
crushable enemies as clear are in S-0209 (claim 5). It also gives where a wall type's damage-stage
count and strength come from, with their engine defaults, and which house owns a wall that the map
places.

## Terms

Glossary terms are used as defined there, in particular *cell*, *lepton*, *tick*, *vehicle*,
*infantry*, *building*, *overlay*, *overlay data value*, *house*, *human-controlled*,
*computer-controlled*, *AI house*, *team member*, *reference point*, *bridge deck*,
*movement zone*, *registration index*, *scenario random stream*, *recorder*, *measurement setup*,
*train* and *vehicle thief*.
Terms from S-0209: *cell layer* with its two kinds, the *ground layer* and the *bridge-deck layer*
(its Terms), *listing order* (its claim 13), *invulnerable*, *entry outcome*, *house skill level*.
From S-0208: *standing position* and its named positions (table P), *scatter request* (with its
*forced* and *move-anywhere* options), *effectively forced* (its claim 11), and *fraidycat*,
*scatter property*, *scatter veterancy ability*, *player-scatter rule* and *locomotor moving* (its
Terms). From S-0207: *track*, *begins a track*,
*current step*, *first cell*, *second cell* and *crush mode* (its claim 7), *target fraction* and
*speed fraction* (its Terms), *passive flag*, *move destination*. From S-0215: *opportunity target*
(its Terms and claim 33). From
S-0107: *crushable* (the flag), *deploying*. Terms used by this spec:

- **Allied.** As in S-0209, but always from the crusher's side: an object is allied when the
  crusher's house is allied with the object's house (a house is allied with itself).
- **Crusher.** A vehicle that can crush: its type has the imported crusher flag, or it has gained the
  crusher veterancy ability.
- **Omni-crusher.** A vehicle whose type has the imported omni-crusher flag. On its own it does not
  make a vehicle a crusher (claim 1).
- **Omni-crush resistant.** A type flag that protects its units from omni-crushers.
- **Crushable flag.** A type's imported flag that lets crushers kill its units by driving over them.
  An infantry type can make its units uncrushable while deployed (S-0107, State and ordering, deploy
  and undeploy rows); this spec calls such a unit *deployed-uncrushable*.
- **Crushable overlay.** An overlay type with the crushable flag (for example a fence).
- **Wall overlay.** An overlay type marked as a wall.
- **Damage stage, damage-stage count.** A wall type has a *damage-stage count* *D*, an imported
  integer (claim 13, Import mapping). A cell holding a wall overlay is at a *damage stage* *s*, an integer from
  0 (undamaged) to *D* − 1, its *last stage*. How *s* is held in the cell's overlay data value is
  claim 8. How other damage changes *s* belongs to the overlay specs.
- **Wall strength.** A wall type's imported strength field (claim 13, Import mapping), in wall-hit
  points.
- **Wall owner.** The house that a wall cell belongs to, or none (claim 14). S-0209's wall rows
  compare it with the mover's house.
- **Crusher-all movement zone.** The movement-zone value that lets a unit pass through walls by
  crushing them (Import mapping).
- **Driving, ship and walking-vehicle locomotors.** The ways a vehicle type can move that this spec
  distinguishes: the driving locomotor is the one S-0207 describes, the ship locomotor moves vessels
  on water, and the walking-vehicle locomotor moves vehicles that walk on legs. A type's locomotor is
  an imported field (Import mapping).
- **Crush range.** A game-wide imported distance in leptons used by automatic crushing (claim 11).
- **Auto-crush flag.** A vehicle type's imported flag that lets its units choose crushing
  targets on their own (claim 11).
- **Player-auto-crush rule.** A game-wide imported yes/no rule that does the same for every vehicle
  (claim 11).
- **Auto-crush threshold.** A game-wide imported house skill level that a computer-controlled house
  of the original must reach before its crushers choose crushing targets on their own. Only the
  original's rule uses it (claim 11); AI houses of our engine never do.
- **Crush-slowdown state.** A yes/no state of a vehicle, set by the events of claim 9 and ended by
  crushing an overlay (claim 8).
- **Body tilt.** A presentation value of a vehicle: the rate per tick at which its drawn body rocks
  forward (negative) or back, in *tilt units* per tick. The rendering specs own how a tilt unit
  relates to the drawn angle, how it is drawn and how it returns to level.
  "Level" means a rate of 0. The *forward rocking angle* is the drawn forward or backward pitch that
  this rate builds up (claim 12). *Tipping forward* means that the body tilt is *set* to a negative
  value (Constants), replacing what it was.
- **Accelerating type.** A vehicle type whose imported acceleration flag is on, so that its speed
  changes gradually (for the driving locomotor, under the acceleration rules of S-0207 claim 10).
  The same flag applies to the ship locomotor (claim 9).
- **Balloon-hover type.** A vehicle type with the imported balloon-hover flag (S-0102 Terms). Only
  the Open questions use it here.
- **Tilt-on-crush flag.** A type's imported flag (Import mapping) that makes a vehicle of that type
  tip forward in the cases of claims 8 and 9.

## Behaviour

1. **Who can crush.** Only a crusher (its type's crusher flag, or the crusher veterancy ability)
   crushes objects by driving over them. A vehicle whose type has only the omni-crusher flag crushes
   nothing by driving over it. Infantry never crush.
   - Backing: `measurable` (scenario K1, part d for the omni-crusher flag alone, part e for the
     veterancy ability, part l for infantry)
   - Confidence: high for the requirement, medium for "infantry never crush" (examined only for the
     walking locomotor)

2. **Who can be crushed.** An object in a crusher's path can be crushed by that crusher when it is a
   unit or a building (an infantry, vehicle, aircraft or building object), not allied, not
   invulnerable, not falling (for example a unit dropped from the air on its way down), and one of
   these holds:
   - the crusher is an omni-crusher, the object is not a building, and the object's type is not
     omni-crush resistant,
   - the object's type has the crushable flag and the object is not deployed-uncrushable.

   So only the crushable flag can make a building crushable.

   An allied unit (the crusher's own house included) is never crushed. Alliance is judged against
   the house that owns the crusher at that point of the tick: when a vehicle thief takes the crusher
   over within a cell layer (claim 5), objects listed before the thief are judged against the
   crusher's original house and objects listed after it against the thief's house (State and
   ordering).
   - Backing: `measurable` (scenario K1: parts a to c for the crushable flag and alliance, f and g
     for the omni-crusher route, h for invulnerable objects, i for buildings, j for deployed
     infantry). The "not falling" condition: scenario K2 part d.
   - Confidence: high; medium for "not falling"

3. **When crushing happens.** A crusher crushes on each tick on which its movement reaches the centre
   of a cell, which is where its movement steps end (S-0207), and on no other tick. It then crushes
   every object of claim 2 in that cell's cell layer whose reference point (its position, or for a
   building the centre of its foundation; S-0204 claim 4) lies horizontally less than 128
   leptons from the crusher's centre (Formulas F1). Height is ignored within the cell layer. Because
   every standing position lies at most about 91 leptons from the cell centre (S-0208 table P), a
   crusher that stops exactly at a cell centre crushes all crushable infantry standing in that cell
   layer at once. An infantry unit that is still walking into the cell is crushed only if it is
   already less than 128 leptons from the centre. A building whose foundation is two cells wide and
   two deep is never crushed, because its reference point is about 181 leptons from the centre of
   each of its cells. A crusher on a bridge deck crushes in the bridge-deck layer, one under the deck
   crushes in the ground layer.

   When a driving harvester gives the track-completion arrival report of S-0402 claim 10,
   that report's result is visible before the cell-centre crushing of this claim and claim 8
   on the same tick. In particular, an accepted report can have queued *unload* before a
   crushable overlay at the arrival cell is crushed. This is separate from claim 9's
   cell-boundary crush-slowdown effects, which can already have occurred while the track
   was still running.
   - Backing: `measurable` (scenario K2 parts a and c; K1 part i for buildings; K10 part d, runs d2 and d3, for the
     bridge-deck layer; S-0402 scenario H14 for the arrival-report ordering)
   - Confidence: high for the 128-lepton rule and the cell layer, medium for "only at cell centres"
     and for the reference point of a building; high for the arrival-report ordering

4. **The outcome.** A crushed object is removed from the game on the same tick. It takes no damage,
   no warhead is applied, it shows no death sequence and leaves no corpse or death animation. The
   kill is credited to the crusher (as for any kill: score and veterancy belong to their specs), and
   the crushed type's crush sound plays at the crusher's position. Removing it ends whatever it
   controlled as on any removal (for example units it held under mind control are released).

   Actually crushing at least one such object makes a cloaked or cloaking crusher start
   uncloaking on that tick, including when the victim is infantry rather than a vehicle
   (S-0516 claim 11 (c)). The cloak sound plays once when this starts an uncloak, not once
   per victim; repeated causes while already uncloaking do not restart it or repeat the sound
   (S-0516 claims 7 and 15). Empty passage does not cause this uncloak.
   - Backing: `measurable` (scenario K1, part a for removal, damage, kill credit and the crush
     sound, part k for the release of mind-controlled units; K14 for the crushing-triggered
     uncloak and its exclusions). The position of the sound is
     `unmeasured`: a capture shows that the sound plays, but the crusher and the crushed object are
     less than 128 leptons apart, too close for the difference in position to be heard.
   - Confidence: high for removal without damage, medium for the kill credit, the release of
     controlled units and the sound position; high for the crushing-triggered uncloak

5. **A vehicle thief is not crushed by its quarry.** When the object to be crushed is an infantry unit
   of a vehicle-thief type whose destination is the crusher itself, and the crusher is not a train,
   it is not crushed: the crusher is taken over instead, as when the thief enters it. The thief
   disappears and the crusher changes owner to the thief's house at once, on that tick. The new house
   already applies to the rest of that cell layer: an object listed after the thief (State and
   ordering) is judged against the crusher's new house, so one allied with the thief's house is not
   crushed, while an object listed before the thief is judged against the original house. A
   take-over alone does not count as crushing for claim 6 and does not cause claim 4's uncloak
   (S-0516 claim 11 (c); scenario K14). What else of the thief's state the crusher
   takes over is left to the vehicle-theft specs.
   - Backing: `measurable` (scenario K3)
   - Confidence: medium for the take-over instead of crushing and for the new house already applying
     to the objects listed after the thief

6. **Crushing a vehicle tips the crusher.** When a crusher crushes at least one vehicle on a tick, it
   tips forward if its body tilt is level, counting the change that claim 8 makes for an overlay
   crushed on the same tick: its body tilt is set to −0.05 (Constants), whatever its locomotor and
   whether or not its type has the tilt-on-crush flag. This only affects how it is drawn. So a
   crusher whose body tilt is level just before it crushes an overlay and a vehicle together does
   not tip forward, and its body pitches back (scenario K8 part c).
   - Backing: `measurable` (scenario K8, with a vehicle of the driving locomotor; no scenario crushes
     a vehicle with another locomotor)
   - Confidence: medium for the driving locomotor; low for "whatever its locomotor"

7. **Warning ahead.** Each time a crusher with the driving locomotor begins a track (S-0207 claim 7)
   while the cell layer of that track's first cell, the cell its current step enters, holds
   infantry, it sends a cell-wide scatter request to that cell layer (S-0209 claim 11),
   with no threat point, forced, not move-anywhere. A vehicle that turns in place before it can begin
   the track sends nothing until the track begins. A crusher with the ship locomotor does the same
   when it begins a track. Which infantry obey, and where they go, is S-0209 claim 11 and S-0208
   claims 11 to 13. In particular, for an infantry unit whose locomotor is moving at that moment
   the request is not effectively forced (S-0208 claim 11), so such a unit refuses it unless all of
   these hold: it is a fraidycat, and its current behaviour has the scatter property, and one of
   the following applies: its owner is computer-controlled, it is a team member, the game-wide
   player-scatter rule is on, or it has the scatter veterancy ability. For units of an AI house the
   first two alternatives never hold: an AI house counts as human-controlled and has no team members
   (glossary, *AI house* and *team member*), so for its units only the rule or the ability can make
   up the last condition. Units of the neutral and special houses keep the computer-controlled
   alternative (glossary, *AI house*, *Player houses only*). A walking unit
   that refuses keeps walking and can walk into the crusher's path.
   - Backing: `measurable` (scenario K2 part b for the driving locomotor, K10 part a for the ship
     locomotor)
   - Confidence: medium for the request, its options and its moment; low for the ship locomotor

8. **Crushing overlays and walls.** A crusher crushes the overlay of a cell when it reaches the cell's
   centre (claim 3) and the cell holds a crushable overlay, or holds a wall overlay and the crusher's
   movement zone is crusher-all. Every crushed overlay plays its crush sound at the crusher, adds 0.02
   to the crusher's body tilt (Constants; a single-precision addition under the number rules of S-0205,
   which affects only the drawing) and ends the crusher's crush-slowdown state (claim 9). What
   becomes of the overlay and of the walls beside it depends on what was crushed. Crushing
   only an overlay or wall does not cause the crusher to uncloak (S-0516 claim 11 (c),
   scenario K14):

   | Crushed overlay | The overlay | The four orthogonal neighbouring cells |
   |---|---|---|
   | wall overlay of a type with *D* > 2, at damage stage *D* − 2 | removed from the cell at once | each one that holds an undamaged (stage 0) wall of the same type takes a wall hit of 200 (F3) |
   | wall overlay at any other damage stage, or of a type with *D* ≤ 2 | removed from the cell at once | no hit |
   | crushable overlay that is not a wall overlay | stays as it is (`quirks.crushable_overlay_stays`) | no hit |

   A crushed wall is removed whatever its damage stage, and the cell becomes free of it: passability
   and connectivity are updated as for any destroyed wall, which S-0211 and S-0212 own. Whether each
   neighbour hit succeeds, and the random draws involved, are given by Formulas F3. What a
   successful hit does to a wall is owned by the overlay specs.

   A wall cell's damage stage is held in its overlay data value *v* as *s* = ⌊*v* / 16⌋ (integer
   division, *v* from 0 to 255). The remainder below 16 belongs to wall linking, which the overlay
   specs own.

   Separately, a crusher with the driving locomotor whose type has the tilt-on-crush flag tips
   forward (body tilt set to −0.05) when, on a track in crush mode (S-0207 claim 7), it drives into a
   cell holding a wall overlay. Whether a crusher may enter such a cell at all is S-0209 claim 5 and
   the passability rules of S-0211.
   - Backing: `measurable` (scenario K4, also for the overlay's crush sound; K8 part c for a crushable
     overlay that is not a wall overlay staying in its cell, `quirks.crushable_overlay_stays`). The draws of F3, their number,
     their order and each neighbour's outcome, are `measurable` with the random-state recorder (K4
     part c); the share of neighbours hit is measurable without it.
   - Confidence: high for the removal of a crushed wall, medium for the neighbour hits and their
     draws, for the stage encoding ⌊*v* / 16⌋ and for the rest

9. **The crush-slowdown state.** A vehicle enters the crush-slowdown state in three cases, one per
   locomotor:
   - **Driving locomotor.** Its movement zone is crusher-all, and it drives into a cell on a track in
     crush mode (S-0207 claim 7) while the ground layer of that cell holds a vehicle. The cell
     layer looked at is always the ground layer, whichever cell layer the driving vehicle is in
     (`quirks.crush_slowdown_ground_layer`), and the driving vehicle itself counts as a vehicle in
     the cell it has just driven into (`quirks.crush_slowdown_counts_self`). So:
     - a vehicle on the ground enters the state each time it drives into a new cell on a crush-mode
       track, whether or not another vehicle is there,
     - a vehicle on a bridge deck is in the bridge-deck layer, so it does not count itself there,
       and enters the state only when a vehicle stands in the ground layer under the deck cell it
       drives into, and vehicles on the deck never count,
     - a vehicle driving on the ground under a bridge counts itself, as on any ground cell, and a
       vehicle on the deck above it makes no difference.

     The crusher flag is not required. Because crush mode is set by
     the second cell as well (S-0207 claim 7), on the ground a single vehicle standing in the path
     sets the state from the moment the vehicle drives into the cell before it, and so does a
     crushable overlay in the path, also for a vehicle that is not a crusher. A wall in the path sets
     it, for a vehicle of movement zone crusher-all, from the cell before the wall to the wall's centre
     (where claim 8 ends it for a crusher).
   - **Ship locomotor.** It is a crusher, and it drives into a cell whose overlay is of the overlay
     type with registration index 0 (S-0004) on a track in the ship locomotor's crush mode. That
     crush mode is set when the vehicle begins a track whose first cell (when its entry outcome is *clear*,
     S-0209 claim 5) or second cell holds a crushable overlay or, for a type of movement zone crusher-all, a wall
     overlay. Unlike the driving locomotor's, it is never set by a vehicle.
   - **Walking-vehicle locomotor.** It is a crusher, and it sets off on a step into a cell whose
     overlay is of the overlay type with registration index 0. No crush mode is involved.

   The ship and walking-vehicle cases depend on the registration index of the overlay type, not on
   any property of it (`quirks.crush_slowdown_first_overlay`).

   Each time one of these events occurs, whether or not the vehicle is already in the state, a
   vehicle whose type has the tilt-on-crush flag tips forward again: its body tilt is set to −0.05
   with the driving locomotor, −0.02 with the ship locomotor and −0.01 with the walking-vehicle
   locomotor. So a vehicle that drives through consecutive cells of a crush-mode track tips afresh
   on each of them.

   While the state lasts, a vehicle with the driving or ship locomotor that is of an accelerating
   type and not passive has its speed held down on every tick on which its locomotor updates its
   speed fraction. The update rules are S-0207 claim 10 for the driving locomotor and apply
   unchanged to the ship locomotor by S-0504 claim 1. The slowdown applies to those updates only
   for a vehicle of an accelerating type that is not passive. On each such update, with either
   locomotor, the crush-slowdown state takes precedence over every other rule for the new speed
   fraction (braking near the destination, sinking, accelerating, decelerating): the target
   fraction and the speed fraction both become the smaller of the target fraction and 0.2
   (Formulas F4). So the speed fraction does not ramp: a vehicle in the
   state jumps at once to the lowered target, from rest as well as from a higher speed. A new target
   fraction set when a later track begins is lowered again on the next update, so the limit lasts as
   long as the state. Outside the state, the ship locomotor's speed-fraction rules are owned by
   S-0504 claim 1, which applies S-0207 claim 10. A vehicle of a type that is not accelerating is not slowed. A vehicle with the
   walking-vehicle locomotor is never slowed by the state; there it affects only how the vehicle is
   drawn (claim 12).

   The state is not ended by orders, by stopping or by leaving the cell. It ends only when the
   vehicle next crushes an overlay (claim 8), which needs the crusher flag or ability. So an
   accelerating crusher-all vehicle with the driving locomotor keeps moving at no more than 0.2 of
   its speed, even under new move orders, until it crushes a wall or a crushable overlay
   (`quirks.crush_slowdown_persists`), and one that is not a crusher stays in the state once it has
   entered it, until it gains the crusher veterancy ability, if ever, and crushes an overlay.
   - Backing: `measurable`: scenario K5 for the driving-locomotor case on the ground, the vehicle
     counting itself, the limit of F4, its persistence across orders and its end (a vehicle under a
     bridge is a vehicle on the ground for this rule, since it always counts itself); K10 part d for
     the bridge-deck case; K10 part b for the ship case and the ship's speed limit; K10 part c for the
     walking-vehicle case and the absence of a speed limit there; K5 run e and K10 part b for the
     tips of the driving and ship locomotors. The walking-vehicle tip of −0.01 is `unmeasured`: it
     builds up a drawn pitch of under 2 degrees, below what screenshots resolve reliably.
   - Confidence: medium for the driving-locomotor case on the ground including the vehicle counting
     itself, for the ground layer being the one looked at, for the bridge-deck case, for the limit of
     F4 and for its persistence across orders; low for the ship and walking-vehicle cases (the ship's crush mode included) and for the tip values
     of those two locomotors

10. **Trains run over everything.** A train that drives into a cell deals 10000 damage to every
    unit and building in that cell's cell layer that fails the test of claim 2 for this train, and takes 20
    damage itself for each of them. The test is applied whether or not the train is a crusher, and
    allied and own objects always fail it, so they are hit too. Both amounts are dealt with the
    game-wide demolition warhead as damage that ignores defences (S-0107 Terms), so the full amounts
    apply without scaling by armour, and with no known attacker: no kill credit, and an infantry unit
    that survives the hit is sent no threat-point scatter request. It can still be sent the general
    damage request, by the case of S-0208 claim 15 (request A) for damage with no known attacker,
    under the conditions stated there. The victims' passengers do not escape. Objects that pass the test
    are not damaged. If the train is a crusher they are crushed by claim 3; if it is not, it leaves
    them unharmed. Whether a train also harms other kinds of object in the cell layer, such as a
    terrain object, is not established (Open questions).
    - Backing: `measurable` (scenario K6: part a for the amounts and the kill credit, b for own
      objects, c for passengers, d for a train that is not a crusher and for infantry being hit).
      The scatter requests a surviving infantry unit is sent are `unmeasured` here: they follow
      from the missing attacker and S-0208 claim 15, and observing them would need an infantry type
      whose strength exceeds 10000.
    - Confidence: medium

11. **Automatic crushing.** A crusher may choose, as a target to drive over, a crushable enemy
    unit, on the occasions on which the behaviour specs (S-0215) let a unit pick a target of its own,
    when all of these hold:
    - the target is crushable by it (claim 2) and does not appear disguised to the crusher's house
      (S-0516 claims 35–36),
    - the distance between their centres (in three dimensions) is at most the crush range (F2),
    - the game-wide player-auto-crush rule is on, or the crusher's type has the auto-crush flag.

    This disguise condition belongs to the automatic choice in this list, not to crushing along an
    existing route: a unit that appears disguised can still be crushed under claims 2–3 when the
    crusher is ordered through its cell. The disguise condition does not add a requirement to the
    omni-crusher or original's attack-target variants below.

    When the target is chosen, the crusher is given a move to the target's cell, as if ordered to move
    there. When it reaches the cell centre, claim 3 applies. Which behaviours give the crusher the
    chance to choose, and so what counts as being busy with another order, belongs to the behaviour
    specs. Scenario K7 shows one occasion that does not offer it: a crusher with an attack order
    (case 7).

    **Omni-crusher variant.** A vehicle whose type has the omni-crusher flag and whose current target
    (the object it has been told or has chosen to attack; glossary, *target, destination*) it can crush
    under claim 2 drives at that target in the same way, at any distance. The crusher flag is not
    required for this choice, although without it the vehicle crushes nothing (claim 1). Unlike the first list, the variant is not tied to the occasions on which a unit picks a target of its own: it applies also while the vehicle has a move destination. There is one exception: the
    vehicle does not do this when its target is an opportunity target (S-0215 claim 33), one that it
    picked by itself while moving, harvesting or guarding.

    **Replacement approach.** The omni-crusher variant above and the original's attack-target
    variant below replace the common approach step of S-0215 claim 37 (part h). Their destination
    is the target object itself, not a fixed cell chosen by the common approach rules. The
    cadence and occasions for taking that step are S-0215 claim 37 (part a); the eligibility
    conditions remain those of this claim. This differs from the automatic choice in the first
    list, which gives a move to the chosen target's cell.

    **Our engine and the original's computer-controlled houses.** The rules above are the rules of
    our engine for every player house, human-controlled or AI house (glossary, *AI house*). The
    original treats a house that its computer player plays differently in three places, and our
    engine does not reproduce them for AI houses. As the glossary entry *AI house* says, an AI house
    follows the rules of a human-controlled house, so for an AI house:
    - the permission to choose targets automatically is the third condition of the list above, never
      the computer-controlled row of the table below,
    - the attack-target variant below does not exist for its units,
    - the opportunity-target exception of the omni-crusher variant applies to it as to a
      human-controlled house.

    This is a departure of our engine by design choice of scope, with no quirk switch. Units of the
    neutral and special houses are not units of an AI house: they keep the original's
    computer-controlled rules below (the table row, the attack-target variant and the exemption from
    the opportunity-target exception) in our engine (glossary, *AI house*, *Player houses only*). Our AI may
    obtain the original's computer behaviour by giving its crushers ordinary move orders to the cells
    of targets it chooses. The original's rules, which scenario K7 measures with computer-owned
    crushers:

    | Original: crusher's owner | Automatic crushing allowed when |
    |---|---|
    | human-controlled | the game-wide player-auto-crush rule is on, or the crusher's type has the auto-crush flag |
    | computer-controlled | its house skill level is at least the game-wide auto-crush threshold, and its difficulty index is not 2, the index given by the lobby's easy computer entry (S-0412 claim 4; the auto-crush flag and the player-auto-crush rule play no part) |

    The neutral and special houses have difficulty index 1 and house skill level 0 (S-0412
    claims 3 and 5). They therefore fail this table's skill-level condition with the engine-default
    auto-crush threshold of 2, or with any positive imported threshold. A threshold of 0 or less
    lets them pass it. This restriction belongs only to the table's automatic choice: it does not
    exclude the attack-target or omni-crusher variants.

    - *Original: attack-target variant.* A crusher of a computer-controlled house that already has a
      target to attack drives at it instead of attacking it in the usual way when all of these hold:
      the target is a unit or building it can crush (claim 2), it is less than the crush range away
      (strict, F2), and the crusher has no move destination at that moment (S-0207), that is, it is
      not on its way somewhere. The auto-crush flag, the player-auto-crush rule, the auto-crush
      threshold and the difficulty level play no part in it. A vehicle of a human-controlled house
      with a target to attack never does this.
    - *Original: opportunity targets.* A vehicle of a computer-controlled house with the
      omni-crusher flag drives at an opportunity target too, which the human-controlled vehicle of
      the exception above does not.
    - Backing: `measurable` (scenarios K7 and K13; K13 for the disguise condition and its distinction
      from crushing along an ordered route). The first list under a human-controlled owner: cases 3
      (rule and flag both off, no crushing), 13 and 14 (flag or rule on, crushing), 15 and 16 for the
      range in force and its absent-key default (F2), and 8 and 9 for the
      range condition under a computer-controlled owner. The omni-crusher variant: case 10, and 12h
      for its exception. The original's computer-controlled rules: cases 1, 2 and 4 (the table
      row), 17 (the neutral and special houses' skill-level condition), 5, 6, 9 and 11 (the attack-target variant, 11 for its move-destination condition) and
      12c (opportunity targets); case 7 shows that a human-controlled vehicle with an attack
      order does not get the variant. The departure itself is a design rule that no measurement of
      the original can confirm.
    - Confidence: medium for the conditions of the first list (the exact moment at which the choice
      is made belongs to the behaviour specs); medium for the meaning of the move-destination
      condition; low for the omni-crusher variant as a whole and for its opportunity-target
      exception; low for the original's attack-target variant

12. **Smaller rocking limit while slowed (presentation only).** While a vehicle is in the
    crush-slowdown state, whatever its locomotor, its forward rocking angle is limited to π/10 in
    either direction instead of the usual π/4. When the angle reaches the limit it stays at the
    limit and the body-tilt rate becomes level. The sideways rocking angle keeps its usual limit.
    This changes only how the vehicle is drawn. The forward tips of this spec (claims 6, 8 and 9) do
    not by themselves rock a vehicle as far as π/10, so the difference shows only when something else
    (for example an explosion that rocks vehicles, which the combat and rendering specs own) rocks it
    harder.
    - Backing: `measurable` (scenario K9; K10 part c for the walking-vehicle locomotor; K8 part a for
      the driving locomotor's tip staying below π/10)
    - Confidence: medium for the two limits, low for "the tips of this spec stay below π/10"

### Wall data and wall owners

13. **Damage-stage count and wall strength: where they come from, and their engine defaults.**
    An overlay type's damage-stage count *D* is read from the art data, from the section of the
    overlay type's image (S-0007), not from the overlay type's rules section. Its wall strength
    *S* is read from the overlay type's rules section. Both follow rules layering (S-0006 for the
    rules data, S-0007 for the art data): a layer without the key keeps the value in force. When
    no layer gives them, *D* = 1 and *S* = 1. These are the values the walls of claim 8, F3 and the
    other specs that read them (S-0209, S-0303) use.

    Consequences: a wall type left at *D* = 1 has only stage 0, so claim 8's first row never
    applies to it and crushing it hits no neighbour; and with *S* = 1 every wall hit of F3
    succeeds without a draw. A *D* written in the overlay's rules section has no effect.
    - Backing: `measurable` (scenario K11)
    - Confidence: high

14. **Owner of a wall.** Each wall cell has a wall owner: a house, or none.
    - (a) **Walls the map places.** Once the map's buildings have been placed and before any
      starting unit of a skirmish is created (S-0009), each cell that holds a wall overlay gets as
      its owner the owner of the nearest building that, at that moment, is on the map and not
      destroyed, and whose owner's country (house type) has the wall-owner flag (Import mapping).
      Distance is the 3-D distance of S-0204 from the building's reference point (the centre of
      its foundation, at the building's height) to the centre of the wall's cell at the terrain
      surface height there (S-0106), without the building reduction of S-0204, compared after
      truncation to whole leptons. On equal distances the building created earliest wins (for
      map-placed buildings, the order in which the map lists them). **There is no distance limit:** a building qualifies however far it is
      from the wall, so a wall far from every building is still owned by the nearest qualifying
      building anywhere on the map. A wall has no owner only when no building on the map
      qualifies (none is on the map, all are destroyed, or the owner of each lacks the wall-owner
      flag). This happens
      once per map load; later changes of building ownership, new buildings and destroyed
      buildings do not change the owner of a map-placed wall.

      This applies equally to walls from the map's overlay list and walls placed by wall
      buildings in its building list (S-0112 claim 7). A successfully placed wall building
      initially gives its wall the building's owner, but that initial owner is replaced by
      this nearest-building rule before play starts, even when the result is no owner.
      Neither a pre-existing owner nor origin in the building list exempts a wall. The
      converted wall building is not a surviving building and cannot itself qualify as the
      nearest building. This reassignment changes the owner, not the wall's damage stage
      or connection bits of S-0112 claim 7.
    - (b) **Consequences.** In a skirmish, the human player's own map-placed structures are not
      placed (S-0009 claim 27). This includes a wall building: dropping its entry places no
      wall, rather than placing a wall that is later reassigned. A wall already supplied
      independently by the overlay list is not removed by dropping that entry. Every player
      starts without buildings, so a map-placed wall belongs to
      a house of the map (for example a neutral or civilian house) when one of its buildings
      qualifies, and otherwise to no house. A wall without an owner is allied with no house, so
      S-0209's wall rows treat it as a wall of another house for every mover (the mover's view of
      S-0209 Terms, *allied*).
    - (c) A wall that loses its last damage stage, is sold or is removed has no owner from then
      on. Who owns a wall that a house builds during play belongs to the structures specs.
    - Backing: `measurable` (scenario K12, part c for building-list walls, reassignment and
      dropped human-owned wall buildings)
    - Confidence: high for building-list walls being included, replacement of their initial
      owner, and dropped human-owned wall buildings placing no wall; high for the rule, the absence of a distance limit and the time it applies; medium for the tie rule (the order of the
      map's building list), and for the cell height used (a difference of height between wall
      cell and building changes the result only when two buildings are almost equally near)

## Formulas

**F1. Horizontal crush distance (claim 3).** An object is close enough when
(*x*<sub>o</sub> − *x*<sub>c</sub>)² + (*y*<sub>o</sub> − *y*<sub>c</sub>)² < 16384, where
(*x*<sub>o</sub>, *y*<sub>o</sub>) is the object's reference point (S-0204 claim 4) and
(*x*<sub>c</sub>, *y*<sub>c</sub>) the crusher's position, in leptons. Exact integer arithmetic (no
root is taken). 16384 is 128² square leptons.

**F2. Crush range (claim 11).** The crush range is *R* = trunc(256 × *c*) leptons for an imported
real *c* in cells. Multiplying by 256 is exact in binary floating point, so *R* depends only on how
*c* is parsed, which S-0003 owns. A value of exactly −1 counts as no value: the range in force is
kept (from an earlier rules layer, or 384 leptons before any rules layer gives one). The distance is
*d* = trunc(√̃(Δ*x*² + Δ*y*² + Δ*z*²)), with the approximate square root √̃ of S-0208 F8 (the table square root of S-0204,
`quirks.table_root_distance`), taken in floating point and truncated. Automatic crushing requires *d* ≤ *R*; the original's attack-target
variant of claim 11 requires *d* < *R*. Both truncations follow the round-toward-zero conversion of S-0205
(number domains). Because √̃ can be one below the exact root above 129 leptons, a target
whose exact distance is slightly above *R* (by less than 2 leptons) can still count as in range; F8
decides each case exactly. When the squared distance is an exact power of 4, √̃ is exact (for
example 512 for a target two cells away along a map axis).

**F3. Wall hit on a neighbouring wall (claim 8).** Let *S* be the wall strength of the wall type.
- When *S* > 200, the hit of 200 succeeds exactly when a ranged draw *r* from the scenario random
  stream over 0 to *S* (inclusive; the ranged draw of S-0202 claim 4, which can take more than one
  raw value) gives *r* < 200, that is with probability 200 / (*S* + 1). One
  draw is taken for each neighbour that takes such a hit, in the order of the neighbour's direction
  from the crushed cell: map north, map east, map south, map west.
- When *S* ≤ 200, the hit always succeeds and no draw is taken.

Exact integer arithmetic, so no rounding applies. Units: wall-hit points.

**F4. Speed limit in the crush-slowdown state (claim 9).** On each tick on which the driving or ship
locomotor updates the speed fraction of an accelerating, non-passive vehicle in the crush-slowdown
state: *φ*ₜ′ = min(*φ*ₜ, 0.2) and *φ*′ = *φ*ₜ′, where *φ*ₜ is the target fraction and *φ* the speed
fraction of S-0207. The lowered target *φ*ₜ′ replaces the target fraction. Both are double-precision
values (S-0207 Terms), and 0.2 is the double nearest to 0.2. A minimum is exact, so no rounding
applies. Unit: fraction of full speed.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Crush radius | 128 (strict) | leptons, horizontal | F1, claim 3 |
| Crush range, absent-key default | 384 | leptons | 1.5 cells, claim 11 |
| Target-fraction cap in the crush-slowdown state | 0.2 | fraction of full speed | claim 9, F4; the speed fraction is set to the capped target; a double in the original |
| Train damage to an object | 10000 | points of health | claim 10 |
| Train self-damage per object | 20 | points of health | claim 10 |
| Body tilt set when a crusher crushes a vehicle while level, any locomotor | −0.05 | tilt units per tick (rendering) | claim 6; a single-precision real |
| Body tilt set when tipping forward on a crush-mode event, driving locomotor | −0.05 | tilt units per tick (rendering) | claims 8, 9; a single-precision real |
| Body tilt set when tipping forward, ship locomotor | −0.02 | tilt units per tick (rendering) | claim 9; a single-precision real |
| Body tilt set when tipping forward, walking-vehicle locomotor | −0.01 | tilt units per tick (rendering) | claim 9; a single-precision real |
| Body tilt added per overlay crushed | 0.02 | tilt units per tick (rendering) | claim 8; a single-precision real |
| Wall hit on undamaged neighbours | 200 | wall-hit points | claim 8, F3 |
| Damage-stage divisor of the overlay data value | 16 | overlay data value per stage | claim 8 |
| Damage-stage count, absent-key default | 1 | stages | claim 13 |
| Wall strength, absent-key default | 1 | wall-hit points | claim 13 |
| Wall-owner flag of a country, absent-key default | yes | boolean | claim 14 |
| Forward rocking angle limit in the crush-slowdown state | π/10 (otherwise π/4) | radians (rendering) | claim 12; single-precision reals |
| Auto-crush threshold, absent-key default | 2 | house skill level | claim 11, the original's computer-controlled houses only |
| Player-auto-crush rule, absent-key default | off | boolean | claim 11 |

## State and ordering

| State | Event | Result |
|---|---|---|
| crusher with the driving or ship locomotor | begins a track whose first cell holds infantry | cell-wide scatter request to that cell layer (claim 7) |
| crusher moving | reaches a cell centre | on that tick, an overlay that claim 8 lets it crush is crushed (a wall is removed) and every object of claim 2 in its cell layer whose reference point is less than 128 leptons away is removed, except that a vehicle thief coming for it takes it over instead (claim 5; ordering below) |
| crusher-all vehicle, driving locomotor, on the ground | drives into a new cell on a crush-mode track | crush-slowdown state on (claim 9) |
| crusher-all vehicle, driving locomotor, on a bridge deck | drives into a new deck cell on a crush-mode track while a vehicle stands in the ground layer under that deck cell | crush-slowdown state on (vehicles on the deck do not count) |
| crusher, ship locomotor | drives into a cell with the overlay type of index 0, on a track in the ship's crush mode | crush-slowdown state on |
| crusher, walking-vehicle locomotor | sets off on a step into a cell with the overlay type of index 0 | crush-slowdown state on |
| crush-slowdown state, accelerating non-passive type, driving or ship locomotor | its locomotor updates the speed fraction | target fraction lowered to at most 0.2, speed fraction set to it (F4) |
| crush-slowdown state | new order, stop, any other event | no change |
| crush-slowdown state, crusher | crushes an overlay or wall | state off |

Ordering stated as consequences:
- A driving harvester's track-completion arrival report has its result before cell-centre
  crushing on that tick (claim 3; S-0402 claim 10 and scenario H14). Boundary-triggered
  crush slowdown (claim 9) is a separate event that can have happened earlier on the track.
- On a tick on which a crusher reaches a cell centre that holds both a crushable overlay and
  crushable units, the overlay and the units are all crushed on that tick. The body tilt that claim 6 requires to be level includes the overlay's change of claim 8, so a crusher that is level just before and crushes a vehicle there pitches back, not forward (scenario K8 part c). The draws of F3 for the neighbouring walls are taken from the scenario random stream on that
  same tick, in the direction order of F3; the random-state recorder shows them (K4 part c). This
  spec states no order between them and any draws that the removal of the units may take (Open
  questions).
- When a vehicle thief takes the crusher over at a cell centre (claim 5), the house that each other
  object of that cell layer is judged against follows the cell layer's listing order (S-0209 claim
  13: the non-building that entered the cell layer most recently first, buildings last). An object
  listed before the thief is judged against the crusher's original house, and one listed after it
  against the thief's house. So a unit of the thief's house that entered the cell before the thief
  survives, and one that entered after the thief is crushed (scenario K3). The listing order cannot
  be seen in the per-tick record of removals or heard in the crush sounds: all the objects are
  removed, and their sounds start, on the same tick.
- A crushed unit is gone before the end of the tick, so it cannot fire later in that tick.

## Edge cases and quirks

- **Omni-crusher alone crushes nothing.** A type with the omni-crusher flag but without the crusher
  flag (and without the crusher veterancy ability) never crushes (claim 1), although claim 2 would let
  it crush vehicles, and claim 11 still lets it drive at a target it could crush. This is the
  original behaviour. The switch `quirks.omnicrusher_needs_crusher` (default: the original) selects
  it. With it off, the omni-crusher flag also makes a vehicle a crusher.
- **The slowing vehicle counts itself, and the ground layer is always looked at.** With the driving
  locomotor, the condition of claim 9 that the cell driven into holds a vehicle is met by the vehicle
  itself on the ground, so there every cell entry on a crush-mode track sets the crush-slowdown
  state, also with no victim: the cell before a vehicle, a wall or a crushable overlay in the path,
  and the wall cell itself. And only vehicles in the ground layer of that cell count, even when
  the driving vehicle is on a bridge deck. Two independent switches select this, each with default the
  original:
  - `quirks.crush_slowdown_counts_self`. With it off, the driving vehicle itself never counts; only
    another vehicle does.
  - `quirks.crush_slowdown_ground_layer`. With it off, the cell layer looked at is the one the
    driving vehicle is in after driving into the cell: the bridge-deck layer for a vehicle on a
    deck, the ground layer otherwise.

  The four combinations, for a vehicle driving into a cell on a crush-mode track:

  | counts self | ground layer | Driving on the ground (also under a bridge) | Driving on a bridge deck |
  |---|---|---|---|
  | on (original) | on (original) | always | when any vehicle stands in the ground layer under that deck cell |
  | off | on | when another vehicle stands in the ground layer of that cell | as in the original, since the vehicle is not in the ground layer |
  | on | off | always | always |
  | off | off | when another vehicle stands in the ground layer of that cell | when another vehicle stands in the bridge-deck layer of that cell |

  In every combination, a vehicle driving on the ground under a bridge never counts vehicles on the
  deck above it, and a vehicle on a deck is affected by vehicles in the ground layer under it only
  while `quirks.crush_slowdown_ground_layer` is on.
- **A crushable overlay that is not a wall stays.** Crushing it (claim 8) plays its crush sound, tilts
  the crusher and ends the crush-slowdown state, but leaves the overlay in its cell unchanged, so the
  next crusher to reach the cell crushes it again in the same way. This is the original behaviour.
  The switch `quirks.crushable_overlay_stays` (default: the original) selects it. With it off, the
  overlay is removed from the cell at once, as a wall is, with no hit on neighbouring cells; the
  sound, the tilt and the end of the state are unchanged. Which cells can be entered and how they connect are then
  updated as for any destroyed overlay, which S-0211 and S-0212 own.
- **Neighbour hits only at stage *D* − 2.** A crushed wall hits its undamaged neighbours only when it
  is of a type with *D* > 2 and was at damage stage *D* − 2 (claim 8). This is the original behaviour and not a defect, and has no
  switch.
- **Persistent slowdown.** The crush-slowdown state of claim 9 is ended only by crushing an overlay,
  so an accelerating crusher-all vehicle stays at no more than 0.2 of its speed, across new orders,
  until it next crushes a wall or a crushable overlay. A crusher-all vehicle without the crusher flag
  or ability cannot crush overlays, so it stays slowed until it gains the crusher ability, if ever,
  and then crushes an overlay. The switch
  `quirks.crush_slowdown_persists` (default: the original) selects this. With it off, the state also
  ends when the vehicle next reaches a cell centre where the cell layer it is in holds no vehicle
  other than itself (whatever the other two switches select), and the speed is no longer limited
  from then on. It is entered again by the next event of claim 9.
- **Overlay of registration index 0.** With the ship and walking-vehicle locomotors the crush-slowdown
  state is set by the overlay type that happens to be first in the overlay registry (claim 9), not by
  a property of the overlay. The switch `quirks.crush_slowdown_first_overlay` (default: the original)
  selects this. With it off, the ship locomotor sets the state when the vehicle drives into a cell
  holding a wall overlay on a track in the ship locomotor's crush mode (the crush-mode condition of
  claim 9 is unchanged), and the walking-vehicle locomotor, which has no crush mode, sets it when
  the vehicle sets off on a step into a cell holding a wall overlay. Both still require the crusher
  flag or ability, as with the overlay of index 0.
- **A train that is not a crusher spares crushable objects.** Claim 10's test ignores the crusher
  flag, so a train without it neither damages nor crushes an enemy object that a crusher could crush.
  This is original behaviour, not a defect, and has no switch.
- **Thief exception.** Claim 5 is original behaviour, not a defect, and has no switch. The change of
  house already applying to the objects listed after the thief follows from it and has no switch
  either.
- **AI houses and the original's computer-controlled rules (departure).** The computer-controlled
  row of the table, the attack-target variant and the computer-controlled exemption from the
  opportunity-target exception (claim 11) describe the original only, except for units of the
  neutral and special houses, which keep them. In our engine every player house, AI houses
  included, follows the human-controlled rules. This is a departure with no switch, as the glossary
  entry *AI house* states for every spec (with its *Player houses only* exception).
- **Rounding.** F2 follows the round-toward-zero mode of S-0205 (number domains), switch
  `quirks.chop_rounding`.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, unit type sections, `Crusher` | crusher flag | boolean | no |
| Rules, unit type sections, `VeteranAbilities` / `EliteAbilities` containing `CRUSHER` | crusher veterancy ability | list | none |
| Rules, type sections, `OmniCrusher` | omni-crusher | boolean | no |
| Rules, type sections, `OmniCrushResistant` | omni-crush resistant | boolean | no |
| Rules, type sections, `Crushable` | crushable flag | boolean | owned by the type specs |
| Rules, infantry type sections, `DeployedCrushable` | whether deploying keeps the unit crushable (S-0107, State and ordering, deploy and undeploy rows) | boolean | yes |
| Rules, type sections, `CrushSound` | crush sound of the crushed type or overlay | sound name | none |
| Rules, unit type sections, `TiltsWhenCrushes` | tilt-on-crush flag | boolean | owned by the vehicle specs |
| Rules, unit type sections, `MovementZone` with the value `CrusherAll` | crusher-all movement zone | name | owned by the movement specs |
| Rules, unit type sections, `Locomotor` | driving, ship or walking-vehicle locomotor | class identifier | owned by the movement specs |
| Rules, unit type sections, `IsTrain` | train | boolean | no |
| Rules, infantry type sections, `VehicleThief` | vehicle thief | boolean | no |
| Rules, overlay sections, `Crushable`, `Wall` | crushable overlay, wall overlay | boolean | no |
| Art data, the section of the overlay type's image (S-0007), `DamageLevels` | damage-stage count *D* (claims 8 and 13) | integer (S-0003, claim 3) | keeps the value in force; engine default 1. The same key in the rules overlay section is not read |
| Rules, overlay sections, `Strength` | wall strength *S* (F3, claim 13) | integer | keeps the value in force; engine default 1 |
| Rules, country (house type) sections, `WallOwner` | wall-owner flag of the country (claim 14) | boolean | keeps the value in force; engine default yes. Not tied to any distance: the flag only decides whether the country's buildings qualify (claim 14 (a)); it does not exempt an already owned wall from reassignment |
| Map, `[Structures]` entries, owner and type fields | wall-building origin and initial owner (claim 14) | comma-separated records; owner resolution and dropped entries: S-0009 claim 27 | no entries means no walls from this list; surviving entries do not preserve their initial wall owner against claim 14 (a) |
| Rules, building type sections, `Wall`, `ToOverlay` | whether a building places a wall, and its overlay type | boolean and overlay type name, respectively (S-0112 claim 7) | no and none, respectively |
| Rules, the combat-damage section, `Crush` | crush range *c* (F2) | real, cells; exactly −1 counts as absent | keeps the value in force; before any rules layer gives it, 384 leptons |
| Rules, the combat-damage section, `PlayerAutoCrush` | player-auto-crush rule | boolean | no |
| Rules, unit type sections, `AutoCrush` | auto-crush flag | boolean | no |
| Rules, `[IQ]`, `AutoCrush` | auto-crush threshold of the house skill level (the original's computer-controlled houses only, claim 11) | integer | 2 |
| Rules, type sections, `OpportunityFire` | opportunity-fire flag, which lets a moving unit pick opportunity targets (S-0215; used by scenario K7 cases 11 and 12) | boolean | owned by S-0215 |
| Rules, unit type sections, `Accelerates` | accelerating type (claim 9) | boolean | owned by S-0207 |
| Rules, the combat-damage section, `C4Warhead` | demolition warhead used by trains | warhead name | owned by the combat specs |

## Measurement scenarios

All run under the *measurement setup* on purpose-made flat maps of clear terrain, with one human
player and one bystander computer player (glossary), unless stated. The computer house
plays at the hardest difficulty unless stated. A *recorder* reads object lists, health, positions,
owners and overlay data values per tick. Where a scenario needs a type property that no stock type
has, it is set through a named test map's override section, and the scenario says so. "The overlay
type of index 0" means the overlay type with registration index 0 in the data in force (S-0004).

**Scenario K1: who is crushed, and how (claims 1, 2, 4).**
- *Setup:* a stock crusher vehicle of the human player. Each victim stands alone in a cell of a
  one-cell-wide corridor on its route, so that infantry cannot scatter away:
  - (a) an enemy infantry of a crushable type,
  - (b) an allied infantry of the same type (an allied computer house),
  - (c) an enemy infantry of a type that is not crushable,
  - (d) the same route as (a), driven by a stock vehicle that is not a crusher, and by a vehicle type
    made omni-crusher without the crusher flag through a named test map's override section,
  - (e) the same route as (a), driven by two units of an armed stock vehicle type that is not a
    crusher, given the crusher ability at veteran level through the override section. Before the
    run, one of them is promoted to veteran by letting it destroy enemy targets placed for the
    purpose until its rank insignia shows veteran (how rank is earned belongs to the veterancy
    specs; the run needs only the rank, read from the insignia); the other stays a rookie,
  - (f) a crusher of a type that also has the omni-crusher flag (override if needed), and a stationary
    enemy vehicle of a type without the crushable flag and not omni-crush resistant,
  - (g) as (f), with the victim's type made omni-crush resistant through the override section,
  - (h) as (f), with the victim made invulnerable by a map trigger (the superweapon specs own how)
    shortly before the crusher reaches it,
  - (i) two enemy buildings made crushable through the override section, one with a foundation of one
    cell and one with a foundation of two by two cells, on the crusher's route,
  - (j) an enemy infantry of a type that is uncrushable while deployed (stock if one is, otherwise set
    through the override section), deployed; and a second one of the same type, not deployed,
  - (k) an enemy infantry of a stock type that can take mind control, its type given the crushable
    flag through the override section if it lacks it, standing in the corridor. A map
    trigger has it take control of one of the human player's vehicles before the crusher sets off.
  - (l) the same route as (a), walked instead by an infantry unit of the human player of a stock
    type; it and the victim are unarmed (override if needed), so that neither fires.
- *Procedure:* order the crusher (or the test vehicle, or in (l) the infantry) to a cell beyond the
  victim and let it drive (in (l), walk) through each cell's centre. In (i), order it through each building's
  cells. In (c) to (e), (g), (h) and (i) a run in which the crusher does not drive into the
  victim's cell (because S-0209 refuses entry, or it never takes the route) decides nothing for that
  victim, and in (l) a run in which the infantry does not walk into the victim's cell decides
  nothing.
- *Observable:* which victims disappear, their health on the last tick before they disappear, the
  presence of a death animation or corpse, the crusher house's kill count, in (a) and (c) whether a
  crush sound starts on the tick the victim disappears (screen and audio capture; the victim type of
  (a) is given a crush sound through the override section if it has none in the data in force), and
  in (k) the owner of the controlled vehicle.
- *Runs:* 1 each; (l) up to 5, stopping at the first run in which the infantry walks into the
  victim's cell.
- *Expected:*
  - (a) is crushed: its health is unchanged up to the tick it disappears, no death sequence or corpse
    appears, the kill counts for the crusher's house, and its type's crush sound starts on that
    tick. (b) and (c) are not crushed (the crusher stops or waits at (b), S-0209), and no crush
    sound plays as it passes (c). In (d) nothing is crushed by either vehicle.
  - (e) the veteran crushes the infantry, the rookie does not.
  - (f) the vehicle is crushed. (g) and (h) it is not.
  - (i) the one-cell building is crushed when the crusher reaches its cell's centre; the two-by-two
    building is not crushed in any of its cells.
  - (j) the deployed infantry is not crushed, the undeployed one is.
  - (k) the controller is crushed and the controlled vehicle returns to the human player on the same
    tick.
  - (l) the victim is not removed.
- *Falsified if:* (b) or (c) is crushed, (a) loses health first, leaves a corpse or is removed with
  no crush sound, a vehicle in (d)
  crushes, the veteran in (e) does not crush or the rookie does, (f) survives or (g) or (h) is crushed,
  the one-cell building in (i) survives or the two-by-two one is crushed, (j) shows a different
  pattern, in (k) the vehicle stays with the enemy, or the victim in (l) disappears.

**Scenario K2: the moment of crushing, the radius, the warning and falling units (claims 2, 3, 7).**
- *Setup:*
  - (a) a computer-owned crushable infantry unit standing at the right standing position (S-0208
    table P) of a cell in a one-cell-wide corridor, so that it has nowhere to scatter to. A stock
    crusher of the human player drives along the corridor through that cell and stops two cells
    beyond it.
  - (b) the same in open ground, with the computer-owned infantry idle and free to move.
  - (c) open ground. A stock crusher of the human player drives in the direction of increasing *y*
    (cell coordinates) along a straight row of cells and passes through the centre of cell *X*.
    *W* is the neighbour of *X* in the direction of decreasing *x*. A computer-owned crushable
    infantry of a type that is not a fraidycat, unarmed (override if needed) so that it does not
    fire, stands at the right standing
    position (S-0208 table P) of *W*. A map trigger orders it to walk to the right standing position
    of *X* (S-0208 claim 3 keeps that position), so that it walks along a line 64 leptons from *X*'s
    centre and is about 143 leptons from *X*'s centre when it crosses into *X* and about 91 when it
    arrives. The trigger's tick is stepped by one tick from run to run, across the ticks around the
    crusher's arrival, so that the infantry sets off before the crusher drives into *X*.
  - (d) the corridor of (a), with *X* the cell in which the infantry of (a) would stand. Instead of
    standing there, a computer-owned crushable infantry unit is dropped from the air into *X* by a
    paradrop that a map trigger starts (the superweapon specs own paradrops), so that it is falling
    for many ticks before it lands, and a stock crusher of the human player drives along the
    corridor through *X*. The trigger's tick is stepped by a few ticks from run to run, as in (c),
    so that in some runs the crusher reaches *X*'s centre while the unit is still falling and in
    others after it has landed. The recorder also reads the height of the unit's position above the
    ground.
- *Procedure:* record per tick the crusher's position and the infantry's presence and position (in
  (d) also its height above the ground, which shows on which ticks it is falling).
- *Observable:* (a) the tick on which the infantry disappears and the crusher's position then. (b)
  whether the infantry leaves the cell before the crusher arrives, and the tick on which it starts to
  move relative to the tick on which the crusher sets off from the centre of the cell before it. (c)
  on the tick on which the crusher's position equals *X*'s centre: whether the infantry's position is
  inside *X*, its horizontal distance from *X*'s centre, and whether it disappears on that tick. (d)
  on the tick on which the crusher's position equals *X*'s centre: whether the unit is still above
  the ground, and whether it disappears on that tick.
- *Runs:* (a) 3. (b) 10. (d) 10, stepping the trigger tick until at least three runs have the
  unit falling when the crusher reaches the centre and at least three have it landed and standing
  inside *X* at less than 128 leptons from the centre; other runs decide nothing. (c) 20, discarding runs in which the infantry was not walking into *X* at
  that tick; if no kept run has the infantry inside *X* at a distance between 100 and 143 leptons,
  add runs until at least four do, two on each side of 128.
- *Expected:* (a) it disappears on the tick on which the crusher's position equals the cell centre,
  not earlier, even though the crusher's body overlaps the infantry before. (b) it starts to move
  within a few ticks after the crusher begins the track into its cell (the request comes when the
  track begins), and it reaches a neighbouring cell before the crusher reaches the centre in most
  runs. (c) an infantry inside *X* at less than 128 leptons from the centre disappears on that tick,
  one at 128 leptons or more survives, and is not crushed later as it walks on to its standing
  position, because the crusher has left the centre; one still in *W* survives. (d) a unit that is still falling when the crusher reaches *X*'s centre
  survives that tick, and a unit that has landed inside *X* at less than 128 leptons from the centre
  disappears on it (the control).
- *Falsified if:* (a) it disappears on a tick on which the crusher is not at the cell centre. (b) it
  never moves, or it starts to move before the crusher begins that track. (c) an infantry inside *X*
  at 128 leptons or more disappears, one inside *X* at less than 128 survives that tick, or one in *W*
  disappears. (d) a unit that is still falling disappears when the crusher reaches the centre, or
  the landed control survives.

**Arrival-report ordering (claim 3).** Use S-0402 scenario H14's driving-harvester setup,
procedure, recorder and runs, including its crushable-overlay arrival-cell variant. The
observable is the queued behaviour relative to the overlay's crush sound and body-tilt change
on the arrival tick. Expected: the arrival report's result, including queued *unload* for an
accepted report, is already present when the overlay is crushed. Crushing before that report's
result falsifies the ordering. A run that cannot establish the relative order decides nothing.
Boundary-triggered slowdown, if present, may begin before arrival and does not count as
cell-centre crushing.

**Scenario K3: the vehicle thief (claim 5, listing order).**
- *Setup:* a straight corridor one cell wide, closed at one end. Counting from the closed end, the
  cells are 1, 2, 3 and so on; cell 3 is the *meeting cell*. A crusher vehicle of the human player
  drives into the corridor from the open end towards cell 3. An enemy computer-owned infantry of a
  vehicle-thief type, crushable by the crusher under claim 2 (its type given the crushable flag
  through the test map's override section if it lacks it), starts in cell 2 and is ordered (by a map trigger) to steal that crusher when it
  is four cells from the meeting cell, so that it walks into the meeting cell before the crusher
  reaches its centre. (a) No other infantry. (b) A second computer-owned crushable infantry of the
  thief's house, the *companion*, starts in the meeting cell, so that it entered the cell before the
  thief. (c) The companion starts in cell 1 and is ordered by a trigger into the meeting cell after
  the thief has entered it.
- *Procedure:* let the crusher reach the centre of the meeting cell.
- *Observable:* whether the thief is crushed or the vehicle changes owner, whether the companion
  disappears, and the tick on which each infantry entered the meeting cell.
- *Runs:* 5 each. Runs in which the infantry did not enter in the intended order, or in which the
  companion left the meeting cell (for example after the crusher's warning, claim 7) before the
  crusher reached its centre, are discarded and repeated.
- *Expected:* (a) the vehicle changes owner. (b) the vehicle changes owner and the companion
  survives. (c) the companion is crushed and the vehicle changes owner, on the same tick.
- *Falsified if:* the thief is crushed, or (b) and (c) do not differ in the companion's fate as
  stated.

**Scenario K4: walls, fences and tilt (claim 8).**
- *Setup:* *D* and *S* are the damage-stage count and wall strength of the wall type used, read from
  the data in force. The wall type must have *D* > 2; if no stock wall type has, set it through the
  test map's override section. *Stage* means ⌊*v* / 16⌋ of the recorded overlay data value *v*
  (claim 8); the remainder below 16 may change through wall linking and is not judged. (a) A stock
  crusher with the tilt-on-crush flag ordered across a row of undamaged fence overlay cells (a
  crushable wall type). (b) A vehicle with the crusher flag and the crusher-all movement zone (a stock
  type if one has both, otherwise set through the test map's override section) ordered through a row
  of undamaged wall cells. (c) As (a), but the crossed fence cells start at damage
  stage *D* − 2 (overlay data value 16 × (*D* − 2) plus any remainder below 16, set in the test map's
  overlay data, S-0009 claim 13), with undamaged fence cells of the same type beside them. (d) As (c),
  but the crossed cells start at stage *D* − 1. The fence and wall types are given a crush sound
  through the override section if they have none in the data in force.
- *Procedure:* record per tick the overlay type and overlay data value of each crossed cell and of its
  neighbours, and take screenshots with an audio capture. In (c) the random-state recorder also reads
  the scenario random stream's state on every tick. A second set of (c) runs, (c2), uses a wall
  type with *S* ≤ 200 (set through the override section if no stock type has it). Nothing else on
  the map is meant to draw from the scenario random stream: no other unit is near.
- *Observable:* when each crossed overlay disappears, whether each neighbour's stage changes or its
  wall disappears, the direction in which the body pitches in the screenshots, and whether the
  overlay type's crush sound starts on the tick each crossed overlay disappears.
- *Runs:* (a), (b), (d) 1 each; (c) 20, for the share of neighbours hit and the draws; (c2) 3.
- *Expected:* (a), (b), (d) each crossed overlay is removed completely on the tick on which the
  crusher reaches its cell centre, whatever its stage, its crush sound starts on that tick, and the
  undamaged neighbours stay at stage 0.
  In (a) the body pitches forward as it drives into each fence cell (tilt-on-crush, claim 8) and is
  pushed back on each crush. In (b), if its type lacks the tilt-on-crush flag, the body only pitches
  back on each crush. (c) the crossed cells are removed as in (a); on that tick each undamaged
  neighbour of the same type either is hit (its stage changes, or its wall is removed; what a hit does
  is for the overlay specs) or keeps stage 0, the share hit being about 200 / (*S* + 1), or all of
  them when *S* ≤ 200. With the recorder, the raw values taken on the crush tick correspond, under the inclusive ranged-draw rule of S-0202 claim 4, to one draw from the integers 0 through *S* for each undamaged neighbour, in the order north, east, south, west. A neighbour is hit exactly when its draw is below 200. In (c2) no raw value
  is taken on that tick and every neighbour is hit.
- *Falsified if:* a crushed wall or fence survives the crusher's passage at a lower stage, is removed
  with no crush sound, or changes at a moment other than the tick on which the crusher reaches its
  cell centre; a neighbour's stage changes in (a), (b) or (d); or in (c) no neighbour is ever hit,
  every neighbour is hit when *S* > 200 makes that unlikely (*S* ≥ 400), or a neighbour changes on
  any other tick; or in (c) the recorded draws differ in number from the undamaged neighbours or
  their outcomes do not follow the draws in the order north, east, south, west; or in (c2) a raw value is taken or a neighbour is not hit.

**Scenario K5: the crush-slowdown state, driving locomotor (claim 9).**
- *Setup:* a straight route of 30 clear cells. The test vehicle is of an accelerating type with the
  driving locomotor, the crusher flag, the omni-crusher flag and the crusher-all movement zone, owned
  by the human player (a stock type if one has all of these, otherwise set through the test map's
  override section). The enemy vehicle used is of a type the test vehicle can crush and is kept
  stationary (the vehicle is given no orders). Runs:
  - (a) an enemy vehicle in cell 5 and a wall section in cell 25. Order the test vehicle to cell 20.
    When it has stopped, order it to cell 30.
  - (b) no vehicle, only the wall in cell 25. Order the test vehicle to cell 30.
  - (c) as (b), but the test vehicle's type has no crusher flag, and cell 25 holds a crushable
    overlay that is not a wall overlay (made crushable through the test map's override section)
    instead of the wall.
  - (d) no vehicle and no overlay, as the full-speed reference. Order the test vehicle to cell 30.
  - (e) as (a) up to the stop at cell 20, with the test vehicle's type given the tilt-on-crush flag
    (override if needed). Screenshots are taken on every tick from cell 2 to cell 7.
- *Procedure:* record per tick the test vehicle's position.
- *Observable:* its per-tick displacement, and the cell boundary or cell centre at which it changes.
- *Runs:* 2 each.
- *Expected:* the limit shows as a displacement no greater than that of a speed fraction of 0.2,
  compared with run (d) cell by cell, reached on the tick the state begins rather than after a
  gradual slowdown. The target fraction of S-0207 claim 9 for the cell being entered also applies,
  so on the wall cell compare with that cell's own target fraction. After the stop in (a), the new
  order starts at no more than 0.2 at once, without a ramp from rest.
  - (a) the limit begins when the vehicle drives into cell 4, the cell before the enemy vehicle,
    and stays through the stop at cell 20 and the new order, up to the centre of cell 25. After
    crushing the wall there it accelerates as in run (d).
  - (b) the limit begins when it drives into cell 24 and ends at the centre of cell 25.
  - (c) the limit begins when it drives into cell 24 and never ends.
  - (e) the body starts to pitch forward on the tick on which the vehicle drives into cell 4, and
    the pitch is renewed on each of the next cell entries (cells 5 to 7), although the vehicle is
    already in the state.
- *Falsified if:* in (a) the limit begins only on driving into cell 5, or never (as when only another
  vehicle counts, or only a second vehicle beyond the first sets crush mode); in (b) there is no
  limit before the wall; in (a) or (b) the speed recovers before cell 25 without an overlay being
  crushed, or stays limited after the wall is crushed; in (c) the speed recovers; the speed falls
  to the limit gradually over several ticks instead of at once; or in (e) the body does not pitch
  forward on driving into cell 4, or the pitch is not renewed on the entries into cells 5 to 7.

**Scenario K6: trains (claim 10).**
- *Setup:* a map with a railway and a stock train type. Runs:
  - (a) an enemy non-crushable unit (for example a vehicle) standing on the track,
  - (b) a non-crushable vehicle of the train's own house standing on the track,
  - (c) an enemy transport vehicle (non-crushable) loaded with infantry, standing on the track,
  - (d) a train type without the crusher flag (stock if one has none, otherwise set through the
    override section), and an enemy infantry of a crushable type standing idle on the track; control:
    the same with an infantry of a type that is not crushable.
- *Procedure:* let the train drive into the victim's cell. A run in which the train does not drive
  into that cell (for example because it stops in front of it) decides nothing.
- *Observable:* the health of the victim and of the train before and after, who is credited with a
  kill, whether infantry leave the transport, and whether the infantry in (d) loses health or is removed.
- *Runs:* 1 each; (d) 3.
- *Expected:* (a), (b), (c) the victim loses 10000 health (so it is destroyed unless its health is
  higher), the train loses exactly 20 per victim, both without scaling by armour, and a destroyed
  victim's kill is credited to no house. (c) no passengers come out. (d) the crushable infantry is
  neither damaged nor removed; in the control, the infantry loses 10000 health.
- *Falsified if:* other damage amounts, a kill credited to the train's house, the own vehicle in (b)
  unharmed, passengers escaping in (c), or the crushable infantry in (d) damaged or removed.

**Scenario K7: automatic crushing (claim 11).**
- *Setup:* a stock crusher type without the omni-crusher flag (unless a case says otherwise), and a
  crushable infantry of the other player standing idle. Cases with a computer-owned crusher measure
  the original's computer-controlled rules of claim 11; our engine does not apply them to AI houses,
  and the human-owned cases (3, 7, 10, 12h, 13 to 16) measure the rules our engine applies to every
  player house. Where a
  case says *unarmed*, the crusher's weapons are removed through the test map's override section, so
  that it never gets a target to attack and only the first list of claim 11 can move it. Where it
  says *armed*, the stock weapons are kept and the victim's health is raised through the override
  section so that the crusher's weapon cannot kill it within the run. Every case starts from a fresh
  program start: the crush-range key and the player-auto-crush rule are game-wide keys, so a value
  that an earlier case's override section gave them would otherwise outlast that game (glossary,
  *rules database*). Cases:
  - (1) unarmed crusher, computer-owned, the infantry at half the crush range in force from it, the
    computer house at the hardest difficulty in the lobby,
  - (2) as (1), the infantry at twice the crush range,
  - (3) unarmed crusher, human-owned, the player-auto-crush rule off, the crusher's type without the
    auto-crush flag (set through the override section if the stock type has it), the infantry
    computer-owned at half the crush range, the computer house at the hardest difficulty,
  - (4) as (1), with the computer house at the easiest difficulty in the lobby,
  - (5) armed crusher, computer-owned, the computer house at the easiest difficulty (so the first
    list cannot apply), the infantry at half the crush range and within weapon range,
  - (6) as (5), the infantry at one and a half times the crush range, within weapon range,
  - (7) armed crusher, human-owned, its type given the auto-crush flag through the override section,
    the player-auto-crush rule off, ordered by the human player to attack the infantry at half the crush
    range,
  - (8) as (1), with the crush range set to 2.0 cells through the override section and the infantry
    at the centre standing position of the cell two cells from the crusher's cell along a map axis,
    the crusher at its cell centre, both on flat ground, so that *d* = *R* = 512 (F2),
  - (9) as (5), with the crush range and placement of (8),
  - (10) armed crusher of a type that also has the omni-crusher flag (override if needed),
    human-owned, ordered by the human player to attack a stationary enemy vehicle of a type it can crush
    under claim 2 (not crushable by flag, not omni-crush resistant), four cells away and within
    weapon range,
  - (11) as (5), with the crusher's type given the opportunity-fire flag through the override
    section, the crush range set to 3.0 cells through the override section, and the infantry at one
    cell from a straight route of 12 cells that does not pass
    through the infantry's cell and ends 8 cells beyond it. A map trigger gives the crusher a move
    order to the end of the route (S-0011),
  - (12) a stationary enemy vehicle as in (10), one cell beside a straight route of 12 cells that
    ends 8 cells beyond it, and an armed vehicle of a type with the omni-crusher flag, the crusher
    flag and the opportunity-fire flag (override as needed) at the start of the route. Run 12h: the
    vehicle is human-owned, the player-auto-crush rule is off and its type is without the auto-crush
    flag (set through the override section if the stock type has it), and the human player orders it
    to move to the end of the route. Run 12c: the vehicle is computer-owned, the computer house at the
    easiest difficulty (so the first list cannot apply), and a map trigger gives it the same move
    order,
  - (13) as (3), with the crusher's type given the auto-crush flag through the override section,
  - (14) as (3), with the player-auto-crush rule on through the override section,
  - (15) as (14), with the crush-range key set to −1 through the override section (F2), and the
    infantry placed once at 0.9 and once at 1.1 times the range in force from the rules layers
    below the override,
  - (16) as (14), with a named test rules file in which no layer gives the crush-range key (F2), and
    the infantry placed once at about 1.4 cells and once at about 1.6 cells from the crusher,
  - (17) unarmed crusher without the omni-crusher flag, owned in separate runs by the neutral
    and special houses, with their countries made non-passive through the test map's override
    section. Place hostile crushable infantry at half the crush range. Use named test rules
    that set the auto-crush threshold first to 2, then to 0. The houses' alliances must make the
    infantry hostile from the crusher's side. Observe only runs that give the crusher an occasion
    to pick a target of its own under S-0215; otherwise the run decides nothing.
- *Procedure:* wait 600 ticks, or give the order and wait 600 ticks.
- *Observable:* whether the crusher drives to the victim's cell and crushes it, and whether it fires,
  and in cases 11 and 12 also whether it leaves the route before reaching its end. In cases 5,
  10 and 12c the recorder also reads whether the destination names the victim object or a cell.
- *Runs:* 10 per case (per run in case 12; per house and threshold in case 17). In cases 5, 6 and 9 a run in which the crusher neither
  fires nor moves (it never took the infantry as a target) decides nothing. In cases 11 and 12 a run
  in which it does not fire at the victim while on the route (it never took the victim as an
  opportunity target) decides nothing. Run 12c is the positive control for case 11: case 11
  decides the move-destination condition only when run 12c leaves the route, which shows that the
  choice of claim 11 is offered while moving; otherwise case 11 decides nothing.
- *Expected:* it drives over the victim in cases 1, 5, 8, 10, 13 and 14, in case 15 at 0.9 and in
  case 16 at about 1.4 cells (and not at 1.1 or at about 1.6), and in run 12c (it leaves the route
  for the vehicle's cell although it has a move destination); it does not in cases 2 (beyond the
  range), 3, 4, 6 (beyond the range), 7 (the variant is for computer-controlled houses only, and the
  attack order is not an occasion on which it picks a target of its own), 9 (*d* is not less than *R*), 11 (it has a move destination) and
  run 12h (a human-controlled vehicle with an opportunity target). In cases 6, 7, 9, 11 and run 12h it
  fires at the victim instead, and in 11 and 12h it keeps to the route. In case 17 neither house's
  crusher chooses to drive over the victim with threshold 2; each does with threshold 0.
  In cases 5, 10 and 12c the crushing approach sets the destination to the victim object itself,
  including when that victim is already within weapon range; the common approach's in-range
  rule does not prevent this destination.
- *Falsified if:* in those variant cases the destination is a fixed cell rather than the victim
  object, or no destination is set solely because the victim is within weapon range; or a
  different pattern, including case 15 behaving as if the range were zero or
  unbounded, or case 16 using any other range than 1.5 cells; in particular, the crusher leaving the
  route for the victim in case 11 or run 12h, or keeping to the route while firing at the vehicle in run 12c.

**Scenario K8: tip on crushing a vehicle (claim 6).**
- *Setup:* a crusher of the human player with the driving locomotor and the omni-crusher flag, without the crusher-all movement
  zone and without the tilt-on-crush flag, so that no other rule tips it (set through the test map's
  override section if needed), on flat clear ground. (a) A stationary enemy vehicle of a type it can
  crush on its route. (b) Instead an enemy crushable infantry, in a one-cell-wide corridor as in
  K2 (a), so that it cannot scatter. (c) As (a), but the victim's cell also
  holds a crushable overlay that is not a wall overlay (made crushable through the test map's
  override section).
- *Procedure:* order the crusher through the victim's cell. Take a screenshot on every tick from 5
  ticks before to 30 ticks after the tick on which the victim disappears.
- *Observable:* whether and in which direction the drawn body pitches after that tick, in (a) its
  largest forward pitch, and in (c) whether the overlay is still in its cell after the crusher has
  passed.
- *Runs:* 2 each.
- *Expected:* (a) the body starts to pitch forward on that tick and then returns to level; its
  largest forward pitch stays below about 18 degrees (π/10, claim 12). (b) the body does not pitch.
  (c) the body pitches back, not forward, and the overlay stays in its cell (claim 8).
- *Falsified if:* (a) no forward pitch, or a forward pitch of clearly 18 degrees or more, (b) any
  pitch, (c) a forward pitch, or the overlay gone from its cell.

**Scenario K9: rocking limit while slowed (claim 12).**
- *Setup:* two identical vehicles of the human player, of an accelerating crusher-all crusher type
  with the driving locomotor and the omni-crusher flag, standing still side by side on flat clear
  ground and facing the same way. Vehicle A has entered the crush-slowdown state (it has driven up to
  an enemy vehicle and crushed it, and crushed no overlay since, as in K5 run a). Vehicle B is placed
  in its spot by the map and never moves, so it never drives a crush-mode track and is not in the
  state. An explosion of a warhead that rocks vehicles (the combat specs own which do) is set off by a
  map trigger at equal distance directly ahead of both.
- *Procedure:* take a screenshot on every tick for 30 ticks after the explosion.
- *Observable:* the largest forward or backward pitch of each body, measured from the screenshots.
- *Runs:* 3, with the explosion strong enough that B pitches visibly more than 18 degrees.
- *Expected:* A's pitch stops at about 18 degrees (π/10); B's goes beyond it, up to 45 degrees (π/4).
  Runs in which B stays below 18 degrees decide nothing.
- *Falsified if:* A pitches clearly beyond 18 degrees, or A and B pitch alike while B goes beyond 18
  degrees.

**Scenario K10: other locomotors and bridge decks (claims 3, 7, 9, 12).**
- *Setup:*
  - (a) *Ship warning.* Open water. A vehicle type with the ship locomotor, given the crusher flag
    through the override section, owned by the human player. An enemy computer-owned infantry of a
    crushable type that can stand in water (stock if one can, otherwise its movement zone set through
    the override section), unarmed, idle in a water cell on the ship's route.
  - (b) *Ship slowdown.* Open water, a straight route of 20 cells. A ship-locomotor vehicle of an
    accelerating type with the crusher flag, the tilt-on-crush flag and a movement zone that may
    enter a crushable overlay on water (all set through the override section as needed). The overlay
    type of index 0 is made crushable, not a wall, with a water land type (S-0211), through the
    override section, and is placed in water cell 10 by the map. Control: the same with an overlay
    type of another registration index made alike, in cell 10 instead.
  - (c) *Walking vehicle.* Open ground. Two vehicles A and B of one type with the walking-vehicle
    locomotor, the crusher flag and the acceleration flag (set through the override section as
    needed). The overlay type of index 0 is made neither crushable nor a wall, and passable, through
    the override section, and is placed in cell 5 of a straight route. A drives along the route
    through cell 5 to cell 15 and stops; B drives a parallel route without the overlay and stops
    beside A. Then an explosion as in K9 is set off at equal distance ahead of both.
  - (d) *Bridge deck.* A high bridge whose deck runs along a straight route, and the test vehicle of
    K5. An enemy vehicle it can crush stands on the deck at deck cell *k*. Run d1: nothing under the
    bridge. Run d2: vehicles of the human player stand in the ground layer under deck cells *k* − 1
    and *k* (ground vehicles under a bridge over land, or ships under a bridge over water), and an
    enemy crushable infantry stands in the ground layer under deck cell *k* + 2. The test vehicle is
    ordered across the bridge to a cell beyond it. Run d3: the test vehicle drives on the ground
    under the bridge along the same cells, with an enemy crushable infantry standing in the ground
    layer under one deck cell and an enemy vehicle it can crush standing on the deck above that cell.
- *Procedure:* record per tick positions, presence and owners, and take screenshots in (b) and (c).
- *Observable:* (a) whether and when the infantry starts to move, relative to the tick on which the
  ship begins the track into its cell. (b) the ship's per-tick displacement against the control,
  and whether its body pitches forward on driving into cell 10. (c) A's and B's speeds after cell 5,
  and their largest pitch after the explosion. (d) the test vehicle's per-tick displacement on the
  bridge and after it, which objects disappear, and when.
- *Runs:* (a) 10. (b) 3 each. (c) 3. (d) 2 each (d3 included).
- *Expected:*
  - (a) the infantry starts to move within a few ticks after the ship begins that track.
  - (b) in the main run the ship's displacement drops at once to no more than that of a speed
    fraction of 0.2 when it drives into cell 10, and its body pitches forward then. Whether the limit
    ends at the centre of cell 10 depends on whether the ship crushes the overlay there (Open
    questions); either way the control shows no limit and no forward pitch.
  - (c) A drives at the same speed as B; after the explosion, A's pitch stops at about 18 degrees
    while B's goes beyond it (runs in which B stays below 18 degrees decide nothing).
  - (d) in d1 there is no speed limit on the deck, before or after the enemy vehicle (the test
    vehicle is not in the ground layer, so it does not count itself there, and the enemy vehicle on
    the deck does not count either); the enemy vehicle is crushed. In d2 the limit begins when the test vehicle drives into deck cell *k* − 1
    and lasts after it leaves the bridge; the enemy vehicle on the deck is crushed and the infantry
    under the deck is not.
  - (d3) the infantry on the ground is crushed and the vehicle on the deck above it is not (claim 3,
    cell layers).
- *Falsified if:* (a) the infantry never moves before the ship reaches it, or starts before the track
  begins. (b) no limit or no forward pitch in the main run, or a limit in the control. (c) A slower
  than B, or A pitching beyond 18 degrees while B does. (d) a limit in d1, or the infantry under the
  deck crushed in d2, or in d3 the infantry on the ground surviving or the vehicle on the deck
  crushed. A d2 run with no limit decides nothing, because the track into deck cell
  *k* − 1 may not have been in crush mode: which cell layer the second-cell rule of S-0207 claim 7 reads
  on a bridge deck is not stated there.

**Scenario K11: damage-stage count and wall strength defaults (claim 13).**
- *Setup:* a test wall overlay type (the slot of a stock wall type, through a named test file)
  whose rules section has the wall and crushable flags and no wall-strength key, and whose image
  section in the art data has no damage-stage-count key (Import mapping); every layer is checked
  for both keys. The map places a straight line of five such wall cells, linked to each other.
  - Run A: a crusher vehicle of the human player is ordered across the middle cell.
  - Run B: as A, with the image section of the art data giving a damage-stage count of 3 and the
    rules section a wall strength of 1000, and the middle cell's overlay data value set by the map
    to stage 1.
  - Run C: as B, but the damage-stage count of 3 written in the rules section of the overlay type
    instead of the art data.
  - Run D: no crusher; a test weapon whose warhead can damage walls and whose damage amount is 0 is
    fired once at the middle wall (S-0303 F4).
  - Run E: as D, with a damage amount of 50.
- *Observable:* the overlay data values and presence of the five wall cells after the event, and the
  number of raw values taken from the scenario random stream on the event tick (the recorder reads
  the stream position).
- *Expected:* A: the middle wall is removed, the neighbours keep stage 0 and no draw is taken. B: the
  two neighbours each take a wall hit with one draw each (*S* = 1000 > 200; claim 8's first row,
  stage *D* − 2 = 1). C: as A (the key in the rules section is not read). D: one draw and no change
  (amount 0 is below *S* = 1, and 0 > *r* never holds). E: no draw, and the middle wall is removed
  at once although it links to its neighbours (stage 1 reaches *D* = 1).
- *Falsified if:* A shows a hit or a draw, B shows no draws, C behaves as B, D shows no draw or a
  stage change (a strength of 0), or E leaves the middle wall standing (a count above 1).
- *Runs:* one each; B three times (draws vary).

**Scenario K12: owner of map-placed walls (claim 14).**
- *Setup:* skirmish, human player and one bystander computer player. The map places, for a map
  house *N* whose country has the wall-owner flag (stock default) and a map house *P* whose
  country is given the flag off through the map's override section, one building each: *P*'s
  building 3 cells from a wall cell *w*₁, *N*'s building 6 cells from *w*₁; a second wall cell
  *w*₂ more than 6 cells from every building of *N* and *P*, with *N*'s building the nearest
  qualifying one; and a third wall cell *w*₃ at least 40 cells from every building of the map
  (*N*'s building is the nearest qualifying one). A second map (part b) places the same walls but
  no building of any house, or only buildings whose owner's country has the wall-owner flag off.
  Neither map house is allied with the human player.
- *Procedure:* a human-player tank with a weapon that can damage walls is ordered through each of
  the three walls in turn, from a cell next to it; a second run makes *N* allied with the human
  player through a map trigger at the start.
- *Observable:* the recorder's owner of each wall cell after loading, and each tank's reaction
  (enemy obstacle: attacks the wall; friendly obstacle: does not attack, re-routes or waits; S-0209
  rows WO3, WO4 and the vehicle rows).
- *Expected:* *w*₁ and *w*₂ owned by *N* (*P* is skipped although nearer), *w*₃ owned by *N*, however far (no distance limit). On the part b map all three walls are
  owned by no house.
  First run: the tank treats all three as walls of another house. Second run: *w*₁, *w*₂ and *w*₃ as
  friendly. On the part b map the tank treats all three as walls of another house.
- *Falsified if:* *w*₁ owned by *P*, *w*₃ owned by no house on the first map (a distance limit), any owner
  for any wall on the part b map, or *w*₃ treated as another house's in the second run.
  Optionally place *w*₃ at 10, 20, 30 and 40 cells from *N*'s building to show that the owner
  does not change.
- *Runs:* one each.

**K12 part c: building-list origin and dropped wall buildings (Q-0684).**
- *Setup:* purpose-made skirmish maps, stock wall and building types, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
  game speed index 2, one human and one bystander computer player. Choose two distinct
  non-human houses *A* and *B* that resolve under S-0009 claim 27, with the wall-owner flag
  enabled. A map building-list entry places a wall of *A* on an otherwise empty cell *w*.
  Place a surviving ordinary building of *B* two cells from *w*, and any surviving building
  of *A* much farther away. Keep other buildings farther away. Compare a map with no
  surviving qualifying ordinary building, and a map with the same wall-building entry
  owned instead by the human player's resolved house. For that last map compare an empty
  overlay cell with a wall independently placed there by the overlay list.
- *Procedure:* load each map and record the wall's presence, owner and damage stage before
  any player builds. Use the read-only recorder, and confirm ownership where possible by
  wall-sale eligibility or the allied-wall entry outcomes of S-0209 claim 5. Do not sell
  the wall until the initial record is taken.
- *Expected:* the surviving *A* wall has owner *B*, not *A*, and stays at stage 0. With no
  qualifying building it is present but has no owner. A human-owned wall-building entry
  places nothing on the empty cell. With an independent overlay-list wall there, that
  wall remains and receives the nearest qualifying building's owner as usual.
- *Falsified if:* the *A* wall keeps *A* instead of *B*, retains any owner without a
  qualifying building, qualifies itself, or changes damage stage on reassignment; or the
  dropped human-owned entry creates a wall or removes the independent overlay-list wall.
- *Runs:* one per case. The recorder is sufficient; no harness changes are required.

**Scenario K13: disguise blocks automatic choice, not a drive-through (claim 11).**
- *Setup:* the measurement setup, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), game speed index 2. On a purpose-made
  flat map, place a human-owned unarmed crusher without the omni-crusher flag, its auto-crush flag
  enabled through the named test map's override section, and an enemy crushable spy within half
  the crush range. The spy has copied an infantry unit of the crusher's house. Keep the spy in
  a one-cell-wide corridor with no scatter exit. Compare separate runs with and without the
  crusher's house having building disguise-detector coverage on the spy's cell (S-0516 claim 35).
  The bystander computer player remains outside the test area.
- *Procedure:* in each run let the crusher stand idle for 600 ticks on an occasion that offers
  automatic choice under S-0215. In a separate no-coverage run, order it through the spy's cell
  to a cell beyond it. Record the spy's disguise, owner, position, presence and health, detector
  coverage, and the crusher's position each tick.
- *Runs:* three per case. A drive-through run decides nothing unless the crusher reaches the
  spy's cell centre while the spy remains within the crush radius and still appears disguised.
- *Expected:* without coverage the crusher does not choose the spy for automatic crushing. With
  coverage it chooses and crushes the spy. The ordered drive-through crushes the still-disguised
  spy on arrival at the cell centre, as in claims 2–4.
- *Falsified if:* the idle crusher automatically chooses the spy while the spy appears disguised,
  the covered control fails to choose it despite an automatic-choice occasion, or the spy survives
  a valid drive-through solely because it appears disguised.

**Scenario K14: object crushing uncloaks, passage alone does not (claims 4, 5 and 8).**
- *Setup:* the measurement setup, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), game speed index 2, one human
  player and one bystander computer player outside the test area. A named test map gives
  an unarmed human-owned crusher self-cloaking without the cloak-stop flag. Keep it at
  full health, outside sensor coverage and cloak fields, with no attacks or other uncloak
  causes. Start each passage fully cloaked. Use separate corridors with (a) one trapped
  enemy crushable infantry, (b) two such infantry in one cell, (c) only a crushable wall,
  (d) only a crushable non-wall overlay, and (e) no victim or overlay. Add (f) K3's
  takeover-only setup with no companion, starting with the same cloaked crusher. Give
  the crusher the omni-crusher flag and add (g) one crushable enemy vehicle.
- *Procedure:* order passage through the cell centre and record cloak state, victim presence,
  ownership and tick-stamped sound. Repeat three times per case. A run decides nothing if
  another uncloak cause occurs, the crusher is not cloaked at arrival, or the intended
  crushing or takeover does not occur. In particular, discard (f) if the approaching thief
  reveals the crusher as a cloaked blocker before takeover.
- *Expected:* (a), (b) and (g) start uncloaking on the object-removal tick, with one cloak
  sound even for the two victims in (b). Cases (c), (d) and (e) stay cloaked through arrival.
  Case (f) changes owner without a crushing-triggered uncloak.
- *Falsified if:* an actual object crush leaves the crusher cloaked, two victims cause two
  cloak sounds, or an overlay-only, empty or takeover-only passage starts uncloaking with
  no other uncloak cause. If no takeover run meets the eligibility conditions, that case
  remains unconfirmed rather than establishing an uncloak.

## Open questions

- Whether other locomotors that carry vehicles over the ground (for example hovering vehicles) crush
  at cell centres the same way was examined only for the driving locomotor. For the ship and
  walking-vehicle locomotors only the crush-slowdown state of claim 9 and the warning of claim 7 were
  examined, so whether a ship crushes the overlay of K10 part b at the cell centre is open.
- Whether a balloon-hover type that is also an omni-crusher follows the omni-crusher variant of
  claim 11 is not established. The original treats balloon-hover types specially in some of its
  target-choice rules, and which of them reach the variant was not examined. Until it is, claim 11
  applies to such a type unchanged. No scenario here uses one.
- Whether a train harms objects of the cell layer that are neither units nor buildings, such as a
  terrain object (claim 10): the test of claim 2 is stated for units and buildings only, so this
  spec gives no outcome for them.
- Whether the removal of a crushed unit takes draws from the scenario random stream of its own (for
  example through effects that it ends, such as claim 4's release of controlled units) was not
  established, so the position of such draws relative to those of F3 on the same tick is not
  specified. K4 part c shows only F3's draws, on a map where no unit is crushed.
- Whether the target choice of claim 11 is offered at all while a vehicle is moving (K7 cases 11
  and 12) belongs to the behaviour specs; if it is not, case 11 and run 12h still come out as
  expected, but run 12c does not, and the omni-crusher variant's reach while moving must be
  restated.
- With the ship and walking-vehicle locomotors, a vehicle whose type has the crusher flag (the
  veterancy ability does not count here) also treats a cell whose overlay is of the
  overlay type with registration index 0 as *clear* when it would otherwise get the entry outcome
  *friendly obstacle* or *enemy obstacle* (S-0209). This is the same quirk family as claim 9 but concerns entry,
  which S-0209 owns; it is not specified here.
