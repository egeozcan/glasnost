---
id: S-0504
title: Naval units (ship locomotion, wakes, sinking, underwater vessels, docking at yards)
topic: movement
revision: 26
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-25
measurements: []
requests: [Q-0600, Q-0640, Q-0642, Q-0643, Q-0644, Q-0646, Q-1871]
supersedes: []
---

# S-0504: Naval units (ship locomotion, wakes, sinking, underwater vessels, docking at yards)

## Summary

How vessels move and what is particular to them once they are on the water: the ship locomotor
(which reuses the driving rules of S-0207 with the differences specified below), the wakes a moving vessel
leaves, what a sinking vessel does between its killing hit and leaving play, what the underwater
flag does and does not change, and the naval rules of docking at a building, including the
distance at which a repair yard starts on a vessel.

This spec does not repeat what other specs own. Entering water, the land-type and zone rules and
ship substitutions are S-0211 (claims 10 to 12) and S-0209 (claim 5f). The naval weapon choice is
S-0308 (claim 11 and table N). Release of a new vessel from a yard is S-0404 (claim 13) and S-0417
(claim 9). The sinking condition at a killing hit is S-0312 (claim 10); claim 11 here supplies
its absent vehicle-weight default. A vessel's crush handling
is S-0210 (claim 9). Cloaking, cloak timing and detection are planned S-0516; this spec only says which
naval states those rules produce (Terms, *submerged*). The squid's grappling is S-0614.

**Carrier and missile vessels.** S-0506 claims 1 to 9 and 11 to 13 own the shared spawn-controller
cycle for both carriers and missile-launching ships, including salvo release, replacement and
reload. Launcher ships' parameter-set-B missiles and their flight belong to S-0506 claims 14 to 21.
Non-missile carrier aircraft flight belongs to S-0501 claim 35, not to this spec.
Shared salvo release (S-0506 claim 8) is distinct from the end of the missile rail interval
(S-0506 claim 9) and the launching lockouts (S-0506 claim 10). Claim 3 here owns the ship's
between-track hold while its launching count is positive; that hold does not apply merely because
a carrier's controller is launching ordinary aircraft.

**Q-1871: the lean of a sinking vessel.** A sinking vessel does lean as it goes down. The lean is a
pitch, a fore-and-aft tilt about the hull's own lateral axis, not a sideways list (claim 15). On
each of its own updates while sinking, its pitch moves 0.01 radians (F4) towards bow-down or
stern-down and stops at π/4. From level, that takes 79 updates. The direction comes from the
8-direction of its body facing at its first update while sinking, is kept for the whole descent,
and takes no random draw. No type key sets the step, the limit or the direction. The rule keys on
the sinking state alone, whatever started it and whatever the starting height, where the body is
drawn by a locomotor that draws rocking (claim 16). Sinking adds no rule of its own for the roll:
a rocking hit still sets it going, as on a vessel that is not sinking. Both are drawn as rocking under S-0106 claim 9, and a sinking vessel draws no shadow
(claim 16). Three oddities of the original's drawing are not copied: its lean passes π/4, can
reverse when a turn crosses between heading groups, and keeps its roll frozen (claim 15; decided at
the delegated sign-off, 2026-10-25). None of the vessel's own rules other than drawing reads the
pitch. The lean does reach the shots of passengers that fire from an open-topped transport sinking
without a killing hit (claim 16).

## Terms

- **Ship locomotor.** The locomotor whose class identifier is the "ship" row of S-0207 table T3.
  Every stock vessel that is not a hovering craft has it. A *ship* in this spec is a vehicle with the
  ship locomotor.
- **Naval vehicle, underwater type.** As S-0303 (Terms): a vehicle whose type has the naval flag,
  and a type with the underwater flag.
- **Submerged.** An underwater-type vehicle whose cloak state is cloaking, cloaked or uncloaking
  (the same states S-0308 table N tests). Submerging and surfacing are therefore the cloak
  transitions of planned S-0516; this spec adds no state of its own. An underwater type without the
  ability to cloak is never submerged (claim 6).
- **Wake.** An instance of the game-wide wake animation (Import mapping) created by a vessel on the
  water (claims 2 and 4). It uses ordinary animation rules (S-0109); these claims add no
  wake-specific effect on other objects.
- **Sinking.** The state in which a vessel goes down in the water while it stays in play.
  S-0312 claim 10 gives the condition at a killing hit; S-0614 claim 11 also starts sinking
  through a cull without a killing hit. Claims 4 and 5 here give what happens while it lasts.
- **Sinking descent.** The lowering of a sinking vessel by a fixed amount each tick until it leaves
  play (claim 4).
- **Pitch and roll.** The two rocking angles of a vehicle's body (S-0106 Terms, *Rocking*), in
  radians, held in single precision.
  - The *pitch* is S-0210's *forward rocking angle*. It is drawn as a rotation about the body's own
    lateral axis, the model frame's *y* axis (S-0105 Terms), with S-0105's rotation convention for
    *Ry*, so a positive pitch lowers the bow and raises the stern.
  - The *roll* is S-0210's *sideways rocking angle*, drawn about the body's own forward (*x*) axis.
  - Each angle also has a rate per tick, in radians per tick. S-0210's body tilt is the pitch rate,
    with the same sign: a negative body tilt raises the bow and a positive one lowers it (see
    Changes for other specs). How the rates are set,
    and how they move the angles outside sinking, belongs to the specs that set them (S-0210, S-0303
    claim 20, S-0614). Claim 15 gives the sinking rule.
- **Docking answer.** As S-0214 (Terms): the answer a building gives a unit that asks whether it
  may enter or dock.
- **Docking point.** The world position a building supplies for a contacted unit's docking distance
  test. It need not lie in an external neighbouring cell. Claim 12 applies S-0314 claim 18's
  selection and stopping rules to naval yards.
- **Stored path, path retry timer.** As S-0212. Claim 9 specifies their ship-locomotor application.
- **Locomotor destination point.** S-0207 claim 25's independently retained movement goal,
  distinct from the order-level destination and the active track's heading point.

## Behaviour

### Ship locomotion

1. **A ship moves by the driving rules.** A ship moves, turns and stops exactly as a drive vehicle
   of S-0207 with the same type fields on ordinary travel tracks, except for claims 2 and 3 and
   the ship crush handling of S-0210 claim 9. This does not assert equivalence for special exit
   tracks. These rules cover skirmish; network games and campaigns are out of scope. In particular these S-0207 rules
   hold for it unchanged: the speed and progress per tick (claims 1 to 4), height and facing on a
   track (claim 5), turning in place before a step (claim 6, with S-0206 claim 12), the choice of
   track from tables T1 and T2 (claim 7), no chaining of curves for ordinary vehicles (claim 8),
   the target fraction and the speed-fraction update with the same cases and constants (claims 9
   and 10), stopping (claim 11), ticks and track boundaries (claim 12), the speed power-up (claim 13),
   the outcomes of the step check (claim 15, with the crush mode of S-0210 claim 9 in place of the
   driving one), scatter requests (claim 16), the lead speed (claim 18), the approach to a target
   (claim 19) and cell-tag arrival timing (claim 22). A ship's *heading point* (S-0207 Terms) is the
   driving one: the track target of the track it is on, from the tick the track begins until the
   tick it ends, and none while it stands still, turns in place, waits or is between tracks.
   Its stored world height follows S-0207 claim 24: it copies the ship's current integer
   world height in leptons when the track begins, before progress, with no rounding,
   target-surface sampling or extra deck lift. This holds for one-cell tracks and two-cell
   curves. The height stays unchanged during the track and through curve chaining, even
   when the ship's own height changes; chaining advances only the horizontal target.
   A new track after completion copies the ship's height anew. A high-bridge target alone
   therefore does not assign deck height. S-0511 claims 9 and 11 own the bridge-loss uses
   of this stored height and the separate current-height track test.

   The separate locomotor destination point follows S-0207 claim 25 unchanged. It is absent
   at creation. An accepted movement instruction installs its supplied integer-lepton point
   immediately, before any successful path search, even for a point in the ship's own cell.
   A high-bridge target adds 416 leptons to the supplied height once, at acceptance; this
   does not change the heading-height rule above. Refused or ignored orders do not necessarily
   supply an instruction. A retained-goal wait, retry, turn or intermediate track completion
   keeps the point, independently of heading-point presence. Crossing into the goal cell
   does not clear it. A stop that stops locomotion clears it immediately even while the
   current track finishes; replacement supplies the new point, and an accepted instruction
   without a point clears it. At track completion with an order destination still present,
   being in that destination's supplied movement-point cell with an absolute difference
   between ship world height and locomotor destination height strictly below 208 leptons
   clears the point before the cell-arrival report. This completion alone does not remove
   the order-level destination. Copies and comparisons use integer leptons without rounding.

   Order-level goal cleanup follows S-0207 claim 26. At a locomotion update without an
   active track, occupying the cell named by a cell destination removes that destination
   and stops locomotion, clearing the locomotor point immediately when there are no
   approach destinations. With approach destinations, the old goal is abandoned and
   S-0215 claim 13's fallback decides any replacement. This cell test needs no exact
   position, height tolerance, successful search or due behaviour processing, and does
   not require the locomotor point to be present or the current behaviour to be *move*.
   After qualifying final completion in tick *t*, an unchanged ordinary cell goal therefore
   remains through that locomotion update and is removed at the next tick's no-track
   update, unless an intervening effect changes it. Arrival-report effects may separately
   change the goal in tick *t*. An accepted same-cell move from rest is cleaned up at its
   first subsequent no-track locomotion update, possibly in the assignment tick itself,
   not after an extra one-tick delay.

   A destination naming a building, vehicle, infantry unit or aircraft is not a cell
   destination merely because its supplied movement point lies in that cell. Mere cell
   equality without a track removes neither destination for these object goals, whether
   the named unit is moving or stationary. Same-cell entry can retain both until a stop,
   replacement or applicable behaviour effect. There is a separate exception at rest in
   *guard*: with a locomotor destination point present, exact equality of the ship's world
   point and that point in all three integer-lepton coordinates triggers the same removal
   or approach-destination fallback, including for all these object goals. Same-cell
   equality alone does not satisfy that exception. Rest and guard describe the ship,
   not the named unit. The comparison uses the retained locomotor point, not the named
   unit's current position; that unit's movement alone does not replace the point, but
   an accepted replacement instruction can. Entry and servicing effects remain S-0215
   claim 29's and S-0314 claims 17 and 18's; transport exchanges and arrival-time boarding
   remain S-0507 claims 5 and 6's. Retention is not guaranteed through those effects.
   Give-up and fallback otherwise use S-0207 claim 25's normal stop/replacement rules,
   not an inferred successful arrival. S-0212 claims 20 to 22 still own search timing
   and give-up conditions.
   A ship stays in the ground layer, or the bridge-deck layer on a deck, by S-0207's rules; an
   underwater type is in the same layer as a surface ship (claim 6). Its body tilts on slopes as
   S-0106 claim 5 says for naval locomotion.
   - Backing: `measurable` (N1 and N2) for ordinary progress, turning, stopping, heading-point
     scatter and approach; S-0207 D27 for heading-height assignment and retention through
     chaining; S-0207 D29's water controls for the separate locomotor destination's
     assignment and lifetime; S-0207 D30's ship controls for cell-goal cleanup, next-update
     timing, retained building goals and the exact-point guard exception; S-0207 D31's
     ship controls for vehicle, infantry and aircraft goals, named-unit motion independence
     and the guard comparison against the retained point. The remaining
     shared-rule assertions are `unmeasured` here: those
     scenarios do not isolate power-ups, lead speed, cell-tag timing, slope tilt or every entry
     outcome; the cited specs' scenarios need ship-specific counterparts.
   - Confidence: high for shared ordinary rules and tables, object-goal classification and
     named-unit motion independence; medium for lead speed, approach
     and cell-tag timing, composed cleanup timing and approach-destination fallback lifetime

