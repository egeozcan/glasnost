---
id: S-0614
title: Parasites
topic: combat
revision: 32
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-05
measurements: []
requests: [Q-0620, Q-0621, Q-0622, Q-0623, Q-1032, Q-2671]
supersedes: []
---

# S-0614: Parasites

## Summary

Skirmish parasite attachment, drone and grappling deliveries, hold effects, release and experience.
S-0315 claims 6 and 7 own suppression by damage and healing; network games and campaigns are out of
scope. Claims 15, 20 and 23 answer Q-0620. Claims 21 and 22 answer Q-0621 and Q-0623, and claim 18
answers Q-0622. S-0209 claims 4 and 5 already supply entry outcomes, and S-0208 claim 3 already
supplies standing-position choice; this spec establishes their placement uses. Claim 24 answers
Q-1032: a higher selected special hit strands the carried parasite rather than returning it after a
failed latch. Claim 25 answers Q-2671: the grab animation of a grappling parasite. S-0514 claim 20
already owns effect precedence.

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
  (Import mapping). Weapons without the launch-inside flag and building-targeted detonations with
  attachment selected are not covered here (Open questions). Claim 24 covers suppressed attachment
  even for building targets.
- **Drone parasite, grappling parasite.** A parasite whose type is both naval (S-0315, Terms) and
  organic is a *grappling parasite*; every other parasite is a *drone parasite*. *Organic* is a
  yes/no field of a mobile type (Import mapping). A *naval parasite* (S-0315) is a parasite of a
  naval type; every grappling parasite is naval.
- **Hold, held.** Every mobile unit has a *hold timer*, a countdown in ticks. The unit is *held*
  while the timer runs. A parasite starts its host's hold timer (claims 7 and 10), and a parasite
  that comes out of a living host is itself held for some ticks (claim 13). This is not the electromagnetic-pulse
  disable that S-0214 calls *paralysed*. S-0214 (Terms, *Movable* and *Parasite*) and S-0207
  (claim 20a) already refer to the hold. What being held changes is
  claim 8.
- **Paralysis length *P*.** An integer field of a warhead, in ticks (Import mapping): how long each
  renewal of the hold lasts.
- **Latch block.** A per-unit tick number: until that tick, no shot of a latching weapon may be
  fired at the unit (claim 5).
- **Latching onto, failing to latch.** The two outcomes when parasite attachment is selected at
  detonation: attachment to the projectile's target (claim 3) or failed-latch return (claim 4).
  Suppression of attachment by a higher selected effect is neither outcome (claim 24).
- **Leaving a living host.** As S-0315 (Terms, *Leaving a living host*): any way a parasite stops being
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
- **Grab animation.** The animation that shows a grappling parasite holding its host (claim 25).
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
   These detonation attachment and failed-latch outcomes apply only when parasite attachment
   is the selected special hit under S-0514 claim 20. Takeover, bomb planting and electric
   charging each suppress attachment, even when the selected effect cannot succeed. Selected
   attachment suppresses temporal erasure even when latching fails. This selection does not
   change the separate firing-time departure from the map or the latch block. When a higher
   effect is selected, claim 24 owns the carried parasite's fate, not claim 4.
   - Backing: `measurable` (scenario P1 for latching onto a moving target, P3 for the failed latch,
     P14 for special-hit selection)
   - Confidence: high (leaving the map, the 20-tick block, the latch on the target object, no area
     damage, special-hit selection), medium (that no distance test applies)

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
   - Backing: `measurable` (scenario P2 and P3); `unmeasured` (the no-cell exception:
     no practical setup is known for an on-map mobile target without a cell)
   - Confidence: high

3. **Latching onto a target.** When the projectile detonates, parasite attachment is selected under
   S-0514 claim 20, and its target is latchable, the parasite becomes attached to it: the target
   becomes its host. The parasite stays off the map (while a grappling parasite holds its host, the
   parasite is not drawn, and its own picture in the hold is at most its grab animation, claim 25),
   travels with the host and is not hit by anything that hits the host. Its first delivery (claims 6
   and 10) happens at the host's next turn (S-0203): in the same tick when the host's turn in the
   update order comes after the projectile's, otherwise in the next tick. The parasite keeps no
   target and does not retarget while attached.
   - Backing: `measurable` (scenario P1)
   - Confidence: high; medium (the first-delivery tick)

4. **Failing to latch.** With parasite attachment selected under S-0514 claim 20, when the
   target is not latchable at detonation (a mobile target has died, been entered
   by another parasite, entered a tank bunker or left the map, or the shot was at a cell), the parasite does not latch. It comes back on the map at the centre of the cell it
   last stood in before firing, facing map north (facing 0, S-0204), under claim 20's placement
   rules. The centre is the requested point; infantry may take a side position. When it cannot be placed there, it is removed silently. Once placed, its
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
   - Backing: `measurable` (P1 for deliveries and the no-escape outcome; P5 for attacker credit),
     `unmeasured` (the absence of a separate attacking house: ordinary credit derives the same
     house from the known parasite attacker, so these setups cannot distinguish that input)
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
   - cannot fire a spawner weapon, regardless of whether its type is organic: the firing check
     answers *impossible* (S-0506 claim 10), subject to earlier refusals in S-0301 claim 9,
   - cannot fire other weapons when its type is organic: its shots are *illegal* (S-0301 claim 9).

   The drop-pod locomotor has not been checked.
   - Backing: `measurable` (scenarios P1 and P9; S-0506 scenario R6 for spawner firing)
   - Confidence: high (the locomotor list, the order refusal, the spawner firing rule); medium
     (the cloak rule, the organic firing rule)

9. **Rocking and effects of a drone parasite.** Each delivery to a host that is not infantry rocks
   the host's body with the strength of F2, which this spec owns, pushed from the point of S-0113 F3
   (S-0113 claim 13 gives the push). The side is drawn: one draw from the scenario random stream
   with range 0 … 1, 1 meaning one side and 0 the other (S-0113 F3). The draw happens after the
   effects are created and before the hit. The visible effects are a game-wide spark effect at the
   host's position (creating it takes no draw, S-0616 claim 1) and one animation from the weapon's
   animation list: the entry indexed directly by the host's 8-direction (S-0204). An empty list
   makes no animation. No modulo reduction is performed. Nonempty lists shorter than eight entries
   have unestablished results (Open questions). The rocking is computed in floating point, and
   S-0113 claims 9 and 19 give how it reaches game state, at their confidence: the rocking can
   destroy a host (S-0113 claim 9). The draw advances the scenario random stream, so it must be made
   exactly once per delivery to a non-infantry host, in this order.
   - Backing: `measurable` (P8 for ranged choices; P1 for rocking and effects; S-0113 scenarios
     RK3 and RK8 for how rocking reaches game state: the rolling-over hit and the fire point)
   - Confidence: high (one draw per delivery and its place); medium (that the rocking can destroy
     a host: S-0113 RK3 rolls a body over with area pushes, not drone deliveries); low (whether
     creating the spark effect or the animation also draws from the stream, Open questions)

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
    - in the grab's first turn: the creation draws of the grab animation, when it is created (claim
      25),
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

    During the push phases the parasite sets the host's roll directly, in each of the host's turns
    after the phase's first, at the push's phase frame *k*, and sets it to 0 in the grab's first
    turn, in the turn in which a push phase ends and at the strike: S-0113 claim 14 gives which turn
    writes which *k* (0 to 9, none from 10), the cadence and the sign of each push, from that push's
    own way draw, and S-0113 F4 the value and its rounding. Wake animations are created at the start
    of each push. The grab animation is claim 25's.
    - Backing: `measurable` (P6 for timing, hits and the no-escape outcome; P8 for ranged choices),
      `unmeasured` (the absence of a separate attacking house, for the same reason as claim 6)
    - Confidence: high (no use of the rate of fire, the draw count per strike); medium (the exact
      tick counts 30, 69 and 40, which depend on how the phase counters are stepped, and additional
      effect-creation choices); medium for the roll value and cadence, which follow S-0113 claim
      14's frame count *k*; high for the roll's sign

