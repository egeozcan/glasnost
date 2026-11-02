---
id: S-0109
title: Animation playback
topic: rendering
revision: 42
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0046, Q-0418, Q-0770, Q-0771, Q-1425, Q-1716, Q-1717]
supersedes: []
---

# S-0109: Animation playback

## Summary

How an animation plays its sprite over time (stage delay, random rates, game-speed normalisation,
passes, reversed and ping-pong play, loop delays, follow-up types, its end, and the start and middle
events tied to its stages), which sprite its derived playback fields come from, and which image
frame it draws and how (draw point, stacked copies, translucency, shadow, detail level, attachment
to another object). Render layers and depth are S-0102's, palettes and brightness S-0103's and
S-0104's, building slot animations S-0108's, and sprite lookup and the theater refresh S-0007's.
This spec covers skirmish. Network games and campaigns are out of scope. Claim 24 preserves a
deferred fog rule only. Burning-walker playback exceptions belong to S-0615 claims 4 to 11; infantry
creation and placement retries at playback end belong to S-0611 claims 9 to 11.

**Hand-off.** Creation-time sound and resource chain reaction are enabled; their suppression
applies only to later start events. Under S-0401 claim 20 and scenario E17, trigger-played
suppression is limited to later start events and does not suppress the creation-time chain
reaction. Under the crater request's taking 6 bales (S-0401 claim 3; S-0202 claim 15 and scenario
R14), a resource cell at stage 6 becomes stage 0, not empty. S-0202 claim 15 completes the
downstream particle creation and smudge selection and placement draw rules during map loading and
play, including trigger-created animations and later playback middle events; those effects are not
draw-free. This spec retains ownership of event timing, sprite-derived μ and the height gate.
Subsequent particle updates remain outside this hand-off, not draw-free; S-0011 scenario T21 does
not establish particle or smudge draw counts.

## Terms

*Tick*, *drawn frame*, *image frame*, *sprite*, *detail level*, *game speed* (index *s*), *scenario
random stream*, *draw*, *ranged draw*, *fog of war*, *departure*, *importer warning*, *quirk
switch*, *art database*, *type name*, *local player*, *house*, *trigger* and *trigger action* are
used as in the glossary. *Render layer* and *slot animation* are as in S-0102 and S-0108, the slot
kinds *freeze-unpowered* and *light-unpowered* (scenario AN16) as in S-0108, *waypoint marker* as in
S-0009, *type read*, *theater refresh*, *theater flag* and *new-theater flag* as in S-0007, *turn
pass* and *update order* as in S-0203.

| Term | Meaning |
|---|---|
| stored rectangle | The image frame's stored pixel rectangle inside the sprite canvas, including transparent pixels within that rectangle. Its dimensions can differ between frames; centred placement still uses the full canvas (S-0102 claim 19). |
| animation | An object whose type is of the animation type kind (glossary *type kind*). Its type's fields come from the art database (S-0006, claim 8). |
| *N* | The number of image frames in a sprite. Claim 25 says which of the type's sprites each derived value uses. |
| sprite end *E*<sub>s</sub> | *N*, or ⌊*N* / 2⌋ for a shadow type (claim 10): the value that an absent end field takes (claim 25). |
| start field *a*, end field *E*, loop start λ<sub>s</sub>, loop end λ<sub>e</sub>, loop count *L* | The type's imported playback fields after their absent-key defaults are applied (Import mapping, claim 25, Formulas F2). *a*, λ<sub>s</sub> and λ<sub>e</sub> are used as image-frame numbers; *E* is used as a count of stages from *a* (claim 6). |
| middle image frame μ | ⌊*N* / 2⌋ of the sprite that claim 25 names (claim 14). No field sets it. |
| stage *k* | An integer per animation, the position in its playback. The image frame shown is *a* + *k* (claim 1). |
| step σ | +1 or −1: the amount added to *k* at each advance. |
| advance | One change of *k* by σ. |
| stage delay *d* | The number of ticks between two advances, an integer ≥ 0 (claims 2 to 4). |
| pass | One run of the stage from its starting value to its end limit (claim 6). |
| remaining passes *r* | An integer per animation: from 0 to 255, where 255 means endless (claim 7). With `quirks.anim_loop_count_wrap` off it is any non-negative integer, or endless. |
| repeat factor *m* | An integer that the creator of an animation may give with it (default 1): the number of times the type's passes are repeated. A factor below 1 counts as 1. |
| loop delay | A count of updates during which an animation does not advance, after its creation (when its creator asks for one) or between two passes (claim 11). |
| update | The processing an animation receives in its turn (S-0203, Terms): animations take turns and enter the update order of S-0203 when they are created (S-0203, claims 2 to 4). So an animation receives one update per tick while it exists, except a tick whose turn a removal costs it (S-0203, claim 10) and a tick in which it is created after the turn pass. |
| start event, middle event | Effects tied to playback (claims 13 and 14): the start event normally plays the type's start sound and the type's resource chain reaction, and the middle event creates the type's particles, scorch marks and craters. A creation start event happens at creation with no loop delay; later start events happen at loop-delay ends or follow-up switches. A trigger-played animation suppresses sound and chain reaction only at those later events (claim 13). Those effects belong to their own specs; this spec says when they happen. |
| follow-up type | The type named by the animation type's follow-up field (claim 15). |
| current type | The type an animation plays as: its creation type, or the follow-up type after a follow-up switch, claim 15. |
| reversed | An animation is reversed while its current type has the reverse flag or its creator asked for reverse play. The test is made against the current type, so it can change when the animation switches to a follow-up type, claim 15. The step σ is separate state. |
| shadow type | A type with the shadow flag: the second half of its sprite holds the shadows of the first half (claim 10). |
| bouncing type | A type with the bouncer flag or the meteor flag. Such an animation follows its own motion, and its landing ends it (the specs of projectiles and debris own both). Until then its stage plays by claims 5 to 16 like any other animation's, but its advances produce no middle event (claim 14) and no damage by claim 17. The middle event that the start event produces when μ = 0 (claim 13) still happens. |
| burning walker | An animation whose current type has the burning-walker flag. S-0615 claims 4 to 11 own its running image frames, frozen running stage, expiry timing and removal, overriding ordinary playback here. |
| trigger-played animation | An animation created by a trigger action that plays an animation at a waypoint marker, including the action that shows the drop-zone animation (S-0011 claim 36). Its creation start event has the usual effects, including start sound and resource chain reaction when enabled. Later start events have neither sound nor chain reaction, and it delivers no damage (claims 13 and 17). |
| erasing weapon | A weapon that removes its target over time instead of damaging it (the temporal weapon of the combat specs, which own it). |
| owner | The object an animation is attached to, if any (claim 22). |
| translucency code | 0, 1, 2 or 3: opaque, 25 %, 50 % or 75 % translucent drawing (claim 20). Two codes combine to the larger of the two, except that 1 and 2 combine to 3. |
| translucency level *t* | An integer per animation from 0 to 16, 0 unless its building sets it (claim 20). |
| building fade level | An integer from 0 to 15 for drawing a building see-through. Under S-0516 claim 18 it rises by one per tick while cloaking and falls by one per tick while uncloaking: a full transition takes 15 ticks each way. This spec uses it only as the source of *t* (claim 20). |
| cloak | To become invisible to other players over a few steps. S-0516 claim 18 owns building transitions and claim 28 owns their appearance to each viewer. |
| sensors | Coverage of cells by which a house sees cloaked objects in them (S-0516 claim 21). |
| cloaked from the viewer | For this spec's slot-animation rule, a building at fade level 15 that is not drawn for the local player under S-0516 claim 28. It is instead drawn see-through when the local player owns it, senses its cell, or is mutually allied with its owner. This term selects *t* in claim 20; it changes only drawing, never game state. |
| being erased | A building is being erased from the moment an erasing weapon locks onto it until the last erasing weapon that locked onto it lets go, or until it is removed. S-0514 claims 4 and 10 own weapon lock-on and release. Claim 21 here covers only slot animations; S-0514 claim 27 owns the building body's erasure translucency. |
| new-theater type | An animation type with the new-theater flag and without the theater flag. Only such a type gets a sprite again, with letter substitution, at the theater refresh (S-0007, claim 14). |

## Behaviour

1. **Image frame and stage.** At every drawn frame an animation shows image frame *a* + *k*,
   except for the burning-walker image selection of S-0615 claim 4. A
   new animation starts with *k* = 0 and σ = +1, unless it is reversed (claim 8). Stage timing
   follows ticks, not drawn frames: *k* changes only in updates. An image frame index outside 0 to
   *N* − 1 draws nothing, neither the image nor its shadow (claim 10), and is not an error: the
   animation keeps playing.
   - Backing: `measurable` (scenario AN1)
   - Confidence: high

2. **Stage delay from the rate field.** The type's rate field ρ gives image frames per game minute.
   The stage delay is *d* = ⌊900 / ρ⌋ ticks when ρ > 0 and *d* = 0 when ρ ≤ 0, except −1 (Formulas
   F1). A type without the field has *d* = 1. The value −1 counts as not given and keeps the value
   held (S-0006, claim 3). A delay of 0 means the animation never advances: it keeps its initial
   stage and never ends through playback. Removal by an owner or creator, or landing of a bouncing
   animation, can still end it. Consequently a rate above 900 also freezes the animation, since ⌊900
   / ρ⌋ = 0 (`quirks.anim_fast_rate_freezes`, Edge cases).
   - Backing: `measurable` (AN1, AN2) for positive rates and the absent field. Zero,
     negative and −1 layered rate cases are `unmeasured` here: neither scenario supplies
     those values in successive type reads.
   - Confidence: high

3. **Random rate.** A type can give a random rate as a pair (ρ<sub>1</sub>, ρ<sub>2</sub>). Each
   value is converted like the rate field (F1) into *d*<sub>1</sub> and *d*<sub>2</sub>, except
   that −1 leaves that end as it was. The pair then gives the range [*d*<sub>1</sub>′,
   *d*<sub>2</sub>′] with *d*<sub>2</sub>′ = max(*d*<sub>2</sub>, 0) and *d*<sub>1</sub>′ =
   min(*d*<sub>1</sub>, *d*<sub>2</sub>′) (Formulas F1). When *d*<sub>1</sub>′ and *d*<sub>2</sub>′
   are not both 0, the animation's stage delay is a ranged draw from the scenario random stream over
   [*d*<sub>1</sub>′, *d*<sub>2</sub>′] ticks instead of the value of claim 2. When *d*<sub>1</sub>′
   = *d*<sub>2</sub>′, no number is taken from the stream and the delay is that value. Because a
   larger rate gives a shorter delay, a pair written in ascending order (ρ<sub>1</sub> <
   ρ<sub>2</sub>) converts to *d*<sub>1</sub> ≥ *d*<sub>2</sub>, so the range collapses to the
   constant delay *d*<sub>2</sub>, with no draw (`quirks.anim_random_rate_collapse`). Only a
   descending pair gives a random delay. The delay is chosen when the animation is created and again
   when it switches to a follow-up type (claim 15), never at a new pass.
   - Draws. A creation or follow-up switch whose range has *d*<sub>1</sub>′ ≠ *d*<sub>2</sub>′
     takes exactly one ranged draw over [*d*<sub>1</sub>′, *d*<sub>2</sub>′] and no other draw for
     the delay; the raw values that ranged draw consumes follow the range rule of S-0202 (claim 4),
     rejected values included. With *d*<sub>1</sub>′ = *d*<sub>2</sub>′ the delay takes no raw
     value.
   - Backing: `measurable` (scenario AN3) for the delays and, through a recorder that reads the
     state of the scenario random stream, for the number and order of creation-delay draws. AN25
     tests random follow-up delays, including their own-turn placement. The −1 layered endpoints
     remain `unmeasured`: neither scenario supplies them. The absence of a new-pass reroll is
     `measurable` through AN3's two-pass variant.
   - Confidence: high

4. **Game-speed normalisation.** If the type is marked normalised, the delay chosen by claim 2 or 3
   is replaced by N(*d*, *s*), the game-speed normalisation (S-0107, Formulas F3), with *s* the
   game speed at the tick the delay is chosen. A later change of game speed does not change the
   delay of a running animation. This is one of the speed-normalised delays of glossary claim G3.
   With `quirks.speed_normalised_delays` off, the delay stays *d* at every game speed.
   - Backing: `measurable` (AN4) for choosing the delay at each speed. Keeping the chosen
     delay after a mid-animation speed change is `unmeasured` here: AN4 changes speeds
     between runs, not during playback.
   - Confidence: high

5. **When the stage advances.** The animation's first update is the first turn it actually
   receives after being created. It may be in the creation tick itself. That update produces no
   advance and counts down no loop delay; it does not restart the stage clock. A turn missed
   because of a removal is not an update and does not consume this first-update exclusion.
   On later updates an animation advances when at least *d* ticks have passed since its last
   advance, or since its creation if it has not advanced yet, provided it is neither in a loop
   delay nor ending one (claim 11), is not held (claim 23), and *d* > 0. Burning walkers instead
   follow S-0615 claims 4 to 11: their running stage never advances and their expiry has its own
   delay and removal rules. A loop delay does not
   restart this clock. In uninterrupted play after the first advance, each image frame is held
   for exactly *d* ticks.
   - **The first image frame.** Let *c* be the integer creation tick and *u* the integer tick of
     the first actual update. With *d* ≥ 1, no hold or loop delay, and an update on every tick
     after *u*, the first advance is in tick max(*c* + *d*, *u* + 1). Its elapsed time from
     creation is max(*d*, *u* + 1 − *c*) ticks; this is exact integer arithmetic.
     For *d* = 1 and *u* = *c*, the update in tick *c* is excluded and the update in tick
     *c* + 1 advances: the first image frame lasts 1 tick, not 2. For *d* = 1 and *u* = *c* + 1,
     the update in tick *c* + 1 is excluded and tick *c* + 2 advances: it lasts 2 ticks.
     For *d* ≥ 2 either of those first-update timings gives a first advance in tick *c* + *d*.
     In these cases with an ordinary forward pass that continues, the end-of-tick drawn frames
     show the initial image in ticks *c* through the tick before the first advance.
   - **Which update is first.** An animation created before the turn pass (for example by a
     trigger action set off by polling map triggers, S-0203 claim 1) or during it takes its first
     turn in that tick, after every object already in the update order, unless a removal costs
     it that turn (S-0203 claims 3, 4 and 10). One created after the turn pass first updates in
     the next tick. Each creator's spec owns its place in the tick.
   - **Vehicle deploy presentations.** The presentation is created during the vehicle's
     turn with no initial loop delay and no reverse request for deploy, or a reverse request
     for undeploy (claim 29). Thus deploy plays forward unless the type itself is reversed
     (claim 8). Its playback retains this first-update exclusion and uses the
     selected, possibly random or normalised stage delay. S-0517 claim 7 owns the separate
     base-delay completion clock: neither this exclusion nor reverse playback pauses or
     reverses that clock.
   - **Lost updates.** An advance due in a tick with no update waits until the next eligible
     update; the next delay runs from that actual advance, with no catch-up advances. One lost
     update at a due advance therefore extends that image frame by 1 tick if the following
     tick's update is eligible. Further lost updates, holds or loop delays can extend it more.
   - Backing: `measurable` (AN1 for before-pass creation; AN26 for during-pass creation,
     after-pass death effects and the first turn lost to projectile removal). Claim 29 supplies
     the creator placements; claim 16 gives the resulting lifetime. Deploy presentation
     playback and its separate completion clock have S-0517 claim 7's backing (P1 C).
   - Confidence: high for the clock and creator placements; medium for the removal-skip
     fixtures before measurement

6. **Passes and their end limits.** The first pass starts at *k* = 0, so it shows image frame *a*
   first. A pass ends on the advance after which *k* is at or above its end limit:
   - while *r* ≥ 2, the end limit is λ<sub>e</sub> − *a*, so the pass shows image frames up to
     λ<sub>e</sub> − 1,
   - on the last pass (*r* ≤ 1) it is *E*, so the pass shows image frames up to *a* + *E* − 1.

   A new pass starts at *k* = λ<sub>s</sub> − *a*, showing image frame λ<sub>s</sub> first. So with
   *L* passes the animation shows *a* … λ<sub>e</sub> − 1, then λ<sub>s</sub> … λ<sub>e</sub> − 1
   for each middle pass, and λ<sub>s</sub> … *a* + *E* − 1 on the last pass. With one pass it shows
   *a* … *a* + *E* − 1 and the loop fields play no part. The image frame at the end limit is never
   drawn: the pass ends, or the animation ends, in the same update. Shadow types (claim 10) and
   reversed animations (claim 8) have further end rules, and ping-pong animations turn instead
   (claim 9). The rules are collected in Formulas F3.

   An absent loop end takes the sprite end *E*<sub>s</sub>, not the end field *E* (claim 25,
   `quirks.anim_loop_end_from_sprite`). So a type that gives an end field but no loop end, and has
   more than one pass (claim 7), plays every pass but the last up to image frame *E*<sub>s</sub> −
   1, past its own end field, and only its last pass stops at *a* + *E* − 1.
   - Backing: `measurable` (scenarios AN1, AN5)
   - Confidence: high; medium for the absent loop end (scenario AN5 checks it)

7. **Remaining passes and endless animations.** At creation *r* = max(1, (*L* × max(*m*, 1)) mod
   256), where mod gives the least non-negative residue. A pass that ends with *r* ≥ 2 is followed
   by a new pass, and *r* goes down by 1, except at 255, where it stays. A pass that ends with *r* ≤
   1 is the last. So a type whose loop count is −1 (or any value congruent to 255 modulo 256) loops
   for ever when *m* = 1. With *m* = 2 the same type gives *r* = 254 and stops after 254 passes, a
   loop count of 256 gives one pass, and so does a repeat factor of 256 with any loop count
   (`quirks.anim_loop_count_wrap`). After the last pass the animation switches to its follow-up
   type, or ends (claims 15 and 16).
   - Backing: `measurable` (scenario AN5); the case *m* ≥ 2 needs a creator that gives a repeat
     factor, which is `unmeasured` until the specs that own such creators say which ones do
   - Confidence: high

