---
id: S-0513
title: Superweapon effects
topic: superweapons
revision: 12
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0742, Q-0732, Q-0733, Q-0734, Q-0735, Q-0739, Q-0740, Q-0738, Q-0741, Q-0800]
supersedes: []
---

# S-0513: Superweapon effects

## Summary

What each Allied and Soviet superweapon does once its fire order is carried out: where it may be
aimed, which cells and objects it reaches, when its parts happen, who is credited, what protects
against it, and how its effects combine. Covered: the nuclear strike, the weather storm, the
invulnerability superweapon (iron curtain), the force shield, the two-stage teleport superweapon
(chronosphere), the paradrop dispatch and the spy plane. Skirmish only; network games and
campaigns are out of scope. Psychic reveal is covered by S-0610 claims 13 to 15, genetic mutation
by S-0611 claims 2 to 5, and psychic domination by S-0612 claims 1 to 17. The shared launch
presentation and swept-layer rules below also apply where those specs cite them.

**What this spec does not restate.**
- **Charge, readiness, firing and refusal:** S-0512. In particular the order checks and the
  charge spent by a launch (claim 8), the two-stage source and restart contract (claim 10), the
  link between the stages (claim 11), one-time grants (claim 12) and the storm and domination
  refusals (claim 8). This spec only says what a carried-out order does. The end of a drain state
  (S-0512 claim 19), whether by expiry or by an order toggling it off, ends no effect already
  started: protection timers, a storm, a warp in progress and aircraft already sent all run their
  course. Toggling off starts no new effect either.
- **Damage:** temporary invulnerability and the delayed kill, S-0302 claims 8 and 16; nullified
  small detonations near curtain protection, S-0303 claim 15; area damage, S-0303.
- **Projectiles:** the vertical flight of the nuclear missiles, S-0306 claim 11; the nuclear
  warhead's delayed detonation and nuke flash, S-0306 claim 25.
- **Lighting:** the nuke flash and the storm lighting, S-0104 claim 2.
- **Power:** the timed blackout the force shield starts, S-0405 claim 6.
- **Sight:** area reveals, S-0409 claim 17 (no M6 superweapon uses one); psychic reveal,
  S-0610 claims 13 to 15; ordinary sight, S-0409 claims 2 and 5 to 8.
- **Psychic sensors:** membership and ownership changes, S-0610 claim 9; point-distance reach,
  S-0610 claim 10.
- **Aircraft flight:** the spy plane's approach, overfly, exit and removal, S-0501 claims 10 and 34.
- **Paradrop delivery:** S-0522 claims 1 to 16.
- **Trigger actions:** S-0011 claim 30 hands the lightning storm at a waypoint to this spec; S-0524
  claims 1 to 3 give the storm request, the lightning strike and the nuclear strike at a
  waypoint.
- **Radar:** the radar outage term and radar cursor offers, S-0410 (Terms, claim 12 table C1).
- **Announcements and sounds:** S-0701 claims 25 and 28.
- **Tick order:** the weather storm's place before the turn pass, S-0203 claim 1 row 3.
- **Parasites:** S-0614 claims 12 and 13.
- **Radiation fields:** S-0525 claims 1 to 4. This spec gives only the hand-off.

## Terms

- **Launch.** The carrying out of a superweapon fire order that S-0512 claim 8 or 10 accepts. The
  **launching house** is the house that owns the superweapon record; the **target cell** is the
  cell of the order.
- **Superweapon kind.** The behaviour family a superweapon type belongs to (Import mapping):
  nuclear strike, invulnerability, weather storm, teleport source, teleport warp, paradrop,
  American paradrop, spy plane, force shield, and the three Yuri kinds. S-0512 uses the same word.
- **Effect block.** The 3 × 3 cells centred on a cell (Formulas F1), in the fixed order of F1.
- **Swept layer.** For one cell of an effect block, the cell layer (S-0209 Terms) whose listed
  objects an effect reaches (claim 4).
- **Protection timer.** The countdown of an object's temporary invulnerability (S-0302 Terms),
  with its *kind*: curtain kind or force-shield kind (S-0303 Terms, *curtain protection*).
- **Protection brightness phase.** One interval of the protection's brightness envelope in
  claim 29. Its remaining time is independent of the protection timer. Brightness is measured
  in per-mille as in S-0104 claim 13; a phase factor of 256 means unchanged brightness.
- **Protection colour adjustment.** An imported RGB triple selected by position in the
  colour-adjustment table, not a house-colour definition or a replacement unit palette.
  Claim 29 distinguishes which drawings receive it.
- **Organic flag.** The object type flag of the same name (Import mapping). Infantry types have
  it unless the rules say otherwise; other types do not.
- **Silo.** A building whose type has the missile-silo flag (Import mapping).
- **Payload weapon.** The weapon type with the fixed payload name (Import mapping) whose projectile
  is the falling nuclear missile.
- **Warning marker.** The animation placed at a nuclear strike's target to warn other houses
  (claim 11).
- **Storm.** The single game-wide weather storm of the weather superweapon, the *ion storm* of
  S-0104. Its states are *idle*, *counting down*, *raging* and *clearing* (State and ordering, T1).
- **Storm owner.** The house the storm credits for its strikes. It can be no house.
- **Cloud, strike.** A *cloud* is a storm cloud animation placed over a cell. When it matures it
  makes a *strike*: the *storm bolt* of S-0203 Terms with its damage.
- **Cloud stage.** The integer playback stage of S-0109 claim 1, not the displayed image-frame
  number when the animation has a nonzero start field. Claim 46 gives its storm thresholds.
- **Strike witness.** The ground-layer infantry, vehicle, aircraft or building selected by
  the distance test of claim 18. This is not a search of neighbouring cells.
- **Warp group.** The mobile objects one teleport warp moves or kills (claim 34).
- **Leaving stage, arrival hold.** The period from a warp member's first update after launch
  until departure, including the update at which its fixed 60-tick wait expires (claim 39), and
  the time after arrival effects during which it is still phasing in (claim 40). The glossary
  needs the same distinction: 60 ticks is the leaving wait, not the whole leaving stage.
  A zero held delay still ends phasing in on the update after arrival effects.
- **Phasing in.** The state of a warped object from the moment it leaves its old position until
  its arrival hold ends (claims 39 and 40). A phasing-in object is not warped again (claim 34).
- **Camera sight.** The spy plane's sight with its radius replaced (claim 44).
- **Radiation level.** A dimensionless integer field intensity, in radiation-level units. One
  unit is one unit of the imported radiation level; it is not a health point, a duration or a
  physical radiation dose. Claim 12 passes this intensity on without conversion. Its subsequent
  decay and conversion to damage belong to S-0525 claims 5 to 10.
- **R(a, b).** A scenario-stream ranged draw over the inclusive integer range a … b, as in
  S-0202 claim 4; it can consume zero, one or several raw values.

## Behaviour

### Targeting and every launch

1. **Where a superweapon may be aimed.** The offer to aim a selected superweapon at a cell depends
   only on its kind:
   - a force shield is offered only over a building owned by the player's own house or a house
     allied with it; over anything else the cursor shows the no-force-shield action and a click
     sends no order and keeps the charge,
   - every other kind is offered at every cell the cursor can point at on the map, shrouded or
     fogged, empty or occupied, own or enemy.

   A non-empty unit selection takes precedence: while units are selected, the cursor offers their
   actions, not the superweapon's. Cursor images are specified by S-0214 claim 31, table CS.
   Which superweapons can be aimed through the radar is S-0410 claim 12, table C1.
   These offers are presentation and input only. A fire order that reaches
   the simulation (S-0213, orders table) is not checked against them again: a force-shield order
   at a cell with no allied building is carried out (claim 31).
   - Backing: `measurable` for cursor offers (scenario SW1). Applied orders bypassing those
     offers are `unmeasured`: no normal-input witness for such an order is established here
   - Confidence: high

2. **Which record an aimed click fires.** In the original a battlefield or radar click does not
   fire the superweapon the player selected: it fires the first superweapon type, in the order of
   the type list (S-0004), whose aim action is the action under the cursor. With stock data every
   type has its own action, so the two agree. When two types share an action, a click always fires
   (or is refused for) the first one's record (`quirks.superweapon_order_first_type_of_action`,
   default on = original; off: the order names the selected type).
   - Backing: `measurable` (scenario SW1 part d, named test data)
   - Confidence: high

3. **Launch needs a ready record.** A launch of any kind except the teleport warp does nothing at
   all when the record is not ready at that moment. S-0512 claim 8 gives what the order then does
   to the charge, including a launch that sends nothing (claims 7, 42 and 43 name those cases).
   No kind checks its target cell again at launch, except that the paradrop and spy plane kinds
   send nothing for a cell that is not a legal map cell (claims 42 and 43).
   - Backing: `unmeasured`: ordinary input prevents illegal-cell orders, and no permitted
     normal-input setup here produces and observes one. The effect-level readiness guard is
     also `unmeasured`: ordinary input already prevents unready orders, so no independent
     normal-input test isolates the additional guard. Legal-cell dispatch is tested separately
     under claims 42 and 43
   - Confidence: high

4. **Effect blocks and swept layers.** The invulnerability superweapon and teleport warp affect
   the objects listed in the nine cells of an effect block (F1). Their observable recipient
   sequence and resulting scenario-stream state match F1's cell order and S-0209 claim 13's
   listing order within each cell. Objects listed in none of those cells are not reached:
   aircraft in flight, passengers inside transports, occupants inside buildings, and objects
   off the map. A building listed in several cells receives an application for each listing.
   In a high-bridge cell (S-0511), objects on the deck are affected and objects on the ground or
   water beneath it are not; in every other cell, only ground-layer objects are affected
   (`quirks.superweapon_block_one_layer`, default on = original; off: both layers of a
   high-bridge cell are affected, with deck-layer effects preceding ground-layer effects in
   the observable recipient sequence). The waypoint invulnerability trigger action always
   affects only the ground layer (claim 22).

   Genetic mutation in cell mode shares this effect block, swept-layer rule and
   `quirks.superweapon_block_one_layer`; S-0611 claim 4 owns its infantry recipients and hits.
   Its recipients are not cut short by removal of an earlier victim: the list-cut quirk in
   State and ordering does not apply to mutation.
   - Backing: `measurable` for recipients and ordering (SW9 parts b and g, SW12 parts e and f).
     Mutation's use of the shared layer rule and its lack of a list cut inherit S-0611 claim 4's
     backing (scenario MU4).
     Repeated application to a building listed in multiple cells is `unmeasured`: the same-tick
     protection replacement has no distinct observable in these tests
   - Confidence: high

5. **Announcements of a launch.** These are presentation. Sound selection retains S-0701
   claims 25 and 28's draw rules.

   | Kind | Heard or seen by | What |
   |---|---|---|
   | Nuclear strike | every player | launch announcement (S-0701 claim 28), and the siren sound at the target point |
   | Weather storm | every player | launch announcement at the launch; at the storm's start the storm sound and start message (claim 15) |
   | Invulnerability | every player | activation announcement; radar event at the target cell |
   | Force shield | every player near the target | the type's start sound at the target, and later its fading sound (claim 32); no announcement, no radar event |
   | Teleport source | the launching player only | the source marker (claim 33); no announcement, no radar event |
   | Teleport warp | every player | activation announcement; radar events at the source and destination cells |
   | Genetic mutation | every player | activation announcement; activation sound at the target; radar event at the target cell (S-0611 claim 2) |
   | Psychic domination | every player | activation announcement; activation sound at the target; radar event at the target cell (S-0612 claim 1) |
   | Paradrop, spy plane | nobody | nothing |

   The genetic-mutation and psychic-domination activation sounds are positional: only players
   near the target hear them, unlike their announcements.

   For the launching player a launch other than a teleport source pick ends the aiming mode and
   silences that type's ready announcement if it is still queued or playing. A source pick instead
   enters warp aiming (claim 33). After the local player's house is defeated, the
   launch announcements are muted for that player (S-0701 claim 28). Radar events of the same
   kind close to an earlier one are merged (the interface specs own radar events).
   - Backing: `measurable` (SW2 part f, SW9 part f, SW11 part e, SW12 part g and SW18).
     The genetic-mutation row inherits S-0611 claim 2's backing (MU1 part a and MU11), and
     the psychic-domination row inherits S-0612 claim 1's backing (PD1).
     Radar-event merging inherits the interface specs' backing rather than being tested here
   - Confidence: high

6. **No game-rule draw after a launch (departure).** In the original every fire order, carried
   out or refused, is followed by a check of each computer-controlled house in turn (by
   descending house number): when the superweapon type is marked as one computer houses defend
   against (Import mapping) and the target is close to that house's base, the house takes one
   ranged draw R(0, 99) from the scenario stream and may mark its base for a defensive
   force shield. Under the lead's AI-house rule this is computer-player behaviour: our engine's
   game rules take no such draw, and the AI module may make its own decision. A scenario that
   compares the original's draw stream must keep every computer house's base far from the
   target, or account for that draw.
   - Backing: `measurable` for the original's conditional draw (SW17). The omission from our
     game rules is a design rule
   - Confidence: high

### Nuclear strike

7. **Which building launches.** A launch of an ordinary (not one-time) nuclear strike record looks
   for the launching building as follows. Among the building types in type-list order, the first
   that is a silo and names this superweapon type in its first or second superweapon field is the
   only type considered, whether or not the house owns one of it
   (`quirks.nuke_silo_first_type_only`, default on = original; off: every such silo type is
   considered, in type-list order). These are the silo type's own fields, not those of an
   upgrade it carries. Launcher selection is distinct from the *giving building* test of
   S-0512 claims 3, 6 and 7: it does not require the superweapon's auxiliary building.
   The auxiliary condition still governs building-based grants and their retention under
   those claims; it is not an additional effect-level launch test. Thus a ready permanent
   trigger grant (S-0512 claim 13) can launch through a matching silo even when the auxiliary
   building is absent. Such a grant does not waive the need for a matching silo.
   Of the house's buildings of the selected type that are on the map,
   the one the house has owned longest is chosen. Distance, health, power, an electromagnetic pulse
   and selling play no part in the choice. The house is *able to launch* in the sense of S-0512
   claim 8 only when such a building exists and accepts the launch behaviour. A selling silo
   refuses it; one already in its launch sequence with nothing queued need not start a second
   sequence. In those cases the target is still recorded and the launch sounds still play,
   although no additional missile is launched. With no matching building, neither missile nor
   launch sounds occur. S-0512 claim 8 decides the charge in all these cases.

   A one-time record does not use a building at all (claim 10).
   - Backing: `measurable` (scenario SW2 parts b, c, e and i, including named test data for
     multiple silo types, a busy silo and the auxiliary-building distinction).
     Exclusion of upgrade-only superweapon fields is `unmeasured`: no scenario part here
     puts the nuclear record solely on an upgrade of a silo. Health and power independence are `unmeasured`:
     those parts do not vary them independently. Electromagnetic-pulse independence is
     `unmeasured`: no stock skirmish pulse delivery is established here
   - Confidence: high for the type rule; medium for "owned longest" and for the busy-silo case

8. **The silo's launch.** When the chosen silo accepts the launch:
   - the launch announcement and siren play at once (claim 5), and the target cell is recorded for
     the house (a later launch of the house replaces it),
   - on the silo's next update the upward missile leaves. It is a projectile of the weapon of the
     superweapon type that stands at the list position equal to the nuclear-strike kind number
     (with stock data, the nuclear strike's own weapon; see
     `quirks.superweapon_kind_as_list_position`, Edge cases), owned by the silo, carrying that
     weapon's damage and warhead, with top speed 255 leptons per tick and a start speed of 10
     leptons per tick straight up, from the silo's first firing position (S-0313). The silo's take-off animation
     appears there. The silo's door animation runs alongside and does not delay the missile,
   - the silo then plays its launch animation phases and returns to guarding. Its picture is
     presentation, but the busy interval is game state (claim 47).

   The upward missile flies straight up under S-0306 claim 11. When its warhead has the
   missile-maker flag it does no damage at its burst: it is replaced by the falling missile
   (claim 9). Its detonation altitude is an absolute world-height threshold, not a height above
   the silo or target terrain. The altitude test bursts it when its new world height exceeds
   that threshold, without clamping to it (S-0306 claim 11); another detonation condition in
   that claim may end flight earlier. Without the flag it detonates where it bursts, as an
   ordinary projectile, and no missile falls.
   - Backing: `measurable` for stock flight (SW2 part a), the altitude distinction (SW2 part h)
     and the type-list-position weapon choice (SW2 part g). A non-missile-making upward warhead is `unmeasured`: no scenario part
     here replaces that flag to isolate its ordinary detonation
   - Confidence: high

