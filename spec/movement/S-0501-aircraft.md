---
id: S-0501
title: Aircraft flight, attack runs and pads
topic: movement
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0630, Q-0631, Q-0632, Q-0633, Q-0635, Q-0636]
supersedes: []
---

# S-0501: Aircraft flight, attack runs and pads

## Summary

How fixed-wing and rotor aircraft fly: their height, horizontal speed, landing and take-off; how
an armed aircraft carries out an attack run and spends its ammunition; how it finds, reserves,
lands on and is serviced at a pad; how the number of pad-bound aircraft a house may build follows
from its pads; and the flight and return of non-missile aircraft launched from a carrier.
The shared spawn-controller cycle, including replacement, reload and release, belongs to
S-0506 claims 1 to 9 and 11 to 13; missile flight also belongs to S-0506. The spy plane's dispatch and reveal are S-0513 claims 43 to 45 and the
delivery of paratroopers and cargo is S-0522 claims 1 to 16; only their flight is covered here.
Ordinary air-transport eligibility is S-0507 claim 2: capacity counts passengers, without a
passenger-size or size-limit test. Boarding releases mind control (S-0507 claim 6).
Landed passenger release is S-0507 claim 13; its airborne unloading transitions remain incomplete.
Aircraft cloaking and hidden-target loss are S-0516 claims 17 and 25.
Shared pad-capacity rules are S-0407 claim 16 and claim 30 here; repair-step rules are S-0314
claims 4 and 5 and claims 26 and 27 here. S-0504 claim 3 and S-0506 claim 10 own the
launching-count movement hold: it prevents the next step, not completion of an active step,
and preparing non-missile aircraft alone do not impose it (claim 35 here).

This spec covers skirmish; network games and campaigns are out of scope. Player houses controlled
by our AI use the human-controlled rules below. Computer-only behaviour described here records
the original only; the neutral and special houses retain it (glossary, *AI house*).

## Terms

- **aircraft** (glossary): in this spec, an object of an aircraft type that moves with the flying
  locomotor. Rocket-driven missiles are S-0506.
- **height** *h*: the height above the terrain (glossary), in leptons. In ordinary altitude
  calculations on a bridge cell, the bridge deck replaces terrain when the aircraft is not
  flagged as standing on the bridge and is at least the bridge height above terrain.
- **cruise height** *FL*: the type's flight-level field, or the game-wide flight level when the
  type's field is absent (Import mapping). The locomotor keeps a **held height** *FL*ₕ, which is
  *FL* during ordinary cruise and the landing height (0) while landing. Near a destination,
  guidance can instead use its height above the terrain at that destination (claim 6).
- **landing height**: the height at which an aircraft counts as landed for the fall-back rules:
  0 for every aircraft this spec covers (carrying aircraft that lift vehicles are out of scope).
- **on the ground** (this spec's test): *h* ≤ landing height, or the aircraft is drawn in the
  ground layer. **In flight** is the opposite. Wherever this spec says "in flight" it means this
  test, not either sense of the glossary's *airborne*. Two other tests appear and are named where
  used: *h* > 0 (claim 37), and the aircraft's **in-the-air answer**, which is S-0307's *airborne*
  (height above the surface at least 208 leptons) for every aircraft except the game-wide
  V3-rocket and dreadnought-missile types, which use their locomotor's own answer (S-0506 claim 22; claim 18 here uses that answer).
  The in-the-air answer is the exact test that S-0308's *airborne* leaves to this spec for
  aircraft.
- **visiting aircraft**: an aircraft marked as not meant to stay on the map. Successful placement
  sets this mark when the type is not both selectable and landable, or its primary weapon is a
  camera weapon. It does not clear an existing mark. A dispatch can mark an aircraft before
  placement. Placement height uses the mark already present, not the type test made afterward.
- **pad**: a building whose type is one of the aircraft type's dock types. In this spec *dock
  types* means only the aircraft type's own list of building types it docks at (glossary,
  *refinery, dock types*: the aircraft sense of S-0404 claim 2). A **helipad building** is a
  building whose type has the helipad flag; a **reload pad** has the reload flag.
- **contact places of a building**: max(*d*, 1), where *d* is its type's docking-place count
  (S-0417 claim 14). Each holds at most one *contact* (glossary). An aircraft holds at most one
  contact. A **pad reservation** is the contact between an aircraft and a pad.
- **pad-bound**: an aircraft type with the airport-bound flag. It may only rest on a pad.
- **landable**: an aircraft type with the landable flag. A type without it never lands.
- **fighter**: an aircraft type with the fighter flag.
- **armed**: for this spec's flight and rest rules, an aircraft with a primary weapon.
- **ready** (table T1): the firing check's *can fire* answer (S-0301 claim 9), not readiness to
  start a queued behaviour.
- **strafing weapon**: a weapon whose projectile type has a signed turn rate at most 1 and is not
  invisible.
- **speed fraction**: a floating-point value from 0 to 1 that multiplies the aircraft's speed.
  The locomotor holds a current and a target value.
- **unfinished flight**: movement accepted but not yet completed by settling at a landing
  destination (claim 40); this is distinct from a nonzero speed fraction.
- **attack run**: one approach to a target ending in one or more firing passes and one round of
  ammunition spent (claim 15). A **firing pass** is one processing in which the run fires.
- **run lock**: a mark an aircraft carries during the later passes of a strafing run, a fighter's
  firing pass and a spy plane's overfly. It makes the aircraft not ready to change (claim 31).
- **service step**: one round added or one repair step made by a pad for one aircraft in contact
  with it (claim 26).
- **pad total** of a house: the sum of the docking-place counts *d* of the house's placed helipad
  buildings (claim 30).
- **pad-aircraft list**: the game-wide list of aircraft types counted against the pad total.
- **standard delay**: S-0215 claim 6, *D*<sub>r</sub> + *U*(0, 2) on the scenario stream.
- Spawn-controller terms (*spawn controller*, *spawner*, *spawned type*, *spawn slot*, *slot
  object*, *slot state* and its values, *controller state*, *controller target*, *pending target*,
  *launch pass*, *launch spacing*, *launching count*, *regeneration*, *reload* and *release*)
  follow S-0506 claims 1 to 9 and 11 to 13. A **carrier slot aircraft** here is a non-missile slot
  object using this spec's ordinary aircraft flight, not a rocket missile.

## Behaviour

### Flight

1. **Placement.** When an aircraft is put on the map (by production, a map, a carrier launch or
   a superweapon), it is placed on the ground when it is not a visiting aircraft and its position
   lies inside the usable area. Otherwise it is placed at its cruise height above the terrain.
   Its current speed fraction starts at 1 when it is placed exactly at cruise height and at 0
   otherwise. Ground placement replaces the requested world height with the terrain height.
   It does not preserve an elevated launch request or a docking displacement's height.
   A later docking-position assignment is separate (claim 24; S-0404 claim 15).
   A type placed carrying passengers is marked as a passenger carrier for the rest of
   its life (claim 19). Where an aircraft factory puts a new aircraft is S-0404 claims 13 and 15,
   and its contact and tether at release are S-0417 claim 11.
   - Backing: `measurable` (scenario AC1)
   - Confidence: high

2. **Climbing.** In an ordinary movement tick with a living aircraft below its held height,
   height rises by min(*FL*ₕ − *h*, *c*) leptons, where *c* = 10 for an aircraft that carries
   passengers and 20 otherwise (Formula F1).
   - Backing: `measurable` (scenario AC2)
   - Confidence: high

3. **Descending.** In an ordinary movement tick with an aircraft above its held height,
   height falls by a step of ⌊(*h* − *FL*ₕ) / 20⌋ leptons, but at least 20 and at most 50
   (Formula F2). A dead aircraft that reaches this altitude adjustment also descends this way,
   but its resulting height is at least 1 lepton. For a living aircraft a result below 0 becomes 0. During a crash tick this ordinary altitude adjustment can follow
   the crash contribution of claim 9 if ordinary movement is still active. Its dead-aircraft
   floor does not prevent the earlier crash-impact test from removing the aircraft.
   - Backing: `measurable` (scenarios AC2 and AC5); the dead-aircraft floor is a conditional
     part of a moving crash tick, not evidence for a separate non-crashing dead state
   - Confidence: high

4. **Horizontal motion.** In each tick in which its locomotor is moving it, an aircraft moves
   along its body facing *F* by *s* = ⌊*V* × *f*⌋ leptons, where *V* is its speed in leptons
   per tick (the imported type speed, without country, veterancy or per-object speed bonuses)
   and *f* its current speed fraction. The displacement is split into x and y with the table trigonometry of S-0105 (F1),
   and each new coordinate is truncated toward zero (Formula F3). Ordinary forward motion
   follows its body facing; the table and coordinate truncations determine the exact step.
   A living descending aircraft with a destination additionally moves 5 leptons toward the
   centre of its destination cell in the same tick, except during *enter* toward a helipad
   building. This extra displacement follows the forward step and ordinary height reduction.
   It retains the resulting world height. Take the direction to that centre by S-0204
   claim 8, then use F3's facing conversion and table trigonometry
   with step length 5, truncating each coordinate sum toward zero. This is not a snap or a
   distance-clamped step: even a nearer cell centre does not reduce its length.
   Body turning follows S-0206 claim 1.
   When the new position is not in a legal map cell, the aircraft moves there anyway unless it
   may leave the map now (claim 10) and its type lacks the fly-by flag. In that case it tries the
   position shifted 128 leptons in x back toward the map's middle, then a random fallback
   position 64 leptons from that shifted point. One raw scenario-stream draw supplies its
   low eight bits as a facing index, multiplied by 256 facing units. F3's displacement and
   coordinate truncation apply, with length 64 and no snapping to a cell centre. If either
   resulting cell coordinate is outside 0 … 511, this random candidate is the shifted point
   itself instead. If neither candidate lies in a legal cell, no forward relocation occurs
   that tick; altitude adjustment and any descent nudge can still occur.
   - Backing: `measurable` (scenarios AC3 and AC23)
   - Confidence: high

5. **Speed fraction.** While a living aircraft that is not landing has a destination, its target
   speed fraction is *τ* = min(1, *d* / *S*), where *d* is the truncated table square root of
   the squared planar difference between its position and destination point (S-0204 claim 5),
   without a building-size subtraction, and *S* its type's slowdown distance (Formula F4). When *τ* < 0.1:
   - if *d* < 86 leptons, the current fraction is halved at once and *τ* = 0,
   - otherwise *τ* = 0.1.

   In ordinary slowing, the distance clamp applies after any halving: when *d* is less than
   the current fraction, that fraction becomes *d*. After this clamp, when *τ* = 0, the current
   fraction is 0 and *d* > 0, the current fraction becomes 0.05. An
   aircraft that is taking off keeps its target unchanged until its height reaches half its
   cruise height. At the end of every living movement tick the current fraction moves toward
   *τ* by at most 0.1. This is the ordinary fraction calculation, not the only approach control.
   After that adjustment, a living aircraft flying toward a destination, neither landing nor
   taking off, can get a replacement target fraction for its next movement tick. With the
   fixed-full-speed exception below inactive and no fly-by flag, the replacement is 0 below
   128 leptons, 0.5 from 128 up to but not including 512, and 0.75 from 512 up to but not
   including 768. At 768 leptons or farther, the ordinary target remains. Distances are raw planar point
   distances after any pad-position adjustment, not building-adjusted object distances.
   - Backing: `measurable` for resulting fractions and next-target replacement (AC3).
     The clamp's exact placement among the ordinary halving, creep and adjustment is
     `unmeasured`: with integer distance and fractions in 0 … 1, it changes only the
     zero-distance case, where several alternative intermediate orders give the same final
     zero fraction. F4 fixes the calculation but AC3 does not distinguish those equivalent
     intermediate orders.
   The target is instead fixed at 1 (no slowing down) while the aircraft carries the run lock,
   and while it holds its height (claim 6) and either its type has the fly-by flag or it is a
   strafer or fighter (claim 11) with ammunition left. This fixed-full-speed case does not apply
   the halving, distance clamp or creep restart. The ordinary 0.1 adjustment follows any
   applicable clamp and creep restart; the later approach target replacement changes only the
   next tick's target, not the already adjusted current fraction.
   - Confidence: high (formula and constants); medium (the full-speed conditions)

