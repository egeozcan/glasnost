---
id: S-0213
title: Human orders (the order set, when orders apply, and what each order changes)
topic: simulation
revision: 34
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0072, Q-0087, Q-0277, Q-0282, Q-0286, Q-0391, Q-0393, Q-0501, Q-0530]
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
  action) for the selected objects instead of ordering them at once (claim 16). An object that has
  a planned route, being built or being followed, is a *route holder*.
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
   selection history.
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
   (Terms), so when more than 128 objects are selected only the first 128 are ordered. The key commands of
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
     first is not)
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
   here for the building column or the neither-reference row; those cells define otherwise
   unreachable orders, not a claim that an eligibility change alone can create them.
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
     since the measurement setup cannot issue them. Applying the translation at the time of
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
     the run wins. The leader keeps the common cell.
   - **Destination priority.** For deciding which look cells count as *taken*, members nearer to the leader have priority; equal distances are resolved by run order. A cell assigned to a higher-priority member counts as taken when determining the destinations of lower-priority members. The leader keeps the common cell.
   - **Placement.** Each member's destination is determined from its *look*: the cells met along
     the line that starts at the centre of the common cell and runs in the direction from the
     leader's cell to the member's own cell, in steps of one cell length, up to a limit of a few
     steps (Formulas, *look line*). Each cell of a member's look belongs to the first class of this
     table that fits it, judged using the higher-priority members' assigned cells as *taken*.

     | Class | Cell of the look |
     |---|---|
     | *off-area* | Outside the usable area |
     | *other region* | Not in the same reach region (S-0212) as the common cell, for the member's movement zone |
     | *taken* | Already given to another member of the run |
     | *too steep* | 3 or more levels above or below the common cell (a bridge deck counted at its height) |
     | *obstructed* | The member's entry outcome (S-0209) for it is friendly obstacle, enemy obstacle or blocked |
     | *open* | Any other cell |

     The *end* of the look is its first cell that is off-area, too steep or open; a look may have
     no end within the step limit. The look is *spoiled* if an other-region or obstructed cell comes
     before its end, or anywhere in it if it has no end. The member's cell follows from its look:

     | The look | Cell the member gets |
     |---|---|
     | Not spoiled, and its end is open | The end cell |
     | Any other look | Its *fallback cell*: the last taken cell before the end (or in the whole look, if it has no end), or the common cell if there is no such taken cell |

     Whether a further search replaces the fallback after a spoiled look is not
     settled. So two members can end with the same cell.
   - **Shared cells.** When a member is placed, every other member that is not yet placed and
     stands in the same cell as it gets the same destination. Several infantry units of one cell
     therefore get one cell between them, and the walk-in rules of the movement specs place them
     inside it. Two points are not settled in the original (Open questions). Our engine's interim
     rule: a member that received a shared destination is not placed again, so it keeps that
     destination and has no look of its own; and the leader is never given a shared destination,
     since it is not placed. A member that stands in the leader's cell gets the common cell by its
     own look (interim rule below), so under the interim rule the result is the same whether the
     leader counts as "not yet placed" or not.

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
   - Interim rule for our engine, where the original is not settled (Open questions): each
     member's look has at most 6 steps of its own, whatever the looks of the members placed before
     it, the first one landing one cell length from the centre of the common cell; a spoiled look gives the fallback cell,
     with no further search; the direction is the zero vector when the member stands in the
     leader's cell, so every step tests the common cell. This is a design choice that measurement
     may revise.
   - Backing: `measurable` (Scenario O10, main part) for the existence of the spreading, the
     leader keeping the common cell and the outward direction of the other cells,
     `measurable` (Scenario O10, part b) for the shared-cell rule; `measurable` (Scenario O10,
     part c) for the leader tie-break (equal distances to the centre); `measurable` (Scenario
     O10, part d) for the look classes: that no member is given a cell of a class that the
     table refuses (off-area, other region, too steep, obstructed), that a look cut short by such a
     cell gives the common cell or a cell already given to another member, and the 3-level limit; and `measurable` (Scenario O10, part e) for the spreading of a run of attack-move orders.
     The exact step limit, the first step of the look and what replaces a spoiled look are recorded
     there as evidence for the Open questions, not tested. The following are `unmeasured`: the
     order of placement as such, because it shows only through which cells count as *taken* and the
     recorder sees only the final cells (part d records them as evidence); that the spreading
     tests no validity (a member off the map still takes part), because input cannot select an
     object that is off the map; and the run identity of attack-move orders, because one input event
     issues one kind of behaviour order (claim 1), so no input makes a run that mixes attack-move
     orders and plain ones. The mean of the positions (the centre) is not recorded directly; the
     leader that follows from it is (part c). The exact priority among overlapping look
     classes, the nearest-member destination priority and the last-taken-cell fallback
     are `unmeasured`: O10 d records final cells but does not isolate those alternatives
     with a setup having a unique predicted assignment. They remain low-confidence
     placement details, not claims that O10 d can independently confirm.
   - Confidence: medium that spreading happens, that only runs of identical consecutive orders with
     a cell destination are spread, and for the leader and the placement order; low for the
     placement details and for the shared-cell rule

