---
id: S-0312
title: Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains)
topic: combat
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0060, Q-0061, Q-0070]
supersedes: []
---

# S-0312: Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains)

## Summary

What happens when damage kills an infantry unit, vehicle, aircraft or building: the deaths that are
put off (crippled cyborgs, delayed building deaths), the effects every kind shares (debris, the
death weapon of exploding types, the deaths of their passengers, an attached bomb), and the effects
of each kind (vehicle explosions, sinking, passengers getting out, crews escaping, aircraft
crashing, building explosions, spilled resources, survivors, smudges, rubble and crates), with
their random draws in observable order, when a destroyed building leaves play, and the defaults
and list items of the game-wide values these rules read. Infantry death sequences and corpses are S-0107's (claim 17); when a
killed object leaves the turn order is S-0203's (claim 6); the damage itself is S-0302's and S-0303's.

## Terms

Glossary terms are used as defined there, in particular *infantry*, *vehicle*, *aircraft*,
*building*, *foundation*, *overlay*, *house*, *human-controlled*, *computer-controlled*, *AI house*, *neutral house*, *special house*, *player house*, *scenario random stream*, *draw*,
*ranged draw*, *scatter request*, *contact*, *vehicle thief*, *known attacker*, *departure*,
*importer warning*. S-0202 owns three terms this spec uses: the ranged-draw notation R(*a*, *b*)
(a ranged draw over *a* to *b*, both included), the *percent draw* (R(0, 99), compared with an
integer percentage) and the *list pick* (S-0202 claim 6: one raw value, read as unsigned, reduced
modulo the list length); the *unit-interval value* of S-0202 claim 6 and its *chance test* are also
used as defined there. S-0303's *detonation*, *hit*, *demolition warhead* (its game-wide warhead of
claim 19) and *curtain protection* are used as defined there. In addition:

- **Killing hit.** The hit or other damage that brings an object's health to 0 or less in damage
  resolution (S-0302). Its *attacker* is the object that damage comes with, if any (glossary *known
  attacker*). Some damage carries the *ignore-defences* marker (glossary, *damage that ignores defences*; S-0302 owns which damage carries it) and some the
  *no-escape* marker; which damage carries them is the specs' of their producers, and this spec names
  the cases it creates itself.
- **Death effects.** Everything this spec lists for the moment of a killing hit.
- **Exploding object.** An object whose type has the explodes flag, or which has reached a
  veterancy rank whose abilities include exploding (S-0311), or whose current weapon has
  the suicide flag (Import mapping).
- **Death weapon** of a type: the weapon that its death detonates (claim 6).
- **Debris.** Animations thrown by a death or a detonation: voxel debris types with per-type maxima,
  sprite debris animations of a type, or the game-wide metallic debris list (Formulas F1).
- **Crew.** An infantry unit created when a vehicle is destroyed (claim 13). **Survivor.** An
  infantry unit created when a building is destroyed (claim 18).
- **Armed** (F4 only). An object is armed in F4's sense when it has the primary weapon of claim 6's
  second choice, or, for a building that infantry can occupy, when its occupants can fire from it
  (structures specs).
- **Charge planter.** The infantry unit that planted an explosive charge on a building, remembered
  by the building (special-abilities specs).
- **Side crew type.** The game-wide crew infantry type of a house's side: allied crew for side 0,
  soviet crew for side 1, third crew for side 2 (Import mapping). Sides are numbered by S-0004
  (claim 9).
- **Hijacker type.** The infantry type remembered by a vehicle that a vehicle thief has taken over
  (special-abilities specs).
- **Bomb mark.** A mark that an object can carry which destroys it when it finishes falling to the
  ground. No situation of stock play is believed to set it, so claim 11 records what the original does for
  such an object without any stock scenario to show it.
- **Absorbed passengers.** Units held inside a building whose type absorbs units or infantry (for
  example a building that converts infantry into power). The structures specs own how they get in.
- **Sinking, crashing.** Two states in which a killed vehicle or aircraft stays in play after its
  death until the naval or the locomotor specs end the state.
- **Nearby point** N(*Q*, *r*, *b*). A point *r* leptons from *Q* in one of 256 directions, chosen
  by the remainder *b* (0 to 255) of one raw value of the scenario random stream modulo 256
  (Formulas F2).
- **Rubble overlay.** The overlay type that a destroyed building can leave on its foundation cells
  (claim 20).
- **Crater, scorch mark.** The two kinds of smudge a destruction can leave. A smudge type belongs to
  a kind through its crater flag or its scorch flag (Import mapping); a type with both flags belongs
  to both kinds. Claim 26 gives which smudge type of a kind appears and the draws that choice
  takes. The terrain and smudge specs own how a smudge looks and what later changes it.
- **Smudge footprint.** The rectangle of cells a smudge type covers: its width in cells along map
  *x* and its height in cells along map *y*, starting at the cell where it is placed (the *anchor
  cell*) and growing towards increasing *x* and *y*.
- **Awaiting removal.** The state of a destroyed building that claim 21 keeps in play after its
  death effects, until its removal turn.

## Behaviour

### When death happens

1. **Death effects come at once, in a fixed order.** All death effects of an object happen at the
   moment of its killing hit, inside damage resolution, before the next hit of the same detonation
   (S-0303 claim 1). When the damage is produced during an object's turn, the death effects happen
   within that turn; when the damage is produced outside an object's turn, the death effects happen
   at that effect's place in the tick (S-0203 claims 1 and 5). The table in State and ordering gives the observable precedence of the death effects and the order in which their random draws affect the scenario random stream. For example, a passenger killed by an exploding object is already dead when the death-weapon detonation's damage is dealt. A death weapon or bomb completes its detonation before the death proceeds to effects that follow it in the table. When the
   object leaves play is S-0203's claim 6; this spec adds when buildings, sinking vehicles and
   crashing objects leave (claims 10, 15, 16 and 21).
   - Backing: `measurable` (scenario D1)
   - Confidence: high for the order of the shared effects and of each kind's effects; medium for the
     relative order of rarely combined effects (Open questions)

2. **Cyborgs are crippled before they die.** S-0302 (claim 15) owns this rule; it is restated here
   because it replaces the death effects. When a hit that does not ignore defences would bring an
   infantry unit of a cyborg type to health 0 or less, and the unit is not prone at that moment, it
   does not die: the game-wide infantry-explosion animation appears at its position, its health becomes the greater of 1 health point and the integer obtained by truncating one quarter of *H* toward zero, where *H* is its type's strength, and it is made to go prone and
   crawl (S-0107). A prone cyborg (crippled before, or prone for any other reason), or one hit with
   damage that ignores defences, dies normally and takes the death effects below (S-0107 claim 17
   gives its death sequence).
   - Backing: `measurable` (scenario D2)
   - Confidence: high for the health and the prone condition; medium for the sequence

3. **Delayed building deaths.** S-0302 (claim 16, F7, `quirks.delay_kill_whole_cell_spread` and
   `quirks.delay_kill_credited_again`) owns this rule, including its defaults and confidences; F6
   below is the same countdown, restated because it replaces the death effects, and S-0302 wins
   wherever the two differ. When a hit of a warhead with the delay-kill flag kills a building
   whose type is eligible for delayed death, the building does not die: it keeps 1 health point and
   a countdown of *T* ticks starts (Formulas F6), unless a countdown is already running with at most
   *T* ticks left, which is then kept. A building killed by any other warhead dies at once. The kill
   is credited to the attacker already at the moment of the delayed-death hit, as S-0302 claim 16
   gives it (score and experience belong to their specs, and S-0302 says what a repeated hit
   credits).

   *End of the countdown.* The countdown does not remove the building by itself. On the building's
   first eligible turn at or after the end, S-0302 claim 16 delivers a hit of the building's whole
   current health with the demolition warhead, ignoring defences. Normally that hit has no attacker
   and no attacking house: the initial hit's attacker is not remembered for expiry. A separately
   remembered charge planter can be its attacker. S-0302 owns its credit, the erasure and
   electromagnetic-pulse postponements, cancellation by temporary invulnerability, survival when
   the demolition warhead stops the hit or delays death again, and the bridge repair hut exception.
   Capture and starting a sale do not cancel the countdown, but removal before expiry leaves no
   later hit or death effects (S-0302 claim 16; Q-0081 parts 1 and 2). When that hit
   destroys the building, every death effect of this spec follows at once, in that turn, with its
   normal order and draws (claims 4 to 7 and 17 to 24), exactly as for a building destroyed by any
   other hit. Because the hit ignores defences, the building then counts as having no crew (F5): it
   releases no survivors and takes no per-cell smudge steps (claim 18 and claim 19 (g)), and its
   absorbed passengers die instead of coming out (claim 24). Its contacts (claim 17), its debris,
   its death weapon when it is exploding, its smudge of claim 19 (a), its explosions and its rubble
   are unaffected. A building that is exploding or being sold then awaits removal as claim 21 gives.
   - Backing: `measurable` (scenario D3; the expiry itself through S-0302's scenario D13, parts e
     to h)
   - Confidence: high (the formula and the kept countdown; the death effects at the expiry hit and
     the absence of survivors); medium (the credit at the moment of the hit, as S-0302 claim 16 rates
     it)

### Effects every kind shares

4. **Debris.** When an object whose type has a positive maximum debris count dies, it throws debris
   by Formulas F1: the count is one ranged draw, and the pieces are created at the object's position
   owned by its house.
   - Backing: `measurable` (scenario D4, statistical)
   - Confidence: high

5. **Exploding objects kill their passengers.** When an exploding object dies, every passenger it
   carries dies with it, and each death is credited to the attacker of the killing hit (score and
   experience belong to their specs). The passengers show no death effects of their own and take no
   draws. Their deaths come before the death weapon goes off (claim 6), so a passenger is already
   dead when the detonation's damage is dealt. The passengers die one after another in the order
   they are held, and the passengers of a passenger die before it does. That order is observable
   only in the order in which the deaths reach things that react to them, such as the triggers
   attached to the passengers (S-0011); nothing else in the game state depends on it.
   - Backing: `measurable` (scenario D5, variant b for the order)
   - Confidence: high for the deaths, the credit and their coming before the detonation; medium
     for the order among passengers