8. **Reversed play.** A reversed animation starts at *k* = λ<sub>e</sub> − 1 with σ = −1: its first
   image frame is *a* + λ<sub>e</sub> − 1, which is λ<sub>e</sub> − 1 only when *a* = 0. A pass of
   a reversed animation ends on the advance after which *k* ≤ 0, so image frame *a* itself is never
   drawn. It also ends, as in claim 6, when *k* is at or above the end limit of claim 6. Each new
   pass starts at *k* = λ<sub>e</sub>, one stage beyond the first pass's start
   (`quirks.anim_reverse_raw_loop_end`). With *a* ≥ 2 and *r* ≥ 2 the first advance already brings
   *k* to or above the end limit λ<sub>e</sub> − *a*, so the first pass shows one image frame only.
   - Backing: `measurable` (scenario AN6)
   - Confidence: high

9. **Ping-pong play.** A type with the ping-pong flag turns at the ends of its range. After an
   advance, σ is negated when:
   - on the last pass (*r* ≤ 1): *k* ≥ *E* or *k* = 0,
   - while *r* ≥ 2: *k* ≥ λ<sub>e</sub> − *a*, or *k* equals the start field *a* itself (not 0).

   An advance that turns never ends the pass. On an advance that does not turn, the end rules of F3
   apply as to any other animation. The stage keeps the value at which it turned, so the turning
   image frame (*a* + *E* or λ<sub>e</sub>) is shown for one stage delay, and draws nothing if it is
   *N* or more. Consequences:
   - a ping-pong animation that is neither a shadow type nor reversed never ends a pass, so its
     remaining passes never change and it never ends on its own,
   - a shadow type still ends a pass when *k* is at or above λ<sub>e</sub> − *a* without turning
     (claim 10). On the last pass that happens when λ<sub>e</sub> − *a* < *E*; with absent end and
     loop-end fields, that is whenever *a* > 0,
   - a reversed type still ends a pass when its stage comes down to 0 or below without turning,
   - with *a* > 0 and *r* ≥ 2, the lower turn needs *k* = *a* exactly. A stage coming down from the
     upper turn passes *a* only when it turned above *a*, so an animation that turns at
     λ<sub>e</sub> − *a* ≤ *a* runs down past 0 into image frames below *a* and then below 0, which
     draw nothing, and stays blank (`quirks.anim_pingpong_lower_turn`).
   - Backing: `measurable` (AN7) for its four listed flag/range combinations. The reversed
     ping-pong end case is `unmeasured` here: AN7 includes no reversed ping-pong type.
   - Confidence: high

10. **Shadow types.** For a shadow type, the sprite end is ⌊*N* / 2⌋ instead of *N*
    (*E*<sub>s</sub>, claim 25). Wherever image frame *f* = *a* + *k* is drawn, image frame *f* +
    ⌊*N* / 2⌋ is also drawn as its shadow (darkening the pixels below it), never
    translucent, whatever claims 20 and 21 give for the image. The image is drawn before
    its shadow, with no other object's drawing between them. Both are centred by S-0102
    claim 19, but their draw points differ when the type has a vertical draw shift:
    the shadow stays at the unshifted screen projection of the animation's position,
    including the height lift (claim 26). Consequently, wherever their non-transparent
    pixels overlap and the shadow passes its depth test, it darkens the image already
    painted, rather than the image covering the shadow. Shadow depth testing, darkening, and
    overlap behaviour are specified by S-0102 claims 10, 13 and 26 to 28. A type that is also drawn
    flat or tiled (S-0102) draws no shadow image at
    all. Its last pass also ends when *k* is at or above λ<sub>e</sub> − *a*, as the other passes
    do, unless it is reversed. The middle image frame (claim 14) is not
    halved.
    - Backing: `measurable` (AN1, AN8) for frame selection and pass end, flat/tiled
      suppression; AN24 for placement, draw order and overlap, including a translucent
      image. Shadow darkening and depth tests also have S-0102's backing.
    - Confidence: high for the shadow image frame, the flat and tiled exception and the end; medium
      for the darkening style, which belongs to the shadow drawing of S-0102

11. **Loop delays.** A loop delay of *D* updates starts:
    - at creation, when the creator asks for one (*D* > 0), and
    - at the end of every pass that is followed by a new pass (claim 7), when the type's random loop
      delay pair (*D*<sub>1</sub>, *D*<sub>2</sub>) is not (0, 0). *D* is then a ranged draw from
      the scenario random stream over the two values, in either order. The pair counts updates
      (Terms), never normalised. A drawn *D* of 0 gives no loop delay: the new pass plays at once
      and has no start event.

    During a loop delay the animation does not advance, and it is drawn at its current stage: the
    first image frame of the new pass, or image frame *a* (*a* + λ<sub>e</sub> − 1 if reversed) at
    creation. Each update counts *D* down by one. On the update that brings it to 0 the start event
    happens (claim 13), and the animation does not advance on that update. The clock of claim 5 is
    not restarted, so the first image frame of a pass that follows a loop delay of *D* is shown for
    max(*d*, *D* + 1) ticks when *d* > 0, every tick receives an update and the animation is not
    held after the delay. Missed updates or a continuing hold can extend it, and *d* = 0 prevents
    the advance altogether. A loop delay given at creation is counted only from the second update
    (claim 5), so its start event comes on update *D* + 1 after creation.
    - Backing: `measurable` (AN9) for its positive delays between passes, their draws and
      start sounds; AN25 for equal-endpoint and zero-pair draw allocation. The reversed
      delay image and either-order input remain `unmeasured`: these scenarios supply
      neither reversed play nor swapped endpoints. AN26 tests the initial loop delay of
      a projectile trailer (claim 29)
    - Confidence: high for the order of events; medium for the drawing during the delay

12. **No random starting image frame.** No field and no creator makes an animation start at a
    random stage. Randomness in the ordinary stage clock comes only from claims 3 and 11.
    This is not an exclusion of event, creation or specialised-motion draws (claim 28).
    - Backing: `measurable` (scenario AN3) for the tested trigger creator and types starting at
      image frame 0. The universal exclusion of every other field and creator is `unmeasured`:
      AN3 cannot exercise all creation paths; creator coverage remains an Open question.
    - Confidence: medium

13. **Start event.** The start event happens when the animation is created with no loop delay, when
    a loop delay ends (claim 11), and when it switches to a follow-up type (claim 15). It plays the
    type's start sound and produces the middle event at once when μ = 0 (claim 14). The middle
    event's effects and draws precede the type's resource chain reaction, when enabled; S-0401
    claim 20 owns that reaction and its draws. Consequently a crater from this immediate middle
    event can empty the animation's cell before the reaction tests it, leaving no reaction draws
    or detonation. A crater take from stage 6 instead leaves the resource overlay at stage 0
    (claim 14), so an enabled, unsuppressed reaction still empties that cell and follows
    S-0401 claim 20's normal draw and detonation rules. When resource remains, the reaction
    removes all of it. So an
    animation plays its start sound once for each type it plays as: at creation (or at the end of
    the loop delay its creator gave), and at each follow-up switch. One with random loop delays also
    plays it at every pass that follows a delay *D* > 0. A trigger-played animation has no loop
    delay at creation (S-0011 claim 36), so its creation start event plays the start sound and
    produces the resource chain reaction when enabled, just as for any other animation. Its later
    start events, at loop-delay ends or follow-up switches, have no start sound and no chain
    reaction. It delivers no damage (claim 17), but its middle events, including the immediate
    one when μ = 0, happen as for any other animation. A burning walker's creation start event
    also retains these effects; its later playback follows S-0615 claims 4 to 11.
    - Backing: `measurable` (AN9, AN10, AN21, and S-0401 E17 for the immediate
      middle event preceding the chain reaction). Follow-up start sounds and later
      trigger-played chain-reaction suppression are `unmeasured` here: AN11 gives no
      follow-up sound, and AN21 neither refills resource nor observes a later start event.
    - Confidence: high for event timing, the trigger-played creation/later distinction and the
      middle-before-reaction order; the reaction's
      effects and composed draw sequence have the confidence of S-0401 claim 20

14. **Middle event.** The middle event happens on every advance after which *a* + *k* = μ, provided
    μ ≠ 0, so it can happen once per pass, and for reversed and ping-pong animations on the way down
    as well. μ is ⌊*N* / 2⌋ of the sprite that claim 25 names, whatever the type's fields say. It
    does not happen on the advances of a bouncing type (the start event's middle event of claim 13
    still does). An animation whose advances never reach image frame μ has no middle event, except
    through claim 13. Reaching μ on an ending advance still produces the event, even though that
    image frame is not drawn (State and ordering). On an advance that also delivers damage (claim
    17), the middle event comes after the damage detonation. On an ending advance, and on the
    advance before a follow-up switch, it comes before the end or that switch, as State and ordering
    says. Burning walkers do not advance while running; S-0615 claims 4 to 11 govern their playback
    and expiry exceptions, without suppressing claim 13's creation-time middle event. These rules
    apply during map loading too. A tile animation (S-0010 claim 19) or an overlay's cell animation
    (S-0009 claim 12) with μ = 0 produces this event at creation, not at its first update. The event
    requests the type's particle count of its named particle type when a particle type is given and
    the count is positive, otherwise none. These particles are created in the shared gas system of
    S-0613 claim 1, regardless of their type's behaviour kind, as in S-0613 claim 2. During map
    loading that system does not exist yet, so a loading-time μ = 0 request meets no shared system
    and the original stops; our engine's departure is in S-0613's edge cases. Particle creation is
    not gated by height. The particle requests precede any random choice between a scorch mark and a
    crater, so their creation draws change the stream state used by that choice. S-0202 claim 15
    owns the particle creation draw groups during loading and play, including their conditional
    speed, lifetime and colour draws. Its R14 A and D test their composition, including
    trigger-created animations and middle events later in playback. The creator and loading/play
    phase do not change the event gates or draw composition. Subsequent particle updates are outside
    this composition, not draw-free; their draws must be accounted for separately.

    The scorch marks and craters of the middle event are attempted only when the animation's
    height above the terrain (glossary) is less than 30 leptons. For world position (X, Y, Z),
    this is the integer difference Z − H(X, Y), with H the terrain surface height of S-0106 claims
    2 and 3 at that position, including its in-cell displacement. It is not the height of the
    creator's cell, nor height above a bridge deck. There is no lower bound: a negative difference
    also passes. A displaced cell animation can therefore pass or fail according to the terrain
    at its displaced position even though its creation height came from the overlay's cell.
    An animation on a high bridge's deck makes no scorch mark or crater, while one at the terrain
    height on raised ground passes as on flat ground. At exactly 30 leptons neither is attempted.

    | Height test | Type's effect flags | Middle-event request |
    |---|---|---|
    | fails | any | no scorch mark, no crater, no choice draw |
    | passes | neither | neither effect, no choice draw |
    | passes | scorch only | scorch mark, no choice draw |
    | passes | crater only | crater, no choice draw |
    | passes | both | one scenario ranged draw over [0, 2147483646], scorch mark when the integer result is below 1073741824, crater otherwise |

    The choice range follows S-0202 claim 4, including any rejected raw values. The original
    compares a floating-point product with 0.5: the integer result times the nearest binary64 value
    to 1 / 2147483646, multiplied with a 53-bit significand in round-toward-zero mode (S-0205 claims
    4 and 5). That constant lies slightly below the exact reciprocal. Thus even result 1073741823
    gives a product below 0.5, and the table's integer threshold is equivalent in that mode. With
    `quirks.chop_rounding` off, use that floating-point comparison in the selected mode instead of
    assuming the integer threshold remains equivalent. This choice is made even if the chosen effect
    cannot subsequently be placed. S-0202 claim 15 owns the smudge candidate preferences and
    selection draws during loading and play, using S-0312 claim 26's footprint fit and draw-free
    placement rules. These draws are separate from the scorch-or-crater choice. A crater request
    takes 6 bales from the animation's cell under S-0401 claim 3 before placement is attempted. A
    resource cell at stage 6 remains resource at stage 0; stages 0 through 5 are emptied. These are
    exact integer stage changes. Any bales delivered follow that claim, so failure to place a crater
    does not undo the removal. S-0202 claim 15 also owns the loading-phase relation to the overlay
    data stream and the conditional resource-removal effects before smudge selection: tile-animation
    creation from recalculation and neighbour spread-offer draws. The same conditional effects apply
    during play; they do not add a fixed draw count to every crater request.
    - Backing: `measurable` (AN10, AN19, AN23) for their forward and creation-time
      events, height/flag cases, particle-before-choice order and resource removal.
      Shared-system membership regardless of particle behaviour kind is `measurable`
      through AN10's particle variant.
      Downstream creation and selection draws during loading and play have S-0202 claim 15's
      backing (R14 A to E) and confidence, including its medium-confidence placement and
      resource-removal recalculation composition.
      Reverse/ping-pong repeated events and bouncing-type suppression are `unmeasured`
      here: none of these scenarios supplies those playback types. A middle event on an
      ending advance, before an end or a follow-up switch, and its order after the same
      advance's damage detonation are `measurable` (AN28), through the detonation's resource
      change, which decides whether the middle event's crater can be placed. The order of the
      two effects' random draws within that tick follows from the same order but is
      `unmeasured`: the random-state recorder reads the stream only between ticks, and no
      outcome here depends on that draw order alone. AN28 does not test the middle event's
      draws against a follow-up switch's delay draw; AN25's ordering variant does (claim 28).
      The exact floating-point
      threshold edge is also `unmeasured`: 32 observed incoming states need not produce
      either integer immediately beside it; AN23 tests predictions only for values reached.
    - Confidence: medium for the order of the draws within one advance (`unmeasured`, above),
      medium for the delivery's detonation coming before the middle event (claim 17, until AN28
      runs), high for event timing, the request gates, the choice and resource removal,
      medium for the terrain-height test

15. **Follow-up type.** When an animation's last pass ends and its type names a follow-up type, the
    same animation continues as that type, at the same place and with the same owner. Burning
    walkers have the playback exceptions of S-0615 claims 4 to 11 and cannot reach a pass end
    while running. The start
    event happens in that update (claim 13), and the switch sets:

    | Quantity | After the switch |
    |---|---|
    | stage *k* | the follow-up type's start field value *a*′, so the first image frame shown is 2*a*′ (`quirks.anim_followup_start_doubled`) |
    | step σ | the same sign as before (`quirks.anim_followup_keeps_step`) |
    | reversed | decided afresh from the follow-up type's reverse flag and the creator's request (Terms) |
    | remaining passes *r* | *L*′ mod 256 (least non-negative residue), *L*′ being the follow-up type's loop count, with no repeat factor and no raising to 1; a value of 0 behaves as 1, because a pass that ends with *r* ≤ 1 is the last (claim 7) |
    | stage delay *d* | chosen afresh from the follow-up type (claims 2 to 4, including a random draw and normalisation) |
    | clock of claim 5 | restarted at that tick, with no new first-update exclusion; the follow-up's initial stage lasts *d* ticks when *d* > 0 and subsequent updates are uninterrupted and not held, and does not advance when *d* = 0 |
    | damage accumulator (claim 17) | 0 |

    The image frame at the old type's end limit is never drawn; the first image frame of the
    follow-up type is drawn from that update on. Because σ and the reversed test are separate:
    - an animation that its creator reversed stays reversed and plays the follow-up type downwards
      from 2*a*′, ending its pass when *k* ≤ 0,
    - an animation reversed only by its old type's flag, whose follow-up type lacks the flag, keeps
      σ = −1 but is no longer reversed. If the follow-up type is neither ping-pong (claim 9) nor a
      shadow type (claim 10), its stage only goes down, so it ends its pass on the first advance
      when the follow-up type's end limit is *a*′ − 1 or less, and otherwise never ends.
      Once its image frame *a*′ + *k* is below 0 it draws nothing, while it still delivers damage
      (claim 17),
    - an animation that was not reversed, whose follow-up type has the flag, keeps σ = +1 and is
      reversed from then on, so the reversed end rule and restart of claim 8 apply to it.
    - Backing: `measurable` (AN11) for doubled start, kept negative step, restarted delay
      and the tested zero loop count. Creator-requested reverse, a forward-to-reversed
      follow-up, other follow-up pass counts and damage reset are `unmeasured` here:
      neither pair in AN11 supplies those cases.
    - Confidence: high

16. **End of an animation.** An animation whose last pass ends and whose type names no follow-up
    type normally ends in that update and is removed. For every infantry-making animation, S-0611
    claims 9 to 11 own creation, ownership, placement and retries: the terminal type attempts
    creation then, but a refused placement keeps the animation alive, steps its stage back by one
    and retries when that frame is next due. Burning-walker expiry and removal instead follow S-0615
    claims 4 to 11. In ordinary removal the last image frame drawn is the one shown before that
    update. The lifetime of an animation with one pass, no loop delay, delay *d* ≥ 1, *E* ≥ 1 and
    *a* = 0, that is neither reversed, ping-pong nor a burning walker and, if it is a shadow type,
    has λ<sub>e</sub> ≥ *E*, with no refused infantry placement, is therefore *E* × *d* ticks when
    its first update is in the creation tick and no update is lost or held. More generally, with
    creation tick *c*, first-update tick *u*, and uninterrupted updates thereafter, it is max(*d*,
    *u* + 1 − *c*) + (*E* − 1)*d* ticks, in exact integer arithmetic. This gives *E* × *d* + 1 ticks
    when *d* = 1 and *u* = *c* + 1 (claim 5).
    - Backing: `measurable` (AN1) for ordinary removal and before-turn-pass lifetimes. AN26
      tests the late and lost-first-update extensions using claim 29's creators. Infantry
      creation and placement retries inherit S-0611 claims 9 to 11's backing and confidence
      (including MU10); burning-walker exceptions inherit S-0615 claims 4 to 11's backing and
      confidence
    - Confidence: high for ordinary playback; delegated exceptions have their owning claims'
      confidence

