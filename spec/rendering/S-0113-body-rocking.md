---
id: S-0113
title: Body rocking
topic: rendering
revision: 17
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-01
measurements: []
requests: [Q-2081, Q-2236]
supersedes: []
---

# S-0113: Body rocking

## Summary

How a push sets a body's pitch rate and roll rate, how the type's weight scales it, how the two
angles then move in each turn of the body and return to level, the limits that hold them, the
destruction of a body that rolls or pitches over, and what reads the angles besides drawing.
Claims 1 to 4 answer Q-2081 question 1, claims 5 to 9 question 2, claim 3 and claim 10
question 3, claim 6 question 4, claims 3 and 11 question 5, claim 13 question 6, claim 14
question 7 and claims 18 and 19 question 8. The causes of a push stay with the specs that own
them (S-0303 claim 20, S-0514 claim 20, S-0614 claims 9 and 10, S-0601 claim 16, S-0210,
S-0312 claim 16); this spec gives what a push of each kind does to the angles. Claim 20 answers
Q-2236: how a crashing aircraft's body is drawn with its tumble.

## Terms

Shared terms used here are defined in the glossary: *tick*, *lepton*, *facing*, *vehicle*,
*aircraft*, *infantry*, *building*, *locomotor*, *voxel model*, *hit*, *warhead*, *health,
strength*. S-0205's notation applies: trunc, RZ₅₃, RZ₂₄, *single* and *double*. S-0105's *table
trigonometry* (F1) gives sinT and cosT. S-0504 Terms define *pitch* and *roll*, and S-0214 Terms
define *melee-linked*.

- **Turn.** In this spec, an object's share of one tick, in the glossary's second sense of *turn*
  (S-0203 Terms). A rotation is written with facing words.
- **Pitch, roll, pitch rate, roll rate.** As S-0504 Terms: two single-precision angles of a body,
  in radians, and a single-precision rate per angle, in radians per tick. A positive pitch lowers
  the bow and raises the stern. A positive roll raises the body's left side, the model frame's
  +*y* side (S-0105 Terms). S-0210's *body tilt* is the pitch rate, and its *tilt units* are
  radians.
- **Rocking body.** An object of any kind whose type is drawn as a voxel model. Only rocking bodies
  have their angles and rates changed by a push or moved by the rocking step.
- **Push.** An event that sets both rates of a rocking body from a push point *Q*, a strength *s*
  and a pitch factor *h* (claims 1 to 4, Formulas F1). A push changes the rates only, never the
  angles.
- **Rocking step.** The change of a body's angles and rates in one of its turns (claims 5 to 9,
  State and ordering, table R).
- **Level band.** Angles from −2 × 10⁻⁵ to 2 × 10⁻⁵ radians inclusive.
- **Limit** *L*. The angle at which an unlinked body's angle is held (claim 6): π/4 for the roll,
  and for the pitch π/10 while the body is in S-0210's crush-slowdown state, π/4 otherwise.
- **Past the edge.** An angle greater than π/2 or less than −π/2.
- **Weight** *W*. A type's double-precision weight field (Import mapping).
- **Fall-back factor** *c*<sub>f</sub> and **direct-rocking factor** *c*<sub>d</sub>. Two
  game-wide single-precision decimals (Import mapping).

## Behaviour

### Pushes

1. **A push sets both rates.** A push of strength *s* from point *Q* on a rocking body at *B*
   computes a magnitude *k* from the distance *d* between *B* and *Q* and from the type's weight
   (claim 3), and sets the pitch rate to *k* × *h* × the forward component of the horizontal unit
   direction from *Q* to *B*, and the roll rate to *k* × its rightward component, both relative to
   the body's facing (Formulas F1). The unit is radians per tick. Physically, the end or side of
   the body nearer the push point rises, at every facing. *h* is 1 for a direct-rocking hit
   (claim 12) and ½ for an area-damage push and a drone delivery (claims 11 and 13).
   - Backing: `measurable` (RK1)
   - Confidence: high

2. **A push replaces the rates.** The new pitch rate and roll rate replace whatever rates the body
   had, and the angles keep their values. When several pushes reach a body before its next turn,
   only the last one counts. A push that changes nothing (claim 3) leaves the earlier rates.
   - Backing: `measurable` (RK1 variant c) for the replacement and for the last of several pushes.
     `unmeasured` for a push that changes nothing: RK1 (c)'s two pushes both act, and no scenario
     here lands a push that changes nothing on a rocking body.
   - Confidence: high; medium for a push that changes nothing

