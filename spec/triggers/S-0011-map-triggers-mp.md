---
id: S-0011
title: Map triggers (multiplayer subset)
topic: triggers
revision: 29
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-07
measurements: []
requests: [Q-0269, Q-0510, Q-2716]
supersedes: []
---

# S-0011: Map triggers (multiplayer subset)

## Summary

How the triggers that a map defines are set up while a game loads, when they are evaluated and fire,
and what the actions of this revision's subset do (the event and action codes of Import mapping,
tables I3 and I4), including the order in which the scenario random stream is drawn while a map loads.
This revision covers skirmish only; network games and campaigns are out of scope. Codes outside
the subset are parsed; their meaning is deferred except for the companion-spec pointers after
tables I3 and I4, which do not extend this spec's supported subset.

**Hand-off checks.** S-0003 claim 14's exceptions, S-0202's loading-draw table,
S-0302's survivor rules and S-0409's all-player-house scope agree in their current texts.
At sign-off, hand claim 19 and T7 off with S-0312 claim 29/D20: normal repeating
vehicle deaths give seven firings with an attacker and four without, common-only deaths
give two, and latch-off gives two when the extra vehicle group remains enabled.
Check claim 35 against S-0414 claim 18/V15 for within-update timing and continued raises
after defeat. Check claim 36 and scenario T21 against S-0109 claim 13, and hand the two
revisions off together: creation-time sound and resource chain reaction are enabled,
and only later start events suppress them. S-0401 claim 20 must limit its trigger-played
suppression to later start events in the same hand-off. Check claim 36's map-load composition
and during-play composition against S-0202 claims 10 and 15 and scenario R14, together with
S-0109 claims 14 and 25. Subsequent particle updates remain outside this hand-off.
Confirm S-0202 claim 15's crater-removal wording with S-0109 claim 14 and S-0401 claim 3:
the request takes 6 bales, so a stage-6 cell becomes stage 0 rather than empty.
Confirm the S-0201 pointers after I4, including its win-action fixture assumption.
Those pointers add no campaign rules. Check claim 18's arrival-timing pointers with S-0207 claim 22/D24 for driving and
own-locomotor teleporting, and S-0208 claim 22/W14 for ordinary walking.
Check the other companion revisions together too. Claim 19 now admits S-0312 claims 30 and 31's
hidden-slave notifications without a slave killing hit, including true sale (S-0408 claim 10a)
and temporary-group cleanup during successful pack-up (S-0415 claim 10). Hand these passages
off together; hidden-slave tag attachment remains an unmeasured setup boundary.
Check claims 20 and 27 with S-0511 claims 1, 7 and 8 at joint sign-off:
bridge-event bands can include intact cells and give repeated raises with no cell,
whereas destroy-attached-objects makes ungated attempts only when its raise carries a cell.

## Terms

Terms used as defined in the glossary: *trigger*, *trigger action*, *cell tag*, *tick*, *house*,
*country*, *human-controlled*, *local player*, *game start*, *game minute*, *type*, *type name*,
*registry*, *registration index*, *weapon*, *warhead*, *cell*, *lepton*, *height level*,
*scenario random stream* (with *draw* and *ranged draw*), *quirk switch*, *departure*, *importer
warning*, *unspecified*, *measurement setup*, *recorder*, *marked (particle system)*. *Map trigger variable* is used as defined
in S-0009 claim 32. *House number* is used as defined in S-0009 (Terms): the house's place in the creation order, counting from 0. *Observer* is used as defined in S-0009 (Terms): a human participant whom the
lobby sets to watch instead of play. Network games, and so how machines synchronise, are outside
this spec. A claim whose result in the original depends on which machine it runs on says what our
engine does instead (claims 26 and 36). The terms below are specific to this spec.

- **Network game.** A multiplayer game played across more than one machine.
- **Trigger definition.** One entry of the map's trigger list. It has an owner country, an optional
  linked trigger (another trigger definition), a name, a disabled flag, three difficulty flags (one
  for each difficulty position), a carry-over flag, an event list and an action list (Import
  mapping, I1).
- **Event.** One condition of a trigger definition: an event code, a parameter kind, a number value
  and, for one kind, a type name.
- **Tag definition.** One entry of the map's tag list: a repeat type, a name and a first trigger.
- **Chain.** The trigger definitions of a tag definition: its first trigger, then the linked trigger
  of each member in turn, until a member has no linked trigger.
- **Tag.** The single runtime object of one tag definition. It is shared by everything that names
  the tag definition. It holds one trigger instance for each member of the chain.
- **Trigger instance.** The runtime state of one chain member inside one tag: an enabled flag, a
  removed flag, a trigger timer, a latch set and a triggering house.
- **Carrier.** A cell or an object that has a tag attached: a cell through the map's cell tags
  (S-0009 claim 31), a pre-placed object through its tag field. A tag counts its carriers.
- **Present object.** An object that is on the map and has not been destroyed. It excludes an
  object inside a transport, a building or a factory (or otherwise taken off the map), and an
  object whose destruction has begun (for example one still showing its death) but which has not
  yet been removed. Claims 24 and 27 act on present objects only.
- **Raise.** An occasion on which the original evaluates a tag: it is raised with an event code,
  and optionally with a causing object and a cell. Claims 6, 18 to 20 and 35 say when tags are
  raised.
- **Repeat type.** The tag definition's integer 0 (*one-time*), 1 (*all-carriers*) or 2
  (*repeating*). Other values are possible in the data (claim 10).
- **Code-matched event.** An event whose code belongs to the code-matched set of Constants. It is
  satisfied only in a raise with its own code.
- **Condition event.** Any other event. It is judged against the current state in every raise of
  its tag, whatever the raise's code.
- **Latch.** A mark on one event of one trigger instance that makes it count as satisfied in every
  later evaluation (claim 9).
- **Event position.** The place of an event in its trigger's event list, counted from 1 in file
  order (Import mapping).
- **Polled tag.** A tag whose chain contains at least one event with a code of the polled set
  (Constants). Polled tags are raised once per tick (claim 6).
- **Trigger time unit.** 15 ticks, that is 1/60 of a game minute. Trigger timers, the mission timer
  and the game-time event count in this unit.
- **Trigger timer.** One countdown per trigger instance, with an arming tick *s* and a length *D* in
  ticks (claims 13 and 14).
- **Mission timer.** The single countdown of a game that the trigger actions of claim 31 start,
  stop and change, and that the game shows on screen with a text label.
- **Owner house.** Of a trigger instance: the first house, in the house order of S-0009 claim 19,
  whose country is the trigger definition's owner country. None if no house plays that country.
- **Triggering house.** Of a trigger instance: the owner of the causing object of the last
  entered-by event judged true in an evaluation of the instance (claim 18). None until then.
- **Number slot, second number slot, waypoint slot.** The values that an action's parameters give
  its code. The number slot is an integer, and I2 says which kinds fill it and how (as an integer,
  or as the result of looking up a name). The second number slot is an integer that only kind 11
  fills. The waypoint slot is the waypoint number that Formulas F5 decodes from the letter text of
  the action's eighth item (the *waypoint parameter*). An action's code reads only the slots that
  table I4 lists for it.
- **Difficulty position.** 0, 1 or 2. It selects which of a trigger definition's three difficulty
  flags must be set for the trigger to be enabled (Import mapping, I1).
- **House value.** The integer that an action's parameter uses to name a house (claim 23).
- **Reference.** An action parameter that names a team definition, a trigger definition or a tag
  definition by its ID (or by its registration index) (Import mapping, I2).
- **Team definition.** A map-defined group that the original's team system builds and runs.
  The team system is outside this spec (a pointer only).
- **End of loading.** The point in loading a game after the starting units are placed (S-0009, its
  table "Start position resolution") and after starting crates and secret-lab choices, but before the game-start terrain
  recalculation pass of S-0010 claim 12 and before the first tick. At this point claim 34 creates
  the remaining tags, claim 6 decides which tags are polled and claim 35 assigns tags to houses.
  The name denotes this tag set-up point, not completion of every row in *Load-time draw order*
  (State and ordering). It is not the glossary's *game start*, which is the moment loading begins.
  The whole of loading, the game-start pass included, happens while the tick counter is 0.
- **Elapsed-time event.** An event with the elapsed-time code (claim 13); with the random-delay
  event it is one of the two timer events.
- **Trigger-played animation.** An animation that claim 36 creates, using S-0109's vocabulary.
  Creation-time start effects and later playback are governed by S-0109 claims 13 and 14.

## Behaviour

### Definitions, tags and instances

1. **Only chained triggers run.** A trigger definition has an effect only as a member of the chain
   of some tag definition. A tag holds its own instance of every member of its chain, so a trigger
   definition that is a member of two chains has two independent instances, and one that is a
   member of no chain never fires. In the original, a map with a chain that comes back to a member
   already in it never finishes loading (Edge cases,
   *Unparseable entries*). Our engine ends the chain before the first repeated member, and the
   importer warns (a departure).
   - Backing: `measurable` (scenario T1 for chain membership and independent instances; T24 D
     for the loading hang on a cycle). The safe chain cut is our engine's departure
   - Confidence: high (medium for the shared member)

2. **One tag per tag definition.** Every carrier of a tag definition carries the same tag, so all of
   them share one set of trigger instances, one trigger timer per instance and one latch set per
   instance. The tag is created the first time a cell tag, a pre-placed object or the end of loading
   needs it (claim 34). The tag counts its carriers: attaching it to a cell or an object adds one,
   and detaching it removes one. Being destroyed, leaving play and being removed (glossary) do not
   by themselves detach an object's tag. A removed object therefore still carries its tag while it
   is a leftover (S-0203 claim 13), for the rest of the tick of its removal, and for the rest of the
   game when it is kept to the game's end (S-0203 claim 13 (g)). Nothing detaches it when it stops
   existing either, so a carrier that stops existing while it carries the tag keeps its place in the
   tag's carrier count for the rest of the game (`quirks.removed_carrier_keeps_tag`). A cell tag
   names its tag definition by the definition's ID, without regard to letter case (Import mapping).
   A pre-placed object's tag field matches either the ID or the name field of a tag definition,
   without regard to letter case: the object gets the first tag definition in registration order
   (claim 7) whose ID or name equals the field. A tag name longer than 48 bytes is cut to its first
   48 bytes, so only that cut name matches. A tag field that matches no tag definition gives the
   object no tag and creates no definition. The `none` marker of an untagged object normally matches
   nothing in this way; the importer warns about any other unmatched field.
   - Backing: `measurable` (scenario T6, variants E and F, and variant H, whose carriers are
     vehicles, for a carrier that stops existing keeping its place in the count). `unmeasured` for a
     destroyed carrier of another kind (an infantry unit, an aircraft or a building) keeping its tag
     and its place in the count: no scenario here destroys a carrier of another kind under an
     all-carriers tag. `unmeasured` for a leftover still carrying its tag: the only rule known to
     reach a leftover is S-0203's defeat sweep, whose raises S-0203 claim 13 (g) gives as
     `unmeasured`, so no scenario here can show the tag on a leftover. `unmeasured` for an object
     that leaves play or is removed without being destroyed: no scenario here takes a carrier out of
     play in another way, and which of those ways raise its tag is not established here
   - Confidence: high (medium for the name-field match and the cut, for a leftover still carrying
     its tag, for an object taken out of play without being destroyed and for a destroyed carrier
     other than a vehicle keeping its tag and its place in the count)

3. **Enabled at start.** A trigger instance starts enabled exactly when its definition's disabled
   flag is clear and its difficulty flag at the game's difficulty position is set (claim 4 gives
   the case of a setting outside 0 … 2). Otherwise it starts disabled. A disabled instance is never satisfied and never fires its actions. Only the
   enable action (claim 25) enables it.
   - Backing: `measurable` (scenario T2)
   - Confidence: high

4. **Difficulty position of a skirmish.** In a skirmish the
   difficulty position is the session's computer-difficulty setting. The program initialises it
   from the base rules' multiplayer-dialog difficulty field when the base rules' game-wide values
   are read (Import mapping; S-0006 says when that happens). A lobby control can change it.
   Whether the stock lobby shows such a control, and whether the value restored at a later game
   start replaces a lobby choice, are not established (Open questions). This setting gates triggers only; it is not a house's
   difficulty row or handicap (glossary). A setting outside 0 … 2 selects no difficulty flag: every trigger instance whose disabled
   flag is clear then starts enabled, and the enable action (claim 25) enables every instance it
   names, whatever their difficulty flags. This is fixed compatibility behaviour, not an optional
   quirk, and the importer warns.
   - Backing: `measurable` (scenario T2, variant C for the out-of-range setting). The presence of
     a stock lobby control and restoration of its choice at a later game start are `unmeasured`:
     no scenario identifies that control or follows a choice across starts; they remain open
   - Confidence: medium for the session setting, its base-rules initialisation and out-of-range
     behaviour; low for the lobby

5. **Owner house.** Each trigger instance has the owner house defined in Terms, as it stands at the
   moment the owner house matters, so it follows the houses that exist at that moment. In a skirmish,
   where players choose countries, a trigger whose owner country nobody plays has no owner house,
   and when two players play the owner country, only the one earlier in house order is the owner.
   The owner-country field names a country as follows:

   | Owner-country field | Country named |
   |---|---|
   | `<none>`, in any letter case | the first country of the country list. Original data behaviour, kept, and the importer warns. An exception to S-0003 claim 14, for which `<none>` means no type (Edge cases, *"None" owner*) |
   | `<random>`, in any letter case | none, even if some country has that name or type name |
   | any other text, a bare `none` included | the country whose type name or name field equals the text without regard to letter case; none if no country does. A bare `none` therefore normally names none |

   Only `<none>` is a marker that gives a country in this field. An owner country that names no
   country (a bare `none`, `<random>` or an unmatched text) is unspecified in the original beyond
   giving no owner house. Our engine gives such a trigger no owner country, and
   the importer warns (a departure).
   - Backing: `measurable` (scenario T22 for the letter case, the name field and `<none>`, and
     scenario T20, variant B, for an owner country that nobody plays). The rule that the earlier of
     two players of the owner country is the owner is `unmeasured`: within this subset the owner
     house shows only through the owner-house raise of claim 35, and either player would give the
     same extra firing. The lightning storm of claim 30, whose owner is the owner house, can show it
     once the superweapon specs exist. The original's result for `none` and `<random>` is
     unspecified beyond naming no country, so it has no expected value
   - Confidence: medium (low for the earlier of two players)

### Polling, evaluation and firing

6. **Polled tags are raised once per tick.** Once per tick, at the start of that tick's game logic,
   every polled tag receives one poll. The observable results of a tick's poll follow tag registration order: when an earlier-registered
   tag's actions change state that a later-registered tag evaluates, that later evaluation observes
   the changed state. For the events of this subset, polling a tag has the effect of a single raise of it in which
   no code-matched event is judged true: condition events, the any-event code, empty event lists
   and latches decide (claim 8). So in every tick each polled tag is evaluated once by polling,
   and each of its trigger instances fires at most once from that poll. Its carriers can raise it
   again in the same tick (claims 18 to 20), and so can its owner house (claim 35). Actions that a polled tag fires take effect before the
   later polled tags are evaluated in the same tick, and before the rest of that tick's game logic.
   Whether a tag is polled is decided once, at the end of loading (Terms), from the codes of its
   chain's events. This applies to every tag that exists then, including those created earlier in
   loading by cell tags and pre-placed objects.
   - Backing: `measurable` (T4 for tag order; T5 b for one poll firing per tick; T20 C for polls
     before owner-house raises). The position before every object's update is `unmeasured`:
     these scenarios do not make a moving object depend on an action in that same tick
   - Confidence: high for once per tick and at most one firing per instance per poll, medium for the position at
     the start of the tick's logic and for the order

7. **Registration order of tag definitions.** Tag definitions are registered in the order in which
   the map first names their IDs. While the trigger list is read (in its entry order, each
   trigger's actions in list order), an action parameter that gives a tag ID registers that tag
   definition. The tag list then registers the rest in its entry order. A cell tag or other later
   reference (I2) to an ID that is still unregistered registers a new tag definition with an empty
   chain, which has no effect (Edge cases, *References that create definitions*). A pre-placed object's tag
   field never registers one (claim 2).
   - Backing: `measurable` (scenario T4)
   - Confidence: medium

8. **When a trigger is satisfied.** In a raise, each enabled, not removed trigger instance of the
   tag is evaluated on its own. It is satisfied when every one of its events is satisfied in that
   same raise, either by a latch (claim 9) or by being judged true:
   - a code-matched event is judged true only when the raise has its code and the rest of its
     condition holds (claims 18 to 20),
   - the attack events 6 and 44, which S-0601 claim 5 raises on a tagged target. Every action of the
     event-6 raise's satisfied triggers has completed before any event-44 trigger is judged, so those
     actions are visible to it. Event 6 is judged true in any raise carrying its code. Event 44 is
     judged true only in a raise carrying its code that has a causing object, and only when the
     event's raw integer value equals the house number (S-0009 Terms, counting from 0) of the causing
     object's owner at the moment of that raise. The value is not a country index and not a
     position letter: −1 and the player-position values of claim 23 equal no house number, so such
     an event is never judged true, and a value one above or one below the owner's house number does
     not match either. Neither event records a triggering house (claim 18). The order of triggers
     within each raise is claim 11's. Neither is latchable (Constants),
   - a condition event is judged against the current state, whatever the raise's code,
   - the any-event code is judged true in every raise.

   A trigger with an empty event list is satisfied in every raise of its tag. Tags are also raised
   on occasions outside this subset, with codes outside it: for example, a hit on a carrier raises
   its tag one or more times, depending on the attacker and on the health thresholds crossed; the
   destruction of a carrier raises its tag once more right after the destroyed-by-anything raise
   (claim 19); a unit's arrival in a tagged cell raises the cell's tag a second time right after the
   entered-by raise (claim 18); a unit in the air raises the tag of the cell beneath it in every tick
   whose number is a multiple of 16; an attack on a building raises every tag assigned to the
   building's owner (claim 35); and some team events raise every polled tag outside the per-tick
   poll (the codes, their number and the exact conditions belong to a later revision). Such raises evaluate condition events, empty event lists and latches
   like any other. Every event of the
   trigger is judged in every evaluation, whatever the results of its other events. That matters for
   latches (claim 9) and for the triggering house (claim 18). Consequence: a trigger that
   combines two code-matched events of different codes can never be satisfied without a latch, and
   one that combines a code-matched event with a condition event fires only in a raise of that code
   in which the condition already holds.
   - Backing: `measurable` (scenarios T5 and T10) for evaluation and the hit raise; T7 for the
     post-destruction raise; for the attack events 6 and 44 raised by S-0601 claim 5, the backing of
     that claim (its scenario MC14, which tests their order, cause and judging, including the
     house-number base through the one-above and one-below values). The second arrival raise,
     airborne-unit raises, building-under-attack raises and team raises are `unmeasured`: these
     scenarios do not isolate their firings from arrival, damage or team activity; their precise
     conditions remain open
   - Confidence: high (low for the list of out-of-subset raises); for the attack events 6 and 44,
     S-0601 claim 5's confidence (high; medium for the outcome with special house values, which MC14
     part a tests), and here medium for the house-number base, which only MC14's one-above and
     one-below values pin down

