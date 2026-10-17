---
id: S-0108
title: Building sprites
topic: rendering
revision: 14
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0540]
supersedes: []
---

# S-0108: Building sprites

## Summary

How a building's sprite is put together: which sprite and image frame the body shows (build-up, normal,
damaged, occupied, gate stages, selling, unloading), where the body is anchored relative to its
foundation, the shadow, the bib and the door image, and the slot animations attached to a building. For the slot
animations it covers the slots, the damaged and occupied variants, the power behaviour, and the
building-specific rules that start and swap them (refinery and silo fill, infantry absorption,
superweapon charge, turret, repair depot). This spec covers skirmish only; network games and campaigns
are out of scope.

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
  already holds an animation, it is replaced, and the new one continues from the old one's image frame.
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
  **Damaged-door type:** a type whose art is marked as having damaged door frames.
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
   - Confidence: high (the sprite choice, invisible types), medium (the missing-sprite case; whether
     such a building ever enters the *construction* state belongs to the production spec)

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
   - Confidence: high

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
   timing; no build-up sprite uses S-0408 claim 4 instead of a build-down. The construction delay is never
   game-speed normalised when the state is entered (but see claim 6 for the wrap).

   The construction range is set whenever the build-up sprite is loaded, with *B* as it is held at
   that moment: at every type read that loads the sprite (S-0007 claim 11), and again at the theater
   refresh for a building with the new-theater flag, which reloads all its sprites as at a type read
   (S-0007 claim 12). No data marks a building to load its build-up sprite later than that.

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
   - Confidence: high (the delay), medium (the exact tick of the switch to the body sprite, and the
     theater-flag rule, which stock data does not use)

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
   - Confidence: high (the two entry rules and the wrap rule), medium (the turret case and the
     last-request-wins rule)

7. **Bib.** When the type has a bib sprite, the bib is drawn whenever the building is not in the
   *construction* state, so never during build-up or selling. It is also not drawn with the
   special gate image (claim 4), including at rest open while *open* is current. It uses the same image-frame index *f* as the body (so image frame 1 when
   damaged, with default art), without the clamp of claim 3. Its depth bias is in S-0102.
   - Backing: `measurable` (B4 for ordinary bib visibility and default damaged frame).
     The unclamped index is `unmeasured`: B4 never requests an index above the body clamp.
     Special-gate-image suppression shares claim 4's limitation.
   - Confidence: high

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
   - Confidence: high (the shadow frame), medium (no shadow past the sprite)

10. **Slot animation placement.** A slot animation is created at the image anchor of claim 8 (the
    foundation's top corner), moved by the slot's screen displacement (*x*, *y*) in pixels, which
    is converted to world units by the projection spec. Its depth adjustment and y-sort come from
    the slot definition (S-0102). While the building is being sold during its *construction* state,
    every slot animation is hidden.
    - Backing: `measurable` (scenario B14)
    - Confidence: medium

11. **Grand opening.** A building's grand opening brings it into service once: on construction
    completion in play, or during placement at game set-up. These slots start:
    - For a building that is not a refinery: *idle* (occupied variants allowed), *active-1*
      (occupied variants allowed), and *active-2*, *active-3* and *active-4* (no occupied variant).
    - For a refinery: only the *active* slot of its fill level (claim 15), with no occupied
      variant. Grand opening does not itself start *idle* for a refinery, nor stop slots
      already running.
    - For a building with a superweapon: *super-3* when the weapon's remaining charge time is
      below 15 ticks, otherwise *super-1*. Both allow occupied variants.
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
    - Backing: `measurable` (B6 for unoccupied completion starts, set-up slot presence and the
      turret case, S-0215 claim 1 for queued *guard* and readiness,
      S-0402 scenario H13 for the free unit).
      `unmeasured` for occupied grand-opening variants (normally no occupants at completion),
      below-15-tick charge at completion (B6 establishes no such setup), and repeated set-up
      body-state changes beyond the placement's idle entry (no input path selecting each
      additional set-up state is established). Unknown-name starts and empty
      upgrade names are also `unmeasured`: no start path for those slots is established here.
    - Confidence: high

