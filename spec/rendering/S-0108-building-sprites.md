---
id: S-0108
title: Building sprites
topic: rendering
revision: 102
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-30
measurements: [M-0007, M-0014]
requests: [Q-0540, Q-1550, Q-1610, Q-1620, Q-1772, Q-1890, Q-2001]
supersedes: []
---

# S-0108: Building sprites

## Summary

How a building's sprite is put together: which sprite and image frame the body shows (build-up,
normal, damaged, occupied, gate stages, selling, unloading), where the body is anchored relative to
its foundation, the shadow, the bib and the door image, and the slot animations attached to a
building. For the slot animations it covers the slots, the damaged and occupied variants, the power
behaviour, and the building-specific rules that start and swap them (refinery and silo fill,
infantry absorption, superweapon charge, turret, repair depot), and what slots do on a building that
needs an engineer before and at its ownership changes (claims 26 to 28). When a slot is started
while it already holds an animation, the new animation keeps the replaced one's stage and takes its
own passes afresh (claim 25); that the stage, not the image frame, is what carries over when the two
types' start fields differ is unmeasured and rated medium there. Claims 26 to 28 answer Q-1772,
and claim 28's building placed switched off by the map answers Q-1890. The superweapon slots of
a building whose owner's record of the type is not granted (claims 11 and 18) answer Q-2001.
Skirmish only; network games and campaigns are out of scope.

## Terms

*Tick*, *lepton*, *foundation* (with its top-left cell), *facing*, *game speed* (index *s*),
*image frame* and *claim* are used as in the glossary. The terms below are specific to this spec.

- **Body sprite:** the building's main shape file, with *N* image frames. The first ⌊N/2⌋ are body frames
  and the rest are the matching shadows. **Build-up sprite:** a second shape file, played while the
  building is being built or sold, with *N_b* image frames split the same way.
- **Image name:** as in the glossary (S-0007): it names the type's art section, which holds the
  type's art keys.
- **Tech level:** the type's imported tech-level field (import mapping), an integer.
- **Refinery:** here, a building whose type has the refinery flag, not every docking building of
  the harvest-cycle spec. **Repair depot:** here, a type with the unit-repair flag. Both flags are
  listed in Import mapping.
- **[P]:** 1 when the condition P holds, else 0.
- **Health ratio** *h*: current health divided by full strength (Formulas F3).
- **Damaged:** *h* ≤ the yellow condition ratio. Equality counts as damaged. **Badly damaged:**
  *h* ≤ the red condition ratio.
- **Body animation state:** one of six, each with an image-frame range (start *a*, count *c*, delay *d*):
  *construction*, *idle*, *active*, *full*, *aux-1*, *aux-2*. Which behaviour puts a building in
  which state belongs to the production, repair and power specs. Here only the image-frame ranges matter.
- **Body stage** *k*: the building's stage counter, with the same timer rule as infantry stages
  (S-0107 claim 4). It grows by one every *d* ticks of the current state's range, and *d* = 0
  freezes it.
- **Occupants:** infantry garrisoned inside an occupiable building. **Occupied:** at least one
  occupant.
- **Held infantry:** infantry taken inside a building that absorbs infantry (claim 17). Held
  infantry are not occupants: they never make a building occupied.
- **Recycling building:** a building that consumes entrants for credits (S-0603 claim 1).
- **Slot animation:** an independent animation drawn on top of the body and kept in one of the
  building's 21 **slots**: *idle*, *active-1*, *active-2*, *active-3*, *active-4*, *pre-production*,
  *production*, *turret*, *special-1*, *special-2*, *special-3*, *special-4*, *super-1*, *super-2*,
  *super-3*, *super-4*, *low-power*, *super-low-power*, *upgrade-1*, *upgrade-2* and *upgrade-3*.
  A slot holds at most one running animation. These have nothing to do with the map overlays of the
  glossary.
- **Slot definition** (per type, per slot): a normal animation name, a damaged name and an occupied
  name (each may be empty), a screen displacement (*x*, *y*) in pixels, a depth adjustment and a
  y-sort value (both used by S-0102), and four power flags: **freeze-unpowered**,
  **light-unpowered**, **effect-unpowered** and **special-outage**.
- **Starting a slot** with a variant: create the animation named by the chosen variant. If the slot
  already holds an animation, it is replaced. Claim 25 says what the new animation keeps from the
  replaced one (its stage) and what it does not (its count of passes).
  Nothing happens when the chosen name is empty, or when it names no animation type. **Stopping a
  slot:** remove its animation.
- **Variant choice** at a start: the name used depends on the building and on the start.

  | Building | Start allows occupied variants | Name used |
  |---|---|---|
  | damaged | either | damaged name |
  | not damaged, occupied | yes | occupied name |
  | not damaged, occupied | no | normal name |
  | not damaged, not occupied | either | normal name |

  The claims say which starts allow occupied variants. The chosen name is used as it stands, with no
  further fallback at the start. A damaged or occupied name that the art leaves empty is already
  filled with the normal name when the type is read (import mapping), except in the upgrade slots,
  so an empty damaged name at a start is possible only in an upgrade slot or in a slot with no names
  at all.
- **Slot condition:** per building, *damaged* or *normal*: the condition its slot animations were
  last started or swapped in (claim 12). A start with the occupied variant counts as *normal*.
- **Finishes on its own:** a slot animation reaches its natural end. Being stopped or replaced is not
  finishing.
- **Delayed-fire type:** a type whose art is marked delayed-fire (import mapping).
- **Turret or barrel art in voxel form:** the type's turret or barrel art is marked as a voxel model
  (import mapping).
- **Powered:** here, the building passes the in-operation test of S-0405 claim 8. This is not
  merely the absence of a house power shortfall. **Needs and draws power:** the type is marked as needing
  power and its power field is negative (it consumes power).
- **Door sprite:** an optional shape file the type's art names, drawn while the building unloads
  (claim 23). **Door-stage count** *D*: the type's imported door-stage field, an integer.
- **Depot processing delay** *D*<sub>p</sub>: the repair depot's processing delay, the *D* of
  S-0314 claim 6 (claim 20 and scenario B18). It is unrelated to the door-stage count.
- **Damaged-door type:** a type whose art is marked as having damaged door frames.
- **Door state:** the state of the building's door, one of *closed*, *opening*, *open* and
  *closing*. Factory door movement, timing and unloading behaviour belong to S-0417
  (structures); the door-image rules remain here. **Door progress** *q*: the completed fraction of the door's current movement,
  from 0 to 1, and 1 when the door is not moving (Formulas F8).
- **Theater flag, new-theater flag, theater refresh:** as in S-0007 (claims 11 and 12).

## Behaviour

1. **Which sprite the body uses.** A building in the *construction* state, or one whose
   construction has not yet completed, draws its build-up sprite. Otherwise it draws its body sprite.
   When the sprite to be used does not exist (for example, a type with no build-up sprite while it
   would use one), nothing of the body is drawn: no body, bib or shadow. A building whose type is
   marked invisible in game draws no body, bib, shadow, unloading, under-door or door image. Its slot animations are separate
   objects and are not hidden by this.
   - Backing: `measurable` (B1 for sprite choice and invisible body, bib and shadow).
     `unmeasured` for missing sprites (no construction setup with a missing build-up is established)
     and invisible unloading/door images (B1 does not unload from the invisible test building).
   - Confidence: high (the sprite choice; an invisible type's missing body, bib and shadow), medium
     (the missing-sprite case, `unmeasured`; whether such a building ever enters the *construction*
     state belongs to the production spec), medium (an invisible type's missing unloading, under-door
     and door images, `unmeasured`)

2. **The body frame.** The image-frame index *f* is an integer. These situations are mutually
   exclusive: construction takes precedence over type properties. Outside construction the
   occupiable property takes precedence over the ordinary gate body-frame rule. A gate whose current
   behaviour is *open* and whose door is opening, closing or at rest open uses claim 4's
   **special gate image** instead. A moving door under any other behaviour does not select that image.

   | Situation | Image-frame index *f* |
   |---|---|
   | Construction, not selling, not a gate | *k* |
   | Construction, selling, not a gate | *a* + *c* − *k* − 1, using the construction range (*a*, *c*); the build-up plays backwards |
   | Gate in construction | Claim 4 |
   | Occupiable type, outside construction, not using the special gate image | 2·[occupied] + [hurt] |
   | Non-occupiable gate, outside construction, not using the special gate image | 0 when not damaged, *G* + 1 when damaged; *G* is the gate-stage count |
   | Any gate outside construction, current behaviour *open*, door opening, closing or at rest open | Claim 4 |
   | Any other building, not damaged | *k* |
   | Any other building, damaged, *idle* state | *k* + 1 |
   | Any other building, damaged, other state | *k* + the maximum of (*a* + *c*) over *idle*, *active*, *aux-1* and *aux-2* |

   For an occupiable type, hurt means badly damaged when the tech level is 0 or below, and damaged
   when it is above 0 (`quirks.occupiable_hurt_rule`). An occupied, hurt building uses frame 3,
   including a civilian building whose tech level is −1. This is a drawing-only departure. The
   damaged-frame maximum excludes *full* (`quirks.full_state_unmaxed`). The four included ranges
   default to *a* = 0, *c* = 1, giving the normal frame + 1 with default art.
   - Backing: `measurable` (B1 for construction/selling, B2 for ordinary damaged idle frames,
     B3 for occupation, B13 for gates); B3's occupied, hurt civilian frame is a departure
     tested against this spec, not the original. The non-idle damaged maximum and overlapping type
     properties are `unmeasured`: no scenario here establishes those states. The exclusion of
     *full* shares that limitation.
   - Confidence: high for the rows that B1, B2, B3 and B13 test; medium for the non-idle damaged
     maximum, the exclusion of *full* and the precedence between overlapping type properties
     (`unmeasured`)

3. **Image-frame clamp at draw time.** For a sprite with at least one body frame, the body frame
   actually drawn is min(*f*, ⌊N/2⌋ − 1), where *N* is the image-frame count of the sprite in use.
   A request past the body half shows the last body frame and its corresponding shadow (claim 9),
   a drawing-only departure. With no body frames, no body or shadow is drawn.
   The clamp does not apply to the bib (claim 7). An
   image-frame index at or past the image-frame count of any sprite
   (body, shadow, bib, unloading sprites) draws nothing for that image. A negative index also
   draws nothing; it is not clamped upward to image frame 0.
   - Backing: `measurable` (B12 for the body clamp, a departure tested against this spec,
     not the original). The no-body-frame result is a defined departure.
     Other out-of-range images are `unmeasured` here: no bib or unloading sprite with an
     out-of-range requested index is supplied by B12. Negative door indices are `measurable`
     through B20; other negative image indices are `unmeasured` here because no scenario requests them.
   - Confidence: high (the clamp), medium (nothing drawn for an index past the sprite)

4. **Gates.** A gate's build-up has *G* + 1 image frames, whatever the sprite holds (except for a gate
   with the theater flag after the theater refresh, claim 5). In the *construction* state a gate
   runs the opposite way to other buildings. During build-up it shows *f = a + c − k − 1*, so its
   build-up plays from the last build-up frame to the first. While it is being sold it also shows
   *f = a + c − k − 1*, playing the selling build-up backwards like other buildings, a drawing-only
   departure that does not alter the stage, range, delay or sale completion tick.
   Outside construction, the special gate image is used only while the current behaviour is
   *open* and the door is opening, closing or at rest open. Its image frame from the body sprite
   is *p* when not damaged and *p* + *G* + 1 when damaged, where *p* is the integer door position
   clamped to 0 … *G* − 1. At rest open, *p* = *G* − 1, so the image frame remains *G* − 1
   undamaged or 2*G* damaged. Progress during motion, its floating-point rounding, reversal and
   zero-time images are S-0215 claim 35 d1–d3. The clamp of claim 3 does not apply to the special
   gate image, and no bib, unloading or under-door image is drawn with it. When the door is at
   rest closed, or *open* is not current even though the door is moving or at rest open, claim 2's
   ordinary body-frame rules apply. Door timing and passage state belong to S-0215 claim 35.
   - Backing: `measurable` (B13 for build-up reversal, moving and rest-open gate frames,
     S-0215 claim 35 d1–d3 and its scenario B28 h–j for progress, reversal and zero-time images).
     Selling reversal is a departure tested against this spec, not the original.
     Suppression of bib/unloading/under-door images and bypass of the body clamp are
     `unmeasured` here: B13 supplies no gate with those extra images or a body half shorter
     than its gate range. The ordinary-image fallback under another behaviour while the door
     still moves or stands open is `unmeasured`: no fixture here establishes that combination.
   - Confidence: medium

5. **Build-up timing.** Let *n* be the number of build-up frames: ⌊N_b / 2⌋ for a build-up sprite of
   *N_b* image frames, or *G* + 1 for a gate. The construction range is *a* = 0, *c* = *n*, and its delay
   is *d_b* = trunc(*B* × 900 / *n*) ticks, with *B* the imported build-up time in game minutes
   (Formulas F1). When *n* ≤ 0, *d_b* = 1. A type with no build-up sprite keeps the default
   construction range *a* = 0, *c* = 1, *d* = 0. The build-up is reported finished on the tick on
   which the stage reaches *n* − 1, the last build-up frame, that is (*n* − 1) × *d_b* ticks after
   the state began when *n* ≥ 2 and *d_b* ≥ 1. This expression does not give the completion time
   of a zero-delay range: its stage does not advance, but it still reports completion on a building
   turn. S-0406 claim 15 owns construction handover: in uninterrupted play a zero-delay range
   hands over one tick after the first construction processing, including the default range of a
   type without a build-up sprite. For the positive-delay multi-stage case, handover occurs on the
   report tick. S-0406 claim 17 gives the first construction processing relative to placement or
   deploy. In uninterrupted play a placed building with at least 2 stages and positive delay shows
   its build-up twice: stage 0 appears again one tick after the first completion report, and
   handover follows the second run (`quirks.placed_buildup_twice`, owned by S-0406). From placement
   tick *T*, the restart is at *T* + (*n* − 1) × *d_b* + 1 and handover is at
   *T* + 2(*n* − 1) × *d_b* + 1. Positive-delay ranges with one stage or fewer never report
   completion (S-0406 claim 18, including `quirks.single_stage_buildup_never_completes`).
   Selling uses the same range and delay, backwards including gates (claim 4), but need not play
   the full range: a type naming a vehicle to pack into, without a guard spot, can complete on
   advancing stage 23 before the last stage (S-0408 claim 17). That claim owns sale completion
   timing. A building whose type has no build-up sprite has no build-down, because a sell order
   does not sell it (S-0408 claim 4); a building that packs up has its build-down as in any sale
   (S-0415 claim 9). The construction delay is never
   game-speed normalised when the state is entered (but see claim 6 for the wrap).

   The construction range is set whenever the build-up sprite is loaded, with *B* as it is held at
   that moment: at every type read that loads the sprite (S-0007 claim 11), and again at the theater
   refresh for a building with the new-theater flag, which reloads all its sprites as at a type read
   (S-0007 claim 12).

   **Theater-flag buildings.** When the theater refresh reloads the build-up sprite of a building
   with the theater flag (S-0007 claim 12) and the file resolves, the construction range becomes
   *a* = 0, *c* = *N_b* and *d_b* = ⌊75 / *N_b*⌋ ticks, an integer quotient truncated toward zero, or
   1 when *N_b* = 0 (Formulas F1). *N_b* is the whole image-frame count of the sprite, shadow half
   included, and a gate gets the same rule. The build-up time *B* plays no part
   (`quirks.theater_refresh_buildup_range`). Because the stage then runs past ⌊*N_b*/2⌋, the clamp of
   claim 3 holds image frame ⌊*N_b*/2⌋ − 1, the last body frame with its corresponding shadow,
   for the second half of the build-up; the displayed frame is a drawing-only departure,
   while the construction range and timing stay unchanged. When the file does not resolve, the building has no build-up sprite (S-0007 claim 12),
   and the range keeps its earlier value.
   - Backing: `measurable` (B1 for positive-delay ordinary build-up, B13 for gates, B19 for
     theater-flag build-up, with the clamped displayed frame tested against this spec as a
     departure, not the original; S-0406 PL11 a to f for placement/deploy timing, the two placed
     build-up runs, zero-delay completion and ranges that never complete; S-0408 S10 for
     stage-23 sale completion). Failed reload retaining the range
     is `unmeasured` (B19 supplies the file; a missing sprite hides its range).
   - Confidence: high (the delay), medium (the exact tick of the switch to the body sprite and the
     theater-flag rule), medium (a failed reload keeping the range, `unmeasured`)

6. **Body animation ranges and game speed.** A state change is either *immediate* or *deferred*.
   It is immediate when the building has no state yet, when the new state is *construction*, or
   during the game's set-up at game start. Every other change is deferred. The table gives the
   stage and the delay the building runs with after each event, where (*a*, *c*, *d*) is the range
   of the state it is then in, and N(*d*, *s*) is the game-speed normalisation of S-0107,
   Formulas F3.

   The original gives the *full* range no defined initial value and no import key, so its frame
   sequence and timing are unspecified if that state is entered. Our engine uses *a* = 0 image
   frames, *c* = 1 image frame and *d* = 0 ticks for *full*, a deterministic departure: entry sets
   *k* to 0 and the stage remains there. Claim 2 still determines the damaged frame. This range
   also supplies the *full* term when `quirks.full_state_unmaxed` is off. It is not an imported
   value, and does not change when other ranges change.

   | Event | Stage | Delay |
   |---|---|---|
   | Immediate change, new state not *construction*, art marked "normalised" | *a* | N(*d*, *s*) |
   | Immediate change, any other case | *a* | *d* |
   | Deferred change takes effect, new state equal to the current one | unchanged, keeps running | unchanged |
   | Deferred change takes effect, new state *construction* or *idle* and different from the current one | *a* | N(*d*, *s*), whether or not the art is marked "normalised" |
   | Deferred change takes effect, any other new state different from the current one | *a* | raw *d*, whether or not the art is marked "normalised" |
   | Stage passes the end of the range (*k* ≥ *a* + *c*): it wraps | *a* | N(*d*, *s*) for *construction* and *idle*, raw *d* otherwise (`quirks.building_wrap_normalisation`) |

   An immediate change applies even when the new state equals the current one, and it cancels every
   deferred change requested before it that has not yet taken effect. A deferred change requested by the building's own behaviour is visible on the request tick. A deferred change requested by anything else is visible either on the request tick or on the following tick: it is visible on the request tick if the request is processed before the building's update, and on the following tick otherwise. When several
   deferred changes are requested before any of them takes effect, only the last one requested
   takes effect.

   These speed-dependent body stages are among the
   exceptions that glossary claim G3 lists. Our engine reproduces them by default
   (`quirks.speed_normalised_delays`, shared with S-0107). With that switch off, every delay that
   this table gives as N(*d*, *s*) is the raw *d* instead, so a body stage lasts the same number of
   ticks at every game speed, and `quirks.building_wrap_normalisation` then has no effect. For a building with a turret the wrap happens only while
   it is being built or sold (`quirks.turret_building_no_body_wrap`). Otherwise its stage keeps
   growing past the range, and the clamp of claim 3 then applies.
   - Backing: `measurable` (B11 for deferred idle entry and ordinary idle wrapping).
     `unmeasured` for other entry rows, repeated-state requests, cancellation, last-request-wins
     timing and turret no-wrap: B11 establishes no event path selecting those cases. The
     original's unspecified *full* range is `unmeasured`: no skirmish event entering it is
     established and no repeatable range can be predicted. Our *full* range is a defined departure.
   - Confidence: high (deferred entry into the idle range and ordinary wrapping of the idle range,
     the two rules B11 covers), medium (`unmeasured`: the other entry rows, repeated-state requests,
     cancellation, the last-request-wins rule and the turret case)

7. **Bib.** When the type has a bib sprite, the bib is drawn whenever the building is not in the
   *construction* state, so never during build-up or selling. It is also not drawn with the
   special gate image (claim 4), including at rest open while *open* is current. It uses the same image-frame index *f* as the body (so image frame 1 when
   damaged, with default art), without the clamp of claim 3. Its depth bias is in S-0102.
   A protected building's bib, under-door, replacement and door images take the protection
   appearance of S-0513 claim 29b.
   - Backing: `measurable` (B4 for ordinary bib visibility and default damaged frame,
     S-0513 claim 29b and its SW10 part i for protection appearance).
     The unclamped index is `unmeasured`: B4 never requests an index above the body clamp.
     Special-gate-image suppression shares claim 4's limitation.
   - Confidence: high for the bib's visibility, its default damaged frame and the protection
     appearance; medium for the unclamped index (`unmeasured`) and for the special-gate-image
     suppression, rated as claim 4 rates it

8. **Anchor of the image.** The body, build-up, bib and shadow frames are drawn centred on the
   screen projection of the foundation's top corner, its topmost point on the screen. That
   point lies half a cell, 128 leptons, before the centre of the top-left cell along both map axes.
   The image is independent of the foundation size: art for larger foundations simply extends
   further down the screen and to both sides.
   Depth images and depth adjustments belong to S-0102.
   - Backing: `measurable` (scenario B5)
   - Confidence: medium

9. **Shadows.** Unless the type suppresses shadows, the shadow of the drawn body frame *F* (after
   the clamp of claim 3) is image frame *F* + ⌊N/2⌋ of the same sprite, drawn at the same position as a
   darkening, with its own depth bias (S-0102). For a body frame clamped to ⌊N/2⌋ − 1,
   its shadow index is 2⌊N/2⌋ − 1, valid for both even and odd *N* with at least one body frame.
   No game option or data turns building shadows off for all buildings.
   The per-type flag is the only control, as for units (S-0107 claim 19).
   - Backing: `measurable` (B12 for shadow indices and per-type suppression,
     shadows of clamped body frames are tested against this spec as a departure, not the original).
     The assertion of no global option is `unmeasured`: the scenario tests the type flag,
     not every possible setting.
   - Confidence: high (the shadow frame), medium (no shadow past the sprite), medium (no global
     option, `unmeasured`)