2. **Wakes behind a moving ship.** In each tick in which a ship is *moving now*, and the tick number
   (glossary) is divisible by 8, the ship creates one wake at its own position, provided all of these
   hold: its type is not an underwater type, it is not on a bridge deck, the land type of its cell is
   *water* (S-0211), and the game-wide wake animation is defined. A ship is *moving now* when its
   body is turning, or when it has a heading point and its current integer progress rate *s* is
   above 0. Here *s* is the rate of S-0207 claim 2, evaluated with the ship's current speed fraction
   and factors at the wake check: *s* = trunc(*s*₂ × *ρ*), in progress units per tick, with the
   floating-point arithmetic and intermediate truncations of that claim. It is not the fraction
   *ρ* itself, accumulated progress, distance actually travelled, or a remembered rate from the
   last moving tick. With no body turn, a positive *ρ* that still gives *s* = 0 produces no wake.
   Positive *s* can produce a wake without a position change when the progress budget has not yet
   reached the next track point. The heading point is the active track's target (claim 1), not a
   path destination retained between tracks.
   The wake check uses the position, heading point, turning state and speed fraction after that
   tick's locomotor movement and track transitions. Thus the wake is at the post-movement
   position; completing a track can remove its heading-point eligibility even if it travelled
   earlier in that tick. Creation occurs during the ship's own turn (S-0203 claim 4), after the
   draws of that locomotor movement, not in a separate end-of-tick wake pass. A wake takes no
   draw of its own beyond those its animation type takes when it is created (S-0109 claim 3).
   - Backing: `measurable` (N3, including its named no-animation variant; N3a for the rate,
     track-boundary eligibility, position and draw ordering)
   - Confidence: high

3. **Launching hold.** A ship whose launching count is positive (S-0506 claim 10) does not
   begin its next step: between tracks it keeps waiting, with its destination kept, until that
   count is zero, and then goes on by claim 1. A ship already on a track finishes that track first.
   The driving locomotor holds in the same way. This is the launching-count lockout, not a hold
   throughout the controller's launching state or until shared salvo release (S-0506 claims 5
   and 8). A missile slot can still count while on the rail after release (S-0506 claims 8 to 10).
   Ordinary non-missile carrier aircraft in preparing, attacking or returning slots do not count
   and do not by themselves hold their carrier still. Order refusal while partially launching is
   separately S-0506 claim 10; this claim describes movement for an accepted destination.
   - Backing: `measurable` (scenario N4) for ships. Driving-locomotor equivalence is
     `unmeasured` here: N4 does not provide a driving spawner or isolate its track boundary.
   - Confidence: high for the ordinary hold; interaction with concurrent waits remains open

### Sinking

4. **Sinking descent and leaving play.** Descent applies only to a sinking vessel retained in
   play after S-0312 claim 15. A crashable surface vessel removed during its first killing hit
   under S-0312 claims 10 and 15 has no subsequent descent, sinking wakes or wake-displacement
   draws. On each of a retained vessel's own updates (once per tick, S-0203), its height above the terrain is 5 leptons lower than before that update. It leaves play on the first such update whose resulting height is strictly below −400 leptons (S-0312 claim 15 says what that implies). Its destruction, when sinking followed a killing hit, was already counted at that hit (S-0312 claim 10); a cull has no killing hit or ordinary destruction credit (S-0614 claim 11). If it remains in play on an update whose tick number is divisible by 4, it takes two ranged draws over [−170, 170] from the scenario random stream (S-0202 claim 4) and creates one wake at the displaced point of F2, at the water surface.
   A vessel that starts sinking at height 0 therefore leaves play at its 81st descent (F1), 80 ticks
   after its first descent, and creates 20 wakes in all with two draws each, since its 80 descents
   that leave it in play fall on 80 consecutive tick numbers. Its first descent comes in its first
   own update after sinking starts. For a killing hit, that is in the tick of the hit when the
   vessel's turn (S-0203) comes after the hit, otherwise in the next tick. S-0614 claim 11 owns
   the cull trigger; its timing within the host's turn remains unestablished here.
   Any passengers still aboard at final sinking removal are killed without attacker credit
   under S-0507 claim 16. They are not released or preserved,
   and have no independent death effects. This does not change earlier passenger escapes or
   attacker-credited deaths under S-0312 claims 5, 12 and 16.
   When no wake animation is defined, our engine still takes the two draws on such a tick and
   creates no wake, and the importer warns (a design choice: the original's behaviour without a
   wake animation was not established, and stock data always defines one).
   - Backing: `measurable` (scenario N5; the draws with the random-state recorder) for
     ordinary sinking from height 0 with a wake animation; N5's loaded-vessel variant and
     S-0507 scenario T14 for remaining-passenger cleanup. F1 at other initial heights is
     `unmeasured` here: N5 does not establish a sinking start at another height. The no-animation outcome is a design rule,
     `unmeasured` against the original because its outcome for this input is not established.
   - Confidence: high for the descent, the threshold, the draws and their order; medium for the tick
     of the first descent, which follows the update order rather than a separate reading

