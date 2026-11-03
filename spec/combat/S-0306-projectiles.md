---
id: S-0306
title: Projectiles (launch, flight, homing, scatter, detonation)
topic: combat
revision: 65
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-03
measurements: []
requests: [Q-0071, Q-0077, Q-0078, Q-0083, Q-0160, Q-1405, Q-1570, Q-1761, Q-1763, Q-2111, Q-2157, Q-2158, Q-2212, Q-2216, Q-2231, Q-2232]
supersedes: []
---

# S-0306: Projectiles (launch, flight, homing, scatter, detonation)

## Summary

How the projectile of a shot is launched (its speed, aim point, scatter, launch direction and
elevation), how it moves each tick by kind (straight, arcing, vertical, homing, invisible), what
makes it detonate, where, and which rule wins when several apply in one turn; the anti-air and
anti-ground eligibility of a weapon; the line-of-fire walk that cliffs and walls block; and the
nuclear warhead, whose detonation waits for an animation. The area damage of a detonation is S-0303,
the range test is S-0307, where the shot starts is S-0313 and when shots are fired is S-0301.
Claim 27 also owns the railgun shot's direct hits and cliff-collapse chance, separately from
its particle trail. Claims 28 to 33 give how the projectile's body is drawn: its sprite frame or
voxel orientation, palette and brightness, its shadow and its light glow. Claim 23 also gives
the firing check's refusals that depend on the firer's type rather than its weapon: the
land-targeting mode's row and the naval-targeting mode's row. The two departures under Edge
cases that state our engine's own choice (the object numbering and the failed comparison solve)
are design, which is why `design` is among this spec's sources. Claim 17's two rules for a
superweapon warp member (its deck count after a refused placement, and no locomotor seizing of
it from departure until its arrival hold ends) are design as well. This spec covers skirmish.
Network games and campaigns are out of scope.

This revision changes claim 23 of the committed text in two places. It states the rank of a
building's pending delayed shot, which the committed text left unstated: its *rearming* comes ahead
of the two rows and of every other row of the claim (S-0301 claims 9 and 14). And a building in
*attack* that holds such a target keeps it until the end of a pending delayed shot instead of
dropping it at once (S-0301 claim 14).

This revision also takes up the shrapnel fragments of S-0318. A fragment is not a shot: claim 1's
shot speed, claim 2's cap and claim 6's pitch do not apply to it, which narrows the committed
claim 6's "every projectile, whatever its kind" (S-0318 claims 5 and 6). Claim 20's (c) does not
move the point from which a burst's fragments leave (S-0318 claim 1). Against revision 35, revision 36 changes no
behaviour: it excepts fragments from claims 5, 7 and 8 as well, says which part of the launch
S-0318 claim 6 replaces, and rests claim 20's burst-point bullet on S-0318.
Against revision 36, revision 37 changes no behaviour: claim 3 points to claim 5's fragment
exception, claim 2 and Open questions list a steering or vertical fragment's start speed as open,
and claim 20's burst-point sentence becomes a note on (c).
Against revision 37, revision 38 changes no behaviour of a shot: State and ordering's first turn of
a projectile whose first turn a removal costs is now in tick *n* + 1 or later, since a fragment that
loses its first turns to removals has its first turn at *n* + 2 or later (S-0318 claim 5), where the
committed text had tick *n* + 1 only. Claim 6 opens with "every projectile except a fragment", and
claim 1's fragment sentence moves from the bullets into the claim's text.
Claim 8 places a projectile whose fire point is not above the terrain 1 lepton above the terrain,
at the same *x* and *y*; claims 5 to 7 and the line-of-fire walk keep the unraised fire point.

## Terms

Glossary terms are used as defined there, in particular *lepton*, *cell*, *height level*, *world
height*, *height above the surface*, *facing*, *direction*, *reference point*, *high bridge*,
*bridge deck*, *render layer*, *weapon*, *warhead*, *target*, *scenario random stream*, *tick*.
From S-0203: *projectile* (the object that carries a shot), *turn*, *enter play*. From S-0204: the
table square root *r*, the direction function F3 with its *table angle* *A* and its facing scale
*c*₂ (about −10430.06 facing units per radian), *signed facing difference* δ. From S-0105: *table
trigonometry* sin_t and cos_t (F1 there). From S-0205: RZ₅₃, RZ₂₄, trunc. From S-0202: R(*a*, *b*),
a ranged draw from the scenario random stream. From S-0301: *firer*, *shot*, *firing check*, *void
shot* (claim 15 there). From S-0312: the *nearby point* N(*Q*, *λ*, snap off) of distance *λ* (F2
there). From S-0303: the warhead's *electromagnetic flag* (F5 there). From S-0313: *fire point* *F*
and the *spray attack* of a firer (claim 9 there). From S-0214: *vehicle-like building* (Terms there).
A *radar event* is a marker of some kind that the interface specs show on the radar (S-0011, claim 32
has the trigger action that raises one). From S-0307: *airborne*, *floor-bound*, *elevation level* of a cell, and the *subject-to-elevation* property (claim
6 there). From S-0211: *firestorm wall*, a house's *firestorm*, *laser-fence flag* and
*inactive laser-fence section* (Terms there). From S-0206:
*homing projectile* (turn-rate field not 0). In addition:

- **Turn-rate field** ρ. The projectile type's integer turn-rate field (Import mapping). S-0206
  calls a projectile homing when ρ ≠ 0; this spec says **steering** when ρ > 0. The two differ only
  for negative ρ (Edge cases).
- **Projectile kinds.** A projectile type's kind is the first of these that fits it: *steering*
  (ρ > 0), *vertical* (vertical flag), *arcing* (arcing flag), *straight* (every other type).
  Independently it may be *invisible* (the
  invisible flag: it reaches its target point at launch), *inaccurate*, *flak-scattering*,
  *dropping*, *airburst*, *very high*, *level*, *floater*, *subject to cliffs*, *subject to walls*,
  *anti-air*, *anti-ground*, *scalable*, *ranged*, *degenerating* (each an imported flag, Import
  mapping). In prose these lower-case names mean the flag. **Flags and kinds.** The launch (claims
  1 to 9) depends on the flags *arcing*, *dropping*, *inaccurate*, *flak-scattering* and *invisible*,
  the voxel-drawn property of the projectile's image and the sign of ρ, whatever the kind: where a
  launch claim says "arcing" it means the arcing flag, so a vertical or steering projectile type
  that also has the arcing flag is launched by the arcing rules (Edge cases). Apart from the start
  speed of claim 2 (and the rescaling of a steering projectile at placement, claim 9), which depend
  on whether the projectile is steering or vertical, the kind decides only the flight (claims 10
  to 15) and T1. A *ranged* projectile that is not
  steering is checked for arrival and for being passed by like a steering one (table T1, the rows
  for "steering or ranged"); a *degenerating* projectile loses damage in flight (claim 18). A
  weapon may have the *area-fire* flag (claim 3 a).
- **Point distances.** For two points with integer lepton coordinates, *d*₃(*X*, *Y*) =
  trunc(*r*(Δ*x*² + Δ*y*² + Δ*z*²)) and *d*₂(*X*, *Y*) = trunc(*r*(Δ*x*² + Δ*y*²)). These are the
  spatial and planar distances of S-0204 (claim 6) between two points, without its building
  reduction β, which a claim here applies only where it says so. The squares and their sum are
  formed in double precision, so they cannot overflow; *r* is the table square root, and the result
  is truncated toward zero. "Spatial distance" and "planar distance" in this spec mean these unless
  a claim says otherwise (claim 1 forms its sum in 32-bit integers, claim 4 in single precision).
- **Gravity** *g*. The game-wide gravity value (leptons per tick per tick, integer), halved for a
  floater projectile: *g*_f = *g* / 2 (double precision).
- **Shot speed** *s*₁. The speed in leptons per tick that claim 1 gives a shot. The symbol *s*₁
  is used only in claim 1, in the cap of claim 2 and in scenario P39.
- **Start speed.** The length that the projectile's first velocity is built with (claims 2, 6 and
  7, F1, F4): the shot speed after the cap of claim 2, replaced by 1 for a steering or vertical
  projectile whose firer type has no radial fire segments. The symbol *s* always denotes the start
  speed.
- **Weapon speed** *S*_w. The weapon's speed field as it stands once the rules are loaded: the
  imported field (Import mapping), except for a weapon whose projectile has ρ = 0, where it is the
  arc speed *v*(*R*₀, *g*) of claim 1, with *g*_f in place of *g* for a floater projectile. It matters in three places only: the top speed of a steering or vertical
  projectile (claims 2, 11, 12), the speed of the children of an airburst (claim 21) and the
  speed of a fragment of S-0318 (claim 1).
- **Arc speed** *v*(*L*, *g*). trunc(*r*(RZ₅₃(RZ₅₃(*L* × *g*) × 1.2))) leptons per tick (F2). It is the
  launch speed that lets an arc reach a little more than *L* on flat ground.
- **Reachability test.** Whether a projectile launched at speed *v* under gravity *g* can reach a
  point *d* away horizontally and *z* higher (F3). S-0307 (claim 9) uses it for the range test of an
  arcing projectile; the launch of this spec solves F4 instead and never applies F3 (claim 6 a).
- **Target point** *P*_T. The target's reference point (S-0204), except that a building's target
  point is its reference point plus its art *aim shift* (Import mapping). It never includes lead.
  (S-0307 measures to a different point, its *measured point*.)
- **Shot target, firer's target.** The *shot target* is the object or cell a shot is fired at and
  that its projectile carries. The *firer's target* is the target the firer is currently attacking
  (S-0309, S-0214). They are the same except when the shot target is redirected (claim 3).
- **A projectile's firer; no firer.** The object that fired the projectile. A projectile *has no
  firer* when it was made without one, or once its detonation would carry no attacker by S-0302
  claim 1: its firer was killed or removed from the game while it flies, or started to cloak, as
  that claim says and rates it. A firer loaded into a closed transport since the shot is still the
  projectile's firer, as S-0302 claim 1 says and rates it (low). S-0302 claim 1's backing is for
  the detonation's attacker only, so what each cause does to the flight is rated here, separately.
  T1's first row rates it in its own cell. Every other rule here that reads whether a projectile
  has a firer (T1's proximity row, the note below T1, the edge-case entry *Impact with no firer*
  and the quirk `quirks.owned_shell_ground_pass`) holds at its own level for a projectile made
  without a firer, for one whose firer is in play outside any transport and has not been killed
  or started to cloak since the shot, and for one whose firer is a vehicle destroyed while it
  flies. For a firer killed but kept in play (a death sequence, a delayed kill) or removed by other
  means it holds at medium, `unmeasured`: no scenario here has such a firer. For a firer that
  started to cloak, or was loaded into a closed transport, since the shot it holds at low,
  `unmeasured`, as S-0302 claim 1 rates those two cases: no scenario here has such a firer.
- **Aim point.** The reference point of the firer's target (no aim shift), moved ahead of a moving
  vehicle by the lead of claim 3.
- **Impact point** *Q*. The aim point after scatter (claim 4); without scatter, the aim point.
- **Stored target point** *K*. The target's reference point taken when the projectile is placed
  (claim 9). It does not move with the target during the flight.
- **Arm time, past its arm time.** The *arm time* of a projectile is a number of ticks fixed at
  placement (claim 9). The projectile is *past its arm time* in a turn when the number of ticks
  from the tick of its placement to the tick of that turn is at least its arm time; with an arm
  time of 0 it is past it from its first turn. (The glossary entry *armed* is not meant here.)
- **Steering distance.** For a steering projectile with a target, at its new position *P*′ (after
  the move, before any cruise lift of claim 15) with target point *P*_T: with Δ = *P*_T − *P*′,
  trunc(*r*(Δ*x*² + Δ*y*² + *ζ*²)), where Δ has integer components, *ζ* = trunc(Δ*z* / 4) (the
  vertical difference counted at a quarter, integer division truncating toward zero) and *ζ* = 0
  for an airburst projectile. Each integer is converted to double precision before it is squared,
  and the squares and their sum are formed in double precision, as for *d*₃, so nothing wraps; *r*
  is the table square root and the result is truncated toward zero. For a steering projectile
  without a target (claim 17) the steering distance is trunc(|*V*|) of its velocity after this
  turn's pitch change (F12).
- **No target point.** A projectile whose target is gone (claim 17) has no target point. Where a
  rule below needs a point for it (the near-target test of claim 13 and the closing average of T1),
  the point is the world origin (0, 0, 0). Conversely, the steering turn tells whether a steering
  projectile has a target only by comparing its target point *P*_T (with a building's aim shift)
  with the world origin: a target whose target point is exactly (0, 0, 0) counts as no target there
  (claims 14, 15 and 17, and the T1 rows that say "no target" or "a target exists"). For a cell, an
  infantry unit, a vehicle or an aircraft this cannot happen, since the point would lie in cell
  (0, 0), which is never a legal cell (S-0009, claim 1). It can happen for a building whose art aim
  shift (Import mapping) is minus its reference point. Both directions belong to one quirk,
  `quirks.origin_as_missing_target_point` (Edge cases).
- **Object number.** A whole number that our engine gives each object when it creates it, from a
  deterministic counter of creations that it keeps itself. This spec uses it only for the phase of
  claim 13. The original numbers objects differently, which makes that phase a departure (Edge
  cases).
- **Legal cell.** As the glossary defines it: a position is *outside the legal map* when its cell
  (S-0204, coordinates divided by 256, truncating toward zero, so a coordinate between −255 and 0
  falls in cell 0) is not a legal cell. This is not the *usable area* of S-0211.
- **Velocity** *V*. Three double-precision components in leptons per tick; |*V*| = *r*(*V*ₓ² +
  *V*ᵧ² + *V*_z²), the table square root of a real number (S-0204 F1, S-0105 F1).
- **Pitch facing** *p*. A 16-bit facing value that stands for an elevation angle
  θ(*p*) = (*p* − 16383) × (−π / 32767) radians, where *p* is read as a signed 16-bit value
  (−32768 to 32767): a value of 32768 or more, as this spec writes some pitch facings, counts as
  that value minus 65536.
  So 16383 is exactly level, values from 0 to 16382 point up (0 is straight up to within one unit),
  values from 16384 to 32767 point down, and negative values lie past the vertical, pointing
  backwards. There are 32767 units per half turn. The opposite map, from an angle θ to a pitch
  facing, is trunc((θ − π/2) × *c*₂) wherever this spec turns an angle into a pitch facing (F4,
  F12, F13). Where that angle is a table angle *A* (F12 and F13), the conversion is S-0204's pitch
  from a signed table angle (claim 12 and F6 there): at one of the 50 exception indices of S-0204
  F3 the pitch comes out one unit nearer to level (16383) than the surrogate gives. F4's angle
  comes from the table arccosine (F8), not from *A*, so it takes no such correction. Since *c*₂ is very nearly −32767/π, before the truncation it gives (up to the
  roundings) 16383.5 − θ × 32767/π, half a unit above the exact inverse of θ(*p*),
  16383 − θ × 32767/π. So exactly 45° up is the pitch facing 8191.25 under
  θ(*p*), while the angle-to-facing map gives 8191.75 and truncates it to 8191: claim 6 a uses 8191,
  and claim 15 names 8192 where it says "45° up". The pitch facing written 32768 is read as −32768, an angle of about 3π/2, which
  points straight down. Table trigonometry (S-0105 F1) resolves an angle only to steps of 2π/8192,
  so every pitch facing from 16380 to 16386 gives a velocity whose vertical component is exactly 0
  and whose horizontal part is the full length: 16384, for example, flies exactly level.
- **Line-of-fire walk.** The cell-by-cell test of claim 22, from a start point *X*₁ to an end point
  *X*₂.
- **Table arcsine.** A table of 4097 single-precision values *A*ₛ[*k*] = RZ₂₄(asin(*k* / 2048 − 1)),
  *k* = 0 … 4096 (Constants). acos_t(*x*) = π/2 − *A*ₛ[trunc(2048 × (*x* + 1))].
- **Drawing fields.** Fields that only change how a projectile looks (claims 28 to 33; Import
  mapping). From the art section of its image: the *rotating flag*, the *low*, *high* and *rate*
  animation fields, the *anim-palette flag* and the voxel and flat flags. From the projectile type
  in the rules: the *shadow flag*, the *firer's-palette flag* and the *colour definition* of a
  voxel projectile.
- **General palette.** A palette read once when the program starts, from the same palette file as
  S-0111's overlay palette (Import mapping), with its components multiplied by 4 as S-0103 claim 1
  says, and given a 53-row shade table built like the anim palette's (S-0104 claim 6). It is never
  reloaded and its shade table is never tinted.

Symbols: ρ is the turn-rate field, *σ* the scatter radius (claim 4), *κ* the course lock duration,
*α* the acceleration, *n*_c the cluster count, *λ* the cluster distance, *ξ* the direction draw,
*c*ₖ the closing step, *t* the steering turn step (claim 13), *ε* the pitch step (F12), *m* the
missile turn variation, δ the signed facing difference and *ν*, *ν*′ the snap distances (F9). *r*
is only the table square root, *A* only the table angle, and *h*, *h*′ only heights above the
surface (T1). *u* is the target vehicle's speed estimate (claim 3, F6). Other letters are defined where
they are used.

## Behaviour

### Launch

The general launch rules below have the aircraft velocity exceptions of claim 7, owned by
S-0501 claim 20.

1. **Shot speed.** A projectile whose turn-rate field ρ is 0 is launched at *s*₁ = *v*(*d*, *g*),
   where *d* is the planar distance from the fire point to the target point of the shot target,
   taken with a wrapping sum of squares (quirk `quirks.shot_distance_wrap`): with Δ the integer
   difference of the two points, *n* is Δ*x*² + Δ*y*² reduced modulo 2³² into the range −2³¹ to
   2³¹ − 1 (the result of 32-bit integer arithmetic), and *d* = trunc(*r*(|*n*|)), *r* being the
   table square root of S-0204 F1 applied to the integer |*n*|. While the exact sum is below 2³¹
   (an exact planar distance below about 46341 leptons, some 181 cells) *n* is the sum itself and
   *d* is the planar distance of Terms. For a larger sum *d* is a shorter distance, and can be 0:
   for example Δ = (51200, 0), 200 cells along the map *x* axis, gives *n* = −1673527296 and
   *d* = 40908 instead of 51200. The gravity is *g*_f for a floater projectile, else *g*. The
   imported speed field plays no part. With ρ ≠ 0 (steering, or a negative ρ) the shot speed is the
   weapon's speed field. A fragment of S-0318 is not a shot: its speed is the fragment weapon's
   weapon speed *S*_w below, without this claim's shot speed or claim 2's cap (S-0318 claim 5).
   - **Weapon speed.** For a weapon whose projectile has ρ = 0 the weapon speed *S*_w of Terms is
     *v*(*R*₀, *g*), *R*₀ being the weapon range of S-0307 and *g* as above (*g*_f for a floater
     projectile), both as they stand once every rules layer has been applied. No speed value that
     any rules layer gave such a weapon has any effect. *S*_w matters only for the
     top speed of a vertical projectile (which has ρ = 0 unless the data gives it a negative turn
     rate, since a positive one makes it steering, Terms; claims 2 and 11), the speed of airburst children (claim 21) and the speed of a fragment (below; S-0318 claim 5). A steering projectile has
     ρ > 0, so its top speed is the imported field (claim 12).
   - Backing: `measurable` (scenario P1; run d for a floater projectile; the replacement through P8
     run c; P39 for the wrapping sum, on a purpose-made map more than 200 cells long with a weapon of
     unlimited range). The fragment sentence is not backed here: it rests on S-0318 claim 5, which
     has a fragment's speed `unmeasured` (an invisible fragment's speed has no effect)
   - Confidence: high; medium for the wrapped distance (the conversion of the wrapped sum to |*n*|); low
     for a fragment's speed, which rests on S-0318 claim 5 and is rated low there

2. **Launch speed limits.** The shot speed is capped at half the spatial distance to the impact
   point: the capped speed is min(*s*₁, trunc(*d*₃(*F*, *Q*) / 2)), *d*₃ (Terms) being taken from the
   fire point *F* to the impact point *Q*. The *start speed* (Terms) is this capped speed, except
   that a steering or vertical projectile whose firer type has no radial fire segments (Import
   mapping) starts at 1; with radial fire segments it keeps the capped speed. A fragment of S-0318
   is not capped (claim 1; S-0318 claim 5). Whether a fragment whose projectile type is steering or
   vertical starts at 1 is not established: it is part of a fragment's launch velocity, which
   S-0318 leaves open (Open questions). In the claims and
   formulas *s* denotes the start speed: it is the length of the first velocity (claim 7,
   F1; a start speed of 0 gives a length of 100 instead) and the speed that the launch elevation
   uses (claim 6, F4). A steering projectile's speed
   after placement is 1 in either case, because placement rescales its velocity (claim 9).
   Steering and vertical projectiles carry the weapon speed *S*_w as their top speed: the imported speed field for a steering projectile, and
   for a vertical one the imported field only when ρ ≠ 0 (a negative ρ), otherwise *v*(*R*₀, *g*)
   (claim 1).
   - Backing: `measurable` (scenario P1, run c for the cap; P8 run a and P20 variant b for the
     start speed 1 of a vertical projectile and the radial-segment exception; P20 variant c for a
     steering projectile). A fragment not being capped is not backed here: it rests on S-0318
     claim 5, which has a fragment's speed `unmeasured`
   - Confidence: high; low for a fragment not being capped, which rests on S-0318 claim 5, where
     a fragment's speed is rated low

3. **Aim point and lead.** The launch direction, elevation and scatter are computed towards the
   aim point: the reference point of the *firer's* target, without a building's aim shift. A
   fragment of S-0318 is excepted: its direction and elevation are not computed towards this aim
   point (claim 5; S-0318 claim 6), and whether its launch takes the scatter of claims 4 and 9 is
   open (S-0318 Open questions). When
   the firer's target is a vehicle whose locomotor is moving it (S-0206, Terms: *locomotor moving
   the vehicle*), the aim point leads it: with *u* the vehicle's speed estimate (below), *D*₃ the spatial distance from firer to
   target and *w* the shot speed that claim 1 would give the firer's *lead weapon* at distance
   *D*₃, the aim point is moved *ℓ* = trunc(*D*₃ / (0.9 *w*) × *u*) leptons along the vehicle's body
   facing (F6). Here *w* = *v*(*D*₃, *g*) for a lead weapon whose projectile has ρ = 0 (*g*_f for a
   floater projectile), with *D*₃ used as it is: the wrapping sum of claim 1 plays no part, and
   *w* is the weapon's speed field when ρ ≠ 0. The lead weapon is the weapon in slot 0 of the
   firer (S-0308, Terms: *weapon in slot i of an object*, which for a building includes S-0308's
   upgrade and occupant rules), or, for a firer of a multi-weapon type (S-0308, Terms), gattling
   types included, the weapon in the slot of its weapon number. A gattling type's stage does not
   change its weapon number, so a gattling firer leads by the slot of its weapon number (slot 0
   unless something else sets it) whatever slot its stage makes it fire (S-0308 claim 7). The lead
   weapon need not be the weapon that fires the shot: a firer that fires another slot (by table P
   of S-0308, a deploy weapon by S-0308 claim 8, or a gattling stage) still leads by the shot speed
   of its lead weapon. When *w* is 0 the quotient is infinite or not a number, and the conversion
   to an integer (S-0205 claim 7) makes *ℓ* = 0, so the aim point is not led (Edge cases). A firer
   whose lead-weapon slot is empty does not lead at all. Infantry and aircraft targets are not
   led, whoever the firer is. The stored target point of claim 9 is not led.
   A firer with no current target aims at world position (0, 0, 0) (quirk
   `quirks.aim_origin_without_target`). No case is known in skirmish play in which a shot leaves
   while its firer has no current target, since S-0309 claim 27 drops a destroyed target before
   the firer's firing step, so the rule matters only if some other path reaches that case.
   - **The speed estimate *u*.** *u* is an integer that the lead formula uses as leptons per tick
     without any conversion. For every vehicle target, whatever its locomotor, it is S-0207's
     speed estimate (claim 18 and F2a there, using F2) formed from that vehicle's own state at the moment the shot is
     aimed: its type speed, its country's vehicle speed factor, its speed bonus factor, the faster
     ability when it applies, and the speed fraction it holds at that moment, with the flag-carrier
     halving. In words (S-0207 F2 gives the formula in its own symbols, which differ from this
     spec's): the type speed times the country factor times the speed bonus factor, truncated; that
     integer times the faster-ability factor, truncated, only when the ability applies; that integer
     times the speed fraction, truncated; and for a flag carrier half of the result, truncated. The
     products are double precision rounded toward zero, as in S-0207. Aiming reads
     this value and changes nothing: it makes no speed-fraction update and earns no progress. It is
     not the distance the vehicle covered in the last tick, and it is not converted from progress
     units to leptons. For a vehicle on the driving locomotor S-0207 (claim 18) gives the
     consequences: a turn in place or a wait between tracks, while the locomotor still moves the
     vehicle, leads by the fraction still held (non-zero after a track, zero before the first track
     from rest or after a stop that zeroes it). For a vehicle on another locomotor the speed fraction
     and the moving test are those of the spec that owns that locomotor.
   - (a) **Redirected shot targets.** A weapon with the area-fire flag makes the shot target the
     cell the firer stands in (its centre, at the terrain height there); a spray-attack firer
     makes it the cell of S-0313 claim 9. The shot speed (claim 1), the stored target point and the
     projectile's target follow the redirected cell, but the aim point stays on the firer's own
     target, so the launch still points at that target (quirk `quirks.redirected_shot_aim`). Off:
     the aim point is the reference point of the redirected cell, so the launch direction,
     elevation, scatter and lead of claims 3 to 7 all work towards that cell.
   - (b) **A switched-off hovering vehicle.** A hovering vehicle is moving while it has a
     destination or a heading point (S-0503, Terms). The stop request of a switch-off clears its
     destination and stored path only when destination and heading point differ as full
     three-dimensional points, and otherwise leaves them (S-0503 claim 10). It keeps the heading
     point (S-0503 claims 16 and 19). It does not zero the speed fraction (S-0503 claim 4). A
     switched-off hovering vehicle that remains moving under S-0503 is led by this claim's
     ordinary moving-vehicle rule: a shot at it is led along its body facing by the amount its
     held speed fraction gives (none when that fraction is 0) (quirk
     `quirks.switched_off_hover_lead`). Whether and how far a switched-off vehicle travels, and
     when it reaches, replaces or loses its heading point, are S-0503's (claims 2, 3, 5, 7, 8, 10,
     12, 15, 16 and 19). Observable consequences of those claims for a group of switched-off
     vehicles, among others:
     - one that has a destination or a heading point when switched off is moving, so a shot at
       it is led, while it keeps either of them.
     - one that arrives at its heading point by S-0503 claim 7's test, with no destination left
       after that arrival and no stored step, loses the heading point at that arrival and is no
       longer moving (S-0503 claim 15), so from then on shots at it are aimed at its reference
       point, scattered about it by claim 4 when the projectile is arcing and inaccurate.

     Whether the targets of M-0005 that never change cell were moving in S-0503's sense is not
     recorded (Open questions), so this case is not shown to apply to them. Off: a switched-off
     vehicle is not led. The lead moves only the aim point of this claim, from which the launch
     direction, elevation and impact point are computed. The projectile keeps no led point: the
     target point it stores is *K* of claim 9, the target's reference point, so a led shot and an
     unled one at the same target store the same point, and a led shot shows its lead only in its
     launch velocity and its flight. Consequences:
     - A lead away from the firer can make an arcing shot void (claim 6 a).
     - Otherwise T1 and T2 determine its detonation and claim 20 its detonation point. The impact
       point of a led arcing shot lies about the led point, moved by the scatter of claim 4 when
       the projectile is inaccurate.
     - A led shot whose new position lies in the cell of *K* (claim 9) at a start-of-turn height
       below 208 detonates by a "reached" row of T1, which T2 puts before every other row. An
       inaccurate one is not moved onto the vehicle (claim 20 b), so whether the vehicle takes
       damage is S-0303's area damage from that point.
     - A straight or arcing projectile whose firer is still in play outside any transport and has
       not been killed or started to cloak since the shot is not detonated by the terrain (T1, its
       first row and note). One that is not flak-scattering and passes below the ground short of
       the vehicle flies on and can detonate away from it. One that is flak-scattering and has a
       target is detonated by T1's flak row where the target's reference point is not below the
       surface beneath the projectile: in the first turn in which it is below the terrain, at
       the surface there (claim 20 a), whatever its firer.

     Which of these rules applies to a given firer follows from its weapon's projectile flags in
     the imported data.
   - Backing: `measurable` (scenario P2, run c for an aircraft target, run d for *w* = 0, run e for
     a lead weapon other than the weapon fired, run f for a switching type, run g for a gattling
     type, runs h and i for the speed estimate at two speed fractions; S-0207 scenario D20 for the
     estimate itself; P17 for case a, spray attack and area fire; P44 part a for case b, including
     the stored point of a led shot, and part b for a shot that passes below the ground
     unexploded; P45 part a for the group of switched-off vehicles, which of them are led and
     where the shots at them land, and part b for the flak row's detonation of a shot with
     a firer). M-0005's per-shot aim point, the target's position at its fire
     event for every detonated projectile, is the point *K* that each projectile stores, which the
     lead does not move: the record agrees with case (b) and does not test it, since it records
     neither launch velocities nor heading points. The flag-carrier halving is `unmeasured` here as
     in S-0207 (no skirmish setup is known that makes a vehicle carry a flag). The empty lead-weapon
     slot is `unmeasured`: such a firer is not *armed* in S-0214's sense (its test is the same
     slot), so it cannot be ordered to attack, and no case is established in which it fires anyway
     (automatic targeting by a deploy-fire firer, S-0309, would be the only candidate). The fragment
     exception is not backed here: it rests on S-0318 claim 6. The aim without a target is
     `unmeasured`: under S-0309 claim 27 a target destroyed earlier in the tick is dropped before
     the firer's firing step, which then fires nothing, so scenario P27 gives no shot with an empty
     current target, and no scenario here sets up a shot that leaves while the firer holds no
     current target.
   - Confidence: medium (the lead weapon; the redirected case (a); the aim without a target, and
     that no skirmish case reaches it; the switched-off case (b), which applies the ordinary lead to
     a vehicle that S-0503 counts as moving, at S-0503's medium for its moving test and for claims
     10, 16 and 19; the unled stored point of a led shot; which vehicles of a switched-off group
     are led, so that one with neither a destination nor a heading point is not, as after the
     arrival of S-0503 claims 7 and 15; where a led shot detonates, by T1, T2
     and claim 20; the flak row's detonation below the ground, at T1's medium for that row),
     high for the speed estimate *u*; high that a shot of a firer still in play, as above, that is
     not flak-scattering passes below the ground unexploded, the case of T1's first row that P44
     part b tests; low for the fragment exception, as S-0318 claim 6 rates it

4. **Scatter of inaccurate arcing projectiles.** Only a projectile that is both arcing and
   inaccurate scatters at launch. The impact point is the aim point moved by a radius *σ* in a
   random direction (F5), with *B* the game-wide ballistic scatter (leptons):
   - not flak-scattering, or invisible: *σ* = R(trunc(*B*/2), *B*),
   - flak-scattering and not invisible: *σ* = trunc(R(0, *B*) × *D* / *R*_W), where *D* is the
     spatial distance from the fire point to the aim point (computed in single precision,
     truncated) and *R*_W the
     firer's weapon range of S-0307 (claim 13) for that weapon: the weapon's range without
     bonuses, reduced for an open-topped transport to the smallest of its own range and its
     passengers' current-weapon ranges. A multi-weapon passenger uses its weapon-number slot.
     Any other passenger uses slot 0. A passenger without a current weapon is skipped. This
     reads the current weapon regardless of the transport's target, not the passenger's weapon
     choice against that target.

   The random-state recorder observes the radius draw before the direction draw R(0, 2147483646). Only the horizontal
   position moves. Inaccurate projectiles of other kinds do not scatter at launch; flak-scattering
   invisible projectiles scatter by claim 9.
   - Backing: `measurable` (scenario P3, statistical, with the random-state recorder for the draw
     count; its variant f for the open-topped reduction of *R*_W). The multi-weapon current-slot
     and missing-passenger-weapon cases are `unmeasured`, as in S-0307 claim 13: no normal-action
     fixture establishing these passenger states is known; RG16 there remains pending.
   - Confidence: high

5. **Launch direction.** A projectile with ρ = 0 that is not dropping (straight, arcing or
   vertical) is launched in the direction (S-0204 claim 8, F3) from the fire point towards the
   impact point. A projectile with ρ ≠ 0 (steering, or a negative ρ) and a dropping projectile
   start in the firer's aiming facing (S-0310). Only a dropping projectile starts from the firer's
   own position instead of the fire point; a steering projectile starts at the fire point (raised by
   claim 8 when it is not above the terrain, at claim 8's confidence). A
   fragment of S-0318 is excepted: it starts at the burst point and is aimed at the reference point
   of its aim point in S-0318's sense (S-0318 claim 6), not at this spec's aim point. The fire
   point of claims 5 to 7 is the point before claim 8's raise (claim 8).
   - Backing: `measurable` (scenario P4; its variant g for the aiming facing of a projectile with
     ρ ≠ 0; P21 for a dropping projectile; P9 run a for the start of a steering projectile at the
     fire point). The fragment exception is not backed here: it rests on S-0318 claim 6 (S-0318
     scenario SH7 for the horizontal direction)
   - Confidence: high (the horizontal start and the direction); medium for the raise in height
     and for the unraised fire point of claims 5 to 7, as claim 8 rates them; low for the fragment
     exception, as S-0318 claim 6 rates it

6. **Launch elevation.** Every projectile except a fragment of S-0318 takes its pitch from the
   first of (a) to (c) that fits its flags (Terms, "Flags and kinds"). (a) to (c) do not apply to a
   fragment: it climbs at S-0318 claim 6's fixed elevation instead, while its
   horizontal direction is claim 5's exception for fragments (S-0318 claims 5 and 6).
   - (a) **Arcing flag.** The pitch is the solution of F4 for the start speed *s* (claim 2), the
     planar distance *d*′ to the impact point, the height difference Δ*z* = *Q*.z − *F*.z and
     gravity. When the impact point is higher than the fire point (Δ*z* > 0) the projectile is
     always launched at 45° (pitch facing 8191; quirk `quirks.uphill_arc_45`), whatever the speed
     and distance, so such a shot is never void. Otherwise the flatter solution is used, unless the
     shot is a *lob*: the weapon has the lobber flag, or the reference point of the firer's target
     is higher above the firer's position than it is far from it horizontally (planar distance
     *d*₂); a lob takes the steeper solution. The reachability test F3 plays no part in the launch:
     the shot is void (S-0301 claim 15; the projectile is removed before it enters play) exactly
     when F4 has no solution, which for Δ*z* ≤ 0 happens when the start speed is too low for the
     distance (for example with a negative turn-rate field and a small speed field, Edge cases) or
     at zero planar distance with Δ*z* = 0 (F4). With ρ = 0 the start speed comes from the distance
     to the shot target's point (claim 1), but F4 is solved for the distance to the impact point,
     which includes the lead of claim 3, so a lead away from the firer can leave F4 without a
     solution and make the shot void. In an ordinary vehicle firing step after the rearm
     countdown has run out, a void launch leaves that countdown unchanged (S-0301 claim 15,
     medium). Further attempts remain subject to the firing check: on each later answer *can fire*
     the firer attempts the shot again (S-0310 claim 9). For a projectile that does not scatter at
     launch (claim 4), while the firer, its target and the target's lead stay as they are, every
     such attempt is void and launches no projectile. An arcing and inaccurate projectile takes a
     new scatter at each attempt, its draws coming before the solution of F4 (claims 4 and 24,
     scenario P5 run d), so its impact point can differ from attempt to attempt, and an attempt
     whose scattered impact point F4 can reach launches a projectile. At zero planar distance with
     Δ*z* < 0 (the impact point exactly below the fire point) F4 has a solution: with a start speed
     above 0 a lob is launched straight up and the flatter shot straight down or straight up by the
     test of F4; with a start speed of 0 the angle comes out just short of π, so the projectile is
     launched horizontally backwards (F4; quirk `quirks.zero_distance_arc`, Edge cases).
   - (b) **No arcing flag, drawn as a voxel model.** Pitch facing 32768 (straight down, Terms) when
     the target's reference point is lower than the firer's, else 16384, one unit below level,
     which table trigonometry turns into exactly the same velocity as level (Terms). The difference
     from 16383 therefore has no observable effect, and there is no switch for it.
   - (c) **No arcing flag, not drawn as a voxel model.** The projectile is launched exactly level
     (pitch facing 16383) when the height difference Δ*z* of (a) is at most 200 leptons in size, for
     every target. When it is larger, the projectile is pitched towards an aim height *Z* over the
     planar distance *d*′ to the impact point (F13). *Z* is Δ*z* itself, except for a building
     target, where *Z* comes from the building's art height value *H*_b and the fire point's world
     height instead (quirk `quirks.building_pitch_term`). *H*_b is the building's art height field
     (S-0111, claim 7; Import mapping; an integer, default 2), counted here at 200 leptons per unit.
     S-0111 counts the same unit as a height level of 104 leptons, so this term is not the
     building's real height.
   - (d) **Steering and vertical projectiles** have no rule of their own: they take (a), (b) or
     (c) by their flags and start at the start speed of claim 2 (1, or the capped shot speed with
     radial fire segments), which is the *v* of F4 in case (a). A vertical projectile never changes direction
     in flight (claim 11), so its launch pitch is its pitch for good: it climbs only when (a), (b)
     or (c) points it up. A steering projectile starts pitched the same way, and its flight
     rules turn it (claim 15).
   - Backing: `measurable` (scenario P5, runs a to c for case (a), run f for the arcing flag on a
     steering type, run d for the void shot and run e for the height-based lob test; P44 part a for
     the repeated attempts at a led aim point, with a projectile that is not inaccurate, that create
     no projectile and leave the rearm countdown unchanged. That these attempts reach the launch and
     are void, rather than refused by the firing check, is `unmeasured`: P44's recorder does not
     read the firing check's answer, and no run here tells the two apart. The new scatter of each
     attempt of an inaccurate one is `unmeasured` as a repetition, since no run repeats a void
     attempt of an inaccurate projectile, and rests on P5 run d's draws, taken before its shot is
     found void; P37 for zero planar distance; S-0204 scenario C4 for the exception index 1024 of
     F13; P4 for cases (b) and (c), its variant f for the building term; P8 runs a and d and P9 run
     h for case (d)). The fragment exception is not backed here: it rests on S-0318 claims 5 and 6
     (S-0318 scenario SH7 for the horizontal direction; the elevation `unmeasured` there)
   - Confidence: high in case (a), except the height-based lob test, zero planar distance, the
     repeated attempts at a led aim point that create no projectile and leave the countdown
     unchanged, the reading of them as void launches rather than refusals (`unmeasured`), and the
     new scatter of each attempt of an inaccurate projectile (medium); medium in cases (b) and (c)
     and (d); low for the fragment exception, which rests on S-0318 claims 5 and 6, where a
     fragment's launch direction is rated low