10. **Slot animation placement.** A slot animation is created at the image anchor of claim 8 (the
    foundation's top corner), moved by the slot's screen displacement (*x*, *y*) in pixels, which
    is converted to world units by the projection spec. Its depth adjustment and y-sort come from
    the slot definition (S-0102). While the building is being sold, every slot
    animation is hidden from the first tick of its build-down until it is removed. Before that
    tick the sale hides none of them; what the loss of operation at its first processing does to
    them is claim 28's (the row *not building or selling*). A building that packs up is sold with
    a build-down (S-0415 claim 9), so its slot animations are hidden from the first tick of that
    build-down in the same way. A sell order to a building whose type has no build-up sprite
    starts no sale (S-0408 claim 4): no slot animation is hidden and none changes, and a building
    of such a type with the firestorm-wall flag is removed at once, its slot animations with it.
    - Backing: `measurable` (scenario B14 for the placement and for the pack-up; B22 part (h) for
      the slots staying drawn until the build-down: its per-tick capture tells this from hiding
      for the whole sale on the one tick between the sale's first processing and the build-down's
      start, S-0408 claim 6, even when no docked unit holds the sale; B14's screenshots need not
      catch that tick). A type with no build-up sprite rests on S-0408 claim 4 and its scenario
      S3, parts c and d, which show that no sale starts; with no sale there is nothing to hide
      the slots.
    - Confidence: medium until B22 part (h) runs; medium for the pack-up until B14's pack-up run
      has run; medium for a type with no build-up sprite, and low for the firestorm-wall case, as
      S-0408 claim 4 rates them.

11. **Grand opening.** A building's grand opening brings it into service once: on construction
    completion in play, during placement at game set-up, or when trigger action code 125 (create a
    building) creates the building in play. These slots start:
    - For a building that is not a refinery: *idle* (occupied variants allowed), *active-1*
      (occupied variants allowed), and *active-2*, *active-3* and *active-4* (no occupied variant).
    - For a refinery: only the *active* slot of its fill level (claim 15), with no occupied
      variant. Grand opening does not itself start *idle* for a refinery, nor stop slots
      already running.
    - For a building with a superweapon (its first superweapon only): when its owner's record of
      that type (S-0512 Terms) is granted at that moment, *super-3* if the record's remaining
      charge time is below 15 ticks, otherwise *super-1*. When the record is not granted, its
      clock is read in the same way (`quirks.ungranted_super_reads_clock`). A record never
      granted and never reset reads 0, so *super-3*. A record reset while it is not granted
      (S-0512 claim 15) reads its clock as the reset left it. A lost record (S-0512 claim 7) reads
      its clock as the loss left it, since the loss neither stops nor resets the clock (S-0512
      Terms give a running clock that counts down every tick, and claim 7 does not stop it): a clock
      running at the loss goes on counting down and reads 0 once it has run out, and a stopped
      clock (S-0512 Terms) keeps the remaining time it had at the loss. Whether the record was
      ready at the loss does not matter: a ready record whose clock has run out reads 0, and one
      whose countdown a set-charge action restarted while it stayed ready (S-0512 claim 14) reads
      that countdown. The
      charge that S-0512 claim 7 forgets is the restart from the full *T* at the next grant.
      With the switch off, a record that is not granted gives *super-1*. Both allow occupied
      variants.
    - For a building with a turret whose turret art is not voxel art: the *turret* slot. This start
      always uses the normal name and counts as "not damaged". As a result, a building that is
      damaged when it completes shows the normal variant on every running slot animation, including
      those started at grand opening, as if repaired (claim 12,
      `quirks.turret_start_clears_damaged`).

    Grand opening also creates the building's free unit when eligible under S-0402 claim 18.
    That claim owns the eligibility, placement, failed-placement refund and initial behaviour.
    This is not an unconditional unit grant.

    A building placed during the game's set-up at game start starts the *idle* and *active-1* …
    *active-4* slots each time its body state is set during the set-up, all with occupied variants
    allowed. Its placement gives it the *idle* body and queues *guard*, then runs the grand
    opening described above. The opening sets readiness, but *guard* becomes current only at the
    first per-tick start (S-0215 claim 1). Thus a set-up refinery can retain *idle* and the other
    *active* slots started with its body state, alongside the slot selected by its fill level.
    A set-up superweapon building or sprite-turret building also receives its corresponding
    grand-opening slot starts. The normal-name turret start can swap the variants of the slots
    already running, just as at construction completion. Set-up grants no free unit
    (S-0402 claim 18).

    A building that trigger action code 125 creates in play has its grand opening at once, as part
    of that action, in the tick the action runs. Its slots listed above start then, and its free unit
    is created then when S-0402 claim 18 makes it eligible; such a building was not produced, so
    it has no start price. Its placement also queues the construction behaviour, as any placement
    in play does (S-0215 claim 1). A later completion of that behaviour, if it comes, is no second
    grand opening: it starts and restarts no slot and creates no second free unit. What the
    building shows while that behaviour lasts, and for how long, is an open question.

    A replacement that trigger action code 130 (restore starting buildings) creates does not open
    in the action's tick. It enters with its ready mark clear and *construction* queued, as
    S-0524 claim 15 states, and its grand opening is the ordinary one at the completion of that
    construction, with the slot starts listed above.

    Whether the record is granted at a grand opening follows from S-0512. Building-grant
    conditions and their check timing follow S-0512 claims 2 and 3: the house is not defeated, a
    giving building of the type stands, and for a disableable type the superweapon setting is
    on. A record that a trigger action granted (S-0512 claims 12 and 13) is met at an opening as
    any granted record. The rest of this paragraph is about building grants. When those
    conditions hold, a building placed by order in play is granted at
    its house's next house update, so after a build-up that ends later its record is granted at
    the grand opening, with its clock started at the grant. When they fail, for example for a
    disableable type with the setting off, no building grants the record.
    During set-up placement the only building-grant check is the one that an upgrade's placement
    makes at once. This is an inference from S-0512 claim 2 (c), under which a building of the
    house gaining an upgrade makes a check, and S-0009 claim 40, under which the map's upgrades
    are placed at set-up right after their building. So a set-up opening meets a granted record only
    when, earlier in the set-up, an upgrade of the same house was placed while a giving building
    of the type stood and that check granted the type. The opening then reads that record as any
    granted record, with its clock started at that grant. Every other set-up opening meets no
    record that a building granted. A code-125 building meets a granted record only if its house
    already holds the type. An opening that meets a record that is not granted reads its clock
    as above. This holds in every house. Each house, the neutral and special houses (glossary,
    *house*) included, holds one record of each type from its creation (S-0512 claim 1), so no
    house is without a record, and the neutral and special houses are granted at their grant
    checks as any house is. A pre-placed building whose owner matches no house never enters play
    (S-0009 claim 27). Whether code 125 can create a building with no owner house is not
    established (Open questions; confidence low). With
    `quirks.ungranted_super_reads_clock` off, a record that was lost and one never granted are
    the same case: *super-1*, whatever charge a lost record had left.
    - Backing: `measurable` (B23 for a building placed in play meeting a granted record at its
      opening, part (f) for a set-up opening after an upgrade's grant, and parts (b), (c), (e)
      and (f) for the original's slot at openings that meet a record never granted and never
      reset; part (g) for
      an opening that meets a lost record, its clock running or stopped at the loss; part (h)
      for the neutral and special houses). The switch-off *super-1* for those is `unmeasured` (a
      design decision, Edge cases; no measurement of the original decides it). `unmeasured` for
      a record ready at the loss (no part loses a ready record before an opening that meets a
      record that is not granted), for a record
      reset while not granted (no part resets one) and
      for using only the first superweapon: no part has a building with two superweapons. That
      a pre-placed building whose owner matches no house never opens rests on S-0009 claim 27.
      B6 for
      unoccupied
      completion starts, set-up slot presence and the turret case, S-0215 claim 1 for queued
      *guard* and readiness,
      S-0402 scenario H13 for the free unit, B22 part (a) for the slots of a building created by
      code 125 starting in the tick of the trigger action and not restarting later in the run,
      except that a replacing start that keeps the stage (claim 25) shows the same frames, so that
      no such start comes is `unmeasured` (no input is known that tells the two apart),
      B22 part (j) for its free unit and for no second one within the run).
      `unmeasured` for a later completion of the construction behaviour being no second grand
      opening: no run here establishes that such a completion comes (Open questions). The
      code-130 replacement rests on S-0524 claim 15 and its scenario.
      `unmeasured` for occupied grand-opening variants (normally no occupants at completion),
      below-15-tick charge at completion (B6 establishes no such setup), and repeated set-up
      body-state changes beyond the placement's idle entry (no input path selecting each
      additional set-up state is established). Unknown-name starts and empty
      upgrade names are also `unmeasured`: no start path for those slots is established here.
    - Confidence: high for the unoccupied completion starts, the set-up slot presence, the turret
      case, the queued *guard* and the free unit; medium for the grand opening of a building created
      by code 125, its slots starting in the tick of the action, its free unit and no second
      opening within the run, until B22 parts (a) and (j) run; medium for a later completion
      being no second opening (`unmeasured`); medium for the occupied grand-opening variants,
      the below-15-tick charge case, repeated set-up body-state changes, unknown-name starts and
      empty upgrade names (`unmeasured`); medium for the slot starts of a code-130 replacement at
      the completion of its construction, which rest on S-0524 claim 15's entry state; high for
      the switch-off *super-1* at an opening that meets no granted record, as decided (Edge
      cases); medium for the original's reading of a record that is not granted, until B23 parts
      (b), (c), (e), (g) and (h) run, and for a record ready at the loss, a record reset while not
      granted and the first-superweapon rule (`unmeasured`); medium for a building
      placed in play meeting a granted record at its opening and for a set-up opening after an
      upgrade's grant, until B23 parts (a) and (f) run; low for code 125 with no owner house
      (Open questions)

12. **Damage swaps the variants.** Whenever the building's health changes through damage or repair
    and its damaged state then differs from its slot condition, the slot condition takes the new
    value and every running slot is restarted, keeping its stage (claim 25), with the damaged name when now
    damaged and the normal name when not. A slot whose name for the new condition is empty keeps
    its current animation. The occupied variant is never chosen by this swap
    (`quirks.damage_swap_drops_occupied`). The swap takes effect on the tick of the damage or
    repair. Starting a slot in the condition opposite to the slot condition triggers the same swap
    of every running slot, on the tick of the start, and the started slot shows the variant chosen
    for that start. The running slots are restarted one after another in this fixed order:
    *upgrade-1* to *upgrade-3*, *active-1* to *active-4*, *pre-production*, *production*,
    *turret*, *special-1* to *special-4*, *super-1* to *super-4*, *idle*, *low-power*,
    *super-low-power*. Each new animation takes its creation draws (S-0109 claim 3) as its slot is
    restarted, so the draws of one swap come in that slot order. The order shows in which
    creation draws each restarted slot takes and in the order in which the new animations enter
    play (S-0203 claim 3). When a start in the opposite
    condition triggers the swap, the swap of every running slot comes first and the started
    slot's own new animation is created after it. A started slot that was already running, and
    whose name for the new condition is not empty, is therefore restarted by the swap and then
    replaced again by the start, keeping the same stage
    both times, and with a random rate it takes the creation draws twice. After a condition swap, existing slots show normal or damaged art, except a slot with an
    empty name for that condition, which keeps its old art until something restarts it. A slot
    started with an occupied variant still shows that variant until a later restart or swap:
    counting it as the normal slot condition does not replace its occupied art.
    - Backing: `measurable` (B2 for damage/repair swaps and occupied art being dropped,
      B6 for the turret start's opposite-condition swap). Empty-condition upgrade slots are
      `unmeasured`: B2's ordinary empty damaged name is filled with the normal name at import,
      so it cannot test retaining an old variant on an empty-name swap. The order of the restarts
      and of their creation draws, and the started slot's second creation, are `unmeasured`: no
      scenario here gives two restarted slots a random rate, and the order shows only in the
      random state. What a restarted slot keeps (its stage) and counts afresh (its passes) is
      claim 25, with its own backing.
    - Confidence: high for the swap itself and its timing; medium for a slot with an empty name
      for the new condition keeping its current animation (`unmeasured`), and for the order of
      the restarts, their draws and the started slot's second creation (`unmeasured`). What the restarted
      slot keeps takes claim 25's ratings: high where the two variants share a start field,
      medium (`unmeasured`) for keeping the stage rather than the image frame where they
      differ.

13. **Power.** For each slot only the first of its power flags in the order *freeze-unpowered*,
    *light-unpowered*, *effect-unpowered* counts. *freeze-unpowered* is on for every slot unless the
    art turns it off (import mapping), so *light-unpowered* or *effect-unpowered* takes effect only in
    a slot whose art sets *freeze-unpowered* off. The slots that have no power-flag keys
    (*pre-production*, *production*, *turret* and the three upgrade slots) always count
    *freeze-unpowered*. A building's powered state here is its in-operation result (every row of
    S-0405 claim 8, claim 28). A building reacts to a change of that state in its next update, so
    on the same tick as the change or the next one, and it reacts once to each change. A newly
    placed or completed building that is not in operation has no power-loss reaction; one that is
    in operation has its power-return reaction in its first update. "Power is lost" below is a change from in operation to not
    in operation, and "power comes back" the reverse. Both columns apply only
    to a building that needs and draws power.

    | Counting flag | Power is lost | Power comes back |
    |---|---|---|
    | *freeze-unpowered* | A running slot holds its image frame and stops advancing. | It resumes. |
    | *light-unpowered* | A running slot is stopped. If it is *special-1* of a delayed-fire type whose *active-1* has *freeze-unpowered*, *active-1* starts (and, being unpowered, starts frozen). | If the slot is not running and the building has completed, it starts. The *special-1* slot of a delayed-fire type is the exception. |
    | *effect-unpowered* | A running slot is stopped and marked as stopped by the outage. If it is *super-3*, *super-low-power* starts. | Marked slots start and are unmarked. |
    | any | | *super-low-power* stops. |

    Slots started by these rules use the normal or damaged variant, never the occupied one.

    A *freeze-unpowered* slot that starts while the building is disabled or not in operation (any
    row of S-0405 claim 8, not only its powered row) starts frozen. This applies only when the type
    itself needs power. Separately, disabling a building (for any reason the disabling spec
    defines) freezes every running *freeze-unpowered* slot, whether or not the type needs power.
    Re-enabling it unfreezes them, even when the building is still unpowered
    (`quirks.enable_unfreezes_unpowered`). The end of a pulse is such a re-enabling whatever the
    building's power switch: a building that is switched off when its pulse ends has its frozen
    *freeze-unpowered* slots resume. The power-loss column applies only at a change from powered to
    unpowered: a building that has not been powered since it was placed or completed has had no
    such change, so its *light-unpowered* and *effect-unpowered* slots keep playing (claim 27).

    Each disabling and each re-enabling acts at once, at its own place in the tick. A building's
    own reaction to a change of its in-operation result, in its next update, acts on the same
    slots: for a type that needs and draws power, the power-loss column freezes its
    *freeze-unpowered* slots and the power-return column resumes them. When several of these
    events reach a building in one tick, its *freeze-unpowered* slots are afterwards frozen or
    playing as the last of them leaves them, the building's reaction counted among them. For a
    type that does not need and draw power there is no such reaction, and the disablings and
    re-enablings alone decide. The disablings are a switch-off (the map's placement of a
    building in the switched-off state, claim 28, among them), a pulse's start, and the map's
    placement of a building of claim 26 that is not manned. The re-enablings are a switch-on, a
    pulse's end, and an ownership change of a building whose type needs an engineer (claim 26).
    An ownership change of any other building is neither. Their places are S-0203's: a trigger
    action acts where its trigger fires, the polled triggers before any object's turn (S-0203 row
    1); an engineer's capture and a pulse's start act in the turn pass (row 5), one after another
    in update order (S-0203 claim 3); a pulse's end acts before the orders of its tick; and a power
    order, which a player gives only for a building of their own house (S-0214), acts after the
    turn pass (row 8, S-0203 claim 7). So, in one tick, for a type that does not need and draw
    power, and for one that needs and draws power, has no reaction still due from an earlier tick
    and whose in-operation result is the same after the tick as before it:
    - A switch-off order applied in the tick of an ownership change that gives the building to the
      order's house, or of a pulse's end, leaves the slots frozen. Whether an order is applied when
      a change earlier in its tick has given the building away from the order's house is open
      (Open questions).
    - A switch-off by a polled trigger's action in the tick of a pulse's end, or of the capture of
      a building whose type needs an engineer, leaves them playing, although the building is
      switched off (`quirks.enable_unfreezes_unpowered`).
    - A pulse that starts in the tick of an ownership change made by a polled trigger's action
      leaves them frozen. A pulse that starts in the turn pass that also holds a capture leaves
      them as the later of the two in update order leaves them.
    - A switch-on order in a tick in which a pulse holds the building is refused (S-0405 claim
      10) and changes no slot.

    For a type that needs and draws power whose in-operation result changes in the tick, the
    building's reaction is one more event in that sequence. For example, take such a type that
    needs an engineer and is in operation before the tick. A switch-off by a polled trigger's
    action takes it out of operation, and a capture later in the turn pass re-enables it but
    leaves it switched off (S-0405 claim 10). If the building's turn comes before the capture, its
    power-loss reaction freezes the slots and the capture then leaves them playing. If its turn
    comes after the capture, the capture leaves them playing and the reaction then freezes them.
    In the same way, a pulse's start and a capture in one turn pass on such a type in operation
    before the tick leave the slots frozen when the building's turn comes after both, since its
    reaction is then the last event.

    Claim 28 applies these rules to a building that the map places switched off.
    - Backing: `measurable` (B7 for freeze/light/effect flags and super-low-power transitions, B11
      for disable/re-enable while unpowered and, in its pulse run, for a pulse ending on a building
      still switched off, B22 part g for a building never powered, B22 parts (a), (e) and (k) for
      disabling and re-enabling a type that does not need power, and part (f)'s short-power run for
      a capture that leaves a building not in operation, which is no change). `unmeasured` for
      delayed-fire outage exceptions and occupied-variant exclusion: no scenario exercises those
      cases or establishes a discriminating event path. `unmeasured` for the power-return reaction
      of a building that is in operation when it is placed or completed: no scenario judges it
      against no reaction. For disablings and re-enablings close together: `measured` by M-0014
      alone, one run each (*n* = 1), in order cycles that B22 parts (n) and (o) do not name: with
      the switch-off order in the cycle of tick *t* − 2, before an ownership
      change that re-enables the building (the change leaves the slots playing although the
      building stays switched off), with the order in the cycle of tick *e* − 4 (the pulse's end
      re-enables a building switched off under the pulse), and with the order in the cycle of tick
      *e* + 1, for a later switch-off that freezes the slots again, each
      for a type that needs an engineer, draws power and does not need power. M-0014's runs, one
      each (*n* = 1), with the order in the cycles of *e* − 3, *e* and *e* + 2 that part (o)
      names, and in that of *e* + 5, which M-0014 alone adds, meet the same expectations
      (M-0014): `measured`, with B22's validity shown only as far as M-0014's end-of-tick lists
      of buildings, vehicles and infantry show removals
      (M-0014, Evidence), for these runs as for the others. With the order in the cycle of
      *t* − 1 (part (n)), M-0014 shows the slots keeping their ordinary schedule, with an advance
      at *t* + 2, and the building enabled after the change while switched off: `measured` for
      that end state; no hold is shown. That they hold from such an order until the change is
      `unmeasured`: the hold would last one tick, less than the slots' stage delay, so no frame
      tells it from no hold, and it rests on the freeze at a switch-off shown with the order in
      the cycle of *t* − 2. For several in one tick: `unmeasured`
      for a switch-off order applied in the tick of an ownership change that gives the building to
      the order's house, since a cycle takes the player's input before its tick runs (S-0201,
      *cycle*), while the building is still another house's, and S-0214 offers power mode only over
      an own selectable building, so no player's order can be given in that tick. A switch-off order
      applied in the tick of a pulse's end is `measured` (M-0014) by part (o) with
      the order in the cycle of tick *e*: the light's state shows again at the end of the tick in
      which the pulse's hold reaches 0, so a recorder that reads that state fixes the tick, and
      the slots show no new frame after the end of tick *e*. `unmeasured`
      for the other combinations, since no scenario here
      can put a capture and a pulse's start in a chosen update order, and no spec states a
      trigger action that switches a building. `unmeasured`
      for the building's own reaction as one more event in that sequence, for a type that needs
      and draws power whose in-operation result changes in the tick: no scenario here can put the
      building's turn before or after a capture or a pulse's start at will, and parts (n) and (o)
      use a type that does not need power, so no reaction comes there.
    - Confidence: high for the power-loss and power-return rules as they act at a change from
      powered to unpowered and back; medium for the disable and enable cases, whose triggers belong
      to other specs, for the same-or-next-tick timing, for the pulse ending on a building still
      switched off, for a type that needs power, until B11's pulse run runs (high for a type
      that does not, `measured` by M-0014's part (o) order in the cycle of *e* − 4), and for the
      rule that a building not powered since
      it was placed or completed has had no such change, until B22 parts (f) and (g) run; medium
      for the power-return reaction of a building in operation when placed or completed
      (`unmeasured`, above); high for disablings and re-enablings close together with the order in
      the cycles of *t* − 2, *e* − 4, *e* − 3, *e*, *e* + 1, *e* + 2 and *e* + 5, the switch-off
      order in the tick of a pulse's end among them, and for the end state after the change
      with the order in the cycle of *t* − 1 (the slots on their ordinary schedule and the building
      enabled while switched off, M-0014); medium for the hold from an order in the
      cycle of *t* − 1 (`unmeasured`, above); medium for the other
      cases of several in
      one tick, the switch-off order in the tick of an ownership change (`unmeasured`, above)
      among them, which rest on the same rule and on S-0203's places in the tick; medium for the
      building's reaction counted among those events (`unmeasured`, above), which rests on the
      power columns, the disable and enable rules and the reaction's timing in its next update

14. **Special outage.** For a type marked for special power behaviour, whether or not it needs
    power: when power comes back, the *low-power* slot stops and every slot with *special-outage* is
    started (normal or damaged variant), running or not. When power is lost while the owner's timed
    power blackout is running (the blackout a spy causes by entering a power plant), or while the
    owner's power is marked as being drained, *low-power* starts and every *special-outage* slot
    stops. An ordinary shortfall with neither condition changes neither.
    - Backing: `measurable` (B15 for timed blackout versus ordinary shortfall).
      `unmeasured` for drain blackout, special-power types not needing power and restarting a
      running special-outage slot: B15 does not exercise those cases.
    - Confidence: medium. S-0405 claim 5 owns drain blackout (a drained output building).

15. **Refinery fill.** For a refinery, the fill level is *L* = 0 when *m* = 0, and otherwise the
    integer quotient of 4·*m* by *S* (Formulas F4), where *m* is the stored ore amount truncated
    to an integer and *S* is the type's storage capacity, an integer. Level 0 selects *active-1*, 1
    *active-2*, 2 *active-3*, and 3 or more *active-4*. When *L* changes, the slot of the old level
    stops and the slot of the new level starts (normal or damaged variant). The check runs every
    tick. When *S* < 0, the quotient truncated toward zero is 0 or negative, and a negative level
    selects no slot: the slot of the old level stops and no slot starts. When *S* = 0 and *m* > 0 the original divides by zero and the program fails. Our engine
    uses *L* = 0 in that case (a departure), and the importer warns about any refinery type whose storage
    capacity is 0 or less (Edge cases and quirks).
    - Backing: `measurable` (B8 for empty storage selecting level 0). All nonempty-storage
      cases are `unmeasured`: no skirmish path filling a building's own storage is known (S-0402
      claim 12, S-0408 claim 10); harvesting pays credits directly. This includes the original's
      zero-capacity failure and negative-capacity result. Our zero-capacity result and warning
      are defined departures.
    - Confidence: high (empty storage, *m* = 0); medium (all nonempty storage, `unmeasured`)

16. **Silo fill.** For a type marked to show silo fill, the level is
    *L* = trunc(4·*m* / *S* + 0.5), clamped to 0 … 3 (Formulas F5). It is 0 when *S* ≤ 0. At level 0
    the *special-1* slot is stopped. Otherwise *special-1* is started if absent (normal or damaged
    variant), and its displayed image frame is forced to *L* on every tick. The animation's own timing
    does not move it, and the setting wins over any hold of the slot's frame (claims 13 and 26).
    - Backing: `measurable` (B8 for *special-1* absent with empty storage). That level 0 stops a
      running *special-1* is `unmeasured`: B8 never makes the slot run first, so it cannot tell
      stopping from never starting. Claim 13's power-return row could start it on art that sets
      the matching flag, but no scenario here sets that up.
      `unmeasured` for positive fill, its rounding and forced image frames, including the forced
      frame winning over a hold: no known skirmish
      path fills building storage. The missing-animation failure under Edge cases shares this
      limitation; our safe result and warning are defined departures.
    - Confidence: high (*special-1* absent with empty storage); medium (level 0 stopping a running
      *special-1*, and positive fill, its rounding and forced image frames, the forced frame
      winning over a hold included, `unmeasured`)