8. **Stop order.** The order acts only on an actor that is live, on the map, not tethered, not
   under construction and not being sold (outside its construction state, for a building), and not
   in the unidentified state of Open questions (our engine: inside a tunnel, see below). Its
   effects all take place when it is applied, and every row whose condition holds applies:

   | The actor | Effect of the stop order |
   |---|---|
   | Any actor | Its target and destination both become nothing, and it breaks off any contact. Its behaviour is unchanged, except as the harvester row and the jumpjet reroute below say |
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
   movement point and commitment survive. This is not S-0502 claim 16's direct
   no-point instruction to the locomotor.

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
   outcomes; the no-cell destruction retains claim 17's medium confidence and Scenario J11.

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
    factory ignores the order. Otherwise the actor breaks off any contact, its target and
    destination become nothing and the behaviour *unload* becomes its queued behaviour. What
    *unload* does depends on the type: deploying into a building, deploying in place, letting
    passengers or garrison occupants out. These consequences belong to S-0215,
    S-0517 claims 1 to 8 (in-place vehicle deploying and deploy-fire vehicles), and the
    special-abilities specs.

    For a jumpjet, the destination clear has S-0502 claim 24's movement consequences,
    just as an effective stop clear does (claim 8), whenever it retains a movement
    commitment. Deploy therefore need not erase its movement point or halt its flight:
    a successful reroute supplies a replacement point, with the flight-state outcomes
    in claim 8's table. A suppressed unchanged vehicle assignment instead retains the
    preceding point and commitment (claim 21). For a landing-first vehicle hovering
    with a commitment, a different replacement horizontal point causes cruise before
    descent; an equal one lets the landing request produce descent. Touchdown uses
    the retained point. With no commitment, the clear does not create one, and the
    landing request alone does not wake idle hover. These are S-0502 claims 13 and 24's
    consequences, not an extra deploy movement instruction.
    - Backing for the jumpjet consequences: `measurable` (S-0502 Scenario J22),
      confidence: high for effective-clear dispatch, medium for exceptional aftermath.
      The uncommitted idle-hover starting case remains `unmeasured`, as in that claim:
      no ordinary stock order sequence establishing it is settled.
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
    - Quirk: `quirks.detonate_in_planter_name` (default: YR behaviour). With the switch off, the
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
    each selected object. When the key is released, the routes are committed and the objects start
    to follow them. Route points are created, committed and deleted by their own order kinds, which
    apply at the tick boundary of claim 1. When a unit falls back to rest while following a route,
    S-0215 claim 13 specifies when the order for the next route point takes effect. Issuing that
    action is not an immediate destination assignment within the fall-back.
    A house may have at most 128 route holders at once. A
    route-mode click is refused when the house's route holders plus
    the selected units and buildings that are not yet route holders would number more than 128, or when a
    selected object cannot take part in a route. A refused click adds no route point.
    The refusal gives a message and the game-wide refusal sound. Multiple route-mode refusal
    notices before the next tick completes share one message-and-sound cue, but completing a
    tick permits a new cue: a refused click made seconds later in the same running game is not
    silent because of an earlier refusal (S-0701 claim 24). Claim 25 specifies the message's
    imported text, lifetime and shared display capacity.
    - Backing: `measurable` (Scenario O8, including its repeated-refusal test)
    The same-tick duplicate-notice limit is `unmeasured`: no setup here creates multiple
    independent route refusals before one tick completes. The reset's observable consequence
    on later clicks is tested by O8.
    - Confidence: high for repeated cues after completed ticks; low for route execution.
      What a committed route does, and how a route point's action is carried out,
      have not been studied in detail (Open questions); medium for the limit of 128 route holders.