17. **Damage over time.** An animation whose type has a damage value δ > 0, and which is not a
    bouncing type, delivers damage at its own position on every advance. It keeps a floating-point
    accumulator that starts at 1.0. On each advance δ is added (5δ when the animation is attached to
    a terrain object), and when the accumulator is 1.0 or more its whole part is delivered and
    subtracted (Formulas F5). So after *n* advances the total delivered is close to ⌊1 + *n*δ⌋ for
    an unattached animation, and the first advance already delivers 1 when δ < 1
    (`quirks.anim_damage_starts_at_one`). The advance that ends the animation delivers too. The
    delivery precedes the same advance's middle event (claim 14) and its end test (State and
    ordering). Burning walkers deliver no such damage while running, since their stage does not
    advance; expiring advances can deliver it, subject to S-0615 claims 4 to 11's playback and
    removal rules. A trigger-played animation delivers no damage. The warhead is one of two
    game-wide warheads (Import mapping), chosen on each delivery by the name of the animation's
    current type (claim 15), so a follow-up switch to or from the built-in invisibility-animation
    type changes the warhead from the follow-up's first delivery on. When no type has exactly that
    name, the other warhead is always chosen (Import mapping, the paragraph after the game-wide key
    table). The type's own warhead field plays no part here. When the chosen warhead is not given,
    the amount is still taken from the accumulator but no damage is done. The area, warhead effect
    and damage rules belong to the combat specs.

    Each delivery is an area detonation with **resource reduction allowed**, no attacker and
    no supplied attacking house, including the expiring advances of a burning walker.
    Animation ownership or attachment does not supply either an attacker or a house
    (S-0302 claim 1). Permission alone does not guarantee resource loss: S-0303 claim 17
    still requires the resource overlay's chain-reaction flag, the chosen warhead's resource
    flag and a large enough delivered amount. Its reduction is based on the integer amount
    delivered by this advance, not the imported fractional damage or the animation's lifetime
    total. This is distinct from the start-event resource chain reaction, whose detonation
    forbids resource reduction (S-0401 claim 20), and from a middle-event crater's removal
    (claim 14).
    - Order (State and ordering, Q-1716): the delivery's detonation comes before the same
      advance's middle event and end test, also on an ending advance and before a follow-up
      switch. Backing `measurable` (AN28, through the detonation's resource change and the middle
      event's crater); confidence medium until AN28 runs. The order of their random draws, and of
      the detonation's kills against the middle event, within that tick follows from it and is
      `unmeasured` (claim 14); confidence medium. An animation that leaves play in its own
      detonation does nothing more in that update (State and ordering, which rates it).
    - Backing: `measurable` (AN27 for resource permission, including walker expiry,
      and S-0302 claim 1's scenario D33 for absent attacker and attacking house).
      AN4, AN12, and AN20 (for the factor 5) make the delivery patterns, trigger
      suppression and final-advance damage `measurable`. The type-name warhead choice,
      including the change at a follow-up switch, is `measurable` (AN29). The ignored type
      warhead and missing game-wide warhead consumption are `unmeasured` here: no scenario
      gives the type's own warhead distinguishing verses or omits a game-wide warhead.
    - Confidence: high for the accumulator and the per-advance timing; medium for the warhead
      choice by the current type's name, until AN29 runs (Import mapping), and for its letter case
      and for the case with no type of exactly that name (Import mapping, the paragraph after the
      game-wide key table) and for the factor 5 (the kind test is clear, which objects count as
      terrain objects is the terrain specs' business)

18. **Playback timing reaches game state.** Several effects of an animation wait on its stages, so
    for a type marked normalised (claim 4) they depend on the game speed, as glossary claim G3
    allows:
    - damage is delivered per advance (claim 17), so its rate in ticks follows N(*d*, *s*),
    - the middle event creates craters, which remove resource from cells, and decides between a
      scorch mark and a crater with a draw from the scenario random stream (claim 14,
      S-0202 claim 15 owns the composition during loading and play); its tick follows the stages,
    - the start event at the end of a loop delay and the ranged draw of claim 11 happen at pass
      ends,
    - the follow-up switch, with its possible random rate, and the creation of infantry at the end
      (claim 16) happen at the end.

    A normalised animation with any of these effects runs a different number of ticks at
    different speeds. With `quirks.speed_normalised_delays` off, none of these timings depends on
    the game speed.
    - Backing: `measurable` (AN4, AN12) for damage timing. Speed-dependent timings of
      middle events, pass-end draws, follow-up draws and infantry creation are `unmeasured`
      here: neither scenario includes those effects at multiple speeds.
    - Confidence: high for damage and the end; medium for speed-dependent middle-event timings

19. **Detail level.** An animation whose type's detail field is greater than the player's detail
    level is not drawn, but it is updated as usual, so nothing but its picture changes. The original
    also stops drawing every animation whose detail field is above 1 while its measured drawing rate
    of the screen falls below a configured minimum (with a band above it before it resumes). That
    depends on the host machine and real time, so our engine never enters that reduced mode. This is
    a departure, with no switch.
    - Backing: `measurable` (scenario AN13) for the detail field; the reduced mode is `unmeasured`
      (it depends on the machine's speed, and our engine leaves it out)
    - Confidence: high

20. **Translucency.** When the player's detail level is at least the type's translucency detail
    field, an animation gets the translucency code and hide rule of the first row of this table that
    fits its current type:

    | Current type | Translucency code | Not drawn when |
    |---|---|---|
    | has the translucent flag | fade by progress: the highest code *c* of 3, 2 and 1 whose threshold *k* exceeds, the thresholds being 0.6 *E*, 0.4 *E* and 0.2 *E* respectively, and 0 when *k* exceeds none; compared in floating point (Formulas F4) | *t* ≥ 15 |
    | translucency field > 0 | 1, 2 or 3 for a field of 25, 50 or 75; 0 for any other positive value | *t* ≥ 15 |
    | neither (translucency field ≤ 0) | from the translucency level: 0 when *t* = 0, 1 when 1 ≤ *t* ≤ 5, 2 when 6 ≤ *t* ≤ 15 (`quirks.anim_translucency_levels`) | *t* > 15 |

    Only one row applies; the rows do not combine with each other. The different hide boundaries,
    *t* ≥ 15 for the first two rows and *t* > 15 for the last, are the original's
    (`quirks.anim_hide_at_level_15`), so a translucent-flag or translucency-field type is hidden at
    *t* = 15 while a type with neither is drawn at 50 %. Below the translucency detail field the
    table does not apply: the code is 0 and the animation is drawn even when *t* ≥ 15. In both cases
    the code is then combined (Terms) with the code of claim 21.

    A building's slot animations take the building fade level of S-0516 claim 18 as *t*, except that
    they take 16 while the building is cloaked from the viewer (Terms, S-0516 claim 28). Other
    animations have *t* = 0. *t* follows the fade level: it is set again when a slot animation of
    the building starts and when the building refreshes its animations, which it does at each change
    of its fade level. So, at a fade level of 15:
    - a viewer from whom the building is cloaked sees none of its slot animations, whatever their
      type, as long as the table applies,
    - a viewer who sees the building see-through (the player who owns it, for example) sees its
      translucent-flag and translucency-field slot animations hidden and the others at 50 %,
    - a slot animation whose type's translucency detail field is above the player's detail level is
      drawn opaque even while its building is cloaked from the viewer, since the table does not
      apply to it (`quirks.anim_low_detail_shows_cloaked`).

    Below a fade level of 15, *t* is the level itself whatever the viewer. S-0516 claim 28 stops
    drawing the building itself from fade level 11 for a viewer who neither owns it nor senses its
    cell nor is mutually allied with its owner. Such a viewer still sees its slot animations at
    fade levels 11 to 14: those of the neither row at 50 %, the others with their own codes.

    Since *t* depends on the local player, it changes only what is drawn, never game state.
    - Backing: `measurable` (AN14 for progress and field codes; AN22 for full-cloak slot
      visibility). Visibility at fade levels 11 to 14 is `unmeasured`: AN22 records it but
      does not establish a tick-by-tick fade fixture or a falsifier. Sensor-covered viewers
      and intermediate level-band boundaries are also `unmeasured` here: neither scenario
      supplies those cases. Negative-end fade comparisons and F4's equivalence over the
      entire 32-bit integer domain are `unmeasured`: AN14 uses only end values 10 and 0,
      and a finite set of playback runs cannot exercise that whole domain.
    - Confidence: high for the table and the fade thresholds; medium for the source of *t*.
      Building fade timing and viewer presentation have S-0516 claims 18 and 28's backing and
      confidence. Slot-animation visibility at fade levels 11 to 14 remains low-confidence.

21. **Slot-animation erasure.** The building body's own erasure translucency is governed by
    S-0514 claim 27: code 2, or 3 when its art has the double-thick flag. This claim covers only
    slot animations. While a building is being erased (Terms), every slot animation it has is drawn
    with translucency code 2, or 3 if its type has the double-thick flag, combined (Terms) with
    the code of claim 20, at every detail level, and is held (claim 23). A slot animation that
    starts during erasure has that same erasure translucency and hold from its start, without
    waiting for another weapon lock-on. Both end when the building stops being erased: the slot
    animations are drawn as before and are released subject to claim 23's independent hold causes.
    The original omits erasure translucency for slots started during erasure until another
    weapon lock-on, a drawing inconsistency fixed here as a departure confirmed by the project lead.
    - Backing: `measurable` (AN14, AN16) for existing-slot drawing and holds; late-slot
      translucency is a departure tested against this spec, not the original (AN16 part c).
    - Confidence: medium for the original's erasure transitions (the combat specs own weapon
      lock-on and release); high for the departure

22. **Attached animations.** An animation can be attached to an owner object. While attached, its
    position is the owner's position plus the displacement it had from the owner when it was
    attached, in leptons and including height, so it follows the owner. It is in the ground render
    layer (S-0102). When it is detached it stays where it is. When the owner is removed from the
    game, the animation disappears at once and is deleted at its next update, whatever its stage.
    That update does nothing else of this spec's: no advance, no damage, no start or middle event,
    no loop-delay count, no follow-up switch and no infantry (claim 16).
    - Backing: `measurable` (scenario AN15) for the position, the removal and the absence of an
      advance; the absence of damage, events and infantry at that last update is `unmeasured` (it
      would need an attached animation due to deliver or end on the very tick its owner is removed,
      which cannot be timed from outside). Detachment is `unmeasured` here: AN15 provides no
      action that detaches without removing the owner. The render layer uses S-0102's backing
    - Confidence: high

23. **Held animations.** A building's slot animation is held while it is frozen by S-0108, claim 13
    (which covers both a building without power and a disabled building), and while its building is
    being erased (claim 21). The two causes are independent: the animation is released only when
    neither applies, so a slot animation of an unpowered building stays frozen when its building
    stops being erased. A held animation does not advance, so nothing that follows from an advance
    happens: no damage, no middle event from an advance, no end of a pass, no follow-up switch and
    no end. Everything else goes on as usual: its first update after creation is still skipped, a
    loop delay still counts down, and the start event at the end of that delay still happens (with
    its sound, chain reaction and, when μ = 0, middle event). Its clock (claim 5) keeps running, so
    when it is released it advances on its first eligible update if *d* > 0 and at least *d* ticks
    have passed since the last advance (or creation before any advance). The first-update exclusion
    and any continuing or ending loop delay still apply (claim 5).
    - Backing: `measurable` (AN16) for stage holds, independent causes, immediate release
      advance, and a delayed start sound while held. First-update exclusion while held,
      absence of damage, and continued chain reaction or μ = 0 effects at a delayed start
      are `unmeasured` here: AN16 neither starts a held animation with an initial delay
      nor supplies those effect fields.
    - Confidence: medium; high for the loop delay and start event going on while held

24. **Fog (deferred).** The original's fog-on rule hides an animation that its building marks
    as hidden under fog of war when its type has the fog-remove flag (set when absent), and
    draws it otherwise. Remembered images of S-0102 show the hidden animations instead.
    No skirmish reaches this state: S-0409 claim 19 turns fog off at skirmish start regardless
    of data. This rule is preserved for a future mode spec, not required for skirmish playback.
    - Backing: `unmeasured` (AN17 is deferred; no permitted skirmish setup enables fog)
    - Confidence: low

25. **Which sprite the derived fields come from.** In each game, an animation type gets a sprite at
    each of its type reads and may get another at the theater refresh (S-0007, claim 14). The
    derived values follow one rule. A *sprite-found moment* is a type read that finds a sprite for
    the type, or the theater refresh of a new-theater type (Terms) when it finds one. Let
    *E*⁻ and λ<sub>e</sub>⁻ be the end field and loop end in force just before that moment (0 for
    both before the first type read of the game; at a type read, what the previous type read left),
    and *n* the image-frame count of the sprite found. The moment gives

    > *E*⁺ = E<sub>s</sub>(*n*) when *E*⁻ = 0, and *E*⁻ otherwise,
    > λ<sub>e</sub>⁺ = *E*⁺ when λ<sub>e</sub>⁻ = 0, and λ<sub>e</sub>⁻ otherwise,
    > μ = ⌊*n* / 2⌋.

    At a type read, an end field or loop end given in the type's own art section takes precedence
    over *E*⁺ or λ<sub>e</sub>⁺, and one it does not give is *E*⁺ or λ<sub>e</sub>⁺. The image-named
    art section never supplies either field (S-0007 claim 34). At the new-theater refresh, *E*⁺ and
    λ<sub>e</sub>⁺ stand. A type read that finds no sprite changes only the fields it gives. The
    theater refresh of a type with the theater flag replaces the sprite that is drawn (S-0007) but
    none of these values. A value of −1 counts as a value here, not as 0; it is resolved from the
    sprite that is drawn, the first time an animation plays as the type in the game. The
    consequences:
    - an **absent end field** takes *E*<sub>s</sub> of the first sprite found for the type in the
      game, normally that of its first type read,
    - an **absent loop end** takes the same *E*<sub>s</sub> when the first type read found the
      sprite, even when an end field is given (claim 6, `quirks.anim_loop_end_from_sprite`). If
      one or more earlier type reads found no sprite, it takes the end field given at the last of
      them when that field is not 0 (−1 included, which then resolves as below), and
      *E*<sub>s</sub> of the first sprite found when that field is 0 or absent,
    - an **end field of −1** takes *E*<sub>s</sub> of the sprite that is drawn the first time an
      animation plays as the type in the game, by being created as it or by switching to it (claim
      15). That is the sprite after the theater refresh, so it can differ from an absent end field
      for theater and new-theater types. A loop end of −1 then takes *E*,
    - an **explicit 0** in the end field or the loop end is kept (an end field of 0 gives the last
      pass an end limit of 0, claim 6). The exception is a new-theater type whose sprite is
      found at the theater refresh. Its end field of 0 then takes *E*<sub>s</sub> of the refreshed
      sprite, and a loop end of 0 then takes the end field,
    - **μ** comes from the last sprite found at a type read or at a new-theater refresh.
      This also determines μ for an animation created during map loading (Q-0046 Q2c): the
      map's theater sprite does not automatically supply μ. For a type with the theater flag,
      the refresh changes the drawn sprite but keeps the μ left by the last successful type
      read. For a new-theater type it recomputes μ from the sprite successfully found at that
      refresh. For a type with neither flag it keeps μ from the last successful type read,
    - a type that never finds a sprite at those moments keeps an end field and loop end of 0 when
      they are absent or 0, and has μ = 0, so its start event also produces the middle event (claim
      13).

    These cases differ from the simple reading (every value from the sprite that is drawn) only when
    the sprites a type gets in one game have different image-frame counts, or when a type read
    finds no sprite. A defined type with no sprite is distinct from having no animation type
    at all; S-0522 claim 13 owns the absent-parachute lifecycle and its release-tick fall-rate
    exception.
    - Backing: `measurable` (AN18, AN23 part C) for their successful two-sprite fixtures
      and explicit-zero ends. Earlier failed sprite finds, repeated reads with changed
      fields and loop-end −1 or zero remain `unmeasured`: neither scenario creates those
      lookup/read histories. AN25 tests a type with no sprite at any find, including its
      absent-limit and explicit-positive-limit playback.
    - Confidence: medium (the order of the type reads and the refresh is S-0007's; the rule itself
      is clear)

26. **Animation draw point and vertical draw shift.** An ordinary or flat animation's
    image is centred on the screen pixel of its current world position, using S-0101
    claim 1's projection with the height lift of its claim 3 and the view origin of its
    claim 5, then moved down by the type's integer vertical draw shift *v* pixels.
    A positive *v* moves the image down; a negative *v* moves it up. There is no further
    horizontal displacement. This drawing displacement does not change the world
    position, attachment displacement, damage position or middle-event height test.
    The shadow image frame of a shadow type does not receive *v* (claim 10).
    Building slot animations use their own current position, already displaced by
    the slot under S-0108 claim 10: the type's shift is additional, not a replacement
    for the slot displacement. Tiled images use claim 27 instead.
    - Backing: `measurable` (AN24)
    - Confidence: high

27. **Tiled copies.** A tiled animation shows a vertical stack of copies of its current
    image frame, not one copy per stage or a fixed imported copy count. Let *h* > 0 be
    the height in pixels of image frame 0's stored rectangle (Terms), not the
    canvas height and not the rectangle height of the current frame. Let (*pₓ*, *pᵧ*)
    be the screen projection of the animation position, with *pᵧ* measured from the
    battlefield view's top row. Copy *j*, counted from 0 upwards, is centred at

    > (*pₓ*, *pᵧ* − ⌊*h*/2⌋ − *j h* + *v*).

    Thus even the lowest copy is lifted by half the first rectangle's height before
    the type's vertical draw shift is applied. The spacing is exactly *h* rows.
    In a full-view drawing the number of copies is

    > *C* = 1 when *pᵧ* − ⌊*h*/2⌋ < 0, otherwise
    > *C* = ⌊(*pᵧ* − ⌊*h*/2⌋) / *h*⌋ + 2.

    All arithmetic here is integer and exact; the divisions take the floor. The highest copy
    therefore has its unshifted centre strictly above the view's top row; clipping removes pixels
    outside the view. The shift *v* moves every copy but does not change this count. Copies are
    drawn from lowest to highest, so a translucent upper copy blends with already-painted lower
    copies wherever they overlap and pass their depth tests. No shadow image frame is drawn, even
    when the type is a shadow type or is also flat (claim 10). S-0102 claim 13 owns the depth of the
    copies, not their number or spacing. With a zero-height first rectangle and a projected centre
    at or below the view's top row, the original's stacking fails to terminate. Our engine draws at
    most one copy centred at (*pₓ*, *pᵧ* + *v*) and warns about the sprite; this is a safe bad-data
    departure, not a quirk switch.
    - Backing: `measurable` (AN24) for positive-height placement, count and spacing.
      The zero-height failure is `unmeasured`: the oracle cannot produce a completed
      drawn frame in that case; the safe result is a design rule.
    - Confidence: high for spacing and the first-rectangle source; medium for the
      full-view count before measurement

28. **Playback draws are simulation effects, whether drawn or not (Q-0770).** Claims 3
    and 11 use the scenario random stream for every ordinary animation in play. Neither
    shroud, being off screen, a detail field above the player's detail level, nor an absent
    sprite changes the clock, pass ends, follow-up switches or their draw allocation. No
    viewer-dependent cosmetic stream substitutes for these draws. Suppression of pictures
    alone does not suppress start or middle effects either. Independent holds, removal,
    zero stage delay and the specialised playback exceptions still have their stated effects.

    A spriteless type plays its passes and follow-up normally. With absent end and loop-end
    fields, both are 0 (claim 25), not an instruction to delete at creation: a forward,
    non-ping-pong animation with one pass ends or switches on its first eligible advance.
    Explicit positive limits permit longer invisible passes and between-pass draws.
    Its μ = 0 gives an immediate middle event at each applicable start event (claim 13).
    Missing-sprite −1 limits retain the safe departure in Edge cases.

    A between-pass delay is chosen in that animation's own turn at the pass end. A follow-up
    delay is chosen in that same turn at the follow-up switch, before the new type's start
    event and its effects. There is no separate animation batch after the turn pass.
    Draw assignment among animations and other objects follows S-0203 claims 3, 4 and 10,
    including missed turns: a missed turn has no playback draw. Due damage and the old
    type's middle event precede either pass-boundary delay choice. A single ordinary pass
    boundary takes either the loop-delay choice or the follow-up-delay choice, never both.
    The former uses one ranged draw only when its endpoints differ; equal nonzero endpoints
    give their constant delay without consuming a raw value, and (0, 0) takes no draw.
    Claim 3 gives the analogous random-rate rule. Every ranged draw retains S-0202 claim 4's
    inclusive range and rejection behaviour.

    These are the only random choices of the ordinary stage clock; advancing, reversing,
    ping-pong turning, pass-count decrement, a delay countdown and its expiry add none.
    A follow-up switch is its only autonomous ordinary type change, not an extra random
    type selection. This does **not** make the whole animation update draw-free otherwise:
    - start and middle effects retain claims 13 and 14's conditional scenario draws,
      including particles, smudges and resource reactions (S-0202 claim 15),
    - damage can cause deaths and their effects, with the combat specs' draws,
    - creating another animation, including a trailer or a landing effect, takes that
      animation's creation and immediate-event draws at the creation, not its first update,
    - terminal infantry creation retains S-0611 claims 9 to 11's draws,
    - burning-walker destination choices retain S-0615 claims 5, 6 and 9's draws.
      Bouncing types' landing effects are not ordinary pass-boundary delay choices.

    Thus claims 3 and 11 are not a complete draw budget for every animation type. A
    building replacing a slot animation creates another animation under S-0108 claims 12
    and 13; it is not a random in-place follow-up selection by this clock.
    - Backing: `measurable` (AN25 for visibility, spriteless passes, both delay choices,
      within-turn precedence and absence of extra ordinary clock draws). Delegated effects
      retain their owning claims' backing. No claim of a complete bouncing-motion draw
      budget is made here.
    - Confidence: high

29. **Creators and first-update timing in play (Q-0771).** An animation is created at the
    event that produces it, not at drawing time. The following placements apply to actual
    animation objects; an infantry death sequence played by the infantry itself is not a
    new animation (S-0107 claim 17).

    | Creator or event | Place of creation relative to the turn pass |
    |---|---|
    | A weapon's shot animation in ordinary firing | during the firer's turn; the shot's successfully placed projectile enters play before this animation; created with no initial loop delay, repeat factor 1 and no reverse request; attached to a firer that is not a building (claim 22), not attached for a building firer (S-0301 claims 20 to 23) |
    | A projectile's warhead detonation animation | during that projectile's detonating turn, before the projectile leaves play |
    | Death or destruction animation from damage | at the damage or death event, in the same tick part as that cause, not a later death-animation pass |
    | Projectile trailer | during the projectile's turn, before its movement and any detonation that turn (S-0306 claim 16); its creator gives it an initial loop delay of 1 update, repeat factor 1 and no reverse request |
    | Moving ship's wake | during the ship's own turn, after locomotor movement (S-0504 claim 2) |
    | In-place vehicle deploy or undeploy presentation | during the vehicle's turn, with no initial loop delay and repeat factor 1; no reverse request for deploy, a reverse request for undeploy (S-0517 claim 7); the type's own reverse flag still applies (claim 8) |
    | Aircraft damage smoke (S-0501 claim 37) | during the aircraft's own turn, right after its smoke chance draw, at the aircraft's position when that draw is taken, after the aircraft's own movement in that tick; created with no initial loop delay, repeat factor 1, no reverse request and no attachment (claim 22); its creation draws are claim 3's delay draw and claim 13's start-event draws, at the creation |

    Death and detonation therefore have no unconditional before/during/after classification.
    A projectile kill, an animation's damaging advance, or a death completed during an
    object's turn creates its effects **during** the pass. Damage from a storm or disc laser
    before the pass creates them **before** it (S-0203 claim 1). Surrender destruction in
    the house update creates them **after** it (S-0203 claim 8). Immediate death weapons
    inherit the death's place too (S-0312 claims 1 and 6). Other causes inherit their owning
    spec's placement; an animation is not postponed merely because its victim is different
    from the object taking the turn.

    Every animation created before or during the pass is appended and is eligible for its
    first update later in that same pass, after objects already in the update order. This
    is eligibility, not a guarantee: counting removals in S-0203 claim 10 can cost it that
    update. An animation created after the pass is first eligible next tick. Receiving its
    first update does not advance its stage or count down a loop delay (claim 5), even when
    creation was in somebody else's turn.

    In particular, there is no special next-tick rule for a projectile's detonation or for
    a death in another object's turn. The difference can instead be a removal: suppose a
    projectile is last in the update order before its turn, creates exactly one explosion
    animation, kills nothing, creates nothing else and leaves play in that turn. The new
    animation directly follows it and loses its creation-tick turn. Its first actual update
    is next tick, and with *d* = 1 its first advance is two ticks after creation. A trailer
    made by a projectile that continues flying, or a wake made by a ship that stays in play,
    normally takes its excluded first update in the creation tick. A shot animation can
    likewise take that update when its projectile keeps flying and no removal costs it a turn. Extra
    objects, death effects and counting removals must be resolved by the actual update
    order; no fixed first-update timing per visual effect replaces that rule. Turning off
    `quirks.update_order_removal_skip` removes this loss, as S-0203 claim 10 specifies.
    - Backing: `measurable` (AN26); existing phase contracts also have S-0203 claims 1,
      4, 5 and 8, S-0306 claim 16, S-0312 claim 1 and S-0504 claim 2's backing.
      Shot-animation creator settings and attachment have S-0301 claim 22's backing and
      confidence. Vehicle deploy presentations have S-0517 claim 7's backing (P1 C).
      Aircraft damage smoke has S-0501 claim 37's backing: `measurable` (S-0501 scenario AC5
      parts b to d) for its row and its delay draw. Its start-event draws are `unmeasured`
      here: their composition belongs to claim 13.
    - Confidence: high for creation placement and the conditional rule, aircraft damage smoke's
      row and delay draw included; medium for the minimal explosion/removal fixture before
      measurement, and for aircraft damage smoke's start-event draws, as S-0501 claim 37 rates
      them

## Formulas

**F1. Stage delay** (integer arithmetic, ticks). ρ is the rate value as read (S-0003 integer form).

> *d* = ⌊900 / ρ⌋ for ρ > 0, *d* = 0 for ρ ≤ 0 and ρ ≠ −1

The division is integer division of positive values, so truncation and the floor agree. ρ = −1
keeps the held *d*; with no value ever given, *d* = 1. A random pair is converted per end into
*d*<sub>1</sub> and *d*<sub>2</sub>, which give the range [*d*<sub>1</sub>′, *d*<sub>2</sub>′]
(claim 3):

> *d*<sub>2</sub>′ = max(*d*<sub>2</sub>, 0),  *d*<sub>1</sub>′ = min(*d*<sub>1</sub>, *d*<sub>2</sub>′)

Normalisation replaces *d* by N(*d*, *s*) (S-0107, F3).

**F2. Playback fields in play** (integers). *N*<sub>1</sub> is the image-frame count of the first
sprite found for the type in the game, *N*<sub>d</sub> that of the sprite that is drawn the first
time an animation plays as the type, and *N*<sub>r</sub> that of the sprite found at the theater
refresh of a new-theater type (claim 25). E<sub>s</sub>(*n*) = *n*, or ⌊*n* / 2⌋ for a shadow type.
The case "the first type read found the sprite" is assumed; claim 25 gives the other.

| End field as given | *E* in play |
|---|---|
| absent | E<sub>s</sub>(*N*<sub>1</sub>) |
| −1 | E<sub>s</sub>(*N*<sub>d</sub>) |
| 0, new-theater type whose sprite is found at the refresh | E<sub>s</sub>(*N*<sub>r</sub>) |
| 0, any other type | 0 |
| any other value *v* | *v* |

| Loop end as given | λ<sub>e</sub> in play |
|---|---|
| absent | E<sub>s</sub>(*N*<sub>1</sub>) |
| −1 | *E* |
| 0, new-theater type whose sprite is found at the refresh | *E* |
| 0, any other type | 0 |
| any other value *v* | *v* |

μ = ⌊*n* / 2⌋ with *n* the image-frame count of the last sprite found at a type read or new-theater
refresh, and 0 when none was found. The divisions are integer divisions of non-negative values. A
type that has no sprite at all uses *N* = 0 everywhere.

**F3. End of a pass** (integers, exact arithmetic). After an advance to stage *k*, with *r* the
remaining passes:

> limit = λ<sub>e</sub> − *a* when *r* ≥ 2, limit = *E* when *r* ≤ 1
>
> the pass ends ⇔ *k* ≥ limit, or (reversed and *k* ≤ 0), or (shadow type, not reversed and
> *k* ≥ λ<sub>e</sub> − *a*)

For a ping-pong type, an advance that turns (claim 9) never ends the pass; on any other advance the
rule above applies. A new pass starts when the pass ends with *r* ≥ 2; a pass that ends with *r* ≤ 1
is the last (claims 15 and 16). The new pass starts with

> *r*′ = *r* − 1 for 2 ≤ *r* ≤ 254, *r*′ = 255 for *r* = 255
>
> *k*′ = λ<sub>s</sub> − *a* (not reversed), *k*′ = λ<sub>e</sub> (reversed)

Initial values, with mod giving the least non-negative residue:

> *k*<sub>0</sub> = 0 (not reversed), *k*<sub>0</sub> = λ<sub>e</sub> − 1 (reversed)
>
> *r*<sub>0</sub> = max(1, (*L* × max(*m*, 1)) mod 256)

**F4. Translucent fade.** The normative rule is the comparison of claim 20: *k* and *E* are integers
converted exactly to floating point, and

> *P*<sub>c</sub> = *E* × *c*, formed with a 53-bit significand (S-0205, claim 4) in the
> round-toward-zero mode (glossary; S-0205, claim 5),

where *c* is the binary64 value nearest to 0.6, 0.4 or 0.2. The value for 0.6 lies just below 0.6,
the other two just above. Then

> code = the largest *c* of 1, 2 and 3 with *k* > *P*<sub>(*c*)</sub>, or 0 when there is none,
> where *P*<sub>(3)</sub> = *P*<sub>0.6</sub>, *P*<sub>(2)</sub> = *P*<sub>0.4</sub> and
> *P*<sub>(1)</sub> = *P*<sub>0.2</sub>.

For integers this reduces to:

| *E* | Code 3 | Code 2 (not 3) | Code 1 (not 2 or 3) |
|---|---|---|---|
| *E* > 0 | 5*k* ≥ 3*E* | 5*k* > 2*E* | 5*k* > *E* |
| *E* = 0 | *k* > 0 | never | never |
| *E* < 0 | 5*k* > 3*E* | 5*k* > 2*E* | 5*k* > *E* |

The reduction holds for every 32-bit integer *E* and every 32-bit integer *k*. When *E* = 0 every
product is 0.

For *E* > 0 the ≥ of code 3 is a consequence of the constant lying below 0.6 and of the rounding
mode: the product stays just below 0.6 *E* when 0.6 *E* is a whole number. With round-to-nearest
(`quirks.chop_rounding` off, S-0205) the product would round up to exactly 0.6 *E*, and code 3 would
need 5*k* > 3*E*. For *E* < 0 rounding toward zero raises each product, so it lies at or just above
the exact multiple 0.6 *E*, 0.4 *E* or 0.2 *E*, and all three comparisons are strict. A negative *E*
comes only from an explicit negative end field, which the importer warns about.

**F5. Damage accumulator** (floating point, binary64, round-toward-zero mode of S-0205, claim
5). With δ the damage value (S-0003 decimal form, so a single-precision value
held exactly in binary64) and *c* = 1 (unattached) or 5 (attached to a terrain object), at the
*j*-th advance:

> *A*<sub>0</sub> = 1.0,  *B*<sub>*j*</sub> = *A*<sub>*j*−1</sub> + *c*δ
>
> *n*<sub>*j*</sub> = trunc(*B*<sub>*j*</sub>) is delivered when *B*<sub>*j*</sub> ≥ 1.0, and
> nothing is delivered when *B*<sub>*j*</sub> < 1.0
>
> *A*<sub>*j*</sub> = *B*<sub>*j*</sub> − *n*<sub>*j*</sub> when *B*<sub>*j*</sub> ≥ 1.0, and
> *A*<sub>*j*</sub> = *B*<sub>*j*</sub> when *B*<sub>*j*</sub> < 1.0

Each product, sum and difference is a binary64 value rounded toward zero. In exact arithmetic the
total after *j* advances would be ⌊1 + *jc*δ⌋; the binary64 sums can differ from it by one when
*jc*δ lies within rounding error of a whole number. For example δ = 0.375 or 0.25 (exact in binary)
gives the delivery patterns 1, 0, 1, 0, 0, 1, 0, 1, 0, 0 and 1, 0, 0, 1, 0, 0, 0, 1 over the first
advances, and δ written as 0.4 gives 1, 0, 1, 0, 1, 0, 0, 1, 0, 1. After a follow-up switch the
accumulator restarts at 0.0 instead of 1.0. With `quirks.anim_damage_starts_at_one` off,
*A*<sub>0</sub> = 0.0.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Rate conversion | 900 | ticks per game minute | F1; the rate field is image frames per game minute |
| Stage delay when the rate field is absent | 1 | tick | F1 |
| "Not given" rate | −1 | rate value | S-0006, claim 3 |
| Remaining-pass modulus | 256 | passes | claim 7 |
| Endless pass count | 255 | passes | claim 7 |
| Smallest repeat factor | 1 | factor | claim 7; smaller factors count as 1 |
| Middle image frame | ⌊*N* / 2⌋ | image frame | claims 14, 25 |
| Fade thresholds | 0.6, 0.4, 0.2 | fraction of *E* | claim 20, F4 |
| Translucency field values | 25, 50, 75 | per cent | claim 20; other positive values are opaque |
| Translucency level bands | 1–5, 6–15, > 15 | level | claim 20 |
| Hide threshold for translucent-flag and translucency-field types | 15 | level | claim 20 |
| Translucency level of a slot animation while its building is cloaked from the viewer | 16 | level | claim 20 |
| Building fade level of a fully cloaked building | 15 | level | Terms, claim 20 |
| Height limit for scorch marks and craters | 30 | leptons, height above the terrain | claim 14; the middle event makes them only below it |
| Damage accumulator start | 1.0 | damage points | F5 |
| Damage accumulator after a follow-up switch | 0.0 | damage points | F5 |
| Damage factor when attached to a terrain object | 5 | factor | claim 17 |
| Detail level above which the reduced mode hides animations | 1 | detail level | claim 19 (departure) |

## State and ordering

**Playback states of one animation.**

| State | Event | Result |
|---|---|---|
| created | creator gives a loop delay *D* > 0 | *delayed*; the first update after creation is skipped, then *D* updates count down |
| created | no loop delay | start event in the same tick; *playing* |
| *delayed* | count reaches 0 | start event; no advance on that update; *playing*. The count and the start event go on while the animation is held (claim 23) |
| *playing* | held (claim 23) | no advance on that update; the clock of claim 5 keeps running |
| *playing* | *d* ticks have passed since the last advance, the update is not skipped, the animation is not held, and *d* > 0 | The next image frame is shown if it is in range; damage due on this advance is delivered first, and then the middle event occurs when the resulting stage satisfies *a* + *k* = μ (Observable order within one update). A turn or pass end has the consequences specified in claims 6, 8, 9 and 10: in particular, the image frame at an ending limit is not drawn, and any same-update event or follow-up switch remains observable as those claims state. |
| *playing* | pass ends with *r* ≥ 2 | new pass (F3); if the random loop delay pair is not (0, 0), ranged draw of *D*, and *delayed* when *D* > 0 |
| *playing* | pass ends with *r* ≤ 1, follow-up type named | follow-up switch of claim 15, start event, *playing* |
| *playing* | pass ends with *r* ≤ 1, no follow-up type | removed, except that a refused infantry placement keeps the animation for a retry under S-0611 claims 9 to 11 |
| burning walker | running or expiring | playback and removal follow S-0615 claims 4 to 11, not the ordinary transitions above |
| any | owner removed | not drawn from then on; removed at the next update, which has no advance, damage, event, follow-up switch or infantry (claim 22) |

**Observable order within one update.** Damage, the middle event and the end test all follow the
advance of the same update, in this order (Q-1716):
- first the damage due on the advance (claim 17). AN28 observes that its detonation's resource
  change comes before the middle event. The rest of that detonation, every random draw and any
  kill it causes, is complete before the middle event begins too; that order within the tick is
  `unmeasured`, medium (claim 14).
- then the middle event, when the resulting stage satisfies *a* + *k* = μ (claim 14): its particle
  requests, then its scorch-or-crater choice and any crater's resource removal.
- last the turn and pass tests (claims 6 to 10). A pass end then gives a new pass with its
  loop-delay draw (claim 11), the follow-up switch with its delay draw and start event (claims 13
  and 15), or the end (claim 16).

The same order holds on the advance that ends the animation and on the advance just before a
follow-up switch: the old type's damage and middle event both come before the switch, and the
follow-up type's start event, with any immediate middle event of its own when its μ is 0, comes
after them. So:
- an animation delivers damage on the advance that ends it,
- when μ lies at the end limit, the middle event and the end (or the follow-up switch) happen in the
  same tick,
- the image frame at the end limit of a pass that ends there is never drawn, and the follow-up
  type's first image frame is drawn from the tick of the switch.

**An animation that leaves play in its own detonation.** When an animation leaves play during its
own damage detonation, nothing else of this spec happens to it in that update: no middle event, no
pass end, no follow-up switch and no end. One way is known: the animation plays in a tank bunker's
*special-1* or *special-2* slot (S-0606, Terms), and its detonation destroys the bunkered vehicle,
so the walls-down effects stop those two animations at once (S-0606 claims 12 and 14). In the
original the result of the rest of that update is undefined; our engine stops (design). An owner
removed by the detonation is a different case: the animation stays in play until its next update,
so its update goes on (claim 22; Open questions).
- Backing: `unmeasured`: the original's result is undefined, so no measurement can show a
  behaviour to copy, and the stop is our engine's rule.
- Confidence: medium.

## Edge cases and quirks

- **`quirks.anim_fast_rate_freezes`** (default: on). A rate field above 900 gives a delay of 0, so
  the animation never advances or ends through playback (claim 2). Off: a positive rate gives *d* =
  max(1, ⌊900 / ρ⌋). Rates of 0 or below still give 0 in both settings, except that −1
  keeps the held delay (claim 2). This conversion also supplies the delay used for a
  vehicle's in-place deploy and undeploy completion (S-0517 claim 7). Thus a rate above 900
  can stall that transition under `quirks.deploy_stall_zero_delay`; turning
  `quirks.anim_fast_rate_freezes` off removes that fast-rate cause of the stall, but does
  not change the zero-delay rule for other rates. S-0517 claim 7 owns completion timing,
  backing and confidence.
- **`quirks.anim_random_rate_collapse`** (default: on). A random rate pair is clamped after
  conversion, so an ascending pair collapses to its faster end and takes no draw (claim 3). Off:
  the converted pair is put in order and the delay is a ranged draw over [min(*d*<sub>1</sub>,
  *d*<sub>2</sub>), max(*d*<sub>1</sub>, *d*<sub>2</sub>)], with no draw when they are equal.
  Turning it off changes the number of draws, so it must be the same on every machine of a game.
- **`quirks.anim_loop_count_wrap`** (default: on). The remaining-pass count is taken modulo 256
  and 255 means endless (claim 7). Off: the count is an unbounded integer. At creation it is
  max(1, *L* × max(*m*, 1)) when *L* ≥ 0, and the animation is endless when *L* < 0. Every pass end
  with *r* ≥ 2 lowers it by 1, so 255 is an ordinary count. At a follow-up switch *r* = *L*′ when
  *L*′ ≥ 0, and endless when *L*′ < 0.
- **`quirks.anim_reverse_raw_loop_end`** (default: on). A reversed animation starts at λ<sub>e</sub>
  − 1 and restarts at λ<sub>e</sub>, without subtracting *a*, and never shows image frame *a* (claim
  8). Off: it starts and restarts at *k* = λ<sub>e</sub> − *a* − 1 and its pass ends when *k* < 0,
  so it shows λ<sub>e</sub> − 1 down to *a*.
- **`quirks.anim_pingpong_lower_turn`** (default: on). While *r* ≥ 2 the lower turn of a ping-pong
  animation compares the stage with the start field instead of with the loop start, so it can miss
  the turn and run blank (claim 9). Off: the lower turn while *r* ≥ 2 happens when *k* ≤
  λ<sub>s</sub> − *a*.
- **`quirks.anim_loop_end_from_sprite`** (default: on). An absent loop end takes the sprite end
  *E*<sub>s</sub> even when the end field is given (claims 6 and 25). Off: an absent loop end
  takes the end field *E*. Stock data is affected only where a type gives an end field without a
  loop end and loops, is reversed or is a shadow type.
- **`quirks.anim_followup_start_doubled`** (default: on). The follow-up type starts at *k* = *a*′,
  showing image frame 2*a*′ (claim 15). Off: it starts at *k*<sub>0</sub> of F3 for the follow-up
  type: *k* = 0, or *k* = λ<sub>e</sub>′ − 1 when the animation is reversed under the follow-up
  type (Terms).
- **`quirks.anim_followup_keeps_step`** (default: on). At a follow-up switch σ keeps its sign while
  the reversed test follows the follow-up type (claim 15), so an animation reversed by its old
  type's flag can run down for ever, invisible. Off: σ is set afresh at the switch, −1 when the
  animation is reversed under the follow-up type and +1 otherwise.
- **`quirks.anim_translucency_levels`** (default: on). Translucency levels 11 to 15 give code 2,
  like 6 to 10 (claim 20). Off: 11 to 15 give code 3.
- **`quirks.anim_hide_at_level_15`** (default: on). Translucent-flag and translucency-field types
  are hidden at *t* ≥ 15, a type with neither only at *t* > 15 (claim 20), so the owner of a fully
  cloaked building sees some of its slot animations and not others. Off: every row hides at *t* >
  15, so at *t* = 15 the first two rows draw with their own codes.
- **`quirks.anim_low_detail_shows_cloaked`** (default: on). Below the translucency detail field the
  hide rules of claim 20 do not apply, so a slot animation of a building cloaked from the viewer is
  drawn, opaque. Off: an animation with *t* > 15 is not drawn at any detail level; below the
  translucency detail field the code stays 0 otherwise.
- **`quirks.anim_damage_starts_at_one`** (default: on). The damage accumulator starts at 1.0 when an
  animation is created, so the first advance delivers at least 1, but at 0.0 after a follow-up
  switch, by claim 17 and F5. Off: it starts at 0.0 in both cases, so the total after *n* advances
  is close to ⌊*n c* δ⌋.
- **`quirks.speed_normalised_delays`** (default: on, shared with S-0107, S-0108 and S-0111; glossary
  claim G3). Claims 4 and 18.
  Off: every animation uses its unnormalised delay at every game speed.
- **Round-toward-zero mode** (no switch here). F4 and F5 depend on the mode; the switch
  `quirks.chop_rounding` belongs to S-0205 (claim 5).
- **Reduced drawing on slow machines** (departure, no switch). Claim 19.
- **Unknown follow-up or missing type.** A follow-up name that names no animation in the list
  creates an animation type of that name with absent-key defaults (S-0004). If no sprite resolves
  for it, it draws nothing and its end field is 0 (claim 25), so after the switch it ends on its
  first advance, unless it keeps σ = −1 without being reversed (claim 15), in which case it never
  ends. An animation created with no type at all has no ordinary playback. This does not imply
  immediate loss of every attachment effect: S-0522 claim 13 owns the absent-parachute lifecycle,
  including its release-tick fall-rate exception and the alternative under
  `quirks.paratrooper_any_animation_slow_fall`. A defined type with no sprite instead follows claim
  25 and the missing-sprite rule below. The importer warns about invalid references where their
  cause is in the data; a genuinely absent parachute type is allowed.
- **Sprite missing.** An animation whose sprite cannot be resolved is updated but draws nothing; by
  claim 25 an absent end field is then 0, so an animation created as such a type, with one pass and
  without the ping-pong flag, ends on its first advance (claims 7 and 9 give the other cases). An
  end field or loop end of −1 on a type that has no sprite when an animation first plays as it makes
  the original crash. Our engine resolves it with *N* = 0, so *E* = 0, and the importer warns. This
  is a departure, with no switch.

## Import mapping

Except for the two theater flags below, every animation-type key in this table is read
exclusively from the **own art section**, whose section name is the animation type name,
at **every type read** (S-0006 claim 8; S-0007 claim 34). An image name that names a different
art section does not redirect these reads. Neither a previously found sprite, a failed sprite
lookup, the arctic suffix nor first registration in a later rules layer changes this rule.
A missing or empty own-section value never falls back to the image-named section: the
field keeps its specified default, retained value or sprite-derived fill. Claims 2 and 3
retain their rate sentinels, and claim 25 retains all sprite-derived defaults and refresh rules.
This applies equally to building slots and independent effects. The theater flags alone
follow S-0007 claim 2's read-history exception. Game-wide keys below are unchanged.
Value forms are those of S-0003.

Section selection has high confidence and is `measurable` through S-0007 scenario A25
(the companion scenario below); it does not assert new measurements of each field's consumer.

| Key | Form | Feeds | Absent |
|---|---|---|---|
| `Rate` | integer | ρ, stage delay (F1) | delay 1 tick; −1 keeps the held value |
| `RandomRate` | two integers | random delay pair, including unseen and spriteless playback (claims 3 and 28) | no random rate; −1 in either place keeps that end |
| `Normalized` | boolean | claim 4 | no |
| `Start` | integer | *a* | 0 |
| `End` | integer | *E* | *E*<sub>s</sub> of the first sprite found, *N* or ⌊*N*/2⌋ with `Shadow`; −1 gives *E*<sub>s</sub> of the drawn sprite; 0 is kept except for new-theater types (Terms, claim 25, F2) |
| `LoopStart` | integer | λ<sub>s</sub> | 0 |
| `LoopEnd` | integer | λ<sub>e</sub> | *E*<sub>s</sub> of the first sprite found, not `End` (`quirks.anim_loop_end_from_sprite`; claim 25 for a type whose earlier reads found no sprite); −1 gives *E*; 0 is kept except for new-theater types (Terms, claim 25, F2) |
| `LoopCount` | integer | *L* (claim 7) | 0, which gives one pass |
| `Reverse` | boolean | reversed (claim 8) | no |
| `PingPong` | boolean | claim 9 | no |
| `Shadow` | boolean | shadow type, drawn after the image at the unshifted point (claim 10) | no |
| `YDrawOffset` | integer, pixels | vertical image shift *v*, not a shadow shift (claims 26 and 27); depth effect in S-0102 claim 13 | 0 |
| `Tiled` | boolean | stack of image copies (claim 27), suppressing the shadow (claim 10) | no |
| `Flat` | boolean | flat drawing profile of S-0102 claim 13, with claim 26's image point and no shadow image frame | no |
| `RandomLoopDelay` | two integers, updates | scenario-stream loop delay pair, including unseen and spriteless playback (claims 11 and 28) | 0,0 (none) |
| `Next` | type name | follow-up type and its own-turn delay choice (claims 15 and 28) | none |
| `Damage` | decimal | δ (claim 17) | 0 |
| `DetailLevel` | integer | detail field (claim 19) | 0 |
| `TranslucencyDetailLevel` | integer | translucency detail field (claim 20) | 0 |
| `Translucent` | boolean | fade by progress (claim 20) | no |
| `Translucency` | integer | translucency field (claim 20) | 0 |
| `DoubleThick` | boolean | claim 21 | no |
| `ShouldFogRemove` | boolean | fog-remove flag (claim 24) | yes |
| `StartSound` | single sound-entry name, case-insensitive, first 127 value characters; unknown retains earlier reference | start sound (claim 13; S-0701 claim 22) | retains earlier reference, initially no sound |
| `Report` | same single-reference form; read only when the start reference after reading `StartSound` is still no sound | fallback alias for the start sound, not an additional sound | no sound |
| `StopSound` | same single-reference form; unknown retains earlier reference | end sound (S-0701 claim 22) | retains earlier reference, initially no sound |
| `TiberiumChainReaction` | boolean | resource chain reaction of the start event (claim 13; S-0401 claim 20) | no |
| `SpawnsParticle` | particle-type name | particle requests of the middle event (claim 14) | none |
| `NumParticles` | integer | number of particle requests per middle event (claim 14) | 0 |
| `Scorch`, `Crater` | boolean | scorch and crater requests of the middle event (claim 14) | no |
| `ForceBigCraters` | boolean | request a large crater when a crater is chosen (S-0202 claim 15 owns size selection during loading and play) | no |
| `Bouncer`, `IsMeteor` | boolean | bouncing type (Terms) | no |
| `MakeInfantry` | integer | infantry creation at the end (claim 16; S-0611 claims 9 to 11 for selection, placement and retries) | −1 (none) |
| `IsFlamingGuy` | boolean | burning-walker flag (S-0615 claims 4 to 11) | no |
| `Theater`, `NewTheater` | boolean | which sprites the type gets (S-0007 claim 14), and so claim 25; section selection follows S-0007 claim 2, not the own-only rule above | initially no; later retention and own/image-section precedence follow S-0007 claim 2 |

A retained start-sound reference also prevents `Report` from being read in a later layer
that omits `StartSound`; this includes a reference previously supplied by the alias.
These sound-reference forms, retention and fallback precedence have high confidence and
are `measurable` in S-0701 scenario F3, including distinct aliases and repeated layers.

Game-wide keys:

| Section and key | Form | Feeds | Absent |
|---|---|---|---|
| rules, combat-damage section, `C4Warhead` | warhead name | warhead of claim 17 for the built-in invisibility-animation type, identified by an exact, byte-for-byte type-name comparison | none: no damage is done |
| rules, combat-damage section, `FlameDamage2` | warhead name | warhead of claim 17 for every other type | none: no damage is done |
| settings file, detail level | integer | claim 19 (S-0104 Import mapping) | as in S-0104 |

Resource-damage inputs delegated to S-0303 claim 17:

| Section and key | Form | Feeds | Absent |
|---|---|---|---|
| rules, selected warhead section, `Tiberium` | boolean | resource flag for claim 17's area detonation | no |
| rules, overlay section, `Tiberium` | boolean | resource overlay eligibility under S-0303 claim 17 | no |
| rules, overlay section, `ChainReaction` | boolean | overlay chain-reaction gate under S-0303 claim 17 | no |

The animation damage producer's permission and absent attacker and attacking house are fixed
behaviour, not additional imported settings. These rows do not enable the separate start-event
reaction; that uses `TiberiumChainReaction` above. The delegated gates have S-0303 claim 17's
backing and confidence; AN27 tests their composition with animation damage.

The built-in invisibility-animation type is identified by an exact, byte-for-byte comparison of the
current type's name with the name `INVISO` (six ASCII capital letters, letter case included). Types
are found by name without regard to letter case, and a type keeps the spelling it was first
registered with (S-0004 claims 2 and 7). A type first registered as `Inviso` is the same type as any
later mention in other cases, and it never matches, so it takes `FlameDamage2`. When the rules
register no type of exactly that name, `C4Warhead` is never chosen for animation damage (Q-1717).
The stock rules' animation list registers a type of exactly that spelling, so in stock play the type
exists and is first registered in capitals.
- Backing: `measurable` (AN29) for the exact-name choice and the switch at a follow-up type,
  and `measurable` (AN29 part b) for the letter case and for the case with no type of exactly
  that name, since a test copy of the rules that spells that one list entry in other letter case
  registers the type in that spelling and leaves no type of the exact name. The stock
  registration is `measurable` through AN29's main run, which keeps the stock spelling: its
  building loses health only if the stock data first registers the type in exactly that
  spelling, and part b shows that any other first spelling gives no loss. AN29 part b's pilot
  check, a search of the whole installed rules file and not only its animation list, confirms
  the fixture: the list entry is spelled in capitals and no other value or section heading
  names the type.
- Confidence: medium for the name, the current-type rule and the stock registration until AN29
  runs, and for the case-sensitive comparison and for the no-type case until AN29 part
  b runs; the kept first spelling has S-0004's confidence.

Import defaults and sentinel cases not explicitly exercised by the named scenarios are
`unmeasured` here (confidence: medium): no scenario systematically omits each key in each layer
or supplies every sentinel value. Playback effects of the selected test values use the
numbered claims' backings. Quirk-off alternatives, importer warnings and safe bad-data
results are design rules (confidence: high), not oracle measurements. The absent-parachute
lifecycle in Edge cases inherits S-0522 claim 13's backing and confidence (its scenario P8).
Other missing-type creators remain `unmeasured` here (confidence: medium): no scenario here
creates an animation without a type. Missing-sprite ordinary playback is `measurable` through AN25
(confidence: high); the crash with missing-sprite −1 limits remains `unmeasured` here
because AN25 deliberately avoids those limits.

An explicitly written zero end field or loop end is an input-value rule, not an optional
playback quirk. Its result is claim 25 and F2's result, including the new-theater refresh
exception. The importer warns but does not change that result.

The importer warns about the following inputs, including quirk-sensitive values, for:
- a `RandomRate` pair written in ascending order (claim 3),
- a `Rate` above 900 (claim 2),
- a `LoopCount` outside 0 to 255 other than −1 (claim 7),
- a ping-pong type with `Start` greater than 0 (claim 9),
- a `Next` type with `Start` greater than 0 (claim 15),
- a `Next` type whose `Reverse` differs from that of the type naming it (claim 15),
- `End=0` or `LoopEnd=0` written explicitly (claim 25),
- an `End` below −1 (F4),
- `End=-1` or `LoopEnd=-1` on a type whose sprite cannot be resolved (Edge cases),
- a type that gives `End` but no `LoopEnd` while it loops, is reversed or has `Shadow` (claim 6).

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*. Animations are played on open flat ground of a
tiny purpose-made skirmish map by a trigger action that plays an animation at a waypoint marker (the
trigger specs own its form), or by the object that normally creates them. Include a human measuring
player and one bystander computer player, kept away under the glossary's void rule. The observed
area is the test waypoint cells and the cells occupied or crossed by named test objects. No engine
add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). No
helper that changes rules, houses or computer players during the game is active, and no scenario
here needs one. Trigger-played animations have a creation start sound but no later start sounds and
no damage (claims 13 and 17), so scenarios that observe repeated sounds or damage use another
creator: the explosion animation of one stock weapon, fired by a unit. Such scenarios assume the
weapon's explosion is created with a repeat factor of 1, no loop delay and no reverse play; a pilot
run with the unchanged animation (its pass count and first image frame) checks that. Because rules
layers cannot change animation fields (S-0006, claim 8), every scenario that needs particular field
values uses named test data: a **test expansion archive** holding a copy of the art database with
the listed test sections added (as in S-0103 and S-0007), each naming a stock sprite; the archive is
removed after the run. Record per tick, with a recorder that reads the running game's state (the
animation's image frame, a target's health) at the same point of every tick, or with a screen
capture that keeps every drawn frame together with the tick counter. The *hold* of an image frame is
the number of consecutive ticks on which it is recorded. A sprite with *N* image frames whose image
frames are visibly different is chosen for each test section; *N* is read from the sprite at
measurement time. Game speed 4 unless stated. Runs: one run per listed case, since playback is
deterministic, unless a scenario gives another number. Where a scenario needs particular rules
values (a building that can cloak), it sets them in the test map's override section, which rules
layering allows.