17. **Infantry absorption.** For a type that absorbs infantry and gives a positive extra power per
    held infantry, once the building has completed and is not in the *construction* state, this is
    checked every tick. The total size *T* of the held infantry is the integer of Formulas F9. For
    sizes of 0 or more, *T* is the sum of the sizes, each truncated toward zero to an integer.
    When *T* < 1, *active-2*
    is stopped and *active-1* is started if absent. When *T* ≥ 1, *active-1* is stopped and
    *active-2* is started if absent. Both starts allow occupied variants. A building that holds only
    infantry whose sizes are each below 1 therefore shows the empty-building slot.
    The owner's deploy order can release held infantry while the building survives (S-0605
    claims 11 to 13). The same periodic check then selects the slot for the remaining held size.
    - Backing: `measurable` (B16 for non-negative sizes, the two sub-unit-size passengers and
      voluntary release using S-0605 claims 11 to 13).
      `unmeasured` for occupied variants (held infantry do not garrison) and negative sizes
      (their ordering is not characterised; Open questions).
    - Confidence: high (non-negative sizes and release); medium (occupied variants and negative
      sizes, `unmeasured`)

18. **Superweapon charge cycle.** For a building with a superweapon (its first superweapon only),
    this is checked every tick while it is not being built or sold. The check runs only when the
    type's charged-animation time *t_c* (game minutes, a single-precision floating-point field) is
    at most 990. Let *t_r* be the weapon's remaining charge time in game minutes, computed as remaining
    ticks × *κ*, where *κ* is the single-precision constant nearest 0.0011111111 (exactly
    2 386 093 / 2³¹) (Formulas F6).
    - When *t_c* < *t_r* and *super-3* is running: *super-3* stops and *super-4* starts.
    - When *t_c* ≥ *t_r* and *super-1* is running: *super-1* stops and *super-2* starts.
    - When *super-2* finishes on its own, *super-3* starts. When *super-4* finishes on its own,
      *super-1* starts.

    The check reads the owner's record of the type whether or not it is granted
    (`quirks.ungranted_super_reads_clock`): while it is not, *t_r* comes from its clock as for a
    granted record, and a record never granted and never reset gives *t_r* = 0. Once the record is
    granted with
    remaining time *T* (S-0512 claim 3), the checks after the grant use *t_r* from that clock.
    So a *super-3* started at an opening without a grant gives way to *super-4* at the first
    check after the grant at which *t_c* < *t_r*, and *super-1* starts when *super-4* finishes.
    With the switch off, the check also needs the record to be granted and changes nothing while
    it is not, so a *super-1* started at an opening without a grant changes only when
    *t_c* ≥ *t_r*.

    All these starts use the normal or damaged variant. The four slots therefore form a loop:
    charging, opening, charged, closing.
    - Backing: `measurable` (B9 for the four-slot cycle and its threshold; B23 part (e) for the
      checks after the grant). In (e) the record was never granted, so a check before the grant
      reads *t_r* = 0 and changes nothing, with the switch on or off, since B23's *t_c* is not
      negative. Such a check cannot be told apart from no check, which gives the same result, so it
      is rated with the checks after the grant. A check that reads a lost record's clock before the
      record is granted again is `unmeasured`: that window lasts until its house's next update, and
      the one part with a building in it, B23 (g), uses type A, whose check never runs. The
      switch-off pause while the record is not granted is `unmeasured` (a design decision, Edge
      cases; no measurement of the original decides it). `unmeasured` for the 990-minute cutoff and
      ignoring a second superweapon: B9 supplies neither a cutoff comparison nor a two-superweapon
      building.
    - Confidence: high (the four-slot cycle and its threshold; the switch-off pause while the
      record is not granted, as decided); medium (the checks after a grant, until B23 part (e)
      runs; a lost record's reading before its grant, the 990-minute cutoff and ignoring a
      second superweapon, `unmeasured`)

19. **Turret slot animation.** For a building with a turret, the *turret* slot animation does not
    advance on its own. Every tick its image frame is set to (28 − *j*) mod 32, a value from 0 to 31,
    where *j* is the turret facing rounded to 32 directions as in S-0107 Formulas F2
    (*j* = (⌊φ / 1024⌋ + 1) div 2 mod 32, with φ the turret facing in facing units). The turret sprite therefore stores 32 directions
    counter-clockwise, with image frame 0 at turret facing 28 × 2048. The setting wins over any
    hold of the slot's frame (claims 13 and 26). When the turret art is voxel art,
    the *turret* slot is not started at grand opening. When the barrel art is voxel art, a *turret*
    slot animation that does start is hidden.
    - Backing: `measurable` (B17 for the sprite-turret formula; B22 part (i) for the setting
      winning over a hold). The voxel turret/barrel suppression
      is `unmeasured` here: B17 has neither voxel-art flag variant.
    - Confidence: medium (the turret facing is defined in S-0206, Terms; the setting winning over
      a hold, until B22 part (i) runs)

20. **Repair-depot, recycling and delayed-fire slot rules.** These apply only to the building kinds named in
    each row. A repair depot here includes a naval yard with the repair flag (S-0504 claim 8); only
    the every-tick row below excludes a naval type. *In contact* means the depot holds a contact with a
    vehicle in any of its contact places (S-0314 claim 17), tethered or not.

    *The repair cycle.* A repair depot also starts and stops slots at the events of the repair
    cycle of S-0314 claim 6 (waiting, settling, working). Each change happens on the tick of the
    depot processing at which the event occurs, and every start uses the normal or damaged
    variant, never the occupied one.

    | Building | Event | Result |
    |---|---|---|
    | Repair depot | it enters the working phase: its settling step answers *more needed* or *done* | *special-1* starts (a running one is replaced, claim 25); *active-1* and *idle* stop |
    | Repair depot in the working phase | a step answers *done*, *not needed* or *cannot pay*, or the depot is no longer in contact | *production* and *special-2* stop; *special-3* and *active-1* start |
    | Repair depot in the waiting phase | its repair processing finds no contact | *production* and *special-2* stop; *special-3* and *active-1* start; the depot is in *guard* from 1 tick later, while *special-3* runs (S-0314 claim 6) |
    | Repair depot in the settling phase, with *production* or *special-2* running | it is no longer in contact | *production* and *special-2* stop; *special-3* and *active-1* start |

    So *special-1* starts once per entry into the working phase, on the tick of the first paid
    repair step (S-0314 claim 3), and again at each re-entry after a *cannot pay* return to
    settling. Docking, the waiting check, a settling step that answers *not needed* or *cannot
    pay*, and a working step that answers *more needed* change no slot. The repair cycle never
    stops *special-1*: leaving the working phase does not by itself end that animation.
    Natural completion or external removal selects *special-2* or *idle* as the rows below say.
    Stopping or replacing the slot does not trigger that follow-up. Claim 13's power rules,
    claim 12's condition swaps (which restart a running slot) and removal of the building can
    also affect it.
    When a working step sends the vehicle away (the working-phase row of the table above), the
    slot changes happen on the tick of that step. A settling
    step that sends the vehicle away changes no slot of its own (*After a send-away* below).
    When contact ends without a step because the vehicle is destroyed (S-0314 claim 22), the depot
    reacts at the processing that S-0314 claim 6's spacing table had already scheduled; the loss
    does not bring it forward. When contact ends because the vehicle leaves (S-0314 claims 20 and
    21), the same is expected, rated as S-0314 claim 22 rates it (medium in working and settling,
    its scenario H4 part h runs 8 and 9). (Q-1610, answered in part through S-0314: its questions
    1 and 2 for a destroyed vehicle only, and its third question for a depot back in *guard*; the
    rest stays open.)
    In the working phase this is the same or the next tick when the depot misses no turn in
    between (S-0203 claim 10), except during the *D*<sub>p</sub> ticks after the
    settling step that started working; in the settling and waiting phases it is the pending
    processing of S-0314 claim 6's table. S-0314 claim 22 gives the cases, and owns the cases for
    a vehicle that takes contact later (Q-1610's third question), including its unresolved
    settling-hold case. These timings, and the no-contact hold below, are for a depot that is
    not also a vehicle bunker, as S-0314 claim 6's table is; for that type they are open (Open
    questions).
    *After a send-away (Q-1620).* A working step that sends the vehicle away has already made its
    changes and put the depot in settling. If the depot's next processing, 1 tick later when the
    depot misses no turn (S-0314 claim 6's table), finds no contact,
    only the settling row above can apply, and it acts only if *production* or *special-2* is
    running at that processing:
    - after a working step that answers *done* or *not needed* and sends the vehicle away, both
      were stopped by that step if they were running, and nothing in the repair cycle restarts them without a contact.
      Unless something outside the repair cycle starts one of them before that processing (see
      *What else starts these slots at a depot* below), it makes no second change: the running
      *special-3* is not replaced, and *active-1* is not started again: the processing itself
      starts no slot. A slot animation that ends on its own on that tick can still start a slot
      under its own row (a *special-1* ending out of contact starts *idle*, which part h run 3
      relies on).
    - after a departure that fails (contact kept, the depot in settling), *special-1* can end
      while the vehicle is still in contact and so start *special-2* (row below). A later settling
      step that answers *not needed* sends the vehicle away without any slot change of its own.
      If *special-2* is then running at the following no-contact processing (*D*<sub>p</sub> ticks later),
      that processing stops *production* and *special-2* and starts *special-3* and *active-1*,
      replacing a running *special-3* and a running *active-1* (each start makes a new animation
      that keeps the replaced one's stage, with its passes counted afresh, claim 25).
      If neither *production* nor *special-2* is running, it makes
      no change.

    The rule depends only on which slots are running at that processing, not on whether the
    previous step sent a vehicle away. A settling depot with no contact stays in its repair
    behaviour from its first no-contact processing, the one already scheduled, checking every
    tick from then on, until *special-3* is no longer running, and only then returns to *guard*
    (S-0314 claim 22 owns that return; see Changes for other specs). The simulation models the slot animation's duration
    from the art data as deterministic simulation state; this wait does not depend on drawing
    frames or whether the building is visible.

    *Other repair-depot, recycling and delayed-fire rows.*

    | Building | Event | Result |
    |---|---|---|
    | Repair depot, not naval, not in its repair behaviour, with *production* or *special-2* running | checked every tick | *special-3* starts (normal or damaged variant), *production* and *special-2* stop |
    | Repair depot in its repair behaviour and in contact | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *special-2* starts (normal or damaged variant) |
    | Repair depot not in its repair behaviour, or not in contact | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *idle* starts (normal or damaged variant) |
    | Repair depot | *special-3* finishes on its own | *idle* starts (normal or damaged variant) |
    | Delayed-fire type that is not a repair depot | *special-1* finishes on its own | *active-1* starts (occupied variants allowed) |
    | Recycling building | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *active-1* starts (damaged variant when the health ratio is at or below the yellow ratio, F3; normal otherwise), provided the selected variant exists (S-0603 claim 12) |

    No building other than the kinds listed reacts to *special-1* or *special-3* ending.

    *What else starts these slots at a depot.* Apart from the repair cycle, other claims can start
    a depot's *special-1* to *special-3* when the art sets the matching flags, for example claim
    13 (its *light-unpowered* and *effect-unpowered* rows on power return) and claim 14 (a
    *special-outage* slot); roles the type may combine with the repair flag (delayed fire, silo
    fill, recycling, vehicle bunker of S-0606) add their own starts. Nothing in the repair cycle starts *production*: only the construction-yard,
    vehicle-factory unloading and refinery-docking roles do, under their own specs. The rows that
    stop *production* therefore matter only for a type that combines one of those roles with the
    repair flag.
    - Backing: `measurable`, by these scenarios:
      - B18 parts a to c and e: the repair-cycle rows other than the waiting-phase loss and the
        settling contact-loss row; the working-entry tick, *active-1* and *idle* stopping, the
        *cannot pay* restart, a destroyed vehicle, the naval yard, and *production* never
        appearing at a depot without a role that starts it.
      - B18 parts a, c and e: natural completion during repair or after contact loss; part f:
        delayed fire.
      - B18 part g: the settling hold and the return to *guard*.
      - B18 part h: the processing after a send-away, and the settling contact-loss row with
        *special-2* running.
      - B18 part i: the every-tick row with *special-2* running, and its exclusion of the naval
        yard; part i run 3: the processing after a working send-away with *special-2* started
        from outside the repair cycle before it.
      - B18 part j: stopping *production*, the every-tick row with *production* running, and the
        settling contact-loss row with *production* running.
      - B18 part k: the exclusion of occupied variants.
      - S-0314 H4 part h: the reaction timing after a loss, `measured` (M-0007, runs 1 to 6).
      - S-0603 claim 12 and scenario G10: recycling-building natural completion.
      Recycling-building restart after external removal is `unmeasured`: no data-only fixture
      establishes removal without the building itself stopping or replacing the slot animation.
      Restart with the selected *active-1* variant absent is also `unmeasured`: no setup establishes
      that absence at completion while allowing *active-1* to run at consumption and start
      *special-1* (S-0603 claim 12).
      What a replacing start keeps and counts afresh is claim 25, with its own backing (B18
      part h run 2 is one of its scenarios).
      Repair-depot non-natural removal of *special-1* is `unmeasured`: no event in play is known
      that removes a running *special-1* other than the building itself stopping or replacing it
      (claim 12's condition swaps replace it), so no setup can produce the case.
      The waiting-phase loss row's start of *special-3* and its *guard* 1 tick later are
      `measured` (M-0007) for a loss by the vehicle leaving on a move order before the depot's
      first waiting processing: the contact ended before that processing, which found none. For
      a destroyed vehicle there they are medium, as S-0314 claim 22 rates them. That run is not a
      B18 part d run, since part d's loss falls with the depot already waiting, and M-0007
      reports it as inconclusive for part d. M-0007 records that *active-1* was already running
      at that processing, and records nothing of *production* or *special-2* there. The row's
      stops of *production* and
      *special-2* are `unmeasured`: no scenario has either running at a waiting-phase loss, and
      the stop of a slot that is not running looks the same as no stop. Its start of *active-1*
      is `measurable` (B18 part d). A loss between two waiting processings is `measurable`
      (B18 part d, with S-0314 H4 part h run 7's set-up).
      The exhaustive list of *production* starters (only the construction-yard, vehicle-factory
      unloading and refinery-docking roles; nothing in the repair cycle) is `unmeasured`: a
      finite set of types cannot establish it. That no building other than the kinds listed
      reacts to *special-1* or *special-3* ending is `unmeasured` for the same reason. That the
      repair cycle never stops *special-1* is `unmeasured` for the same reason too: B18 parts a
      to c, in their repeats with the longer *special-1* art, and part h run 4 (a working step's
      *not needed* answer) show that the events they exercise leave it running, but no finite set of runs
      shows that no other event of the cycle stops it. So is the
      statement that roles the type may combine with the repair flag add their own starts of
      *special-1* to *special-3*: it belongs to the same exhaustive family, and no scenario here
      combines such a role with the repair flag. It is rated low.
      That *production* running at the processing after a send-away makes the same change as
      *special-2* running there is `unmeasured`: no run has *production* running at that
      processing. It is the settling contact-loss row that part j run 2 tests with *production*
      running, and that row depends only on which slots are running.
    - Confidence (the backing of each item is given in the list above):
      - high: the slot starts and stops of the working-phase entry, natural completion of
        *special-1* during contact and of *special-3*, leaving the working phase, the *cannot
        pay* restart, a destroyed vehicle, the naval yard and delayed fire; the normal and
        damaged variants; that a working step's *done* answer and send-away (part a) and the
        vehicle's destruction (part c) leave *special-1* running, each judged in its repeat with
        the longer *special-1* art. The two loss rows of the
        cycle's table take their own ratings: in the waiting-phase loss row, the start of
        *special-3* and the *guard* 1 tick later high for a loss by the vehicle leaving before
        the depot's first waiting processing (`measured`, M-0007's move order), medium for a
        destroyed vehicle there as S-0314 claim 22 rates it, and medium for a loss between two
        waiting processings until B18 part d runs with S-0314 H18 b's tethered vehicle, its start of
        *active-1* medium until B18 part d runs, and its stops of *production* and *special-2*
        medium and `unmeasured` for the reason given above; the settling contact-loss row
        medium until part h runs. That the cycle never stops *special-1* and never starts
        *production* are exhaustive negatives,
        rated with the others below.
      - medium until part b's repeat with the longer *special-1* art runs: that the *cannot
        pay* failing step and restart (part b) leave *special-1* running. That restart's
        replacing of a running *special-1* is medium as claim 25 rates it.
      - medium until B18 parts i to k run, which use named test types that no stock type
        matches and may be reported unreached: stopping *production*; the every-tick row with
        *production* or *special-2* running; the settling contact-loss row with *production*
        running; the processing after a working send-away with *special-2* started from outside
        the repair cycle; the exclusion of occupied variants; and, until B18 part h runs, that a
        replacing start counts its passes afresh.
      - medium, `unmeasured` for the reason given with each item above: the recycling-building
        restart after external removal; the restart with the selected *active-1* variant
        absent; the repair depot's non-natural removal of *special-1*; the exhaustive list of
        *production* starters; that no other building kind reacts to *special-1* or *special-3*
        ending; that the repair cycle never stops *special-1*; that *production* running at the
        processing after a send-away makes the same change (no run has *production* running
        there; it is the settling row that part j run 2 tests with *production* running, and
        that row depends only on which slots are running). Each of these except the three
        exhaustive negatives is one rule that the cited rows state for every case, with no known
        exception. The three exhaustive negatives (the list of *production* starters, that no
        other building kind reacts to *special-1* or *special-3* ending, and that the repair
        cycle never stops *special-1*) are rated medium
        because they rest on a listing of every start of these slots that the original makes,
        not on a sample of types; they stay `unmeasured` because no finite set of runs can show
        that no other start exists.
      - medium until part h runs: that a working step's *not needed* answer leaves
        *special-1* running (part h run 4); the settling contact-loss row with *special-2* running; that
        the processing after a working send-away makes no second change when neither
        *production* nor *special-2* is running; the change after a failed departure and a
        settling send-away with *special-2* running.
      - low, `unmeasured`: that roles the type may combine with the repair flag add their own
        starts of *special-1* to *special-3* (the reason is given in the list above).
      - high, `measured` (M-0007, S-0314 H4 part h runs 1 to 6), for a loss by the vehicle's
        destruction: after a loss in working, the
        tick of the depot's first processing at or after the loss, which in run 2 is the tick of
        the loss itself; after a loss in settling, the tick of the pending processing and the
        return to *guard* at it (runs 3 and 5); and the
        settling hold while *special-3* runs. That a loss in working is met in the next tick
        when the depot's processing falls there rests on S-0203 claim 3, and is rated with it.
      - medium: the tick of the changes when the depot's processing falls on a missed turn, as
        S-0314 claim 22 rates it.
      - medium, `measurable` (S-0314 H4 part h run 8 in working, run 9 in settling): the same
        ticks for a loss by the vehicle leaving, as S-0314 claim 22 rates it.
      - medium, resting on another rule: recycling-building natural completion, which inherits
        S-0603 claim 12's rating of the restart.

21. **Parsed but without effect.** The type's extra-damage-stage flag has no effect on how a
    building is drawn or behaves.
    - Backing: `measurable` (B11 for recorded rendering differences of the toggled type).
      No effect on any behaviour is `unmeasured`: a finite rendering test cannot establish
      absence of every possible gameplay effect.
    - Confidence: high (no drawing difference, B11); medium (no behavioural effect, `unmeasured`)

22. **Unloading sprites.** While a building is unloading a unit (factory unloading belongs to
    S-0417), the sprite depends on the unit being unloaded.
    - **Unit not marked jumpjet or balloon-hover:** when the type has a deploying sprite, it replaces
      the body sprite, with the depth adjustment −20 instead of the type's normal one. Without one,
      the body sprite stays.
    - **Unit marked jumpjet or balloon-hover:** when the type has a roof-deploying sprite, it
      replaces the body sprite, with the depth adjustment −40. Without one, the body sprite stays with
      its normal depth adjustment. The ordinary deploying sprite is not used as a fallback
      (`quirks.roof_unload_no_fallback`).

    In both cases the image-frame index is *f* of claim 2 with the clamp of claim 3. In addition, an
    under-door sprite is drawn with image frame 1 when damaged and 0 otherwise: the under-door sprite in the
    first case, the under-roof-door sprite in the second. When the sprite for the case is absent, no
    under-door image is drawn, again without fallback. The under-door image has the depth bias given
    in S-0102. Where it and the body or bib have exactly the same depth, the body or bib pixel stays
    visible: the under-door image loses every depth tie with them, because it is drawn after them
    and S-0102 has the rule that an object pixel does not replace a stored depth equal to its own.
    A protected building's bib, under-door, replacement and door images take the protection
    appearance of S-0513 claim 29b.
    - Backing: `measurable` (B10 for sprite selection and under-door damage frames,
      S-0513 claim 29b and its SW10 part i for protection appearance).
      Exact depth ties and adjustments are `unmeasured` here: B10 supplies no equal-depth probe.
      Depth compositing itself belongs to S-0102.
    - Confidence: medium

23. **Door image.** While a building is unloading a unit (claim 22) and its type has a door sprite,
    the door image is drawn in addition to the body, the bib and the under-door image, whatever the
    unloaded unit. It is drawn in every door state, *closed* included, so it shows for the whole
    unload. Its door frame, an image-frame index, is an integer computed from the door state, the door progress *q* and the
    door-stage count *D* (Formulas F8):

    | Door state | Door frame before the damage term |
    |---|---|
    | *closed* | 0 |
    | *opening* | trunc(*q* × *D*) |
    | *open* | *D* − 1 |
    | *closing* | *D* − trunc(*q* × *D*), so the door frames run backwards |

    The value is then clamped to 0 … *D* − 1, and it is 0 when *D* ≤ 0. When the building is damaged
    and its type is a damaged-door type, *D* is added. The clamp of claim 3 does not apply, and a
    door frame below 0 or at or past the sprite's image-frame count draws nothing. In particular,
    with *D* < 0 a damaged damaged-door type requests frame *D* and draws no door image; an
    undamaged building requests frame 0. With *D* = 0 both request frame 0. The door image is drawn at the image
    anchor of claim 8 with the depth bias of S-0102. Outside unloading, and for a building without
    a door sprite, no door image is drawn.
    A protected building's bib, under-door, replacement and door images take the protection
    appearance of S-0513 claim 29b.
    - Backing: `measurable` (B10 for presence throughout unloading and absence outside it,
      S-0513 claim 29b and its SW10 part i for protection appearance, including its drawing
      correction tested against the specified result rather than the original).
      The positive-stage frame rule and damaged-door variant are `measurable` (B10).
      Non-positive stage counts and negative or high out-of-range door frames are `measurable`
      (B20). The door anchor is `unmeasured` here: neither scenario isolates its canvas centre
      against the foundation corner; its depth bias belongs to S-0102.
    - Confidence: high (drawn only while unloading, in every door state), medium (the door-frame rule,
      the anchor)

24. **Door-art inputs.** The door-stage count and damaged-door flag are independent settings
    in the building's art section named by its image name, not its rules section or, when the
    names differ, the art section named by its type name. The count is a signed 32-bit integer
    read with S-0003 claims 3 to 5; the flag is a boolean read with S-0003 claim 2. Their initial
    defaults are 0 image frames and false. A missing or empty key keeps the value already in force.
    Neither setting is inferred from the door sprite's image-frame count or its existence.
    Changing the count changes claim 23's requested frames, not the door movement duration of
    S-0417 claim 2 and Formulas F2. S-0102 claim 13 owns the door image's depth bias, not these
    inputs. The exact key names are in Import mapping.
    - Backing: `measurable` (B10, B20 and B21)
    - Confidence: high

25. **Replacing a running slot animation.** When a slot is started while it already holds an
    animation (any start of this spec, including claim 12's condition swaps and claim 20's repair
    cycle), the old animation is removed and the new one is created as any new animation is
    (S-0109 claims 5 to 7), except for its stage. The symbols *a*, *k*, *E*, λ<sub>e</sub>, *N*
    and the step σ are those of S-0109's Terms; *e* is defined below:
    - **The stage is kept.** The new animation's stage *k* starts at the replaced one's stage, not
      at the stage a new animation starts at. It shows image frame *a* + *k*, where *a* is its own
      type's start field (S-0109 claim 1). It therefore shows the replaced one's image frame only
      when the two types have the same start field. That the stage, not the image frame, is what
      carries over when the two types' start fields differ is `unmeasured` and rated medium
      (backing below); B2 cannot tell the two apart.
    - **The passes are not kept.** Its remaining passes take their creation value, so the
      replaced one's completed passes do not carry over.
    - **The partial pass is a whole pass.** From the kept stage it plays by S-0109 claims 6 to 10,
      and the pass it starts in uses up one of its remaining passes like any other pass, although
      it is partial.
    - **A kept stage beyond the new type's range.** The kept stage is not limited to the new
      type's range. The new animation plays on from it by S-0109 claims 1 and 6 to 10, with its own
      step direction σ: −1 when the new type is reversed (S-0109 claim 8), +1 otherwise. An image
      frame *a* + *k* of *N* or more draws nothing (S-0109 claim 1). So, with *e* the end limit of
      the pass it starts in (S-0109 claim 6; for a shadow type that is not reversed, the last
      pass's end limit is the smaller of *E* and λ<sub>e</sub> − *a*, since its last pass also ends
      at λ<sub>e</sub> − *a*, S-0109 claim 10):
      - a forward type that is not ping-pong shows image frame *a* + *k* until its first advance
        when *k* ≥ *e*, and that advance ends the pass, using up one remaining pass,
      - a reversed type that is not ping-pong counts down from the kept stage (a reversed
        ping-pong type follows S-0109 claim 9); when *k* ≥ *e* + 1 its first advance still
        leaves the stage at or above *e*, so that advance ends the pass; when *k* = *e* it plays down
        through the range as a reversed pass does,
      - a forward ping-pong type that is not a shadow type (a shadow type follows S-0109 claims 9
        and 10) turns on its first advance (stage *k* + 1, at or above *e*) and
        turns again on the next (stage *k*, still at or above *e*), so it alternates between stages
        *k* and *k* + 1 and never comes back into its range, never ends a pass, and stays until the
        slot is started again or the building's animations are removed.

      None of this is special to replacement: it is S-0109's ordinary play from a stage that the
      replacement put beyond the range.

    A start whose chosen name is empty or names no animation type does nothing (Terms), so it
    neither removes nor replaces the running animation.
    - Backing: `measurable`: B2 for the kept stage across a damage swap whose two types have the
      same start field; B18 part h run 2 for the passes taking their creation value and the partial
      pass counting as whole, judged on a *special-3* that replaces a running one of the same type.
      The kept stage when the two types' start fields differ is `unmeasured`: no scenario here
      gives the two types different start fields. A kept stage beyond the new type's range is
      `unmeasured` too: no scenario here replaces a running animation by a type with a shorter
      range, and the consequences follow from S-0109's approved play rules, not from a run.
    - Confidence: high for the kept stage across a damage swap between two types with the same
      start field (B2); medium for the kept stage in the other starts of this spec, such as claim
      20's repair cycle (`unmeasured`: B18 part h run 2 predicts from the restart frame, so it
      does not test the kept stage),
      for the passes taking their creation value and the partial pass counting as whole, until
      B18 part h runs (as claim 20 rates them), for the kept stage across types with different
      start fields (`unmeasured`), and for a kept stage beyond the new type's range (`unmeasured`)

26. **Buildings that need an engineer (Q-1772).** This claim covers a building whose type has the
    needs-engineer flag. *Manned* is S-0515 claim 2's state: a building is manned from its first
    ownership change, so by S-0405 claim 8 such a building is not in operation before that change.
    Not being manned does not itself hide its slots (the hiding rules of claim 10 and of other
    specs still apply); what changes is whether they advance. Two further rules that Q-1772 asks
    about apply to every building and are claims 27 and 28.
    - **Placed by the map's building list while not manned.** This holds whatever the owner: the
      neutral or special house, or a computer player's house that the map gives the building from
      the start. (In skirmish the map's building list places nothing for a human player's house,
      S-0009 claim 27.) Observable: from its placement on, every slot started at its set-up
      (claim 11) whose counting flag is *freeze-unpowered* shows the image frame it was created
      with (S-0109 claims 1, 8 and 12) and does not advance, whether or not the type needs power
      (confidence medium until B22 part (a) runs). This is because such a building counts as
      disabled in claim 13's sense from its placement on, a further cause of disabling beside the
      switch-off and the pulse of S-0405 claim 9, which this claim adds. A slot whose art turns
      *freeze-unpowered* off keeps playing. The *idle* slot and the *active-1* to *active-4* slots
      follow the same rule; only each slot's counting flag decides. The *guard* order that becomes
      current at the first per-tick start (claim 11) starts no slot, so it unfreezes nothing.
      Two slots whose image frame another rule sets on every tick are not held at a frame: the
      *turret* slot shows the frame claim 19 gives for the turret facing, and a running
      *special-1* silo-fill slot shows the fill level of claim 16. That per-tick setting wins over
      the hold, here and wherever else this claim or claim 13 holds a slot's frame.
    - **Placed any other way.** A building of such a type that trigger action code 125 creates, or
      that is produced, is not disabled at placement. Its slots play from its grand opening (claim
      11): at the end of its build-up when produced, and in the tick the action runs when code 125
      creates it. Unlike the map-placed case, this building not being disabled at placement is
      `unmeasured` and rated medium: a disabling before its slots start would show the same frames,
      and no input is known that tells the two apart. Whether a replacement that code 130 restores
      is disabled at its placement is open (Open questions).
      For a type that needs power, its *freeze-unpowered* slots start frozen: a building that is
      not manned is not in operation (S-0405 claim 8's manned row), and claim 13's start rule
      reads the in-operation result, whatever its house's power.
    - **Slot starts while not manned.** A slot started while the building is not manned follows
      claim 13's rule for a start: it starts frozen only when the type needs power. So, for a type
      that does not need power, a damage swap (claim 12) or any other start restarts the slots it
      touches running and unfrozen, and they play from then on, unless an independent hold such as
      an erasure holds them (S-0109 claims 21 and 23), although the building is still not manned
      and still disabled. For a type that needs power the restarted *freeze-unpowered* slots start
      frozen. This is the original's behaviour, kept as our rule with no quirk switch, as
      decided at the delegated sign-off, 2026-10-25.
    - **A pulse on a building that is not manned.** When the pulse ends, its re-enabling unfreezes
      the *freeze-unpowered* slots as claim 13 states for any re-enabling, so they play from then on,
      unless an independent hold such as an erasure holds them (S-0109 claim 23), although the
      building is still not manned (`quirks.enable_unfreezes_unpowered`).
    - **At every ownership change.** As part of every ownership change of such a building, on the
      tick of the change, the building is re-enabled and every running *freeze-unpowered* slot that
      is frozen, and that no independent hold still holds (an erasure, S-0109 claim 23), resumes
      from the image frame it held. It does not start again from the frame it was
      created with. In B22 the held frame is the frame the slot was created with, so the two
      readings show the same frame there and differ in timing: a resumed slot advances at its
      first eligible update, as below, while a slot started again would begin its first stage
      afresh at the change. B22 part (b) records that timing, and part (k), which changes the
      owner of a building whose slots a pulse froze after they had advanced, tells a resume from
      a start at the slot's first frame by frame. Part (k) does not tell a resume from a replacing
      start that keeps the replaced animation's stage (claim 25), which shows the same frame; that
      no such start comes is `unmeasured` (no input is known that tells the two apart). No slot is started, stopped or restarted, so each slot keeps the variant it was
      started with (normal or damaged, claim 12). This happens whether or not the building is in
      operation after the change, for example under a pulse or when it is not powered under
      S-0405 claim 8's power row (`quirks.enable_unfreezes_unpowered`). It also happens at every later ownership change of a
      building that is already manned, where it changes nothing unless a slot was frozen at the
      time, for example by a power loss or by disabling. A resumed slot advances at its first
      eligible update (S-0109 claim 23): its clock ran while it was held, so this is normally its
      first update after the change, on the tick of the change or the next. A slot that was already
      playing before the change is not affected by it.
    - **Afterwards.** The change of owner can change the building's in-operation result, its
      powered state in claim 13's sense. Claims 13 and 14 then apply to that change in the
      building's next update, as for any other change of powered state. A building that was not
      manned has never been in operation, so before its first ownership change its powered state
      is "not in operation". For a type that needs and draws power, a capture into a house with
      enough power (every other row of S-0405 claim 8 holding) brings it into operation, and claim
      13's power-return column applies in its next update; its resumed slots go on advancing. A
      capture that leaves it not powered under S-0405 claim 8's power row (its new house has low
      power and the building has fewer than two chargers), or under a pulse, leaves it not in
      operation: its
      powered state does not change, claim 13's power-loss column does not apply (claim 27), and
      the resumed slots go on advancing, though it is not in operation. They freeze again only at
      a later change from in operation to not in operation, or at a disabling. A
      *freeze-unpowered* slot started while it is not in operation still starts frozen (claim
      13). A type that neither needs and draws power nor is marked for special power behaviour is
      not affected further.
    - Backing: `measurable`: B22 part (a) for the freeze at map placement, the frame the slot was
      created with held, the slots that keep playing, the *guard* order unfreezing nothing, and the
      buildings created by code 125, including their slots starting in the tick of the trigger
      action, but not their not being disabled at placement (below); part (i) for the *turret* slot's per-tick frame winning over the hold; parts (b) and
      (c) for the resume at the first and at a later ownership change, and part (k) for the resume
      from a held frame other than the first; part (d) for the damage swap; part (e) for the
      pulse; part (f) for a type that needs power, captured into houses with ample and with short power,
      including the short-power capture being no change of powered state. `unmeasured`: a produced building of such a type
      (no scenario here builds one); the code-125 building's not being disabled at placement (a
      disabling before its slots start shows the same frames in B22 part (a), and no input is
      known that tells the two apart); claim 14's branch after the change (B22 has no type with
      special power behaviour); a later ownership change resuming slots frozen by a power loss (B22 has no
      needs-power type that loses power after it is manned); the silo-fill *special-1* slot's per-tick frame winning over
      the hold (it runs only with storage above 0, and no skirmish path fills a building's own
      storage, claim 16).
    - Confidence: medium for the absence of a stop or a variant change at an ownership change,
      until B22 parts (b) to (d) run, since it is the same rule as the resume (no slot is
      started, stopped or restarted), which rests on claim 13's disable and enable cases. Medium
      for the absence of a restart, which parts (b) and (c) tell from a resume only by timing,
      until part (k) runs, and medium for the absence of a replacing start that keeps the stage
      (`unmeasured`, as the backing says); medium for the *guard* order unfreezing nothing, which
      rests on the freeze at map placement (above). Medium for the code-125 building's slots playing, at the level of claim
      11, until B22 parts (a) and (j) run. Medium for the code-125 building's slots
      starting in the tick of the action (claim 11), until B22 part (a) runs; medium for its not
      being disabled at placement (`unmeasured`, as the backing says). Medium for the
      per-tick frames of the *turret* and *special-1* slots winning over the hold, until B22
      part (i) runs (*special-1* `unmeasured`), and medium for the *turret* frame's being set on
      every tick rather than once at placement and then held, even after part (i) runs: part (i)
      tells the setting from a hold at the frame the slot was created with, but the turret facing
      of a building that is not manned does not change during the hold, and no input is known that
      turns it. Medium for the parts that rest on claim 13's
      disable and enable cases: the freeze at map placement, the frame the slot was created with held,
      the slots that keep playing there, and the resume from the held frame at every ownership change,
      until B22 parts (a), (b) and (k) run. Medium for the short-power capture being no change of
      powered state, so that the resumed slots go on advancing, until B22 part (f) runs.
      Medium also for the produced building (`unmeasured`), the restart running after a
      damage swap, the unfreeze at the end of a pulse, the resume timing, the next-update effects
      after the change (the claim 14 branch `unmeasured`), and a later change resuming slots
      frozen by a power loss (`unmeasured`).