17. **Every order to an actor ends its route.** Any order that names an actor, apart from the
    route-mode orders themselves, removes that actor's planned route. This happens when the order
    is applied, for any actor that still exists, whether or not the order then passes the checks of
    its kind: a behaviour order whose target died in the same tick, a duplicate enter order that
    part (a) of table T3 ignores, and an order to an actor carried inside a transport all end the
    route. Whether an object whose route was removed this way still counts toward the house's limit
    of 128 route holders (claim 16) is not settled (Open questions).
    - Backing: `measurable` (Scenario O8, step 2) for the removal of the route by an order that
      takes effect, and (Scenario O13, part c) for an order that fails its target check. The other
      failing cases are `unmeasured`: an actor that fails its own checks would have to be a route
      holder whose state changes in the tick between click and application (the input rules of
      S-0214 test the same conditions at the click), and its route would have to be observable
      afterwards.
    - Confidence: medium

18. **Key commands.** The commands of table T2 turn one key press into orders for the selection.
    Each command uses S-0214's selection order (Terms) and issues at most one order
    for each selected object, under the conditions in the table. The pending-list limit
    therefore drops the last objects in that order, not necessarily those selected last.
    Keyboard-command feedback is owned by S-0701 claim 19 and Scenario D4: outside
    route mode, guard, stop and scatter request one command cue for a nonempty selection,
    even when no selected object qualifies for an order. An empty selection requests none.
    Failed order issuance suppresses the cue even if earlier objects already received orders.
    Route-mode guard requests no ordinary guard cue. Deploy's game-wide command cue and
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
    - Quirk: `quirks.cheer_memory_spans_session` (default: YR behaviour). With the switch off, the
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
    the jumpjet filters below can suppress a destination assignment without cancelling
    the order's other effects.

    | Order | Destination request at application, after its checks pass |
    |---|---|
    | Plain move | The order's destination, after any group spreading of claim 7 |
    | Attack-move with a destination | The same assignment as plain move after claim 6's translation |
    | Area guard to a mobile actor | The order's target as destination and guard spot, even when it is the actor's own cell (table T3, part e) |
    | Enter or capture | The order's destination, normally the object to enter or capture (table T3, part f; S-0214 table M). A duplicate enter ignored by table T3 part a makes no assignment |
    | Ordinary attack on an object | Target becomes the object, destination becomes nothing (table T3, part f). Being out of range does not make the order itself assign a chase destination |
    | Stop or deploy | Destination becomes nothing (claims 8 and 10); this cannot request destination-driven get-up |
    | Scatter | Claim 9 sends a request, not a move order. Only a destination assignment made by an accepted request is subject to the equality rule (S-0208 claims 11 to 13) |
    | Queued move or route point | Recording a future point is not destination assignment. When following it assigns a destination, S-0107's same rule applies; route following is distinct from a plain behaviour order (claims 16 and 17; S-0215 claims 13 and 14) |

    **Destination filters and clears for jumpjets.** S-0502 claims 23 and 24 qualify these
    requests, including table T3 parts (e) and (f):
    - Outside an ion storm, jumpjet-type infantry ignore a nonempty destination assignment
      whose cell equals their current cell, before any movement-point search. This is a
      test of the type's jumpjet flag, not merely its locomotor. It applies while grounded
      too: an ordinary own-cell order does not promise take-off, an assignment or get-up.
      It does not prevent the order's separate target, behaviour or guard-spot effects.
    - An ordinary jumpjet vehicle ignores an unchanged order-level destination, including
      empty-to-empty, unless its destination-refresh state requires reassignment
      (S-0502 Terms). A balloon-hover vehicle can first substitute its retained destination
      while it still has a target. A suppressed assignment neither reroutes nor erases
      the movement point nor consumes an infantry standing-position draw.
    - Every effective destination clear reroutes a committed jumpjet under S-0502 claims
      17 and 24, whether caused by stop, deploy or behaviour-order application. The
      order-level destination is empty after rerouting, but the replacement movement
      point and commitment survive. Changing behaviour alone is not a destination clear.
      Without a commitment, the clear has no locomotor effect. Claim 8 gives the
      successful reroute's flight-state consequences; failure has S-0502 claim 17's outcome.

    These filters and clear outcomes inherit S-0502 claims 23 and 24's backing and
    confidence: `measurable` (J2 and J16 for own-cell infantry rejection; J19 and J22
    for effective clears and ordinary vehicle suppression), high for the filters and
    effective-clear dispatch. Destination-refresh activation and balloon-hover target
    substitution remain `unmeasured` under J22's diagnostic: normal landing-first
    vehicle runs do not isolate them. Exceptional aftermath retains medium confidence.

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
    - Backing: `measurable` (Scenario O19, part a)
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
    - Quirk: `quirks.blocked_arrow_events` (default: YR behaviour). With it off,
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
    caused by too many route holders displays the route-holder-limit text; a selection
    that fails the route eligibility check displays the incompatible-selection text.
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

