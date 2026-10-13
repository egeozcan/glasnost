---
id: S-0213
title: Human orders (the order set, when orders apply, and what each order changes)
topic: simulation
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0072, Q-0087]
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
are used as S-0214 (Terms) defines them; *garrison occupants* are the infantry inside a garrisoned
building. Whether a permanent takeover that leaves no controlling object counts as
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
  issues an order (claim 3); it has the same pending-order limit (claim 2) and the same checks when
  its orders are applied (claims 4 and 15), with no exemption; it counts as human-controlled
  wherever this spec tests that (claims 9, 11, 12 and 20); and it has no teams (table T3, part c).
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
  lose when that manoeuvre ends or the link is broken off. Which situations of play send these
  notices is open (Open questions). While tethered, the unit ignores stop, scatter and deploy
  orders (table T1), and a behaviour order keeps the link, except at a building of the unloading kind (table T3, part b).
- **Inside a tunnel.** The state of a mobile actor that is travelling through a tunnel, a map
  feature whose movement belongs to the movement specs.
- **Usable area.** As S-0211 (Terms) defines it: the part of the map inside the local area that
  objects may occupy. It is the area to which S-0214 limits clicks.
- **Route mode, planned route, route holder.** Route mode is the input mode, entered by holding
  the route-mode key, in which clicks build a *planned route* (a list of places, each with an
  action) for the selected objects instead of ordering them at once (claim 16). An object that has
  a planned route, being built or being followed, is a *route holder*.
- **Selection.** The local player's set of selected objects, in the order they were selected.
  Selection is an input-side notion and is not game state.
- **Pending list.** The list in which orders wait between being issued and being applied
  (claim 2).
- **Attack-move memory.** What an attack-move order leaves on its actor: that the actor's *move*
  or *attack* is an attack move, and the destination or target it is attack-moving to (claim 6).
  What an actor with the memory does belongs to S-0215 and the combat specs.
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
- **Cycle, order block.** As S-0201 (Terms) defines them: a cycle is the stretch of real time in
  which the game takes the player's input once and runs one tick; during an order block the game
  accepts no orders from the player.
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
   - **Which tick an input belongs to.** In a single-player game the player's input is taken once
     per cycle, before that cycle's tick (S-0201, *cycle*, claim 2), and not while the game waits
     between ticks. Each cycle takes at most one input event from the player's waiting inputs, in
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
   that orders a selection issues one order per selected object, in selection order (S-0214), so
   when more than 128 objects are selected only the first 128 are ordered. The key commands of
   table T2 are limited in the same way: once the list is full, every further order of the
   command is dropped. Since a cycle takes one input event (claim 1), the orders that the player's
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

3. **Order of application within a tick.** For one house, non-behaviour orders have issue-order precedence among themselves, and behaviour orders have issue-order precedence among themselves. When a non-behaviour order and a behaviour order conflict for the same actor in one tick, the resulting state is the one produced by the behaviour order, regardless of which was issued later. Thus a stop issued after a move to the same actor in that tick leaves the move in force. A person's input in the original cannot produce that conflict, because each tick's orders come from one input event (claim 1); it can arise through our engine's AI houses, whose orders follow the same precedence rule.

   If orders from different houses in one tick affect the same state, the resulting state in our engine is the one given by resolving the houses in their creation order (S-0009 owns that order). In the original this order between houses has no consequence that we know of, because each house's orders act only on its own objects and on its own house state; our engine fixes the outcome for determinism, not as a claim about the original.
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
     observe it. The drop of a computer house's orders is `measurable` (Scenario O11). The order
     between houses and the rules for AI houses are design rules of our engine, not claims about
     the original, so they need no backing.
   - Confidence: medium (the grouping is clear; that nothing else reorders orders of the same
     house is not certain)

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
     in that state at a tick boundary.
   - Confidence: high for the actor checks of table T1; medium for the target and destination

