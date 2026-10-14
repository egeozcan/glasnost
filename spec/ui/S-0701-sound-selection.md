---
id: S-0701
title: Sound selection
topic: ui
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0261, Q-0262, Q-0266]
supersedes: []
---

# S-0701: Sound selection

## Summary

Sound data, selection and order responses, event sounds, announcements, mixing limits,
placement and traced random choices in skirmish; network games and campaigns are out of scope.
Playback is presentation, but claim 33's traced scenario-stream choices must retain their draws.
Music belongs to planned S-0703; archive mounting to S-0005 claim 10. Which files hold the samples
is claim 34; the byte formats of those files are outside this spec. Revision 4 confirms repeated
route-refusal cues, literal announcement bindings and distinct production-refusal feedback,
and cites S-0005 claim 10a for independent bank-file precedence. Revision 3 supplied literal
import bindings and qualified full-level screen requests. Revision 2 already answered Q-0260
in claims 1 to 15 except complete delayed playback draws; Q-0261's queue, voice-set and sample
rules in claims 26, 27 and 34; Q-0264's normal report, common death and building fallback rules
in claims 20 to 22 and 37; and Q-0265's selection and ordinary order rules in claims 16 to 18
and F6. This revision completes Q-0261's event-name bindings. Delayed draw totals, infantry
death multiplicity and specialised deploy exceptions remain open.

## Terms

Glossary terms are used as defined there (*tick*, *drawn frame*, *battlefield view*, *local
player*, *house*, *cell*, *lepton*, *unsynchronised random stream*, *scenario random stream*,
*explored*). From S-0202: *raw value* and the ranged draw R(*a*, *b*) of claim 4 (no raw value is
taken when *a* = *b*). From S-0302: the *damage-state result* of a hit (*unaffected*, *unchanged*,
*now yellow*, *now red*, *now dead*). From S-0011: *radar event*. From S-0107: the game-speed
normalisation N of its Formulas F3. In addition:

- **Sound entry.** A named entry of the sound data: a list of samples and the settings of claim 2.
  Every in-game sound other than music and announcements is the playing of a sound entry.
- **Sample.** One recorded clip in the game's audio archive, named by its file name.
- **Sound instance.** One playing of a sound entry, from its request until it ends or is cut.
- **Sound level** (*level* for short in this spec; not the glossary's *height level*). Loudness on
  a scale from 0 (silent) to 16384 (full). Where a level is a fraction in 0 … 1 it says so.
- **Pan.** Left-right position on a scale from 0 (full left) to 16384 (full right); 8192 is centre.
- **Channel.** One of the 16 mixing voices that sound instances play on (claim 9).
- **Audio clock.** Real time in milliseconds. Every time in claims 9 to 14, 27 and 29 is on the audio
  clock, not in ticks, so it does not depend on game speed.
- **Audio pass.** One service of the sound instances. Successive passes are separated by more
  than 33 ms of the audio clock, whatever the tick rate (claim 12).
- **Screen sound.** A request that is not tied to a map position: centre pan and full request
  level by default (claim 5); claim 35 specifies explicit volume-fraction exceptions.
- **Positional sound.** A request at a map position: its level and pan follow from where the
  position appears in the battlefield view (claim 6, Formulas F2).
- **Attached sound.** A positional sound whose placement is updated while it exists (claim 7).
  Some follow an object or animation; others keep their original map position but follow scrolling.
- **Repair-response vehicle.** The passenger-configurable vehicle whose weapon can become a
  repair weapon. Only this particular vehicle type receives F6's dedicated repair response.
  A negative-damage weapon alone does not qualify another type. Import mapping gives its
  exact type-name binding.
- **Response.** A spoken acknowledgement by one of the local player's objects: on selection, on an
  order, on taking damage, on dying. A response kind uses a list or a single sound entry on the object's
  type, with the fallbacks of F6 (Import mapping). S-0214 calls the selection kind *selection responses*.
- **Response slot.** Per object, at most one response waiting to be spoken (claim 16).
- **Announcement.** A spoken line of the battlefield announcer, from the announcement data
  (claim 26). Announcements do not use channels; they play one at a time on their own (claim 27).
- **Announcement kind.** One of *standard*, *queued*, *interrupt*, *queued interrupt*.
- **Announcement priority.** One of *low*, *normal*, *important*, *critical* (0 to 3).
- **Sound priority.** One of *lowest*, *low*, *normal*, *high*, *critical* (0 to 4), per sound entry.

## Behaviour

### Sound data

1. **Sound entries come from a named list.** The sound data file has a list section whose values
   are the entry names, at most 31 characters each (a longer name is cut to 31). Each name's
   settings are in a section of that name. A name listed twice is one entry: the second listing
   reads its section again over the same entry, appending its sample names again up to the
   32-sample cap rather than clearing the old list. A listed name with no section is an entry with no
   samples, which is silent. Everywhere a rules file or the game names a sound, the name is matched
   against the entries without regard to case. The whole reference is compared: a reference is not
   cut to 31 characters, so one longer than 31 characters never matches an entry whose listed
   name was cut (medium). An unknown name resolves to no entry.
   Whether an existing imported setting is retained is specified in Import mapping.
   - Backing: `measurable` (scenario A1)
   - Confidence: high

2. **What an entry holds and its defaults.** Read per entry, each key with the parse form of the
   Import mapping:
   - *samples*: up to 32 sample names, separated by spaces, tabs or newlines (not commas); every
     leading `$` and `#` character of a name is dropped; names past the 32nd are ignored,
   - *volume*: a percentage *v*. The entry's level is
     given by Formulas F1, including its full-volume quirk alternative, in floating point as
     F1 specifies,
   - *volume shift*: an integer percentage, clamped to 0 … 100 (claim 13),
   - *minimum level*: a percentage, × 0.01 and clamped to 0 … 1 (claim 6),
   - *sound priority*: the whole value matched without regard to case against the five priority
     words; anything else, and an absent key, gives *normal*,
   - *attack count* and *decay count*: integers, default 0 (claim 14),
   - *control*: a set of words (*all*, *loop*, *random*, *pre-delay*, *interrupt*, *attack*,
     *decay*, *ambient*) separated by spaces, tabs or newlines; unknown words are ignored. Without
     the *attack* word the attack count becomes 0; with it and a count of 0 the count becomes 1.
     The same holds for *decay*,
   - *placement words*: *global*, *local* and *shroud* directly affect positional playback
     (claim 6). Of *screen* and *local*, the last word given applies: *screen* cancels
     centre-distance placement and restores view-edge distance, but does not turn a positional
     request into a screen sound. Likewise the last of *unshroud* and *shroud* applies:
     *unshroud* cancels the explored-cell restriction. These words do not change screen
     requests. The remaining recognised words, including *quiet* and *loud*, have no
     established playback effect (medium),
   - *limit*: an integer, the most instances of this entry that play at once; 0 means no limit
     (claim 8),
   - *loop count*: an integer, default 0 (claim 14),
   - *range*: an integer number of cells (claim 6),
   - *delay*: one or two integers *a* [*b*] in milliseconds, *b* = *a* when only one is given,
     default 0 0 (claims 13 and 14),
   - *pitch shift*: one or two integers in percent, in the same form, default 0 0 (claim 13).

   A game-wide defaults section supplies the volume, minimum level, limit, range, control set and
   placement set of every entry that does not give its own. Without that section, or without the
   key in it, the defaults are those of Constants. When an entry gives at least one nonempty control token, even an unknown one,
   its control set is exactly its own words, not added to the default set; the same holds for
   placement words, which start from *screen*. Keys are matched by exact case (S-0001, claim 13), so
   a key spelled with other capitals is ignored (Import mapping lists the stock cases).
   - Backing: `measurable` (A1, A2, B2, C1 and E1) for the tested fields, active words and
     placement-word cancellation. Inactive placement words are `unmeasured` for the reason
     in A2; F1's exact integer at
     full volume is also `unmeasured` for A2's output-resolution reason.
   - Confidence: high (keys, forms, clamps, word lists); medium (which placement words act)

3. **The game-wide default sound priority is ignored.** The defaults section's priority key is
   read but never applied: an entry without its own priority is *normal* whatever the defaults say.
   Quirk `quirks.default_sound_priority_ignored` (Edge cases).
   - Backing: `measurable` (scenario A3)
   - Confidence: high

4. **One missing sample silences the whole entry.** If any sample an entry names is not in the
   audio archive, the entry never plays at all, not even its other samples. Quirk
   `quirks.missing_sample_silences_entry`.
   - Backing: `measurable` (scenario A1, part c)
   - Confidence: high

### Starting a sound

5. **Screen sounds.** A screen sound plays at the entry's level and centre pan, wherever the view
   is, unless its request explicitly supplies the volume fraction of claim 35. Responses to orders and selection (claim 16), interface sounds (claim 24), the base under
   attack sound (claim 28) and building placement (claim 23) are screen sounds.
   - Backing: `measurable` (B1 for responses and interface sounds, F4 for placement,
     G3 for attack warnings, each repeated after scrolling)
   - Confidence: high

6. **Positional sounds: heard only near the view.** A positional sound's level fraction *L* and pan
   are worked out from the position's place on screen against the battlefield view (Formulas F2):
   the distance is measured from the edge of the view rectangle (zero inside it), with the vertical
   distance counted double, and the sound fades linearly to silence at the entry's range of 60
   pixels per cell. With the *local* word the distance is measured from the view centre instead.
   With the *global* word *L* is raised to at least the entry's minimum level. Placement words
   are resolved as in claim 2: a later *screen* cancels *local* and a later *unshroud* cancels
   *shroud*. With the *shroud*
   word the sound is silent when its cell is not explored by the local player, and also when its
   cell is cell (0, 0) (quirk `quirks.shroud_sound_corner_cell`). A sound whose *L* is below 0.05 is
   not started at all and takes none of the draws of claim 13. The audible area therefore depends on
   the view size in pixels: a larger screen hears more of the map.
   - Backing: `measurable` (B2) except the corner-cell exclusion, which is `unmeasured`:
     B2 has no established legal sound placement at that cell; its attempted test is exploratory.
   - Confidence: high (formula); medium (the *local* and *global* cases, no stock entry exercises
     them in a skirmish)

7. **Placement can follow scrolling even for a one-shot.** Direct positional requests, including
   normal weapon reports and death responses, keep their start level and pan. Other requests,
   including the doubled damage sound, retain their map position but are placed again at audio
   passes. Object-attached sounds such as movement and turret sounds are re-placed during the
   object's updates, not only at audio passes. Updated placement below 0.05 stops a playing
   instance. An indefinitely looping entry can restart when placement becomes audible again, but
   a finished or stopped finite one-shot is not restarted by scrolling alone.
   - Backing: `measurable` (scenario B3)
   - Confidence: high (the two request forms and indefinite-loop restart); medium (the complete
     classification of event requests)

### Limits and channels

8. **Per-entry limit: the quietest instance goes.** At each audio pass the instances of one entry
   are ranked loudest first, by their level. Two instances whose integer levels differ by less than
   ⌊the entry's level / 10⌋ count as equally loud (integer arithmetic); between those, a newly requested instance ranks
   above the playing ones if the entry has the *interrupt* word, and below them otherwise. While
   more instances exist than the entry's limit, the lowest-ranked one is cut (stopped at once if it
   is playing). So without *interrupt* a new request at about the same loudness is dropped, and with
   it the oldest one is cut.
   - Backing: `measurable` (scenario C1)
   - Confidence: high

9. **Sixteen channels; priority decides who gets one.** All sound instances (not announcements)
   share 16 channels. A starting instance takes a free channel if there is one. Otherwise compare
   two candidates: the lowest-priority playing channel, and the lowest-priority busy channel that
   is not playing yet, such as one waiting through a pre-delay. Among playing channels of the same
   priority, a later channel replaces the candidate only if its level is lower by at least
   1638/65536 of one level unit. Among non-playing channels, equal priority keeps the first in
   channel order. Choose the playing candidate only if its priority is strictly lower than the
   non-playing candidate's, or the latter does not exist. If the chosen candidate's priority is
   higher than the new instance's, the new instance is discarded; otherwise it is cut and replaced. A discarded instance is not
   retried. The original turns its sound off entirely when it cannot get all 16 channels from the
   sound device; our engine provides 16 channels always.
   - Backing: `measurable` (scenario C2) for priority, quietest-playing replacement and
     playing versus pre-delay precedence. The exact fractional level tie threshold, channel-order
     ties and device-allocation failure are `unmeasured`: C2 cannot identify channel order,
     loopback levels cannot resolve the sub-unit threshold, and no failing-device setup is given.
     The guarantee of 16 channels in our engine is a design requirement, not an oracle claim.
   - Confidence: high (rules); medium (the order among equally loud channels)