9. **The falling missile of a silo launch.** At the upward missile's burst a falling missile is
   created over the recorded target cell: a projectile of the payload weapon, owned by the
   upward missile's owner (the silo, or no object if the silo has left play), with the payload
   weapon's damage, warhead and speed (S-0306 claim 1), aimed at the target cell. It starts at the
   cell centre with world height equal to the target terrain surface height plus the *upward*
   projectile type's detonation-altitude value, with no bridge lift (F8). This reuses the value
   as a height above the target terrain; it does not inherit the upward missile's actual burst
   height, and it is not the upward flight's absolute-height test of claim 8. Thus raising the
   target terrain raises the falling missile's start height, without changing that upward
   threshold. The falling missile starts straight down at 1 lepton per tick and accelerates
   under S-0306 claim 11 and formula F10. Its own projectile type's detonation altitude remains
   an absolute world-height threshold: a new position above it causes a burst on the first
   turn, even though the missile is descending.
   - Backing: `measurable` (scenario SW2 parts a and h)
   - Confidence: high

10. **One-time nuclear strike.** A launch of a one-time record (S-0512 claim 12) creates the falling
    missile directly: a payload-weapon projectile with no owner, the payload weapon's damage,
    warhead and speed, aimed at the target cell, starting 20 000 leptons above the cell centre's
    terrain surface (plus the 416-lepton lift on a high-bridge cell, F2), moving straight down with
    start speed 100 leptons per tick. The warning marker (claim 11) is attached to it, and the
    launch announcement and siren play (claim 5).
    - Backing: `measurable` (scenario SW3)
    - Confidence: high

11. **The warning marker.** A nuclear strike places the warning-marker animation at the target
    cell's centre, attached to a missile: in a silo launch to the *upward* missile (from the
    silo's first update), in a one-time launch to the falling missile. The marker ends when that
    missile leaves play, so in a silo launch it disappears at the upward missile's burst and the
    falling missile has none. The marker is shown to a viewing player only while all of these hold:
    the viewer's house is not allied with the launching house (a house is allied with itself); the
    viewer's house owns a psychic sensor from the set defined by S-0610 claim 9 that is in
    operation (S-0405); and that sensor's reference point is within its psychic-detection radius
    of the target cell's centre. Reach uses the point-to-point spatial distance of S-0610
    claim 10, heights included: coordinate differences are integer leptons, their squares and
    sum are evaluated in floating point, and S-0204 claim 5's table root is truncated toward
    zero to integer leptons. The resulting distance must be at most 256 times the radius in
    whole cells, an integer comparison. No building-size subtraction applies. Sensor membership
    follows placement and capture under S-0610 claim 9. The reach and operating tests are
    re-evaluated in every update of the marker. This is presentation.
    - Backing: `measurable` (scenario SW4)
    - Confidence: medium (the operating test of the detection building)

12. **Detonation, radiation and credit.** The falling missile's detonation is S-0306 claim 25 when
    its warhead has the nuclear warhead name, otherwise an ordinary detonation (S-0306). Its damage
    and spread are its own (S-0303). Its attacker is its owner: the silo, or no object (a
    one-time strike, or a silo that has left play), which S-0303 and S-0302 treat as a hit without
    attacker. **Radiation hand-off:** when the payload weapon's radiation level is above 0, its
    detonation leaves radiation at the detonation cell with that level and a spread of the
    warhead's cell spread truncated toward zero to whole cells. If that cell is the centre of a
    still-existing radiation field, its new level is that field's decayed current level plus the
    weapon's radiation level, not its original level plus the weapon's level; the new spread is
    ignored (S-0525 claims 2 and 4). S-0525 claim 4 owns the floating-point calculation and
    truncation of the current level, and the restarted duration and decay timers.
    The same applies to the upward missile if its own weapon has a positive radiation level.
    - Backing: `measurable` (SW2 part d, SW3) for credit, field creation and level addition.
      The decayed-current-level calculation and retention of the old spread also inherit
      S-0525 claim 4's backing (its scenario RF4); SW2 part d alone repeats the same spread.
      Upward-missile
      radiation is `unmeasured`: SW2 tests the payload's radiation, not this separate weapon
    - Confidence: high for the credit and the hand-off condition; medium for the stacking