5. **A sinking vessel in play.** A sinking vessel:
   - makes no progress by its locomotor, including its locomotor's turning and speed-fraction
     updates (so S-0207 claim 10's sinking case is never reached for it). This does not freeze
     other updates or guarantee that every facing change stops,
   - gets the answer *illegal* when its firing check reaches the sinking-firer test, a reason of
     S-0301 claim 9 rank 4 (the firer's state); earlier reasons retain their precedence,
   - cannot be newly assigned as a target: assigning it leaves no target, subject to the
     redirected-target rules of S-0309 claim 24. This does not itself establish when a target
     already held before sinking is cleared,
   - while it is still asked about as a target, gives the firer the answer *illegal* among the late
     target tests of S-0301 claim 9 rank 9, so a firer whose rearm countdown runs, that is out of
     ammunition or that is cloaked gets that answer instead,
   - retains its horizontal position during ordinary descent. Continued blocking in every
     occupancy test is not established here.
   Other sinking triggers belong to their movement specs. S-0502 claim 19 owns the
   jumpjet trigger after a magnetic-lift drop onto water where the vehicle cannot stand,
   not on a deck or beach; an ordinary shot-down jumpjet is not set sinking by its movement.
   The magnetic-lift water-landing outcome belongs to S-0607 claim 17. Other possible movement triggers remain
   outside this claim. A squid's cull is S-0614 claim 11, not an ordinary killing hit.
   Once sinking is set, claims 4 and 5 apply in the same way, without adding health loss
   or destruction credit to the cull.
   - Backing: `measurable` (N6) for firing prevention, rejection of new target assignments and
     horizontal position. Locomotor-only turning suppression is `unmeasured`: N6 does not
     separate locomotor turning from other facing updates.
     Exact precedence of unsuccessful firing answers is `unmeasured`: no setup is established
     that retains a sinking target through earlier tests and exposes the answer by normal input.
     Unchanged speed fraction is `unmeasured`: sinking prevents independent travel-speed tests.
   - Confidence: high for locomotor suppression, firing checks and rejection of new targets; medium for
     horizontal position

### Underwater vessels

6. **What the underwater flag changes.** An underwater-type vehicle using the ship locomotor
   moves by claims 1 and 3 exactly as a surface ship, is in the same layer and occupies cells in
   the same way, and has no depth of its own.
   Its effects covered here are: it makes no wakes while moving (claim 2), it does not enter
   sinking from the ordinary killing-hit condition (S-0312 claim 10), and it is tested by area damage (S-0303 claim 21), the naval weapon choice
   (S-0308 table N, which also tests whether it is submerged) and the threat search (S-0309 F1).
   Whether other objects see it while it is submerged is planned S-0516's cloak rule.
   The underwater flag and the ability to cloak are independent type settings. Without that
   ability, a vehicle remains surfaced for S-0308 claim 11's naval weapon choice; the underwater
   flag still suppresses moving-ship wakes and excludes ordinary killing-hit sinking. This is
   the ordinary combination of those settings, not a failed submerging transition.
   - Backing: `measurable` (N1 and N2 repeated with an underwater ship for movement; N3 for
     wakes; N9 for the no-cloak combination). Other damage, targeting, death and detection rules
     inherit the cited claims' backings. Separate
     depth and cell-occupancy equivalence are `unmeasured` here: no scenario isolates height or
     competing occupancy tests.
   - Confidence: high

### Docking at buildings

7. **Naval match (Q-0643).** A building refuses a unit that asks to enter or dock (its docking
   answer is *refuses*) when exactly one of the building and the unit has the naval flag,
   **unless the unit's movement zone is plain amphibious**. Only that one zone bypasses the
   naval-flag comparison. Amphibious-crusher and amphibious-destroyer do not. Neither a ship zone,
   the ability to cross water, a terrain-speed class nor locomotor kind supplies another exception.

   | Unit's movement zone | Equal building/unit naval flags | Unequal naval flags |
   |---|---|---|
   | amphibious | passes this test | passes this test |
   | amphibious-crusher | passes this test | refuses |
   | amphibious-destroyer | passes this test | refuses |
   | every other zone, including ship | passes this test | refuses |

   Thus a plain-amphibious hovering craft passes this test at both a depot and a naval yard.
   Its naval flag remains unchanged. A craft with either other amphibious zone must match the
   building's naval flag. A ship-zone vehicle whose naval flag is off does not match a naval
   building merely because of its zone. Passing this test is not acceptance: other refusal
   conditions (alliance, construction, power and the like) are unchanged.
   - Backing: `measurable` (scenario N7 for stock pairings and the plain-amphibious exception).
     The other two amphibious zones and zone/flag mismatches absent from stock types are
     `unmeasured`: N7 does not change imported type data to isolate them.
   - Confidence: high

8. **Repair at a yard, and the starting distance.** A naval yard whose type has the repair flag
   repairs vessels by the depot rules of S-0314 (claim 6). It also takes S-0108 claim 20's
   repair-cycle animation-slot rows; only that claim's every-tick depot row excludes naval types.
   Claim 12 specifies the naval-yard
   docking point and stopping rule; click legality remains S-0214's building-entry rule, and
   ordinary enter behaviour is S-0215 claim 29. An enter order or initial contact is not itself
   a paid repair. Before its settling phase a depot (land
   or naval) with a docked vehicle *waits for it*: at each of its processings (S-0215 claim 2) it
   asks whether the vehicle stands still and measures its spatial distance to the vehicle's
   docking point at the building, not necessarily the building centre (F3). When the vehicle stands still
   and is closer than the starting distance, 200 leptons for a vehicle with the ship or the hovering
   locomotor and 100 leptons for every other locomotor (strictly less in both cases), the depot ends
   waiting and its next processing comes 3 ticks later, in the settling phase of S-0314 claim 6.
   At that later processing, the first repair step additionally requires continued contact,
   continued stillness and completed locomotor settling (S-0314 claim 6); scheduling settling
   does not guarantee a paid repair on that tick.
   An unsuccessful waiting check instead uses the depot's processing delay *D* of S-0314 claim 6.
   Thus settling is scheduled 3 ticks after the processing that first finds the vehicle at rest
   within the starting distance, not in the tick it comes to rest. The settling phase's separate
   200-lepton distance check controls approach and stopping, not eligibility for a repair step.
   It is not a second repair-distance gate.
   - Backing: `measurable` (N8) for thresholds, spatial distance and delays. Animation-slot rules
     inherit S-0108 claim 20's backings, including B18's naval-yard control and the unmeasured
     limitation for the every-tick row outside repair. Complete docking-point
     selection is `unmeasured` here: no scenario covers every building and dock-place combination.
   - Confidence: high for thresholds, locomotor test, spatial distance and 3-tick delay,
     medium for *D* while waiting and the exact docking point at each yard

9. **Ship path refresh and retry timing (Q-0644).** The ship locomotor shares S-0212 claim 20's
   last-stored-step refresh with the driving locomotor. As the last stored step begins, a distance
   strictly greater than 512 leptons to its heading point triggers a search in mode 0,
   immediately, without waiting for the path retry timer. Distance is the spatial table-root
   distance of S-0212 claim 20 and S-0204 claim 5, truncated toward zero to integer leptons.
   The squared coordinate differences and their sum are evaluated in double-precision floating
   point under S-0205's round-toward-zero mode, exact for signed 32-bit differences whose
   squared sum is below 2⁵³ square leptons. The shared `quirks.table_root_distance` switch
   selects the exact-root alternative of S-0204 claim 5 when off; there is no ship-specific
   exact-root exception. This definition applies to both refresh and chase cutting below.
   At 512 leptons there is no such refresh. The refresh does not itself start a new retry delay.
   With `quirks.path_refresh_keeps_old_step` on (the original behaviour), the ship keeps the old
   step already chosen and drops the first step of the replacement path, just as a driver does.
   With it off, the replacement's first step is used. S-0212 claim 20's train exception is unchanged.

   The ship also shares that claim's chase-path shortening with driving and hovering vehicles:
   for a vehicle or infantry destination it cuts the stored path at ⌊*D* / 256⌋ steps when that
   count is below 24, with *D* the distance just defined. S-0212 claim 20 includes ships in
   both rules; there is no new search algorithm or ship-specific path buffer. Hovering vehicles
   do not make the last-step refresh; driving vehicles do (S-0212 claim 20).

   The ordinary no-path search waits for expiry of the same retry timer. Its delay is S-0212
   F3's *R* = trunc(*P* × 900) ticks, with *P* the imported path delay in game minutes:
   floating-point multiplication, truncated toward zero to signed 32-bit ticks under S-0205.
   The drive rules of S-0212 claims 21 and 22 also apply to ships: a successful ordinary fresh
   search clears the delay, unlike a hovering vehicle's successful fresh search (S-0503 and
   S-0212 claim 21); a failed one leaves it running from that search tick. The moving-blocker
   re-path waits for this timer, chooses its search mode using the separate blockage countdown,
   and restarts *R* after a search, whether successful or failed; a failed search gate clears it.
   Neither ship speed nor the water terrain coefficient changes *R*, and it takes no jitter draw.

   At a first step check, a cloaked blocker discards the path and reveals the cloaked objects
   as S-0207 claim 15 specifies; a new search waits for timer expiry. A blocked cell, stationary
   friendly occupant, friendly obstacle or enemy obstacle instead discards the path and searches
   immediately, regardless of that timer; a found path is checked again on the same tick.
   This agrees with S-0207 claim 15 and differs from S-0503 claim 12, where the hovering
   locomotor also searches immediately after a first cloaked-blocker check. Other step-check
   effects remain claim 1's driving equivalence, including the ship-specific crush exception.
   - Backing: `measurable` (N11 for search timing and blockers). The exact lepton-scale refresh
     boundary and table-root versus exact-root distinction are `unmeasured`: N11 does not
     establish a qualifying separation that distinguishes them. Also `unmeasured` for isolating
     the retained-step refresh and chase cut with stock data: no setup here separates these from
     unchanged routes or the chased unit's own cell-entry effects.
   - Confidence: high (timer, triggers and shared branches); medium (retained-step route effect)

10. **Effects that continue while sinking (Q-0642).** Beginning the killing-hit sinking state
    cancels the vessel's destination and its own target and asks its locomotor to stop. This
    does not clear every other object's already-held target. Body-facing updates are separate
    from locomotor processing: a turn already in progress can continue towards the saved desired
    facing while the vessel descends. Sinking does not itself assign a new desired body facing.
    Claim 15 owns the vessel's lean during descent, a fore-and-aft pitch whose direction is set by
    the body facing at the first update while sinking, so a turn that continues does not change
    it. The visual tilt of a vessel during descent, which S-0106 draws, is this lean: claim 15's
    pitch, drawn as the rocking of S-0106 claim 9 (claim 16). It is a fore-and-aft lean, not a
    sideways list. Sinking adds no rule of its own for the roll (claim 15).

    Sinking is not immunity to damage delivered to the object. An already-fired shot can still
    damage it when the ordinary impact rules deliver damage to that object; applicable area
    damage and radiation can also do so. Sinking does not disable ordinary self-healing or
    the owner's periodic unit healing (S-0314). Their normal eligibility, cadence and caps
    still apply. Healing does not clear sinking or cancel descent/removal. These rules grant
    neither a new target nor a guaranteed impact to a projectile whose ordinary impact test
    misses. Claim 5's targeting and firing refusals therefore do not cancel airborne projectiles.
    Health-changing effects use the ordinary integer-health arithmetic of their owning specs.
    Sinking adds no health multiplier or rounding. Claim 13 specifies the descent consequence
    of a later killing hit. Its repeated destruction credit, raises and death effects belong
    to S-0312 claim 32, not a new naval death rule here.
    - Backing: `measurable` (N10).
    - Confidence: high (continuing updates and no damage/healing exemption); medium (isolating
      each stock effect within the short descent interval)

11. **Absent vehicle weight (Q-0640).** A newly imported vehicle type without a weight entry
    starts with exactly 1.0, a dimensionless floating-point double. A later import without
    that entry preserves its existing weight (S-0006), not necessarily 1.0. This default enters
    S-0312 claim 10's inclusive comparison with the imported sinking threshold unchanged.
    With its other killing-hit conditions satisfied, absent weight permits sinking exactly
    when that threshold is at most 1.0; a threshold greater than 1.0 prevents it. Locomotor
    kind does not select another default or bypass the weight test. A parasite's cull bypasses
    this comparison (S-0614 claim 11).
    - Backing: `measurable` (N12 for stock death outcomes); `unmeasured` for the isolated absent
      value and threshold boundary: a stock-only setup cannot independently vary the threshold
      or the presence of a weight entry.
    - Confidence: high

12. **Naval-yard docking point and stopping (Q-0641, partial).** An ordinary naval repair
    yard uses S-0314 claim 18's docking-point selection. Without obsolete-unloading,
    resource-unloading or tank-bunker flags, the point is the foundation centre, including
    world height, plus the imported docking displacement: the sole displacement for a count
    of 1, or the assigned contact place's displacement for a greater count. A count of 0 or
    no assigned place within the displacement count gives the foundation centre instead.
    The obsolete-unloading flag, not the vehicle-factory flag, selects the fixed centre of
    the cell displaced by (2, 1) cells from the top-left foundation cell, at yard height,
    taking precedence over pad displacements. Coordinates and additions use signed 32-bit
    integer leptons with S-0205 claim 1's wrapping arithmetic; displacement selection adds
    no floating-point calculation, rotation or random draw.

    A ship completes its ordinary cell-to-cell track. At track completion, reaching the
    docking-point cell with a vertical difference strictly below 208 leptons suffices to
    stop the approach (S-0314 claim 18). On ordinary level tracks its stopping position is
    the cell centre, not necessarily the supplied point's horizontal position. Claim 8 still
    measures distance to the supplied point, including its height: stopping need not put the
    ship inside the starting distance. Neither point selection nor stopping searches for a
    nearest-water cell, uses the rally point or uses S-0404 claim 13's production release
    position. The docking cell may belong to the foundation; S-0209 claim 5 retains ownership
    of entry legality and blocking versus non-blocking foundation cells.
    - Backing: `measurable` (N13) for ordinary point selection and cell-centre stopping.
      Unused flag combinations and the exact 208-lepton vertical boundary are `unmeasured`:
      no stock-only setup here isolates them without also changing the route.
    - Confidence: high (point selection, precedence, integer arithmetic and cell-based stop),
      medium (stock approach observations)

13. **Further killing hits during descent (Q-0646, naval part).** For a sinking vessel without
    death image frames that still meets S-0312 claim 10's ordinary killing-hit sinking
    conditions, another killing hit leaves it at 1 health point again. Unless S-0312 claim 15's
    crashable removal applies, it remains sinking and in play. The hit does not put the vessel
    back at the surface or restart its descent: claim 4 continues from its current height.
    Thus, absent another effect that changes height or removes it, a further killing hit does
    not extend its removal tick. Healing between hits can allow a later hit to be nonlethal.
    Ordinary health arithmetic then applies (S-0302 claim 11).

    The same descent rule applies when sinking began through a cull: the cull itself changes no
    health, whereas a later killing hit is tested against the ordinary killing-hit conditions.
    A cull does not make an underwater or insufficient-weight type satisfy those conditions.
    The death outcome when they fail belongs to S-0312 claims 9 to 15, not an unconditional
    naval restoration to 1 health. S-0312 claim 32 owns repeated destruction credit, raises,
    applicable death effects and their draws, and the health/removal outcomes when ordinary
    sinking eligibility fails. Its scenario D23 tests those consequences. Types with death
    image frames retain that claim's unresolved separate path; neither health outcome is
    asserted for them here.

    If a sinking vessel has 0 health, sinking does not waive the no-health exclusion on
    ordinary damage or healing hits (S-0302 claim 8), individual self-healing (S-0314 claim 9),
    or house-wide self-healing (S-0314 claim 10). In particular those two periodic heals do
    not revive it. This conditional rule does not assert that every cull type remains in play
    at 0 health or continues taking ordinary living updates.
    - Backing: `measurable` (N14 for further hits and unchanged descent; N10 for positive-health
      healing). The zero-health case inherits the cited claims' `unmeasured` limitations:
      N14 does not establish a surviving zero-health vessel.
    - Confidence: high (restoration under the ordinary sinking condition, no height reset and
      unchanged healing exclusions); medium (isolating a later impact during descent)

14. **Naval-yard interruptions and queued entry.** At a naval yard with the repair flag,
    S-0314 claims 20 to 23's depot contact rules apply to vessels too. In particular an order
    other than the ignored repeated approach releases an approaching vessel's contact, whereas
    a docked vessel ordinarily retains contact and tether under a move or attack order.
    The same-depot entry and successful entry elsewhere retain claim 20's exceptions. With
    contact retained, waiting still requires claim 8's starting distance, but a stationary
    vessel in the working phase can receive paid repair away from the yard. Settling has
    S-0314 claim 21's approach/stop checks, not a universal repair-distance gate.

    Sale gives the contacted vessels the run-away notice and releases contact at the yard's
    next processing. Destruction releases contact without that notice: an approaching vessel
    falls back to guard, and a docked vessel stays asleep until ordered. Low power or switching
    the yard off does not stop repair already in contact; switching off refuses new acceptance.
    Destruction of a contacted vessel frees its place. The shared switches
    `quirks.depot_remote_repair` and `quirks.depot_loss_leaves_sleep`, including their alternatives,
    are S-0314 claims 21 and 22's, not separate naval switches.

    Simultaneous enter orders use S-0314 claim 23's event-order and queued-entry rules: a
    damaged later vessel waits and retries for a place, while a full-health later vessel does
    not queue. This is distinct from a later click whose permission fails and issues no enter
    order. The serviced place and several-place departure remain S-0314's; this claim does
    not promise parallel repair of every contact place.
    - Backing: `measurable` (N15).
    - Confidence: high (shared repair checks, power and queuing); medium (order and sale/removal
      consequences); low (same-yard enter order while docked, inherited from S-0314 claim 20)

### The lean of a sinking vessel

15. **Pitch while sinking (Q-1871).** While a vehicle drawn as a voxel model is sinking, each of
    its own updates takes one pitch step towards a full lean of π/4, bow
    down or stern down. A step adds 0.01 radians (bow down) or subtracts it (stern down). A step
    that would pass the full lean stops at it, and once the pitch is at the full lean it stays
    there. F4 gives the arithmetic.
    - **Direction.** The lean is bow down when the 8-direction of the vehicle's body facing
      (S-0204 F4) at its first step is 1, 2, 3, 4 or 5: map north-east, east, south-east, south
      or south-west. It is stern down when that 8-direction is 0, 6 or 7: map north, west or
      north-west. On screen, the end of the hull that is lower on the screen goes down; for the
      two headings that run across the screen, north-east and south-west, it is the bow. The
      direction is kept for the rest of the descent, and no random draw is taken.
    - **Which facing.** The direction is taken from the body facing in the tick of the first
      step, its value for that tick number under S-0206 claim 1, also while a turn continues
      (claim 10).
    - **Course of the lean.** From a level hull, after *n* steps the pitch's magnitude is about
      0.01*n* radians; F4 gives the exact single-precision values. The 79th step brings it to the
      full lean, π/4 to single precision, and no later step changes it.
    - **Timing.** The step comes in the same own update as claim 4's descent. After *n* descents
      the vessel has taken *n* steps, until the full lean stops them. A vessel that starts sinking
      at height 0 leaves play at its 81st descent. Starting from level, with no effect that sets
      the pitch directly, its lean is complete at the 79th descent; the 80th update holds the lean,
      and the 81st removes the vessel. When the lean is still incomplete on the update that removes
      the vessel, that update's step is never drawn. A vessel whose descent ends in fewer than 79
      updates, because it started below height 0, leaves play with its lean incomplete. A start
      above height 0 only lengthens the time it spends at the full lean.
    - **Starting value.** Steps start from the pitch the vehicle had when sinking began (for
      example, left over from earlier rocking), not from level. A pitch that starts in the
      opposite sense passes through level first.
    - **The pitch belongs to the lean; the roll is left free.** While sinking, the forward
      rocking rate is not added to the pitch, the pitch does not return to level, and S-0210
      claim 12's π/10 limit does not apply to it. Sinking adds no rule of its own for the roll
      and the sideways rocking rate, and does not hold them: a rocking hit during descent sets
      the sideways rocking rate as S-0303 claim 20 sets it (a strength scaled by the type's
      weight; S-0614 claim 10 for a squid's strike), as on a vessel that is not sinking, and S-0106
      claim 9 draws the roll that follows. Such a hit rolls the hull but does
      not change its lean. How a rolled body comes back to level is not stated by any approved
      spec, and this spec does not supply it (Open questions); whether a roll left over from
      before sinking, or set during descent, returns to level while the vessel sinks is
      `unmeasured` here.
    - **Turning.** A body turn that continues during descent (claim 10) does not change the
      direction of the lean, even when it carries the body facing into the other group of
      8-directions.
    - **The original's drawing.** Three oddities of the original are not copied and have no switch
      (decided at the delegated sign-off, 2026-10-25):
      - The original tests the limit on the pitch's magnitude before each step, so its lean ends
        up to one step past π/4 (about 0.79 radians from level), and a pitch already at the limit
        in the opposite sense stays there.
      - The original reads the direction again on every update, so a turn that crosses into the
        other group of 8-directions before its lean reaches the limit reverses its lean back
        through level towards the other end.
      - The original holds the roll and both rocking rates for the whole descent, so a roll from
        before sinking stays drawn until the vessel leaves play, and a rocking hit during descent
        does not move the body.
    - **Causes and types.** The rule depends on the sinking state alone: it is the same after a
      killing hit (S-0312 claim 10), a squid's cull (S-0614 claim 11), a magnetic-lift drop into
      water (S-0607 claim 17), the jumpjet trigger of S-0502 claim 19 and any other cause, for
      every type. Underwater types do not start sinking at a killing hit (S-0312 claim 10). One that
      starts sinking another way leans in the same way. A vehicle not drawn as a voxel model
      takes no pitch steps.
    - Backing: the full-lean stop, the kept direction and the roll that sinking leaves free are
      our engine's design and have no original result to measure; N16 checks them in our engine.
      `measurable` (N16 for direction at all eight 8-directions, the lean's growth by about equal
      amounts per descent, its stop near the 79th descent and the absence of draws; its parts a and
      b for the original's reversal and held roll; part c for the original's held angles under a
      rocking hit during descent; part h for a start from a rocked pitch; part e for the
      cull cause). The magnetic-lift cause is `unmeasured`: part f records it, but a level body
      there is not a falsifier (claim 16, Open questions), so that run cannot tell the rule from
      its absence. The original's held rates are `unmeasured`: a drawn body that
      does not change cannot tell a held rate from one that is cleared or ignored while sinking,
      and no stock observable reads the rates. The original's overshoot is recorded only where
      the object recorder reads the pitch, and is otherwise `unmeasured` like the exact values. The
      original's hold of a pitch already at the limit in the opposite sense is `unmeasured`: no
      stock setup rocks a hull to the limit just before it sinks. The exact single-precision values
      are `unmeasured`: a drawn image cannot resolve them, and they only fix the step on which the
      full lean is reached. For the same reason the exact step of 0.01 radians and a stop at the
      79th descent rather than the 78th or the 80th are `unmeasured` from frames: the 78th step
      gives 0.78 radians against a limit of about 0.7854, and a drawn image cannot tell these
      apart. They are shown only where the object recorder reads the pitch (N16). The roll's return
      to level during descent is `unmeasured` (above). A step on the removal update, when the lean is still incomplete, is
      `unmeasured` because nothing shows it. Starts at other heights are `unmeasured`, as for claim
      4: N16 part d cannot set them with stock data. The other causes (the jumpjet trigger, and any
      cause not named here) and underwater types are `unmeasured`: no scenario here sets them up
      with stock data, and the claim for them rests on the rule keying on the sinking state alone.
      "No pitch steps" for a vehicle not drawn as a voxel model is `unmeasured`: its pitch is not
      drawn and nothing else reads it (claim 16), so the difference has no observable consequence.
    - Confidence: high (the direction groups, no draw, the lean growing by about equal amounts per
      descent from level and stopping at about 45 degrees within a descent or two of the 79th, the
      original's held roll); medium (the exact step of 0.01 radians and the stop at the 79th
      descent itself, which frames cannot resolve and which rest on F4's arithmetic unless the
      object recorder reads the pitch; the roll that sinking leaves free, a design rule checked
      only in our engine; the original's held rates, `unmeasured` because part c's drawn body shows
      the held angles and not the rates themselves; the original's limit test, which a drawn image
      cannot resolve; the original's reversal while turning, which depends on claim 10's continuing
      turn; the starting value; the same rule for the magnetic-lift drop and the other causes, and
      for underwater types; no steps for a vehicle not drawn as a voxel model)

16. **How the lean is drawn, and what reads it (Q-1871).** A sinking vessel's pitch and roll are
    drawn as the rocking of S-0106 claim 9.
    - **Drawn transform.** For the driving and ship locomotors, inside its facing rotation and
      slope tilt, the body is rolled about its own forward axis and pitched about its own lateral
      axis, roll outermost, and it is also lifted and shifted by whole model units. F5 gives the
      whole transform, which these two locomotors use for any rocking, not only for sinking. For
      the hovering and jumpjet locomotors this spec states only that the lean is drawn as S-0106
      claim 9's rocking: their order of roll and pitch, and any lift or shift, are not established
      here (Open questions). A turret and barrel are placed on the body transform (S-0105 claim 8),
      so they lean with the body.
    - **Image cache.** Whether a leaning body is drawn fresh or from the image cache is S-0105
      claim 26's rule for the driving and ship locomotors and S-0106 claim 13's for the hovering
      and jumpjet locomotors. Sinking adds nothing to them.
    - **Which locomotors show it.** Only bodies whose locomotor draws rocking show the lean: the
      driving, ship, hovering and jumpjet locomotors (S-0106 claims 5 and 9). For a body drawn by
      another locomotor, the pitch still changes by claim 15 but is not drawn.
    - **Shadow.** A sinking vehicle drawn as a voxel model draws no shadow at all, from the first
      drawn frame after sinking begins until it leaves play. So the lean never reaches a shadow.
    - **What reads it.** Among the vessel's own rules, drawing is the only reader of the pitch and
      roll of a sinking vessel.
      - None of its own distance, occupancy, targeting or damage rules uses them.
      - A sinking vessel cannot fire (claim 5), so none of its own firing positions of S-0313 is
        formed from its tilted body.
      - **Passengers.** A vehicle that starts sinking at a killing hit carries no passengers
        during its descent: S-0312 claims 10 and 12 settle them at the hit. A transport
        that starts sinking another way (claim 15, *Causes and types*) keeps its passengers until
        it leaves play, when they die (S-0507 claim 16). Those of an open-topped transport stay in
        play and take their own turns (S-0507 claim 14). The sinking test of the firing check is
        the firer's own, so the transport's sinking does not stop them firing. They fire from the
        transport's fire point (S-0313 claims 1 and 5), which is built on the transport's body
        transform, so the lean and the roll of its descent move the starting point of their
        shots. The fire point and the drawing use the same transform: S-0313 claim 1 reads its
        translation as a lepton shift, so F5's lift and shifts, which drawing takes in whole model
        units, move the fire point by the same numbers in leptons.
      - Our engine may hold the angles as presentation state, provided they advance once per own
        update by claim 15 and not once per drawn frame.
    - Backing: `measurable` (N16 for the end that goes down, no shadow and the visible
      locomotors; its frames also compare F5's lift against a render). "No other
      reader" among the vessel's own rules is `unmeasured`: no stock observable can show the
      absence of a reader. F5's roll terms and their signs are `measurable` through N16 part b's
      runs on the original, which hold a roll at full lean. The half unit added to *W* (F5) changes
      the lift or the sideways shift by at most one model unit, which is one screen pixel (S-0105
      Terms), and only while the roll is not zero. The main run of N16, with no roll, cannot show
      it. Part b's frames show it as a one-pixel shift of the outline, and only where the recorded
      roll puts a truncation on the other side of a whole unit; elsewhere it is `unmeasured`. That
      F5's size values come from the first section of the body's main model is `unmeasured`: it
      can be told from another section only on a body whose model has more than one section, and
      no N16 run is set up with such a body. The roll-outermost order and the drawn sense are
      `measurable` for the ship locomotor only (N16 and its part b); for the driving locomotor
      they rest on the same F5 with no run of their own (part f records a level body only
      as evidence). The passengers' firing, and the
      lepton reading of F5's lift and shifts in their launch positions, are `measurable` (N16
      part g).
    - Confidence: high (for the ship locomotor, the drawn pitch axis and its sense under S-0105's
      convention; no shadow); medium (for the ship locomotor, the roll axis, its sense and the
      roll-outermost order, which rest on N16 part b's comparison with renders, as F5's roll terms
      do; the same order and sense for the driving locomotor, which no run here checks; the first
      section as the source of F5's size values; F5's lift and shift, and its roll terms, not yet
      compared with a render; no reader other than drawing among the vessel's own rules; the
      passengers' firing during descent and its starting point; the locomotor list where the body is
      drawn by a locomotor attached for a lift or a teleport)

## Formulas

**F1. Sinking descent (claim 4).** Integer leptons, for a vessel retained in play as claim 4
requires. With *h*₀ the vessel's height above the terrain when it starts sinking, after its *n*-th descent

> *h*ₙ = *h*₀ − 5*n*.

It leaves play at the first *n* with *h*ₙ < −400, that is *n* = ⌊(*h*₀ + 400) / 5⌋ + 1 for
*h*₀ ≥ −400 (floor of a non-negative quotient). For *h*₀ = 0, *n* = 81.

**F2. Sinking wake position (claim 4).** Integer leptons. With (*x*, *y*, *z*) the vessel's position
after the tick's descent and *h* its height above the terrain then, the first draw gives *r*₁ and
the second *r*₂, each a ranged draw over [−170, 170] inclusive:

> wake position = (*x* + *r*₂, *y* + *r*₁, *z* − *h*).

The first draw displaces along the map's *y* axis and the second along *x*. *z* − *h* is the height
of the terrain (the water surface) under the vessel.

**F3. Depot distance (claim 8).** Vehicle position (*x*, *y*, *z*) and its docking point
(*a*, *b*, *c*) have integer-lepton coordinates. The original evaluates the squared
differences, their sum and the square root in double-precision floating point, with
round-toward-zero arithmetic, then truncates the result toward zero to integer leptons:

> *d* = trunc(√((*x* − *a*)² + (*y* − *b*)² + (*z* − *c*)²)).

Height counts. This is not the table square root of S-0204. Claim 12 applies the naval-yard
point selection of S-0314 claim 18; the ship's stopping position need not equal that point.

**F4. Pitch step while sinking (claim 15).** The pitch *p* is a single-precision real in radians.
*L* = 0.785398185253143310546875 is π/4 rounded to single precision. *c* = 0.01 is rounded to double
precision. *g* is the 8-direction of the body facing at the first step while sinking (S-0204 F4).
On each own update while sinking:

> *p*′ = min(RZ₂₄(*p* + *c*), *L*) when 1 ≤ *g* ≤ 5,
> *p*′ = max(RZ₂₄(*p* − *c*), −*L*) when *g* is 0, 6 or 7,

where *p*′ is the pitch after the update. A pitch at the full lean (*L* bow down, −*L* stern down)
therefore stays there.

The sum or difference is formed in double precision and narrowed to single precision, both under
S-0205 claim 5's round-toward-zero mode. That equals RZ₂₄ (S-0205 Terms) of the exact sum. From
*p* = 0 in either direction, the 78th step gives |*p*| = 0.779999315738677978515625 and the 79th
stops at *L*, the final value. With `quirks.chop_rounding` off (S-0205), RN replaces RZ: the step
count is still 79, and the 78th magnitude is 0.779999554157257080078125. For any other starting
pitch, the number of steps follows from the same recurrence. The original instead holds *p* when
|*p*| ≥ *L*, otherwise takes the step without the min or max, and reads *g* again on every update,
so from level its 79th step gives |*p*| = 0.789999306201934814453125 (claim 15, *The original's
drawing*).

**F5. Rocking body transform, driving and ship locomotors (claim 16).** S-0106 claim 9 leaves this
composition to the vehicle specs. With roll *r* and pitch *p*, the body transform *D* of S-0105
claim 7 is

> *D* = Z(lift) · *T* · Rz(*θ*_body) · X(*s*ₓ) · Y(*s*ᵧ) · Rx(*r*) · Ry(*p*),

where *T* is the tilt of S-0106 claim 9, Rz(*θ*_body) is the body rotation of S-0105 claim 7, and
X, Y and Z are translations along the current local frame's axes, as X(*d*) is in S-0105 claim 8.
So the lift is along the map's vertical, and the shift is along the body's forward and left axes
after its facing rotation, before it is rolled and pitched. The lift and the two shifts are whole
model units:

> lift = trunc(|sin *r*| × *W* + |sin *p*| × *H*),
> *s*ₓ = trunc(*H* − trunc(*H* × cos *p*)), negated when *p* < 0,
> *s*ᵧ = trunc(*W* − trunc(*W* × cos *r*)), negated when *r* > 0.

*W* and *H* are size values of the body model: *W* = *Ny* × 0.5 + 0.5 and *H* = *Nx* × 0.5, where
(*Nx*, *Ny*, *Nz*) is the voxel grid size of the first section of the body's main model (S-0105
Terms). Each is formed in double precision and held in single precision. sin and cos are the table
values of S-0105 F1 (single precision), and trunc is the conversion to integer of S-0205 claim 7.
The products, sums and differences are double precision under S-0205 claims 4 and 5. A level body
(*r* = *p* = 0) gets no lift and no shift.

The outline this gives, for a body model centred on its pivot at one model unit per voxel: the
body tips about the edge of the end that goes down and, for a roll, of the side that goes down.
That edge stays where the level body's edge was, to within the whole-unit rounding and the extra
half unit of *W*, and the rest of the body rises from it.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ship wake period | 8 | ticks (tick number divisible by) | Claim 2 |
| Sinking descent | 5 | leptons per tick | Claim 4 |
| Sinking removal height | −400 | leptons (strictly below) | Claim 4 |
| Sinking wake period | 4 | ticks (tick number divisible by) | Claim 4 |
| Sinking wake displacement | [−170, 170] | leptons per axis, inclusive | Claim 4, F2 |
| Sinking pitch step | 0.01 | radians per own update (double precision, sum stored single) | Claim 15, F4 |
| Sinking pitch limit | π/4 (single precision); a step stops at it | radians | Claim 15, F4 |
| Rocking size factors | 0.5 per voxel, plus 0.5 on the width | model units | Claim 16, F5: *W* from the y grid size, *H* from the x grid size |
| Steps to the limit from level | 79 | own updates | Claim 15, F4 |
| Bow-down 8-directions | 1 to 5 (others: stern down), read at the first step | 8-direction | Claim 15 |
| Depot starting distance, ship and hovering locomotors | 200 | leptons | Claim 8 |
| Depot starting distance, other locomotors | 100 | leptons | Claim 8 |
| Waiting-to-settling delay | 3 | ticks | Claim 8 |

## State and ordering

| State | Event | Result |
|---|---|---|
| ship, moving now after locomotor movement | tick number divisible by 8, conditions of claim 2 hold | one wake at its post-movement position |
| ship, between tracks | launching count is positive | keeps waiting with destination kept (claim 3) |
| alive vessel | killing hit, S-0312 claim 10 holds and claim 15 retains it | sinking at 1 health point, in play |
| alive crashable surface vessel | first killing hit, S-0312 claim 10 holds, height 0 above water | removed during the hit under S-0312 claim 15; no subsequent descent or sinking wakes |
| sinking | own update | descends 5 leptons; then leaves play if below −400 leptons, else on a tick number divisible by 4 two draws and one wake |
| sinking | firing check reaches its sinking test | *illegal*, subject to claim 5's earlier answers |
| sinking | ordinary damage or healing reaches it | ordinary health rules apply, with no sinking exemption (claim 10) |
| sinking with a body turn in progress | own facing update | saved body-facing turn continues independently of locomotor suppression (claim 10) |
| sinking, voxel model, pitch short of the full lean | own update | one pitch step towards bow down (8-direction 1 to 5) or stern down (0, 6, 7), stopping at π/4; the direction is the one set at the first step; sinking holds no roll (claim 15) |
| sinking, voxel model, pitch at the full lean | own update | pitch unchanged; sinking holds no roll (claim 15) |
| sinking, voxel model | drawn frame | body drawn with its pitch and roll where its locomotor draws rocking; no shadow (claim 16) |
| docked vehicle at a depot | stands still and is closer than the starting distance | settling scheduled 3 ticks later (claim 8) |

The pitch step of claim 15 takes no draw and comes in the same update as the descent, so the frame drawn after a vessel's *n*-th descent shows *n* pitch steps, until the limit stops them. The observable consequences within a sinking vessel's update are that its height is 5 leptons lower, and, if it leaves play on that update, it takes no wake-displacement draws on that tick. If it remains in play on a tick divisible by 4, it takes the two wake-displacement draws during its own update. Relative to other objects' draws, these draws follow the update order of S-0203.

## Edge cases and quirks

Claim 9 extends the existing `quirks.path_refresh_keeps_old_step` switch of S-0212 to ships,
with the same original default and corrected replacement-step choice. Its refresh and chase-cut
distances also share `quirks.table_root_distance` of S-0204 claim 5, default original table root,
with that claim's exact-root alternative when off. It introduces no new switch.
The following distinctions are not optional original-versus-fixed behaviours:

- **Independent type settings.** Claim 6's no-cloak combination follows the glossary's
  *submerged* definition. It is normal configuration behaviour, not a quirk that enables
  submerging without cloak ability. N9 tests the combination explicitly.
- **Locomotor-specific presentation.** Claim 2 owns ship wakes; S-0207 claim 23 owns driving
  wakes. Different wake cadences are ordinary presentation rules, not an identified drawing
  defect. This spec does not offer an alternative cadence or override either claim.
- **Unknown concurrent waits.** No observable launch/wait interaction has been established, so
  there is no original result to preserve or fixed result to select. This remains an open
  question, not a quirk rule.
- **Missing sinking-wake animation.** Claim 4 explicitly supplies a deterministic design
  fallback, with an importer warning. The original result for that input is unestablished,
  so the fallback is not claimed to be the original default of a quirk switch. It has
  `unmeasured` backing against the original for that reason. No alternate result is specified.
- **The lean of a sinking vessel (claims 15 and 16).** Claims 15 and 16 add no switch. Three
  oddities of the original are fixed (decided at the delegated sign-off, 2026-10-25), and claim 15
  states the fixed drawing:
  - The lean stops at π/4. The original's lean passes it by up to one step and holds a pitch
    already at the limit in the opposite sense.
  - The direction is set at the first step and kept. The original's lean reverses back through
    level when a continuing turn crosses between the two groups of 8-directions before the lean
    reaches the limit.
  - Sinking does not hold the roll: a rocking hit during descent still rolls the hull. The
    original holds the roll from before sinking, and both rocking rates, for the whole descent.

  The asymmetric split of 8-directions (five bow down, three stern down) is the original's rule,
  kept, not a defect.

## Import mapping

| Key | Concept | Parse form | Absent |
|---|---|---|---|
| Rules, type section, `Locomotor` | ship locomotor (claim 1; S-0207 claim 14 owns the field) | class identifier | S-0207 |
| Rules, type section, `Naval` | naval flag (claim 7; S-0303 and S-0312 also use it) | boolean | no |
| Rules, type section, `Underwater` | underwater type (claims 2, 6) | boolean | no |
| Rules, type section, `Cloakable` | whether an underwater type can be submerged (Terms; planned S-0516 owns it) | boolean | no |
| Rules, type section, `MovementZone` | the plain *amphibious* exception only (claim 7; S-0211 owns the field) | movement zone name; the other two amphibious names do not bypass naval matching | S-0211 |
| Rules, `[General]`, `Wake` | the game-wide wake animation (claims 2, 4); claim 2's speed test and within-turn timing are engine rules, not additional imported settings | animation type name | none (engine default): a ship makes no wakes (claim 2); a sinking vessel takes its draws and makes no wake, with an importer warning (claim 4) |
| Rules, building section, `UnitRepair` | repair flag of a depot or yard (claim 8; S-0314 owns it) | boolean | no |
| Rules, building section, `Naval` | a naval building, for the naval match (claim 7) | boolean | no |
| Rules, type section, `Spawns` | spawned type in the shared controller (S-0506 claim 1); claim 3 uses its launching count, not ordinary aircraft preparation | type name | none |
| Rules, `[General]`, `ShipSinkingWeight` | sinking condition (S-0312 claim 10 owns it) | decimal, double precision | S-0312 |
| Rules, vehicle-type section, `Weight` | claim 11's vehicle weight | decimal, double precision | 1.0 at type creation; preserve current value on later import |
| Rules, building section, `WeaponsFactory` | vehicle-factory flag; does not select claim 12's fixed docking point | boolean | no |
| Rules, building section, `Weeder` | obsolete-unloading flag selecting claim 12's fixed docking point | boolean | no |
| Art, building section, `DockingOffset0` and subsequent numbered entries | docking displacements used by claim 12; S-0404 claim 15 owns their import | integer lepton triples; S-0404 | S-0404 |
| Rules, `[AI]`, `PathDelay` | claim 9's shared retry-delay field | decimal game minutes, single precision held as a double | S-0212 |
| Rules, building section, `NumberOfDocks` | contact-place count for claim 14; shared depot capacity and several-place handling | integer | 1; later absent entries preserve the value in force (S-0314) |

Claims 15 and 16 read no imported key. The pitch step, the limit, the direction groups and the
missing shadow are engine constants. Whether a type is drawn as a voxel model follows S-0105's
import. Claims 13 and 14 add no imported damage immunity, descent-restart delay or naval interruption
setting. Claim 13 uses the weight and sinking-condition rows above; healing amounts, periods and
eligibility remain S-0314 claims 9 and 10's import mapping.

## Measurement scenarios

Stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), a small purpose-made skirmish water map unless stated.
Use game-speed index 2 and one run per deterministic sub-test, with the glossary's isolated
bystander computer player and void rule. Repeat N5 four times to vary first-descent tick modulo 4.
Use tick-labelled frame capture and a read-only object recorder for positions, health, targets
and removal, and the random-state recorder where named. Record actual ticks, not assumed seeds.
N8 uses the processing delay at this speed (S-0215); speed can affect that delay.

