---
id: S-0701
title: Sound selection
topic: ui
revision: 36
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0260, Q-0264, Q-0265, Q-0261, Q-0262, Q-0266, Q-0325, Q-0327, Q-0329, Q-0326, Q-0328, Q-0330, Q-0332, Q-0333, Q-0334, Q-0335, Q-0336, Q-0337, Q-0338, Q-0340, Q-0671, Q-1735]
supersedes: []
---

# S-0701: Sound selection

## Summary

Sound data, selection and order responses, event sounds, announcements, mixing limits, placement and
traced random choices in skirmish; network games and campaigns are out of scope. Playback is
presentation, but claim 33's traced scenario-stream choices must retain their draws. Music belongs
to S-0703; claim 30's audio passes and music-only passes carry its music updates. Archive mounting
belongs to S-0005 claim 10. Which files hold the samples is claim 34; the byte formats of those
files are outside this spec. Claim 21 aligns spin-sound choices and ordinary report suppression with
S-0609 claim 13.

**Coverage boundary.** This is a partial sound specification, not an exhaustive event catalogue.
Claim 40 indexes every question of Q-0260, Q-0264 and Q-0265 against the established answers
and explicitly unmeasured remainder; listing those requests does not close that remainder.
The following are deferred, with no conformance rule or implied silence: complete delayed-mode
attack/decay playback and draw totals; total infantry death-sound multiplicity across specialised
death reactions; specialised movement-state and building-deploy response gates; alliance
announcement selection; specialised superweapon activation bindings not listed here; and
announcement eligibility for ownership-change causes outside claim 39 (trigger and occupation
changes, and passive new owners). The retained delayed-mode rules apply only to the explicitly described choices, not a
complete replacement sequence. Q-0325 and Q-0329 remain partial answers. E3, F9, G5 and G6
include exploratory work for deferred coverage; those parts are not backing for additional
claims. Feature specs retain their independently stated rules. A future revision must settle
these gaps before an implementation can claim complete audio coverage. Claim 26's caption
setting is our engine's design, an accessibility addition that answers Q-1735; it is not the
original's behaviour, which, as far as examined, shows no text for an announcement (claim 26:
`measurable` for part T's settings, `unmeasured` beyond them).

**Timing boundary.** Audio-clock durations and service opportunities below describe the
original. They do not authorise host-dependent choices in our engine. The glossary's
*departure* rule remains authoritative: our engine must use deterministic presentation
scheduling. Claim 30 supplies our engine's replacement clock, waits, sample-completion opportunities
and draw interleaving as design rules, not oracle findings. Claim 14's separately deferred
complete delayed-mode sequence remains outside coverage; settling its scheduling does not
supply that missing sequence. Music time in the options menu is the one place where this
spec's sound and music presentation follows real time; the project lead has accepted it, so
that the music plays on in the menu as in the original (claim 30, S-0703 claim 13). It reaches
no game state.

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
- **Audio clock.** The original's real time in milliseconds. Durations stated in milliseconds
  in claims 9 to 14 and 27 use this clock, not ticks. Their thresholds are independent of
  game speed, although actual service opportunities need not be (claim 12). Our engine's
  deterministic replacement is the logical audio time of claim 30.
- **Logical audio time.** Our engine's nonnegative integer millisecond count, advanced only
  by completed skirmish cycles, not by the host or audio device (claim 30).
- **Audio pass.** One service of the sound instances. In the original successive passes are
  separated by more than 33 ms of the audio clock (claim 12). Our engine uses claim 30.
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
- **Needs-engineer flag.** S-0405 claim 8's building-type boolean, also used to distinguish
  claim 39's custom capture/tech-building-loss announcement branch from ordinary capture.
  This spec uses it only to select an announcement, not to define capture legality.
- **Vehicle-like takeover.** The takeover of a vehicle-like building (S-0214 Terms) by an infantry
  unit under S-0509 claim 17. S-0509 claim 12 lists it as requesting a capture announcement.
- **Mind-control takeover and release.** The taking of a building by a mind controller (S-0601
  claim 6) and its release back to its original owner (S-0601 claim 9).

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
     requests. The playback effect of the remaining recognised words, including *quiet* and *loud*,
     is outside this spec's coverage,
   - *limit*: an integer, the most instances of this entry that play at once; 0 means no limit
     (claim 8),
   - *loop count*: an integer, default 0 (claim 14),
   - *range*: an integer number of cells (claim 6),
   - *delay*: one or two signed decimal integers *a* [*b*] in milliseconds, separated only by
     spaces, tabs or newlines; *b* = *a* when only one token is given, default 0 0
     (claims 13 and 14),
   - *pitch shift*: the same two-token form, in percent, default 0 0 (claim 13).

   These two fields read each token's leading signed decimal digits, ignoring trailing text.
   a token without leading digits gives 0. Commas are not separators. Consequently the values
   `40,80` and `40, 80` give (40, 40) and (40, 80) respectively, `40 x 80` gives (40, 0),
   `10 20 30` gives (10, 20), and `40ms 80` gives (40, 80). Tokens after the second are ignored.

   A game-wide defaults section supplies the volume, minimum level, limit, range, control set and
   placement set of every entry that does not give its own. Without that section, or without the
   key in it, the defaults are those of Constants. When an entry gives at least one nonempty control token, even an unknown one,
   its control set is exactly its own words, not added to the default set; the same holds for
   placement words, which start from *screen*. Keys are matched by exact case (S-0001, claim 13), so
   a key spelled with other capitals is ignored (Import mapping lists the stock cases).
   - Backing: `measurable` (A1, A2, B2, C1 and E1) for the tested fields, active words and
     placement-word cancellation; E4 tests the endpoint token forms. No no-effect claim is made for the remaining placement words.
     F1's exact integer at full volume is `unmeasured` for A2's output-resolution reason.
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
   including the damage sound, retain their map position but are placed again at audio
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
   retried. Our engine provides 16 channels always; host sound-device allocation failures are
   outside the oracle coverage here.
   - Backing: `measurable` (scenario C2) for priority, quietest-playing replacement and
     playing versus pre-delay precedence. The exact fractional level tie threshold and channel-order ties are `unmeasured`:
     C2 cannot identify channel order, and loopback levels cannot resolve the sub-unit threshold.
     The guarantee of 16 channels in our engine is a design requirement, not an oracle claim.
   - Confidence: high (rules); medium (the order among equally loud channels)

10. **Instance pool.** At most 300 sound instances exist at once (playing, waiting or between
    samples) and a separate limit of 200 applies to positional requests whose placement follows scrolling
    at their fixed map position (claim 7), not to all positional sounds. A request when the
    pool is full, after dead instances are cleared, plays nothing.
    - Backing: `measurable` (C4's simultaneous-request tests, not ordinary-play saturation)
    - Confidence: medium

11. **No shared response lock is traced.** Responses of different objects can overlap, subject
    to the limits of claims 8 to 10 and each object's slot of claim 16. This does not establish
    that no specialised event has an additional throttle.
    - Backing: `measurable` (scenario D1, two independently selected objects)
    - Confidence: high (overlap); medium (absence of a general response throttle)

12. **Original audio-pass cadence.** In the original, a pass is eligible only when more than 33 ms have
    elapsed since the preceding pass. It runs when the main loop next services audio; 33 ms is
    therefore a minimum interval, not a maximum request latency. Channel allocation and per-entry
    ranking wait for a pass. Sample completion and object-attached placement have other update
    opportunities (claim 7). Several ticks may pass between passes at fast game speeds.
    This is an oracle timing rule, not permission to use host timing for our engine's choices
    (claim 30).
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
      indices are consumed without repetition within a pass. The first sample's explicit wait
      is claim 13's pre-delay; delayed mode alone does not add that initial wait. Between
      sample completions, delay choices use a fresh R(*a*, *b*) in milliseconds from the
      unsynchronised stream, not one delay reused for the whole instance. Without either
      *pre-delay* or *ambient*, completion requests that delay even when no next sample remains.
      With either word, whether a terminal delay is requested also depends on remaining samples.
      The completion delay uses the second endpoint's 33-ms threshold; a chosen delay below
      33 ms is no wait. These established delay choices rule out a universal no-wait substitute.
      With decay enabled, a decay candidate R(*n* − *D*, *n* − 1) is chosen before the next
      attack/body selection at each sample preparation, including preparations before the
      decay is audible. A first-pass attack selection, when present, is R(0, *A* − 1).
      Random-only body selection follows it. Those choices follow claim 13's instance draws.
      Do not move all decay choices to the end as in immediate mode.
      The complete audible attack/decay sequence, repeated-preparation count, terminal wait
      duration and full draw ordering are outside this spec's coverage. E3 explores these
      deferred parts of Q-0329 and Q-0260. No complete draw total, absence of startup choices
      or immediate-mode replacement follows from these partial rules.
    All traced draws use the unsynchronised stream, after claim 13's draws. In particular an
    immediate instance chooses its attack sample, its body sample when applicable, and its decay
    sample before drawing the body's audible order.
    - Backing: `measurable` (E1 for immediate mode and delayed body choices; E3 for the
      initial wait, completion-delay choices and decay-before-attack/body choice order).
      E3's exploratory parts do not establish the deferred complete sequence.
    - Confidence: high (immediate mode and the stated delayed choice ranges/order)

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
    first object that becomes selected speaks. A tap of the type-select role (S-0214 claim 32) is
    such an input: it gives one selection response, that of the first object it adds, and none if it
    adds nothing. With the type-select role held, its clicks and drags (S-0214 claims 29 and 32)
    give no response at all. The tap's single response is our engine's rule (S-0214 claim 32): the
    original instead lets every object a tap adds queue a selection response of its own. Responses
    are presentation only, so no switch. A plain click on an object deselects everything first, so
    clicking an already selected object makes it speak again; a modifier-click that removes an
    object from the selection gives no response. Selecting an object of a passive house takes the
    raw value but gives no response. An ordinary other house's object, enemy or allied, takes no
    selection-response draw and gives no response. An empty applicable list takes no draw for any
    owner. These rules also apply to individually selectable buildings; group-selectability does not
    change the owner test.
    - Backing: `measurable` (scenarios D2, D3). The tap's single response in our engine is a
      design rule (S-0214 claim 32), not an oracle claim. The original's per-object responses
      to a tap are `measurable` by D3, which records each response heard.
    - Confidence: high (draw, first-object rule, the held type-select role's silence, the tap's
      single response in our engine); medium (which objects count as slaves and as deactivated,
      band-box order, re-click, the original's per-object responses to a tap, as S-0214 claim 32
      rates them)

18. **Orders.** For ordinary immediate battlefield clicks (not the planning-mode path), only the first
    object in selection order can respond. If that object receives no order from the click,
    no object responds (S-0214 claim 14).
    Selection order: an object whose type's primary weapon does positive damage is put at the
    front of the selection when it is selected, and any other object at the end. So the object
    that responds is the most recently selected object of a positive-primary-damage type if the
    selection holds one, and the earliest selected object otherwise. This is also the order
    in which the click issues orders, not a separate speech-only ordering (S-0214 claims 11/12,
    S-0213 claims 2/3).
    Its response depends on the order kind (Formulas F6): move and attack-move use the move
    list; attack uses a weapon-specific single response when the type has one for the weapon it
    would use against that target and its veterancy (primary or secondary, normal or elite), and
    the attack list otherwise; enter, capture and harvest use their single responses, each falling
    back as F6 shows; unload and deploy use the deploy or undeploy single response; every other
    order kind uses the special-attack list. A list is chosen from by one raw value of the
    unsynchronised random stream (F5); a single response takes no draw. An empty list or a missing
    single response with no fallback means silence; no other object responds instead.
    - Backing: `measurable` (scenario D2; S-0214 T7 for a first object receiving no order)
    - Confidence: high (move, attack, special-attack list, draws, selection order); medium (enter,
      capture, harvest, deploy fallbacks)

19. **Keyboard commands.** The guard, stop and scatter commands play their game-wide interface
    sounds as screen sounds instead of a response. The deploy command has a game-wide sound that the
    original never reads from the rules, so it is always silent (quirk
    `quirks.deploy_command_silent`). Outside planning mode, guard, stop and scatter request
    one interface sound per key press after processing a nonempty selection, even if none of
    its objects qualifies for an order. An empty selection requests none. If order issuance
    fails during processing, the command exits without that sound, including after earlier
    objects have been ordered. Guard's planning-mode path does not request this ordinary cue.
    These are command sounds, independent of how many objects were ordered.
    - Backing: `measurable` (scenario D4)
    - Confidence: high

### Event sounds

20. **Damage and death.** The common sound reactions to a hit on an infantry unit, vehicle,
    aircraft or building follow its damage-state result (S-0302). A berserk-causing hit has no
    such reactions (S-0302 claim 10). This does not override kind-specific death reactions:
    - *unchanged* (or any result beyond *now dead*): the type's damage sound plays at the object,
      positional, any owner. Request exactly one instance per damage-sound event, subject to
      the ordinary placement, entry-limit and channel rules of claims 6 to 9,
    - *now yellow*: one R(0, 99) is drawn from the unsynchronised random stream for every object
      whose type has a feedback list, whoever owns it; if the result is below 30 and the owner is
      the local player, one feedback response is chosen by a raw value (F5) and played at the
      object, positional (not through the response slot). So a 30 % chance, only on the hit that
      turns the object yellow,
    - *unaffected* or *now red*: nothing,
    - *now dead*: if the owner is the local player and the type has a death response list, one is
      chosen (F5) and played at the object; then, for every owner, if the type has a death sound list
      one is chosen (F5) and played at the object. Both positional.
    The same death-response and death-sound choices also occur when a recycling building
    consumes an entrant (S-0603 claim 7), at the entrant's position, with the same owner
    eligibility, response-before-sound order and unsynchronised stream. Consumption is a
    non-death occasion for these sounds, not an additional reaction to a lethal hit.
    Buildings also have the game-wide building damage and death sounds of claim 23.
    - Backing: `measurable` (scenario F1) for the original's reaction eligibility, feedback
      and death choices; S-0603 claim 7 and its scenario G10 for consumption choices.
      The single damage-sound instance is a lead-approved departure,
      tested against this spec in F1, not against the original.
    - Confidence: high (common reactions)
    The death rule describes one common reaction, not a total per infantry death. Counting
    additional kind-specific reactions is deferred under Q-0264; F9 is exploratory and
    supplies no exactly-once requirement.

21. **Weapon report: one fixed variant per object.** When an object fires a weapon that has a list
    of report sounds, the report plays at the firing position, positional, any owner. The entry
    used is fixed for the object's whole life: entry *r* mod *n*, where *r* is the low 16 bits of
    the object's creation draw (S-0202, claim 10) and *n* the list's length. Firing takes no draw
    for the report. Our engine retains *r* as persistent simulation state for pre-placed,
    starting, newly created and restored objects, including objects created before presentation
    begins; claim 32 gives the save/hash contract. This is an engine requirement, not an
    additional draw or a rule that gameplay reads the sound variant.
    A gattling-flagged type (S-0609, Terms) never plays this ordinary per-shot report.
    Its spin sound follows S-0609 claim 13: a spin-up update chooses a sound only when
    the playing mark is clear or the stage rises, not at every spin-up update or shot.
    The choice uses the report list of the weapon in slot 2*s*, where *s* is the stage
    after the update, after elite replacement. This is the ground slot even against an
    airborne target. A nonempty list takes one raw value of the unsynchronised stream
    (F5); an empty list takes none and leaves the mark clear. S-0609 claim 13 owns
    the playing mark's lifecycle and the attached positional sound's start and end rules.
    A shot fired by occupants from inside a building instead chooses its report afresh
    per shot, taking one unsynchronised raw value (F5). These rules cover ordinary firing, spin sounds and occupant fire only; they do not
    classify other special weapon effects. S-0615 claims 12, 15, 16 and 18 already specify
    the disc laser's absence of a shot-time report, separate charge cue, final-step report
    and creation-derived variant; those are not deferred here.
    - Backing: `measurable` (F2 for normal firing and occupant choices,
      S-0609 claim 13 and its G10 for spin-sound choices and per-shot suppression).
    - Confidence: high (fixed variant, no draw, spin-sound choice triggers and slot,
      gattling per-shot suppression); medium (occupant exception)

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
      not to stop its current sample immediately. This rule covers ordinary ground movement. Falling, crashing and carried-unit states
      are outside this spec's movement-sound coverage.
    - *Turret:* a turreted type's rotation sound plays as an attached sound while the turret turns.
      At the object's tick sound update, a rotation sample indicating no turn asks the instance
      to end looping, with no grace counter and without cutting the current sample. A later
      sample indicating a turn starts a fresh instance without cutting the still-finishing old
      instance, which can overlap it.
      Successive turning samples keep the existing instance. A turn entirely between rotation
      samples gives no request; a turn sampled as turning can give a request even if it has
      finished by the later sound update. Rotation sampling itself belongs to S-0206.
    - *Unit ready:* a new infantry, vehicle or aircraft that leaves its factory plays its type's
      creation sound, or the game-wide creation sound of its kind (infantry, vehicle, aircraft)
      when the type has none. This claim covers local production only.
    - *Building placement:* when the local player's house places a building, the game-wide
      building placement sound plays as a screen sound.
    - *Sale:* an own ordinary building's sell and pack-up sounds are positional at the
      build-down-start step of S-0408 claim 6. Packing up omits the sell sound (S-0415).
      An ordinary wall sale by the local player plays the sell sound as a screen sound.
      Only a silent wall sale omits it (S-0408 claim 12).
    - *Building damage and death:* a building's own damage and death sounds (claim 20), or the
      game-wide fallbacks of claim 37 (S-0302 claim 12 owns when the damage results occur).
    - *Promotion:* the veteran or elite promotion sound, for the local player's objects
      (S-0311 claim 14 owns request eligibility and its coverage limits).
    - Crushing (S-0210), jumps (S-0207), radar on and off (S-0410), credit ticks (S-0416), the idle
      comment (S-0107, claim 14) and triggered sounds (S-0011, claim 32) are specified there.
    - Backing: `measurable` (F4 for the local production, movement, placement and sale
      controls; F12 for turret lifecycle; S-0311 claim 14 and X1 for promotion feedback)
    - Confidence: high (movement draw, placement, sale, turret instance lifecycle),
      medium (rotation sampling across specialised object kinds, grace counter).
      Non-local production and specialised movement states are outside this claim's coverage.

24. **Interface sounds and refusal cues.** Sidebar clicks, tab and button clicks and the other
    interface sounds are screen sounds of game-wide entries (Import mapping). The cues are:
    - *start, queue, resume, hold, cancel* (S-0411 claim 12): the game-wide build click sound.
      Start, resume, hold and cancel also request the matching announcement of claim 28.
      Adding a queued copy to an active line requests no new start announcement.
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
      the message. Completing a tick permits a new cue; multiple notices before that completion are
      outside this claim's coverage. A refusal seconds later in the same running game
      therefore gives feedback again, not silence for the rest of that game.
    - *sidebar scroll-button request at an end*: the game-wide refusal sound when the strip
      is already at the requested end. A page request with some room remaining is accepted
      without this refusal even if its eventual movement is clamped; it requests the
      tab-scroll sound instead. A dispatched button request on a strip whose icons all fit
      is refused in either direction. S-0411 claim 18 specifies dispatch: both page buttons
      are disabled when all icons fit; on an overflowing tab they remain enabled at either
      end, activate on release over the button without held repeat, and wheel events move
      one row silently without dispatching a page-button request. Thus an ordinary click on
      a disabled button requests neither sound.
    None of these depends on the local player's side. Movement-order refusal conditions
    belong to the movement specs, not this claim. Clicking
    an item already at its build limit gives no order (S-0411 claim 4) and plays no click sound and
    no announcement. No cue is added for an ignored order or for a sidebar action whose feedback S-0411
    leaves unresolved. Other interface button and tab cues are outside this claim's coverage.
    - Backing: `measurable` (F5 for the sidebar cues, full-queue refusal and refused placement,
      F8 for repeated route-mode feedback after completed ticks). Scroll-button boundary feedback is `measurable` in F10. Build-limit refusal at application
      is `measurable` in S-0404 P3(k), using its
      issue/application recorder and void rule to distinguish that refusal from an at-limit
      click that issues no order. Scroll input dispatch has S-0411 claim 18's
      `measurable` backing (S13).
    - Confidence: high (sidebar click cues, busy line, refused placement, repeated route cues),
      medium (refusal as the request is applied)

25. **Lightning storm sound.** Each lightning bolt of the weather superweapon's storm (S-0203,
    *storm bolt*) that strikes plays one of the game-wide lightning sounds at the strike point,
    positional, chosen by one raw value of the scenario random stream (F5). This is a draw of the
    synchronised game state, taken whether or not anyone hears it, whatever the sound settings, the
    view or the shroud. Within one strike it comes after the raw value that chooses the bolt's
    animation and that animation's creation draws, and before the draws of the strike's explosion
    animation and of its damage. No sound-choice value is taken when the game-wide lightning
    sound list is empty. Repeating the previous strike's point does not suppress the sound
    choice or the later strike effects and draws (S-0513 claim 18).
    - Backing: `measurable` (scenario F6)
    - Confidence: high