27. **A building never powered (Q-1772).** Claim 13's power-loss column applies only at a change
    from powered to unpowered. A building that has not been powered since it was placed or completed
    has had no such change. Its *light-unpowered* and *effect-unpowered* slots started at its
    grand opening therefore keep playing, and *super-low-power* has not started. This holds for a
    building of a needs-engineer type that needs power while it is not manned, and after its
    capture into a house short of power (claim 26). It holds as well for any building that needs and draws power and completes while its house is short of power,
    and for one that the map places switched off (claim 28), before and after it is switched on
    in a house short of power, until it first comes into operation.
    Claim 13's rule for a start still decides whether its *freeze-unpowered* slots start frozen.
    - Backing: `measurable` (B22 part (g) for its *light-unpowered* and *effect-unpowered*
      slots, and part (m) for a building the map places switched off). `unmeasured` for *super-low-power* not having started: it starts only from a running
      *super-3* at a power loss, and B22 has no superweapon type.
    - Confidence: medium until B22 parts (g) and (m) run, as claim 13 rates the rule; medium for
      *super-low-power* (`unmeasured`)

28. **The working animations of S-0405 claim 9 (Q-1772).** They are the slot rules of claims 13
    and 14, applied at any change of the building's in-operation result whichever row of S-0405 claim 8
    causes it, together with the rules below for particular rows. Per row of claim 8:
    - *switched on*: switching off disables the building in claim 13's sense. Switching on
      re-enables it, so its *freeze-unpowered* slots resume even under low power (claim 13). The
      end of a pulse re-enables it too while it is still switched off (claim 13).
      - **Placed switched off by the map (Q-1890).** The map's request to start a building
        switched off takes effect only for a type with a base drain above 0 or the needs-power
        flag (S-0405 claim 18). Such a building is switched off, and so disabled, at the end of
        its placement, after its set-up and grand-opening slot starts (claim 11). From its
        placement on, each of those slots whose counting flag is *freeze-unpowered* shows the
        image frame it was created with (S-0109 claims 1, 8 and 12) and does not advance, whether
        or not the type needs power, as for the map-placed building of claim 26. The *idle* and
        *active-1* to *active-4* slots follow the same rule; only each slot's counting flag
        decides. A slot whose art turns *freeze-unpowered* off keeps playing, and for a type that
        needs and draws power claim 27 applies to it. The per-tick frames of claim 26 (*turret*,
        silo fill) win over the hold here too. A type with neither condition is not switched off
        by the map; unless claim 26 disables it, its slots play from its placement. A slot started while the
        building is switched off follows claim 13's rule for a start: it starts frozen only when
        the type needs power.
      - **An ownership change while switched off.** It leaves the building switched off (S-0405
        claim 10). For a type that does not need an engineer it is no re-enabling, so the held
        slots stay held. For a type that needs an engineer it is one (claim 26), and they resume.
      - **Switching on such a building.** The switch-on re-enables it at once, whether or not it is
        then in operation, so its held slots resume from the image frame they held; they do not
        start again. The switch-on's re-enabling starts, stops and restarts no slot, so each keeps
        its variant (claim 12). A resumed slot advances at its first eligible update (S-0109 claim
        23), which for a power order is in the next tick (S-0203 claim 7) when the slot gets its
        turn there; a counting removal earlier in that tick's turn pass can cost it that turn
        (S-0109 claim 5, S-0203 claim 10), and it then advances at its next turn. In a house short
        of power a building of a type that needs and draws power, with fewer than two chargers,
        stays not in operation, so no change of its powered state follows: its resumed slots go on
        advancing and do not freeze again in its next update or later, until a change from in
        operation to not in operation or a disabling
        (`quirks.enable_unfreezes_unpowered`). When the switch-on brings a type that needs and
        draws power into operation, claim 13's power-return column applies in its next update, and
        claim 14's for a type marked for special power behaviour. A type with the needs-power flag
        and no base drain never stops for low power (S-0405 claim 8), so the house's power does not
        keep it out of operation at its switch-on. Neither of claim 13's power columns applies to
        it, since claim 13 limits both to a type that needs and draws power: the switch-on's
        re-enabling alone resumes its held slots, and they go on advancing. For such a type that is
        not marked for special power behaviour, no slot starts or stops at a change of its
        in-operation result at the switch-on, if any. For one that is so marked, claim 14 applies
        to a change into operation at the switch-on in its next update: its *low-power* slot stops
        and every slot with *special-outage* is started (normal or damaged variant), running or
        not. While a pulse holds the building, a switch-on is refused (S-0405 claim 10)
        and changes no slot.
    - *not under a pulse*: the pulse disables the building and its end re-enables it (claim 13).
    - *powered*: claim 13.
    - *no blackout*: claim 14.
    - *manned when required*: claim 26.
    - *not building or selling*: the sale's own processings start, stop and freeze none of the
      sold building's slots. The building stops being in operation at the sale's first
      processing, and claims 13 and 14 apply to that change like any other, in the building's
      next update. So only a type that needs and draws power, or one marked for special power
      behaviour, changes slots then. For any other type, such as a refinery without the
      needs-power flag, each running slot that no rule independent of the sale holds (claim 13's
      disabling, an erasure, S-0109 claim 23) is neither paused nor hidden by the sale from its first
      processing, through every processing at which a docked unit holds the sale (S-0408 claim
      6), up to the start of the build-down. From the tick the build-down starts, claim 10 hides
      every slot for the rest of the sale. The slots
      that a docked unit's unloading starts and stops follow the docking roles under their own
      specs (claim 20), including when the sale tells that unit to leave. Claim 18's charge
      check, which can start superweapon slots, does not run while the building is built or
      sold. The build-up and build-down sprites themselves are claims 1 to 5.
    - *alive*: this spec adds no slot rule.
    An erased building's slots are held as S-0109 claim 23 states. Apart from these, no row of
    S-0405 claim 8 stops, freezes or starts a slot animation. In particular, apart from the
    docking-role slots (claim 20), a sale stops, freezes or starts no slot of a type that
    neither needs and draws power nor is marked for special power behaviour before its
    build-down starts.
    - Backing: `measurable`: B22 part (h) for the sale of a building with no needs-power flag, its
      slots drawn from its first processing to the build-down's start; the slots advancing through
      a held sale are `unmeasured`, since no input is known that ensures a hold (S-0402 claim 11),
      and part (h) reports that only when a hold arises; B14 covers the hiding during the
      build-down. B11 covers switching off and on. B22 part (l) covers a building the map places
      switched off: the hold from its placement, the slots that keep playing, a type with neither
      condition playing, the hold through an ownership change of a type that does not need an
      engineer, and the resume at its switch-on without a restart; part (m) covers its switch-on
      in a house short of power, after which the slots keep playing. `unmeasured`: the switch-on of
      a type with the needs-power flag and no base drain in a house short of power (it rests on
      S-0405 claim 8's power row and claim 13's scope, which leaves out both power columns for
      it, and no part places such a type switched off); the switch-on
      refused under a pulse for such a building (it is S-0405 claim 10's rule, which S-0405
      scenario W7 measures for a building switched off in play, and no scenario here adds a
      pulse to part (l)); the ownership change of a needs-engineer type placed switched off
      (claim 26's resume, which B22 parts (b) and (k) measure for a building the map placed
      switched on); the per-tick frames winning over this hold (claim 26 rates them); claim 14's
      reaction at the switch-on of a type marked for special power behaviour (B22 has no such
      type, and claim 14 rates the rule). The other rows rest on the claims they name and their
      backings.
    - Confidence: medium for a sale changing no slot before its build-down, until B22 part (h)
      runs with a hold of at least 6 ticks: S-0402 claim 11 has the sale break off the unloading
      harvester's contact, and no input is known that ensures a hold. Medium for the reading of
      S-0405 claim 9 as a whole. For a building the map places switched off: medium for the hold
      from its placement and for the resume at its switch-on, which rest on claim 13's disable
      and enable rules, until B22 part (l) runs; medium for its slots going on advancing after a
      switch-on in a house short of power, until part (m) runs; medium for the `unmeasured`
      points above, which rest on S-0405 claim 10 and claims 14 and 26.

## Formulas

Where a formula below says that a floating-point result is rounded toward zero, that is the
original's round-toward-zero mode (glossary). Its scope and its switch belong to S-0205 (number
domains). This spec defines no switch of its own for it.

**F1 Build-up delay** (ticks, integer):

> n = ⌊N_b / 2⌋ (integer, the image-frame count halved and truncated), or n = G + 1 for gates
>
> d_b = trunc(B × 900 / n) for n > 0, else d_b = 1
>
> theater-flag building after the theater refresh (claim 5): c = N_b, d_b = 75 div N_b (integer
> quotient truncated toward zero) for N_b > 0, else d_b = 1

*B* is held as binary64 game minutes. When data supplies it, S-0003 claims 6 and 7 give its
single-first decimal conversion and any percentage multiplication. Without data, the binary64
absent-key default stays unchanged. The constant 900 has units of ticks per game minute.
The multiplication and the division are each done in binary64 (53-bit significand) and rounded
toward zero. The quotient is then truncated toward zero to an integer. When *B* × 900 is meant to
be a whole number but *B* is not exactly representable, the product can fall just below it, and
*d_b* comes out one tick lower than with round-to-nearest arithmetic. This is the round-toward-zero
mode of S-0205 (number domains). The delay is computed each time the build-up sprite is loaded, with *B* as it
is held then: at each type read that loads it, and at the theater refresh for a building with the
new-theater flag (claim 5).

**F2 Body frame** (integer): see claim 2. With at least one body frame, the body frame drawn is
min(f, ⌊N/2⌋ − 1), and the shadow frame is that body frame + ⌊N/2⌋. With no body frames,
no body or shadow is drawn. Any image-frame index below 0 or ≥ N draws nothing. The special gate image is exempt from the body clamp (claim 4).
The bib and door image are also unclamped (claims 7 and 23).

**F3 Damaged tests.** h = health / strength, where health and strength are integers and the
division is done in binary64, rounded toward zero. The yellow and red ratios are binary64 values.
Damaged ⇔ h ≤ yellow, badly damaged ⇔ h ≤ red, compared exactly in binary64.

**F4 Refinery level** (integers): m = trunc(stored amount), S = storage capacity (integer field).
L = 0 when m = 0, else L = (4m) div S, the integer quotient truncated toward zero. With m ≥ 0 and
S > 0 this equals ⌊4m / S⌋. The original divides by zero when S = 0 and m > 0 and fails. Our engine
uses L = 0 in that case, and the importer warns (claim 15). With S < 0, L = trunc(4m / S), truncated
toward zero, which is 0 or negative; a negative L selects no slot.

**F5 Silo level:** L = 0 when S ≤ 0. Otherwise L = clamp(trunc(4m / S + 0.5), 0, 3), where
m = trunc(stored amount) and 4m are integers, the division and the addition are done in binary64
rounded toward zero, and trunc truncates toward zero.

**F6 Remaining charge in game minutes:** t_r = r × κ, with r the remaining ticks (integer) and
κ = 2 386 093 / 2³¹ ≈ 0.00111111114, the single-precision value nearest 0.0011111111, which
approximates 1/900. The product is formed in binary64 (53-bit significand), rounded toward zero,
and compared with the single-precision field t_c widened exactly to binary64. The "charged" branch
is taken when t_c ≥ t_r.

**F7 Turret frame** (integer): (28 − j) mod 32, where mod gives the non-negative remainder, so the
result is 0 … 31 for j in 0 … 31.

**F8 Door frame** (claim 23). q = (M − r) / M, where M is the length in ticks of the door's current
movement and r the ticks it still has to run, both integers, divided in binary64 and rounded toward
zero. q = 1 when the door is not moving or M = 0. The product q × D is formed in binary64, rounded toward
zero, and truncated toward zero to an integer. With p₀ the value from the table of claim 23:

> p = max(0, min(p₀, D − 1)), door frame = p + D × [damaged ∧ damaged-door type]