11. **Culling.** At a strike, when the warhead has the cull flag and the host's health colour
    (S-0111, claim 5) is red with health above 0, or yellow while the grappling parasite is elite
    (S-0311), the parasite *culls* the host instead of hitting it:
    - when the parasite's type is trainable (S-0311), the parasite gains experience by F5,
    - the parasite leaves the host (claim 13) and is **not** held afterwards: the 3 × *R* hold of
      claim 13 is cancelled at once,
    - the host starts to sink, whatever its type's weight. Its health is not changed and no hit
      is delivered. Once sinking is set, S-0504 claims 4 and 5 own the descent, wakes, random
      draws, restrictions while sinking and removal; this claim owns only the cull's trigger.

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
    | The host starts curtain protection (S-0513 claim 25) | removed silently: starting protection sets 50 ticks of suppression just before release | exits |
    | The host, a transport, lets a passenger out (unloading) | removed silently: unloading suppresses the parasite for 50 ticks just before it exits | exits |
    | The host reaches a consuming building while being eaten | removed silently: consumption sets 50 ticks of suppression before release | exits before host removal |
    | The host is caught by a superweapon that teleports it (S-0513 claims 34 and 39) | not released at launch; release at departure remains unresolved | exits at launch, at the host's old position, before the teleport |
    | The host makes its own jump under the teleporting locomotor (S-0207 claim 20e) | exits, before the host leaves its old position | exits, before the host leaves its old position |
    | An effect replaces the host's locomotor (a weapon that seizes a vehicle) | stays attached | exits |
    | A grappling parasite's host stands on a cell that is not a water tile (claim 10) | not applicable | exits, not held afterwards |
    | A grappling parasite culls its host (claim 11) | not applicable | exits, not held afterwards |

    Hit-triggered release excludes S-0315 claim 6's vehicle-factory exception: such a hit does
    not release the parasite. Otherwise, when the parasite leaves through a hit (the first two
    rows), it leaves before that hit changes the host's health (S-0315 claim 7).
    - Backing: `measurable` (P4 for hit, repair and curtain rows, P5 for unloading and consumption,
      P7 for superweapon launch, ordinary teleport and replacement rows, P6 for cull and off-water
      rows). No drone release outcome is promised for superweapon departure; P7 records it.
    - Confidence: high (sonic, healing, repair, curtain protection, unloading, the ordinary
      teleport jump, the locomotor replacement); medium (the consuming-building row),
      low (the unresolved drone outcome at superweapon departure)

