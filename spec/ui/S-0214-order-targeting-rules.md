---
id: S-0214
title: Order targeting rules (what a click means for each kind of object, and group orders)
topic: ui
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0214: Order targeting rules

## Summary

How a click on the battlefield, with or without modifier keys, becomes orders for the selected
objects: which *action* an object takes on the thing under the cursor (an object or a cell),
which of the selected objects decides the cursor, how each selected object then resolves the
click for itself, and which order of S-0213 each action produces. The rules depend on the kind of
the acting object, its type's abilities, the relation of the target to it and the modifier keys
held. These are input rules. They decide which orders are created, never how an order is applied
(S-0213).

## Terms

Glossary terms are used as defined there, in particular *house*, *local player*, *object*,
*type*, *field*, *building*, *infantry*, *vehicle*, *aircraft*, *cell*, *lepton*, *shroud*,
*fog of war*, *disguise*, *jumpjet*, *movement zone*, *height above the surface*, *armour class*,
*unsynchronised random stream*, *departure*, *quirk switch*, *measurement setup* and *recorder*.
*Order*, *actor*, *behaviour order*, *selection*, *route mode*, *planned route*, *unit or
building*, *inside a tunnel*, *attack-move memory*, *planted bomb*, *paralysed*, *berserk* and
*primary factory* are defined in S-0213. *Behaviour*, the behaviour names, *contact*, *team
member* and *loaner* are S-0215's. *Reference point* and *table square root* are S-0204's. The
*usable area* and the *passability category* of a cell are S-0211's. The *entry outcome* is
S-0209's. The *destroyable-cliff role* of a theater's tiles is S-0010's.

- **Action.** The result of resolving a click for one selected object: one of the actions of
  table A, or *nothing*. The cursor shape the original shows for an action is presentation, and
  this spec does not specify it.
- **Acting house.** The house whose input is being resolved: the local player's house for a
  person's clicks. An *AI house*, a house with a computer-controlled owner (glossary
  *human-controlled*), is held to the same rules: when its order is checked
  against this spec, it is the acting house, and every rule below that names the local player
  names the acting house instead. By decision of the project lead, an AI house has the same order
  legality as a human-controlled house and no exemption (S-0213, claims 2, 3 and 15). Wherever a
  rule here asks whether a house is human-controlled, an AI house meets that condition too. The
  cursor, the click feedback (claim 14) and the
  modifier keys concern a person's input only.
- **Own object.** An object owned by the acting house. An actor that is not an own object
  always has the generic result of table G as its final action, which is *select* or *nothing*.
  All kind tables below assume an own actor.
- **Allied, enemy.** A target object is *allied* with the actor when the target's house and the
  actor's house are allied, or are the same house. Otherwise it is an *enemy*. A cell has no
  relation.
- **Modifier roles.** Three keys held while clicking change the action: *force-move*,
  *force-fire* and *force-select*. They are roles. Which physical keys play them is the player's
  key binding (the original's default binding is outside the scope of this spec). The
  *attack-move gesture* is force-fire and force-select held together.
- **Counted modifiers.** When force-fire and force-select are both held, every rule of this spec
  except claim 5 treats both as not held. "Held" below always means counted in this way.
- **Attack-move mode.** A one-click mode that the attack-move button of the original's optional
  command bar switches on (claim 5). While it is on, the attack-move gesture counts as held.
- **Armed.** The object has a weapon in the weapon slot it currently uses: its first slot, or for
  a type that switches between weapons, the current one. Every rule of this spec that asks whether
  an actor is armed asks this one question. A rule that looks at the first weapon slot instead
  says so explicitly ("a weapon in its first slot", "its first weapon"), and it differs only for a
  type that switches between weapons. For a building, the weapon in a slot can come from an
  upgrade on it, or, while it holds garrison occupants and its type lets them fire from inside
  (Import mapping), from its occupants (the combat specs own which weapon is used). A building that
  holds occupants and whose type lets them fire counts as armed even without a weapon of its own
  (medium). Rows GC4 and GC5 ask instead whether the weapon itself exists; for a building the two
  questions differ only while its occupants supply no weapon (low).
- **Healer.** An armed object whose weapon heals: the weapon it would use by default does negative
  damage. Combat specs own the weapon choice.
- **Disabled.** For these input rules: paralysed, or, for a vehicle, playing its death sequence.
- **Being erased, phasing in.** An object that a chrono weapon is removing from the map is *being
  erased*. An object that is materialising after a chrono jump is *phasing in*.
- **Parasite.** An object that has latched onto a mobile unit and harms it from there (for
  example a unit that climbs inside a vehicle, or one that grabs a ship). Its rules belong to the
  special-abilities specs. A unit with a parasite on it may be held still by a paralysis timer
  that the parasite starts.
- **Spawned aircraft type, slave unit.** An aircraft type that a carrier-like unit launches and
  controls (Import mapping), and a unit that a slave master of S-0215 controls.
- **Movable.** An own object that the player can move. For a mobile object all of these must
  hold: it is not disabled; it is not being erased or phasing in; it is not held still by a
  parasite's paralysis timer; it is not held inside a tank bunker; its type is not a spawned
  aircraft type; it is not a slave unit; it is not falling (for example under a parachute); and,
  if it launches spawned aircraft, it is not in the middle of launching them while fewer than its
  launch count are out. A building is movable when it is not paralysed and either it is
  rally-capable, or its type undeploys into a vehicle; a construction yard counts only when its
  house is human-controlled or an AI house (Terms, *acting house*), the game is not a campaign game, the game option that lets
  construction yards be packed up again is on, and the yard is not mind-controlled.
- **Able to fire.** An own object that is armed, not disabled and not falling.
- **Melee-linked.** An infantry unit and a vehicle that a close-combat weapon has locked onto each
  other until the fight ends. A melee-linked object whose type lacks the keep-acting flag (Import
  mapping) takes no actions (table G).
- **Deploy check.** An object passes it unless it is a vehicle that is disabled, or a vehicle that
  is inside a tunnel while its type has the deploy-to-fire flag (Import mapping). Infantry,
  aircraft and buildings always pass it.
- **Deploy readiness.** Whether an object can carry out a deploy now, by its kind. Never on a
  tunnel cell or at a tunnel's edge (*tunnel cell* below). A building never has it. An aircraft has
  it when it is not disabled and its type can carry passengers. An infantry unit has it when its
  type deploys in place, and, if its type has an undeploy delay, when it is neither deployed nor
  undeploying nor within the timer that follows a deploy. A vehicle has it when it is not
  disabled, not inside a tunnel, and it deploys into a building, carries passengers, deploys in
  place, sits in a tank bunker or has a deploy weapon; but not when it deploys into a building
  while a parasite is on it, not when it deploys into a construction yard while it is
  mind-controlled, and not when it unloads passengers on a bridge, next to a bridge, onto water in
  some cases, or into a building cell other than a tank bunker's (low for the details).
- **In-place deployer.** An infantry or vehicle type that deploys in place and later undeploys (it
  changes between a deployed and an undeployed state without becoming another object).
- **Representative.** The selected object whose action is shown as the cursor (claim 1).
- **Health ratio.** An object's current hit points divided by its type's hit points, computed in
  floating point (Formulas F2). *Full health* means a health ratio of exactly 1.
- **Garrison.** Infantry entering a building to fight from inside it.
- **Recycling building.** A building whose type turns units that enter it into credits.
- **Bridge repair hut.** A building type that, when an engineer enters it, rebuilds the bridge
  linked to it.
- **Docking building.** A building listed in an aircraft type's dock list, such as a landing pad.
- **Vehicle-like building.** A building whose type undeploys into a vehicle and has a 1 × 1
  foundation. Rules below that speak of "a building" exclude it where they say so.
- **Counts as a vehicle.** A vehicle whose type is not flagged as a non-vehicle; an aircraft that
  is on the ground; or a vehicle-like building.
- **Train.** A vehicle whose type carries the train flag (Import mapping).
- **Appears disguised.** A target that is disguised in the eyes of the actor's house (glossary
  *disguise*).
- **Disguised as scenery.** A target that is disguised and whose disguise belongs to no house, so
  that it looks like part of the map (for example a tree). *Disguised as an overlay* is the case in
  which the disguise is an overlay type.
- **Scenery check, fallback move.** Results used by the kind tables for a target object *T*. The
  *scenery check* gives *move* if *T* is disguised as an overlay, and *no-move* otherwise. The
  *fallback move* gives the scenery check's result for a movable actor, and *nothing* for an actor
  that is not movable.
- **Docking answer.** When a row asks whether the target accepts the actor now, the target answers
  through the contact exchange of S-0215 (and the building-kind behaviours). The answer is
  *accepts*, *refuses* or neither. A full building or transport refuses.
- **Terrain-fire weapon.** A weapon whose type carries the terrain-fire flag (Import mapping).
- **Rock overlay.** An overlay whose type carries the rock flag (Import mapping).
- **Tunnel cell.** A cell whose land type is the tunnel land type, or a cell that the original
  counts as lying at the edge of one (low for the edge rule). A *tunnel entrance* is a cell whose
  tile is a tunnel tile.
- **Destroyable-cliff cell.** A cell whose tile number *t* satisfies *s* ≤ *t* ≤ *s* + 1 and
  0 ≤ *t* < *N*, where *s* is the start of the theater's destroyable-cliff role (S-0010) and *N*
  the theater's tile count: the role covers two tile numbers. The same rule holds whether or not
  the theater sets the role. A cell holding the no-tile marker is never a destroyable-cliff cell,
  however the marker is represented. An unset role has *s* = −2 (S-0010), so it covers only −2
  and −1, neither of which satisfies 0 ≤ *t*, and on a theater without the role no cell is a
  destroyable-cliff cell. This is why the unset start matters: with *s* = −1 the role would cover
  tile 0.
- **Monster of a veinhole (veinhole monster).** A stationary map object of the original's veinhole kind. Only its effect
  on the target of an attack order (table M) is used here.
- **Balloon-hovering type.** An infantry type with the balloon-hover flag (Import mapping).
- **Rally-capable building.** A factory of infantry or vehicles, a cloning building or a repair
  depot. Nothing else is rally-capable: a tank bunker is not, and an aircraft factory is only when
  its type is also a repair depot.
