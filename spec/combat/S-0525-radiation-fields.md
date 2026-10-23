---
id: S-0525
title: Radiation fields
topic: combat
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-23
measurements: []
requests: []
supersedes: []
---

# S-0525: Radiation fields

## Summary

How a detonation whose weapon has a positive radiation level leaves a radiation field on the
ground, how the field spreads its radiation over the cells around its centre, how a second
detonation at the same centre adds to it, how it fades and ends, how it damages the mobile units
standing in it, and how it lights the ground. Skirmish only; network games and campaigns are out
of scope.

This spec builds on, and does not restate:
- **Tick order:** S-0203 Terms (*radiation field*) and claim 1 row 13 (fields are updated in their
  own part of the tick, the most recently created first).
- **Immunity of the hit:** S-0302 claim 8 table row 2 (a radiation warhead on a type immune to
  radiation). Claim 8 here adds the separate test that keeps immune types from being hit at all.
- **The nuclear strike's hand-off:** S-0513 claim 12 (the payload leaves radiation at its
  detonation cell with the payload weapon's level and the warhead's spread truncated to whole
  cells, or adds to the field already there). Claims 1 to 4 here are the general rule it hands
  off to.
- **The deploy shot:** S-0215 claim 31 (the radiation-immune infantry unit's own-cell shot) and
  S-0517 claim 16 (the deployed-firing rule); the shot's
  detonation follows claims 1 to 4 here like any other. S-0215 claim 31 reads the field level of
  claim 7 for the deployed rows of radiation-immune types.
- **Projectile detonations:** S-0306 claims 19 to 21 (when, where and how many times a projectile
  detonates), S-0312 claim 6 (death weapons) and S-0524 claim 3 (the trigger-created nuclear
  payload).
- **Hits:** S-0302 (what a hit does), S-0312 (deaths and the no-escape marker).
- **Lighting:** S-0104 claims 9, 11 and 12 (how a light source lights the cells it reaches).
  Claims 13 to 15 here answer S-0104's open question on light sources other than buildings.
- **Number domains:** S-0205 claims 3 to 7 (integer division by zero, double precision, the
  round-toward-zero mode, conversions to integers) and S-0204 claim 5 with F1 (the table square
  root).

This revision states explicitly what the committed text did not say: the fragments of S-0318
leave no radiation, since they carry no weapon, so a fragment weapon's radiation level has no
effect (claim 1; S-0318 claim 5).

## Terms

Shared terms used here are defined in the glossary: *tick*, *turn*, *weapon*, *warhead*,
*projectile*, *detonation, detonation point*, *hit*, *house*, *infantry*, *vehicle*, *aircraft*,
*building*, *in play, on the map, off the map*, *airborne*, *in-the-air answer*, *light source*,
*importer warning*, *scenario random stream*, *quirk switch*. In this spec *field* always means a
radiation field, never a data setting. In addition:

- **Radiation field (field).** The irradiated patch of ground of S-0203 Terms. A field has a
  *centre cell*, a *spread* *s* (whole cells), a *field level* *L* (an integer, in the
  radiation-level units of S-0513 Terms), a *duration* *N* and a *time left* *T* (both in ticks),
  and one light source (claim 13). Any number of fields can exist; at most one field has a given
  cell as its centre cell (claim 2).
- **Radiation level of a weapon.** The weapon's integer radiation level *L*<sub>w</sub>
  (Import mapping). A weapon with *L*<sub>w</sub> > 0 is a *radiation weapon*.
- **Field reach.** The radius *R* = 256·*s* + 128 leptons of a field (F1).
- **Contribution.** The amount *c* of radiation a field adds to one cell (F1): a non-negative
  double.
- **Cell radiation.** The radiation *C* held by a cell: a double, the running sum of the
  contributions the fields have added to the cell, less what their decay and topping up have taken
  from it (claims 3 to 6), never below 0. It is distinct from a field level: a cell can hold
  radiation from several fields, and from fields that have ended (claim 6).
- **Exposure level.** The integer ℓ that a hit reads from a cell's radiation (F6).
- **Field update.** The once-per-tick processing of one field in S-0203 row 13.
- **Level delay *D*<sub>L</sub>, light delay *D*<sub>I</sub>, application delay *A*, duration
  multiple *M*, level cap *C*<sub>max</sub>, level factor *k*<sub>L</sub>, light factor
  *k*<sub>I</sub>, tint factor *k*<sub>T</sub>, radiation colour, radiation-site warhead.** The
  game-wide radiation settings (Import mapping). Delays are in ticks.
- **Level steps *n*<sub>L</sub>, light steps *n*<sub>I</sub>.** How many decay steps the level and
  the light of a field are divided into (F2).
- **Airborne** in this spec means the in-the-air state that S-0308 uses: on the map and at least
  208 leptons above the surface (an object standing on a bridge deck measures from the deck), with
  the in-the-air answer of S-0501 claim 18 for the two game-wide rocket aircraft types.

## Behaviour

### Making a field

1. **Which detonations leave radiation.** Every detonation of a projectile that carries a
   radiation weapon leaves radiation at the cell of its detonation point, the *detonation cell*.
   This covers:
   - a fired projectile's detonation (S-0306 claims 19 and 20), and each detonation of a projectile
     with a cluster count above 1 (S-0306 claim 21), each at its own point,
   - a death weapon's detonation at the dying object's position (S-0312 claim 6),
   - the nuclear payload, fired from a silo (S-0513 claim 12) or created by a trigger action (S-0524
     claim 3), and its upward missile if that weapon is a radiation weapon,
   - the own-cell deploy shot of S-0215 claim 31.

   Radiation is created or added on the detonation's tick. When the own-cell deploy shot
   detonates immediately, this is the tick of the behaviour processing that decides to fire:
   neither a *deployed-fire* animation frame nor a later field update delays it. This does
   not bypass projectile travel or detonation rules for other weapons.

   It happens whatever the warhead does: a parasite warhead (S-0306 claim 21) and the nuclear
   warhead (S-0306 claim 25) leave radiation too, and so does a detonation over water, on a bridge
   or on a cliff. Nothing about the detonation's targets or damage matters. Area damage that does
   not come from a projectile carrying a weapon leaves none: planted bombs, storm bolts, crashing
   aircraft without a death weapon, animations and trigger damage (S-0303 producers), the
   nine children of an airburst projectile, which carry no weapon (S-0306 claim 21), and the
   fragments of a shrapnel weapon, which carry none either, so a fragment weapon's radiation
   level has no effect (S-0318 claim 5). The airburst projectile's own detonation follows the
   rule with its own weapon.
   The spread of the new radiation is *s* = trunc(*S*<sub>w</sub>), the warhead's cell spread
   truncated toward zero to whole cells.
   - Backing: `measurable` (scenario RF1 for a fired shot; RF9 for cluster detonations, airburst
     children and a death weapon; RF4 part e for immediate own-cell detonation timing; S-0318 scenario SH1 part n, run 4, for
     the fragments of a shrapnel weapon)
   - Confidence: high for fired shots, death weapons and the payload, and for fragments, as
     S-0318 claim 5 rates the absence of their radiation; medium for cluster detonations and
     airburst children

2. **New field or added radiation.** When the detonation cell is the centre cell of a field that
   still exists, the radiation is added to that field (claim 4), whatever spread the new
   detonation has. Otherwise a new field is made with the detonation cell as its centre cell,
   spread *s*, field level *L* = *L*<sub>w</sub>, duration and time left *N* = *T* =
   *L*<sub>w</sub>·*M* (F2), and its light source (claim 13); it adds its contribution to every
   cell it reaches (claim 3). Only the centre cell counts: a detonation in a cell that another
   field reaches, but that is not that field's centre, makes a new field, and the two overlap
   (their contributions add up in the cells both reach). A new field is the most recently
   created field for S-0203 row 13; a field that radiation is added to keeps its place.
   - Backing: `measurable` (scenarios RF1, RF4)
   - Confidence: high

3. **Spread over cells.** A field reaches every cell whose centre point lies within its field
   reach *R* of the centre cell's centre point, measured in three dimensions with the table square
   root (F1): height differences count, so a cell above or below the centre is reached less, or
   not at all. Each reached cell gets the contribution *c* = (*R* − *d*)/*R* × *L* of F1, a
   double: the full field level at the centre cell, falling linearly to 0 at distance *R*. Cells
   beyond *R* get nothing. With spread 0 only the centre cell is reached. The cell radiation is the
   sum of the contributions of every field, added up with double-precision arithmetic (F1).
   - Backing: `measurable` (scenario RF2)
   - Confidence: high

4. **Adding to a field.** When radiation of level *L*<sub>w</sub> is added to an existing field
   (claim 2), the field takes, as one change at the moment of the detonation:
   - its new field level *L*′ = ⌊*L*·*T*/*N*⌋ + *L*<sub>w</sub>: its current field level of
     claim 7 (formed in floating point, F4) plus the new level, not its original level,
   - from every cell its square of cells covers (F5), the amount *r* of F5, computed from its old
     level, its level steps and its time left, with the cell radiation not going below 0
     (`quirks.radiation_add_overdraw`),
   - its new duration and time left *N*′ = *T*′ = *L*′·*M*, with both its decay timers starting
     again from the current tick and its level and light steps recomputed for *N*′ (F2),
   - its spread unchanged, so it then adds the contribution of *L*′ over its old reach (claim 3),
   - its light set afresh from *L*′ (claim 14), with its old radius.

   For an immediately detonating own-cell deploy shot (S-0215 claim 31), this top-up is
   already effective on the deciding processing's tick, without waiting for an animation
   frame. Later turns in that tick read the changed cell radiation under claim 11.
   - Backing: `measurable` (scenario RF4, including part e for immediate own-cell top-up timing); the over-removal of `quirks.radiation_add_overdraw`
     by scenario RF4 part b
   - Confidence: high for the new level, duration and spread; medium for the removal

### Fading and ending

5. **Decay.** Each field update lowers the time left *T* by 1. When at least *D*<sub>L</sub>
   ticks have passed since the field was made, since radiation was last added to it, or since its
   last decay step, the field takes a decay step: every cell of its square (F5) loses
   *c*/*n*<sub>L</sub>, the cell's contribution of F1 for the level set at creation or the latest
   top-up, divided by the level steps, in floating point (F3), and a cell's radiation that would fall below 0
   becomes 0. With the usual data, a field takes exactly *n*<sub>L</sub> decay steps over its
   life, at *D*<sub>L</sub>, 2*D*<sub>L</sub>, … ticks after it was made or last added to, so its
   contributions are all taken away again, up to the rounding of the steps. The cell radiation
   therefore falls in steps every *D*<sub>L</sub> ticks, while the field level of claim 7 falls by
   one part in *N* every tick. The decay of the light is claim 14.
   - Backing: `measurable` (scenario RF3)
   - Confidence: high

6. **End of a field.** A field ends in the field update in which its time left reaches 0, after
   that update's decay step if one falls due: *N* field updates after it was made or last added
   to. From then on its centre cell is no longer a centre cell (a later detonation there makes a
   new field) and its light source is gone (claim 14). Ending takes nothing from the cells: any
   contribution its decay steps have not taken remains, adding to any later field's and still
   damaging mobile units there (`quirks.radiation_residue`), until later radiation additions or
   subtractions change it. This happens with zero level steps, when duration is shorter than
   one level delay, or with fewer than *n*<sub>L</sub> steps, when it was made before row 13
   (for example by a death weapon of an object a storm bolt kills) and *D*<sub>L</sub> divides
   its duration, so its first field update falls in the tick it was made and it ends one decay
   step short. A field made in the turn pass, as fired shots are, first updates in the next tick.
   - Backing: `measurable` (scenario RF3 for the end tick; RF5 parts a and b for the residue)
   - Confidence: high for the end tick; medium for the residue cases

7. **Field level.** A field's current *field level* is ⌊*L*·*T*/*N*⌋ (F4): its level scaled by
   the share of its duration still left, truncated toward zero, and 0 when *N* < 1. Adding to a
   field uses its floating-point counterpart (claim 4, F4), whereas S-0215 claim 31 reads the
   integer form for the deployed rows of
   radiation-immune types: there "its cell has radiation of level at least *R*" means that the
   unit's cell is the centre cell of a field whose field level is at least that threshold.
   Radiation that reaches the cell from a field centred elsewhere, and radiation left by ended
   fields, does not count there.
   - Backing: `measurable` (scenario RF4 for the level added to; S-0215 scenario B19 for the
     deployed rows)
   - Confidence: high

### Damage to objects in a field

8. **Who is hit.** A field damages infantry units, vehicles (naval and sinking vehicles included)
   and aircraft. It damages one only while it is on the map, is not airborne (Terms) and its type
   is not immune to radiation. Buildings are never hit, whatever radiation their cells hold, and
   neither are objects off the map: passengers of a closed transport, occupants of a garrisoned
   building, or units inside another building. A landed aircraft and a hovering
   vehicle near the ground are hit; a jumpjet unit at a cruise height of at least 208 leptons
   above the surface is not. The cell
   that counts is the cell of the object's position, on whatever layer it stands: a unit on a
   bridge deck reads the radiation of the cell below the deck, and is not airborne there.
   The immunity test is the type's own flag, applied before any hit is made: an immune type gets
   no hit at all, so none of S-0302 claim 10's side effects happen, whether or not the
   radiation-site warhead is a radiation warhead (S-0302 claim 8 row 2 is a separate rule for
   other hits).
   - Backing: `measurable` (scenario RF6)
   - Confidence: high for infantry, vehicles, buildings, immunity and objects off the map; medium
     for the airborne test, aircraft and bridge decks

9. **Cadence and amount.** In each tick whose tick number *t* (S-0201) is a multiple of the
   application delay *A* (*t* mod *A* = 0), every object of claim 8 reads its cell's radiation
   during its own turn in the turn pass (S-0203 row 5). Its exposure level is ℓ =
   trunc(min(*C*, *C*<sub>max</sub>)) (F6). When ℓ ≥ 1 it takes one hit with the amount
   trunc(ℓ·*k*<sub>L</sub>), formed in floating point (F6). So every object in a field is hit on the
   same ticks, each by the radiation of its own cell, capped at the level cap; between those ticks
   no radiation damage happens, however long an object stands in a field, and an object that
   enters a field just after such a tick takes nothing until the next one.
   - Backing: `measurable` (scenarios RF1, RF2)
   - Confidence: high

10. **The hit.** The hit of claim 9 has:
    - the radiation-site warhead (Import mapping), at distance 0 from its detonation point, so
      S-0302's versus value and falloff (claims 5 and 6) apply with no distance reduction,
    - defences applied (it does not ignore defences),
    - no attacker and no attacking house (S-0302 claim 1), so a kill by radiation credits no
      object and no house (S-0302 claim 14), whoever made the field,
    - the no-escape marker (S-0312 Terms), so the passengers and crew of a vehicle it kills follow
      S-0312's rules for such a kill.

    The hit is delivered even when its amount is 0; S-0302 resolves it. A kill ends the object's
    turn as S-0312 says.
    - Backing: `measurable` (scenario RF7 parts a and b for credit and the no-escape marker,
      RF1 for defences and the amount)
    - Confidence: high

11. **Order within a tick.** Observable consequences of where the parts of a field happen:
    - a decay step taken in row 13 of a tick is already in the cells when the objects of that
      tick's turn pass read them,
    - a field made in the turn pass of a damage tick is read by the objects whose turns come after
      the detonation in that tick, and not by those whose turns came before it,
    - an object's self-healing (S-0314) in its turn comes before its radiation hit in the same
      turn, so a self-healing unit at full strength that is hit in a healing tick ends that tick
      below full strength by the whole hit.
    - Backing: `measurable` (scenario RF3 for the first; RF8 for the second and third)
    - Confidence: high for the first; medium for the second and third

### Draws

12. **No draws.** Making a field, adding to it, its field updates, decay steps and light changes
    and its end take no value from the scenario random stream (S-0202). Row 13 of S-0203 holds its
    place, but in the original it never draws. The draws that a hit causes (a damage effect, a
    death) belong to S-0302 and S-0312, in the hit object's turn.
    - Backing: `measurable` (scenario RF10, with the random-state recorder)
    - Confidence: high

### Light

13. **A field is a light source.** Each field has one light source (S-0104 Terms) at the centre
    point of its centre cell, at that cell's ground height, with radius *R* (claim 3's field reach,
    in leptons) and the intensity *I* and tint (*t*<sub>r</sub>, *t*<sub>g</sub>,
    *t*<sub>b</sub>) of F7, all in ‰ like a building's light (S-0104 claim 12). It lights the cells
    it reaches by S-0104 claims 9 and 11 (a reach in two dimensions, unlike claim 3). As in
    S-0104 claim 11, the light-reach claim covers radii below 32000 leptons; larger light radii
    remain open. Unlike a building's light it counts at every detail level, low and medium included. Every change of it
    (made, faded, set afresh, removed) relights the cells it reaches in the same tick, as S-0104
    claim 12 says for building lights. Field lights of overlapping fields add up like any light
    sources.
    - Backing: `measurable` (scenario RF11)
    - Confidence: high for position, radius and the detail levels; medium for the tint

