---
id: S-0501
title: Aircraft flight, attack runs and pads
topic: movement
revision: 45
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-24
measurements: []
requests: [Q-0630, Q-0631, Q-0632, Q-0633, Q-0634, Q-0635, Q-0636, Q-0650, Q-0651, Q-0653, Q-0654, Q-0841, Q-0872, Q-1755, Q-1756, Q-1775, Q-1776, Q-1837, Q-1838, Q-1845]
supersedes: []
---

# S-0501: Aircraft flight, attack runs and pads

## Summary

How fixed-wing and rotor aircraft fly: their height, horizontal speed, landing and take-off; how
an armed aircraft carries out an attack run and spends its ammunition; how it finds, reserves,
lands on and is serviced at a pad; how the number of pad-bound aircraft a house may build follows
from its pads; and the flight and return of non-missile aircraft launched from a carrier.
Every run that does not go straight into another run gives the aircraft a destination on its
house's map edge, with that choice's draws (claim 17). What an armed aircraft with no dock types
then does depends on its house and on whether it must return to a pad (claim 45); an
aircraft that must return to a pad and has none to go to crashes (claim 46).
The shared spawn-controller cycle, including replacement, reload and release, belongs to
S-0506 claims 1 to 9 and 11 to 13; missile flight also belongs to S-0506. The spy plane's dispatch and reveal are S-0513 claims 43 to 45 and the
delivery of paratroopers and cargo is S-0522 claims 1 to 16; only their flight is covered here.
Ordinary air-transport eligibility is S-0507 claim 2: capacity counts passengers, without a
passenger-size or size-limit test. Boarding releases mind control (S-0507 claim 6).
Passenger release and airborne unloading transitions are S-0507 claim 13, including dock
search, stopped polling and replacement around map waypoint 700. That claim uses this spec's
claim 21 for dock selection, claim 41 for its landing-spot fallback and claim 40 for the
replacement-cell search; entering an unload phase is not itself a descent command.
Aircraft cloaking and hidden-target loss are S-0516 claims 17 and 25.
Requests answered here, besides those named in claim headings: Q-1756 in full, by claims 40 and
45. Its questions 1 and 2 are claim 45's: no fixed delay before the descent after a run: the delay is the length
of the flight from the firing spot to the replacement cell, which AC29 part b's per-tick positions
record, and no landing at the firing spot: a human-controlled aircraft lands
on a replacement cell near its house's computed edge, and a computer-controlled one never
rests, except in claim 45's north-edge case; and the same edge destination when the target is lost before the
aircraft fires (AC29 part d). Its question 3 is claim 40's: the first processing of every
ordinary *move* applies claim 40's selection to its destination, whether or not the aircraft
already flies to it, and the aircraft's own earlier choice does not count against it, so the
destination is kept when F7 and the flight-reservation test accept it and replaced otherwise
(AC29 part b, the case after a run). Q-1755 in part, by claim 32 and AC19's Q-1755 parts:
(1) an aircraft does not keep a target that boards a transport or goes into a building: it loses
it at that moment, in that tick, and its run ends by claim 17 at the processing after its next
one, with no round spent; a tank bunker keeps the vehicle on the map, so the target is kept
(medium). (2) The target is not kept, so no hold, no new firing spots and no rounds spent; the
re-approach flag plays no part. (3) The run does not resume when the object comes back. (4) A
ground attacker loses the target by the same rule (Changes for other specs, S-0309); what it does
next is S-0215's and S-0309's. Q-1755's harvester dock and the suicide shot inside a burst stay
open (Open questions 23 and 24).
Q-1837 in full, by claim 45: an aircraft's *area guard* processing compares its height (Terms),
taken at the start of the processing, with its cruise height, not with its held height. Only
equality gives the fall-back to rest with a delay of 1 tick; above its cruise height, as below
it, the processing is the spent-ammunition fall-back, the held-target *attack* with 1 tick, or
S-0215 claim 26's rows with *k* = 2 (AC29 part j). Q-1838 in full, by claim 1 and Terms: (1) a
type that is not landable or not selectable, with no visiting mark before placement, is placed on
the ground inside the usable area; (2) its first fall-back is the placement fall-back, made
during placement before the type test marks it, so it takes T2's ground row for an aircraft that
is not visiting: target and destination forgotten, *guard* (*area guard* for an armed aircraft of
a computer-controlled house outside a team), and resting there makes no later fall-back by
itself; (3) claim 34 holds for any *retreat* of an aircraft that is not a strike aircraft, but
this one is not in *retreat* after placement, so orders and a carrier's launch-pass *move* and
*attack* reach it; (4) AC29 parts g and h stand, with the starting heights stated there. (1) is
rated high (AC1), (2) to (4) medium (AC29 parts g (4) and h).
Q-1845 in full, by claim 47: (1) an aircraft becomes a tracked flying object when its take-off
starts, on the ground at the height it has then, normally 0 leptons, or when it is placed on the
map with its in-the-air answer *yes* and the counts-as-aircraft flag, which an aircraft type has
unless its data turns it off; placement on the ground, from a factory, by the map or by a carrier's
launch inside the usable area, does not track it, and leaving a pad is a take-off; (2) it stops
being tracked at touchdown, not when its landing starts, on a pad as on open ground, at a crash
impact, and when it leaves the map in any way, going inside a carrier, transport or building
included; a dead aircraft stays tracked while it falls; (3) a
landed aircraft is never tracked; (4) its sector is brought up to date at the start of each
movement update in which it has changed cell and its speed fraction carried from the previous
update is not zero; it moves to the end of the new sector's list only when the sector changes, so
the sector follows its position one movement update late and not at all while it stands still.
(5) aircraft that start being tracked in one tick join in the order of the events that start
their tracking, with no sorting; (6) hovering, waiting, circling and attacking in flight are
ordinary flight and tracked, and no temporary effect that holds an aircraft in flight ends its
tracking: one that switches its locomotor off makes it crash (claim 9), and it stays tracked until
the impact.
Against the committed text, these rows of tables T1 and T2 change. In T1: the *approaching* row
with no target or no ammunition goes to *done* after 1 tick, not after the standard delay (claim
32, AC19 parts a and j); the spend of a pending round with the run lock turned off is stated
above the table as coming first at every processing of *approaching*, whatever it then finds, a
strafer's after its fifth pass included (the committed wording put it among that row's
conditions); the *firing* row with no target or no ammunition now states its delay, 1 tick; and a
*refiring* row with no target, going to *done* after 1 tick, is new. In T2: the in-flight, armed
*guard* row keeps the destination and names *area guard* for an armed aircraft of a
computer-controlled house outside a team (claim 18); the in-flight, pad-bound crash row points to
claim 46; the after-the-row entry for a pad-bound aircraft whose new pad search finds none no
longer reads only "crash": the row's next behaviour is not given and the current behaviour stays,
and the aircraft crashes in flight (claim 46) but not on the ground (claim 9); and a paragraph
after T2 states that the after-the-row entries follow every row from the fourth on, the visiting
rows included, so a pad-bound visiting aircraft in flight with no ammunition and no contact
crashes when its pad search finds none, instead of taking its row's *retreat*.
Revision 38 changes no behaviour relative to revision 37: it names the landing rules that decide claim 45's north-edge fall-back cell, says that claim 35 covers the carrier slot aircraft claim 45 excludes, and moves the answers to Q-1756 and Q-1755 from the open questions into this Summary.
Revision 39 changes no behaviour relative to revision 38: it adds claim 37's target-loss draw to AC29 parts c and g (2), states claim 45's computer case by its result, says in the Summary that the aircraft does not land at its firing spot, and says why Open question 16 leaves Q-1756's answer unaffected.
Against revision 39, revision 40 changes no behaviour: the Summary lists every row of tables T1
and T2 that changes against the committed text, gives the delay before claim 45's descent as the
flight's length with a pointer to AC29 part b's per-tick positions, and AC29 part c says the type
is made pad-bound by the test rules.
Against revision 40, revision 41 states one behaviour that revision 40 left open: an aircraft's
*area guard* processing above its cruise height follows the branch stated for below it (claim
45). It also states that the placement fall-back comes before the type test that marks a
visiting aircraft (Terms, claim 1), and adds AC29 parts g (4) and j and a placement record to
part h.
Against revision 41, revision 45 adds claim 47 on when an aircraft is a tracked flying object, its
quirk switch, the counts-as-aircraft flag in Import mapping, and scenario AC30.
Shared pad-capacity rules are S-0407 claim 16 and claim 30 here; repair-step rules are S-0314
claims 4 and 5 and claims 26 and 27 here. S-0504 claim 3 and S-0506 claim 10 own the
launching-count movement hold: it prevents the next step, not completion of an active step,
and preparing non-missile aircraft alone do not impose it (claim 35 here).

Unless an alternative is explicitly stated, the behaviour and oracle scenarios below use every
quirk switch in its default, original-behaviour state.

This spec covers skirmish; network games and campaigns are out of scope. Player houses controlled
by our AI use the human-controlled rules below. Computer-only behaviour described here records
the original only; the neutral and special houses retain it (glossary, *AI house*).

## Terms

- **aircraft** (glossary): in this spec, an object of an aircraft type that moves with the flying
  locomotor. Rocket-driven missiles are S-0506.
- **aircraft deck mark**: the aircraft's own standing-on-deck state (claim 43), distinct
  from its render layer, its presence in a cell's occupant list and that cell's occupancy
  flags. This is what “marked on a bridge deck” means in S-0105 claim 40.
- **height** *h*: signed integer world height minus terrain surface height at the aircraft's
  horizontal position, minus 416 leptons when its aircraft deck mark is set. In ordinary
  altitude calculations on a high-bridge cell, an unmarked aircraft at least 416 leptons
  above terrain also uses the deck as its reference. Claim 43 distinguishes that temporary
  altitude reference from the aircraft deck mark.
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
  V3-rocket and dreadnought-missile types, which use their locomotor's own answer (S-0506 claim
  22, and claim 18 here uses that answer). The in-the-air answer is the exact test that S-0308's
  *airborne* leaves to this spec for aircraft.
- **strike aircraft**: an aircraft linked to a strike (S-0519 claims 17 and 20), not every
  visiting aircraft or every aircraft dispatched from outside the map.
- **visiting aircraft**: an aircraft marked as not meant to stay on the map. Successful placement
  sets this mark when the type is not both selectable and landable, or its primary weapon is a
  camera weapon. It does not clear an existing mark. A dispatch can mark an aircraft before
  placement. Placement height uses the mark already present, not the type test made afterward.
  The placement fall-back (S-0215 Terms) also comes before that type test, so it too sees only a
  mark already present: an aircraft marked only by the type test makes it by T2's rows for an
  aircraft that is not visiting.
  A carrier's launch of a slot aircraft is such a placement (claim 35), so the same type test
  decides whether a slot aircraft is visiting.
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
- **halving zone**: for an aircraft with positive slowdown distance *S*, the post-step
  distances *d* to its destination point with *d* < 86 leptons and *d* < *S* / 10, where
  ordinary slowing halves its fraction (claim 5, F4).
- **unfinished flight**: movement accepted but not yet completed by settling at a landing
  destination (claim 40); this is distinct from a nonzero speed fraction.
- **attack run**: one approach to a target ending in one or more firing passes and a pending
  round of ammunition spent (claim 15), separate from immediate spending by passenger drops. A **firing pass** is one processing in which the run fires.
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
   A type that only the type test marks visiting (Terms), with no visiting mark before placement,
   is therefore placed on the ground inside the usable area. Its placement fall-back is made before
   the type test marks it (Terms): on the ground it forgets target and destination and has
   *guard* (*area guard* for an armed aircraft of a computer-controlled house outside a team), not
   *retreat*, and it makes no further fall-back while it rests there. Orders, and a carrier's
   launch pass (claim 35), reach it in that behaviour. A later fall-back to rest on the ground
   takes T2's visiting rows (Terms).
   - Backing: `measurable` (scenario AC1; AC29 parts g (4) and h for a type that is not both
     selectable and landable)
   - Confidence: high; medium for the placement fall-back of a type marked visiting only by the
     type test, until AC29 parts g (4) and h run

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
   No fraction of a lepton is kept from one step to the next. Because map positions are
   positive, each coordinate moves by its component rounded down, so a one-lepton step whose
   body facing lies between east and south (both components positive and below one; F3's
   table gives the exact facings, 16391 to 32766) leaves the aircraft where it is (`quirks.aircraft_unit_step_stall`). A step of two or more
   leptons always changes at least one coordinate. Under the default round-toward-zero
   arithmetic (S-0205 claim 5) a negative component, however small, also costs a whole lepton:
   every step at body facings 32767 to 32772 (just past due south) also moves the aircraft one
   lepton west, and every step at 49148 to 49155 (around due west) one lepton north, so a long
   flight at such a facing drifts sideways by one lepton per step (F3).
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
   - Backing: `measurable` for original motion (scenarios AC3, AC23 and AC26).
     The unit-step-stall switch-off alternative is `unmeasured`: it is a design rule for
     our engine, not an outcome of the original. The rounding alternative is owned by
     S-0205 claim 5
   - Confidence: high