- **Veterancy rank, veterancy ability.** An object's veterancy rank is rookie, veteran or
  elite, and elite is the top rank. How rank is earned belongs to the veterancy specs. A *veterancy ability*
  is S-0215's term: a capability that the type grants once the object reaches a given rank. This
  spec uses the explosive-charges ability (row I16) and the crusher ability (value *k* of table V).
- **Mind-controlled.** An object that is currently under the *mind control* of S-0215: held by a
  controlling object, which took it over and keeps it while the control lasts. The combat topic
  owns mind control. Whether a permanent takeover that leaves no controlling object counts is not
  settled (low; Open questions).

## Behaviour

1. **The representative.** When the selection is not empty, the action the cursor shows over a
   target (an object or a cell) is the action of one selected object, the representative. Each
   selected object gets a rank from table R. The representative is the object with the highest
   rank. Among objects of equal rank it is the one nearest to the target (Formulas F1), and among
   those, the one selected earliest. With an empty selection the cursor shows the actions of the
   current sidebar mode (claim 13), or *select* over an object that can be selected.
   - Backing: `measurable` (Scenario T1)
   - Confidence: high for the rank order, medium for the distance

2. **Each object resolves the click itself.** When the player clicks, the click is given to every
   selected object in selection order. Each object works out its own action against the same
   target by the rules of claims 4 to 9, and turns its action into an order by table M (claim 10).
   An object whose own action gives no order gets none. The representative's action decides only
   what the cursor shows, and whether the click is treated as a command at all (claim 3). In a
   mixed group, each object therefore does what it alone would do. For example, a click on an
   enemy building with engineers and tanks selected sends the engineers to capture it and the
   tanks to attack it.
   - Backing: `measurable` (Scenario T2)
   - Confidence: high

3. **Clicks that are not commands.** If the representative's action is *select* or *toggle
   select*, the click changes the selection instead (claim 12), and no object gets an order. If it
   is *nothing* and the clicked object can be selected and is not selected yet, the click acts as
   *select*. If it is *nothing* over a cell, or over an object that cannot be selected, nothing
   happens. While route mode is held (S-0213, claim 16), a command click adds route points instead
   of creating orders, and is refused when the route-holder limit would be passed (the limit and
   its refusal are S-0213's, claim 16).
   - Backing: `measurable` (Scenario T2; the refusal at the route-holder limit by S-0213's
     Scenario O8)
   - Confidence: high

4. **The generic result on an object.** Over an object, every actor has a *generic result* *g*
   by table G. It covers the modifier roles, clicks on the actor itself, the attack rule of claim
   6 and plain selection. Table G is a precedence list: the first row whose condition holds gives
   *g*. The final action of an own actor is a function of *g* and the actor's and target's
   properties, given by its kind's table (claims 7 and 8). For an actor that is not an own object,
   *g* is the final action.
   - Backing: `measurable` (Scenario T3, including row G3, and Scenarios T2 and T6 for rows G4,
     G5 and G10; Scenario T6 also for an armed garrisoned building). Row G2,
     and the being-erased and phasing-in part of row GC1, are `unmeasured`: they need an actor that
     is being erased or phasing in at the moment of the click, which a scenario cannot hold steady.
   - Confidence: medium

5. **The attack-move gesture.** If the attack-move gesture is held (or attack-move mode is on)
   and every selected unit or building is eligible, then a click whose order would be *move* or
   *attack* produces an attack-move order instead (S-0213, claim 6): an order that would be *move*
   is issued as *attack-move to a place*, and one that would be *attack* as *attack-move on a
   target*. This covers every action whose order is *move* or *attack* in table M, including bomb,
   demolish, airstrike and enter tunnel. An object is eligible if it is infantry or a vehicle, has a
   weapon in its first slot, and its type does not carry the prevent-attack-move flag. Buildings
   and aircraft are never eligible. One ineligible object in the selection turns the gesture off
   for the whole selection. Whether or not the gesture is on, force-fire and force-select held
   together count as if neither were held for every other rule of this spec (Terms, *counted
   modifiers*). Attack-move mode is switched on by the command-bar button only when the selection
   is not empty and every selected unit or building is eligible (otherwise a message appears and
   the mode stays off); it ends when the next battlefield click has been handled or the selection
   is cleared.
   - Backing: `measurable` (Scenario T3)
   - Confidence: medium (low for the end of attack-move mode)

6. **The attack rule.** The attack rule holds for an own actor and a target object when all of
   the following hold. Row G8 of table G uses it, after the rows above it:
   - (a) force-fire is held; or all three of: the target is an enemy or the actor may attack it as
     a friend (below); the target passes the disguise rule (below); and the target's type is a
     legal target, or the terrain-fire rule (below) allows the target,
   - (b) the target can be attacked at all: its height above the surface is more than −20
     leptons (Formulas F3), so a unit travelling underground cannot be clicked for attack,
   - (c) the actor is armed, or is an infantry type with the infiltrate flag,
   - (d) the actor is movable, or the target is already in range of the weapon it would use, and
   - (e) unless force-fire is held, or the actor is an infiltrating infantry type facing a building
     it can enter (a spy and a spyable building, another infiltrator and a capturable building, or
     an infiltrator with the charge flag and a building that accepts charges), the firing rules of
     the combat specs do not rule the target illegal for the weapon it would use. Other reasons for
     not firing at this moment (facing, reloading, rotating, range and so on) do not count.

   *May attack as a friend:* the actor's type carries the attack-friendlies flag; or it carries the
   attack-cursor-on-friendlies flag, unless the warhead of the weapon it would use plants bombs and
   the target's type can carry passengers; or that warhead is sonic and the target is a mobile unit
   with a parasite on it.

   *Disguise rule:* a target that does not appear disguised passes. One that appears disguised
   passes only when the general rules allow an attack cursor on disguised units, or its type is
   permanently disguised, or its disguise is currently disrupted (blinking), or the actor may
   attack it as a friend, or it is disguised as scenery and the terrain-fire rule allows it (low
   for the last case).

   *Terrain-fire rule:* the target is a terrain object (such as a tree) and the general rules
   allow tree targeting or the weapon the actor would use against it is a terrain-fire weapon; or
   that weapon is a terrain-fire weapon and the target is a unit or building disguised as scenery
   (low).

   Whether a generic *attack* ends as *attack* is for the kind tables: for example, against a
   target whose type is immune to damage most kinds show *no-move* instead.
   - Backing: `measurable` (Scenario T4, including a unit travelling underground for part (b)).
     The exact limit of −20 leptons in part (b) is `unmeasured`: only a unit passing between the
     surface and underground travel has a height near it, and that passage cannot be held at the
     moment of a click.
   - Confidence: medium (low where marked)

7. **Infantry on an object.** The final action of an own infantry actor on a target object is
   given by table I, a precedence list over the generic result and the actor's and target's
   properties.
   - Backing: `measurable` (Scenario T5)
   - Confidence: medium (low where a row says so)

8. **Vehicles, aircraft and buildings on an object.** The final action of an own vehicle follows
   table V, of an own aircraft table AC and of an own building table B, in the same way.
   - Backing: `measurable` (Scenario T6)
   - Confidence: medium for table V; low for tables AC and B

9. **Actions on a cell.** Over a cell, every actor has a generic cell result *c* by table GC.
   For an infantry unit, vehicle or aircraft, a shrouded cell changes it into *c*′: if *c* is not
   *nothing* and the cell is shrouded, *c*′ is *move* (or *patrol*, if *c* is *patrol*) when the
   actor's type may move into shroud and the cell is inside the usable area, and *no-move*
   otherwise; in every other case *c*′ = *c*. The final action is a function of *c*′ (of *c* for a
   building), given by the kind's cell table (IC, VC, ACC or BC). For an
   infantry unit, vehicle or aircraft, a click on a cell outside the usable area produces an order
   only when the action is *no-move*.
   - Backing: `measurable` (Scenario T7)
   - Confidence: medium (low for the building cell table)

10. **From action to order.** Table M gives the order that each action produces when an object
    resolves a click, including the actions for which an actor gets no order, by the actor's
    kind.
    - Backing: `measurable` (Scenarios T2 and T6)
    - Confidence: medium (low for buildings and aircraft)

11. **Move destinations.** The destination of a *move* or *no-move* order is not always the
    clicked cell.
    - *On a cell:* it is a nearby cell that the actor's movement zone can use, found by the
      nearby-cell search of S-0212 (claim 25) from the clicked cell. When none is found, or the
      clicked cell is shrouded and the actor's type may not move into shroud, the actor gets no
      order.
    - *On an object:* it is the clicked object itself when the action is *move* and either the
      actor flies, travels underground or is a jumpjet infantry type, or the object's cell is
      inside the usable area, not shrouded, reachable within the actor's movement zone, and
      force-move is not held by an infantry actor. Otherwise it is a nearby usable cell found by
      the same search, and the object itself when the search finds none, so a *no-move* on an
      object still orders a move, except where part 1 of table M gives no order. A *no-move* on an object in shroud gives no order unless the
      actor's type may move into shroud. A ground actor that travels underground (unless the
      general rules allow underground moves into shroud) and an aircraft get no order for a move
      onto an object in shroud.
    - *Zones in the search:* an underground movement zone always counts as the ordinary ground
      zone. A flying zone counts as the on-foot zone for a click on an object always, and for a
      click on a cell only while an ion storm is active (low).
    - Backing: `measurable` (Scenario T7). The ion-storm part is `unmeasured`: no way to start an
      ion storm in a skirmish with stock data is known (Open questions).
    - Confidence: medium

12. **Selection clicks.** *Select* replaces the selection with the clicked object. *Toggle
    select* adds the clicked object to the selection, or removes it if it is already selected; it
    acts only when the first selected object is the acting house's own, and otherwise acts as
    *select*. While the type-select key is held, both act on every object of the clicked object's
    type. The type-select key pressed twice within 500 milliseconds selects that type across the
    whole map, instead of on the screen. The original measures this interval in real time; our
    engine's interface applies the same 500 ms to the times of its input events (Edge cases,
    departure).
    - Backing: `measurable` (Scenario T2)
    - Confidence: medium