13. **Silo lost or captured.** After the silo accepted the launch:
    - sold or destroyed before its next update: no missile leaves,
    - destroyed after the upward missile left: both missiles carry on; the falling missile has no
      owner, so its kills credit nobody,
    - captured after the upward missile left: the silo is still the owner, so its new house is
      credited.

    A silo captured between accepting the launch and its next update launches at the target last
    recorded for its *new* house, or at no valid target if that house never launched
    (`quirks.silo_target_of_current_owner`, default on = original; off: it launches at the target
    recorded when it accepted).
    - Backing: `measurable` (scenario SW2 part e); `unmeasured` for the capture inside one tick
      (no normal-input capture can be timed between acceptance and the silo's next update)
    - Confidence: medium

### Weather storm

14. **Requesting a storm.** A launch of the weather kind requests a storm at the target cell, owned
    by the launching house, with the game-wide storm duration and storm countdown (Import mapping).
    A request while the storm is raging or clearing does nothing. Otherwise:
    - with a countdown *L* > 0 the storm becomes *counting down*: centre, owner and duration are
      replaced by the request's, and the remaining countdown becomes *L* if none was running, or
      the smaller of the running remainder and *L*,
    - with *L* = 0 the storm starts at once (claim 15).

    S-0512 claim 8 refuses a weather launch while a countdown runs or the storm is raging or
    clearing, so with stock data only trigger actions replace a running countdown (S-0524 claim 1).
    The countdown decreases by 1 in every weather update (S-0203 claim 1, row 3). When the
    remainder becomes 0 the storm starts in that update, and makes no cloud in it. While counting
    down, when the remainder is a positive multiple of 225 ticks and the storm-message flag is set,
    every player hears the storm-approach announcement and sees the storm-approach message for
    150 ticks. A storm requested in tick *T* before that tick's weather update therefore starts in
    tick *T* + *L* − 1 (F5). The weather launch itself also plays the storm-created
    announcement to every player, whatever the storm-message flag.
    - Backing: `measurable` (scenario SW6 parts a and b)
    - Confidence: high

15. **Start of a storm.** When a storm starts it becomes *raging*, with its start tick, centre, owner
    and duration. In the same tick:
    - every house that is not defeated and not allied with the storm owner (every such house when
      there is no owner) starts a radar outage (S-0410 Terms) of the storm's duration,
    - a radar event is raised at the centre cell, the lighting is re-evaluated (S-0104 claim 2),
      and, when the storm-message flag is set, every player hears the storm sound and sees the
      storm-start message for 150 ticks.

    **Departure (no switch).** In the original, once the local player's house is defeated, no
    storm starts a radar outage for any house again. Game state in our engine does not depend on
    which house is the local player, so outages always start as above.
    - Backing: `measurable` (scenario SW6 parts c and d)
    - Confidence: high; medium that the defeat test reads the defeated state of S-0414

16. **Clouds while raging.** In each weather update of a raging storm whose end test (claim 19)
    does not end it, clouds are made over cells:
    - a *centre cloud* over the centre cell in every tick whose number is a multiple of the hit
      period (the absolute tick number, not ticks since the start), with no further test,
    - a *scatter cloud* in every tick whose number is a multiple of the scatter period: up to three
      tries, each drawing a column displacement and then a row displacement, each R(−*r*, *r*)
      on the scenario stream with *r* = ⌊*s* / 2⌋ for the storm's cell spread *s* (F6). A try
      fails when its cell is not a legal map cell, or when its Manhattan distance in cells to the
      cell of any present cloud (including a centre cloud made in the same tick and clouds made by
      trigger actions) is below the separation. The first try that does not fail makes one cloud; at
      most one scatter cloud is made per period.

    A cloud is *present* from its creation until a weather update finds its cloud stage at
    least its current image's frame count minus 1 (claim 46). Clouds
    prefer nothing: no unit or building is sought. A cloud is placed over a cell regardless of
    shroud and of the usable area.
    - Backing: `measurable` (scenario SW7 parts a to c, with the random-state recorder)
    - Confidence: high

17. **A cloud.** A cloud is placed over the centre of its cell at the height of F4, which depends on
    the image of the first entry in the bolt animation list, whichever bolt the cloud later uses.
    Its animation is chosen by one raw value of the scenario stream reduced modulo the length of
    the cloud animation list (S-0202), and its creation then takes the animation's own creation
    draws (S-0109).
    - Backing: `measurable` (scenario SW8 part c for the height; SW7 part c for the draws)
    - Confidence: high; medium for the floating-point height (F4)

18. **A strike.** In each weather update, before any new cloud is made, every cloud that is still
    *pending* strikes once its cloud stage exceeds ⌊*F* / 2⌋, *F* being the frame
    count of the cloud's current image (claim 46). For clouds striking in the same update, damage, animation
    choices and the resulting scenario-stream state match newest-cloud-first strike order. This applies in every storm
    state: a trigger-created cloud needs no raging storm to produce its bolt or damage.
    An ordinary image-length forward cloud as in claim 46 whose image has
    *F* ≤ 2 frames never strikes. Custom playback that survives beyond that limit
    still uses the stage test above. A strike at the cloud's cell, in that update:
    - creates a bolt animation chosen by one raw value modulo the bolt list length, at the cell
      centre at the terrain surface (visual only),
    - plays a lightning sound (S-0701 claim 25),
    - delivers area damage (S-0303) of the storm damage with the storm warhead at the cell centre at
      the cell's height level (plus the bridge lift on a high-bridge cell), with no attacker
      object, affecting resources, with the storm owner *at that moment* as attacking house. The
      warhead's explosion animation is the storm explosion animation, except that a water splash
      replaces it as for any warhead on water (S-0303). No damage and no explosion when the storm
      damage is 0,
    - leaves debris when its pre-damage strike witness is not infantry (no witness also passes)
      and either the first ground-layer building, strike witness or cell height level differs
      after damage, or there was neither a ground-layer building nor a strike witness before
      damage and the land type was road, rock, wall or weeds (S-0211): R(2, 4) pieces, each
      chosen by R(0, *n* − 1) over the game-wide metallic debris list of *n* entries.

    The strike witness is selected only from the cell's ground-layer listing, including
    buildings and landed aircraft as well as infantry and vehicles, but excluding terrain
    objects. Deck objects and aircraft not in that listing do not count. For each candidate,
    let *x* and *y* be its reference point's integer lepton coordinates modulo 256, in
    0 … 255 (S-0204 claim 4). For a non-building this is its own position.
    Its distance is trunc(*r*(*x*² + *y*²)) leptons, where *r* is S-0204 claim 5's table
    square root: squares and sum are evaluated in floating point and the root truncates
    toward zero. This compares the point's within-cell remainders with (0, 0), not with
    the cell centre. Height and the building-size distance reduction do not enter.
    Foundation dimensions do affect a building's reference point, as detailed below.
    The smallest resulting integer distance wins, with ties
    going to the first candidate in S-0209 claim 13's listing order. The post-damage
    witness uses the same test. Mere health loss without a changed witness, building or
    height does not trigger debris. “Held nothing” means no qualifying witness
    and no ground-layer building, not an empty object list: a terrain object alone does
    not prevent this condition.

    **Building witnesses (Q-0800).** An on-map ground-layer building is listed in every
    foundation cell, not just the cell containing its own position or reference point.
    Foundation cells are those of S-0406 claim 2, excluding any missing cell of an irregular
    foundation. A strike on any listed foundation cell can therefore select that building.
    The point tested is always the same foundation centre from S-0204 claim 4, without a
    bib row: for top-left foundation cell (*a*, *b*) and dimensions *W* × *H* cells,
    its horizontal coordinates are (256*a* + 128*W*, 256*b* + 128*H*) integer leptons.
    It is neither the top-left cell centre nor a point relocated into the struck cell.
    Only its remainders modulo 256 enter the test, even when that centre lies in a
    different cell. Thus the following distances apply in every listed foundation cell:

    | Foundation dimensions | Remainders (*x*, *y*), leptons | Truncated table-root distance, leptons |
    |---|---|---|
    | Both even | (0, 0) | 0 |
    | Odd width, even height | (128, 0) | 128 |
    | Even width, odd height | (0, 128) | 128 |
    | Both odd, including 1 × 1 | (128, 128) | 181 |

    These are integer coordinate calculations with no rounding, followed by the
    floating-point table-root calculation already specified above. In particular, a
    one-cell building's own position and reference point coincide at its cell centre
    (S-0204 claims 3 and 4). A listed infantry unit or vehicle with distance below
    181 leptons wins against it; at exactly 181 leptons that non-building also wins
    because it precedes buildings in the listing. A 2 × 2 building instead has distance
    0 in all four tests, although an earlier-listed candidate also at distance 0 wins
    the tie. This is a remainder comparison, not the physical distance from the struck
    cell's top corner to the building centre.

    Every debris animation is created at the strike point: the cell centre at its height
    level plus the high-bridge lift, with no additional 20-lepton lift and no horizontal
    displacement. This is distinct from the bolt's terrain-surface point.

    Damage lands in the same tick as the bolt appears. The scenario-stream state and choices
    observed for a strike match this order: bolt choice and animation-creation draws; sound
    choice; explosion-animation draws; damage draws; debris-count draw; and, for each debris
    piece, its choice and animation-creation draws. Debris pieces deal what their animations
    deal (S-0109).
    - Backing: `measurable` (scenario SW7 parts d and e; building reference point,
      per-foundation-cell listing and parity distances: SW7 part f)
    - Confidence: high for the witness, empty-cell test and debris creation point; medium
      for changes to the cell height caused by damage

19. **End and clearing.** A raging storm whose duration is not the endless value (−1) begins
    *clearing* in the first weather update in which its start tick plus duration is below the
    current tick; no cloud is made in that update. A storm-stop request (claim 20)
    also begins clearing, but only while raging. While clearing, clouds already made still strike
    as claim 18 says. The storm becomes *idle* in the first weather update after the one in which
    clearing began that finds no present cloud at its start: its owner becomes no house and the
    lighting is re-evaluated. A cloud at claim 46's presence threshold is removed in that
    update, so the
    storm becomes idle one weather update after the last cloud's removal; with no clouds at all,
    in the weather update right after clearing began.

    Who is credited for a strike: the owner of a raging or clearing storm; while counting down, the
    house of the latest request; no house when idle. So a cloud made by a trigger action strikes
    for whoever is storm owner when it matures (S-0524 claim 2), even when that house differs from
    both the cloud's trigger owner and the storm owner at cloud creation. Before any storm request
    since game setup, such a cloud's strike credits no house. A trigger-created cloud counts as
    present during clearing too, so adding one can postpone the return to idle.
    The defeat of the owner changes
    nothing: the storm runs on and keeps crediting that house. Neither defeat nor a stop request
    cancels a running countdown.
    - Backing: `measurable` (scenario SW8 parts a and b)
    - Confidence: high

20. **Other ways to start or stop a storm.** The storm of claims 14 to 19 is the one storm the trigger
    actions also use:
    - The lightning-storm-at-a-waypoint action requests this same single storm (S-0524 claim 1),
      owned by the trigger's owner house, possibly no house, with the game-wide duration and
      countdown. It may replace a pending request as in claim 14, but not a raging or clearing storm.
    - The lightning-strike-at-a-waypoint action (S-0524 claim 2) creates a cloud even while the
      storm is idle. Repeating the action at the same waypoint creates another cloud, with a
      separate cloud-list pick and creation draws as in claim 17. These clouds strike under
      claim 18 in every storm state, including idle, and use claim 19's owner at strike time,
      not the trigger's owner. They use F4's height and claim 21's empty-list departures.
    - The storm-start action requests a storm whose duration is the action's number, with the
      game-wide countdown, at a random legal cell and with no owner. The random cell is found by
      drawing a row, then a column, each R(0, *W* + *H* − 1) on the scenario stream for the map's
      width *W* and height *H* in cells, until the cell is a legal map cell. Like claim 14 it does
      nothing while the storm is raging or clearing.
    - The storm-stop action begins clearing while the storm is raging.
    - Backing: `measurable` (SW8 parts b and d; repeated waypoint clouds also use S-0524 claim 2's
      scenario C2). Unsafe-list fallbacks retain claim 21's `unmeasured` design backing
    - Confidence: medium (the owner of the storm-start action)

21. **Unsafe storm data (departure).** The original does not handle these safely. Our engine warns
    at import and behaves as stated:
    - an empty cloud animation list: no cloud is made,
    - a selected cloud type with no image, or a nonempty bolt list whose first type has
      no image: no cloud is made (the header reads needed by claims 16, 18 and F4 are unsafe),
    - an empty bolt animation list: the cloud height term *h*_c of F4 is 0, and strikes make no bolt
      animation but otherwise happen,
    - no storm warhead: strikes deliver no damage,
    - a hit or scatter period of 0 or less: that kind of cloud is never made.
    - Backing: `unmeasured` (design rules for data the original cannot run)
    - Confidence: high

### Invulnerability (iron curtain)

22. **What the curtain reaches.** A launch of the invulnerability kind places the curtain invoke
    animation at the target cell's centre (5 leptons above its surface, plus the bridge lift) and
    acts on every listed object of the target cell's effect block (claim 4) of any house, whatever
    the shroud. An infantry unit, vehicle or aircraft that is in the leaving stage of a teleport
    warp, or already has that warp pending (claims 34 and 39), is skipped. A non-mobile object other than a building (a terrain object, for
    example) is unaffected. Each reached object is treated by its kind (claims 23 to 25).

    The trigger action numbered 109 applies invulnerability at its waypoint-slot cell
    (Import mapping). It acts on the ground layer of the waypoint cell's effect block
    (whatever the bridge), with no launching house and without the leaving-stage skip.
    It uses the same imported curtain duration, not an action-supplied duration or house.
    It creates the curtain invoke animation at the waypoint cell centre's terrain surface
    plus 5 leptons, with no bridge lift. It raises no radar event and makes no activation
    announcement; ordinary animation creation effects still apply.
    - Backing: `measurable` (scenario SW9 parts a and b, SW19)
    - Confidence: high

23. **Infantry die under the curtain.** Every reached infantry unit receives a hit of its type's whole
    strength, distance 0, with the demolition warhead, ignoring defences (S-0302 claim 9), with no
    attacker object and the launching house as attacking house. Its organic flag plays no part.
    It is never protected. Because the hit ignores defences it kills a cyborg outright (S-0302
    claim 15) and is not stopped by an existing protection (S-0302 claim 8, column I).
    - Backing: `measurable` (scenario SW9 part a)
    - Confidence: high

24. **Organic vehicles.** A reached vehicle or aircraft whose type has the organic flag instead
    receives a hit of its type's whole strength, distance 0, with the demolition warhead, with no
    attacker and no attacking house, that does **not** ignore defences, and is not protected. So its
    armour can let it survive, a protected one takes nothing and keeps its old protection, and
    nobody is credited (`quirks.curtain_organic_vehicle_defended`, default on = original; off: the
    hit ignores defences and credits the launching house, as for infantry). No stock vehicle type
    has the flag.
    - Backing: `measurable` (scenario SW9 part c, named test data)
    - Confidence: high

25. **Starting protection.** Every other reached vehicle, aircraft and building becomes protected,
    curtain kind, for the curtain duration from this tick (claim 26). Starting it also:
    - for a vehicle or aircraft: releases a parasite inside it, a drone parasite being removed
      silently and a naval parasite exiting, as for unloading in S-0614 claim 12 (the curtain sets
      50 ticks of suppression just before the exit), and ends a paralysis it is under,
    - for a building: cancels a pending delayed-kill countdown (S-0302 claim 16) and forgets the
      recorded charge planter.

    Starting protection on an object that is already protected replaces its timer and its kind: a
    shorter new duration shortens the protection, and a later force shield turns curtain
    protection into force-shield protection and back (`quirks.protection_restart_replaces`,
    default on = original; off: the longer remaining time is kept, with the kind of the effect
    that gave it).
    - Backing: `measurable` (scenario SW9 parts d and e, SW10 part b)
    - Confidence: high

26. **Duration.** An object protected in tick *t*₀ for *D* ticks is protected in every tick *t*
    with *t*₀ ≤ *t* < *t*₀ + *D* (F7), whether it is on the map, inside a transport or out of play,
    and whatever happens to its owner. A duration of 0 or less protects nothing (an infantry unit
    still dies). There is no weakening phase: protection is complete until it ends.
    - Backing: `measurable` for on-map, transport and ownership cases (SW10 parts a and f).
      Other out-of-play states are `unmeasured`: those parts do not isolate their timer expiry
    - Confidence: high

27. **What protection refuses besides damage.** Apart from S-0302 claim 8 and S-0303 claim 15, a
    protected object:
    - cannot be mind-controlled, permanently or temporarily, and is passed over by a psychic
      domination,
    - cannot start to be erased by a chrono weapon (S-0514),
    - is not moved by a teleport warp, and kills an arriving warped object (claims 34, 37, 38),
    - cannot be crushed by an object that is not allied with it (S-0210),
    - has no locomotor seized by a locomotor-seizing warhead, and is not grabbed by a warhead that
      flings its target,
    - as a building, cannot have a demolition charge planted on it: the planter ends its action,
    - is an illegal target for a weapon whose warhead is a parasite warhead (S-0614).

    An electromagnetic pulse and an existing erasure are not stopped by protection.
    - Backing: `measurable` for the listed refusals (SW10 part c). Pulse independence and an
      already-running erasure are `unmeasured`: no independent stock pulse delivery or
      reproducible protection-during-erasure setup is established here
    - Confidence: high for the listed refusals; low for the pulse and existing erasure, whose
      independent delivery and continuation paths remain untraced here

28. **Firing at protected objects (departure).** In the original a weapon of a house that is not
    human-controlled gets the *illegal target* answer against a protected object, while a human
    house's units may fire at it and their hits do nothing. Under the lead's AI-house rule our
    engine gives every player house the human rule; the AI module may choose not to fire. The two
    non-player houses keep the original's rule for houses that are not human-controlled.
    - Backing: `measurable` for the original's behaviour (scenario SW10 part d); our rule is design
    - Confidence: high

29. **Protection appearance and its draw (Q-0742 question 4).** Both protection kinds use
    the same brightness envelope below. It brightens initially, dims, pulses at low brightness,
    then brightens and returns to ordinary brightness near expiry. It does not weaken the
    protection of claim 26. There is no fixed RGB colour to hard-code: colour adjustments come
    from the imported table. Nor is a different palette file loaded for protection.

    | Drawing | Additional protection colour adjustment |
    |---|---|
    | Curtain-protected vehicle drawn as voxels | Curtain selector's RGB triple |
    | Curtain-protected building body drawn as a sprite | None: brightness envelope only |
    | Force-shielded building body drawn as a sprite | Force-shield selector's RGB triple |

    Thus sprites and voxels share the brightness timing, but not an unconditional identical
    colour treatment. Ordinary infantry are killed, not recoloured (claim 23). This table
    is a subset of S-0102 claim 34's ordinary opaque drawing coverage, which also covers
    sprite vehicles, aircraft, building voxel turrets and marked building animations.
    Exact packed-pixel colour composition is S-0102 claim 34 and F9, including its
    single-layout drawing correction without a quirk switch. The narrower unresolved
    drawing variants are identified in Open questions. S-0104 claim 13 supplies ordinary brightness,
    and its claim 15 maps the adjusted brightness to shade rows.

    Let *q* be the first turn of the object while this application is still protecting it,
    *P* its remaining protection in integer ticks, and *r* the remaining phase time in ticks.
    A phase of length *L* entered in tick *s* has *r* = max(0, *L* − (*t* − *s*)) in tick *t*.
    With one turn each tick, its successor starts in tick *s* + *L*. The phase table gives
    the dimensionless integer factor *m*; F9 gives the brightness calculation.

    | Brightness phase | Length | Factor *m* | Following phase |
    |---|---|---|---|
    | Before the first protected turn | Until *q* | 256 | Initial rise |
    | Initial rise | 6 ticks, starting at *q* | trunc((12 − *r*) × 256 / 6) | Bright hold |
    | Bright hold | 4 ticks | 512 | Initial dimming |
    | Initial dimming | 20 + R(−5, 5) ticks | trunc((461 × *r* + 1020) / 20) | Pulse rise |
    | Pulse rise | 8 ticks | trunc((1024 − 77 × *r*) / 8) | Pulse fall |
    | Pulse fall | 16 ticks | trunc((77 × *r* + 816) / 16) | Low hold if *P* < 54 ticks, otherwise Pulse rise |
    | Low hold | Until a turn finds *P* < 31 ticks | 51 | Final rise |
    | Final rise | 6 ticks | trunc((3072 − 461 × *r*) / 6) | Final bright hold |
    | Final bright hold | 4 ticks | 512 | Return to ordinary |
    | Return to ordinary | 20 ticks | trunc((*r* + 20) × 256 / 20) | Ordinary |
    | Ordinary | Until protection expires | 256 | No protection appearance |

    All table arithmetic is integer, with divisions truncated toward zero. The remaining-time
    tests happen only at the indicated transitions, not continuously in the preceding phases.
    Low hold occupies at least its entry turn: even if its threshold is already met, Final
    rise starts no earlier than the next turn. If an object misses turns, an expired phase
    advances only to its next phase on its next protected turn, without skipping intervening
    phases. Protection expiry immediately removes its appearance regardless of the current phase.
    Reapplication restarts the envelope, even if it changes protection kind (claim 25).

    There is exactly one ranged draw R(−5, 5) from the scenario stream for each application
    that reaches Initial dimming. It occurs in turn *q* + 10 with uninterrupted turns.
    For a launch applied after the turn pass in tick *t*₀ this is *t*₀ + 11, not *t*₀ + 10.
    This is identical for curtain and force shield and for buildings and mobile objects
    that take these protected turns; force shield itself protects buildings only (claim 31).
    Expiry or restart before that turn prevents that application's draw. No turns means no
    phase transitions or draw; elapsed phase time is still in game ticks, not counted turns.
    A missed due turn postpones the transition to the next protected turn, but there is
    no independently scheduled draw due at application plus 10. For example, after entering
    Bright hold, an object absent across its expiry draws on its first protected turn back.
    An object absent across the expiry of Initial rise instead enters Bright hold on its
    return, starts a fresh 4-tick hold then, and draws only when that hold expires.
    The draw is game state; the brightness and colour are presentation.
    - Backing: `measurable` (SW10 parts e and g). Ordinary opaque pixel composition and
      the additional drawing cases inherit S-0102 claim 34's backing (D44).
      The unresolved variants in Open questions are not assigned guessed values
    - Confidence: high for phase lengths, factors, draw timing relative to *q*, the ordinary
      post-turn launch boundary and the three colour-selection rows

### Force shield

30. **Blackout of the launching house.** A force-shield launch starts a timed blackout of the
    launching house (S-0405 claim 6) whose length is the force-shield blackout length (Import
    mapping). The house is already blacked out when protection begins on any reached building.
    - Backing: `measurable` (scenario SW11 part c)
    - Confidence: high

31. **Shielded buildings.** A force-shield launch places the force-shield invoke animation at the
    target cell's centre and protects, force-shield kind, for the force-shield duration, every
    building whose owner counts the launching house as allied (including itself) and whose reference
    point (S-0204 claim 4) lies within the force-shield radius of the target cell's centre at the
    terrain surface (no bridge lift): the spatial distance, with the table square root truncated,
    is below 256 times the radius in cells (F3). Infantry, vehicles, aircraft and the buildings of
    other houses are not protected, and nothing is killed. Starting protection has claim 25's
    effects on each building. The building need not be the one the cursor offered (claim 1).
    There is no separate on-map or health test for a building that still exists and has a
    qualifying reference point.
    - Backing: `measurable` for on-map buildings (SW11 parts a and b). Off-map buildings are
      `unmeasured`: no normal-input witness with a qualifying off-map reference point is
      established here
    - Confidence: high

32. **Force-shield sounds.** The type's start sound plays at the target when the type has one.
    The type's special sound plays at the target in the house update that is *D* − *P* house
    updates after the launch, *D* being the force-shield duration and *P* the fade-sound lead time,
    or in the next house update when *D* − *P* = 0, and never when it is negative. A second launch
    of the same house's record before that replaces the pending sound. Presentation.
    - Backing: `measurable` (scenario SW11 part e)
    - Confidence: high

### Teleport (chronosphere)

33. **Source pick.** A launch of the teleport-source kind records the target cell as the source of
    the teleport and places the source marker animation over it. Only the launching player sees and
    hears the marker; for that player it starts aiming the type at registration index 4, counting
    from 0, under `quirks.superweapon_kind_as_list_position` (Edge cases). With stock data this
    is the warp type. If another type occupies that position, the player aims that other type,
    not necessarily the linked second stage. A new pick replaces the source and its marker. When the player leaves warp aiming without firing, the marker ends; the
    recorded source stays until replaced. The source cell need not hold any object.
    The marker is created regardless of which house is local, taking its ordinary animation
    creation draws on the scenario stream (S-0202 claim 10; S-0109 claim 13). Hiding it and
    silencing it for a non-launching viewer does not undo or suppress those draws. It is not
    a draw-free local click effect. S-0512 claim 10
    gives source retention and the charge contract: an ordinary completed first stage keeps its
    charge, and source selection alone does not commit the warp.
    - Backing: `measurable` (scenario SW12 parts a and h; SW22 for marker draws)
    - Confidence: high

34. **The warp group.** The second-stage order uses the linked first stage's last picked source.
    Its unchecked acceptance and the first stage's recharge are S-0512 claims 10 and 11, not an
    additional readiness test here. Before any source pick, it uses cell (0, 0) and attempts
    the ordinary warp effect there; it is not a no-op or a substitution of the giving
    building. A granted linked first stage still restarts, as S-0512 claim 10 specifies. A launch of the teleport-warp kind at a destination cell plays the blast
    animations (one at the source centre, one at the destination centre), ends the source marker,
    and affects the listed objects of the source cell's effect block (claim 4), of any house.
    The observable recipient sequence for infantry, vehicles and aircraft follows claim 4,
    not a grouping by kind; buildings and other objects are never moved. An object in the air (an aircraft or
    jumpjet in flight) is left alone. A naval parasite inside an examined object is released
    with 500 ticks of suppression before any other effect of the warp on that object
    (S-0614 claim 12). This release occurs whether the object is killed, left in place or joins
    the warp group. An examined object with the organic flag and without the teleporter flag
    is killed even if it is protected or meets another condition that would leave it in place
    (claim 35). Every other examined object is left in place if it is protected, already has
    a warp pending, is being erased, is in a leaving stage, is phasing in, or is a vehicle still
    inside the vehicle factory with which it is in contact. Every remaining examined object
    joins the warp group: its tank-bunker link ends in place with walls-down effects
    (S-0606 claim 14), its docking, repair and similar contact links end, a vehicle ceases to
    occupy its cell, and the object receives the landing point defined in claim 36. Its subsequent warp follows claims 37 to 40.

    No owner test applies: enemy and neutral objects are warped or killed too. Passengers travel
    inside their transport; a mind-controlled object stays controlled.
    - Backing: `measurable` for recipients, parasite release and ordinary contact endings
      (SW12 parts b to f). Independent exclusions for an ongoing erasure and phasing in are
      `unmeasured`: these parts establish a pending or leaving warp, not a separately
      observable recipient held in either of those other states. Parasite release from an
      already protected host is also `unmeasured`: protection prevents infection and starting
      curtain protection already releases the parasite, so these tests cannot isolate it
    - Confidence: high

35. **Organic objects die.** An examined object with the organic flag and without the teleporter flag
    receives a hit of its type's whole strength, distance 0, with the demolition warhead, ignoring
    defences, with no attacker object and the launching house as attacking house. Protection
    does not prevent this death, so a protected infantry unit dies too
    (`quirks.warp_kill_before_protection`, default on = original; off: a protected organic object
    is left alone like any protected object).
    - Backing: `measurable` (scenario SW12 part b, SW13 part a)
    - Confidence: high

36. **Landing points.** Each member's landing point is in the destination cell that has the same
    position in the destination's effect block as its own cell has in the source's block (F1): the
    group keeps its 3 × 3 shape, with no rotation. A vehicle lands at that cell's centre. An
    infantry unit or aircraft keeps its position within the cell. On a high-bridge cell the bridge
    lift is added.
    - Backing: `measurable` (scenario SW12 part c)
    - Confidence: high; low for an object moving from one bridge deck to another (the lift may be
      added twice)

### Teleport landing and timing

37. **Landing check.** When a member is about to jump (claim 39), its landing cell's swept layer is
    checked. For each listed object there:
    - an unprotected infantry unit, when the member is infantry too, is killed only when it stands
      exactly on the member's landing point; otherwise both share the cell,
    - any other unprotected infantry unit, vehicle or aircraft is killed,
    - any protected object (also a shielded building) kills the member,
    - any other object (an unprotected building, a terrain object) blocks the landing.

    A high-bridge cell whose deck may not be entered also blocks it. The kills are hits of the
    victim's type's whole strength, distance 0, with the demolition warhead, ignoring defences,
    with no attacker and no attacking house. A blocked member looks for another cell with the
    nearby-cell search of S-0212 claim 25: centred on the landing cell, locomotion class *track*
    whatever the member's own, movement zone that of the member with *fly* and *destroyer* read as
    *normal* and *amphibious destroyer* as *amphibious*, reach region of the member's current
    cell, no height condition, bridge cells allowed, deck mode when the landing cell is a deck
    cell, no reference cell (the pick uses the tick number). Its landing point moves to the found
    cell with the same position within the cell; the check is made again in each later update
    until it passes (`quirks.warp_redirect_as_tracked`, default on = original; off: the member's
    own locomotion class and unsimplified movement zone are used).
    - Backing: `measurable` (scenario SW13 parts a to c)
    - Confidence: high for the outcomes; medium for the search settings

38. **Fate on arrival.** When a member is placed at its landing point:
    - if any protected object is listed in the landing cell's ground layer (even under a deck),
      the member is killed as in claim 37,
    - a member that lands on water, is not infantry, not naval, not a hover type (a hover type
      counts as not hovering when its type has the powered-unit flag and its owner has no active
      control centre, S-0405 claim 13), and not on a
      deck, sinks; its kill is credited to the launching house,
    - otherwise, a member for which the cell cannot be entered (S-0209), or a naval member landing
      on a deck cell or on a road cell (`quirks.warp_naval_road_cell`, default on = original; off:
      a road cell that is water is not counted), sinks with no credit when the cell is a water
      tile and the member is not infantry, and is otherwise killed with no credit, as in claim 37.
    - Backing: `measurable` (SW13 parts d and e). The independent effect of the naval road-cell
      test is `unmeasured`: ordinary land entry already kills a ship, and no stock water-road
      fixture isolating this extra test is established here
    - Confidence: high for the control-centre and road-cell tests; medium for other arrival outcomes

39. **Timing of a warp.** Let *t*_L be the tick of the warp launch (an order, applied after the turn
    pass, S-0203 claim 7). The following tick labels assume a surviving member with one warp
    update in every tick, without an interruption or failed placement. Each member:
    - enters its **leaving stage** in tick *t*_L + 1. Its fixed wait expires 60 elapsed ticks
      later, in tick *t*_L + 61 (an engine constant, not the rules' teleport delay). It still
      stays at its old position throughout that expiry update and departs on the following
      update. It counts as *being erased* from its update in *t*_L + 1 until departure in
      *t*_L + 62, including the expiry update; S-0214 and S-0301 give what that refuses,
    - leaves the map in its update of tick *t*_L + 62: the warp-out animation and departure sound
      play at its old position. Parasite release at this departure is unresolved for a drone
      parasite (S-0614 claim 12); a naval parasite was already released at launch (claim 34).
      At departure,
      every object targeting it drops it as in S-0207 claim 20 (e), it starts *phasing in*, and the
      landing check of claim 37 runs,
    - when the check passes, appears at its landing point in its next update, and in the update
      after that has its arrival effects: arrival sound, claim 38's fate, a surviving member's
      sensor-disc transfer and adjacency reveal (S-0516 claim 22), the warp-out animation at
      the new position, and the start of its arrival hold (claim 40). Sensor coverage remains
      at the old recorded cell through departure, retries and appearance; it transfers before
      the landing animation and hold, not at hold expiry. A member killed by the arrival-fate
      test gets no arrival sensor update. With a clean landing, appearance and arrival effects
      are ticks *t*_L + 63 and *t*_L + 64,
    - stops phasing in when the hold ends.

    When the 60-tick wait expires (*t*_L + 61), not at departure, and when the arrival hold
    ends, a member with no target tries a target search with the conditional 4-to-8-tick draw
    of S-0207 claim 20 (f). Thus the first search can consume draws one tick before the
    departure animation. The landing, redirection and placement take no draw; the animations
    take their creation draws (S-0202 claim 10). All timing arithmetic here is integer ticks,
    without rounding.
    - Backing: `measurable` for timing (SW12 part d); sensor coverage and transfer inherit
      S-0516 claim 22's backing (CS20 superweapon-warp part). Drone-parasite release on this path is
      `unmeasured`: the ordinary teleport release path is traced, but this superweapon path
      has not been shown to reach the same release; no result is promised here
    - Confidence: medium for the absolute tick timeline; low for drone-parasite release

40. **Arrival hold.** Let *t*_A be the tick of the arrival effects and *H* the member's integer
    *held delay*, in ticks. With one update per tick, phasing in ends in tick
    *t*_A + max(1, *H*), not in the arrival-effects update itself; no rounding is involved.
    Every object starts with a held delay of 0; a landing that had to be redirected (claim 37)
    sets it to the rules' teleport delay, and a teleport reinforcement arrival sets it to the
    reinforcement delay. Nothing resets
    it: a member that once had a redirected landing keeps that hold for every later warp, and a
    clean first landing has held delay 0, ending phasing in on the next update
    (`quirks.warp_hold_persists`, default on = original; off:
    the held delay is the teleport delay after a redirected landing and 0 otherwise, for each warp
    separately).
    - Backing: `measurable` for clean and redirected warps and persistent redirected holds
      (SW13 part c). Inheritance from teleport reinforcement is `unmeasured`: no scenario
      part here follows a reinforced object through a later superweapon warp
    - Confidence: medium

41. **Credit during a warp.** A member that sinks because it is a non-infantry, non-naval,
    non-hovering object arriving on water without a deck credits the launching house
    (claim 38). Sinking caused by failed cell entry, and every other death under claims 37
    and 38, credits nobody. Kills of claim 35 credit the launching house.
    - Backing: `measurable` (scenario SW13 parts a and d)
    - Confidence: medium

### Paradrop

42. **Paradrop dispatch.** A launch of either paradrop kind hands the target cell and the kind to
    S-0522 (claims 1 to 3 and 16), which decides the planes and their payload. When S-0522 sends
    nothing (for example an illegal cell, no paradrop plane type, or a rejected payload list)
    the launch still counts
    as carried out (S-0512 claim 8). There is no announcement (claim 5).
    - Backing: `measurable` for the missing-plane launch and charge consumption (SW14).
      Successful delivery and rejected payloads inherit S-0522 claims 1 to 3 and 16's backing.
      Illegal-cell orders are `unmeasured`: no permitted normal-input fixture here produces one
    - Confidence: high

### Spy plane

43. **Spy plane dispatch.** A launch of the spy plane kind at a legal map cell sends spy planes of
    the fixed spy plane type (Import mapping), each as a visiting aircraft (S-0501) with the target
    cell as target, no destination and behaviour *spy-plane approach*, placed over a cell on its
    house's arrival edge chosen as in S-0522 claims 4 and 5 (with the same draws), facing 0, with
    no passengers. If placement fails, that plane is removed and the next is still tried. The
    number of planes is the length of the first side group's resolved paradrop infantry list,
    for every side, and none when that compact list and its independently parsed count list
    differ in length. Resolution is S-0522 claim 2: unknown ordinary names create infantry
    types and count as entries, including names also used by a vehicle or aircraft kind.
    Empty items and reserved no-type names are omitted. Count resolved entries, not text
    tokens or only names explicitly registered in the infantry type list
    (`quirks.spy_plane_count_from_paradrop_list`, default on = original; off: exactly one plane).
    No plane, with an illegal cell or no spy plane type, still counts as carried out (S-0512 claim
    8). There is no announcement.
    - Backing: `measurable` for plane count and arrival (SW15 part a); resolved-list counting
      also inherits S-0522 claim 2's backing (P2). Failed placement and
      invalid-cell orders are `unmeasured`: no normal-input fixture is established here.
      A missing spy plane type is `unmeasured`: SW15 does not remove that type
    - Confidence: high

44. **The camera sight.** The spy plane reveals through its own sight, refreshed as a *camera*:
    - in a processing of *spy-plane approach* in which it already has its destination and its
      ground distance to the target (S-0204 claim 6) is at most its primary weapon's range, and in
      every processing of *spy-plane overfly* in which that holds,
    - a camera replaces the plane's sight with one centred on its own position (with its height,
      S-0409 claims 5 to 7), whose radius in cells is its primary weapon's damage instead of the
      computed sight radius (capped at 10 by S-0409 claim 6; a damage of 0 gives the ordinary
      radius). It is a sight held for its owner (S-0409 claim 2), not an area reveal,
    - in approach each camera also plays the camera sound at the plane; in overfly there is none.

    Approach is processed every camera-frame-count ticks and overfly every 3 ticks (S-0501 claim
    34), so the camera follows the plane in steps. Between cameras the ordinary sight refresh
    (S-0409 claim 8) may restore the ordinary radius. When the plane is removed or dies its sight
    ends (S-0409 claim 8); explored cells stay explored. A spy type with no primary weapon gives no
    camera (departure: the original does not handle it safely; our engine warns).
    - Backing: `measurable` for camera sight and sound (scenario SW15 parts b and c).
      The missing-weapon fallback is `unmeasured` design behaviour: the original has no safe
      reference outcome
    - Confidence: high for the radius, gate and cadence; medium for the interplay with the ordinary
      refresh

45. **Nothing else is special.** The camera adds no detection of cloaked or disguised objects
    (S-0516). A spy plane shot down before the target reveals nothing more. Its route and exit are
    S-0501's (claims 10 and 34).
    - Backing: `measurable` (scenario SW15 part d)
    - Confidence: medium

46. **Cloud playback and the weather observation boundary (Q-0732).** The cloud uses ordinary
    animation playback, not a separate one-frame-per-tick storm clock. The storm tests its
    stage against the frame count *F* of its current image, not against a stored middle-event
    field. S-0109 claims 2 to 5 supply the rate conversion, random delay, speed normalisation,
    first-update exclusion and missed-update rules. In particular a zero stage delay freezes
    the stage; it is not replaced by 1 tick. The cloud is created with one pass and without
    a creator-requested loop delay, reverse or damage suppression. Imported playback flags
    and fields still apply under S-0109 claims 6 to 17.

    For an ordinary forward cloud with start field 0, end equal to *F*, no follow-up or
    special playback flags, no missed turns and effective delay *d* ≥ 1 ticks, let *c* be
    its creation tick and *u* its first animation-update tick. Its first advance is
    *a* = max(*c* + *d*, *u* + 1), and advance *j* ≥ 1 is in tick
    *a* + (*j* − 1)*d*. All quantities are integers; no rounding beyond the imported delay.
    Weather precedes animation turns, so for *F* ≥ 3 the strike is in tick
    *a* + ⌊*F*/2⌋*d* + 1, and the cloud ceases to be present in tick
    *a* + (*F* − 2)*d* + 1. In the usual before-turn creation case *u* = *c*, these
    reduce to *c* + (⌊*F*/2⌋ + 1)*d* + 1 and *c* + (*F* − 1)*d* + 1.
    The latter is not the animation's deletion tick. A cloud may strike and cease to be
    present in the same weather update. With *F* of 1 or 2 under these same playback
    conditions it is removed before a striking stage survives to a weather update and
    never strikes. For other imported
    playback, use the actual stages under S-0109 rather than these forward-pass formulas.

    S-0109 claim 25's middle image frame equals ⌊*F*/2⌋ only when the image that supplied
    that derived field has the same frame count as the cloud's current image. A theater
    replacement can break that equality. Neither *F* = 2μ nor a guessed missing middle
    field reconstructs *F*: odd frame counts already disprove that equation.
    - Backing: `measurable` (SW20); unusual playback inherits S-0109's individual backings
    - Confidence: high

47. **Busy silo interval (Q-0735).** With one ordinary building turn each tick and no
    intervening order, sale, erasure or interruption, a silo accepting a post-turn launch
    in tick *t* emits its upward missile in *t* + 1, changes its launch picture in *t* + 2,
    and queues *guard* in *t* + 8. A second post-turn launch starts a new sequence in
    *t* + 8 or later, not immediately after take-off. The new missile leaves on the next
    building turn. The 1-tick and 6-tick processing waits are engine constants and do not
    depend on image frame counts, animation rates or the animation-completion flag.

    While launch is current and no different behaviour is queued, another launch does not
    queue a missile or restart the sequence. It still replaces the house's recorded target
    and plays the launch sounds (claim 7). An already-created upward missile keeps its own
    target; a replacement before its creation changes that missile's target. If a different
    behaviour is already queued, the new launch replaces that queued behaviour and starts a
    fresh launch immediately, even while launch is current. These are the ordinary current
    and queued behaviour slots of S-0215, not a missile queue. The initial 8-tick result
    assumes those slots are not otherwise changed.
    - Backing: `measurable` (SW21); exceptional externally queued behaviours are
      `unmeasured` here because no independent normal-input fixture is established
    - Confidence: high for processing waits and the queue rule; medium for uninterrupted
      absolute tick labels before oracle confirmation

48. **Other work during a warp (Q-0738).** A
    vehicle, infantry unit or landed aircraft that is still in its leaving stage after
    its warp update skips ordinary self-healing, behaviour processing, movement, firing
    and turret turning in that turn. It drops target and destination. S-0215 claim 2
    already owns that behaviour-processing exclusion; S-0314 claims 9 and 10's healing
    requires reaching ordinary update work. Thus those turns must not be treated as
    ordinary living turns with only driving disabled. The pending
    application is not itself that leaving-stage test: the member's first following turn
    can reach ordinary work before entering the leaving stage. Its former driving
    locomotion does not continue driving it while the superweapon warp controls movement.
    Protection's elapsed duration still runs even on a turn that skips its brightness
    phase processing (claim 29).

    For a vehicle, infantry unit or landed aircraft whose turn begins in arrival hold,
    the hold-end test precedes any ordinary healing, behaviour, movement, firing or turret
    work in that turn. This answers the ordering question, not whether each such operation
    is eligible to run. It does not make the warp step the whole turn.

    The following table explicitly marks the remaining answers rather than adopting the
    request's interim whole-turn-only rule. Each row applies separately to vehicles,
    teleporter infantry and landed aircraft, and to each of self-healing, behaviour
    processing, ordinary movement, firing and turret turning where that kind has it.

    | Boundary in the request | Ordinary-work answer | Backing and confidence |
    |---|---|---|
    | Joining at launch | Former driving does not continue under the warp. Immediate clearing of the path, queued destinations and driving step is not established here | Stopped driving: measurable, SW23(a), high. Exact clearing: unmeasured, SW23(a), low |
    | First pending turn, before leaving starts | Pending alone is not the exclusion. Whether each ordinary operation occurs before that turn enters leaving is unmeasured | Unmeasured, SW23(a), low |
    | Turn still in leaving after its warp update, including wait expiry | No ordinary healing, behaviour, movement, firing or turret work; target and destination dropped | Measurable, SW23(b), high |
    | Departure turn, after leaving ends | Continuation of each ordinary operation is unmeasured | Unmeasured, SW23(c), low |
    | Off-map landing-check retries | Whether any work other than warp work occurs is unmeasured; being off-map alone is not evidence of a whole-turn exclusion | Unmeasured, SW23(c), low |
    | Placement update | Continuation of each ordinary operation after placement is unmeasured | Unmeasured, SW23(d), low |
    | Arrival-effects update | Continuation of each ordinary operation after effects is unmeasured. The surviving sensor update is independently settled by claim 39 | Unmeasured for ordinary continuation, SW23(d), low |
    | Arrival hold, including the hold-expiry turn | Hold-end test precedes ordinary-work opportunities; eligibility and continuation of each ordinary operation remain unmeasured | Ordering: measurable, SW23(e), high. Continuation: unmeasured, SW23(e), low |

    The unmeasured entries are incomplete evidence, not a claim of an unstable original
    result and not permission to treat the request's workaround as confirmed behaviour.
    The per-kind interaction of early warp work, ordinary-work eligibility and a later
    movement opportunity has not been fully established. In particular claim 39/T2's
    one-warp-update-per-tick assumption does not establish that count for every kind.
    SW23 is the discriminating proposal for every marked gap.
    - Backing: `measurable` for leaving exclusions, stopped driving and hold-test ordering
      (SW23); `unmeasured` for each explicitly marked continuation or clearing entry above,
      for the incomplete per-kind eligibility trace just described
    - Confidence: high for the confirmed boundaries; low for the marked gaps

49. **Exceptional warp cells (Q-0741).** The three questions have these separate answers.

    | Case | Answer | Backing and confidence |
    |---|---|---|
    | Source marker creation | Ordinary scenario-stream animation-creation draws occur regardless of the local viewer; only visibility and audio are local. This was already answered by claim 33 | Measurable, SW22, high |
    | A source object's corresponding destination cell is off-map: parasite and organic tests | Claim 34's naval-parasite release and claim 35's organic hit still occur. Off-map destination is not an early refusal that protects either | Measurable, SW24(a), high |
    | Same off-map case: otherwise eligible survivor | There is no destination-legality rejection before joining the warp. Its eventual position, placement success and death or survival are explicitly unmeasured; neither leaving it at source nor immediate death is established | No early rejection: measurable, SW24(a), high. Final outcome: unmeasured, SW24(a), low |
    | Blocked landing, nearby search finds no cell | The failed result replaces the landing point without a success test; it does not preserve the old point. The unsuccessful check remains in the landing-retry stage. A retry sets the held delay to the teleport delay, including when the check subsequently passes | Measurable, SW24(b), high |
    | Failed-search conversion and later fate | The exact replacement world point and its subsequent placement, death or survival are explicitly unmeasured. Subsequent checks use the replaced point, not a promised repeat at the original blocked point | Unmeasured, SW24(b), low |

    These gaps arise from the incomplete invalid-cell point-conversion and placement trace,
    not an established undefined result. No safe fallback, immediate kill, or retained-point
    retry is prescribed as original behaviour. S-0212 claim 25 owns search candidate
    selection; claim 37 owns this caller's settings and treatment of its result.
    - Backing: `measurable` for the confirmed rows (SW22 and SW24); `unmeasured` for the
      explicitly marked world-point and final-fate rows, for the incomplete trace above
    - Confidence: high for the confirmed rows; low for the marked gaps

## Formulas

- **F1. Effect block.** For centre cell (*x*, *y*), the nine cells in order are
  (*x* + *a*ᵢ, *y* + *b*ᵢ) with (*a*ᵢ, *b*ᵢ) = (0, 0), (1, 0), (1, −1), (0, −1), (−1, −1),
  (−1, 0), (−1, 1), (0, 1), (1, 1). Integers, cells. A cell that is not on the map is skipped.
  A warp maps source cell *i* to destination cell *i*.
- **F2. One-time missile start height.** *z*₀ = *h*(*c*) + *b* + 20 000 leptons, *h*(*c*) the
  terrain surface height at the cell centre, *b* = 416 on a high-bridge cell and 0 otherwise.
  Integer leptons. The start velocity is (0, 0, −100) leptons per tick. (The original builds both
  missiles' start velocities from table sines and cosines of an angle just below a right angle in
  floating point; every horizontal part truncates to 0, so the vectors are exactly vertical.)
- **F3. Force-shield test.** A building qualifies when
  ⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋ < 256 × *R*, with *r* the table square root of S-0204 claim 5
  (`quirks.table_root_distance`), Δ the difference between the building's reference point and
  (256*x* + 128, 256*y* + 128, *h*), *h* the terrain surface height at the target cell centre, and
  *R* the force-shield radius in cells. Coordinate differences are integer leptons; their
  products and sum are formed in floating point, passed to the table square root and truncated
  toward zero to integer leptons before the strict comparison. S-0205 claim 5 gives the
  floating-point rounding policy.
- **F4. Cloud height.** *z* = *h*_level + *b*₄ + *h*_c, with *h*_level the cell's height level times
  the level height, *b*₄ the lift of 4 levels on a high-bridge cell, and
  *h*_c = trunc((trunc(*H* / 2) − 0.5) × *k*) leptons, *H* the pixel height of the image of the
  first bolt animation, trunc(*H* / 2) an integer division toward zero, and
  *k* = 362.0386657714844 / (0.865897536277771 × 60) ≈ 6.968466256176568 (the original's table
  square root of 2 × 256² divided by 60 times its table sine of 60°). The original computes the
  product in double-precision floating point with S-0205 claim 5's rounding policy, then
  truncates toward zero to integer leptons. The operations defining *k* use that same policy. Examples:
  image heights *H* of 0 or 1 pixels give −3 leptons, 2 pixels give 3 leptons,
  100 pixels give 344 leptons, and 200 pixels give 693 leptons. *H* is the image header's
  canvas height, not a frame's stored rectangle height or the cloud image's height.
  This is a world-position value, not merely a drawing displacement. The lightning damage
  point and bolt point of claim 18 do not inherit it, but the cloud's own ordinary animation
  damage is delivered at its own position (S-0109 claim 17); ordinary animation middle
  events also retain their height tests (S-0109 claim 14). Import it even when an ordinary
  stock cloud delivers no such damage. A missing first-bolt image does not supply a
  readable zero-height header in the original: that cloud-creation path is unsafe.
  Our engine warns and makes no cloud for that request (bad-data departure, no switch),
  rather than claiming the original uses *H* = 0. Backing: `measurable` (SW8 part c,
  SW25) for existing images and height-dependent animation effects; `unmeasured` design
  for the missing-image fallback. Confidence: high.
- **F5. Storm timing.** A request in tick *T* before the weather update, with countdown *L* ≥ 1 and
  none running, starts the storm in tick *T* + *L* − 1. Approach warnings come in ticks
  *T* + *L* − 225*k* − 1 for integers *k* ≥ 1 for which that tick is at least *T* and less
  than the start tick. A countdown-started storm in tick *t*_s with integer duration *D* ≥ 0
  makes clouds in ticks *t*_s + 1 … *t*_s + *D* that are multiples of the periods, and begins
  clearing in tick *t*_s + *D* + 1. A zero-countdown request made before the weather update can
  also make clouds in its start tick. Duration −1 disables the duration end test. All timing
  arithmetic is integer ticks, without rounding.
- **F6. Scatter displacement.** *r* = ⌊*s* / 2⌋ (toward minus infinity for negative *s*); Δ*x*
  = R(−*r*, *r*), then Δ*y* = R(−*r*, *r*) (S-0202 claim 4; no draw when the bounds are equal); candidate
  (*x*_c + Δ*x*, *y*_c + Δ*y*); fails when |*x* − *x*′| + |*y* − *y*′| < separation for any
  present cloud cell (*x*′, *y*′). Integers, cells.
- **F7. Protection window.** Protected in tick *t* exactly when *t* − *t*₀ < *D*, i.e. for *D*
  ticks counting the start tick, provided *t* ≥ *t*₀ and *D* > 0. Integer ticks, no rounding.
- **F9. Protection brightness.** For ordinary non-negative integer brightness *b* in per-mille
  and claim 29's dimensionless phase factor *m*, the protection-adjusted brightness is
  *b*′ = min(2000, ⌊*b* × *m* / 256⌋) per-mille. The multiplication is integer and the
  division truncates toward zero. Before the first protected turn, after the envelope's
  Ordinary phase begins, and without protection, brightness is unchanged, not newly capped.
  This formula isolates protection with no simultaneous damage flash or air-strike marking.
- **F8. Silo-launched falling missile start height.** *z*₀ = *h*(*c*) + *A*_up, where *h*(*c*)
  is the terrain surface height at the target cell centre and *A*_up is the upward projectile
  type's detonation-altitude value. Integer leptons, no rounding or bridge lift. This is a
  creation-height formula only; each missile's vertical-flight detonation test still compares
  its new absolute world height against its own type's threshold (S-0306 claim 11).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Upward missile top speed | 255 | leptons per tick | silo launch, claim 8 |
| Upward missile start speed | 10 | leptons per tick | claim 8 |
| Falling missile start speed, silo | 1 | leptons per tick | claim 9 |
| One-time missile start height | 20 000 | leptons above surface | claim 10, F2 |
| One-time missile start speed | 100 | leptons per tick | claim 10 |
| Storm warning interval | 225 | ticks | claim 14 |
| Storm message display | 150 | ticks | claims 14, 15 |
| Scatter tries per period | 3 | tries | claim 16 |
| Cloud height factor *k* | 6.968466256176568 | leptons per pixel | F4 |
| Invoke animation lift | 5 | leptons | claims 22, 31, 33, 34 |
| Curtain parasite suppression | 50 | ticks | claim 25 |
| Warp parasite suppression | 500 | ticks | claim 34 |
| Warp leaving wait | 60 | elapsed ticks | expires one update before departure, claim 39 |
| Tint draw range | −5 … 5 | ticks | claim 29 |
| Spy overfly processing period | 3 | ticks | claim 44, S-0501 |
| Effect block | 3 × 3 | cells | F1 |

Absent-key defaults are in Import mapping.

## State and ordering

**T1. The storm.**

| State | Event | Result |
|---|---|---|
| idle | request with countdown *L* > 0 | counting down; centre, owner, duration set; remainder *L* |
| idle or counting down | request with countdown 0 | raging (claim 15) |
| counting down | request with countdown *L* > 0 | centre, owner, duration replaced; remainder min(remainder, *L*) |
| counting down | weather update | remainder − 1; at 0: raging, no cloud this update; on positive multiples of 225: warning |
| raging | weather update, start + duration < now (duration ≠ −1) | clearing; no cloud this update |
| raging | weather update otherwise | centre and scatter clouds per claim 16 |
| raging | stop request | clearing |
| raging or clearing | request | nothing |
| clearing | weather update with no present cloud | idle; owner none; lighting re-evaluated |
| any | weather update | pending clouds strike first (claim 18) |

**T2. A warp group member under claim 39's stated one-update-per-tick, uninterrupted-survivor assumption.**

| State | Event | Result |
|---|---|---|
| examined | joins the group (claim 34) | warp pending; landing point fixed |
| warp pending | update in *t*_L + 1 | leaving stage; counts as being erased; fixed wait starts |
| leaving stage, waiting | update in *t*_L + 61, 60 elapsed ticks after the wait starts | remains at old position and still counts as being erased; target search if targetless |
| leaving stage, wait expired | update in *t*_L + 62 | leaves the map; being-erased state ends; phasing in; landing check |
| check failed | each update | redirected search and check again; held delay set to the teleport delay |
| check passed | next update | placed at the landing point |
| placed | next update | arrival effects; arrival hold starts (claim 40) |
| arrival hold | update in *t*_A + max(1, *H*) (claim 40) | phasing in ends; target search if it has no target |

**Ordering consequences.**
- The storm's update comes before the turn pass (S-0203 claim 1, row 3), so an object a strike
  kills takes no turn in that tick, and within one update every pending cloud strikes before any
  new cloud is made.
- For a curtain or warp, the observable recipient sequence and resulting scenario-stream state
  match F1's cell order and S-0209 claim 13's listing order within each cell. Backing:
  `measurable` (SW9 part g); confidence: high.
- With `quirks.superweapon_list_cut_by_removal` on (default = original), when a reached object
  leaves its cell immediately during its own curtain or warp effect, later listed objects in that cell are not
  reached. With it off, objects listed when the effect begins remain eligible to be reached.
  This distinction also applies to a warp's source effects and landing conflicts when a victim
  is removed immediately. It does not apply to genetic mutation in cell mode (S-0611 claim 4),
  which still hits later listed infantry after a conversion removes an earlier victim.
  Backing: `unmeasured` for the curtain and warp list cut: no fixture here establishes immediate removal
  during the effect, rather than a death that completes later. SW9 part g is an exploratory
  search for such a witness, not a guaranteed test of this condition. Confidence: low.
- A fire order's draws come after the turn pass (S-0203 claim 7); a warp member's later draws come
  in its own updates.

## Edge cases and quirks

- **`quirks.superweapon_order_first_type_of_action`** (claim 2). Default on.
- **`quirks.superweapon_kind_as_list_position`.** Default on = original. Three effect-side
  readings use a kind number as a position in the superweapon type list: the silo fires the
  weapon of the type at the nuclear-strike kind's position (claim 8); a teleport-source pick arms
  for the launching player the type at the warp kind's position; and the radar's superweapon
  offers (S-0410 table C1) are chosen by list position. With stock data the positions and kinds
  agree. Off: each reading uses the type of that kind (the fired type for the silo). S-0512's
  `quirks.predependent_kind_as_list_position` is separate. Backing: `measurable` (SW2 part g
  for the silo, SW12 part h for source aiming, SW1 part e for radar offers); confidence: high.
- **`quirks.nuke_silo_first_type_only`** (claim 7). Default on.
- **`quirks.silo_target_of_current_owner`** (claim 13). Default on.
- **`quirks.superweapon_block_one_layer`** (claim 4). Default on.
- **`quirks.curtain_organic_vehicle_defended`** (claim 24). Default on.
- **`quirks.protection_restart_replaces`** (claim 25). Default on.
- **`quirks.superweapon_list_cut_by_removal`** (State and ordering). Default on. Low confidence
  that a stock victim is immediately removed during an effect; SW9 part g proposes a search
  for a named-data witness, not an established fixture.
- **`quirks.warp_kill_before_protection`** (claim 35). Default on.
- **`quirks.warp_redirect_as_tracked`** (claim 37). Default on.
- **`quirks.warp_naval_road_cell`** (claim 38). Default on.
- **`quirks.warp_hold_persists`** (claim 40). Default on.
- **`quirks.spy_plane_count_from_paradrop_list`** (claim 43). Default on.
- **Departures (no switch):** the storm outage after the local player's defeat (claim 15); the
  post-launch computer draw (claim 6); the firing answer against protected objects (claim 28); unsafe
  storm data (claim 21); a spy type with no weapon (claim 44); a missing payload weapon or
  silo weapon, or a missing invoke, marker or blast animation type: no projectile or no animation
  is created and the importer warns (the original does not handle these safely). These invalid-data
  fallbacks are `unmeasured` design rules: the original provides no safe reference result.
- **Presentation fixed outright.** A warning marker is removed when its owner house leaves play.
  The original clears the wrong link before removing the marker. Backing: `unmeasured`: SW4
  tests visibility and missile expiry, not owner-house removal. Confidence: medium.
- **A missile with no launching building** is S-0512's (`quirks.missile_without_launcher_consumes_charge`); claim
  7 defines when the house is able to launch.
- **A nuclear-strike order on a silo already in its launch sequence** with nothing queued is lost
  (claim 7); the charge is S-0512's.
- **A storm request's centre and owner** are stored by the original even when a storm is raging,
  although no caller reaches that case; our engine leaves a raging storm unchanged (claim 14).
  Backing: `unmeasured`: no permitted input reaches the otherwise observable owner replacement.
  Confidence: high.

## Import mapping

Units in the Form column also apply to numeric absent-key defaults in that row. Integer
parsing follows S-0003 claims 3 and 4 and the integer domain of S-0205 claim 1. Health points are
S-0302's damage and strength units: storm damage is the delivered amount before target
modifiers, not a promise of that many health points lost. Radiation-level units are defined
in Terms and are not damage units. A selector is a dimensionless integer index, not a colour
component or a time.

The absent-key defaults have high confidence and `unmeasured` backing here: the scenarios
below retain inherited fields or explicitly override them, rather than removing each key
from every rules layer in a fresh session to isolate its default. Delegated field semantics
retain the backing of their owning specs. The two cursor-circle fields have high confidence
and `unmeasured` backing here: no scenario part varies them to isolate their presentation.
Protection colour selection is `measurable` under SW10 part g. The emergency nuclear fallback is also `unmeasured`: no scenario
can reliably force its effect-creation failure through ordinary input.

| Source | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| superweapon type sections | `Type` | superweapon kind (S-0512 claim 24; psychic reveal: S-0610 claims 13 to 15; genetic mutation: S-0611 claims 2 to 5; psychic domination: S-0612 claims 1 to 17) | case-insensitive complete word from S-0512 claim 24's Import mapping vocabulary | omitted, empty or unknown retains prior layer; initially no kind |
| superweapon type sections | `Action` | aim action, claims 1 and 2 | case-insensitive complete word from S-0512 claim 23's Import mapping vocabulary | omitted or empty retains prior layer; initially no action; unknown nonempty word clears action (S-0512 claim 23) |
| superweapon type sections | `WeaponType` | the silo's upward weapon (claim 8) | weapon name | none |
| superweapon type sections | `StartSound`, `SpecialSound` | force-shield sounds (claim 32) | sound name | none |
| superweapon type sections | `AIDefendAgainst` | computer defence check (claim 6; AI module only) | boolean | no |
| superweapon type sections | `Range` | cursor-circle radius (presentation only; no game rule) | decimal, held in single-precision floating point, cells | 0 cells |
| superweapon type sections | `LineMultiplier` | cursor-circle subdivisions (presentation only; no game rule) | integer, subdivisions per cell; non-positive selects a single circle | 0 subdivisions per cell |
| superweapon type sections | `AuxBuilding` | giving-building condition only (S-0512 claims 3, 6 and 7); not a launcher-selection condition (claim 7) | building type name | none (S-0512 Import mapping) |
| building type sections | `NukeSilo` | silo (claim 7) | boolean | no |
| building type sections | `SuperWeapon`, `SuperWeapon2` | the type's superweapon fields (claim 7, S-0512) | type name | none |
| building type sections | `PsychicDetectionRadius` | warning-marker visibility (claim 11) | integer, cells | 0 |
| building type sections | `WeaponsFactory` | vehicle factory (claim 34) | boolean | no |
| warhead sections | `NukeMaker` | missile-maker flag (claim 8) | boolean | no |
| projectile sections | `DetonationAltitude` | burst height (claims 8, 9) | integer, leptons | 0 |
| weapon section `[NukePayload]` | fixed weapon type name | payload weapon (claims 9, 10) | name | no payload: departure |
| weapon sections | `RadLevel` | radiation hand-off (claim 12) | integer, dimensionless radiation-level units | 0 radiation-level units |
| object type sections | `Organic` | organic flag (claims 23, 24, 35) | boolean | yes for infantry, no otherwise |
| object type sections | `Teleporter` | teleporter flag (claim 35) | boolean | no |
| object type sections | `Naval`, `SpeedType`, `PoweredUnit`, `MovementZone` | arrival fate and redirect (claims 37, 38) | as their owning specs | as their owning specs |
| object type sections | `Strength` | whole-strength hits (claims 23, 24, 35, 37) | integer, health points (S-0302 claim 12) | 0 health points (S-0302 Import mapping) |
| `[General]` | `NukeTakeOff` | silo take-off animation (claim 8) | animation name | none |
| `[AudioVisual]` | `DigSound` | nuclear siren (claim 5) | sound name | none |
| animation type `PSIWARN` | fixed animation type name | warning marker (claim 11) | name | none: no marker |
| `[SpecialWeapons]` | `NukeProjectile`, `NukeDown` | read and kept, unused by these launches | names | none |
| `[SpecialWeapons]` | `NukeWarhead` | emergency detonation fallback when the delayed nuclear effect cannot be created, not the normal payload warhead | warhead name | none |
| `[General]` | `LightningDeferment` | storm countdown *L* | integer, ticks | 250 |
| `[General]` | `LightningStormDuration` | storm duration *D* (−1 endless); radar outage length | integer, ticks | 900 |
| `[General]` | `LightningHitDelay` | hit period | integer, ticks | 90 |
| `[General]` | `LightningScatterDelay` | scatter period | integer, ticks | 10 |
| `[General]` | `LightningCellSpread` | cell spread *s* | integer, cells | 10 |
| `[General]` | `LightningSeparation` | separation | integer, cells | 3 |
| building art sections | `Foundation` | dimensions and listed foundation cells for building strike witnesses (claim 18) | named foundation vocabulary of S-0108 claim 8; cell exceptions of S-0406 claim 2 | `1x1` (S-0108 claim 8) |
| `[General]` | `LightningDamage` | storm damage, delivered before target modifiers (S-0302 claim 1) | integer, health points | 200 health points |
| `[General]` | `LightningWarhead` | storm warhead | warhead name | none: departure (claim 21) |
| `[General]` | `LightningPrintText` | storm-message flag | boolean | yes |
| `[General]` | `WeatherConClouds` | cloud animation list | list of animation names | empty: departure |
| `[General]` | `WeatherConBolts` | bolt animation list; first entry's image header canvas height in F4 | list of animation names; image resolved under S-0007 claim 14 | empty: claim 21; nonempty list with missing first image: no cloud and warning (F4) |
| resolved cloud image | image header frame count | *F* in claims 16, 18 and 46 | integer count of image frames, not twice the middle field | missing image: unsafe storm threshold access; no cloud and warning in our engine |
| cloud animation art section | `Rate`, `RandomRate`, `Normalized`, `Start`, `End`, `LoopStart`, `LoopEnd`, `LoopCount`, `Reverse`, `PingPong`, `Shadow`, `Next` | actual stage history used by claim 46 | forms and defaults of S-0109 claims 2 to 9, 15 and 25; creator supplies one pass | S-0109 Import mapping, not an invented delay of 1 |
| map `[Actions]` | action code 109 | waypoint curtain, claim 22 | S-0011 claim 33's waypoint slot, eighth action item, letter-encoded waypoint; no number-slot house or duration | S-0011 claim 33's missing/invalid waypoint rules |
| `[General]` | `WeatherConBoltExplosion` | storm explosion animation | animation name | none |
| `[General]` | `MetallicDebris` | debris list (claim 18) | list of animation names | empty: no debris |
| `[AudioVisual]` | `StormSound` | storm sound (claim 15) | sound name | none |
| `[AudioVisual]` | `LightningSounds` | lightning sounds (S-0701 claim 25) | list of sound names | empty |
| `[CombatDamage]` | `IronCurtainDuration` | curtain duration | integer, ticks | 0 |
| `[CombatDamage]` | `C4Warhead` | demolition warhead (claims 23, 24, 35, 37, 38) | warhead name | none (S-0302) |
| `[General]` | `IronCurtainInvokeAnim` | curtain invoke animation | animation name | none |
| `[General]` | `ForceShieldInvokeAnim` | force-shield invoke animation | animation name | none |
| `[General]` | `ForceShieldRadius` | force-shield radius *R* | integer, cells | 10 |
| `[General]` | `ForceShieldDuration` | force-shield duration *D* | integer, ticks | 400 |
| `[General]` | `ForceShieldBlackoutDuration` | blackout length (claim 30) | integer, ticks | 800 |
| `[General]` | `ForceShieldPlayFadeSoundTime` | fade-sound lead time *P* | integer, ticks | 50 |
| `[AudioVisual]` | `IronCurtainColor`, `ForceShieldColor` | curtain and force-shield colour-adjustment selectors, respectively (claim 29) | integer, zero-based position in the colour-adjustment table; not a colour-definition name | 0 (selector index); absent in a later layer retains the held selector |
| `[ColorAdd]` | each entry, in section order; key spelling does not select a slot | RGB colour-adjustment triples (claim 29); layering owned by S-0006 claim 2 | three byte components, red, green, blue, parsed under S-0003; not palette components multiplied by 4 | table initially has 16 black triples (0, 0, 0); omitted entries and rules-layer clearing follow S-0006 claim 2 |
| `[General]` | `ChronoPlacement` | source marker | animation name | none |
| `[General]` | `ChronoBlast`, `ChronoBlastDest` | blast at the source, blast at the destination | animation names | none |
| `[General]` | `WarpOut` | warp-out animation (claim 39) | animation name | none |
| `[General]` | `ChronoInSound`, `ChronoOutSound` | arrival and departure sounds; a type's own keys override | sound names | none |
| `[General]` | `ChronoDelay` | teleport delay (claim 40) | integer, ticks | 60 |
| `[General]` | `ChronoReinfDelay` | reinforcement delay (claim 40) | integer, ticks | 60 |
| aircraft type `PDPLANE` | fixed type name | paradrop plane (S-0522) | name | none: nothing sent |
| aircraft type `SPYP` | fixed type name | spy plane type (claim 43) | name | none: nothing sent |
| `[General]` | `AllyParaDropInf`, `AllyParaDropNum` | spy plane count (claim 43) | creating infantry-reference list and independent integer-count list, S-0522 claim 2; compare resolved compact lengths, not text-token counts | empty: no plane |
| No data key | no imported field | warp turn continuation and exceptional-cell rules (claims 48 and 49) | engine behaviour; unmeasured boundaries are explicitly marked in those claims; no INI option supplies the missing answers | no configurable default |
| spy type's primary weapon | `Damage` | camera radius (claim 44) | integer, cells | 0 |
| spy type's primary weapon | `Range` | camera gate (claim 44) | cells times 256, truncated | 0 |
| `[AudioVisual]` | `SpyPlaneCamera` | camera sound | sound name | none |
| `[AudioVisual]` | `SpyPlaneCameraFrames` | approach processing period (S-0501 claim 34) | integer, ticks | 16 |
| `[General]` | `AISuperDefenseDistance`, `AISuperDefenseProbability`, `AISuperDefenseFrames` | the original's computer defence (claim 6); not read by our game rules | — | — |

The storm-created, storm-approach and storm-active/refusal messages use their corresponding imported string-table entries. Announcement entries are S-0701's Import mapping.

## Measurement scenarios

All runs: stock data unless a named test file is stated, skirmish mode, director and mods off,
800 × 600 resolution, game speed index 2, superweapons on. Each part uses a tiny purpose-made
map and is run three times; draw tests use ten independently seeded runs, recording the seed
without setting it. Observe through scripted normal mouse/keyboard input, video, a read-only
tick counter and the permitted read-only random-state recorder. A named test file below means a map
override named after its scenario and part (for example SW9-c.map), removed after that part,
except for the type-list reordering tests SW1e, SW2g and SW12h. Those require named loose
rules-file overrides, recorded as SW1-e.rulesmd.ini, SW2-g.rulesmd.ini and SW12-h.rulesmd.ini,
respectively, and installed under the normal rules filename only for that part. A map layer
cannot reorder types already registered by lower layers. Restore the original file situation
after each part.
Record ticks, health, kills, cells, animation frames and explored footprints with tick labels.
A mismatch with any part's expected result falsifies that part's claim, including an unexpected
draw, changed recipient, changed credit or off-by-one tick. Computer houses' bases are kept
beyond the imported computer-defence distance from every target unless stated (claim 6); a run
is void if an unlisted bystander acts on the observed area. The superweapons are
granted with the one-time or permanent trigger grants of S-0512 claims 12 and 13 where a building
is not under test.

- **SW1 — targeting (claims 1 and 2).** (a) Aim a nuclear strike, a weather storm and an
  invulnerability at a fully shrouded cell; expected: each is carried out. (b) Hover a force shield
  over an enemy building, open ground and an own building; expected: no-force-shield cursor,
  no-force-shield cursor, offer; a click on the first two sends no order and keeps the charge.
  (c) With units selected, hover any superweapon; expected: the units' cursor. (d) Named test file
  giving two types the same aim action; select the second and click; expected: the first type's
  record fires (or is refused if not ready). The raw-order cases of claims 1 and 3 have no
  permitted normal-input oracle fixture and are not claimed measurable here. (e) Named rules file
  swaps a radar-allowed type and a radar-disallowed type in the type list, preserving their
  kinds and distinct aim actions. Select each through its sidebar icon and hover the radar.
  Expected: offers follow the list positions of S-0410 claim 12 table C1, not the relocated
  kinds; offers following kinds instead falsify the position rule.
- **SW2 — silo launch (claims 7, 8, 9, 12, 13).** Flat map, one house with two silos built 300
  ticks apart, the second nearer the target. (a) Fire at a distant cell; record the tick of the
  order, of the take-off animation, of the upward missile's disappearance and of the nuke flash; expected:
  take-off on the tick after the order, flash at the burst plus the falling time from
  claim 9, then S-0306 claim 25's wait. (b) Expected: the first-built silo launches. Capture a silo built earlier by another house
  after both own silos exist; expected: the longest-owned silo still launches, not the oldest
  by creation tick. (c) Named test
  file with a second silo type listed later; the house owns only that one; expected: nothing
  launches and the charge is spent. (d) Kill a target with the strike; expected: the silo's house
  credited. In a named map give the payload a positive radiation level and a fractional
  warhead spread. Record the field's level and spread read-only immediately at detonation:
  expected the imported level and spread truncated to whole cells. Repeat at the same cell
  before the existing field ends, recording its decayed current level immediately before the
  second hit using S-0525 claim 4's floating-point calculation and truncation:
  expected that current level plus the imported level, not the original level plus the imported
  level. A different sum or a non-truncated initial
  spread falsifies the corresponding part of claim 12. This same-spread repeat does not
  distinguish retaining the old spread from replacing it with an equal value. (e) Sell the first-built silo on the order
  tick (sale accepted first, use a named map that grants the ready record and slows sale),
  expected: no missile, launch announcement and siren still heard, charge spent. Destroy the silo
  while the upward missile climbs; expected: the strike's kills credit nobody. Capture it during
  ascent instead; expected: the capturing house receives the kills. For a busy-silo test use two
  ready nuclear types named by the silo's two superweapon fields and fire the second during the
  first's launch sequence with no queued behaviour; record missile count and replacement target,
  not only sound. Expected: the second order replaces the house's target and plays the sounds,
  but starts no second missile sequence. A second missile sequence falsifies the busy-silo rule.
  (f) Every player hears the launch announcement and siren. Falsified by another silo launching
  or by a delay before take-off. Capture just before the next update remains unmeasured.
  (g) Named rules file moves the fired nuclear type away from index 0 and gives the type at index 0
  a visibly different upward weapon, retaining valid missile-producing projectiles for both.
  Expected: the silo fires the index-0 type's weapon, not the fired type's weapon. The latter
  projectile instead falsifies the list-position weapon rule.
  (h) Use SW2-h.map with a silo on flat ground and targets at two different terrain levels,
  plus a high-bridge target. Keep the weapons and projectile types unchanged. Record each
  missile's world height and vertical velocity read-only at creation and on each turn,
  together with the target surface height, upward type's altitude threshold and tick labels.
  Predict the upward flight with S-0306 claim 11 and formula F10, including threshold overshoot.
  Repeat with a silo on raised terrain firing at the original flat target, to distinguish
  world height from height above the silo. Expected: the upward altitude test uses the same
  absolute threshold for all targets and both silo elevations. The falling missile starts at
  F8, rising by exactly the target surface-height difference
  between runs and gaining no bridge lift. A start at the upward missile's actual burst
  height, at the threshold alone, or with added deck height falsifies claim 9. A threshold
  shifted by target or silo terrain, or a burst clamped to the threshold, falsifies claim 8.
  In a separate SW2-h-low.map variant, lower only the falling type's threshold below its
  predicted first-turn world height; expected: it bursts on its first turn despite descending.
  A continued descent falsifies that threshold rule. Do not count bridge-crossing or other
  earlier detonations as evidence for an altitude test.
  (i) SW2-i.map gives the nuclear type an auxiliary-building requirement, leaves the normal
  matching silo and weapons unchanged, and grants the launcher a permanent, non-one-time
  record by trigger. Keep the house at full power and the silo idle. Wait for readiness,
  then fire through the sidebar with no auxiliary building owned. Repeat with the required
  auxiliary building owned, and with no matching silo but the permanent record still ready.
  Record readiness before the order, charge after it, missile count and launch sounds.
  Expected: the absent-auxiliary and present-auxiliary runs both launch from the silo on its
  next update and spend the charge; the no-silo run spends the charge with no missile or
  launch sounds. Refusal or no missile only when the auxiliary is absent falsifies the
  distinction from the giving-building test. A missile in the no-silo control falsifies the
  requirement for a matching silo even with a permanent grant.
- **SW3 — one-time strike (claims 10, 12).** Grant a one-time nuclear strike by trigger; fire at a
  flat cell with a vehicle; repeat at a high bridge. Record height versus tick, impact tick and
  kill credit. Expected: F2's initial height and downward velocity followed by S-0306 claim 11's
  flight and claim 25's wait, with no kill credit. The bridge run adds only F2's bridge lift.
  use imported projectile acceleration and weapon speed to predict each flight tick.
- **SW4 — warning marker (claim 11).** Four houses: launcher, an ally of the launcher with a
  psychic sensor near the target, an enemy with an operating psychic sensor near the target, and
  an enemy whose sensor is switched off. Fire a silo strike and a one-time strike, viewing each
  house in separate runs. Expected: only the operating enemy sensor's house sees the marker.
  For the silo strike it vanishes when the upward missile bursts. Record sensor reference points
  and target centres in integer leptons. Repeat at the inclusive reach boundary of claim 11
  and just outside it, including a height difference. Expected: height counts and sensor
  foundation size does not extend reach. Capture the sensor before a later strike: membership
  follows the new owner under S-0610 claim 9. A marker seen without an operating owned sensor,
  missed at a reachable inclusive boundary, seen outside reach, or retained for the old owner
  solely through the captured sensor falsifies the corresponding condition. A boundary that
  distinguishes the table root from an exact root is not established here.
- **SW6 — storm request and start (claims 14, 15).** (a) Fire the weather superweapon in tick *T*; record
  the activation tick (lighting change, radar outage); expected *T* + *L* (an order is
  applied after the weather update, so the countdown's first decrease is in tick *T* + 1), and
  approach warnings when the remainder is a multiple of 225. (b) Named test file with the
  storm-message flag off; expected: no warnings and no start message; the launch announcement
  still plays. A separate named map requests the storm by trigger before the weather update.
  use countdowns 225, 226 and 227 ticks to discriminate F5's warning boundary, and zero
  countdown on a hit-period tick to check same-tick centre-cloud creation. (c) Four houses: launcher, its ally, two enemies, one enemy defeated before the
  start; expected: outages for the undefeated enemy only, of the storm duration. (d) Our engine
  only: defeat the person-played house, then let a computer house's storm start; expected (design)
  outages as in (c). The original gives none (record it).
- **SW7 — clouds and strikes (claims 16 to 18).** Storm centred on an open area, recorder on.
  (a) Expected: centre clouds exactly in ticks that are multiples of the hit period, whatever the
  start tick. (b) Scatter clouds only in multiples of the scatter period, at most one per period,
  within ±⌊*s*/2⌋ cells, never within the separation of a present cloud. Repeat near the legal
  map edge with named data making separation too large for any candidate: each scatter period
  takes exactly three rejected tries, unless a legal separated candidate succeeds sooner.
  (c) The draws per cloud:
  displacement draws, then one raw value for the animation, then its creation draws. (d) A
  vehicle under a cloud: record the tick its health drops and the tick the cloud's middle frame
  passes; expected: the same weather update as the bolt's creation. (e) Debris: none when an
  infantry unit stands at the centre, present on an empty road cell, absent on an empty clear cell.
  Use two same-tick trigger clouds over distinct targets to test newest-first strike and death
  draws. Named test maps give cloud images one, two and several frames: the first two never
  strike. Repeat a zero-damage strike to test the absence of damage and explosion draws.
  In SW7-e.map use multiple bolt and sound choices, animation types with known creation
  draws and a lethal non-infantry target producing debris. Record the stream, chosen bolt,
  sound, explosion, death effects and each debris choice. Expected: choices and final stream
  state match claim 18's sequence, including each debris animation's draws before the next
  debris choice. Any differing choice, draw count or final state falsifies that sequence.
  In additional named fixtures place infantry closer to the centre but farther from the
  top corner than a vehicle, then reverse their top-corner distances. Kill the vehicle
  without removing the infantry: expected debris depends on the pre-hit top-corner
  witness, not the centre-nearest unit. Set equal truncated distances and reverse listing
  order to test ties, including distances with unequal squared sums but equal truncated
  roots. Repeat with a ground building, a deck vehicle, a ground vehicle under the deck
  and a lone terrain object on road. Expected: claim 18's ground-only witness and empty
  test. Record each debris creation point read-only before its first turn: exactly the
  strike point, including bridge lift but no extra lift. Different witness, tie, empty
  result or creation position falsifies the corresponding subclaim.
  (f) Building witness geometry: on a tiny level skirmish map, speed index 2, director
  and mods off, use waypoint-triggered isolated strikes on each foundation cell of
  stock 1 × 1, 2 × 2, odd-by-even and even-by-odd building types where available.
  Run each cell in a fresh fixture so earlier destruction cannot change the listing.
  Use named test data only where needed to permit infantry on the footprint, make the
  building die while that infantry survives, or supply a missing foundation shape.
  Retain the stock foundation geometry, storm rules and debris list. Keep one bystander
  computer player outside the test area, voiding runs where it interferes or is defeated.
  Use the read-only recorder with scripted input to record positions, building removal,
  infantry survival and storm-debris creation at 800 × 600 over seeds 1 to 16.
  Baseline each building without infantry: its lethal strike should produce storm debris
  on every foundation cell. Add infantry with distance strictly between 0 and 128,
  between 128 and 181, then above 181 leptons, using recorded positions rather than
  assuming input achieves exact points. Expected: infantry suppress storm debris only
  when they win claim 18's distance test, with the same building threshold in every
  foundation cell. In particular the 2 × 2 building wins against every positive-distance
  infantry candidate, whereas a 1 × 1 building loses to a candidate below 181 leptons.
  Include a centre-standing infantry candidate to test its 181-lepton tie against a
  one-cell building. Distinguish storm debris from the building's own death debris by
  recorded animation choices and the storm's extra count/choice draws. Different
  thresholds, a threshold changing with the struck foundation cell, or the absence of
  the building witness from a non-anchor foundation cell falsifies the respective rule.
- **SW8 — end, credit, height (claims 17, 19, 20).** (a) Record the tick the lighting returns; expected:
  the first weather update after clearing begins that finds no present cloud at its start,
  specifically one update after removal of the last present cloud. Use a named map with a storm
  duration that ends before any cloud is created to check the empty case: idle next update.
  (b) Trigger a storm-stop
  while clouds are pending; expected: they still strike. During a countdown requested by a trigger
  of another house, let a trigger-made cloud strike a unit; expected: the requesting house is
  credited. Make a second request during that countdown with a different house, centre,
  duration and longer countdown: expected replacement centre/owner/duration, unchanged shorter
  remainder. Repeat with a shorter countdown. A stop during countdown does not cancel it.
  Defeat the owner while raging, keeping two other contenders: strikes retain its credit.
  Create an idle-storm cloud with the lightning-strike-at-a-waypoint action before any storm
  request: expected a bolt and damage
  with no house credited. In separate named maps, create that cloud under trigger owner A, then
  request a deferred storm for B before it strikes: expected B's credit, not A's. Change the
  pending request to C before the same cloud matures: expected C's credit. Arrange a lethal target
  under each test cloud using named target-strength overrides so kill counts distinguish owners.
  While clearing, add a waypoint-strike cloud whose last frame is later than all existing clouds:
  expected idle exactly one weather update after its removal, not after the older clouds end.
  (c) Named test file with a one-entry bolt list of known image height *H*; measure the
  cloud's height above its cell; expected F4. Repeat on flat ground, raised ground and a high
  bridge; then use a two-entry bolt list with unequal image heights. Expected: all cloud heights
  use the first bolt image even when the later bolt uses the second entry. Swapping the list
  entries changes the height according to F4. (d) Storm-start action with number *N*; expected: a
  storm of duration *N* at a cell chosen by the recorded draws, crediting nobody.
- **SW9 — curtain area and kinds (claims 22 to 25).** (a) A 5 × 5 square of infantry of two houses
  centred on the target, including a cyborg; expected: the central nine die, credited to the
  launcher; the ring lives. (b) A vehicle on a bridge deck and one under it in a block cell; expected:
  only the deck vehicle is protected (quirk on). Repeat using the waypoint curtain action:
  only the unit underneath is protected and infantry kills credit nobody. Put a vehicle in
  warp-pending state before applying curtain later in the same order tick: it is skipped, as
  is a vehicle already in its leaving stage. Add a loaded transport and a garrisoned building:
  passengers and occupants survive while the transport and building are protected. Place a
  terrain object and fly an aircraft over an otherwise empty block cell: neither gains
  protection. Any affected contained unit, terrain object or flying aircraft falsifies the
  recipient exclusions. (c) Named test file giving a vehicle the
  organic flag and an armour the demolition warhead does not fully affect; expected: it survives
  damaged, unprotected, nobody credited. (d) A tank with a drone parasite, a boat with a naval
  parasite; expected: the drone vanishes, the naval parasite is released and the boat's paralysis
  ends. (e) A building in a delayed-kill countdown; expected: no expiry hit. (f) Every player hears
  the activation; a radar event at the target. (g) Put one lethal curtain or warp victim in
  each block cell, using distinguishable death animations with known scenario-stream draws.
  Record the stream and deaths; expected: the draws identify victims in F1 order. Repeat with
  several infantry in one cell, recording entry order to predict S-0209 claim 13's listing
  order; death draws must follow that order, not unit kind or creation order. Reversed cell or
  within-cell draw order falsifies the ordering claim. As a separate exploratory run, place
  several infantry and a building in one cell. A named map also tries an organic vehicle and
  a colocated building, repeating with a teleport launch and a landing conflict. Record whether
  any victim leaves the cell immediately during its effect and whether later recipients remain
  unaffected. Only a run establishing immediate removal can test the removal quirk: later
  recipients still being affected would then falsify it. A later death or no immediate removal
  is inconclusive, not confirmation.
- **SW10 — protection (claims 25 to 29).** (a) Named test file with a curtain duration of 300; hit a
  protected vehicle in tick *t*₀ + 299 and *t*₀ + 300; expected: no damage, then damage.
  (b) Force-shield a building, then curtain it with a shorter duration; expected: it becomes
  vulnerable when the curtain ends, and the damage flash changes kind. (c) Try temporary and permanent mind control, psychic domination, a chrono weapon, a
  locomotor-seizing weapon, a flinging weapon, a demolition charge, parasite fire, enemy crushing
  and a teleport warp on non-organic protected objects; expected: all refused or skipped.
  Pulse and already-running erasure cases remain unmeasured, as claim 27 says. (d) A computer house's unit and a human house's unit next to a protected enemy; expected
  (original): the computer unit does not fire, the human unit fires without effect.
  (e) Recorder: one draw R(−5, 5) per protected object on entering Initial dimming, at *q* + 10
  for uninterrupted turns, hence 11 ticks after a normal post-turn launch. Record the first
  protected turn separately from the launch. Repeat with protection shorter than that and
  with reapplication before the phase: no draw for an interrupted start. Record each protected
  object's draw separately, not a combined count contaminated by invocation animations.
  Repeat for a curtain-protected building and a force-shielded building: the same *q* + 10
  rule. Load a protected unit after its first protected turn but before its Initial rise
  expires, and separately during Bright hold, and unload while protection remains.
  Expected: the next protected turn advances only one expired phase; returning during
  expired Initial rise starts a new 4-tick hold, whereas returning during expired Bright
  hold draws at once. Any draw taken while absent, skipped intervening phase or a
  universal application-plus-10 schedule falsifies claim 29. (f) Load a protected vehicle into a
  transport, wait beyond expiry, and unload under fire: protection has expired. Capture a
  protected vehicle in a named map through an ownership-change trigger: timer expiry is unchanged.
  Named maps with zero and negative durations confer no protection but retain infantry kills.
  (g) Protection appearance: use a stationary voxel vehicle and a stationary ordinary sprite
  building on flat, neutrally lit ground, no combat, damage flashes or air-strike marking.
  Capture their same visible pixels on every tick from application through expiry, with a
  read-only record of ordinary brightness, protection remainder, first protected turn and
  scenario-stream draws. Curtain both, then force-shield the building in a separate run.
  Expected: claim 29's phase boundaries and F9's brightness, with the first draw determining
  only Initial dimming's length; low-hold entry tests the strict 54-tick boundary and final
  rise the strict 31-tick boundary. The bright hold doubles brightness subject to the cap,
  low hold uses factor 51, not zero. Reapplication resets to the initial envelope; expiry
  removes it immediately, even in an unfinished phase. In SW10-g.map use invented durations
  of 170 and 230 ticks to obtain different pulse counts; add 7- and 12-tick applications to
  distinguish early expiry and a truncated dimming phase. Repeat for ten recorded seeds.
  A wrong phase length, threshold, brightness or draw boundary falsifies that part.
  In SW10-g-colour.map change only the selected adjustment triple to invented (7, 19, 3),
  then (21, 2, 11), in two fresh-session runs; preserve all table positions. Repeat by changing
  the selector instead to another existing valid position. Expected: curtain vehicle and
  force-shield building colour responds to the selected triple; curtain sprite building
  brightness still follows the envelope but its colour does not respond to either selector.
  A response in that last control, or no response in the first two, falsifies the respective
  row. Compare frames with equal phase factor and brightness; this selection test alone does
  not establish the exact per-pixel composition formula. S-0102 claim 34 and F9 supply
  that contract; its D44 extends these captures to the other ordinary opaque drawing cases
  and tests the single-layout drawing correction with engine fixtures.
- **SW11 — force shield (claims 30 to 32).** (a) Allied buildings with reference points at
  distances just below and at 256 × *R* from the target centre, one on raised ground; an enemy
  building inside; expected: only allied buildings strictly inside F3 protected. (b) Vehicles
  inside the radius: unprotected. (c) The launcher's blackout lasts the blackout length (S-0405
  scenario W4 method); record that the house is blacked out already when protection begins.
  (d) Battle-lab grant: see SW16. (e) Start sound at once, fading sound *D* − *P* house updates
  later. Named tests set that difference to zero (next update), negative (no fade sound), and
  positive with a second launch before expiry (only the replacement scheduled sound).
  Repeat (a) with one-way alliances: the building owner's attitude to the launcher decides.
- **SW12 — warp (claims 33 to 36, 39).** (a) Pick a source; only the launching player sees the
  marker; leaving warp aiming ends it. Pick an empty source, cancel, then pick a new source:
  no unit requirement, and the marker moves to the new source. Add S-0512 claim 10's X17
  no-pick control: fire an accessible second-stage grant before picking any source, record
  the source coordinate and the linked first-stage clock. Expect cell (0, 0), an attempted
  ordinary warp there, and restart of a granted linked first stage; no restart when that
  first stage is ungranted. A substituted building/source or no restart of the granted
  record falsifies the contract. (b) A 3 × 3 block with own tanks, an enemy tank, own
  infantry and a teleporter-flag infantry; expected: tanks move (the enemy too), ordinary infantry
  die credited to the launcher, the teleporter infantry moves. (c) Expected: the block's shape is
  kept; infantry keep their position within the cell; vehicles end at cell centres. Repeat
  with an aircraft landed on open ground, recording its within-cell position before and after:
  expected unchanged position within the corresponding destination cell. A changed relative
  cell, rotated block, retained off-centre vehicle position or lost infantry/aircraft position
  within the cell falsifies the corresponding part of claim 36. (d) Use a
  stationary, newly created tank with no previous warp or reinforcement hold, an empty flat
  destination and no deaths or removals that could cost it a turn. Record the applied order
  tick *t*_L, old-position presence, disappearance, placement, arrival effects and phasing-end
  ticks, and the being-erased and phasing-in states read-only at each boundary. Expected:
  being erased from its update in *t*_L + 1 through the expiry update in *t*_L + 61, then
  phasing in at departure. Still at its old position in *t*_L + 61, disappearance in *t*_L + 62,
  placement in *t*_L + 63, arrival effects in *t*_L + 64 and phasing end in *t*_L + 65.
  A one-tick discrepancy falsifies the corresponding boundary, rather than being silently
  accepted. Record scenario draws at both targetless searches against S-0207 claim 20 (f):
  the first search attempt is in *t*_L + 61, one tick before departure-animation draws.
  The second is in *t*_L + 65. For a draw witness, use a named variant with a long targeting
  period and a nearby visible enemy whose weapon cannot kill the member during the run.
  Retain only runs whose read-only record shows no target and more than 10 ticks left on
  the targeting timer just before expiry. Expected: the conditional 4-to-8-tick draw at
  expiry, followed by any ordinary search draws under S-0207 claim 20 (f); a draw delayed
  until departure falsifies the expiry/search boundary. Other runs test positions and effects
  but are inconclusive for this conditional draw.
  Repeat a deck-to-deck warp, recording both the initial landing point and final placed height
  to test claim 36's low-confidence lift. Infect a member with a naval parasite: expected release
  at launch with the suppression period of claim 34. In a named map variant give a naval host
  the organic flag and no teleporter flag, leaving adjacent water clear for parasite exit.
  Infect it with a naval parasite before the launch. Expected: the parasite exits beside the
  old position and the host dies with the launcher's credit. A retained parasite or a parasite
  removed by its host's death instead of released falsifies the release precedence.
  A drone-parasite run records departure
  outcome without assuming the ordinary-teleport release path. (e) A vehicle inside
  its factory and an aircraft in flight over the block; expected: both stay. A named map makes
  the factory vehicle organic and non-teleporting: it is killed, not exempted. Include a
  protected non-organic unit, a unit already warping, a bunkered vehicle, a controlled vehicle
  and a loaded transport: the first two stay, the bunker link ends in place with S-0606
  claim 14's walls-down effects and no release track before the warp, control and passengers
  persist. Record the bunker's wall animations, sound, occupancy, contact and queued behaviour:
  expected cleared link and contact, *clearing* entry stage and *guard* queued, without an
  immediate start. A retained link or contact, missing walls-down effects or a release track
  before the warp falsifies the bunker-release claim.
  Repeat the pending unit test during its leaving stage: it stays on its original warp timeline.
  Warp a vehicle undergoing repair: its repair link ends and repair does not continue remotely
  after departure. Continued bunker confinement or remote repair falsifies the contact-ending
  claim. (f) A unit under a
  bridge whose deck is the source cell; expected: it stays. (g) Every player hears the
  activation; two radar events. (h) Named rules file with a reordered superweapon type list: keep
  the source type selectable, put a non-warp type with a distinct aim action at index 4, and
  leave the warp type at a different index. Pick a source through the sidebar and record the
  next cursor action and resulting effect after a click. Expected: aiming and firing the type
  at index 4, not the relocated warp type. A warp cursor/effect instead falsifies the fixed-index
  rule. Restore the list after this part; do not conflate this aiming test with S-0512 claim 11's
  link lookup.
- **SW13 — landing conflicts (claims 35, 37, 38, 40, 41).** (a) A protected unit at a landing cell; expected:
  the arriving unit dies. A plain unit at a landing cell; expected: it dies, nobody
  credited. A protected infantry unit in the source block; expected: it dies. A named map warps
  teleporter infantry into a cell containing infantry at equal and unequal standing points:
  only the equal-point occupant dies. Repeat over a deck with a protected unit on the ground
  underneath: arrival effects kill the arriving member even when the deck check passed.
  (b) A building on a
  landing cell; expected: the unit lands in a nearby cell by the stated search. Include a
  blocked bridge entry and a terrain object, and arrange candidates legal only to the unit's
  own locomotion, not to tracks: expected redirect follows the tracked search, with no draw.
  (c) Warp the
  redirected unit of (b) again to a clean cell; expected: it phases in for the teleport delay,
  where a never-redirected unit has held delay 0 and stops phasing in on the update after
  arrival effects. Record the arrival-effects tick *t*_A and the phasing-end tick in both
  cases: expected *t*_A + max(1, *H*) from claim 40. Repeat with named data giving redirected
  holds of 0, 1 and 2 ticks: expected ends at *t*_A + 1, *t*_A + 1 and *t*_A + 2,
  respectively; same-update expiry or any other tick difference falsifies this boundary.
  (d) A tank warped onto deep water sinks and
  the launcher's kill count rises; a hover vehicle survives; a powered-unit hover vehicle of a
  house with no active control centre sinks. Contrast full house power without a centre against
  low house power with a centre that does not need power, to distinguish the count from house
  power. Use named test data for those centre settings. (e) A ship warped onto a land cell dies
  with no credit. Repeat onto a bridge deck above water with an otherwise empty landing cell:
  the ship sinks with no credit, unlike the tank's credited sinking in (d). A launcher kill-count
  increase for the ship falsifies the no-credit sinking rule.
- **SW14 — paradrop (claim 42).** Named test file without a paradrop plane type; fire; expected:
  nothing sent, charge spent, no announcement for anyone.
- **SW15 — spy plane (claims 43 to 45).** (a) Stock: one plane per first-side-group paradrop entry
  as imported. Named test file with two entries: two planes; unequal lists: none,
  charge spent. Recorder: the arrival-edge draws of S-0522 claim 4. (b) Record the shroud around
  the plane; expected: S-0409 claim 6's footprint with radius equal to its weapon's damage
  (capped at 10), appearing once
  the plane is within its weapon's range of the target, refreshed every 3 ticks in overfly.
  (c) Camera sound in approach only. (d) A cloaked unit in the disc stays hidden; shoot down a plane
  on approach: nothing further revealed. Include a disguised object: the camera adds no
  disguise detection. Named maps set weapon damage to zero (ordinary sight), to a radius
  distinct from ordinary sight, and to a value above the cap; record the exact explored cells
  of S-0409 claim 6, not an assumed circular disc. An ordinary moving-sight refresh between
  cameras can restore the ordinary radius.
- **SW16 — battle-lab force shield (S-0512 claim 3).** Stock skirmish, superweapons on, build a
  battle lab of each side; record whether a force-shield icon appears and charges. Expected:
  granting follows S-0512 claim 3 when the imported building data gives that superweapon.
  This is an availability test for S-0512, not an additional effect or scope gate here.
- **SW17 — original computer defence draw (claim 6).** A named map places two computer-player
  bases within the imported defence distance and one outside. Fire a defence-marked nuclear
  type at empty ground, recording the scenario stream and house creation numbers. Expected:
  one R(0, 99) per qualifying computer house, descending house number, after the order's
  effect draws; none for the distant house. Repeat with the defence flag off: no such draw.
  Any extra draw, omitted qualifying draw or reversed house order falsifies the original's
  draw contract; our engine's game rules deliberately omit all these draws. Also fire a second
  ready weather type while a storm is already raging: the order is refused and keeps its
  charge, but still takes the qualifying defence draws in the original.
- **SW18 — local launch presentation (claim 5).** Grant the launching house curtain, weather,
  teleport-source, teleport-warp, paradrop and spy-plane records on separate named maps.
  Use a second undefeated enemy and an ally so defeat of the viewing player need not end the
  test. Record video and audio of the same scripted launches, repeating each run for each
  viewing house. Expected: source marker and its sound only for the launcher, no launch
  announcement for paradrop or spy plane, aiming ends for the launcher after a completed
  non-source launch and source picking instead enters second-stage aiming. Fire while the
  corresponding ready announcement is queued or playing: the launching player's instance is
  cancelled by the non-source launch. Repeat an enemy launch after the viewing player's
  defeat, keeping the other two houses opposed: its launch announcement is muted for that
  viewer. A marker heard or seen by another house, an unexpected announcement, persistent
  non-source aiming, an uncancelled ready announcement, or an unmuted post-defeat launch
  announcement falsifies the corresponding part of claim 5. Sounds still follow S-0701's
  ordinary priority and cooldown rules; compare only runs where those rules admit playback.

- **SW19 — waypoint curtain (claim 22).** A named map fires action 109 at a waypoint with
  ground infantry and a vehicle, repeated beneath a high bridge with a deck vehicle. Use
  arbitrary unused number-slot values in separate otherwise identical runs. Record health,
  protection expiry, invocation height, sound and radar events. Expected: ground infantry
  die without house credit, ground vehicle gains the imported curtain duration, deck
  vehicle is unaffected, invocation at surface plus 5 leptons without bridge lift, no
  activation announcement or radar event. Changing the unused values changes nothing.
- **SW20 — cloud clock (claim 46).** Named cloud-image fixtures of 3, 5 and 8 frames,
  ordinary forward one-pass playback, effective delays 1, 2 and 4 ticks, no other clouds.
  Create by a waypoint strike before the turn pass, then by weather, recording creation,
  first update, each stage, bolt creation and present-cloud membership read-only. Expected:
  the two threshold ticks of claim 46; animation deletion is separately recorded. Repeat
  with delay 0: stage remains 0 and no strike for these images. A named theater replacement
  with a different frame count tests the current-image threshold rather than stored μ.
  A different stage advance or weather tick falsifies the corresponding formula. Observe
  naturally recorded seeds for a random-rate variant and use its drawn effective delay.
- **SW21 — silo busy boundary (claim 47).** Named map with two ready, non-one-time nuclear
  types on one silo's two superweapon fields. Fire at different targets with gaps of 0,
  1, 2, 7, 8 and 9 ticks in separate runs. No other behaviour orders. Record current and
  queued behaviour, processing ticks, missiles and their targets. Expected: no second
  sequence for gaps below 8, a second sequence and next-turn missile for gaps of 8 or 9.
  Gap 0 replaces the first missile's target; replacements after its creation do not.
  Repeat with altered launch-picture lengths and rates only: the boundary is unchanged.
  Any queued extra missile, length-dependent boundary or earlier acceptance falsifies it.
- **SW22 — source-marker draws (claim 33).** In skirmish, record the stream at source
  picks by the local house and by another house's scripted normal play in separate runs,
  accounting for all other draws. A named marker type with a distinctive random-rate
  draw isolates its creation. Expected: its creation draws in both cases, but visibility
  and sound only for its launching viewer. No network-game test is required or claimed.
- **SW23 — warp turn continuation (claim 48).** Extend SW12(d) with a damaged self-healing
  vehicle already driving, a turret aimed away from its body, and a harmless enemy target.
  also test teleporter infantry and landed aircraft separately. Named map SW23 supplies
  healing every tick, a weapon able to fire at the target without killing it, and a long
  redirected hold. Use the common skirmish settings and three runs per kind and part.
  Record health, position, current/queued destinations, route, target, behaviour, shot
  count, turret facing and warp stages with tick labels. Use video and the permitted
  read-only recorder, never injected tracing. Behaviour processing needs a visible
  due-action witness, not merely unchanged behaviour text. Parts:
  (a) Record launch and the first pending turn separately. Expected: no continuation of
  the old driving movement. For immediate route clearing and each first-turn ordinary
  operation, the expected value is unmeasured: retained versus cleared routes, a heal
  versus no heal, and a shot/turret change versus none distinguish the alternatives.
  (b) Record turns still leaving, including wait expiry. Expect no ordinary heal, due
  behaviour action, physical movement, shot or turret progress; any such event falsifies
  that exclusion. (c) Record departure and every off-map retry separately. Whether each
  ordinary operation continues is unmeasured; any heal, due action, shot or turret
  progress refutes the whole-turn-only hypothesis for that boundary, while its absence
  across an independently eligible due event refutes continuation for that operation.
  (d) Apply the same per-operation test separately to placement and arrival-effects turns.
  Their continuation values are unmeasured, not assumed equal to off-map retries.
  (e) Record the arrival hold and expiry, including held delays 0, 1 and a longer value.
  Expect the hold-end decision before any ordinary-work witness in a turn that begins
  holding; a resolvable reversed order falsifies that ordering. Eligibility/continuation
  of each operation on holding and expiry turns is unmeasured; compare due-action
  witnesses as in (c). Record stage transitions per tick to discriminate one versus
  multiple warp updates; exact per-kind absolute timelines remain unmeasured. An
  observation that cannot separate same-tick events is inconclusive, not confirmation.
- **SW24 — exceptional warp cells (claim 49).** Use the common skirmish settings and
  three runs of each part. (a) Source block in the interior and destination centre on a
  legal map edge: place a normal vehicle, organic infantry and an infected naval host
  in source cells mapped outside the map. Keep exit water clear. Record parasite exit,
  deaths and credit, pending flags, landing points and all later positions using video
  and the permitted read-only recorder. Expect ordinary source-side parasite release,
  organic death credited to the launcher, and an otherwise eligible survivor joining the
  warp rather than an early whole-cell refusal. Any contrary outcome falsifies that
  confirmed portion. The survivor's final position and fate are unmeasured: distinguish
  staying at source, a different landing, removal and death, without selecting one as
  the expected result. (b) Separately block every candidate through the nearby search's
  full radius, verified read-only under S-0212 claim 25, and record each failed result,
  replaced landing point, retry stage, held delay, subsequent positions and fate. Expect
  the failed result to replace the point, the unsuccessful check to remain in retry,
  and a retry to set the held delay to the imported teleport delay. Preserving the old
  point despite a distinct failed-result conversion, or not setting the delay during
  retry, falsifies those rules. Exact conversion and eventual placement/death/survival
  remain unmeasured; this part distinguishes those alternatives and does not confirm
  retained-point retries merely because no landing is visible.
- **SW25 — bolt header height (F4).** Extend SW8(c) with two images having equal stored
  rectangles but unequal canvas heights, then equal canvas heights but unequal rectangles.
  Expected: cloud world height follows canvas height alone. Use a named damaging cloud
  animation with a documented ordinary-animation warhead and targets at distinct heights:
  its own deliveries originate at the cloud height while the lightning hit stays at
  claim 18's strike point. Record world positions and health per advance. A cloud-height
  change following rectangle size, or lightning inheriting cloud altitude, falsifies F4.
  Missing-image fallback is tested only in our engine: warning, no cloud, no invented
  zero-height header. The original's unsafe access is not a stable oracle expectation.

Genetic mutation's shared swept-layer rule and absence of a list cut are tested by S-0611
claim 4's scenario MU4. The added launch-feedback rows retain their owning specs' scenarios:
S-0611 claim 2 (MU1 part a and MU11) for genetic mutation, and S-0612 claim 1 (PD1) for
psychic domination. Those scenarios, rather than SW18, own the target sounds and radar events
for these two kinds.

## Open questions

- Whether an electromagnetic-pulse-disabled silo still launches (claim 7).
- Jumpjets in the air and falling paratroopers under the curtain: S-0209 claim 13 settles listing
  for ordinary flight only (claim 4).
- The deck-to-deck bridge lift of claim 36 remains low confidence; SW12 needs a deck-to-deck
  run recording both the initial landing point and actual placed height.
- The absolute tick of each warp step (claim 39).
- Q-0742 question 4: claim 29 supplies the protection envelope and its draw; S-0102
  claim 34 and F9 settle ordinary opaque composition and the sprite-vehicle, aircraft,
  building-voxel and marked-building-animation cases. Exact mask handling for translucent
  animations, cloaked or translucent units, building bibs and door accessories, remembered
  images and unusual sprite-turret accessory combinations remains unconfirmed there.
  SW10(g)'s same-pixel captures extended to those variants, with named art overrides where
  necessary, remain proposed discriminating observations. Do not infer their colour
  treatment from the opaque cases or from identical brightness timing.
- Q-0742 questions 1 to 3 are answered by S-0214 claim 31, table CS; question 5 belongs to
  S-0512 claim 16. They are not answered by the protection changes here.
- Drone-parasite release by the superweapon warp remains untraced (claim 39).
- Negative storm countdowns and durations other than the endless value are not covered by the
  non-negative timing formulas; their safe import policy remains unresolved.
- Q-0738 is accounted for question by question in claim 48: leaving exclusions, stopped
  driving and hold-test ordering are confirmed source-derived rules, not measurements.
  Each pending/departure/off-map/placement/arrival/hold ordinary-work continuation and
  immediate route-clearing gap is explicitly unmeasured with SW23(a) to (e). These gaps
  remain open to investigation despite the request's inclusion in front matter. Reconcile
  claim 39/T2's one-update assumption before using unconditional per-kind tick labels.
- Q-0741 is accounted for in claim 49: marker draws were already answered by claim 33.
  source-side effects and unchecked failed-search replacement have rules. The off-map
  survivor's fate and failed-search world-point conversion/fate remain explicitly
  unmeasured under SW24(a) and (b). Front-matter inclusion records this complete
  question accounting, not oracle confirmation or closure of those factual gaps.
- Q-0732: rate/frame-count timing is claim 46, using S-0109 claims 2 to 5 and 25; Q-0733
  was already answered by claim 29's first-protected-turn rule, now explicit about skipped
  phases. Q-0734 is claim 22 and Import mapping, Q-0735 is claim 47, Q-0739 is F4,
  Q-0740 is claim 18. No oracle measurements are claimed.