**N1. Ship crossing time and path (claim 1).** *Setup:* open water, a stock ship and, on a land strip
of the same length, a drive vehicle; record both types' speed and turn fields from the data.
*Procedure:* order each straight along 10 cells, then along a path with one 45° and one 90° turn.
Repeat the straight run and issue a stop order mid-track by scripted input.
*Observable:* the tick of each cell arrival, the body facing per tick and the position and tick
at which the stop takes effect. *Expected:* the ship's
arrival ticks match S-0207 claims 2 to 12 computed from its own fields, as for the drive vehicle,
the turn in place before a step follows S-0206 claim 12; no tilt on water. *Falsified if:* an
arrival differs by a tick or more from the S-0207 prediction for the ship's own fields, or its
body facing differs from the predicted turning progression. In the stop variant, stopping must
match S-0207 claim 11; a different final position or stopping tick falsifies that equivalence.

**N2. Ship scatter and approach (claim 1).** *Setup:* a ship in *guard*, an enemy ship placed
out of range. *Procedure:* (a) issue a scatter on a moving ship mid-track; (b) order an attack.
*Observable:* the heading point and chosen scatter destination from the read-only recorder,
with the random-state recorder's draws to compare against S-0207 claim 16's predicted destination,
and the tick the ship stops at range. *Expected:* (a) the cell of the track target; (b) S-0207 claim 19's stop rule. *Falsified
if:* the heading point differs from the track target, the chosen scatter destination differs
from the prediction using that target and the recorded draws, or the stop tick differs.