14. **Light fading.** When at least *D*<sub>I</sub> ticks have passed since the field was made,
    since radiation was last added to it, or since its last light step, its field update takes a
    light step: the intensity drops by Δ*I* = trunc(*I*/*n*<sub>I</sub>) and each tint channel becomes
    trunc(*t*<sub>c</sub>·*T*/*N*), the channel's starting value scaled by the share of time left
    (F7). The intensity therefore falls by equal steps, and the tint in proportion to the time left.
    When radiation is added (claim 4) the light gets the fresh intensity and tint of F7 for the new
    level *L*′ and keeps its radius. When the field ends, its light is removed.
    These are the required presentation results, including a step that changes only green.
    The original sometimes leaves a green-only light change invisible when intensity, red and
    blue remain unchanged; this drawing-only defect is corrected without a quirk switch.
    - Backing: `measurable` (scenario RF11, including part d for that original defect)
    - Confidence: medium

15. **Light colour on snow maps.** On a map of the snow theater, the red and blue channels of the
    radiation colour are replaced by −128 before the conversion of F7, whatever the imported
    colour; green keeps its imported value. So on snow the field light subtracts red and blue
    (−501 ‰ before the tint factor) and the irradiated ground looks more strongly green than on
    other theaters. This is presentation only (Open questions).
    - Backing: `measurable` (scenario RF11 part c)
    - Confidence: medium