**F9 Held infantry size** (claim 17): T₀ = 0 and Tᵢ = trunc(Tᵢ₋₁ + sᵢ) for the held infantry in turn,
where sᵢ is the infantry type's size, a binary64 number, the addition is done in binary64 rounded
toward zero and trunc truncates toward zero to an integer. T is the last Tᵢ, or 0 with no held
infantry. For sizes of 0 or more this equals the sum of the sizes each truncated toward zero.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ticks per game minute (build-up) | 900 | ticks per game minute | Engine constant in F1 |
| Slots | 21 | count | |
| Body animation states | 6 | count | *full* has no import key; its deterministic range is the departure of claim 6 |
| Default state range | a = 0, c = 1, d = 0 | image frames, image frames, ticks | *idle*, *active*, *aux-1*, *aux-2*; also *construction* for a type without a build-up sprite (claim 5) |
| Anchor displacement | 128, 128 | leptons | From the top-left cell centre toward the foundation's top corner |
| Grand-opening "charged" test | 15 | ticks | A granted record below it starts *super-3* |
| Charged-animation time limit | 990 | game minutes | Above it, claim 18 is disabled |
| Charge time scale *κ* | 2 386 093 / 2³¹ | game minutes per tick | Single-precision constant, F6 |
| Unloading depth adjustment | −20, −40 | depth units | Deploying sprite, roof-deploying sprite (claim 22) |
| Refinery levels | 0 … 3 | level | 3 covers every quotient of 3 or more |
| Silo levels | 0 … 3 | level | Level 0 hides the slot animation |
| Turret directions | 32 | image frames | Turret frame = (28 − j) mod 32 |
| Theater-refresh build-up total | 75 | ticks | Divided by the image-frame count for a theater-flag building (claim 5, F1) |

## State and ordering

**Body frame by situation** (claim 2). The occupiable, gate and other rows below apply only
outside construction; gates using the special gate image use claim 4, not this table:

| Situation | Body frame *f* |
|---|---|
| construction state, building (gate: claim 4) | *k* (build-up sprite) |
| construction state, selling (gate: claim 4) | *a* + *c* − *k* − 1 (build-up sprite) |
| occupiable, not hurt | 0 empty, 2 occupied |
| occupiable, hurt | 1 empty, 3 occupied |
| gate, any state but *construction*, not using the special gate image | 0, or *G* + 1 damaged |
| other, not damaged | *k* |
| other, damaged, *idle* state | *k* + 1 |
| other, damaged, other states | *k* + max(*a* + *c*) over *idle*, *active*, *aux-1*, *aux-2* |

**Superweapon slots** (claims 13 and 18; with `quirks.ungranted_super_reads_clock` off, the two
charge rows need a granted record):

| Running | Event | Result |
|---|---|---|
| *super-1* | remaining charge ≤ charged-animation time | *super-2* starts, *super-1* stops |
| *super-2* | animation finishes on its own | *super-3* starts |
| *super-3* | remaining charge > charged-animation time | *super-4* starts, *super-3* stops |
| *super-4* | animation finishes on its own | *super-1* starts |
| *super-3* (with *effect-unpowered*) | power lost | *super-3* stops, *super-low-power* starts |
| *super-low-power* | power comes back | *super-low-power* stops, *super-3* starts again if it was marked |

**Ordering consequences:**
- The body-stage advance and periodic slot checks share the building's update. A fill-level change
  and its slot swap therefore have no extra tick of delay within that update. This consequence
  inherits the `unmeasured` backing of claims 15 and 16 for nonempty storage; no skirmish path
  establishes such a change.
- The damaged-variant swap is visible on the damage or repair tick; the body frame changes on
  that tick too (claim 12, B2). Body-stage entry timing is claim 6's, not a guarantee that every
  external event is processed before the building's turn.

## Edge cases and quirks

- **Occupied civilian damaged frame** (departure, claim 2). The original shows the empty
  damaged frame 1 rather than occupied damaged frame 3 for a hurt occupied building with tech
  level −1, a wrong-frame glitch whose correction without a switch is confirmed by the project lead.
- **`quirks.occupiable_hurt_rule`** (default: on). For occupiable types, "hurt" means badly damaged
  when the tech level is 0 or below (unbuildable civilian buildings), and merely damaged when it is
  above 0. Off: always use the damaged threshold.
- **Body-frame clamp** (departure, claims 3 and 9). The original clamps to ⌊N/2⌋ and can
  display a shadow silhouette as the body with a missing shadow for even *N* or a mismatched shadow
  for odd *N*, a wrong-frame glitch whose correction without a switch is confirmed by the project lead.
- **Gate selling direction** (departure, claim 4). The original plays a sold gate's build-up
  forwards, a drawing-only reversal glitch whose correction without a switch is confirmed by
  the project lead.
- **`quirks.building_wrap_normalisation`** (default: on). Speed normalisation of a body range
  follows two different rules. When a state is entered immediately, it needs the "normalised" mark and
  excludes construction. When a state is entered deferred, and on every wrap, it ignores the mark
  and covers construction and idle only (claim 6). Off: use the immediate-entry rule everywhere.
- **`quirks.turret_building_no_body_wrap`** (default: on). A building with a turret wraps its body
  range only while it is being built or sold. Otherwise a range with a non-zero delay runs past its
  end and then shows the clamped body frame of claim 3. Off: wrap as other buildings do.
- **`quirks.turret_start_clears_damaged`** (default: on). Starting the *turret* slot at grand
  opening always counts as "not damaged". A building that completes while damaged therefore shows
  normal art on all its running slot animations until its next damage or repair event, or the next
  slot start in the damaged condition (claims 11 and 12). Off: start the *turret* slot in the building's actual condition.
- **`quirks.enable_unfreezes_unpowered`** (default: on). Re-enabling a disabled building unfreezes
  its *freeze-unpowered* slot animations even when it has no power (claim 13). This includes a
  switch-on in a house short of power, also of a building the map placed switched off (claim 28),
  the end
  of a pulse on a building that is switched off or that needs an engineer and is not manned, and every ownership change of
  such a building, whether or not it is then in operation (claim 26), also while it is switched off. Off: a re-enabling or an
  ownership change unfreezes them only when the building is in operation right after it. Otherwise
  they stay frozen until power comes back, or until a later re-enabling or ownership change after
  which the building is in operation.
- **`quirks.damage_swap_drops_occupied`** (default: on). Crossing the damaged threshold restarts the
  running slot animations as normal or damaged, never as occupied. An occupied building that is hurt
  and then repaired shows normal slot animations, not occupied ones, until something restarts them.
  Off: choose the occupied variant when occupied.
- **`quirks.roof_unload_no_fallback`** (default: on). Unloading a jumpjet or balloon-hover unit from
  a building with no roof-deploying sprite keeps the body sprite, and with no under-roof-door sprite
  draws no under-door image, even when the ordinary deploying and under-door sprites exist
  (claim 22). Off: fall back to the ordinary sprites.
- **`quirks.speed_normalised_delays`** (default: on, shared with S-0107; glossary claim G3). The
  body-stage delays that claim 6 gives as N(*d*, *s*) depend on the game speed: the delay after an
  immediate change for art marked normalised, and the *construction* and *idle* delays after a
  deferred change or a wrap. Off: each of them is the raw *d*, so those body stages take the same
  number of ticks at every game speed. `quirks.building_wrap_normalisation` has no effect then.
  The build-up delay of claim 5 is raw in both settings; only a build-up stage that wraps is
  affected.
- **Round-toward-zero mode** (no switch here). F1, F3, F5, F6, F8 and F9 round every intermediate
  floating-point result toward zero in binary64. The single switch for that mode belongs to
  S-0205 (number domains).
- **`quirks.full_state_unmaxed`** (default: on). The *full* range is left out of the damaged-frame
  maximum of claim 2. Off: include it.
- **`quirks.theater_refresh_buildup_range`** (default: on). At the theater refresh, a building with
  the theater flag gets the construction range *c* = *N_b*, *d_b* = ⌊75 / *N_b*⌋, ignoring the
  build-up time, the shadow half of the sprite and the gate rule (claim 5). Off: keep the range of
  F1 computed from the reloaded sprite.
- **Unspecified *full* range** (no switch). The original specifies neither an initial range nor
  an import key for this state. Our engine gives it the fixed range of claim 6, displaying stage
  0 without advancing. This is a safe deterministic departure, not an original engine constant.
  No data key selects this state, so there is no importer warning for a nonexistent key.
- **Freeze flag on by default** (no switch: it is a default that data can change). Every slot
  starts with *freeze-unpowered* on, and that flag counts first, so art that sets only
  *light-unpowered* or *effect-unpowered* on a slot gets a frozen slot, not a stopped one, unless
  it also turns *freeze-unpowered* off (claim 13).
- **Silo-fill type without a *special-1* animation** (no switch). When the level is above 0 and the
  *special-1* name is empty or names no animation, the original fails on the first such tick, when
  claim 16 would force the image frame of a *special-1* slot animation that is not there. This is
  invalid data, not play behaviour. Our engine skips forcing the image frame (a departure), and the
  importer warns about a silo-fill type whose *special-1* name is
  empty or names no animation type.
- **`quirks.ungranted_super_reads_clock`** (default: on; claims 11 and 18). The original reads
  a superweapon record's clock whether or not the record is granted. A record never granted
  and never reset reads 0, and a reset or lost record reads its clock as the reset or the loss
  left it (claim 11). So a superweapon
  building that opens with a record that was never granted and never reset, as at most set-up placements or from
  code 125 (claim 11), shows the charged *super-3* although nothing has charged, and claim 18's
  check runs on that clock. With a lost record it reads the clock as the loss left it. Off: a record
  that is not granted counts as
  not charged, so the grand opening starts *super-1* and claim 18's check waits for the grant.
  DECIDED (autonomy, 2026-10-28): this is a switch, not a correction without one, because the
  slot starts it changes take their creation draws (claim 12) and so can change the scenario
  random stream, and an animation that ends can cost another object its turn (S-0203 claim 10).
- **Refinery with zero storage capacity** (no switch). When a refinery type's storage capacity is 0
  and ore is stored in it, the original divides by zero when it computes the fill level, and the
  program fails. This is invalid data, not play behaviour. Our engine uses level 0 (a departure,
  claim 15, F4),
  and the importer warns about a refinery type whose storage capacity is 0 or less (a negative
  capacity is reproduced as claim 15 says).

## Import mapping

The keys are in the art file, in the building's art section (the section named by its image name),
unless a row says otherwise. Booleans, decimals and named values follow S-0003, and a text value is
cut to the byte limit given, not a decoded-character count (S-0003 claim 1). As S-0006 claim 3 says, a key a rules layer does not give keeps
the held value. The defaults below are the values before any data is read. Type fields start from
them in every game, because every type is created anew at each game start. The game-wide keys of
`[General]` and `[AudioVisual]` are set to their defaults only once, when the program starts, so the
default holds in the first game of a session, and in a later game an absent key keeps the value an
earlier game's rules layers left (S-0006 claim 3).

The import rules below inherit the backings of the claims they feed, where those claims' scenarios
exercise them. Their remaining parse details (partial body-range tuples, conditional reads of slot
metadata, upgrade-slot reads, and the two-section foundation priority) are `unmeasured` in this
spec: no scenario here supplies those particular import cases. Confidence: high for the stated
reads and defaults. Key names use the exact spelling and case shown (S-0001 claim 13).

**Building art keys**

| Key | Form | Absent-key default | Use here |
|---|---|---|---|
| `Foundation` | named value, text cut to 31 bytes (vocabulary below) | `1x1` | the foundation's W × H (claim 8) |
| `GateStages` | integer | 9 | *G* (claims 2, 4, 5) |
| `DoorAnim` | text, cut to 63 bytes (S-0007 claim 11) | none (no door sprite) | door sprite (claim 23) |
| `DoorStages` | integer, signed 32-bit (S-0003 claims 3 to 5); read from the image name's art section, not derived from the sprite | 0 | *D* (claims 23, 24) |
| `DamagedDoor` | boolean (S-0003 claim 2); read from the image name's art section, independent of the sprite's frame count | false | damaged-door type (claims 23, 24) |
| `SiloDamage` | boolean | false | silo fill (claim 16) |
| `IsAnimDelayedFire` | boolean | false | delayed-fire type (claims 13, 20) |
| `ExtraDamageStage` | boolean | true | no effect (claim 21) |
| `AnimIdle`, `AnimActive`, `AnimAux1`, `AnimAux2` | three integers `start,count,delay` | 0,1,0 | the *idle*, *active*, *aux-1* and *aux-2* ranges (claims 2, 6). The syntax is that of S-0003 claim 9, but unlike the generic triple there, the components that are not read keep their held values, which is defined behaviour and is reproduced. The importer warns about such a value. *full* has no key |
| `Buildup` | text, cut to 15 bytes | none (no build-up sprite). An empty value keeps the earlier name (S-0007 claim 11) | build-up sprite (claims 1, 5). At a type read the file is name ‖ .shp with letter substitution and the generic fallback (S-0007 claim 11). At the theater refresh, for a building with the theater flag only, it is name ‖ .E with neither (S-0007 claim 12) |
| `Normalized` | boolean | false | game-speed normalisation (claim 6) |
| `BibShape` | text, cut to 63 bytes | none (no bib) | bib (claim 7) |
| `DeployingAnim`, `RoofDeployingAnim`, `UnderDoorAnim`, `UnderRoofDoorAnim` | text, cut to 63 bytes | none | the unloading sprites (claim 22) |

**Foundation vocabulary.** The words, compared ignoring case, with their size W × H in cells: `1x1`
(1 × 1), `2x1` (2 × 1), `1x2` (1 × 2), `2x2` (2 × 2), `2x3` (2 × 3), `3x2` (3 × 2), `3x3`
(3 × 3), `3x5` (3 × 5), `4x2` (4 × 2), `3x3Refinery` (3 × 3; which of its cells are occupied
belongs to the placement spec), `1x3` (1 × 3), `3x1` (3 × 1), `4x3` (4 × 3), `1x4` (1 × 4), `1x5`
(1 × 5), `2x6` (2 × 6), `2x5` (2 × 5), `5x3` (5 × 3), `4x4` (4 × 4), `3x4` (3 × 4), `6x4` (6 × 4),
and the zero-size word, written as the digit zero, a lowercase x and the digit zero (0 × 0). A
word that matches none of these gives `1x1` (S-0003 claim 16). The key counts in two art sections:
the type's art section (the one named by its image name) and the one named by the type's own id,
which are the same section unless the type gives a different image name. At each type read, a value
in the id's section that gives a size other than 1 × 1 decides the foundation. Otherwise (no key
there, a word that gives `1x1`, or a word that matches none) the size from the image name's section
stands. So
the id's section can change the size but cannot set it back to `1x1`.

**Slot keys.** Each slot has a base key. The suffixes `Damaged`, `Garrisoned`, `X`, `Y`, `ZAdjust`
and `YSort` give the damaged name, occupied name, displacement (*x*, *y*, integer pixels), depth
adjustment and y-sort (integers), and `Powered`, `PoweredLight`, `PoweredEffect` and
`PoweredSpecial` give the four power flags (*freeze-unpowered*, *light-unpowered*,
*effect-unpowered*, *special-outage*, booleans). Base keys: *active-1* … *active-4* `ActiveAnim`,
`ActiveAnimTwo`, `ActiveAnimThree`, `ActiveAnimFour`, *pre-production* `PreProductionAnim`,
*production* `ProductionAnim`, *turret* `TurretAnim`, *special-1* … *special-4* `SpecialAnim`,
`SpecialAnimTwo`, `SpecialAnimThree`, `SpecialAnimFour`, *super-1* … *super-4* `SuperAnim`,
`SuperAnimTwo`, `SuperAnimThree`, `SuperAnimFour`, *idle* `IdleAnim`, *low-power* `LowPower`,
*super-low-power* `SuperLowPower`.

- **Names** are text cut to 15 bytes. An empty value keeps the earlier name. At each read of a
  slot's keys, a damaged or occupied name that is empty once that read's names are taken takes the
  normal name held at that point. The names of an animation type are looked up when a slot starts
  (Terms).
- **The other keys** of a slot (displacement, depth adjustment, y-sort and the power flags) take
  effect at a read only when at least one of the slot's three names is non-empty once that read's
  names are taken. Otherwise they keep their held values, even where the data gives them. The
  *turret* slot is the exception: its other keys always take effect.
- **Defaults:** names empty. Displacement, depth adjustment and y-sort 0. *freeze-unpowered* on,
  and *light-unpowered*, *effect-unpowered* and *special-outage* off.
- **Power flags:** the *pre-production*, *production* and *turret* slots have no power-flag keys and
  keep the defaults, so they count *freeze-unpowered* (claim 13).
- **Upgrade slots:** slot *upgrade-N* (N = 1, 2, 3) uses `PowerUpNAnim`, `PowerUpNDamagedAnim`,
  `PowerUpNLocXX`, `PowerUpNLocYY`, `PowerUpNLocZZ` and `PowerUpNYSort`, with the digit N written in
  place of the letter (for example `PowerUp2LocXX`). They give the normal name, the damaged name,
  the displacement *x* and *y*, the depth adjustment and the y-sort. Their names are cut to 15
  bytes, and an empty damaged name is *not* filled with the normal name. Their other keys are
  always read. Upgrade slots have no occupied name and no power-flag keys, so they keep the defaults
  and count *freeze-unpowered*. The upgrade index runs from 1 to the rules key `Upgrades`.
- **Turret slot:** its keys (`TurretAnim`, `TurretAnimDamaged`, `TurretAnimGarrisoned`,
  `TurretAnimX`, `TurretAnimY`, `TurretAnimZAdjust`, `TurretAnimYSort`) are rules keys, not art
  keys. All of them are read from the building's rules section except `TurretAnimGarrisoned`, which
  is read from the rules section named by the building's image name. That is the same section
  unless the type gives a different image name. All other slot keys are art keys.

**Rules keys**

| Key | Section | Form | Absent-key default | Use here |
|---|---|---|---|---|
| `Gate` | building | boolean | false | gate (claims 2, 4, 5) |
| `CanBeOccupied` | building | boolean | false | occupiable type (claim 2) |
| `TechLevel` | building | integer | 255 | tech level (claim 2). The default is above 0, so "hurt" means damaged |
| `Refinery` | building | boolean | false | refinery (claims 11, 15) |
| `Storage` | building | integer | 0 | *S* (claims 15, 16) |
| `InfantryAbsorb` | building | boolean | false | claim 17 |
| `ExtraPower` | building | integer | 0 | claim 17 (must be positive) |
| `SuperWeapon` | building | superweapon type name | none | claims 11, 18 |
| `ChargedAnimTime` | building | decimal, held in single precision, game minutes | 999 | *t_c* (claim 18). The default is above 990, so claim 18 is off unless the data sets the key |
| `UnitRepair` | building | boolean | false | repair depot: the repair-cycle rows and the other depot rows of claim 20 |
| `Naval` | building | boolean | false | a naval type is exempt from claim 20's every-tick depot row only; the repair-cycle rows still apply |
| `Powered` | building | boolean | false | needs power (claims 13, 28) |
| `Power` | building | integer | 0 | a negative value means the building draws power (claim 13); it is the base drain above 0 that lets the map start the building switched off (claim 28) |
| `PoweredSpecial` | building | boolean | false | special outage (claim 14) |
| `NoShadow` | building | boolean | false | suppresses the shadow (claim 9) |
| `InvisibleInGame` | building | boolean | false | claim 1 |
| `TurretAnimIsVoxel`, `BarrelAnimIsVoxel` | building | boolean | false | voxel turret and barrel art, which disable or hide the *turret* slot animation (claims 11, 19) |
| `Upgrades` | building | integer | 0 | number of upgrade slots read |
| `JumpJet`, `BalloonHover` | the unloaded unit's type | boolean | false | either one selects the roof variants (claim 22) |
| `Size` | infantry type | decimal | 1 | *sᵢ* (claim 17, F9) |
| `BuildupTime` | `[General]` | decimal, game minutes | 0.05 (the binary64 value nearest 0.05) | *B* (claim 5, F1) |
| `ConditionYellow` | `[AudioVisual]` | decimal or percentage ratio | 0.5 | yellow ratio (F3) |
| `ConditionRed` | `[AudioVisual]` | decimal or percentage ratio | 0.5 | red ratio (F3) |

**Map data.** The switched-on field of an entry in the map's building list is read as S-0009
claim 40 and S-0405 claim 18 give, including its default when absent (switched on). This spec
uses it only through the switch state it leaves (claim 28).

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*: an unmodified install with no loose override
files and no engine add-ons, stock data, a tiny purpose-made map, one player with ample credits and
one bystander computer player kept outside the observed area unless a scenario needs its named
objects. The observed area is the test buildings, their nearby cells, and any route or units the
scenario uses. Apply the glossary's bystander void rule. Use standard skirmish, 800 × 600 pixels,
game speed 4 unless stated, high detail, no engine add-on and no mods (the named test files of a
scenario and the purpose-made test maps excepted, and a pulse created outside the stock inputs where
a scenario says it needs one), and no unrelated trigger effects. Record per tick, from a screen
capture that keeps every drawn frame or a
read-only recorder's tick-labelled log of the game state, the body sprite and image-frame index in use, bib and shadow
visibility, and the slot animations present (slot, variant, image frame). One run per variant
is enough unless a scenario says otherwise. Use deterministic animation playback in named test art
where an exact slot timing is tested; random playback of the animation itself is outside these
body-frame and slot-selection tests. Where a scenario needs test data
(a test art section or test sprite), it says so, and the test data is removed afterwards. Named
building kinds (power plant, refinery and so on) are stock types, and no stock value is assumed:
expected numbers are computed from the type's own fields at measurement time.

- **B1 (claims 1, 5).**
  - Setup: game speeds 0 and 4. A power plant and a barracks of one side. A test building type,
    marked invisible in game through a test map override section, with at least one slot animation.
    Choose stock build-ups with at least two body frames and positive F1 delay; record their counts.
  - Procedure: place each stock building and let it complete, then sell one. Place the test type.
  - Observable: ticks from placement to the first body-sprite frame, the number of ticks each
    build-up frame is shown, the build-up frame sequence while selling, and whether the invisible
    type shows any body, bib or shadow.
  - Runs: one per building and speed.
  - Expected: with *d_b* from F1 and placement tick *T*, the last stage first appears at
    *R* = *T* + (*n* − 1) × *d_b*. Stage 0 reappears on *R* + 1, the first construction
    processing tick, and the body sprite appears at *T* + 2(*n* − 1) × *d_b* + 1 in
    uninterrupted play (S-0406 claims 15 and 17). Both speeds give the same tick counts.
    Each pre-last stage lasts *d_b* ticks in each run; the first run's last stage shows for
    one tick only. Selling runs backwards at the same delay until completion under S-0408
    claim 17, including its stage-23 exception for a packing-capable type without a guard spot.
    The invisible type shows no body, bib or shadow, but its slot animations appear.
  - Falsified if: placement shows only one build-up run, the restart or handover differs from
    those ticks, the pre-last-frame periods depend on game speed or differ from F1, selling
    plays forwards or continues beyond its specified completion, or the invisible type shows
    any body, bib or shadow or hides its named slot. Record placement, the first report and
    the restart separately. The test does not by itself discriminate arithmetic rounding unless
    the imported *B* and sprite count give different integer delays under the two modes.