13. **Sidebar modes.** With an empty selection, the sidebar's mode buttons give these actions:
    - *Sell mode:* over an own building that can be sold and is not vehicle-like, *sell*. Over a
      wall cell owned by the acting house that is neither shrouded nor fogged, *sell*, which
      produces the sell wall order. Anywhere else, *no-sell*, including over own vehicles,
      aircraft and vehicle-like buildings. No rule studied gives a *sell unit* action, which would
      produce a sell order for a vehicle or aircraft (Open questions).
    - *Repair mode:* over an own object that can be repaired, *repair switch*. Anywhere else,
      *no-repair*. A click with *repair switch* produces a repair order only on a building.
    - *Power mode:* over an own selectable building that is not vehicle-like, whose type can be
      switched off, and that makes power or needs power, *power switch*. Anywhere else,
      *no-power*.
    - *Superweapon targeting:* the superweapon's own targeting rule (superweapon specs).

    A click with *sell* produces a sell order (S-0213, claim 12), with *repair switch* a repair
    order, and with *power switch* a power-on order for a building that is offline and a
    power-off order for one that is online.
    - Backing: `measurable` (Scenario T8)
    - Confidence: medium

14. **Click feedback.** Only the first selected object that receives the click gives the spoken
    acknowledgement. For an order that has no acknowledgement of its own kind (the kinds are
    harvest, attack, move and attack move, enter, capture and unload), the acknowledgement is one
    of the type's selection responses, chosen by one draw from the unsynchronised random stream.
    The marker at a move target is likewise created once per click, by the first object. The
    original gives both for the local player's own clicks only. In our engine both are drawing and
    sound only: they take no part in game state and make no draw from the scenario random stream,
    and an AI house's orders give neither. This is the departure confirmed by the project lead and
    stated in S-0213 (Edge cases, *the click marker*): since the original creates them only on the
    machine of the player who clicked, any effect on game state would already have split its own
    network games, so drawing only is consistent with it.
    - Backing: `measurable` (Scenario T2), by ear and by eye, for the single acknowledgement and
      the single marker. The draw is also `measurable`: Scenario T2 part B reads the unsynchronised
      random stream with the read-only random-state recorder of the measurement setup (the same
      extension of S-0202's state recorder that S-0010's scenario V2 part B uses), counts the draws
      a click takes and compares the chosen response with the draw's value.
    - Confidence: high (medium for the draw)

15. **Objects that get no order.** An infantry unit, vehicle or aircraft that is berserk, or that
    is melee-linked while its type lacks the keep-acting flag, gets no order from any click on an
    object or a cell, whatever action the tables give it, with one exception: a berserk carrier
    aircraft can still get *tote* (rows AC2 and AC3) and the move order that *tote* produces
    (table M, low). (Its generic result is *nothing*, but
    some infantry and vehicle rows do not depend on the generic result, so such an actor can still
    show an action when it is the representative.) Its rank is 0 (table R) if it is berserk.
    - Backing: `measurable` (Scenario T6) for the berserk case. The melee-linked case is
      `unmeasured`: it needs a melee link at the moment of the click, which a scenario cannot hold
      steady.
    - Confidence: medium

## Formulas

> **F1 (representative distance).** *d* = ⌊*r*(*n*)⌋, *n* = (*x*ₒ − *x*ₜ)² + (*y*ₒ − *y*ₜ)² +
> (*z*ₒ − *z*ₜ)², where (*x*ₒ, *y*ₒ, *z*ₒ) is the selected object's reference point (S-0204,
> claim 4) and (*x*ₜ, *y*ₜ, *z*ₜ) is the target object's reference point, or for a cell target the
> cell centre (256*x* + 128, 256*y* + 128) at height 0. *r* is S-0204's table square root
> (S-0204, claim 5 and F1); no building reduction applies. Units: leptons. The original forms
> the squares and their sum in double precision from integer coordinates, which is exact for every
> map position, and converts the root to an integer by truncation toward zero, which holds in every
> rounding mode (S-0205, claim 7). Equal *d* values are tied (claim 1).

> **F2 (health ratio).** *h* = *p* / *P*, where *p* is the object's current hit points and *P* its
> type's hit points, both integers, divided in double-precision floating point. Comparisons:
> *h* < 1 (below full health), *h* = 1 (full health), and *h* ≤ *c* (row I7), where *c* is the
> engineer-capture field. In the original, *c* is a single-precision floating-point value and the
> comparison *h* ≤ *c* is made in double precision, into which a single-precision value converts
> exactly. Since *p* and *P* are integers, *h* = 1 exactly when *p* = *P*. Rounding of the
> quotient is owned by S-0205 (number domains). Row I4 tests the acting engineer's own *h* < 1,
> while rows I5 and I12 and value *b* of table V test the target's *h*. The hospital part of value *e*
> in table I compares the actor's integers *p* < *P* directly.

> **F3 (attackable height).** The target can be attacked at all when *z*<sub>s</sub> > −20, where
> *z*<sub>s</sub> is its height above the surface in leptons. Integer comparison, exact.

> **F4 (passenger land-speed limit, row VD7).** The first passenger counts as unable to move on
> the cell's land type when *m* < 0.01, where *m* is the speed multiplier of the passenger's
> movement class on that land type. In the original, *m* is a single-precision floating-point
> value and 0.01 is the double-precision value nearest to 0.01; the comparison is made in double
> precision, into which *m* converts exactly, so it is exact and no rounding mode affects it.
> The single-precision value nearest to 0.01 is slightly below the double-precision 0.01, so a
> multiplier that becomes 0.01 when rounded to single precision counts as unable to move.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Type-select double press | 500 | milliseconds | Claim 12. A second press counts if it comes less than 501 ms after the first. Input only. |
| Full health | 1 | health ratio | Tables I and V. The original fixes the threshold at 1 and does not read it from data. |
| Attackable height limit | −20 | leptons of height above the surface | F3; a target must be strictly above it |
| Passenger land-speed limit | 0.01 | speed multiplier of a land type (a factor; 1 = unchanged speed) | Row VD7 and Formulas F4; a first passenger whose multiplier on the cell's land type is strictly below it counts as unable to move there (not on water) |

## State and ordering

**Table A: actions.** Our own names for the actions, for reference in the other tables.

| Action | Meaning |
|---|---|
| select, toggle select | Change the selection (claim 12) |
| move, no-move | Move to the target, or a move that the cursor shows as not possible |
| attack | Attack the target (object or cell) |
| attack-move to a place, attack-move on a target | Claim 5 |
| enter, no-enter | Enter the target (transport, building, docking building), or refused |
| deploy, no-deploy | Deploy or unload, or refused |
| area guard | Guard the target object (escort), or guard a cell |
| patrol | A route action of route mode on a cell (S-0213, claim 16) |
| capture | Capture, garrison, infiltrate or hijack |
| damaging entry | An engineer entering an enemy capturable building that is above the capture threshold, to damage it instead of capturing it (special-abilities specs) |
| repair building, no-repair | An engineer repairing an own or allied building or a bridge, or refused |
| recycle | Enter a recycling building to be recycled |
| heal, repair vehicle | Heal infantry, or repair a vehicle |
| sabotage | Plant an explosive charge on a building |
| bomb, no-bomb | Attach a bomb, or refused |
| detonate | Set off the planted bomb the actor carries |
| disarm bomb | An engineer removing a planted bomb from an object |
| tote | A carrier aircraft picking up a vehicle |
| harvest | Harvest the resource at the cell |
| enter tunnel, no-enter tunnel | Use a tunnel entrance |
| area attack | Fire a deployed weapon over an area |
| demolish, airstrike | Special attacks of some weapons on buildings and vehicles |
| idle | No usable action: a healer infantry unit over an enemy object (row I10), or a healer vehicle whose cell action would be attack (row VC1) |
| sell, no-sell, repair switch, no-repair, power switch, no-power | Sidebar modes (claim 13) |
| nothing | No action |

**Table R: representative rank** (claim 1). The first matching row gives the rank.

| Rank | Selected object |
|---|---|
| 0 | Not a unit or building, or berserk |
| 1 | Disabled |
| 2 | Not armed |
| 3 | An armed building |
| 4 | An armed unit whose weapon does zero or negative damage (a healer, or a weapon without damage) |
| 5 | An armed unit whose weapon does positive damage |

**Table G: generic result *g* on an object** (claim 4). The first row whose condition holds gives
*g*. In a campaign game a further rule applies to targets under shroud (Open questions).

| Row | Condition | *g* |
|---|---|---|
| G1 | The actor is berserk, or melee-linked while its type lacks the keep-acting flag | nothing |
| G2 | The actor is being erased | nothing if the actor is armed and own and the target belongs to another house, or the actor is an own loaner; otherwise select if the target can be selected and is not selected yet, else nothing |
| G3 | The target is the actor itself, it is the only selected object, it is own, and it carries a planted bomb of a kind the general rules let the player set off | detonate |
| G4 | Force-select held, the actor own and not a loaner | toggle select |
| G5 | The target is the actor itself, it is the only selected object, and it is own | deploy |
| G6 | Force-move and force-fire held, the target an allied object, the actor own, movable and not an aircraft | area guard (escort the target) |
| G7 | Force-move held, the actor own and movable | move |
| G8 | The attack rule of claim 6 holds | attack |
| G9 | The actor armed and own, and the target owned by another house | nothing |
| G10 | The actor is not an own loaner, and the target can be selected, is not selected yet and is not being erased | select |
| G11 | Otherwise | nothing |

**Table I: infantry on an object** (claim 7). The rows use *g* and two values derived from it,
*g*₁ and *g*₂, each defined in full below. The derived values of this table and of table V are
functions of the actor, the target, the modifiers held and *g*. Values *e*, *v* and *x* are given
by small tables in which the first matching row gives the value; no row of table I reads the
result of another row.
"Engineer", "spy" and the rest name type flags (Import mapping). An *engineer-friendly building*
is a repairable building, not vehicle-like, that is allied with the actor, or that belongs to a
passive neutral house, can be garrisoned and is not being erased. A *deployed in-place deployer*
is an in-place deployer in its deployed state.