### Data that the original cannot run

16. **Delays and durations that stop the original.** The original stops with a program error
    (S-0205 claim 3) when the application delay is 0 (at the first turn of any infantry unit,
    vehicle or aircraft, field or no field), when the level delay or the light delay is 0 (at the
    first field made), and when a field has zero duration or positive duration shorter than a
    positive light delay (*n*<sub>I</sub> = 0, at that field's making, or at radiation being added). **Our engine (a departure):** the
    importer warns about an application delay, level delay or light delay of 0 or less and about a
    duration multiple of 0 or less (glossary, *importer warning*), and then:
    - with *A* = 0, fields damage nothing, including in tick 0; this is an explicit safe rule
      rather than applying the general remainder rule to the cadence test,
    - a decay step with *n*<sub>L</sub> = 0 takes nothing, and so does the removal of old
      radiation when adding to such a field; the new contribution is still added,
    - with *n*<sub>I</sub> = 0, Δ*I* = 0 (S-0205 claim 3): the intensity stays as made until the
      field ends, and the tint still scales with the time left.

    The importer also warns when a field duration gives zero light steps, or when a top-up
    would remove old radiation with zero level steps (Edge cases).
    A negative application delay behaves like its absolute value, as in the original. A duration
    multiple of 0 or less gives *N* ≤ 0 when the product does not overflow: the field ends at its first field update, after adding its
    contributions, which then stay in the cells (claim 6). No switch reproduces the stops.
    - Backing: `unmeasured` for our engine's safe results (they are departures rather than
      observations of the original); `measurable` for the original's zero-divisor stops and
      negative application-delay cadence (scenario RF12)
    - Confidence: high for the application delay and the level and light delays; medium for the
      short duration