6. **Death weapon.** An exploding object's death sets off a projectile detonation (S-0303) of the
   chosen weapon's projectile type at its position: the area damage and the explosion animation
   are at the object's position, and S-0303's other projectile effects (claims 21 to 23) apply.
   That projectile is never placed on the map, so the tests of S-0303 claim 21 that use the
   projectile's own position (land type, height, water) and the warhead debris of S-0303 claim 22
   are believed to use its unplaced default position rather than the object's (Open questions,
   `quirks.death_weapon_unplaced_projectile`). The
   weapon is the type's own death weapon if it has one, otherwise its current primary weapon (the
   weapon in its first slot, its elite weapon when it is elite, or for a type with several turret
   weapons the one in use), otherwise the game-wide death weapon. The damage amount is Formulas F3:
   the weapon's damage scaled by the type's death-weapon damage factor, or half the type's strength
   for the game-wide weapon. The detonation uses that weapon's warhead and brightness, has the dying
   object as its source object and the object's house as its source house, so the dying object
   itself is not hit (it is also dead, S-0303 claim 9). An object that is not exploding fires no death
   weapon at death; a crashing aircraft fires one when it hits the ground (claim 16). The type's
   death-weapon damage factor *φ* (F3) applies to the primary weapon too, not only to a death
   weapon of the type's own.
   - Backing: `measurable` (scenario D6; variant d for the debris position)
   - Confidence: high for the choice and the formula; medium for the multi-turret case; low for
     the position used by the land-type tests and the debris

7. **Attached bombs go off.** A bomb attached to the dying object (an infantry-planted time bomb,
   special-abilities specs) detonates at the end of the shared effects, whether or not the object
   is exploding.
   - Backing: `measurable` (scenario D6, variant c)
   - Confidence: medium

8. **Infantry.** A killed infantry unit takes the shared effects of claims 4 to 7 and then the death
   sequence or removal of S-0107 claim 17, which also owns its corpse. Nothing further belongs to
   this spec.
   - Backing: `measurable` for the debris, the death weapon and the attached bomb of an infantry unit
     (scenario D4 variant b, D6 variant e; S-0107 scenario M10 for the rest); `unmeasured` for the
     passengers of an infantry unit (no stock infantry type is believed to carry passengers, so no
     scenario can show the passenger part of claim 5 for infantry)
   - Confidence: high for the debris, the death weapon and the bomb; low for the passengers

### Vehicles

9. **Explosion and destruction animations.** A killed vehicle that neither sinks (claim 10) nor
   plays death image frames (Open questions) shows, at its position, one entry of its type's explosion
   list and one entry of its type's destruction-animation list, each chosen by one *list pick*
   (S-0202 claim 6), explosion first, each only when its list is not empty. A vehicle whose type has the explodes
   flag, or which has reached a veterancy rank whose abilities include exploding (S-0311; here the suicide
   flag of its weapon does not count), and whose type has unlimited ammunition or which still has
   ammunition, shows the last entry of its explosion list instead, but the list pick's draw is still taken.
   The value of any resource it carries plays no part (a full harvester explodes like an empty
   one).
   - Backing: `measurable` (scenario D7)
   - Confidence: high for the picks; medium for the harvester statement

10. **Sinking.** A killed vehicle of a naval type that is not an underwater type and not organic,
    whose type's weight is at least the game-wide sinking weight, and that is on a water cell, sinks
    instead of exploding: it keeps 1 health point, stays in play and shows no explosion or
    destruction animation (claim 9). Its passengers are handled as in claim 12. How it sinks and
    when it leaves play belong to the naval specs.
    - Backing: `measurable` (scenario D8)
    - Confidence: high for the condition; low for anything after it

11. **Vehicles with the bomb mark that die low over water.** A vehicle that carries the bomb mark
    (Terms) and dies within 10 leptons above a water cell skips claims 4 to 6 and 9 (its attached
    bomb, claim 7, still goes off). Instead it shows the game-wide wake animation and the last entry
    of the game-wide splash list (S-0303, F5) at its position. Claims 12 to 15 then apply as for any
    other destroyed vehicle, except that its passengers do not get out: claim 12 kills them, or
    keeps them inside until the crash when the vehicle's type can crash. The wake and splash animations take the place of claim 9's
    animations in the order of death effects (State and ordering). No situation of stock play is
    believed to give a vehicle the bomb mark, so there is no stock scenario for this claim; it is
    recorded so that an engine that gives some object the mark (a mod's dropped units, say) can
    reproduce the original.
    - Backing: `unmeasured` (nothing in stock play sets the condition, so it cannot be observed
      without a modified game; if a scenario that sets the mark is ever found, it becomes
      `measurable` with a variant of D8)
    - Confidence: low

12. **Passengers of a destroyed vehicle.** Passengers that claim 5 has not killed are handled after
    the vehicle's animations. What happens to them depends on the vehicle as follows (the rows
    exclude one another):

    | Vehicle's height above the terrain | Type can crash (claim 15) | Bomb mark (claim 11) | Passengers |
    |---|---|---|---|
    | more than 208 leptons | any | any | all die, credited to the attacker |
    | 208 leptons or less | yes | any | stay inside; they die when it starts to crash, or, when it is on the ground and so cannot crash, when it leaves play |
    | 208 leptons or less | no | yes | all die, credited to the attacker |
    | 208 leptons or less | no | no | get out, as below |

    In the last row each passenger in turn gets out: it appears at the vehicle's horizontal
    position, at the height of the ground surface at the centre of the vehicle's cell (on a bridge
    deck, at a point the bridge specs give, when the vehicle was on one), facing the vehicle's body
    facing rounded to the nearest of 256 steps (S-0204 F4 with *n* = 256), and receives a scatter
    request with no threat point, forced and not move-anywhere (S-0208). A passenger dies instead,
    credited to the attacker, when the killing damage ignores defences, when it cannot enter the
    vehicle's cell (an entry outcome other than clear or moving blocker, S-0209 claim 4), or when
    it cannot be placed.
    A passenger that has got out is given no order beyond the scatter request when its house is
    human-controlled. When its house is computer-controlled, the original also puts it on the *hunt*
    behaviour (S-0215), unless the destroyed vehicle belonged to a team, which the passenger then
    joins instead. Which of these our engine applies depends on the owner (claim 25): a player
    house's passenger gets no further order, a neutral or special house's passenger is put on
    *hunt*, and the team join never applies.
    This refines S-0203 claim 6(e): S-0312 decides whether passengers of a destroyed vehicle die
    or get out, while S-0203 claim 3 determines whether passengers of an open-topped transport are
    in play.
    - Backing: `measurable` (scenario D9; variant e for the orders); `unmeasured` for the team join
      (the original's team system is outside our scope and has no counterpart in our engine, so
      nothing of ours depends on it, and no test map is planned for it)
    - Confidence: high for the height rule and the escape; medium for the entry-outcome condition
      and the facing; medium for the orders; low for the team join

13. **Crew.** After its passengers, a destroyed vehicle may leave a crew infantry unit at its
    position, owned by its house:
    - A vehicle with a hijacker type always leaves one infantry of that type, with no chance test
      and no draw for its type.
    - Otherwise, when the killing damage does not carry the no-escape marker, the type has the
      crewed flag and has no passenger capacity, a chance test "less than" with the game-wide crew
      escape chance (one unit-interval value, S-0202 claim 6) decides, and the crew's type is chosen
      by Formulas F4.
    A crew that is due takes S-0202's creation draw (claim 10) whether or not it can then be placed
    at the vehicle's position. A crew that cannot be placed there never enters play and takes no
    health draw. A placed crew's health is a ranged draw over 5 to half its type's strength
    (truncated toward zero). It faces map north, receives the same scatter request as escaping
    passengers (claim 12), is put on the *guard* behaviour when its house is human-controlled and
    on the *hunt* behaviour when it is computer-controlled (S-0215; which one our engine applies
    depends on the owner, claim 25), and keeps the vehicle's trigger tag when that tag is of a kind that passes
    on (trigger specs). The draws come in the order: chance test (not for a hijacker type), crew type
    (F4, when it takes a draw), creation, health (only for a placed crew).
    - Backing: `measurable` (scenario D10, statistical for the chance; variant d for the behaviour)
    - Confidence: high for the chance and the health draw; medium for the hijacker case, the tag and
      the behaviour

14. **Crates from crate-carrying vehicles.** A destroyed vehicle whose type carries a crate leaves a
    random crate on a free cell near its position, when the game's crate setting allows crates
    (crate specs).
    - Backing: `measurable` (scenario D10, variant c)
    - Confidence: low (the exact condition on the game settings is not known)

15. **Crashing vehicles and removal.** A destroyed vehicle whose type can crash, and which is above
    the terrain, starts to crash (claim 16) and stays in play until it hits the ground. A vehicle
    that sinks stays in play (claim 10). Every other destroyed vehicle leaves play at the end of its
    death effects, in the same moment (S-0203 claim 6a).
    - Backing: `measurable` (scenario D11)
    - Confidence: high

### Aircraft and crashing

16. **Aircraft deaths and crashes.** A killed aircraft takes the shared effects, then shows one entry
    of its type's explosion list (one list pick) at its position, and then starts to crash if it is
    above the terrain; otherwise it leaves play at once. An object that starts to crash loses all
    its passengers (credited to the attacker) and, unless it is infantry, takes three draws for its
    tumbling (one unit-interval value, one ranged draw over 0 to 1 that chooses the sense, one more
    unit-interval value). It then falls (locomotor specs). When it reaches the ground:
    - if it was shot down, its death weapon goes off at the impact point (claim 6, even if it already
      went off at the moment of death because the aircraft was exploding), and it leaves play,
    - if it was still alive (a crash started without a kill), it is destroyed by damage equal to its
      remaining health with the demolition warhead, ignoring defences, with the no-escape marker and
      no attacker (so its kill is credited to no one); an explosion
      animation chosen by S-0303 F5 for that amount appears, and a detonation with that amount, the
      demolition warhead, no source object and no source house follows at the impact point.
    - Backing: `measurable` (scenario D11)
    - Confidence: medium for the impact rules; low for the double death weapon and for which objects
      use this crash (aircraft and crashable jumpjet types are believed to)

### Buildings

17. **What a destroyed building does to the objects in contact with it.** When a building is
    destroyed, every object in contact with it (glossary *contact*, for example a vehicle leaving
    its factory or docked at it) is destroyed by damage of ten times its own strength with the
    demolition warhead, ignoring defences, with the no-escape marker and with no attacker, when the
    spatial distance from the building's reference point (S-0204 claim 4) to the contact's position
    is less than 256 leptons, or whatever the distance when the building is a helipad. Any other
    contact is only released. Garrisoned infantry leave the building (structures specs).
    - Backing: `measurable` (scenario D12)
    - Confidence: high (the distance rule, the helipad rule and the damage); low (which situations
      of play are contacts at that moment, and the garrison statement)

