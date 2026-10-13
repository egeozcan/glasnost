---
id: S-0011
title: Map triggers (multiplayer subset)
topic: triggers
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: []
supersedes: []
---

# S-0011: Map triggers (multiplayer subset)

## Summary

How the triggers that a map defines are set up while a game loads, when they are evaluated and fire,
and what the actions of this revision's subset do (the event and action codes of Import mapping,
tables I3 and I4), including the order in which the scenario random stream is drawn while a map loads.
Codes outside the subset are parsed, and their meaning is left to a later revision.

## Terms

Terms used as defined in the glossary: *trigger*, *trigger action*, *cell tag*, *tick*, *house*,
*country*, *human-controlled*, *local player*, *game start*, *game minute*, *type*, *type name*,
*registry*, *registration index*, *weapon*, *warhead*, *cell*, *lepton*, *height level*,
*scenario random stream* (with *draw* and *ranged draw*), *quirk switch*, *departure*, *importer
warning*, *unspecified*, *measurement setup*, *recorder*. *Map trigger variable* is used as defined
in S-0009 claim 32. *Observer* is used as defined in S-0009 (Terms): a human participant whom the
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
  table "Start position resolution") and after starting crates, but before the game-start terrain
  recalculation pass of S-0010 claim 12 and before the first tick. At this point claim 34 creates
  the remaining tags, claim 6 decides which tags are polled and claim 35 assigns tags to houses.
  The name denotes this tag set-up point, not completion of every row in *Load-time draw order*
  (State and ordering). It is not the glossary's *game start*, which is the moment loading begins.
  The whole of loading, the game-start pass included, happens while the tick counter is 0.
- **Elapsed-time event.** An event with the elapsed-time code (claim 13); with the random-delay
  event it is one of the two timer events.
- **Trigger-played animation.** As defined in S-0109: an animation that claim 36 creates. This spec
  says from when the mark applies; S-0109 says what it changes.

## Behaviour

### Definitions, tags and instances

1. **Only chained triggers run.** A trigger definition has an effect only as a member of the chain
   of some tag definition. A tag holds its own instance of every member of its chain, so a trigger
   definition that is a member of two chains has two independent instances, and one that is a
   member of no chain never fires. In the original, a map with a chain that comes back to a member
   already in it never finishes loading (Edge cases,
   *Unparseable entries*). Our engine ends the chain before the first repeated member, and the
   importer warns (a departure).
   - Backing: `measurable` (scenario T1)
   - Confidence: high (medium for the shared member)

2. **One tag per tag definition.** Every carrier of a tag definition carries the same tag, so all
   of them share one set of trigger instances, one trigger timer per instance and one latch set
   per instance. The tag is created the first time a cell tag, a pre-placed object or the end
   of loading needs it (claim 34). The tag counts its carriers: attaching it to a cell or an object adds
   one, and detaching it removes one.
   A cell tag names its tag definition by the definition's ID, without regard to letter case
   (Import mapping). A pre-placed object's tag field matches either the ID or the name field of a
   tag definition, without regard to letter case: the object gets the first tag definition in
   registration order (claim 7) whose ID or name equals the field. A tag name longer than 48 bytes
   is cut to its first 48 bytes, so only that cut name matches. A tag field that matches no tag
   definition gives the object no tag and creates no definition. The `none` marker of an untagged
   object normally matches nothing in this way; the importer warns about any other unmatched
   field.
   - Backing: `measurable` (scenario T6, variants E and F)
   - Confidence: high (medium for the name-field match and the cut)

3. **Enabled at start.** A trigger instance starts enabled exactly when its definition's disabled
   flag is clear and its difficulty flag at the game's difficulty position is set (claim 4 gives
   the case of a setting outside 0 … 2). Otherwise it starts disabled. A disabled instance is never satisfied and never fires its actions. Only the
   enable action (claim 25) enables it.
   - Backing: `measurable` (scenario T2)
   - Confidence: high

4. **Difficulty position of skirmish and multiplayer games.** In a skirmish or multiplayer game the
   difficulty position is the session's computer-difficulty setting. The program initialises it
   from the base rules' multiplayer-dialog difficulty field when the base rules' game-wide values
   are read (Import mapping; S-0006 says when that happens). A lobby control can change it.
   Whether the stock lobby shows such a control, and whether the value restored at a later game
   start replaces a lobby choice, are not established (Open questions). Campaign missions take it from the chosen campaign difficulty, which is outside this
   spec. A setting outside 0 … 2 selects no difficulty flag: every trigger instance whose disabled
   flag is clear then starts enabled, and the enable action (claim 25) enables every instance it
   names, whatever their difficulty flags. Original data behaviour, kept, and the importer warns.
   - Backing: `measurable` (scenario T2, variant C for the out-of-range setting)
   - Confidence: medium for "the session setting, initialised from the base rules" and for the
     out-of-range setting, low for the lobby

5. **Owner house.** Each trigger instance has the owner house defined in Terms, as it stands at the
   moment the owner house matters, so it follows the houses that exist at that moment. In a skirmish or multiplayer game,
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
   the original polls every polled tag, one tag after the other in the registration order of claim
   7. For the events of this subset, polling a tag has the effect of a single raise of it in which
   no code-matched event is judged true: condition events, the any-event code, empty event lists
   and latches decide (claim 8). So in every tick each polled tag is evaluated, and it fires at most
   once in that tick from polling. Its carriers can raise it
   again in the same tick (claims 18 to 20), and so can its owner house (claim 35). Actions that a polled tag fires take effect before the
   later polled tags are evaluated in the same tick, and before the rest of that tick's game logic.
   Whether a tag is polled is decided once, at the end of loading (Terms), from the codes of its
   chain's events. This applies to every tag that exists then, including those created earlier in
   loading by cell tags and pre-placed objects.
   - Backing: `measurable` (scenario T4)
   - Confidence: high for once per tick and at most one firing per tick, medium for the position at
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
   - Backing: `measurable` (scenarios T5 and T10)
   - Confidence: high (low for the list of out-of-subset raises)