## Formulas

All positions are in leptons (256 per cell edge). Integer divisions truncate toward zero (S-0205
claim 2). Floating-point operations are in double precision with the round-toward-zero mode of
S-0205 claims 4 and 5 (`quirks.chop_rounding`), and every conversion to an integer truncates toward
zero by the common form of S-0205 claim 7. Integer products and sums wrap as in S-0205 claim 1,
including reach, level addition and light fading. Every floating-point site of this spec is named below.
Unless explicitly discussing bad data, the formulas below assume non-negative spread, positive
level and delays, and positive duration without overflow. Their ordinary geometric explanations
do not extend to a wrapped reach or duration.

**F1, reach and contribution** (claims 2, 3). For a field with centre cell (*x*₀, *y*₀), spread *s*
and level *L*:

> *R* = 256·*s* + 128 leptons (integer).
>
> For a cell (*x*, *y*), let *P* and *P*₀ be the centre points of the cell and of the centre cell,
> each (256·*x* + 128, 256·*y* + 128, *h*) with *h* the ground height at that point in leptons
> (S-0204). *n* = |*P* − *P*₀|² (leptons²), formed from double-precision squares and sums
> of the integer coordinate differences; these are exact at map-sized distances.
> *d* = ⌊√ₜ(*n*)⌋, with √ₜ the table
> square root of S-0204 F1 (held in double precision, narrowed to single precision as there).
>
> *c* = ((*R* − *d*) / *R*) × *L* when *d* ≤ *R*, else 0. **Floating point:** *R* − *d*, *R* and
> *L* are converted exactly to doubles; the quotient is formed first, then the product.
>
> Cell radiation: *C* ← *C* + *c* when a field is made or added to (double addition).

Only cells with |*x* − *x*₀| ≤ *s* and |*y* − *y*₀| ≤ *s* can be reached; every cell outside
that square is farther than *R* at ordinary map distances, so the square is no extra condition
there. The square remains the explicit limit for extreme data. Cells that are not map cells take
and keep no playable radiation.

**F2, duration and steps** (claims 2, 4, 5, 14). All integer:

> *N* = *L*·*M* ticks (wrapping as S-0205 claim 1), *T* starts at *N*.
>
> *n*<sub>L</sub> = trunc(*N* / *D*<sub>L</sub>), *n*<sub>I</sub> = trunc(*N* / *D*<sub>I</sub>),
> Δ*I* = trunc(*I* / *n*<sub>I</sub>) (claim 16 for divisors of 0).

Decay timers: a step falls due in the first field update at a tick *t* with *t* − *t*<sub>s</sub> ≥
*D*, where *t*<sub>s</sub> is the tick the timer last started (the making, the last adding, or its
own last step) and *D* is *D*<sub>L</sub> or *D*<sub>I</sub>; the step restarts it at *t*.

**F3, decay step** (claim 5). For every cell of the field's square, with *c* its contribution of F1
for the *L* set at creation or latest top-up, and its unchanged *R*:

> *C* ← max(0, *C* − *c* / *n*<sub>L</sub>). **Floating point:** *n*<sub>L</sub> converted exactly
> to a double; quotient, then subtraction; a negative result becomes 0.

**F4, field level** (claims 4, 7):

> For S-0215 claim 31: *F* = trunc(*T*·*L* / *N*) in integers (32-bit product, S-0205 claim 1),
> and *F* = 0 when *N* < 1.
>
> For adding (claim 4): *F* = trunc(*L*·*T* / *N*), **floating point:** *L* and *T* converted to
> doubles, product, then quotient, then the conversion of S-0205 claim 7.

For a live field with positive duration, the two agree whenever *T*·*L* fits in 32 bits.

**F5, removal when adding** (claim 4). With the old *L*, *n*<sub>L</sub> and *T* (before the
change) and *c* each cell's contribution of F1 for the old *L*:

