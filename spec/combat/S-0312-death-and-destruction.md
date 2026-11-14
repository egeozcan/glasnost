---
id: S-0312
title: Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains)
topic: combat
revision: 78
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-14
measurements: []
requests: [Q-0060, Q-0061, Q-0070, Q-0081, Q-0236, Q-0268, Q-0413, Q-0652, Q-0707, Q-2131, Q-2132,
Q-2133, Q-2135, Q-2716, Q-2793, Q-2973, Q-3000, Q-3042, Q-3086]
supersedes: []
---

# S-0312: Death and destruction (death weapons, debris, passengers, crews, survivors, sinking, crashing and building remains)

## Summary

This spec covers skirmish only; network games and campaigns are out of scope.

What happens when damage kills an infantry unit, vehicle, aircraft or building: the deaths that
are put off (crippled cyborgs, delayed building deaths), the effects every kind shares (debris,
the death weapon of exploding types, the deaths of their passengers, an attached bomb), and the
effects of each kind (vehicle explosions, sinking, passengers getting out, crews escaping,
aircraft crashing, building explosions, spilled resources, survivors, smudges, rubble and
crates), with their random draws in observable order, when a destroyed building leaves play, and
the defaults and list items of the game-wide values these rules read, and how deep death weapons
set off by other death weapons nest inside one killing hit (claim 33, Q-3000). Infantry death
sequences and corpses are S-0107's (claim 17); when a killed object leaves the turn order is
S-0203's (claim 6); the damage itself is S-0302's and S-0303's.

Against revision 33, this revision takes in the S-0312 entries of the Changes for other specs
of S-0317 (voxel debris as S-0317's pieces) and of S-0501 (the fall-back crash of an aircraft,
which starts dead without a hit).

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
- **Debris.** What a death or a detonation throws: voxel-model pieces (S-0317) with per-type
  maxima, sprite debris animations of a type, or animations from the game-wide metallic
  debris list (Formulas F1).
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
  (S-0524 claim 11).
- **Bomb mark.** A mark that an object can carry which causes landing damage when it finishes
  falling. Landing does not clear it: a survivor retains it for later landing hits and for claim
  11's low-water death effects even when no longer falling (S-0511 claim 10). Stock sources include a hovering vehicle's switch-off fall over water
  (S-0503 claim 11) and a bridge collapse dropping objects from its deck (S-0511 claim 9).
- **Absorbed passengers.** Units held inside a building whose type absorbs units or infantry (for
  example a building that converts infantry into power). The structures specs own how they get in.
- **Sinking, crashing.** States that can keep a vehicle or aircraft in play after a killing hit,
  or start without one. S-0504 claims 4 and 5 own vehicle sinking after it starts.
  S-0501 claim 9 owns flying-aircraft crash motion and sinking. S-0614 claim 11 owns
  culling-induced sinking without a killing hit. Rocket-missile interception is S-0506 claim 20.
- **Nearby point** N(*Q*, *r*, *b*). A point displaced from *Q* by a nominal distance of *r*
  leptons in one of 256 directions, chosen by the remainder *b* (0 to 255) of one raw value of
  the scenario random stream modulo 256, with the coordinate rounding and fallback of F2.
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
   moment of its killing hit, inside damage resolution and, when the damage is applied during an
   object's turn, inside that turn (S-0203 claims 1 and 5; S-0303 claim 1). When the damage is
   applied outside any object's turn, the death effects occur at that same out-of-turn place in
   the tick, before the next hit of the same detonation. The table in State and ordering gives the observable precedence of the death effects and the order in which their random draws affect the scenario random stream. For example, a passenger killed by an exploding object is already dead when the death-weapon detonation's damage is dealt. A death weapon or bomb completes its detonation before the death proceeds to effects that follow it in the table. When the
   object leaves play is S-0203's claim 6; this spec adds when buildings, sinking vehicles and
   crashing objects leave (claims 10, 15, 16 and 21). Death effects, draws and immediate deletion
   of a never-placed production item killed by defeat or surrender remain open under Q-0193:
   being off the map alone establishes neither their presence nor their absence. S-0414 claim 17
   answers the existence-count part only, not those death rules.
   - Backing: `measurable` (scenario D1; S-0203 scenario O10 for out-of-turn damage)
   - Confidence: high for the order of the shared effects and of each kind's effects; medium for the
     relative order of rarely combined effects (Open questions)

2. **Cyborgs are crippled before they die.** S-0302 (claim 15) owns this rule; it is restated here
   because it replaces the death effects. When a hit that does not ignore defences would bring an
   infantry unit of a cyborg type to health 0 or less, and the unit is not prone at that moment, it
   does not die: the game-wide infantry-explosion animation appears at its position, its health becomes the greater of 1 health point and the integer obtained by truncating one quarter of *H* toward zero, where *H* is its type's strength, and it is made to go prone and
   crawl (S-0107). A prone cyborg (crippled before, or prone for any other reason), or one hit with
   damage that ignores defences, dies normally and takes the death effects below (S-0107 claim 17
   gives its death sequence).
   - Backing: `measurable` (scenario D2; S-0302 scenario D12 for a hit that ignores defences)
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
   are unaffected. A building whose type has the explodes flag, or which is being sold, then
   awaits removal as claim 21 gives; a suicide weapon or exploding veterancy ability alone does
   not defer removal.
   - Backing: `measurable` (scenario D3; the expiry itself through S-0302's scenario D13, parts e
     to h)
   - Confidence: high (the formula and the kept countdown; the death effects at the expiry hit and
     the absence of survivors); medium (the credit at the moment of the hit, as S-0302 claim 16 rates
     it)

### Effects every kind shares

4. **Debris.** When an object whose type has a positive maximum debris count dies, it throws debris
   by Formulas F1: the count is one ranged draw, and the debris is created at the object's
   position (sprite and metallic debris raised as F1 specifies), owned by its house. The voxel
   debris of F1's first case consists of pieces of S-0317: objects of play, created there with the draws of
   S-0317 claims 3 and 4, whose start positions, flight, impacts and end are S-0317's. The house
   that owns a piece changes neither its ground-impact hits nor the area damage of its end: both
   carry no attacking house (S-0317 claims 1, 7 and 11). Everything else a piece does, its
   further pieces included, is S-0317's, with the ratings S-0317 gives it.
   - Backing, by part:
     - The count and the creation points: `measurable` (scenario D4, statistical).
     - The ownership of sprite and metallic debris by the object's house: `measurable`
       (scenario D4, its metallic and sprite repeats).
     - The pieces' ownership by the object's house: `unmeasured`: a piece's house changes no
       outcome in play (S-0317 claim 1), and the hits below carry no house at all.
     - A voxel piece's creation draws: their count `measurable` (S-0317 scenario V1); the use of
       each draw rests on S-0317 claims 3 and 4, with the backing those claims give it.
     - The absence of an attacking house in a piece's ground-impact hits: `measurable` (S-0317
       scenario V5 parts f and g).
     - The absence of an attacking house in its end's area damage: `measurable` (S-0317 scenario
       V8), resting on S-0317 claim 11.
   - Confidence: high for the count and the creation points; high for the ownership of sprite
     and metallic debris; medium for the pieces' ownership; medium for the draws of a voxel
     piece's creation, as S-0317 claims 3 and 4 rate them until S-0317 scenario V1 runs; medium
     for the piece's ground-impact hits carrying no attacking house, until S-0317 scenario V5
     parts f and g run, and for its end's area damage carrying none, until S-0317 scenario V8
     runs.

5. **Exploding objects kill their passengers.** When an exploding object dies, every passenger it
   carries dies with it, and each death is credited to the attacker of the killing hit (score and
   experience belong to their specs). The passengers show no death effects of their own and take no
   draws for those suppressed effects. Tagged passengers still receive destruction notifications
   under S-0011 claim 19; actions they fire complete with their effects and draws before that
   passenger is removed. Their deaths come before the death weapon goes off (claim 6), so a passenger is already
   dead when the detonation's damage is dealt. The passengers die one after another in the order
   they are held, and the passengers of a passenger die before it does. That order is observable
   through reactions to their deaths, including attached triggers and their actions' effects
   and draws (S-0011 claim 19).
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
   weapon at death; a non-rocket crashing aircraft fires one when it hits the ground (claim 16).
   Rocket interception follows S-0506 claim 20 instead. The type's
   death-weapon damage factor *φ* (F3) applies to the primary weapon too, not only to a death
   weapon of the type's own. When the chosen weapon has a positive radiation level, its
   detonation leaves a radiation field at the cell of the dying object's position, or adds
   to the field centred there (S-0525 claims 1 and 2). This uses the detonation point, not
   the unplaced projectile's position.
   - Backing: `measurable` (scenario D6; variant d for the debris position,
     S-0525 scenario RF9 for radiation-producing death weapons)
   - Confidence: high for the choice and the formula; medium for the multi-turret case; low for
     the position used by the land-type tests and the debris

7. **Attached bombs at death and removal.** A planted bomb still attached to a dying object
   on the map detonates at the end of the shared effects, whether or not the object is
   exploding (S-0519 claim 5). The bomb is detached before its damage is dealt, so a carrier
   killed by that detonation does not detonate the same bomb again. If death sets off an
   attached bomb while its carrier is off the map, the bomb is removed with no detonation,
   animation or damage (S-0519 claim 6). Removal of the carrier other than by death disarms
   its bomb with no damage, animation or credit (S-0519 claim 7).
   A building ownership change likewise removes its planted bomb without detonation, and
   that bomb never goes off later, except on a garrisonable type, where the bomb stays and
   keeps counting down (S-0509 claim 13).
   The detonation at the end of the shared effects needs a death by a killing hit. A carrier
   destroyed without a hit, as by the fall-back crash of S-0501 claim 46, has no shared death
   effects, so its planted bomb does not detonate at the destruction. The bomb stays attached
   while the carrier stays on the map: during the aircraft's fall its timer and the detonate order
   still set it off (S-0519 claim 4 for the timer, claim 9 for the detonate order, claim 5 for
   the detonation). If it has not gone off by the impact, it is removed then
   with no effect, as for a carrier removed other than by death (S-0519 claim 7, under S-0501's
   `quirks.crash_drops_planted_bomb`).
   - Backing: `measurable` (scenario D6, variant c; S-0519 scenarios T7 to T9 for lethal
     detonation, off-map deaths and non-death removal; S-0509 scenario E8 for ownership
     changes; S-0501 scenario AC29 part i for a bomb kept through the fall-back crash's start, removed
     without effect at impact, and set off by a detonate order during the fall). A bomb whose
     timer comes due during that fall is `unmeasured`, as S-0501 claim 46 gives: no normal input
     places the crash start against the bomb's due tick.
   - Confidence: high; medium for the carrier destroyed without a hit, as S-0501 claim 46 rates
     the planted bomb's handling until AC29 part i runs; medium for its timer during the fall

8. **Infantry.** A killed infantry unit takes the shared effects of claims 4 to 7 and then the death
   sequence or removal of S-0107 claim 17, which also owns its corpse. Nothing further belongs to
   this spec.
   - Backing: `measurable` for the debris, the death weapon and the attached bomb of an infantry unit
     (scenario D4 variant b, D6 variant e; S-0107 scenario M10 for the rest); `unmeasured` for the
     passengers of an infantry unit (no scenario here gives an infantry unit passengers, so none
     shows the passenger part of claim 5 for infantry)
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

10. **Sinking.** For types without death image frames (Open questions), a killed vehicle
    of a naval type that is not an underwater type and not organic,
    whose type's weight is at least the game-wide sinking weight, is on a water cell and
    does not carry the bomb mark, sinks
    instead of showing claim 9's explosion and destruction animations: neither animation
    appears and neither list-pick draw is taken. Its health is restored to 1 health point,
    but staying in play is subject to claim 15's crashable rule, even on its first killing
    hit. A crashable vessel at the water surface cannot start a crash and is removed at the
    end of that hit's effects; it does not survive for a sinking descent. This removal adds
    neither claim 9's animations or picks nor claim 16's crash-start tumbling draws.
    Applicable shared effects and their draws (claims 4 to 7), passenger handling (claim 12),
    crew (claim 13) and crates (claim 14) still apply. A non-crashable vessel stays in play
    at 1 health point. S-0504 claims 4 and 5 own descent, duration, removal and behaviour
    only for a vessel that remains in play.
    Separately, a grappling parasite's cull starts its host sinking without a killing hit,
    regardless of the host type's weight: health is unchanged and no damage is delivered
    (S-0614 claim 11). This is not a killing hit that takes this spec's death effects.
    A later killing hit during descent instead follows claim 32, including when the cull's
    host fails the ordinary sinking conditions.
    - Backing: `measurable` (scenario D8, S-0504 scenarios N5 and N6 for sinking after it starts,
      S-0614 scenario P6 for culling)
    - Confidence: high for the killing-hit condition, first-hit crashable removal, omitted
      vehicle animations and picks, absent crash-start draws, and the cull distinction; sinking after
      it starts inherits S-0504 claims 4 and 5's confidence

11. **Vehicles with the bomb mark that die low over water.** A vehicle that carries the bomb mark
    (Terms) and dies low over water, as the water test below gives it, skips claims 4 to 6 and 9
    (its attached bomb, claim 7, still goes off). It shows instead the game-wide wake animation
    and the last entry of the game-wide splash list (S-0303, F5) at its position. Claims 12 to 15
    then apply as for any other destroyed vehicle, except that its passengers do not get out:
    claim 12 kills them, or keeps them inside until the crash when the vehicle's type can crash.
    The wake and splash animations take the place of claim 9's animations in the order of death
    effects (State and ordering). Stock sources of the mark include a hovering vehicle whose
    switch-off spin ends over water less than 104 leptons above the surface (S-0503 claim 11), and
    a vehicle dropped from a collapsing bridge deck (S-0511 claims 9 and 10). The hover's strict
    104-lepton fall-start threshold is separate from this claim's inclusive 10-lepton
    death-effects threshold; receiving the mark alone does not suppress the ordinary death effects
    at greater heights. Landing does not clear the mark (S-0511 claim 10), so a survivor's later
    death still takes this claim's effects when its water and height conditions hold, even when it
    is no longer falling.

    **The water test (Q-2793).** The vehicle passes it when, at its death, its height above the
    surface (glossary) is 10 leptons or less and the land type of the cell at its position is
    water. Both parts are read at the killing hit, and the same test decides both the skip of
    claims 4 to 6 and the wake and splash in place of claim 9.
    - *Height.* For a vehicle on a bridge deck the height is measured from the deck, otherwise
      from the terrain of its cell. The test has no lower bound: a negative height passes it.
    - *Cell.* The cell's land type is the one S-0211 claim 5 gives it (the same test as claim
      10's), whatever cell layer the vehicle is on. A high-bridge cell over water whose overlay
      types keep the ground's land type is a water cell (S-0211 claim 9), and that holds for a
      vehicle on its deck as well as for one under it. A low-bridge cell has the land type
      S-0211 claim 9 gives it.
    - *On a deck over water.* So a marked vehicle that dies standing on an intact bridge deck over
      water, at height 0 above the surface, takes this claim's effects in the original: wake and
      splash at its position on the deck, no debris, no passenger deaths of claim 5, no death
      weapon and no claim 9 animations, and its passengers do not get out (claim 12).
      `quirks.bomb_mark_water_death_on_deck` (Edge cases) owns this case.
    - Backing: `measurable` (scenario D8's marked-water variant; S-0503 scenario H7 and S-0511
      scenario B3 for stock falls and water-death animations). H7 does not establish passenger
      loss or the exact fall-start boundary; D8's variant supplies the passenger test.
      `measurable` by D8's marked-deck variant for a later death after surviving a landing, for
      the height read from the deck and for the land type of the cell under a high bridge.
      The exact 10-lepton death threshold remains `unmeasured`: the falls end at the surface and
      the deck run stands at height 0. A negative height with the mark is `unmeasured`: no setup
      here holds a marked vehicle below the surface at its death.
    - Confidence: high for retention after landing and the absence of an active-falling
      requirement; high for the height reference off any deck and for the cell's land type; medium
      for the reading from a bridge deck, the one level this spec gives it (claims 12 and 15 use it
      at the same level): no run has read it, and D8's marked-deck variant, part a, read with the
      marked-water variant, can fail it (part a's falsifier); medium for the missing lower bound
      (`unmeasured`, above); medium for the stock sources; low for the low-water death-effects
      rule, including the effects it gives a marked vehicle that dies on a deck over water

12. **Passengers of a destroyed vehicle.** Passengers that claim 5 has not killed are handled after
    the vehicle's animations. What happens to them depends on the vehicle as follows (the rows
    exclude one another):

    | Vehicle's height above the surface | Type can crash (claim 15) | Bomb mark (Terms) | Passengers |
    |---|---|---|---|
    | more than 208 leptons | any | any | all die, credited to the attacker |
    | 208 leptons or less | yes | any | stay inside; they die credited to the attacker when it starts to crash, or without attacker credit at removal when it is on the ground and cannot crash (S-0507 claim 16) |
    | 208 leptons or less | no | yes | all die, credited to the attacker |
    | 208 leptons or less | no | no | get out, as below |

    The height is the vehicle's height above the surface (glossary), as in claim 11: on a bridge
    deck it is measured from the deck, so a vehicle standing on a deck is at height 0 and its
    passengers follow the rows for 208 leptons or less. The bomb-mark column tests the mark alone,
    not claim 11's water test (Q-2793): a marked vehicle whose type cannot crash and which dies
    at 208 leptons or less above the surface kills all its passengers, credited to the attacker,
    wherever it dies: on land, or (medium, `unmeasured`, Backing boundaries) on a bridge deck or
    more than 10 leptons above water.

    In the last row each passenger in turn gets out: it appears at the vehicle's horizontal
    position, at the height of the ground surface at the centre of the vehicle's cell (on a bridge
    deck, at a point the bridge specs give, when the vehicle was on one), facing the vehicle's body
    facing rounded to the nearest of 256 steps (S-0204 F4 with *n* = 256), and receives a scatter
    request with no threat point, forced and not move-anywhere (S-0208). A passenger dies instead,
    credited to the attacker, when the killing damage ignores defences, when it cannot enter the
    vehicle's cell (an entry outcome other than clear or moving blocker, S-0209 claim 4), or when
    it cannot be placed.

    **Dying vehicle excluded (Q-0707).** The dying vehicle itself does not obstruct these
    escape entry tests: it contributes neither a listed blocker nor its vehicle-present flag
    (S-0209 claim 4). Other occupants still count normally; this is not a general exemption
    for destroyed objects, nor a rule for ordinary unloading. Thus a stationary transport
    alone on otherwise enterable open ground does not prevent its passengers escaping.
    For comparison, if that vehicle were counted as an ordinary allied occupant, with no
    movement in progress, no destination and no entry override, it would give *stationary
    friendly*, outcome 6 (S-0209 claim 4, row WP16), not *clear* or *moving blocker*.
    In an otherwise clear cell that outcome would kill the passenger under this claim's
    entry gate. Other vehicle states use S-0209 claim 4's corresponding rows; there is no
    single counterfactual outcome for every destroyed vehicle.

    Any passengers still aboard at final removal die recursively without attacker credit
    under S-0507 claim 16, with the suppressed passenger death effects and destruction
    notifications of claim 5. This cleanup does not change credit for passengers already
    killed during the killing hit or at crash start.

    A passenger that has got out is given no order beyond the scatter request when its house is
    human-controlled. When its house is computer-controlled, the original also puts it on the *hunt*
    behaviour (S-0215), unless the destroyed vehicle belonged to a team, which the passenger then
    joins instead. Which of these our engine applies depends on the owner (claim 25): a player
    house's passenger gets no further order, a neutral or special house's passenger is put on
    *hunt*, and the team join never applies.
    This refines S-0203 claim 6e, which refers to this claim for whether passengers of a destroyed
    object die or get out; S-0203 claim 3 owns the in-play status of passengers of open-topped
    transports.
    - Backing: `measurable` (scenario D9; variant h for the height read from a bridge deck, which
      it confirms when a passenger gets out and leaves undecided when all five die; D8's
      marked-deck variant, part b, for the mark-only test on land; variant e for the orders,
      variant g for the scoped exclusion and ordinary stationary-vehicle comparison; S-0507
      scenario T14 for removal cleanup); `unmeasured` for the team join (the original's team system
      is outside our scope and has no counterpart in our engine, so nothing of ours depends on it,
      and no test map is planned for it)
    - Confidence: high for the height rule; medium for its height read from a bridge deck,
      claim 11's reading at claim 11's level (D9 variant h can confirm its use here but not
      fail it); high for the bomb-mark column's mark-only test on land; medium for that test
      on a bridge deck and more than 10 leptons above water (`unmeasured`, Backing
      boundaries); high for the escape, the dying-vehicle exclusion, the conditional
      stationary-friendly comparison and uncredited removal cleanup; medium for the
      entry-outcome condition and the facing; medium for the orders; low for the team join