9. **Latches on repeating tags.** When an event of the latchable set (Constants) is judged true in
   an evaluation of a trigger instance of a repeating tag, it is latched, whether or not the
   trigger as a whole is satisfied. A latch is never cleared. So, after its first satisfaction, a
   latched event counts as satisfied in every later raise of its tag, of any code. Example: a
   repeating tag carried by two vehicles, with one trigger whose single event is
   destroyed-by-anything. The first vehicle's destruction fires the trigger twice: in the
   destroyed-by-anything raise, and again, by the latch, in the raise that follows it (claim 19).
   After that the trigger fires again in every later raise of the tag, for example in each of the
   raises that every hit on the second vehicle causes (`quirks.persistent_event_latch`). Latches are kept per event position, and positions from 33
   on share the latch of the position 32 lower (`quirks.event_latch_aliasing`). A few codes outside
   this subset also latch on other repeat types (Open questions).
   - Backing: `measurable` (scenario T7; variant C for the shared latch of positions 1 and 33)
   - Confidence: medium

10. **Repeat types.** The repeat type decides what happens when a trigger instance is satisfied in a
    raise. The full table is in State and ordering. In short: a one-time tag fires the trigger, and
    then the tag leaves all its carriers and is deleted. An all-carriers tag fires the trigger only
    when the tag has exactly one carrier left, and otherwise only detaches the raising carrier. So
    an all-carriers tag with no carrier (for example one that is only polled) never fires
    (`quirks.all_carriers_without_carrier`). A repeating tag fires the trigger every time and stays.
    Any other repeat value never fires, and the importer warns.
    - Backing: `measurable` (scenario T6)
    - Confidence: high for one-time and repeating, medium for all-carriers

11. **Firing.** When a trigger fires, its actions take effect in the order of its action list,
    immediately, within the raise that satisfied it: the effects are in force for whatever follows
    in the same tick. Every action takes effect even when an earlier one had no effect. All chain
    members satisfied in the same raise fire in that raise, the last member of the chain first and
    the first trigger last. Each member is judged only when its turn comes, after the actions of the
    members that fired before it in that raise have taken effect. So a member that fires earlier in
    a raise can stop a later one (by disabling it, for example) or let it be satisfied (by setting a
    variable that it tests). A fired member of a one-time tag is removed at once. The other members
    of a one-time tag that fired stay able to fire through other raises until the end of that tick
    and are gone from the next tick on, together with the tag.
    - Backing: `measurable` (scenario T8; variant B for the effect of an earlier member on a later
      one) for the order within a raise. The survival of the other
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
    fires in the tick numbered 15*v* (the first tick is numbered 0).
    - Backing: `measurable` (scenario T10)
    - Confidence: high for the length and the tick, medium for "first written wins"

14. **Random delay.** A random-delay event with value *v* arms the timer with the length of
    Formulas F2. Each arming takes one ranged draw from the scenario random stream for every
    random-delay event of the trigger (none when *v* = 0), in reverse order of the event list,
    whether or not that event sets the timer. The draws happen when the timer is armed (claims
    15 and 34).
    - Backing: `measurable` (scenario T11). Variant A, statistical, gives the distribution of the
      delay. Variant B reads the number, order and values of the draws with the random-state
      recorder of the measurement setup, which only reads the original's seed and stream state
    - Confidence: high for the formula, medium for the draw count and order

15. **Re-arming.** A trigger timer is armed again, at the current tick, with a new length (and new
    draws), when:
    - the instance is satisfied in a raise (whether or not its repeat type then lets it fire),
      and either the tag is repeating or an entered-by event of the trigger was judged true in that
      raise. An entered-by event judged true in a raise in which the trigger as a whole is not
      satisfied does not re-arm the timer. For a repeating tag this means: a repeating timer
      trigger fires every *D* ticks (a new *D* each time for a random delay),
    - the enable action enables it (claim 25), even when it was already enabled,
    - a map trigger variable changes value and the trigger has a variable-set or variable-clear
      event for that variable (claim 22).

    Nothing else arms it again. In particular disabling does not.
    - Backing: `measurable` (scenario T10, variants B to D; scenario T5 (c) for the entered-by
      case)
    - Confidence: medium

16. **Game-time event.** A game-time event with value *v* is satisfied in the tick numbered *t*
    exactly when F4 holds. It counts from the start of the game, so it is not affected by arming.
    - Backing: `measurable` (scenario T10, variant E)
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
    object, when a unit arrives in a cell that carries the tag (the moment of arrival belongs to the
    movement specs, a pointer only). In a cell that carries a high bridge, only units on the bridge
    deck raise it. The tag of a building is raised with this code, with the infantry unit as causing
    object, when an infantry unit enters the building to take it over (capture rules: no spec yet, a
    pointer only). A few other ways of entering an object also raise the entered object's tag with
    this code, with the entering unit as causing object; which ones belongs to the specs of those
    abilities (pointers only). The event is satisfied when the raise has this code, there is a causing object,
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
    - Backing: `measurable` (scenario T3, in a skirmish with one human player and one computer
      player)
    - Confidence: medium (low for the bridge rule, the capture case and the other entries)

19. **Destroyed-by-anything.** When an object that carries a tag is destroyed, whatever destroyed it,
    its tag is raised with this code, with the object as causing object. The event is satisfied in
    every such raise. It is latchable (claim 9). If the object still carries the tag after that
    raise (the tag is repeating, or the trigger was not satisfied), the destruction raises the tag
    once more, with a code outside this subset; when the destruction has an attacker, further raises
    with codes outside this subset come before the destroyed-by-anything raise. These extra raises
    evaluate condition events and latches (claim 8).
    - Backing: `measurable` (scenario T7)
    - Confidence: medium (medium for the raise after it, low for the raises before it)