> *r* = (*c* / *n*<sub>L</sub>) × (⌊*T* / *D*<sub>L</sub>⌋ + 1); *C* ← max(0, *C* − *r*).
> **Floating point:** the quotient first, then the product with the exactly converted integer
> ⌊*T*/*D*<sub>L</sub>⌋ + 1 (an integer division).

The following comparison of amounts ignores floating-point roundoff. For the ordinary schedule
whose first field update is in the following tick, write *k* for the decay steps the field has taken since it was made or last added to, so the cell
still holds *c*·(1 − *k*/*n*<sub>L</sub>) of it. With *N* = *a*·*D*<sub>L</sub> + ρ and *j* = *N* −
*T* = *b*·*D*<sub>L</sub> + σ (0 ≤ ρ, σ < *D*<sub>L</sub>), *r* takes exactly what the field still
holds when ρ < σ, and one decay step more (*c*/*n*<sub>L</sub>) when ρ ≥ σ, for example in the first
*ρ* + 1 ticks after the making, or at any tick of a decay step when *D*<sub>L</sub> divides *N*. The
surplus comes out of the other fields' radiation in that cell, or vanishes in the clamp at 0 if
there is none. With `quirks.radiation_add_overdraw` off, *r* = *c*·(1 − *k*/*n*<sub>L</sub>), the
field's own remaining contribution (*r* = 0 when *n*<sub>L</sub> = 0).

**F6, exposure and hit amount** (claim 9). *C*<sub>max</sub> is an integer converted exactly:

> ℓ = trunc(min(*C*, *C*<sub>max</sub>)); amount = trunc(ℓ·*k*<sub>L</sub>) when ℓ ≥ 1.
> **Floating point:** the comparison is exact (S-0205 claim 4), *k*<sub>L</sub> is a value held as a double,
> the product is one rounded multiplication.

**F7, field light** (claims 13 to 15). With (*κ*<sub>r</sub>, *κ*<sub>g</sub>, *κ*<sub>b</sub>) the
radiation colour's channels (0 to 255; on the snow theater *κ*<sub>r</sub> = *κ*<sub>b</sub> = −128):

> *I* = trunc(min(*L*·*k*<sub>I</sub>, 2000)) ‰.
>
> *q*<sub>c</sub> = trunc(1000·*κ*<sub>c</sub> / 255) (integer, toward zero, so −128 gives −501),
> *t*<sub>c</sub> = trunc(min(*q*<sub>c</sub>·*k*<sub>T</sub>, 2000)) ‰.
>
> **Floating point:** both products are one rounded multiplication of an exactly converted
> integer by a value held as a double, and there is an upper cap only (no lower cap).
>
> Light step: intensity ← intensity − Δ*I*; tint channel ← trunc(*t*<sub>c</sub>·*T* / *N*)
> (integers, 32-bit product).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Lepton per cell edge | 256 | leptons | F1 |
| Reach margin | 128 | leptons | half a cell, F1 |
| Colour scale | 1000/255 | ‰ per colour step | F7, truncated toward zero |
| Light cap | 2000 | ‰ | F7, intensity and each tint channel |
| Snow-theater red and blue | −128 | colour steps | claim 15 |
| Airborne threshold | 208 | leptons | Terms; 2 height levels of 104 leptons |
| Hit distance | 0 | leptons | claim 10 |
| Field light detail requirement | none | detail level | claim 13; counts at every level |

## State and ordering

A field's states, per field update (S-0203 row 13) and detonation:

| State | Event | Result |
|---|---|---|
| none at cell *X* | radiation-weapon detonation in *X* | new field centred on *X* (claim 2), contributions added, light on |
| live, centre *X* | radiation-weapon detonation in *X* | radiation added (claim 4): F5 removal, new level, duration and steps, timers restart, contributions of *L*′ added, light set afresh |
| live | radiation-weapon detonation in another cell it reaches | unchanged; a second field is made there (claim 2) |
| live | field update | *T* − 1; a decay step if due (claim 5); a light step if due (claim 14) |
| live | field update with *T* reaching 0 | as the row above, then ended: light removed, centre free, cells keep any remainder (claim 6) |

Ordering stated as consequences: claim 11 (decay before the turn pass's hits, fields made in the
turn pass, self-healing before the hit) and claim 6 (the first field update of a field made before
row 13 falls in its own tick). The cell-radiation result must match the result of accumulating the fields' contributions in the row-13 order specified by S-0203; any difference between orders is observable only through rounding of the cell sums. An implementation may use any internal processing order that produces that result.

## Edge cases and quirks

- **Overlapping fields.** Cells reached by several fields hold the sum of their contributions,
  each field subtracts its scheduled amount (F3), and the cap of F6 applies to the sum.
  A clamp after an overdraw can leave less than another field's scheduled remainder.
- **Topping up a field.** The usual case is a deployed infantry unit firing again at its own cell
  (S-0215 claim 31): each shot adds to the field centred on that cell (claim 4) instead of making a new
  one, so the field's level is its decayed level plus the shot's, and its life restarts.
- **Height.** A field on a hill gives less to the cells below it, and none to cells far enough below
  or above it (claim 3); its light ignores height (claim 13).
- **Water, bridges, cliffs.** Fields form anywhere a projectile detonates (claim 1); naval units
  are hit like other vehicles (claim 8); a unit on a bridge deck and a unit under it read the same
  cell (claim 8).
- **Spread below 0** (bad data): a warhead's cell spread that truncates below 0 gives a field
  that reaches no cell and whose light reaches no cell. The field still exists, so it can be added to
  and S-0215 claim 31 reads its level. In the original the light of such a field still reaches the
  centre cell with a negative radius; our engine's light reaches no cell (presentation only; no
  switch).
- **Level cap 0 or less:** ℓ ≤ 0 always, so fields never damage (claim 9).
- **Radiation-site warhead absent:** the original's behaviour is unknown (Open questions). Our
  engine: the importer warns, and fields make no hits (a departure, no switch).
- **Not-a-number cell radiation** (original only): adding to a field with *n*<sub>L</sub> = 0
  divides by zero in the original. Reached cells lose all their old radiation before receiving
  the new contribution, and cells of its square with zero contribution become not-a-number. Those cells then read as exposure 0 and never radiate again for
  the rest of the game. Our engine follows claim 16 instead (a top-up removes no old
  radiation before adding the new contribution), a documented departure for bad data, with no switch.