18. **Survivors.** A destroyed building may release survivors, at most *C* of them (Formulas F5).
    When *C* = 0 nothing in this claim happens, and neither does the per-cell smudge step of claim
    19 (g). Otherwise its foundation cells are taken one after another in the cell order of S-0302
    (claim 13: row by row, the rows by increasing map *y* from the top-left cell and the cells of a
    row by increasing map *x*; for the 3 × 3 foundation that has only 8 cells, glossary
    *foundation*, the eight cells S-0302 gives), starting after the cells that absorbed passengers
    have used (claim 24).

    *Chance.* A cell takes one ranged draw R(0, *g*) when, at its place in the order, fewer than *C*
    survivors have been released and the owner's game is not over (the owner is not in the state that
    S-0414's defeat puts a house in; Open questions); otherwise it takes no draw. A
    result of 1 releases one survivor in that cell. Here *g* = 2, or 1 when an infantry unit had
    planted an explosive charge on the building (it has a charge planter), plus 6 when the building
    has ever changed owner.

    *Type.* The survivor's type and the draws it takes are:

    | Building has changed owner | Building produces buildings | Draws for the type | Type |
    |---|---|---|---|
    | no | yes | one percent draw *p*; when *p* ≥ 25, then F4's percent draw when F4 takes one | engineer when *p* < 25, otherwise F4's type |
    | no | no | one percent draw, whose result changes nothing; then F4's percent draw when F4 takes one | F4's type |
    | yes | any | F4's percent draw when F4 takes one | F4's type |

    So an armed building that produces buildings and has not changed owner, and whose owner's
    country belongs to a side (F4), takes one percent draw when it releases an engineer, and two when
    it does not.

    After its type draws, a survivor takes S-0202's creation draw (claim 10) whether or not it can
    then be placed. It goes on a free standing position of that cell. A survivor for which no
    standing position is free, or which cannot be placed, never enters play, takes no health draw
    and does not count against *C*. A placed survivor's health is a ranged draw over 5 to its
    type's strength, and it receives the same scatter
    request as escaping passengers (claim 12). Its behaviour then depends on the building's charge
    planter (Terms): when the building has one and the owner is not allied with it, the survivor is
    ordered to attack it. Otherwise the survivor is put on the *move* behaviour when its house is
    human-controlled and on the *hunt* behaviour when it is computer-controlled (S-0215; which one our
    engine applies depends on the owner, claim 25). The attacker of the killing hit plays no
    part in this order. When the killing damage ignores defences, the building counts as having no
    crew (F5), so it releases no survivors at all.
    - Backing: `measurable` (scenario D13, statistical; variant d for the damage that ignores
      defences; variant e for the engineer draw and the behaviours)
    - Confidence: high for the counts, chances, types and the draw order; medium for the charge
      planter as the target of the attack order (which events set and clear the remembered planter
      is not known) and for the *hunt* behaviour; low for the *move* behaviour of a survivor
      without an attack order

19. **Other effects of a building's destruction.** In addition to the shared effects, a destroyed
    building, in this order:
    - (a) leaves a smudge at the centre of the cell of its position, when its foundation is at least
      2 × 2: first one discarded ranged draw over 0 to *W* − 2 when *W* > 2 and one over 0 to *H* − 2
      when *H* > 2 (*W* × *H* its foundation), then a percent draw: below 50 a scorch mark, otherwise
      a crater, then the smudge choice of claim 26 for that kind at that cell, in its form that
      prefers large smudges and ignores buildings,
    - (b) for each foundation cell, when its type has an explosion list: an explosion animation at
      the nearby point N(cell centre, 64, *b*) at the world height of the building's position,
      with a starting
      delay of a ranged draw over 0 to 3 ticks, and its type picked by one list pick; the draws come
      in the order nearby point's raw value, delay, list pick,
    - (c) for a building whose type has the explodes flag: animations of fire on exploding
      overlays (claim 23),
    - (d) spills the resources it stores (claim 22),
    - (e) shows one entry of its destruction-animation list (one list pick),
    - (f) creates one particle system from its type's destruction particle-system list, when the
      building has no particle system attached already, at least one entry is eligible (the
      particle-system specs say which kinds are) and it stands higher than 10 leptons below the
      terrain: one ranged draw R(0, *m* − 1) over the *m* eligible entries, which are counted from
      the end of the list (*r* = 0 picks the last eligible entry), placed at its position moved by
      its type's destruction smoke displacement,
    - (g) releases its absorbed passengers (claim 24), then its survivors (claim 18). When *C* = 0
      (claim 18) nothing more happens in this item. Otherwise, for each foundation cell in claim 18's cell order, starting after the cells used by absorbed passengers, the applicable survivor and smudge outcomes have the draws shown in the table below. For each cell, those draws precede the draws applicable to the next cell. Every such cell has the smudge outcome whether or not survivors remain to release and regardless of which survivor outcomes apply.

      During these steps the destroyed building is still in its foundation cells (it leaves them
      only when it leaves play, claim 21). Two consequences follow. The clear-for-movement test of
      the smudge step ignores buildings, vehicles and infantry in the foundation cell, so the dying
      building does not fail it; the cell's land type and walls still decide it (S-0211). The plain
      form of claim 26 refuses every smudge type whose footprint touches a cell holding a building,
      the dying building included, so a nearby point that lands in one of the building's own
      foundation cells leaves no smudge and takes no draw for a smudge type. A nearby point 128
      leptons from a cell centre stays in that cell, except a point exactly on the cell's east or
      south edge, which belongs to the next cell (F2 truncates). So in a destruction the smudge
      steps leave a smudge only in the rare case of a point on the outer east or south edge of the
      foundation; every other smudge step takes its percent draw and its nearby point's raw value
      and nothing more. Claim 21's second round, which comes after the building has left its
      cells, is not limited in this way.

      | Row | Taken when | Draws | Effect |
      |---|---|---|---|
      | survivor chance | as claim 18 (*Chance*) gives | one ranged draw R(0, *g*) | a result of 1 releases a survivor |
      | survivor | the chance row released a survivor | its type, creation and health draws, as claim 18 gives them | one survivor, when it can be placed |
      | smudge step | the foundation cell itself (not the cell of the nearby point) is clear for movement | one percent draw, then the raw value of the nearby point N(cell centre, 128, *b*), then the draws of claim 26's smudge choice in its plain form | a scorch mark when the percent draw is below 50, otherwise a crater, at the cell of the nearby point, when claim 26 finds a smudge type that fits there |

    - (h) kills any passengers still inside it.
    - Backing: `measurable` (scenario D14; the draw order through the random-state recorder, which
      gives the number of raw values taken, and through the statistics of the effects)
    - Confidence: medium (the list and the draws, and the building still being in its cells during
      the smudge steps); high for the order of the draws within a cell and for the cell the smudge
      step tests; low for the eligibility in item f

20. **Rubble and crates beneath.** When a destroyed building's type leaves rubble and has a rubble
    image (a separate rubble building image, or a sprite with more than 7 image frames), each of its
    foundation cells receives the overlay type of registration index 239 (in stock data the rubble
    overlay, S-0004; the index is built into the original, so a mod's overlay list can change which
    type it is: `quirks.rubble_overlay_by_index`, Edge cases), with its overlay data value left as
    it was, and the building's top-left cell remembers the building's type for drawing the rubble
    (rendering specs). When the type has a crate beneath, a crate appears
    on the building's top-left foundation cell: a money crate when the type says so, otherwise a crate
    of a kind the crate specs choose.
    - Backing: `measurable` (scenario D14, variant b)
    - Confidence: medium (the rubble condition and the crate cell); low (the kind of a crate that is
      not money)

21. **When a destroyed building leaves play.** Here only the type's explodes flag counts (a
    veterancy ability or a weapon's suicide flag, which make an object exploding elsewhere in this
    spec, do not). A destroyed building whose type lacks the explodes flag and whose current
    behaviour is not the sale's at the moment of its destruction leaves play at the end of its
    death effects, together with its rubble and crate of claim 20, in the same moment. A building
    whose type has the explodes flag, or whose current behaviour is the sale's when it is
    destroyed, instead awaits removal (Terms) after its death effects, with 0 health:
    - It cannot be hit (S-0303 claim 9 skips an object with 0 health) and it is no candidate for
      target acquisition (S-0309 claim 12, row *exists*).
    - It keeps occupying its foundation cells, with whatever claim 20 has not yet placed: its
      rubble and crate are not placed yet.
    - It takes its turns (S-0203), but its behaviour is not processed while its health is 0
      (S-0215 claim 2). In particular a sale cannot advance, release further sale survivors or
      complete its refund after this death. When the building is still in play with 0 health,
      and is neither being erased nor under an electromagnetic-pulse effect, its own turn removes
      it after the general part of that turn. On an eligible removal turn, the building ceases to occupy its foundation cells. It then has a second round of claim 18 and claim 19(g). The survivor count *C* is worked out afresh by F5, so it is 0 when the killing damage ignored defences, which left the building without crew. The cells are taken in claim 18's order with the survivor-chance, survivor and smudge-step outcomes and all their draws, as in claim 19(g). Absorbed passengers have already come out or died, so none come out now. Because the building has left its cells, the smudge steps of this round are not refused by the building (claim 19(g)). Any passengers still inside die. The building then leaves play, and claim 20's rubble and crate are placed.
      Quirk `quirks.building_removal_second_round` (Edge cases) covers this second round.
    - So a building of an exploding type normally stays in play until its own next turn, which comes later in
      the same tick when its turn has not yet come in that tick's turn pass, and otherwise in the
      next tick. While it is under an electromagnetic-pulse effect, or being erased, the removal
      waits for a later turn. This is the same for a building destroyed while being sold,
      whatever stage the sale had reached. Sale survivors released before the killing hit are
      unaffected; after it there are only the destruction's survivor rounds, not the sale's.
      It pays no sale refund (S-0408 claim 6). A sale that already removed a still-living building
      before a pending delayed-kill expiry instead leaves no death at all (claim 3).
    Whether a building awaiting removal can still be selected by a player is not known (Open
    questions). This refines S-0203 claim 6b.
    - Backing: `measurable` (scenario D14, variants c and g)
    - Confidence: high for immediate removal, the next eligible own-turn removal of exploding
      and selling buildings, occupancy until removal, the second round, the erasure and
      electromagnetic-pulse exceptions, and the stopped sale; selection remains open

22. **Spilled resources.** In the original a building's holdings of each resource type are
    fractional amounts (single-precision floating point), and the building spills for as long as the
    sum of its holdings over all resource types is at least 1. Each spilled unit comes from the
    resource type earliest in registration order whose holding is above 0, and takes up to 1 from it:
    a holding of at least 1 drops by exactly 1, and a holding below 1 is emptied. The same amount, by
    the same rule, is taken from that type in its house's total. One whole resource unit is added to
    the cell of the nearby point N(*Q*, *ρ*, *b*) snapped to the cell centre, where *Q* is the
    building's position and *ρ* a ranged draw over 256 to 768 leptons, whatever the amount taken.
    Each unit takes its *ρ* draw and then the raw value of the nearby point. So holdings of
    whole units spill exactly their count of units. With fractions, a unit may be taken from a type
    that holds less than 1 (it is emptied), and the spilling ends when the sum is below 1, so a
    remainder below 1 in total is not spilled. How a cell gains resource
    belongs to the economy specs.
    - Backing: `measurable` for holdings of whole units (scenario D15; how a building comes to hold
      resources in a skirmish is the economy specs', and the scenario needs a map setting or a
      start-credit conversion that fills one, Open questions); `unmeasured` for fractional holdings
      (no stock situation is believed to leave a building with a fraction, so the clamp at 1 cannot
      be shown)
    - Confidence: medium for whole units; low for fractions