7. **Initial velocity.** The velocity is built with table trigonometry (F1) from the start speed
   *s* of claim 2 (so its length is 1 for a steering or vertical projectile whose firer type has no
   radial fire segments) and the launch direction and pitch of claims 5 and 6. Its length is *s* up
   to the table square root and the roundings of F1. A firer type with *n* > 0 radial fire
   segments launches each successive shot in the direction of its body facing turned by
   (π / *n*) × *k* − π/2, *k* = 0, 1, … *n* − 1 cycling per shot, where a positive angle turns
   anticlockwise on the map, against the growth of facings (F1). So shot *k* = 0 leaves a quarter
   turn clockwise of the body facing (the body facing plus 16384 facing units), and each later *k*
   turns π / *n* further anticlockwise. A start speed of 0 gives a
   horizontal speed of 100 (F1, Edge cases). A fragment of S-0318 is excepted: its start point,
   direction and elevation are S-0318 claim 6's, and how its velocity is formed is left open
   there.
   - **Aircraft velocity exceptions.** For a shot from an aircraft, S-0501 claim 20 overrides the
     general launch velocity: with ρ = 0 the projectile takes the aircraft's body heading,
     pitch and current ground speed. With ρ = 1 it is aimed towards the target in three
     dimensions, including the height difference, at the weapon's speed, not horizontally.
     These exceptions replace the velocity left by the general launch and placement rules,
     including the steering rescaling of claim 9. Other turn rates keep the general rules.
   - Backing: `measurable` (scenario P4; P20 for radial segments; P28 for the zero speed, and
     S-0501 scenario AC12 for the aircraft exceptions). The fragment exception is not backed here:
     it rests on S-0318 claim 6, where the velocity's forming is `unmeasured`
   - Confidence: high for the construction; medium for radial segments and the aircraft
     exceptions; low for the fragment exception, as S-0318 claim 6 rates it

8. **Placement.** The projectile is placed at its *start point* and enters play (S-0203 claim 4).
   The start point is the fire point (claim 5 for dropping ones and fragments), except in height:
   when the fire point (or claim 5's start position, for those kinds) is not above the terrain
   height at its *x* and *y* (a bridge deck there is ignored), the start point is 1 lepton above
   that terrain height, with the same *x* and *y*. A position above the terrain is used as it is. So
   a projectile never starts at or below the ground, and a steering projectile fired from a muzzle
   at the surface starts at height 1 and is not detonated in its first turn by T1's
   at-or-below-the-surface row (*h* ≤ 0). The other rows of T1 apply to that turn as to any other.
   Every projectile placed this way is raised by the same rule, shots, fragments, the projectiles a
   burst releases and superweapon payloads included. The later move of an invisible projectile
   (claim 9) is not. The launch direction, pitch and velocity (claims 5 to 7) and the line-of-fire
   walk's *X*₁ (claim 22) use the fire point as it is, not raised. T1 gives the placement distance Ω
   of its passed-by row. Its first turn is in the same tick unless an earlier removal in the turn
   pass costs it that turn (State and ordering).
   - Backing: `measurable` through S-0203 scenario O1, and scenario P43 for the raised start of an
     ordinary shot from a fire point at the surface or above it. The raise for fragments, the
     projectiles a burst releases and superweapon payloads, for a fire point below the terrain and
     for a fire point under a bridge deck is `unmeasured`: no scenario here places such a start at
     or below the ground. A fragment's placement at the burst point is not backed here: it rests on
     S-0318 claim 5. That the line-of-fire walk and claims 5 to 7 keep the unraised point is
     `unmeasured`: for a fire point at the surface the two points differ by 1 lepton, which no
     recorded direction or blocked cell tells apart
   - DECIDED (autonomy, 2026-10-29): the engine applies this raise to every projectile placed
     this way, and claims 5 to 7 and the line-of-fire walk use the unraised fire point. The rule
     is meant to fit the stock outcome that Q-2111 reports, and no projectile then starts at or
     below the ground.
   - Confidence: high for the placement at the fire point in *x* and *y*; medium that the
     original raises an ordinary shot exactly so, until P43 is run; medium for the raise in the
     other cases above; medium for the unraised point of claims 5 to 7 and the walk; medium for
     a fragment's placement, which rests on S-0318 claim 5

9. **Stored target point, arming and invisible projectiles.** At placement the projectile stores
   *K*, the target's reference point at that moment (quirk `quirks.stored_target_point`), and an
   arm time: the projectile type's arm field in ticks, or 0 when the target is an aircraft. An
   invisible projectile is moved at once to the point that the table below gives, and its velocity
   becomes 0. When it is both invisible and flak-scattering, *K* is a scattered point: the
   target's reference point moved by a radius trunc(R(0, 2*B*) × *D* / *R*₀) in a random direction
   (the random-state recorder observes the radius draw before the direction draw, F5), where *D* is the spatial distance from the fire point to
   the unscattered point and *R*₀ the weapon range. The table and every later rule use this
   scattered *K*:

   | Situation | Placed at |
   |---|---|
   | a firestorm wall stands on the line from the fire point to *K* (a wall of the firer's own house excepted); the wall met first when walking from the fire point towards *K* counts | the point of that wall, at terrain height; the line-of-fire walk plays no part |
   | otherwise, the line-of-fire walk from the fire point to *K* finds a blocking cell (the wall-damaging warhead exception of S-0307 claim 11 does not apply here) | the centre of the first blocking cell, at the terrain height of that centre |
   | otherwise | *K* |

   A steering projectile's velocity is scaled to length 1 (F10), before any aircraft velocity
   replacement of claim 7.
   - Backing: `measurable` (scenario P6; P3 run d for the flak scatter; P22 for the arm time). The
     firestorm point is `unmeasured`: no event of the original turns a house's firestorm on
     (S-0211, Terms), so it cannot be observed.
   - Confidence: high; medium for the firestorm point

### Flight

10. **Straight and arcing flight.** In each of its turns the vertical velocity *V*_z of a straight
    or arcing projectile falls by *g* (or *g*_f), and the projectile moves to trunc(*P* + *V*)
    componentwise with the reduced velocity (double precision, truncated toward zero), so even its
    first step already falls by *g* (or *g*_f). Straight projectiles are therefore also pulled down by
    gravity; their launch elevation does not compensate for it.
    - Backing: `measurable` (scenario P1, flight time and the height fall of a straight projectile,
      run d for the floater gravity)
    - Confidence: high

11. **Vertical flight.** A vertical projectile has no gravity and never turns: the direction of
    its velocity stays the one it was launched with (claims 5 to 7), so it flies a straight line,
    level or tilted as launched, and only its speed changes. While trunc(|*V*|) is below its top
    speed *S*_w (claim 2) its velocity is rescaled to length trunc(|*V*|) + *α* (F10: the
    projectile type's acceleration *α* is added each turn, and the sum is not capped at *S*_w, so
    the speed can end above it; quirk `quirks.vertical_speed_uncapped`). It then moves to
    *P* + trunc(*V*) (the rescaled *V*). It detonates when its world height exceeds the
    projectile type's detonation altitude (an absolute world height, default 0), when its height
    above the surface at the start of the turn is negative, or when it crosses a bridge deck
    height (T1); in every one of these cases the outcome of T2 is its new position with no change
    of height (unlike a straight or arcing projectile, it is not moved to the deck height and not
    dropped 20 leptons), and claim 20 then applies as for every projectile. It gains height only when launched upwards (claim 6). One fired level
    from below its detonation altitude therefore never reaches it: it detonates by another row of
    T1 (for example the proximity row) or is removed when it leaves the legal map.
    - Backing: `measurable` (scenario P8; run e for the detonation by a negative height; P33
      runs c and e for the deck crossings)
    - Confidence: medium

12. **Steering speed.** A steering projectile's speed |*V*| changes each turn towards its top speed
    *S* (the weapon's imported speed field, since ρ > 0), by F10: it grows by the acceleration *α*
    (capped at *S*) or falls by trunc(*α*/2) (not below 0); at exactly *S* it keeps its speed.
    During *course lock* the turn step *t* of claim 13 is 0. The heading does not change (claim 14).
    With a target the pitch of a projectile that is not level still turns, in direct mode (claim 15:
    *t* = 0), by up to trunc((0 + 256)/2) = 128 facing units per turn; a level projectile keeps its
    pitch (claim 15), and without a target the pitch does not change (claim 15, step *t* = 0).
    When the course lock duration *κ* is 0, the acceleration during the lock is also replaced by 1 in
    ticks with an even tick number and 0 in odd ones (with *κ* ≠ 0 the normal acceleration applies).
    Course lock starts at placement and ends:
    - with *κ* ≥ 1: the lock covers the first *κ* − 1 turns (counted from 1 at the first turn), so
      the *κ*-th turn is the first unlocked one, and a *κ* of 1 gives no locked turn,
    - with *κ* = 0: in the first turn in which *S* ≥ 40 or *S* ≤ |*V*| + 0.5 (|*V*| measured before
      this turn's speed change, as a real number; the margin 0.5 is in leptons per tick); that
      turn is already unlocked, with the normal acceleration and turn step,
    - with *κ* < 0: never (Edge cases).
    - Backing: `measurable` (scenario P9, runs a, b and d; run p for a negative course lock duration)
    - Confidence: high

13. **Steering turn rate.** The turn step of a steering projectile in a turn is
    *t* = 256 × (*q* mod 256) facing units, where *q* = trunc((1 + *m* + *m* sin_t(2π *j* / 15)) × ρ),
    *m* is the game-wide missile turn variation, and *j* = (the projectile's object number (Terms)
    + the tick number) mod 15. Our engine's object numbers differ from the original's, so the phase
    of this 15-tick oscillation may differ from the original's while its period, amplitude and
    formula are exact (departure, Edge cases). When the spatial distance *d*₃ from the projectile's position at the start
    of the turn to its target point *P*_T is less than 256 leptons, the step is computed from
    *q*\* = trunc(1.5 *q*) instead of *q* (F7). This is the target point of claim 14 (the target where it stands in this turn, with the
    aim shift of a building), not *K*; with no target the point is the world origin (Terms), so a
    projectile without a target is "near" only when it is within 256 leptons of the origin (quirk
    `quirks.origin_as_missing_target_point`). The
    step is 0 during course lock. Only the residue of *q* (or *q*\* near the target) mod 256, from
    0 to 255, counts, and *t* is a signed 16-bit facing step, so a value of 256 or more wraps and a
    residue from 128 to 255 gives a negative step: the projectile then snaps to its goal facing when it is within |*t*| of it, and
    otherwise turns *away* from the goal by |*t*| (quirk `quirks.missile_turn_wrap`).
    - Backing: `measurable` (scenario P9; run e for the wrap and run f for the near-target factor,
      P31 variant b for the origin as the point of a projectile without a target).
      The phase *j* is read from the recording as the position of the maximum of the 15-tick
      oscillation of the heading change (P9), so the formula is measurable whatever the object
      number is.
    - Confidence: high for the formula and the signed step

14. **Steering: horizontal.** For a steering projectile with a target point *P*_T (the target where
    it stands in this turn, a building's with its aim shift, no lead), the displacement of a turn is
    trunc(*V*) of the velocity after this turn's speed change (claim 12) and before this turn's
    change of heading, so the turn ends at *P*′ = *P* + trunc(*V*). In the same turn the horizontal
    direction of that velocity is turned towards the direction (S-0204 F3) of *P*_T as seen from
    *P*′, by at most *t* (snapping when the signed facing difference is within *t*), keeping its
    horizontal speed (F11), and the pitch change of claim 15 applies to the result (F12). So a
    change of heading first shows in the next turn's displacement. With no target (claim 17) the
    horizontal direction is kept, and the displacement of the turn already includes this turn's
    pitch change of claim 15: *P*′ = *P* + trunc(*V*), with *V* after F12.
    - Backing: `measurable` (scenario P9)
    - Confidence: high with a target; medium without (scenario P9 run g)

15. **Steering: vertical.** In the same turn the pitch turns towards a goal pitch, keeping |*V*|
    (F12 gives the turn and the rebuild of *V* from the new pitch):
    - *direct mode*, when the target is an aircraft, or the planar distance *d*_P from *P*′ to the
      target point is at most 3 cells (6 for a very high projectile) and the projectile is not
      airburst, or *t* is 0 or 256 (that is, the residue mod 256 of claim 13 is 0 or 1; a negative step from
      the wrap of claim 13 does not give direct mode): goal = pitch facing of the vector to the
      target point, step trunc((*t* + 256)/2) (16-bit arithmetic; so during course lock, where
      *t* = 0, a projectile with a target is in direct mode),
    - otherwise *cruise mode*: the projectile holds a cruising altitude, the world height *E* +
      *n* × 104 leptons, where *E* is the terrain height of the cell at *P*′ + 6 × trunc(*V*) (plus
      416 when that cell holds a high bridge), *n* = 10 for an airburst or very high projectile and
      min(trunc(*D* / 256), 5) otherwise, *D* being the *spatial* distance *d*₃ from *P*′ to the
      target point (so *n* is 5 only from 5 cells away, spatially). With *e* = the world height of
      *P*′ minus the cruising altitude: it is lifted 18 leptons when *e* < −20 and lowered 18 when
      *e* > 20 (a direct change of position; the steering distance of Terms, the distance *D* and
      the cell ahead are measured at *P*′ before this lift; everything after the steering turn uses
      the lifted position as the projectile's new position *P*′: every row of T1, the closing step
      *c*ₖ, the bridge-deck test and the detonation point of T2, and the position the projectile
      keeps when no row applies),
      and the goal pitch is 8192 (45° up) when *e* < −52, 18432 (about 11° down) when *e* > 52,
      else 16384, with step trunc(*t*/2) of the signed 16-bit *t*,
    - a level projectile with a target keeps its pitch and gets no cruise lift; its velocity is
      nevertheless the F12 rebuild for that unchanged pitch, which differs from the old velocity
      only by rounding,
    - with no target (any kind of steering projectile, level ones included) the goal pitch is 8192
      with step *t*, so it climbs (not during course lock, where *t* = 0).

    The rebuild keeps the length of the velocity whose pitch it turns: with a target that is the
    velocity after this turn's heading change (F11), without one the velocity after this turn's
    speed change (F10) (F12, its *M*). The heading change keeps the horizontal speed, so this
    differs from the length before the heading change only by rounding, except when the speed
    change has left the velocity at zero length (for example with a top speed of 0, Edge cases):
    F11 then gives it a horizontal length of 100, and the rebuilt velocity has a length of about
    100 and a vertical component of about 100 × sin_t(θ(*p*′)), so the projectile moves again in
    the next turn.
    - Backing: `measurable` (scenario P9, recorder of projectile positions; run q for the rows
      that see the lifted position; S-0204 scenario C4 for the pitch conversion of F12 at the
      exception index 1024; run d for the pitch
      during course lock; runs k to o for the very high, airburst, level, aircraft-target and
      high-bridge cases; run j for the cruise mode of a negative step; run i for the length
      kept by the rebuild)
    - Confidence: medium

16. **Trailer animations.** A projectile type with a trailer animation creates one at the position
    the projectile holds at the start of the turn, in every turn whose tick number is a multiple of
    the type's effective spawn delay (the tick number of the game, not the projectile's age). These
    are animations of S-0109 and take whatever draws their creation takes. Each is created with an
    initial loop delay of 1 update, repeat factor 1 and no reverse request. S-0109 claims 11 and 29
    govern its first-update timing: it is eligible later in the same turn pass, but counting removals
    can cost it that update. Its first actual update neither advances its stage nor counts down the
    initial loop delay; its second actual update ends that delay without advancing the stage.
    The effective spawn delay is the type's spawn delay field, except for a type with the scalable flag after a vehicle has
    fired it: each time a vehicle fires a projectile of a scalable type, that type's effective delay
    becomes spawn delay + trunc(max(*N* − 5, 0) / 3), *N* being the number of projectiles of
    scalable types fired by vehicles that are in play, the new one included, except that a value
    of 0 there means the spawn delay field itself (which matters only for a negative field, for
    example a field of −1 with *N* from 8 to 10). The value belongs to the type, is shared by all
    its projectiles and stays until the next such shot; removals do not lower it. The effective
    spawn delay is therefore 0 only when the spawn delay field is 0 and the raised value, if any,
    is 0 too (Edge cases).
    - **Where the two fields are read.** The trailer animation and the spawn delay field are read
      from the art file, in the section named by the image value that the projectile's rules
      section gives in the same layer, before any arctic change to the name used for drawing (so a
      snow map never selects a suffixed section for them). S-0007 claim 33 and S-0004 claim 14
      give this selection and its gate: when the projectile's rules section in a layer gives no image value, or an empty
      one, neither field is read in that layer, and both keep their earlier values (initially no
      trailer and a spawn delay of 3); there is no fallback to the section named by the type's own
      name. Keys of the same names in the projectile's rules section have no effect. In the selected
      art section, a missing or empty value keeps the earlier field; a reserved no-type name clears
      the trailer reference (S-0004). The zero-delay departure and importer warning in Edge cases
      concern this art-supplied delay, including a delay retained from an earlier layer, not any
      ignored value in rules.
    - Backing: `measurable` (scenario P10; its variant c for the position of the animation, its
      variant b for the raised delay, its variant d for a delay that removals do not lower and
      that a later single vehicle shot resets, its variant e for a raised value of 0 and its
      variant f for the file and section the two fields are read from; S-0004 scenarios K1 to K4
      for the gate and the arctic case; S-0007 A24 for retained settings; P10 variant g for the
      zero-delay departure and its art source; S-0109 scenario AN26 for the initial loop delay,
      repeat factor, reverse request and first-update timing)
    - Confidence: high for the unscaled delay; medium for the scaled delay (the exact count *N*
      and when entries leave it)

17. **Losing the target.** When the target object leaves the game, or becomes untargetable (for
    example by being teleported), the projectile's target becomes the cell the target stood in if
    the target was not airborne, else nothing. Being teleported here is the teleporting
    locomotor's own jump (S-0207 claim 20 e). The departure of a superweapon warp member is not such
    a loss (S-0513 claim 39): a projectile in flight towards the member keeps it as its target
    object, and its target does not become the cell the member stood in. A steering
    projectile without target (Terms, *No target point*, which includes a target whose target point
    is the world origin) climbs (claim 15) and detonates once its height above the surface reaches
    the game-wide missile safety altitude.
    A projectile whose firer leaves the game keeps flying and detonates normally (S-0203 claim 6 d).

    For a projectile whose target object is a departed warp member, every rule of this spec that
    reads the target's position reads the position the member had at departure until it appears at
    its landing point (S-0513 claim 39). So a steering
    projectile steers towards the member's old position, where the steering-distance row of T1 and
    claim 20 place a detonation, and from its first turn after the member appears it turns towards
    the landing point. A straight or arcing projectile retains its stored target point *K* (claim
    9), as for any target, and its flight and detonation remain governed by claim 10 and T1. While
    the member is off the map it is in no cell, so neither the proximity row of T1 nor the area
    damage of a detonation (S-0303) finds it: the detonation's ordinary area damage does not reach
    the member while it is off the map. Once the member has appeared it is found in its landing cell
    like any object, and its phasing in does not stop a hit (no row of S-0302 claim 8 is for it). A
    member that its landing check kills (S-0513 claim 37) leaves the game before it appears, so the
    projectile's target becomes the cell the member stood in at departure, or nothing if it counts
    as airborne by the next sentence. Where the rules of the projectile's flight and detonation here
    (T1, T2, claim 20 and the loss above) ask whether the target is airborne, a member between
    departure and appearance does not fall under S-0307's rule that an object not on the map is
    neither airborne nor floor-bound. It is counted by its height above the surface beneath its kept
    position, and as standing on a deck only while a landing check that passed for a landing cell
    on a deck awaits its placement. So a member that departed from a high-bridge deck counts as
    airborne from departure until it appears, except while such a passed check awaits placement
    (`quirks.warp_deck_departure_airborne`). In our engine a placement can be refused after a
    passed check (S-0513 claim 50, engine design). With the quirk on (the default), the member
    then counts by its height again until a later check passes for a landing cell on a deck. A
    later check that fails, that redirects it, or that passes for a landing cell not on a deck
    leaves it counted by its height.
    Each passed check sets the deck count from its own landing cell alone, whatever an earlier
    check gave. With the quirk off, the member counts as standing on the deck it departed from
    until it appears, through any refusal. This refusal rule is our engine's design (Q-2232), not a
    reproduction claim, and has no quirk switch of its own.

    *Special hits on a warp member.* A detonation whose warhead selects one of the rows of S-0514
    claim 20 listed here acts on the projectile's target object, not through area damage, so the
    off-map rule above does not stop it. When that target object is a departed warp member, off
    the map or phasing in after it appears, the selected row acts as follows:
    - takeover (S-0601 claim 5): the member changes owner at the detonation, by that claim's
      tests,
    - bomb planting (S-0519 claim 2) and bomb disarming (S-0519 claim 7): the member gets or
      loses its bomb at the detonation, by those claims' tests,
    - air-strike designation (S-0519 claim 16): the member is designated, by that claim's tests,
    - disguise-making (S-0516 claim 31): the firer copies the member as that claim gives,
    - electric charging (S-0518 claim 11): nothing happens, since it acts on buildings only, and
      the member's health is unchanged,
    - temporal erasure (S-0514 claims 1 to 4): the lock attaches, by those claims' tests. Its
      ground-layer test (S-0514 claim 1) passes, by the next paragraph. A lock made while the member
      phases in, or after, has S-0513 claim 39a's effects (*A lock after the departure*): the member
      is being erased again, with that state's protection, skipped turns and drawing, and is
      removed at the update S-0514 claims 7 and 8 give. What a lock made while the member is off
      the map does to its warp is not established (Open questions),
    - locomotor replacement: nothing happens to the member from its departure until its arrival
      hold ends. This is our engine's design, not a reproduction claim, and has no quirk switch:
      in the original what the seizure then does
      to its warp is not established (Open questions).

    In every case listed here the selected row still replaces ordinary damage (S-0514 claim 20).
    Parasite attachment is S-0614's (claim 21 here), and this paragraph does not cover it. What
    the direct-rocking and nuke-making rows of S-0514 claim 20 do to a departed member is not
    established (Open questions).

    *Render layer of a warp member.* From its warp's launch until its own movement applies again
    after its arrival hold, a warp member is in the ground render layer, whatever its height
    (S-0102 claims 3 and 4). Unless it moves from one bridge deck to another, it then stands at
    no height above its surface (on the deck when it lands on a high-bridge cell from the
    ground), so its own rows of S-0102's layer table give the ground layer as well. It leaves
    that layer only when its own movement raises it, at that movement's moment of S-0102 claim
    4, except an aircraft whose type cannot land, which keeps the ground layer while it flies,
    until a crash. For a member moving from one bridge deck to another, its height at the
    landing point is not established (S-0513 claim 36), and so, for a unit with jumpjet movement
    or an aircraft whose type can land, neither is when it leaves the ground layer (S-0102 claim
    3 and Open questions). So claim 20 (c) uses the threshold 42 for a warp
    member, also for one that departed from a high-bridge deck and counts as airborne by the
    paragraph above. No member uses 128 there:
    an aircraft is never warped while in the air (S-0513 claim 34).
    - Backing: `measurable` (scenario P11: run a for the airborne target, run c for the target on
      the ground, run d for a target that teleports, run e for a firer that leaves the game, run f
      for a projectile that keeps a warp member and turns towards its landing point; P38 for a
      building target whose target point is the world origin). The other warp parts are
      `unmeasured`: P11 run f judges post-appearance flight and damage, and it provides no
      distinguishing expectation for flight while the member is off the map. No part here
      detonates a projectile of ordinary damage near a member then, places an ordinary hit before
      or after the end of a member's phasing in, has the landing check kill a member that a
      projectile targets, or tests whether a member that departed from a deck counts as airborne
      (run j's threshold 42 holds either way). That S-0307's off-the-map rule does not apply is
      `unmeasured` as well.
      The special hits are `measurable` by run g (off the map) and run h (phasing in). The
      effects of an erasure lock made while the member phases in, or after, are S-0513 claim
      39a's: `unmeasured` for a lock while it phases in, by that claim's stated reason, and
      `measurable` by S-0513 SW12 part l5 for a lock after the phasing-in hold, within the limits
      that claim gives for part l5, with the exact removal tick `unmeasured`. The
      refusal rule of the deck count is
      `measurable` in our engine by run i; the original has no refused placement to compare. The
      rule of no seizing is `measurable` in our engine by run h part h run there; run in the
      original, that part only records the original's outcome. The threshold 42 for a member
      that counts as airborne is `measurable` by run j. The ground render layer until the
      member's own movement applies again after its hold, and its leaving that layer at that
      movement's moment, are `measurable` (S-0102 D49 (b)). They are `unmeasured` for a landing
      on a high-bridge deck, from the ground or from another deck, since D49 runs on a flat map,
      and for an aircraft member, since no scenario there warps one.
    - Confidence: high; medium for the target point at the world origin; medium for the warp
      member kept as target, the turn towards its landing point and the hit there (run f), and
      medium, unmeasured, for the position read while the member is off the map, for the lack of
      area damage to it then, for the hit while it phases in, for the loss when its landing check
      kills it, for the deck rule and for the airborne count of a departed member. Medium for each
      special-hit row on a departed member (runs g and h), medium for an erasure lock's effects
      while the member phases in or after (S-0513 claim 39a), medium for the threshold 42 (run j),
      medium for the ground render layer until the member's own movement applies again after its
      hold and its leaving that layer then (S-0102 claims 3 and 4), low for a member that moves
      from one bridge deck to another, and high
      for the two engine-design rules (the deck count after a refused placement, and no seizing
      of a member from its departure until its arrival hold ends) as design contracts

18. **Degeneration.** A projectile type with the degenerating flag loses 1 point of its shot damage in
    each turn in which it does not detonate, but not below 5.
    - Backing: `measurable` (scenario P12)
    - Confidence: high

### Detonation

19. **When a projectile detonates.** In its turn a projectile detonates when any row of table T1
    (State and ordering) applies to its kind. Some rows test its new position and others its height
    above the surface at the start of the turn (T1 says which). T2 gives, for the rows that apply in
    a turn, one or several, the detonation point or whether the projectile is removed instead. A
    projectile of a kind other than steering whose new position is outside the legal map (Terms) is removed
    without detonating, unless a "reached" row or the proximity row of T1 applies in the same turn
    (T2). Detonation and removal happen in that same turn (S-0203 claim 5).
    When a latching weapon's carrying projectile is removed without detonation, S-0614 claim 21
    owns the consequence: its parasite remains stranded off the map, unattached and unable to
    act, rather than attempting a failed-latch return. That claim also owns
    `quirks.parasite_carrier_loss_strands_unit` and its alternative.
    - Backing: `measurable` (scenarios P6, P7, P9, P11, P13, P18; P22 to P25 and P29 to P34 for
      single rows, P36 for the speed row; P26 for removal and T2; S-0614 scenario P11 for
      carrier loss, with that spec's unmeasured qualification for other removal causes, and
      P32 runs f, g, i and j for the open-gate control, the projectile with a firer, the inactive
      laser-fence section exception and the vehicle-like-building exception of the first row; P44
      parts b and c for the first row with and without a firer). The rule that the rows of this spec
      never remove a steering projectile for leaving the map is `unmeasured`: what then happens to
      such a projectile is not established (Open questions), so no scenario could predict an outcome
      that would confirm or falsify it.
    - Confidence: high for the rows marked high in T1 and for the high part of its first row,
      medium and low for the rows and parts that T1 marks so (the proximity row per cause of a
      lost firer as Terms rates it), and medium for T2

20. **Where it detonates.** The detonation point is the projectile's position after the moves of T1
    and T2, changed by the three rules below, in the order listed. Each rule works on the point that
    the rules before it leave, so the snap distance of (b) is measured from the raised point and the
    distances of (c) from the point that (a) and (b) leave:
    - (a) a projectile whose height above the surface is negative is raised to height 0,
    - (b) a projectile that is neither inaccurate nor airburst and has a target (an object or a
      cell) is placed on the target's reference point, when it detonates in a turn in which the
      arrival row of T1 applies (always), or in a turn in which one of the two "reached" rows of T1
      applies and the snap distance *ν*′ of F9 is at most max(2|*V*|, 128) (|*V*| the speed at the
      start of the turn),
    - (c) for a projectile that is neither inaccurate nor airburst and has a target. Two distances
      are measured from the point that (a) and (b) leave: *D*_R, the spatial distance *d*₃ to the
      target's reference point, and *D*_S, the spatial distance of S-0204 (claim 6) to the target,
      which is *D*_R reduced, for a building target, by 64 leptons for each cell of the sum of the
      building's foundation width and height (not below 0). The point becomes:

      | Condition | Point becomes |
      |---|---|
      | the warhead lacks the electromagnetic flag (S-0303 F5), and *D*_S is under the target's threshold: 42 for a target that is not airborne or is airborne in the ground render layer, 128 for a target airborne in another render layer | the target's target point |
      | the condition above does not hold, and *D*_R < 32 | the target's reference point |
      | neither | unchanged |

      All comparisons are strict. Inaccurate and airburst projectiles are not snapped by (b) or
      (c); the raise of (a) still applies.

      Note on (c): the point that (a) and (b) leave is also the burst point of S-0318 (Terms), so
      (c) moves the detonation point of the projectile's own area damage and explosion, not the
      point from which its fragments burst (S-0318 claim 1).
    - Backing: `measurable` (scenario P13; P24 for the raise of (a)). The note on (c) is
      not backed here, since neither P13 nor P24 tells the burst point from the detonation point:
      it rests on S-0318 claim 1 and S-0318 scenario SH1 part r
    - Confidence: medium, the note on (c) included until S-0318 scenario SH1 part r runs

21. **Several detonations.** Ordinary detonation damage is S-0303's area damage at the detonation
    point. A projectile whose warhead has the parasite flag is an exception (S-0303 claim 2):
    its detonation bypasses area damage, whether attachment succeeds or fails. Its later
    explosion animation and debris remain unchanged. This exception applies only to projectile
    detonations, not to non-projectile producers using the same warhead. For a latching weapon,
    S-0614 claim 1 owns delivery: it attempts to latch onto the projectile's target object
    regardless of the detonation point's distance from that object; a cell target or a target
    that no longer exists gives a failed latch.
    What each ordinary area-damage detonation passes to S-0303 (Terms there; glossary
    *detonation*) is:
    the projectile's damage amount at that moment (the shot's damage, less the points that claim 18
    took in earlier turns; the same amount for every cluster detonation), the warhead of the shot's
    weapon, and the attacker and attacking house that S-0302 (claim 1) gives a projectile's
    detonation: the firer and its house unless, between the shot and detonation, the firer was
    killed, removed from the game by any other means or, believed, started to cloak; neither is
    carried in those cases. Loading the firer into a closed transport is believed not to end
    this attribution. These are S-0302 claim 1's rules and confidence qualifications, also used by
    S-0303 claim 14; claim 17 here concerns continued flight, not attribution. The nuclear wait
    of claim 25 does not preserve attribution against any of those intervening events. The children of an
    airburst (below) are projectiles of their own: each passes the damage amount and warhead of the
    airburst weapon and takes the firer of the projectile that launched it, so the same
    attacker rule applies at their detonation.
    A projectile that is not airburst detonates *n*_c times, *n*_c being its cluster count: the first
    at the point of claim 20, each later one at a nearby point of the point before it. For every detonation, including the last, the random-state recorder observes a ranged draw R(256, 512), whose result is the distance *λ*, followed by one raw value of the scenario random stream; that raw value is used for the nearby point N(*Q*, *λ*, snap off) of S-0312 F2, where *Q* is the point just used. So a
    cluster count of 1 still takes the two draws once, and the points form a chain: each is 256 to
    512 leptons from the one before it, horizontally and up to the rounding of F2, except that a
    point whose cell would lie outside 0 to 511 in either coordinate is replaced by *Q* itself
    (F2). (Were a detonation ever to remove the projectile, its remaining detonations and the draws
    after that detonation would not happen. Nothing that this spec or S-0303 describes removes it during its own
    detonation, so in every case described here they all happen; Open questions.) A cluster
    count of 0 or less gives no detonation and no draw at all (Edge cases). An
    airburst projectile detonates once and launches nine projectiles of its airburst weapon: at the
    eight neighbouring cells and at its own cell, falling straight down at a tenth of that
    weapon's speed *S*_w (integer division, truncated, with *S*_w as claim 1 gives it). Each child
    takes one draw R(0, 32) when it is made; that draw has no other observable consequence, because
    the children fall vertically. A projectile with a shrapnel weapon also launches fragments
    (S-0318).
    - Backing: `measurable` (scenario P14 for the cluster count, the chain, the nine children and
      the draw of each; P14 variant d, with the random-state recorder, for the ranged draw and raw
      value after every detonation, the last included, for a cluster count of 1 and of 3, and for
      no draw at a cluster count of 0; S-0302's scenarios D13, D17 and D24, which own the rule, for the attacker and attacking house
      that each detonation carries; S-0614 scenarios P1 and P3 for the parasite-projectile
      exception; S-0303 scenario A14 variant o for its later explosion animation and debris),
      and `unmeasured` for the case of a projectile that leaves play during its own detonation (no such
      case is established) and for loss of attribution through a firer's removal by a means other
      than killing or starting to cloak (S-0302 claim 1 identifies no reproducible setup).
      The cloak and closed-transport cases use S-0302 scenario D24 parts b5 and b4 respectively.
    - Confidence: high for the count, the draw after each detonation and the absence of area
      damage for a parasite-warhead projectile; medium for the children, distance-independent
      latching and killed-firer attribution; low for the cloak and closed-transport attribution,
      as in S-0302 claim 1

### Eligibility and line of fire

22. **Line-of-fire walk.** For a projectile subject to cliffs or walls, the walk from *X*₁ to *X*₂
    samples *n* = max(|Δ*c*ₓ|, |Δ*c*ᵧ|) points *X*₁ + *i* × trunc((*X*₂ − *X*₁) / *n*), *i* = 0 … *n* − 1
    (each component divided separately, truncating toward zero), where Δ*c* is the difference of
    the cells of *X*₁ and *X*₂. With *n* = 0 (both in one cell) no point is sampled and nothing
    blocks. The *previous sample* of the sample *i* is the cell of the sample *i* − 1, and the
    cell of *X*₁ for *i* = 0. A sampled point is blocked:
    - by a *cliff* when the projectile is subject to cliffs and the elevation level (S-0307 Terms:
      height level plus 4 for a cell marked as carrying a bridge deck) of its cell is at least 4
      above that of the previous sample's cell and strictly above that of the cell of *X*₁,
    - by a *wall* when the projectile is subject to walls, its cell holds an overlay whose type has
      the wall property, it is not the cell of *X*₂, the height level of the cell of *X*₁ is not
      above that of the cell of *X*₂ (plain height levels here), and the wall does not belong to a
      house allied with the firer's house while allied wall transparency is on.

    The first blocked sample's cell is reported. The walk is used by the range test (S-0307 claim 11
    and its scenarios), by invisible projectiles (claim 9) and in flight (T1), where the test is made
    for the single sample at the new position *P*′, with *X*₁ the fire point, *X*₂ the stored
    target point and the "previous sample" the cell the projectile was in at the end of its previous
    turn.
    - Backing: `measurable` (S-0307 scenario RG8; scenario P15 for the walk; P25 for the test in
      flight; P35 for allied wall transparency, its variant c for the height condition and variant
      d for the end cell)
    - Confidence: high

23. **Anti-air, anti-ground and targeting modes in the firing check.** The firing check answers
    *illegal* when one of these rows holds, except against the firer's own gripped unit (see
    below):
    - the target is airborne and the projectile is not anti-air,
    - the target is an infantry or vehicle whose render layer is not the ground layer (flying,
      jumping, falling from a transport, or digging underground) and the projectile is not anti-air,
    - the target is not an infantry, vehicle, aircraft or building (a cell, overlay or terrain
      object), it is not airborne and the projectile is not anti-ground,
    - the target is an infantry, vehicle, aircraft or building that is not airborne, is not on a
      bridge and stands on a cell whose land type is water or beach, and the firer's naval-targeting
      mode gives "none" for it (S-0308, table N: naval-targeting mode 0 against an
      underwater target that is cloaking, cloaked or uncloaking, mode 2 against a target whose
      type is not underwater, and mode 6 against every target),
    - the firer's type has land-targeting mode 1 and the target is one of these:
      - an infantry, vehicle, aircraft or building that is floor-bound (S-0307, Terms) and that is
        on a bridge or stands on a cell whose land type is neither water nor beach,
      - a cell whose land type is neither water nor beach.

    **The firer's gripped unit.** No row of this claim refuses the firer's own gripped unit
    (S-0607 Terms): for it every row yields, whatever the weapon, to the *rearming* answer of
    S-0301 claim 9 (rank 4) and S-0607 claim 11, which comes ahead of every test of the target's
    layer, height, kind or armour, subject to the precedence and exceptions that S-0301 claim 9
    gives it. Every other airborne object asked about with a projectile that is not anti-air is
    refused by the first row (an infantry or vehicle among them also by the second),
    including one the firer already holds as its target: the answer is *illegal* unless an
    answer that S-0301 claim 9 ranks above it comes first. So a firer that took a target while it
    stood on the ground and still holds it when it lifts off is refused, and its attack drops
    that target no later than the first check that answers *illegal* (as S-0304 claim 2 gives for
    a target that its claim 1 makes illegal; S-0215 claim 32 for buildings). For a building the
    drop is rated as S-0215 claim 32 rates it, and for a unit low (`unmeasured`). Whether another
    rule can drop such a target earlier is an open question of S-0309.

    The land-targeting mode is an integer of the firer's type: 0 allows land targets, 1 allows
    no land targets on the cells and objects of the last row, and 2 allows them and makes the
    weapon choice prefer slot 1 against them (S-0308, table P, for mode 2); any other value
    acts as 0 here. The water and land rows depend on the
    firer's type, not on its weapon. An overlay or terrain object target meets neither of them.
    Neither row depends on distance.

    **Rank of the water and land rows.** Both rows give their *illegal* at rank 4 of S-0301 claim
    9, as the first three rows do. They therefore outrank *rearming* (rank 5), *out of
    ammunition* (rank 7), *cloaked* (rank 8), the late target tests (rank 9) and *out of range*
    (rank 10): a firer that is rearming, out of ammunition or cloaked, or that is asked with the
    range test about a target beyond its range, is answered *illegal* against a target that one
    of these rows refuses.

    Within rank 4 the two rows come after these answers, which win when they hold as well: the
    *impossible* answers for a hidden target, an absent weapon, a power-sensitive weapon, the
    other weapon's live effect and a spawner weapon whose firer is under the parasite hold; the
    *rearming* answers for a materialising firer, the firer's own gripped unit, a held temporal
    target and a spawner weapon none of whose spawned aircraft is alive; for a vehicle, the
    tube-transit *impossible* and launching-spawn *busy* answers that S-0301 claim 9 already puts
    earlier; and for a building, the *impossible* of a building without power. A building's pending
    delayed shot (S-0301 claim 14) gives *rearming* ahead of the two rows and of every other row
    of this claim; S-0301 claim 9 gives its rank. The magnetic-beam *rearming* of a firer whose
    grip and live wave coexist comes after the two rows, as S-0301 claim 9 gives for the
    target-layer and projectile tests. The order among the rank-4 *illegal* answers shows in no
    answer.

    Consequences: retaliation against such an attacker stops whatever its rearm countdown (S-0309
    claim 18 stops on *illegal* and *impossible*); a building in *attack* that holds such a target
    drops it (S-0215 claim 32) rather than waiting out its countdown, unless its delayed shot is
    pending. It then keeps the target until the end of the delay; the end-of-delay check and the
    target handling after it follow S-0301 claim 14. Separately from the rank, an
    ordinary click offers no attack on such a target even while the firer is rearming, since
    S-0214 claim 6(e)'s cursor query does not count rearming at all.

    The answer of these two rows is *illegal*, not *impossible*. Retaliation and a building's
    *attack* treat the two alike, but S-0214 claim 6(e) withholds an ordinary click's attack only
    from a target that the firing rules rule illegal, so *impossible* would leave the attack
    offered.

    The firing check does not test the anti-ground flag for infantry, vehicle, aircraft or building
    targets. So the firing check does not refuse a ground object to an object whose weapons'
    projectiles are anti-air only when its type's land-targeting mode allows land targets, and with
    mode 1 refuses the targets that the land row describes. The firing check's other rows, S-0214's
    order legality and the behaviour rules of S-0215 still apply. No flight or detonation rule of
    this spec (no claim other than this one) depends on the anti-ground flag, so such a shot flies
    and detonates by those claims as any other; whether it damages the ground unit is open (Open
    questions). Automatic target choice uses both flags (S-0309); order legality is S-0214.

    **Q-1763, question 1.** The firing check refuses no infantry, vehicle, aircraft or building
    target by the projectile's anti-ground flag: an infantry unit, a vehicle, a building or a landed
    aircraft is refused only by the check's other rows, among them the land and water rows,
    which depend on the firer's targeting modes. An airborne aircraft is refused to a
    projectile that is not anti-air (first row). Retaliation adds no flag test of its own
    (S-0309 claim 18). This answer has this claim's backing and confidence for the missing
    anti-ground test.

    **Q-1763, question 3.** Every shot passes this firing check, whatever chose its target, except
    the shots that S-0301 claim 9 makes without a check of their own after a checked shot (the later
    shots of an aircraft's full burst, and a building's secondary shot or attempt in the processing
    of its primary): a target kept from an earlier order or search, a force-fire at an object, an
    attack named by a script or a trigger, or retaliation. No other shot is known to be fired
    without it. So the land row with mode 1 refuses the targets that row describes on every one of
    these routes, and the missing anti-ground test lets a firer whose type has mode 0 or 2 fire at a
    ground object on each of them, its projectiles being anti-air only, subject to S-0214's order
    legality and to S-0215 claim 32's rule for a building whose type has the SAM flag. A force-fire
    at a cell is refused by the third row when the projectile is not anti-ground. A held target that
    lifts off is refused by the first row, unless it is the firer's gripped unit. The parts of this
    answer are rated as the rows they rest on (Confidence below): high for the first row (apart
    from a held target that lifts off) and the third row, medium for the other rows and for the
    answer as a whole, low for the rank of the water and land rows and for the first row as the
    refusal of a held target that lifts off. Which routes can
    name a ground object for such a firer is S-0214's (order legality) and S-0309's (searches); what
    attack behaviour does with a refused target is S-0304 claim 2's for units other than buildings
    and S-0215 claim 32's for buildings (it drops it).
    The nuclear payload of claim 26 is not one of these shots: it is created by a trigger action
    with no firer, so no firing check is asked for it, and the answer to question 3 does not
    cover it.
    - Backing: `measurable` (scenario P16; run e for the first row; part i for the first row's
      refusal of a held target that lifts off, which shows that the firer neither shoots at it nor
      keeps it but cannot tell the first row's refusal from an earlier drop by another rule, an open
      question of S-0309; part f for the refusals of the water and land rows and for the absence of
      the anti-ground test on object targets; part g for the land row's refusal being one that stops
      retaliation; part h for the land row's answer being *illegal* and, through its mode-0 copy,
      for the absence of the anti-ground test on object targets; its water variant for the water
      row's answer being *illegal*; part f (4) for the land row's refusal of a cell target).
      `unmeasured` for a unit's deadline for the drop (no later than the first *illegal* answer):
      part i does not judge the tick of the drop. `unmeasured` for
      the two rows' place within rank 4: no scenario combines a refused target with a hidden
      target, a spent spawner or the other answers named there, except a building's pending
      delayed shot, which S-0301 scenario F21 sets against the water and land rows (parts a and
      b) through the target kept and dropped. `unmeasured` for the rank of the
      two rows against a rearm countdown's *rearming*: S-0214 claim 6(e)'s cursor query does not count rearming, so
      no cursor probe tells the ranks apart, and no scenario here reads the firing answer during
      a countdown.
      `unmeasured` for their rank against *out of ammunition*, *cloaked* and *out of range*: no
      scenario puts an empty or cloaked firer against such a target, and the readers of part h
      ask the check without its range test. `unmeasured` for the
      absence of an anti-ground test in the flight and detonation claims: it is the absence of a
      rule across this spec's other claims, whose scenarios do not vary the flag. `unmeasured` for the water
      row's mode-0 branch: its target must be named while it is cloaking, cloaked or uncloaking, and
      no order or script that names it in those states is established; it rests on S-0308 table N.
      `unmeasured` for the gripped-unit exception: it rests on S-0607 claim 11 and S-0301 claim 9.
    - Confidence:
      - high: the first row, except its refusal of a held target that lifts off; the third row.
      - medium: a held target that lifts off is neither shot at nor kept (part i).
      - low: that the first row is what refuses it, rather than another rule dropping it earlier
        (part i cannot tell the two apart, see the backing).
      - low: a unit's deadline for that drop, no later than the first *illegal* answer
        (`unmeasured`).
      - medium: the gripped-unit exception, below the ratings of S-0607 claim 11 and S-0301
        claim 9.
      - medium: the layer row.
      - medium: a building's pending delayed shot ahead of the two rows, and the target kept until
        the end of the delay, as S-0301 claim 9 rates it, until S-0301 scenario F21 runs.
      - medium: the water row's modes 2 and 6, and its answer *illegal* for mode 6 (part h's water
        variant, through S-0214 claim 6(e)); for mode 2 that answer is `unmeasured`, since part f
        (3) shows only whether a shot is fired.
      - medium: the land row for an object, its stopping retaliation, and its answer *illegal*
        (parts f, g and h, none of them run yet; part h through S-0214 claim 6(e), itself
        medium).
      - medium: the land row for a cell, until part f (4) runs.
      - medium: the absence of the anti-ground test on object targets, and the answer to Q-1763
        question 3 (part f (1) and (2) cover the force-fire and the script or trigger routes,
        and no route without the firing check is known, but not every route was checked).
      - low: the rank of the water and land rows against a rearm countdown's *rearming*, *out of ammunition*,
        *cloaked* and *out of range*, and their place within rank 4 (`unmeasured`).
      - low: the absence of an anti-ground test in the flight and detonation claims
        (`unmeasured`).
      - low: the water row's mode-0 branch.

24. **Draws of a launch.** For a shot, the random-state recorder observes the projectile's claim 4 radius draw followed by its direction draw (inaccurate arcing only), then the claim 9 radius draw followed by its direction draw (invisible flak-scattering only). Placement takes no other draw. The recorder observes a spray-attack firer's pick (S-0313 claim 9) before these projectile draws (S-0301 claim 18). Later recorded draws belong to the projectile's turns (trailer animations, claim 16) and to its detonation (claim 21, including the draws after the last cluster detonation, and S-0303).
    - Backing: `measurable` (scenario P3 with the random-state recorder for the launch draws; P14
      variant d for the draws of claim 21 and P10 variant c for a trailer's draw before the
      detonation's)
    - Confidence: high

### Detonation that waits for an animation

25. **The nuclear warhead.** A projectile whose warhead is registered under the nuclear warhead
    name (Import mapping; compared without regard to case; a name the original fixes, not a field
    of the rules and not the game-wide setting that names the nuclear-missile superweapon's
    warhead) does not detonate in
    a turn in which claim 19 says it does, whichever row of T1 is the cause (a projectile that is
    removed for leaving the map is removed as claim 19 says, with none of what follows). In that turn it instead does the
    following, at the position that claim 20 (a) and (b) give where they apply:
    - starts the nuke flash of S-0104 on that tick,
    - raises a radar event at its cell (the interface specs own radar events),
    - and creates the animation of the animation type registered under the nuclear fireball
      animation name (Import mapping) at that position; when no animation type has that name, none
      is created and the projectile detonates at once, as every other projectile does.

    The projectile then stays in play and does nothing in its later turns (no move, no trailer, no
    degeneration, no T1 row) for as long as that animation exists. In its first turn after the
    animation has ended it detonates, with claims 20 (c) and 21 applying to the position it
    holds, and leaves play in that turn. So the damage of such a shot lands one animation length
    after the projectile arrives, not on arrival (S-0203 claim 5 d). A projectile whose warhead has
    any other name detonates as every other projectile does, whatever the superweapon settings say.
    - Backing: `measurable` (scenario P19; its run d for the comparison without regard to case
      and run e for the absent fireball-animation type)
    - Confidence: medium (the selection by name and the wait: high; the radar event and the
      creation of the animation at the snapped position: medium)

26. **Trigger-created nuclear payload flight.** S-0524 claim 3 owns the launch inputs of
    trigger action 95, not the ordinary shot launch of claims 1 to 7: there is no firer or
    attacking house, the stored projectile speed is 50 leptons per tick, and its initial velocity
    is S-0524's nuclear velocity formula. With the original table trigonometry, that formula
    gives a vertical component of exactly −100 leptons per tick and two tiny negative horizontal
    components, not a downward velocity of 50. Placement still applies claim 9, including its
    steering rescaling when the projectile type has a positive turn rate. Subsequent flight is
    determined by the imported projectile properties, not by the nuclear warhead name.

    For a vertical payload with turn rate 0, not invisible, ranged, dropping, level or
    flak-scattering, its unchanged initial speed already exceeds the stored speed of 50.
    Claim 11 therefore makes no acceleration change. The integer displacement in every turn is
    exactly (0, 0, −100) leptons before detonation-point adjustments: the tiny horizontal
    components truncate to 0. Provided its
    detonation altitude is above its start, no other T1 row intervenes and the target is a flat
    empty cell without a bridge, it reaches surface height 0 in turn 200, ends turn 201 at
    height −100, and requests detonation in turn 202. The negative-height test uses the
    start-of-turn height and is strict; height 0 in turn 201 does not suffice. Claim 20 raises
    the detonation point to the surface. These heights are relative to the target surface, not
    absolute world heights.

    On the same flat setup with a stationary ground vehicle centred on the target cell,
    proximity instead requests detonation in turn 199, at height 100 before the proximity
    snap, and moves the point onto the vehicle. On an empty high-bridge cell, the downward
    deck crossing requests detonation in turn 201, at 100 leptons below the deck, without
    moving the vertical projectile to the deck height (claim 11). The terrain is still 316
    leptons below that point. Claim 25 starts its nuclear wait on the predicted turn, not
    after a constant-speed 400-turn fall; damage waits for the animation as usual.
    - Backing: `measurable` (P40; S-0524 scenario C3 for attribution and action inputs)
    - Confidence: high for the launch-speed distinction and the vertical predictions under
      the stated conditions; other projectile-property combinations retain claims 10 to 15's
      confidence

### Railgun shot

27. **Railgun hits and terrain.** A weapon with the railgun flag computes this effect at the
    shot, only when the firer's railgun emitter place is empty (S-0616 claim 1). It uses the
    start point, target position, ordered samples and terrain stopping condition of S-0616
    F14, not the projectile's flight or the railgun particles' later positions.

    **Objects hit.** Only cells newly entered by those samples contribute candidates; the
    start cell is not visited unless a later sample enters it. In a visited bridge cell the
    candidates are the objects of its bridge-deck layer when the sample's world height is at
    least (the cell's height level + 4) × 104 leptons, otherwise its ground layer. In other
    cells they are the ground-layer objects. Among these candidates, infantry, vehicles,
    aircraft and buildings other than the firer qualify when they are the shot target or pass
    the line-distance test below against the game-wide railgun hit distance (integer leptons).
    For a finite distance the comparison is strictly less than. Buildings count
    as distance 0, whatever their position within the cell. No alliance test excludes friendly
    candidates. An object qualifies at most once even when listed in several visited cells.
    If the shot target is an object rather than a cell and was not encountered, it is included
    at the end, including a terrain-object target. Thus a target in the start cell can still
    be hit. Cell listing order is S-0209 claim 13's; hits are ordered by the first qualifying
    encounter in sample order, with that otherwise-unencountered target last.

    **Line distance and precision (Q-1405 questions 2 and 3).** Let *P*, *Q* and *C* be the
    integer lepton positions of the beam start, shot target and candidate. For a non-building
    candidate the distance is *D* below. Every subtraction, multiplication, addition and division
    in these equations rounds to double precision toward zero after that individual operation
    (S-0205 claims 4 and 5). RZ₂₄ additionally narrows to single precision. Parentheses fix the
    grouping, and no multiply-add is fused. Integers enter double precision exactly, not through
    a wrapping integer difference.

    > *p* = (RZ₂₄(*P*ₓ), RZ₂₄(*P*ᵧ), RZ₂₄(*P*_z))
    >
    > *d* = (RZ₂₄(*Q*ₓ − *p*ₓ), RZ₂₄(RZ₂₄(*Q*ᵧ) − *p*ᵧ), RZ₂₄(RZ₂₄(*Q*_z) − *p*_z))
    >
    > *u* = (*C*ₓ − *p*ₓ, RZ₂₄(*C*ᵧ) − *p*ᵧ, RZ₂₄(*C*_z) − *p*_z)
    >
    > *a* = (*u*ᵧ × *d*_z − *u*_z × *d*ᵧ, *u*_z × *d*ₓ − *d*_z × *u*ₓ, *d*ᵧ × *u*ₓ − *u*ᵧ × *d*ₓ)
    >
    > *h*ₓ = RZ₂₄(*a*ᵧ × *d*_z − *a*_z × *d*ᵧ)
    >
    > *h*ᵧ = RZ₂₄(*a*_z × *d*ₓ − *d*_z × *a*ₓ)
    >
    > *h*_z = *d*ᵧ × *a*ₓ − *a*ᵧ × *d*ₓ
    >
    > *N* = RZ₂₄((*h*_z × *u*_z + *h*ᵧ × *u*ᵧ) + *h*ₓ × *u*ₓ)
    >
    > *L* = (*h*ᵧ × *h*ᵧ + *h*_z × *h*_z) + *h*ₓ × *h*ₓ
    >
    > *D* = |*N* / *r*(*L*)|

    Thus *d* is single precision, *u* and *a* are double precision, *h*ₓ and *h*ᵧ are single
    precision, *h*_z is double precision, *N* is single precision and *L* is double precision.
    Single values widen exactly when used in an operation. The table square root *r* is S-0204
    claim 5's real-input root, including its single-precision input narrowing. It is used only
    on *L*, not on a separate beam length or numerator. *D* remains double precision, in
    leptons, with no integer truncation before comparison. Geometrically this is the magnitude
    of the projection of *u* on the double cross product (*u* × *d*) × *d*, not the simpler
    cross-product-length quotient; replacing it with that quotient changes boundary results.
    With `quirks.chop_rounding` off, RN₅₃ and RN₂₄ replace the respective toward-zero roundings
    throughout, as in S-0205 claim 5.

    **Degenerate comparisons.** When the calculated *h* is zero, *N* and *r*(*L*) are both 0,
    so *D* is not a number. The original accepts this unordered comparison as a hit, even when
    the hit distance is 0 or negative (`quirks.railgun_collinear_hits`, default on). In
    particular, a non-target mobile candidate exactly on an axis-aligned beam qualifies, not
    merely when the hit distance is positive. For a general diagonal, use the equations: exact
    geometric collinearity need not survive their rounding. A building instead uses finite
    distance 0 and, unless it is the shot target, qualifies only for a positive hit distance.
    The shot target bypasses both comparisons.

    With *P* = *Q* the geometric line has no direction; a non-building candidate tested against
    the resulting zero *d* would get not-a-number, not its distance to *P*. But a zero-length
    beam at ordinary legal map coordinates has every sample in the start cell, so no cell
    contributes any candidates. Only the object shot target is hit through the target fallback,
    not other occupants of that cell; a cell target adds no object. The emitter's end point
    remains *Q* (S-0616 F14).

    **Damage.** When no sample stops in terrain, every qualifying object takes one ordinary
    hit of S-0302 claim 1: the weapon's integer damage field, without the firer's shot-damage
    bonuses, distance 0 leptons, the weapon's warhead, the firer as attacker, no explicit
    attacking house, defences not ignored and passenger escape allowed. This is not an area
    detonation. In particular its amount is not the projectile's stored shot damage.

    **Terrain and random draws.** A terrain stop cancels all of these hits, including hits on
    objects before the stop. Only when the stop cell is a destroyable-cliff cell (S-0307 claim
    23) is one ranged scenario draw R(0, 99) taken; the cliff is collapsed when that integer is
    strictly below the game-wide cliff-collapse chance (integer percentage points). A stop on
    other terrain takes no such draw. Without a terrain stop, the same one-draw test is made
    after the ordinary hits only when the shot target is a destroyable-cliff cell. An ordinary
    cell target takes no draw. Even chances at or below 0 or at least 100 still take the draw
    when the cell qualifies. These draws precede creation of the railgun emitter and its
    later particle draws (S-0616 claims 1 and 17).
    - Backing: `measurable` (P41 for hits, ordering, damage, terrain cancellation and draw gates,
      with S-0616 scenario P22 for the samples and end point; P41 parts b and c for precision,
      collinearity and zero-length beams). No oracle measurement is claimed.
    - Confidence: high for the gates, candidate kinds, layer choice, unique hits, raw damage,
      arithmetic, degenerate comparisons and draw conditions; medium for observable hit ordering

### Drawing the projectile

Claims 28 to 33 are presentation only: they decide what the screen shows and never change the
game state. They answer Q-1570.

28. **What is drawn, and where.** In every drawn frame, each projectile in play is drawn in the
    object pass of S-0102 at its *draw point*: the screen point of its position, height lift
    included (S-0101 F1), with no further shift. Its body is a sprite image frame (claims 29 and
    31) or, when the art section of its image has the voxel flag, the body model of its voxel art
    (claim 30); never both. A sprite body is placed centred on the draw point (S-0102 claim 19).
    A voxel body's copy rectangle is placed around the draw point as S-0105 claim 25 places a part.
    The layer (surface for an image marked flat, air otherwise) and the depth are S-0102's (claims
    3, 12 and 13). Nothing at all is drawn for the projectile (no body, no shadow, no stand-in
    picture) when any of these holds:
    - it is invisible (the invisible flag),
    - it is waiting for its detonation animation (claim 25),
    - it is drawn as a sprite and has no sprite: its image name is empty (S-0007 claim 26) or no
      file resolves for it (S-0007 claim 27),
    - it is drawn as a voxel model and its voxel art is absent or discarded (S-0007 claim 18; the
      original faults instead, Edge cases).

    A sprite image frame number at or above the image's frame count draws nothing, so the body and
    its shadow (claim 32) are both missing in that drawn frame.
    - Backing: `measurable` (scenario P42 parts a, f and g)
    - Confidence: high (the draw point, the cases that draw nothing); medium (that an image frame
      number beyond the frame count draws nothing)

29. **Sprite image frame.** The image frame of a sprite body is chosen afresh in every drawn frame
    from the projectile's state after its latest turn (F14):
    - **Animated.** When the image's animation range is set (its low or its high animation field is
      not 0), the image frame is the projectile's animation counter *a* (F15), whatever the
      rotating flag says. The counter starts at 0 when the projectile is placed, even when the low
      field is above 0, and advances by one every *R*ₐ turns, where *R*ₐ is the rate field (a rate of
      0 holds each value for 256 turns). After the high field it returns to the low field.
    - **Rotating.** Otherwise, when the image has the rotating flag, the image frame shows the
      horizontal direction of the projectile's velocity, in 32 directions: image frame
      (28 − *ψ*) mod 32, where *ψ* is the 32-step facing of that direction (F14). Image frame 0
      points straight up the screen, and the numbers grow as the direction turns anticlockwise on
      the screen: 8 points left, 16 down and 24 right. The vertical component of the velocity
      plays no part. A velocity with no horizontal part counts as pointing map east (image frame
      20).
    - **Neither.** Otherwise the image frame is 0.
    - Backing: `measurable` (scenario P42 parts a to c)
    - Confidence: high (the direction table, its rounding and the animation counter); medium (the
      single-precision inputs of the direction at sector boundaries)

30. **Model body.** A projectile drawn as a voxel model draws only the body model of its voxel art,
    never a turret or barrel model (S-0007 claim 27), in motion-file frame 0, freshly in every
    drawn frame:
    - **Orientation.** It is drawn with the body transform Rz(−(π/16)(*ψ* − 8)) · Ry(+(π/16)(*e* − 8)),
      under S-0105's fixed view transform (claim 1) and followed by each section's motion-file
      transform (S-0105 claims 13 to 15), where *ψ* is
      the 32-step facing of the horizontal direction of its velocity (as in claim 29) and *e* the
      32-step facing of its velocity's pitch facing (F14). The heading follows S-0105 claim 3 and
      the pitch S-0105 claim 6's sign: *e* below 8 raises the nose. The pitch therefore shows in
      steps of 11.25°, from straight up (*e* = 0) through level (*e* = 8) to straight down
      (*e* = 16). It does not use the launch facing or the firer's facing.
    - **Light.** Before the sections are drawn, each normal index of the model's normal table gets
      the shade of F16: one diffuse term of the fixed light of S-0105 claim 17, scaled to 16 and
      truncated, with no specular term and no eye vector (unlike S-0105 F4). The table is the one
      selected by the normal mode of the model's first section. Normal indices 253 to 255 get shade
      16, and indices at or above the table's size keep the shade an earlier draw gave them
      (S-0105 claim 20). The pixel is then looked up in the voxel lighting table as S-0105 claim 19
      says, and claims 21 to 23 of S-0105 rasterise it.
    - **Palette and brightness.** The voxel image is copied to the screen with the house-colour
      palette that the projectile type's colour definition selects (Import mapping; S-0103 claim
      5), at brightness 1000, through the light-alpha values (S-0104 claim 15). That is the
      53-shade palette of the named definition; with the key missing or empty, the 53-shade
      palette of the definition already in force, at first the first definition. A name that
      matches no definition keeps the palette in force; before any read has selected one, that is
      the first definition's single-shade palette, which has one shade row only. It is never the
      owner's colour, and the cell's light does not change it. Whether that palette's shade table
      takes the light's colour is S-0104's (claim 3).
    - **Depth.** A non-flat voxel projectile neither tests nor writes depth: it is not an
      upright depth reader with zero bias. A flat voxel projectile reads depth with the flat
      profile and strict test of S-0102 claims 10, 12 and 13, with final bias −65 depth units
      and no height-lift term. It never writes depth.
    - **No shadow.** A voxel projectile never draws a shadow, whatever its shadow flag.
    - The rotating flag, the animation fields and the palette flags of claim 31 do not affect it.
    - Backing: `measurable` (scenario P42 parts d and e; depth: S-0102 scenario D45)
    - Confidence: high (orientation, steps, frame 0, depth, no shadow, fresh drawing); medium (the shade
      rule and the palette selection)