12. **Damage swaps the variants.** Whenever the building's health changes through damage or repair
    and its damaged state then differs from its slot condition, the slot condition takes the new
    value and every running slot is restarted, keeping its image frame, with the damaged name when now
    damaged and the normal name when not. A slot whose name for the new condition is empty keeps
    its current animation. The occupied variant is never chosen by this swap
    (`quirks.damage_swap_drops_occupied`). The swap takes effect on the tick of the damage or
    repair. Starting a slot in the condition opposite to the slot condition triggers the same swap
    of every running slot, on the tick of the start, and the started slot shows the variant chosen
    for that start. After a condition swap, existing slots show normal or damaged art, except a slot with an
    empty name for that condition, which keeps its old art until something restarts it. A slot
    started with an occupied variant still shows that variant until a later restart or swap:
    counting it as the normal slot condition does not replace its occupied art.
    - Backing: `measurable` (B2 for damage/repair swaps and occupied art being dropped,
      B6 for the turret start's opposite-condition swap). Empty-condition upgrade slots are
      `unmeasured`: B2's ordinary empty damaged name is filled with the normal name at import,
      so it cannot test retaining an old variant on an empty-name swap.
    - Confidence: high

13. **Power.** For each slot only the first of its power flags in the order *freeze-unpowered*,
    *light-unpowered*, *effect-unpowered* counts. *freeze-unpowered* is on for every slot unless the
    art turns it off (import mapping), so *light-unpowered* or *effect-unpowered* takes effect only in
    a slot whose art sets *freeze-unpowered* off. The slots that have no power-flag keys
    (*pre-production*, *production*, *turret* and the three upgrade slots) always count
    *freeze-unpowered*. A building reacts to a change of its powered state
    in its next update, so on the same tick as the change or the next one. Both columns apply only
    to a building that needs and draws power.

    | Counting flag | Power is lost | Power comes back |
    |---|---|---|
    | *freeze-unpowered* | A running slot holds its image frame and stops advancing. | It resumes. |
    | *light-unpowered* | A running slot is stopped. If it is *special-1* of a delayed-fire type whose *active-1* has *freeze-unpowered*, *active-1* starts (and, being unpowered, starts frozen). | If the slot is not running and the building has completed, it starts. The *special-1* slot of a delayed-fire type is the exception. |
    | *effect-unpowered* | A running slot is stopped and marked as stopped by the outage. If it is *super-3*, *super-low-power* starts. | Marked slots start and are unmarked. |
    | any | | *super-low-power* stops. |

    Slots started by these rules use the normal or damaged variant, never the occupied one.

    A *freeze-unpowered* slot that starts while the building is disabled or unpowered starts frozen.
    This applies only when the type itself needs power. Separately, disabling a building (for any
    reason the disabling spec defines) freezes every running *freeze-unpowered* slot, whether or not
    the type needs power. Re-enabling it unfreezes them, even when the building is still unpowered
    (`quirks.enable_unfreezes_unpowered`).
    - Backing: `measurable` (B7 for freeze/light/effect flags and super-low-power transitions,
      B11 for disable/re-enable while unpowered). `unmeasured` for delayed-fire outage exceptions,
      occupied-variant exclusion and disabling a type not needing power: neither scenario
      exercises those cases or establishes a discriminating event path.
    - Confidence: high (the power-loss and power-return rules), medium (the disable and enable
      cases, whose triggers belong to other specs, and the same-or-next-tick timing)

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
    - Confidence: high (*S* > 0), medium (*S* < 0)

16. **Silo fill.** For a type marked to show silo fill, the level is
    *L* = trunc(4·*m* / *S* + 0.5), clamped to 0 … 3 (Formulas F5). It is 0 when *S* ≤ 0. At level 0
    the *special-1* slot is stopped. Otherwise *special-1* is started if absent (normal or damaged
    variant), and its displayed image frame is forced to *L* on every tick. The animation's own timing
    does not move it.
    - Backing: `measurable` (B8 for empty storage stopping the slot).
      `unmeasured` for positive fill, its rounding and forced image frames: no known skirmish
      path fills building storage. The missing-animation failure under Edge cases shares this
      limitation; our safe result and warning are defined departures.
    - Confidence: high

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
    - Confidence: high

18. **Superweapon charge cycle.** For a building with a superweapon (its first superweapon only),
    this is checked every tick while it is not being built or sold. The check runs only when the
    type's charged-animation time *t_c* (game minutes, a single-precision floating-point field) is
    at most 990. Let *t_r* be the weapon's remaining charge time in game minutes, computed as remaining
    ticks × *κ*, where *κ* is the single-precision constant nearest 0.0011111111 (exactly
    2386093 / 2³¹) (Formulas F6).
    - When *t_c* < *t_r* and *super-3* is running: *super-3* stops and *super-4* starts.
    - When *t_c* ≥ *t_r* and *super-1* is running: *super-1* stops and *super-2* starts.
    - When *super-2* finishes on its own, *super-3* starts. When *super-4* finishes on its own,
      *super-1* starts.

    All these starts use the normal or damaged variant. The four slots therefore form a loop:
    charging, opening, charged, closing.
    - Backing: `measurable` (B9 for the four-slot cycle and its threshold).
      `unmeasured` for the 990-minute cutoff and ignoring a second superweapon: B9 supplies
      neither a cutoff comparison nor a two-superweapon building.
    - Confidence: high

19. **Turret slot animation.** For a building with a turret, the *turret* slot animation does not
    advance on its own. Every tick its image frame is set to (28 − *j*) mod 32, a value from 0 to 31,
    where *j* is the turret facing rounded to 32 directions as in S-0107 Formulas F2
    (*j* = (⌊φ / 1024⌋ + 1) div 2 mod 32, with φ the turret facing in facing units). The turret sprite therefore stores 32 directions
    counter-clockwise, with image frame 0 at turret facing 28 × 2048. When the turret art is voxel art,
    the *turret* slot is not started at grand opening. When the barrel art is voxel art, a *turret*
    slot animation that does start is hidden.
    - Backing: `measurable` (B17 for the sprite-turret formula). The voxel turret/barrel suppression
      is `unmeasured` here: B17 has neither voxel-art flag variant.
    - Confidence: medium (the turret facing is defined in S-0206, Terms)

20. **Repair-depot, recycling and delayed-fire slot rules.** These apply only to the building kinds named in
    each row.

    | Building | Event | Result |
    |---|---|---|
    | Repair depot, not naval, not in its repair behaviour, with *production* or *special-2* running | checked every tick | *special-3* starts (normal or damaged variant), *production* and *special-2* stop |
    | Repair depot repairing a docked unit | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *special-2* starts (normal or damaged variant) |
    | Repair depot not repairing a docked unit | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *idle* starts (normal or damaged variant) |
    | Repair depot | *special-3* finishes on its own | *idle* starts (normal or damaged variant) |
    | Delayed-fire type that is not a repair depot | *special-1* finishes on its own | *active-1* starts (occupied variants allowed) |
    | Recycling building | *special-1* ends (on its own, or removed by anything but the building stopping or replacing it) | *active-1* starts (damaged variant when the health ratio is at or below the yellow ratio, F3; normal otherwise), provided the selected variant exists (S-0603 claim 12) |

    No building other than the kinds listed reacts to *special-1* or *special-3* ending.
    - Backing: `measurable` (B18 for natural completion during repair, leaving repair and
      delayed fire; S-0603 scenario G10 for recycling-building completion and external removal).
      `unmeasured` for repair-depot non-natural removal of *special-1* and its completion while
      not repairing: B18 establishes neither event. The exhaustive other-types assertion is
      `unmeasured`: a finite set of types cannot establish it.
    - Confidence: medium (the triggers of the repair behaviour belong to the repair spec)

21. **Parsed but without effect.** The type's extra-damage-stage flag has no effect on how a
    building is drawn or behaves.
    - Backing: `measurable` (B11 for recorded rendering differences of the toggled type).
      No effect on any behaviour is `unmeasured`: a finite rendering test cannot establish
      absence of every possible gameplay effect.
    - Confidence: high

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
    - Backing: `measurable` (B10 for sprite selection and under-door damage frames).
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
    - Backing: `measurable` (B10 for presence throughout unloading and absence outside it).
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
κ = 2386093 / 2³¹ ≈ 0.00111111114, the single-precision value nearest 0.0011111111, which
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
| Grand-opening "charged" test | 15 | ticks | Remaining charge below this starts *super-3* |
| Charged-animation time limit | 990 | game minutes | Above it, claim 18 is disabled |
| Charge time scale *κ* | 2386093 / 2³¹ | game minutes per tick | Single-precision constant, F6 |
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

**Superweapon slots** (claims 13 and 18):

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
  its *freeze-unpowered* slot animations even when it has no power (claim 13). Off: they stay frozen
  until power comes back.
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
| `UnitRepair` | building | boolean | false | repair depot (claim 20) |
| `Naval` | building | boolean | false | claim 20 |
| `Powered` | building | boolean | false | needs power (claim 13) |
| `Power` | building | integer | 0 | a negative value means the building draws power (claim 13) |
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

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*: an unmodified install with no loose override
files and no engine add-ons, stock data, a tiny purpose-made map, one player with ample credits and
one bystander computer player kept outside the observed area unless a scenario needs its named
objects. The observed area is the test buildings, their nearby cells, and any route or units the
scenario uses. Apply the glossary's bystander void rule. Use standard skirmish, 800 × 600 pixels,
game speed 4 unless stated, high detail, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), and no unrelated trigger effects. Record per tick, from a screen capture that keeps every drawn frame or a
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
- **B2 (claims 2, 12).**
  - Setup: one building with slot animations that have damaged names, one occupiable building with
    occupied-variant slot animations, a weapon of known damage, and repair available.
  - Procedure: damage the first building in steps that approach the yellow ratio from above, then
    hit it exactly at the ratio. Repair it past the ratio. Garrison the occupiable building, damage
    it past the yellow ratio, then repair it. Repeat the first part with test data: a copy of the
    first type whose art leaves the damaged name of one slot empty.
  - Observable: the tick of each body-frame change and slot-variant change.
  - Runs: one per building.
  - Expected: the body frame changes to the damaged frame at *h* = yellow exactly, not one step
    later, and the slot animations switch to their damaged art on the same tick, keeping their image
    frame.
    Repair restores the normal art on the tick the threshold is crossed. The garrisoned building
    shows normal art, not occupied art, after the repair. In the copy, the imported damaged name
    has been filled with the normal name, so that slot shows normal art when damaged. This is not
    a test of an empty-name swap retaining its old animation.
  - Falsified if: a switch comes one damage step early or late, a slot animation restarts from image
    frame 0, the repaired garrisoned building shows occupied art, or the slot without a damaged name
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
  - Falsified if: a frozen slot animation keeps advancing, *super-low-power* does not appear, a
    reaction comes more than one tick after the power change, the *production* slot animation keeps
    advancing, or the slot with only the light flag set vanishes.
