---
id: S-0614
title: Parasites
topic: combat
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0614: Parasites

## Summary

Skirmish parasite attachment, drone and grappling deliveries, hold effects, release and experience.
S-0315 claims 6 and 7 own suppression by damage and healing; network games and campaigns are
out of scope.

## Terms

Shared terms used here are defined in the glossary: *parasite, victim*, *infantry*,
*vehicle*, *aircraft*, *house*, *tick*, *weapon*, *warhead*, *projectile*, *hit*, *healing hit*,
*known attacker*, *health, strength, health ratio*, *experience*, *veterancy rank*, *locomotor*,
*locomotion class*, *movement zone*, *in play, on the map, off the map, removed*, *curtain protection,
temporarily invulnerable*, *bunkered, tank bunker*, *scenario random stream*, *quirk switch*. In
addition:

- **Mobile unit.** An infantry unit, a vehicle or an aircraft (not a building).
- **Host.** As S-0315 (Terms): the victim of a parasite while the parasite is attached to it.
- **Latching weapon.** A weapon whose warhead has the *parasite* flag and whose launch puts the
  firer inside its own projectile (the *launch-inside* flag of the weapon). The firer leaves the
  map when it fires and travels in the projectile (claim 1). Both flags come from the imported data
  (Import mapping). Weapons without the launch-inside flag and building-targeted latching detonations
  are not covered here (Open questions).
- **Drone parasite, grappling parasite.** A parasite whose type is both naval (S-0315, Terms) and
  organic is a *grappling parasite*; every other parasite is a *drone parasite*. *Organic* is a
  yes/no field of a mobile type (Import mapping). A *naval parasite* (S-0315) is a parasite of a
  naval type; every grappling parasite is naval, and in stock data every naval parasite is a
  grappling one.
- **Hold, held.** Every mobile unit has a *hold timer*, a countdown in ticks. The unit is *held*
  while the timer runs. A parasite starts its host's hold timer (claims 7 and 10), and a parasite
  that comes out of a living host is itself held for some ticks (claim 13). This is not the electromagnetic-pulse
  disable that S-0214 calls *paralysed*. S-0214 (Terms, *Movable* and *Parasite*) and S-0207
  (claim 20a, "under a paralysing effect") already refer to the hold. What being held changes is
  claim 8.
- **Paralysis length *P*.** An integer field of a warhead, in ticks (Import mapping): how long each
  renewal of the hold lasts.
- **Latch block.** A per-unit tick number: until that tick, no shot of a latching weapon may be
  fired at the unit (claim 5).
- **Latching onto, failing to latch.** What a parasite does when its projectile detonates: it
  either becomes attached to the projectile's target (claim 3) or fails to latch (claim 4).
- **Leaving a living host.** As S-0315 (Terms, *Leaving the host*): any way a parasite stops being
  attached while the host still exists. Claim 12 lists the ways; claim 13 gives where it comes out.
- **Losing the host.** The host leaves the map or is removed while a parasite is attached (claim 15).
  This is distinct from leaving a living host.
- **Silent removal.** As S-0315 (claim 7): the parasite is removed from the game at once. It is not
  killed by damage, leaves no wreck, death animation or survivor, gives no experience to anyone, and
  plays no death sound.
- **Exit direction.** The facing a parasite takes when it leaves a living host: a quarter turn from
  the host's facing, clockwise when the host faces one of the 8-directions 0, 1 or 2 (map north,
  north-east, east; S-0204, claim 9) and counter-clockwise otherwise (Formulas F4).
- **Cull flag.** A yes/no field of a warhead (Import mapping) that lets a grappling parasite finish
  off a weakened host (claim 11).
- **Grappling cycle.** The repeating sequence of a grappling parasite's phases (claim 10 and the
  state table).
- **Health colour.** As S-0111 (Terms, claim 5): green, yellow or red, from the health ratio and the
  two game-wide condition ratios, compared in binary64.

## Behaviour

### Latching on

1. **Firing a latching weapon.** When a unit fires a latching weapon, it leaves the map at once and
   travels inside the projectile that the shot creates. The projectile flies as any projectile of
   its kind does (S-0306). Its detonation delivers **no damage**: no hit of the detonation's area
   (S-0303) is made, whether or not the parasite latches. The detonation's later visual effects are
   not changed by this spec. When the shot is fired at a mobile unit, that unit's latch
   block is set to the current tick plus 20 (claim 5). When the projectile detonates, the parasite
   tries to latch onto the projectile's target object: the object the shot was fired at, wherever
   the detonation happens. No distance between the detonation point and the target is tested. When
   the target is a cell or no longer exists, the parasite fails to latch (claim 4).
   - Backing: `measurable` (scenario P1 for latching onto a moving target, P3 for the failed latch)
   - Confidence: high (leaving the map, the 20-tick block, the latch on the target object, no area
     damage), medium (that no distance test applies)

2. **Which units a parasite can latch onto.** A target is *latchable* when all of these hold:
   - it is a mobile unit (infantry, vehicle or aircraft) that is on the map and not destroyed, with
     health different from 0,
   - no parasite is attached to it: a host carries at most one parasite,
   - its type has the *can-carry-a-parasite* flag (Import mapping; absent-key default yes),
   - it is not bunkered in a tank bunker,
   - for a naval parasite only: when the target has a cell, it is a water tile (one of the 14 tiles
     starting at the theater's water-set role, S-0010; the same test as S-0306's level projectile).
     No cell does not itself reject the target.

   For mobile targets the same test decides the firing legality of claim 5 and the outcome of claim 3. Nothing else
   about the target matters here: in particular its owner, its armour and the parasite's own weapon
   range are tested only by the general firing rules (S-0301).
   - Backing: `measurable` (scenario P2 and P3)
   - Confidence: high