**Heading-height comparison (claim 1).** Use S-0207 D27's ship cases, including its
legal-water control and qualified chaining control. Record the ship's world height immediately
before track start and its stored heading height throughout the track and each chain.
Expected: exact copying at track start and retention until completion, not target terrain or
deck height; a new track copies height anew. A different initial value, mid-track height
change or height refresh at a qualified chain falsifies the rule. N1 and N2 alone do not
isolate these alternatives. S-0511 B6 separately tests the bridge-loss consequences.

**Locomotor destination lifetime (claim 1).** Use S-0207 D29's water-lane controls
at speed index 2 with a stock ship, a remote bystander computer player, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted); repeat each control three times. Record order destination, locomotor destination,
heading point, world position, search outcome, track completion and arrival report.
Include reachable and temporarily blocked goals, a retained-goal retry, an intermediate
completion, entry into the goal cell, final completion, mid-track stop, replacement and
same-cell assignment before subsequent behaviour processing. Expected: assignment before
search success, persistence through retained-goal waits and intermediate tracks, no clearing
at the cell boundary, clearing at qualifying completion before reporting, immediate clearing
on stop despite finishing the track, and a present point after same-cell assignment.
Any contrary point lifetime falsifies the corresponding rule. Use D29's give-up controls
and its named docking-displacement controls for the strict 207/208-lepton boundary.
if legal ship approaches cannot be established, that boundary control is inconclusive.
N1 and N2 alone do not isolate destination-point presence.