5. **Behaviour order.** A behaviour order names a behaviour, and optionally a target and a
   destination. Its effect on a valid actor whose target and destination pass claim 4 is given by
   table T3 in State and ordering. If part (a) of that table holds, the order is ignored (it still
   ends the actor's planned route, claim 17) and parts (b) to (h) do not apply; otherwise each part from (b) to (h) whose condition holds
   applies. An attack-move order has the effect of the plain order that claim 6 makes of it, and
   "the resulting behaviour" in table T3 means that order's behaviour. Every destination
   assignment uses S-0107 claim 15's repeated-current-nonempty-destination get-up rule
   (claim 21 here), regardless of the behaviour named by the order.
   - Backing: `measurable` (Scenario O3) for parts d, e, f (step 7 for its passenger clause) and
     the guard-spot part of g, and `measurable` (Scenario O4) for parts a and h and for the
     contact break of part b at a building of the unloading kind (Scenario O4, part iii, with a
     named test rules file). The rest of part b is `unmeasured` for now. Its two plain branches,
     that an actor that is not tethered breaks off contact and that a tethered actor keeps its
     link at a building of another kind, can only be told apart with an exchange known to tether
     or known not to, and none is identified (Open questions; Scenario O4, part iv, records the
     refinery case for that question only); and forgetting the building the actor was
     waiting in line to enter changes nothing that the recorder reads until a later docking
     request, whose rules belong to the exchange specs. Part c is `unmeasured`: it needs a team of
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
   both a target and a destination has not been checked). The other cells arise only from a
   change of the actor's type eligibility between issue and application; an AI house is held to
   the same input rules (claim 3), so it cannot issue them directly either. The table still
   defines them, so that every order has a result. An attack-move order to a mobile actor whose type is not eligible leaves any attack-move memory it already had unchanged.
   - Backing: `measurable` (Scenario O3, steps 5a and 5b) for the two eligible rows that input
     creates: the recorded behaviour of an attack-moving unit is *move* with the cell as its
     destination (destination only) and *attack* with the object as its target (target only). What
     the attack-move memory holds is not recorded by the measurement setup, so the "memory holds"
     parts of the table and the removal of the memory by another behaviour order are `unmeasured`
     (S-0215 owns what the memory changes). The rows that input cannot create are `unmeasured`,
     since the measurement setup cannot issue them. Applying the translation at the time of
     application, not of issue, is `unmeasured` for the same reason.
   - Confidence: high for the translation; medium for what the memory changes afterwards (S-0215)

7. **Group destinations.** When, within one house's behaviour orders of one tick, two or more
   orders in a row name the same behaviour, the same target and the same destination, and that
   destination is a cell (the *common cell*), the destinations of that run are spread, so that the
   group does not try to stand in one cell. Orders are compared as they were issued, by the
   behaviour they name, before claim 6 turns an attack-move order into a plain one: an attack-move
   order and a plain move order to the same cell are different in this comparison and never form
   one run. A *run* is maximal: it takes in every such order in
   a row, so the order before it and the order after it, if any, differ from it in behaviour,
   target or destination. Only the mobile actors of the run that are not inside a
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
   covers only the floating-point operations before it. The members and cells are judged in the
   state left by every order that claim 3 places before this house's behaviour orders at the same
   boundary, and by none of this house's behaviour orders; so a building placed by an order of the
   same tick already counts. Runs arise when the destinations that a click gives the selected
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
     leader that follows from it is (part c).
   - Confidence: medium that spreading happens, that only runs of identical consecutive orders with
     a cell destination are spread, and for the leader and the placement order; low for the
     placement details and for the shared-cell rule