9. **Latches on repeating tags.** When an event of the latchable set (Constants) is judged true in
   an evaluation of a trigger instance of a repeating tag, it is latched, whether or not the
   trigger as a whole is satisfied. A latch is never cleared except for the timer-position
   alias described below. So, after its first satisfaction, a
   latched event counts as satisfied in every later raise of its tag, of any code. Example: a
   repeating tag carried by two vehicles, with one trigger whose single event is
   destroyed-by-anything. A normal first vehicle's destruction fires the trigger seven times
   with an attacker and four without (claim 19): its first destroyed-by-anything raise sets the
   latch, so every remaining destruction raise fires it. A common-group-only death gives two
   firings instead. After that the trigger fires again in every later raise of the tag, for example in each of the
   raises that every hit on the second vehicle causes (`quirks.persistent_event_latch`). Latches are kept per event position, and positions from 33
   on share the latch of the position 32 lower (`quirks.event_latch_aliasing`). Arming a timer
   clears the latch at each timer event's position; with aliasing on this also clears any
   latchable event sharing that position. Such an aliased latch can therefore be lost on creation
   or re-arming, including the re-arming caused by a firing. With aliasing off, timer positions
   cannot clear a latchable event's independent latch. A few codes outside
   this subset also latch on other repeat types (Open questions).
   - Backing: `measurable` (scenario T7; variants C and D for alias satisfaction and clearing)
   - Confidence: medium

10. **Repeat types.** The repeat type decides what happens when a trigger instance is satisfied in a
    raise. The full table is in State and ordering. In short: a one-time tag fires the trigger, and
    then the tag leaves all its carriers and is deleted. An all-carriers tag fires the trigger only
    when the tag has exactly one carrier left, and otherwise only detaches the raising carrier. A
    carrier that stopped existing while it carried the tag still counts (claim 2), so when one of
    two carriers is destroyed and none of its destruction raises satisfies the trigger, the other's
    later raise finds two carriers and only detaches it (scenario T6, variant H). So an all-carriers
    tag with no carrier (for example one that is only polled) never fires
    (`quirks.all_carriers_without_carrier`). A repeating tag fires the trigger every time and stays.
    Any other repeat value never fires, and the importer warns.
    - Backing: `measurable` (scenario T6, including G for an unknown repeat value and H for a
      carrier that stopped existing)
    - Confidence: high for one-time and repeating, medium for all-carriers

11. **Firing.** When a trigger fires, its actions take effect in the order of its action list,
    immediately, within the raise that satisfied it: the effects are in force for whatever follows
    in the same tick. Every action takes effect even when an earlier one had no effect. All chain members satisfied in the same raise fire in that raise. Their observable effects have
    the reverse of chain order: the first trigger's effects are observed after those of later
    members. A member's eligibility reflects effects that are observable before its own effects in
    this order. Thus one member's actions can prevent another member from firing (for example, by
    disabling it) or make it satisfy a condition (for example, by setting a variable it tests). A fired member of a one-time tag is removed at once. The other members
    that have not fired in a one-time tag stay eligible for other raises until the end of that tick
    and are gone from the next tick on, together with the tag.
    - Backing: `measurable` (scenario T8; B for an earlier member's effect on a later one and
      C for action order and continuation after a no-effect action) for the order within a raise. The survival of the other
      members until the end of the tick is `unmeasured`: it needs two different raises of one
      one-time tag in one tick after it fired, which no test map can arrange reliably
    - Confidence: medium for the order within a raise, high for the action order, low for the
      survival until the end of the tick

12. **No nested raises.** A raise of a tag that happens while that same tag is being raised (for
    example because one of its actions destroys one of its carriers) has no effect.
    - Backing: `measurable` (scenario T9)
    - Confidence: medium

### Time

13. **Trigger timer and elapsed-time event.** Every trigger instance has one trigger timer. It is
    armed when the instance is created, at tick *s*, with the length *D* of Formulas F1 (or F2 for a
    random-delay event). Every instance created while the game loads (claim 34) has *s* = 0, because
    the tick counter stays 0 until the first tick. If the trigger has several timer events (elapsed-time or random-delay), the
    one written first in the trigger's event list sets the timer, and both kinds of timer event
    are satisfied when that one timer has run out (F3). A trigger without a timer event ignores its
    timer. Consequence: a one-time polled trigger whose only event is elapsed-time with value *v*
    fires in the tick numbered 15*v* when that product is nonnegative and fits the integer
    range (the first tick is numbered 0). A nonpositive length has already run out in tick 0.
    - Backing: `measurable` (scenario T10, variant H for a wrapped product). Timer subtraction
      across tick-counter overflow is `unmeasured`: no scenario runs through that boundary
    - Confidence: high for the length and the tick, medium for "first written wins"

14. **Random delay.** A random-delay event with value *v* arms the timer with the length of
    Formulas F2. Each arming takes one ranged draw from the scenario random stream for every
    random-delay event of the trigger (none when *v* = 0), in reverse order of the event list,
    whether or not that event sets the timer. The draws happen when the timer is armed (claims
    15 and 34).
    - Backing: `measurable` (scenario T11). Variant A, statistical, gives the distribution of the
      delay. Variant B reads the number, order and values of the draws with the random-state
      recorder of the measurement setup, which only reads the original's seed and stream state.
      Variant C tests a negative value. Sum/product overflow in F2 is `unmeasured`: these
      variants use small values and do not distinguish a wrapped random-delay result
    - Confidence: high for the formula, medium for the draw count and order

15. **Re-arming.** A trigger timer is armed again, at the current tick, with a new length (and new
    draws), when:
    - the instance is satisfied in a raise (whether or not its repeat type then lets it fire),
      and either the tag is repeating or an entered-by event of the trigger was judged true in that
      raise. An entered-by event judged true in a raise in which the trigger as a whole is not
      satisfied does not re-arm the timer. A repeating polled trigger with only a timer event
      therefore fires every *D* ticks when *D* > 0 (a new *D* each time for a random delay).
      With *D* ≤ 0 it fires at each poll, once per tick, and can fire at other raises too,
    - the enable action enables it (claim 25), even when it was already enabled,
    - a map trigger variable changes value and the trigger has a variable-set or variable-clear
      event for that variable (claim 22).

    **Draw order and a self-enable action (Q-0269).** Satisfaction-caused re-arming takes all
    its draws before any action of that instance takes effect or draws from the scenario random
    stream. Thus a repeating random-delay trigger's new delay draws precede its explosion's
    animation-creation and area-damage draws (claim 28). This is per instance, not a batch
    before every member's actions: the effects of an earlier-firing chain member, including
    its action draws, precede the satisfaction-caused re-arming of a later-firing member
    (claim 11's reverse chain order).
    If an instance's own action list enables its definition and passes claim 25's difficulty
    gate, that enable action arms it again even though it was already enabled. Both armings
    take their full applicable draws. The enable action's length replaces the length from
    satisfaction; both have the firing tick as their arming tick. Other existing instances of
    the definition are also armed by that action under claim 25. A difficulty-gated enable
    action that has no effect does not replace the satisfaction-caused length. Any still later
    re-arming in the same action list replaces that length in turn.

    Nothing else arms it again. In particular disabling does not. The exhaustive assertion
    “nothing else” applies to the events and actions covered here, not to the deferred codes.
    - Backing: `measurable` (scenario T10, variants B to D; scenario T5 (c) for the entered-by
      case; T10 G for re-enabling an already-enabled instance; T11 D for re-arming/action
      draw order and the self-enable replacement)
    - Confidence: medium overall; high for satisfaction-before-action draw order and
      the effective self-enable action replacing the satisfaction-caused length

16. **Game-time event.** A game-time event with value *v* is satisfied in the tick numbered *t*
    exactly when F4 holds. It counts from the start of the game, so it is not affected by arming.
    - Backing: `measurable` (scenario T10, variants E and I)
    - Confidence: high

### Events of the subset

17. **Any event.** The any-event code is satisfied in every raise. Because it is in the polled
    set, a tag whose chain contains it is polled, so a one-time trigger whose only event is the
    any-event code fires in the first tick of the game (tick 0). A repeating one fires in every tick
    from polling, and once more in every tick whose number is a multiple of 8 when its tag is also
    raised by an owner house (claim 35). It also fires in every other raise of its tag.
    - Backing: `measurable` (scenarios T5 and T20)
    - Confidence: high (medium for the second firing, which claim 35 covers)

18. **Entered-by.** A tag is raised with the entered-by code, with the arriving object as causing
    object, when a unit arrives in a cell that carries the tag. S-0207 claim 22 owns the
    arrival timing for driving vehicles and own-locomotor teleports of any object kind.
    S-0208 claim 22 owns ordinary walking infantry arrival timing. These are movement
    rules, not a requirement to stop in the cell: a completed driving track target or walking
    target can raise the tag while the unit continues its journey. Teleport arrival is in
    the jump tick after landing placement, not at reservation or phasing-in completion.
    The cited claims own intermediate-cell exceptions and ordering against other arrival
    effects. In a cell that carries a high bridge, only units on the bridge
    deck raise it. S-0524 claims 10 to 12 own object-entry raises: engineer capture,
    vehicle hijacking and the other classified entry paths raise the entered object's tag
    with the entrant as causing object. Capture and hijack entry actions see the old owner
    and tag, before carry-over and takeover. These are distinct from notifications on the
    entrant's own tag when it is consumed (claim 19); merely receiving an enter order
    does not raise the entered object's tag. The event is satisfied when the raise has this code, there is a causing object,
    and either the event's value is −1 (any house) or the value is a country's registration index,
    a house plays that country, and the causing object's owner is the first such house in house
    order. A value in the player-position range of claim 23 names no country, so the event is never
    judged true (`quirks.entered_by_country_only`). Whenever the event is judged true in an evaluation of the
    instance, whether or not the trigger as a whole is then satisfied, the owner of the causing
    object becomes the instance's triggering house. Within this subset the triggering house is read
    only by the actions of a firing, and a trigger fires only in a raise in which its entered-by
    event is judged true, so an action always reads the house of that raise. For a one-time or all-carriers
    tag the raising carrier loses the tag when the trigger fires or is satisfied (State and
    ordering).
    - Backing: `measurable` (scenario T3, including variants B and C for the bridge and capture)
     for cell arrival, capture, country matching and the triggering house. Movement timing
     inherits S-0207 claim 22/D24 and S-0208 claim 22/W14's backing and confidence.
     Object-entry eligibility, timing, tag transfer and consumption distinctions inherit
     S-0524 claims 10 to 12's backing and confidence (C9 and C10, including their
     unmeasured entry-classification boundaries)
    - Confidence: medium (low for the bridge rule); object-entry rules inherit the cited claims

19. **Destroyed-by-anything.** A killing hit on an object carrying a tag raises that tag with
    this code, with the dying object as causing object. The no-hit destruction cases below
    also raise it; a killing hit is not a prerequisite of the event itself. The event is
    satisfied in every such raise and is latchable (claim 9). S-0312 claim 29 owns the killing-hit timing and destruction-raise
    groups: their actions and draws take effect within the killing hit, before death debris,
    passengers and death weapons, not at final removal. The raises themselves take no draw.

    The common group has three attacker-dependent raises, followed by destroyed-by-anything and one
    further outside-subset raise. The first of the three, code 6, is in this subset: its raise judges
    event-6 triggers as claim 8 says. The other two are outside it. Vehicles normally have an
    additional earlier group: an attacker-dependent raise with code 7 (outside this subset),
    destroyed-by-anything and its following outside-subset raise. Every raise evaluates condition events and latches
    (claim 8). A raise that detaches or deletes the tag prevents further raises of that tag
    from this object. Thus a fresh repeating trigger with destroyed-by-anything as its only
    event fires seven times for a normal vehicle death with an attacker and four without,
    while a common-group-only death fires twice. These are counts of firings within the
    killing hit, not counts of ticks; earlier hit-related raises do not yet have this latch.
    With persistent latching off and the extra vehicle group on, the same normal vehicle
    death fires twice, once per destroyed-by-anything raise, and subsequent nonlethal hits
    do not fire it. With the extra group off and persistent latching off, it fires once.

    S-0312 claim 29 owns the common-only cases: a vehicle with a remembered hijacker type
    when any definition in its tag has the carry-over flag, separately supplied attacking-house
    attribution, and `quirks.vehicle_destruction_extra_raises` (default original; off omits
    the extra vehicle group). It also owns deferred-death timing: a cyborg reprieve has no
    destruction raise; a delayed-kill building raises on the initial lethal hit and again on
    lethal expiry if still tagged; sinking and crashing do not postpone the killing-hit raises.
    Infantry still showing a death sequence keep their tag unless evaluation or another
    detachment removed it, but are no longer present objects for claims 24 and 27. Completing
    that sequence and removing the infantry produce no new destroyed-by-anything raise.
    This is an ordinary final-removal rule, not a rule that every destruction without a hit
    lacks notifications.

    **Hidden slaves without their own killing hit (Q-0510).** When a hidden slave is
    destroyed as its group ends, it receives the common group, with itself as causing object,
    before its removal, without receiving damage. This applies when the master takes its
    death effects after a killing hit (S-0312 claim 30), when removal ends its remaining,
    untransferred group without a killing hit (S-0312 claim 31), when a true sale removes
    the master (S-0408 claim 10a), and when successful pack-up discards the return vehicle's
    temporary hidden slaves before transferring the old working group (S-0312 claim 31 and
    S-0415 claim 10). The master's known attacker enables the common group's three
    preliminary raises in the killing-hit case. The other three cases have no attacker
    and omit those raises. Destroyed-by-anything and its following outside-subset raise
    occur in all four cases, provided the slave still exists and carries the tag.
    Successful transfer of the old working group is not destruction of its surviving slaves.
    Earlier group-ending events and the exact cleanup tick retain the boundaries of the
    cited claims; this claim does not replace their attribution or timing rules.

    Claims 8 to 12 apply unchanged: there is no hidden-slave exemption from evaluation,
    latching, repeat types, action ordering, detachment or same-tag nested-raise suppression.
    A fresh repeating tag with only destroyed-by-anything, no prior latch and actions that
    leave the slave and tag alone fires twice for one hidden slave, with or without an
    attacker: the first firing latches the event and the following outside-subset raise
    fires it again. With persistent latching off it fires once. A one-time tag fires once
    and detaches; all-carriers tags use their actual carrier count under claim 10, not a
    special hidden-slave count. If the tag was already latched, preliminary raises with an
    attacker can also fire it. A raise suppressed by claim 12 fires nothing and sets no
    latch. Removal after these notifications produces no second group.

    **Entrants consumed by entry.** S-0524 claim 10 owns the engineer's single
    destroyed-by-anything raise on its own tag before removal, including damage-only
    capture attempts. With carry-over, that tag can already be shared with the captured
    building. S-0524 claim 12 owns the other traced consuming-entry notifications and
    their incomplete classification. These are not the killing-hit groups above: do not
    add their preliminary or following raises. Hijacking removes the thief without this
    notification (S-0524 claim 11); garrisoning does not itself consume the entrant.

    **Other no-hit destructions.** Hidden slaves are not the only such case: destruction of
    an exploding object's passengers under S-0312 claim 5 also gives tagged passengers
    destruction notifications without a hit on each passenger. Their object kind decides
    the common and extra-vehicle groups above; the transport's known attacker supplies the
    attribution. The passenger's own notifications precede its removal and the transport's
    death weapon. The same evaluation and repeat rules apply. This is a confirmed additional
    case, not an exhaustive inventory of removals. Removal alone must not be treated as a
    universal destruction raise, nor must every no-hit notification be suppressed.
    - Backing: `measurable` (scenario T7 for killing-hit counts and latches; S-0312 D20 for
      timing, deferred deaths and retained infantry tags; T25 A for tagged exploding-transport
      passengers). Entry-consumption notifications inherit S-0524 claims 10 to 12's
      backing and confidence (C9; claim 12's other consumed-entrant paths remain
      `unmeasured`). Hidden-slave notifications and their firing counts are `unmeasured`:
      no established data-only setup attaches a tag to a generated slave; T25 B is a
      conditional proposal, not an established fixture. Separately supplied attacking-house
      attribution and actions that remove the dying object during the raises inherit
      S-0312 claim 29's `unmeasured` boundary. The event-6 firings of the code-6 raise are
      `unmeasured`: no scenario here puts an event-6 trigger on a dying object, so their count and
      timing have no measured backing. The firing counts above are of destroyed-by-anything triggers
      only
    - Confidence: high for the groups, killing-hit counts and timing; medium for latches,
      no-hit notifications and their counts, and the retained-infantry and hijacker/carry-over cases, and low for
      the event-6 firings of the code-6 raise