*Entry verdict e.* Defined only when the target is a building allied with the actor and not being
erased. Where no row matches, *e* is undefined.

| Row | Target and actor | *e* |
|---|---|---|
| e1 | An armory; the actor at elite veterancy rank | no-enter |
| e2 | An armory; the actor not a deployed in-place deployer; docking answer *accepts* | enter |
| e3 | An armory; the actor not a deployed in-place deployer; docking answer *refuses* | no-enter |
| e4 | A hospital; force-fire not held; the actor has all its hit points | no-enter |
| e5 | A hospital; force-fire not held; the actor not a deployed in-place deployer; docking answer *accepts* | enter |
| e6 | A hospital; force-fire not held; the actor not a deployed in-place deployer; docking answer *refuses* | no-enter |

So a building that is both an armory and a hospital is judged by the hospital rows whenever the
armory rows give no verdict (the actor is below elite rank, and is a deployed
in-place deployer or gets a docking answer that is neither).

*g*₁ = *e* where *e* is defined, and *g* otherwise.

*Deploy value v.*

| Row | Condition | *v* |
|---|---|---|
| v1 | *g*₁ is not deploy | *g*₁ |
| v2 | Route mode held | no-deploy |
| v3 | The actor's deploy weapon fires over an area | area attack |
| v4 | The actor's type deploys in place | deploy |
| v5 | Otherwise | nothing |

*Transport verdict x.* Defined only when the target is an allied unit or building whose type
carries passengers, not being erased and not phasing in, the actor is movable, and *v* is none of
attack, area guard and no-enter. Where no row matches (the docking answer is neither), *x* is
undefined.

| Row | Actor and target | *x* |
|---|---|---|
| x1 | The actor is a balloon-hovering type | no-enter |
| x2 | The target is a mobile unit that is moving, or a team member whose team's type lacks a flag that this spec has not identified (low) | no-enter |
| x3 | Docking answer *accepts* | enter |
| x4 | Docking answer *refuses* | no-enter |

*g*₂ = *x* where *x* is defined, and *v* otherwise.

*Final action.* The first matching row gives it.