### Announcements

26. **Announcement data.** The announcement data file has a list section of announcement names,
    and each name's section gives a kind (default *standard*), a priority (default *normal*), and one
    sample file name per voice set (Allied, Soviet, Yuri), cut to 8 characters with the extension
    added. An empty or unknown kind or priority word keeps the value already in force (the default
    for a new entry). Duplicate list names re-read the same entry. A volume key exists and has
    no effect (quirk `quirks.announcement_volume_ignored`). The voice set is fixed when the game is
    loaded from the local player's side: the first side's set, the second side's set, any other
    side the third set.

    **Announcement text (Q-1735).** In the original, only the kind, priority and per-voice-set
    sample keys of an announcement's section have an effect; the volume key (above), the text key
    and any other key have none. Our engine gives the text key one use, the caption below; the
    importer reads a text key's value as the caption text itself, not as the name of an entry of
    the game's string data.

    The original's announcement data gives some entries a text key, with literal words or an
    empty value, and gives most entries none; no text key's value is the name of an entry of the
    game's string data. No entry of the string data is named after an announcement entry. Entries
    named after the sound files that some announcements play exist there, but the original never
    looks any of them up, so no string entry is tied to an announcement. What the data holds has
    no effect on play in the original, since the original shows none of it (`unmeasured`, below).

    In the original, changing or removing the text key changes nothing heard or shown. A missing
    text key, or a missing entry of the game's string data, has no observable effect: nothing is
    shown with an announcement either way. The original's options menu offers no setting for
    announcement text. An announcement is only heard: playing one posts no line to the message
    area and draws no words anywhere. So in a skirmish no option, and no combination of options,
    shows text for an announcement (`measurable` for part T's settings, `unmeasured` beyond them,
    below), and no text depends on the voice set. G1 part T checks this for the settings it
    covers (each display option its listing finds that can show text, on and off, and voice
    volume full and zero, each of these under each of the three voice sets); for every other
    setting of a skirmish it rests on the examination of the original (`unmeasured`, below).
    Campaign missions and network games are outside this spec (Summary).

    *Announcement text* here means words shown because an announcement plays. A line that a game
    event posts in the message area itself, such as S-0414 claim 13's break message, is outside
    this claim: it belongs to that event's spec, and it is shown or not regardless of the
    announcement data. Part T tells the two apart by removing the announcement's entry.

    Our engine's caption setting is an accessibility addition, presentation only and never part of
    the simulation. Its caption for an announcement that starts playing is the entry's text key when
    that is present with a nonempty value (S-0001 claim 11), and otherwise the entry's name. It is
    the same for every voice set. The caption keeps the whole value as read: no length limit of its
    own applies beyond S-0001 claim 3's line length limit, which bounds every value. How long
    captions stay, how many show at once, where they are drawn and how an over-wide caption is
    wrapped or cut on screen are our engine's interface choices, outside the original's behaviour:
    the original shows no caption, so nothing here constrains them. The values the engine already
    uses (each caption shown for 4 seconds, at most 3 at once, newest last) stand as those interface
    choices.
    - Backing: `measurable` (scenario G1)
    - Backing (text): `measurable` (scenario G1 part T) for: no options-menu setting for
      announcement text; no text shown with an announcement with part T's options; changing or
      removing the text key having no effect on what is heard or shown, for the announcements that
      play both in part T's first setting and in its repeats with the changed copies. `unmeasured`
      for: no text shown in a skirmish with options or combinations of options that part T does not
      set, since part T samples the settings rather than covering them (the settings file can hold
      options that the options menu does not list), so this rests on the examination of the
      original, which finds that an announcement shows no text and posts no line when it plays,
      whatever the options. Also `unmeasured`: the text key having no effect for entries that do not
      play in both of those games, since an entry's key could show an effect only when that entry
      plays; a missing entry of the game's string data having no effect, since part T does not vary
      the string data; the section's other keys having no effect, since G1 does not vary them; what
      the original's announcement data and string data hold (text keys with literal words, empty or
      absent; no string entry named after an announcement entry or by a text key's value; no string
      entry looked up for an announcement), which was read from the installed data files and from
      the original's handling of an announcement, not observed in play, and which no scenario can
      show since the original displays none of it. The caption rule, its length rule and its
      interface values included, is `unmeasured` (our engine's design).
    - Confidence: high for the kind, priority and per-voice-set sample keys and for the voice set's
      fixing at load (scenario G1). Medium for every part-T claim while part T is unrun: no
      options-menu setting for announcement text, no text shown with part T's options, no text
      depending on the voice set, and the text key without effect for the announcements that play in
      both of those games; part T also watches only the battlefield, the message area and the
      sidebar, so an effect elsewhere would escape it. Medium, as `unmeasured`: no text shown with
      other options of a skirmish, the text key without effect for entries that do not play in both
      of those games, a missing string entry without effect, the section's other keys without
      effect, what the announcement data and string data hold, and the caption rule. Medium also:
      volume ignored.

27. **One at a time, with a gap.** Announcements play one at a time, separate from the 16
    channels. The following queue rules cover unmuted playback. A request for the announcement
    playing at that moment is ignored. Otherwise:
    - an *interrupt* request stops the current announcement and empties every waiting list
      when there is a current announcement; without one, it does not flush waiting lists,
    - a request for an announcement already waiting with the same kind is not added again,
    - *queued interrupt* requests wait in one list, *critical* requests that are not *queued* wait
      in a second, and *queued* requests wait in one list per priority; all are first come, first
      served,
    - every other request (a *standard* or *interrupt* one below *critical*) gets the single
      *standard* place only if the first two lists are empty and the place is empty or holds an
      announcement of strictly lower priority; otherwise it is dropped.
    The next announcement starts when the previous one has finished and more than 500 ms of the
    audio clock have passed since it started, taken from the queued-interrupt list, then the
    critical list, then the standard place, then the queued lists from *critical* down to *low*.
    Taking one from the first two lists discards whatever holds the standard place.
    This claim does not specify maximum queue capacity or requests during the teleport
    screen effect; movies are outside skirmish scope.
    - Backing: `measurable` (G2) for queue and gap rules
    - Confidence: high

28. **The announcements of a skirmish.** Every announcement below is heard only by the local
    player, with the house conditions given; Formulas F7 gives the nearby-event suppression that
    several of them pass through:

    | Announcement | Raised when | Throttle |
    |---|---|---|
    | base under attack | damage from an attacker to an ordinary local building as tested in G3; also plays the game-wide base under attack sound; wall and specialised-type eligibility is deferred | F7, kind *base attacked* |
    | ally under attack | the same for a non-passive house allied to the local player; also plays the same game-wide attack-warning sound | F7, kind *ally attacked* |
    | harvester under attack | the local player's harvester (or deployed slave miner) takes damage without dying | F7, kind *harvester attacked* |
    | low funds | the local player's available credits are below 100, its speech-delay countdown has expired, and it owns at least one infantry, vehicle, building or naval factory | restarts that countdown at N(⌊*k* × 900⌋) ticks, *k* the speak-delay rule value in minutes; the depot's low-funds case is S-0314's |
    | low power | the local player's output is below its positive drain and it owns at least one of the first three types in the imported construction-building list | once per low-power episode: again only after power has recovered |
    | unit ready | a produced unit of the local player leaves its factory | F7, kind *unit produced*, at the factory |
    | construction complete | a building in the local player's sidebar becomes done | none |
    | new construction options | a new item appears in the sidebar, not during map loading | none |
    | building, training, on hold, cancelled, unable to comply | the matching production-feedback click of S-0411 claim 12; training only for infantry, building for every other producible kind, including vehicles, aircraft, naval units and defences | none |
    | structure sold, unit sold | S-0408 | none |
    | unit lost | local vehicle death as tested in G4; exemptions and other specialised death eligibility are deferred | F7, kind *unit lost* |
    | unit repaired | S-0314 | F7, kind *unit repaired* |
    | unit promoted | S-0311 | none |
    | primary building selected, rally point set | the local player's matching action | none |
    | building captured, tech building lost | building ownership changes that request an announcement and involve the local player, under claim 39 | none |
    | structure garrisoned | an entry makes the occupant count 1 and the entering unit belongs to the local player, regardless of the building's owner (S-0508 claim 6); also requests the positional garrisoned sound at the entering unit's last position | none beyond claim 27 |
    | structure abandoned | the building-turn ownership change to the civilian house of S-0508 claim 8, when the owner losing the building is the local player; also requests the screen abandoned sound without radar-event throttling | announcement only when F7 creates a *garrison abandoned* event at the building's top-left cell |
    | cannot deploy here | S-0415, S-0406 and a refused placement (S-0411 claim 10, claim 24) | none |
    | building online, building offline | S-0405 | none |
    | superweapon ready | an ordinary superweapon record of the local player becomes ready under S-0512 claim 5; its kind selects the entry through that spec's Import mapping and this spec's Import mapping | none beyond claim 27 |
    | superweapon detected | an eligible superweapon-building placement of the non-allied house in G4; this row does not complete eligibility | none |
    | superweapon activated or launched | an eligible activation of a mapped category in G4; other categories and complete ownership eligibility are deferred | none |
    | victory, defeat, player defeated | S-0414 | none |

    Start and resume use that same infantry-versus-other split. Adding a queued copy to an
    already active line does not request another building or training announcement; it still
    requests the build-click sound. These rules describe announcement requests, not guaranteed
    playback through claim 27's queue. F11 tests this distinction.
    Drain-mode charge completion requests no ready announcement (S-0512 claim 19).
    Ordinary readiness timing and request eligibility belong to S-0512 claim 5, not to
    announcement playback: a request can still be dropped or delayed by claim 27.
    Announcement sample selection and queue service take no random draw. This does not
    assert the absence of random choices in the feature that raises an announcement.
    Infiltration announcements and their request conditions belong to S-0510 claim 11.
    The table lists common announcements, not every skirmish announcement:
    alliances, beacons, crates and specialised abilities also raise announcements whose complete
    conditions belong to their feature specs. Superweapon launch and detection announcements
    additionally respect the game's announcement-muting setting; detection has type-specific
    eligibility conditions not resolved here.
    Rows naming unresolved exemptions or specialised eligibility are event roles, not
    complete eligibility rules. Their scope is limited to the controls below; no rule for
    every type or owner follows. In particular this claim adds no non-local garrison-entry
    prediction beyond S-0508 claim 6.
    - Backing: `measurable` (G3 for low funds, low power, ordinary base and harvester
      attack; G4 for the listed fixed bindings and ally-attack control; S-0512 claim 5,
      X1 and X11 for ordinary readiness and drain-mode completion; F11 for production
      kinds; S-0508 claims 6 and 8, G4 and G6 for local garrison entry and abandonment).
      Other feature-owned request rules retain their cited specs' backing, not a new
      exhaustive eligibility claim here. Queue-service draw absence is `measurable` in G1.
    - Confidence: high (tested thresholds and bindings, local garrison feedback),
      medium (radar-event gating of the abandoned announcement)

### Determinism

29. **Game state is independent of playback.** Our engine must not make game rules depend on
    whether, when or how a sound or announcement played, on the audio clock, or on audio passes.
    H1's paired original-game continuations predict identical scenario-stream state and
    game-state observables despite different playback. Oracle coverage is limited to
    H1's events; no exhaustive catalogue of original game-rule readers is claimed.
    - Backing: `measurable` (scenario H1) for the tested events; design requirement for
      our engine's general separation
    - Confidence: high (requirement); medium (oracle coverage)

30. **Draw summary.** All traced draws in claims 13, 14, 17, 18, 20, 21 (exceptions) and 23 use
    the unsynchronised stream. State and ordering lists the immediate-mode instance draws.
    Delayed-mode full ordering remains open. Instance choices happen at playback service
    opportunities, rather than all at the game-side request, so their position in the stream
    depends on real time and playback interruptions in the original. Claim 14 defines
    immediate-mode draws for uninterrupted playback, not a fixed count for every instance.
    Our engine retains this shared unsynchronised stream, as required by S-0202 claim 11,
    for both game-side sound choices and per-instance choices; no separate presentation
    generator is substituted. The following schedule is our engine's design rule, a
    deterministic departure except for music time in the options menu (Music time below), not a
    description or proposed measurement of the original.

    **Clock and passes.** In a loaded skirmish, logical audio time *C* and last-pass time *L* begin
    at 0 ms, and no pass occurs at that instant. At the end of each cycle—after that cycle's input,
    presentation requests and any tick—*C* is greater than its previous value by *P(s)*, where *P(0)
    = 8 ms* and *P(s) = 16s* ms for speed indices 1 through 6, using the speed in force during that
    cycle (S-0201 claims 4, 5 and 8). A pass occurs at that cycle end exactly when *C − L > 33
    ms*; when it occurs, the new value of *L* is *C*. Otherwise *L* is unchanged. These are exact
    integer-millisecond operations, without rounding or clock wrap. At constant speed the interval
    between passes is *P(s) × (⌊33 / P(s)⌋ + 1)* logical milliseconds: 40, 48, 64, 48, 64, 80 and 96
    ms at indices 0 through 6 respectively. Speed changes do not restart the cadence. Each pass ends
    with one music update (S-0703 claim 10), after that pass's sound choices, so the ranged draws of
    a music pick follow that pass's draws on the same unsynchronised stream. Thus the strict lower
    bound of claim 12 is retained without making pass choices depend on host execution time. Only
    music time in the options menu follows real time (Music time below). Frozen cycles advance
    logical audio time and can service audio despite running no tick. During a stand-still,
    including the options menu or lost focus, neither logical audio time nor sound playback
    advances, and resumption causes no catch-up. Music time alone advances in the options menu
    (Music time below). No skirmish audio passes occur in setup menus before loading, after the
    cycle that runs the end delay's final tick (the game is over, S-0414 claim 8), or after leaving
    the match. The end delay's ticks run in ordinary cycles of the loaded skirmish, so this schedule
    gives them audio passes, each with its music update. The options-menu stand-still has no audio
    pass and no sound progress. Other menu audio is outside scope. The music of the options menu in
    a loaded skirmish, and of our engine's own settings screen opened from a game, is covered by
    Music time below.

    **Music time.** Music time *M* and last-music-pass time *K* begin at 0 ms with *C* and *L*.
    Outside the options menu, *M* increases with *C* at each cycle end by the same *P(s)*, and *K*
    becomes *M* at each audio pass. While the options menu is open in a loaded skirmish, *C* and *L*
    stay unchanged and no audio pass occurs, but *M* advances with real time: the host's monotonic
    clock is read once as the menu opens and then once for each frame the menu draws, and at each
    later reading *M* increases by the whole milliseconds elapsed since the previous reading, any
    fraction of a millisecond being carried to the next. A music-only pass occurs at a reading
    exactly when *M − K > 33 ms*, so there is at most one per reading; it sets *K* to *M* and runs
    one music update (S-0703 claim 10) and nothing else: no sound choice, wait, completion,
    announcement service or draw other than that update's. A track counts its logical duration on
    music time: one started at a pass with music time *M₀* completes at the first pass, audio or
    music-only, with *M* ≥ *M₀* + *T*, *T* as in Completion and waits below with *p* = 100. On the
    menu's closing the next cycle resumes the ordinary schedule with no catch-up; the time spent in
    the menu counts for music only. Our engine's own settings screen opened from a game counts as
    the options menu here. Lost focus remains a full stand-still, for music as for sound, also while
    the options menu is open: from the loss of focus music time stops, and no reading is taken and
    no music-only pass occurs until focus returns; at its return the clock is read afresh and that
    reading becomes the previous reading, with the carried fraction kept, so the next increment
    counts only time since focus returned and the unfocused stretch adds nothing to *M*. **What
    depends on real time.** These, through the time the options menu is open with focus: while it is
    open, when music-only passes occur, so when a track completes there and when the next one
    starts, and how many music picks are made there; after it closes, when the track playing at its
    closing completes, since part of that track's duration was counted in the menu, and so the cycle
    of the next music pick and of every later track's start and completion; and, since music picks
    take their ranged draws from the unsynchronised stream, the stream's position after any of these
    picks, so the later sound choices drawn from that stream (claims 13, 14 and 17 to 23), with
    the pitches they draw and so those sounds' durations, completions and channel use, and later
    music picks. No tick, game state, scenario-stream draw or game rule depends on it (claim 29),
    and no sound choice is made while the menu is open. Time with focus lost, and time with the menu
    closed, add to none of these. The project lead has accepted this dependence (S-0703 claim 13).

    **Completion and waits.** Each selected sample has a logical duration
    *T* = max(1, ⌈100000 × *N* / (*f* × *p*)⌉) ms, where *N* is its decoded number of
    sample frames (one frame contains all channels), *f* its positive integer sample rate
    in frames per second, and *p* its positive pitch percentage from claim 13. This is exact
    rational arithmetic rounded upward once to integer milliseconds, not a device-reported
    duration. Announcements use *p* = 100. A sample logically started at pass time *C₀*
    completes at the first pass with *C* ≥ *C₀* + *T*. There is no callback-driven completion
    and no carry-over of elapsed excess into the next sample. Even a zero-frame sample has
    duration 1 ms, preventing an instantaneous endless loop. Nonpositive rates or pitch are
    invalid data: warn and discard that playback without sample-preparation draws; already
    taken instance draws are not undone. This invalid-data rule is also an engine departure.

    Retain the delay draws and the no-wait thresholds of claims 13 and 14. An effective
    wait of *d* ms chosen at time *C₀* expires at the first pass with *C* > *C₀* + *d*.
    A no-wait result permits preparation/start in that same pass, not a new pass or a new
    completion. Each newly started sample can complete only at a later pass. Delayed-mode
    completion choices happen at these logical completions, with terminal-delay eligibility
    and partial choice ordering as claim 14 states. This supplies their opportunities, not
    the still-deferred complete attack/decay sequence. Announcement requests keep claim 27's
    immediate interruption and queue effects; queue playback is serviced only at passes,
    after sound instances. Its next start requires both logical sample completion and
    *C* − previous start > 500 ms. A first announcement needs no previous-start gap.
    Device callbacks, underruns, mute settings and delivery failures never create or remove
    a logical completion, wait, channel or draw opportunity.

    **Draw assignment within a cycle.** Every game-side choice in claims 17 to 23 is assigned to its
    causing input or game event, in the event order supplied to presentation, and precedes any
    per-instance choice at that cycle's end pass. Such choices are not postponed until channel
    allocation, and no pass separates damage, movement or order events belonging to one tick. If a
    game-side stop or cut has taken effect, the cancelled sample has no later completion draw. Until
    a pass recognises completion, an object's intervening update continues to regard its response as
    speaking, even if the logical duration deadline has passed.

    Sound requests have increasing presentation request numbers in causing-event order, including
    requests received before the first pass. At a pass, the shared-stream choices attributable to
    due completions occur in request-number order. Entry placement and limits are resolved before
    new channel allocation, under claims 6 to 10; exact entry-ranking ties favour older requests
    unless claim 8's interrupt rule favours the new request. Among instances that survive,
    allocation, expired-wait preparation and start choices occur in request-number order, with each
    instance retaining claim 13/14's internal draw order. When channels are free, the
    lowest-numbered free channel among channels 0 through 15 is used, and claim 9's channel-order
    ties follow that same order. A sample cut by an entry limit or channel replacement has no later
    completion event. Consequently, these observable event and draw assignments are fixed without
    prescribing data structures or the implementation's work schedule. Other presentation consumers
    retain their event order in the supplied trace.

    Identical initial stream state, data, cycle-indexed events, presentation inputs, speed
    changes and options-menu clock readings (Music time) must therefore give identical logical
    pass times and assigned choices regardless of host load or audio callbacks. Two runs that
    differ only in how long the options menu stays open with focus can differ in what Music time
    lists as depending on real time. A run that never opens the options menu
    with focus takes no clock reading, so the listed inputs without readings decide it. Physical
    delivery timestamps may differ.
    Scenario-stream state and game rules remain unaffected (claim 29).
    - Backing: `measurable` (E1 for original instance draw order, E2 for original audio-clock
      eligibility, H1 for original separation from game state). The replacement schedule,
      duration calculation, tie rules, shared-stream retention, the music update at each pass's end,
      the absence of passes after the end delay's final cycle, music time with the options menu's
      music-only passes, and the absence of music updates in a lost-focus stand-still are design
      requirements, not oracle measurements: engine-only H2's music controls test them, and
      engine-only S-0703 MU7 also tests the music update's place after the pass's sound choices.
    - Confidence: high for the original's instance draw order and audio-clock eligibility (E1,
      E2); medium for the original's separation from game state, as claim 29 rates H1's oracle
      coverage. Medium for the design requirements, music time
      in the options menu included: they are our engine's rules, checked only on our engine
      (H2, MU7), with no oracle run behind them.