20. **Bridge destroyed in a tagged cell.** When a bridge is destroyed, each cell of the destroyed
    part that carries a tag raises that tag with this code. The event is satisfied in every such
    raise. Which cells count as part of the destroyed bridge belongs to the bridge specs (no spec
    yet, a pointer only).
    - Backing: `measurable` (scenario T13)
    - Confidence: low

21. **Type exists and type does not exist.** These condition events name a type (by its type name)
    and a count *n*. The type is the building, vehicle, infantry or aircraft type whose stored name
    equals the event's name exactly, letter case included. If several do (S-0004 claim 8), the
    original takes the last of them in a single order that runs over those four kinds together; that order is not established. If none does, neither event is ever judged true, and the importer warns. Otherwise
    type-exists is satisfied when at least *n* objects of that type exist, counting every object of
    the type of every house, including objects inside transports or buildings. When *n* ≤ 0 it is
    satisfied as soon as any building, vehicle, infantry or aircraft object exists, of any type
    (original data behaviour, kept, and the importer warns about a count below 1).
    Type-does-not-exist is satisfied when no object of the found type exists. The exact match is an
    exception to S-0003 claim 14, which matches type names without regard to case (Edge cases,
    *Exact type names*). The `<none>` and `none` markers get no special treatment here: they match no type,
    so the events are never judged true.
    - Backing: `measurable` (scenario T14; variant C for *n* ≤ 0, variant D for an object inside a
      building)
    - Confidence: medium (low for which objects count while inside another object, and for
      type-does-not-exist)

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
    - Backing: `measurable` (scenario T12)
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
      medium for "without announcement"

25. **Enable and disable a trigger.** The enable action names a trigger definition. Every existing
    instance of that definition, in any tag, becomes enabled and has its timer re-armed (claim 15) when its difficulty flag at the game's difficulty position is set. If that position is outside 0 … 2, every existing instance of the definition becomes enabled and has its timer re-armed, whatever its three difficulty flags. Instances that do not meet the applicable condition are left as they are. The disable action disables every instance of the
    named definition. Neither action creates instances, so naming a definition that is no member of
    any existing tag's chain has no effect.
    - Backing: `measurable` (scenario T2, variant B, and T10, variant C)
    - Confidence: high

26. **Reveal around a waypoint.** The action's number slot is a waypoint number. If the local player has
    not already had the whole map revealed, the original reveals, for the local player only, the
    area of radius *R* cells around the point of Formulas F6, where *R* is the reveal-trigger radius
    of the rules (Import mapping). How an area is revealed belongs to the visibility specs (a pointer
    only). In a skirmish the local player is the single human-controlled house, so
    that house has the area revealed and no computer-controlled house has. In a network game each
    machine reveals the area for its own local player, which is outside this spec (Terms). Our
    engine applies the reveal to every human-controlled house that has not had the whole map
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
    object is normally destroyed. At least two kinds of protection still apply, because they depend
    on the warhead's kind rather than on defences: a type immune to radiation, psychic or poison damage
    takes no damage from a warhead of that kind, and a warhead that makes its target go berserk
    takes no health. The rules' demolition warhead (Import mapping) is named as the damage's warhead, which
    matters only for how the destruction looks and what it leaves (combat specs, a pointer only). 
    If such an object survives the damage (possible through the protections above, which need a
    demolition warhead of such a kind), the action never completes in the original: from that tick
    on the game stops responding, the tick counter stops advancing, and neither the trigger's later
    actions nor any later tick happen. Our engine
    damages each such object once per firing and goes on, so a survivor stays (a departure: the
    original's result is a hang), and the importer warns about a demolition warhead of such a kind. For a trigger fired from a cell, the original also acts on the
    cell, which is not established here (Open questions).
    - Backing: `measurable` (scenario T9, including a carrier whose armour the demolition warhead
      cannot damage, and variant C for the survivor and the hang)
    - Confidence: medium (low for the two remaining protections; the hang has not been observed
      yet and becomes high-confidence only once variant C has run)

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
    - Backing: `measurable` (scenario T18)
    - Confidence: medium

30. **Actions that hand over to other specs.** These actions are recognised, and their parameters are
    read as Import mapping says. What they do belongs to specs that are not written yet (pointers
    only): create team, destroy team, reinforce and reinforce at a waypoint (the team system), create
    crate at a waypoint (the crate rules), lightning storm at a waypoint (the superweapon specs, with
    the trigger's owner house as the storm's owner) and preferred target at a waypoint (computer-
    controlled behaviour, which is out of scope, with the owner house as the house concerned).
    - Backing: `unmeasured`: the claim only assigns these effects to their owning specs, which will
      carry the measurements. The parameter reading is part of Import mapping
    - Confidence: medium

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
      default label)
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
    - Backing: `measurable` (scenario T17, variant B)
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
    the map. An object whose house or type is not found is not created and creates no tag. The table *Load-time draw order* (State and ordering)
    puts these draws together with the draws that S-0009, S-0010, S-0109 and S-0202 assign to
    loading, in one stream order. Every one of them happens before the first tick.
    - Backing: `measurable` (scenario T23): the random-state recorder reads the stream state before
      the first tick and the draws between the groups the table lists. The boundary of the claim,
      the first terrain recalculation pass, is a group the table lists
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
    any-event code, without a causing object or cell. The houses take their turns in house order
    (S-0009 claim 19), and a house raises its tags in reverse registration order (claim 7). So the
    actions that one house's raises fire are in force when the next house raises its tags, and
    within one house the actions fired by a later-registered tag are in force when an
    earlier-registered tag is raised. The
    owner-house raise evaluates every trigger of the tag like any other (claim 8), so condition events (elapsed
    time, game time, variables, type counts) and latches can fire in it, as well as the any-event
    code. The assignment is made once and never changes. A defeated house stays in the house list and keeps
    raising its tags, and a tag whose first trigger's owner country nobody plays is never raised
    this way. A house also raises all its assigned tags, with a code outside this subset, each time
    it is told that one of its buildings is under attack; how often that happens belongs to the
    combat specs (a pointer only). That raise, too, evaluates condition events and latches.
    - Backing: `measurable` (scenario T20)
    - Confidence: medium (low for the reverse order within a house and for the under-attack raise)