23. **Exploding buildings set off barrels.** A destroyed building whose type has the explodes flag
    (here, as in claim 21, a veterancy ability or a weapon's suicide flag does not count) probes four
    points, each one cell away from the building's position (the centre of its top-left foundation
    cell) towards map north, east, south and west, in that order. On each probe whose cell holds an exploding overlay
    (S-0303 claim 19) it starts an animation of a fixed type (Import mapping) at the probe's point,
    at the same height above the terrain as the building's position, with a starting delay of 4 to
    6 ticks (a ranged draw over 1 to 3, plus 3). What the animation then does to the barrel is the
    animation's (S-0109). For a foundation larger than 1 × 1 the east and south probes land on the
    building's own foundation cells, so a barrel beside the east or south side of such a building is
    never found; only the north and west probes can find one (`quirks.barrel_probe_top_left_only`,
    Edge cases).
    - Backing: `measurable` (scenario D14, variant d)
    - Confidence: medium

24. **Absorbed passengers.** When a building that absorbs units or infantry is destroyed, its
    absorbed passengers come out one after another, each into the next foundation cell in the cell
    order of S-0302 (claim 13), whether or not the previous passenger could be placed: the first
    passenger uses the first cell, the second the second, and so on, and the survivors of claim 18
    start after the last cell used. A vehicle comes out at the cell centre, an infantry unit on a
    free standing position. Each gets out facing the building's facing and receives the scatter
    request of claim 12. Once out, a passenger is given no further order when its house is
    human-controlled and is put on the *hunt* behaviour when it is computer-controlled (S-0215; which
    one our engine applies depends on the owner, claim 25). One that cannot be placed
    dies, credited to the building's charge planter (Terms), if any, and otherwise to no one (not to
    the attacker of the killing hit). When the building has no crew (a map setting, or killing
    damage that ignores defences, F5), all of its absorbed passengers die instead of coming out,
    credited in the same way. They come out whatever the survivor count. The original does not
    stop at the last foundation cell: with more passengers than cells it places the further
    passengers at unspecified positions (Edge cases, *Absorbed passengers beyond the last cell*).
    - Backing: `measurable` (scenario D16)
    - Confidence: medium for the placement and the credit; high for the cell order and for the
      survivors starting after the last cell used; medium for the *hunt* behaviour

### Computer-controlled houses

25. **Orders after a death depend on the owner, not on who plays it.** In our engine the orders
    that claims 12, 13, 18 and 24 give to the units a death releases are:

    | Released unit | Owner is a player house (human-controlled or an AI house) | Owner is the neutral or the special house |
    |---|---|---|
    | crew (claim 13) | *guard* | *hunt* |
    | survivor without an attack order (claim 18) | *move* | *hunt* |
    | escaping passenger (claim 12), absorbed passenger that came out (claim 24) | no order beyond its scatter request | *hunt* |

    The original gives these units the *hunt* behaviour, and an escaping passenger the join of the
    destroyed vehicle's team, whenever their owner is computer-controlled. By the project lead's
    decision (glossary, *AI house*, *Player houses only*) an AI house follows the player-house
    column, while the neutral and the special house keep the original's rule for owners that are not
    human-controlled. Our engine has no team system (glossary, *team member*): no vehicle belongs to
    a team, so the join of claim 12 never happens for any owner, and the neutral and special houses'
    passengers take *hunt* instead (Questions for the project lead).
    - Backing: `measurable` for the human-controlled column and for the neutral-house column
      (scenarios D9 e, D10 d, D13 e and D16 c, each also with a neutral-owned object); `unmeasured`
      for the AI-house application (a design rule of our engine, following the project lead's
      decision on AI houses, so no original shows it) and for the team join
    - Confidence: high for the table; low for the neutral and special houses' passengers, since the
      team system that the original prefers for them is out of scope

### Smudge choice

26. **Which smudge type a destruction leaves.** Claim 19 (a) and the smudge steps of claim 19 (g)
    choose a smudge type of a given kind (crater or scorch mark) for an anchor cell (the cell of the
    point they give) as follows.
    - *No smudge at one cell.* When the anchor cell is the map cell with both coordinates 0, nothing
      is placed and no draw is taken.
    - *Fit.* A smudge type of the kind fits at the anchor cell when the anchor cell lies on the map
      and every cell of its footprint (Terms) is flat (no slope), holds no overlay and no smudge, and
      has a tile whose tileset has the morphable flag (S-0010 claim 15). In the *plain form*
      (claim 19 (g)) a footprint cell that holds a building also refuses the type; the *large form*
      (claim 19 (a)) does not test for buildings. Only the anchor cell is tested for lying on the
      map (Edge cases, *Smudge footprints beyond the map edge*).
    - *Candidates.* The fitting types of the kind, in registration order (S-0004). In the large form,
      when at least one fitting type is wider than one cell and taller than one cell, the candidates
      are only those types; otherwise they are all fitting types. In the plain form they are all
      fitting types.
    - *Choice.* With *n* candidates, one ranged draw R(0, *n* − 1) gives the position of the chosen
      type in the candidate list. So *n* = 1 takes no raw value (S-0202 claim 4), and *n* = 0
      places nothing and takes no draw.
    - *Placing.* The chosen type is placed with the anchor cell as its top-left cell. Placing it takes
      no draw.

    The choice depends on the theater only through the tiles' tilesets and their morphable flag,
    and on the cells' state (slopes, overlays, smudges already there, and in the plain form
    buildings). A smudge already in a footprint cell refuses the type, so a second destruction at
    the same place leaves no smudge where the first one did. These are the only draws a smudge
    choice takes.
    - Backing: `measurable` (scenario D17; the draw count through the random-state recorder)
    - Confidence: high for the candidates, the preference and the draw; medium for the absence of
      draws in placing; low for footprints beyond the map edge

### Game-wide values and lists

27. **Defaults of the crew, survivor and sinking values.** At a fresh program start the crew escape
    chance of claim 13 is 0.5, the sinking weight of claim 10 is 3.0, and the survivor divisors of F5
    are 200 for side 0, 100 for side 1 and 300 for side 2. Omitting any of these fields from a rules
    layer keeps the value in force, including a value left by an earlier game of the same session
    (S-0006). An omission never disables the rule that uses the value: crews still escape by the
    chance, vessels still sink by the weight, and survivors are still counted with the divisor (a
    divisor of 0, given explicitly, is what gives *C* = 0, F5).
    - Backing: `measurable` (scenario D18)
    - Confidence: high

28. **Item handling of this spec's lists.** The explosion list, the destruction-animation list, the
    debris animations, the voxel debris types and the destruction particle-system list of a type,
    and the game-wide metallic debris list omit an item naming no type (Import mapping): it holds
    no place, the other items keep their order, and every list length and entry number in claims
    9, 16 and 19 and F1 is taken after those omissions. S-0003 claim 19 owns this shared syntax
    for animation and voxel-animation lists; the destruction particle-system list has the same
    omission and splitting rules. A type's debris maxima list follows S-0003 claim 20:
    decimal-prefix signed 32-bit integers, an unreadable item holding its place with 0,
    independently of the types list, so F1 pairs the first kept debris type with the first maxima
    item. S-0303 claims 25 and 26 use the same rules for detonation lists. A name-list key absent from a layer, or given with an empty value, keeps
    the list in force (S-0003, S-0006); a value whose items all name no type replaces the list with
    an empty one. Name-list text is limited to 127 bytes; maxima-list text to 511 bytes, with
    the physical-line limit of S-0001 applying first. Whitespace within an item follows S-0003
    claim 12 and `quirks.list_items_untrimmed`.
    - Backing: `measurable` (scenario D19); `unmeasured` for the separate 511-byte reader limit,
      which file input cannot reach after S-0001's shorter physical-line limit
    - Confidence: high

## Formulas

**F1. Death debris** (claim 4). With *X* the type's maximum debris count (*X* > 0) and *Y* its minimum:

> *N* = R(*Y*, *X* − 1)

If the type lists voxel debris types with maxima, the *N* pieces are dealt out to the types in turn
exactly as in S-0303 F6 (one raw value per turn, "magnitude first" remainder modulo the maximum plus
one, capped at the pieces left), each piece one debris animation at the object's position. Otherwise,
if the type lists debris animations (*c* of them), *N* animations each of the entry R(0, *c* − 1),
at the object's position raised by 20 leptons. Otherwise *N* animations from the game-wide metallic
debris list, chosen the same way, at the same point. Exact integer arithmetic. Unit: pieces (*N*).

**F2. Nearby point** (claims 19, 22, 23). For a point *Q* = (*x*, *y*, *z*), a distance *r* in
leptons and the remainder *b* (0 to 255) of one raw value modulo 256:

> *f* = 256*b* for *b* < 128 and 256*b* − 65536 for *b* ≥ 128, *θ* = RZ₅₃((*f* − 16383) × *κ*),
>
> *κ* = the double nearest −π / 32767 (= −2π / 65534, about −9.5876725 × 10⁻⁵ radians per facing unit)
>
> *y*′ = trunc(RZ₅₃(*y* − RZ₅₃(*r* × sin̂ *θ*))), *x*′ = trunc(RZ₅₃(RZ₅₃(*r* × coŝ *θ*) + *x*)), *z*′ = *z*

*κ* is S-0204's direction scale (not −2π / 65536), which the original uses here; this is not a defect
and has no switch. sin̂ and coŝ are the table trigonometry of S-0105 (F1); RZ₅₃ is S-0205's rounding of each result
(with `quirks.chop_rounding` off, RN₅₃). When the cell of (*x*′, *y*′) has a coordinate outside 0 to
511, the result is *Q* itself. When the rule snaps to cell centres, the result is the centre of the
cell of (*x*′, *y*′). Unit: leptons.

**F3. Death-weapon damage** (claim 6).

> own or primary weapon: *D* = trunc(RZ₅₃(*w* × *φ*))
>
> game-wide death weapon: *D* = trunc(RZ₅₃(*H* × 0.5))

*w* is the weapon's damage field (integer), *φ* the type's death-weapon damage factor (single
precision), *H* the type's strength. Unit: health points.