- **B2 (claims 2, 12, 25).**
  - Setup: one building with slot animations that have damaged names, one occupiable building with
    occupied-variant slot animations, a weapon of known damage, and repair available.
  - Procedure: damage the first building in steps that approach the yellow ratio from above, then
    hit it exactly at the ratio. Repair it past the ratio. Garrison the occupiable building, damage
    it past the yellow ratio, then repair it. Repeat the first part with test data: a copy of the
    first type whose art leaves the damaged name of one slot empty.
  - Condition: in the first building, at least one slot animation whose normal and damaged types
    have the same start field, and a stage delay of at least 2 ticks (test data if the stock types
    differ).
  - Observable: the tick of each body-frame change and slot-variant change, and for each slot
    animation of the first building, per tick, its type, its stage and the image frame it shows.
  - Runs: one per building.
  - Expected: the body frame changes to the damaged frame at *h* = yellow exactly, not one step
    later, and the slot animations switch to their damaged art on the same tick, keeping their stage
    (claim 25). For the same-start-field slot, the stage recorded on the swap tick is the stage of
    the tick before (or the stage its playback moves to under S-0109 claims 6 to 10, if the animation advances on that tick), never the stage a new
    animation starts at, and the image frame is the damaged type's start field plus that stage. A run in which that slot is at the starting stage when the swap happens is void for
    this reading and is repeated with the hit at a different tick.
    Repair restores the normal art on the tick the threshold is crossed. The garrisoned building
    shows normal art, not occupied art, after the repair. In the copy, the imported damaged name
    has been filled with the normal name, so that slot shows normal art when damaged. This is not
    a test of an empty-name swap retaining its old animation.
  - Falsified if: a switch comes one damage step early or late, the same-start-field slot animation
    restarts at the starting stage or at any stage other than its kept one, the repaired garrisoned building shows occupied art, or the slot without a damaged name
    vanishes when the copy is damaged.
- **B3 (claim 2).** The occupied, hurt tech-level-−1 frame is a departure tested against
  this spec on our engine, not against the original; the remaining threshold and frame tests
  still use the original.
  - Setup: an occupiable civilian building with tech level −1 and an occupiable type with a
    positive tech level. If stock types cannot reach health strictly between the two imported
    condition ratios, use named test rules data setting separated ratios. A map-override copy of
    the first type has tech level 0, to check the hurt threshold at both non-positive tech levels.
  - Procedure: garrison the first, damage it to between the yellow and red ratios, then below red.
    Garrison the second and damage it to between the yellow and red ratios. Repeat with the
    tech-level-0 copy; also record its ungarrisoned damaged frames and a hit exactly at the red ratio.
  - Observable: the body frame index.
  - Runs: one per building.
  - Expected: for the first, body frame 2 between the ratios and body frame 3 at or below red
    on our engine. Body frame 3
    for the second. The tech-level-0 copy shows 2 between the ratios, 3 at or below red while
    occupied, and 1 at or below red while empty.
  - Falsified if: any of those frame indices differs, or equality at red does not count as hurt.
- **B4 (claim 7).**
  - Setup: a building whose type has a bib sprite.
  - Procedure: build it, damage it past the yellow ratio, sell it.
  - Observable: bib visibility and bib frame per tick.
  - Runs: one.
  - Expected: no bib during build-up and selling, the bib from the first body-sprite frame on, bib
    frame 1 when damaged (with default ranges).
  - Falsified if: the bib shows during build-up or selling, is missing after completion, or keeps
    bib frame 0 when damaged.
- **B5 (claim 8).**
  - Setup: 1×1, 2×2 and 3×3 buildings placed with their top-left cell at a known cell.
  - Procedure: screenshot each after completion, during build-up, and after damage. Include
    named test bib art if the chosen type has no bib. Determine placement from each sprite's
    full canvas, not the bounding rectangle of its nontransparent pixels.
  - Observable: body, build-up, bib and shadow canvas centres against the projected top corner
    of the foundation.
  - Runs: one per building, repeated at two scroll positions.
  - Expected: each full canvas's centre coincides with the projected top corner, using S-0102's
    centred placement and integer-pixel convention. Transparent margins change visible bounds,
    not this canvas centre.
  - Falsified if: any canvas centre differs from that projected pixel at either scroll position.
- **B6 (claim 11, Terms: variant choice).**
  - Setup: a superweapon building, a refinery with no stored ore, a building with slot animations
    for *idle* and *active-1*, and a building with a sprite (not voxel) turret whose slot animations
    have damaged names. Test data: a building type without a turret whose art gives the *active-1*
    slot a normal name and leaves its damaged name out.
  - Procedure: let each complete. Damage the turret building past the yellow ratio while it is
    still being built, let it complete, then damage it once more without crossing the threshold.
    Damage the test type past the yellow ratio while it is being built, and let it complete.
  - Observable: the slot animations present just after completion, and their variants.
  - Runs: one per building.
  - Expected: the superweapon building shows *super-1* (*super-3* only if less than 15 ticks of
    charge remain). The refinery shows *active-1* and no *idle*. The other building shows *idle* and
    *active-1*. The turret building completes with normal art on every slot animation although
    damaged, and switches to damaged art on the next damage. The test type, damaged when it
    completes, shows its *active-1* slot animation with the normal art.
  - Falsified if: the refinery shows *idle*, the superweapon starts in the wrong slot, the turret
    building shows damaged art right after completion, or the test type shows no *active-1* slot
    animation.
  - Set-up comparison: place stock instances of the refinery, superweapon building and
    sprite-turret building on a tiny skirmish map, owned by a non-player house. Include full-health
    and below-yellow-health instances. Record their slot animation types and variants at the end
    of set-up, before any building turn, with the read-only recorder, then through their first
    turn; compare with the construction-completion runs. Use the common game speed and settings,
    one run per case. Only slots with a valid selected animation name are expected to appear.
    Expect the set-up refinery to retain the idle and active slots supplied by its idle body
    entry, with its fill-level slot also started by grand opening; expect the superweapon and
    turret starts of claim 11 already present before the first turn. A damaged sprite-turret
    building shows normal variants after its turret start. Absence of an expected defined slot,
    or postponement of its start until a turn, falsifies this claim. Later periodic slot changes
    follow their own claims, not a second grand opening.
- **B7 (claims 13, 18).**
  - Setup: a building that needs and draws power, with *freeze-unpowered* and *light-unpowered*
    slot animations, and a superweapon building with an *effect-unpowered* *super-3* slot and a
    *super-low-power* slot (test data if no stock type has these flags). Also a building that needs
    and draws power with a running *production* slot animation (a factory while it builds), and test data
    giving one slot only the light flag, with the freeze flag left out, and a copy whose art also
    sets the freeze flag off.
  - Procedure: let the superweapon reach its charged state. Drop power below demand, then restore it.
  - Observable: per slot, whether it is present and whether its image frame advances, per tick.
  - Runs: one.
  - Expected: within one tick of the loss, the frozen slot animations hold their image frame, the
    light ones vanish, *super-3* vanishes and *super-low-power* appears. The *production* slot
    animation holds its image frame. The slot with only the light flag set holds its image frame; its copy with the freeze
    flag off vanishes. Within one tick of the restore, frozen ones resume, light ones reappear,
    *super-low-power* vanishes and *super-3* reappears.
  - Falsified if: any slot's presence or image-frame advancement differs from the stated
    loss/return result, a reaction comes more than one tick after the power change, the
    *production* slot animation keeps advancing, or the slot with only the light flag set vanishes.
- **B8 (claims 15, 16, empty storage only).**
  - Setup: a refinery and a silo-fill type with named *special-1* art, with positive capacities.
    A read-only recorder verifies that each building's own stored amount is 0.
  - Procedure: complete each building and observe for 30 ticks. Harvesting may continue, but is
    not a means of filling building storage; credits are paid directly (S-0402 claim 12).
  - Observable: stored amount, the refinery's active slots and the silo's *special-1* slot.
  - Runs: one per building.
  - Expected: while its recorded amount stays 0, the refinery has *active-1* only among its
    fill-selected active slots, and the silo's *special-1* is absent. Nothing here makes
    *special-1* run first, so the run shows its absence, not its stopping.
  - Falsified if: another refinery fill slot shows or silo *special-1* persists with stored amount 0.
    This scenario supplies no backing for positive fill or fill thresholds. Those remain unmeasured
    until a skirmish path filling the building's own storage is found.
- **B9 (claim 18).**
  - Setup: a superweapon building whose type's charged-animation time is at most 990 game minutes.
  - Procedure: watch it from completion to firing and back to charging.
  - Observable: the tick each superweapon slot starts and stops, and the remaining charge ticks.
  - Runs: one.
  - Expected: *super-1* while charging, *super-2* from the first tick where *r* × *κ* ≤ *t_c*
    (F6), *super-3* when *super-2* finishes, *super-4* after firing (when the charge restarts), then
    *super-1* when *super-4* finishes.
  - Falsified if: *super-2* starts on a different building-update tick from the F6 comparison
    becoming true, or any of the stated slot starts/stops or natural-end follow-ups differs.
- **B10 (claims 22, 23).**
  - Setup: a war factory with a deploying sprite, and a factory with roof-deploying art. For the door
    image, named test art gives a stock factory a door sprite with 8 visually distinct image frames,
    a door-stage count of 4, and a copy marked as a damaged-door type. These counts are invented
    test values. Both factories use the same door sprite; only the damaged-door flag differs.
    Named test art also gives one factory copy ordinary deploying and under-door sprites but
    no roof-deploying or under-roof-door sprites, and another copy no deploying sprite.
    Choose visible under-door pixels that extend beyond the body's opaque pixels.
  - Procedure: build a vehicle at the first, and a jumpjet or balloon-hover unit at the second.
    Build a jumpjet or balloon-hover unit at the no-roof-art copy, and an ordinary vehicle at
    the no-deploying-art copy. At the door factory and its copy, build a unit as well.
    Repeat every unload after damaging its factory past the yellow ratio.
  - Observable: the body sprite and under-door image during each unload, and the door image's
    presence and door frame per tick, with the door state and the remaining ticks of its movement.
  - Runs: one per factory and unit.
  - Expected: the deploying sprite replaces the body during the vehicle unload, and the roof variant
    appears only for the jumpjet or balloon-hover unit. At both missing-replacement-art copies
    the body sprite stays. The no-roof-art copy shows no under-door image, despite having the
    ordinary under-door sprite. An available under-door or under-roof-door image shows frame 0
    undamaged and frame 1 when the factory is damaged. The door image is present on
    every tick of the unload, including the ticks before the door starts to open and after it has
    closed, and absent outside the unload. Its door frame rises from 0 to *D* − 1 while the door
    opens, stays at *D* − 1 while it is open and falls back while it closes, each door frame
    following F8. Damaged, it is the same plus *D* for the damaged-door type, and unchanged otherwise.
  - Falsified if: the body sprite stays during the first factory's vehicle unload despite
    available deploying art, available roof art is absent for the second factory's roof-unit
    unload, the ordinary deploying sprite appears for a jumpjet or balloon-hover unit,
    either missing-replacement-art copy replaces
    its body, the no-roof-art copy draws the ordinary under-door image, an under-door damage
    frame differs, the door image is missing on a tick of the unload
    or shown outside it, or its door frames differ from F8 in any recorded door state, including the damaged
    copy's addition of *D* and the unmarked copy's unchanged frames.
- **B11 (claims 6, 13, 21, 28).**
  - Setup: game speeds 0 and 4. Two named test-art copies of a stock non-turret building: each
    has an idle range of three frames with delay 3 ticks, invented test values, and a sprite with
    enough distinct body frames and matching shadows. One art copy is normalised, the other not.
    Use a type that needs and draws power, with advancing freeze-unpowered slot art. A third copy
    toggles only the extra-damage-stage flag relative to the first.
  - Procedure: build each and log body stages from its deferred construction-to-idle change
    through at least three idle wraps.
    While its house remains short of power, switch that building off and then on through the
    normal power-toggle order (S-0405 claim 10). Keep the house shortfall unchanged throughout. In
    a further pulse run at speed 4, with the house at ample power, switch the building off, hit it
    with a pulse while it is switched off (from a stock input, or else a pulse created outside
    the stock inputs, as B22 says; without a pulse, the pulse run is reported unreached), and keep
    it switched off until at
    least 30 ticks after
    the pulse's end; log the ticks of the switch-off, the hit and the pulse's end. The toggled-flag run is required, since it is the only
    backing of claim 21: run the copy through the same procedure as its original type, undamaged
    and then damaged past the yellow ratio, at both speeds.
  - Observable: idle body-frame period in ticks, whether the frozen slot animations advance, and,
    for the toggled copy and its original, every quantity of the common setup's per-tick record.
    The recorder also distinguishes initial state entry from the first idle wrap; the wrap's
    following delay is the period under test.
  - Runs: one per building and speed, one per speed for the toggled copy, and one pulse run.
  - Expected: the deferred idle-entry delay and each delay after an idle wrap follow N(*d*, *s*)
    in both marked and unmarked cases. Use the imported stage count to separate an image-cycle
    period from one stage delay. The frozen slot animations resume
    when the building is re-enabled although the power is still out. In the pulse run the frozen
    slot animations resume on the tick the pulse ends or the next, although the building is still
    switched off, and keep advancing. The toggled copy's record matches its original's on every tick.
  - Falsified if: either entry or post-wrap stage delay differs from N(*d*, *s*), the slot
    animations stay frozen until power comes back, the pulse run's slots stay frozen after the
    pulse ends while the building is switched off, or the toggled copy's record differs.
- **B12 (claims 3, 9).** Damaged-frame clamp and corresponding-shadow checks are departures
  tested against this spec on our engine, not against the original; undamaged frames and
  per-type shadow suppression still use the original.
  - Setup: test data. A test building whose body sprite has *N* = 2 image frames (one body, one
    shadow), and a second with *N* = 3, both with default ranges and each image frame visually
    distinct. A third
    test type marked to suppress shadows.
  - Procedure: place each, then damage it past the yellow ratio.
  - Observable: the image drawn as the body, and whether a shadow is drawn.
  - Runs: one per sprite.
  - Expected: undamaged, image frame 0 with shadow frame 1 for both sprites. Damaged on our
    engine, both sprites still show body frame 0 and shadow frame 1: ⌊N/2⌋ − 1 = 0 in both
    cases. No shadow for the third type.
  - Falsified if: any recorded body or shadow index differs from the expected indices,
    either damaged sprite lacks its corresponding shadow, or the shadow-suppressed type has one.
- **B13 (claims 2, 4).** Selling direction is a departure tested against this spec on our engine,
  not against the original; build-up direction and special gate images still use the original.
  - Setup: a stock gate and a friendly ground unit, with the common setup's settings.
  - Procedure: build the gate, send the unit through it and let it close. Repeat after damaging
    it past the yellow ratio. In each health condition, also hold the unit on a foundation cell
    after the door reaches rest open, then move it clear. Finally sell the gate.
  - Observable: body image-frame index per tick, current behaviour, door state, movement-start
    tick and bib visibility. Use the recorded imported gate-stage count for *G*.
  - Runs: one per health condition, plus one sale.
  - Expected: the build-up plays from its last build-up frame to its first; on our engine the
    sale also plays backwards, with frame *a + c − k − 1* and unchanged sale completion timing
    under S-0408 claim 17. While *open* is current, opening and closing images follow the
    positions of S-0215 claim 35 d1, plus *G* + 1 when damaged. At rest open they remain
    *G* − 1 undamaged and 2*G* damaged, including throughout the hold. At rest closed the
    ordinary non-occupiable gate frame is 0 undamaged or *G* + 1 damaged. No bib is drawn with
    a special gate image. Reversal and zero-time image checks use S-0215 claim 35 d2–d3 and
    scenario B28 i–j, with that scenario's setup, observables and falsification criteria.
  - Falsified if: either construction sequence has the wrong direction, any special gate image
    differs from claim 4 for the recorded behaviour and door state, the rest-open image returns
    to the closed image, the rest-closed image differs from claim 2, or a bib shows with the
    special gate image.
- **B14 (claims 10 and 28).**
  - Setup: test data. A test building with one slot animation whose displacement is (0, 0), and a
    copy with displacement (10, −6). For the pack-up, a test copy of the construction-yard type
    with the same slot animation, which the human house deploys from its vehicle, with room
    around it for the vehicle to return, and the game option that lets construction yards be
    packed up again on (S-0213 claim 12).
  - Procedure: place both, let them complete, screenshot them, then sell one. In a second run,
    select the deployed yard copy and give a move click on an open cell, which sets its guard
    spot and issues the sell order, so that it packs up (S-0213 claim 12); capture every tick
    from the order to its removal.
  - Observable: the slot animation's screen position relative to the image anchor, and its
    visibility during the sale and the pack-up.
  - Runs: two.
  - Expected: the first is centred on the anchor, the second is 10 pixels right and 6 pixels up of
    it. The slot animations of the building being sold stay drawn until the first tick of its
    build-down and are hidden from that tick until the building is removed. The packing copy's
    slot animation is drawn on every tick before its build-down starts and on none from then on.
  - Falsified if: the displacement differs from the given pixels, or a slot animation of the
    sold or the packing building is visible after the build-down starts, or hidden before it
    while no other rule hides it (a slot whose art has ended its passes is not counted).
  - Note: with no docked unit holding the sale, the build-down starts at the sale's second
    processing, one tick after the first (S-0408 claim 6), so B14 tells "hidden from the
    build-down" from "hidden for the whole sale" only on that one tick, which its screenshots need
    not catch. B22 part (h) separates them over a longer hold.
- **B15 (claim 14).**
  - Setup: a human house with a spy, and an enemy bystander owning a power plant and a building
    marked for special power behaviour, with *low-power* and a running *active-1* marked
    special-outage. Named test art gives distinct slot sprites and turns off its other power flags.
    Named test map triggers create and remove enough enemy consumers to cause and end a shortfall.
    The spy, enemy plant, consumers and special-power building are named test objects in the
    observed area; all unrelated bystander activity voids the run.
  - Procedure: observe the trigger-caused ordinary shortfall and its recovery. Then order the
    human spy into the enemy plant and observe the blackout and recovery.
  - Observable: whether *low-power* and the *special-outage* slot are present, per tick.
  - Runs: one.
  - Expected: ordinary loss does not start *low-power* or stop the special-outage slot through
    the special-outage rule. Ordinary power return stops *low-power* and starts special-outage
    slots, as does return from blackout. During the spy blackout, *low-power* appears and the
    special-outage slot vanishes. Disable that slot's other power flags in the named test art,
    so their independent effects cannot imitate a special-outage transition.
  - Falsified if: ordinary loss starts *low-power* or removes that slot, blackout fails to do
    so, or power return leaves *low-power* present or a named special-outage slot absent.
- **B16 (claim 17).**
  - Setup: a stock type that absorbs infantry and gives extra power per held infantry. Test data:
    an infantry type of size 0.75 and a second of size 1.5, invented exact binary values. The
    absorbing building has room for two passengers and positive extra power.
  - Procedure: use a fresh completed building per case. Send in one stock infantry, one size-0.75
    infantry, two size-0.75 infantry, or one size-1.5 infantry. Record each intake and next update.
    Then give the owner's deploy order to release the passengers, using S-0605 claims 11 to 13
    and scenario A5's voluntary-release setup, with open exits. Keep recording through the final
    passenger's exit and the next building update. Keep the common setup's game speed and runs.
  - Observable: held passenger sizes and count, exit ticks, and which of *active-1* and
    *active-2* is present, per tick.
  - Runs: one per passenger configuration.
  - Expected: *active-1* while empty; the stock-size case follows F9. One or two size-0.75
    passengers retain *active-1*, because each addition is truncated. One size-1.5 passenger gives
    *active-2*. Any necessary swap is visible at the next building update, at most one tick after intake
    or successful release. After the final passenger leaves, *active-2* stops and *active-1*
    runs by that update. The sub-unit-size cases retain *active-1* throughout their release.
  - Falsified if: both or neither slot is present after that update, the swap is later, two
    size-0.75 passengers give *active-2*, one size-1.5 passenger retains *active-1*, or
    *active-2* persists after the update following the final successful release.
- **B17 (claim 19).**
  - Setup: a building with a sprite (not voxel) turret, and enemy targets placed at the eight
    compass directions around it.
  - Procedure: let it aim at each target in turn, recording while it turns as well as after
    settling. Choose a turn rate low enough to visit all 32 displayed facing bins.
  - Observable: the *turret* slot frame against the exact turret facing from the read-only
    recorder, including the ticks on either side of each rounding boundary.
  - Runs: one per direction.
  - Expected: turret frame (28 − *j*) mod 32. Facing north (*j* = 0) gives turret frame 28, east
    (*j* = 8) turret frame 20.
  - Falsified if: any recorded turret image frame differs from (28 − *j*) mod 32 using the
    recorded facing and claim 19's rounding, including at a bin boundary.