20. **Bridge-destroyed event bands.** This event is satisfied in every raise with its code.
    S-0511 claims 7 and 8 define the bands that raise it; they are not limited to cells that
    actually collapse. A high-bridge band's endpoints are the first cell after the span's
    start piece and the far piece. A low-bridge band's endpoints are the first and last low
    pieces of its whole contiguous run, destroyed pieces included.
    The band is 4 cells wide across the bridge: for each centre-row cell, its own cell,
    the next cell across toward increasing cross-coordinate, and the one and two cells
    toward decreasing cross-coordinate. Along an x bridge the cross-coordinate is y,
    and along a y bridge it is x. Tagged cells raise their tags in increasing
    along-bridge coordinate order, in that cross-cell order. The last centre-row cell
    along the band and its cross-cells are skipped; a one-cell band raises nothing
    (`quirks.bridge_event_band_short`, owned by S-0511 claim 8).
    Each raise carries no causing object and no cell. A shared tag is raised once per
    qualifying tagged cell, not once per distinct tag. A destroyed low strip gives one
    pass over its band. For a high span with a missing section, each pass of the span
    check gives one band pass: with *n* isolated section losses, there are *n* + 1 passes,
    before the first loss, between losses and after the last (S-0511 claim 7).
    A span with no missing section raises nothing.
    - Backing: `measurable` (scenario T13; S-0511 B5 and B14)
    - Confidence: medium for low bands, cross-cells and no-object/no-cell raises,
      low for high-band endpoints and repeated passes

21. **Type exists and type does not exist.** These condition events name a type (by its type name)
    and a count *n*. The type is the building, vehicle, infantry or aircraft type whose stored name
    equals the event's name exactly, letter case included. If several do (S-0004 claim 8), the
    original takes the last of them in a single order that runs over those four kinds together; that order is not established. If none does, neither event is ever judged true, and the importer warns. Otherwise
    type-exists is satisfied when at least *n* objects of that type exist, counting every object of
    the type of every house, including objects inside transports or buildings. When *n* ≤ 0 it is
    satisfied as soon as any building, vehicle, infantry or aircraft object exists, of any type
    (fixed compatibility behaviour, not an optional quirk; the importer warns about a count below 1).
    Type-does-not-exist is satisfied when no object of the found type exists. The exact match is an
    exception to S-0003 claim 14, which matches type names without regard to case (Edge cases,
    *Exact type names*). The text `<none>` and `none` gets no marker treatment here: each is
    looked up as an ordinary exact type name. When no type has that exact name, neither event
    is judged true.
    - Backing: `measurable` (scenario T14; variant C for *n* ≤ 0, variant D for an object inside a
      building)
    - Confidence: medium (low for which objects count while inside another object, and for
      type-does-not-exist). Which duplicate type name wins across kinds is `unmeasured`: T14
      uses a unique name and the combined lookup order remains open. The absence of marker
      treatment is `unmeasured`: when no registered type has a marker spelling, lookup failure
      and marker rejection both make these events false, so T14 cannot distinguish them

22. **Map trigger variables.** The map trigger variables of S-0009 claim 32 are numbered 0 to 99.
    The variable-set event is satisfied while its variable is set, and the variable-clear event while
    it is clear: both are level conditions, so a repeating trigger with one of them fires in every
    tick in which the state holds. The set and clear actions change the variable named by their
    number slot. A change happens only when the state differs. Then every trigger instance that has a
    variable-set or variable-clear event for that variable has its timer re-armed (claim 15), and all
    tags see the new state in the rest of the same tick and in later ticks. A set or clear action naming a variable outside 0 … 99 has no
    effect. A variable event naming a variable outside 0 … 99 is unspecified in the original. Our
    engine treats that variable as clear (the set event is never judged true and the clear event
    always is), and the importer warns (a departure).
    - Backing: `measurable` (scenario T12 and T10 D/J for re-arming and unchanged assignments). Out-of-range variable-event
      results are `unmeasured`: the original is unspecified, so there is no stable oracle value
      for our clear-variable fallback
    - Confidence: medium

### Actions of the subset

23. **House values.** An action that names a house resolves its house value when it takes effect:
    | House value | House |
    |---|---|
    | 9013 | the trigger instance's triggering house (claim 18), or none |
    | −1 | none |
    | 4475 … 4482 | the player at position A … H, or none: S-0009 claim 21, and on a map whose header count N_h is 9 or more the "References" rule of S-0009 claim 29, by which the house can be a passive one |
    | any other value *c* | the first house in house order that plays the country with registration index *c*, or none |

    For player positions this is one rule on every map: the position's start position mark (S-0009
    claim 29) names the house by its house number, and a mark that names no house gives none. On
    maps with fewer than 9 header pairs that is the player who holds the position (S-0009 claim 21).
    An action whose house resolves to none has no effect.
    - Backing: `measurable` (scenario T15, variant C for N_h ≥ 9)
    - Confidence: high for the player positions, medium for the others

24. **Change owner.** Every present object (Terms) that carries the tag holding the firing trigger
    changes its owner to the resolved house. A carrier inside a transport or building keeps its
    owner. Cells are unaffected. The ownership change itself (what an
    object keeps or loses, and any announcement) belongs to the object specs (a pointer only); this
    action asks for a change without announcement.
    - Backing: `measurable` (scenario T15)
    - Confidence: high for which objects change (medium for objects whose destruction has begun),
      medium for "without announcement". The destruction-in-progress exclusion is `unmeasured`:
      T15 does not align this action with a surviving death sequence; “without announcement” is
      `unmeasured` because T15 records ownership only, not announcements

25. **Enable and disable a trigger.** The enable action names a trigger definition. Every existing
    instance of that definition, in any tag, becomes enabled and has its timer re-armed (claim 15) when its difficulty flag at the game's difficulty position is set. If that position is outside 0 … 2, every existing instance of the definition becomes enabled and has its timer re-armed, whatever its three difficulty flags. Instances that do not meet the applicable condition are left as they are. The disable action disables every instance of the
    named definition. Neither action creates instances, so naming a definition that is no member of
    any existing tag's chain has no effect.
    - Backing: `measurable` (scenario T2 B, T8 B for disable, T10 C/G for enable, and T1's
      independent-instance variant for enabling all instances)
    - Confidence: high