31. **Response draws come before the local-player check in two places.** The feedback chance of
    claim 20 is drawn for every owner's object; a selection response draw for a passive house's
    object (claim 17) is taken and then discarded. Only the unsynchronised stream is affected.
    - Backing: `measurable` (F1 for other owners' feedback draws, D3 for passive selection)
    - Confidence: high

32. **Weapon report variant depends on the creation draw.** The report variant (claim 21) is
    determined by the scenario random stream through the object's creation draw, so it is the same
    on every replay of the same game, and a change to the creation draws changes which report each
    object uses. By the project's Q-0327 decision, our engine keeps the unsigned integer
    *r* = *x* mod 65536, in 0 … 65535, where *x* is the existing creation raw value.
    This exact integer reduction requires no rounding and no additional draw. Saved games
    preserve *r*, loading restores it, and the deterministic state hash includes it.
    Presentation can obtain it for every existing object, without replaying creation or
    choosing another value. Simulation gameplay never consults it. This agrees with
    S-0202 claim 10's creation-derived report contract; forcing every object to variant zero
    is not a conforming substitute.
    - Backing: `measurable` (F2 for the original's variant; F2's engine-only conformance
      control for persistence, hashing and presentation access). The latter is a design
      contract, not an oracle claim
    - Confidence: high

33. **Scenario-stream sound choice in scope.** Lightning's sound choice is claim 25's
    scenario-stream exception, even when inaudible. This statement classifies the choices
    specified here, not every indirect sound-related choice. Random-waypoint sound actions
    outside S-0011 claim 32's trigger subset are deferred to the future trigger specification.
    - Backing: `measurable` (scenario F6)
    - Confidence: high

### Sample sources and volume options

34. **Where samples come from.** Every sample a sound entry names is looked up, ignoring case, in
    one sample bank: an index file and a data file (Import mapping), found through the ordinary
    file lookup of S-0005 claim 10a. The two files resolve independently: a loose index can
    accompany archived data, and the files need not come from the same archive. Sound entries
    never play separate sample files. Of the installation's two audio archives the expansion's is
    mounted when present, else the original's (S-0005 owns the mounting). Announcements work the
    other way: each is a separate file named by claim 26's eight-character name plus the extension
    `.wav`, opened through the same file lookup and never taken from the bank. This claim covers unique bank sample names only; duplicate-name resolution is outside
    this spec's coverage.
    The public byte layout of the bank is outside this spec; no new formats spec is required.
    - Backing: `measurable` (scenario A4 for separate files versus bank samples and
      case-insensitive sample lookup; S-0005 M22 part a for archive fallback).
      Independent replacement of the two bank files is `measurable` in S-0005 M25(b),
      which supplies compatible distinguishable bank fixtures; A4 alone does not test it.
    - Confidence: high (one bank, separate announcement files)

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
    - For S-0414 claim 5's eligible defeat requests, the local player's defeat uses the defeat
      entry and another non-passive house's defeat uses the player-defeated entry. A local
      win uses the victory entry. Observer eligibility remains with S-0414 claim 5.
    - S-0414 claim 5 owns post-defeat muting, including alliance-formed and superweapon
      launch/detection announcements. This spec does not specify alliance announcement
      selection or entry bindings.
    - The play-speech trigger action (S-0011 claim 32) requests the entry it names as kind
      *interrupt*, whatever kind the entry has.
    - The additional fixed bindings for primary selection, rally, garrison occupancy and
      building power state are in Import mapping. Superweapon announcement entries vary
      with the superweapon category, not with the local player's faction; Import mapping
      lists the established category bindings. Unlisted activation/launch bindings and
      exhaustive activation/detection eligibility are outside this spec's coverage.
      G6 explores those gaps; an unestablished binding is not proven silence.
      Feature eligibility remains with the feature specs. Claim 39 covers entry selection
      on the ownership changes of claim 39 only.
    - Entry names are matched ignoring case; an unknown name plays nothing.
    These are presentation rules; they do not end a game. Our engine keeps playing to the
    last alliance under S-0414 claim 6, even after the local player's defeat. The described
    announcement muting still applies during that continued observation.
    - Backing: `measurable` (scenario G4) for the fixed entry bindings and events exercised
      there. S-0414 claim 5 and V20 carry post-defeat muting's own backing and limits.
      Nothing here strengthens its conditional alliance controls. Muting during our engine's
      extended observation is a presentation requirement, not an oracle claim about that period.
    - Confidence: high (entry per tested event)

37. **Building damage and death fallbacks.** When a building's damage result (S-0302 claim 12) is
    *now yellow* or *now red* and its type names no damage sound of its own, the game-wide building
    damage sound plays at the building in the attached form that follows scrolling (claim 7),
    for any owner. On a lethal damage result, when the building begins its destruction reaction
    rather than when it is finally removed, an empty type death-sound list requests the game-wide
    building death sound the same way, for any owner. Delayed removal does not delay this request.
    A type with its own sound uses claim 20 instead.
    - Backing: `measurable` (scenario F7)
    - Confidence: high (fallback guards, ownership and lethal-hit timing)

38. **Initial game-wide audio defaults.** At a fresh program start, the game-wide sound
    references in Import mapping mean no sound, and the lightning list is empty. There are no
    built-in entry-name fallbacks for those references. The low-funds repeat field initially
    equals 2 game minutes, so its countdown is N(1800) ticks. Later missing keys retain the
    values already in force, including across games under S-0006's rules-database persistence.
    - Backing: `measurable` (A5)
    - Confidence: high

39. **Capture announcement entry selection.** This claim covers building ownership changes whose
    cause requests a capture or loss announcement: engineer captures and vehicle-like takeovers, which
    S-0509 claim 12 lists, and the mind-control takeover and release of a building (S-0601 claims 6 and
    9), which S-0509 claim 12 also lists. The local player gains or loses the building and the new owner's
    country is not passive. The entry choice below is the same for every such cause.
    A building whose type has the needs-engineer flag (S-0405 claim 8) uses its imported capture
    announcement when its new owner is the local player. With no capture announcement, this
    branch is silent: it does not fall back to the ordinary building-captured entry. Losing
    such a building as the local player requests the tech-building-lost entry instead.
    A building without that flag uses the ordinary building-captured entry for the eligible
    ownership-change announcement, not its imported custom capture entry. These alternatives
    do not depend on the local player's faction; their samples still use claim 26's voice set.
    The imported custom reference is an announcement entry, never a sound entry or a sample.
    Ownership-change causes that S-0509 claim 12 does not list as requesting an announcement
    (trigger and occupation changes) and passive new owners are outside this claim's coverage.
    - Backing: `measurable` (G5) for engineer captures, for the mind-control takeover and release (MC9) and
      for vehicle-like takeovers (S-0509 scenario E12's setup as a G5 fixture); eligibility itself is stated
      by S-0509 claim 12 and S-0601 claims 6 and 9. S-0509's own scenario E14 does not exercise vehicle-like
      takeovers, so that case rests on G5.
    - Confidence: high (engineer-capture entry selection and missing-custom silence); medium (mind-control
      entry selection, because its fixture MC9 is proposed, not run; vehicle-like entry selection,
      because its G5 fixture is proposed, not run)

### Request coverage and remaining limits

40. **Sound requests, entry settings and response choices are distinct.** A sound reference
    selects an entry, not a same-named sample. Choosing that entry at a game event and choosing
    its samples during playback are separate choices. Thus a one-entry response list still
    takes one raw choice value (F5), even when its entry has only one sample; a single response
    reference takes none. Either entry can later take the instance draws of claims 13 and 14.
    Neither an entry's delay nor its loop count imposes a minimum interval between independent
    requests of that entry. Delay controls waits within each instance; overlapping requests
    remain subject to placement, entry limits, channels and any event-specific gate.
    Screen versus positional placement and local-owner eligibility belong to the requesting
    event, not to a universal rule that every sound is heard only by its owner. Claim 2's
    established placement words then modify positional requests as claim 6 specifies.
    - Backing: `measurable` (D2 for list versus single choices, E1 for playback choices,
      C1 with a nonzero pre-delay for independent overlapping instances, B1/B2 for placement)
    - Confidence: high

    The following table locates every part of the three requests. References to Import mapping
    mean the rows for the named concept, with their parse forms and absent-key defaults.
    These answers replace the requests' interim rules only within the stated coverage.

    | Request question | Answer and import concepts |
    |---|---|
    | Q-0260, 1: name resolution | claim 1: case-insensitive entry lookup, 31-character listed-name cap, full reference comparison, unknown reference means no entry; Import mapping: entry names and reference retention rules |
    | Q-0260, 2: samples, order, repeats and draws | claims 2, 13 and 14, F5 and State and ordering: sample list versus event-choice list, immediate order and looping, pitch/shift/pre-delay draws; claim 34: bank lookup, not separate same-named files; Import mapping: samples, control, attack/decay counts, loop and delay; complete delayed-mode totals remain unmeasured below |
    | Q-0260, 3: volume, pitch, priority, limits, gaps, placement, owner and range | claims 2, 3, 5 to 15, 35 and this claim; Constants and Import mapping give entry and game-wide defaults; request-specific owner checks are claims 16 to 23; remaining placement-word semantics remain unmeasured below |
    | Q-0260, 4: missing samples | claim 4 and its quirk: one missing bank sample silences the whole entry; A1 and A4 distinguish missing bank samples from loose files |
    | Q-0264, 1: firing and spin sounds | claims 21 and 32: weapon report list, fixed creation-derived ordinary variant with no per-shot choice draw; occupant and spin exceptions; S-0609 claim 13 owns spin lifecycle; Import mapping: weapon report list; special-effect classification remains unmeasured below |
    | Q-0264, 2: death sounds and draws | claim 20: local death-response choice before any-owner death-sound choice, each F5; claim 22: separate animation sounds under S-0109 claim 13; claim 37: building death fallback; Import mapping: the two death lists, building death fallback and animation sounds; total specialised infantry multiplicity remains unmeasured below |
    | Q-0264, 3: building threshold damage sound | claim 37: game-wide building damage fallback only when the type names no damage sound; claim 20's own damage sound instead belongs to unchanged-hit feedback, not a guaranteed threshold sound; Import mapping: type damage sound and game-wide building damage sound; S-0302 claim 12 owns result classification |
    | Q-0264, 4: owner and audibility | claims 20 to 22 and 37: death response local only, death sound/report/animation/building fallback any owner, all positional; claims 6 and 7 still govern actual audibility, not ownership alone |
    | Q-0265, 1: each order and empty-list fallback | claim 18 and F6 give every requested ordinary order kind, weapon-specific singles and repair exception, fallback exhaustion means silence, never the selection list; Import mapping: order lists and single responses; building unload/deploy exceptions remain unmeasured below |
    | Q-0265, 2: choice algorithm | claim 18 and F5: exactly one unsynchronised raw value modulo list length, including a one-entry list; none for an empty list or a single response reference; claims 13 and 14 govern separate playback draws |
    | Q-0265, 3: selection and repeat selection | claims 16 and 17: first newly selected object only, specialised selection lists, ordinary re-click versus modifier deselection, the held type-select role's silence and a type-select tap's single response (S-0214 claim 32); claim 18 gives resulting selection order for subsequent orders; Import mapping: the three selection lists |

    **Unsettled parts are not silence or substitute rules.** For Q-0260, the complete
    delayed attack/decay sequence, terminal lifetime and draw totals are `unmeasured`, low
    confidence: the established preparation and completion choices have not been composed
    into a complete sequence (E3). Playback semantics of the remaining recognised placement
    words, including any additional owner-only restriction, are `unmeasured`, low confidence:
    the positional rules alone do not establish their effect at every requesting event (A6).
    For Q-0264, total specialised infantry death multiplicity and report choices of special
    weapon effects outside claim 21 are `unmeasured`, low confidence: the common reactions
    do not establish those paths' total requests (F9 and F13). For Q-0265, building
    unload/deploy response gates and specialised deploy-response suppression are `unmeasured`,
    low confidence: F6's ordinary-object controls do not establish those exceptions (D5).
    These deferred values have settling scenarios, not provisional conformance values.
    The request IDs in front matter include this explicit partial disposition, not a claim
    that these remaining questions are closed.

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

The following master-level rule covers settled levels only; the transition ramp is outside
this spec's coverage. The master level is not necessarily full even in an ordinary skirmish. At an audio pass, let *C*
be the sum of ⌊20 × *e* / 16384⌋ over surviving, non-paused sound instances. Its target level is
16384 when *C* ≤ 100, and ⌊1638400 / *C*⌋ otherwise. These are exact integer operations. After settling, many simultaneous sounds therefore reduce the loudness of all sounds.
The target is `measurable` in C3, high confidence; no transition-timing requirement is made.

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

No raw value is taken when *n* = 0. For the ordinary per-shot weapon report of claim 21,
*x* is replaced by the low 16 bits of the object's creation draw and no value is taken.
Its spin-sound and occupant exceptions use fresh raw values as that claim specifies.

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
multi-weapon type. Building deploy acknowledgements and specialised deploy-response
suppression are outside this table's coverage; the deploy row covers ordinary infantry and
vehicle acknowledgements tested in D2, not a building override.

**F7. Nearby-event suppression** (claim 28). Each announcement that passes through it first tries to
create a radar event of its kind at a cell (the building, unit or factory). For *garrison abandoned*,
S-0508 claim 8 owns the request at the building's top-left cell and its announcement gate.
The attempt fails, and the announcement is not raised, when a radar event of the same kind exists whose cell is at a distance
⌊√(Δ*x*² + Δ*y*²)⌋ < *r* cells (floating point, truncated), *r* from this table. The event lasts its
zoom-in phase and then *t* ticks. S-0410 claims 14–15 specify the zoom duration and
cadence, with their formula F9 and scenario R13: zoom advances once per drawn frame,
even without radar or while the panel opens or closes. Completion starts the *t*-tick
tail at that frame's tick number, not at event creation. A timed freeze advances zoom
but not ticks; a stand-still with no drawn frames advances neither. Restoring radar
does not restart an event. The duration must not be replaced by zero or by a fixed tick
count. The suppression radii and post-zoom tails are:

| Kind | *r* (cells) | *t* (ticks) | Suppresses |
|---|---|---|---|
| combat | 8 | 400 | yes |
| non-combat | — | 400 | no |
| drop zone | — | 400 | no |
| base attacked | 8 | 600 | yes |
| harvester attacked | 8 | 400 | yes |
| enemy sensed | 6 | 400 | yes |
| unit produced | 2 | 200 | yes |
| unit lost | 8 | 200 | yes |
| unit repaired | 2 | 400 | yes |
| infiltrated | — | 400 | no |
| captured | — | 100 | no |
| beacon | 8 | 200 | yes |
| superweapon detected | — | 400 | no |
| superweapon activated | — | 5 | no |
| bridge repaired | 8 | 200 | yes |
| garrison abandoned | 8 | 400 | yes |
| ally attacked | 8 | 600 | yes |

For a kind marked *yes*, radar events are game-wide, not per house: a matching event suppresses
another request regardless of why the first event was created, including triggered events
(S-0011 claim 32). Kinds marked *no* do not apply the distance rejection. The complete set of event creators is outside this spec's coverage. F7 is high confidence
and `measurable`: G3 tests production, base and harvester creators; G7 tests every table row
through the radar-event trigger and cross-creator pairs for production, attack, repair and
abandonment. This does not assert coverage of every possible creator. The *garrison abandoned*
creator and announcement gate retain S-0508 claim 8's medium confidence and G4/G6 backing.
Zoom timing retains S-0410 claims 14–15's backing and confidence, including their unresolved
exact floating-point equality boundaries. Proposed G7 observations are not completed measurements.

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
| Audio pass interval | more than 33 | ms, original audio clock | claim 12 |
| Logical cycle increment, speed 0 | 8 | logical ms per cycle | engine design, claim 30 |
| Logical cycle increment, speeds 1 to 6 | 16 × speed index | logical ms per cycle | engine design, claim 30 |
| Minimum logical sample duration | 1 | logical ms | engine design, claim 30 |
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

Rows 4 to 8 describe immediate mode only. Delayed mode follows only claim 14's partial
ordering: instance pitch, shift and optional pre-delay precede sample preparation; each decay
candidate precedes that preparation's attack/body choices; sample-completion delay choices
occur at completions, not all at instance start. A complete sequence and count are deferred,
not specified by this table (claim 14, E3).

**Game-side sound draws** (claims 17 to 25): one raw value per list choice, at the moment the sound is
requested, in the tick's order of events; the feedback chance R(0, 99) precedes its choice; for a
death, the death response choice precedes the death sound choice.

**Engine schedule.** Claim 30 fixes cycle-end pass times and the assignment order of
completion, allocation and preparation draws. Game-side sound choices of the cycle precede
all of those pass choices. The original's host-driven service opportunities are not retained.

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
  whose samples leave no body sample is silent when requested, including repeated requests
  without changing its data. Off: the attack and decay samples play. Recovery after changing
  an already-loaded entry to valid data is outside this spec's coverage.
  Backing: `measurable` (A2). Confidence: high.
- **`quirks.shroud_sound_corner_cell`** (default: on). A *shroud* sound at cell (0, 0) is never heard.
  Off: that cell is judged like any other.
- **Damage-sound departure** (claim 20): the original requests two identical damage-sound
  instances per eligible hit, an obvious audio bug confirmed by the project lead and fixed
  without a switch.
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
- **Audio scheduling departure.** The original's real-time
  service interval (claim 12) is a strict lower bound, not an exact schedule or latency
  guarantee. Our engine follows the glossary's deterministic-departure rule, with no quirk
  switch, and with one presentation exception that the project lead has accepted: music time
  in the options menu, which follows real time as claim 30 states and reaches no game state
  (see Changes for other specs, glossary). Both game-side and per-instance choices retain
  S-0202 claim 11's shared unsynchronised stream. Claim 30 fixes the cycle-end clock,
  completion deadlines, waits and draw assignment. Host-dependent output delivery must not
  feed back into choice assignment, the scenario stream or game state.

## Import mapping

Claim 30 adds no INI key or configurable timing field. Its cycle period comes from the
speed index imported under S-0201 claims 7 and 8. Its waits use `Delay` and its pitch uses
`FShift` below with their existing defaults. Sample duration uses the decoded frame count
and sample rate from the selected sample's own audio data, not a sound-rule duration key.
There is no absent duration default. Device metadata is not an import source.

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
| `Type` | placement words | whitespace-separated words, matched ignoring case: `GLOBAL`, `LOCAL`, `SHROUD` directly affect positional playback; last of `SCREEN`/`LOCAL` chooses view-edge/centre distance, last of `UNSHROUD`/`SHROUD` disables/enables explored-cell gating; `AMBIENT`, `VIOLENT`, `MOVEMENT`, `QUIET`, `LOUD`, `PLAYER`, `NORMAL`, `GUN_SHY`, `NOISE_SHY` are recognised but their playback semantics are deferred | defaults section `Type`, else `SCREEN` |
| `Limit` | limit | integer, 0 = none | defaults section `Limit`, else 5 |
| `Loop` | loop count | integer, 0 = forever | 0 |
| `Range` | range | integer, cells | defaults section `Range`, else 10 |
| `Delay` | delay | first two space/tab/newline-separated tokens, leading signed decimal digits, ms; nonnumeric token = 0; one token repeats; further tokens ignored (claim 2) | 0 0 |
| `FShift` | pitch shift | same token form, percent | 0 0 |
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
| `Text` | caption text of our engine's caption setting (claim 26), spelled exactly so (S-0001 claim 13); the value is read as the caption text itself; the original shows it nowhere (claim 26 gives the backing) | string, no length limit beyond S-0001 claim 3; an empty value (S-0001 claim 11) counts as absent | absent or empty: the entry's name |

Announcement requests (claim 36): literal INI section headers in the player's announcement
data file, also named by values in its announcement list. The square brackets below denote
section headers, not program identifiers; list values and announcement references use the
name inside the brackets, without the brackets. Like the other INI section names in this Import
mapping, they are input-format bindings, not names of program classes, fields or functions.
Lookup ignores case; an absent or unknown entry is silent. These bindings are not configurable
rules keys and do not specify sample names: the voice-set fields above specify those.

| Event | Fixed announcement section header |
|---|---|
| construction complete | `[EVA_ConstructionComplete]` |
| unit ready | `[EVA_UnitReady]` |
| new construction options | `[EVA_NewConstructionOptions]` |
| building | `[EVA_Building]` |
| training | `[EVA_Training]` |
| on hold | `[EVA_OnHold]` |
| cancelled | `[EVA_Canceled]` |
| unable to comply (claim 24) | `[EVA_UnableToComply]` |
| cannot deploy here (claim 24) | `[EVA_CannotDeployHere]` |
| structure sold | `[EVA_StructureSold]` |
| unit sold | `[EVA_UnitSold]` |
| unit repaired | `[EVA_UnitRepaired]` |
| unit promoted (either rank) | `[EVA_UnitPromoted]` |
| low funds (house countdown or repair-depot refusal) | `[EVA_InsufficientFunds]` |
| unit lost | `[EVA_UnitLost]` |
| low power | `[EVA_LowPower]` |
| base under attack | `[EVA_OurBaseIsUnderAttack]` |
| ally under attack | `[EVA_OurAllyIsUnderAttack]` |
| harvester under attack | `[EVA_OreMinerUnderAttack]` |
| defeat | `[EVA_YouHaveLost]` |
| player defeated | `[EVA_PlayerDefeated]` |
| victory | `[EVA_YouAreVictorious]` |
| play-speech trigger action | section named by the action, with interrupt kind override |
| primary building selected | `[EVA_PrimaryBuildingSelected]` |
| rally point set | `[EVA_NewRallyPointEstablished]` |
| building captured, ordinary capture announcement | `[EVA_BuildingCaptured]` |
| tech building lost | `[EVA_TechBuildingLost]` |
| structure garrisoned | `[EVA_StructureGarrisoned]` |
| structure abandoned | `[EVA_StructureAbandoned]` |
| building online | `[EVA_BuildingOnLine]` |
| building offline | `[EVA_BuildingOffLine]` |

Superweapon category bindings for claim 36, using the same literal-section-header notation
and case-insensitive lookup. The ready bindings apply only when S-0512 claim 5 requests
an announcement, with its Import mapping selecting the kind; drain-mode completion requests
none (S-0512 claim 19). A teleport second-stage type and a type with no kind have no ready
binding:

| Category | Ready section header | Activation or launch section header |
|---|---|---|
| nuclear missile | `[EVA_NuclearMissileReady]` | `[EVA_NuclearMissileLaunched]` |
| iron protection | `[EVA_IronCurtainReady]` | `[EVA_IronCurtainActivated]` |
| weather storm | `[EVA_LightningStormReady]` | `[EVA_LightningStormCreated]` |
| teleport source | `[EVA_ChronosphereReady]` | `[EVA_ChronosphereActivated]` |
| either paradrop category | `[EVA_ReinforcementsReady]` | no binding established here |
| psychic domination | `[EVA_PsychicDominatorReady]` | `[EVA_PsychicDominatorActivated]` |
| spy plane | `[EVA_SpyPlaneReady]` | no binding established here |
| genetic mutation | `[EVA_GeneticMutatorReady]` | `[EVA_GeneticMutatorActivated]` |
| force shield | `[EVA_ForceShieldReady]` | no binding established here |
| psychic reveal | `[EVA_PsychicRevealReady]` | no binding established here |

Detection bindings traced at superweapon-building placement:

| Building's superweapon category | Detection section header |
|---|---|
| nuclear missile | `[EVA_NuclearSiloDetected]` |
| iron protection | `[EVA_IronCurtainDetected]` |
| weather storm | `[EVA_WeatherDeviceReady]` |
| psychic domination | `[EVA_PsychicDominatorDetected]` |
| genetic mutation | `[EVA_GeneticMutatorDetected]` |
| teleport source | `[EVA_ChronosphereDetected]` |

An unestablished cell is outside this spec's coverage, not a claim that the event is silent.
G6 explores those deferred parts of Q-0325; no entry or universal eligibility rule is invented.
The listed fixed bindings are high confidence, `measurable` in G4's event controls. Feature
request conditions are not completed by this table. Alliance bindings are deferred.
Custom capture selection is claim 39 and the following rows.

| Key (building type section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `NeedsEngineer` | needs-engineer flag (S-0405 claim 8), also selecting claim 39's announcement branch | boolean | no at type creation; later omission retains the value in force |
| `CaptureEvaEvent` | custom capture announcement for claim 39 | announcement entry name, case-insensitive, first 99 value characters; unknown nonempty name gives no entry | no entry at type creation; absent or empty retains the value in force |

Radar-event trigger kind bindings for G7 (S-0011 claim 32): the action's integer number
slot selects the following kind. These are map input-format values, not program identifiers.

| Number slot | Event kind |
|---|---|
| 0 | combat |
| 1 | non-combat |
| 2 | drop zone |
| 3 | base attacked |
| 4 | harvester attacked |
| 5 | enemy sensed |
| 6 | unit produced |
| 7 | unit lost |
| 8 | unit repaired |
| 9 | infiltrated |
| 10 | captured |
| 11 | beacon |
| 12 | superweapon detected |
| 13 | superweapon activated |
| 14 | bridge repaired |
| 15 | garrison abandoned |
| 16 | ally attacked |

The custom capture entry uses the announcement sample-name, separate `.wav` file and voice
volume rules of claims 26, 34 and 35, not the sound-entry bank. Claim 39 covers the ownership
changes its scope names (engineer captures, vehicle-like takeovers, and the mind-control takeover and
release), where the new owner is not passive. These rows do not define a
universal ownership-change announcement.

In the key table below, a plus sign joins the two quoted spelling fragments with no separator.
for example `Voice` + `Die` denotes one exact-case key, not two keys or an alias.

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
| `Voice` + `PrimaryWeaponAttack` | normal primary-weapon response (F6) | name |
| `Voice` + `PrimaryEliteWeaponAttack` | elite primary-weapon response (F6) | name |
| `VoiceSecondaryWeaponAttack` | normal secondary-weapon response (F6) | name |
| `Voice` + `SecondaryEliteWeaponAttack` | elite secondary-weapon response (F6) | name |
| `VoiceEnter`, `VoiceCapture`, `VoiceHarvest`, `VoiceDeploy`, `Voice` + `Undeploy` | single responses (F6) | name |
| `VoiceFeedback`, `Voice` + `Die`, `DieSound` | damage feedback, local death response and any-owner death sound, respectively (claim 20) | list |
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
| Key (weapon section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `Report` | weapon report list (claim 21) | comma-separated sound-entry names, first 127 value characters, case-insensitive references, listed order, unknown names dropped | retains earlier list, initially empty; present nonempty value replaces it |

| Key (animation section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `StartSound` | start-event sound (claim 22; S-0109 claim 13) | single sound-entry name, case-insensitive, first 127 value characters; unknown retains earlier reference | retains earlier reference, initially no sound |
| `Report` | legacy start-event alias, not an additional sound | same single-reference form; read only when the start reference after reading `StartSound` is still no sound | no sound |
| `StopSound` | end sound (claim 22) | single-reference form above; unknown retains earlier reference | retains earlier reference, initially no sound |

A valid retained `StartSound` therefore also prevents the alias from being read in a later
layer that omits that key. These animation-reference rules are high confidence, `measurable`
in F3 with distinct aliases, repeated import layers and omitted/unknown references.

Rules file, `[AudioVisual]`, sound names: `CreateUnitSound`, `CreateInfantrySound`, `CreateAircraftSound`
(claim 23), `BuildingSlam` (placement), `SellSound`,
`UpgradeVeteranSound`, `UpgradeEliteSound`, `BaseUnderAttackSound` (claim 28),
`BuildingGarrisonedSound`, `BuildingAbandonedSound` (S-0508 claims 6 and 8; claim 28), `GuardSound`, `StopSound`,
`ScatterSound` (claim 19), `ScoldSound` (refusal, claim 24), `GUIBuildSound` (build click, claim 24),
`BuildingDamageSound` and `BuildingDieSound` (also the fallbacks of claim 37),
`GUIMainButtonSound`, `GUITabSound`, `GUIOpenSound`, `GUICloseSound`, `GUIMoveOutSound`,
`GUIMoveInSound`, `GUIComboOpenSound`, `GUIComboCloseSound`, `GUICheckboxSound`,
`GenericClick`, `RadarOn`, `RadarOff`, `VoiceIFVRepair` (F6), `LightningSounds`
(list, claim 25). F6's repair-response exception depends on a particular vehicle's type name,
not its display or image name; it is not selected by an imported repair-vehicle field.
The repair-response vehicle is the vehicle type named `FV`; comparison ignores case and uses
its type name, not an image name. A mod's vehicle registered under that same name also
qualifies, regardless of its passengers or other properties; the negative-damage weapon
condition of F6 still applies.
The construction-building list tested for low power is `BuildConst` in `[AI]`; only its first
three entries are counted for this announcement.
`DeploySound` in that section is never read (claim 19). `SpeakDelay` (`[AudioVisual]`,
floating point, minutes) is the low-funds repeat *k* of claim 28, initially 2 game minutes.
Every game-wide single sound reference listed above initially means no sound, with no built-in
entry-name fallback. An absent key in a later rules layer retains the earlier value; an unknown
single sound name also retains it. The lightning list starts empty. These game-wide fields
follow S-0006's rules-database persistence across games, rather than resetting whenever a key
is omitted. Claim 38 and A5 own the initial defaults.

## Measurement scenarios

All runs: the measurement setup (glossary), with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), skirmish,
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
pitch and loop forms, B2 tests placement words, C1 tests limit. No no-effect claim is made for the other placement words; their semantics are deferred. *Falsified if:* any tested comparison differs.
F1's exact 16383 full-volume integer result is `unmeasured`, high confidence: loopback capture
cannot distinguish it independently from the later percentage conversion.

**A6. Remaining placement words and owner-only playback (Q-0260).** Named sound data gives
otherwise identical one-sample entries each remaining recognised placement word from Import
mapping, separately and combined with the established positional words. Use a timed
sound-at-waypoint request, a normal report and an unchanged-hit damage sound, each in range,
with local, allied and enemy sources where the event has an owner. Compare a screen response
control. Keep sample, level, view and explored state identical. A proposed read-only request
recorder distinguishes an absent request, a rejected instance and an inaudible instance.
Loopback records identity and stereo level. Run three repetitions per fixture at normal speed.
Alternatives are no additional restriction, an owner restriction, or an event-dependent
restriction; identical accepted requests with different playback by owner reject the first,
while any audible enemy request rejects a universal owner-only rule. Record per-word results
rather than inferring behaviour from the word's name. This exploratory scenario has no
selected alternative yet and does not weaken the established event-owner tests.

**A3. Default priority.** Defaults section priority set to critical; 17 positional instances of an
entry without its own priority are started while 16 channels are busy with a *normal* entry.
Repeat with 16 busy *high*-priority channels instead. Set per-entry limits to 0 so they do not
mask channel allocation. *Expected:* the entry without its own priority can replace *normal*
channels but cannot steal a *high* channel; an otherwise identical explicitly *critical* entry can.
*Falsified if:* the entry with no explicit priority steals a high-priority channel.

**A5. Initial game-wide defaults.** Named test rules omit the game-wide sound references
listed in Import mapping and the low-funds repeat field from every layer, including the base
rules; use a fresh program start. Keep the local house below the low-funds threshold with an
eligible factory. Use G4's distinct announcement test samples and A4's bank-backed sound entries.
Exercise selection-independent guard/stop/scatter and ordinary production/placement events,
then record low-funds request ticks with a read-only request recorder. Expected: omitted
sound references request no entry; low-funds repeats at N(1800) ticks when no other event resets
its countdown. In a second run explicitly give the references, then omit them from a later
map layer: the earlier sounds remain. An initial built-in sound fallback, permanent one-shot
low-funds behaviour or loss of earlier-layer references falsifies the corresponding rule.
Repeat three times; void a repeat-window run when any other speech-countdown reset occurs.

**A4. Sample sources.** (a) Put a loose `.wav` file named like a sample of a test entry next
to the game: expect the bank's sample, not the loose file, when the entry plays. (b) Put a loose
`.wav` file with a distinct identifiable sound named after an announcement's eight-character
name for the local voice set: expect that file when the announcement plays. (c) Name a
test entry's sample that exists only as a loose file and not in the bank: expect silence
(claim 4). (d) Repeat a bank sample reference with changed letter case: expect the same sample.
*Falsified if:* a sound entry plays a loose file, an announcement ignores it, or case alone
changes bank-sample lookup.

**B1. Screen sounds.** Assign a local unit to a group, move the view far away from it, then
recall that group without centring the view. *Expected:* the response at full level, centred. Repeat a guard interface sound after moving the view without changing the request.
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

**C1. Per-entry limit.** Also run two identical requests with a nonzero fixed pre-delay,
long samples and limit 0, close enough that the first still waits when the second is requested.
Expected: each waits from its own start opportunity; neither waits for a shared per-entry
cooldown and they can overlap once started. A second instance delayed until the first ends,
with channels available, falsifies claim 40's absence of a shared entry gap.
An entry with limit 2, with and without *interrupt*; start 3 equal
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

**C4. Instance-pool boundaries.** Use named sound data with no per-entry limit, one long
sample per entry and no random instance choices. On a quiet test map, one elapsed-time
trigger issues 299, 300 or 301 screen sound requests through S-0011 claim 32 in the same tick,
using the play-sound action's screen-request form. Use a separate run for each count.
A proposed read-only recorder logs request acceptance, live instance count and audio-pass
timestamps. Expected: 299 and 300 are accepted in full; at 301, exactly the last request fails
before channel service. Then channel allocation may discard further instances under claim 9.
That later discard is not pool rejection. Void a run if an audio pass or unrelated sound
intervenes between the test requests. Repeat with 99, 100 and 101 ordinary ground units hit in one detonation, giving each
a long damage sound and no other sounds. Named test rules give an otherwise silent explosion
weapon enough spread to reach every target and a nonlethal hit that leaves each target's damage
state unchanged. Fire it with S-0011 claim 28's explosion-at-waypoint action. Give each damage
entry global placement with a full minimum level so view range does not discard requests.
The fixed-map-position requests of claim 7, not threshold feedback, must be exercised.
The original's duplicate damage requests give 198, 200 and 202 requests respectively.
Expected: at most 200 fixed-position controllers; subsequent such requests fail even if fewer
than 300 ordinary instances exist. Use a distinct entry per unit if entry limits would interfere.
Repeat each boundary three times. Any different acceptance boundary with the fixture's
request form established falsifies claim 10. Without request/count recording, or without a
same-tick damage fixture, that pool test is inconclusive, not evidence from audible channel count.
These recorder and map fixtures are proposals, not changes to the existing harness.

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
Repeat with named test vehicle data registered under the mapped repair-response name but with
another image/passenger setup, and a differently named vehicle with the same negative-damage
weapon. Expected: only the mapped name qualifies for the dedicated response, ignoring its
letter case; unrelated passenger properties do not change the identity test.
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

**D3. Selection variants.** Select an enslaved miner, select with the type-select role held (a
held click and a held drag, S-0214 claims 29 and 32), tap the type-select role, and click an
already selected unit. *Expected:* enslaved list (or silence); no response for the held role's
click and drag; for the tap, in our engine one response, from the first object it adds, and none
when it adds nothing, while the original gives each added object its own selection response
(S-0214 claim 32), so record each response heard and each selection-response choice draw; a
response on re-click. Add a deactivated unit with a distinguishable deactivated list, then repeat
with each specialised list empty: expect the matching list, then silence without normal-list
fallback. Modifier-click a selected unit to remove it: expect no selection response or choice
draw. Recall a group: only its first newly selected object responds. Finally click a selectable
object of a passive house with a nonempty selection list. Expect one unsynchronised raw choice
value but silence and no scenario-stream choice. Repeat on ordinary enemy and allied objects:
expect silence and no selection-response choice draw. Repeat each ownership case with an empty
applicable list and with an individually selectable building: empty lists always take no draw,
and buildings retain the same ownership rule. Use one-sample responses without instance draws
and isolate each click with the random-state recorder. *Falsified if:* any differs.

**D4. Keyboard commands.** Guard, stop, scatter and deploy keys with a unit selected. *Expected:* the
three game-wide sounds, and silence for deploy (apart from the unit's own deploy response).
Use named test rules assigning distinct one-sample entries and a read-only request/order recorder.
Repeat with nothing selected, only a building selected for scatter, and three qualifying units:
expect respectively no command cue, one scatter cue despite no orders, and one command cue
rather than three. Repeat with a pending-order list full under S-0213's limit: a failed issuance
suppresses the cue. The planning-mode guard control requests no ordinary guard cue.
*Falsified if:* a request count or order-failure gate differs, guard/stop/scatter requests the
wrong entry, or the deploy key plays a game-wide sound.

**D5. Specialised unload/deploy responses (Q-0265).** D2's named distinct response entries,
quiet map and read-only request/order recorder. Separately test a building that unloads a
harvesting vehicle, ordinary vehicle unloading, and an infantry or vehicle deploy cycle.
Request another legal deploy/unload while the earlier cycle is pending and again after it
ends; log the issued order kind, cycle state, queued response entry, unsynchronised choices
and eventual playback. Distinguish the building's harvest, deploy and move responses in the
fixture. Repeat three times for each reachable control at normal speed. Alternatives for the
building are F6's ordinary deploy response, a harvest response during unloading, or silence.
For repeated deploy they are a response on each accepted request or state-dependent
suppression. An observed queued entry or choice count different from a candidate rejects it.
An input that issues no order does not establish a response-suppression gate. This is a
settling test, not an extension of F6; unreachable states remain unresolved rather than
being created by a state-writing recorder.

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

**E4. Endpoint token forms (Q-0334).** Named test sound data assigns one short sample per
entry, with pre-delay and delay values `400,800`, `400, 800`, `400 x 800`, `400ms 800`,
`400 800 1200`, and equivalent space/tab-separated controls. Record endpoints with a read-only
loaded-setting recorder, and loopback start latencies and pitch over 30 isolated requests per
fixture. Expected endpoint pairs: (400, 400), (400, 800), (400, 0), (400, 800), (400, 800).
The nonnumeric second token does not repeat the first. For pitch, `-50,-50` gives (-50, -50),
so playback is at half rate. Extra numbers never change either endpoint. Endpoint mismatch
falsifies claim 2; delay timing alone is inconclusive where endpoint reversal or pass latency
masks a difference. Apply the same token comparisons to both endpoint fields.

**E3. Settling delayed draw totals (Q-0260, Q-0329).** Named test sound data: two distinct attack
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
selection is retained across loops; *all* has no body repetition within a pass. With neither
*pre-delay* nor *ambient*, expect no explicit initial wait and a fresh delay choice at each
sample completion, including the terminal completion. With those words present, expect claim
13's initial wait and conditional terminal-delay omission, not a universally reused delay.
With decay enabled, expect decay candidate choices before attack/body preparation choices,
including candidates that do not yet become audible. Vary only the decay range first, then
only the attack, body and delay ranges to discriminate choice order using the recorded stream.
An initial wait without either word, no inter-sample delay on the 64-ms fixture, a reused
range choice, or decay choices occurring only after all body preparation falsifies those
established parts of claim 14. *Open alternatives:* the number of discarded decay candidates
across reloads, the complete audible attack/decay sequence and startup ordering, and which
terminal waits actually delay instance end. Count and identify draws and end timestamps against
each alternative; a mismatch falsifies that candidate sequence.
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
value when spoken). A death: death response (local only) then
death sound. *Falsified if:* the share is far from 30 % or the chance is drawn on other results. Repeat the threshold sequence on enemy-owned test vehicles,
with feedback and death lists referring to distinguishable entries that have no instance draws:
expect the feedback chance draw but no spoken feedback, and a death-sound choice without a
death-response choice. Local deaths choose the response before the death sound. Include
*unaffected*, *now red* and a berserk hit: expect no common reaction. Named test data provides
one distinguishable damage sample with no instance draws, a high per-entry limit and free
channels. For the departure test, run an isolated *unchanged* hit on our engine, recording
requested instances as well as loopback playback: expect exactly one positional damage-sound
request, following scrolling as in B3. Zero or multiple requests for an eligible hit falsify
claim 20's fixed behaviour; inaudible or suppressed playback alone does not. This part is
conformance testing against this spec, not measurement against the original. For the 200 local yellow crossings, use an exact binomial test at significance 0.01
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
open question; it does not yet select one candidate as the requirement. Recycling-building
consumption is tested separately by S-0603 claim 7 and its scenario G10; it is not evidence
for an additional reaction to an infantry death.

**F2. Report variant.** Build 20 copies of a unit whose weapon has several report sounds; each copy
fires many times. *Expected:* each copy always uses one variant, and each variant equals F5 applied to the recorded creation draw's low 16 bits. Use the
random-state recorder, not an injected seed; a repeat from the same saved game should keep each
existing object's variant. *Falsified if:* a copy
changes variant, or the variant differs on replay. Repeat with named report lists whose
entries contain no instance draws, using a gattling-flagged unit and infantry firing from an
occupied building. For the gattling unit, use S-0609 claim 13's G10: expect one fresh
unsynchronised raw list choice when a spin-up finds the playing mark clear or raises the
stage, only once when both apply, and none on other spin-up updates or per shot.
Use distinct ground and anti-air lists and elite replacements: the list must belong to
slot 2*s* after the update and elite replacement, even against air. An empty list takes
no draw. G10 also tests restarting after spin-down and map exit, and no new choice solely
because playback finishes. Occupant fire takes one fresh unsynchronised raw choice per shot.
The ordinary report must add none; its variant still matches the creation value.
For our engine only, save the 20 existing objects, reload without creating replacements,
and record their retained report values and deterministic state hashes before and after.
Expected: values and hashes survive the round trip, and presentation of each restored object
obtains its retained value without a new random draw. In an engine-only controlled state
fixture, vary just one retained report value: the hash must change while gameplay outcomes
and both random streams match the unaltered fixture throughout subsequent ordinary firing.
Only that object's report identity may differ. A lost value,
extra draw, presentation substitution or unchanged hash falsifies claim 32's design contract.
These are conformance tests, not oracle observations.
*Falsified if:* an exception uses a fixed creation variant or the scenario stream,
a gattling shot adds an ordinary report, a spin choice occurs without an eligible trigger,
or the spin-sound list or choice count differs from the stated rules.

**F13. Special-effect reports and report position (Q-0264).** F2's quiet-map setup and named
report entries without instance draws. Fire a sonic-effect weapon using its legal firing
setup. Give the weapon's report list identifiable entries and record effect-creation times,
every requested report entry and position, both streams, and stereo playback for three
repetitions at normal speed. Compare ordinary firing and S-0615 claims 12, 15, 16 and 18's
already specified disc-laser timing and variant as separate controls. For the sonic effect,
alternatives are claim 21's fixed variant only, a fresh unsynchronised list choice at effect
creation, and an additional effect-specific request. Each candidate is rejected by a different
recorded request or choice count. Do not count an animation's independently requested sound
as a second weapon report. In the ordinary control use a unit with a visibly displaced muzzle,
record its shot's fire point under S-0313 claim 1 and the report position, and pan the view
to put that displacement across the view edge. Claim 21 predicts the firing position, not
a screen request or target position. A different recorded position falsifies that prediction.
The sonic-effect classification remains unmeasured until these alternatives are discriminated.
This does not reopen the disc laser's feature-owned timing and variant.

**F3. Impact.** A weapon whose warhead creates an explosion animation with a start sound, fired at
ground. Use named test art to give the animation distinguishable start and stop samples, with
an otherwise silent weapon. *Expected:* the start sample at the animation's start event and
the stop sample at its end, placed at the animation. Repeat with both animation sounds absent:
expect no impact sample. *Falsified if:* a sound with no animation sound configured, a missing
start/stop sample, or a screen-centred sound independent of animation position.
For import precedence, give the animation's start reference and legacy alias distinct samples.
Expected: the valid start reference wins, never both; a newly imported animation without a
valid start reference uses its valid alias. In a later import layer omit or give an unknown
start reference while retaining a valid earlier one: that earlier reference remains and the
new alias is not used. Omitted or unknown stop references likewise retain an earlier valid
stop reference. Record loaded references and requested entries as well as playback; a different
retained reference or two start requests for one start event falsifies the import rules.

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
Repeat local factory production for infantry, vehicles and aircraft, using named test rules
with a distinguishable type creation sound, then with that reference omitted from every
layer. Expected: the type sound when present and the matching game-wide kind fallback
otherwise, never both. A wrong kind fallback, both sounds, or silence with a valid fallback
falsifies the local production rule. Non-local production is not tested here.
Repeat building placement and ordinary wall sale after scrolling: their screen sounds retain
level and centre pan. Packing a building up omits the sell sound but retains the pack-up sound.

**F5. Interface sounds.** Exercise each production-feedback click of S-0411 claim 12 (start,
queue, resume, hold, cancel) and the busy-line refusal (a building click while a building is in
production), then a refused placement click, then a production request when the queue is full.
Space actions until all earlier announcements have finished and their waiting queues are empty.
Use distinguishable named test samples for the build-click and refusal entries and the two
refusal announcements; capture requested entries as well as playback with a proposed read-only
recorder. A suppressed announcement is not evidence of an absent request.
*Expected:* the build click entry for all five production clicks; the matching announcement
for start, resume, hold and cancel, but no new building/training request for a queued copy
added to an active line (claim 28, F11); only the "unable to comply" announcement for the
busy line; only "cannot deploy here" for the
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

**F10. Scroll-button feedback (Q-0339).** Named test sound data makes tab-scroll and refusal
entries distinct. On a tab with two extra rows, press each page-scroll button at its end and
with exactly one row remaining; then repeat on a tab whose icons fit. Record dispatched input,
scroll position and requested entry using a read-only recorder, not only audible playback.
Expected for dispatched page-button requests: refusal only when already at that end,
tab-scroll sound on accepted partial movement, and no additional refusal when movement
reaches the end. A fitting strip refuses a dispatched request conditionally, but S-0411
claim 18 disables its buttons, so ordinary clicks there dispatch nothing and request neither
sound. Repeat holding each button, releasing over it and releasing outside it: expect one
request only on release over the pressed button, without held repeat. Use the wheel at
both ends and with room remaining: expect no tab-scroll or refusal request. S-0411 claim 18
and S13 own dispatch and row movement. Different feedback on an identical dispatched
page-button request, sound from a disabled-button click or sound from a wheel event
falsifies claim 24; silence without a dispatched request does not falsify the conditional
refusal rule.

**F11. Production announcement kinds (Q-0340).** G4's distinct test samples and a read-only
request/order recorder. Start, hold, resume and queue infantry, vehicle, aircraft, naval-unit,
building and defence items, spacing tests until earlier speech queues empty. Expected: infantry
start/resume requests training, every other kind requests building. Add a second queued copy
while the first is active: expect a build-click sound but no new building/training request.
A different kind, or a new start announcement on that active-line queue click, falsifies claim
28. Void a start/resume case if the click does not reach the matching S-0411 action row.

**F12. Turret lifecycle (Q-0337).** Named test rules give a turreted vehicle a long looping
rotation entry. Turn, stop, turn again before the old sample finishes, then sustain a turn.
Record rotation samples and sound requests/stops at object updates through a read-only recorder,
and sample boundaries with loopback capture. Expected: no grace counter at the first nonturning
sample. A new sampled turn starts a fresh instance even while the old one finishes, so they
may overlap; continuous turning does not restart it every tick. Compare a short turn captured
by a rotation sample with one entirely between samples: only the former requests a sound.
A cut at the stop alone, loss of the old sample solely on restart, grace delay or per-tick restart falsifies claim
23. Void the between-samples case unless the recorder establishes both motion and sample timing.

**F6. Lightning sound.** Fire the weather superweapon; record the scenario stream per tick
and strike positions with a read-only recorder. Repeat with a named test rules file giving
an empty lightning sound list: expect no sound-choice value, without deleting the bolt or
explosion animation's own draws. An extra sound-choice value falsifies that guard.
Compare consecutive recorded strike positions. To obtain exact repeats, use a tiny named
map with the lightning-strike-at-waypoint trigger of S-0524 claim 2, requesting two strikes
at the same waypoint with enough time between them for each cloud to strike. Record 20
pairs, including each bolt, explosion and damage event and the scenario-stream state.
*Expected:* for every strike with a nonempty sound list, including the second of each pair,
one raw value after the bolt animation's creation draws and before explosion creation and
damage, whose modulo selects the audible entry. Repeating the point suppresses neither that
choice nor later strike effects. Account for animation draws using S-0109 and damage effects
using S-0302; replay the logged seed's mathematical stream rather than setting a seed inside
the original. *Falsified if:* the choice or its position differs, or an exact repeat suppresses
that choice or later strike effects. If no exact repeat is recorded, the repeat comparison
remains inconclusive.

**F7. Building fallbacks.** Damage a local building whose type names no damage sound through the
yellow and then the red threshold, then destroy one whose type has an empty death list. Then
repeat with named test rules giving the type its own damage sound and death list.
*Expected:* without own sounds, the game-wide building damage sound at each threshold crossing
and the game-wide building death sound at destruction, positional at the building and following
scrolling when the view moves during the sample. With own sounds, neither fallback plays.
Claim 20 does not promise the type's own damage sound at a threshold crossing: add a separate
*unchanged* hit to exercise its damage-sound event. Test that event's single-instance
requirement on our engine under F1's departure control, not against the original.
Destruction uses the type's death list.
*Falsified if:* an expected fallback is absent, a fallback plays with an own sound present,
the unchanged-hit or death choices differ from claim 20, or a fallback is heard at screen level.
Repeat threshold and lethal hits on an enemy building, with a long destruction animation
separating the lethal hit from removal. A read-only request recorder should show one fallback
request on the lethal reaction, not on later removal; the owner does not suppress it. A later-only
request or enemy-only silence falsifies claim 37. Sale and script-removal controls remain
exploratory, not a prediction that they invoke lethal-damage feedback.

**F8. Repeated route-mode refusal.** Exceed the route-mode limit of S-0213 claim 16 twice,
some seconds apart, in one running game; log completed ticks between the clicks. Then start
a new game and exceed it again. Record messages and the refusal entry separately from
announcement playback. *Expected:* message and refusal sound on all three refusals.
*Falsified if:* the later refusal after completed ticks is silent or lacks its message, or the
new game's refusal gives neither cue. This does not test multiple notices before one tick
completes; coalescing within one tick is outside claim 24's coverage.

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

*Part T (Q-1735).* Settings: a skirmish on a small map against one computer player, played once
as a country of each of the three voice sets, with no helper that changes rules, houses or
computer players during the game active. A test map trigger shows a message of its own once,
early in each game, as the control that the recording captures the message area. First open the
game's options and list every setting it offers. *Expected:* none concerns announcement text.
Then play with every display option the listing finds that can show text on, at full voice
volume; then again with all of them off; then, when the listing finds more than one, once with each of
them on alone; then once with voice volume zero. If the listing finds no such display option,
play one game with voice volume full and one with voice volume zero. Each game of these sweeps
is played once under each of the three voice sets, so every display-option and volume setting
is seen with every voice set. In each game build a structure
and watch the battlefield, the message area and the sidebar when the construction-complete
announcement plays, and likewise for two other announcements (for example low power and unit
ready). *Expected:* the control message appears, and no text appears with any of the three
announcements, in any of these settings (with voice volume zero, nothing is heard and still
nothing is shown). If a line does appear when one of them plays, repeat that game with a named
announcement test copy that removes that announcement's entry: a line that still appears is the
event's own (claim 26) and that announcement is replaced by another for part T; a line that no
longer appears is announcement text. Repeat the first setting with a named announcement test
copy that changes the text key of every listed entry to distinct words, adding it where it is
absent, and removes it from one of the three watched entries, chosen where possible as one that
carries a nonempty text key in the stock data; that entry is compared with itself in the first
setting and with the other two watched entries of the same copy: *expected* the same
announcements, heard the same, and still no text, with the three watched ones and with any other
announcement that plays during the game. Repeat the first setting once more with a second named
copy that sets the text key of every listed entry to the name of an existing entry of the game's
string data, a different entry for each announcement, so that a lookup of the value as a
string-entry name would find words to show: *expected* the same, still no text. *Void if:* the
control message does not appear. *Falsified if:* an option for announcement text exists, text
that the entry removal shows to be announcement text appears in any setting (in particular only
with one display option on, or only with the voice muted), or either text-key copy has any
effect.

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
(d) Repeat (a) before the local player has a radar building, using S-0410 claims 14–15
and R13's read-only zoom/lifetime recorder. Expect the same drawn-frame zoom progression
and post-completion 200-tick tail as with radar, not a restart when radar becomes available.
Use R13's freeze and stand-still controls to distinguish drawn-frame zoom from tick-based
tails. Suppression past the predicted zoom completion plus tail, or a restart on restoring
radar, falsifies this prediction.
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

**G7. Radar-event suppression table (claim 28, F7).** Use S-0011 claim 32's radar-event
at waypoint action, with the numeric event-kind bindings in Import mapping below. One kind
per quiet-map fixture, three repetitions at normal game speed. Keep radar visible. A proposed
read-only recorder logs accepted events, kind, cell, zoom completion tick and removal frame.
Video records the displayed squares. Schedule two same-kind requests on the same tick at
separations 0, r − 1 and r cells for every suppressing row. Repeat a diagonal pair whose
truncated Euclidean distance lies just below r, and one at r. Expected: exactly one event
below r, two at r; a different kind never rejects the second. For each non-suppressing row,
request two events at the same cell: both must be accepted. No radius is asserted for those rows.
For every row, create an isolated event and log its removal: it must remain until zoom completion
plus the row's tail and be removed at the next eligible expiry-processing opportunity under
S-0410 claims 14–15. Compare acceptance of a new same-kind request before and after removal.
For suppressing rows it must be rejected only while the first exists. Record update ordering
at the expiry boundary rather than assuming which request sees removal in that frame.
A wrong distance boundary, same-cell rejection in a no-suppression row, or a lifetime different
from the predicted completion-plus-tail falsifies that part of F7. Unrelated events of that
kind or unobserved zoom completion void the corresponding comparison.

For cross-creator controls, first create a trigger event at the test factory, damaged building,
harvester, repair depot or civilian garrison building. Then repeat G3's production/base/harvester
event, S-0314 claim 6's successful repair completion, or S-0508 claim 8's abandonment, at a
separation within the matching radius while the trigger event exists. Expected: the ordinary
creator's event and its gated announcement are suppressed. After expiry, the same feature
event is accepted. Repeat in reverse: ordinary creator first, trigger request second. Expected:
the trigger event is rejected while the first exists and accepted after removal. Use a read-only
announcement-request recorder, not missing audio, to identify suppression. Space other speech
out of the test window. This tests these named creator pairs only, not every possible creator.
An accepted matching event inside the live radius, or continued rejection after expiry,
falsifies the cross-creator rule for that pair. Void a control if the ordinary feature event
is independently ineligible, has a different cell, or cannot be generated by the permitted map.

**G4. Announcement entry bindings.** Named announcement test data gives each fixed entry
in Import mapping a distinct brief sample for every voice set. Use the mapped names, not
roles inferred from baseline recordings, so this test can falsify the binding itself. Raise start, queue, resume, hold and
cancel clicks, a busy building-line click, refused placement, construction completion, a new
sidebar option, unit ready, building and unit sales, completed repair, promotion to each rank,
low funds, low power, a local ordinary vehicle's lethal hit for unit lost, and base, ally and
harvester attacks. For the ally control, pre-place an ordinary non-wall building owned by a
named allied bystander in the observed area; a named enemy attacker damages it. Those test
objects alone are exempt from the bystander observed-area void rule. Use the same building
type for the local and allied controls, with enough health to survive. Space events beyond F7 suppression
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

For the additional fixed bindings, select an own primary factory, set its rally point, use
S-0508 G4 and G6's civilian garrison entry and building-turn abandonment controls, switch an
own power-switchable building offline and online, capture an
ordinary enemy building and lose an own tech building. Capture's generic-event control omits
any custom announcement override. Record requested entry identities as well as samples.
Expected: each role's mapped entry, independent of local faction. For each superweapon category
in the mapping, use a separate purpose-made skirmish with a pre-placed charging instance.
record its ready request, then activation/launch requests. For each mapped detection category,
observe a non-allied computer player's ordinary placement of its building. Obtain a saved
stock skirmish just before that observed placement for each category, then repeat its
continuation three times with the named announcement samples and request recorder. The
named placing house/building are exempt from the bystander void rule for this control.
Other bystander interference still voids it. Do not try to give human orders for that house
or substitute a building-restoration trigger for placement. The weather detection control must request the
mapped detection entry, not a name inferred from the ready entry. Clear prior announcement
queues before each control, and use three repetitions. A different requested entry on an
eligible event falsifies the binding. A run that raises no request because its specialised
eligibility was not met is inconclusive, not proof of silence; unestablished activation cells
remain exploratory under the explicit Q-0325 limits and G6 settling scenario.

S-0414 claim 5 and V20 own observer eligibility and post-defeat muting, with their own
limits. No alliance-selection test or binding is asserted in this spec.

**G5. Capture entry alternatives (Q-0325, claim 39).** Use G4's named announcement data,
with distinct brief samples for the generic capture, tech-building-lost and two invented
custom entries. Named test map overrides give otherwise capturable building types the
needs-engineer flag either on or off and the custom reference either present,
absent or unknown. Keep all prior announcement queues empty. Capture a bystander's building
with the local player's engineer, then arrange the reverse ownership change using a named
bystander engineer; that unit is exempt from the observed-area void rule. Repeat three times
per fixture and as each local side. A proposed read-only request recorder logs eligibility,
requested entry and owner change, alongside loopback capture. Expected: on gaining a flagged
building, the configured custom entry, or no entry when it is absent/unknown; on losing one,
the tech-building-lost entry. An eligible change on an unflagged building requests the generic
capture entry even when a custom reference is configured. Faction changes sample voice set,
not requested entry. A generic fallback in the flagged/missing-custom control, custom entry
on the unflagged control or different entry on an eligible event falsifies claim 39.
Void a run when capture does not occur or a competing announcement masks request observation.
Ownership changes that the recorder establishes as ineligible are inconclusive for entry
selection, not evidence that an entry binding is absent. For a mind-control takeover and release of a
flagged and an unflagged building (S-0601 scenario MC9), the same expected entries apply; record
them as a separate fixture from the engineer capture.
For a vehicle-like takeover, the same expected entries apply under S-0509 scenario E12's setup (a
test-map override gives a 1 × 1 building type an undeploy-into vehicle, and a vehicle thief takes it),
with the needs-engineer flag on and off, recorded as a further separate fixture. As in the base control,
the local player's vehicle thief takes a bystander's building, and a named bystander vehicle thief then
takes it back; that thief is exempt from the observed-area void rule.
Exploratory extension, outside claim 39: transfer an own building to a passive house, with
flagged and unflagged types, using S-0011 claim 24's ownership-change trigger. Compare an
otherwise matching transfer to a non-passive house. Investigate the hypothesis that a passive
new owner suppresses both capture branches; either request would falsify that hypothesis.
Silence is inconclusive unless the non-passive control independently enables announcements.
This is not yet a validated way to isolate the passive-owner condition or a conformance rule.

**G6. Settling specialised superweapon announcements (Q-0325).** Use G4's per-entry distinct
announcement samples and one isolated superweapon category per purpose-made skirmish map,
normal game speed and three repetitions per control. Observe the local house's charging and
ordinary activation, and a named enemy bystander's superweapon-building placement. A proposed
read-only recorder logs every announcement request, its entry identity and local muting state.
Loopback capture alone cannot distinguish no request from a queued or muted request.
For each unestablished activation cell in Import mapping, record whether an entry is requested
and which one. No-entry, a fixed category entry and an indirect feature-specific entry are
unresolved alternatives, not established silence. A recorded request rules out no-entry.
Repeatable differing identities under identical controls rule out a single fixed binding.
For mapped categories expect the mapped entries whenever a request occurs; another identity
falsifies the binding. Vary owner (local, allied, non-allied), ordinary versus passive owner,
announcement muting and presence of the category's auxiliary prerequisite building in separate
named test controls. Keep the category, charging and placement events otherwise unchanged.
Record eligibility per control; those observations investigate deferred coverage rather
than treating the common-event table as exhaustive. If an event cannot be generated by the
permitted setup, that control remains inconclusive, with its unreachable condition named.
Do not fill an unestablished binding from a sample name or faction voice set.

**H1. Sound does not reach the game.** Use a saved quiet-map skirmish with local ground
units, a factory finishing a unit, a local harvester taking nonlethal damage, and named enemy
test attackers/targets separated from the bystander's other objects. Those named test objects
are exempt from its observed-area void rule. Continue it for 900 ticks with scripted move,
attack and production orders, recording the applied order identities and ticks. Make two
continuations per save: effects volume zero versus full, with different view positions but
the same applied orders. Adapt scripted screen coordinates to the view, not the orders.
Repeat three pairs at each of speed indices 0, 2 and 6. Read both random streams and record
unit positions, health, counts, credits and production progress per tick; do not set a seed.
Expected: identical scenario stream and game-state observables per tick in each pair.
Unsynchronised state may differ: placement and playback choices follow presentation timing.
This comparison must not assume that only per-instance draws differ. Any scenario-stream or
game-state difference falsifies claim 29's prediction for these controls. A different applied
order, unrelated bystander action, or loss of a surviving test object before its named event
voids the pair rather than counting as an audio-caused mismatch.

**H2. Engine-only scheduling determinism.** Conformance proposal, not an original-game measurement.
Use E1's immediate-mode test entries and D2's order-response fixtures together, with explicit
initial states for both streams. Replay one identical cycle-indexed sequence of game events,
game-speed changes, requests, view changes and local input, with the options menu's clock readings
(claim 30's Music time) wherever the menu is open with focus, in three runs: ordinary host load,
added host delays, and delayed audio-device callbacks. A test driver must keep those logical inputs
identical while varying only host execution and delivery timing. Record chosen entries, pitch/shift
values, sample order, every stream draw's assigned use and both stream states at matching logical
service opportunities. Expected: identical assignments and states in all three runs; only device
delivery timestamps may differ. A changed assignment or state caused solely by host timing falsifies
claim 30's design requirement. Changed logical input voids the comparison. Also check these
canonical controls:
- From a fresh clock, run 20 cycles at each constant speed 0 to 6. Expect passes every
  5, 3, 2, 1, 1, 1 and 1 cycles respectively, with the logical intervals of claim 30.
  At speed 1 after two cycles (*C* = 32, *L* = 0), change to speed 0: the next cycle
  passes at 40 ms, rather than resetting the cadence.
- At speed 0, start a 100-frame, 1000-frame-per-second sample with pitch 100 at the
  first pass, 40 ms. Expect its completion at 160 ms, not 120 ms or a callback time.
  A 33-ms effective wait selected at 40 ms resumes at 80 ms. A 40-ms wait resumes at
  120 ms, not 80 ms, because expiry is strict. A below-threshold no-wait choice can
  start a sample immediately, but cannot complete that sample in the same pass.
- Put two due completions and two new requests in one pass, with different request
  numbers, while a selection and a feedback event occur in that cycle's tick. Record
  draw-use labels: the tick's event choices precede completions in request-number order,
  which precede surviving allocation/preparation choices in request-number order.
  Repeat with a game-side cut before the pass: the cancelled completion takes no draw.
- Insert ten frozen cycles: expect clock and audio progress with unchanged tick number.
  Insert a lost-focus stand-still: expect no logical progress or draws however many
  host frames/callbacks occur; closing it resumes at the next cycle with no catch-up.
  Then open the options menu, lose focus with it open for a supplied stretch longer than the
  time left in the current track, and regain focus: expect music time unchanged across the
  unfocused stretch, no reading, no music-only pass and no draw while focus is lost, and the
  reading at the return taken as the new previous reading, adding nothing to *M*. The track
  then completes at the first pass, music-only or audio, with *M* ≥ *M₀* + *T*, where *M₀* is
  its start time and the unfocused stretch adds nothing to *M*. A completion or pick during the
  unfocused stretch, an increment that includes it, or a completion before *M* reaches
  *M₀* + *T* falsifies claim 30's focus rule. Insert
  an options-menu stand-still with no track due to complete in it: expect no sound progress
  or sound draws, music-only passes as the supplied clock readings give them, and resumption
  of the ordinary schedule at the next cycle with no catch-up.
  Test two announcements with known decoded durations: their starts obey both completion
  and the strict 500-ms start gap, rounded to the first qualifying pass.
- Music controls, with S-0703 MU7's music fixture (shuffle on, test tracks of known length) and
  a log of each pass's logical time and of each music update. Expect exactly one music update
  per pass, its ranged draws after that pass's sound-choice draws, and none in a cycle without
  a pass. Bring a test skirmish to its end by a timed defeat with a known end delay (S-0414
  claim 8): expect passes and music updates in the end delay's cycles as the cadence gives them,
  a pass at the end of the cycle that runs the end delay's final tick only if the cadence puts
  one there, and no pass or music update after that cycle. Open the options menu for many
  host frames just before a track would complete, the test driver supplying the menu's clock
  readings as logical input, identical in all three runs: expect a music-only pass at each
  reading at which *M − K > 33 ms*, the track's completion and its pick at the first such pass
  with *M* ≥ *M₀* + *T*, the next track started there while the menu is open, the pick's ranged
  draws as the only draws while the menu is open, and no sound progress until it closes.
Any different pass, deadline, ordering or stand-still result falsifies the design rule.
Use positive-rate, positive-pitch fixtures, plus one invalid-rate and one nonpositive-pitch
fixture to check warning/discard without additional sample-preparation draws. These are
engine-only named test data, not stock values or oracle tests. The test driver is a proposal,
not a change to the existing harness.

**I1. Volume options.** Set the sound volume option to zero and the voice option to full: expect
announcements at full and no audible sound instances, responses included; then the reverse: expect
sounds and no announcements. With C3's sixteen loud entries playing, expect an announcement's
level unchanged while sounds are lowered. Named test sound data gives the credit-tick entry and a
selection response entry the same sample at volume 100, with no shifts: expect the credit ticks
10 dB below the response at the same option setting. *Falsified if:* an option scales the other
group, the crowded level lowers an announcement, or the credit-tick difference is not 10 dB
within one F4 step.

## Open questions

Q-1735 is answered by claim 26 and G1 part T. Question 1: an entry's written text, where it has
one, is its text key (Import mapping), and the importer reads that key's value as the caption
text itself, not as the name of an entry of the game's string data. In the original's data some
announcement entries carry that key, with literal words or empty, and most carry none; no string
entry is tied to an announcement by name, and the original looks none up (claim 26, `unmeasured`,
medium). In the original a missing text key or a missing string entry has no observable effect,
since the original shows no announcement text at all. Question 2: in a skirmish the original never shows the text of an
announcement, with any option. Playing an announcement shows no words, and the original
offers no option for announcement text (claim 26: `measurable` with part T's
settings; `unmeasured`, medium, for the other settings, for the reason given there). So where,
for how long and how many in a row do not arise in the original; for our engine's caption they
are interface choices, and the engine's current values (4 seconds, at most 3 at once, newest
last) stand as those choices (claim 26). Campaign missions and network games lie outside the question's
skirmish scope and outside this spec. Question 3: nothing shown depends on the voice set. The
caption rule is our engine's design.

The Summary's deferred coverage is not a collection of provisional implementation rules.
E3, F9 and G5/G6 preserve exploratory scenarios for later revisions; they do not close
Q-0329's complete delayed sequence or Q-0325's specialised eligibility/binding gaps.
No immediate-mode, zero-wait or silent substitute is established for a deferred case.

Q-0671's clock, completion/wait opportunities and same-cycle draw assignment are supplied
by claim 30 and H2 as engine design rules. Claims 12 to 14 and 27 retain their original-timing
meaning. Claim 14's complete delayed-mode sequence remains independently deferred: the
replacement clock neither resolves it nor authorises an immediate-mode substitute.

Within the retained scope, four precision/observability limits remain `unmeasured` with
reasons at their claims: the corner-cell exclusion (claim 6), fractional channel tie/order
rules (claim 9), the exact full-volume integer (claim 2/F1) and intermediate gain truncations
(claim 15/F3). Their observable coarser consequences retain their stated scenarios.

Claim 14's regular playback rules assume nonnegative attack/decay counts with at least one
body sample, nonnegative delay endpoints and nonnegative loop count. Negative or oversized
partitions, negative limits/ranges and negative delays are outside this spec's coverage.
The no-body quirk is a particular tested exception, not a general bad-data rule. Claim 37
covers lethal-damage reactions, not sale, crushing or script removal.

## Changes for other specs

Glossary, for the project lead's acceptance of music time in the options menu (claim 30; S-0703
claim 13):
- *departure*, "Host or real time": add at the end "One exception has been accepted by the project
  lead: while the options menu is open in a loaded skirmish, the music plays on as in the original,
  and our engine advances music time with the host's monotonic clock, with lost focus still a full
  stand-still (S-0701, claim 30). Only presentation depends on it, the unsynchronised stream included: when a track ends in the menu
  and which tracks are picked there; when the track playing as the menu closes ends afterwards, and
  so when every later track starts and ends; and, through the music picks' draws, the position of
  the unsynchronised stream at later sound choices. No tick, game state or scenario-stream draw
  depends on it."
- *audio clock*: replace "The glossary's *departure* rule remains in force." with "The
  glossary's *departure* rule remains in force, apart from the music time of the options menu
  that S-0701 claim 30 defines (see *departure*)."

Q-0264 companion import clarification:
- S-0109 Import mapping: include the animation start-sound reference and stop-sound
  reference as well as the existing legacy start-sound alias. Their exact input bindings
  and fallback precedence are specified in this spec's Import mapping. Keep S-0109 claim 13
  as owner of start-event timing; these bindings do not add a second start sound.

Q-0671's schedule corrections below are already present in the approved companions.
Retain them at joint hand-off, with no further change requested:
- glossary, *audio clock*: retain the original-only real-time definition, but replace the
  statement that claim 30 defers the schedule with a pointer to its logical cycle-end clock,
  completion deadlines and waits.
- S-0202 claim 11: replace its statement that S-0701 claim 30 defers the schedule with a
  pointer to the now-specified cycle-end passes, deterministic completion/wait opportunities
  and game-side-before-pass draw assignment. Retain the shared unsynchronised stream and
  attribute host-dependent service interleaving only to the original. Complete delayed-mode
  sequence coverage remains deferred under S-0701 claim 14, not under the schedule.

The following earlier scope/coverage repairs are already present in the current companion
drafts; retain them at joint hand-off, rather than treating them as new outstanding edits:
- S-0314 claim 6 and its suppression pointers: cite S-0701 claim 28's G7 for the
  *unit repaired* row and named trigger/depot creator pair. These are now measurable
  proposals, not measurements or exhaustive creator coverage. Keep S-0410 claims 14–15
  as zoom-timing owner and retain the post-zoom tail.
- S-0410 claim 11 and R13: replace the blanket assertion that remaining suppression
  rows are unmeasured with S-0701 claim 28's G7 coverage of all rows through trigger
  requests and the explicitly named cross-creator pairs. Do not imply all creators are covered.
- S-0202 claim 11's sound-draw backing: replace the inherited "unmeasured complete
  delayed-mode sequence" with deferred coverage under S-0701 claim 14. Its partial
  choice rules remain measurable in E1/E3; no complete sequence is supplied.
- S-0414 claim 12: S-0701 claim 36 does not specify request-versus-mutual-alliance
  announcement selection or its bindings. Keep independently supported presentation and
  muting rules in S-0414 claims 5 and 12–13, but mark that selection/binding detail deferred
  rather than claiming S-0701 supplies it. Victory/defeat bindings remain specified here.

Existing companion agreements to retain:
- S-0213 claim 16 agrees with S-0701 claim 24 that a completed tick permits a new route-mode
  refusal cue; there is no once-per-game silence rule.

Further items for other specs:
- S-0411: state the infantry-only training versus other-kind building split for start/resume,
  and no repeated start announcement when adding to an active queue (claim 28, F11).
  Q-0339's dispatch is already specified in S-0411 claim 18 and S13; claim 24/F10
  here retain sound-feedback ownership. No additional dispatch change is requested.
- S-0213: point keyboard-command feedback to claim 19/D4, including empty selection silence,
  nonempty-but-ineligible selection feedback and suppression on failed order issuance.
- S-0206: distinguish tick rotation samples from audible instance lifetime; claim 23/F12 own
  the no-grace end-loop request and fresh-instance restart that can overlap a finishing one.
- S-0302: point building threshold/death feedback to claim 37/F7 here rather than an unwritten
  structures rule; the fallback request is any-owner and occurs in the lethal reaction.
- S-0410 claims 14–15 already specify Q-0331's zoom progression, duration and cadence,
  with F9 and R13. F7 and G3 here now cite them; no further zoom change is requested.
- S-0202 claim 10's current Q-0327 contract already agrees with claims 21/32 here:
  preserve, save, restore and hash the creation-derived report value, with presentation access
  for existing objects and no extra draw.

These hand-off checks include corrections already present in other drafts; they are not
instructions to undo those corrections. S-0214's current claims 12 and 14 and scenarios T1/T2
already agree with claims 17/18 here. S-0411's current claim 12 agrees with claim 24 and F5:
busy-building refusal and refused placement use distinct announcements and neither uses
the game-wide refusal sound. No cue is added for its unresolved actions.
The approved S-0406 reviewed here agrees with claim 24: its claims 1 and 10, State and
ordering, and refusal measurements specify the cannot-deploy-here announcement request
only for failed placement, without the game-wide refusal sound. No change to the approved
S-0406 is indicated by this comparison. An ignored place order without an eligible factory
remains silent.
Production/payment, offer membership/capacity and cancellation rules remain owned by
S-0404, S-0407, S-0004 and S-0411, not by this audio revision.

Other existing hand-off checks:

- S-0214 claim 14, F5 and Import mapping: the otherwise-used order response is the
  special-attack list, not the selection list. Add claim 18's selection ordering and claim 17's
  enslaved/deactivated selection rules without normal-list fallback.
- S-0213: use that same selection order wherever it orders a selection; S-0214 owns its
  definition, including the resulting representative tie-break.
- S-0202 claim 10 already agrees with claim 25's lightning exception. Its raw value follows
  bolt-animation creation and precedes explosion creation, except for an empty sound list.
  A repeated strike point does not suppress it. Retain claim 11's shared unsynchronised
  stream for instance draws and feedback choices.
- S-0411: resolve its refusal-entry question and replace planned sound pointers with claims
  24/28. The busy-line refusal of claim 12 and the
  refused placement of claim 10 play announcements only ("unable to comply", "cannot deploy
  here"), not the refusal sound. S-0701 does not add cues to sidebar actions that S-0411 leaves
  unresolved.
- S-0213 claim 16 and O8 already correct the earlier once-per-game route cue assertion:
  a completed tick permits another message-and-sound cue. Claim 24 and F8 here now agree.
  No once-per-game quirk remains.
- S-0406 already agrees in claims 1/10, State and ordering and its
  scenarios: failed placement requests only the cannot-deploy-here announcement, not the
  game-wide refusal sound.
  No placement-audio correction is needed. Keep no-eligible-factory orders silent and
  distinguish announcement requests from queue-dependent playback (claim 27).
- S-0404: a request refused as it is applied (full queue or build limit) plays the refusal sound
  for the local player's house (claim 24).
- S-0414 claim 5 owns post-defeat muting and V20 supplies its end-delay test, with its own
  limits. Claim 36 retains defeat, player-defeated and victory bindings; those do not end
  continued observation. Alliance selection/bindings are deferred as noted above.
- S-0005 claim 10a and M25 agree with claim 34 and its M25(b) backing on independent
  bank-file precedence. Claim 10 owns expansion audio archive preference. The public
  bank formats need no new spec.
- S-0416: half-volume credit ticks are request level 8192 (claim 35).
- S-0314 claim 6 and S-0410 claim 11: use the updated G7 backing described above.
  Zoom timing stays with S-0410 claims 14–15, not an unresolved total tick duration.
  R13 and G7 are proposals, not evidence that lifetimes have been measured.
- glossary already records claim 28's low-funds countdown as presentation
  timing, not a new game-state timing exception. The separate *audio clock* clarification
  required by this repair is listed above.

## Questions for the project lead

Deferred cases need later investigation, not guessed implementation defaults. Q-0327 is applied
in claims 21/32. The glossary's existing deterministic-departure policy is retained, with no new
quirk default and one real-time presentation exception, which the project lead has accepted:
music time in the options menu (claim 30). Q-0671 authorises the engine schedule
now specified in claim 30; no scheduling decision remains deferred to the project lead.

Claim 30 retains S-0202 claim 11's existing shared-stream contract. The former separate-generator
proposal remains withdrawn. Neither shared-stream retention nor separation from game state
waives presentation scheduling determinism. Sound feedback remains in the existing *ui*
topic. Announcement entry names and the repair-response type name remain only in Import mapping,
not new configurable fields.