| Row | Actor and target | Final action |
|---|---|---|
| I1 | *g* is toggle select | toggle select |
| I2 | Engineer; the target carries an active planted bomb | disarm bomb |
| I3 | Engineer; the target a repairable building, not vehicle-like, that is a bridge repair hut | repair building if the linked bridge is destroyed, else no-repair |
| I4 | Engineer below full health (the engineer's own health ratio, F2); the target an engineer-friendly building that is a hospital | enter |
| I5 | Engineer; the target an engineer-friendly building at full health | recycle if the building recycles, else no-repair |
| I6 | Engineer; the target any other engineer-friendly building | repair building |
| I7 | Engineer; the target a repairable building, not vehicle-like and not engineer-friendly, of a capturable type | capture if *h* ≤ *c* (Formulas F2), else damaging entry |
| I8 | The target a building, not being erased, that this infantry can garrison; force-fire not held | capture (garrison) |
| I9 | Engineer; *g* is attack | no-move |
| I10 | Healer; the target an enemy | idle |
| I11 | Healer; the target is the actor itself | area guard |
| I12 | Healer; the target infantry below full health | heal |
| I13 | Healer; the target is not a unit or building, or its type carries no passengers | *g* if *g* is move or area guard, else select |
| I14 | Vehicle thief; the target counts as a vehicle and is a train | select |
| I15 | Vehicle thief; the target counts as a vehicle and belongs to another house (allied ones included) | capture (hijack); see the harvester-truce quirk |
| I16 | Charge planter (the type flag, or the explosive-charges veterancy ability); *g*₁ is attack; the target a building, not vehicle-like | sabotage if the building's type accepts charges and is not invisible in the game, else attack |
| I17 | *g*₁ is attack, and the weapon the actor would use against the target has the demolition-cursor flag | select if the target is a vehicle held inside a tank bunker, else demolish |
| I18 | *g*₁ is attack, and that weapon has the air-strike-cursor flag | airstrike if the target is a building that accepts charges and is not invisible in the game, else attack |
| I19 | Bomb planter; *g*₁ is attack | bomb if the target's type accepts bombs and it carries none, else no-bomb |
| I20 | An in-place deployer in its deployed state; *g*₁ is move | no-move |
| I21 | An in-place deployer in its deployed state; *g*₁ is attack; its type does not fire while deployed, or the target is out of range of its first weapon | no-move |
| I22 | An in-place deployer in its deployed state; *g*₁ is deploy; route mode held, or its type has an undeploy delay | no-deploy |
| I23 | An in-place deployer with an undeploy delay that is undeploying; *g*₁ is deploy | no-deploy |
| I24 | Infiltrator; *g*₂ is attack; the target an enemy building of a spyable type (for a spy) or of a capturable type (for another infiltrator) | capture (infiltrate) |
| I25 | Infiltrator, not armed; *g*₂ is attack | fallback move |
| I26 | *g*₂ is select; the target a recycling building, not vehicle-like | recycle if the building is the acting house's own, else no-enter |
| I27 | *g*₂ is attack; the target's type is immune to damage | no-move |
| I28 | *g*₂ is nothing | fallback move |
| I29 | Otherwise | *g*₂ |

Consequences of the precedence: an immune building still gets *sabotage*, *demolish*, *airstrike*
or *attack* from rows I16 to I18, since those come before I27; an engineer told to attack gets *no-move* from I9
without the scenery check; and an infantry unit that does not deploy, clicked on itself, gets the
fallback move (row I28), usually *no-move*.

**Table V: vehicle on an object** (claim 8). The rows use *g* and the derived values *k*, *b*, *n*
and *q*, each defined in full below by a small table in which the first matching row gives the
value. Like those of table I, they are functions of the actor, the target, the modifiers held and
*g*: the letters name results, not steps, and fix no order of evaluation.

*Fire-or-deploy value k* (a vehicle unable to fire, or clicked on itself).

| Row | Condition | *k* |
|---|---|---|
| k1 | *g* is attack; the vehicle is not able to fire; it can crush (its type flag, or the crusher veterancy ability) and the target can be crushed by it | move |
| k2 | *g* is attack; the vehicle is not able to fire | select |
| k3 | *g* is deploy | the result of table VD |
| k4 | Otherwise | *g* |

*Healer value b.* Rows b1 to b3 apply only to a healer vehicle with *g* other than deploy, and they
exclude each other.

| Row | Condition | *b* |
|---|---|---|
| b1 | The target is an enemy | nothing |
| b2 | The target is allied and not a building, and it does not count as a vehicle, is the actor itself or cannot be attacked at all (F3) | select |
| b3 | The target is allied, counts as a vehicle, is not the actor, can be attacked (F3), is below full health and is not an aircraft standing on a building's cell | repair vehicle |
| b4 | Otherwise, and for every vehicle that is not a healer | *k* |

*Building-entry value n.* An *accepting allied building* is a building allied with the actor, not being erased,
whose docking answer is *accepts*.

| Row | Condition | *n* |
|---|---|---|
| n1 | *b* is select; the target an accepting allied building; the vehicle's movement zone is flying | no-enter |
| n2 | *b* is select; the target an accepting allied building that is a recycling building | recycle |
| n3 | *b* is select; the target an accepting allied building | enter |
| n4 | *b* is select; the target a repair depot or tank bunker whose docking answer is *accepts*, that has no unit in contact with it and holds no unit | move (low; for an allied building that is not being erased rows n1 to n3 always come first, so this row can only concern a building that is not allied or is being erased) |
| n5 | Otherwise | *b* |

*Transport verdict q.* Rows q1 to q3 apply only when *n* is none of attack, repair vehicle, area guard and
no-enter, and the target is an allied unit or building other than the actor, not being erased and
not phasing in, whose type carries passengers.

| Row | Condition | *q* |
|---|---|---|
| q1 | The target is a mobile unit that is moving | no-enter |
| q2 | Docking answer *accepts* | enter |
| q3 | Docking answer *refuses* | no-enter |
| q4 | Otherwise (including a docking answer that is neither) | *n* |

*Final action.* The first matching row gives it.

| Row | Vehicle and target | Final action |
|---|---|---|
| V1 | *g* is toggle select | toggle select |
| V2 | An in-place deployer in its deployed state; *q* is move | scenery check |
| V3 | An in-place deployer in its deployed state; *q* is attack; its type does not fire while deployed, or the target is out of range of its first weapon | no-move |
| V4 | *q* is nothing | fallback move |
| V5 | *q* is attack; the vehicle fails the deploy check | nothing |
| V6 | *q* is attack; the target's type is immune to damage | no-move |
| V7 | Otherwise | *q* |

Consequences: a healer vehicle over an enemy gets the fallback move (usually *no-move*, and a
click then moves it toward the enemy, claim 11); and an unarmed vehicle clicked on an own building
that accepts it gets *enter* with or without force-fire, by rows G10 and n3, since force-fire
gives an unarmed actor no generic *attack* (claim 6 (c)).

**Table VD: a vehicle clicked on itself** (*g* = deploy). The first matching row gives *k*.

| Row | Vehicle | *k* |
|---|---|---|
| VD1 | Route mode held | no-deploy |
| VD2 | It deploys into a building | deploy if that building could be placed where the vehicle stands (the vehicle itself not counting as an obstacle; for a foundation wider or deeper than 2 cells the placement is anchored one fixed neighbour step away from the vehicle's cell, low) and the vehicle has deploy readiness; else no-deploy |
| VD3 | It deploys in place | deploy if it has deploy readiness, else no-deploy |
| VD4 | Its type carries no passengers, it has no deploy weapon, and it is not held in a tank bunker | nothing |
| VD5 | It is held in a tank bunker, or its type carries no passengers and it has a deploy weapon | deploy if it has deploy readiness, else no-deploy |
| VD6 | A transport that carries no passengers now | no-deploy |
| VD7 | A transport, carrying passengers, not on a bridge, and either its first passenger could not move on the land type of the cell (a speed multiplier below 0.01 there, Formulas F4, the land not being water), or the transport is in the air while its type does not hover | no-deploy |
| VD8 | Otherwise | deploy if it has deploy readiness, else no-deploy |

**Table AC: aircraft on an object** (claim 8). The first matching row gives the final action. All
rows: low.

| Row | Aircraft and target | Final action |
|---|---|---|
| AC1 | *g* is toggle select | toggle select |
| AC2 | Carrier aircraft (it can pick up vehicles), carrying nothing; *g* is select or nothing; the target a vehicle allied with it; the target stands on a cell that holds a vehicle factory | nothing |
| AC3 | As AC2, but the target stands anywhere else | tote |
| AC4 | *g* is nothing; the aircraft's action on the target's cell (cell tables, without fog of war) is no-move; the target is disguised as an overlay | move |
| AC5 | *g* is nothing; that cell action is attack; the target's type is immune to damage | no-move |
| AC6 | *g* is nothing | that cell action |
| AC7 | *g* is deploy | deploy if the aircraft carries passengers, else nothing |
| AC8 | *g* is attack; no weapon in its first slot | nothing |
| AC9 | *g* is attack; the target's type is immune to damage | no-move |
| AC10 | *g* is select; the target a building with the repair-depot flag | enter if the target's type is in the aircraft's dock list, it has a free place for this aircraft, it holds no unit and its docking answer is *accepts*; else no-enter |
| AC11 | Otherwise | *g* |

**Table B: building on an object** (claim 8). The first matching row gives the final action. All
rows: low.

| Row | Building and target | Final action |
|---|---|---|
| B1 | The actor's type, or the target's type if the target is a building, is invisible in the game | nothing |
| B2 | *g* is toggle select | toggle select |
| B3 | *g* is deploy; the building is a factory that is already its house's primary factory | nothing |
| B4 | *g* is deploy; a gap generator | deploy if its radius is already boosted, or its house's spare power exceeds the extra power drain of its type (low); else nothing |
| B5 | *g* is deploy; an absorbing building holding units it has absorbed, a building with garrison occupants, or a factory | deploy |
| B6 | *g* is deploy | nothing |
| B7 | *g* is attack; it has a weapon in its first slot, and that weapon's projectile cannot hit ground targets, or the building's type is a pulse cannon (it uses its own targeting), or the building is being sold | nothing |
| B8 | *g* is attack; the target's type is immune to damage | no-move |
| B9 | *g* is move; a factory of aircraft (reachable only when the factory is movable, that is, also a repair depot or a type that undeploys into a vehicle) | move |
| B10 | *g* is move; a factory of infantry or vehicles; the target's cell is not reachable within the building type's movement zone (low) | no-move |
| B11 | *g* is move; a factory of infantry or vehicles; the target's cell is a bridge cell | move |
| B12 | *g* is move; a factory of infantry or vehicles; the target's cell's passability category is not *open* | no-move |
| B13 | Otherwise | *g* |

A building's *g* is *move* only with force-move held (row G7), since no other row of table G
gives it.

**Table GC: generic result *c* on a cell** (claim 9). The first row whose condition holds gives *c*.

| Row | Condition | *c* |
|---|---|---|
| GC1 | The actor is berserk, melee-linked while its type lacks the keep-acting flag, being erased or phasing in | nothing |
| GC2 | The actor is not own | nothing |
| GC3 | Force-move and force-fire held, the actor movable, and able to fire or of an engineer type | patrol if the cell holds a route point of route mode (or the actor is in a route-following state that this spec has not identified, low), else area guard |
| GC4 | A weapon is in the weapon slot it currently uses (the weapon itself, not *armed*: Terms), and force-fire held or the cell attackable (below), and the actor movable or the cell centre in range of the weapon it would use against the cell | attack |
| GC5 | The weapon in the slot it currently uses (the weapon itself, as in row GC4) is a terrain-fire weapon, the cell holds a rock overlay, and the actor movable or the cell centre in range | attack (low) |
| GC6 | Movable or a rally-capable building, and the cell outside the usable area | no-move |
| GC7 | Movable or a rally-capable building, and force-select held | move |
| GC8 | A rally-capable building, and force-move held | move |
| GC9 | Movable, and either fog of war is considered for this cell (low), or the actor's entry outcome for the cell (S-0209) is *clear* or *cloaked blocker*, or the actor is a building that undeploys into a resource-gathering vehicle (low) | move |
| GC10 | Movable, its type travels underground, and a relaxed form of the entry test passes (low) | move |
| GC11 | Movable or a rally-capable building | no-move |
| GC12 | Otherwise | nothing |

*Attackable cell* (row GC4, when force-fire is not held): both of these hold.
(i) The cell's overlay type is a legal target (under fog of war, the remembered overlay, low); or
the cell is a destroyable-cliff cell and the warhead of the weapon lacks the fire flag; or, under
fog of war, the cell shows a remembered building of a house not allied with the actor, or of no
house, whose type is a legal target.
(ii) The cell has no overlay; or a live veinhole monster stands there; or the cell belongs to a
house not allied with the actor and its overlay is a wall that the warhead can damage (the warhead
has the wall flag, or it has the wood flag and the overlay's armour class is wood).

The shroud change of claim 9 is applied to *c* for infantry, vehicles and aircraft; *c*′ below is
the result.

**Table IC: infantry on a cell.**

| Row | Infantry and cell | Final action |
|---|---|---|
| IC1 | *c*′ is move; not a jumpjet type; not an in-place deployer in its deployed state; the cell is a tunnel entrance | enter tunnel if it can use the tunnel, else no-enter tunnel |
| IC2 | Engineer; fog of war is on, and the cell shows a remembered building under fog that is a bridge repair hut | repair building if its bridge is destroyed, else no-repair (low) |
| IC3 | Engineer; fog of war is on, and the cell shows a remembered repairable building under fog | capture (low) |
| IC4 | Healer; *c*′ is attack | area guard |
| IC5 | A jumpjet type; *c*′ is move or no-move; a tunnel cell | nothing |
| IC6 | An in-place deployer in its deployed state; *c*′ is move | no-move |
| IC7 | An in-place deployer in its deployed state; *c*′ is attack; its type does not fire while deployed, or the cell centre is out of range of its first weapon | no-move |
| IC8 | *c*′ is nothing; not a jumpjet type; movable | no-move |
| IC9 | Otherwise | *c*′ |

**Table VC: vehicle on a cell.**

| Row | Vehicle and cell | Final action |
|---|---|---|
| VC1 | Healer; *c*′ is attack | idle |
| VC2 | *c*′ is move; the cell is not shrouded; and either the cell has the ore land type (or, under fog of war, a remembered ore overlay, low) and the vehicle is a harvester, or the cell has the weed land type (or a remembered fully grown weed overlay, low) and the vehicle eats weeds, or the cell has the ore land type (or a remembered ore overlay) and the vehicle's type gathers resources and processes them itself | harvest |
| VC3 | *c*′ is move; the cell is a tunnel entrance | enter tunnel if it can use the tunnel, else no-enter tunnel |
| VC4 | An in-place deployer that is deployed, deploying or undeploying; *c*′ is move | nothing |
| VC5 | An in-place deployer in its deployed state; *c*′ is attack; its type does not fire while deployed, or the cell centre is out of range of its first weapon | no-move |
| VC6 | *c*′ is attack; the vehicle fails the deploy check | nothing |
| VC7 | Otherwise | *c*′ |

**Table ACC: aircraft on a cell.** *c*′ is attack and the aircraft has no weapon in its first slot:
*nothing*. Otherwise *c*′. Low.

**Table BC: building on a cell.** No shroud change applies. All rows: low.

| Row | Building and cell | Final action |
|---|---|---|
| BC1 | Its type is invisible in the game | nothing |
| BC2 | Its type undeploys into a vehicle and gathers and processes resources itself; *c* is move; the cell has the ore land type and is not fogged | harvest |
| BC3 | *c* is attack; no weapon in its first slot | attack |
| BC4 | *c* is attack; its projectile can hit ground targets, its type is not a pulse cannon, and it is not being sold | attack |
| BC5 | An own rally-capable building | move (sets its rally point) |
| BC6 | *c* is attack | nothing |
| BC7 | *c* is move; not movable | nothing |
| BC8 | Otherwise | *c* |

So an own rally-capable building gets *move* over every cell unless it has an *attack* to make,
with or without force-move.

**Table M: action to order** (claim 10). Every row creates a behaviour order (S-0213, claim 5)
unless the row says otherwise. "Object" means the clicked object, "cell" the clicked cell.
Actions not listed give no order. A row's "Needs" must hold at the click, or no order is made; a
dash means that no such requirement was found (low).

Part 1, cases with no order or a changed action, which take precedence over part 2:

| Actor | Action | Result |
|---|---|---|
| Infantry, vehicle or aircraft that is berserk, or melee-linked while its type lacks the keep-acting flag | any, except *tote* for a berserk carrier aircraft, which gives its order below (claim 15) | no order (claim 15) |
| Infantry | capture, damaging entry or repair building | as for *capture* |
| Infantry | heal or disarm bomb | as for *attack* |
| Infantry | attack, heal or disarm bomb, with the actor itself as the target | no order |
| Vehicle | heal or repair vehicle | as for *attack* |
| Vehicle | attack, no-move, heal or repair vehicle, with the actor itself as the target | no order |
| Infantry or vehicle in-place deployer in its deployed state | no-move on a cell | no order |
| Aircraft, on an object | no-move | no order |
| Aircraft, on an object | enter | behaviour *enter*, destination the object |
| Aircraft, on an object | deploy | behaviour *unload* |
| Aircraft, on an object | tote | behaviour *move*, destination the vehicle (low) |
| Building | any | part 3 instead of part 2 |

Among the part 1 rows for one actor, a "no order" row takes precedence over an "as for" row.

Part 2, infantry, vehicles and aircraft:

| Action | Behaviour | Target | Destination | Needs |
|---|---|---|---|---|
| move, no-move | move | nothing | claim 11 | movable |
| attack-move to a place | attack move order (S-0213, claim 6) | nothing | as for move | movable |
| attack | attack | object, or cell (on a cell with a live veinhole monster: the monster, low) | nothing | able to fire |
| attack-move on a target | attack move order (S-0213, claim 6) | object or cell | nothing | able to fire |
| enter | enter | nothing | object (a unit or building) or cell | movable |
| capture | capture | nothing | object or cell | movable |
| recycle | eaten | nothing | object | movable |
| sabotage | sabotage | nothing | object or cell | — |
| area guard (object) | area guard; for a charge planter facing an enemy building, sabotage with the object as destination instead | object | nothing | able to fire and movable |
| area guard (cell) | area guard | cell | nothing | movable, and armed or of an engineer type |
| patrol | patrol | nothing | nearby usable cell | — |
| deploy, area attack | unload | nothing | nothing | — |
| bomb, demolish, airstrike | attack | object | nothing | able to fire |
| harvest | harvest | nothing | cell | — |
| enter tunnel | move | nothing | the cell next to the tunnel entrance on its open side (low) | — |
| idle | guard, or area guard if the actor's first weapon heals | the actor itself | nothing | — |
| detonate | not a behaviour order: a detonate order (S-0213, claim 11) | | | |

The *idle* row looks at the first weapon slot, while *healer* (Terms) looks at the weapon the
actor would use by default. The two differ only for a type whose default weapon is not its first.

Part 3, buildings (low):

| Action | Order |
|---|---|
| attack | behaviour *attack*, target the object or cell |
| deploy | a primary order (S-0213, claim 13) if the building is a factory; and a behaviour order *unload* for each of these that holds: it is a gap generator; it is an absorbing building; it has garrison occupants |
| detonate | a detonate order (S-0213, claim 11) |
| move, on an object | for a factory of infantry, vehicles or aircraft, a guard-spot order (S-0213, claim 20) chosen around the object's cell; for any other building, a guard-spot order naming the object |
| move or harvest, on a cell | for a rally-capable building or one that undeploys into a vehicle, a guard-spot order (S-0213, claim 20) chosen around the cell; for one that undeploys into a vehicle, also a sell order (S-0213, claim 12), with which it packs up |

Ordering consequences:
- Orders from one click are issued in selection order, so with more than 128 selected objects the
  last ones get no order (S-0213, claim 2). Because they are issued in a row with the same
  behaviour, target and destination, the destinations of a group move are spread (S-0213,
  claim 7).
- Because each object resolves the click itself (claim 2), objects that would not get the
  representative's action are not forced into it. A tank in a group led by an engineer does not
  try to capture.

## Edge cases and quirks

- **Damaging entry.** An engineer's action on an enemy capturable building above the capture
  threshold is *damaging entry*. It produces a capture order like any other (table M). What the
  engineer does on arrival belongs to the special-abilities specs.
- **Full health is exact.** Rows I4, I5, I12 and value *b* of table V test *h* = 1 and *h* < 1
  exactly (Formulas F2). Hit points are integers, so this is not a defect. No switch.
- **No-move still moves.** A *no-move* action produces a move when a usable cell is found, and on
  an object even when none is (claim 11), except where part 1 of table M gives no order. The
  cursor suggests otherwise. This is original
  behaviour. No switch, since it is not a defect of game state.
- **The harvester-truce exception of row I15 never applies.** The original means to stop a
  vehicle thief from hijacking a harvester of the rules' harvester list while the game's
  harvester-truce setting is on, but the exception never takes effect: whatever the target's
  type and the truce setting, a vehicle thief gets *capture* on another house's harvester.
  `quirks.hijack_truce_harvester_check` (default: the original, no exception). With the switch
  off, a harvester whose type is in the harvester list gives *select* instead while the truce
  setting is on.
- **Permanently disguised targets pass the disguise rule** (claim 6). A target whose type is
  permanently disguised passes the disguise rule as if it were not disguised at all, while a
  temporarily disguised one does not. `quirks.permadisguise_attack_cursor` (default: the
  original). With the switch off, a permanently disguised target is treated like any other
  disguised one.
- **Berserk actors show actions they cannot use** (claim 15). Some infantry and vehicle rows do not
  look at the generic result, so a berserk engineer, for example, can show *capture* over an enemy
  building while it is the representative, although the click gives it no order. This is
  presentation only and needs no switch.
- **Inputs that never occur.** No aircraft has the generic result *area guard* on an object (row
  G6 excludes aircraft), and no building has the generic result *move* without force-move (row
  G7). Tables AC and B therefore have no rows for those inputs. No unarmed actor has a generic
  *attack*, on an object or on a cell, except an infiltrating infantry type (claim 6 (c), rows GC4
  and GC5). Every derived value that can be *attack* (*g*₁, *g*₂, *k*, *b*, *n*, *q*, *c*′)
  inherits it from that generic *attack*, so no kind table needs a row for an unarmed actor told
  to attack; the unarmed infiltrator is handled by rows I24 and I25. Force-fire never gives an
  unarmed actor an *attack*, unless it is an infiltrator; it can still change an unarmed actor's
  action through rows that read force-fire directly (for example value *e*, row I8 and row G6).
- **Type-select interval** (claim 12). Departure: the original times the double press with the
  machine's clock. Our engine keeps the 500 ms threshold but measures it between the time stamps
  of its own input events. Selection is not game state, so this never reaches the simulation. No
  switch.
- **Representative distance.** Formulas F1. Ties between selected objects are rare. Our engine
  computes the same value with the table square root of S-0204 (`quirks.table_root_distance`
  there).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Infantry type `Engineer` | engineer (tables I, IC; rows GC3 and the cell area guard of table M) | boolean (S-0003) | no |
| Infantry type `Agent` | spy (row I24) | boolean (S-0003) | no |
| Infantry type `Infiltrate` | infiltrator (claim 6, rows I24, I25) | boolean (S-0003) | no |
| Infantry type, `VehicleThief` | vehicle thief (rows I14, I15) | boolean (S-0003) | no |
| Infantry type `C4` | charge planter (claim 6, row I16, table M) | boolean (S-0003) | no |
| Infantry and vehicle types, `VeteranAbilities` and `EliteAbilities` | the charge ability (row I16) and the crushing ability (value *k* of table V): the list entries named after the flags `C4` and `Crusher` | list (S-0003) | empty |
| Infantry type `Ivan` | bomb planter (row I19) | boolean (S-0003) | no |
| Infantry type `Occupier` | can garrison (row I8, with the building's flag below) | boolean (S-0003) | no |
| Infantry type `Deployer`, vehicle type `IsSimpleDeployer` | in-place deployer (tables I, V, VD, IC, VC, M) | boolean (S-0003) | no |
| Infantry and vehicle types, `UndeployDelay` | undeploy delay (rows I22, I23; deploy readiness): present when 0 or more | integer (S-0003) | −1 (none) |
| Infantry type `BalloonHover` | balloon-hovering type (value *x* of table I) | boolean (S-0003) | no |
| Infantry and vehicle types, `JumpJet` | jumpjet type (claim 11, rows IC1, IC5) | boolean (S-0003) | no |
| Infantry, vehicle, aircraft and building types, `PreventAttackMove` | eligibility for the gesture (claim 5) | boolean (S-0003) | no |
| Infantry, vehicle, aircraft and building types, `AttackFriendlies` | attack-friendlies flag (claim 6) | boolean (S-0003) | no |
| Infantry, vehicle, aircraft and building types, `AttackCursorOnFriendlies` | attack-cursor-on-friendlies flag (claim 6) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft types, `Pushy` | keep-acting flag of a melee-linked object (Terms, rows G1, GC1) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft types, `MoveToShroud` | may move into shroud (claims 9, 11) | boolean (S-0003) | yes |
| Infantry, vehicle and aircraft types, `PermaDisguise` | permanently disguised (claim 6) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft types, `MovementZone` | flying and underground zones (claim 11, value *n* of table V); a building type's zone (row B10) | name (S-0003) | — |
| Aircraft types, `Spawned` | spawned aircraft type (Terms, *movable*) | boolean (S-0003) | no |
| Vehicle and aircraft types, `SpawnsNumber` | launch count (Terms, *movable*) | integer (S-0003) | 0 |
| Building types, `Repairable` | repairable building (rows I3 to I7, IC3) | boolean (S-0003) | yes |
| Infantry and vehicle types, `DeployFire` | fires while deployed (rows I21, V3, IC7, VC5); deploy weapon (row VD4) | boolean (S-0003) | no |
| Weapon `AreaFire` | deploy weapon fires over an area (value *v* of table I) | boolean (S-0003) | no |
| Vehicle types, `DeployToFire` | deploy-to-fire flag (Terms, *deploy check*) | boolean (S-0003) | no |
| Vehicle types, `Crusher` | can crush (value *k* of table V) | boolean (S-0003) | no |
| Vehicle types, `DeploysInto` | deploys into a building (row VD2; deploy readiness) | building type name | none |
| Vehicle types, `NonVehicle` | does not count as a vehicle (Terms) | boolean (S-0003) | no |
| Vehicle types, `IsTrain` | train (row I14) | boolean (S-0003) | no |
| Vehicle and aircraft types, `Passengers` | transport (values *x* of table I and *q* of table V, rows VD4 to VD8, AC7): more than 0 | integer (S-0003) | 0 |
| Building types, `UndeploysInto` | vehicle-like building, with a 1 × 1 foundation (Terms); undeploys into a vehicle (Terms, *movable*; rows GC9, BC2; table M) | vehicle type name | none |
| Aircraft types, `Dock` | docking buildings (row AC10) | list of building types | empty |
| Building type `Capturable` | capturable (claim 6, rows I7, I24) | boolean (S-0003) | no |
| Building type `Spyable` | spyable (claim 6, row I24) | boolean (S-0003) | no |
| Building type `CanC4` | accepts charges (claim 6, rows I16, I18) | boolean (S-0003) | yes |
| Building type `CanBeOccupied` | garrisonable (row I8, engineer-friendly building; Terms, *armed*) | boolean (S-0003) | no |
| Building type `CanOccupyFire` | garrison occupants may fire from inside (Terms, *armed*) | boolean (S-0003) | no |
| Building type `BridgeRepairHut` | bridge repair hut (rows I3, IC2) | boolean (S-0003) | no |
| Building type `Hospital`, `Armory` | value *e* of table I, row I4 | boolean (S-0003) | no (both keys) |
| Building type `Grinding` | recycling building (rows I5, I26, value *n* of table V) | boolean (S-0003) | no |
| Building type `UnitRepair` | repair depot (value *n* of table V, row AC10; rally-capable) | boolean (S-0003) | no |
| Building type `Bunker` | tank bunker (value *n* of table V; not rally-capable) | boolean (S-0003) | no |
| Building type `Factory` | factory and the kind it builds (rows B3, B5, B9 to B12, table M; rally-capable) | type kind name | none |
| Building type `WeaponsFactory` | vehicle factory (row AC2) | boolean (S-0003) | no |
| Building type `ConstructionYard` | construction yard (Terms, *movable*; deploy readiness) | boolean (S-0003) | no |
| Building type `InvisibleInGame` | invisible in the game (rows B1, BC1, I16, I18) | boolean (S-0003) | no |
| Building type `TogglePower` | can be switched off (claim 13) | boolean (S-0003) | yes |
| Building type `Powered` | needs power (claim 13) | boolean (S-0003) | no |
| Building type `Cloning` | rally-capable (Terms) | boolean (S-0003) | no |
| Building type `UnitAbsorb`, `InfantryAbsorb` | absorbing building (row B5, table M) | boolean (S-0003) | no (both keys) |
| Building type `GapGenerator` | gap generator (row B4, table M) | boolean (S-0003) | no |
| Building type `ExtraPower` | the extra power drain of row B4 (the drain part of a negative value, low) | integer (S-0003) | 0 |
| Building type, `EMPulseCannon` | pulse cannon (rows B7, BC4) | boolean (S-0003) | no |
| House type `MultiplayPassive` | passive neutral house (engineer-friendly building) | boolean (S-0003) | no |
| Weapon `SabotageCursor` | demolition-cursor flag (row I17) | boolean (S-0003) | no |
| Weapon `MigAttackCursor` | air-strike-cursor flag (row I18) | boolean (S-0003) | no |
| Weapon `TerrainFire` | terrain-fire weapon (claim 6, row GC5) | boolean (S-0003) | no |
| Projectile `AG` | can hit ground targets (rows B7, BC4) | boolean (S-0003) | yes |
| Warhead `Wall`, `Wood` | can damage walls, wooden walls (attackable cell) | boolean (S-0003) | no (both keys) |
| Warhead `IvanBomb` | warhead that plants bombs (claim 6) | boolean (S-0003) | no |
| Warhead `Sonic` | sonic warhead (claim 6) | boolean (S-0003) | no |
| Warhead `Fire` | excludes destroyable-cliff cells from the attackable cells | boolean (S-0003) | no |
| Overlay type, `IsARock` | rock overlay (row GC5) | boolean (S-0003) | no |
| Aircraft type `Carryall` | carrier aircraft (rows AC2, AC3) | boolean (S-0003) | no |
| Vehicle type `Harvester` | harvester (row VC2) | boolean (S-0003) | no |
| Vehicle type `Weeder` | eats weeds (row VC2) | boolean (S-0003) | no |
| Vehicle and building types, `ResourceGatherer` and `ResourceDestination` | gathers and processes resources itself: both set (rows VC2, BC2); a resource-gathering vehicle: the first (row GC9) | boolean (S-0003) | no (both keys) |
| Object type `Immune` | immune target (rows I27, V6, AC5, AC9, B8) | boolean (S-0003) | no |
| Object type `LegalTarget` | legal target (claim 6 (a), attackable cell) | boolean (S-0003) | yes |
| Object type `Bombable` | accepts bombs (row I19) | boolean (S-0003) | yes |
| Rules `[General]`, `EngineerCaptureLevel` | *c* of row I7 | decimal or percent (S-0003), stored in single precision | 1.0 (a health ratio of 1) |
| Rules `[General]` `AttackCursorOnDisguise` | attack cursor on disguised units (claim 6) | boolean (S-0003) | no |
| Rules, combat-damage section, `CanDetonateTimeBomb`, `CanDetonateDeathBomb` | row G3 | boolean (S-0003) | no (both keys) |
| Rules, combat-damage section, `TreeTargeting` | tree targeting (claim 6) | boolean (S-0003) | no |
| Rules `[General]` `AllowShroudedSubteranneanMoves` | underground moves into shroud (claim 11) | boolean (S-0003) | no |
| Rules `[General]` `HarvesterUnit` | the harvester list of the row I15 quirk | list of vehicle types | empty |

The defaults in the Absent column are the values the original gives a field when the type (or the
rules data) is created. They are engine constants, read from the original's program rather than
from stock data (confidence: medium). Each rules layer that omits a key keeps the value already in
force (S-0006).

The harvester-truce setting of row I15 is a setting of the game, which the original takes from the
multiplayer settings of the rules data. How it reaches a skirmish game is not settled (Open
questions); with the quirk on (the default) it changes nothing here. The game option that lets
construction yards be packed up again (Terms, *movable*) is a lobby setting, not a rules key.

## Measurement scenarios

All scenarios run under the measurement setup: a skirmish on a small purpose-made flat map with
one human player and one computer player whose start is far away. Objects are pre-placed by the
map. A recorder logs each object's behaviour, target and destination per tick, and each order
issued. The cursor shape is observed on screen and used only to read the representative's action.
Each test is done once, because no random draw of the scenario random stream is involved. Where a
scenario says "alone", that object is the only one selected, so that it is the representative.

**Scenario T1: representative (claim 1).** Select an unarmed engineer and an armed tank, with the
engineer nearer to an enemy building. Hover over the building: expected the tank's action
(attack), since rank 5 beats rank 2. Then select an armed ground vehicle and an armed amphibious
vehicle (both rank 5), with the ground vehicle nearer to a water cell, and hover over the water:
expected the ground vehicle's action (no-move), although the amphibious vehicle would show move.
Move the amphibious vehicle nearer than the ground vehicle and hover again: expected move.
Falsified if the engineer's action (capture) shows in the first case, or the farther vehicle's
action shows in either of the others.

**Scenario T2: per-object resolution and selection (claims 2, 3, 10, 12, 14).**
- Setup: an own engineer, tank and medic; an enemy capturable building; own units of one further
  type, some on the screen and some off it.
- Select the engineer, the tank and the medic, and click the enemy building. Expected: the
  engineer gets a capture order (behaviour capture, destination the building), the tank an attack
  order, and the medic, whose action is *idle*, an area-guard order on itself (table M). Only one
  acknowledgement is heard. Falsified if the tank tries to capture, the engineer attacks, the
  medic moves towards the building, or more than one acknowledgement is heard.
- Select the tank and two own units of the further type that are on the screen, and click an
  empty, visible cell. Expected: each gets a move order, exactly one move-target marker appears
  for the click, and one acknowledgement is heard (claim 14). Falsified if more than one marker
  appears for the click, or more than one acknowledgement is heard.
- Start with an own unit selected. With force-select held, click an own unselected unit, then
  click it again. Expected: it is added, then removed. Falsified if the selection does not change,
  or changes otherwise.
- Select an enemy unit by clicking it, then click an own unselected unit with force-select held.
  Expected: the own unit replaces the enemy unit in the selection (*select*, since the first
  selected object is not own). Falsified if both end up selected.
- Hold the type-select key and click one own unit of the further type, once. Expected: every
  object of that type on the screen is selected, and none off it. Falsified if only the clicked
  unit is selected, or off-screen ones are too.
- Press the type-select key twice with about 200 ms between the presses, and later twice with
  about 1 s between them. Expected: the first pair selects that type across the whole map, the
  second only on the screen. Falsified if the two pairs give the same selection while objects of
  the type exist off screen. (200 ms and 1 s lie clearly on either side of the 500 ms constant.)
- With the tank selected, hold route mode and click two cells. Expected: no order is recorded for
  the tank until route mode ends (claim 3). Falsified if a move order is recorded at the first
  click.
- *Part B, the acknowledgement draw (claim 14; needs the random-state recorder reading the
  unsynchronised random stream, with scripted input for the clicks).* *Setup:* own units of a
  type that has at least two selection responses, and a click target for which the tables give
  those units an order outside the kinds that have their own acknowledgement (harvest, attack,
  move and attack move, enter, capture, unload). *Procedure:* the script selects one such unit and
  clicks the target, then selects three and clicks it again, and then does the same with a move
  click, while the recorder logs the unsynchronised stream's position each tick. *Observable:*
  the position's advance across the click's tick, the response heard, and the scenario random
  stream's position. *Expected:* each click of the first kind advances the unsynchronised
  stream's position by one draw however many units are selected, and the response heard is the one
  numbered by that draw modulo the number of the type's selection responses; a move click gives
  its own acknowledgement and advances it by none; the scenario random stream's position does not
  change for any click. *Falsified by:* one advance per selected unit, an advance for the move
  click, a response that does not match the draw, or any change of the scenario stream. The
  marker makes no draw of either stream and is checked by eye only.

**Scenario T3: modifiers and attack-move (claims 4, 5).** One tank, one friendly tank, one enemy
tank out of range, empty cells. With force-move on the enemy: the tank moves to it without
firing. With force-fire on the friendly tank: an attack order. With force-move and force-fire on
the friendly tank: area guard on it, and it follows the friendly tank. With force-move and
force-fire on a cell: area guard on the cell. With the gesture on a far cell: a move with the
attack-move memory (it engages enemies met on the way). Repeat the gesture with a building or an
aircraft added to the selection: expected a plain move (no attack-move). If the command bar is
available, repeat with its attack-move button instead of the gesture; then press the button,
click a far cell (attack-move expected), and click another far cell without the gesture (a plain
move expected, since the mode ended with the first click); then press the button, clear the
selection, select the tank again and click a far cell (a plain move expected). Row G3: an own
bomb planter, alone, force-fires at the own tank (rows G8 and I19 give *bomb*), so that its bomb
is attached to the tank; then select the tank alone and click it,
once with a named test rules file that turns both bomb-detonation settings on (expected: a
detonate order, and the bomb goes off at once) and once with a file that turns both off
(expected: no detonate order, and the bomb goes off only when its timer ends). Falsified if any
outcome differs.

**Scenario T4: attack rule (claim 6).** An own tank, alone, against: an enemy tank (attack), a
second own tank without force-fire (select), a tank of an allied computer house without force-fire
(no-move, by rows G9 and V4), an object whose type is immune (no-move cursor), a disguised
enemy spy, with and without force-fire, a tree, and an own unit whose type travels underground
(stock or test type), first on the surface with force-fire held (expected: attack, the baseline
that shows the unit can be targeted), then, after it has been ordered to move and is travelling
underground, again with force-fire held. A named test rules file clears the spy
type's permanent-disguise flag, makes the tree's type not a legal target, and is used in four
variants: the general attack-cursor-on-disguise setting off or on, and tree targeting off or on.
Expected: with the disguise setting off, the spy shows no attack without force-fire, and
force-fire attacks it; with the setting on, the spy shows attack without force-fire. The tree
shows attack only with tree targeting on or with force-fire. While the own unit travels underground, no
click records an attack order whose target is that unit (a force-fire click there may attack the
cell, by table GC). Falsified if any of these differs.

**Scenario T5: infantry rows (claim 7, table I).**
- Setup: own infantry, each tried alone: an engineer, a spy, a vehicle thief, a charge planter, a
  bomb planter, a medic, a garrison-capable infantry, an unarmed infiltrator that is not a spy,
  and a deployer infantry, deployed and not. Targets: an enemy capturable building at full
  health and at a health ratio below the capture field (a named test rules file sets the capture
  field to 50 %), an enemy building that is neither capturable nor spyable, an own damaged
  building, an own undamaged building, a bridge repair hut with its bridge destroyed and intact, a
  neutral garrisonable building, an allied hospital (tried with a plain armed infantry unit, hurt
  and unhurt, since a healer's own rows come first), an allied hospital whose type is repairable
  (a named test rules file makes it so if needed), at full health and damaged, tried with the
  engineer hurt and unhurt (the map places the hurt one below its full hit points), an enemy
  vehicle, an enemy harvester (also with the harvester-truce setting on, if a skirmish can set it),
  an allied transport standing and moving (tried with the plain infantry unit), an own recycling
  building.
- Observable: the cursor for each pair, and the order recorded on a click.
- Expected: the actions of table I, mapped by table M. In particular: the engineer gets capture
  below the capture field and damaging entry above it, repair building on the damaged own
  building and no-repair on the undamaged one; the vehicle thief hijacks the harvester in both
  cases; with force-fire held, the unarmed infiltrator shows no-move over the enemy building that
  it cannot infiltrate, and a click moves it there (row I25); the hurt plain infantry gets enter on the hospital and the
  unhurt one no-enter; the hurt engineer gets enter on the allied hospital whatever the
  hospital's health (row I4), while the unhurt engineer gets no-repair on the full-health hospital
  (row I5) and repair building on the damaged one (row I6); the transport gives enter when
  standing and no-enter when moving.
- A plain infantry unit whose type does not deploy, alone, clicked on itself. Expected: the
  no-move cursor (row I28) and a recorded move order.
- Falsified by any differing action or order, in particular if the unarmed infiltrator shows no
  action and gets no order, the self-click records no order, or the hurt engineer gets repair
  building or no-repair on the hospital (which would mean row I4 tests the hospital's health).

**Scenario T6: vehicle, aircraft and building rows (claims 8, 10, 15, tables V, VD, AC, B, M).**
Each actor is tried alone. The cursor gives its action, and the recorder its order on a click.
- A repair vehicle against a damaged own tank, a full-health own tank and an enemy tank. Expected:
  repair vehicle on the damaged tank (and an attack order on a click), select on the full-health
  one, and over the enemy the
  fallback move (the no-move cursor), after which a click records a move toward the enemy.
- An unarmed crushing vehicle against enemy infantry. Expected: never *attack*, since an unarmed
  vehicle never has the generic result *attack* (claim 6 (c)): *select* if the enemy
  infantry can be selected (row G10), otherwise the fallback move, shown as no-move (rows G11
  and V4). A crushing tank that is
  paralysed by a stock weapon with that effect, with enemy infantry in range of its weapon.
  Expected: move (value *k* of table V, low).
- A deploy-into-building vehicle clicked on itself on open ground, on a cramped spot, and once
  while route mode is held. Expected: deploy, no-deploy, no-deploy.
- An empty transport and a loaded one, each clicked on itself. Expected: the empty one no-deploy
  and no order, the loaded one deploy and an unload order.
- A vehicle clicked on an own repair depot. Expected: enter, and an enter order.
- A carrier aircraft and an allied tank. Expected: tote, and a move order with the tank as
  destination.
- An aircraft and its own landing pad whose type lacks the repair-depot flag (a named test rules
  file clears it if needed), and the same aircraft and an own repair depot that is not in its dock
  list. Expected: select over the landing pad, since row AC10 needs the repair-depot flag, and
  no-enter over the repair depot. Then, with a named test rules file that puts the repair depot
  in the aircraft's dock list: enter over the depot when it is free, no-enter when it holds a unit.
- A barracks clicked on a cell without and with force-move. Expected: move in both cases, and a
  guard-spot order each time.
- A vehicle factory that is not primary, clicked on itself. Expected: deploy, and a primary order.
- A berserk own unit (made berserk by a stock weapon with that effect), clicked on a cell and on an
  enemy. Expected: no order on either click.
- A neutral garrisonable building with no weapon of its own, garrisoned by own infantry that can
  fire from inside, whose type lets occupants fire. Over an enemy tank in range: expected attack,
  and an attack order. The same with a named test rules file that clears the type's
  occupants-fire flag: expected select, and no order. Then the garrisoned building and an own
  unarmed engineer selected together, hovered over the enemy tank: expected the building's
  action (attack), since rank 3 beats rank 2.

Falsified if any listed actor gets a different action or a different order. Results that would
show a specific error: an unload order for the empty transport (table VD), no order for the
barracks without force-move (row BC5), an order for the berserk unit (claim 15), and no attack
for the garrisoned building with the flag on (Terms, *armed*).

**Scenario T7: cells (claims 9, 11, tables GC, IC, VC).**
- A tank and a cell inside a cliff ring (unusable): expected the no-move cursor, and a click moves
  the tank to the nearest usable cell next to the ring. Falsified if the click leaves the tank in
  place while a usable cell exists nearby.
- A tank clicked onto an enemy vehicle standing in the open with force-move: destination the
  vehicle itself. Falsified if the destination is a cell.
- A harvester over visible ore (harvest) and over shrouded ore (move, and a move order). Falsified
  if the shrouded ore gives harvest.
- A tank over a shrouded cell inside the usable area: expected move, and a move order. The same
  with a named test rules file that clears the may-move-into-shroud flag of the tank's type:
  no-move, and no order. Falsified if the first
  gives no-move or the second gives an order.
- On a test map whose local area leaves a strip of legal cells outside the usable area (as in
  S-0212's scenario P11): a tank over a cell in the strip shows no-move, and a click orders a move
  to a usable cell; with force-fire held over the same cell the tank gets no order. Falsified if
  either differs.
- A jumpjet infantry over a tunnel cell (nothing). A tunnel-using unit over a tunnel entrance
  (enter tunnel). Falsified if either differs.
- On a theater whose tiles include the destroyable-cliff role: an armed tank over a
  destroyable-cliff cell shows attack, and a test weapon whose warhead has the fire flag shows
  move or no-move there. The same tank over a cell holding the tile after the role's start also
  shows attack. Falsified if either does not show attack.
- On a test theater whose destroyable-cliff key is absent (the role is unset): the armed tank
  over an empty cell of tile 0 shows move. Falsified if it shows attack, which would mean the
  unset start is −1 rather than −2.

**Scenario T8: sidebar modes (claim 13).** With nothing selected: sell mode over an own building,
an own vehicle on the repair depot, an own vehicle on open ground, an own wall; repair mode over a
damaged own building twice; power mode over an own power-using building twice. Expected: building
sold, no-sell over both vehicles (nothing happens on a click), wall sold, repair on and then off,
and the building powered off and then on again. Falsified if any of these differs; in particular, a vehicle sold by a sell-mode
click would show where a sell unit action comes from.

## Open questions

- Tables AC and B, and the building cell table BC, are low confidence throughout and are confirmed
  only by Scenario T6.
- Rows IC2 and IC3 (an engineer clicking a cell under fog of war) depend on what fog shows, and
  are not yet confirmed. When the original considers fog for a cell (rows GC9, IC2, IC3, VC2) is
  not identified.
- The flag of a team's type that lets a moving team member be entered (value *x* of table I), and
  the route-following state of row GC3, are not identified.
- Where a *sell unit* action would come from, if anywhere (claim 13).
- How the harvester-truce setting of row I15 is set in a skirmish game.
- The default physical keys for the three modifier roles are not yet confirmed, nor whether the
  command bar with its attack-move button is available in a default skirmish.
- In campaign missions an extra rule changes actions on objects under shroud. It is not covered,
  since these claims cover skirmish games.
- Which disguises are overlay types (scenery check), and how often a disguise belongs to no house
  in stock data.
- Several low-confidence details are stated only by their effect and are not identified: the rule
  by which a cell counts as lying at the edge of a tunnel (Terms, *tunnel cell*); the bridge,
  water and building-cell cases in which a vehicle may not unload (Terms, *deploy readiness*); the
  neighbour step that anchors a foundation wider or deeper than 2 cells (row VD2); and the relaxed
  entry test of row GC10.
- Row BC3: a building with no weapon in its first slot gets *attack* on a cell. Since row GC4
  needs a weapon in the slot the building currently uses, this can only concern a building type
  that switches between weapons. It is surprising, low, and no scenario tests it.
- Whether a permanent takeover that leaves no controlling object makes an object
  *mind-controlled* in the sense of Terms (it matters for packing up a construction yard and for
  deploying into one).
- Whether a skirmish with stock data can produce an active ion storm, which the zone rule of
  claim 11 depends on.