- **AN1 (claims 1, 2, 5, 6, 10, 16).** Test types T1 (rate 100, no loop fields), T2 (rate 300, end
  6), T3 (rate 300, end *N* + 2), T4 (rate 300, shadow flag, a sprite with shadows), T5 (rate 900),
  T6 (rate absent from every read, one pass, end 6). Play each once, in a run of its own, by a
  trigger action of a one-time polled trigger whose only event is elapsed-time (S-0011, claim 13).
  The polling of map triggers sets it off, so the animation is created before the turn pass of its
  tick (S-0203, claim 1), and no object leaves play on the test map during the run, so no removal
  costs it a turn (S-0203, claim 10).
  - Observable: the sequence of image frames, the hold of each, lifetime in ticks.
  - Runs: one per type; T5 three times, with different event values.
  - Expected: T1 shows 0 … *N* − 1 with every image frame, the first included, held ⌊900/100⌋ = 9
    ticks, and its lifetime is exactly 9*N* ticks. T2 shows 0 … 5, 3 ticks each. T3 shows 0 … *N* −
    1 and then nothing for 2 × 3 ticks before it ends. T4 shows image frames 0 … ⌊*N*/2⌋ − 1, each
    with a shadow taken from the second half. T5 holds its first image frame exactly 1 tick in every
    run, since it takes its first turn in the tick it is created (claim 5), and every later one
    exactly 1 tick. T6 shows image frames 0 … 5 for 1 tick each and ends 6 ticks after creation.
    For T5 and T6, record creation tick *c*: the initial image is present at the end of tick *c*,
    image frame 1 first appears at the end of tick *c* + 1. This directly tests that the
    excluded first update is in *c*, not the following tick.
  - Falsified if: T5 or T6 advances in its creation tick or first advances after *c* + 1, T6's hold
    or lifetime differs, any hold of T1 or T2 differs from ⌊900/ρ⌋, T5's first hold is not 1 in any
    run (a first hold of 2 would mean that the animation took its first turn only in the next tick)
    or a later hold is not 1, the image frame at the end limit is ever drawn, T3 ends without
    showing nothing for exactly 2 × 3 ticks after image frame *N* − 1, or T4 plays into its shadow
    image frames.