- **B8 (claims 15, 16, empty storage only).**
  - Setup: a refinery and a silo-fill type with named *special-1* art, with positive capacities.
    A read-only recorder verifies that each building's own stored amount is 0.
  - Procedure: complete each building and observe for 30 ticks. Harvesting may continue, but is
    not a means of filling building storage; credits are paid directly (S-0402 claim 12).
  - Observable: stored amount, the refinery's active slots and the silo's *special-1* slot.
  - Runs: one per building.
  - Expected: while its recorded amount stays 0, the refinery has *active-1* only among its
    fill-selected active slots, and the silo's *special-1* is absent.
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
  - Procedure: build a vehicle at the first, and a jumpjet or balloon-hover unit at the second. If
    a factory exists whose art has an ordinary deploying sprite but no roof-deploying sprite, build a
    jumpjet or balloon-hover unit there too. At the door factory and its copy, build a unit
    undamaged, then damage the factory past the yellow ratio and build another.
  - Observable: the body sprite and under-door image during each unload, and the door image's
    presence and door frame per tick, with the door state and the remaining ticks of its movement.
  - Runs: one per factory and unit.
  - Expected: the deploying sprite replaces the body during the vehicle unload, and the roof variant
    appears only for the jumpjet or balloon-hover unit. At the third factory the body sprite stays.
    The under-door image shows image frame 1 when the factory is damaged. The door image is present on
    every tick of the unload, including the ticks before the door starts to open and after it has
    closed, and absent outside the unload. Its door frame rises from 0 to *D* − 1 while the door
    opens, stays at *D* − 1 while it is open and falls back while it closes, each door frame
    following F8. Damaged, it is the same plus *D* for the damaged-door type, and unchanged otherwise.
  - Falsified if: the body sprite stays during a vehicle unload, the ordinary deploying sprite
    appears for a jumpjet or balloon-hover unit, the door image is missing on a tick of the unload
    or shown outside it, or its door frames differ from F8 in any recorded door state, including the damaged
    copy's addition of *D* and the unmarked copy's unchanged frames.