31. **Sprite palette and brightness.** A sprite body uses the first of these palettes that applies:
    1. the anim palette (S-0103 claims 1 and 8), when the image has the anim-palette flag,
    2. when the projectile type has the firer's-palette flag, the 53-shade house-colour palette of
       the house that owned the firer when the projectile was created. A projectile created without
       a firer uses the 53-shade palette of the local player's house instead, taken at each drawing,
    3. otherwise the general palette (Terms).

    The body is always drawn at brightness 1000, through the light-alpha values (S-0104 claim 15):
    the light of its cell, its height and lighting effects do not change it, but shroud cover
    darkens it and a light-glow image brightens it (S-0110 claims 5 and 6). The general palette's
    shade table is never tinted (Terms), nor is the anim palette's (S-0104 claim 6), so a coloured
    light leaves the body's colours unchanged; for the house-colour palette of case 2, S-0104
    (claim 3) says whether the table is tinted. The body is never drawn translucent: no art field of the image
    makes it see-through (claim 33).
    - Backing: `measurable` (scenario P42 parts f and h)
    - Confidence: high (the order of the three palettes, brightness 1000, no translucency); medium
      (the palette of a projectile created without a firer)

32. **Shadow.** A projectile drawn as a sprite whose type has the shadow flag (rules) also draws a
    shadow while its shadow height *η* is greater than 0. *η* is its height above the terrain
    (glossary), except over a high bridge's cell when that height is at least the deck height of 4
    height levels (416 leptons): there *η* is the height above the deck, its height above the terrain
    minus 416, and the shadow lies on the deck. (A projectile is never treated as standing on a
    bridge deck.) The shadow is the body's own image frame of claim 29, drawn as a shadow (S-0102
    claim 26: a plain darkening that ignores palette, brightness and light-alpha values), centred
    on the point *L*(*η*) rows below the draw point, where *L* is S-0101's height lift of the
    height *η*. It is drawn just before the body, so the body covers it where they overlap. Its
    depth is S-0102's (claim 13). There is no generic shadow image.
    - Backing: `measurable` (scenario P42 part g)
    - Confidence: high (same image frame, placement, the height condition); medium (the deck case)

33. **Light-glow image of a projectile.** A projectile whose type has a translucent sprite
    (S-0007 claim 23) shows it as a light-glow image (S-0104 Terms, S-0110 claim 6) from the tick
    it enters play: it changes the light-alpha values under it and is never painted. This is the
    only thing the translucent sprite does for a projectile. Whether the glow follows the
    projectile while it flies, and when it goes, is not established (Open questions).
    - Backing: `measurable` (scenario P42 part h)
    - Confidence: medium

## Formulas

Every floating-point step is rounded toward zero in the original (S-0205 claim 5,
`quirks.chop_rounding`); trunc is truncation toward zero. *r* is the table square root (S-0204 F1,
`quirks.table_root_distance`); for a real argument *x* it is applied to |*x*| narrowed to single
precision. The table angle *A*(*Y*, *X*) and the facing scale are those of S-0204 F3
(`quirks.table_direction`), with both arguments narrowed to single precision. This spec defines no
switch of its own for these. Angles from a facing *f*: θ(*f*) = (*f* − 16383) × (−π / 32767)
radians, the constant being the double nearest −π/32767 (Constants), with *f* read as a signed
16-bit value as for the pitch facing (Terms): every facing and pitch facing in this spec is.

**F1. Initial velocity** (claim 7). With launch facing *y* (S-0204 F3 output), pitch facing *p* and
the start speed *s* of claim 2 (1 for a steering or vertical projectile whose firer type has no
radial fire segments, else the capped shot speed):

> *s*₀ = *s* when *s* > 0, and *s*₀ = 100 when *s* = 0 (Edge cases)
> horizontal part: *L* = *r*(*s*₀²); *χ*ₓ = RZ₅₃(cos_t(θ(*y*)) × *L*), *χ*ᵧ = −RZ₅₃(sin_t(θ(*y*)) × *L*)
> velocity: *M* = *r*(RZ₅₃(*χ*ₓ² + *χ*ᵧ²)); *V*ₓ = RZ₅₃(cos_t(θ(*p*)) × *χ*ₓ), *V*ᵧ = RZ₅₃(cos_t(θ(*p*)) × *χ*ᵧ), *V*_z = RZ₅₃(sin_t(θ(*p*)) × *M*).

So the horizontal part has length *L*, the table root of *s*₀² (*s*₀ = 100 when *s* = 0), up to the
rounding of the products, and points in the launch facing *y* in every case, a zero start speed
included. *L* equals *s*₀ for *s*₀ below 129 and can be one lepton short for
larger values (S-0204 claim 5); *M* is a second table root of the rotated horizontal vector, so
*V*_z need not equal sin × *s* exactly. Radial segments (claim 7) replace *y* by

> *y* = trunc(RZ₅₃(RZ₅₃(RZ₅₃(θ(*b*) + *φ*ₖ) − π/2) × *c*₂)), with *φ*ₖ = RZ₅₃(RZ₅₃(RZ₅₃(π / *n*) × *k*) − π/2),

*b* being the body facing, *n* the radial segment count and *k* the shot's index; π and π/2 are
the doubles nearest them, *c*₂ the facing scale of S-0204, θ(*b*) the double of Terms, and every
step is in double precision rounded toward zero.

**F2. Arc speed.** *v*(*L*, *g*) = trunc(*r*(RZ₅₃(RZ₅₃(*L* × *g*) × 1.2))), *L* in leptons, *g* in
leptons per tick², result in leptons per tick.

**F3. Reachability** (*v*, *d*, *z*, *g*; *d* = 0 counts as 0.001; used by S-0307 claim 9, not by the
launch):