- **AN2 (claim 2).** Test type with rate 1000, played by a trigger action.
  - Observable: the image frame per tick for 600 ticks, and whether the animation is removed.
  - Runs: one.
  - Expected: it shows its first image frame on every one of the 600 ticks and is not removed.
  - Falsified if: any other image frame is recorded, or it is removed within the 600 ticks.
  - Vehicle variant: use S-0517 scenario P1's named in-place deploy test vehicle on the
    ground, with this presentation, no random rate or normalisation, and end field 5.
    Give a deploy order and record transition state for 600 ticks, once. Expected in the
    oracle: no completion, as for P1 Z's zero-rate presentation. Completion falsifies the
    fast-rate stall. On our engine only, repeat with `quirks.anim_fast_rate_freezes` off
    and `quirks.deploy_stall_zero_delay` on: the delay is 1 tick and completion follows
    S-0517 claim 7. This last comparison tests the design alternative, not the oracle.
- **AN3 (claims 3, 12).** Test types R1 (random rate 225, 450: ascending) and R2 (random rate 450,
  225: descending), played by a trigger action.
  - Observable: the hold per image frame of each run, and its first image frame; with a recorder
    that reads the state of the scenario random stream, the raw values the scenario random stream
    consumes between the tick before the run's animation is created and the tick it is created,
    compared with the raw values S-0202's range rule predicts for the state recorded before that
    tick.
  - Runs: 30 per type, in separate games or at separate times. Repeat R2 with two passes,
    no loop delay and loop start 0, in 30 further runs; record stream state at its pass end.
    No other random playback or middle/start effect is enabled in these tests.
  - Expected: R1 always holds ⌊900/450⌋ = 2 ticks. Each run of R2 holds one of 2, 3 or 4 ticks
    throughout, and all three values occur in the 30 runs (under the hypothesis a value is missing
    with probability at most 3 × (2/3)<sup>30</sup>, about 0.002 %). Every run starts at image
    frame 0.
  - Expected (draws): R1 consumes no raw value for the delay (no ranged draw); R2 consumes the raw
    values of one ranged draw over its converted range, which the recorded state before creation
    predicts value for value. The two-pass R2 variant uses the same delay in both passes
    and consumes no raw value for a delay at the pass boundary.
  - Falsified if: the two-pass variant rerolls or consumes a delay draw at a new pass, R1 consumes a
    raw value for the delay, R2 consumes values the range rule does not predict from the recorded
    state, R1 ever holds 3 or 4 ticks, an R2 run holds a value outside 2 to 4, fewer than three
    distinct values occur in R2's 30 runs, or a run starts elsewhere.