- **B11 (claims 6, 13, 21).**
  - Setup: game speeds 0 and 4. Two named test-art copies of a stock non-turret building: each
    has an idle range of three frames with delay 3 ticks, invented test values, and a sprite with
    enough distinct body frames and matching shadows. One art copy is normalised, the other not.
    Use a type that needs and draws power, with advancing freeze-unpowered slot art. A third copy
    toggles only the extra-damage-stage flag relative to the first.
  - Procedure: build each and log body stages from its deferred construction-to-idle change
    through at least three idle wraps.
    While its house remains short of power, switch that building off and then on through the
    normal power-toggle order (S-0405 claim 10). Keep the house shortfall unchanged throughout. The toggled-flag run is required, since it is the only
    backing of claim 21: run the copy through the same procedure as its original type, undamaged
    and then damaged past the yellow ratio, at both speeds.
  - Observable: idle body-frame period in ticks, whether the frozen slot animations advance, and,
    for the toggled copy and its original, every quantity of the common setup's per-tick record.
    The recorder also distinguishes initial state entry from the first idle wrap; the wrap's
    following delay is the period under test.
  - Runs: one per building and speed, and one per speed for the toggled copy.
  - Expected: the deferred idle-entry delay and each delay after an idle wrap follow N(*d*, *s*)
    in both marked and unmarked cases. Use the imported stage count to separate an image-cycle
    period from one stage delay. The frozen slot animations resume
    when the building is re-enabled although the power is still out. The toggled copy's record
    matches its original's on every tick.
  - Falsified if: either entry or post-wrap stage delay differs from N(*d*, *s*), the slot
    animations stay frozen until power comes back, or the toggled copy's record differs.
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
- **B14 (claim 10).**
  - Setup: test data. A test building with one slot animation whose displacement is (0, 0), and a
    copy with displacement (10, −6).
  - Procedure: place both, let them complete, screenshot them, then sell one.
  - Observable: the slot animation's screen position relative to the image anchor, and its
    visibility during the sale.
  - Runs: one.
  - Expected: the first is centred on the anchor, the second is 10 pixels right and 6 pixels up of
    it. All slot animations of the building being sold are hidden for the whole sale.
  - Falsified if: the displacement differs from the given pixels, or a slot animation stays visible.
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
- **B18 (claim 20).**
  - Setup: a repair depot and a damaged vehicle. A delayed-fire type with *special-1* and *active-1*
    slot animations (test data if no stock type has the mark).
  - Procedure: drive the vehicle onto the depot and let it be repaired, then drive it off. Let the
    delayed-fire type fire.
  - Observable: the depot's and the delayed-fire building's slot animations, per tick.
  - Runs: one.
  - Expected: while repairing, *special-2* follows *special-1*. After the vehicle leaves,
    *special-3* replaces *production* or *special-2*, and *idle* starts when *special-3* finishes.
    The delayed-fire building restarts *active-1* when *special-1* finishes on its own.
  - Falsified if: *special-2* does not follow natural *special-1* completion during repair,
    *special-3* does not replace the named slots after repair ends, *idle* does not follow
    natural *special-3* completion, or the delayed-fire building fails to restart *active-1*.

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