8. **Stop order.** The order acts only on an actor that is live, on the map, not tethered, not
   under construction and not being sold (outside its construction state, for a building), and not
   in the unidentified state of Open questions (our engine: inside a tunnel, see below). Its
   effects all take place when it is applied, and every row whose condition holds applies:

   | The actor | Effect of the stop order |
   |---|---|
   | Any actor | Its target and destination both become nothing, and it breaks off any contact. Its behaviour is unchanged, except as the harvester row says |
   | A mobile actor | It has no queued destinations left (claim 5: how they arise is S-0215's) and stops following any planned route. With its destination gone it comes to rest as the movement specs describe for a unit whose destination is removed (this spec does not claim that it halts within the same tick) |
   | An actor with spawned aircraft (such as a carrier) | Its spawned aircraft are called off their target |
   | An open-topped transport | Its passengers lose their targets too |
   | A harvester whose behaviour is *harvest* or *return* | It gets the behaviour *guard* at once |
   | A vehicle that controls slave units, whatever its behaviour | Its slaves are sent back to guard it |

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
     route. The following are `unmeasured`: (i) the tethered exception, because a scenario needs a
     unit that is known to be tethered, and the exchanges that tether a unit are not identified
     (Open questions); once one is identified (Scenario O4, part iv, may identify docking at a
     refinery), a stop order given during it can be observed. (ii) The slave rule "whatever its
     behaviour", because to have the slaves out mining while the miner's behaviour is neither
     *harvest* nor *return*, the miner must first get a behaviour order, which already calls the
     slaves back (table T3, part h). (iii) "No queued destinations left", as in claim 5. (iv) "Breaks
     off any contact", because the exchanges that a stop order may find an actor in are either
     tethered or not identified as untethered (the same gap as in table T3, part b), so no setup
     separates it from the actor's own end of an exchange. (v) The checks "not under construction"
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
   - Backing: `measurable` (Scenario O5, step 4)
   - Confidence: medium

10. **Deploy order.** The actor must be live, on the map, not tethered, not an aircraft, not
    paralysed, not under construction or being sold, and not in the unidentified state of Open
    questions (our engine's interim rule: claim 8). An actor that stands on a cell occupied by a building whose type is a vehicle
    factory ignores the order. Otherwise the actor breaks off any contact, its target and
    destination become nothing and the behaviour *unload* becomes its queued behaviour. What
    *unload* does depends on the type: deploying into a building, deploying in place, letting
    passengers or garrison occupants out. It belongs to S-0215 and the special-abilities specs.
    - Backing: `measurable` (Scenario O6) for the vehicle-factory cell and for the unit that
      deploys in the way its type says (which shows that *unload* was queued and took effect). That
      the actor also breaks off contact and loses its target and destination is `unmeasured`: the
      recorder would have to catch a deploying unit in contact or with a target, and no setup
      identified here does. The refusals for an aircraft, a paralysed actor, an actor under
      construction or being sold, and a tethered actor are `unmeasured`: input may already refuse
      the order for such an actor (S-0214), or the actor would have to enter that state between
      the click and the application, which the scenarios cannot hold steady, and the tethered
      state is open (claim 8).
    - Confidence: medium

11. **Detonate order.** If the actor exists and carries a planted bomb, the bomb goes off at once.
    No other check is made. The order is issued in the name of the planter's house, not the house
    of the player who gave it. By claim 3 it is therefore applied only when the planter's house is
    human-controlled: a player who sets off a bomb that a computer house planted on one of the
    player's own units gets no effect, and the order is dropped.
    - **In our engine (departure).** An AI house counts as human-controlled for the checking of
      orders (Terms, *AI house*), so a detonate order issued in an AI planter's name is applied: a
      player can set off a bomb that an AI house planted on the player's unit. The drop of Scenario
      O11 is the original's behaviour against its computer player, which our engine does not
      reproduce under either setting of the switch below. This is a *departure* with no switch,
      decided by the project lead (2026-10-09): AI houses follow the order rules of human houses.
    - Backing: `measurable` (Scenario O6, step 4, and Scenario O11)
    - Confidence: medium
    - Quirk: `quirks.detonate_in_planter_name` (default: YR behaviour). With the switch off, the
      order is issued and applied in the name of the house of the player who gave it. Input offers
      the detonate action only on the player's own object (S-0214), so that house is always the
      owner of the object that carries the bomb. In our engine the switch decides which house's
      pending list (claim 2) the order fills and in which house's turn of claim 3 it is applied,
      and the bomb goes off under either setting.

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
    - The *primary* order needs a live building, which may be off the map, whose type is a
      factory. It is a switch: if the building is already its house's primary factory, it stops
      being primary; otherwise it becomes primary, and every other building of the same house
      that is on the map, is a factory of the same kind and has the same water-or-land kind of type
      (naval or not) stops being primary. A building whose type is not a factory is not changed.
      Input creates the order only when the player clicks a selected factory building of the
      player's on itself (the deploy click of S-0214, row G5) and that factory is not already
      primary: a click on a factory that is already primary gives no action (S-0214, row B3). So the
      switching-off case arises only when a second primary order for the same factory is issued
      before the first is applied: a person's input cannot do that (claim 1), but an AI house can,
      by issuing two in one tick. The local player hears a notice when one
      of the player's buildings becomes primary.

    None of these checks the owner of the building when it is applied (see claim 15). What
    repairing, powering and the primary factory change belongs to the economy and production specs.
    - Backing: `measurable` (Scenario O6, step 3, for repair; Scenario O12 for power, the
      firestorm-wall flag and making a factory primary). "Off the map" for repair and primary, and
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
    apply at the tick boundary of claim 1. A house may have at most 128 route holders at once. A
    route-mode click is refused, with a message and a sound, when the house's route holders plus
    the selected units and buildings that are not yet route holders would number more than 128, or when a
    selected object cannot take part in a route. A refused click adds no route point.
    - Backing: `measurable` (Scenario O8)
    - Confidence: low. What a committed route does, and how a route point's action is carried out,
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
    Each command looks at every selected object in selection order and issues at most one order
    for each, under the conditions in the table.
    - Backing: `measurable` (Scenarios O5 and O6)
    - Confidence: medium (high for stop and scatter)

19. **Cheer rate limit.** The cheer command is accepted only if the current tick is more than
    *R* ticks after the last accepted cheer, where *R* is the general rules' maximum-cheer-rate
    field. An accepted press records the current tick as the last accepted cheer even if its order
    is then dropped by claim 2. The last-cheer tick is remembered across the games of one session.
    At program start it is −100, and a game start does not reset it. So in the first game of a
    session the first cheer is accepted only after tick *R* − 100. In a later game no cheer is
    accepted until the tick counter passes the tick of the last cheer of an earlier game plus *R*.
    - Backing: `measurable` (Scenario O7)
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
    The following are the destination assignments made by the orders of this spec.

    | Order | Assignment at application, after its checks pass |
    |---|---|
    | Plain move | The order's destination, after any group spreading of claim 7 |
    | Attack-move with a destination | The same assignment as plain move after claim 6's translation |
    | Area guard to a mobile actor | The order's target as destination and guard spot, even when it is the actor's own cell (table T3, part e) |
    | Enter or capture | The order's destination, normally the object to enter or capture (table T3, part f; S-0214 table M). A duplicate enter ignored by table T3 part a makes no assignment |
    | Ordinary attack on an object | Target becomes the object, destination becomes nothing (table T3, part f). Being out of range does not make the order itself assign a chase destination |
    | Stop or deploy | Destination becomes nothing (claims 8 and 10); this cannot request destination-driven get-up |
    | Scatter | Claim 9 sends a request, not a move order. Only a destination assignment made by an accepted request is subject to the equality rule (S-0208 claims 11 to 13) |
    | Queued move or route point | Recording a future point is not destination assignment. When following it assigns a destination, S-0107's same rule applies; route following is distinct from a plain behaviour order (claims 16 and 17; S-0215 claims 13 and 14) |

    A qualifying reassignment at order application requests get-up at that boundary, before
    any movement in the next tick (claim 1), not at locomotor start. Merely moving to the cell
    the unit occupies does not qualify when it had no destination; repeating that cell while
    it is still its destination does. A refused sequence request is not saved until movement
    starts. Later destination assignments during behaviour processing or movement can request
    get-up on a later tick; S-0215 and the movement specs own when those assignments occur.
    No order-family whitelist is imported.

    **Attack order versus approach (Q-0072).** For an ordinary armed infantry unit or vehicle,
    an attack order with an object target and no destination leaves that target and an empty
    destination at the order boundary. This does not mean the actor remains stationary:
    S-0215 claim 37 owns the subsequent approach step's trigger, gating and destination
    renewal; S-0307 claim 15 owns the approach-position search within its stated scope.
    Stopping belongs to S-0207 and S-0208 and remains open where those specs do not state it.
    None of those is an additional effect of the order here. Buildings likewise receive
    the target without a movement destination and never take that approach step
    (S-0215 claims 32 and 37 g). Special infantry actions that use entering,
    capture or sabotage are not this ordinary-attack case (S-0214 table M).
    - Backing: `measurable` (Scenario O17, Scenario O3 for the attack-order references, and
      S-0107 Scenario M4 for equality, type exceptions and sequence refusal). Later approach
      and route scheduling are claims of their owning specs, not new timing claims here.
    - Confidence: high for destination equality and the assignments of table T3; medium for
      their observable timing through input and for route following, as in claims 1 and 16

## Formulas

> cheer passes the rate limit ⇔ *t* > *L* + *R*. *t* is the current tick, *L* the last accepted
> cheer tick (−100 at program start) and *R* the maximum-cheer-rate field. All are integers in
> ticks; the sum and the comparison are exact integer operations. (Claim 19. Open questions name
> one further cause for which a cheer is refused.)

> route-mode click refused by the route-holder limit ⇔ *H* + *S* > 128, where *H* is the number of
> the house's route holders and *S* the number of selected units and buildings that are not route
> holders. Integers, exact. (Claim 16, which also refuses a click when a selected object cannot
> take part in a route.)

> Group destinations (claim 7). For the *n* members with positions *p*ᵢ = (*x*ᵢ, *y*ᵢ, *z*ᵢ) in
> leptons: centre *c* = (trunc(Σ*x*ᵢ / *n*), trunc(Σ*y*ᵢ / *n*), trunc(Σ*z*ᵢ / *n*)), integer sums
> and division truncated toward zero. *d*(*a*, *b*) = trunc(√((*a*ₓ − *b*ₓ)² + (*a*ᵧ − *b*ᵧ)² +
> (*a*_z − *b*_z)²)), the sum of squares and the root in floating point, the root being the
> approximate square root of S-0204, and the conversion to an integer truncating toward zero in
> every rounding mode (S-0205, claim 7). Leader *L* = the member with
> the smallest *d*(*p*ᵢ, *c*), the latest in the run on equal values. The other members are placed in
> increasing order of *d*(*p*ᵢ, *p*_L), ties in run order.

> Look line (claim 7). Cells are written as integer pairs (*X*, *Y*) of cell coordinates. For a
> member in cell *m* and the leader in cell *ℓ*, let *v* = *m* − *ℓ* (integers) and *u* = *v* /
> √(*v*ₓ² + *v*ᵧ²), in floating point with the approximate square root of S-0204; *u* = (0, 0) when
> *v* = (0, 0) (our engine's interim rule, claim 7). With the common cell *k* = (*k*ₓ, *k*ᵧ), the look's *j*-th point is
> *q*ⱼ = (*k*ₓ + ½ + *j*·*u*ₓ, *k*ᵧ + ½ + *j*·*u*ᵧ), in cell lengths, floating point, and its cell is
> (trunc(*q*ⱼ,ₓ), trunc(*q*ⱼ,ᵧ)), each conversion truncating toward zero in every rounding mode
> (S-0205, claim 7). The look is the cells of *j* = *j*₀, *j*₀ + 1, … up to the step limit. In the
> original the first index *j*₀ and the limit are not settled (Open questions); our engine's
> interim rule is *j* = 1 to 6 (claim 7). Confidence: low, as for the other placement details.

No other formula: apart from claim 7, the orders change references and behaviours, not
quantities.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Pending-list capacity | 128 | orders | Claim 2 |
| Initial last-cheer tick | −100 | ticks | Claim 19 |
| Route-holder limit per house | 128 | objects | Claim 16 |
| Order delay | 1 | ticks | Claim 1: issued in tick *N*, first acted on in tick *N* + 1 |
| Input events taken per cycle | 1 | events | Claim 1 |
| Smallest spread group | 2 | orders | Claim 7 |
| Largest level difference of a spread cell | 2 | levels | Claim 7: fewer than 3 levels from the common cell |
| Steps of a spread look (our engine, interim) | 6 | steps of one cell length | Claim 7, design choice |

## State and ordering

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
| Primary | actor (a building) | live (it may be off the map); its type is a factory | Primary factory switched (claim 13) |
| Place | building type, cell | (production specs) | A finished building is placed |
| Build, hold, cancel, cancel all | type | (production specs) | Production started, held or cancelled |
| Fire superweapon | superweapon, cell | (superweapon specs) | The superweapon fires at the cell |
| Alliance | other house B | both multiplayer alliance flags on (S-0414, claim 14); when issuer A does not yet ally B: B is not A, B's country is not of the civilian side, A is undefeated, and the last-enemy test of S-0414 claim 12 passes (with exactly one enemy contender, A can ally nobody); when A already allies B: neither house is passive (S-0414, claim 13) | The issuer allies the other house one way, or the alliance ends in both directions (S-0414, claims 12 and 13) |
| Surrender | none | issuing house undefeated and with no result pending (S-0414, claim 11) | The issuer's objects are destroyed in its next house update, and the house is defeated when its defeat test then fails, which may be later or never (S-0414, claims 5 and 11); the game ends only through S-0414 claim 6 (play to the last side; a defeated person keeps watching); legal for every player house, human or AI |
| Game speed | speed index | none | Game speed changed; network games only (claim 14; S-0201, claims 8 and 13) |
| Cheer | none | none | All of the issuer's infantry cheer |
| Route point add, commit, delete | route data | (claim 16) | Route mode (claim 16) |

**Table T2: key commands.** The command keys are the player's own key bindings. "Selected object"
means each object of the selection in turn. *Movable*, *able to fire*, the *deploy check* and
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

**The deploy rule.** If the first selected object is a building of the player's that holds
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
| (h) | The actor controls slave units (a slave-miner harvester, which works with its slaves) and the resulting behaviour is not *attack* | Its slaves are sent back to guard it |

Destination assignments in table T3 parts (e) and (f), including assignments that repeat the
current destination, use claim 21 and S-0107 claim 15. They must not be interpreted as an
unconditional get-up effect of a move behaviour. Scatter and route following use the same rule
only when they actually assign a destination.

**Ordering consequences.**
- Two orders of the same kind for the same actor issued in the same tick both take effect, with
  the result of applying them in issue order: the later one wins wherever both change the same
  thing. When a behaviour order and an order of another kind for the same actor are issued in the
  same tick, the result is that of the other order followed by the behaviour order, whichever was
  issued first (claim 3).
- An order is applied after the tick's object updates, so an object destroyed during tick *N*
  is no longer live when an order issued in tick *N* is applied, and every order kind that needs
  its actor, target or destination to be live or valid has no effect (claim 4, table T1).

## Edge cases and quirks

- **Repeated-destination get-up** (claim 21): S-0107 claim 15 owns
  `quirks.prone_get_up_repeated_destination`, default YR behaviour. With it off, every non-empty
  destination assignment requests get-up for an otherwise eligible prone unit, not only a
  reassignment of the same destination. This applies across order families, not just move.
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
- **The click marker.** When a selection is ordered to move, the original shows a marker
  animation at the clicked cell. In a skirmish game it is created as an ordinary animation
  object, and only for the local player's own clicks. Our engine treats it as drawing only: it
  never takes part in game state and never draws from the scenario random stream. This is a
  departure, confirmed by the project lead. Because the original creates the marker only on the
  machine of the player who clicked, any effect it had on game state would already have split
  the original's own network games, so drawing only is consistent with the original. The
  glossary allows nothing that depends on the local player to reach game state.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules `[General]` `MaximumCheerRate` | *R* of claim 19 | integer, ticks (value form of S-0003) | 1 tick |
| Vehicle type sections, `Harvester` | harvester in claim 8 and table T2 | boolean (S-0003) | no |
| Building type sections, `WeaponsFactory` | vehicle-factory cell of claim 10 | boolean (S-0003) | no |
| Building type sections, `DockUnload` | the unloading kind of building of table T3, part (b) | boolean (S-0003) | no |
| Building type sections, `FirestormWall` | firestorm-wall flag of claim 13 | boolean (S-0003) | no |
| Building type sections, `Factory` | the factory kind of claims 13 and 20 (which kind of object it builds) | type-kind name (S-0003) | not a factory |
| Building type sections, `Naval` | the naval kind of factory in claims 13 and 20 | boolean (S-0003) | no |
| Building type sections, `UndeploysInto` | building that undeploys into a vehicle, which packs up when sold (claims 12 and 20) | vehicle type name (S-0003) | none |
| Building type sections, `ConstructionYard` | construction yard (claims 12 and 20) | boolean (S-0003) | no |
| Building type sections, `UnitRepair` | repair depot: rally-capable (Terms, claim 20); the stock case of a building under a sold vehicle (claim 12) | boolean (S-0003) | no |
| Building type sections, `Cloning` | cloning building: rally-capable (Terms, claim 20) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft type sections, `OpenTopped` | open-topped transport in table T3, part (f), and claim 8 | boolean (S-0003) | no |
| Infantry type sections, `Fraidycat`, `Cyborg` | type exclusions from destination-driven get-up (claim 21; S-0107 claim 15 owns the full rule) | booleans (S-0003) | no for each |
| Infantry type section, `Sequence`; its art sequence section, `Up` | sequence table and get-up entry used by claim 21 | section name; sequence entry as S-0107 Import mapping defines | no table; a missing entry has zero image frames and refuses a direct get-up request |
| No data key | order-to-destination assignments of claim 21, and the distinction between attack-order application and later approach | fixed behaviour, not a parsed list of movement orders | no imported order whitelist or chase destination |

The defaults in the Absent column are the values the original gives a field when the type (or the rules data) is created. They are engine constants (absent-key defaults), not values from stock data (confidence: medium). Each rules layer that omits a key keeps the value already in force (S-0006).

## Measurement scenarios

All scenarios run under the measurement setup: a skirmish on a small purpose-made flat map, one
human player and one bystander computer player (glossary) whose start position is far away, with
the bystander's units out of reach unless a scenario places some of them (Scenarios O3, O5, O11 and
O13 do) or hands objects to it (Scenario O16 does). A recorder reads the tick counter and, where
named, an object's behaviour, target, destination, guard spot, contact, position and owner, and
whether it is being sold, or the house's production, each tick. The recorder samples once per
tick, at the moment the tick counter advances, so a sample shows the state after all of that
tick's work, the applied orders included. The orders are given by hand, or by scripted input
where a scenario says so or exact timing helps. Scripted input is one of the allowances of the
measurement setup in the batch's glossary (*measurement setup*), added by the
project lead's decision of 2026-10-09: a script produces mouse and keyboard input through the
game's normal input only, as a player would, and never writes into the game.

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
  on open ground.
- *Procedure:* (a) select all 130 (by drag selection in several steps if needed) and give one
  move click to a far cell. Repeat (a) 3 times from the same start. (b) From the same start:
  select 128 of them and give a move click to a far cell; select the other 2 and give a move click
  to the same far cell; while all 130 are moving, select all 130 and press the stop key once. Repeat (b) 3 times from the same start.
- *Observable:* (a) the number of infantry whose destination changes, and which ones fail, in
  selection order where the selection order is known. (b) After the stop key: how many have no
  destination and how many keep moving, and which ones keep moving.
- *Expected:* (a) exactly 128 change their destination. (b) Exactly 128 stop and 2 keep moving,
  and the 2 that keep moving are the last 2 in selection order.
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
  selection and click again. Game C: (5) as step 1, then, while the vehicle follows the route and
  before it reaches the first cell, select it and press the stop key.
- *Runs:* 3 of each game. Route mode involves no random draw.
- *Observable:* whether the vehicle moves before the key is released; the order in which it
  visits the cells; whether it continues the route after (2) and after (5); whether the clicks of (3) and (4)
  are refused (message and sound).
- *Expected:* no movement before release, then the cells in click order; after (2) it goes to the
  new cell and never returns to the route; after (5) it comes to rest and does not go on to any
  cell of the route; in game B the first click of (3) is accepted and the
  second is refused, because 128 route holders plus 2 new ones exceed 128. Step 4 has no expected
  value: it answers the open question of claim 17. If its click is refused, the vehicle whose route
  was removed still counts as a route holder (1 + 128 > 128); if it is accepted, it no longer
  counts.
- *Falsified if:* the vehicle moves before release, it resumes the route after (2) or after (5), or in game B
  the refusal comes at another count.

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
- *Part (c), leader tie-break:* 2 identical vehicles *P* and *Q* of the player's standing at the
  same place within their cells, in two cells 4 cells apart, so that the centre of the two lies exactly half way
  between them and both are at the same distance from it; they are selected in the order *P*, *Q*; one move click to an empty cell 10 cells away. Then the same with the selection order
  *Q* then *P*. 10 runs of each order. Expected: the vehicle selected last gets the clicked cell
  (it is the latest in the run on equal distances), and the other gets a different cell. Falsified
  if in any run the vehicle selected first gets the clicked cell. (Spreading involves no random
  draw.)
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

## Open questions

- Q-0072: claim 21 answers the order-application contribution in full. S-0215 claim 37 now
  supplies the approach trigger, gating, renewal and building/deployed-infantry rules
  (questions 1, 4 and 5); its immobile-type case remains low confidence. S-0307 claim 15
  supplies question 2's open-ground search, with the remaining search paths still open in
  that spec. Question 3's stopping event and destination changes belong to S-0207 and
  S-0208: the approach step itself never clears a held destination merely because the
  target came in range (S-0215 claim 37 e). No owned part of Q-0087 remains open. The
  requests list records this spec's contributions, not closure of the cross-spec Q-0072.

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
- Which situations of play send the notices that tether a unit (Terms, *tethered*). Probably an
  aircraft that has landed on its pad; others, such as a vehicle leaving a factory or a harvester
  docking at a refinery, have not been checked (Scenario O4, part iv, records the refinery case).
- Claim 7: how many steps a member's look takes (the limit of the look line in Formulas), whether
  that limit applies to each member's look separately or to the run as a whole, what a member gets
  after a spoiled look, which direction the look of a member standing in the leader's own cell
  takes, and the first
  index *j*₀ of the look line, that is, whether the first step tests the common cell itself,
  whether a member that received a shared destination is still placed with a look of its own, and
  whether the leader counts as a member that is not yet placed when destinations are shared.
  Our engine's interim rule is stated in claim 7.

## Questions for the project lead

No decision is pending: revision 6 applied the project lead's decisions of 2026-10-09 (AI houses,
scripted input, the click marker). Revision 7 defers the surrender, alliance and power rows of
table T1 to S-0414 (claims 11 to 14) and S-0405 (claim 10); the game-end decision of 2026-10-11
(play to the last side) is S-0414's. Items for the lead to route at sign-off:

- **Cited specs of the same batch,** as they stand in it: S-0003 (revision 7), S-0006 (revision
  6), S-0009 (revision 6), S-0011 (revision 4), S-0107 (revision 4), S-0201 (revision 6), S-0203
  (revision 7), S-0204 (revision 5), S-0205 (revision 4), S-0208 (revision 7), S-0209 (revision
  13), S-0211 (revision 7), S-0212 (revision 13), S-0214 (revision 6) and S-0215 (revision 9), and
  the batch's glossary (revision 7). The
  terms that appear first in that glossary revision (*AI house*, *team member*, *contact*,
  *tether link*, and the scripted-input allowance of *measurement setup*) are given in this spec's
  Terms and scenario preamble, citing S-0208, S-0215 and the lead's decision, so the spec does not
  depend on them. Glossary revision 7 no longer lists S-0201, S-0204 or S-0205 as planned; the
  approved glossary (revision 4) still does.
- **S-0201 owner:** claim 2 (the cycle) should state, as claim 1 here does, that a cycle takes at most
  one input event, so that a click and a key press are never issued in the same tick (owner
  S-0201).
- **Check at sign-off:** S-0208, Terms and claim 11, define the scatter request's threat point, *forced*,
  *move anywhere* and *effectively forced* that claim 9 uses.