> disc = *v*⁴ − 2*gzv*² − *g*²*d*²; *ω* = 2(1 + *z*²/*d*²); reachable ⇔ disc ≥ 0 and (*v*² − *gz* + *r*(disc)) / *ω* ≥ 0.

All in double precision rounded toward zero.

**F4. Ballistic launch elevation** (claim 6 a). Unlike F3, F4 makes no substitution for *d* = 0.
With *v* the start speed *s* (claim 2), *d* the planar distance to the impact point and *z* = Δ*z*, all in
double precision rounded toward zero, the solution θ for distance *d* is:

> for *z* > 0: θ = π/4, whatever *v*, *d* and *g* are, so a solution always exists,
> for *z* ≤ 0: disc = *v*⁴ − 2*gzv*² − *g*²*d*², *ω* = 2(1 + *z*²/*d*²),
> *U* = (*v*² − *gz* ± *r*(disc)) / *ω* (+ for the flatter shot, − for a lob),
> θ = acos_t(*r*(*U*) / *v*).

For *z* ≤ 0 a solution exists exactly when disc ≥ 0, *ω* is neither 0 nor undefined, and *U* ≥ 0.
A disc, *ω* or *U* that is not a number fails its test and gives no solution. (A quotient
*r*(*U*) / *v* that is not a number is not such a failure: the table arccosine maps it to index 0,
see *Zero planar distance* below.) F4 has a solution exactly when θ exists. θ′ is the flatter
solution of the same solve with *d* + 1 in place of *d*. The launch angle is:

| Shot | Launch angle |
|---|---|
| lob | θ |
| flatter shot, θ′ < θ | −θ |
| flatter shot, θ′ ≥ θ | θ |
| flatter shot, no solution for *d* + 1 | unspecified in the original (Edge cases); our engine: θ |

The pitch facing is trunc((launch angle − π/2) × *c*₂). The 45° case gives pitch facing
trunc((π/4 − π/2) × *c*₂) = 8191; for *z* > 0 the solve for *d* + 1 also gives θ′ = π/4, so the
flatter shot keeps θ.

*Zero planar distance* (*d* = 0, `quirks.zero_distance_arc`). With *z* = 0, *z*²/*d*² is 0/0, not a
number, so *ω* is not a number and F4 has no solution (a void shot). With *z* < 0, *z*²/*d*² is
infinite, so *ω* is infinite and *U* = 0:
- *v* > 0: *r*(*U*)/*v* = 0, the table index is 2048 and θ = π/2 exactly. A lob is launched
  straight up (pitch facing 0). The flatter shot compares with the solve for distance 1, whose
  *ω* = 2(1 + *z*²) is finite: θ′ < π/2 exactly when its table index trunc(2048 × (*r*(*U*′)/*v* + 1))
  is 2049 or more, that is when *r*(*U*′)/*v* reaches about 1/2048 (for example *v* = 20, *g* = 10,
  *z* = −300 gives index 2065). Then the launch angle is −π/2 and the pitch facing is
  trunc(RZ₅₃(−π × *c*₂)) = 32766 (straight down; the exact product lies just below 32767). With a
  smaller index it is launched straight up (pitch facing 0).
- *v* = 0: *r*(*U*)/*v* is 0/0, not a number, and the table arccosine converts it to the index 0
  (S-0205 claim 7: a value that is not a number converts to 0), so θ = π/2 − *A*ₛ[0] =
  3.1415925780918936, just short of π. A lob gets pitch facing trunc((θ − π/2) × *c*₂) = −16383,
  which points horizontally backwards (Terms). For the flatter shot the solve for distance 1 has
  disc = −*g*² < 0 when *g* > 0, so the unspecified case of Edge cases applies, and our engine
  launches it at θ like the lob. With *g* = 0 that solve also gives θ′ = θ, with the same result.
  With the start speed of 0, F1 builds the velocity at a horizontal speed of 100 (Edge cases,
  `quirks.zero_speed_launch`), so the projectile leaves backwards at 100 leptons per tick. A start
  speed of 0 is the usual case for ρ = 0 here, since the shot speed of claim 1 is then *v*(0, *g*) =
  0 whenever the shot target's target point lies exactly below the fire point too.

"Backwards" means opposite the launch direction of claim 5. For ρ = 0 that direction at *d* = 0
is the facing that S-0204 F3 gives for a zero displacement (map east), so such a projectile leaves
towards map west.

**F5. Scatter** (claims 4 and 9). Direction draw *ξ* = R(0, 2147483646). Every real-valued step
is in double precision, rounded toward zero (RZ₅₃); no step is narrowed to single precision:

> *f*_ξ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(*ξ* × *k*_ξ) × 2π) − π/2) × *c*₂)), taken as a signed 16-bit value,
> *Q*ₓ = trunc(RZ₅₃(*P*ₓ + RZ₅₃(cos_t(θ(*f*_ξ)) × *σ*))), *Q*ᵧ = trunc(RZ₅₃(*P*ᵧ − RZ₅₃(sin_t(θ(*f*_ξ)) × *σ*))), *Q*_z = *P*_z,

where *k*_ξ = 4.656612877414201 × 10⁻¹⁰ is the double nearest 1/2147483646, 2π and π/2 are the
doubles nearest them, *c*₂ is the facing scale of S-0204 F3, θ(*f*_ξ) is the double of Terms, sin_t
and cos_t are the single-precision table values (S-0105 F1), and *P* and *σ* are integers. In
claim 4, *P* is the displacement from the fire point *F* to the aim point (aim point − *F*, integer
components) and the result (*Q*ₓ, *Q*ᵧ, *Q*_z) is likewise a displacement from *F*: the impact
point is *F* plus it. The truncation toward zero therefore acts on the displacement, and for a
negative displacement component it can differ by one lepton from truncating the map coordinate itself. In claim 9, *P* is
*K* in map coordinates and the result is the scattered *K*. The products R(…) × *D* in
claims 4 and 9 are 32-bit integer products and the divisions truncate toward zero.

**F6. Lead** (claim 3). In double precision, every step rounded toward zero (RZ₅₃), no step
narrowed to single precision:

> ℓ = trunc(RZ₅₃(RZ₅₃(*D*₃ / RZ₅₃(*w* × 0.9)) × *u*)),
> aim point = (trunc(RZ₅₃(*x* + RZ₅₃(cos_t(θ(*b*)) × ℓ))), trunc(RZ₅₃(*y* − RZ₅₃(sin_t(θ(*b*)) × ℓ))), *z*),

with (*x*, *y*, *z*) the reference point of the firer's target, *b* the vehicle's body facing,
θ(*b*) the double of Terms, 0.9 the double nearest it, and *D*₃, *w*, *u*, ℓ, *x*, *y* integers.
(A map coordinate needs up to 17 integer bits, so single precision would give other leptons; the
original uses double precision here.) When *w* = 0, RZ₅₃(*w* × 0.9) is 0 and the quotient is
infinite (or not a number when *D*₃ = 0 or *u* = 0), and the conversion of S-0205 claim 7 turns
it into ℓ = 0, so the aim point is the target's reference point itself.

**F7. Steering turn step** (claim 13): *q* = trunc(RZ₅₃(RZ₅₃(sin_t(RZ₅₃(RZ₅₃(*j* × *k*₁₅) × 2π)) × *m*) +
(*m* + 1)) × ρ), *k*₁₅ being the double nearest 1/15 and 2π the double nearest 2π. The effective
value is *q*\* = trunc(*q* × 1.5) near the target (claim 13) and *q*\* = *q* otherwise, and
*t* = 256 × (*q*\* mod 256) reduced to a signed 16-bit value, that is
256 × (*q*\* mod 256) − 65536 when 256 × (*q*\* mod 256) ≥ 32768. Turning towards
a goal facing *G* from *f* by step *t*: when |*t*| ≥ |δ(*f*, *G*)| the facing becomes *G*; otherwise
it becomes *f* + *t* when δ(*f*, *G*) ≥ 0 and *f* − *t* when δ < 0, in 16-bit facing arithmetic (δ
the signed facing difference, S-0204).

**F8. Table arccosine.** acos_t(*x*) = π/2 − *A*ₛ[*k*], *k* = trunc(2048(*x* + 1)), with *A*ₛ the
table arcsine of Terms; for *x* in [−1, 1] the index lies in 0 … 4096.

**F9. Snap distance** (claim 20 b). With *P* the detonation point after the raise of claim 20 (a) and
*P*_R the target's reference point, all integer lepton coordinates: *ζ* = trunc((*P*_R.z + *P*.z) / 2) −
*P*_R.z; *ν* = trunc(*r*(Δ*x*² + Δ*y*² + *ζ*²)) with Δ = *P* − *P*_R and the squares formed in double
precision; *ν*′ = trunc(*ν* / 3). So the vertical difference counts at about a half, and the
distance is divided by 3 (integer division). *ν*′ is used only in a turn in which a "reached" row
applies and the arrival row does not, since the arrival row always snaps (claim 20 b). The threshold is max(2|*V*|, 128) in real arithmetic, and the snap happens when
*ν*′ is not above it.

**F10. Velocity rescaling and the speed change** (claims 9, 11, 12). The velocity *V* *rescaled to
length L* (a real number) is the vector with components RZ₅₃(*f* × *Ṽ*ᵢ), *i* = x, y, z, where
*Ṽ* = *V*, except *Ṽ* = (100, 0, 0) leptons per tick when *V* = (0, 0, 0); *μ* =
*r*(RZ₅₃(*Ṽ*ₓ² + *Ṽ*ᵧ² + *Ṽ*_z²)); and *f* = RZ₅₃(*L* / *μ*). Its length is *L* up to the table root
and the roundings, and its direction is that of *Ṽ*. In the formulas below |*V*| is the table root
*r*(RZ₅₃(*V*ₓ² + *V*ᵧ² + *V*_z²)) of the velocity at that point, a real number (not truncated)
except where trunc is written.

> Steering, with *S* the top speed (integer), and *a* = *α*, or, during course lock with *κ* = 0, 1
> in an even tick and 0 in an odd tick:
> - |*V*| < *S*: *V* becomes *V* rescaled to min(RZ₅₃(|*V*| + *a*), *S*),
> - |*V*| > *S*: *V* becomes *V* rescaled to max(RZ₅₃(|*V*| − trunc(*a* / 2)), 0),
> - |*V*| = *S*: *V* is unchanged.
>
> Vertical: when trunc(|*V*|) < *S*_w, *V* becomes *V* rescaled to trunc(|*V*|) + *α* (an integer,
> not capped); otherwise *V* is unchanged. Steering projectile at placement (claim 9): *V* becomes
> *V* rescaled to 1.

**F11. Horizontal turn** (claim 14), with a target point. Let *P*′ = *P* + (trunc *V*ₓ, trunc *V*ᵧ,
trunc *V*_z) and *W* = *P*_T − *P*′ (integers). Let *y*_c be the direction facing (S-0204 F3) of the
displacement (*V*ₓ, *V*ᵧ), taking the real components as the arguments, narrowed to single
precision as F3 has it, and *y*_g that of the displacement (*W*ₓ, *W*ᵧ). The new facing *y*′ is *y*_c
turned towards *y*_g by the step *t* (F7: *y*′ = *y*_g when |*t*| ≥ |δ(*y*_c, *y*_g)|, else *y*_c + *t*
for δ ≥ 0 and *y*_c − *t* for δ < 0). The turned velocity is

> (RZ₅₃(cos_t(θ(*y*′)) × *H*), −RZ₅₃(sin_t(θ(*y*′)) × *H*), *V*_z), where *H* = *r*(RZ₅₃(*V*ₓ² + *V*ᵧ²)),
> except *H* = *r*(10000) when *V*ₓ = *V*ᵧ = 0,

so the horizontal speed is kept and the vertical component is unchanged.

**F12. Pitch turn and rebuild** (claim 15; the same rebuild as the launch of claim 7). The pitch
facing of a velocity is *p*(*V*) = trunc((*A*(*V*_z, *H*) − π/2) × *c*₂) (the table angle and *c*₂ of
S-0204, *H* = *r*(RZ₅₃(*V*ₓ² + *V*ᵧ²)), the arguments narrowed to single precision, the result a
16-bit value). It inverts θ(*p*) of Terms only up to half a unit before the truncation (Terms,
*pitch facing*). This conversion is S-0204 F6 (claim 12 there), with the elevation angle *A*(*V*_z,
*H*) as its signed angle: at an exception index of S-0204 F3 the result is one unit larger than the
surrogate's for a positive angle (pointing up) and one unit smaller for a negative angle (pointing
down), and exact at every other index. The same holds for every pitch facing *p*(·) of this
formula, the current pitch *p*_c = *p*(*V*) and the goal pitch *p*(*W*) alike.

The velocity *V* that the pitch turn works on is, for a projectile with a target, its velocity
after this turn's heading change (F11, with its substitution for a zero horizontal part), and for
one without a target, its velocity after this turn's speed change (F10). With *p*_g the goal pitch
and *ε* the step of claim 15:

> *p*_c = *p*(*V*), *M* = *r*(RZ₅₃(*V*ₓ² + *V*ᵧ² + *V*_z²)),
> new pitch *p*′ = *p*_c for a level projectile with a target; otherwise *p*′ = *p*_g when |*ε*| ≥ |*p*_g − *p*_c|, *p*_c + *ε* when |*ε*| < |*p*_g − *p*_c| and *p*_g − *p*_c ≥ 0, and *p*_c − *ε* when |*ε*| < |*p*_g − *p*_c| and *p*_g − *p*_c < 0 (*p*_g − *p*_c a 16-bit difference),
> *Ĥ*ᵢ = RZ₅₃(*V*ᵢ / cos_t(θ(*p*_c))) (*i* = x, y),
> rebuilt velocity = (RZ₅₃(cos_t(θ(*p*′)) × *Ĥ*ₓ), RZ₅₃(cos_t(θ(*p*′)) × *Ĥ*ᵧ), RZ₅₃(sin_t(θ(*p*′)) × *M*)),

for every *p*_c, a level velocity included: cos_t(0) = 1 exactly, so for a level velocity the
division leaves *V*ₓ and *V*ᵧ unchanged. cos_t is never
exactly 0 (S-0105 F1 gives the table a small non-zero value where the exact cosine is 0), so the
division is defined for every *p*_c, including a velocity pointing straight up or down. Observable consequence of taking *M* after F11 (claim
15): a velocity of zero length before the heading change leaves F11 with a horizontal length of
*r*(10000) = 100 and a vertical component of 0, so *M* is about 100 and the rebuilt vertical
component is about 100 × sin_t(θ(*p*′)), not 0 (scenario P9 run i).

The goal pitch *p*_g for the vector to the target point is *p*(*W*) with *W* = *P*_T − *P*′ taken
as real components (so with its full vertical part). Without a target the pitch step is
*t* and *p*_g = 8192, the displacement of the turn uses the rebuilt *V* (claim 14), and the
steering distance of Terms is trunc(*r*(RZ₅₃(*V*ₓ² + *V*ᵧ² + *V*_z²))) of the rebuilt *V*.


**F13. Straight launch pitch** (claim 6 c). With Δ*z* = *Q*.z − *F*.z, *d*′ = *d*₂(*F*, *Q*) the
planar distance from *F* to *Q* (an integer), *H*_b an integer and heights in leptons:

> pitch facing = 16383 when |Δ*z*| ≤ 200,
> pitch facing = trunc((*θ*_c − π/2) × *c*₂) when |Δ*z*| > 200,

where *θ*_c = sgn(*Z*) × *A*(|*Z*| − *τ*, max(*d*′, 0.05)), the table angle of S-0204 F3 negated
when *Z* is negative (the first argument itself is never negated), and

> *Z* = Δ*z* and *τ* = 20 when the firer's target is not a building,
> *Z* = 200 × *H*_b − (world height of *F*) when it is a building, with *τ* = 20 when |*Z*| ≥ 20
> and *τ* = 0 when |*Z*| < 20.

The comparison with 200 always uses Δ*z*, also for a building target. The rounding and the
narrowing of the table angle's arguments to single precision are those of the Formulas preamble.
The conversion of *θ*_c to a pitch facing is S-0204 F6 (claim 12 there) with *θ*_c as its signed
angle: at an exception index of the table lookup *A*(|*Z*| − *τ*, max(*d*′, 0.05)) the pitch is one
unit nearer to level than the surrogate gives: +1 for a positive *θ*_c, −1 for a negative
*θ*_c, and no correction for zero. The
correction follows the sign of *θ*_c after the negation, not the sign of the table's argument.
With `quirks.building_pitch_term` off, *Z* = Δ*z* and *τ* = 20 for every target.


**F14. Drawn direction and pitch** (claims 29 and 30). Let *V* be the projectile's velocity after
its latest turn. The *drawn facing* φ is S-0204 F3's facing with *X* = *V*ₓ and *Y* = −*V*ᵧ, each
narrowed to single precision before F3's quotient (F3 with *X* = *Y* = 0 gives θ = 0, so φ = 16383,
map east). Its 32-step facing is *ψ* = f₃₂(φ) = ⌊(⌊φ / 1024⌋ + 1) / 2⌋ mod 32 (glossary). The
rotating sprite's image frame is

> image frame = (28 − *ψ*) mod 32 (non-negative remainder).

The *drawn pitch facing* *p* is S-0204 F6's pitch from a signed table angle, with the angle
*α* = the table angle of F3 taken with the horizontal argument *X* = *r*(*V*ₓ² + *V*ᵧ²) (the table
square root of the double-precision sum, so *X* ≥ 0) and the vertical argument *Y* = *V*_z, both
narrowed to single precision. *p* is read as a value from 0 to 65535 (mod 65536), and *e* = f₃₂(*p*).
For *α* from −π/2 to π/2 this gives *e* from 16 down to 0, with 8 for level flight. A velocity
with no horizontal part and *V*_z > 0 gives *e* = 0, with *V*_z < 0 gives *e* = 16.

**F15. Animation counter** (claim 29). The low field *a*_lo, the high field *a*_hi and the rate
*R*ₐ are whole numbers from 0 to 255 (Import mapping). When the projectile is placed, its counter
*a* = 0 and its countdown *c* = *R*ₐ. In each of its turns, except the waiting turns of claim 25,
when *a*_lo ≠ 0 or *a*_hi ≠ 0:

> On each such turn, the countdown decreases by one modulo 256. If the resulting countdown is nonzero, the animation counter and countdown are otherwise unchanged. If the resulting countdown is zero, the animation counter increases by one modulo 256 and the countdown is reset to `R`ₐ; when the resulting animation counter is greater than `a`ₕᵢ, it is instead set to `a`ₗₒ. These operations use whole-number values from 0 to 255, and the comparison is unsigned.

All arithmetic is on whole numbers from 0 to 255 and the comparison is unsigned. Consequences:
with 1 ≤ *R*ₐ ≤ 255, *a* advances on the *R*ₐ-th turn, the 2*R*ₐ-th turn and so on; with *R*ₐ = 0 the
countdown wraps to 255 and *a* advances every 256 turns. The values shown are 0, 1, …, *a*_hi, then
*a*_lo, …, *a*_hi repeatedly when *a*_lo ≤ *a*_hi. With *a*_lo > *a*_hi the counter stays at *a*_lo
once it gets there. With *a*_hi = 255 the step after 255 gives 0, not *a*_lo, since 0 is not above
255.

**F16. Shade of a voxel projectile** (claim 30). Let *L*ᵥ be the light vector of S-0105 claim 17 and
*M* the projectile's draw transform of claim 30 (the view included, no motion-file transform). Its
light in the model frame is *L*ᵥ′ = *R*ₘᵀ*L*ᵥ, with *R*ₘ the rotation part of *M*, in single precision.
For each normal *n* of the selected table (single-precision components):

> *d* = *n*ₓ*L*ᵥ′ₓ + *n*ᵧ*L*ᵥ′ᵧ + *n*_z*L*ᵥ′_z; shade = 0 if *d* < 0, otherwise trunc(16 × *d*).

The products and the sum are formed in at least double precision before the comparison and the
scaling. Since the vectors have unit length, the shade lies from 0 to 16.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Arc speed margin | 1.2 | factor | F2 |
| Floater gravity factor | 0.5 | factor | Terms |
| Lead speed factor | 0.9 | factor | F6 |
| Launch speed cap | 1/2 | of the spatial distance to the impact point | Claim 2 |
| Steering start speed | 1 | lepton per tick | Claims 2 and 9 |
| Horizontal speed substituted for a start speed of 0 | 100 | leptons per tick | F1 |
| Velocity substituted for a zero velocity | (100, 0, 0) in F10; a horizontal length of *r*(10000) in F11 | leptons per tick | F10, F11 |
| Course-lock acceleration (course lock duration 0) | 1 in an even tick, 0 in an odd tick | leptons per tick per turn | Claim 12, F10 |
| Course-lock speed threshold | 40 | leptons per tick | Claim 12 |
| Course-lock end margin | 0.5 | leptons per tick | Claim 12 |
| Turn variation period | 15 | ticks | F7 |
| Near-target turn factor, distance | 1.5, 256 | factor, leptons | Claim 13 |
| Direct-mode distance | 3 (6 very high) | cells | Claim 15 |
| Direct-mode turn steps | 0 and 256 (residue mod 256 of claim 13 at most 1) | facing units | Claim 15 |
| Cruise look-ahead | 6 | ticks of velocity | Claim 15 |
| Cruise height cap | 5 (10 airburst or very high) | height levels | Claim 15 |
| High-bridge allowance of the cruising altitude | 416 | leptons added to the terrain height | Claim 15 |
| Steering-distance vertical weight | 1/4 (0 for an airburst projectile) | factor on the vertical difference, integer division | Terms |
| Cruise correction, dead band | 18, 20, 52 | leptons | Claim 15 |
| Climb, descent pitch facings | 8192, 18432 | facing units | Claim 15 |
| Straight pitch threshold, allowance | 200, 20 | leptons | Claim 6 c |
| Building pitch term factor | 200 | leptons per unit of the building's art height value | Claim 6 c |
| Minimum horizontal distance in pitch | 0.05 | leptons | Claim 6 c |
| Pitch scale | the double nearest −π/32767 | radians per facing unit | Terms |
| Uphill arc pitch | 8191 | facing units | Claim 6 a |
| Zero-distance launch pitches | 0 (straight up), 32766 (straight down), −16383 (backwards); table index 2049 for the flatter shot's test | facing units (signed 16-bit), table index | F4 |
| Pitch facings that give an exactly level velocity | 16380 to 16386 | facing units | Terms (table trigonometry steps) |
| Table arcsine | 4097 entries, step 1/2048 | entries; the step is in units of the (dimensionless) sine argument | Terms, F8 |
| Reachability substitute for a zero distance | 0.001 | leptons | F3 |
| Scatter direction draw range, scale | 0 to 2147483646, 4.656612877414201 × 10⁻¹⁰ (the double nearest 1/2147483646) | draw value, factor | F5 |
| Arrival distance (steering or ranged projectile) | 64 | leptons (half-distance below 32) | T1 |
| Pass-by window | 512 | leptons (half-distance below 256) | T1 |
| Closing average | 60 turns, factor *f*₆₀ = the double nearest 59/60 (8857079267161975 / 2⁵³, 0.98333…), threshold 60, start value 0 | unlocked turns, factor, leptons, leptons | T1 |
| Drawn directions of a rotating sprite | 32, image frame (28 − *ψ*) mod 32 | directions, image frame | Claim 29, F14 |
| Drawn steps of a voxel projectile | 32-step heading, 32-step pitch (11.25° each) | facing steps | Claim 30, F14 |
| Animation counter at placement; rate 0 period | 0; 256 | image frame; turns | F15 |
| Shade scale of a projectile drawn as a voxel model; shade of normal indices 253 to 255 | 16; 16 | shade index | F16 |
| Projectile body brightness | 1000 | per-mille | Claims 30 and 31 |
| High-bridge deck height for the shadow | 4 height levels (416) | leptons | Claim 32 |
| Motion-file frame of a projectile drawn as a voxel model | 0 | frame | Claim 30 |
| Degeneration | 1 per turn, floor 5 | damage points | Claim 18 |
| Proximity to a unit in the cell | 128 | leptons | T1 |
| Anti-air proximity | 128 | leptons | T1 |
| Ground contact | 208 | leptons above the surface | T1 |
| Arcing building-hit band | 150 | leptons above the terrain | T1 |
| Drop below a deck going down | 20 | leptons | T1 |
| Deep fall | 416 | leptons below the surface | T1 |
| Water tiles of the level-projectile test | 14 | tiles from the theater's water set | T1 |
| Stopped | speed 10, height 10 | leptons per tick, leptons | T1 |
| Snap distances | 32, 42, 128, max(2\|*V*\|, 128); building reduction 64 per cell of width + height | leptons | Claim 20, F9 |
| Reached-row snap divisor | 3 | divisor | F9 |
| Snap-distance vertical weight | 1/2 (of the summed heights, integer division) | factor | F9 |
| Scalable spawn delay terms | 5, 3 | projectiles, divisor | Claim 16 |
| Hard-coded wall overlays | 2, 26, 243 | registration indices (from 0) | T1, S-0307 claim 11 |
| Cliff step | 4 | height levels | Claim 22 |
| Cluster scatter | 256 to 512 | leptons | Claim 21 |
| Airburst children, speed divisor, heading draw | 9, 10, R(0, 32) | projectiles, divisor, draw range | Claim 21 |
| Nuclear warhead name, nuclear fireball animation name | fixed by the original; the names are given in Import mapping | type names (case-insensitive) | Claim 25, Import mapping |
| Shot-distance wrap | 2³¹, modulus 2³² | square leptons (the sum of squares at which the 32-bit sum wraps, an exact planar distance of about 46341 leptons; the modulus of the wrap) | Claim 1, Edge cases |
| Gravity | 3 | leptons per tick² | Absent-key default |
| Ballistic scatter *B* | 256 | leptons | Absent-key default |
| Missile turn variation *m* | 0.25 | factor | Absent-key default |
| Missile safety altitude | 500 | leptons | Absent-key default |
| Projectile type defaults | acceleration 3, spawn delay 3, cluster 1, arm 0, detonation altitude 0, turn rate 0, course lock 0, anti-ground yes, anti-air no | in order: leptons per tick per turn, ticks, detonations, ticks, leptons of world height, turn-rate field units (F7 turns them into facing units), turns, flag, flag | Absent-key defaults |

## State and ordering

**T1. Detonation conditions** (claim 19). *P* is the projectile's position at the start of its
turn and *P*′ its new position; for a steering projectile *P*′ includes the cruise lift of
claim 15 in every row (only the steering distance itself is measured before that lift). Two
heights above the surface are used: *h*, that of *P*, the
height at the start of the turn, which a row uses even when it otherwise tests *P*′; and *h*′,
that of *P*′. A row that says "detonate" makes the projectile detonate in that turn; T2 resolves
what happens when several rows apply. Straight and arcing projectiles share every row marked
"straight, arcing".

| Kind | Condition | Result | Confidence |
|---|---|---|---|
| straight, arcing | *P*′ crosses the bridge deck height of a high-bridge cell (old or new cell) going up or down; or, only for a projectile that has no firer (Terms; quirk `quirks.owned_shell_ground_pass`), *P*′ is below the terrain of its cell, or *P*′ is 0 to 149 leptons above the terrain and its cell holds either an eligible building (not an inactive laser-fence section, not a vehicle-like building; any house's) or, with no building present, one of the three wall overlays of S-0307 claim 11 | detonate (z at the deck height going up, 20 below it going down; otherwise the point ends at the terrain height of the cell of *P*′, lowered to it from above or raised to it by claim 20 a) | medium; high that only the deck crossings detonate a projectile whose firer is still in play outside any transport and has not been killed or started to cloak since the shot, and that the whole row applies to one whose firer is a vehicle destroyed while it flies (P32 runs a and g, P44 parts b and c); medium, `unmeasured`, for another firer killed but kept in play (a death sequence, a delayed kill) or removed by other means: no scenario here has such a firer; low, `unmeasured`, that the whole row applies when the firer started to cloak since the shot and only the deck crossings when it was loaded into a closed transport since the shot, as S-0302 claim 1 rates those two cases (Terms): no scenario here has such a firer |
| straight, arcing | the cell of *P*′ is the cell of *K* and *h* < 208 | detonate; "reached" for claim 20 | high |
| straight, arcing | the cells of *P*′ and *K* hold the same building and *h* < 208 | detonate; "reached" | medium |
| not steering | the infantry, vehicle, aircraft or building nearest to *P*′ in its cell is at *d*₃ < 128 leptons from *P*′ and is neither the firer nor allied with the firer's house (with no firer, any object counts) | detonate; the position moves onto that object unless inaccurate | medium for a projectile made without a firer, a living firer in play outside any transport and a vehicle firer destroyed while it flies (P30); for another lost firer, or a firer loaded into a closed transport, as Terms rates it (medium or low, `unmeasured`) |
| not steering | speed \|*V*\| (after this turn's change of the velocity) < 10 and *h* < 10 | detonate | high |
| vertical | world height of *P*′ above the detonation altitude, or *h* < 0, or *P*′ crosses the bridge deck height of a high-bridge cell (old or new cell; deck height = terrain height of the cell of *P*′ + 416): going up when *P* is below the deck height and *P*′ at or above it, going down when *P* is at or above it and *P*′ below it | detonate; the outcome is *P*′ with no change of height (no move to the deck height, no 20-lepton drop), then claim 20 as always | medium |
| steering | half its speed (\|*V*\| after this turn's heading and pitch changes) ≥ its steering distance (Terms; without a target trunc(\|*V*\|), so only a speed below 1) | detonate; the position moves to the target point when *h* > 0, not airburst and a target exists (its target point is not the world origin, Terms) | medium |
| steering | *h* ≤ 0 | detonate | high |
| steering | no target (or a target point at the world origin, Terms) and *h* ≥ missile safety altitude | detonate | high |
| steering, not airburst or very high | after its first 60 unlocked turns, the closing average (below) lies in 0 … 59 | detonate (orbiting missile) | medium |
| steering, when none of the steering-distance, *h* ≤ 0, safety-altitude and closing-average rows applies | *P*′ strictly crosses the bridge deck height of a high-bridge cell (old or new cell) | detonate; z at the deck height | medium |
| steering or ranged (or both) | past its arm time, and η < 32 where η = trunc(*d*₃(*P*′, *K*) / 2) | detonate (arrival) | high |
| steering or ranged (or both) | past its arm time, η < 256 and larger than the recorded value Ω (below; until the first check past the arm time Ω is the full, unhalved *d*₃ from the projectile's position at placement to *K*) | detonate (passed by; counts as arrival when the firer's type is a jumpjet type) | high |
| any, subject to cliffs or walls | the line-of-fire test of claim 22 for the single sample *P*′ blocks | detonate | high |
| any | *h*′ ≤ −416 | detonate | high |
| flak-scattering with a target | *P*′ below the target's reference height and *h*′ < 0 | detonate | medium |
| level | the cell of *P*′ is not a water tile (not one of the 14 tiles starting at the theater's water set role, S-0010) | detonate | medium |
| anti-air with an airborne target | *d*₃ from *P*′ to the target's reference point < 128 | detonate | high |
| dropping | no longer falling (tested at the start of the turn) | detonate | low |
| not steering | *P*′ outside the legal map (Terms) | removed, no detonation | high |

The recorded value Ω of the passed-by row is set at placement to the full, unhalved spatial distance
*d*₃ from the projectile's position at placement (its start point, claim 8, or for an invisible
projectile the point claim 9 moves it to) to *K*.
Placement itself makes no arrival or passed-by check, so nothing at placement detonates a
projectile by these rows. The checks are made in the projectile's turns only, the first in its
first turn (which falls in the tick of placement, claim 8). Each check past the arm time that
detonates by neither arrival row replaces Ω by that turn's η; before the arm time Ω keeps its
placement value. Observable consequence: in the first check past the arm time, η is compared with
the unhalved placement distance, so with an arm time of 0 a projectile that starts less than 512
leptons from *K* and moves away from it detonates by the passed-by row in its first turn only when
η₁ = trunc(*d*₃(*P*′, *K*) / 2) exceeds that full distance (its distance to *K* has more than
doubled). Otherwise the first turn only records η₁, and the earliest passed-by detonation is in the
second turn (scenario P29 variant d). The closing average: in each unlocked turn *k* (counted
from 1) the closing step is *c*ₖ = *d*₃(*P*, target point) − *d*₃(*P*′, target point) (the world origin when there is no target; quirk `quirks.origin_as_missing_target_point`), and the closing
average is *C*₀ = 0, *C*ₖ = *C*ₖ₋₁ + *c*ₖ for *k* ≤ 60, *C*ₖ = RZ₅₃(RZ₅₃(*C*ₖ₋₁ × *f*₆₀) + *c*ₖ) for
*k* > 60, in double precision, where *f*₆₀ = 8857079267161975 / 2⁵³ (0.98333…) is the double nearest
59/60, multiplied in one step (not × 59 then ÷ 60); the *c*ₖ are integers, so the sums for *k* ≤ 60
are exact. The row tests 0 ≤ *C*ₖ < 60 on that double, for *k* > 60 only. A dropping projectile ignores the arrival and
passed-by rows. A row's "detonate" takes effect at once, except for a projectile with the nuclear
warhead (claim 25), which first waits for an animation. A straight or arcing projectile that has a
firer is not detonated by the terrain, a building or a wall overlay through the first row: below
the terrain it flies on, following its velocity and gravity as above it, and detonates by the
first other row of T1 that applies, with T2 and claim 20 giving the point. For a
flak-scattering projectile with a target (an object or a cell) the flak row is such a row. Where
the target's reference point is not below the surface beneath the projectile, the flak row
detonates it in the first turn in which it is below the terrain, at the surface there (claim 20
a), as the first row would detonate one with no firer: under that condition such a projectile
never flies on below the terrain. The first row's
confidence cell rates this for each cause by which a projectile can lose its firer (Terms). For
example, a "reached" row applies when it enters the cell of *K* with *h* < 208, which includes a
negative *h*, so a shot that passes below the ground short of that cell can still hit there. The
proximity row applies near an object. The deep-fall row applies where it has sunk 416 leptons below
the surface, which can lie short of its impact point as well as beyond it, since gravity also pulls
a straight projectile below its line (claim 10). Claim 20 (a) then raises the point to the
surface. For a straight or arcing projectile with no firer over open terrain (away from high
bridges) the deep-fall row is already covered by the first row (below the terrain), so it shows
there only as that row. The deep-fall row is observable for a steering or vertical projectile,
whose rows otherwise test the height at the start of the turn.

**T2. Outcome of a turn in which rows apply** (claim 19). The outcome of a turn, the position of
the detonation or a removal, depends on which rows of T1 apply in it, one or several. The tables
give no outcome for a turn in which only the dropping row applies (Open questions).

For a projectile that is not steering:

| Rows of T1 that apply in the turn | Outcome |
|---|---|
| one of the two "reached" rows (straight, arcing), whatever other rows apply | detonation at *P*′, with the vertical change of the first row when that row also applies. The proximity row and leaving the map have no effect |
| the proximity row, and no "reached" row | detonation at the other object's position in all three coordinates; for an inaccurate projectile at *P*′ instead, with the vertical change of the first row when that row also applies. Leaving the map has no effect |
| *P*′ outside the legal map, and neither a "reached" row nor the proximity row | removal without detonation, at *P*, whatever other rows apply, a ground hit, a deck crossing or a building hit included |
| none of the three cases above, and any of: the first row, the speed row, a vertical row, the line-of-fire, deep-fall, flak, level or anti-air row, an arrival or passed-by row | detonation at *P*′, with the vertical change of the first row when that row applies (the first row is for straight and arcing projectiles only, so a vertical projectile, a deck crossing included, detonates at its own new height) |

For a steering projectile:

| Rows of T1 that apply in the turn | Outcome |
|---|---|
| the steering-distance row, while *h* > 0, the projectile is not airburst and it has a target | detonation at the target point |
| the bridge-crossing row (by T1 it applies only when none of the steering-distance, *h* ≤ 0, safety-altitude and closing-average rows does) | detonation at *P*′ (after the cruise lift) with z at the deck height |
| any other row or set of rows | detonation at *P*′, after the cruise lift of claim 15 when one applied in that turn |

No row of this spec removes a steering projectile for leaving the map (whether another part of
the game does is an open question).

After the outcome, in every case, claim 20 (a), (b), (c) and then claim 21 apply to the position.

**Ordering.** All rows are evaluated in the projectile's own turn. A projectile fired in tick *n*
takes its first turn in tick *n* (S-0203 claims 4 and 5), unless a removal earlier in the turn pass
costs it that turn, in which case its first turn is in tick *n* + 1 or later (S-0203 claims 5 b
and 10): a fragment of S-0318 that loses its first turns to removals has its first turn at *n* + 2
or later (S-0318 claim 5). So an invisible projectile at a ground target detonates in the tick it is
fired, unless such a removal delays its first turn; scenario P6 has a single firer and nothing else leaving play in
that turn pass. In a turn in which a projectile both creates a trailer animation and detonates, the
animation stands at the projectile's start-of-turn position (claim 16) and the draws of its creation
come before those of the detonation. The draws of a shot come in the order of claim 24.

## Edge cases and quirks

Four kinds of entry follow. A `quirks.<name>` entry names a quirk switch, whose default is the
original. A *departure* follows the glossary. An entry marked *reproduced, the importer warns, no
switch* is unusual data whose effect in the original is well defined and deterministic: our engine
reproduces it and the importer warns (glossary, *importer warning*), and no switch is proposed
because a mod that wants the other behaviour gets it by changing that one value, as each entry
says. The remaining entries (marked *no switch*) note behaviour that our engine reproduces and
that no fix would change.

- **`quirks.railgun_collinear_hits`** (proposed, default on). Claim 27 accepts a non-building
  candidate whose calculated line distance is not a number, including exact collinearity,
  regardless of the hit-distance threshold. Off: for a nonzero beam use the perpendicular
  Euclidean distance to its infinite line, and for a zero-length beam use the Euclidean
  distance to its start, with S-0204 claim 5's table root and double-precision arithmetic.
  Require the resulting finite distance to be strictly below the threshold. Candidate-cell
  sampling, the building distance of 0 and the shot-target bypass remain unchanged.
- **`quirks.uphill_arc_45`** (default on). Arcing shots at a higher impact point are launched at 45°
  whatever F4 would give (claim 6 a). Off: F4 is solved for Δ*z* > 0 as for Δ*z* ≤ 0.
- **`quirks.building_pitch_term`** (default on). Claim 6 c uses 200 × the building's height value
  *H*_b minus the fire point's height for a building target. Off: *Z* = Δ*z* for every target.
- **`quirks.missile_turn_wrap`** (default on). The steering turn step uses only the residue mod 256 of *q* (of *q*\* near the target) and is a
  signed 16-bit value, so a large turn rate can make a missile turn away from its target
  (claim 13). Off: *t* = 256 × min(*q*, 127) (*q*\* near the target), always turning towards the goal.
- **`quirks.aim_origin_without_target`** (default on). A shot fired while the firer has no current
  target is launched towards world position (0, 0, 0) (claim 3); an invisible projectile still lands
  on its shot target, other kinds fly off towards the map's corner. No case is known in skirmish
  play that reaches this (claim 3). Off: the aim point is the shot target's reference point.
- **`quirks.shot_distance_wrap`** (default on). The sum of squares of the shot-speed distance of
  claim 1 is formed in 32-bit integer arithmetic: once the exact sum Δ*x*² + Δ*y*² reaches 2³¹
  (an exact planar distance of about 46341 leptons, some 181 cells), the value used is the sum
  reduced modulo 2³² into −2³¹ … 2³¹ − 1, and the distance is the table square root of its
  magnitude, truncated (claim 1, worked example there). The shot speed is then *v* of that shorter
  distance, so a very long shot is launched too slowly, and a sum that is a multiple of 2³² gives
  the distance 0 and the shot speed 0 (and so `quirks.zero_speed_launch`). Off: the sum is formed
  exactly, as for *d*₂ (Terms). The original's result is well defined and deterministic, so this
  is a quirk and not a departure. Only a weapon of unlimited or very large range on a map more than
  181 cells across can fire so far; no stock situation is known to.
- **`quirks.zero_speed_launch`** (default on). A start speed of 0 launches the projectile at a
  horizontal speed of 100
  leptons per tick in its launch direction, as if the speed had been 100 (F1). Off: the horizontal
  speed stays 0, so the projectile starts without horizontal motion and falls by gravity or
  detonates by the speed row of T1. The start speed is 0 whenever the capped speed of claim 2 is 0
  and the projectile keeps it: a shot speed of 0 (ρ = 0 with a planar shot distance of 0, a
  wrapped shot distance of 0 (claim 1) or a gravity of 0; ρ ≠ 0 with a speed field of 0) or an impact point at most 1 lepton from the fire
  point. Every projectile of a shot keeps it except a steering or vertical one whose firer type has no
  radial fire segments, which starts at 1 instead; so a steering or vertical projectile fired by
  a firer type with radial fire segments meets this case too, for example a steering one whose
  weapon has a speed field of 0.
- **`quirks.redirected_shot_aim`** (default on). A spray-attack or area-fire shot is launched
  towards the firer's own target, while its stored target point and its speed come from the
  redirected cell (claim 3 a). Off: the aim point is the reference point of the redirected cell, so
  the whole launch of claims 3 to 7 works towards that cell.
- **Object number of the steering oscillation (departure, accepted by the project lead on
  2026-10-10).** In the
  original the object number of claim 13 comes from one creation counter shared by everything the
  game creates (objects, types, cells, houses, animations, particles, effects and projectiles),
  counted in order of creation from the start of map loading. Our engine numbers objects with its
  own deterministic creation counter (Terms, *Object number*), so the phase of the 15-tick
  oscillation can differ from the original's. Its period, amplitude and formula are exact. No
  switch, no warning (it is not a data problem).
- **Failed comparison solve in the flatter arc (departure, unspecified result).** When F4 has a
  solution for distance *d* but none for *d* + 1, the sign of the original's launch angle is
  unspecified: nothing that this spec describes determines it. Our engine treats a failed
  *d* + 1 solve as θ′ ≥ θ (launch at θ); fixed, no switch, no warning (it is not a data problem).
  This design-choice departure was accepted under the project lead's delegated sign-off
  (Questions for the project lead).
- **`quirks.zero_distance_arc`** (default on). An arcing shot whose impact point lies exactly
  below the fire point (planar distance 0, Δ*z* < 0) is launched by F4's zero-distance rule
  (claim 6 a): with a start speed above 0 a lob goes straight up and the flatter shot straight
  down or straight up by F4's index test. With a start speed of 0, the usual case for ρ = 0, a lob
  comes out at an angle just short of π (S-0205 claim 7 converts the 0/0 to the table index 0), so
  the projectile leaves horizontally backwards at the 100 leptons per tick of
  `quirks.zero_speed_launch`. The original's flatter shot with a start speed of 0 falls into the
  failed-comparison departure above, and our engine's choice there (launch at θ) gives it the same
  backwards launch. Off: a lob goes straight up (pitch facing 0) and the flatter shot straight
  down (pitch facing 32766), whatever the start speed and the index.
- **Lead with a zero shot speed.** When *w* in claim 3 is 0 (a lead weapon whose projectile has
  ρ ≠ 0 and a speed field of 0, or one with ρ = 0 and a distance *D*₃ of 0 or a gravity of 0) the
  lead's quotient is infinite or not a number, which converts to ℓ = 0 (S-0205 claim 7): the aim
  point is not led, and our engine does the same (F6). A distance *D*₃ of 0 is a game situation,
  not bad data. The importer warns about a weapon with ρ ≠ 0 and speed 0 and about a gravity of 0,
  because of their other effects (the steering entry below, claim 1). No switch: the result is the
  unled aim point, which is also what a fix would give.
- **`quirks.stored_target_point`** (default on). The arrival, pass-by and same-cell rows of T1 and
  the line-of-fire test in flight use *K*, fixed at placement (claim 9), while steering follows the
  moving target (claim 14). Off: *K* is the target's reference point as it stands at the start of
  each turn while the target is in play (and keeps its last value once the target is gone).
- **`quirks.origin_as_missing_target_point`** (default on). The world origin (0, 0, 0) stands for
  "no target" in both directions (Terms, *No target point*):
  - where a rule needs the target point of a projectile that has no target (the near-target test
    of claim 13 and the closing average of T1), the original uses the world origin,
  - a steering projectile whose target's target point is exactly the world origin, which only a
    building whose art aim shift is minus its reference point can have, is treated as having no
    target: it keeps its heading and climbs (claims 14 and 15), the safety-altitude row of T1
    applies to it, and the steering-distance row does not move it to the target point. Claim 20
    is unaffected.

  Off: a projectile without a target never counts as near its target, the closing-average row
  does not apply to it, and a target counts as a target wherever its target point lies.
- **`quirks.vertical_speed_uncapped`** (default on). The speed of a vertical projectile grows by its
  acceleration without a cap, so it can end above its top speed (claim 11, scenario P8). Off: the
  new length is min(trunc(|*V*|) + *α*, *S*_w).
- **Negative turn-rate field.** ρ < 0 makes the projectile fly by its other flags (vertical,
  arcing or straight; Terms, *Projectile kinds*), not as steering, but its shot
  speed is the weapon's speed field and it starts in the firer's aiming facing, and S-0206 treats it
  as homing. Such a shot can be void (claim 6 a) when the speed field is too small for the
  distance. Reproduced, the importer warns, no switch: a mod that wants a vertical, arcing or
  straight projectile sets the turn-rate field to 0 (for a vertical type this also changes its
  top speed from the imported speed field to *v*(*R*₀, *g*), claims 1 and 2), and one that wants a
  steering projectile makes it positive.
- **Negative course lock duration.** With *κ* < 0 the lock never ends: the projectile never turns,
  its closing average never counts, and its acceleration is the normal one. Reproduced, the
  importer warns, no switch: a mod that wants a lock that ends gives a duration of 0 or more.
- **Cluster count ≤ 0.** The projectile leaves play without any detonation, damage or draw of
  claim 21. Reproduced, the importer warns, no switch: a mod that wants one detonation sets the
  cluster count to 1 (its default), which is the only alternative a switch could offer.
- **Steering projectile whose speed reaches 0.** With a top speed of 0 (an imported speed field of
  0 on a weapon with ρ > 0) the speed change of claim 12 brings the velocity to zero length in the
  first turn (from 1, for an acceleration of 2 or more), so the projectile does not move in that
  turn. In the same turn the heading change gives it a horizontal length of 100 (F11) and the
  rebuild keeps that length (claim 15, F12), so from the next turn it flies at about 100 leptons
  per tick less the deceleration of claim 12, and the cycle repeats whenever the speed reaches 0
  again. Reproduced, the importer warns (the warning of the zero-speed lead entry above), no
  switch: a mod that wants a usable missile gives the weapon a positive speed field.
- **Effective spawn delay 0 with a trailer (departure, bad data).** The effective spawn delay of
  claim 16 is 0 only for a type whose spawn delay field is 0, unless a vehicle shot of a scalable
  type has raised it above 0 (a field of 0 with *N* ≥ 8). With an effective spawn delay of 0 the
  original divides by zero; our engine creates no trailer animation, and the importer warns about
  a spawn delay field of 0 on a projectile type with a trailer.
- **Flak range 0 (departure, bad data).** A flak-scattering inaccurate arcing weapon with range 0
  divides by zero in claim 4 (and an invisible flak projectile in claim 9); our engine uses a scatter
  radius of 0, and the importer warns.
- **Detonation altitude 0.** A vertical projectile whose type leaves the detonation altitude at its
  default 0 detonates in its first turn in which its new world height is above 0, so in practice
  in its first turn. Reproduced, the importer warns, no switch: a mod that wants a longer vertical
  flight gives the detonation altitude a positive world height.
- **Arcing flag on a steering or vertical type (bad data).** Such a type is launched by the
  arcing rules at its start speed of claim 2 (1, or the capped shot speed with radial fire segments). F4 with a speed
  that low has no solution in almost every case (it needs the drop to be large against the
  horizontal distance), so the shot is void, except that an impact point higher than the fire
  point gives the 45° launch of claim 6 a. Reproduced, the importer warns about the combination,
  no switch: a mod that wants the vertical or steering flight with a usable launch removes the
  arcing flag.
- **`quirks.owned_shell_ground_pass`** (default on). A straight or arcing projectile that has a
  firer (Terms; T1's first row rates each cause by which a projectile loses its firer, low for
  a firer that started to cloak or was loaded) is not detonated by the terrain, by the building
  band or by the wall overlays of T1's first row. Only that row's deck crossings detonate it. It
  can fly on below the surface until another row of T1 detonates it (the note below T1), with T2
  and claim 20 giving the point, and claim 20 (a) raises a detonation point below the surface. A
  flak-scattering one with a target is detonated by T1's flak row where the target's reference
  point is not below the surface beneath it, in the first turn in which it is below the terrain
  (the note below T1), so the switch does not change where it
  detonates there. An enemy building in its path that does not share a cell with *K* stops it
  only by the proximity row,
  a wall only by the line-of-fire row. Off: the first row applies to every straight or arcing
  projectile as it does to one with no firer, so a shot detonates where it first passes below the
  terrain or enters the band of a building or wall overlay, except that the firer itself and a
  building allied with the firer's house do not stop it.
- **Impact with no firer.** A straight or arcing projectile with no firer detonates in the turn in
  which it passes below the terrain or enters the band of an eligible building or wall overlay
  (T1's first row, which rates each cause by which a projectile loses its firer), at the point of
  claim 20 (raised to the surface by claim 20 (a)). The
  projectile type's elasticity field has no observable effect on it, so it is not imported. No
  switch.
- **`quirks.switched_off_hover_lead`** (default on). A hovering vehicle switched off while it
  remains moving under S-0503 is led as a moving vehicle for as long as S-0503 counts it as moving
  (claim 3 b). Shots of a straight or arcing projectile at it are aimed
  ahead of it along its body facing. An arcing shot at it can be void (claim 6 a). Otherwise T1 and
  T2 determine their detonation, and with `quirks.owned_shell_ground_pass` a shot that passes below
  the ground short of it flies on and can detonate away from it, unless the projectile is
  flak-scattering and has a target (T1's flak row). Attackers that keep it as their
  target can then fail to destroy it for the rest of the game. Off: a vehicle that is switched off
  is not led, so the aim point is its reference point.
- **`quirks.warp_deck_departure_airborne`** (default on). A warp member that departed from a
  high-bridge deck keeps the deck's height but no longer counts as standing on the deck, so the
  rules here that ask whether the target is airborne count it as airborne until it appears, except
  while a landing check that passed for a landing cell on a deck awaits its placement (claim 17).
  At a landing-check kill,
  the projectile loses its target according to claim 17's airborne classification (a steering one
  left without target climbs, claim 15). Off: until it appears, the member counts as standing on
  the deck it departed from, so it is not airborne and its loss gives the deck cell it stood in.

- **A voxel-flagged projectile without voxel art (departure, a fault of the original).** When a
  projectile type's voxel art is absent or discarded (S-0007 claim 18), the original keeps the
  type's voxel flag and faults when it next draws such a projectile on screen. Our engine draws
  nothing for it (claim 28). No switch.
- **Animation counter starts at 0.** An animated projectile shows image frames 0 to the low field
  minus 1 once, before its loop from the low field to the high field starts (F15). A low field
  above the high field leaves the counter at the low field once reached; a rate of 0 holds each
  image frame for 256 turns. Reproduced, no switch: a mod that wants no lead-in uses image frames
  from 0 or sets the low field to 0.
- **Animation fields outside 0 to 255 (bad data).** The low, high and rate fields keep only their
  value modulo 256 (Import mapping), so 256 acts as 0 and −1 as 255. Reproduced, the importer
  warns, no switch.
- **Too few image frames.** A rotating or animated sprite whose image has fewer image frames than
  the number chosen draws neither body nor shadow in the drawn frames that choose a missing image
  frame (claim 28): a rotating projectile with a one-frame image is seen only while it flies in
  the direction of image frame 0. Reproduced, no switch; the data decides.
- **Fog of war.** With fog of war in force, the original does not draw a projectile whose position
  is fogged. Fog of war is not reachable in skirmish (S-0110), so this spec does not specify it.

## Import mapping

| File, section | Key | Concept | Form | Absent |
|---|---|---|---|---|
| rules, weapon | `Speed` | the imported speed field: the shot speed and top speed of a projectile with ρ ≠ 0, and the top speed of a steering projectile; for ρ = 0 it is replaced by *v*(*R*₀, *g*) and read only through *S*_w (claims 1, 2, 11, 12, 21) | integer *v*; stored speed = min(trunc(256 × min(*v*, 100) / 100), 255) leptons per tick; −1 keeps the value in force, any other *v* ≤ 0 gives 0 | 0 |
| rules, weapon | `IsRailgun` | railgun shot flag (claim 27) | boolean | no |
| rules, weapon | `Damage` | unmodified damage of each railgun hit (claim 27) | integer health points | 0 |
| rules, `[CombatDamage]` | `railgundamageradius` | railgun hit distance (claim 27), including its finite-distance and unordered comparisons | signed integer leptons, not a cells-form value, no clamp at 0 | 128 |
| rules, `[CombatDamage]` | `CollapseChance` | railgun cliff-collapse chance (claim 27) | integer percentage points, compared directly with R(0, 99) | 100 |
| rules, weapon | `Lobber` | lob (claim 6 a) | boolean | no |
| rules, weapon | `AreaFire` | area-fire flag (claim 3 a) | boolean | no |
| rules, weapon | `Projectile`, `Warhead` | the projectile and warhead types | type names | — |
| rules, projectile | `ROT` | turn-rate field ρ | integer | 0 |
| rules, projectile | `Arcing`, `Vertical`, `Inviso`, `Inaccurate`, `FlakScatter`, `Dropping`, `Airburst`, `VeryHigh`, `Level`, `Floater`, `SubjectToCliffs`, `SubjectToWalls`, `AA`, `AG`, `Ranged`, `Degenerates` | the flags of Terms (invisible = `Inviso`, anti-air = `AA`, anti-ground = `AG`, ranged = `Ranged`, degenerating = `Degenerates`) | boolean | no, except `AG` yes |
| rules, projectile | `Acceleration` | acceleration *α* (claims 11, 12) | integer, leptons per tick per turn | 3 |
| rules, projectile | `CourseLockDuration` | *κ* (claim 12) | integer, turns | 0 |
| rules, projectile | `Arm` | arm time (claim 9) | integer, ticks | 0 |
| rules, projectile | `DetonationAltitude` | detonation altitude (claim 11) | integer, leptons of world height | 0 |
| rules, projectile | `Cluster` | cluster count *n*_c (claim 21) | integer | 1 |
| rules, projectile | `AirburstWeapon` | the weapon of the airburst children (claim 21) | weapon name | none |
| rules, projectile | `Image` | names the art section that supplies `Trailer`, `SpawnDelay` and the art drawing fields below (claims 16 and 28 to 32; S-0007 claims 26, 27 and 33, S-0004 claim 14) | text, cut to 24 bytes, taken afresh in each layer | empty: those two fields are not read in that layer |
| art, section named by the projectile's current non-empty rules `Image` (no arctic suffix) | `Trailer` | trailer animation (claim 16); the rules key has no effect | creating animation reference, cut to 127 bytes (S-0004); reserved no-type name clears | missing or empty keeps the reference in force; initially none; skipped when the current rules image is empty |
| art, same selected section | `SpawnDelay` | trailer period and source of the zero-delay warning (claim 16, Edge cases); the rules key has no effect | integer ticks (S-0003) | missing or empty keeps the delay in force; initially 3 ticks; skipped when the current rules image is empty |
| rules, projectile | `Scalable` | scalable flag (claim 16) | boolean | no |
| art, section selected as for `Trailer` | `Rotates` | rotating flag (claim 29) | boolean | missing or empty keeps the value in force; initially no; skipped when the current rules image is empty |
| art, section selected as for `Trailer` | `AnimLow`, `AnimHigh`, `AnimRate` | low field *a*_lo, high field *a*_hi and rate *R*ₐ of the animation counter (claim 29, F15) | integer (S-0003), kept as its value modulo 256 (0 to 255) | missing or empty keeps the value in force; initially 0 each; nothing is read when the current rules image is empty |
| art, section selected as for `Trailer` | `AnimPalette` | anim-palette flag (claim 31) | boolean | missing or empty keeps the value in force; initially no; nothing is read when the current rules image is empty |
| rules, projectile | `FirersPalette` | firer's-palette flag (claim 31) | boolean | no |
| rules, projectile | `Shadow` | shadow flag (claim 32) | boolean | yes |
| rules, projectile | `Color` | colour definition of a voxel projectile (claim 30) | colour definition name (S-0103 claim 5), compared without regard to case; selects that definition's 53-shade palette; a name with no 53-shade palette keeps the palette in force | missing or empty re-selects the 53-shade palette of the definition in force; before any read the palette in force is the first definition's single-shade palette (claim 30) |
| none (file name fixed by the original) | `PALETTE.PAL` | the general palette's file (Terms), the same file as S-0111's overlay palette | palette file, read once at program start | — |
| art, projectile image | `Voxel` | drawn as a voxel model (claim 6 b) | boolean | no |
| rules, infantry, vehicle, aircraft or building type | `RadialFireSegments` | *n* (claims 2, 7) | integer | 0 |
| art, building | `TargetCoordOffset` | aim shift (Terms) | integer triple, leptons | 0, 0, 0 |
| art, building | `Height` | building height value *H*_b (claim 6 c; S-0111, claim 7, reads the same field) | integer | 2 |
| none (names fixed by the original) | `NUKE`, `NUKEBALL` | the nuclear warhead name and the nuclear fireball animation name of claim 25 | hard-coded type names, matched without regard to case against the warhead types and animation types of the user's data | — |
| rules, `[AudioVisual]` | `Gravity` | *g* | integer | 3 |
| rules, combat-damage section | `BallisticScatter` | *B* (claim 4) | lepton form of S-0307 claim 1 | 256 |
| rules, `[General]` | `MissileROTVar` | *m* (claim 13) | decimal, double | 0.25 |
| rules, `[General]` | `MissileSafetyAltitude` | claim 17 | integer, leptons | 500 |
| rules, `[WallModel]` | `AlliedWallTransparency` | claim 22 | boolean | no |
| rules, building | `LaserFence` | laser-fence flag for T1's inactive-section exception (S-0211 claim 11) | boolean | no |
| rules, building | `UndeploysInto` | vehicle reference used by the vehicle-like-building test, together with a 1 × 1 foundation (S-0214 Terms and Import mapping) | vehicle type name | none |
| rules, warhead | `Wall` | wall-damaging warhead (S-0307 claim 11) | boolean | no |
| rules, infantry, vehicle, aircraft or building type | `LandTargeting` | land-targeting mode (claim 23: 0 allows land targets, 1 allows none, 2 as 0 here and prefers slot 1 by S-0308; S-0308 owns the key) | integer, used as given | 0 |
| rules, infantry, vehicle, aircraft or building type | `NavalTargeting` | naval-targeting mode (claim 23, water row; S-0308 owns the key) | integer, used as given | 0 |
| rules, warhead | `EMEffect` | electromagnetic flag (claim 20 c; S-0303 F5) | boolean | no |

Except for the projectile image, which is taken afresh as its row specifies, each field keeps
its value in force when a layer omits its key or skips the field's read (S-0006). The combat-damage section is
written by role. The original also reads a projectile
elasticity, which has no effect this spec specifies (Edge cases, Open questions), so it is not
imported here. The shrapnel weapon and count are imported by S-0318. The image's flat flag is S-0102's and its
translucent sprite S-0007's (claim 23).

## Measurement scenarios

All under the glossary's measurement setup, with named test files (loose copies of the rules or
art file changing only the fields named) removed after each run, and a recorder that reads object
positions, projectile positions and velocities, health and the tick number (and the random state
where stated). The recorder reads a projectile's velocity when it enters play, before its first
turn (its *first velocity*), and its position and velocity at the start of each of its turns.
Unless a scenario says otherwise the test file sets gravity to 10, the missile turn variation *m*
to 0.25 and the missile safety altitude to 500 (values chosen for the test, written into the test
file so that no expected value depends on the stock rules), the firer is a stock turreted vehicle pre-placed on flat open ground of height level
0 on cell (*x*, *y*), and targets are unarmed enemy vehicles of a computer player left without
orders. *Flight time* is the number of ticks from the tick the projectile enters play to the tick it
detonates. A straight projectile at gravity 10 sags by about a third of the distance it flies (its
speed is chosen for an arc, claims 1 and 10), so scenarios that need it to reach a point raise the
fire point (S-0313, the fire point's height term in the test file) and check the recording.

**P1. Shot speed and gravity on straight projectiles (claims 1, 2, 10).**
- Setup: two test weapons identical except for their speed field (10 and 100), each with a
  straight projectile that is not inaccurate, not subject to cliffs or walls and without the
  subject-to-elevation property (S-0307 claim 6); target on (*x* + 8, *y*) (planar distance 2048
  from the firer's position; the fire point's own shift is read from the recorder).
- Observable: the first recorded step of the projectile, its height in each tick, its flight time.
- Runs: 1 per weapon (no draw involved).
- Expected: identical recordings for both weapons. With the recorded planar distance *d* from the
  fire point to the target point, the horizontal step is *v*(*d*, 10) of F2 (for
  *d* = 2048: 156), and the height falls by 10, 20, 30 … leptons per tick relative to a level
  path (claim 10). Run c (the cap of claim 2): a third test weapon whose straight, not inaccurate
  projectile has turn-rate field −1 (so the shot speed is the weapon's speed field, claim 1),
  speed field 100 (stored 255 leptons per tick), target on (*x* + 1, *y*): the length of the first
  velocity is min(255, trunc(*d*₃(*F*, *Q*) / 2)) for the recorded fire point and impact point,
  within one lepton (F1), so well below 255. Run d (floater, claims 1 and 10): the first weapon with the floater flag added to its
  projectile, target as in run a: the horizontal step is *v*(*d*, 5) of F2 (the gravity halved to
  5) and the height falls by 5, 10, 15 … leptons per tick relative to a level path.
- Falsified if: the two weapons give different flight times, the step differs from *v*(*d*, 10)
  by more than one lepton, or the height does not fall; in run d, the step is *v*(*d*, 10) or the
  height falls by 10, 20, 30 …; in run c, the first velocity's length exceeds
  trunc(*d*₃(*F*, *Q*) / 2) + 1.

**P2. Lead (claim 3).**
- Setup: target vehicle driving at constant speed *u* (recorded) along a straight road crossing the
  firer's line of fire at right angles, 6 cells away; a second run with an infantry target walking
  the same road; run c: the weapon's straight projectile made anti-air, at an enemy aircraft
  flying at constant speed across the line of fire 6 cells away; run d (*w* = 0): as the first
  run, with the test file setting gravity to 0, so that the shot speed of claim 1 is 0 for the
  firer's lead weapon (its slot 0, the weapon it fires) at every distance. Runs e to g use the
  vehicle target of the first run and two test weapons: *W*₀, whose straight projectile has
  turn-rate field −1 and whose speed field is 10 (stored 25 leptons per tick, so its shot speed is
  25 at every distance, claim 1), and *W*₁, the weapon of the first run (a straight projectile with
  ρ = 0), both with a range of 8 cells set by the test file. Run e (lead weapon other than the weapon
  fired): the firer is a test infantry type with the deploy-fire flag and deploy weapon index 1 (as
  in S-0308 scenario W8 part b), *W*₀ in slot 0 and *W*₁ in slot 1, deployed, so that it fires
  *W*₁ (S-0308 claim 8). Run f (switching type): a test gunner vehicle type (S-0308, Terms) with
  turret count 2, *W*₀ in slot 0 and *W*₁ in slot 1, carrying one infantry passenger whose type's
  mode index is 1, so that its weapon number is 1 and it fires *W*₁ (S-0308 claims 5 and 6). Run g
  (gattling type): a test gattling vehicle type as in S-0308 scenario W5 (turret count 1, weapon
  count 4), with *W*₀ in slot 0 and *W*₁ in slot 2, attacking the target until sustained fire has
  raised its gattling stage to 1, so that it fires *W*₁ from slot 2 (S-0308 claim 7); only the
  shots of *W*₁ are evaluated. Runs h and i (the speed estimate *u*): the vehicle target of the
  first run, a test vehicle type on the driving locomotor whose house's country has its vehicle
  speed factor set to 1 by the test file, not veteran, without a speed bonus, and the recorder also
  reading its speed fraction, its type speed and its position in every tick. Run h: shots
  while it drives straight along the road at its full fraction (1). Run i: shots while it
  accelerates from rest along the road, at recorded fractions from about 0.3 to 0.7.
- Observable: the launch direction (first velocity) and the target's position and body facing in
  the tick of the shot; in runs e to g also which weapon fired (the projectile type recorded); in
  runs h and i also the target's speed fraction in the tick of the shot and its displacement over
  the tick before.
- Runs: 3 per target; 3 shots of *W*₁ each for runs e to g; 5 shots each for runs h and i.
- Expected: the launch direction equals S-0204 F3 from the fire point to the target point moved by
  ℓ of F6 along the body facing (ℓ computed from the recorded *D*₃, *u* and the lead weapon's
  *v*(*D*₃, 10)); for the infantry and the aircraft (run c), to the target point itself. Run d:
  the launch direction points at the vehicle's unled target point (ℓ = 0, F6), and the first
  velocity is horizontal with length 100 (the start speed is 0, `quirks.zero_speed_launch`). In
  runs e to g the shot is *W*₁'s. Run e: the launch direction points at the target point led by ℓ
  of F6 with *w* = 25, *W*₀'s shot speed, not with *W*₁'s *v*(*D*₃, 10). Run f: it points at the
  target point led by ℓ with *w* = *v*(*D*₃, 10) of *W*₁, the slot of the weapon number, not with
  *w* = 25. Run g: as run e, led with *w* = 25 of *W*₀ in slot 0 (the gattling's weapon number
  stays 0). (At about 6 cells *v*(*D*₃, 10) is about 135, so the two leads differ by a factor of
  about 5 and their directions by far more than 8 facing units whenever *u* is a few leptons per
  tick.) Runs h and i: the shot is led by ℓ of F6 with *u* = trunc(type speed × fraction), the integer of
  S-0207 F2 for the recorded type speed and the fraction held in the tick of the shot (the country
  factor 1, no bonus, no faster ability), not with the recorded displacement of the tick before
  (which, being measured in leptons along the track, generally differs from that integer).
- Falsified if: the vehicle shot points at the unled target point (more than 8 facing units from
  the led direction), or the infantry or aircraft shot is led; in run d, the launch direction is
  more than 8 facing units from the direction to the unled target point; in run e or g, it is
  more than 8 facing units from the direction led with 25; in run f, it is more than 8 facing
  units from the direction led with *W*₁'s *v*(*D*₃, 10); in run h or i, it is more than 8 facing
  units from the direction led with *u* = trunc(type speed × fraction) (the named alternative being
  the lead computed from the recorded displacement of the tick before). (The other lead of each run, and the
  unled direction, are the alternatives these falsifiers rule out.) A run in which *W*₁ never
  fires does not count.

**P3. Scatter (claims 4, 9, 24).**
- Setup: an arcing, inaccurate test weapon firing at a fixed cell 14 cells away (a ground attack
  on the cell) on level ground, its range set to cover that distance, and ballistic scatter
  set to 512. Its test projectile has a turn-rate field of 0 (ρ = 0), so its shot speed is claim
  1's, and is not a floater. The test file sets the gravity to a local test value. The fixture
  keeps every scattered impact point reachable: for every impact point within 512 leptons of the
  cell's point, F4 has a solution with the start speed of claim 2 for that point. Without claim
  2's cap the speed of claim 1 alone gives a level reach of about 1.2 *d*, which at 14 cells
  passes *d* + 512 (0.2 × 3584 is above 512); the test gravity must also keep the cap from
  binding below that reach. The observer confirms the condition from the recorded planar
  distance *d*, the test gravity and claims 1 and 2 before a run counts. Variants b, d and e use
  the same distance.
- Observable: each projectile's impact point (Terms), which the recorder reads directly when the
  projectile enters play, as it reads its first velocity. The detonation point is not used for
  it, since the detonation point of a projectile with a firer need not lie at the impact point
  (T1's note). Also the detonation
  points of the invisible projectiles of (d) and (e), and the random-stream draws per shot with
  the recorder. If the recorder cannot read the impact point, runs a, b, e and f give no result
  for their distances. The recorder also logs each fire attempt and whether it creates a
  projectile, as in P44 part a. A run of a, b or e counts only if the fixture condition is
  confirmed and every fire attempt creates a projectile; otherwise it is void.
- Runs: 200 shots; variant b with the flak-scatter flag; variant d with an invisible
  flak-scattering projectile that is straight and not inaccurate (so claim 4 does not apply to
  it); variant e with the same projectile made arcing and inaccurate as well.
- Expected: (a) every horizontal distance from the aim point to the impact point between 256 and
  512 (±1); the directions show no preferred sector (chi-square over 8 sectors, *p* > 0.01).
  In the main run also exactly two draws per shot, the radius draw and the direction draw of
  claim 4, before the delay draw of S-0301 claim 18. (b) distances between 0 and 512 × *D* /
  *R*_W. (d) two draws (the radius and direction of claim 9), the invisible projectile's detonation point
  within 1024 × *D* / *R*₀ of the target's reference point. (e) four draws, in the order of claim
  24: the claim 4 radius and direction (the impact point at 256 to 512 from the aim point, which
  changes only the launch direction), then the claim 9 radius and direction; the detonation point as in (d).
- Falsified if: any distance lies outside its band, the chi-square test of the main run rejects
  evenly spread directions (*p* ≤ 0.01), or the draw count differs (in particular two draws in
  (e), or four in (d)).
- Variant f (open-topped reduction, claim 4 and S-0307 claim 13): the flak-scattering weapon of
  variant b with a range of 10 cells, fired by a stock vehicle that the test file makes an
  open-topped transport with that weapon, carrying one infantry passenger whose type's primary
  weapon has a range of 7 cells; the target 6 cells away (the range test uses the transport's own
  range, S-0307 claim 13). The passenger fires from the transport too, so only projectiles of the
  transport weapon's projectile type (recorded, as in P2) count, each by its recorded impact
  point. Runs: 200 shots of the transport weapon, and a control of 200 shots without the
  passenger.
  Expected: distances between 0 and 512 × *D* / *R*_W with *R*_W = 7 cells (1792 leptons), some
  of them above 512 × *D* / 2560, the bound of a 10-cell range; the control stays within
  512 × *D* / 2560. Here the scatter radius can pass 0.2 *D*, so an outward impact point can lie
  beyond the reach of claim 1 and F2 and its attempt is void (claim 6 a): only created
  projectiles are judged, and the share above 512 × *D* / 2560 is not predicted. Falsified if no
  distance with the passenger exceeds 512 × *D* / 2560 (+1), or one exceeds 512 × *D* / 1792
  (+1). This ordinary passenger uses slot 0 as its current
  weapon, so this run does not distinguish current-weapon selection from a primary-only rule.
  The distinguishing multi-weapon and missing-weapon fixtures remain pending in S-0307 RG16.

**P4. Direction and elevation of straight shots (claims 5, 6 b–c, 7).**
- Setup: straight test projectile (not drawn as a voxel model), targets 5 cells away on level
  ground (a), on ground 1 level lower (b), 2 levels lower (c) and 3 levels higher (d); a variant e
  with a projectile drawn as a voxel model, at a higher and at a lower target.
- Observable: first velocity vector, the recorded fire point and impact point (so the recorded
  Δ*z*), the recorded planar distance *d*′.
- Runs: 1 per target.
- Expected: horizontal direction = S-0204 F3 from the fire point to the impact point. The pitch is
  read from the recorded Δ*z*: when |Δ*z*| ≤ 200 the pitch facing is exactly 16383 and *V*_z = 0; when
  |Δ*z*| > 200 it is F13's pitch from the signed angle
  sgn(Δ*z*) × *A*(|Δ*z*| − 20, max(*d*′, 0.05)), including S-0204 F6's correction. Which
  case applies to each target follows from the recording (a fire point a few leptons above the
  terrain puts a one-level drop under 200 and a two-level drop over it, but the fire point's height
  is read, not assumed). Variant e (drawn as a voxel model): pitch facing 16384 for the higher
  target, which gives exactly the velocity of a level launch (*V*_z = 0, Terms), and 32768 for the
  lower, which gives a velocity pointing straight down (horizontal components below 10⁻¹³ in size,
  *V*_z about −*s*).
- Falsified if: a velocity component differs from F1 with these pitches by more than 10⁻⁶, a shot
  with |Δ*z*| > 200 starts level, or one with |Δ*z*| ≤ 200 does not.
- Variant f (building pitch term, claim 6 c, `quirks.building_pitch_term`): the firer stands on
  ground 4 levels high and fires the same straight projectile at an enemy stock building on
  level-0 ground 5 cells away, so that the recorded |Δ*z*| is about 416 plus the fire point's
  height, well over 200, while *Z* below stays small; the building's art height value *H*_b is
  read from the user's art data. Expected: the pitch facing is
  F13's pitch from sgn(*Z*) × *A*(|*Z*| − *τ*, max(*d*′, 0.05)), including S-0204 F6's
  correction, with *Z* = 200 × *H*_b − the recorded world height of the fire point
  (*τ* = 20, or 0 when |*Z*| < 20). The run counts only when this pitch differs from the one the
  recorded Δ*z* would give by more than 50 facing units. Falsified if the pitch is the one the
  recorded Δ*z* gives.
- Variant g (launch in the aiming facing for ρ ≠ 0, claim 5): an arcing, inaccurate test
  projectile with turn-rate field −1 and a weapon speed field of 100 (stored 255 leptons per tick,
  so the shot speed is 255 capped by claim 2), the test file setting the ballistic scatter to 512,
  fired at a stationary vehicle 5 cells away on level ground. Observable: the first velocity, the
  firer's aiming facing (S-0310) in the tick of the shot and the recorded impact point. Runs: 20
  shots; only shots whose direction from the fire point to the impact point differs from the
  aiming facing by more than 16 facing units count. Expected: the horizontal direction of the first
  velocity (S-0204 F3 of its components) equals the aiming facing within 8 facing units. Falsified
  if it lies within 8 facing units of the direction from the fire point to the impact point
  instead.

- Variant h (F13 exception-index control): use S-0204 C4's two straight-shot setups, one for
  each sign of aim height, alongside P4. Record the first velocity before gravity acts and compare
  F1 at pitch 417 versus 416 for the positive case, and 32349 versus 32350 for the negative case.
  The narrowed table arguments must select index 1024. Expected: 417 and 32349 respectively.
  Matching the uncorrected candidates falsifies F13's correction. C4 supplies the setup,
  qualification, one valid run per sign and void rules. No arctangent correction is applied
  to the arcsine-based ballistic conversion of F4.

**P5. Arcing launch (claim 6 a).**
- Setup: arcing test weapon without the lobber flag; targets at 5 cells on level ground, on ground
  2 levels lower, on ground 2 levels higher; a variant with the lobber flag. Run d (void shot): a
  test arcing projectile with turn-rate field −1 (so the shot speed is the weapon's speed field,
  claim 1), the weapon's speed field set to 10 (stored 25 leptons per tick), not inaccurate, target on
  level ground 5 cells away; a second run d with the inaccurate flag. Run e (height-based lob): the weapon has no lobber flag, the test file
  raises the fire point 500 leptons above the firer, and the target stands on ground 3 levels higher
  in the next cell (planar distance 256, height above the firer 312); its control run puts the same
  target on that ground 2 cells away (planar distance 512). Run f (arcing flag on a steering type,
  Edge cases): a test steering projectile (turn rate 5) that also has the arcing flag, fired by a
  firer type without radial fire segments at a target on level ground 5 cells away, and again at a
  target on ground 2 levels higher 5 cells away.
- Observable: first velocity; for run d, whether a projectile appears and the firer's rearm
  countdown.
- Runs: 1 each.
- Expected: the launch angle, determined from the recorded vertical velocity and horizontal speed, equals F4's flatter angle on level ground and lower ground, exactly 45° (pitch facing 8191, within 0.01°) for the higher target, and F4's steeper angle with the lobber flag; run d: no projectile and no rearm (a void shot: F4 has no solution because *v*⁴ is far below *g*²*d*²), and no draw; the inaccurate run d: the same, with the two scatter draws of claim 4 recorded before the void shot, and run e: F4's steeper angle (the target is higher than it is far, though lower than the fire point), the control run F4's flatter angle.
  Run f: no projectile and no rearm at the level target (F4 with the start speed 1 has no
  solution), and a projectile launched at pitch facing 8191 at the higher target.
- Falsified if: the higher target's angle differs from 45° by more than 0.1°, the level shot is
  steep, run d launches a projectile, run e and its control give the same angle, or in run f a
  projectile is launched at the level target or the higher target's shot is void or not at 45°.

**P6. Invisible projectiles (claim 9).**
- Setup: test weapon with an invisible straight projectile; run b with a wall cell between firer
  and target and the projectile subject to walls (the range test passes because the warhead has the
  wall flag). One firer, and no other object enters or leaves play in that tick's turn pass.
- Observable: the tick of the target's health loss, the detonation point.
- Runs: 1 each.
- Expected: (a) health falls in the tick of the shot; (b) detonation at the wall cell's centre.
- Falsified if: (a) the loss comes a tick later, or (b) the target is hit.

**P7. Reaching the target cell (T1).**
- Setup: straight test weapon fired at an empty cell 2 cells away (ground attack); the test file
  raises the fire point to 150 to 300 leptons above the terrain, so that the projectile is still above
  the terrain when it enters the cell (the recording shows this).
- Observable: detonation tick and point, and the start-of-turn height in the detonating turn.
- Runs: 1.
- Expected: detonation in the first turn whose new position lies in the target cell, with a
  start-of-turn height below 208. The detonation point is given by claim 20: a cell target can
  snap too, so use the recorded velocity and F9 to predict whether it stays at the new position
  or moves to the cell's reference point.
- Falsified if: it flies through the cell, detonates by the reached-cell row with a start-of-turn
  height of 208 or more, or its detonation point differs from claim 20's predicted point.

**P8. Vertical projectiles (claims 1, 2, 5, 6, 11).**
- Setup: a large flat open map, so that a level shot flies at least 60 cells before it leaves the
  legal map; test vertical projectile with turn-rate field 0, not arcing, not drawn as a voxel
  model, acceleration 50 (a value chosen for the test, so that a shortfall of one lepton per tick in
  the table root of F10 cannot hide the overshoot), detonation altitude 5000 (a world height it never reaches in runs a
  and c), fired by a weapon of range 6 cells (1536 leptons) with gravity 10, so that the top speed
  is *T* = *v*(1536, 10) of F2, which is 135 leptons per tick; the weapon's imported speed field is
  20 (stored 51 leptons per tick). Run a: force-fire at an empty cell 5 cells away on level ground
  (no object near the line; the recorded |Δ*z*| from the fire point to the cell is at most 200, so
  claim 6 (c) gives a level launch). Run b: as run a with the detonation altitude absent. Run c: as
  run a with the imported speed field 60 (stored 153 leptons per tick). Run d (climbing): an
  anti-air vertical projectile (anti-air flag on), acceleration 3, detonation altitude 300, at a
  hovering enemy aircraft 6 cells away whose height above the fire point is at least 400
  (recorded); the weapon range is 6 cells. Run e (into the ground): as run a with acceleration 3,
  fired at an empty cell 5 cells away on ground 3 levels lower, so that the recorded |Δ*z*| exceeds
  200 and claim 6 (c) pitches the projectile down.
- Observable: velocity (speed, heading, pitch) and world height per tick, detonation tick and
  point, or removal. The speed is the length computed exactly from the recorded components.
- Runs: 1 each.
- Expected: (a) the speed is about 1 at placement (the start speed of claim 2); in each later turn
  the new length is trunc(*r*(…)) of the previous recorded velocity's squared length (S-0204 F1,
  computed from the recording) plus 50, as long as that truncated value is below 135, and after
  that the velocity no longer changes. Because the table root and the roundings of F10 can make a
  truncated value one less than the length the previous turn aimed at, the speeds are about 1,
  then 50 or 51, then 100 or 101, then 150 or 151, and the constant speed is one of these last
  values: the first term at or above *T*, not *T* (135) and not the stored speed field (51); heading and pitch never change and
  the projectile stays level (pitch facing 16383, no gravity), so it flies on at a constant height
  and is removed without a detonation when it leaves the legal map; (b) detonation in its first
  turn; (c) the same recording as run a; (d) the first velocity has the pitch of claim 6 (c) for
  the recorded Δ*z* (|Δ*z*| > 200), the heading and the pitch stay at their launch values for the
  whole flight (no turn towards the aircraft), the height rises in a straight line, and the
  detonation comes in the first turn in which the new world height is above 300, before the
  projectile is within 128 leptons of the aircraft; (e) the projectile flies down its launch line
  and detonates in the first turn whose start-of-turn height above the surface is negative (the
  turn after the first turn that ends below the surface), unless the deep-fall row (*h*′ ≤ −416)
  acts first, which the recording shows.
- Falsified if: in run e it is still flying in a turn that starts below the surface, or in run a the constant speed is below 140 (the stored speed field 51 as the top
  speed gives a constant of 51, or 101 after a rounding just below 51; a cap at *T* gives 135), in
  run c the speed rises above 160 (the
  stored speed field 153 as the top speed), or the speed keeps rising; run c differs from run a; run b flies; in run a the projectile climbs or
  turns; in run d it starts level, its pitch changes in flight, or it does not detonate by the
  altitude.

**P9. Steering (claims 12–15).**
- Setup: steering test projectile (turn rate 5, acceleration 3, speed field 25, stored as 64
  leptons per tick), course lock duration 0, fired at a vehicle 15 cells away; run b with speed
  field 10 (25 leptons per tick, below 40); run c with turn rate 1 (*q* = 1, so *t* = 256: direct mode),
  run d with course lock duration 10, the speed field 25 again, the projectile drawn as a voxel
  model (launch pitch facing 16384, which gives a level velocity, claim 6 b) and the target on ground 4 levels higher, 10 cells
  away. Run e (wrap of claim 13): turn
  rate 200, speed field 25, a firer type with 1 radial fire segment whose body faces a stationary
  target 10 cells away (the shot then leaves a quarter turn clockwise of the body facing, claim 7,
  so the first signed facing difference is about 16384). Run f (near-target factor): turn rate
  20, speed field 25, the same firer and a stationary target 8 cells away, arm field 1000.
  Run g (no target): as run a, but with an anti-air projectile and a hovering aircraft target
  destroyed by another weapon when the projectile is half way, while still airborne (as in P11
  run a). A ground target would leave a cell target instead and cannot test this case. The
  missile safety altitude is the 500 of the test file. Run h (launch pitch): as run a with the target on
  ground 4 levels higher, 5 cells away. Run i (length kept by the rebuild, claim 15): turn rate 5,
  acceleration 3, speed field 0 (top speed 0), course lock duration 0, a stationary target vehicle
  on ground 4 levels higher, 2 cells away (so direct mode by distance, and the goal pitch is well
  above level). Run j (cruise mode of a negative step, claim 15): the test file sets the missile
  turn variation *m* to 0; turn rate 150, speed field 25, a stationary target vehicle on flat
  ground 10 cells away; with *m* = 0 every turn has *q* = 150, so *t* = 256 × 150 − 65536 =
  −27136. Runs k to o (as run a, one change each): k, the very-high flag on the projectile; l, the
  airburst flag, with an airburst weapon whose projectile is harmless to the target; m, the level
  flag, with a stock naval vehicle as the firer and an enemy naval vehicle 15 cells away as the
  target, all over water (over land the level row of T1 would detonate it at once); n, the
  projectile made anti-air and the target an enemy aircraft hovering 15 cells away at a recorded
  height; o, the
  flight path passes over a high bridge, along the bridge, at least 8 cells from the target. Run p
  (negative course lock): as run a with course lock duration −1. Run q (the lifted position, claim
  15 and T1): as run a, not subject to cliffs or walls, from low ground at a target on a plateau 9
  height levels higher (10 if no turn qualifies), so that a projectile cruising at about 5 height
  levels meets the edge some 416 leptons below the plateau surface, whose cliff edge lies across the flight path more than 6 cells from the
  target, repeated with speed fields 20 to 40 so that turns end at many points past the edge. A
  *qualifying turn* is one whose unlifted new position *P* + trunc(*V*) (the recorded
  start-of-turn position plus trunc of the velocity after the turn's speed change and before its
  heading and pitch changes, claim 14; at top speed in practice the velocity recorded at the start
  of that turn, as in (g)) lies over the plateau at a
  height above the surface from −433 to −416 leptons, in which the cruise lift of claim 15 raises it
  by 18 and whose start-of-turn height is positive. The runs continue until at least 3 turns
  qualify.
- Observable: the firer's position and the fire point in the tick of the shot and the
  projectile's first position; speed, horizontal heading, pitch and height per tick; the recorded spatial distance
  *D* to the target; the position at the start of each turn.
- Runs: 1 each (the turn phase *j* is read from the 15-tick oscillation of the heading change).
- Expected: (a) the projectile's first position is the recorded fire point (S-0313), not the
  firer's position, compared in *x* and *y*, and in height too when the fire point is above the
  terrain (claim 8); speed 1, 4, 7 … to 64; turning from the first turn; heading change per tick at
  most
  *t* of F7; while the planar distance to the target exceeds 3 cells the height stays within 52
  leptons of the cruising altitude of claim 15, *E* + *n* × 104 with *n* = min(trunc(*D* / 256), 5),
  so 5 height levels above the terrain of the cell ahead from 5 cells (spatial) on, and fewer
  levels (4, then 3) as the distance falls to 3 cells planar. (b) speed +1 on even ticks only, no
  heading change until |*V*| + 0.5 ≥ 25, and the first heading change in the first turn whose
  start-of-turn speed |*V*| satisfies |*V*| + 0.5 ≥ 25, that turn itself (the pitch can already
  turn, claim 12). (c) no cruising: the pitch turns straight at the target. (d)
  speed 1, 4, 7 … as in (a) from the first turn, no heading change in the first 9 turns and the
  first heading change in turn 10; in those 9 turns the pitch facing falls by 128 per turn (the
  goal pitch lies above the launch pitch; claims 12 and 15). (e) with *m* = 0.25 and ρ = 200, *q* runs from 200 to 300; in
  each tick whose phase *j* gives *q* mod 256 ≥ 128 (8 of 15 phases: *j* = 0 and 8 to 14) and in
  which the recorded facing difference is larger than |*t*| = 65536 − 256 × (*q* mod 256) facing
  units, the heading turns *away* from the target by |*t*|; in the other ticks it turns towards
  it by *t* or snaps. (f) the heading change per tick is 256 × (*q* mod 256) for the *q*
  of the tick's phase while the recorded spatial distance from the start-of-turn position to the
  target point is 256 or more, and 256 × (trunc(1.5 *q*) mod 256) while it is below 256; at least
  three recorded turns lie in the second band. (g) in the first turn without a target the position
  change of the turn equals trunc of the velocity recorded at the start of the next turn (the
  turned velocity) and its pitch turns towards 8192 by *t*; in a turn with a target the position
  change equals trunc of the velocity recorded at the start of that turn (after the speed change
  of the turn; at the top speed of 64 that change is below 0.01 leptons per tick, so in practice
  this is trunc of the recorded velocity itself). The projectile detonates when its
  height above the surface reaches 500. (h) the first recorded velocity (length 1) has the pitch
  facing of claim 6 (c) for the recorded Δ*z* and is not level. (i) in the first turn the speed
  falls from 1 to 0 and the position does not change; the velocity recorded at the start of the
  second turn has a horizontal length of about 100 × cos_t(θ(*p*′)) and an upward vertical
  component of about 100 × sin_t(θ(*p*′)), between 7 and 10 leptons per tick, *p*′ being one
  direct-mode pitch step of 768 to 1024 facing units above level (*q* from 5 to 7); in the second
  turn the projectile moves about 99 leptons. (j) in its first turn the projectile is in cruise
  mode: far below its cruising altitude (*e* < −52) its goal pitch is 8192, the cruise step
  trunc(*t*/2) = −13568 has a magnitude above the 8191 units to that goal, so the pitch snaps to
  8192 (45° up), and it climbs at 45° until it is within 52 leptons of the cruising altitude of 5
  height levels above the terrain ahead; its heading stays on the target (a snap by the same
  argument). (k) the height holds within 52 leptons of 10 height levels (1040 leptons) above the
  terrain of the cell ahead while the planar distance exceeds 6 cells, and the pitch turns
  straight at the target (direct mode) once it is at most 6 cells. (l) the height holds within 52
  leptons of 10 height levels above the terrain ahead at every distance, including within 3
  cells (an airburst projectile is never in direct mode by distance). (m) the pitch facing stays
  at its launch value and the height changes only by the launch pitch's slope (no cruise lift of
  18 leptons, no climb to a cruising altitude). (n) no cruising altitude: from the first turn the
  pitch turns towards the aircraft by trunc((*t* + 256)/2) per turn (direct mode for an aircraft
  target at any distance). (o) while the cell 6 velocity steps ahead holds the high bridge the
  height holds within 52 leptons of the bridge cell's terrain height + 416 + the cruise height of
  run a (5 × 104 leptons), so the projectile rises by about 416 leptons before the bridge and
  comes down after it.
  (p) the heading never changes, the speed grows by the acceleration 3 each turn (not by 1 in
  even ticks), and the closing-average row never detonates it. (q) in every qualifying turn (run q)
  the projectile does not detonate in that turn; it detonates in the next turn (by the *h* ≤ 0 row,
  its start-of-turn height then being negative).