10. **Instance pool.** At most 300 sound instances exist at once (playing, waiting or between
    samples) and a separate limit of 200 applies to positional requests whose placement follows scrolling
    at their fixed map position (claim 7), not to all positional sounds. A request when the
    pool is full, after dead instances are cleared, plays nothing.
    - Backing: `unmeasured` (normal play does not reach either number; stated so an implementation
      does not silently cap lower)
    - Confidence: medium

11. **No shared response lock is traced.** Responses of different objects can overlap, subject
    to the limits of claims 8 to 10 and each object's slot of claim 16. This does not establish
    that no specialised event has an additional throttle.
    - Backing: `measurable` (scenario D1, two independently selected objects)
    - Confidence: high (overlap); medium (absence of a general response throttle)

12. **Audio passes are on the audio clock.** A pass is eligible only when more than 33 ms have
    elapsed since the preceding pass. It runs when the main loop next services audio; 33 ms is
    therefore a minimum interval, not a maximum request latency. Channel allocation and per-entry
    ranking wait for a pass. Sample completion and object-attached placement have other update
    opportunities (claim 7). Several ticks may pass between passes at fast game speeds.
    - Backing: `measurable` (scenario E2)
    - Confidence: high (strict interval); medium (observable latency)

### Inside one sound instance

13. **Per-instance draws: pitch, volume shift, pre-delay.** When an instance gets its channel it
    takes, from the unsynchronised random stream and in this order: the pitch *p* = 100 +
    R(*f*₁, *f*₂) percent from the entry's pitch shift; the volume shift *s* = R(0, *vs*), which
    lowers the instance's level by *s* percent (Formulas F3); and, only if the entry has the
    *pre-delay* or *ambient* word, a pre-delay R(*a*, *d*₂) ms where *a* is 33 for *ambient* and
    the delay's first number otherwise. A pre-delay of 33 ms or more is waited before the first
    sample starts; a shorter one is no wait. Following S-0202 claim 4, an entry with no pitch shift
    and no volume shift takes no raw value for them.
    - Backing: `measurable` (scenario E1)
    - Confidence: high

14. **Which samples play and in which order.** Let the entry have *n* samples, attack count *A*
    and decay count *D*; the *body* is the samples after the first *A* and before the last *D*.
    - *Immediate mode* (delay's first number below 33): the instance plays, in this order, one
      attack sample R(0, *A* − 1) if *A* > 0, then the body part, then one decay sample
      R(*n* − *D*, *n* − 1) if *D* > 0. The body part is every body sample in list order with *all*,
      one body sample R(*A*, *n* − *D* − 1) with *random* but not *all*, and the first body sample
      otherwise. With *all* and *random* together the body samples are played in a random order
      without repetition: each next one is R(0, *m* − 1) among the *m* not yet played. With *loop*
      the body part is played again after it ends: forever if the loop count is 0, otherwise for
      that many passes in total; a body sample chosen once by *random* is the one repeated.
    Before audible playback in immediate mode, when there is no attack sample, a *random* body
    ordering takes an extra R(0, *m* − 1) over the loaded body samples and discards its result.
    The audible ordering then starts from the full body list again. This also occurs with one
    loaded body sample, but that equal-ended draw consumes no raw value. Quirk
    `quirks.sound_order_startup_draw`.
    - *Delayed mode* (delay's first number 33 or more): samples are supplied one at a time.
      A *random* body without *all* is selected once and retained across loops; with *all*, body
      indices are consumed without repetition within a pass. The complete attack/decay and wait
      draw counts in this mode are not established here and must not be inferred from the
      immediate-mode table. This is Q-0260's remaining part, explicitly `unmeasured`: the
      completion and reload choices have not been composed across attack, body, decay and the
      final silent wait. Scenario E3 is the settling test; no fixed total is asserted.
    All traced draws use the unsynchronised stream, after claim 13's draws. In particular an
    immediate instance chooses its attack sample, its body sample when applicable, and its decay
    sample before drawing the body's audible order.
    - Backing: `measurable` (scenario E1 for immediate mode and the delayed body choices), and
      `unmeasured` for the complete delayed sequence (the sample-completion path and terminal
      waits have not been traced end to end; E1 does not yet discriminate them)
    - Confidence: high (immediate mode); low (complete delayed-mode sequence)

15. **Loudness and pan of what is heard.** The level heard is the product of the entry's level,
    the request's level (screen: full by default, or claim 35's fraction; positional: Formulas F2), the volume-shift factor of claim 13
    and the player's sound volume option, combined as in Formulas F3. The heard level is turned into
    a whole percentage and that into an attenuation in hundredths of a decibel on a base-2
    logarithmic law (Formulas F4): halving the level costs 10 dB, and 0 % is silence. The pan is
    applied by attenuating the far side on the same law, so a sound at the view's right edge
    (pan 12288) is heard 10 dB quieter on the left. Pitch scales the sample's playback rate.
    - Backing: `measurable` (B2 parts b and c for output level and pan, E1 for pitch).
      The exact intermediate integer truncations of F3 are `unmeasured`: the loopback output
      resolves whole output percentages, not all intermediate level units.
    - Confidence: high (the law and the tables); medium (the order of the roundings in F3)

### Responses to selection and orders

16. **Who speaks, and how.** Only objects owned by the local player give responses (claims 17 to
    19); an AI house's objects never do. A response is put in the object's response slot,
    replacing any response already waiting there. At its next update, in every tick, an object with
    a waiting response speaks it as a screen sound if the object is not already speaking. If it is
    still speaking, a waiting response that is the same sound entry as the one being spoken is
    dropped, and a different one keeps waiting until the current one ends. So an object never
    speaks two response-slot lines at once, repeating the same order while it speaks does not restart its
    line, and a different order's response follows the current one.
    - Backing: `measurable` (scenario D1)
    - Confidence: high

17. **Selection.** When an object becomes selected by the local player's input, it gives a
    selection response: one entry of its type's list chosen by one raw value *x* of the
    unsynchronised random stream, entry *x* mod *n* (Formulas F5). An object under another
    object owned by a slave-gathering unit (a slave) uses its type's enslaved selection list, and
    a deactivated object (one that has lost what it needs to operate) uses its deactivated selection
    list; if the list that applies is empty the object says nothing, without falling back to the
    normal list. When one input selects several objects (a band box, a group recall), only the
    first object that becomes selected speaks; selecting all objects of a type gives no response at
    all. A plain click on an object deselects everything first, so clicking an already selected
    object makes it speak again; a modifier-click that removes an object from the selection gives no
    response. Selecting an object of a passive house takes the raw value but gives no response.
    - Backing: `measurable` (scenarios D2, D3)
    - Confidence: high (draw, first-object rule, type-select silence); medium (which objects count
      as slaves and as deactivated, band-box order, re-click)

18. **Orders.** For ordinary immediate battlefield clicks (not the planning-mode path), only the first
    object in selection order can respond, whether or not it is itself ordered (S-0214, claim 14).
    Selection order: an object whose type's primary weapon does positive damage is put at the
    front of the selection when it is selected, and any other object at the end. So the object
    that responds is the most recently selected object of a positive-primary-damage type if the
    selection holds one, and the earliest selected object otherwise.
    Its response depends on the order kind (Formulas F6): move and attack-move use the move
    list; attack uses a weapon-specific single response when the type has one for the weapon it
    would use against that target and its veterancy (primary or secondary, normal or elite), and
    the attack list otherwise; enter, capture and harvest use their single responses, each falling
    back as F6 shows; unload and deploy use the deploy or undeploy single response; every other
    order kind uses the special-attack list. A list is chosen from by one raw value of the
    unsynchronised random stream (F5); a single response takes no draw. An empty list or a missing
    single response with no fallback means silence; no other object responds instead.
    - Backing: `measurable` (scenario D2)
    - Confidence: high (move, attack, special-attack list, draws, selection order); medium (enter,
      capture, harvest, deploy fallbacks)

19. **Keyboard commands.** The guard, stop and scatter commands play their game-wide interface
    sounds as screen sounds instead of a response. The deploy command has a game-wide sound that the
    original never reads from the rules, so it is always silent (quirk
    `quirks.deploy_command_silent`).
    - Backing: `measurable` (scenario D4)
    - Confidence: high

### Event sounds

20. **Damage and death.** The common sound reactions to a hit on an infantry unit, vehicle,
    aircraft or building follow its damage-state result (S-0302). A berserk-causing hit has no
    such reactions (S-0302 claim 10). This does not override kind-specific death reactions:
    - *unchanged* (or any result beyond *now dead*): the type's damage sound plays at the object,
      positional, any owner. It is started twice, as two instances of the same entry (quirk
      `quirks.damage_sound_doubled`; the per-entry limit of claim 8 applies to both),
    - *now yellow*: one R(0, 99) is drawn from the unsynchronised random stream for every object
      whose type has a feedback list, whoever owns it; if the result is below 30 and the owner is
      the local player, one feedback response is chosen by a raw value (F5) and played at the
      object, positional (not through the response slot). So a 30 % chance, only on the hit that
      turns the object yellow,
    - *unaffected* or *now red*: nothing,
    - *now dead*: if the owner is the local player and the type has a death response list, one is
      chosen (F5) and played at the object; then, for every owner, if the type has a death sound list
      one is chosen (F5) and played at the object. Both positional.
    Buildings also have the game-wide building damage and death sounds of claim 23.
    - Backing: `measurable` (scenario F1)
    - Confidence: high (common reactions); low (multiplicity on infantry death). Exactly once
      per infantry death is not established and is `unmeasured`: the relationship between its
      common and kind-specific death paths is not established for every death outcome.
      This is Q-0264's remaining multiplicity question; F9 distinguishes one pair from two
      using choice draws as well as sound capture, so channel suppression cannot masquerade
      as one request.

21. **Weapon report: one fixed variant per object.** When an object fires a weapon that has a list
    of report sounds, the report plays at the firing position, positional, any owner. The entry
    used is fixed for the object's whole life: entry *r* mod *n*, where *r* is the low 16 bits of
    the object's creation draw (S-0202, claim 10) and *n* the list's length. Firing takes no draw
    for the report. Exceptions, each taking one raw value of the unsynchronised stream per sound
    (F5): a rapid-fire weapon's spin-up sound, chosen at each spin-up step from the weapon's report
    list instead of the per-shot report; and a shot fired by occupants from inside a building,
    whose report is chosen afresh per shot. This is the normal firing rule, not an exhaustive
    classification of special weapon effects; at least one other effect makes a fresh
    unsynchronised report choice, but its gameplay identity is unresolved.
    - Backing: `measurable` (F2) for normal firing, spin-up and occupant choices.
      `unmeasured` for the full exception
      classification (the additional effect has not been identified or given a scenario)
    - Confidence: high (fixed variant, no draw); medium (spin-up and occupant exceptions)

22. **Impacts, explosions and animations.** Warheads have no sounds of their own: an impact is heard
    through the start sound of the explosion animation it creates (S-0109, claim 13), and an
    animation's stop sound plays when it ends. Both are positional, any owner.
    - Backing: `measurable` (scenario F3)
    - Confidence: high

23. **Movement, turret, building and unit events.** All positional at the object unless stated:
    - *Movement:* the start of a movement-sound episode chooses one movement-list entry (F5)
      and requests it as an attached sound. The episode is marked begun even if the request is
      inaudible. Movement resets a grace counter to 3 ticks. After stopping, three updates
      reduce it to zero and the following eligible update asks the sound to finish looping,
      not to stop its current sample immediately. Falling, crashing and certain carried-unit
      states can prevent a start or end the episode earlier. The exact gates of those states
      remain unresolved.
    - *Turret:* a turreted type's rotation sound plays as an attached sound while the turret turns.
    - *Unit ready:* a new infantry, vehicle or aircraft that leaves its factory plays its type's
      creation sound, or the game-wide creation sound of its kind (infantry, vehicle, aircraft)
      when the type has none, for any owner (medium for the owner).
    - *Building placement:* when the local player's house places a building, the game-wide
      building placement sound plays as a screen sound.
    - *Sale:* an own ordinary building's sell and pack-up sounds are positional at the
      build-down-start step of S-0408 claim 6. Packing up omits the sell sound (S-0415).
      An ordinary wall sale by the local player plays the sell sound as a screen sound.
      Only a silent wall sale omits it (S-0408 claim 12).
    - *Building damage and death:* a building's own damage and death sounds (claim 20), or the
      game-wide fallbacks of claim 37 (S-0302 claim 12 owns when the damage results occur).
    - *Promotion:* the veteran or elite promotion sound, for the local player's objects (S-0311).
    - Crushing (S-0210), jumps (S-0207), radar on and off (S-0410), credit ticks (S-0416), the idle
      comment (S-0107, claim 14) and triggered sounds (S-0011, claim 32) are specified there.
    - Backing: `measurable` (scenario F4)
    - Confidence: high (movement draw, placement, sale); medium (turret, grace counter).
      Unit-ready owner gating and the complete movement-state gates are `unmeasured`, low
      confidence: upstream production and specialised motion paths are not traced here and F4
      tests only ordinary local production and ground movement.

24. **Interface sounds and refusal cues.** Sidebar clicks, tab and button clicks and the other
    interface sounds are screen sounds of game-wide entries (Import mapping). Revision 2 corrects
    revision 1, which gave every refused click the game-wide refusal sound. The cues are:
    - *start, queue, resume, hold, cancel* (S-0411 claim 12): the game-wide build click sound, plus
      the matching announcement of claim 28.
    - *busy production line* (S-0411 claim 12: a building or defence click while that line already
      holds an item): only the "unable to comply" announcement, no interface sound.
    - *refused placement* (S-0411 claim 10, S-0406): only the "cannot deploy here" announcement, no
      interface sound. It is requested both by the check made at the click and when a place order of
      the local player's house fails its placement test as it is applied. An order ignored because
      no eligible factory remains is silent (S-0406 claim 1), not a refused-placement cue.
      Announcement playback remains subject to claim 27.
    - *production request refused as it is applied* (S-0404): when the house's queue count has
      reached the imported queue limit, or the type has reached its build limit, the game-wide
      refusal sound plays, for the local player's house only. This is neither the busy-line
      announcement nor the refused-placement announcement above.
    - *refused route-mode click* (S-0213 claim 16): the game-wide refusal sound together with
      the message. Notices before the next tick completes share one message-and-sound cue.
      Completing a tick permits a new cue. A refusal seconds later in the same running game
      therefore gives feedback again, not silence for the rest of that game.
    - *sidebar strip scrolled past its end*: the game-wide refusal sound.
    None of these depends on the local player's side. Some refused movement orders of vehicles and
    infantry also play the refusal sound; their conditions belong to the movement specs. Clicking
    an item already at its build limit gives no order (S-0411 claim 4) and plays no click sound and
    no announcement. A second "unable to comply"
    request that can follow a build click has not been tied to an S-0411 row (open). No cue is added
    for an ignored order or for a sidebar action whose feedback S-0411 leaves unresolved.
    - Backing: `measurable` (F5 for the sidebar cues, full-queue refusal and refused placement,
      F8 for repeated route-mode feedback after completed ticks). Coalescing multiple route
      notices before a tick completes is `unmeasured`: F8 does not isolate such notices.
      Build-limit refusal at application is `measurable` in S-0404 P3(k), using its
      issue/application recorder and void rule to distinguish that refusal from an at-limit
      click that issues no order. Strip scrolling, movement refusals and other button and
      tab cues are `unmeasured`: F5 does not isolate those actions.
    - Confidence: high (sidebar click cues, busy line, refused placement, repeated route cues),
      medium (refusal as the request is applied); low (movement refusals)

25. **Lightning storm sound.** Each lightning bolt of the weather superweapon's storm (S-0203,
    *storm bolt*) that strikes plays one of the game-wide lightning sounds at the strike point,
    positional, chosen by one raw value of the scenario random stream (F5). This is a draw of the
    synchronised game state, taken whether or not anyone hears it, whatever the sound settings, the
    view or the shroud. Within one strike it comes after the raw value that chooses the bolt's
    animation and that animation's creation draws, and before the draws of the strike's explosion
    animation and of its damage. No value is taken when the game-wide lightning sound list is empty,
    nor when the strike falls on exactly the same point as the previous strike, which ends that
    strike right after its bolt animation.
    - Backing: `measurable` (scenario F6)
    - Confidence: high (stream, guard, position within the strike); medium (the same-point case)