6. **Holding height and the landing trigger.** An aircraft is marked as holding height while
   any of these holds: its destination point is more than 120 leptons above terrain there,
   it has a target and ammunition left; it carries the run lock; its type is not landable.
   In ordinary flight its held height is the cruise height. Below 768 leptons from the destination,
   guidance instead uses destination height minus terrain height there when the holding-height
   mark is set. This applies to every marked approach, not just an originally elevated
   destination. However, accepting a nonempty movement point while the aircraft has a target
   and ammunition replaces that point's world height with terrain height there plus cruise
   height. Thus an attacker approaching a ground target ordinarily keeps cruise height,
   rather than descending to the ground. This replacement concerns the flight point held for
   movement, not the destination object's position. An *enter* movement update clears that mark. Without the mark, in a tick
   in which it is alive, neither landing nor taking off, standing in its destination cell with a
   target speed fraction of 0, and it has no target or no ammunition, it **starts to land**.
   When its behaviour is not *enter* and it has a further waypoint of a planned route (the
   waypoint planning of the order specs), it does not land but queues *wait* and goes on to that
   waypoint. A pad-bound aircraft whose cell
   holds no building it is in contact with does not land: it falls back to rest instead
   (claim 18), which sends it to a pad or makes it crash. A landing sets the held height to the
   landing height. There is also a close-approach trigger: below 128 leptons, with ordinary
   slowing permitted, no holding-height mark and current fraction strictly below 0.05,
   guidance requests landing without waiting for the exact-cell and zero-target-fraction test.
   Both triggers still obey the planned-route and pad-bound restrictions.
   - Backing: `measurable` (scenario AC4)
   - Confidence: high (the triggers); medium (the 120-lepton rule, the planned-route exception)

7. **Landing and touchdown.** Landing sets the target speed fraction to 0; descent follows
   claim 3. The ground render layer applies exactly when height above the surface is at most
   0 leptons; any positive height uses the top layer. Both directions use that boundary,
   with no hysteresis and no separate 208-lepton combat threshold.
   On changing into the ground render layer, its landing cell is tested (Formula F7).
   If the cell is refused the aircraft rises by 10 leptons and starts to take off again. The
   exceptions in F7 mean occupancy is not an unconditional refusal: its own contact, itself,
   and certain spawned-object combinations are accepted.
   - Backing: `measurable` (scenario AC4)
   - Confidence: high

8. **Take-off.** An aircraft on the ground (or landing) that gets a destination takes off: its
   held height becomes its cruise height and it climbs by claim 2. During take-off, with
   *H* = cruise height minus landing height, above *H* − trunc(*H* / 3) its aiming facing is
   requested to match its current body facing. Above trunc(*H* / 2), up to and including that
   upper threshold, its body is requested to face the destination and it requests full speed.
   At or below trunc(*H* / 2), take-off makes no new heading request: the previously requested
   heading persists, which need not equal the current facing. These are take-off rules, not descent stages. The
   divisions are integer divisions toward zero, with heights in leptons. Setting no destination while it is in flight
   makes it land where it is (claim 6). An aircraft standing on a pad leaves its pad reservation
   when it takes any destination other than that pad (claim 24).
   - Backing: `measurable` (scenario AC4)
   - Confidence: high

9. **Crash fall.** An aircraft crashes when it is above the ground and either it is dead or its
   locomotor is switched off (for example by an effect that disables vehicles). Each tick a
   fall counter *k* grows, by 1 for a dead aircraft and by 3 for a living one, and makes a
   downward contribution of *k* leptons (Formula F5). If that contribution brings height below
   1, impact occurs before ordinary movement or altitude adjustment can lift it again:
   on the ground, a dead aircraft's impact effects and removal follow S-0312 claim 16, and a living one
   is destroyed by S-0312 claim 16's crash-without-kill rule. If no impact occurred and
   ordinary movement remains active, horizontal motion and the ordinary altitude rules still
   apply. Thus the net drop need not be the counter alone; a moving dead aircraft also gets
   F2's descent, and a living aircraft can get F1's climb or F2's descent. An aircraft that is sinking in water instead sinks by 5 leptons per tick
   and is removed when its height is below −400 leptons (claim 37 gives its draws).
   - Backing: `measurable` for dead crash flight (AC5). The living switched-off crash and
     sinking branches are `unmeasured`: AC5 has no established normal-input effect that reaches
     them without killing immediately or using a different movement mode. Its exploratory
     variants cannot yet confirm those branches.
   - Confidence: high (the counter); medium (which switched-off states start a crash)

10. **Leaving the map.** An aircraft **may leave the map now** when all of these hold: it is a
    visiting aircraft (for an aircraft in a team, the team's own rule decides instead: out of
    scope); it has entered the usable area at least once since it appeared; it has no target, or
    its behaviour is *spy-plane overfly*; and its behaviour is neither *paradrop approach* nor
    *paradrop overfly*. Such an aircraft is removed from the game at once, without dying, when its
    position leaves the usable area and its type has neither the fly-by nor the fly-back flag, and
    in any case when its position leaves the map. An aircraft that may not leave the map now is
    never removed this way; a visiting aircraft that appears outside the usable area is therefore
    not removed by this eligibility test before it has flown in. Separately, an aircraft in
    *retreat* is removed as soon as its position is outside the usable area, irrespective of
    these conditions or the two flight flags. Thus *retreat* is not a way to wait outside the
    usable area for a first entry.
    - Backing: `measurable` (scenario AC6)
    - Confidence: medium (the entered-the-area condition)

### Attack runs

11. **Kinds of attacker.** An aircraft's run depends on two properties of its primary weapon and
    type:
    - it is a **strafer** when its primary weapon is a strafing weapon,
    - it is a **fighter** when its type is a fighter.

    A type can be both. The strafing rule takes precedence over the fighter run rule.
    - Backing: `measurable` (scenarios AC7 and AC8)
    - Confidence: high (the strafer test); medium (the invisible-projectile exclusion)

12. **The attack run.** An aircraft in *attack* follows table T1 (State and ordering). It flies
    toward a firing spot near its target, aims, fires, and then either makes further passes or
    leaves to rest. The firing spot is chosen anew at the start of every run (claim 39).
    For a strafer the firing spot is the target itself.
    In T1, **close enough** means that the range test of S-0307 claims 7 to 11 passes for
    the weapon selected against the current target, using that claim 8's firing source point.
    It is not distance to the firing spot and is not the last firing-check answer: an earlier
    refusal such as *not facing* does not decide whether this separate range test passes.
    A **full burst** is *B* shots (S-0301 claim 4) made one after another in the same tick, with no
    firing check between them; each shot still advances the burst counter and sets the rearm
    countdown as S-0301 claims 1 to 6 say, including any in-burst draw, so after a full burst the
    countdown holds the full delay.
    - Backing: `measurable` (scenarios AC7, AC8, AC9 and AC23)
    - Confidence: high

13. **Strafing run.** A strafer that reaches its target makes five firing passes in one run: the
    first pass fires its full burst, the next four fire one shot each. The passes after the first
    are processed one bare rate-of-fire field apart, in ticks, and the aircraft flies straight on
    toward the target during them. Each shot still sets the rearm countdown of S-0301 (claims 1
    to 3, 5 and 6), and a pass whose firing check answers *rearming* waits and retries every tick,
    so with no other refusal the spacing is the larger of the bare field and S-0301's delay. Later passes fire even when
    the check answers *not facing*, *cloaked* or *out of range* (claim 19). After the fifth shot the aircraft waits
    ⌊(*R* + 1024) / *V*ₜ⌋ ticks before it starts the next run, where *R* is the weapon's range in
    leptons and *V*ₜ the converted type speed (Formula F6, including its zero-speed departure). A strafer begins its firing passes as
    soon as the target is within its weapon's range; a strafer that has not yet reached that
    range keeps flying at the target. If the first pass's check answers *not facing*, the strafer
    goes back to choosing a firing spot and waits 45 ticks.
    - Backing: `measurable` (scenarios AC7 and AC23)
    - Confidence: high

14. **Fighter and ordinary runs.** A fighter fires its full burst in one pass and then starts the
    next run, choosing a new firing spot, while it has ammunition. Any other aircraft fires its
    full burst in one pass and one more single shot in the next pass. With the game-wide
    re-approach flag on, it then comes back to choose a new firing spot, which ends the run. With
    the flag off it fires its full burst again from where it is, then another single shot, and so
    on: the run never reaches a point that spends its round, so one round gives unlimited fire at
    S-0301's rate until the target is gone or aiming fails (`quirks.aircraft_endless_refire`).
    - Backing: `measurable` (scenario AC8, including its explicitly named test-map variants)
    - Confidence: high

15. **Ammunition spent once per run.** An aircraft spends exactly one round of ammunition per run
    in which it fired at least once, whatever its burst and however many passes the run had.
    The round is taken from the count at the next point where the run either starts its next
    approach or ends, or, if the aircraft leaves *attack* first, at the end of that aircraft's
    update, including a change that starts a queued behaviour in that update. Firing
    itself does not change the count. A run in which nothing was fired spends nothing. This
    replaces the per-shot spending of S-0301 claim 10 for aircraft.
    - Backing: `measurable` (scenarios AC7 and AC8)
    - Confidence: high

16. **Scatter.** Every firing pass sends a scatter request (S-0215 claim 19) to the occupants of
    the target's ground cell layer. Its threat point is the aircraft's current position.
    It is forced and not move-anywhere. The cell-wide forwarding gates of S-0209 claim 11
    still apply: the cell request need not reach every occupant, or be accepted.
    - Backing: `measurable` (scenarios AC9 and AC23)
    - Confidence: high

17. **End of the run.** A run ends when the aircraft has no target or no ammunition left. Then:
    - with no ammunition, a visiting aircraft or an aircraft of a human-controlled house forgets
      its target,
    - an airstrike aircraft that still has ammunition queues *retreat*,
    - every other aircraft falls back to rest (claim 18), which normally sends it to a pad.

    An aircraft that still has ammunition and a target when its run ends goes straight into its
    next run.
    - Backing: `measurable` (scenario AC8)
    - Confidence: high