- **B18 (claims 20, 25).**
  - Setup: a land repair depot whose art names distinct test animations for *idle*, *active-1*,
    *production* and *special-1* to *special-3* (non-looping *special-1* and *special-3* in parts a
    to g; part h replaces this art with its own, given there), with
    *freeze-unpowered* left on and the owner powered. Damaged vehicles of a house with ample
    credits and of a house with 0 credits. A naval yard with the repair flag and the same test
    art, with a damaged vessel. A delayed-fire type with *special-1* and *active-1* slot
    animations (test data if no stock type has the mark). Use a read-only recorder for the
    repair phase and processing ticks. Choose enough missing vehicle health and a short enough
    *special-1* duration to observe its natural end during funded repair. Repeat a, b and c with
    longer named *special-1* art so it also survives the departure or contact loss, and, in b,
    the working step on which the funds run out. Repeat a
    with the depot damaged past the yellow ratio and distinct damaged slot variants.
  - Procedure and parts, each recording the depot's slot animations, its repair phase,
    the vehicle's health and the paying house's credits per tick, from the
    order until well after the vehicle has left:
    - a. Order the funded vehicle onto the depot and let it be repaired to full health and sent away.
    - b. As a, but with the 0-credit house; after some *cannot pay* retries, give that house
      credits. Then, with a house whose credits pay for only a few steps, let the funds run out
      during the working phase and restore them after some retries.
    - c. As a, but destroy the vehicle during the working phase.
    - d. Order a vehicle onto the depot and hold it in waiting away from the docking point as
      S-0314 scenario H4 part h run 7 does: on the first tick after its movement arrival on
      which the recorder shows it docked, its contact held and the depot waiting, order it 6
      cells away; destroy it once it stands still there and the recorder shows a waiting
      processing after the order that did not accept it, with the contact still held. A repeat
      in which the depot leaves waiting before the order, or the order releases the contact, is
      inconclusive and is run again, from a different starting cell. Use S-0314 H18 b's
      tethered vehicle: an untethered one loses its contact on the order (S-0314 claim 20).
      A loss after
      that acceptance, in settling before the first paid step, is S-0314 scenario H4 part h,
      run 5.
    - e. As a, at the naval yard with the vessel.
    - f. Let the delayed-fire type fire.
    - g. Let a funded vehicle enter working, then exhaust its owner's credits so the next
      repair step returns the depot to settling and starts *special-3*. The depot's first
      settling retry follows 1 tick after that failing step and answers *cannot pay*, so its next
      processing is scheduled *D*<sub>p</sub> ticks after that retry (S-0314 claim 6's table; use a depot
      processing delay *D*<sub>p</sub> of at least 10 ticks). Destroy the vehicle 5 ticks after that retry,
      well inside the gap. Use named test art with a deterministic *special-3* duration that
      outlasts the scheduled processing by at least 10 ticks, and a non-looping *special-1*
      whose duration outlasts the run from the first paid step to the depot's return to
      *guard*, so that no *special-2* starts while the vehicle is in contact. Repeat with the view
      scrolled away from the depot during the wait, keeping the recorder active.
    - h. (Q-1620) Named test art with a *special-3* of several passes (a positive loop count).
      Its *idle*, *active-1* and *production* animations are the setup's, unchanged; only
      *special-1* to *special-3* are given per run below.
      *Run 1:* as a, with a rally point and a *special-1* that ends during working; the last
      working step answers *done* and sends the vehicle away. *Run 2:* a vehicle one step short
      of full health, no rally point and every exit cell blocked. The settling step then answers
      *done*, the depot starts working, and its first working step answers *not needed*; that
      step's departure fails and the vehicle stays in contact in settling. Use a *special-1*
      that outlasts the working phase but ends during this blocked wait, so that *special-2*
      starts. Free one exit cell *W* ticks after the failed working step, with *W* fixed in
      advance and long enough that the recorder shows *special-2* started before the freeing.
      *W* is also at least the length of one pass of *special-3*, so that the *special-3* the
      later restart replaces has finished at least one pass by then. Record the ticks of the
      freeing and of the send-away, the passes the replaced *special-3* has finished at the
      restart, and the image frame the new *special-3* shows at the end of the restart tick.
      The next settling processing
      after the freeing, at most *D*<sub>p</sub> ticks later, reports *not needed* and sends the
      vehicle away, and the processing after it follows *D*<sub>p</sub> ticks after that. The
      send-away tick depends on the retry phase after the exit is freed, as in run 3, so the
      passes below are set from a pilot run of the same set-up that records it; a run in which
      *special-2* has ended before the processing after the send-away is inconclusive. So give
      *special-2* enough passes to run for at least 2 *D*<sub>p</sub> + 5 ticks after the freeing,
      and *special-3* enough passes that the one started by the failed working step runs for at
      least *W* + 2 *D*<sub>p</sub> + 5 ticks. *Run 3:* as run 2, but with a *special-1* that still
      runs at that send-away and ends before the processing *D*<sub>p</sub> ticks after it. The
      send-away tick depends on the retry phase after the exit is freed, so the *special-1*
      duration is set from a pilot run of the same set-up that records that tick; a run in which
      *special-1* has ended by the send-away, or still runs at the next processing, is
      inconclusive. Run 2's requirement that *special-2* has started before the freeing does not
      apply. *Run 4:* a working send-away on *not needed*,
      with a vehicle one step short of full health, a rally point and no blocked exit, so that the
      settling step answers *done*, the depot starts working, and its first working step answers
      *not needed* and sends the vehicle away. Its art: a non-looping *special-1* that still runs
      at that send-away, so that no *special-2* starts while the vehicle is in contact, and a
      *special-3* with enough passes that the one started at that working send-away still runs
      at the next processing, which comes 1 tick later (a working step's outcome is followed 1
      tick later, S-0314 claim 6's table). Run 4 has no blocked wait, so *special-2* does not start before its
      send-away; it shows only that *special-3* does not restart, not the
      *special-2* row. *Run 5 (naval yard):* runs 1 to 4 at the naval
      yard with the vessel, with every water exit cell blocked in runs 2 and 3.
    - i. *special-2* started from outside the repair cycle. Named test art for the depot sets
      *freeze-unpowered* off and *light-unpowered* on for *special-2* only (claim 13), and the
      depot type needs and draws power. *Run 1:* with no vehicle at the depot and the depot in
      *guard*, take the owner's power below its demand and then restore it. *Run 2:* run 1 at
      the naval yard, given the same art and power need. *Run 3:* run 4 of part h with this art,
      with the owner's power taken below demand before the working send-away and restored at
      the tick of that send-away, set from a pilot run of the same set-up. Run 3 is an attempt.
      The recorder reads the slots once per tick, after every object's turn in that tick, so it
      cannot order two events of one tick by itself. The working step that sends the vehicle
      away stops a running *special-2* (claim 20), so a *special-2* that the power return starts
      before that step on the send-away tick is gone by that tick's reading. A *special-2* shown
      running in the reading of the send-away tick was therefore started after the working step.
      The run backs the row only in that case; when *special-2* first shows in a later reading,
      or not at all, the run is inconclusive and backs nothing. Because a building reacts to a
      power change on the tick of the change or the next one (claim 13), the slots that stay on
      across the power return (*special-3* and *special-1*, which have *freeze-unpowered* on)
      may resume on either tick; the run records the start tick of every slot event after the
      power return, including theirs, and the pilot run records the same, so that a one-tick
      shift in their resumption is not read as a change made by the processing.
    - j. *production* at a combined-role depot. The named non-stock test type of S-0314 H23: a
      repair depot that also has a role starting its *production* slot, with a finite
      *production* duration. Use the refinery-docking role: in every run the role starts
      *production* when one of the owner's harvesters, docked at the building to unload, has
      emptied its load. Time the harvester's return so that it empties in the state each run
      names. *Run 1:* with no vehicle at the depot for repair and the depot in *guard*, make
      the role start *production*. *Run 2:* as S-0314 H23 part a, with *production* started by the role
      just before the vehicle docks, by H23's input sequence, so that it still runs in the
      depot's settling phase; destroy the vehicle in settling while *production* runs. *Run 3:* as part a, and make the role start *production*
      during the working phase, before the step that answers *done*. If the combined roles
      cannot start *production* in these states through that input (for example because the
      repaired vehicle and the harvester cannot both be in contact), the run is reported
      unreached, not confirmed.
    - k. Occupied variants. A named test depot type that infantry can occupy, with distinct
      occupied variants of its slot art; occupy it, then run part a. If the type cannot be
      occupied, the part is reported unreached.
  - Observable: the tick of each slot start and stop, the tick of the first health gain or credit
    deduction, and the delayed-fire building's slot animations. In g, also record contact,
    the depot's current behaviour and processing ticks, and *special-3*'s lifetime. In h, also
    record the depot's phase at each processing, so that the processing after each send-away is
    seen to be in settling (S-0314 claim 22).
  - Runs: one per part and named art or health variant; g has one visible and one offscreen run,
    h has its eight runs (run 5 repeats runs 1 to 4 at the naval yard), i and j have their three runs each, and k has one run.
  - Expected: in a, c and e, *special-1* appears on the tick of the first paid repair step, not
    on docking, and *active-1* and *idle* disappear on that tick; *special-2* follows *special-1*'s
    natural end while the vehicle is in contact. On the step that sends the vehicle away (a, e) or
    the depot's first processing after it is destroyed, the one S-0314 claim 6's table had
    already scheduled (c), *special-2* disappears and *special-3* and *active-1*
    appear; *special-1*, if still running, runs to its end and *idle* follows it, and *idle*
    follows *special-3*'s end. In b, *special-1* does not appear during *cannot pay* settling and
    appears on the first paid step after credits arrive. When funds run out during working,
    *special-2* stops and *special-3* and *active-1* start on the failing step; on the first paid
    step after the credits return, *special-1* starts again and *active-1* and *idle* stop. In
    b's repeat with the longer *special-1* art, *special-1* is still running after the failing
    step: that step starts *special-3* and *active-1* and does not stop *special-1*, and the first
    paid step after the credits return starts *special-1* again, replacing the running one
    (claim 25). In d, when the recorder shows waiting at the loss (otherwise the part is
    inconclusive), *special-1* never appears, and *special-3* and *active-1* appear when the
    depot finds no contact; the depot is in *guard* 1 tick later, while *special-3* still runs.
    Outside part j, *production* never appears.
    Every depot start uses the normal variant while undamaged and the damaged variant in
    the damaged repeat. In f, the delayed-fire building restarts *active-1* when *special-1*
    finishes on its own.
    In g, the first processing after contact is lost is the one S-0314 claim 6's table had
    already scheduled; from then on the depot remains in repair, rechecking every tick while
    *special-3* runs, and returns to *guard* at its first processing with that slot no longer
    running. The art-derived lifetime and this return are unchanged when scrolled offscreen.
    In h, *special-3*'s passes are counted from the step that sends the vehicle away in run 1, and
    from the working step whose departure failed in runs 2 and 3, and from the working step that
    sends the vehicle away in run 4: the processing after the run 1 and run 4 send-aways, 1 tick
    after each, and, in run 3, the processing after the settling send-away, *D*<sub>p</sub> ticks
    after it, restart nothing: no *special-3* start and no *active-1* start occurs at that
    processing. Those start events are what part h judges. In run 2, where the restart replaces a
    running *special-3*, its end tick is judged too. The replaced *special-3* is removed during the
    restart tick, so its own frame on that tick is not read. As in B2's swap-tick reading, the
    recorder logs the image frame the new *special-3* shows at the end of the restart tick and the
    ticks of its later advances; that frame less the type's start field is the kept stage, or the stage it advances to if an advance falls
    on that tick, and the prediction starts from it, with the new animation's advances spaced by
    its stage delay. Both are of the same type, so the kept stage shows the frame the replaced one
    showed. The test *special-3* plays forward,
    neither reversed nor ping-pong, so the new *special-3* runs from that frame to the end of its
    first pass, which counts as one whole pass, then plays the rest of its creation-time passes (claim 25,
    S-0109 claims 5 to 7). It ends within one
    tick of that predicted tick (the tolerance allows for a restart moving its end by a tick). A
    count carried over from the replaced animation would end it a whole pass or more earlier; not
    counting the partial first pass would end it a whole pass later. The prediction starts from
    the new animation's own logged frame, so this run does not test whether the stage is kept. A
    run in which the
    replaced *special-3* has finished no pass by the restart is void for the carried-over
    reading: a carried-over count of zero would end it at the same tick as a fresh count. A run
    whose frame logged at the end of the restart tick is the first frame of a pass is void for the
    partial-pass reading: with no partial first pass, counting it or not gives
    the same end.
    Elsewhere *special-3*'s end tick is recorded only. In run 3, *special-1* ends
    after the settling send-away and before that processing, with the depot out of contact, so
    *idle* starts at that end by the out-of-contact *special-1* row below, while *special-3* may
    still run; that start is the row's, not a change made by the depot's processing. In run 2,
    *special-2* runs until the first processing after the settling send-away, *D*<sub>p</sub> ticks
    later. That processing stops it, starts *active-1* again, replacing the one still running from
    the failed working step if it runs (the recorder logs a start event either way), and restarts
    *special-3* (a start event, judged with the others). Run 5 gives at the naval yard the
    results of runs 1 to 4. In i run 1, *special-2* starts when power returns, and on that tick or
    the next (the every-tick row) it stops and *special-3* starts; in run 2, at the naval yard,
    *special-2* keeps running. A run 1 or run 2 in which *special-2* has already ended on its own
    before the every-tick check is inconclusive and is repeated with more passes. In i run
    3, the processing 1 tick after the send-away stops *special-2* and starts *special-3* and
    *active-1*. In j run 1, *production* stops and *special-3* starts on the tick it starts or the
    next; in run 2, the first processing after the loss stops *production* and starts *special-3*
    and *active-1*; in run 3, the step that answers *done* stops *production*. A j run 2 or run 3 in which
    *production* has already ended on its own before that processing or step is inconclusive and
    is repeated with a longer *production* duration. In k every start
    uses the normal or damaged variant, never an occupied one.
  - Falsified if: *special-1* appears on docking or on any tick other than a working-phase entry,
    *active-1* or *idle* survives the entry, *special-2* does not follow natural *special-1*
    completion during contact, *special-3* and *active-1* do not start on leaving the working
    phase, the naval yard differs from the land depot in these rows, *production* appears outside part j, *idle*
    does not follow natural *special-3* completion, or the delayed-fire building fails to
    restart *active-1*. Part b is also falsified by *special-1* starting during unpaid settling,
    failing to restart on re-entry into working, or *active-1* or *idle* surviving that entry.
    Part d, when the recorder shows waiting at the loss (otherwise it is inconclusive and
    backs nothing), is falsified if the waiting depot starts *special-1*, fails to start *special-3* and
    *active-1* at its first no-contact processing, or is not in *guard* 1 tick after it.
    Any wrong normal/damaged variant, or premature stopping of the long *special-1* at
    departure, at contact loss or at b's failing step, also falsifies the corresponding part.
    Part g is falsified by any depot processing between the loss and the processing *D*<sub>p</sub> ticks
    after the retry, by a first no-contact processing on any other tick while the depot misses
    no turn, by a return to *guard*
    while *special-3* still runs,
    a recheck interval other than 1 tick after that first no-contact processing, failure to return at the first
    processing after the slot is no longer running, or a lifetime or return changed by visibility.
    Part h is falsified if *special-3* restarts, or *active-1* starts, at the
    processing after the send-away in run 1, run 3 or run 4; if run 2 shows no such restart while
    *special-2* runs, or a replacing *special-3* there that ends more than one tick before or after
    the predicted tick; if the settling send-away step itself changes any slot; or if run 5 differs
    at the naval yard from runs 1 to 4; or if, in run 3, *idle* does not start when *special-1*
    ends. A run 2 in which *special-2* has already ended before that processing is inconclusive
    and is repeated with more passes. Run 2 is also falsified if that processing shows no
    *active-1* start event. The stop of *special-2* and the start of *active-1* there
    are judged whether or not *special-3* still runs, and that start event whether or not it
    replaces a running *active-1*; only the replacement of a running
    *special-3* is judged just when it still runs at that processing, and is otherwise
    inconclusive. Parts i to k are falsified by any result other than the one expected for
    them, including a naval yard that stops *special-2* in i run 2.

- **B19 (claim 5, theater-flag rule).** Clamped displayed frames are a departure tested
  against this spec on our engine, not against the original; the range and stage delay still
  use the original.
  - Setup: test data. In a copy of the art data, a test building type with the theater flag set
    (and not the new-theater flag) and a build-up name *X*. In the test expansion archive, the file
    *X* ‖ .E for the theater of the test map (S-0007 claim 12), a build-up sprite of *N_b* image
    frames, *N_b* even, 4 ≤ *N_b* ≤ 74, and each image frame visually distinct. These limits
    are invented test conditions that keep the theater-refresh delay positive. A copy of the type without the
    theater flag whose build-up sprite is the same image frames under the name *X* ‖ .shp.
  - Procedure: place each type and log its build-up frame per tick. Repeat with a different
    build-up time in the rules.
  - Observable: the build-up frames shown and the ticks per build-up frame.
  - Runs: one per type and build-up time.
  - Expected: the theater-flag type's recorded stage reaches *N_b* − 1 after
    (*N_b* − 1) × ⌊75 / *N_b*⌋ ticks, whatever the build-up time. While build-up is still drawn,
    on our engine its image frame is ⌊*N_b*/2⌋ − 1 with its corresponding shadow for each
    recorded stage from ⌊*N_b*/2⌋ on. The copy
    without the flag runs through ⌊*N_b*/2⌋ build-up frames at the delay of F1, which changes with the build-up
    time.
  - Falsified if: the theater-flag type's stage delay differs from ⌊75 / *N_b*⌋ or changes with
    build-up time, its recorded construction range has only ⌊*N_b*/2⌋ stages, or our engine
    displays a frame other than the last body frame or lacks its corresponding shadow in the
    clamped stages. Construction
    handover timing is recorded separately, as in B1, not inferred from image disappearance.

- **B20 (claims 3, 23, non-positive and out-of-range door frames).**
  - Setup: named test art gives otherwise identical stock factory copies a door sprite with
    8 visually distinct image frames. Door-stage counts are −2, 0 and 10 image frames, invented
    test values. For each count, use one damaged-door type and one unmarked copy. The door
    sprite's pixels must be visible when frame 0 is drawn, outside the body's opaque pixels.
  - Procedure: at each factory, unload one vehicle undamaged, damage the factory past the yellow
    ratio, and unload another. Record all door states, including the open state. Leave the body
    and under-door sprites identical across copies.
  - Observable: requested door frame from the read-only tick-labelled recorder and visible door
    pixels from the screen capture. This distinguishes a missing image from a request for frame 0.
  - Runs: one undamaged and one damaged unload per copy.
  - Expected: with count 0, door frame 0 in every state in both health conditions. With count −2,
    frame 0 except for the damaged damaged-door copy, which requests frame −2 and shows no door
    pixels in any state. With count 10, F8 applies: the undamaged open state requests frame 9
    and shows no door pixels; the damaged damaged-door copy requests frames 10 or greater
    throughout the unload and likewise shows no door pixels. The unmarked damaged copy behaves
    like its undamaged run. Other building images remain visible.
  - Falsified if: a negative or high out-of-range request draws any door pixels, a non-positive
    count produces a request other than the stated 0 or −2, the unmarked damaged copy gains a
    damage term, or the count-10 frames differ from F8.

- **B21 (claim 24, door-art input independence).**
  - Setup: common setup with named test rules and art. Give copies of a stock vehicle factory
    distinct image names that differ from their type names. Every copy uses the same door sprite
    with 12 visibly distinct frames, invented test art. In their image-name art sections, give
    counts 3 and 4, each with the damaged-door flag false and true. Give another copy neither
    setting. In the type-name art sections and building rules sections, give conflicting counts
    and flags, using the exact keys in Import mapping. Keep all door movement times identical.
    Additional copies use the integer spelling `4abc` and boolean spelling `Yes please` in
    the image-name art section to discriminate the general value forms.
  - Procedure: unload a vehicle from each copy undamaged, then damage it past the yellow ratio
    and unload another. Repeat the absent-setting copy with a door sprite of 8 frames, keeping
    the settings absent. Remove all named test files afterwards.
  - Observable: visible door frame per tick, door state, movement start and finish ticks, and
    health condition, from tick-labelled screen capture and read-only recording.
  - Runs: one undamaged and one damaged unload per copy and sprite variant.
  - Expected: F8 uses the image-section counts and flags only. The open frame is 2 or 3
    undamaged, and 5 or 7 for the corresponding damaged marked copy. Unmarked copies retain
    their undamaged frames. The absent-setting copy shows frame 0 in every door state and
    health condition with either sprite. The partial integer and boolean spellings act as 4
    and true, respectively. All copies have the same movement duration under S-0417 Formulas F2.
  - Falsified if: a conflicting type-name or rules section determines the frames, either absent
    setting is derived from the sprite, the partial spellings give a different result, any frame
    differs from F8, or changing only the count changes door movement duration.
- **B22 (claims 11, 13, 19, 26 to 28, buildings that need an engineer).**
  - Setup: common setup with named test rules and art and a purpose-made test map; game speed 4.
    Test type T is a copy of a stock tech building with the needs-engineer flag and no
    needs-power flag. Test type P is the same with the needs-power flag and a positive power
    drain. Test type Q is a copy of a stock building that needs and draws power, without the
    needs-engineer flag. Each has named test slot art with deterministic playback, five visibly
    distinct image frames, a stage delay of 3 ticks and no loop delay (invented test values). The test slot art
    is not reversed, so a slot's first image frame is the frame it was created with (S-0109
    claim 8), and it has no pass limit, so it keeps playing for the whole run and no slot ends on its own. The
    *idle* and *active-1* slots keep *freeze-unpowered* on. The *active-2* slot turns it off and
    sets *light-unpowered*. The *active-3* slot turns it off and sets *effect-unpowered*. Each
    slot has a damaged variant with distinct art. Test type R is a copy of a stock refinery with
    no needs-power flag and not marked for special power behaviour. Its *active-1* to *active-4*
    slots all use the same test slot art, with *freeze-unpowered* on; its other slots keep their
    stock art. Test type U is T with a turret whose turret art is a sprite: named test turret art
    with 32 visibly distinct image frames, created at image frame 0. The map's building list
    places T1, P1 and U1 for the neutral house, T2 for the bystander computer house, and T4, T5 and
    T6 for the neutral house, all in view. A map trigger action of code 125 (create a building) creates T3 and P3 for the neutral house in the first tick,
    and in the same tick creates, out of view, a power plant for the neutral house whose output
    exceeds the neutral house's total drain, so that the neutral house is not short of power. The
    record includes the neutral house's power output and drain on every tick.
    For part (j), test type F is a copy of a stock building type that has a free unit, and a map
    trigger action of code 125 creates F for the bystander computer house in the first tick, in view with open
    cells around it.
    The player has engineers and a unit that can damage a building. Parts (e), (k) and (o) need
    a pulse. No stock input that makes one is established (M-0014's pulse is not one), so
    these parts may use a pulse created outside the stock inputs: a harness that starts the game's
    own pulse
    at a stated
    cell, with a stated spread and length. Without a pulse, those parts are reported unreached.
    A map trigger action, fired at least 30 ticks after part (b)'s capture, gives T1 to the
    bystander house. For parts (f) and (g) the player's house has no power plant, so any positive
    drain leaves it short of power. In the ample-power run of (f), and in (h), a map trigger
    creates, in the first tick, a power plant for the player's house whose output exceeds the
    house's total drain (a structure in the map's building list owned by a human player's house is
    dropped, S-0009 claim 27).
    For parts (l) and (m), test type D is a copy of a stock building with a positive power drain
    and neither the needs-power nor the needs-engineer flag, test type E is D with the
    needs-power flag, and test type Z is D with no power drain. Each has T's test slot art. The
    map's building list places D1, E1 and Z1 for the neutral house, in view, with the map's
    switched-on field 0 (S-0009 claim 40 owns its encoding). A map trigger action, fired at least
    60 ticks after the first tick, gives D1 and E1 to the player's house. Before it fires, the
    player builds a building of type Q and no power plant, so that the player's house is short of power
    on every tick from then on; if the record shows no shortfall, the player builds more of type
    Q before the trigger fires. The record includes that house's power output and drain on every
    tick.
    For parts (n) and (o), test type W is a copy of a stock building whose light shows (S-0405
    claim 9), given the needs-engineer flag, a positive power drain, no needs-power flag and T's
    test slot art. The map's
    building list places W7 and W8 for the neutral house, in view. A map trigger action, fired
    at a set time, gives W7 to the bystander house. Every power order of these parts is
    for a building of the player's own house at the order's tick, as S-0214 offers power mode only
    over an own selectable building. Parts (n) and (o) need an input that gives a
    power order in the cycle of a chosen tick (S-0201, *cycle*), such as a scripted input or a
    replay of the recording run with the same seed and the same earlier inputs, plus the one
    power order; without one they are reported unreached. A scripted order applied just before
    the logic of tick F is an order in the cycle of tick F − 1. This is derived from S-0203
    claim 7 and S-0405 claim 7, and M-0014 is consistent with it: there a human's power drain
    from such an order falls in tick F, and the light goes out in tick F.
  - Procedure and parts:
    - (a) Record from the first drawn frame of play for at least 30 ticks, before any capture,
      and log the tick on which the trigger action creates T3 and P3.
    - (b) Capture T1 with an engineer and log the tick of the ownership change.
    - (c) After at least 30 ticks, change T1's owner again to the bystander house by the trigger
      action.
    - (d) Damage T4 below the yellow ratio while it is not manned, then, at least 30 ticks after
      the damage tick, capture it.
    - (e) Hit T5 with the pulse while it is not manned and record past the pulse's end.
    - (f) Capture P1 once into a house with ample power and, in a second run, into a house short
      of power.
    - (g) With the player's house short of power, build Q and record from its completion.
    - (h) At full power, build R and let a loaded harvester dock in it with its way out
      blocked, as S-0405 scenario W6 sets up its refinery. Sell R. Log the tick of the sell
      order, the tick of the sale's first processing and the first tick of the build-down.
    - (i) Within part (a)'s record, read U1's *turret* slot and its turret facing on every tick.
    - (j) Log the tick on which the trigger action creates F and the tick on which F's free unit
      appears, and count F's free units at the end of the run.
    - (k) Capture T6 with an engineer. Once its *idle* and *active-1* slots have advanced to an
      image frame other than their first, hit T6 with the pulse, and while the pulse still holds,
      give T6 to the bystander house by a second map trigger action, no sooner than the stage
      delay of those slots (S-0109 claim 5) after the pulse hit. Log the ticks of the
      capture, the pulse hit, the change and the pulse's end. If the pulse ends before the
      change, or the slots are at their first or their last image frame when the pulse freezes
      them, the run is repeated, so that the frame after the held one is not the first.
    - (l) Record D1, E1 and Z1 from the first drawn frame of play. Log the tick of the trigger
      action that gives D1 and E1 to the player's house, and record at least 30 ticks after it.
      Then switch D1 on by the power order (S-0405 claim 10), log the tick of the order, and
      record at least 60 ticks after it.
    - (m) In the same run, at least 30 ticks after the transfer, switch E1 on by the power order,
      log the tick of the order, and record at least 60 ticks after it.
    - (n) Capture W7 with an engineer at least 30 ticks before the trigger action gives it to the
      bystander house. In a recording run, log the tick *t* of that change of owner. In a further
      run with the same seed and the same earlier inputs, the player gives the power-off order for
      W7 in the cycle of tick *t* − 1, while W7 is still the player's. Record at least 30 ticks
      after *t* + 2.
    - (o) Capture W8 with an engineer, and once its *idle* and *active-1* slots have advanced, hit
      it with the pulse. In a recording run, log the tick *e*: the first tick after the hit at
      whose end a read-only recorder shows W8's light state on again. That is the tick in which
      the pulse's hold reaches 0 (M-0014), so the pulse ends in tick *e*. The light on the
      ground can show one tick later (S-0405 scenario W15), so a screen capture alone does not
      fix *e*. In three further runs with the same seed and the same earlier inputs, the player
      gives the power-off order for W8 in the cycle of tick *e* − 3, of tick *e* and of tick
      *e* + 2, one per run. Record at least 30 ticks after *e* + 2.
    - Remove all named test files afterwards.
  - Observable: per tick, for each test building and each of its four test slots (R: its running
    *active* slot; U1: its *turret* slot and turret facing; F: its free units; T3 also its body
    image frame), whether the slot is drawn, its
    variant and image frame and whether it changed, the owner, and the health condition (W7 and W8:
    also whether the building is switched on and whether its light shows; in the runs of
    parts (l) to (o), also each object that left play in the tick), from a screen
    capture that keeps every drawn frame or a read-only recorder's tick-labelled log. A tick
    named in the expected results is the tick at whose end the state is recorded; a screen
    capture shows that state in the next drawn frame (S-0201 claim 6).
  - Runs: part (a) records at the start of the run that parts (b) and (c) continue, before part
    (b)'s capture; part (c) follows part (b) in that same run, and part (i) is read from part
    (a)'s record. Part (j) shares that run, which lasts at least 300 ticks after the first tick.
    Parts (d), (e), (g) and (k) take one run each, part (f) two, and part (h) up to five attempts, as its expected result says.
    Parts (l) and (m) share one run. Part (n) takes a recording run and one order run, and part
    (o) a recording run and three order runs.
  - Expected:
    - (a) On T1, T2, T4, T5 and P1 the *idle* and *active-1* slots show their first image frame
      on every tick, and *active-2* and *active-3* advance every 3 ticks. On T3 all four slots
      are present and drawn from the tick on which the trigger action creates it, and advance; no
      slot of T3 restarts at any later tick of the run.
      On P3 *idle* and *active-1* hold their first image frame and *active-2* and *active-3*
      advance, while the neutral house's recorded output is at least its drain. If that record
      shows a shortfall in any tick, P3's result does not tell "not manned" from "short of power"
      and is reported as inconclusive.
    - (b) From the tick of the change or the next, T1's *idle* and *active-1* advance to the frame
      after the held one; *active-2* and *active-3* go on without a break; no slot shows a
      restart.
    - (c) No slot's sequence changes at the second change.
    - (d) On the damage tick T4's slots switch to damaged art and all four advance before the
      capture. The capture changes none of them.
    - (e) After the pulse ends T5's *idle* and *active-1* advance although T5 is not manned.
    - (f) In both runs P1's *idle* and *active-1* resume at the capture. With ample power nothing
      else changes. Short of power they keep playing at their stage delay (S-0109 claim 5) after the
      capture, and do not hold again in the building's next update or later, while the house stays short of
      power: the capture is no change of P1's powered state (claims 13 and 26). If the player's
      house's recorded output reaches its drain in any tick of that run, the run is void.
    - (g) Q's *active-2* and *active-3* advance from completion while its house stays short of
      power; its *idle* and *active-1* hold their first frames.
    - (h) R's *active* slot advances every 3 ticks, and is drawn, on every tick from the sell order
      through the whole hold until the first tick of the build-down. From that tick on no slot of
      R is drawn. Every run shows R's slots drawn on the tick of the sale's first processing,
      which is at least 1 tick before the build-down starts (S-0408 claim 6), so every run tells
      "hidden from the build-down" from "hidden for the whole sale". Only a hold of at least
      6 ticks shows the slot advancing rather than held or stopped. No input is known that
      ensures such a hold: the first processing tells every docked unit to leave (S-0408 claim 6),
      and an unloading harvester breaks off at once (S-0402 claim 11). A run with a shorter hold
      is kept for the hiding and repeated with the way out blocked more fully, up to five attempts
      in all. If no attempt gives a hold of at least 6 ticks, the advancing part is reported
      unreached and the sale changing no slot before its build-down stays medium.
    - (i) On every tick U1's *turret* slot shows image frame (28 − *j*) mod 32 for its recorded
      turret facing *j* (claim 19), not the held image frame 0. If the facing the map gives U1
      makes that frame 0, the part is repeated with another map facing; if no facing tried gives
      another frame, the part is reported unreached. The part tells the per-tick setting from a
      hold at the frame the slot was created with. It does not tell it from a frame set once at
      placement and then held, since U1's facing does not change while it is not manned (claim 26
      rates that point medium).
    - (j) F's free unit appears on the tick on which the trigger action creates F, and F has one
      free unit at the end of the run.
    - (k) From the pulse hit T6's *idle* and *active-1* hold the frame they had reached. From the
      tick of the change or the next, while the pulse still holds, the first new frame each
      shows is the frame after that held frame, not its first frame (later passes show the first
      frame again, as any pass does); *active-2* and *active-3* go on without a
      break. The pulse's end changes nothing.
    - (l) From the first tick, the *idle* and *active-1* slots of D1 and E1 show their first image
      frame on every tick, and their *active-2* and *active-3* advance every 3 ticks; all four
      slots of Z1 advance every 3 ticks. D1's and E1's held slots are still held through the
      change of owner and up to their switch-on. In the tick after D1's switch-on order, and not
      later, D1's *idle* and *active-1* advance to the frame after the held one, with no restart
      and no change of variant, and go on advancing. Their clocks are due after the long hold and
      the art has no loop delay, so their first eligible update (S-0109 claim 23) is their turn
      in that tick.
    - (m) In the tick after E1's switch-on order, and not later, as in part (l), E1's *idle* and
      *active-1* advance to the frame after the held one, with no restart, and go on advancing at
      their stage delay to the end of the record, while the record shows the player's house short
      of power; E1's *active-2* and *active-3* advance throughout. A tick of that record without
      a shortfall voids the part.
    - (n) With the order in tick *t* − 1, W7's *idle* and *active-1* advance again, although W7
      stays switched off, at the first tick from *t* on at which at least 3 ticks have passed
      since their last advance (S-0109 claim 23), so no later than tick *t* + 2. Their hold from
      the order until the change is shorter than their stage delay, and the part does not judge
      it.
    - (o) With the order in tick *e* − 3, W8's *idle* and *active-1* show no new frame up to
      the end of tick *e* − 2, and advance again, although W8 stays switched off, no later than
      tick *e* + 1: the pulse's end re-enables W8, and their clocks are due after the pulse's
      hold. With the order in tick *e*, they show no new frame after the end of tick *e*, to the
      end of the record: the part does not judge the
      frame they show at the end of tick *e*. With the order in tick *e* + 2, they show no new
      frame after the end of tick *e* + 2, to the end of the record.
  - Falsified if: an unmanned map-placed building's *freeze-unpowered* slot advances or is not
    drawn before any capture, damage or pulse, or its *active-2* or *active-3* holds; T3 holds,
    its slots first appear on a later tick than the one on which the action creates it, or one of
    them restarts later; F's free unit appears on a later tick than F's creation, or F has none or
    a second one; U1's
    *turret* slot holds image frame 0 while its turret facing gives another frame; P3's
    *idle* or *active-1* advances or its *active-2* or *active-3* holds; a resumed slot
    restarts from its first frame or changes variant at a capture; the second change alters any
    slot; T4's damaged slots hold before the capture; T5 holds after the pulse; P1 holds after a
    capture in either run, including a hold in a later update of the short-power run; the first new frame
    that T6's *idle* or *active-1* shows after the change is its first image frame, or it still
    holds after the change; Q's *active-2* or *active-3* holds while the house is short of
    power; or R's *active* slot stops being drawn at or after the sale's first processing before
    the build-down starts, or holds its frame through a hold of at least 6 ticks, or a slot of R is
    drawn after the build-down starts; a held slot of D1 or E1 advances before its switch-on, a
    slot of Z1 holds, or D1's or E1's *active-2* or *active-3* holds; D1's or E1's held slots stay
    held in the tick after the switch-on order, restart from their first frame or change variant at
    it; E1's resumed slots hold again while its house stays short of power; in part (n), the
    order for W7 in tick *t* − 1 leaves its slots held at the end of tick *t* + 2; or, in part
    (o), the order in
    tick *e* − 3 leaves them advancing before the end of tick *e* − 2 or held at the end of tick
    *e* + 1, or the order in tick *e* or in tick *e* + 2 leaves them advancing after the end of
    that tick.
  - Validity: a counting removal can cost a slot its turn (S-0109 claim 5, S-0203 claim 10). A
    run of parts (l) to (o) in whose record any object leaves play in a tick that a timing
    expectation of the part names is void for that expectation and is repeated. Those parts'
    inputs put no removal near those ticks (their captures and pulse hit come well before), so
    none is expected.