- **AN4 (claims 4, 17, 18).** Test types U (rate 90, not normalised) and V (rate 90, normalised),
  played by a trigger action at game speeds 0, 3 and 6. In a second part, the test copy of the art
  database gives the explosion animation of one stock weapon the fields of V plus damage 0.25, end
  8 and one pass; a unit fires the weapon at a stock building that the game-wide warhead damages
  (checked in a pilot run), at game speeds 0 and 6.
  - Observable: the hold per image frame of U and V; in the second part, the ticks at which the
    building's health drops, apart from the weapon's own impact tick.
  - Runs: one per type and speed; the second part three shots per speed.
  - Expected: U holds 10 ticks at every speed. V holds N(10, *s*) = ⌊80/(*s* + 1)⌋ ticks: 80, 20 and
    11. In the second part the health drops on the 1st, 4th and 8th advance (F5), so the drops
    are 3 × N(10, *s*) and then 4 × N(10, *s*) ticks apart: 240 and 320 ticks at speed 0, 33 and 44
    at speed 6.
  - Falsified if: U's hold changes with speed, V's does not follow N, or the drops fall on other
    advances or other spacings.
- **AN5 (claims 6, 7).** Test types, all with rate 300 and played by a trigger action: W1 with start
  2, loop start 4, loop end 7, end 9, loop count 3, on a sprite with *N* ≥ 12; W2 with loop count
  256; W3 with loop count −1; W4 with end 6, loop count 2 and no loop start or loop end, on a sprite
  with *N* ≥ 10.
  - Observable: the sequence of image frames and whether and when the animation is removed.
  - Runs: one per type; W3 watched for 2000 ticks.
  - Expected: W1 shows 2 3 4 5 6, 4 5 6, 4 5 6 7 8 9 10, then is removed. W2 plays one pass and is
    removed. W3 is still looping after 2000 ticks. W4 shows 0 … *N* − 1 and then 0 … 5 (the absent
    loop end is *N*, claim 6), then is removed.
  - Falsified if: any pass of W1 deviates, W2 plays more than one pass, W3 ends, or W4's first pass
    stops at 5.
- **AN6 (claim 8).** Test types Q1 (reverse flag, start 0, end 8, loop end 8, loop count 2, rate
  300) and Q2 (start 3, otherwise as Q1), on a sprite with *N* ≥ 12, played by a trigger action.
  - Observable: the sequence of image frames until removal.
  - Runs: one per type.
  - Expected: Q1 shows 7 … 1, then 8 … 1, then is removed; image frame 0 never shown. Q2 shows 10
    (its first pass ends on the first advance, claim 8), then 11, 10, 9 … 4, then is removed.
  - Falsified if: either sequence differs, including Q1 showing image frame 0 or starting
    its first pass at 8.
- **AN7 (claim 9).** Test types P1 (ping-pong, loop count 1, end 6, rate 300), P2 (ping-pong, loop
  count −1, start 0, loop end 5, rate 300), P3 (ping-pong, loop count −1, start 4, loop start 4,
  loop end 8, rate 300), P4 (ping-pong, shadow flag, loop count 1, end 6, loop end 3, rate 300, a
  sprite with shadows and *N* ≥ 12), played by a trigger action.
  - Observable: the sequence of image frames, and whether the animation is removed within 2000
    ticks.
  - Runs: one per type.
  - Expected: P1 cycles 0 … 6, 5 … 0, 1 … without ending, image frame 6 drawn only if *N* > 6; P2
    cycles 0 … 5 … 0 without ending; P3 shows 4 … 8, then 7 … 0, then nothing, without ending; P4
    shows 0, 1, 2 and is removed, without turning.
  - Falsified if: any recorded sequence differs from its prediction, P3 turns at image
    frame 4 coming down, P1, P2 or P3 is removed, or P4 shows image frame 3 or keeps playing.
- **AN8 (claim 10).** Test types S1 (shadow flag, loop end 3, end 6, loop count 1, rate 300), S2
  (shadow and flat flags) and S3 (shadow and tiled flags), each on a sprite with shadows, played by
  a trigger action.
  - Observable: the sequence of image frames of S1; whether any shadow pixels appear under S2 and
    S3 (screen capture against flat ground).
  - Runs: one per type.
  - Expected: S1 shows 0, 1, 2 and is removed. No shadow is drawn under S2 or S3.
  - Falsified if: S1 shows image frame 3, or a shadow appears under S2 or S3.
- **AN9 (claims 11, 13).** The test copy of the art database gives the explosion animation of one
  stock weapon a loop count of 3, a random loop delay of 30, 60, a start sound and a rate of 300; a
  unit fires that weapon at open ground. A second run plays the same animation by a trigger action.
  - Observable: ticks between passes, start sounds heard, image frame during the gap, and
    incoming/outgoing scenario-stream states at each pass end. Disable particles, scorch,
    crater, chain reaction and random rates in the test section. The incoming state predicts
    *D* by S-0202's ranged-draw rule; do not infer *D* only from a measured gap.
  - Runs: 20 animations per run.
  - Expected: every gap is max(3, *D* + 1) with *D* from 30 to 60, the loop start image frame is
    shown during the gap, and the sound plays at creation and after each of the two gaps (three
    times per animation). The trigger-played run shows the same gaps and image frames, with its
    start sound only at creation. Each pass end consumes exactly the raw values predicted for one
    ranged draw over [30, 60] updates, with no other playback draw; the measured gap is *D* + 1
    ticks for that predicted *D*.
  - Falsified if: the stream transition differs from that one draw or the gap differs from the
    predicted *D* + 1, a gap lies outside 31 … 61, the animation vanishes during a gap, the weapon
    run's sound plays once per animation, or the trigger-played run has no creation sound or plays a
    sound after a gap. A contrary trigger-played result calls for investigation of claim 13, not
    adoption of creation-time suppression as an alternative rule.
- **AN10 (claims 13, 14).** A test type with the scorch flag (no crater flag), rate 300, on a sprite
  with *N* ≥ 4, played by a trigger action, once with start 0 (M1) and once with start ⌊*N*/2⌋ + 1
  (M2).
  - Observable: the tick at which a scorch mark appears in the animation's cell, read by the
    recorder from the cell's state, and the tick each image frame is first shown.
  - Runs: one per type.
  - Expected: for M1 a scorch mark appears on the tick image frame ⌊*N*/2⌋ is first shown; M2 never
    makes one. This also shows that trigger-played animations keep their middle events.
  - Falsified if: M1 makes no scorch mark or makes it on another tick, or M2 makes one.
  - Particle variant: on the same map at game speed 2, use named test art to give M1 a
    positive particle count and a named particle type. Run once with a gas-behaviour type
    and once with a smoke-behaviour type. Record each requested particle's system membership
    at creation, before subsequent particle updates, and the number of particle systems.
    Expected: in both runs the requested particles belong to the existing, ownerless shared
    gas system of S-0613 claim 1; the event creates no new particle system. Falsified by
    either type using a different system, or by the event creating a system.
- **AN11 (claim 15).** Test types F (rate 300, end 4, follow-up G) and G (start 2, loop count 0,
  rate 150, no end field, sprite with *N*<sub>G</sub> image frames); a second pair F2 (reverse flag,
  end 4, loop end 4, rate 300, follow-up G2) and G2 (start 2, end 6, rate 300, no reverse flag). F
  and F2 are played by a trigger action.
  - Observable: the image frames shown, their holds, and whether the animation is removed within
    2000 ticks.
  - Runs: one per pair.
  - Expected: F shows 0 1 2 3, then G shows image frames 4, 5, … *N*<sub>G</sub> + 1 at 6 ticks each
    (the last two blank), one pass, then removal. F2 shows 3, 2, 1, then G2 shows 4, 3, 2, 1, 0 and
    then nothing, and is not removed.
  - Falsified if: either image-frame sequence or any predicted hold differs, G starts at
    image frame 2 or plays two passes, G2 plays upwards, or G2 ends.