3. **Latching onto a target.** When the projectile detonates and its target is latchable, the
   parasite becomes attached to it: the target becomes its host. The parasite stays off the map,
   travels with the host and is not hit by anything that hits the host. Its first delivery
   (claims 6 and 10) happens at the host's next turn (S-0203): in the same tick when the host's
   turn in the update order comes after the projectile's, otherwise in the next tick. The parasite
   keeps no target and does not retarget while attached.
   - Backing: `measurable` (scenario P1)
   - Confidence: high; medium (the first-delivery tick)

4. **Failing to latch.** When the target is not latchable at detonation (a mobile target has died, been entered
   by another parasite, entered a tank bunker or left the map, or the shot was at a cell), the parasite does not latch. It comes back on the map at the centre of the cell it
   last stood in before firing, facing map north (facing 0, S-0204), under the ordinary placement
   rules for that cell. When it cannot be placed there, it is removed silently. Once placed, its
   target and destination are cleared unless it has attack-move memory (S-0213, Terms), and it falls
   back to rest (S-0215, claim 13). It is not held.
   - Backing: `measurable` (scenario P3)
   - Confidence: high (the cell, the facing, the clearing); medium (the silent removal when the
     cell is blocked)

5. **When a latching weapon may fire.** Against a target that is a mobile unit, a latching weapon's
   shot is *illegal* (S-0301, claim 9) when any of these holds:
   - the target is not latchable (claim 2),
   - the current tick is before the target's latch block: another latching shot was fired at it
     within the last 20 ticks (claim 1), by any firer,
   - the difference between the firer's height and the target's height is more than 2 levels
     (208 leptons), in either direction.

   Against any other target (a building, a cell) the parasite firing tests do not apply.
   A cell-targeted shot fails to latch (claim 4). Building-targeted detonation is untraced
   and is not specified here. The latch block stops a second parasite from being sent
   at a target while the first is still in flight. It also stops another firer's shot in the same tick when that firer's firing check follows
   the first shot.
   - Backing: `measurable` (scenario P2)
   - Confidence: high

### While attached