- **B23 (claims 11 and 18, a record that is not granted; Q-2001).**
  - Setup: a tiny skirmish map, stock data, the common game speed and settings, superweapons on.
    Two houses: the human player's house P and one computer player's house C, neither defeated in
    the recorded window. A pre-placed structure of a human player's house is dropped (S-0009
    claim 27), so the set-up parts use C. C has power plants placed at set-up and P builds power
    plants before its parts, so that each house's power output stays above its drain for the
    whole recorded window. The house named in each part holds no superweapon
    building at its start. Type A: test data, a copy of a stock superweapon building type whose
    art names *super-1* and *super-3*, both endless (S-0109 claim 7; test art where stock art
    is not), with its charged-animation time set above 990 so that
    claim 18's check never runs for it. Its superweapon type is power-dependent and has a charge
    time *T* of at least 600 ticks (a test copy of the superweapon type where needed). For type
    A only the slot that each opening starts is judged. Type A′: test data, a copy of A whose
    art names *super-1* to *super-4* (test
    art where stock art lacks one or where it differs from what follows), whose *super-1* and
    *super-3* are endless (S-0109 claim 7), so that neither ends on its own in the recorded
    window (S-0109 claim 16), whose *super-4* plays once and ends in fewer than 200 ticks,
    and whose charged-animation time *t_c* is at least 0, at most 990 and below
    (*T* − 300) × *κ*, with *T*
    the type's full charge time in ticks (claim 18, S-0512 claim 3).
  - Procedure, one run per part:
    - (a) P builds its first building of type A by order.
    - (b) The map places a building of type A for C at set-up.
    - (c) A trigger runs code 125 in play to create a building of type A for P.
    - (d) P builds A, sells it with the charge partly done, and builds A again once the record
      is lost. Repeat with the first one sold when its weapon is ready.
    - (e) As (b), with type A′.
    - (f) The map places for C, in this list order, a building of type A, a building of C's with
      a map-listed upgrade (S-0009 claim 40), and a second building of type A.
    - (g) P builds A and sells it while its record's clock runs with at least 300 ticks left,
      so that the record is lost (S-0512 claim 7) with P holding no other building of the type.
      A trigger then runs code 125 to create a building of type A for P. Let *R_L* be the
      remaining charge ticks recorded at the loss and *k* the ticks from the loss to the opening.
      Run 1: *R_L* − *k* is at least 30. Run 2: *k* is *R_L* or more. Run 3: as run 2, with P
      switching A off (S-0405 claim 10) before the sale, so that the record is suspended and its
      clock stopped at the loss (S-0512 claim 6). A run whose recorded *R_L* and *k* miss its
      condition is void and is run again.
    - (h) As (e), with the map placing the building of type A′ and a power plant for the neutral
      house. A second run places them for the special house.
  - Observable, with the read-only recorder, per tick: the building's superweapon slots, and the
    house's record of the type, granted or not and suspended or not, with its remaining charge
    ticks, and whether each superweapon slot is frozen. Parts (a) to (d)
    and (f): from each grand opening for 10 ticks. Part (g): from the sale to 10 ticks after the
    opening. Parts (e) and (h): from the grand opening for 300 ticks.
  - Expected in the original: (a) and both runs of (d) meet a granted record and start the slot
    that claim 11 gives for the recorded remaining charge at the opening: *super-1* when it is
    15 ticks or more, *super-3* when it is below. (b) and (c) meet a record that is not granted,
    reading 0, and start *super-3*. (e) and both runs of (h) start *super-3*, then after the
    grant *super-4* and, when it finishes, *super-1*, all within the 300 ticks. (f): the first A
    meets no granted record and starts *super-3*; the upgrade's placement grants the type; the
    second A meets the granted record and starts the slot that claim 11 gives for the recorded
    remaining charge (*super-1* when it is 15 ticks or more). (g) meets the lost record, not
    granted, and starts *super-1* in run 1, *super-3* in run 2 (the clock has run out) and
    *super-1* in run 3 (the stopped clock keeps *R_L*). A clock that the loss stopped would give
    *super-1* in run 2, and one that the loss reset to 0 would give *super-3* in runs 1 and 3.
  - Expected with `quirks.ungranted_super_reads_clock` on: as in the original. With it off:
    parts (a) and (d) and the second A of part (f) as in the original. Parts (b), (c), (e), (g)
    and (h) and the first A of part (f) start *super-1*. In (e) and (h) *super-1* runs for all 300
    ticks, since *t_r* stays above *t_c* there.
  - Falsified, for the original and the switch-on default, when (a), (d) or the second A of (f)
    meets a record that is not granted at the opening, or starts a slot other than the one claim 11
    gives for the recorded remaining charge, when (b), (c), (g) or the first A of (f) meets a
    granted record, when (b), (c), (e), a run of (h) or the first A of (f) starts other than
    *super-3*, when a run of (g) starts other than its expected slot, or when the slots of
    (e) or (h) after the grant differ from the original's expectation (in (h), also when the
    record is never granted in the 300 ticks). A switch-off run
    of our engine, when one is judged,
    is falsified when (b), (c), (e), (g), (h) or the first A of (f) starts other than *super-1*,
    or when *super-1* changes within the 300 ticks of (e) or (h). A run in which the record is
    suspended (S-0512 claim 4) or a superweapon slot is frozen (claim 13) at any recorded tick
    is void and is run again, except that in run 3 of (g) the record is suspended and A's
    superweapon slots are frozen (A is switched off before its sale), both by design.

Construction timing in claim 5 uses S-0406 scenario PL11 a to f, including its setup,
read-only per-tick observables and falsification criteria: placed buildings show two runs,
deployed buildings restart at their first construction processing, zero-delay ranges complete,
and positive-delay ranges of one stage or fewer never complete. Stage-23 sale completion uses
S-0408 scenario S10, with its setup, observables and falsification criteria.

The free-unit effect of claim 11 is measured by S-0402 scenario H13, parts a to f, with that
scenario's setup, observables and falsification criteria; no separate unit-placement rule is
specified here.

## Changes for other specs

- **S-0314 claims 6 and 22 (Q-1610).** S-0108 claim 20's timing sentence relies on S-0314 claim
  6's per-phase processing spacing and on claim 22's statement that a depot left without any
  contact reacts at the repair processing that claim 6's spacing table had already scheduled, and
  that a settling depot's every-tick checks start at it.
- **S-0109 claim 1, cross-reference.** S-0109 claim 1 says that a new animation starts with
  *k* = 0. A start that replaces a running slot animation of this spec instead keeps the old
  one's stage (claim 25). S-0109 claim 28 already says that a building replacing a slot
  animation creates another animation under this spec's claims 12 and 13. Add to S-0109 claim 1
  a reference to this spec's claim 25 for the kept-stage exception.
- **S-0109 claims 6 and 8, cross-reference.** S-0109 claim 6 starts the first pass at *k* = 0, and
  claim 8 starts a reversed animation at *k* = λ<sub>e</sub> − 1. Both are starting stages that
  claim 25 here replaces for a start that replaces a running slot animation: the new animation
  starts at the kept stage instead, and then plays by claims 6 to 10 from there (claim 25 states
  the result, including a kept stage beyond the new type's range). Add to each of S-0109 claims 6
  and 8 the same reference to this spec's claim 25 as for claim 1. No other rule of those claims
  changes: the end limits, the pass count and the step direction stay as S-0109 states them.
- **S-0504 claim 8, backing.** Its sentence "Animation-slot rules inherit S-0108 claim 20's
  backings, including B18's naval-yard control and the unmeasured limitation for the every-tick
  row outside repair" should read "... including B18's naval-yard control; the every-tick row
  outside repair is `measurable` by B18 part i", since B18 part i now tests that row.

- **S-0405 claim 9, working animations (Q-1772).** Where claim 9 says that a building out of
  operation "changes its working animations only as S-0108 claims 13 and 14 give for a loss of
  power", it should name S-0108 claims 13, 14, 26 and 28. Claim 26 states the slots of a building
  that needs an engineer, before and at its ownership changes, and claim 28 states which slot
  animations a loss of operation leaves alone. Claim 9's light paragraph and scenario W6 part (i)
  need no change.
- **S-0405 claim 10 and its Open question on a turret or silo-fill slot.** Claim 10 says that
  whether a turret slot or a silo-fill slot, frozen by a switch-off, shows a held frame or the
  frame S-0108 claims 19 and 16 give "is an Open question", and its Open questions list that
  point. Claims 16, 19 and 26 here settle it: the per-tick setting from the turret facing or the
  fill level wins over any hold of the slot's frame. Claim 10 should say so with a reference to
  those claims, and the Open question should be removed. The silo case stays `unmeasured` here
  (claim 16) and the turret case is `measurable` by scenario B22 part (i), both at medium.
- **S-0402 Terms and claim 18, a grand opening in a trigger action's tick (noticed for
  Q-1772).** S-0402's term reads "at construction completion in play or during placement at game
  set-up". Claim 11 here adds a third case: a building that trigger action code 125 (create a
  building) creates in play has its grand opening at once, as part of that action, in the tick
  the action runs. The term should name that case. A replacement that trigger action code 130
  restores is not this case: it opens at the completion of its construction (S-0524 claim 15),
  which the term's first case already covers. Claim 18 should say that a code-125 building, if
  its type has a free unit, gets it in that tick, with the exclusions of claim 18 applying as for any building that was not
  produced (it has no start price), and that the construction behaviour it then runs gives no
  second unit. Its scenario H13 could add a part in which trigger action code 125 creates the
  free-unit building for a computer player's house, logging the tick of the action and of the unit's
  appearance. S-0215's term needs no change: it names no case.
- **S-0515 scenario TB3 part (c), setup (noticed for Q-1772).** Part (c) has the test map give
  the derrick to H from the start. If H is the human player's house, S-0009 claim 27 drops that
  map-listed structure in skirmish, so the part has no derrick to read. Give it to a computer
  player's house, or create it by a trigger action as part (e) does.

## Open questions

- **Code 125 with no owner house.** Whether trigger action code 125 creates a building when the
  trigger has no owner house, for example when its owner country is played by nobody (S-0011
  claim 5), is not established. Confidence: low.

- **A code-125 building's construction behaviour.** Claim 11 has a building that trigger action
  code 125 creates in play open at once, and its placement queues the construction
  behaviour as any placement in play does. What the building shows while that behaviour lasts
  (a build-up or its normal body), how long it lasts, and whether it ends at all when no build-up
  sprite has been loaded for its type, are not established. While it lasts the building is not in
  operation (S-0405 claim 8), so it cannot fire. Confidence: low. B22 part (a)'s per-tick record
  of T3's body would show the first two.
- Whether a replacement that trigger action code 130 restores (S-0524 claim 15) is disabled at its
  placement when its type needs an engineer, as the map's building list disables one (claim 26),
  is not established.
- Which behaviours put a building into the *active*, *full*, *aux-1* and *aux-2* states, and when
  the *pre-production*, *production*, *special* and *upgrade* slots start, is for the production
  and upgrade specs. Only the image-frame mechanics are fixed here. A repair depot's slot starts
  are in claim 20.
- S-0405 claim 5 identifies the drain blackout of claim 14. A separate special-outage slot
  measurement during that blackout remains wanted.
- Body-frame selection for types marked as laser-fence posts or firestorm-wall sections is
  outside this spec's coverage.
- Claim 5 computes the build-up delay with the build-up time held when the sprite is loaded. Within
  one rules layer, whether the game-wide build-up time is read before or after the building types is not
  known, so whether a build-up time that only a game-mode file or a map sets reaches a building's
  delay in the same rules layer is open. A building with the new-theater flag gets the final value at the
  theater refresh either way.
- Turret-slot art selection for wall-tower-style upgrades remains outside this spec's coverage.
- (Q-1610 question 3, the part left open; S-0314's Open questions hold the entry.) A vehicle that takes the freed contact place before the depot's pending processing
  after a loss in waiting, working or settling (S-0314 claim 22: the loss neither brings that
  processing forward nor puts it back; S-0314 claim 23 lets a queued vehicle take the place at
  its first eligible retry after the first leaves). After a loss in working or settling: whether that
  processing's step schedule restarts for the new vehicle or carries over from the lost one.
  After a loss in waiting: whether the pending waiting processing checks the new vehicle's
  stillness and starting distance as for any vehicle, or finds no contact and returns the depot
  to *guard*. Likewise for a vehicle that arrives at a depot still in its no-contact settling
  hold: whether the settling checks apply to it directly, skipping waiting. The slot changes of
  claim 20 for such a vehicle follow from those answers and are open with them.
- (Q-1610 questions 1 and 2, S-0314's Open questions hold the entry.) For a vehicle that leaves,
  both questions are open in working and settling until S-0314 scenario H4 part h runs 8
  (working) and 9 (settling) run (S-0314 claim 22). For a depot
  type that is also a vehicle bunker, when its processings come after a vehicle's contact ends, and
  so when claim 20's slot changes for that loss happen, is open: S-0606 claim 20 gives only that its
  processing delay is the depot's.
- B1 tests the delay from the imported value. Single-first input conversion is S-0003's, not
  a fresh binary64 conversion. No specific test fixture combining a sprite count and imported value
  that distinguish F1's arithmetic rounding modes is established here; that discrimination is unmeasured.
- The charged-animation time uses S-0003's decimal conversion and percentage rule, with final
  single-precision narrowing toward zero (S-0205). A threshold-sensitive B9 run remains wanted.
  No completed measurement is claimed.
- No skirmish event entering the *full* body state is established. Its deterministic rendering
  if entered is nevertheless defined by claim 6; identifying such an event does not require
  choosing a range later.
- No skirmish path filling any building's own storage is known (S-0408 claim 10). Positive,
  zero-capacity and negative-capacity nonempty cases of claims 15 and 16 all remain unmeasured.
- Whether the door image of claim 23 also gets a shadow frame, as the body does under claim 9, is
  not known.
- F9 takes the held infantry in turn. With a negative infantry size the result can depend on that
  order, which is not stated (claim 17).
- Whether a power order given for a building of the player's own house is still applied when an
  ownership change earlier in the tick of its application (a polled trigger's action, a capture)
  has given the building to another house is not known. Claim 13 states the slots only for a
  change that gives the building to the order's house; B22 part (n) gives every order for a
  building that is the player's at its tick. Rated low, `unmeasured`: no own-building order can
  be timed after such a change in this spec's scenarios without the original's acceptance rule.

## Questions for the project lead

Decided (delegated sign-off, 2026-10-25):

- Claim 6's deterministic *full* range. Unlike the original's unspecified range, its result is
  completely defined here; it adds no imported key.