**Order-level cleanup (claim 1).** Use S-0207 D30's water-lane ship controls at
speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), three repetitions per case and an isolated
bystander computer player. Record destination kind and presence, locomotor point,
world point, active track, approach destinations, behaviour processing, searches and
arrival reports immediately before and after locomotion. Test an accepted same-cell
move, final arrival from several cells away, accepted same-cell building entry, and
verified resting *guard* states with a building goal at exact equality, a horizontal
displacement within the cell, and a height displacement. Include D30's verified
approach-destination fallback control. Expected: first no-track update removes a
cell goal without requiring search or due behaviour processing; final completion in
tick *t* clears only the locomotor point and the unchanged cell goal clears in tick
*t* + 1. Building cell equality alone clears neither, but exact three-coordinate
equality in resting guard triggers cleanup or fallback. A different removal phase,
cell-only building cleanup, or guard cleanup despite a displaced point falsifies
the corresponding rule. Record intervening entry/arrival effects separately; runs
without verified guard or fallback preconditions are inconclusive for those controls.

For destinations naming units, also use S-0207 D31's ship controls with separate
vehicle, infantry and aircraft goals, first stationary and then moving. Keep the
same speed, settings, repetition count and read-only recording above, and record the
named unit's position and movement state, supplied movement point, accepted replacement
instructions and any boarding exchange. Expected: no-track supplied-point cell equality
alone clears neither destination for any of these kinds. In verified resting *guard*,
only exact equality with the retained locomotor point triggers cleanup or the verified
approach fallback, even if the named unit moves elsewhere without a new instruction.
Cell-only cleanup, a result changed solely by the named unit's motion, or a guard
comparison against its new position instead of the retained point falsifies the rule.
Separate boarding, behaviour and replacement effects from locomotion cleanup. If stock
orders cannot establish a particular kind or guard combination, that control needs a
verified named-map setup and remains inconclusive; do not substitute a cell destination
or use memory-writing instrumentation.

**N3. Wake cadence (claims 2, 6).** *Setup:* one surface ship and one underwater ship moving on
open water; a bridge over part of the path. *Procedure:* run 200 ticks of straight movement and a
turn in place. *Observable:* the tick numbers at which wakes appear for each vessel (frame capture
per tick or the object list), and whether one appears while turning in place, under the bridge and
on the deck. *Expected:* surface ship: wakes exactly at tick numbers divisible by 8 while moving or
turning, none on the deck; underwater ship: none. *Falsified if:* any wake at a tick number not
divisible by 8, any missing eligible wake, any wake on a deck, or any wake from the underwater
ship. Wakes below a bridge remain eligible. If no stock ship reaches a deck, use named map test
data placing a ship-locomotor vehicle there; failure to reach a deck is inconclusive. Compare
stream states around wake creation with S-0109 claim 3's animation draws: an additional draw
falsifies claim 2's draw statement. Repeat surface travel with named map test data that leaves
no game-wide wake animation defined: no moving wake should appear, and any such wake falsifies
the missing-animation guard. This variant does not test sinking. Also record 32 ticks with the
surface ship stationary and not turning: any wake from it falsifies the moving-now gate.
For the water-land gate, use a named map override permitting the same ship-locomotor type to
travel across a non-water strip. Record 32 moving ticks on that strip: no wake is expected,
and any moving wake there falsifies claim 2. An impassable strip makes that variant inconclusive.

**N3a. Wake speed and within-turn timing (claim 2; Q-0600).** *Setup:* the N3 water
map, a surface ship initially at rest, already facing its straight travel direction. Use named
map test data, "naval-wake-speed", with acceleration enabled, a low acceleration factor and a positive type speed so
that *ρ* > 0 while *s* = 0 lasts at least 8 ticks; keep the ship locomotor and define a wake
animation. Remove the override after the run. *Procedure:* order a long straight move, then
repeat with an unmodified stock ship; record through acceleration and final arrival, repeating
from each of the eight start-tick residues modulo 8. *Observable:* per-tick speed fraction,
computed *s*, progress budget, track target, turning state, pre/post-movement position and wake
creation position, using the read-only recorder. *Expected:* no wake with no body turn and
*s* = 0, including positive-fraction ticks; eligible positive-rate ticks can create a wake even
when the budget buys no track point. Each created wake uses the post-movement position and
eligibility. An arrival with no remaining heading point and no body turn creates none.
*Falsified if:* a wake appears solely because the fraction is positive, is missing solely because
position did not change despite positive rate and the other gates holding, uses the pre-movement
position, or uses a track target already cleared by arrival. A run without the intended zero-rate
interval or boundary tick is inconclusive for that sub-test.
For draw ordering, additionally record a turn or track transition that takes identified movement
draws under S-0207, on an eligible wake tick, and use a named test animation with distinguishable
creation draws under S-0109 claim 3. Compare the movement outcome and animation choices with the
raw stream values. Movement receives the earlier values and wake creation the later ones.
Reversed assignments falsify the claimed order. A tick with no movement draws does not test
that part. This is a recorder proposal, not a measurement already made.

**N4. Launching hold (claim 3).** *Setup:* a stock missile-launching ship with its slots docked
and an enemy target in range. *Procedure:* order an attack, then issue move input during
preparation and again after salvo release while a slot is on the rail. *Observable:* accepted
destination, launching count, slot states, release tick and the tick the ship begins its next
track, using the read-only object recorder. *Expected:* an accepted destination waits while the
count is positive, including the rail interval after release; with other blockers absent, travel
resumes once the count is zero. *Falsified if:* a new track starts with a positive count or the
ship never resumes with that count zero. A move order refused by S-0506 claim 10's movable test
does not test an accepted destination's hold.
Repeat with a stock carrier: ordinary non-missile preparation and release alone must not prevent
movement. For the active-track sub-test, use a named map override giving the carrier's ordinary
spawned aircraft the missile-spawn flag without making it a rocket type; its placement can then
raise the launching count while the carrier is travelling. The existing track must finish, and
no next track may begin while that count is positive. A mid-track pause caused solely by the
count or a next track begun with a positive count falsifies claim 3. This override tests the
between-track distinction, not stock carrier flight. Restore stock data after the run.

**N5. Sinking duration and wakes (claim 4, F1, F2).** *Setup:* a stock non-crashable surface vessel
without death image frames that meets the sinking condition and is retained under S-0312
claim 15, at height 0 on open water; a weapon that kills it in one shot. *Procedure:* kill
it; run the random-state recorder. *Observable:* the tick of the killing hit, the tick it leaves
play (it disappears from the object list), the wakes created and their positions relative to the
vessel, and the number of scenario-stream draws per tick. *Expected:* it leaves play at its 81st
descent; wakes only on tick numbers divisible by 4, each within 170 leptons of the vessel on each axis,
exactly two extra draws on each such tick, the first fixing the *y* displacement. *Falsified if:* the
duration differs, a wake falls outside the box, a draw predicts the wrong axis or position, or
the draw count differs. Also record height each tick: descent other than 5 leptons, removal at
−400 rather than below it, other than 20 wakes, or a first descent not in the vessel's first
update after the hit falsifies claim 4. Exclude animation-owned draws from the two displacement
draws. Repeat from a squid cull using S-0614 claim 11's setup and scenario P6: once sinking
starts, the same descent, wake and removal rules apply, with the health unchanged at the cull.
Record the sinking-start tick and first descent rather than assuming a killing-hit tick.
Different descent or wake rules after culling falsify claim 5's shared aftermath.
For the first-hit crashable comparison, use S-0312 scenario D8's first-hit crashable variant:
removal during that hit, with no subsequent descent or sinking-wake draws, is expected rather
than 81 descents. Retention for descent or later sinking wakes falsifies that removal rule.
The no-animation rule is a design choice, not a stock measurement.
For passenger cleanup, repeat with a loaded non-crashable vessel, using named test-map data
"naval-sinking-passengers" to supply transport capacity if stock types
cannot reach that setup. Use a killing hit or squid cull that leaves passengers aboard during
descent. Record passenger identities, in-play status, kill/loss counts and independent death
effects through final removal, distinguishing any earlier passenger deaths or escapes.
Expected: every passenger still aboard at final removal dies
without attacker credit or independent death effects; earlier credited deaths stay credited.
A surviving carried unit, release, extra attacker kill or independent passenger death effect
at final removal falsifies that outcome. A run with no passengers left before final removal
is inconclusive for cleanup. S-0507 scenario T14 supplies the shared transport comparison.
Remove the named test data after the run.