### Announcements

26. **Announcement data.** The announcement data file has a list section of announcement names,
    and each name's section gives a kind (default *standard*), a priority (default *normal*), and one
    sample file name per voice set (Allied, Soviet, Yuri), cut to 8 characters with the extension
    added. An empty or unknown kind or priority word keeps the value already in force (the default
    for a new entry). Duplicate list names re-read the same entry. A volume key exists and has
    no effect (quirk `quirks.announcement_volume_ignored`). The voice set is fixed when the game is
    loaded from the local player's side: the first side's set, the second side's set, any other
    side the third set.
    - Backing: `measurable` (scenario G1)
    - Confidence: high (keys, voice set); medium (volume ignored)

27. **One at a time, with a gap.** Announcements play one at a time, separate from the 16
    channels. A request is ignored while announcements are muted (during a movie or the
    teleport screen effect), and when it is the announcement playing at that moment. Otherwise:
    - an *interrupt* request stops the current announcement and empties every waiting list
      when there is a current announcement; without one, it does not flush waiting lists,
    - a request for an announcement already waiting with the same kind is not added again,
    - *queued interrupt* requests wait in one list, *critical* requests that are not *queued* wait
      in a second, and *queued* requests wait in one list per priority; all are first come, first
      served and unlimited,
    - every other request (a *standard* or *interrupt* one below *critical*) gets the single
      *standard* place only if the first two lists are empty and the place is empty or holds an
      announcement of strictly lower priority; otherwise it is dropped.
    The next announcement starts when the previous one has finished and more than 500 ms of the
    audio clock have passed since it started, taken from the queued-interrupt list, then the
    critical list, then the standard place, then the queued lists from *critical* down to *low*.
    Taking one from the first two lists discards whatever holds the standard place.
    - Backing: `measurable` (G2) for queue and gap rules. The muted-request rule is
      `unmeasured`: G2 has no request isolated inside the teleport screen effect, and movies
      are outside this skirmish scope. Unlimited queue capacity is `unmeasured`: a finite
      sequence cannot establish the absence of a larger cap.
    - Confidence: high

28. **The announcements of a skirmish.** Every announcement below is heard only by the local
    player, with the house conditions given; Formulas F7 gives the nearby-event suppression that
    several of them pass through:

    | Announcement | Raised when | Throttle |
    |---|---|---|
    | base under attack | one of the local player's buildings takes damage from an attacker (walls and some exempt types excepted); also plays the game-wide base under attack sound | F7, kind *base attacked* |
    | ally under attack | the same for a non-passive house allied to the local player; also plays the same game-wide attack-warning sound | F7, kind *ally attacked* |
    | harvester under attack | the local player's harvester (or deployed slave miner) takes damage without dying | F7, kind *harvester attacked* |
    | low funds | the local player's available credits are below 100, its speech-delay countdown has expired, and it owns at least one infantry, vehicle, building or naval factory | restarts that countdown at N(⌊*k* × 900⌋) ticks, *k* the speak-delay rule value in minutes; the depot's low-funds case is S-0314's |
    | low power | the local player's output is below its positive drain and it owns at least one of the first three types in the imported construction-building list | once per low-power episode: again only after power has recovered |
    | unit ready | a produced unit of the local player leaves its factory | F7, kind *unit produced*, at the factory |
    | construction complete | a building in the local player's sidebar becomes done | none |
    | new construction options | a new item appears in the sidebar, not during map loading | none |
    | building, training, on hold, cancelled, unable to comply | the matching production-feedback click of S-0411 claim 12 | none |
    | structure sold, unit sold | S-0408 | none |
    | unit lost | an infantry unit, vehicle or aircraft of the local player dies (some types exempt) | F7, kind *unit lost* |
    | unit repaired | S-0314 | F7, kind *unit repaired* |
    | unit promoted | S-0311 | none |
    | primary building selected, rally point set | the local player's matching action | none |
    | building captured, tech building lost | ownership changes involving the local player | none |
    | structure garrisoned, structure abandoned | the local player's building gets its first occupant or its last leaves | F7 for abandoned |
    | cannot deploy here | S-0415, S-0406 and a refused placement (S-0411 claim 10, claim 24) | none |
    | building online, building offline | S-0405 | none |
    | superweapon ready | the local player's superweapon finishes charging | none |
    | superweapon detected | another, non-allied house places a superweapon building | none |
    | superweapon activated or launched | any house's superweapon fires; heard by every player | none |
    | victory, defeat, player defeated | S-0414 | none |

    No announcement takes a random draw.
    The table lists common announcements, not every skirmish announcement: infiltration,
    alliances, beacons, crates and specialised abilities also raise announcements whose complete
    conditions belong to their feature specs. Superweapon launch and detection announcements
    additionally respect the game's announcement-muting setting; detection has type-specific
    eligibility conditions not resolved here.
    - Backing: `measurable` (scenario G3) for ready, low funds, low power, base and harvester
      attack; other rows inherit the scenarios of their cited specs where those establish the
      feedback. Announcement draw absence is `measurable` for G1's tested events and
      `unmeasured` exhaustively: G1 does not exercise every announcer event. Remaining
      ownership, exemption and superweapon-detection conditions are
      `unmeasured`: this verification has not traced all of those event paths and G3 does not
      exercise them. They are provisional, not complete eligibility rules.
    - Confidence: high (the traced thresholds and throttles); low (unresolved eligibility)

### Determinism

29. **Nothing in the game reads sound.** No rule of the game depends on whether, when or how a sound
    or announcement played, on the audio clock, or on the audio passes.
    - Backing: `measurable` (scenario H1) for the tested events; `unmeasured` for the exhaustive
      absence of game-rule readers (a finite comparison cannot establish every sound path)
    - Confidence: medium (exhaustiveness)

30. **Draw summary.** All traced draws in claims 13, 14, 17, 18, 20, 21 (exceptions) and 23 use
    the unsynchronised stream. State and ordering lists the immediate-mode instance draws.
    Delayed-mode full ordering remains open. Instance choices happen at playback service
    opportunities, rather than all at the game-side request, so their position in the stream
    depends on real time and playback interruptions. Claim 14 defines
    immediate-mode draws for uninterrupted playback, not a fixed count for every instance.
    Our engine retains this shared unsynchronised stream, as required by S-0202 claim 11,
    for both game-side sound choices and per-instance choices; no separate presentation
    generator is substituted. Claims 12 to 14's audio-clock eligibility, delays and sample
    rules apply to our engine too. The exact host service times need not match across runs:
    presentation choices need not be tick-identical on replay. Scenario-stream state and game
    rules must remain unaffected (claim 29).
    - Backing: `measurable` (E1 for instance draw order, E2 for audio-clock eligibility,
      H1 for separation from game state). The engine contract applies those traced rules,
      not a new oracle measurement.
    - Confidence: high

31. **Response draws come before the local-player check in two places.** The feedback chance of
    claim 20 is drawn for every owner's object; a selection response draw for a passive house's
    object (claim 17) is taken and then discarded. Only the unsynchronised stream is affected.
    - Backing: `measurable` (F1 for other owners' feedback draws, D3 for passive selection)
    - Confidence: high

32. **Weapon report variant depends on the creation draw.** The report variant (claim 21) is
    determined by the scenario random stream through the object's creation draw, so it is the same
    on every replay of the same game, and a change to the creation draws changes which report each
    object uses.
    - Backing: `measurable` (scenario F2)
    - Confidence: high

33. **Two traced sound-related scenario choices.** Of the traced sound-selection paths here,
    only the lightning sound choice
    (claim 25) takes a draw from the scenario random stream. Apart from it, the trigger action that
    plays a sound at a random waypoint, which is not in S-0011's multiplayer subset, takes one draw
    R(0, *w* − 1) over its *w* waypoint markers to choose the waypoint (medium). Both happen in every
    execution whether or not anyone hears them. The random-waypoint action remains outside this
    skirmish trigger subset: this records its original behaviour, not a requirement to add that
    action. Exact waypoint enumeration and execution guards remain with the future trigger spec.
    - Backing: `measurable` (scenario F6) for lightning; `unmeasured` for the random-waypoint
      action's complete guards and for exhaustiveness (no scenario here exercises that action,
      and the scan does not prove the absence of every indirect sound-related draw)
    - Confidence: high (lightning); medium (random-waypoint range); low (exhaustiveness)

### Sample sources and volume options

