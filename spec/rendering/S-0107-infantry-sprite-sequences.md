---
id: S-0107
title: Infantry sprite sequences
topic: rendering
revision: 10
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0087, Q-0212]
supersedes: []
---

# S-0107: Infantry sprite sequences

## Summary

Skirmish only; network games and campaigns are out of scope. How a soldier's image frame and shadow are chosen from its type's sequence table, its facing and a
stage counter, and which sequence kind plays at any moment (movement, firing, fear, water, idling,
dying). The fear rules and the attack state (claims 11, 15 and 16) are infantry behaviour kept here
for now because they choose sequences; they are expected to move to a future infantry-behaviour
spec. S-0315 lists every trigger of fear and gives its effects in combat. The infantry scatter from damage has already moved: which scatter requests a hit sends, and
whether an infantry unit acts on them, are S-0208's (claims 15 and 11). This spec keeps the
scatters of its idle actions and of its flee rule (claims 14 and 15).

## Terms

Terms from the glossary are used as defined there, in particular *tick*, *facing*, *game speed*
(index *s*), *sprite*, *image frame*, *drawn frame*, *lepton*, *height level*, *cell*, *locomotor*,
*scenario random stream*, *unsynchronised random stream*, *scatter*, *jumpjet*, *movement zone*,
*disguise*, *human-controlled*, *local player*, *warhead*, *measurement setup*, *recorder* and
*oracle*. Terms
used by this spec only:

- **Body sprite** (glossary *sprite*): the infantry type's body sprite holds *L* image frames. The
  first ⌊L/2⌋ image frames are body image frames, the second half are their shadows in the same
  order.
- **Sequence kind:** one of 42 named roles a run of image frames can play. Our names, with the
  import key of each in the Import mapping: *stand*, guard, *prone*, *walk*, *fire-standing*,
  *go-prone*, *crawl*, *get-up*, *fire-prone*, *idle-a*, *idle-b*, *die-1* to *die-5*, *tread*,
  *swim*, *wet-idle-a*, *wet-idle-b*, *wet-die-1*, *wet-die-2*, *wet-fire*, *hover*, *fly*,
  *tumble*, *fire-airborne*, *deploy*, *deployed*, *deployed-fire*, *deployed-idle*, *undeploy*,
  *cheer*, *parachute*, *air-death-start*, *air-death-fall*, *air-death-end*, *panic*, *shovel*,
  *carry*, *secondary-fire*, *secondary-fire-prone*. The kind guard is written in roman, because the glossary reserves
  italics for the behaviour of that name. The **deploy kinds** are *deploy*, *deployed*,
  *deployed-fire* and *deployed-idle*. A unit is **deploying** while its current sequence is one of
  them.
- **Sequence entry** (per type, per kind): start image frame *S*, frame count *C*, facing multiplier *M*
  (all integers), an optional **declared direction** *D* (one of the eight compass points), and up
  to two **sound cues**, each a pair (cue frame, sound).
- **Current sequence:** the kind a unit is playing, or *none*.
- **Stage** *k*: a non-negative integer counter owned by the unit. It restarts at 0 whenever a new
  sequence starts and grows by one every *d* ticks, where *d* is the **stage delay** of the current
  sequence (claims 4 and 5).
- **Interruptible:** an engine attribute of each kind (table T1). A playing kind that is not
  interruptible can only be replaced by a forced start.
- **Forced start:** a start request that ignores the interruptible attribute. The text says which
  requests are forced.
- **Fear level:** an integer from 0 to 300 per infantry unit that rises when it is hurt (and in the
  other cases S-0315 claim 1 lists) and decays over time (claims 15 and 16).
- **Prone state:** a yes/no state of each unit meaning "lying down". It begins when *go-prone*
  starts and ends when *get-up* or *deploy* starts.
- **Land type:** the terrain category of a cell (clear, road, water, beach, rock and so on), which
  the map and terrain specs define. This spec only asks whether it is *water* or *beach*.
- **Wet cell:** a cell whose land type is water or beach.
- **Bridge cell, on a bridge deck, bridge deck height:** a *bridge cell* is a cell that carries a
  bridge. A unit is *on a bridge deck* (glossary *bridge*) when it stands or moves on the bridge's
  deck rather than on the ground or water below it; which units are on a bridge deck belongs to the
  movement specs. The *bridge deck height* is the engine's fixed height of a high bridge's deck
  above the terrain of its cell: 4 height levels, 416 leptons, an engine constant. S-0102 gives it
  (Behaviour 15 and Constants, *bridge deck height*), and S-0104 uses the same value (Terms,
  *bridge deck*).
- **Height above the surface** (glossary): the unit's world height less the height of the terrain
  of its cell, and also less the bridge deck height when the unit is on a bridge deck. A unit
  standing on a bridge deck therefore has a height above the surface of 0. In leptons, an integer
  that is negative when the unit is below the terrain level of its cell.
- **Airborne:** the unit is on the map and its height above the surface is at least 2 height
  levels (208 leptons).
- **Falling:** a parachuting unit that has not yet landed.
- **Speed fraction:** the unit's current speed divided by its full speed, a value from 0 to 1 that
  the original holds as a double (53-bit significand). How it is set belongs to the movement specs.
  This spec only compares it with the constants 0.8 and 0.1. Each of these is the double nearest to
  the decimal, and the comparison is an exact comparison of doubles.
- **Has movement in progress / is moving now:** two different answers of the unit's locomotor. The
  first is true while it still has movement to carry out. It is used by the completion follow-up,
  the fear rules and the idle checks. The second is true while the unit is physically moving on
  this tick. It is used by the movement rule of claim 10. The two can differ briefly when a unit
  starts or stops, or while it waits during a move.
- **Attack state:** a yes/no state of each unit meaning "an attack is in progress". Claim 11 says
  when it begins (also when the firing request is turned down) and when it ends.
- **Secondary weapon:** each infantry type has a primary and may have a secondary weapon. Which of
  the two a unit uses against a target belongs to S-0308. Here "firing its secondary
  weapon" means that the weapon chosen for the current target is the secondary one.
- **Behaviour processing that attempts the idle action:** a unit's current behaviour is
  processed from time to time, and in some cases the processing attempts the unit's idle action.
  S-0215 owns the behaviours and says in which cases their processing attempts it:
  - *guard* and *sticky* (S-0215 claims 24 and 25), when the unit had no target at the start of
    the processing and no charge target was made.
  - *area guard* (S-0215 claim 26), when the leash and search table leaves the unit with nothing to
    search or engage, or its search found nothing.
  - *hunt* (S-0215 claim 27), when the unit has no target after the search, or its type has the
    stupid-hunt flag. A hunting unit that finds nothing in a skirmish therefore idles in place.

  In each case a deployer rule that S-0215 gives for the behaviour (S-0215 claim 31), and the rules it
  gives only for the original's computer player, take precedence and make no attempt. No other
  behaviour attempts it.
  Claim 14 only says what an attempt does. The *guard* behaviour is not to be confused with the
  sequence kind guard, which no stock behaviour starts (claim 20).
- **Selected:** the unit is in the local player's current selection (picked with the mouse or a
  hotkey). Here it only matters for the idle comment of claim 14, which a selected unit does not
  make.
- **Crushable:** whether vehicles that crush can kill the unit by driving over it. Crushing belongs
  to S-0210. This spec only says when deploying and undeploying change it.
- **Veterancy ability:** a property a unit gains on reaching a veterancy rank, as listed for its
  type in the rules data (S-0311, claims 3 to 5, owns which abilities exist and when they are gained).
  The **fearless veterancy ability** is one of them.
- **Fraidycat:** an infantry type flag (Import mapping) for civilians who run away instead of
  lying down: it never asks for *go-prone* and scatters when frightened.
- **Cyborg:** an infantry type flag (Import mapping). A prone cyborg, and any cyborg killed by
  damage that ignores defences, dies without a death sequence (claim 17), and a prone cyborg whose owner is human-controlled makes no destination-driven get-up request (claim 15).
- **Fearless:** an infantry type flag (Import mapping). Its units never lose fear over time, and
  damage raises their fear only in the fraidycat case of claim 16. A unit with the fearless
  veterancy ability is treated the same way when damage raises fear, but its fear still decays.
  So a fearless unit that is not a fraidycat never gains fear, while a fraidycat that is fearless,
  by its type or by the ability, still has its fear set to 300 by a known attacker while its fear
  level is below 100 (claim 16, `quirks.fraidycat_fear_ignores_fearless`).
- **Known attacker:** damage has a known attacker when it comes with the object that caused it (a
  unit or a structure), so that the hit unit knows where the attack came from. Damage that comes
  with no such object has no known attacker, whatever caused it. Which damage sources carry their
  attacking object belongs to S-0302 (Terms, claim 1). Here a known attacker decides which rows of the
  fear table of claim 16 apply. Which scatter requests a hit sends, with or without a known
  attacker, belongs to S-0208 (claim 15).
- **Not human:** an infantry type flag (Import mapping) for animals and robots. It changes the death
  sequence and the corpse (claim 17).
- **Infantry-death kind:** an integer field of a warhead (Import mapping) that says how an infantry
  unit killed by it dies. It is one input of claim 17, not the whole choice. Unless rows 1 to 3 of
  claim 17 apply (the water case, a prone cyborg or a jumpjet type, a type with its own death
  animations), which show no sequence whatever the kind, a type marked "not human" requests *die-1*
  whatever the kind (a request that claim 6 can still turn down, for example while the unit is a
  falling parachutist). In every other case, kinds 1 and 2 lead to death sequences, as does kind 9
  unless its replacement effect applies, and the other kinds spawn animations that other specs
  describe.
- **Damage that ignores defences:** damage delivered with a marker that tells the target to
  disregard its defences. Which damage carries the marker belongs to S-0302 (claims 1 and 9). This spec
  uses it only for the cyborg case of claim 17.
- **Crash flag:** a unit type flag (Import mapping) for types that crash rather than vanish when
  they die in the air. What a crash does belongs to S-0312 (claim 16) and the jumpjet locomotor's spec. Here it only
  decides, in claim 17, whether a killed unit that shows no death sequence is removed at once.
- **Enslaved to a master:** the unit belongs to a master unit that controls it, as harvesting
  slaves do for their master. The mechanics belong to the economy specs.
- **Storage capacity, fill fraction:** the unit's resource capacity field (Import mapping) and the
  amount it carries divided by that capacity, a floating-point value from 0 to 1. This spec only
  tests whether the fill fraction is exactly 1, which holds when the amount carried equals the
  capacity.
- **Harvesting:** the unit is carrying out a harvest order (economy specs).
- **Designating an airstrike:** the unit is marking a target for an airstrike that is linked to it
  (superweapon specs).
- **Laser-fence building:** a building whose type has the laser-fence flag (structures specs).

## Behaviour

1. **Every infantry type has one sequence entry per sequence kind.** Any field that the art data
   does not give (Import mapping) is *S* = 0, *C* = 0, *M* = 0, no declared direction and no sound
   cues. In particular, a kind that the data leaves out has *C* = 0. A direct request for such a
   kind is always turned down (claim 6), but the kind can still become current through a
   substitution (claim 7, `quirks.unchecked_substitute`). A type whose art gives no sequence table
   at all therefore never starts a sequence and, unless it is drawn through a disguise (claim 13),
   always shows image frame 0.
   - Backing: `measurable` (scenario M1)
   - Confidence: high

2. **The displayed body image frame** is
   *frame = S + (k mod max(C, 1)) + M × g(φ)* when *M* > 0, and *S + (k mod max(C, 1))* when
   *M* ≤ 0. Here *k* is the stage, φ the unit's current body facing, and *g* the facing group of
   claim 3 (Formulas F1, F2). With *M* > 0 the image frames of one facing group are consecutive and
   the groups lie *M* image frames apart. The entry used is that of the current sequence, or of
   *tread* when the current sequence is *none* and the unit stands on a water cell (beach does not
   count here, `quirks.tread_default_water_only`) and not on a bridge deck, or else of *stand*.
   - Backing: `measurable` (scenarios M2 and, for the *tread* fallback, M5(b))
   - Confidence: high

3. **Facing groups.** Each facing belongs to one of eight groups. The group depends only on which
   of 32 evenly spaced directions the facing is nearest to (Formulas F2). Group 7 holds map north, and the group number falls by one for each eighth of a
   turn clockwise. So group 5 is map east, 3 map south and 1 map west, and group 0 is map
   north-west. The art therefore stores its eight facings counter-clockwise, starting with map
   north-west. Each group covers four of the 32 directions (one eighth of a turn), but the sectors
   are not centred on the compass points. Group 7 covers the 32-way directions 30, 31, 0 and 1,
   which are facings 60416 to 65535 and 0 to 3071, so its centre lies 1024 facing units
   counter-clockwise of map north. The same skew holds for every group
   (`quirks.infantry_facing_sector_skew`). For example, a unit facing 3500 (about 19° clockwise of
   map north) shows group 6, not 7.
   On the screen, map north is towards the upper right and map north-west straight up, as the
   glossary's *facing* entry defines and its claim G1 states for the original.
   - Backing: `measurable` (scenario M2)
   - Confidence: high

4. **Stage timing.** When a sequence starts on tick *t₀*, its stage is 0 and its stage delay *d* is
   fixed for as long as that sequence stays current (claim 5 says how *d* is found). When *d* ≥ 1,
   the stage seen after the updates of tick *t* is ⌊(*t* − *t*ᵣ) / *d*⌋, where *t*ᵣ is *t₀* or,
   after an airstrike reset of claim 11, the tick of the latest reset. A stage delay of 0 freezes
   the stage at 0, so the sequence shows a single image frame per facing. The airstrike reset sets
   the stage back to 0 while the sequence stays current, and is the only change to the stage other
   than a start. It applies only to the two secondary-fire kinds, whose delay is 1 (T1), so after a
   reset on tick *t* the stage is 0 after the updates of tick *t*, 1 after those of tick *t* + 1,
   and it grows by one per tick from there.
   - Backing: `measurable` (scenarios M3, and M8(b) for the airstrike reset)
   - Confidence: high

5. **Stage delays.** Each kind has an engine base delay *b* (Constants, table T1). For *idle-a*,
   *idle-b*, *wet-idle-a*, *wet-idle-b*, *hover* and *cheer*, the stage delay is *d = N(b, s)*, the
   game-speed normalisation of Formulas F3. For every other kind *d = b*, whatever the game speed.
   *s* is the game speed at the tick the sequence starts. A change of game speed while a
   normalised kind plays does not change its delay. The next start of a normalised kind uses the
   new speed. A normalised kind therefore lasts about the same real time at every game speed, while
   the others last a fixed number of ticks. The switch is `quirks.speed_normalised_delays`, shared
   with S-0108 (glossary claim G3).
   - Backing: `measurable` (scenario M3)
   - Confidence: high