26. **Reveal around a waypoint.** The action's number slot is a waypoint number. If the local player has not already had the whole map revealed, the original reveals, for the
    local player only, the area around the point of Formulas F6 using the area-reveal radius rule of
    S-0409 claim 17: a positive radius *R* is capped at 10 cells, and a radius of 0 reveals no cells.
    Here *R* is the reveal-trigger radius of the rules (Import mapping). Negative *R* is not specified
    here; its area-reveal result remains open rather than being inferred from this claim. How an area
    is revealed belongs to the visibility specs (a pointer only). In a skirmish the local player is the single human-controlled house, so
    that house has the area revealed and no computer-controlled house has. In a network game each
    machine reveals the area for its own local player, which is outside this spec (Terms). Our
    engine applies the reveal to every player house (human or AI) that has not had the whole map
    revealed, observers included, so the result does not depend on the machine (a departure
    from the original's per-machine result).
    - Backing: `measurable` (scenario T16) for the area in a skirmish. The condition "not already had the whole map revealed"
      is `unmeasured`: revealing a fully revealed map changes nothing that can be seen, and the
      condition would show only where shroud has come back after the whole map was revealed, which
      no action of this subset arranges
    - Confidence: medium (low for the condition)

27. **Destroy attached objects.** Every present object (Terms) that carries the tag holding the
    firing trigger takes damage equal to its current health, with no attacker and no attacking house. The damage is applied with defences
    ignored: armour, the warhead's percentages and temporary invulnerability do not reduce it, so the
    object is normally destroyed. Protections that depend on the warhead's kind still apply: a type
    immune to radiation, psychic or poison damage takes no damage from a warhead of that kind,
    and a warhead that makes its target go berserk takes no health. General immunity to all damage
    does not stop this ignoring-defences hit, except for an immune bridge repair hut (S-0302 claim 8). A building that is eligible for a delayed kill remains alive at
    1 health when the demolition warhead causes delayed kills. An infantry unit whose height
    above the surface is strictly greater than 0 leptons, hit by a demolition warhead of
    infantry-death kind 9, has its delivered amount reset to 0 and the hit is stopped by the
    zero-amount rule. This is a height test, not an airborne-state test: standing infantry on
    a high bridge deck have height 0 above the surface and do not qualify. These additional
    cases are the damage-resolution rules of S-0302 claims 8, 9, 16 and 17. Confirm this
    height test together with S-0302 claim 17 at sign-off. The rules' demolition warhead
    (Import mapping) is named as the
    damage's warhead. Its kind matters for the protections above as well as for how the destruction
    looks and what it leaves (S-0312, a pointer only).
    If an object survives the damage, the action never completes in the original: from that tick
    on the game stops responding, the tick counter stops advancing, and neither the trigger's later
    actions nor any later tick happen. Our engine
    damages each such object once per firing and goes on, so a survivor stays (a departure: the
    original's result is a hang), and the importer warns when a trigger-kill setup can leave an
    object alive under any of the survivor rules above.
    After the object hits, when the raise that fired the trigger carries a cell, the action
    makes up to 4 bridge destroy attempts at that cell, stopping at the first success
    (S-0511 claim 1). These attempts ignore the game's bridge-destruction setting.
    S-0511 claim 2 owns what each attempt affects. A raise with no cell makes no cell
    attempt; in particular, claim 20's bridge-destroyed raises carry no cell, even though
    tagged cells produce them.
    - Backing: `measurable` (scenario T9, including the armour variant and variants C and D for
      radiation-kind and berserk survivors; S-0302 scenario D16 parts d2, d3 and d4 for the immune hut,
      delayed-kill building and positive-height-above-the-surface infantry survivors and the
      resulting hangs; T9 E and S-0511 B10 for cell attempts, the gate and no-cell control).
      Psychic-kind and poison-kind survivors and bypass of temporary invulnerability are
      `unmeasured` here: T9 varies radiation, berserk and armour, not those protections
    - Confidence: medium for object damage (the hang has not been observed),
      low for the cell effect

28. **Explosion at a waypoint.** The action's number slot is a weapon's registration index (S-0004)
    and its waypoint slot names a point: the centre of the waypoint's cell, at the cell's centre
    height (the terrain surface height at the cell's centre, S-0106 claim 2, so on a slope it can
    differ from height level × 104), raised by 4 height levels (416 leptons) when the cell carries a
    high bridge. At that point the action has three effects. It creates the damage animation that the
    weapon's warhead selects for the weapon's damage and the cell's land, when the warhead selects
    one (the draws of an animation's creation belong to S-0109). It shows a light flash (S-0102),
    for a warhead that makes one and under the conditions that S-0102 gives for showing it. The flash
    is drawing only, takes no draw and changes no game state. And it deals area damage of the
    weapon's damage with the weapon's warhead, with no attacker and no attacking house.

    In the scenario random stream, the draws that the animation's creation takes come before the
    draws that the area damage takes. That is the only order among the three effects that can be
    observed. Animation choice and area damage belong to the
    combat and animation specs (a pointer only). A number-slot value that is no registration index of a weapon is unspecified in the
    original, which does not reject it. Our engine gives such an action no effect, and the importer
    warns (a departure).
    - Backing: `measurable` (scenario T17; the height of the point by scenario T21, which uses the
      same point). The order of those two sets of draws is `measurable` with the recorder
      (scenario T17, variant C)
    - Confidence: medium (low for the order of draws)

29. **Make a house cheer.** Every object owned by the resolved house is told to cheer, and a cheer
    sound plays. For infantry this starts the cheer sequence of S-0107 (claims 6 and 21), which a unit
    cannot interrupt, so it can delay its next shot. What other objects do is not established.
    - Backing: `measurable` (scenario T18 for infantry and the sound). Effects on other object
      kinds are `unmeasured`: T18 observes infantry only and their responses remain open
    - Confidence: medium

30. **Actions that hand over to other specs.** Parameters use Import mapping; the
    following are pointers to the claims that own the effects.
    - Create crate at a waypoint: S-0524 claim 4. Lightning storm at a waypoint:
      S-0524 claim 1, with the trigger's owner house as storm owner.
    - Resize local area, demolition blast, large light flash, base ambient light,
      lightning strike, nuclear strike and restore starting buildings: S-0524
      claims 8, 5, 6, 7, 2, 3 and 9 respectively.
    - One-time superweapon grant, permanent grant, set charge and reset charge:
      S-0512 claims 12 to 15 respectively. Setting a charge percentage preserves
      existing readiness under S-0512 claim 14's default quirk; resetting charge
      clears readiness under S-0512 claim 15.
    - Evict occupants from the first eligible attached building: S-0508 claims 10 to 13,
      without hunting and without inside fallback. The house-wide hunt action instead
      resolves its number slot as a house value (claim 23); its garrison evictions use
      hunting, without inside fallback (S-0508 claim 13). These are different actions,
      not two spellings of non-hunting eviction. Their eviction effects inherit S-0508
      claims 10 to 13's backing and confidence (scenario G13).
    - Create team, destroy team, reinforce and reinforce at a waypoint belong to the
      out-of-scope team system. Preferred target at a waypoint belongs to out-of-scope
      computer-controlled targeting policy, with the owner house as the house concerned.
    - Backing: `unmeasured`: this is a pointer inventory, not an independent effect
      claim. The companion claims carry their own backing and measurement scenarios
    - Confidence: medium for the inventory; effect-specific confidence is inherited

31. **Mission timer actions.** In this claim *v* is the action's number slot (timer-set, timer-extend,
    timer-shorten). Timer-set starts the mission timer with a length of 15*v* ticks from the
    current tick and resets its label to the default label, the same one that timer-text with an
    empty text gives. Timer-start restarts a stopped timer with its remaining
    length, and timer-stop stops it and keeps the remaining length. Timer-extend and timer-shorten set
    the timer running from the current tick with its remaining length plus or minus 15*v* ticks (not
    below 0), whether it was running or stopped. Timer-text sets the label from the action's text.
    When a running mission timer reaches 0 it stops. The mission-timer-expired event is outside this
    subset.
    - Backing: `measurable` (scenario T19: the main setup for timer-set and timer-extend, variant B
      for timer-stop and timer-start, variant C for timer-shorten, variant D for timer-text and the
      default label). Extension/shortening of a stopped timer and the clamp at 0 are
      `unmeasured`: these variants modify a running timer only and never cross below 0
    - Confidence: medium (low for timer-start, timer-stop, timer-extend and timer-shorten, and for
      what the default label shows)

32. **Presentation-only actions.** Text message, play sound, play speech, radar event at a waypoint,
    and sound at a waypoint change only what is shown or heard. Sound at a waypoint gives a building or
    terrain object on the waypoint's cell that sound as its ambient sound (or stops it, for an unknown
    sound), and otherwise plays the sound at the cell's centre. A radar event is not shown when an
    event of the same kind is already shown close by; the distance that counts as close by belongs
    to the interface specs. How text, sounds and radar events are shown belongs to the interface and
    audio specs (pointers only). Our engine may show them for the local player.
    - Backing: `measurable` by eye and ear (scenario T5, parts (a), (e) and (f)): nothing in game
      state changes, so the observables are what is seen and heard
    - Confidence: medium (low for the radar-event suppression)

33. **The waypoint parameter.** An action that uses a waypoint parameter acts at the cell of that
    waypoint marker (S-0009 claim 15). An undefined marker stands for cell (0, 0), and the importer
    warns. A waypoint parameter that decodes to −1 (no letter, Formulas F5), and a reveal action whose
    waypoint number lies outside 0 … 701, are unspecified in the original. Our engine gives such an
    action no effect, and the importer warns (a departure).
    - Backing: `measurable` (scenario T17, variants B and D). Unspecified original results for
      invalid waypoint numbers have no stable expectation and are `unmeasured`; the no-effect
      fallback is our engine's design rule
    - Confidence: medium

### Loading and draws

34. **When tags are created, and the order of draws while a map loads.** A tag is created, and its
    trigger instances with it (arming their timers, claim 13, and taking the draws of claim 14), at
    the first of: the reading of a cell tag that names it, the reading of a pre-placed object that
    names it, or the end of loading, where every remaining tag definition that is polled (or needed
    by codes outside this subset) gets its tag, in registration order. A tag named by both a cell
    tag and a pre-placed object is created, and takes its draws, at its cell tag. The draws of tags
    created at cell tags come after the draws of the initial fill and before the draws of the first terrain recalculation pass (table *Load-time draw order*). For
    a pre-placed object the draws come in this order: the object's creation draw (S-0202 claim 10),
    then the draws of its tag if the tag is created there, then the draws of placing the object on
    the map (S-0202's map-loading table owns the established placement draws; others remain open). An object whose house or type is not found is not created and creates no tag. The table *Load-time draw order* (State and ordering)
    puts these draws together with the draws that S-0009, S-0010, S-0109 and S-0202 assign to
    loading, in one stream order. Cleanup can take conditional tile-animation draws (S-0202
    claim 14), and secret-lab choices (S-0407 claim 14) precede the remaining-tag arming draws.
    The game-start pass stays after those groups even when no starting units are placed or their
    setup stops early. Conditional resource-glint choices and their animation-creation and
    immediate start-event draws follow the game-start recalculation (S-0202 claim 14).
    Recalculation is not an exhaustive final-draw boundary for loading. Every one of these
    draws happens before the first tick.
    - Backing: `measurable` (scenario T23, including variant D for secret labs and variant E
      for resource glints): the random-state recorder reads the stream state before
      the first tick and the draws between the groups the table lists. The boundary of the claim,
      the first terrain recalculation pass, is a group the table lists
    - Cleanup's relative position and conditional animation draws are `unmeasured`, as in
      S-0202 claim 14: its former R11 used a network fixture, not a supported discriminating
      skirmish setup. T23 does not supply a replacement fixture.
    - An object with an unresolved house or type creating neither an object nor a tag is
      `unmeasured`: T23 uses valid object entries and does not test rejected entries.
    - Confidence: medium (high for the order of the groups the table lists; whether other draws
      happen while a map loads is not established)

### Owner-house raise

35. **House-scope tags are raised by their house every 8 ticks.** At the end of loading (Terms), a
    tag whose chain contains an any-event (or an event of a house-scope code outside this subset) is
    assigned to the house that is then the owner house (Terms) of the *first* trigger of its chain, if
    there is one. That moment comes after the unused-house cleanup of S-0009 claim 28 and after the
    starting units are placed, so a house that the cleanup removed gets no tags.
    In every tick whose number is a multiple of 8 (tick 0 included), each house, as part of its own
    per-tick update near the end of that tick's game logic, raises every tag assigned to it with the
    any-event code, without a causing object or cell. The observable effects of house raises follow house order (S-0009 claim 19): effects of an
    earlier house are in force when a later house's assigned tags are evaluated. For tags assigned
    to one house, the observable effects follow reverse registration order (claim 7): effects of a
    later-registered tag are in force when an earlier-registered tag is evaluated. The
    owner-house raise evaluates every trigger of the tag like any other (claim 8), so condition events (elapsed
    time, game time, variables, type counts) and latches can fire in it, as well as the any-event
    code. The assignment is made once and never changes. S-0414 claim 18 owns the
    within-update timing: this raise sees that update's surrender destruction and defeat test,
    including defeat destruction, after power and ordinary radar rechecks but before due
    tech-tree work and its sidebar cancellation orders. It does not cause a second defeat
    test in that update. A defeated house stays in house order and keeps raising its tags,
    including in the update that defeats it and later eligible ticks during the end delay
    (S-0414 claim 18/V15). A tag whose first trigger's owner country nobody plays is never
    raised this way. A house also raises all its assigned tags, with a code outside this subset, each time
    it is told that one of its buildings is under attack; how often that happens belongs to the
    combat specs (a pointer only). That raise, too, evaluates condition events and latches.
    - Backing: `measurable` (scenario T20, including variant C for house/tag order, and
      S-0414 V15 for within-update timing and continued raises after defeat).
      The under-attack raise is `unmeasured`: T20 does not isolate attack notifications,
      and this subset does not control their timing.
      The position after all objects' updates is `unmeasured`: T20 compares house and poll
      effects, not movement effects in the same tick
    - Confidence: high for within-update timing and continued raises after defeat,
      medium otherwise (low for the reverse order within a house and for the under-attack raise)

### Animations at a waypoint

36. **Play an animation at a waypoint, and the drop-zone animation.** The play-animation action's
    number slot is an animation type's registration index (S-0004); the drop-zone action uses the
    rules' drop-zone animation type (Import mapping) and ignores its number slot. Both create one animation of
    that type at the point that claim 28 uses for its waypoint slot: the centre of the
    waypoint's cell, at the cell's centre height (S-0106 claim 2), raised by 4 height levels when
    the cell carries a high bridge. In the terms of S-0109, the animation has no loop delay, a
    repeat factor of 1 and no reversed play, and is attached to no owner. How it plays, and the
    draws its creation takes, belong to S-0109 (a pointer only).
    - **Trigger-played animation behaviour.** The animation delivers no damage. Its creation
      start event has the usual start sound and resource chain reaction when enabled, as in
      S-0109 claim 13. Later start events have no start sound or resource chain reaction.
      Middle events, including the immediate one when due at creation, remain governed by
      S-0109 claims 13, 14 and 25. S-0202 claims 10 and 15 specify creation draws and
      middle-event draw composition during map loading and during play, including
      trigger-created animations and middle events later in playback. That composition
      includes particle creation, smudge choice and placement, and conditional resource-removal
      effects (scenario R14). Event timing, the sprite-derived middle frame and the height
      gate remain S-0109's. Subsequent particle updates remain outside that composition,
      not draw-free. T21 disables particles and smudge effects; their draw counts inherit
      S-0202 claim 15's backing and confidence rather than being established by T21.
    - **Drop-zone reveal.** Creating an animation of the drop-zone type, by this action or otherwise, reveals the area around
      the animation's position for the local player, using the area-reveal radius rule of S-0409 claim
      17. The radius before that rule is *r* quo 256 cells (truncated toward zero), where *r* is the
      rules' drop-zone radius in leptons (Import mapping); a positive result is capped at 10 cells, and
      a result of 0 reveals no cells. A negative result is not specified here; its area-reveal result
      remains open rather than being guessed. Unlike claim 26, the reveal takes place whether or not
      the local player has already had the whole map revealed. How an area is revealed
      belongs to the visibility specs (a pointer only). As in claim 26, the original reveals for each machine's
      local player, which is outside this spec for network games (Terms), and our engine applies the
      reveal to every player house (human or AI), observers included, so the result does not depend on
      the machine (a departure).
    - A play-animation number-slot value that is no registration index of an animation type is unspecified in
      the original, which does not reject it. Our engine gives such an action no effect, and the importer
      warns (a departure). Without a drop-zone animation type in the rules, the drop-zone action is
      not covered here.
    - Backing: `measurable` (scenario T21 for creation, location, radius, creation-time sound
      and resource chain reaction).
      Damage suppression, later playback and middle-event behaviour inherit S-0109's backing.
      The creator's repeat factor, direction and owner attachment are
      `unmeasured`: T21 does not vary loop/reversal settings or test an owner-specific effect. That the drop-zone reveal takes place whether or not
      the whole map is already revealed is `unmeasured`, for the reason given under claim 26
    - Confidence: high for the point, creation, damage suppression and creation-time
      start effects; medium for the drop-zone reveal and its radius, and for the
      point on a sloped cell

## Formulas

Integer game-state calculations use signed 32-bit integers and wrap after each addition,
subtraction and multiplication as in S-0205 claim 1, unless a formula explicitly gives exact
integer arithmetic. Quotients truncate toward zero. This revision's tick examples assume the
tick counter has not wrapped. Overflow of an imported decimal item is still outside this
revision's import coverage.

> **F1. Elapsed-time length.** *D* = the signed 32-bit wrapped value of 15 × *v*, where *v* is the
> event's integer value. Integer arithmetic, with no rounding. Unit: ticks.

> **F2. Random-delay length.** *D* = 15 × (*q* + *r*), where *q* = *v* quo 2 (truncated toward zero)
> and *r* is a ranged draw over the inclusive range between 0 and *v* (from *v* to 0 when *v* is
> negative). When *v* = 0 there is no draw and *r* = 0. The sum and product each wrap to signed
> 32 bits; no floating point is used. Unit: ticks.
> Example: *v* = 10 gives lengths from 75 to 225 ticks.

> **F3. Timer run out.** The timer armed at tick *s* with length *D* has run out in the tick numbered *t*
> when the signed 32-bit wrapped difference *t* − *s* ≥ *D*. Unit: ticks.

> **F4. Game time reached.** *v* ≤ *t* quo 15, with *t* ≥ 0 the tick number and quo the integer quotient
> truncated toward zero. For *v* ≥ 1 this is *t* ≥ 15*v* when that comparison's product is
> interpreted as an exact mathematical integer, not a wrapped timer length. Unit: trigger time units.

> **F5. Letters of a waypoint parameter.** A waypoint parameter is read from its first two characters
> *c*₁ and *c*₂; further characters are ignored. Here “letter” means an ASCII letter A … Z or a … z.
> Let *L*(*c*) be the position of letter *c* in the
> alphabet, A = 0 to Z = 25, upper and lower case alike. The waypoint number is
>
> - *w* = −1 when *c*₁ is not a letter,
> - *w* = *L*(*c*₁) when only *c*₁ is a letter,
> - *w* = 26 × *L*(*c*₁) + *L*(*c*₂) + 26 when both are letters.
>
> So A … Z give waypoint numbers 0 … 25, AA … AZ give 26 … 51 and ZZ gives 701. Exact integers.

> **F6. Reveal centre.** For the waypoint's cell (*x*, *y*) with height level *h*₀, let *h* = *h*₀ + 4 if
> the cell carries a high bridge and *h* = *h*₀ otherwise, and *k* = *h* quo 2 (truncated toward zero).
> The reveal is centred at (256 × (*x* − *k*) + 128, 256 × (*y* − *k*) + 128, 104 × *h*) leptons, that is
> the centre of the cell shifted *k* cells towards the top corner, raised by *h* height levels. Exact
> integers. Unit: leptons.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Trigger time unit | 15 | ticks | 1/60 of a game minute (900 ticks per game minute) |
| Map trigger variables usable by events and actions | 100 | variables | numbers 0 … 99 |
| Latch positions per trigger instance | 32 | event positions | positions from 33 on share (claim 9) |
| Triggering-house value | 9013 | house value | claim 23 |
| No-house value | −1 | house value | claim 23 |
| Player-position values | 4475 … 4482 | house value | positions A … H (S-0009 claims 21 and 29) |
| Event 44 value | house number, from 0 | integer | claim 8; neither a country index nor a player-position value |
| Letters of a waypoint parameter | 2 | characters | F5. They cover waypoint numbers 0 … 701 |
| High-bridge lift | 4 | height levels | F6 and the point of claims 28 and 36 (416 leptons) |
| Height level | 104 | leptons | glossary |
| Event type-name length | 24 | bytes | longer names are cut |
| Action text length | 31 | bytes | after trimming |
| Tag names | 48 | bytes | longer names are cut; a pre-placed object's tag field can match the cut name (claim 2) |
| Reveal-trigger radius, absent-key default | 5 | cells | Import mapping |
| Session computer difficulty, absent-key default | 1 | difficulty position | Import mapping |
| Drop-zone radius, absent-key default | 1024 | leptons | Import mapping; 4 cells for the reveal of claim 36 |
| Drop-zone radius unit | 256 | leptons per cell | claim 36, truncated toward zero |
| Owner-house raise interval | 8 | ticks | claim 35; ticks whose number is a multiple of 8 |
| Airborne-unit cell raise interval | 16 | ticks | claim 8; ticks whose number is a multiple of 16 |
| Polled set | 8, 13, 14, 23, 27, 28, 36, 37, 45, 46, 47, 50, 51, 60, 61 | event codes | all codes, subset or not |
| Code-matched set | 1, 2, 3, 4, 6, 7, 18 … 26, 29, 31, 33, 34, 35, 38 … 44, 48, 49, 50, 53, 54, 59 | event codes | all codes |
| Latchable set | the code-matched set without 1, 6, 31, 44 and 59 | event codes | claim 9 |

## State and ordering

**Repeat types.** What a raise does with a trigger instance that is satisfied in it (claim 8).
"Leaves the raising carrier" means: if the raise came from an object that carries this tag, the
object loses the tag, and if it came from a cell, the cell loses it. The tag's carrier count drops
by one each time.

| Repeat type | Carrier count of the tag | Result |
|---|---|---|
| 0, one-time | any | the trigger fires and is removed. After the raise, the raising carrier loses the tag, every other carrier loses it too, and the tag is deleted at the end of the tick (claim 11) |
| 1, all-carriers | exactly 1 | as one-time |
| 1, all-carriers | 0, or 2 and more | nothing fires. After the raise, the raising carrier loses the tag. The timer is armed again only if an entered-by event of the trigger was judged true in the raise (claim 15) |
| 2, repeating | any | the trigger fires and stays. Its timer is armed again (claim 15) |
| any other value | any | nothing fires (the importer warns) |

**Trigger instance states.**

| State | Event | Result |
|---|---|---|
| enabled | satisfied in a raise | fires as the repeat-type table says |
| enabled | disable action | disabled (timer keeps running) |
| enabled | enable action, difficulty flag set or position outside 0 … 2 | stays enabled, timer re-armed |
| enabled | enable action, difficulty flag clear at a position in 0 … 2 | stays enabled, timer unchanged |
| disabled | enable action, difficulty flag set or position outside 0 … 2 | enabled, timer re-armed |
| disabled | enable action, difficulty flag clear at a position in 0 … 2 | stays disabled |
| disabled | any raise | nothing |
| any | fired in a one-time or all-carriers tag, or its tag deleted | removed: never evaluated again |

**Observable ordering within a tick.**
- Poll-caused effects are observable at the start of the tick's game logic. Their relative outcomes
  follow tag registration order (claims 6 and 7). If a carrier event and a poll occur in the same
  tick, the poll-caused actions are in force before the carrier event's actions.
- A carrier raise (arrival, destruction, capture or bridge collapse) is observable during the game
  logic that causes it; its actions are in force for the remainder of that logic.
- On ticks whose number is a multiple of 8, owner-house effects (claim 35) are observable near the
  end of the tick's game logic, after poll-caused effects and after the objects' updates of that
  tick.
- In one raise, the observable effects of chain members follow reverse chain order (claim 11),
  while each member's actions take effect in action-list order.

**Load-time draw order.** The draws that the scenario random stream gives while a map loads, by
point in loading, in stream order: every draw of a row comes after the draws of the rows above it,
and within a row the draws come in the order listed. It merges S-0009's table "Start position
resolution" with the draws of this spec, of S-0010, S-0109 and S-0202. S-0202 claim 10 owns the
overlay, vein-pattern and animation-creation draws. Together with claim 15 it supplies the
immediate start-event composition, including immediate middle-event particle creation,
smudge selection and conditional resource-removal effects. The whole immediate middle
event's effects and draws precede any start-event resource chain reaction (S-0109 claim 13
and S-0401 claim 20/E17). The reaction's conditional neighbour-offer, child-animation, detonation
and post-detonation terrain-recalculation draws inherit S-0401's composition, not a fixed
additional draw count. These apply within each listed animation-creation group, not as a
separate loading group. Subsequent particle updates are
outside this table's established composition, not draw-free.
A row marked "not established" names a point at which draws may happen that no spec yet lists. Draws at points the table does not
name are not established either. Nothing else in the table fixes an order of internal work.

| Point in loading | Draws, in stream order | Owning spec |
|---|---|---|
| Early phase | list 1's random entries; list 2's random entries; the random-position draws, in house order | S-0009, claim 20 |
| Initial fill | one per visited cell, for a water fill only | S-0009, claim 6 |
| Cell tags, in entry order | for each tag created there: claim 14's draws for each of its trigger instances, in chain order | this spec, claim 34 |
| Terrain stream, tunnels | no draws established | S-0009 |
| Overlay type stream, in stream order | S-0202 claim 10's overlay-placement draws: one raw value per low-bridge piece (3 per cell step of a completed bridge), and the creation and immediate start-event draws of cell animations. Recalculations caused by overlay placement or a bridge piece can start tile animations and take their draws here too. A placement's cell-animation draws precede its own recalculation's tile-animation draws. Placing a vein from this stream takes no draw | S-0202, claim 10 |
| First terrain recalculation pass, in S-0009's visiting order | for each tile animation started (S-0010 claim 19): S-0202 claim 10's creation and immediate start-event draws. A cell that already started a tile animation at an earlier loading recalculation starts no second one in this pass. These animations are removed again before the second pass, but their draws and start-event effects remain | S-0010, claims 12 and 19; S-0202, claim 10; S-0109 |
| Vein candidate rebuilding, in reverse visiting order | every vein is removed first; each candidate that passes the placement test takes the vein-pattern draws of S-0202 claim 10: one raw value for the placed cell and one for each edge neighbour set or re-patterned under that claim. Recalculations that start a tile animation take its creation and immediate start-event draws here too | S-0009, claim 14; S-0202, claim 10 |
| Terrain objects | not established | no spec yet |
| Pre-placed objects: the vehicle entries, then aircraft, then infantry, then buildings, each in entry order | per object: its creation draw (S-0202 claim 10); then, if its tag is created there, claim 14's draws for that tag's instances; then the draws of placing it (not established). A building's upgrades are created after the building's tag and take their own creation draws | S-0202; this spec, claim 34 |
| Smudges | not established | no spec yet |
| Second terrain recalculation pass | as the first pass, for the tile animations it starts, replacing the earlier loading animations | S-0010, claims 12 and 19; S-0202, claim 10; S-0109 |
| Unused-house cleanup, after the second terrain recalculation pass and before the late phase | conditional tile-animation creation and immediate start-event draws from recalculations of occupied ground cells caused by removing objects, including building foundation cells. A cell that already started a tile animation since the second pass takes no new creation draws | S-0009, claim 28; S-0010, claim 19; S-0202, claims 10 and 14 |
| Late phase | list 3's random entries; the re-assignment draws | S-0009, claims 20, 28 and 29 |
| Starting units | the draws of type choice, creation and placement that S-0009 claims 45 and 46 list, in their order, including the raw draw after all houses' starting units | S-0009, claims 45 and 46; S-0202 |
| Starting crates, when crates are on | successive random placements: two ranged requests per candidate attempt, x then y; on success, any overlay-placement draws precede one lifetime ranged request. A full crate-place list takes no requests. Ranged requests are not fixed raw-draw counts: equal bounds and rejection follow S-0202 claim 4 | S-0520 claims 2 and 3 |
| Secret labs | the conditional secret-type choice draws of S-0407 claim 14, in lab creation order, after initial crates and before end-of-loading tags and the game-start recalculation | S-0407, claim 14; S-0202, claim 14 |
| End of loading: remaining tag set-up (Terms) | claim 14's draws for every tag created there, in registration order, and for each tag its instances in chain order | this spec, claim 34 |
| Game-start terrain recalculation pass, in S-0009's visiting order | after starting units, initial crates, secret-lab choices and end-of-loading tags, and before the first tick: for each tile animation this pass starts, its creation and immediate start-event draws. A cell that already started one in the second pass starts no second animation, so takes no further animation-creation draws in this pass | S-0010, claims 12 and 19; S-0011, claim 34; S-0202 claims 10 and 14; S-0109 |
| Resource-glint setup, after game-start terrain recalculation | S-0202 claim 14's conditional choices in visiting order; each successful choice's applicable animation-creation and immediate start-event draws precede the next cell's choice. No configured glint animation means no draws in this group. All precede tick 0 | S-0202, claims 10 and 14 |

S-0202 claim 10 specifies the overlay and vein-pattern draws; its loading-draw table incorporates them.
Claim 14 owns the resource-glint conditions, draw ranges and singleton-frequency case.

## Edge cases and quirks

- **Persistent latch** (`quirks.persistent_event_latch`, YR default: on). A latchable event of a
  repeating tag stays satisfied after its first satisfaction, except for timer-position alias
  clearing (claim 9). *Fixed behaviour:* the
  latches of a trigger instance are cleared each time it fires.
- **Latch aliasing** (`quirks.event_latch_aliasing`, YR default: on). Event positions 33 and later
  share latches with the positions 32 lower, including clearing by aliased timer positions
  (claim 9). *Fixed behaviour:* every position has its own latch.
- **All-carriers tag without a carrier** (`quirks.all_carriers_without_carrier`, YR default: on).
  It never fires (claim 10). *Fixed behaviour:* a carrier count of 0 counts like 1, so the tag fires
  once and is deleted.
- **Removed carrier keeps its tag** (`quirks.removed_carrier_keeps_tag`, YR default: on). An
  object keeps its tag, and its place in the tag's carrier count, when it is destroyed, leaves
  play or is removed, and the place is not given back when it stops existing (claim 2). So an
  all-carriers tag that has lost a carrier in this way does not fire at the raise of its last
  remaining carrier (claim 10). *Fixed behaviour:* an object loses its tag when it is removed, as
  at a detachment, so the count drops by one and a leftover (S-0203 claim 13) carries no tag.
- **Entered-by with a player position** (`quirks.entered_by_country_only`, YR default: on). The
  entered-by value is read only as a country index, so the player-position values never match (claim
  18). *Fixed behaviour:* the value is resolved like an action's house value (claim 23), and the event
  is satisfied when the causing object's owner is that house (including the "References" rule of
  S-0009 claim 29 on maps with 9 or more header pairs). The importer warns about such a value in
  either case.