34. **Where samples come from.** Every sample a sound entry names is looked up, ignoring case, in
    one sample bank: an index file and a data file (Import mapping), found through the ordinary
    file lookup of S-0005 claim 10a. The two files resolve independently: a loose index can
    accompany archived data, and the files need not come from the same archive. Sound entries
    never play separate sample files. Of the installation's two audio archives the expansion's is
    mounted when present, else the original's (S-0005 owns the mounting). Announcements work the
    other way: each is a separate file named by claim 26's eight-character name plus the extension
    `.wav`, opened through the same file lookup and never taken from the bank. If the bank's index
    lists one name twice, which copy is used is not defined: the original assumes unique names.
    The public byte layout of the bank is outside this spec; no new formats spec is required.
    - Backing: `measurable` (scenario A4 for separate files versus bank samples and
      case-insensitive sample lookup; S-0005 M22 part a for archive fallback).
      Independent replacement of the two bank files is `measurable` in S-0005 M25(b),
      which supplies compatible distinguishable bank fixtures; A4 alone does not test it.
      Duplicate names are `unmeasured`: no scenario controls their ordering, and no
      deterministic duplicate-name outcome is specified.
    - Confidence: high (one bank, separate announcement files); low (duplicate names)

35. **Volume options.** The player's sound volume option is *o* of F3: it scales every sound
    instance, including responses, interface and event sounds. It does not scale announcements.
    Announcements follow the player's separate voice volume option instead, and the master sound
    level of F3 does not apply to them either. A screen request made with a fraction *v* of full
    volume has request level ℓ = min(trunc(*v* × 16384), 16384), *v* in floating point. The credit
    ticks of S-0416 are made at half volume, so ℓ = 8192: one halving of the level, about 10 dB
    below the same entry requested at full level (F3, F4; the exact *k* also depends on
    `quirks.full_volume_16383`).
    - Backing: `measurable` (scenario I1)
    - Confidence: high (which option scales what); medium (credit tick level)

### Announcement entries and building fallbacks

36. **Which entry each announcement event requests.** The Import mapping's announcement request
    table gives the fixed entry name requested by each listed event, not a sample name or a
    new configurable rules field. In addition:
    - A promotion to veteran and a promotion to elite request the same entry.
    - When the local player's house is defeated or loses, it hears the defeat entry unless it is an
      observer; when another non-passive house is defeated, the local player hears the
      player-defeated entry; a win of the local player's house requests the victory entry.
    - A request by house A for an alliance with house B that is not yet mutual requests the
      alliance-requesting entry when A is the local player's house, the alliance-requested entry when
      B is, and nothing otherwise. When the alliance becomes mutual, the alliance-formed entry plays
      if each of A and B is the local player's house or a house allied with it, otherwise the
      enemy-alliance-formed entry. Neither plays once the local player's house has been defeated in
      that game; that defeat also mutes the superweapon launch and detection announcements of
      claim 28. When A ends an alliance with B, the alliance-broken entry plays if B was allied
      with A and A or B is the local player's house.
    - The play-speech trigger action (S-0011 claim 32) requests the entry it names as kind
      *interrupt*, whatever kind the entry has.
    - Entry names are matched ignoring case; an unknown name plays nothing.
    These are presentation rules; they do not end a game. Our engine keeps playing to the
    last alliance under S-0414 claim 6, even after the local player's defeat. The described
    announcement muting still applies during that continued observation.
    - Backing: `measurable` (scenario G4) for the fixed entry bindings and events exercised there.
      Alliance announcement conditions are `unmeasured`: G4 supplies no isolated alliance
      sequence distinguishing each branch. Post-defeat muting is `measurable` in S-0414
      V20 for superweapon launch/detection requests isolated in a named lengthened end delay.
      Its alliance controls are conditional: if no matching computer alliance request occurs,
      alliance muting remains inconclusive, not confirmed.
      The observer defeat exception is `unmeasured`: G4 does not set up an observer.
      Muting during our engine's extended observation is a presentation requirement, not
      a claim that the original provides that observation period.
    - Confidence: high (entry per event); medium (alliance conditions)

37. **Building damage and death fallbacks.** When a building's damage result (S-0302 claim 12) is
    *now yellow* or *now red* and its type names no damage sound of its own, the game-wide building
    damage sound plays at the building's centre in the attached form that follows scrolling
    (claim 7). When a building is destroyed and its type's death-sound list is empty, the game-wide
    building death sound plays the same way. A type with its own sound uses claim 20 instead.
    - Backing: `measurable` (scenario F7)
    - Confidence: high (damage fallback); medium (death fallback)

## Formulas

**F1. Entry level** (claim 2). The percentage is read through S-0003's floating-point import,
then multiplied and clamped in double-precision arithmetic under S-0205's rounding mode, without
an intervening single-precision rounding of the product. The scale constant is the exactly widened
single-precision approximation to 0.01. Conversion to the integer level truncates toward zero:

> *e* = ⌊clamp(*v* × 0.01, 0, 1) × 16384⌋, where *v* is the volume percentage as read and 0.01 is
> 0.00999999977648258209228515625 (slightly below one hundredth).

This formula applies with `quirks.full_volume_16383` on (the default). With it off,
*v* ≥ 100 instead gives *e* = 16384 exactly; for *v* < 100 the formula is unchanged.
Thus *v* = 100 gives 16383 with the quirk on and 16384 with it off. With either setting,
*v* of 0 or less gives 0. The off-state override is an engine design requirement,
not an additional oracle claim.
The minimum level uses the same scaled clamp, but is then narrowed to single precision as a
fraction; the quirk does not change that fraction.

**F2. Positional level and pan** (claim 6). Floating point with single-precision stored distances
and level, with arithmetic results formed in double precision before each stated single-precision
storage, following S-0205's arithmetic and round-toward-zero mode, except at the stated integer
conversions.
*W* and *H* are the battlefield view's width and height in pixels; (*p*ₓ, *p*ᵧ) is the pixel at which
the sound's position (with its height) appears in the view, measured from the view's top-left corner,
and *R* = 60 × range (integer, pixels).

> *δ* = *p*ₓ − *W*/2
> *u*ₓ = |trunc(*δ*)|, *u*ᵧ = |trunc(*p*ᵧ − *H*/2)|
> With *local*: *a*ₓ = *u*ₓ and *a*ᵧ = 2 × *u*ᵧ.
> Otherwise: *a*ₓ = max(0, *u*ₓ − *W*/2) and *a*ᵧ = 2 × max(0, *u*ᵧ − *H*/2).
> These subtractions remain in single-precision stored floating point, without integer truncation.
> *L*₀ = (*R* − max(*a*ₓ, *a*ᵧ)) / *R* when *a*ₓ < *R*, *a*ᵧ < *R* and *R* > 0, and 0 otherwise.
> *L* = max(*L*₀, minimum level) with *global*, and *L* = *L*₀ otherwise.
> The sound is not started when *L* < 0.05 (comparison in double precision).
> Request level *ℓ* = min(trunc(*L* × 16384), 16384).
> Pan = trunc(clamp(*δ*, −*W*, *W*) × 8192 / *W* + 8192).

A screen sound has *ℓ* = 16384 by default and pan 8192. An explicit volume fraction uses
claim 35's request-level formula instead.

**F3. Combined level** (claims 13, 15). Integers; each product is truncated:

> *σ* = 16384 − ⌊*s* × 16384 / 100⌋ (volume-shift factor, *s* from claim 13)
> heard level *h* = ⌊⌊⌊⌊*e* × *ℓ* / 16384⌋ × *σ* / 16384⌋ × *g* / 16384⌋ × *o* / 16384⌋

where *g* is the game's master sound level and *o*
the player's sound volume option, stored as trunc(option fraction × 16384). Announcements use
neither *g* nor *o* (claim 35). Pans combine as
clamp(*p*₁ + *p*₂ − 8192, 0, 16384), and every stage except the request is centred, so the heard pan
is the request's pan. Pitches combine as trunc(*p*₁ × *p*₂ / 100), in integer percent. The order of the four truncations is
medium confidence; the result differs by at most a few units of 16384.

The master level is not necessarily full even in an ordinary skirmish. At an audio pass, let *C*
be the sum of ⌊20 × *e* / 16384⌋ over surviving, non-paused sound instances. Its target level is
16384 when *C* ≤ 100, and ⌊1638400 / *C*⌋ otherwise. These are exact integer operations. The
current master level moves toward that target on the audio clock, so many simultaneous sounds
reduce the loudness of all sounds. The exact transition timing is `unmeasured`, medium confidence:
no scenario yet distinguishes the fractional level ramp from an instantaneous target change.
The target is `measurable` in C3, high confidence.

**F4. Output law** (claim 15). Integers.