- Falsified if: (a) the first position is the firer's position rather than the fire point (in
  the same comparison), a
  heading change exceeds *t*, or the cruising altitude differs from the stated
  one by more than 52 leptons, (b) its heading changes during the lock, or the first heading change comes one turn after the
  first turn whose start-of-turn |*V*| + 0.5 ≥ 25, (c) it cruises, (d) it
  turns in turn 9 or not yet in turn 10, or its pitch is constant during the lock, (e) the heading always turns towards the target, (f) the
  near-target band shows no 1.5 factor, (g) the no-target move uses the unturned velocity or the
  target move the turned one, (h) it starts level, (i) the vertical component recorded at the
  start of the second turn is below 1 leptons per tick in magnitude (the length *M* taken before
  the heading change, 0, or before the speed change, 1) or the projectile does not move in the
  second turn, (j) its pitch in the first turns stays within 2000 facing units of the line to
  the target (direct mode for a negative step), (k) or (l) the cruise height is 5 height levels,
  (k) it cruises within 6 cells, (l) it turns straight at the target within 3 cells, (m) its pitch
  changes or it is lifted by 18 leptons, (n) it cruises, (o) the height ignores the bridge,
  (p) the heading turns or the speed grows by 1 on even ticks only, or (q) it detonates in a
  qualifying turn (as it would if the deep-fall row tested the position before the lift), or a
  turn whose lifted height above the surface is −416 or less ends without a detonation.