**Quirk switches**

- `quirks.radiation_add_overdraw` (default on, original): adding to a field removes the amount *r*
  of F5 from the cells of its square, which can exceed the field's remaining contribution by one
  decay step and so take radiation that other fields gave those cells. Off: adding removes exactly
  the field's own remaining contribution from each cell (F5).
- `quirks.radiation_residue` (default on, original): a field's end takes nothing from the cells, so
  a contribution its decay steps did not take stays and keeps damaging (claim 6). Off: when a field
  ends, each cell loses whatever remains of that field's contribution (not below 0), so a cell holds
  radiation only from live fields, up to rounding.

## Import mapping

| Source | Key | Parse | Feeds | Absent |
|---|---|---|---|---|
| rules, `[Radiation]` | `RadDurationMultiple` | integer | duration multiple *M* (F2) | value in force; 0 at a fresh start |
| rules, `[Radiation]` | `RadApplicationDelay` | integer, ticks | application delay *A* (claim 9) | value in force; 0 at a fresh start (claim 16) |
| rules, `[Radiation]` | `RadLevelMax` | integer | level cap *C*<sub>max</sub> (F6) | value in force; 0 at a fresh start |
| rules, `[Radiation]` | `RadLevelDelay` | integer, ticks | level delay *D*<sub>L</sub> (claim 5) | value in force; 0 at a fresh start (claim 16) |
| rules, `[Radiation]` | `RadLightDelay` | integer, ticks | light delay *D*<sub>I</sub> (claim 14) | value in force; 0 at a fresh start (claim 16) |
| rules, `[Radiation]` | `RadLevelFactor` | decimal or percent (S-0003 claims 6 and 7), held as a double | level factor *k*<sub>L</sub> (F6) | value in force; 0.0 at a fresh start |
| rules, `[Radiation]` | `RadLightFactor` | decimal or percent, held as a double | light factor *k*<sub>I</sub> (F7) | value in force; 0.0 at a fresh start |
| rules, `[Radiation]` | `RadTintFactor` | decimal or percent, held as a double | tint factor *k*<sub>T</sub> (F7) | value in force; 0.0 at a fresh start |
| rules, `[Radiation]` | `RadColor` | byte triple (S-0003 C7) | radiation colour (F7) | value in force; (0, 0, 0) at a fresh start |
| rules, `[Radiation]` | `RadSiteWarhead` | warhead name (registers the warhead, S-0004) | radiation-site warhead (claim 10) | value in force; none at a fresh start (Edge cases) |
| rules, weapon section | `RadLevel` | integer, radiation-level units | radiation level of a weapon *L*<sub>w</sub> (claim 1); 0 or less leaves no radiation | 0 |
| rules, warhead section | `CellSpread` | decimal (single), cells; S-0303 owns its main use | spread *s* = trunc of it (claim 1) | 0 |
| rules, unit or building type section | `ImmuneToRadiation` | boolean | immunity test of claim 8 | no |

When the rules file has no `[Radiation]` section at all, none of these game-wide keys is read and
every value in force is kept; game-wide values carry over between games as S-0006 claim 10 says.
The theater of claim 15 is the map's theater (S-0010).

## Measurement scenarios

Every scenario uses the glossary's measurement setup, game speed index 4 and 800×600 capture,
with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). The named test map
*radiation-verification.map* supplies the explicit test overrides below (invented, not stock values).
Use one human-controlled test house and one bystander computer player kept away from the observed
area, the field and all its probe cells; the glossary's void rule applies. Use a test
radiation weapon *W* on a test infantry type or a test vehicle that fires it at a cell, a test
radiation-site warhead with versus 100 % against the test armour and no other effect, and a flat
open test map of the temperate theater unless stated. Base test values: *M* = 2, *A* = 15,
*C*<sub>max</sub> = 10000, *D*<sub>L</sub> = 60, *D*<sub>I</sub> = 60, *k*<sub>L</sub> = 0.0625,
*k*<sub>I</sub> = 0.5, *k*<sub>T</sub> = 1.0, colour (0, 255, 0); *W* has radiation level 1000 and
a warhead with cell spread 2.0 and 0 damage. Then *N* = 2000, *R* = 640, *n*<sub>L</sub> =
*n*<sub>I</sub> = 33. Map ambient is 500 ‰, map tint is white, height terms are 0, and no global
lighting effect runs except the storm explicitly required by RF5 part b. Health is read from the health-bar value or the recorder; ticks from the tick
counter. Probe units have strength 100000 unless a part states otherwise, no healing or veterancy,
armour factors 1.0 and no other source of damage. Infantry probes have prone factor 1.0 and are
kept stationary; void a spatial test if any probe leaves its named cell. Use the read-only recorder
for exact health and tick values, and screen capture for lighting. One run each unless stated.
Field maintenance itself takes no draws; firing, damage and death may do so.