- **Short bridge-event band** (`quirks.bridge_event_band_short`, YR default: on).
  S-0511 claim 8 owns this switch: the last cell along the band and its cross-cells are
  skipped, and a one-cell band raises nothing (claim 20). Off: the final cell and its
  cross-cells are included.
- **“None” owner.** `quirks.none_owner_is_first_country` (YR default: on). An owner written as `<none>` gives the first country of the country list (claim 5), with an importer warning. Off: `<none>` names no owner country.
- **Exact type names.** `quirks.exact_trigger_type_names` (YR default: on). The type name of the type-exists and type-does-not-exist events is matched case-sensitively, and `none` and `<none>` are ordinary type-name text, not no-type markers (claim 21); the importer warns when the name matches no type. Off: use S-0003 claim 14's case-insensitive type-name matching and no-type markers.

### Fixed input-compatibility rules

The following are fixed compatibility rules of this revision, not optional quirks: the short
**action** reference syntax, absent trigger-field defaults, creation of definitions by unresolved
IDs, the out-of-range difficulty position (claim 4), and the nonpositive type-exists count
(claim 21). They have no switch or alternative mode. Their importer warnings do not change the
result. In particular, no ID-only reference mode or rejection of missing definitions is implied.
The switches above are the optional corrections; crash, hang and unspecified results use the
explicit departures below or in the owning claims.

- **Short action references are registry indexes.** An action reference parameter whose text is
  shorter than 3 characters is read by its decimal integer prefix as a registration index into
  the team, trigger or tag definitions (Import mapping, I2), not as an ID. A prefix of −1 means no
  reference even for longer text. A short text with no digits gives index 0. The importer warns
  about the short-index form. This does not apply to linked triggers, a tag's first trigger or
  cell-tag values: those are ID text even when short. An index outside the registry is
  unspecified in the original. Our engine treats it as empty and warns (a departure).
  - Backing: `measurable` (T24 A for trigger action references and short ID links; T4 for tag
    registration). Short tag action references are `unmeasured`: no action of this subset uses
    the resolved tag, so it has no isolated effect here. The team-reference variant is `unmeasured`: its effects belong to the
    out-of-scope team system; exact registration orders not established here remain open
  - Confidence: high for the syntax, medium for the ID-only contexts
- **Missing disabled field.** A trigger entry that ends before its fourth field starts disabled,
  and one that ends before a difficulty field has that flag clear. The importer warns about a
  trigger entry with fewer than seven fields.
  - Backing: `measurable` (T24 B)
  - Confidence: high
- **References that create definitions.** A linked trigger, a tag's first trigger or an action
  parameter that gives an ID that no trigger definition of the map has creates an empty one: no events,
  no actions, no owner country, not disabled, and all three difficulty flags set. Its instance is
  therefore satisfied in every raise of a tag that holds it (an empty event list, claim 8) and fires
  with no effect. In a one-time tag that firing removes the tag at its first raise of any kind (claim
  10). Other members can still fire during that same raise in claim 11's order; deletion stops
  later raises after that tick, not the other members' evaluation in the current raise.
  In an all-carriers tag it detaches each raising carrier until one is left. A tag reference (or a cell tag) giving an undefined tag ID
  creates a tag definition with an empty chain, which has no effect. A pre-placed object's tag field
  creates nothing (claim 2). The importer warns.
  - Backing: `measurable` (T24 C for the empty trigger and T4 run 4 for the empty tag)
  - Confidence: high for the empty definition's defaults, medium for its firing consequences
- **Unparseable entries.** A trigger list key written as the "none" marker, a trigger entry that ends
  before its second field, a tag entry that ends before its third item, an event or action count
  that is negative or larger than the values that follow, an event that ends before its value item,
  an action that ends before its seventh item, and a chain cycle (claim 1) make the original crash or
  never finish loading. Our engine skips the entry (for a count: reads only the complete events or
  actions; for a short trigger entry: treats the missing fields as absent; for a chain cycle: ends
  the chain before its first repeated member, claim 1), and the importer warns (departures).
  - Backing: `measurable` (T24 D); our safe fallback has no oracle counterpart
  - Confidence: medium

## Import mapping

All entries below are read from the map file. Entry order is the file's entry order after S-0002's
rules for repeated keys. Every value below is split into items at commas, and empty items are
skipped: two commas in a row, or a comma at the start, count as one separator, so the items after an
empty one move up one field. The importer warns about every empty item. An item that is read as an
integer is read by the decimal form of S-0003 claim 3 only: the `$` and `h` hexadecimal forms do not
apply to these items. Overflow of such an item is not covered here. Trigger, event and action values
are limited only by the line length limit of S-0001 claim 3 (their own text limit, in the sense of
S-0003, is at least as long as any value a line can hold, so it never binds), and a tag value has a text limit of 127 bytes. Backing for the tag-value limit is `unmeasured`:
no scenario here distinguishes truncation from a missing final reference; confidence: medium.
The independent trigger/event/action limit is `unmeasured` because the physical-line limit binds
first; confidence: medium. Non-ASCII waypoint text is outside this revision's coverage.
A long event or action list cut by the line limit whose count needs
items beyond the cut is an unparseable entry (Edge cases). The importer warns about every cut value.

**I1. Trigger list** — section `[Triggers]`, one key per trigger definition (the key is its ID).
Value fields, in order:

| Field | Meaning | Form | Absent or empty |
|---|---|---|---|
| 1 | owner country | country name, matched without regard to letter case, or `<none>` in any letter case (claim 5); a bare `none` is looked up as a name like any other text; `<random>`, in any letter case, names no country, even when a country has that name (claim 5) | — (an empty value makes no entry) |
| 2 | linked trigger | trigger ID, or `<none>` / `none` | unparseable entry (edge case); our engine: no link |
| 3 | name | text; it only labels the entry and is never matched, so it has no effect | empty |
| 4 | disabled | integer, non-zero = disabled | disabled (edge case) |
| 5, 6, 7 | difficulty flags for positions 0, 1, 2 | integer, non-zero = set | clear |
| 8 | carry-over flag | integer, non-zero = set | clear |

When any trigger in the entrant's tag chain has the carry-over flag, the tag passes to the
captured building (S-0524 claim 10) or hijacked vehicle (S-0524 claim 11), replacing the
target's previous tag. The capture and hijack cases have different entrant-consumption
notifications (claim 19). Its vehicle-destruction effect is owned by S-0312 claim 29 and
its Import mapping: together with a remembered hijacker type, it suppresses the extra vehicle
raise group, not the common group (claim 19).

**Events** — section `[Events]`, key = trigger ID. Value: *n*, then *n* events. Each event is three
items, its code, its parameter kind and its value, followed by a fourth item, the type name, exactly
when the kind is 2:

| Kind | What the event gets |
|---|---|
| 0 | the value as the event's number |
| 1 | the value read as a team definition name; the event's number is 0 |
| 2 | the value as the number, and the fourth item as the type name (cut to 24 bytes) |
| other | neither a team definition name nor a type name; the event's number is 0 |

Event positions count from 1 in file order, and the claims state every order of events in file
order where the order matters (claims 9, 13 and 14).

**Actions** — section `[Actions]`, key = trigger ID. Value: *n*, then *n* actions of 8 items each:
the code, the parameter kind, the value, four further items (the fourth to seventh) and an eighth
item. Only the eighth item may be missing, and only in the last action.

**I2. Action parameter kinds.** The kind decides how the value (the third item) and the eighth item
are read. The code then uses the slot it needs (table I4). The fourth to seventh items are read as
integers; code 40 uses them for the local-area rectangle (S-0524 claim 8).

| Kind | The value is read as | The eighth item is read as |
|---|---|---|
| 0, 11 | integer into the number slot | kind 0: waypoint letters (F5) into the waypoint slot. Kind 11: integer into a second number slot |
| 1, 5 | team reference (−1 = none) | kind 1: waypoint letters. Kind 5: integer into the number slot |
| 2 | trigger reference (−1 = none) | waypoint letters |
| 3 | tag reference (−1 = none) | waypoint letters |
| 4 | text label: leading and trailing characters with codes 1 to 32 dropped, cut to 31 bytes (−1 = empty) | waypoint letters |
| 6 | speech name, looked up into the number slot | waypoint letters |
| 7 | sound name, looked up into the number slot (unknown = −1) | waypoint letters |
| 8 | music theme name, looked up into the number slot | waypoint letters |
| 9, 10 | type name, cut to 24 bytes | kind 9: integer into the number slot. Kind 10: waypoint letters |

A reference names a team, trigger or tag definition as follows:

| Reference text | Names |
|---|---|
| decimal integer prefix −1, regardless of length | nothing |
| otherwise shorter than 3 characters | the definition with the decimal-prefix registration index (0 if no digits), not that ID (Edge cases, *Short action references are registry indexes*) |
| `<none>` or `none`, in any letter case | nothing |
| any other text | the definition whose ID (key) equals the text without regard to letter case, never one whose name field does. An ID that is not defined creates the definition (Edge cases, *References that create definitions*) |

The linked trigger of I1, the first trigger of a tag and a cell-tag value always use the ID lookup
of the last two rows, regardless of length: `0` names ID `0`, not registration index 0. They do not
use the action parameter's decimal-prefix −1 test. A missing eighth item leaves the waypoint slot
at 0 (waypoint A).

**Tags** — section `[Tags]`, key = tag ID. Value: three items, the repeat type (an integer), the
name (cut to 48 bytes and matched by pre-placed objects' tag fields, claim 2) and the first trigger
(a trigger ID, or `<none>` or `none`). A key whose value is empty is skipped.

**Cell tags, variables, objects.** `[CellTags]` and `[VariableNames]` are read as S-0009 claims 31
and 32 say; a cell tag's value is matched against tag IDs by the ID-only rule of I2. A pre-placed
object's tag field is matched against tag IDs and tag names as claim 2 says (S-0009 claims
34 to 43 own the other object-entry semantics).

**Rules fields.**

| Key | Feeds | Form | Absent |
|---|---|---|---|
| `[General]` `RevealTriggerRadius` | *R* of claim 26 | integer, cells | 5 |
| the combat-damage section, `C4Warhead` | demolition warhead of claim 27 | warhead name | no warhead (claim 27 is then not covered) |
| `[General]` `DropZoneAnim` | drop-zone animation type of claim 36 | animation type name | no type (the drop-zone action is then not covered) |
| `[AudioVisual]` `DropZoneRadius` | *r* of claim 36 | floating-point number of cells; the product with 256, a floating-point result (S-0205 claim 5), is converted to leptons by truncation toward zero (S-0205 claim 7); a value of exactly −1 keeps the default | 1024 leptons |
| `[MultiplayerDialogSettings]` `AIDifficulty` | initial session computer difficulty (claim 4), read from the base rules together with the other game-wide values (claim 4) | integer; 0 … 2 are the difficulty positions, and any other value selects no difficulty flag (claim 4) | 1 |

**I3. Event codes.** "Polled" and "matched" repeat the sets of Constants. Codes not listed are parsed
but not specified in this revision.

| Code | Our name | Value | Polled | Matched | Claim |
|---|---|---|---|---|---|
| 1 | entered-by | country index or −1 | no | yes | 18 |
| 6 | attacked (tagged object) | unused | no | yes | 8 |
| 44 | attacked by house | house number (S-0009 Terms, from 0) of the attacking house | no | yes | 8 |
| 8 | any event | unused | yes | no | 17 |
| 13 | elapsed time | *v* trigger time units | yes | no | 13 |
| 31 | bridge destroyed in tagged cell | unused | no | yes | 20 |
| 36 | variable set | variable number | yes | no | 22 |
| 37 | variable clear | variable number | yes | no | 22 |
| 47 | game time | *v* trigger time units | yes | no | 16 |
| 48 | destroyed by anything, including claim 19's no-hit destructions | unused; no attacker parameter or killing-hit requirement | no | yes (latchable) | 19 |
| 51 | random delay | *v* trigger time units; each arming uses F2, with satisfaction-caused draws before actions (claim 15) | yes | no | 14, 15 |
| 60 | type exists | count, with kind 2 type name | yes | no | 21 |
| 61 | type does not exist | kind 2 type name | yes | no | 21 |

**Companion events outside this subset.** S-0520 claim 7 owns the pickup gates that
raise event 49 on a tagged collector, with that collector as causing object, and set
the scenario's any-crate-picked-up mark associated with event 50. The mark is set
only for a crate-trigger overlay whose pickup continues after the tagged raise.
If that raise kills the collector, the crate remains and the mark is not set by
that pickup. These pickup rules inherit S-0520 claim 7's backing and confidence
(scenario CR5). They do not establish event 50's evaluation at a poll: its presence
in both the polled and code-matched sets remains an open question below. Nor does
the mark alone establish a firing. These pointers do not extend I3's supported subset.

Event code 3 tests the specified house's
lost-to-capture mark under S-0509 claim 11: whether an engineer has ever captured one of
that house's buildings. Nothing else sets the mark, and nothing clears it during the
game. This is a code-matched event (Constants), not a polled condition: an unlatched
event still needs a raise with code 3 under claim 8. The mark's backing and confidence
are S-0509 claim 11's; this pointer does not establish when code-3 raises occur or
extend I3's supported subset. In particular, persistence of the mark alone does not
establish a later firing without a matching raise or an existing latch.

**I4. Action codes.** Slots are those of I2. Rows citing S-0524 are companion pointers:
its Import mapping I1 owns those action items, and its cited claims own the effects.

| Code | Our name | Uses | Claim |
|---|---|---|---|
| 4 | create team | team reference | 30 |
| 5 | destroy team | team reference | 30 |
| 7 | reinforce | team reference | 30 |
| 8 | drop-zone animation at a waypoint | waypoint slot | 36 |
| 11 | text message | text label | 32 |
| 14 | change owner | number slot as house value | 24 |
| 17 | reveal around a waypoint | number slot as waypoint number | 26 |
| 19 | play sound | number slot (sound) | 32 |
| 21 | play speech | number slot (speech) | 32 |
| 23 | timer start | — | 31 |
| 24 | timer stop | — | 31 |
| 25 | timer extend | number slot *v* | 31 |
| 26 | timer shorten | number slot *v* | 31 |
| 27 | timer set | number slot *v* | 31 |
| 32 | destroy attached objects | — | 27 |
| 40 | resize local area | fourth to seventh items: left, top, width, height | S-0524 claim 8 |
| 41 | play an animation at a waypoint | number slot as animation type index, waypoint slot | 36 |
| 42 | explosion at a waypoint | number slot as weapon index, waypoint slot | 28 |
| 53 | enable trigger | trigger reference; an effective self-enable replaces the satisfaction-caused timer length (claim 15) | 25, 15 |
| 54 | disable trigger | trigger reference | 25 |
| 55 | radar event at a waypoint | number slot (event kind), waypoint slot | 32 |
| 56 | set variable | number slot as variable number | 22 |
| 57 | clear variable | number slot as variable number | 22 |
| 63 | demolition blast at a waypoint | number slot as waypoint number | S-0524 claim 5 |
| 66 | large light flash at a waypoint | number slot as waypoint number | S-0524 claim 6 |
| 73 | set base ambient light | number slot as integer hundredths | S-0524 claim 7 |
| 80 | reinforce at a waypoint | team reference, waypoint slot | 30 |
| 90 | lightning strike at a waypoint | waypoint slot | S-0524 claim 2 |
| 95 | nuclear strike at a waypoint | waypoint slot; number slot unused | S-0524 claim 3 |
| 99 | sound at a waypoint | number slot (sound), waypoint slot | 32 |
| 102 | lightning storm at a waypoint | waypoint slot | S-0524 claim 1 |
| 103 | timer text | text label | 31 |
| 108 | create crate at a waypoint | number slot (crate kind), waypoint slot | S-0524 claim 4 |
| 113 | make a house cheer | number slot as house value | 29 |
| 130 | restore starting buildings | number slot as house value | S-0524 claim 9 |
| 135 | preferred target at a waypoint | waypoint slot | 30 |