**N6. A sinking vessel cannot fire or be fired on (claim 5).** *Setup:* a sinking vessel with an
enemy in range and a second enemy already targeting it before the killing hit. *Procedure:*
kill it, observe through removal, and try to retarget it by scripted input. *Observable:* new shots
from or at it, successful target reassignment and horizontal position. *Expected:* no new
shots pass either sinking test, a new assignment leaves no target by S-0309 claim 24, and
horizontal position stays unchanged. Already airborne projectiles are not new firing attempts. *Falsified
if:* a new shot passes either sinking test, a successful assignment retains the sinking target,
or it travels horizontally. If the click produces no order, that attempt does not test target
assignment; use S-0309 claim 24's guard-search variant instead. This does not test every competing firing answer or every occupancy use.

**N7. Naval match (claim 7).** *Setup:* a land repair depot and a naval yard with the repair flag,
a damaged drive vehicle, a damaged ship and a damaged amphibious hovering craft. *Procedure:* order
each to dock at each building. *Observable:* whether the vehicle docks and is repaired. *Expected:*
drive vehicle at the depot only; ship at the yard only; hovering craft at both. *Falsified if:* any
other pairing docks or a listed pairing is refused.

**N8. Starting distance (claim 8).** *Setup:* a damaged ship at a yard; a damaged amphibious hovering
craft and a damaged drive vehicle at a land depot. *Procedure:* dock each; read positions per tick.
*Observable:* the first tick at which the vehicle stands still within 200 (ship, hovering craft) or
100 leptons (drive vehicle) of its docking point (F3), the waiting-to-settling tick from a read-only recorder, and first repair
(health and credits change). Give its owner ample credits. *Expected:* the depot enters its settling processing
3 ticks after the successful waiting gate. If contact persists, the vehicle remains still and
its locomotor has finished settling as required by S-0314 claim 6, the first repair occurs then.
Otherwise record the later repair separately; the 3-tick transition alone does not guarantee
repair. The settling phase's 200-lepton check must not be treated as a second repair-distance
gate. Unsuccessful waiting processings recur at *D*.
*Falsified if:* the gate accepts a
drive vehicle at 100 leptons or more, refuses a still ship from 100 inclusive to 200 exclusive,
accepts exactly 200, the gate-to-settling interval differs from 3 ticks, the first paid repair
step does not occur at that settling processing despite the stated conditions, or unsuccessful
waiting processings recur at an interval other than *D*. For strict boundaries
and height, use named map test data with docking-point displacements giving rest distances just
below, exactly at and just above each threshold, and repeat with a height difference. A result
inconsistent with F3 falsifies spatial distance. Failure to establish a still docked vehicle at
the intended distance makes that sub-test inconclusive.

**N9. Underwater type without cloak ability (claim 6).** *Setup:* named test-map override
removing only the cloak ability from a stock underwater ship; leave its underwater flag, ship
locomotor and other fields unchanged. Use open water, an opposing naval firer whose two weapons
distinguish surfaced and submerged underwater targets under S-0308 claim 11, and a second
ordinary surface ship as a sinking control. *Procedure:* leave the test ship idle beyond its
normal cloak delay, move it for 32 ticks, then attack it to destruction. Repeat the killing-hit
test on the control. *Observable:* cloak state, selected weapon, moving wakes and whether each
vessel enters sinking. *Expected:* the test ship stays uncloaked and surfaced, receives the
surfaced-underwater weapon choice, makes no moving wakes and does not enter ordinary killing-hit
sinking. The control sinks when S-0312 claim 10's condition holds. *Falsified if:* the no-cloak
ship submerges, gets the submerged weapon choice, creates a moving wake or enters sinking from
that killing hit. If the weapons do not distinguish the two states, weapon selection is
inconclusive; choose a named weapon-selection test override from S-0308 claim 11's scenario.
Remove the named override after this sub-test. This does not test forced cloak effects or culls.

**N10. Effects during descent (claims 5 and 10).** *Setup:* separate runs on the same small
water map, with a stock surface vessel eligible for killing-hit sinking. For (a), put a second
stock firer on a sufficiently long flight-time shot, and kill the vessel with the first firer
before that shot arrives. For (b), use a second explosion with area covering the vessel after
sinking begins. For (c), use an elite stock vessel with a self-healing ability, and separately
an owner benefiting from a captured stock vehicle-healing source. Record each healing eligibility
and next due tick before sinking. For (d), order a turn and kill the vessel part-way through it.
*Procedure:* run each through removal with tick-labelled recording. Time a due healing tick to
fall before the 81st descent. A squid-cull variant can begin descent above 1 health to expose
nonlethal damage independently of later killing-hit effects. *Observable:* delivered impact,
health before/after every damage or healing event, world position/height, current and desired body
facing, and removal tick. Also record repeated death effects for S-0312 rather than presuming
immunity. *Expected:* (a,b) ordinary damage eligibility/arithmetic, not automatic unaffected due
to sinking, (c) ordinary healing up to its ordinary cap, with sinking/descent continuing, (d)
body facing progresses towards its saved desired facing under S-0206's schedule. *Falsified if:*
an otherwise eligible delivered damage or due heal is suppressed solely by sinking, healing
stops descent, or sinking freezes an already progressing body-facing update. An impact that
misses, ineligible healing, no due healing tick before removal or a turn already complete is
inconclusive. Radiation can be probed on stock water beside an irradiated shore only if the
recorder confirms an applicable radiation exposure before removal; absence of such exposure does
not test that claim. This requires read-only health/facing/event recording under the measurement setup,
not modification of the oracle. No effect is claimed measured yet.

**N11. Ship search cadence (claim 9).** *Setup:* a winding open-water route exceeding 24 steps,
a stock ship, and separate runs with an allied moving ship temporarily blocking its next step.
Use the same stock path-delay field for a driving vehicle on a land control route and a hovering
craft on a water control route. *Procedure:* issue a move, let the blocker cross and then clear
the route. Record search ticks, stored steps, destination, retry-timer expiry and blockage
countdown through several searches. *Observable:* delay after a successful ordinary fresh search,
last-stored-step refresh with heading point over 512 leptons away, moving-blocker search mode and
timer restart. *Expected:* ship/drive successful ordinary searches clear retry delay, hover fresh
searches restart it. Ship/drive last-step refresh does not wait for expiry. Moving-blocker re-path
waits for the retry timer and restarts the same *R*, independent of success; search mode follows
S-0212 claim 22's blockage countdown. *Falsified if:* the ship defers the last-step refresh until
its stored steps are exhausted, retains a fresh-search delay on success like hover, uses a
ship-specific *R*, or omits the moving-blocker restart. Add a cloaked stock underwater blocker
only when the recorder confirms the cloaked-blocker entry outcome, and a stationary friendly
blocker as a control: with a nonzero retry timer the former waits to search and the latter
searches immediately. Opposite timing falsifies those branches. Failure to establish that
outcome or a nonzero timer is inconclusive. A stored-path change alone is insufficient to prove
the retained-old-step quirk when old and new first steps agree; that subclaim remains unmeasured.
The refresh distance and chase-cut distance use claim 9's spatial table root. This setup tests
refresh timing well beyond 512 leptons, not the exact boundary or table-root distinction.
Those remain unmeasured without a qualifying lepton-scale separation.
Read-only search/timer recording is a proposed recorder capability, not assumed present.

**N12. Absent weight and stock death (claim 11).** *Setup:* a stock naval surface vehicle whose
type omits its weight, on water, with a stock enemy able to kill it. Inspect its imported weight
and the imported sinking threshold in a read-only recorder; do not alter either. *Procedure:*
kill it and record health, sinking and removal. *Expected:* weight is 1.0, and sinking occurs
only if the imported threshold is at most 1.0 and S-0312 claim 10's other conditions hold.
*Falsified if:* a newly imported absent-weight type has another weight, or its otherwise eligible
killing hit contradicts that comparison. A recorder that exposes only death, not imported weight,
checks the stock outcome but cannot distinguish candidate absent defaults that fall on the same
side of the threshold. Stock-only observations do not isolate the exact threshold boundary.

**N13. Naval-yard point and repair start (claims 8 and 12).** *Setup:* each faction's stock naval
yard in separate water areas, ample power/credits, one damaged stock compatible ship each.
*Procedure:* click the yard with the ship selected and record its approach, contact, position,
building processing phase, docking point, health and credits per tick. Repeat from each accessible
side, then with a second damaged ship sent while the first is docked, and at full health.
*Observable:* supplied docking point relative to the top-left foundation cell and yard height,
foundation cells traversed, the successful waiting processing, settling and first paid repair.
*Expected:* the supplied point follows claim 12's foundation-centre-plus-displacement
selection for the assigned contact place, including height. Approach side and release/rally
location do not change it. On a level ordinary track, the ship stops at the docking-point
cell's centre, even when the supplied point is off-centre. Waiting/settling/repair timings
follow claim 8's distance to the supplied point, not immediate payment on contact or click.
*Falsified if:* an ordinary naval yard selects the fixed (2, 1) point solely because it is a
vehicle factory, the point varies by approach rather than contact place, the ship relocates
horizontally to the off-centre supplied point, or payment precedes the stated gates.
If stock data supplies no off-centre point, that stopping distinction is inconclusive; use
S-0314 H19 b's proposed recorder test rather than assuming exact alignment.
Only use ordinary playable placements; an unenterable approach is not a distance-test failure.
Record the second-ship, full-health and cancellation outcomes without assigning an untraced
expected handshake: they are proposals to finish Q-0641, not completed claims. The missing
contact/event recorder capability should be proposed to the user, not installed or patched here.

**N14. Killing hit during an existing descent (claim 13; Q-0646).** *Setup:* N10's water map
and recorders, a stock surface vessel meeting the ordinary sinking condition, and two firers
whose impacts can be separated in time. *Procedure:* start sinking with one killing hit, then
let a previously fired slow shot or an applicable area hit arrive after at least ten descents.
Repeat with a cull-started surface host meeting the same ordinary killing-hit conditions, and
with healing timed between hits. One run per variant, plus a no-further-hit control.
*Observable:* health and height immediately before and after the later hit, descent count,
removal tick, kill counts, destruction raises, passengers, death effects and random draws.
*Expected:* an otherwise ordinary later killing hit leaves 1 health; a nonlethal hit gives
S-0302 claim 11's health result. Neither resets height or extends the descent. A cull's first
killing hit uses the same ordinary sinking eligibility rather than bypassing weight or type.
*Falsified if:* an eligible ordinary later killing hit leaves another health value, resets
height, or extends removal without a separately recorded height-changing effect. A miss or
filtered area candidate is inconclusive. Use types without death image frames and with
crashability off for these descent comparisons. Credit, raises, death effects and ineligible
cull-host outcomes follow S-0312 claim 32 and its scenario D23, not suppression merely because
sinking is already active. Death-image-frame variants remain exploratory, without a predicted
health or removal outcome.
If any run leaves a zero-health vessel in play, record through the next due healing ticks:
no individual or house-wide self-heal should raise it, and an ordinary delivered healing hit
must not revive it. Absence of such a state does not test that conditional exclusion.