5. **Speed fraction.** While a living aircraft that is not landing has a destination, its target
   speed fraction is *τ* = min(1, *d* / *S*), where *d* is the truncated table square root of
   the squared planar difference between its position and destination point (S-0204 claim 5),
   without a building-size subtraction, and *S* its type's signed integer slowdown distance
   (Formula F4). This quotient applies only for *S* ≠ 0. For *S* = 0, the ordinary target
   fraction is 1 at every distance, including *d* = 0; the distance clamp and 0.1 adjustment
   below still apply. This is not the fixed-full-speed exception. When *τ* < 0.1:
   - if *d* < 86 leptons, the current fraction is halved at once and *τ* = 0,
   - otherwise *τ* = 0.1.

   In ordinary slowing, F4 defines the distance-clamped fraction from the possibly halved
   fraction and *d*. Its creep-restart value is 0.05 when *τ* = 0, the distance-clamped
   fraction is 0 and *d* > 0. These intermediate values define the resulting fraction,
   not a required internal evaluation order. An
   aircraft that is taking off keeps its target unchanged until its height reaches half its
   cruise height. At the end of every living movement tick the current fraction moves toward
   *τ* by at most 0.1.

   **Observable consequences within one movement tick (Q-0651).** The position change in a tick
   is determined by the current fraction retained from the preceding movement tick. The distance
   *d* used for the fraction calculation is the distance from the position resulting from that
   change. The fraction retained at the end of the tick affects movement only in the following
   tick. When the creep-restart conditions hold, the end-of-tick fraction is 0; accordingly, the
   restart does not change any position, and omitting it produces the same observable behaviour.

   **Consequence: where ordinary slowing ends.** Without the fixed-full-speed case below, and
   with *S* > 0, the halving zone (Terms) is where *d* < 86 and *d* < *S* / 10. A fraction calculated outside that zone is never below 0.1, so the next
   forward step is at least ⌊*V* / 10⌋ leptons, *V* the converted speed.
   - With *V* ≥ 20 every step taken outside the zone is at least 2 leptons and the aircraft
     advances at every heading until, after some step, it is in the zone. From then on each
     tick halves its fraction and lowers it by 0.1, so it reaches exactly 0 within a few ticks,
     normally in one, and stays there while the destination point is unchanged: it hovers that
     far short of the point and makes no further forward progress. The stopping distance is
     therefore normally below min(86, *S* / 10) leptons and not necessarily below 16.
   - With 10 ≤ *V* < 20 the step is exactly 1 lepton at every post-step distance outside
     the zone with *d* < 2*S* / *V*, once the fraction has settled on its target
     max(0.1, *d* / *S*). Such distances not below 16 leptons exist when
     2*S* / *V* > max(16, min(86, *S* / 10)). If, in any tick with such a 1-lepton step, the
     body facing is one of F3's facings 16391 to 32766 (between east and south), that step
     changes neither coordinate (claim 4, F3). Then *d*, *f* and the facing stay as they are
     and the aircraft hovers there with a positive fraction indefinitely while the
     destination point is unchanged (`quirks.aircraft_unit_step_stall`). At other facings the
     1-lepton steps move it on, and it stops at fraction 0 in the zone as for *V* ≥ 20.
     Inside the zone a motionless step changes nothing, because halving and the zero target
     do not depend on progress. The facing is the quantised direction to the point (S-0204
     claim 8), so the outcome follows from the exact positions. For example, with *V* = 12 and
     *S* = 1000, an approach from the west 11 leptons north of the point's row hovers at
     *d* = 166 with fraction 0.166. Starting 11 leptons south of that row, the aircraft drifts
     north while its steps are longer, until the direction becomes exactly east; its 1-lepton
     steps then go east and the fraction reaches 0 at about 85 leptons. From the north, 11 leptons
     to either side of the point's column, it hovers as well: from the east side the drift ends
     1 lepton east of the column, where the quantised direction lies just east of south. From
     the west side the quantised direction is first 32767 (S-0204 claim 8 (b)), so each step
     also moves the aircraft one lepton west (claim 4), until the side distance exceeds about
     1/100 of the remaining distance along the column; it then hovers at *d* = 166 with the
     body facing well east of south. Started 20 cells (5120 leptons) north of the point, the
     drift ends 47 leptons west of the column and the hover facing is 29778.
     The landing triggers of claim 6 need a zero target fraction or a current fraction below
     0.05, so the same hover also stops a *move* or a return to a pad short of landing.
   - With *V* < 10 the step ⌊*V* × *f*⌋ can become 0 while *f* is still at least 0.1, before
     the zone is reached; the aircraft then hovers with a positive fraction indefinitely at
     every heading (Edge cases).
   - With a small *S* the aircraft can instead come within 16 leptons while its fraction is
     still positive, before either kind of hover.

   **Approach target replacement.** After the adjustment, a living aircraft flying toward a
   destination, neither landing nor taking off, can get a replacement target fraction. With
   the fixed-full-speed exception below inactive and no fly-by flag, the replacement is 0
   below 128 leptons, 0.5 from 128 up to but not including 512, and 0.75 from 512 up to but
   not including 768. At 768 leptons or farther, the ordinary target remains. Distances are
   raw planar point distances after any pad-position adjustment, not building-adjusted object
   distances. The next movement tick's ordinary calculation recomputes *τ* before its 0.1
   adjustment, so a replacement has no effect when that calculation runs. It governs an
   adjustment only in a later tick in which the ordinary calculation is skipped and nothing
   else has set the target, such as a take-off restarted below half its cruise height by a
   refused touchdown (claim 7). Landing and take-off set their own targets (claims 7 and 8).
   - Backing: `measurable` for resulting fractions, the stopping distance, the 1-lepton
     hover and the ineffective replacement (AC3 and AC24). The clamp's exact placement among the ordinary halving,
     creep and adjustment is `unmeasured`: with integer distance and fractions in 0 … 1, it
     changes only the zero-distance case, where several alternative intermediate orders give
     the same final zero fraction. The replacement's effect in skipped-calculation ticks is
     `unmeasured`: AC3 and AC24 isolate its ordinary ineffectiveness, not a refused-touchdown
     restart retaining that replacement below half height.
   The target is instead fixed at 1 (no slowing down) while the aircraft carries the run lock,
   and while it holds its height (claim 6) and either its type has the fly-by flag or it is a
   strafer or fighter (claim 11) with ammunition left. This fixed-full-speed case does not apply
   the halving, distance clamp or creep restart; while it applies the fraction does not fall to 0.
   - Confidence: high (formula, constants, order within the tick, the stopping consequence
     and the 1-lepton hover); medium (the full-speed conditions and the skipped-calculation cases of the
     replacement)

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
   (claim 18), which sends it to a pad or makes it crash, except for claim 45's pad-bound type
   with no dock types and ammunition left, which keeps its cell and stays above it. A landing
   sets the held height to the landing height. There is also a close-approach trigger: below
   128 leptons, with ordinary slowing permitted, no holding-height mark and current fraction
   strictly below 0.05, guidance requests landing without waiting for the exact-cell and
   zero-target-fraction test.
   Both triggers still obey the planned-route and pad-bound restrictions.
   - Backing: `measurable` (scenario AC4)
   - Confidence: high (the triggers); medium (the 120-lepton rule, the planned-route exception,
     and claim 45's pad-bound exception, as claim 45 rates it)

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
   is destroyed by S-0312 claim 16's crash-without-kill rule. The fall-back crash of a pad-bound
   aircraft (claim 46) is not a crash without a kill: it starts at health 0 and falls as a dead
   aircraft. If no impact occurred and
   ordinary movement remains active, horizontal motion and the ordinary altitude rules still
   apply. Thus the net drop need not be the counter alone; a moving dead aircraft also gets
   F2's descent, and a living aircraft can get F1's climb or F2's descent. An aircraft that is sinking in water instead sinks by 5 leptons per tick
   and is removed when its height is below −400 leptons (claim 37 gives its draws).
   - Backing: `measurable` for dead crash flight (AC5) and for the fall-back crash of claim 46
     starting at health 0 (AC29 part f). The living switched-off crash and
     sinking branches are `unmeasured`: AC5 has no established normal-input effect that reaches
     them without killing immediately or using a different movement mode. Its exploratory
     variants cannot yet confirm those branches.
   - Confidence: high (the counter); medium (which switched-off states start a crash, and the
     fall-back crash of claim 46 starting at health 0, until AC29 part f runs)

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
    firing check between them. Ordinary shots advance the burst counter and set the rearm
    countdown as S-0301 claims 1 to 6 say, including any in-burst draw, so after an ordinary
    full burst the countdown holds the full delay. While a first passenger exists, each firing
    attempt is instead a drop under claim 20 and S-0522 claim 11, without those ordinary-shot
    effects. If a burst releases the last passenger, its remaining attempts use the ordinary
    shot rules: there is no new firing check between them.

    **Drain continuation (departure).** Drain attempts have S-0608 claims 1, 2 and 16's
    effects instead of ordinary-shot effects. When an attempt clears the object target of
    an aircraft that remains in play, our engine cancels the remaining attempts of that burst,
    sends no scatter request for that pass, and enters *done* with a 1-tick processing delay. This overrides the normal
    post-firing transitions in T1 for every kind of aircraft and every firing pass. It neither
    restores the target nor undoes a formed drain link, and it retains any pending round for
    claim 15's ordinary spending boundary. It does not itself clear the destination or start
    a rearm countdown. The original can fault after such target clearing when the aircraft
    continues its burst or tries to scatter at the missing target. This legal-data fault has
    no quirk switch or importer warning. A cell-target drain attempt keeps its target and
    uses the normal pass transitions, not this departure.
    - Backing: `measurable` (scenarios AC7, AC8, AC9 and AC23; S-0608 claim 16,
      scenario D12 for cell-target drain attempts). The safe object-target continuation is
      `unmeasured`: it is our engine's departure, checked by AC28, not an oracle outcome.
      The original fault is `unmeasured`: no destructive crash test is proposed
    - Confidence: high for the ordinary run and pending-round admission; medium for the
      drain composition; the safe continuation is a design rule

13. **Strafing run.** A strafer that reaches its target makes five firing passes in one run: the
    first pass fires its full burst, the next four fire one shot each. The passes after the first
    are processed one bare rate-of-fire field apart, in ticks, and the aircraft flies straight on
    toward the target during them. Each ordinary shot sets the rearm countdown of S-0301
    claims 1 to 3, 5 and 6; a drop attempt instead follows claim 20. A pass whose firing check
    answers *rearming* waits and retries every tick. For ordinary shots with no other refusal,
    spacing is the larger of the bare field and S-0301's delay. A drop sets no ordinary rearm
    delay, but does not remove the pass's bare rate-of-fire processing delay. Later passes fire even when
    the check answers *not facing*, *cloaked* or *out of range* (claim 19). After the fifth shot the aircraft waits
    ⌊(*R* + 1024) / *V*ₜ⌋ ticks before it starts the next run, where *R* is the weapon's range in
    leptons and *V*ₜ the converted type speed (Formula F6, including its zero-speed departure). A strafer begins its firing passes as
    soon as the target is within its weapon's range; a strafer that has not yet reached that
    range keeps flying at the target. If the first pass's check answers *not facing*, the strafer
    goes back to choosing a firing spot and waits 45 ticks.
    - Backing: `measurable` (scenarios AC7 and AC23; S-0522 claim 11, scenario P7 for drops)
    - Confidence: high

14. **Fighter and ordinary runs.** A fighter fires its full burst in one pass and then starts the
    next run, choosing a new firing spot, while it has ammunition. Any other aircraft fires its
    full burst in one pass and one more single shot in the next pass. With the game-wide
    re-approach flag on, it then comes back to choose a new firing spot, which ends the run. With
    the flag off it fires its full burst again from where it is, then another single shot, and so
    on: the run never reaches a point that spends its round, so one round gives unlimited fire at
    S-0301's rate until the target is gone or aiming fails (`quirks.aircraft_endless_refire`).
    This unlimited-fire consequence concerns ordinary shots. A carrying aircraft's attempts
    instead obey claim 20: successful drops spend ammunition immediately, and drop attempts
    do not set the ordinary rearm delay. The run transitions and processing delays still apply.
    - Backing: `measurable` for the original runs (scenario AC8, including its explicitly
      named test-map variants). The endless-refire switch-off alternative is `unmeasured`:
      it is a design rule for our engine, not an outcome of the original
    - Confidence: high

15. **Pending ammunition spent once per run.** The initial firing pass admitted by the firing check
    makes one round pending for the run, whatever its burst and however many passes the run
    has. Drop attempts count for this bookkeeping even if every attempt is refused.
    An admitted initial pass using a draining weapon also makes its round pending, even
    though its attempts make no ordinary shot and use no ordinary per-shot ammunition
    (S-0608 claim 16). Neither failed attachment nor target clearing rolls that round back.
    Claim 12's safe continuation keeps it pending until the usual spending boundary.
    The round is taken from the count at the next point where the run either starts its next
    approach or ends, or, if the aircraft leaves *attack* first, at the end of that aircraft's
    update, including a change that starts a queued behaviour in that update. At the *done*
    boundary, the pending round is removed only when the count is positive (table T1).
    Ordinary firing itself does not change the count. Each successful drop additionally
    spends one round immediately; refusal has no net immediate change (S-0522 claims 9
    and 11). Thus a refused-drop run can still spend its pending round, and successful
    drops' immediate spending is separate from that round. A run that never reaches an
    admitted initial firing pass makes no round pending. This replaces the ordinary per-shot
    spending of S-0301 claim 10 for aircraft.
    - Backing: `measurable` (scenarios AC7 and AC8, S-0522 claim 11 scenario P7,
      and S-0608 claim 16 scenario D12 for cell-target drain admission). Object-target
      continuation is the design departure of claim 12, not an oracle outcome
    - Confidence: high

16. **Scatter.** Except for claim 12's target-clearing drain departure, every firing pass
    sends a scatter request (S-0215 claim 19) to the occupants of
    the target's ground cell layer. Its threat point is the aircraft's current position.
    It is forced and not move-anywhere. The cell-wide forwarding gates of S-0209 claim 11
    still apply: the cell request need not reach every occupant, or be accepted.
    - Backing: `measurable` for ordinary scatter (scenarios AC9 and AC23).
      The target-clearing drain exception is `unmeasured`: it is claim 12's design
      departure, checked only on our engine by AC28
    - Confidence: high

17. **End of the run.** A run ends when the aircraft has no target or no ammunition left
    (claim 32 says when a target is lost during a run). Then:
    - with no ammunition, a visiting aircraft or an aircraft of a human-controlled house forgets
      its target,
    - a strike aircraft that still has ammunition queues *retreat*,
    - every other aircraft falls back to rest (claim 18), which normally sends it to a pad.

    An aircraft that still has ammunition and a target when its run ends goes straight into its
    next run. Every other run end also gives the aircraft a new destination, after the target
    is forgotten and before *retreat* is queued or the fall-back is made: a cell on its house's
    computed edge (S-0522 claim 5), whatever edge the house's map section names, selected as in
    S-0522 claim 4 with that claim's draws (claim 37). The aircraft's flight follows this
    destination at once, whatever its behaviour (claim 5), until something replaces it:
    - wherever table T2 reaches its pad rows (claim 18), the fall-back replaces it with the pad
      or an F8 landing spot, or makes the aircraft crash, so the aircraft does not head for the
      edge,
    - T2's *guard* row keeps it; for an armed aircraft with no dock types this is claim 45, where
      the *move* that *guard* queues for a human-controlled house replaces it (claim 40),
    - a visiting aircraft without ammunition forgets it (T2), and its *retreat* chooses an edge
      cell again (claim 34), on its house's arrival edge, which can differ from the computed
      edge of the first choice; a pad-bound one with no pad found crashes instead (claim 45),
    - a visiting aircraft with ammunition left that is not in a team keeps it in *hunt* (T2).
      While its hunt finds no target, the fall-back gives it *hunt* again, so it heads for the
      edge cell, where claim 10 can remove it,
    - a strike aircraft with ammunition left that queues *retreat* keeps it (claim 34),
    - a carrier slot aircraft that is not visiting falls back by its own type's rows of T2
      (claim 35). With no dock types it keeps the edge cell itself as destination through that
      fall-back, and its flight follows that cell, or claim 40's replacement for it once the
      *move* that *guard* queues for a human-controlled house has replaced it, until its next
      launch pass, where T4's attacking rows decide what it does next (claim 35); a visiting
      one takes the visiting rows above until that pass (claim 35).

    So each such run end takes S-0522 claim 4's draws, one or two, whether or not the cell is
    then used. Separately, a strike cancellation queues *retreat* for the affected strike aircraft
    under S-0519 claim 19, without waiting for the run to end. Cancellation does not itself
    unlink them; their later fall-back and retreat follow claims 18 and 34 here.
    - Backing: `measurable` (scenario AC8; S-0519 claim 19's cancellation scenario; AC29
      parts a and e for the edge destination and its draws). `unmeasured` for ignoring an edge
      that the house's map section names, which inherits S-0522 claim 5's unresolved loading of
      that edge, as claim 34 says. The interval in which a carrier
      slot aircraft heads for the edge before its next launch pass is `unmeasured`: AC21 does
      not record the slot's destination between its run end and that pass (claim 35, Open
      questions). The two visiting
      bullets are `measurable` through AC29 part g (claim 45's type that is not landable).
    - Confidence: high, except that the outcomes the bullets take from claims 18, 34, 35, 40 and
      45 and from T2 have those claims' confidence; medium for the carrier slot aircraft's interval and for the strike
      aircraft's kept cell, which claim 34 leaves `unmeasured`; low for ignoring a map-section
      edge, as S-0522 claim 5 rates whether a player house reads one

18. **Falling back to rest (aircraft).** When one of the first three situations in table T2 (State and ordering) applies, the aircraft has that row's stated outcome and S-0215 claim 13's common effects do not occur. In all other cases the common effects apply, and the aircraft's next behaviour is the result specified by the applicable remaining situation in table T2. In flight, an armed aircraft
    with dock types looks for a pad (claim 21) and asks it for contact. When the pad grants it, it
    gets the pad as destination and queues *enter*. Otherwise a pad-bound aircraft crashes
    (claim 46 gives how that crash starts, claim 9 its fall), and any other aircraft gets a landing spot near its house's buildings (Formula F8)
    and queues *move*. On the ground it forgets its target and destination and queues *guard*, or
    *area guard* for a computer-controlled house's aircraft outside a team. A visiting aircraft
    without passengers queues *retreat*. Whatever the table chose, an armed aircraft without
    ammunition that has no contact looks for a pad once more. If it finds one, it gets the pad as
    destination, forgets its target and queues *enter*. If it finds none and is pad-bound, the
    behaviour the table chose is not queued: in flight it crashes (claim 46); on the ground it
    does not, since a crash starts only above the ground (claim 9), and it keeps its current
    behaviour, with its target and destination as the row left them (forgotten, for the ground
    row above). An aircraft that holds any contact queues *enter*. If the aircraft is ready to
    change (claim 31) the queued behaviour becomes current at once. This is the aircraft part
    that S-0215 claim 13 leaves to this spec.
    - Backing: `measurable` for human-owned, non-team rest outcomes (AC10). The original's
      computer-only ground choice and team-dependent rows of T2 are `unmeasured`: the setup
      cannot require those owners or teams to perform a specified rest transition. The restored
      overridden behaviour and the two other early-exit rows are also `unmeasured`: AC10 has
      not established normal inputs that isolate a fall-back invocation in those states.
      The computer and team cases are context, not extra rules for our AI houses. This includes
      the in-flight *guard* row's *area guard* for a computer-controlled house (claim 45).
      F8's other-object fallback and equal-distance object preference are covered by claim 41
      and AC23; footprint qualification and candidate choice are S-0212 claim 25a.
    The in-flight rows' own pad search needs the in-the-air answer (Terms); the other rows use
    the on-the-ground test, and the search after the row needs neither.
    - Confidence: high (the in-flight and pad rows); medium (the ground rows and the computer
      house's choice on the ground); low (the in-flight *guard* row's *area guard* for a
      computer-controlled house, `unmeasured`)

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

20. **What a shot does.** While an aircraft has a first passenger, a firing attempt is a drop
    attempt (S-0522 claim 11), not an ordinary projectile shot or a failed projectile launch.
    Refusal retains the rearm countdown and has no net immediate ammunition change. Success
    expires that countdown immediately, spends one round and resets the drop pass counter to 5.
    Neither outcome advances the burst counter, sets an ordinary full or in-burst delay,
    takes its delay or spray draw, or produces a weapon report, shot animation (S-0301 claims 20
    to 23) or projectile.
    Successful drops retain their parachute sound and applicable animation draws. Run-level
    pending ammunition and pass processing delays remain separate (claims 13 to 15).
    This drop path also bypasses the firing reveal and kamikaze removal described below.
    Without a first passenger, the ordinary shot path applies, including to remaining
    attempts of a burst that has just released its last passenger.
    For an ordinary shot, the projectile starts as in S-0306, with two aircraft changes: a projectile whose
    type has turn rate 0 (a bomb) takes the aircraft's body heading, pitch and current ground
    speed instead of its own launch velocity, so it keeps flying forward as it falls; a projectile
    whose type has turn rate 1 is redirected toward the target in three dimensions, including
    the height difference, at the weapon's speed.
    For an aircraft of the currently viewing house, a successful shot can reveal an area around
    the aircraft when the aircraft, certain nearby samples or the target are under shroud.
    The radius is the game-wide attacking-aircraft sight range; the area reveal follows S-0409
    claim 17. This is not an unconditional reveal for every owner's shot. The exact set of
    nearby samples remains untraced here. A kamikaze slot
    aircraft (S-0506) is removed after its ordinary shot.
    - Backing: `measurable` (scenario AC12; S-0522 claim 11, scenario P7)
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
      The overflow switch-off exact-distance comparison is `unmeasured`: it is a design
      alternative for our engine, not an outcome of the original.
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
    Ordinary aircraft pad landing, docking and departure add no entered-by raise on the pad's
    object tag and no destroyed-by-anything raise on the aircraft's own tag merely for
    docking (S-0524 claim 14). This does not suppress cell-tag effects during approach.
    - Backing: `measurable` (scenarios AC14 and AC23; S-0524 claim 14, scenario C16)
    - Confidence: high for docking; medium for absence of additional object-tag raises

25. **Landing-facing choice.** Where flight calls for the landing direction, an aircraft in
    contact uses its contact's current body facing rounded to 8 directions, not the direction
    toward the contact. With passengers but no contact it uses its own current body facing
    rounded to 8 directions. With neither it uses the game-wide pose-direction integer as an
    eight-direction index: the resulting facing is that integer times 8192, modulo 65536.
    This differs from the pose-direction conversion used for production placement (S-0404
    claim 13). Claim 42 gives one use: guidance requests the aiming facing toward it below
    256 leptons of the flight point when the aircraft lacks a target or ammunition. When other
    uses make it the requested heading remains open.
    - Backing: `measurable` (scenario AC14); `unmeasured` for application times other than
      claim 42's aiming request, because those uses have not been traced
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
    and raises its health by the game-wide repair step (at least 1). F11 distinguishes the
    original's raw cost divisor from our engine's guarded divisor: pad repair uses the same
    departure as S-0314 claim 5, including for a negative imported repair step, not only when
    the original would stop with a program error. When the result reaches full
    strength, health is set to exactly its strength and the aircraft is released in the same
    step. If the house cannot pay, the aircraft is released unrepaired. An aircraft whose health
    ratio is at least 1 gets no repair. The reload-pad service branch repairs without a separate repair flag. A building that
    also has the repair-building flag instead follows S-0314 claim 6's service branch.
    - Backing: `measurable` for original repair outcomes and invalid-divisor failures
      (AC16). The guarded-cost alternative is `unmeasured`: it is the design departure
      owned by S-0314 claim 5, tested on our engine rather than on the original.
    - Confidence: high (step and cost formula); medium (release when the house cannot pay)

28. **Taking off with half ammunition.** An aircraft on a pad that has a target is served only up
    to half its rounds: once its ammunition is at least ⌊*A* / 2⌋ the pad's reload request is
    confirmed without giving a round (`quirks.aircraft_half_reload_with_target`). In *guard* on
    the pad, an aircraft with a target stays while its ammunition is below ⌊*A* / 2⌋ and takes off
    into *attack* as soon as it reaches that. An aircraft of a human-controlled house gets a
    target on the pad only from an order.
    - Backing: `measurable` for the original half-reload rule (scenario AC17).
      The switch-off full-reload alternative is `unmeasured`: it is a design rule for our
      engine, not an outcome of the original
    - Confidence: high

29. **Guard below cruise height.** An armed aircraft in *guard* below its cruise height that has
    less than full ammunition and no contact looks for a pad (claim 21); if it finds one it queues
    *enter* toward it and forgets its target. An aircraft in *guard* at cruise height with a
    destination queues *move* if it is not pad-bound (claim 33); an armed one without destination falls back to rest (claim 18),
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
    - Backing: `measurable` for the original capacity rule (scenario AC18).
      The list-only switch-off alternative is `unmeasured`: it is a design rule for our
      engine, not an outcome of the original. The build-limit switch is owned by
      S-0407 claim 16
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
    not be treated as equivalent to destruction: they destroy no aircraft by themselves; the fall-back that follows is claim 46's. A sale leaves an
    approaching aircraft without the pad as its destination when it falls back; a refusal does so
    when no behaviour is queued for the aircraft (claim 46; Open question 19 otherwise). An aircraft in *attack* whose target is gone
    ends its run (claim 17). Before processing, an aircraft also clears a target whose
    in-the-air answer is yes, even when that target is still alive. A fully cloaked ground
    target is also cleared before processing when it is not allied with the aircraft and
    the aircraft's house does not sense it (S-0516 claim 25). An allied or sensed ground
    target is not cleared by this additional cloak test.

    **A target that leaves the map (Q-1755).** Apart from those tests before processing, an
    aircraft loses an object as its target at the moment the object is killed (at the hit that
    takes its last health), leaves the map or is removed. The loss happens wherever in the tick
    that event falls, also during another object's turn, so from then on in that tick the
    aircraft holds no target, unless a revert brings that object back (below). Leaving the map includes boarding any transport, open-topped or not
    (S-0507 claim 6), and going into a building as an occupant (S-0508 claim 5). For these events
    no owner, alliance or sensor exception applies. This is the loss every actor has (Changes for
    other specs, S-0309). S-0215 claim 10 lists the same events as revert causes, so an aircraft
    whose behaviour has been overridden is also reverted at that moment, right after the loss.
    That claim's revert brings back the target the override put aside, even when it is the very
    object that left the map. If the
    aircraft's targeting timer (S-0215, Terms) has more than 10 integer ticks left at the loss,
    one draw restarts it first (claim 37). A vehicle that enters a tank bunker stays on the map
    (S-0606), so it is not lost that way: the aircraft keeps it as its target, and what the firing
    check answers for it belongs to the firing and bunker specs. A harvester docked at a refinery
    is not known to leave the map, so it is not known to be lost either (Open question 23).

    After the loss, unless a revert brings the object back (above), the aircraft keeps flying in its current T1 stage until its next *attack*
    processing, which comes when the delay set by its previous processing has run out; the loss
    does not bring it forward. That processing takes the stage's no-target row to *done*. Every
    stage of T1 has such a row, *refiring* included. A strafer whose target is killed by one of its
    five strafing shots is in a strafe pass or, after the fifth, in *approaching*, and takes that
    stage's row (T1). Its delay is 1 tick, except from *choosing*,
    which takes the standard delay. The processing of *done* then ends the run as claim 17 says.
    When the loss falls while the aircraft is already in *done*, its next processing ends the run.

    A loss of these kinds never falls inside a full burst, except through a drain attempt (claim 12, *Drain
    continuation*). The *B* shots of a full burst are made one after another within the
    aircraft's own processing, and no ordinary shot's hit lands before that processing has
    ended. So none of these losses cuts a full burst short, even when its first hit then kills the target, and the
    run spends its one pending round (claim 15) whichever hit kills. This holds for weapons without the
    suicide flag. A suicide shot (S-0519 claim 11) makes no projectile: its hit on the firer itself
    lands at once, inside the aircraft's processing, and is the one hit that can. So an aircraft
    whose weapon has the suicide flag takes its own hit at its burst's first shot, and dies of it
    unless it survives as S-0519's edge case of a suicide firer that survives its own hit allows; what
    its further attempts then do, and whether its death's detonation can reach the target before
    the burst ends, is not established (Open questions). A death weapon fired inside a burst can
    only be that firer's own, since no other hit lands there. A target killed by a hit
    while the aircraft waits in *refiring* is lost as above, and *refiring*'s no-target row then
    ends the run with no further shot. Apart from a revert (above), the original does not keep the
    object as target while it is inside, so the aircraft neither waits at its firing spot nor chooses new firing spots
    for it. It fires nothing more. A loss before the run's admitted initial firing pass leaves no
    round pending, so it keeps all its ammunition. A round already pending is spent as claim 15
    says. The
    re-approach flag plays no part, since the rows that read it need a target. With ammunition
    left, claim 17 and T2 then decide what it does. An aircraft with dock types whose pad grants
    contact takes the run end's edge-cell draws, then gets the pad as destination and *enter*.
    When the object comes back on the map (it is unloaded, or leaves the building), the aircraft
    does not get it back as target and its run does not resume. Only an order or a target search
    (S-0309, claim 33) gives it a target again.
    - Backing: `measurable` (scenario AC19, including its Q-1755 parts, where the target is the
      test house's own unit, attacked by force-fire, since the loss has no owner exception).
      The no-target rows' delays and the full burst before a kill: `measurable` (AC19, Q-1755
      parts a and i; part j for a strafer's *approaching* after its fifth pass, with no target and
      with no ammunition). `unmeasured` for the 1-tick delays of the *firing* and *start* rows: *start*
      and *firing* each hold the aircraft for one tick at a time (T1), so no scenario here can
      place the loss of the target or of the ammunition in either. `unmeasured` for a suicide shot inside a burst, which no scenario here gives an
      aircraft; it rests on S-0519 claim 11 and its edge case of a suicide firer that survives its
      own hit.
    - Confidence: high for destruction, and for the loss when a target boards a transport or goes
      into a building, its moment, the run end that follows with no round spent when the loss
      comes before firing (AC19), and no
      resumption. Medium for the full burst made in full before a killing hit and the run end
      from *refiring* (AC19 part i, until it runs), for a strafer's run end after its fifth pass
      (AC19 part j, until it runs), and for the *firing* and *start* rows' delays and the suicide shot's
      hit inside the burst (`unmeasured`). Medium (recovery after a lost contact and hidden-target clearing; the tank-bunker case,
      which rests on S-0606's statement that the vehicle stays on the map); low (the refinery
      dock, which no trace here confirms stays on the map)

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
    It is removed when it leaves the usable area (claim 10). A strike aircraft that queues
    *retreat* at the end of its run with ammunition left already has claim 17's edge cell, on
    its house's computed edge, so its *retreat* makes no choice of its own and takes no draw.
    This differs from a strike cancelled before its run ends, whose aircraft have no target or
    destination when *retreat* is queued (S-0519 claim 19); whether their run end or their
    *retreat* then picks the edge cell is open (Open question 17).
    An aircraft that is not a strike aircraft and is in *retreat*, *paradrop approach*, *paradrop
    overfly*, *spy-plane approach* or *spy-plane overfly* accepts a queued behaviour only when it
    is one of those five; a request to queue any other, such as *move* or *attack*, is ignored
    entirely, with no start, and its current behaviour goes on (Changes for other specs).
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
    - Backing: `measurable` for ordinary flight, exit destinations and cadence (AC20). The
      strike aircraft's kept cell is `unmeasured`: a strike's run ends with ammunition left only
      when its target is lost, and the loss of a strike's target cancels the strike first
      (S-0519 claim 19), which no fixture here separates from the run end (Open questions).
      The targetless approach is `unmeasured`: dispatch targets a map cell, and no normal-input
      way to remove that target during approach has been established. The map-section edge
      contrast inherits S-0522 claim 5's unresolved skirmish loading limitation. The dropped
      queued behaviour is `measurable` (scenario AC29 part h) for a visiting carrier slot aircraft in
      *retreat*; for the other four behaviours and the strike-aircraft exception it is
      `unmeasured`, since no scenario here queues a behaviour for an aircraft in them
    - Confidence: high (targetless outcome, computed exit edge and delays); medium (flight, the
      strike aircraft's kept cell and the dropped queued behaviour)

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
    Between an attack run's end (claim 17) and the next launch pass, at most 10 ticks later
    (S-0506 claim 2), a slot aircraft is not driven by T4. Whether it is visiting follows the
    Terms' type test, applied at its launch placement. One that is not visiting has claim 17's
    edge cell as destination and falls back to rest by T2's rows for its own type, as any armed
    aircraft in flight does: the fall-back sets apart missile slot types (S-0506), not other
    slot aircraft, so its dock types and pad-bound field decide as for any aircraft (claim 18).
    Unless an earlier row of T2 applies, with no dock types this is T2's *guard* row: *guard*, with
    claim 33's *move* for a human-controlled house, or *area guard* for a computer-controlled
    one, so it turns toward that cell or toward claim 40's replacement for it, as in claim 45.
    A slot type with dock types instead, again unless an earlier row applies, looks for a pad (claim 21), and a pad-bound one that no
    pad grants contact crashes (claim 46), within the same interval.
    At the next pass, for a slot aircraft that has not crashed, T4's attacking rows decide: with no ammunition or no controller target it
    gets its spawner as destination with *move*; otherwise it receives the controller target and
    queues *attack* again. Whether
    it can land on a replacement cell within that interval is not established (Open questions).
    This interval is `unmeasured`, as claim 17's backing says, and rated medium; the slot type
    with dock types is part of it, since AC21's carrier set-up uses a type without them.
    A visiting slot aircraft instead takes T2's visiting rows at its run end, as any visiting
    aircraft does (claims 17 and 45). With its ammunition spent it forgets its target and
    destination and is in *retreat*, unless an earlier row of T2 applies or it is pad-bound (T2's after-the-row entry, claim 45). With ammunition left after its target is lost it is in
    *hunt*, and the next pass's *move* or *attack* applies to it as above. An aircraft in
    *retreat* does not accept that pass's *move* or *attack* (claim 34). So for one in *retreat*
    the pass clears its target and makes its spawner its destination, but no *move* is queued and
    it stays in *retreat*. Its *retreat* chooses an edge cell only when it has no destination
    (claim 34): it takes that second choice's draws only if a *retreat* processing comes before
    the pass, and it keeps the spawner as destination after it. What it does once it reaches its
    spawner, and whether it is still in *retreat* on its next sortie, is open (Open question 22).
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
    - Backing: `measurable` (scenario AC21); `unmeasured` for the interval between a run end and
      the next launch pass of a slot aircraft that is not visiting, which AC21 does not record
      (Open question 20); `measurable` for a visiting slot aircraft's run end, its *retreat* and
      that pass (scenario AC29 part h)
    - Confidence: high; medium for that interval and for the visiting slot aircraft's rows

36. **Loss of a slot aircraft.** S-0506 claim 9 owns loss notification and regeneration.
    A notification for a slot aircraft with health below 1 or already released empties its slot
    immediately and starts its regeneration countdown; it need not wait for impact or final
    removal. Crash start can therefore free the slot while the old aircraft is still falling.
    A notification for a living, unreleased non-missile aircraft leaves its slot unchanged.
    Spawner destruction, ownership change and the other slot-clearing events follow S-0506
    claim 13; release of a non-missile aircraft through that rule starts its crash (S-0506
    claim 12), rather than letting it return to the carrier. For an ordinary aircraft that is
    not seized (S-0607 claims 5, 12 and 20), starting this crash sets health to 0: it is not a
    new living disabled-fall state. Crash start does not clear its unfinished flight (claim 40) or speed
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
    - **Edge cells.** A paradrop plane's arrival cell, a retreating aircraft's destination and
      the destination given at the end of a run (claim 17) use S-0522 claim 4's no-reference
      edge-cell selection: one ranged draw, or two for south, with that claim's ranges and
      order. A visiting aircraft that ends its run without ammunition therefore takes these
      draws twice: at the run end, and again when its *retreat* chooses (claim 34), except a pad-bound one, which crashes or gets a pad at the run end's fall-back instead of retreating (claim 45), and a carrier slot aircraft whose next
      launch pass gives it its spawner as destination before a *retreat* processing (claim 35).
      Spy-plane arrival uses the same selection (S-0513
      claim 43), as do the spy exit destinations of claim 34 on their computed opposite edge.
      Choosing the edge itself takes no draw (S-0522 claim 5).
    - **Target loss.** When an aircraft loses its target object by claim 32's events (killed,
      off the map, removed) and its targeting timer has more than 10 integer ticks left, one
      *U*(4, 8) draw restarts that timer with the drawn number of ticks, before the target is
      cleared, except while the scenario is being set up. With at most 10 ticks left there is
      no draw. The draw is taken in the turn in
      which the event happens, which may be another object's. It is the loss every actor has
      (Changes for other specs, S-0309).

    Flight-destination substitution takes one *U*(0, 7) per examined radius (claim 40).
    The failed edge push-back takes one raw draw only when its random fallback is attempted
    (claim 4). These are not an exhaustive flight draw inventory. Pad selection and the service steps
    themselves take no random draw. The crash tumble draws are S-0312 claim 16's.
    - Backing: `measurable` for smoke, firing-spot and processing-delay draws, and pad/service
      draw absence (AC22, with the permitted random-state recorder); the target-loss draw
      through AC19's Q-1755 part a with the same recorder; edge-cell draws are
      `measurable` through AC20, AC22, AC29 and S-0522 scenario P3. Sinking splash draws are
      `unmeasured`: AC5 and AC22 have no established normal-input witness for aircraft sinking.
      The smoke switch-off alternative is `unmeasured`, a design rule of our engine
      rather than an outcome of the original
    - Confidence: high (the listed draws); low (the unresolved draw inventory)

38. **Boundaries.** This spec does not cover the paradrop and cargo delivery behaviours
    (*paradrop approach* and *paradrop overfly*: S-0522 claims 6 to 8). Ordinary air-transport
    eligibility and *unload*, including airborne phase transitions, dock search and waypoint-700
    replacement, are S-0507 claims 2 and 13, not a future cargo-delivery rule. This spec also excludes the spy
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
    test excludes the searching aircraft itself. This selection runs at the first processing of
    every such *move*, also one that starts with the destination the aircraft already flies
    to, such as the *move* that *guard* queues after a run (claim 45). The aircraft's own
    earlier choice of the cell does not count against it, so such a destination is kept
    whenever these same tests accept it, and replaced as below otherwise (Q-1756 question 3). The heading
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

    The air-unload caller is S-0507 claim 13. Its undefined replacement marker supplies
    centre cell (0, 0); the search and its direction draws still run, including the draw
    for radius 0. Exhaustion takes 32 ranged draws and returns that centre placeholder,
    not the aircraft's previous destination. Initial map loading retains the centre without
    search draws. Claim 41's end-of-update placeholder cleanup still applies after this
    selection, with the fall-back to rest of claim 18.

    After destination selection, flight takes that destination's heading point, now with the
    aircraft as requesting actor. For a cell this is its centre at its surface height; for
    a mobile object its next movement point or stationary position; for a building its
    centre or docking point as S-0506 claim 14 specifies. Claim 6 can replace the world
    height for an armed approach. Pad guidance can subsequently replace a building point
    with the contacting aircraft's docking position (S-0404 claim 15).

    For *move* completion, **stopped** is not the zero-current-fraction test. A movement
    point accepted by the locomotor remains an
    **unfinished flight** until landing has settled in its destination cell, or on its contact
    building. That completion clears the unfinished flight and destination. The locomotor
    reports stopped only with no unfinished flight and no positive forward pitch (the pitch
    comparison is single precision against 0). Thus a holding or non-landable aircraft
    which has accepted a movement point does not report stopped merely by hovering at that
    point with speed fraction 0. A freshly placed aircraft with no unfinished flight can
    nevertheless report stopped above terrain. T1's ordinary-attacker approach uses the other
    test: it moves to *firing* when the current speed fraction is exactly 0, whether or not the
    flight is unfinished (claim 42). A positive fraction whose forward step truncates to 0 does
    not count as stopped there under the default of `quirks.aircraft_zero_step_attack_stall`.
    Claim 42 gives its attack-only alternative.
    - Backing: `measurable` (scenarios AC23 and AC24; AC29 part b for a *move* that starts
      with the destination the aircraft already flies to, a destination that F7 refuses). The
      aircraft's own earlier choice not counting against a destination that passes both tests is
      `unmeasured`: AC29 part b's destination fails F7 whichever way that choice counts
    - Confidence: high for destination selection, draws and the stopped test; medium for
      unusual object heading-point cases inherited from S-0506 claim 14, and for the selection
      on a destination already flown to, until AC29 part b runs

41. **Landing-search settings and fallback (Q-0634).** F8's search near a house building
    uses locomotion class *foot*, movement zone *normal*, the building cell's ground reach
    region for that zone, no deck mode, and a 3 × 3-cell free footprint. It allows bridges,
    asks for no height, no-overlay or burrowing condition, and has no reference cell. The
    aircraft type's own movement zone is not used. S-0212 claim 25a supplies the complete
    footprint qualification and candidate choice: the footprint extends from the candidate
    through two cells in each positive map axis, and the selected top-left cell is returned
    unchanged, not shifted to the middle. Its no-reference candidate choice uses the search
    tick modulo the preferred candidate count, not distance to the aircraft.

    Below F8's 256-lepton weighted-distance threshold, the replacement search centred on
    the aircraft's own cell instead uses a 1 × 1 footprint, the same fixed class and zone,
    the own cell's ground reach region, and its own cell as reference. With no successful
    building choice, or no dock types, the fallback considers other own-house infantry,
    vehicles, aircraft and buildings, excluding self and objects in limbo. It prefers the
    smallest spatial table-root distance, truncated toward zero to integer leptons, for
    which a nearby search succeeds. Equal integer distances favour the object created
    earliest, across all four kinds together; there is no preference by kind, current cell
    or most recent entry onto the map. Removing another object does not change that
    preference among the survivors. This is object creation order, not tick-update order.
    That search uses the other object's cell, a 1 × 1 footprint and otherwise the
    building-search settings, with no reference cell. Its candidate choice uses tick modulo
    as in S-0212 claim 25a. The own-cell replacement instead uses its reference to minimize
    candidate distance, retaining the earliest collected candidate on a tie (S-0212 claim 25a).
    These candidate ties are distinct from the preference between fallback objects.
    With no successful object choice, it returns the aircraft's own current cell without
    another free-cell search. No random draw belongs to these nearby searches.

    **Own-cell search with no result (Q-0650).** When a building is chosen below the
    256-lepton threshold and the search around the aircraft's own cell finds no cell, the
    landing spot is not the spot near the building, not the other-object fallback and not the
    aircraft's own cell. The original uses the search's no-result value unchecked: the landing
    destination becomes the cell position with map coordinates (0, 0). No map has a cell
    there, so the destination is a placeholder cell located at that corner position, whose
    centre is 128 leptons from the map origin in each axis. Any aircraft whose destination or
    target is this placeholder discards both at the end of its update, after its behaviour
    processing and movement, and falls back to rest again (claim 18). Unless the situation has
    changed, that repeats the same search and destination. A *move* toward the placeholder
    first applies claim 40: the placeholder fails F7 because it lies outside the usable area,
    so claim 40's replacement search runs around cell (0, 0), with its direction draws, and a
    usable cell it finds becomes a real destination that is not discarded. On maps where none
    is found the aircraft keeps being given and discarding the placeholder in every update.
    Its flight point is the placeholder's centre, so it heads toward the map's (0, 0) corner
    until its own-cell search succeeds or its weighted building distance reaches 256 leptons.
    Because the other-object fallback is never reached here, the question whether it skips
    the chosen building does not arise in the original (`quirks.landing_own_cell_no_result`
    gives the fixed alternative).

    The edge half of Q-0634 is already answered by claim 34, claim 37 and S-0522 claims 4
    and 5: retreat uses the arrival edge, spy overfly the opposite computed edge, with one
    ranged draw except two on south. It is not a nearest-cell projection with no draws.
    - Backing: `measurable` (AC23 and AC25); footprint qualification and candidate choice also have
      S-0212 claim 25a's scenario P28
    - Confidence: high for caller settings, object preference, empty-house fallback, the
      own-cell no-result destination and its discard. Candidate choice inherits S-0212 claim
      25a's confidence. Medium for the resulting flight path while the placeholder repeats

42. **Close approach and guidance facings (Q-0651).**

    **Observable update consequences.** T1's test of the current speed fraction uses the value
    retained from the preceding movement tick. The facing after an update reflects the last
    direction requested during that update (S-0206 claim 1); an earlier request replaced during the
    same update has no lasting effect. The guidance rules below determine that final request.

    **Guidance.** After each movement tick of a living aircraft that has a flight point, is
    neither landing nor taking off, and is above the terrain:
    - **Body.** Unless it carries the run lock, its body facing is requested toward its flight
      point, using the bearing of S-0204 claim 8 from its current position. Exception: a
      strafer at most 768 leptons from that point makes this request only when the bearing
      differs from its current body facing by at most 8192 facing units; otherwise there is
      no body request in that tick.
    - **Aiming.** Unless it carries the run lock: when its distance to the flight point (after
      any pad-position adjustment) is below 256 leptons, its type lacks the fly-by flag, and it
      is not a strafer with a target and ammunition left, its aiming facing is requested toward
      its target if it has a target and ammunition left, otherwise toward its landing direction
      (claim 25's heading). In every other case its aiming facing is requested toward the flight
      point.

    The body facing alone gives the direction of travel (claim 4). Clearing the destination of
    an aircraft in *attack* (current or queued) that has a target does not end its flight: the
    flight point and unfinished flight remain, so guidance and claim 5's slowing go on toward
    the former firing spot.

    **The ordinary attacker (not a strafer or fighter, no fixed full speed).** Consequences:
    - It flies toward its firing spot throughout the approach. T1's steering rows turn only
      its aiming facing; between 512 and 256 leptons guidance turns that facing back toward the
      spot in the same update, and below 256 leptons guidance also aims it at the target. Its
      direction of travel never turns toward the target.
    - With positive slowdown distance and converted speed at least 20 it slows to a fraction
      of exactly 0 short of the spot (claim 5) and keeps no minimum speed. With converted
      speed from 10 to 19 it does the same, except that it hovers with a positive fraction
      when its 1-lepton step arrives while its body facing is between east and south
      (claim 5, F3's facings 16391 to 32766). Only the zero-fraction hover moves to *firing*
      at the next processing of *approaching*, keeping its destination. If it first comes within 16 leptons with a positive fraction,
      the distance row applies instead and it then stops the same way.
    - In *firing*, its request to turn the body toward the target is replaced in the same
      update by guidance's request toward the spot point, so the body stays pointed at the
      spot. Its aiming facing is requested toward the target by both, and the firing check's
      facing test (claim 19) uses the aiming facing, so it fires from where it hovers once
      that facing is within 2048 facing units of the target and the other firing conditions
      hold. A refusal is handled by T1 as usual.
    - In either positive-fraction hover (converted speed below 10 at any heading, or from 10
      to 19 between east and south) it never reaches *firing* and fires no shot, unless it
      first came within 16 leptons. The zero-lepton-step case is controlled by
      `quirks.aircraft_zero_step_attack_stall`; the one-lepton-step case by
      `quirks.aircraft_unit_step_stall` (Edge cases).
    These ordinary slowing consequences assume *S* > 0 and both switches on, as in AC24.
    With `quirks.aircraft_zero_step_attack_stall` off, an ordinary attacker also moves from
    *approaching* to *firing* when its positive current fraction gives F3 step length 0
    leptons. This does not change its fractions, position, guidance or *move* completion.
    A step length of 1 lepton that changes neither coordinate does not meet this additional
    test and remains governed by the other switch.
    - Backing: `measurable` (scenario AC24), except the strafer's 8192-unit exception:
      `unmeasured`, because AC24's ordinary aircraft cannot exercise it and no isolated
      strafer setup is supplied here
      The zero-step switch-off rule is `unmeasured`: it is a proposed design alternative for
      our engine, not an observable outcome of the original.
    - Confidence: high for the order within an update, the zero-fraction test, the facings
      requested, the retained flight point and the stated design alternative; medium for the strafer's 8192-unit exception

43. **Aircraft deck marking (Q-0841).** Ordinary flight can land an aircraft on a
    high bridge and give it the aircraft deck mark. The mark is not inferred from the
    render layer or from either cell layer's occupancy flags. For a living aircraft,
    the following transitions apply; all height comparisons use signed integer leptons.

    | Situation | Result for the aircraft deck mark |
    |---|---|
    | An unmarked aircraft flies across or hovers above a high bridge without settling onto it | Remains clear, even above deck height |
    | Ordinary descent uses the high bridge as its reference and its descent step reaches or passes height 0 above that deck | Set, with resulting height 0 above the deck |
    | Landing finishes on a high-bridge cell while the aircraft's world height is at least terrain surface height plus 416 leptons | Set before settling at landing height |
    | An ordinary altitude adjustment climbs toward the held height | Cleared in that movement tick, including the first climbing tick after deck take-off |
    | A touchdown is refused under claim 7 | Cleared as take-off restarts |
    | Horizontal flight or hovering alone, without one of these altitude or landing transitions | Unchanged |

    Thus landing can produce a deck-marked aircraft, but merely hovering over a bridge
    cannot acquire the mark. A previously marked aircraft does not lose it merely because
    it is above the deck or crosses a cell boundary. Normal climbing take-off does clear
    it: ordinary cruise over the bridge is not marked just because the aircraft once
    landed there. Render-layer selection still follows claim 7's height test, now with
    the deck mark included in height. A marked aircraft at deck height therefore has
    height 0 and the ground render layer; this is not a separate bridge render layer.
    A positive-height aircraft may retain a mark when it is not climbing, so render layer
    alone cannot recover the mark. S-0105 claim 40 owns the shadow consequence and its
    additional neighbouring-cell qualification. S-0209 claim 13 owns occupant listing.
    Listing and occupancy must not be substituted for the aircraft's own mark.
    Claim 1's ordinary fresh ground placement still replaces the requested height with
    terrain height; requesting an elevated placement is not a way to require a deck landing.
    - Backing: `measurable` (scenario AC27)
    - Confidence: high

44. **Ordinary deck-rest occupancy and departure (Q-0872).** For an ordinary flying
    aircraft resting at height 0 with its aircraft deck mark set on a high bridge,
    the object-present flag is set in the **deck occupancy record only**, not in both
    records and not in ground. This assumes the cell was otherwise clear and no other
    object has marked or cleared it. The aircraft is listed on deck as S-0209 claim 13
    specifies. These are separate properties: setting or clearing the object-present
    flag uses the aircraft's actual world height and the high bridge's presence, not
    its aircraft deck mark or its listing alone. At a marking or clearing event, deck
    is used when a high bridge is present and world height is at least terrain surface
    height plus 416 leptons; otherwise ground is used. Comparisons are signed integer
    comparisons in leptons, with no rounding. Ordinary rest on terrain marks ground.

    On normal deck take-off, the deck object-present flag is cleared in the first
    movement tick that climbs away from rest. It is not transferred to ground when
    claim 43 clears the aircraft deck mark. By the end of that tick the aircraft has
    positive listing height and is absent from both occupant lists. Merely accepting
    the take-off destination without advancing movement does not release occupancy.
    There is no later 208-lepton release threshold: that combat-height test is unrelated.
    These flags are not reference counts; clearing one does not preserve another
    object's contribution to the same flag (S-0209 claim 1).

    **Vehicles below and on deck.** On a bridge over level ground, a vehicle below
    uses the ground occupants and ground record, while a vehicle on deck uses the deck
    occupants and deck record (S-0209 Terms, cell layer). Thus only the deck vehicle's
    selected record contains this flag. However, **the object-present flag itself gives
    no vehicle entry refusal on either surface**: S-0209 claim 5's record table does not
    test it. With no other obstacle, the vehicle below gets *clear*. An ordinary allied
    aircraft at rest on deck blocks a deck vehicle through its listed presence, giving
    *stationary friendly* under S-0209 claim 5, row VP20, not through the flag. Enemy
    aircraft and exceptional moving/contact states retain that claim's occupant rules.
    After take-off removes the deck listing, neither the old listing nor this flag
    blocks a deck vehicle. Landing and placement tests are separate: F7 remains a
    ground-list lookup, and tests of record emptiness must use their own surface rules.
    This claim covers ordinary flight, not S-0207 claim 20's teleport reservations.
    - Backing: `measurable` (scenario AC27, occupancy and vehicle variants)
    - Confidence: high for marking, clearing and vehicle flag exclusion; medium for
      the combined ordinary deck-take-off and vehicle-entry consequences

### After the run

45. **Armed aircraft with no dock types after the run (Q-1756, Q-1776).** An armed aircraft
    with no dock types that is neither visiting nor a carrier slot aircraft, and carries no
    passengers (T2's passenger rows come first), keeps claim 17's
    edge destination through its fall-back (table T2's *guard* row). That cell lies outside the
    usable area (S-0522 claim 4), except in that claim's no-acceptable-candidate case, which
    arises only on the north edge when every cell of the local area's top row lies below the
    negative height level that claim names. Its fall-back cell can then be usable: if claim 40's
    retention tests accept it, a human-owned aircraft keeps it with no replacement search and no
    draws.
    Whether it lands there is decided by claims 6 and 7 alone, as for any destination cell. Every
    aircraft this claim covers is of a landable type (a type that is not landable is visiting,
    Terms), so one that is not pad-bound lands there, unless claim 5's hover stops it
    short, and rests in its T2 resting behaviour, whatever its owner; a pad-bound one does not land, by claim 6's pad-bound rule, and behaves
    as the pad-bound exception below says (this case is `unmeasured` and low, as S-0522 claim
    4's fall-back is: no set-up here makes every cell of that top row lie below that level). In every other case the cell is
    outside, and the rest of this claim describes that case. A carrier slot aircraft that is not
    visiting is excluded here only because claim 35 owns its flight; between its run's end and
    its next launch pass claim 35 applies this claim's rule to it. The aircraft's flight follows the
    cell at once (claim 17). The
    aircraft therefore neither lands at nor holds above its last firing spot or its target after
    its run: it heads for its house's edge. This holds whether its ammunition is spent or its
    target was lost before it fired with ammunition left, since both end the run (claims 17 and
    32). During a strafing run its destination is its target itself (table T1's strafe passes and
    claim 39), so no firing spot survives the run either. What follows depends on its owner:
    - **Human-controlled house.** It is in *guard*, which queues *move* toward that destination
      (claim 33). That *move*'s first processing applies claim 40 to the edge cell. F7 refuses
      the cell, since it is outside the usable area, so the *move* replaces it with the first
      accepted cell of claim 40's search around it, taking one *U*(0, 7) draw for each radius
      examined, from radius 0 up to the radius where a cell is accepted. The edge cell lies just
      beyond the usable area's border (S-0522 claim 4), so the replacement is normally a usable
      cell a few cells from it, toward the inside of the map. The aircraft flies toward that cell
      and, unless claim 5's hover stops it short, lands there by claim 6's landing trigger, since
      it has no target or no ammunition, with claim 7's cell test. It then rests on the ground in
      *guard* (T2's ground row). So its descent comes at no fixed delay after its last round
      (Q-1756 question 1): it first turns toward the edge cell, then toward the replacement cell,
      and its descent starts when one of claim 6's landing triggers holds for that cell (claims 6
      and 7), after a flight whose length depends on where it fired. Should claim
      40's search accept no cell, the edge cell is kept, and the aircraft behaves at it as in the
      computer case below.
    - **Computer-controlled house, outside a team.** It is in *area guard* (T2). An aircraft's
      *area guard* follows S-0215 claim 26 only in part, by height. Each processing compares the
      aircraft's height (Terms), taken at its start, with its cruise height *FL*, not with its
      held height. At exactly its cruise height
      each processing is this spec's fall-back to rest (claim 18) instead of claim 26's rows, and
      ends with a delay of 1 tick: claim 26's delay, with its factor *k* = 2 for aircraft, does
      not apply there. At any other height, above or below, an armed aircraft with its ammunition
      spent and no contact shows at each processing the same fall-back to rest as at its cruise
      height, and in addition: if it holds a target, it has *attack* queued after that processing
      and its next processing comes 1 tick later; if it has no target, the processing also has the
      result of claim 26's rows and table, and the next one comes after claim 26's delay, with *k*
      = 2. T2's in-flight row for an armed aircraft with no dock types keeps the destination and
      *area guard* (Changes for other specs). Claim 26's leash needs an actor with no destination,
      so it does not hold while the edge cell is the destination. Observably, it keeps the edge
      cell as destination, stays in *area guard* and queues no *move* (rated in claim 18), so no
      replacement is sought and its destination alone moves it, unless claim 26's search away from
      its cruise height gives it a target; what such a target then leads to was not traced (Open
      questions). It flies past the usable area's border to the edge cell, since it may not leave
      the map now (claims 4 and 10), unless claim 5's hover stops it short. In that cell the
      landing trigger of claim 6 starts a landing, and claim 7's cell test refuses every touchdown
      there, because F7 accepts no cell outside the usable area. Each refusal makes it rise by 10
      leptons and start to take off again (claim 7). It is still in its destination cell with no
      target or no ammunition, so a new landing starts. Unless that hover stops it, it never comes
      to rest: it repeats the descent and the refused touchdown above the edge cell for as long as
      it keeps that destination.
    - **A visiting aircraft**, such as one of a type that is not both selectable and landable
      (Terms), does not reach the rules above: T2's visiting rows apply to it (Q-1776 questions
      2 and 3). The owner changes neither case below. A visiting carrier slot aircraft takes the
      same rows; its next launch pass makes its spawner its destination (claim 35).
      - With its ammunition spent, it forgets its target (claim 17) and its destination and
        queues *retreat* (T2), whose first processing chooses an edge cell again with S-0522
        claim 4's draws (claims 34 and 37). It flies there and is removed, without dying, as
        soon as its position is outside the usable area (claim 10). Neither flight flag changes
        this, since that removal ignores them. A pad-bound visiting type is the exception: T2's
        after-the-row entry follows the visiting row as it follows every row, its pad search
        finds no pad (it has no dock types), and the aircraft crashes at this fall-back (claim
        46) instead of retreating.
      - With ammunition left, the after-the-row entry does not apply (it needs no ammunition),
        so a pad-bound visiting type is treated as any other visiting type here.
      - With ammunition left after its target is lost, outside a team, it is in *hunt* (T2) and
        keeps claim 17's edge destination while its hunt finds no target, so it flies toward
        that cell; a target that the hunt finds starts a new run. With neither flight flag it is
        removed, without dying, when its position leaves the usable area on the way, provided
        it has entered the usable area at least once since it appeared, since it may then leave
        the map now (claim 10); one that has never entered it is not removed there. With the fly-by or the fly-back flag it is not removed
        at that border, only if its position leaves the map; what it does once it reaches the
        edge cell is open (Open questions).

      Where it is removed at that border, the time to its removal is its flight from where its
      run ended to the border; no fixed delay applies.
    - An aircraft that is not visiting is not removed: claim 10 removes only visiting aircraft and aircraft in
      *retreat* for leaving the usable area. The fly-by flag changes nothing here. It fixes the
      speed only while the aircraft holds its height (claim 5), and after its run this aircraft
      does not (claim 6): it has no target or no ammunition, and its destination point lies at
      the surface. The fly-back flag acts only on an aircraft that may leave the map now (claims
      4 and 10).
    - A pad-bound type with no dock types that is not visiting is the exception (the visiting
      case is above). With its ammunition spent it crashes at this fall-back, by T2's after-the-row entry, since no pad can be found (claim
      46). With ammunition left it keeps the edge cell in T2's resting behaviour for it (*guard*,
      or *area guard* for a computer-controlled house outside a team) without a *move*, because
      claim 33 queues *move* only for an aircraft that is not pad-bound. It flies to the edge
      cell, does not land there (claim 6's pad-bound rule), falls back to rest again with the
      same result, and stays above the cell.
    - Backing: `measurable` (scenario AC29 parts b to d, human owner; T1's backing for the
      strafer's destination; AC29 part g for a type that is not landable, except its hunt case
      with a flight flag, and its part (3) for a pad-bound visiting type). The computer owner's *area guard* in flight (except its height test, AC29 part j), its queueing no *move*,
      and its refused landings are `unmeasured`, as for T2's other computer rows (claim 18). The
      hunt case of a type that is not landable with a flight flag, beyond the border, is
      `unmeasured`. The height test of the computer owner's *area guard*, above its cruise height
      included, is `measurable` (scenario AC29 part j, an attempt).
    - Confidence: medium for the height test of *area guard* and its branch above the cruise
      height; medium for the replacement cell and its draws, the landing there, the
      pad-bound exception and the crash of a pad-bound type without ammunition that it leads to,
      the crash of a pad-bound visiting type without ammunition instead of its retreat, and a
      type that is not landable, until AC29 runs; low for the computer case's repeated
      landings (its *area guard* is rated in claim 18) and for the flagged hunt case beyond the
      border.
      High for no removal of an aircraft that is not visiting.

46. **Nowhere to land (Q-1775).** An armed aircraft in flight that must return to a pad (pad-bound)
    and that ends its run, or loses its pad, with no pad granting it contact crashes at that
    fall-back, without first waiting in the air or landing elsewhere. Table T2 decides the
    cases: one whose in-the-air answer is *no* (Terms) skips the row's own pad search, keeps its
    destination and gets *guard* (*area guard* for a computer-controlled house outside a team).
    With ammunition left, or while it holds a contact, it does not crash then. With its
    ammunition spent and no contact, T2's after-the-row entry still makes a new pad search, as
    after every row (claim 18): it then gets a pad it finds as destination, with *enter*, and
    crashes when it finds none, like any other pad-bound aircraft in flight. By the same entry a
    pad-bound type with no dock types crashes only with its ammunition spent (claim 45). The
    crash comes:
    - when its house has no building of any of its dock types (claims 18 and 21, T2),
    - when every such building has no free contact place or does not accept it (claims 21 and
      22): a pad that frees up later does not matter, since the crash has already begun,
    - when the pad it flies to is sold during its approach, or ends its contact by a refusal then
      (a case for which no set-up is known, and which with a behaviour queued at the refusal is
      open, Open question 19), and no other pad grants contact (claim 32), with or without
      ammunition left. A sale removes the
      pad as the aircraft's destination, and a refusal that ends the contact clears the
      destination when no behaviour is queued for it, both before the aircraft falls back to rest.
      So T2's row for an aircraft already flying to its *enter* destination, which needs that
      destination, does not keep it in *guard*: the pad search decides.

    A helipad's destruction destroys every aircraft in contact with it instead, whether still
    approaching or settled on it (claim 32). A destroyed pad that is not a helipad destroys only
    contacts within claim 32's distance threshold; a surviving approaching aircraft has lost its
    pad (T3's "pad is gone" row) and falls back as after a sale, so it crashes here when no other
    pad grants contact.

    Provisionally (medium until AC29 part f runs, as the confidence says), the crash begins with
    the aircraft's destruction, without a hit: no damage is dealt and its health is set to 0.
    That destruction is credited to nobody. It comes from no hit, so S-0302 claim 14, which
    credits the hit that takes the last health, has no hit to credit; it is treated as a
    destruction with neither an attacker nor an attacking house (S-0414 claim 21 for the
    loss), so no house's count of units destroyed, score or
    experience changes, while it counts once in all among its own house's units lost, as the
    score screen shows them after the crash, unless its type is not scored (S-0414 claim 21).
    The score screen's totals do not show at which moment of the crash that loss is counted, and
    this spec does not fix it. A tag on the aircraft is expected to be raised then, as for
    other destructions (destroyed-by-anything); S-0011 claim 19 does not list this destruction,
    so this is unconfirmed. The crash has S-0312 claim 16's crash-start effects: its passengers are lost and three
    tumbling draws are taken. As no hit kills it, it takes none of the effects that S-0312 gives
    a killing hit (the shared death effects, such as debris, and claim 16's explosion-list
    pick), neither at the start nor at impact. That includes a planted bomb (S-0519), which an
    aircraft can carry from a planting while it stood on the ground: this destruction does not
    set it off, unlike the detonation at the end of the shared effects (S-0312 claim 7). The bomb
    stays attached, with its timer, while the aircraft falls on the map, so its timer (S-0519
    claim 4) and the detonate order (S-0519 claim 9) can still set it off during the fall, by
    S-0519 claim 5 at the aircraft's position at that moment. The timer test comes before the
    aircraft's movement in its turn, so a bomb due in the tick of impact goes off before the
    impact. A bomb that has not gone off by the impact is removed then, with no detonation, no
    animation and no damage, as for a carrier removed other than by death (S-0519 claim 7),
    under `quirks.crash_drops_planted_bomb`. It then falls as a dead aircraft, its fall counter
    growing by 1 per tick (claim 9), and at impact its death weapon goes off and it leaves play, as
    for an aircraft shot down (S-0312 claim 16), with nothing counted again. S-0312 claim 16's
    demolition damage for a crash started without a kill does not apply to it: its health is
    already 0 when it falls, so it is a shot-down aircraft in that claim's sense (Changes for
    other specs). An aircraft with dock types that is
    not pad-bound gets an F8 landing spot instead (T2), and one with no dock types follows claim
    45. The owner being human- or computer-controlled changes none of these outcomes: it changes
    only whether the target is forgotten (claim 17) and *guard* against *area guard* (T2).

    An aircraft standing on its pad when the pad is sold is on the ground, so it does not crash.
    T2's ground row makes it forget its target and destination and selects *guard* (*area guard*
    in the computer case). With ammunition left that *guard* is queued. With none, T2's
    after-the-row entry makes a new pad search first. When it finds a pad, the aircraft is given that pad and *enter*. When
    it finds none and the type is pad-bound, the aircraft stays on the ground as it is: it does
    not crash, since a crash starts only above the ground (claim 9). The ground row's forgetting
    of target and destination stands, its *guard* is not queued, and its current behaviour does
    not change (claim 18).
    - Backing: `measurable` (scenario AC19 for a lost pad, its sale run for a crash with
      ammunition left; AC29 part f for no pad from the start,
      every contact place taken, the crash's health, fall and impact, a sale while it stands on
      the pad with ammunition, the score screen's counts of units lost and destroyed, and the
      absence of debris and of the explosion-list animation, each where the type has them).
      `unmeasured` for an aircraft in flight whose in-the-air answer is *no* when it falls back
      with its ammunition spent (a new pad search, and the crash when none is found): no set-up
      here ends a pad-bound aircraft's run below that height. `unmeasured` for the tag raised at
      the crash start, which no fixture here records, for a
      refusal that ends the contact without a sale or a
      destruction, since no set-up is known that makes a pad do that during an approach, and for
      the case with a behaviour queued at that refusal, which is open (Open questions).
      `unmeasured` for a sale while it stands on the pad without ammunition: its first service
      step can come in its touchdown tick (claim 26), so no normal input sells the pad before
      that step. `measurable` (AC29 part i) for a planted bomb kept through the crash start,
      removed without effect at impact, and set off by a detonate order during the fall.
      `unmeasured` for a bomb whose timer comes due during the fall, and for its order against
      the impact in the impact tick: no normal input places the crash start against the bomb's
      due tick.
    - Confidence: high for the crash itself (T2's crash row, as committed, and AC19's lost-pad
      runs), for a type that has dock types, with no building
      of its dock types, with every such building full or not accepting, and after a sale of its
      pad without ammunition left; a pad-bound type with no dock types is rated in claim 45. How
      the crash starts and what it counts are not part of that high. Medium for health 0 at the start and the dead fall,
      until AC29 part f runs, for the counts and the absent killing-hit effects, for the impact, as
      S-0312 claim 16 rates its non-rocket impact rules, for the sale while it stands on the pad,
      for its unchanged behaviour without ammunition, for the in-flight case whose in-the-air
      answer is *no* (`unmeasured`), and for the crash with ammunition left after a sale, until
      AC19's sale run runs. Low for the tag raised at the crash start and
      for the crash and the destination cleared at a refusal without a sale (both `unmeasured`).
      Medium for the planted bomb's handling, until AC29 part i runs, and for its timer during
      the fall (`unmeasured`)

### Tracking as a flying object

47. **When an aircraft is a tracked flying object (Q-1845).** S-0303 Terms define the *tracked
    flying objects* and their *sectors* (F7) that area damage and the airborne scan of S-0309
    claim 9 search, and leave to the movement spec the events that start and end tracking and that
    change the sector. For an aircraft they are these.

    (a) **Start.** While it is not tracked, an aircraft becomes tracked, at the end of its
    current sector's list (S-0303 claim 11):
    - when its take-off starts (claim 8), that is, when it accepts a destination while on the
      ground or landing. This happens at the height it has at that moment, normally 0 leptons,
      before it rises. Leaving a pad is such a take-off: the aircraft accepts its destination
      while resting on the pad (claim 8). An aircraft that is already tracked, such as one taking
      off again from a landing or after a refused touchdown (claim 7), is not added a second
      time.
    - when it is placed on the map (claim 1) with its in-the-air answer (Terms) *yes* and its
      type's counts-as-aircraft flag set (Import mapping). An aircraft placed on the ground,
      from a factory, by the map or by a carrier's launch inside the usable area (claim 35), is
      therefore first tracked when its take-off starts. One placed at its cruise height, as a
      visiting aircraft or outside the usable area, is tracked from its placement when that
      height is at least 208 leptons.
    - at a sector update of (c), in the cases (c) states.

    A destination given while the aircraft is under an effect that disables vehicles, while it
    is paralysed, while it is being erased or warped by a chrono weapon, or while its locomotor
    is switched off is ignored. No take-off starts, so the aircraft does not become tracked.

    (b) **End.** An aircraft stops being tracked:
    - at touchdown (claim 7), when its landing ends at the landing height, on open ground and
      on a pad alike. Starting to land does not end it, and a refused touchdown keeps it.
    - at a crash impact (claim 9), when the fall brings its height below 1 lepton.
    - when it leaves the map in any way: removal from the game (claim 10 included), going
      inside a carrier, a transport or a building, and removal after death.

    Nothing else ends it. An aircraft stays tracked while it hovers, waits, circles or attacks
    in flight, while its speed fraction is 0, and while it falls, dead or with its locomotor
    switched off, until the impact. Whether a dead object is a candidate of a search is S-0303's
    and S-0309's. A landed aircraft resting on the ground or on a pad is never tracked. It is
    listed in its cell layer instead (S-0209 claim 13). Since tracking starts with the take-off
    (a), an aircraft is both tracked and listed in a cell layer from the start of its take-off
    until its listing height becomes positive, the dual membership of S-0303 claim 6; this
    overlap stands or falls with the start at take-off and carries its rating (below). At an
    accepted touchdown the listing returns and tracking ends in the same update, so such a landing
    makes no overlap. A refused touchdown lists the aircraft again in its cell layer while it
    stays tracked (claim 7), so it does make one.

    (c) **Sector update.** A tracked aircraft's sector is brought up to date at the start of
    each of its movement updates, before that update's forward step (claim 4), when it is in
    another cell than at its last start of tracking or sector update and its speed fraction
    carried from the previous update is not zero. When its current cell lies in another sector
    than its list's, it leaves that list and joins the end of its current sector's list
    (S-0303 claim 11); otherwise its place in the list does not change. This has three
    consequences.
    - The sector follows the position one movement update late. An aircraft whose step takes it
      into another sector moves to that sector's list at its next movement update; a search made
      in between finds it in the old sector.
    - No update happens while the carried speed fraction is 0. An aircraft that stops just after
      crossing into another sector, or changes cell only by claim 4's descent nudge while its
      fraction is 0, stays in the old sector's list until it moves on with a positive fraction.
    - An aircraft in flight that is not tracked, such as one placed in the air below 208
      leptons or one whose type lacks the counts-as-aircraft flag, becomes tracked at its first
      movement update with a positive carried fraction, at the end of its current sector's list,
      when its cell then lies outside sector (0, 0) of F7. While its cell lies in sector (0, 0),
      cell (0, 0) itself included, it is not listed. It joins a list only at the first such
      update after it has moved into another sector, at the end of that sector's list
      (`quirks.aircraft_corner_sector_untracked`). An aircraft type that keeps the
      counts-as-aircraft flag and has a cruise height of at least 208 leptons never reaches this
      case.

    (d) **Order.** Several aircraft that start being tracked in one tick, or move into the same
    sector in one tick, join that sector's list in the order of the events, within the tick's
    order (S-0203), that start their tracking or move them. Nothing sorts them. Only their order
    within one sector's list is observable, through S-0303 claim 11's hit order and S-0309 claim
    9's equal-score order.

    (e) **Anti-air consequence.** The airborne scan of S-0309 claim 9 reads these objects, and
    of those it finds it looks only at the ones on the map and not in the ground render layer
    (S-0309 claim 9; claim 7 here: a height of at most 0). A building or unit whose automatic
    search looks for aircraft therefore finds an aircraft flying on a move order without first
    being damaged by it: from the first search after the aircraft is both tracked and out of
    the ground render layer, while its sector is among those the search reaches. An aircraft
    that has just started its take-off is tracked but not yet looked at. Whether a weapon may
    then fire at it, and so whether the search takes it as a target, is S-0301's and S-0309's.
    A weapon that may fire only at an object in the air waits for its in-the-air answer
    (Terms).

    With S-0303's `quirks.flying_search_sectors` off, (a), (b) and (d) are unchanged. Sector
    updates then change nothing observable (S-0303 claim 11), and (c) decides only when an
    aircraft in flight that is not tracked starts being tracked.
    - Backing: `measurable` (scenario AC30: parts a, b, d and e back tracking in flight after a
      take-off, bounded on both sides, the hover and the anti-air consequence of part (e); part c
      and part i (4) that tracking holds after the landing starts; part i (1) to (3) the end at
      touchdown, that a landed aircraft is not tracked, and the pad as open ground, through the
      count of area-damage entries, which shows that tracking has ended by a detonation at least
      one search interval after touchdown; part h (1) tracking from placement in the air with the
      counts-as-aircraft flag set; part h (2) and part j the absent default of that flag; part f
      the stop after a sector crossing in part (c); part g the untracked aircraft and sector
      (0, 0)).
      `unmeasured`, each for its own reason:
      - tracking from the start of the take-off, below 208 leptons: a tracked aircraft that low is
        found only by area damage from a detonation above the floor and by searches whose weapons
        may fire at it there, and no fixture here sets either up during the climb; AC30 parts d
        and e bound the start from both sides.
      - the end at a crash impact: the only effect that follows the end in the same moment is the
        impact's own damage, and the aircraft is destroyed by the impact either way.
      - staying tracked while dead and falling: the area damage and the searches that read the
        tracked objects pass over a dead object, so no observable shows the membership.
      - the end on going inside a carrier, a transport or a building: an object inside is off the
        map, and S-0309 claim 9 looks only at objects on the map, so the end itself does not show
        through a search.
      - a refused touchdown keeping the aircraft tracked: no fixture here forces a refusal at the
        moment of touchdown.
      - the ignored-destination list: no fixture here applies one of those effects to a landed
        aircraft at the moment it gets a destination and then separates the missing take-off
        from the effect's own hold on its movement.
      - the one-update lag of (c): it shows only between the aircraft's own movement update and
        its next one, and no fixture places a search in that window.
      - the order of (d): no fixture here makes two aircraft start being tracked in one tick in
        one sector with distinguishable hits.
    - Confidence: high that a landed aircraft at rest is not tracked, on open ground and on a pad
      alike, whether it landed after a move order or the map placed it on the ground (AC30 part
      i (1) to (3), through area damage). Medium for tracking in flight after a take-off, for the
      hover and for tracking from placement in the air with the flag set: AC30 parts a, b, d, e
      and h (1) see them only through the airborne scan of S-0309 claim 9, which that claim rates
      medium, and h (1) is an attempt that may be void. Medium for the end falling at the
      touchdown update itself: part i shows that tracking has ended by a detonation at least one
      search interval after touchdown, and parts c and i (4) that it holds after the landing
      starts, but no part pins the update. Medium for the anti-air consequence of part (e), which
      can be no firmer than the airborne scan and its ground render layer rule in S-0309 claim 9,
      and for the absent default of the counts-as-aircraft flag (AC30 h (2), an attempt, and
      part j, which rests on S-0004 claim 11's ranking); medium for each `unmeasured` point of (a)
      and (b) above, the overlap at take-off included; medium for part (c) and for part (d).

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
> *x*′ = trunc(*x* + cos *θ* × *s*), *y*′ = trunc(*y* − sin *θ* × *s*), the product and the sum
> each formed in floating point and the sum truncated toward zero once. This is the computation
> of S-0208 F3 with step length *s*, and the rounding of each product and sum follows S-0205
> claim 5: under the default (`quirks.chop_rounding` on) each is rounded toward zero. Facing
> 16384 moves along +x, facing 0 along −y, both by exactly *s*. The new coordinates are
> integers and no fractional remainder is carried into the next step.
>
> Because map positions are positive and each product of a table value with *s* is exact,
> under the default each new coordinate is exactly the old one plus its component rounded
> down: *x*′ = *x* + ⌊cos *θ* × *s*⌋ and *y*′ = *y* + ⌊−sin *θ* × *s*⌋. A negative component of
> any size, however small, therefore costs a whole lepton. S-0105 F1's table has two tiny
> nonzero entries, below 10⁻¹⁵ in magnitude, where the exact value would be zero. They give
> a tiny negative x component at body facings 32767 to 32772, just past due south, and a tiny
> negative y component at body facings 49148 to 49155, around due west. A step of any length *s* ≥ 1 at those facings moves the
> aircraft one lepton west (32767 to 32772) or one lepton north (49148 to 49155) in addition
> to its full step along the other axis. A long flight at such a facing drifts one lepton
> sideways on every step. With *s* = 1 this gives, by body facing *F*:
>
> | *F* | Change (x, y) |
> |---|---|
> | 0 to 16379 | (0, −1) |
> | 16380 to 16386 | (+1, 0) |
> | 16387 to 16390 | (0, −1), the table's negative-angle bias giving a positive sine |
> | 16391 to 32766, between east and south | (0, 0): no motion (`quirks.aircraft_unit_step_stall`) |
> | 32767 to 32772, just past due south | (−1, +1) |
> | 32773 to 49147 | (−1, 0) |
> | 49148 to 65531 | (−1, −1) |
> | 65532 to 65535 | (0, −1) |
>
> So between north and east a 1-lepton step moves one lepton north, between south and west
> one lepton west, and from just short of due west to north one lepton on each axis; just
> past due south it moves one lepton on each axis. For *s* ≥ 2 at least one coordinate
> always changes. With `quirks.chop_rounding` off, each sum is rounded to nearest instead,
> and at every position of at least 1024 leptons the two near-zero components are lost: the
> rows for 32767 to 32772 and 49148 to 49155 become (0, +1) and (−1, 0), and steps of every
> length at those facings take no sideways lepton. Every other row is the same in both
> settings. With `quirks.aircraft_unit_step_stall` off, a forward step
> with *s* ≥ 1 that would change neither coordinate instead moves 1 lepton along the axis
> whose component has the larger magnitude, in that component's direction, taking x on equal
> magnitudes. The descent nudge and the off-map fallback (claim 4) use F3's trigonometry with
> lengths 5 and 64 and are not affected by the switch.

> **F4 Speed fraction.** This formula gives ordinary destination-dependent slowing under
> claim 5, not its fixed-full-speed exception. Let *f*₀ be the current fraction before slowing,
> *d* the non-negative integer raw planar point distance in leptons, and *S* the signed
> integer slowdown distance in leptons. For *S* ≠ 0, *τ*₀ = min(1, *d* / *S*), with
> floating-point division, not integer division. For *S* = 0, *τ*₀ = 1 for both positive
> and zero *d*, without evaluating a quotient in this formula. Thus zero slowdown distance
> causes no distance-dependent halving: at positive *d* the fraction approaches 1; at
> *d* = 0 the distance clamp makes it 0 before the adjustment raises it to 0.1.
> These are the fractions before subsequent guidance or landing rules, not a promise of
> eventual arrival. For *τ*₀ < 0.1 and *d* < 86, *τ* = 0 and
> *f*ₕ = *f*₀ × 0.5. Otherwise *f*ₕ = *f*₀ and *τ* = max(0.1, *τ*₀).
> Let *d*ₙ = *d* / (1 lepton): the dimensionless numerical magnitude of the distance in
> leptons, converted exactly from its integer value to double precision. The distance-clamped
> fraction is *f*꜀ = min(*f*ₕ, *d*ₙ), a dimensionless fraction. This compares numerical
> magnitudes, not a physical distance with a speed: it introduces no conversion to cells or
> distance travelled per tick. Since *d* is a non-negative integer distance and
> 0 ≤ *f*ₕ ≤ 1, equivalently *f*꜀ = 0 at *d* = 0 leptons and *f*꜀ = *f*ₕ at every
> *d* ≥ 1 lepton. The pre-adjustment fraction *f*ₚ is 0.05 when *τ* = 0, *f*꜀ = 0 and
> *d* > 0, and *f*꜀ otherwise.
> The adjusted fraction is min(*τ*, *f*ₚ + 0.1) when *f*ₚ < *τ*, max(*τ*, *f*ₚ − 0.1)
> when *f*ₚ > *τ*, and *f*ₚ when equal. The adjusted fraction is the value defined by
> the preceding equations. Only that resulting fraction is specified as observable; the
> intermediate evaluation order is not. *f*₀ is the fraction that the same tick's
> forward step (F3) has just used, and *d* is measured from the position after that step; the
> adjusted fraction is the one the next tick's step uses. When the creep restart applies,
> *τ* = 0 and the adjusted fraction is max(0, 0.05 − 0.1) = 0, so the restart never changes
> the outcome. Claim 5's subsequent approach replacement does not recalculate this adjusted
> fraction, and the next tick's calculation of *τ* replaces that target before its adjustment.
> All fractions and their arithmetic use double precision with S-0205 claim 5's rounding
> mode; there is no integer conversion of a fraction. An adjusted fraction calculated with
> *τ* ≥ 0.1 is itself at least 0.1, so a step taken after it is at least ⌊*V* / 10⌋ leptons.
> F4 itself never refers to the heading, and the heading-dependent hover of claim 5 comes
> only from F3's truncation of each coordinate.

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
> When the own-cell search below 256 leptons finds no cell, the result is claim 41's
> placeholder at map cell (0, 0), not either fallback (`quirks.landing_own_cell_no_result`).
> Claim 41 supplies caller settings and fallback-object preference; S-0212 claim 25a supplies
> footprint qualification, unchanged result cells and candidate choice.

> **F9 Pad distance.** *D*² = (*x*ₚ − *x*ₐ)² + (*y*ₚ − *y*ₐ)² between the centre of the pad and of the
> aircraft, in leptons, as a signed 32-bit integer that wraps on overflow (S-0205 claim 1); height is
> ignored. A candidate replaces the best so far when its *D*² is smaller (signed comparison)
> (`quirks.pad_distance_overflow`).

> **F10 Service cadence.** *P* = ⌊*r* × 900⌋ ticks, *r* the reload rate in game minutes read as a
> double (S-0003), the product in double precision truncated toward zero (S-0205,
> `quirks.chop_rounding`).

> **F11 Repair cost (claim 27; S-0314 claim 5).** Let *Σ* be the type's strength in hit
> points, *σ* the imported game-wide repair step in hit points, *C* the type's cost in
> credits, and *p* the dimensionless repair percentage as a fraction. *Σ*, *σ* and *C* are
> integers; *p* is double precision. The cost uses no house or country cost factor.
>
> **Original:** *q* = trunc(*Σ* / *σ*), without a minimum on either *σ* or *q*.
> **Our engine:** *σ*′ = max(1, *σ*), *q* = max(1, trunc(*Σ* / *σ*′)), the departure
> owned by S-0314 claim 5. The two agree when 1 ≤ *σ* ≤ *Σ*.
> In either case, *c*₀ = trunc(*C* / *q*), and cost = max(1, trunc(*c*₀ × *p*)) credits.
> Both quotients are integer division truncated toward zero. The final product uses double
> precision with S-0205 claim 5's rounding mode, then truncation toward zero to integer
> credits. The minimum cost is 1 credit. Zero divisors stop the original; our engine's
> guards avoid them and the importer warns as specified by S-0314 claim 5.

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
| Approach turn-in distance | 512 | leptons | table T1 (aiming facing only) |
| Guidance aiming distance | 256 | leptons (distance < 256) | claim 42 |
| Strafer body re-aim window | 768, 8192 | leptons (at most), facing units (at most) | claim 42 |
| No-result landing cell | (0, 0) | map cell coordinates | claim 41 |
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
"Ammunition left" counts the round of the run still to be spent. The run's facing requests are
made before that update's movement tick; claim 42 gives the guidance requests that follow them,
which decide the direction of travel. Firing attempts in this table use claim 20 while a
first passenger exists, rather than ordinary-shot effects; the run-level pending round and
processing delays are unchanged. Drain attempts use S-0608 claims 1 and 16 instead of
ordinary-shot effects. Claim 12's target-clearing departure overrides every firing row:
no remaining burst attempts or scatter, *done*, 1 tick; any pending round is retained. Every
processing of *approaching* first spends a pending round and turns the run lock off, whatever
else it then finds; its rows below apply after that.

| State | Event | Result |
|---|---|---|
| start | has a target | *choosing*; 1 tick |
| start | no target | *done*; 1 tick |
| *choosing* | run lock off; the pending round is spent; target and ammunition left | the firing spot becomes the destination; *approaching* when it exists, else *done*; standard delay |
| *choosing* | no target or no ammunition | *done*; standard delay |
| *approaching* | no target or no ammunition, after the spend above (a strafer arriving from *strafe pass 5* included) | *done*; 1 tick |
| *approaching* | strafer, target at or beyond weapon range | keep flying at the destination (row "steer") |
| *approaching* | strafer, target within range; or fighter; or other aircraft whose current speed fraction is exactly 0 (claim 42; its zero-step switch-off alternative also applies here) | *firing*, the destination kept; 1 tick |
| *approaching* (steer) | no destination | *choosing* |
| *approaching* (steer) | distance to destination ≥ 512 | request the aiming facing toward the destination; 1 tick |
| *approaching* (steer) | distance < 512 | request the aiming facing toward the target (the body is not turned); if distance ≥ 16, 1 tick; else the destination is cleared, the flight point kept, and *firing*; 1 tick |
| *firing* | no target or no ammunition | *done*; 1 tick |
| *firing* | non-strafer requests body and aiming facing toward the target (claim 42 decides which request lasts); the firing check answers *ready* | full burst fired, a round becomes pending, scatter. Strafer: *strafe pass 2*, run lock on, one rate of fire. Otherwise fighter: run lock on, *choosing* (or *done* with no ammunition), one rate of fire. Other: *refiring*, 1 tick |
| *firing* | answer *not facing* | no ammunition: *done*. Strafer or not close enough: *choosing*. Else fighter: *firing*; other: *choosing* with the re-approach flag on, *firing* with it off. 45 ticks for a strafer, else 1 |
| *firing* | answer *rearming* | 1 tick |
| *firing* | answer *cloaked* | uncloak; 1 tick |
| *firing* | any other answer | no ammunition: *done*; strafer: 1 tick; other: *refiring* |
| *refiring* | no target | *done*; 1 tick |
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
| current behaviour is *retreat*, *paradrop approach*, *paradrop overfly*, *spy-plane approach* or *spy-plane overfly*, and it is not a strike aircraft | unchanged |
| queued behaviour is *paradrop approach* or *spy-plane approach* | that behaviour starts now |
| on the ground, visiting, with passengers | *unload* (in a team: *guard*) |
| on the ground, visiting, no passengers, not in a team | forgets target and destination; *retreat* |
| on the ground, otherwise | forgets target and destination; *guard* (*area guard* for an armed aircraft of a computer-controlled house outside a team) |
| in flight, with passengers | destination: landing spot F8; visiting: *unload* (in a team: *guard*); else *move* |
| in flight, unarmed, in a team | unchanged |
| in flight, armed, visiting, no ammunition | forgets target and destination; *retreat* |
| in flight, armed, visiting, ammunition left, not in a team | *hunt* |
| in flight, armed, and either *attack* is queued, or *attack* is current with ammunition left and a target | *attack* |
| in flight, armed, and the in-the-air answer is no, or it is already flying to its *enter* destination, or it has no dock types, or it is outside the usable area in a team | destination kept; *guard* (*area guard* for an armed aircraft of a computer-controlled house outside a team) |
| in flight, armed, a pad is found and grants contact | destination: the pad; *enter* |
| in flight, armed, no pad, pad-bound | crash (claim 46) |
| in flight, otherwise | destination: landing spot F8; *move* |
| after the row: no ammunition, armed, no contact | a new pad search; found: destination the pad, target forgotten, *enter*; none and pad-bound: the row's next behaviour is not given and the current behaviour stays (target and destination stay as the row left them); in flight the aircraft crashes (claim 46), on the ground it does not (claim 9) |
| after the row: holds a contact | *enter* |

Claim 18 rates the computer-controlled *area guard* that T2's rows name. The after-the-row
entries follow every row from the fourth on, the visiting rows included. So a pad-bound
visiting aircraft in flight with no ammunition and no contact crashes when its pad search finds
no pad, instead of taking the *retreat* that its row names (claims 45 and 46); with a pad found
it gets that pad and *enter*.

**T3. Pad servicing.** See claim 26. Aircraft events on the pad:

| Aircraft state | Event | Result |
|---|---|---|
| flying to the pad (*enter*) | pad accepts, contact held | lands, put on its docking position; *sleep*, *guard* queued |
| flying to the pad | pad refuses or is gone | destination cleared and falls back to rest (claim 46): always when the pad is gone; on a refusal, when no behaviour is queued, Open question 19 otherwise |
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
Between an attack run's end and the next pass, the slot aircraft's flight is claim 35's
interval, not a row of this table. A visiting slot aircraft in *retreat* drops the *move* and
*attack* that these rows queue, keeping their target and destination changes (claims 34 and 35).

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

- **Drain clears an aircraft's object target** (departure, claim 12). Cancel the remaining
  burst attempts and that pass's scatter request, keeping the admitted pass's pending round.
  For an aircraft that remains in play, *done* is due after 1 tick. Drain-link effects remain
  S-0608's. No crash-reproduction switch or importer warning applies.
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
- **Repair-cost divisors** (departure, S-0314 claim 5). For positive strength, an imported
  repair step of 0, or one whose absolute value exceeds strength, makes a divisor in the
  original's F11 zero and stops it with a program error. Our engine uses F11's guarded
  formula for every pad repair cost, with an importer warning for the guarded inputs.
  This also changes defined original outcomes: with a negative repair step whose absolute
  value is at most strength, non-negative type cost and non-negative repair percentage,
  the original charges 1 credit but still restores at least 1 hit point. Our engine instead
  computes cost using a repair step of 1; that charge can exceed 1 credit. This is the same
  departure already specified by S-0314 claim 5, not a pad-only exception or quirk.
- **A pad with no docking-place key** has one contact place and adds the absent default of
  the docking-place count to the pad total (Import mapping); a key set to 0 also gives one
  contact place but adds 0.
- **Endless ordinary refire** (`quirks.aircraft_endless_refire`, default on). With the
  re-approach flag off, an ordinary non-fighter, non-strafer repeats burst-plus-single-shot
  cycles without spending its pending round. Off: end each such cycle as one run, spending
  its round and selecting a new approach regardless of that flag.
- **Own-cell landing search with no result** (`quirks.landing_own_cell_no_result`, default
  on = original). Claim 41: the landing destination becomes the placeholder at map cell
  (0, 0), which the aircraft discards and requests again, possibly heading for that map corner.
  Off: treat the failed own-cell search as no successful building choice and continue with the
  other-object fallback, in which the chosen building counts like any other own-house object,
  and finally the aircraft's own cell.
- **Slow ordinary attacker never fires** (`quirks.aircraft_zero_step_attack_stall`, default
  on = original). With converted speed below 10, an ordinary attacker (claim 42) can hover
  with a positive speed fraction whose forward step truncates to 0 leptons, at least 16
  leptons from its firing spot; it then never reaches *firing*. Off: in *approaching*, a
  positive current fraction with F3 step length 0 leptons also permits transition to
  *firing*, retaining the destination and using the same 1-tick delay as the zero-fraction
  transition. The firing checks still apply. No movement or slowing formula changes, and
  *move* and pad landing keep their existing completion rules. This is independent of the
  one-lepton coordinate-stall switch below: a length-1 step is not a length-0 step, even
  when both coordinates stay unchanged. The off alternative is a proposed design rule,
  not an oracle measurement.
- **One-lepton step between east and south** (`quirks.aircraft_unit_step_stall`, default on =
  original). A forward step of exactly 1 lepton at a body facing between east and south (F3's
  facings 16391 to 32766) changes neither coordinate, because each coordinate is truncated and
  no remainder is kept. An ordinary attacker with converted speed from 10 to 19 whose step
  falls to 1 lepton outside claim 5's halving zone at such a facing hovers there with a
  positive fraction and never fires (claims 5 and 42); the same approach from another
  direction stops at fraction 0 and fires. Any aircraft approaching a destination under
  ordinary slowing hovers the same way, so a *move* or a return to a pad can stop short of
  landing (claim 6). Off: a forward step of at least 1 lepton that would
  change neither coordinate moves 1 lepton along the axis of its larger component, in that
  component's direction, x on equal magnitudes (F3); every other step is unchanged. This
  alternative is a design rule, not an oracle measurement.
- **A crash drops a planted bomb** (`quirks.crash_drops_planted_bomb`, default on = original).
  A bomb planted on a pad-bound aircraft that crashes at claim 46's fall-back without going off
  during the fall is removed at impact with no effect, so its damage is lost. Off: at impact, in
  the impact processing and before the death weapon, the bomb goes off by S-0519 claim 5 at the
  aircraft's position, with its attacker and attacking house as S-0519 claim 5 gives them.
  Nothing else in the crash changes. This alternative is a design rule, not an oracle
  measurement.
- **Damage smoke consumes game-state randomness** (`quirks.aircraft_smoke_scenario_draw`,
  default on = original). Claim 37's damage-smoke chance uses the scenario stream, including
  updates that show no smoke. This can change later game-state random outcomes, so it is not
  a presentation-only bug. Off: disable this damage-smoke emission entirely. Take no
  damage-smoke chance draw on either stream and create no damage-smoke animation, even when
  claim 37's eligibility holds. Other aircraft draws and effects are unchanged. The default
  retains the higher smoke chance for dead aircraft. This alternative is a design rule,
  not an oracle measurement.
- **Untracked aircraft in sector (0, 0)** (`quirks.aircraft_corner_sector_untracked`, default on =
  original). An aircraft in flight that is not tracked and whose cell is in sector (0, 0) of
  S-0303's F7, cell (0, 0) itself included, is not listed until it moves into another sector, so
  neither area damage nor the airborne scan finds it there through the flying-object search
  (claim 47 (c)). Off: at its first movement update with a positive carried speed fraction it
  joins the end of sector (0, 0)'s list, like an aircraft in any other sector.
  This alternative is a design rule, not an oracle measurement.

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
| none | map terrain and high-bridge presence | aircraft deck mark and the 416-lepton deck reference (claim 43) | fixed runtime transitions; terrain and bridge inputs follow S-0106 claim 2 and S-0211 claim 15 | no mark is granted merely by an aircraft type or a pad |
| none | actual aircraft position and high-bridge presence | ordinary object-present marking and clearing surface (claim 44) | fixed runtime rule, not a separately imported boolean; terrain and bridge inputs as above | no aircraft-type override and no ground copy of an ordinary deck-rest flag |
| `Ammo` | rules, type | maximum ammunition *A* | integer | −1 (unlimited) |
| `Selectable` | rules, object type | visiting test | boolean | yes |
| `Strength`, `Cost` | rules, type | F11 | integer | S-0302, S-0403 |
| `Spawned` | rules, type | spawned type (F7) | boolean | no |
| `ConsideredAircraft` | rules, aircraft type | counts-as-aircraft flag: tracking from placement in the air (claim 47); other uses belong to the specs that read it | boolean | yes for an aircraft type |
| `InitialAmmo` | rules, type | starting ammunition | S-0301 claim 10 | −1 (use maximum) |
| `Spawns`, `SpawnsNumber`, `SpawnRegenRate`, `SpawnReloadRate` | rules, type | S-0506 Import mapping | — | S-0506 |
| `Camera` | rules, weapon | camera weapon (visiting test) | boolean | no |
| `DrainWeapon` | rules, weapon | drain attempts and target-clearing continuation (claims 12 and 15); S-0608 claims 1 and 16 | boolean | no |
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

**Common setup and falsification rule.** Use a purpose-made skirmish map, game speed index 6,
800 × 600 pixels, stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Include one bystander computer player at the
farthest start, isolated from the observed area; apply the glossary's bystander void rule.
Run each variant five times unless stated otherwise. Use scripted normal input and a read-only
per-tick recorder for positions, heights, facings, current and queued behaviours, ammunition,
health, credits, contacts, current and target speed fractions, held height and shot events.
The observed area is the test aircraft's route, its targets and every candidate pad or landing
cell named by the variant; keep the bystander outside that area. Scripted normal input can drive the game and a read-only recorder can collect these logs. No game-state writes, patched executable or injected instrumentation are allowed.

Where stock types or normal orders cannot expose a branch, the scenario explicitly names a
**test-map override**. These are the glossary's named-test-file allowance, not stock-data claims.
Remove the overrides after that variant. Never infer an untested branch from an ordinary run.
Each expected result below is falsifiable: an isolated record that differs in its stated count,
tick, value, destination, behaviour or survival outcome falsifies that part. An unreachable setup
is a coverage gap, not confirmation. Record actual seeds and tick boundaries. Unresolved parts
listed in Open questions are not claimed confirmed by these scenarios.

- **AC1 Placement (claim 1).** Produce a pad aircraft; dispatch a paradrop plane and spy plane.
  Observe a carrier launch. Record the position immediately at placement, before movement.
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
  No reachable non-crashing dead-aircraft descent setup is established here.
- **AC3 Forward motion and slowing (claims 4 and 5).** Fly straight routes in all four map-axis
  directions, then diagonal routes, logging type speed, fraction and coordinates. Repeat with
  named speed-bonus test-map overrides for country, veterancy and per-object speed bonuses:
  expected F3 uses the unchanged type speed in every case, not a bonus-scaled speed.
  Compute F3 with the signed facing and compare every forward step. Use sustained cruise ticks
  to separate the forward formula from claim 4's additional descent step. On approach to a cell destination, log distance
  and fractions around each F4 threshold, including distance 0, and around 128, 512 and 768
  leptons. Expected: the resulting adjusted fraction defined by F4, and claim 5's target
  replacement without a retroactive change to that fraction. The next movement tick's own
  calculation replaces that target (AC24 isolates this). With positive slowdown distance, at *d* = 0
  in ordinary slowing with positive incoming fraction, the adjusted fraction must be 0.
  At post-step distances of 1 lepton or more, F4's numerical distance clamp leaves the
  possibly halved fraction unchanged; do not divide the distance by the type speed or by
  256 leptons per cell. Compare the adjusted fraction with F4 at these distances as well as
  at zero. Record the pre-slowing and adjusted fractions separately; a result differing from F4
  falsifies the resulting-fraction rule, not every observationally equivalent intermediate
  order. Distinguish current from target fractions; a target replacement must
  not retroactively alter the already adjusted fraction. Compare take-off below and above
  half height. Repeat during a strafing run, fighter run and visiting fly-by: the fixed-full-speed
  cases must not follow ordinary slowing, including no distance clamp at a zero-distance
  tick reached with the run lock held. An unreachable zero-distance example is recorded as a
  coverage gap, not inferred from another distance.
  A named zero-slowdown test-map override sets the slowdown distance to 0 on a controllable,
  non-fighter, non-strafing, non-fly-by aircraft. Keep a target and ammunition so that it holds
  height without the fixed-full-speed exception. Record post-step distance and fractions before
  guidance as well as at the end of the tick. Expected at positive distance: target 1 and current
  fraction min(1, incoming fraction + 0.1), without halving. If a post-step distance of exactly
  0 is reached, expect current fraction 0.1 before guidance, not 0 or a non-finite value.
  Retargeting to a stationary object at the aircraft's planar position may expose that boundary.
  Failure to reach it is a coverage gap, not confirmation. A differing fraction, non-finite
  result or program error falsifies the corresponding zero-distance or positive-distance rule.
  Repeat with a named negative-slowdown override of −500 leptons: below 86 leptons expect
  halving and target 0; at or above 86 expect target 0.1, under F4's remaining rules.
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
  Take-off steering follows claim 8, not the deleted descent-stage hypothesis. Log the held
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
  After entry, target-free visitors without either flag leave at the usable edge, other eligible
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
  its target and rests; a strike aircraft with ammunition and no target retreats. In-burst
  draws and the final rearm delay must match S-0301 claims 2 and 4, not one check per burst shot.
- **AC9 Scatter (claim 16).** Have infantry in the target cell of a durable target under a strafing
  run. Record scatter requests and accepted destinations at each firing pass. Expected: a
  request each pass; actual movement is subject to S-0215 claim 19 and may be refused. A shot
  without a corresponding request falsifies the claim; lack of movement alone does not.
- **AC10 Rest, guard, hunt and queue timing (claims 18, 29, 31 and 33).** Exhaust ammunition with
  a free pad, then full pads, then no pads. Expected: reservation and *enter* when available.
  Pad-bound crash otherwise; a non-pad-bound aircraft seeks a landing spot by F8. Named rest-test
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
  the imported attack-reveal radius. A passenger attempt instead follows S-0522 claim 11,
  scenario P7: compare successful and refused drops, log rearm countdown, burst counter,
  ammunition, pending round and scenario draws. Expect no ordinary-shot effects, immediate
  spending only on successful drops, and separate pending spending even after an all-refused
  run. A named burst variant carries fewer passengers than its burst count: remaining attempts
  after the last release use ordinary-shot effects without a new firing check. Contradictory
  effects, countdowns, draws or spending falsify these rules. An ordinary kamikaze shot removes
  its aircraft; a drop attempt does not. Those type variants use the named launch-test override.
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
  Contact persists until departure. Use S-0524 claim 14, scenario C16's repeating pad entered-by
  tag, aircraft destroyed-by-anything tag and read-only raise recorder: expect neither raise
  merely for docking, servicing or departure. Any such extra object-tag raise falsifies this
  rule; account separately for cell tags during approach. For direction, compare the contact's body facing with the
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
  Record health and credits at each service tick. Expected: F11 cost and the repair step, exact
  full-health cap and release when full; insufficient credits cause release with no charge or
  healing. Repeat on a reload pad without the separate repair-building flag. Each different
  charge, health gain, cap or release outcome falsifies the corresponding prediction.
  The named `aircraft-pad-repair` test-map override supplies the following invented inputs,
  with enough credits to pay and no other healing or credit changes in the observed area:
  - Integer-division contrast: strength 11 hit points, repair step 4 hit points, type cost
    101 credits and repair percentage 0.5. At the first damaged, full-ammunition service step,
    expect *q* = 2, *c*₀ = 50 and a 25-credit charge, with 4 hit points restored unless capped.
    A different charge, including 18 from retaining both quotients in floating point,
    falsifies the integer divisions.
  - Negative-step contrast: strength 300 hit points, repair step −3 hit points, type cost
    2400 credits and repair percentage 0.5. Expect the original to charge 1 credit and
    restore 1 hit point per non-final step. A different charge or gain, or a program error,
    falsifies that prediction. Replay separately on our engine: its F11 guards instead
    give *σ*′ = 1, *q* = 300, *c*₀ = 8 and cost 4 credits, still restoring 1 hit point,
    with an importer warning. Charging the original's 1 credit there fails the departure.
  - Zero-divisor contrasts, separate runs: strength 11 hit points and repair steps 0, 12
    and −12 hit points. Keep the type cost and percentage of the integer-division contrast.
    Expect the original to stop with a program error at the first attempted repair cost.
    Successful payment or healing falsifies that outcome. Our engine must instead warn
    and continue: cost 4 credits for step 0 or −12, and 50 credits for step 12. These
    guarded outcomes are engine tests, not oracle measurements.
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
  is obtainable. Sale run with ammunition left: a pad-bound aircraft of a human-controlled
  house with some but not all of its rounds is ordered onto its pad, the house's only pad, and
  the pad is sold while the aircraft is still at least 10 cells from it. Record its behaviour,
  destination and health each tick. Expected: at its fall-back it crashes (health 0, falling),
  as claim 46 gives with ammunition left. Falsified if it rests in *guard* or keeps the pad's
  cell as its destination. Record target destruction during a run: the run ends and a pending round is
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
  **Q-1755 parts (a target that leaves the map).** Skirmish on open flat ground, human-controlled
  test house, measurement harness with the random-state recorder. Named test rules: an armed,
  landable aircraft type with dock types, neither strafer nor fighter, ammunition 2; the test
  house has one free pad of its dock types. The test house also has an infantry unit 12 cells
  from the aircraft and a closed transport next to that unit. An enemy house far away keeps the
  game going and takes no part. Order the aircraft, by force-fire, to attack the test house's own
  infantry unit. When the aircraft is two cells from its firing spot (its destination in T1's
  *approaching*), order the infantry unit into the transport. Every order is the test house's
  normal input to its own units. The target is the test house's own unit because the loss has
  no owner, alliance or sensor exception (claim 32), so it stands for any target here. Record per tick for 600 ticks the
  aircraft's target, behaviour, queued behaviour, destination, position, height and ammunition,
  every shot, the tick in which the infantry unit leaves the map, and each draw with the object
  it is attributed to; record the aircraft's targeting timer where the harness can read it.
  - a. The game-wide re-approach flag off (named test rules). Expected: the
    aircraft's target is empty from the tick in which the infantry unit leaves the map, in that
    same tick. In that tick one *U*(4, 8) draw restarts the aircraft's targeting timer when that
    timer had more than 10 ticks left, and none otherwise (when the timer cannot be read, only the
    count of 0 or 1 is recorded and the condition is not judged). No shot, ammunition 2 throughout.
    At the aircraft's next *attack* processing *done*, and 1 tick later, at the processing after
    it, the run end:
    the edge-cell draws of claim 17, then the pad as destination and *enter*, and a landing on the
    pad. No firing-spot draw after the boarding. Any other movement of the aircraft between the
    boarding and the run end is recorded. Falsified by: the target still held after the boarding
    tick, a hold at the firing spot, a new firing spot chosen for the unit, a round spent or a shot.
  - b. Part a with the game-wide re-approach flag on (named test rules). Expected: the same as part
    a. Falsified by any difference from part a's result.
  - c. Part a with the transport ordered to unload 100 ticks after the boarding. Expected: when the
    unit is back on the map the aircraft does not hold it as target; it is already falling back by
    part a's result (flying to its pad, or resting on it) and no *attack* starts against the
    unit. Falsified by the unit becoming its target again, or a run against it.
  - d. Part a with an open-topped transport. Expected: the same as part a. Falsified as part a.
  - e. Part a with the test house's infantry unit of a type that can occupy buildings, and a
    garrisonable neutral building next to it instead of a transport, the unit ordered to occupy it. Expected: the same as part a, from the tick in which the unit goes
    inside. Falsified as part a.
  - f. Part a with a vehicle of the test house, of a type a tank bunker accepts, as the target
    (force-fire) and an empty tank bunker of the test house next to it, the vehicle ordered into
    the bunker. Expected: the aircraft still holds
    the vehicle as target after it is bunkered; the firing check's answers and the shots are
    recorded, and only the retention is judged. Falsified by the target
    being lost when the vehicle is bunkered.
  - g. Part a with a harvester of the test house as the target (force-fire), ordered to dock at the
    test house's refinery when the aircraft is two cells from its firing spot. Expected: the target is kept while the harvester
    is docked (low: claim 32 and Open question 23). A loss at docking would show that a docked
    harvester is off the map; either result settles that question.
  - h. Part a without the aircraft: a ground vehicle of the test house, armed against infantry, is
    ordered by force-fire to attack the unit from 12 cells away, and the unit is ordered into the transport
    when the vehicle is 2 cells short of its weapon's range. Expected: the vehicle's target is empty from the boarding
    tick, with part a's draw condition. What it does next is S-0215's and S-0309's, and is not
    judged here. Falsified by the target still held after the boarding tick.
  - i. A kill while the aircraft waits in *refiring* (claims 12, 15 and 32). Part a's aircraft
    type, with a named test weapon whose burst is 3, whose projectile flies for at least 3 ticks
    to the target, and whose rate of fire is longer than that flight. The target is the test
    house's own infantry unit, of a named test type whose health is below one shot's damage,
    attacked by force-fire; no transport is used. Record what part a records, plus each
    projectile's creation tick and each hit. Expected: three shots in the firing tick, all
    before the first hit; the target killed by the first hit while the aircraft is in
    *refiring* (its firing check answering *rearming*); at the aircraft's next *attack*
    processing *done*, and 1 tick later the run end; ammunition 2 then 1 (one round spent), no
    shot after the kill. Falsified by fewer than three shots in the firing tick, a shot after
    the kill, a spent amount other than one round, or a delay other than 1 tick between the
    processing that gives *done* and the run end.
  - j. A strafer's run after its fifth shot (claim 32, T1). Part a's set-up with a named test
    aircraft type whose weapon strafes and whose projectile reaches the target in fewer ticks than
    F6 gives for that type. The target is a vehicle of the test house, attacked by force-fire, of
    a named test type; no transport is used. Record what part a records, plus each shot and hit.
    Run 1: ammunition 2 and a target whose health only the fifth shot's hit takes away. Run 2:
    ammunition 1 and a target that survives all five shots. Run 3: ammunition 2 and a target that
    survives all five. A run in which the target dies at another shot is recorded and not judged.
    Expected: in every run, F6 ticks after the fifth shot, the aircraft's ammunition drops by one
    (the run's round spent). In runs 1 and 2, 1 tick later the run end of part a (the edge-cell
    draws, then the pad as destination and *enter*), with no sixth shot and no new firing spot. In
    run 3 the run goes on (it flies at the target, or fires again when within range), with no run
    end in that tick. Falsified by: no ammunition drop at that tick, a drop other than one round,
    a sixth shot or a new firing spot in runs 1 or 2, a delay other than 1 tick between the drop
    and the run end in runs 1 or 2, or a run end in run 3 at that tick.
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
    using the aircraft's own movement settings. Add pairs of equal-integer-distance anchors,
    first of the same kind and then of different kinds. Produce them in a recorded order,
    move them to separate symmetric positions whose successful nearby-search results differ,
    and trigger rest at the same recorded aircraft position and height. Reverse creation
    order in a fresh run while retaining those positions and search conditions. Expected:
    the earliest-created successful anchor wins, even when created later than an unrelated
    object since removed. Move the earlier anchor away and back, or board and unload it
    before restoring the setup: later entry onto the map must not make it lose its tie.
    Record creation, entry and search ticks, integer spatial distances, qualifying anchors
    and the selected destination. A later-created successful equal-distance anchor winning
    falsifies the preference. Void a contrast if movement or visibility changes qualification.
    For the 3 × 3-building branch, use the named rest-test override with dock types and no
    accepting pad. Record the chosen building, candidate footprint and destination cell.
    Apply S-0212 claim 25a's P28 variants. Expected: a qualifying forward 3 × 3 rectangle
    with its top-left cell returned unchanged, and tick-modulo choice among preferred cells.
    A middle-cell shift or refusal solely from the aircraft's own mobility settings falsifies
    that part. Own-cell 1 × 1 replacement instead uses reference-distance choice; compare
    equal-distance candidates at different search ticks to distinguish its stable collection
    tie from the no-reference searches' tick-modulo choice. No search takes a random draw.
- **AC24 Ordinary attacker close approach (claims 4, 5, 40 and 42; Q-0651, Q-0653).** Named test map
  `aircraft-close-approach`: a test-map override gives a landable, non-pad-bound aircraft type an
  ordinary attack: a primary weapon whose projectile has a turn rate above 1 and is visible,
  no fighter or fly-by flag, several rounds. One durable stationary enemy vehicle stands in open
  level ground; the aircraft starts parked at a distance of at least 20 cells. Issue one attack
  order and log per tick, with the read-only recorder: position, body facing, aiming facing,
  current and target fractions, distance to the destination point, behaviour stage, destination
  and flight point, and shot events. Run three type variants: (a) speed field and slowdown
  distance chosen so the converted speed is about 35 and *S* = 500; (b) converted speed about
  12 with *S* = 1000; (c) converted speed about 35 with *S* = 100. Variant (b) is flown from
  several starts about 20 cells from the flight point along one axis and 11 leptons off its
  row or column, measured from the logged flight point (Q-0653): (b1) from the west, north of
  the point's row; (b2) from the west, south of the row; (b3) from the north, west of the
  point's column; (b4) from the north, east of the column; (b5) from the east and (b6) from
  the south, on either side. If the firing spot does not let these starts be set exactly,
  record the actual side distances and compare with claim 5 and F3 computed from the logged
  positions and facings. Expected:
  - in each tick the forward step equals ⌊*V* × *f*⌋ for the fraction logged at the end of the
    previous tick, and the distance used for slowing is the post-step distance.
  - a logged target replacement (0, 0.5 or 0.75) never survives into the next tick's
    adjustment: the next adjusted fraction follows F4 from min(1, *d* / *S*).
  - no tick shows fraction 0.05 at its end, and once the fraction is 0 the position stops changing.
  - (a), (b2), (b5) and (b6): the fraction reaches 0 at a post-step distance below
    min(86, *S* / 10) and above 16 leptons (about 48 leptons in variant a and about 80 to 85
    in the (b) runs), the run moves to *firing* at the next processing with the destination
    still set, and the first shot leaves from that hovering position. In (b2) the aircraft
    drifts north while its steps are several leptons long, stops drifting about 2 leptons south
    of the row when its body facing becomes exactly east (16383), and its 1-lepton steps then
    go east. In (b5), from either side of the row, the westward legs at body facings 49148
    to 49155 move the aircraft one lepton north on every step (F3), so it crosses or leaves
    the row northward until its direction turns south of west; it still reaches fraction 0
    at about 85 leptons and fires, but from about 84 leptons north of the row and about
    18 leptons east of the point, with the body facing just west of south (about 35000).
    With `quirks.chop_rounding` off (engine replay only) it keeps its side: 11 leptons north,
    or about 2 leptons south, of the row. The expectations for variants (a) and (c) hold from
    any of the four approach directions; only the final position and facing differ.
  - (b1), (b3) and (b4): once the forward step falls to 1 lepton (post-step distance about
    166, the first *d* with ⌊*V* × *d* / *S*⌋ = 1), the position stops changing with the
    fraction positive (about 0.166) and steady and the distance and body facing constant, the
    facing lying in 16391 to 32766 (about 16892 in b1, about 29778 in b3 and 32766 in b4,
    where the westward drift has ended 1 lepton east of the column). In (b3) the aircraft
    first drifts one lepton west on every step while its body facing is 32767, and the side
    distance at the hover is about 47 leptons from a start 5120 leptons north of the point
    (45 from 4800, 50 from 5376: it depends on the start distance), not 11. The run never reaches
    *firing* and fires no shot within 2000 ticks. A run that keeps advancing one lepton per
    tick and fires falsifies the truncation of each coordinate without remainder (claim 4,
    F3); a hover in (b2), (b5) or (b6) falsifies F3's per-facing table or the direction
    quantisation of S-0204 claim 8. A (b3) side distance that stays 11 with a hover facing
    near 32076 falsifies the round-toward-zero sum of F3 (that is the result with
    `quirks.chop_rounding` off, which an engine replay must reproduce). In (b1) the side distance stays 11 leptons, because the
    southward component of each step truncates away. A (b1) run at most 2 leptons off the row
    flies exactly east through the 1-lepton distances and does not stall; at 3 or 4 leptons it
    hovers nearer, at about 122 or 163 leptons. Check the logged side distance before reading
    an advancing run as a falsification.
  - (c): the aircraft comes within 16 leptons with a positive fraction, the destination is
    cleared, the flight point remains and the fraction then falls to 0.
  - in every variant the body facing keeps pointing at the firing spot point and never turns to
    the target, including in *firing*; the aiming facing begins turning to the target only once
    the distance to the point is below 256 leptons, not at 512.

  An implementation that turns the direction of travel toward the target at 512 leptons and
  reaches *firing* only below 16 leptons or with no unfinished flight fails this scenario:
  the zero-fraction-hover variants (a), (b2), (b5) and (b6) then never fire. A hover with fraction 0 that is not
  followed by *firing* at the next processing falsifies the zero-fraction test, and a body
  turning to the target before the first shot falsifies the guidance rule. A fourth variant
  (d) with converted speed 7 (raw speed field 3) and *S* = 500 should hover with a positive
  fraction and F3 step length 0 leptons at a distance of at least 16 leptons, and fire no shot
  within 2000 ticks. Fly (d) from each map-axis direction and record its first zero-step
  approach processing. A transition to *firing* or a shot while the fraction remains positive,
  the step length is 0 and the distance is at least 16 falsifies the default zero-step stall
  rule. Failure to reach those conditions is a coverage gap.
  Separately replay (d) on our engine with `quirks.aircraft_zero_step_attack_stall` off:
  expect *firing* at the first approach processing with step length 0, followed by its
  ordinary 1-tick delay and firing checks, with no change to F3 or F4. A retained
  *approaching* state under those conditions fails the design rule. Repeat a length-1
  coordinate-stall variant with only this switch off: it must remain *approaching*.
  These switch-off checks test our engine only, not the original.
- **AC25 Own-cell landing search with no result (claim 41, F8; Q-0650).** Named test map
  `aircraft-no-own-cell`: a house owns one building of the aircraft type's first dock type and
  no free pad. 1 to 3 cells from that building (weighted distance below 256 leptons) lies a
  patch of ground-unreachable or fully occupied cells, for example a cliff-enclosed or walled
  area whose free cells are all occupied. Fly a landable, non-pad-bound, unarmed aircraft of
  that type over the patch and end its order there (for example with a stop order, or by
  removing its move destination), so that it falls back to rest in flight with no pad granting
  contact. If no normal order reaches that fallback over the patch, record a coverage gap. Log its destination object and coordinates, flight point, behaviour
  and random-state draws per update. Expected: the destination becomes the cell position
  (0, 0), not a cell near the building or near another own-house object; it is discarded at
  the end of the same update and given again; a *move* attempt takes claim 40's direction draws
  around cell (0, 0). On a small map where a usable cell lies within 31 cells of (0, 0), the
  aircraft adopts that cell and flies to it; on a large map it heads toward the (0, 0) corner and
  is redirected to the building's spot once its weighted distance reaches 256 leptons. A landing
  spot near the building or another object while the own-cell search still fails falsifies the
  claim. Repeat with the first variant's patch freed: the own-cell spot must then be used.
- **AC26 Sideways lepton just past due south and around due west (claim 4, F3; Q-0654).**
  Named test map `aircraft-axis-drift`: open level ground at least 40 cells across. Give a
  landable, non-pad-bound, unarmed aircraft cruising at full fraction a *move* order to a cell
  20 cells due south of its own cell centre, and separately one 20 cells due west of it. Log
  position, flight point and body facing per tick with the read-only recorder, and read every
  expectation against the logged flight point and facing. Expected: on every forward step
  with body facing 32767 to 32772 the x coordinate falls by exactly 1 lepton while y rises by
  the full step, so the aircraft moves west of the flight point's column until its direction
  leaves that range. When the start lies exactly on that column, the direction is first 32767
  (S-0204 claim 8 (a)). On the westward flight, on every step with body facing 49148 to 49155
  the y coordinate falls by exactly 1 lepton. An x (or y) that does not change on those steps
  falsifies the round-toward-zero sum of F3 and matches F3 with `quirks.chop_rounding` off. Repeat due north and due east as controls: no sideways change.

- **AC27 Aircraft deck mark, hover and departure (claims 43 and 44; Q-0841, Q-0872).** Use the common
  setup with a high bridge over level traversable ground, an otherwise empty landing
  cell, and a human-owned landable aircraft. If necessary, name a test-map override
  making an ordinary flying aircraft landable and not pad-bound; retain its ordinary
  flying movement. Use an interior bridge cell with the neighbour required by S-0105
  claim 40, and a non-bridge control. Fly across the bridge, land on it from above,
  remain at rest, then take off across it. Repeat five times. Record per tick world
  height, terrain surface height, the aircraft's own deck mark, render layer, both
  occupant lists and occupancy flags separately, plus the shadow anchor as in S-0105
  MS-30. Expect no mark on the initial flyover, a mark when descent settles at deck
  height, height 0 and ground rendering at deck rest, and clearing in the first ordinary
  climbing tick. The shadow follows S-0105 claim 40 with that recorded mark, not a
  guessed layer. A fresh requested elevated placement instead follows claim 1.
  For retention above deck, use a named test-map flight-level override equal to the
  deck-relative height reached by a marked aircraft when it crosses from a bank onto
  lower bridge terrain, so it neither climbs nor descends there; record whether such
  a positive-height marked crossing is actually reached. Expect horizontal movement
  alone to retain its prior mark, and hovering without a prior mark never to acquire
  it. Failure to reach that retention state is inconclusive, not evidence that the mark
  is equivalent to ground rendering. Any isolated transition or shadow anchor contrary
  to these predictions falsifies the corresponding rule. Do not write the mark with a
  recorder or assume that a pad can legally be built on the bridge.

  **Occupancy and vehicle variants.** At the same common game speed, record both
  object-present flags separately before descent, at rest, after the take-off order
  and at each movement tick of departure. Use initially clear records and no other
  flag writer. Expected: ground stays clear; deck becomes set at deck rest and is
  clear by the end of the first climbing movement tick, with neither list then
  containing the aircraft. Record world height and the mark as well, so the clearing
  of the aircraft deck mark is not mistaken for a ground occupancy transfer. A ground
  flag set by this ordinary deck landing, both records set, or a deck flag retained
  until a later height threshold falsifies claim 44.

  In separate five-run repeats, keep the aircraft allied and at rest without a
  destination. Drive one ordinary vehicle toward that cell below the bridge, then
  repeat from deck level with a different vehicle and no other blockers. Record the
  entry outcome and whether the cell is entered, not just the chosen route. Expect
  *clear* below and *stationary friendly* on deck from the listed aircraft. Repeat
  deck entry after take-off: expect *clear* once the aircraft has left both lists.
  The read-only recorder and scripted inputs of the common setup suffice; do not
  manufacture flags or remove listings. The lower vehicle being refused solely by
  the resting aircraft, or persistent deck refusal after its listing and flag have
  gone with every other condition unchanged, falsifies the stated consequences.

- **AC28 Drain attempts in aircraft runs (claims 12, 15 and 16).** For the original,
  use S-0608 claim 16's D12 cell-target fixture and its recorder: a named draining-primary
  aircraft variant with finite ammunition, no passengers and ground-target eligibility.
  Force-fire at a cell. Record pass admission, pending-round mark, ammunition, rearm
  countdown and target through the next spending boundary. Expect a pending round at
  initial admission, no immediate per-shot spending or new rearm countdown from the drain
  attempt, and one pending round spent at claim 15's boundary. A rolled-back mark or
  missing boundary spending falsifies the bookkeeping claim. D12 supplies the common
  skirmish settings and repeats. Do not use an object target to test the original fault.

  **Engine-only departure check, not an oracle measurement.** On a small skirmish map,
  give that aircraft a burst longer than one attempt, ammunition for more than one run,
  and a drainable enemy building. Test both a position over its foundation and one outside
  it but within weapon range. Repeat for ordinary, fighter and strafing runs, five runs
  per variant at speed index 2, with only the named test-map overrides active. Use the
  bystander's AI-house counterpart giving no orders and the common void rule. Record attempted shots, target, destination, pending
  round, ammunition, link, scatter requests and processing ticks. Expect exactly one
  attempt before target clearing, no remaining burst attempts or pass scatter, *done*
  due 1 tick later and the pending round spent there while ammunition is positive.
  Attachment succeeds only over the foundation. Neither case restores the target or
  cancels a formed link; the departure itself does not change the destination or rearm
  countdown. Later fall-back effects remain ordinary. Any contrary outcome fails the
  departure check, not a claim about the original. For a separate engine-only fixture,
  initialize an aircraft at a later single-shot pass with one round already pending and
  a drainable object target. Its target-clearing attempt creates no second pending round
  and likewise reaches *done* after 1 tick without scatter. This constructed engine state
  is not an allowed way to prepare an oracle measurement.

- **AC29 After the run (claims 1, 17, 34, 35, 37, 45 and 46; Q-1756, Q-1775, Q-1776, Q-1837, Q-1838).** Skirmish on
  open flat ground, human-controlled test house, measurement harness with the random-state
  recorder. Record per tick each aircraft's behaviour, queued behaviour, destination, position,
  height, ammunition, health and whether it exists, and attribute each draw.
  - a. A stock armed, pad-bound aircraft type with one pad of its dock types attacks a durable
    enemy vehicle 16 cells away until its ammunition is spent. Expected: in the processing that
    ends the run, S-0522 claim 4's draws for one cell on the house's computed edge (two when that
    edge is south), then the pad as destination and *enter*; the aircraft does not turn toward
    the edge. Repeat with the house's construction yard placed so that its computed edge is
    south. Falsified by: no such draws, a different count, or a flight toward the edge.
  - b. Named test rules: an armed, landable aircraft type with no dock types, not pad-bound,
    ammunition 1, with a converted speed of at least 20 so that claim 5's hover cannot stop it
    short, produced or placed for the test house (selectable and landable, so not visiting). Order
    the same attack. Record until 1500 ticks after its last round, 5 runs. Expected: at the run
    end the draws of part a and a destination outside the usable area on the computed edge (on
    flat ground S-0522 claim 4's no-acceptable-candidate case does not arise),
    then *guard* and *move*; at that *move*'s first processing one *U*(0, 7) draw per radius from
    radius 0 up to the first radius with an accepted cell, and that usable cell, a few cells
    from the edge cell, as destination; flight to it, a landing there and rest in *guard*; not
    removed, no crash, no hold above the target or the firing spot. Falsified by: no
    replacement (a flight to the edge cell itself), a different draw count, a landing at or a
    hold above the firing spot or the target, a removal or a crash.
  - c. Part b once with the fly-by and fly-back flags on, and once with part b's named test
    type also made pad-bound by the test rules, with its dock-types list left
    empty, placed by the map for the test house. Expected: with the flags, the same
    result as part b. Pad-bound, with its ammunition spent: a crash in the processing that ends its
    run (as in part f). Pad-bound, with the vehicle destroyed by another unit before it fires:
    claim 37's target-loss draw when the aircraft's targeting timer has more than 10 ticks left
    at the kill, the edge cell kept, no *move* queued, a flight to the edge cell, no landing and
    a hold above it for the rest of the record. Falsified by: a result that differs between flags off and on, or, for the pad-bound
    type, a replacement cell, a landing or no crash as stated.
  - d. Part b with the vehicle destroyed by another unit while the aircraft approaches, before
    it fires. Expected: the same edge destination, edge-cell draws, replacement, flight and
    landing as part b, with its ammunition unspent, and in addition claim 37's target-loss draw
    when the aircraft's targeting timer has more than 10 ticks left at the kill.
  - e. A stock strike called under S-0519 whose aircraft spends all its ammunition. Expected:
    part a's draws at its run end and again at its first *retreat* processing. A different
    count or order of edge draws falsifies claims 34 and 37. A strike whose target is destroyed
    before its aircraft fire is not used: that loss cancels the strike (S-0519 claim 19).
  - f. Part a's aircraft: (1) with its only pad sold during its run; (2) with every contact
    place of its only pad taken by other aircraft of the type standing on it; (3) placed by
    the map for a house with no pad; (4) standing on its pad with full ammunition when the pad
    is sold. Expected: in (1) to
    (3) the crash fall (claim 9) begins in the processing that ends its run, with health 0 on
    its first tick, a fall counter growing by 1 per tick, and at impact its death weapon and no
    demolition detonation; none of the debris of its type's own debris counts (S-0312 claim 4,
    checked when the type has a positive maximum) and no explosion-list animation (checked when
    the type's explosion list is not empty, S-0312 claim 16), at the crash
    start or at impact. At the game's end the score screen shows the test house's units lost raised
    by one for each crashed aircraft and no other house's units destroyed raised for them. In (4)
    no crash, and *guard* on the ground. Falsified
    by: waiting in the air, a landing elsewhere, positive health during the fall, a fall of 3 per
    tick, a demolition detonation at impact, debris of its type's own debris counts, a crash not
    counted as the test house's loss or counted as another house's kill, or a crash in (4).
  - g. Part b's type made not landable (so visiting, Terms), with an enemy house that has no other
    object its weapon can target: (1) ammunition spent on the vehicle; (2) the vehicle destroyed
    by another unit before it fires; each once with no flight flag and once with the fly-by and
    fly-back flags. Expected: in (1) the run-end draws, then *retreat* with a second set of edge
    draws, a flight to that edge cell and removal without dying in the tick its position leaves
    the usable area, with and without the flags; in (2) claim 37's target-loss draw when the
    aircraft's targeting timer has more than 10 ticks left at the kill, *hunt* and a flight toward the run end's
    edge cell, with and without the flags, and, without the flags, removal in the tick its
    position leaves the usable area. With the flags, (2) predicts nothing once the aircraft
    reaches the edge cell (claim 45, Open question 18): the run records its position and
    behaviour each tick from then on, and that record judges nothing here. Falsified by: a
    landing, a hold above the target or the firing spot, a crash, no flight toward the edge cell,
    or no removal in (1) or in (2) without the flags. (3) Cases (1) and (2) again, with no flight
    flag and part g's type also made pad-bound (named test rules). Expected: in (1) a crash
    (health 0, falling) at the run end's fall-back, after the run-end draws, with no *retreat*
    and no flight to a second edge cell; in (2) part (2)'s result, with no crash. Falsified by a
    *retreat* or a removal in (1), or a crash in (2). (4) Part g's type placed by the map for the
    test house inside the usable area, and in a second run produced for it. Record from the tick
    of placement its height above the terrain, behaviour and queued behaviour, and give the attack
    order in the first tick it is accepted. Expected: placed by the map, height 0, *guard*, no
    *retreat* and no edge draws before the order; produced (no docking place, so entering from the
    edge), cruise height; in both runs the order accepted in that tick, then (1)'s result.
    Falsified by *retreat* current before the run ends, edge draws before the order, or the
    order ignored. If the
    enemy house is defeated before the aircraft's run ends, the part is void.
  - h. AC21's carrier set-up, on water well inside the usable area so that no slot aircraft
    leaves it before a launch pass (claim 10), with named test rules: the carrier's slot type is
    made not selectable (so its slot aircraft are visiting, Terms), otherwise unchanged. The
    carrier attacks a durable enemy vehicle in range; then the same with the vehicle destroyed
    by another unit while a slot aircraft approaches it. Record each slot aircraft per tick,
    with the launch passes (S-0506 claim 2), from its launch placement on: its height above the
    terrain, behaviour and queued behaviour.
    Expected: at its launch placement height 0 above the water surface, *guard* and no edge
    draws, and the next launch pass's rows (T4) accepted, not ignored (claims 1 and 35); at a run end with its ammunition spent, the run-end draws, then *retreat* with
    target and destination forgotten; a second set of edge draws only when a *retreat*
    processing comes before the next pass; at that pass the spawner as destination, no target,
    no *move* queued and *retreat* still current (claims 34 and 35). With the target lost and
    ammunition left: *hunt*, and at the next pass T4's *move* toward the spawner. From then on
    the record judges nothing (Open question 22). Falsified by: *guard*, *area guard* or a *move*
    toward a replacement cell after a run end, or *move* current after the pass while the
    aircraft was in *retreat*.
  - i. Part f (1)'s set-up with named test rules: the timed-bomb detonate permission on, a
    game-wide bomb delay longer than the whole record, and a game-wide bomb damage that differs
    from the aircraft's death-weapon damage. A durable test vehicle of the test house stands
    beside the expected impact cell, and its health is recorded per tick. While the aircraft
    stands on its pad before the attack, a bomb planter of the test house plants a bomb on it,
    as S-0519 scenario T9 plants on an own object. Record per tick whether the bomb is attached,
    its icon, the ticking sound and every detonation with its position. (1) Let the crash run
    to impact. (2) Repeat, giving the detonate order during the fall; this is an attempt, and
    the run is reported unreached if the order does not land between the crash start and the
    impact. Expected: in (1) the bomb stays attached and ticking from the crash start through
    the fall, no bomb detonation at the crash start, during the fall or at impact, and at impact
    the bomb, its icon and its sound end with no bomb damage to the test vehicle (only the death
    weapon's); in (2) one bomb detonation at the aircraft's position in the air in the order's
    tick, then the impact as in part f. Falsified by: a bomb detonation at the crash start, a
    bomb detonation or bomb damage at impact in (1), the bomb gone before the impact in (1), or
    no detonation in (2) when the order landed during the fall.
  - j. Named test rules: part b's type with a low flight level, for a computer-controlled test
    house, placed by the map; the durable enemy vehicle stands on a raised plateau whose cliff
    drops toward the house's computed edge, so that the aircraft crosses the rim after its run.
    No order is given: the vehicle stands within the aircraft's *area guard* search (S-0215 claim
    26), so that its own search picks the vehicle and starts the run. Record
    per tick its height above the terrain, behaviour, queued behaviour and the ticks between
    *area guard* processings. This is an attempt: the run is reported unreached if no *area
    guard* processing finds the aircraft above its cruise height. Expected at a processing above
    it: with no target held, S-0215 claim 26's delay with *k* = 2 to the next processing and
    *area guard* kept (with its ammunition spent, after claim 45's fall-back to rest in the same
    processing); with a target held, *attack* queued and 1 tick to the next processing; at a
    processing at exactly its cruise height, the fall-back to rest and 1 tick. Falsified by a
    1-tick delay with no target held at a processing above the cruise height. Void if the
    aircraft joins a team.
- **AC30 Tracking as a flying object (claim 47, Q-1845).** A flat purpose-made map at least 60
  cells along each axis, so that each sector of S-0303's F7 is at least 3 cells wide. The
  recorded house owns one stock defence whose weapons can engage only aircraft. The enemy
  house is human-controlled, and its aircraft get only move orders, never a target: a stock
  landable aircraft, or a named test copy of it without weapons. Neither house takes damage
  from the other unless the defence fires. Record per tick the defence's target, turret
  facing and shot events, and the aircraft's position, cell, height above the terrain, current
  speed fraction, health and render layer.
  - a. The aircraft starts on the ground inside the usable area, outside sector (0, 0) and
    outside the defence's range, and takes off on a move order whose straight path crosses the
    defence's range at cruise height. Expected: the defence takes the aircraft as its target and
    turns toward it, then fires, with no damage to the defence first, at its first automatic search
    after the aircraft enters its reach (S-0309). Falsified by no target until the defence is
    damaged.
  - b. A named test copy with the landable flag removed gets a move order to a cell within the
    defence's range, so that it hovers there (AC4). Expected: the defence acquires it and
    keeps it as a target while it hovers with speed fraction 0.
  - c. The landable aircraft gets a move order, along a straight path from outside the
    defence's range, to an empty cell that is the first cell of that path within the range, so
    that it enters the range only on reaching that cell, where it slows and starts to land
    (claim 6). The defence must not have acquired it before its landing starts. Record the tick
    the aircraft enters the range, the tick its landing starts, the tick its in-the-air answer
    becomes *no* and the touchdown tick. Run only if the descent keeps the in-the-air answer
    *yes* for longer than the defence's interval between automatic searches (S-0309), since the
    defence's weapons may fire only at an object in the air. Expected: the defence acquires it
    during the descent, at its first automatic search after the aircraft enters the range.
    Whether the defence keeps its target after touchdown is S-0309's and is only recorded; the
    end at touchdown itself is part i's. Falsified (the end at the start of landing) by no
    acquisition between the start of landing and the tick its in-the-air answer becomes *no*.
    Void if the defence acquires it before its landing starts.
  - d. The map places the aircraft on the ground within the defence's range, not visiting.
    Expected: no acquisition while it stands there. Then give it a move order away. Expected:
    acquisition no earlier than the first search of the defence after the aircraft has left the
    ground render layer, and no later than the first search after its in-the-air answer becomes
    *yes* (claim 47 (e)). Record the order's tick, the tick it leaves the ground render layer,
    the tick its in-the-air answer becomes *yes*, the tick of the first target and the
    aircraft's height at that tick, and the first shot.
  - e. As d, with the aircraft standing on its house's pad inside the defence's range. The
    expectation is the same as d.
  - f. With the defence's search reaching sector *A* but not the neighbouring sector *B*, the
    test copy of b hovers in the first cell of *A* past the *A*/*B* boundary, coming from *B*.
    Its last forward step is the one that crosses into *A*: the movement update that makes the
    crossing leaves its speed fraction at 0, so it takes no further step. Expected: the defence
    does not acquire it while it hovers there; it acquires it once a new move order inside
    *A* has made it move on with a positive fraction. This is an attempt: the run is reported
    unreached if the aircraft takes a forward step inside *A* after the crossing. Falsified by
    acquisition while it hovers stopped.
  - g. Named test rules: a copy of the landable type with the counts-as-aircraft flag turned off,
    placed by the map outside the usable area, so at its cruise height (claim 1), and given a
    move order across the defence's range. (1) Outside sector (0, 0): expect acquisition as in
    a. (2) Placed in sector (0, 0) and flying within sector (0, 0) past a defence that stands in
    that sector: expect no acquisition until the aircraft first enters another sector. This is
    an attempt: void if no usable cell in sector (0, 0) is within the defence's range of the
    route.
  - h. As g (2), with the test copy's counts-as-aircraft flag (1) set explicitly, and (2) left
    out of its section. Expected in both: the aircraft is tracked from its placement in the air,
    so the defence acquires it within sector (0, 0), at its first search after the aircraft
    comes within its range, unlike g (2). Falsified by no acquisition until the aircraft first
    enters another sector, which in (2) would mean an absent default of *no*. The same attempt
    and void rule as g (2).
  - i. Area-damage entries (S-0303 claim 6). Use S-0303 scenario A21's warhead override and
    target health, with the aircraft as the target and the recorder of A21 for each candidate
    entry. Cause one detonation whose height is above the terrain height of the centre cell
    (S-0303 claim 6) with the aircraft within the spread, for example a shot at a flying decoy
    that hovers above it, whose own entries are recorded apart. (1) The aircraft has landed on open
    ground after a move order, at least one search interval before the detonation. (2) It has
    landed on its house's pad in the same way. (3) The map placed it on the ground and it has
    never moved. (4) It is in the descent of part c, after its landing has started and while
    its listing height is positive. Record the number of entries that deliver a hit, their
    order and the damage of each. Expected: one hit, from the scanned cell, in (1) to (3); one
    hit, from the flying-object search, in (4). Falsified: two hits in (1), (2) or (3) (the
    aircraft still tracked after touchdown, on the ground or on the pad), or no hit in (4) (the
    end at the start of landing). Void when the detonation of (4) falls outside the descent, or
    when the decoy's own hits cannot be told apart.
  - j. Sidebar rank (S-0004 claim 11). Named test rules: a buildable test aircraft type with
    the counts-as-aircraft key left out and a buildable test vehicle type with the naval flag
    clear, of the same side, the same base-planning side, tech level and cost, offered in the same
    tab; the aircraft's
    display name sorts before the vehicle's. Make the aircraft available first, then the
    vehicle. Expected: the vehicle's icon before the aircraft's, the aircraft ranking as air
    with the absent default *yes*. Control: the same with the aircraft's key set to *no*, where
    the aircraft's icon is expected first, by display name. Falsified: the aircraft first with the
    key left out.

## Changes for other specs

- **S-0303 Terms (*Tracked flying object*).** For aircraft, "an aircraft-kind object in
  ordinary flight" could point to claim 47 here, which owns the events: tracking starts when
  the take-off starts, at height 0, or at placement in the air, and ends at touchdown, at a
  crash impact or on leaving the map. An aircraft placed in the air with the counts-as-aircraft
  flag ends its placement registration at touchdown like any other aircraft, not only on
  leaving the map. S-0303's Import mapping row for that flag gives "no" as its absent default,
  but for an aircraft type the absent default is *yes* (Import mapping here).
- **S-0105 Import mapping.** The absent default of the counts-as-aircraft flag is *yes* for an
  aircraft type and *no* for other types, so S-0105's half-size shadow rule (claim 31) reads
  *yes* for an aircraft type whose data leaves the key out.
- **S-0004 open question 2 and claim 11.** The default that S-0004 leaves to this spec is
  settled: the counts-as-aircraft flag of an aircraft type whose data leaves the key out is
  *yes* (Import mapping here), so under claim 11's rule 3 such an aircraft ranks as air (1), not
  as ground. Open question 2 can be closed with a reference to this spec (AC30 part j).
- **S-0215 claim 7.** Its queueing table has no row for this aircraft rule: an aircraft that is
  not a strike aircraft and is in *retreat*, *paradrop approach*, *paradrop overfly*,
  *spy-plane approach* or *spy-plane overfly* ignores entirely a request to queue any behaviour
  outside those five (claim 34). Claim 7 could add it as a first row for aircraft.

- **S-0215 claim 26.** Its rows, its table and its delay, with *k* = 2 for aircraft (scenario
  B10, case i, an aircraft standing landed), reach an aircraft's *area guard* only at a height
  other than its cruise height, above or below it, and only when no held target makes it queue
  *attack* (1 tick).
  At exactly its cruise height an aircraft's *area guard* processing is instead this spec's
  fall-back to rest, with a delay of 1 tick; at any other height an armed aircraft with its
  ammunition spent and no contact makes the same fall-back first, in the same processing
  (claims 18 and 45, table T2). S-0215 claim 26 should say so.
- **S-0309, holding a target.** No approved spec states the plain loss of a target object for an
  actor whose behaviour is not overridden. Every actor (vehicle, infantry, aircraft, building)
  holds no target from the moment its target object is killed (at the killing hit), leaves the
  map (any transport, a building as occupant) or is removed, wherever in the tick that falls,
  also during another object's turn, with no owner, alliance or sensor exception. When the object
  starts to cloak or goes underground, the same loss applies with S-0215 claim 10's own-house
  and sensor exceptions. At the loss, if the actor's targeting timer has more than 10 integer
  ticks left, one *U*(4, 8) scenario-stream draw restarts it first, except while the scenario is
  being set up; it is the same draw S-0309 gives at the phasing-in end. S-0215 claim 10's
  revert follows in the same moment, and S-0615 claim 14 and S-0207 claim 20 e list the same
  events for their own losses. A unit in a tank bunker stays on the map and is not lost (S-0606).
  S-0309 could state this as its target-holding rule; claim 32 and AC19's Q-1755 part h use it.
  What this spec backs: the loss for an aircraft (claim 32, AC19) and for a ground vehicle (AC19
  part h), at a boarding, an occupation and a kill. The same loss for infantry and buildings, and
  the cloak and underground losses, are `unmeasured` here: no scenario of this spec sets them up.
  They are taken to follow the same loss, with no difference by the actor's kind, and are
  rated medium; S-0309's revision would back them itself.
- **S-0312 claim 16.** Its demolition damage for "a crash started without a kill" does not reach
  the fall-back crash of claim 46 here: that crash sets the aircraft's health to 0 at its start
  without a hit, so the aircraft falls and strikes the ground as one shot down. Its destruction is
  credited to nobody and counted among its house's losses, and it takes no
  killing-hit effects. Claim 16 could name this crash among those that start dead.
- **S-0519 claims 5 to 7 and its state table, and S-0312 claim 7.** A carrier destroyed without a
  hit, as by claim 46's fall-back crash here, has no shared death effects, so its planted bomb
  does not detonate at the destruction. The bomb stays attached while the carrier stays on the
  map: during the aircraft's fall its timer and the detonate order still set it off by S-0519
  claim 5. If it has not gone off by the impact, it is removed then with no effect, as in the
  row "carrier removed other than by death" (`quirks.crash_drops_planted_bomb` here). S-0519's
  row "carrier dies on the map" could say "dies from a killing hit", with this case as a separate
  row. Backed by AC29 part i here.

## Open questions

1. Q-0630 is answered by claims 12 and 39; no firing-spot geometry question remains.
2. Q-0631 is answered by claims 33 and 40; *move*'s stopped test is distinct from speed
   fraction, while T1's ordinary-attacker test is the zero fraction (claim 42).
3. The planned-route exception of claim 6 belongs with the order specs' waypoint planning; this
   spec states only that it stops a landing.
4. The application timing of the landing-facing choice (claim 25) is resolved only for
   claim 42's aiming request. Do not infer height stages from the take-off rules.
5. Q-0634 is answered by claim 41 and F8 with S-0212 claim 25a for footprint qualification,
   unchanged result cells and candidate choice. Equal-distance fallback objects favour
   earliest creation; map-edge destinations are claims 17, 34 and 37 with S-0522 claims 4 and 5.
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
12. Q-0651 is answered by claims 5 and 42, F4 and table T1: the order within an update, the
    ineffective creep restart and approach replacement, the stop at fraction 0 short of the spot,
    the facings and the zero-fraction test. The strafer's body exception in claim 42 was traced
    but not exercised by a scenario.
13. Q-0650 is answered by claim 41 and F8: the placeholder destination at map cell (0, 0) and its
    discard. The net flight path while the placeholder repeats has medium confidence; AC25
    measures it.
14. Q-0653 is answered by claims 4, 5 and 42, F3 and Edge cases. (1) The original hovers: it
    truncates each new coordinate and keeps no remainder, so a 1-lepton step between east and
    south does not move the aircraft (`quirks.aircraft_unit_step_stall`). (2) Nothing in the
    step or the slowing differs from F3 and F4; F3 now gives the per-facing result of a
    1-lepton step. (3) Claim 5 now states the condition: converted speed from 10 to 19,
    2*S* / *V* > max(16, min(86, *S* / 10)), and a body facing in 16391 to 32766 during a
    1-lepton step outside the halving zone; converted speed of at least 20 never takes such a
    step. AC24's (b1) to (b6) measure it.
15. Q-0654 is answered by claim 4, claim 5, F3, AC24 and AC26. (1) Under the default
    round-toward-zero arithmetic a 1-lepton step at body facings 32767 to 32772 moves one
    lepton south and one lepton west, and at 49148 to 49155 one lepton west and one lepton
    north. (2) On a long flight at those facings x (or y) falls by one lepton on every step,
    whatever the step length. (3) The two agree once F3's table is computed with the
    default rounding: the aircraft step is the computation of S-0208 F3 with S-0105 F1 and
    S-0205 claim 5, and F3's table now follows it; the earlier table had been computed with
    sums rounded to nearest. AC24 (b3) now expects the westward drift and a hover facing near
    29778 with a side distance near 47 from a 5120-lepton start; facing 32076 and side
    distance 11 are the result with `quirks.chop_rounding` off. AC26 measures the drift
    directly.
16. For a *move* accepted while the aircraft has a target and ammunition left, whether claim 6's
    height replacement then differs from claim 40's earlier acceptance of the destination was not
    traced. It does not affect Q-1756's settled answer: that answer is about which destination
    cell is kept or replaced (claim 40), and claim 6's replacement changes only the height of the
    flight point held over that cell, not the cell.
17. Whether a strike aircraft's run can end with ammunition left without its strike being
    cancelled first (claim 34), and, for a cancelled strike, whether its aircraft's run end or
    its *retreat* picks the edge cell.
18. Q-1776 questions 2 and 3 for an aircraft that is not landable, in *hunt* with ammunition
    left and a flight flag (claim 45): what it does once it reaches claim 17's edge cell, and
    whether it then leaves the map; and, for question 3, the computer-controlled owner's case of
    claim 45, whose *area guard* and repeated landings are `unmeasured` (claim 45).
19. A pad that ends an approaching aircraft's contact by a refusal while a behaviour is queued for
    the aircraft (claim 46): whether the destination is kept and the queued behaviour starts
    instead of the fall-back to rest. No set-up is known that makes a pad refuse an approaching
    aircraft in this way.
20. A carrier slot aircraft that is not visiting, between its attack run's end and the next
    launch pass (claims 17 and 35): whether a slot type with no dock types can reach and land on
    claim 40's replacement cell before that pass (T4), and how a slot type with dock types
    fares in that interval. The interval is
    at most 10 ticks, and AC21 does not record the slot's destination in it.
21. A computer-controlled house's armed aircraft with no dock types, outside a team, in *area
    guard* away from its cruise height (claim 45): S-0215 claim 26's search there can give it a
    target, and a later processing away from that height that finds it still holding the target
    queues *attack*; whether it keeps that target, and what that *attack* does with
    the edge cell as destination, and with its ammunition spent, was not traced.
22. A visiting carrier slot aircraft that stays in *retreat* with its spawner as destination
    (claim 35): whether it lands on its spawner and is taken off the map as T4's returning rows
    describe, and whether its *retreat* persists into its next sortie, where it would drop T4's
    *move* and *attack* as well. Not traced; AC29 part h records it.
23. Q-1755, the part left open (the Summary lists what claim 32 answers): whether a harvester
    docked at a refinery stays on the map, which no trace here confirms (claim 32, low; AC19's
    Q-1755 part g).
24. An aircraft whose weapon has the suicide flag takes its own hit at the first shot of its
    burst, and dies of it unless S-0519's edge case of a suicide firer that survives its own hit
    applies (claim 32). What its further burst attempts and the scatter then do, and whether
    its death's detonation can reach the target before the burst ends, was not traced. No
    scenario here gives an aircraft a suicide weapon.