> A route-mode click is refused by the route-holder limit exactly when \(H+S>128\), where \(H\) is the number of the house's route holders and \(S\) is the number of selected units and buildings that are not route holders. These are exact integer counts. (Claim 16, which also refuses a click when a selected object cannot take part in a route.)

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
> *v* = (0, 0) (our engine's interim rule, claim 7). With the common cell *k* = (*k*ₓ, *k*ᵧ), the look's *j*-th point is
> *q*ⱼ = (*k*ₓ + ½ + *j*·*u*ₓ, *k*ᵧ + ½ + *j*·*u*ᵧ), in cell lengths, floating point, and its cell is
> (trunc(*q*ⱼ,ₓ), trunc(*q*ⱼ,ᵧ)), each conversion truncating toward zero in every rounding mode
> (S-0205, claim 7). The look is the cells of *j* = *j*₀, *j*₀ + 1, … up to the step limit. In the
> original the first index *j*₀ and the limit are not settled (Open questions); our engine's
> interim rule is *j* = 1 to 6 (claim 7). Confidence: low, as for the other placement details.

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
| Route-holder limit per house | 128 | objects | Claim 16 |
| Order delay | 1 | ticks | Claim 1: issued in tick *N*, first acted on in tick *N* + 1 |
| Input events taken per unfrozen cycle | at most 1 | events | Claim 1; frozen cycles take none |
| Smallest spread group | 2 | orders | Claim 7 |
| Largest level difference of a spread cell | 2 | levels | Claim 7: fewer than 3 levels from the common cell |
| Steps of a spread look (our engine, interim) | 6 | steps of one cell length | Claim 7, design choice |

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
| Route point add, commit, delete | route data | (claim 16) | Route mode (claim 16) |

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

While route mode is held, the guard command adds a guard action to the planned routes instead of
issuing orders.

**The deploy rule.** If the first object in selection order is a building of the player's that holds
garrison occupants, only that building gets a deploy order. Otherwise each selected object gets a
deploy order if it is a unit or building, is movable or able to fire, is not berserk, and passes
both the deploy check and deploy readiness of S-0214. Deploy readiness is never met by a
building, so apart from the garrison case the command orders units only. There is one exception,
for in-place deployers (S-0214, Terms): if the selection holds both in-place deployers that are
deployed and in-place deployers that are not, the deployed ones get no order. So one press deploys
the rest of a mixed group instead of undeploying the deployed members.

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

- **Repeated-destination get-up** (claim 21): S-0107 claim 15 owns
  `quirks.prone_get_up_repeated_destination`, default YR behaviour. With it off, every non-empty
  destination assignment requests get-up for an otherwise eligible prone unit, not only a
  reassignment of the same destination. This applies across order families, not just move.
- **Blocked arrow events** (claim 23): `quirks.blocked_arrow_events`, default YR
  behaviour. Discarded releases can leave scrolling active after the key is let go.
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
| No rules, art or map key | route refusal duration and shared status display (claim 25) | fixed presentation rules | 480 clock units of 16 milliseconds; six lines |
| Game font file `GAME.FNT` | status-text glyph mapping, masks, widths, bitmap height and line pitch (claim 25) | unsigned 16-bit text-unit mapping; zero means missing, positive values select zero-based glyph number value minus one; widths in integer pixels, binary masks; resolved through S-0005 claims 1 to 4 and 18a | no original status text without a loaded font; our engine reports a missing-asset diagnostic naming the file and reason and omits the text for missing files or whole-font rejection under S-0014 claim 6; no partial use or substitute font; binary layout and masks are S-0014 claims 1–3, fallback image and transparency are claims 4 and 5 |
| Common bitmap height in `GAME.FNT` | stored glyph row count H (claim 25) | unsigned integer pixels, S-0014 claim 1; row zero starts at the line origin | no independent absent-field default; incomplete files are rejected under S-0014 claim 6 |
| Header line pitch in `GAME.FNT` | stored status line's clipping height P (claim 25) | unsigned integer pixels, S-0014 claim 1; does not set the fixed 19-pixel origin spacing; our engine paints only rows below min(H, P) | no independent absent-field default; incomplete files are rejected under S-0014 claim 6 |
| Mapped capital X in `GAME.FNT` | generated fallback shape and width (claim 25) | text unit 88, complemented mask with the same width and bitmap height | no reserved fallback entry; an absent X mapping rejects the entire font under S-0014 claim 6; the original's mapped-text case without X is claim 4 |
| Rules `[Colors]` | route-notice foreground (claim 25) | local player's chosen definition, parsed by S-0103 claims 4 and 5; unmodified HSV converted and packed by claims 6 and 9 | colour-definition import defaults remain S-0103's; no font palette or battlefield lighting |
| No rules, art or map key | status-text splitting (claim 25) | fixed integer-pixel budgets, 111-character limit, one-pixel character spacing and 64-pixel tab stops | claim 25's initial and resized budgets and separator rules |
| No rules, art or map key | acceptance of arrow events during an order block (claim 23) | fixed input rule, subject to the presentation quirk | presses and releases made during the block discarded |
| Cursor image and palette of S-0214 Import mapping | edge-scroll images of table E | same imported image and palette, full-size frames | same missing-file handling as S-0214 |
| Player's keyboard preferences, set through the key-binding screen | command keys of table T2 (claim 22) | assigned keyboard key or key combination; preferences, not rules-layer fields | no user override: stop S, scatter X, guard G, deploy D, cheer C, route mode Z; a removed entry in a successfully loaded preferences file: unbound |
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
J2, J16, J19 and J22, with their setup and recorder requirements. In particular, compare
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
- *Setup:* the preamble's flat stock-data skirmish map, speed 6, director and mods off.
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

**Scenario O8: route mode (claims 8, 16 and 17).**
- *Setup:* game A: one vehicle of the player's on open ground and 128 infantry of the player's,
  all selectable. Game B, a fresh game: 130 infantry of the player's, all selectable, and no other
  unit of the player's. Game C: as game A, with the vehicle only.
- *Procedure:* game A: (1) select the vehicle, hold the route-mode key, click 3 cells, release.
  (2) While it follows the route, select it and give a plain move click. (4) After the vehicle has
  arrived, select the 128 infantry, hold the route-mode key and click one cell. Game B: (3) select
  128 infantry, hold the route-mode key, click one cell, keep holding, add 2 more infantry to the
  selection and click again. Wait several completed ticks, then repeat the refused click twice,
  several seconds apart, without changing the selection. Game C: (5) as step 1, then, while the vehicle follows the route and
  before it reaches the first cell, select it and press the stop key.
- *Runs:* 3 of each game. Route mode involves no random draw.
- *Observable:* whether the vehicle moves before the key is released; the order in which it
  visits the cells; whether it continues the route after (2) and after (5); whether the clicks of (3) and (4)
  are refused (message and sound), including video and audio of each repeated refusal in game B.
- *Expected:* each game-B refusal separated by completed ticks gives a message and refusal
  sound, with no new route point. No once-per-game silence is expected. No movement before release, then the cells in click order; after (2) it goes to the
  new cell and never returns to the route; after (5) it comes to rest and does not go on to any
  cell of the route; in game B the first click of (3) is accepted and the
  second is refused, because 128 route holders plus 2 new ones exceed 128. Step 4 has no expected
  value: it answers the open question of claim 17. If its click is refused, the vehicle whose route
  was removed still counts as a route holder (1 + 128 > 128); if it is accepted, it no longer
  counts.
- *Falsified if:* the vehicle moves before release, it resumes the route after (2) or after (5), or in game B
  the refusal comes at another count, or a repeated refusal after completed ticks loses its
  message or sound solely because an earlier refusal was shown. The repeated-click test
  distinguishes the claim from a once-per-game limit; it does not isolate same-tick duplicate
  notices, for which normal input supplies at most one event per cycle (claim 1).

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
  that the click gives all of them the same destination cell before the spreading.
- *Procedure:* one move click to an empty cell 10 cells away. Repeat 5 times from the same start.
- *Observable:* the destination cell of each vehicle after the click, relative to the clicked cell
  and to the vehicle's place in the block.
- *Expected:* the destinations are spread around the clicked cell; the centre vehicle of the block
  (the leader) gets the clicked cell; every other vehicle's cell lies on the side of the clicked
  cell that matches its place in the block (the map north-west vehicle map north-west of it, and so on),
  within a few cells. On open ground all 9 cells are expected to differ.
- *Falsified if:* all 9 vehicles keep the clicked cell (no spreading), the centre vehicle does not
  get the clicked cell, or a vehicle's cell lies on the opposite side of the clicked cell from its
  place in the block. Two vehicles sharing a cell do not falsify the medium-confidence parts (the
  fallback of claim 7 allows it); any such case is recorded as evidence on the look's open
  details.
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
  on top of it may be given. Falsified if any vehicle gets a cell of a refused class. Whether a
  vehicle whose look is spoiled gets a different open cell beyond the obstacle (a further
  search), the largest number of steps a look takes, and which cell a vehicle that
  stands in the leader's own cell gets are recorded as evidence for the Open questions and falsify
  nothing.
- *Part (e), attack-move group:* the 9 vehicles of the main part, each of an attack-move-eligible
  type, one attack-move click to the same cell, 3 runs. Expected: the destinations are spread as in
  the main part. This shows that a run of attack-move orders spreads like one of plain orders.
  Falsified if all 9 vehicles keep the clicked cell.

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
- *Setup:* stock data, director and mods off, game speed 6, on the flat skirmish map of
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
- *Setup:* the preamble's flat skirmish map, game speed 6, director and mods off. Produce
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
  area and the bystander isolated as in the preamble. Stock data, director and mods off,
  speed 2, 800 × 600. Pre-place a selected armed vehicle, an infantry unit and a deploy-capable
  unit of stock types. For scrolling use a map large enough to keep the view away from its
  limits. Record the key-binding screen and a video, with the tick and order recorder of
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
- *Runs and tool:* three runs per part and keyboard-preference variant; manual input or
  scripted normal input with a read-only recorder. These are proposals, not recorded measurements.

**Scenario O20: edge-scroll presentation (claim 24, table E).**
- *Setup:* a large flat stock-data skirmish test map at 800 × 600, speed 6,
  director and mods off, with no selected units and the view away from all scroll limits.
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
- *Setup:* O8 game B, stock data, director and mods off, at speeds 1 and 6.
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
  afterwards. Director and mods remain off except these named test assets.
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
  and its void rule, director and unrelated mods off. Use named valid test font,
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
- *Setup:* O21's skirmish fixture and void rule, speed 6, 800 × 600, director and
  unrelated mods off. Named test assets are a string table with short invented
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
  Director and mods off. Use S-0201 claim 11's map-triggered chrono flash late
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
- Whether an object whose planned route was removed by an order (claim 17) still counts toward
  the house's 128 route holders of claim 16.
- The full behaviour of route mode: how a route point's action is carried out, looping routes,
  and deleting route points; and which selected objects "cannot take part in a route" (claim 16).
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
- Claim 7: how many steps a member's look takes (the limit of the look line in Formulas), whether
  that limit applies to each member's look separately or to the run as a whole, what a member gets
  after a spoiled look, which direction the look of a member standing in the leader's own cell
  takes, and the first
  index *j*₀ of the look line, that is, whether the first step tests the common cell itself,
  whether a member that received a shared destination is still placed with a look of its own, and
  whether the leader counts as a member that is not yet placed when destinations are shared.
  Our engine's interim rule is stated in claim 7.

## Questions for the project lead

No decision is pending. The project lead's decisions of 2026-10-09 (AI houses,
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
  is implied. Confirm the current companion text at sign-off rather than an older revision number.
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
  No companion edit is required; confirm the matching passages together at sign-off.
- **Production-start ordering already agrees:** S-0404 claim 11/P11 B/C, S-0411
  claim 13/S9 and S-0202 claim 10 agree with claim 3 here: causing-action draw assignment,
  cancellation-order consequences and turn-pass starts before order-boundary starts.
  The between-house order-boundary extension remains a design rule, not an original
  computer-production measurement. Confirm these passages together at hand-off.
- **Check at sign-off:** S-0208, Terms and claim 11, define the scatter request's threat point, *forced*,
  *move anywhere* and *effectively forced* that claim 9 uses.