3. **Strength and distance.** *k* = (0.04 − 2.5 × 10⁻⁵ × *d*) × *s* ÷ *W*, with *d* the
   three-dimensional distance in leptons, taken with S-0204's table square root (F1). The push
   changes nothing at all when *Q* is the body's own position, or when *k* is below 0.01 or is not
   a number. A *k* above 0.05 is set to 0.05. So the weight divides the push.
   - **The reach.** With F1's single-precision constants the numerator's zero lies a few
     millionths of a lepton beyond 1600. Near 1600 leptons the table root moves in steps of about
     0.08 leptons: every squared distance from 1600² to 1600² + 255 square leptons gives exactly
     1600, where the numerator is still positive, about 1.2 × 10⁻¹⁰, and the next step, from
     1600² + 256, is beyond the zero. With S-0204's `quirks.table_root_distance` off, the exact
     root of F1 makes 1600² the last squared distance with a positive numerator.
   - **When *s* ÷ *W* is positive** (the usual case: a positive weight, and an area push, whose
     strength is always positive, F2): no push reaches a body beyond that last squared distance,
     and at a root of exactly 1600 a push reaches it only when *s* ÷ *W* exceeds about 8.6 × 10⁷
     (an area push can reach that far only with a large height difference).
   - **When *s* ÷ *W* is negative** (a negative weight, or a negative direct-rocking strength from
     a negative damage or factor, F2): *k* is negative up to that distance, so nothing happens, and
     positive beyond it. Beyond that distance, the 0.01 threshold still applies: the body rocks
     there only when *k* is at least 0.01.
   - **A weight of 0** gives the cap whenever the product of numerator and strength is positive,
     and nothing otherwise.
   - Backing: `measurable` (RK1, RK10) for a positive *s* ÷ *W*, and for a negative one within the
     reach. `unmeasured` for a negative *s* ÷ *W* beyond the reach: only an area push from *P*
     reaches that far, and only a body in claim 11's 7 × 7 cells at a height difference of at least
     about 680 leptons from *P*, and no scenario here places a rocking body there. `unmeasured`
     also for a push at a root of exactly 1600: it needs a push point placed to the lepton at that
     distance and *s* ÷ *W* above 8.6 × 10⁷, and no scenario here can place one. Both rest on F1
     and S-0204 F1. `unmeasured` also for a push point at the body's own position: no scenario
     here places one there (RK1 (d)'s points are 64 leptons from it).
   - Confidence: high within the reach; medium for a negative *s* ÷ *W* beyond it, for a push
     at a root of exactly 1600 and for a push point at the body's own position

4. **A push from straight above or below levels the body.** When *Q* lies straight above or
   below *B* (the same *x* and *y*, but not *B* itself) and *k* is at least 0.01, both rates are set
   to 0. So, whatever its angles were, each angle that takes table R's step at the body's next turn
   is level after it (claim 5). A sinking body's pitch takes the lean instead, and a body in a warp
   or crashing takes no table R step.
   - Backing: `measurable` (RK7) for a push point straight above. `unmeasured` for one straight
     below: no scenario here places a push point straight below a body that the push reaches.
   - Confidence: high for a push point straight above; medium for one straight below

### The rocking step

5. **One step per turn.** In each turn of a rocking body, its angles and rates take one rocking
   step, whatever its locomotor, except during a superweapon warp (table S). For a body that
   is sinking, its pitch takes S-0504 claim 15's lean step instead of table R's, and its pitch rate
   is left as it is. Its roll takes table R's roll step as in any other turn, so a push during the
   descent rolls the hull, and the roll then returns to level, as S-0504's decided fix leaves free
   (Edge cases). A crashing body takes claim 16's step. Otherwise the roll and
   the pitch each take a step of table R:
   an angle whose rate is exactly 0 becomes 0, and any other angle adds its rate, is held at its
   limit (claim 6), and the rate is then pulled back toward level (claim 7) or away from it past
   the edge (claim 9). An angle that comes back into the level band from outside stops there: the
   angle and its rate become 0. So a body whose rates are pulled toward level never swings through
   level and back: it rises, turns and settles. That is every unlinked body, and a melee-linked
   body whose fall-back factor *c*<sub>f</sub> is positive. With *c*<sub>f</sub> of 0 a linked
   body's rates are not pulled, and with a negative *c*<sub>f</sub> the pull is away from level:
   table R's steps apply to whatever value the import gives (Import mapping).
   - Backing: `measurable` (RK2 for a driving body; RK2 part b for ship, hovering, jumpjet and
     walking bodies). `unmeasured` for the mechanical walking, drop-pod and teleporting locomotors
     and for a flying body outside a crash: no scenario here runs them. The sinking roll is a design
     rule of our engine (S-0504, decided at the delegated sign-off, 2026-10-25); RK11 records the
     original's held roll (Edge cases). DECIDED (autonomy, 2026-10-29): our engine takes no rocking
     step during a superweapon warp from the first pending update through the arrival effects, the
     stages in which S-0513 claim 51 suppresses other ordinary work, and takes it again in arrival
     hold after the hold-end test (table S). What the original does there is open (Open questions).
     A linked body with *c*<sub>f</sub> of 0 or below is `unmeasured`: RK4 (b) sets positive
     values only.
   - Confidence: high for a driving body; medium for ship, hovering, jumpjet and walking bodies,
     until RK2 part b records them; medium for the mechanical walking, drop-pod and teleporting
     locomotors and a flying body outside a crash, which are `unmeasured`; medium for a linked body
     with *c*<sub>f</sub> of 0 or below; high for the sinking roll and the warp, which are design
     rules of our engine

6. **Limits.** An unlinked body whose angle was below its limit *L* and not past the edge, and
   whose new angle would pass *L*, is held at *L* and its rate becomes 0 (the same at −*L*). The
   roll limit is π/4 always. The pitch limit is π/10 while the body is in the crush-slowdown
   state and π/4 otherwise (S-0210 claim 12). A melee-linked body has no limit. A body held at
   its limit stays there for that one step only: the pull of claim 7 acts in the same step, so it
   starts back at its next turn.
   - Backing: `measurable` (RK3 and RK4 for a driving body; RK3 part c for ship, hovering and
     jumpjet bodies). `unmeasured` for the walking, mechanical walking, drop-pod and teleporting
     locomotors and for a flying body outside a crash: no scenario here holds one at a limit.
   - Confidence: high for a driving body; medium for ship, hovering and jumpjet bodies, until RK3
     part c records them; medium for the walking, mechanical walking, drop-pod and teleporting
     locomotors and a flying body outside a crash, which are `unmeasured`; medium for a
     melee-linked body's having no limit, until RK4 (b) records it, since the link it rests on is
     rated low (claim 12)

7. **Return to level.** After the angle moves, its rate changes by an amount chosen from the angle
   before the step (table R):
   - a pitch rate is lowered by 0.005 per step while the pitch is above the level band, and raised
     by 0.005 while it is within the band or below it,
   - a roll rate is changed in the same direction by the roll's side of the band: by 0.005 per
     step while the roll is moving back toward level (or its rate is 0), and by 0.002 per step
     while it is still moving away,
   - a melee-linked body's rates are changed in the same direction by the angle's side of the band
     by *c*<sub>f</sub> × 0.002 per step instead, on both angles.

   An angle more than π/2 from level takes claim 9's rows instead (table R).

   An angle within the level band counts as being on the negative side. So, for an unlinked body, in
   the first step after a push from level, a positive rate grows by 0.005, while a negative rate
   shrinks by 0.002 (roll) or 0.005 (pitch). A push from level therefore rises higher and stays away
   longer on the positive side than the same push on the negative side (RK2's table). This is the
   original's rule and our engine's.
   - Backing: `measurable` (RK2; RK4 variant b for the melee link)
   - Confidence: high; medium for a melee-linked body's pull, until RK4 (b) records it, since the
     link it rests on is rated low (claim 12)

8. **Precision.** Each new angle is the sum of the angle and its rate rounded to single precision,
   and each new rate is formed in double precision and rounded to single, as S-0205 sets.
   0.002 is a single-precision constant. The large pull is the double-precision constant
   0.0050000002374872565, slightly above 0.005 and not itself a single, and it is added in
   double precision before the rate is rounded. The level band, the limits, π/2 and π are
   compared exactly with the single-precision angle, π/4, π/10, π/2 and π being single-precision
   constants.
   - Backing: `measurable` (RK2: the tick of the peak and of the return to level depends on it)
     for the formulas and each recorded angle. The large pull's exact digits are `unmeasured`:
     RK2's angles cannot tell it from a nearby double that gives the same stored rates
   - Confidence: high for the formulas, medium for the last-bit rounding of each rate and for
     the large pull's exact digits

9. **Rolling and pitching over.** An angle past the edge has no limit, and its rate takes table R's
   outward pull of 0.002 per step. After both angles have stepped, a body whose roll or pitch is
   greater than π or less than −π (table S's otherwise row) receives a hit with damage equal to its
   type's strength, with the demolition warhead (S-0303 Import mapping), ignoring defences, with no
   attacker and no attacking house, and without the no-escape marker. S-0302 resolves the hit, so
   the body is destroyed unless S-0302 stops it (for example by the warhead's immunity row, S-0302
   claim 9). A body destroyed this way takes no further part in that turn. An unlinked body can pass
   the edge only through claim 15's escape. A melee-linked body can be carried past it by repeated
   direct-rocking hits.
   - Backing: `measurable` (RK3) for the rule, the warhead and the missing attacker. The hit's
     damage, its ignoring of defences, its missing attacking house and its missing no-escape
     marker are `unmeasured`: RK3 records only the killing hit's attacker and warhead, which do
     not tell them apart.
   - Confidence: high for the rule, the warhead and the missing attacker; medium for the hit's
     other attributes

10. **Weight of every kind.** Every type kind has the weight field, read from its own type section
    with 1.0 as its value at type creation (Import mapping), so the weight divides pushes on
    vehicles, aircraft, infantry and buildings alike when they are drawn as voxel models. The
    drone's push uses the host's weight. A grappling parasite's roll (claim 14), a mind-control
    overload's roll rate, a crush tip and a crash's tumble (claims 16 and 17) ignore the
    weight.
    - Backing: `measurable` (RK10) for vehicles and aircraft. For infantry and buildings drawn
      as voxel models `unmeasured`: no scenario here sets such a type up.
      The weight of 1.0 when the key is absent, and the two coefficients' engine defaults, are
      `unmeasured`: RK10 sets every weight it uses, and no scenario here checks a key left
      absent. That a crash's tumble ignores the weight is `unmeasured`: RK9 does not vary the
      weight of a crashing body. That a grappling parasite's roll ignores the weight is
      `measurable` (RK6, with *W* = 1 and 2). That an overload's roll rate and a crush tip ignore
      it is `unmeasured`, as claim 17 gives.
    - Confidence: high for vehicles and aircraft and for the parasite's roll ignoring the weight,
      medium for infantry and buildings, for the absent-key defaults, and for an overload's roll
      rate, a crush tip and a crash's tumble ignoring the weight

### Causes

11. **Area damage.** S-0303 claim 20's rocking is a push with *h* = ½ and strength *s* = min(*D* ×
    0.01, 4.0), formed in double precision and rounded to single (Formulas F2).
    - A body in the centre cell, when the detonation has a source object, is pushed from the point
      *Q* = *B* + trunc(10 × *v*) leptons, per axis, where *v* is the three-dimensional unit vector
      from *B* toward the source object (its root and precision are open, Open questions). It is
      pushed away from the source, whatever *P* is. A source at exactly the body's position changes
      nothing (claim 3).
    - Every other body in the 7 × 7 cells, and a centre-cell body when the detonation has no
      source object, is pushed from *P*, only when *S* > 0. A body standing exactly on *P* is not
      rocked (claim 3).
    - A projectile's detonation has a source object exactly when S-0302 claim 1 gives it an
      attacker. A firer killed, removed from the game or (believed) starting to cloak while its
      projectile flies leaves none, so a centre-cell body is then pushed from *P*. A firer that
      has left play without being killed, such as one loaded into a closed transport since the
      shot, is not known to stop being the source (S-0302 claim 1, low), and the centre-cell
      body is then pushed away from the position that firer holds out of play, which is its
      position when it left play (low; Open questions).
    - Backing: `measurable` (RK1, with part d for a centre-cell body when there is no source
      object, part e for a firer destroyed while its projectile flies and part f for a firer
      loaded into a closed transport while its projectile flies).
      `unmeasured` for a firer removed from the game by other means or starting to cloak while
      its projectile flies: both rest on S-0302 claim 1, which rates them, and no scenario here
      sets either up.
      `unmeasured` for the reach to the edge of the 7 × 7 cells: RK1 places its detonations one
      cell from the subject or inside its cell, and no scenario here places one at the edge of the
      area or just beyond it.
      `unmeasured` for a source at exactly the body's position and for a body standing exactly on
      *P*: no scenario here places either (claim 3)
    - Confidence: high; medium for the 7 × 7 reach, for a source at exactly the body's position,
      for a body standing exactly on *P*, for a firer destroyed in flight and for one
      removed from the game by other means (resting on S-0302 claim 1); low for a firer starting
      to cloak (believed, as S-0302 claim 1 gives it) and for one that has left play without
      being killed

12. **Direct-rocking hit.** A hit selected by S-0514 claim 20's direct-rocking row, with data that
    sets the flag and with its shooter present, and that passes the further check of the target
    named in Open questions, is a push with *h* = 1 from the point 10 leptons from the target
    toward the shooter (as claim 11's centre-cell point), with strength min(*A* × *c*<sub>d</sub> /
    100, 4.0), *A* being the amount the detonation delivers (S-0302 claim 2, under S-0518 F1's wrap
    rule; Formulas F2). The same hit then makes the shooter and the target melee-linked. The
    direct-rocking flag is the warhead link flag of S-0214 Terms (*melee-linked*), so this is the
    creation that S-0214 gives, and the target's passing the further check is a condition of both
    the push and the link (Changes for other specs). The linked target's rates return to level by
    *c*<sub>f</sub> × 0.002 per step and it has no limit (claims 6 and 7).
    - Backing: `measurable` (RK4 variant b for the push, variant d for the strength: its
      product with *c*<sub>d</sub>, the divisor 100 and the cap of 4.0). Which targets pass the
      further check is open (Open
      questions), so the push part holds only for a target that RK4 (b) shows is rocked. That the
      hit makes the link is `measurable` (RK4 variants b and c): in (b) the pair is unlinked
      before the first flagged hit, so a pull of *c*<sub>f</sub> × 0.002 per step, which only a
      linked body takes, shows the link that hit made; in (c) hits without the flag leave the
      pushed subject with the unlinked step and limit.
    - Confidence: medium for the strength and for which hits push; low for the hit making the
      link, as S-0214 Terms rate what creates it; medium for the link's step (claims 6 and 7) for
      a body that is linked

13. **Drone delivery.** S-0614 claim 9's rocking is a push with *h* = ½ and S-0614 F2's strength,
    using the host's weight, from the point of Formulas F3: a draw of 0 puts the point about 2
    leptons to the host's **left**, so the host is pushed to its right and its left side rises (a
    positive roll).
    A draw of 1 puts it to the right. The point is truncated to whole leptons, so the direction is
    only roughly square to the facing.
    - Backing: `measurable` (RK5)
    - Confidence: high

14. **Grappling parasite.** During a push phase of S-0614 claim 10, the parasite sets its host's
    **roll** directly, in each of the host's turns after the phase's first, after the host's rocking
    step: roll = sinT(*k* × *σ* × π/10) × π/4 at phase frame *k*, rounded to single (Formulas F4),
    with *σ* = +1 (left side up) when that push's own way draw is 0 and *σ* = −1 (left side down)
    when it is 1. The phase frame *k* is 0 when the phase starts and goes up by 1 every 4 of the
    host's turns. Counting the turn in which the phase starts (the turn of its way draw) as turn 1,
    the phase writes no roll in turn 1, writes *k* = 0 (a roll of 0) in turns 2 and 3, and writes
    each *k* from 1 to 9 in 4 turns, turns 4 to 39. In turn 40, the turn in which *k* would reach
    10, the phase ends and the roll is set to 0, as at a strike. So the phase writes *k* = 0 to 9
    and no roll from *k* = 10. Each push has its own draw (S-0614 claim 10: the first at the end of
    the grab, each later one after a strike), so *σ* can change from one push to the next. It sets
    the roll to 0 in the turn the grab starts, in each strike turn and when it leaves; in the rest
    of the grab the roll moves only by the host's own rocking step. It never changes the roll rate.
    So in a push phase the drawn roll is the parasite's value whatever else rocks the host, while a
    roll rate from a push keeps stepping underneath, from the parasite's last value, and shows once
    the parasite stops writing. The pitch is unaffected.
    - Backing: `measurable` (RK6; part b for the rate showing after a release)
    - Confidence: high for the axis and side; medium for the frame count *k* and for the grab's
      turns after its first, until RK6 records them

15. **Escape past a limit.** A push that lands while an unlinked angle sits exactly at its limit
    (between the step that held it there and the next step) is not held at the next step, because
    the angle is not below the limit. Once beyond its limit the angle is not held again until it
    has come back below it. The same happens on the pitch when the crush-slowdown state begins
    while the pitch is already beyond π/10. Repeated pushes can then carry the angle past the edge,
    after which claim 9 rolls the body over and destroys it (`quirks.rocking_limit_escape`, Edge
    cases and quirks).
    - Backing: `measurable` (RK3)
    - Confidence: high

16. **Crashing.** When an object that is not infantry starts to crash, S-0312 claim 16's three
    draws set its rates: the roll rate is 0.1 + 0.15 × *u*₁, negated when the sense draw gives 0,
    and the pitch rate is 0.1 × *u*₃, each formed in double precision and rounded to single
    (Formulas F5). While it crashes, each angle adds its rate in every turn, with no limit, no pull
    toward level, no stop and no rolling-over destruction. For a balloon-hover type, the pitch is
    then raised to −π/4 when below it (it has no upper bound) and the roll is held within ±π/4.
    - Backing: `measurable` (RK9) for the rates, the growth and the balloon-hover roll bound.
      `unmeasured` for the balloon-hover lower pitch bound: RK9 does not exercise it.
    - Confidence: high for the rates, medium for the balloon-hover bounds

17. **Other writers.** A mind-control overload sets the roll rate to S-0601 F4's value, replacing
    the rate, unscaled. A crush tip of S-0210 claims 6 and 9 sets the pitch rate to S-0210's body
    tilt, replacing it, unscaled, and each overlay crushed under S-0210 claim 8 adds its amount to
    the pitch rate, unscaled. A claim 6 tip happens only while the pitch rate is level, so it never
    replaces a nonzero rate. A negative body tilt raises the bow. The psychic dominator's blast ring
    also moves the angles (Open questions).
    - Backing: `measurable` (RK2) for the value each writer sets. The crush tips' setting and the
      overlay's addition are backed and rated by S-0210 claims 6, 8 and 9. `unmeasured` for the
      overload's replacing a nonzero rate, as S-0601 F4 rates it, and for the weight not scaling any
      of these writers: RK2 starts each writer from rest on a subject of weight 1, which does not
      tell a replaced rate from an added one or a scaled value from an unscaled one. S-0601 MC8
      repeats its pulses, so a later pulse can land on a rolling controller, but it does not compare
      such a pulse with one onto a controller at rest, and it does not vary the weight. No scenario
      here places a crush tip within the ticks a pushed rate lasts.
    - Confidence: high for the overload's value; the crush tips' values, setting and the
      overlay's addition at S-0210's ratings; medium for the overload's replacement and for the
      weight

### What reads the angles

18. **Drawing.** The driving, naval, hovering and jumpjet locomotors draw a body with its pitch
    and roll (S-0106 claim 9, S-0504 F5). The flying locomotor draws them only while the aircraft
    is crashing at strictly positive signed height (claim 20). The walking, mechanical walking,
    drop-pod and teleporting locomotors draw the body without them, although their angles still
    step. The tunnelling locomotor draws the body without them too: with the cell's slope tilt, or
    with its own digging pitch (S-0106). The rocket locomotor draws its own attitude (S-0105
    claim 41) and not the rocking angles.
    - Backing: `measurable` (RK2 part b) for the walking body's angles stepping, and (RK12) for
      the flying locomotor's crashing body. The drawing is `unmeasured` for every other
      locomotor: RK8 records fire points and RK9 records angles and rates, not drawing; no scenario
      here runs the mechanical walking, drop-pod, teleporting, tunnelling or rocket locomotors, and
      RK2 part b records the walking body's angles, not its drawing.
    - Confidence: high for the flying locomotor's crashing body at strictly positive signed
      height; medium for the others, except low for the rocket locomotor's drawing, which is
      believed but not checked as closely as the others.

19. **Fire points.** A body of the driving, naval, hovering and jumpjet locomotors of claim 18
    takes its fire point (S-0313 claim 1) through its body transform: the rocking angles with
    S-0106 claim 9's table tilt, which is the transform the original also draws with. S-0106
    claim 4's presentation-only correction of the drawn tilt does not enter the fire point. So
    rocking moves the point where its shots start, outside sinking as well as in it (S-0504
    claim 16). With claim 9's destruction, this makes rocking game state.
    DECIDED (autonomy, 2026-10-29): our engine simulates the angles and rates as game state,
    advances them once in each turn of the body as this spec gives, and builds fire points on them.
    The view only reads them. The rocking step is the first part of the body's turn, before its
    movement and its firing (in the arrival hold, after the hold-end test), so a shot fired in a
    turn starts from the angles after that turn's step, and a push in that turn shows from the next
    one. Which order the original uses is open (Open questions).
    - Backing: `measurable` (RK8)
    - Confidence: medium

20. **A crashing aircraft's body (Q-2236).** While an aircraft with the flying locomotor crashes
    at strictly positive signed height above the current surface (the height of S-0105 claims 40
    and 42), its body transform is

    > *D*<sub>crash</sub> = Rz(*θ*<sub>flight</sub>) · Rx(*φ*) · Ry(*ψ*′),

    and it is drawn with *V* · *D*<sub>crash</sub> before the HVA, *V* being S-0105's view
    transform (its claim 1), as S-0105 claim 42's transform is. *θ*<sub>flight</sub> is S-0105
    claim 3's 32-step flight drawing heading, from the aiming facing as in flight, not the body
    facing. *φ* is the roll and *ψ* the pitch of claim 16, with the signs of the Terms. So the body
    is pitched about its own lateral axis first, then rolled about its own forward axis, then
    turned to its heading. That is S-0504 F5's order of roll and pitch, roll outermost, and the
    reverse of flight's order Rz(*θ*<sub>flight</sub>) · Ry(*α*) · Rx(*β*) (S-0105 claim 42).
    - No slope tilt, lift or shift enters *D*<sub>crash</sub>: the tumble moves no part of the
      body's placement, which is the same as in non-crashing flight at the same position and
      height.
    - *ψ*′ = *ψ* + *α* when S-0105 claim 42's speed test passes (the current speed fraction
      strictly greater than the imported drawing-pitch threshold), with *α* the imported
      drawing-pitch angle. Otherwise *ψ*′ = *ψ*. The single-precision pitch and the
      double-precision angle are added at 53-bit precision and the sum is stored in single
      precision, truncated toward zero (S-0105 F0). The angle is not first narrowed to single
      precision, as claim 42 narrows it for a non-crashing body.
    - The bank *β* never enters a crashing body, in a turn or not, and whether or not the speed
      test passes.
    - The rotations use S-0105 F1 and F0, as claim 42's do. F1's table index wraps every whole
      turn, so it covers every angle the tumble reaches.
    - At zero or negative signed height, a crashing body is turned to its heading only, with no
      tumble, no *α* and no *β*. This is the original's behaviour, kept: it is the one height rule
      that flight also follows.
    - The shadow follows only the heading and the slope shape of S-0106 claim 11 (with its
      dropship exception), never *φ*, *ψ*, *α* or *β*. Its image key is S-0105's (claim 42).
      The body at positive height is drawn fresh, never reused (S-0105 claim 26).
    - Backing: `measurable` (RK12 parts a to c and e) for *D*<sub>crash</sub>, its order and
      axes, the absent tilt, lift and shift and the shadow's outline; (RK12 part c) for the *α*
      term; (RK12 part e, with the speed test passing in a turn) for the absent bank; (RK12
      part d) for the heading source; (S-0105 scenario MS-32's crash run) for the fresh body.
      `unmeasured` for the precision of the sum *ψ* + *α*, which changes the pitch only in the last
      binary digits of a single-precision value, so no outline can show it; for the
      case at zero or negative height, which a crashing aircraft reaches only as it meets the
      ground; and for the shadow's key, since pixels alone do not show reuse.
    - Confidence: high for *D*<sub>crash</sub>, its order and axes, the heading source, the absent
      tilt, lift, shift and bank, the *α* term, the shadow's outline and the fresh body; medium
      for the precision of the sum, for the case at zero or negative height and for the shadow's
      key.

## Formulas

**F1. A push.** Inputs: the body's position *B* and the push point *Q*, integers in leptons; the
single-precision strength *s*; the pitch factor *h*; the double-precision weight *W*; the body's
facing *f* (0 … 65535). Every arithmetic result is RZ₅₃ and every value stored in single precision
is RZ₂₄ (S-0205). 0.04, 2.5 × 10⁻⁵, 0.01, 0.05 and ½ are single-precision constants, widened.

> *w* = *B* − *Q* per axis (*x*, *y*, *z*), integers.
> *d* = *r*(*w*ₓ² + *w*ᵧ² + *w*_z²), *r* S-0204's table square root (its F1) of the exact sum,
> a single value, not truncated. No change when RZ₂₄(*d*) < 2 × 10⁻⁵, that is when *w* = 0.
> *k* = RZ₂₄(((0.04 − *d* × 2.5 × 10⁻⁵) × *s*) / *W*). No change when *k* < 0.01 or *k* is
> not a number. When *k* > 0.05, *k* = 0.05.
> *ℓ* = *r*(*w*ₓ² + *w*ᵧ²), the same table root. When *ℓ* = 0, both rates become 0. With
> S-0204's `quirks.table_root_distance` off, *d*, *ℓ* and *m* are the exact square roots,
> RZ₅₃(√…), not truncated to integers.
> *a*ₓ = RZ₂₄(*w*ₓ / *ℓ*), *a*ᵧ = RZ₂₄(−*w*ᵧ / *ℓ*).
> *g* = *f* as a signed 16-bit value, minus 16383 (an integer from −49151 to 16384, not
> reduced again), *θ* = *g* × (−π / 32767), with −π/32767 a double-precision constant: S-0204's
> short scale, which S-0204's `quirks.table_direction` replaces when off, as S-0204 gives.
> *c* = cosT(*θ*), *n* = sinT(*θ*), single values.
> forward = RZ₂₄(*a*ₓ × *c* + *a*ᵧ × *n*).
> *e* = RZ₅₃(*a*ₓ × *n* − *a*ᵧ × *c*), the vertical component of the cross product of the unit
> direction and the facing, not rounded to single.
> *m* = √ₜ(RZ₅₃(*e*²)), the side magnitude: S-0105's table square root (its F1) of the square of
> *e*, a single value.
> side = −*m* when |forward × *c* − *m* × *n* − *a*ₓ| > 0.0002 or |*m* × *c* + forward × *n* −
> *a*ᵧ| > 0.0002, else +*m*. (side is the leftward component: *a* = forward × (*c*, *n*) +
> side × (−*n*, *c*) up to rounding.)
> pitch rate = RZ₂₄(forward × *k* × *h*), roll rate = RZ₂₄(−(side × *k*)).

The sign test's tolerance 0.0002 is a double-precision constant. Each component of *w* is converted
to single precision and squared and summed in double precision, so no result wraps (S-0205) and
every square and partial sum is exact: the components stay below 2²⁴ leptons for height differences
below 2²⁴ leptons, at most 10 leptons from the body for a centre-cell push or a direct-rocking hit,
about 2 for a drone, and within the 7 × 7 cells and the height difference for the other area pushes.
That bound is F1's input domain. S-0204 gives no bound on height differences, so for a component
of 2²⁴ leptons or more this formula is not established, and what the original computes then is
open.

For *m* the argument is a real number at most about 1, for which S-0105 F1 defines the table square
root: it is the same table as S-0204's *r*, which S-0204 F1 gives for integer arguments. *m* differs
from |*e*| by up to the table's relative error (below 2⁻¹³, S-0105 F1), so the roll rate keeps about
13 significant binary digits of the exact side component, while the pitch rate keeps single
precision of the normalised component. DECIDED (autonomy, 2026-10-29): S-0204's
`quirks.table_root_distance` covers *m* as it covers *d* and *ℓ*, so with it off F1 takes no table
root at all, and *m* = RZ₅₃(√RZ₅₃(*e*²)).

**F2. Strengths.** Area damage, with the integer damage *D* of S-0303:

> *s* = RZ₂₄(min(RZ₅₃(*D* × 0.01), 4.0)), with 0.01 and 4.0 double-precision constants. The push
> happens only when the double-precision value before rounding exceeds 0.3 (S-0303 claim 20).

Direct-rocking hit, with *A* the integer amount the detonation delivers: S-0302 claim 2's
⌊*D*ₜ × *M* / 256⌋ from the damage remaining at detonation and the integer support multiplier,
under S-0518 F1's signed 32-bit product-wrap rule, so the strength takes exactly the delivered
amount for every damage and multiplier:

> *s* = RZ₂₄(min(RZ₅₃(*A* × *c*<sub>d</sub>) / 100, 4.0)), with 100 and 4.0 double-precision
> constants.

**F3. Drone push point.** With the host's position (*X*, *Y*, *Z*) and facing *f*, and the side
draw *r* (0 or 1):

> *g* = (*f* − 16383) as a signed 16-bit value, *t* = (*g* − 16383) × (−π / 32767),
> *σ*₂ = +2 when *r* = 0 and −2 when *r* = 1,
> *Q* = (trunc(*X* + *σ*₂ × cosT(*t*)), trunc(*Y* − *σ*₂ × sinT(*t*)), *Z*).

The two subtractions of 16383 are not one subtraction of 32766: the first difference is taken as a
signed 16-bit value before the second is subtracted. (cosT(*t*), −sinT(*t*)) is the map direction of
the facing a quarter turn anticlockwise from the host's, its left. Truncation is of the sum, so the
shift is whole leptons and not quite symmetric between the two sides.

DECIDED (autonomy, 2026-10-30): S-0204's `quirks.table_direction` covers F3's two subtractions and
its scale, as it covers F1's. With it off, *g* = (*f* − 16384) as a signed 16-bit value and *t* =
(*g* − 16384) × (−2π / 65536), in double precision, so the direction is a quarter turn anticlockwise
from the host's facing on S-0204's exact scale. The tables cosT and sinT and the truncation of the
sum are the same with the switch on or off.

**F4. Grappling parasite's roll.** With phase frame *k* = 0, …, 9 as claim 14 counts it and
*σ* = ±1 (claim 14):

> roll = RZ₂₄(sinT(*k* × *σ* × π/10) × π/4), with π/10 a double-precision constant and π/4 a
> single-precision one.

**F5. Crash tumble.** With *u*₁ and *u*₃ the first and third draws of S-0312 claim 16 as S-0202
unit-interval values (doubles), and *b* the second draw (0 or 1):

> roll rate = RZ₂₄(RZ₅₃(*u*₁ × 0.15) + 0.1), negated when *b* = 0; pitch rate = RZ₂₄(*u*₃ × 0.1).
> 0.15 and 0.1 are double-precision constants.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Push base | 0.04 | radians per tick | F1, single |
| Push fall-off with distance | 2.5 × 10⁻⁵ | radians per tick per lepton | F1, single |
| Smallest push | 0.01 | radians per tick | F1, single; below it nothing changes |
| Push cap | 0.05 | radians per tick | F1, single |
| Pitch factor of area damage and drones | ½ | – | claims 1, 11 and 13 |
| Side sign tolerance | 0.0002 | – | F1, double |
| Level band half-width | 2 × 10⁻⁵ | radians | claims 5 and 7, double |
| Small pull | 0.002 | radians per tick per step | claims 7 and 9, single |
| Large pull | 0.0050000002374872565 | radians per tick per step | claims 7 and 8, double; written 0.005 elsewhere |
| Usual limit | π/4 | radians | claim 6, single |
| Crush-slowdown pitch limit | π/10 | radians | claim 6, single |
| Edge | π/2 | radians | claim 9, single |
| Rolling-over angle | π | radians | claim 9, single |
| Push point distance toward a source or shooter | 10 | leptons | claims 11 and 12, per axis truncated |
| Drone point distance | 2 | leptons | F3 |
| Direct-rocking divisor | 100 | – | F2 |
| Strength cap | 4.0 | – | F2 |
| Crash roll rate, base and spread | 0.1, 0.15 | radians per tick | F5 |
| Crash pitch rate spread | 0.1 | radians per tick | F5 |
| Weight at type creation | 1.0 | – | claim 10 |
| *c*<sub>d</sub> and *c*<sub>f</sub> engine defaults | 1.0 | – | Import mapping |

## State and ordering

**Table R, one axis in one rocking step** (not crashing; of a sinking body, the roll only). *a* is
the angle before the step, *r* the rate after any push. The rows are applied from top to bottom
within each part.

| Part | Condition | Result |
|---|---|---|
| move | *r* = 0 exactly | angle becomes 0, rate stays 0, the rest of the step does nothing |
| move | otherwise | new angle = RZ₂₄(*a* + *r*) |
| hold | not linked, *a* not past the edge, *a* < *L*, new angle > *L* | angle = *L*, *r* = 0 |
| hold | not linked, *a* not past the edge, *a* > −*L*, new angle < −*L* | angle = −*L*, *r* = 0 |
| pull | *a* > π/2 | *r* + 0.002 (away from level) |
| pull | *a* < −π/2 | *r* − 0.002 (away from level) |
| pull | linked, *a* > 2 × 10⁻⁵ | *r* − *c*<sub>f</sub> × 0.002 |
| pull | linked, *a* ≤ 2 × 10⁻⁵ | *r* + *c*<sub>f</sub> × 0.002 |
| pull | roll, *a* > 2 × 10⁻⁵, *r* ≤ 0 | *r* − 0.005 |
| pull | roll, *a* > 2 × 10⁻⁵, *r* > 0 | *r* − 0.002 |
| pull | roll, *a* ≤ 2 × 10⁻⁵, *r* ≥ 0 | *r* + 0.005 |
| pull | roll, *a* ≤ 2 × 10⁻⁵, *r* < 0 | *r* + 0.002 |
| pull | pitch, *a* > 2 × 10⁻⁵ | *r* − 0.005 |
| pull | pitch, *a* ≤ 2 × 10⁻⁵ | *r* + 0.005 |
| stop | *a* > 2 × 10⁻⁵ and new angle < 2 × 10⁻⁵ | angle = 0, *r* = 0 |
| stop | *a* < −2 × 10⁻⁵ and new angle > −2 × 10⁻⁵ | angle = 0, *r* = 0 |

Only the first matching row of the *pull* part applies. Each pulled rate is rounded to single.

**Table S, the body's state in a turn.**

| State | In each turn | Destruction check |
|---|---|---|
| sinking | S-0504 claim 15's lean for the pitch, pitch rate unchanged; table R for the roll (claim 5). A sinking body in a warp takes the warp row instead | none |
| in a superweapon warp, from the first pending update through the arrival effects (S-0513 claim 51) | nothing: angles and rates hold | none |
| crashing | each angle + its rate, rounded to single; balloon-hover bounds (claim 16) | none |
| otherwise | table R for the roll, then for the pitch | claim 9, after both |

Ordering, as observable consequences:

- In our engine the rocking step comes first in the body's turn, before its movement and firing
  (claim 19).
- A push takes effect at the body's next rocking step: the first angle after a push from level is
  the pushed rate itself.
- Pushes replace, so of several pushes before the same turn only the last counts (claim 2).
- In a host's turn, a grappling parasite's roll is written after the host's rocking step, so that
  value is the one drawn and the one the next step starts from (claim 14).
- A body destroyed by rolling over takes no further part in that turn (claim 9).
- Simulated by table R, with RZ₂₄ for every stored value, a push from level gives (peak angle in
  radians, the step of the peak, the step at which the angle is level again):

| Axis | Starting rate | Peak | Step of peak | Level at step |
|---|---|---|---|---|
| roll | +0.05 | π/4 (held) | 23 | 41 |
| roll | −0.05 | −0.65 | 25 | 42 |
| roll | +0.03 | 0.354 | 19 | 32 |
| roll | −0.03 | −0.24 | 15 | 26 |
| roll | +0.015 | 0.125 | 11 | 19 |
| roll | −0.015 | −0.064 | 8 | 14 |
| pitch | +0.02 | 0.095 | 6 | 13 |
| pitch | −0.05 | −0.275 | 10 | 21 |
| pitch | +0.025 | 0.13 | 7 | 15 |
| pitch | −0.025 | −0.075 | 5 | 11 |

## Edge cases and quirks

- **Limit escape** (`quirks.rocking_limit_escape`, default on). DECIDED (autonomy, 2026-10-29):
  one switch, defaulting to the original's behaviour. On: claim 15, so repeated pushes can carry
  an unlinked body past its limit and over, destroying it (claim 9). Off: an unlinked angle that
  is not past the edge is held at ±*L* whenever its new angle would pass ±*L*, whatever the angle
  before the step.
- **No limit while melee-linked.** A linked body has no limit and is pulled back only by
  *c*<sub>f</sub> × 0.002 per step, so repeated direct-rocking hits can roll it over (claim 9). This
  is the original's design and gets no switch.
- **Level-band asymmetry.** Claim 7's first-step difference between positive and negative pushes
  from level is the rule, copied, with no switch.
- **A rate of exactly 0 snaps the angle level.** Any angle whose rate is exactly 0 at a step becomes
  0 at once (table R). A push from straight above or below (claim 4) therefore levels a rocking body
  in one step. This is the rule, with no switch.
- **Weight of 0 or below.** A weight of 0 gives the cap 0.05 for every push whose numerator and
  strength have a positive product, and nothing otherwise. A negative weight with a positive
  strength gives nothing within claim 3's reach, and beyond it a push that rocks the body when
  *k* is at least 0.01 (the threshold still applies there). Both follow from
  the formula, with no switch.
- **The sinking roll.** No switch: S-0504 fixed this at the delegated sign-off (2026-10-25). Our
  engine steps a sinking body's roll by table R (claim 5). The original holds the roll and both
  rates for the whole descent, so a push during descent sets the rates and never rolls the hull
  (RK11).

## Import mapping

| Source | Key | Feeds | Parse form | Absent |
|---|---|---|---|---|
| rules, object type section (every kind) | `Weight` | *W* (claims 3 and 10) | decimal, held as a double | keeps the current value: 1.0 at type creation (S-0504 claim 11 for vehicles; this spec's default for the other kinds, claim 10) |
| rules, `[AudioVisual]` | `DirectRockingCoefficient` | *c*<sub>d</sub> (F2) | decimal, held as a single | keeps the current value: 1.0, an engine default (Constants) |
| rules, `[AudioVisual]` | `FallBackCoefficient` | *c*<sub>f</sub> (table R) | decimal, held as a single | keeps the current value: 1.0, an engine default (Constants) |
| rules, warhead | `Rocker` | area-damage rocking (S-0303 claim 20) | boolean | as S-0303 |
| rules, warhead | `directrocker` | direct-rocking row (S-0514 claim 20) | boolean | as S-0514 |
| rules, vehicle type | `BalloonHover` | balloon-hover bounds while crashing (claim 16) | boolean | as S-0310 |
| rules, object type | `Strength` | damage of the rolling-over hit (claim 9) | integer | as S-0302 |

## Measurement scenarios

All scenarios use stock data under the oracle harness with its house director and gameplay changes
off, a tiny flat test map, and a
recorder that reads each subject's pitch, roll, pitch rate and roll rate (radians and radians per
tick), position and facing once per tick, as S-0106 T11's recorder does. Subjects are copies of a
driving vehicle type with a weight *W* the test sets, except where a part names another kind.
Expected values are functions of *W*. Pushes come from test weapon copies whose warhead has the
rocking flag and versus values that do no damage to the subject, with a damage amount *D* and spread
the test sets. Expected values come from F1 to F5 and table R, computed by our engine for the same
inputs.

**RK1. Push split and strength (claims 1 to 3, 11).** *W* = 1. Facings 0, 8192, 16384 and 40960.
Detonations with no source object in parts a and d, and in the other scenarios that name them, are
made by the measurement harness, which detonates the test warhead at *P* with no source object and
no firer. (a) Detonations with no source object at *P* one cell ahead, behind, left, right and at
45° of the
subject, *D* = 50, *D* = 73 (a strength that is not exact in single precision, for F2's rounding)
and *D* = 400. (b) Centre-cell detonations from a test firer standing two cells away at each
bearing, with *P* on the subject's far side. (c) Two detonations in one tick from opposite sides.
(d) Detonations with no source object at *P* inside the subject's cell, 64 leptons from its position
ahead, behind, left and right. (e) As (b), with a test firer whose projectile flies for several
ticks, destroyed by scripted damage while its projectile flies. (f) As (e), with the firer loaded
into a closed transport while its projectile flies instead of destroyed. Observable: the first
nonzero pitch and roll after each push; in (f), also the firer's recorded position at loading.
Expected: the pushed rates of F1 (from level the first angle equals the rate); the side or end
nearer *Q* rises; in (b) the subject is pushed away from the firer, not from *P*; in (c) only the
later push shows; in (d) and (e) the subject is pushed away from *P*; in (f) it is pushed away
from the firer's position at loading. A rate that is added, a split by any
other rule, a pitch not halved, or no push in (d) falsifies.

**RK2. Steps and the return to level (claims 5, 7, 8, 17).** One pushed subject per row of the State
and ordering simulation table, each from an input that reaches it unlinked: area pushes for the roll
rows and the pitch rows of ±0.025, a crush tip for −0.05 on the pitch, an overlay crushed from level
for +0.02 on the pitch (S-0210 claim 8), and both overload roll rates of S-0601 F4 for the roll rows
of ±0.015 and ±0.03 as well. Observable: the angle in every tick until level. Expected: the table's
peaks, peak steps and level steps, no swing through level, and the positive-negative asymmetry. An
overshoot past level, an exponential or spring-like decay, or equal positive and negative curves
falsify. (b) The area-push rows of ±0.03 on the roll and of ±0.025 on the pitch, repeated on test
copies of a ship type on water, a hovering type, a jumpjet type standing landed and a walking
vehicle type (recorder only), each with *W* = 1. The landed jumpjet is listed in its cell's ground
layer (S-0502 claim 4), the layer S-0303 claim 20's rocking searches. In high hover it has no
listing (S-0502 claim 22), so an area push would not reach it. Expected: the same angles per tick
as the driving
subject's. A difference falsifies claim 5 for that locomotor.

**RK3. Limit hold, escape and rolling over (claims 6, 9, 15).** (a) A sideways push giving a roll
rate of +0.05: the roll is π/4 for exactly one tick, then returns. (b) The same subject with
*W* set so every push is capped, pushed sideways in every tick for 100 ticks. Observable: roll per
tick and whether and when the subject is destroyed, with its attacker and the warhead of the
killing hit. Expected: held at π/4 once, then beyond it on the next push, past π/2, and destroyed
by a demolition-warhead hit with no attacker once |roll| > π. With the switch off in our engine:
held at π/4, no destruction. A roll that stays at π/4 in the original falsifies claim 15. (c) Part
(a) on RK2 (b)'s ship, hovering and jumpjet copies. Expected: the roll is π/4 for exactly one tick,
then returns, as on the driving subject. A roll that passes π/4 or stays there falsifies claim 6 for
that locomotor.

**RK4. Crush-slowdown limit and the melee link (claims 6, 7, 12).** (a) A subject in the
crush-slowdown state (S-0210 scenario K5's setup) pushed from behind in every tick: the pitch is
first held at π/10 for one tick, not at π/4, and then escapes as in RK3. (b) With data that sets
the direct-rocking flag on a test weapon copy and *c*<sub>f</sub> set by the test (1.0 and 0.5),
a shooter directly behind the subject: full pitch rate, no limit at π/4 or π/10, and a pull of
*c*<sub>f</sub> × 0.002 per step while the shooter keeps its target. The shooter and the subject
are fresh objects of the game that no earlier hit has linked. (c) Control: the same pair with the
flag off on the copy, the subject rocked by RK1 (a)'s abeam push while the shooter fires at it.
Expected: the roll takes the unlinked step of claims 6 and 7 and is held at π/4, with no pull of
*c*<sub>f</sub> × 0.002 per step. Observable: pitch per tick in (a) and (b); roll per tick in (c).
A held pitch in (b), or a roll in (c) that passes π/4 or is pulled by *c*<sub>f</sub> × 0.002
per step, falsifies. (d) Strength: (b)'s pair, fresh in each run, on a subject copy with *W* =
4.8, from level, with *c*<sub>d</sub> set by the test to 1.0 and to 2.0 and the test weapon's
damage set to 200 and to 300 (four runs), from a shooter whose support multiplier is neutral, so
that *A* is the damage (S-0302 claim 2). Observable: the first pitch after the first hit, which
from level equals the pitch rate. Expected: F1's rate for *s* = 2.0, 3.0, 4.0 and 4.0, with *k* in
the band from 0.01 to 0.05 in each run, so neither of F1's bounds acts: the first three in
proportion to *A* × *c*<sub>d</sub>, and the fourth equal to the third (the strength's cap of
4.0; without it *k* would be about 0.05). Rates not in that proportion (for example equal rates
for the two factors at damage 200), or a fourth rate above the third, falsify the strength.

**RK5. Drone side (claims 10, 13).** S-0614 scenario P1's setup with hosts of *W* = 1 and
*W* = 2 at several facings. Observable: the sign and size of the first roll after each delivery,
matched with our engine's draw for the same seed. Expected: positive roll (left side up) for a
draw of 0, negative for 1, magnitude from F1 and F3 for each weight, with F1's cap.

**RK6. Grappling roll (claims 10, 14).** S-0614 scenario P6's setup, with hosts of *W* = 1 and
*W* = 2. Observable: the host's roll and pitch per tick, and each push's way draw, through the grab
and the push phases until two pushes of one run have different way draws (more seeds if needed),
with and without a detonation that rocks the host from abeam during a push. Expected: F4's roll at
each frame with the sign of that push's own draw, changing sign between the two pushes, the same for
both weights and unchanged by the detonation, whose pitch still shows. After a strike, a leftover
roll rate moves the roll from 0. In each push phase, counted from the turn of its way draw as turn
1: turn 1 shows no roll from the phase, turns 2 and 3 a roll of 0, turns 4 to 39 F4's roll for *k* =
1 to 9, four turns each, and turn 40 a roll of 0. In the grab, after its first turn, the roll
follows only the host's own step. A shift of the frames by one turn falsifies the frame count. (b)
The detonation's push during a push phase, then a release of the host under S-0614 claim 13 before
the pushed rate dies out. Expected: roll 0 at the release, then moving from 0 under the leftover
rate. A roll held at 0 after the release falsifies claim 14's last sentence.

**RK7. A push from straight above (claims 4, 11).** A subject at the centre of its cell, rocking
from an earlier push, then hit by a test jumpjet firer hovering over the same cell centre.
Observable: both angles in the following ticks.
Expected: both level at the next tick. An angle that keeps its value or moves on falsifies.

**RK8. Fire point (claims 18, 19).** A test vehicle with a muzzle height *H* (the test art of
S-0313 scenario P1 part b, whose muzzle position has a non-zero height part) fires at a fixed
target while rocked by a sideways push, and again at rest. Observable: the
shot's start position. Expected: shifted toward the lowered side by about *H* × sin(roll). (b) The
same with a hovering and a jumpjet subject. The jumpjet, hovering to fire, is rocked by RK4 (b)'s
direct-rocking hit from abeam with a test weapon copy that can target it, since an area push does
not reach a jumpjet in high hover (RK2 b). A run in which that hit does not rock it is void. An
unshifted start position falsifies claim 19.

**RK9. Crash tumble (claims 16, 18).** Aircraft and jumpjet test copies destroyed in the air, and a
balloon-hover copy. Observable: rates at the start of the crash and angles per tick. Expected:
roll rate magnitude from 0.1 up to 0.25, pitch rate from 0 up to 0.1, matched to our engine's
draws, and angles grow by the rate every tick with no return. The balloon-hover roll stays
within ±π/4.

**RK10. Weight (claims 3, 10).** RK1 (a)'s abeam push on copies with *W* = 1, 2, 0.5, 0 and −1, and
on a voxel-drawn aircraft copy (recorder only). Expected: F1's rates for each weight, including no
change where *k* falls below 0.01, and the cap; *W* = 0 gives the cap, *W* = −1 gives no rocking,
within the reach. Rocking of the *W* = −1 copy falsifies claim 3's negative case within the reach.

**RK11. A push while sinking (claim 5, Edge cases).** A naval test copy reduced to sinking (S-0504
claim 15's scenario setup), then hit abeam during its descent by RK1 (a)'s detonation. Observable:
roll, roll rate and pitch per tick through the descent. Expected in the original: the roll rate
takes the pushed value and the roll does not change from its value when sinking began; the pitch
follows S-0504 claim 15's lean. A roll that moves during the descent falsifies the Edge cases
entry. Expected in our engine: the roll follows table R from the pushed rate and returns to level,
and the pitch follows the lean.

**RK12. A crashing aircraft's drawing (claims 18, 20).** RK9's aircraft run, on a test aircraft
type with the flying locomotor whose body is an asymmetric test model: a long arm along its forward
axis, a fin on its left side only and a block on its top only, so that its outline differs under
every nonzero roll, pitch and heading and under either order of roll and pitch. The test sets the
type's drawing-pitch angle and bank angle to large nonzero values. The aircraft is destroyed in the
air by scripted damage, as in RK9, with screenshots in every tick from the crash start until it
leaves play (the bench at 800 × 600). The run uses a recorder that reads, in the same ticks, the
aircraft's roll, pitch, aiming facing, body facing, current speed fraction and signed height (more
than RK9's records, which hold only rates and angles). (a) Flying straight along one map axis,
with the drawing-pitch threshold set by the test above every speed fraction the recorder reads (a
run whose recorded fraction passes it is void). (b) As (a), flying along a diagonal. (c) As (a)
and (b), with the threshold set to zero. (d) Shot down while it turns its aim toward a target off
its course (S-0501 claims 19 and 42), so that the recorded aiming facing and body facing fall in
different 32-step buckets. A run in which they share a bucket in every frame of the crash is void.
(e) As (c), with the threshold at zero, shot down during a turn of its aiming facing, the recorder
showing the facing change between ticks. Frames are compared from the first tick in which both
recorded angles are at least 0.3 radians in magnitude; a run whose pitch never gets there before
the aircraft leaves play is void (another seed). In (c), the *α* term is judged only on compared
frames whose recorded speed fraction is strictly positive, so that S-0105 claim 42's speed test
passes; a run of (c) with no such frame decides nothing for the *α* term (another seed). In (e),
the bank is judged only on compared frames whose recorded speed fraction is strictly positive and
whose recorded aiming facing has changed since the previous tick, so that the speed test passes
during a turn; a run of (e) with no such frame decides nothing for the bank (another seed).
Observable: the body's outline and the shadow's outline in each compared frame, beside the recorded
values. Expected: each body outline matches our engine's render of claim 20's *D*<sub>crash</sub>
for the recorded roll, pitch, aiming facing and speed fraction, placed as that render places
it; in (c) the pitch is *ψ* + *α* in each judged frame; in (e) the judged frames show no bank; the
shadow outline is the one level flight has for the recorded heading and cell, unchanged while only
the angles change. An outline that matches the reverse order Rz · Ry(*ψ*′) · Rx(*φ*) instead, a
displacement of the outline that grows with |sin *φ*| or |sin *ψ*| (S-0504 F5's lift and shifts),
an outline in a judged frame of (e) that matches *D*<sub>crash</sub> with S-0105 claim 42's bank
for the recorded turn added, a pitch in a judged frame of (c) equal to (a)'s at the same recorded
angles, an outline in (d) turned to the body facing, or a shadow that changes with the tumble
falsifies claim 20.

## Changes for other specs

- **For S-0105 claim 42 (*Flying bodies pitch and bank*) and Terms (*Flight bank*, *flight
  drawing pitch*).** Its sentence "This claim does not replace S-0312's crash attitude rules" can
  point to S-0113 claim 20, which gives a crashing flying body's transform at strictly positive
  signed height: Rz(*θ*<sub>flight</sub>) · Rx(*φ*) · Ry(*ψ*′), with *ψ*′ the tumble pitch plus
  *α* under this claim's speed test, and no *β*. Its shadow sentence, "never these pitch or bank
  angles", holds for a crashing body's tumble angles as well. The Terms entry's "for a
  non-crashing flying-locomotor body" can add that *α* also enters a crashing body's pitch
  (S-0113 claim 20). Confidence: high. Backing: `measurable` (S-0113 RK12). Its sentence on
  narrowing the configured angles to single precision can add that a crashing body's *α* is not
  narrowed first: it is added to the tumble pitch at 53-bit precision and the sum is stored in
  single precision (S-0113 claim 20). Confidence: medium. Backing: `unmeasured`, as S-0113 claim
  20 gives.

- **For S-0106 claim 11 (*Shadow tilt*).** Its "Neither shadow includes body pitch or bank" also
  covers a crashing aircraft's tumble roll and pitch (S-0113 claim 20): the shadow of a crashing
  flying body keeps the transform S-0106 claim 11 gives. Scenario T4's attitude part can
  cite S-0113 RK12 for a crashing aircraft. Confidence: high. Backing: `measurable` (S-0113 RK12).

- **For S-0504 claim 15 (*The pitch belongs to the lean; the roll is left free*).** Its statement
  that no approved spec gives how a rolled body comes back to level, and its open question on
  whether the roll returns to level while the vessel sinks, are answered by S-0113 claims 5 and 7:
  in our engine the roll takes table R's roll step while the pitch takes the lean, so a roll left
  over from before sinking, or set during descent, returns to level. Its Edge cases entry on the
  original's held roll can cite S-0113 scenario RK11 as the run that records it.

- **For S-0210 Terms (*Body tilt*), claims 6, 8 and 12 and Constants.** The body tilt is S-0113's
  pitch rate, in radians per tick, and the forward rocking angle is its pitch. A negative body tilt
  makes the pitch negative, which raises the bow and lowers the stern (S-0113 Terms). Both are game
  state: S-0113 claim 19 builds fire points on the angles, and claim 9 destroys a body that pitches
  or rolls over. So the Terms' "a presentation value", claim 6's "This only affects how it is
  drawn", claim 8's "which affects only the drawing", claim 12's "presentation only" in its title
  and its sentence "This changes only how the vehicle is drawn", and the Constants' "tilt units per
  tick (rendering)" are replaced by S-0113 claims 9 and 19, the unit being radians per tick. Claim
  12's limit is S-0113 claim 6's π/10: a pitch that would pass it is held there and its rate becomes
  0 for that one step only, because the pull of claim 7 acts in the same step, so the pitch starts
  back at the next turn rather than staying at the limit. S-0113 claim 15's escape applies to it, as
  to every limit.

- **For S-0210 Terms (*Body tilt*, *Tipping forward*), claim 6 and Constants: the direction.**
  S-0113's "A negative body tilt raises the bow" (claim 17, Terms) reverses S-0210's naming. The
  Terms' "forward (negative) or back" reads "back (negative: the bow rises) or forward", the
  term *Tipping forward* and claim 6's "tips forward" read *tipping back* and "tips back", and the
  three Constants rows "Body tilt set when tipping forward" read "when tipping back". The values
  and their signs are unchanged. The rest of the direction fix (claim 6's "pitches back, not
  forward", claims 8 and 9 and the scenario directions) is committed S-0504's Changes for other
  specs entry for S-0210, which this entry is a part of.

- **For S-0210 scenarios K9 and K10 part c.** Both need one explosion to pitch a level body past
  18 degrees. Under S-0113 claims 1 and 3 no single area push does that: its pitch rate is at most
  half the cap, which peaks well below 18 degrees (State and ordering, simulation table). Their
  runs therefore decide nothing (K9's "Runs in which B stays below 18 degrees decide nothing").
  S-0113 scenario RK4 part a, which pushes a slowed body from behind in every tick, tests claim
  12's limit in their place. K10 part c can push A and B in the same way, and its walking vehicles
  need a recorder of the angles instead of screenshots, since that locomotor draws the body without
  them (S-0113 claim 18).

- **For S-0504 claim 16 (*What reads it*).** Its permission that our engine may hold the angles
  as presentation state is replaced by S-0113 claim 19: the angles and rates are game state,
  advanced once in each turn of the body, and the view only reads them.

- **For S-0204 claim 5 (*The table square root*).** Its switch `quirks.table_root_distance` also
  covers the three table roots of S-0113 F1, the distance *d*, the horizontal length *ℓ* and the
  side magnitude *m*, so that one switch decides every table root of a push.

- **For S-0302 claim 1 (*Hits, their two markers and where they come from*).** Its table can list
  S-0113 claim 9's rolling-over hit as a source: damage equal to the body's type's strength, the
  demolition warhead, ignoring defences, no attacker and no attacking house, and without the
  no-escape marker.

- **For S-0214 Terms (*Melee-linked*) and its Open questions on what creates the link.** S-0113
  claim 12 identifies the link flag with S-0514 claim 20's direct-rocking flag: the hit that
  the direct-rocking row selects, once its target passes the further check, both pushes and
  links (low, as S-0214 rates the link's creation). S-0214's Import mapping row for the link flag
  can point to S-0514's direct-rocking warhead row. S-0113 scenario RK4 parts b and c are the
  runs that would show it: in (b) a fresh pair takes the pull of *c*<sub>f</sub> × 0.002 per step
  that only a linked body takes, and in (c) hits without the flag do not.

## Open questions

- Whether the original takes the rocking step during a superweapon warp. Our engine does not
  (table S), with S-0513 claim 51's other ordinary work. In our engine, the angles and rates of a
  body rocked as it warps hold during the excluded warp updates and resume their steps
  afterwards.

- The psychic dominator's blast ring (S-0612 claim 8) also moves the angles, by a formula not
  established here (Q-2081).
- (Claim 11, Q-2081 question 5.) For a centre-cell body, whether a firer that has left play
  without being killed, such as one loaded into a closed transport while its projectile flies,
  is still the detonation's source, and from which position it then pushes. Expected (low): it
  is, from its position when it left play. RK1 part f tests it.
- (Claim 11.) Which square root and which precision the original uses for the components of the
  unit vector *v* before trunc(10 × *v*). Only components that fall within rounding of a whole
  lepton depend on it.
- The direct-rocking hit applies one further check of the target that is not established, so the
  set of vehicles it rocks is of medium confidence (claim 12).
- Where the original's rocking step falls in a body's turn relative to its movement and firing,
  which decides whether its shot in a turn uses the angle before or after that turn's step. Our
  engine puts the step first (claim 19).
- How the hovering and jumpjet locomotors compose pitch and roll with their other transforms
  (S-0106 claim 9, S-0504 F5).