### Animations at a waypoint

36. **Play an animation at a waypoint, and the drop-zone animation.** The play-animation action's
    number slot is an animation type's registration index (S-0004); the drop-zone action uses the
    rules' drop-zone animation type (Import mapping) and ignores its number slot. Both create one animation of
    that type at the point that claim 28 uses for its waypoint slot: the centre of the
    waypoint's cell, at the cell's centre height (S-0106 claim 2), raised by 4 height levels when
    the cell carries a high bridge. In the terms of S-0109, the animation has no loop delay, a
    repeat factor of 1 and no reversed play, and is attached to no owner. How it plays, and the
    draws its creation takes, belong to S-0109 (a pointer only).
    - **The trigger-played mark.** The animation is a trigger-played animation (Terms) in
      everything except what happens at its creation. Because it has no loop delay, its first start
      event (S-0109 claim 13) happens at its creation and is not under the mark: like any other
      animation's, it plays the type's start sound and starts the type's resource chain reaction.
      Everything after its creation is under the mark: later start events (a follow-up switch, a
      pass after a loop delay), the damage over time of S-0109 claim 17 and the sounds that S-0109
      ties to the mark. What exactly the mark suppresses is S-0109's.
    - **Drop-zone reveal.** Creating an animation of the drop-zone type, by this action or
      otherwise, reveals the area of radius *r* quo 256 cells (truncated toward zero) around the
      animation's position for the local player, where *r* is the rules' drop-zone radius in leptons
      (Import mapping). Unlike claim 26, the reveal takes place whether or not the local player has
      already had the whole map revealed. How an area is revealed
      belongs to the visibility specs (a pointer only). As in claim 26, the original reveals for each machine's
      local player, which is outside this spec for network games (Terms), and our engine applies the
      reveal to every human-controlled house, observers included, so the result does not depend on
      the machine (a departure).
    - A play-animation number-slot value that is no registration index of an animation type is unspecified in
      the original, which does not reject it. Our engine gives such an action no effect, and the importer
      warns (a departure). Without a drop-zone animation type in the rules, the drop-zone action is
      not covered here.
    - Backing: `measurable` (scenario T21). That the drop-zone reveal takes place whether or not
      the whole map is already revealed is `unmeasured`, for the reason given under claim 26
    - Confidence: high for the point, the creation and the timing of the mark; medium for the
      drop-zone reveal and its radius, and for the point on a sloped cell

## Formulas

> **F1. Elapsed-time length.** *D* = 15 × *v*, where *v* is the event's integer value. Exact integer
> arithmetic. Unit: ticks.

> **F2. Random-delay length.** *D* = 15 × (*q* + *r*), where *q* = *v* quo 2 (truncated toward zero)
> and *r* is a ranged draw over the inclusive range between 0 and *v* (from *v* to 0 when *v* is
> negative). When *v* = 0 there is no draw and *r* = 0. Exact integer arithmetic. Unit: ticks.
> Example: *v* = 10 gives lengths from 75 to 225 ticks.

> **F3. Timer run out.** The timer armed at tick *s* with length *D* has run out in the tick numbered *t*
> when *t* − *s* ≥ *D*. Integers. Unit: ticks.

> **F4. Game time reached.** *v* ≤ *t* quo 15, with *t* ≥ 0 the tick number and quo the integer quotient
> truncated toward zero. For *v* ≥ 1 this is *t* ≥ 15*v*. Unit: trigger time units.

> **F5. Letters of a waypoint parameter.** A waypoint parameter is read from its first two characters
> *c*₁ and *c*₂; further characters are ignored. Let *L*(*c*) be the position of letter *c* in the
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
| enabled | enable action | stays enabled, timer re-armed |
| disabled | enable action, difficulty flag set | enabled, timer re-armed |
| disabled | enable action, difficulty flag clear | stays disabled |
| disabled | any raise | nothing |
| any | fired in a one-time or all-carriers tag, or its tag deleted | removed: never evaluated again |

**Observable ordering within a tick.**
- The polled tags are evaluated at the start of the tick's game logic, in registration order (claims
  6 and 7). So a polled trigger and a carrier event in the same tick: the polled trigger's actions are
  in force first.
- A raise by a carrier (arrival, destruction, capture, bridge collapse) happens during the game logic
  that causes it, and the actions it fires are in force for the rest of that logic.
- In ticks whose number is a multiple of 8, the owner-house raises (claim 35) come near the end of the
  tick's game logic, after the polled tags and after the objects' own updates of that tick.
- In one raise, chain members fire last-member first (claim 11), and each member's actions take effect
  in list order.

**Load-time draw order.** The draws that the scenario random stream gives while a map loads, by
point in loading, in stream order: every draw of a row comes after the draws of the rows above it,
and within a row the draws come in the order listed. It merges S-0009's table "Start position
resolution" with the draws of this spec, of S-0010, S-0109 and S-0202. A row marked "not established"
names a point at which draws may happen that no spec yet lists. Draws at points the table does not
name are not established either. Nothing else in the table fixes an order of internal work.