13. **Where a parasite comes out of a living host.** When a parasite leaves a living host (claim 12)
    and is not removed silently by suppression:
    - **The candidate cell.** For a parasite that is not naval, the host's own cell. For a naval
      parasite, the neighbour cell of the host's cell in the 8-direction of the exit direction
      (Terms, F4): beside the host, to its right or left.
    - **The clear test.** The candidate is kept when it is clear for the parasite: enterable by
      class (S-0211, claim 10) for the parasite's own locomotion class and movement zone, a bridge
      cell judged on its deck, and holding no infantry and no vehicle at all, of any house. The
      host itself occupies its own cell, so for a parasite that is not naval the candidate is never
      clear. The override of S-0209 (claim 4 row WP1 and claim 5 row VP2), under which objects in the
      victim's cell do not block a parasite attached to that victim, belongs to entering a cell
      while moving. It does not apply to this test.
    - **The search.** Otherwise the nearby-cell search of S-0212 (claim 25) chooses the cell, as
      the *Parasite exit* row of S-0212 table T5 fixes: centred on the host's cell, with the
      **host's** locomotion class (*winged* counted as *track*) and movement zone, not the
      parasite's (`quirks.parasite_exit_search_host_mobility`).
    - **The host-cell test.** The parasite comes out only when the **host's** cell, not the chosen
      cell, passes all of these (`quirks.parasite_exit_tests_host_cell`):
      - the host is not in the air as defined by claim 22,
      - no building stands in the host's cell,
      - for a parasite that is not naval: the host's cell either has a bridge or has a land type
        other than water, rock or beach; and, when the parasite is a vehicle, the cell holds no
        terrain object such as a tree. A bridge bypasses only the land-type restriction.
    - **Placement.** The requested point is the centre of the chosen cell, facing the exit
      direction, under claim 20's placement rules. Infantry may take a side position.

    When the search gives no cell, the host-cell test fails or the placement fails, the parasite is
    removed silently. Once out, the parasite's target and destination are cleared unless it has
    attack-move memory (S-0213, Terms), it falls back to rest (S-0215, claim 13), and it is held for
    3 × *R* ticks, *R* being its primary weapon's rate of fire (claim 8, Formulas F1). In either
    case the host's hold ends. A grappling parasite's own rocking of the host stops: it sets the
    host's roll to 0 on leaving (S-0113 claim 14). A roll rate from a push is not changed by the
    grappling parasite, so it goes on stepping and shows once the parasite stops writing the roll.
    - Backing: `measurable` (scenarios P4 and P9; S-0113 RK6 part b for the roll at a release)
    - Confidence: high (the candidate cells, the search on the host's mobility, the 3 × *R* hold),
      medium (the host-cell test, that the host's own occupancy always sends a drone parasite to
      the search, and the roll set to 0 on leaving with the push's rate going on, until S-0113
      RK6 part b records it)

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
      not the neighbour's (`quirks.parasite_host_loss_bridge_test`). The fallback makes the
      occupant-aware centre-class position choice of S-0208 claim 3, reading the neighbour's
      ground record, even for a vehicle parasite. It consumes one ranged scenario choice over
      0 … 3 unless the vehicle-present or object-present refusal prevents it. An infantry
      parasite takes the resulting side position; no position means silent removal. A vehicle
      uses the resulting point's cell centre when the choice succeeds. Claim 23 specifies failed
      choice, including a vehicle's erroneous placement far from the host, and bridge height.
    - Final placement skips claim 20's entry-legality check, so the parasite may share a cell
      with another unit. This does not bypass the fallback's earlier position choice. When the
      host-cell test passes, an infantry parasite instead uses the host's position class
      (S-0208 claim 2) without regard to occupants or a position-choice draw. Forced placement
      makes no second occupant-aware choice after a successful fallback choice.
    - It faces the host's facing. Its target and destination are cleared unless it has attack-move
      memory, and it falls back to rest. Unlike claim 13, it is **not** held afterwards.
    - Rejection by the fallback's land test, or failure of final forced placement, removes it
      silently. Failed standing-position choice instead follows claim 23; it is not invariably
      removal for a vehicle.

    This answers S-0315's open question on a host destroyed while it carries a parasite.
    - Backing: `measurable` (P5; P10 for fallback position choice and draws),
      `unmeasured` for the object-present refusal: S-0208 claim 3 and S-0209 claim 1
      establish no practical fixture isolating that flag's effect
    - Confidence: high (release on destruction, suppression, no hold, position choice and draws),
      medium (the in-front cell, forced placement, the invulnerability exception)

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
    - Backing: `measurable` (P5 for ordinary kill experience; P6 for cull experience)
    - Confidence: high (the formula's parts); medium (that the unscored test is skipped)

17. **Retaliation and targeting.** A host never retaliates against its own parasite (S-0309,
    claim 18). Objects that fire at the host are not redirected to the parasite: the parasite is off
    the map and cannot be targeted while attached.
    - Backing: `measurable` (scenario P1)
    - Confidence: high

### Order within a tick

18. **The parasite acts late in its host's turn.** An attached parasite has no turn of its own:
    deliveries and grappling events occur at the host's place in the update order (S-0203 claim 3).
    Both kinds of self-healing (S-0314 claims 9 and 10), due behaviour processing (S-0215 claim 2)
    and the host's ordinary locomotion update precede the parasite event. Two parasites therefore
    act in host order, not their own order. This ordering applies to turns that reach the
    ordinary attached-parasite step, not to every warp update.

    For vehicles, teleporter infantry and landed aircraft undergoing a superweapon warp,
    S-0513 claim 51's engine-design contract suppresses the ordinary attached-parasite step
    from the first pending update through arrival effects, inclusive. No delivery or grappling
    advancement occurs through that step in those turns. During arrival hold, ordinary step
    eligibility returns after the hold-end test, whether the hold continues or expires, with
    no catch-up burst of skipped steps. Elapsed-tick countdowns still keep their clocks.
    Immediate event-driven releases are not suppressed: S-0513 claim 34's naval release at
    launch remains distinct from claim 12's unresolved drone release at departure. This is
    an engine-design rule, not a finding that the original suppresses every such step:
    original per-stage continuation remains unresolved under S-0513 claim 52.

    A hold first renewed by that event cannot prevent movement already taken in that turn.
    Movement in the next turn sees the timer then remaining, subject to claim 8's exceptions.
    A one-tick renewal can therefore expire before preventing any movement step.
    A grapple's off-water check sees the cell reached by that turn's movement. Healing already
    received can prevent a delivery from being lethal; no self-healing step remains after that
    delivery in the same turn. A hit mark made by the delivery cannot change behaviour processing
    that already happened.

    There is not one common firing step for every kind of host. The ordinary separate firing
    update of infantry and vehicles follows the parasite event, so a newly renewed hold already
    refuses an organic host's otherwise legal shot. Shots made by behaviour processing instead
    precede it, including aircraft attack-run shots (S-0501 claims 13 and 14); those shots are not undone.
    This claim does not reorder a shot or release caused immediately by another event.
    - Backing: `measurable` (P12; P6 for movement before the off-water check),
      `unmeasured` for a hit mark's effect on behaviour: the host cannot retaliate against
      its own parasite (claim 17), and no other isolating behavioural response is established.
      Warp exclusion and resumption: `measurable` in our engine (P12 warp part).
      original per-stage continuation remains `unmeasured` (S-0513 claim 52's incomplete
      per-kind continuation evidence, with observations proposed in its SW23 part g)
    - Confidence: high for ordinary ordering and the engine-design contract; low for matching
      the original's unresolved warp-stage continuation

19. **Re-selection after release.** A locally owned parasite with the imported re-select flag
    that was selected when it fired is selected again after a successful release by claim 13 or 15.
    The original remembers selection only for an infantry-targeted shot, a display-only selection
    glitch; our engine remembers it for every target, without a quirk switch.
    - Backing: `measurable` (P9)
    - Confidence: high

20. **Ordinary placement of a returning parasite.** Claims 4 and 13 attempt placement in
    exactly the specified cell. Refusal does not search another cell, wait for occupants, push
    them away or attack an obstruction: the parasite is removed silently. Claim 13's nearby-cell
    search occurs before this attempt, not as a retry after it.

    For infantry and vehicle parasites, acceptance requires the entry outcome to be *clear*:
    S-0209 claims 4 and 5 own overlay, occupant and occupancy outcomes, with the terrain rules
    of S-0211 claim 10. The placement asks without an approach direction or source elevation.
    Its entry test reads ground occupants and ground occupancy, not a deck record inferred from
    the parasite's former bridge state. Every non-clear outcome refuses immediately. Thus an
    ordinary stationary allied vehicle and an ordinary enemy vehicle both refuse a non-crushing
    vehicle parasite's return, despite different entry outcomes. A normal building, tree or
    intact wall also refuses such a vehicle. There is no extra blanket ban on all buildings or
    units: entry-table exceptions still apply. An enemy infantry unit crushable by the returning
    vehicle can give *clear*, unlike allied stationary infantry.

    A vehicle uses the requested centre. For infantry requested at terrain height inside the
    usable area, the occupant-aware position choice of S-0208 claim 3 is also required, with
    the requested centre as desired point and the ground occupancy record. One ranged scenario
    choice over 0 … 3 selects a row of that claim's table O; it does not try positions 0 through
    4 numerically. Only right, left and bottom can be selected. Vehicle and object-present
    refusals precede the draw; otherwise a failed choice still consumes it. No free side position
    means failure even if centre or top is free. A tree can prevent the choice through its
    position marks rather than its mere presence. Position choice precedes entry acceptance:
    its draw is retained even if entry then refuses. The cited tables already exist; this claim
    supplies their placement use, not new occupant precedence.
    - Backing: `measurable` (P10); `unmeasured` for the object-present refusal:
      no practical fixture isolating that flag's effect is established (S-0208 claim 3)
    - Confidence: high (single attempt, clear-only acceptance, position choice); medium
      (entry-table application to uncommon building and crushing exceptions)

21. **A carrying projectile removed without detonation.** When the projectile of claim 1 is
    removed without detonating, including leaving the legal map under S-0306 claim 19,
    its parasite does not attempt a latch or return to its firing cell. It remains off the map,
    unattached and unable to act; no recovery timer starts. Projectile removal does not itself
    kill the parasite, give kill experience, play its death effects or place a replacement unit.
    The same result applies to another removal that ends the carrying projectile without a
    detonation, provided no separate effect removes the parasite itself. A detonation that fails
    to latch remains claim 4, a different outcome.
    This is `quirks.parasite_carrier_loss_strands_unit` (default original); with it off, carrier
    removal without detonation instead attempts claim 4's failed-latch return.
    - Backing: `measurable` (P11 for map-border removal); `unmeasured` for other removal causes:
      no distinct normal-action setup is established here that removes such a projectile without
      detonation or separately removing its firer
    - Confidence: high

22. **The host-cell air test.** In claims 13 and 15, an ordinary infantry or vehicle host is
    in the air exactly when it is on the map and its integer height above the surface is at
    least 208 leptons. There is no fractional rounding. Height subtracts terrain height at its
    position, and also subtracts 416 leptons when marked as standing on a high bridge. A host
    standing on that deck therefore has height 0 above the surface and does not fail merely
    because its world height is high.

    This is S-0307's airborne test, not a locomotor's moving or flying state. A jumping, falling
    or lifted host passes or fails by current height and bridge mark: rising at 207 leptons
    does not reject it, and 208 does. Being carried is not another rejection: an off-map host
    gives no in-the-air answer, though its map loss has already triggered claim 15. For aircraft,
    use S-0501 claim 18's in-the-air answer, including its two exceptional missile types, as
    S-0307 claim 26 already specifies. Do not add S-0501's separate in-flight test. Passing the
    air test does not waive other host-cell restrictions or placement.
    - Backing: `measurable` (P13); `unmeasured` for an attached exceptional missile host:
      no normal-action fixture that latches onto one and releases it is established
    - Confidence: high

23. **Host-loss fallback failure and surface.** This supplements claim 15, not the ordinary
    return of claims 4 and 13. There is only one front-neighbour candidate; a refusal never
    searches farther or retries with another standing position outside S-0208 claim 3's table.
    Its outcomes are:

    | Front-neighbour outcome | Infantry parasite | Vehicle parasite | Explicit ranged choices for placement |
    |---|---|---|---|
    | Restricted land rejected by claim 15 | removed silently | removed silently | none |
    | Vehicle-present or object-present refusal under S-0208 claim 3 | removed silently | placed at world position (128, 128, 0) leptons | none |
    | No free side position, with neither earlier refusal | removed silently | placed at world position (128, 128, 0) leptons | one over 0 … 3, even though no position was accepted |
    | Side position accepted | that side position | centre of that position's cell | one over 0 … 3 |

    The erroneous vehicle position is the centre of world cell (0, 0), outside the playable
    map, not the centre of the map's local area or the front neighbour. The vehicle survives
    release there, unattached, with no new hold and with claim 15's facing and order clearing.
    It is not silently removed just because the standing-position choice failed. This is
    `quirks.parasite_host_loss_failed_choice_origin` (default original); off, failed choice
    removes a vehicle silently just as it does infantry. The choice and its draw are unchanged.

    For an accepted point, let *z* be its terrain surface height, *h* the host cell's centre
    terrain height, and *B* whether the chosen cell carries a high bridge. A released parasite
    stands on the deck exactly when *B* and either the host stood on a high bridge or *z* < *h*.
    In that case its world height is *z* + 416 leptons and it is marked on the bridge.
    Otherwise it keeps *z* and is not marked on the bridge. The height comparison is exact
    integer arithmetic without fractional rounding. This applies after vehicle centring and
    before forced placement; it does not change the ground-record choice to a deck-record
    choice. An empty choice supplies no bridge to rescue an infantry parasite from removal.
    The failed-choice vehicle at world cell (0, 0) is not marked on a bridge.

    Neither centring, surface selection nor final forced placement makes another placement
    draw. S-0202 claim 4 owns raw draws for the one ranged choice, including rejection.
    These are placement choices only, not any choices independently caused by the host's death
    or later unit behaviour. S-0209 claims 1, 4 and 5 own occupancy and entry outcomes.
    S-0208 claim 3 owns the shared position-choice refusal and side-preference rules.
    - Backing: `measurable` (P10); `unmeasured` for the object-present refusal:
      no practical fixture isolating that flag's effect is established (S-0208 claim 3)
    - Confidence: high (failed-choice position, survival versus removal, placement choices),
      medium (accepted-point bridge-height consequences)

24. **A higher special hit strands the carried parasite.** When the projectile of claim 1
    detonates with takeover, bomb planting or electric charging selected under S-0514 claim 20,
    its carried parasite stays off the map, unattached and unable to act. It does not attempt
    claim 4's return to its firing cell, and no recovery timer starts. Detonation and subsequent
    projectile removal do not themselves kill or silently remove it, award kill experience,
    play its death effects or create a replacement unit. This is the same stranded state as
    claim 21, but reached through a detonation rather than removal without detonation.

    Target latchability makes no difference: a legal mobile host, a mobile target that has
    become non-latchable during flight, a building, a cell or a lost target all give this
    outcome. Success or failure of the selected higher effect makes no difference either.
    The selected effect's own consequences remain with S-0601 claim 5, S-0519 claim 2 and
    S-0518 claim 11 respectively; this claim does not prevent a separate effect from removing
    the parasite itself. Firing legality remains claim 5: a shot refused before launch never
    creates a carried parasite.

    This is `quirks.parasite_suppressed_latch_strands_unit` (default original). With it off,
    after the higher selected effect the surviving carried parasite instead attempts claim 4's
    return, including claim 20's placement refusal and silent removal. This alternative does
    not permit attachment or any other suppressed special effect. It is independent of
    `quirks.parasite_carrier_loss_strands_unit`, which covers removal without detonation only.
    - Backing: `measurable` (P14)
    - Confidence: high

25. **The grab animation (Q-2671).** In this claim a *hold* is the time from a grappling parasite's
    latch until it leaves its host, not claim 7's paralysis hold. While a grappling parasite holds a
    host, the parasite itself is not drawn. In its place it has at most one animation of the fixed
    grab animation type (Import mapping), from its creation (below) until its removal or its end.
    The splash effects and wakes that claim 10 creates during the hold are other animations, not the
    parasite's picture. No field of the parasite's type, weapon or warhead names that type, so it is
    the same for every grappling parasite type.
    - **Creation.** It is created once per hold, in the grab's first turn (the host's turn *g* of
      F3), before that turn's phase-frame step. Like every step of the cycle, this happens only in a
      turn of the host that reaches the attached-parasite step (claim 18): when the host's first
      turns after the latch do not (claim 18's warp turns in our engine), the grab and the creation
      start in the first turn that does. When the host's cell in that turn is not water, the
      parasite leaves at once (claim 10) and no animation is created. It is created at the host's
      position, height included, with no initial loop delay, a repeat factor of 1 and no reverse
      request. It is not created at the end of the grab, at a push start or at a strike. A later
      latch, starting a new hold, creates a new one. When no animation type has the fixed name, none
      is created and the cycle runs as claim 10 gives.
    - **Attachment.** In that same turn the parasite attaches it to the host with no displacement
      (S-0109 claim 22), and it attaches it again in any later turn of the hold that reaches that
      step and finds it detached. So it follows the host's position, height included, and is in the
      ground render layer. The host's rocking (S-0113 claim 14) tilts the host's body but not the
      animation, which is always drawn upright.
    - **Stage.** The cycle sets the animation's stage *k* (S-0109 claim 1). Each time the phase
      frame *j* of table G steps to a value from 1 to 9, the parasite sets *k* = *b* + 10 × *d* +
      *j*, leaves the animation's direction of play unchanged, and restarts its stage clock with a
      stage delay of 128 ticks. That restart is in ticks at every game speed: S-0109 claim 4's
      normalisation does not replace it, even for a type marked normalised. *d* is the 8-direction
      (glossary, S-0204 F4) of the host's current body facing at that step. *b* is 0 in the grab, 80
      in a push with *σ* = −1 and 160 in a push with *σ* = +1 (*σ* as S-0113 claim 14 gives it from
      the push's way draw). Counting a phase's first turn as turn 1, in turns that reach the
      attached-parasite step (claim 18), *k* is therefore set in the grab's turns 3, 6, …, 27 and in
      a push's turns 4, 8, …, 36, never more than 8 such turns apart. When the host takes such a
      turn in every tick, the settings are at most 8 ticks apart, inside the 128-tick restart, so
      the animation does not advance by itself between them. When ticks pass without such a turn (a
      missed turn, S-0203 claims 3, 4 and 10, or a warp turn of claim 18), its clock runs on from
      the last setting under S-0109's rules (below). A step to phase frame 10 sets nothing, and
      phase frame 0 is never set: a push's first three turns set nothing, so the animation keeps the
      last value set unless its clock runs out under S-0109's rules (below). The image frame shown
      is *a* + *k*, *a* being the type's start frame (S-0109 claim 1).
    - **Before the first setting.** In the grab's first two turns the animation plays by its own
      type's fields under S-0109's rules (claims 2 to 8). Its first update follows S-0109 claim 5:
      it takes its first turn in the creation tick, after every object already in the update order,
      unless a removal costs it that turn, and its first advance is the one that claim's
      first-image-frame rule gives, including its case of a 1-tick delay whose first update falls in
      the next tick. In the grab's third turn the host's step sets the stage before the animation's
      own update. So, when the host takes its turn in each tick from the latch to the grab's third
      turn and each of those turns reaches the attached-parasite step (no removal costs it one,
      S-0203 claims 3, 4 and 10, and no warp suppresses the step, claim 18), only a type whose stage
      delay is 1 tick can advance by itself before the first setting, and whether it does follows
      those rules of S-0109 claim 5. When the host misses a turn there, or takes one that does not
      reach the step, more ticks pass before the first setting, and S-0109's rules decide whether a
      longer delay also runs out first. The creation's lack of a reverse request does not override
      the type's own reverse flag: a type marked reversed starts at *k* = λ<sub>e</sub> − 1 and
      plays reversed (S-0109 claim 8), while the cycle's settings are the same for both directions.
    - **Between settings and after a switch.** Outside the cycle's settings, S-0109's rules (claims
      2 to 16) apply from the stage, direction and clock the cycle last left (before the first
      setting, from those of the creation): an own advance once the clock runs out, a loop delay at
      the end of a pass that a new pass follows (S-0109 claim 11), the end of its last pass (S-0109
      claim 16, including its rule for a refused placement) or the switch to its type's follow-up
      type with that type's fresh stage delay (S-0109 claim 15). This happens in the grab's first
      two turns and in turns of the host that do not reach the attached-parasite step (claim 18's
      warp turns in our engine), where nothing sets the stage or restarts its clock. When the
      animation has ended, nothing recreates it in that hold. When it has switched, the cycle's
      later settings set the new animation's stage and restart its clock as above.
    - **Palette and brightness.** In each of the host's turns of the hold that reaches the
      attached-parasite step (claim 18), its creation turn included, the parasite gives the
      animation the house-colour palette of the parasite's house, the one the parasite is drawn with
      when it is on the map (S-0103 claim 8), and, when the host is in a cell, the unscaled ground
      level of the host's cell (S-0104) as its brightness. In a turn that does not reach the step
      (claim 18's warp turns in our engine), it gives neither, so the palette and brightness keep
      the values last given and a change of the cell's light shows from the next turn that reaches
      the step. It is then drawn by row 3 of S-0103 claim 8 (Changes for other specs), as an
      object's own animation drawn with its object, unless an earlier row of that table, or its
      tiled-copies rule, applies to its type: so the type's normal-light flag still gives it
      brightness 1000, and a tiled type's copies use the anim palette with this brightness kept.
      Otherwise the palette is the parasite's house colour, not the host's. The parasite sets the
      palette itself, so the art flag for drawing an animation with its object (S-0103 Import
      mapping) does not change it.
    - **Translucency and viewers.** Its creator asks for no translucency, so its type's own
      translucency and detail fields apply (S-0109 claim 20 for translucency, claim 19 for the
      detail field). It is drawn for every viewer by the ordinary animation rules. Neither the cloak
      or submerged state of the parasite, which is off the map, nor the host's cloak changes whether
      it is drawn. These states seldom last: a held host is uncloaked (claim 8), and a parasite
      uncloaks when it fires its latching weapon (S-0516 claim 11).
    - **Removal.** Apart from an end under S-0109's rules (above), it never ends by itself. It is
      removed at once when the parasite leaves the host by any way of claims 11 to 13 (a cull, any
      way of claim 12, a host cell that is not water), when the strike's own hit destroys the host
      (in that strike's turn), when the host is lost otherwise (claim 15) and when the parasite is
      removed from the game. A culled host therefore sinks without it. At each of these removals it
      stops being drawn from that turn on and leaves play in that turn, as an animation that ends
      does (S-0203 claim 10). The parasite removes it before the host leaves play, so S-0109 claim
      22's deletion at the animation's next update after its owner's removal does not apply. Created
      during the host's turn, it stands after the host in the update order (S-0109 claim 29), so its
      leaving during the host's turn costs no object a turn. When it leaves during another object's
      turn, as at a host loss of claim 15 caused by another object, S-0203 claim 10 decides by its
      place against that object.
    - **Draws.** Its creation takes, from the scenario random stream, the draws S-0109 gives any
      animation created with no loop delay: claim 3's stage-delay draw, when its type has a
      random rate, then claim 13's start-event draws. They are taken at the creation, in turn
      *g*. Setting its stage, palette, brightness or attachment takes no draw. It belongs to the
      simulation like every animation (S-0109 claim 28), and no cosmetic stream is used.
    - Backing: `measurable` (P15 for the type, creation, the position on the map, no tilt, stage,
      no own advance between settings when every turn reaches the step, the start before the first
      setting for a type with a stage delay of exactly 2 ticks, as P15's test type has, and a host
      that misses no turn, a reversed type, palette, brightness, its following of a light change,
      the type's own translucency, the viewers owning the parasite and the host, the attachment
      and the animation following the host while it moves in the release variant, a new animation
      at a second latch, and removal on the off-water release, at the destroying strike and at the
      cull; P8 for the draws; the tiled-copies rule and the precedence of S-0103's earlier rows
      have S-0103 claim 8's backing, and the detail field has S-0109 claim 19's). `unmeasured` for
      where the removed animation leaves the update order and so for its effect on later objects'
      turns: P15 records only the drawing. `unmeasured` for the creation's lack of an initial loop
      delay and its repeat factor of 1: the first setting restarts the clock, and no run reaches a
      pass end before it. `unmeasured` for the level being the unscaled ground level rather than a
      scaled one: P15's lighting variants change the ambient level only, which may move both
      alike. P15's extra-light variant tells the unscaled ground level from the host's own
      brightness. `unmeasured` for a first grab turn that finds a cell that is not water: the
      latch needs a water cell (claim 2), and no run is known to move the host off water between
      the latch and the grab's first turn. Removal by the other ways of claim 12 is `measurable`
      by claim 12's scenarios with P15's recording of the grab animation added to them.
      `unmeasured` for the art flag for drawing with its object, which no P15 variant sets: it
      matters only to a type that sets it off, and the flag is on when absent (S-0103 Import
      mapping). `unmeasured` for the 128-tick value itself, its being in ticks at every game
      speed, and for the animation's own rules between settings and after a switch: in turns that
      reach the step the settings come at most 8 turns apart, so 128 shows only in turns without a
      step, which in our engine are claim 18's engine-design warp turns while the original's step
      through them is unresolved (S-0513 claim 52), and a follow-up switch, a loop delay or a pass
      end before the first setting needs a type with a 1-tick delay, whose first advance S-0109
      claim 5 conditions on the update order. `unmeasured` for removal with the parasite removed
      from the game while attached: no skirmish action removes an attached grappling parasite
      without it leaving the host or the host being lost. The rows of claim 12 that remove the
      parasite silently do so as it exits the host (their last column), so they are ways of
      leaving the host. `unmeasured` for the absent type, which needs an install without that
      animation type, and the stock-data setup offers none. `unmeasured` for the reattachment of a
      detached animation: no skirmish event is known to detach it during a hold (S-0109 claim 22
      rates detachment `unmeasured`). The creation, settings, palette and brightness in turns that
      do not reach the step follow claim 18's engine-design warp contract and take its backing.
      `unmeasured` for the cloak and submerged states: a held host is uncloaked (claim 8) and a
      parasite uncloaks when it fires its latching weapon (S-0516 claim 11), so no run is known to
      hold either state while the animation is drawn. `unmeasured` for the height: a host held by
      a grappling parasite is on water, where no run tells its height from the cell's.
    - Confidence: high (the fixed type, one creation per hold in the grab's first turn on water, the
      position on the map at creation, the stage formula, the type's own reverse flag applying, the
      parasite's palette in turns that reach the step for a type that meets no earlier row of S-0103
      claim 8 and is not tiled, the normal-light flag's brightness 1000, no translucency asked by
      the creator, no tilt, no draw for the settings, removal at the off-water release, at the
      destroying strike and at the cull, the animation no longer drawn from that turn on, the
      viewers owning the parasite and the host); medium (the attachment and the animation following
      the host's position, a new animation at a later latch, the removed animation leaving play in
      that turn and its effect on later objects' turns, the creation's lack of an initial loop delay
      and its repeat factor of 1, the same type for every other grappling parasite type, the palette
      where an earlier row or the tiled-copies rule applies, as S-0103 rates the precedence between
      rows, the detail field, the art flag for drawing with its object not changing the palette, no
      creation when the first grab turn finds a cell that is not water, removal by the other ways of
      claim 12, the brightness and its following of a light change, as S-0103 claim 8 rates the
      brightness of row 3, the height, the cloak and submerged states, no own advance between
      settings, which rests on the 128-tick value, the 128-tick value of the restart and its being
      in ticks at every game speed, the reattachment of a detached animation, the creation,
      settings, palette and brightness in turns that do not reach the step, which follow claim 18's
      engine-design contract, the turns that set the stage, which rest on the phase stepping of
      claim 10 and S-0113 claim 14's frame count *k*, the first update and any own advance before
      the first setting, and the start when the host misses a turn or takes one that does not reach
      the step, S-0109's rules between settings and after a switch, viewers other than the owners of
      the parasite and of the host, which P15 does not run, removal at host loss and at the
      parasite's removal, the creation draws, which depend on the type's fields under S-0109, the
      absent type)

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

**F2, drone rocking strength.** This spec owns the strength of a drone delivery's push:

> strength = 1.5

The push point is S-0113 F3's: about 2 leptons to the host's left when the side draw gives 0 and
about 2 leptons to its right when it gives 1, with floating-point trigonometry and truncation
toward zero to whole leptons.

**F3, grappling cycle timing.** *g* is the host's first turn after latching that reaches the
attached-parasite step of claim 18 (the grab's first tick):

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
| Suppression set by curtain protection, unloading or consumption | 50 | ticks | Claim 12 |
| Drone rock displacement | 2 | leptons | Claim 9, S-0113 F3 |
| Drone rock strength | 1.5 | (rocking units) | F2 (owner); claim 9, S-0113 claim 13 |
| Grab phase | 10 frames × 3 ticks | ticks | Claim 10 |
| Push phase | 10 frames × 4 ticks | ticks | Claim 10 |
| First strike after the grab starts | 69 | ticks | Claim 10, F3 |
| Strike period | 40 | ticks | Claim 10, F3 |
| Splash effects per strike | 3 | effects | Claim 10 |
| Push roll | as S-0113 F4 | radians | Claim 10, S-0113 claim 14 |
| Exit quarter turn | 16383 | 16-bit facing units | F4 |
| Failed-latch facing | 0 (map north) | facing | Claim 4 |

## State and ordering

**Parasite states.**

| State | Event | Result |
|---|---|---|
| on the map, armed with a latching weapon | fires at a target (claim 5 allows it) | off the map, inside the projectile; mobile target's latch block set to now + 20 |
| inside the projectile | the projectile detonates with attachment selected under S-0514 claim 20; its target is latchable | attached to the target (claim 3) |
| inside the projectile | the projectile detonates with attachment selected under S-0514 claim 20; its target is not latchable, or is a cell | placement attempted in its last cell, facing north, not held (claims 4 and 20); removed silently when that fails |
| inside the projectile | detonates with takeover, bomb planting or electric charging selected, whether successful or not | stranded off the map, unattached, regardless of target latchability (claim 24) |
| inside the projectile | projectile removed without detonation | stranded off the map, unattached (claim 21) |
| attached (drone) | the host's turn, delivery due | delivery (claim 6); host held for *P* |
| attached (grappling) | the host's turn | host held for *P*; the cycle advances (table G), or leaves when the host's cell is not water |
| attached | a way of claim 12 | removed silently, or exits by claim 13 and is held for 3 × *R* (not held after a cull or leaving because the host's cell is not water) |
| attached | the host leaves the map or is removed | removed silently when suppressed (naval too), unless temporarily invulnerable, else placed by claim 15 without a hold |

**Table G, grappling cycle.** One row per phase. Phase frames (cycle counters, not drawn frames)
advance one per 3 ticks in the grab and
one per 4 ticks in a push. The phase ends when its 10th frame is reached.

| Phase | Each turn | When the phase ends | Draws at the end |
|---|---|---|---|
| grab (starts at latching) | host held; the grab animation created in the first turn (claim 25) | a push phase starts in the same tick | 1 (range 0 … 1: push way), then six wake-displacement choices (claim 10) |
| push (either way) | host held; host roll per Constants | the strike happens in the next tick | none |
| strike (one tick) | host roll 0; three splash effects; cull test (F6) | cull: the parasite leaves, not held afterwards (claim 11); otherwise the hit, then, when the host still has the parasite, a push phase starts in the same tick | 3 × (range 0 … 1, range 0 … *n* − 1) before the test; 1 (range 0 … 1) and six wake-displacement choices after a hit that leaves the attachment intact |

**Ordering consequences.**
- A parasite acts late in its host's turn (claim 18), after due healing, behaviour processing
  and movement. When two drone parasites' deliveries fall in the same tick, the host that comes
  earlier in the update order is damaged first.
- A sonic or healing hit releases a parasite before the hit's own health change (claim 12, supplementing S-0315 claim 7), so a sonic
  hit that also kills the host frees the parasite by claim 13, not claim 15, when it was not
  suppressed.
- In a cull, the parasite comes out (claim 13) before the host starts to sink.
- The latch block is set when the shot is fired, so a second latching shot at the same target is
  refused from the next firing check onward.

## Edge cases and quirks

- **`quirks.parasite_suppressed_latch_strands_unit`** (default original): claim 24 leaves a
  carried parasite stranded when a higher selected special hit suppresses attachment, even
  against a latchable target. Off: attempt claim 4's return after that selected effect,
  without allowing any suppressed effect. This does not change claim 21's separate switch.

- **`quirks.parasite_carrier_loss_strands_unit`** (default original): claim 21 strands a parasite
  off the map when its carrying projectile is removed without detonating. Off: attempt claim 4's
  failed-latch return, including silent removal when placement fails.

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
- **Curtain protection, unloading and consumption remove a non-naval parasite** (no switch; defined behaviour):
  the 50-tick suppression set just before the exit makes the silent removal certain.
- **`quirks.parasite_host_loss_bridge_test`** (default on): the fallback neighbour's restricted
  land passes when the host's cell has a bridge, even if that neighbour has none. *Fixed:* test
  the neighbour's bridge instead.
- **`quirks.parasite_host_loss_failed_choice_origin`** (default original): claim 23's failed
  standing-position choice sends a vehicle parasite to world position (128, 128, 0) leptons
  outside the playable map instead of removing it. Off: remove silently, keeping the same
  position-choice draw count. This is a gameplay bug, not merely a misplaced drawing.
- **Re-selection on exit** is presentation; claim 19 fixes the original's display-only glitch
  without a quirk switch.
- **A drone parasite on an infantry host** kills it at the first delivery whatever its health,
  since the hit ignores defences. The can-carry-a-parasite default for infantry types is yes.

## Import mapping

| Source | Key | Parse form | Feeds | Absent-key default |
|---|---|---|---|---|
| rules, warhead section | `Parasite` | boolean | the parasite flag (latching weapon, Terms) | no |
| rules, warhead section | `MindControl`, `IvanBomb`, `ElectricAssault` | boolean, each | takeover, bomb-planting and electric-charging selection, respectively (S-0514 claim 20); each suppresses attachment and invokes claim 24's stranded outcome | no, each |
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
| rules, mobile type section | `SpeedType`, `MovementZone`, `Locomotor` | class name, zone name, locomotor identifier | placement mobility (claim 20); interpretation owned by S-0211 claim 10 and S-0207 claim 14 | as S-0211 and S-0207 |
| none | none | not imported | within-host-turn order (claim 18), carrier loss (claim 21), suppressed-attachment fate (claim 24), air threshold (claim 22), placement options, failed-choice outcome and fallback surface (claims 15, 20 and 23) | fixed engine behaviour; no separate INI setting |
| rules `[AudioVisual]` | `ConditionRed`, `ConditionYellow` | ratio (S-0003) | red and yellow ratios (F6) | as S-0111 |
| rules `[General]` | `VeteranRatio`, `VeteranCap` | as S-0311 | F5 | as S-0311 |
| rules `[CombatDamage]` | `SplashList` | list of animation names | the splash effects and the size *n* of claim 10 | as S-0303 |
| rules `[General]` | `Wake` | animation name | the wakes of claim 10 | as S-0312 |

The grab animation of claim 25 is not named by any key of the parasite's type, weapon or warhead:
the original uses a fixed animation type name, given in the table below. It is an engine constant,
not stock data, and it is the same for every grappling parasite type.

| Source | Key | Parse form | Feeds | Absent-key default |
|---|---|---|---|---|
| none (fixed in the original) | animation type `SQDG` | name, found without regard to letter case (S-0004 claims 2 and 7) | the grab animation (claim 25) | none: no grab animation, and the cycle runs as claim 10 gives |

The team keys that ask a parasite to rejoin its team after coming out concern the original's
computer player and are out of scope.

| Source | Key | Parse form | Feeds | Absent-key default |
|---|---|---|---|---|
| rules `[CombatDamage]` | `DefaultSparkSystem` | particle-system type name | spark effect of claim 9; the same key also feeds the overload sparks of S-0601 claim 16 | not established |

## Measurement scenarios

Use a small purpose-made skirmish map, game-speed index 2, 800×600, no engine add-on and no mods (the named test files of a scenario excepted),
with one bystander computer player kept outside the observed area and the glossary's void rule.
Run each deterministic part three times. Use stock data except the named map override variants
below, removed after the test. Scripted input, video and a read-only state recorder are permitted.
A bench or arena adaptation should record ticks, health, positions, targets, experience and release
state; these are requested observables, not a claim that the existing harness records them already.
For every part, a result different from its stated expectation falsifies that part.

- **P1, latch, deliveries and targeting (claims 1, 3, 6, 7, 9, 17).** Place a drone-parasite test
  unit owned by A, a moving vehicle of enemy B, and a second A vehicle within weapon reach. Order
  the parasite to attack B's vehicle, then the second A vehicle to attack its host. Record launch,
  attachment and health-change ticks, the host's roll and pitch per tick, effects, movement and
  attack targets. Expected: disappearance at launch, attachment to the original target even when it
  has moved away from the detonation, first delivery on the next host turn, then F1; no detonation
  area hit on a nearby bystander. The host can attack the second A vehicle but never its own
  parasite, and the second A vehicle still targets the host. Named map variant: make the host
  infantry and latchable with a nonzero versus value. Expected: the infantry delivery ignores
  defences and kills at first delivery, whereas ordinary non-infantry damage follows S-0302. Give
  the drone's primary weapon distinct rookie and elite damage and cadence; expected the rank's
  primary slot, without a firing-bonus multiplication. Set paralysis length shorter than, equal to
  and longer than rate of fire, plus zero rate of fire. Record hold state and move attempts between
  deliveries: expected F1 and cessation of hold on release. Do not infer the hold's starting tick
  from movement that occurred earlier in that same tick. For the no-escape marker, use a named
  transport-host variant carrying passengers whose ordinary lethal-hit control permits escape under
  S-0312. Kill it by a parasite delivery: expected no escaping passengers. Repeat the
  lethal-delivery comparison in P6 with culling disabled. An escaping passenger falsifies the marker
  claim. For facing effects, give the delivery animation list eight visually distinct test entries,
  then an empty list. Repeat for all eight host directions: expected the corresponding entry, or no
  delivery animation for the empty list, with the spark effect unchanged. During attachment, deliver
  a nonlethal area hit that reaches the host and record both units' health: expected no damage to
  the off-map parasite. Record the parasite's target throughout attachment: expected none, even
  while other enemies enter range.
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
  parasite; the host's hold ends. Repeat sonic release for all eight host directions,
  with blocked candidate cells and with no qualifying nearby cell: expected host-mobility
  search, or silent removal on failure. Repeat with a naval parasite for the side candidate
  and its living-host suppression exemption. Named map variants: a latchable airborne host
  (expected silent removal at release), a latchable amphibious host on a beach or water,
  a host on a bridge over water, and a host with a terrain object or building
  in its cell. Expected: exactly the host-cell test of claim 13, including the bridge exception
  only to land type. Compare a host and parasite with different movement zones to discriminate
  host-based search from parasite-based search. Make the sonic hit lethal: expected release
  before the host's health changes, using claim 13 rather than claim 15.
  In separate stock-data runs, start curtain protection on an infected non-organic vehicle
  and on a ship carrying a naval parasite, keeping legal exit cells clear. Record suppression,
  attachment, parasite existence and host hold at activation. Expected: suppression set to
  50 ticks before release, drone silently removed, naval parasite exiting by claim 13, and
  host hold ended (S-0513 claim 25). Continued attachment or a surviving released drone
  falsifies this result; missing naval release with clear placement also falsifies it.
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
- **P6, grapple and cull (claims 7, 10, 11, 16).** Order a grappling-parasite test unit to attack an
  enemy naval host on open water. Record health, phase changes, the host's roll and pitch per tick,
  wake and splash counts and experience. Expected: first strike at F3's exact tick, later strikes 40
  ticks apart, uninterrupted hold, three wakes at each push start, three splashes at each strike,
  and push roll following S-0113 claim 14 and F4, with the sign of each push's own way draw, and
  zero roll at the strike. Record local loss announcements too: expected the ordinary local
  unit-lost gating, not an unconditional sound. A one-tick discrepancy falsifies the proposed
  timing, rather than being accepted as tolerance. At red health the next eligible strike culls,
  releases without a hold and starts sinking without health loss. Named variants: different weapon
  rate of fire (no cadence change), cull flag off (ordinary hit), elite parasite with yellow host
  (cull), untrainable parasite (no cull experience), differing house-cost factors, allied and
  unscored hosts. Expected: F5 without victim-rank, alliance or unscored filtering, and no ordinary
  kill credit or score from culling. In a named map variant, set paralysis length to zero so an
  amphibious host can follow a route onto dry terrain while carrying a grappling parasite. Expected:
  release at the first host turn that finds a non-water cell, without a post-exit hold. Named
  long-lived hosts allow the phase test to finish before culling.
- **P7, teleport and replacement (claim 12).** Infect a unit using the teleporting locomotor
  with a drone parasite, then order a jump; record the old and new positions and release tick.
  Expected: exit at the old position before departure. Separately record a teleporting
  superweapon's relocation of an infected vehicle, distinguishing attachment, map presence,
  silent removal and release at launch, departure and arrival. Expected: no drone release at
  launch; its later fate remains an open question, not an asserted ordinary-jump outcome.
  Repeat with a ship carrying a grappling parasite: expected release
  at launch at its old position. Apply a locomotor-replacing weapon to a latchable host carrying
  a naval parasite and then a non-naval one (named map variants if needed); expected release
  only for the naval parasite.
- **P8, random choices (claims 9, 10, 25).** Use P1 and P6 setups and the random-state recorder for
  twenty deliveries and twenty surviving strikes. Predict ranged choices with S-0202 claim 4.
  Named map overrides disable optional particles and animation randomness to isolate these
  choices, then restore each effect separately. Expected: a non-infantry drone delivery's one
  0 … 1 choice before its hit; no such choice for an infantry delivery. For grappling, expected
  the push-direction choice, six wake-displacement choices in the listed order, six splash
  choices before each cull test, and a new direction plus wake choices only after a surviving
  hit. In the grab's first turn, expected only the grab animation's creation draws that S-0109
  claims 3 and 13 give for its type's fields, and none from its later settings. Account
  separately for raw-value rejection and effect-owned randomness; an extra
  unaccounted ranged choice, a different range or a changed order falsifies the explicit list.
- **P9, hold, reuse and selection (claims 8, 14, 19).** Release a parasite from a vehicle by
  a sonic hit, then immediately issue move and attack orders to the released parasite.
  Record its hold, movement, shots, experience and
  selection. Expected: hold for 3 × primary rate of fire; non-organic firing is not blocked by
  hold alone for non-spawner weapons, organic firing is blocked; retained experience after release.
  Spawner weapons are refused regardless of organic type, as S-0506 claim 10 and its scenario R6
  specify. Named map variants
  give latchable hosts each locomotor listed in claim 8, organic and cloak flags as appropriate,
  and a harmless delivery weapon so hosts survive. Compare autonomous movement already ordered
  before attachment, fresh movement orders, organic firing, and cloak state while held:
  expected claim 8, excluding the unchecked drop-pod case. Give a locally owned parasite the
  re-select flag, select it before launch and observe release. Expected in the original:
  selection remembered only for infantry targets; our corrected presentation remembers every
  target. The correction itself is a design rule, not an oracle expectation.

- **P10, returning placement (claims 4, 13, 15, 20 and 23; Q-0620).** Use P3's failed latch,
  first with a vehicle parasite and then a named infantry-parasite map variant. During flight
  occupy the original cell with an allied vehicle, enemy vehicle, allied infantry, enemy
  infantry, tree, wall and ordinary building, separately. Keep an adjacent cell clear. Record
  position, removal and scenario-stream state. Expected: claim 20's clear-only acceptance,
  never a retry in the adjacent cell, waiting or obstruction attack. A named crusher variant
  separates enemy crushable infantry from allied infantry. Record occupant health before any
  later parasite turn to distinguish placement from later crushing.
  For infantry, vary ground side-position marks from none to all three, leaving centre and top
  free; expected table O's selected free side position or removal, not numeric position order.
  Record the draw even on a later entry refusal. Repeat on a high bridge with ground and deck
  blockers separately: expected the ground-record entry test. Repeat sonic release to exercise
  claim 13's final placement separately from its earlier search. For host loss arrange a rejected
  host cell and a valid front neighbour, with zero, one and two occupied side positions, then
  all three for both kinds. Also repeat with a vehicle-present refusal and with restricted
  land rejected before choice. The object-present refusal is unmeasured: a non-traversable
  object alone does not establish that flag or isolate its effect (S-0208 claim 3).
  Expected: ground-record centre-class choice before forced placement and claim 23's table.
  All three side positions taken consumes one ranged choice; the vehicle-present refusal
  consumes none. Infantry is removed in both tested failed-choice cases, but the vehicle survives at
  world position (128, 128, 0) leptons, unmarked on a bridge, not at the front neighbour or
  another nearby cell. Record the parasite's actual position and existence immediately during
  release, before its next behaviour turn; lack of a visible unit in the local area alone
  cannot distinguish this outcome from removal. Restricted-land rejection instead removes
  either kind without a placement choice. Any differing fate, point or placement-choice
  count falsifies the corresponding claim. No extra placement draw follows a successful choice.
  For accepted fallback points compare a host on and off a high bridge, a chosen cell with
  and without a high bridge, and chosen terrain heights below, equal to and above the host
  cell's centre terrain height where a map permits each case. Record world height and bridge
  mark as well as ground/deck standing-position marks. Expected: claim 23's surface rule,
  independent of the ground-record choice. Use named map variants where needed to make
  infantry parasites or hosts latchable, but do not alter placement or random behaviour.
  Run each arrangement three times with exact recorded seeds and both kinds of parasite.
  The common recorder settings and the no-engine-add-on, no-mods setting apply. The recorder must include positions
  outside the playable map and stream state around release, not only visible units.
- **P11, loss of the carrier (claim 21; Q-0621).** A named map variant gives a drone parasite
  an inaccurate non-steering projectile and a distant moving enemy at a legal map border.
  Keep its firing cell and neighbours clear. Record projectile position, detonation, parasite
  map presence, health and kill experience for 300 ticks after removal. Collect three shots
  that cross the legal border without detonating. Expected: no return or attachment, unchanged
  parasite health, no death effects or experience award, no recovery. Distinguish an extant
  off-map parasite from silent removal with the recorder. Return or deletion falsifies the
  claim. A control detonation on a non-latchable target instead returns under claim 4.
- **P12, within-host-turn order (claim 18; Q-0622).** Named P1 map variants give the host
  self-healing, survivable drone damage, and paralysis length shorter than delivery interval.
  Arrange a delivery on a healing tick at full strength: expected final health is strength
  minus the resolved hit, not that amount plus a later heal. Repeat with only house-wide healing.
  Record health, hold and position on a pre-ordered straight drive. On a delivery after a
  hold-free gap, expected movement that turn followed by hold; discard turns already held at
  their start. Give an organic vehicle an otherwise-ready shot due that turn: expected refusal
  after renewal. Repeat with an aircraft executing a due attack-run shot during behaviour
  processing: expected the earlier shot is not cancelled. For behaviour order arrange a due
  processing that chooses a random delay and the drone rocking choice in one turn. Isolated
  controls account for all other effect draws; expected behaviour choice before parasite choice.
  In another part, use two hosts whose update order is opposite to their parasites' original
  order, with deliveries due in the same tick and other random effects disabled. Record the
  two rocking choices and health changes: expected assignment in host order, not parasite order.
  The recorder logs per-turn health, movement, shots, targets and stream state. Opposite ordering
  falsifies the corresponding part. P6's zero-hold off-water run expects release on the turn movement first
  reaches dry terrain, not one turn later.

  **Warp part (engine-design check, not a stock-game expectation).** With a surviving
  attachment and an otherwise-due parasite step, record each pending, leaving, departure,
  retry, placement and arrival-effects update of an eligible host, following S-0513
  claim 51 and its SW23 part f. Record health, deliveries, grappling phase, countdown
  deadlines and explicit release events separately. Expected: no ordinary parasite step
  through arrival effects; during arrival hold and its expiry, ordinary eligibility only
  after the hold-end test, with elapsed countdown time retained and no catch-up burst.
  An ordinary delivery or phase advance during exclusion, or a deferred burst, falsifies
  the contract. A launch or departure release is not an ordinary step and does not falsify
  it. Original-game observations follow S-0513 claim 52 and SW23 part g without predicting
  this design result or promising that a drone attachment survives departure.
- **P13, air threshold (claims 13, 15 and 22; Q-0623).** Named P4 map variants provide
  latchable jumpjet-infantry, hover-vehicle and ordinary aircraft hosts. Release with nonlethal
  sonic hits on ground, on a high-bridge deck and during ascent or descent. Record map presence,
  world and terrain heights, bridge mark and parasite fate. Obtain three releases below 208
  leptons and three at or above it; include exact 207 and 208 where normal movement reaches them.
  Keep other host-cell tests passing and exit cells clear. Expected: release below 208, silent
  removal at or above 208; a deck-standing host does not fail this test. An ordinary aircraft
  in flight but below 208 still passes. Repeat host removal at matched heights: claim 15 chooses
  direct versus fallback placement with this air answer, rather than forbidding both. Include
  falling or lifted hosts where reachable; motion labels alone do not change the answer. Record
  transport entry separately, expecting host-loss handling rather than another carried-state air
  rejection. Any different classification falsifies claim 22.

- **P14, special-hit selection and carried-parasite fate (claims 1 and 24; Q-1032).** Under the common setup, use named map weapon
  variants retaining the launch-inside and parasite flags. Separately add takeover, bomb
  planting or electric charging, with the firer capability needed by that effect, following
  S-0514 claim 20 and its E13 scenario. Record firing-time map departure, latch block,
  attachment, owner, bomb presence, charger count and health. Expected: departure and latch
  block still occur, but the higher-priority effect suppresses attachment. Repeat with an
  ineligible target for that effect, changing eligibility after firing where firing legality
  otherwise prevents delivery: expected no fallback attachment. In a separate variant enable
  temporal erasure with attachment and use a firer capable of both. Expected: attachment,
  not erasure; make the target non-latchable during flight and expect claim 4's return, still
  without erasure. Record beam attachment as well as parasite attachment. Any attachment
  despite a higher selected effect, erasure despite selected attachment, or missing launch
  departure falsifies the respective prediction. A shot refused before firing is inconclusive.

  For each of the three higher effects, retain a clear firing cell and record the parasite's
  existence, health, map presence and attachment at detonation and for 300 ticks afterwards.
  Include an ordinary vehicle firer whose electric-charging attempt cannot succeed, and a
  named infantry-parasite variant that can charge a building or plant a bomb. A takeover
  variant must supply its ordinary takeover capability. Compare a latchable mobile target
  with one made non-latchable during flight, plus building and forced cell targets where
  firing rules permit. Record lost-target detonations too; discard shots removed without
  detonation, which instead test claim 21. Where eligibility prevents firing, change the
  target during flight rather than bypassing the firing check. Use three completed shots
  per reachable arrangement. Expected: the surviving parasite remains extant, off the map,
  unattached and inactive, with no failed-latch return, death effects, kill experience or
  recovery, regardless of host legality or higher-effect success. The building's charging
  or bomb outcome is recorded separately, not mistaken for parasite attachment. Return,
  attachment or removal caused solely by carrier completion falsifies claim 24. Include an
  attachment-selected control that becomes non-latchable during flight: it returns under
  claim 4. The recorder must distinguish an extant off-map unit from silent removal, not
  infer either from disappearance on screen. No completed oracle measurement is claimed.
- **P15, grab animation (claim 25; Q-2671).** Use P6's set-up: a grappling-parasite test unit
  latching onto a ship on open water, the parasite's and the ship's houses with different house
  colours. A named test art file gives the fixed grab animation type a test sprite whose image
  frames each show their own index and each hold a block of pixels in the house-colour remap range
  of the palette (S-0103), with start frame 0, 240 image frames (enough for every stage *k* of claim
  25), a loop end of 240 and an end field of 240 given in the test art section, a stage delay of
  exactly 2 ticks and no reverse or tiled flag, and a named map lighting variant gives the map an
  ambient level other than neutral. Record frame by frame, from the latch to the sixth strike, and
  repeat with further seeds until a push of each way (S-0113 claim 14's *σ*, read from the host's
  rocking) has been recorded: every animation created near the host, with its tick, type, image
  frame, screen position against the host's draw point, render layer, palette remap colour and
  brightness, the host's body facing, each push's rocking way and whether the picture is rotated or
  skewed against the test sprite. Then release by moving the host onto a cell that is not water
  (P6's zero-paralysis amphibious variant) and record the same until two ticks after the release.
  Run it with the host at each of the eight 8-directions in turn, with the viewer owning the
  parasite and with the viewer owning the host. No superweapon is fired. The test map places no
  terrain object that takes turns and has the host as its first vehicle entry, so the host is the
  first map-placed object in S-0203 claim 3's order. The parasite, the second ship and the starting
  units stand after it, and every object created during the run, the splash effects, wakes and
  ending animations of the hold included, enters play after it (S-0203 claims 3 and 4), so a removal
  of any of them costs the host no turn (S-0203 claim 10). Only the game-start animations and the
  shared gas system, which S-0203 claim 3 does not order, can stand before it. Record every removal
  from the map with its tick for the whole run, and note which objects were in play at game start. A
  run is used for the frames and setting turns only up to its first removal of a game-start object
  other than the map-placed ones, which may cost the host a turn. Until then the host takes a turn
  in every tick and every one reaches the attached-parasite step, so its turns are counted by ticks.
  Repeat with further seeds until the grab and a push of each way have been recorded before any such
  removal. Named variants: a weak host that the first strike's hit destroys, with the cull flag off,
  and a culled host. In the weak-host variant a second ship of the host's house, put in play after
  the host, stands on open water near the first, and the parasite, back on the map on water, is
  ordered to attack it, so that it latches onto it. A named art variant of the test type with its
  reverse flag set, one with its translucency field set to a level other than none, and one with its
  normal-light flag set. A named rules variant sets the game-wide extra unit light (S-0104 claim 13)
  to a value other than 0, so that the host's own brightness differs from the unscaled ground level
  of its cell. Also record each drawn animation's translucency level. A named light-change variant
  sets the ambient change step and rate in the test map's game-wide visual section and fires a map
  trigger action that changes the ambient level (S-0104 claims 14 and 4) in the grab's first turn,
  so that the fade ends before the first strike. Expected: one grab animation, created in the grab's
  first turn at the host's position and in the ground layer, following the host while it moves in
  the release variant until it is removed, no second one in the hold, and a new one in the second
  ship's hold. Image frame 0 in the grab's first two turns; image frames 10 × *d* + *j* in grab
  turns 3*j* and *b* + 10 × *d* + *j* in push turns 4*j*, each held until the next setting, with *b*
  = 80 for a push of way *σ* = −1 and 160 for *σ* = +1. The parasite's house colour and the shade of
  the unscaled ground level of the host's cell, following that level in the light-change variant
  with a lag of at most one host turn (claim 25: a change shows from the next turn that reaches the
  step), brightness 1000 in the normal-light variant, the unscaled ground level, not the host's
  brightness, in the extra-light variant, and the same picture for both viewers; no translucency in
  the base run and the set level in the translucent variant; in the reversed variant, image frame
  239, which is λ<sub>e</sub> − 1 for the fixture's loop end (S-0109 claim 8), in the grab's first
  two turns and the base run's frames from the grab's third turn on; removal in the release turn, in
  the destroying strike's turn and at the cull, with no grab animation while the culled host sinks.
  Falsified by: a second grab animation in one hold, none in the second ship's hold, the animation
  left behind while the held host moves, a picture rotated or skewed with the host's rocking, an
  image frame or a setting turn other than these in the first ship's hold, an image frame change
  between settings, the host's house colour or the anim palette on the remap block, a viewer who
  does not see it, a translucency other than the type's, a brightness that lags the cell's level by
  more than one host turn, or the animation remaining after the release.

## Changes for other specs

- **For S-0103 claim 8 (*Which own animations are drawn with their object (row 3)* and *Brightness
  of row 3*), its Terms entry *Object's own animation*, its Import mapping row for the art flag for
  drawing an animation with its object, and its Open question on rows 2 and 3 of claim 8.** The list
  and the Terms entry can add a grappling parasite's grab animation (S-0614 claim 25): it always is
  an own animation drawn with its object, that object being the parasite, so it takes the parasite's
  house-colour palette, not the host's. The Import mapping row can add it to the animations that
  ignore that art flag, since the parasite sets its palette itself. The Open question's list of own
  animations, which may be incomplete, gains the grab animation. *Brightness of row 3* can add that
  a grab animation is drawn at the unscaled ground level of its host's cell (S-0104), set when it is
  created and set again in each of the host's turns that reach the attached-parasite step (S-0614
  claims 18 and 25), so it follows changes to that cell's light from the next such turn. The
  normal-light flag and the tiled-copies rule apply to it as to the other row-3 animations, and
  S-0103's precedence between rows is unchanged. Confidence: high for the palette in turns that
  reach the step for a type that meets no earlier row and is not tiled, and for the normal-light
  flag's brightness 1000; medium for ignoring the art flag, for the brightness and its following of
  a light change, as S-0103 already rates the brightness of row 3, and for both in turns that do not
  reach the step, which follow S-0614 claim 18's engine-design contract. Backing: `measurable`
  (S-0614 P15, its light-change and normal-light variants included), and `unmeasured` for the art
  flag, which no P15 variant sets, and for the level being the unscaled ground level rather than a
  scaled one (S-0614 claim 25's Backing).
- **For S-0104 claim 13 (*Object brightness*, the row-3 list of its *Animations* item), its Formulas
  table *Object brightness*, its Formulas line on the unscaled ground level and its Open question on
  the brightness of an object's own animation drawn with its object.** Beside "Any other animation
  of this row: not established", claim 13 can add the grab animation: *U* of its host's cell, given
  as in the S-0103 entry above, with the normal-light flag's 1000 taking precedence. The other
  animations of that row stay open. The Formulas table *Object brightness* can add a row: grab
  animation of a grappling parasite, *U* of the host's cell, given at creation and in each of the
  host's turns that reach the attached-parasite step (S-0614 claims 18 and 25), 1000 with the
  normal-light flag. The Formulas line on who uses the unscaled ground level can add the grab
  animation beside a building's own animations and parachutes, and the Open question can name it
  beside them as an exception. Confidence: high for the normal-light flag's 1000, medium for *U* and
  its following of a light change. Backing: `measurable` (S-0614 P15, its normal-light and
  light-change variants included), and `unmeasured` for *U* being the unscaled ground level rather
  than a scaled one: P15's lighting variants change the ambient level only (S-0614 claim 25's
  Backing).

## Open questions

- Whether creating the delivery animation, the wakes or the splash animations draws from the
  scenario random stream besides the draws listed in claims 9 and 10, and whether creating the
  game-wide spark effect of claim 9 draws (claim 9 relies on S-0616 claim 1 for no draw, which is
  not confirmed for the original).
- A weapon with the parasite flag on its warhead but without the launch-inside flag.
- Building-targeted detonations with parasite attachment selected: no parasite-specific firing
  guard applies, but their attachment result is not established. Suppression by a higher selected
  effect is settled by claim 24, including building targets.
- Nonempty delivery-animation lists shorter than eight entries: out-of-list results are not
  established; no wrap or safe fallback is claimed.
- Whether and how a superweapon's teleport releases a drone parasite at departure: the ordinary
  teleport jump's release is not established for that path (claim 12; S-0513 claim 39). P7
  records its fate without promising the ordinary-jump outcome.
- A drone parasite whose ship host is killed and sinks: whether the parasite stays inside during
  the sinking until the host is removed (claim 15 then applies).
- Whether the host-cell test of claim 13 can fail in skirmish play for reasons other than an
  amphibious host on water or a beach (a host in the cell of a building's exit, for example).
- How a visceroid-type host interacts with a parasite (the original reads the attachment there; not
  traced).
- The absent-key default of the spark effect.