Construction timing in claim 5 uses S-0406 scenario PL11 a to f, including its setup,
read-only per-tick observables and falsification criteria: placed buildings show two runs,
deployed buildings restart at their first construction processing, zero-delay ranges complete,
and positive-delay ranges of one stage or fewer never complete. Stage-23 sale completion uses
S-0408 scenario S10, with its setup, observables and falsification criteria.

The free-unit effect of claim 11 is measured by S-0402 scenario H13, parts a to f, with that
scenario's setup, observables and falsification criteria; no separate unit-placement rule is
specified here.

## Open questions

- Which behaviours put a building into the *active*, *full*, *aux-1* and *aux-2* states, and when
  the *pre-production*, *production*, *special* and *upgrade* slots start, is for the production,
  repair and upgrade specs. Only the image-frame mechanics are fixed here.
- S-0405 claim 5 identifies the drain blackout of claim 14. A separate special-outage slot
  measurement during that blackout remains wanted.
- Types marked as laser-fence posts or firestorm-wall sections take their body frame from a
  separate per-building value that this spec does not cover. Stock YR is not known to use them.
- Claim 5 computes the build-up delay with the build-up time held when the sprite is loaded. Within
  one rules layer, whether the game-wide build-up time is read before or after the building types is not
  traced, so whether a build-up time that only a game-mode file or a map sets reaches a building's
  delay in the same rules layer is open. A building with the new-theater flag gets the final value at the
  theater refresh either way.