| Point in loading | Draws, in stream order | Owning spec |
|---|---|---|
| Early phase | list 1's random entries; list 2's random entries; the random-position draws, in house order | S-0009, claim 20 |
| Initial fill | one per visited cell, for a water fill only | S-0009, claim 6 |
| Cell tags, in entry order | for each tag created there: claim 14's draws for each of its trigger instances, in chain order | this spec, claim 34 |
| Terrain stream, tunnels, overlays | none known | S-0009 |
| First terrain recalculation pass, in S-0009's visiting order | for each tile animation started (S-0010 claim 19): the draws of its creation (S-0109 claim 3) and of its immediate start event (S-0109 claim 13 and the specs of its effects). These animations are removed again before the second pass, but their draws and start-event effects remain | S-0010, claims 12 and 19; S-0109 |
| Vein set-up around veinhole markers | draws for the vein overlays, when the map has veinholes (not established) | no spec yet |
| Terrain objects | not established | no spec yet |
| Pre-placed objects: the vehicle entries, then aircraft, then infantry, then buildings, each in entry order | per object: its creation draw (S-0202 claim 10); then, if its tag is created there, claim 14's draws for that tag's instances; then the draws of placing it (not established). A building's upgrades are created after the building's tag and take their own creation draws | S-0202; this spec, claim 34 |
| Smudges | not established | no spec yet |
| Second terrain recalculation pass | as the first pass, for the tile animations it starts | S-0010, claim 12; S-0109 |
| Unused-house cleanup | none known | S-0009, claim 28 |
| Late phase | list 3's random entries; the re-assignment draws | S-0009, claims 20, 28 and 29 |
| Starting units | the creation draw of each starting unit, and whatever else placing it draws (not established) | S-0202; starting-unit rules, no spec yet |
| Starting crates, when crates are on | the draws of placing them (not established) | crate rules, no spec yet |
| End of loading: remaining tag set-up (Terms) | claim 14's draws for every tag created there, in registration order, and for each tag its instances in chain order | this spec, claim 34 |
| Game-start terrain recalculation pass, in S-0009's visiting order | as the first pass, for the tile animations it starts. A cell that already started one in the second pass starts no second animation, so takes no further animation-creation draws in this pass | S-0010, claims 12 and 19; S-0109 |

## Edge cases and quirks

- **Persistent latch** (`quirks.persistent_event_latch`, default: original). A latchable event of a
  repeating tag stays satisfied forever after its first satisfaction (claim 9). *Fixed behaviour:* the
  latches of a trigger instance are cleared each time it fires.
- **Latch aliasing** (`quirks.event_latch_aliasing`, default: original). Event positions 33 and later
  share latches with the positions 32 lower. *Fixed behaviour:* every position has its own latch.
- **All-carriers tag without a carrier** (`quirks.all_carriers_without_carrier`, default: original).
  It never fires (claim 10). *Fixed behaviour:* a carrier count of 0 counts like 1, so the tag fires
  once and is deleted.
- **Entered-by with a player position** (`quirks.entered_by_country_only`, default: original). The
  entered-by value is read only as a country index, so the player-position values never match (claim
  18). *Fixed behaviour:* the value is resolved like an action's house value (claim 23), and the event
  is satisfied when the causing object's owner is that house (including the "References" rule of
  S-0009 claim 29 on maps with 9 or more header pairs). The importer warns about such a value in
  either case.
- **“None” owner.** `quirks.none_owner_is_first_country` (YR default: on). An owner written as `<none>` gives the first country of the country list (claim 5), with an importer warning. Off: `<none>` names no owner country.
- **Exact type names.** `quirks.exact_trigger_type_names` (YR default: on). The type name of the type-exists and type-does-not-exist events is matched case-sensitively, and `none` and `<none>` are ordinary unmatched type names (claim 21); the importer warns when the name matches no type. Off: use S-0003 claim 14's case-insensitive type-name matching and no-type markers.
- **Short references are registry indexes.** A reference parameter whose text is shorter than 3
  characters is read as a registration index into the team, trigger or tag definitions (Import
  mapping, I2), not as a name. Original data behaviour, kept for every map, with an importer warning.
  An index outside the registry is unspecified in the original. Our engine treats the reference as
  empty and warns (a departure).
- **Missing disabled field.** A trigger entry that ends before its fourth field starts disabled, and
  one that ends before a difficulty field has that flag clear. Original data behaviour, kept, and the
  importer warns about a trigger entry with fewer than seven fields.
- **References that create definitions.** A linked trigger, a tag's first trigger or an action
  parameter that gives an ID that no trigger definition of the map has creates an empty one: no events,
  no actions, no owner country, not disabled, and all three difficulty flags set. Its instance is
  therefore satisfied in every raise of a tag that holds it (an empty event list, claim 8) and fires
  with no effect. In a one-time tag that firing removes the tag at its first raise of any kind (claim
  10), so the other members of the chain lose their chance; in an all-carriers tag it detaches each
  raising carrier until one is left. A tag reference (or a cell tag) giving an undefined tag ID
  creates a tag definition with an empty chain, which has no effect. A pre-placed object's tag field
  creates nothing (claim 2). Original data behaviour, kept, and the importer warns.
- **Unparseable entries.** A trigger list key written as the "none" marker, a trigger entry that ends
  before its second field, a tag entry that ends before its third item, an event or action count
  that is negative or larger than the values that follow, an event that ends before its value item,
  an action that ends before its seventh item, and a chain cycle (claim 1) make the original crash or
  never finish loading. Our engine skips the entry (for a count: reads only the complete events or
  actions; for a short trigger entry: treats the missing fields as absent; for a chain cycle: ends
  the chain before its first repeated member, claim 1), and the importer warns (departures).

## Import mapping