**Companion-spec pointers outside this subset.** The following codes still use I2's parameter
parsing. These rows only identify the owning spec; they add no behavioural claims here and
are not part of I4's supported subset.

| Code | Our name | Parameter and owning spec |
|---|---|---|
| 1 | win action | number slot; S-0201 scenario T1 part C's fixture assumption only, not a general victory rule specified here |
| 6 | house-wide hunt | number slot as house value (claim 23); garrison evictions hunt without inside fallback, S-0508 claim 13 |
| 33 | one-time superweapon grant | number slot as superweapon type index, trigger owner house; S-0512 claim 12 |
| 34 | permanent superweapon grant | number slot as superweapon type index, trigger owner house; S-0512 claim 13 |
| 100, 117 | plain and freezing radar-movie actions | number slot, ignored in skirmish; S-0201 claim 11 and its Import mapping. Campaign behaviour is out of scope |
| 109 | invulnerability at a waypoint | waypoint slot only, eighth action item; number-slot house and duration values unused; S-0513 claim 22 |
| 110 | pause action | number slot; S-0201 claim 10, including its departures, warnings and Import mapping |
| 111 | evict occupants of first eligible attached building | no parameter slot; without hunting or inside fallback, S-0508 claims 10 to 13 |
| 127 | chrono flash | number slot; S-0201 claim 11, including its quirk rules, warnings and Import mapping |
| 129 | set superweapon charge | number slot as superweapon type index, kind 11 second number slot as percentage, trigger owner house; S-0512 claim 14 |
| 133 | reset superweapon charge | number slot as superweapon type index, trigger owner house; S-0512 claim 15 |

The two particle-system actions are likewise outside I4's supported subset. They are parsed by I2
like any action; their behaviour is specified in S-0616 claim 1 (Trigger actions), and nothing here
adds to it. Their codes and parameter slots are these, in the form of the companion table above:

| Code | Our name | Parameter and owning spec |
|---|---|---|
| 88 | create a particle system of a given type at a waypoint's cell centre | number slot as the particle-system type index, plus the waypoint slot (eighth action item); S-0616 claim 1 |
| 89 | mark every particle system in a waypoint's cell (*marked (particle system)*) | waypoint slot only (eighth action item); number slot unused; S-0616 claim 1 |

**No-hit destruction mapping (claim 19).** These are not new sale or pack-up event codes.

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Map `[Tags]` repeat type, `[Triggers]`, `[Events]` code 48 and `[Actions]` | evaluation and firing for tagged no-hit victims, including hidden slaves and passengers | existing I1 to I4 forms; repeat integers 0, 1 and 2 keep claim 10's meanings | no attached tag: no tag raises; no special generated-slave tag-attachment key is established |
| Destruction notification codes, engine constants rather than imported settings | the ordered common group of claim 19, including hidden slaves | with known attacker: 6, 4, 7, 48, 29; without: 48, 29. Only the event meanings of codes 6 and 48 are in this subset (claims 8 and 19); the extra vehicle group's attacker-dependent raise has code 7 | not configurable; subsequent raises require the victim still to exist and carry the tag |

This mapping inherits claim 19's backing and confidence; it does not establish a means of
attaching a tag to generated slaves. The slave-group and conversion data remain owned by
S-0312 claims 30 and 31, S-0408 claim 10a and S-0415 claim 10.

**Import evidence coverage.** The effects and action slots in I3/I4 inherit the backings of their
named claims. Splitting, missing trigger-field defaults, ID-only links and short action-reference
syntax are `measurable` through T24 (confidence: high, except ID-only contexts: medium).
The waypoint-letter decoding and its missing-item default are `measurable` through T17 D (confidence:
high). The 24-byte event/type-name cut and 31-byte action-text cut are `unmeasured`: no scenario
here contrasts long names/labels with their prefixes (confidence: medium). Parameter kinds unused
by the covered effects or companion claims are `unmeasured`: no scenario here supplies an
observable for them (confidence: medium); their deferred effects remain open. The carry-over
flag's capture and hijack effects inherit S-0524 claims 10 and 11/C9's backing and confidence.
Its vehicle-destruction effect inherits S-0312 claim 29/D20's backing.
The set membership of event codes outside this subset is `unmeasured`: these scenarios do not
exercise the deferred codes (confidence: medium).
Absent rules defaults are `unmeasured`: T16/T21 use the stock imported fields, not absent keys
(confidence: high for the defaults). All importer warnings and safe fallbacks are design rules,
`unmeasured` against the oracle, which has no such reporting/fallback contract (confidence: high).

## Measurement scenarios