- A wall-tower-style building, its type named by a rules key, takes upgrades that switch its turret
  slot animation among art named after the image name plus a letter. It is not known whether stock YR
  uses this.
- B1 tests the delay from the imported value. Single-first input conversion is S-0003's, not
  a fresh binary64 conversion. No specific stock sprite count and imported value that distinguish
  F1's arithmetic rounding modes are established here; that discrimination is unmeasured.
- The charged-animation time uses S-0003's decimal conversion and percentage rule, with final
  single-precision narrowing toward zero (S-0205). A threshold-sensitive B9 run remains wanted.
  No completed measurement is claimed.
- No skirmish event entering the *full* body state is established. Its deterministic rendering
  if entered is nevertheless defined by claim 6; identifying such an event does not require
  choosing a range later.
- No skirmish path filling any building's own storage is known (S-0408 claim 10). Positive,
  zero-capacity and negative-capacity nonempty cases of claims 15 and 16 all remain unmeasured.
- Whether the door image of claim 23 also gets a shadow frame, as the body does under claim 9, is
  not traced.
- F9 takes the held infantry in turn. With a negative infantry size the result can depend on that
  order, which is not stated (claim 17).

## Questions for the project lead

- Confirm the proposed deterministic *full* range in claim 6 at sign-off. Unlike the original's
  unspecified range, its result is completely defined here; it adds no imported key.