All entries below are read from the map file. Entry order is the file's entry order after S-0002's
rules for repeated keys. Every value below is split into items at commas, and empty items are
skipped: two commas in a row, or a comma at the start, count as one separator, so the items after an
empty one move up one field. The importer warns about every empty item. An item that is read as an
integer is read by the decimal form of S-0003 claim 3 only: the `$` and `h` hexadecimal forms do not
apply to these items. Overflow of such an item is not covered here. Trigger, event and action values
are limited only by the line length limit of S-0001 claim 3 (their own text limit, in the sense of
S-0003, is at least as long as any value a line can hold, so it never binds), and a tag value has a text limit of 127 bytes (medium
confidence for both limits). A long event or action list cut by the line limit whose count needs
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

The carry-over flag makes a tag pass from an infantry unit to the building it takes over, and changes
what a vehicle's destruction raises in one case. Neither is part of this revision (Open questions).

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
integers and no covered code uses them.

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
| shorter than 3 characters | the definition with that registration index (Edge cases, *Short references are registry indexes*) |
| `<none>` or `none`, in any letter case | nothing |
| any other text | the definition whose ID (key) equals the text without regard to letter case, never one whose name field does. An ID that is not defined creates the definition (Edge cases, *References that create definitions*) |

The linked trigger of I1 and the first trigger of a tag are looked up the same way. A missing eighth
item leaves the waypoint slot at 0 (waypoint A).

**Tags** — section `[Tags]`, key = tag ID. Value: three items, the repeat type (an integer), the
name (cut to 48 bytes and matched by pre-placed objects' tag fields, claim 2) and the first trigger
(a trigger ID, or `<none>` or `none`). A key whose value is empty is skipped.

**Cell tags, variables, objects.** `[CellTags]` and `[VariableNames]` are read as S-0009 claims 31
and 32 say; a cell tag's value is matched against tag IDs like a reference (I2). A pre-placed
object's tag field is matched against tag IDs and tag names as claim 2 says (the object entries
have no spec yet).

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
| 8 | any event | unused | yes | no | 17 |
| 13 | elapsed time | *v* trigger time units | yes | no | 13 |
| 31 | bridge destroyed in tagged cell | unused | no | yes | 20 |
| 36 | variable set | variable number | yes | no | 22 |
| 37 | variable clear | variable number | yes | no | 22 |
| 47 | game time | *v* trigger time units | yes | no | 16 |
| 48 | destroyed by anything | unused | no | yes (latchable) | 19 |
| 51 | random delay | *v* trigger time units | yes | no | 14 |
| 60 | type exists | count, with kind 2 type name | yes | no | 21 |
| 61 | type does not exist | kind 2 type name | yes | no | 21 |

**I4. Action codes.** Slots are those of I2.

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
| 41 | play an animation at a waypoint | number slot as animation type index, waypoint slot | 36 |
| 42 | explosion at a waypoint | number slot as weapon index, waypoint slot | 28 |
| 53 | enable trigger | trigger reference | 25 |
| 54 | disable trigger | trigger reference | 25 |
| 55 | radar event at a waypoint | number slot (event kind), waypoint slot | 32 |
| 56 | set variable | number slot as variable number | 22 |
| 57 | clear variable | number slot as variable number | 22 |
| 80 | reinforce at a waypoint | team reference, waypoint slot | 30 |
| 99 | sound at a waypoint | number slot (sound), waypoint slot | 32 |
| 102 | lightning storm at a waypoint | waypoint slot | 30 |
| 103 | timer text | text label | 31 |
| 108 | create crate at a waypoint | number slot (crate kind), waypoint slot | 30 |
| 113 | make a house cheer | number slot as house value | 29 |
| 135 | preferred target at a waypoint | waypoint slot | 30 |

## Measurement scenarios

**Common setup.** Unless a scenario says otherwise: the measurement setup of the glossary, with a
purpose-made test map as named test data, removed after the run. The map is small and flat with two
start waypoint markers. It is played as a skirmish of one human player at position A and one computer
player at position B, at any game speed. The human player's actions are scripted mouse and keyboard input. The recorder is a program that
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
- *Owner countries.* Unless a scenario says otherwise, every test trigger's owner country is one that
  no house plays (a playable country that neither player chose), so that no owner-house raise
  (claim 35) adds firings. Scenarios T20 and T22 test that raise. House values in actions are unaffected.
- *Naming.* Wherever a scenario says that an action, a linked trigger, a tag or a carrier "names" a
  trigger or tag definition, it gives that definition's ID (Import mapping, I2), except where the
  scenario says it gives the name field (T6, variants E and F).

Runs: 1 per variant unless stated, since nothing below depends on the random stream except T11 and T23, which read it with the random-state recorder.

**Scenario T1 (claim 1).** *Setup:* trigger definitions P and Q, Q linked from P, both with event
any-event and a count action, and U, a trigger definition with the same event and a count action that
no tag's chain reaches. Tag X (one-time) names P. Variant: a second tag Y (one-time) names Q as its
first trigger. *Observable:* firings per tick on the counter. *Expected:* two firings in tick 0 (P
and Q), three in the variant (P, Q in X, and Q in Y), none from U and none in later ticks.
*Falsified if:* any other number of firings.

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

**Scenario T3 (claims 18, 23).** *Setup:* a skirmish of one human player and one computer player
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
*Observable:* firings per tick on each counter, and what is seen and heard in parts (a), (e) and (f).
*Expected:* (a) one firing in tick 0, and the four presentation actions are seen and heard at the
start. (b) one firing in every tick. (c) one firing, at the second entry (the first entry is judged
true but the trigger is not satisfied, so it does not re-arm the timer, claim 15). (d) exactly one
firing, in the tick of the first hit (the hit raises the tag at least once, claim 8), and none at
later hits, since the vehicle then carries no tag. (e) the building keeps playing the sound as its
ambient sound while the view is near it, and the sound at the empty cell is played once. (f) one
radar event is shown, not two. *Falsified if:* any differs. In part (c), in particular, no firing at
the second entry (which would mean that the first entry re-armed the timer).

