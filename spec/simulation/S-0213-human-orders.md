---
id: S-0213
title: Human orders (the order set, when orders apply, and what each order changes)
topic: simulation
revision: 94
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-01
measurements: []
requests: [Q-0072, Q-0087, Q-0277, Q-0282, Q-0286, Q-0391, Q-0393, Q-0501, Q-0530, Q-1750, Q-1753, Q-1986, Q-1988, Q-2136, Q-2137, Q-2138, Q-2139, Q-2166, Q-2191, Q-2192, Q-2193, Q-2226, Q-2227, Q-2228, Q-2229, Q-2230, Q-2247]
supersedes: []
---

# S-0213: Human orders

## Summary

The orders a player can give during a game, including those that key commands create: what an
order carries, when and in which order the orders of one tick are applied, which checks each must
pass, and which of the object's behaviour, target, destination and guard spot it changes. How a
click on the battlefield becomes an order is the subject of S-0214; our engine's AI gives its own
objects orders through the same set, so this spec also defines that interface.

## Terms

Glossary terms are used as defined there, in particular *tick*, *house*, *human-controlled*,
*local player*, *local area*, *cell*, *building*, *infantry*, *vehicle*, *aircraft*, *object*,
*type*, *field*, *weapon*, *scatter*, *credits*, *scenario random stream*, *departure*, *quirk switch*,
*measurement setup* and *recorder*, and *computer-controlled* as the opposite of
*human-controlled*. *Behaviour*, *target*, *destination*,
*guard spot*, *queued destinations*, *guard reset*, *mind control* (a takeover that changes the
controlled object's owner to the controller's house), *slave master*, *harvesting slave* and
*mobile actor* (an infantry unit, vehicle or aircraft) are used as S-0215 defines them, and the
behaviour names (*move*, *attack*, *enter*, *capture*, *eaten*, *harvest*, *return*, *area guard*,
*guard*, *unload*, *sabotage*, *patrol*, *attack move*) are S-0215's. *Spawned aircraft* and
*slave unit* (S-0214, Terms, *spawned aircraft type, slave unit*), *garrison* and *mind-controlled*
are used as S-0214 (Terms) defines them; *garrison occupants* are S-0508's occupants
(Terms), the infantry inside one garrisonable building. Whether a permanent takeover that leaves no controlling object counts as
mind-controlled is open in S-0214. Further terms are used as the specs named here define them:
*queued behaviour* (S-0215, claim 12); *scatter request*, with its *threat point*, *forced*,
*move-anywhere* and *effectively forced* (S-0208, Terms and claim 11); *reach region* and the
*nearby-cell search* (S-0212); *entry outcome* (S-0209); *deploying* (S-0107, Terms); and
*movable*, *able to fire*, *deploy check*, *deploy readiness* and *in-place deployer* (S-0214,
Terms).

- **AI house.** A house of our engine that our own AI plays, in place of the original's computer
  player. By decision of the project lead (2026-10-09), an AI house has the same unit rules and the
  same order legality as a human-controlled house; the batch's glossary (*AI house*) records the
  decision. For this spec that means: its orders are applied although the original
  drops orders in a computer house's name (claim 3); it is held to S-0214's input rules when it
  issues an order (claim 3); An AI house has the same pending-order limit (claim 2) and the same application checks as a human-controlled house (claims 4 and 15): only sell checks the actor's owner, and the other listed orders do not (claim 15); it counts as human-controlled wherever this spec tests that (claims 9, 11, 12 and 20); and it has no teams (table T3, part c).
  This is a rule of our engine, not a claim about the original.
- **Scripted input.** Mouse and keyboard input that a script produces through the game's normal
  input only, as a player would, never writing into the game. It is an allowance of the
  measurement setup, added by the project lead's decision of 2026-10-09 (Measurement scenarios,
  preamble).
- **Team member.** As S-0215 (Terms) defines it: an actor that belongs to a team of the original's
  computer-player script system. An AI house has no such teams (*AI house* above).

- **Order.** A request from one house that the game applies at a tick boundary (claim 1). An order
  has a *kind* (the rows of table T1), the issuing house and, depending on its kind, an *actor*,
  a *behaviour*, a *target* and a *destination*.
- **Actor.** The object an order is meant for.
- **Behaviour order.** The order kind that names a behaviour for its actor (claim 5). The other
  kinds are named after what they do (stop, scatter, deploy and so on).
- **Existing, live, on the map, valid.** An object *exists* from its creation until it is finally
  deleted. It is *live* while it exists and has not been destroyed. It is *on the map* unless it is
  carried inside a transport or building, has not yet appeared, or is being removed. A *valid*
  object is live, on the map and has more than zero hit points. Table T1 says which of these each
  order kind requires.
- **Unit or building.** An infantry unit, vehicle, aircraft or building, as opposed to other
  objects such as terrain objects, overlays or animations.
- **Contact.** As S-0215 (Terms) defines it: the one pairing a mobile actor can
  hold with another object while the two carry out an exchange (a harvester with its refinery, a
  unit with the transport or building it is entering, and so on). "Breaks off contact" means the
  actor ends that pairing.
- **Tether link, tethered.** As S-0208 (Terms) defines the tether link: a firm link
  that both objects in contact take when one of them gets notice from the other that it begins a
  loading or unloading manoeuvre, clearance to unload, or a request to dock at a refinery, and
  lose when that manoeuvre ends or the link is broken off. S-0417 identifies ground vehicle
  factory release and successful infantry factory or cloning-building release exchanges
  (claims 1, 7 and 10). A docked aircraft is tethered at release unless its type must return
  to a dock (S-0417, claim 11); landing and take-off timing remains provisional,
  and other exchanges remain open.
  While tethered, the unit ignores stop, scatter and deploy
  orders (table T1), and a behaviour order keeps the link, except at a building of the unloading kind (table T3, part b).
- **Inside a tunnel.** The state of a mobile actor that is travelling through a tunnel, a map
  feature whose movement belongs to the movement specs.
- **Usable area.** As S-0211 (Terms) defines it: the part of the map inside the local area that
  objects may occupy. It is the area to which S-0214 limits clicks.