- **AN12 (claims 17, 18).** The test copy of the art database gives the explosion animation of one
  stock weapon a damage of 0.375, rate 300, one pass and end 10. A unit fires the weapon at a stock
  building that the game-wide warhead damages (checked in a pilot run). A control run plays the same
  animation by a trigger action next to the building.
  - Observable: the building's health per tick, read by the recorder, and the ticks of image frame
    changes.
  - Runs: three shots, one control. Repeat the damaging run with end 8 for three more shots
    to put a nonzero delivery on the ending advance.
  - Expected: apart from the weapon's own impact tick, health drops on the 1st, 3rd, 6th and 8th
    advances and on no others (F5's pattern 1, 0, 1, 0, 0, 1, 0, 1, 0, 0), each on the tick of an
    image frame change except an ending advance, recorded through removal instead. In the end-8
    variant the 8th advance removes the animation and still delivers its damage in that tick. The
    control run delivers nothing.
  - Falsified if: the ending advance in the end-8 variant delivers nothing,
    the first advance delivers nothing, a drop falls on another advance or between
    advances, or the control run delivers damage. (Totals are not compared, since warhead scaling of
    a 1-point hit belongs to the combat specs.)
- **AN13 (claim 19).** Test type with detail field 2, rate 300, played by a trigger action at detail
  levels 1 and 2.
  - Observable: whether the animation's pixels appear (screen capture), and its lifetime in ticks
    from the recorder.
  - Runs: one per detail level.
  - Expected: no pixels at level 1, visible at level 2, and the same lifetime at both.
  - Falsified if: it is drawn at level 1, not drawn at level 2, or the lifetimes differ.
- **AN14 (claims 20, 21, F4).** Test types: X1 translucent flag, end 10, rate 300; X2..X5
  translucency 25, 50, 75, 40; X6 translucent flag with translucency detail field 2, at detail level
  1; X7 translucent flag, explicit end 0, rate 9 (100 ticks). A test building with two slot
  animations, one with the double-thick flag and one without, is hit by an erasing weapon.
  - Observable: the blend of each image frame against a uniform background (screen capture).
  - Runs: one per type; the erasure case one run.
  - Expected: X1 opaque for stages 0–2, 25 % for 3–4, 50 % for 5, 75 % for 6–9. X2..X4 at 25, 50,
    75 %; X5 opaque; X6 opaque. X7 shows stage 0 opaque for 100 ticks and ends. From the tick the
    erasing weapon locks on, the double-thick slot animation is drawn at 75 % and the other at 50 %.
  - Falsified if: stage 6 of X1 is drawn at 50 % (that would mean round-to-nearest at 53 or 24
    bits), stage 2 is translucent, X7 is drawn translucent, the erased building's slot animations
    keep their earlier look, or a value differs.
- **AN15 (claim 22).** A stock vehicle with an attached animation (for example a burning or smoking
  animation a damaged vehicle carries), driven across the map and then destroyed.
  - Observable: per tick, the animation's position and the vehicle's position (recorder), and the
    tick on which each is removed.
  - Runs: three.
  - Expected: the difference between the two positions stays constant while the vehicle moves; the
    animation is no longer drawn from the tick the vehicle is removed, and it is gone from the game
    no later than the next tick, with no image frame change in between.
  - Falsified if: the difference changes while attached, the animation is drawn after the vehicle's
    removal, it still exists after the following tick, or its image frame changes after removal.
- **AN16 (claims 21, 23).** A test building that needs power, with test slot animations of rate 9
  (100 ticks each image frame).
  - Part (a), power: one *freeze-unpowered* slot; power is cut for 250 ticks just after an advance.
  - Part (b), erasure: the same building, powered, is hit by an erasing weapon whose firer is
    destroyed 150 ticks after it locks on, before the building is erased. A second run cuts the
    power before the weapon locks on and restores it only 300 ticks after the firer is destroyed.
  - Part (c), late slot: a slot that starts when power comes back (*light-unpowered*), with power
    cut before the erasing weapon locks on and restored while the building is being erased.
    Test both an otherwise opaque type without the double-thick flag and one with it; use a
    translucency detail field of 0. The blend check is a departure test against this spec on
    our engine, not an oracle measurement; the held-stage check remains an oracle measurement.
  - Part (d), loop delay while held: a *freeze-unpowered* slot of a test type with loop count −1,
    random loop delay 200, 200, a start sound and rate 300; power is cut within the 200 ticks after
    a pass ends and restored 400 ticks later.
  - Observable: the slot animations' image frames per tick, their blend (screen capture), start
    sounds, and the ticks at which the erasing weapon fires (recorder).
  - Runs: one per part, except (c), one per flag case on each engine.
  - Expected: (a) no advance while unpowered; one advance on the first tick after power returns.
    (b) no advance during the erasure; one advance on the first tick after the firer is destroyed
    and the translucency gone from then on; in the second run no advance until the power returns.
    (c) on our engine the slot has erasure translucency from its first drawn frame: 50 %
    without the double-thick flag, 75 % with it, without waiting for a further shot. On both
    engines it starts held on its first image frame and does not advance until the firer is
    destroyed. (d) the start sound plays 200 updates after the pass end although the building is
    unpowered; the image frame stays on the loop start until power returns.
  - Falsified if: (a) or (b) waits a further 100 ticks after release, an image frame changes while
    held, the second run of (b) advances before power returns, the late slot of (c) on our
    engine lacks its expected erasure blend at any drawn frame during erasure (including before
    a further shot), the late slot advances during erasure on either engine, or (d) plays no
    start sound while unpowered.
- **AN17 (claim 24; deferred, not a skirmish measurement).** A future mode that genuinely
  enables fog would compare two buildings' slot animations, one with the fog-remove flag absent
  and one with it disabled, after seeing and then fogging both. The expected observation is a
  remembered, non-animating image for the first and live animation for the second; the opposite
  result would falsify claim 24. No run count or executable fixture is proposed until that
  mode is in scope. Data overrides cannot enable this test in a skirmish (S-0409 claim 19).
- **AN18 (claim 25).** The test expansion archive also holds two stock sprites copied under new
  names: one with *N*<sub>1</sub> image frames under an image name *I* to which S-0007's letter
  substitution applies, and one with *N*<sub>2</sub> ≠ *N*<sub>1</sub> image frames under the name
  that letter substitution gives *I* in the test map's theater (S-0007 claim 14, scenario A8). Four
  test types name *I*, each with rate 300, one pass and no theater flag: Y1 with the new-theater
  flag and no end field, Y2 with the new-theater flag and end −1, Y3 with the new-theater flag and
  end 0, Y4 without the new-theater flag and end 0. Each is played by a trigger action.
  - Observable: the image frames shown and the number of advances before removal.
  - Runs: one per type.
  - Expected: Y1, Y2 and Y3 draw the *N*<sub>2</sub> sprite. Y1 ends after *N*<sub>1</sub>
    advances, Y2 and Y3 after *N*<sub>2</sub>. Y4 draws the *N*<sub>1</sub> sprite's image frame
    0 for 3 ticks and ends on its first advance.
  - Falsified if: Y1 ends after *N*<sub>2</sub> advances, Y2 after *N*<sub>1</sub>, Y3 on its
    first advance, or Y4 plays more than one image frame.
- **AN19 (claim 14, height condition).** The test copy of the art database gives the explosion
  animation of one stock weapon the scorch flag (and no crater flag). A unit fires the weapon (a) at
  a unit on open flat ground, (b) at a unit on raised ground at least one height level up, (c) at a
  unit standing on a high bridge's deck, and (d) at an aircraft in flight over open ground. If that
  weapon cannot hit aircraft, (d) uses a stock anti-air weapon whose explosion animation the test
  copy gives the same scorch flag. Weapon impacts are used, not a trigger action, so that the
  animation is created at the target's height.
  - Observable: whether a scorch mark appears in the impact cell, read per tick by the recorder from
    the cell's state (a mark under a deck may be hidden on screen).
  - Runs: ten shots per case.
  - Expected: runs (a) and (b) leave scorch marks; runs (c) and (d) leave none.
  - Falsified if: (b) makes none (the test would then use the world height), (c) makes one (it would
    then use the height above the surface), or (d) makes one (no height test). If the smudge specs
    show that a scorch mark is never placed in a bridge cell, (c) cannot tell the two heights apart
    and the choice stays at medium confidence.
- **AN20 (claim 17, factor 5).** A stock tree is set on fire with a weapon that ignites trees (found
  in a pilot run). A burning tree carries a fire animation attached to it, taken from a game-wide
  list of tree-fire animations. The test copy of the art database gives every animation of that list
  a damage of 0.5 and a rate of 300. A control run with the unchanged fire animations records the
  drops the tree takes without the test damage.
  - Observable: the tree's health per tick (recorder) and the ticks of the fire animation's image
    frame changes.
  - Runs: three trees, one control.
  - Expected: beyond the control's drops, the tree's health drops on every advance of the fire
    animation, each on the tick of an image frame change (F5 with *c*δ = 2.5: deliveries 3, 3, 2, 3,
    2, 3, 2, …, all exact in binary).
  - Falsified if: some advance brings no extra drop, in particular the 3rd and 5th (unattached, F5
    with δ = 0.5 delivers 1, 1, 0, 1, 0, 1, 0, …), or an extra drop falls on a tick without an
    advance. A pilot run checks that a 2-point delivery of the game-wide warhead damages the tree.
    If it does not, the scenario is inconclusive.
- **AN21 (claim 13, chain reaction).** The test copy of the art database gives the explosion
  animation of one stock weapon the chain-reaction flag (no crater or scorch flag). A unit fires
  the weapon at a cell holding resource; a second run plays the same animation by a trigger action
  on another cell holding resource; a control run fires the weapon with its unchanged explosion
  animation. This tests the creation start event, not the later events suppressed in a
  trigger-played animation.
  - Observable: the amount of resource in the target cell per tick, read by the recorder.
  - Runs: one each.
  - Expected: in both test runs the cell holds no resource from the tick the animation is created
    (its creation start event, claim 13). The control leaves the cell's resource unchanged.
  - Falsified if: resource remains in either test cell immediately after animation creation, or
    the control removes it. A contrary trigger-played result calls for investigation of claim 13:
    creation-time suppression is not an established alternative behaviour.
- **AN22 (claim 20, cloaked buildings).** A skirmish on a test map whose override section makes a
  stock building type that has slot animations able to cloak. The test copy of the art database
  gives its slot animations three test types: K1 with neither the translucent flag nor a
  translucency field, K2 with the translucent flag, K3 like K1 but with a translucency detail field
  above the detail level, played at detail level 1 (so K3's translucency detail field is 2, the
  others' 0). Three copies of the building stand on the map: one owned by the measuring player, one
  by a computer house allied with the measuring player in both directions, one by an enemy computer
  house; each stands where the measuring player sees it, and no sensors of the measuring player
  cover it. A pilot run confirms that the building cloaks (it is drawn see-through to its owner once
  cloaked).
  - Observable: per drawn frame, whether each slot animation's pixels appear and their blend
    against the ground (screen capture), once each building is fully cloaked; in the enemy view also
    during the cloaking steps, together with whether the building itself is drawn.
  - Runs: one.
  - Expected: own and allied building: K1 at 50 %, K2 not drawn, K3 drawn opaque. Enemy building:
    K1 and K2 not drawn, K3 drawn opaque. During the last steps before full cloak, the enemy view
    may show K1 at 50 % and K2 with its fade code while the building itself is no longer drawn
    (recorded for the cloaking specs; it does not falsify this spec).
  - Falsified if: any K1 or K2 of the enemy building is drawn, the own or allied building's K1 is
    not drawn at 50 % or its K2 is drawn, or K3 is hidden. If the building does not cloak in the
    pilot run, the scenario is inconclusive. Campaign visibility is outside this spec.

- **AN23 (claims 14 and 25; Q-0046 map-load events).** Common measurement setup, skirmish at speed
  4, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps
  excepted). A tiny test map names one test animation on a tile (S-0010 claim 19), and a separate
  run gives an overlay a cell animation (S-0009 claim 12). Record each creation separately,
  including creations removed later during loading, rather than counting only animations left at the
  first drawn frame. Use the read-only recorder for animation positions, event times, cell resource
  levels and scenario-stream state. Take one valid run per deterministic case and 32 independently
  started games for the choice cases, recording their actual seeds and incoming stream states rather
  than setting either. Void a run if an unrelated effect acts on the observed cells before recording
  finishes.
  - Part A, height: use a sprite with one image frame, so μ = 0, no particles or chain reaction,
    and scorch only, then crater only. Put the animation position on clear, smudge-placeable
    ground and repeat with world heights giving Z − H of −1, 0, 29 and 30 leptons. The tile
    displacement or terrain in the displaced cell supplies these differences where available,
    record the actual difference rather than assuming the creator cell's height. For a crater
    case also put resource in the displaced cell and record both overlay presence and the
    stage of the resource cell immediately before and after the event. Use separate cells at
    stages 0, 5, 6 and 7 (purpose-made test values), with growth and unrelated removal excluded
    during observation. Include a separate case whose crater placement is blocked but whose
    cell has resource.
    If the map creators cannot reach a listed difference, use a weapon-impact animation at that
    height to test the boundary, and record that it is not a map-load observation.
    Expected: events occur at creation during loading when μ = 0. Differences below 30 permit
    the requested ground effect, 30 forbids it. An eligible crater request takes 6 bales under
    S-0401 claim 3, even in the blocked-placement case: starting stages 0 and 5 lose their
    resource overlay, stage 6 keeps it at stage 0, and stage 7 keeps it at stage 1. At height
    difference 30 the resource is unchanged. A different overlay/stage result, a ground effect
    at 30, no removal in the blocked case, or an event deferred to the first update falsifies
    the corresponding rule. An inability to reach a boundary is inconclusive, not confirmation.
  - Part B, draws: repeat with neither flag, scorch only, crater only and both, with particles
    absent. Read stream state immediately before and after the scorch-or-crater choice, separately
    from smudge placement. Expected: only both flags below the height boundary take the choice
    ranged draw. Apply S-0202's range rule to the recorded incoming state: the predicted result
    below 1073741824 requests scorch, otherwise crater. Repeat with a positive particle count,
    created by a play-time creator (a weapon-impact animation, or a trigger-played animation) and
    not by a map-placed tile or cell animation. At map load the original stops at the particle
    request (claim 14), so that repeat is not a map-load observation. Record the state after
    particle creation and before the choice: that later state must predict the choice. A choice draw
    in another flag/height case, a different request from the predicted result, or particle creation
    after that choice falsifies the rule. Do not infer the full event's draw count from the choice
    alone. S-0202 scenario R14 A to E supplies the complementary full-composition tests during
    loading and play: per-particle draw groups, smudge candidate selection without placement draws,
    and conditional tile-animation and neighbour spread-offer draws caused by resource removal
    before smudge selection. Its D includes trigger-created animations and later playback events; E
    tests the shared height gate.
  - Part C, sprite: use named copies of stock sprites with different image-frame counts, one
    with a single image frame and the other with at least four. Use the lookup fixtures of AN18
    and S-0007 A8 to make the last successful type read find the first sprite and the theater
    refresh find the second. Test a theater-flag type and a new-theater type without the theater
    flag, both scorch only with no chain reaction or particles. Confirm the two resolved sprites
    with the recorder. Expected: the theater-flag type has μ = 0 and attempts a scorch mark at
    map-load creation despite drawing the multi-image sprite. The new-theater type has μ > 0
    and attempts none until playback reaches that image frame. An immediate middle event for
    the new-theater type, or none for the theater-flag type on eligible ground, falsifies the
    sprite rule. A fixture that does not produce the intended two successful sprite finds is
    inconclusive.

- **AN24 (claims 10, 26 and 27; Q-0418 placement).** Common setup at speed 4, 800×600, maximum
  detail, no engine add-on and no mods (the named test files of a scenario and the purpose-made test
  maps excepted). Use a long-lived test animation with labelled canvas points and a distinct shadow
  rectangle overlapping part of the body. Play it by trigger at a recorded position on flat ground
  and on raised ground; also use it as a building slot with a known slot displacement. Record world
  position and view origin with the read-only recorder and capture pixels in a full-view redraw. Use
  one valid run per case. On raised ground, account for the original height-lift discrepancy
  documented by S-0101 claim 3 and its Formulas F2 when interpreting oracle captures; engine
  acceptance uses that spec's fixed lift. That known discrepancy is not a failure of the additional
  image or shadow placement asserted here.
  - Ordinary and flat variants: compare shifts 0, +7 and −7 pixels, with the same
    playback frame and view. Expected: the image follows claim 26's projection and
    moves by exactly the chosen shift, including the slot displacement where present.
    A different image point or a changed world position falsifies claim 26.
  - Shadow variant, neither flat nor tiled: repeat the shifts, first with an opaque
    image, then with translucency code 2. Expected: the shadow stays on the unshifted
    projected point in all three runs. At an overlap pixel where depth permits it,
    each channel of the painted body or blended body pixel is halved and rounded
    down by the shadow. A shadow moving with the image, or an undarkened overlapping
    body pixel where the shadow passes depth, falsifies claim 10. Flat and tiled
    shadow variants must draw no shadow image frame.
  - Tiled variant: give frame 0 a stored rectangle height of 10 pixels in a canvas
    of 32 rows, and the displayed frame a rectangle of 14 rows. Repeat with frame 0
    height 11, keeping canvas and displayed frame unchanged. Record every visible
    copy's labelled canvas point, then scroll so that the unshifted lowest centre
    crosses row 0 and a multiple of *h*, forcing a full redraw after each scroll.
    Repeat with shifts 0 and +7. Expected: claim 27's centres, spacing and count,
    using heights 10 and 11, with shifts changing only centres. Any spacing based
    on 32 or 14 rows, an omitted lowest half-height lift, or a count inconsistent
    with the formula falsifies claim 27. Use translucent overlapping copies as a
    separate order test: the upper copy blends over the lower, not the reverse.
    Recover clipped copy positions from the sprite's known labelled rows; a copy
    with no visible pixels cannot be counted from the capture alone.

- **AN25 (claim 28; Q-0770).** Use the common setup at 800 × 600, speed 4, with a
  read-only random-state recorder. Named test art supplies two ordinary trigger-played
  types: a forward type with two passes, explicit positive end and loop limits, stage
  delay 1 and a random loop delay pair (2, 5); and a one-pass type with a follow-up
  whose descending random rate pair converts to delays [2, 4]. Give both explicit
  playback limits, no damage, particles, smudges, resource reaction, trailers or special
  motion. A second pass uses loop endpoints (3, 3), then (0, 0), and equal converted
  follow-up-rate endpoints. Create one type at a time and record creation tick, stage,
  current type, remaining passes, delay, lifetime and stream state each tick through
  removal. Repeat 32 times per case. Compare predicted allocations from each run's own
  incoming stream state, not an assumed identical initial seed.
  - Visibility variants: visible; outside the view; in an unexplored cell not seen by
    any house; above the player's detail level; and a named test image that resolves
    to no sprite at any lookup. Keep explicit positive limits for the spriteless run.
    Also run a spriteless type with absent limits, one pass, delay 1 and a named
    follow-up with positive limits. Do not use −1 limits in the missing-image fixture.
  - Expected: the unequal loop pair takes exactly one ranged draw at each nonterminal
    pass end, and the follow-up takes exactly one at the type switch, with raw values
    predicted by S-0202 claim 4. The equal pairs and zero loop pair consume no raw
    value for their choices. Visibility variants have the same clock and conditional
    draw rules. The spriteless positive-limit type plays every pass; the absent-limit
    type switches at its first eligible advance, not at creation. No ordinary clock
    transition consumes an additional raw value.
  - Ordering variant: create two long-lived ordinary animations in a known order so
    that both reach a drawing pass boundary in the same tick, neither being removed.
    Their selected delays must receive the ranged results in update order, not image
    depth order. In a further variant let the earlier animation's ending advance
    produce a middle-event particle request and let its follow-up have μ = 0 and
    particles. Predict old middle-event draws, follow-up-delay draw, then new immediate
    middle-event draws using claim 14 and S-0202 claim 15. Record particle attributes
    as well as stream state to distinguish a swap with the same total consumption.
  - Falsified by a missing draw solely in an invisible variant, creation-time deletion
    of the spriteless positive-limit type, any extra ordinary clock draw, a type switch
    at the wrong advance, or any outcome assigned a value outside the stated order.

- **AN26 (claims 5, 16 and 29; Q-0771).** Common tiny skirmish setup, speed 4, 800 × 600. Named test
  art gives each tested shot, warhead, death, projectile-trailer and wake animation a
  distinguishable stock sprite, explicit end 3, one ordinary forward pass, delay 1, no random rate,
  no normalisation, no follow-up and no event or damage effects. Named map test data selects these
  types through their creators' import mappings. No engine add-on and no mods (the named test files
  of a scenario and the purpose-made test maps excepted). Use the read-only recorder for creation
  tick, update order, actual first update, stage and removal tick, plus end-of-tick captures for
  visible types. Run each fixture three times.
  - During-pass fixtures: one firer launches a long-flight projectile, testing its
    shot animation; the still-flying projectile creates one trailer; a surface ship
    moves on open water and creates a wake. Keep them alive, with no other removal
    in those passes. Expected for creation tick *c*: first update in *c*, stage 0 at
    the end of *c*, stage 1 at the end of *c* + 1, removal in *c* + 3. For a trailer,
    retain its creator-supplied initial loop delay, recording that delay explicitly:
    its first update still excludes countdown, the start event is in *c* + 1, the
    first advance in *c* + 2, and removal in *c* + 4. These are claims 5 and 11's
    results for its initial delay of 1 update, not the no-loop-delay example above.
  - Detonation/removal fixture: one projectile is last in the update order before
    its turn, has no shot animation or trailer, detonates nonlethally on open ground,
    creates exactly one explosion animation and is removed, with no other creation or
    removal. Expected: its explosion joins during the pass but loses that tick's turn.
    First actual update is *c* + 1, stage 1 first appears in *c* + 2 and removal is
    *c* + 4. Confirm the recorded order and creation count; reject the fixture, not
    the claim, if other objects or effects invalidate those preconditions.
  - Death fixture: let a killing projectile produce one victim death animation.
    Record all counting removals and objects still following the current object at
    the end of the turn. The death animation is created during that projectile turn.
    Whether it first updates in *c* follows S-0203 claim 10, not a special death rule.
    Repeat with a pre-pass storm kill and an after-pass surrender destruction that
    actually creates the named death animation. Expected: creation before the pass
    for the storm and after it for surrender; absent intervening skip/removal, first
    update respectively in *c* and *c* + 1. First advances and lifetime follow claim
    5 and claim 16's formulas. Exclude sinking, crashing, deferred deaths and infantry
    death sequences unless separately recording their actual later creation event.
  - Falsified by an ordinary creator running in a different phase, a no-removal
    during-pass fixture withholding its first update until next tick, the minimal
    projectile fixture updating its sole new explosion in *c*, or a first-stage hold
    or lifetime inconsistent with the recorded first update and initial delay.
    Turning off the removal-skip quirk is an engine-only companion test: the minimal
    explosion should then first update in *c*, advance in *c* + 1 and end in *c* + 3.