13. **Crew.** After its passengers, a destroyed vehicle may leave a crew infantry unit at its
    position, owned by its house:
    - Hijacking records the vehicle thief's infantry type as the vehicle's hijacker type
      (S-0524 claim 11). A vehicle with a hijacker type always leaves one infantry of that type,
      with no chance test and no draw for its type.
    - Otherwise, when the killing damage does not carry the no-escape marker, the type has the
      crewed flag and has no passenger capacity, a chance test "less than" with the game-wide crew
      escape chance (one unit-interval value, S-0202 claim 6) decides, and the crew's type is chosen
      by Formulas F4.
    A crew that is due takes S-0202's creation draw (claim 10) whether or not it can then be placed.
    It is put down at the vehicle's exact position, after the dying vehicle has given up its place
    in the cell's occupancy record, so the vehicle never refuses its own crew.
    - When the height of that position is the terrain surface height there, the crew's standing
      position is chosen by S-0208 claim 3's occupant-aware table on the cell's ground record, with
      the vehicle's exact position as the desired point. Its class is S-0208 claim 2's. Only a
      centre-class choice that the table's first row does not refuse takes a draw (one ranged draw
      over 0 to 3). In a cell outside the usable area of the map the choice ignores the cell's
      occupants and takes no draw: the crew stands at its class's own position. This exception
      belongs to the crew's placement, not to S-0208 claim 3's rule for walking into a cell.
    - When the height differs (for example a vehicle on a bridge deck), no position is chosen and no
      draw is taken: the crew is put at the vehicle's exact position.
    - The crew's placement then makes one entry test for that cell: the single placement test
      that S-0209 Terms give for an ordinary return placement, under S-0209 claim 4's table for an
      infantry mover, with no approach direction and no source elevation. Only *clear* accepts the
      crew. Every other outcome refuses it at once, *moving blocker* included, with no waiting,
      re-routing, scatter request, gate opening or attack. So the crew's test is narrower than the
      escape gate of claim 12, which also passes *moving blocker*. A survivor (claim 18) differs:
      it is put at the position the table chose with no entry test, so no occupant or cell outcome
      refuses it there, while a crew is refused there unless its entry test gives *clear*.
      A crew that gets no position, or whose placement is refused, never enters play and takes no
      health draw. A placed crew's health is a ranged draw over 5 to half its type's strength
    (truncated toward zero). It faces map north, receives the same scatter request as escaping
    passengers (claim 12), is put on the *guard* behaviour when its house is human-controlled and
    on the *hunt* behaviour when it is computer-controlled (S-0215; which one our engine applies
    depends on the owner, claim 25), and keeps the vehicle's trigger tag when that tag is of a kind that passes
    on (trigger specs). The draws come in the order: chance test (not for a hijacker type), crew type
    (F4, when it takes a draw), creation, position (only for a centre-class choice that is not
    refused), health (only for a placed crew).
    - Backing: `measurable` (scenario D10, statistical for the chance; variant d for the
      behaviour; variant f for the standing position and its draw). `unmeasured` for a cell
      outside the usable area and for a position whose height differs from the surface: variant f
      runs on flat ground inside the usable area, and no scenario here destroys a crewed vehicle
      on a bridge deck or outside the usable area. `unmeasured` for the entry test (clear only)
      and for the survivor's differing from it: variant f's cells give the crew *clear*, and no
      scenario here destroys a crewed vehicle in a cell whose entry outcome for the crew is
      another one while the standing-position table still gives a position.
    - Confidence: high for the chance and the health draw; medium for the hijacker case, the tag,
      the behaviour, the standing position and its two exceptions; medium for the entry test
      (only *clear* accepts the crew) and for the survivor's making none

14. **Crates from crate-carrying vehicles.** A destroyed vehicle whose type carries a crate leaves a
    crate with random contents on a free cell near its position, when the game's crate setting
    allows crates. S-0520 claim 5 owns its contents and collection rules.
    - Backing: `measurable` (scenario D10, variant c; S-0520 scenario CR3 for contents)
    - Confidence: high for random contents; low for the exact condition on the game settings

15. **Crashing vehicles and removal.** A destroyed vehicle whose type can crash, and whose height
    above the surface is positive, starts to crash (claim 16) and stays in play until it hits the
    ground. For a jumpjet, S-0502 claim 18 says when its crash ends; a held-up crash can keep it
    in play indefinitely, as long as nothing else ends it, which claim 16 and S-0502 claim 18
    rate medium. On a bridge deck that height is measured from the deck (claim 11's reading), so a
    crashable vehicle standing on a deck cannot start a crash and is removed as below. A vehicle
    already sinking and whose type cannot crash stays in play, including the zero-health
    aftermath of claim 32. For a crashable type, failure to start a crash instead removes it
    immediately, whether sinking started at this killing hit (claim 10) or before it
    (claim 32). This includes a first killing hit on a crashable surface vessel at height
    0 above water: the restored health does not prevent removal. Only a vessel retained
    after these effects proceeds to S-0504 claim 4's descent. Every other destroyed vehicle
    leaves play at the end of its death effects, in the same moment (S-0203 claim 6a). Types with death image
    frames retain the separate unresolved death path in Open questions.
    - Backing: `measurable` (scenarios D11, its variant h for a deck, and D8's first-hit crashable
      variant)
    - Confidence: high, except for a vehicle on a bridge deck: medium there, claim 11's reading at
      claim 11's level (D11 variant h, with its control, can fail it); and medium that nothing
      else ends a held-up jumpjet crash, as S-0502 claim 18 rates it

### Aircraft and crashing

16. **Aircraft deaths and crashes.** A killed aircraft takes the shared effects, then, when its
    type's explosion list is not empty, shows one entry of that list at its position, chosen by
    one list pick. When the list is empty, this aircraft-specific explosion creates no animation
    and takes no draw: there is no fallback animation or fallback list. This rule is the same
    in flight and on the ground. It does not suppress animations or draws from the shared death
    effects, crash start or later impact. The aircraft then starts to crash if it is above the
    terrain; otherwise it leaves play at once, and any passengers still aboard die without
    attacker credit under S-0507 claim 16. An object that starts to crash loses all
    its passengers (credited to the attacker; for the fall-back crash, to nobody, as the last
    paragraph of this claim states) and, unless it is infantry, takes three draws for its
    tumbling (one unit-interval value, one ranged draw over 0 to 1 that chooses the sense, one more
    unit-interval value). For an aircraft using ordinary flying movement, S-0501 claim 9 owns
    crash motion: its growing fall counter supplies a downward contribution, not necessarily
    the net descent, because ordinary altitude adjustment can still apply. That claim also owns
    its sinking branch. A crashable vehicle using jumpjet movement uses the crash motion of
    S-0502 claim 18, with its magnetic-lift qualifications. For any object using jumpjet
    movement, the crash start also makes S-0502 claim 18's empty destination assignment at
    once, inside the hit. A committed jumpjet infantry unit then reroutes with its eligible
    standing-position draw over 0 to 3; a jumpjet vehicle's reroute takes no draw. S-0502 claim 3's safety
    departure guarantees the next movement update for a pending crash above the surface,
    even without a move order: a dead hovering unit cannot remain suspended merely because
    its order ended. This guarantee is an engine design rule, not established original
    behaviour. A raised look-ahead ground reference can still hold the fall up in the
    original (S-0502 claim 18, `quirks.jumpjet_crash_lookahead`). The ordinary non-rocket impact rules below still belong to this claim. Magnetic drops are
    different: S-0607 claims 16 to 19 own a dropped seized unit's impact, including clear-cell
    survival and sinking. Only a vehicle whose type can crash retains this claim's ordinary
    crash-end exception (S-0607 claim 18); a dropped seized aircraft does not (S-0607 claim 19).
    The magnetic drop itself is not this claim's crash start: it neither kills passengers nor
    takes tumbling draws (S-0607 claim 13). For other crashable movement
    modes, the owning movement spec supplies the motion. The following impact rules also exclude rocket missiles: S-0506 claim 20 owns their
    interception outcomes, with detonation governed by S-0506 claim 19. An intercepted rocket
    above the terrain starts to crash but keeps rocket-flight movement and its full detonation
    damage on the next positive-speed movement attempt, including a rejected map-edge attempt.
    At zero speed the health-triggered detonation waits for such an attempt. On the ground it
    is removed without a rocket detonation. Interception neither restarts nor suspends the
    rocket's phase timer, and adds no crash-fall displacement: pause and tilt keep its position
    fixed, raise keeps its ordinary rise, and positive-speed motion follows S-0506 claim 20.
    For an ordinary non-rocket crash within this claim's scope, when it reaches the ground:
    - if it was shot down, its death weapon goes off at the impact point (claim 6, even if it already
      went off at the moment of death because the aircraft was exploding), and it leaves play,
    - if it was still alive (a crash started without a kill), it is destroyed by damage equal to its
      remaining health with the demolition warhead, ignoring defences, with the no-escape marker and
      no attacker (so its kill is credited to no one); an explosion
      animation chosen by S-0303 F5 for that amount appears, and a detonation with that amount, the
      demolition warhead, no source object and no source house follows at the impact point.

    The fall-back crash of S-0501 claim 46 is not a crash started without a kill in this sense. It
    sets the aircraft's health to 0 at its start without a hit, so the aircraft falls and strikes
    the ground as one shot down: its death weapon goes off at the impact point and it leaves play.
    Its destruction is credited to nobody and counted among its house's losses unless its type
    is not scored (S-0414 claim 21), and it takes none of the other killing-hit effects, neither
    at the start nor at impact: no debris (claim 4), no deaths of passengers by claim 5, no
    detonation of an attached bomb by claim 7, and no animation from this claim's explosion-list
    pick. The crash start's effects above apply as S-0501 claim 46 states, with one difference in
    credit: its passengers are lost and credited to nobody, since the crash has no attacker, and
    the three tumbling draws are taken.
    - Backing: `measurable` for the death effects and dead-object crash cases covered by scenario D11, variant f for an empty explosion list, S-0501 scenario AC5 for dead flying-aircraft crash motion, S-0507 scenario T14 for uncredited ground-aircraft passenger cleanup, S-0522 scenario P9 for the attacker credit of the passengers lost at the crash start of an aircraft that was shot down (P9 observes only that case), S-0502 scenario J12 for jumpjet crash motion, S-0607 scenarios L3 to L6 for magnetic-drop outcomes, and S-0506 scenario R13 for rocket interception, unchanged phase timers and absence of additional crash fall. A crash that starts while the aircraft is still alive is `unmeasured`: D11 variant b is conditional and supplies no measurable backing until a reachable setup is established. Aircraft sinking retains the backing limitations of S-0501 claim 9. The original jumpjet idle-crash aftermath remains `unmeasured` under S-0502 claim 3 (diagnostic J15): the crash start's destination clear of S-0502 claim 18 can reroute a committed unit, so the otherwise idle case has not been isolated. The destination clear and the held-up crash have S-0502 claim 18's backing (S-0502 scenarios J12 and J31). The pending-crash update guarantee is design, not an oracle observation.
      The fall-back crash of S-0501 claim 46 is `measurable` through S-0501 scenario AC29 part f
      (its health, fall and impact, the score screen's counts, and the absence of debris and of
      the explosion-list animation), and its three tumbling draws through scenario D11 variant
      g. The loss of its passengers rests on S-0501 claim 46, with that claim's backing and
      rating; no run known to S-0501 or to this spec reaches a loaded aircraft of that kind,
      so the loss is in effect `unmeasured` here as well, for the reason below. That they are credited to nobody is this claim's own statement, following from
      the crash having no attacker (S-0501 claim 46): `unmeasured`, since it shows only for a
      pad-bound armed aircraft that carries passengers, and no set-up is known that loads
      passengers into such a type before its run.
    - Confidence: high for the explosion-list choice, the absence of an animation and draw for an
      empty list, their independence from height, uncredited passenger cleanup at removal, and
      the attacker credit of the passengers lost at the crash start of an aircraft that was shot
      down, which S-0522 scenario P9 observes (S-0522 claim 15).
      Rocket interception inherits S-0506
      claim 20's confidence; jumpjet motion is high for its fall, spin and the held-up crash's
      reference, height per update, speed reset and held-up condition, and medium that nothing else ends a
      held-up crash, as S-0502 claim 18 rates them; high for the jumpjet crash start's
      destination clear at the hit, with the infantry reroute draw (S-0502 claim 18),
      medium for the non-rocket impact rules; low for the double death weapon; medium for the
      fall-back crash of S-0501 claim 46, as that claim rates its health 0 at the start, its dead
      fall, its counts and the absent killing-hit effects until AC29 part f runs, and its
      tumbling draws until D11 variant g runs; medium for its passenger loss, as S-0501 claim 46
      rates it; medium for those passengers being credited to nobody, which no run observes

### Buildings

17. **What a destroyed building does to the objects in contact with it.** When a building is
    destroyed, this claim treats the objects it held in contact just before the destroying hit
    (glossary *contact*, for example a vehicle leaving its factory or docked at it), except for
    its bunkered vehicle. That vehicle is let out first under S-0606 claim 13 and is neither
    destroyed nor sent away by this claim; it has no immunity to other damage from the attack
    or death effects. A vehicle still entering, with contact but no bunker link, remains an
    ordinary contact for this rule. Each included contact is destroyed by damage of ten times its own strength with the
    demolition warhead, ignoring defences, with the no-escape marker and with no attacker, when the
    spatial distance from the building's reference point (S-0204 claim 4) to the contact's position
    is less than 256 leptons, or whatever the distance when the building is a helipad. Any other
    contact is only released. Garrisoned infantry are evicted under S-0508 claims 10 to 12,
    without hunting or an inside fallback (S-0508 claim 13). With no qualifying exit cell,
    the original removes every occupant without killing it: no occupant death, kill or survivor.
    S-0508 owns the eviction quirks and their alternatives, including keeping occupants inside
    when its no-exit-removal quirk is off.
    - Backing: `measurable` (scenario D12; S-0606 scenario K5 for bunker destruction,
      S-0508 scenarios G7 and G8 for eviction)
    - Confidence: high (the distance rule, the helipad rule and the damage); low (which situations
      of play are contacts at that moment); eviction inherits S-0508 claims 10 to 12's confidence

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
    then be placed. Its standing position in that cell is chosen as for an absorbed infantry
    passenger (claim 24): the desired point lies 36 leptons from the cell centre along the map y
    axis, so its class is centre, and S-0208 claim 3's table on the cell's ground record gives the
    position, with one ranged draw over 0 to 3 unless the table's first row refuses the cell (a
    vehicle in it, or the object-present flag without an open gate), which takes no draw. The
    destroyed building itself never refuses its survivors: the building-present flag plays no part
    in that table, in either round (claim 21). The chosen position is kept: putting the survivor
    down makes no second choice, takes no further draw and makes no entry test. A survivor of
    claim 21's second round has its position chosen in the same way. A survivor for which
    the table gives no position never enters play, takes no health draw and does not count against
    *C*. A placed survivor's health is a ranged draw over 5 to its type's strength, and it receives
    the same scatter request as escaping passengers (claim 12). Its behaviour then depends on the
    building's charge planter (Terms): when the building has one and the owner is not allied with
    it, the survivor is ordered to attack it. Otherwise the survivor is put on the *move* behaviour
    when its house is human-controlled and on the *hunt* behaviour when it is computer-controlled
    (S-0215; which one our engine applies depends on the owner, claim 25). The attacker of the
    killing hit plays no part in this order.

    *No crew.* When the killing damage ignores defences, the building counts as having no crew
    (F5), so it releases no survivors at all, and its absorbed passengers die instead of coming
    out (claim 24). That is the only way a building comes to have no
    crew. No field of a map's building entry sets the state, and no other event does: a building
    has crew whenever it is created (placed from a map, built or deployed), and an owner change, a
    repair or an upgrade cannot carry the no-crew state, which arises only at the building's
    destruction. So a sale's survivors (S-0408 claim 7), a vehicle's crew (claim 13) and a capture
    are never affected by it. The crewed flag is a separate field of the type. A type whose section
    leaves it out is not crewed (Import mapping), so its *C* is 0 by F5.
    - Backing: `measurable` (scenario D13, statistical; variant d for the damage that ignores
      defences; variant e for the engineer draw and the behaviours; variant g for the standing
      position and its draw, and for a survivor with no free position; variant h for the crewed
      flag's absent-key default on building types)
      - `unmeasured` for the no-crew state's lifecycle (no field of a map's building entry or other
        event sets it; a building placed from a map, built or deployed has crew; an owner change,
        a repair or an upgrade does not carry the state): no part of D13 deploys the building, or
        repairs or upgrades it before the death, and no finite set of runs shows that no other
        event sets the state
      - `unmeasured` for the absence of an entry test when the survivor's cell holds another
        house's infantry: variant g's cell holds infantry of the building's own house, which an
        entry test would not refuse either, so the run does not tell the two apart
      - `unmeasured` for the crewed flag's absent-key default on vehicle, infantry and aircraft
        types, and on walls, gates and types infantry can occupy: variant h uses two plain
        building types only
    - Confidence: high for the counts, chances, types and the draw order, and for the crewed
      flag's absent-key default on plain building types; medium for the absence of an entry test
      when the survivor's cell holds another house's infantry; medium for the no-crew state's
      lifecycle and for that default on the other kinds of type; medium for the standing position
      rule, which is claim 24's and rated medium there; medium for the charge
      planter as the target of the attack order (which events set and clear the remembered planter
      is not known) and for the *hunt* behaviour; low for the *move* behaviour of a survivor
      without an attack order

19. **Other effects of a building's destruction.** In addition to the shared effects, the
    following outcomes have the listed observable precedence and scenario-random-stream draw order; this
    order does not prescribe an implementation's internal work sequence:
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
    - (f) creates one smoke-kind particle system from its type's destruction particle-system
      list, when no earlier destruction-smoke system of this same building remains in play,
      at least one entry is eligible and it stands higher than 10 leptons below the terrain.
      Only smoke-kind entries are eligible; damage smoke, natural smoke and other emitters do
      not prevent this creation (S-0616 claim 1). One ranged draw R(0, *m* − 1) over the *m*
      eligible entries picks the system, counted from the end of the list (*r* = 0 picks the
      last eligible entry), placed at its position moved by its type's destruction smoke
      displacement. With no eligible entries, no system is created and no selection draw is
      taken. The system starts empty, with the building as owner object and no owner house
      (S-0616 claim 1). Removal of the building marks it, stopping further emission
      (S-0616 claim 4). Thus a building removed immediately under claim 21 makes destruction
      smoke that never emits and leaves play in its first turn. A building awaiting removal,
      including one destroyed while being sold, instead permits emission under S-0616 claim 9
      until its removal; any particles already made then finish normally,
    - (g) releases its absorbed passengers (claim 24), then its survivors (claim 18). When *C* = 0
      (claim 18) nothing more happens in this item. Otherwise, the table below gives the survivor and smudge outcomes and their draws in claim 18's cell order, beginning after the cells used by absorbed passengers. The observable draw precedence is cell-ordered: the applicable draws for one cell precede those for the next. Every cell in that order has the smudge outcome, whether or not survivors remain to release and regardless of which survivor outcomes apply.

      Throughout these first-round cell outcomes, the destroyed building still occupies its foundation cells; it leaves them only when it leaves play (claim 21). For the smudge step, a foundation cell is *clear for movement* unless one of three tests fails it. (1) The occupancy record it reads has its object-present flag on (S-0209 Terms; for example a landed aircraft). The infantry positions, the positions a terrain object marks, and the vehicle-present and building-present flags are ignored: they do not affect this test. (2) The cell's overlay type has the wall flag. (3) The tracked locomotion class's multiplier *M*(*ℓ*, track) of the cell's land type *ℓ* (S-0211 claims 2 and 5) is exactly 0 (not merely below S-0211's 0.01 threshold). No mover is involved, so the class is fixed: test 3 reads the tracked class for every building type, whatever the building's own kind of movement and whatever stands in the cell. For a high-bridge cell (S-0211 claim 15) the record read is the bridge-deck record and test 3 is not made; for every other cell it is the ground record. This is not a passability category of S-0211 claim 11: a cell with a crushable or rubble overlay that is not a wall, a beach or water cell, or a cell holding a terrain object is clear exactly when the three tests pass. The same test applies in claim 21's second round. The plain form of claim 26 rejects any smudge type whose footprint touches a cell holding a building, including the destroyed building. Accordingly, when the nearby point lands in one of the destroyed building's foundation cells, no smudge is placed and no smudge-type draw is taken.

      A nearby point 128 leptons from a cell centre remains in that cell unless it lies exactly on the cell's east or south edge; a point on either edge belongs to the next cell (F2 truncates). Therefore the first-round smudge steps can place a smudge only in the rare case that the point lies on the foundation's outer east or south edge. Every other smudge step takes its percent draw and the nearby point's raw value, and no further smudge-type draw. Claim 21's second round occurs after the building has left its cells and is not limited by the dying building's obstruction.

      | Row | Taken when | Draws | Effect |
      |---|---|---|---|
      | survivor chance | as claim 18 (*Chance*) gives | one ranged draw R(0, *g*) | a result of 1 releases a survivor |
      | survivor | the chance row released a survivor | its type, creation, position and health draws, as claim 18 gives them | one survivor, when it can be placed |
      | smudge step | the foundation cell itself (not the cell of the nearby point) is clear for movement | one percent draw, then the raw value of the nearby point N(cell centre, 128, *b*), then the draws of claim 26's smudge choice in its plain form | a scorch mark when the percent draw is below 50, otherwise a crater, at the cell of the nearby point, when claim 26 finds a smudge type that fits there |

    - (h) kills any passengers still inside it.
    - Backing: `measurable` (scenario D14; the draw order through the random-state recorder, which
      gives the number of raw values taken, and through the statistics of the effects; variant i for
      the clear-for-movement test). `unmeasured` for the high-bridge case: variant i tests ground
      cells only, and no scenario here puts a foundation cell under a high bridge. `unmeasured`
      for test 1's ignoring the vehicle-present flag: no part of D14 has a vehicle on a
      foundation cell at the smudge step. Test 3's class being the tracked one for building types
      other than variant i's test building is `unmeasured`: no scenario here repeats variant i
      with another building type.
    - Confidence: medium (the list and the draws, and the building still being in its cells during
      the smudge steps); high for the order of the draws within a cell and for the cell the smudge
      step tests; high for test 2 on the overlays variant i tests (a wall overlay and a crushable
      overlay that is not a wall), medium for test 2 on other overlays; high for test 3, and for
      test 1's ignoring the building-present flag (the
      destroyed building's own foundation, in every run) and the terrain object's positions
      (variant i); medium for test 1's ignoring infantry positions, which D14 meets only where a
      survivor stands in the cell at its smudge step, and for its ignoring the vehicle-present
      flag; medium for the object-present flag in test 1, for the high-bridge case and for test
      3's tracked class on other building types; high for
      item f's eligibility, attachment
      condition, selection, displacement and emission lifetime