**F4. Crew and survivor type** (claims 13 and 18). A type without the crewed flag gives no crew. For
a type with the crewed flag owned by house *h*:

| *h*'s country belongs to a side | *h*'s side number | Object armed (Terms) | Draws | Type |
|---|---|---|---|---|
| no | any | any | none | game-wide technician |
| yes | 0, 1 or 2 | no | none | side crew type of that side |
| yes | 0, 1 or 2 | yes | one percent draw | technician when the draw is below 15, otherwise the side crew type |
| yes | other | no | none | technician |
| yes | other | yes | one percent draw, whose result changes nothing | technician |

The country column and the side-number column are separate: a house whose country belongs to a side
can still have a side number other than 0, 1 or 2.

**F5. Survivor count** (claim 18). With *K* the building type's refund for its owner (S-0408
claim 1 with its F1: an integer number of credits, which includes the country cost factor, the
cost-bonus product and the refund percentage, or the type's refund override when it has one, and
which starts from the type's adjusted cost of S-0408 claim 2) and *q* the game-wide survivor divisor
of the owner's side (allied, soviet or third; an integer), doubled when the building has ever
changed owner:

> *C* = min(max(trunc(*K* / *q*), 1), 5)

*K* is the refund as it stands when the survivors are counted, for the building's owner at that
moment, not the plain cost field and not the price paid. A sold building takes the same count
(S-0408 claim 7), so the count of a sale and of a destruction agree for the same building type,
owner and factors. The refund percentage is part of *K* for every player house, as S-0408 claim 1
gives it (the original leaves it out for a computer-controlled house, whose *K* is then the larger
unreduced value; S-0408 records that rule); the neutral and special houses take the unreduced
value, as there. *K* ≤ 0 (a negative refund, S-0408 Edge cases) gives *C* = 1 by the lower bound.

*C* = 0 when the building has no crew, its type lacks the crewed flag, the owner's side is not 0, 1
or 2, or *q* = 0 (the original makes no division then). A building has no crew when its map entry
says so, or when its killing damage ignores defences (claims 18 and 24). Integer division truncated
toward zero. Unit: infantry units.

**F6. Delayed death** (claim 3; S-0302 F7 owns it). With *F* the warhead's delay-kill tick count, *A* its delay factor at
the edge (single precision), *d* the epicentre distance of the killing hit (leptons, in the number
domain S-0302 F7 gives it) and *S* the warhead's spread:

> *T* = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(*A* × *F*) − *F*) / (256 × trunc(*S*))) × *d*) + *F*))

so *T* rises from *F* at the epicentre to about *A* × *F* at the edge of the spread. The divisor uses
the spread truncated to whole cells (S-0302's whole-cell-spread switch, see Edge cases: on by default,
off, it uses *R* of S-0302 F3 instead). For *S* < 1 the divisor is 0, the quotient is infinite or not a
number, and the conversion gives *T* = 0 (S-0205 claim 7), so the building's destruction is timed 0
ticks later. Unit: ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cyborg health after crippling | 0.25 | fraction of type strength, at least 1 point | claim 2 |
| Height above which passengers die with the vehicle | 208 (exclusive) | leptons above the terrain | claim 12 |
| Bomb-marked vehicle water height | 10 (inclusive) | leptons above a water cell | claim 11 |
| Lowest crew health | 5 | health points | claim 13 |
| Game-wide death weapon damage | 0.5 | fraction of type strength | F3 |
| Debris raise | 20 | leptons | F1 |
| Contact kill distance | 256 (exclusive) | leptons | claim 17 |
| Contact kill damage | 10 | times the contact's strength | claim 17 |
| Survivor draw upper bound *g* | 2, or 1 after a planted charge, plus 6 after a change of owner | integer (bound of a ranged draw) | claim 18: the draw is R(0, *g*), and a result of 1 releases a survivor |
| Survivor count bounds | 1 to 5 | infantry units | F5 |
| Engineer chance for building factories | 25 | percent (draw < 25) | claim 18 |
| Technician chance for armed objects | 15 | percent (draw < 15) | F4 |
| Smudge choice | 50 | percent (draw < 50 gives a scorch mark) | claim 19 |
| Building explosion scatter | 64 | leptons | claim 19 (b) |
| Building explosion delay | 0 to 3 | ticks | claim 19 (b) |
| Foundation smudge scatter | 128 | leptons | claim 19 (g) |
| Spill distance | 256 to 768 | leptons | claim 22 |
| Spill unit | 1 | resource unit | claim 22 |
| Barrel fire delay | 4 to 6 | ticks | claim 23 |
| Rubble overlay | 239 | overlay registration index | claim 20 |
| Rubble sprite threshold | more than 7 | image frames of a building sprite | claim 20 |
| Lowest survivor health | 5 | health points | claim 18 |
| Crew health upper bound | 0.5 | fraction of the crew type's strength, truncated toward zero | claim 13 |
| Particle-system height limit | 10 | leptons below the terrain (the system is made only above it) | claim 19 (f) |
| Barrel probe distance | 1 | cells from the building's position | claim 23 |
| Direction scale *κ* | −π / 32767 | radians per facing unit (double) | F2 |
| Directions of the nearby point | 256 | directions | F2 |
| Map cell coordinate limit for the nearby point | 0 to 511 | cells | F2 |
| Large-smudge preference | wider than 1 and taller than 1 | cells of a smudge footprint | claim 26, large form |
| Cell that never receives a destruction smudge | both coordinates 0 | map cell | claim 26 |
| Crew escape chance at a fresh start | 0.5 | chance (decimal) | claim 27 |
| Sinking weight at a fresh start | 3.0 | weight units of the type's weight field | claim 27 |
| Survivor divisors at a fresh start | 200, 100, 300 | credits per survivor, for sides 0, 1, 2 | claim 27 |

## State and ordering

**Observable precedence of death effects.** All effects occur in the moment of the killing hit, except where a row states that an object remains in play or an effect occurs on a later turn. The table records which outcomes and random-stream draws precede others; it does not prescribe an implementation's internal work order. Draws are from the scenario random stream unless noted. The sounds a death plays draw from the unsynchronised random stream and reach no game state.

| Precedence | Kind | Effect | Draws |
|---|---|---|---|
| 0 | infantry | cyborg crippling replaces death (claim 2) | none |
| 0 | building | delayed death replaces death (claim 3) | none |
| 1 | all | debris (claim 4, F1); skipped for a bomb-marked vehicle low over water (claim 11) | F1 |
| 2 | all, exploding | passengers die (claim 5), then death weapon (claim 6); both skipped for a bomb-marked vehicle low over water (claim 11) | the detonation's |
| 3 | all | attached bomb (claim 7) | the bomb's |
| 4 | infantry | death sequence or removal (S-0107 claim 17) | S-0107 |
| 4 | vehicle | sinking (claim 10), or wake and splash (claim 11), or explosion and destruction animations (claim 9) | up to two list picks (claim 9); none for sinking or for claim 11 |
| 5 | vehicle | passengers (claim 12): they get out, or die, or stay inside as the table of claim 12 gives (a vehicle with the bomb mark that cannot crash kills them) | their scatter requests' |
| 6 | vehicle | crew (claim 13) | unit-interval value, crew type (F4), creation draw, health |
| 7 | vehicle | crate (claim 14) | the crate's |
| 8 | vehicle | crash, sinking or leaving play (claim 15); passengers still inside die at the start of a crash (claim 16) | crash: three |
| 4 | aircraft | explosion animation, then crash or leaving play (claim 16) | list pick, crash: three |
| 4 | building | contacts (claim 17), garrison out | the contacts' deaths |
| 5 | building | claim 19 (a) to (h), with claim 23 inside (c), claim 22 inside (d), and claims 24 and 18 inside (g); claim 18's cells start after the cells claim 24 used | as listed |
| 6 | building | rubble and crate, leaving play (claims 20, 21); for a type with the explodes flag or a building being sold, awaiting removal instead | the crate's (when it leaves now) |
| later turn | building awaiting removal | after the general part of its own turn: leaves its cells, second round of claims 18 and 19 (g), passengers die, leaves play, rubble and crate (claim 21) | the second round's, then the crate's |

| State | Event | Result |
|---|---|---|
| alive vehicle | killing hit, sinking conditions hold | sinking, 1 health point, in play |
| alive vehicle or aircraft | killing hit, can crash, above the terrain | crashing, in play |
| crashing | reaches the ground, was shot down | death weapon, leaves play |
| crashing | reaches the ground, was alive | destroyed by its remaining health, detonation |
| alive cyborg infantry, not prone | lethal hit that does not ignore defences | crippled, health max(1, trunc(25 % of strength)) |
| prone cyborg | lethal hit | dies (S-0107 claim 17) |
| alive eligible building | lethal hit of a delay-kill warhead | 1 health point, countdown of *T* ticks (claim 3) |
| building in a countdown | its turn at or after the end of the countdown | S-0302 claim 16's expiry hit; if it destroys the building, the death effects with no crew (claim 3) |
| alive building, type with the explodes flag or behaviour the sale's | killing hit | death effects, then awaiting removal with 0 health (claim 21) |
| awaiting removal | its own turn, still in play, not erased, no pulse effect | second round, leaves play with rubble and crate (claim 21) |

## Edge cases and quirks

- **`quirks.crash_death_weapon_twice`** (default: on, the original). An exploding aircraft fires its
  death weapon at death and again at the impact of its crash (claim 16). Off: it fires only at the
  impact.
- **`quirks.building_smudge_discarded_draws`** (default: on). Claim 19 (a) takes up to two ranged
  draws whose results are not used (none for a 2 × 2 foundation). Off: those draws are not taken. Turning it off changes every
  later draw, so it is a switch only for mods that do not need the original's random sequence.
- **Harvester cargo** (claim 9). The original computes the value of a destroyed vehicle's cargo and
  does nothing with it. Not a switch: no behaviour depends on it.
- **`quirks.death_weapon_unplaced_projectile`** (default: on, the original). The death weapon's
  projectile is never placed on the map, so the land-type, height and water tests of S-0303
  claim 21 and the warhead debris of S-0303 claim 22 use its unplaced default position (claim 6).
  Off: they use the dying object's position.
- **Departure: empty game-wide metallic debris list** (no switch). In F1 (and S-0303 F6), when the
  metallic debris list is used with *N* > 0 but is empty, the original takes the ranged draw
  R(0, −1) and then takes an entry from outside the list. Our engine takes the same draws and creates no debris, and
  the importer warns about a positive maximum debris count with nothing to throw.
- **Departure: empty splash list for a dropped vehicle in water** (no switch). Claim 11 reads the
  last entry of the splash list; with an empty list the original takes an entry from before the
  start of the list. Our engine
  shows no splash, and the importer warns about an empty splash list.