18. **Falling back to rest (aircraft).** When one of the first three situations in table T2 (State and ordering) applies, the aircraft has that row's stated outcome and S-0215 claim 13's common effects do not occur. In all other cases the common effects apply, and the aircraft's next behaviour is the result specified by the applicable remaining situation in table T2. In flight, an armed aircraft
    with dock types looks for a pad (claim 21) and asks it for contact. When the pad grants it, it
    gets the pad as destination and queues *enter*. Otherwise a pad-bound aircraft crashes
    (claim 9), and any other aircraft gets a landing spot near its house's buildings (Formula F8)
    and queues *move*. On the ground it queues *guard*, or *area guard* for a computer-controlled
    house's aircraft outside a team. A visiting aircraft without passengers queues *retreat*.
    Whatever the table chose, an aircraft without ammunition that has no contact looks for a pad
    once more, and an aircraft that holds any contact queues *enter*. If the aircraft is ready to
    change (claim 31) the queued behaviour becomes current at once. This is the aircraft part
    that S-0215 claim 13 leaves to this spec.
    - Backing: `measurable` for human-owned, non-team rest outcomes (AC10). The original's
      computer-only ground choice and team-dependent rows of T2 are `unmeasured`: the setup
      cannot require those owners or teams to perform a specified rest transition. The restored
      overridden behaviour and the two other early-exit rows are also `unmeasured`: AC10 has
      not established normal inputs that isolate a fall-back invocation in those states.
      The computer and team cases are context, not extra rules for our AI houses.
      F8's other-object fallback is covered by claim 41 and AC23; its equal-distance tie
      order and complete building-footprint search remain open.
    The pad-search rows need the in-the-air answer (Terms); the other rows use the on-the-ground
    test.
    - Confidence: high (the in-flight and pad rows); medium (the ground rows and the computer
      house's choice)

19. **Firing check additions.** For an aircraft the firing check of S-0301 claim 9 has two more
    refusals after every general answer has been passed:
    - an aircraft marked as a passenger carrier (claim 1) that now carries none answers *out of
      ammunition*,
    - an aircraft that is not a fighter answers *not facing* when its aiming facing differs from
      the direction to the target by more than 2048 facing units (S-0204 claim 10); a fighter
      never answers *not facing*.

    These are the aircraft facing rule and the extra refusal of S-0310 claim 5. How the attack
    run reacts to each answer, including *rearming* and *cloaked* (S-0301 claim 12), is table T1.
    Cloaking may start only at height 0 or below; take-off alone does not break an already
    completed cloak (S-0516 claim 17). This height test is not the ground-render-layer
    alternative in this spec's on-the-ground test.
    - Backing: `measurable` (scenario AC11)
    - Confidence: high (the 2048 tolerance and fighter exemption); medium (the passenger-carrier
      refusal and cloak interaction)

20. **What a shot does.** A shot of an aircraft that carries passengers is a drop (S-0522 claim 11). For any
    other shot, the projectile starts as in S-0306, with two aircraft changes: a projectile whose
    type has turn rate 0 (a bomb) takes the aircraft's body heading, pitch and current ground
    speed instead of its own launch velocity, so it keeps flying forward as it falls; a projectile
    whose type has turn rate 1 is redirected toward the target in three dimensions, including
    the height difference, at the weapon's speed.
    For an aircraft of the currently viewing house, a successful shot can reveal an area around
    the aircraft when the aircraft, certain nearby samples or the target are under shroud.
    The radius is the game-wide attacking-aircraft sight range; the area reveal follows S-0409
    claim 17. This is not an unconditional reveal for every owner's shot. The exact set of
    nearby samples remains untraced here. A kamikaze slot
    aircraft (S-0506) is removed after its shot.
    - Backing: `measurable` (scenario AC12)
    - Confidence: medium; the exact nearby-sample predicate is `unmeasured` because its full
      geometry has not been independently traced

### Pads

21. **Choosing a pad.** A pad-bound aircraft first asks again the pad it was last heading for; if
    that pad still accepts it (claim 22) it keeps it. Otherwise the eligible candidates are its
    house's buildings of its dock types, not in limbo, with a free contact place and accepting it.
    Each dock type contributes one candidate. Within that type, house building-list order breaks
    ties; a building replaces the current candidate when its squared planar distance (F9) is
    strictly smaller, or it is marked primary. A primary candidate can therefore be replaced by
    a later, closer building. Among the resulting per-type candidates, dock-list order breaks
    ties and the same distance-or-primary preference applies. This is not one flat comparison
    of all buildings: a nearer building discarded in its own type's primary comparison does
    not compete against another type. A saved distance of −1 is treated as no choice on the
    next comparison, including when overflow produced that value. No random draw is taken.
    - Backing: `measurable` for candidate choice and draw absence (AC13 and AC22). The
      saved-distance −1 condition is `unmeasured`: no legal-coordinate example yielding that
      sentinel has been established; AC13's overflow comparison does not establish it.
    - Confidence: high (nearest, full pads skipped); medium (primary override and ties)

22. **When a pad accepts an aircraft.** A pad accepts an aircraft only when all of these hold:
    the pad's house is allied with the aircraft's house; the pad is not being built up or sold
    and has finished its build-up; it has a free contact place for the aircraft (one that is
    empty or already holds this aircraft); it is switched on (S-0405; low power does not
    matter); and a helipad building accepts only aircraft. A pad of a dead building accepts
    nothing.
    - Backing: `measurable` (scenario AC13)
    - Confidence: high

23. **Reservation.** Asking a pad for contact succeeds when the pad accepts the aircraft and has a
    free contact place, with the two houses mutually allied; the aircraft then holds the
    contact, so no other aircraft can be given that place. An aircraft that already holds another contact drops it first. The reservation
    lasts while the aircraft flies to the pad, lands on it and stays there.
    - Backing: `measurable` (scenario AC13)
    - Confidence: high

24. **Docking.** An aircraft in *enter* with a pad as destination flies to it and asks the pad
    each processing to be loaded, which the pad answers by confirming the contact and the
    approach. When it is over the pad it lands (claims 6 and 7), and on touchdown it is put at its
    contact place's docking position (S-0404 claim 15). It then reports that it has entered: the
    pad starts servicing (claim 26) and the aircraft's behaviour becomes *sleep*, with *guard*
    queued. A pad-bound aircraft that is on the ground but not on its destination pad leaves its
    current contact and takes off toward the pad. If the reservation is refused or lost while it
    approaches, the aircraft falls back to rest (claim 18).
    Entry begins with a 1-tick delay. While in contact during the approaching stage it returns
    3 ticks, including the processing that recognizes arrival and advances to docking.
    The docking-position and entered-report stage returns 1 tick. The unlinked retry and
    fallback paths also return 1 tick. None of these delays takes a standard-delay draw.
    - Backing: `measurable` (scenarios AC14 and AC23)
    - Confidence: high

25. **Landing-facing choice.** Where flight calls for the landing direction, an aircraft in
    contact uses its contact's current body facing rounded to 8 directions, not the direction
    toward the contact. With passengers but no contact it uses its own current body facing
    rounded to 8 directions. With neither it uses the game-wide pose-direction integer as an
    eight-direction index: the resulting facing is that integer times 8192, modulo 65536.
    This differs from the pose-direction conversion used for production placement (S-0404
    claim 13). The exact time this choice becomes the requested heading remains open.
    - Backing: `measurable` (scenario AC14); `unmeasured` for the time of application, which
      has not been traced
    - Confidence: high (the choice and conversion); low (its application timing)

26. **Servicing.** A pad that has one or more aircraft settled on it services them at a fixed
    cadence: every *P* = ⌊*r* × 900⌋ ticks, where *r* is the game-wide reload rate in game minutes
    (Formula F10). In each service step, service outcomes follow contact-place order. For example, if two aircraft need paid repair and the house can afford only one step, the earlier contact place is repaired and charged first; the later aircraft is then handled with the credits that remain:
    - an aircraft that is ready for take-off (no target, full ammunition) and has full health is
      released: it falls back to rest and its *guard* is forced,
    - an aircraft still flying in (behaviour *enter*) or still moving is skipped,
    - a settled aircraft whose behaviour is not *sleep* is told to queue *sleep*,
    - a settled aircraft in *sleep* gets one round when its ammunition is not full; otherwise it
      gets one repair step (claim 27); when neither applies, it is released as above. A target
    at or above the half-ammunition threshold is an exception: claim 28's acknowledgement
    adds no round but still counts as a successful reload, so that service step neither
    repairs nor releases the aircraft.

    When no aircraft was settled, the pad's servicing ends and the pad returns to *guard* until
    the next aircraft reports that it has entered (claim 24). One
    round per step means filling *A* missing rounds takes *A* service steps, followed by a
    further step to release an undamaged aircraft. The delay before the first step depends on
    when servicing begins; this is not a guaranteed touchdown-to-release duration.
    An accepted entered report sets the pad ready to change and queues servicing, without
    starting it synchronously or resetting its existing processing timer. At its next ready
    per-tick start, servicing becomes current with its timer expired, so its first processing
    can service immediately (S-0215 claims 3 and 11). It does not wait one reload period after
    the report. If the pad's turn has passed, this opportunity is in the next tick; otherwise
    it can be in the report tick. This describes a previously non-servicing pad, not a
    universal minimum interval across new arrival reports.
    - Backing: `measurable` (scenarios AC15 and AC23)
    - Confidence: high

27. **Repair on a pad.** A repair step costs the aircraft's house the repair cost of Formula F11
    and raises its health by the game-wide repair step (at least 1). When the result reaches full
    strength, health is set to exactly its strength and the aircraft is released in the same
    step. If the house cannot pay, the aircraft is released unrepaired. An aircraft whose health
    ratio is at least 1 gets no repair. The reload-pad service branch repairs without a separate repair flag. A building that
    also has the repair-building flag instead follows S-0314 claim 6's service branch.
    - Backing: `measurable` (scenario AC16)
    - Confidence: high (step and cost formula); medium (release when the house cannot pay)

28. **Taking off with half ammunition.** An aircraft on a pad that has a target is served only up
    to half its rounds: once its ammunition is at least ⌊*A* / 2⌋ the pad's reload request is
    confirmed without giving a round (`quirks.aircraft_half_reload_with_target`). In *guard* on
    the pad, an aircraft with a target stays while its ammunition is below ⌊*A* / 2⌋ and takes off
    into *attack* as soon as it reaches that. An aircraft of a human-controlled house gets a
    target on the pad only from an order.
    - Backing: `measurable` (scenario AC17)
    - Confidence: high

29. **Guard below cruise height.** An armed aircraft in *guard* below its cruise height that has
    less than full ammunition and no contact looks for a pad (claim 21); if it finds one it queues
    *enter* toward it and forgets its target. An aircraft in *guard* at cruise height with a
    destination queues *move*; an armed one without destination falls back to rest (claim 18),
    an unarmed one is given its own cell and *move*, so it lands. Other outcomes of *guard* on the
    ground use the standard delay, or 45 ticks for an unarmed aircraft and for an aircraft on the
    ground with no contact.
    - Backing: `measurable` (scenario AC10)
    - Confidence: medium

30. **Pad total and the build limit (free docks).** A house's pad total grows by a helipad
    building's docking-place count *d* when the building is placed, moves with the building
    when it changes owner, and shrinks by *d* when it is removed from the map (never below 0).
    The house-wide queue-capacity test refuses another pad-bound aircraft while
    *n* ≥ its pad total, where *n* is the sum, over every type in the pad-aircraft list, of the
    house's present count of aircraft of that type plus those being built and queued in its aircraft
    production line (Formula F12; S-0407 claim 16). “Present count” means aircraft on the map, as
    defined by S-0407; it does not mean the glossary's update-order sense of “in play.” This queue-time test does not check the
    type's own build limit (`quirks.dock_bound_aircraft_ignores_limit`, owned by S-0407 claim 16).
    Only types in the pad-aircraft list are counted, whatever type is asked for:
    `quirks.pad_limit_counts_list_only`. This is separate from the factory free-dock requirement
    of S-0404 claim 3 and the tech-tree build-limit test of S-0407 claim 9; it does not replace
    either. Sidebar icon presence and enabled state follow S-0411 claim 2.
    - Backing: `measurable` (scenario AC18)
    - Confidence: high (count rule and comparison); medium (owner change)

31. **Readiness and starting queued behaviours.** An aircraft is ready to change unless its
    current behaviour is *sticky* or *rescue*, it carries the run lock (except in *spy-plane
    approach*), or its processing has cleared its ready mark. Aircraft have no S-0215 claim 11
    per-tick start before their processing. Instead a queued behaviour of a ready aircraft
    becomes current at any of these three points:
    - in its locomotor's step, while alive and neither landing nor taking off (no height test),
    - at the end of an ordinary fall-back to rest (claim 18's early exits are exceptions),
    - once more each update tick, after the tick's processing and movement of that aircraft.

    A behaviour started at the last point is first processed on a later tick. The run lock is
    cleared in every update tick in which the behaviour is none of *attack*, *paradrop overfly*,
    *spy-plane approach* and *spy-plane overfly*.
    - Backing: `measurable` for ordinary flight, take-off, landing and run-lock queue timing
      (AC10). The *sticky*, *rescue* and processing-cleared-ready exclusions are `unmeasured`:
      no normal-input aircraft setup here isolates those exclusions from a run lock or another
      refusal. The *spy-plane approach* run-lock exception is also `unmeasured`: AC20 dispatches
      a plane but cannot require a queued behaviour change while that exception applies.
      AC10 does not infer these exclusions from a delayed order.
    - Confidence: medium (the order of the three points within a tick)

32. **Losing a pad or a target.** Destruction of a helipad destroys every aircraft in contact
    with it, even one still approaching at a distance; destruction of another building destroys
    contacts only within the distance threshold of S-0312 claim 17. A surviving approaching
    aircraft whose contact or destination is lost falls back to rest, looking for another pad.
    A pad-bound one crashes if none can be obtained. Sale and a refusal during approach must
    not be treated as equivalent to destruction. An aircraft in *attack* whose target is gone
    ends its run (claim 17). Before processing, an aircraft also clears a target whose
    in-the-air answer is yes, even when that target is still alive. A fully cloaked ground
    target is also cleared before processing when it is not allied with the aircraft and
    the aircraft's house does not sense it (S-0516 claim 25). An allied or sensed ground
    target is not cleared by this additional cloak test.
    - Backing: `measurable` (scenario AC19)
    - Confidence: high (destruction); medium (recovery after a lost contact and hidden-target
      clearing)

### Other behaviours

33. **Hunt and move.** In *hunt* an armed aircraft with ammunition picks a target (S-0309) and
    queues *attack* without an immediate start. It becomes current only at a permitted
    readiness-dependent opportunity of claim 31; the hunt processing does not itself start it.
    Without a target, or without ammunition, it falls back to rest. In *move* an
    aircraft flies to the flight point of its destination, and when its locomotor has stopped
    there it falls back to rest. *Move* uses the standard delay. *Hunt* returns a delay of
    1 tick after finding a target or failing to find one, and the standard delay only when
    it had no ammunition. When a living
    aircraft that is not pad-bound is in flight in *guard* with a destination and nothing queued,
    it queues *move*.
    - Backing: `measurable` for hunt queueing, delays and move completion (scenario AC10).
      Flight-destination substitution and the stopped answer are specified in claim 40 and
      tested by AC23; a zero speed fraction alone does not complete *move*.
    - Confidence: medium

34. **Retreat and the spy plane's flight.** In *retreat* an aircraft with no destination takes a
    cell on its house's arrival edge (S-0522 claim 5), selected as in S-0522 claim 4, as destination.
    It is removed when it leaves the usable area (claim 10).
    In *spy-plane approach* an aircraft flies at its target. When its ground distance
    (S-0204 claim 6) to the target is below 769 leptons it queues *spy-plane overfly*, holds the
    run lock and chooses an exit destination. With no target it also ends that processing with
    *spy-plane overfly* queued and the run lock held, not *retreat*.
    The spy plane's exit destination is selected as in S-0522 claim 4 on the edge opposite its
    house's computed edge (S-0522 claim 5): north and south are opposites, as are east and west.
    This uses the computed edge at destination choice, ignoring any map-section arrival edge.
    In *spy-plane overfly* an absent destination is replaced by a new choice on that exit edge.
    Approach processing has the game-wide spy-plane camera frame count as its delay; overfly
    and retreat processing each have a delay of 3 ticks. Queued changes follow claim 31.
    Dispatch, camera sight and reveal are S-0513 claims 43 to 45.
    - Backing: `measurable` for ordinary flight, exit destinations and cadence (AC20).
      The targetless approach is `unmeasured`: dispatch targets a map cell, and no normal-input
      way to remove that target during approach has been established. The map-section edge
      contrast inherits S-0522 claim 5's unresolved skirmish loading limitation
    - Confidence: high (targetless outcome, computed exit edge and delays); medium (flight)

35. **Carrier slot aircraft flight.** S-0506 claims 1 to 9 and 11 to 13 own the shared slot
    cycle; table T4 supplies only the preparing, attacking and returning flight of a carrier
    slot aircraft. Launch placement is requested 10 leptons above the spawner's firing point
    (S-0506 claim 7), but claim 1 can replace that height. At launch the aircraft gets *move*
    toward the neighbouring cell of the spawner's cell in step direction 0 (glossary).
    That is the initial launch destination, not a direction maintained throughout preparing.
    At each later launch pass in which the slot is already preparing and a controller target
    is present, its destination is replaced by the step-direction-4 neighbour of the spawner's
    cell at that pass, with *move* queued again. No arrival at the direction-0 neighbour is
    required. With no controller target, such a preparing slot instead returns (T4).
    Release into attacking follows S-0506 claim 8 and can occur in the launch pass itself.
    An aircraft released in that pass has no later preparing pass for that sortie and need
    not receive a direction-4 destination. Its attack runs follow claims 12 to 17.
    A returning aircraft is taken off the map only in the spawner's cell and with the signed integer world-height difference
    *z*ₐ − *z*ₛ < 20 leptons. This is not an absolute-distance test: there is no lower bound,
    so an aircraft below the spawner also satisfies the height condition. Reload and the
    subsequent full-health, full-ammunition docked state follow S-0506 claim 9. A slot aircraft
    may land in a cell that already holds another slot aircraft (Formula F7).
    A ship's launch movement hold is S-0504 claim 3: a positive launching count prevents the
    next step, not completion of a step already begun. Use S-0506's launching count, not its
    controller state: preparing non-missile aircraft do not by themselves raise that count.
    - Backing: `measurable` (scenario AC21)
    - Confidence: high

36. **Loss of a slot aircraft.** S-0506 claim 9 owns loss notification and regeneration.
    A notification for a slot aircraft with health below 1 or already released empties its slot
    immediately and starts its regeneration countdown; it need not wait for impact or final
    removal. Crash start can therefore free the slot while the old aircraft is still falling.
    A notification for a living, unreleased non-missile aircraft leaves its slot unchanged.
    Spawner destruction, ownership change and the other slot-clearing events follow S-0506
    claim 13; release of a non-missile aircraft through that rule starts its crash (S-0506
    claim 12), rather than letting it return to the carrier. For an ordinary aircraft not
    held by a magnetic beam, starting this crash sets health to 0: it is not a new living
    disabled-fall state. Crash start does not clear its unfinished flight (claim 40) or speed
    fraction. Ordinary movement can continue, combining F5's dead counter contribution
    with F2's dead descent in the same non-impact tick. Release does not impose a universal
    no-forward-motion fall. AC23 tests this distinction.
    - Backing: `measurable` (scenarios AC21 and AC23); `unmeasured` for loss notification of a living,
      unreleased aircraft, because no normal-input setup has been established that generates it
    - Confidence: high

37. **Random draws traced here.** The following draws use the scenario stream (S-0202 claim 10),
    with inclusive integer ranges as in S-0202 claim 4:
    - **Non-strafing firing spot.** When a non-strafer finds a firing-spot candidate, one
      *U*(0, 99) draw chooses between claim 39's best and preceding record candidate, split
      at 50. Strafers return the target without that draw; failure to find a candidate also
      takes none. This draw precedes that approach's standard-delay draw.
    - **Damage smoke.** In each update tick in which an aircraft's health ratio is below the
      game-wide condition-red ratio and its height is above 0, one draw *u* = *U*(0, 99) is taken.
      Smoke is shown when *u* < 10 for a living aircraft and *u* < 80 for a dead one. The draw is
      taken whether or not smoke is then shown, and whether or not the smoke animation can be
      created. This scenario-stream consumption is controlled by
      `quirks.aircraft_smoke_scenario_draw` (default on); its off alternative is below.
    - **Sinking splashes.** While an aircraft is sinking (claim 9), in each tick whose number is a
      multiple of 4, two draws *U*(−170, 170) are taken; the first is discarded and the second
      shifts the splash in x.
    - **Processing delays.** The standard delay of S-0215 claim 6 takes its own draw wherever
      claims 12, 29 and 33 use it.
    - **Edge cells.** A paradrop plane's arrival cell and a retreating aircraft's destination
      use S-0522 claim 4's no-reference edge-cell selection: one ranged draw, or two for south,
      with that claim's ranges and order. Spy-plane arrival uses the same selection (S-0513
      claim 43), as do the spy exit destinations of claim 34 on their computed opposite edge.
      Choosing the edge itself takes no draw (S-0522 claim 5).

    Flight-destination substitution takes one *U*(0, 7) per examined radius (claim 40).
    The failed edge push-back takes one raw draw only when its random fallback is attempted
    (claim 4). These are not an exhaustive flight draw inventory. Pad selection and the service steps
    themselves take no random draw. The crash tumble draws are S-0312 claim 16's.
    - Backing: `measurable` for smoke, firing-spot and processing-delay draws, and pad/service
      draw absence (AC22, with the permitted random-state recorder); edge-cell draws are
      `measurable` through AC20, AC22 and S-0522 scenario P3. Sinking splash draws are
      `unmeasured`: AC5 and AC22 have no established normal-input witness for aircraft sinking.
      The smoke switch-off alternative is `unmeasured`, a design rule of our engine
      rather than an outcome of the original
    - Confidence: high (the listed draws); low (the unresolved draw inventory)

38. **Boundaries.** This spec does not cover the paradrop and cargo delivery behaviours
    (*paradrop approach* and *paradrop overfly*: S-0522 claims 6 to 8). Ordinary air-transport
    eligibility and landed *unload* are S-0507 claims 2 and 13, not a future cargo-delivery
    rule; airborne unloading transitions remain incomplete there. This spec also excludes the spy
    plane's dispatch and reveal (S-0513 claims 43 to 45), missile flight (S-0506 claim 14), aircraft that lift vehicles
    (out of scope), or the original AI's choice of targets for computer aircraft (out of scope).
    An aircraft's cell-entry outcome is always *clear* in a skirmish (S-0209 claim 14); the landing
    test of Formula F7 is a separate test. The specialized drop-ship climb, descent and pitch
    variants and specialized target-seeking detonation flight are not specified here.
    Claims 2 to 8 cover ordinary flying aircraft.
    - Backing: `unmeasured` (pointer only)
    - Confidence: high

39. **Non-strafing firing spots (Q-0630).** Let *T* be the target's current position and
    *ρ* the firer's primary-weapon range of S-0307 claim 13. Candidate circles have radii
    *ρ* − 256, *ρ* − 512, … leptons, retaining only radii strictly above 256. Use the
    largest circle with any eligible candidate; smaller circles matter only when every larger
    one has none. Each circle's ordered candidates use facings 0, 4096, …, 61440, with
    F3's trigonometry and truncation applied to *T* and the radius. Convert each resulting
    planar point to a cell by integer division toward zero by 256. The destination is a cell,
    not the unsnapped point or a point retaining the target's world height.

    A candidate must be inside the height-aware usable area and pass the flight-reservation
    test below. There is no terrain-passability or F7 landing test at this choice. The
    reference point is the destination object's current position when the target is a mobile
    unit with a destination, otherwise the attacking aircraft's current position. Score each
    eligible unsnapped candidate by planar distance to that reference, using the table
    root and integer truncation of S-0204 claim 5. In candidate order, a **record candidate**
    is the first eligible candidate or one scoring strictly below all preceding eligible
    candidates. Equal scores do not replace a record. One scenario draw *U*(0, 99) chooses
    the last record's cell below 50, or the preceding record's cell otherwise. With only one
    record both outcomes use it. The preceding record is not necessarily the second-nearest
    eligible candidate. No target, no eligible circle, or failure of all its candidates gives
    no destination and no selection draw. Strafers bypass this search and return the target.

    **Flight-reservation test.** A pad-bound aircraft passes. A spawned aircraft also passes
    when the ground occupant selected as in F7 is spawned or has a spawn controller.
    Otherwise another placed, non-limbo mobile actor in that cell, or whose destination is
    that cell object, refuses it. This includes infantry, vehicles and aircraft, not just
    other flying aircraft. In firing-spot selection the searching aircraft itself also
    counts as a reservation. Vehicle-lifting aircraft's carried-vehicle exception is outside
    this spec's scope.
    - Backing: `measurable` (scenario AC23). The multi-occupant spawned exemption inherits
      F7's unresolved found-object choice, not a new nearest-object rule
    - Confidence: high for circles, records, draw and ordinary reservation test; medium for
      the spawned exemption

40. **Move destinations and the stopped answer (Q-0631).** Ordinary non-vehicle-lifting
    *move* first retains its destination when F7 accepts the cell at that object's current
    position and the flight-reservation test of claim 39 accepts the cell of its heading
    point. In this initial F7 use, its competing-aircraft destination comparison is against
    the original destination object, not always against a cell object. Here the reservation
    test excludes the searching aircraft itself. The heading
    point is S-0506 claim 14's destination heading point, evaluated with no requesting actor
    for this initial test. Otherwise, outside initial map loading, it seeks a replacement
    cell around the destination's current-position cell.

    Replacement candidates have integer radii 0 … 31 cells, in increasing order. Each radius
    takes one scenario *U*(0, 7) draw as its first step direction, then tries all eight
    clockwise directions, wrapping modulo eight. With that direction's facing of 8192 times
    its index, apply F3's trigonometry in cell units with length equal to the radius; truncate
    the coordinate sums toward zero. Candidates must be height-aware usable cells, pass F7,
    and pass the flight-reservation test excluding self. Consecutive duplicate usable
    candidates in the same radius are not tested again; the remembered initial pair is
    (0, 0). Choose the first accepted cell. Failure of all 32 radii, initial map loading,
    or an absent destination retains the original destination, without a replacement.
    These are eight samples per radius, not a scan of every cell of a square ring.

    After destination selection, flight takes that destination's heading point, now with the
    aircraft as requesting actor. For a cell this is its centre at its surface height; for
    a mobile object its next movement point or stationary position; for a building its
    centre or docking point as S-0506 claim 14 specifies. Claim 6 can replace the world
    height for an armed approach. Pad guidance can subsequently replace a building point
    with the contacting aircraft's docking position (S-0404 claim 15).

    For *move* completion and T1's ordinary-attacker approach, **stopped** is not the
    zero-current-fraction test. A movement point accepted by the locomotor remains an
    **unfinished flight** until landing has settled in its destination cell, or on its contact
    building. That completion clears the unfinished flight and destination. The locomotor
    reports stopped only with no unfinished flight and no positive forward pitch (the pitch
    comparison is single precision against 0). Thus a holding or non-landable aircraft
    which has accepted a movement point does not report stopped merely by hovering at that
    point with speed fraction 0. A freshly placed aircraft with no unfinished flight can
    nevertheless report stopped above terrain.
    - Backing: `measurable` (scenario AC23)
    - Confidence: high for destination selection, draws and the stopped test; medium for
      unusual object heading-point cases inherited from S-0506 claim 14

41. **Landing-search settings and fallback (Q-0634).** F8's search near a house building
    uses locomotion class *foot*, movement zone *normal*, the building cell's ground reach
    region for that zone, no deck mode, and a 3 × 3-cell free footprint. It allows bridges,
    asks for no height, no-overlay or burrowing condition, and has no reference cell. The
    aircraft type's own movement zone is not used. These are caller settings for S-0212
    claim 25; its complete 3 × 3 footprint qualification and result displacement remain
    open, so this is not yet a complete F8 destination rule.

    Below F8's 256-lepton weighted-distance threshold, the replacement search centred on
    the aircraft's own cell instead uses a 1 × 1 footprint, the same fixed class and zone,
    the own cell's ground reach region, and its own cell as reference. With no successful
    building choice, or no dock types, the fallback considers other own-house infantry,
    vehicles, aircraft and buildings, excluding self and objects in limbo. It prefers the
    smallest spatial table-root distance for which a nearby search succeeds. That search
    uses the other object's cell, a 1 × 1 footprint and otherwise the building-search settings.
    With no successful object choice, it returns the aircraft's own current cell without
    another free-cell search. No random draw belongs to these nearby searches. The exact
    tie order between equally distant fallback objects remains open.

    The edge half of Q-0634 is already answered by claim 34, claim 37 and S-0522 claims 4
    and 5: retreat uses the arrival edge, spy overfly the opposite computed edge, with one
    ranged draw except two on south. It is not a nearest-cell projection with no draws.
    - Backing: `measurable` (AC23) for caller settings, unequal-distance object choice and
      empty-house fallback; `unmeasured` for a complete 3 × 3 search outcome because its
      footprint predicate and result displacement have not been established here
    - Confidence: high for caller settings and empty-house fallback; medium for object choice

## Formulas

All heights and positions are integers in leptons (S-0204). "⌊ ⌋" on a quotient of integers is
division truncated toward zero.

> **F1 Climb.** *h*′ = *h* + min(*FL*ₕ − *h*, *c*), *c* = 10 with passengers, else 20. Integer.

> **F2 Descent.** *e* = *h* − *FL*ₕ; *δ* = min(50, max(20, ⌊*e* / 20⌋)); *h*′ = *h* − *δ*. For a dead
> aircraft reaching this adjustment after the crash-impact test, *h*′ = max(*h*′, 1). Then *h*′ = max(*h*′, 0). Integer.

> **F3 Horizontal step.** *s* = ⌊*V* × *f*⌋, *V* an integer speed in leptons per tick and *f* the
> current speed fraction (double precision), truncated toward zero. With *F* the body facing in
> 65536 units (S-0204 claim 10), let *F*ₛ = *F* for *F* < 32768, else *F* − 65536.
> *θ* = (*F*ₛ − 16383) × *k* radians in double precision, where
> *k* = −2π / 65534 rounded to double precision, that is −9.587672516830327 × 10⁻⁵. sin *θ* and cos *θ* are
> S-0105's table trigonometry (F1), single-precision table values.
> *x*′ = trunc(*x* + cos *θ* × *s*), *y*′ = trunc(*y* − sin *θ* × *s*), the sum formed in floating
> point and truncated toward zero once. Facing 16384 moves along +x, facing 0 along −y, both by
> exactly *s*. Because map positions are positive, the truncation of the sum acts as rounding the
> step down, so a step with a negative component can be one lepton longer in that axis than a
> step with the same positive component.

> **F4 Speed fraction.** This formula gives ordinary destination-dependent slowing under
> claim 5, not its fixed-full-speed exception. Let *f*₀ be the current fraction before slowing,
> *d* the integer raw planar point distance in leptons, and *S* the slowdown distance in leptons.
> *τ*₀ = min(1, *d* / *S*). For *τ*₀ < 0.1 and *d* < 86, *τ* = 0 and
> *f*ₕ = *f*₀ × 0.5. Otherwise *f*ₕ = *f*₀ and *τ* = max(0.1, *τ*₀).
> The distance-clamped fraction is *f*꜀ = min(*f*ₕ, *d*). The pre-adjustment fraction *f*ₚ
> is 0.05 when *τ* = 0, *f*꜀ = 0 and *d* > 0, and *f*꜀ otherwise.
> The adjusted fraction is min(*τ*, *f*ₚ + 0.1) when *f*ₚ < *τ*, max(*τ*, *f*ₚ − 0.1)
> when *f*ₚ > *τ*, and *f*ₚ when equal. Thus halving precedes the distance clamp, which
> precedes creep restart and the 0.1 adjustment. Claim 5's subsequent approach replacement
> sets the target for the next movement tick without recalculating this adjusted fraction.
> All fractions and their arithmetic use double precision with S-0205 claim 5's rounding
> mode; there is no integer conversion of a fraction.

> **F5 Crash contribution.** The fall counter starts at zero. On each crashing tick, increase it by 1 for a dead aircraft or by 3 for a living aircraft. The downward height contribution on that tick equals the updated counter: *h*₁ = *h* − *k*. Integer, leptons. Impact is tested against *h*₁ before any ordinary altitude adjustment. Without impact and with ordinary movement active, claims 2 and 3 give the remaining altitude change, using *h*₁ as their input height.

> **F6 End of a strafing run.** wait = ⌊(*R* + 1024) / *V*ₜ⌋ ticks, *R* the weapon range in leptons
> and *V*ₜ the converted integer type speed of S-0207 claim 1, the same value used as *V*
> in F3, not the raw imported number. Integer division truncates toward zero. There is no
> minimum divisor in the original: reaching this calculation with converted speed 0 stops
> it with a program error. A raw negative speed does not supply a negative divisor: import
> clamps it to 0, except the −1 keep-existing rule of S-0207 claim 1. Our engine follows
> S-0205 claim 3's departure: quotient 0, with an importer warning for a zero converted speed.
> A delay of 0 processes again next tick under S-0215 claim 2.

> **F7 Landing cell test.** A landing cell is accepted when the touchdown position is inside the
> usable area and either
> - no unit, infantry, aircraft or building is found in the cell, the cell is passable for
>   tracked ground movement (S-0211 claim 3) with no occupants, and no other aircraft in play
>   (not in limbo) has this cell object as its destination, or
> - the found object is the lander itself or its contact, or
> - the lander's type is spawned, and the found object either has a spawn controller or is
>   itself of a spawned type.
>
> The test searches the ground cell layer, not the bridge-deck list. With several occupants,
> the exact choice of the found object is unresolved; this revision's guaranteed outcomes
> concern cells with at most one such occupant. No nearest-to-touchdown rule is asserted.

> **F8 Landing spot without a pad.** When the aircraft has dock types, compare the house's
> buildings not in limbo for which a nearby free spot can be found (S-0212 claim 25). Distance
> is the truncated table square root of the spatial centre-to-centre squared difference,
> without building-size subtraction. For the first dock type divide that integer by 4,
> truncating toward zero. Choose the smallest resulting distance, retaining the earlier
> eligible building on a tie. Below 256 leptons seek a free cell near the aircraft's own cell.
> Otherwise use the free spot near the chosen building. Without dock types, or when no building
> yields a spot, use claim 41's other-object fallback and ultimately the aircraft's own cell.
> Claim 41 supplies caller settings; the building search's complete footprint semantics remain open.

> **F9 Pad distance.** *D*² = (*x*ₚ − *x*ₐ)² + (*y*ₚ − *y*ₐ)² between the centre of the pad and of the
> aircraft, in leptons, as a signed 32-bit integer that wraps on overflow (S-0205 claim 1); height is
> ignored. A candidate replaces the best so far when its *D*² is smaller (signed comparison)
> (`quirks.pad_distance_overflow`).

> **F10 Service cadence.** *P* = ⌊*r* × 900⌋ ticks, *r* the reload rate in game minutes read as a
> double (S-0003), the product in double precision truncated toward zero (S-0205,
> `quirks.chop_rounding`).

> **F11 Repair cost.** *q* = ⌊*Σ* / *σ*⌋ (integer), *Σ* the type's strength and *σ* the game-wide repair step as imported (no minimum here).
> *c*₀ = ⌊*C* / *q*⌋ (integer), *C* the type's cost; cost = max(1, ⌊*c*₀ × *p*⌋), *p* the repair
> percent as a fraction, the product in double precision and truncated toward zero. Credits.

> **F12 Pad build limit.** *n* = Σ over types *t* in the pad-aircraft list of (the house's present count of *t* +
> copies of *t* being built and queued in the house's aircraft line). Refuse when *n* ≥ pad total.
> “Present count” is the on-map count defined by S-0407. The sum and comparison are exact
> non-negative integer count arithmetic.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Climb step | 20 | leptons per tick | 10 when carrying passengers (F1) |
| Descent divisor | 20 | dimensionless | F2 |
| Descent step bounds | 20 … 50 | leptons per tick | F2 |
| Dead aircraft floor | 1 | lepton | F2 |
| Slow-speed floor | 0.1 | fraction | F4 |
| Creep distance | 86 | leptons | F4 |
| Creep fraction | 0.05 | fraction | F4 |
| Speed fraction step | 0.1 | per tick | F4 |
| Close-approach distance bands | 128, 512, 768 | leptons | claim 5 |
| Close-approach target fractions | 0, 0.5, 0.75 | fraction | claim 5 |
| Close-approach landing fraction | strictly below 0.05 | fraction | claim 6 |
| Hold-height threshold | 120 | leptons | claim 6 |
| Touchdown retry lift | 10 | leptons | claim 7 |
| Crash counter increments | 1 dead, 3 alive | leptons per tick² | F5 |
| Sinking rate, removal depth | 5, −400 | leptons per tick, leptons | claim 9 |
| Edge push-back | 128 | leptons | claim 4 |
| Edge random fallback radius | 64 | leptons | claim 4 |
| Descent horizontal nudge | 5 | leptons per descending movement tick | claim 4 |
| Flight-destination search radii | 0 … 31 | cells | claim 40 |
| Firing-spot radius interval | 256 | leptons | claim 39 |
| Firing-spot angular interval | 4096 | facing units | claim 39 |
| In-the-air height | 208 | leptons | Terms (S-0307) |
| Strafing passes | 5 | passes per run | full burst, then four single shots (claim 13) |
| Strafe run-out extra | 1024 | leptons | F6 |
| Strafe aiming retry | 45 | ticks | claim 13 |
| Ordinary aiming retry | 1 | tick | table T1 |
| Aircraft aiming tolerance | 2048 | facing units of 65536 | claim 19 |
| Approach turn-in distance | 512 | leptons | table T1 |
| Firing spot reached | 16 | leptons (distance < 16) | table T1 |
| Spy-plane overfly distance | 769 | leptons (distance < 769) | claim 34 |
| Ticks per game minute | 900 | ticks | F10 (S-0215 claim 6) |
| Unarmed guard recheck | 45 | ticks | claim 29 |
| Landing spot near own cell | 256 | leptons | F8 |
| Dock-type distance weight | 1/4 | dimensionless | F8 |
| Slot return height bound | 20 | leptons, signed world-height difference | strictly below, no lower bound (claim 35) |
| Requested slot launch height | 10 | leptons above the firing point | placement may replace it (claim 35) |
| Smoke chance | 10 alive, 80 dead | out of 100 | claim 37 |
| Splash shift range | −170 … 170 | leptons | claim 37 |
| Repair refusal ratio | 1 | health ratio | claim 27 |

## State and ordering

**T1. Attack run.** The delay is the number of ticks before the next processing of *attack*.
"Ammunition left" counts the round of the run still to be spent.

| State | Event | Result |
|---|---|---|
| start | has a target | *choosing*; 1 tick |
| start | no target | *done*; 1 tick |
| *choosing* | run lock off; the pending round is spent; target and ammunition left | the firing spot becomes the destination; *approaching* when it exists, else *done*; standard delay |
| *choosing* | no target or no ammunition | *done*; standard delay |
| *approaching* | pending round spent, run lock off; no target or no ammunition | *done*; standard delay |
| *approaching* | strafer, target at or beyond weapon range | keep flying at the destination (row "steer") |
| *approaching* | strafer, target within range; or fighter; or other aircraft whose locomotor has stopped | *firing* |
| *approaching* (steer) | no destination | *choosing* |
| *approaching* (steer) | distance to destination ≥ 512 | turn toward the destination; 1 tick |
| *approaching* (steer) | distance < 512 | turn toward the target; if distance ≥ 16, 1 tick; else the destination is cleared and *firing* |
| *firing* | no target or no ammunition | *done* |
| *firing* | non-strafer turns body to the target; the firing check answers *ready* | full burst fired, a round becomes pending, scatter. Strafer: *strafe pass 2*, run lock on, one rate of fire. Otherwise fighter: run lock on, *choosing* (or *done* with no ammunition), one rate of fire. Other: *refiring*, 1 tick |
| *firing* | answer *not facing* | no ammunition: *done*. Strafer or not close enough: *choosing*. Else fighter: *firing*; other: *choosing* with the re-approach flag on, *firing* with it off. 45 ticks for a strafer, else 1 |
| *firing* | answer *rearming* | 1 tick |
| *firing* | answer *cloaked* | uncloak; 1 tick |
| *firing* | any other answer | no ammunition: *done*; strafer: 1 tick; other: *refiring* |
| *refiring* | answer *ready* | one shot, scatter; ammunition left: *choosing* (flag on) or *firing* (flag off); else *done*; standard delay |
| *refiring* | answer *cloaked* or *rearming* | uncloak when cloaked; standard delay |
| *refiring* | any other answer | no ammunition: *done*; otherwise not close enough: *choosing*; otherwise *choosing* with the re-approach flag on, *firing* with it off. A strafer's *not facing* answer always chooses again and waits 45 ticks; all other cases use the standard delay |
| *strafe pass 2, 3, 4* | answer *ready*, *not facing*, *cloaked* or *out of range* | one shot at the target (out of range: the target becomes the destination first), scatter, the target becomes the destination, next pass, one rate of fire |
| *strafe pass 5* | same answers | one shot, scatter, *approaching*; F6 ticks |
| any strafe pass | no target | *done*; 1 tick |
| any strafe pass | other refusal | no ammunition: *done*, run lock off; otherwise retry same pass; 1 tick |
| *done* | — | run lock off, pending round spent (if any ammunition), then claim 17; 1 tick |

**T2. Fall back to rest (aircraft).** Where more than one situation applies, the result is the one in the earlier table row. "In flight" and "on the ground" are this spec's tests (Terms).

| Situation | Next behaviour |
|---|---|
| its current behaviour has been overridden | the overridden one is restored; a restored *patrol* starts from its first stage |
| current behaviour is *retreat*, *paradrop approach*, *paradrop overfly*, *spy-plane approach* or *spy-plane overfly*, and it is not an airstrike aircraft | unchanged |
| queued behaviour is *paradrop approach* or *spy-plane approach* | that behaviour starts now |
| on the ground, visiting, with passengers | *unload* (in a team: *guard*) |
| on the ground, visiting, no passengers, not in a team | forgets target and destination; *retreat* |
| on the ground, otherwise | forgets target and destination; *guard* (*area guard* for an armed aircraft of a computer-controlled house outside a team) |
| in flight, with passengers | destination: landing spot F8; visiting: *unload* (in a team: *guard*); else *move* |
| in flight, unarmed, in a team | unchanged |
| in flight, armed, visiting, no ammunition | forgets target and destination; *retreat* |
| in flight, armed, visiting, ammunition left, not in a team | *hunt* |
| in flight, armed, and either *attack* is queued, or *attack* is current with ammunition left and a target | *attack* |
| in flight, armed, and the in-the-air answer is no, or it is already flying to its *enter* destination, or it has no dock types, or it is outside the usable area in a team | *guard* |
| in flight, armed, a pad is found and grants contact | destination: the pad; *enter* |
| in flight, armed, no pad, pad-bound | crash |
| in flight, otherwise | destination: landing spot F8; *move* |
| after the row: no ammunition, armed, no contact | a new pad search; found: destination the pad, target forgotten, *enter*; none and pad-bound: crash |
| after the row: holds a contact | *enter* |

**T3. Pad servicing.** See claim 26. Aircraft events on the pad:

| Aircraft state | Event | Result |
|---|---|---|
| flying to the pad (*enter*) | pad accepts, contact held | lands, put on its docking position; *sleep*, *guard* queued |
| flying to the pad | pad refuses or is gone | falls back to rest |
| settled in *sleep* | service step, ammunition not full | +1 round (with a target: none once at ⌊*A*/2⌋) |
| settled in *sleep* | service step, full, damaged, house can pay, no target | repair step |
| settled in *sleep* | service step, target held, ammunition at least ⌊*A*/2⌋ | no round, no repair, no release |
| settled in *sleep* | service step, nothing to do, or cannot pay | released: fall back to rest, *guard* forced |
| *guard* on the pad, target, ammunition ≥ ⌊*A*/2⌋ | its processing | *attack*; takes off; the reservation ends |

**T4. Carrier slot aircraft flight.** Events are evaluated at S-0506 claim 2's launch passes.
The state in the first column is the slot's state entering that pass. Thus the launch row and
an already-preparing row do not both apply to the same slot in its launch pass. The shared
release decision of S-0506 claim 8 can still make a newly launched or already-preparing slot
attacking in that same pass; it does not require either neighbour to have been reached.

| Slot state | Event | Next state | Flight effect |
|---|---|---|---|
| docked | launches under S-0506 claim 6 | preparing, until shared release | initial destination is the spawner-cell neighbour in step direction 0; queue *move* |
| preparing | controller target present | preparing, until shared release | replace destination with the current spawner-cell neighbour in step direction 4; queue *move* again |
| preparing | no controller target | returning | clear target, spawner as destination, *move* |
| attacking | ammunition left and controller target present | attacking | receive controller target and queue *attack* again |
| attacking | no ammunition or no controller target | returning | clear target, spawner as destination, *move* |
| returning | ammunition left and controller target present | attacking | receive controller target and queue *attack* |
| returning | otherwise, same cell as spawner and signed world-height difference < 20 leptons | reloading | taken off the map, reload countdown begins (S-0506 claim 9) |
| returning | otherwise | returning | clear target, spawner as destination, *move* again |

**Ordering.**
- A queued behaviour started at the end-of-update opportunity can already cause the pending
  attack-run round to be spent in that same update. The smoke draw follows that opportunity.
  An aircraft removed by the edge test takes no later smoke draw in that update. These
  consequences do not prescribe the internal arrangement of processing and movement.
- In a service step, aircraft are dealt with in contact-place order, one step each.
- A round of a run is spent no earlier than the next approach after it was fired, so an aircraft
  that fired its last round still counts it until that point; the firing check never refuses for
  a pending round.

## Edge cases and quirks

- **Half reload with a target** (`quirks.aircraft_half_reload_with_target`, default on). An aircraft
  on a pad with a target is never reloaded past ⌊*A* / 2⌋ rounds while it keeps the target; the pad
  acknowledges the reload without adding a round, repairing or releasing it in that step. Fixed behaviour: reload to full
  regardless of target, and take off at full ammunition.
- **Queue-capacity test skips the build limit** (`quirks.dock_bound_aircraft_ignores_limit`,
  default on, S-0407 claim 16). For a pad-bound type this queue-time test does not check its own
  build limit. Off: that test applies both limits. Other production-legality checks remain.
- **Pad limit counts the list only** (`quirks.pad_limit_counts_list_only`, default on). The count *n*
  includes only types in the pad-aircraft list, so a pad-bound type outside the list is limited by
  the list types' numbers and not by its own. Fixed: also count the asked-for type.
- **Pad distance overflow** (`quirks.pad_distance_overflow`, default on). The squared distance of F9
  wraps beyond about 46 340 leptons, so a very distant pad can look nearer than a close one.
  Fixed: compare exact squared distances.
- **Repair step of 0 or above strength.** When the repair step is 0, or larger than the type's
  strength, a divisor in F11 is 0 and the original stops with a program error. Our engine uses
  S-0314 claim 5's existing safe departure: use at least 1 for the repair step in the cost
  calculation and at least 1 for the resulting number of steps, with an importer warning.
- **A pad with no docking-place key** has one contact place and adds the absent default of
  the docking-place count to the pad total (Import mapping); a key set to 0 also gives one
  contact place but adds 0.
- **Endless ordinary refire** (`quirks.aircraft_endless_refire`, default on). With the
  re-approach flag off, an ordinary non-fighter, non-strafer repeats burst-plus-single-shot
  cycles without spending its pending round. Off: end each such cycle as one run, spending
  its round and selecting a new approach regardless of that flag.
- **Damage smoke consumes game-state randomness** (`quirks.aircraft_smoke_scenario_draw`,
  default on = original). Claim 37's damage-smoke chance uses the scenario stream, including
  updates that show no smoke. This can change later game-state random outcomes, so it is not
  a presentation-only bug. Off: disable this damage-smoke emission entirely. Take no
  damage-smoke chance draw on either stream and create no damage-smoke animation, even when
  claim 37's eligibility holds. Other aircraft draws and effects are unchanged. The default
  retains the higher smoke chance for dead aircraft. This alternative is a design rule,
  not an oracle measurement.

## Import mapping

| Key | File, section | Concept | Parse form | Absent |
|---|---|---|---|---|
| `Landable` | rules, aircraft type | landable | boolean | no |
| `AirportBound` | rules, aircraft type | pad-bound | boolean | no |
| `Fighter` | rules, aircraft type | fighter | boolean | no |
| `FlyBy` | rules, aircraft type | may leave the map (claim 10) | boolean | no |
| `FlyBack` | rules, aircraft type | may leave the map (claim 10) | boolean | no |
| `Dock` | rules, aircraft type | dock types, in list order | list of building types | empty |
| `FlightLevel` | rules, type | cruise height | integer, leptons | the game-wide value |
| `FlightLevel` | rules, `[General]` | game-wide cruise height | integer, leptons | 500 |
| `SlowdownDistance` | rules, type | slowdown distance *S* | integer, leptons | 500 |
| `Speed` | rules, type | converted integer *V* and *V*ₜ, including F6's zero-divisor warning | S-0207 claim 1; not the raw value in F6 | S-0207 claim 1 |
| none | — | layer boundary, entry delays, service start, search and edge constants | fixed engine rules, not new import fields | — |
| `Ammo` | rules, type | maximum ammunition *A* | integer | −1 (unlimited) |
| `Selectable` | rules, object type | visiting test | boolean | yes |
| `Strength`, `Cost` | rules, type | F11 | integer | S-0302, S-0403 |
| `Spawned` | rules, type | spawned type (F7) | boolean | no |
| `InitialAmmo` | rules, type | starting ammunition | S-0301 claim 10 | −1 (use maximum) |
| `Spawns`, `SpawnsNumber`, `SpawnRegenRate`, `SpawnReloadRate` | rules, type | S-0506 Import mapping | — | S-0506 |
| `Camera` | rules, weapon | camera weapon (visiting test) | boolean | no |
| `Burst`, `ROF`, `Range`, `Speed`, `Projectile` | rules, weapon | S-0301, S-0306 | — | those specs |
| `ROT` | rules, projectile | strafing test (at most 1), bomb rule (exactly 0), aimed redirection (exactly 1) | signed integer | S-0306 |
| `Inviso` | rules, projectile | strafing test | boolean | S-0306 |
| `Helipad` | rules, building type | helipad building; adds to pad total | boolean | no |
| `UnitReload` | rules, building type | reload pad (services aircraft) | boolean | no |
| `UnitRepair` | rules, building type | repair building: accepts aircraft for repair (S-0314) | boolean | no |
| `NumberOfDocks` | rules, building type | docking-place count *d* | integer | 1 |
| `DockingOffset0` … | rules, building type | S-0404 claim 15 | — | S-0404 |
| `V3RocketType`, `DMislType` | rules, `[General]` | the two types whose in-the-air answer is their locomotor's (Terms) | type name | none |
| `PadAircraft` | rules, `[General]` | pad-aircraft list | list of aircraft types | empty |
| `ReloadRate` | rules, `[General]` | reload rate *r* | double, game minutes | 0.05 |
| `RepairStep` | rules, `[General]` | repair step | integer, hit points | 5 |
| `RepairPercent` | rules, `[General]` | repair percent *p* | double; a percent sign divides by 100 (S-0003) | 0.25 |
| `CurleyShuffle` | rules, `[General]` | re-approach flag | boolean | no |
| `AttackingAircraftSightRange` | rules, `[General]` | reveal radius on firing | integer, cells | 5 |
| `SpyPlaneCameraFrames` | rules, `[AudioVisual]` | spy-plane approach delay | integer, ticks | 16 |
| `ConditionRed` | rules, `[AudioVisual]` | smoke threshold | double, ratio | 0.5 |
| `PoseDir` | rules, `[AudioVisual]` | landing direction without contact | S-0404 Import mapping | S-0404 |

The aircraft-type flags are read only for aircraft types. An aircraft type's `FlightLevel` that is
absent stays unset (−1), which means "use the game-wide value", not a height of −1.

## Measurement scenarios

**Common setup and falsification rule.** Use a purpose-made skirmish map, game speed 6,
800 × 600, stock data, director off and mods off. Include one bystander computer player at the
farthest start, isolated from the observed area; apply the glossary's bystander void rule.
Run each variant five times unless stated otherwise. Use scripted normal input and a read-only
per-tick recorder for positions, heights, facings, current and queued behaviours, ammunition,
health, credits, contacts, current and target speed fractions, held height and shot events.
The observed area is the test aircraft's route, its targets and every candidate pad or landing
cell named by the variant; keep the bystander outside that area. Scripted normal input can drive the game and a read-only recorder can collect these logs. No game-state writes, patched executable or injected instrumentation are allowed.

Where stock types or normal orders cannot expose a branch, the scenario explicitly names a
**test-map override**. These are the glossary's named-test-file allowance, not stock-data claims.
remove the overrides after that variant. Never infer an untested branch from an ordinary run.
Each expected result below is falsifiable: an isolated record that differs in its stated count,
tick, value, destination, behaviour or survival outcome falsifies that part. An unreachable setup
is a coverage gap, not confirmation. Record actual seeds and tick boundaries. Unresolved parts
listed in Open questions are not claimed confirmed by these scenarios.

- **AC1 Placement (claim 1).** Produce a pad aircraft; dispatch a paradrop plane and spy plane.
  observe a carrier launch. Record the position immediately at placement, before movement.
  Expected: the existing visiting mark and usable-area test select ground versus cruise height
  by claim 1, not the requested launch height; speed fraction is 1 only at cruise height.
  Also place a fresh type on the test map without a dispatch mark: the type's later visiting
  classification must not retroactively change its placement height. A named placement test-map
  override supplies selectable/landable/camera combinations if no stock object distinguishes
  those branches. Record the retained passenger-carrier mark after troops leave (AC11).
- **AC2 Vertical flight (claims 2 and 3).** Order take-off, long flight and landing of a pad
  aircraft over level ground. Record every height change with held height and living state.
  Expected: F1's climb, F2's descent including the final clamp, not a constant descent rate.
  Repeat with passengers; if stock dispatch cannot produce a low starting height, the named
  loaded-flight test-map variant supplies a landable, selectable transport placed loaded.
  Its climb step is 10 instead of 20 leptons. Observe killed aircraft separately in AC5.
  no reachable non-crashing dead-aircraft descent setup is established here.
- **AC3 Forward motion and slowing (claims 4 and 5).** Fly straight routes in all four map-axis
  directions, then diagonal routes, logging type speed, fraction and coordinates. Repeat with
  named speed-bonus test-map overrides for country, veterancy and per-object speed bonuses:
  expected F3 uses the unchanged type speed in every case, not a bonus-scaled speed.
  Compute F3 with the signed facing and compare every forward step. Use sustained cruise ticks
  to separate the forward formula from claim 4's additional descent step. On approach to a cell destination, log distance
  and fractions around each F4 threshold, including distance 0, and around 128, 512 and 768
  leptons. Expected: F4's halving, then distance clamp, then creep restart, then the 0.1
  adjustment, followed by claim 5's target replacement for the next movement tick. At *d* = 0
  in ordinary slowing with positive incoming fraction, the adjusted fraction must be 0.
  Record the pre-slowing and adjusted fractions separately; a result differing from F4
  falsifies the resulting-fraction rule, not every observationally equivalent intermediate
  order. Distinguish current from target fractions; a target replacement must
  not retroactively alter the already adjusted fraction. Compare take-off below and above
  half height. Repeat during a strafing run, fighter run and visiting fly-by: the fixed-full-speed
  cases must not follow ordinary slowing, including no distance clamp at a zero-distance
  tick reached with the run lock held. An unreachable zero-distance example is recorded as a
  coverage gap, not inferred from another distance.
  An edge-flight variant records the push-back attempt
  and a refused tick; AC23 isolates the random fallback and its draw.
- **AC4 Landing, take-off and refusal (claims 6 to 8).** Start with an empty tracked-passable
  destination, then a single vehicle occupant, then an impassable destination. Use the named
  landing-test map override to give an ordinary controllable aircraft landable but not pad-bound
  status. Expected: empty cell accepted; refused touchdown rises 10 leptons and resumes take-off.
  Repeat with the landable flag removed: no landing. Test a pad-bound aircraft with a free pad,
  all pads full, and no pad: ordinary ground cannot serve as its rest place. Record contact loss
  when ordering departure from a pad. A loaded version distinguishes passenger climb speed.
  Give a planned route with a further waypoint: no intermediate landing outside *enter*.
  Log target-present/ammunition-present and elevated-destination cases: they hold height.
  take-off steering follows claim 8, not the deleted descent-stage hypothesis. Log the held
  height on crossing the 768-lepton boundary with an elevated destination; it adopts the
  destination's terrain-relative height while marked as holding height. Below 128 leptons,
  an unmarked slow approach can request landing before the ordinary exact-cell trigger.
  For F7, compare cells containing just the lander, its contact, another spawned object, a
  non-spawned object with a spawn controller, and an ordinary object without one. Use the named
  spawned-landing test-map override for otherwise unreachable type combinations. With the lander
  spawned, the first four are accepted; the last is refused. A second aircraft whose destination
  is the otherwise empty cell blocks landing. Repeat with one blocker on a bridge deck and an otherwise empty
  passable ground layer: F7's occupant lookup must ignore the deck blocker. Multi-occupant
  selection remains open.
- **AC5 Crash and sinking (claim 9).** Shoot down an aircraft at cruise height over land and water.
  log height and horizontal coordinates each tick, death, impact and removal. Expected: falling
  counter increments of 1 while dead. Compare the net height with F5 followed by F2 when movement
  remains active, not with triangular-number drops alone. Forward motion continues only while
  ordinary movement remains active. The living and sinking branches are exploratory only and
  remain unmeasured. A proposed named disable-flight test-map variant must first identify a
  normal-input disabling effect that leaves the aircraft alive in a crash: if found, expect
  increments of 3 combined with ordinary altitude adjustment. Immediate killing does not
  cover that branch. A proposed named sinking-aircraft variant must first demonstrate aircraft
  sinking without changing movement mode: if found, expect 5 leptons lower each update and
  removal only below −400. A different drop or removal threshold would falsify the respective
  hypothesis; failure to reach either state leaves the coverage gap.
- **AC6 Exit eligibility (claim 10).** Follow dispatched visiting aircraft from outside the usable
  area through entry and departure. Record visiting mark, entered-area state, target, behaviour,
  the two flight flags and removal. Expected: no eligibility-based removal before first entry.
  after entry, target-free visitors without either flag leave at the usable edge, other eligible
  visitors at the legal-map edge. A non-visiting controllable aircraft must not disappear solely
  for this reason. Use the named exit-flags test-map override to vary the flags independently.
  Separately place an aircraft outside the usable area in *retreat*: expect immediate removal
  from the retreat rule even without the entered-area state. Exclude team-controlled variants.
- **AC7 Strafing (claims 11 to 15).** A strafer attacks a stationary durable target. Log every
  shot, weapon burst position, rearm countdown, behaviour stage, ammunition and destination.
  Expected: full burst in one tick followed by four single-shot passes, no immediate ammunition
  spend, later spacing no shorter than either the bare rate or the current firing delay, and F6
  delay after the last pass. Interrupt after an early pass: one pending round is spent at the
  end-of-update boundary, not per projectile. Move the target out of range during later passes:
  *out of range* does not suppress them. Named strafing-test overrides supply a burst above 1,
  a simultaneous fighter flag, and an invisible projectile comparison. Strafer takes precedence
  over fighter; invisible projectile or turn rate above 1 removes the strafe classification.
- **AC8 Fighter and ordinary runs (claims 11 to 15 and 17).** Record the same events for a
  non-strafing fighter and a non-strafing non-fighter. Use the named ordinary-run test-map
  override for a controllable non-fighter with several rounds; repeat with the re-approach flag
  on and off. Expected: fighter, one full burst per run; ordinary with flag on, full burst then
  one shot, followed by spending one round; flag off, alternating burst and single shot without
  spending the pending round while the run continues. End the target or interrupt the order:
  the pending round must then be spent once. With no ammunition, a human-owned aircraft clears
  its target and rests; a visiting airstrike with ammunition and no target retreats. In-burst
  draws and the final rearm delay must match S-0301 claims 2 and 4, not one check per burst shot.
- **AC9 Scatter (claim 16).** Have infantry in the target cell of a durable target under a strafing
  run. Record scatter requests and accepted destinations at each firing pass. Expected: a
  request each pass; actual movement is subject to S-0215 claim 19 and may be refused. A shot
  without a corresponding request falsifies the claim; lack of movement alone does not.
- **AC10 Rest, guard, hunt and queue timing (claims 18, 29, 31 and 33).** Exhaust ammunition with
  a free pad, then full pads, then no pads. Expected: reservation and *enter* when available.
  pad-bound crash otherwise; a non-pad-bound aircraft seeks a landing spot by F8. Named rest-test
  overrides supply non-pad-bound landable aircraft with dock lists, without dock lists, with and
  without a weapon, with and without passengers. Record fallback's target, destination, current
  and queued behaviours and compare every applicable row of T2. Test already-overridden and
  retained special behaviours separately: those early exits must skip the common rest effects.
  If no normal-input method is found for an overridden or retained-special transition, mark
  that row unmeasured rather than counting ordinary fallback as its test.
  Observe guard at cruise height and below it, and hunt both with and without ammunition.
  Expected hunt delay: 1 tick except the no-ammunition standard delay; guard and move use the
  delays stated in claims 29 and 33. For a successful hunt, use a named hunt-test map placing an
  armed, ammunition-bearing aircraft in *hunt* with an eligible ground target. Record current
  and queued behaviours immediately after hunt processing and at claim 31's later start
  opportunities, along with readiness, landing and take-off state. Expected: *hunt* remains
  current and *attack* is queued after that processing; *attack* becomes current only at a
  permitted ready opportunity, and is not processed again in that update. A ready aircraft
  may therefore end the same tick in *attack*, which alone does not establish an immediate
  start by hunt. A start within hunt processing, a start while not ready, or a start outside
  claim 31's opportunities falsifies this distinction. An end-of-tick-only record cannot
  distinguish queueing from an immediate start and does not confirm it.
  Use a named guard-test override if a stock aircraft cannot
  reach a required state by orders. Queue a different order during a run lock, ordinary flight,
  take-off and landing. Log when it becomes current and first processes: no change through the
  run lock, the ordinary living non-transition movement opportunity has no height restriction,
  and an end-of-update start cannot process earlier in that update. Record the human-owned
  outcomes for our AI-house rule; any original computer-only choice is labelled separately.
- **AC11 Firing refusals (claim 19).** Attack with a non-fighter crossing a ground target at
  signed facing differences just below, at and above 2048, in both senses. Record aiming facing
  and the firing-check answer before the behaviour turns. Expected: extra *not facing* only
  above 2048. Repeat with a fighter: that extra refusal never occurs. The named facing-test
  override supplies controllable non-fighters and a passenger carrier with a weapon. Place it
  loaded, unload every passenger, then attempt fire: expect *out of ammunition* from the retained
  carrier mark. A fresh aircraft of the same type placed empty must not get that extra refusal.
  For claim 19's cloak interaction (S-0516 claim 17), a named test-map override
  supplies a controllable aircraft with a cloak ability. Compare cloak starts at height 0
  and above 0, then take off after full cloak without firing or losing its cloak source.
  Expected: no airborne start, and no loss of a completed cloak solely from take-off.
  Either contrary outcome falsifies that part; record cloak state and height each tick.
- **AC12 Projectile launch and reveal (claim 20).** Record the initial projectile velocity of
  a bomb released by an aircraft flying straight, and a turn-rate-1 shot at targets above and
  below the release height. Expected: bomb velocity follows body heading, pitch and effective
  forward speed; the turn-rate-1 velocity aims in three dimensions at weapon speed, not
  horizontally. Compare initial velocity, not impact displacement alone, because later gravity
  and flight rules are S-0306's. The named launch-test override supplies these weapons where
  stock use cannot separate them. Observe explored-cell changes on firing near shroud from an
  aircraft of the viewing house, separating ordinary sight refresh from firing reveal. Record
  the imported attack-reveal radius. A passenger shot instead releases cargo; a kamikaze shot
  removes its aircraft. Those type variants use the named launch-test override.
- **AC13 Pad choice, acceptance and reservation (claims 21 to 23).** Give a returning aircraft
  two compatible pads; fill one, then free it. Expected: only an eligible free pad is selected,
  and the contact reserves the place before arrival so a second aircraft cannot claim it.
  Repeat with equal distances and varied building-list order, then primary status. With two
  dock types use three pads: a nearer pad displaced by a later primary within its own type,
  plus a second-type pad at an intermediate distance. Expected: compare the two per-type
  winners, not all three flatly. This distinguishes claim 21's two-level rule. For overflow,
  place a very distant pad on a sufficiently large legal map and compare signed F9, rather
  than the exact squared distance. The named pad-choice test-map override supplies the extra
  compatible types and contacts. Test sale/build-up, switched-off, low-power and allied versus
  enemy acceptance separately: low power alone does not refuse; sale/build-up, off state,
  enemy ownership and full contacts do. Keep a pad-bound aircraft's already accepted heading pad
  while a nearer pad becomes free: the original reservation choice is retained.
- **AC14 Docking and facing (claims 24 and 25).** Record approach, touchdown, docking position,
  contact, *sleep* and queued *guard*. Expected: position is S-0404 claim 15's pad position.
  contact persists until departure. For direction, compare the contact's body facing with the
  direction toward it; approach from different sides. Expected choice: rounded contact facing,
  not approach bearing. Named landing-facing test-map overrides vary the pose-direction integer
  and give a passenger aircraft with no contact. The choices are pose integer times 8192 modulo
  65536, or its own rounded body facing with passengers. Record the chosen heading as well as
  final facing: application timing remains an open question, not an asserted touchdown deadline.
- **AC15 Reload cadence (claim 26).** Land an undamaged empty aircraft with no target. Record the
  first and subsequent service ticks, rounds and release. Expected: successive service steps
  *P* ticks apart, one round per step, release on the step after full ammunition; do not assume
  that the first service step is *P* ticks after touchdown. Use multiple contact places and
  aircraft to record place-order servicing and skipping an aircraft still in *enter* or moving.
  The named service-test override supplies several rounds and independently damaged aircraft.
- **AC16 Pad repair (claim 27).** Land a full-ammunition damaged aircraft without a target.
  record health and credits at each service tick. Expected: F11 cost and the repair step, exact
  full-health cap and release when full; insufficient credits cause release with no charge or
  healing. Repeat on a reload pad without the separate repair-building flag. The named
  repair-test override varies strength versus repair step to distinguish integer divisions
  from floating division. Invalid zero-divisor inputs may stop the original; our engine uses
  the already specified S-0314 claim 5 departure and does not reproduce that stop.
- **AC17 Half reload with a target (claim 28).** Issue an attack order while an aircraft reloads.
  The named half-reload test-map override gives several rounds, including separate odd and even
  maxima so the half threshold is not degenerate. Record target retention, round count and
  take-off tick. Expected: held below trunc(*A*/2), attack allowed at the threshold. While
  *sleep* still holds with a target at or above that count, a service acknowledgement must
  neither add ammunition nor repair, charge money or release it. Compare clearing the target:
  full reload and repair then proceed normally.
- **AC18 Capacity (claim 30).** Keep factory free-dock eligibility satisfied while queueing and
  producing aircraft. Record helipad counts, imported docking counts, present aircraft and
  the in-production and queued counts of every pad-list type. Expected refusal at F12's
  threshold, including items in progress and queue. Sell, remove and capture a helipad and
  repeat for old and new owners. Named capacity-test overrides distinguish a pad-bound type
  absent from the list, its own build limit, and docking counts absent versus explicitly zero.
  The last two contribute 1 and 0 respectively to capacity but both have one contact place.
  Record queue refusal separately from icon presence, tech-tree refusal and a full factory.
- **AC19 Pad and target loss (claim 32).** Destroy a helipad with one aircraft docked and one
  reserved but far away in approach. Expected: both aircraft are destroyed by S-0312 claim 17's
  contact damage, not retargeted to another pad. In separate runs sell a pad or lose a contact
  without destroying the aircraft; survivors seek another pad or crash if pad-bound and none
  is obtainable. Record target destruction during a run: the run ends and a pending round is
  spent once. Use the named airborne-target test-map override to give the attacker a legal
  anti-air weapon, then assign an airborne target: expect the target to clear before processing
  despite its being alive and otherwise attackable. These outcomes distinguish contact destruction
  from simple destination loss and target removal from target invalidation.
  For the hidden-ground-target variant, use the named cloak-target test-map override to
  supply a durable cloaking ground target, allied and enemy ownership variants, and a
  sensor that can be moved into and out of coverage. Acquire the target before it cloaks.
  Record full-cloak completion, alliance, the attacker's house's sensor coverage and target
  retention immediately before processing. Expected: a fully cloaked enemy target clears
  without coverage, but the cloak test alone does not clear an allied or sensed ground
  target. Clearing before full cloak, retaining an unsensed fully cloaked enemy, or clearing
  solely for cloak while allied or sensed falsifies the respective rule. Keep the target
  alive, on the ground and otherwise valid to isolate this test from the airborne rule.
- **AC20 Retreat and spy flight (claim 34).** Dispatch a spy plane and record ground target
  distance, behaviour, run lock, edge destination, processing ticks and exit. Expected: overfly
  begins below 769 leptons, approach cadence is the imported camera-frame count, and retreat
  without a destination chooses the owner's arrival edge (S-0522 claim 5). Compare starts
  near each edge and a later change of primary building factory: each new retreat destination
  uses the then-current arrival edge; a spy exit uses the opposite of the then-current computed
  edge, not a remembered launch edge. Use S-0522 scenario P3's edge-cell predictions and
  random-state records to test each destination and its one ranged draw (two for south).
  Record overfly and retreat processing 3 ticks apart and compare removal with AC6's retreat
  exception. A named spy-flight test map with an aircraft initially in retreat and no
  destination tests destination choice. Targetless spy approach remains unmeasured: dispatch
  targets a cell, so a test cannot assume that destroying an object deletes that cell. If a
  normal-input witness is established, expect overfly queued and run lock held, not retreat.
  Otherwise report a coverage gap. A named map-section edge variant contrasts an explicit
  arrival edge with a different computed edge only if loading that edge for the player house
  is established; it must not be inferred from ordinary dispatch.
- **AC21 Carrier aircraft (claims 35 and 36).** Attack with a carrier and record each slot's
  launch request, actual position, destination changes, slot states entering and leaving each
  launch pass, controller target, return, off-map interval, health and ammunition. Keep a
  durable target in range and use several slots so an early aircraft remains preparing across
  passes while later slots wait for launch spacing. Expected: its launch pass assigns the
  direction-0 neighbour; the next pass it enters already preparing with a controller target
  replaces that destination with the direction-4 neighbour, regardless of arrival at the first
  neighbour. Each further preparing pass with a target refreshes the direction-4 destination
  using the spawner's cell at that pass. Move the spawner to distinguish a refreshed destination
  from a retained old cell. A direction-4 assignment at launch instead of direction 0, or a
  retained direction-0 destination after a later targeted preparing pass, falsifies this rule.
  Separately use the named single-slot carrier test-map override: release in the launch pass
  must allow attacking without an intervening direction-4 assignment. Neither a second
  destination nor arrival at either neighbour is a universal release requirement. Observe the
  aircraft's own attack runs, return when empty or target-free, and pickup only in the spawner
  cell with signed world-height difference
  strictly below 20 leptons. Record world heights separately: a difference of 20 must not pass,
  while 19 does. If normal flight and carrier motion can produce a negative difference,
  it must pass too; failure to reach that variant is a coverage gap, not confirmation of an
  absolute-height test. Preparing without a controller target must return rather than continue
  moving to the direction-4 neighbour. Expect reload completion at the next controller pass
  at or after the spawner's reload countdown expires. Relaunch
  with full health and ammunition. Restore a target during an ammunition-bearing return:
  expect attack resumed, not unconditional continued return. Kill one aircraft: its slot's
  regeneration countdown starts at its crash-start loss notification, not impact, and
  replacement waits for a controller pass at or after that countdown expires. Destroy or
  change ownership of the carrier during a sortie: its linked aircraft start to crash under
  S-0506 claims 12 and 13 rather than return. A notification while a non-missile aircraft is
  living and unreleased is not claimed covered by this test.
  Issue a move order to the carrier both between steps and during a step, recording its
  launching count separately from controller state and preparing aircraft. Expect S-0504
  claim 3's next-step hold only while that count is positive, with a current step completing.
  Ordinary preparing non-missile aircraft alone must not cause that hold. A positive-count
  variant needs a missile-spawning ship; record that separately rather than inferring it from
  the carrier's controller being in its launching state.
- **AC22 Random draws (claim 37).** Use the permitted random-state recorder in addition to the
  per-tick records. Compare a healthy aircraft with a damaged one below the smoke threshold
  at height 0 and above it, and the same aircraft during crash. Expected above 0: one
  *U*(0, 99) smoke draw per surviving update, with thresholds 10 living and 80 dead; height 0
  takes no such draw. Observe 200 eligible updates for each smoke-rate variant; the exact
  recorder comparison, not a statistical smoke frequency, decides the draw claim. Accumulate
  dead-aircraft updates across repeated crashes rather than expecting one crash to last 200 ticks.
  The proposed sinking variant is exploratory and unmeasured until AC5 establishes a witness:
  expected ticks divisible by 4 take two *U*(−170, 170) draws before smoke, and only the second
  moves the splash in x. A different draw count, order or splash shift would falsify that part. Compare a strafer's target selection with a non-strafer
  finding a firing spot: only the latter takes the additional *U*(0, 99) selection draw, before
  its standard-delay draw. Separate projectile, creation and firing-delay draws through S-0202
  claim 10 and S-0301 claim 18. Repeat AC13's pad searches and AC15's service steps with this
  recorder: neither event itself advances the stream, after accounting separately for the
  aircraft's smoke and processing-delay draws. Any additional attributed draw falsifies that
  absence claim. Record paradrop and spy arrivals and AC20's retreat and spy exit destination
  choices separately: expect S-0522 claim 4's ranged draws and cell, with two draws for south
  and one otherwise. Edge choice itself takes none. A different attributed count, range,
  order or selected cell falsifies that rule; separate aircraft creation and smoke draws.
  For a tick with no visible smoke, verify that the same eligible smoke chance draw is still
  consumed. An eligible update without that draw falsifies the default rule even though its
  screen image is unchanged. The quirk-off absence of damage smoke and its chance draw is an
  engine test, not an expected outcome to test on the original.

- **AC23 Aircraft-lane discriminators (claims 1, 4, 6 to 8, 12, 13, 16, 24, 26,
  36 and 39 to 41).** Use the common five-run skirmish setup, scripted normal input and
  read-only per-tick and random-state recorders. Named test map `aircraft-lane` supplies
  non-strafing fighter and ordinary aircraft, landable non-pad-bound variants, a non-landable
  variant, durable stationary ground targets, a carrier and pads. Remove all overrides
  afterwards. The observed area includes every candidate cell and own-house fallback object.
  Record intermediate movement points separately from destination objects and current position.
  Each mismatching point, flag, draw count, processing tick or outcome below falsifies that
  prediction; an unreachable boundary is a coverage gap, not a passing test.
  - Attack from several directions with two distinct non-strafing primary ranges. Compute
    claim 39's circle candidates, distances and successive record cells from the logged
    target and aircraft positions. Expected: the first eligible circle, best cell for draws
    below 50 and previous record cell otherwise, even when that is not the second nearest.
    Move a mobile target toward a new destination to distinguish its destination-position
    reference from its current position and the attacker's position. Block selected cells
    with mobile actors or their cell destinations; a pad-bound variant bypasses that
    reservation. A short-range named variant with no radius above 256 must fail without a
    selection draw. Separate this from the following standard-delay draw. At a facing
    refusal compare the target range test with distance to the firing spot: T1 follows the
    former, even though the firing check's answer is not *out of range*.
  - Move to empty cells, occupied cells and moving objects. Record the retained object or
    substituted cell and the point actually accepted for movement. Expected: claim 40's
    heading point, eight-sample circles, one direction draw per attempted radius, and the
    original object on exhaustion. A non-landable variant and an ammunition-bearing attacker
    can hover with current fraction 0 while unfinished flight still prevents a stopped
    answer. The landed ordinary variant clears unfinished flight; forward pitch must be
    nonpositive for stopped. Record the completion tick, not just absence of displacement.
  - Approach ground targets from beyond 768 leptons: the accepted movement point is raised
    to terrain plus cruise height before close guidance adopts its height. Compare a
    non-attacking elevated destination and an ordinary ground move. Record requested body
    and aiming headings through both take-off thresholds. At or below half height no new
    take-off heading request is made; do not substitute current facing for a retained request.
  - Record every descending position after subtracting the ordinary forward step. Except
    *enter* toward a helipad, expect the additional length-5 step toward the destination-cell
    centre, including a near-centre case that can overshoot. At the map edge isolate failure
    of the 128-lepton shift: one raw draw and its low-eight-bit direction select the length-64
    candidate, or no forward relocation when that is illegal. Compare the 0 … 511 guard.
  - Record height and render layer on take-off and landing: positive surface-relative height
    uses top, nonpositive uses ground. Log a carrier placement request and actual placement
    before movement: the requested positive height is discarded for non-visiting placement
    inside the usable area. Log a produced pad aircraft separately at initial placement and
    final docking assignment, preserving S-0404 claim 15's final docking height.
  - Land empty aircraft at an initially non-servicing pad, on both sides of its turn in the
    tick when normal timing permits. Record the entered report, queued and current pad
    behaviours, pad turn, service effects and subsequent cadence. Expected: servicing starts
    at the next per-tick start and processes without a new reload-period wait; an already
    passed pad turn defers it to the next tick. Aircraft entry processing has initial delay
    1, contacted approach delay 3 and docking/report delay 1, with no standard-delay draws.
  - For F6 use positive raw speeds whose converted values differ from them; the post-fifth-pass
    delay divides by converted speed. A named zero-speed variant already in firing range
    attempts all five passes; if it reaches the quotient, the original stops with a program
    error. Test raw negative and −1 separately against S-0207 claim 1's import rule. Our
    engine's zero-delay warning departure is an engine test, not an original-game outcome.
  - At each attack pass record the cell scatter inputs and delivered requests: threat at the
    aircraft position, forced, not move-anywhere, ground layer. Verify forwarding against
    S-0209 claim 11; no movement alone does not falsify a sent request.
  - Destroy or change ownership of a carrier while one ordinary slot aircraft is moving.
    Expected: release-started crash sets health to 0 and retains unfinished flight and its
    fraction, allowing forward motion plus ordinary dead descent until impact. Repeat with
    a settled slot if normal play can produce it; release is not a blanket movement stop.
  - For F8, compare no house buildings, no other own object, and separate unequal-distance
    own infantry/vehicle/aircraft anchors with no dock types. Keep the short-game setting off
    and the bystander alive. Expected with no qualifying anchor: own cell without search.
    Otherwise expect the nearest anchor yielding a successful 1 × 1 search under claim 41's fixed
    foot/normal settings, with no search draw. Compare zones and water to discriminate from
    using the aircraft's own movement settings. The 3 × 3-building search and equal-distance
    fallback ties remain incomplete and are not confirmed by those 1 × 1 runs.

## Open questions

1. Q-0630 is answered by claims 12 and 39; no firing-spot geometry question remains.
2. Q-0631 is answered by claims 33 and 40; the stopped test is distinct from speed fraction.
3. The planned-route exception of claim 6 belongs with the order specs' waypoint planning; this
   spec states only that it stops a landing.
4. The exact application timing of the landing-facing choice (claim 25) remains unresolved.
   Do not infer height stages from the take-off rules.
5. Q-0634 remains partial: S-0212 claim 25 needs the 3 × 3-footprint qualification and
   result displacement for claim 41's building search. Equal-distance other-object fallback
   tie order also remains unresolved. Unequal-distance and empty-house fallbacks are specified.
6. Q-0633 is answered by claim 4.
7. The landing test's selection among multiple ground-layer occupants remains unresolved
   (F7). Single-occupant outcomes are specified.
8. Climb and descent variants for drop-ship aircraft types (a separate type flag) are not covered.
9. Which states switch off an aircraft's locomotor and start a crash (claim 9).
10. Claim 40 answers the ordinary flight-destination search draws. A complete inventory
    across specialized flight outside this spec remains open; edge cells are S-0522 claim 4.
11. The two game-wide rocket types whose in-the-air answer comes from their locomotor (Terms) are
    missiles in stock use (S-0506); whether any aircraft of those types reaches this spec's rules
    was not checked.