20. **Rubble and crates beneath.** When a destroyed building's type leaves rubble and has a rubble
    image (a separate rubble building image, or a sprite with more than 7 image frames), each of its
    foundation cells receives the overlay type of registration index 239 (the overlay at that index
    of the list in force, S-0004; the index is built into the original, so the overlay list decides
    which type it is: `quirks.rubble_overlay_by_index`, Edge cases), with its overlay data value
    left as it was, and the building's top-left cell remembers the building's type for drawing the
    rubble (rendering specs).

    When the type has a crate beneath, the crate is placed after any rubble, through a crate place
    (S-0520 claim 1). Its cell is the result of S-0212 claim 25's single-cell nearby search centred
    on the building's top-left foundation cell: *float* locomotion class when that cell's land type
    (after any rubble, S-0211 claim 5) is *water* and *track* otherwise, *normal* movement zone, no
    reach region, deck mode off, no height condition, no further test, no no-overlay condition,
    bridges allowed and no reference cell. The building has already left its cells (claim 21), so
    its own presence does not keep the top-left cell from qualifying. No crate appears, and no other
    cell is tried, when the search finds no cell, when every crate place is occupied, or when the
    selected cell is outside the height-aware usable area or holds an overlay of any kind; none of
    these takes a draw. A crate beneath therefore never replaces an overlay. Otherwise the selected
    cell receives a water-crate overlay when its land type is *water* and a wooden-crate overlay
    otherwise, with S-0520 claim 3's overlay draws and lifetime request, and the crate holds the
    money power-up (0) when the type says so and random contents otherwise (S-0520 claim 5).

    So when the top-left cell qualifies for the search and passes its visibility test, it is the
    selected cell: the crate appears there when the cell holds no overlay, and no crate appears at
    all when it holds one, a resource overlay or the building's own rubble alike. When the top-left
    cell does not qualify (it lies outside the usable area, its land type or overlay keeps the class
    out by S-0211 claim 10, or an object stands on it), the crate can appear on another cell that the search selects,
    provided that cell holds no overlay. Quirk `quirks.crate_beneath_overlay_refusal` (Edge cases)
    covers the lost crate.
    - Backing: `measurable` (scenario D14, variant b, for the top-left cell with and without an
      overlay and with rubble; S-0520 scenario CR3 for contents). `unmeasured` for a top-left cell
      that does not qualify, for a *water* top-left cell and for occupied crate places: D14 places
      its buildings on clear ground inside the usable area, with no object on the top-left cell and
      free crate places.
    - Confidence: medium for the rubble condition; high for the contents, for rubble being placed
      first and for the outcomes at a qualifying top-left cell on land; medium for the search settings at a
      top-left cell that does not qualify, for the *water* class and image, and for occupied crate
      places