**N15. Naval-yard interruptions (claim 14).** *Setup:* N13's stock compatible damaged ships
and yard with ample credits, a rally point, and a second ship. Use separate runs for approaching,
waiting, settling and working, with the read-only recorder distinguishing contact and tether.
*Procedure:* issue move, stop, attack and same-yard enter orders at each phase, one run per
order/phase pair. In separate runs sell or destroy the yard during approach and after docking,
switch it off or cause low power during repair, and send two damaged vessels in the same input
tick. Repeat the latter with the later vessel at full health and with an ordinary later click.
*Observable:* event order, destination, behaviour, contact place, tether, position, propulsion
state, health, credits and processing ticks. *Expected:* S-0314 claims 20 to 23's outcomes with
claim 8's ship threshold and claim 12's docking point. In particular approach contact releases
under another order, docked working contact permits paid repair after standing still away,
sale and destruction differ in their notice and behaviour, existing service survives power
changes, a simultaneous damaged later entrant queues but a full-health one does not, and a
permission-refused later click creates no queue. *Falsified if:* a recorded eligible exchange
violates any of these shared outcomes. A refused click does not test an executed enter order.
After a post-settling yard loss, try moving the ship in several directions and record whether
propulsion remains off; this last probe is exploratory for the stranding question below,
not a prediction that ships share the hovering failure. Record events read-only under the
measurement setup.

**N16. The lean of a sinking vessel (claims 15 and 16; Q-1871).** *Setup:* N5's map and its stock
non-crashable surface vessel with the ship locomotor, drawn as a voxel model, at height 0 and at
rest. Use eight runs, one per 8-direction, with the body facing set exactly to that step (0, 8192,
…, 57344 facing units). The killer is a firer whose weapon kills the vessel in one shot and whose
warhead has no rocking flag. It is placed so that the shot does not turn the vessel. *Procedure:*
kill the vessel. Capture one paused frame per tick, with the tick number, from the killing hit until
it leaves play. Run the random-state recorder over the same ticks. If the object recorder can read
the body's pitch and roll, record them every tick too. The same scenario, run in our engine, checks
the fixed drawing of claim 15; the runs on the original record its three oddities (parts a, b
and c, and the recorded pitch).

*Observables:*
- Which end of the hull goes down.
- Whether the lean grows by equal amounts per tick.
- The descent number at which the lean stops growing, counted with N5's height record.
- The draws taken during the descent.
- Whether any shadow is drawn.
- Whether the sideways attitude changes.
- The drawn body's outline at full lean, at headings 0 and 2, against a render of the same model
  under F5 and against one without F5's lift and shift.

*Expected,* in both the original and our engine:
- The bow goes down at 8-directions 1 to 5 and the stern at 0, 6 and 7.
- The lean grows and stops at about 45 degrees, at or within a descent or two of the 79th
  descent (two descents before the vessel leaves play). Frames cannot tell a stop at the 78th,
  79th or 80th descent apart: the 78th step gives 0.78 radians against a limit of about 0.7854.
  Only a recorded pitch fixes the descent (below).
- The same result in a second run with another seed at each heading.
- No draws other than claim 4's wake draws, counted as N5 counts them (animation-owned draws
  excluded).
- No shadow from the first frame after the hit.
- No sideways change.
- An outline that matches the render under F5.

A recorded pitch follows F4 exactly (up to the precision the recorder shows): in our engine it ends
at π/4 to single precision; the original's run records its final value, expected to be about 0.79
radians.

The primary falsifier is the end that goes down at headings north (0) and east (2): a stern-down
east run or a bow-down north run reverses claim 15's direction groups or claim 16's drawn sense.
Also falsifying:
- A sign that differs between seeds (a random direction).
- Growth that stops more than two descents from the 79th, or none.
- A draw during the descent beyond the wake draws, counted as in N5.
- A drawn shadow.
- A sideways roll that develops.
- An outline that matches the render without F5's lift and shift rather than the one under F5.

*Further parts:*
- **a. Continuing turn.** Order the vessel to turn from south-west (5) towards north (0) through
  west. Kill it while its body facing is still in the south-west 8-direction, so that the turn
  continues during descent (claim 10). Expected in our engine: the bow goes down and keeps going
  down after the facing reaches the west 8-direction (6); a lean that shrinks back falsifies the
  kept direction. The original's run records the reversal, expected as the lean shrinking back
  through level once the facing reaches the west 8-direction, then the stern going down.
- **b. Rolled hull.** Just before the killing shot, rock the vessel sideways with a stock weapon
  whose warhead has the rocking flag, fired from abeam (S-0303 claim 20). Run the same rocking
  shot once on a vessel that is not killed, as a control. Expected in our engine: the roll is not
  held, and over the descent it changes as the control's roll does over the same ticks, while
  the pitch grows. Whether either returns to level is `unmeasured` (claim 15) and is recorded
  only. A roll that stays as it was at the hit while the control's roll changes falsifies it. The
  original's run records its held roll, expected to stay as it was at the hit through the whole
  descent. Its frames at full lean also test F5's roll terms: compare the outline against renders
  under F5 with the roll terms, without them, and with the sign of *s*ᵧ reversed, at the recorded
  roll or, without a recorder, at the roll that best fits the rotation of the outline. A better
  match without the roll terms, or with the reversed sign, falsifies them.
- **c. Rocking during descent.** During the descent, detonate a rocking warhead beside the
  vessel, abeam. Expected in our engine: the hull rolls as S-0303 claim 20 sets it going (any
  return to level is recorded only, as in part b), and the lean goes on growing
  as in the main run; a change to the lean falsifies the pitch's ownership by claim 15. The
  original's run records the drawn body. The rocking hit is expected to add no change of its own
  to the body's angles: no roll develops, and the lean goes on with claim 15's pitch steps as in
  the main run. This shows the held angles; it cannot show the held rates (claim 15).
- **d. Start below height 0.** Not settable with stock data, so it is left unmeasured (claim 15,
  timing).
- **e. Squid cull.** Repeat once from a squid cull (S-0614 scenario P6's setup) at headings 0 and 2.
  Expected: the same lean, with the hull level sideways throughout, because the squid's tilt is
  0 at the strike and the squid leaves at the cull (S-0614 claims 10 and 11).
- **f. Magnetic-lift drop.** Repeat once from a magnetic-lift drop into water (S-0607 scenario L6's
  water case) with a stock voxel vehicle that has the driving locomotor. Expected: the same lean and
  no shadow. A level body is recorded as evidence for the open question on locomotors attached for a
  lift, not as a falsifier.
- **g. Passengers of an open-topped transport.** A test rules file named *n16g-rules*, supplied
  through the file system for this part and removed after it, adds a test vessel type named
  *N16TRAN*: a copy of a stock voxel ship that a squid can grapple and cull, with the ship
  locomotor, its voxel model, the open-topped flag and room for one passenger, and no other
  change. All other data are stock. It carries one stock
  infantry passenger with a weapon, at heading 2. An enemy target stands on a shore within the
  passenger's range. Sink the vessel by a squid cull as in part e. Record each of the passenger's
  shots during the descent and its launch position from the object recorder. Expected: the
  passenger keeps firing during the descent, and each launch position matches S-0313 claims 1 and 5
  computed with the transport's body transform under F5 at that tick's pitch, with F5's lift and
  shifts read as leptons. Falsified if the passenger stops firing when sinking begins, or if the
  launch positions stay those of a level body.
- **h. Start from a rocked pitch.** As part b, but fire the rocking weapon from ahead, at heading
  2, so that the hull is pitched at the killing hit. Expected: the lean starts from that pitch, not
  from level, and reaches its full lean as F4 gives for that starting pitch. A hull that jumps to
  level at the hit falsifies the starting value.

Use tick-labelled frame capture at the measurement speed, and record read-only.

## Changes for other specs

- **S-0210, the drawn direction of the body tilt.** The body tilt is added to the pitch of this
  spec's Terms with its own sign, and a positive pitch lowers the bow under S-0105's rotation
  convention. So a negative body tilt raises the bow and a positive one lowers it. S-0210 uses the
  opposite words in these places:
  - Terms, *Body tilt*: "rocks forward (negative) or back".
  - Terms, *Tipping forward*, and the uses of "tips forward" in claims 6, 8 and 9 and in its
    Constants rows.
  - Claim 6, and its scenario K8 part c: "pitches back, not forward", where the overlay's
    positive change in fact lowers the bow.
  - Its scenarios' forward and backward pitch observables.

  Observable: a crusher that tips under S-0210 claims 6, 8 or 9 is drawn with its bow rising, and
  a crush of an overlay alone lowers it. Whether S-0210 swaps the words or renames the terms is
  S-0210's choice; the values and signs it gives stay as they are.
  - Backing: `measurable` (S-0210 scenario K8, parts a and c, read as which end of the body
    rises; and this spec's N16 for the sense of a positive pitch)
  - Confidence: medium
- **S-0106 claim 9, and S-0210 Terms, *Body tilt*: where the rocking composition is.** S-0106
  claim 9 leaves the composition of pitch, roll and tilt to the vehicle specs. This spec's F5 is
  that composition for the driving and ship locomotors, for any rocking, not only for sinking.
  A pointer to F5 in both places lets a reader of the land-vehicle rules find it. F5 stays in
  this spec.

## Open questions

- How a rocking body returns to level. Claim 15 adds no rule of its own for a sinking vessel's
  roll. S-0303 claim 20 and S-0614 claim 10 set rocking going, S-0210 claim 12 limits it while
  slowed and S-0106 claim 9 draws it, but the rate at which a rocked body comes back to level
  belongs to a rendering or vehicle spec that has not stated it. S-0303 claim 20 gives only the
  strength that sets the rocking going. Until a spec states it, the roll's return to level during descent
  is `unmeasured` here. The gap reaches gameplay through claim 16: the passengers of an
  open-topped transport fire from a fire point built on the rolled body transform, so their
  launch positions during descent depend on how the roll evolves. The spec that owns the return
  to level should state it with that consequence in view.
- Q-0641 is partly answered here: claims 8 and 12 specify the naval yard's waiting gate,
  docking point and cell-based stopping, with S-0314 claim 18 owning the shared rule. Full approach handshake, contact reservation timing and failure/retry routing are not
  established here. Do not infer them from aircraft pads, land depots or production release.
  Q-0641 is therefore not included in the answered-request list.
- Q-0646: the death-image-frame path remains unresolved in S-0312 claim 32. That claim
  and scenario D23 now cover repeated destruction credit, raises, applicable death effects
  and ordinary health/removal outcomes; claim 13 here owns continued descent, not that
  unresolved path. S-0302 claim 8 owns hit filtering at 0 health.
- Whether a ship whose propulsion was switched off during settling becomes permanently
  immobile after its naval yard is sold or destroyed. S-0314 claim 22 establishes this for
  hovering vehicles, not ships. Claim 14 does not extend `quirks.depot_loss_strands_hover`
  to ships without establishing that result; N15 includes a separate probe.
- Whether concurrent launch and other movement waits produce an observable difference between
  ship and driving locomotors. This interaction is `unmeasured`: no setup or expected observable
  has been established. Claim 3 fixes only the ordinary launching-count hold, not precedence
  over unrelated cancellations or delays.
- Whether a sinking vessel keeps its cell's occupancy until it leaves play in every respect (claim 5).
- Q-1871, open details, outside claims 15 and 16:
  - **Composition for other locomotors.** F5 is stated for the driving and ship
    locomotors only. For the hovering and jumpjet locomotors, the order in which roll and pitch
    are composed, and whether they add the same lift and shift, are not established here; no
    scenario sets up such a body with both a pitch and a roll.
  - **Locomotors attached for a lift or a teleport.** Whether the lean shows when the sinking
    body is drawn by such a locomotor (a magnetic-lift drop, a teleport arrival on water).
  - **Clipping.** Whether the original cuts off a sinking vehicle's image below a screen row
    fixed at its first drawn frame after sinking begins (the "going under" look), and if so,
    which row.
- The driving locomotor keeps a few tracks beyond S-0207's tables (for example for a factory exit)
  that the ship locomotor does not have. It is believed that no ship ever selects them, since ships
  leave a yard by S-0417 claim 9; not exhaustively checked.