6. **Starting a sequence.** A request to start kind *q* has the outcome of the first row of this
   table whose condition holds. The *resulting kind* is *q* after the substitutions of claim 7.

   | Row | Condition | Outcome | Water record of claim 7 |
   |---|---|---|---|
   | 1 | *q*'s own entry has *C* = 0. The entry tested is that of the requested kind, not that of a substitute from claim 7 | Turned down: the current sequence and its stage are left as they are | Not updated |
   | 2 | The current sequence is *parachute* and the unit is still falling | Turned down, as row 1 | Not updated |
   | 3 | *q* is *go-prone* and the type cannot crawl (Import mapping) | Turned down, as row 1 | Not updated |
   | 4 | The resulting kind equals the current sequence, even for a forced request | Turned down, as row 1, so the stage keeps running (claim 8) | Updated |
   | 5 | The request is not forced, and a current sequence exists that is not interruptible | Turned down, as row 1 | Updated |
   | 6 | Any other case | The resulting kind becomes the current sequence, the stage is set to 0 and its timer starts on this tick | Updated |

   Every start sets the stage to 0. No start takes a random draw.
   - Backing: `measurable` (scenarios M1, M4)
   - Confidence: high

7. **Substitutions made when a sequence starts.** Four rules can change the requested kind:
   - **Carrying.** *walk* becomes *carry* when the unit is enslaved to a master, its type's storage
     capacity is not 0, and its fill fraction is exactly 1.
   - **Water.** For types whose movement zone is amphibious-destroyer only, when the unit's cell is
     a wet cell and the unit is not on a bridge deck: *walk* and *crawl* become *swim*; *prone* and
     *stand* become *tread*; *idle-a* and *idle-b* become *wet-idle-a* and *wet-idle-b*; *die-1* and
     *die-2* become *wet-die-1* and *wet-die-2*; *fire-standing* and *fire-prone* become *wet-fire*.
   - **Hovering.** *stand* becomes *hover* when the unit is airborne, not on a bridge deck, and its
     *hover* entry has *S* ≥ 1. A *hover* entry starting at image frame 0 can never be shown this
     way (`quirks.hover_needs_nonzero_start`).
   - **Panic.** *walk* becomes *panic* while the fear level is above 199. Being airborne makes no
     difference.

   At most one rule changes a request. Where several match the same requested kind, the one listed
   first wins. So a full carrier shows *carry* rather than *swim* or *panic*, a frightened
   amphibious walker on a wet cell shows *swim* rather than *panic*, and a resting amphibious unit
   on a wet cell gets *tread* rather than *hover*. The kind that results is not checked for *C* = 0
   (`quirks.unchecked_substitute`).

   **Water side effect.** A unit of an amphibious-destroyer type remembers whether it was last on
   a wet cell (not on a bridge deck) or not. Every start request of such a unit updates this
   record, whether or not the request succeeds, except a request that claim 6 turns down because
   the requested kind has no image frames, because the unit is a falling parachutist, or because a type that
   cannot crawl asks for *go-prone*. When the record changes, the type's enter-water sound plays
   (dry to wet) or its leave-water sound (wet to dry), at the unit's position. A freshly created
   unit has no record yet, so its first request plays no sound.
   - Backing: `measurable` (scenarios M1, M5 with M5(b) for a fresh unit's first request, M6, M17)
   - Confidence: high

8. **Looping and completion.** A sequence is **complete** once its stage has reached *C*.
   Completion is detected on every tick, including the tick on which the stage first reaches *C*.
   What follows is in the state table (State and ordering). For most kinds, the follow-up asks for
   the same kind again: for example, still walking asks for *walk*. Claim 6 turns that request down,
   so the stage simply keeps growing and the displayed image frame wraps through *k mod C*. Such
   kinds therefore loop with no break. A **one-shot** is a sequence whose completion follow-up
   starts a different kind. For *C* ≥ 1 and *d* ≥ 1, an uninterrupted one-shot of *C* image frames lasts exactly *C × d* ticks, and its follow-up starts on the same tick as the completion. A positive-count sequence with *d* = 0 never reaches completion by stage advancement. The wrapped image frame *S* + 0
   of the finished positive-count one-shot is never displayed.
   - Backing: `measurable` (scenario M3)
   - Confidence: high

9. **Declared direction.** When a sequence whose entry has a declared direction *D* is complete
   and its follow-up is one of the "any other kind" rows of the follow-up table (State and
   ordering), which include the secondary-fire rows and *air-death-fall*, the unit's body facing is
   set at once, with no turning, to *D* × 8192 (N = 0, NE = 1, … NW = 7). This happens on every
   tick on which the completed sequence is still current (`quirks.repeat_direction_snap`). Completing any other kind (*die-1* …
   *die-5*, *wet-die-1*, *wet-die-2*, *air-death-end*, *deploy*, *undeploy*, *air-death-start*,
   *parachute* and *shovel*) never changes the facing, whatever its entry declares. Entries without a
   declared direction never change the facing. A one-facing sequence (*M* = 0) with a declared
   direction thus leaves the unit facing the way its image frames showed it.
   - Backing: `measurable` (scenario M7: part (a) for the snap and its repetition, part (b) for *deploy* and
     *undeploy*, part (c) for *parachute* and *shovel*); `unmeasured` (that the die kinds and the
     air-death kinds never snap: a die kind removes the unit on completion, so no facing is left to
     observe, and the air-death kinds cannot be started by a scenario, claim 21)
   - Confidence: high

10. **Movement picks the moving and resting kinds.** On every tick:
    - while the unit is not moving now: a current *walk*, *fly* or *hover* is replaced by a request
      for *stand*, *crawl* by *prone*, and *swim* by *tread*. Other kinds are left alone.
    - while it is moving now: a unit that is a jumpjet asks for nothing while in the attack state,
      and otherwise asks for *hover* at a speed fraction of at most 0.8 and *fly* above it. Any
      other unit asks for *crawl* when in the prone state and *walk* otherwise.

    These requests are not forced. The substitutions of claim 7 then turn *stand* into *tread* on
    wet cells, *walk* into *swim*, and so on.
    - Backing: `measurable` (scenarios M5, M17(c)); `unmeasured` (the sentence that a jumpjet in
      the attack state asks for nothing: no scenario is known that reliably holds a jumpjet in the
      attack state while it is moving now, so that sentence stands on the original's rule as read)
    - Confidence: high (medium for that sentence)

11. **Firing.** On a tick on which an infantry unit has a target, is not in the attack state and
    its weapon is ready to fire, it enters the attack state, its body facing is set at once (no
    turning) to the direction of the target, and it asks (not forced) for a firing kind. The
    highest row of this table whose situation holds decides:

    | Priority | Situation | Request |
    |---|---|---|
    | 1 | The unit is a jumpjet | *fire-airborne* |
    | 2 | It is deploying | *deployed-fire* |
    | 3 | It is designating an airstrike, and *secondary-fire* or *secondary-fire-prone* is current | None. Instead the stage of that sequence is set back to 0, and it advances again on the next tick (claim 4) |
    | 4 | It fires its secondary weapon, is prone, and *secondary-fire-prone* has image frames | *secondary-fire-prone* |
    | 5 | It fires its secondary weapon, and *secondary-fire* has image frames | *secondary-fire* |
    | 6 | It is prone | *fire-prone* |
    | 7 | Any other case | *fire-standing* |

    The attack state is entered even when the request is turned down.

    **Fire frame** *F*: one of the type's four fire-frame fields (Import mapping), chosen by the
    unit's stance and the weapon it fires:

    | Weapon fired | Prone | Not prone |
    |---|---|---|
    | Primary | prone field | standing field |
    | Secondary, and the matching secondary kind (*secondary-fire-prone* when prone, *secondary-fire* when not prone) has image frames | secondary-prone field | secondary field |
    | Secondary, and that matching kind has no image frames | prone field | standing field |

    *F* does not depend on which kind is playing, so *deployed-fire*, *fire-airborne* and
    *wet-fire* use the standing or prone value (`quirks.fire_frame_by_stance`).

    **Shot:** on every tick in the attack state on which the stage of the current sequence equals
    *F*, the weapon fires if it is ready. If it is not ready, the attack state ends and the unit asks
    (not forced) for *deployed* when deploying, for *prone* when prone, and for *stand* otherwise.
    With *F* = 0 the shot leaves on the tick the firing kind starts.

    **End of the attack state**, besides the case above: on a tick that begins with a current
    sequence whose stage delay is 0 (only *stand* and guard have one), in which case the unit also
    asks (not forced) for *stand*; when the unit has no target; when its target changes; and when a
    completed *secondary-fire* or *secondary-fire-prone* is repeated for an airstrike (State and
    ordering). When a firing kind completes and its follow-up (State and ordering) is *stand*, whose
    delay is 0, the unit leaves the attack state on the next tick and can start a new firing kind
    once its weapon is ready. A prone unit's follow-up, *prone*, has a non-zero delay, so it stays
    in the attack state.
    It then fires again without a new firing sequence when the stage of *prone* equals *F* while
    the weapon is ready (`quirks.prone_attack_state_persists`). The test uses the stage itself, not
    the displayed *k* mod *C*, and the stage only grows while *prone* loops (claim 8), so it equals
    *F* only on the *d* = 6 consecutive ticks (T1) of one stage value per start of *prone*. On the
    first of those ticks the weapon fires if it is ready, and on any of them on which it is not
    ready the attack state ends (shot rule). This repeat shot therefore happens at most once per
    start of *prone*, provided a shot leaves the weapon not ready on the following tick (firing
    cycle, S-0301).

    Shot timing beyond this belongs to the firing-cycle spec (S-0301). Firing kinds are
    interruptible.
    - Backing: `measurable` (scenario M8: part (a) for the shot timing, the facing snap, fire
      frames 0 and *C* and the prone repeat shot; part (b) for the airstrike reset; part (c) for
      the shot during *go-prone*; part (d) for the end of the attack state on a change of target)
    - Confidence: high (the kind choice, the fire-frame rule, the facing snap), medium (the end
      conditions of the attack state and the prone repeat shot)

12. **Facing toward a target for jumpjets.** For a jumpjet that has a target, the facing used in
    claim 2 is the direction from the unit to its target rather than its body facing. This applies
    unless the type allows jumpjet turning (Import mapping). Only the drawn image frame changes. The
    body facing itself, and with it the movement, are not affected.
    - Backing: `measurable` (scenario M12)
    - Confidence: medium

13. **Disguise.** When the unit's disguise is in effect against the local player, the image frame
    is computed with claim 2 from the sequence table of the disguise type, using the unit's own
    current sequence, stage and facing. The disguise type's entry decides *S*, *C* and *M*, so the
    wrap of the stage follows the disguise type's frame count.
    - Backing: `measurable` (scenario M13)
    - Confidence: medium

14. **Idle actions.** Each infantry unit has an idle timer. An infantry unit attempts an idle action
    whenever a processing of its current behaviour attempts it (Terms). The triggers are *guard*
    and *sticky* with no target (S-0215 claims 24 and 25), *area guard* in the cases of S-0215
    claim 26, and *hunt* without a target, or with the stupid-hunt flag (S-0215 claim 27). S-0215
    claim 27 is the one case that is not a resting behaviour: a unit that hunts and finds no target stays in
    *hunt* and attempts the idle action at every processing, like a resting one. The attempt goes ahead only if
    all of these hold: the idle
    timer has run out; the unit has no movement in progress; it is not prone; it is not in the
    attack state; and its current sequence is *stand*, guard or *tread*. A newly created unit's
    idle timer has already run out, so its first attempt that meets the other conditions goes
    ahead (medium confidence).

    Every attempt that goes ahead restarts the timer with a length of *T* ticks (Formulas F4). The
    first row of this table that matches the unit gives the rest of the attempt's effect and the
    draws it takes from the scenario random stream:

    | Unit | Effect | Draws, in stream order |
    |---|---|---|
    | A fraidycat whose fear level is above 50 and whose owner is not human-controlled (see below) | It scatters, and nothing else happens | The timer draw only |
    | The hard-coded "cow" type (see Import mapping) | The outcome of *r* in the table below, except that when *e* < 5 it is the outcome of *r* = 8, whatever *r* was (`quirks.cow_idle`) | The timer draw, *r*, *e*, then δ when the outcome is a turn |
    | Any other unit | The outcome of *r* in the table below | The timer draw, *r*, then δ when the outcome is a turn |

    *r* and *e* are integers, each drawn uniformly from 0 to 10 inclusive.
    The cow designation depends only on the infantry type's registry name, compared
    case-insensitively with the identifier given in Import mapping. Neither its display
    name nor its image name selects this rule (Q-0212). A renamed copy with the same
    appearance therefore loses the cow-specific idle effects, while another infantry
    type registered under the designated name gains them.

    **Owners that are not human-controlled.** The original applies the first row, and the scatter of
    the outcome *r* = 8, to units whose owner is not human-controlled. In our engine an AI house
    counts as human-controlled (glossary *AI house*, decision of the project lead, 2026-10-09), so
    a fraidycat of an AI house takes neither rule: it takes the third row, and on *r* = 8 it
    turns without scattering, as one of a human-controlled house does. The neutral house and the
    special house are not AI houses and keep the original's rule (decision of the project lead,
    2026-10-10): a fraidycat of either takes the rows above as written. The cow's scatter is a
    separate rule of its own (`quirks.cow_idle`) and holds for every owner.

    | *r* | Outcome |
    |---|---|
    | 0 | Nothing visible happens |
    | 1, 2, 7 | *idle-b* is requested (not forced) |
    | 3, 4, 5 | *idle-a* is requested (not forced) |
    | 6, 9, 10 | A direction δ is drawn uniformly from 0 to 7, and the body facing is set at once to δ × 8192 |
    | 8 | The same facing change, followed by a scatter of the cow (`quirks.cow_idle`), or of a fraidycat whose owner is not human-controlled |

    Every draw named here is a ranged draw. Their number and their order in the stream are part of
    the behaviour, because every other user of the scenario random stream receives the draws that
    follow them (glossary *scenario random stream*).

    When the outcome is *idle-b*, a unit owned by the local player that is not selected, and whose
    type has at least one comment sound, also takes a comment chance on the same tick, whether or
    not the *idle-b* request was turned down. It uses the unsynchronised
    random stream (an integer drawn from 0 to 2, a comment on 0), and only the type's first comment
    sound is ever used (`quirks.first_comment_only`).
    - Backing: `measurable` (scenario M9: part (a) for the timer bounds and the outcome ratios,
      measured statistically over many games; part (b) for the streams used, the number and the
      order of the draws, read with a *recorder* (glossary) that reads the random streams'
      state; part (c) for the *hunt* trigger; part (d) for the cow identifier and the
      exclusion of display and image names)
    - Confidence: high (outcomes, the streams used, the draw order, and that the four behaviours
      named are the only ones whose processing attempts an idle action)

15. **Fear selects prone, get-up and panic.** Write *f* for a unit's fear level at the start of a
    tick and *f*′ for that start-of-tick level once that tick's decay has applied to it: *f*′ = *f* − 1 when *f* > 0 and
    the type is not fearless, otherwise *f*′ = *f*. On each tick the unit follows these rules:

    | Rule | Ticks on which it applies | Unit | Fear test | Result |
    |---|---|---|---|---|
    | Lie down | Those with *f* > 0 | Not prone, not deploying, not a fraidycat, and either its owner is not human-controlled or it has no movement destination and no movement in progress | *f*′ ≥ 50 | Asks (not forced) for *go-prone* |
    | Get up | Those with *f* > 0 | Prone, not deploying | *f*′ < 50 | Asks (not forced) for *get-up* |
    | Flee | Every tick | A fraidycat that is not deploying, not falling, has no movement in progress and has no destination | *f*′ > 50 | Scatters |

    All three rules test *f*′, the level after that tick's decay. So on a tick that begins with a
    fear level of 0, these rules make a unit neither lie down nor get up, and a fraidycat whose
    level is 51 at the start of a tick (and is not fearless) does not scatter on that tick, because
    *f*′ = 50. A hit that changes the fear level during a tick (claim 16) changes neither *f* nor
    *f*′ of that tick, so it first takes part in these rules on the next tick. Observable
    consequence: for a unit that the lie-down rule applies to, whose fear level of 0 a hit sets to
    100, *go-prone* is asked for on the tick after the hit, not on the tick of the hit, and *get-up*
    51 ticks after the tick of the hit, not 50 (scenario M6(a) tests the *get-up* time).

    *go-prone* is turned down for a type that cannot crawl (claim 6), so such types never lie down.
    Walking with a fear level above 199 shows *panic* (claim 7).

    **Destination-driven get-up (Q-0087).** A prone unit whose owner is human-controlled,
    whose type is neither a fraidycat nor a cyborg, asks (not forced) for *get-up* when a
    non-empty destination is assigned to it **that is already its current destination**.
    Assigning a different destination, assigning the first destination to a unit with none,
    and clearing a destination do not make this request
    (`quirks.prone_get_up_repeated_destination`). Equality means the same cell or the same
    object, not merely two objects in the same cell. The request is subject to claim 6:
    a missing *get-up* entry or a non-interruptible current sequence can refuse it. It is
    not saved for later, and neither starting to crawl nor entering a new cell retries it.
    No fear threshold is tested by this destination rule. Fear-driven get-up is separate:
    it follows the fear table above, not the destination-equality condition.

    The name of the order does not decide this rule. The following table gives its application
    to the orders Q-0087 asks about. S-0213 owns how an applied order assigns destinations,
    and S-0215 and the movement specs own subsequent destination changes.

    | Order or event | Destination-driven get-up |
    |---|---|
    | Plain move | At application only when the non-empty destination assigned is already current |
    | Attack-move with a destination | Same as plain move, after S-0213's translation |
    | Attack on an out-of-reach target | Being out of reach does not itself request get-up. Assigning or reassigning a chase destination uses the same equality rule, whether at order application or later |
    | Area guard | Assigning the guard spot as destination uses the same equality rule |
    | Queued move or waypoint | Merely recording a future waypoint does not count. When it becomes the assigned destination, the same equality rule applies |
    | Scatter | Only an accepted scatter that assigns a destination can count, and only if that destination is already current. The usual first scatter from rest assigns a new destination and does not count |
    | Enter or capture | Assigning a destination uses the same equality rule. Setting only a target does not count |
    | Move to the cell occupied by the unit | Occupying that cell is not the equality test. With no current destination it does not count; reassigning that cell while it is already the current destination does |

    The request happens at destination assignment, not when the locomotor starts moving.
    When an applied order reassigns the current destination, it can start *get-up* at that
    order boundary (S-0213 claim 1), before the next tick's movement. Later reassignment
    by a behaviour can start it on a later tick. Once *get-up* starts the unit is no longer
    prone (Terms), although its non-interruptible get-up image frames still play to completion.
    An owner that is not human-controlled makes no destination-driven get-up request.

    **Owners that are not human-controlled.** The table and the get-up rule above give the original's
    rule for each owner kind. In our engine an AI house counts as human-controlled (glossary *AI
    house*, decision of the project lead, 2026-10-09), so its units follow the human-controlled
    wording: the lie-down rule waits for them to be at rest, and reassigning their current
    non-empty destination makes a prone one ask for *get-up*. The neutral house and the special house keep the original's rule for owners that are
    not human-controlled (decision of the project lead, 2026-10-10): their units lie down whether
    or not they are moving, and a prone one keeps crawling when it is ordered to move.
    - Backing: `measurable` (scenario M6, with M4 for destination-driven get-up, its order
      cases, equality test and timing, and M6(g) for the lie-down timing of a human-controlled owner),
      and `unmeasured` (the half of the get-up rule that makes a prone unit of an owner that is
      not human-controlled keep crawling on a move order, and the fraidycat exception of the same
      rule: a move order for such a unit comes only from the original's computer-player scripts,
      which no scenario can drive, and a fraidycat never lies down in stock play, so those two
      parts stand on the original's rule as read)
    - Confidence: high (including the repeated-destination test; medium for those two
      unmeasured parts and for later assignment timing, which the cited owning specs determine)

16. **How damage raises fear.** When a living infantry unit takes damage and survives it, its new
    fear level is given by this table. A unit is *fearless* here when its type is fearless or it
    has the fearless veterancy ability. Which scatter requests the hit sends, and whether the unit
    acts on them, belong to S-0208 (claims 15 and 11); this claim only sets the fear level.

    | Known attacker | Fear before the hit | Fraidycat | Fearless | New fear level |
    |---|---|---|---|---|
    | yes | below 100 | yes | yes or no | 300 |
    | yes | below 100 | no | no | 100 |
    | yes | below 100 | no | yes | unchanged |
    | yes | 100 or more | yes or no | no | min(fear + *i*, 300) |
    | yes | 100 or more | yes or no | yes | unchanged |
    | no | any | yes or no | no | min(fear + *i*, 300) |
    | no | any | yes or no | yes | unchanged |

    *i* is the increment of Formulas F5, an integer, so the new fear level is an integer from 0 to
    300. The health ratio in F5 is taken after the damage has been applied. The first row holds even for a fearless fraidycat
    (`quirks.fraidycat_fear_ignores_fearless`). Because a fearless type's fear never decays
    (claim 15), a fraidycat whose type is fearless keeps a fear level of 300 for good after its
    first hit by a known attacker: it shows *panic* whenever it walks, and it scatters by the flee
    rule of claim 15 and, when its owner is not human-controlled (in our engine, the neutral or
    special house only, claim 14), at every idle attempt that goes ahead (claim 14). A fraidycat that is fearless only through the veterancy ability decays from
    300 like any other unit.

    The fear mechanics stay in this spec for now because they choose sequences. They are expected
    to move to a future infantry-behaviour spec (Summary).
    - Backing: `measurable` (scenarios M6, M15(a) for a known attacker, M15(b) for no known
      attacker)
    - Confidence: high (the table), medium (the fearless-fraidycat consequences, which follow from
      the table and claim 15 but have not been seen)

17. **Death sequences.** When an infantry unit is killed, the highest row of this table whose case
    holds decides what is shown:

    | Priority | Case | What is shown |
    |---|---|---|
    | 1 | The unit's height above the surface (Terms) is less than 11 leptons, the land type of its cell is water (beach does not count), and a further per-object condition holds that has not been identified | Splash animations; no sequence |
    | 2 | Its type is a cyborg and it is prone, or its type is a jumpjet type (whatever its current locomotor) | A general death animation (rules data, another spec); no sequence |
    | 3 | Its type has its own list of death animations | Entry *w* of that list, where *w* is the death kind below (entry 0 when *w* is outside the list or entry *w* is empty); no sequence |
    | 4 | Its type is marked "not human" | A forced request for *die-1*, whatever the death kind *w* (0, 3, 5 and 8 included). When *w* = 8, an extra animation from rules data (another spec) also appears |
    | 5 | Any other case, with *w* = 1 | A forced request for *die-1* |
    | 6 | Any other case, with *w* = 2 | A forced request for *die-2* |
    | 7 | Any other case, with *w* = 9 | A forced request for *die-2*, unless that kind's own replacement effect (another spec) takes place, in which case there is no sequence |
    | 8 | Any other case, with any other *w* | Animations that other specs describe; no sequence |

    In the rows that show no sequence (1, 2, 3, 8 and the replacement of row 7), the unit is
    removed at once, unless its type has the crash flag and a crash begins instead (jumpjet locomotor,
    another spec). Rows 5 to 8 apply only to types that are not marked "not human".

    The death requests of rows 4 to 7 go through claim 6 like any other start request. Being
    forced only overrides the interruptible attribute (row 5 of claim 6), so they are turned down
    when the requested entry has *C* = 0 (claim 6, row 1) and while the unit is a falling
    parachutist (claim 6, row 2). A turned-down death request shows no death sequence. The
    death handling then does not remove the unit either (except in the cyborg case below): it keeps its current sequence (for a
    falling parachutist, *parachute*). What becomes of such a unit afterwards is not established
    (Open questions). Consequences: a type marked "not human" that rows 1 to 3 do not catch shows
    *die-1* when it dies on the ground, also to a laser fence, but no death sequence while it falls
    under a parachute. A unit of any other type that rows 1 to 3 do not catch, killed while parachuting, has death
    kind 3 (below), so row 8 applies and it is removed at once.

    **A death request turned down because the entry has no image frames**
    (`quirks.refused_death_keeps_unit`, default: on). For a type whose *die-1* or *die-2* entry has
    *C* = 0, a death request for that kind is turned down by row 1 of claim 6, which is judged
    before row 2. By default the death handling then leaves the unit in play with its current
    sequence and shows no death sequence, exactly as in the falling-parachutist case above. What
    becomes of it afterwards is the same open question (scenario M10(b) records it). Off: when row
    1 turns the death request down, whatever the unit's other state, and the cyborg case below does
    not apply, the unit is treated as if the kind that would have resulted (after the
    substitutions of claim 7, so *wet-die-1* or *wet-die-2* on a wet cell) had completed on the
    tick of its death, with the corpse or removal that completion gives (below). The importer
    warns about any infantry type whose *die-1* or *die-2* entry has no image frames, because the
    original loses the death silently. The scope is exactly row 1. A falling parachutist of a type
    marked "not human" whose *die-1* entry has image frames is turned down by row 2 only, and
    keeps the original's behaviour whatever the switch says. If that type's *die-1* entry has
    *C* = 0, row 1 applies first, so the switch decides: on, the unit stays in play while it
    falls; off, it is removed even though it is falling.

    The **death kind** *w* is 5 when the killer is a laser-fence building, otherwise 3 when the
    unit's current sequence is *parachute*, and otherwise the infantry-death kind of the killing
    warhead. When rows 1 and 2 do not apply, a unit killed while its current sequence is
    *parachute* by a warhead of infantry-death kind 8 also shows an extra animation from rules
    data, whatever *w* then is (so also with a laser-fence killer).

    **Cyborgs and damage that ignores defences.** When the killing damage ignores defences (Terms)
    and the type is a cyborg, two things change. If the unidentified per-object condition of row 1
    holds, the general death animation of row 2 also appears, in addition to what the table shows. And
    when rows 4 to 7 lead to a death request, the unit is removed at once on the same tick
    (subject to the same crash-flag exception), whether or not the request succeeded, so no death
    sequence is seen and no corpse is left.

    *die-3*, *die-4* and *die-5* are read from the art but never played by the stock engine. A
    death sequence is not interruptible. When *die-1* to *die-5* complete,
    a corpse animation is left at the unit's position and the unit is removed on the same tick. The
    corpse comes from the type's own corpse list when that list is not empty. Otherwise it comes
    from the general corpse list (for a list left over from an earlier game, see Import mapping),
    unless the type is marked "not human", in which case there is no corpse. The pick takes one draw from the scenario random stream, read as an unsigned 32-bit
    number and reduced modulo the list length. When *wet-die-1*, *wet-die-2* or *air-death-end*
    complete, the unit is removed with no corpse. Because of the water case above, a wet death
    sequence may in practice be seen only on beach cells.
    - Backing: `measurable` (scenario M10: part (a) for the tested death kinds and falling
      paratroopers, part (b) for the switch's default, part (c) for the removal of a cyborg killed by
      damage that ignores defences, part (d) for corpse-list precedence and the raw draw,
      part (e) for the type's death-animation list); `unmeasured` (row 1, the water case, and the sentence of the cyborg paragraph that
      depends on its per-object condition: that condition is not identified, so no scenario can
      set it up, and M10 only records the water-cell outcome to help identify it; and a general
      corpse list left over from an earlier game, Import mapping: the original's result is
      unspecified, and our engine's result is a design choice with nothing to compare it with)
    - Confidence: high (the die-1/die-2 choice, the completion behaviour, the corpse pick, the
      precedence of the laser-fence replacement, the refusal of a falling parachutist's death
      request), medium (the order of the special cases, the extra animations for kind 8, the
      removal in the no-sequence rows and its crash-flag exception, the cyborg paragraph), low (the
      water case and its unidentified condition)

18. **Sound cues.** A cue can play only on a tick on which the unit's stage advanced. On such a
    tick, each cue of the sequence then current plays at the unit's position when
    *k mod max(C, 1)*, with *k* the stage of that sequence, equals the cue frame. The cues that
    count are those of the sequence that is current once that tick's fear, firing and completion
    rules (claims 8, 11 and 15) have made their changes. A change that the movement rule (claim 10)
    makes on the same tick does not count for that tick's cues. Consequences:
    - a looping sequence repeats its cues on every pass, once per *C × d* ticks.
    - a frame-0 cue of a sequence started by the fear, firing or completion rules plays at once when
      the stage advanced earlier on the same tick. A one-shot reaches *C* on a tick whose stage
      advanced, so the follow-up that the completion starts on that tick is always such a case. So
      a frame-0 cue of *stand* plays each time a completed one-shot hands over to *stand*. A
      follow-up that a looping sequence's completion starts later, on a tick without an advance,
      does not play it.
    - a sequence started by the movement rule never plays a frame-0 cue on its start tick. If its
      stage delay is 0 (*stand*), it never plays any cue. Otherwise its frame-0 cue first plays when
      the stage wraps, so a one-shot started that way never plays it.
    - Backing: `measurable` (scenario M14)
    - Confidence: high (the rule), medium (when sequences started by other behaviour, such as idle
      actions and orders, play a frame-0 cue, which depends on when that behaviour runs in the tick)

19. **Shadows.** An infantry unit whose height above the surface is 0 and whose type does not
    suppress shadows (Import mapping) also draws shadow image frame *frame* + ⌊L/2⌋ of its body
    sprite, at the same screen position, as a darkening. A unit standing on a bridge deck has a
    height above the surface of 0 (Terms), so it draws this shadow too. A unit whose height above
    the surface is not 0 draws no shadow image frame from its own body sprite. When its height
    above the surface is above 0, a generic infantry-shadow sprite is drawn instead, image frame 1,
    at the ground position below the unit. For a unit that is not on a bridge deck but is above a
    bridge cell at or above the deck, the deck counts as the ground for this generic shadow, under
    a further condition on the neighbouring cells that is not specified here. No game option or
    data turns the body-sprite shadow off for all units.
    - Backing: `measurable` (scenario M11); `unmeasured` (the neighbouring-cell condition of the
      case above a bridge deck: it is not identified, so no scenario can set it up)
    - Confidence: high (on the ground and on a bridge deck), medium (the generic shadow and its
      image frame number, and that nothing turns the body-sprite shadow off for all units), low
      (the case above a bridge deck)

20. **Unused kinds.** The kinds guard, *tumble* and *deployed-idle* are read from the art
    (Import mapping), but no stock behaviour ever starts any of the three. Their entries therefore
    never show. The guard conditions of claims 11 and 14 apply only if a mechanic of another
    spec starts guard.
    - Backing: `measurable` for the activities exercised in M16; `unmeasured` for the exhaustive
      absence of all three kinds from every stock behaviour, because finite play tests cannot
      exercise every behaviour path. M16 can falsify that broader claim but cannot confirm it.
    - Confidence: medium

21. **Kinds started by other mechanics.** These kinds are started by mechanics that other specs
    own, which also say when: *deploy* and *undeploy* (deployment, which also plays the type's
    deploy and undeploy sounds when those kinds start), *deployed* (deployment and claim 11),
    *cheer* (the celebration order and successful slave-release transfers under S-0312 claims 30
    and 31), *parachute* (paradrops, forced), *shovel* (harvesting),
    *air-death-start* (the jumpjet locomotor) and *air-death-end* (the end of an air death, forced).
    *idle-a* and *idle-b* can also be started by scripted idle orders. Their timing (claims 4 and 5)
    and their completion (State and ordering) are given here. *air-death-fall* is started only by
    the completion of *air-death-start*. It has no completion rule of its own, so when it completes
    the general follow-up rows apply (`quirks.air_death_fall_generic_follow_up`).
    - Backing: `measurable` for deploy/undeploy starts (M7(b)), parachute starts (M7(c)), and
      harvesting starts of shovel (M7(c), M17(a)), and slave-release cheer requests (M19).
      `unmeasured` for the air-death starts and
      follow-up: no controlled scenario is known that starts these kinds through the jumpjet
      locomotor. Celebration-order cheer starts, scripted idle orders and the deployment sounds are
      also `unmeasured` here: no controlled legal skirmish trigger for celebration or scripted idle
      orders is specified here, and the deployment-sound start boundary has no isolated observation
      in the proposed deployment runs.
      Their timing, once started, follows claims 4 and 5; this is not evidence that their start
      triggers have been measured.
    - Confidence: high (that these mechanics start these kinds), medium (the *air-death-fall*
      follow-up)

22. **Limits of the sequence data.** Three rules of reading the sequence data (Import mapping) have
    effects that can be seen:
    - A declared direction is recognised only when it is written exactly as one of the eight
      upper-case compass abbreviations. Anything else, lower case included, gives an entry with no
      declared direction, so claim 9 never changes the facing for it
      (`quirks.case_sensitive_direction`).
    - An entry keeps at most two sound cues: the first two pairs, in the order written, whose sound
      is known. A pair with an unknown sound is dropped and does not count towards the two. Later
      pairs never play (`quirks.two_sound_cues`).
    - The reference to the sequence section, each kind value and each cue value are read as at
      most 31 bytes of text. The value is parsed as if it ended after its 31st byte, so a cue
      pair that begins after it never plays, and a field of a kind value that begins after it keeps
      its default of claim 1 (`quirks.sequence_value_length_limit`).
    - Backing: `measurable` (scenario M18; the 31-byte limit is tested on a kind value, a cue value and the sequence-section reference)
    - Confidence: high (the direction rule and the two-cue limit), medium (the 31-byte limit)

## Formulas

**F1 Image frame.** Integer arithmetic, with *k* ≥ 0 and mod the non-negative remainder:

> frame = S + (k mod max(C, 1)) + [M > 0] × M × g(φ)

**F2 Facing group.** Integer arithmetic on the facing φ (0 to 65535):

> j = ⌊(φ + 1024) / 2048⌋ mod 32
>
> g = (7 − ⌊(j + 2) / 4⌋) mod 8

Here mod is the non-negative remainder. *j* is φ rounded to the nearest 32nd of a turn, a facing
exactly half-way rounding clockwise. Equivalently, in one step, g = (7 − ⌊(φ + 5120) / 8192⌋)
mod 8. The table form is in Constants T2.
The alternative selected by turning `quirks.infantry_facing_sector_skew` off is

> g = (7 − ⌊(φ + 4096) / 8192⌋) mod 8

with sectors centred on the compass points. A facing exactly half-way between two compass points
(an odd multiple of 4096) belongs to the clockwise one.

**F3 Game-speed normalisation** of a base delay *b* at game-speed index *s*: integer arithmetic,
result in ticks.

> N(0, s) = 0
>
> N(b, s) = A(b, s), the entry in row b and column s of Constants T3, for 1 ≤ b ≤ 4
>
> N(b, s) = ⌊8b / (s + 1)⌋ for b ≥ 5

*b* is a non-negative integer and *s* an integer from 0 to 7. The division of the last line is
integer division of non-negative values, so truncation and the floor agree. No normalised kind in
T1 has *b* ≥ 5, so the last line is never used by infantry. It is given because the normalisation
is one rule of the original, and S-0108 applies it to building delays that can be 5 or more. The
jump from N(4, 0) = 7 to N(5, 0) = 40 is the original's: the table and the formula are separate
rules that do not join smoothly.

**F4 Idle timer length** in ticks. *R* is the imported idle-action frequency in game minutes, read as a
decimal per S-0003 (claims 6 and 7: single precision, widened to a double) and held as a double.
When the key is absent, *R* is its absent-key default (Import mapping), which is a double and does
not pass through single precision. *x* is an integer from one ranged draw, uniform from 0 to
2147483646 inclusive.

> P = R × 1800, Q = R × 450, u = x × c
>
> T = trunc(Q + (P − Q) × u)

*c* is the double nearest to 1/2147483646, namely (2³⁰ + 1) / 2⁶¹. All arithmetic is done at double precision, with a 53-bit significand, and each
of these results is rounded toward zero: the two products, the difference P − Q, the scaled draw
*u*, the product (P − Q) × *u* and the final sum: the glossary's *round-toward-zero mode*, owned
by S-0205 (number domains, claim 5) with its switch `quirks.chop_rounding`. trunc converts the
sum to an integer by truncation toward zero. The grouping shown is part of the behaviour.
Mathematically *T* ≈ 450R + 1350R·x/2147483646. *u* is at most 1 − 2⁻⁵³, so for *R* ≥ 0, *T* lies
between about 450R and just below 1800R. 450 and 1800 are half a game minute and two game minutes at
900 ticks per game minute.

**F5 Fear increment.** *i* is an integer:

> i₀ = 50 when h ≤ red, i₀ = 25 when h > red
>
> i = ⌊i₀ / 2⌋ when h > yellow, i = i₀ otherwise
>
> new fear = min(fear + i, 300)

So *i* is 50, 25 or 12. *h* is the health ratio: the unit's health divided by its type's strength
field, both integers, divided in double precision and rounded toward zero
(the *round-toward-zero mode*, S-0205 claim 5). *red* and *yellow* are the
imported condition ratios, read as decimals with an optional percent sign per S-0003 (claims 6
to 8) and held as doubles. The comparisons are exact comparisons of doubles. *i* = 25 occurs
whenever the two ratios differ and *h* lies above the lower of them and at or below the higher:
when red < *h* ≤ yellow, *i*₀ = 25 is not halved, and when yellow < *h* ≤ red, *i*₀ = 50 is halved.
So *i* is 12 above both ratios and 50 at or below both, whichever of the two is higher. With both
ratios at their absent-key default of 0.5, *i* is 50 or 12 and never 25.

**Strength 0.** The health ratio needs a strength other than 0. With a strength of 0, the
original's health ratio comes out as positive infinity, with no error, which lies above both
ratios, so *i* is 12 (confidence: medium; backing: `unmeasured`, because no controlled scenario is known in which a zero-strength infantry unit survives damage and reaches this fear calculation). Our engine reaches the same result without dividing: for a strength of
0 it takes *h* to lie above both ratios, so *i* = 12. A strength of 0 or below is bad data, and the
importer warns about any infantry type that has one (glossary *importer warning*). Whether a unit of strength 0 can survive
damage at all is an open question.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Sequence kinds | 42 | count | Listed in Terms |
| Max sound cues per entry | 2 | count | Further cues are ignored (claim 22, `quirks.two_sound_cues`) |
| Facing sectors before grouping | 32 | directions | 2048 facing units each |
| Facing groups | 8 | groups | |
| Sector skew | 1024 | facing units | Counter-clockwise, `quirks.infantry_facing_sector_skew` |
| Declared direction step | 8192 | facing units | One eighth of a turn |
| Prone threshold | 50 | fear points | *go-prone* at ≥ 50, *get-up* below 50 |
| Panic threshold | 199 | fear points | *panic* when fear > 199 |
| Fraidycat scatter threshold | 50 | fear points | Scatter when fear > 50 |
| Fear on attack | 100 | fear points | 300 for fraidycats, fearless or not (claim 16) |
| Fear cap | 300 | fear points | |
| Fear decay | 1 | fear point per tick | Not for fearless types |
| Fear increments | 50, 25, 12 | fear points | F5 |
| Hover/fly speed split | 0.8 | speed fraction | *hover* at ≤ 0.8; the double nearest to 0.8 |
| Moving test at completion | 0.1 | speed fraction | Above it, the follow-up is *walk* or *crawl*; the double nearest to 0.1 |
| Idle choice range | 0 to 10 | ranged-draw result (integer) | Inclusive, 11 outcomes |
| Ticks per game minute | 900 | ticks per game minute | Converts *R* in F4 (glossary *game minute*) |
| Idle timer bounds | 450·R to 1800·R | ticks | R in game minutes (F4) |
| Idle timer draw range | 0 to 2147483646 | ranged-draw result (integer) | Inclusive (F4) |
| Idle timer scale *c* | (2³⁰ + 1) / 2⁶¹ | per draw | A double (F4) |
| Airborne height | 2 | height levels | 208 leptons, for the *hover* substitution |
| Water death height | 11 | leptons | Height above the surface; killed below it over water: splash, no sequence |
| Idle comment chance | 1 in 3 | draws | Unsynchronised random stream, draw 0 of 0 to 2 |
| Value length limit | 31 | bytes of text | Sequence art values (claim 22, Import mapping, `quirks.sequence_value_length_limit`) |
| Generic shadow image frame | 1 | image frame | Of the generic infantry-shadow sprite (claim 19) |

**T1. Engine table per sequence kind** (I = interruptible, b = base delay in ticks, n = normalised
by game speed):

| Kind | I | b | n | Kind | I | b | n |
|---|---|---|---|---|---|---|---|
| stand | yes | 0 | | wet-die-2 | no | 1 | |
| guard | yes | 0 | | wet-fire | yes | 1 | |
| prone | yes | 6 | | hover | yes | 2 | yes |
| walk | yes | 3 | | fly | yes | 1 | |
| fire-standing | yes | 1 | | tumble | yes | 1 | |
| go-prone | no | 1 | | fire-airborne | yes | 1 | |
| crawl | yes | 1 | | deploy | no | 1 | |
| get-up | no | 1 | | deployed | yes | 1 | |
| fire-prone | yes | 1 | | deployed-fire | yes | 1 | |
| idle-a | yes | 3 | yes | deployed-idle | yes | 1 | |
| idle-b | yes | 3 | yes | undeploy | no | 1 | |
| die-1 … die-5 | no | 1 | | cheer | no | 3 | yes |
| tread | yes | 3 | | parachute | yes | 1 | |
| swim | yes | 1 | | air-death-start | no | 3 | |
| wet-idle-a | yes | 3 | yes | air-death-fall | no | 1 | |
| wet-idle-b | yes | 3 | yes | air-death-end | no | 3 | |
| wet-die-1 | no | 1 | | panic | yes | 4 | |
| | | | | shovel | yes | 6 | |
| | | | | carry | yes | 3 | |
| | | | | secondary-fire | yes | 1 | |
| | | | | secondary-fire-prone | yes | 1 | |

**T2. Facing group by 32-way direction j:**

| j | 0–1 | 2–5 | 6–9 | 10–13 | 14–17 | 18–21 | 22–25 | 26–29 | 30–31 |
|---|---|---|---|---|---|---|---|---|---|
| g | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 7 |

**T3. Normalisation table A(b, s)** (ticks):

| b \ s | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 1 | 2 | 2 | 1 | 1 | 1 | 1 | 1 | 1 |
| 2 | 3 | 3 | 3 | 2 | 2 | 2 | 1 | 1 |
| 3 | 5 | 4 | 4 | 3 | 3 | 2 | 2 | 1 |
| 4 | 7 | 6 | 5 | 4 | 4 | 4 | 3 | 2 |

## State and ordering

**Follow-up when the current sequence is complete** (checked every tick). "Forced" marks a
forced start. A follow-up equal to the current kind is turned down by claim 6 and the sequence
loops. "Not moving" in this table means: no movement in progress, or a speed fraction of at most
0.1. "Moving" means the opposite.

| Completed kind | Condition | Follow-up |
|---|---|---|
| die-1 … die-5 | always | corpse animation, unit removed |
| wet-die-1, wet-die-2, air-death-end | always | unit removed, no corpse |
| deploy | always | *deployed* (forced). Unless the type stays crushable when deployed, the unit stops being crushable |
| undeploy | always | *stand* (forced). The unit becomes crushable again, same condition |
| air-death-start | always | *air-death-fall* (forced) |
| parachute | always | none here. Separately, on every tick, a unit whose current sequence is *parachute* and that is no longer falling asks for *stand* (forced) |
| shovel | the unit is harvesting | *shovel* (forced), which loops |
| shovel | otherwise | *stand* (forced) |
| secondary-fire, secondary-fire-prone | no movement in progress (whatever the speed fraction), designating an airstrike, and has a target | *secondary-fire-prone* (forced) when prone, otherwise *secondary-fire* (forced); the attack state ends. Asking for the current kind loops it |
| any other kind (and the two above otherwise) | not moving, and the completed kind is *deployed*, *deployed-fire* or *deployed-idle* | *deployed* (forced) |
| any other kind | not moving, and prone | *prone* (forced) |
| any other kind | not moving, otherwise | *stand* (forced) |
| any other kind | moving, and prone | *crawl* (forced) |
| any other kind | moving, otherwise | *walk* (forced) |
| none (no current sequence) | as for "any other kind" | as for "any other kind", without a declared direction |

"Any other kind" includes *air-death-fall* (claim 21). In the "any other kind" rows, including the
secondary-fire rows, the facing is set to the completed kind's declared direction (claim 9) on the
completion tick, also when the follow-up replaces that kind on the same tick.

**Same-tick interactions of one unit's rules, as observable consequences:**
- A stage value reached on a tick already holds on that tick for the shot (claim 11) and for
  completion (claim 8). The shot is released on the very tick the stage reaches the fire frame.
- A sequence that the fear rules (claim 15) start on a tick already holds for the firing rules
  (claim 11) on that same tick. A unit that starts *go-prone* on a tick already counts as prone
  for the firing rules on that tick; its firing request is turned down while *go-prone* plays (not
  interruptible), but it enters the attack state all the same. Observable consequence: its body
  facing snaps to the target on the tick *go-prone* starts, no firing kind is drawn, and its shot
  leaves, if its weapon is still ready, on the tick on which the stage of *go-prone*, or of the
  *prone* that follows it, first equals the prone fire frame (claim 11, shot rule; scenario M8(c)).
- The fear and firing rules of a tick (claims 11 and 15) are judged against the sequence that is
  current before that tick's completion follow-up (claim 8). So a sequence that is not
  interruptible still turns their requests down on the tick it completes; the follow-up starts on
  that tick, and the turned-down request can succeed on the next tick at the earliest. Observable
  consequence: a computer-controlled unit that is frightened while *get-up* plays shows, after the
  last *get-up* image frame, exactly one tick of *stand* or *walk* (its follow-up, or the movement
  rule's replacement of it while it scatters) before *go-prone* starts (scenario M6(f)). Conversely, when the fear or firing rules
  replace an interruptible sequence on the tick it would complete, the new sequence is not complete
  (unless it has no image frames, claim 7), so that tick has no follow-up and no declared-direction snap (claim 9) from the replaced one.
- When the attack state ends because a tick begins with a stage delay of 0 (claim 11), it has
  already ended for the fear and firing rules of that same tick. A unit whose firing kind
  completed on tick *t* with *stand* as its follow-up leaves the attack state on tick *t* + 1 and
  can start a new firing kind on that same tick *t* + 1.
- When the fire frame equals *C*, the shot is still released, on the tick the sequence completes.
- A sequence that a completion follow-up starts on a tick already holds for the movement rule
  (claim 10) on that same tick. The sound cues of a tick are those of the sequence current before
  any change the movement rule makes on that tick (claim 18). A unit that stops moving in mid-walk
  switches to *stand* on the first tick on which it is not moving now, wherever the walk cycle is.
- The image frame drawn after a tick is that of the sequence in effect once all of the unit's
  rules for that tick have made their changes. When a completion starts a follow-up and the
  movement rule replaces it on the same tick, for example a *walk* follow-up chosen because
  movement is in progress while the unit is not moving now, the replacement (*stand*) is drawn and
  the follow-up is never seen. The finished one-shot's wrapped image frame never appears either
  way.

## Edge cases and quirks

- **`quirks.infantry_facing_sector_skew`** (default: on). The eight facing sectors are shifted 1024
  facing units counter-clockwise of the compass points (claim 3). For example, a unit facing
  φ = 3500 (about 19° clockwise of map north) shows group 6, not 7. Off: sectors centred on the
  compass points (F2 alternative).
- **`quirks.unchecked_substitute`** (default: on). The *C* = 0 test applies to the requested kind
  only. A substituted kind with no image frames (*panic*, *carry*, *tread*, *swim* and the other wet kinds)
  is still started, and then shows the single image frame per facing that claim 2 gives with
  *C* = 0 (image frame 0 when the data leaves the key out), with no animation. Off: when the
  substitute has *C* = 0, keep the requested kind instead.
- **`quirks.case_sensitive_direction`** (default: on, claim 22). The declared direction is recognised only
  as an exact upper-case compass abbreviation. Anything else, including lower case, means "no
  declared direction". Off: case-insensitive.
- **`quirks.tread_default_water_only`** (default: on). With no current sequence, the *tread*
  fallback of claim 2 checks for water only, while the substitutions of claim 7 also treat beach as
  wet. Off: beach also counts in claim 2.
- **`quirks.repeat_direction_snap`** (default: on). A completed looping sequence with a declared
  direction re-applies the direction on every tick until it is replaced, so the unit cannot turn
  while it stays in that sequence. Off: apply it once, on the completion tick.
- **`quirks.hover_needs_nonzero_start`** (default: on). The *hover* substitution requires the
  *hover* entry to start at image frame 1 or later. Off: require *C* > 0 instead.
- **`quirks.fraidycat_fear_ignores_fearless`** (default: on). A fraidycat hit by a known attacker
  while its fear level is below 100 gets a fear level of 300 even when its type is fearless or it
  has the fearless veterancy ability (claim 16). Because a fearless type never loses fear, such a
  unit then panics and scatters for the rest of its life. Off: a fearless fraidycat keeps its fear
  level unchanged, like any other fearless unit.
- **`quirks.prone_get_up_repeated_destination`** (default: on, claim 15). A destination-driven
  get-up request requires reassigning the current non-empty destination. A first or different
  destination can therefore make a prone soldier crawl instead of getting up. Off: every
  non-empty destination assignment requests get-up for an otherwise eligible unit, whether
  first, different or repeated. Clearing a destination still makes no request, and the owner,
  type exceptions and sequence refusal rules are unchanged. No request is tied to physical
  movement starting.
- **`quirks.cow_idle`** (default: on). The hard-coded cow type is singled out in two ways when it
  idles (claim 14): it takes the second draw *e*, and *e* < 5 gives the outcome of *r* = 8; and on
  the outcome of *r* = 8 it scatters after the facing change, whether or not it is a fraidycat. So
  it turns and wanders more often than other types. Off: no type is singled out by its identifier,
  and the cow is treated exactly like any other type. It takes no draw *e*, so each attempt takes
  the timer draw, *r* and, on a turn, δ, as in the "any other unit" row. On the outcome of *r* = 8
  it scatters only if its type is a fraidycat whose owner is not human-controlled, as any type
  would.
- **`quirks.first_comment_only`** (default: on). The idle comment always uses the type's first
  comment sound. Off: when the comment chance gives a comment, one more ranged draw from the
  unsynchronised random stream, uniform from 0 to *n* − 1 over the type's *n* comment sounds and
  taken right after the chance draw, picks the sound. Like the chance draw, it never touches the
  scenario random stream, because the comment concerns the local player only.
- **`quirks.speed_normalised_delays`** (default: on, shared with S-0108; glossary claim G3). The
  stage delays of *idle-a*, *idle-b*, *wet-idle-a*, *wet-idle-b*, *hover* and *cheer* are
  game-speed normalised when the sequence starts (claim 5, Formulas F3), so their length in ticks
  depends on the game speed, and so does every rule that waits on them, such as which idle-timer
  expiries lead to an idle attempt (claim 14) and the shot timing of a cheering unit (claims 6
  and 11). Off: every kind uses its base delay, *d = b* (T1), at every game speed, so every
  sequence lasts the same number of ticks at every speed.
- **Round-toward-zero mode.** F4 and F5 round every floating-point step toward zero at double
  precision (53-bit significand), not to nearest. A result that is mathematically an exact integer,
  but built from inputs that are not exactly representable, can come out one below it before the
  final truncation. This spec defines no switch for it: the mode and its single switch
  `quirks.chop_rounding` (default: the original behaviour) is owned by S-0205 (number domains,
  claim 5),
  and round-to-nearest double arithmetic is what that switch selects when off.
- **`quirks.fire_frame_by_stance`** (default: on). The fire frame follows the unit's stance and
  weapon (claim 11), not the firing kind that plays. A prone unit firing its secondary weapon with
  *secondary-fire* (because *secondary-fire-prone* has no image frames) uses the prone fire frame.
  *deployed-fire*, *fire-airborne* and *wet-fire* use the standing or prone fire frame. Off: while
  one of the four kinds that has a fire-frame field of its own is current, *F* is that kind's
  field: the standing field for *fire-standing*, the prone field for *fire-prone*, the secondary
  field for *secondary-fire* and the secondary-prone field for *secondary-fire-prone*. While any
  other sequence is current (*deployed-fire*, *fire-airborne* and *wet-fire*, none of which has a
  field of its own, *wet-fire* also replacing both standing and prone firing, and the non-firing
  *go-prone* and *prone* of the shot rules), *F* follows the unit's stance and weapon as in claim 11, as when the switch is on.
  The two settings differ only while one of those four kinds is current and the stance-and-weapon
  rule picks another field. The case the kind choice of claim 11 produces is a prone unit firing
  its secondary weapon whose request fell back to *secondary-fire*: it uses the prone field when
  on and the secondary field when off.
- **`quirks.prone_attack_state_persists`** (default: on). Only a sequence with a stage delay of 0
  ends the attack state at the start of a tick, so a prone unit stays in the attack state after
  *fire-prone* completes. It fires again, with no firing sequence, when the stage of *prone* equals the
  fire frame while the weapon is ready. This can happen at most once per start of *prone*, provided
  a shot leaves the weapon not ready on the following tick (S-0301): the stage keeps
  growing while *prone* loops, and on the later ticks of the same stage value the weapon is not
  ready, which ends the attack state (claim 11). Off: end the attack state whenever a
  firing kind completes.
- **`quirks.air_death_fall_generic_follow_up`** (default: on). A completed *air-death-fall* gets the
  general follow-up (*stand*, *walk*, *prone* or *crawl*, forced), even while the unit is still in
  the air. Off: *air-death-fall* loops until *air-death-end* replaces it.
- **`quirks.sequence_value_length_limit`** (default: on, claim 22). The art values of the sequence table are
  read as at most 31 bytes of text (Import mapping), so a long cue list loses its tail. Off: no limit.
- **`quirks.two_sound_cues`** (default: on, claim 22). At most two sound cues are kept per entry.
  Off: keep every pair whose sound is known.
- *die-3* to *die-5* are never selected (claim 17). This is not a switch, because no rule refers
  to them.
- **`quirks.refused_death_keeps_unit`** (default: on, claim 17). A killed unit whose *die-1* or
  *die-2* entry has *C* = 0 gets its death request turned down (claim 6, row 1) and is left in
  play by the death handling; what becomes of it is not established (Open questions). Off: the
  death counts as completed on the tick of the death, with the corpse or removal that gives
  (claim 17 gives the scope and the cyborg exception). The importer warns about the entry either
  way.
- **Departure: a general corpse list left over from an earlier game** (no switch, no importer
  warning). The original's pick from it is *unspecified*. Our engine shows no corpse and takes the
  pick's draw as usual (Import mapping). It gives no importer warning, because the cause lies in
  an earlier game's data and not in any input of this game.
- **Strength 0** (no switch). F5 divides by the strength. Our engine gives the original's result
  for a strength of 0 (*i* = 12) without the division, and the importer warns (Formulas F5).

## Import mapping

All keys live in the art file, in the infantry type's art section, unless noted otherwise. Which
art section belongs to the type (through its image name, or in some cases its type name) is given
by S-0007; "the type's art section" below means that section. Kind names map to keys as follows:

| Our kind | Key | Our kind | Key |
|---|---|---|---|
| stand | `Ready` | wet-die-2 | `WetDie2` |
| guard | `Guard` | wet-fire | `WetAttack` |
| prone | `Prone` | hover | `Hover` |
| walk | `Walk` | fly | `Fly` |
| fire-standing | `FireUp` | tumble | `Tumble` |
| go-prone | `Down` | fire-airborne | `FireFly` |
| crawl | `Crawl` | deploy | `Deploy` |
| get-up | `Up` | deployed | `Deployed` |
| fire-prone | `FireProne` | deployed-fire | `DeployedFire` |
| idle-a | `Idle1` | deployed-idle | `DeployedIdle` |
| idle-b | `Idle2` | undeploy | `Undeploy` |
| die-1 … die-5 | `Die1` … `Die5` | cheer | `Cheer` |
| tread | `Tread` | parachute | `Paradrop` |
| swim | `Swim` | air-death-start | `AirDeathStart` |
| wet-idle-a | `WetIdle1` | air-death-fall | `AirDeathFalling` |
| wet-idle-b | `WetIdle2` | air-death-end | `AirDeathFinish` |
| wet-die-1 | `WetDie1` | panic | `Panic` |
| shovel | `Shovel` | carry | `Carry` |
| secondary-fire | `SecondaryFire` | secondary-fire-prone | `SecondaryProne` |

- **`Sequence=`** in the type's art section names a second art section that holds the entries. Absent:
  the type keeps the all-zero table of claim 1.
- **`<kind key>=start,count,multiplier,direction`** in that section gives *S*, *C*, *M* and *D*.
  The first three are integers. The fourth is one of `N`, `NE`, `E`, `SE`, `S`, `SW`, `W`, `NW`
  (exact, case-sensitive, claim 22, `quirks.case_sensitive_direction`). When a field is missing or does not
  parse, that field and every field after it keep their defaults from claim 1 (0, 0, 0, none),
  while the fields before it take their given values. Reading the same art data again gives the
  same entries. Which of several entries with the same key counts is given by S-0002. The key name
  is matched as the art parser matches keys (S-0001). The importer warns about an entry with a
  missing or unparseable field (other than a left-out direction) and about a direction that is not
  an exact upper-case compass abbreviation, since both lose data silently in the original.
- **`<kind key>Sounds=frame sound [frame sound]`**, in the same sequence section: tokens separated
  by spaces, commas or tabs. Each pair is a cue frame (integer) and a sound name. A pair whose sound
  is unknown is dropped, and at most two pairs are kept (claim 22).
- **Value length:** the `Sequence=` value, each kind value and each `Sounds` value are read as at
  most 31 bytes of text, not 31 decoded characters (S-0003 claim 1). Anything longer is cut off before it is parsed, so a long cue list loses its
  tail (claim 22). The importer warns about any such value that is longer than 31 bytes, and about a
  `Sounds` value with more than two valid pairs.
- **Fire frames:** `FireUp` (standing), `FireProne` (prone), `SecondaryFire` (secondary) and
  `SecondaryProne` (secondary-prone) as integers in the type's art section itself, not the sequence
  section. They share names with sequence keys, but they give the stage on which the shot is
  released (claim 11). Absent: 0.
- **`Crawls`** (boolean, the type's art section): can crawl, which *go-prone* requires.
  Absent: true (the engine default for infantry).
- **Destination-driven get-up inputs (claim 15, Q-0087).** There is no imported list of orders
  that cause get-up and no imported destination-equality flag: these are engine rules.
  The existing mappings give all data inputs:

  | Key and section | Form | Absent-key default | Destination-driven get-up use |
  |---|---|---|---|
  | `Up`, sequence section named by `Sequence` | sequence entry as above | start 0, count 0, multiplier 0, no direction or cues | requested get-up kind; count 0 refuses the request |
  | `Fraidycat`, infantry rules section | boolean | false | true excludes the unit |
  | `Cyborg`, infantry rules section | boolean | false | true excludes the unit |

- **Rules keys.** The table gives each rules key this spec uses, its form and its absent-key
  default. The default is the value the field has before any rules data is read. As S-0003 says,
  when rules are layered (S-0006) an absent key keeps the value an earlier layer gave instead.
  For the type rows that value is the default in every game, because every type is created anew
  at each game start (S-0004). The four game-wide rows (`IdleActionFrequency`, `ConditionYellow`,
  `ConditionRed` and `[General]` `DeadBodies`) are set to their defaults only once, when the
  program starts, and are not reset at a game start. So the default holds in the first game of a
  session. In a later game, one of the three decimal keys that no layer of that game gives keeps
  the value an earlier game's layers left (glossary *rules database*, S-0006). The general corpse
  list also stays when no layer of the game gives `[General]` `DeadBodies`, but the animation
  types it names are those of the earlier game, which no longer exist, since animation types are
  created anew at each game start too. A corpse picked from such a list is *unspecified* (glossary): the original's list names
  animation types that no longer exist, and what a pick from it shows depends on unrelated memory
  state. Our engine's result: a general corpse list that an earlier game gave and no layer of this
  game gives counts as a list of its own length whose entries show nothing. A death that would
  pick from it takes the pick's one draw as for any list of that length, so the scenario random
  stream stays in step, and leaves no corpse; the unit is removed as usual. There is no importer
  warning, because the cause is an earlier game's data and not an input of this game.
  Booleans and decimals follow S-0003. A sound or animation name is looked up as S-0003 says for
  references by name. For the sound keys, a missing value or a sound name that is not known leaves
  the field as it was. For the three list keys, a value that is absent or empty leaves the list as
  it was; otherwise the new list replaces the old one.

  | Key | Section | Form | Absent-key default | Use here |
  |---|---|---|---|---|
  | `Fearless` | infantry type | boolean | false | fearless (Terms, claims 15, 16) |
  | `Fraidycat` | infantry type | boolean | false | fraidycat (claims 14 to 16) |
  | `Cyborg` | infantry type | boolean | false | cyborg (claims 15, 17) |
  | `NotHuman` | infantry type | boolean | false | not human (claim 17) |
  | `DeployedCrushable` | infantry type | boolean | true | stays crushable when deployed (State and ordering) |
  | `JumpJetTurn` | infantry type | boolean | false | allows jumpjet turning (claim 12) |
  | `EnterWaterSound`, `LeaveWaterSound` | infantry type | sound name | none (no sound) | water side effect (claim 7) |
  | `VoiceComment` | infantry type | list of sound names | empty | idle comment, first entry only (claim 14) |
  | `DeadBodies` | infantry type | list of animation names | empty | the type's corpse list (claim 17) |
  | `DeathAnims` | infantry type | list of animation names | empty | death animations indexed by death kind (claim 17) |
  | `JumpJet` | any unit type | boolean | false | jumpjet type (claims 10, 11, 12, 17) |
  | `NoShadow` | any unit type | boolean | false | suppresses the body-sprite shadow (claim 19) |
  | `MovementZone` | any unit type | movement zone name | `Normal` | the amphibious-destroyer value enables the water substitutions (claim 7) |
  | `Storage` | any unit type | integer | 0 | storage capacity (claim 7) |
  | `Strength` | any object type | integer | 0 | the strength of F5; the importer warns when it is 0 or below for an infantry type (F5) |
  | `Crashable` | any unit type | boolean | false | crash flag (Terms, claim 17) |
  | `DeploySound`, `UndeploySound` | any unit type | sound name | none (no sound) | played when *deploy* and *undeploy* start (claim 21) |
  | `DeadBodies` | `[General]` | list of animation names | empty | the general corpse list (claim 17) |
  | `IdleActionFrequency` | `[AudioVisual]` | decimal, game minutes | the double nearest to 0.083 | *R* in F4 |
  | `ConditionYellow` | `[AudioVisual]` | decimal with optional `%` (S-0003 claims 6 to 8) | 0.5 | *yellow* in F5 |
  | `ConditionRed` | `[AudioVisual]` | decimal with optional `%` (S-0003 claims 6 to 8) | 0.5 | *red* in F5 |
  | `InfDeath` | warhead type | integer | 0 | infantry-death kind (claim 17); kind 0 requests no death sequence, except for a type marked "not human" (claim 17, row 4) |

  "Infantry type" means the infantry type's own section in the rules file. "Any unit type" and
  "any object type" mean the same section, through keys that other kinds of type also have; their
  other uses belong to other specs. `Cyborg` also selects the cyborg death case of claim 17.
- **Hard-coded identifiers:** Claim 14 singles out the cow infantry type using the following
  data-name mapping (Q-0212). It is an engine comparison, not an absent-key default.

  | Input | Form | Match | Use |
  |---|---|---|---|
  | Infantry type name registered from `[InfantryTypes]`, naming its rules section | type identifier, as imported by S-0004 | `COW`, case-insensitive | cow-specific idle draw and scatter of claim 14 |
  | `Name`, `UIName`, `Image` in the infantry rules section | display-name or image-name inputs, owned by their respective specs | not tested | changing these cannot grant or remove the cow designation |

  No matching registered type means no unit receives this designation. Any infantry type
  with the matching registry name receives it, regardless of its art or display name.
  Claim 19 uses the generic infantry-shadow sprite for an airborne infantry unit.

## Measurement scenarios

**M19 (claim 21, slave-release cheer).** Use S-0312 claims 30 and 31's scenarios D21
and D22, including their stock-data setup, speed 6, director and mods off, and run counts.
Record the released on-map slaves' sequence requests and drawn image frames at transfer.
Expected: every successful release transfer requests *cheer*, subject to claim 6's start
rules; hidden removal and the no-recipient damage path make no such request. A successful
transfer without the request, or a release cheer request on either excluded path, falsifies
this claim. A refused request need not produce a visible cheer.

Common setup: skirmish, game speed 6 unless a part varies it, 800 × 600, director and mods off, and the glossary's measurement setup (an unmodified install with stock data, no loose
override files and no engine add-ons), a tiny flat map with clear terrain and one water and beach
strip, and a fixed game speed unless stated otherwise. Include one distant bystander computer
player unless a part already includes a computer-controlled test owner. The observed area contains
only that part's named test objects; apply the glossary's bystander void rule. Image frame numbers are read from a
screen capture that keeps every drawn frame, by matching the drawn sprite against the image frames
of the type's body sprite. Ticks are counted by a recorder that reports the tick number of each
captured drawn frame, as
the glossary's Scenario G2 assumes, not from the capture's real-time timing: how many ticks run per
second belongs to S-0201 (claims 2 and 5), and game speed 0 has no fixed tick rate (glossary claim G2,
part (b)). Where a scenario needs test art or test rules data, it is a named test file (a test
copy of the art or rules data that changes only what the scenario names, or a test map's override
section) and it is removed after the run, as the measurement setup allows. Each measurement record
names the display path it used, naming any presentation-only display compatibility layer
generically, as the measurement setup requires. Sub-tests whose result depends on the random seed
are measured statistically, over the number of runs each states. A scenario that names a type by property ("a type
whose …") means any stock type with that property; the property is a precondition, not a claim.

- **M1 (claims 1, 6, 7).** Test art, three variants, each changing one infantry type:
  - (a) its art section without the reference to a sequence section (Import mapping). Place a
    unit, move it, damage it, let it idle.
    Expected: it always shows image frame 0, whatever it does or faces. Falsified by any other
    image frame.
  - (b) its sequence section without the *go-prone* key. Damage a unit of that type, owned by a computer-controlled house, from
    a known attacker. Expected: it never lies down and keeps showing *stand* or *walk* image frames.
    Falsified if it drops prone.
  - (c) an amphibious-destroyer type whose sequence section lacks the *swim* key. Order a unit into
    water. Expected: while moving in water it shows the single image frame 0 with no animation
    (the empty *swim* is still started). Falsified if it shows *walk* image frames in water.
  - Runs: one per variant; the claims involve no random draw.
- **M2 (claims 2, 3).**
  - (a) Compass facings. Place a soldier alone at a cell centre and move it one cell in each of the
    eight compass directions, returning to the centre each time. Record the *stand* image frame
    after each outward move, before the return. Expected: *S* + *M*·*g* with *g* = 7, 6, 5, 4, 3, 2, 1, 0 for map north,
    north-east, east, … north-west. Falsified by any other group order. Runs: one.
  - (b) The skew. Place a target the soldier can attack, such as a wall segment or a building, so
    that the direction from the soldier to the target lies between 16.9° and 22.5° clockwise of map
    north (facings 3072 to 4095), where the skewed and the centred rules disagree. Example
    displacements, with map north as decreasing cell y and map east as increasing cell x: 11 cells
    north and 4 cells east (about 20.0°), 8 cells north and 3 cells east (about 20.6°), or 5 cells
    north and 2 cells east (about 21.8°), within the soldier's weapon range. Order the attack and
    record the facing group drawn once the facing has snapped (claim 11). Infantry stand at spots
    inside a cell, not always at its centre, so read both world positions off the capture and
    compute the actual direction from them. Repeat at several such directions and also at 10° and
    30°. Expected: group 6 from 16.9° to 61.9°, group 7 below 16.9°. Falsified if group 7 shows at
    a measured direction of 18.5° or more, or if the change from group 7 to group 6 lies near
    22.5°. The margin of 1.6° above the 16.9° boundary allows for the error of positions read off
    the capture; a direction measured between 16.9° and 18.5° counts for neither outcome. Runs:
    one per direction, at least three directions inside the band.
- **M3 (claims 4, 5, 8).** Make a soldier walk in a straight line for 20 cells and log its image
  frame per tick at game speeds 1, 3 and 6. Expected: the walk image frame changes every 3 ticks at
  every speed, and wraps after *C* × 3 ticks. Then let it idle until an idle action plays (claim
  14). Expected: an idle image frame changes every N(3, s) ticks, 4 at s = 1, 3 at s = 3 and 2 at
  s = 6, and an idle of *C* image frames lasts *C* × N(3, s) ticks. Finally change the game speed
  from 1 to 6 while an idle plays. Expected: that idle keeps 4 ticks per image frame, and the next
  idle uses 2. Falsified
  by a walk delay that depends on speed, by idle delays other than T3, or by a delay that changes
  in mid-idle. The recorder reads tick numbers (common setup), so speed 0 can be added as a fourth
  run with the expected idle delay 5; it is optional. Also use named test art with a *stand*
  entry of at least three image frames, no declared direction or cues, and an idle-frequency
  field long enough to outlast the observation after the first attempt. Observe 300 ticks of
  uninterrupted *stand*: expected stage 0 and the entry's first frame throughout, not a
  zero-tick completion. Any advancing *stand* frame or completion falsifies the zero-delay
  rule. Runs: one per game speed, including this stationary variant.
- **M4 (claims 6, 15).**
  - (a) Destination equality and timing (Q-0087). On a flat skirmish test map, game speed 6,
    place a soldier of the human player that can crawl and has get-up image frames, and a
    weak known attacker beyond retaliation reach. One bystander computer player stays far
    away. Director and mods are off. Hit the soldier once, then remove the attacker by a map
    trigger. Wait until *prone* plays, but give the test orders before fear falls below 50.
    From a fresh prone state with no destination, issue a move to a distant clear cell A,
    then, while the unit still crawls toward A, reissue the move to A. In separate fresh
    runs, replace the second order by a move to B, and issue a first move to the soldier's
    own occupied cell. Use scripted input and a read-only recorder to log ticks, current
    destinations, sequence image frames and positions around application boundaries.
    Expected: no destination-driven get-up on the first move, on the change to B, or on
    the first move to the occupied cell; the repeat to A requests get-up at application,
    not at the first displacement. It plays to completion before a walk image frame.
    Record any later destination reassignments separately: they, not locomotor start,
    can cause a later get-up. Falsified by a get-up on a first or different assignment
    while fear remains at least 50, by no get-up on an eligible repeat to A, or by walk
    replacing get-up before completion. Runs: three per case.
  - (c) Order families. Repeat (a) from fresh prone states with attack-move to a cell,
    attack on an out-of-reach target, area guard, route-mode destinations, scatter, enter a
    transport and capture a building. Use stock types legally capable of each order and
    stock data, game speed 6, director and mods off. Record the destination before and
    after each assignment, its tick and the sequence, including later chase and waypoint
    assignments. For each family test a first destination and, where input permits it,
    a repeat of the same destination while still prone. Expected: get-up only at a
    repeated non-empty destination assignment while eligible, never just from queuing a
    waypoint, acquiring an out-of-reach target or beginning physical movement. A same-cell
    move qualifies only if that cell is already the destination. Refused orders or
    rejected scatters make no request. Falsified by a get-up that fails the equality
    test, or by an eligible repeat with no get-up. Runs: three per legal variant.
    This records actual assignment timing, rather than assuming all orders assign at
    application. No new recorder or harness is installed or changed by this proposal.
  - (b) The cyborg exception. Order a crippled (prone) cyborg of the human player to a
    distant cell A, then reissue the order to A while A is still its destination. Record
    destination identity and image frames as in (a). Expected: neither assignment requests
    *get-up* (it keeps crawling, or shows *prone* if its *crawl* entry has no image frames).
    The repeat is essential: an ordinary soldier also makes no get-up request on a first
    assignment. Falsified by a destination-driven *get-up* on the repeat. Runs: three.
- **M5 (claims 2, 7, 10).**
  - (a) Move an amphibious-destroyer soldier from land onto beach, then water, then
  back. Expected: *swim* image frames on beach and water, *tread* when stopped there, and the enter-water
  and leave-water sounds once per transition. Falsified if a beach cell shows *walk*, or a sound
  repeats without a transition. Runs: one round trip.
  - (b) Fresh units. Place an amphibious-destroyer soldier on a water cell at the start of the game
  and another on a beach cell. Record the sounds and the drawn image frame of each over the first
  30 ticks, and, if the capture has one, the first drawn frame before the first update. Expected:
  neither unit plays an enter-water or leave-water sound before it moves (a fresh unit's first
  request plays none, claim 7). The first drawn frame, when it exists, is a *tread* image frame
  on the water cell and a *stand* image frame on the beach cell (claim 2,
  `quirks.tread_default_water_only`). Then order the water unit onto land: its leave-water sound
  plays once, at that later transition. Falsified by a sound from a fresh unit before it moves,
  or by a *stand* image frame on the water cell or a *tread* one on the beach cell in that first
  frame. Runs: three units per cell kind.
- **M6 (claims 15, 16; State and ordering).**
  - (a) Damage an idle soldier of a computer-controlled owner once with a weak attack from a known
    attacker. Expected: *go-prone* on the tick after the hit, then *prone* (with *crawl* while any move lasts: a move
    to attack back, or a scatter that S-0208 claims 11 and 15 may give, which this scenario does not
    test); and *get-up* 51 ticks after the hit (fear 100 decays to 49). Falsified by *go-prone* on the hit tick rather than the next tick, or by another time
    from the hit to *get-up*.
  - (b) The same with a fraidycat (a civilian type). Expected: scattering, and *panic* image frames
    while it runs for about 100 ticks (300 down to 199). Falsified by no *panic* image frames, or
    by *panic* lasting far longer than 100 ticks.
  - (c) Test rules data that makes that fraidycat type fearless. Hit one unit of it once from a
    known attacker, then leave it alone for 4500 ticks. Expected: it keeps scattering for the
    whole 4500 ticks and shows *panic* image frames whenever it moves (fear stays at 300).
    Falsified if
    it settles down and stands, or walks without *panic* image frames, after about 100 ticks.
  - (d) Test rules data that gives the fraidycat type the fearless veterancy ability at veteran
    rank, and a test map that places one unit of it at veteran rank. Hit it once from a known
    attacker. Expected: as (b), *panic* for about 100 ticks and then calm. Falsified if it shows no
    *panic* image frames (the ability blocking the 300), or panics for good as in (c).
  - (e) A fearless type that is not a fraidycat (stock, or test rules data), hit once from a known
    attacker. Expected: it never shows *go-prone*. Falsified if it lies down. Whether it moves is
    not tested here (S-0208 claims 11 and 15).
  - (f) Same-tick order of fear and completion (State and ordering). As (a), and once the unit
    has started *get-up*, hit it again from a known attacker before *get-up* completes. Log the
    drawn image frame per tick. Expected: after the last *get-up* image frame, exactly one tick
    shows a *stand* or *walk* image frame, and *go-prone* starts on the next tick. Falsified if
    *go-prone* starts on the tick that follows the last *get-up* image frame directly (no tick in
    between), or if more than one tick lies in between. Repeat until the second hit lands while
    *get-up* plays.
  - (g) The lie-down timing of a human-controlled owner, under the conditions of M15(b) (every
    hit from beyond the soldier's retaliation reach, so that no scatter or move to attack back
    occurs). A soldier of the human player is given a move order of 2 to 4 cells, so that the move
    ends within about 40 ticks (read from the capture), and is hit once from a known attacker 5
    ticks after it starts. Expected: no *go-prone* image frame while it still has a destination or
    movement in progress; *go-prone* starts after the move has ended if its fear level is still at
    least 50 then, and not at all if it has decayed below 50. Falsified by *go-prone* during the
    move. Runs: three.
  - Runs: three per case.
- **M7 (claim 9).**
  - (a) The snap. Use an infantry type whose *idle-a* entry declares a direction. Watch an idle
    unit of that type finish *idle-a*. Expected: its facing afterwards equals the declared
    direction, whatever it faced before. Falsified if the old facing returns. Runs: three idles
    that start from different facings. For repeated snaps, named test art gives a two-frame
    *walk* entry a declared direction different from a long straight move's direction, with
    *M* > 0 and no fear or target. Record the facing group during at least three wraps.
    Expected: after the first completion it shows the declared direction on every tick while
    *walk* stays current, including non-advance ticks. Falsified by a return to the movement
    direction between completions. Control: remove only the declared direction; its group
    should follow the move. Runs: three moves per variant.
  - (b) No snap after a kind with its own follow-up. Use a type that can deploy, whose *deploy*
    and *undeploy* entries have image frames and whose *stand* entry has *M* > 0. Test art gives the
    *deploy* and *undeploy* entries the declared direction map south (*D* = 4) and removes any
    declared direction from the *deployed* and *stand* entries, so that only the two named kinds
    declare one. Start each run from a different facing, with the unit at rest in *stand*: map
    north, map east and map west, all at least two facing groups away from map south. Deploy the
    unit, let *deploy* complete, then undeploy it and let *undeploy* complete. Record the facing
    group drawn on the tick *deploy* completes (in the *deployed* image frames, when that entry
    has *M* > 0) and on the tick *undeploy* completes (in the *stand* image frames). A resting unit can turn
    through an idle action (claim 14), and the measurement setup cannot fix the random state from
    outside, so both the test-art runs and the control runs (unmodified art, same starting facing,
    same orders on the same ticks) use test rules data that sets the idle frequency *R* (F4) so that ⌊450·*R*⌋ ticks is longer
    than the whole run, while 1800·*R* stays far below 2³¹ ticks, so that the idle timer cannot
    expire again during a run. An idle attempt can then happen at
    most once, when the unit first rests; record the drawn group just before the deploy order, and
    restart the run if it is not the intended starting facing. This also checks that a new unit's
    idle timer has already run out (claim 14): across all the runs, visible idle outcomes should
    appear at first rest (about 19 attempts in 22 are visible when the type's *idle-a* and *idle-b*
    entries have image frames); falsified if none appears in any run. The readings are taken on the two
    completion ticks only; the control runs show any turning the deployment itself does.
    Expected: on both ticks the group equals that of the control run. Falsified if group 3 (map
    south) appears on the tick *deploy* or *undeploy* completes where the control run shows another
    group. A death kind is not used here: its completion removes the unit on the same tick, so no
    facing is left to observe. Runs: one per starting facing, with the test art and with the
    control art.
  - (c) *parachute* and *shovel*. Test art gives the *parachute* entry, and in a second variant
    the *shovel* entry of an infantry type that harvests, the declared direction of a compass point
    at least two facing groups away from the unit's facing at the start (read from the first
    capture), with *M* > 0. Paradrop one soldier (a test map whose trigger paradrops it) and log
    the facing group on every tick of its fall and on landing; in the second variant let a
    harvesting unit run several shovel loops and log its facing group on every tick. Expected: the
    group never changes to the declared direction's group because of the sequence (the control
    runs with the declared direction removed show the same groups). Falsified if the group jumps to
    the declared direction on a *parachute* or *shovel* completion tick. Runs: three per variant.
- **M8 (claim 11; State and ordering, the go-prone bullet).**
  - (a) Make a soldier attack a wall and log the image frames and its facing per tick from the
    start of *fire-standing*. Expected: the facing jumps to the target direction on the start
    tick; the projectile leaves on the tick the stage equals the standing fire frame (fire frame
    × 1 tick after the start); the next firing sequence starts no earlier than one tick after the
    previous one completed. Repeat with a prone soldier (hit it first) whose weapon reloads
    quickly. Expected: an extra shot with no firing sequence when the *prone* stage reaches the
    prone fire frame. Falsified by any other delay, or if the facing turns gradually. Runs: ten
    shots standing and ten prone. Then repeat with test art that gives the standing fire frame 0,
    and with test art that gives it the value of *C* of *fire-standing*. Expected: the shot leaves
    on the tick the sequence starts, and on the tick it completes, respectively. Falsified by a
    shot on any other tick. Runs: three per variant.
  - (b) The airstrike reset. Use an infantry type that can designate an airstrike, whose
    *secondary-fire* entry has *C* ≥ 4 and whose weapon's delay between shots is longer than *C*
    ticks (stock data, or test rules and art data that give one type these properties). Let it
    designate a target and log its image frames per tick and its shots. Expected: for as long as it
    designates, the *secondary-fire* image frames keep cycling between shots; the drawn image frame
    goes back to the entry's start image frame *S* once per shot, possibly in mid-cycle, and the
    shot leaves *F* ticks after that return (*F* the fire frame of claim 11). Falsified if the image
    frames stop at the end of the entry while it still designates, by a return to *S* that no shot
    follows *F* ticks later, or by a shot with no return to *S* *F* ticks before it. Runs: one
    designation of at least three shots.
  - (c) Shooting during *go-prone*. Test art gives a soldier type a prone fire frame of 1 and a
    *go-prone* entry with *C* ≥ 3. A unit of that type is owned by a computer-controlled house
    and rests in guard, not in the attack state. An enemy unit with a weak weapon approaches within
    the soldier's range and fires at it, so that the soldier's guard behaviour picks a target by
    itself while the weak hit, from a known attacker, reaches it. For a computer-controlled owner,
    movement does not block the lie-down rule (claim 15), so any move the hit may cause (S-0208
    claims 11 and 15) does not matter here. Repeat until
    the log shows the facing snap and the first *go-prone* image frame on the same tick. Expected: no *fire-standing* or *fire-prone* image
    frame is drawn before the shot, and the projectile leaves on the tick after *go-prone* starts
    (its stage 1). Falsified by a firing kind drawn first, or by a shot on any other tick. Runs:
    three runs in which the two events fall on the same tick.
  - (d) The end of the attack state on a change of target. Use a soldier whose weapon fires again
    in fewer ticks than the *fire-standing* entry has image frames. It attacks wall A; after its
    shot and before its sequence completes, the human player orders it to attack wall B, which lies
    in a clearly different direction. Variant: instead, wall A is destroyed by a trigger in that
    interval and a new target comes within the soldier's reach a few ticks later, while the
    sequence still runs. Expected: the body facing turns to the new target at once, within the
    running sequence, and the stage of the sequence is not restarted (the same kind is requested
    again and turned down). Falsified if the facing stays on the old direction until the sequence
    completes. Runs: three per variant.
- **M9 (claim 14).**
  - (a) Timer bounds and outcome ratios. Over 50 games, leave one soldier idle for 18000 ticks per game, and log the
    ticks between visible idle outcomes and the outcomes. Only some attempts can be seen: *r* = 0
    shows nothing, and a turn to the facing the unit already has shows nothing either, which is 1
    turn in 8 once the facing is a multiple of 8192 (after the first visible turn). Of the
    attempts, 3/11 give *idle-a*, 3/11 *idle-b* and 4/11 × 7/8 a visible turn, so the visible
    outcomes come in the ratio 6 : 6 : 7. Invisible attempts,
    and the rule that an attempt goes ahead only while *stand*, guard or *tread* is current, can
    only lengthen the time between visible outcomes, so the distribution of *T* itself is left to
    F4. Falsified by two consecutive visible outcomes less than ⌊450·R⌋ ticks
    apart, or by a chi-square test of the pooled visible-outcome counts (from the first visible turn
    on) against 6 : 6 : 7 (2 degrees of freedom) that rejects at the 1% level. Runs: 50 games, as stated.
    The cow (`quirks.cow_idle`): in the same 50 games, also leave one unit of the hard-coded cow type
    (Import mapping), owned by a computer-controlled house, idle on open ground, never hurt, so that
    its fear level stays 0. Its *stand*
    entry must have *M* > 0 so that turns can be seen (test art gives it one otherwise). Count the
    attempts that end in a scatter (the cow wanders off) and the plain visible turns with no
    scatter; its *idle-a* and *idle-b* outcomes are not counted, because they depend on the cow's
    art. A scatter follows 61/121 of the attempts (*e* < 5, or *r* = 8) and a plain turn 18/121, of
    which between 7/8 and all are visible, depending on the facing the previous scatter left. So the
    scatter share of the counted outcomes lies between about 0.77 and 0.80, whether or not the cow
    type is a fraidycat. Were the cow treated like any other type (the switch off), it would be about
    0.25 to 0.28 (a scatter on 1/11 of the attempts against 3/11 turns) if the cow type is a
    fraidycat, and 0 otherwise. Falsified if the 99% confidence interval of the pooled share lies
    wholly outside 0.76 to 0.81.
  - (b) Streams, number and order of the draws, with a *recorder* (glossary) that reads the random
    streams' state. Map: a skirmish test map with the local player, one distant bystander computer player,
    and test units of the neutral house, which is not human-controlled and runs no computer
    player. The observed area is the isolated clear area containing only the named test units.
    Apply the glossary's bystander void rule. Log and predict the bystander's random draws
    separately under the owning specs; a tick whose unrelated draws cannot be accounted for
    is void for the stream comparison. Control: for each case, a control game on the same map with the same
    houses and orders but without the case's unit (and, in case (2), without its attacker), run for
    300 ticks with the recorder on. Expected in the control game: no scenario-stream advance beyond the separately accounted
    bystander and other unrelated draws. Any unexplained advance voids the control and must
    be resolved before the case is run; a different seed is predicted from its own recorded
    state, never assumed to reproduce the control's raw values.
    The unsynchronised random stream may advance through presentation uses such as music or
    object flicker (S-0202 claim 11). The control identifies these uses; its per-tick advance is
    not a numerical baseline for a differently seeded run. In case (3), predict the idle-comment
    draw from that run's recorded state and account separately for identifiable presentation
    draws. Leave a tick out of the comparison if those unrelated draws cannot be accounted for,
    for example during a music-track change whose intervening draws are not isolated. In separate games,
    the test map then places: (1) one unit of the hard-coded cow type, owned by the neutral house,
    never hurt; (2) one unit of a fraidycat type, owned by the neutral house, after test rules data
    has made the type fearless and it has been hit once, as in M6(c), so that its fear level stays
    300, with the attacker removed after the hit and the recording counted from that
    removal; (3) one soldier of the local player whose type has at least one comment sound, left unselected,
    and then the same soldier kept selected. The test map gives each of these units *guard* as its
    initial behaviour. Record the scenario random stream's position and state at every tick, in
    case (3) also the unsynchronised random stream's. From the recorded state, predict each
    attempt's draws under claim 14: (1) the timer draw, *r*, *e*, then δ when the outcome is a
    turn, plus the draws of the scatter that follows when *e* < 5 or *r* = 8 (S-0208 claims 11 to
    13); (2) the timer draw only, plus the draws of its scatter; (3) the timer draw, *r*, then δ on
    a turn, and, when the outcome is *idle-b* and the unit is not selected, one ranged draw over 0
    to 2 from the unsynchronised stream, with a comment exactly when it gives 0. A ranged draw can
    take more than one raw value from its stream; the prediction counts raw values, as S-0202
    claim 4 gives them for each ranged draw from the recorded state, rejected values included.
    In case (2), the flee rule of claim 15 also scatters the unit between attempts; predict those
    scatters' draws as S-0208 gives them. Every processing of the unit's behaviour, whether or not
    it attempts the idle action and whether or not the attempt goes ahead, also takes the draw of
    that behaviour's delay, after any draws of the attempt; predict it, and the ticks of the
    processings, from S-0215's delay rule for *guard* (claims 6 and 24). Expected: on every non-void tick the
    recorded advance of the scenario random stream, less accounted unrelated draws, equals the prediction, and that of the
    unsynchronised stream equals the prediction plus its accounted presentation draws; the visible outcomes match the
    predicted *r*, *e* and δ; the timer expires after the length F4 gives from the predicted timer draw, and the next
    eligible behaviour processing makes the next attempt; the comment heard is always the type's first comment sound; a selected unit takes
    no draw from the unsynchronised stream at its attempts. Falsified by any other advance (for
    example an *r* draw in case (2), no *e* draw in case (1), or an unsynchronised draw from a
    selected unit), by an outcome that differs from the prediction, or by an attempt before expiry or at a different eligible processing. Runs: one game per case, until at least 30 attempts of the unit have been recorded. Timer
    expiry only enables an attempt: if expiry falls between behaviour processings, or the unit
    is moving or playing an ineligible sequence then, the next attempt is the first later
    processing that meets every eligibility condition of claim 14, not necessarily the expiry tick.
  - (c) The *hunt* trigger. A skirmish test map (not a campaign game) places one soldier of the
    human player with *hunt* as its initial behaviour, and test rules data gives its type the
    stupid-hunt flag (S-0215 claim 27 and Import mapping), so that its hunt never keeps a target
    and, outside a campaign game, only attempts the idle action. Every enemy object stays far
    outside the soldier's weapon range and sight. Leave it for 18000 ticks and log its image frames,
    its facing and its behaviour. Expected: it stays in *hunt* and shows idle outcomes (*idle-a*,
    *idle-b* image frames or turns of facing), never two visible outcomes less than ⌊450·R⌋ ticks
    apart. Falsified if no idle outcome appears in the 18000 ticks while the unit stays in *hunt*.
    Runs: three games.
- **M9(d) (claim 14, Q-0212).** On a tiny flat skirmish map at game speed 6, with one
  distant bystander computer player, director and mods off, use named test rules data
  to register an invented infantry type under the designated cow identifier of Import
  mapping and a copy of the stock cow under an unrelated invented identifier. Give both
  to the human player, keep them unselected, unhurt and without targets, and use clear
  open areas where scatter is legal. Keep the invented type's display and image names
  unrelated to the cow identifier. For the renamed copy, test separately retaining the
  cow's original display and image names and setting those inputs to the designated
  identifier. In separate runs change only the letter case of the matching registry name.
  Record idle attempts, positions and the scenario random stream with the read-only
  recorder, accounting separately for scatter-destination draws (S-0208).
  Expected: only the matching registry name takes the extra idle draw *e* and the
  owner-independent scatter on outcome 8, in every letter-case variant. In particular
  draws satisfying *e* < 5 force outcome 8 for that type. The renamed copy takes no
  *e* draw and does not scatter on outcome 8 under this owner, whatever its display or
  image name. Falsified by either effect following a display or image name instead, or
  by a letter-case variant losing the designation. Runs: at least 100 eligible idle
  attempts per variant over at least three games; remove the named test data afterwards.

- **M10 (claim 17).**
  - (a) Kill soldiers standing on a land cell with warheads of infantry-death kinds
  1 and 2. Expected: *die-1* and *die-2*; the corpse appears and the unit disappears *C* ticks after
  the death starts. Kill a soldier the same way with a warhead of kind 9. Expected: either *die-2*
  followed by a corpse, or removal at once with no death sequence (the replacement effect of row 7,
  which another spec owns; record which of the two happened). Kill a soldier with a laser-fence
  building. Expected: kind-5 handling (no *die-1* or *die-2*). Kill a jumpjet soldier with a warhead
  of kind 1. Expected: no death sequence. Kill units of a type marked "not human" that is not a
  jumpjet or cyborg type and has no death-animation list (stock, or test rules data), standing on a
  land cell, with warheads of kinds 0, 2 and 8. Expected: *die-1* in all three cases, and with kind
  8 also the extra animation. Kill paratroopers while they still fall under their parachutes (a
  test map whose trigger paradrops them, and a weapon or other damage that reaches them in the air): first of an
  ordinary soldier type, then of the "not human" type above. Expected: the ordinary paratrooper
  disappears at once with no death sequence (death kind 3, row 8); the "not human" one shows no
  *die-1* image frame while it falls and goes on showing *parachute* image frames. What becomes of
  the "not human" unit afterwards (whether it lands, stays or is removed, and when) is recorded,
  with no expected value (Open questions). Falsified by any other result, in particular by *die-1*
  or any sequence other than *die-2* for kind 9, by *die-2* or no sequence for the not-human type on
  land, by a *die-1* image frame from the falling not-human paratrooper, or if *die-3* to *die-5*
  image frames ever appear. Also kill an amphibious-destroyer soldier with a warhead of kind 1, 2 or
  9, once on a beach cell and once on a water cell. Expected on the beach: *wet-die*. The water-cell
  result is only recorded: it has no expected value, because it is meant to settle the open
  condition of claim 17 (low confidence). Runs: three kills per case; the corpse pick is the only death-specific draw in these cases; unrelated behaviour draws
  are accounted for separately.
  - (b) The default of `quirks.refused_death_keeps_unit` (claim 17). Test art empties the *die-1* entry of
    one infantry type (*C* = 0). Kill a unit of it on a land cell with a warhead of infantry-death
    kind 1. In a second run, empty the *die-1* entry of the "not human" type used above and kill
    one of its units while it falls under a parachute. Log the drawn image frames and whether the
    unit is still in play (selectable or attackable) for 3000 ticks after the death, and whether the
    game keeps ticking and answering input. Expected: no death image frame, and the unit still in
    play after the death tick. What it does afterwards, whether it is ever removed and whether
    the game stays responsive are recorded with no expected value, since they decide the
    aftermath (Open questions). Falsified if the unit is removed on the
    tick of its death or a death image frame appears. Runs: three kills per case.
  - (c) A cyborg killed by damage that ignores defences (claim 17, cyborg paragraph). On a land cell,
    a standing (not prone) soldier of a cyborg type without the crash flag carries a tag whose
    trigger fires the action that destroys attached objects (S-0011 claim 27, scenario T9). Test
    rules data sets the infantry-death kind of the demolition warhead of that action to 1. Control:
    the same unit with test rules data that clears the cyborg flag. Expected: the cyborg is removed
    on the tick of the damage with no *die-1* image frame and no corpse; the control shows *die-1*
    and then leaves a corpse (or none, if its type has none). Falsified if the cyborg shows a
    *die-1* image frame or leaves a corpse, or if the control shows no *die-1*. Runs: three per
    case.
  - (d) Corpse list and pick. Start each variant in a fresh program session. Named test rules
    replace all definitions of the chosen corpse lists, rather than trying to clear an existing
    list with an empty override value (Import mapping). Give a land soldier a three-entry corpse
    list of visibly distinct stock animations. Kill it with infantry-death kind 1 and record the
    scenario random stream around completion, as in M9(b). Predict the next unsigned raw
    draw *x*, accounting for unrelated draws, and record the corpse. Expected: entry
    *x* mod 3, exactly one raw draw and removal on the completion tick. Repeat with the
    type's list empty and the same three entries in the general list; then with both lists
    empty. Expected: the general list supplies the first variant's pick when the type's
    list is empty; with both empty, no corpse and no corpse-pick draw. Any different pick,
    extra raw draw or failure to remove falsifies the claim. Runs: ten kills per variant.
  - (e) Type death animations. Named test rules give the soldier above a list with distinct
    entries for kinds 0, 1 and 2. Kill it with kinds 1, 2 and an integer outside that list.
    Expected: the indexed animation for 1 and 2, entry 0 for the outside kind, no body death
    sequence and removal on the death tick. Repeat after marking the type not human:
    the list still takes precedence over *die-1*. Any body death sequence, different list
    entry or delayed removal falsifies this rule. Runs: three per variant.
- **M11 (claim 19).** Screenshot a soldier standing on the ground, one standing on a bridge deck,
  and a jumpjet soldier hovering. Expected: the two standing soldiers show their own shadow, image
  frame *frame* + ⌊L/2⌋. The hovering one has the generic shadow on the ground below it, drawn as
  image frame 1 of the generic infantry-shadow sprite (matched against that sprite's image frames
  in the capture), and no body-sprite shadow. Falsified if either standing soldier shows no body-sprite shadow or a
  shadow image frame other than *frame* + ⌊L/2⌋, or if the hovering one shows a body-sprite
  shadow, no generic shadow or a generic shadow image frame other than 1. Runs: one screenshot per case. Repeat the ground case with named test rules that suppress
  the type's shadows: expected no body-sprite shadow, falsified by that shadow still drawing.
  Runs: three screenshots with and without the flag.
- **M12 (claim 12).** Use a jumpjet infantry type that does not allow jumpjet turning and whose
  weapon has a long delay between shots. Let a hovering unit of that type attack a vehicle that is
  driven in a circle around it, within range. Expected: between shots, the drawn facing group
  follows the direction to the vehicle tick by tick, although the body facing changes only when a
  shot snaps it (claim 11). Repeat with test data that allows jumpjet turning. Expected: the drawn
  group then follows the body facing. Falsified if, in the first run, the drawn group changes only
  at the shots. Runs: one circuit of the vehicle per data set.
- **M13 (claim 13).** Use an infantry type that can disguise, disguised as a type whose walk entry
  has a different frame count. Watch it walk from the view of an enemy player, the local player.
  Expected: its image frames are those of the disguise type's walk entry, wrapping after the
  disguise type's frame count. From its owner's view, its own image frames show. Falsified if the
  enemy sees the disguised unit's own image frames, or a wrap at its own frame count. Runs: one walk of at least
  two wraps per view.
- **M14 (claim 18).** Test art for one type: a *walk* cue with cue frame 1 (with *C* ≥ 2) and a
  *stand* cue with cue frame 0. Expected: while walking, the walk cue sounds once per *C* × 3 ticks. The
  *stand* cue sounds once on the tick a completed one-shot hands over to *stand*, for example when
  *fire-standing* completes, and never when the unit simply stops walking. Falsified by a *stand*
  cue after a plain stop, or by a missing cue after a completed one-shot. Runs: five stops and five
  completed firing sequences.
- **M15 (claim 16, F5).**
  - (a) Known attacker. A soldier of a computer-controlled owner is hit by two attacks that land
    on the same tick, for example from two identical attackers placed at the same distance and
    ordered together; the health bar shows whether both hits landed on one tick. The first hit sets
    the fear level to 100 and the second adds *i*. Record the ticks from the hits to *get-up*, which
    are 51 + *i*.
    - First configuration, yellow ratio above the red one (test rules data sets them so if the
      rules data in use does not). Run it with the soldier's health ratio after the hits above
      the yellow ratio, between the red and the yellow ratio (above red, at or below yellow), and
      at or below the red ratio. Expected: 63, 76 and 101 ticks (*i* = 12, 25 and 50). Falsified
      by other durations, in particular 76 above the yellow ratio (the halving missing).
    - Second configuration, red ratio above the yellow one (test rules data). Run it with the
      health ratio after the hits above the yellow ratio and at or below the red ratio. Expected:
      76 ticks (*i* = 25: 50 for being at or below red, halved for being above yellow). Falsified
      by 63 ticks, which would mean a chained rule (12 above yellow, else 25 above red, else 50)
      instead of F5's, or by any other duration. The first configuration cannot tell these two
      rules apart, because they agree whenever the yellow ratio is at or above the red one.
    - Runs: three per health band in each configuration.
  - (b) No known attacker. First pick a damage source that carries no attacking object (which
    sources do belongs to S-0302, claim 1; a radiation field is the expected candidate). Every
    soldier in this part belongs to the human player, rests with no movement order, and every hit
    from a known attacker comes from beyond the soldier's retaliation reach (S-0215, Formulas F7),
    so that with default
    rules neither a scatter from damage (S-0208 claims 11 and 15) nor a move to attack back moves
    it, and the lie-down rule of claim 15 applies to it. Check the source on such a soldier whose
    fear level is 0: with no known attacker one hit does not make it lie down (claim 15: *i* ≤ 50
    gives *f*′ ≤ 49), while a hit from a known attacker does. Then hit such a soldier from a known
    attacker on tick *h*₁, which sets its fear level to 100, and expose it to the chosen source so
    that it takes its next hit or hits on later ticks, while it is still prone and at most 50 ticks
    after *h*₁. Read each of these hits and the health ratio after it from the health bar, and take
    *i*ⱼ from F5 for hit *j*. Expected: *get-up* on tick *h*₁ + 51 + Σ *i*ⱼ, over the hits that
    landed before it. Falsified by *get-up* at another tick; in particular, if the first such hit,
    on tick *h*₂, were treated like a hit from a known attacker, it would reset the fear level to
    100 and *get-up* would come on tick *h*₂ + 51 (choose *h*₂ − *h*₁ different from *i* so that
    the two outcomes differ). Runs: three, with single hits where the source allows them.
- **M16 (claim 20).** Place one unit of each stock infantry type whose guard, *tumble* or
  *deployed-idle* entry differs from its *stand* entry. Play them through idling, deploying and
  combat for 9000 ticks. Expected: no image frame that belongs only to guard, *tumble* or
  *deployed-idle* ever appears. Falsified if one does. Runs: one game of 9000 ticks.
- **M17 (claim 7, carrying and hovering; claim 10, the hover and fly split).**
  - (a) Carrying. Use an infantry type with a non-zero storage capacity that works enslaved to a
    master, and whose *carry* entry has image frames. Log a unit of it through several harvesting
    trips. Expected: it shows *carry* image frames while it walks back to its master full, and
    *walk* image frames while it walks out empty. Then order one away before it is full. Expected:
    *walk*. Falsified by *walk* on a full return trip, or by *carry* when it is not full.
    Runs: five trips, and two early recalls.
  - (b) Hovering. Use a jumpjet infantry type. Test art gives its *hover* entry *C* ≥ 2 and a
    start image frame of 1 or more. Let a unit of it stop in the air. Expected: *hover* image frames
    while it is stopped in the air. Repeat with
    test art that gives the *hover* entry a start image frame of 0 and *C* ≥ 2. Expected: *stand*
    image frames while it is stopped in the air (`quirks.hover_needs_nonzero_start`). Falsified if
    the first run shows *stand* while stopped in the air, or the second shows *hover* while
    stopped in the air (in flight, claim 10 can still ask for *hover* directly). Runs: three stops per art
    variant.
  - (c) The *hover* and *fly* split (claim 10). Use a jumpjet infantry type whose *hover* and *fly*
    entries have image frames that differ. Order a hovering unit of it, not in the attack state, on
    a long straight flight and let it stop at the end. From the per-tick positions read off the
    capture, compute its speed on each tick and its full speed (the steady speed in mid-flight).
    Expected: *hover* image frames while the speed is at most 0.8 of the full speed, while it speeds
    up and slows down, and *fly* image frames above that. Falsified by *fly* image frames at a
    speed clearly below 0.8 of the full speed (0.7 or less), or *hover* image frames at a clearly
    higher one (0.9 or more), allowing for the error of positions read off the capture. Runs: three
    flights.

- **M18 (claim 22).** Test art for one infantry type whose *stand* and *idle-a* entries have *M* > 0
  and image frames, and whose *walk* entry has *C* ≥ 4. Each variant changes only what it names.
  The sounds named below are stock sounds whose names are short enough for the stated lengths.
  - (a) Direction case and kind-value length. Variant 1 gives the *idle-a* entry the declared
    direction map south written in upper case. Variant 2 writes the same direction in lower case.
    Variant 3 writes it in upper case but gives the facing multiplier leading zeros, so that the
    multiplier ends exactly at the 31st byte of the value and the direction lies beyond it.
    Place one idle unit per variant and watch its *idle-a* completions, each starting from a
    facing group other than 3. Record the facing group drawn just before and just after each
    completion. Expected: in variant 1 the group after every completion is 3 (map south). In
    variants 2 and 3 it equals the group before the completion. Falsified if variant 2 or 3
    changes the group at a completion, or variant 1 does not show group 3. Runs: five *idle-a*
    completions per variant; idle attempts are random, so wait for them.
  - (b) Two cues. Give the *walk* entry a cue value of three pairs, at cue frames 1, 2 and 3, with
    three different sounds, the whole value within 31 bytes. Make the unit walk for five walk
    cycles. Expected: the first two sounds play once per cycle; the third never plays. Falsified
    if the third sound plays.
  - (c) Cue-value length. Variant 1 gives the *walk* entry a cue value of two pairs, at cue frames
    1 and 3, with two different sounds, within 31 bytes. Variant 2 is the same value with
    commas inserted between the two pairs, so that the second pair begins after the 31st
    byte. Make a unit of each variant walk for five walk cycles. Expected: variant 1 plays
    both sounds once per cycle; variant 2 plays only the first. Falsified if variant 2 plays the
    second sound, or variant 1 does not. No cue sits at cue frame 0, which a sequence started by
    the movement rule skips on its start tick (claim 18).
  - (d) Unknown sound. Give the *walk* entry a cue value of three pairs at cue frames 1, 2 and 3,
    whose first pair names a sound that does not exist and whose other two pairs name two
    different stock sounds, the whole value within 31 bytes. Make the unit walk for five walk
    cycles. Expected: the second and third sounds play once per cycle (the unknown pair is
    dropped and does not count towards the two); nothing plays at cue frame 1. Falsified if the
    third sound does not play, or a sound plays at cue frame 1.
  - Runs: as stated per part; the cue checks of parts (b) to (d) take no random draw.
  - (e) Sequence-section reference. Named test art supplies a 32-byte ASCII section name and two
    sections: its 31-byte prefix gives distinctive *stand* image frames, while the full name
    gives different ones. Place a stationary soldier using that reference. Expected: the prefix
    section's frames. Control: a reference of exactly 31 bytes selects that same section.
    Falsified by the full-name section's frames or by failure of the control. Runs: three per
    variant. ASCII makes the byte count explicit; no decoded-character limit is assumed.

**Cross-spec scenario alignment confirmed.** S-0202 scenarios R5(a) and R6(a) count the
behaviour-processing delay draws (S-0215 claims 6 and 24), and R5(a) distinguishes idle-timer
expiry from the first eligible later processing. Their setup includes a distant bystander computer
player and accounts for its unrelated draws. These scenario corrections do not change S-0202's
generator or F4's constant.

## Open questions

- With a fire frame larger than *C*, the firing kind completes before the shot. The
  unit then stays in the attack state only while its next sequence has a non-zero delay (claim
  11), so a standing unit would never fire. Stock data may never do this; not measured.
- The per-object condition of the water death case and of the cyborg case (claim 17), and the
  neighbouring-cell condition of the generic shadow over a bridge (claim 19), are not identified.
- A killed unit whose death request claim 6 turns down while it falls under a parachute (a type
  marked "not human", claim 17), or because its entry has no image frames
  (`quirks.refused_death_keeps_unit`), is not removed by the death handling and keeps its current
  sequence (for a falling parachutist, *parachute*). What becomes of it afterwards (whether it
  lands and stands with no health, and whether or when it is removed) is not established.
  Scenario M10(b) records it.
- Whether a unit whose type has a strength of 0 can take damage and survive, so that F5's
  strength-0 rule is ever reached, is not established. Stock data may never do this.