> *k* = ⌊*h* × 100 / 16384⌋ (a percentage, 0 … 100)
> attenuation *T*(*k*) = round-nearest(1000 × log₂(*k* / 100)) hundredths of a decibel for 1 ≤ *k* ≤ 100,
> and *T*(0) = −10000 (silence)
> *q* = ⌊pan × 200 / 16384⌋ − 100 (−100 full left … 100 full right)
> balance: for *q* < 0 the right side is attenuated by −*T*(100 + *q*); for *q* ≥ 0 the left side by
> −*T*(100 − *q*)
> playback rate = trunc(the sample's rate × pitch / 100), integer samples per second

Here round-nearest means the nearest integer; none of these 100 nonzero arguments is an exact
half-integer, so no tie rule is needed. This logarithmic formula specifies the integer table,
not a floating-point computation during playback.

**F5. Choice from a list** (claims 17, 18, 20, 21, 23, 25). Over a list of *n* ≥ 1 entries, numbered
from 0 in the listed order:

> *i* = *x* mod *n*, *x* the next raw value of the stated stream (unsigned 32-bit)

No raw value is taken when *n* = 0. For the weapon report of claim 21, *x* is replaced by the low 16
bits of the object's creation draw and no value is taken.

**F6. Order response** (claim 18). By order kind:

| Order | Response used | Fallback |
|---|---|---|
| move, attack move | move list (F5) | none |
| attack | the single response for the weapon chosen against the target: primary or secondary, elite or not | attack list (F5); then none |
| attack with a weapon of negative damage by the repair-response vehicle | the game-wide repair-vehicle response | attack list (F5); then none |
| enter | enter response | move list |
| capture | capture response | enter response, then move list |
| harvest | harvest response | move list |
| unload, deploy | infantry that is deployed: undeploy response; otherwise deploy response | none |
| any other order | special-attack list (F5) | none |

The repair-vehicle row is medium confidence (scenario D2's repair variant). For the four weapon responses, weapon number 0
uses the primary response and any nonzero number uses the secondary response, including a
multi-weapon type. A building's deploy acknowledgement is a specialised override, not the
infantry deploy row: its exact response and the one-request deploy suppression remain unresolved
under Q-0265. These exceptions are `unmeasured`, low confidence: their observable eligibility
has not been translated into a complete rule. D2 tests ordinary responses only; a settling
extension should repeat deploy/undeploy on a slave-gathering building while linked to its
vehicle and while unlinked, recording single-response identities and omissions.

**F7. Nearby-event suppression** (claim 28). Each announcement that passes through it first tries to
create a radar event of its kind at a cell (the building, unit or factory). The attempt fails, and the
announcement is not raised, when a radar event of the same kind exists whose cell is at a distance
⌊√(Δ*x*² + Δ*y*²)⌋ < *r* cells (floating point, truncated), *r* from this table. The event lasts its
zoom-in phase on the radar and then *t* ticks:

| Kind | *r* (cells) | *t* (ticks) | Suppresses |
|---|---|---|---|
| combat | 8 | 400 | yes |
| non-combat | 8 | 400 | no |
| drop zone | 8 | 400 | no |
| base attacked | 8 | 600 | yes |
| harvester attacked | 8 | 400 | yes |
| enemy sensed | 6 | 400 | yes |
| unit produced | 2 | 200 | yes |
| unit lost | 8 | 200 | yes |
| unit repaired | 2 | 400 | yes |
| infiltrated | 5 | 400 | no |
| captured | 8 | 100 | no |
| beacon | 8 | 200 | yes |
| superweapon detected | 8 | 400 | no |
| superweapon activated | 8 | 5 | no |
| bridge repaired | 8 | 200 | yes |
| garrison abandoned | 8 | 400 | yes |
| ally attacked | 8 | 600 | yes |

For a kind marked *yes*, radar events are game-wide, not per house: a matching event suppresses
another request regardless of why the first event was created, including triggered events
(S-0011 claim 32). Kinds marked *no* do not apply the distance rejection. The complete set of
event creators is not established here. F7 is high confidence and `measurable` in G3 for
*unit produced*, *base attacked* and *harvester attacked*. The remaining table rows and
cross-creator suppression are `unmeasured`: G3 has no isolated request sequence for those
kinds or creators. The unresolved zoom timing is not a resolved tick duration.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Channels | 16 | channels | claim 9 |
| Instance pool | 300 | instances | claim 10 |
| Positional one-shot pool | 200 | instances | claim 10 |
| Most samples per entry | 32 | samples | claim 2 |
| Longest entry name | 31 | characters | claim 1 |
| Default volume | 80 | percent | claim 2, absent from both the entry and the defaults section |
| Default minimum level | 20 | percent | claim 2 |
| Default limit | 5 | instances | claim 2 |
| Default range | 10 | cells | claim 2 |
| Default sound priority | *normal* | priority category | claims 2, 3 |
| Default placement | *screen* | placement category | claim 2 |
| Pixels per cell of range | 60 | pixels per cell | F2 |
| Vertical distance weight | 2 | dimensionless multiplier | F2 |
| Start threshold | 0.05 | level fraction | claim 6 |
| Equal-loudness band | 1/10 | of the entry's level | claim 8 |
| Audio pass interval | more than 33 | ms, audio clock | claim 12 |
| Smallest effective delay | 33 | ms | claims 13, 14 |
| Ambient pre-delay minimum | 33 | ms | claim 13 |
| Announcement gap | more than 500 | ms from the start of the previous one, audio clock | claim 27 |
| Feedback chance | 30 in 100 | percent probability | claim 20 |
| Movement sound grace counter | 3 | ticks | end-loop request on the following eligible update, claim 23 |
| Low funds threshold | 100 | credits | claim 28 |
| Low funds repeat | N(⌊*k* × 900⌋) | ticks | claim 28 |

## State and ordering

**Response slot of an object** (claim 16):

| State | Event | Result |
|---|---|---|
| idle | response *r* requested | waiting *r* |
| waiting *r* | response *r*′ requested | waiting *r*′ (replaces) |
| waiting *r*, not speaking | object update | speaking *r*; slot empty |
| waiting *r*, speaking *r* | object update | slot empty (dropped) |
| waiting *r*, speaking another | object update | still waiting *r* |

**Draws of one sound instance** (claims 13, 14), in this order, all from the unsynchronised stream.
A range with equal ends takes nothing.

| Order | What | Range | Count |
|---|---|---|---|
| 1 | pitch | R(*f*₁, *f*₂) | 1 |
| 2 | volume shift | R(0, *vs*) | 1 |
| 3 | pre-delay (only *pre-delay* or *ambient*) | R(*d*₁ or 33, *d*₂) | 1 |
| 4 | attack sample (*A* > 0) | R(0, *A* − 1) | 1 |
| 5 | body sample (*random*, not *all*) | R(*A*, *n* − *D* − 1) | 1 |
| 6 | decay sample (*D* > 0) | R(*n* − *D*, *n* − 1) | 1 |
| 7 | discarded startup ordering (*random*, immediate, no attack sample) | R(0, *m* − 1) | one before audible ordering |
| 8 | audible body order (*random*, immediate) | R(0, *m* − 1) | one per body sample still to place, *m* down to 1; equal ends consume no raw value |

Rows 4 to 8 describe immediate mode only. Delayed mode's full draw ordering remains open (claim 14).

**Game-side sound draws** (claims 17 to 25): one raw value per list choice, at the moment the sound is
requested, in the tick's order of events; the feedback chance R(0, 99) precedes its choice; for a
death, the death response choice precedes the death sound choice.

**Ordering.** Requests wait for an audio pass to obtain channels; pre-delays can prevent requests
from one tick from becoming audible together. A per-entry
limit (claim 8) is applied before channels are given out (claim 9), so a sound cut by its own limit
never takes a channel or the draws of claim 13.

## Edge cases and quirks

- **`quirks.sound_order_startup_draw`** (default: on). Immediate-mode random ordering with no
  attack sample discards one startup ordering draw (claim 14). Off: omit that discarded draw.
- **`quirks.missing_sample_silences_entry`** (default: on). One sample missing from the archive
  silences the whole entry. Off: the missing sample is left out of the list and the rest play.
- **`quirks.default_sound_priority_ignored`** (default: on). The defaults section's priority has no
  effect. Off: it is the default for entries without their own priority.
- **`quirks.full_volume_16383`** (default: on). A volume of 100 % gives level 16383. Off:
  entry volume percentages of 100 or more give level 16384, while lower percentages keep
  F1's original calculation. Minimum-level fractions are unchanged.
  The exact integer distinction is unmeasured as explained in A2; its downstream output
  percentage still follows F3/F4.
- **`quirks.attack_decay_without_body`** (default: on). An entry with the *attack* or *decay* word
  whose samples leave no body sample makes no sound when first played, and after that the entry is
  permanently silent for the rest of the session. Off: the attack and decay samples play.
  Backing: `measurable` (A2) for initial silence, `unmeasured` for permanent invalidation:
  A2 cannot restore a valid body to the already-loaded entry. Confidence: high.
- **`quirks.shroud_sound_corner_cell`** (default: on). A *shroud* sound at cell (0, 0) is never heard.
  Off: that cell is judged like any other.
- **`quirks.damage_sound_doubled`** (default: on). The damage sound of an object starts twice per *unchanged* hit (louder, and two instances against its limit). Off: once.
- **`quirks.deploy_command_silent`** (default: on). The deploy command's game-wide sound is never
  read, so it is silent. Off: it is read under the key the Import mapping gives and played.
- **`quirks.announcement_volume_ignored`** (default: on). An announcement's volume key has no effect.
  Off: the volume (a fraction, default 1) scales the announcement.
- **Misspelt stock keys.** Keys are read by exact case (claim 2); stock data contains a few entries
  spelled with other capitals or a shorter name, which therefore have no effect (Import mapping).
  This is the S-0001 rule, not a separate switch.
- **No fallback between response lists.** An enslaved or deactivated object with an empty list of
  its own is silent (claim 17); the first object of a selection with no response is silent even when
  another selected object has one (claim 18).
- **Audio passes follow real time.** Claims 12 and 30 apply to both the original and our engine.
  The service interval is a strict lower bound, not an exact schedule or latency guarantee.
  Both game-side and per-instance audio choices use S-0202's shared unsynchronised stream.
  Real-time variation affects presentation only; it cannot alter the scenario stream or game
  state. No third generator or tick-identical presentation replay is required.

## Import mapping

Sound data file (the sound rules file of S-0005):

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[SoundList]`, every value | entry names, in order | string, at most 31 characters | none |
| `Sounds` | samples | words separated by space, tab or newline; leading `$` and `#` dropped; at most 32 | none (silent) |
| `Volume` | entry volume (F1) | floating-point number via S-0003, then additional × 0.01 scaling | defaults section `Volume`, else 80 |
| `VShift` | volume shift | integer percentage, clamped 0 … 100 | 0 |
| `MinVolume` | minimum level | floating-point number via S-0003, then additional × 0.01 scaling | defaults section `MinVolume`, else 20 |
| `Priority` | sound priority | one of `LOWEST`, `LOW`, `NORMAL`, `HIGH`, `CRITICAL` (any case), else `NORMAL` | `NORMAL` (the defaults section's key is read and ignored, claim 3) |
| `Attack`, `Decay` | attack and decay counts | integer | 0 |
| `Control` | control words | words `ALL`, `LOOP`, `RANDOM`, `PREDELAY`, `INTERRUPT`, `ATTACK`, `DECAY`, `AMBIENT` | defaults section `Control`, else none |
| `Type` | placement words | whitespace-separated words, matched ignoring case: `GLOBAL`, `LOCAL`, `SHROUD` directly affect positional playback; last of `SCREEN`/`LOCAL` chooses view-edge/centre distance, last of `UNSHROUD`/`SHROUD` disables/enables explored-cell gating; `AMBIENT`, `VIOLENT`, `MOVEMENT`, `QUIET`, `LOUD`, `PLAYER`, `NORMAL`, `GUN_SHY`, `NOISE_SHY` have no established playback effect | defaults section `Type`, else `SCREEN` |
| `Limit` | limit | integer, 0 = none | defaults section `Limit`, else 5 |
| `Loop` | loop count | integer, 0 = forever | 0 |
| `Range` | range | integer, cells | defaults section `Range`, else 10 |
| `Delay` | delay | one or two integers, ms | 0 0 |
| `FShift` | pitch shift | one or two integers, percent | 0 0 |
| `[Defaults]` | game-wide defaults | the keys `Volume`, `MinVolume`, `Priority`, `Control`, `Type`, `Limit`, `Range` | as above |

A percent sign in `Volume` or `MinVolume` invokes S-0003's percent conversion before the extra
scaling above: write 100, not 100 %, for the usual full-scale sound volume.
Stock data spells some keys `volume`, `Vshift`, `Fshift` and `MinVol`; under claim 2 those entries are
ignored.

Sample bank (claim 34): the index file `AUDIO.IDX` and the data file `AUDIO.BAG`, each found through
the independent file lookup of S-0005 claim 10a and matched ignoring case; their public byte
layout is outside this spec.
Announcement samples are separate files, the eight-character voice-set sample name
specified below plus `.wav`, not the announcement entry's name.

Announcement data file:

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[DialogList]`, every value | announcement names | string | none |
| `Type` | announcement kind | `STANDARD`, `QUEUE` (*queued*), `INTERRUPT`, `QUEUED_INTERRUPT`, any case | `STANDARD` |
| `Priority` | announcement priority | `LOW`, `NORMAL`, `IMPORTANT`, `CRITICAL`, any case | `NORMAL` |
| `Allied`, `Russian`, `Yuri` | sample per voice set | file name, first 8 characters, extension added | empty (silent) |
| `Volume` | ignored (claim 26) | floating point | 1 |

Announcement requests (claim 36): fixed section names in the announcement data file.
Lookup ignores case; an absent or unknown entry is silent. These bindings are not configurable
rules keys and do not specify sample names: the voice-set fields above specify those.

| Event | Fixed announcement entry |
|---|---|
| construction complete | `EVA_ConstructionComplete` |
| unit ready | `EVA_UnitReady` |
| new construction options | `EVA_NewConstructionOptions` |
| building | `EVA_Building` |
| training | `EVA_Training` |
| on hold | `EVA_OnHold` |
| cancelled | `EVA_Canceled` |
| unable to comply (claim 24) | `EVA_UnableToComply` |
| cannot deploy here (claim 24) | `EVA_CannotDeployHere` |
| structure sold | `EVA_StructureSold` |
| unit sold | `EVA_UnitSold` |
| unit repaired | `EVA_UnitRepaired` |
| unit promoted (either rank) | `EVA_UnitPromoted` |
| low funds (house countdown or repair-depot refusal) | `EVA_InsufficientFunds` |
| unit lost | `EVA_UnitLost` |
| low power | `EVA_LowPower` |
| base under attack | `EVA_OurBaseIsUnderAttack` |
| ally under attack | `EVA_OurAllyIsUnderAttack` |
| harvester under attack | `EVA_OreMinerUnderAttack` |
| defeat | `EVA_YouHaveLost` |
| player defeated | `EVA_PlayerDefeated` |
| victory | `EVA_YouAreVictorious` |
| alliance requesting | `EVA_RequestingAlliance` |
| alliance requested | `EVA_AllianceRequested` |
| alliance formed | `EVA_AllianceFormed` |
| enemy alliance formed | `EVA_EnemyAllianceFormed` |
| alliance broken | `EVA_AllianceBroken` |
| play-speech trigger action | entry named by the action, with interrupt kind override |

Rules file, object type sections (infantry, vehicle, aircraft, building), each a sound name or a list of
sound names (S-0003). Lists are comma-separated, limited to the first 127 value characters, keep
listed order and drop unknown names. An absent list keeps its earlier value, initially empty.
A present nonempty list replaces it. A single sound reference that is absent or unknown keeps
its earlier value, initially no sound. Every reference is compared in full with the entry names,
so a reference longer than 31 characters matches no entry (claim 1). These defaults apply to every row below:

| Key | Concept | Form |
|---|---|---|
| `VoiceSelect`, `VoiceSelectEnslaved`, `VoiceSelectDeactivated` | selection lists (claim 17) | list |
| `VoiceMove`, `VoiceAttack`, `VoiceSpecialAttack` | order lists (F6) | list |
| normal primary-weapon response | F6 response | name |
| elite primary-weapon response | F6 response | name |
| `VoiceSecondaryWeaponAttack` | normal secondary-weapon response (F6) | name |
| elite secondary-weapon response | F6 response | name |
| `VoiceEnter`, `VoiceCapture`, `VoiceHarvest`, `VoiceDeploy`, and the undeploy response | single responses (F6) | name |
| `VoiceFeedback`, local death response, `DieSound` | damage feedback, local death response and any-owner death sound, respectively (claim 20) | list |
| `DamageSound` | damage sound (claim 20) | name |
| `MoveSound` | movement list (claim 23) | list |
| `TurretRotateSound`, `CreateSound` | turret, unit ready (claim 23) | name |
| `CrushSound`, `ChronoInSound`, `ChronoOutSound`, `VoiceComment` | S-0210, S-0207, S-0107 | as there |

Building type sections: the following are single sound names, with the single-reference parse
form and defaults above. Their detailed triggers remain with the structures specs.

| Key | Concept |
|---|---|
| `PackupSound` | packing or build-down sound (claim 23) |
| build-up sound | build-up sound |
| `CreateUnitSound` | factory unit-creation sound |
| building entry and exit sounds | building entry and exit sounds |
| `WorkingSound`, `NotWorkingSound` | working and not-working sounds |
Weapon sections: `Report` (list, claim 21). Animation sections: `StartSound` (or `Report`, S-0109) and
`StopSound` (claim 22).

Rules file, `[AudioVisual]`, sound names: `CreateUnitSound`, `CreateInfantrySound`, `CreateAircraftSound`
(claim 23), `BuildingSlam` (placement), `SellSound`,
`UpgradeVeteranSound`, `UpgradeEliteSound`, `BaseUnderAttackSound` (claim 28), `GuardSound`, `StopSound`,
`ScatterSound` (claim 19), `ScoldSound` (refusal, claim 24), `GUIBuildSound` (build click, claim 24),
`BuildingDamageSound` and `BuildingDieSound` (also the fallbacks of claim 37),
the other `GUI…` keys, `GenericClick`, `RadarOn`, `RadarOff`, `VoiceIFVRepair` (F6), `LightningSounds`
(list, claim 25). F6's repair-response exception depends on a particular vehicle's type name,
not its display or image name; it is not selected by an imported repair-vehicle field.
The repair-response vehicle is the vehicle type named `FV`; comparison ignores case and uses
its type name, not an image name.
The construction-building list tested for low power is `BuildConst` in `[AI]`; only its first
three entries are counted for this announcement.
`DeploySound` in that section is never read (claim 19). `SpeakDelay` (`[AudioVisual]`,
floating point, minutes) is the low-funds repeat *k* of claim 28.

## Measurement scenarios

All runs: the measurement setup (glossary), with the director and mods off, skirmish,
800 × 600, normal game speed unless stated, sound effects volume at full. Use a small flat
purpose-made map with the named test objects in an isolated observed area and one bystander
computer player kept away from it; apply the glossary's void rule. Unless a count is given, run
three repetitions of each deterministic sub-test; record identity, stereo level and sample start
times, not merely whether some sound was heard. Every scenario that specifies test entries, altered response lists or altered announcement
settings uses named test files (glossary, *measurement setup*): a loose copy of the sound data file
or of the announcement data file with the test entries stated, and, where a unit must use a test
entry, a loose copy of the rules file pointing one of its lists at it; all are removed after the run.
Values of stock entries are not stated here; a scenario that needs a stock entry chooses it by
property. Audio is recorded with a stereo loopback capture of the
game's output (a harness addition to propose to the user); the random-state recorder of S-0202 reads
the positions of both streams per tick.

**A1. Entry reading.** Named test files with test entries: (a) a name listed twice, both listings
referring to the same section; (b) a
listed name without a section; (c) an entry naming one sample that does not exist plus one that
does; (d) a rules reference spelled in other capitals. Map a unit's selection list to each.
For (a), use two distinct samples and sequential *all* playback: repeated listing gives four
samples in the repeated pair order. *Observable:* what is heard on selection. *Expected:* (a)
the doubled sample pair from one entry, (b) silence,
(c) silence, (d) the sound. Add (e) an unknown sound reference: expect silence, not a fallback.
Add (f) two listed entry names identical through character 31 but different afterwards.
Reference their common first 31 characters from the rules: expect one truncated-name entry
with accumulated samples as in (a). Add (g) a rules reference of 32 or more characters whose
first 31 characters equal a listed name cut to 31: expect silence. *Falsified if:* any differs.

**A2. Fields.** Test entries with: a 33-sample list (the 33rd distinct); `$`-prefixed names; volume 50
against 100 (heard level ratio about −10 dB); a control word list given with commas. *Expected:* the
33rd never plays; prefixed names play; the level ratio; the comma list acts as one unknown word (no
control words), replacing any inherited control set. Add an attack/decay-only entry with no body,
then select it twice: both are silent with the default quirk. Repeat the prefix test with
multiple leading `$` and `#` characters, and token separation by spaces, tabs, newlines and commas.
Expect whitespace separation, stripped prefixes, and no comma separation.
Use separate fixtures for inherited settings, omitted defaults, exact and altered key case,
unknown control tokens and unknown priority words. Compare their playback, limit, range and
priority with explicit entries giving claim 2's stated defaults: they must match, except that a
nonempty unknown control token clears inherited control. Exact defaults of minimum level,
limit, range and priority are compared through B2, C1 and C2; output percentages cannot
establish the entry level's last integer unit. Test volumes below 0 and above 100,
volume shifts below 0 and above 100, and minimum levels below 0 and above 100: expect the stated
clamps. Compare attack and decay count 0 with count 1 while their words are present: expect the
same partition; remove the words and expect no attack or decay partition. E1 tests the delay,
pitch and loop forms, B2 tests placement words, C1 tests limit. The inactive placement words
are `unmeasured`, medium confidence: no scenario here isolates each word's
absence of effects across all event kinds. *Falsified if:* any tested comparison differs.
F1's exact 16383 full-volume integer result is `unmeasured`, high confidence: loopback capture
cannot distinguish it independently from the later percentage conversion.

**A3. Default priority.** Defaults section priority set to critical; 17 positional instances of an
entry without its own priority are started while 16 channels are busy with a *normal* entry.
Repeat with 16 busy *high*-priority channels instead. Set per-entry limits to 0 so they do not
mask channel allocation. *Expected:* the entry without its own priority can replace *normal*
channels but cannot steal a *high* channel; an otherwise identical explicitly *critical* entry can.
*Falsified if:* the entry with no explicit priority steals a high-priority channel.

**A4. Sample sources.** (a) Put a loose `.wav` file named like a sample of a test entry next
to the game: expect the bank's sample, not the loose file, when the entry plays. (b) Put a loose
`.wav` file with a distinct identifiable sound named after an announcement's eight-character
name for the local voice set: expect that file when the announcement plays. (c) Name a
test entry's sample that exists only as a loose file and not in the bank: expect silence
(claim 4). (d) Repeat a bank sample reference with changed letter case: expect the same sample.
*Falsified if:* a sound entry plays a loose file, an announcement ignores it, or case alone
changes bank-sample lookup.

**B1. Screen sounds.** Select a unit at the far corner of the map from the view. *Expected:* the
response at full level, centred. Repeat a guard interface sound after moving the view without changing the request.
*Falsified if:* either response or interface sound changes level or pan with view position.

**B2. Positional level and pan.** A tank fires (fixed report variant, claim 21) at known screen
positions: inside the view, 1, 3, 5 cells beyond the right edge and beyond the bottom edge.
*Observable:* left and right channel levels. *Expected:* (a) full inside, linear fade to silence at
the entry's range horizontally and at half the range vertically, nothing below 0.05; (b) right-edge
source: left side 10 dB below right; (c) source at 50 % level: 10 dB below full. *Falsified if:*
another fade law, symmetric vertical range, or another pan law. (d) Named test sound data
uses the same report sample with *local*, *global* and *shroud*, separately. Compare centre,
edge and out-of-range positions at two view sizes. Expect F2's centre-distance law for *local*,
the imported minimum for *global*, and silence only in unexplored cells for *shroud* (also
cell (0, 0), even explored). Use a purpose-made map allowing that corner cell; if no legal
placement there is possible, the corner-cell quirk is inconclusive, not confirmed.
(e) Compare otherwise identical entries whose placement words are *local screen* and
*screen local*: the first uses view-edge distance, the second centre distance. Neither changes
F2's positional pan. For the shroud comparisons use a timed sound-at-waypoint action
(S-0011 claim 32) at an empty, unexplored legal cell within audible range, with no exploring
unit near it. Compare *shroud unshroud* with *unshroud shroud*: the first is audible, the
second silent. Repeat at the same explored cell: both are audible. Repeat these four
entries as selection responses: all remain centred screen requests regardless of these words.
Record both random streams for audible and sub-threshold reports with nontrivial pitch and
volume-shift ranges: no instance draws on the latter. *Falsified if:* any word-order comparison
differs, a placement word changes a screen response, levels differ from F2/F4,
the larger view does not enlarge the ordinary audible area, or inaudible reports take instance draws.

**B3. Placement forms.** Use named test sound data to give a long non-looping sample to a normal
weapon report and to an unchanged-hit damage sound, then scroll after each starts. Also scroll
away from and back to a vehicle with an indefinitely looping movement sound. *Expected:* the
report keeps its start placement, the damage sound follows scrolling and stops out of range,
and the indefinitely looping sound can resume. *Falsified if:* either one-shot follows the other
placement rule, a stopped finite damage sound resumes solely from scrolling, or the loop cannot
resume.

**C1. Per-entry limit.** An entry with limit 2, with and without *interrupt*; start 3 equal
instances in sequence. *Expected:* without *interrupt* the third is never heard; with it the first is
cut when the third starts. Repeat with a quieter and a louder third request, and levels on
both sides of the integer equal-loudness band in claim 8, computed from F2 and the imported
entry volume. Expect the lowest-ranked instance to go; with limit 0 all three survive.
Give the entry nontrivial pitch shifts and record draws: a request rejected by its entry limit
takes no instance draw. *Falsified if:* the wrong instance survives, limit 0 drops one, or an
entry-limit rejection takes instance draws.

**C2. Channels.** Start 16 long *normal* instances, then one *low* and one *high*. *Expected:* the
*low* one is not heard; the *high* one cuts the quietest *normal* one. Repeat with 15 playing
channels and one channel waiting through a long pre-delay. At equal lowest priority the
pre-delay instance is replaced, not a playing instance; when the playing minimum is strictly
lower, a playing instance is replaced. Use identifiable entries and repeat with new requests
of equal and lower priority. Equal can replace, lower cannot; a refused request does not start
later when a channel becomes free. *Falsified if:* any priority or busy-class outcome differs.

**C3. Crowded-sound level.** Original/default full-volume quirk setting. Named test sound data:
16 distinct long entries of volume 100,
no shifts or delays, all requested at full level. Record one distinguishable sample alone, then
with the other 15. After the master level settles, *expected:* the master target is
⌊1638400 / (16 × 19)⌋ = 5389, against 16384 alone. Compare the output percentages through F3
and F4, not directly as an amplitude ratio. *Falsified if:* the isolated component's level stays
unchanged or settles at a different output-table value.

**D1. Response slot.** Order one unit to move three times quickly, then move-attack-move. Count
responses. Select another unit independently while the first speaks: the two may speak together. *Expected:* the same move line is not restarted while speaking; a different response is
heard after the current one ends, and never two from that object at once; two independently
selected objects can overlap. Use named response lists containing one distinguishable entry
per order kind, so repeating an order really requests the same entry. While the first line
plays, request two different later lines before it ends: only the last waits and plays.
*Falsified if:* the same line restarts, two lines from one object overlap, an overwritten line
plays, or independent objects are consistently serialised despite free channels.

**D2. Who speaks and the draws.** Select a band of 5 units of two types; give a move, an attack, a
deploy and a guard order. Record the unsynchronised stream position per tick. *Expected:* only the
first successfully selected unit speaks for the band selection; orders use the first unit in the
resulting selection order (claim 18), one raw value per response from a list (none for single responses), the
variant equals F5 for the recorded value, and the scenario stream does not move for it.
Repeat using the repair-response vehicle ordered to heal with a negative-damage weapon, with
and without its game-wide single response in named test rules. *Expected:* the single response
when present, otherwise the attack list; weapon numbers above 0 use the secondary response.
*Falsified if:* two units answer one click, the variant or fallback does not match F5/F6, or the
scenario stream advances.

For order fallbacks, named test rules give distinguishable single responses and one-entry
lists. Exercise move, attack, enter, capture, harvest, unload/deploy and an order without its
own kind. Remove each single response in turn, then empty its fallback list. Expect exactly
F6's fallback chain, silence when exhausted, and one raw value only for a nonempty list choice.
Give elite and normal primary/secondary attacks distinct responses; repeat against targets
selecting each weapon. Test two armed units selected in opposite orders with a non-armed unit
first and last: the latest positive-primary-damage type responds; with no such type, the
earliest selected responds. An empty list on the first responder does not hand speech to
another. *Falsified if:* any identity, order or choice draw differs.

**D3. Selection variants.** Select an enslaved miner, a type-select (all of a type) and click an
already selected unit. *Expected:* enslaved list (or silence), no response for type-select, a response
on re-click. Add a deactivated unit with a distinguishable deactivated list, then repeat
with each specialised list empty: expect the matching list, then silence without normal-list
fallback. Modifier-click a selected unit to remove it: expect no selection response or choice
draw. Recall a group: only its first newly selected object responds. Finally click a selectable
object of a passive house with a nonempty selection list. Expect one unsynchronised raw choice
value but silence and no scenario-stream choice. *Falsified if:* any differs.

**D4. Keyboard commands.** Guard, stop, scatter and deploy keys with a unit selected. *Expected:* the
three game-wide sounds, and silence for deploy (apart from the unit's own deploy response).
*Falsified if:* guard, stop or scatter requests the wrong entry or none, or the deploy key
plays a game-wide sound.

**E1. Draws inside an instance.** Named test files: entries with (a) no pitch or volume shift and no
words, (b) a pitch shift and a volume shift whose ranges are powers of two minus one (one raw value
each), (c) *random* over 4 samples, (d) *all* and *random* over 4 samples, (e) *attack* and *decay*
with 2 of each around 2 body samples, (f) *pre-delay* with a delay range. Each is played alone, as a
unit's selection response, with nothing else sounding. Record the unsynchronised stream position
per tick with the random-state recorder, and the order of samples heard. *Expected:* the advance per
instance equals the draw table (State and ordering) less the response draw itself: (a) none, (b)
two, (c) one, (d) one discarded R(0, 3), then audible-order draws R(0, 3), R(0, 2), R(0, 1),
(e) attack and decay choices plus any nontrivial body ordering, (f) one more. Each ranged draw
consumes the raw values, including rejected values, that S-0202 claim 4 predicts. In
(d) all four samples, each once. Also record pitch and level in part (b), and first-sample delay
in part (f): their values must match F3/F4 and the pitch, shift, pre-delay draw order, not merely the
total count. Repeat fixed delays of 32 and 33 ms with *pre-delay*, and an *ambient* fixture
whose second endpoint exceeds 33 ms: expect no explicit wait below 33, a wait from 33, and
ambient lower endpoint 33. Compare one and two endpoint import forms.
(g) Immediate *all* without *random*, and finite and indefinite loops: expect listed body order,
the imported number of body passes (0 indefinite), attack first and decay last. (h) Delayed
mode with four body samples, no attack or decay, and *loop*: with *random* alone the same chosen
sample repeats; with *all* every body sample occurs once per pass. E3 separately settles the
unresolved attack/decay choices, terminal waits and complete draw counts.
*Falsified if:* any tested count, identity, order, pitch, level or delay differs.

**E3. Settling delayed draw totals (Q-0260).** Named test sound data: two distinct attack
samples, four body samples and two decay samples, each short and identifiable; no pitch or
volume shifts. Compare fixed delay 64 ms with a delay range 64 … 127 ms, with and without
*pre-delay*, and with *all*, *random*, both and neither. Repeat without attack, without decay,
and with finite loop counts 1 and 2. In the quiet test map, trigger each entry once through
selection, allow it to end without scrolling or interruption, and record the unsynchronised
stream and stereo sample identities/times through one second after the last audible sample.
Run each fixture three times; subtract the known single response-list choice. Replay each
recorded stream mathematically using S-0202, including rejected raw values. A proposed
read-only recorder extension records instance end times to expose silent terminal waits.
*Established expectation:* fixed equal-ended waits consume no raw value; random-only body
selection is retained across loops; *all* has no body repetition within a pass. *Open alternatives:*
decay choices only for the final sample versus choices also discarded during earlier sample
loads, and delay draws only between audible samples versus additional terminal waits. Count
and identify draws against each alternative; a mismatch falsifies that candidate sequence.
If several candidates remain indistinguishable, vary the sample and delay ranges separately
and report the total as unresolved rather than assuming immediate-mode counts. This is a
settling scenario, not a claim that the full delayed count has already been established.

**E2. Audio cadence.** Named test sound data with a short identifiable sample, no shifts,
no pre-delay and no random sample selection. In a quiet small map, use scripted clicks at varied
real-time phases, recording input timestamps and stereo sample starts for 100 requests at each
of game-speed indices 0, 2 and 6. A proposed read-only recorder extension also records audio-pass
millisecond timestamps; it changes no state or scheduling. *Expected:* passes are separated
by more than 33 ms, possibly much longer at slow speed or under host load, and fresh requests
obtain channels at those opportunities. Several starts may belong to one pass; loopback
buffering can shift audible timestamps. There is no 33 ms maximum latency guarantee.
*Falsified if:* recorded passes are 33 ms or less apart, or a fresh request obtains a channel
between passes. Without that recorder extension the exact threshold and allocation timing
are inconclusive, not established by audible start times alone.

**F1. Damage and death.** Hit a local unit with a feedback list 200 times across its yellow
threshold over many seeds; record the share that speak (*expected:* about 30 %) and the
unsynchronised draws per hit (*expected:* one R(0, 99) on now yellow for any owner, one more raw
value when spoken). A damage-sound hit: two instances. A death: death response (local only) then
death sound. *Falsified if:* the share is far from 30 %, the chance is drawn on other results, or a
single damage-sound instance. Repeat the threshold sequence on enemy-owned test vehicles,
with feedback and death lists referring to distinguishable entries that have no instance draws:
expect the feedback chance draw but no spoken feedback, and a death-sound choice without a
death-response choice. Local deaths choose the response before the death sound. Include
*unaffected*, *now red* and a berserk hit: expect no common reaction. Named test data provides
a sufficiently high per-entry limit to distinguish the doubled damage sample in loopback
capture. For the 200 local yellow crossings, use an exact binomial test at significance 0.01
against probability 0.30; a rejection falsifies the chance hypothesis, not a small deviation.
F9 separately settles infantry multiplicity; this scenario makes no exactly-once prediction.

**F9. Settling infantry death multiplicity (Q-0264).** Named test rules give a local infantry
type two distinguishable death-response entries and two different death-sound entries. Named
test sound data makes each entry one sample, with no random controls, pitch or volume shifts,
no delay, high per-entry limits and enough free channels. Make explosion animations silent.
On the isolated test map, kill one such infantry by an ordinary lethal hit, then repeat with
the same type owned by the bystander. Use a comparable vehicle as the common-reaction control.
Record stereo playback and both random streams over the killing tick and later corpse events.
Run three repetitions. *Alternatives:* one pair requests one response and one death sound,
with two unsynchronised raw choices for a local infantry and one for enemy infantry; two pairs
request each twice, with four and two choices respectively. Either count falsifies the other
candidate, even if identical samples overlap or one request is suppressed. Record ordering
and variants, not just loudness. Then repeat crushing, a berserk-causing lethal hit and a
special infantry death mode separately, using their owning specs' setups; differing counts
establish a path-dependent rule, not a universal exactly-once result. Any unrelated random
choice that cannot be isolated makes that run inconclusive. The scenario settles the current
open question; it does not yet select one candidate as the requirement.

**F2. Report variant.** Build 20 copies of a unit whose weapon has several report sounds; each copy
fires many times. *Expected:* each copy always uses one variant, and each variant equals F5 applied to the recorded creation draw's low 16 bits. Use the
random-state recorder, not an injected seed; a repeat from the same saved game should keep each
existing object's variant. *Falsified if:* a copy
changes variant, or the variant differs on replay. Repeat with named report lists whose
entries contain no instance draws, using a rapid-fire spin-up unit and infantry firing from an
occupied building: expect a fresh unsynchronised raw list choice per spin-up step and per
occupant shot, respectively. The ordinary report must add none; its variant still matches
the creation value. *Falsified if:* either exception uses a fixed creation variant or takes
its report choice from the scenario stream.

**F3. Impact.** A weapon whose warhead creates an explosion animation with a start sound, fired at
ground. Use named test art to give the animation distinguishable start and stop samples, with
an otherwise silent weapon. *Expected:* the start sample at the animation's start event and
the stop sample at its end, placed at the animation. Repeat with both animation sounds absent:
expect no impact sample. *Falsified if:* a sound with no animation sound configured, a missing
start/stop sample, or a screen-centred sound independent of animation position.

**F4. Event sounds.** Produce a unit (creation sound), place a building (placement sound, screen),
sell a building (sell and pack-up sounds), sell an ordinary wall (screen sell sound), move a
vehicle and stop it. Record stationary update ticks and the end of the final looping sample,
using named test sound data with short identifiable loop samples. *Expected:* the movement
grace counter lasts three stationary updates and
the next eligible update asks the sound to finish its loop; its current sample may continue.
Repeat out of range and return without stopping to test claim 7's loop restart, not a new
movement-list pick. *Falsified if:* any tested sound is missing, the wall is silent, or stopping
always cuts the current movement sample immediately, or the end-loop timing differs from
three stationary updates followed by the next eligible update. Use a vehicle with a distinguishable
movement list and record the unsynchronised choice when its episode begins out of range:
expect one choice, not a second choice solely on returning to audible range. Turn a turret
without moving the body, then stop the turn: expect its attached rotation sound while turning.
Repeat building placement and ordinary wall sale after scrolling: their screen sounds retain
level and centre pan. Packing a building up omits the sell sound but retains the pack-up sound.

**F5. Interface sounds.** Exercise each production-feedback click of S-0411 claim 12 (start,
queue, resume, hold, cancel) and the busy-line refusal (a building click while a building is in
production), then a refused placement click, then a production request when the queue is full.
Space actions until all earlier announcements have finished and their waiting queues are empty.
Use distinguishable named test samples for the build-click and refusal entries and the two
refusal announcements; capture requested entries as well as playback with a proposed read-only
recorder. A suppressed announcement is not evidence of an absent request.
*Expected:* the build click entry plus the matching announcement for the five production clicks,
only the "unable to comply" announcement for the busy line; only "cannot deploy here" for the
refused placement; the refusal entry for the full queue. Repeat as each side.
*Falsified if:* a listed cue is absent, an interface sound accompanies the busy-line or placement
refusal, or a cue depends on side. A click on an item at its build limit: expect no click sound
and no announcement. Repeat placement with an order that passes the click check but fails its
cell test at application, using S-0406 PL1's controlled failure setup and order recorder.
Expect the same placement announcement request, no interface-sound request and a retained
ready item. A different entry, no announcement request, or a game-wide refusal-sound request
falsifies the cue rule. Use S-0406 claim 1's no-eligible-factory control separately: expect no
sound or announcement request, not the placement-refusal cue. Void the cell-failure run if
no order is issued or it is ignored for lack of a factory. For full-queue and build-limit
refusal at application, use S-0404 P3(k)'s issue/application recording and void rules.
Do not substitute an at-limit click that issues no order.
Actions that S-0411 leaves unresolved are recorded, not assigned a prediction.

**F6. Lightning sound.** Fire the weather superweapon; record the scenario stream per tick.
*Expected:* for each non-repeated strike point with a nonempty sound list, one raw value after
the bolt animation's creation draws and before explosion creation and damage, whose modulo
selects the audible entry. A repeated point has no sound draw. Account for animation draws using
S-0109 and damage effects using S-0302; replay the logged seed's mathematical stream rather than
setting a seed inside the original. *Falsified if:* the choice or its position differs.

**F7. Building fallbacks.** Damage a local building whose type names no damage sound through the
yellow and then the red threshold, then destroy one whose type has an empty death list. Then
repeat with named test rules giving the type its own damage sound and death list.
*Expected:* without own sounds, the game-wide building damage sound at each threshold crossing
and the game-wide building death sound at destruction, positional at the building and following
scrolling when the view moves during the sample. With own sounds, neither fallback plays.
Claim 20 does not promise the type's own damage sound at a threshold crossing: add a separate
*unchanged* hit to hear its doubled damage sound. Destruction uses the type's death list.
*Falsified if:* an expected fallback is absent, a fallback plays with an own sound present,
the unchanged-hit or death choices differ from claim 20, or a fallback is heard at screen level.

**F8. Repeated route-mode refusal.** Exceed the route-mode limit of S-0213 claim 16 twice,
some seconds apart, in one running game; log completed ticks between the clicks. Then start
a new game and exceed it again. Record messages and the refusal entry separately from
announcement playback. *Expected:* message and refusal sound on all three refusals.
*Falsified if:* the later refusal after completed ticks is silent or lacks its message, or the
new game's refusal gives neither cue. This does not test multiple notices before one tick
completes; that coalescing rule remains unmeasured.

**G1. Announcement data and voice set.** Play the same short game as each side's country.
*Expected:* the announcement voice set of the local player's side throughout. Edit an announcement's
volume to 0 in a private copy. *Expected:* still heard at full level. *Falsified if:* the voice
changes in-game or the volume edit is heard.

Repeat with named announcement test data omitting kind and priority, then with unknown or
empty words, and with duplicate names: expect the defaults for new entries and retained
kind/priority on re-read (claim 26), judged by G2's request sequences. Name test samples whose
first eight characters distinguish them; supply a longer file name and expect the first-eight
version to play. With silent unrelated sound lists, record both random streams around queued
announcement requests and starts: expect no announcement-choice draw. *Falsified if:* a
truncated name fails when its corresponding sample exists, defaults/re-read change queue
behaviour, or an announcement adds a random choice.

**G2. Announcement queue.** Named announcement test data gives distinct normal game events
(for example low power, construction complete, unit ready and sidebar feedback) short identifiable
samples and the kinds/priorities needed for each test. While a long current line plays, use
scripted input to request, in deliberately reversed priority order, a queued-interrupt line,
a critical standard line, a normal standard line and queued lines at each priority. *Observable:*
stereo sample identity and start milliseconds. *Expected:* no overlap, more than 500 ms between
ordinary starts, and the precedence of claim 27. Repeat requests and separately request an
interrupt: expect no duplicate of the same pending kind, and an interrupt cutting the current
line and clearing pending lines. Repeat with no current line to check that an interrupt does not
clear pending requests solely because of its kind. *Falsified if:* overlap, retained duplicates,
the wrong precedence or the wrong flushing condition.

**G3. Announcement throttles.** (a) Produce units continuously from one factory: *expected* one
unit-ready announcement, then none for the same factory until its radar event has gone (zoom-in plus
200 ticks), while a factory 3 or more cells away announces. (b) Hold credits below 100 with a factory:
low funds repeats at the normalised speak delay while funds remain low; falling below the
threshold alone does not override an unexpired countdown. Repeat part (b) at indices 0, 2 and 6
and compare the tick intervals with N, rather than real-time intervals.
(c) Attack two of the local buildings 10 cells apart: two base-under-attack announcements; 5 cells
apart: one. Record the attack-warning sound too, then repeat with the attacked building off-screen:
that sound remains centred at its screen-request level. Space requests so announcement queue
suppression cannot masquerade as radar-event suppression.
(d) Repeat (a) before the local player has a radar building: record whether it repeats and when.
This resolves the open radar dependence; indefinite suppression is not itself a falsifier of
an already-established zoom duration.
(e) Sustain low power while owning a building of the first three imported construction types:
expect one warning; restore power and lower it again, expect a second. Remove every qualifying
building and repeat: expect none. (f) Damage two local harvesters at separations inside and
outside F7's radius, without killing them: expect one warning inside, two outside. Repeat
attacks after the first event expires: another warning. For produced units, base attacks and
harvester attacks compare separations exactly at the radius with one cell less, and record
expiry relative to the observed end of zoom-in: rejection is strict and the tail is F7's ticks.
If zoom completion cannot be identified, record total duration as inconclusive for the exact tail.
*Falsified if:* any resolved repeat interval, eligibility condition, distance boundary or low-power
reset differs. Claims about other F7 kinds are not tested by these parts.

**G4. Announcement entry bindings.** Named announcement test data gives each fixed entry
in Import mapping a distinct brief sample for every voice set. Use the mapped names, not
roles inferred from baseline recordings, so this test can falsify the binding itself. Raise start, queue, resume, hold and
cancel clicks, a busy building-line click, refused placement, construction completion, a new
sidebar option, unit ready, building and unit sales, completed repair, promotion to each rank,
low funds, low power, and base, ally and harvester attacks. Space events beyond F7 suppression
and let each line finish. Use separate runs for local victory, local defeat and another
non-passive player's defeat, keeping an extra bystander alive for the last test; the intentional
defeat is exempt from the general bystander void rule. Record stereo sample identity for each
event. *Expected:* the corresponding role's sample, including one shared promotion entry at
both ranks. Each event must play the sample assigned to its mapped entry. Also request
repair at a depot with zero available credits: expect the same low-funds entry as the house
countdown. Repeat as two countries of the same side and a country of each other side:
entry identity stays fixed, while the sample follows the local side's voice-set field. Run a small test map whose trigger plays a test entry of kind *queued* by name
while another announcement plays, and one naming an unknown entry. Expect the named entry
to cut the current one as an *interrupt*, and silence for the unknown name.
*Falsified if:* an event produces a sample from a different mapped entry, promotions use
different entries, country alone changes voice set within one side, the trigger queues
rather than interrupts, or an unknown name produces a line.
Alliance conditions and the observer exception remain unmeasured here; S-0414 V20 tests
post-defeat muting within the original's end delay, with conditional alliance controls.
For a settling
alliance test, use separately timed one-way alliance, reciprocal alliance and alliance-break
trigger actions involving the local house and a bystander, then two non-allied bystanders.
Record requested entry identities, not only audible starts. Expect the entries for requesting,
requested, formed, enemy-formed and broken under claim 36's conditions. An entry outside that
mapping falsifies its binding. If the permitted trigger subset cannot generate a needed
request, that branch stays inconclusive rather than being replaced by a different event.

**H1. Sound does not reach the game.** Two continuations of the same saved game, with the same
orders at the same ticks: one with effects off, one at full with the view moved differently.
Use scripted input and record the scenario stream and game-state observables; do not set the
original's seed. *Expected:* identical scenario stream,
identical game state per tick. The unsynchronised stream may differ: placement, failed or repeated playback attempts and
per-instance draws follow presentation timing. This comparison must not assume that only the
per-instance draws differ. *Falsified if:* any scenario-stream or state difference.

**I1. Volume options.** Set the sound volume option to zero and the voice option to full: expect
announcements at full and no sound instances, responses included; then the reverse: expect
sounds and no announcements. With C3's sixteen loud entries playing, expect an announcement's
level unchanged while sounds are lowered. Named test sound data gives the credit-tick entry and a
selection response entry the same sample at volume 100, with no shifts: expect the credit ticks
10 dB below the response at the same option setting. *Falsified if:* an option scales the other
group, the crowded level lowers an announcement, or the credit-tick difference is not 10 dB
within one F4 step.

## Open questions

- Whether an infantry death plays its death response and death sound once or twice (two copies of
  the death handling exist; claim 20, low).
- The type flag that exempts some types from the unit lost announcement and the one that exempts some
  buildings from base under attack (claim 28).
- How long the zoom-in phase of a radar event lasts, whether it advances with ticks or drawn frames,
  and whether it advances at all while the local player has no radar (F7, scenario G3 d); it adds
  to every suppression window.
- Which copy of a sample the bank plays when its index lists the name twice (claim 34).
- The sidebar event that raises the second "unable to comply" request following a build click
  (claim 24).
- The death fallback of claim 37 is traced on one destruction path; other building removal paths
  (sale, crushing, scenario removal) have not been checked for it.
- Absent-key defaults of the game-wide rules sound references and the speak-delay field have
  not been traced in this verification. Their listed import forms and uses are established,
  but a missing game-wide key must not be assigned a guessed default. This part is `unmeasured`,
  low confidence, until constructor and earlier-layer values are established.
- Negative or oversized sample partition counts, negative delays, and negative limits/ranges:
  their complete failure behaviour is not established. Claim 14's regular playback sequence
  assumes nonnegative attack/decay counts with at least one body sample, nonnegative delay
  endpoints and loop count. The no-body quirk is the traced exception, not a general bad-data rule.
- Complete delayed-mode attack/decay sequencing, terminal waits and draw counts (claim 14).
- Full classification of special weapon report choices (claim 21).
- The building deploy acknowledgement and the condition that suppresses one deploy response.
- Exact fractional level smoothing and complete gain-chain truncation order (F3).

## Changes for other specs

These hand-off checks include corrections already present in other drafts; they are not
instructions to undo those corrections. S-0214's current claims 12 and 14 and scenarios T1/T2
already agree with claims 17/18 here. S-0411's current claim 12 agrees with claim 24 and F5:
busy-building refusal and refused placement use distinct announcements and neither uses
the game-wide refusal sound. Its current draft has removed the stale correction assertion
and clarified the corresponding table and scenario wording. Hand off that corrected version,
not the approved copy's stale assertion. No cue is added for its unresolved actions.
The approved S-0406 still contradicts claim 24: its claims 1 and 10, State and ordering,
and refusal measurements describe the game-wide refusal sound for failed placement.
S-0406's current draft corrects these to the cannot-deploy-here announcement request only,
without the game-wide refusal sound, including click-time and application-time failure.
Hand off that correction jointly with this revision; do not retain the approved wording.
An ignored place order without an eligible factory remains silent.
Production/payment, offer membership/capacity and cancellation rules remain owned by
S-0404, S-0407, S-0004 and S-0411, not by this audio revision.

Other existing hand-off checks:

- S-0214 claim 14, F5 and Import mapping: the otherwise-used order response is the
  special-attack list, not the selection list. Add claim 18's selection ordering and claim 17's
  enslaved/deactivated selection rules without normal-list fallback.
- S-0213: use that same selection order wherever it orders a selection; S-0214 owns its
  definition, including the resulting representative tie-break.
- S-0202: qualify its sound-variant stream classification with claim 25's lightning exception,
  and include claims 13/14's instance draws and claim 20's feedback chance among
  unsynchronised choices. Lightning's raw value follows bolt-animation creation and precedes
  explosion creation, except for an empty sound list or a repeated strike point.
- S-0411: resolve its refusal-entry question and replace planned sound pointers with claims
  24/28. Revision 2 corrects the earlier hand-off: the busy-line refusal of claim 12 and the
  refused placement of claim 10 play announcements only ("unable to comply", "cannot deploy
  here"), not the refusal sound. S-0701 does not add cues to sidebar actions that S-0411 leaves
  unresolved.
- S-0213 claim 16 and O8 already correct the earlier once-per-game route cue assertion:
  a completed tick permits another message-and-sound cue. Claim 24 and F8 here now agree.
  No once-per-game quirk remains. S-0213 revision 17's hand-off note already confirms
  that agreement; confirm both current passages at sign-off.
- S-0406: replace failed-placement refusal-sound wording in claims 1 and 10, State and
  ordering and the associated scenarios with the cannot-deploy-here announcement request
  only, with no game-wide refusal sound. Its current draft already makes this correction.
  Confirm that corrected version at joint hand-off. Keep no-eligible-factory orders silent
  and distinguish announcement requests from queue-dependent playback (claim 27).
- S-0404: a request refused as it is applied (full queue or build limit) plays the refusal sound
  for the local player's house (claim 24).
- S-0414 already agrees on post-defeat alliance-formed and superweapon announcement
  muting, and V20 supplies its end-delay test. Confirm claim 36's defeat, player-defeated
  and victory bindings at hand-off; muting does not end continued observation. Replace
  S-0414's stale request to restore withheld bindings with confirmation that this Import
  mapping already supplies them.
- S-0005 claim 10a and M25: confirm independent bank-file precedence against claim 34
  and its M25(b) backing. Claim 10 owns expansion audio archive preference. The public
  bank formats need no new spec.
- S-0416: half-volume credit ticks are request level 8192 (claim 35).
- S-0314 and S-0410: point repeat-suppression questions to F7, retaining the stated unmeasured
  kinds and unresolved zoom duration rather than treating all lifetimes as measured.
- glossary: revision 20 already records claim 28's low-funds countdown as presentation
  timing, not a new game-state timing exception. No further glossary change is needed.

## Questions for the project lead

None for this revision. Claim 30 retains S-0202 claim 11's existing shared-stream contract.
The former separate-generator proposal is withdrawn, not adopted as a departure.
Sound feedback remains in the existing *ui* topic. Announcement entry names and the
repair-response type name remain only in Import mapping, not new configurable fields.