21. **When a destroyed building leaves play.** Here only the type's explodes flag counts (a
    veterancy ability or a weapon's suicide flag, which make an object exploding elsewhere in this
    spec, do not). A destroyed building whose type lacks the explodes flag and whose current
    behaviour is not the sale's at the moment of its destruction leaves play at the end of its
    death effects; its rubble and crate of claim 20 are placed in that same moment. A building
    whose type has the explodes flag, or whose current behaviour is the sale's when it is
    destroyed, instead awaits removal (Terms) after its death effects, with 0 health:
    - It cannot be hit (S-0303 claim 9 skips an object with 0 health) and it is no candidate for
      target acquisition (S-0309 claim 12, row *exists*).
    - It keeps occupying its foundation cells, with whatever claim 20 has not yet placed: its
      rubble and crate are not placed yet.
    - It takes its turns (S-0203), but its behaviour is not processed while its health is 0
      (S-0215 claim 2). In particular a sale cannot advance, release further sale survivors or
      complete its refund after this death.

      | State/event | Observable result |
      |---|---|
      | Destroyed building remains in play at 0 health; it is not being erased or affected by an electromagnetic pulse | Its own turn removes it after that turn's general part. Until then it occupies its foundation. |
      | Eligible removal turn | The building leaves play and ceases to occupy its foundation before the second survivor and smudge round. |
      | Second round, after the building has left play | Claim 18 and claim 19(g) apply again. The survivor count *C* is recalculated by F5 and is 0 when the killing damage ignored defences, which left the building without crew. Cells follow claim 18's order and the survivor-chance, survivor and smudge-step outcomes and draws of claim 19(g). Absorbed passengers have already come out or died, so none come out now. Because the building no longer occupies its cells, its presence does not refuse this round's smudges. Any passengers still inside die. Claim 20's rubble and crate are placed in this same removal turn, completing removal; the building has already left the update order before this round. |
      | The building is still being erased or affected by an electromagnetic pulse | Removal and the second round wait for a later eligible own turn. |

      This table states observable states and timing, not an implementation's update-loop sequence. Quirk `quirks.building_removal_second_round` (Edge cases) covers the second round.
    - So a building of an exploding type normally stays in play until its own next turn, which comes later in
      the same tick when its turn has not yet come in that tick's turn pass, and otherwise in the
      next tick. While it is under an electromagnetic-pulse effect, or being erased, the removal
      waits for a later turn. This is the same for a building destroyed while being sold,
      whatever stage the sale had reached. Sale survivors released before the killing hit are
      unaffected; after it there are only the destruction's survivor rounds, not the sale's.
      It pays no sale refund (S-0408 claim 6). A sale that already removed a still-living building
      before a pending delayed-kill expiry instead leaves no death at all (claim 3).
    An otherwise selectable building awaiting removal remains selectable at 0 health, both
    for an exploding type and for a building destroyed while being sold; being erased still
    excludes selection (S-0214 claim 16). Selection neither restores health nor postpones
    removal nor restarts a sale. S-0214 owns the battlefield-click conditions and scenario T12.
    This refines S-0203 claim 6b and answers Q-0081 parts 3 and 4 together with S-0214 claim 16.
    Claim 3 and S-0302 claim 16 answer parts 1 and 2.
    - Backing: `measurable` (scenario D14, variants c, g and h; S-0214 scenario T12 for selection)
    - Confidence: high for immediate removal, the next eligible own-turn removal of exploding
      and selling buildings, occupancy until removal, the second round, the erasure and
      electromagnetic-pulse exceptions, the stopped sale and the selection rule; medium for
      obtaining a click during the short interval in stock play (S-0214 claim 16)

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
    - Backing: `unmeasured` for whole and fractional holdings: no known skirmish setup fills a
      building's storage, including through named test data. Ordinary harvested income goes
      directly to credits (S-0402 claim 12; S-0408 claim 10). D15 is a conditional proposal only,
      not backing until a way to fill the storage is established.
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
    absorbed passengers come out most-recently-entered first (S-0605 claim 14), each into the
    next foundation cell in the cell order of S-0302 (claim 13), whether or not the previous
    passenger could be placed: the most recently entered passenger uses the first cell, the
    next most recent the second, and so on, and the survivors of claim 18
    start after the last cell used. A vehicle comes out at the cell centre. An infantry unit goes on a
    free standing position: its desired point is 36 leptons from the cell centre along the map y axis (the second of the cell's two axes; S-0408 claim 7 gives the same point as 128 leptons along x and 164 along y from the cell's top corner), which S-0208 claim 2 counts as the centre class. An infantry unit whose cell is open takes one draw over 0 to 3, then the first free position of row R*k* of table O of S-0208 (claim 3), where *k* is that draw. A cell that the first row of table O refuses (a vehicle in it, or a flag that row names) takes no draw. When that row gives no position, the unit cannot be placed and dies, as below (S-0208 claim 3 gives no position then). This draw is taken before the scatter request that the passenger then receives (claim 12). Each gets out facing the building's facing and receives the scatter
    request of claim 12. A passenger that comes out keeps its owner and counts again towards
    that owner's crate infantry count if it was not already counted (S-0605 claim 14).
    Once out, a passenger is given no further order when its house is
    human-controlled and is put on the *hunt* behaviour when it is computer-controlled (S-0215; which
    one our engine applies depends on the owner, claim 25). One that cannot be placed
    dies, credited to the building's charge planter (Terms), if any, and otherwise to no one (not to
    the attacker of the killing hit). When the building has no crew (killing damage that
    ignores defences, F5), all of its absorbed passengers die instead of coming out,
    credited in the same way. They come out whatever the survivor count. The release tests only
    that no-crew state, not the type's crewed flag: a building whose type lacks the crewed flag
    releases its absorbed passengers as any other does, and no survivors (*C* = 0 by F5). The
    original does not
    stop at the last foundation cell: with more passengers than cells it places the further
    passengers at unspecified positions (Edge cases, *Absorbed passengers beyond the last cell*).
    - Backing: `measurable` (scenario D16; variant f for the release's ignoring the crewed flag)
    - Confidence: medium for the placement and the credit; high for the passenger order, crate
      infantry count restoration, cell order and survivors starting after the last cell used,
      medium for the *hunt* behaviour; high for the release's ignoring the crewed flag

### Computer-controlled houses

25. **Orders after a death depend on the owner, not on who plays it.** In our engine the orders
    that claims 12, 13, 18 and 24 give to the units a death releases are:

    | Released unit | Owner is a player house (human-controlled or an AI house) | Owner is the neutral or the special house |
    |---|---|---|
    | crew (claim 13) | *guard* | *hunt* |
    | survivor without an attack order (claim 18) | *move* | *hunt* |
    | escaping passenger (claim 12), absorbed passenger that came out (claim 24) | no order beyond its scatter request | *hunt* |

    For a computer-controlled owner, the original gives these units *hunt*, except that an escaping
    vehicle passenger joins the destroyed vehicle's team when that vehicle belonged to one. By the project lead's
    decision (glossary, *AI house*, *Player houses only*) an AI house follows the player-house
    column, while the neutral and the special house keep the original's rule for owners that are not
    human-controlled. Our engine has no team system (glossary, *team member*): no vehicle belongs to
    a team, so the join of claim 12 never happens for any owner, and the neutral and special houses'
    passengers take *hunt* instead. This is the non-team case of the original's rule, retained
    by the lead's existing neutral/special-house ruling, not a new exception to that ruling.
    There is no choice between *hunt* and no order for these owners in this spec.
    - Backing: `measurable` for the human-controlled column and for the neutral- and special-house
      column (scenarios D9 e, D10 d, D13 e and D16 c); `unmeasured` for the AI-house application
      (a design rule of our engine, following the project lead's decision on AI houses, so no
      original shows it) and for the team join (claim 12's reason)
    - Confidence: high for applying the lead's owner rule; medium for the original's non-team
      passenger orders, tested without any team membership

### Smudge choice

26. **Which smudge type a destruction leaves.** Claim 19 (a) and the smudge steps of claim 19 (g)
    choose a smudge type of a given kind (crater or scorch mark) for an anchor cell (the cell of the
    point they give) as follows. Animation-specific smudge selection is owned by S-0202
    claim 15, including during-play creation and its size-dependent candidate preferences.
    The destruction forms below remain unchanged.
    - *Zero-coordinate exception.* With `quirks.destruction_smudge_zero_anchor` on (default,
      the original), an anchor cell with both coordinates 0 receives no smudge and the smudge
      choice takes no draw. With it off, that anchor uses the same fit, candidate and choice
      rules as every other anchor, including the requirement that it lie on the map. Draws
      already taken to choose the kind and nearby point are unaffected.
    - *Fit.* A smudge type of the kind fits at the anchor cell when the anchor cell lies on the map
      and every cell of its footprint (Terms) is flat (no slope), holds no overlay and no smudge, and
      has a tile whose tileset has the morphable flag (S-0010 claim 15). For this test only, a
      negative tile number or a tile number at least the theater's tile count uses tile 0's
      morphable flag; it does not pass unconditionally. This differs from S-0010 claim 15's
      to-tile building placement test. In the *plain form*
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
    - Backing: `measurable` (scenario D17; the draw count through the random-state recorder).
      The zero-coordinate exception is `unmeasured`: no reachable destruction-smudge setup
      at that anchor is established. Its off setting is a design rule, not an observation.
      A negative tile number other than the no-tile marker is also `unmeasured`, since no practical
      setup producing it is established (S-0010 claim 15)
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

### Destruction and map triggers

29. **A killing hit raises the object's tag before death debris.** For a tagged infantry unit,
    vehicle, aircraft or building, the destruction raises of S-0011 claim 19 happen within the
    killing hit after health reaches 0, before this spec's debris, exploding-passenger deaths,
    death weapon, attached bomb and kind-specific effects. Any actions that fire complete at
    that point, including their immediate effects and random draws; they are not queued until
    removal or until the end of the tick. Raises themselves add no random draw. Effects of the
    incoming detonation that S-0303 places before this hit still precede these raises; effects
    it places after the hit follow the hit's death effects. In particular, death debris is not
    the same ordering milestone as incoming-warhead debris.

    For infantry, aircraft and buildings, a known attacker gives three raises before the
    destroyed-by-anything raise; the first of them, code 6, is inside S-0011's subset and the
    other two, codes 4 and 7, are outside it. Without an attacker those
    three are absent, but destroyed-by-anything still occurs. If the object still carries the
    tag, one further raise with a code outside that subset follows. S-0011 owns each raise's
    evaluation, latches, actions and tag detachment; a raise that detaches the tag prevents
    subsequent raises of that tag from this object. Import mapping gives the codes in order.

    Vehicles normally have an additional earlier group: one attacker-dependent raise outside
    the subset, destroyed-by-anything, and its following outside-subset raise. The common group
    described above follows it, still before debris. Thus a repeating destroyed-by-anything
    trigger can fire more often for a vehicle than for infantry; later outside-subset raises
    also fire it through its latch. This extra group is absent when the vehicle remembers a
    hijacker type and any trigger definition in its tag has the tag-carries-over flag. The
    common group remains. When the hit instead credits an explicitly supplied attacking house
    different from the known attacker's house, or supplies a house without an attacker, only
    the common group occurs, with its three preliminary raises enabled by that supplied house.
    With `quirks.vehicle_destruction_extra_raises` off, only the common
    group occurs for every vehicle. S-0011 claim 19 and T7 agree with these groups:
    with the original's persistent latch, a fresh repeating destroyed-by-anything trigger
    fires seven times for a normal vehicle with an attacker, four without, and twice for
    a common-group-only death. With `quirks.persistent_event_latch` off and the extra
    vehicle group on, the normal vehicle instead fires twice, once per destroyed-by-anything
    raise; a common-group-only death fires once. These latch-off alternatives are rules of
    our engine under S-0011 claim 9, not additional original behaviour.
    Hand these passages off together with D20.

    **Deferred deaths.** A cyborg's crippling hit does not reach health 0 after its reprieve
    and makes no destruction raise. A delayed-kill building does reach 0 before the reprieve:
    its destruction raises occur on that initial lethal hit, before its health is restored to
    1 and the countdown is kept or started. A later expiry hit that reaches 0 raises any tag
    still attached again before its death effects. The same early moment applies to a killed
    vessel that goes on to sink and an aircraft that goes on to crash; neither waits for
    sinking or impact to make this killing-hit raise.

    **Infantry still showing death.** Starting an infantry death sequence does not by itself detach
    a tag. If S-0011's evaluation has not detached or deleted it, the infantry remains a carrier
    until an independent tag-detachment event: its removal does not detach the tag (S-0011 claim 2,
    under `quirks.removed_carrier_keeps_tag`; with that switch off, its removal detaches the tag).
    It is nevertheless no longer a present object for S-0011 claims 24 and 27. Completion of its
    death sequence and final removal make no new destroyed-by-anything raise. A tag's own polls and
    raises from other carriers remain S-0011's, not additional death raises from this infantry.
    - Backing: `measurable` (scenario D20; S-0011 T7 for evaluation and latches). `unmeasured` for
      the removal not detaching an infantry unit's tag: S-0011 claim 2's scenario for a carrier that
      stops existing, T6 variant H, has vehicles as its carriers, and in D20's retained-infantry run
      the first raise detaches the first carrier, so no run leaves a dead infantry carrier attached
      for a later raise to count
    - Confidence: high for the before-debris timing, vehicle extra group, deferred-death
      timing and absence of a final infantry-removal raise; medium for retained infantry
      carrier status and the hijacker/tag-carries-over combination

### Slaves of a destroyed master

30. **A killing hit ends the slave group before the master's debris.** For an on-map slave
    miner, whether a vehicle or its deployed building, an actual killing hit releases or removes
    its existing slaves in the same damage resolution, before claims 4 to 7. A delayed-death
    reprieve does not end the group at the initial hit: this rule applies when the master actually
    takes its death effects. Initial slave ownership and existence are S-0402 claim 19, and
    deploying or packing up preserves the working group under S-0415 claims 6 and 10 and is
    not this destruction rule.

    - A slave inside the master is destroyed and removed immediately, credited to the killing
      hit's known attacker, if any. It does not emerge, play a death sequence or take its own
      debris, death-weapon or attached-bomb effects. If it carries a tag, its destruction
      notifications are claim 29's common group, with the slave as causing object and the
      master's known attacker enabling the three preliminary raises. Destroyed-by-anything
      and its following outside-subset raise occur even without an attacker. Each requires
      the slave still to exist and carry the tag; S-0011 claims 8, 9 and 12 own evaluation,
      latches and suppression during an active raise. Resulting actions finish before the
      slave is removed. This notification group accompanies destruction without delivering
      damage to the slave: there is no slave killing hit and no second group at final removal.
    - A slave already on the map loses its master and stays at its existing position. With a
      known attacker, it changes owner to that attacker's current house, even if that house
      is allied with the master's house. Without a known attacker, its new owner is the first
      existing house in house-creation order whose country belongs to the civilian side
      (Import mapping). An attacking-house attribution supplied without an attacker object
      does not substitute for that object in this ownership choice.
    - A transferred slave is not recreated or healed by this release. Its attack target and
      movement destination are cleared and it is queued to *guard*, for every owner; claim 25's
      crew and passenger orders do not apply to it. Normal later behaviour processing remains
      S-0215's. If no fallback house exists in the no-attacker case, the slave instead receives
      damage equal to its current health with the demolition warhead, no attacker, and neither
      the ignore-defences nor the no-escape marker. S-0302 resolves that hit; survival is not
      excluded when defences stop it. This no-recipient path does not clear its attack target
      or movement destination, queue *guard* or request the *cheer* sequence. The damage
      itself and normal later processing can still change those states. A successful transfer,
      in contrast, requests *cheer* (S-0107 claim 21) after the target and destination reset.

    No slave in the group remains enslaved to this master, and this group can no longer
    regenerate lost slaves or release replacements. An on-map slave freed before the master's
    death weapon can still be hit by that detonation under S-0303's normal rules: freeing is
    not protection from the rest of the death effects. Sale, capture, mind-control and erasure
    without a killing hit are not established by this claim. Never-placed production masters
    retain Q-0193's boundary.

    **Companion-spec hand-off confirmation.** S-0011 claim 19 includes the hidden-slave
    destruction notifications in this claim and claim 31, including temporary hidden slaves
    discarded during successful pack-up. These are specific no-hit destruction cases, not a
    rule that every removal raises destruction events. Ordinary final removal after an infantry
    death sequence makes no new destruction raises as claim 29 states. Hand these passages off
    together with S-0011 claim 19.
    - Backing: `measurable` (scenario D21) for hidden removal, attacker and normal civilian
      transfer, guard queueing, no regeneration and before-debris timing; `unmeasured` for
      hidden-slave tag notifications (no established data-only setup attaches a tag to a
      generated slave), absent fallback houses and separately supplied attacking-house
      attribution (no isolated skirmish fixture for those paths is established here)
    - Confidence: high for hidden removal, ownership choice and no regeneration; medium for
      notifications, retained health and release ordering

31. **Removal without a killing hit ends an untransferred slave group.** When a slave
    master is removed without a killing hit, and its existing group has neither already ended
    nor been transferred to a replacement master, removal ends that group. Successful deploying
    or packing up preserves the old working group (S-0415 claims 6 and 10), but can discard
    a temporary group already created for the replacement master, as below. No killing hit
    is implied by removal.
    A production master whose item is cancelled is such a removal (*A cancelled production
    master* below). A never-placed production master killed by a defeat or surrender hit
    remains outside this claim under Q-0193.

    Hidden slaves are destroyed and removed without attacker credit, with the suppressed
    death effects and common destruction notifications of claim 30, without its three
    attacker-dependent preliminary raises. A tagged hidden slave can therefore raise
    destroyed-by-anything without receiving any damage; the cause is its destruction during
    group cleanup, not a killing hit. On-map slaves lose their
    master and keep their identity and position. They transfer to the first existing civilian-side
    house in house-creation order, using claim 30's no-attacker choice and its transferred-slave
    target reset, destination reset, *guard* queue and *cheer* request. If no such house exists,
    they instead receive claim 30's health-sized demolition hit, with no recipient transfer,
    target or destination reset, *guard* queue or *cheer* request. The group cannot regenerate
    or release further slaves.

    **Temporary hidden slaves during successful pack-up.** The return vehicle's newly
    created hidden slaves are destroyed and removed before the old working group becomes
    its group (S-0402 claim 19a). These temporary slaves receive the same common destruction
    notifications and suppressed death effects as hidden slaves removed under this claim,
    with no attacker credit. They contribute losses subject to S-0414 claim 21's
    victim exclusions, even though the old working group survives. Their creation
    draws are not undone. The old slaves survive the group transfer without recreation
    or healing; they are not freed to a civilian-side house by this temporary-group cleanup.
    S-0402 claim 19a owns creation and placement ordering and the refused-placement case.

    **A cancelled production master.** A production item whose type names a slave infantry type has
    its slaves from the start of its production, hidden inside it (S-0402 claim 19). The item can be
    cancelled by a cancel or cancel-all order of its house, including the cancellation orders that
    follow the removal of its icon (S-0411 claim 2), or by the check of S-0404 claim 8 when no
    factory building of its house could make it any longer. Either way, deleting its object (S-0404
    claim 6) is a removal without a killing hit, of a group that has neither ended nor been
    transferred. Every slave of the group is therefore destroyed and removed at the cancellation,
    with no attacker credit and with the common destruction notifications and suppressed death
    effects above. Each one gives its house a loss, subject to S-0414 claim 21's victim exclusions.
    No object is credited with such a slave's destruction: it is not a destruction by a hit, and it
    is none of the other kinds of destruction that S-0311 claim 9 credits to an object. So it gives
    no house a kill or score (S-0414 claims 21 and 22). The master itself gives no loss, since a
    cancellation is not a destruction (S-0414 claim 21). The slaves' removal adds no sound or
    message of its own to those of the cancellation. Their creation draws are not undone.

    This specifies the remaining group's fate at removal, not when a sale, failed conversion
    or erasure reaches that removal. S-0408 claim 10a owns true-sale fate and its timing boundary.
    S-0415 owns whether a failed packing attempt transfers the old group. An earlier group-ending
    event with a remembered attacker or forced new owner is not replaced by this attacker-less
    rule. In particular, erasure and environmental sinking's earlier group-ending events remain
    open below.
    - Backing: `measurable` (scenario D22) for the remaining group's hidden removal, civilian
      transfer and no regeneration after refused pack-up; `measurable` (S-0402 scenario H24) for
      temporary-slave removal during successful pack-up and its losses. `unmeasured` for tag
      notifications (no established data-only setup attaches a tag to these generated slaves) and
      absent recipient houses (no isolated skirmish fixture is established here). `measurable`
      (scenario D24) for a cancelled production master's slaves: their removal, their losses and the
      absence of kills and score, and the absence of a message of the slaves' removal (parts a and
      e). `unmeasured` for the absence of a sound at that removal (D24 records no audio)
    - Confidence: high for the removal ending an untransferred group and for the no-recipient
      exception; medium for the notification and presentation outcomes, and for a cancelled master's
      slaves: their removal, their losses and the absence of kills (a master that was never on the
      map comes under the rule by inference until scenario D24 is run); low for the absence of score
      from their destruction, as S-0414 claim 22 rates a destruction with neither an attacking
      object nor an attacking house

### Further killing hits during sinking

32. **Sinking does not suppress another killing hit's death effects.** A positive-health
    sinking vehicle can take another killing hit (S-0302 claims 8 and 11). This applies
    whether descent began with a killing hit or a grappling parasite's cull. Each such hit
    registers destruction again, with that hit's attacker or attacking house under S-0302
    claim 14 and the ordinary score, experience and victim exclusions of S-0311 claim 9
    and S-0414 claim 21. Claim 29's destruction raises occur again when the vehicle still
    carries a tag, subject to its existing latches, detachment and extra-group exception.
    Sinking completion is not another killing hit and adds no such credit or raises.

    The applicable shared effects of claims 4 to 7 occur again, with their ordinary draws
    and order: debris, any remaining exploding passengers, the death weapon of an exploding
    object, and a still-attached bomb. Already removed passengers are not recreated and an
    already detonated bomb does not detonate again. Claims 12 to 14 again handle passengers
    still held, crew and crates under their ordinary conditions and placement rules. There
    is no building-survivor round for a vehicle. A previously released crew does not by
    itself prevent another crew chance and its draws.

    For types without death image frames, the health and animation outcomes are:

    | Condition at this killing hit | Health, unless claim 15 removes it | Vehicle animations and removal |
    |---|---|---|
    | Ordinary sinking conditions of claim 10 hold | 1 health point | No claim 9 explosion or destruction animation; absent claim 15's crashable removal, descent continues from the existing height, without restarting its removal interval (S-0504 claim 13) |
    | Those conditions fail, including an underwater or insufficient-weight cull host | 0 health points | Claim 9's explosion and destruction animations and their draws apply, except for claim 11's marked-water replacement; a non-crashable vehicle remains in play because it is already sinking |

    The second row does not grant a new sinking reprieve. At 0 health ordinary subsequent
    damage and healing cannot revive it (S-0302 claim 8; S-0314 claims 9 and 10).
    A type that can crash retains claim 15's crash-start/removal precedence; below the surface it
    cannot start a crash and is removed at the end of these effects. Death-image-frame
    types retain their unresolved separate path, rather than inheriting either health row.
    - Backing: `measurable` (scenario D23; S-0504 scenario N14 for eligible repeated hits
      and unchanged descent). The death-image-frame path is unresolved, not an asserted
      outcome of this claim.
    - Confidence: high for repeated credit, raises, applicable effects and the ordinary
      health/removal branches; medium for isolating the ineligible cull aftermath in play

### Death weapons that kill further exploding objects

33. **Death weapons inside death weapons nest depth first, with no depth limit known.** A death
    weapon's detonation (claim 6) can kill a further exploding object, whose death weapon then
    goes off inside that killing hit (claim 1), before the outer detonation's later hits (S-0303
    claim 1), and its own detonation can kill a third, and so on. No limit of the original's own
    on how deep this nests is known: an object killed at any depth takes its death effects at
    once, in claim 1's order, and S-0303 claim 1's order holds at every depth. So a row of
    exploding objects, each within reach of the next death weapon, dies depth first inside the
    one killing hit of its first object, all in the same tick. Each object's death effects that
    come before its death weapon in the State and ordering table (its debris and its passengers'
    deaths) come before the death of the object its death weapon kills. That nested death, with
    all its own effects in the table's order and every death nested inside it, completes inside
    the death weapon's detonation, so before the killing object's effects that the table puts
    after its death weapon (an attached bomb of claim 7, then its kind's effects). An object hit
    later by an outer detonation (for example a second object within reach of the first death
    weapon) takes its hit only after every death nested before it has completed, at whatever
    depth. Whether a long enough chain makes the original fail, and at what length, is not
    established (Open questions).
    Our engine keeps claim 1's and S-0303 claim 1's depth-first order, as this claim states it,
    at every depth: it sets no depth limit, and it gives the same result for a chain of any
    length on every machine, so how deep it can nest must not depend on the machine it runs on.
    If a long enough chain makes the original fail, this departs from it there (design rule,
    decided at the delegated sign-off, 2026-11-12). There is no switch: a failure has no use in
    play.
    - Backing: `measurable` (scenario D25 parts a and b, at depths of 50 and of 600) for the
      deaths of the whole row in the one killing hit's tick and for a later hit of an outer
      detonation coming after every death nested before it. `unmeasured` for the absence of a
      depth limit as such: no finite chain shows every depth. The place of a nested death among
      the other death effects of the object whose death weapon killed it is claim 1's order, with
      claim 1's backing (scenario D1) for one level of nesting, and `unmeasured` at greater
      depths: D25 does not tell those places apart.
    - Confidence: high for the deaths in one tick and for the later hit after every nested death,
      at the depths of D25 parts a and b; medium for these at every other depth and for the
      absence of a depth limit; high for the place of a nested death among its killer's other
      death effects at one level of nesting, as claim 1 rates the order of the shared effects,
      and medium at greater depths; high for the departure, a design rule.

### Backing boundaries

These qualifications narrow the numbered claims' backing; a scenario for a neighbouring case
does not make these parts measurable. Confidence stays as in the owning claim unless stated
otherwise. No measurement has been completed for this revision.

| Claim or formula | Backing and reason for the part not covered by a scenario |
|---|---|
| Death rules for never-placed production items killed by a defeat or surrender hit | Unresolved under Q-0193, not covered by the numbered death rules' backing. S-0414's off-map hit eligibility and existence counts establish neither effects, draws nor immediate deletion. No prediction for these outcomes is asserted here. |
| 30, 31 | Hidden-slave tag notifications are `unmeasured`: D21, D22 and S-0402 H24 do not establish tag attachment to generated slaves. H24 tests temporary creation, removal and losses, not tag evaluation. Suppression of an attached bomb on a hidden slave is also `unmeasured`: these scenarios establish no such bomb attachment. D22 tests refused pack-up, not the timing or attribution of sale, erasure or environmental sinking. |
| 29 | `unmeasured` for separately supplied attacking-house attribution and interruption by a destruction-trigger action that removes the raising object during these raises: no isolated safe fixture is established here. D20 uses actions that leave the dying object alone. Q-0193's never-placed production items remain outside this claim. |
| 1, 19(d), 22 | `unmeasured` for resource spilling combined with other building death effects: no skirmish setup fills building storage. D1 and D14 test the effects their setups reach. |
| 6 | `unmeasured` for multi-turret selection and the unplaced projectile's land-type, height and water tests: D6 tests ordinary/elite primary selection and debris position, not these tests, and no setup isolating them is supplied. |
| 12 | `unmeasured` for bridge-deck escape placement: D9 variant h tests only which row applies on a deck, and the placement point remains for the bridge specs to establish. The bomb-mark row is `measurable` on land by D8's marked-deck variant, part b; on a bridge deck and more than 10 leptons above water it is `unmeasured`, since no setup kills a marked transport on a deck or holds a marked vehicle more than 10 leptons above water at its death. |
| 13 | `unmeasured` for crew trigger-tag inheritance: the kinds of tag that pass on are not established here, so D10 cannot predict a tag outcome. |
| 16 | `unmeasured` for a crash begun while still alive and the absence of tumbling draws for crashing infantry: no setup is established that starts either case. D11 b is conditional, not measurable backing. Also `unmeasured`: the fall-back crash's lost passengers and their credit to nobody, since no set-up is known that loads passengers into a pad-bound armed aircraft before its run. |
| 17 | `unmeasured` for the exact ten-times-strength amount and no-escape marker: D12's contacts die, which alone cannot distinguish that amount from another lethal amount or test crew escape. Release of a non-helipad contact at or beyond 256 leptons is also unmeasured: D12 does not establish those contacts. Garrison eviction instead uses S-0508 G7 and G8's backing. |
| 18, F4 | `unmeasured` for countries without a side and side numbers beyond 2: D10 and D13 do not construct those owners. The eight-cell 3 × 3 foundation is also unmeasured here: D13 uses a full nine-cell foundation. |
| 18 | `unmeasured`, low confidence, for equating the owner's survivor-suppression state with defeat: no setup separates the two possible moments in Open questions. Ordinary undefeated-owner runs test the rest. |
| 19(f) | `unmeasured` for the exact eligibility kinds, suppression by an earlier destruction-smoke system and the below-terrain height boundary: D14 f tests creation, displacement and marking at removal, not these conditions. |
| 20 | `unmeasured` for the sprite-frame threshold: D14 b tests a type already having a rubble image, not the threshold. `unmeasured` for a crate beneath at a top-left cell that does not qualify for the search, on *water*, or with every crate place occupied (claim 20 gives the reason). Crate contents use S-0520 claim 5 and scenario CR3. Remembering the building type for rubble drawing is a pointer to the rendering specs, not independently tested by D14 b. |
| 24 | `unmeasured` for vehicle passenger placement and credit to a separately remembered planter: D16 establishes neither vehicle absorption nor a surviving planter for failed releases. |
| 26, F2 | `unmeasured` for the zero-coordinate anchor, footprints beyond legal map cells and nearby-point fallback beyond coordinate 511: no reachable death/smudge setup exercising those boundaries is supplied. D17 tests fitting within the map. Negative tile numbers other than the no-tile marker retain claim 26's backing. |
| Edge-case departures on bad data | `unmeasured` for the original's out-of-list, missing-type, hang and beyond-foundation results: no scenario here isolates those bad-data outcomes. The safe alternatives, warnings and non-default quirk settings are design rules, not observations of the original. |

## Formulas

**F1. Death debris** (claim 4). With *X* the type's maximum debris count (*X* > 0) and *Y* its minimum:

> *N* = R(*Y*, *X* − 1)

If the type lists voxel debris types with maxima, the *N* pieces are dealt out to the types in turn
exactly as in S-0303 F6 (one raw value per turn, "magnitude first" remainder modulo the maximum plus
one, capped at the pieces left), each piece a piece of S-0317 created at the object's position
(its creation point, S-0317 claims 3 and 4). The start positions of S-0317 claims 3 and 4 apply
to these pieces, and the 20-lepton raise below only to sprite and metallic debris. Otherwise,
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

**F4. Crew and survivor type** (claims 13 and 18). A type without the crewed flag gives no normal crew
or survivor through this formula; claim 13's hijacker release is independent of that flag. For
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
or 2, or *q* = 0 (the original makes no division then). When a building has no crew is claim 18's
(*No crew*). Integer division truncated toward zero. Unit: infantry units.

**F6. Delayed death** (claim 3; S-0302 F7 owns it). With *F* the warhead's delay-kill tick count, *A* its delay factor at the edge (single precision), *d* the epicentre distance of the killing hit (leptons, in the number domain S-0302 F7 gives it) and *S* the warhead's spread, let *V* be the divisor used by S-0302 F7: *V* = 256 × trunc(*S*) with `quirks.delay_kill_whole_cell_spread` on, and *V* = *R* of S-0302 F3 with it off. Define, in order, *a* = RZ₅₃(*A* × *F*), *b* = RZ₅₃(*a* − *F*), *c* = RZ₅₃(*b* / *V*), *e* = RZ₅₃(*c* × *d*), and *T* = trunc(RZ₅₃(*e* + *F*)).

Each RZ₅₃ operation is a separate rounding at the indicated stage. Thus *T* rises from *F* at the epicentre to about *A* × *F* at the edge of the spread. The divisor uses the spread truncated to whole cells when the switch is on; when it is off, it uses *R* of S-0302 F3 instead. For *S* < 1 with the switch on, *V* is 0, the quotient is infinite or not a number, and the conversion gives *T* = 0 (S-0205 claim 7), so the building's destruction is timed 0 ticks later. Unit: ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cyborg health after crippling | 0.25 | fraction of type strength, at least 1 point | claim 2 |
| Height above which passengers die with the vehicle | 208 (exclusive) | leptons above the surface | claim 12 |
| Bomb-marked vehicle water height | 10 (inclusive), no lower bound | leptons above the surface, over a water cell | claim 11 |
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
| Anchor excluded by the default destruction-smudge rule | both coordinates 0 | map cell | claim 26; `quirks.destruction_smudge_zero_anchor` |
| Crew escape chance at a fresh start | 0.5 | chance (decimal) | claim 27 |
| Sinking weight at a fresh start | 3.0 | weight units of the type's weight field | claim 27 |
| Survivor divisors at a fresh start | 200, 100, 300 | credits per survivor, for sides 0, 1, 2 | claim 27 |

## State and ordering

**Observable precedence of death effects.** All effects occur in the moment of the killing hit, except where a row states that an object remains in play or an effect occurs on a later turn. The table records which outcomes and random-stream draws precede others; it does not prescribe an implementation's internal work order. Draws are from the scenario random stream unless noted. The sounds a death plays draw from the unsynchronised random stream and reach no game state.

| Precedence | Kind | Effect | Draws |
|---|---|---|---|
| 0 | infantry | cyborg crippling replaces death (claim 2) | none |
| 0 | building | delayed death replaces death (claim 3) | none |
| before 1 | tagged infantry, vehicle, aircraft, building | destruction raises and all actions they fire (claim 29); the initial lethal hit of a delayed-death building has these raises before its reprieve | only the actions' and their effects' draws |
| before 1 | slave master | hidden slaves removed, on-map slaves freed or hit when no new owner exists (claim 30); after the master's killing-hit destruction raises | resulting slave notifications and damage effects; release-draw details remain open |
| 1 | all | debris (claim 4, F1); skipped for a bomb-marked vehicle low over water (claim 11) | F1 |
| 2 | all, exploding | passengers die (claim 5), then death weapon (claim 6); both skipped for a bomb-marked vehicle low over water (claim 11) | passenger-trigger actions and their effects, then the detonation's |
| 3 | all | still-attached bomb detonates on the map, or is removed without effects off the map (claim 7); detachment before damage prevents a second detonation from the carrier's death | the bomb's, only when it detonates |
| 4 | infantry | death sequence or removal (S-0107 claim 17) | S-0107 |
| 4 | vehicle | sinking (claim 10), or wake and splash (claim 11), or explosion and destruction animations (claim 9) | up to two list picks (claim 9); none for sinking or for claim 11 |
| 5 | vehicle | passengers (claim 12): they get out, or die, or stay inside as the table of claim 12 gives (a vehicle with the bomb mark that cannot crash kills them) | their scatter requests' |
| 6 | vehicle | crew (claim 13) | unit-interval value, crew type (F4), creation draw, position (centre class not refused), health |
| 7 | vehicle | crate (claim 14) | the crate's |
| 8 | vehicle | crash, sinking or leaving play (claim 15); passengers still inside die at the start of a crash (claim 16) | crash: three |
| 4 | aircraft | explosion animation only with a nonempty explosion list, then crash or leaving play (claim 16) | one list pick only with a nonempty list; crash: three |
| 4 | building | bunkered vehicle let out and excluded from contact treatment (S-0606 claim 13), then other contacts and garrison eviction (claim 17), including removal without occupant deaths when no exit qualifies | the contacts' deaths and successful occupants' scatter requests |
| 5 | building | claim 19 (a) to (h), with claim 23 inside (c), claim 22 inside (d), and claims 24 and 18 inside (g); claim 18's cells start after the cells claim 24 used | as listed |
| 6 | building | rubble and crate, leaving play (claims 20, 21); for a type with the explodes flag or a building being sold, awaiting removal instead | the crate's (when it leaves now) |
| later turn | building awaiting removal | after the general part of its own eligible turn: leaves play and its cells before the second round of claims 18 and 19 (g), then passengers die; removal completes and rubble and crate are placed in this same turn (claim 21) | the second round's, then the crate's |

| State | Event | Result |
|---|---|---|
| alive vehicle | killing hit, sinking conditions hold | 1 health point restored, no claim 9 animations or picks; crashable failure to start a crash removes it in this hit, otherwise it remains in play sinking (claims 10 and 15) |
| living grappling host | cull under S-0614 claim 11 | sinking without a hit or health change, regardless of weight |
| positive-health sinking vehicle | another killing hit | credit, raises and applicable death effects again; 1 health only under the ordinary reprieve, otherwise claim 32's zero-health/removal outcome |
| alive vehicle or aircraft | killing hit, can crash, positive height above the surface for a vehicle (claim 15), above the terrain for an aircraft (claim 16) | crashing, in play; rocket-missile outcomes follow S-0506 claim 20 |
| ordinary non-rocket crashing object within claim 16's scope | reaches the ground, was shot down | death weapon, leaves play |
| ordinary non-rocket crashing object within claim 16's scope | reaches the ground, was alive | destroyed by its remaining health, detonation |
| dropped seized unit | fall ends | S-0607 claims 16 to 19 own the outcome; only a crashable vehicle retains claim 16's ordinary crash-end exception, not an aircraft |
| alive cyborg infantry, not prone | lethal hit that does not ignore defences | crippled; health becomes the greater of 1 health point and one quarter of its type's strength, truncated toward zero |
| prone cyborg | lethal hit | dies (S-0107 claim 17) |
| alive eligible building | lethal hit of a delay-kill warhead | 1 health point, countdown of *T* ticks (claim 3) |
| building in a countdown | its turn at or after the end of the countdown | S-0302 claim 16's expiry hit; if it destroys the building, the death effects with no crew (claim 3) |
| alive building, type with the explodes flag or behaviour the sale's | killing hit | death effects, then awaiting removal with 0 health (claim 21) |
| awaiting removal | its own turn, still in play, not erased, no pulse effect | leaves play and its cells before the second round; removal completes and rubble and crate are placed in the same turn (claim 21) |

## Edge cases and quirks

- **`quirks.vehicle_destruction_extra_raises`** (default: on, the original). Claim 29's extra
  vehicle group precedes the common destruction raises, so a repeating latched trigger can
  fire more times at a vehicle death. Off: omit that extra group, leaving only the common
  group; this changes any draws taken by the omitted firings' actions.

- **`quirks.crash_death_weapon_twice`** (default: on, the original). An exploding non-rocket aircraft fires its
  death weapon at death and again at the impact of its crash (claim 16). Off: it fires only at the
  impact. Rocket interception and its quirk are owned by S-0506 claim 20.
- **`quirks.building_smudge_discarded_draws`** (default: on). Claim 19 (a) takes up to two ranged
  draws whose results are not used (none for a 2 × 2 foundation). Off: those draws are not taken. Turning it off changes every
  later draw, so it is a switch only for mods that do not need the original's random sequence.
- **Harvester cargo** (claim 9). The original computes the value of a destroyed vehicle's cargo and
  does nothing with it. Not a switch: no behaviour depends on it.
- **`quirks.death_weapon_unplaced_projectile`** (default: on, the original). The death weapon's
  projectile is never placed on the map, so the land-type, height and water tests of S-0303
  claim 21 and the warhead debris of S-0303 claim 22 use its unplaced default position (claim 6).
  Off: they use the dying object's position.
- **Departure: long chains of death weapons** (no switch; design rule, decided at the delegated
  sign-off, 2026-11-12). Our engine keeps claim 33's depth-first order for a chain of death
  weapons of any length, with no depth limit and the same result on every machine; whether and
  where a long chain makes the original fail is not established (claim 33).
- **Departure: empty game-wide metallic debris list** (no switch). In F1 (and S-0303 F6), when the
  metallic debris list is used with *N* > 0 but is empty, the original takes the ranged draw
  R(0, −1) and then takes an entry from outside the list. Our engine takes the same draws and creates no debris, and
  the importer warns about a positive maximum debris count with nothing to throw.
- **Departure: empty splash list for a dropped vehicle in water** (no switch). Claim 11 reads the
  last entry of the splash list; with an empty list the original takes an entry from before the
  start of the list. Our engine
  shows no splash, and the importer warns about an empty splash list.
- **`quirks.bomb_mark_water_death_on_deck`** (default: on, the original). Claim 11's water test
  measures a vehicle's height from the deck when it is on a bridge deck, and reads the land type
  of the cell under it, so a marked vehicle that dies on an intact bridge deck over water shows
  wake and splash on the deck, throws no debris, fires no death weapon and loses its passengers as
  if it had died in the water. Off: a vehicle on a bridge deck never passes claim 11's water test,
  and its death takes claims 4 to 6 and 9 as for an unmarked vehicle. Claim 12's bomb-mark column
  tests the mark alone and does not depend on this switch, so its passengers still die when its
  type cannot crash.
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
- **`quirks.crate_beneath_overlay_refusal`** (default: on, the original). Claim 20 places the
  rubble first and then refuses a crate beneath whose selected cell holds an overlay, so a type that
  leaves rubble loses its crate whenever its rubbled top-left cell is selected, and an overlay
  already on that cell has the same effect. On: as claim 20. Off: the search for a crate beneath
  also asks for the no-overlay condition (S-0212 claim 25), so it selects a nearby cell without an
  overlay.
- **`quirks.barrel_probe_top_left_only`** (default: on, the original). Claim 23 probes from the
  building's position, so for a larger foundation the east and south probes fall on the building
  itself. Off: the east probe is the cell next to the foundation's east edge in the top row and the
  south probe the cell next to its south edge in the left column, each with the animation at that
  cell's point; the north and west probes are unchanged. A 1 × 1 foundation is the same either way.
- **`quirks.building_removal_second_round`** (default: on, the original). A building that awaits
  removal (claim 21) takes a second round of survivors and smudge steps when it is removed, so it
  can release up to twice the survivor count *C* in total, and it takes that round's draws. Off: it
  takes no second round; it leaves play and its cells, its remaining passengers die, and its
  rubble and crate are placed, taking no draws but the crate's. Turning it off changes every later draw.
- **`quirks.destruction_smudge_zero_anchor`** (proposed; default: on, the original).
  Claim 26 suppresses the smudge choice at an anchor with both cell coordinates 0, with
  no smudge-choice draw. Off: apply the ordinary fit, candidate and choice rules there,
  without bypassing map legality or any other fit requirement. Earlier kind and nearby-point
  draws are unchanged. If candidates could fit there, omitting their choice can change the
  scenario random stream and later gameplay; this is not classified as a drawing-only fix.
  Reachability remains unestablished, so the original's exception is explicitly unmeasured.
- **`quirks.smudge_footprints_beyond_map_edge`** (default: on, the original). Claim 26 tests only
  the anchor cell for lying on the map. A footprint cell beyond the map edge is tested as a
  placeholder cell that holds nothing and has the theater's first tile type; it passes when that
  tile type's tileset has the morphable flag. Our engine does the same by default. This can matter
  only for a smudge larger than one cell anchored on the last row or column of the map. Off: a
  footprint extending beyond the map edge does not fit, so that smudge type is not a candidate.
- **Departure: absorbed passengers beyond the last cell** (no switch). With more absorbed passengers
  than foundation cells, the original goes on past the last foundation cell and places the rest at
  unspecified positions. Our engine treats each
  passenger beyond the last cell as one that cannot be placed (claim 24), releases no survivors
  and takes no smudge steps (claim 18 has no cells left).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| rules, type section, `Enslaves`, `SlavesNumber`, `SlaveRegenRate`, `SlaveReloadRate` | slave group whose destruction claim 30 ends | S-0402 claim 19 and Import mapping | S-0402's defaults |
| the side named `Civilian`, a name built into the original | no-attacker fallback ownership in claim 30 | first matching side name, compared ignoring letter case; then first existing house whose country's side index matches | missing side gives index −1, so a country without a side can match; with no matching house, claim 30's damage applies |
| map object tag field, map `[Tags]`, `[Triggers]`, `[Events]`, `[Actions]` | attached tag and its evaluations in claim 29 | S-0011 Import mapping, including its object tag reference matching | no matching tag: no destruction raises |
| map `[Events]`, event code 48 | destroyed-by-anything raise (claim 29) | integer event code; unused event value, as S-0011 claim 19 | no such event: no destroyed-by-anything condition, but other events and latches still evaluate |
| destruction raise codes (engine constants, not a configurable list) | common group (claims 29 to 31) and extra vehicle group (claim 29) | common group with known attacker: 6, 4, 7, 48, 29; without: 48, 29. Extra vehicle group with known attacker: 7, 48, 29; without: 48, 29. Each requires the tag still attached and, for the common group, the object not removed by a preceding action | not applicable |
| map `[Triggers]`, eighth comma-separated value | tag-carries-over flag (claim 29) | decimal-prefix integer, nonzero sets the flag; a tag has it when any definition in its chain does | initially false; omission or zero leaves the flag as it is |
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
| rules, type section, `Explosion` | explosion list (claims 9, 16, 19); an aircraft's empty list gives no aircraft-specific explosion animation or list-pick draw, at any height (claim 16) | animation-name list (items as claim 28) | initially empty; absent or empty text keeps the list in force, while nonempty text consisting only of no-type items clears it (claim 28) |
| rules, type section, `DestroyAnim` | destruction-animation list (claims 9, 19) | animation-name list (items as claim 28) | empty |
| rules, infantry type section, `Cyborg` | cyborg type (claim 2); also the damage-sparks flag (S-0616 claim 1) | boolean | no |
| rules, warhead section, `CausesDelayKill`, `DelayKillFrames`, `DelayKillAtMax` | delay-kill flag, *F*, *A* (F6) | boolean, integer, decimal (held as a single) | no; 5 ticks; 1.0 (as S-0302 Import mapping, which owns them) |
| rules, building type section, `EligibleForDelayKill` | eligible for delayed death (claim 3) | boolean | no |
| rules, type section, `Naval`, `Underwater`, `Organic`, `Weight` | sinking conditions (claim 10) | boolean, boolean, boolean, decimal | type specs |
| rules, type section, `Crashable` | can crash (claims 12, 15); failed crash start removes the vehicle, including a first sinking hit (claim 10), and remaining passengers die without attacker credit (S-0507 claim 16) | boolean | no at type creation; later absence preserves the value in force |
| rules, type section, `Crewed` | crewed flag (claim 13, F4, F5) | boolean | no (not crewed), for every kind of type (building, vehicle, infantry, aircraft) and whatever else the type is (a wall, a gate, a type infantry can occupy, a neutral map building); a later rules layer without the key keeps the value in force. Claim 18 rates this default: high for plain building types, medium for the other kinds |
| rules, type section, `Passengers` | passenger capacity (claim 13); capacity and boarding are S-0507 claims 1 to 6. Destruction escape excludes the dying vehicle independently of this value (claim 12); removal cleanup is S-0507 claim 16 | integer | 0 |
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
| the animation type named `FIRE3`, a name built into the original rather than read from a key | barrel fire animation (claim 23) | animation type name | when no animation type has that name the original takes an entry from outside the animation type list (bad data): our engine creates no animation and the importer warns |

The importer warns about debris maxima lists shorter than their types lists or all 0 with a
positive maximum count, about a positive maximum debris count that falls back on an empty metallic
debris list, about an empty splash list, about a missing crew type for a side with a survivor
divisor above 0, about a delay-kill warhead with a spread below 1 while `quirks.delay_kill_whole_cell_spread` is on, and about an overlay list whose entry at index 239 is missing or differs from the base rules' (claim 20).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* on purpose-made flat test maps, in a
skirmish with one human player and one bystander computer player (glossary), at game speed 6, with
test types given in the test map's override sections (named test files) and invented values. A
*recorder* that reads the game's state gives health values, object counts, object types, positions
and each object's current behaviour. The read-only recorder of the scenario random stream (glossary,
*recorder*) gives the stream's state before and after a kill, and so how many raw values the kill
took; replaying the claims' draw rules from the recorded start state predicts every outcome.
Use game speed 6 and 800 × 600 pixels, no engine add-on and no mods (the named test files of a scenario excepted).
Unless a part fixes another count, run each deterministic part once and each random part over
30 independent deaths, restoring the test area between deaths. Issue normal attack, boarding,
absorption or planting orders as needed, then kill the named object with the named source and
record from immediately before the kill through all its effects. No state is written by a
recorder. A trace unable to resolve before/after stream state or required event order is
inconclusive, not a pass. For replay-based parts, record effect types, positions, health and
behaviour as well as draw counts: a different outcome or end state falsifies the replay even
when the total draw count is equal. A share described as roughly uniform or 'about' a probability
must lie within 3 standard errors of it; a share outside that band falsifies the expectation.