- Exception-index control (F12, current and goal pitches): run S-0204 C4's steering variants
  alongside P9, using only read-only recording. At index 1024 a positive signed elevation angle
  must give pitch 417, not 416; a negative one must give 32349, not 32350. Record the velocity
  before and after each qualifying turn and compare the full F12 rebuild, including the current
  pitch and, in a snapping turn, the goal pitch. Matching the uncorrected velocity instead
  falsifies the correction. C4 supplies the setup, qualification, run counts and void rules.

**P10. Trailers (claim 16).**
- Setup: test steering projectile (no gravity, so it flies long) with a trailer animation and
  spawn delay 5, speed field 10 (25 leptons per tick), at a vehicle 12 cells away, so that it flies
  at least 30 ticks. In this scenario the trailer animation and the spawn delay are set in a loose
  art copy, in an art section whose name the projectile's rules section gives as its image value
  (claim 16); every variant does the same unless it says otherwise.
- Observable: the ticks in which trailer animations appear.
- Runs: 1.
- Expected: exactly the ticks of flight whose tick number is a multiple of 5.
- Variant b (scalable): the same projectile type with the scalable flag, fired by 8 test vehicles
  in the same tick so that 8 such projectiles are in play (*N* = 8 for the last shot): trailer ticks
  are multiples of 5 + trunc(3 / 3) = 6 for every projectile of the type.
- Variant c (position and draws; claim 16 and Ordering): as the first run, with the projectile's
  position recorded at the start of each turn and the random-state recorder on; the trailer
  animation is a test animation type whose random rate pair is descending (S-0109 claim 3), so its
  creation takes one draw; the projectile has cluster 2 and the target stands so that the
  detonating turn's tick number is a multiple of 5 (chosen from a first run). Expected: each
  trailer animation stands at the projectile's start-of-turn position of its tick, not at the new
  position; in the detonating turn the trailer's draw comes before the draws of the detonation
  (for each of the two cluster detonations, S-0303's draws followed by the ranged draw R(256, 512)
  and the raw value of claim 21, so claim 21 takes two such pairs in that turn).
- Variant d (removals do not lower the delay; claim 16): after variant b, once all 8 projectiles
  have left play, a test infantry unit given the same weapon fires one projectile of the scalable
  type (a shot by a firer that is not a vehicle does not change the delay); then, once that one
  has left play, one of the test vehicles fires a single projectile of the type with no other
  projectile of a scalable type in play. Expected: the infantry's projectile still has trailer
  ticks at multiples of 6 (the delay that the vehicle shots of variant b set, although none of
  their projectiles is in play); the last vehicle shot has *N* = 1, so its trailer ticks are
  multiples of 5 + trunc(max(1 − 5, 0) / 3) = 5.
- Variant e (a raised value of 0 falls back to the field; claim 16): as variant b, with the type's
  spawn delay field set to −1. The raised value is −1 + trunc(max(*N* − 5, 0) / 3), which is −1
  for *N* up to 7 and 0 for the eighth shot (*N* = 8), so the type uses its field, −1, throughout.
  Every tick number is a multiple of −1. Expected: every projectile of the type creates a trailer
  animation in every turn, before and after the eighth shot.
- Variant f (file and section of the two fields; claim 16): three runs of the first run's setup,
  with the art section named by the image value holding the trailer animation *X* and spawn delay
  5. Run f1: the projectile's rules section also gives a different trailer animation *Y* and spawn
  delay 7. Run f2: no image value in the rules section (and none in any earlier layer for this test
  type), while an art section named after the type itself holds *X* and spawn delay 5. Run f3: as
  the first run, with the art spawn delay changed to 7 and a rules spawn delay of 5 added.
  Expected: f1, animations of *X* only, in ticks that are multiples of 5; f2, no trailer animation
  at all; f3, trailer ticks at multiples of 7.
- Falsified if: the pattern starts from the launch tick instead, or variant b keeps a period of 5,
  or in variant c an animation stands at the end-of-turn position, or the detonation's draws come
  before the trailer's; in variant d, the infantry's projectile has a period of 5 (removals
  lowered the delay, or the infantry shot reset it), or the last vehicle shot keeps a period of
  6; in variant e, the projectiles stop creating trailer animations after the eighth shot (the
  value 0 used as the delay); in variant f, *Y* appears or the period is 7 in f1, any trailer
  appears in f2, or the period is 5 in f3.

- Variant g (Q-0160, zero art delay): an isolated run of the first setup, not scalable, with
  the selected art section's delay 0 and a non-empty trailer reference, while rules gives delay 5.
  Record whether the first projectile turn completes. Expected in the original: a divide-by-zero
  failure at that turn, not trailer ticks at multiples of 5. On our engine: the importer warns,
  the projectile flies normally, and creates no trailer. Control: art delay 5, ignored rules delay
  0, with the same trailer. Expected: normal trailer ticks at multiples of 5 and no zero-delay
  warning on our engine. One run each; remove test files afterwards. A completed original turn
  in the zero-art-delay run, or a failure in the control, falsifies the art-source failure rule.

**P11. Target loss (claim 17).**
- Setup: run a: a steering projectile (top speed 25 leptons per tick) at a hovering aircraft that is
  removed from the game mid-flight while still airborne (the recorder gives its state at removal,
  and the run counts only if it was airborne in the sense of S-0307), the missile safety altitude
  set to 500 by the test file; run c: the same projectile at a vehicle on the ground that another
  weapon destroys mid-flight; run b: an arcing projectile with a long flight (10 cells) at a
  vehicle that is destroyed while it flies; run d: the steering projectile of run c at a stock
  vehicle that the test file gives a teleporting locomotor (as S-0215's chrono-move scenario
  does), ordered to move 10 cells while the projectile flies, so that it teleports away; run e: the steering projectile of run c fired at a stationary vehicle, and the
  firer destroyed by another weapon while the projectile flies.
- Observable: the projectile's height, position and detonation.
- Runs: 1 each.
- Expected: (a) it climbs and detonates when its height above the surface reaches 500; (c) it does
  not climb to 500: its target is the cell the vehicle stood in, so it steers to that cell and
  detonates there (by the steering-distance row of T1, at the target point); (b) it detonates in
  the target's former cell; (d) once the vehicle has teleported, the projectile steers to the cell
  the vehicle left and detonates there, not at the vehicle's new place; (e) the projectile keeps
  flying after the firer is gone and detonates at the target as in a control run with the firer
  alive, the target losing health.
- Falsified if: (a) it dives to the ground, (c) it climbs and detonates at height 500, (b) it
  flies on, (d) it follows the vehicle to its new place, or (e) it vanishes or stops when the
  firer dies.
- Run f (claim 17, a departed warp member as target):
  - Setup: a test steering projectile slow enough, at a firer far enough from a vehicle V on open
    level ground, to be still in flight when V departs; its warhead damages V, and its type's arm
    time is longer than its whole flight, so that neither arrival row of T1 applies. V has no
    weapon. The testing house
    has a warp superweapon ready (S-0513, as its scenario SW12 runs one). The landing point is a
    level cell several cells from V's cell and off the firer's line to V, with nothing in it, so
    that the landing check passes at its first try.
  - Procedure: order the firer to attack V. Once the projectile is in flight, launch the warp with
    V as its only member and the landing cell as destination. Nothing else attacks or moves V.
  - Observable: the projectile's position in each turn, its detonation point and tick, V's health
    in each tick, and the ticks of V's departure and appearance (S-0513 claim 39).
  - Expected: after V appears, the projectile turns towards V at its landing point and detonates
    there, and V loses health.
  - Falsified if: any other outcome, for example the projectile steers to the cell V left and
    detonates there (its target became that cell, as in run d), it climbs and detonates at the
    missile safety altitude (it was left without target), or it detonates at the landing point
    with V's health unchanged.
  - Void if: the projectile detonates or leaves the map before V appears; the landing check does
    not pass at its first try or kills an object; the firer makes a second projectile before the
    first detonates; V is moved, attacked or damaged by anything else.
- Run g (claim 17, special hits on a member off the map):
  - Setup: as run f, except that the projectile's arm time is short and the test file gives its
    warhead one special-hit flag per part, the others cleared. Part a: takeover, from a firer
    that can take over. Part b: bomb planting, from an infantry firer. Part c: electric charging.
    Part d: temporal erasure, from a firer that holds a temporal beam (S-0514 claim 1). Part e:
    air-strike designation, from a firer that can designate. Part f: bomb disarming, with a bomb
    planted on V by another unit before the warp. Part g: disguise-making, from a vehicle firer,
    with V disguised as another vehicle type before the warp. A control run of each part without
    the warp gives the tick *t*_A in which the projectile detonates on V. The warp is then
    launched in tick *t*_A − 62, so that V departs in that tick (S-0513 claim 39). V entered play
    before the projectile, so in each tick V's update comes before the projectile's turn (S-0203
    claim 3): the projectile's turn in *t*_A falls after V's departure and before its appearance.
  - Procedure: order the firer to attack V, launch the warp in tick *t*_A − 62 with V as its only
    member, and let V land.
  - Observable: V's departure and appearance ticks, the detonation tick and point, V's owner,
    bomb, erasure state and health in each tick, the designation, the firer's disguise, and
    the ticks of V's appearance and arrival effects in part d.
  - Expected: the detonation is at V's old position while V is off the map. Part a: V's owner is
    the firer's house from the detonation on. Part b: V carries a bomb from the detonation on.
    Part c: nothing changes, and V's health is unchanged. Part d: V is being erased from the
    detonation on. Part e: V is designated. Part f: V's bomb is gone from the detonation on.
    Part g: the firer takes V's disguise. In every part V's health is unchanged by the hit.
  - Falsified if: the effect of a part does not happen, happens only once V appears, or V loses
    health from the hit. Part d's appearance and arrival-effect ticks are recorded for the Open
    questions, and no outcome of them falsifies a claim here.
  - Void if: the recorder does not show V's departure before the detonation and, except in part
    d, V's appearance after it; the landing check does not pass at its first try; anything else
    attacks V.
- Run h (claim 17, special hits on a member phasing in): as run g, except that the warp is
  launched so that the detonation falls after V's appearance and before its arrival hold ends.
  A part with a locomotor-seizing warhead, from a firer that can seize V, is added as part h, and
  is also run with run g's timing. Expected: the detonation is on V at its landing point, and
  parts a to g give their effects as in run g. Part h, run in the original: its result, in both
  timings, is recorded for the Open questions, and no outcome falsifies a claim here. Part h, run
  in our engine: V is not seized, and it appears and has its arrival effects on run g's ticks.
  A seizure, or a later appearance, falsifies claim 17's design rule. Void if: the detonation is
  not inside that span.
- Run i (claim 17, the deck count after a refused placement; our engine only, since the original
  has no refused placement after a passed check, S-0513 claim 50):
  - Setup: as run f, with `quirks.warp_deck_departure_airborne` on, V on a high-bridge deck, its
    landing cell on another deck, and an anti-air test projectile whose type cannot hit a target
    that is not airborne. The engine test forces a placement refusal after the first passed
    check, as S-0513 scenario SW24 part c forces one in our engine, and then forces the outcome
    of the next landing check: in part a it fails, in part b it redirects V to a cell not on a
    deck, in part c it passes for a landing cell not on a deck, and in part d it passes for a
    landing cell on a deck. In parts a to c no later check passes for a cell on a deck before V
    appears.
  - Procedure: order the firer to attack V and, once the projectile is in flight, launch the warp
    with V as its only member. The projectile's speed and the launch distance are chosen so that
    it is still in flight from the refusal until at least two ticks after the forced check.
  - Observable: in each tick, whether the projectile's rows treat V as airborne, and the check
    results, redirects and placement of V.
  - Expected: V counts as standing on a deck only in the ticks between a check passed for a deck
    cell and that check's placement or refusal. In every part V counts as airborne from the
    refusal until the forced check. In parts a, b and c it still counts as airborne after the
    forced check (counted by its height after a failed check, a redirect, and a pass for a cell
    not on a deck). In part d it counts as on a deck from the forced check until its placement.
  - Falsified if: V counts as on a deck in any other tick, as airborne between a deck pass and
    its placement or refusal, or as on a deck after the failed check of part a, the redirect of
    part b or the pass of part c.
  - Void if: the projectile detonates before the second tick after the forced check, or the
    forced outcome does not occur.
- Run j (claim 17, the threshold of claim 20 c for a member that departed from a deck):
  - Setup: V on a high-bridge deck, `quirks.warp_deck_departure_airborne` on (the default), its
    landing cell on level ground several cells away. A straight test projectile with the ranged
    flag and a short arm time flies level at the deck's height, so its height above the surface
    over the deck's cells is at least 208 and no "reached" row of T1 applies. Its speed and the
    launch distance are chosen so that its positions in two turns straddle V's reference point,
    the last one before it at least 64 leptons from it (so the arrival row does not apply) and
    the first one after it farther from it: the passed-by row of T1 then detonates it there. A
    control run in which V has already appeared at its landing point before the projectile
    passes gives the tick *t*_P of that detonation and the distance from its point to V's old
    reference point, which must lie between 64 and 128 leptons.
  - Procedure: as run g, with the warp launched in tick *t*_P − 62.
  - Observable: V's departure and appearance ticks, the detonation tick and point.
  - Expected: the detonation point is the projectile's own position, not V's old target point,
    because the threshold is 42.
  - Falsified if: the point is moved onto V's old target point (the threshold 128).
  - Void if: the detonation is not in tick *t*_P or not by the passed-by row, V is not off the
    map then, or the distance falls outside 64 to 128 leptons.

**P12. Degeneration (claim 18).**
- Setup: straight test projectile, damage field 20, degenerates flag, warhead 100 % against the
  target's armour, target with known health; flight time *N* recorded.
- Observable: health lost.
- Runs: 1.
- Expected: max(5, 20 − *N*) (one point lost in each of the *N* turns without detonation) times the S-0302
  factors.
- Falsified if: the full 20 lands, or the loss differs from max(5, 20 − *N*) times the S-0302
  factors by more than the rounding of S-0302.

**P13. Detonation point (claim 20).**
- Setup: run a: straight test projectile at a stationary vehicle 2 cells away, the fire point raised
  as in P7; run b: the same with the inaccurate flag.
- Observable: the detonation point versus the vehicle's reference point, the projectile's position
  and start-of-turn velocity (so *ν*′ of F9 is computed from the recording).
- Runs: 10 each.
- Expected: (a) the detonation point is the vehicle's reference point when the recorded *ν*′ is at
  most max(2|*V*|, 128), else the projectile's own position; (b) always the projectile's own
  position.
- Falsified if: an accurate detonation lands off the vehicle's reference point while *ν*′ is within
  the threshold, or on it while it is beyond the threshold, or an inaccurate one is moved.
- Variant c (claim 20 c): an accurate straight projectile at a building whose art gives a non-zero
  aim shift (for any other target the target point is the reference point and the flag cannot
  show), once with a warhead that has the electromagnetic flag and once without. Runs: 20 of each,
  the projectile's recorded distance from the building's reference point at detonation falling
  into the bands below.
- Expected in variant c: where that distance is under 32 leptons the detonation point is the target
  point without the flag and the reference point with it; where it is at least 32 but the distance
  reduced by 64 per cell of the sum of foundation width and height is under 42, it is the target
  point without the flag and the projectile's own position with it. (A detonation through the snap of
  claim 20 b lies on the reference point, so it falls in the first band.)
- Falsified in variant c when the flag makes no difference in either band, or when a detonation
  lands on another point than the one its band and flag give.

**P14. Cluster and airburst (claim 21).**
- Setup: test projectiles with cluster 3, cluster 0, and an airburst projectile with an airburst
  weapon.
- Observable: explosion animations, health of nearby targets, projectiles created, and with the
  random-state recorder the draws taken when each airburst child is made.
- Runs: 3 each (cluster points are random).
- Expected: cluster 3: three explosions, the second 256 to 512 leptons from the first and the third
  256 to 512 from the second (±1; the same point again when a nearby point would leave the map),
  cluster 0: no explosion and no damage; airburst: nine new projectiles, one above each of the eight
  neighbouring cells and its own cell, each falling straight down at trunc(*S*_w / 10) of the
  airburst weapon's recorded speed *S*_w (so the speed does not depend on that weapon's imported
  speed field when its projectile has a turn-rate field of 0); each child takes exactly one draw
  R(0, 32) when it is made (nine such draws in all).
- Falsified if: cluster 0 explodes, the third explosion lies 256 to 512 leptons from the first
  instead of the second, or the airburst gives another count, another speed or other draws.