- **Departure: missing survivor type** (no switch). When a survivor is due but F4 gives no type (a
  side crew, technician or engineer type missing from the data), the original stops when it
  reaches the missing type. Our engine releases no survivor in that cell, takes no creation or
  health draw for it, and the importer warns about a missing crew type for a side with a survivor
  divisor above 0. (A vehicle's crew with no type is simply not created in the original too.)
- **Departure: debris that can never be placed** (no switch). As S-0303's departure for F6: when
  every maximum is 0 and *N* > 0, the original never finishes. Our engine stops after one full pass
  that places nothing, and the importer warns. A maxima list shorter than the types list is bad data:
  missing maxima count as 0, with a warning.
- **Delayed death with a spread below one cell** (`quirks.delay_kill_whole_cell_spread`, owned by
  S-0302 claim 16 and F7; default on, the original). On: F6 gives *T* = 0 for *S* < 1 (its divisor
  is 0, and S-0205 claim 7 converts the infinite or undefined quotient to 0), and the importer
  warns about a delay-kill warhead with a spread below 1, since the delay field then has no
  effect. Off: the countdown uses *R* of S-0302 F3, so a spread below one cell still gives a
  countdown that rises from *F*, and no warning is due.
- **`quirks.rubble_overlay_by_index`** (default: on, the original). Claim 20 places the overlay
  type at registration index 239, a number built into the original, so a mod that changes the
  overlay list changes which type that is, with no sign in the building's data. On: our engine does
  the same when the registry has an entry at that index. When it has fewer than 240 entries the
  original gives the foundation cells an overlay registration index that names no type (bad data); our engine places no rubble overlay then (a
  departure, not covered by the switch). Off: the importer binds the overlay type that has index 239
  in the unmodified base rules to the rubble role by its name, so a mod that reorders the overlay
  list keeps the same rubble; the importer warns when that type is missing. With the switch on, the
  importer warns when the overlay at index 239 is not the one at that index in the unmodified base
  rules, or is missing.
- **`quirks.barrel_probe_top_left_only`** (default: on, the original). Claim 23 probes from the
  building's position, so for a larger foundation the east and south probes fall on the building
  itself. Off: the east probe is the cell next to the foundation's east edge in the top row and the
  south probe the cell next to its south edge in the left column, each with the animation at that
  cell's point; the north and west probes are unchanged. A 1 × 1 foundation is the same either way.
- **`quirks.building_removal_second_round`** (default: on, the original). A building that awaits
  removal (claim 21) takes a second round of survivors and smudge steps when it is removed, so it
  can release up to twice the survivor count *C* in total, and it takes that round's draws. Off: it
  takes no second round; it leaves its cells, its remaining passengers die, and it leaves play with
  its rubble and crate, taking no draws but the crate's. Turning it off changes every later draw.
- **Smudge footprints beyond the map edge** (no switch). Claim 26 tests only the anchor cell for
  lying on the map. A footprint cell beyond the map edge is tested in the original as a placeholder
  cell that holds nothing and has the first tile type of the theater, so it is believed to pass
  when that tile type's tileset has the morphable flag. Our engine does the same. This can matter
  only for a smudge larger than one cell anchored on the last row or column of the map.
- **Departure: absorbed passengers beyond the last cell** (no switch). With more absorbed passengers
  than foundation cells, the original goes on past the last foundation cell and places the rest at
  unspecified positions. Our engine treats each
  passenger beyond the last cell as one that cannot be placed (claim 24), releases no survivors
  and takes no smudge steps (claim 18 has no cells left).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| rules, type section, `MaxDebris`, `MinDebris` | *X*, *Y* (F1) | integer | 0, 0 |
| rules, type section, `DebrisTypes`, `DebrisMaximums` | voxel debris types and maxima (F1) | voxel-animation list, 127-byte text limit (S-0003 claim 19); decimal-prefix integer list, 511-byte reader limit (S-0003 claim 20); independently paired as claim 28 | empty |
| rules, type section, `DebrisAnims` | debris animations (F1) | animation-name list (items as claim 28) | empty |
| rules, type section, `Explodes` | explodes flag (exploding object) | boolean | no |
| rules, type section, `VeteranAbilities`, `EliteAbilities` | exploding veterancy ability (S-0311) | ability list | none |
| rules, weapon section, `Suicide` | suicide flag (exploding object) | boolean | no |
| rules, type section, `DeathWeapon` | death weapon (claim 6) | weapon name | none |
| rules, type section, `DeathWeaponDamageModifier` | death-weapon damage factor *φ* (F3) | decimal (single) | 1.0 |
| rules, weapon section, `Damage`, `Warhead`, `Bright` | *w*, warhead and brightness of the death weapon | integer, warhead name, boolean | weapon specs |
| rules, type section, `Strength` | *H* (claim 2, F3) | integer | type specs |
| rules, type section, `Explosion` | explosion list (claims 9, 16, 19) | animation-name list (items as claim 28) | empty |
| rules, type section, `DestroyAnim` | destruction-animation list (claims 9, 19) | animation-name list (items as claim 28) | empty |
| rules, infantry type section, `Cyborg` | cyborg type (claim 2) | boolean | no |
| rules, warhead section, `CausesDelayKill`, `DelayKillFrames`, `DelayKillAtMax` | delay-kill flag, *F*, *A* (F6) | boolean, integer, decimal (held as a single) | no; 5 ticks; 1.0 (as S-0302 Import mapping, which owns them) |
| rules, building type section, `EligibleForDelayKill` | eligible for delayed death (claim 3) | boolean | no |
| rules, type section, `Naval`, `Underwater`, `Organic`, `Weight` | sinking conditions (claim 10) | boolean, boolean, boolean, decimal | type specs |
| rules, type section, `Crashable` | can crash (claims 12, 15) | boolean | no |
| rules, type section, `Crewed` | crewed flag (claim 13, F4, F5) | boolean | type specs |
| rules, type section, `Passengers` | passenger capacity (claim 13) | integer | 0 |
| rules, unit type section, `CarriesCrate` | crate carrier (claim 14) | boolean | no |
| rules, building type section, `Helipad` | helipad (claim 17) | boolean | no |
| rules, building type section, `UnitAbsorb`, `InfantryAbsorb` | absorbs units or infantry (claim 24) | boolean | no |
| rules, building type section, `LeaveRubble` | leaves rubble (claim 20) | boolean | no |
| rules, building type section, `CrateBeneath`, `CrateBeneathIsMoney` | crate beneath (claim 20) | boolean | no |
| rules, type section, `DestroyParticleSystems`, `DestroySmokeOffset` | destruction particle systems and their displacement (claim 19 f) | comma-separated particle-system name list, 127-byte text limit, no-type items omitted (claim 28); three integers in leptons | empty, 0, 0, 0 |
| rules, the combat-damage section, `DeathWeapon` | game-wide death weapon (claim 6) | weapon name | none |
| rules, the combat-damage section, `C4Warhead` | demolition warhead (claims 16, 17) | warhead name | none |
| rules, the general section, `CrewEscape` | crew escape chance (claim 13) | decimal (S-0003's decimal form, read in single precision and held as a double) | initially 0.5; a missing key keeps the value in force (claim 27) |
| rules, the general section, `AlliedCrew`, `SovietCrew`, `ThirdCrew`, `Technician`, `Engineer` | crew types (F4, claim 18) | infantry name | none |
| rules, the general section, `AlliedSurvivorDivisor`, `SovietSurvivorDivisor`, `ThirdSurvivorDivisor` | *q* (F5), for owners of side 0, 1 and 2 | integer | initially 200, 100 and 300; a missing key keeps the value in force (claim 27) |
| the inputs of *K* (F5): the type's cost, free-unit, refund-override and pad-aircraft fields, the country and cost-bonus factors, the refund percentage | *K* (F5) | S-0408's Import mapping | S-0408's |
| rules, the general section, `ShipSinkingWeight` | sinking weight (claim 10) | decimal (S-0003's decimal form, read in single precision and held as a double) | initially 3.0; a missing key keeps the value in force (claim 27) |
| rules, the general section, `MetallicDebris` | metallic debris list (F1) | animation-name list (items as claim 28) | empty |
| rules, smudge type section, `Crater`, `Burn` | the smudge type is of the crater kind, of the scorch-mark kind (claim 26) | boolean, boolean | no, no |
| rules, smudge type section, `Width`, `Height` | the smudge footprint (claim 26) | integer, integer, in cells | 1, 1 |
| theater tileset section, `Morphable` | the morphable flag of a tile's tileset (claim 26) | as S-0010's Import mapping | as S-0010's |
| the no-type spellings of a list item | an item that claim 28 leaves out | `none` or `<none>`, compared ignoring letter case, as S-0303's Import mapping gives them | |
| rules, the general section, `InfantryExplode`, `Wake` | cyborg crippling animation (claim 2), wake animation (claim 11) | animation name | none |
| map, building entry | building has no crew (F5) | map flag (map specs) | crew present |
| the animation type named `FIRE3`, a name built into the original rather than read from a key | barrel fire animation (claim 23) | animation type name | when no animation type has that name the original takes an entry from outside the animation type list (bad data): our engine creates no animation and the importer warns |

The importer warns about debris maxima lists shorter than their types lists or all 0 with a
positive maximum count, about a positive maximum debris count that falls back on an empty metallic
debris list, about an empty splash list, about a missing crew type for a side with a survivor
divisor above 0, about a delay-kill warhead with a spread below 1 while `quirks.delay_kill_whole_cell_spread` is on, and about an overlay list whose entry at index 239 is missing or differs from the base rules' (claim 20).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* on purpose-made flat test maps, in a
skirmish with one human player and one bystander computer player (glossary), at any game speed, with
test types given in the test map's override sections (named test files) and invented values. A
*recorder* that reads the game's state gives health values, object counts, object types, positions
and each object's current behaviour. The read-only recorder of the scenario random stream (glossary,
*recorder*) gives the stream's state before and after a kill, and so how many raw values the kill
took; replaying the claims' draw rules from the recorded start state predicts every outcome. Scenarios
with random draws are statistical, as stated.

*Observed areas* (glossary, *bystander computer player*, void rule). The bystander's start position
is the one farthest from the test objects. A scenario's observed area is the cells within 10 of its
test objects for D1 to D10 and D13 to D19, with these exceptions: D11 observes the aircraft's flight
path and the cells within 10 of its target and of its impact point; D12 observes the building, its
exit cells and the vehicle's route. A run is void when an object of the bystander that the scenario
does not name enters that area or acts on a test object, or when the bystander is defeated
before the observation ends. Scenarios with a neutral-house object use a test object owned by the
neutral house as the map gives it, and the same void rule.

**D1. Order inside one death (claim 1).** *Setup:* a test vehicle type with the explodes flag, a test
death weapon whose detonation (spread 2) kills a second test vehicle standing next to it, a voxel
debris count of 3, and one passenger. *Observable:* the passenger's death credit and the second
vehicle's death. *Expected:* the passenger dies (credited to the attacker of the first vehicle) and
the second vehicle dies in the same tick as the first. *Falsified if:* the passenger escapes or the
second vehicle dies in a later tick.

**D2. Cyborg crippling (claim 2).** *Setup:* a test infantry type with the cyborg flag and strength
200; a weapon that deals 1000 damage (not ignoring defences). *Observable:* the unit's health and
posture after each hit, and whether it dies. *Expected:* after the first hit it lives with 50 health
and crawls; the second hit kills it. Variant b: a type with strength 2 (expected health 1). Variant
c: a cyborg that is prone (for example after taking fire, if its type can go prone) dies on the
first lethal hit. *Falsified if:* it dies on the first hit or has another health, or the prone
cyborg of variant c survives.

**D3. Delayed building death (claim 3).** *Setup:* a stock building type made eligible; a test
warhead with the delay-kill flag, tick count 5, edge factor 7 and spread 2; lethal damage at
epicentre distances 0 and 256. *Observable:* the building's health after the hit and the tick of its
destruction. *Expected:* the building keeps 1 health and is destroyed 5 ticks later at distance 0,
and trunc(5 + 30 × 256 / 512) = 20 ticks later at 256 (each on its first turn at or after that
tick, claim 3). Variant b: a lethal hit at 256 and, 3 ticks
later, a second one at 0: the building is destroyed 5 ticks after the second hit (the shorter new
delay replaces the 17 ticks left); in the reverse order the first delay of 5 is kept. Variant c (the end of the countdown): a crewed test building type made
eligible, whose refund gives *C* ≥ 1 and which has an explosion list, killed at distance 0 by the
same warhead. *Expected of c:* at the end of the countdown the building is destroyed with its
explosion animations but releases no survivors, over 50 runs. The tick and the credit of the expiry hit are S-0302's scenario D13, parts e to h. *Falsified
if:* it dies at once, the delays differ, or a building of variant c releases a survivor.

**D4. Death debris (claim 4).** *Setup:* a test vehicle type with minimum debris 1, maximum debris 4
and no debris types; 200 kills. *Observable:* the number of debris pieces per kill. *Expected:* 1 to
3 debris pieces each time, roughly uniform. Variant b (claim 8): the same with a test infantry type.
*Falsified if:* 4 pieces or 0 pieces appear, or an infantry kill in variant b throws no debris or a
number outside 1 to 3.

**D5. Passengers of exploding objects (claim 5).** *Setup:* a test transport with the explodes flag
carrying a transport that carries an infantry unit. *Observable:* each passenger's death and kill
credit, and the tick of each relative to the detonation. *Expected:* all die, credited to the
attacker, before the death weapon's detonation. *Falsified if:* any passenger survives or dies
after the detonation. Variant b (the order among passengers): a test transport with the explodes
flag carrying two infantry units, each with its own attached trigger whose action, when it runs
after that unit's death, sets the same map variable to a different value; the variable's final
value shows which passenger's trigger ran last, and the same with a passenger carrying a passenger.
*Expected:* the holder's first passenger's trigger runs before the second's, and a passenger's own
passengers' before its own; *falsified if* the order differs. (If triggers do not run at once, the
order of their queued actions shows the same thing; S-0011 owns which.)

**D6. Death weapon (claim 6).** *Setup:* (a) a test vehicle with the explodes flag, a death weapon
of damage 100 and damage factor 1.5, killed beside a target with versus 100 %; (b) no death weapon
and no weapon: the game-wide death weapon with half of strength 300; (c) a vehicle carrying a
planted time bomb, not exploding; (d) a death weapon whose warhead throws metallic debris (a test
warhead with a positive maximum debris count), killed in the middle of the map; (e) as (a) and (c)
with a test infantry type that has the explodes flag, and an infantry unit carrying a planted time
bomb that is not exploding (claim 8). *Observable:* the
target's health loss, whether the bomb detonates, and where the debris appears. *Expected:* (a) the
target loses 150 at the centre; (b) 150 times the game-wide weapon's falloff; (c) the bomb detonates
on death; (d) the debris appears at the map's corner, not at the dying vehicle; (e) the same loss of 150, and
the bomb detonates. *Falsified if:* the amounts differ, the bomb does not detonate (for the vehicle
or the infantry unit), or (d) shows the debris at the vehicle (then the
projectile is placed, and claim 6 and the quirk change).

**D7. Vehicle animations (claim 9).** *Setup:* a test vehicle type with an explosion list of four
distinct animations; 200 kills; variant b with the explodes flag and unlimited ammunition; variant c
the same type as the first setup, a harvester carrying a full load. *Observable:* which explosion
animation appears per kill. *Expected:* each of the four entries appears roughly equally; in variant
b always the last; in variant c the same spread as the first setup. *Falsified if:* variant b shows
another entry or variant c differs from the first setup.

**D8. Sinking (claim 10).** *Setup:* a stock naval vessel whose weight meets the sinking weight,
killed on water; variant b a test naval type with weight below it. *Observable:* whether the vessel
sinks (stays in play at 1 health) or shows an explosion. *Expected:* the first sinks with no
explosion animation; the second explodes. *Falsified if:* the first explodes. Claim 11 has no
scenario (it is `unmeasured`).

**D9. Passengers of destroyed vehicles (claim 12).** *Setup:* a stock transport with five infantry,
killed (a) on open ground by normal damage, (b) by a train driving into its cell (damage that
ignores defences, S-0210 claim 10), (c) as a flying transport more than 208 leptons up; variant d: a
test crashable vehicle with passengers, killed on the ground; variant e: (a) once with the
transport owned by the human player, once by the computer player and once by the neutral house. *Observable:* how many
passengers get out and how many die; in variant d the tick of their deaths; in variant e each
escaped passenger's behaviour and position some ticks later. *Expected:* (a) all five get out and
scatter; (b) and (c) all die; (d) the passengers die when the vehicle leaves play, which is the end
of its death effects; (e) the computer player's and the neutral house's passengers start hunting (walk towards the other
player), the human player's stay where they scattered to. *Falsified if:* any differs.

**D10. Crew (claims 13, 14).** *Setup:* a stock crewed vehicle with no passenger capacity, killed 500
times; crew escape chance from stock data. *Observable:* share with a crew, crew types, crew health.
*Expected:* share within 3 standard errors of the chance; health between 5 and half the crew type's
strength; the type matches F4 (about 15 % technicians for an armed vehicle). Variant b: a vehicle
taken over by a vehicle thief always leaves that thief's type. Variant c: a test crate carrier with
crates enabled leaves a crate. Variant d: the vehicle killed once while owned by the human player,
once by the computer player and once by the neutral house. *Observable of d:* each crew unit's behaviour and position some
ticks after the kill. *Expected of d:* the human player's crew stays near where it scattered to
(guard); the computer player's and the neutral house's crew walk towards the other player (hunt).
*Falsified if:* the share lies more than 3 standard errors from the chance, a crew health lies
outside 5 to half the crew type's strength, the share of technicians among the crews of an armed
vehicle differs clearly from 15 % (outside 3 standard errors), a variant b vehicle leaves another
type, variant c leaves no crate, or in variant d the human player's crew hunts or the other two
crews stay.

**D11. Crashing (claims 15, 16).** *Setup:* a stock aircraft with a test death weapon, shot down in
flight above a target vehicle; variant b an aircraft that crashes alive (out of a landing place, if
a stock situation does this). *Observable:* where the death weapon detonates (impact point or point
of death), and in variant b the amount. *Expected:* the death weapon goes off at the impact point,
not where the aircraft died; variant b: a detonation with the aircraft's remaining health.
Variant c (claim 15): a test vehicle type with the crashable flag, killed while above the terrain
(for example in jumpjet flight): it stays in play until it reaches the ground.
*Falsified if:* the death weapon goes off in the air for a non-exploding aircraft, the detonation of
variant b has an amount other than the aircraft's remaining health, or the vehicle of variant c
leaves play before it reaches the ground.

**D12. Contacts of a destroyed building (claim 17).** *Setup:* a stock building type that produces
vehicles, destroyed while a vehicle is leaving it; a stock building type that harvesters dock at,
destroyed while a harvester is docked. *Observable:* whether each contact survives, and the kill
credit. *Expected:* the vehicles in contact within 256 leptons of the building's reference point are
destroyed with no kill credit. Variant b: a stock building type with the helipad flag destroyed
while an aircraft is docked on it: the aircraft is destroyed whatever its distance. *Falsified if:*
they survive, or the docked aircraft of variant b survives.

**D13. Survivors (claim 18).** *Setup:* a stock 3 × 3 building with the crewed flag whose cost gives
*C* = 5 (its refund for the owner at least five times the divisor of the owner's side), destroyed 300 times; variant b after a capture; variant c with a planted charge. *Observable:*
the number and types of survivors. *Expected:* per cell a chance of 1/3 (b: 1/9, c: 1/2), never more
than *C* survivors, types by F4; in variant c the survivors are ordered to attack the unit that
planted the charge, not the unit whose weapon destroyed the building. Variant d, if a stock source of
damage that ignores defences can destroy a building: no survivors at all. Variant e: a stock building
type that produces buildings, 300 kills; the share of engineers among the survivors of a building
that has not changed owner, and the behaviour of survivors of a human-controlled and of a
computer-controlled owner and of the neutral house some ticks after the kill (no planter). *Expected of e:* about a quarter of
the survivors are engineers (the engineer draw is below 25), the others by F4 (about 15 %
technicians if armed); a human owner's survivors move, a computer owner's and the neutral house's hunt. *Falsified if:* the
statistics fall outside 3 standard errors, more than *C* appear, the survivors of variant c attack
the destroyer, or the share of engineers differs.

**D14. Building destruction effects (claims 19 to 21, 23).** *Setup:* test buildings with explosion
lists and the crewed flag off (so *C* = 0), one of 1 × 1 and one of 3 × 3 foundation, each
destroyed 200 times on clear ground. *Observable:* explosion animations and their positions, smudges,
rubble overlay, crate, the tick the building leaves play, the tick each barrel fire starts, and the
number of raw values the recorder reports for the destruction. *Expected:* one explosion animation
per foundation cell within 64 leptons of the cell centres; for the 3 × 3 building exactly one smudge
per destruction (claim 19 (a)), at the centre of the cell of its position, and none at all for the
1 × 1 building, with scorch marks and craters about half each over the 200 destructions (claim 19
(a): below 50 a scorch mark). Variant b: a building type that leaves rubble leaves the rubble
overlay on every foundation cell; a type with a money crate beneath leaves a money crate. Variant c
(claim 21): a test building type with the explodes flag and rubble, with an object recorder each
tick from the killing hit, killed (i) by an object whose turn comes before the building's in the
turn pass and (ii) by one whose turn comes after it. *Expected of c:* after its death effects the
building is still in play with 0 health and its foundation cells still occupied, no rubble yet; it
leaves play, and its rubble appears, in the tick of the kill in (i) and in the next tick in (ii).
Repeat c with a building killed while its sale is in progress; the same removal and occupancy
results are expected.
Variant d:
barrels placed on the four sides of an exploding 1 × 1 test building catch fire 4 to 6 ticks
later, and with a 3 × 3 exploding test building, only barrels to its north and west (next to the top-left cell)
do, and none to its east or south. Variant e (claims 18 and 19 (g), with the recorder): the same
test building destroyed (i) with its crewed flag off and (ii) with it on and *C* ≥ 1. *Expected:*
each run's raw-value count equals the count the claims predict by replay from the recorded start
state. In (i) it contains no smudge-step draws at all. In (ii) it contains a smudge step for every
clear foundation cell, including the cells after *C* survivors have been released; a step whose
nearby point lies inside the foundation takes no smudge-type draw and leaves no smudge, and a smudge
that a step does leave lies at the cell of its nearby point. Variant f: a test building type with a destruction particle-system
list: a particle system appears at the position displaced by the type's smoke displacement.
Variant g (claim 21, the second round): a crewed 3 × 3 test building type with the explodes flag
whose refund gives *C* = 1, destroyed 300 times, with the recorder running from the killing hit
until it leaves play. *Expected of g:* survivors appear at two moments, at the kill and at the
removal turn, so some destructions release 2 survivors; smudges inside the foundation appear only
at the removal turn; each run's raw-value count matches the replay with the second round. With
`quirks.building_removal_second_round` off our engine releases at most 1. *Falsified if:* no
destruction releases 2 survivors over the 300 runs, or a survivor appears in a tick between the
kill and the removal turn. Variant h (the sale and postponement): kill a crewless selling test
building (i) before its release processing, (ii) during build-down, and (iii) just before the
processing that would pay its refund. Record its health, presence, foundation occupancy, house
credits, released units and random-stream state across the killing hit and every turn until
removal. Repeat with an electromagnetic-pulse effect that lasts beyond the kill, and separately
with the building being erased. *Expected of h:* no sale refund or further sale releases after
health reaches 0; the same occupied-foundation awaiting-removal state as c, and the second round
of g when crew is enabled. Pulse or erasure postpones removal until the first eligible own turn.
There is no second round during the wait. A refund, further sale release, early foundation
vacancy, second round during the wait, or removal before the first eligible own turn falsifies h.
*Falsified if:* a smudge appears on a cell other than the centre of the position cell while
*C* = 0, the 3 × 3 building leaves no smudge or the 1 × 1 building leaves one, the share of scorch
marks lies more than 3 standard errors from one half, an explosion animation lies more than 64
leptons from its cell centre, the raw-value count of a run differs from the replayed count, or
any other statement of the variants fails.

**D15. Spilled resources (claim 22).** *Setup:* a building type that stores resources, filled by a map setting or a start-credit
conversion (the economy specs say which exists in a skirmish; if none does, claim 22 stays
`unmeasured` and this scenario is not run), holding a known whole amount in each of two resource
types, destroyed. *Observable:* the house's stored total, and resource units by cell.
*Expected:* the house's stored total drops by that amount and the same number of resource units
appear in cells 1 to 3 cells away (256 to 768 leptons from the building's position), the units of
the earlier type in registration order being drawn first. *Falsified if:* the total drops by another
amount, the number of units differs, or a unit appears in a cell that contains no point 256 to 768
leptons from the building's position (a unit goes to the centre of the cell of its nearby point, so
the cell centre itself can lie somewhat beyond 768 leptons).

**D16. Absorbed passengers (claim 24).** *Setup:* a stock building that absorbs infantry, holding
three, destroyed. *Observable:* infantry appearing on the foundation cells. *Expected:* three
infantry appear on its foundation cells, one in each of the first three cells in the cell order.
Variant b: the same building with the crewed flag on and *C* ≥ 1, destroyed 100 times: survivors,
and smudges, appear only on the cells after the first three. Variant c: the same building owned by
the computer player, and by the neutral house: the passengers hunt. *Falsified if:* fewer appear while cells are free,
survivors or smudges appear on the first three cells, or a passenger of variant c does not hunt.

**D17. Smudge choice (claim 26).** *Setup:* named test files that replace the smudge type list
with test types: two scorch-mark types of footprint 1 × 1, one scorch-mark type of footprint 3 × 3,
and one crater type of footprint 1 × 1. A crewless test building of foundation 2 × 2 with an empty
explosion list stands on flat clear ground of a morphable tileset, so its position cell is the
anchor and the 3 × 3 footprint reaches one row and one column beyond the foundation. The
random-state recorder runs. Parts: (a) 200 destructions. (b) The same with the 3 × 3 scorch type
removed. (c) As (a) with a sloped cell or an overlay in the row or column of the 3 × 3 footprint
outside the foundation. (d) As (a) with the percent draw replayed to a crater. (e) The building
placed on a tileset without the morphable flag. *Observable:* the smudge type left at the building's
position per destruction, and the number of raw values taken between claim 19 (a)'s percent draw
and the next known draw. *Expected:* (a) every scorch mark is the 3 × 3 type (the large form
prefers it) and the type choice takes no raw value; (b) the two 1 × 1 types appear about half each,
with one raw value for the choice; (c) the 3 × 3 type no longer fits, so the 1 × 1 types appear as
in (b); (d) the crater appears and its choice takes no raw value; (e) no smudge and no raw value
for the choice. *Falsified if:* a 1 × 1 scorch type appears in (a), the shares in (b) lie more than
3 standard errors from one half, a smudge appears in (e), or a raw-value count differs from the
replay.

**D18. Defaults of the crew, survivor and sinking values (claim 27).** *Setup:* named test files
that omit the crew escape chance, the sinking weight and the three survivor divisors from every
layer, started on a freshly launched game. Parts: (a) a stock crewed vehicle with no passenger
capacity killed 500 times; (b) test naval types of weight 2.9 and 3.0 killed on water; (c) a crewed
test building of a side-0 owner whose refund is 1000 credits, destroyed 100 times, and the same for
side-1 and side-2 owners. A second run of each part follows an earlier game of the same session
whose files gave other values. *Observable:* the share of kills with a crew; whether each vessel
sinks; the largest survivor count per destruction. *Expected:* (a) a share within 3 standard errors
of one half; (b) the vessel of weight 3.0 sinks and the one of 2.9 does not; (c) at most 5, 5 and 3
survivors (1000 / 200, 1000 / 100 capped at 5, 1000 / 300 truncated); in the second runs, the earlier
game's values. *Falsified if:* any of these differs.

**D19. List items (claim 28).** *Setup:* named test files giving a test vehicle type an explosion
list `none,X` (X a test animation) and a debris-animation list `<none>,Y` with a minimum and
maximum debris count giving at least one piece, and, in a second type, a destruction-animation list
given as an empty value after a base layer that gave one entry, and in a third type, one given as
`none` after such a base layer. *Observable:* which animations appear at death. *Expected:* X always
appears as the explosion (a one-entry list); Y is every debris animation; the second type keeps the
base layer's destruction animation; the third shows none. Repeat the no-type omission forms with voxel debris types and with eligible destruction
particle-system types on a building, using one remaining named type in each list: every created
piece or system must have that type. Test a maxima list `2,,abc,3` with three named voxel types:
it pairs them with 2, 0 and 3; compare the recorder's draws and pieces with F1. Repeat with an
omitted no-type item before the first voxel type to check that omission does not remove a maxima
slot. The additional decimal-prefix and wrapping cases follow S-0003 scenario V15 and S-0303
scenario A19, whose rule claim 28 shares. *Falsified if:* a death shows no explosion where X is
due, a debris piece other than Y appears, the second type shows no destruction animation, or the
third shows one.

## Open questions

- The unplaced default position of a death weapon's projectile (claim 6): believed to be the map
  origin, which would put its warhead debris at the map's corner and take the land type of that
  corner's cell. Scenario D6, variant d, would show it.
- Vehicles whose type has death image frames (a sprite death animation) take another path at death that is
  not known (claim 9).
- What a sinking vehicle does until it leaves play, and how long it takes (claim 10).
- Whether anything in the original sets the bomb mark (claim 11). None was found, and if something
  does, claim 11 becomes measurable.
- How many absorbed passengers a stock building can hold compared with its foundation cells, which
  decides whether the departure for passengers beyond the last cell is ever reached (claim 24).
- Which events set and clear a building's charge planter (claims 18 and 24).
- Which situations of play are contacts of a building at the moment of its destruction (claim 17),
  beyond factory exits and docking.
- Q-0081 parts 3 and 4, selection only: whether a player can still select a building that awaits
  removal (claim 21). The general selection test has no health exclusion, but that alone does not
  establish that a battlefield click can still find the destroyed building. Its stay normally
  lasts less than one tick, so it matters only for a selection made in that window or while an
  electromagnetic-pulse effect delays the removal.
- Whether jumpjet types and other crashable vehicles crash by the same rules as aircraft (claim 16).
- Whether the state that suppresses survivors (claim 18, "the owner's game is not over") is set by
  the defeat of S-0414 itself or only by the destruction of all the defeated house's objects that
  follows it. The project lead's decision of 2026-10-11 (the game runs on until one alliance
  remains) does not change this: a defeated house stays defeated.
- How a building comes to hold resources in a skirmish, and whether a fractional holding can occur
  (claim 22).
- The relative order of the death effects of a building that is also exploding and holds absorbed
  passengers and resources, beyond the order listed (claim 1).

## Questions for the project lead

- Decided (2026-10-09, item 4; 2026-10-10 afternoon, item 2): AI houses follow the unit rules and
  order legality of human-controlled houses, and the neutral and special houses keep the original's
  rules for owners that are not human-controlled. Claim 25 applies both.
- Decided (2026-10-11): the game plays on to the last alliance. This spec has no rule that depends
  on the end test; claim 18's condition on the owner is unaffected (Open questions).
- Open: a passenger of a destroyed vehicle owned by the neutral or the special house. The original
  makes it join the vehicle's team when the vehicle belonged to one, and puts it on *hunt*
  otherwise. The team system is out of scope, so claim 25 proposes that no vehicle belongs to a
  team and the passenger always takes *hunt*. Should that stand, or should it get no order (as a
  player house's passenger does)? Reversible: it changes one row of claim 25.