*Observed areas* (glossary, *bystander computer player*, void rule). The bystander's start position
is the one farthest from the test objects. A scenario's observed area is the cells within 10 of its
test objects for D1 to D10 and D13 to D19, with these exceptions: D11 observes the aircraft's flight
path and the cells within 10 of its target and of its impact point; D12 observes the building, its
exit cells and the vehicle's route. A run is void when an object of the bystander that the scenario
does not name enters that area or acts on a test object, or when the bystander is defeated
before the observation ends. Scenarios with a neutral- or special-house object use a test object owned by that house as the
map gives it, and the same void rule. For every owner-order variant (D9 e, D10 d, D13 e and D16 c),
repeat with the human player, computer player, neutral house and special house. Record the
released unit's behaviour immediately after release, before its next processing; later motion
is only corroboration. Computer-player vehicles must have no team membership. A later computer
order makes the later-motion observation inconclusive. Place a reachable hostile test target
for each owner whose units should hunt. Expected: the original's human-player result is the
player-house column of claim 25; each other owner gives non-team *hunt*. A different immediate
release behaviour falsifies that owner's row. On our engine, replay with an AI owner instead of
the original's computer player: expect the player-house column, a design check rather than a
measurement of the original.

**D1. Order inside one death (claim 1).** *Setup:* a test vehicle type with the explodes flag, a test
death weapon whose detonation (spread 2) kills a second test vehicle standing next to it, a voxel
debris count of 3, and one passenger. *Observable:* the passenger's death credit and the second
vehicle's death. *Expected:* the passenger dies (credited to the attacker of the first vehicle) and
the second vehicle dies in the same tick as the first. *Falsified if:* the passenger escapes or the
second vehicle dies in a later tick. Also record passenger death relative to the death weapon's
target health loss: target damage preceding passenger death falsifies the precedence even within
one tick. Variant b: attach a planted time bomb to the exploding transport, give its passengers
positive debris counts and death weapons, and give the transport distinct explosion and
destruction animations. Record and replay debris, detonations, deaths and animations. Expected:
transport debris precedes passenger deaths; no passenger debris or death weapon appears; the
transport death-weapon detonation completes before the bomb's, and both precede its vehicle
animations. Contrary precedence or a draw for a passenger's suppressed death effects falsifies b.
Any passenger-trigger action draws are accounted for separately under S-0011 claim 19.

**D2. Cyborg crippling (claim 2).** *Setup:* a test infantry type with the cyborg flag and strength
200; a weapon that deals 1000 damage (not ignoring defences). *Observable:* the unit's health and
posture after each hit, and whether it dies. *Expected:* after the first hit it lives with 50 health
and crawls; the second hit kills it. Repeat with strength 203: the first hit leaves 50 health,
not 51, testing truncation. Variant b: a type with strength 2 (expected health 1). Variant
c: a cyborg that is prone (for example after taking fire, if its type can go prone) dies on the
first lethal hit. *Falsified if:* an initially non-prone cyborg dies on the first hit or has another health, or the prone
cyborg of variant c survives. The first-hit survival expectation applies only to the initially
non-prone cyborgs; first-hit death is expected in variant c.

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

**D4. Death debris (claim 4).** *Setup:* a test vehicle type with minimum debris 1, maximum debris
4 and no debris types; 200 kills. *Observable:* the number of debris pieces per kill. *Expected:* 1
to 3 debris pieces each time, roughly uniform. Variant b (claim 8): the same with a test infantry
type. *Falsified if:* 4 pieces or 0 pieces appear, or an infantry kill in variant b throws no
debris or a number outside 1 to 3. The kills are run in three repeats, each also recording the
creation position and the stream state: the setup above, with no debris types, gives metallic
pieces (the game-wide fallback), the second repeat names two sprite debris animations and gives
sprite pieces, and the third names two voxel debris types, each with a maximum of 3 so that no
count of 1 to 3 is capped, both falling types (not meteor types, whose start position S-0317
claim 4 moves back along their velocity), and gives voxel pieces. In every repeat the test
vehicle type has no explosion list, no death weapon, no passengers, no crew and no survivors,
so the kill's tick takes no draws for other death effects. The metallic and sprite repeats also
record each animation's owning house. Expected: metallic and sprite pieces start 20 leptons
above the dying object, owned by its house, and voxel pieces are created at its position, with
counts and types that match F1's replay. The draws compared are those the random-state recorder
logs in the kill's tick from the killing hit on, as S-0317 scenario V1 reads its detonation's
tick: F1's replay in the metallic and sprite repeats, and in the voxel repeat F1's replay with
each piece's creation draws (S-0317 claims 3 and 4). Draws after those, which a voxel piece's
turns may take (S-0317 claims 7 and 12), are not compared, so the check does not depend on the
tick of a piece's first turn. A different position or type, a compared draw that differs from
that replay, or, in the metallic and sprite repeats, an owner other than the dying object's
house falsifies that repeat. A voxel piece's
ownership is not judged here.

**D5. Passengers of exploding objects (claim 5).** *Setup:* a test transport with the explodes flag
carrying a transport that carries an infantry unit. *Observable:* each passenger's death and kill
credit, and the tick of each relative to the detonation. *Expected:* all die, credited to the
attacker, before the death weapon's detonation. *Falsified if:* any passenger survives or dies
after the detonation. Variant b (the order among passengers): a test transport with the explodes
flag carrying two infantry units, each with its own attached trigger whose action, when it runs
after that unit's death, sets the same map variable to a different value; the variable's final
value shows which passenger's trigger ran last, and the same with a passenger carrying a passenger.
*Expected:* the holder's first passenger's trigger runs before the second's, and a passenger's own
passengers' before its own; *falsified if* the order differs. S-0011 claim 19 fixes these
notifications and their actions before passenger removal and the transport's death weapon.
Repeat with a passenger-trigger action that creates an animation with known scenario-stream
draws, recording and replaying those draws separately from the transport's effects. Expected:
only the actions and their effects add passenger-related draws; no suppressed passenger death
effects draw. A missing action draw, a draw for a suppressed effect, or an action completing
after the death weapon falsifies this variant.

**D6. Death weapon (claim 6).** *Setup:* (a) a test vehicle with the explodes flag, a death weapon
of damage 100 and damage factor 1.5, killed beside a target with versus 100 %, no other damage
factors and enough health to survive; this test weapon has uniform falloff factor 1 throughout
its spread; (b) no death weapon
and no weapon: the game-wide death weapon with half of strength 300; (c) a vehicle carrying a
planted time bomb, not exploding; (d) a death weapon whose warhead throws metallic debris (a test
warhead with a positive maximum debris count), killed in the middle of the map; (e) as (a) and (c)
with a test infantry type that has the explodes flag, and an infantry unit carrying a planted time
bomb that is not exploding (claim 8). *Observable:* the
target's health loss, whether the bomb detonates, and where the debris appears. *Expected:* (a) the
target loses 150 with that uniform falloff; (b) 150 times the game-wide weapon's falloff; (c) the bomb detonates
on death; (d) the debris appears at the map's corner, not at the dying vehicle; (e) the same loss of 150, and
the bomb detonates. *Falsified if:* the amounts differ, the bomb does not detonate (for the vehicle
or the infantry unit), or (d) shows the debris at the vehicle (then the
position hypothesis is wrong, and claim 6 and the quirk need re-investigation). Variant f:
remove the explicit death weapon from a, retain a primary of damage 101 with factor 1.5, and give
its elite slot damage 201. Kill rookie and elite instances beside identical targets at identical
epicentre distances. Expected amounts before falloff: 151 and 301, truncated by F3; restoring
the explicit death weapon overrides either primary. A different amount or selection falsifies f.
Repeat b with strength 301: its game-wide amount remains 150. Kill a non-exploding otherwise
identical vehicle with no bomb: no death-weapon detonation is expected; one falsifies this case.