- Variant d (the draws of claim 21, random-state recorder):
  - Setup: one firer, a test vehicle, and no other unit on a small flat map. The firer
    force-fires one shot at an empty cell 10 cells away. The test weapon has range 12 cells,
    burst 1 and a rate-of-fire field long enough that no second shot falls in the ticks recorded.
    Its projectile is arcing and not inaccurate (so it takes no launch draw, claim 24, and reaches
    the target cell), not invisible, has no trailer and is not airburst. Its test warhead has a
    spread of 1 cell, no electromagnetic flag, no debris, no particle system and no special
    effect, and an animation list of one test animation type that takes no draw: no random rate
    (or both ends equal), no random loop delay, no follow-up type, no crater or scorch effect, no
    damage and no infantry it creates (S-0109 claims 3, 11 and 15 to 18). The target cell and
    every cell within 8 cells of it hold no object, overlay, wall, resource or bridge. By S-0303
    (its order of a projectile's detonation) such a detonation takes no draw of its own, so in
    the detonating turn only claim 21 draws. Four runs with the same seed, differing only in the
    cluster count: 0, 1, 2 and 3.
  - Observable: the stream state and the raw values taken in each tick (the recorder), the
    detonation tick, the position of each explosion animation, the firer's position.
  - Runs: 1 per cluster count (the recorder makes the draws exact, so no statistics are needed).
  - Validity, read from the recording: the four runs show the same stream state at the start of
    the detonation tick (the cluster count has no earlier effect); in each run the stream takes
    no raw value in the tick before and the tick after the detonation tick (nothing else on the
    map draws around then); and every explosion lies more than 4 cells from the firer, so no area
    damage reaches it.
  - Expected: in the detonation tick the cluster-0 run takes no raw value. The cluster-1 run takes
    one ranged draw R(256, 512), that is *k* raw values where *k* is the number that the range rule
    of S-0202 (claim 4) consumes from the recorded stream at that point, and then exactly one more
    raw value: *k* + 1 in all, at least 2. The cluster-3 run takes three such groups in a row, and
    its second and third explosions lie exactly at N(*Q*, *λ*, snap off) of S-0312 F2 for the
    preceding explosion point *Q*, with *λ* the result of the group's ranged draw and *b* the
    lowest 8 bits of the group's raw value, both read from the recorder; the third group is taken
    after the third explosion although no fourth explosion follows. The cluster-2 run takes two
    groups, the second after its last explosion.
  - Falsified if: the cluster-0 run takes any raw value in that tick; the cluster-1 run takes no
    raw value, or a count other than *k* + 1; the cluster-3 run takes only two groups (none after
    the last detonation) or its explosions do not lie at the points the recorded groups give; or
    the cluster-2 run takes other than two groups.

**P15. Cliffs (claim 22).**
- Setup: straight test weapon whose projectile is subject to cliffs; firer and target on two plains
  of level 0 separated by a ridge 4 levels high (run a) or 3 levels high (run b).
- Observable: fires or moves (S-0307 RG observable).
- Runs: 1 each.
- Expected: (a) moves (blocked), (b) fires.
- Falsified if: (a) fires or (b) moves.

**P16. Eligibility (claim 23).**
- Setup: (a) test weapon whose projectile has the anti-ground flag off, force-fired at an enemy
  ground vehicle in range; (b) weapon without anti-air at a jumping jumpjet infantry; (c) without
  and with anti-air at a tunnelling unit while it digs, with a control at a ground unit at the same
  distance; (d) the same weapon as in (a) force-fired at
  a ground cell, and a control with the anti-ground flag on; (e) a weapon without anti-air at a
  hovering enemy aircraft in range, with a control weapon with anti-air.
- Observable: whether a shot is fired.
- Runs: 1 each.
- Expected: (a) fired; (b) refused; (c) refused without anti-air; with anti-air the shot is fired, as at
  the control (so it is not refused for that reason); (d) refused with the flag off, fired with it on,
  (e) refused without anti-air, fired by the control.
- Falsified if: (a) is refused, (b) fires, (c) fires without anti-air or is refused with it while
  the control fires, (d) fires with the flag off, or (e) fires without anti-air.
- Part f (targeting modes, claim 23's water and land rows): (1) a test vehicle type whose only
  weapon's projectile is anti-air and not anti-ground, in two copies that differ only in
  land-targeting mode (0 and 1), each force-fired at an enemy ground vehicle on clear land in
  range, then at the same vehicle on a bridge, then at an empty clear cell; (2) the same two
  copies given an attack on that vehicle, on clear land and then on the bridge, by a map trigger
  or a team script; (3) the test vehicle type of (1)
  with land-targeting mode 0, in three copies with naval-targeting modes 0, 2 and 6 (the same
  anti-air-only weapon, so that the water row is seen not to depend on the weapon), force-fired
  at an enemy floor-bound (S-0307, Terms) hovering vehicle whose type is not underwater, on a water cell, then on a beach cell, then on a bridge over
  water; (4) a test vehicle type whose only weapon's projectile is anti-ground and not anti-air,
  in two copies with land-targeting modes 0 and 1, each force-fired at an empty clear cell in
  range. Every copy in parts f to h is a type defined in the named test file, which sets each
  copy's land-targeting and naval-targeting modes and nothing beyond what each part names, as in part g and
  S-0309 scenario A5 part b. If a floor-bound hovering enemy vehicle cannot be made to stand on
  one of (3)'s water, beach or bridge cells, the record says so, and that cell's sub-part is
  void; the other cells are still judged.
  Observable: whether a shot is fired within 300 ticks, 3 runs each. Expected: (1) and (2) mode 0
  fires at the vehicle, on land and on the bridge, and in (1) is refused at the empty cell (third
  row), mode 1 fires at none; (3) mode 0 fires everywhere; modes 2 and 6 are refused on the
  water and beach cells and fire on the bridge; (4) mode 0 fires at the cell and mode 1 is
  refused (the land row). Falsified if mode 0 in (1) or (2) does not fire at the
  vehicle (the firing check then tests the anti-ground flag for object targets after all), if
  mode 1 fires, or if the results of (3) depart from that pattern. (4) is void if its mode-0
  copy does not fire at the cell, since another rule then refuses it.
- Part g (the land row's refusal stops retaliation): two copies of the test vehicle type of part
  f (1), land-targeting modes 1 and 0, each of a computer house, in *guard* on clear land, with
  no aircraft about. Their weapon uses a test warhead with every versus value at 100%, so that
  S-0304 claim 3 cannot refuse retaliation (as in S-0309 scenario A5 part b). The copies meet
  S-0309 claim 18's other conditions: the test file sets their type's retaliate flag and a sight
  range at least 2 cells above their weapon range, and each copy is armed, holds no target and
  has not fired before. An enemy ground vehicle whose weapon outranges theirs hits each from a
  cell beyond the copy's weapon range but within its sight limit (S + 0.5 cells, S-0215 claim
  18).
  Observable: whether the copy moves toward the attacker within 300
  ticks of the first hit, 3 runs each. Retaliation does not restart the copy's processing timer
  (S-0215 claim 10), so its first processing in *attack* can wait until the *guard* processing
  delay then running ends. Before the runs, check in the behaviour data (S-0215 Import mapping)
  that the upper bound of that delay (S-0215 claim 6) is below 300 ticks; if it is not, the
  deadline is that bound plus 300 ticks. The copy has not fired before the first hit, so no
  rearm countdown adds to it. Expected: the mode-0 copy retaliates and approaches (S-0309
  claim 18; its firing check without range has no refusal); the mode-1 copy stays where it is.
  Falsified if the mode-1 copy approaches: its land-row refusal is then not one that stops
  retaliation. If the mode-0 copy does not approach, the run is void: this part judges whether
  the land row stops retaliation, and a mode-0 copy that does not approach means some other
  condition stopped it, so the comparison shows nothing about the land row. Whether a mode-0
  copy fires back at all is judged by part f and by S-0309 scenario A5 part b, where a mode-0
  copy that does not fire falsifies the absence of the anti-ground test. If every mode-0 run is
  void, part g backs nothing, and the land row's stopping retaliation is `unmeasured` until a
  fixture makes the mode-0 copy approach.
- Part h (the land row's answer is *illegal*): the two copies of part f (1), owned by the local
  player. Select each in turn and, without force-fire, hold the cursor over an enemy ground
  vehicle on clear land, once within the copy's weapon range and once beyond it. Observable:
  whether the cursor offers an attack. Expected: the mode-1 copy is offered no attack at either
  distance (S-0214 claim 6(e) withholds it for *illegal*); the mode-0 copy is offered the attack
  at both. Falsified if the mode-1 copy is offered an attack: the answer is then not *illegal*.
  Falsified as well if the mode-0 copy is offered no attack: the firing check would then refuse a
  ground object to an anti-air-only weapon whatever the land-targeting mode, against the absence
  of the anti-ground test on object targets. That reading relies on no other row of S-0214
  withholding the attack: the copy is not berserk, melee-linked or being erased (rows G1 and
  G2), it is not an in-place deployer and passes the deploy check (rows V3 and V5), and the
  target's type is not immune to damage (row V6). S-0214 scenario T4 part C (e), as now written,
  predicts no attack for a unit with such a weapon; it is amended below (Changes for other
  specs). Water variant (the water row's answer is *illegal*): the copies of part f (3) with
  naval-targeting modes 0 and 6, owned by the local player. Select each in turn and, without
  force-fire, hold the cursor over an enemy floor-bound hovering vehicle on a water cell within the copy's
  weapon range. Expected: the mode-6 copy is offered no attack; the mode-0 copy is offered it.
  Falsified if the mode-6 copy is offered an attack (the answer is then not *illegal*). Void for
  the water row if the mode-0 copy is offered none: another rule then withholds it. The mode-0
  copy is only a control here; the water row's mode-0 branch is not tested by this variant. The same conditions on S-0214's other rows apply as above.
- Part i (a held target that lifts off, claim 23's first row): an enemy aircraft of a type with
  the landing behaviour stands landed on its pad. A test vehicle and a test building, each with a
  weapon whose projectile is anti-ground and not anti-air, and neither able to grip a unit, are
  ordered to attack it in range. As soon as the target recorder shows it as their target, the
  aircraft's owner orders it to fly to a cell within their range and to stay in the air there. A
  second enemy ground vehicle stands within both firers' range throughout. Observable, with the
  target recorder: the firers' targets from the order until 300 ticks after lift-off, and their
  shots. Expected: once the aircraft is airborne, neither firer shoots at it, and each drops it as
  its target no later than the first check of its attack after lift-off, which answers *illegal*
  (the first row's rank 4 comes ahead of *rearming*; S-0304 claim 2, S-0215 claim 32). An earlier
  drop agrees with this part; the part does not judge the tick of the drop. Falsified if either
  firer keeps the aircraft as its target for the 300 ticks while it stays airborne in range, or
  shoots at it while it is airborne.

**P17. Redirected shots (claim 3 a).**
- Setup: run a: a test rules file gives a stock vehicle the spray-attack flag (S-0313 claim 9) and a
  weapon with a straight, not invisible, not inaccurate projectile and burst 1; an enemy target 5
  cells away along the map x axis. Run b: the same weapon, without the spray-attack flag, with the
  area-fire flag.
- Observable: the first velocity of each projectile, the cell where it detonates, the recorded
  distance from the fire point to the firer's own cell centre (run b).
- Runs: 10 shots (the spray pick is random).
- Expected: (a) every launch direction is S-0204 F3 from the fire point towards the target vehicle
  (not towards the spray cell), and the projectile detonates in the spray cell only if its path
  crosses it, otherwise when another row of T1 applies (for example the deep-fall row below the
  ground). (b)
  the launch direction points at the target vehicle, while the first step is *v*(*d*, 10) of the
  distance *d* to the firer's own cell (capped by claim 2), not of the distance to the target.
- Falsified if: the launch direction points at the spray cell or the firer's cell, or in run b the
  first step is that of the distance to the target.

**P18. Level projectiles and pre-move heights (T1).**
- Setup: run a: a test weapon with a level, straight projectile fired from a stock naval
  vehicle across water towards a target on the shore; run b: a straight projectile fired downhill
  from a cliff top at a cell target 6 cells away on lower ground.
- Observable: detonation tick and point.
- Runs: 1 each.
- Expected: (a) detonation in the first turn whose new cell is not a water tile; (b) in the turn in
  which the new position is in the target cell, provided the height above the surface at the start
  of that turn is below 208; when it is not, the projectile flies on and detonates by another row
  (the deep-fall row once below the terrain, or speed and height).
- Falsified if: (a) it flies over land, or (b) it detonates in the target cell while its height at
  the start of that turn was 208 or more.

**P19. The nuclear warhead (claim 25).**
- Setup: a test rules file with a straight projectile that is not inaccurate and a warhead
  registered under the nuclear warhead name (run a), the same weapon with a warhead of another name and the same
  fields (run b), and run c: the warhead of run b named as the nuclear-missile superweapon's warhead
  in the game-wide special-weapons setting. A unit target on flat ground 2 cells away (fire point
  raised as in P7), and the animation type registered under the nuclear fireball animation name left as it is in the
  stock data.
- Observable: the tick the projectile reaches the target's cell, the tick of the target's health
  loss, the tick the nuke flash starts (S-0104), the position and the life of the nuclear fireball
  animation (S-0109), the radar events of the target's owner and of the firer's owner (tick,
  cell and kind, from the recorder or the radar pane), and the projectile's position in between.
- Runs: 1 each.
- Expected: (a) the nuke flash and the animation start on the arrival tick, the projectile sits
  still until the animation ends, and the health loss comes in the first turn of the projectile
  after that, so the delay equals the animation's length in ticks, give or take the turn order of
  that tick; (b) and (c) health falls on arrival, with no flash and no animation. In run a a radar
  event appears at the projectile's cell on the arrival tick; its kind is the one the interface
  specs give for it, and run b shows no event of that kind.
- Falsified if: (a) loses health on arrival, or (b) or (c) waits for an animation or starts a flash,
  or run a shows no radar event at the arrival cell and tick, or run b shows one.
- Run d (comparison without regard to case): as run a, with every occurrence of the nuclear
  warhead name in the loose rules copy (its entry in the warhead list, its section name and every
  reference to it) re-spelt in lower case, so that the warhead type is first registered, and so
  stored, in lower case (S-0004 claim 7). Expected: as run a. Falsified if it behaves as run b
  (health falls on arrival, no flash, no animation).
- Run e (absent fireball-animation type): use run a with a named loose rules copy that removes
  the fireball type from the animation list and removes every reference that would register it
  indirectly (S-0004). The recorder must confirm no type is registered under that name before
  the shot; otherwise the run is void. Record one shot's arrival, flash, radar event, animation
  creation and health loss. Expected: the flash and radar event still occur on arrival, no
  fireball animation is created, and health falls in that same projectile turn with no waiting
  interval. A created fireball or a postponed health loss in a valid run falsifies the rule.

**P20. Radial fire segments (claims 2, 7).**
- Setup: the test file gives a stock turreted vehicle 4 radial fire segments and a weapon with a
  straight, not inaccurate projectile; a target 5 cells ahead; 8 shots in a row. Variant b: the
  projectile is vertical (turn-rate field 0), once with and once without radial segments. Variant
  c: the projectile is steering, with radial segments.
- Observable: first velocity of each projectile, the firer's body facing at each shot.
- Runs: 1 each.
- Expected: the horizontal direction of shot *k* (*k* = 0, 1, 2, 3, 0, …) is the body facing turned
  by π *k* / 4 − π/2 anticlockwise (claim 7), that is the body facing *b* plus 16384, 8192, 0 and
  −8192 facing units for *k* = 0 to 3 (within a unit or two of the rounding of F1), at speed *s*; (b) with segments the first speed is *s* (not 1), without it is
  1; (c) the first speed after placement is 1 whatever the segments (claim 9).
- Falsified if: all shots go in one direction, the direction of shot *k* differs from *b* plus
  16384, 8192, 0 or −8192 facing units (for *k* = 0 to 3) by more than 8 facing units (in
  particular a mirrored sequence, *b* − 16384 first, falsifies the sense of the turn), a vertical projectile with segments
  starts at 1, one without segments starts at a speed other than 1, or (c) the steering
  projectile's first speed after placement is not 1.

**P21. Dropping projectiles (claims 5, 8; T1).**
- Setup: a test aircraft with a weapon whose projectile has the dropping flag, not inaccurate,
  and turn-rate field 5 (so neither aircraft velocity exception of claim 7 applies),
  flying over an enemy ground vehicle; the test file gives the weapon a lateral fire-point shift of
  100 leptons (S-0313) so that the fire point differs from the aircraft's position.
- Observable: the firer's position and aiming facing and the projectile's first position and first
  horizontal direction; the tick of detonation and the projectile's falling state.
- Runs: 3.
- Expected: the first position is the firer's position (not the fire point), the first direction
  its aiming facing; the projectile detonates in the first turn in which it is no longer falling
  (low confidence).
- Falsified if: the projectile starts at the fire point, or starts in another direction than the
  firer's aiming facing (more than 8 facing units off), or (low confidence) it detonates in a turn
  that starts with the recorder still showing it falling, or flies on in a turn that starts with
  it no longer falling.

**P22. Arm time and arrival (claim 9; T1 arrival rows).**
- Setup: a test steering projectile with a very low speed (speed field 4, stored 10 leptons per
  tick), turn rate 8, at a stationary vehicle 3 cells away; run a: arm field 0; run b: arm field
  1000; run c: arm field 1000 and an airborne target that hovers (the arm time of an aircraft target
  is 0); run d (`quirks.stored_target_point`): speed field 10 (25 leptons per tick), arm field 0, a
  target vehicle 3 cells away driving straight away from the firer along the line of fire at a
  recorded speed of at least 6 leptons per tick from before the shot, so that the projectile's
  path passes over the point where the target stood at placement.
- Observable: detonation tick, position, and the recorded spatial distance *d*₃ from the projectile
  to the stored target point in each turn.
- Runs: 1 each.
- Expected: (a) detonation in the first turn in which trunc(*d*₃ / 2) < 32, at the target's reference
  point (arrival, claim 20 b); (b) no arrival detonation: it flies on and detonates about 5 to 6
  turns later by the steering-distance row (half its speed, 5, at least the steering distance); (c)
  as (a); (d) the projectile detonates by the arrival row or the passed-by row against the stored
  point *K* (where the target stood at placement), in the turn the recorded half distance η to *K*
  first falls below 32 or first grows again below 256, while the recorded distance from the
  projectile to the moving target is still at least 128 leptons; by the arrival row it lands on the target's reference point (claim 20 b), by the
  passed-by row at its own position.
- Falsified if: (b) detonates at the tick of (a), (a) later than that turn, (c) differs from (a),
  or (d) the projectile flies past *K* (η growing again below 256) without detonating and reaches
  the moving target.

**P23. Anti-air proximity (T1).**
- Setup: a test weapon with an anti-air straight projectile and a stationary, hovering enemy
  aircraft within range, no other object near the projectile's path.
- Observable: detonation tick, position, and the recorded spatial distance to the aircraft.
- Runs: 1.
- Expected: detonation in the first turn in which *d*₃ from the new position to the aircraft's
  reference point is below 128 (strict); the warhead lacks the electromagnetic flag, so claim 20
  (c) moves the detonation point onto the aircraft's target point (its reference point; the
  aircraft is airborne in a render layer other than the ground layer and the distance is below
  128).
- Falsified if: the projectile passes within 128 leptons without detonating, detonates farther,
  or detonates at its own position instead of the aircraft's reference point.

**P24. Deep fall (T1).**
- Setup: a test steering projectile that is not subject to cliffs, speed field 25 (64 leptons per
  tick), fired along a flat plain at a target on a plateau 6 levels high (624 leptons) whose foot
  is 3 cells away.
- Observable: detonation tick, position, and the recorded heights *h* and *h*′ in each turn.
- Runs: 1.
- Expected: detonation in the first turn in which *h*′ ≤ −416 (the new position far inside the
  rising ground), with *h* above 0 at the start of that turn, so one turn before the *h* ≤ 0 row
  would act; the detonation point is at the surface there, the new position raised to height 0
  (claim 20 a).
- Falsified if: the projectile detonates only in a later turn although a turn with *h*′ ≤ −416 is
  recorded.
- Variant b (the *h* ≤ 0 row): the plateau 2 levels high (208 leptons), so that no single step of
  64 leptons per tick can end 416 below the surface; the run counts only when the recording shows
  a turn ending below the surface. Expected: detonation in the first turn whose start-of-turn
  height *h* is at most 0. Falsified if the projectile flies on in such a turn.

**P25. Line of fire in flight (claim 22, T1).**
- Setup: a test weapon with a straight projectile subject to cliffs (the fire point raised as in P7) and
  a firer on a plain; a ridge of 4 levels within 3 cells of the firer beside a road; a target vehicle drives along the road's edge so that the line from the
  fire point to its reference point at the moment of fire passes beside the ridge (the range test
  passes, S-0307) while the led aim point of claim 3 lies behind it, so that the flight crosses ridge
  cells. A control run with a projectile not subject to cliffs.
- Observable: the projectile's cell per turn, the detonation tick and position, the target's health.
- Runs: 3 each.
- Expected: the projectile subject to cliffs detonates in the turn in which its new position enters
  a ridge cell (the cell's elevation level at least 4 above that of the cell it was in and above
  that of the fire point's cell), without hitting the target; the control flies on.
- Falsified if: it enters the ridge without detonating.

**P26. Leaving the map (claim 19, T2).**
- Setup: an arcing, inaccurate test weapon, the test file setting the ballistic scatter to 512; the firer 6 cells inside the
  map edge, the target standing in the last legal row on that edge (so that part of the scatter
  annulus lies outside the legal map).
- Observable: for each of 100 shots, the explosion animation (count and position), the target's
  damage, whether a projectile remains, and the projectile's position and height above the
  surface at the start of each of its turns (the recorder).
- Runs: 100 shots.
- Expected: the outcome of each shot is the one that T1 and T2 give for its recorded positions
  and heights: a shot that leaves the legal map in a turn in which neither a "reached" row nor the
  proximity row applies, and that has not detonated before, produces no explosion and no damage
  and leaves no projectile (removal at its position of the start of that turn); every other shot
  detonates where T2 and claim 20 place it. The share of removed shots is recorded; it is not
  predicted from the scatter alone, since a shot whose impact point lies outside the map can be
  detonated before it leaves the map (T2), and one whose impact point lies inside can still leave
  it below the ground (T1's note).
- Falsified if: a shot whose recorded flight T1 and T2 make a removal explodes or does damage, or
  one that they make a detonation is removed without one.

**P27. A target destroyed before the shot (S-0309 claim 27; claim 3's aim without a target).**
- Setup: a straight, not inaccurate test projectile (not invisible) fired by a turreted vehicle at a
  vehicle 6 cells away; the target is destroyed by a second, instant weapon in the same tick in
  which the turreted vehicle's rearm countdown has run out (the tick is chosen from the recording
  of a first run). The second weapon's firer belongs to the turreted vehicle's house, and no other
  object of a house that is not its ally is on the map. A control run with the target alive.
- Observable: the firer's fire events, its current target in each turn, and the first velocity of
  any shot it fires in that tick.
- Runs: 3 each.
- Expected: when the destruction comes earlier in the tick than the firer's turn, the firer holds
  no target in its turn and fires nothing in that tick (S-0309 claim 27); when its turn comes first,
  it fires at the live target, its launch direction towards that target (the control: the same).
  This part therefore does not reach claim 3's aim without a target, which stays `unmeasured`.
- Falsified if: the firer fires in its turn after the destruction in that tick.

**P28. Zero shot speed (claim 7, `quirks.zero_speed_launch`).**
- Setup: a weapon with the area-fire flag and a straight, not inaccurate projectile with ρ = 0, a
  firer standing on the centre of its cell, the test file setting the weapon's forward and lateral
  fire-point shifts to 0 (S-0313), so that the planar distance from the fire point to the
  redirected cell centre (claim 3 a) is 0 (the run counts only when the recording shows a planar
  distance of 0); the target is an enemy vehicle 5 cells away.
- Observable: the first velocity; the tick of detonation.
- Runs: 1.
- Expected: the horizontal length of the first velocity is 100 leptons per tick (not 0) and its
  direction is towards the firer's own target (the aim point, claim 3 a); the projectile detonates
  within a few ticks by the rows of T1.
- Falsified if: the projectile starts with no horizontal motion, or the speed is not 100.

**P29. Passed-by row and jumpjet firers (T1).**
- Setup: a steering test projectile, turn rate 5, speed field 25, arm field 0, fired by a firer
  type with one radial fire segment whose body faces a stationary vehicle 6 cells away (the
  missile leaves a quarter turn clockwise of the body facing, claim 7, and approaches on a curve); run b: the same weapon fired by a jumpjet infantry
  type (the test file gives it the weapon).
- Observable: the position per tick, the recorded half spatial distance η to the stored target
  point, the detonation tick and point.
- Runs: 1 each.
- Validity: a run counts only when the recorded η stays at 32 or more in every turn up to the
  detonating turn (otherwise the arrival row, not the passed-by row, is what acts).
- Expected: (a) detonation in the first turn in which η < 256 and η is larger than Ω (T1: in the
  first turn the unhalved placement distance, later the η of the previous turn), that is once the
  projectile has begun to recede, at its own position, and not an arrival (no snap to the
  target); (b) the same turn, but the detonation point is the target's reference point (counted
  as an arrival, claim 20 b).
- Falsified if: (a) it snaps to the target or detonates before it recedes, (b) it does not snap.
- Variant c (a ranged projectile that is not steering): a straight, anti-air test projectile with
  the ranged flag and arm field 0, fired at an enemy aircraft hovering 4 cells away; gravity 10
  makes it sag below the aircraft, and the run counts only when the recording shows it passing
  the aircraft at a spatial distance between 128 and 384 leptons (so neither the anti-air row nor
  the arrival row applies); a control run without the ranged flag. Expected: the ranged
  projectile detonates in the first turn in which η is below 256 and larger than Ω (T1; here
  the η of the previous turn, since the projectile starts more than 512 leptons from *K*); the control flies on past that turn. Falsified if the ranged projectile flies on past
  that turn too.
- Variant d (T1, the paragraph after the table: the first check is made in the first turn, against
  the unhalved placement distance):
  - Setup: a steering test projectile, turn rate 5, arm field 0, acceleration 50 (a test value),
    course lock duration 10 (so its heading cannot change in the turns recorded and the normal
    acceleration applies, claim 12), not subject to cliffs or walls, not airburst, fired by a stock
    turreted vehicle without radial fire segments that stands at the centre of its cell, with a
    weapon that has the area-fire flag, a range of 8 cells set by the test file and a speed field
    of 20 (stored 51 leptons per tick). The
    test file sets the weapon's forward fire-point shift to 150 leptons and its lateral shift to 0,
    and the fire point about 100 leptons above the terrain (S-0313). The firer is ordered to
    attack an enemy vehicle 5 cells away on level ground. By claim 3 (a) the shot target and *K*
    are then the centre of the firer's own cell, while the launch heads along the firer's aiming
    facing, towards the vehicle and so away from *K*. The first turn takes the speed from 1 to 51
    (claims 9 and 12), so the projectile moves about 51 leptons away from *K* in each of its first
    two turns.
  - Observable: the projectile's position when it enters play and at the start of each turn, the
    recorded *d*₃ to *K* and η in each turn, the detonation tick and point.
  - Runs: 1.
  - Validity, read from the recording: the placement distance *d*₀ (from the position when it
    enters play to *K*) is at least 64 and below 512, trunc(*d*₀ / 2) < η₁ ≤ *d*₀, η₁ < η₂ < 256, and
    no other row of T1 applies in the first two turns (the height above the surface stays above
    0, and half the speed stays below the steering distance to *K*, which is the target point,
    since the projectile's target is the firer's cell). With the setup above *d*₀ is about 180,
    η₁ about 112 and η₂ about 135.
  - Expected: no detonation in the first turn (η₁ is compared with the full *d*₀ and is not larger),
    and detonation by the passed-by row in the second turn, at the projectile's own position (no
    snap: the passed-by row is not an arrival for a firer that is not a jumpjet type).
  - Falsified if: the projectile detonates in its first turn (as it would if placement already
    recorded the halved distance trunc(*d*₀ / 2), or checked), or flies on through its second turn.

**P30. Proximity (T1).**
- Setup: a straight, not inaccurate test projectile (fire point raised as in P7) fired by a test
  vehicle at a cell 8 cells away; an enemy vehicle stands on the line 4 cells from the firer,
  and, on a second run, an allied vehicle instead; a third run with the inaccurate flag and the
  enemy vehicle; a fourth run with the allied vehicle and the firer destroyed by another weapon
  after the shot (no firer).
- Observable: the detonation tick and point, the distance from the projectile to the vehicle.
- Runs: 4 (one of each; no draw decides the outcome).
- Expected: with the enemy vehicle the projectile detonates in the first turn in which the new
  position is less than 128 leptons from the vehicle's reference point (spatial), at the
  vehicle's position; with the allied vehicle it flies on to the cell; the inaccurate projectile
  detonates in the same turn at its own position; without a firer (fourth run) it detonates at the
  allied vehicle as at the enemy one (with no firer any object counts).
- Falsified if: it detonates at 128 or more, flies through the enemy vehicle, detonates at the
  allied one while the firer lives, or flies through the allied one in the fourth run.

**P31. Closing average (T1).**
- Setup: a steering test projectile that cannot reach its target: turn rate 1, speed field 25
  (64 leptons per tick), arm field 1000 (no arrival rows), course lock duration 0, firer with one
  radial fire segment, stationary target vehicle 2 cells ahead of the body facing; a run with
  the airburst flag as control (the row does not apply to it).
- Observable: the position per tick and the spatial distance *d*₃ to the target point.
- Runs: 1 each.
- Expected: the recorded *c*ₖ give *C*ₖ as in T1; the projectile detonates in the first turn *k* > 60
  with 0 ≤ *C*ₖ < 60 (turn count from the end of the lock), at its own position; the control is not
  detonated by this row.
- Falsified if: it flies on in such a turn or detonates by this row earlier.
- Variant b (claim 13 and the closing average without a target; `quirks.origin_as_missing_target_point`):
  as the first run, but the target is a hovering aircraft removed from the game while airborne
  (as P11 run a) in the first turn after the lock, and the test file sets the missile safety
  altitude so high that its row does not act. Observable: the projectile's distance to the world
  origin and to the old target point in each turn. Expected: after the removal the closing steps
  *c*ₖ are the recorded decreases of the distance to the world origin, so *C*ₖ and the turn of a
  detonation by this row follow from those. Falsified when *C*ₖ computed against the old target
  point predicts the recorded outcome and the one against the origin does not.

**P32. Building band, wall overlays and the same building (T1, the first row and the same-building row).**
- The exceptions here concern only the building-band test. Except in run h, use a target cell beyond
  the obstacle and accept a crossing only when no other detonation row applies, in particular no
  nearest-object proximity, arrival or same-building row. Use named test-map overrides for the projectile and
  exceptional building types, with no engine add-on and no mods (the named test files of a scenario excepted), at game speed 2.
- Setup: a straight, not inaccurate test projectile with ρ = 0, fired by a test vehicle across a
  cell holding an enemy building so that it is about 100 leptons above the terrain there (run a) or
  200 above it (run b). Except in runs g and h the firer is destroyed by another weapon after the
  shot and before the projectile reaches that cell, so that the projectile has no firer (Terms); a
  run in which the firer survives that long does not count. Run c: the building is allied with the
  firer's house; run d: a wall segment of the wall overlay in that cell, the projectile 100 above
  the terrain; run e: an overlay that is not a wall (an ore patch) instead; run f: the building is
  an enemy gate held open by its owner's vehicle passing through it while the projectile crosses
  at about 100 above the terrain (the gate is neither laser-fence flagged nor vehicle-like); run g
  (the projectile with a firer): run a with the firer left alive; run h (the same-building row):
  an enemy stock building with a foundation of at least 2 × 2 cells as the shot target, the fire
  point raised so that the projectile enters a cell of the building other than the cell of *K* at
  150 to 207 leptons above the terrain, with a start-of-turn height below 208 (the run counts only
  when the recording shows this). Run i: replace run a's building by a map-placed enemy
  laser-fence section, initially inactive (S-0211 Terms). Run j: replace it by an enemy
  vehicle-like building (S-0214 Terms). Pair i and j with an otherwise matching ordinary-building
  control.
- Observable: the projectile's cell and height per tick, the detonation tick and point.
- Runs: 1 each.
- Expected: (a), (c), (d) and (f) detonation in the first turn whose new position lies in that
  cell and 0 to 149 above the terrain, at the terrain height there. In (b), (e), (g), (i) and (j)
  the projectile flies over the cell, while the ordinary-building controls detonate. (h)
  detonation in the first turn whose new position lies in a cell of the building, although it is
  above the band of the first row and not in the cell of *K*.
- Falsified if: (a), (c), (d) or (f) flies on, (b), (e), (g), (i) or (j) detonates in that cell,
  an ordinary-building control flies on, or (h) flies on into the cell of *K* before detonating.

**P33. Bridge deck crossings (T1).**
- Setup: a map with a high bridge. Run a: an arcing projectile lobbed over the bridge so that it
  crosses the deck height downward; run b: a straight projectile fired from below the deck at a
  target above it (crossing upward); run c: a vertical projectile launched upward through the
  deck; run d: a steering projectile that meets the deck; run e: a vertical projectile launched
  downward through the deck from a firer standing on it (or above it), at a target below. In runs c
  and e the vertical projectile has no target object within 128 leptons of its path and a
  detonation altitude, set by the test file, above every height it reaches in the run (above the
  fire point in run e), so that only the deck row can detonate it in that turn, and
  its speed is chosen so that the turn of the crossing ends at least 30 leptons beyond the deck
  height.
- Observable: the position per tick, the detonation point.
- Runs: 1 each.
- Expected: detonation in the turn in which the new position first lies on the other side of the
  deck height than the old one; the detonation height is the deck height for the upward crossing of
  the straight projectile (b) and for the steering projectile (d), and 20 leptons below it for the
  downward crossing (a). For the vertical projectile (c, e) the detonation height is its own new
  height in that turn (claim 11, T1), at least 30 leptons above the deck height in run c and at
  least 30 below it in run e.
- Falsified if: it passes through the deck, or the height differs from these by more than 1 lepton,
  in particular, in run c or e, a detonation at the deck height or 20 leptons below it.

**P34. Flak row (T1).**
- Setup: as P24 (steering test projectile along a plain at a target on a plateau 6 levels high), with
  the flak-scatter flag on, and a control run with it off.
- Observable: the recorded heights *h*′ of each turn and the detonation tick.
- Runs: 1 each.
- Expected: the flak run detonates in the first turn in which the new position is below the target's
  reference height and its height above the surface is negative, which is at least one turn before
  the control detonates.
- Falsified if: both detonate in the same turn, or the flak run flies on in a turn whose new
  position is below the target's reference height with *h*′ < 0.

**P35. Allied wall transparency (claim 22).**
- Setup: a straight test projectile subject to walls, a wall segment between the firer and the
  target owned by an ally of the firer (run a) or an enemy (run b); each with the allied wall
  transparency setting on and off.
- Observable: fires or moves (S-0307 RG observable); the projectile's fate if forced to fire.
- Runs: 1 each.
- Expected: the ally's wall blocks only when the setting is off; the enemy's wall blocks in both
  cases.
- Falsified if: the setting changes the enemy case, or has no effect on the ally's wall.
- Variant c (the height condition of claim 22): an enemy wall segment between the firer and the
  target with allied wall transparency off, the firer's cell 2 height levels above the target's cell
  (c1) and 2 levels below it (c2). Expected: c1 is not blocked (the firer's cell is above the
  target's), c2 is blocked. Falsified if c1 blocks or c2 does not.
- Variant d (a wall in the end cell does not block, claim 22): a steering test projectile subject
  to walls (turn rate 5, speed field 10, 25 leptons per tick), with a warhead without the wall
  flag, force-fired at the cell of an enemy wall segment 5 cells away. Observable: the position
  per turn and the detonation point. Expected: the projectile enters the wall cell without
  detonating and detonates at the cell's centre (by the arrival row or the steering-distance row,
  both of which put it on the target point). Falsified if it detonates at its first position inside
  the wall cell while that position is more than 64 leptons from the centre.

**P36. The speed row (T1, claim 19).**
- Setup: the test file sets gravity to 0 and the fire point's height shift so that the recorded height
  above the surface at the fire point is between 1 and 9 leptons. A straight test projectile that
  is not inaccurate, with a turn-rate field of −1 (so the shot speed is the weapon's speed field,
  claim 1) and the weapon's speed field 2 (stored 5 leptons per tick), fired at a vehicle 6 cells
  away on level ground (run a). Run b: the same with the fire point 60 leptons above the surface.
  Run c (control): as run a with the speed field 10 (stored 25 leptons per tick).
- Observable: the projectile's speed, position and height above the surface at the start of each
  turn, the detonation tick and point.
- Runs: 1 each.
- Expected: (a) detonation in the first turn, at the position that turn ends at (its speed 5 is
  below 10 and its height is below 10); (b) no detonation in the first
  turn: the height of 60 is not below 10, so the projectile flies on level; (c) no detonation in
  the first turn: the speed 25 is not below 10.
- Falsified if: (a) flies on, or (b) or (c) detonates in the first turn.

**P37. Zero planar distance (claim 6 a, F4, `quirks.zero_distance_arc`).**
- Setup: a map with a high bridge. The firer, a test vehicle, stands still on the bridge deck at
  the centre of a cell; an enemy test vehicle stands still on the ground under the bridge at the
  centre of the same cell, so the target's reference point lies exactly below the firer's. The
  test file sets the weapon's forward and lateral fire-point shifts to 0 (S-0313), so that the
  fire point lies exactly above the target too, and gives the firer an arcing, not inaccurate test
  weapon whose minimum range is 0, and the firer is ordered to force-fire at the target. Run a: the projectile has ρ = 0 and the weapon has the lobber
  flag. Run b: ρ = −1, speed field 10 (stored 25 leptons per tick), with the lobber flag. Run c:
  as run b without the lobber flag. Each run counts only when the firing check passes (a shot or a
  void shot is recorded, told apart by the rearm countdown) and the recording shows a planar
  distance of 0 from the fire point to the impact point and a fire point higher than the impact
  point. The range test and the turret measure to the target's reference point (S-0307 claim 7,
  S-0310), so the alignment does not stop the firer from engaging.
- Observable: the first velocity, the recorded fire point and impact point.
- Runs: 1 each.
- Expected: (a) the start speed is 0 and the first velocity is horizontal, of length 100, pointing
  towards map west (pitch facing −16383 on the launch facing of a zero displacement, which is map
  east, F4 and F1). (b) the first velocity points straight up (pitch facing 0), with length 25. (c) with
  the recorded Δ*z* and *g* = 10, F4's table index for distance 1 is computed: at 2049 or more the
  first velocity points straight down (pitch facing 32766), below 2049 straight up.
- Falsified if: (a) the shot is void, or its first velocity points down, up or towards map east,
  (b) the shot is void or does not point straight up; (c) it points the other way from the one the
  index gives, or the shot is void.

**P38. A target point at the world origin (Terms, claim 17, `quirks.origin_as_missing_target_point`).**
- Setup: a stock building of the enemy pre-placed on the map; a first run records its reference
  point *P*_R. A test art file then gives that building type an aim shift of −*P*_R, so that its
  target point is exactly (0, 0, 0). A steering test projectile (turn rate 5, speed field 25,
  stored 64 leptons per tick), fired by force-fire at the building from 8 cells away. A control
  run without the test art file.
- Observable: the projectile's height above the surface and position per turn, the detonation
  tick and point, the building's health.
- Runs: 1 each; the run counts only when a shot is fired and the recorder shows the building's
  target point at (0, 0, 0). The range test and the turret use the building's reference point
  (S-0307 claim 7, S-0310), not its target point, so the aim shift does not stop the firer from
  firing.
- Expected: in the shifted run the projectile keeps its heading, climbs (pitch towards 8192) and
  detonates when its height above the surface reaches 500, the building losing no health from
  it; the control hits the building.
- Falsified if: the shifted run's projectile steers towards the building, or towards the world
  origin, or hits the building.

**P39. The wrapping shot distance (claim 1, `quirks.shot_distance_wrap`).**
- Setup: a purpose-made flat map of height level 0, large enough (S-0009 claim 1) to hold the
  firer's cell (*x*, *y*) and the cell (*x* + 200, *y*), 200 cells along the map *x* axis. The test
  file gives the stock turreted firer a weapon with a range field of −2 (unlimited range, S-0307
  claim 2) and a straight projectile with ρ = 0 that is not inaccurate, not a floater, not subject
  to cliffs or walls and without the subject-to-elevation property. A vehicle of the firer's
  house stands next to the target cell so that the cell is visible. The firer force-fires at the
  empty cell (*x* + 200, *y*) (a ground attack). Control run: the same at the cell (*x* + 150, *y*).
- Observable: the first velocity, the recorded fire point *F* and the target point of the cell.
- Runs: 1 each (no draw involved).
- Expected: from the recorded points, *n* is Δ*x*² + Δ*y*² reduced modulo 2³² into −2³¹ … 2³¹ − 1
  and *d* = trunc(*r*(|*n*|)) (claim 1); the first velocity is level (claim 6 c, |Δ*z*| ≤ 200) and its
  length is *L* = *r*(*s*₁²) of F1 for *s*₁ = *v*(*d*, 10), within one lepton of *s*₁ (the cap of claim
  2, about 25600, does not act). For Δ close to (51200, 0) that is about 700 leptons per tick
  (*d* about 40908), where the unwrapped planar distance would give about 783. In the control run
  (Δ close to (38400, 0), exact sum below 2³¹) *n* is the exact sum and the length is about 678,
  *v* of the planar distance itself.
- Falsified if: in the 200-cell run the length differs by more than one lepton from *L* of F1 for
  *s*₁ = *v*(*d*, 10) of the wrapped *d* (the unwrapped alternative is about 783), or in the control
  run it differs by more than one lepton from *L* for *v* of the planar distance.

**P40. Trigger nuclear descent (claim 26).**
- Setup: a tiny flat purpose-made skirmish map, one human player and one bystander computer
  player kept away from the test area, standard mode, game-speed index 2, 800 × 600, no engine
  add-on and no mods (the named test files of a scenario excepted). Fire one trigger action 95 at an empty waypoint
  cell. A named test-map override selects a vertical payload projectile with turn rate 0,
  detonation altitude 40000 leptons, and the flags excluded by claim 26 off; it has no trailer,
  is not airburst, inaccurate or subject to cliffs or walls, and does not degenerate. Keep the
  payload weapon's warhead nuclear so that the start of the wait is visible. Do not change
  its velocity through any game-state writer. Use a read-only recorder of position, velocity,
  projectile turns, animation existence and ticks. Exclude all other units, overlays and
  effects from the test cell and its neighbours. Void a run if the bystander interferes.
- Procedure: record one launch through the nuclear wait and damage. Repeat with a stationary
  ground vehicle of the human house centred exactly on the waypoint on level-0 terrain, then
  with an empty intact high-bridge cell whose terrain and deck heights are recorded. Repeat
  each fixture twice. A stock-payload control is evaluated by these same predictions only
  when its recorded properties meet claim 26's conditions; otherwise predict its flight from
  the applicable flight claim, not from the stored speed alone.
- Observable: initial velocity, successive integer position differences, turn of flash and
  fireball creation, their position, later damage tick, and any kill credit. A turn is counted
  from 1 at the projectile's first turn, not from the action tick.
- Expected: the initial vertical velocity is exactly −100, and every non-detonating turn's
  displacement is (0, 0, −100). The empty flat fixture starts its flash and fireball in turn 202 at the
  surface; the centred-vehicle fixture does so in turn 199 at the vehicle's position; the empty
  bridge fixture does so in turn 201 at deck height minus 100. If the first turn occurs in the
  action tick, these correspond to tick differences 201, 198 and 200 respectively; otherwise
  add the recorded first-turn delay. No movement occurs during the wait. Damage follows
  claim 25's animation-end rule and has no attacker or attacking house. The vehicle case
  must not attribute a kill to the human house merely because it owns the victim.
- Falsified if: a qualifying fixture falls at 50 leptons per tick, accelerates despite its
  initial speed exceeding 50, drifts horizontally, starts the wait in another turn or at
  another point, or deals damage before the animation ends. A kill credited to a house
  falsifies the attribution rule. An early unrelated T1 condition voids the timing fixture,
  rather than falsifying its explicitly conditional prediction.

**P41. Railgun hits and terrain (claim 27).**
- Setup: a tiny skirmish map with a flat firing lane, a second lane crossing a ridge, and a
  destroyable cliff. Stock railgun firer and stock targets, no game modifications, fixed game
  speed and recorded seeds; run one game instance at 800×600. Put friendly and enemy vehicles
  beside the flat beam, a multi-cell building intersecting it, and a target in the firer's
  cell in a separate run. Repeat under and over a high bridge with occupants on both layers.
  Keep all objects alive through one shot and record their health immediately around the shot,
  then end the run before a second shot. Record projectile damage separately if that weapon
  also delivers it; do not attribute it to the beam.
- Procedure: fire once in each lane. Sweep the beam past stationary bystanders by varying the
  target position across runs. For ordering, use otherwise identical runs with a first-hit
  object's death effect able to affect a later candidate, and record hit/death times and health.
  For the raw amount, compare a stock firer before and after a firepower bonus, recording only
  the beam's hits. Repeat cliff-cell and ordinary-cell force attacks across exact seeds with
  the random-state recorder; also fire across a non-cliff ridge with a bystander before it.
- Observable: qualifying objects, health deltas at the shot, whether a building receives more
  than one hit, deck versus ground hits, hit order, cliff collapse and the position of each
  R(0, 99) draw relative to ordinary damage and particle creation. Record the imported weapon
  damage, warhead, hit distance and collapse chance rather than assuming stock values.
- Expected: the firer is excluded, friendly bystanders can be hit, each qualifying object is
  hit once with the unmodified weapon amount through ordinary defences, and the target still
  receives its hit when unencountered by the samples. Buildings in visited cells use distance
  0. Bridge occupants qualify only from the selected layer. Terrain cancellation leaves even
  the earlier bystander unharmed by the beam. There is exactly one collapse draw for a
  destroyable-cliff terrain stop or an unobstructed destroyable-cliff cell target, none for an
  ordinary ridge or ordinary cell target. Collapse agrees with the strict chance comparison.
  Any contrary health delta, duplicate hit, opposite layer, bonus-scaled beam amount, earlier
  terrain-blocked hit or extra/missing collapse draw falsifies the corresponding rule.
- Limits: if the installed stock data provides no railgun weapon or cannot isolate its direct
  damage, this fixture cannot validate those rows without a separately authorised test-data
  variant. Do not treat the visible particle trail as an exact position recorder.
- **Part b, distance arithmetic and collinearity (Q-1405 questions 2 and 3).** On the flat lane,
  record exact integer start, target and bystander positions with the read-only recorder,
  along with sampled cells and health before and after one shot. Sweep diagonal beams across
  stationary mobile bystanders just inside and outside the imported threshold, and compare
  the observed hit set against claim 27's equations, not rounded integer distances. Include
  a mobile bystander exactly on an axis-aligned beam in a newly visited cell, then one lepton
  off it. Expected: the collinear bystander qualifies despite the zero denominator, while the
  displaced bystander is tested by the finite quotient. Repeat with buildings in visited cells.
  A discrepancy in the predicted hit set falsifies the arithmetic or comparison. If stock
  data cannot distinguish the mixed-precision formula from a simpler quotient, retain this
  as a pending precision measurement. Thresholds of 0 and below additionally require a
  separately authorised test-data variant: the collinear mobile still qualifies, a non-target
  building does not, and the shot target still qualifies. The ordinary stock run does not
  establish those threshold cases.
- **Part c, zero-length beam (Q-1405 question 3).** Arrange the shot target's position exactly
  at the fire point, with another occupant of that cell, using stock types where normal orders
  and overlap allow it. Record the three-dimensional positions, sampled cells and beam-only
  health deltas for one shot. Expected: no newly visited cell, only the object shot target
  receives a beam hit, end point equal to the start. Any incidental hit falsifies that rule.
  Merely sharing a cell is insufficient: the positions must coincide exactly. If no stock
  fixture permits that coincidence, a separately authorised fixture is needed; no successful
  stock setup or measurement is claimed here.

**P42. Drawing a projectile (claims 28 to 33, F14 to F16; Q-1570).**
- Setup: a tiny flat skirmish map of height level 0 with ordinary terrain, evenly lit, plus a
  high-bridge crossing and one cell lit by a coloured building light (an ordinary stock lit building
  is enough). Named test files: a loose art file that gives a test projectile image section the
  fields each part names; a purpose-made test sprite of 32 image frames, each frame showing its own
  number in large digits; a purpose-made asymmetric test voxel model (a long body with a fin on one
  side and a coloured nose) with a one-frame motion file; a loose rules file whose test weapon uses
  a test projectile type with that image and otherwise copies a stock straight projectile type. No
  game modifications other than the named test files, game speed index 2, 800×600, scripted input, a
  recorder of projectile positions and velocities with the tick, and tick-labelled lossless screen
  capture with at least one drawn frame per tick. The firer is a stock turreted vehicle in the
  middle of the map; targets are unarmed enemy vehicles of a computer player.
- **Part a, rotating sprite.** Rotating flag on, animation fields 0. Fire at targets placed in
  32 directions around the firer (one per run), and in eight extra runs at targets placed just
  either side of four sector boundaries. Observable: the number in the drawn image frame in every
  drawn frame, against the recorded velocity. Expected: image frame (28 − ψ) mod 32 with ψ from
  F14; image frame 0 when flying straight up the screen, 8 left, 16 down, 24 right; a target
  straight above or below changes nothing. In one more run, give the test image to a
  vertical projectile (claim 11) launched straight up, so that its velocity has no horizontal
  part, and record the image frame while it climbs: expected image frame 20 (a velocity with no horizontal part counts as
  map east). Falsified by any other number, by a frame that follows the pitch, or by a
  mirror-image numbering.
- **Part b, control.** Rotating flag off, animation fields 0: every drawn frame shows image frame
  0 in all 32 directions. Any other image frame falsifies claim 29's last case.
- **Part c, animated sprite.** Rotating flag on and low, high and rate set in turn to (2, 5, 3),
  (0, 3, 1), (5, 2, 1) and (1, 4, 0), with a slow test weapon whose flight lasts at least 300
  ticks (use a far target and a low speed). Observable: the image frame number per tick from the
  projectile's first drawn frame. Expected by F15: (2, 5, 3) shows 0 for its first turns, then
  1, 2, 3, 4, 5, 2, 3, 4, 5, …, each held 3 turns; (0, 3, 1) cycles 0 to 3 each turn; (5, 2, 1)
  shows 0, 1, 2 and then 5 for good; (1, 4, 0) changes only every 256 turns. The rotating flag
  makes no difference. Falsified by a first image frame equal to the low field, by a different
  hold time, or by a direction-dependent image frame.
- **Part d, voxel orientation.** The image's voxel flag on. Fire a straight test projectile at the
  32 targets of part a, then a steering test projectile copied from a stock missile at a far
  target so that it climbs, levels and dives; record per tick. Expected: the body's heading takes
  32 steps (ψ of F14) and its pitch 11.25° steps (e of F14), nose up while climbing, no shadow,
  no turret or barrel model even when files named for them exist; the same picture at every tick
  with the same ψ and e. Falsified by smooth pitch, pitch on a sprite, a shadow or a drawn
  turret model.
- **Model depth variant (claim 30).** Use S-0102 scenario D45's paired flat and non-flat
  voxel-projectile fixture behind a high bridge, cliff and building, at two projectile heights.
  Expected: the non-flat model ignores stored depth; the flat model passes the strict flat
  test with final bias −65 depth units and no height-lift term. Neither changes stored depth
  or hides a later reader through depth writes. Depth hiding of the non-flat model, a different
  flat acceptance boundary or any depth write falsifies the claim.
- **Part e, voxel light and palette.** With the projectile of part d flying level east (ψ = 8,
  e = 8), capture its pixels; repeat with the colour definition key absent, set to a known
  definition name in a different letter case, and set to an unknown name; then repeat over the
  coloured-light cell. Compute the expected pixels from F16, the voxel lighting table and the
  named definition's 53-shade palette at brightness 1000 (row 26 at light-alpha 127). Expected:
  absent gives the first definition's 53-shade palette and an unknown name (set in the base rules
  section of the test projectile type, so that no earlier read selected a palette) its single-shade palette, which shows no shade-row
  change over the coloured light or at night; the named definition
  changes the colours; the coloured light does not change them (or changes them only as S-0104
  claim 3 says for house-colour tables). Falsified by the specular shading of S-0105 F4 or by the
  owner's colour.
- **Part f, sprite palette and brightness; nothing drawn.** Sprite projectile with the 32-frame
  image: fire with no palette flag, with the anim-palette flag, with the firer's-palette flag (the
  firer's house in a colour other than the local player's), and with both flags. Repeat each over
  the coloured-light cell and at night-time ambient (a map lighting section of its own), and once
  across the edge of shroud. Then fire with an empty image name and with an image name for which
  no file exists, and with the invisible flag. Expected: general palette, anim palette, the firer's
  house colours, and the anim palette when both flags are set; identical pixels at every light
  except where shroud cover darkens them; nothing at all drawn in the last three runs. Falsified by
  any light-dependent colour, a translucent body or any drawn stand-in.
- **Part g, shadow.** The 32-frame sprite with a falling test projectile (arcing, copied from a
  stock artillery type), shadow flag on, then off; then fire it across the high bridge so that it
  passes the deck at heights above and below the deck height. Observable: the darkened shape, its
  screen position against the body and the projectile's recorded height. Expected: a darkened copy
  of the body's own image frame, its centre *L*(η) rows below the body's centre, drawn under the
  body; none at height 0 or with the flag off; over the bridge cell above deck height it sits on the
  deck, below deck height on the ground. Image frame numbers above the image's frame count (use a
  copy of the test sprite cut to 8 frames) show neither body nor shadow. Falsified by a generic
  shadow shape, a shadow of a voxel projectile, or a shadow at the ground under the deck while the
  projectile is above the deck.
- **Part h, light-glow image.** Give the test projectile type a translucent sprite (S-0007 claim
  23) of a plain bright square and fire over dark ground. Expected: the light-alpha values brighten
  under the glow from the tick the projectile enters play; the body itself is never see-through.
  Record where the glow is in later ticks (Open questions). Falsified by a translucent body or by
  no glow at the projectile's entry point.

**P43. Start point raised above the ground (claim 8; Q-2111).**
- Setup: a tiny flat map with clear ground, no bridge and no cliff; stock data, director and mods
  off. A test vehicle type with no turret and no turret displacement (S-0313) with a steering
  anti-air projectile and no muzzle-position keys for the slot it fires (named test file), so that
  its fire point is its own position at the surface (S-0313 claims 1 and 1a, its frame being the
  flat body rotation of S-0313 scenario P1). One such vehicle of house A at rest, and one hovering
  enemy jumpjet unit of house B held in place in range. Variant b: a test vehicle of the same kind
  whose straight projectile has a muzzle position of 0, 0, 0, attacking a ground vehicle. Variant c:
  the same as a, with a muzzle height 40 above the surface.
- Procedure: order the vehicle to attack; record every projectile it creates at its placement,
  before its first movement, and then on every tick: its position, the terrain height at its *x*
  and *y*, its height above the surface and its speed; and the target's strength on every tick.
- Observable: each projectile's height at its placement, where it detonates, and the target's
  damage events.
- Expected: in a and b, the height at placement is 1, at the vehicle's *x* and *y*; in a, its
  detonations and the target's damage events are recorded, not judged. In c, the height at
  placement is 40. Falsified if a height at placement other than 1 is recorded in a or b, or other
  than 40 in c. If the recorder cannot log a projectile before its first movement, the run is
  inconclusive for the height at placement.
- Runs: 3 of each variant.

**P44. A switched-off hovering target and a projectile with a firer below the ground (claims 3
b and 6 a, T1's first row).**
- Setup: a tiny flat map, stock data apart from the named test files, director and mods off, game
  speed 2. Part a: one test hovering vehicle type that depends on a control centre, of house B,
  switched off at its first heading-point choice as in S-0405 scenario W8; the recorder reads its
  heading point, its speed fraction and its body facing in every tick, and the run counts only when
  a heading point is recorded once it has come to rest. House A has two test vehicles with one test
  weapon whose arcing projectile is not inaccurate, each ordered to attack it from within range: one
  standing ahead of it along its body facing, one behind it, with no other object within two cells
  of the target. The test file sets the hovering type's speed, and the firers' distances are chosen
  from a first run, so that the lead ℓ of F6 is at least 256 leptons, F4 has no solution for the led
  point of the firer behind but has one for the vehicle's reference point, and the led point of the
  firer ahead lies outside the vehicle's cell. The run counts only when the observer confirms all
  three from the recorded geometry. Part b: a test vehicle with an arcing, inaccurate test weapon,
  the ballistic scatter set to 512 by the test file, attacking a fixed cell 6 cells away on level
  open ground (a ground attack on the cell), with level open land for at least 4 cells beyond it,
  inside the legal map and away from high bridges. Of the flags that T1 reads, its test
  projectile type has only the arcing and inaccurate flags: it is not steering, ranged, vertical,
  dropping, level, flak-scattering, invisible, anti-air or subject to cliffs or walls. It keeps
  the anti-ground flag, so that it can attack the cell. No object other than the firer stands within
  two cells of the line from the firer to 4 cells beyond the target cell. Part c: part b with the
  firer destroyed by another weapon after its shot, while the projectile still climbs, the
  destroying object standing outside that strip; one firer per shot. With these flags and that
  ground, T1's rows that can apply in parts b and c are the first row, the two "reached" rows,
  the proximity row, the speed row, the deep-fall row and leaving the map.
- Observable: each fire attempt and whether it creates a projectile, with the firer's rearm
  countdown before and after it (a read-only recorder, as in S-0301's scenarios); each projectile's
  launch velocity, its stored target point, its position and height above the surface in every turn,
  and its detonation point; the target's damage events.
- Runs: 1 for part a over at least 10 attempts of each firer; 5 shots each for parts b and c. In
  parts b and c a shot is set aside, and replaced by another, when its recorded positions,
  heights and speeds show that a row other than the one its expectation names applies first: its
  path enters the target cell with a start-of-turn height below 208 (a "reached" row), it comes
  within 128 leptons of an object in its cell (the proximity row), its speed after a turn is below
  10 with a start-of-turn height below 10 (the speed row), or it leaves the legal map.
- Expected: (a) every launch direction lies within 8 facing units of the direction to the point that
  F6 gives with the recorded speed fraction and body facing, not to the reference point, and every
  projectile's stored target point is the vehicle's reference point at placement. The firer behind
  (F4 without a solution for its led point; the observer computes it from the recorded fire point,
  the start speed of claim 2 and the led point) creates no projectile at any attempt and leaves its
  countdown unchanged, at each firing step. Each projectile of the firer ahead detonates in the turn
  and at the point that T1, T2 and claim 20 give for its recorded positions and heights, with T1's
  first row applying to deck crossings only: in particular it does not detonate in the first turn in
  which it passes below the terrain unless another row of T1 applies in that turn. (b) Each
  projectile flies on below the surface and detonates in the first turn in which its height is 416
  or more below the surface, at the surface there, farther along its flight than the point where it
  passed below. (c) Each detonates in the first turn in which it is below the terrain, at the
  surface there.
- Falsified if: (a) a launch direction lies more than 8 facing units from the led direction, a
  stored target point differs from the vehicle's reference point at placement, the firer behind
  creates a projectile or changes its countdown, or a projectile of the firer ahead detonates in
  another turn or at another point than T1, T2 and claim 20 give, (b) a projectile detonates in the
  first turn in which it is below the terrain, or (c) one flies on below the terrain.

**P45. A group of switched-off hovering targets, and the flak row for a shot with a firer (claims
3 b and 4, T1's flak row; Q-2216).**
- Setup: part a replays M-0003's pairing of the robot tank against the HTK with the robot tanks on
  side B, seed 2, with the harness's five gameplay changes off as M-0017 replays it, and a
  read-only recorder. The front column is the robot tanks' column nearer side A at placement, the
  rear column the other. Part b is P44 part b with one change: its test projectile type also has the
  flak-scattering flag, and the test weapon's range is set to the 6-cell distance of the attack.
- Observable: part a, for every fire attempt of a side-A unit, its tick, its target and whether it
  creates a projectile. For every projectile, its launch velocity, its impact point (Terms) and
  stored target point, its position and height above the surface in every turn, its detonation
  tick and its detonation point and height before and after claim 20. For every robot tank in
  every tick: its position, its destination (the locomotor's, S-0503 Terms), its heading point,
  the length of its stored path,
  its speed fraction, its hover speed, its body facing, whether it is moving in S-0503's sense
  and whether it is switched off, and its health. The tick each robot tank leaves play. Every
  damage event, with the projectile that caused it. Part b: as P44 part b.
- Runs: part a 1, repeated for seeds 0, 1, 3, 4 and 5 when it does not falsify; part b 5 shots,
  with shots set aside and replaced by the rules of P44 parts b and c.
- Expected: (a) For every shot at a robot tank, the observer computes the led point of F6 from
  the robot tank's recorded speed fraction and body facing at the shot's tick when it is moving,
  and takes its reference point when it is not. The impact point lies no farther from that point
  than the largest radius that claim 4 gives for the shot (0 for a projectile type that does not
  scatter). A robot tank that arrives at its heading point after its switch-off by S-0503 claim
  7's test, with no destination and no stored step left after that arrival, has no heading point
  and is not moving from that arrival on. Each projectile detonates in the turn and at the point
  that T1, T2 and claim 20 give for its recorded positions and heights and its projectile type's
  flags. In particular no flak-scattering one is still in play after a turn in which the target's
  reference point is not below the surface beneath it and its height after the turn (*h*′) is
  negative, and one whose new position lies in the
  cell of its stored point *K* at a start-of-turn height below 208 detonates in that turn. Each
  damage event of a robot tank is recorded with its cause and judges nothing here. (b) Each
  projectile detonates in the first turn in which its height after the turn (*h*′) is negative,
  at the surface there, short of the depth of 416 below the surface at which part b of P44
  detonates.
- Falsified if: (a) an impact point lies farther from the computed point than that radius, a
  robot tank keeps its heading point after such an arrival with no destination and no stored step
  left, or a projectile detonates in another turn or at another point than T1, T2 and claim 20
  give, (b) a projectile flies on after a turn in which its height after the turn (*h*′) is
  negative.

## Open questions

- Q-1761 question 2, beyond this spec: this spec answers it for the firing check on every route
  and for the shot's flight and detonation (claim 23, with its answer to Q-1763 question 3). The
  rest belongs to other specs: which orders reach the firing check is S-0214's, and S-0309 claim 5
  keeps such a firer's automatic searches to aircraft. Whether the shot damages the ground unit
  is not settled here.
- Whether a detonation can remove the projectile that causes it, which would end a cluster early
  (claim 21), is not established. S-0303 leaves the same case open (its claim on the animation
  after the area damage).
- The launch velocity of a fragment of a shrapnel weapon that is not invisible is not
  established (S-0318 Open questions), including whether a fragment whose projectile type is
  steering or vertical starts at speed 1 (claim 2). The selection of a burst's targets, its draws and the
  launch of its fragments, which a projectile with a shrapnel weapon makes before its area damage,
  are S-0318's (claims 1 to 6 and its order table). The falling of dropping projectiles (T1), with the
  outcome when only the dropping row applies (T2), is not established either; a later revision
  should cover it. (The original also deflects an arcing projectile that has no firer when it
  hits the ground, but it detonates in that same turn, so nothing of the deflection is observable,
  see Edge cases.)
- Which animation type a projectile with the nuclear warhead creates is fixed by name (claim 25); how
  the radar event of that detonation behaves is the interface specs', and what the nuke flash does
  is S-0104's.
- Overlapping bridge markings and the repair lifecycle of the *elevation-deck mark* used by
  the elevation level in claim 22's cliff test remain open (S-0307, Open questions). S-0307
  Terms establishes the ordinary intact-overlay and destruction scope. This mark is distinct
  from S-0212's *deck-entry mark*, which controls movement onto a deck, and from the
  high-bridge-cell test; those marks are not interchangeable.
- The moving test and the speed fraction of a vehicle target on a locomotor other than the driving
  and hovering ones (claim 3, the speed estimate *u*) belong to the specs of those locomotors, which
  do not yet define them; S-0503 defines both for a hovering vehicle (its Terms and claim 4, claim 3
  b here). The formula of *u* itself is the same for every vehicle.
- Whether a steering projectile whose new position lies outside the legal map is removed by
  some other part of the game: T1 and T2 say it is not removed by the rows of this spec, which was
  not checked beyond them.
- (Q-2111 question 3.) Which of the column's weapons hits the discs in M-0001's pairing, and the
  damage they take, is not answered here: the slot is S-0308's choice and the damage follows the
  imported data (S-0302). P43 records the damage events of its test pairing only.
- Whether the light-glow image of a projectile (claim 33) follows the projectile while it flies,
  and when it is removed, is not established; scenario P42 part h records it.
- (Q-2157.) A switched-off hovering vehicle is led only while S-0503 counts it as moving (claim
  3 b): one that has a destination or a heading point is led, one that has neither, as after the
  arrival of S-0503 claims 7 and 15, is not. P44 part a records its heading point and speed
  fraction, P45 part a its destination too.
  M-0005 does not record heading points or launch velocities, and its per-shot aim point is the
  stored point *K*, which a lead does not move (claim 3 b), so which of its targets were led is not
  established from that record; scenario P44 part a records it. Which of claims 3 b, 6 a and T1's
  first row applies to M-0005's hunters also follows from their weapon's flags in the imported data
  (arcing or straight, inaccurate or not), which this spec does not state.
- (Q-2216.) M-0003 records each game's winner and end sample only, so which of its robot tanks
  were led, and where the shots at them detonated, is not established from that record; scenario
  P45 part a records them. Whether the rear-column robot tanks are hit chiefly by shots aimed at
  them or by the area damage of shots aimed at others (Q-2216 question 3) is not established: it
  depends on the hunters' weapons, their scatter and spread and the placement of the columns,
  which this spec does not state. P45 part a records each damage event with its cause.
  Which of the rules of claim 3 b applies to its hunters follows from their weapon's projectile
  flags in the imported data, which this spec does not state. Which target a hunter takes when its
  target leaves play is S-0309's.
- (Q-2158.) M-0005 records its hunters' fire attempts and whether each made a projectile, not the
  firing check's answer, and not whether an attempt reached the launch. An attempt that reaches the
  launch is made only on the answer *can fire* (S-0310 claim 9, S-0301 claim 9). M-0005 therefore
  does not establish the firing check's answer for its hunters' attempts, nor that their targets
  were inside range and not refused by a minimum range. Which void case of claim 6 a applies to its
  hunters, if any, follows from their weapon's flags in the imported data (an arcing projectile,
  inaccurate or not), which this spec does not state. Scenario P44 part a records, with test types,
  the attempts at a led aim point that F4 cannot reach. It does not tell a void launch from a
  refusal by the firing check either, so claim 6 a's reading of those attempts as void launches is
  `unmeasured` (medium).
- (Q-2158 question 4.) What ends the stall of the hunter of M-0005 that makes projectiles again
  after a stretch of attempts without one is not established. M-0005 does not record why those
  attempts made no projectile, so neither the cause of that stall nor what ended it is known from
  the record. Claim 6 a says what ends a repeating void shot in general (a change of the firer,
  its target or the target's lead, or for an arcing and inaccurate projectile a scatter whose
  impact point F4 can reach), and this spec does not claim that it applies to that hunter.
- (Q-2231.) What a locomotor-seizing hit does in the original to a superweapon warp member from
  its departure until its arrival hold ends is not established: whether the
  member still lands, and when, is not known. Claim 17 gives our engine's rule instead (scenario
  P11 run h, part h, records the original). Whether, in the original, an erasure lock on a member
  that is off the map holds back its warp is not established either (run g part d records V's
  appearance and arrival-effect ticks). Nor is what the direct-rocking and nuke-making rows of
  S-0514 claim 20 do to a departed member, which claim 17 does not list. Nor is the moment at which
  a warp member that moves from one bridge deck to another leaves the ground render layer, for a
  unit with jumpjet movement or an aircraft whose type can land (S-0102 claim 3).

## Changes for other specs

- S-0307 Terms (*airborne*, *floor-bound*): an ordinary object not on the map is neither. Claim
  17 here gives an exception for a superweapon warp member between its departure and its
  appearance (S-0513 claim 39), which is off the map then. For the rules of a projectile's flight
  and detonation that target it (S-0306 T1, T2, claim 20 and claim 17's loss), it is counted by
  its height above the surface beneath its kept position, so one that departed from a high-bridge
  deck counts as airborne (`quirks.warp_deck_departure_airborne`). S-0307 Terms should name the
  exception and point here. Medium, `unmeasured`, as claim 17 rates it.

- S-0513 claim 39 and its Open questions leave open, in the words of the latter: "Where a projectile
  that keeps a warp member as its target flies while the member waits to land, and whether it can
  hit the member then or after it appears (claim 39)." Claim 17 here settles it. Every
  rule here reads the member's position at departure until it appears. A
  steering projectile therefore steers to the old position and then turns to the landing point,
  while a straight or arcing one retains its stored target point. The detonation's ordinary area
  damage does not reach the member while it is off the map, and phasing in does not stop a hit after
  it appears. A landing-check kill gives the cell the member stood in at departure, or no target for
  a member counted as airborne (`quirks.warp_deck_departure_airborne`). S-0513 can point here and
  drop the open question. For the projectile that keeps the member it can cite scenario P11 run f
  here: medium, `measurable`. The other parts are medium, `unmeasured`, as claim 17 rates them.
  Claim 17 also gives what the special hits of S-0514 claim 20 that it lists do to a departed
  member, off the map or phasing in. Takeover, bomb planting, bomb disarming, air-strike
  designation, disguise-making and the erasure lock act on it, charging does nothing, and in our
  engine a locomotor-seizing
  hit does nothing until its arrival hold ends (design). The selected row replaces ordinary
  damage in every case. A warp member is in the ground render layer from the launch until its
  own movement applies again after its arrival hold (S-0102 claims 3 and 4). With
  `quirks.warp_deck_departure_airborne` on, its deck count after a
  refused placement follows the latest passed check (design, with claim 50). S-0513 claim 39 can
  point here for these: medium, `measurable` by P11 runs g, h and j, except how long the layer
  lasts, which S-0102 claims 3 and 4 give and rate, and the two design rules, high as design
  contracts.

- S-0601 claim 5, S-0519 claims 2, 7 and 16, S-0516 claim 31 and S-0514 claims 1 to 4: a
  departed warp member is an ordinary target object for these hits, off the map or phasing in
  (claim 17 here). Each can point here for that case. S-0514 claim 1's ground-layer test passes
  for a warp member by claim 17's render-layer rule. Medium, `measurable` by P11 runs g and h.

- S-0405 claim 20 (its bullet on a disabled vehicle as a target) and its Open questions on Q-1710
  leave open what sets where M-0005's stall shells land, and why some hunters make a fire attempt
  every tick and no projectile. Claims 3 (b), 6 (a) and T1 here give mechanisms for shots at a
  switched-off hovering vehicle that S-0503 counts as moving: it is led by the ordinary rule. A
  led arcing shot can be void, and in an ordinary vehicle firing step after the rearm countdown
  has run out a void launch leaves that countdown unchanged, so the firer attempts the shot again
  on each later *can fire*. A straight or arcing projectile whose firer is still in play outside
  any transport and has not been killed or started to cloak that passes below the ground is not
  detonated there but flies on, so it can detonate away from the vehicle (T1 and T2 determine its
  detonation), unless it is flak-scattering and has a target: where the target's reference point
  is not below the surface beneath it, T1's flak row then detonates it in the first turn in which
  it is below the terrain. Which vehicles of a switched-off group are led follows S-0503's moving
  test (claim 3 b here): one with a destination or a heading point is led, one that has neither,
  as after the arrival of S-0503 claims 7 and 15, is not. S-0405 can point
  here for those rules. Which of them applies to M-0005's hunters follows from their weapon's flags
  in the imported data, which neither spec states. S-0405's sentence that no targeting or damage
  rule reads the disabled state stays true: the lead reads the vehicle's moving state (S-0503
  Terms), not its disabled state. The pointer would be medium, `measurable` by scenarios P44
  and P45 here.

- S-0318 claim 5 places a fragment at the burst point and points to claim 8 here. By claim 8, a
  fragment whose burst point is not above the terrain starts 1 lepton above it, at the same *x*
  and *y*, so a burst at height 0 gives fragments that start at height 1. Claim 5 should say so,
  pointing here, where that raise is `unmeasured` and rated medium.

- S-0308 claim 11 leaves open whether the original refuses a shot when the firer's naval-targeting
  mode gives "none" (table N). Claim 23 here answers it: the firing check answers
  *illegal* for such a target on a water or beach cell. Claim 23 rates that answer medium,
  `measurable` by scenario P16 part h's water variant, except the water row's mode-0 branch,
  which it rates low and `unmeasured`, so S-0308 claim 11 can point here but
  should not treat it as measured.
- S-0301 claim 9: the water and land rows of claim 23 here give their *illegal* at rank 4. Rank
  4's list should name them beside the airborne and non-ground rows, and the *illegal* row of the
  answer table should name them among the cases of S-0306 claim 23. Its closing sentence, that no
  other precedence within rank 4 is specified, should give way to claim 23's order: the rows
  yield to the rank-4 *impossible* and *rearming* answers that claim 23 names, and the
  magnetic-beam *rearming* yields to them.
- S-0309 claim 18 (revision 38, companion of this revision): retaliation is bounded by the
  firing check and the behaviour rules of S-0215, so the land row here is what stops an object
  whose type has land-targeting mode 1 from firing back at a ground attacker (a building whose
  type has the SAM flag is stopped by S-0215 claim 32 instead).
- S-0214 scenario T4 part C (e) gives an own unit "a weapon that cannot hit ground targets" and
  expects no attack without force-fire over an enemy tank. By claim 23 the firing rules rule a
  ground object illegal for such a unit only when its type's land-targeting mode is 1: the
  anti-ground flag is not tested for object targets, so with mode 0 (the default) a weapon whose
  projectile is anti-air only is offered the attack (scenario P16 part h). T4 part C (e) should
  name the test unit's type as having land-targeting mode 1, or add that mode to its test file.
  Its expectations then stand as written.
- S-0503 claims 7, 12, 15, 16 and 19 together give an observable consequence for a switched-off
  hovering vehicle that S-0503 could state beside claim 10, pointing here: one that has a
  destination or a heading point when switched off is moving and shots at it are led while it
  keeps either (claim 3 b here, `quirks.switched_off_hover_lead`). One that has neither is not
  moving, so shots at it are not led: for example one that arrives at its heading point by claim
  7's test, with no destination and no stored step left after that arrival (claim 15). Losing
  the heading point alone, as at a refused step (claim 12), leaves it moving while it keeps its
  destination. Medium, as S-0503 rates claim 10's exceptions, `measurable` by scenario P45
  part a here, which records the heading point, speed fraction and moving state of each vehicle
  of such a group.

## Questions for the project lead

Decided under the project lead's delegated sign-off (2026-10-10); recorded on the spec side:

- The firestorm point of claim 9 stays in the engine as a dead rule until a later spec gives the
  firestorm a way to turn on.
- The flatter-arc launch after a failed solve for *d* + 1 (F4, Edge cases) is accepted as a
  design-choice departure with no switch and no importer warning, as for S-0205 claim 8: launch
  at θ.
- No locomotor seizing of a superweapon warp member from its departure until its arrival hold
  ends (claim 17, Q-2231). The original's outcome is not established and could leave a member
  stuck or carried while unseen; our engine keeps the warp self-contained. The row still replaces
  ordinary damage. Design with no switch, decided under the autonomy granted 2026-10-25.
- With `quirks.warp_deck_departure_airborne` on, the deck count of a warp member after a refused
  placement follows the latest passed check (claim 17, Q-2232), as S-0513 claim 50's refusal is
  itself our engine's design. Each pass sets the count from its own landing cell. Design with no
  switch, under the same autonomy.
- The glossary's *fire point* entry agrees with claim 5 here: only a dropping projectile starts from the firer's own position. No glossary correction is required.