- **AN27 (claim 17; Q-1425: animation damage and resources).** Use the common tiny
  skirmish setup at speed 2, with mods off except the named test data and no helper that
  changes rules, houses or computer players during the game. A named
  art fixture gives an ordinary weapon-created explosion animation damage 20, rate 300,
  start 0, end 3 and one forward pass, with no follow-up, particles, craters, scorch marks
  or start-event resource reaction. The weapon's own impact must not reduce resources.
  A named map fixture supplies a resource patch whose overlay has its chain-reaction flag
  and selects a game-wide flame warhead with its resource flag enabled. Disable resource
  growth, spreading and secondary explosions for the observation. Use a patch with more
  than 2 resource units per observed cell, and keep harvesters out.
  - Procedure: create the animation with an ordinary shot, not a trigger-played animation.
    Snapshot resource amounts after that impact and before the first animation advance.
    Record stage, position and each scanned cell's resource amount on every damaging
    advance, using the read-only recorder and frame capture. Repeat with the selected
    warhead's resource flag off, then with the overlay's chain-reaction flag off.
  - Walker variant: use S-0615 scenario FW3's expiry fixture on the same resource patch,
    with named art damage 20 and at least one expiring advance before removal. Record
    running versus expiring stages and the resource cells scanned at each actual position.
    Keep the initial infantry-killing impact outside the resource observation interval.
    Repeat the same two flag-off controls. Run each ordinary and walker case three times.
  - Expected: with both flags on, each permitted scanned resource cell loses 2 resource
    units on each damaging advance while sufficient resource remains, under S-0303
    claim 17. F5 gives first delivery 21 damage points and later deliveries 20, both
    truncated toward zero after division by 10 for resource reduction. There is no loss
    during a walker's running stages or between advances. Either flag-off control loses
    nothing through this damage. No crater or start-event removal contributes.
  - Falsified by no reduction with both gates satisfied, reduction in either flag-off
    control, reduction during running, or loss inconsistent with the actual per-advance
    delivered amount and S-0303 claim 17's cell scan.
  - Attribution companion: S-0302 scenario D33 supplies owned and ownerless damaging
    animations, defended and lethal targets, kill/loss records and crew-escape controls.
    Repeat its owned lethal case with FW3's expiring walker. Expect no house credited
    with an animation-damage kill, even when the animation has an owner; the victim's
    house still records the loss. Owner-derived kill credit falsifies the absent-house
    contract. Neither a running walker nor a trigger-played animation is a damaging
    control for this purpose.

- **AN28 (claims 14, 15 and 17; Q-1716: damage before the middle event).** Use AN27's common setup,
  resource fixture and flag settings at speed 2, with mods off except the named test data and no
  helper that changes rules, houses or computer players during the game: the observed cell's
  resource overlay has its chain-reaction flag, the game-wide warhead that an ordinary type's damage
  uses (claim 17, Import mapping) has its resource flag, resource growth, spreading and secondary
  explosions are off, harvesters are kept out, and the creating weapon's own impact neither reduces
  resources nor leaves a smudge. A named art fixture gives an ordinary weapon-created animation
  damage 9.0, rate 300, start 1, a sprite of 4 image frames (so μ = 2, reached by the first
  advance), the crater flag but not the scorch flag (so the middle event requests a crater and, by
  claim 14's table, takes no choice draw), no particles, no start-event resource reaction and one
  pass. By F5 the first advance delivers 10 damage points, so the detonation takes trunc(10 / 10) =
  1 bale from each permitted scanned resource cell (S-0303 claim 17). The animation stands on open
  flat ground at terrain height, so the height test passes, in a cell *C* holding resource at stage
  6; every other cell within three cells of *C* is flat, holds no overlay and no smudge, and has a
  morphable tile, so that a crater's fit at *C* depends only on whether *C* still holds its overlay
  (S-0312 claim 26).
  - Why the order shows: under damage first, the detonation leaves *C* at stage 5, the crater
    request's 6 bales then empty it (S-0401 claim 3), and the crater fits and appears. Under
    the middle event first, the crater request leaves *C* at stage 0 with its overlay, the crater
    does not fit, and the detonation then empties *C*. Both orders end with *C* empty; only the
    crater differs.
  - Variants: (a) end field 3, so the advance reaching μ is not the last and image frame 2 is
    drawn; (b) end field 1, so the advance reaching μ ends the animation; (c) as (b), with a
    follow-up type that has neither the crater nor the scorch flag, no damage and a random rate
    pair, so the follow-up's own first advances make no smudge.
  - Controls, run first as a pilot of the fixture: (d) as (a) with no damage field, *C* at stage
    6: expected no crater and *C* left at stage 0; (e) as (d) with *C* at stage 5: expected a
    crater and *C* empty. A fixture in which (d) shows a crater or (e) shows none is not judged.
  - Procedure: snapshot *C*'s stage after the impact and before the first advance, and record
    with the read-only recorder and frame capture *C*'s stage and any smudge at *C* in the tick
    of the advance that reaches μ and the tick after. Three runs of each variant.
  - Expected: in (a), (b) and (c) a crater appears at *C* in the tick of the advance that reaches
    μ, and *C* is empty from that tick. In (b) and (c) image frame 2 (at μ) is never drawn, and in
    (c) the follow-up type's first image frame is drawn from that tick.
  - Falsified by no crater at *C* in any of (a) to (c) while the controls behave as expected
    (the middle event before the detonation), by a crater with *C* not empty, or, in (b) and (c),
    by no crater at all (no middle event on the ending advance, or a follow-up switch before
    it).

- **AN29 (claim 17, Import mapping; Q-1717: the type-name warhead choice).** Use the common setup
  at speed 2, with mods off except the named test data and no helper that changes rules, houses or
  computer players during the game. Test rules give the two game-wide warheads of Import mapping
  different verses against the armour of a stock building whose type is not undemolishable (S-0302
  claim 7): 100 % for the invisibility-animation type's warhead and 0 % for the other, both with no
  area beyond the building's cell. A named art fixture gives the built-in invisibility-animation
  type (Import mapping) and an ordinary test type the same damage value 1.0, rate 300 and one pass
  of 8 image frames, with no follow-up, particles, craters or scorch marks. Each is created by its
  own test weapon on its own building of a third house, and the creating weapons' warhead has 0 %
  verses against that building's armour, so the impact itself does no damage. Snapshot each
  building's health after the impact and before the first advance, and judge only the losses after
  it. Variant: an ordinary test type with damage 1.0 and one 4-frame pass whose follow-up type is
  the invisibility-animation type.
  - Procedure: record each building's health after every advance with the read-only recorder.
    Three runs each of the main run, the variant and part b.
  - Expected: the building under the invisibility-animation type loses health on every
    advance by the F5 deliveries under 100 % verses (S-0302); the building under the ordinary
    type loses none. In the variant, no loss during the ordinary type's advances and a loss
    from the follow-up's first delivery on, with the accumulator reset of claim 15.
  - Falsified by the ordinary type's building losing health, the invisibility-animation type's
    building losing none, or a loss during the variant's first type.
  - Part b (letter case and no type of the exact name): repeat the main run with a test copy of the
    rules in which the animation list's entry for the built-in invisibility-animation type is
    spelled with only its first letter capital, and a test copy of the art in which that type's
    section heading is spelled the same way. Pilot check: the installed rules file's animation list
    spells that entry in capitals, a search of the whole installed rules file finds no other value
    or section heading that names the type (key names do not count), and every other mention of the
    name in the test rules and the map used is spelled the same new way, so the type is first
    registered in the new spelling. Expected: the building under that type loses no health, like the
    ordinary type's. A loss falsifies the exact, case-sensitive comparison and the no-type rule.

- **Companion section-selection scenario (Import mapping).** S-0007 scenario A25 supplies
  the tiny skirmish setup, conflicting own/image-section values, omitted and empty own keys,
  damaged and normal building slots, independent effects, preceding-sprite/read-count variants,
  per-field observables, controls and falsifiers. Use the relevant playback or event scenario
  here as its consumer fixture. Expected: every non-theater property matches its own-section
  control at every read, never the image-section control; claim 25 still determines
  sprite-derived limits. A contrary section choice falsifies the Import mapping rule.

- **Deploy presentation companion scenario (claims 5 and 29).** S-0517 scenario P1 C
  supplies the grounded vehicle, named presentation data, game settings, recorder, repeated
  deploy/undeploy pairs and falsifiers. Expected: creation during the vehicle's turn with
  no initial loop delay; with same-tick first update, positive playback delay *d* and no
  lost updates, the first image advance is *d* ticks after creation, in the requested
  direction subject to claim 8. Random rate or normalisation changes playback, not the
  separate base-delay completion clock of S-0517 claim 7. A creation-tick advance or
  completion timing that instead tracks the selected playback delay falsifies that contract.

- **Companion scenarios (claims 5 and 13 to 17).** S-0611 scenario MU10 tests generic
  infantry-making animations, terminal-only creation and position-choice draws; its MU9
  tests refused placement and retries. Use those scenarios for claim 16's delegated contract.
  S-0615 scenarios FW1 to FW4 test running-stage freeze, retained creation effects,
  expiry damage and early removal. Use those scenarios for the burning-walker exceptions:
  they retain their owning specs' setup, expected results and falsifiers.

## Changes for other specs

None. These statements in other specs already agree with this spec:
- S-0011 claim 36 points to claims 13, 14 and 25 here for the creation-time and later middle
  events, and to S-0202 claims 10 and 15 for the creation draws and the middle-event draw
  composition during map loading and during play; its table *Load-time draw order* places the
  creation draws.
- S-0202 claims 10 and 15 and scenario R14 agree with claims 13, 14 and 25.
- S-0103 claim 8 gives the palette exception for tiled copies (claim 27).

## Open questions

- Q-1717 is answered by claim 17 and Import mapping: the name is the one given there, compared
  exactly with the current type's name on each delivery; with no type of that name the other
  warhead is always used. AN29 tests it, its part b the letter case and the no-type case.
  Decided by the project lead (2026-10-24): Import mapping may give the literal name of the
  built-in invisibility-animation type (Q-1717 question 1), as a name the importer must recognise
  in the user's data. It appears there only; the rest of this spec names the type by its role.
- Q-1716 is answered by State and ordering, claims 14 and 17 and AN28: an advance's damage
  detonation comes before its middle event, and both come before the end test, a new pass or a
  follow-up switch, on ending advances too. AN28 observes the order through a resource cell that
  the detonation's bale decides a crater on; the order of the draws within the tick is left
  `unmeasured` (claim 14).
- An attached animation whose own damage detonation removes its owner, such as a damaging fire on a
  building it destroys, which leaves play at once (S-0203 claim 6 (b)). Unlike an animation that
  leaves play in its own detonation (State and ordering), it goes on with the rest of that update
  after the detonation, since the owner's removal only marks the animation for deletion at its next
  update (claim 22); `unmeasured`, confidence medium, since no scenario here has an animation's own
  damage destroy its owner on an advance that reaches μ. Not settled: which position the rest of
  that update's effects use (a middle event's height test, its particles and its scorch mark or
  crater). Our engine uses the owner's last position as the owner's position of claim 22 (design).
  Backing `unmeasured`, confidence low, for the same reason.
- Q-1425 is answered by claim 17 and AN27. The existing approved S-0303 claim 17 already
  permits resource reduction for damaging-animation detonations, and S-0302 claim 1
  already specifies no attacker or attacking house, even for an owned animation.
  S-0615 claim 11 already delegates expiring-walker damage to claim 17 here. This spec
  makes the composition explicit; it changes neither those gates nor walker expiry timing.
- Q-0418 questions 1 to 4 are answered by claims 10, 26 and 27. S-0108 claim 10
  already supplies the building-slot world position, and S-0102 claim 19 already
  supplies canvas centring. Question 5's exact packed-pixel blending is answered by
  S-0102 claim 32 and Formulas F8; claims 20 and 21 here choose the code only, and
  S-0103 claim 9 supplies the pixel layouts. S-0102 claim 13 already specifies
  corrected tiled depth and records the original drawing-only defect without a
  quirk switch.
- Q-0046 is answered across specs. Claim 25 answers Q2c, including animations created during
  map loading. Claim 14 answers Q2b's event gates, height condition and scorch-or-crater choice.
  S-0202 claim 15 and R14 complete the downstream particle creation draws, smudge candidate
  preferences, selection and draw-free placement during loading and play, including
  trigger-created animations and later playback middle events. They also cover the map-data
  relation and conditional resource-removal recalculation and neighbour spread-offer composition.
  Event timing, sprite-derived μ and the height gate remain claims 13, 14 and 25's rules here.
  S-0401 claim 20 answers Q2a's chain-reaction
  draw sequence, including its no-resource gate and ordering after the whole immediate middle
  event. Subsequent particle updates are not specified here and must not be treated as draw-free.
- Q-0770 is answered by claim 28, supplementing the existing claims 3, 11, 14, 19 and 25
  and the missing-sprite rule in Edge cases. S-0214 claim 14 retains its separate, draw-free
  move-target marker. Q-0771 is answered by claim 29 and claim 5's clock formula:
  existing S-0203 claims 1, 4, 5 and 8, S-0306 claim 16 and S-0504 claim 2 already
  establish several creator phases. Claim 29 adds the explicit creator table and removal
  qualification. At hand-off, update S-0203 claims 2 and 10's obsolete uncertainty about
  animation placement to cite claim 29 and AN26. Creation phase alone must not be read as
  a guarantee of a creation-tick update.
- Claim 29 identifies a projectile trailer's initial delay, repeat factor and reverse
  request. A complete inventory of other creators giving a repeat factor *m* > 1,
  reverse play or an initial loop delay remains with their owning specs.
- The terrain specs can end the looping of the animations attached to a terrain object early, by
  making their current pass the last (the fire on a burning tree). When that happens is theirs to
  say and to measure.
- A creator can also ask for translucent drawing of its own, which would combine with the codes of
  claims 20 and 21 as in Terms. No creator is known to ask for one; the creators' specs should
  confirm it.
- A "random start" of animations has been suggested. No random starting image frame exists (claim
  12); if a measurement shows one, this spec needs a revision.
- When an erasing weapon locks onto a building and lets go, and how several erasing weapons on one
  building interact, belong to the combat specs; this spec uses only the resulting *being erased*
  state (claims 21 and 23).