**D7. Vehicle animations (claim 9).** *Setup:* a test vehicle type with an explosion list of four
distinct animations; 200 kills; variant b with the explodes flag and unlimited ammunition; variant c
the same type as the first setup, a harvester carrying a full load. *Observable:* which explosion
animation appears per kill. *Expected:* each of the four entries appears roughly equally; in variant
b always the last; in variant c the same spread as the first setup. *Falsified if:* variant b shows
another entry or variant c differs outside the statistical band. Give the type a distinguishable
destruction-animation list and record both choices and stream state. Repeat b with finite ammo
left, zero ammo, only the exploding veterancy ability, and only a suicide primary weapon.
Expected: last-entry override only for the flag or ability with unlimited or positive ammo,
not for zero ammo or suicide alone; the explosion list pick is still consumed, followed by the
destruction list pick. Missing animations, wrong override cases or a replay mismatch falsify
these parts.

**D8. Sinking (claim 10).** *Setup:* a naval vessel type whose weight meets the sinking weight (a
test copy that sets its weight when needed), killed on water, with crashability off; variant b a
test naval type with weight below it. *Observable:* whether the vessel sinks (stays in play at 1
health) or shows an explosion. *Expected:* the first sinks with no explosion animation; the second
explodes. *Falsified if:* the first explodes, or the below-threshold vessel sinks instead of showing
an explosion. Use named map overrides to repeat the above-threshold vessel with its underwater flag
set, its organic flag set, and as an amphibious non-naval type on water. Each fails the sinking
condition; sinking falsifies that part. Repeat the ordinary vessel at exactly the imported sinking
weight: it must sink. Record both explosion and destruction animations; either on a sinking instance
falsifies their omission. S-0504 scenarios N5 and N6 test the subsequent duration, removal and
sinking behaviour. S-0614 scenario P6 separately tests culling-induced sinking without a killing
hit, including unchanged health; the killing-hit expectations above do not apply to that cull.

*First-hit crashable variant (claims 10 and 15; Q-0652).* Use named map override
"sinking-first-crashable" on the tiny water map, speed 6, 800×600, no engine add-on and no mods (the named test files of a scenario excepted)
apart from this test data. Compare two otherwise identical surface vessels at height 0,
meeting every sinking condition, with death image frames absent and crashability on for
one and off for the other. Give both distinguishable nonempty explosion and destruction
animation lists. Initially exclude passengers, crew, crates, debris, exploding abilities
and attached bombs to isolate the animation and crash-start draws. Deliver each vessel's
first killing hit, in 30 trials per variant. Record in-play membership immediately after
the hit, health while present, height through the following own updates, animations and
the scenario stream state, separating the attacker's and projectile's effects.
Expected: the crashable vessel leaves play in the killing hit, without either list pick,
either vehicle animation or any crash-start tumbling draw. It has no subsequent descent
or sinking wakes. The non-crashable control remains at 1 health and descends under S-0504
claim 4, also without either list pick. Remaining in play after the hit, beginning a
later descent, either vehicle animation or an associated pick or tumbling draw falsifies
the crashable prediction. Then repeat with D4's identifiable debris and D6's exploding
death weapon: their applicable effects and draws still occur before removal, not claim 9's
picks. Omitting an applicable shared effect or adding either pick falsifies that extension.
A recorder that cannot separate the hit from later own updates is inconclusive for timing.
Remove the named override after the test.

*Marked-water variant (claim 11).* Use S-0503 claim 11's scenario H7 for a stock hover switched off
over flat water, and S-0511 claims 9 and 10's scenario B3 for a stock vehicle dropped from a bridge
deck over water. Use stock data, no engine add-on and no mods (the named test files of a scenario
excepted), at a fixed game speed. Record heights, death ticks and animations with tick-aligned
capture and a read-only recorder. Expect a fall followed by wake and the last splash animation at
low-water death, without the ordinary debris, death weapon or vehicle explosion/destruction
animations. Ordinary death effects in place of wake and splash falsify this rule. Repeat the bridge
test with a loaded amphibious transport type whose crashability is off (a test copy that sets it
when needed): record passenger deaths and any escape. Expect no passenger to get out; a surviving
escaped passenger falsifies the passenger rule. This is not a test of where passengers leave a
vehicle destroyed on an intact deck (claim 12); that placement remains open in S-0511. These
ordinary falls do not discriminate the exact 10-lepton death threshold.

*Marked-deck variant (claims 11 and 12; Q-2793).* A tiny test map with two high bridges: bridge
A over land and bridge B over deep water, joined by a land route from A's landing cells to B's
end, at a fixed game speed, stock data, no engine add-on and no mods (the named test files of a
scenario excepted). Use S-0511 scenario B3's retained-mark variant on bridge A: give each deck
vehicle there the temporary invulnerability of that variant, then lose A's deck section, so that
each lands on land, survives its landing hit and keeps the bomb mark. Run with two vehicles of
the human player: V1, a test copy of a stock ground vehicle with the explodes flag on and a
positive maximum debris count (claim 4); V2, a stock transport whose type cannot crash and does not
explode (claim 5), and neither of whose explosion and destruction lists is empty, loaded with five
infantry (a test copy with crashability or the explodes flag off, or with an entry in an empty
list, when needed). Wait until the invulnerability
has run out, and record with the read-only recorder that each still carries the mark, that no
other writer of the mark has acted on it and that it has positive health.
- (a) Move V1 onto the deck of bridge B, over a water cell, and stop it there. Kill it there with
  one ordinary hit (no random impact effects). *Observable:* the recorder's height above the
  surface and above the terrain, its cell layer and the cell's land type at the hit; the animations
  created at its position, debris, any death-weapon detonation, and the scenario stream state.
  *Expected:* height 0 above the surface, 416 above the terrain (the water cells under bridge B are
  flat), deck layer, water land type; the wake animation and the last entry of the splash list on
  the deck, no debris, no death weapon, no explosion or destruction animation and neither list
  pick. *Falsified by:* any ordinary death effect of claims 4 to 6 and 9 or a missing wake and
  splash with those readings. Such a run falsifies claim 11's reading from the deck when the
  marked-water variant shows the wake and splash without the ordinary effects; when that variant
  fails as well, it falsifies only the low-water death-effects rule and says nothing about the
  reading. In a second run with `quirks.bomb_mark_water_death_on_deck` off, our engine must show
  the ordinary effects; this second run is a check of the engine, not of the original.
- (b) Kill V2 on land, at height 0 above the surface, with one ordinary hit. Control: an unmarked
  V2 of the same type, loaded alike, killed alike on the same cell. *Observable:* how many
  passengers die and how many get out; the vehicle's animations. *Expected:* the marked V2's
  five passengers all die, credited to the attacker, and its death shows claim 9's explosion and
  destruction animations (land: no wake or splash); the control's passengers get out. *Falsified
  by:* a passenger of the marked V2 getting out, a missing explosion or destruction animation, or
  wake and splash on land. A run in which a passenger of the control dies voids (b).
- Note: a landing hit cannot test (b) in place of an ordinary hit. It ignores defences, and under
  claim 12 a passenger dies whenever the killing damage ignores defences, so the passengers of an
  unmarked vehicle would die too.
- A run in which a vehicle loses the mark, gains it again from another writer, or cannot reach
  bridge B's deck is void, not a falsifier.

**D9. Passengers of destroyed vehicles (claim 12).** *Setup:* a stock transport with five infantry,
killed (a) on open ground by normal damage, (b) by a train driving into its cell (damage that
ignores defences, S-0210 claim 10), (c) as a flying transport more than 208 leptons up; variant d: a
test crashable vehicle with passengers, killed on the ground; variant e: (a) once with the
transport owned by the human player, once by the computer player and once by the neutral house. *Observable:* how many
passengers get out and how many die; in variant d the tick of their deaths; in variant e each
escaped passenger's behaviour and position some ticks later. *Expected:* (a) all five get out and
scatter; (b) and (c) all die; (d) the passengers die when the vehicle leaves play, which is the end
of its death effects; (e) the computer player's and the neutral house's passengers start hunting (walk towards the other
player), the human player's receive no additional order after scatter. The common owner-order
protocol also tests the special house and uses immediate behaviour, not an assumption that a
human passenger never moves again. *Falsified if:* any differs. Variant f: repeat a with a
land-only passenger in a non-crashable amphibious transport killed on water, and with all its
passenger's standing positions in the transport cell occupied. Expected: a passenger refused
entry or placement dies, credited to the attacker. In an open cell, record initial position and
facing before scatter: the transport's horizontal position, cell-centre terrain height and body
facing rounded by S-0204 F4 are expected. Survival in a refused cell, wrong credit, position or
facing falsifies f. Repeat c with a named flying vehicle of cruise height 208, then 209 leptons:
for a non-crashable type the first releases passengers and the second kills them. Repeat d in
flight at or below 208: passengers die at crash start, not impact. A reversed threshold result or
surviving passenger after crash start falsifies those cases. In d record passenger destruction
notifications, the attacker's kill count and experience: removal cleanup gives no passenger kill
credit or passenger death effects; an attacker-credited passenger death at ground removal
falsifies this result. For the aircraft counterpart use S-0507 scenario T14.

Variant g (Q-0707): on a tiny flat skirmish map at speed index 2, no engine add-on and no mods (the named test files of a scenario excepted), kill a
stationary stock transport carrying one infantry unit on otherwise empty enterable ground.
Record the passenger's entry outcome and survival with the read-only recorder before scatter.
Expect *clear* and escape despite the dying vehicle's presence until its removal. As an ordinary
entry control, ask a same-house infantry unit with no target, tether or other entry override to
walk into a stationary same-house vehicle's cell; the vehicle must have no destination or
movement in progress. Expect *stationary friendly* (6), as S-0209 claim 4 row WP16, not the escape's
*clear*. Also preplace a second stationary allied vehicle in the transport's cell on a named test
map, keeping a passenger standing position free. Verify both vehicles are present before the
hit and that the second survives the hit and death effects. The escaping passenger now gets
*stationary friendly* and dies credited to the attacker: only its own dying transport is excluded.
Void a run if the second vehicle moves, acquires a destination or dies before the escape test,
or a bystander interferes. Use scripted mouse and keyboard input through the game's normal
controls and a read-only recorder at 800×600, with one run at a time. Keep the common setup's
human player and bystander opponent; record exact seeds and each object's identity without
setting the random state. A blocker attributed to the dying transport in the empty
case, failure of the stationary control, or omission of the surviving second vehicle falsifies g.
This compares reachable original outcomes, not an altered escape test that counts the carrier.

Variant h (Q-2793, the height read from a bridge deck): on a tiny test map with an intact high
bridge over flat land, at a fixed game speed, stock data, no engine add-on and no mods (the named
test files of a scenario excepted), stop a stock transport whose type cannot crash (a test copy
with crashability off when needed), without the bomb mark and loaded with five infantry, on the
deck, with the deck cells around it empty. Kill it with one ordinary hit. *Observable:* the
recorder's height above the surface and above the terrain and the cell layer at the hit; how many
passengers die and how many get out. *Expected:* height 0 above the surface and 416 above the
terrain, deck layer; the passengers get out (the rows for 208 leptons or less), each surviving
its entry test. *Falsified by:* nothing in a single run. A passenger getting out confirms the
height read from the deck, since with a height read above the terrain (416, more than 208) every
passenger dies without being placed. A run in which all five die does not decide the reading,
because a passenger whose deck placement fails dies too (claim 12) and that placement is
`unmeasured` (Backing boundaries, claim 12); it is recorded as undecided. Where each passenger is
placed is not judged here.