**Scenario T6 (claims 2, 10).** *Setup:* three neutral vehicles carry the same tag, whose trigger
(destroyed-by-anything) counts. The human destroys them one by one with single shots of a weapon that
kills each in one hit. Variants: repeat type 1, 0, and 2. Variant D: an all-carriers tag carried by
nothing, with event any-event and a count action. Variant E: as type 0, but the tag's name field
differs from its ID, and the vehicles' tag fields give the name, in another letter case. Variant F:
as type 0, but the tag's name is 50 characters long; the three vehicles' tag fields give its first
48 characters, and a fourth such vehicle, whose tag field gives all 50 characters, is destroyed
first. *Observable:* counter drops after each destruction. *Expected:* type 1: one drop, after the
third destruction. Type 0: one drop, after the first; the other two no longer carry the tag. Type 2:
a drop after each destruction (and more, see T7). D: no drop. E: as type 0. F: no drop at the fourth
vehicle's destruction (it carries no tag), then as type 0. *Falsified if:* any differs.

**Scenario T7 (claims 9, 19).** *Setup:* two neutral vehicles V1 and V2 carry one repeating tag whose
only trigger has the single event destroyed-by-anything and a count action. The human destroys V1, then
hits V2 several times with a weapon too weak to destroy it. *Observable:* counter drops, against the
ticks of the hits. *Expected:* two firings in the tick of V1's destruction (the destroyed-by-anything
raise and the raise that follows it, claim 19), then at least one firing in the tick of each hit on V2
(a hit raises the tag one or more times, and the latched trigger fires in each raise). Variant: the same
with repeat type 0: one firing at V1's destruction and none afterwards. With
`quirks.persistent_event_latch` off, which only our engine offers, one firing at V1's destruction and
none at the hits. Variant C (latch aliasing): the trigger instead has 33 events, on the same repeating
tag: position 1 destroyed-by-anything, positions 2 to 32 any-event, and position 33 type-exists
naming a type name that no type has (which is never judged true). *Expected* in variant C: no
firing before V1's destruction; at least one firing in the tick of V1's destruction and in every
later tick, because position 33 shares the latch that position 1 gets (the tag is polled, since
its chain contains any-event). With `quirks.event_latch_aliasing` off, which only our engine offers,
no firing at all. *Falsified if:* in the main run V1's destruction brings other than two firings or
the ticks of the hits on V2 bring no drops, or a variant differs from its expectation.

**Scenario T8 (claim 11).** *Setup:* an owner marker carries a one-time tag whose chain is K1 → K2,
both with elapsed time 1. K1 changes the marker's owner with house value 4475, K2 with 4476.
Variant B: K2 also has, after its change-owner action, a disable action naming K1. *Observable:* the
marker's owner at the end of tick 15. *Expected:* K2 acts first and K1 last, so the owner is the
human player (4475). Variant B: K1 is judged after K2's actions have taken effect, so it is disabled
and does not fire, and the owner is the computer player (4476). *Falsified if:* the owner is the
computer player in the main run, or the human player in variant B.

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

*Observable:* ticks of the counter drops. *Falsified if:* any tick differs, allowing one tick of slack
where the scenario's own enabling or variable change and the tested tag share a tick (C, D, E).

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
random-delay event. A second case adds an elapsed-time event before the random-delay events in the
event list. *Observable:* the recorder's log of the stream state before the first tick and in every
tick, from which the number and values of the draws taken by each trigger instance's arming are
recovered by replaying the stream's recurrence (S-0202) from the logged state, and the tick of each
counter drop. *Expected:* the arming takes one draw per random-delay event of non-zero value, in
reverse order of the event list, and none for the value 0 or for the trigger without such an event.
The delay is the claim 14 formula applied to the draw of the event that sets the timer, as the tick
of the drop shows. *Falsified if:* another number or order of draws, or a delay taken from the other
event's draw.