6. **A drone parasite's deliveries.** A drone parasite makes a *delivery* at its first delivery
   tick *t*₀ (claim 3) and then every max(1, *R*) ticks, *R* being the rate-of-fire field of its primary
   weapon (the elite one when the parasite is elite, S-0311 claim 6), as long as it stays attached
   (Formulas F1). Each delivery:
   - restarts the host's hold timer at *P* ticks, *P* being the paralysis length of the weapon's
     warhead (claim 8),
   - **on an infantry host**, kills it: a hit whose amount equals the host's current health, with
     the weapon's warhead, ignoring defences, with the no-escape marker (S-0302, Terms and claim 1),
     and with the parasite as known attacker and no attacking house. It makes no rocking
     choice; random choices of the host's death effects belong to their owning specs,
   - **on any other host**, shows a spark effect and the warhead animation for the host's 8-direction
     (claim 9), makes one draw from the scenario random stream (claim 9), and then delivers a hit of
     the weapon's damage with its warhead, at distance 0 (no distance reduction), not ignoring
     defences, with the no-escape marker, and with the parasite as known attacker and no attacking
     house. The hit is resolved by S-0302 like any other: the host's armour, the versus value and applicable hit modifiers apply.
     The delivered amount is imported weapon damage, not a shot scaled by firing bonuses.

   Hits on a host by its own parasite never suppress the parasite (S-0315, claim 6).
   - Backing: `measurable` (scenario P1)
   - Confidence: high (the cadence, the hit's amount and warhead, the instant kill of infantry),
     medium (the hit's markers and attacking house, which show only in rare cases)

7. **The hold that a parasite starts.** A drone parasite restarts its host's hold timer at *P*
   ticks at every delivery, so, for nonnegative *P* and *R*, the hold after each renewal lasts
   min(*P*, max(1, *R*)) ticks of each delivery interval, continuously when
   *P* ≥ max(1, *R*). Earlier actions in the renewal tick are not undone. A grappling parasite restarts it at *P*
   ticks in every one of its host's turns, so the host is held throughout the grapple when *P* ≥ 1.
   When the parasite leaves or is lost (claims 12 to 15), the host's hold ends at once.
   - Backing: `measurable` (scenarios P1 and P4)
   - Confidence: high

8. **What being held changes.** A held unit:
   - does not move under the driving, hovering, walking-vehicle, ship, flying, teleporting and
     tunnelling locomotors (S-0207, table T3). The walking, jumpjet and rocket locomotors ignore
     the hold,
   - is not *Movable* for orders (S-0214, Terms),
   - cannot stay cloaked: a held unit is uncloaked,
   - cannot fire, when its type is organic: its shots are *illegal* (S-0301, claim 9).

   The drop-pod locomotor has not been checked.
   - Backing: `measurable` (scenarios P1 and P9)
   - Confidence: high (the locomotor list, the order refusal); medium (the cloak rule, the organic
     firing rule)

9. **Rocking and effects of a drone parasite.** Each delivery to a host that is not infantry rocks
   the host's body with a strength of 1.5. F2 gives intended sideways presentation, not an
   established exact reconstruction of the original rocking point. The side is drawn: one draw from the
   scenario random stream with range 0 … 1, 1 meaning one side and 0 the other (Formulas F2). The
   draw happens after the effects are created and before the hit. The visible effects are a
   game-wide spark effect at the host's position and one animation from the weapon's animation list:
   the entry indexed directly by the host's 8-direction (S-0204). An empty list makes no animation.
   No modulo reduction is performed. Nonempty lists shorter than eight entries have unestablished
   results (Open questions). The rocking is
   presentation, computed in floating point by the original (Formulas F2). The draw is not
   presentation: it advances the scenario random stream, so it must be made exactly once per
   delivery to a non-infantry host, in this order.
   - Backing: `measurable` (P8 for ranged choices; P1 for rocking and effects)
   - Confidence: high (one draw per delivery and its place); low (whether creating the spark effect
     or the animation also draws from the stream, Open questions)

10. **A grappling parasite's cycle.** A grappling parasite does not use its weapon's rate of fire.
    In each of its host's turns it first restarts the host's hold timer (claim 7) and checks the
    host's cell. When that cell is not a water tile (claim 2), the parasite leaves the host at once
    (claim 13), and it is **not** held afterwards: the 3 × *R* hold of claim 13 is cancelled at once. Otherwise it advances its grappling cycle (State
    and ordering, table G). Its delivery weapon is its primary weapon, using the elite slot when
    elite (S-0311 claim 6):
    - a **grab** phase of 30 ticks, starting when it latches,
    - then **push** phases of 40 ticks each, each followed by a **strike** in the next tick, which
      also starts the next push phase.

    The first strike comes 69 ticks after the first turn of the grab, and later strikes every
    40 ticks (Formulas F3). At each strike, the parasite either culls the host (claim 11) or delivers
    a hit of its weapon's damage with its warhead, at distance 0, not ignoring defences, with the
    no-escape marker, with the parasite as known attacker and no attacking house. Draws from the
    scenario random stream, in this order:
    - at the end of the grab: one draw with range 0 … 1 (which way the first push rocks),
    - at each strike, before the cull test: three splash effects, each with one draw with range
      0 … 1 and then one draw with range 0 … *n* − 1, *n* being the number of entries in the
      game-wide splash list,
    - after a strike's hit, when the host still has this parasite: one ranged draw with range
      0 … 1 (the next push's way),
    - at every push start, after choosing its way: three wakes, each preceded by a ranged draw
      from −180 … 180 leptons and then one from −64 … 64 leptons for its displacement.

    These are explicit ranged choices. Effect creation and playback can make further choices
    not established here. Raw draw counts follow S-0202 claim 4, including rejection and
    single-valued ranges.

    The rocking of the host during the push phases is presentation: its tilt angle is
    sin(*k* × (±π/10)) × π/4 at frame *k* of the push, in floating point, and 0 during the grab and
    at the strike. Wake animations are created at the start of each push.
    - Backing: `measurable` (P6 for timing and hits; P8 for ranged choices)
    - Confidence: high (no use of the rate of fire, the draw count per strike); medium (the exact
      tick counts 30, 69 and 40, which depend on how the phase counters are stepped, and additional effect-creation choices)

11. **Culling.** At a strike, when the warhead has the cull flag and the host's health colour
    (S-0111, claim 5) is red with health above 0, or yellow while the grappling parasite is elite
    (S-0311), the parasite *culls* the host instead of hitting it:
    - when the parasite's type is trainable (S-0311), the parasite gains experience by F5,
    - the parasite leaves the host (claim 13) and is **not** held afterwards: the 3 × *R* hold of
      claim 13 is cancelled at once,
    - the host starts to sink as S-0312 (claim 10) describes for a killed ship, whatever its type's
      weight. Its health is not changed and no hit is delivered.

    Because no hit is delivered, the cull credits no destruction to anyone under S-0302 (claim 14)
    and S-0311 (claims 7 to 10): there is no kill value, no rank multiplier and no score. The host uses the ordinary local unit-lost announcement rules.
    - Backing: `measurable` (scenario P6)
    - Confidence: high (the condition, the sinking without a hit); medium (no kill credit at all)

12. **Leaving a living host: the ways.** The following traced events make a parasite leave a living host. Each
    row gives what happens to a drone parasite (not naval) and to a naval parasite. "Exits" means
    claim 13 runs; there, a parasite that is not naval is removed silently when it is suppressed
    (S-0315, claims 6 and 7), and a naval one never is.

    | Way (owner) | Drone parasite (not naval) | Naval parasite |
    |---|---|---|
    | The host takes a sonic hit (S-0315, claim 7) | exits | exits |
    | The host takes a healing hit whose warhead is not sonic, or a repair step (S-0315, claim 7; S-0314, claim 3) | removed silently | exits |
    | The host, a transport, lets a passenger out (unloading) | removed silently: unloading suppresses the parasite for 50 ticks just before it exits | exits |
    | The host reaches a consuming building while being eaten | removed silently: consumption sets 50 ticks of suppression before release | exits before host removal |
    | The host is caught by a superweapon that teleports it | not at launch; it exits at the teleport jump (next row) | exits at launch, at the host's old position, before the teleport |
    | The host jumps under the teleporting locomotor, its own move or a superweapon's (S-0207, claim 20e) | exits, before the host leaves its old position | exits, before the host leaves its old position |
    | An effect replaces the host's locomotor (a weapon that seizes a vehicle) | stays attached | exits |
    | A grappling parasite's host stands on a cell that is not a water tile (claim 10) | not applicable | exits, not held afterwards |
    | A grappling parasite culls its host (claim 11) | not applicable | exits, not held afterwards |

    When the parasite leaves through a hit (the first two rows), it leaves before that hit changes
    the host's health (claim 12's ordering, which supplements S-0315 claim 7).
    - Backing: `measurable` (P4 for hit and repair rows, P5 for unloading and consumption, P7 for
      superweapon, teleport and replacement rows, P6 for cull and off-water rows)
    - Confidence: high (sonic, healing, repair, unloading, the teleport jump, the locomotor
      replacement); medium (the consuming-building row); low (that a
      superweapon's teleport releases a drone parasite through the jump, as the table says)

13. **Where a parasite comes out of a living host.** When a parasite leaves a living host (claim 12)
    and is not removed silently by suppression:
    - **The candidate cell.** For a parasite that is not naval, the host's own cell. For a naval
      parasite, the neighbour cell of the host's cell in the 8-direction of the exit direction
      (Terms, F4): beside the host, to its right or left.
    - **The clear test.** The candidate is kept when it is clear for the parasite: enterable by
      class (S-0211, claim 10) for the parasite's own locomotion class and movement zone, a bridge
      cell judged on its deck, and holding no infantry and no vehicle at all, of any house. The
      host itself occupies its own cell, so for a parasite that is not naval the candidate is never
      clear. The override of S-0209 (claim 5, rows VP2 and WP1), under which objects in the
      victim's cell do not block a parasite attached to that victim, belongs to entering a cell
      while moving. It does not apply to this test.
    - **The search.** Otherwise the nearby-cell search of S-0212 (claim 25) chooses the cell, as
      the *Parasite exit* row of S-0212 table T5 fixes: centred on the host's cell, with the
      **host's** locomotion class (*winged* counted as *track*) and movement zone, not the
      parasite's (`quirks.parasite_exit_search_host_mobility`).
    - **The host-cell test.** The parasite comes out only when the **host's** cell, not the chosen
      cell, passes all of these (`quirks.parasite_exit_tests_host_cell`):
      - the host is not in the air,
      - no building stands in the host's cell,
      - for a parasite that is not naval: the host's cell either has a bridge or has a land type
        other than water, rock or beach; and, when the parasite is a vehicle, the cell holds no
        terrain object such as a tree. A bridge bypasses only the land-type restriction.
    - **Placement.** The parasite is placed at the centre of the chosen cell, facing the exit
      direction, under the ordinary placement rules.

    When the search gives no cell, the host-cell test fails or the placement fails, the parasite is
    removed silently. Once out, the parasite's target and destination are cleared unless it has
    attack-move memory (S-0213, Terms), it falls back to rest (S-0215, claim 13), and it is held
    for 3 × *R* ticks, *R* being its primary weapon's rate of fire (claim 8, Formulas F1). In either case the host's hold ends and the host's body stops
    rocking.
    - Backing: `measurable` (scenarios P4 and P9)
    - Confidence: high (the candidate cells, the search on the host's mobility, the 3 × *R* hold),
      medium (the host-cell test, and that the host's own occupancy always sends a drone parasite to
      the search)

14. **After coming out.** A parasite that has come out (claims 4, 13 or 15) is an ordinary unit
    again. It may fire its latching weapon again as soon as the firing rules allow: after claim 13,
    only once its hold has run out when its type is organic (claim 8), and otherwise as soon as it
    has a target. Its experience is kept.
    - Backing: `measurable` (scenario P9)
    - Confidence: medium

15. **Losing the host.** When the host leaves the map or is removed while a parasite is attached
    (it is removed after destruction, it enters a transport or a building,
    or it is otherwise taken off the map), the parasite is released at once:
    - It is removed silently when it is suppressed (S-0315, claim 6), **whether or not it is naval**
      (`quirks.parasite_host_loss_suppresses_naval`), unless the parasite is itself temporarily
      invulnerable.
    - Otherwise it is placed at the **host's position** when the host's cell passes the host-cell
      test of claim 13. A vehicle parasite is placed at the centre of that cell, at the deck's
      height when the host stood on a bridge. When the test fails, it is placed in the neighbour cell in
      the host's own 8-direction (in front of the host), unless the neighbour has land type water, rock or beach, the parasite is not naval,
      and the **host's cell** has no bridge. This fallback tests the host's bridge,
      not the neighbour's (`quirks.parasite_host_loss_bridge_test`). A point within that cell is chosen
      as for any placement.
    - The placement skips the ordinary placement rules: the parasite may share a cell with any
      other unit.
    - It faces the host's facing. Its target and destination are cleared unless it has attack-move
      memory, and it falls back to rest. Unlike claim 13, it is **not** held afterwards.
    - When no position is found, it is removed silently.

    This answers S-0315's open question on a host destroyed while it carries a parasite.
    - Backing: `measurable` (scenario P5)
    - Confidence: high (release on destruction, the suppression rule, no hold); medium (the
      in-front cell, the skipped placement rules, the invulnerability exception)

### Experience

16. **Experience of a parasite.** A host destroyed by a parasite's hit (claims 6 and 10) is credited
    to the parasite as attacker by the ordinary rules (S-0302, claim 14; S-0311, claims 7 to 10),
    kill value, rank multiplier and alliance test included. A cull (claim 11) instead gives a
    trainable grappling parasite the experience of F5: the gain is the host type's actual cost for
    the host's house, with no rank multiplier and no alliance test, divided by the parasite type's
    actual cost **for its own house** times the veteran ratio, and limited by the veteran cap
    (`quirks.cull_experience_path`). The gain is given even when the host's type is unscored or the host is
    allied with the parasite. This answers the open question of S-0311 on the grappling parasite's
    own experience path.
    - Backing: `measurable` (scenario P6)
    - Confidence: high (the formula's parts); medium (that the unscored test is skipped)

17. **Retaliation and targeting.** A host never retaliates against its own parasite (S-0309,
    claim 18). Objects that fire at the host are not redirected to the parasite: the parasite is off
    the map and cannot be targeted while attached.
    - Backing: `measurable` (scenario P1)
    - Confidence: high

### Order within a tick

18. **The parasite acts in its host's turn.** An attached parasite has no turn of its own: it acts
    during its host's turn, at the host's place in the update order (S-0203, claim 3).
    Its exact ordering relative to the host's movement is not established here. Two parasites therefore act in the order of their hosts, not
    in their own order.
    - Backing: `unmeasured` (the place within the host's turn shows only when two deliveries in one
      tick interact, which no practical scenario isolates)
    - Confidence: medium (the place within the host's turn)

19. **Re-selection after release.** A locally owned parasite with the imported re-select flag
    that was selected when it fired is selected again after a successful release by claim 13 or 15.
    The original remembers selection only for an infantry-targeted shot, a display-only selection
    glitch; our engine remembers it for every target, without a quirk switch.
    - Backing: `measurable` (P9)
    - Confidence: high

## Formulas

**F1, drone delivery ticks and the hold.** *t*₀ is the first delivery tick (claim 3), *R* ≥ 0 the
weapon's rate of fire, *P* the warhead's paralysis length (integer ticks):

> deliveries at *t*ₖ = *t*₀ + *k* × max(1, *R*), *k* = 0, 1, 2, …, while attached
>
> the host is held during the ticks *t*ₖ ≤ *t* < *t*ₖ + *P*
>
> after coming out of a living host (claim 13): the parasite is held during *t*ₓ ≤ *t* < *t*ₓ + 3 × *R*

Integer arithmetic in ticks, without fractional rounding. Durations use signed 32-bit
arithmetic and multiplication wraps as S-0205 claim 1. Negative rate-of-fire or paralysis
lengths are outside the established claims. With *R* = 0 a delivery happens in every host turn.
The hold intervals describe the state after renewal, not actions earlier in that tick.

**F2, rocking point (presentation).** θ is the host's facing as an angle, *s* = +2 when the draw
gives 0 and −2 when it gives 1:

> rock point = host position + *s* leptons along the direction perpendicular to θ, in the map plane
>
> strength = 1.5

This is an intended presentation formula. The original computes the rocking point using
floating-point trigonometry and truncation toward zero to whole leptons; its exact angle is
not established here. Presentation may use the formula above without changing the ranged choice.

**F3, grappling cycle timing.** *g* is the host's first turn after latching (the grab's first tick):

> first strike at *g* + 69; strike *j* (j = 1, 2, …) at *g* + 69 + 40 × (*j* − 1)

The grab lasts 10 phase frames of 3 ticks; a push lasts 10 phase frames of 4 ticks; the first frame of each
phase is counted in the tick the phase starts. Integer ticks.

**F4, exit direction.** *f* is the host's facing in 16-bit facing units (S-0204), *q* = 16383 (one
quarter turn, less one unit), *o*(*f*) its 8-direction (S-0204):

> exit direction = *f* + *q* (wrapped) when *o*(*f*) is 0, 1 or 2, otherwise *f* − *q* (wrapped)

The naval candidate cell is the neighbour in 8-direction *o*(exit direction). The facing given to
the placed parasite is the exit direction converted to 8-bit facing units by S-0204's rounding.

**F5, cull experience.** *e* is the parasite's experience (single precision), *A*ₕ is S-0311 F1 for
the host's type and the host's house, *A*ₚ is S-0311 F1 for the parasite's type and **the parasite's**
house, ρ the veteran ratio and *c* the veteran cap:

> *e*′ = S-0311 F3 with *V* = *A*ₕ and *A*ᵣ = *A*ₚ

Same precision and rounding as S-0311 F3.

**F6, cull test.** *h* is the host's health ratio in binary64 as S-0111 (Terms):

> cull when the warhead has the cull flag and (*h* ≤ red ratio and health > 0, or red ratio < *h* ≤
> yellow ratio and the parasite is elite)

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Latch block length | 20 | ticks | Claims 1 and 5 |
| Height limit for a latching shot | 2 levels (208) | leptons | Claim 5 |
| Hold after coming out of a living host | 3 × the weapon's rate of fire | ticks | Claim 13 |
| Suppression set by unloading or consumption | 50 | ticks | Claim 12 |
| Drone rock displacement | 2 | leptons | Claim 9, presentation |
| Drone rock strength | 1.5 | (rocking units) | Claim 9, presentation |
| Grab phase | 10 frames × 3 ticks | ticks | Claim 10 |
| Push phase | 10 frames × 4 ticks | ticks | Claim 10 |
| First strike after the grab starts | 69 | ticks | Claim 10, F3 |
| Strike period | 40 | ticks | Claim 10, F3 |
| Splash effects per strike | 3 | effects | Claim 10 |
| Push tilt | sin(*k* × π/10) × π/4 | radians | Claim 10, presentation |
| Exit quarter turn | 16383 | 16-bit facing units | F4 |
| Failed-latch facing | 0 (map north) | facing | Claim 4 |

## State and ordering

**Parasite states.**

| State | Event | Result |
|---|---|---|
| on the map, armed with a latching weapon | fires at a target (claim 5 allows it) | off the map, inside the projectile; mobile target's latch block set to now + 20 |
| inside the projectile | the projectile detonates; its target is latchable | attached to the target (claim 3) |
| inside the projectile | the projectile detonates; its target is not latchable, or is a cell | placed back at its last cell, facing north, not held (claim 4); removed silently when that fails |
| attached (drone) | the host's turn, delivery due | delivery (claim 6); host held for *P* |
| attached (grappling) | the host's turn | host held for *P*; the cycle advances (table G), or leaves when the host's cell is not water |
| attached | a way of claim 12 | removed silently, or exits by claim 13 and is held for 3 × *R* (not held after a cull or leaving because the host's cell is not water) |
| attached | the host leaves the map or is removed | removed silently when suppressed (naval too), unless temporarily invulnerable, else placed by claim 15 without a hold |

**Table G, grappling cycle.** One row per phase. Phase frames (cycle counters, not drawn frames)
advance one per 3 ticks in the grab and
one per 4 ticks in a push. The phase ends when its 10th frame is reached.

| Phase | Each turn | When the phase ends | Draws at the end |
|---|---|---|---|
| grab (starts at latching) | host held; host tilt 0 | a push phase starts in the same tick | 1 (range 0 … 1: push way), then six wake-displacement choices (claim 10) |
| push (either way) | host held; host tilt per Constants | the strike happens in the next tick | none |
| strike (one tick) | host tilt 0; three splash effects; cull test (F6) | cull: the parasite leaves, not held afterwards (claim 11); otherwise the hit, then, when the host still has the parasite, a push phase starts in the same tick | 3 × (range 0 … 1, range 0 … *n* − 1) before the test; 1 (range 0 … 1) and six wake-displacement choices after a hit that leaves the attachment intact |

**Ordering consequences.**
- A parasite acts in its host's turn (claim 18). When two drone parasites' deliveries fall in the
  same tick, the host that comes earlier in the update order is damaged first.
- A sonic or healing hit releases a parasite before the hit's own health change (claim 12, supplementing S-0315 claim 7), so a sonic
  hit that also kills the host frees the parasite by claim 13, not claim 15, when it was not
  suppressed.
- In a cull, the parasite comes out (claim 13) before the host starts to sink.
- The latch block is set when the shot is fired, so a second latching shot at the same target is
  refused from the next firing check onward.

## Edge cases and quirks

- **`quirks.parasite_exit_search_host_mobility`** (default on): the search for an exit cell uses
  the host's locomotion class and movement zone, not the parasite's. A drone parasite leaving a
  host on a shore can therefore be offered a cell that it could not drive into; the ordinary
  placement then fails and it is removed silently. *Fixed:* search with the parasite's own class and
  zone.
- **`quirks.parasite_exit_tests_host_cell`** (default on): whether a parasite may come out at all is
  judged on the host's cell, not on the cell chosen. A drone parasite on an amphibious host standing
  on water or on a beach is always removed silently when it leaves, even when the search found dry
  land nearby. *Fixed:* apply the test to the chosen cell.
- **`quirks.parasite_host_loss_suppresses_naval`** (default on): when the host is lost (claim 15), a
  suppressed naval parasite is removed silently, although suppression never affects a naval
  parasite leaving a living host (S-0315, claim 7). *Fixed:* a naval parasite ignores suppression
  here too.
- **`quirks.cull_experience_path`** (default on): a cull's experience has no rank multiplier, no
  alliance test and no unscored test, and divides by the parasite's cost for its own house, unlike
  every other kill (S-0311). *Fixed:* treat the cull as a destruction credited to the parasite,
  with S-0311's claims 7 to 10.
- **Unloading and consumption remove a non-naval parasite** (no switch; defined behaviour):
  the 50-tick suppression set just before the exit makes the silent removal certain.
- **`quirks.parasite_host_loss_bridge_test`** (default on): the fallback neighbour's restricted
  land passes when the host's cell has a bridge, even if that neighbour has none. *Fixed:* test
  the neighbour's bridge instead.
- **Re-selection on exit** is presentation; claim 19 fixes the original's display-only glitch
  without a quirk switch.
- **A drone parasite on an infantry host** kills it at the first delivery whatever its health,
  since the hit ignores defences. The can-carry-a-parasite default for infantry types is yes.

## Import mapping

| Source | Key | Parse form | Feeds | Absent-key default |
|---|---|---|---|---|
| rules, warhead section | `Parasite` | boolean | the parasite flag (latching weapon, Terms) | no |
| rules, warhead section | `Paralyzes` | integer | paralysis length *P*, ticks | 0 |
| rules, warhead section | `Culling` | boolean | cull flag (claim 11) | no |
| rules, warhead section | `Sonic` | boolean | sonic warhead (S-0315) | as S-0315 |
| rules, weapon section | `LimboLaunch` | boolean | launch-inside flag (Terms) | no |
| rules, weapon section | `ROF` | integer | rate of fire *R* (claims 6 and 13) | as S-0301 |
| rules, weapon section | `Damage`, `Warhead` | integer, section name | the delivery hit (claims 6 and 10) | as S-0301 |
| rules, weapon section | `Anim` | list of animation names | the per-direction delivery animation (claim 9) | empty |
| rules, infantry, vehicle or aircraft section | `Parasiteable` | boolean | the can-carry-a-parasite flag, latchable (claim 2) | yes for all three type kinds |
| rules, infantry, vehicle or aircraft section | `Organic` | boolean | organic (Terms; claim 8) | yes for infantry types, no for vehicle and aircraft types |
| rules, vehicle section | `Naval` | boolean | naval type (S-0315) | no |
| rules, any mobile section | `Trainable` | boolean | trainable (claim 16) | yes |
| rules, any mobile section | `SuppressionThreshold` | integer | S-0315 | 0 |
| rules, any mobile section | `ReselectIfLimboed` | boolean | the re-select flag: re-selection on exit (presentation) | no |
| rules `[AudioVisual]` | `ConditionRed`, `ConditionYellow` | ratio (S-0003) | red and yellow ratios (F6) | as S-0111 |
| rules `[General]` | `VeteranRatio`, `VeteranCap` | as S-0311 | F5 | as S-0311 |
| rules `[CombatDamage]` | `SplashList` | list of animation names | the splash effects and the size *n* of claim 10 | as S-0303 |
| rules `[General]` | `Wake` | animation name | the wakes of claim 10 | as S-0312 |

The grab animation of claim 10 is not read from a key: the original selects the animation type
identified by a fixed engine constant (presentation). When no such animation type exists, no
grab animation is shown and the cycle runs as described.

The team keys that ask a parasite to rejoin its team after coming out concern the original's
computer player and are out of scope.

| Source | Key | Parse form | Feeds | Absent-key default |
|---|---|---|---|---|
| rules `[CombatDamage]` | `DefaultSparkSystem` | particle-system type name | spark effect of claim 9 | not established |

## Measurement scenarios

Use a small purpose-made skirmish map, game-speed index 2, 800×600, director and mods off,
with one bystander computer player kept outside the observed area and the glossary's void rule.
Run each deterministic part three times. Use stock data except the named map override variants
below, removed after the test. Scripted input, video and a read-only state recorder are permitted.
A bench or arena adaptation should record ticks, health, positions, targets, experience and release
state; these are requested observables, not a claim that the existing harness records them already.
For every part, a result different from its stated expectation falsifies that part.

- **P1, latch, deliveries and targeting (claims 1, 3, 6, 7, 9, 17).** Place a terror drone of A,
  a moving tank of enemy B, and a second B tank within weapon reach. Order the drone to attack,
  then the second tank to attack its host. Record launch, attachment and health-change ticks,
  effects, movement and attack targets. Expected: disappearance at launch, attachment to the
  original target even when it has moved away from the detonation, first delivery on the next
  host turn, then F1; no detonation area hit on a nearby bystander. The host can attack the
  second tank but never its own parasite, and the second tank still targets the host.
  Named map variants: make the host infantry and latchable with a nonzero versus value,
  or make the host temporarily invulnerable. Expected: the infantry delivery ignores defences
  and kills at first delivery, whereas ordinary non-infantry damage follows S-0302. Give the
  drone's primary weapon distinct rookie and elite damage and cadence; expected the rank's
  primary slot, without a firing-bonus multiplication. Set paralysis length shorter than,
  equal to and longer than rate of fire, plus zero rate of fire. Record hold state and move
  attempts between deliveries: expected F1 and cessation of hold on release. Do not infer the
  hold's starting tick from movement that occurred earlier in that same tick.
- **P2, legality (claims 2, 5).** Order two A drones at one B tank in one tick. Record every shot.
  Expected: once the first fires, the second check refuses for 20 ticks, and refuses while an
  attachment exists. Named map variant: slow the latching projectile enough that it remains in
  flight for longer than 20 ticks, and give the second drone an always-ready weapon; expected
  that the latch-block refusal ends at exactly 20 ticks, subject to other firing rules.
  Repeat with a bunkered target, an already infected target, the carry flag off, and height
  differences exactly 208 and greater than 208 leptons in both directions. Expected: claim 5.
  Named naval variant: a latchable amphibious host on water and on dry terrain, with a naval
  parasite; expected only the water target to pass the naval cell test.
- **P3, failed latch (claims 1, 4).** Fire at a tank beside its bunker, then have it enter the
  bunker before impact. Repeat with a destroyed target and a forced cell-targeted shot.
  Record area health, release position, facing, target and destination. Expected: no area hit,
  return to the original firing cell's centre facing north, idle and not held. Occupy that cell
  during flight: expected silent removal if ordinary placement fails. Repeat with attack-move
  memory; expected only the stated clearing exception, not a new target during attachment.
- **P4, living-host exit (claims 7, 12, 13).** Put an unsuppressed drone in a tank, then apply
  a sonic hit, healing hit or paid repair step. Record exit cell, facing, hold and host rocking.
  Expected: sonic release per claim 13 and F4; healing and repair silently remove a non-naval
  parasite; the host's hold and rocking end. Repeat sonic release for all eight host directions,
  with blocked candidate cells and with no qualifying nearby cell: expected host-mobility
  search, or silent removal on failure. Named map variants: a latchable amphibious host on a
  beach or water, a host on a bridge over water, and a host with a terrain object or building
  in its cell. Expected: exactly the host-cell test of claim 13, including the bridge exception
  only to land type. Compare a host and parasite with different movement zones to discriminate
  host-based search from parasite-based search. Make the sonic hit lethal: expected release
  before the host's health changes, using claim 13 rather than claim 15.
- **P5, host loss and consumption (claims 12, 15, 16).** Destroy an infected tank by its own
  parasite, recording both destruction and actual map-removal ticks. Expected: ordinary kill
  experience under S-0311, and claim 15 release when the host leaves the map or is removed,
  facing as the host without a new hold. Have a second unit at the release point: expected
  forced overlapping placement. Repeat after suppressing the parasite with an external hit:
  expected silent removal. Named variants: suppress a naval parasite and remove its host,
  then repeat with the parasite temporarily invulnerable; expected removal only without that
  protection. Arrange a rejected host cell and test the front neighbour: expected claim 15,
  including its host-bridge test. Unload a passenger from an infected transport, and send an
  infected vehicle into a consuming building: expected 50-tick suppression before living-host
  exit, hence non-naval removal. Enter an ordinary transport with an infected unit: expected
  host-loss release, not the passenger-unloading path. Test a ship killed by damage separately
  and record whether release waits until removal; this tests the sinking open question.
- **P6, grapple and cull (claims 7, 10, 11, 16).** Order a squid to attack an enemy destroyer
  on open water. Record health, phase changes, rocking, wake and splash counts and experience.
  Expected: first strike at F3's exact tick, later strikes 40 ticks apart, uninterrupted hold,
  three wakes at each push start, three splashes at each strike. A one-tick discrepancy
  falsifies the proposed timing, rather than being accepted as tolerance. At red health the
  next eligible strike culls, releases without a hold and starts sinking without health loss.
  Named variants: different weapon rate of fire (no cadence change), cull flag off (ordinary
  hit), elite parasite with yellow host (cull), untrainable parasite (no cull experience),
  differing house-cost factors, allied and unscored hosts. Expected: F5 without victim-rank,
  alliance or unscored filtering, and no ordinary kill credit or score from culling.
  Give an amphibious grappling host a route onto dry terrain: expected immediate release,
  without a post-exit hold. Named long-lived hosts allow the phase test to finish before culling.
- **P7, teleport and replacement (claim 12).** Infect a chrono-moving unit with a drone, then
  order a jump; record the old and new positions and release tick. Expected: exit at the old
  position before departure. Repeat a chronosphere relocation of an infected tank: hypothesis
  is release at the warp jump, not launch. Repeat with a squid-carrying ship: expected release
  at launch at its old position. Apply a locomotor-replacing weapon to a latchable host carrying
  a naval parasite and then a non-naval one (named map variants if needed); expected release
  only for the naval parasite.
- **P8, random choices (claims 9, 10).** Use P1 and P6 setups and the random-state recorder for
  twenty deliveries and twenty surviving strikes. Predict ranged choices with S-0202 claim 4.
  Named map overrides disable optional particles and animation randomness to isolate these
  choices, then restore each effect separately. Expected: a non-infantry drone delivery's one
  0 … 1 choice before its hit; no such choice for an infantry delivery. For grappling, expected
  the push-direction choice, six wake-displacement choices in the listed order, six splash
  choices before each cull test, and a new direction plus wake choices only after a surviving
  hit. Account separately for raw-value rejection and effect-owned randomness; an extra
  unaccounted ranged choice, a different range or a changed order falsifies the explicit list.
- **P9, hold, reuse and selection (claims 8, 14, 19).** Give an infected vehicle a sonic release,
  then immediately issue move and attack orders. Record hold, movement, shots, experience and
  selection. Expected: hold for 3 × primary rate of fire; non-organic firing is not blocked by
  hold alone, organic firing is blocked; retained experience after release. Named map variants
  give latchable hosts each locomotor listed in claim 8, organic and cloak flags as appropriate,
  and a harmless delivery weapon so hosts survive. Compare autonomous movement already ordered
  before attachment, fresh movement orders, organic firing, and cloak state while held:
  expected claim 8, excluding the unchecked drop-pod case. Give a locally owned parasite the
  re-select flag, select it before launch and observe release. Expected in the original:
  selection remembered only for infantry targets; our corrected presentation remembers every
  target. The correction itself is a design rule, not an oracle expectation.

## Open questions

- Whether creating the spark effect, the delivery animation, the wakes or the splash animations draws
  from the scenario random stream besides the draws listed in claims 9 and 10.
- A weapon with the parasite flag on its warhead but without the launch-inside flag.
- Building-targeted latching detonations: no parasite-specific firing guard applies, but their
  attachment result is not established.
- Nonempty delivery-animation lists shorter than eight entries: out-of-list results are not
  established; no wrap or safe fallback is claimed.
- Whether a superweapon's teleport releases a drone parasite through the teleport jump (claim 12,
  low): scenario P7.
- A drone parasite whose ship host is killed and sinks: whether the parasite stays inside during
  the sinking until the host is removed (claim 15 then applies).
- Whether the host-cell test of claim 13 can fail in skirmish play for reasons other than an
  amphibious host on water or a beach (a host in the cell of a building's exit, for example).
- How a visceroid-type host interacts with a parasite (the original reads the attachment there; not
  traced).
- The absent-key default of the spark effect, and exact drone rocking geometry beyond the
  intended sideways displacement of F2.