**D10. Crew (claims 13, 14).** *Setup:* a stock crewed vehicle with no passenger capacity, killed 500
times; crew escape chance from stock data. *Observable:* share with a crew, crew types, crew health.
*Expected:* share within 3 standard errors of the chance; health between 5 and half the crew type's
strength; the type matches F4 (about 15 % technicians for an armed vehicle). Variant b: a vehicle
taken over by a vehicle thief always leaves that thief's type. Variant c: a test crate carrier is killed while the game's crate setting allows crates and a free
nearby cell is available; it leaves a crate. Failure to leave a crate under those conditions
falsifies the result. Variant d: the vehicle killed once while owned by the human player,
once by the computer player and once by the neutral house. *Observable of d:* each crew unit's behaviour and position some
ticks after the kill. *Expected of d:* the human player's crew stays near where it scattered to
(guard); the computer player's and the neutral house's crew walk towards the other player (hunt).
*Falsified if:* the share lies more than 3 standard errors from the chance, a crew health lies
outside 5 to half the crew type's strength, the share of technicians among the crews of an armed
vehicle differs clearly from 15 % (outside 3 standard errors), a variant b vehicle leaves another
type, variant c leaves no crate, or in variant d the human player's crew hunts or the other two
crews stay. Variant e: give the named test map a normal escape chance of 1; repeat with the
crewed flag absent, nonzero passenger capacity, and S-0414's no-escape defeat hit. No normal crew
is expected; one falsifies the exclusion. Repeat the hijacked vehicle with its crewed flag off:
the remembered thief is still due. Use a water death location where that infantry cannot be
placed: no crew enters play, but its creation draw remains and no health draw is taken. A
successful placement or different replay falsifies this part. For successful releases record
initial facing (map north), health and the F4, creation and health draws. Repeat an unarmed
ordinary crewed type for sides 0 to 2: its side crew with no F4 percent draw is expected. A
different type or replay falsifies that case. Repeat c with crates disabled: no crate is expected,
one falsifies the negative case. Variant f (the crew's standing position, claim 13): a crewed test
vehicle with the escape chance set to 1 in the named test map, killed 100 times standing still at
the cell centre and 100 times in the middle of a step between two cells, each on flat ground with
no other occupant; record the vehicle's exact position at the kill. A mid-step run stays in the
sample when its point lies outside S-0208 claim 2's centre radius, whatever class that claim then
gives it (the centre class included); a mid-step run whose point lies within the radius is void,
since its point is of the centre class by distance alone and only repeats the centre runs. Repeat
the centre runs with two own infantry placed by the map on the right and left positions of the
vehicle's cell; runs in which they do not stay there are inconclusive. Record the crew's position
and the draws the recorder reports.
*Expected of f:* at the centre, and at a mid-step point whose class is the centre (S-0208 claim
2), one draw over 0 to 3 between the creation and health draws, and the first free position of the
row of S-0208 table O it picks (right about half the time in the empty cell); at a mid-step point
of another class, no position draw and the position of that class (or the first free position of
its table O row). A different position, row or draw count falsifies f.

**D11. Crashing (claims 15, 16).** *Setup:* a stock aircraft with a test death weapon, shot down in
flight above a target vehicle; variant b an aircraft that crashes alive (out of a landing place, if
a stock situation does this). *Observable:* where the death weapon detonates (impact point or point
of death), and in variant b the amount. *Expected:* the death weapon goes off at the impact point,
not where the aircraft died; variant b: a detonation with the aircraft's remaining health. Variant
c (claim 15): a test vehicle type with the crashable flag, killed while above the terrain, off any
deck (for example in jumpjet flight). For a jumpjet, place no other object in its own cell or the
neighbouring cells and give the test type a crash rate above 0, so that its crash is not held up
and does fall (S-0502 claim 18 and its Edge cases): it stays in play until it reaches the
ground.

*Falsified if:* the death weapon goes off in the air for a non-exploding aircraft, the detonation of
variant b has an amount other than the aircraft's remaining health, or the vehicle of variant c
leaves play before it reaches the ground. Variant d: repeat the shot-down aircraft with its
explodes flag on and off. Expected: on gives death weapons at both death and impact; off gives
one only at impact. A missing second detonation or one at death in the flag-off case falsifies d.
Give the aircraft two distinct explosion-list entries. Record the selected animation at the
death position and replay its list pick and three crash-start draws separately from detonation
draws; a missing or wrong animation, wrong position or different stream state falsifies that
choice, count or order. Variant e: kill the aircraft on its pad at terrain
height: it leaves play at the end of death effects without crash-start draws; delayed removal or
those draws falsifies e. Variant f (Q-0236): use a named test-map override that clears the
same aircraft type's explosion list with a no-type item, rather than empty text, which would
keep a lower layer's list. Disable its debris and exploding properties, carry no passengers or
bomb, and use an ordinary lethal weapon with no random impact effects. Give the type a distinct
nonempty destruction-animation list to distinguish a fallback to it. Kill 30 aircraft in flight
and 30 on a pad, confirming terrain height with the recorder for the grounded kills. Record
created animation types, positions, height, removal and scenario-stream states immediately
before the killing hit and after the death effects, separately from later crash impact.
Expected: no aircraft-specific explosion animation and no fallback animation in either case,
with no raw value consumed for the absent list pick. The flying case still takes its three
crash-start draws; the grounded case takes none of those. Replay all remaining effects under
their owning rules, rather than treating an entire death as necessarily draw-free. Repeat both
heights with one explosion-list entry, then the two entries above: each nonempty list takes
exactly one raw value for its list pick, even the one-entry list, before any crash-start draws.
A fallback animation, an extra list-choice draw with the empty list, a missing pick draw with
one entry, or a height-dependent list-choice rule falsifies f. A trace that cannot separate
weapon impact, death effects and later crash impact is inconclusive. Variant b cannot run until
a normal input or named-data setup that starts a living crash is established; it provides no
measurable backing in this revision. S-0501 scenario AC5 tests dead flying-aircraft crash
motion, distinguishing the counter contribution from net descent. Rocket interception is
instead tested by S-0506 scenario R13, not by D11's ordinary impact expectations.
Magnetic drops use S-0607 scenarios L3 to L6, including L4's crashable-vehicle and aircraft
variants; D11's ordinary impact predictions do not override those outcomes.
Variant g (claim 16, fall-back crash): run S-0501 scenario AC29 part f with the random-state
recorder, with the test aircraft type given minimum debris 1, maximum debris 4, one sprite
debris animation and an explosion list of two entries, so that a killing hit would take a
debris count draw and a list pick. Replay, under their owning rules, the draws of the tick in
which the fall-back crash sets the aircraft's health to 0 and starts its crash. Expected: the
crash start takes the three tumbling draws in the order of claim 16, and no explosion-list
pick and no debris draws. A missing tumbling draw, an extra list pick or debris
draw, or a different stream state falsifies g.

Variant h (claim 15, Q-2793): a test copy of a stock ground vehicle with the crashable flag on,
standing on an intact high-bridge deck over flat land, killed by one ordinary hit. Record its
height above the surface and the terrain, in-play membership after the hit and the scenario stream
state. Expected: height 0 above the surface, 416 above the terrain; it leaves play at the end of
its death effects, takes no crash-start tumbling draws and does not fall. A crash fall from the
deck, a removal on a later tick, or the three tumbling draws falsifies h. Its control is a run of
variant c in the same session, with a test type that has the same crashable flag: killed while
above the terrain off any deck, that vehicle crashes. A control that does not crash voids h.

**D12. Contacts of a destroyed building (claim 17).** *Setup:* a stock building type that produces
vehicles, destroyed while a vehicle is leaving it; a stock building type that harvesters dock at,
destroyed while a harvester is docked. *Observable:* whether each contact survives, and the kill
credit. *Expected:* the vehicles in contact within 256 leptons of the building's reference point are
destroyed with no kill credit. Variant b: a stock building type with the helipad flag destroyed
while an aircraft is docked on it: the aircraft is destroyed whatever its distance. *Falsified if:*
they survive, the docked aircraft of variant b survives, or either contact death credits an attacker.
For bunker contacts, use S-0606 scenario K5: compare an occupied bunker with an entering
vehicle that holds contact but no bunker link. Record the relation just before the destroying
hit and isolate collateral damage. Expected: the bunkered vehicle is let out without this
claim's demolition hit or run-away instruction. An entering vehicle at a flat 2 × 2 bunker's
docking-cell centre is within 256 leptons and receives the ordinary contact hit. Treating the
released bunkered vehicle as such a contact, or exempting the entrant, falsifies the distinction.

**D13. Survivors (claim 18).** *Setup:* a test 3 × 3 building with the crewed flag, explodes flag
off and no sale in progress, whose refund gives *C* = 5 (at least five times the divisor of the
owner's side), destroyed 300 times; variant b after a capture; variant c with a planted charge. *Observable:*
the number and types of survivors. *Expected:* per cell a chance of 1/3 (b: 1/9, c: 1/2), never more
than *C* survivors, types by F4; in variant c the survivors are ordered to attack the unit that
planted the charge, not the unit whose weapon destroyed the building. Variant d: destroy the building with S-0011's trigger kill, which ignores defences: no survivors.
Variant e: a test building type that produces buildings, also non-exploding and not being sold,
300 kills; the share of engineers among the survivors of a building
that has not changed owner, and the behaviour of survivors of a human-controlled and of a
computer-controlled owner and of the neutral house some ticks after the kill (no planter). *Expected of e:* about a quarter of
the survivors are engineers (the engineer draw is below 25), the others by F4 (about 15 %
technicians among non-engineers if armed); a human owner's survivors move, the computer player's
and neutral and special houses' hunt, using the common owner-order protocol. *Falsified if:*
the chance or type shares fall outside 3 standard errors of their predicted conditional
probabilities (chance is evaluated only before *C* is reached), more than *C* appear in this
single-round setup,
variant c's survivors attack the destroyer rather than a live hostile planter, or recorded
behaviours differ. Use distinct live planter and destroyer units in c; repeat with the planter
allied to the owner, when the attack order must be absent. Record survivor health and ordered
type, creation and health draws: health outside 5 to its strength or a replay mismatch falsifies
this part. Variant f: give divisor 0, and clear the crewed type flag, in separate kills. Each gives
no survivors and no per-cell smudge steps; an occurrence falsifies
the exclusion. With crew present, use invented refund/divisor pairs giving quotients below 1,
between 1 and 2, and above 5. Expected per-round limits: 1, 1 and 5, from refund rather than cost.
Record 300 kills per pair and replay the stopping point. A release beyond the limit or continued
survivor-chance draws after reaching it falsifies F5. D14 g tests the separate second-round limit.
Variant g (the survivor's standing position): the base building with own infantry already on two
positions of its first foundation cell (placed as D16 variant d occupies a foundation cell), and,
separately, on all of that cell's positions; a run in which the map does not keep those infantry
in the cell is inconclusive. Record each survivor's position and the draws the recorder reports.
*Expected of g:* each survivor that a cell releases takes one draw over 0 to 3 after its creation
draw and stands on the first free position of the table O row that draw picks; with every position
taken, a survivor of that cell still takes the creation and position draws, takes no health draw,
never enters play and does not count against *C*. A different position, row or draw count
falsifies g. Variant h (the crewed flag's absent-key default): two test building types,
defined wholly in the named test file (not copied from a stock section), that differ only in that
one sets the crewed flag and the other leaves it unset in its section, with refund and divisor
giving *C* = 5; sell each (S-0408 claim 7, which takes no chance draw).
*Expected of h:* at least one survivor from the first, none from the second (a survivor attempt can
fail to be placed, S-0408 claim 7). A survivor from the second falsifies the default. A run in which
the first releases none is inconclusive.

**D14. Building destruction effects (claims 19 to 21, 23).** *Setup:* test buildings with explosion
lists and the crewed flag off (so *C* = 0), one of 1 × 1 and one of 3 × 3 foundation, each
destroyed 200 times on flat, morphable, clear ground with no pre-existing smudges or overlays,
restoring those conditions between deaths. *Observable:* explosion animations and their positions, smudges,
rubble overlay, crate, the tick the building leaves play, the tick each barrel fire starts, and the
number of raw values the recorder reports for the destruction. *Expected:* one explosion animation
per foundation cell at F2's predicted nearby point with *r* = 64 leptons (coordinate truncation
can put the resulting integer point slightly more than 64 leptons from the cell centre); for the 3 × 3 building exactly one smudge
per destruction (claim 19 (a)), at the centre of the cell of its position, and none at all for the
1 × 1 building, with scorch marks and craters about half each over the 200 destructions (claim 19
(a): below 50 a scorch mark). Variant b: a building type that leaves rubble leaves the rubble
overlay of registration index 239 on every foundation cell; record overlay data immediately
before the kill and after rubble placement, expecting it unchanged. A 1 × 1 type with a money crate
beneath and no rubble leaves a money crate on its cell. The same type is then destroyed (i) with an
overlay of another type laid on its cell before the kill, and (ii) given the rubble flag and
rubble image of b's rubble type as well; a test rules file gives both overlay types a land type
that *track* can enter (S-0211 claim 10) and none of the wall, rock and crushable flags.
*Expected* in (i) and (ii): no crate appears in the tick the building leaves play, the
cell keeps its overlay ((i)'s, or the rubble in (ii)), and the number of raw values the recorder
reports for the destruction includes none for a crate (S-0520 claim 3's overlay draws and
lifetime request). A wrong index, changed data or a different crate cell falsifies b; so does a
crate in (i) or (ii), a replaced overlay, or raw values taken for a crate there. Variant c
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
that a step does leave lies at the cell of its nearby point. Also give the base test buildings two distinct destruction-animation entries. Expected: one
entry at the building's position, matching its list-pick replay after the foundation explosions
and any barrel-fire effects, with no resource-spill draws in these empty-storage setups. A
missing or wrong entry, wrong position or replay mismatch falsifies this part.
Variant f: a test building type with a destruction particle-system
list containing one smoke-behaviour particle-system type defined by named test data, with no attached damage system
before the kill and a named displacement of (32, 48, 64) leptons. Use the read-only recorder
from the killing hit through the system's first turn. One empty system is created at the
building position plus that displacement. For a type without the explodes flag, killed while
not being sold, the building leaves play immediately: the system is marked before its first
turn, emits no particles and leaves play in that turn. Repeat with the building being sold:
it instead awaits removal under claim 21, and the system is marked when that removal occurs.
Any emission before then must satisfy S-0616 claim 9. Absence, a second system, a different
position, emission after immediate removal or a mark before the selling building's removal
falsifies the corresponding case. This does not settle the eligibility of other kinds.
Variant g (claim 21, the second round): a crewed 3 × 3 test building type with the explodes flag
whose refund gives *C* = 1, destroyed 300 times, with the recorder running from the killing hit
through the end of its removal turn, including the outcomes after it leaves play. *Expected of g:*
survivors appear at two moments, at the kill and at the
removal turn, so some destructions release 2 survivors; smudges from the per-cell steps inside
the foundation appear only at removal (claim 19(a)'s central smudge can already exist at the kill).
The building is already absent from the update order when a second-round
survivor enters it, and is absent for that round's smudge steps. Record update-order membership
alongside survivor creation and foundation vacancy to distinguish leaving play from later final
removal. Each run's raw-value count matches the replay with the second round. With
`quirks.building_removal_second_round` off our engine releases at most 1. *Falsified if:* no
destruction releases 2 survivors over the 300 runs, a survivor appears in a tick between the
kill and the removal turn, or the building is still in the update order when the second round
releases a survivor or takes a smudge step. Variant h (the sale and postponement): kill a crewless selling test
building (i) before its release processing, (ii) during build-down, and (iii) just before the
processing that would pay its refund. Record its health, presence, foundation occupancy, house
credits, released units and random-stream state across the killing hit and every turn until
removal. Repeat using a named test weapon with a non-damaging electromagnetic-pulse warhead lasting
longer than the kill interval, and separately while a stock erasing weapon holds the building.
Use a second, ordinary weapon for the killing hit. Confirm the pulse or erasure is still active
at 0 health with the recorder; failure to arrange that overlap is inconclusive. *Expected of h:* no sale refund or further sale releases after
health reaches 0; the same occupied-foundation awaiting-removal state as c, and the second round
of g when crew is enabled. Pulse or erasure postpones removal until the first eligible own turn.
There is no second round during the wait. A refund, further sale release, early foundation
vacancy, second round during the wait, or removal before the first eligible own turn falsifies h.
*Falsified if:* a smudge appears on a cell other than the centre of the position cell while
*C* = 0, the 3 × 3 building leaves no smudge or the 1 × 1 building leaves one, the share of scorch
marks lies more than 3 standard errors from one half, an explosion position or starting delay
differs from the exact F2 and draw replay, the raw-value count differs from the replayed count, or
any other statement of the variants fails. Variant i (the clear-for-movement test of claim 19 (g)):
a crewed 3 × 3 test building with *C* ≥ 1 whose foundation is built with a map editor to cover, in
separate maps, a beach cell, a cell with a crushable overlay that is not a wall, a cell with a wall
overlay, and a cell holding a terrain object; each map also with a named test file that sets the
tracked class's multiplier of that cell's land type to 0 and, separately, to a value above 0
and below S-0211's 0.01 threshold as S-0211 parses it. Add a test building type with a landing pad whose landed aircraft stands on a
foundation cell
when the building is destroyed. That aircraft is in contact with the pad, whose type has the helipad
flag, so it does not survive: claim 17's helipad rule destroys it with the building, whatever its
distance. The part tests test 1 only in a run in which the recorder still shows its object-present
flag set on that cell at the smudge step, and is inconclusive otherwise. Destroy each 50 times.
*Expected of i:* the smudge step's percent draw and nearby-point raw value are present for the cell
exactly when tests 1 to 3 pass: absent for the wall overlay, the
landed aircraft and the test file's zero multiplier, present under the test file's
below-threshold multiplier for every other map, the terrain object's included, whatever S-0211's
category of the cell. A step present or absent against this falsifies i. In particular, a step
absent under the below-threshold multiplier, where tests 1 and 2 pass, shows a threshold rule
rather than test 3's exactly 0.

**D15. Spilled resources (claim 22; conditional, not runnable yet).** No known skirmish map
setting or start-credit conversion fills building storage, so claim 22 is unmeasured. If a
normal-input or named-data path is established, record it before running this proposal.
*Setup then:* a building holding a known whole amount of each of two resource types, destroyed. *Observable:* the house's stored total, and resource units by cell.
*Expected:* the house's stored total drops by that amount and the same number of resource units
appear in cells 1 to 3 cells away (256 to 768 leptons from the building's position), the units of
the earlier type in registration order being drawn first. *Falsified if:* the total drops by another
amount, the number of units differs, or a unit appears in a cell that contains no point 256 to 768
leptons from the building's position (a unit goes to the centre of the cell of its nearby point, so
the cell centre itself can lie somewhat beyond 768 leptons).

**D16. Absorbed passengers (claim 24).** *Setup:* a stock absorbing building given named map
overrides to keep its explodes flag off, holding three individually identifiable infantry
entered in a recorded order, destroyed while not being sold.
*Observable:* each passenger's release cell, initial facing, owner and behaviour before scatter,
and its owner's crate infantry count immediately before and after release.
*Expected:* the first three foundation cells in cell order receive the passengers in reverse
entry order, facing the building. Each released passenger keeps its owner and, if uncounted,
adds 1 to that owner's crate infantry count; the human player's passengers receive only the
scatter request. Entry-order release, a changed owner or a missing count restoration falsifies
these predictions. Variant b: the
same building with the crewed flag on and *C* ≥ 1, 100 deaths. Survivors and per-cell smudge steps
start after the first three cells in this single round. This does not exclude claim 19(a)'s
central smudge or smudges whose nearby point falls into a neighbouring cell: replay the step's
source cell separately from the smudge's placement cell. Variant c uses the common owner-order
protocol: computer, neutral and special owners' passengers hunt. *Falsified if:* fewer passengers
appear while placement is free, initial facing or behaviour differs, or a survivor chance or
smudge-step draw is attributed to one of the first three cells.
Variant d: use S-0011's defence-ignoring trigger kill.
All passengers die and none get out; without a charge planter nobody gets their kill credit.
With crew present, occupy all standing positions of the first foundation cell: its passenger
dies, while the next still uses the second cell, not the first. Any escape in a no-crew case,
credit to the ordinary killing attacker, or cell reuse after failed placement falsifies d.
Placement position and draw (claim 24, infantry): record each released infantry unit's position and the draws the recorder reports for its placement. Expected: each open cell's unit takes one draw over 0 to 3, and its position is the first free position of the row that draw picks in table O of S-0208 (claim 3); a unit in a cell that refuses it takes no draw. A different position, row or draw count falsifies this part.
Vehicle passenger placement and credit to a separately remembered planter remain unmeasured:
no setup here establishes vehicle absorption or a surviving planter for the failed releases.
Variant e: set the side's survivor divisor to 0 with crew otherwise present. The three passengers
still come out although *C* = 0. No passenger release in this case falsifies that independence.
Variant f: the main case with named overrides that turn the building's crewed flag off, destroyed
while not being sold. *Expected of f:* the three passengers come out as in the main case, and no
survivor is released (*C* = 0). A missing passenger release, or a survivor, falsifies f.

**D17. Smudge choice (claim 26).** *Setup:* named test files that replace the smudge type list
with test types: two scorch-mark types of footprint 1 × 1, one scorch-mark type of footprint 3 × 3,
and one crater type of footprint 1 × 1. A crewless test building of foundation 2 × 2 with an empty
explosion list stands on flat clear ground of a morphable tileset, so its position cell is the
anchor and the 3 × 3 footprint reaches one row and one column beyond the foundation. The
random-state recorder runs. Parts: (a) 200 destructions. (b) The same with the 3 × 3 scorch type
removed. (c) As (a) with a sloped cell or an overlay in the row or column of the 3 × 3 footprint
outside the foundation. (d) Retain the runs of (a) whose recorded percent draw gives a crater; never set the stream state. (e) The building
placed on a tileset without the morphable flag. *Observable:* the smudge type left at the building's
position per destruction, and the number of raw values taken between claim 19 (a)'s percent draw
and the next known draw. *Expected:* (a) every scorch mark is the 3 × 3 type (the large form
prefers it) and the type choice takes no raw value; (b) the two 1 × 1 types appear about half each,
with one raw value for the choice; (c) the 3 × 3 type no longer fits, so the 1 × 1 types appear as
in (b); (d) the crater appears and its choice takes no raw value; (e) no smudge and no raw value
for the choice. *Falsified if:* a 1 × 1 scorch type appears in (a), the shares in (b) lie more than
3 standard errors from one half, a smudge appears in (e), or a raw-value count differs from the
replay. Variant f: use named test theater files with tile 0's morphable flag first off and then
on. Put the no-tile marker in a flat, empty cell of the footprint outside the foundation, and
repeat with a map-produced stray tile number that survives until the test (S-0010 scenario R1
part D2). Keep the other footprint cells morphable. Record the cell's tile number at the fit
test; a run in which it was recalculated first is inconclusive. Expected: the 3 × 3 footprint fails with tile 0's flag off and fits with it on, subject to the
other fit conditions. The 1 × 1 footprints at the anchor do not touch the changed outside cell
and remain eligible in both runs.
Acceptance with that flag off, or refusal with it on while all other conditions pass, falsifies
the fallback rule. Variant g: repeat a with an existing smudge in the outside row or column of
the 3 × 3 footprint; it refuses that type just like c's overlay. Repeat a second destruction at
the same anchor while its previous smudge still exists: no overlapping smudge is placed and no
type-choice draw is taken when all candidates are refused. A large type accepted across an
existing smudge, an overlapping second smudge or an unexpected type-choice draw falsifies g.
Record all fitted candidate types in registration order and compare every chosen entry against
its R(0, *n* − 1) replay, not merely the frequency histogram; a different choice falsifies the
registration-order mapping.

**D18. Defaults of the crew, survivor and sinking values (claim 27).** *Setup:* named test files
that omit the crew escape chance, the sinking weight and the three survivor divisors from every
layer, started on a freshly launched game. Parts: (a) a stock crewed vehicle with no passenger
capacity killed 500 times; (b) test naval types of weight 2.9 and 3.0 killed on water; (c) a crewed
test non-exploding building, not being sold, of a side-0 owner whose refund is 1000 credits,
destroyed 100 times, and the same for
side-1 and side-2 owners. A second run of each part follows an earlier game of the same session
whose files gave other values. *Observable:* the share of kills with a crew; whether each vessel
sinks; the largest survivor count per destruction. *Expected:* (a) a share within 3 standard errors
of one half; (b) the vessel of weight 3.0 sinks and the one of 2.9 does not; (c) at most 5, 5 and 3
survivors (1000 / 200, 1000 / 100 capped at 5, 1000 / 300 truncated); in the second runs, the earlier
game's values. Record and replay each destruction's survivor chances and stopping point as well as the
largest count: a different count from the replay falsifies the divisor even if no run exceeds
its predicted maximum. For carry-over, name invented earlier-game values in the measurement
record and compare with their predicted crew chance, sinking boundary and F5 count, rather
than expecting a fresh-start reset. *Falsified if:* any of these differs.

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
third shows one, or the voxel types, maxima pairings, particle types or replay differ.
Variant b: use named test lists of repeated short valid animation or particle-system names,
whose last complete item ends at byte 127, followed by comma and another valid short name. Only
the prefix through byte 127 survives the reader limit; repeated entries keep their places. Test an absent key as well as an empty one after a nonempty lower layer, and a nonempty
list consisting only of no-type items. Repeat with a space within a name item, using distinct
registered names with and without that space, as S-0003 scenario V7 tests untrimmed items.
Expected: absent and empty keys retain the lower list, all-no-type items clear it, names with
spaces follow the literal item, and list lengths are those after truncation and omission.
Different retained entries, lengths or subsequent draw outcomes falsify these syntax cases.

**D20. Destruction-trigger timing and retained infantry carriers (claim 29; Q-0268 parts 2
and 3).** Use the common measurement setup at speed 6 with no engine add-on and no mods (the named test files of a scenario excepted).
The observed area is within 10 cells of the test objects and action waypoint cells. Named test-map
objects carry tags whose one-time destroyed-by-anything action starts a distinguishable
animation at a remote waypoint. Choose a named test animation with known scenario-stream
creation draws, and give the dying types distinct sprite debris, a death weapon, and a
kind-specific death animation. Kill tagged infantry, a vehicle, an aircraft and a building
with ordinary lethal hits. Record and replay the stream state, animation creations, health,
passenger deaths and removal, separating incoming-weapon effects from death effects. The
trigger animation's immediate draws and effects must precede the dying object's debris and
all remaining death effects. A later action, or action draws after death-debris draws,
falsifies the timing; an unresolved within-hit trace is inconclusive. Repeat without a known
attacker using a separate polled tag's explosion-at-a-waypoint action (S-0011 claim 28),
with a weapon lethal to the test object. The separate tag must not be the dying object's
counter tag, and the explosion must not hit the counter target. Destroyed-by-anything still
fires before debris. Do not use destroy-attached-objects for this control: it damages carriers
of its own firing tag, whose nested raises are suppressed while that tag is being raised
(S-0011 claim 12), so it cannot isolate the no-attacker destruction raises. Run 30 kills per
kind and variant.

*Repeating groups:* attach a repeating tag with a single destroyed-by-anything event and an
explosion-at-a-waypoint action aimed at an isolated durable neutral building. Use S-0011's
calibrated counter protocol: a constant positive health loss per firing, enough health to
survive the run, no healing or repair, and no unrelated damage. Divide the recorded health
loss by the calibrated single-firing loss using exact integer arithmetic, before any later
poll. Give each trigger an owner country no house plays, avoiding owner-house raises. Fresh tags and full-health objects ensure no earlier latch. With the
original's persistent latch, a normal vehicle killed by a known attacker gives seven firings
within the killing hit, compared with two for infantry, aircraft and buildings; without an
attacker the vehicle gives four and the other kinds two. Use the separate polled tag's
lethal waypoint explosion above for every no-attacker count, not destroy-attached-objects.
Each action's effects and draws precede death debris. A different count or position in the replay falsifies the groups.
On our engine with the extra-raise quirk off and persistent latching on, the vehicle gives
two. With `quirks.persistent_event_latch` off and the extra vehicle group on, it gives two
firings, one per destroyed-by-anything raise, with or without an attacker. With both switches
off it gives one. A different count falsifies the corresponding alternative; these switch-off
runs test our engine, not the original. Repeat the original's
vehicle case after takeover by a vehicle thief, first with the tag-carries-over flag clear and
then set on one definition in its chain: the flag-clear case retains the extra group, the
flag-set case gives only the common group's two firings. Record the remembered hijacker type
before killing; absence of that state makes this variant inconclusive.

*Deferred deaths:* repeat with a cyborg killed while initially not prone, a tagged building
eligible for delayed death and a named delay-kill warhead, a sinking vessel, and an aircraft
shot down in flight. A one-time tag fires on the building's initial lethal hit, on the vessel's
killing hit and on the aircraft's killing hit, not first at expiry, sinking completion or
impact. The cyborg's reprieve fires nothing; a later lethal hit while prone fires normally.
Use a repeating building tag and the calibrated counter action to verify a second common destruction
group at countdown expiry if the tag is still attached, recording later poll firings
separately. Any reversed moment falsifies these cases.

*Retained infantry:* pre-place two infantry sharing an all-carriers tag with the same single
event and calibrated counter action. Give the first a long positive-count death sequence through named test
art. Kill it, then kill the second before the first sequence completes. The first raise
satisfies but detaches only the first carrier, with no counter health loss; the second death now fires the
action once. In a separate run, use a repeating tag shared by the two infantry. It stays attached to the first while that unit plays death; record attachment read-only,
and distinguish later polls from destruction raises. No fresh
destruction group occurs at sequence completion. Loss of the repeating tag just on starting
death or an extra removal-time destruction raise falsifies the claim. S-0011 owns the exclusion from present-object actions; retaining
the tag must not make change-owner act on dying infantry. Perform five runs of each carrier
variant. Q-0268 part 1 is owned by the movement specs, not by this death scenario.

**D21. Slaves after master destruction (claim 30; Q-0270 part 3, Q-0413 part 4).** Use the common
measurement setup, speed 6, no engine add-on and no mods (the named test files of a scenario excepted), with stock slave miners on a tiny flat
resource-field map and the bystander kept outside the observed area (within 10 cells of the
master and its slaves). Use ordinary harvesting to obtain a mixture of slaves outside and
inside; log that state before the kill rather than assuming every slave has emerged. Destroy
30 vehicle masters and 30 deployed building masters with an ordinary enemy attack. Separately
use a polled explosion-at-a-waypoint action lethal to the master to obtain a no-attacker hit, and
confirm the source object is absent. Keep outside slaves beyond the lethal incoming area where
possible and record any collateral hits separately.

Record slave identities, positions, health, owners, master links, queued behaviours, deaths
and removal from immediately before the hit through its death effects. Expected: hidden slaves
are removed with attacker credit when there is an attacker and without credit otherwise.
Outside slaves retain identity, position and health at release, lose their master, clear target
and destination, queue guard and receive a cheer request. Record the sequence request
separately from later animation completion or interruption. Their new owner is the attacker object's house, or the first
civilian-side house in creation order for the no-attacker run. Repeat with an allied attacker
using a named test-map weapon that damages allies: alliance does not suppress transfer. Give
the slave type named positive debris and a death weapon through test-map overrides to
confirm their absence for hidden removal. Record hidden removal relative to the master's
own debris creation, giving the master a positive debris count through the same named
test-map overrides; removal after that creation falsifies its precedence. This setup does
not establish tag attachment to generated slaves, so hidden-slave tag notifications remain
unmeasured; merely recording removal cannot test them. Repeat with an exploding master and a death weapon whose area
can hit an outside slave: transfer must precede that detonation's damage; later death of that
slave is not a failed release. Continue recording for longer than the imported regeneration
delay plus 20 ticks: this dead master produces no replacements. Missing hidden removal,
wrong owner or release state, a recreation or healing, a reversed death-weapon precedence,
or a replacement from the dead group falsifies the corresponding result. A trace that cannot
separate incoming collateral damage from release and death-weapon damage is inconclusive.
No draw-count prediction for the freed-slave presentation sequence is asserted in this scenario.
Variant b: give the deployed building master delayed-death eligibility and use a named lethal
delay-kill warhead with a positive countdown. Record the same group states across the initial
hit and expiry. Expected: the initial reprieve does not free or remove the group; the actual
death at expiry ends it under claim 30. Early group release or a retained group after actual
death falsifies that boundary.

**D22. Master removal without a killing hit (claim 31; Q-0413 parts 2 and 4).**
Use D21's map, speed, houses and observation area, with a deployed building master and
its existing group. In 30 runs, obtain both hidden and on-map slaves, then request pack-up
with a move click. Use S-0415 claim 11 and its D8 refused-placement setup: a named hostile
blocker makes the return cell unenterable at completion, without damaging the master or
slaves. Confirm the return vehicle's placement was refused and the building removed, with
no old-group transfer. A successful pack-up is a control: the old slaves stay with the
return vehicle, not a civilian recipient; temporary hidden-slave cleanup in that control
is separately S-0402 claim 19a and H24.
Record pack-up completion, placement refusal, master removal, each old slave's identity,
health, position, owner, master link, target, destination, queued behaviour and sequence, and continue for the imported
regeneration delay plus 20 ticks. Confirm that no killing hit, earlier group-ending event or
replacement-master transfer occurred; otherwise the run does not test claim 31.
Expected: hidden slaves are removed without attacker credit or their own death effects.
On-map slaves transfer to the first civilian-side house, retain health and position, clear
target and destination, queue *guard* and receive a *cheer* request. No replacements appear.
A retained slave-master link, different recipient, recreated or healed slave, retained target
or destination after transfer, missing guard queue or cheer request, or later replacement
falsifies the corresponding prediction. Failure to arrange refused placement without damage
or an earlier group ending makes the attempted setup inconclusive. A missing hidden removal
or any old-slave debris, death weapon or death sequence falsifies that suppression rule.
No absent-recipient prediction is tested
by this civilian-recipient setup; that path remains unmeasured, rather than being inferred
from successful transfers. No release draw-count prediction is asserted.

**D23. Repeated sinking deaths and culled hosts (claim 32).** Use the common setup at
speed 6 on a tiny water map, no engine add-on and no mods (the named test files of a scenario excepted),
apart from a named map override "sinking-repeat". Use a surface vessel meeting claim 10 as a control, and two cullable
naval vehicle variants, one below the sinking weight and one underwater. Keep death image
frames absent and crashability off initially. Begin the variants' sinking with S-0614
scenario P6's cull. Confirm unchanged health at cull, then deliver a lethal area hit from
an identified enemy before descent ends, rather than relying on a new target assignment.
For the control, arrange a second already-fired or area hit after its first killing hit.
Run 30 trials per case, recording health, height, in-play membership, attacker experience,
house score, kills and losses, tag attachment, animations, passengers, crew and stream state
immediately around each hit and through removal. Use D20's repeating-tag counter and
separate action target, distinguishing later tag polls from within-hit raises.

Expected: each actual killing hit registers destruction under the ordinary attribution
and exclusions and raises the still-attached tag under claim 29. The control returns to
1 health without a height reset or later removal tick. An ineligible culled vehicle ends
its hit at 0 health but remains in play, with explosion and destruction-animation list
picks. Repeat with distinguishable sprite debris and an exploding death weapon, then with
passengers, a crewed type and a planted bomb in separate runs. Replay the applicable shared
and vehicle effects from their current state: no duplicate already-lost passengers or bomb,
but a fresh crew chance when eligible. Include a control capable of releasing a crew on
land through named amphibious crew test data, so placement failure on water cannot hide a
second crew attempt. In a crashable ineligible variant already below the surface, expect
immediate removal instead of retained zero-health sinking. Repeat the marked-water case
using a bridge-drop fixture only if it establishes the mark on the same descending host.
A fixture failing to establish cull, hit delivery, passenger loading, bomb attachment or
the intended mark is inconclusive for that part, not evidence of suppression.

Missing repeat credit or raises with their ordinary conditions met, a different health
result, an unaccounted effect/draw, a surface reset, or failure of the stated immediate
crashable removal falsifies the corresponding rule. Merely missing the short interval is
inconclusive. Types with death image frames are an exploratory recording only: this
scenario makes no prediction for their untraced path. No measurement is claimed completed.

**D24. Slaves of a cancelled production master (claim 31; Q-2973).** Use the common measurement
setup, speed 6, no engine add-on and no mods (the named test files of a scenario excepted), on a
tiny flat map that holds no resources, where the computer house owns nothing that can attack and
neither house owns a harvester. The human player's house owns a construction yard, one factory
building that can build a vehicle type whose enslaves field names an infantry type and whose slave
count *k* is positive (S-0402 claim 19), one building of each of that type's prerequisites and
enough credits for the whole item. In 10 runs per part, start the master type's production and
record, from the start of production to 200 ticks after the item ends, the number of the house's
objects of the slave type, the house's loss and kill totals as S-0414 scenario V17 records them,
every house's kill total and score, and the messages shown. In parts (a) to (c) a named test-map
override gives the slave type no not-scored mark.
- Part (a): give a cancel order for the item at about half progress.
- Part (b): sell a prerequisite building of the master type during production, so that the
  first recheck after its removal takes the icon away and gives the cancellation orders
  (S-0411 claim 2).
- Part (c): sell the factory building during production (S-0404 claim 8, or S-0411 claim 2 when
  its recheck cancels the item first).
- Part (d): as part (a), with a named test-map override that gives the slave type the not-scored
  mark.
- Part (e): as part (a), with a vehicle type whose enslaves field is empty in place of the master
  type, built by the same factory building.
- Controls: the same production completed and the unit placed, and a run with no production.

Expected: in parts (a) to (d) the house's count of slave-type objects rises by *k* at the start of
production and falls by *k* in the tick in which the cancellation takes effect. In parts (a) to (c)
the house's loss total rises by *k* in that tick. In part (d) it does not change. No house's kill
total changes in any part, no house's score changes in parts (a) and (d), and no message names the
slaves. In part (a) the messages shown from the cancellation to 200 ticks after it are those that
part (e) shows over the same span. A loss total unchanged in (a) to (c), slave-type objects left
after the cancellation, a kill change, a score change in (a) or (d), a message in part (a) that part
(e) does not show, or a loss in part (d) falsifies the corresponding prediction. A run in which the
item is done before it is cancelled, or in which the sale of (b) or (c) does not lead to a
cancellation, does not test this claim.

**D25. Death weapons nesting inside one killing hit (claim 33; Q-3000).** Use the common setup,
with the shroud on, on a flat test map large enough for the path below, every vehicle at the
centre of its cell. A named test file gives a test vehicle type the explodes flag and a test
death weapon whose warhead has a spread of 1.25 (reach 320 leptons, S-0303 claim 3), an edge
fraction of 1 and enough damage to kill a test vehicle. So the detonation of a vehicle hits a
vehicle in a cell straight next to its own (256 leptons away) and kills it, and hits no vehicle
in a diagonally neighbouring cell (about 362 leptons) or further away. The type has no other
death effect of the State and ordering table that can reach another object: a maximum debris
count of 0 (so no debris and no debris draw), no passengers, no crewed flag (so no crew), no
crate, and it cannot crash. Every animation that the test types and weapons use does no damage.
Check the reach with three vehicles before the runs. Lay *N* test vehicles of the human player's
house along a path of cells, each straight next to the cells before and after it on the path:
rows along the map's x axis, two cells apart, the first row running from the first vehicle toward
decreasing x, and each pair of rows joined at alternate ends by one vehicle in the cell between
the rows' end cells. Then every two vehicles of the path that are not next to each other on it
stand at least a diagonal neighbour's distance apart, at the joins included, so each vehicle's
death weapon reaches only the vehicles before and after it on the path. Add one further test
vehicle *S* in the cell one step toward increasing x from the first vehicle. *S* is within reach
of no other vehicle of the path, and in S-0303 table T1's ring 1 the second vehicle's cell comes
before *S*'s, so the first vehicle's detonation hits *S* after the second vehicle.
Map trigger variable 1 starts clear. The last vehicle of the path carries a one-time tag whose
trigger has a single destroyed-by-anything event and sets variable 1, and *S* carries a one-time
tag whose trigger has the same event and clears variable 1 (S-0011 claim 22). A third, one-time
trigger *W*, in a polled tag attached to no object, has a variable-set event for variable 1 and
reveals the area around a waypoint at least 20 cells from every test object (S-0011 claim 26).
Give each trigger an owner country that no house plays. No other object carries a tag.

Kill the first vehicle with one hit of a named test weapon with no inaccuracy, fired by a unit of
a second house, an enemy of the human player's house, that stands at least 4 cells from every
test vehicle. Its warhead has a spread of 0.5 (reach 128 leptons), so its detonation at the first
vehicle reaches no other vehicle, every one of which is 256 leptons or more away. Before the
runs, fire that hit once at a vehicle of a second test type, the same but without the explodes
flag, standing in the first vehicle's cell with *S* beside it as above: that vehicle must die and
*S* must still have its full health at the end of that tick. In the runs, record the vehicles
still alive at the end of every tick until 30 ticks after the kill, and whether, and in which
tick, *W* reveals its area.
- Part (a): *N* = 50.
- Part (b): *N* = 600.
- Control (c): as part (a) but with no *S*.

Expected: in both parts every vehicle of the path and *S* are dead at the end of the kill's tick.
By claim 33 the last vehicle of the path dies inside the first vehicle's detonation before that
detonation hits *S*, so variable 1 is set and then cleared again within the kill's tick, and *W*,
which is judged only at its polls (S-0011 claims 6 and 22), finds it clear and never fires. If
*S* died before the last vehicle, as when an outer detonation's later hits come before the deaths
nested in its earlier ones, variable 1 would end the tick set and *W* would reveal its area at
its next poll. Any vehicle alive after the kill's tick, or *W* revealing its area within the 30
ticks, falsifies claim 33 at that depth. A run that does not complete the kill's tick is
inconclusive for the order at that depth. In control (c) nothing clears variable 1, so *W* must
reveal its area within the 30 ticks. If it does not, the trigger setup does not work as
described, and parts (a) and (b) are inconclusive. On our engine all three end as expected.

## Open questions

- Q-3000 question 2: whether a chain of death weapons nesting inside one killing hit (claim 33)
  makes the original fail once it is long enough, and at about what length, is not established.
  Our engine's rule does not depend on it (claim 33).
- Q-0270 part 3: slave-group destruction paths without an ordinary killing hit, including
  erasure and environmental sinking with remembered attacker or forced owner, remain to be
  integrated. Claim 30 answers ordinary killing hits and claim 31 the remaining group's
  attacker-less removal fate; neither establishes earlier erasure or sinking group endings.
  Sale belongs to S-0408 claim 10a; surviving-master capture and mind-control ownership
  are S-0402 claim 23. The release presentation's
  random draws and within-group notification order are not yet specified.
- Q-0193: which death effects, scoring or notifications and random draws a never-placed
  production item takes when a defeat or surrender hit kills it, and whether its own death
  immediately deletes it. These remain unestablished, not a draw-free-death rule. S-0414 claim
  17 already answers the existence-count part of question 3. Production-line retention,
  progress, completion, refund, release and placement, including survival of a factory without
  later cancellation, remain for S-0404. Off-map hit eligibility does not answer these parts.
  Q-0193 must be reopened at hand-off: S-0414's answer resolves only the defeat-count portion,
  not the whole request. This spec does not list Q-0193 as answered. The removal of a
  cancelled item is not such a death: claim 31 gives the fate of a cancelled master's slaves.
- The unplaced default position of a death weapon's projectile (claim 6): believed to be the map
  origin, which would put its warhead debris at the map's corner and take the land type of that
  corner's cell. Scenario D6, variant d, would show it.
- Vehicles whose type has death image frames (a sprite death animation) take another path at death that is
  not known (claim 9).
- How many absorbed passengers a stock building can hold compared with its foundation cells, which
  decides whether the departure for passengers beyond the last cell is ever reached (claim 24).
- Which events set and clear a building's charge planter (claims 18 and 24).
- Which situations of play are contacts of a building at the moment of its destruction (claim 17),
  beyond factory exits and docking.
- Crash motion of crashable movement modes other than ordinary flying aircraft, jumpjets
  and rockets remains for their owning movement specs (claim 16). The jumpjet motion itself
  is specified by S-0502 claim 18.
- Whether the state that suppresses survivors (claim 18, "the owner's game is not over") is set by
  the defeat of S-0414 itself or only by the destruction of all the defeated house's objects that
  follows it. The project lead's decision of 2026-10-11 (the game runs on until one alliance
  remains) does not change this: a defeated house stays defeated.
- How a building comes to hold resources in a skirmish, and whether a fractional holding can occur
  (claim 22).
- The relative order of resource spilling with other death effects of an exploding absorbing
  building (claim 1). This combined case is unmeasured until a storage-filling setup exists.
- Measurement of the destruction particle-system below-terrain boundary (claim 19(f))
  and the other boundary cases excluded from measurement in Backing boundaries remains wanted.
  S-0616 claim 1 resolves the eligible kinds and the attachment condition.

## Changes for other specs

S-0404 claim 6: "Delete the item's object" can point to claim 31 here (*A cancelled production
master*): deleting a slave master's object destroys and removes its hidden slaves, and each one
gives its house a loss, subject to S-0414 claim 21's victim exclusions. S-0402 claim 19 can point to
the same paragraph for the fate of a group created at the start of production whose item is then
cancelled.

S-0408 claims 7 and 8 and scenario S6 (e): the no-crew state of F5 arises only at a destruction
whose killing damage ignores defences (claim 18 here, *No crew*), so it never holds at a sale.
Claim 7's "A building with no crew (S-0312, F5) releases none" and claim 8's "A building with no
crew deletes them instead" can then never apply. At a sale, a type without the crewed flag
releases no survivors because its *C* is 0 by F5, and its absorbed passengers come out as from
any other building: the passengers' release tests only the no-crew state, not the crewed flag
(claim 24 here gives the same for a destruction). S6 (e)'s expectation "None come from the type
with the crewed flag off" and its falsifier "any passenger from the type with the crewed flag off"
should then be reversed: that type's passengers come out on its first foundation cells in reverse
entry order, as from the crewed type. Claims 7 and 8 can drop the two sentences or say that the
no-crew case does not arise at a sale.

S-0511 claim 10's Backing and scenario B3's retained-mark variant: they give the later low-water
death of a survivor as `unmeasured`, for lack of a legal route from the protected land landing to a
low-water death without another mark writer. A survivor that drives onto an intact bridge deck over
water and dies there passes claim 11's water test here (height 0 above the surface, the cell's
water land type), so S-0312 scenario D8's marked-deck variant, part a, is such a route, in the
original's behaviour (`quirks.bomb_mark_water_death_on_deck` on; claim 11 rates the effects on a
deck low). The Backing and B3's later-death comparison can point to it. The later fall without a
new mark stays `unmeasured` as it is.

S-0203 claim 13 (g): its *Sinking* bullet's "within 10 leptons above a water cell" can point to
claim 11's water test here (height above the surface, no lower bound). Its *Crashing* bullet's
"above the terrain" is a height above the surface in claim 15 here, for a killing hit on a vehicle
that stands on a deck. Its Edge case on a vehicle whose crash ended on a bridge deck is not settled
here and stays as it is.

S-0203 claim 6 (a): "a vehicle whose type can crash and which is above the terrain when it is
destroyed starts to crash" reads, for a vehicle, its height above the surface (claim 15 here). A
crashable vehicle that stands on a bridge deck when it is destroyed is at height 0 above the
surface: it starts no crash and leaves play at the end of its death effects, with no crash-start
draws (claim 15, scenario D11 variant h). The claim should say "above the surface" for a vehicle
and point to claim 15 here; for an aircraft it stays "above the terrain" (claim 16).

S-0212 claim 25, table T5: add a row *Building crate beneath (S-0312 claim 20)* with these
settings: the building's top-left foundation cell as centre, *float* when the centre's land type
is *water* and *track* otherwise, *normal* movement zone, no reach region, deck mode never, no
height condition, no further test and no reference cell. Add it to the rows that allow bridge cells and ask for no no-overlay
condition (with `quirks.crate_beneath_overlay_refusal` off it asks for that condition), to the
sentence on trigger and random crates whose selection is not placement (S-0312 claim 20 owns
its refusals), and to the tick list: the tick of placement, which is the tick the building leaves
play (S-0312 claim 21).

## Questions for the project lead

- Decided (2026-10-09, item 4; 2026-10-10 afternoon, item 2): AI houses follow the unit rules and
  order legality of human-controlled houses, and the neutral and special houses keep the original's
  rules for owners that are not human-controlled. Claim 25 applies both.
- Decided (2026-10-11): the game plays on to the last alliance. This spec has no rule that depends
  on the end test; claim 18's condition on the owner is unaffected (Open questions).

The former passenger-order question is resolved by applying the existing neutral/special-house
ruling and the stated exclusion of the team system: claim 25 retains the original's non-team
*hunt* order. No new project-lead decision is requested.

S-0203 claim 6, whose three passages say a crashing object "stays in play until it hits the
ground" (citing claims 15 and 16 here): for an object using jumpjet movement, S-0502
claim 18 says when the crash ends, and a held-up crash can keep it in play indefinitely in the
original (`quirks.jumpjet_crash_lookahead` on). Those passages can add that cross-reference.