- **RF1 (making, cadence, amount; claims 1, 2, 9, 10).** A test infantry unit *V*,
  not immune, stands on cell *X*; *W* fires once at *X*. *Observable:* the ticks at which *V* loses
  health and each loss. *Expected:* losses only on ticks that are multiples of 15, each 62 at first
  (ℓ = 1000, 1000 × 0.0625 = 62.5 truncated), the first on the first multiple of 15 after the
  detonation (or the detonation's own tick if it is one and *V*'s turn comes after the shot's).
  *Falsified if:* a loss of 63, losses on other ticks, or no loss. Variant b: make the test armour's
  versus 50 %: losses of 31 (S-0302 claim 6). Variant c: level factor 0 but positive exposure:
  the zero-amount hit still loses 1 health through S-0302 claim 5's minimum; no loss falsifies
  delivery of that hit. Variant d: cap 800, expecting losses of 50 even at the centre; 62
  falsifies the cap. Variant e: weapon level 0, then −1, each on a clean map, expecting no light
  and no radiation loss; either effect falsifies the positive-weapon-level gate.
- **RF2 (spread; claim 3).** As RF1 but with seven test infantry units on cells *X*,
  *X* + (1, 0), *X* + (2, 0), *X* + (1, 1), *X* + (2, 1), *X* + (2, 2) and *X* + (3, 0), on flat ground.
  *Expected* first losses: 62, 37, 12, 27, 6, none, none (contributions 1000, 600, 200, about 434,
  about 106, 0, 0; the written 600 and 200 are approximate after floating-point rounding). Variant b: *X* + (1, 0) one height level (104 leptons) higher than *X*: *d* is the
  table root of 256² + 104² (276), contribution 568.75, loss 35. *Falsified if:* any other loss
  pattern, in particular a loss at *X* + (2, 2) or 37 at the raised cell (a horizontal-only distance).
  Variant c: spread 2.9 gives the same pattern as 2.0; any difference falsifies whole-cell
  truncation. Spread 0 instead reaches only *X*; any neighbour loss falsifies that limit.
- **RF3 (decay and end; claims 5, 6, 11).** RF1's *V* stays for 2100 ticks. *Expected:* the loss per
  hit is 62 until the first decay step 60 ticks after the shot, then falls each 60 ticks (60 after one
  step: ℓ = 969); the last positive loss comes before the 33rd step at 1980 ticks, after which there
  are no losses; the field's light (RF11) disappears 2000 ticks after the shot. A hit tick that is
  also a step tick already shows the lower loss. *Falsified if:* the loss falls every tick, the
  losses stop at 2000 ticks instead, or a step tick shows the old loss.
- **RF4 (adding; claims 2, 4, 7, `quirks.radiation_add_overdraw`).** Test values as base, but
  *D*<sub>L</sub> = 1500 (so *n*<sub>L</sub> = 1 and no step falls in the first 1500 ticks).
  Part a: *W* fires at *X* at tick *t*₀ and again at *X* at *t*₀ + 300 (*T* = 1700, field level 850,
  new level 1850, new duration 3700). *Observable:* *V*'s loss on *X* after the second shot and the
  tick the field's light disappears. *Expected:* 115 (1850 × 0.0625 = 115.625, truncated),
  falling to 57 at the first restarted level step 1500 ticks after the second shot, not 1500
  after the first shot; light gone 3700 ticks after the second shot. *Falsified if:* 125 (the original level 1000 + 1000 added), or 62, or the light ending 2000
  ticks after the first shot. Part b: a second *W* fires at *Y* = *X* + (1, 0) at *t*₀ + 10, making
  a second field; a unit *V*′ stands on *Y*. Before the top-up *V*′ loses 99: the first field's
  contribution is just below 600, so the sum is just below 1600, exposure is 1599, and the amount
  truncates to 99. After *W* tops up the field at *X* (as part a, at *t*₀ + 300), it loses 94
  with the switch's original behaviour (exposure 1509 after floating-point subtraction and
  addition), and 131 with exact removal (exposure 2109). A different before-top-up loss or
  a post-top-up loss other than 94 falsifies the stated original result. Part c: repeat part a, but give the second shot a spread of 0, keeping its level 1000.
  A probe on *X* + (2, 0) must still lose 23 just after the top-up and the light keeps its old
  radius; no loss there or a shrunken light falsifies retention of the original spread.
  Part d: use S-0215 scenario B19's immune deployed infantry fixture on *Y*, with its secondary
  radiation level 1000 and thus threshold trunc(1000/3). Before its first eligible guard
  processing, make a field only at neighbouring *X*, so *Y*'s exposure exceeds the threshold
  but *Y* is no field's centre. Expect its otherwise-legal own-cell shot to go ahead. In a
  control with a field already centred on *Y* and current level above the threshold, expect
  that shot to be withheld. Suppression by overlap alone, or firing in the centred control,
  falsifies the centre-field-level test.
  Part e: use S-0215 scenario B19 case e's immune deployed infantry fixture, with an
  immediately detonating secondary radiation weapon. Record the deciding guard processing's
  tick, the centre field's existence and level, and cell radiation before and after that
  processing with the read-only recorder. Test first on a clean cell, then with a live field
  centred there whose current level is below the firing threshold. Repeat with different
  named-test fire frames and deployed-fire frame counts, keeping firing legal.
  Expected: the new field or claim 4's top-up is effective in the deciding processing's tick
  in every run. A change postponed until an animation fire frame or a later tick falsifies
  the claim. This tests field timing, not an exemption from RF1's radiation-hit cadence.
- **RF5 (residue; claim 6, `quirks.radiation_residue`).** Base values but *D*<sub>L</sub> = 3000
  (*n*<sub>L</sub> = 0). *W* fires at *X*; *V* stays on *X* for 2500 ticks. *Expected:* losses of 62
  every 15 ticks throughout, also after the field's light disappears at 2000 ticks; a new shot at *X*
  after that makes a new field (light reappears) and the loss becomes 125 (2000 × 0.0625). *Falsified
  if:* losses stop at or before 2000 ticks. Part b: set both delays to 100 and *A* = 1. Arrange
  for a storm bolt to kill an exploding test vehicle whose death weapon is *W*, before the field
  update in that tick. Keep a high-health probe on the same cell and record the death tick *t*₀.
  The field ends at *t*₀ + 1999 after only 19 level steps; its remaining centre exposure is about
  50 and continues causing hits of 3 after its light disappears. No post-end hits falsifies this
  same-tick-creation residue case. Keep other storm strikes away; discard a run with another hit
  on the probe or another field at its cell. Part c: use part a's delays and top up the live field
  before it ends. Then fire *W* at *Z* = *X* + (2, 2), a square corner beyond the first field's
  reach. In the original, a probe on *Z* takes no radiation damage even from the new field centred
  on *Z*, although its light appears; a radiation loss there falsifies the persistent
  not-a-number result. Our engine instead follows claim 16 and permits the new field to damage.
- **RF6 (who is hit; claim 8).** Base values, a field at *X* reaching its neighbours. In it: a test
  building of the test armour; a test vehicle transporting *V*; *V*₂ of an immune test type; a test
  aircraft landed on the ground; a test jumpjet infantry unit flying over *X*; a test vehicle on a
  bridge deck over an irradiated cell (on a map with a bridge, field made under the deck). *Expected:*
  the building, *V* inside the transport, *V*₂ and the flying jumpjet lose nothing; the transport,
  the landed aircraft and the deck vehicle lose health on the hit ticks. *Falsified if:* any other
  pattern.