**Scenario T12 (claim 22).** *Setup:* map trigger variable 3 starts clear (the map gives it initial
state 0). Trigger S (one-time, elapsed time 2) sets it, trigger C (one-time, elapsed time 6) clears
it, trigger W
(repeating, variable-set 3) counts, trigger Z (one-time, variable-clear 3) counts on a second counter.
Variant: S sets variable 150 instead. *Observable:* counter drops per tick. *Expected:* Z in tick 0. W in
every tick from tick 30 (or 31, if W's tag is polled before S's) to tick 89 (or 90, if W's tag is polled
before C's). Variant: W never. *Falsified if:* W drops only once, outside that range, or in the variant.

**Scenario T13 (claim 20).** *Setup:* a destroyable bridge whose cells carry a one-time tag with the
bridge-destroyed code and a count action. The human destroys the bridge. *Observable:* the tick of the
drop against the tick of the collapse. *Expected:* the same tick. *Falsified if:* no drop, or a drop
without a collapse.

**Scenario T14 (claim 21).** *Setup:* a repeating polled trigger with type-exists (a vehicle type name,
count 2) and one with type-does-not-exist (same name), each with its own counter. The human builds two
vehicles of the type, loads one into a transport, then loses both. Variant B: the name in different
letter case. Variant C: type-exists only, with count 0, naming a vehicle type of which no object
exists during the run. Variant D: type-exists only, with count 1, naming an infantry type; the human
has exactly one infantry unit of that type and garrisons it in a pre-placed neutral building, then
takes it out again. *Observable:* the ticks in which each counter drops, against the number of
such vehicles. *Expected:* type-exists drops in every tick with at least 2 vehicles of the type, the
one in the transport included. Type-does-not-exist drops in every tick with none. In variant B neither
ever drops. In variant C the counter drops in every tick from tick 0, because the common setup's
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
weapon's registration index, with a neutral vehicle on the waypoint's cell. Variant B, on a map whose cell
(0, 0) is legal: the waypoint slot names an undefined waypoint marker, with a neutral vehicle on cell
(0, 0). *Observable:* the vehicles' health loss, compared with the named weapon's damage against the
vehicle's armour by the combat rules. *Expected:* the loss matches that weapon's single hit. In B, the
vehicle on cell (0, 0) takes it. Variant C, with the random-state recorder: a weapon whose warhead selects a damage
animation for the cell's land and whose area damage takes a draw (S-0303); the recorder's log shows
the draws of the animation's creation before those of the damage. *Falsified if:* no damage, a loss
that does not match the weapon, or in variant C the damage's draws before the animation's.

**Scenario T18 (claim 29).** *Setup:* computer-player infantry firing at a target. A polled trigger
makes the computer player's country cheer. *Observable:* the infantry's sequence and the tick of its next
shot. *Expected:* the cheer starts in the trigger's tick, and the next shot comes only after it ends.
*Falsified if:* no cheer, or a shot during the cheer.

**Scenario T19 (claim 31).** *Setup:* a polled trigger with timer-set *v* = 4 and elapsed time 1, and a
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

**Scenario T21 (claim 36).** *Setup:* named test data as in S-0109's common setup: a test
expansion archive whose art database copy adds an animation type R with a resource chain reaction
and a start sound, and no damage. A polled trigger (elapsed time 1) plays R at a waypoint marker
whose cell holds resource, and a second polled trigger (elapsed time 2) uses the drop-zone action at
a waypoint marker in shroud, far from both players' units. A third trigger plays R at a waypoint
marker on a high bridge. A fourth plays R at a waypoint marker on an edge-slope cell (S-0106), next
to a flat cell of the same height level whose own marker gets R from a fifth trigger. *Observable:* the resource in the first cell after tick 15 (the recorder
reads the cell's resource); whether R's start sound is heard in tick 15 (by ear); the shroud around
the second marker after tick 30 (screenshot), against the stock drop-zone radius read from the rules, and
the third animation's drawn height; the drawn heights of the fourth and fifth animations
(screenshot). *Expected:* the resource is gone from the first cell after tick
15, and the start sound is heard; a disc of *r* quo 256 cells around the second marker is revealed
to the human player; the third animation is drawn at the bridge's height; the fourth is drawn half
a height level (the terrain height lift of S-0101 claim 4, halved) higher than the fifth, as the
centre height of an edge slope (S-0106 claim 2) gives. *Falsified if:* the resource stays, the sound
is not heard, the reveal is missing or has another radius, the third animation is drawn on the
ground below the bridge, or the fourth and fifth are drawn at the same height.

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
and the variant only checks that `none` does not act like `<none>`: a second firing in the ticks that are multiples of 8 falsifies the claim.
*Falsified if:* (a) to (d) show no second firing in the ticks that are multiples of 8, or (e) shows
one.

**Scenario T23 (claim 34).** *Setup:* with the random-state recorder. A map in which: one cell
carries a cell tag naming tag A (one polled trigger with a random-delay event); one pre-placed
vehicle carries tag B (the same kind of trigger); tag C is named by a cell tag and, later, by a
pre-placed object; a purely polled tag D with a random-delay event is carried by nothing; tag E is
polled and defined last in the tag list. A pre-placed vehicle and a pre-placed building each carry a
further tag with a random-delay event. *Observable:* the recorder's log of the stream state before
the first tick, replayed (S-0202) into the sequence of draws, with the draws of the other loading
steps (S-0009, S-0010, S-0109, S-0202) identified by count, and the drop ticks of the counters, which
identify the arming draws. *Expected:* the arming draws of A and C come at their cell tags, in
cell-tag entry order, before the draws of the first terrain recalculation pass; for each pre-placed object the creation
draw comes before the draws of its tag; the vehicle entries come before the building entries; D and
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

## Open questions

- **Trigger-played animations (for S-0109).** S-0109 says that a trigger-played animation has no
  start sound and no chain reaction. By claim 36 that holds only for the start events after its
  creation: its first start event, at its creation, is not under the mark. Scenario T21 discriminates. To be
  settled with the owner of S-0109.
- **Lobby control for the computer difficulty.** Whether the stock skirmish and multiplayer lobbies set
  the session computer difficulty (claim 4), from which control, and whether the value restored at a
  later game start replaces a lobby choice.
- **Carry-over flag.** Its use in captures and in one vehicle-destruction case (Import mapping, I1) is
  not specified.
- **Codes outside this subset.** Every event and action code not in tables I3 and I4 is parsed by
  Import mapping, and its meaning belongs to a later revision. That includes the extra latch rule
  (a few codes of the latchable set, the build-type events for example, latch on any repeat type)
  and how polling (claim 6) treats a code that is both polled and code-matched (code 50).
- **Carrier count of a lost carrier.** Whether an object that is destroyed, or leaves the map
  otherwise, while it still carries a tag stops counting as a carrier of it (claims 2, 10 and 19).
  It matters for all-carriers tags.
- **Re-raises outside the subset** (claim 8). The exact codes and conditions of the extra raises at a
  carrier's destruction, at a unit's arrival, under a unit in the air every 16 ticks, at an attack
  on a building (claim 35) and at team events.
- **Destroy attached objects on a cell.** What the action does to the cell that raised the tag.
- **Type-exists name lookup across kinds** (claim 21). Which type the type-exists and
  type-does-not-exist events use when types of two kinds share a type name.
- **Registration order of trigger definitions.** A trigger reference shorter than 3 characters
  names the trigger definition with that registration index (Import mapping, I2), but this revision
  does not state the order in which trigger definitions are registered.
- **Entered-by on a high-bridge cell** (claim 18). That only units on the bridge deck raise the tag
  of a cell that carries a high bridge is not covered by scenario T3.
- **Order of the owner-house raises** (claim 35). The house order between houses, and the reverse
  registration order within one house, are not covered by scenario T20.