**Common setup.** Unless a scenario says otherwise: standard skirmish mode, game speed index 3,
800 × 600 pixels, no engine add-on and no mods (the named test files of a scenario excepted), and the measurement setup of the
glossary, with a purpose-made test map as named test data, removed after the run. The map is small and flat with two
start waypoint markers. It is played as a skirmish of one human player at position A and one computer
player at position B. The computer player is a bystander kept away from the observed area (the
scenario's named objects, cells and counters); apply the glossary's bystander void rule, except
where the scenario deliberately uses its units. The human player's actions are scripted mouse and keyboard input. The recorder is a program that
only reads the running game's state: each tick's number (the first tick is 0), and the owner, health and existence of the objects
the scenario names. Two observation devices are used:
- *Owner marker.* A pre-placed neutral vehicle. A trigger "marks" it with a change-owner action
  (code 14) with house value 4475 or 4476. The marker must carry the trigger's tag, since that action
  changes only carriers of the firing trigger's tag (claim 24), so each owner marker carries the tag
  of the trigger it reports.
- *Counter.* A durable pre-placed neutral building on its own waypoint marker. A trigger "counts"
  with an explosion-at-a-waypoint action (code 42) at that waypoint, naming a weapon whose single hit
  leaves the building standing. Each firing shows as one health drop in the tick of the firing, and
  several firings in one tick show as one drop that many times the size of a single firing's drop,
  so the number of firings in a tick is the tick's health loss divided by a single firing's loss.
  Calibrate that loss with one firing first; select a weapon/counter pairing with a constant
  positive loss and no healing, repair or variable defensive state, with enough health for the
  run. If necessary use a named map override raising only the counter type's strength. Separate
  counters beyond the weapon's spread. A counter destroyed or damaged by another source makes
  the run void. Health-loss division is exact integer arithmetic; a nonintegral ratio is a
  failed calibration, not a rounded firing count.
- *Owner countries.* Unless a scenario says otherwise, every test trigger's owner country is one that
  no house plays (a playable country that neither player chose), so that no owner-house raise
  (claim 35) adds firings. Scenarios T20 and T22 test that raise. House values in actions are unaffected.
- *Naming.* Wherever a scenario says that an action, a linked trigger, a tag or a carrier "names" a
  trigger or tag definition, it gives that definition's ID (Import mapping, I2), except where the
  scenario says it gives the name field (T6, variants E and F).

Runs: 1 per variant unless stated. T11, T17 C and T23 use the random-state recorder for
stream-dependent results. When a scenario recovers a loading draw sequence, record the seed and
stream state during loading as well as before tick 0; take paired control runs with one test group
omitted. Compare each run with the sequence predicted from its own recorded seed, never assume
that two new games share a seed. A log too coarse to separate the relevant groups is inconclusive,
not a passing draw-order measurement.

**Scenario T1 (claim 1).** *Setup:* trigger definitions P and Q, Q linked from P, both with event
any-event and a count action, and U, a trigger definition with the same event and a count action that
no tag's chain reaches. Tag X (one-time) names P. Variant: a second tag Y (one-time) names Q as its
first trigger. *Observable:* firings per tick on the counter. *Expected:* two firings in tick 0 (P
and Q), three in the variant (P, Q in X, and Q in Y), none from U and none in later ticks.
*Falsified if:* any other number of firings. Independent-instance variant: replace Q's any-event
with elapsed time 2, make X and Y repeating, and re-enable Q with a separate trigger in tick 75.
*Expected:* Q in both tags fires in ticks 30, 60 and 105, giving two counter increments at each.
P still fires every poll. Use a separate counter for Q. *Falsified if:* Q fires only once at a
shared deadline, or one instance's re-arming postpones the other's firing. Variant with a cycle
is T24 D, not part of this successful-load setup.

**Scenario T2 (claims 3, 4, 25).** *Setup:* three one-time triggers with event any-event, each with
exactly one difficulty flag set (fifth, sixth, seventh field), each marking its own owner marker, and
a fourth with all three flags set and the disabled flag set, also marking its own. Run once with the
stock lobby defaults, and again with each setting of a lobby computer-difficulty control if the lobby
offers one. Variant B: a fifth trigger, with all three difficulty flags set, the disabled flag
clear and elapsed time 2, has two enable actions: one names the fourth trigger, the other names the
single-flag trigger whose flag is the fifth field if the session's position is 1 or 2, and the one
whose flag is the sixth field if it is 0 (so its flag is clear at the session's position). Variant
C, for a setting outside 0 … 2: as the main run, with named test data whose base rules copy sets
the multiplayer-dialog difficulty field to 3, and the lobby left untouched. Also include variant B's fifth trigger and its enable actions. *Observable:* which
markers change owner, and when. *Expected:* in tick 0 only the trigger whose flag matches the
session's position fires; the position is the base rules' multiplayer-dialog difficulty value
unless the lobby changed it. The fourth never fires. In variant B the fourth fires in tick 30 or 31
(the enable in tick 30, the firing in the same tick's poll or the next one, by registration order),
and the named single-flag trigger never fires. In variant C all three single-flag triggers fire in
tick 0 and the fourth fires in tick 30 or 31 after the enable action. *Falsified if:* not exactly one of the three fires in the main
run, the position does not follow the setting, or variant B or C differs. *Inconclusive if* in
variant C the lobby shows a computer-difficulty control that replaces the value 3.

**Scenario T3 (claims 18, 23).** Arrival-timing discrimination uses S-0207 D24
(driving and own-locomotor teleporting) and S-0208 W14 (ordinary walking), alongside
this scenario's event eligibility and house matching. *Setup:* a skirmish of one human player and one computer player
who both play the same country C, on a map with a cell carrying a one-time cell tag whose trigger has
an entered-by event and a count action. The cell is walled off from the computer player's start so
that none of its units can reach it (the human's unit is pre-placed next to it). Variants of the
event's value: the registration index of C, −1, 4475. Run X: the human starts in the position that is
later in house order (S-0009 claim 19). Run Y: the human starts in the earlier position. In each run
the human's unit moves into the cell. *Observable:* whether and in which tick the counter drops
(repeated over the three values and both runs). *Expected:* value C: no drop in run X (the human's
house is not the first house of country C), a drop with the entry in run Y. −1: a drop with the entry
in both runs. 4475: never a drop. *Falsified if:* value C drops in run X or does not drop in run Y, −1
does not drop in either run, or 4475 drops. *Inconclusive if* a computer-player unit reaches the cell.

T3 variant B (bridge): use a one-time cell tag with entered-by −1 on a high bridge cell, and a
human vehicle whose route can cross that cell's ground layer and, separately, its deck. Give the
trigger a count action and a change-owner action with house value 9013 on an attached neutral owner
marker. *Procedure:* traverse under the bridge, then cross the deck. *Observable:* counter and
marker owner against the arrival ticks. *Expected:* no firing below; one firing on deck entry,
and the marker becomes the arriving unit's owner's. *Falsified if:* a firing below, none on the
deck, or another owner. T3 variant C (capture): put that tag on a capturable neutral building
instead of a cell, and order a human engineer to capture it. *Expected:* one count in the capture
tick and the marker becomes the engineer's owner's. *Falsified if:* no count, a count before entry,
or another marker owner. Other entry abilities are not tested here.

**Scenario T4 (claims 6, 7).** *Setup:* two one-time polled tags G and H. G's trigger has elapsed
time 1 and sets map trigger variable 1. H's trigger has elapsed time 1 and variable-clear 1, and sets
variable 2. A repeating polled trigger R with variable-set 2 counts. Run 1: the tag list lists G before
H. Run 2: H before G. Run 3: H before G in the tag list, but a trigger definition earlier in the
trigger list, member of no chain, has an action of kind 3 (tag reference) naming G, so G is registered
first. *Observable:* counter drops. *Expected:* runs 1 and 3: in tick 15 G acts before H is evaluated,
so H never fires and the counter never drops. Run 2: H acts in tick 15 before G, variable 2 is set, and
the counter drops in every tick from tick 15 (or 16, if R is registered before H). *Falsified if:* any
run differs.
Run 4 (a cell tag naming an undefined tag, S-0009 claim 31): as run 1, plus a cell-tag section with
the entry for cell (15, 10) keyed "10015" naming a tag ID that the map does not define, and then the
entry keyed "010015" naming a defined one-time tag whose trigger (entered-by, −1) changes the owner
of an owner marker that carries that tag. A unit then enters the cell. *Expected:* the marker never
changes owner, because the first entry attached an empty tag that occupies the cell. *Falsified if:*
the marker changes owner.

**Scenario T5 (claims 8, 15, 17, 32).** *Setup:* (a) a one-time trigger with event any-event and a count
action, plus a text message, a sound, a speech and a radar event (checked by eye). (b) A repeating
trigger with event any-event and a count action on a second counter. (c) A one-time cell-tag trigger
with events entered-by (−1) and elapsed time 20 and a count action on a third counter, with the human
moving a unit into the cell once between ticks 100 and 200 and once more after tick 300 but less
than 300 ticks after the first entry. (d) A one-time trigger with an empty event list and a count
action on a fourth counter, on a tag carried by a pre-placed neutral vehicle that the human attacks
several times with a weapon too weak to destroy it. (e) A one-time trigger with elapsed time 1 and two sound
at a waypoint actions: one at a waypoint marker on the cell of a pre-placed neutral building, the
other at a waypoint marker on an empty cell, both naming a looping sound of the kind that stock
maps give buildings with this action (chosen from the game's sound list; no value here).
(f) A one-time trigger with elapsed time 1 and two radar events of the same kind, at one waypoint marker.
In part (e), another trigger at elapsed time 3 gives the building an unknown sound name.
*Observable:* firings per tick on each counter, and what is seen and heard in parts (a), (e) and (f).
*Expected:* (a) one firing in tick 0, and the four presentation actions are seen and heard at the
start. (b) one firing in every tick. (c) one firing, at the second entry (the first entry is judged
true but the trigger is not satisfied, so it does not re-arm the timer, claim 15). (d) exactly one
firing, in the tick of the first hit (the hit raises the tag at least once, claim 8), and none at
later hits, since the vehicle then carries no tag. (e) the building keeps playing the sound as its
ambient sound while the view is near it, then stops it at tick 45 when the unknown sound is
assigned; the sound at the empty cell is played once. (f) one
radar event is shown, not two. *Falsified if:* any differs. In part (c), in particular, no firing at
the second entry (which would mean that the first entry re-armed the timer).

**Scenario T6 (claims 2, 10).** *Setup:* three neutral vehicles carry the same tag, whose trigger
(destroyed-by-anything) counts. The human destroys them one by one with single shots of a weapon
that kills each in one hit. Variants: repeat type 1, 0, and 2. Variant D: an all-carriers tag
carried by nothing, with event any-event and a count action. Variant E: as type 0, but the tag's
name field differs from its ID, and the vehicles' tag fields give the name, in another letter case.
Variant F: as type 0, but the tag's name is 50 characters long; the three vehicles' tag fields give
its first 48 characters, and a fourth such vehicle, whose tag field gives all 50 characters, is
destroyed first. *Observable:* counter drops after each destruction. *Expected:* type 1: one drop,
after the third destruction. Type 0: one drop, after the first; the other two no longer carry the
tag. Type 2: a drop after each destruction (and more, see T7). D: no drop. E: as type 0. F: no drop
at the fourth vehicle's destruction (it carries no tag), then as type 0. Variant G: repeat value 3,
and no firing at any destruction. Variant H: an all-carriers tag carried by two neutral vehicles A
and B, whose trigger has a game-time event (claim 16) as its first event and destroyed-by-anything
as its second, and counts. In run 1 the human destroys A in a tick before the game-time event is
satisfied and B in a tick after it, and in run 2 both in ticks after it. Expected: in run 1 no drop,
and in run 2 one drop, at B's destruction. Run 1 is the run that tells the two settings of
`quirks.removed_carrier_keeps_tag` apart: with the switch off, B's raise finds one carrier and the
counter drops. In run 2, A's satisfied raise detaches A under either setting. *Falsified if:* any
differs.

**Scenario T7 (claims 9, 19).** *Setup:* two neutral vehicles V1 and V2 carry one repeating tag whose
only trigger has the single event destroyed-by-anything and a count action. The human destroys V1, then
hits V2 several times with a weapon too weak to destroy it. *Observable:* counter drops, against the
ticks of the hits. Use a fresh tag for each run and a count action that leaves both vehicles
and their tag alone. *Expected:* seven firings within the tick of V1's destruction by the
human's attacker, then at least one firing in the tick of each hit on V2 (the latch fires in
every hit-related raise). A no-attacker variant uses a separate polled trigger's explosion
at a waypoint (claim 28), lethal to V1 and far enough from V2 not to hit it: four firings.
The explosion does not change V1's repeating counter tag; no counter action kills a carrier.
S-0312 D20 supplies the common-group-only controls, including the remembered-hijacker plus
carry-over-flag case: two firings, with or without an attacker. With
`quirks.vehicle_destruction_extra_raises` off, which only our engine offers, the normal
vehicle likewise gives two firings. Repeat type 0 instead gives one firing at V1's destruction
and none afterwards. With `quirks.persistent_event_latch` off and the extra vehicle group
on, which only our engine offers, the normal repeating vehicle gives two firings at its death
and none at the later nonlethal hits. With both switches off it gives one death firing and
none at those hits. Variant C (latch aliasing): the trigger instead has 33 events, on the same repeating
tag: position 1 destroyed-by-anything, positions 2 to 32 any-event, and position 33 type-exists
naming a type name that no type has (which is never judged true). *Expected* in variant C: no
firing before V1's destruction; at least one firing in the tick of V1's destruction and in every
later tick, because position 33 shares the latch that position 1 gets (the tag is polled, since
its chain contains any-event). With `quirks.event_latch_aliasing` off, which only our engine offers,
no firing at all. *Falsified if:* in the main run V1's destruction brings other than seven
firings, the no-attacker run brings other than four, a common-only control brings other than
two, the ticks of later hits on V2 bring no drops, or a variant differs from its expectation.
Timing, deferred deaths and retained infantry tags are tested by S-0312 D20, not by this
vehicle counter alone.
Variant D (timer alias clearing): use position 1 destroyed-by-anything, positions 2 … 32
any-event, and position 33 elapsed-time 1. Destroy V1 after tick 15, then hit V2 without killing
it. *Expected:* one firing on V1's destruction, not two; no firings at later polls or V2 hits.
Re-arming clears the timer's shared latch. With aliasing off and persistent latching on, the
destruction event stays latched: one firing at the destruction, then one every 15 ticks from
that firing as the independent timer expires. *Falsified if:* the oracle shows two destruction
firings or later firings under the original aliasing behaviour.

**Scenario T8 (claim 11).** *Setup:* an owner marker carries a one-time tag whose chain is K1 → K2,
both with elapsed time 1. K1 changes the marker's owner with house value 4475, K2 with 4476.
Variant B: K2 also has, after its change-owner action, a disable action naming K1. *Observable:* the
marker's owner at the end of tick 15. *Expected:* K2 acts first and K1 last, so the owner is the
human player (4475). Variant B: K1 is judged after K2's actions have taken effect, so it is disabled
and does not fire, and the owner is the computer player (4476). *Falsified if:* the owner is the
computer player in the main run, or the human player in variant B. Variant C (action list):
K1 instead has a no-effect change-owner action with value −1, then values 4476 and 4475, in
that order. *Expected:* human owner, because the last action wins and the earlier no-effect
action stops nothing. *Falsified if:* another owner.

**Scenario T9 (claims 12, 27).** *Setup:* a neutral vehicle V carries a repeating tag with two
triggers: A (elapsed time 2, destroy attached objects) and B (destroyed-by-anything, count). Control
run: the tag holds only B, and a separate polled tag fires an explosion at V's waypoint with a weapon
that destroys V in tick 30. *Observable:* V's destruction tick and counter drops. *Expected:* main run:
V is destroyed in tick 30 and the counter does not drop (B's raise happened while the tag was being
raised). Control: V is destroyed in tick 30 and the counter drops in tick 30. Variant: V is a vehicle
whose armour class the demolition warhead does no damage to (found in the rules, or made so by the test map's override section), in the
main run's setup. Expected: still destroyed in tick 30. *Falsified if:* the main run drops, the control does not, or V
survives in the main run, the control or that variant. Variant C (survivor), on a test map whose override section
makes the demolition warhead one of the radiation kind and V's type immune to radiation: the
main run's setup. *Observable:* the tick counter, read by the recorder, and the game's response to
input. *Expected:* from tick 30 the tick counter stops advancing and the game stops responding
(the recorder shows V's health unchanged, above 0). *Falsified if:* the game goes on past tick 30,
or V is destroyed. Variant D (berserk protection): use the main-run setup, but make the demolition warhead cause berserk and ensure V is not immune to radiation, psychic or poison damage. *Observable:* V's health and the tick counter and response to input after tick 30. *Expected:* V takes no health damage; as a survivor, it remains above 0 health and the original's action hangs from tick 30. *Falsified if:* V loses health or the game continues responding after tick 30.

T9 variant E (cell attempts): on a small test map, put a cell tag on an intact high-bridge
section, with entered-by −1 and the destroy-attached-objects action. No object carries
that tag. Use stock data, no engine add-on and no mods (the named test files of a scenario excepted), one human and a bystander computer player
kept outside the bridge and approach cells, game speed 2, and the bridge-destruction option
off. Move a human vehicle onto the tagged deck cell. Record its arrival tick and the
section's state with video and a read-only recorder; repeat three fresh runs.
*Expected:* the section is lost in the arrival action despite the option being off: its
first attempt damages it and the next succeeds under S-0511 claim 3. As a no-cell control,
use bridge-destroyed instead of entered-by on a tagged cell in the band that retains its
section, then lose a different section with the option on. The control's action changes
no additional section. *Falsified if:* qualified arrival leaves the attempted section
intact or only damaged, or the no-cell control starts further bridge attempts. Record
bridge state immediately before and after the action to exclude notice and span-check
losses. S-0511 B10 owns the paired gate/control test.

**Scenario T10 (claims 8, 13, 15, 16, 25).** Variants, each with its own counter:
- *A:* one-time polled triggers with elapsed time 0, 1 and 7. Expected drops in ticks 0, 15 and 105.
- *B:* a repeating trigger with elapsed time 3. Expected drops in ticks 45, 90, 135 and so on.
- *C:* a trigger disabled at start with elapsed time 4, enabled by another trigger's enable action in
  tick 150 (that one has elapsed time 10). Expected: a drop in tick 210, none in tick 150.
- *D:* a repeating trigger with events variable-set (variable 5) and elapsed time 2, with variable 5
  set by another trigger in tick 75 (elapsed time 5). Expected: drops in ticks 105, 135, 165 and so on.
- *E:* a trigger disabled at start with game time 4, enabled in tick 30 by another trigger's enable
  action. Expected: a drop in tick 60, which an elapsed time 4 would have put at tick 90.
- *F:* a one-time trigger with two timer events, elapsed time 2 written first and elapsed time 6
  second. Expected: a drop in tick 30.

*Observable:* ticks of the counter drops. *Falsified if:* any tick differs. C, D and E have
positive delays after the enabling or variable change, so registration order permits no extra
tick of slack at their deadlines.

T10 variant G (already enabled): a repeating elapsed-time 4 trigger counts. A separate one-time
trigger enables it at tick 30. *Expected:* no count at tick 60; first count at tick 90, then every
60 ticks. *Falsified if:* enabling did not restart the timer. Variant H (wrapped length): a
one-time elapsed-time trigger has value 286331154. *Expected:* the product with 15 wraps to
14 ticks, so one count in tick 14. *Falsified if:* another firing tick or no firing by tick 15.
Variant I: one-time game-time events with values 0 and −1, each on its own counter.
*Expected:* both count at tick 0. *Falsified if:* either starts later or never fires.
Variant J: repeat D, adding another set-variable-5 action at tick 90, when variable 5 is already
set. *Expected:* first count remains tick 105, not 120: an unchanged assignment does not re-arm.
*Falsified if:* the extra assignment postpones the count.

**Scenario T11 (claim 14).** Variant A, statistical. *Setup:* a one-time polled trigger with random
delay *v* = 10 and another with *v* = 1, each with its own counter. 220 runs, a new game each run.
*Observable:* the tick of each drop. *Expected:* for *v* = 10 every tick lies in 75 … 225 and is a
multiple of 15, and the 11 possible values each occur with frequency 1/11, which a chi-square test at
the 1 % level does not reject. For *v* = 1 the ticks are 0 and 15, each about half the time (binomial
test at the 1 % level). *Falsified if:* a tick outside the range or not a multiple of 15, or a test
rejects.
Variant B, with the random-state recorder; one run per case. *Setup:* a one-time polled trigger with
two random-delay events (values 10 and 7) and one with a random-delay event of value 0, each tag
carried by a pre-placed neutral vehicle so that it is created during loading, plus a trigger with no
random-delay event. Each tested trigger has its own count action and counter. A second case adds an elapsed-time event before the random-delay events in the
event list. *Observable:* the recorder's log of the stream state before the first tick and in every
tick, from which the number and values of the draws taken by each trigger instance's arming are
recovered by replaying the stream's recurrence (S-0202) from the logged state, and the tick of each
counter drop. *Expected:* the arming takes one draw per random-delay event of non-zero value, in
reverse order of the event list, and none for the value 0 or for the trigger without such an event.
The delay is the claim 14 formula applied to the draw of the event that sets the timer, as the tick
of the drop shows. *Falsified if:* another number or order of draws, or a delay taken from the other
event's draw. Variant C: use random-delay values −3 and 0 on separate one-time triggers,
created at pre-placed vehicles. Read each trigger timer's initial length with the read-only
recorder as well as its stream state. *Expected:* −3 uses one ranged draw from −3 … 0 and
length 15 × (−1 + the drawn value), with lengths −60, −45, −30 or −15 ticks; 0 uses no draw
and length 0. Both fire at tick 0. *Falsified if:* another draw range/count, length or firing tick.

Variant D (claim 15; Q-0269), with the random-state recorder. *Setup:* a repeating polled
trigger whose only event is random delay with value 10, with one instance and an unplayed
owner country. Its actions are an explosion at a waypoint and, in the self-enable case,
an enable action naming its own definition. All difficulty flags are set. Use the common
calibrated counter. Select stock weapon/warhead/animation data whose explosion takes at least
one established scenario draw under claim 28 and its companion specs; verify that in a
one-time explosion control. Keep unrelated objects out of the explosion's area.
*Procedure:* record through three firings in each of two cases, without and with self-enable.
Repeat each case until 10 incoming recorded states distinguish the hypothesised draw order.
Compare each run against its own seed. Log the firing ticks and stream states finely enough to
separate each firing's draw groups. Account for ranged-draw rejection under S-0202, rather
than treating a ranged draw as necessarily one raw value. A coarse log or a control explosion
with no established draw is inconclusive.
*Observable:* ordered draw groups and the next firing tick. *Expected:* without self-enable,
the satisfaction arming group precedes all explosion groups, and the next firing is D ticks
later, using that arming's F2 length. With self-enable the order is satisfaction arming,
explosion groups, enable arming; the next firing uses the enable arming's F2 length, not the
satisfaction length. Both arming groups are present. A further case puts the self-enable
before the explosion in the action list: satisfaction arming, enable arming, explosion groups,
with the enable length still determining the next firing. Use recorded states with unequal
arming lengths to distinguish replacement. *Falsified if:* action draws precede satisfaction
arming, either applicable arming is omitted, or the next firing uses the superseded length.
All lengths here are positive multiples of 15 ticks, so no zero-delay same-tick raise obscures
the deadline. No extra data field is needed: I3's random-delay row and I4's explosion and
enable rows supply the setup.

**Scenario T12 (claim 22).** *Setup:* map trigger variable 3 starts clear (the map gives it initial
state 0). Trigger S (one-time, elapsed time 2) sets it, trigger C (one-time, elapsed time 6) clears
it, trigger W
(repeating, variable-set 3) counts, trigger Z (one-time, variable-clear 3) counts on a second counter.
Variant: S sets variable 150 instead. *Observable:* counter drops per tick. *Expected:* Z in tick 0. W in
every tick from tick 30 (or 31, if W's tag is polled before S's) to tick 89 (or 90, if W's tag is polled
before C's). Variant: W never. *Falsified if:* W drops only once, outside that range, or in the variant.

**Scenario T13 (claim 20).** *Setup:* small test maps with a low bridge and a high bridge,
separately, along each axis. Tag the centre row and four neighbouring rows with distinct
one-time bridge-destroyed tags and calibrated count actions; keep counters outside all
collapse and weapon effects. Use stock data, no engine add-on and no mods (the named test files of a scenario excepted), game speed 2,
one human and a bystander computer player kept outside the bridge and counters, with
the glossary's void rule. Turn bridge destruction on. *Procedure:* destroy one interior
low strip, or one high section. Repeat each layout three times with scripted input,
video and a read-only state recorder. *Observable:* each counter's firing tick and
position, compared with changed bridge cells. *Expected:* in the destruction tick,
counts cover claim 20's four cross-cells from the first band cell through the cell before
its last endpoint, including tagged cells that stay intact. The high cell immediately
before the far piece counts; the far piece does not. The low run's last strip does not
count. *Falsified if:* a skipped endpoint counts, a qualifying cell does not, or counts
are limited to cells that actually collapse.

*Repeating variant:* give several qualifying cells one repeating tag, with count actions
that leave the bridge and tag unchanged. Record firings within the destruction tick.
For a destroyed low strip expect one firing per qualifying tagged cell. For a high span
that loses *n* isolated sections, expect *n* + 1 such groups, before, between and after
isolated losses, including one group when *n* = 0. A different count or interleaving
falsifies the claim. Use S-0511 B14's isolated-run fixtures; B5 also tests cross-cell order
and coincident endpoints, for which a one-cell band gives no firing. A one-time shared
tag is not suitable for measuring repeated raises because its first firing detaches it.

**Scenario T14 (claim 21).** *Setup:* a repeating polled trigger with type-exists (a vehicle type name,
count 2) and one with type-does-not-exist (same name), each with its own counter. The human builds two
vehicles of the type, loads one into a transport, then loses both. Variant B: the name in different
letter case. Variant C: type-exists only, with count 0 and, in a separate run, count −1, naming a vehicle type of which no object
exists during the run. Variant D: type-exists only, with count 1, naming an infantry type; the human
has exactly one infantry unit of that type and garrisons it in a pre-placed neutral building, then
takes it out again. *Observable:* the ticks in which each counter drops, against the number of
such vehicles. *Expected:* type-exists drops in every tick with at least 2 vehicles of the type, the
one in the transport included. Type-does-not-exist drops in every tick with none. In variant B neither
ever drops, provided no registered type has that differently cased exact name. In variant C the counter drops in every tick from tick 0, because the common setup's
pre-placed objects exist. In variant D the counter drops in every tick, including those in which
the unit is inside the building. *Falsified if:* any differs.

**Scenario T15 (claims 23, 24).** *Setup:* owner markers, each carrying its own one-time tag whose
trigger (elapsed time 20) changes its owner with one of the house values 4475, 4476, 4477 (no player at
position C), −1, 9013 (no triggering house yet) and the registration index of the computer player's
country. A second vehicle, pre-placed for the human player (owner given by the human placeholder name,
S-0009 claim 27), also carries the 4476 tag, and the human loads it into a transport before tick 300.
*Observable:* owners after tick 300. *Expected:* 4475: the human player. 4476: the computer player
for the marker, while the transported vehicle stays the human player's. 4477, −1 and 9013: unchanged. The country index: the computer
player. Variant: both players play the same country, and the country-index marker uses that country's
index. Expected: that marker goes to the player earlier in house order (S-0009 claim 19). Variant C,
player positions on a map with 9 or more header pairs: the map's header count N_h is 9 and its pair 9
is (1, 0), both players are on "random" (the human is house 0 and the computer house 1, as in S-0009's
scenario M13b), and only the 4475 and 4476 markers are used. Expected: 4475 gives the marker to the
computer player and 4476 to the human, because the marks of positions A and B are 1 and 0. *Falsified
if:* any differs.

**Scenario T16 (claim 26).** *Setup:* a polled trigger (elapsed time 1) with reveal around a waypoint on
a flat cell of height level 0, and a second one on a cell of height level 4. Rules radius at its stock
value. *Observable:* screenshots of the shroud after tick 15, and the stock radius value from the rules.
*Expected:* a revealed disc of that radius around the first cell's centre, and around the point two cells
towards the top corner of the second cell. *Falsified if:* the centre or the radius differs.
(The condition that the whole map is not already revealed is not tested, claim 26.)

**Scenario T17 (claims 28, 33).** *Setup:* a polled trigger with an explosion at a waypoint naming a
weapon's registration index, with a neutral vehicle on the waypoint's cell. *Observable:* the
vehicle's health loss, compared with the named weapon's damage against its armour by S-0303's
area-damage rules and S-0302's damage resolution, including its distance from the detonation
point. *Expected:* the loss matches that one area-damage application.

Variant B uses S-0524 scenario C11's off-map-point fixture, adapted to this explosion action:
start a fresh oracle process per run, leave an in-range waypoint marker undefined, and record
the requested detonation point and health with a read-only recorder. Use named test-map data
giving the selected weapon's warhead enough spread to reach neutral vehicles on the nearest
legal cells. Repeat with those legal corner cells at two different height levels, and compare
an explicit non-legal waypoint near the same corner. With a blank stand-in, expect the
undefined marker's point to be (128, 128, 0) integer leptons regardless of those legal terrain
levels; S-0009 claim 44 owns the stand-in's original shared state and our blank-cell departure.
Health loss must follow the ordinary area-damage calculation from that point, not from a
snapped legal cell. Cell (0, 0) is never legal (S-0009 claims 1 and 2), so no vehicle is placed
there. A different point, early rejection solely because the marker is undefined, or health
loss inconsistent with that point falsifies this variant. Altered stand-in state remains
exploratory rather than a second stable-height prediction.

Variant C, with the random-state recorder: a weapon whose warhead selects a damage
animation for the cell's land and whose area damage takes a draw (S-0303); the recorder's log shows
the draws of the animation's creation before those of the damage. *Falsified if:* no damage, a loss
that does not match the weapon, or in variant C the damage's draws before the animation's.

T17 variant D (waypoint syntax): place durable neutral counters at distinct legal waypoint
markers numbered 0, 25, 26, 51 and 701, away from each other. One run per text, using a one-time
explosion action: `A`, `Z`, `AA`, `AZ`, `ZZ`, `aaZ`, `A?` and a last action whose eighth item
is absent. *Observable:* which counter loses health. *Expected:* respectively markers 0, 25, 26,
51, 701, 26, 0 and 0; every other counter stays unchanged. *Falsified if:* another counter changes
or none does. Each counter's loss is compared with a control action at that same marker.

**Scenario T18 (claim 29).** *Setup:* computer-player infantry firing at a target. A polled trigger
makes the computer player's country cheer. *Observable:* the infantry's sequence and the tick of its next
shot. *Expected:* the cheer starts in the trigger's tick, and the next shot comes only after it ends.
*Falsified if:* no cheer, or a shot during the cheer.

T18 also records whether the cheer sound is heard. *Falsified if:* the sound is absent despite
working audio and a successful control cheer. Other object kinds are not tested by this scenario.

**Scenario T19 (claim 31).** Read the mission timer's remaining ticks with the recorder as well
as recording its on-screen label: the display alone need not resolve a single tick. *Setup:* a polled trigger with timer-set *v* = 4 and elapsed time 1, and a
second polled trigger with timer-extend 2 and elapsed time 2. *Observable:* the on-screen timer against
the tick count. *Expected:* the timer reaches 0 in tick 15 + 60 + 30 = 105 and stops there. *Falsified
if:* it reaches 0 in another tick. Variants, each with the timer-set trigger (tick 15) and the actions named, but without the
timer-extend trigger:
- *B (stop and start):* a polled trigger with timer-stop at elapsed time 3 (tick 45) and one with
  timer-start at elapsed time 5 (tick 75). *Expected:* the timer stands at 30 ticks remaining from
  tick 45 until tick 75, then runs on and reaches 0 in tick 105. *Falsified if:* it keeps running
  between ticks 45 and 75, or restarts from another length.
- *C (shorten):* a trigger with timer-shorten 1 at elapsed time 2 (tick 30). *Expected:* the timer
  reaches 0 in tick 15 + 60 − 15 = 60. *Falsified if:* another tick.
- *D (text):* a trigger with timer-text with a given text at elapsed time 1 and another with
  timer-text with an empty text at elapsed time 2, with the timer started by the timer-set trigger of the main setup.
  *Expected:* the on-screen label is the given text after tick 15 and the default label (the one
  the timer-set action gives) after tick 30. *Falsified if:* another label.

**Scenario T20 (claims 17, 35).** *Setup:* the common setup, but with a repeating trigger R1 whose
only event is any-event and which counts, owned by the human player's country, and a second repeating
trigger R2, identical but on its own counter and owned by a country no house plays. Variant B: R1's
tag has as its first trigger a trigger owned by a country no house plays (with no events that can be
satisfied, for example a type-exists event naming a type name no type has), and R1 linked from it.
*Observable:* firings per tick on each counter, over ticks 0 … 40. *Expected:* R2: one firing in every
tick. R1: one firing in every tick, and two in ticks 0, 8, 16, 24, 32 and 40. Variant B: R1 one firing
in every tick (its tag is assigned to no house, because the first trigger's owner country is not
played). *Falsified if:* R1 shows no second firing in ticks that are multiples of 8, a second firing in
other ticks, or R2 or variant B shows a second firing.

T20 variant C (ordering): two repeating any-event tags G then H in registration order; G sets
variable 9 and H clears it. Both own no other actions and the variable starts clear. Run 1: both
triggers are owned by the human's country. Run 2: G belongs to the house earlier in house order,
H to the later one; choose distinct countries. *Observable:* variable 9, read by the recorder at
the end of tick 0. *Expected:* run 1 set, because the house raises H before G, reversing the poll
order; run 2 clear, because the later house's H acts last. *Falsified if:* run 1 clear or run 2
set. The run is void if another action changes variable 9. Continued raises after defeat and
under-attack notifications are not tested by T20.

**Scenario T21 (claim 36).** *Setup:* named test data as in S-0109's common setup: a test
expansion archive whose art database copy adds an animation type R with a resource chain reaction
and a start sound, no damage, no particles, and neither scorch nor crater effects. A polled trigger (elapsed time 1) plays R at a waypoint marker
whose cell holds resource, and a second polled trigger (elapsed time 2) uses the drop-zone action at
a waypoint marker in shroud, far from both players' units. A third trigger plays R at a waypoint
marker on a high bridge. A fourth plays R at a waypoint marker on an edge-slope cell (S-0106), next
to a flat cell of the same height level whose own marker gets R from a fifth trigger. *Observable:* the resource in the first cell after tick 15 (the recorder
reads the cell's resource); whether R's start sound is heard in tick 15 (by ear); the shroud around
the second marker after tick 30 (screenshot), against the stock drop-zone radius read from the rules, and
the third animation's drawn height; the drawn heights of the fourth and fifth animations
(screenshot). *Procedure:* let the polled triggers fire without moving units into the observed
cells. Confirm the sound is audible in a separate ordinary-animation control, and that resource
remains in a control cell without R. *Expected:*
the resource is removed from the first cell in tick 15, and the start sound is heard; a disc of *r* quo 256 cells around the second marker is revealed
to the human player; the third animation is drawn at the bridge's height; the fourth is drawn half
a height level (the terrain height lift of S-0101 claim 4, halved) higher than the fifth, as the
centre height of an edge slope (S-0106 claim 2) gives. *Falsified if:* the resource remains,
the start sound is not heard, the reveal is missing or has another radius, the third animation is drawn
on the ground below the bridge, or the fourth and fifth are drawn at the same height.
Audio failure in the control, or unrelated resource removal, makes that part inconclusive.

**Scenario T22 (claim 5).** *Setup:* the common setup, with T20's repeating trigger R1 (only event
any-event, counting) as the first and only trigger of its tag, in variants of R1's owner-country
field. The human plays country H. (a) H's type name in another letter case. (b) H's name field
instead of its type name, for a country H whose name field differs from its type name (in the rules,
or set so by named test data). (c) `<none>`, with the human playing the first country of the country
list (S-0004). (d) `<NONE>`, as (c). (e) `<none>`, with neither player playing the first country of
the list. (f) A bare `none`, with the human playing the first country of the list, run as a game of its own. *Observable:* firings per tick on R1's counter,
over ticks 0 … 40. *Expected:* (a) to (d): one firing in every tick and two in ticks 0, 8, 16, 24, 32
and 40, as T20 gives for an owner country that is played. (e): one firing in every tick and no
second firing. (f): the original's result is unspecified (claim 5), so no firing count is expected,
and the variant is exploratory only: no result confirms or falsifies the unspecified case.
*Falsified if:* (a) to (d) show no second firing in the ticks that are multiples of 8, or (e) shows
one.

**Scenario T23 (claim 34).** *Setup:* with the random-state recorder. A map in which: one cell
carries a cell tag naming tag A (one polled trigger with a random-delay event); one pre-placed
vehicle carries tag B (the same kind of trigger); tag C is named by a cell tag and, later, by a
pre-placed object; a purely polled tag D with a random-delay event is carried by nothing; tag E is
polled and defined last in the tag list. A pre-placed vehicle, aircraft, infantry unit and
building each carry a further tag with a random-delay event. Give A two linked random-delay
triggers with separate counters to test chain-order arming too. Every timer trigger has a distinct count action/counter
so its firing deadline identifies its arming value. *Observable:* the recorder's log of the stream state before
the first tick, replayed (S-0202) into the sequence of draws, with the draws of the other loading
steps (S-0009, S-0010, S-0109, S-0202) identified by count, and the drop ticks of the counters, which
identify the arming draws. *Expected:* the arming draws of A and C come at their cell tags, in
cell-tag entry order, before the draws of the first terrain recalculation pass; for each pre-placed object the creation
draw comes before the draws of its tag; the kinds' tag draws occur in vehicle, aircraft,
infantry, building order; A's trigger arming draws occur in chain order; D and
E take their draws at the end of loading, in registration order; every draw happens before the first
tick. *Falsified if:* another order, or a tag created at its object or cell tag taking its draws at
the end of loading.
Variant B, one run per case: retain the tags above and add the terrain and named theater test data
of S-0010 scenario G4 part C, runs 1 and 2. The observed area is that scenario's lower ramp cell,
away from starting units and pre-placed objects. Record its animation at the first drawn frame,
and recover the creation and immediate start-event draws from the stream log using S-0109 and the
specs of the animation's effects. *Expected:* in run 1 the animation starts in the game-start pass
and its draws follow the arming draws of D and E. In run 2 the second pass starts its animation
before D and E take their arming draws, and the game-start pass takes no further creation draws
for that cell. Both runs finish these draws before tick 0. *Falsified if:* the draw groups occur in
another order, or run 2 creates a second tile animation in that cell. A run in which a unit enters
or leaves the observed cell before the first drawn frame is void.
Variant C: retain the tags above and use the isolated bridge and vein sites of S-0202 scenario
R8 parts A and B, and its cell-animation and placement-recalculation setups of parts F and I,
each on a separate map, with the recorder. Use those scenarios' run counts and observed areas.
*Observable:* the raw values that determine bridge pieces and vein patterns, the animations at
those sites, and the stream positions of these draws relative to the tags' arming draws.
*Expected:* bridge-piece and overlay cell-animation draws lie after the cell-tag arming draws
and before the first terrain pass; vein-pattern draws lie after that pass and before pre-placed
objects' creation and tag draws. A cell animation's creation and immediate start-event draws
precede its placement's tile-animation draws. Counts and values are S-0202 claim 10's, not
additional draws of this spec. *Falsified if:* a group lies on the other side of a tag-arming
group, the cell-animation/tile-animation order is reversed, or a count or value differs from R8.
Variant D: retain D and E's remaining-tag timers and use S-0202 R10 variants A and B's
secret-lab and skipped-starting-unit fixtures, separately. Keep those scenarios' settings,
observed areas, run counts and recorder requirements. *Observable:* the stream positions of
lab choices, D/E arming and game-start animation draws. *Expected:* lab choices follow initial
crates and precede D/E arming, which precedes game-start animation draws, even with absent
or prematurely stopped starting-unit placement. All these draws precede tick 0.
*Falsified if:* any group has a different relative position. Cleanup is not tested here:
S-0202 R11 is deferred because its former fixture was a network game, outside this revision.

Variant E: retain D and E's remaining-tag timers and use S-0202 R13's resource-glint fixtures,
settings, observed area, five runs per case and recorder checkpoints. *Procedure:* let loading
finish, logging each group's stream state from the run's own recorded seed. *Observable:*
the stream positions of D/E arming, game-start tile-animation creation, glint choices and each
glint's creation draws. *Expected:* D/E arming precedes the game-start recalculation's draws,
which precede the glint group; all precede tick 0. Within that group the counts, values,
empty-cell and zero-value controls, and choice-before-creation-before-next-choice order are
S-0202 claim 14's, as R13 predicts. *Falsified if:* any group has another relative position,
or any draw count, value or control differs from R13. Retain R13's void rule. An insufficiently
resolved log is inconclusive, not a passing ordering test.

**Scenario T24 (input-compatibility rules).** Common setup and recorder. One run per case,
using named test maps removed afterwards.
- *A, references:* create a disabled trigger `Alpha` first, with all difficulty flags set, an
  any-event and its own count action. A separate enabled elapsed-time 2 trigger enables it.
  Run once with the action reference `0`, once with `xx` (no decimal digits), and once with
  `-1suffix`. No earlier reference registers another trigger. *Expected:* the first two enable
  Alpha, which counts in tick 30 or 31 depending on tag order; the third never does.
  Contrast: a separate map defines trigger ID `0` with any-event/count, reached through a tag's
  first-trigger field `0`, while an earlier unrelated disabled Alpha occupies index 0. It counts
  in tick 0 by ID, not by index. Repeat the contrast with `0` as a linked-trigger field instead.
  *Observable:* counter drops. *Falsified if:* the first runs do not enable Alpha, the −1-prefix
  run does, or a short ID link selects Alpha instead. This tests trigger references only, not teams.
- *B, missing fields:* use T2 C's named base rules test copy with session position 3 and no lobby
  replacement. A trigger entry ends after its name (before disabled), has any-event/count, and
  a separate elapsed-time 2 trigger enables it. *Expected:* no count in tick 0; one in tick 30 or
  31 after enabling. In a separate map with session position 1, end the tested entry after its
  position-0 flag, which is set, with disabled clear. *Expected:* it never counts, even after an
  enable action, because the absent position-1 flag is clear. *Falsified if:* a premature count
  in the first run, no count after enabling there, or any count in the second. *Inconclusive if:*
  the lobby replaces the test session position.
- *C, undefined definitions:* an enabled trigger P (elapsed time 1/count) links to undefined
  trigger ID `Missing`. A one-time tag naming P is carried by a neutral vehicle. *Observable:*
  carrier attachment after tick 0, read by the recorder, and the counter through tick 16.
  *Expected:* the empty linked trigger fires at the tag's first poll in tick 0, the vehicle loses
  its tag, and P never counts at tick 15. Control: no link; the vehicle keeps its tag until P
  counts at tick 15. *Falsified if:* the main run retains the tag after tick 0 or counts, or the
  control drops its tag early. Repeat with each session position 0, 1 and 2 using T2's named
  base-rules test copy; this checks the empty definition's three enabled difficulty flags.
  Undefined cell-tag definitions are tested by T4 run 4.
- *D, malformed entries:* run a valid control map, then one separate map for each of: a trigger
  key that is a no-definition marker, a trigger ending before its link field, a tag ending before
  its first-trigger field, a negative event count, a negative action count, an event count greater than the complete events,
  an action count greater than the complete actions, an event missing its value, an action missing
  its seventh item, and a linked-trigger cycle.
  *Observable:* whether tick 0 is reached and whether loading reports a program error or ceases
  responding. *Expected:* the control reaches tick 0; no malformed case reaches tick 0.
  A cycle is expected to stay in loading without progress. Record a 60-second timeout as
  provisional evidence of a loading hang, not proof of an infinite wait. *Falsified if:* any
  malformed case finishes loading. Our engine's safe fallbacks are not oracle expectations.
- *E, empty items:* a valid one-time any-event trigger entry with disabled clear and all three
  difficulty flags set. Insert an empty item between its name and disabled flag, without
  removing any of the nonempty items. *Observable:* its counter at tick 0. *Expected:* one
  count, just as the control without the extra comma: empty items do not reserve a field.
  *Falsified if:* the extra comma disables it or prevents loading.

**Scenario T25 (claim 19; Q-0510).**
- *A, tagged passenger control:* common skirmish setup, speed index 3, no engine add-on and no mods (the named test files of a scenario excepted).
  Pre-place one ordinary infantry unit with a fresh repeating tag whose only event is
  destroyed-by-anything and whose action counts on the calibrated durable counter. Its
  trigger owner country is unplayed; it has no timer or polling event. Load it into an
  exploding, non-open-topped transport using normal input. Verify with the read-only recorder that it still
  carries the tag. Keep the counter beyond all death-weapon damage. Kill the transport with
  one known attacker, then repeat in a fresh game using a separate polled tag's lethal
  explosion at its waypoint, with no attacker. Use one passenger only and a non-crashing
  transport so the destruction path is unambiguous. Record passenger health before cleanup,
  its removal tick, counter health loss and transport death-weapon effects. Run 5 valid
  trials per attribution. *Expected:* two calibrated counter firings for the passenger in
  the transport's destruction tick, before the death weapon, with or without an attacker.
  No separate hit on the passenger and no later removal firing occur. A one-time-tag control
  gives one firing. *Falsified if:* a valid run gives another count or a later notification.
  Tag loss on loading, counter damage from another source or a trace too coarse to resolve
  the ordering makes the corresponding test inconclusive.
- *B, conditional hidden-slave test:* no data-only generated-slave tag-attachment fixture
  is established. Do not write into the running game to create one. If such a fixture is
  found, use A's tag, counter and recorder under the common setup, recording that exactly
  one hidden slave carries the fresh tag. Repeat the master's killing-hit case of S-0312
  claim 30, remaining-group removal of S-0312 claim 31, true sale of S-0408 claim 10a,
  and temporary-group cleanup on successful pack-up of S-0415 claim 10. Use 5 valid runs
  per case, with the counter and tag unaffected by other actions. *Expected:* two firings
  before that slave's removal in each case; only the killing-hit case with a known attacker
  has the three preliminary raises, which do not fire a fresh destroyed-by-anything trigger.
  No hit or second final-removal group occurs. The old working slaves survive successful
  transfer. A one-time-tag variant gives one firing; on our engine only, persistent latching
  off gives one repeating firing. *Falsified if:* the original gives any other valid count,
  a slave hit, or a second removal group. A shared previously latched tag does not test
  these fresh-tag counts. Until tag attachment is established this is an unmeasured proposal,
  not measurable backing; losses or disappearing slaves alone cannot test notifications.

## Open questions

- **No-hit notification coverage.** Claim 19 lists confirmed cases, not every possible
  deletion or ownership transition. Generated-slave tag attachment remains unestablished
  (Q-0510; conditional T25 B). Earlier erasure and environmental-sinking group-ending
  attribution and never-placed production masters retain S-0312 claims 30 and 31's open
  boundaries; this revision assigns them no new raise or firing count.
- **Cleanup draw measurement.** A discriminating skirmish fixture for cleanup's phase order
  and conditional tile-animation draws is not established (claim 34; S-0202 claim 14).
- **Lobby control for the computer difficulty.** Whether the stock skirmish lobby sets
  the session computer difficulty (claim 4), from which control, and whether the value restored at a
  later game start replaces a lobby choice.
- **Codes outside this subset.** Every event and action code not in tables I3 and I4 is parsed by
  Import mapping. Except for the companion-spec pointers, its meaning remains deferred.
  That includes the extra latch rule
  (a few codes of the latchable set, the build-type events for example, latch on any repeat type)
  and how polling (claim 6) treats a code that is both polled and code-matched (code 50).
- **Re-raises outside the subset** (claim 8). The exact codes and conditions of extra raises
  at a unit's arrival, under a unit in the air every 16 ticks, at an attack on a building
  (claim 35) and at team events. S-0312 claim 29 and its Import mapping establish the
  destruction groups; the out-of-subset events' full conditions remain deferred.
- **Type-exists name lookup across kinds** (claim 21). Which type the type-exists and
  type-does-not-exist events use when types of two kinds share a type name.
- **Registration order of trigger definitions.** A trigger reference shorter than 3 characters
  names the trigger definition with that registration index (Import mapping, I2), but this revision
  does not state the order in which trigger definitions are registered.
- **Other cheering objects** (claim 29). What the cheer action does to non-infantry objects.
- **Building-under-attack notification frequency** (claim 35). T20 does not isolate those
  notifications. Continued owner-house raises after defeat are established by S-0414
  claim 18/V15.