- **RF7 (credit and crew; claim 10).** Part a: A test unit of strength 100 is killed only by field damage from a
  field made by a test infantry firer of another house. *Observable:* the firer's experience and the
  kill counts of both houses at game end (S-0414). *Expected:* the firer gains no experience, and no
  house is credited with the kill. *Falsified if:* the firer's house or the firer is credited.
  Part b: repeat with a non-exploding crewed vehicle of strength 100, no passenger capacity and
  no hijacker, on an open cell. The named map sets crew escape chance to 1. In 20 radiation kills,
  expect no crew; in 20 otherwise identical ordinary-weapon kills, expect crew. A crew from
  radiation falsifies the no-escape marker; failure of the ordinary-kill control voids the test.
- **RF8 (order in a tick; claim 11).** Part a: *A* = 1 (every tick a damage tick). Test unit *P*
  stands on *X* before the shot. *W*'s projectile is fired at *X* from far enough away to fly for
  several ticks, and while it is in flight a second test unit *Q* leaves a test factory and is moved
  onto *X*, so *P* comes before the projectile in the update order and *Q* after it. *Expected:* in the detonation tick *Q*
  loses health and *P* does not. Part b: a self-healing test vehicle at full strength in a field with
  *A* equal to the self-healing period (S-0314 claim 9). *Expected:* after the first common
  hit/heal tick it is below full strength by the whole hit. *Falsified if:* *P* is hit in the
  detonation tick, *Q* is not hit, or the vehicle recovers any of that first tick's loss. For part b choose only individual self-healing (S-0314 claim 9),
  explicitly set its period to 15 ticks and begin at full health immediately before a common
  hit/heal tick; record that first common tick. Later ticks need not start at full health.
- **RF9 (which detonations; claim 1).** Part a: a projectile with cluster count 2 carrying *W*:
  expected two lit fields at the two chain points (RF11's observable), or one field with an added
  level when both points share a cell. Part b: an airburst projectile whose airburst weapon has
  radiation level 1000 and whose own weapon has 0: expected no field at all. Part c: an exploding
  test vehicle whose death weapon is *W* is destroyed: expected a field at its cell. Part d: *W*'s warhead given
  the parasite flag: expected a field. *Falsified if:* part b shows a field, or a, c or d none.
- **RF10 (no draws; claim 12).** With the random-state recorder, record the raw values drawn in the
  ticks around a decay step and a field's end, and in the tick of a shot of *W* compared with an
  identical shot of a weapon with radiation level 0. Keep damageable objects out of both fields,
  and suppress projectile damage and optional detonation effects with the named test data, so
  radiation hits and deaths cannot add draws. Compare draw counts, not raw values from separately
  seeded games. *Expected:* equal event-local draw counts for the shots, and no draws in row 13's
  place around decay or ending. Also repeat a same-centre top-up. *Falsified if:* any extra draw
  attributable to creation, top-up, decay, light changes or ending.
- **RF11 (light; claims 13 to 15).** The pixel colour of the ground at *X* and at *X* + (2, 0) is
  measured as in S-0104's lighting scenarios, before the shot, after it, after a light step and after
  the end, at low and at high detail. Part a (intensity): colour (0, 0, 0) and *k*<sub>T</sub> = 0,
  so the field light has no tint and the cells keep the map's own tint. *Expected:* at both detail
  levels the ground level at *X* rises by ⌊*I*·*f*/1000⌋ = 500 ‰ (*I* = 1000 × 0.5, *f* = 1000 at
  the centre, S-0104 F "Cell light"), and at *X* + (2, 0) by ⌊500·*f*/1000⌋ with *f* = 200; after one
  light step the intensity is 485 (500 − ⌊500/33⌋); after the end, the original lighting. Part b
  (tint): base colour and tint factor. *Expected:* the raw cell tint of *X* (S-0104 Terms) is the map
  tint plus (0, 1000, 0) ‰, and plus (0, 970, 0) after one light step (⌊1000 × 1940/2000⌋); the
  drawn colour follows from S-0104's normalisation of that raw tint. Part c: part b on a
  snow-theater test map: raw cell tint the map tint plus (−501, 1000, −501) ‰ at the centre.
  Part d: repeat part b with the light factor 0, so intensity stays 0 while only green should
  fade. Our engine must show green 970 after the first step; the original instead keeps green
  1000 because of claim 14's drawing-only defect. Capture each step and the end.
  *Falsified if:* part a's rise or step differs from its stated value, low and high detail differ,
  part b's centre tint does not follow F7, part c lacks its red and blue reduction, or the original
  in part d fades green on that first step. These exact-value checks use the read-only light/tint
  recorder; pixel capture alone may quantise different values to the same shade.
- **RF12 (bad divisors and signed cadence; claim 16).** Separate runs with the named map:
  (a) *A* = 0 and a living infantry unit on the map, expecting a program error on its first turn,
  with no radiation weapon required; (b) base values but level delay 0, expecting an error at the
  first field's making; (c) light delay 0, likewise; (d) base values but *W*'s level 10, so duration
  20 ticks is shorter than light delay 60, likewise; (e) RF1 with *A* = −15, expecting the same
  hit ticks as with +15. Each stop claim is falsified if the game continues past the stated event,
  part e is falsified by a different cadence.

## Open questions

1. **Missing radiation-site warhead.** What the original does when a field hits with no warhead
   is `unmeasured`, confidence low: the receiving object kinds' missing-warhead paths are not
   established here, so no expected original outcome is specified. Our engine makes fields
   deal no hits then (Edge cases), a design rule rather than an observation.
2. **Large light radii.** As in S-0104 claim 11, light reach at radii of 32000 leptons or greater
   remains unestablished. This does not change the radiation contribution formulas. This range
   is `unmeasured`, with low confidence: no large-radius fixture or expected light-reach result
   is established here.

## Questions for the project lead

1. **Snow-theater colour (claim 15).** The original replaces red and blue by −128 on snow maps,
   which looks like a sign slip for +128 but could be a deliberate green shift. It is presentation
   only. Should our engine keep the original look (this draft) or use +128?