- **Route mode, planned route, route holder.** Route mode is the input mode, entered by holding
  the route-mode key, in which clicks build a *planned route* (a list of places, each with an
  action) for the selected objects instead of ordering them at once (claim 16). An object whose
  planned route holds at least one point, being built or being followed, is a *route holder*
  (which objects can become one, and by which presses, is claim 16d's). A
  *route point* is one place of a planned route; it belongs to the house and can be shared by
  several routes (claim 16a). It is deleted once no route holds it (claim 16f). An object has a
  *route record* from its first route point until it is removed from play, even while its route
  is empty (claim 16b).
- **Route-holder count.** A number kept for each house, which the route-holder limit of claim 16
  reads. It starts at 0. Applying a point add to an object whose route holds no point raises it
  by 1. It is lowered by 1, never below 0, when an object whose route still holds a point stops
  being a route holder by an order, by a used-up route or by its destruction (claim 16f).
  Nothing else changes it: a point delete never does. In the original it can therefore differ
  from the number of the house's route holders (claim 16f).
- **Selection, selection order.** The local player's selected objects, ordered by S-0214's
  selection-order rule (also S-0701 claim 18), not simply by when each was selected. An object
  whose type's primary weapon has positive damage goes before the existing selection when
  selected; any other object goes after it. Thus positive-primary-damage types come first,
  most recently selected first, followed by the others, earliest selected first. This is not
  a test of the weapon the object currently uses. Selection is input-side state, not game state.
- **Pending list.** The list in which orders wait between being issued and being applied
  (claim 2).
- **Attack-move memory.** What an attack-move order leaves on its actor: that the actor's *move*
  or *attack* is an attack move, and the destination or target it is attack-moving to (claim 6).
  S-0309 claim 23 owns the memory's lifecycle, searches and resumption; S-0215 owns the
  ordinary rest outcomes after the memory is removed.
- **Paralysed.** The state of an object that an electromagnetic-pulse effect has disabled for a
  number of ticks. Its rules belong to the combat specs.
- **Berserk.** The state of an object that a weapon effect has driven to attack anything near it,
  for a number of ticks, ignoring its owner. Its rules belong to the combat specs.
- **Planted bomb.** A bomb that a bomb-planting infantry unit has attached to an object. It
  belongs to the house of the infantry that planted it, the *planter's house*, which may differ
  from the owner of the object that carries it.
- **Primary factory.** The factory building of a house that receives the house's newly built
  objects of its kind, when the house has several such factories (production specs).
- **Rally-capable building.** As S-0214 (Terms) defines it: a factory of infantry or vehicles, a
  cloning building or a repair depot.
- **Cycle, order block.** As S-0201 (Terms, *Cycle* and *Order block*) defines them: an unfrozen cycle takes at most one
  waiting input event and runs one tick; a frozen cycle takes no input and runs no tick. During
  an order block the game accepts no orders from the player.
- **Construction yard.** In this spec, a building whose type has the construction-yard flag
  (Import mapping), the sense of S-0214 and S-0415 (Terms), not S-0215's.
- **Vehicle-like building, packs up.** A building whose type undeploys into a vehicle *packs up*
  when it turns back into that vehicle. *Vehicle-like building* is S-0214's term (Terms) for such a
  building with a 1 × 1 foundation.

## Behaviour

1. **Orders apply between ticks.** In a skirmish game, every order issued during tick *N* is
   applied after all objects have been updated for tick *N* and before any object is updated for
   tick *N* + 1, as S-0203 (claim 7) states for the tick as a whole. The first tick in which an
   object can act on an order is therefore the tick after the one in which the order was issued.
   All orders issued in the same tick are applied together, in the order of claim 3.
   Orders issued for a selection use S-0214's selection order (Terms), not chronological
   selection history; a route-mode press on a point uses claim 16b's list instead (claim 2).
   - **Which tick an input belongs to.** In a single-player game the player's input is taken once
     per unfrozen cycle, before that cycle's tick (S-0201, Terms, *Cycle*), and not while the game waits
     between ticks. Each unfrozen cycle takes at most one input event from the player's waiting inputs, in
     the order they were made; an input event is a key or mouse button going down or coming back
     up. An input made during the wait after tick *N* is therefore issued in tick *N* + 1 if no
     earlier input is still waiting, and several inputs made during one wait are taken one per
     cycle, so they are issued in consecutive ticks. One input event can still issue many orders in
     its tick (a click that orders a whole selection, or a key command, claim 18). This holds in
     the original and in our engine alike; it needs no departure.
   - **Order blocks.** While an order block of S-0201 is in force, input issues no orders at all,
     neither from clicks nor from the key commands of table T2; the input is refused, not
     delayed. Such blocks cover the cycles of ticks 0 to 3 of every single-player game (S-0201,
     claim 14), so the earliest order of a game is issued in tick 4 and applied at the end of tick
     4; they also cover every freeze (S-0201, claim 10) and every chrono flash (S-0201, claim 11).
     A chrono flash that starts during ticks 0 to 3 keeps orders blocked after it ends, until a
     trigger action of the map lifts the block (S-0201, Edge cases and quirks,
     `quirks.chrono_flash_keeps_order_block`). S-0201 owns these rules.
   - Backing: `measurable` (Scenario O1) for "the order takes effect at a tick boundary and no
     object reacts in the tick of the click", and (Scenario O9) for "one input event per cycle".
     The exact tick relative to a single click is `unmeasured`: a recorder cannot see when the
     program read a click. The order blocks are S-0201's claims, `measurable` by its scenarios T6
     and T7.
   - Confidence: high, as for S-0203 claim 7; medium for one input event per cycle

2. **At most 128 pending orders.** The pending list holds at most 128 orders. An order issued
   while it is full is dropped silently and never applied. The list is emptied when the orders
   are applied (claim 1), so the limit applies to the orders issued during one tick. A click
   that orders a selection issues one order per selected object, in S-0214's selection order
   (Terms), so when more than 128 objects are selected only the first 128 are ordered. A
   route-mode press on a point is the exception: its orders go to the objects of claim 16b's
   list (*Who gets a press's orders*), in that list's order, so when the list is longer than
   128 only its first 128 objects are ordered. The key commands of
   table T2 are limited in the same way: once the list is full, every further order of the
   command is dropped. Since an unfrozen cycle takes at most one input event (claim 1), the orders that the player's
   input issues in one tick all come from a single click or key command. The list holds the orders of
   the player at this machine, so in a skirmish game the limit is per human player (confidence:
   medium; how the orders of other players enter in a network game has not been studied).
   - **Every house has the same limit.** In our engine each house, human or AI, has its own
     pending list with the same limit, so that a house's orders fill only its own list. The one
     exception is a detonate order issued in the planter's name (claim 11), which goes into the
     planter's list. An AI house has no exemption: the limit and the switch below apply to it
     exactly as to a human house (Terms, *AI house*). The original's computer houses issue no
     orders, so this is a rule of our engine, not a claim about the original.
   - Backing: `measurable` (Scenario O2)
   - Confidence: medium (the limit is certain; that no other limit on selection size applies
     first is not); medium for a press on a point ordering the first 128 objects of claim 16b's
     list, which is `unmeasured`: no scenario here gives such a list more than 128 objects
   - Quirk: `quirks.pending_order_limit` (default: the YR limit of 128). With the switch
     off, no house's pending list has a limit.

3. **Order precedence within a tick.** For one house, non-behaviour orders have
   issue-order precedence among themselves, and behaviour orders have issue-order precedence
   among themselves. Results, including application checks, are those against the effects
   of preceding orders under that precedence. The whole non-behaviour group has precedence
   before the behaviour group. Thus when successful orders in the two groups change the
   same actor state, the behaviour order determines that state, regardless of issue order.
   Enter orders follow this same precedence: a later order sees any contact taken by an
   earlier order, even when both came from one selection click. Thus two vehicles sent
   together to a free repair depot do not both see it as free: the first takes contact,
   and the later order has S-0314 claim 23's occupied-depot outcome. Their order is issue
   order, hence selection order for that click (claim 1).
   **Air-strike cancellation at applied target changes.** S-0519 claim 19 applies to
   each target change produced under this precedence, not just to the target left at the
   boundary's end. Changing a designator's target from A to B and back to A within one
   tick still cancels its strike; restoring A does not restore the aircraft's attack
   orders, lost ammunition or forgotten strike target. An order that assigns the identical
   current target does not cancel the strike. Issuing an order without an applied target
   change does not itself cancel it. These effects inherit S-0519 claim 19's high
   confidence and backing (Scenario T19); the same-tick order conflict is an engine
   consequence, not an established normal-input oracle fixture (claim 1).

   This specifies outcomes and checks, not an internal work schedule. Thus a stop issued after a move to the same actor in that tick leaves the move in force. A person's input in the original cannot produce that conflict, because each tick's orders come from one input event (claim 1); it can arise through our engine's AI houses, whose orders follow the same precedence rule.

   In our engine, house creation order (S-0009 claim 19) supplies between-house
   precedence: a later-created house's orders have the results they would have against
   the state including all earlier-created houses' order effects at that boundary.
   A later house's successful changes therefore prevail over conflicting earlier-house
   changes, but an order whose actor an earlier house removed fails its actor check.
   Earlier-created houses' order-caused production starts receive earlier scenario-stream
   values than later-created houses' starts. These are requirements on resulting state
   and draw assignments, not on internal scheduling. This between-house extension is a
   deterministic design rule, not a finding about the original's computer production.
   - **Production-start draw assignment.** S-0404 claim 11 owns the creation draw at each
     successful start. The association of starts with values follows their causing orders'
     precedence, not the lines' creation or payment priority. Replacements caused by
     automatic cancellations therefore receive values in S-0411 claim 13's tab/icon and
     per-type cancellation order. A replacement that starts and is cancelled by a later
     order at the same boundary still accounts for one creation draw; removing a queued
     type before it starts accounts for none. Other draw-taking actions with precedence
     between two starts have their values between those starts' values, with no values
     reserved for a line in advance. A turn-pass start receives an earlier value than
     order-caused starts at that tick's end (S-0203 claim 7). In our engine an earlier-created
     house's order-boundary starts receive earlier values; within a house, starts caused
     by earlier-issued production orders receive earlier values. The original's computer production is separate (S-0404 claim 12).
   - **Which houses' orders are applied.** In the original only the orders of human-controlled
     houses are applied, and an order issued in the name of a computer-controlled house is dropped
     silently. The original's computer houses never issue orders: its computer player changes its
     objects' behaviours directly, with no order. So the only observable effect of the drop is on
     the detonate order of claim 11.
   - **AI houses (our engine).** An AI house acts only through orders, and its orders are applied
     like those of a human-controlled house (Terms, *AI house*). Before it issues an order, it
     is held to the same legality rules as a person's input (S-0214): an order that S-0214 would
     not let a person give for the same object and target, such as an order to an object that is
     not movable or not able to fire for the action, to a slave unit or to a berserk object, or to
     an object of another house, is not issued. When applied, its orders pass the checks of table
     T1 and of claim 4, with no exemption. What the original's computer player achieved by changing
     behaviours directly is recorded in the specs (S-0215) as behaviour of the original, which our
     AI may choose to reproduce by giving ordinary orders; no engine rule gives an AI house's units
     a behaviour that a human house's units would not get.
   - Backing: the grouping is `unmeasured`: no input of the original issues a behaviour order and
     a later order of another kind for the same actor in one tick (claim 1), so no scenario can
     observe it. The drop of a computer house's orders is `measurable` (Scenario O11).
     Enter-order contact precedence is `measurable` (S-0314 H18 f).
     Production-start assignment is `measurable` (S-0404 P11 B/C and S-0411 S9, referenced
     in Measurement scenarios). Between-house order-boundary assignment is `unmeasured`:
     it is a design extension, not an oracle outcome. The other rules for AI houses are also
     design rules, not findings about the original.
   - Confidence: medium (the grouping is clear; that nothing else reorders orders of the same
     house is not certain); medium for production-start assignment, as in S-0404 claim 11

4. **Target and destination checks.** Every check of table T1 is made when the order is applied,
   against the state at that tick boundary, and an order that fails a check of its kind has no
   effect apart from ending the actor's planned route (claim 17). Besides needing a valid actor,
   a behaviour order fails its checks, and so has no effect apart from claim 17, when the target
   it names, or the destination it names, is a unit or building that is no longer valid. A target
   or destination that is a cell, or an object that is not a unit or building, is not checked. Since the orders are applied after the
   tick's object updates, an object destroyed during tick *N*, or an actor that entered a
   transport during tick *N*, already fails these checks for an order issued in tick *N*.
   - Backing: `measurable` (Scenario O13) for a target destroyed in the tick of the order and for
     an actor that has just entered a transport. The difference between *live* and *valid* (an
     object that is live but has no hit points left) is `unmeasured`: no setup can hold an object
     in that state at a tick boundary. The absence of validity checks on cells and non-unit,
     non-building references is `unmeasured`: O13 supplies no invalid reference of those
     kinds that normal input can name and preserve until application.
   - Confidence: high for the actor checks of table T1; medium for the target and destination

5. **Behaviour order.** A behaviour order names a behaviour, and optionally a target and a
   destination. Its effect on a valid actor whose target and destination pass claim 4 is given by
   table T3 in State and ordering. If part (a) of that table holds, the order is ignored (it still
   ends the actor's planned route, claim 17) and parts (b) to (h) do not apply; otherwise each part from (b) to (h) whose condition holds
   applies. An attack-move order has the effect of the plain order that claim 6 makes of it, and
   "the resulting behaviour" in table T3 means that order's behaviour. Every destination
   assignment uses S-0107 claim 15's repeated-current-nonempty-destination get-up rule
   (claim 21 here), regardless of the behaviour named by the order.
   Generic slave recall in table T3 part (h) excludes buildings, including a deployed
   slave miner, and excludes a resulting *attack* behaviour even for a mobile master.
   A building's separate packing-time recall is S-0402 claim 28's, not an effect of
   applying an ordinary behaviour order here.
   - Backing: `measurable` (Scenario O4 part ii b for these recall exclusions,
     Scenario O3) for parts d, e, f (step 7 for its passenger clause) and
     the guard-spot part of g, and `measurable` (Scenario O4) for parts a and h and for the
     contact break of part b at a building of the unloading kind (Scenario O4, part iii, with a
     named test rules file). Keeping a tethered actor's link at a building that is not of the
     unloading kind, and breaking that link when the building is of that kind, are
     `measurable` (Scenario O18 a, factory release). The untethered-contact branch remains
     `unmeasured`: factory release establishes a tether, not a separate interval of untethered
     contact in which an order can apply. Scenario O4 iv still explores the refinery case.
     Forgetting the building the actor was waiting in line to enter is `unmeasured`: it changes
     nothing that the recorder reads until a later docking request, whose rules belong to the
     exchange specs. Part c is `unmeasured`: it needs a team of
     the original's computer player, which the measurement setup cannot form by input, and it never
     applies to an AI house. What parts e and g do to the queued destinations is `unmeasured`: no
     input rule studied gives an object queued destinations (S-0215 owns how they arise), so the
     measurement setup cannot fill the list first.
   - Confidence: medium

6. **Attack-move orders.** Input creates an attack-move order with a target or with a
   destination, and only for actors whose type is eligible (S-0214, claim 5). Eligibility is
   decided again when the order is applied. The order then acts as the plain behaviour order in
   the table below, and claim 5 applies to that order with the order's own target and destination.
   Any other behaviour order given to a mobile actor removes its attack-move memory.

   | The order carries | Mobile actor, type not eligible | Mobile actor, type eligible | Building |
   |---|---|---|---|
   | a target only | *attack* | *attack*; the memory holds the target | *attack move* |
   | a destination only | *move* | *move*; the memory holds the destination | *attack move* |
   | both | *attack* | *move*; the memory holds the destination | *attack move* |
   | neither | *move* | *attack move*; the memory is removed | *attack move* |

   As far as studied, only the rows "a target only" and "a destination only" of the eligible
   column arise from input (confidence: medium; whether an attack-move click on an object can carry
   both a target and a destination has not been checked). Losing eligibility between issue
   and application can expose the ineligible-mobile column. No input path is established
   here for the neither-reference row, and for the building column only through a route: a
   building that holds a loop's *move* point when the loop's attack-move change is made gets an
   attack-move order with a destination there (claim 16c, *Attack moves in a loop*). Outside
   that path those cells define otherwise unreachable orders, not a claim that an eligibility
   change alone can create them.
   An AI house follows the same input rules (claim 3) and cannot issue those orders directly.
   The table defines them so that every order has a result.
   An attack-move order is distinct from the standing behaviour named *attack move*:
   the ordinary eligible target-only and destination-only orders produce *attack* and
   *move*, respectively, with memory, not that standing behaviour. S-0215 claims 5 and 12
   already make this distinction and limit the data-only conclusion to the ordinary-input
   cases studied. An attack-move order to a mobile actor whose type is not eligible leaves any attack-move memory it already had unchanged.
   - Backing: `measurable` (Scenario O3, steps 5a and 5b) for the two eligible rows that input
     creates: the recorded behaviour of an attack-moving unit is *move* with the cell as its
     destination (destination only) and *attack* with the object as its target (target only). What
     the attack-move memory holds is not recorded by the measurement setup, so the "memory holds"
     parts of the table and the removal of the memory by another behaviour order are `unmeasured`
     (S-0309 claim 23 owns the memory's lifecycle, searches and resumption; S-0215 owns
     ordinary rest outcomes after removal). The cells with no established input path are `unmeasured`,
     since the measurement setup cannot issue them. The building column's route path is
     `unmeasured` too: no scenario gives a building a *move* point in a loop (claim 16c).
     Applying the translation at the time of
     application, not of issue, is `unmeasured` for the same reason.
   - Confidence: high for the translation, medium for which payloads input creates.
     later memory behaviour has the confidence of
     S-0309 claim 23, with ordinary rest outcomes owned by S-0215

7. **Group destinations.** When, within one house's behaviour orders of one tick, two or more
   orders in a row name the same behaviour, the same target and the same destination, and that
   destination is a cell (the *common cell*), the destinations of that run are spread, so that the
   group does not try to stand in one cell. Orders are compared as they were issued, by the
   behaviour they name, before claim 6 turns an attack-move order into a plain one: an attack-move
   order and a plain move order to the same cell are different in this comparison and never form
   one run. A *run* is maximal: it takes in every such order in
   a row, so the order before it and the order after it, if any, differ from it in behaviour,
   target or destination. Run order is issue order; for a single selection click this is
   S-0214's selection order (Terms), including its positive-primary-damage types in
   most-recently-selected-first order. Only the mobile actors of the run that are not inside a
   tunnel take part (the *members*); an actor inside a tunnel keeps the common cell. The spreading
   does not check validity: a member that is off the map still takes part (confidence: low), and its
   own order then has no effect (claim 4). The result is as follows.
   - **Leader.** The *centre* of the members is the mean of their positions, per axis in leptons,
     each sum divided by the number of members and truncated toward zero (integers). The *leader* is
     the member whose distance to the centre is smallest; among equal distances the member latest in
     the run wins.
   - **Destination priority.** For deciding which look cells count as *taken*, members nearer to the leader have priority; among equal distances the earlier member in the run has priority. A cell assigned to a higher-priority member counts as taken when determining the destinations of lower-priority members. The leader keeps the common cell.
   - **Placement.** Each member's destination is determined from its *look*: the cells met along
     the line that starts at the centre of the common cell and runs in the direction from the
     leader's cell to the member's own cell, in steps of one cell length (Formulas, *look
     line*). The first point lies one cell length from the centre of the common cell, so the
     common cell itself is never on a member's look. A look has at most 6 points, and the limit
     is counted for each member on its own: every member's look starts afresh, whatever the looks
     of the members placed before it. A cell met twice (on a near-diagonal line two consecutive
     points can fall in one cell) uses up two of the 6 points; this is the original's rule, `unmeasured` and rated medium (Backing). Each cell of a member's look belongs to the first class of this
     table that fits it, judged using the higher-priority members' assigned cells as *taken*.

     | Class | Cell of the look |
     |---|---|
     | *off-area* | Outside the usable area |
     | *other region* | Not in the same reach region (S-0212) as the common cell, for the member's movement zone |
     | *taken* | Already given to another member of the run |
     | *too steep* | 3 or more levels above or below the common cell (a bridge deck counted at its height) |
     | *obstructed* | The member's entry outcome (S-0209) for it is friendly obstacle, enemy obstacle or blocked |
     | *open* | Any other cell |

     So a cell that fits none of the first four classes, and whose entry outcome for the
     member is anything other than friendly obstacle, enemy obstacle or blocked, is open and
     can be given (Q-1750 question 3). A cell that friendly units occupy is one of these where S-0209
     gives it *moving blocker* or *stationary friendly* for the member.

     The entry check of the *obstructed* class is made with these inputs, which S-0209 leaves to
     the situation: its reference height level is the common cell's height level, plus 4 when the
     common cell is on a high-bridge deck, and it has no connecting-step direction, so no
     connecting-step height restriction applies (the *too steep* class has already set the height
     limit). This is the original's rule, `unmeasured` and rated medium (Backing).

     The *end* of the look is its first cell that is off-area, too steep or open; a look may have
     no end within its 6 points. The look is *spoiled* if an other-region or obstructed cell comes
     before its end, or anywhere in it if it has no end. The member's cell follows from its look:

     | The look | Cell the member gets |
     |---|---|
     | Not spoiled, and its end is open | The end cell |
     | Any other look | Its *fallback cell*: the last taken cell before the end (or in the whole look, if it has no end), or the common cell if there is no such taken cell |

     No further search follows a spoiled look: the member gets its fallback cell even when an
     open cell lies beyond the obstacle that a path around it could reach (Edge cases: a switch
     for a search is deferred to the project lead). So two members can
     end with the same cell, and a member can be sent to the common cell itself.
   - **Shared cells.** Members are given their destinations in destination priority, the
     leader first. As soon as a member has its destination, every member later in that priority
     that has none yet and stands in the same cell as it gets the same destination. Such a member
     keeps it: it is not placed again and has no look of its own. A member standing in the
     leader's cell therefore gets the common cell, through the leader. Several infantry units of
     one cell get one cell between them, and the walk-in rules of the movement specs place them
     inside it. The leader is never given a shared destination, since it comes first.

   Distances here are straight-line distances in leptons, computed in floating point with the
   approximate square root of S-0204 and then converted to integers. The line of a placement is
   followed in floating point, and each point on it is converted to a cell (Formulas). Every such conversion
   truncates toward zero, in every rounding mode (S-0205, claim 7); `quirks.chop_rounding` (S-0205)
   covers only the floating-point operations before it. For group-destination purposes,
   members and cells include the effects of orders with precedence over this house's
   behaviour-order group under claim 3, but exclude effects of that group's behaviour orders.
   Thus a building placed by this house's non-behaviour order at the same boundary already
   counts as an obstacle; this house's behaviour orders do not alter the group geometry
   against which their destinations are determined. Runs arise when the destinations that a click gives the selected
   objects (S-0214, claims 2 and 11) coincide, which is typical for a group of one movement zone
   moved onto a usable cell; a mixed group whose members get different nearby cells forms several
   shorter runs, or none.
   - **How often destinations are shared.** On open, flat ground with free cells around the
     common cell, a compact block of vehicles one per cell, each at its cell's centre, reproduces its own shape around the
     common cell: each vehicle's destination is the common cell plus its cell displacement from
     the leader, for blocks up to 4 × 3 and straight rows of up to 13 along a map axis (Scenario
     O10, main part and part f). Vehicles in cells of their own share a destination only when one of them gets
     its fallback cell from its look, typically next to water, cliffs, buildings or other cells of the refused classes
     around the common cell, or, in a row along a map axis, 7 or more cells out along one line
     from the leader, where all 6 points are taken. On a line off the axes a cell met twice uses
     up two points (`unmeasured`, Backing), so a look there can run out of points closer to the leader. Infantry share by the shared-cell rule:
     the members of one starting cell get one destination between them, and members of different
     starting cells can still end with the same cell, as above. A scattered selection of one type, such as loosely
     placed units gathered by a box, follows the same rules: its destinations are decided member by
     member as above, and members share a cell only where a member gets its fallback cell (its look is spoiled,
     ends at an off-area or too steep cell, or has all 6 points taken) or where they start in one
     cell. These rules settle Q-1750 question 6 for any
     selection. For compact vehicle blocks and rows and for the infantry of one starting cell they
     give the exact counts above; for any other layout the count follows from the same rules and
     depends on where each member stands, so no single number holds for every scattered selection.
     On open ground, meaning that no cell that a member's look reaches is off-area,
     of another region, too steep or obstructed, no look is spoiled and none ends at an
     off-area or too steep cell, so members standing in different cells share a destination only
     where a member's look has all 6 of its points taken: at least 6 higher-priority members
     already placed along its line, or fewer on a line off the map axes where a cell met twice
     uses up two points. With no such line, every member standing in a cell of its own gets a
     cell of its own, and only members that start in one cell share. This open-ground rule has
     the backing and confidence of the look limit and the shared-cell rule below (Scenario O10,
     main part and parts b and f).
   - Backing: `measurable` (Scenario O10, main part) for the existence of the spreading, the
     leader keeping the common cell and each vehicle's destination being the common cell plus its
     cell displacement from the leader. `measurable` (Scenario O10, part b) for the shared-cell
     rule, and (part c) for the leader tie-break (equal distances to the centre). `measurable`
     (Scenario O10, part d) for the classes other region, too steep and obstructed, the
     3-level limit, the fallback to the common cell and the absence of a further search, by comparing
     every vehicle's destination with the one this claim predicts from the recorded start cells,
     and (part f, the 14-vehicle row) for the destination priority of nearer members and a
     fallback to the last taken cell. `unmeasured`: the off-area class, since no part has a look
     that leaves the usable area; the order of the classes where a cell fits more
     than one, since in no part does a look meet a cell that fits two classes with different
     outcomes, and the run-order tie-break of the destination priority. In part d's 3 × 3 block
     each vehicle's look runs in its own direction from the common cell, so no look meets another
     vehicle's cell, and no part has two members at equal distances whose looks meet.
     `measurable` (Scenario O10, part e) for the spreading of a run of attack-move orders, and
     (part f) for the limit of 6 points for each member, the first
     point one cell out and how often destinations are shared. That a cell met twice uses up two
     of the 6 points is `unmeasured`: part f's rows run along a map axis, where no cell is met
     twice. The inputs of the entry check (its reference height level and the absence of a
     connecting-step direction) are `unmeasured`: no part of O10 puts the common cell on a
     high-bridge deck or has a look meet a cell whose outcome depends on the step direction. That a
     cell whose entry outcome is *moving blocker*, *stationary friendly*, *cloaked blocker* or
     *closed friendly gate* is open is `unmeasured`: no part of O10 sets out to have a look meet
     such a cell. The following are `unmeasured`:
     that the spreading tests no validity (a member off the map still takes part), because input
     cannot select an object that is off the map; and the run identity of attack-move orders,
     because one input event issues one kind of behaviour order (claim 1), so no input makes a run
     that mixes attack-move orders and plain ones. The mean of the positions (the centre) is not
     recorded directly; the leader that follows from it is (part c).
   - Confidence: medium that spreading happens, that only runs of identical consecutive orders with
     a cell destination are spread, and for the leader, the placement order, the look limit, the
     first point, the fallback, the absence of a further search and the shared-cell rule, all
     pending Scenario O10; medium for the off-area class, the order of the classes, the run-order
     tie-break, a cell met twice using up two of the 6 points, the inputs of the entry check and
     the open class for a cell that units occupy (`unmeasured`, as the backing says)

8. **Stop order.** The order acts only on an actor that is live, on the map, not tethered, not
   under construction and not being sold (outside its construction state, for a building), and not
   in the unidentified state of Open questions (our engine: inside a tunnel, see below). Its
   effects all take place when it is applied, and every row whose condition holds applies:

   | The actor | Effect of the stop order |
   |---|---|
   | Any actor | Its target and destination both become nothing, and it breaks off any contact. Its behaviour is unchanged, except as the harvester row, balloon-hover retention (claim 21) and jumpjet reroute below say |
   | A mobile actor | It has no queued destinations left (claim 5: how they arise is S-0215's) and stops following any planned route. With its order-level destination gone it comes to rest as the movement specs describe, including the hovering vehicle's retained movement destination and the jumpjet reroute below (this spec does not claim that it halts within the same tick) |
   | An actor with spawned aircraft (such as a carrier) | Its spawned aircraft are called off their target |
   | An open-topped transport | Its passengers lose their targets too |
   | A harvester whose behaviour is *harvest* or *return* | It gets the behaviour *guard* at once |
   | A vehicle that controls slave units, whatever its behaviour | Its slaves are sent back to guard it |

   Destination clearing here is order-level clearing. Behaviours and the factory-report
   tests of S-0417 claim 7 read that cleared destination. A hovering vehicle's throttle,
   moving-state tests and completion/search rules instead use its separate movement
   destination (S-0503 claim 16). When that point equals its heading point in all three
   dimensions, stop retains it and the stored path; otherwise stop clears both. In either
   case the heading point and current movement quantities are retained at application:
   clearing the order-level destination is not a forced immediate halt.

   Under S-0503 claims 15–16, equal retained points still complete on arrival when the
   ordinary destination-cell and height condition holds. That completion restores the
   actual-position vehicle-present flag and produces a cell-arrival update even without
   same-cell contact. Unequal points cleared by stop instead use claim 15's no-movement-
   destination arrival outcomes. Either case can permit a final already-computed travel
   step, not an exact-centre stop. The contact break above remains in force: an arrival
   update does not promise a later factory report through an unchanged contact.
   These movement outcomes inherit S-0503 claims 15–16's backing, confidence and
   Scenarios H12 and H15.

   A jumpjet instead uses S-0502 claims 17 and 24's reroute when the stop produces
   an effective destination clear and it has a movement commitment (S-0502 Terms,
   *moving*). Its order-level destination becomes empty, but its separate movement
   point can be replaced and movement can continue. An unchanged vehicle assignment
   can be suppressed instead (claim 21); then no reroute occurs and the preceding
   movement point and commitment survive. Infantry has no such equality suppression:
   an empty-to-empty assignment reroutes again while a commitment remains.
   This is not S-0502 claim 16's direct no-point instruction to the locomotor.

   Stop requests destination clearing while the target is still present. For a
   balloon-hover type it also requests clearing after the target is gone. Under
   S-0502 claim 28, a balloon-hover vehicle can therefore retain its destination at
   the first request, but not at the second: stop ends with an empty destination.
   If the first clear was rejected, the second is effective and reroutes with a
   commitment; if the first was effective, the second unchanged empty assignment is
   suppressed. A first rejection can nevertheless leave *attack* current after stop.
   For balloon-hover infantry, neither assignment has vehicle equality suppression:
   both can reroute if the commitment survives. The rule is per effective assignment,
   not one reroute per stop order.

   | Flight state at stop | Successful reroute outcome |
   |---|---|
   | Ascending | Continues ascending towards the replacement point |
   | Cruising | Continues cruising towards the replacement point, with ordinary arrival |
   | Hovering with a movement commitment | Keeps that commitment and remains hovering at application; on its next update starts cruising if the replacement horizontal point differs from its position, otherwise follows S-0502 claim 13's hover rules |
   | Descending | Releases its landing reservation and re-ascends under S-0502 claim 16, including that claim's deploy-request and *unload*-to-*guard* consequences |

   Without a movement commitment, stop has no locomotor effect on a grounded or hovering
   jumpjet. Both nearby searches of S-0502 claims 17 and 16 apply to rerouting, so the
   replacement need not be the current position; infantry can receive another standing
   position in the same cell. Failure to find a cell has S-0502 claim 17's self-destruction
   outcome for a still-living unit. Later arrival uses the then-current target, including
   this order's target clearing. Stop therefore promises neither an immediate halt nor
   completion at the old movement point. These outcomes inherit S-0502 claim 24's
   `measurable` backing (Scenario J19) and high confidence for stop dispatch and flight-state
   outcomes; repeated infantry clears inherit claim 24's Scenario J23 backing.
   Balloon-hover stop retention, retry and resulting behaviour inherit claim 28's
   high confidence and Scenario J25 backing. The combined balloon-hover infantry
   fixture remains `unmeasured` as in J23. The no-cell destruction retains claim 17's
   medium confidence and Scenario J11.

   The slave-recall row applies only to vehicles: a stop order to a deployed slave-miner
   building does not itself recall its slaves (Scenario O4 part ii b).
   An object whose behaviour has no target and no destination left falls back to rest by the
   rules of S-0215.
   - Interim rule for our engine, where the original is not settled (Open questions): the
     unidentified state of claims 8 and 10 and table T1 is taken to be *inside a tunnel* (Terms),
     so a stop or deploy order to an actor inside a tunnel has no effect apart from claim 17. This
     is a design choice that measurement may revise. Tethered is set and released as Terms says.
   - Backing: `measurable` (Scenario O5) for the clearing of target and destination, the unchanged
     behaviour (step 1), the harvester row (step 2), the passengers of an open-topped
     transport (step 6), the spawned aircraft (step 7) and the slaves of a slave miner whose
     behaviour is *harvest* (step 8); and `measurable` (Scenario O8, game C) for the end of a planned
     route. The tethered exception is `measurable` (Scenario O18 b): factory release supplies
     a tethered vehicle, and an attack target distinguishes an ignored stop from a stop that
     clears references. The following are `unmeasured`: (i) The slave rule "whatever its
     behaviour", because to have the slaves out mining while the miner's behaviour is neither
     *harvest* nor *return*, the miner must first get a behaviour order, which already calls the
     slaves back (table T3, part h). (ii) "No queued destinations left", as in claim 5. (iii) "Breaks
     off any contact", because the exchanges that a stop order may find an actor in are either
     tethered, including the identified factory-release exchange, or not identified as
     untethered (the remaining gap in table T3, part b), so no setup
     separates it from the actor's own end of an exchange. (iv) The checks "not under construction"
     and "not being sold": the actor would have to change into that state between the click and the
     application, and the scenarios cannot hold an object in that state steady.
   - Confidence: medium (high for the clearing of target and destination)

9. **Scatter order.** A mobile actor that is live, on the map and not tethered is sent a scatter
   request in the sense of S-0208 (Terms, *scatter request*): one that names no threat point, is
   forced and is not move-anywhere. Whether the actor accepts it, where it goes and the one draw from
   the scenario random stream that an accepted request takes belong to S-0208 (claims 11 to 13) for
   infantry and to the movement specs for the other kinds. With those rules an infantry unit that
   stands still accepts the request, unless its current sequence cannot be interrupted (with
   S-0208 claim 11's exceptions), or it is
   deploying (S-0107, Terms) and its owner is human-controlled. For a walking unit the request is
   not effectively forced (S-0208, claim 11), so a walking unit refuses it unless it meets S-0208
   claim 11's conditions for accepting such a request (among them, a flag of its type that S-0208,
   claim 11, names).
   Buildings ignore the order.
   - Backing: `measurable` (Scenario O5, step 4; Scenario O18 c for the tethered refusal)
   - Confidence: medium

10. **Deploy order.** The actor must be live, on the map, not tethered, not an aircraft, not
    paralysed, not under construction or being sold, and not in the unidentified state of Open
    questions (our engine's interim rule: claim 8). An actor that stands on a cell occupied by a building whose type is a vehicle
    factory ignores the order. Otherwise the actor breaks off any contact, its target
    becomes nothing, it requests an empty destination and the behaviour *unload* becomes
    its queued behaviour. The destination becomes empty except for the balloon-hover
    retention below. What
    *unload* does depends on the type: deploying into a building, deploying in place, letting
    passengers or garrison occupants out. These consequences belong to S-0215,
    S-0517 claims 1 to 8 (in-place vehicle deploying and deploy-fire vehicles), and the
    special-abilities specs.

    Deploy makes only one destination-clear request, while the target is still present.
    It does not make stop's post-target-clear retry. A balloon-hover vehicle satisfying
    claim 21's retention predicate therefore keeps its nonempty destination, movement
    point, landing reservation and commitment despite ending with no target. Retention
    can make *attack* current before *unload* becomes queued (S-0502 claim 28).
    Deploy does not itself enable destination refresh (S-0502 claim 27).

    For a jumpjet, an effective destination clear has S-0502 claim 24's movement
    consequences, just as an effective stop clear does (claim 8), whenever it retains a
    movement commitment. Infantry has no vehicle equality suppression, so an
    empty-to-empty deploy assignment is effective too. Deploy therefore need not erase its movement point or halt its flight:
    a successful reroute supplies a replacement point, with the flight-state outcomes
    in claim 8's table. A suppressed unchanged vehicle assignment instead retains the
    preceding point and commitment (claim 21). For a landing-first vehicle hovering
    with a commitment, a different replacement horizontal point causes cruise before
    descent; an equal one lets the landing request produce descent. Touchdown uses
    the retained point. With no commitment, the clear does not create one, and the
    landing request alone does not wake idle hover. These are S-0502 claims 13 and 24's
    consequences, not an extra deploy movement instruction.
    - Backing for the jumpjet consequences: `measurable` (S-0502 Scenarios J22 to J24),
      confidence: high for effective-clear dispatch and refresh nonactivation, medium
      for exceptional aftermath. The uncommitted idle-hover starting case remains
      `unmeasured`, as in claim 24: no ordinary stock order sequence establishing it is
      settled. Balloon-hover deploy retention has high-confidence predicates but an
      `unmeasured` stock deployable fixture (S-0502 claim 28, J25 diagnostic).
    - Backing: `measurable` (Scenario O6) for the vehicle-factory cell and for the unit that
      deploys in the way its type says (which shows that *unload* was queued and took effect). That
      the actor also breaks off contact and loses its target and destination is `unmeasured`: the
      recorder would have to catch a deploying unit in contact or with a target, and no setup
      identified here does. The refusals for an aircraft, a paralysed actor, an actor under
      construction or being sold, and a tethered actor are `unmeasured`: input may already refuse
      the order for such an actor (S-0214), or the actor would have to enter that state between
      the click and the application, which the scenarios cannot hold steady. The tethered
      factory-release setup does not isolate the tether refusal: a vehicle still on its factory's
      cell also fails the vehicle-factory-cell check. Infantry release ends its tether at the
      next cell arrival (S-0417 claim 10), and no setup here establishes an input-legal deploy
      order applying during that interval. The identified tether alone does not upgrade this
      backing.
    - Confidence: medium

11. **Detonate order.** If the actor exists and carries a planted bomb, the order resolves the
    bomb at once: it detonates if the carrier is on the map (S-0519 claim 5), but is removed
    silently, with no detonation, animation or damage, if the carrier is off the map
    (S-0519 claim 6). No other order-application check is made. The order is issued in the name of the planter's house, not the house
    of the player who gave it. By claim 3 it is therefore applied only when the planter's house is
    human-controlled: a player who sets off a bomb that a computer house planted on one of the
    player's own units gets no effect, and the order is dropped.
    - **In our engine (departure).** An AI house counts as human-controlled for the checking of
      orders (Terms, *AI house*), so a detonate order issued in an AI planter's name is applied: a
      player can set off a bomb that an AI house planted on the player's unit. The drop of Scenario
      O11 is the original's behaviour against its computer player, which our engine does not
      reproduce under either setting of the switch below. This is a *departure* with no switch,
      decided by the project lead (2026-10-09): AI houses follow the order rules of human houses.
    - Backing: `measurable` (Scenario O6, step 4, and Scenario O11) for on-map orders.
      The off-map application is `unmeasured`: no normal-input fixture is established
      (S-0519 claim 9).
    - Confidence: medium
    - Quirk: `quirks.detonate_in_planter_name` (default: original). With the switch off, the
      order is issued and applied in the name of the house of the player who gave it. Input offers
      the detonate action only on the player's own object (S-0214), so that house is always the
      owner of the object that carries the bomb. In our engine the switch decides which house's
      pending list (claim 2) the order fills and in which house's turn of claim 3 it is applied,
      and the bomb is resolved as above under either setting.

12. **Sell order.** The actor must be live and owned by the issuing house. A building is sold.
    A vehicle or aircraft is sold only if a building occupies the cell it stands on, which in stock
    data means a repair depot. Otherwise the order does nothing. What selling yields belongs to the
    economy specs. A separate order sells the wall in a given cell.
    - **Packing up instead of selling.** A building whose type undeploys into a vehicle is not
      sold: it packs up into that vehicle. A construction yard packs up only when all of these
      hold: its house is human-controlled, the game is not a campaign game, the game option that
      lets construction yards be packed up again is on, the yard is not mind-controlled, and it
      has a guard spot (claim 20). Otherwise it is sold like any other building. This applies to
      every sell order, whichever input created it. Input creates one in two ways: by the sell
      mode of S-0214 (claim 13), which offers *sell* on a construction yard but never on a
      vehicle-like building, and by a move or harvest click on a cell with a selected building
      whose type undeploys into a vehicle, which issues a guard-spot order (claim 20) followed by
      the sell order (S-0214, table M, part 3). Both are orders that are not behaviour orders, so
      by claim 3 the result is that of the guard-spot order followed by the sell order: the sell
      order finds the guard spot that the same click set. An AI house of our engine meets the
      condition "its house is human-controlled" (Terms, *AI house*). How the packing up proceeds,
      and what the vehicle does afterwards, belongs to the special-abilities specs.
    - Backing: `measurable` (Scenario O6) for buildings, (Scenario O15) for packing up, and
      (Scenario O16, part b) for the owner check on a building whose owner changes in the tick of
      the order. For
      vehicles and aircraft the claim is `unmeasured`: no input rule creates a sell order for a
      vehicle or aircraft (S-0214, claim 13), so neither a person nor scripted input can create
      one under the measurement setup. An AI house, held to the same input rules (claim 3), cannot
      issue one either; the rule is stated so that every sell order has a defined result.
    - Confidence: high for buildings, medium for vehicles and aircraft and for packing up; low for
      the guard-spot condition of a construction yard

13. **Building switches.**
    - The *repair* order switches an object's repair on if it is off and off if it is on. It needs
      only a live actor, which may be off the map; input creates it for buildings only (S-0214,
      claim 13).
    - The *power* orders (*power off* and *power on*) switch a building off or on (*switched off*,
      *switched on*).
      They need a building that is live and on the map, and do nothing to a building whose type
      carries the firestorm-wall flag (Edge cases). What the order then changes, and the further
      conditions under which it changes nothing (a power-off order on a building whose type has
      neither a base drain above 0 nor the needs-power flag; a power-on order while a pulse
      disables the building), belong to S-0405 (claim 10).
    - The *primary* order needs a live building, which may be off the map, whose imported
      factory-category field is not the no-factory value. This test includes the four producing
      kinds and other recognised categories of S-0003 claim 21; it is not a production-line
      eligibility test. It is a switch: if the building already has the primary designation, it
      loses it; otherwise it gains it, and every other building of the same house that is on
      the map, has exactly the same imported category and has the same water-or-land kind of type
      (naval or not) loses its designation. A building with the no-factory value is not changed.
      A recognised non-producing category grants no production line even when designated primary.
      Input creates the order only when the player clicks such a selected building of the
      player's on itself (the deploy click of S-0214, row G5) and that building is not already
      primary: a click on a factory that is already primary gives no action (S-0214, row B3). So the
      switching-off case arises only when a second primary order for the same factory is issued
      before the first is applied: a person's input cannot do that (claim 1), but an AI house can,
      by issuing two in one tick. The local player hears a notice when one
      of the player's buildings becomes primary.

    None of these checks the owner of the building when it is applied (see claim 15). What
    repairing, powering and the primary factory change belongs to the economy and production specs.
    - Backing: `measurable` (Scenario O6, step 3, for repair; Scenario O12 for power, the
      firestorm-wall flag and making a factory primary; part b tests producing versus
      non-producing categories). "Off the map" for repair and primary, and
      the switching off of a primary factory, are `unmeasured`: input cannot click a building that
      is off the map, and creates no primary order for a factory that is already primary.
    - Confidence: medium

14. **House orders.** Some orders have no actor (table T1, lower part): place a finished building,
    start, hold or cancel production, fire a superweapon at a cell, change an alliance, surrender,
    change the game speed, and order all infantry to cheer. Each belongs to the spec of its topic:
    the alliance and surrender orders to S-0414 (claims 11 to 14), whose checks and effects table
    T1 repeats only as pointers. This spec only lists them, and fixes that they too are applied at the tick boundary of claim 1,
    in the order of claim 3. The game-speed order arises only in a network game (S-0201, claim
    13): in a single-player game the options menu changes the speed directly, with no order, and
    the new speed takes effect from the next cycle (S-0201, claim 8).
    - Backing: `measurable` (Scenario O1, step 3) for the production order. For the other house
      orders the claim is `unmeasured`: they pass through the same pending list, and none of them
      leaves a per-tick observable that separates the tick of the input from the tick of the
      boundary.
    - Confidence: medium

15. **No owner check on most orders.** When it applies a behaviour, stop, scatter, deploy,
    detonate, repair, power, primary or guard-spot (claim 20) order, the original does not check
    that the issuing house owns the actor. Only the sell order checks the owner. In the original
    only the owner's input can create these orders (S-0214), so the difference shows only when the
    actor's owner changes between the issue and the application of an order: for example by mind
    control, capture or a map trigger's change of owner (S-0011, claim 24) during the tick of the
    order. Such an actor still gets the order, and a building in that case is not sold. Our engine
    applies the same rule to every house, human or AI, with no exemption: the checks of table T1
    are the same for all, and an AI house is held to S-0214's input rules when it issues an order
    (claim 3), which let a house order only its own objects. So a house's orders act only on its
    own objects, with two exceptions that hold in the original too and that our engine reproduces
    for every house, with no departure: the detonate order of claim 11, which names the planter's
    house, and an actor whose owner changes between the issue and the application of an order.
    - Backing: `measurable` (Scenario O16, part a) for an object whose owner changes in the tick of
      the order, which is the case that tells "no owner check" from "owner check" apart; Scenario
      O16, part b, gives the contrast of the sell order. An order to an object that its house
      already did not own when the order was issued is `unmeasured`: no input path of the original
      creates one (S-0214). The rules for AI houses are design rules of our engine and need no
      backing.
    - Confidence: medium

16. **Route mode.** While the player holds the route-mode key, clicks do not order the selection
    directly. Each click adds a route point, with the action it stands for, to the planned route of
    each selected object (claims 16a and 16b). When the key is released, the routes are committed
    and the objects start to follow them (claim 16c). Route points are created, joined, committed
    and deleted by their own order kinds (claim 16a), which apply at the tick boundary of claim 1.
    When a unit falls back to rest while following a route, S-0215 claim 13 specifies when the
    order for the next route point takes effect. Issuing that action is not an immediate
    destination assignment within the fall-back.
    A route-mode press outside every point's click area is refused when a selected object
    cannot take part in a route (claim 16d). A press on an existing point is refused when the
    selection can neither join nor close a loop at it (claim 16b). A click that claim 16d tests
    for the holder limit is also refused when the house's route-holder count (Terms) plus the
    number of selected objects whose routes hold no point would be more than 128 (Formulas). A
    click that takes a point (a join or a loop close), a selecting click, a band box's release
    and a tool click are not tested for it (claim 16d). A refused click adds no route point.
    Since a press that takes a point is not tested for the holder limit, joins can give a house
    more than 128 route holders. A point join also does not raise the count (claim 16f), as far
    as studied (low).
    The refusal gives a message and the game-wide refusal sound. Multiple route-mode refusal
    notices before the next tick completes share one message-and-sound cue, but completing a
    tick permits a new cue: a refused click made seconds later in the same running game is not
    silent because of an earlier refusal (S-0701 claim 24). Claim 25 specifies the message's
    imported text, lifetime and shared display capacity.
    - Backing: `measurable` (Scenario O8, including its repeated-refusal test, and part o for
      a join that is not tested for the holder limit and gives more than 128 holders)
    The same-tick duplicate-notice limit is `unmeasured`: no setup here creates multiple
    independent route refusals before one tick completes. The reset's observable consequence
    on later clicks is tested by O8.
    - Confidence: high for repeated cues after completed ticks and for the limit of 128 on the
      count plus new objects at a click that adds a point; medium for a join not tested for the
      limit and so giving more than 128 holders; low for a join not raising the count. Claims
      16a to 16h carry their own ratings.

16a. **Route orders and who owns a route, Q-1986 question 1.** Every selected object has its
    own planned route: a list of route points, followed in order. Point adds stop at 16 points
    (claim 16b). A join can take a route past 16 points only when the route it copies is already
    longer, or when the copy is added after points of the joining object's own. A selected
    object that joins has no points of its own unless it has lost the pressed point before its
    order is applied (claim 16b, *Who gets a press's orders*). So a route first passes 16 points
    through a copy added after an object's own points: one that a press gives an object that is
    not selected, or a selected object that has lost the point. A join of a route so lengthened
    can pass 16 points too. A route point is a
    place on the map that belongs to the house (claim 16b gives the place of each point add), so
    one click with several objects selected adds the same point to each of their routes whose
    orders have that place. Point adds share a point only when they are applied in the same
    tick: a point add applied in a later tick makes a new point, even at the place of an
    existing one, so two points can stand at one place. Route mode sends four order kinds:

    | Order kind | Names | Effect when applied |
    |---|---|---|
    | Point add | one actor; the behaviour, target and destination that the click gave it | Adds a point at the order's place (claim 16b) to the end of the actor's route, subject to claim 16b |
    | Point join | one actor; an existing route point of the house | Gives the actor a copy of that point and the points after it in the route of one of its holders (claim 16b), added after the actor's own points (a selected actor has none unless it has lost the point before the order is applied, claim 16b), or closes a loop when, at the order's application, the point is in the actor's route (claim 16c) |
    | Commit | the issuing house only | Commits every point of the house's routes and starts the holders that can start (claim 16c) |
    | Point delete | one route point | Removes the point from every route that holds it (claim 16e) |

    A point-add order is the ordinary behaviour order of claim 5 marked as a route point, one per
    eligible selected object and click, in S-0214's selection order. A click on an existing
    point that claim 16b's point-click test accepts sends point-join orders instead, to the
    objects that claim 16b lists (*Who gets a press's orders*). Releasing
    the key sends one commit order for the local player's house.
    - Backing: `measurable` (Scenario O8, parts c and d, for shared points and per-object
      routes, part s for a later point add at an existing point's place, and part e for the 16
      points of point adds). The division into order kinds is
      `unmeasured` as such: only its consequences
      cross the lockstep stream observably, and those are claims 16b to 16e.
    - Confidence: high for one route per object and the 16-point limit of point adds on a route
      that point adds have filled; medium for shared points, for a later point add making a new
      point, and for a selected object that joins with no points of its own passing 16 points
      only when the route it copies is already longer; low for a selected object that has lost
      the pressed point joining after its own points, and for a route first passing 16 points
      through a copy added after an object's own points (claim 16b's ratings)

16b. **What a route point stands for, Q-1986 question 2.** A route-mode click over an object
    or a cell stands for the action that the same click would order outside route mode under
    S-0214 for each selected object: *move* to a cell, *attack* on an enemy object, *enter* a
    transport or building, *capture*, *area guard* and the rest. For a vehicle or infantry unit
    the deploy action is refused in route mode (S-0214 row I22 gives an in-place deployer no
    deploy order in route mode).
    Of the actions that a click gives a selected vehicle or infantry unit, every one that orders
    something, except the detonate (claim 11), gives a behaviour order, which route mode turns into
    a point add. An action that S-0214 gives no order for gives none here either and adds no point.
    A click that selects or toggles the selection
    gives the unit no action (S-0214 claim 3), so it adds no point. Claim 16h's tool clicks are
    not counted among those actions.
    *Aircraft and buildings, Q-2192 question 3.* The same holds for the actions that a click
    gives a selected aircraft: each that orders something, except the detonate, gives a
    behaviour order and so a point add. An aircraft's deploy action is not refused for kind, but
    it adds no point: S-0214 gives an aircraft the deploy action on an object only when it
    carries passengers (its table AC, row AC7), and that action orders the behaviour *unload*
    (its part 1 table), a behaviour order that names no target and no destination, so it has no
    place (below), and when it is applied it adds no point and does nothing else (the aircraft
    does not unload). Of a building's actions, the *attack* action gives a
    behaviour order and so a point add. A click on a cell that sets a factory's rally point, or
    that undeploys a building, sends that order directly: it is not a behaviour order, adds no
    point and takes effect when applied, as outside route mode. It meets claim 16d's
    release-time tests and is sent at the release, as the detonate below is. A building's other
    actions on an object (S-0214 table M, part 3) go the same two ways. Sent directly, with no
    point, are: its *move* on an object, which sends the guard-spot order of that table (chosen
    around the object's cell for a factory of infantry, vehicles or aircraft, naming the object
    for any other building), which then acts as claim 20 says when it is applied. Also its
    *detonate*, which sends its detonate order (claim 11), and its *deploy*'s primary order
    (claim 13) when the building is factory-marked. Each meets claim 16d's release-time tests
    and is sent at the release, as the clicks above are. In the original, by this spec's reading
    (low, not S-0214's), a building is given its *deploy* action only by a click on the building
    itself, and each behaviour order *unload* that the action gives names the building itself as
    its target. It is not the target-less *unload* of the aircraft above or of S-0214 table M
    part 2, so it has a place, the building's own, and is a point add there. The points it adds
    are issued as claim 16d says. The direct sends have no issue step.
    A building's deploy action is not refused in route mode. In the original, a click gives an
    aircraft or a building these actions only by a press inside a click area that the press
    neither takes nor refuses, since any other press is refused for kind (claim 16d).
    A route-mode click that would order a detonate adds no point. It meets claim 16d's tests as
    an ordinary route-mode click does: the kind test when the button is pressed outside every
    click area, and the same-route-state and holder-limit tests when it is released. When all
    pass, the detonate order is sent at the release, as outside route mode, with no route-mode
    message, and that order ends the carrier's route when it is applied (claim 17). When one
    refuses, the click is refused with the message that claim 16d gives, and no detonate order
    is sent.
    The point stands at the order's *place* (S-0214 claims 10 and 11). That is the place of the
    target that the click gave the order. A *move* order names no target, and its place is the
    clicked cell's, or, for a click on an object, that of the cell the object stands on at the
    click. This holds even when the move's destination is another cell, found by the nearby-cell
    search of S-0214 claim 11: the point is then on the clicked cell, whatever the move order's
    destination. Any
    other order that names no target has its destination's place, so a *patrol* click on a cell
    puts the point on the cell that the search finds. An attack-move click (claim 6, Q-2230) has
    the place that the same click gives without attack-move entry: entry changes only the
    order's kind, not the target, destination or cell that the click gave it. So an attack-move
    click on a cell puts the point on the clicked cell, not on the cell the search finds. One on
    an object puts it at the target's place when its order names a target, and on the cell that
    the object stands on at the click when its order would be a *move* without entry. An
    order that names neither a target nor a destination has no place: when it is applied it
    adds no point, with no message and no sound, and has no other effect. For each holder the
    point records the order that the click gave that holder: its action, with the click's
    target and destination (medium).
    One click can give the holders of a shared point
    different actions: an engineer *capture* and a tank *attack* on the same enemy building.
    Each holder later issues its own recorded order, and the checks below read the holder's own
    action at its route's last point. The order that a holder issues for a point is the one
    recorded for it at the add, or copied to it at a join (*Joins*), as changed by a loop's
    attack-move change (claim 16c). It is not classified anew when it is issued: a point on an
    object issues the action that the click gave over that object, even when the object or the
    holder has changed since in a way that would now give another action, and a point with no
    object issues the order that the click gave on the cell (medium). So a vehicle's point on a
    repair or reload building of its own house or an allied one issues the order that the click
    gave it there, whatever the vehicle's health and ammunition when the point is issued: a
    full-health vehicle's point on an own repair building that accepts it issues *enter*, since
    S-0214's table V (row n3) gives a ground vehicle *enter* on such a building at any health
    (medium, `measurable` in Scenario O8 part ab, Changes for other specs, S-0215 claim 13 and
    Scenario B40).
    S-0214's *patrol* action (row GC3) is the action for a cell that holds a route point when
    force-move and force-fire are held. A press inside a point's click area is taken by the
    point first, whatever keys are held (*Clicks on existing points* below), so the patrol
    action is given only by a click on the cell outside every click area, or inside one that the
    press neither takes nor refuses. That click is an ordinary point add with the *patrol*
    action. *Patrol* is neither terminating nor continual, so later points can be added.
    A point add is refused, when the order is applied, in each of these cases:
    - the route's last point is *terminating*: its action is *enter*, *capture* or *eaten*,
      after which the holder has left the map or the route cannot go on.
    - the route's last point is *continual*: its action is *guard*, *attack* on a cell, or
      *area guard* on a cell or with no target, which never finishes by itself.
    - the route has a loop (claim 16c).
    - the route already holds 16 points (a longer route is not refused for its length: *Who
      gets a press's orders* below).

    *Clicks on existing points.* With objects selected, a route-mode click on the marker of an
    existing route point of the house is not a point add. It is refused, with the invalid-point
    message (Import mapping) and the refusal cue of claim 16, when any selected object that has
    a *route record* meets one of these conditions:
    - its route has a loop.
    - the point is not in its route.
    - the point is its route's last point or the one before it (its first place in the
      route, *A point that a route holds twice* below).

    An object gets a route record with its first route point, and keeps it, even once its route
    is empty, until it is removed from play (low). Otherwise the click is accepted, and its
    orders go to the objects that *Who gets a press's orders* below lists. As the routes stand at
    the click, the orders close a loop (claim 16c) for the selected objects whose routes hold the
    point and join it for the others: a selected object with a route is listed only to close a
    loop, and only a selected object with no route record joins. What each order does is decided
    when it is applied, though, and a selected holder can lose the point before then (*Who gets
    a press's orders*). A press on a point is not tested for the same route state (claim 16d),
    so one press can close a loop for some selected objects and join the point for the others.
    While the cursor is over a point that this test
    would refuse, the point is not highlighted as a click target. The test is made at the
    click, so a refused click sends no order.
    - Quirk: `quirks.spent_route_blocks_join` (default: original). With the switch off, an
      object whose route is empty is tested as one with no route record, so an object that
      has used up, lost or emptied its route can join a point again.

    *Who gets a press's orders.* A press on a point that the point-click test accepts, with
    objects selected, sends one order for each object of a list made by going through the
    selection in S-0214's selection order. A selected object with no route record, or with an
    empty route, is listed itself. For a selected object whose route holds points, every object
    whose route holds that route's last point is listed, the selected object among them, in the
    order in which they were given that point, whether they are selected or not. An object
    already listed is not listed again. The orders are applied in the list's order, each on the
    routes as the earlier ones left them. They are issued in the list's order too, so the
    pending-list limit (claim 2) drops the orders of the objects after the first 128 in the list.

    Whether an order closes a loop or joins is decided when it is applied, for every object of
    the list, selected or not: it closes a loop (claim 16c) when the actor's route then holds the
    point that the order acts on (*Click areas* below), and joins that point otherwise. Neither
    is tested then for the point-click test's refusals. A selected object's route mostly stands
    at its order's application as at the click, so its order does what the test found. But a
    selected holder's route can change first: for example, the holder can issue the pressed
    point (claim 16c) in the tick in which the press's orders are applied, since they are
    applied after every object's update (claim 1). Claim 16f deletes the pressed point once no
    route holds it, and the order then acts on a later-made point of the house at the place, if
    any. When the holder's route still holds the point that the order acts on, at a later place
    (*A point that a route holds twice* below) or as a later-made point at the place, the order
    closes a loop there with no test, so the loop can begin at the route's last point or the one
    before it and hold fewer than 3 points (low). When its route does not hold that point, the
    order joins it: the holder is given the copy that *Joins* below gives a joining holder,
    added after the points it still holds, as for an object that is not selected below, so its
    route can pass 16 points (low). If no point of the house is left at the place, the order
    does nothing (*Click areas* below) (low).

    So an object that is not selected also gets an order when its route holds the last point of
    a selected object's route. The point-click test is not made for an object that is not
    selected:
    - when its route holds the pressed point, the press closes a loop there for it (claim 16c),
      from the pressed point's first place in its route to its route's last point. This holds
      even when the pressed point is that last point or the one before it, and the new loop
      replaces any loop its route already has (low).
    - when its route does not hold the pressed point, it is given the copy that *Joins* below
      gives a joining holder, added after its own points, whatever their number, so its route
      can pass 16 points. Its route then has the copy's loop in place of any loop it had, or no
      loop when the copy has none. The loop begins at the first point of its route that is the
      copied loop's first point. When that is the joined point, which its own points do not
      hold, the loop is the copy's own, also when its own points hold other points of the
      copied loop. When the copied loop begins at a later point that its own points also hold,
      the loop begins at that own point and has as many points as the copied loop, so it ends
      before its route's last point (low). How a holder goes round such a loop, and what a
      join copies from it, are claim 16c's (*A loop that ends early*) and *Joins* below.
    - a route that holds more than 16 points is not refused a point add for its length: the
      length refusal above is made only at 16 points (low).
    - Quirk: `quirks.press_copy_ignores_route_state` (default: original). With the switch off,
      a join that a press gives any object of its list, selected or not, does nothing when the
      copy would take its route past 16 points, and the copy's loop always begins in the copy,
      at the copied loop's first point.

    *A point that a route holds twice.* The copy that an object that is not selected is given
    can make its route hold one point at two places: part w (8) gives W the route P₂, P₃, P₁,
    P₂, P₃, where P₂ and P₃ each stand twice. A join that copies such a route copies both
    places. Wherever a rule takes a point's place in a route, the place is the point's first one
    in that route:
    - the point-click test's last point and the one before it, so a press on a point that is a
      route's last point and also stands earlier in it is not refused for that route as one on
      its last point.
    - a loop that a press closes, which runs from the pressed point's first place to the route's
      last point, the later places included.
    - a join's copy, which runs from the joined point's first place in the copied route to its
      end, the later places included. Whether the copied loop's first point comes before the
      joined point is judged against that first place (*Joins* below), and the copy's loop
      begins at the first place, in the joining holder's route, of the point that *Joins* and
      *Who gets a press's orders* say it begins at.
    - the action recorded for the holder at the point, which the refusals above and the holder's
      issue of the point read: the action recorded at the point's earliest place still in the
      route.
    - the point that a point delete selects next (claim 16e): the one after the deleted point's
      first place in its longest holder's route.

    A point delete (claim 16e) removes the point from every place in every route that holds it.
    Whether a route holds a point, and which point is its last, do not depend on how often the
    route holds it. Our engine follows the same rules. Every case here is `unmeasured`: no
    scenario here makes a press, join or delete on a route that holds a point twice, and part
    w (8)'s routes hold the joined point once, so its first place is its only one.

    *Click areas, Q-2138 question 3.* A press is inside a point's *click area* when the cursor is
    in a square of 16 by 16 screen pixels around the screen position of the point's place: from
    8 pixels left of and above that position to 7 pixels right of and below it. The point's cell
    plays no part. Of the points drawn for the player (claim 16g) whose click areas hold the
    cursor, the earliest-made one is the candidate. It is highlighted, and a press takes it, only
    when it is a point of the player's own house that the point-click test above accepts for the
    selection. Otherwise no point is highlighted, and no later-made point under it is considered.
    A press that takes no highlighted point is tested against the earliest-made point of any
    house, drawn or not, whose click area holds the cursor. When the point-click test refuses
    that point for the selection, the press is refused with the invalid-point message.
    Otherwise, and when no click area holds the cursor, the press is an ordinary route-mode
    click (claim 16d). With nothing selected, or only objects with no route record, the test
    accepts every point.
    The orders that a press on a point sends name the point by its place. When applied, they act
    on the house's earliest-made point at that place. If no point of the house is at that place
    when such an order is applied, the order does nothing.
    - Quirk: `quirks.hidden_point_refuses_click` (default: original). With the switch off, a
      press that takes no highlighted point is tested only against points drawn for the player
      (claim 16g), so a point that the player cannot see never refuses a press. Such a point
      then plays no part in a press at all: a press inside only the click areas of points not
      drawn for the player counts as one outside every click area, and meets claim 16d's kind
      test.

    *Two points at one place, Q-2166.* Several points of one house can stand at one place (claim
    16a): in the routes of different objects, or twice in one route. They have the same click
    area, so the earliest-made of them is the candidate, whatever the selected objects' routes
    hold, and the point-click test is made for that point alone:
    - a selection whose routes hold the earliest-made point is tested for it as above, and when
      the test accepts it, the press closes a loop there, also when a selected route holds a
      later-made point at the same place (unless the route changes before the order is applied,
      *Who gets a press's orders*).
    - a selection with no route record joins the earliest-made point, copying its longest
      holder's route (*Joins*, below).
    - a selection with a route record whose routes do not hold the earliest-made point is
      refused with the invalid-point message, even when they hold a later-made point there.
      Such an object can still add a point at that place by a click on the place's cell outside
      the click area, which makes a further point there.
    - with nothing selected, the press selects the earliest-made point (claim 16e), and the
      delete key then removes that one.

    The markers of such points are drawn in the order the points were made, so the later-made
    marker lies over the earlier-made one. Which marker lies on top plays no part in which point
    a press takes.
    - Quirk: `quirks.stacked_point_takes_earliest` (default: original). With the switch off, the
      candidate where several points of the house stand at one place, with objects selected, is
      the earliest-made of them that a selected object's route holds, else the earliest-made
      one. The highlight and the point-click test use that point, under the same test as above.
      The press's orders name that point itself, not its place: when applied they act on it, and
      an order whose point has gone does nothing. With nothing selected, the candidate is the
      earliest-made one, as in the original.

    *Joins.* A joining holder does not get an action of its own from the click. The join gives
    it a copy of the route of the point's longest holder: of the objects whose routes hold the
    point when the join is applied, the one whose route has held it for the longest time (since
    the point add or join that gave it the point). Of holders given the point in the same tick,
    the one whose order was applied first counts as the longest holder: for the orders of one
    click, the first in S-0214's selection order (claim 16a). The copy runs from the joined
    point's first place in that route (*A point that a route holds twice*) to the
    end of that route as it then stands, with the actions recorded there for that holder.
    - If that route has no loop, the copy has none.
    - If its loop's first point (as that route then stands, claim 16c) is the joined point or a
      later one, the copy has the same loop,
      from the same first point.
    - If its loop's first point comes before the joined point, the copy goes on, after that
      route's last point, with that route's points from its loop's first point up to the one
      before the joined point. The copy's loop then begins at the joined point.
    - In both cases the copy's loop is given as many points as that route's loop has, also when
      that loop ends early (claim 16c). When the source loop does not end early, the copy's loop
      in the last case is the whole copy, and the joining holder goes round the same loop,
      entered at the joined point. When the source loop ends early, the copy holds the same
      points as above (which points the copy holds does not depend on where the loop ends), so
      the copy's loop also ends early: its number of points is less than the copy's length.
    - The holder goes round every point from the copy's loop's first point to the copy's last,
      whatever that number (claim 16c, *A loop that ends early*). The number counts for a later
      point delete: each delete at or after the loop's first point lowers it by 1, and the
      loop is cancelled once it is below 3 (claim 16e). When the joined point comes before the
      loop's first point, the copy's loop begins at the place of the loop's first point in the
      copy and ends early as the source loop does.
    - Backing and confidence: a copy whose source loop ends early, and the number such a copy's
      loop is given, are `unmeasured`, medium, as claim 16c's early loop. A copy whose source
      loop does not end early is rated in this claim's Backing and Confidence below (parts n
      and r).

    One press can join some objects and close a loop for others (claim 16c): its orders are
    applied in the order of its list (*Who gets a press's orders*), each on the routes as the
    earlier ones left them. A joining holder copies the route of the joined point's longest
    holder as that route stands when the join is applied. When the press also closes that
    longest holder's loop, a joining holder listed before it copies the route before the loop
    is closed, so its copy has no loop, and one listed after it copies the closed loop. When the
    press does not close the longest holder's loop, the copy is that holder's route as it
    stands, whatever loops the press closes for others.

    A holder that is going round a loop has its points in the order it will next issue them
    (claim 16c), so the copy starts at the joined point and holds the whole loop, whatever point
    that holder is going to. A holder that has already issued a point of a route without a loop
    no longer holds it (claim 16c), so it is never the one copied. The copied points are
    uncommitted until the next commit (claim 16c). A join is not tested for the four refusals
    above, whatever the joining holder's route holds: the copy is added after its own points.
    A selected joining holder has none, unless it has lost the pressed point before its order
    is applied (*Who gets a press's orders*). Nor do claim 16d's tests: a press that takes a
    point is not tested for kind, route state or the holder limit (claim 16d). Each copied point
    keeps the order recorded there for the copied holder, now with the joining holder as its
    holder, and the joining holder issues it as any holder issues its own (above). Its action is
    not resolved anew for the joining holder, at the join or when the point is issued (medium).
    So a joining holder can issue an action that a click would not give it, such as an engineer's
    *capture* copied to a vehicle (Q-2227 question 2). That order is applied as it would be for
    any actor (claims 4 and 5), with no test of whether a click would give the joining holder that
    action. A vehicle that issues an engineer's copied *capture* on an enemy building gets
    *capture* with the building as its destination and no target, and S-0215 claim 28 processes
    it as for any vehicle in *capture*. The order names no target, so the vehicle fires at nothing
    while it heads for the building. A drive vehicle (S-0207 Terms) armed in S-0209 claim 5's sense
    (it has a primary weapon) whose type is neither a train type nor has the crusher flag attacks
    the building once its next step is a cell of the
    building: an armed vehicle's step check of that cell gives *enemy obstacle* (S-0209 claim 5,
    row VP28, unless an earlier row of that table decides it), which S-0209 part 5f does not
    change for such a type, and a repeated check (S-0207 Terms) whose step enters a cell of the
    building makes it attack the first object listed in the cell by an override, with no
    destination (S-0207 claim 15, table T4, and S-0215 claim 10). For an unarmed vehicle,
    a vehicle of another movement kind, or one of a train type or with the crusher flag, this is
    not claimed.
    That object is the building when nothing else is listed there (medium). The vehicle never
    captures the building, since only an infantry unit makes a building entry (S-0509 claim 1).
    A copied *sabotage* behaves in the same way: its order also has the building as its
    destination and no target (S-0214 table M), and S-0215 claim 28 processes *sabotage* as
    *capture* (medium). Its route goes on as any holder's does (claim 16c): it issues its next
    point, if it has one, at its next fall-back to rest (medium).

    A refusal leaves the route unchanged. For the local player's own orders it shows a message,
    each case its own text: after a terminating point, after a continual point, in a loop and
    at 16 points (Import mapping). An accepted point plays the imported route-point sound
    (Import mapping), and when
    the added point is itself terminating or continual, or is the 16th point, the message of
    that case is shown as information. The checks are made when the order is applied, and only
    the messages depend on the local player.
    - Backing: `measurable` (Scenario O8, parts a, b, e, f and l, parts n and r for joins, with
      part r (5) for holders given the point in the same tick, and part q for the order's place
      (part q (6) for an attack-move click on a cell),
      the clicks on existing points and the route record kept, part v for the detonate, the
      moment it is sent and its refusal by the holder limit, part w for the click area, the
      patrol action, a press for a mixed selection in both selection orders,
      and parts w (6) and (7) for the point that a press takes, selects, deletes, joins or
      closes a loop at among points at one place, and for a further point added there by a
      click outside the click area, part w (8) for the orders a press gives objects that are
      not selected, a loop closed and a copy added after a route's own points that hold points of
      the copied loop other than the joined point, and part h for presses on points of other
      houses). For an object that is not selected, a copy that takes its route past 16 points, a
      point add to a route of more than 16 points, the copy's loop replacing a loop its route had
      or leaving it none, and a loop that begins at one of its own points are `unmeasured`: no
      scenario here gives an object that is not selected such a route. A selected holder whose
      route has changed before its order is applied, and that order's join after the
      holder's own points, its loop closed with no test at a later place or a later-made point,
      or its doing nothing, are `unmeasured`: no scenario here has a selected holder issue the
      pressed point in the tick in which the press's orders are applied. That the choice between a
      loop close and a join is made when each order is applied, rather than at the click, is
      `unmeasured` as a rule for the same reason: in every run here a selected object's route holds
      the pressed point at its order's application exactly when it held it at the click. A loop
      closed for an object that is not selected at
      its route's last point or the one before it, or over a loop its route already has, is
      `unmeasured`: no scenario here gives such an object a route that the point-click test
      would refuse. The list's order is tested only where the selected object is the first
      to have been given its route's last point (part w (8)). That the pending-list limit drops
      the orders of the objects after the first 128 in the list is `unmeasured`: no scenario
      here gives a press's list more than 128 objects. That a click that selects or
      toggles the selection adds no point is `measurable` (part x, steps (2) and (4), where the
      click selects and no new marker appears). An order that finds no point of
      its house at its place is `unmeasured`: it needs the point to go between the press and the
      order's application, which no scenario here sets up. That an order acts on the earliest-made
      point at its place when applied, rather than
      on the point the press took, is `unmeasured` too: in every run here the point that the
      press takes is still the earliest-made one at its place when the order is applied, so both
      readings give the same result. Which of two markers at one place lies on top is
      `unmeasured`: no scenario here records the drawing, and it decides nothing that the runs
      observe. That every click action of a vehicle or infantry unit that orders something,
      other than the detonate, gives a behaviour order, and that an action S-0214 gives no
      order for adds no point, are `unmeasured` as a whole: the scenarios here run only some
      of those actions in route mode. So are an aircraft's click actions and a building's
      *attack* action giving behaviour orders, and a building's other actions on an object going
      the two ways above: no scenario here runs them as ordinary clicks in route mode. The
      rally-point click sent directly is `measurable` (part h's third game). Its
      being sent at the release after claim 16d's release-time tests is `unmeasured`: part h's
      third game sets the rally point with tests that pass, so a send at the press would give
      the same record. An aircraft's deploy action adding no point, and an order that names
      neither a target nor a destination adding none, are `unmeasured`: no scenario here gives a
      selected aircraft that carries passengers the deploy action, and no other click gives
      such an order. The place of a *move* or attack-move click on an object, and that of a
      *patrol* point, are `unmeasured`: parts q (5) and q (6) run only clicks on a cell. The
      undeploy click is `unmeasured`: no scenario here selects a building that can undeploy.
      That the detonate click's route-state test is made at the release is `unmeasured`: the
      test can refuse it only with a selection of more than one object, and no scenario here
      gives such a selection the detonate action.
      The precedence among click areas that overlap is `unmeasured` for points at different
      places, for a point of an allied house over one of the player's own, and for a point not
      drawn for the player against a drawn one: no scenario here sets such an overlap up, since
      part w uses points at one place and part h a single point of another house.
      The copied points' being uncommitted until the next commit is `unmeasured`: parts n and r
      release the key after the join, so the commit follows at once and no run shows a joining
      holder waiting at a copied point before a commit. That a holder issues the order recorded
      for it at the add or copied to it at a join, not one classified anew when it is issued, is
      `measurable` for a loop's *move* point changed into an attack-move order (part aa) and for
      the *enter* that a full-health vehicle's point on an own repair building issues (part ab),
      and for the engineer's *capture* that a joining vehicle issues (part ac), and `unmeasured`
      otherwise: no scenario here changes, between the add and the issue, anything that a point's
      action depends on. The joining vehicle's making no shot at the building before its next step
      is a cell of the building, its attack on the building by the override at that step, and its
      not capturing the building are `measurable` in part ac, for *capture* in run 1 and for
      *sabotage* in run 2. That its route goes on at its next fall-back to rest is
      `unmeasured`: a *capture* point is terminating, so part ac's copy has no later point.
    - Confidence: high that the click takes its ordinary S-0214 action, and for the deploy
      refusal for a vehicle or infantry unit; medium for the four refusal cases and their
      separate texts (an aircraft's deploy is not one of them: it adds no point because its
      order has no place, rated low below with the aircraft's click actions), the action
      kept per holder, the order's place, the order a holder issues being the one recorded at the
      add or copied at a join with no new classification, a copied action that a click would
      not give the joining holder being applied as for any actor (a joining vehicle's copied
      *capture* or *sabotage* giving it that behaviour, no shot at the building before its step
      into the building, the override's attack on the building at that step, and no capture, and
      its route going on at its next fall-back to rest, unmeasured), the info messages, the sound
      and the three conditions
      of the point-click test, and for the holder that a join copies (holders given the point
      in the same tick included) and what the copy holds and loops; medium for the copied
      points' being uncommitted until the next commit; low for
      the route record kept once the route is empty, for the hover highlight, for which point a
      press takes where click areas overlap (the unmeasured branches above included), for
      presses on points of other houses. Medium for the point that a press takes among points at
      one place, in each case of *Two points at one place*, for the further point added there by
      a click outside the click area, and for the later-made marker drawn on top; medium
      for an order that finds no point at its place doing nothing, and for an order's acting on
      the earliest-made point at its place rather than on the point the press took. Medium for
      one press that closes a loop and joins, the detonate sent at the release after claim 16d's
      tests and refused with no order when one of them refuses (its route-state test included),
      every other click action of a vehicle or infantry unit that orders something giving a
      behaviour order, and an action that S-0214 gives no order for adding no point, the
      rally-point click sent directly at the release, the click area, and the patrol action's
      point add and its being neither terminating nor continual. Low for an aircraft's click
      actions and a building's *attack* action giving behaviour orders in the same way, a
      building's other actions on an object going the two ways of claim 16b (its *unload* orders
      as point adds at its own place, which takes in its *deploy* action coming only from a
      click on the building itself and each *unload* naming the building as its target, read
      from the original with no scenario here giving a selected building the deploy action, its
      primary, guard-spot and detonate orders sent directly, and its deploy action not refused
      in route mode), an
      aircraft's deploy action adding no point, and
      the undeploy click sent directly (all unmeasured): which action a click gives an aircraft
      or a building is S-0214 claim 10's mapping, rated low there for both kinds. Medium for an
      order that names neither a target nor a destination adding no point, and for the place of
      a *move* click on an object and of a *patrol* point (unmeasured). Medium for a
      click that selects or toggles the selection adding no point (no object gets an order,
      S-0214 claim 3). Medium for the
      objects listed by a press on a point
      and the list's order, and for the pending-list limit dropping the orders after the first
      128 in the list (unmeasured). Low
      for each outcome for an object that is not selected: the loop closed (at its last point
      or the one before it, or over an existing loop, included) and the copy added after its own
      points with the copy's loop (past 16 points, in place of a loop it had, and beginning at
      one of its own points, included), and for a route of more than 16 points taking a point
      add. Medium for the choice between a loop close and a join being made when each order is
      applied (unmeasured), as a rule. Low for each outcome for a selected holder whose route
      has changed before its order is applied: the join after its own points (past 16 points
      included), the loop closed with no test at a later place or a later-made point at the
      place, and the order that does nothing. Medium for the first-place rule of *A
      point that a route holds twice* and for a point
      delete removing every place (unmeasured), as rules; routes that hold a point twice come
      about only through the low outcomes above, which bound how often they apply, not the
      rules. Low for the action read at a point held twice and for the point a delete selects
      next on such a route (unmeasured)

16c. **Commit, following and loops, Q-1986 question 3.** Points added since the last commit are
    not followed: a holder whose route reaches them falls back to rest there and waits for a
    commit. When a commit order is applied, every point of the house's routes becomes committed.
    Being committed belongs to each route, not to the place. A point add applied after a commit
    at a committed point's place does not share that point (claim 16a): it adds a new,
    uncommitted point to the actor's route, and the actor waits there for the next commit as at
    any new point. The routes holding the committed point are not changed.
    A holder whose route could not run before the commit (it had no committed point left to
    follow) starts at once if its current behaviour is *guard*, or is a continual action of
    claim 16b: it gives up that behaviour and waits, and it issues the order of its next point,
    which takes effect at the next order boundary (claim 1). Giving up is a queue of *wait* with
    an immediate start (S-0215 claim 7). So a holder that is ready to change (S-0215 claim 11)
    has *wait* current from the commit until the issued order's behaviour becomes current: at
    the holder's first per-tick start after that order applies if it is ready then, otherwise at
    the first later permitted start (S-0215 claims 11 and 12). Its *guard* or *area guard*
    processing does not run in that time: a processing of *wait* does nothing (S-0215
    claim 4), so no shot or move that processing would have made is made. The commit itself
    does not change the holder's target or destination. The commit applies at its tick's order
    boundary (claim 1), after that tick's object updates, so in the commit's tick the holder's
    drop rule and firing step still run under its earlier behaviour: it keeps its target and can
    fire at it. For a ready holder, *wait* is first current at S-0215 claim 33's drop rule in the
    tick after the commit's tick, since the issued order applies at that tick's order boundary at
    the earliest. So a target of the holder's that is an opportunity target (S-0309 claims 23 and
    24) is dropped in that tick, before its firing step (S-0310 claim 9 for a vehicle), and the
    holder makes no shot at it from that tick until the issued order's behaviour becomes
    current. A target that is not an
    opportunity target is kept, and that firing step, which runs in each tick in which the holder
    has a target, can still fire at it while *wait* is current. Any other holder, for example one
    still moving under an earlier order or point, is not interrupted. It issues its next point
    at its next fall-back to rest (S-0215 claim 13), as it does for each later point. An
    aircraft or building holder issues its later points at the moments of claim 16d instead
    (*Aircraft and buildings*). Each such
    issue is preceded by the same queue of *wait* with an immediate start as at the commit, so
    a holder that is ready to change has *wait* current from that fall-back until the issued
    order's behaviour becomes current, and the fall-back queues no *guard* (Changes for other
    specs, S-0215 claim 13). Issuing a
    point takes it off the front of the route, so a route without a loop is used up when its
    last point has been issued. The holder then keeps doing the last point's action: the route
    neither stops it nor starts again.
    Issuing a point of a loop moves it to the end of the route instead, so the route of a holder
    going round a loop always holds its points in the order it will next issue them. As the
    route stands, its loop's first point is the earliest of the loop's points in it: the point
    the loop was closed from while points before the loop remain, and the point the holder will
    issue next once they have all been issued.
    *A loop that ends early, Q-2191.* Claim 16b's rules for a press's list can give a route a
    loop whose last point comes before the route's last point, with points after it (claim 16b,
    *Who gets a press's orders*). Such a route is issued by the same rule: while points before
    the loop's first point remain, issuing one takes it off the route, and once they have all
    been issued, every issued point, of the loop or after it, moves to the end of the route. So
    the points after the loop are issued too: the holder goes round every point from the loop's
    first point to the route's last, and in each round the points after the loop follow the
    loop's last point. The number of points that the loop was given plays no part in the order
    of issue. It counts only for a point delete (claim 16e) and for a join's copy (claim 16b,
    *Joins*). A selected object whose route has a loop has every press on a point refused (claim
    16b), so the point-click test's last point and the one before it never apply to such a
    route. Where claim 16b's list and claim 16h's selection of other holders read a route's last
    point, it is the route's last point, after the loop.
    Looping is an input, never automatic. A click, in route mode, on a point that is already in
    the selection's routes closes a loop from that point to the route's end. The loop's points
    are then issued in turn, the first loop point again after the last, until the holder gets an
    order outside route mode (claim 17). Points before the loop's first point are issued once.
    Closing a loop shows an information message to the local player (Import mapping).
    *Attack moves in a loop.* When a loop-close order is applied, if any holder of the loop's
    first point is then of a type eligible for attack moves (claim 6), every point from the
    loop's first point to the route's end whose order for a holder is a *move* becomes an
    attack-move order with the same destination, for every holder of that point. The test is
    made per holder: at a point that holds different actions for different holders (claim 16b),
    each holder whose action there is a *move* gets the attack-move order, and each whose action
    there is not a *move* keeps it. Whether a type is eligible is read from the holders of the
    loop's first point only. Each holder
    then issues the changed order as its recorded order (claim 16b), which claim 6's table
    translates, so a mobile holder of an eligible type attack-moves
    there and one of a type that is not eligible moves as before. A building holder of such a
    point gets claim 6's building result, *attack move* itself, which S-0215 claim 5 processes as
    *sleep*: the building does nothing there, as under *move* (claim 16d), and issues no further
    point. This is an input path to claim 6's building column. Points before the loop's first
    point keep their orders. On a loop that ends early the change covers the points after the loop
    too. The change is made at every loop close, a second loop closed in the same tick
    included. The original can leave such a second loop unchanged on some clients of a network
    game and not on others, so the clients disagree. That is not copied and has no switch. A press
    that would close a loop on a selected object's own route at that route's last point or the
    one before it is refused (claim 16b), so a loop that a press closes on a selected object's
    own route holds at least 3 points, unless that route changes between the click and the
    order's application (claim 16b, *Who gets a press's orders*). Shorter loops come about only
    through claim 16b's rules for a press's list. A press closes a loop for an object that is not
    selected even at its route's last point or the one before it. A selected object that joins
    copies the joined point's longest holder's loop (claim 16b, *Joins*), which can be such a
    shorter loop, so a selected object that joins can get a loop of fewer than 3 points.
    Holders that share a point appear to start that point's action together, each waiting in
    place until the others have reached the point before it. That synchronisation's exact rule
    has not been settled (Open questions).
    - Backing: `measurable` (Scenario O8, steps 1 and 2, parts b, c and g, part q for the
      3-point minimum of a loop that a press closes on a selected object's own route, part s
      for a point add at a committed point's place, part t for the
      *wait* at the commit and at P₁ for the *wait* before a fall-back's issue, part t (2) for the
      target kept in the commit's tick and dropped in the next, with no shot at it from the next
      tick on, and part r (4)
      for the order of a looping holder's route and its loop's first point, part (aa) for the
      attack moves in a loop). That the commit
      itself leaves the holder's destination, and a target that is not an opportunity target,
      unchanged, and that the firing step can still fire at such a target, are `unmeasured`:
      part t's V has no destination, and no scenario here gives a holder in *guard* a target that
      is not an opportunity target. That a selected object that joins can get a copied loop of
      fewer than 3 points is `unmeasured`: no scenario here gives a point's longest holder such
      a loop. A selected object's route that changes between the click and the order's
      application is claim 16b's `unmeasured` part. A loop that ends early is `unmeasured`: it
      comes about only through claim 16b's low outcomes for a press's list, and no scenario here
      gives a route such a loop. Of the attack moves in a loop, the points before the loop's
      first point keeping their orders, the holders of a type that is not eligible, the points
      of other holders and those after a loop that ends early are `unmeasured`: part (aa) has
      one holder, an eligible one, and no point before its loop. A building holder's *attack
      move* at such a point, and its issuing no further point, are `unmeasured` as well: no
      scenario here gives a building a *move* point in a loop. The per-holder test at a point
      that holds different actions for different holders is `unmeasured` too: no scenario here
      gives two holders of one point different actions there. The change at a second loop
      closed in the same tick, and the original's disagreement between clients there, are
      `unmeasured`: no scenario here closes two loops in one tick
    - Confidence: high for looping by input; medium for the 3-point minimum of a loop that a
      press closes on a selected object's own route that is unchanged since the click; low for
      a shorter loop closed on a selected object's route that has changed by then (claim 16b's
      low rule), and for a selected object that joins
      getting a copied loop of fewer than 3 points (from claim 16b's low rules); medium for the
      start at commit and its behaviour condition, for the *wait* at the commit and at a fall-back's
      issue, for the opportunity target kept in the commit's tick and dropped in the next, with no
      shot at it from the next tick on,
      for the commit's leaving the destination and any other target unchanged and the firing at
      such a target, for the order of a looping holder's route and its loop's first point, for
      uncommitted points
      and a point add at a committed point's place, for the holder keeping its last action and
      for points before the loop; medium for the order of issue on a route whose loop ends early,
      the loop's number of points playing no part in it, and the last point read after such a
      loop (unmeasured), as rules; such routes come about only through claim 16b's low outcomes,
      which bound how often they apply, not the rules; medium for the attack moves in a loop,
      the eligible holder's change, the points before the loop kept, the ineligible holder, the
      other holders' points, the per-holder test at a point with different actions, a building
      holder's *attack move* with no further point issued and the points after a loop that ends
      early included, and for the
      change at a second loop closed in the same tick; low for the original's disagreement
      between clients there, and for synchronisation

16d. **Who can take part, Q-1986 question 5.** At a route-mode press outside every point's
    click area, only vehicles and infantry can take part: such a press is refused when an
    aircraft, a building or an object of any other kind is selected. A press inside a click area
    is not tested for kind (below).
    The rule is fixed for each kind of object and no imported key changes it. A route-mode
    click is refused, with a message for each case (Import mapping), when:
    - the press is outside every point's click area and a selected object is an aircraft, or a
      building, or of any other kind.
    - the selected objects are not all in the same route state: either none of them holds a
      route, or all hold routes that end at the same point.
    - the click is on an existing route point that the selection can neither join nor close a
      loop at (claim 16b), which gives its own message.
    - the route-holder limit of claim 16 would be passed.

    When a selection both mixes route states and would pass the holder limit, only the
    incompatible-selection message and one sound are given: the two tests are made together,
    and claim 16's shared cue goes to the route-state refusal.
    Harvesters, unit-controlling vehicles and deployed infantry are vehicles or infantry and are
    not refused for their kind. Objects carried inside a transport cannot be selected (S-0214),
    so they never meet the check. No owner check is made beyond S-0214's selection rules.

    *When the tests are made, Q-2138 question 4.* The kind test is made when the button is
    pressed, for every route-mode press on the battlefield with the cursor outside every click
    area (claim 16b), and only for battlefield presses: key commands and clicks on the sidebar or
    the command bar are not tested for kind (claim 16h). *Q-2193.* The click areas here are
    those of every point of every house, drawn for the player or not (claim 16b, *Click
    areas*; with `quirks.hidden_point_refuses_click` off, only those drawn for the player): a
    press inside the click area of a point that the player cannot see, a point of an enemy
    house for example, is not tested for kind, whether the point-click test then refuses
    it or the press is an ordinary click. Only left-button presses are tested: a right press or
    right click is never tested for kind, so with an aircraft or a building selected it gives no
    message and no sound, and acts as claim 16h gives. The kind test is made whatever the click
    would do: give an order, select another object, start a band box or nothing. A refused press
    ends the click: the selection is kept, and no order,
    point or band box follows. So with a building selected, a click on bare ground or on another
    object to select it is refused in route mode. With nothing selected, no object is tested.
    When several selected objects fail the kind test, the message is that of the first of them
    in S-0214's selection order. The same-route-state and holder-limit tests are made when the
    button is released, for every click except one that selects, toggles the selection or does
    nothing, the release of a band box that selects objects, and a click with a tool of claim
    16h. So a press refused for kind never also gets their messages, and a selecting click is
    never refused by them. A click whose ordinary action is a detonate is not among those
    exceptions: it meets both release-time tests, and its order is sent only when both pass
    (claim 16b).
    A press inside a click area is not tested for kind. A press that takes a point meets only
    the point-click test, so its message is never given with another refusal of this claim, and
    its join and loop-close orders are not tested for route state or the holder limit either.
    So in the original a selected aircraft or building with no route record joins a point as a
    vehicle does and gets a route. A press inside a click area that is an ordinary click is not
    refused for kind either, and gives such an object what claim 16b gives it.
    *Aircraft and buildings, Q-2192.* An aircraft or a building that holds route points is a route
    holder like any other object. Only claim 16b's four refusals apply to its point adds. It
    counts towards the route-holder limit as a vehicle does: a join that gives it its first points
    does not raise the count, a point add that gives it its first point does, and the
    release-time limit test counts it as new while it holds no route point (claims 16 and 16f). A
    commit starts it under claim 16c's rule: an aircraft at rest on the ground and a building at
    rest are in *guard* or *area guard* with no target, so a commit after its join or first point
    add makes it issue its first point. It issues each point as any holder does (claim 16b),
    with the attack-move translation of claim 6. Its later points are issued at these moments:
    - an aircraft issues its next point when a flight ends, at the moment it would begin to land
      (either landing trigger of S-0501 claim 6), unless its behaviour is then *enter*. This issue
      is S-0501 claim 6's going on to a further waypoint of a planned route, not a step beside it,
      and it comes before that claim's pad-bound test (Q-2229). So a pad-bound aircraft away from
      its pad that has a point to issue issues it and does not fall back to rest. An aircraft that
      issues a point begins no landing in that tick and stays in flight. As for every holder, the
      issue is preceded by claim 16c's queue of *wait* with an immediate start, so an aircraft
      that is ready to change has *wait* current from the flight's end until the issued order's
      behaviour becomes current. The issued order applies at the end of that tick (claim 1). When
      it names a destination, it gives the aircraft that destination while the aircraft is
      neither landing nor taking off, so the aircraft flies on with no landing and no take-off.
      What an issued order with no destination makes the aircraft do is that order's own effect
      (claim 5 and S-0501). When the aircraft has no point to issue, S-0501 claim 6's pad-bound
      test and landing follow unchanged. It issues none when it falls back to rest. So after a
      point whose flight ends in *enter*, such as a return to its dock, its remaining points are
      not issued.
    - a building whose type does not have the SAM flag issues its next point only when its
      *attack* behaviour has no target, so after an *attack* point it goes on when that target is
      gone. This is S-0215 claim 32's case of
      *attack* without a target: the building first issues its next point, with claim 16c's
      queue of *wait* with an immediate start, and when *wait* is then its current behaviour it
      queues no *guard*. When it has no point to issue, or *wait* is not current, the *guard*
      of S-0215 claim 32 follows as committed, and an issued order still applies at the next
      order boundary. Under *move* a building does nothing:
      it neither moves nor looks for targets nor fires, and it issues no further point.
      An *attack* order given to it outside route mode (Q-2228) ends its route (claim 17) and
      queues *attack* with the order's target (claim 5). The building starts that *attack* only
      at a per-tick start at which its ready mark is set (S-0215 claim 11). A building without
      a turret has the mark set by its body range, so it starts the *attack* and fires.
      Nothing sets a turreted building's mark while *move* is current. *Move* has no processing
      rule for a building (S-0215 claim 4), its body range sets no mark outside *construction*
      and *selling*, and the start that made *move* current cleared the mark (S-0215 Terms,
      *ready mark*). So a turreted building keeps *move* current and *attack* queued, and makes
      no shot at that target. This is S-0215 claim 11 applied, not a rule of its own.
    - a building whose type has the SAM flag issues no later point. Its *attack* follows S-0215
      claim 32's SAM rows unchanged: without a suitable target it drops the target and queues
      *guard*, and no point is issued, so it issues only the first point that a commit makes it
      issue.
    A holder whose remaining points are not issued in this way keeps them until an order outside
    route mode ends its route (claim 17) or it is removed from play. A later commit does not
    start it again: its route could already run, and a building under *move* is not in *guard*
    (claim 16c).
    - Quirk: `quirks.click_area_skips_kind_test` (default: original). With the switch off, every
      route-mode battlefield press with objects selected meets the kind test first, inside a
      click area or not, so no aircraft or building gets a route point, none is left with
      points that it does not issue, and no turreted building is left under *move* with an
      *attack* that never starts.
    - Backing: `measurable` (Scenario O8, part d, part q for clicks on existing points, and game
      B step 3 for the two refusals together, part x for the kind test at the press, the clicks
      that skip the other two tests, the message among several kind failures, the presses
      inside click areas and a sidebar click not tested for kind, part w (5) for a press that
      takes a point with a selection of mixed route states, part o for a press that takes a
      point at more than the holder limit, and part v for a detonate click refused by the holder
      limit; its route-state test is claim 16b's `unmeasured` part, part x (5) to (7) for an
      aircraft's and a building's join, the start at the commit and the points then issued or
      not, part x (7)'s first and third games for a building under *move* that starts a later
      *attack* order or not by whether it has a turret, part x (8) for right clicks with a
      building selected, and part h's third game for a
      press inside the click area of a point that the player cannot see, with a factory
      selected, and the rally point it sets). An aircraft's or building's place in the
      route-holder count is `unmeasured`: no scenario here brings a house to the limit with one
      among its holders. Part x (6)'s per-tick record backs an aircraft's issue at a flight's
      end beginning no landing and no take-off, and its pad-bound game the issue coming before
      the pad-bound test. That *wait* precedes an aircraft's issue is `unmeasured`: whether it is
      ever current depends on the aircraft's readiness to change in flight, which no scenario
      here sets or records. A building's issue of its next point when its *attack* has no target,
      and the *guard* it then queues or not, are `unmeasured`: no scenario here gives a building
      an *attack* point. So is a building with the SAM flag issuing no later point: no scenario
      here gives such a building a route point. That an aircraft
      issues no point when it falls back to rest, and keeps its remaining points after a flight
      that ends in *enter*, is `unmeasured`: no scenario here gives an aircraft a route point
      whose flight ends in *enter*. That a later commit does not start such a holder again is
      `unmeasured` for the same reason, and for a building because no scenario here commits
      again after part x (7)'s press.
      That the same-route-state and holder-limit tests are made at the release rather than at
      the press is `unmeasured`: no scenario here changes the selection's route state or the
      house's route-holder count between a press and its release, so every run gives the same
      result for either moment
    - Confidence: high for the kind rule at presses outside click areas. Medium for each of
      these: the same-route-state rule; the message given when it and the holder limit both
      refuse; the kind test made at the press and only at battlefield presses; the clicks that
      skip the other two tests; the first failing object's message; a press inside a click area
      not tested for kind; a press that takes a point not tested for route state or the holder
      limit; the release as the moment of the same-route-state and holder-limit tests; a
      detonate click meeting both release-time tests. Medium for each of these too: an aircraft
      or building with no route record joining a point and getting a route; its start at a
      commit; the moments at which it issues its later points (unmeasured but for an aircraft's
      issue at a flight's end); an aircraft's issue coming before the pad-bound test and
      beginning no landing in its tick; its issue being preceded by *wait* (unmeasured); a
      building's *guard* queued or not after such an issue
      (unmeasured), a building with the SAM flag issuing no later point (unmeasured), and a
      later commit not starting it again (unmeasured); a
      building under *move* doing nothing; a building under *move* starting a later *attack*
      order when it has no turret and never when it has one; an aircraft or building
      counted as a route holder (unmeasured), but for a join that gives it its first points,
      which is low (unmeasured, from claim 16f's low rule for joins); the click areas of points
      not drawn for the player counting for the kind test; right presses never tested for kind

16e. **Deleting and extending, Q-1986 question 4.** A click, in route mode and with no object
    selected, on an existing route point selects that point and shows an information message.
    With objects selected, the same click joins the point, closes a loop or is refused instead
    (claims 16b and 16c). The delete key
    then sends a point-delete order for the selected point, which removes it from every route
    holding it. Selecting any object, or a short right click (claim 16h), clears the point
    selection.
    When a deleted point lay
    inside a loop and the loop is left with fewer than 3 points, the loop is cancelled and the
    remaining points form an ordinary route. On a route whose loop ends early (claim 16c), a
    point after the loop counts as one inside it: deleting any point at or after the loop's first
    point lowers the loop's number of points by 1, and the loop is cancelled once that number is
    below 3. A route can be extended: holding the
    key again with the same selection and clicking adds points to the end of each route,
    whether or not the route is being followed, under the checks of claims 16b and 16d.
    Releasing the key commits again. A holder still busy with an earlier point is not
    interrupted and later reaches the new points (claim 16c).
    *The delete key, Q-2139 question 5.* The delete key sends the point-delete order whenever a
    point is selected, whether or not the route-mode key is held. Otherwise it does what it does
    outside route mode. A click can select a point only while the route-mode key is held and no
    object is selected. Releasing the key clears the point selection, and so does any selection of
    an object (claim 16h), by any means. The one exception is a point delete applied after the
    release, below. Where several points of the house stand at one place, the click selects the
    earliest-made of them (claim 16b, Q-2166). When a point-delete order of the local player's house
    is applied, the point after the deleted one in the route of its longest holder (claim 16b),
    going round a loop, becomes the selected point, if there is one. So repeated presses delete the
    following points in turn. The point is found in that route as it stands before the delete.
    When the deleted point is that route's last point, the count goes on round the loop's points
    only (Q-2226). Let *n* be the number of points from the loop's first point to the route's
    last, both included, and *C* the loop's number of points (claim 16c). The selected point is
    the one (*n* mod *C*) places after the loop's first point, where 0 places is that point
    itself. On a loop that runs to the route's end, *n* = *C*, so the selected point is the
    loop's first point. On a loop that ends early (claim 16c) it is a point of the loop, never
    one after the loop, and it is the loop's first point only when *n* is a multiple of *C*.
    A route without a loop gives no point when its last point is deleted. Where the route holds
    the deleted point at two places, the point after its first place is taken (claim 16b, *A
    point that a route holds twice*). That first place is never the route's last point, so the
    count round the loop does not apply. A point delete applied after the key's release selects
    that next point too, and a later press of the delete key then deletes it, with route mode off
    and no marker drawn.
    - Quirk: `quirks.delete_selects_after_release` (default: original). With the switch off, a
      point delete applied while the route-mode key is not held selects no point.
    - Backing: `measurable` (Scenario O8, part g, and part i for the loop cancelled below
      3 points, part z for the delete key, and part w (6) for the point selected and deleted
      among points at one place). A point delete applied after the key's release
      is `unmeasured`: one player's inputs are taken one per cycle (claim 1), so a delete pressed
      before the release is applied before the release is taken, and no scenario here applies a
      point delete in a later tick than the one it is issued in. A delete on a route
      whose loop ends early, and the point it selects, are `unmeasured`, as claim 16c's early
      loop is
    - Confidence: medium, the point selected after a delete of the last point on a route whose
      loop ends early included (unmeasured, as a rule: such routes come about only through claim
      16b's low outcomes, which bound how often it applies, not the rule); low for the loop
      cancelled below 3 points, for the point
      selected where the route holds the deleted point at two places (claim 16b's low rating),
      for which holder's route
      gives the next point when several hold the deleted one, and for a point delete applied
      after the release

16f. **When an object stops holding a route, Q-1986 question 6.** An object whose route still
    holds at least one point stops being a route holder, and lowers its house's route-holder
    count (Terms) by 1 at once, when:
    - its route is removed by an order (claim 17), at that order's application.
    - its route without a loop is used up (claim 16c), when the last point is issued. When the
      last point needs travel, the holder is then on its way to it, not yet there.
    - it is destroyed and removed from play. It leaves every point it held.

    Deleting a route's last remaining point (claim 16e) also ends holding, but in the original it
    does not lower the count. The count then stays raised for that object for the rest of the
    game. A later order to the object and its destruction do not lower it, because both lower
    the count only for an object whose route still holds a point. A new route for the object
    raises the count again, on top. While its route holds no point, the object also counts as
    new when it is selected for a route-mode click (claim 16). So deletions can leave a house
    refused at the limit with fewer than 128 route holders.
    A point join that gives an object its first points does not raise the count, as far as
    studied (low), and the end of that route lowers it like any other, never below 0. So after
    joins the count can be lower than the number of route holders, and a later click that adds
    a point is tested against that lower count (claim 16).
    A route point that no route holds any more is deleted at once, whichever way its last holder
    left it: by an order that removes the route (claim 17), by issuing it from a route without a
    loop or before a loop's first point (claim 16c), by a point delete (claim 16e) or by the
    holder's destruction. A deleted point's marker is no longer drawn, a click can no longer
    select or join it. A loop's points stay while the loop is followed.
    - Quirk: `quirks.route_holder_count_drift` (default: original). With the switch off, the
      house's route-holder count is always the number of its route holders: a deletion that
      empties a route lowers it, and a join that begins a route raises it.
    Whether leaving the map for a time (entering a transport, a building or a tunnel) ends a
    route by itself has not been studied. In practice such an entry comes from an order, which
    ends the route (claim 17), or from a terminating point, which is the route's last.
    - Backing: `measurable` (Scenario O8: step 4 for removal by an order, part j for the used-up
      route and the moment it ends, part k for destruction and its deleted points, part m for
      the last point deleted and the order, destruction and new selection after it, part p for a
      new route on top, part o for a join's effect on the count, and part u for points that no
      route holds)
    - Confidence: high for removal by an order; medium for the used-up route and its moment,
      for destruction, for the count kept after the last point is deleted, and for the deletion
      of points that no route holds; low for joins

16g. **What the player sees, Q-1986 question 7.** Route points and the routes that lead
    through them are drawn only while the viewing player holds the route-mode key. They are
    drawn for the viewer's own house and for houses allied with it, never for other houses.
    Which direction of a one-way alliance counts has not been settled. Each point is shown
    with a marker, and each holder's route is drawn as lines from the holder through its points. The markers' and lines' look has not been studied
    (Open questions). In our engine this is presentation only and changes no game state.
    - Backing: `measurable` (Scenario O8, part h)
    - Confidence: high for the drawing while the key is held; medium for allied houses; low for
      the look

16h. **Other input while the key is held, Q-2139.** Apart from the clicks of claims 16b to 16e
    and the commands of table T2, route mode leaves the player's input as it is outside route
    mode, with these rules.
    - *Selection, questions 1 and 3.* Clicks that select, band boxes, the control-group commands
      (claim 22, S-0214 claim 33) and the type-select role (S-0214 claim 32) act as outside
      route mode, after the press test of claim 16d (a band box starts with a press).
      Selecting, while the key is held, an object whose route holds a point also selects every
      other object whose route holds that route's last point. They are added to the selection
      right after that object, before any further object the same input selects (a band box, a
      group key), in the order in which they were given that point, any already selected being
      left where it is. S-0214's selection order (Terms) follows from that order of addition.
      The objects selected in this way add
      none of their own: an object that holds the last point of such an object's route, but not that
      of the route first selected, is not selected. While any object is selected, no point is
      selected (claim 16e), so selecting a group clears the point selection.
    - *Tools, question 2.* The sidebar, the command bar and the tool modes act as outside route
      mode: arming a superweapon (S-0512 claim 25), the sell, repair, power and beacon modes,
      and picking a finished building to place. A battlefield press with a tool active first
      meets route mode's press rules: a press that takes a point (claim 16b) does that and
      nothing else, and claim 16d's kind test follows. A click that passes them does what the
      tool does outside route mode: it launches the superweapon, sells, repairs or switches
      power, places the building, or places a beacon. That order is sent at once and adds no
      route point, and the release-time tests of claim 16d are not made for it.
    - *Right clicks and the tool modes.* Route mode is not one of the modes that S-0214 claim 28
      lists, and that claim's route-editing mode is not route mode: no input of the original
      starts route-editing, and our engine has no such mode (Changes for other specs). A
      battlefield right click while the key is held acts as outside route mode
      (S-0214 claims 27 and 28): it ends an armed tool mode, or clears the selection when no tool
      mode is armed. It never ends route mode, and it is not tested for kind (claim 16d), whatever
      is selected. Whether a right-button gesture is a click or a
      scroll drag is S-0214 claim 27's rule: in our engine a drag starts only beyond 8 pixels
      on an axis, and the original's bound is that claim's. A right click
      made while the key is held and no object is selected also clears the point selection
      (claim 16e), but only when its release is less than 4 screen pixels from its press on
      each axis. That 4 is a bound of its own, apart from the drag bound: in the original it does
      not follow the host's drag settings (`unmeasured`, below), and our engine keeps it. It
      equals the metric of 4 that S-0214 claim 27 sets for our engine, but the drag starts only
      beyond twice that metric. So in our engine a right click whose release is 4 to 8 pixels
      from its press on an axis acts as a right click of S-0214 claim 28, as above, but keeps
      the point selection. Route mode and the tool modes do not exclude each other: entering
      route mode keeps an armed tool mode, and arming, using or ending a tool mode leaves route
      mode on.
    - *End of route mode, question 4.* Route mode ends, with the commit of claim 16c, when the
      route-mode key's release reaches the game, when the command bar's route-mode button is
      turned off, and when the local player's house is defeated, after which it cannot be
      entered again. In our engine, route mode also ends, with the commit, when the game window
      loses focus or the options menu is opened while route mode is on. No other event ends it.
      This is a design rule of our engine, where the original is not settled: whether a release
      made while the window lacks focus, or while the options menu is open, reaches the
      original's game has not been established (Open questions).
    - Backing: `measurable` (Scenario O8, part y for the selection, an accepted band box and the
      other holders it selects, with a third holder that tells one round of selection from
      several, the tools, the right clicks
      and the tool modes kept with route mode, and part z for the key's release, the original's
      case of a focus loss or the options menu, and the command bar's button). The end at
      defeat is `unmeasured`: no scenario here defeats a house while its player holds the key.
      That no input of the original starts route-editing is `unmeasured`: the absence of such
      an input cannot be shown by a run. The beacon mode's click is `unmeasured`: no scenario
      here places a beacon. The end at a focus loss or at the
      options menu, and our engine's having no route-editing mode, are design rules and need no
      backing. That the original's bound of 4 for the short right click does not follow the
      host's drag settings is `unmeasured`: part y (5) runs on one host, and no scenario here
      changes the host's drag settings. The order in which the other holders of a last point are
      added to the selection is `unmeasured`: part y records which objects are selected, not
      their order, and no run here makes an outcome depend on that order
    - Confidence: medium for selection acting as usual, an accepted band box included, the
      selection of the other holders of a last point, by a click or a band box, and the order in
      which they are added (unmeasured), the cleared point selection, the tools and their
      orders sent at once (the beacon mode's included), and the end at the key's release, with
      no event other than those listed ending route mode; medium
      for a right click acting as outside route mode and never ending it, the point selection
      cleared by a right click under 4 pixels and kept by one of 4 pixels or more, the original's
      bound of 4 not following the host's drag settings (unmeasured), the tool modes
      and route mode not excluding each other, and no input of the original starting
      route-editing. Where the original's click ends and its drag begins is S-0214 claim
      27's, and this claim asserts no bound for it. Low for
      the command bar's button and the defeat. That the original's route mode also ends at a
      focus loss or at the options menu, as our engine's design rule has it, is low

17. **Every order to an actor ends its route.** Any order that names an actor, apart from the
    route-mode orders themselves, removes that actor's planned route. This happens when the order
    is applied, for any actor that still exists, whether or not the order then passes the checks of
    its kind: a behaviour order whose target died in the same tick, a duplicate enter order that
    part (a) of table T3 ignores, and an order to an actor carried inside a transport all end the
    route. An object whose route still holds a point when the order is applied stops being a
    route holder and lowers the house's route-holder count (Terms) at once (claim 16f). Orders
    that a
    route itself issues for its
    points (claim 16c) do not end it.
    - Backing: `measurable` (Scenario O8, steps 2 and 4) for the removal of the route by an order
      that takes effect, and (Scenario O13, part c) for an order that fails its target check.
      The other failing cases are `unmeasured`: an actor that fails its own checks would have
      to be a route holder whose state changes in the tick between click and application (the
      input rules of S-0214 test the same conditions at the click), and its route would have
      to be observable afterwards.
    - Confidence: medium

18. **Key commands.** The commands of table T2 turn one key press into orders for the selection.
    Each command uses S-0214's selection order (Terms) and issues at most one order
    for each selected object, under the conditions in the table. The pending-list limit
    therefore drops the last objects in that order, not necessarily those selected last.
    Keyboard-command feedback is owned by S-0701 claim 19 and Scenario D4: outside
    route mode, guard, stop and scatter request one command cue for a nonempty selection,
    even when no selected object qualifies for an order. An empty selection requests none.
    Failed order issuance suppresses the cue even if earlier objects already received orders.
    In route mode the guard command's refusal message and sound (table T2) take the place of
    the ordinary guard cue. Deploy's game-wide command cue and
    its silence quirk are also S-0701's, not a change to deploy eligibility here.
    Ordinary click-response selection is instead S-0214 claim 14 and S-0701 claim 18.
    Neither kind of feedback establishes that an order took effect at application.
    - Backing: `measurable` (Scenarios O5 and O6; Scenario O2 for order precedence at the limit,
      and S-0701 Scenario D4 for keyboard-command feedback)
    - Confidence: medium (high for stop and scatter); high for the feedback gates, as in
      S-0701 claim 19

19. **Cheer rate limit.** The cheer command is accepted only if the current tick is more than
    *R* ticks after the last accepted cheer, where *R* is the general rules' maximum-cheer-rate
    field. An accepted press records the current tick as the last accepted cheer even if its order
    is then dropped by claim 2. The last-cheer tick is remembered across the games of one session.
    At program start it is −100, and a game start does not reset it. So in the first game of a
    session the first cheer is accepted only after tick *R* − 100. In a later game no cheer is
    accepted until the tick counter passes the tick of the last cheer of an earlier game plus *R*.
    - Backing: `measurable` (Scenario O7) for the strict interval and session carry-over.
      Remembering an accepted cheer whose order is dropped is `unmeasured`: no scenario
      here fills the pending list before a cheer input in that same tick.
    - Confidence: medium
    - Quirk: `quirks.cheer_memory_spans_session` (default: original). With the switch off, the
      remembered tick is set to −100 at every game start, so it never carries over from an
      earlier game.

20. **Guard-spot order.** The order names an actor and an object or cell, and sets the actor's
    guard spot to that object or cell. It needs only a live actor: it is applied to an actor that
    is off the map, and it checks neither hit points nor owner (claim 15). Like every order it
    passes through the pending list (claim 2). For a factory the guard spot is its rally point,
    the place toward which its new units head (production specs). Input creates the order for a
    selected building of the player's in three cases (S-0214, table M, part 3):
    - **(a) Rally point on a cell.** A rally-capable building clicked on a cell (S-0214, row BC5).
    - **(b) A building that undeploys into a vehicle, on a cell.** A move or harvest click on a
      cell with such a building selected. The same click then also issues a sell order, with
      which the building packs up (claim 12).
    - **(c) Move on an object.** A building whose action on a clicked object is *move*: for a
      factory of infantry, vehicles or aircraft the guard spot is a cell chosen around the clicked
      object's cell, as below; for any other building the guard spot is the clicked object itself,
      with no search. A building gets *move* on an object only when it is movable in the sense of
      S-0214 (Terms), for example a rally-capable building clicked with force-move held
      (S-0214, row G7).

    In cases (a) and (b), and for the factories of case (c), the cell is chosen when the click is
    made, not when the order is applied: it is the cell that the nearby-cell search of S-0212
    (claim 25) finds around the clicked cell (or the clicked object's cell), within the reach
    region of the building's own cell for a movement zone that depends on the building's type:

    | The building's type | Movement zone of the search |
    |---|---|
    | Naval (whatever it builds) | The naval zone |
    | Not naval, a factory of aircraft | The flying zone |
    | Any other | The ground zone |

    An aircraft factory reaches the flying case only when it gets a move action at all, that is,
    when its type is also a repair depot (which makes it rally-capable) or undeploys into a
    vehicle (S-0214, row B9). Which of the qualifying cells
    results follows S-0212's rules for the search (no reference cell is given; confidence low). So
    a clicked cell on open, flat ground is the result itself, and a blocked one is replaced by a
    usable cell near it. When the search finds no cell, no guard-spot order is issued, with one
    exception (confidence low): a construction yard whose house is human-controlled, in a game that
    is not a campaign game, with the option that lets construction yards be packed up again on,
    and that is not mind-controlled, gets the cell it stands on as its guard spot (an AI house
    meets the human-controlled condition, as in claim 12). In case (b) the
    sell order is issued whether or not the search found a cell.
    - Backing: `measurable` (Scenario O14) for cases (a) and (c), and (Scenario O15) for case (b).
      "Off the map" is `unmeasured`: input cannot click a building that is off the map. The case of
      a failed search is `unmeasured`: it needs a clicked cell with no usable cell within the
      search's reach, which a map can hardly provide for a ground factory. The flying zone of an
      aircraft factory is `measurable` with a named test rules file that makes an aircraft factory
      type a repair depot (Scenario O14, step 5).
    - Confidence: medium for the effect, the check, the three input cases and the search around
      the clicked cell; low for the movement zone used per kind of building and for the choice
      among qualifying cells

21. **Destination assignment, not an order name, governs prone get-up (Q-0087).**
    S-0107 claim 15 owns the rule: an eligible prone infantry unit asks, not forced, for
    *get-up* only when assigned the same non-empty destination it already has. Eligibility
    requires a human-controlled owner (including our AI houses), a type that is neither a
    fraidycat nor a cyborg, and a sequence that permits the request. A first destination,
    a different destination and clearing a destination make no such request. Equality means
    the same cell or the same object, not different objects occupying the same cell.
    The following are the destination requests made by the orders of this spec.
    The jumpjet filters below can suppress a destination assignment without cancelling
    the order's other effects.

    | Order | Destination request at application, after its checks pass |
    |---|---|
    | Plain move | The order's destination, after any group spreading of claim 7 |
    | Attack-move with a destination | The same assignment as plain move after claim 6's translation |
    | Area guard to a mobile actor | The order's target as destination and guard spot, even when it is the actor's own cell (table T3, part e) |
    | Enter or capture | The order's destination, normally the object to enter or capture (table T3, part f; S-0214 table M). A duplicate enter ignored by table T3 part a makes no assignment |
    | Ordinary attack on an object | Target becomes the object, destination becomes nothing (table T3, part f). Being out of range does not make the order itself assign a chase destination |
    | Stop | Requests an empty destination while the target remains; balloon-hover types also request it after target clearing (claim 8). The final destination is empty; neither request can cause destination-driven get-up |
    | Deploy | Requests an empty destination once, before target clearing; qualifying balloon-hover retention can preserve a nonempty destination (claim 10). No destination-driven get-up |
    | Scatter | Claim 9 sends a request, not a move order. Only a destination assignment made by an accepted request is subject to the equality rule (S-0208 claims 11 to 13) |
    | Queued move or route point | Recording a future point is not destination assignment. When following it assigns a destination, S-0107's same rule applies; route following is distinct from a plain behaviour order (claims 16 and 17; S-0215 claims 13 and 14) |

    **Destination filters and clears for jumpjets.** S-0502 claims 23, 24, 27 and 28 qualify these
    requests, including table T3 parts (e) and (f):
    - Outside an ion storm, jumpjet-type infantry ignore a nonempty destination assignment
      whose cell equals their current cell, before any movement-point search. This is a
      test of the type's jumpjet flag, not merely its locomotor. It applies while grounded
      too: an ordinary own-cell order does not promise take-off, an assignment or get-up.
      It does not prevent the order's separate target, behaviour or guard-spot effects.
    - Infantry has no vehicle unchanged-destination suppression. Subject to the own-cell
      filter and ordinary legality, repeating a nonempty destination elsewhere, whether
      the same cell or the same object, repeats movement-point adjustment and its
      eligible standing-position draw over 0 to 3 (S-0502 claims 16, 23 and 24).
      Search failure and pre-draw refusals still apply; a draw is not unconditional.
      Repeating an empty destination is an effective clear and reroutes with a commitment.
    - An ordinary jumpjet vehicle ignores an unchanged order-level destination, including
      empty-to-empty, unless destination refresh requires reassignment (S-0502 claim 27).
      Refresh starts absent, is temporarily enabled during placement's initial idle setup
      and is cleared when that setup ends. Its only later activation is claim 27's
      qualified teleport-capable vehicle/unloading-dock fallback, not successful
      restoration of suspended locomotion. Stop, deploy, move, behaviour changes,
      flight state, commitment and landing requests do not independently enable it.
      An otherwise eligible assignment consumes pending refresh even when changed or
      when point adjustment fails; a rejected balloon-hover clear does not consume it.
    - A balloon-hover vehicle rejects an empty assignment when it has both a nonempty
      destination and a target, and either the destination is that target object, the
      destination is a cell whose first ground-layer listed object is the target, or
      the horizontal target direction is within 16384 facing units of its current
      turret facing (S-0502 claim 28). The angular comparison uses the absolute signed
      wrapped difference in −32768 to 32767 facing units, inclusive at 16384, with
      integer arithmetic and no rounding. It uses turret facing even without a turret,
      not movement heading. Rejection retains the destination, movement point,
      commitment and reservation, without search or movement-position draw. If the
      current behaviour, or queued behaviour when none is current, is not *attack*,
      *attack* becomes current and queued behaviour is cleared. This rejection precedes
      equality suppression even with refresh pending; it is retention, not substitution.
      An empty destination, absent target or nonempty requested destination cannot
      trigger it. Claims 8 and 10 distinguish stop's retry from deploy's single request.
      A suppressed unchanged assignment likewise neither reroutes nor erases the point.
    - Every effective destination clear reroutes a committed jumpjet under S-0502 claims
      17 and 24, whether caused by stop, deploy or behaviour-order application. The
      order-level destination is empty after rerouting, but the replacement movement
      point and commitment survive. Changing behaviour alone is not a destination clear.
      Without a commitment, the clear has no locomotor effect. Claim 8 gives the
      successful reroute's flight-state consequences; failure has S-0502 claim 17's outcome.

    These filters and clear outcomes inherit S-0502 claims 23, 24, 27 and 28's backing
    and confidence: `measurable` (J2 and J16 for own-cell infantry rejection; J19 and J22
    for effective clears and ordinary vehicle suppression, J23 for infantry reassignment,
    J24 for placement refresh and ordinary-order nonactivation; J25 for ordinary
    balloon-hover move/attack/stop retention and retry). Confidence is high for the
    predicates and dispatch. The later qualified teleport/dock refresh on a jumpjet,
    combined balloon-hover infantry and stock deployable balloon-hover fixtures remain
    `unmeasured` with those scenarios' diagnostic limitations. Exceptional aftermath
    retains medium confidence.

    A qualifying reassignment at order application requests get-up at that boundary, before
    any movement in the next tick (claim 1), not at locomotor start. Merely moving to the cell
    the unit occupies does not qualify when it had no destination; repeating that cell while
    it is still its destination does, unless the jumpjet own-cell filter suppresses assignment. A refused sequence request is not saved until movement
    starts. Later destination assignments during behaviour processing or movement can request
    get-up on a later tick; S-0215 and the movement specs own when those assignments occur.
    No order-family whitelist is imported.

    **Attack order versus approach (Q-0072).** For an ordinary armed infantry unit or vehicle,
    an attack order with an object target and no destination leaves that target and an empty
    destination at the order boundary, subject to the jumpjet destination filters above. This does not mean the actor remains stationary:
    S-0215 claim 37 owns the subsequent approach step's trigger, gating and destination
    renewal, including its medium-confidence zero-speed qualifications in part g; those
    qualifications do not promise displacement. S-0307 claims 15 to 18 own the covered
    approach-position searches, failure outcomes and vertical balloon-hover exception.
    S-0207 claim 19 owns the driving, sailing and hovering vehicle outcomes once in range.
    S-0208 claims 16 and 17 own walk completion and the conditional infantry stop on arrival.
    None of those is an additional effect of the order here. Buildings likewise receive
    the target without a movement destination and never take that approach step
    (S-0215 claims 32 and 37 g). Special infantry actions that use entering,
    capture or sabotage are not this ordinary-attack case (S-0214 table M).
    - Backing: `measurable` (Scenario O17, Scenario O3 for the attack-order references, and
      S-0107 Scenario M4 for equality, type exceptions and sequence refusal). Later approach
      and route scheduling are claims of their owning specs, not new timing claims here.
    - Confidence: high for destination equality and the assignments of table T3; medium for
      their observable timing through input and for route following, as in claims 1 and 16

22. **Fresh-profile keyboard bindings (Q-0286).** With the original's default keyboard
    preferences, table T2's commands are bound to S for stop, X for scatter, G for guard,
    D for deploy, C for cheer and Z for route mode. The first five act on key press, not
    key release. Route mode uses Z held down, with release committing the route (claim 16).
    These are keyboard-preference defaults, not unit-rule values: a player's saved bindings
    replace them. A fresh profile means no user override of the supplied keyboard preferences,
    not a preferences file from which individual command entries have been removed. A command
    entry removed from a successfully loaded preferences file has no binding, rather than
    automatically regaining its supplied default. This applies to the six commands of table T2,
    not to the reserved Escape key of claim 23.
    The three physical force gestures and entry into attack-move mode are owned by S-0214
    claims 5 and 23, tested by its Scenario T17, not by this table.
    The four control-group commands of S-0214 claim 33, for each of the ten groups, are player
    bindings too. Their defaults are the group's digit alone to select the group, Shift with the
    digit to add the group to the selection, Ctrl with the digit to make the group from the
    selection, and Alt with the digit to select the group and centre the view on it. Digits 1
    to 9 stand for groups 1 to 9 and digit 0 for group 10. No default command adds the
    selection to a group: the player selects the group, adds to the selection and makes the
    group again. The group commands act on key press and issue no orders (S-0214 claim 33), so
    table T2 does not list them.
    - Backing: `measurable` (Scenario O19, parts a and d)
    - Confidence: high

23. **Options and view keys (Q-0286).** In ordinary running skirmish play, Escape opens the
    in-game options menu. Once open, the stand-still and resumption rules are S-0201 claim 8's,
    not an order to a selected unit. Escape is reserved
    for the options command even when the saved keyboard preferences omit or reassign it.
    With no modifier held and no conflicting saved command binding, holding Left, Right, Up
    or Down scrolls the battlefield view in that screen direction; releasing an arrow ends
    that arrow's contribution when the release input is accepted. These four keys are the
    original's fallback view controls, rather than four entries in table T2. S-0101 claims
    14 and 15 own displacement, simultaneous directions, scrolling limits and freeze behaviour.
    **Arrows during an order block (Q-0391).** A new arrow press or release made
    while orders are blocked is discarded, not saved for the end of the block. This
    applies to the initial block and to a chrono flash's block, including its unfrozen
    cycles. An arrow whose press was already accepted keeps its scrolling contribution
    even if physically released during the block. Releasing and pressing it again
    during the block changes nothing. Conversely, an arrow first pressed during the
    block contributes nothing, even if still physically held in the first unblocked
    cycle. Holding it does not generate a newly accepted press: a release and fresh
    press after the block are needed. An already accepted contribution ends when a
    release is accepted after the block. S-0101 claim 15 still suppresses displacement
    during frozen cycles and stand-stills, without clearing the contribution.
    These rules concern events made during the block, with no older waiting input.
    They do not cancel events already accepted into the waiting inputs before it.
    - Quirk: `quirks.blocked_arrow_events` (default: original). With it off,
      ordinary view-arrow presses and releases remain accepted during an order block,
      although unit orders remain blocked and frozen cycles still do not scroll.
      Thus a release stops a retained direction and a new press can scroll during
      unfrozen blocked cycles. This changes presentation only.
    This claim does not specify options-dialog closing keys or modifier-assisted scrolling.
    S-0214 claims 5 and 23 and Scenario T17 answer the physical force-gesture and
    attack-move-entry part of Q-0286.
    - Backing: `measurable` (Scenario O19, parts b and c; Scenario O22 for blocked
      arrow events; S-0101 Scenario M7 for displacement)
    - Confidence: high

24. **Edge-scroll cursor (Q-0277).** During ordinary edge scrolling, the cursor
    points in the requested screen direction, not a map-grid direction. The outer edge
    of the game view, including its sidebar portion, selects this presentation; the
    internal battlefield/sidebar boundary alone is not an outer edge. S-0101 claim 12
    and F9 own edge activation and direction, including the snapped client point,
    full-client centre, floating-point band comparisons and two-stage integer bearing
    quantisation. The image uses that requested direction, including the retained
    direction while edge momentum decays. No independent cursor angle is calculated.

    The cursor is the allowed variant when a trial scroll of one pixel in that direction
    can move the view under S-0101's limits; otherwise it is the blocked variant. This
    is not unit passability or order legality. Table E gives its static image and hot spot.
    The ordinary edge-scroll cursor is full-size, not a radar miniature. This claim
    does not specify drag-scroll controls or displacement per update.
    - Backing: `measurable` (Scenario O20)
    - Confidence: high for images and the one-pixel refusal test; medium for edge direction

25. **Route refusal text and shared status display (Q-0282).** A refused route click
    caused by the route-holder limit (claim 16) displays the route-holder-limit text; a selection
    whose objects are not in the same route state displays the incompatible-selection text.
    The other route-mode messages of claims 16b to 16e and table T2 are each their own text.
    These are imported translated messages (Import mapping), not hard-coded English
    sentences. Claim 16 governs their message-and-sound coalescing. A later refusal
    after a completed tick appends another message; it does not refresh an older copy.

    Each such message expires after 480 units of the real-time presentation clock,
    whose unit is 16 milliseconds, independent of game speed. Its deadline is the clock
    count when added plus 480, using exact integer arithmetic. It is removed at the
    first status-display update whose count is strictly greater than that deadline,
    not at equality. The clock continues during a timed freeze, options-menu pause
    or loss of focus. While the display is not being updated an expired message may
    remain in the last drawn image, but it is removed at the first subsequent update.
    resuming does not grant a fresh lifetime.

    The battlefield's shared status display holds at most six text lines, including
    these notices and other events' status messages, rather than six independent
    routes or six messages per event kind. Lines are left-aligned near the battlefield's
    top-left, oldest above newest. A new line at capacity removes the oldest line.
    expiry closes the gap. Long imported text wraps to further lines and each line
    counts against the same capacity. Each wrapped line receives its own 480-clock-unit
    lifetime when added, so lines added across a clock boundary can expire on different
    updates. In the original the
    first line begins 3 pixels right of the battlefield's left edge, at its top edge,
    with a fixed 19 pixels between line origins, independent of the imported font's
    line pitch and bitmap height. These pixel positions describe the original
    display, not a simulation-state rule. This claim specifies the shared display
    contract used by route refusals, not new events for sidebar, placement or sell/repair
    feedback; those remain with S-0411, S-0406 and S-0408 respectively.
    **Line splitting and font (Q-0393).** For an unprefixed notice, the width limit
    is the battlefield view's width minus 22 pixels after game start. After an
    in-game change of the battlefield view's dimensions, it is the new width minus
    16 pixels, until the next game start. The sidebar's width is not included.
    A video-mode change clears the existing status lines rather than re-splitting
    them. These are original
    presentation dimensions, not simulation quantities.

    Measurement and drawing use the imported game bitmap font, with its individual
    glyph widths and one extra pixel of advance per character, including the last.
    There is no kerning or extra proportional scaling. A tab advances to the next
    multiple of 64 pixels from the line origin. All width arithmetic
    is integer pixels, with no rounding. File identity is in Import mapping.
    S-0014 claims 1–3 supply the binary font layout and masks; claims 4 and 5
    supply the generated fallback image and mask transparency. This claim owns
    status-text metrics, splitting, control characters, fallback use and foreground
    selection.

    **Vertical metrics (Q-0530 questions 2 and 3).** The top stored row of each
    ordinary or fallback glyph lies at its line origin; row r lies at y + r,
    where y is that origin's vertical pixel position and r is counted from zero.
    No baseline adjustment, centring or displacement depends on bitmap height H
    or line pitch P. Blank leading bitmap rows remain blank rows at those positions.
    Changing P does not move subsequent stored lines, including wrapped lines,
    or change their capacity, lifetime or horizontal splitting.

    P instead supplies each stored line's vertical clipping height. In our engine,
    a glyph row is eligible to paint exactly when 0 ≤ r < min(H, P), also subject
    to the display surface's bounds. These are integer pixel quantities with exact
    arithmetic, no rounding and no floating point. A zero pitch paints no rows.
    A pitch smaller than H clips the bottom, rather than shifting or scaling the
    glyph. A pitch greater than 19 pixels can permit tall masks of neighbouring
    lines to overlap, without enlarging their origin spacing. It neither reserves
    an opaque background rectangle nor clears a pitch-sized band: clear mask pixels
    remain transparent. The original has a cosmetic clipping bug: it includes the
    bottom boundary at y + P, permitting row P when H > P; our engine excludes
    that extra row, with no quirk switch.
    - Backing: `measurable` (Scenario O21 d) for fixed spacing, top-row placement,
      clipping and transparency. Excluding the extra bottom row is a presentation
      design rule, not an oracle prediction.
    - Confidence: high

    **Glyph selection and drawing (Q-0501, questions 2 to 4).** Each unsigned
    16-bit text unit independently selects the font's imported glyph mapping,
    without reducing it to its low byte or combining surrogate pairs. A mapping
    value of zero means missing; a positive value selects glyph number one less
    than that value, with glyph numbers counted from zero. Aliases are allowed.
    A blank bitmap or a stored width of zero does not make a mapped glyph missing:
    a present zero-width glyph still advances by the one spacing pixel but paints
    no pixels in our engine, as S-0014 claim 3 specifies. The original has a cosmetic
    bug: a zero-width glyph can paint its first column when the first bit of a
    stored row is set. This drawing-only correction has no quirk switch.
    The zero text unit ends the text rather than selecting a glyph. Tabs and
    explicit breaks use their special rules here rather than mapped glyph metrics.

    A missing glyph uses a generated fallback, not a reserved glyph entry or a
    question mark. Its mask is the complement of the mapped capital X (text unit
    88), within that glyph's stored width and the font's bitmap height, and its
    width is the capital X's stored width. Thus changing that imported X changes
    both the fallback shape and its advance. These statements cover a valid font
    with a mapped capital X. S-0014 claim 4 owns the original's usable mapped-text
    case without X, with no defined missing-character fallback. Under S-0014
    claim 6 our engine rejects that font, or any structurally invalid resolved font,
    as a whole: report a missing-asset diagnostic naming the file and reason and
    omit status text, without partial use or a substitute font. This rejection
    is a design rule, not a prediction of the original's malformed-font response.
    Its engine acceptance test is S-0014 Scenario F5.

    Glyph masks have two values: zero leaves the destination pixel unchanged,
    and one paints the notice's foreground. They do not select palette indices.
    For route refusals the foreground comes from the local player's colour
    definition: convert its unmodified hue, saturation and value to RGB using
    S-0103 claim 6's integer conversion, then retain the display-format channel
    bits using S-0103 claim 9. No ramp-entry selection, shade subtraction or
    battlefield lighting is applied. Other events retain their own choice of
    colour definition. The fallback uses this same foreground in our engine.
    The original has a cosmetic bug: a fallback glyph inverts the even-numbered
    bits of the packed 16-bit foreground, counting from bit 0, rather than using
    the ordinary foreground. This drawing-only defect has no quirk switch.
    - Backing for glyph selection, generated fallback and original mask/colour
      behaviour: `measurable` (Scenario O21 c). The fixed fallback colour is a
      presentation design rule, not an oracle prediction. Suppressing zero-width
      pixels is likewise a presentation design rule; the original defect is
      `measurable` (Scenario O21 c; S-0014 claim 3).
    - Confidence for these glyph and colour rules: high

    A line contains at most 111 text characters. The width limit is inclusive:
    a character whose advance brings the accumulated width exactly to the limit
    fits. At a width overflow inside a word, the break is after the last preceding
    character whose value is at most 32, if one exists in the fitting prefix.
    This includes an ordinary space or a tab, not punctuation. Without such a
    separator, the fitting prefix of the word is used, so an overlong word can be
    split inside it. If the overflowing character is itself a separator, the line
    ends just before it. Reaching 111 characters makes a break there without
    searching back for a word boundary. Character counts refer to the string
    table's 16-bit text units, not encoded byte counts.

    An explicit carriage return or line feed has no horizontal advance and draws
    no glyph. It ends a line after that character, before any width overflow would
    be needed (Q-0501 question 5). At the start of a continuation,
    characters with values 1 to 31 are skipped, but ordinary spaces are kept.
    Thus a paired carriage return and line feed makes one break, and consecutive
    break characters do not create additional empty stored lines. Leading spaces
    of continuations are not generally trimmed. Every resulting stored line uses
    the capacity and lifetime above, including lines made by explicit breaks.
    An empty imported message adds one blank stored line with the same capacity
    and lifetime. Bad data that leaves no character fitting is covered
    in Edge cases and quirks.
    - Backing: `measurable` (Scenario O21, including part b for splitting and font)
    - Confidence: high for lifetime, capacity, splitting and font metrics; medium for
      pause observations and the two width regimes pending their resize test

## Formulas

> cheer passes the rate limit ⇔ *t* > *L* + *R*. *t* is the current tick, *L* the last accepted
> cheer tick (−100 at program start) and *R* the maximum-cheer-rate field. All are integers in
> ticks; the sum and the comparison are exact integer operations. (Claim 19. Open questions name
> one further cause for which a cheer is refused.)

> A route-mode click that claim 16d tests for the holder limit (not one that takes a point,
> selects, releases a band box or uses a tool) is refused by it exactly when \(C+S>128\),
> where \(C\) is the house's route-holder count (Terms) and \(S\) is the number of selected
> objects whose routes hold no point. These are exact integer counts. In the original \(C\) can
> differ from the number of route holders, and an object whose last point was deleted is
> counted in both \(C\) and \(S\) (claim 16f). (Claim 16, which also refuses a press outside
> every point's click area when a selected object cannot take part in a route.)

> Group destinations (claim 7). For the *n* members with positions *p*ᵢ = (*x*ᵢ, *y*ᵢ, *z*ᵢ) in
> leptons: centre *c* = (trunc(Σ*x*ᵢ / *n*), trunc(Σ*y*ᵢ / *n*), trunc(Σ*z*ᵢ / *n*)), integer sums
> and division truncated toward zero. *d*(*a*, *b*) = trunc(√((*a*ₓ − *b*ₓ)² + (*a*ᵧ − *b*ᵧ)² +
> (*a*_z − *b*_z)²)), the sum of squares and the root in floating point, the root being the
> approximate square root of S-0204, and the conversion to an integer truncating toward zero in
> every rounding mode (S-0205, claim 7). Leader *L* = the member with
> the smallest *d*(*p*ᵢ, *c*), the latest in the run on equal values. The other members have
> destination priority by increasing *d*(*p*ᵢ, *p*_L), ties in run order (claim 7).

> Look line (claim 7). Cells are written as integer pairs (*X*, *Y*) of cell coordinates. For a
> member in cell *m* and the leader in cell *ℓ*, let *v* = *m* − *ℓ* (integers) and *u* = *v* /
> √(*v*ₓ² + *v*ᵧ²), in floating point with the approximate square root of S-0204; *u* = (0, 0) when
> *v* = (0, 0), a case that does not arise, since a member in the leader's cell is given the
> shared destination before any look (claim 7). With the common cell *k* = (*k*ₓ, *k*ᵧ), the look's *j*-th point is
> *q*ⱼ = (*k*ₓ + ½ + *j*·*u*ₓ, *k*ᵧ + ½ + *j*·*u*ᵧ), in cell lengths, floating point, and its cell is
> (trunc(*q*ⱼ,ₓ), trunc(*q*ⱼ,ᵧ)), each conversion truncating toward zero in every rounding mode
> (S-0205, claim 7). The look is the cells of *j* = 1, 2, …, 6, examined in that order, for each
> member separately. Confidence: medium, as for the other placement details.

For claim 25, an ordinary character's text advance is its imported glyph width
plus 1 pixel, including a trailing character. A tab at advance *a* gives
64 × (⌊*a*/64⌋ + 1) pixels. These are non-negative integer pixels with exact sums
and floor division; carriage return and line feed consume no horizontal advance.
The unprefixed status-text budget is *W* − 22 pixels at game start and *W* − 16
pixels after a view-dimension change, where *W* is the battlefield width in integer
pixels. No floating-point arithmetic is used.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Pending-list capacity | 128 | orders | Claim 2 |
| Initial last-cheer tick | −100 | ticks | Claim 19 |
| Route-holder limit per house | 128 | route-holder count plus new objects | Claim 16 |
| Route length at which a point add is refused | 16 | points | Claims 16a and 16b: only at exactly 16; a longer route (claim 16b's copy) is not refused for its length |
| Fewest points in a loop that a press closes on a selected object's own route, unless that route changes between the click and the order's application | 3 | points | Claims 16b and 16c |
| Fewest loop points kept after a deletion | 3 | points | Claim 16e |
| Order delay | 1 | ticks | Claim 1: issued in tick *N*, first acted on in tick *N* + 1 |
| Input events taken per unfrozen cycle | at most 1 | events | Claim 1; frozen cycles take none |
| Smallest spread group | 2 | orders | Claim 7 |
| Largest level difference of a spread cell | 2 | levels | Claim 7: fewer than 3 levels from the common cell |
| Points of a spread look, for each member | 6 | steps of one cell length, the first one cell out | Claim 7 |

## State and ordering

**Table E: edge-scroll cursor images (claim 24).** Frames are zero-based in the
cursor image imported by S-0214. With full-canvas width *w* and height *h* in pixels,
centre means ⌊*w*/2⌋ horizontally and ⌊*h*/2⌋ vertically, exact integer division.
Right and bottom hot spots use *w* and *h*, not their last pixel indices.

| Screen direction | Allowed frame | Blocked frame | Hot spot (horizontal, vertical) |
|---|---|---|---|
| Up | 2 | 10 | centre, top |
| Upper right | 3 | 11 | right, top |
| Right | 4 | 12 | right, centre |
| Lower right | 5 | 13 | right, bottom |
| Down | 6 | 14 | centre, bottom |
| Lower left | 7 | 15 | left, bottom |
| Left | 8 | 16 | left, centre |
| Upper left | 9 | 17 | left, top |

**Table T1: the order kinds.** The terms *exists*, *live*, *on the map* and *valid* are those of
Terms. All checks are made when the order is applied; an order that fails one has no effect apart
from claim 17. Every row that carries an actor also ends the actor's planned route (claim 17).

| Order | Carries | Checks when applied | Result |
|---|---|---|---|
| Behaviour | actor, behaviour, target, destination | actor valid; a named target or destination that is a unit or building valid | Claims 5 to 7 |
| Stop | actor | actor live, on the map, not tethered, not under construction or being sold, not in the unidentified state (Open questions; interim rule in claim 8) | Claim 8 |
| Scatter | actor | actor mobile, live, on the map, not tethered | Claim 9 |
| Deploy | actor | actor live, on the map, not tethered, not an aircraft, not paralysed, not under construction or being sold, not in the unidentified state (Open questions; interim rule in claim 8), not on a vehicle-factory cell | Claim 10 |
| Detonate | actor | actor exists and carries a planted bomb; issued in the planter's name (claims 3, 11) | Claim 11 |
| Sell | actor | actor live and owned by the issuer; a vehicle or aircraft must stand on a building | Claim 12 |
| Guard spot | actor, object or cell | actor live (it may be off the map) | The actor's guard spot becomes the object or cell (claim 20) |
| Sell wall | cell | (economy specs) | The wall in the cell is sold |
| Repair | actor | actor live (it may be off the map) | Repair switched on or off (claim 13) |
| Power off, power on | actor (a building) | live and on the map; no firestorm-wall flag | Building switched off or on (claim 13), under the further conditions of S-0405 (claim 10) |
| Primary | actor (a building) | live (it may be off the map); imported factory category not the no-factory value, including recognised non-producing categories | Primary designation switched (claim 13) |
| Place | building type, cell | (production specs) | A finished building is placed |
| Build, hold, cancel, cancel all | type | (production specs) | Production started, held or cancelled |
| Fire superweapon | superweapon, cell | S-0512 claim 8: ordinarily granted, ready and charge clock running, not a direct suspension check; claim 10: second-stage orders accepted unchecked under `quirks.second_stage_order_unchecked`; claim 19: drain-mode state transitions instead | Charge and state changes follow S-0512 claims 8, 10 and 19; targeting, record choice and effect-level launch checks follow S-0513 claims 1 to 3 |
| Alliance | other house B | both multiplayer alliance flags on (S-0414, claim 14); when issuer A does not yet ally B: B is not A, B's country is not of the civilian side, A is undefeated, and the last-enemy test of S-0414 claim 12 passes (with exactly one enemy contender, A can ally nobody); when A already allies B: neither house is passive (S-0414, claim 13) | The issuer allies the other house one way, or the alliance ends in both directions (S-0414, claims 12 and 13) |
| Surrender | none | issuing house undefeated and with no result pending (S-0414, claim 11) | The issuer's objects are destroyed in its next house update, and the house is defeated when its defeat test then fails, which may be later or never (S-0414, claims 5 and 11); the game ends only through S-0414 claim 6 (play to the last side; a defeated person keeps watching); legal for every player house, human or AI |
| Game speed | speed index | none | Game speed changed; network games only (claim 14; S-0201, claims 8 and 13) |
| Cheer | none | none | All of the issuer's infantry cheer |
| Route point add, join, commit, delete | route data | (claim 16) | Route mode (claim 16) |

For the fire-superweapon row, sidebar eligibility (S-0512 claim 16) and cursor target offers
(S-0513 claims 1 and 2) govern issuing an order, not checking it again when applied. In
particular, suspension normally stops the charge clock, but a suspended record with a running
clock can pass claim 8's original order check. Applied orders do not repeat the cursor's allied-building
test for a force shield; effect-level readiness and legal-cell exceptions remain S-0513 claim 3's
rules. These companion claims own the confidence, backing and measurement scenarios for this row.

**Table T2: key commands.** The command keys are the player's own key bindings, initially those
of claim 22. "Selected object"
means each object in S-0214's selection order (Terms). *Movable*, *able to fire*, the *deploy check* and
*deploy readiness* are the input-side conditions of S-0214 (Terms). An object that fails a row's
condition gets no order.

| Command | Condition per selected object | Order issued |
|---|---|---|
| Stop | movable or able to fire | Stop |
| Scatter | movable | Scatter |
| Guard | movable, able to fire and not a building; or a vehicle that is a harvester or controls slave units, movable or not | Behaviour order *area guard*, with the cell the object stands in as its target |
| Deploy | see the deploy rule below | Deploy |
| Cheer | (whole house, rate-limited, claim 19) | Cheer |
| Route mode | held down / released | Claim 16 |

While route mode is held, the guard, stop, scatter and deploy commands issue no orders and add no
route point. Each is refused with its own message and a sound (Import mapping). The guard command
adds no guard action to the routes.
- Backing: `measurable` (Scenario O8, part f)
- Confidence: high for no order and no route point; medium for the four separate messages and
  the sound

**The deploy rule.** If the first object in selection order is a building of the player's that holds
garrison occupants, only that building gets a deploy order. Otherwise each selected object gets a
deploy order if it is a unit or building, is movable or able to fire, is not berserk, and passes
both the deploy check and deploy readiness of S-0214. Deploy readiness is never met by a
building, so apart from the garrison case the command orders units only. There is one exception,
for in-place deployers (S-0214, Terms): if the selection holds both in-place deployers that are
deployed and in-place deployers that are not, the deployed ones get no order. So one press deploys
the rest of a mixed group instead of undeploying the deployed members.

**Deployed, for the deploy rule (Q-1753).** The press sorts the selected in-place deployers into
deployed and not deployed by their state at the press, and the sorting counts a deployer that is
part-way through a transition by the side it is heading to:

| In-place deployer | Counts as deployed | Counts as not deployed |
|---|---|---|
| Infantry | its current sequence is one of the deploy kinds (S-0107, Terms), *deploy* itself included, so a unit in its *deploy* sequence | any other sequence, *undeploy* included, so a unit in its *undeploy* sequence |
| Vehicle | it is deploying (the deploying state of S-0517 claim 4), or it is deployed and not undeploying | it is undeploying, or it is undeployed |

This sorting, including how it counts a deployer that is part-way through a transition, is the
original's rule; S-0107 and S-0517 only name the sequences and states it reads, and this spec
states how the press reads them. The sorting looks at every selected in-place deployer, including one that then
fails another condition of the rule and gets no order, so such a member can still make the
selection mixed. A press never interrupts a transition that is under way. What a member that is
mid-transition does with an order follows from the rules this order meets on the way:

- **Infantry whose type's undeploy delay is 0 or more** lacks deploy readiness during *deploy*,
  every deployed sequence and *undeploy*, so it gets no order. This reads S-0214's *deploy
  readiness* (Terms) with "deployed" covering every deploy kind of sequence, *deploy* itself
  included, and "the timer that follows a deploy" being the unit's rearm countdown; S-0214's
  committed wording says neither outright, and Changes for other specs gives it the wording. A
  deployed one undeploys later by itself (S-0215 claim 31).
- **Infantry whose type's undeploy delay is negative** has readiness whatever its sequence, and gets
  the order unless it counts as deployed in a mixed selection. The order queues *unload*
  (claim 10), but *deploy* and *undeploy* are not interruptible (S-0107, table T1), so the unit is
  not ready to change (S-0215 claim 11) until its sequence ends. It finishes the transition it is
  in, and then *unload* starts the opposite one (S-0215 claim 30): a unit that was deploying
  finishes deploying and then undeploys, and one that was undeploying finishes and then deploys
  again.
- **A vehicle** that deploys in place has deploy readiness whatever its transition state (S-0214,
  Terms: none of its conditions for a vehicle names a transition state), so it gets the order unless it counts as deployed in a mixed selection.
  During an uninterrupted transition its current behaviour is *unload* (S-0517 claim 3: the transition
  advances only in the processing of *unload*, every tick until it finishes), and queueing
  the current behaviour with nothing queued changes nothing (S-0215 claim 7). The transition
  carries on to its end, and completion gives *guard* as usual, so the vehicle does not go the
  other way afterwards. A behaviour forced on the vehicle mid-transition is left to S-0517's
  Edge cases. The order's other effects (contact, target and destination, claim 10)
  still happen. If some other behaviour is queued at the press, the order replaces it with
  *unload*, and completion discards that too under S-0517 claim 8's
  `quirks.deploy_transition_discards_orders` (default original); with that switch off, the
  vehicle would go the other way after completing.

So, for two members of one type:

In the next table, a member *in its deploy sequence* is an infantry unit whose current sequence is *deploy*, or a vehicle in the deploying state of S-0517 claim 4; *in its undeploy sequence* likewise means the *undeploy* sequence or the undeploying state.

| Selection at the press | Infantry, undeploy delay negative | Infantry, undeploy delay 0 or more | Vehicle |
|---|---|---|---|
| one deployed, one in its deploy sequence | not mixed: both get the order; the deployed one undeploys; the other finishes deploying, then undeploys | neither gets an order | not mixed: both get the order; the deployed one undeploys; the other finishes deploying and stays deployed |
| one in its undeploy sequence, one undeployed | not mixed: both get the order; the undeployed one deploys; the other finishes undeploying, then deploys | only the undeployed one can get an order: it gets one only if it has readiness (its rearm countdown has ended, as read above), and then deploys | not mixed: both get the order; the undeployed one deploys; the other finishes undeploying and stays undeployed |
| one in its deploy sequence, one in its undeploy sequence | mixed: the one in its deploy sequence gets no order and stays deployed; the other finishes undeploying, then deploys | neither gets an order | mixed: the one in its deploy sequence gets no order; the other gets the order; each finishes its own transition |
| both in their deploy sequence | not mixed: both get the order; each finishes deploying, then undeploys | neither gets an order | not mixed: both get the order; each finishes deploying and stays deployed |
| both in their undeploy sequence | not mixed: both get the order; each finishes undeploying, then deploys | neither gets an order | not mixed: both get the order; each finishes undeploying and stays undeployed |

Infantry and vehicles therefore differ (Q-1753 question 4). A press can make an infantry unit with
a negative undeploy delay go back after its transition, and refuses one with a delay of 0 or more
outright. It reaches a vehicle without changing the transition's result. The selection is sorted at
the press, while the order is applied later (claim 3) and *unload* reads the unit's sequence or
state when it starts. The rows for a member in its deploy or undeploy sequence hold while its
transition is still under way when the order is applied. A member whose transition ends in the
tick of the press, before the order is applied, meets the order as a deployed or undeployed member
does, so a vehicle among them that gets the order then makes the opposite transition (S-0517
claim 3).
- Backing: `measurable` (Scenario O6 part B, cases a to e), except for three parts that are
  `unmeasured`:
  - that the sorting counts a member that then fails another condition of the rule, since no
    case of O6 part B has such a member in a selection whose sorting it could change,
  - sorting at the press rather than when the order is applied, and the course of a member whose
    transition ends in the tick of the press, since O6 part B keeps each press clear of every
    transition's end,
  - the sorting for infantry whose type's undeploy delay is 0 or more: such a member lacks deploy
    readiness whenever it is deploying, deployed or undeploying, so whichever side it is counted
    on, no two-member selection of that type gets a different outcome, and no case can tell the
    sorting apart.
- Confidence: medium. The sorting, the readiness and the course of each transition are
  `measurable` by O6 part B and stay medium until it runs; the three `unmeasured` parts are
  medium.

**Table T3: effects of a behaviour order (claim 5).** The actor is valid, and the order's target
and destination pass claim 4. If part (a) holds, the order is ignored (claim 17 still applies)
and parts (b) to (h) do not apply. Otherwise every part from (b) to (h) whose condition holds
applies. "The resulting behaviour" is the order's behaviour, or for an attack-move order the behaviour that claim 6 gives.

| Part | Condition | Effect |
|---|---|---|
| (a) | The behaviour is *enter*; the actor is a mobile actor whose current behaviour is already *enter*; the order's destination is a building (an enter order names the object to enter as its destination, S-0214); and the actor is already in contact with that building | The order is ignored, apart from ending the actor's planned route (claim 17). An enter order whose destination is a vehicle, such as a transport, is never ignored this way |
| (b) | Always | If the actor is not tethered, it breaks off any contact. If it is tethered, it keeps its link, unless the object it is in contact with is a building of the unloading kind (Import mapping), in which case it breaks off the contact and is no longer tethered. The actor also forgets any building it was waiting in line to enter |
| (c) | The actor is a team member (Terms) and the behaviour is not *unload* | It leaves the team. Only the original's computer player has teams; an AI house has none (Terms, *AI house*), so this part never applies to its units |
| (d) | Always | The resulting behaviour becomes the actor's queued behaviour, as S-0215 (claim 12) describes. When it becomes current belongs to S-0215. For a mobile actor, claim 6 also says what happens to its attack-move memory |
| (e) | *Area guard*, mobile actor | Target: nothing. Destination and guard spot: both the order's target (an object or a cell). Destination assignment uses S-0107 claim 15 (claim 21 here). The order's destination is not used. The queued destinations are kept |
| (f) | Any other behaviour, or *area guard* to a building | Target: the order's target. Destination: the order's destination. Destination assignment uses S-0107 claim 15 (claim 21 here). Either may be nothing. An open-topped transport (a type whose passengers can fire out of it) gives the same target to its passengers when the target is not nothing |
| (g) | As (f), and the actor is a mobile actor | It also forgets its guard spot and its queued destinations |
| (h) | The actor is a mobile actor controlling slave units (a slave-miner harvester, which works with its slaves) and the resulting behaviour is not *attack* | Its slaves are sent back to guard it |

Destination assignments in table T3 parts (e) and (f), including assignments that repeat the
current destination, use claim 21 and S-0107 claim 15, subject to claim 21's jumpjet
assignment filters and effective-clear rerouting. They must not be interpreted as an
unconditional get-up effect of a move behaviour. Scatter and route following use the same rule
only when they actually assign a destination.

**Ordering consequences.**
- Production starts are assigned scenario-stream values by their causing actions'
  precedence, not by a line-update draw batch. Cancellation order can therefore change both
  which replacements exist briefly at the boundary and which values they receive
  (claim 3, S-0404 claim 11, S-0411 claim 13). Turn-pass starts receive earlier values than order-boundary
  starts; across houses at the boundary, earlier-created houses receive earlier values under
  our engine's design extension. This constrains draw-to-start associations, not scheduling.
- When two successful orders of the same kind for the same actor are issued in one tick,
  any state changed by both has the value specified by the later-issued order. When a
  successful behaviour order and a successful different-kind order affect the same actor
  in one tick, any state they both change has the value specified by the behaviour order,
  regardless of issue order. An order that fails its application checks does not overwrite
  state changed by a preceding order (claim 4). Thus a behaviour order
  issued before or after a stop, scatter or deploy order determines the conflicting state
  (claim 3).
- An order is applied after the tick's object updates. Consequently, an object destroyed during
  tick *N* is no longer live when an order issued in tick *N* is applied; an order whose actor,
  target or destination must then be live or valid has no effect (claim 4, table T1).

## Edge cases and quirks

- **No further search after a spoiled look** (claim 7): the original gives a member its fallback
  cell without searching beyond the obstacle, so members can share a cell next to water, cliffs
  or buildings. This spec states that rule and adopts no switch. Whether our engine should also offer
  a search, as `quirks.group_spread_no_detour` (default on: the original), is a question for the
  project lead (Questions for the project lead).
- **Repeated-destination get-up** (claim 21): S-0107 claim 15 owns
  `quirks.prone_get_up_repeated_destination`, default YR behaviour. With it off, every non-empty
  destination assignment requests get-up for an otherwise eligible prone unit, not only a
  reassignment of the same destination. This applies across order families, not just move.
- **A spent route blocks joins** (claim 16b): `quirks.spent_route_blocks_join`, default:
  original. An object keeps its route record after its route is empty, and the point-click test
  then refuses its clicks on points that its route does not hold, so it can never join a point
  again. With it off, an empty route counts as no route record.
- **Route-holder count drift** (claim 16f): `quirks.route_holder_count_drift`, default: original.
  A deletion that empties a route leaves the count raised, and a join that begins a route does
  not raise it. With it off, the count is always the number of the house's route holders.
- **Hidden points refuse presses** (claim 16b): `quirks.hidden_point_refuses_click`, default:
  original. A press that takes no highlighted point is tested against points of every house, so
  a holder's press at another house's unseen point is refused. With it off, only points drawn
  for the player are tested.
- **Click areas skip the kind test** (claim 16d): `quirks.click_area_skips_kind_test`, default:
  original. A press inside a click area is not tested for kind, so an aircraft or building can
  join a point. With it off, every route-mode battlefield press with objects selected is tested
  for kind.
- **Point selection after the release** (claim 16e): `quirks.delete_selects_after_release`,
  default: original. A point delete applied after the key's release selects the next point,
  which the delete key then deletes unseen. With it off, such a delete selects no point.
- For the two switches above (click areas, point selection), the original side is the rule
  that claims 16d and 16e state, at their ratings there. What an aircraft or building does
  after it joins a point is claim 16d's, at its ratings there.
- **Stacked points take the earliest-made** (claim 16b, *Two points at one place*):
  `quirks.stacked_point_takes_earliest`, default: original. Where several points of the house
  stand at one place, a press takes the earliest-made of them, so a selection whose routes hold
  only a later-made one is refused on its marker. With it off, the press takes the
  earliest-made point that a selected object's route holds, else the earliest-made one, and its
  orders name that point itself. The original side is claim 16b's rule, at its rating there.
- **A press's copy ignores the route's state** (claim 16b):
  `quirks.press_copy_ignores_route_state`, default: original. The copy that a press gives an
  object that is not selected, or a selected object that has lost the pressed point, can take
  its route past 16 points, and its loop can begin at one of the object's own points and end
  before the route's last point. With it off, a press's join does nothing when the copy would
  take the route past 16 points, and the copy's loop always begins in the copy. The original
  side is claim 16b's rule, at its low rating there.
- **Blocked arrow events** (claim 23): `quirks.blocked_arrow_events`, default: original.
  Discarded releases can leave scrolling active after the key is let go.
  The alternative accepts view-arrow events during the block; it grants no unit orders.
- **Unfittable status text** (claim 25). A non-empty remainder whose first character
  cannot fit the width budget makes the original repeatedly add empty lines without
  consuming that character, and can stop the game with a program failure. Our engine
  omits that remainder and reports an importer warning for the offending imported
  text or font. This is a safe departure with no switch, not an inside-word break
  that silently loses its first character.
- **Partial group orders** (claim 2): `quirks.pending_order_limit`.
- **Detonation in the planter's name** (claim 11): `quirks.detonate_in_planter_name`.
- **Detonating a bomb that an AI house planted** (claim 11). The original drops the order, because
  its computer houses' orders are never applied; our engine applies it, under either setting of
  `quirks.detonate_in_planter_name`. This is a *departure* with no switch, from the project lead's
  decision that AI houses follow the order rules of human houses.
- **The firestorm-wall flag** (claim 13). The original reads a building-type flag, inherited from
  an earlier game of the series, that marks a building as a segment of an energy wall. No stock
  type sets it. A building with the flag ignores power orders, although the input rules of S-0214
  still let the player click it in power mode. The behaviour follows the imported flag, so a mod
  controls it through its data; no switch is proposed.
- **Session-wide cheer memory** (claim 19): `quirks.cheer_memory_spans_session`.
- **Repeated route refusal cues** (claim 16). Earlier refusals on completed ticks do not
  suppress a later click's cue. The same-tick duplicate-notice limit is ordinary feedback
  coalescing, not a once-per-game quirk.
- **Stop keeps the behaviour** (claim 8). A stop order leaves the behaviour as it was (apart from
  harvesters) and only removes the target and destination. This is deliberate behaviour and
  needs no switch.
- **Behaviour orders after other orders** (claim 3). The original's grouping makes a behaviour
  order win over a stop, scatter or deploy order given later in the same tick. This is the
  original's rule for the orders of one tick, not a defect of game state, so no switch is proposed.
- **No owner check** (claim 15). The original checks the owner only for the sell order. Our
  engine keeps that, for every house alike; ownership is kept by the input rules that every house's
  orders pass when issued (claim 3). An object whose owner changes between issue and application
  still gets the order, as in the original. No switch is proposed.
- **The click marker.** S-0214 claim 30 owns click-marker eligibility and placement.
  In a skirmish game the original creates it as an ordinary animation object.
  Our engine treats it as drawing only: it never takes part in
  game state and never draws from the scenario random stream. This is a departure, confirmed by
  the project lead. Because the original creates the marker only on the machine of the player
  who clicked, any effect it had on game state would already have split the original's own
  network games, so drawing only is consistent with the original. The glossary allows nothing
  that depends on the local player to reach game state.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| String-table entry `MSG:PlannerMaximum` | route-holder-limit text (claims 16 and 25) | translated text from the user's string table | missing-entry diagnostic, not substitute stock English |
| String-table entry `MSG:PlanningModeHeteroSel` | incompatible-selection text (claims 16 and 25) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoAircraft` | aircraft-in-selection text (claim 16d) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoBuildings` | building-in-selection text (claim 16d) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoUnknowns` | other-kind-in-selection text (claim 16d) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeInvalidNodeX` | invalid-point text (claims 16b and 16d) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PostTerminatingCommand` | after-terminating-point text (claim 16b) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PostContinualCommand` | after-continual-point text (claim 16b) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PostLoop` | after-loop text (claim 16b) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:NodeMaximum` | 16-point text (claim 16b) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeLoopHelp` | loop-made text (claim 16c) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeDeleteHelp` | point-selected text (claim 16e) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoGuardArea` | route-mode guard text (table T2) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoStop` | route-mode stop text (table T2) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoScatter` | route-mode scatter text (table T2) | translated text from the user's string table | missing-entry diagnostic |
| String-table entry `MSG:PlanningModeNoDeploy` | route-mode deploy text (table T2) | translated text from the user's string table | missing-entry diagnostic |
| Rules `[AudioVisual]` `AddPlanningModeCommandSound` | route-point sound (claim 16b) | sound name, resolved as the other interface sounds of S-0701 | not established (Open questions) |
| No rules, art or map key | route refusal duration and shared status display (claim 25) | fixed presentation rules | 480 clock units of 16 milliseconds; six lines |
| Game font file `GAME.FNT` | status-text glyph mapping, masks, widths, bitmap height and line pitch (claim 25) | unsigned 16-bit text-unit mapping; zero means missing, positive values select zero-based glyph number value minus one; widths in integer pixels, binary masks; resolved through S-0005 claims 1 to 4 and 18a | no original status text without a loaded font; our engine reports a missing-asset diagnostic naming the file and reason and omits the text for missing files or whole-font rejection under S-0014 claim 6; no partial use or substitute font; binary layout and masks are S-0014 claims 1–3, fallback image and transparency are claims 4 and 5 |
| Common bitmap height in `GAME.FNT` | stored glyph row count H (claim 25) | unsigned integer pixels, S-0014 claim 1; row zero starts at the line origin | no independent absent-field default; incomplete files are rejected under S-0014 claim 6 |
| Header line pitch in `GAME.FNT` | stored status line's clipping height P (claim 25) | unsigned integer pixels, S-0014 claim 1; does not set the fixed 19-pixel origin spacing; our engine paints only rows below min(H, P) | no independent absent-field default; incomplete files are rejected under S-0014 claim 6 |
| Mapped capital X in `GAME.FNT` | generated fallback shape and width (claim 25) | text unit 88, complemented mask with the same width and bitmap height | no reserved fallback entry; an absent X mapping rejects the entire font under S-0014 claim 6; the original's mapped-text case without X is claim 4 |
| Rules `[Colors]` | route-notice foreground (claim 25) | local player's chosen definition, parsed by S-0103 claims 4 and 5; unmodified HSV converted and packed by claims 6 and 9 | colour-definition import defaults remain S-0103's; no font palette or battlefield lighting |
| No rules, art or map key | status-text splitting (claim 25) | fixed integer-pixel budgets, 111-character limit, one-pixel character spacing and 64-pixel tab stops | claim 25's initial and resized budgets and separator rules |
| No rules, art or map key | acceptance of arrow events during an order block (claim 23) | fixed input rule, subject to the presentation quirk | presses and releases made during the block discarded |
| Cursor image and palette of S-0214 Import mapping | edge-scroll images of table E | same imported image and palette, full-size frames | same missing-file handling as S-0214 |
| Player's keyboard preferences, set through the key-binding screen | command keys of table T2 and the control-group keys (claim 22) | assigned keyboard key or key combination; preferences, not rules-layer fields | no user override: stop S, scatter X, guard G, deploy D, cheer C, route mode Z; for group *g* with digit *d* (1 to 9, and 0 for group 10): select *d*, add Shift+*d*, make Ctrl+*d*, select and centre Alt+*d*; a removed entry in a successfully loaded preferences file: unbound |
| No rules, art or map key | reserved options-menu key (claim 23) | fixed Escape binding | Escape remains available regardless of saved command preferences |
| No rules, art or map key | ordinary view scrolling keys (claim 23) | held Left, Right, Up and Down; fallback when not bound to another command | arrow keys; displacement and scroll preferences are S-0101's |
| Rules `[General]` `MaximumCheerRate` | *R* of claim 19 | integer, ticks (value form of S-0003) | 1 tick |
| Vehicle type sections, `Harvester` | harvester in claim 8 and table T2 | boolean (S-0003) | no |
| Building type sections, `WeaponsFactory` | vehicle-factory cell of claim 10 | boolean (S-0003) | no |
| Building type sections, `DockUnload` | the unloading kind of building of table T3, part (b) | boolean (S-0003) | no |
| Building type sections, `FirestormWall` | firestorm-wall flag of claim 13 | boolean (S-0003) | no |
| Building type sections, `Factory` | imported category: claim 13 tests any category except no-factory; claim 20's kind-specific tests use only the corresponding producing kinds | named value, S-0003 claim 21 and Import mapping row C19; other recognised categories are not aliases for producing kinds | no factory; missing or empty in a later layer keeps the value in force |
| Building type sections, `Naval` | the naval kind of factory in claims 13 and 20 | boolean (S-0003) | no |
| Building type sections, `UndeploysInto` | building that undeploys into a vehicle, which packs up when sold (claims 12 and 20) | vehicle type name (S-0003) | none |
| Building type sections, `ConstructionYard` | construction yard (claims 12 and 20) | boolean (S-0003) | no |
| Building type sections, `UnitRepair` | repair depot: rally-capable (Terms, claim 20); the stock case of a building under a sold vehicle (claim 12) | boolean (S-0003) | no |
| Building type sections, `Cloning` | cloning building: rally-capable (Terms, claim 20) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft type sections, `OpenTopped` | open-topped transport in table T3, part (f), and claim 8 | boolean (S-0003) | no |
| Infantry type sections, `Fraidycat`, `Cyborg` | type exclusions from destination-driven get-up (claim 21; S-0107 claim 15 owns the full rule) | booleans (S-0003) | no for each |
| Infantry type section, `Sequence`; its art sequence section, `Up` | sequence table and get-up entry used by claim 21 | section name; sequence entry as S-0107 Import mapping defines | no table; a missing entry has zero image frames and refuses a direct get-up request |
| No data key | order-to-destination assignments of claim 21, and the distinction between attack-order application and later approach | fixed behaviour, not a parsed list of movement orders | no imported order whitelist or chase destination |

For the rules and type rows, the defaults in the Absent column are the values the original gives
a field when the type or rules data is created. They are engine constants (absent-key defaults),
not values from stock rules (confidence: medium). Each rules layer that omits a key keeps the
value already in force (S-0006). Keyboard preferences are separate player settings, not a rules
layer: claim 22 distinguishes the supplied fresh profile from an omitted command entry. The
full keyboard-preferences file encoding is outside this spec.

## Measurement scenarios

All scenarios run under the measurement setup: a skirmish on a small purpose-made flat map, one
human player and one bystander computer player (glossary) whose start position is far away, with
the bystander's units out of reach unless a scenario places some of them (Scenarios O3, O5, O11 and
O13 do) or hands objects to it (Scenario O16 does). A recorder reads the tick counter and, where
named, an object's behaviour, target, destination, guard spot, contact, tether, position and owner, and
whether it is being sold, or the house's production, each tick. The recorder samples once per
tick, at the moment the tick counter advances, so a sample shows the state after all of that
tick's work, the applied orders included. The orders are given by hand, or by scripted input
where a scenario says so or exact timing helps. Scripted input is one of the allowances of the
measurement setup in the batch's glossary (*measurement setup*), added by the
project lead's decision of 2026-10-09: a script produces mouse and keyboard input through the
game's normal input only, as a player would, and never writes into the game.

Enter-order contact precedence (claim 3) is tested by S-0314 Scenario H18 f:
two vehicles ordered by one selection click give the first in selection order the free
depot contact, and the later vehicle the occupied-depot outcome of S-0314 claim 23.
Both taking the initially free contact, or the later vehicle taking it instead of the
first without an intervening release or refusal, falsifies this precedence.

Keyboard-command feedback (claim 18) is tested by S-0701 Scenario D4, including empty
and nonempty-but-ineligible selections, one cue for several ordered objects, failed issuance
at the pending-order limit, and route-mode guard. Record cue requests separately from orders
applied: a cue is not proof that an order passed its application checks.

The jumpjet destination assignments (claims 8, 10 and 21) are tested by S-0502 Scenarios
J2, J16, J19 and J22 to J25, with their setup and recorder requirements. In particular, compare
deploy with a nonempty order-level destination against deploy after stop has emptied it
while retaining a movement commitment. The effective clear reroutes and preserves the
replacement movement point; an ordinary suppressed empty-to-empty vehicle assignment
preserves the preceding point without another reroute. Record flight state and touchdown
position: a different replacement causes cruise before descent, and landing uses the
retained point. Erasing either retained point, or rerouting on a suppressed assignment,
falsifies the distinction. For jumpjet-type infantry outside an ion storm, J2/J16's
own-cell order must be rejected before point adjustment, including while grounded.
Starting ascent solely from that assignment falsifies the filter. These are companion
scenarios, not measurements already performed here.
For J23, compare repeated nonempty infantry assignments elsewhere and repeated empty
assignments before touchdown: each accepted reassignment repeats adjustment, including
its eligible standing-position draw, and each committed clear reroutes. Suppression
solely because the infantry destination is unchanged falsifies claim 21. J24 checks
that ordinary orders do not enable refresh and that placement's temporary refresh ends.
For J25, establish a balloon-hover vehicle's move destination, then attack a legal target
within the retention bound. Record destination, target, current and queued behaviour,
movement point, reservation and random stream at each attempted clear. A qualifying
attack clear retains movement without a draw; stop must still end with an empty
destination through its post-target-clear retry. Final retention after stop falsifies
claim 8. Deploy has no retry and can retain the nonempty destination; that subcase is
diagnostic until a stock deployable balloon-hover fixture exists. Record the same fixture
limitations as S-0502 rather than injecting state or claiming measurements have run.

**Scenario O1: tick boundary (claims 1 and 14).**
- *Setup:* two identical vehicles of the human player, side by side and selected together, facing
  the same free route; a barracks of the player's and enough credits. Game speed 6 (slowest).
- *Procedure:* (1) order both vehicles to move to a cell 8 cells away with one click. (2) Repeat
  10 times. (3) With nothing selected, start production of one infantry unit at the barracks from
  the sidebar. Repeat 10 times, cancelling in between.
- *Observable:* (1, 2) for each vehicle, the first sample in which its destination is set, and the
  first sample in which its position changes. (3) The first sample in which the item appears in
  the house's production, and the first sample in which its production progress is above zero.
- *Expected:* (1, 2) both vehicles get their destination in the same sample. In every run, the
  first position change appears at least one sample after the sample in which the destination
  appeared, because the order is applied after that tick's object updates. (3) Progress first
  appears at least one sample after the sample in which the item appears, because the order is
  applied after that tick's production lines (claim 1; S-0203, claims 7 and 8), so the production
  line meets the new item first in the next tick.
- *Falsified if:* the two vehicles get their destinations in different samples, a position change
  appears in the same sample as the change of destination, or production progress appears in the
  same sample as the new item. Step 3 assumes that starting production does not itself advance
  the progress (production specs); if it does, step 3 cannot falsify claim 14.

**Scenario O2: pending-list limit (claims 2 and 18).**
- *Setup:* 130 infantry of one cheap stock type, owned by the human player and placed by the map
  on open ground. Use a type whose primary weapon has positive damage. For the ordering
  test, add the objects individually to the selection in a recorded order, using scripted
  modifier-clicks if needed; band-box discovery order is not assumed.
- *Procedure:* (a) select all 130 and give one
  move click to a far cell. Repeat (a) 3 times from the same start. (b) From the same start:
  select 128 of them and give a move click to a far cell; select the other 2 and give a move click
  to the same far cell; while all 130 are moving, select all 130 and press the stop key once. Repeat (b) 3 times from the same start.
- *Observable:* (a) the number of infantry whose destination changes, and which ones fail, in
  selection order where the selection order is known. (b) After the stop key: how many have no
  destination and how many keep moving, and which ones keep moving.
- *Expected:* (a) exactly 128 change their destination. (b) Exactly 128 stop and 2 keep moving,
  and the 2 that keep moving are the last 2 in selection order. With the individually
  recorded additions of this positive-primary-damage type, those are the first 2 selected,
  not the last 2 selected.
- *Falsified if:* (a) all 130 respond, or fewer than 128 respond; (b) all 130 stop, or fewer than
  128 stop, or, where the selection order is known, the 2 that keep moving are not the last 2 in
  selection order.


**Scenario O3: behaviour order results (claim 5, table T3 parts (d) to (g), and claim 6).**
- *Setup:* one armed infantry unit of an attack-move-eligible stock type and one armed vehicle of
  the human player; one enemy vehicle in weapon range; one friendly vehicle; one open-topped
  transport of a stock type, of the player's, carrying armed infantry.
- *Procedure:* (1) order the vehicle to attack the enemy; (2) order a move to an empty cell; (3)
  give the vehicle area guard on the friendly vehicle (S-0214, table G, row G6); (4) give an area guard on
  a cell (S-0214); (5) give the infantry an attack move on the enemy vehicle with the attack-move
  gesture (5a), then an attack move to an empty cell (5b); (6) give the infantry area guard on a cell (S-0214), then a move to an empty cell 6
  cells away; (7) order the open-topped transport to attack the enemy.
- *Runs:* 3, from the same start. None of the recorded references depends on a random draw, so
  the runs only guard against mistakes in giving the orders by hand.
- *Observable:* after each order: behaviour, target, destination and guard spot; whether the
  vehicle stays near the cell or the friendly vehicle; after step 6, the infantry's guard spot in
  the sample after each of its two orders.
- *Expected:* (1) behaviour attack, target the enemy, destination nothing; (2) behaviour move,
  target nothing, destination a cell at or next to the clicked cell, guard spot nothing; (3) and
  (4) behaviour area guard, target nothing, destination and guard spot both the friendly vehicle
  or the cell, and the vehicle stays near it; (5a) behaviour attack with the enemy vehicle as target (not
  *attack move*); (5b) behaviour move with the cell as destination (not *attack move*); (6) after the area-guard order the guard spot is the cell, after the move order
  it is nothing; (7) every passenger's target becomes the enemy in the same sample as the
  transport's.
- *Falsified if:* any recorded reference differs from the expectation in any run.

**Scenario O4: duplicate enter, contact break and slaves (claim 5, table T3 parts (a), (b) and (h)).**
- *Setup:* (i) a harvester vehicle of the human player, loaded, and a refinery of the player's
  with its docking place free; separately (i′) an infantry unit of the player's with reduced hit
  points (set by the test map) and a hospital that the test map gives to the player. Both pairs
  are ones for which S-0214 gives the enter action (value *n* of table V and value *e* of table
  I), so a click on the building makes an enter order with the building as its destination. A
  named test rules file gives the refinery type and the hospital type the unloading-kind flag of
  the Import mapping, so that by part (b) any behaviour order that part (a) does not ignore breaks
  the contact, whether or not the exchange tethers the actor. (ii) A slave-miner harvester with
  its slaves, mining. (iii) As (i), with the same test rules file. (iv) As (i), but with a named
  test rules file that gives the refinery type the unloading-kind flag switched off.
- *Procedure:* (i) select the harvester and click the refinery; while it approaches, and again
  after the recorder shows it in contact with the refinery, click the refinery again, 5 times at
  intervals of about one second. (i′) The same with the infantry and the hospital. (ii) Order the
  slave miner to move. (iii) Once the recorder shows the harvester in contact with the refinery
  and unloading, give it a move click on an open cell 6 cells away. (iv) The same as (iii).
- *Runs:* 3 of each part. No random draw affects the recorded contact or behaviour.
- *Observable:* (i, i′) per sample: the actor's behaviour and contact, and whether each repeat
  click produced an order (the acknowledgement); (ii) the slaves' behaviour; (iii, iv) per
  sample: the harvester's destination and contact.
- *Expected:* (i, i′) once the actor is in contact with the building and its behaviour is *enter*,
  a repeat leaves the contact as it was: no sample after the repeat shows the actor in contact
  with nothing before it has finished entering. (ii) The slaves stop mining and return to the
  miner. (iii) The first sample that shows the move destination shows the harvester in contact
  with nothing. (iv) No expected value, and it can falsify nothing: it only records, for the open
  question of which situations tether, whether the first sample with the move destination still
  shows the contact with the refinery. Either result fits part (b) under one of the two plain
  branches, so it is evidence for the open question, not a test of the claim.
- *Falsified if:* (i, i′) after a repeat given while the actor was in contact and in *enter*, a
  sample shows its contact broken off (part (b) would do this if part (a) did not hold),
  or (ii) the slaves keep mining, or (iii) the first sample with the move destination still shows
  the contact with the refinery. In (iii) and (iv), if no sample shows the move destination (the
  click made no order, or the order failed its actor check), the step is inconclusive. A repeat
  click that gives no acknowledgement made no order (the
  building refused, S-0214) and is not counted; if no repeat after contact makes an order, the
  part is inconclusive.

**Scenario O4 part ii b: recall exclusions (claims 5 and 8).**
- *Setup:* the preamble's flat stock-data skirmish map, speed 6, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
  Use a human-owned mobile slave miner and, in a separate run, a deployed slave-miner
  building, each with slaves outside mining. Place a durable enemy target in the master's
  weapon range, unable to damage it during observation.
- *Procedure:* from fresh working states, order each master to attack the target. In a
  separate deployed-master attempt, press stop. Repeat three valid attempts per case.
  Record the master's target and current/queued behaviour and its slaves' behaviours at
  each tick boundary. Compare no-order controls, and void attempts overlapping conversion,
  a group-state transition or another cause of slave recall. Input must actually issue the
  tested order, as shown by its target change or acknowledgement, or the attempt is inconclusive.
- *Expected:* neither master's attack order recalls the slaves at application. The building's
  stop likewise has no order-time slave recall. The mobile non-attack control in O4 ii
  recalls them. Later group-update or packing effects remain S-0402 claims 21 to 28's.
- *Falsified if:* a valid tested attack order, or the deployed master's stop, itself sends
  the mining slaves back to guard at its application boundary.

**Scenario O5: stop, scatter, guard keys (claims 8, 9, 18).**
- *Setup:* one armed vehicle, one harvester working an ore field, one unarmed vehicle that is not
  a harvester, one building with a weapon, a group of 5 armed infantry of a stock type that does
  not deploy and whose walking units refuse a scatter request that is not effectively forced (S-0208,
  claim 11), standing still on open ground, each in its own cell; and 3 more such infantry walking
  to a far cell. One enemy vehicle of the computer player, placed by the test map within the armed
  vehicle's weapon range and away from the other objects, for step 1. For steps 6 to 8 only,
  three more setups on the same map, each with that enemy vehicle in range: an open-topped
  transport of a stock type carrying armed infantry (step 6); an object of a stock type that
  controls spawned aircraft, on a map with the water it needs if it is naval (step 7); and a
  slave-miner harvester with its slaves, mining (step 8).
- *Procedure:* (1) vehicle attacking the enemy vehicle: stop key; (2) harvester harvesting: stop
  key; (3) select all except the 3 walking infantry: guard key; (4) the standing group: scatter
  key; (5) the walking infantry: scatter key; (6) order the transport to attack the enemy, then press
  the stop key with it selected; (7) order the spawner to attack the enemy, wait until its
  spawned aircraft attack, then press the stop key with it selected; (8) with the slave miner
  harvesting and its slaves out, press the stop key with it selected.
- *Runs:* 3 from the same start. Steps 1 to 3 involve no random draw. In step 4 each accepting
  unit takes one draw from the scenario random stream, but its destination does not depend on it
  when the nearby-cell search finds a cell (S-0208, claim 13), which on open ground it does.
- *Observable:* behaviours, targets and destinations after each key.
- *Expected:* (1) target and destination nothing in the next sample, and the vehicle falls back to
  rest by S-0215; (2) harvester behaviour guard; (3) the armed vehicle, the standing infantry and
  the harvester get area guard on their own cells; the unarmed non-harvester vehicle and the building
  get nothing; (4) each standing infantry unit gets a destination near its own cell in the sample
  after the key, as S-0208 claim 13 predicts, and moves there; (5) the walking infantry ignore the key and keep their destination; (6) the transport and every
  passenger have no target in the next sample; (7) the spawner has no target, and its spawned
  aircraft have no target in the next sample or return to it; (8) the miner gets the behaviour
  *guard* (step 2's rule) and, in the next sample, its slaves have the behaviour that sends
  them back to guard it, not *harvest*. In step 1 the vehicle's behaviour is still *attack* in
  the next sample.
- *Falsified if:* (1) the vehicle still has a target or destination after the stop is applied,
  (2) the harvester keeps harvesting after stop; (3) the building or the unarmed non-harvester gets
  area guard; (4) a standing infantry unit gets no new destination; (5) a walking infantry unit
  changes its destination because of the key; (6) a passenger keeps its target; (7) a spawned
  aircraft keeps attacking after the stop is applied; (8) the slaves keep mining after the stop is
  applied; in step 1, the behaviour is already something other than *attack* in the sample after
  the key (this falsifies only the "behaviour unchanged" part).

**Scenario O6: deploy, sell, repair, detonate (claims 10, 11, 12, 13, table T2).**
- *Setup:* two deployer vehicles of one stock type that deploys in place (one deployed and one
  not); an own building that can be sold; a vehicle on open ground; a damaged building; an own
  vehicle on which the player's own bomb-planting infantry has planted a bomb; a vehicle of a type
  that can deploy, standing on a cell of the player's vehicle factory.
- *Procedure:* (1) select both deployers: deploy key; (2) sell mode, click on an own building
  and on the vehicle on open ground; (3) repair click twice on the building; (4) select only the
  bombed vehicle and click it (if the general rules forbid setting off that kind of bomb, a named
  test rules file allows it); (5) select the vehicle on the vehicle factory's cell: deploy key.
- *Runs:* 3, each from a fresh start of the test map. No random draw affects the observables.
- *Observable:* which deployer changes state; what is sold; the building's repair state
  after each click; the bomb; whether (5) deploys.
- *Expected:* (1) only the undeployed one deploys; (2) only the building is sold; (3) repair
  on, then off; (4) the bomb goes off at once; (5) nothing happens.
- *Falsified if:* in (1) the deployed deployer undeploys or the undeployed one does not deploy,
  in (2) the building is not sold or the vehicle is sold; in (3) the repair state does not go on
  at the first click and off at the second; the bomb does not go off in (4); or (5) deploys.
- *Note:* the click on the vehicle in step 2 tests S-0214's input rule (no sell order is made for
  a vehicle), not claim 12's vehicle rule, which stays unmeasured.

**Scenario O6 part B: mid-transition members of a deploy press (the deploy rule of table T2,
Q-1753).**
- *Setup:* the usual isolated test map, speed 6, no enemy. A named test rules and art file defines
  three in-place deployer types, two of each placed side by side on open ground: infantry type *P*
  with a negative undeploy delay; infantry type *Q* with an undeploy delay of 300 ticks; vehicle
  type *V* that deploys in place. The deploy and
  undeploy sequences of *P* and *Q* last at least 40 ticks each, and *V*'s deploying and undeploying
  presentations at least 60 ticks each, so that the selection and the press fit inside the
  transitions (below). No deploy weapon, so no shot sets a
  rearm countdown.
- *Procedure:* for each type, five cases, each from a fresh start. A member is put into a
  transition by a deploy press with it selected alone. For *P* and *V* in (a), deploy the first
  member with a press with it selected alone and wait until it is deployed; then select the second
  alone with a click and press to start its deploy; then add the first with a *toggle select*
  click (S-0214 claim 3) and make the measured press within 20 ticks of the second member's press.
  For *P* and *V* in (b), deploy the first member alone as in (a) and wait until it is deployed,
  then, with it still selected alone, press to start its undeploy; then add the undeployed second member
  with a *toggle select* click and make the measured press within 20 ticks of that undeploy press.
  For *Q*, whose deployed members have no
  deploy readiness, the undeploying member is instead one that undeploys by itself after its delay
  (S-0215 claims 30 and 31: about 300 ticks after the processing that started its deploy), and the
  other presses are timed around that. Selecting both members and pressing takes several input
  events, one per cycle (claim 1), so the measured press is made no later than 20 ticks after the
  last transition began. A run in which a transition ends before the measured press's order is
  carried out is void. The
  measured press is a single deploy press with both members selected: (a) one deployed and one
  deploying; (b) one undeploying and one undeployed; (c) one deploying and one undeploying; (d)
  both deploying; (e) both undeploying. For *P* in (c), start the first member's undeploy with it
  selected alone; then select the second alone with a click and press to start its deploy, as
  soon as those input events allow (one per cycle, so a few ticks later); then add the first with
  a *toggle select* click (S-0214 claim 3) and make the measured press. For *P* and *V* in (d),
  select both undeployed members, press once to start both deploying, and make the measured
  press within 20 ticks of that. For *P* and *V* in (e), deploy both with one press with both selected, wait until both are deployed,
  press once to start both undeploying, and make the measured press within 20 ticks of that. For
  *Q* the measured presses are timed as follows, with *t*₀ the tick the first member's deploy
  begins: (a) the second member is deployed alone at *t*₀ + 100, while the first is still
  deployed; (b) the measured press comes within 20 ticks of the first member beginning to undeploy
  by itself, near *t*₀ + 300; (c) with the second member already selected alone, its deploy
  press comes 2 ticks after the first begins to undeploy by itself, the first is then added with
  a *toggle select* click, and the measured press follows within 20 ticks; (d) both are deployed
  by one press at *t*₀, and the measured press follows within 20 ticks of it; (e) the pair of (d)
  undeploys by itself together near *t*₀ + 300, and the measured press comes within 20 ticks of
  that. A vehicle playing its
  deploying or undeploying presentation supplies no ordinary candidate for a click or a band
  (S-0214 claim 24), so for *V* the member in a transition stays selected from its own press and
  the other is added with a *toggle select* click (S-0214 claim 3); in (d) both start with one
  press with both selected, and in (e) both stay selected from the press that started their
  undeploy. For *V*, case (c) is reached from case (a): (a)'s measured press leaves the deployed
  member in its undeploy sequence and the other in its deploy sequence, both still selected, and a second press within
  20 ticks of the first is (c)'s measured press, with at least 20 ticks of each presentation still
  to run.
- *Runs:* 3 per case and type. No random draw affects the observables.
- *Observable:* from the measured press, every tick for 150 ticks: each member's sequence (infantry) or
  deployed, deploying and undeploying state (vehicle), read by the read-only recorder; whether the
  press queued a deploy order for each member (the recorder's order log); and whether the
  acknowledgement plays.
- *Expected:* the rows of the deploy rule's table for two members. Orders: for *P* and *V*, both
  members get one in (a), (b), (d) and (e), and for *P* and *V* in (c) only the undeploying member
  does; for *Q*, no
  member gets one in (a), (c), (d) and (e), so no acknowledgement plays, and in (b) only the undeployed
  member does. States: *P* finishes any transition in progress and then goes the other way when it
  got an order; a *Q* member that got an order deploys; *V* in every case finishes the transition
  in progress and then stays as it ended, while a deployed or undeployed *V* member that got an
  order changes once.
- *Falsified if:* any transition under way is cut short or reversed before it ends; a
  mid-transition *P* or *V* member in a selection that is not mixed gets no order; for *P* or *V*,
  in (c) the deploying member gets an order, or the undeploying one does not; a *Q* member that is deploying,
  deployed or undeploying gets an order; *P*'s member that got an order does not go the other way
  after finishing; or a *V* member that was mid-transition goes the other way after finishing.

**Scenario O7: cheer rate (claim 19).**
- *Setup:* fresh program start; a skirmish with infantry of the player's. A named test rules file
  sets the general rules' maximum-cheer-rate field to *R* = 450 ticks, so that the expected values
  do not depend on stock data.
- *Procedure:* press the cheer key repeatedly from the first tick; note the first tick of a cheer.
  Then press again at known ticks. After tick 1000, end the game right after an accepted cheer,
  *E* is the tick of that cheer, the last accepted cheer of the first game. Start a new skirmish
  and press cheer repeatedly.
- *Runs:* 2, each a fresh program start followed by the two games. The cheer rule involves no
  random draw.
- *Observable:* ticks of accepted cheers in both games.
- *Expected:* first game: no cheer at or before tick *R* − 100, then cheers at least *R* + 1 ticks
  apart; second game: no cheer at or before tick *E* + *R*.
- *Falsified if:* the second game accepts a cheer at or before tick *E* + *R* (memory reset), the
  first game accepts one at or before tick *R* − 100, or two accepted cheers of one game are *R*
  or fewer ticks apart.

**Scenario O8: route mode (claims 8, 16 to 16h and 17).**
- *Setup:* game A: one vehicle of the player's on open ground and 128 infantry of the player's,
  all selectable. Game B, a fresh game: 130 infantry of the player's, all selectable, and no other
  unit of the player's, with a cell P₁ 30 cells away from them and a cell P₂ 10 cells beyond
  it. The recorder logs each infantry unit's route, and a refused click made after any of the
  128 has reached P₁ (and so issued P₂) is void. Game C: as game A, with the vehicle only.
- *Procedure:* game A: (1) select the vehicle, hold the route-mode key, click 3 cells, release.
  (2) While it follows the route, select it and give a plain move click. (4) After the vehicle has
  arrived, select the 128 infantry, hold the route-mode key and click one cell. Game B: (3) select
  128 infantry, hold the route-mode key, click P₁ and P₂ and release, so that each keeps P₂ in its
  route while it walks to P₁. Select only the 2 other infantry, hold the key and click a cell.
  Wait several completed ticks, then repeat the refused click twice, several seconds apart,
  without changing the selection. Then add one of the 128 to the selection and click again.
  Game C: (5) as step 1, then, while the vehicle follows the route and
  before it reaches the first cell, select it and press the stop key.
- *Runs:* 3 of each game. Route mode involves no random draw.
- *Observable:* whether the vehicle moves before the key is released; the order in which it
  visits the cells; whether it continues the route after (2) and after (5); whether the clicks of (3) and (4)
  are refused (message, its text and sound), including video and audio of each repeated
  refusal in game B.
- *Expected:* each game-B refusal separated by completed ticks gives a message and refusal
  sound, with no new route point. No once-per-game silence is expected. No movement before release, then the cells in click order; after (2) it goes to the
  new cell and never returns to the route; after (5) it comes to rest and does not go on to any
  cell of the route; in game B the clicks on P₁ and P₂ are accepted and the
  click by the 2 others is refused with the route-holder-limit text, because a route-holder
  count of 128 plus 2 new objects exceeds 128. The last click of (3), whose selection also
  mixes route
  states, gives only the
  incompatible-selection text and one sound (claim 16d). Step 4's click is
  accepted, because the removal of the vehicle's route lowers the count at once
  (claim 16f); a refusal (1 + 128 > 128) falsifies claim 16f.
- *Falsified if:* the vehicle moves before release, it resumes the route after (2) or after (5), or in game B
  the refusal comes at another count or with another text, the last click of (3) gives both
  texts, or a repeated refusal after completed ticks loses its
  message or sound solely because an earlier refusal was shown. The repeated-click test
  distinguishes the claim from a once-per-game limit; it does not isolate same-tick duplicate
  notices, for which normal input supplies at most one event per cycle (claim 1).
- *Further parts (Q-1986), each in a fresh game with the vehicles of the player's on open ground,
  an enemy vehicle E standing in the open and game speed 6.* Record video with the route-mode key
  held at the end of each part, each message shown, and each vehicle's cell and current action
  every tick.
  - *(a) Object point:* route vehicle V through cells P₁, E's cell (clicked over E) and P₃, then
    release. Expected: V moves to P₁, then attacks E until E is destroyed, then moves to P₃.
    Falsified if V moves to E's cell without attacking, or skips P₃ after E dies.
  - *(b) Loop:* route V through P₁, P₂, P₃, click P₁ again, release. Expected: the loop message,
    then V visits P₁, P₂, P₃, P₁, P₂, … until given a move order outside route mode. A second
    route through P₁, P₂, P₃ without the extra click: V stays at P₃. In a third run, after the
    click that closes the loop and before the release, click a new cell P₄. Expected: the click
    on P₄ is refused with the in-a-loop text (claim 16b), and V never visits P₄. In (b), E stands
    out of V's weapon range of every point and leg, so the loop's attack moves (claim 16c) find
    no target. Falsified if the
    route repeats without the extra click, stops after one round with it, or P₄ is added or
    refused with another text.
  - *(c) Two vehicles:* select V and W together, route them through P₁, P₂, release. Expected:
    both start at the commit, both go to each point in turn, and one marker per point is drawn.
    Record the tick on which each leaves P₁, for the synchronisation question (claim 16c).
  - *(d) Ineligible objects:* select V with an aircraft of the player's, then V with a building
    of the player's, and click in route mode. Expected: each click refused, with the aircraft
    and the building message respectively, and no point added to V's route. Then select V with a
    route holder H of another route and click: refused with the incompatible-selection text.
  - *(e) Limits:* add 17 points to V's route. Expected: the 16-point message after the 16th and a
    refusal with that message at the 17th. Make the 3rd point an *enter* into a transport of the
    player's, then click a 4th cell: refused with the after-terminating text. In a second route,
    make a point an attack on a cell (force-fire on ground), then click again: refused with the
    after-continual text. Falsified if a 17th point or a point after either kind is added, or if
    the two refusals show the same text.
  - *(f) Key commands:* hold the route-mode key with V selected and press guard, stop, scatter and
    deploy in turn. Expected: four messages, no order and no route point; V's route after release
    is the one built by clicks alone. Falsified if a guard point is added.
  - *(g) Delete and extend:* route V through P₁ to P₄ and release. Deselect V, hold the key,
    click P₂, press delete and release. Expected: the point-selected message, then V visits
    P₁, P₃, P₄. In a second run, while V is on its way to P₃, select V, hold the key, click P₅
    and release. Expected: V does not turn away from P₃ at the release and visits P₄, then P₅.
    In a third run, add P₅ while V moves to P₃ and keep holding the key until V reaches P₄.
    Expected: V stops at P₄ and moves on to P₅ only after the release.
    Falsified if V abandons P₃ at the release, or reaches P₅ before the release.
  - *(h) Visibility:* in a two-human network game, players A and B allied, with a third player C
    not allied, A lays out a route and keeps the key held. Expected: B sees A's route only while
    B holds its own route-mode key, C never sees it. A sees nothing of the route once its key is
    released.
    In a second network game of three humans, A and C not allied, A gives a vehicle a plain move
    order to a far cell and, while it moves, routes it through far cells P and Q and releases.
    The vehicle is busy, so the commit does not interrupt it and its route keeps P until it
    comes to rest (claim 16c). C's press below is made while A's vehicle is still on its plain
    move; if it has come to rest first, repeat the game with a farther cell. C gives one of its
    vehicles a route through two other far cells, selects that vehicle and, holding the
    key, presses with scripted input at the screen
    position of P's place, found from the camera position (claim 16b). Expected: refused with
    the invalid-point text, though C sees no marker there. With a vehicle of C's that has not
    held a route point selected instead, the same press adds a point for it. Falsified if the
    first press is accepted, or the second refused. In a third such game, with a unit of C's
    standing in view of P's cell so that the cell is not shrouded for C, C selects a vehicle
    factory of its own instead and makes the same press, then holds the key and presses on bare
    ground away from every marker. Expected: no building-in-selection text at the first press,
    and the factory's rally point set at P's place; the building-in-selection text at the
    second. A building-in-selection text at the first press falsifies the hidden point's click
    area that skips the kind test.
  - *(i) Loop cancelled:* route V through P₁, P₂, P₃, click P₁ again and release. After V's first
    round, deselect V, hold the key, click P₂, press delete and release. Expected: V visits each
    remaining point at most once more and then stays. In a second run, loop V through P₁ to P₄
    and delete P₂ in the same way. Expected: V keeps going round P₁, P₃ and P₄. Falsified if the
    3-point loop keeps going round after the deletion, or the 4-point loop stops.
  - *(j) Used up:* game A again, with the third cell at least 20 cells beyond the second. Do
    step 1. While the vehicle is on its way to the second cell, do step 4's click: expected
    refused, since the route still holds the third point. Let the infantry stay as they are (the
    refused click gave them no point). After the vehicle has left the second cell and before it
    reaches the third, click again with the same 128: expected accepted, because issuing the last
    point used the route up. In a second run, wait until the vehicle has come to rest at the
    third cell, then do step 4: expected accepted. A refusal in either accepted case, or an
    acceptance before the second cell, falsifies claim 16f's used-up case and its moment.
  - *(k) Destroyed:* game A again. Do step 1, then select the 128 infantry and have them fire on
    the vehicle (force-fire) until it is destroyed before it reaches the second cell. Hold the
    key: no marker remains at the second and third cells. Then do step 4. Expected: accepted.
    A marker left at those cells, or a refusal, falsifies claim 16f's destruction case.
  - *(l) Shared point, two actions:* select V and an engineer of the player's, hold the key, click
    a capturable enemy building and then a cell P, and keep holding. Expected: the
    after-terminating text twice, as information at the first click (the engineer's capture
    point is accepted) and as the engineer's refusal at the second, and a route line from the
    building to P for V only. Falsified if both routes or neither reach P, or the refusal text is
    missing.
  - *(m) Last point deleted:* game B again. Do the first part of step (3). While the 128 walk to
    P₁, deselect them, hold the key, click P₂, press delete and release, so that every route is
    empty. Select the 2 other infantry, hold the key and click a cell. Expected under the quirk:
    refused with the route-holder-limit text. Accepted means the 128 stopped counting when their
    last point was deleted, which falsifies claim 16f's last-point case and settles the quirk.
    Then select the 128, give them a plain move order, select the 2 again and click in route
    mode. Expected: still refused, because the order does not lower the count. Accepted falsifies
    claim 16f's statement that a later order leaves the count as it was. Then have the 2 destroy
    2 of the 128 (force-fire) and click with the 2 again. Expected: refused (128 + 2). Accepted
    (126 + 2) falsifies the statement that destruction leaves the count as it was. Last, select
    one of the remaining 126 alone and click in route mode. Expected: refused (128 + 1), because
    the object counts again as new. Accepted falsifies that part of claim 16f.
  - *(n) Join:* route V through P₁, P₂ and P₃ and release, with P₁ far enough away that V is
    still on its way to it. Select a vehicle W that holds no route, hold the key, click P₂ and
    release. Expected: W goes to P₂ and then to P₃, and with the key held W's route line runs
    through P₂ and P₃. W stopping at P₂ falsifies the copied route of claim 16b. W must be a
    vehicle that has never held a route point in the game (claim 16b, route record).
  - *(o) Joins and the count:* game B again, with a cell P₃ 10 cells beyond P₂ and a far cell
    P₄ away from all three. Select one infantry unit X alone, hold the key, click P₁, P₂ and P₃
    and release. Select 128 others, hold the key, press P₂'s marker and release. Expected:
    accepted, though 1 + 128 is more than 128, because a press that takes a point is not tested
    for the holder limit (claim 16d). The 128 start for P₂ and keep P₃ in their routes, so the
    house has 129 route holders. A refusal with the route-holder-limit text falsifies that
    exemption. Then select the last infantry unit Y alone, hold the key, click P₄ and release.
    Expected under claim 16f: accepted (1 + 1), and Y goes to P₄. A refusal with the
    route-holder-limit text (129 + 1) means a join raises the count, which falsifies claim 16f's
    join sentence. Record the tick of Y's click and the tick each of the 129 reaches P₂. A click
    by Y made after any of the 129 has reached P₂ is void: at P₂ that holder issues its last
    point, P₃, so its route is used up and the count is lowered (claim 16f).
  - *(p) A new route on top:* game A again. Route the vehicle V through P₁ and P₂ and release,
    with P₁ far away. While V walks to P₁, deselect it, hold the key, click P₂, press delete and
    release, so that V's route is empty. After V has come to rest at P₁, select it, hold the key,
    click a far cell P₃ and then P₄, and release. While V walks to P₃, select 127 of the
    infantry and click in route mode. Expected: refused (2 + 127). Accepted (1 + 127) falsifies
    claim 16f's statement that a new route counts again on top. The click must come before V
    reaches P₃ (and so issues P₄).
  - *(q) Clicks on existing points and the order's place (claim 16b):* a fresh game, with
    vehicles V, W and T of the player's that have not held a route point, and water next to the
    open ground. (1) Select V, hold the key, click P₁, P₂ and P₃ and keep holding. Click P₃'s
    marker, then P₂'s. Expected: each refused with the invalid-point text and no loop message.
    Then click P₁'s marker: the loop message. Release: V goes round P₁, P₂, P₃. A loop closed on
    P₃ or P₂, or P₁ refused, falsifies the 3-point minimum. (2) With V's loop running, select V,
    hold the key and click P₂'s marker. Expected: refused with the invalid-point text (a route
    with a loop). (3) Select W, hold the key, click far cells Q₁ and Q₂, release, and while W is
    on its way to Q₁, select W, hold the key and click P₂'s marker. Expected: refused with the
    invalid-point text, and W's route line still ends at Q₂. W's route line reaching P₂ falsifies
    the point-not-in-the-route condition. (4) After W has come to rest at Q₂ (its route used
    up), select W, hold the key and click P₂'s marker. Expected under the quirk: refused with the
    invalid-point text. W's route line reaching P₂ means that an empty route counts as none,
    which falsifies the route record kept and settles the quirk. Record in (1) to (4) whether
    the hovered marker is highlighted before each click. (5) Select T, hold the key and click a
    water cell one cell from the shore, a click to which S-0214 claim 11 gives a destination
    found by the nearby-cell search, and keep holding. Expected: the marker on the water cell,
    with T's route line ending there. A marker on a land cell falsifies the order's place. Where
    T goes after the release is not an observable of this part. (6) In a fresh game with the same
    set-up, where T is of a type eligible for attack moves (claim 6), make (5)'s click with
    attack-move entry (S-0214 claim 5) active as well. Expected: as in (5), the marker on the
    water cell. A marker on a land cell falsifies the attack-move click's place.
  - *(r) Which route a join copies (claim 16b):* a fresh game, with vehicles A to H of the player's
    that have not held a route point. B is created before A (a lower object number) and stands
    nearer to C than A does. All points are cells. (1) Give A a plain move order to a far cell and,
    while it moves, select A, hold the key, click P₁, P₂ and P₃ and release (A is not interrupted,
    claim 16c). Give B a plain move order to a far cell too and, while it moves, select B, hold the
    key, click P₂'s marker and then a cell P₄, and release: B's route is P₂, P₃, P₄, and B is not
    interrupted. Select C, hold the key, click P₂'s marker and release. Expected: with the key held,
    C's route line runs through P₂ and ends at P₃, and C goes to P₂, then P₃, and stays. C's line
    reaching P₄, or C going to P₄, means B's route was copied, which falsifies the longest holder.
    The clicks must come before A or B has issued P₂. (2) Select D, at rest, hold the key, click Q₁,
    Q₂, Q₃ and Q₄, then Q₂'s marker (a loop from Q₂), and release: D starts at once and issues Q₁.
    Hold the key with nothing selected. Expected: no marker at Q₁, which was issued before the
    loop's first point and which no other route holds, and markers at Q₂, Q₃ and Q₄. A marker at Q₁
    falsifies claim 16f's deletion of such a point. Release. Select E, hold the key, click Q₃'s
    marker and release. Expected: E goes to Q₃, Q₄, Q₂, Q₃, Q₄, Q₂ and on, never to Q₁. E going only
    between Q₃ and Q₄, stopping, or visiting Q₁ falsifies the copied loop. (3) Give F a plain move
    order to a far cell and, while it moves, route it through R₁ to R₄ with a loop from R₂. Select
    G, hold the key, click R₁'s marker and release before F has issued R₁. Expected: G goes to R₁,
    then round R₂, R₃ and R₄, and never back to R₁. (4) After D has gone round its loop at least
    once, select H, hold the key, click Q₄'s marker and release. Expected: H goes to Q₄, then round
    Q₂, Q₃ and Q₄, whatever point D is heading for. H stopping before it has visited all three, or
    visiting Q₁, falsifies the copy of a loop whose holder is part of the way round. (5) A tie, in
    a fresh game, with vehicles J, K and L of the player's, of one type whose primary weapon has
    positive damage, that have not held a route point. Give J and K plain move orders to far cells
    and, while they move, select K and
    then add J, so that J is first in selection order (Terms). Hold the key, click S₁ and release:
    the click's orders give J and K the point S₁ in the same tick. Select J alone, hold the key,
    click S₂ and release. Select K alone, hold the key, click S₃ and release. Select L, hold the
    key, click S₁'s marker and release, before J or K has issued S₁. Expected: L's route line runs
    through S₁ to S₂, and L goes to S₁, then S₂ (J's route). Repeat in a fresh game,
    with fresh vehicles J, K and L set up in the same way but with J selected first and K
    added, so that K is first: L goes to S₁, then S₃. L taking the other holder's continuation in
    either run falsifies the tie rule.
  - *(s) A point add at a committed point's place (claims 16a and 16c):* a fresh game, with an
    enemy building Y far from armed vehicles V and W of the player's, neither of which has held a
    route point. (1) Give V a plain move order to a far cell and, while it moves, select V, hold
    the key, click a cell P₁ and then Y (an attack point), and release: V keeps moving and holds
    committed points. Select W at rest, hold the key, click far cells Q₁ and then Q₂ (away from
    Y and from each other), and release: W heads for Q₁ and still holds Q₂ (claim 16c). While W
    is on its way to Q₁, select W, hold the key, click Y on a cell of its foundation away from
    the point's marker, and release. W now has a route record and V's point at Y is not in its
    route, so a click that meets the marker is refused with the invalid-point text (claim 16b)
    and adds nothing: such a click is void, repeat it farther from the marker. Then, with
    nothing selected,
    hold the key, click the marker at Y, press delete and release. Expected: the next time the
    key is held, exactly one of V's and W's route lines reaches Y. Neither reaching it means
    that the two point adds shared one point, which falsifies claim 16a's later point add.
    (2) In a second game, set V up as in (1). Select W at rest, hold the key, click a far cell
    Q₁ and release: W heads for Q₁. While W is on its way, select W, hold the key, click Y
    away from the marker as in (1), and keep holding the key until W has come to rest at Q₁ and
    for several seconds after. Expected: W rests at Q₁ while the key is held and attacks Y only
    after the release. W heading for Y while the key is still held falsifies the new point's
    being uncommitted.
  - *(t) Waiting at the commit (claim 16c):* a vehicle V of the player's, at rest in *guard*,
    with no route and no order for several seconds. (1) Select V, hold the key, click a far cell
    P₁ and then a far cell P₂, and release. Record V's current behaviour and whether it is ready
    to change (S-0215 claim 11) on every tick from 3 ticks before the commit to 3 ticks after
    it, and again from its arrival at P₁ until it heads for P₂. A run in which V is not ready
    to change at the commit is void. Expected: *guard* until the commit's tick, *wait* in the
    commit's tick after its order boundary, and *move* from V's next per-tick start after the
    issued order applies (S-0215 claims 11 and 12).
    *Guard* followed directly by *move*, with no tick of *wait*, falsifies the queue of *wait*.
    At P₁, expected: at least one tick of *wait* between the arrival and *move* towards P₂ (the
    issue at the fall-back to rest; whether another behaviour comes before the *wait* is not
    part of the test). No *wait* there falsifies the hand-off for S-0215 below. Repeat with an
    infantry unit. (2) A second run, as (1) but with an enemy vehicle E in V's range before the
    key is held, which V takes as its target in *guard* by its per-tick search (an opportunity
    target, S-0309 claims 23 and 24) and does not destroy within the run. A run in which E is
    destroyed, or V has no target at the commit, is void. Record V's target and its shots on
    every tick from 3 ticks before the commit until the issued order applies. Expected: E as
    V's target up to and including the commit's tick (a shot at E in that tick is allowed), no
    target from the tick after the commit's tick on, and no shot by V from that tick until the
    issued order's behaviour is current. V keeping E as its target in the tick after the
    commit's tick, or firing at it from then until that behaviour is current, falsifies the drop
    under *wait*; E dropped already in the commit's tick falsifies the commit's timing at the
    order boundary. Repeat with an infantry unit.
  - *(u) Points that no route holds (claim 16f):* a fresh game, with vehicles V and W of the
    player's, neither of which has held a route point, and cells P₁, P₂ and P₃ far from both
    and from each other. Give V a plain move order to a far cell and, while it moves, select V,
    hold the key, click P₁, P₂ and P₃ and release. Select W, hold the key, click P₂'s marker
    and release: W joins (its route is P₂ and P₃) and starts for P₂ at once, issuing it. While
    W is on its way to P₂: (1) with nothing selected, hold the key. Expected: markers at P₁, P₂
    and P₃. Release. (2) Select V and give it a plain move order, which ends its route (claim
    17). With nothing selected, hold the key. Expected: no marker at P₁ or P₂, and a marker at
    P₃, which W still holds. Click where P₂'s marker was: no point is selected and no
    point-selected message is shown. Release. (3) After W has reached P₂ and while it is on its
    way to P₃, hold the key with nothing selected. Expected: no marker at P₃, since W issued it
    at P₂. A marker left at P₁ or P₂ in (2), or at P₃ in (3), falsifies the deletion of points
    that no route holds.
  - *(v) Detonate in route mode (claims 16b and 16d):* a fresh game, with an own vehicle V on
    which the player's own bomb-planting infantry has planted a bomb, as in Scenario O6, and a
    far cell P₁. A named test rules file lets the player set off that kind of bomb and gives the
    bomb a timer longer than the run (a test value), so the bomb cannot go off by its own timer.
    (1) Select only V, hold the key, click P₁, then press the mouse button on V itself, keep the
    button down for 2 seconds and release it, and keep holding the key for 5 seconds more.
    Record the recorder's order log with each order's issue tick, the ticks of the button's
    press and release on V, and the tick at which the bomb goes off. Expected: a detonate order
    for V is logged as issued in the tick of the button's release, not of its press, and is
    applied while the key is still held. The bomb goes off in the tick the order is applied,
    with no route-mode message, and no marker is drawn at V's place. Falsified if no detonate
    order is logged before the key's release, if one is logged as issued in the press's tick or
    any tick before the button's release, if the bomb goes off in another tick than the one the
    order is applied in, or if a marker is drawn at V's place.
    (2) A fresh game as game B, with the same test rules file, V and the planter added, and the
    bomb planted on V before route mode is used. Do the first part of game B's step 3 (the 128
    infantry routed through P₁ and P₂), so that 128 infantry hold routes made by point adds.
    Before any of them reaches P₁, select only V, which holds no route, hold the key, click V
    itself and release the key. Expected: refused with the route-holder-limit text (128 + 1),
    no detonate order is logged and the bomb does not go off. A detonate order logged, or the
    bomb going off, falsifies the holder-limit test of a detonate click at its release. A run in
    which any of the 128 has reached P₁ (and so issued P₂) before the click on V is void.
  - *(w) Click areas and the patrol action (claim 16b):* a fresh game on open flat ground, with
    vehicles V, W, X, Y, U, Z, T, S, E, F, G and H of the player's that have not held a route
    point, all
    able to fire, and cells P₁, P₂, P₃ and P₄ far from them and from each other. The scripted
    input places the cursor at exact screen pixels, measured from the screen position of P₂'s
    centre, which the camera position gives. Give V a plain move order to a far cell and, while
    it moves, select V, hold the key, click P₁, P₂ and P₃ and release. In each step below, select
    only the named vehicle without the key, then hold the key, make the press and release the
    key. (1) W, at 7 pixels right of and 7 below P₂'s position, X, at 8 left of and 8 above it,
    E, at 8 left of it and level with it, and F, at 8 straight above it. Expected: each joins,
    so it goes to P₂ and then P₃. (2) Y, at 8 pixels right of P₂'s position, U, at 8
    below it, G, at 9 left of it and level with it, and H, at 9 straight above it, all on P₂'s
    cell. Expected: each adds a point, so it goes to P₂'s cell and stays there, never going on
    to P₃. (3) Z, with force-move and force-fire held, at P₂'s position. Expected: a join, as in
    (1). (4) T, with force-move and force-fire
    held, at 20 pixels left of P₂'s position and level with it, a place on P₂'s cell
    outside the click area and beyond (2)'s presses, then, with force-move and force-fire let go,
    a click on P₄. Expected: T's
    route line ends at P₄, through a point at P₂'s cell, and after
    the release T is in *patrol* on its way to that point. (5) V's and S's types are test copies
    whose primary weapons do positive damage (test values, S-0214's selection order, Terms). In
    a second game, set up V's route as above, select V and then S with a toggle-select click
    (S-0214 claim 3) without the key, hold the key, press P₁'s marker and release. Expected: the
    loop-made text, V goes round P₁, P₂ and P₃, and S, the newer and so first, goes to P₁, P₂
    and P₃ and comes to rest there. In a third game, select S and then V, with the same press.
    Expected: V goes round the loop as before, and S goes to P₁ and round the same loop. (6) In
    a fourth game, give V and W plain move orders to far cells and,
    while they move, route V through P₁ and P₂. Select W alone, hold the key, click a corner of
    P₁'s cell outside the click area, and release: W's route is a second point at P₁'s place.
    With nothing selected, hold the key, press P₁'s marker, press delete and release. Expected:
    V's route line no longer passes P₁, W's still ends at P₁'s cell. In a fifth game, make the
    two points in the other order: W first, by the same corner click, and then V, by a click on
    another corner of P₁'s cell outside the click area and a click on P₂. The same press and
    delete. Expected: W's point is deleted, so W shows no route line, and V's line still runs
    from P₁'s cell to P₂. The point made later deleted in either game falsifies the earliest-made
    point's selection. (7) In a sixth game, give V, W and S plain move orders to far cells and,
    while they move, route V through P₁, P₂ and P₃. Select W alone, hold the key, click a corner
    of P₁'s cell outside the click area, then P₄ and then a far cell P₅, and release: W's route
    starts with a second point at P₁'s place. While all three still move: (i) select W alone,
    hold the key, press P₁'s marker and release. Expected: the invalid-point text, and no
    loop-made text. (ii) Select V alone, hold the key, click a corner of P₁'s cell outside the
    click area (a fourth point of V's route, at P₁'s place, whose marker is drawn last), then
    press P₁'s marker and release. Expected: the loop-made text, and V later goes round P₁, P₂,
    P₃ and the fourth point. A refusal falsifies, since it means the press took a later-made
    point (W's, or V's fourth).
    (iii) Select S alone, hold the key, press P₁'s marker and release. Expected: S later goes
    to P₁ and then to P₂, as V's route does. S going on towards P₄ falsifies, and so does the
    loop-made text in (i). (8) In a seventh game, give V, W and X plain move orders to far
    cells and, while they move, route V through P₁, P₂ and P₃. While all three still move,
    select W alone, hold the key, press P₂'s marker and release (W joins: its route is P₂ and
    P₃), then select X alone, hold the key, press P₁'s marker and release (X joins: its route is
    P₁, P₂ and P₃). Select V alone without the key, so that W and X stay unselected (claim 16h),
    hold the key, press P₁'s marker and release. Expected: the loop-made text, V and X later go
    round P₁, P₂ and P₃, and W goes to P₂ and P₃ and then round P₁, P₂ and P₃. W or X coming to
    rest at P₃ falsifies the orders for objects that are not selected, and W coming to rest
    after a second visit to P₃ falsifies the list's order. Falsified if any expectation fails:
    a join at 8 right or 8 below, at 9 left or at 9 above, or a point add at 7 right and 7 below,
    at 8 left and above, at 8 left or at 8 above, falsifies the click area.
  - *(x) When the tests are made (claim 16d):* a fresh game, with an own building B, an own
    aircraft A at rest on the ground, vehicles V and W and an infantry unit I of the player's
    that have not held a route point, and a far cell P₁. Give V a plain move order to a far cell
    and, while it moves, select V, hold the key, click P₁ and release. (1) Select B, hold the
    key and click W, then bare ground, then drag a band box over W and I. Expected: the
    building-in-selection text for each, B still the only selection, and no box drawn. With B
    still selected and the key held, click the sidebar's sell button. Expected: sell mode is
    armed, with no building-in-selection text; disarm it. (2) With
    nothing selected, hold the key and click W. Expected: W selected, with no message and no new
    marker. (3)
    B's and A's types are test copies whose primary weapons do positive damage (test values,
    S-0214's selection order, Terms). Select B and then A with a toggle-select click (S-0214
    claim 3), hold the key and click bare ground. Expected: the aircraft-in-selection text only,
    since A, the newer, comes first. Repeat with A selected first: the building-in-selection
    text only. Repeat both with a second test copy of B's type that has no primary weapon:
    the aircraft-in-selection text only, in both orders. (4) Select W, hold the key and click V
    to select it.
    Expected: V becomes the selection, with no incompatible-selection text and no new marker.
    (5) Select A alone,
    hold the key, press P₁'s marker and release. Expected: no aircraft-in-selection text, and A
    takes off after the release and flies to P₁. (6) In a second game with the same set-up,
    route V through P₁ and a far cell P₂ instead. Select A alone, hold the key, press P₁'s
    marker and release. Expected: A flies to P₁ and on to P₂ without landing at P₁ or turning
    back. A's type is one with a take-off sound (a test copy with a test value if needed).
    Record every tick from A's arrival over P₁ until it is on its way to P₂: its height, its
    current and queued behaviour, its destination, and any take-off sound. Expected: A's height
    does not fall below its cruise height in those ticks, no take-off sound is played at P₁, A's
    destination goes from P₁ to P₂ with no tick in which it has none, and no tick shows *guard*
    current or queued. Record what A does after P₂. In a further game with the same set-up (the
    pad-bound game), A's type is a pad-bound test copy (test value) and A stands at rest on an
    own pad building of a type that it docks at, far from P₁, and (6)'s steps are made.
    Expected: as above, with A neither turning back towards a pad nor crashing at P₁. Record
    what A does after P₂. (7) In a network game of two humans not allied (as in
    part h), with the same set-up for the first player and a vehicle E of the second player's,
    whose type is a test copy that has no weapon and a strength that no shot of B's can use up
    (test values), kept out of B's range. B's type is a test copy without the turret flag (test
    value). With no target in B's range, select B alone, hold the
    key, press P₁'s marker and release. Expected: no building-in-selection text. Then the
    second player drives E into B's range and stops it there. Expected: B makes no shot at E.
    Then the first player, without the key, gives B an *attack* order on E. Expected: B fires
    at E. In a second such game, the second player first drives E into B's range, and once B
    fires at E the first player makes the same press on P₁'s marker with B selected. Expected:
    B keeps firing at E after the release, since a holder under *attack* on an object is not
    started by the commit (claim 16c). In a third such game, B's type is instead a copy of the
    same test type with the turret flag (test value), and the first game's steps are made.
    Expected: B makes no shot at E in the 1,000 ticks after the *attack* order, with *move*
    current and *attack* queued throughout. (8) With B selected
    and the key held, right-click bare ground with press and release at one screen position,
    then select B again, hold the key and right-drag across the battlefield. Expected: no
    building-in-selection text and no refusal sound for either, B no longer selected after the
    right click, and the view scrolled by the drag. Falsified if any expectation fails. An
    aircraft-in-selection text in (5) falsifies the press inside a click area untested for kind,
    A coming to rest at P₁ in (6) falsifies the aircraft's issue at a flight's end. In (6), a
    fall in height or a take-off sound at P₁ falsifies the issue that begins no landing, and in
    its pad-bound game A turning back towards a pad or crashing at P₁ falsifies the
    issue coming before the pad-bound test. A shot
    of B's at E before the *attack* order in (7)'s first game falsifies the building that does
    nothing under *move*. In (7)'s third game, a shot of B's at E, or *attack* current, falsifies
    the turreted building whose ready mark nothing sets under *move*, and in the first game no
    shot of B's at E after the *attack* order falsifies the start that the body range allows.
  - *(y) Selection and tools (claim 16h):* a fresh game, with vehicles V and X of one type, W of
    another and Z of a third, none of which has held a route point, V in control group 1 (claim 22),
    cells P₁, P₂ and P₃ far from them, an own building that can be sold, an own damaged building, an
    own building that uses power, a finished building ready to place and a charged superweapon of
    the player's whose icon offers targeting. X stands in view. Give V and W plain move orders to
    far cells, and Z one to a farther cell. While they move: select V alone, hold the key, click P₁
    and P₂ and release. Select W alone, hold the key, press P₂'s marker and release (W joins, its
    route is P₂). Select W alone, hold the key, click P₃ and release (W's route is P₂ and P₃).
    Select Z alone, hold the key, press P₃'s marker and release (Z joins, its route is P₃). (1) With
    nothing selected, hold the key and click V. Expected: V and W selected, since W holds V's last
    point, and Z not selected, since Z holds W's last point but not V's. Z selected too falsifies
    the single round of selection. Release, deselect, hold the key and click W. Expected: W and Z
    selected, since Z holds P₃, and V not. Release, deselect and click V without the key: V alone
    selected. Deselect, hold the key and drag a band box over V and X, with W and Z outside it.
    Expected: V, X and W selected, Z not, with no incompatible-selection text, though V holds a
    route and X does not: the box selects as outside route mode, selecting V also selects W, and the
    box's release is not tested for route state. Release. (2) With nothing selected, hold the key
    and press group key 1. Expected: V and W selected, Z not. Release, select X alone without the
    key, hold the key and tap the type-select role. Expected: V, X and W selected, Z not. (3)
    Release, deselect, hold the key, press P₃'s marker (the point-selected text), press group key 1,
    wait 1 second, press delete and release. Expected: W's route line still ends at P₃. (4) Select
    V, hold the key and, with it held, click the building that can be sold in sell mode, the damaged
    building in repair mode and the power-using building in power mode, and place the finished
    building on a free cell. Expected: each takes effect at once, before the release, and no marker
    appears. Then arm the superweapon, which empties the selection (S-0512 claim 25), and press P₁'s
    marker. Expected: the point-selected text and no launch. Then click a far cell: the superweapon
    is launched at once. Release. Falsified if any expectation fails, or V's route line changed in
    (4).
    (5) While V and W are still on their plain moves (else repeat the set-up in a fresh game),
    arm sell mode, then hold the key: the route markers are drawn and sell mode stays armed.
    Right-click bare ground with scripted input, press and release at one screen position.
    Expected: sell mode ends, no building is sold, and the markers are still drawn. Arm sell
    mode again while the key is held: the markers are still drawn. Release the key: sell mode
    stays armed; disarm it. With nothing selected, hold the key and press P₃'s marker (the
    point-selected text). Right-click with press and release 4 screen pixels apart on one axis,
    then press delete. Expected: P₃ is deleted (W's route line ends at P₂). Press P₂'s marker,
    right-click with press and release 3 screen pixels apart on one axis, then press delete.
    Expected: no point is deleted. Press P₂'s marker again, right-click with press and release
    at one position, then press delete. Expected: no point is deleted. Record for each right
    click whether the view scrolled: a run in which the 3- or 4-pixel right click scrolled the
    view (a drag on that host, S-0214 claim 27) is void for that right click. Each marker is
    pressed at its screen position found from the camera position (as in part h). Release.
    Falsified if a right click ends route mode or leaves sell mode armed, if arming or ending
    sell mode ends route mode or route mode ends sell mode, if the 4-pixel right click clears
    the point selection, or if the 3-pixel or one-position right click leaves it.
  - *(z) The delete key and the end of route mode (claims 16e and 16h):* a fresh game, with a
    vehicle V of the player's that has not held a route point and far cells P₁ to P₈. Give V a
    plain move order to a far cell and, while it moves, select V, hold the key, click P₁ to P₆
    and release. (1) Deselect V, hold the key, press P₂'s marker, press delete, wait 1 second,
    press delete again and release. Hold the key with nothing selected. Expected: markers at P₁,
    P₄, P₅ and P₆ only. Release. (2) Press delete with the key released, then hold the key.
    Expected: no further marker gone. (3) By scripted input, hold the key with nothing selected,
    press P₄'s marker, then press delete and release the key within the same wait between two
    ticks (Scenario O9). Wait 1 second, press delete, then hold the key. Expected: markers at P₁,
    P₅ and P₆ only: the delete is applied in its own tick before the release is taken (claim 1),
    and the release clears the point selection it gave. (4) Select V, hold the key, switch to
    another window, let the key go there and return. Then click P₇. Record whether the markers
    are still drawn and whether the click adds P₇ to V's route or gives V a move order. This
    settles the original's open case of claim 16h, either way; our engine ends route mode at
    the window change. (5) As (4), but open the options menu while holding the key, let the key
    go there and close the menu, with the same records. (6) If the command
    bar shows a route-mode button, select V, click the button, click P₈ and click the button
    again. Expected: markers drawn from the first click on the button to the second, and P₈
    added and committed at the second. Falsified if any of steps 1 to 3 fails, or the button
    does not end route mode with a commit.
  - *(aa) Attack moves in a loop (claim 16c):* a fresh game on open flat ground, with an armed
    vehicle V of the player's whose type is eligible for attack moves (claim 6; which type is
    kept local), and an enemy vehicle E of a type that cannot cloak, made unable to fire (a test
    value), which stays where it is. E stands out of V's weapon range of V's start cell and of
    every cell that V passes on its way from there to P₁ and on from P₁ to P₂. Q is the first
    cell of the leg from P₂ to P₃ at which E is within that range, and E stays within it from
    Q on to P₃ (cells kept local). V takes more than *N* + 3 ticks to go from Q to P₃, where
    *N* is the normal targeting delay of S-0215 claim 33 (the lengths, V's speed and *N* kept
    local), so V's targeting timer runs out at least once while V is between Q and P₃ and E is
    in range.
    Run 1: select V, hold the key, click P₁, P₂ and P₃ (bare ground), click P₁'s marker again
    and release. Run 2: the same route without the click on P₁'s marker. Record V's cell, its
    current behaviour, its target and its shots every tick until it first reaches P₃ or takes E
    as target. Expected: in run 1, V takes E as target between Q and P₃. In run 2, V reaches P₃
    without taking E as target. A run in which V reaches P₃ within *N* + 3 ticks of first
    reaching Q is void, and so is a run in which V takes any target, or passes a cell within
    that range of E, before it first reaches Q. Falsified if V in a run 1 that is not void
    reaches P₃ without taking E as target. If V in run 2 also takes E as target, the part
    decides nothing.
  - *(ab) A recorded enter order at a repair building (claim 16b):* a fresh game on open flat
    ground, with a ground vehicle V of the player's at full health, of a type that a repair building
    repairs (which type is kept local), and an own repair building D, built up, powered and not
    being sold. No other vehicle stands on, enters, is queued for or waits at a contact place of
    D, from before the clicks to the end of the run, so that a contact place of D is free for V
    (S-0214 table V, building-entry value). P₁ is a bare cell a short way from D. Run 1: select
    V, hold the key, click P₁, then D, and release. Run 2: a fresh game with V damaged (a local
    value below full health), the same clicks. Record the action that the cursor shows over D at
    the click, the order that V issues when it falls back to rest at P₁ (its action, target and
    destination) and V's cell every tick until it stops. Expected: in both runs the cursor shows
    *enter* over D, and V issues an order whose action is *enter*, with no target and with D as
    its destination (S-0214 table M), and heads for a contact place of D. A run in which any
    other vehicle stands on, enters, is queued for or waits at a contact place of D before V
    stops is void. Falsified if any expectation fails in a run 1 or a run 2 that is not void,
    with the cursor showing *enter* over D in both. This tells the recorded order apart from a
    cell fallback in run 1 or in both runs (V issues an order with a cell as its destination, or
    with no destination), from a classification made at the issue (V's orders in the two runs
    differ) and from any other order (an action other than *enter*, or another destination). If
    the cursor in either run does not show *enter* over D, the part decides nothing.
  - *(ac) A copied capture issued by a vehicle (claim 16b, *Joins*):* a network game of two humans
    not allied (as in part h). The first player has an infantry unit N of a type that captures
    buildings and a drive vehicle T (S-0207 Terms) with a primary weapon (armed as in S-0209
    claim 5), of a type that is neither a train type
    nor has the crusher flag, not a crusher and in contact with nothing (S-0209 Terms), both at
    rest and neither having held a route point. The second player has a building X of a type that
    N can capture, built up and not being sold, not cloaked, and of none
    of the kinds that S-0209 claim 5 rows VP5 to VP16 and VP27 name (so that, with nothing else
    listed in X's cells, no row before VP28 decides any of them for T), and makes no input. T's own
    click action on X is *attack*. F is a bare cell far from X, on the side of N away from X, so
    that N's trip to F lasts longer than T's trip to X. Select N alone, hold the key, click F, then
    X (the cursor shows *capture* over X), and keep the key held. Click T to select it, press the
    marker of the point at X's place, and release. Record, every tick from the release until N
    reaches F, T's current and queued behaviour, its target and destination, its cell, every shot of
    T's, and X's owner and health. Expected (run 1): T issues an order whose action is *capture*,
    with no target and with X as its destination, has *capture* current, and heads for X. T makes no
    shot at X, and X keeps its owner and health, until T stands in a cell next to X. While T stands
    in that cell, before it enters any cell of X, it gets *attack* current with X as its target and
    no destination, and X keeps its owner while N is on its way to F. Falsified if T issues another
    order (*attack* on X, as its own click would give, or a *move*) or issues none, if T fires at X
    or X loses health before T stands next to X, if T enters a cell of X, or if T still has
    *capture* current with no target when N reaches F, X standing. Run 2: as run 1, with N of a type
    that plants explosive charges and X of a type that accepts them and meets run 1's conditions on
    X, the cursor showing *sabotage* over X at N's click; the same expectations and falsifiers with
    *sabotage* in place of *capture*. If the cursor does not show the run's action over X at N's
    click, or T is not given the point, the run decides nothing.

**Scenario O9: one input event per cycle (claims 1 and 3).**
- *Setup:* one vehicle of the player's, selected, moving towards a far cell *A*. Game speed 6
  (slowest), so that each tick lasts as long as possible. Input is scripted (measurement setup,
  scripted input).
- *Procedure:* the script waits until the recorder shows the tick counter advance, then at once
  gives a move click (button down and up) on a new cell *B* and presses the stop key, all within
  a few milliseconds, well inside one wait between ticks. Restore the vehicle's move to *A* and
  repeat, 20 attempts.
- *Runs:* the 20 attempts form one run; no random draw is involved.
- *Observable:* for each attempt, the sequence of the vehicle's recorded destination over the
  samples, and the tick of each change. Each attempt falls into one of three classes: (A) the
  destination goes from *A* to *B* and never becomes nothing; (B) it goes from *A* to *B* and then
  to nothing in a later sample; (C) it goes from *A* straight to nothing, with no sample showing
  *B*.
- *Expected:* class B in every attempt, with the stop applied 1 to 3 samples after the move (one
  sample per input event still waiting: the button going up, if it is taken as its own event,
  and the key). This confirms that the inputs of one wait are taken one per cycle, and so that a
  person's input cannot put a move and a later stop to the same actor into one tick (claim 3).
- *Falsified if:* any attempt is of class A or C, which needs both orders in one tick.

**Scenario O10: group destinations (claim 7).**
- *Setup:* 9 identical vehicles of the player's in a 3 × 3 block on open ground, all selected, so
  that the click gives all of them the same destination cell before the spreading. In this part
  and in parts d and f, every vehicle stands at its cell's centre at the click, as the recorder
  shows (claim 7's statement for compact blocks); a run in which one does not is void.
- *Procedure:* one move click to an empty cell 10 cells away. Repeat 5 times from the same start.
- *Observable:* in every part of this scenario, the start cell and the destination cell of every
  selected unit after the click, read by the read-only recorder, with the issue order of the
  click's orders.
- *Expected:* the centre vehicle of the block (the leader) gets the clicked cell, and every other
  vehicle gets the clicked cell plus its own cell displacement from the centre vehicle, so the 9
  destinations repeat the 3 × 3 block around the clicked cell, all different.
- *Falsified if:* all 9 vehicles keep the clicked cell (no spreading), the centre vehicle does not
  get the clicked cell, or any vehicle's destination differs from the clicked cell plus its
  displacement. A vehicle one cell out on a diagonal given a cell two out falsifies the first point
  of the look.
- *Part (b), shared cells:* 3 infantry of one stock type standing in one cell and 3 more standing
  one per cell in a row 3 cells away, all selected; one move click to an empty cell 10 cells
  away; 3 runs. Expected: the 3 infantry that started in one cell get one destination cell
  between them. Falsified if they get different destination cells. (Spreading involves no random draw.)
- *Part (c), leader tie-break:* 2 identical vehicles *P* and *Q* of a stock type whose primary
  weapon has positive damage, owned by the player and standing at the same place within their
  cells, in two cells 4 cells apart, so that the centre of the two lies exactly half way
  between them and both are at the same distance from it. Select *P*, then add *Q* by a
  modifier-click, and give one move click to an empty cell 10 cells away. Repeat from the same
  start selecting *Q*, then adding *P*. 10 runs of each chronological selection order.
  Expected: selecting *P* then *Q* produces selection and issue order *Q*, *P*, so *P* gets
  the clicked cell as the latest in the run. Reversing the additions gives that cell to *Q*.
  Thus the vehicle selected earliest wins this tie, and the other gets a different cell.
  Falsified if the vehicle selected last gets the clicked cell. Spreading involves no random draw.
- *Part (d), the look classes:* a purpose-made map with, around one clicked cell *k* on open ground, three
  variants, each with 9 identical vehicles of the player's in a 3 × 3 block 8 cells from *k*, all
  selected, one move click on *k*, 5 runs each: (i) *k* on a shore, with water on the side
  away from the block (a cell the vehicles cannot enter, in another reach region); (ii) *k*
  beside a cliff whose top is 2 levels above *k* on the side away from the block, and the same
  map with the cliff 3 levels high; (iii) *k* with a row of the player's own buildings on the side
  away from the block. Observable: the destination cell of each vehicle. Expected: no vehicle gets
  a water cell, a cell on the far side of the water, a cell covered by a building or a cell 3 or
  more levels from *k*; in (i) and (iii) the vehicles whose look would cross the obstacle get the
  clicked cell or a cell that another vehicle already got; in (ii) with the 3-level cliff the
  same holds for the vehicles whose look reaches the cliff, and with the 2-level cliff the cells
  on top of it may be given. Each vehicle's destination is also compared with the one claim 7
  predicts from the recorded start cells, issue order and map. Falsified if any vehicle gets a cell
  of a refused class, if a vehicle whose look is spoiled gets an open cell beyond the obstacle
  rather than its fallback cell (a further search), or if any destination differs from the
  prediction.
- *Part (e), attack-move group:* the 9 vehicles of the main part, each of an attack-move-eligible
  type, one attack-move click to the same cell, 3 runs. Expected: the destinations are spread as in
  the main part. This shows that a run of attack-move orders spreads like one of plain orders.
  Falsified if all 9 vehicles keep the clicked cell.
- *Part (f), larger groups:* on the open ground of the main part, each block selected alone, one
  move click to an empty cell 10 cells away, 5 runs each: (i) 12 identical vehicles in a 4 × 3
  block; (ii) 13 and then 14 identical vehicles in one straight row of adjacent cells along a map
  axis; (iii) 12 infantry of one stock type, 3 per cell in a 2 × 2 block of cells. Expected for
  the vehicles, in runs in which every vehicle starts at its cell's centre: in
  (i), and with 13 vehicles in (ii), every vehicle's destination is the clicked cell plus its
  displacement from the leader, so no two share. With 14, the one vehicle 7 cells from the
  leader shares the cell of the vehicle 6 cells out on its side (its 6 points all meet taken
  cells), and no other two share. Expected for the infantry in (iii), in every run whatever
  their positions inside their cells: each starting cell's 3 infantry share one destination,
  4 destinations in all. Their positions inside the cells decide only which unit leads; the
  leader's cell gets the clicked cell, and the other three starting cells, which lie in three
  different directions from it, each get the cell next to the clicked cell in their own
  direction, since a look runs from the leader's cell to the member's cell. The number of units sharing a
  destination is counted per run. Falsified if, in such a run, a 13-vehicle row or the 4 × 3 block has two
  vehicles with one destination, if the vehicle 7 cells from the leader gets a cell 7 cells out
  (a longer look), or
  if infantry of one starting cell get different destinations.

**Scenario O11: detonation of a bomb planted by a computer house (claims 3 and 11).**
- *Setup:* a named test rules file that lets the player set off that kind of bomb; a computer
  house's bomb-planting infantry plants a bomb on a vehicle of the player's (set up by a map
  trigger, or by placing the units so that the computer's infantry attacks).
- *Procedure:* select only the bombed vehicle and click it.
- *Runs:* 3. The detonate rule involves no random draw.
- *Observable:* whether the bomb goes off at the click.
- *Expected:* nothing happens at the click; the bomb goes off only when its own timer ends.
- *Falsified if:* the bomb goes off at the click.

**Scenario O12: power and primary switches (claim 13).**
- *Setup:* two factory buildings of the player's that build the same kind of object (for example
  two barracks), not naval; one building of the player's that needs power (its type has a base drain above 0 or the
  needs-power flag, S-0405 claim 10) and can be switched off. For step 3 only, a named test rules file sets the firestorm-wall flag on that power-using
  building type.
- *Procedure:* (1) power mode: click the power-using building twice. (2) Select factory *A* and
  click it on itself; then select factory *B* and click it on itself; then click *B* on itself
  again (a click that creates no order, by the input rule of claim 13). (3) With the test rules file: power mode, click the power-using building once.
- *Runs:* 3 from the same start. No random draw is involved.
- *Observable:* (1) whether the building is switched on after each click (its power use in the
  house's power balance); (2) which factory shows the primary marker after each click; (3) whether
  the building's power switch changes state.
- *Expected:* (1) switched off after the first click, switched on after the second; (2) *A* primary, then
  *B* primary and *A* not, then still *B* primary; (3) no change.
- *Falsified if:* (1) the state does not switch; (2) *A* stays primary after *B* is made primary,
  or *B* stops being primary at the third click; (3) the flagged building is switched off.
- *Part (b), imported categories:* named test rules give two otherwise selectable buildings
  the same recognised non-producing category from S-0003 Import mapping row C19, and a
  third the corresponding producing kind. Keep all three non-naval. At game speed 6,
  select and self-click each non-producing building in turn, then the producing building.
  Repeat three times from a fresh start. Record the primary designation after each click
  and whether any production line becomes available. Expected: the two non-producing
  buildings gain the designation in turn, the second clears the first, and designating the
  producing building does not clear the second because the categories differ. Neither
  non-producing category grants a production line. A no-factory control cannot gain the
  designation. A refusal to designate either recognised non-producing category, clearing
  across different categories, or production enabled solely by such a category falsifies
  the relevant claim. This part tests category recognition separately from production.

**Scenario O13: checks when the order is applied (claims 4 and 17).**
- *Setup:* (a) an armed vehicle *V* of the player's, selected, moving towards a far cell *D*; an
  enemy infantry unit *E* at low hit points that a defensive building of the player's is about to
  kill. (b) An infantry unit *I* of the player's, selected, walking into a friendly transport.
- *Procedure:* (a) at the moment *E* is about to die, click *E* with the attack cursor. Repeat for
  at least 30 attempts (statistical run). (b) Just before *I* reaches the transport, give it a move
  click to an open cell *C*. Repeat for at least 30 attempts. The clicks may be given by scripted
  input (measurement setup, scripted input), timed from the recorder's samples of *E*'s hit points and of *I*'s
  position, so that more attempts fall into the tick of interest; the counting is the same.
- *Observable:* per sample: (a) *V*'s target and destination, and whether *E* is still live,
  (b) whether *I* is inside the transport, and its destination.
- *Expected:* (a) each attempt shows one of: *V*'s target becomes *E* in a sample in which *E* is
  still live (the order came in an earlier tick); *V*'s destination becomes a cell at *E*'s place
  (the click came after *E* was gone, so it was a move click); or *V* keeps destination *D* and no
  target (the order came in the tick *E* died and had no effect). (b) Each attempt shows either
  *I* walking to *C* outside the transport, or *I* inside the transport with no destination.
- *Statistic:* the attempts of (a) and (c) are counted per class. At least one attempt of the
  third class of (a) is expected over 30; a series with none is inconclusive and is extended.
- *Falsified if:* (a) *V*'s target becomes *E* in the first sample in which *E* is no longer live,
  (b) *I* is recorded inside the transport with *C* as its destination, or leaves the transport
  because of the click.
- *Part (c), route removal by an order that fails its check (claim 17):* as (a), but *V* is a route
  holder: before each attempt it is given a planned route of three cells *D*₁, *D*₂, *D*₃ in route
  mode, and the attempt is made while it heads for *D*₁. At least 30 attempts. Observable: the
  classes of (a), and in each attempt of the third class (the order had no effect) whether *V*
  later heads for *D*₂. Expected: in no attempt of the third class does *V* go on to *D*₂ or *D*₃.
  Falsified if in any attempt of the third class *V* continues its route to *D*₂.

**Scenario O14: guard-spot orders from building clicks (claim 20, cases (a) and (c)).**
- *Setup:* a barracks, a vehicle factory and a repair depot of the player's on open, flat
  ground; a building of the player's standing 8 cells from them; a vehicle of the player's standing still
  on an open cell 6 cells from them. For step 5 only, a separate game with a named test rules file
  that gives a stock aircraft factory type the repair-depot flag, and a map on which that aircraft
  factory stands on one land area and an open cell lies on a second land area that ground units
  cannot reach from the first (separated by water or cliffs).
- *Procedure:* (1) select the barracks and set its rally point with a plain click (S-0214, row
  BC5) on an empty open cell 6 cells away. (2) Set it again on a cell covered by the other
  building. (3) The same two steps with the vehicle factory. (4) After step 2, build one infantry
  unit. (5) In the test game, select the aircraft factory and click the open cell on the second
  land area. (6) Select the barracks and click the player's vehicle with force-move held (S-0214,
  row G7). (7) Select the repair depot and click the same vehicle with force-move held.
- *Runs:* 3 from the same start. No random draw is involved; the search's choice among several
  equal cells may depend on the tick count (S-0212, claim 25), so the cell chosen in steps 2 and 6
  may differ between runs.
- *Observable:* the building's guard spot (a cell or an object) in each sample after each
  click; in step 4, the new unit's destination once it has left the barracks.
- *Expected:* (1) the guard spot becomes the clicked cell in the sample after the click; (2) the
  guard spot becomes a cell next to or near the other building that a unit of the factory can
  stand on, not a cell covered by the building; (3) the same for the vehicle factory; (4) the new
  unit heads for the barracks' guard spot; (5) the guard spot becomes the clicked cell on the
  second land area (flying zone); (6) the guard spot becomes a cell at or next to the vehicle's
  cell, not the vehicle itself; (7) the guard spot becomes the vehicle itself.
- *Falsified if:* the guard spot does not change after a click; in (2) or (3) it becomes a cell
  covered by the building or a cell more than a few cells from it; in (5) it stays on the first
  land area or no order is made; in (6) it is the vehicle; in (7) it is a cell. In (1), a guard
  spot other than the clicked cell falsifies only the low-confidence part of claim 20 (the choice
  among qualifying cells); (5) tests the low-confidence movement-zone part only.

**Scenario O15: packing up by a sell order (claims 12 and 20, case (b)).**
- *Setup:* a skirmish with the lobby option that lets construction yards be packed up again on
  (game A) and off (game B). In each game the player has two construction yards on open, flat
  ground, one of which (*Y*₁) has never been clicked, and enough room around them.
- *Procedure:* game A: (1) select *Y*₂ and click an empty open cell 6 cells away. (2) With nothing
  selected, sell mode: click *Y*₁. Game B: (3) the same two steps.
- *Runs:* 3 of each game, each from a fresh start. No random draw is involved.
- *Observable:* per sample, each yard's guard spot and whether it still exists, whether a vehicle
  of the yard's undeploy type appears in its place, and the house's credits.
- *Expected:* (1) in the sample after the click *Y*₂ has the clicked cell as its guard spot and
  starts to pack up, and a vehicle replaces it. (2) *Y*₁, which has no
  guard spot, is sold: the house is paid for it and no vehicle appears. (3) In game B the click of
  step 1 gives no order (*Y*₂ is not movable, S-0214 Terms), and *Y*₁ is sold in step 2.
- *Falsified if:* in (1) *Y*₂ is sold with no vehicle in its place, or nothing happens; in (2) *Y*₁ packs up (that
  falsifies only the low-confidence guard-spot condition of claim 12); in (3) a yard packs up.

**Scenario O16: an owner change in the tick of the order (claims 15 and 12).**
- *Setup:* a named test map. The human player owns a vehicle *V*, selected and moving towards a far
  cell *D*, and a building *K* that can be sold. Both carry the tag of a map trigger whose event is
  an elapsed time and whose action is *change owner* (S-0011, claim 24), with a house value that
  resolves to the computer house (S-0011, claim 23), such as the computer player's start position.
  That action takes effect in the polling of map triggers, before the turn pass of its tick
  (S-0203, claim 1). Input is scripted (measurement setup, scripted input).
- *Procedure:* a first run records the tick *T* in which the owner of *V* and *K* changes, and
  the attempts aim at that *T*. The classes below are judged from the samples alone, so if *T*
  varies between runs, only the spread of the attempts over the classes changes. Part (a): from a fresh start, the
  script waits for the tick counter and gives one move click for *V* on an open cell *B*, far from
  *D* and from *V*, aimed at the wait before one of the ticks *T* − 2 to *T* + 2. At least 5
  attempts per aimed tick, 25 in all, each from a fresh start. Part (b): the same with *K*: with
  nothing selected, enter sell mode beforehand and give one sell click on *K*, aimed in the same
  way; 25 attempts.
- *Runs:* the attempts of each part form one statistical series. No random draw affects the
  observables before the owner change.
- *Observable:* per sample, the owner, destination and target of *V*, and the owner of *K* and
  whether it is being sold. Each attempt of (a) falls into one class: (A) *V*'s destination
  becomes *B* in a sample in which *V* is still the player's (the order was applied before the
  change); (B) the first sample in which *V* belongs to the computer house is also the first one
  in which its destination is *B*; (C) *V* changes owner and its destination never becomes *B*
  (the click came after the change, and S-0214 made no order for an object that is no longer the
  player's). Each attempt of (b) falls into one class: (A′) *K* is being sold in a sample in which
  it is still the player's; (B′) the first sample in which *K* belongs to the computer house is
  also the first one in which it is being sold; (C′) *K* changes owner and is not sold.
- *Expected:* (a) all three classes occur; class B occurs in at least one attempt, since a click
  taken in the cycle of tick *T* passes the input rules while *V* is still the player's, and is
  applied after the change with no owner check (claim 15). (b) Classes A′ and C′ occur and B′ never
  does, because the sell order checks the owner when it is applied (claim 12). This contrast
  confirms only weakly: it relies on part (a) to show that clicks aimed this way do land in the
  tick of the change.
- *Falsified if:* (a) a series of at least 25 attempts that contains both class A and class C
  contains no class B (the owner is then checked); (b) any attempt is of class B′. A series of (a)
  without both A and C did not bracket the tick of the change, is inconclusive, and is repeated
  with the aim moved.

**Scenario O17: orders and repeated destinations (claim 21, Q-0087 and Q-0072).**
- *Setup:* stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), game speed 6, on the flat skirmish map of
  the preamble. Use a human-owned ordinary crawling infantry unit whose type is neither a
  fraidycat nor a cyborg, and cause it to become prone by a nonlethal enemy hit. Place two
  open destination cells A and B well away, an enterable friendly transport, a capturable
  building and an enemy target beyond weapon range. Keep the observed area free of all
  bystander objects except these named ones. Use fresh prone states for each part and void
  attempts in which fear-driven get-up occurs before the tested assignment.
- *Procedure:* in separate attempts give a first move to A, repeat A while it is still the
  current destination, and change A to B. Repeat with attack-move to A. Give area guard
  on a cell, repeat while that cell is still the destination, and use the guard key on the
  occupied cell. Give enter and capture orders with suitable stock infantry, repeat them,
  and distinguish a duplicate enter ignored during contact. From fresh prone rest, scatter,
  commit a one-point route and attack the distant enemy, each in a separate attempt.
  Include a plain move to the occupied cell from no destination. Three valid attempts per
  case; scripted input may aim repeats from recorder samples, without writing game state.
- *Observable:* per tick, destination identity, target, current sequence, prone state and
  position; mark the first sample with each applied order's new references. Separate later
  destination changes during approach or route following from the order boundary.
- *Expected:* first or different assignments, an empty assignment and a move to the occupied
  cell from no destination make no destination-driven get-up request. An eligible repeat of
  the current non-empty destination can start get-up at the boundary, before movement. This
  holds for move, translated attack-move, area guard and enter or capture alike, except that
  an ignored duplicate enter assigns nothing. First scatter normally assigns a new cell and
  does not qualify. A recorded future route point alone does not qualify. The ordinary
  attack order leaves target the enemy and destination empty at its boundary; a later chase
  assignment is judged separately by the same equality rule, with its timing outside this
  spec. S-0107 M4 supplies the controls for refusal and fear-driven get-up.
- *Falsified if:* an otherwise eligible first, changed or cleared assignment directly starts
  get-up, or a qualifying repeated assignment fails despite an available, interruptible
  sequence. Get-up caused by a later assignment does not falsify boundary timing. An ordinary
  attack order assigning a chase destination at its own boundary falsifies the distinction
  from later approach; this test records no expected approach radius or stopping rule.

**Scenario O18: orders during factory release (claims 5, 8 and 9).**
- *Setup:* the preamble's flat skirmish map, game speed 6, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Produce
  an ordinary armed drive vehicle at a human-owned ground vehicle factory K. Use a named
  test map override setting K's door time to 1 game minute solely to extend the pre-exit
  observation window; this is test data, not a stock value. Separate runs set K's
  unloading-kind flag off and on (Import mapping). Keep the apron and its neighbours free
  of other objects. For part b, place a durable enemy target within the produced vehicle's
  weapon range, unable to damage it during observation. Use scripted input and the read-only
  recorder, recording contact and tether on both objects, target, destination, current and
  queued behaviour, position, body turning and factory stage. These are proposed recorder
  observables, not a claim that the recorder already exposes them.
- *Procedure:* select the newly released vehicle by drag selection after K's starting
  processing, while the door is opening and before the exit instruction. In separate fresh
  releases: (a) give a plain move click to an open distant cell, once with K's unloading-kind
  flag off and once with it on; (b) with the flag off, attack the enemy target, wait for the
  target assignment, then press stop while still tethered; (c) with the flag off, press
  scatter while the vehicle is stationary in guard with no destination and no body turn.
  Run matching untethered controls: an ordinary stationary vehicle of the same type on open
  ground attacks the same target and receives stop, or receives scatter from guard with no
  destination and no body turn. Verify the same input actions are available and the
  movement receiver's other refusal gates do not hold (S-0207 claim 16).
- *Runs:* three valid attempts per part and flag setting. Void an attempt if selection or
  input legality prevents the order, if contact or tether ends before application, or if a
  factory exit instruction, cell arrival, target death or another object's action overlaps
  the tested boundary. A series with no valid attempts is inconclusive, not confirmation.
- *Observable:* the boundary sample with the new move destination or attack target; after
  each tested key, target, destination, contact and tether compared with the preceding
  sample and with a no-key release control. Record the normal input acknowledgement where
  offered; no acknowledgement or no selected actor is not an ignored applied order.
- *Expected:* (a) the new destination appears while contact and tether remain with the flag
  off; with it on, that same boundary has no contact and no tether. This tests table T3 b,
  not how the subsequent exit proceeds. (b) the tethered vehicle retains its attack target
  and link after stop, whereas the untethered control loses target and destination.
  (c) the tethered vehicle gains no scatter destination; the untethered control gets the
  movement spec's nearby destination. No claim is made that stop halts an existing track.
- *Falsified if:* a valid move order breaks the tether with the flag off or keeps it with
  the flag on; a valid tethered stop clears the established target; or a valid tethered
  scatter assigns a destination. Control failure makes the corresponding key test
  inconclusive. Factory release does not by itself test untethered contact break, deploy's
  separate tether gate or aircraft tether timing.

**Scenario O19: default command, options and view keys (claims 22 and 23).**
- *Setup:* a fresh keyboard profile, a small purpose-made flat skirmish map with an open test
  area and the bystander isolated as in the preamble. Stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
  speed 2, 800 × 600. Pre-place a selected armed vehicle, an infantry unit and a deploy-capable
  unit of stock types, and for part d a second armed vehicle of the first one's type. For
  scrolling use a map large enough to keep the view away from its limits. Record the
  key-binding screen and a video, with the tick and order recorder of
  the preamble. Do not assume the workaround bindings in Q-0286.
- *Procedure, part a:* inspect the six command bindings before changing any. With appropriate
  objects selected, test S during movement, X at rest, G at rest and D on the deploy-capable
  unit; press C after the cheer interval permits it. Hold Z, click three empty cells and
  release Z. Observe each key-down and key-up separately. Repeat with stop assigned to an
  otherwise unused letter through the binding screen. Then restart with a named test copy
  of the keyboard preferences that removes only the stop-command entry, and test S again
  during movement. Restore the profile afterwards.
- *Observable and expected, part a:* the screen shows S, X, G, D, C and Z for the six roles.
  Their effects match table T2 and Scenarios O5, O6, O7 and O8. The first five issue on press,
  without a second issue on release; Z defers route following until release. After reassignment,
  the new stop key issues stop instead of S. With the stop entry removed, S does not stop
  the vehicle. A different fresh binding, an order issued only on release, movement along
  the route before releasing Z, or automatic restoration of the removed binding falsifies
  the corresponding claim.
- *Procedure, part b:* with the menu closed and no input block, press Escape. Close the menu
  using its return control. In a named test copy of the keyboard preferences, remove the
  options-command entry; in a second copy assign it to an unused letter. Restart before
  each test, and press Escape again.
- *Observable and expected, part b:* Escape opens the menu in all three unblocked tests,
  regardless of the saved options entry. Ticks stand still while it is open, as S-0201
  claim 8 specifies. Failure to open in an ordinary running test, or loss of Escape after
  changing the preferences, falsifies the corresponding assertion.
- *Procedure, part c:* pointer in the interior, hold and release each unmodified arrow in
  turn. Record view displacement by screen captures. Use S-0101 Scenario M7 for exact
  distances and combined directions rather than giving selected units movement orders.
- *Observable and expected, part c:* each arrow moves the view in its screen direction,
  and that contribution stops after release is accepted. A wrong axis or direction, no
  scrolling in an unblocked interior test, or continuing contribution after accepted release
  falsifies this claim. No unit receives an order from these default arrow inputs.
- *Procedure, part d:* inspect the forty control-group bindings before changing any. Select two
  tanks and press Ctrl+1, clear the selection and press 1, then select an infantry unit and press
  Shift+1. Scroll the view away, clear the selection and press Alt+1. Repeat with digit 0, then
  with the select binding of group 1 moved to an unused letter. Record the order recorder.
- *Observable and expected, part d:* the screen shows digit, Shift, Ctrl and Alt with the digit
  for the four commands of each group, and 0 for group 10. Ctrl+1 makes the group, 1 selects
  the two tanks, Shift+1 adds them to the infantry unit, and Alt+1 selects them and moves the
  view to them (S-0214 Scenario T26). The moved binding selects the group instead of 1. No
  group key issues an order. Other default keys, or an order from a group key, falsify claim 22.
- *Runs and tool:* three runs per part and keyboard-preference variant; manual input or
  scripted normal input with a read-only recorder. These are proposals, not recorded measurements.

**Scenario O20: edge-scroll presentation (claim 24, table E).**
- *Setup:* a large flat stock-data skirmish test map at 800 × 600, speed 6,
  no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), with no selected units and the view away from all scroll limits.
- *Procedure:* hold the pointer at the eight outer-edge sectors, then repeat at the
  corresponding map scroll limits. At the top and bottom edges also test positions just
  inside, exactly at, and just outside the 16-percent horizontal boundaries. Repeat
  left/right-edge tests at the 21-percent vertical boundaries. Repeat three times.
- *Observable:* cursor frame, full-canvas hot spot and view displacement, by screen
  capture and a read-only view-position recorder; include the outer sidebar edge.
- *Expected:* table E's allowed frames in the interior and blocked frames for a refused
  one-pixel trial. Band tests use S-0101 F9's represented constants and rounding,
  not decimal-percent arithmetic. Internal sidebar-boundary
  hover does not alone select an edge cursor. Any different frame, hot spot or refusal
  classification falsifies the corresponding part of claim 24.

**Scenario O21: route status messages (claim 25).**
- *Setup:* O8 game B, stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), at speeds 1 and 6.
  use screen/audio capture with a real-time clock and tick recorder. Repeat three times
  per speed. A separate named test string table replaces each of the two refusal entries
  with a distinct short invented text, then with text long enough to wrap; remove it after
  the test. No shipped wording is copied into the measurement proposal.
- *Procedure:* make the holder-limit refusal, then seven more refusals separated by
  completed ticks within less than seven seconds. Repeat with an incompatible selection
  that the eligibility check refuses. In separate runs pause by opening options, lose
  focus, or use a test map's timed freeze just after showing a notice, waiting longer
  than eight seconds before resuming display updates.
- *Observable:* imported text identity, line count/order, position, oldest-line eviction,
  wrapping, and clock count at appearance/removal. Record actual display-update times.
- *Expected:* the corresponding replacement text appears. At most six lines remain.
  later notices append rather than extend an earlier deadline. Each line remains through
  deadline equality and disappears on the first update after 480 clock units, about
  7.68 seconds, irrespective of speed. A pause longer than its lifetime leaves no notice
  after the first resumed update. Wrapped lines share capacity and duration, with
  deadlines based on each line's addition clock count.
  Wrong text, a seventh retained line, tick-based expiry, paused lifetime or refreshed
  old copies falsifies the corresponding claim. An update not sampled at exact equality
  cannot test that equality boundary.

**Scenario O21 b: status line breaks and font (claim 25, Q-0393).**
- *Setup:* O21's skirmish fixture, speed 6, separately at 640 × 480 and 800 × 600.
  Use a named test string table for the two refusal messages, with invented text
  only. Include different narrow and wide glyphs, spaces, punctuation, tabs,
  carriage returns, line feeds, paired breaks and an empty message. Use a named
  test font copy with
  changed glyph widths for a separate font-dependency control. Restore both assets
  afterwards. There is no engine add-on and there are no mods, except these named test assets and the purpose-made test maps.
- *Procedure:* show each message from fresh starts. Construct text whose advance
  is exactly the initial budget, then exceeds it by one character. Include a word
  longer than that budget, several spaces at the break, a 112-character narrow
  string, and repeated explicit breaks. In separate runs change the video
  resolution through the normal options controls, then show fresh messages again.
  Repeat three times per resolution and asset variant.
- *Observable:* screen-captured glyphs and their origins, battlefield width,
  text of each stored line, line count, continuation indentation and expiry.
  A read-only recorder may log the presented lines, distinguishing stored lines
  from a trailing break's blank drawing area.
- *Expected:* claim 25's integer advances select the fitting prefixes at *W* − 22
  pixels initially and *W* − 16 after the dimension change. Equality fits. Spaces
  or tabs permit a word-boundary break, punctuation does not, and a word alone
  splits inside itself. The character cap breaks after character 111 even when
  more width remains. Continuation spaces remain, while consecutive control
  breaks are skipped. A video-mode change clears old notices. Each explicit break
  produces the same capacity/lifetime
  treatment as wrapping, and empty text adds one blank stored line. The imported
  font, not a host font, supplies glyphs and
  metrics. Changed glyph widths change the breaks accordingly.
- *Falsified if:* any stable mismatch in break position, width regime, font,
  advance, retained spaces, explicit-break treatment or character cap. Keep
  individual tests to six lines or fewer so eviction does not hide a break.
  Do not run unfittable-font failure data against the original.

**Scenario O21 c: status glyph selection and foreground (claim 25; Q-0501).**
- *Setup:* O21's tiny skirmish fixture, speed 6, 800 × 600, isolated bystander
  and its void rule, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use named valid test font,
  string-table and colour-definition copies, restoring them afterwards. Keep a
  mapped capital X. Choose an invented HSV definition for the local player's
  slot whose direct packed RGB differs from shaded/ramp alternatives. Capture
  against two different battlefield backgrounds.
- *Procedure:* display a refusal notice containing two units mapped to the same
  glyph, a unit above 255 with a distinct low-byte glyph, a zero mapping, a
  positive-width blank glyph and a present zero-width blank glyph. Add a separate
  zero-width mapped glyph whose first stored row bit is set, keeping X positive-width.
  In separate fresh sessions alter only X's width or mask. Also assign large widths to the
  carriage-return and line-feed mappings and display explicit breaks. Repeat
  each variant three times, then repeat the colour test with altered battlefield
  lighting. Keep each notice below the wrapping budget and six-line capacity.
- *Observable:* screen-captured glyph masks, pixels inside and outside each mask,
  origins of following glyphs, explicit line breaks and foreground channel values.
- *Expected:* aliases match; the full 16-bit unit selects the glyph, not its low
  byte. Only a zero mapping uses complemented X with X's width plus one pixel of
  advance. Present blank glyphs retain their widths; zero width advances one pixel.
  The nonblank zero-width control can paint a stray first-column pixel in the oracle.
  Our engine paints none, without changing that advance.
  X alterations change the fallback only as claim 25 states. Breaks draw nothing
  and take no advance despite their altered mapped widths. Zero mask pixels
  retain each background; one mask pixels have direct converted/packed foreground,
  unchanged by battlefield lighting. The oracle's missing glyph has the recorded
  even-bit foreground inversion; our engine instead uses the ordinary foreground.
- *Falsified if:* low-byte selection, replacement of a present blank/zero-width
  glyph, a dedicated fallback independent of X, different advances, painted break
  glyphs, opaque zero pixels, palette-derived foreground, lighting-dependent text,
  or a different original fallback colour transformation is observed. Absence of
  the stray first-column pixel in the unclipped nonblank zero-width oracle control
  falsifies the documented original defect. This tests status drawing and metrics.
  S-0014 claims 1–3 own the binary file-layout contract.

**Scenario O21 d: fixed line spacing, pitch clipping and glyph top (claim 25; Q-0530).**
- *Setup:* O21's skirmish fixture and void rule, speed 6, 800 × 600, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Named test assets are a string table with short invented
  refusal text and structurally valid font copies under S-0014 claims 1–3, all
  with mapped X. Use positive-width synthetic glyphs with distinguishable set
  pixels in their first, middle and final rows. Keep text comfortably below the
  horizontal budget. Restore assets afterwards.
- *Procedure:* show two notices before either expires, and separately one notice
  containing an explicit break and one that wraps. Repeat with copies differing
  only in P: below H, equal to H, above H, and zero. Include a valid tall-mask
  copy with both H and P above 19 pixels, and a height-only variation that still
  fits its records. Use asymmetric row marks to distinguish overlap from a moved
  origin. Repeat each variant three times against two battlefield backgrounds.
  Capture again after expiry to check that no blank band remains.
- *Observable:* first stored row positions, differences between successive line
  origins, visible bottom row, wrapping, retained line count and background pixels.
- *Expected:* origins remain 19 pixels apart, each 3 pixels right of the battlefield's
  left edge, with the first at battlefield top.
  Stored row zero starts at each origin, regardless of H or P. The oracle clips
  rows below row P but can paint row P itself when H > P, including row zero at
  zero pitch. Our engine instead clips to min(H, P) rows and paints none at zero
  pitch. Tall admitted masks can overlap; P does not re-space the lines or clear
  a background band. Width-based breaks and capacity remain unchanged.
- *Falsified if:* an origin has a different horizontal inset or spacing, or changing P
  moves an origin, changes horizontal splitting or
  capacity, shifts the glyph top, or paints oracle rows beyond P; failure to
  paint a set boundary pixel at row P with H > P falsifies the recorded cosmetic
  defect. The corrected clipping is an engine acceptance check, not an oracle
  prediction. S-0014 F5 supplies invalid-font acceptance tests separately.

**Scenario O22: arrows during order blocks (claim 23, Q-0391).**
- *Setup:* a large flat skirmish map, no scroll limits near the observed view,
  speed 6, 800 × 600, fresh unconflicted arrow bindings, pointer in the interior.
  No engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use S-0201 claim 11's map-triggered chrono flash late
  enough to hold an arrow beforehand. Its early unfrozen blocked interval must
  contain the input tests. Use scripted normal key events and a read-only
  cycle/tick/view recorder, with video of pointer hiding and the flash.
- *Procedure:* separate fresh runs (a) press a previously unheld arrow during
  the unfrozen block and hold through its end, then release and press after it.
  (b) press before the block, release during it and leave released through its
  end, then press and release after it; (c) as b, but re-press during the block.
  Include the initial ticks 0 to 3 block for part a. Three runs of each case.
  Ensure no prior waiting input and identify events that fall wholly within the
  block; boundary-ambiguous attempts are void.
- *Observable:* directional displacement per unfrozen cycle, no displacement
  during frozen cycles, physical event times and first unblocked cycle.
- *Expected:* a has no arrow contribution during the block or at its end despite
  the held physical key; only the later accepted fresh press starts it. In b and
  c the pre-block contribution continues in every unfrozen cycle, even after
  physical release; frozen cycles stop displacement but retain the contribution.
  A release accepted after the block ends it. Distances and limits are S-0101
  claims 14 and 15, not new formulas here.
- *Falsified if:* a starts scrolling from the blocked press, including automatic
  start at unblock, or b stops from the blocked release. An unblocked press/release
  control must work; otherwise that run is inconclusive.

**Air-strike target changes (claim 3).** S-0519 Scenario T19 supplies the setup,
procedure, cancellation observables and falsifiers, including identical-target orders
and successive changes back to the original target. Record applied changes, not merely
issued orders or the final target. Claim 1's one-input-event limit means that a same-tick
A-to-B-to-A order sequence has no established normal-input oracle fixture here; test
that precedence consequence on our engine without claiming an oracle measurement.

**Production-start ordering (claim 3).** S-0404 P11 B/C supplies the setup, procedure,
observables, expected assignment and falsifiers for same-boundary cancellation starts and
turn-pass starts before order-boundary starts. S-0411 S9 tests cancellation-order consequences
when a queued replacement starts and is cancelled at the same boundary, versus removal before
it starts. These are proposed tests, not recorded results. The between-house order-boundary
extension is a design rule and has no oracle measurement.

## Open questions

- Q-0391's three input-acceptance questions are answered by claim 23 and O22.
  S-0101 claims 14 and 15 already own displacement and retained scrolling.
  S-0201 claim 14 owns the initial block and claim 11 the chrono-flash block.
  Their pointers need the cross-spec updates listed for sign-off below.
- Q-0501 questions 2 to 5 are answered by claim 25 and O21 b/c. Question 5's
  zero advance already appeared in Formulas before this request; claim 25 now
  makes it explicit. Question 1's binary font-file layout is answered by S-0014
  claims 1–3; its claims 4 and 5 specify fallback image and transparency.
  S-0005 claim 18a retains file identity and lookup ownership.
  S-0014 claims 4 and 6 now define missing-X original usability and our engine's
  whole-font rejection; claim 25 and Import mapping here apply that policy.
- Q-0530 questions 2 and 3 are answered by claim 25 and O21 d: origin spacing
  stays fixed, pitch controls vertical clipping, and row zero starts at the origin.
  Question 1 is owned and answered by S-0014 claims 4 and 6, including its F4/F5
  tests. No part owned here remains open.
- Q-0393's four shared-status-display questions are answered by claim 25 and O21 b.
  S-0411 claim 17 already excludes sidebar production labels from this display.
  Their font and layout are not changed by the status-text contract here.

- Q-0286 is answered jointly: claims 22 and 23 here answer default command bindings and
  options/view-key identities (questions 1 and 3); S-0101 claims 14 and 15 and Scenario M7
  own ordinary arrow scrolling. S-0214 claims 5 and 23 and Scenario T17 answer question 2:
  Alt force-move, Ctrl force-fire and Shift force-select, with Ctrl+Shift for the held
  attack-move gesture or the command-bar button for one-click mode, not a dedicated
  attack-move key. No part of this request remains open. The keyboard preferences file's
  full encoding and options-dialog closing shortcuts are outside this request.
- Q-2226 is answered by claim 16e (*The delete key*): the point that a delete of its longest
  holder's last point selects on a route whose loop ends early (question 1), and on a route that
  holds the deleted point at two places (question 2).
- Q-2227 question 1 is answered by claim 16b (*Aircraft and buildings*): in route mode a click
  that makes a factory the primary factory, or that sets a factory's rally point on an object,
  sends its order directly with no point, and the *unload* that lets a building's occupants out
  is a point add at the building's own place. Question 2 is answered by claim 16b (*Joins*) and
  Scenario O8 part ac.
- Q-2247 is answered by claim 16b (*Joins*) and Scenario O8 part ac: a drive vehicle with a
  primary weapon, not of a train type and without the crusher flag, in a copied *capture*
  attacks the building by an
  override once its next step is a cell of the building (question 1, so question 2 does not
  arise), and a copied *sabotage* does the same (question 3).
- Q-2228 is answered by claim 16d (*Aircraft and buildings*) and Scenario O8 part x (7): a
  building under *move* starts a later *attack* order when it has no turret and never when it
  has one, as S-0215 claim 11 already gives. S-0215 needs no change.
- Q-2229 is answered by claim 16d (*Aircraft and buildings*) and Scenario O8 part x (6): the
  issue at a flight's end is S-0501 claim 6's planned-route step and comes before its pad-bound
  test (question 1), an aircraft that issues a point begins no landing in that tick and stays
  in flight (question 2), and the issue is preceded by claim 16c's queue of *wait* (question 3).
- Q-2230 is answered by claim 16b and Scenario O8 part q (6): an attack-move click has the place
  that the same click gives without attack-move entry, so the clicked cell for a click on a
  cell (question 1), and for a click on an object the target's place or the object's cell at
  the click (question 2).
- Q-1988 is answered jointly: claim 22 here gives the control-group keys (question 1's binding
  part), and S-0214 claim 33 answers the rest.

- Q-0072: claim 21 answers the order-application contribution in full. S-0215 claim 37
  supplies the approach trigger, gating, renewal and building/deployed-infantry rules
  (questions 1, 4 and 5); part g's zero-speed qualifications are medium confidence and
  do not specify a complete locomotor contract. S-0307 claims 15 to 18 supply question 2's
  covered open-ground and obstructed searches, failure outcomes and vertical balloon-hover
  exception; search cases outside their stated scopes remain open there. Question 3's
  infantry part is answered by S-0208 claims 16 and 17. Its driving, sailing and hovering
  vehicle part is S-0207 claim 19. The approach step itself never clears a held destination
  merely because the target came in range (S-0215 claim 37 e); that does not override the
  movement-side rules. No owned part of Q-0087 remains open. The requests list records
  this spec's contributions, not closure of the cross-spec Q-0072.

- A stop or deploy order is ignored while the actor is in one further state that has not been
  identified. It is probably a unit travelling underground or through a tunnel, whose position
  has left the cell in which the game still counts it. Our engine's interim rule is in claim 8.
- How holders that share a route point synchronise their start there (claim 16c), and one
  further case, not identified, in which a point add with a cell target is ignored without a
  message.
- Whether leaving the map for a time ends a route by itself (claim 16f), the part of Q-1986
  question 6 left open, and whether an owner change keeps it (S-0215 claim 13, Open questions).
- Whether, in the original, a release of the route-mode key made while the game window lacks
  focus, or while the options menu is open, reaches the game, so that route mode ends then, or
  whether route mode stays on until the key is pressed and released in the game (claim 16h,
  O8 z). Our engine's rule (route mode ends with the commit at either event) does not depend
  on the answer.
- How the command bar's route-mode button behaves: whether it stays on until clicked again, and
  how it is shown (claim 16h).
- Whether a point join raises the route-holder count: claim 16f states that it does not, at
  low confidence (O8 o).
- Whether a point whose place is an object's moves with the object (claim 16b). It appears to.
- The look of route markers and lines (claim 16g) and the alliance direction that lets a house
  see another's routes; which sound a refused point add or a refused route-mode key command
  plays, and what an accepted point add plays when the route-point sound is absent.
- The scatter order also records on the actor that it was told to scatter. What this record
  changes later has not been studied.
- A behaviour order can name one further object besides its target and destination. No input
  names one, so our engine's AI houses, held to the same input rules (claim 3), name none either.
  What naming one would change has not been studied.
- The cheer command is also refused while a game-wide setting of unknown purpose is on. Whether
  anything in a skirmish game turns it on is not known.
- When a behaviour order names an object of a house allied with the actor's, the original also
  triggers a further effect on that object, probably a brief highlight on screen. It has not been
  studied; nothing suggests that it changes game state.
- Other tether exchanges beyond the ground vehicle factory release and successful infantry
  factory or cloning-building release exchanges of S-0417 (claims 1, 7 and 10), and docked
  aircraft release (S-0417 claim 11). A docked aircraft is tethered at release
  unless its type must return to a dock; landing and take-off timing remains provisional.
  The refinery case remains exploratory (Scenario O4 iv). Factory release does not yet
  identify an order-time untethered-contact interval or isolate deploy's tether refusal
  (claims 5 and 10).

## Changes for other specs

- **S-0214, Terms, *deploy readiness*.** The infantry clause should say that "deployed" covers
  every deploy kind of sequence (S-0107, Terms), *deploy* itself included, and that "the timer
  that follows a deploy" is the unit's rearm countdown (S-0301), which shots start and deploying
  does not. The vehicle clause should say that readiness does not depend on the vehicle's
  transition state. Table T2's deploy rule here (Q-1753) uses that reading. The same wording is
  submitted with S-0214's Q-1754 answer; the two go together.
- **S-0209, entry check inputs.** S-0209's list of reference height levels gives order
  targeting (S-0213, S-0214) as not established (Open questions). Claim 7 here fixes it for the
  *obstructed* class of a group destination look: the reference height level is the common cell's height level, plus 4
  when the common cell is on a high-bridge deck, and there is no connecting-step direction, so no
  connecting-step height restriction applies (`unmeasured`, medium). S-0209 may add that case
  to its list, pointing to S-0213 claim 7.

- **S-0701, claim 24.** Its route-mode cue covers the refused route-mode click only. Claims
  16b and 16e and table T2 here add further route-mode feedback for the local player: a refused
  point add (message), an accepted point add (the route-point sound), and the guard, stop,
  scatter and deploy commands refused in route mode (message and a sound). S-0701 may list
  these as cues; which sound each plays is not established here (Open questions).
- **S-0701, Scenario D4.** Its route-mode guard case should expect the refusal message and sound
  of table T2 here, with no guard point added, and no ordinary guard cue.
- **S-0215, claim 13, route row.** Every issue of a route point, at a fall-back to rest as at
  a commit, is preceded by the same queue of *wait* with an immediate start that claim 16c
  gives for the commit (Scenario O8, part t at P₁, medium). For a holder that is ready to
  change, *wait* is then current when the rest tables of S-0215 claims 14 and 15 run, since they
  run after claim 13's common effects, and their row 1 (current *wait*: nothing changes)
  applies, so a fall-back that issues a route point queues no *guard*. B15 b's fall-back at its
  first point issues the next point, so under this row it queues *wait*, not *guard*: its "that
  fall-back can queue *guard*" is an allowance that this case does not use, and B15 c's like
  allowance for infantry is the same. Neither allowance is part of what falsifies B15 b or c,
  so their expected results (the next point's *move*, at an order boundary, not through the
  queued destinations) are unchanged. S-0215's route row may say so, and B15 b and c may then
  expect *wait* where they allow *guard*. The route row's sentence "Within this fall-back it
  leaves the refreshed destination, target and slots alone" holds for the destination and the
  target, but not for the slots: the issue's queue of *wait* changes them, as at any other
  fall-back that issues a route point, also when memory survives. That sentence should read
  that the fall-back leaves the refreshed destination and target alone, and changes the slots
  only by the queue of *wait* that precedes the issue (S-0213 claim 16c).
- **S-0215, claim 13, route row: aircraft and buildings.** The row's fall-back that issues the
  next route point applies to infantry units and vehicles. An aircraft's fall-back to rest
  issues no route point: an aircraft issues its next point when a flight ends, at the moment it
  would begin to land, unless its behaviour is then *enter*. A building without the SAM flag
  issues its next point only when its *attack* behaviour has no target, and one with the SAM
  flag issues no later point (S-0213 claim 16d, medium, `measurable` in Scenario O8 part x (6)
  for the aircraft, `unmeasured` for both kinds of building). The
  row should name infantry units and vehicles only, pointing to S-0213 claim 16d for aircraft
  and buildings.
- **S-0215, claim 32, *attack* without a target.** For a building whose type does not have the
  SAM flag and that holds a route, the no-target case first issues the building's next route
  point, with the queue of *wait* with an
  immediate start of S-0213 claim 16c. When *wait* is then current, the building queues no
  *guard*. When it has no point to issue, or *wait* is not current, the claim's *guard* follows
  as stated (S-0213 claim 16d, medium, `unmeasured`: no scenario there gives a building an
  *attack* point). The SAM rows are unchanged: a building with the SAM flag issues no route
  point from its *attack* (S-0213 claim 16d, medium, `unmeasured`). Claim 32 may say so,
  pointing to S-0213 claim 16d.
- **S-0215, claim 13, *Repair/reload route points*, *Skirmish distinction* and Scenario B40.**
  For a point of S-0213 claim 16's routes,
  the order that the route row issues is the one recorded for the holder when the point was
  added, or copied to it at a join, as changed by a loop's attack-move change (S-0213 claims
  16b and 16c). It is not classified anew when it is issued, so the paragraph's classification
  as an ordinary object click, its service guards and its cell-action fallback ("A point with
  no object also uses the cell action") do not apply to those points (S-0213 claim 16b,
  medium, `measurable` in Scenario O8 part aa for a loop's *move* point changed into an
  attack-move order and part ab for a full-health vehicle's point on an own repair building,
  `unmeasured` otherwise).
  - Scenario B40 makes its routes with route-mode scripted input, so they are S-0213 claim
    16's routes, and these of its sentences do not hold for them: "Full-health repair-only and
    full-ammunition reload-only controls use the cell fallback.", "Add infantry at a
    repair-only building and an unlimited-ammunition actor at a reload-only building: neither
    service condition qualifies." and the falsifier "an object order in a nonqualifying
    control". With those routes each holder's point issues the order that the click gave that
    holder at the add, whatever its health and ammunition at the issue: the full-health vehicle
    at its own or a mutually allied repair building issues *enter*, as the damaged one does
    (S-0214 table V, row n3, and S-0213 claim 16b: medium, `measurable` in S-0213 Scenario O8
    part ab for the own building, `unmeasured` for the allied building and the reload cases,
    which no scenario in S-0213 sets up). A click that only selects gives no action and adds no
    point (S-0213 claim 16b), so a holder that the click gives no other action there has no
    point on the building. B40's original-game cases then test S-0213 claim 16b, not the
    paragraph's guards: B40 should expect the recorded order in every case, and claim 13's
    Backing should not name "B40 for repair/reload route-point selection" for those guards.
    B40's engine acceptance case, an AI house's damaged vehicle routed to its own repair
    building, still expects the object order, which the recorded *enter* gives.
  - The *Skirmish distinction (Q-0663 question 3)* paragraph's example, an AI-owned damaged
    vehicle routing to its own repair building, is a route of S-0213 claim 16 in our engine,
    since that claim lets an AI house issue route orders. The vehicle issues the order that its
    click recorded, with the AI house as acting house (S-0214), and no service or alliance guard
    is tested at the issue (S-0213 claim 16b, medium, `unmeasured`: no scenario in S-0213 has an
    AI house give route orders). The example's object order is what the vehicle issues, but
    not by the owner-based service branch: that branch and its two alliance guards apply only
    to routes other than S-0213 claim 16's.
  - A holder of one of S-0213 claim 16's routes
  issues the next point of its own route, as recorded for it (S-0213 claims 16b and 16c), so
  the *Whose route (original only)* paragraph's choice between the routes of the house played
  on the machine and those of the owner does not arise for it (medium, `unmeasured`: in every
  scenario here each holder's owner is the house played on the machine, so the two readings
  give the same next point).
  - The *Repair/reload route points*, *Whose route (original only)* and *Skirmish
    distinction* paragraphs should be limited to routes other than S-0213 claim 16's, pointing
    to S-0213 claim 16b for route points, and B40 changed as above. Whether any other player
    input gives an actor a route that those paragraphs cover is outside this spec.
- **S-0215, claim 5, input that creates *attack move*.** The sentence that ordinary player input
  does not create an attack-move order whose translation is the standing behaviour *attack
  move* has one exception: a building that holds a *move* point of a loop when the loop's
  attack-move change is made gets an attack-move order with a destination there, which claim 6
  turns into *attack move* itself. The building then does nothing, as claim 5 states for that
  behaviour, and issues no further route point (S-0213 claims 6 and 16c, medium,
  `unmeasured`: no scenario there gives a building a *move* point in a loop). Claim 5's
  data-only conclusion, claim 12's last sentences and the open question on whether an
  attack-move order can reach a building may say so, pointing to S-0213 claim 16c.

- **S-0214, claim 3.** Its route-mode sentence covers command clicks. In route mode a click that
  selects, toggles the selection or falls back to *select* is also refused, with the
  aircraft-in-selection, building-in-selection or other-kind-in-selection text, when a selected
  object cannot take part in a route and the press is outside every point's click area (S-0213
  claim 16d, medium). S-0214 may say so, pointing to that claim. The sentence's "adds route points
  instead of creating orders" also has exceptions: a click whose ordinary action is a detonate
  adds no route point and sends the detonate order at the button's release, when claim 16d's
  tests pass (S-0213 claim 16b, medium), and a sell, repair, power, placement, beacon or
  superweapon tool click on the battlefield is sent at once and adds no route point (S-0213
  claim 16h, medium). Our engine follows S-0213 for these clicks. Its refusals also have
  exceptions: a press that takes a point is not tested for route state or the route-holder
  limit, and a press inside a point's click area is not tested for kind (S-0213 claims 16b and
  16d, at their ratings there). S-0214 claim 3 should name these exceptions, pointing to those
  claims.
- **S-0214, claims 32 and 33, and selection by click and band box.** While the route-mode key is
  held, selecting an object whose route holds a point also selects the other objects whose
  routes hold that route's last point (S-0213 claim 16h, medium). S-0214 may point to it.
- **S-0214, claim 28, and Scenario T20.** The route-editing mode that the claim lists is not route
  mode (S-0213 claim 16): no input of the original starts route-editing (S-0213 claim 16h,
  `unmeasured`, medium), and our engine has no such mode. The claim should drop it from the
  modes that a right click cancels and from the modes that exclude one another, and T20 should
  drop its route-editing case. It may add that a right click while the route-mode key is held
  acts as the claim says and does not end route mode, that a short one also clears a selected
  route point, and that route mode and the claim's modes do not exclude one another (S-0213
  claim 16h, medium).
- **S-0214, claim 27.** Route mode's short right click, which clears a selected route point, has
  a bound of its own: its release must be less than 4 screen pixels from its press on each axis.
  The bound is measurable on one host (S-0213 claim 16h, medium, O8 y (5)). In the original it
  does not follow the host's drag settings (S-0213 claim 16h, `unmeasured`, medium), and our
  engine keeps it. It is not claim 27's drag bound, and claim 27 need
  not change: it may point to S-0213 claim 16h for that bound.
- **S-0214, table M, part 3, deploy row.** A building is given the *deploy* action only by a click
  on the building itself, and each behaviour order *unload* that the row gives names the
  building itself as its target (S-0213 claim 16b, low, `unmeasured`: read from the original,
  no scenario there gives a selected building the deploy action, and the level is capped by
  S-0214 claim 10's low mapping). Part 2's *unload* row, which names no target and no
  destination, is unchanged: it is a different order from this one. If S-0214 adopts it, the
  row names the building as the target of each *unload*, at low, pointing to S-0213 claim 16b,
  which uses it for the place of a building's route point.

- **S-0501, claim 6, planned-route exception.** Its further waypoint of a planned route is the
  next route point of S-0213 claim 16, and going on to it is the issue of S-0213 claim 16d
  (*Aircraft and buildings*). That issue comes before the pad-bound test, so a pad-bound
  aircraft away from its pad that has a point to issue issues it and does not fall back to
  rest, and an aircraft that issues a point begins no landing in that tick (S-0213 claim 16d,
  medium, `measurable` in Scenario O8 part x (6)). S-0501 may point to S-0213 claim 16d in place
  of "the waypoint planning of the order specs". Its rule is otherwise unchanged.
- **S-0512, claim 25.** It leaves open whether arming a superweapon ends route mode. Arming it
  leaves route mode on (S-0213 claim 16h, medium, Scenario O8 part y (4)). S-0512 may close that
  open question, pointing to S-0213 claim 16h.

## Questions for the project lead

- **Claim 7, no further search.** After a spoiled look the original gives the fallback cell
  even when an open cell beyond the obstacle could be reached around it, so groups moved next to
  water, cliffs or buildings send several members to one cell, where they jostle. The default is
  the original's rule. Should our engine also offer a search for an open, reachable cell near the end
  of a spoiled look, as `quirks.group_spread_no_detour` (default on: the original)? If so, the search's rules are
  specified before it is built.

No other decision is pending. The project lead's decisions of 2026-10-09 (AI houses,
scripted input, the click marker) are applied. The surrender, alliance and power rows of
table T1 defer to S-0414 (claims 11 to 14) and S-0405 (claim 10); the game-end decision of
2026-10-11 (play to the last side) is S-0414's. Items to confirm against the versions handed
off together at sign-off:

- **Q-0391 companions:** S-0201 claim 14 and its open question must replace
  the unresolved new-arrow-input qualification with claim 23 here: blocked presses
  and releases are discarded, retained contributions are unchanged, and unblock
  does not synthesize a press. S-0101 claims 14 and 15 should cite that input rule
  alongside their unchanged displacement and pause rules. Their scenarios should
  point to O22, rather than treating newly pressed arrows as expectation-free probes.
- **Q-0393 companion:** S-0411 claim 17 should explicitly point to claim 25 here
  for shared status wrapping, explicit breaks and font metrics, while retaining
  sidebar labels as a separate presentation contract. It already supplies the
  no-production-status-message part; no new sidebar message event is needed.
- **Selection order already agrees:** current S-0214 claim 12 and claim 1's representative
  tie-break use the order here and in S-0701 claim 18: positive-primary-damage types first,
  most recently selected first, then other types, earliest selected first. Confirm these
  matching passages and the tied-representative scenario at sign-off. Claims 1, 2, 7 and 18 here, table T2 and Scenarios
  O2 and O10 use that order; the pending-order limit and leader tie-break themselves are unchanged.
- **Route timing confirmed:** S-0215 claim 13 already issues the next route point through
  the ordinary order boundary, as claim 16 here states; no immediate destination assignment
  is implied. The first point is issued at the commit (claim 16c), not at a fall-back. The
  order each point issues is the one recorded for its holder (claim 16b), not one chosen by
  S-0215 claim 13's route-point selection rule (Changes for other specs).
  Confirm the current companion text at sign-off rather than an older revision number.
- **Cited specs:** pointers refer to the current approved specs and their latest working
  drafts in this correction round, under the glossary's convention. Confirm the versions
  handed off together, particularly the Q-0072 movement and approach pointers of claim 21.
- **Input timing wording satisfied, pending hand-off:** S-0201 Terms, *Cycle*, already
  cites claim 1 here for the at-most-one-input-event limit in an unfrozen cycle; its claim 2
  owns pacing. No additional S-0201 change is required. Confirm these passages and the
  glossary's compatible cycle definition against the versions handed off together, not an
  earlier revision list.
- **Tick boundary already agrees:** S-0203 claims 7 and 8 retain orders after the turn pass,
  production lines and house updates. Its preserved row labels remain compatible with S-0201's
  cited timing rules. No change to the order boundary or precedence here is needed.
- **Route cues already agree:** current S-0701 claim 24 and Scenario F8 allow another
  message-and-sound cue after a completed tick, matching claim 16 and O8 here.
  No further companion edit is required.
- **Behaviour coordination:** S-0215's cross-spec section lists matching movement,
  occupancy, search, placement and factory contracts in S-0207, S-0209, S-0307,
  S-0406, S-0417, S-0212 and S-0208, including the shared
  `quirks.teleport_vehicle_ground_reservation` and same-tick search/landing consequences.
  These remain owned by those specs and introduce no new order-application effect here.
  Confirm that suite together at hand-off; claim 16's route timing already matches.
- **Q-0286 joint answer already supplied:** confirm S-0214 claims 5 and 23, Scenario T17
  and its Import mapping together with claims 22 and 23 here. The request's current
  answered-by already lists S-0213 and S-0214; preserve both at hand-off.
- **Attack-move distinction already agrees:** S-0215 claims 5 and 12 distinguish
  attack-move orders from the standing behaviour *attack move*, preserve claim 6's
  translation, and qualify the data-only conclusion to the ordinary-input cases studied.
  The one companion edit is the S-0215 claim 5 entry above, for a building that holds a
  loop's changed point; confirm the matching passages together at sign-off.
- **Production-start ordering already agrees:** S-0404 claim 11/P11 B/C, S-0411
  claim 13/S9 and S-0202 claim 10 agree with claim 3 here: causing-action draw assignment,
  cancellation-order consequences and turn-pass starts before order-boundary starts.
  The between-house order-boundary extension remains a design rule, not an original
  computer-production measurement. Confirm these passages together at hand-off.
- **Check at sign-off:** S-0208, Terms and claim 11, define the scatter request's threat point, *forced*,
  *move anywhere* and *effectively forced* that claim 9 uses.
