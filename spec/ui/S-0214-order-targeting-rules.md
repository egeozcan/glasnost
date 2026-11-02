---
id: S-0214
title: Order targeting rules (what a click means for each kind of object, and group orders)
topic: ui
revision: 66
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0081, Q-0277, Q-0278, Q-0286, Q-0392, Q-0450, Q-0452, Q-0742, Q-1754, Q-1987, Q-1988, Q-2140]
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
(S-0213). This spec also covers battlefield pointer images, ordinary band-box selection,
mouse-button gestures, the type-select role, control groups and the distinction between a
right click and right-drag scrolling.
Skirmish only; network games and campaigns are out of scope.

## Terms

Glossary terms are used as defined there, in particular *house*, *local player*, *object*,
*type*, *field*, *building*, *infantry*, *vehicle*, *aircraft*, *cell*, *lepton*, *shroud*,
*fog of war*, *disguise*, *jumpjet*, *movement zone*, *locomotion class*, *land multiplier*,
*height above the surface*, *armour class*, *AI house*, *unsynchronised random stream*,
*departure*, *quirk switch*, *measurement setup* and *recorder*.
*Order*, *actor*, *behaviour order*, *selection*, *route mode*, *planned route*, *unit or
building*, *inside a tunnel*, *attack-move memory*, *planted bomb*, *paralysed*, *berserk* and
*primary factory* are defined in S-0213. *Behaviour*, the behaviour names, *contact*, *team
member* and *loaner* are S-0215's. *Reference point* and *table square root* are S-0204's. The
*usable area* is S-0009's (S-0211 uses it); the *passability category* of a cell is S-0211's. The
*entry outcome* is S-0209's. The *reach region* of a movement zone is S-0212's. The
*destroyable-cliff role* of a theater's tiles is S-0010's.

- **Action.** The result of resolving a click for one selected object: one of the actions of
  table A, or *nothing*. The cursor image is presentation, specified by claims 21 and 22.
  It does not change the action or the resulting order, including when the presentation-only
  berserk-cursor quirk below is off.
- **Acting house.** The house whose input is being resolved: the local player's house for a
  person's clicks. An *AI house* (glossary) is held to the same rules: when its order is checked
  against this spec, it is the acting house, and every rule below that names the local player
  names the acting house instead. By decision of the project lead, an AI house has the same order
  legality as a human-controlled house and no exemption (S-0213, claims 2, 3 and 15). Wherever a
  rule here asks whether a house is human-controlled, an AI house meets that condition too. The
  cursor, the click feedback (claim 14) and the
  modifier keys concern a person's input only.
- **Own object.** An object owned by the acting house. An actor that is not an own object
  always has the generic result of table G as its final action, which is *select* or *nothing*.
  All kind tables below assume an own actor.
- **Allied, enemy.** Judged from the actor's side, with the glossary's one-way *alliance*: a
  target object is *allied* when the actor's house allies the target's house (a house counts as
  allied with itself, so the same house is allied too). Whether the target's house allies the
  actor's house is not asked. Otherwise the target is an *enemy*. "A house not allied with the
  actor" in the cell rules means a house that the actor's house does not ally. The two readings
  differ only for a one-way alliance, which a skirmish has in one case: the special house of
  S-0412 allies the non-civilian houses and no house allies it, so an own actor treats an object
  of the special house as an enemy (medium; scenario T11). A cell has no relation.
- **Modifier roles.** Three keys held while clicking change the action: *force-move*,
  *force-fire* and *force-select*. The original uses Alt for force-move, Ctrl for force-fire
  and Shift for force-select (claim 23). These held gestures are distinct from the saved
  command bindings of S-0213. The *attack-move gesture* is force-fire and force-select held
  together: Ctrl+Shift with the original's bindings.
- **Type-select role.** A fourth role, like the modifier roles a role and not a physical key:
  while it is held during a *select* or *toggle select* click, or a band-box drag, the click or
  drag applies to the clicked object's whole type (claims 12, 29 and 32). A *tap* of the role,
  with no click needed, extends the selection by its own types (claim 32). Which physical key
  plays it is the player's key binding.
- **Tap.** One press and release of the type-select role whose release comes at most the tap
  limit (Constants) after its press (claim 32).
- **Type-select reach.** Interface state with two values, *screen* and *map*: whether the role
  acts on the screen candidates (claim 25) or on every object in play. It is *screen* at first.
  Taps change it. In the original one further command, which no spec of this project defines,
  can also change it (claim 32); our engine has no such command. Held clicks and drags only read
  it.
- **Control group, member.** One of ten numbered sets of the local player's objects, 1 to 10,
  that the group commands of claim 33 make and recall. An object is in at most one group. A
  *member* of group *N* is an object in group *N* that is the local player's own at that moment
  and is on the map: not carried in a transport and not inside a building.
- **Counted modifiers.** When force-fire and force-select are both held, every rule of this spec
  for resolving an action except claim 5 treats both as not held. "Held" in the action tables
  means counted in this way. Band-box selection reads force-select independently (claim 19).
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
- **Healing actor.** An armed object whose weapon heals: the weapon it would use by default does
  negative damage. Combat specs own the weapon choice. This is S-0214's own test, and it is not
  the glossary's *healer* (*healer for acquisition*), which asks whether the mean damage of the
  object's weapon slots is below 0 (S-0308, claim 15, which states the same difference). The two
  can differ, for example for a type that switches to a healing weapon only through its weapon
  number (S-0308's example). Every row below that names a healing actor uses this test.
- **Selection order.** Positive-primary-damage types first, newest-added first, followed by
  other types, oldest-added first. The primary weapon is the type's normal first weapon, not
  the weapon the object currently chooses. Claim 12 gives the addition and removal rules.
  This order decides representative ties, order issuance and order acknowledgements.
- **Primary battlefield button.** The left mouse button. Unless a passage says right click,
  a battlefield *click* in this spec means a left-button press and release, not a right click.
- **Band box.** The screen-aligned rectangle dragged with the left mouse
  button. Its endpoints and inclusion test are F6. It selects by one screen point per object,
  not by intersection with the object's image or foundation.
- **Box point.** The point used for a band box: for a mobile object, its drawing origin in the
  battlefield view, including world-height lift, rather than its ground shadow or the centre
  of its visible pixels. Normal standing objects use the projection of their position
  (S-0101 F1 and claim 5). This normal-position formula is not a universal promise for
  transient drawing states: claim 19 records the remaining uncertainty and its settling test.
  A building uses its position projected with height lift, not the extent of its foundation.
- **Cursor hot spot.** The point of a cursor image that coincides with the pointer's input
  pixel. Cursor frames below are numbered from 0 in the imported cursor image.
- **Selection response, special-attack response.** Separate kinds of spoken line. Selection
  responses accompany selection (S-0701 claim 17); special-attack responses acknowledge order
  kinds without their own response kind (claim 14; S-0701 claim 18). Each list is ordered as its
  entries appear in the user's data, possibly empty. Formulas F5 numbers entries from 0.
- **Clicked object.** The object that the game finds under the cursor at the moment of a click,
  from the cursor's position on the battlefield. It is the *target* of the claims when the target
  is an object. Whether a click finds an invisible building at all is an open question (Open
  questions).
- **Disabled.** For these input rules: paralysed, or, for a vehicle, playing its death sequence.
- **Being erased, phasing in.** An object that an erasing weapon (a temporal weapon) has locked
  onto is *being erased* (glossary), from the first lock until the last lets go or the object is
  removed. An object that is materialising after a chrono jump is *phasing in*. For an
  object's own jump with the teleporting locomotor, its delay is *D* ticks from S-0207 F9,
  and its end tick and harvester exception are S-0207 claim 20 (f)'s. A superweapon teleport's
  after-effect, the *arrival hold*, instead follows S-0513 claim 40, including its persistent
  held delay and `quirks.warp_hold_persists`; it is not a universal rules-delay hold.
  A destination given to the teleporting locomotor while phasing in is dropped, not saved for
  the end (S-0207 claim 20 (a), (f)), except under a pending point suppression (S-0207 Terms
  and claim 27), which keeps it as the order-level destination, fixes no landing point and
  consumes the suppression.
  This is separate from the click refusal of row GC1.
- **Parasite.** An object that has latched onto a mobile unit and harms it from there (for
  example a unit that climbs inside a vehicle, or one that grabs a ship). Its rules belong to
  S-0614. The parasite's *hold* is S-0614's hold timer (claims 7 and 8), not the separate
  paralysed state used by these input rules.
- **Spawned aircraft type, slave unit.** An aircraft type that a carrier-like unit launches and
  controls (Import mapping), and a unit that a slave master of S-0215 controls.
- **Movable.** An own object that the player can move. For a mobile object all of these must
  hold: it is not disabled; it is not deactivated (S-0405 claims 13 and 20); it is not being
  erased or phasing in; it is not held
  (S-0614 claims 7 and 8); it is not held inside a tank bunker; its type is not a spawned
  aircraft type; it is not a slave unit; and,
  if it launches spawned aircraft, its launching count is not strictly between 0 and its spawn
  count (S-0506 claim 10). These are S-0506's counts, not the number of aircraft away from the
  spawner. A building is movable when it is not paralysed and either it is
  rally-capable, or its type undeploys into a vehicle; a construction yard counts only when the
  game is not a campaign game, the game option that lets construction yards be packed up again is
  on, and the yard is not mind-controlled. (The original also asks that the yard's house be
  human-controlled; every acting house meets that, Terms, *acting house*.)
- **Able to fire.** An own object that is armed, not disabled and not deactivated (S-0405 claims 13
  and 20). Falling (for example under a parachute) is part of neither this nor *movable*. A unit
  falling under a parachute takes orders, and carries them out once landed (S-0522 claim 17, whose
  backing and medium rating this carries). During the fall, though, a click offers *attack* on an
  object only as claim 6 (e) allows, with that claim's backing and rating. Being movable or able to
  fire does not make an order take effect: a seized unit ignores every new destination (S-0607 claim
  9), and until it has its own locomotor back every weapon and target is *illegal* for it (S-0607
  claim 10). Those rules, with their backing and ratings, are S-0607's.
- **Melee-linked.** One of two objects joined by a *link*: the link that a hit of a weapon whose
  warhead carries the link flag (Import mapping) makes between the shooter and the object it hit,
  with both ends linked to each other (low for what creates it). It ends when the shooter is given a
  different target or either object is removed, so it can last as long as the shooter keeps firing at
  its target. A melee-linked object whose type lacks the keep-acting flag (Import
  mapping) takes no actions (table G).
- **Deploy check.** An object passes it unless it is a vehicle that is disabled, or a vehicle that
  is inside a tunnel while its type has the deploy-to-fire flag (Import mapping). Infantry,
  aircraft and buildings always pass it.
- **Deploy readiness.** Whether an object can carry out a deploy now, by its kind. Never on a
  tunnel cell (the marked-cell or edge predicate below; S-0507 claim 7). A building never has it. An aircraft has
  it when it is not disabled and its type can carry passengers. An infantry unit has it when its
  type deploys in place, and, if its type has an undeploy delay (0 or more), when all of these
  hold: its current sequence is none of the deploy kinds (S-0107, Terms; *deploy* itself counts,
  so a unit still deploying lacks it) and is not *undeploy*, and its rearm countdown (glossary,
  S-0301) has ended. For a type with a negative undeploy delay neither the sequence nor the
  countdown matters (Edge cases, *infantry deploy readiness and the rearm countdown*). A vehicle's
  deploy readiness does not depend on whether it is deploying, deployed or undeploying (for a
  deploy press, the consequences are S-0213, table T2's deploy rule). A vehicle has it when it is not
  disabled, not inside a tunnel, and it deploys into a building, carries passengers, deploys in
  place, sits in a tank bunker or has a deploy weapon; but not when it deploys into a building
  while a parasite is on it, not when it deploys into a construction yard while it is
  controlled by a controlling object. This is an exception to *mind-controlled* (Terms) for these
  input rules: a permanently controlled vehicle, or a vehicle held by a trigger's house takeover,
  is not refused (S-0601 claims 3, 13, 14 and 15). For a vehicle transport, the unload exclusions are S-0507 claim 7's:
  on a bridge, on a cell or straight-neighbour cell carrying a bridge deck, on water, or on a
  building's cell other than a tank bunker's, as well as that claim's disabled and tunnel cases.
  The water refusal applies equally to human and AI player houses; neutral and special houses
  retain the original's non-human exemption (S-0507 claim 7). Row VD7 adds independent cursor
  refusals; passing it does not establish readiness.
- **In-place deployer.** An infantry or vehicle type that deploys in place and later undeploys (it
  changes between a deployed and an undeployed state without becoming another object).
- **Representative.** The selected object whose action is shown as the cursor (claim 1).
- **Health ratio.** An object's current hit points divided by its type's hit points, computed in
  floating point (Formulas F2). *Full health* means a health ratio of exactly 1.
- **Garrison.** Infantry entering a building to fight from inside it. "Can garrison" in
  row I8 means the occupation test of S-0508 claim 1, not merely matching type flags.
- **Damaging entry.** The above-threshold engineer action of row I7: a cursor/order
  distinction, not a promise of damage on arrival. It issues the same capture order as
  *capture*. Skirmish arrival outcomes follow S-0509 claims 6 and 7.
- **Recycling building.** A building whose type turns units that enter it into credits.
- **Bridge repair hut.** A building type that an engineer enters to repair a bridge. The cursor
  uses S-0511 claim 21's exact test: the last qualifying candidate in the hut cell's 5 × 5 block,
  in increasing y then increasing x order, and that candidate's low-piece run or high-candidate
  bridge-link traversal. It is not a nearest-link test or the actual repair selection around
  the entering engineer of S-0511 claim 18. The cursor and actual repair can select different
  bridges; S-0511 claim 21 supplies the mixed-candidate and link-chain measurement variants. Entry effects check neither the hut's owner nor alliance and, with
  `quirks.repair_consumes_engineer_always` at its original default, use up the engineer even
  when nothing is repaired (S-0511 claim 17). With that switch off, entry is refused when
  claim 18 would repair nothing, leaving the engineer alive; S-0511 owns the switch.
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
- **Free mover.** An actor that is in the air, is an aircraft, is of an underground type or is
  a jumpjet infantry type. For a click on a cell, a jumpjet infantry type counts only while no
  ion storm is active. For a click on an object it counts always.
- **Can be selected.** An object that a click may put into the selection. Its type's selection
  flag must permit selection. Zero health alone does not exclude a building still in play
  (claim 16). Units and buildings can, unless something excludes them; a building whose type is invisible in the game is believed
  not to be selectable (low). A fully cloaked object that the viewer neither owns nor senses
  cannot be clicked, even if allied. Drawn fading objects can be clicked, subject to ordinary
  selection exclusions (S-0516 claim 26). A unit disguised as terrain or an overlay that appears
  disguised to the viewer cannot be selected (S-0516 claim 36). Whether a unit below the surface
  can be selected remains unidentified (low; Open questions). Rows G2, G10, claim 3 and claim 12
  all ask this one question.
- **Passive neutral house.** A house whose type carries the passive flag (Import mapping). Its
  buildings count as engineer-friendly for table I.
- **Self-teleporting type.** A type that teleports itself to where it is sent (the teleporter flag of
  Import mapping).
- **Underground type.** A type whose movement zone is *subterranean* (S-0211, claim 12). The
  property is the zone itself: a type is an underground type exactly when its movement zone is
  *subterranean*. It is a property of the type, in the surface state as well as below the surface. A
  unit that is *travelling underground* is one of this type that is below the surface (claim 6
  (b)).
- **Enclosed cell.** A cell that a ground unit can stand on but that lies in a different reach
  region (S-0212) of the unit's movement zone from the unit's own cell, for example a cell
  ringed by cliffs. It is not the same as an impassable cell.
- **Docking answer.** When a row asks whether the target accepts the actor now, the target answers
  through the contact exchange of S-0215 (and the building-kind behaviours). The answer is
  *accepts*, *refuses* or *neither*. A capacity refusal is distinct from an eligibility exclusion.
  A full building or an otherwise eligible full transport refuses. A vehicle transport answers
  *neither* to a non-allied or mind-controlled candidate, an infantry or vehicle candidate carrying
  a parasite, or a candidate controlling another object; it also answers *neither* while being
  erased or standing on a building's cell (S-0507 claim 4). An air transport answers *neither* to
  a non-allied candidate (S-0507 claim 2). The transport verdicts below give no entry verdict for
  *neither*: they keep their other result, not necessarily *no-enter*. A building also refuses
  when exactly one of it and the unit is naval, unless the unit's movement zone is *amphibious*
  (S-0504 claim 7). That exception removes only the naval-mismatch refusal, not other reasons
  to refuse. Asking changes nothing,
  so when and how often an implementation asks (eagerly or lazily, once or many times) cannot
  change any result (claim 2).
- **Terrain-fire weapon.** A weapon whose type carries the terrain-fire flag (Import mapping).
- **Rock overlay.** An overlay whose type carries the rock flag (Import mapping).
- **Tunnel cell.** For these input rules, a tunnel-marked cell or a tunnel edge for unloading
  as used by S-0507 claim 7. A cell is *tunnel-marked* only when its land type is tunnel and it
  is recorded as the entrance of an existing map tube (S-0212 claim 7); tunnel land alone does
  not qualify. A cell is an *edge* when, in map north or map west, either its first neighbour
  is tunnel-marked and its second is not, or its second is tunnel-marked and its third is not.
  All cells in the chosen sequence through the non-marked cell must exist. No south, east or
  diagonal test is added. Thus non-tunnel-land cells one or two cells south or east of the last
  marked cell of a tunnel band can qualify. Confidence: high; backing: `measurable`
  (S-0507 claim 7, scenario T7). A *tunnel entrance* for the cell-action tables is instead a
  cell whose tile is a tunnel tile.
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
- **Factory-marked building.** A building whose imported factory category is anything other
  than no factory, including a recognised non-producing category (S-0003 claim 21). This term
  concerns clicks only, not eligibility to serve a production line. Rows B3 and B5 and the
  building *deploy* row of table M use this wider category test (claim 17).
- **Rally-capable building.** A factory of infantry or vehicles, a cloning building or a repair
  depot. Nothing else is rally-capable: a tank bunker is not, and an aircraft factory is only when
  its type is also a repair depot. A recognised non-producing factory category neither grants
  this ability by itself nor cancels the independent cloning or repair-depot ability (claim 17).
- **Veterancy rank, veterancy ability.** As the glossary defines them; S-0311 owns both. An
  object's veterancy rank is rookie, veteran or elite, derived from its experience (S-0311, Terms
  and claim 1), and elite is the top rank. How rank is earned, and which ranks grant which
  abilities, are S-0311's (claims 3 to 5). This spec only tests whether the actor has an ability
  at its current rank: it has the explosive-charges ability or the crusher ability when S-0311
  claim 3 gives it, that is, when the ability is on the type's veteran list and the object is
  veteran or elite, or on its elite list and the object is elite. This spec uses the
  explosive-charges ability (row I16) and the crusher ability (value *k* of table V); what each
  ability does beyond those tests is S-0311's claim 4 (explosive charges: row I16 here, and
  crusher: S-0210 and table V here).
- **Mind-controlled.** As defined in S-0601 Terms: either *controlled* by a controlling object
  or *permanently controlled* without one. Permanent control counts for these input rules
  (S-0601 claim 13).

## Behaviour

1. **The representative.** When the selection is not empty, the action the cursor shows over a
   target (an object or a cell) is the action of one selected object, the representative. Each
   selected object gets a rank from table R. The representative is the object with the highest
   rank. The presentation-only berserk-cursor quirk can replace its cursor image, not its
   action or rank (claim 15). Among objects of equal rank it is the one nearest to the target
   (Formulas F1), and among those, the first in selection order (claim 12). With an empty selection the cursor shows the actions of the
   current sidebar mode (claim 13), or *select* over an object that can be selected.
   - Backing: `measurable` (Scenario T1, which also separates the tie-break by selection order)
   - Confidence: high for the rank order, medium for the distance

2. **Each object resolves the click itself.** When the player clicks, every selected object has
   its own action against the same target, by the rules of claims 4 to 9, and gets the order that
   table M gives for that action (claim 10). An object whose own action gives no order gets none.
   The orders of one click are issued in selection order, with the observable consequences listed
   under State and ordering (*ordering consequences*). Every selected object resolves the click
   against the state as it was at the click: no order made for the same click changes what
   another selected object sees, whatever the objects before it were given (two infantry units
   clicked on a transport with room for one both get an enter order; which of them gets the seat
   is for S-0215 and S-0213). The answers the tables read (the docking answer and the like) have
   no side effects. The representative's action decides only
   what the cursor shows, and whether the click is treated as a command at all (claim 3). In a
   mixed group, each object therefore does what it alone would do. For example, a click on an
   enemy building with engineers and tanks selected sends the engineers to capture it and the
   tanks to attack it.
   - Backing: `measurable` (Scenario T2; the same state for every object and the absence of side
     effects by Scenario T10, lines *shared state* and *no side effects*)
   - Confidence: high (medium for the absence of side effects)

3. **Clicks that are not commands.** If the representative's action is *select* or *toggle
   select*, the click changes the selection instead (claim 12), and no object gets an order. If it
   is *nothing* and the clicked object can be selected and is not selected yet, the click acts as
   *select*. If it is *nothing* over a cell, over an object that cannot be selected, or over an
   object that is already selected, nothing happens (the selection stays and no order is made).
   This fallback holds for every actor, whatever the reason for *nothing*: an actor that is being
   erased (row G2), an armed own actor over an object of another house (row G9) and an own loaner
   included. So the split between *select* and *nothing* in rows G2, G9 and G10 changes only the
   cursor that the representative shows. A click does the same under both. For example, a tank that
   is being erased and is alone clicks an enemy tank: the action is *nothing*, no order is made, and
   the enemy tank becomes the selection. While route mode is held (S-0213, claim 16), a command
   click adds route points instead of creating orders. The click is refused when the route-holder
   limit would be passed, when a selected object cannot take part in a route, or when it is a
   click on an existing point that S-0213 claim 16b refuses; no refusal adds a route point
   (S-0213, claim 16).
   - Backing: `measurable` (Scenario T2, whose tower and power-plant steps give the *nothing*
     cases, Scenario T9 part A for an actor that is being erased, and S-0213's Scenario O8 for the
     refusal at the route-holder limit)
   - Confidence: high (medium for the already-selected case)

4. **The generic result on an object.** Over an object, every actor has a *generic result* *g*
   by table G. It covers the modifier roles, clicks on the actor itself, the attack rule of claim
   6 and plain selection. Table G states the generic result *g*. When multiple row conditions hold, the result is the one in the highest-priority applicable row; the row order specifies precedence between outcomes, not the order or method of testing conditions. The final action of an own actor is a function of *g* and the actor's and target's
   properties, given by its kind's table (claims 7 and 8). For an actor that is not an own object,
   *g* is the final action.
   - Backing: `measurable` (Scenario T3, including row G3, and Scenarios T2 and T6 for rows G4,
     G5 and G10; Scenario T6 also for an armed garrisoned building). Row G2, and the being-erased
     and phasing-in part of row GC1, by Scenario T9 parts A and B, which click at a tick timed to
     the state with scripted input. The own-jump duration and dropped-destination rule cited
     in Terms are backed by S-0207 claim 20 and Scenario D22 runs 5 and 6. The superweapon
     arrival hold is backed by S-0513 claim 40 and Scenario SW13 part c. The launching-count
     and hold exclusions in *movable* are backed by S-0506 claim 10 (R6) and S-0614 claims 7
     and 8 (P1 and P9), respectively. The deactivated exclusions in *movable* and *able to
     fire* are backed by S-0405 claims 13 and 20, scenario W8 part a.
   - Confidence: medium

5. **The attack-move gesture.** If the attack-move gesture is held (or attack-move mode is on)
   and every selected unit or building is eligible, then a click whose order would be *move* or
   *attack* produces an attack-move order instead (S-0213, claim 6): an order that would be *move*
   is issued as *attack-move to a place*, and one that would be *attack* as *attack-move on a
   target*. This covers every action whose order is *move* or *attack* in table M, including bomb,
   demolish, airstrike and enter tunnel, and also the actions that part 1 of table M treats as
   *attack* (heal, disarm bomb and repair vehicle; low, Scenario T3), because the gesture changes
   the order and not the action. An object is eligible if it is infantry or a vehicle, has a
   weapon in its first slot, and its type does not carry the prevent-attack-move flag. Buildings
   and aircraft are never eligible. One ineligible object in the selection turns the gesture off
   for the whole selection. Whether or not the gesture is on, force-fire and force-select held
   together count as if neither were held for every other rule of this spec (Terms, *counted
   modifiers*). Attack-move mode is entered with the command-bar button, not with a
   dedicated keyboard command. With a nonempty selection it is switched on only when every
   selected unit or building is eligible; an ineligible selection gives a message and does not
   newly arm it. With an empty selection the button gives a no-selection message but still arms
   the mode; this creates no order by itself. The mode ends when the next battlefield click has
   been handled or the selection is cleared. The held Ctrl+Shift gesture needs no command bar
   and is not a latched mode: releasing either key ends the gesture.
   - Backing: `measurable` (Scenario T3, which includes the healing case; the prevent-attack-move
     flag by the T10 line of that name; Scenario T17 for physical keys, button entry, refusal,
     empty-selection warning and the one-click lifetime)
   - Confidence: high for the physical gesture and button entry; medium for order conversion

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
     The combat specs rule every target illegal for a falling firer (S-0301 claim 9, rank 4). That
     is a firing rule, not one of those other reasons, so a falling actor, for example under a
     parachute, fails part (e) for every target unless force-fire is held or the infiltrator
     exception applies. Over an enemy during its fall, an own armed infantry unit therefore gets
     *nothing* from row G9 and then, unless an earlier row of table I applies, the fallback move
     (row I28), usually *no-move*. A falling vehicle gets the same by row V4. The click gives the
     move order of table M toward the target (claim 11), not an attack order. With force-fire held
     the click gives an attack order at once, which the actor carries out after landing (Terms,
     *able to fire*).

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
   - Backing: `measurable` (Scenario T4, including a unit travelling underground for part (b),
     the permanently disguised target, T4 part C for parts (d) and (e) and T4 part D for a falling
     infantry unit; Scenario T10 for the
     attack-friendlies flag, the attack-cursor-on-friendlies flag and its bomb-planting
     exception). Four parts are `unmeasured`: a falling vehicle, because no scenario here drops
     one; the sonic-warhead exception of *May attack as a
     friend* and the case of a disguise that is currently disrupted (blinking), because the setup
     has no known way to put a parasite on a mobile unit or to disrupt a disguise at a chosen
     tick; and the case of a unit or building disguised as scenery over a terrain-fire weapon (low),
     because no disguise that belongs to no house is known in the setup (Open questions).
     The exact limit of −20 leptons in part (b) is `measurable` by Scenario T4 part B: scripted
     clicks on every tick of a unit's descent, with the recorder reading its height. The limit is
     fixed only as finely as the height changes between two ticks of the descent (Open questions).
   - Confidence: medium (low where marked)

7. **Infantry on an object.** The final action of an own infantry actor on a target object is
   given by table I, a precedence list over the generic result and the actor's and target's
   properties.
   - Backing: `measurable` (Scenarios T5 and T10; S-0511 claim 21, B13 for row I3's cursor
     test, and claims 17 and 18, B12 and B14 for hut-entry effects and repair selection,
     S-0508 claim 1, G1 and G2 for row I8's occupation test; S-0509 claims 6 and 7,
     E4 for the distinction between engineer click action and arrival outcome; S-0519
     claim 10, T11 for row I2's bomb-visibility requirement, and claim 13, T13 for
     row I16's veterancy-only planting failure). The building count test of rows x and
     I13 is backed by S-0605 claim 1, scenarios A1 and A1b, which set capacity 3, 0 and −1
     for an entry and for a healer.
     The rows that the list after Scenario T10 gives
     as `unmeasured` have no step, each with its reason.
   - Confidence: medium (low where a row says so)

8. **Vehicles, aircraft and buildings on an object.** The final action of an own vehicle follows
   table V, of an own aircraft table AC and of an own building table B, in the same way.
   An ordinary click on an already-full repair depot gives no enter order. This differs from
   enter orders issued together before contact is reserved: their execution can leave a damaged
   second vehicle waiting under S-0314 claim 23, rather than approaching immediately.
   - Backing: `measurable` (Scenarios T6, T9 part D and T10; S-0504 claim 7 and scenario N7
     for the naval-mismatch docking refusal and amphibious exception; S-0507 claims 2, 4 and 7,
     T2, T4 and T7 for transport answers and readiness; S-0314 claim 16 and H16 a, b and d,
     plus Scenario T6 below, for depot permission and click outcomes; S-0314 claim 23 and
     H18 f for orders issued together and queued entry). Rows that the list after Scenario
     T10 gives as `unmeasured` have no step, each with its reason. The building count test
     of transport verdict q (table V) is backed as in claim 7: S-0605 claim 1, scenarios A1
     and A1b.
   - Confidence: medium for table V; low for tables AC and B

9. **Actions on a cell.** Over a cell, every actor has a generic cell result *c* by table GC.
   For an infantry unit, vehicle or aircraft, the *shroud change* of table S turns *c* into *c*′:
   on a shrouded cell a result other than *nothing* becomes *move*, *patrol* or *no-move*.
   The final action is a function of *c*′ (of *c* for a
   building), given by the kind's cell table (IC, VC, ACC or BC). For an
   infantry unit, vehicle or aircraft, a click on a cell outside the usable area produces an order
   only when the action is *no-move*.
   A force-fire click that produces an attack order on a destroyable-cliff cell names the
   clicked cell at input. At a later approach step, that target can be replaced by another
   cell of the same tile under S-0307 claim 23. That replacement belongs to approach,
   not to click resolution; S-0215 claim 37 owns when the approach step occurs.
   - Backing: `measurable` (Scenarios T7 and T10; S-0307 RG24 for the cited target replacement), except the rows that read what fog of war shows
     (rows IC2 and IC3, and the fog parts of rows GC9, VC2 and BC2), which are `unmeasured`: no
     setting that turns fog of war on in a skirmish is known (Open questions).
   - Confidence: medium (low for the building cell table)

10. **From action to order.** Table M gives the order that each action produces when an object
    resolves a click, including the actions for which an actor gets no order, by the actor's
    kind.
    - Backing: `measurable`. The *Coverage of table M* paragraph after Scenario T10 names, for
      each action and each case of table M, the scenario step that gives its order. The cases it
      lists as `unmeasured` have a reason there.
    - Confidence: medium (low for buildings and aircraft)

11. **Move destinations.** The destination of a *move* or *no-move* order is the clicked cell or
    object itself, or a cell found by the nearby-cell search of S-0212 (claim 25; the parameters
    of each use of the search are its table T5). In this claim a cell is *inside* when it is inside
    the usable area, and *in reach* when it lies in the same reach region of the actor's movement
    zone (S-0212) as the actor's own cell.
    - *On a cell:* when the cell is shrouded and the actor's type may not move into shroud, the
      actor gets no order. Otherwise:
      - The destination is the clicked cell itself, with no search and whether or not the cell is
        occupied, when the cell is inside and not shrouded and one of these holds: the action is
        *move* and the actor is a free mover (Terms); the action is *move* and the actor's type
        teleports (Terms, *self-teleporting type*); the cell is in reach and the actor is not a free
        mover performing a *no-move*.
      - In every other case the destination is a cell found by the search from the clicked cell.
        A *no-move* by a free mover searches without a reach region, whether or not the cell is
        inside the usable area. Every other search uses the reach region of the actor's own cell. When the search
        finds no cell, the actor gets no order.
    - *On an object:*
      - The destination is the clicked object itself when the action is *move*, the object's cell
        is inside, and either (i) the actor is a free mover, or (ii) the object's cell is not
        shrouded and in reach, and force-move is not held by an infantry actor.
      - Otherwise it is a cell found by the search from the object's cell (without a reach region
        for a *no-move* by a free mover, with the reach region of the actor's own cell otherwise),
        and the object itself when the search finds none. So for an infantry unit, vehicle or
        aircraft a *no-move* on an object still orders a move, except where part 1 of table M
        gives no order (a building's *no-move* gives none, part 3).
      - Shroud takes precedence over both: a *no-move* on an object in shroud gives no order unless
        the actor's type may move into shroud, and an actor of an underground type (unless the
        general rules allow underground moves into shroud) and an aircraft get no order from a
        click on an object in shroud, whether its action is *move* or *no-move*.
    These are mobile move-order destinations. A building's guard-spot or rally-point order
    instead uses table M part 3 and S-0213 claim 20: rally-point searches have their own
    parameters in S-0212 claim 25 and do not inherit this direct-destination bypass.
    - *Zones in the search:* every search uses the actor's movement zone, with two substitutions
      (zone names are S-0211's, claim 12). An actor of zone *subterranean* is searched as zone
      *normal*. An actor of zone *fly* is searched as zone *infantry* for a click on an object
      always, and for a click on a cell only while an ion storm is active (low); with no ion storm,
      a click on a cell searches it as *fly*. Every other zone is searched as itself.
    - Backing: `measurable` (Scenario T7, which has a step for the clicked cell being the
      destination with no search, for the *subterranean* substitution, for the destination being
      the object itself, and for the shroud cases). Three parts are `unmeasured`. The first is that
      a free mover's *no-move* searches without a reach region on a cell or object *inside* the
      usable area: step (a2) of Scenario T7 reaches only the strip outside it, and no input is
      known that gives a free mover a *no-move* over a usable, unshrouded cell (rows GC9 and GC10
      give it *move*), so no step can contrast the two; the rule is stated for both because it
      does not depend on the area. The second is the ion-storm
      part: no way to start an ion storm in a skirmish with stock data is known (Open questions).
      The third is the substitution of *infantry* for *fly* in a click on an object: it matters only to an
      actor of zone *fly* that is not an aircraft, or to an aircraft clicking an object outside the
      usable area (an aircraft otherwise takes the object itself as destination, or gets no order,
      by part 1 of table M), and no such click is known to be possible in the setup. A test type
      with a driving locomotor and zone *fly* would measure S-0212's rules for that zone more
      than this click rule.
    - Confidence: medium

12. **Selection clicks.** When an object is added to the selection, it goes at the front if
    its type's primary weapon exists and does positive damage, and at the end otherwise.
    This is a type-level test, not the object's current weapon choice, veterancy or ability to
    fire. Thus positive-primary-damage objects precede the others in reverse order of addition.
    The others retain their order of addition. Removing an object preserves the relative order
    of the remaining objects. The first object is the most recently added of the former group,
    or the earliest added when that group is empty (S-0701 claim 18).
    Selection uses the type's enslaved selection-response list for a slave unit, otherwise its
    deactivated list for a deactivated object, otherwise its normal list. A deactivated object
    is one that has lost what it needs to operate (S-0701 claim 17). An empty applicable list
    gives silence without fallback to the normal list. Selection responses and their playback
    are owned by S-0701 claims 16 and 17, not the order-response rule of claim 14.
    *Select* replaces the selection with the clicked object. *Toggle
    select* adds the clicked object to the selection, or removes it if it is already selected; it
    acts only when the first selected object is the acting house's own, and otherwise acts as
    *select*. While the type-select role is held, both act on the clicked object's type as a
    whole, within the type-select reach. Claim 32 gives which objects that takes, their order and
    the silence, and what a tap of the role does without a click.
    Cloaking can remove an object from an existing selection: at cloak start every non-owner
    viewer loses it, even with sensors; at later re-evaluation a fully cloaked object is removed
    from a non-owner viewer's selection when unsensed (S-0516 claim 26). Owning the object and
    being allied with its owner are not interchangeable for these tests.
    - Backing: `measurable` (Scenarios T1 and T2 for selection order; S-0701 scenarios D2 and D3
      for selection-response lists and silence; S-0516 claims 26 and 36, CS9 part e and CS15
      for cloak/scenery click exclusions and cloak selection
      loss), except one part, which is `unmeasured`: the fallback of *toggle
      select* to *select* when the first selected object is not the acting house's own while an
      own object is the representative: the rows give *toggle select* only to an own actor, so
      the case needs an object that changes owner while it is selected, and whether such an
      object stays selected is not known (Open questions). Over a first selected object that is
      an enemy, the click gives *select* by row G10 and does not reach the fallback.
    - Confidence: high for selection order and absence of response-list fallback; medium for
      selection clicks and which objects use the variant lists

13. **Sidebar modes.** With an empty selection, the sidebar's mode buttons give these actions:
    - *Sell mode:* over an own building that can be sold and is not vehicle-like, *sell*. Over a
      wall cell owned by the acting house that is neither shrouded nor fogged, *sell*.
      This produces a sell-wall order only when no individual object was picked. Its target
      is the cursor cell of S-0101 claims 9 and 10, not a cell chosen by wall-art pixels.
      S-0408 claim 12a gives the cell rule and the distinction from object-sale dispatch.
      A picked individual object with a *sell* action receives an object-sale order instead,
      never a fallback wall-sale order. Anywhere else, *no-sell*, including over own vehicles,
      aircraft and vehicle-like buildings. No rule studied gives a *sell unit* action, which would
      produce a sell order for a vehicle or aircraft (Open questions).
    - *Repair mode:* over an own object that can be repaired, *repair switch*. Anywhere else,
      *no-repair*. A click with *repair switch* produces a repair order only on a building.
    - *Power mode:* over an own selectable building that is not vehicle-like, whose type can be
      switched off, and whose type either has a base drain above 0 (S-0405, Terms) or has the
      needs-power flag (S-0405, Terms), *power switch*. The test reads the type only, never the
      building's present output, drain, upgrades or switch state, so a power plant whose type has
      neither a base drain nor the needs-power flag gets *no-power* whatever it makes, and a
      building that has a base drain gets *power switch* even when it is already offline. Anywhere else,
      *no-power*.
    - *Superweapon targeting:* the superweapon's own targeting rule (superweapon specs).

    A click with *sell* produces a sell order (S-0213, claim 12), with *repair switch* a repair
    order, and with *power switch* a power-on order for a building that is offline and a
    power-off order for one that is online.
    - Backing: `measurable` (Scenario T8; S-0408 Scenario S11 for the wall-cell and
      object-versus-wall dispatch)
    - Confidence: medium; high for wall-cell targeting

14. **Click feedback.** Only the first object of the selection, in selection order, can give the
    spoken acknowledgement, whether or not its own action gives it an order: when that object gets
    no order from the click, no object gives one (medium). For ordinary immediate battlefield
    clicks, the response kind and fallbacks are S-0701 claim 18 and F6's: harvest, attack, move,
    attack-move, enter, capture and unload have their own response kinds; every other behaviour
    order kind uses the type's special-attack list, not its selection list. A nonempty applicable
    list takes one raw value from the unsynchronised random stream (Formulas F5), not a ranged
    draw (S-0202 claim 4). An empty list takes no choice draw and gives silence; no later selected
    object responds instead. A single sound entry takes no choice draw. Playback may take
    additional unsynchronised draws under S-0701; these are separate from choosing the response.
    Selection's enslaved and deactivated lists (claim 12; S-0701 claim 17) do not replace an
    order's response list.
    Claim 30 owns the move-target marker: its eligible clicks, single-marker group feedback,
    position and height. The original gives both kinds of feedback for the local player's own clicks only. In our engine both are drawing and
    sound only: they take no part in game state and make no draw from the scenario random stream
    (any response-choice and playback draws use the unsynchronised stream, as above), and an
    AI house's orders give neither (a design rule of ours: the original has no AI input path to
    observe). This is the departure confirmed by the project lead and
    stated in S-0213 (Edge cases, *the click marker*): since the original creates them only on the
    machine of the player who clicked, any effect on game state would already have split its own
    network games, so drawing only is consistent with it.
    - Backing: `measurable` (Scenario T2), by ear and by eye, for the single acknowledgement and
      the single marker, for no marker over a *no-move* (the *no-move* step of Scenario T2), and
      for the first object giving none when it gets no order. The AI-house
      part is `unmeasured`: it is our own design rule and has no counterpart in the original. The draw is also `measurable`: Scenario T2 part B reads the unsynchronised
      random stream with the read-only random-state recorder of the measurement setup (the same
      extension of S-0202's state recorder that S-0010's scenario V2 part B uses), counts the raw
      values a click takes and compares the chosen response with the mapping of Formulas F5.
    - Confidence: high for one acknowledgement per click, medium for the draw and for the first
      object being the one that decides; marker eligibility and height are claim 30's

15. **Objects that get no order.** An infantry unit, vehicle or aircraft that is berserk, or that
    is melee-linked while its type lacks the keep-acting flag, gets no order from any click on an
    object or a cell, whatever action the tables give it, with one exception: a berserk carrier
    aircraft can still get *tote* (rows AC2 and AC3) and the move order that *tote* produces
    (table M, low). (Its generic result is *nothing*, but
    some infantry and vehicle rows do not depend on the generic result, so such an actor can still
    show an action when it is the representative.) A berserk object has rank 0 (table R).
    The infantry/vehicle cursor mismatch has the presentation-only switch
    `quirks.berserk_action_cursor`, defined below; it changes neither the resolved action nor
    claim 3's selection fallback. The tote exception is original behaviour and has its own
    independent quirk switch, in Edge cases under *berserk carrier tote*.
    - Backing: `measurable` (Scenario T6, including part B for the cursor mismatch) for the
      berserk case, and Scenario T9 part C for the melee-linked case, with scripted clicks
      while the link holds.
    - Confidence: medium (low for the melee-linked case, whose cause is low)

16. **Selecting a destroyed building awaiting removal (Q-0081 parts 3 and 4).** A building
    awaiting removal under S-0312 claim 21 can still be selected by a battlefield click if it
    otherwise permits selection: zero health is not a selection exclusion. This holds both for
    a destroyed building of an exploding type and for one destroyed while being sold. With an
    empty selection and no sidebar targeting mode, a click on an own visible, selectable
    building that is not being erased selects it during that interval. Its occupied foundation
    can still supply the clicked object even when the destruction has removed its visible
    animations. Selection does not restore health, postpone removal or restart a sale.
    Being erased remains a selection exclusion; postponing removal that way does not make
    the building selectable. This claim does not promise selection for an invisible type or
    a type whose selection flag is off.

    S-0302 claim 16 already answers Q-0081 parts 1 and 2 (countdown expiry, its hit and credit,
    capture, sale and removal before expiry). S-0312 claims 3 and 21 already answer the death
    effects, removal timing, further hits, acquisition eligibility, turns, occupancy, draws,
    rubble and crates in parts 3 and 4. Those remain their rules, not selection rules here.
    - Backing: `measurable` (Scenario T12; S-0302 D13 and S-0312 D14 for the cited rules)
    - Confidence: high for zero health not excluding selection and the occupied-foundation
      click; medium for obtaining a click during the short interval in stock play

17. **Factory categories do not replace the other click abilities.** S-0003 claim 21 and
    Import mapping C19 own the imported category vocabulary. A recognised category that
    produces none of the four production kinds still counts as factory-marked for rows B3
    and B5 and for issuing a primary order from *deploy* in table M. It does not count as an
    infantry, vehicle or aircraft factory in rows that name those kinds specifically. In
    particular, a non-producing category alone does not make a building rally-capable, but
    a building of that category that is also a repair depot or a cloning building remains
    rally-capable. Not being a producer therefore does not imply that clicks do nothing.
    S-0213 claim 13 owns what the primary order does when applied.
    - Backing: `measurable` (Scenario T13)
    - Confidence: high

18. **Beginning a band box (Q-0278 part 1).** A left-button press in the battlefield can
    start a selection drag when sell, repair, power, building-placement and superweapon
    targeting modes are off. After a pointer-motion input with F6 distance greater than
    4 pixels, release is handled as a band box rather than an ordinary click, subject to
    claim 19's empty-box exception. The distance is from the original press pixel, not
    from the previous motion event; the box remains anchored at that press pixel. Returning
    closer after crossing the threshold does not turn it back into a click. Once dragging,
    the moving endpoint is limited to the battlefield view's last pixel on each axis.
    Route-editing and beacon interactions are not ordinary selection drags here.
    - Backing: `measurable` (Scenario T14)
    - Confidence: high for the threshold and anchor; medium for the mode exclusions

19. **Objects and modifiers in an ordinary band box (Q-0278 parts 2, 3 and 5).** The released
    rectangle includes its four edges (F6). It takes own selectable infantry, vehicles and
    aircraft whose box point lies inside, provided they are in play, not already selected,
    not deactivated, not controlled by a slave master and not held in a tank bunker. A unit
    tethered to a building is excluded while its cell holds that building. Thus ordinary
    harvesters have no type-level exemption, but a harvester docking at a refinery can be
    excluded by this tether test. Aircraft are tested at their lifted drawing origin, not
    the shadow below them. Ordinary buildings are excluded, but vehicle-like buildings
    can be taken if they meet the same eligibility tests. Enemy and allied objects are
    never added by an ordinary box. A selectable object being erased still cannot be added
    (the normal selection exclusion of claim 16). A bunkered vehicle and an occupied
    bunker are excluded from this ordinary band-box selection. This is the band-box
    part of S-0606 claim 11, not a blanket refusal of click selection or proof that
    the bunker cases of rows VD5 and I17 cannot be reached by player input.

    Without force-select, a box containing at least one live box candidate clears the old
    selection before adding eligible objects. The clearing test is broader than eligibility:
    even an enemy or an ordinary building inside can make the old selection clear while
    nothing new is selected. With force-select, eligible objects are added, not toggled:
    already-selected objects remain selected. Force-move and force-fire do not otherwise
    change this ordinary box selection; holding force-fire with force-select still adds,
    regardless of the counted-modifiers rule (Terms) that governs actions. A type-select drag instead expands
    the types encountered through claim 32's type-selection rule, and is silent (S-0701
    claim 17). Which objects in the box start that expansion, and which objects it adds, are
    claim 32's (*held drags*): none of this claim's candidate tests applies to them, while its
    clearing rule and its empty-box rule below still apply.

    If there is no live box candidate inside, an unmodified box does not clear the selection:
    its release continues as the ordinary release click at the final pixel. With force-select
    it performs no such fallback. See the empty-box quirk below. There is no additional
    shroud-cell test when the box is released: eligible own candidates are not rejected
    just because their cells are shrouded. Objects omitted from the current screen candidates
    cannot be taken; this is not a search of all objects in the rectangle's ground area.
    Exact points during transient mobile drawing displacements remain `unmeasured`,
    with low confidence: a normal projected position does not establish the point during
    every deploy, factory-exit, sinking or lifted state, and no complete state-to-point
    correspondence has been confirmed. No universal reference-point formula is claimed
    for those states. Scenario T14 part B is the settling scenario, not an assertion that
    transient points equal their normal positions.
    - Backing: `measurable` (Scenario T14) for the stated membership rules; `unmeasured`
      for the transient-point qualification above (reason and settling scenario given there)
    - Confidence: high for ownership, point inclusion and add-not-toggle; medium for state
      exclusions, type-select interaction and shroud; low for transient points

20. **Box capacity is not a selection limit (Q-0278 part 4).** The original admits at most
    500 screen candidates in one drawn frame, shared by box selection and pointing at
    objects. This is a candidate limit, not a limit of 500 own selected units: other kinds
    and other owners can consume places. An ordinary box release can therefore add no more than
    500 objects, and often fewer. The selection itself has no fixed count limit; separate
    clicks or additive boxes can grow it beyond 500. The 128 pending-order limit of S-0213
    claim 2 is independent: selecting an object does not reserve an order place. Additions
    use claim 12's selection order; S-0701 claim 17 gives the response only to the first
    successful addition, not necessarily the eventual first object in selection order.
    Candidate admission at overflow is not nearest-to-box-centre selection and does not
    prefer own eligible units: the box uses candidates already admitted for the view,
    without replenishing places consumed by ineligible objects. Buildings consume places
    before mobile drawing candidates, including buildings that an ordinary box cannot select.
    Claims 24 and 25 give membership and overflow priority. This corrects the earlier
    mobile-before-building statement.
    - Backing: `measurable` (Scenario T15)
    - Confidence: high

21. **Battlefield cursor images (Q-0277 parts 1 and 3, battlefield actions).** Table C gives
    the cursor frames for every action in table A, outside route-point editing. A range is
    inclusive. Unless stated otherwise the hot spot is at the image's centre on both axes.
    Shroud can change the action under claims 9 and 11 before this presentation rule applies.
    Table C's shrouded column describes any action that still reaches presentation with a
    shrouded target, not permission to issue an order. In particular *nothing* shows the
    ordinary arrow, and an illegal move has a dedicated refusal image: there is no single
    universal no-action cursor. Damaging entry retains the previous cursor image rather
    than choosing a fresh capture image (quirk below). Sell, repair and power input offers
    are still claim 13's; their images are included in table C. With the presentation-only
    berserk-cursor quirk off, its arrow replacement takes precedence over table C.
    - Backing: `measurable` (Scenario T16, with the existing targeting scenarios; T6 part B
      for the berserk-cursor exception). Table C cases with no known reachable actor/target
      setup are `unmeasured`: T16 cannot present those actions, in particular the shrouded
      cases eliminated by the action tables. Their assignments are not inferred from clicks.
    - Confidence: high for frame assignments; medium for shrouded cases not normally reachable

22. **Cursor frames, hot spot and time (Q-0277 part 2, battlefield cursors).** Cursor images
    and palette are imported as listed below. Animated ranges in tables C and CS advance one frame
    whenever the original's 16-millisecond presentation clock has advanced at least
    4 clock units since the previous advance. The clock is real time, independent of game
    speed, not simulation ticks. One update advances at most one frame, wrapping to the
    first frame; delayed updates do not skip frames to catch up. Changing cursor kind
    starts at its first frame and restarts the interval. Repeating the same kind does not
    restart it. Static rows stay on their one frame. The centre hot spot is
    (⌊width/2⌋, ⌊height/2⌋) pixels in the cursor image's full canvas; the ordinary arrow
    uses (0, 0). This is not the centre of the opaque pixels or of an individual frame's
    cropped bounding box. Our interface measures the same intervals with its own real-time
    input/display clock; no clock value affects simulation state.
    - Backing: `measurable` (Scenario T16)
    - Confidence: high

23. **Physical held gestures (Q-0286 question 2).** In the original Alt held gives
    force-move, Ctrl held gives force-fire and Shift held gives force-select. Either side
    of each modifier key plays the same role. Pressing or releasing such a key alone creates
    no unit order: its held state changes the subsequent click. There is no separate Cmd
    binding in the original. These gestures are not the rebindable stop, scatter, guard,
    deploy, cheer or route commands of S-0213 claim 22. Ctrl+Shift enters the attack-move
    gesture of claim 5 only for an eligible selection; the command-bar button is the other
    entry described there, not an additional default attack-move key. Ordinary box addition
    reads Shift independently, including while Ctrl is also held (claim 19).
    - Backing: `measurable` (Scenario T17)
    - Confidence: high

24. **Which objects consume candidate places (Q-0392 questions 1, 3 and 4).**
    Candidate membership is not image intersection with the battlefield view, and is not
    restricted to objects whose box points are inside that view. For a mobile object
    participating in the frame's ordinary body drawing, its point qualifies exactly
    when it satisfies F7's expanded-view test. The drawing tests of S-0101 claim 6
    still apply; a candidate is not created for a unit excluded from that drawing. A body can therefore consume a place while entirely outside the
    view, or while all its pixels are covered by other objects. Conversely, a large image
    entering the view does not suffice when its point fails that test. This is a drawing
    participation rule, not a scan of all units in play: an object omitted from mobile
    drawing contributes no mobile candidate. A vehicle playing its deploying or
    undeploying presentation supplies no ordinary vehicle candidate even when that
    presentation has visible pixels. It does not require any opaque body pixel
    to survive clipping or depth comparison. Claim 19's transient-point qualification
    still applies; this claim does not substitute the shadow or image centre for that point.

    Buildings have a separate point test: their height-lifted projected positions must
    lie in the unexpanded view, including its right and bottom boundary coordinates
    (F7). An in-play building that is on the map and whose type is not invisible in the
    game contributes a candidate on that basis, without requiring its image to intersect
    the view, be freshly drawn, or be visible through shroud. Zero health alone does not
    remove it while it remains in play. A vehicle-like building is a building for this
    admission rule and for claim 25's priority, even though claim 19 can select it.

    Neither group is filtered by owner, selection eligibility or already-selected status.
    Ordinary terrain objects, overlays, animations, projectiles, particles and their
    effects consume no places. Nor does a veinhole monster: it supplies no drawing in
    a stock frame. A unit disguised as scenery is still a unit, not an additional terrain
    candidate. Candidate admission does not change claim 19's later selection tests.
    - Backing: `measurable` (Scenario T18; T15 part B for vehicle-like buildings and
      ineligible objects consuming capacity)
    - Confidence: high for the point tests, building classification and excluded ordinary
      kinds; medium for stock mobile drawing participation in transient states

25. **Which candidates survive overflow (Q-0392 question 2).** The admitted set is the
    first 500 qualifying candidates in this priority order, or all of them when fewer
    qualify. Qualifying buildings precede every qualifying mobile object. Among buildings,
    earlier-created surviving buildings precede later-created ones; changing owner does
    not move one ahead. Among mobile objects the priority is their current drawing order:
    underground, surface, ground, air, then top render layer, and within each layer the
    order of S-0102 claims 4–7. Non-candidate drawings between two mobile drawings consume
    no places and do not reverse their priority. Thus ground mobile candidates precede
    air and top candidates regardless of house, selection eligibility or distance from
    the pointer. Equal ground sort keys keep the drawing-order tie rule, not a new
    house or object-number tie-break. Upper-layer priority follows layer entry and
    re-entry rather than screen position. Use the actual current layer and drawing order,
    not a fresh full sort of ground positions at box release. S-0102 claim 4's unresolved
    flight re-entry moments remain that spec's uncertainty, not permission to choose a
    different order among the drawings of a given frame.

    Only the admitted set is available to an ordinary box. For example, 20 qualifying
    ordinary buildings and 490 qualifying mobile objects leave places for only the
    earliest 480 mobile candidates, although none of those buildings can be added to
    an ordinary selection. Changing ownership alone does not replenish those places.
    This order concerns selection availability; it does not change which overlapping
    object's image is drawn on top.
    - Backing: `measurable` (Scenario T15 part B; S-0102 claims 4–7 for drawing order)
    - Confidence: high for building-before-mobile and render-order priority; medium for
      building creation priority

26. **Mouse buttons for clicks and selection.** In ordinary battlefield input, a left-button
    release resolves the action at the release pixel: it selects or toggles selection under
    claims 3 and 12, or gives the move, attack, enter and other orders of claims 2–11 and
    table M. Pressing the button alone issues none of these orders. A left-button drag
    makes the band box of claims 18 and 19, with the F6 threshold; it is not drag scrolling.
    A right-button press alone neither deselects nor gives an order. A right-button release
    without an intervening scroll drag clears the selection in ordinary mode, regardless
    of whether it is over an object or empty ground. It never resolves table M as an order
    click. This clears selection, not the units' existing behaviours or destinations.
    The original has no in-game setting or saved option that exchanges the selecting/order
    button with the deselecting button. Its scrolling-method preference changes how a
    right drag scrolls, not which button selects, orders or deselects (Import mapping).
    These are the original's left/right button roles; a host-level exchange of mouse buttons
    is not an additional game control scheme.
    - Backing: `measurable` (Scenario T19)
    - Confidence: high for button dispatch and release timing; medium for absence of an
      alternative scheme across the settings interface

27. **Right-drag versus right-click.** An accepted right-button press in the battlefield
    anchors a possible scroll drag at that screen pixel. While held, a sampled displacement
    starts the scroll drag exactly when F8's axis test passes. This is not the 4-pixel
    table-root test of a left-button band box. Once started, the scroll-drag classification
    stays until release, even if the pointer returns to its anchor or scrolling is stopped
    by a view limit. Its release does not clear selection, cancel a targeting mode or give
    an object order. Without a sampled crossing it is the right click of claim 26 or 28.
    Dragging with the right button therefore permits changing the view while retaining
    the selection. The scrolling rate, direction, pointer recentering and view limits are
    companion rules of S-0101 claim 16 and F10; its claim 17 owns the separate drag-scroll
    cursor, not the edge-scroll images (Q-0450). This claim fixes only the gesture and
    release distinction. The original's threshold depends on host drag metrics, not on
    imported game rules or the scroll-speed setting. By decision of the project lead, our
    engine uses a fixed metric of 4 pixels on each axis, so a right drag starts beyond
    8 pixels on either axis; no universal numeric pixel threshold is asserted for the original.
    - Backing: `measurable` (Scenario T19)
    - Confidence: high for the axis test, strict comparison and latched release distinction

28. **Right-click cancellation in targeting modes.** A battlefield right click, without a
    scroll drag, cancels building placement, repair, sell, power-switch, superweapon
    targeting, route-editing or beacon-placement mode instead of performing ordinary
    deselection. It creates no move, attack, sell, repair, place or beacon order. In
    particular it does not sell the building under the pointer, repair it, or put down a
    beacon. Placement cancellation removes the carried outline but leaves the completed
    building ready for placement: it is not the production-item cancellation/refund of a
    right click on its sidebar icon (S-0411 claims 5 and 10).
    Only when no targeting mode applies does the click clear selection. Ending a mode
    does not additionally perform the ordinary deselection fallback. Ordinary sell,
    repair, power-switch, route-editing and beacon modes exclude one another; this claim
    does not prescribe outcomes for impossible combinations of those modes. A right drag
    followed by release leaves the mode active under claim 27.
    - Backing: `measurable` (Scenario T20)
    - Confidence: high for cancellation versus deselection and placement-item retention,
      medium for route-editing interactions

29. **Box addition order (Q-0452).** For an ordinary box, the successful additions retain
    the relative order of the admitted candidates of claim 25 after restricting them to
    the rectangle and claim 19's eligibility. This is the same order whether the box
    replaces or adds to the selection, and regardless of drag direction. There is no
    reversal, new position sort or creation-order sort of mobile candidates at release.
    Each addition uses claim 12: the newly added positive-primary-damage objects therefore
    end up in reverse candidate order at the front, while other newly added objects retain
    candidate order at the end. In an additive box, already-selected objects are not
    added again or moved to a new position. S-0701 claim 17's selection response belongs
    to the earliest successful addition in candidate order, not the eventual first selected
    object; that response rule is already specified there.

    A type-select drag encounters its in-box live candidates in the same candidate order,
    but its additions are grouped by encountered type rather than simply being the ordinary
    box's eligible candidates. Types are expanded in the order of their first in-box
    encounter. Within each type's on-screen expansion, additions retain the candidate order
    of claim 25, including matching candidates outside the box. Already-selected objects
    are not added again or repositioned, so encountering that type again does not reorder
    its earlier additions. When the type-select reach is *map* (claim 32), the expansion
    covers the whole map, and additions within each type follow creation order among surviving
    objects.
    Claim 12's front/end rule still determines the final selection in both variants.
    The type-select drag is silent under S-0701 claim 17, independently of these orders.
    - Backing: `measurable` (Scenario T21; S-0701 claim 17 for response ownership and silence)
    - Confidence: high for ordinary and on-screen type-expansion order; medium for whole-map
      creation order

30. **Move-target marker.** A person's battlefield cell click shows one move-target marker
    when the first selected object is a mobile actor that handles the click with action *move*.
    This includes a force-move click whose action is *move*, and a move click converted into
    *attack-move to a place* by claim 5. The same rule holds for a group: later selected objects
    never add markers, and never supply one when the first object supplies none. Infantry,
    vehicles and aircraft use this rule; a building's rally-point or guard-spot click does not
    create this marker. A *no-move* click gives no marker even when claim 11 finds a nearby move
    destination. Clicks on objects do not create it, including a move towards an object and
    *attack-move on a target*. Attack, force-fire, enter, harvest, deploy and selection clicks
    do not create it merely because the resulting behaviour may involve movement.

    The marker appears as click feedback, not on arrival or when the order is applied. It marks
    the input cell, not any nearby destination subsequently chosen by claim 11; its appearance
    does not promise that an order can be queued or that the destination can be reached. An
    accepted cell click in route mode uses the same feedback rule, even though claim 3 adds
    route points rather than issuing an immediate order. A click refused before command
    dispatch creates no marker.

    Its fixed world position is the clicked cell's centre, not the precise point within the
    cell at which the pointer was released. For integer cell coordinates (*x*, *y*), its
    horizontal position is (256*x* + 128, 256*y* + 128) leptons, using exact integer arithmetic
    with no rounding. Its world height is the terrain surface height at that centre
    (S-0106 claim 2), plus 416 leptons (four height levels of 104 leptons, S-0204 Constants; an engine constant derived
    from the projection, not a data value) for an intact high-bridge deck cell. Thus a click on
    the deck shows the marker at deck height, never beneath it; the actor's own height and
    current cell layer do not set the marker height. The original adds the deck lift only
    on cells carrying the deck-entry mark of S-0212 claim 10, so an unmarked deck cell can
    show the marker at terrain height instead; this drawing-only defect is corrected without
    a quirk switch.

    Only the screen of the person who clicked shows it, never an allied or enemy house's
    orders or an AI house's orders. Claim 14's drawing-only and no-scenario-draw rule applies.
    The marker uses the imported move-feedback animation, starts unattached at that position
    with no requested loop delay and repeat factor 1, and stays there while it plays.
    Animation playback and image placement remain S-0109 claims 1, 5, 7, 16 and 26's,
    subject to claim 14's drawing-only restriction; animation effects cannot change game state.
    - Backing: `measurable` (Scenario T22, with T2 for first-object suppression); the AI-house
      and corrected unmarked-deck rules are design decisions tested on our engine, not claims
      that the original reproduces them
    - Confidence: high for cell-centre placement, action eligibility and single group feedback,
      medium for the original's unmarked-deck manifestation pending T22

31. **Superweapon aim images (Q-0742 questions 1–3).** Table CS supplies the battlefield
    images for superweapon aim actions, including refused force-shield aiming. These are
    images for the offered action, not a new targeting-legality rule: S-0513 claim 1 owns
    battlefield offers, and S-0512 claim 23 distinguishes the imported aim action from the
    effect kind. Shrouded and revealed targets use the same superweapon image. All these
    images use the cursor file and palette in Import mapping, not a superweapon's own art.

    A warp destination uses exactly the same static battlefield frame as a source pick.
    Arming the second stage after a source pick (S-0513 claim 33) therefore does not change
    the cursor picture, although the next click has a different meaning. The source marker
    is a separate animation, not a cursor frame.

    Table CS also records the radar result already specified by S-0410 claims 12 and 13,
    table C1. Radar does not simply substitute a miniature for every battlefield aim image.
    Its list-position/kind distinction and selection fallbacks remain S-0410 claim 12's,
    including the shared quirk switch. Its invulnerability, teleport and force-shield rows
    are not radar aim offers. No-force-shield is a battlefield refusal image, not a new
    radar offer. With a nonempty selection, the battlefield instead uses that selection's
    ordinary action (S-0513 claim 1); radar's selection handling remains S-0410 claim 12's.

    Static entries make zero visible frame advances per second. Animated battlefield
    ranges use claim 22's 4 presentation-clock units, nominally 64 milliseconds per frame
    or 15.625 advances per second with sufficiently frequent display updates, never a
    simulation-tick rate. Every superweapon image has the centre hot spot of claim 22.
    Radar's static miniature arrow alone uses (0, 0); a representative fallback inherits
    its ordinary radar image's timing and hot spot (S-0410 claim 13).
    - Backing: `measurable` (Scenario T23; S-0410 scenario R12 for radar)
    - Confidence: high

32. **Type-select role: tap, reach and scope (Q-1987).** The role only ever takes the acting
    house's own objects. An ally's objects, and those of every other house, never join. A
    mind-controlled or captured object that the acting house now owns is own and does join.

    *Tap.* A tap acts when the role is released. Its release time minus its press time, both in
    whole milliseconds, must be at most the tap limit of 500 (Constants): 500 counts and 501 does
    not. A longer hold does nothing at its release. The press time is that of the first press, so
    a key's automatic repeat while it is held does not restart it. Two taps have no timing between
    them. After the acting player has been defeated, a tap does nothing.

    *What a tap takes.* If the selection is a single object that is not own, the tap first
    removes it. The tap's types are the types of the objects then selected, together with the
    building type that each of them deploys into, if any. A candidate of the tap is an own object
    in play of one of those types that passes one of two tests. Either it passes the eligibility
    tests of an ordinary band box (claim 19) other than the screen: it is not deactivated, not
    controlled by a slave master, not held in a tank bunker and not tethered to a building that
    its cell holds, and a building passes only when it is vehicle-like. Or it is a building whose
    type powers a unit, which needs none of those tests. A candidate is added only if it can be
    selected (Terms) and is not selected yet, and never while it is being erased (claim 16). A
    candidate need not be on the screen: only reach *screen* below asks for screen candidates.
    A tap never adds another object of those types, an ordinary building for example.

    *Reach.* A tap keeps the reach as the previous tap left it only when, since that tap, no
    object has been added to the selection, the selection has not been cleared, and none of the
    original's further selection commands (below) has been used. Otherwise it first sets the
    reach to *screen*. Every addition counts, whatever makes it: a band box, a click, a held click,
    a type-select drag, *select group*, *add group* or *centre group* (claim 33). Every clearing
    counts too, even of a selection that is already empty, so a *select group* press that is not
    a repeat always counts. Removing single objects from the selection, by a held click or a death,
    does not count, and neither do *make group*, a repeat *select group* press, an *add group*
    press that adds nothing, or a *centre group* press that neither clears nor adds. The
    original has four further selection commands that no spec of this project defines. One of
    them selects the player's combat units, and it can also set the reach to *map*, which a held
    click then uses. The effects of the other three are not established here. Using any of the
    four makes the next tap set the reach to *screen*. Our engine has none of them, so for our
    engine the list above is complete. With reach *screen*, if at least
    one screen candidate (claim 25) that is a candidate of the tap is unselected, the tap adds
    every such screen candidate in candidate order, whatever their types, and the reach stays
    *screen*. Otherwise the reach becomes
    *map*. With reach *map*, the tap adds every candidate of the tap in play, in creation order
    among surviving objects. Each addition follows claim 12's front/end rule, and objects already
    selected keep their places. So the first tap after a selection change extends on the screen,
    and a second tap with no change between them extends across the map, however late it comes.
    A single tap goes across the map at once when every candidate on the screen is already
    selected, for example right after a box took them all. Types that match nothing anywhere
    leave the selection as it was.

    *Message.* After each tap the message list shows, in the acting player's colour, the
    nothing-selected text when the selection is empty, otherwise the across-screen text for reach
    *screen* and the across-map text for reach *map* (Import mapping).

    *Selection response.* DECIDED (autonomy, 2026-10-28): a tap gives one selection response, that
    of the first object it adds (S-0701 claim 17), as a band box does, and none if it adds nothing.
    The original instead lets every object a tap adds queue a selection response of its own.
    Responses are presentation only, so no switch.

    *Held clicks.* With the role held, a click uses the reach as the last tap left it:
    - *select* clears the selection, then adds every own object of exactly the clicked object's
      type that can be selected (Terms) within the reach: the screen candidates in candidate
      order, or every such object in play in creation order among surviving objects. Claim 12's
      front/end rule then orders them. The clicked object gets no place of its own and joins only
      if it is own. A click on another house's object of a type the player also has selects the
      player's own objects of that type, and a type the player has none of leaves the selection
      empty.
    - *toggle select* on an unselected object makes the same additions and keeps the selection.
    - *toggle select* on a selected object removes every selected own object of that type within
      the reach. With reach *screen*, selected ones off the screen stay selected.
    - The role does not change row B6's result.

    A held click asks only *can be selected*, not the band-box tests of a tap's candidates. So a
    deactivated own object of the type, or one held in a tank bunker, joins a held click's
    selection while a tap leaves it out.

    *Held drags.* With the role held, a band box does not use claim 19's candidate tests, though its
    clearing and empty-box rules still apply. Instead every live box candidate inside the box
    (claims 19 and 25), whatever its house and whatever its kind, buildings included, stands for its
    own exact type. Each such object, in candidate order, then makes the additions of a held *toggle
    select* click on an unselected object of its type (above), within the reach: own objects of that
    type that can be selected, whether inside the box or not. So an enemy object in the box adds the
    player's own objects of its type, and an own object in the box that a band box refuses (a
    deactivated vehicle, or a building that is not vehicle-like, whether or not it powers a unit)
    joins with the others of its type when it can be selected. A drag adds none of a tap's further
    types: no building type that an object deploys into. The order of the additions is claim 29's.

    Held clicks and the type-select drag of claim 29 are silent (S-0701 claim 17) and do not
    change the reach they read. One that adds an object or clears the selection still makes the
    next tap set the reach to *screen*, as above. A quick press, click and release is a held click
    followed by a tap. For a
    type that deploys into no building, the tap after a held *select* goes across the map,
    because every screen candidate of the type is then selected.
    - Backing: `measurable` (Scenario T25; part c bounds the tap limit between 400 and 600 ms,
      parts k and l test the powers-a-unit and deploys-into candidates, part m the deactivation
      and tank-bunker tests for taps and held clicks, part n the tether test). The exact limit
      of 500 and "at most" are `unmeasured`: the game reads the press and release times when it
      handles the key events, not when the key moves, so no hold is exact to the millisecond.
      The slave-master test is `unmeasured`: a tap would need a selected own object of a type
      that slave masters control, but under no slave master, and no stock route to such an
      object for the human is identified. The erasure exclusion is claim 16's. Parts o to r test
      which changes reset the reach and which do not, part r the drag's reading of the reach, and
      part s a mind-controlled object's joining, part t the objects a held drag starts from and
      adds. The original's four further selection commands
      are `unmeasured`: our engine has none of them, so no scenario here uses them.
    - Confidence: high for a tap limit between 400 and 600 ms, own objects only and the
      held-click results.
    - Confidence: medium for the held drags (the objects in the box that start an expansion, the
      joining of objects that a band box refuses, and the absence of the tap's further types),
      until part t runs.
    - Confidence: medium for the reach rule and the list of changes that reset it, until parts o
      to r run, for a mind-controlled or captured object's joining, until part s runs, the exact
      limit of 500 and its boundary, the tap's candidate test (its band-box tests, the
      slave-master test included, and the deploys-into and powers-a-unit parts), the held
      clicks' joining of objects that fail those tests, the message, and the original's
      per-object responses.
    - Confidence: low for removals by a held click or a death not resetting the reach.

33. **Control groups (Q-1988).** The local player has ten control groups (Terms), and four
    commands act on each group *N*. Their keys are player bindings (S-0213 claim 22). Each
    command acts on the key press and issues no order. Its effects on the selection, the groups,
    the view and the interface modes are given below (*Modes* for the modes).

    *Make group.* Every own object in group *N* leaves it. Then every selected own object joins
    group *N* and leaves any other group it was in. Infantry, vehicles, aircraft and buildings can
    all join. With an empty selection the command leaves group *N* empty. Objects of other houses
    that carry the number *N* keep it (the owner-change quirk below). Making a group neither
    changes the selection nor resets the type-select reach (claim 32).

    *Select group.* Unless the press is a repeat (below), the selection is cleared, and then every
    member of group *N* that can be selected (Terms) is added, in creation order among surviving
    objects, each by claim 12's front/end rule. The first member that becomes selected gives the
    only selection response (S-0701 claim 17). A group with no member leaves the selection empty.
    DECIDED (autonomy, 2026-10-28): this follows S-0701 with no switch, because the response is
    presentation only. The original gives no response at all when the first member it tries cannot
    be selected.

    *Add group.* As *select group*, but the selection is not cleared first and no press is a
    repeat. Members already selected keep their places, and the first member that becomes selected
    gives the only response. When the selection holds another house's object, the first member
    added replaces it, by the owner rule of *toggle select* (claim 12).

    *Repeat.* A *select group* press for group *N* is a repeat when the last *select group* press
    that was not a repeat was for group *N*, came less than 800 ms of real time before it
    (Constants), and the selection's own objects are exactly the members of group *N*. A repeat
    leaves the selection as it is and centres the view as *centre group* does. It does not restart
    the 800 ms, so repeats centre the view only within 800 ms of the press that last selected the
    group, and the next press after that selects the group again. *Add group*, *make group* and
    *centre group* presses neither start nor restart the 800 ms.

    *Centre group.* If the first selected object is not in group *N*, the selection is cleared.
    Then every member not yet selected is added as by *select group*, the first that becomes
    selected giving the only response. If the selection is then not empty, the view is centred on
    the group centre
    point of F9 over the selected objects. If it is then empty, as it is for an empty group, the
    view stays where it was. Quirk `quirks.group_center_keeps_extra_selection`:
    when the first selected object is in group *N*, the rest of the selection stays, objects
    outside the group included, and they count in F9 (Edge cases).

    *Membership over time.* A member keeps its group while it is carried or inside a building,
    and is not selected by a group command until it is on the map again. A deactivated member
    keeps its group and is selected like any other member that can be selected. A vehicle that
    deploys into a building passes its group to the building (S-0415 claim 6), and a building
    that undeploys into a vehicle passes it back (S-0415 claim 10). An object leaves its group when
    it is destroyed and
    removed from play, and when a vehicle enters a tank bunker (S-0606 claim 7). An object can
    also start in a group: a map's object line can give an infantry unit, a vehicle or an aircraft
    a group number, and a scripted team gives each object it takes on its team type's group
    number (Import mapping).
    Quirk `quirks.group_kept_on_owner_change`: the group number stays with the object when its
    owner changes. A member that is mind-controlled or captured away is skipped while another
    house owns it, and is a member again when it comes back. An object that the local player
    gains while it carries a number joins the player's group of that number, whichever house
    gave it the number (Edge cases).

    *Modes.* *Select group*, *add group* and *centre group* end sell, repair and power mode
    (claim 13) and beacon placement before they act. *Make group* ends none. No group command
    ends superweapon targeting or a building's placement: a recall while a superweapon is armed
    selects the group and the superweapon stays armed (S-0512 claim 25). While the options menu
    is open (S-0213 claim 23), the group keys do nothing. In a skirmish the game is paused only
    while that menu is open (no default key binding pauses it), so the group keys do not act
    during a pause.
    - Backing: `measurable` (Scenario T26, S-0512 scenario X18 part e for the armed superweapon, and
      S-0213 Scenario O19 part d for the keys). Part e of T26 bounds the repeat interval between 600
      and 1000 ms, and its third run covers a *centre group* press within it. Part k covers the
      response when the first member tried cannot be selected. The exact 800 ms and "less than" are
      `unmeasured` for the reason given in claim 32 for the tap limit. F9's branches for *n* ≤ 2,
      for a tie for the largest *d*ᵢ and for every *d*ᵢ 0 are `unmeasured`: part f centres on three
      objects with one far from the other two, part e's repeat moves the view to a group of two
      without naming the point, and no part places two objects equally far or objects at one point.
      The team type's group number is `unmeasured`: no stock skirmish map builds a scripted team for
      the human player's house. The deactivated member is `unmeasured` here: it rests on the
      selection rules (Terms, *can be selected*), which T26 does not vary. The modes that *add
      group* and *centre group* end, beacon placement and the pause are `unmeasured` here: part j
      presses only *select group*, in sell, repair and power mode and with the options menu open.
    - Confidence: high for make, select and add, one group per object, the empty-group results,
      the repeat condition, a repeat interval between 600 and 1000 ms, the end of sell, repair
      and power mode at *select group* and of none at *make group*, and a member taken away and
      given back rejoining its group. Medium for an object gained with a number joining the
      player's group of that number (T26 part i's second case, which can be void), for the
      type-select reach after a group command (T25 parts o to q), for the original giving no
      response when the first member it tries cannot be
      selected, until T26 part k runs, for the presses other than *select group* not restarting
      the 800 ms (only *centre group* is tested, by T26 part e), for the exact 800 ms and its
      boundary, the view centring and F9 (its
      branches for *n* ≤ 2, a tie and every *d*ᵢ 0 included, `unmeasured`), membership
      through deploy and undeploy, the deactivated member, and the map and team group
      numbers; low for the options menu and the pause, for the modes that *add group* and *centre
      group* end,
      and for beacon placement

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
> in table I compares the actor's integers *p* < *P* directly. Row I7's threshold comparison
> chooses only the click action, not an arrival damage/capture outcome: in skirmish an
> otherwise eligible engineer captures at any health (S-0509 claims 6 and 7).

> **F3 (attackable height).** The target can be attacked at all when *z*<sub>s</sub> > −20, where
> *z*<sub>s</sub> is its height above the surface in leptons. Integer comparison, exact.

> **F4 (passenger land-speed limit, row VD7).** The first passenger counts as unable to move on
> the cell's land type when *m* < 0.01, where *m* is the land multiplier (glossary) of the
> passenger's locomotion class on that land type. In the original, *m* is a single-precision floating-point
> value and 0.01 is the double-precision value nearest to 0.01; the comparison is made in double
> precision, into which *m* converts exactly, so it is exact and no rounding mode affects it.
> The single-precision value nearest to 0.01 is slightly below the double-precision 0.01, so a
> multiplier that becomes 0.01 when rounded to single precision counts as unable to move.

> **F5 (acknowledgement choice, claim 14).** *i* = *x* mod *n*, where *x* is the next raw value of
> the unsynchronised random stream, an unsigned 32-bit integer taken whole (S-0202, claim 1), *n*
> is the number of entries in the applicable order-response list of the type of the object that
> gives the acknowledgement (the first object of the selection, claim 14). For an order kind
> without its own response kind this is the special-attack list (S-0701 claim 18 and F6), never
> the selection list. Mod is the remainder
> of unsigned integer division (never negative). For *n* ≥ 1, exactly one raw value is taken and
> response *i* is played, the responses being numbered from 0 in the order the type lists them.
> For *n* = 0, no raw value is taken and no response is played. A single-entry sound response
> outside a list takes no choice draw. These counts exclude sound playback's own draws
> (S-0701). Integer arithmetic, no rounding, no rejection of values.

> **F6 (left-drag threshold and box).** For integer screen displacements Δ*x*, Δ*y* from the
> button-press pixel, *d* = ⌊*r*(Δ*x*² + Δ*y*²)⌋ pixels, using the table square root of
> S-0204 F1. The original forms the squared sum in double precision and truncates the root
> toward zero. Drag begins exactly when *d* > 4. For press (*a*, *b*) and final endpoint
> (*c*, *e*), an integer box point (*x*, *y*) is inside exactly when
> min(*a*, *c*) ≤ *x* ≤ max(*a*, *c*) and min(*b*, *e*) ≤ *y* ≤ max(*b*, *e*).
> The limiting of the moving endpoint to the battlefield view (claim 18) and these comparisons are exact integer operations, without rounding.

> **F7 (screen-candidate point bounds).** Coordinates are integer pixels relative to
> the battlefield view's top-left corner; *W* and *H* are its width and height in pixels,
> not the whole window or sidebar dimensions. A mobile candidate point (*x*, *y*) passes
> exactly when −32 ≤ *x* ≤ *W* + 32 and −32 ≤ *y* ≤ *H* + 32. A building point passes
> exactly when 0 ≤ *x* ≤ *W* and 0 ≤ *y* ≤ *H*. These are inclusive integer comparisons,
> with no rounding. In particular the mobile upper bound is *W* + 32, not *W* − 1 + 32.
> Projection and height-lift rounding are S-0101 claim 5 and F1's; transient mobile point
> corrections remain qualified by claim 19.

> **F8 (right-drag threshold).** Let Δ*x*, Δ*y* be integer screen-pixel displacements
> from the right-button press pixel. Let *h*ₓ and *h*ᵧ be the host's horizontal and vertical
> drag metrics in pixels. Right-drag scrolling starts when |Δ*x*| > 2*h*ₓ or
> |Δ*y*| > 2*h*ᵧ. Equality on both axes remains a click. All operations and comparisons
> are exact integers with no rounding; there is no radial distance calculation here.
> The factor 2 is an engine constant. In the original the two metrics are host input
> settings; our engine fixes both at 4 pixels (lead decision, claim 27).

> **F9 (group centre point).** Let the selection hold *n* ≥ 1 objects with positions
> *p*₁ … *p*ₙ in selection order, each three integer coordinates in leptons, height included.
> The mean *m* is the coordinate-wise sum divided by *n*, each coordinate truncated toward
> zero. For *n* ≤ 2 the centre point is *m*. For *n* ≥ 3 let *s*ᵢ be the sum of the squares of
> the three coordinate differences between *p*ᵢ and *m*, an exact integer, and let
> *d*ᵢ = ⌊*r*(*s*ᵢ)⌋ in whole leptons, where *r* is S-0204's table square root (S-0204 F1), under
> S-0204's switch `quirks.table_root_distance`: with that switch off, *d*ᵢ = ⌊√*s*ᵢ⌋. Two objects
> whose exact distances differ by less than the table's shortfall can swap which is farthest.
> Scenario T26 keeps the distances apart and does not test such a tie. Let *k* be the first object
> in selection order whose *d*ₖ is the largest, and the largest is
> above 0. The centre point is the coordinate-wise sum of all positions minus *p*ₖ, divided by
> *n* − 1 and truncated toward zero, so the object farthest from the mean is left out. If every
> *d*ᵢ is 0, the centre point is *m*. Claim 33 centres the view on the centre point; how the view
> maps a point to its screen position is S-0101's.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Left-drag threshold | 4 | pixels of truncated table-root distance | Claim 18, F6; strictly greater starts the band box |
| Right-drag metric factor | 2 | dimensionless | Claim 27, F8; applied separately to each host drag metric |
| Right-drag metric (ours) | 4 | pixels per axis | Claim 27, F8; fixed by lead decision in place of the host setting |
| Screen candidates | 500 | objects per drawn frame | Claims 20, 24 and 25; not the selected-object limit |
| Mobile candidate margin | 32 | pixels on each axis | F7; independent of image dimensions |
| Cursor frame interval | 4 | 16-millisecond presentation-clock units | Claim 22; nominally 64 milliseconds per advance, checked only on updates |
| Control groups | 10 | groups per player | Claim 33; digit keys 1 to 9 and 0 for group 10 (S-0213 claim 22) |
| Group repeat interval | 800 | milliseconds of real time | Claim 33; a *select group* press less than 800 after the last one that selected the group can be a repeat, 800 cannot. Not ticks and not a host setting. Input only. |
| Type-select tap limit | 500 | milliseconds | Claim 32; an engine constant of the original, not a host setting. Times are whole milliseconds; a release is a tap when its time minus the press time is at most 500. Input only. |
| Full health | 1 | health ratio | Tables I and V. The original fixes the threshold at 1 and does not read it from data. |
| Attackable height limit | −20 | leptons of height above the surface | F3; a target must be strictly above it |
| Passenger land-speed limit | 0.01 | land multiplier (a factor; 1 = unchanged speed) | Row VD7 and Formulas F4; a first passenger whose multiplier on the cell's land type is strictly below it counts as unable to move there, except that the test is not applied where the land is water. A data value of exactly 0.01 counts as below it, because the original stores the multiplier in single precision (Formulas F4) |

## State and ordering

| Input state | Event | Result |
|---|---|---|
| No drag armed, ordinary battlefield mode | Primary-button press | Drag armed at the press pixel |
| Drag armed | Motion with F6 distance at most 4 pixels | Still armed, ordinary click remains possible |
| Drag armed | Motion with F6 distance above 4 pixels | Box active, anchored at press pixel |
| Box active | Motion | Box endpoint changes; returning near the anchor does not disarm it |
| Box active | Left-button release | Selection result of claim 19; box ends |
| Ordinary mode, right press accepted | Right-button motion not crossing F8 | Selection unchanged; right click still possible |
| Right press accepted | Sampled motion crossing F8 | Scroll drag active; selection and targeting mode retained |
| Scroll drag active | Return to anchor, or reach view limit | Still classified as a scroll drag |
| Scroll drag active | Right-button release | Drag ends, no deselection or mode cancellation |
| Right press accepted, no scroll drag | Right-button release | Cancel targeting mode under claim 28, otherwise clear selection |

**Table C: battlefield cursor frames** (claims 21 and 22). These are frame numbers in the
imported cursor image, not action numbers. The frame range of each action and its length are
engine constants of the original, not read from the imported cursor file, which supplies only the
pixels of the frames (confidence high, backing measurable, Scenario T16's fixed-mapping step).
All multi-frame rows animate with claim 22's interval, itself an engine constant (Constants).
“Arrow” means static frame 0, hot spot (0, 0) (pixels from the image's top-left corner).

| Action | Revealed target | Shrouded target |
|---|---|---|
| nothing, idle | arrow | arrow |
| select, toggle select | 18–30 | arrow |
| move | 31–40 | 31–40 |
| no-move | 41 | 31–40 if the first selected object's type permits shroud moves, otherwise 41 |
| attack | 53–57 only with one selected unit or building whose weapon is in range of the target; otherwise 58–62 | 31–40 |
| harvest | 58–62 | arrow |
| enter, capture, recycle, enter tunnel | 89–98 | arrow |
| deploy, area attack | 110–118 | arrow |
| no-deploy | 119 | 119 |
| no-enter, no-enter tunnel | 99 | 99 |
| repair building, repair vehicle | 150–169 | arrow |
| no-repair | 190 | 190 |
| repair switch | 170–189 | 190 |
| sell | 129–138 | 149 |
| no-sell | 149 | 149 |
| power switch, no-power, heal, patrol, tote | 384 | 384 |
| sabotage, demolish, detonate | 309–318 | arrow |
| area guard | 68–72 | 68–72 |
| bomb | 204 (static) | arrow |
| no-bomb | 41 | arrow |
| disarm bomb | 369 (static) | 369 (static) |
| airstrike | 480–487 | arrow |
| attack-move to a place, attack-move on a target | 404–412 | 404–412 |
| damaging entry | unchanged previous cursor | arrow |

**Table CS: superweapon aim images** (claim 31). Frame numbers in the imported cursor
image, counted from 0, with inclusive ranges. Every single frame below is static. Every
range animates with claim 22's interval. The battlefield column names actions; the radar
column names the corresponding targeting mode subject to S-0410 claim 12's list-position
rule. These distinctions matter when imported aim actions and effect kinds differ.

| Aim action / corresponding mode | Battlefield, revealed or shrouded | Radar, revealed or shrouded |
|---|---|---|
| nuclear strike | 319–328 | 513 |
| lightning storm | 279–298 | 514 |
| iron curtain (invulnerability) | 346 | 1, miniature arrow, even with a selection |
| chronosphere source pick (teleport source) | 357 | 1 without a selection; ordinary representative radar image with a selection |
| chronosphere destination (teleport warp) | 357 | 1 without a selection; ordinary representative radar image with a selection |
| force shield | 450–459 | 1 without a selection; ordinary representative radar image with a selection |
| no-force-shield | 460–469 | no separate radar refusal image; force-shield mode uses the preceding row's radar fallback |
| paradrop, either kind | 259 | 516 |
| spy plane | 504–511 | 512 |
| psychic domination | 488–495 | 516 |
| genetic mutation | 470–479 | 1, miniature arrow, even with a selection |
| psychic reveal | 496–503 | 515 |

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
| damaging entry | The above-threshold engineer cursor/order action of row I7, producing a capture order, not a promised damaging hit on arrival (S-0509 claims 6 and 7) |
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
| idle | No usable action: a healing infantry unit over an enemy object (row I10), or a healing vehicle whose cell action would be attack (row VC1) |
| sell, no-sell, repair switch, no-repair, power switch, no-power | Sidebar modes (claim 13) |
| nothing | No action |

*No-repair* is one action: the refusal of an engineer's repair (rows I3, I5) and the answer of the
sidebar's repair mode over an object that cannot be repaired (claim 13) are the same action. It
produces no order either way.

**Table R: representative rank** (claim 1). When multiple row conditions hold, the representative has the rank in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions.

| Rank | Selected object |
|---|---|
| 0 | Not a unit or building, or berserk |
| 1 | Disabled |
| 2 | Not armed |
| 3 | An armed building |
| 4 | An armed unit whose default weapon (the weapon it would use by default, as for a healing actor, Terms) does zero or negative damage (a healing actor, or a weapon without damage) |
| 5 | An armed unit whose default weapon does positive damage |

**Table G: generic result *g* on an object** (claim 4). When multiple row conditions hold, *g* is the result in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions. In a campaign game a further rule applies to targets under shroud. Campaigns are out of scope for now, so it is not
specified. Row G2 asks only whether the actor is being erased. Unlike row G10, it does not ask
whether the target is being erased. That difference is the original's, not an omission.

| Row | Condition | *g* |
|---|---|---|
| G1 | The actor is berserk, or melee-linked while its type lacks the keep-acting flag | nothing |
| G2 | The actor is being erased | select when the target can be selected and is not selected yet, unless the actor is an own loaner, or is armed and own while the target belongs to another house. Nothing in every other case |
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
functions of the actor, the target, the modifiers held and *g*. Values *e*, *v* and *x* are given by small tables; when multiple row conditions hold, the value is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions. No row of table I reads the result of another row.
"Engineer", "spy" and the rest name type flags (Import mapping). An *engineer-friendly building*
is a repairable building, not vehicle-like, that is allied with the actor, or that belongs to a
passive neutral house, can be garrisoned and is not being erased. A *deployed in-place deployer*
is an in-place deployer in its deployed state.

*Entry verdict e.* Defined only when the target is a building allied with the actor and not being
erased. Where no row matches, *e* is undefined.

| Row | Target and actor | *e* |
|---|---|---|
| e1 | An armory; the actor of elite veterancy rank (S-0311, claim 1) | no-enter |
| e2 | An armory; the actor not a deployed in-place deployer; docking answer *accepts* | enter |
| e3 | An armory; the actor not a deployed in-place deployer; docking answer *refuses* | no-enter |
| e4 | A hospital; force-fire not held; the actor has all its hit points | no-enter |
| e5 | A hospital; force-fire not held; the actor not a deployed in-place deployer; docking answer *accepts* | enter |
| e6 | A hospital; force-fire not held; the actor not a deployed in-place deployer; docking answer *refuses* | no-enter |

So a building that is both an armory and a hospital is judged by the hospital rows whenever the
armory rows give no verdict (the actor is below elite rank, and is a deployed
in-place deployer or gets a docking answer that is neither).

**Derived result g₁.** If the entry-verdict table assigns a value to *e*, *g₁* is that value. If that table assigns no value, *g₁* is *g*.

*Deploy value v.*

| Row | Condition | *v* |
|---|---|---|
| v1 | *g*₁ is not deploy | *g*₁ |
| v2 | Route mode held | no-deploy |
| v3 | The actor's deploy weapon fires over an area | area attack |
| v4 | The actor's type deploys in place | deploy |
| v5 | Otherwise | nothing |

*Transport verdict x.* Defined only when the target is an allied unit or building whose type
carries passengers (its imported passenger count is above 0, whatever the kind of the target), not being erased and not phasing in, the actor is movable, and *v* is none of
attack, area guard and no-enter. Where no row matches (the docking answer is neither), *x* is
undefined.

| Row | Actor and target | *x* |
|---|---|---|
| x1 | The actor is a balloon-hovering type | no-enter |
| x2 | The target is a mobile unit that is moving, or a team member whose team's type lacks a flag that this spec has not identified (low) | no-enter |
| x3 | Docking answer *accepts* | enter |
| x4 | Docking answer *refuses* | no-enter |

**Derived result g₂.** If the transport-verdict table assigns a value to *x*, *g₂* is that value. If that table assigns no value, *g₂* is *v*.

*Final action.* When multiple row conditions hold, the final action is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions.

| Row | Actor and target | Final action |
|---|---|---|
| I1 | *g* is toggle select | toggle select |
| I2 | Engineer; the target carries a planted bomb visible to the acting player (S-0519 claim 10) | disarm bomb |
| I3 | Engineer; the target a repairable building, not vehicle-like, that is a bridge repair hut | repair building when S-0511 claim 21's last-candidate selection and low-run or high-link traversal offer repair, else no-repair; no owner or alliance test |
| I4 | Engineer below full health (the engineer's own health ratio, F2); the target an engineer-friendly building that is a hospital | enter |
| I5 | Engineer; the target an engineer-friendly building at full health | recycle if the building recycles, else no-repair |
| I6 | Engineer; the target any other engineer-friendly building | repair building |
| I7 | Engineer; the target a repairable building, not vehicle-like and not engineer-friendly, of a capturable type | capture if *h* ≤ *c* (Formulas F2), else damaging entry; this distinguishes click actions only, not arrival outcomes (S-0509 claims 6 and 7) |
| I8 | The target a building that this infantry can garrison under S-0508 claim 1; force-fire not held | capture (garrison) |
| I9 | Engineer; *g* is attack | no-move |
| I10 | Healing actor; the target an enemy | idle |
| I11 | Healing actor; the target is the actor itself | area guard |
| I12 | Healing actor; the target infantry below full health | heal |
| I13 | Healing actor; the target is not a unit or building, or its type carries no passengers (for a unit, its count is not above 0; for a building, its count equals 0, so a negative building count does not meet it) | *g* if *g* is move or area guard, else select |
| I14 | Vehicle thief; the target counts as a vehicle and is a train | select |
| I15 | Vehicle thief; the target counts as a vehicle and belongs to another house (allied ones included) | capture (hijack); see the harvester-truce quirk |
| I16 | Charge planter (the type flag, or the actor has the explosive-charges veterancy ability at its current rank); *g*₁ is attack; the target a building, not vehicle-like | sabotage if the building's type accepts charges and is not invisible in the game, else attack |
| I17 | *g*₁ is attack, and the weapon the actor would use against the target has the demolition-cursor flag | select if the target is a vehicle held inside a tank bunker, else demolish |
| I18 | *g*₁ is attack, and that weapon has the air-strike-cursor flag | airstrike if the target is a building that accepts charges and is not invisible in the game, else attack; the flag affects only the cursor, not the designation effect (S-0519 claim 16) |
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

Row I16 grants the *sabotage* action and order for the veterancy ability as well as the
type flag. It does not promise a charge on arrival: with `quirks.charge_needs_type_flag`
at its original default, veterancy alone plants nothing. S-0519 claim 13 owns planting
and that switch, whose off setting also permits the veterancy ability to plant.
Row I2 tests the bomb's visible state, not merely its presence. A previously ordered
disarm shot does not acquire a visibility requirement (S-0519 claims 3 and 10).

Consequences of the precedence: an immune building still gets *sabotage*, *demolish*, *airstrike*
or *attack* from rows I16 to I18, since those come before I27; an engineer told to attack gets *no-move* from I9
without the scenery check; and an infantry unit that does not deploy, clicked on itself, gets the
fallback move (row I28), usually *no-move*.

**Table V: vehicle on an object** (claim 8). The rows use *g* and the derived values *k*, *b*, *n* and *q*, each defined in full below by a small table; when multiple row conditions hold, the value is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions. Like those of table I, they are functions of the actor, the target, the modifiers held and
*g*: the letters name results, not steps, and fix no order of evaluation.

*Fire-or-deploy value k* (a vehicle unable to fire, or clicked on itself).

| Row | Condition | *k* |
|---|---|---|
| k1 | *g* is attack; the vehicle is not able to fire; it can crush (its type flag, or it has the crusher veterancy ability at its current rank) and the target can be crushed by it | move |
| k2 | *g* is attack; the vehicle is not able to fire | select |
| k3 | *g* is deploy | the result of table VD |
| k4 | Otherwise | *g* |

*Healing-actor value b.* Rows b1 to b3 apply only to a healing vehicle with *g* other than deploy, and they
exclude each other.

| Row | Condition | *b* |
|---|---|---|
| b1 | The target is an enemy | nothing |
| b2 | The target is allied and not a building, and at least one of these holds: it does not count as a vehicle; it is the actor itself; it cannot be attacked at all (F3) | select |
| b3 | The target is allied, counts as a vehicle, is not the actor, can be attacked (F3), is below full health and is not an aircraft standing on a building's cell | repair vehicle |
| b4 | Otherwise, and for every vehicle that is not a healing actor | *k* |

*Building-entry value n.* An *accepting allied building* is a building allied with the actor, not being erased,
whose docking answer is *accepts*. For a repair depot, permission is S-0314 claim 16's:
the depot's owner must also count the vehicle as allied, the depot must not be under
construction, being sold, still in its build-up state or switched off, and a contact place
must be free for this vehicle. A place assigned to this vehicle counts as available to it.
The naval-match rule of S-0504 claim 7 applies. A vehicle already standing on that very
depot gets no new acceptance. Neither full health nor zero funds prevents acceptance.
These permission tests do not replace the table's actor-side alliance and generic-action
tests. Repair after arrival belongs to S-0314 claim 6.

At an ordinary one-place depot whose contact is already held by another vehicle, an ordinary
second vehicle click with *b* = *select* keeps *select* through n4: it does not produce an
enter order or enqueue a waiting vehicle. This is not automatically *no-enter*. Claim 3
then makes it a selection click. Enter orders issued in the same tick before any contact is
reserved, including a group click under claim 2's shared-state rule, instead follow S-0314
claim 23 when executed: the vehicle whose enter order comes first in event order (S-0213) takes
contact, and a damaged second vehicle waits with queued entry. For a group click, event order is
selection order (claim 2; S-0213's rule for orders issued for a selection), so the first selected
vehicle takes contact. A full-health second vehicle does not queue. This does not grant an enter
order to a later click on an already-full depot.

| Row | Condition | *n* |
|---|---|---|
| n1 | *b* is select; the target an accepting allied building; the vehicle's movement zone is *fly* | no-enter |
| n2 | *b* is select; the target an accepting allied building that is a recycling building | recycle |
| n3 | *b* is select; the target an accepting allied building | enter |
| n4 | Otherwise | *b* |

*Transport verdict q.* Rows q1 to q3 apply only when *n* is none of attack, repair vehicle, area guard and
no-enter, and the target is an allied unit or building other than the actor, not being erased and
not phasing in, whose type carries passengers (its imported passenger count is above 0, for a building type as for a vehicle type; an absorbing building of count 1 or more reaches rows q1 to q3, one of count 0 or below does not). Row I13 for a healing actor uses a different test for a building target (a count of exactly 0 means no passengers), so a building with a negative count meets neither this condition nor I13's no-passengers condition.

| Row | Condition | *q* |
|---|---|---|
| q1 | The target is a mobile unit that is moving | no-enter |
| q2 | Docking answer *accepts* | enter |
| q3 | Docking answer *refuses* | no-enter |
| q4 | Otherwise (including a docking answer that is neither) | *n* |

*Final action.* When multiple row conditions hold, the final action is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions.

| Row | Vehicle and target | Final action |
|---|---|---|
| V1 | *g* is toggle select | toggle select |
| V2 | An in-place deployer in its deployed state; *q* is move | scenery check |
| V3 | An in-place deployer in its deployed state; *q* is attack; its type does not fire while deployed, or the target is out of range of its first weapon | no-move |
| V4 | *q* is nothing | fallback move |
| V5 | *q* is attack; the vehicle fails the deploy check | nothing |
| V6 | *q* is attack; the target's type is immune to damage | no-move |
| V7 | Otherwise | *q* |

Consequences: a healing vehicle over an enemy gets the fallback move (usually *no-move*, and a
click then moves it toward the enemy, claim 11); and an unarmed vehicle clicked on an own building
that accepts it gets *enter* with or without force-fire, by rows G10 and n3, since force-fire
gives an unarmed actor no generic *attack* (claim 6 (c)).

**Table VD: a vehicle clicked on itself** (*g* = deploy). When multiple row conditions hold, *k* is the result in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions.

| Row | Vehicle | *k* |
|---|---|---|
| VD1 | Route mode held | no-deploy |
| VD2 | It deploys into a building | deploy if that building could be placed where the vehicle stands (the vehicle itself not counting as an obstacle; for a foundation wider or deeper than 2 cells the placement is anchored one fixed neighbour step away from the vehicle's cell, low) and the vehicle has deploy readiness; else no-deploy |
| VD3 | It deploys in place | deploy if it has deploy readiness, else no-deploy |
| VD4 | Its type carries no passengers, it has no deploy weapon, and it is not held in a tank bunker | nothing |
| VD5 | It is held in a tank bunker, or its type carries no passengers and it has a deploy weapon | deploy if it has deploy readiness, else no-deploy |
| VD6 | A transport that carries no passengers now | no-deploy |
| VD7 | A transport, carrying passengers, not on a bridge, and either its first passenger could not move on the land type of the cell (a land multiplier below 0.01 there, Formulas F4, the land not being water), or the transport is in the air while its type does not hover | no-deploy; independent of the readiness exclusions of S-0507 claim 7 |
| VD8 | Otherwise | deploy if it has deploy readiness (for transports, S-0507 claim 7), else no-deploy |

**Table AC: aircraft on an object** (claim 8). When multiple row conditions hold, the final action is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions. All rows: low.

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

**Table B: building on an object** (claim 8). When multiple row conditions hold, the final action is the one in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions. All rows: low, except the factory-category tests clarified by claim 17 (high).

| Row | Building and target | Final action |
|---|---|---|
| B1 | The actor's type, or the target's type if the target is a building, is invisible in the game | nothing |
| B2 | *g* is toggle select | toggle select |
| B3 | *g* is deploy; the building is factory-marked and already marked primary | nothing |
| B4 | *g* is deploy; a gap generator | deploy if its radius is already boosted (a state of the gap generator that this spec does not define, low; Open questions), or its house's spare power exceeds the extra power drain of its type (low; how the drain is derived from the type's field is not settled, Import mapping and Open questions); else nothing |
| B5 | *g* is deploy; an absorbing building holding units it has absorbed, a building with garrison occupants, or a factory-marked building | deploy |
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

**Table GC: generic result *c* on a cell** (claim 9). When multiple row conditions hold, *c* is the result in the highest-priority applicable row. Row order specifies precedence between outcomes, not the order or method of testing conditions.

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
| GC9 | Movable, and either a condition that is not yet identified makes the original consider fog of war for this cell (low; Open questions), or the actor's entry outcome for the cell (S-0209) is *clear* or *cloaked blocker*, or the actor is a building that undeploys into a resource-gathering vehicle (low) | move |
| GC10 | Movable, an underground type, and a relaxed form of the entry test passes (low) | move |
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

**Table S: the shroud change** (claim 9). It applies to *c* for infantry, vehicles and aircraft,
and *c*′ below is the result. "Permitted" means that the actor's type may move into shroud and the
cell is inside the usable area.

| *c* | Cell shrouded | Permitted | *c*′ |
|---|---|---|---|
| nothing | either | either | nothing |
| anything else | no | either | *c* |
| patrol | yes | yes | patrol |
| neither nothing nor patrol | yes | yes | move |
| anything other than nothing | yes | no | no-move |

**Table IC: infantry on a cell.**

| Row | Infantry and cell | Final action |
|---|---|---|
| IC1 | *c*′ is move; not a jumpjet type; not an in-place deployer in its deployed state; the cell is a tunnel entrance | enter tunnel if it can use the tunnel, else no-enter tunnel |
| IC2 | Engineer; fog of war is on, and the cell shows a remembered building under fog that is a bridge repair hut | repair building when S-0511 claim 21's last-candidate selection and low-run or high-link traversal offer repair, else no-repair; no owner or alliance test (low for the fog case) |
| IC3 | Engineer; fog of war is on, and the cell shows a remembered repairable building under fog | capture (low) |
| IC4 | Healing actor; *c*′ is attack | area guard |
| IC5 | A jumpjet type; *c*′ is move or no-move; a tunnel cell | nothing |
| IC6 | An in-place deployer in its deployed state; *c*′ is move | no-move |
| IC7 | An in-place deployer in its deployed state; *c*′ is attack; its type does not fire while deployed, or the cell centre is out of range of its first weapon | no-move |
| IC8 | *c*′ is nothing; not a jumpjet type; movable | no-move |
| IC9 | Otherwise | *c*′ |

**Table VC: vehicle on a cell.**

| Row | Vehicle and cell | Final action |
|---|---|---|
| VC1 | Healing actor; *c*′ is attack | idle |
| VC2 | *c*′ is move; the cell is not shrouded; and either the cell has the *tiberium* land type (or, under fog of war, a remembered ore overlay, low) and the vehicle is a harvester, or the cell has the *weeds* land type (or a remembered fully grown weed overlay, low) and the vehicle eats weeds, or the cell has the *tiberium* land type (or a remembered ore overlay) and the vehicle's type gathers resources and processes them itself | harvest |
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
| BC2 | Its type undeploys into a vehicle and gathers and processes resources itself; *c* is move; the cell has the *tiberium* land type and is not fogged | harvest |
| BC3 | *c* is attack; no weapon in its first slot | attack |
| BC4 | *c* is attack; its projectile can hit ground targets, its type is not a pulse cannon, and it is not being sold | attack |
| BC5 | An own rally-capable building | move (the guard-spot order of part 3 of table M) |
| BC6 | *c* is attack | nothing |
| BC7 | Otherwise | *c* |

So an own rally-capable building gets *move* over every cell unless it has an *attack* to make,
with or without force-move.

**Table M: action to order** (claim 10). Every row creates a behaviour order (S-0213, claim 5)
unless the row says otherwise. "Object" means the clicked object, "cell" the clicked cell.
Actions not listed give no order. A row's "Needs" must hold at the click, or no order is made; a
dash means that no such requirement was found (low).

*As for X* in part 1 means that the action is replaced by X, and that part 2 then applies to X in
full, its Needs included. So an actor that is not able to fire gets no order from *heal* or
*disarm bomb* (as for *attack*, which needs "able to fire"), whatever the cursor showed (Edge
cases, *disarm and heal go through the attack check*). Capture, damaging entry and repair
building are as for *capture*, which needs only "movable".

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
| area guard (object) | area guard | object | nothing | able to fire and movable |
| area guard (cell) | area guard | cell | nothing | movable, and armed or of an engineer type |
| patrol | patrol | nothing | a nearby cell found by the search of claim 11 | — |
| deploy, area attack | unload | nothing | nothing | — |
| bomb, demolish, airstrike | attack (airstrike designation effects: S-0519 claims 16 to 21) | object | nothing | able to fire |
| harvest | harvest | nothing | cell | — |
| enter tunnel | move | nothing | the cell next to the tunnel entrance on its open side (low) | — |
| idle | guard, or area guard if the actor's first weapon heals | the actor itself | nothing | — |
| detonate | not a behaviour order: a detonate order (S-0213, claim 11) | | | |

The *idle* row looks at the first weapon slot, while *healing actor* (Terms) looks at the weapon the
actor would use by default. The two differ only for a type whose default weapon is not its first.

Part 3, buildings (low):

| Action | Order |
|---|---|
| attack | behaviour *attack*, target the object or cell |
| deploy | a primary order (S-0213, claim 13) if the building is factory-marked; and a behaviour order *unload* for each of these that holds: it is a gap generator; it is an absorbing building; it has garrison occupants |
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
  threshold is *damaging entry*. This is only a cursor/order distinction: it produces the same
  capture order as *capture* (table M), not a promised damaging hit. Skirmish arrival outcomes
  follow S-0509 claims 6 and 7: the multi-engineer damage rule is off, and an otherwise eligible
  engineer captures at any health, including full health.
- **Full health is exact.** Rows I4, I5, I12 and value *b* of table V test *h* = 1 and *h* < 1
  exactly (Formulas F2). Hit points are integers, so this is not a defect. No switch.
- **No-move still moves.** This is the rule, not a quirk. *No-move* is the action that the
  original's cursor shows as a refused move, and it produces a move order to a nearby cell found
  by the search of claim 11 (on an object, to the object itself when the search finds none),
  except where part 1 of table M gives no order. Only the cursor suggests otherwise, and the
  cursor is presentation (claims 21 and 22). So the table
  definition of *no-move* (table A: "a move that the cursor shows as not possible") is the whole
  rule, and a rule set that wants *no-move* to give no order changes table M, not a switch.
  Confidence: medium; `measurable` by Scenarios T7 and T10.
- **Disarm and heal go through the attack check.** The orders of *heal* and *disarm bomb* are
  made as attack orders (part 1 of table M, *as for*), and an attack order needs an actor that
  is able to fire. A medic is armed, so *heal* is unaffected. An engineer type
  that has no weapon is not able to fire, so over an object that carries a planted bomb it shows *disarm bomb* (row I2) and a
  click creates no order. `quirks.disarm_bomb_needs_weapon` (default: the original). With the
  switch off, *disarm bomb* needs only that the actor is movable, like *capture*, and its order
  is the attack order of table M (behaviour *attack*, target the object), so that an engineer
  with no weapon can start the disarming; what that order then does for an actor with no weapon
  belongs to the special-abilities specs (confidence: medium that the original behaves so,
  since the able-to-fire check of an attack order is the same for every kind of object; `measurable` by Scenario T10, line I2, which
  tries an unarmed and an armed engineer).
- **The harvester-truce exception of row I15 never applies.** The original means to stop a
  vehicle thief from hijacking a harvester of the rules' harvester list while the game's
  harvester-truce setting is on, but the exception never takes effect: whatever the target's
  type and the truce setting, a vehicle thief gets *capture* on another house's harvester.
  `quirks.hijack_truce_harvester_check` (default: the original, no exception). With the switch
  off, a harvester whose type is in the harvester list gives *select* instead while the truce
  setting is on. Confidence: medium; backing `unmeasured` in a skirmish. The approved S-0412 fixes
  the harvester truce off in every skirmish (claim 1, its table row "always off in a skirmish"),
  and its scenario K2 checks that a truce setting turned on in a test file has no effect. So this
  switch's truce branch cannot be reached in a skirmish (see Open questions).
- **Permanently disguised targets pass the disguise rule** (claim 6). A target whose type is
  permanently disguised passes the disguise rule as if it were not disguised at all, while a
  temporarily disguised one does not. `quirks.permadisguise_attack_cursor` (default: the
  original). With the switch off, a permanently disguised target is treated like any other
  disguised one. Confidence: medium; backing `measurable` (Scenario T4, the enemy spy whose type keeps the permanent-disguise flag).
- **Berserk actors show actions they cannot use** (claim 15).
  `quirks.berserk_action_cursor` (default: original). A berserk infantry or vehicle
  representative shows the cursor for its resolved action even though it can receive no order.
  For example, a berserk engineer can show *capture* over an enemy capturable building below
  the capture threshold. Off: whenever the representative is berserk infantry or a berserk
  vehicle, display the ordinary arrow (table C, frame 0, hot spot (0, 0)) instead.
  This replaces only the displayed image: tables G, I, V and the cell tables still give the
  same action, claim 3 still uses that action for command handling and selection fallback,
  and table M still decides each object's order. Thus hiding a capture cursor does not turn
  that capture click into a selection click or allow an order to the berserk actor. Other
  selected actors still resolve the click under claim 2. Representative rank and tie-breaking
  do not change. Aircraft and buildings are unaffected; the independent carrier-tote switch
  below retains its own meaning. Backing: `measurable` (T6 part B) for the original mismatch.
  The off behaviour is `unmeasured` against the original because it has no such setting.
  It is a proposed presentation rule of our engine, checked by T6 part B's off-state replay.
  Confidence: medium.
- **Inputs that never occur.** No aircraft has the generic result *area guard* on an object (row
  G6 excludes aircraft), and no building has the generic result *move* without force-move (row
  G7). Tables AC and B therefore have no rows for those inputs. Three further combinations are
  reached by no known input, and this spec does not fix a final action for them: the *Otherwise*
  rows of tables I, V and BC (rows I29, V7 and BC7) are not meant to cover them. Each is written as
  the input, then the result that a literal reading of this spec's tables gives for it. That
  reading is not a fixed final action of this spec, and it is unmeasured (no known input reaches
  it, so no scenario can observe it):
  - an *area guard* on an enemy building by a charge planter gives *sabotage* (row G6 gives *area
    guard* only on an allied object).
  - the action *select* (value *b* of table V) by a vehicle on a repair depot or tank bunker that is
    not allied and answers *accepts*: rows n1 to n3 take every allied accepting building, a
    building that is being erased never gets *select* from row G10, and no non-allied building is
    known to answer *accepts*. Read literally, table V passes *select* through rows n4, q4 and V7,
    so a literal reading gives *select*; whether the original gives something else here is not
    known.
  - a building whose cell result is *move* and that is not movable: a literal reading gives *move*
    through row BC5 (every such building is rally-capable), and part 3 of table M gives that
    building its guard-spot order.

  If a measurement finds an input that reaches one of them, a revision adds it. No unarmed actor has a generic
  *attack*, on an object or on a cell, except an infiltrating infantry type (claim 6 (c), rows GC4
  and GC5). Every derived value that can be *attack* (*g*₁, *g*₂, *k*, *b*, *n*, *q*, *c*′)
  inherits it from that generic *attack*, so no kind table needs a row for an unarmed actor told
  to attack; the unarmed infiltrator is handled by rows I24 and I25. Force-fire never gives an
  unarmed actor an *attack*, unless it is an infiltrator; it can still change an unarmed actor's
  action through rows that read force-fire directly (for example value *e*, row I8 and row G6).
- **Type-select tap limit** (claim 32). Departure: the original times the tap with the
  machine's clock. Our engine keeps the limit (a release at most 500 whole milliseconds after the
  press) but measures it between the time stamps of its own input events, which it keeps in whole
  milliseconds. Selection is not game state, so this never reaches the simulation. No switch.
- **Berserk carrier tote** (claim 15). Every other berserk object gets no order, but a berserk
  carrier aircraft can still pick up an allied vehicle. This is probably an oversight of the
  original. `quirks.berserk_carrier_tote` (default: the original, the exception applies). With the
  switch off, a berserk carrier aircraft gets no order from any click, like every other berserk
  object, and shows no *tote* (confidence low; backing `measurable`, the berserk carrier aircraft line of Scenario T6, which expects tote with the move order when the switch is on and no order when it is off).
- **Building cell attack without a weapon** (row BC3). A building with no weapon in its first slot
  gets *attack* over a cell on which its generic result is *attack*, an action it cannot carry out
  (it can only concern a type that switches between weapons). `quirks.building_cell_attack_no_weapon`
  (default: the original). With the switch off, such a building gets *nothing*, as a building with a
  weapon that cannot be used does (row BC6). Confidence low; backing `measurable`: Scenario T10's
  line *empty first slot* tries a named test type with no weapon in its first slot. If the original
  does not let such a type fire at all, the record says so and the row stays `unmeasured` (Open
  questions).
- **Infantry deploy readiness and the rearm countdown (Q-1754).** For an infantry type that
  deploys in place with an undeploy delay of 0 or more, the timer in deploy readiness (Terms) is the
  unit's rearm countdown (glossary; S-0301), not a timer of its own. Nothing about deploying starts
  it: not the press, not the start or the end of the *deploy* sequence. The countdown is started by
  each ordinary shot the unit makes, deployed or not (a drain attempt and a void shot leave it
  unchanged, S-0301 Terms and claim 15), and runs for that shot's delay (S-0301 claims 1, 2 and 8,
  which include the rate of fire, its factors and draw, and the berserk halving), not for the
  undeploy delay and not for a sequence's frame count.
  When a unit with explosive charges in *sabotage* reaches its target building, its countdown is
  replaced by S-0301 claim 19's arrival delay for the unit's second weapon, whether the charge is
  planted or refused; this is not a shot (S-0301 claim 19). The undeploy delay is a different quantity, separate from the countdown: S-0215 claims 30
  and 31 own when the unit undeploys by itself. As a result:
  - while the unit is deploying, deployed (any deployed sequence) or undeploying, a deploy press
    never gives it an order, whatever the countdown says. As far as a deploy press goes, a deployed unit
    leaves that state only by undeploying by itself (S-0215 claim 31).
  - an undeployed unit gets no order from a deploy press while its countdown runs after a shot
    (for example a primary shot while standing, or a shot made while deployed at the target of an
    attack order, which the unit fires before it undeploys), and gets one again from the first
    press after the countdown ends.
  - for an infantry unit, only S-0301's own rules change it early (claims 1 and 19; whether a
    temporal weapon's release does is open there). A move order, a stop order or
    a hit on the unit does not.

  For a type with a negative undeploy delay neither the sequence nor the countdown affects deploy
  readiness, so a deploy press reaches the unit in every sequence (S-0213, table T2's deploy rule).
  - Backing, by Scenario T24 (`measurable`): parts a, c and d for which sequences refuse the
    press and for the refusal after a primary shot; part b for the refusal after a shot made
    while deployed; part e for "nothing else ends it"; part g for the negative-delay rule in the
    sequences of the unit's deploy cycle in which a press falls.
  - Backing, by other specs: the replacement at a sabotage arrival is S-0301 claim 19's, with its
    backing (T24 has no sabotage).
  - `unmeasured`, each rated medium: a hit on the unit (the measurement setup has no house whose
    unit a scenario orders to hit *D*); that the countdown does not affect a type with a negative
    undeploy delay (T24 g need not make a shot); that a press reaches such a type in a sequence
    outside its deploy cycle (T24 g passes only through that cycle).
  - Confidence: medium until T24 runs, the same as S-0213's deploy rule (table T2), which reads
    this readiness and rests on its own scenario that has not run either.
- **Representative distance.** Formulas F1. Ties between selected objects are rare. Our engine
  computes the same value with the table square root of S-0204 (`quirks.table_root_distance`
  there).

- **Centre group keeps extra objects.** `quirks.group_center_keeps_extra_selection` (default:
  YR behaviour). When the first selected object is in the group, *centre group* keeps the rest
  of the selection, objects outside the group included, and centres the view on all of them
  (claim 33). Off: *centre group* replaces the selection with the group's members, as *select
  group* does, before it centres the view. Backing: `measurable` (T26 part f), confidence high.
- **Group number kept on owner change.** `quirks.group_kept_on_owner_change` (default: YR
  behaviour). The group number stays with the object through an owner change, so a member
  taken away and given back rejoins, and an object the player gains with a number joins the
  player's group of that number (claim 33). Off: a group belongs to the house that made it. An
  object counts as a member only while that house owns it, and *make group* by that house
  removes its earlier members wherever they are. So a member that comes back rejoins unless the
  house made the group again meanwhile, and a gained object joins no group until the gaining
  house groups it. When the gaining house makes a group of it, it leaves the earlier house's
  group, since an object is in at most one group. Backing: `measurable` (T26 part i). Confidence
  high for a member taken away and given back, medium for a gained object (part i's second case
  can be void).
- **Group centre with co-located objects.** When three or more selected objects all lie exactly
  at their truncated mean, so that every *d*ᵢ of F9 is 0, F9 takes the mean and the view centres
  on them. In the original the view jumps away from them instead: a display bug with no
  switch. Backing: `unmeasured` (no T26 part places objects at one
  point). Confidence: medium (F9's mean is our rule; the original's jump is not observed).
- **Empty box release.** `quirks.empty_box_release_click` (default: original). An unmodified
  box with no live screen candidate falls through to a click at its release pixel instead of
  clearing the selection. Off: every unmodified box replaces the selection with its eligible
  contents, so an empty box clears it and creates no battlefield order. Additive empty boxes
  remain inert. Backing: `measurable` (T14), confidence high.
- **Damaging-entry cursor retention.** `quirks.damaging_entry_cursor_retained` (default:
  original). Hovering an engineer over a building above its capture threshold retains the
  preceding cursor rather than choosing a new image; the action and capture order do not
  change. Off: it uses the capture cursor of table C. Backing: `measurable` (T16), confidence high.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Infantry type `Engineer` | engineer (tables I, IC; rows GC3 and the cell area guard of table M) | boolean (S-0003) | no |
| Infantry type `Agent` | spy (row I24) | boolean (S-0003) | no |
| Infantry type `Infiltrate` | infiltrator (claim 6, rows I24, I25) | boolean (S-0003) | no |
| Infantry type `VehicleThief` | vehicle thief (rows I14, I15) | boolean (S-0003) | no |
| Infantry type `C4` | charge planter (claim 6, row I16, table M) | boolean (S-0003) | no |
| Infantry and vehicle types, `VeteranAbilities` and `EliteAbilities` | the charge ability (row I16) and the crushing ability (value *k* of table V): the list entries named after the flags `C4` and `Crusher`; the lists, their parsing and which rank grants an entry are S-0311's (claim 3, Import mapping) | list (S-0003) | empty |
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
| Infantry and vehicle types, `Teleporter` | self-teleporting type (claim 11) | boolean (S-0003) | no |
| Infantry, vehicle and aircraft types, `MovementZone` | the zones *subterranean* and *fly* (claim 11; *fly* also in value *n* of table V); the underground type (Terms: the zone *subterranean*, in claims 6, 11 and row GC10); a building type's zone (row B10) | one of the thirteen zone names of S-0211 (claim 12), read as S-0211's Import mapping says | *normal* (S-0211, Import mapping) |
| Aircraft types, `Spawned` | spawned aircraft type (Terms, *movable*) | boolean (S-0003) | no |
| Vehicle and aircraft types, `SpawnsNumber` | spawn count (Terms, *movable*; S-0506 claim 10) | integer (S-0003) | 0 |
| Building types, `Repairable` | repairable building (rows I3 to I7, IC3) | boolean (S-0003) | yes |
| Infantry and vehicle types, `DeployFire` | fires while deployed (rows I21, V3, IC7, VC5); deploy weapon (row VD4) | boolean (S-0003) | no |
| Weapon `AreaFire` | deploy weapon fires over an area (value *v* of table I) | boolean (S-0003) | no |
| Vehicle types, `DeployToFire` | deploy-to-fire flag (Terms, *deploy check*) | boolean (S-0003) | no |
| Vehicle types, `Crusher` | can crush (value *k* of table V) | boolean (S-0003) | no |
| Vehicle types, `DeploysInto` | deploys into a building (row VD2; deploy readiness; the tap's types, claim 32) | building type name | none |
| Vehicle types, `NonVehicle` | does not count as a vehicle (Terms) | boolean (S-0003) | no |
| Vehicle types, `IsTrain` | train (row I14) | boolean (S-0003) | no |
| Vehicle and aircraft types, `Passengers` | transport (values *x* of table I and *q* of table V, rows VD4 to VD8, AC7): more than 0 | integer (S-0003) | 0 |
| Building types, `Passengers` | transport test of rows x and q for a building target (more than 0), and row I13 for a building target (equal to 0); read for building types only | integer (S-0003) | 0 |
| Building types, `UndeploysInto` | vehicle-like building, with a 1 × 1 foundation (Terms); undeploys into a vehicle (Terms, *movable*; rows GC9, BC2; table M); does not change its building candidate group (claim 24) | vehicle type name | none |
| Aircraft types, `Dock` | docking buildings (row AC10) | list of building types | empty |
| Building type `Capturable` | capturable (claim 6, rows I7, I24) | boolean (S-0003) | no |
| Building type `Spyable` | spyable (claim 6, row I24) | boolean (S-0003) | no |
| Building type `CanC4` | accepts charges (claim 6, rows I16, I18) | boolean (S-0003) | yes |
| Building type `CanBeOccupied` | garrisonable (row I8, engineer-friendly building; Terms, *armed*) | boolean (S-0003) | no |
| Building type `CanOccupyFire` | garrison occupants may fire from inside (Terms, *armed*) | boolean (S-0003) | no |
| Building type `BridgeRepairHut` | bridge repair hut (rows I3, IC2) | boolean (S-0003) | no |
| Building type `Hospital`, `Armory` | value *e* of table I, row I4 | boolean (S-0003) | no (both keys) |
| Building type `Grinding` | recycling building (rows I5, I26, value *n* of table V) | boolean (S-0003) | no |
| Building type `UnitRepair` | repair depot (row AC10; rally-capable) | boolean (S-0003) | no |
| Building type `Bunker` | tank bunker (Terms: *movable*, *deploy readiness*; rows I17, VD4, VD5; not rally-capable) | boolean (S-0003) | no |
| Building type `Factory` | imported category (claim 17): factory-marked for rows B3, B5 and table M's deploy row; only the corresponding producing kinds for rows B9 to B12 and the kind-specific rally rules | named value, S-0003 claim 21 and Import mapping row C19, including recognised non-producing categories | no factory; missing or empty in a later layer keeps the value in force |
| Building type `WeaponsFactory` | vehicle factory (row AC2) | boolean (S-0003) | no |
| Building type `ConstructionYard` | construction yard (Terms, *movable*; deploy readiness) | boolean (S-0003) | no |
| Building type `InvisibleInGame` | invisible in the game (rows B1, BC1, I16, I18); excludes the building from frame candidates (claim 24) | boolean (S-0003) | no |
| Building type `TogglePower` | can be switched off (claim 13) | boolean (S-0003) | yes |
| Building type `Powered` | the needs-power flag (claim 13; S-0405, Terms) | boolean (S-0003) | no |
| Building type `Cloning` | rally-capable (Terms) | boolean (S-0003) | no |
| Building type `UnitAbsorb`, `InfantryAbsorb` | absorbing building (row B5, table M) | boolean (S-0003) | no (both keys) |
| Building type `GapGenerator` | gap generator (row B4, table M) | boolean (S-0003) | no |
| Building type `ExtraPower` | the extra power drain of row B4 (the drain part of a negative value, low; the exact derivation is not defined here, Open questions) | integer (S-0003) | 0 |
| Building type `EMPulseCannon` | pulse cannon (rows B7, BC4) | boolean (S-0003) | no |
| House type `MultiplayPassive` | passive neutral house (engineer-friendly building) | boolean (S-0003) | no |
| Weapon `SabotageCursor` | demolition-cursor flag (row I17) | boolean (S-0003) | no |
| Weapon `MigAttackCursor` | air-strike-cursor flag (row I18) | boolean (S-0003) | no |
| Weapon `TerrainFire` | terrain-fire weapon (claim 6, row GC5) | boolean (S-0003) | no |
| Projectile `AG` | can hit ground targets (rows B7, BC4) | boolean (S-0003) | yes |
| Warhead `Wall`, `Wood` | can damage walls, wooden walls (attackable cell) | boolean (S-0003) | no (both keys) |
| Warhead `IvanBomb` | warhead that plants bombs (claim 6) | boolean (S-0003) | no |
| Warhead `Sonic` | sonic warhead (claim 6) | boolean (S-0003) | no |
| Warhead, the link flag (its key name is left out until it is on the allowed-key list) | link flag of the melee link (Terms, *melee-linked*; low) | boolean (S-0003) | no |
| Warhead `Fire` | excludes destroyable-cliff cells from the attackable cells | boolean (S-0003) | no |
| Overlay type `IsARock` | rock overlay (row GC5) | boolean (S-0003) | no |
| Aircraft type `Carryall` | carrier aircraft (rows AC2, AC3) | boolean (S-0003) | no |
| Vehicle type `Harvester` | harvester (row VC2) | boolean (S-0003) | no |
| Vehicle type `Weeder` | eats weeds (row VC2) | boolean (S-0003) | no |
| Vehicle and building types, `ResourceGatherer` and `ResourceDestination` | gathers and processes resources itself: both set (rows VC2, BC2); a resource-gathering vehicle: the first (row GC9) | boolean (S-0003) | no (both keys) |
| Building type `Explodes` | the exploding-type case that awaits removal in claim 16; destruction and the sale case are S-0312 claim 21's | boolean (S-0003) | no |
| Object type `Selectable` | type permits click and box selection (Terms, claims 16 and 19); destruction does not clear this type property | boolean (S-0003) | yes |
| Cursor image `MOUSE.SHA` | tables C and CS's frame numbers and full canvas dimensions, for battlefield and radar alike | sprite image, resolved by S-0005; frame encoding follows the sprite format | no replacement file is specified if missing |
| Cursor palette `MOUSEPAL.PAL` | cursor colours, including superweapon aim images | 256-entry palette, S-0103 | no replacement palette is specified if missing |
| Superweapon type `Action` | battlefield aim action whose image is table CS (claim 31); independent of effect kind | action word, case-insensitive, S-0512 claim 23; unrecognised word gives no action | keep previous layer's action; initially no action |
| Superweapon type `Type` and registration order | radar targeting row, not the battlefield image assignment | kind word and type-list order, S-0512 Import mapping and S-0410 claim 12 | no kind initially; radar fallback and shared quirk as S-0410 claim 12 |
| Superweapon cursor frames, timing and hot spots | claim 31 and table CS | engine constants, not per-superweapon cursor-art keys | no additional imported cursor default |
| Box threshold, candidate count, point bounds, admission priority and cursor timing | claims 18, 20, 22, 24 and 25 | engine constants and drawing state, not imported keys; neither the 500-place capacity nor the 32-pixel margin is a data field | F6, F7 and Constants |
| Physical held modifiers | force-move, force-fire and force-select (claim 23) | engine bindings, not rules keys or saved command entries | Alt, Ctrl and Shift respectively |
| Attack-move entry | claim 5 | held Ctrl+Shift, or the available command-bar button; no dedicated default keyboard command | gesture available without a command bar |
| Mouse-button scheme | claims 26–28 | no game INI/settings key exchanges left selecting/order clicks with right deselect/cancel clicks | fixed left/right roles |
| Right-drag threshold | claim 27, F8 | host drag metrics, not an INI/rules key; distinct from left-drag F6 | host-dependent in the original; ours is fixed at 4 pixels per axis (lead decision) |
| Right-drag movement and image | S-0101 claims 16 and 17 | scrolling-method preference, scroll index and imported cursor asset as mapped there | S-0101 Import mapping |
| Box addition order | claim 29 | candidate/drawing order and type expansion, not an imported selection-order key | no separate data field |
| Transient box points and crowded-view admission | qualifications of claims 19 and 20 | drawing-state behaviour, not imported selection-distance or count keys | no additional imported default; unresolved details have settling scenarios |
| Object type `Immune` | immune target (rows I27, V6, AC5, AC9, B8) | boolean (S-0003) | no |
| Object type `LegalTarget` | legal target (claim 6 (a), attackable cell) | boolean (S-0003) | yes |
| Object type `Bombable` | accepts bombs (row I19) | boolean (S-0003) | yes |
| Rules `[General]`, `MoveFlash` | move-feedback animation of claim 30 | animation type name, registered under S-0004 claim 13 | none initially; a missing or empty later-layer value keeps the current reference (S-0006) |
| Rules `[General]`, `EngineerCaptureLevel` | *c* of row I7 | decimal or percent (S-0003), stored in single precision | 1.0 (a health ratio of 1) |
| Rules `[General]` `AttackCursorOnDisguise` | attack cursor on disguised units (claim 6) | boolean (S-0003) | no |
| Rules, combat-damage section, `CanDetonateTimeBomb`, `CanDetonateDeathBomb` | row G3 | boolean (S-0003) | no (both keys) |
| Rules, combat-damage section, `TreeTargeting` | tree targeting (claim 6) | boolean (S-0003) | no |
| Rules `[General]` `AllowShroudedSubteranneanMoves` | underground moves into shroud (claim 11) | boolean (S-0003) | no |
| Infantry, vehicle, aircraft and building types, `VoiceSelect`, `VoiceSelectEnslaved`, `VoiceSelectDeactivated` | normal, enslaved and deactivated selection lists (claim 12; S-0701 claim 17), with no fallback between lists | list of sound names (S-0003), kept in the listed order | empty (no response), each list |
| Infantry, vehicle, aircraft and building types, `VoiceSpecialAttack` | special-attack order-response list (claim 14; F5); *n* counts its entries, numbered from 0 in listed order | list of sound names (S-0003), kept in the listed order | empty (no response) |
| Object type `Primary`, weapon `Damage` | primary-weapon presence and positive damage for selection order (claim 12); the weapon mapping is S-0308's | weapon type name; integer (S-0003) | no primary weapon; 0 damage |
| Rules `[General]` `HarvesterUnit` | the harvester list of the row I15 quirk | list of vehicle types | empty |
| Building types, `PowersUnit` | a building whose type powers a unit is a candidate of a tap (claim 32) | vehicle type name | none |
| Map sections `[Infantry]`, `[Units]` and `[Aircraft]`, the group field of each object line | the object's starting control group (claim 33) | integer: 0 to 9 for groups 1 to 10, −1 for none; any other value matches no group; the field's place in the line is the map format's | none |
| Team type sections, `Group` | the group number a scripted team gives each object it takes on (claim 33) | integer, as the map's group field | no group |
| String-table entries `MSG:NothingSelected`, `MSG:SelAcrossScreen` and `MSG:SelAcrossMap` | the tap's nothing-selected, across-screen and across-map texts (claim 32) | translated text from the user's string table | missing-entry diagnostic |

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
one human player and one enemy bystander computer player (glossary); a scenario that needs an ally
adds one allied bystander computer player, made mutually allied with the human player by the lobby
teams (S-0412, claim 8). The glossary's void rule applies to each bystander, and the *observed area*
of each scenario is the part of the map that holds its test objects. A scenario that clicks an
object of the allied bystander uses one of that bystander's objects, which the scenario names. The
scenarios use Alt for force-move, Ctrl for force-fire and Shift for force-select.
Scenario T17 checks those physical bindings directly. Objects are pre-placed by the map, except
the human player's buildings: a skirmish drops a pre-placed structure that a human player's house
owns (S-0009 claim 27). Each building that a scenario gives the human (T8, T25 and T26 among
them) is built by the human before the part starts, from the construction yard that its starting
vehicle deploys into. A building type that the human's side cannot build is made
buildable for that side by the scenario's named test file. The human's other starting units stay
out of the observed area, and a part that counts objects of a type across the whole map uses a
type that none of them has. A recorder logs each object's behaviour, target and destination
per tick, and each order issued.
The cursor shape is observed on screen and used only to read the representative's action.
Each test is done once, because no click draws from the scenario random stream (response-choice
and playback draws use the unsynchronised stream, which T2 part B reads). Where a
scenario says "alone", that object is the only one selected, so that it is the representative.

The cross-spec input checks use S-0506 claim 10's R6 for the launching-count exclusion,
S-0614 claims 7 and 8's P1 and P9 for the hold exclusion, and S-0504 claim 7's N7 for
naval-mismatch refusal with the amphibious exception. S-0507 claims 2, 4 and 7 supply T2,
T4 and T7 for air-transport alliance, vehicle eligibility answers and unload readiness.
S-0511 claims 17, 18 and 21 supply B12–B14 for engineer entry, repair selection and cursor
selection; S-0516 claims 26 and 36 supply CS9 part e and CS15 for cloak selection loss,
click exclusions and scenery-disguise selection. Those scenarios own the setups,
procedures and falsifiers for these shared rules.

**Scenario T1: representative (claim 1).** Select an unarmed engineer and an armed tank, with the
engineer nearer to an enemy building. Hover over the building: expected the tank's action
(attack), since rank 5 beats rank 2. Then select an armed ground vehicle and an armed amphibious
vehicle (both rank 5), with the ground vehicle nearer to a water cell, and hover over the water:
expected the ground vehicle's action (no-move), although the amphibious vehicle would show move.
Move the amphibious vehicle nearer than the ground vehicle and hover again: expected move.
Last, the tie-break: place the two vehicles so that their distances to the water cell are equal
(mirror positions, so that F1 gives the same *d*), select the ground vehicle first and then add the
amphibious one, and hover: expected the amphibious vehicle's action (move), because the later
addition goes at the front. Repeat with the selection built in the other order: expected no-move. Falsified if the engineer's action (capture)
shows in the first case, or the farther vehicle's action shows in either of the second and third,
or the cursor does not follow the selection order in the tie-break.

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
- The marker for *no-move* (claim 14, low). With the tank alone selected, click a cell of the
  passability category *blocked* (a cliff tile) inside the usable area, with open cells around it,
  and then, as control, an empty open visible cell. Expected: over the blocked cell the cursor shows
  *no-move*, the tank gets a move order to a nearby open cell and no marker appears; over the open
  cell the cursor shows *move*, the tank gets a move order to that cell and one marker appears.
  Falsified if a marker appears for the blocked-cell click, or none for the open-cell click. The
  marker is judged by eye on a recording of the click.
- Start with an own unit selected. With force-select held, click an own unselected unit, then
  click it again. Expected: it is added, then removed. Falsified if the selection does not change,
  or changes otherwise.
- Select an enemy unit by clicking it, then click an own unselected unit with force-select held.
  Expected: the own unit replaces the enemy unit in the selection. The enemy unit is not an own
  actor, so its generic result is *select* (row G10), and the click does not reach the fallback
  of *toggle select* (claim 12, which is `unmeasured`). Falsified if both end up selected.
- An own armed tower (a ground weapon), alone, over an enemy tank that is out of range of the
  weapon, without force-fire. Expected: the action is *nothing* (rows G8 (d) and G9), and a click
  selects the tank (claim 3: the clicked object can be selected and is not selected yet); no
  order is recorded. Falsified if the selection does not change or an order is recorded.
- An own power plant, alone, clicked on itself (row B6, *nothing*, and the plant is already
  selected), and then clicked on an empty cell. Expected: no selection change and no order in
  both. Falsified if either click changes the selection or records an order.
- Hold the type-select role for more than 1 s and click one own unit of the further type, once,
  with no tap made earlier in the game. Expected: every object of that type on the screen is
  selected, and none off it. Falsified if only the clicked unit is selected, or off-screen ones
  are too. Scenario T25 tests taps and the reach (claim 32).
- With the tank selected, hold route mode and click two cells. Expected: no order is recorded for
  the tank until route mode ends (claim 3). Falsified if a move order is recorded at the first
  click.
- Select a healthy own tank first and add a paralysed own tank second (a stock weapon with
  that effect paralyses it). Both types have positive primary-weapon damage, so the paralysed
  tank is first in selection order despite being unable to move. Click an empty visible cell.
  Expected: the healthy tank gets a move order, and no acknowledgement is heard and no marker
  appears (claim 14, low). Build the selection in the opposite order and click again: expected
  one acknowledgement and one marker. Falsified if the acknowledgement or marker comes in the
  first case, or does not in the second.
- Selection order (claim 12): add an engineer without a primary weapon, a tank with positive
  primary-weapon damage, a medic with negative primary-weapon damage, and a second such tank,
  in that sequence by modifier-clicks. Record selected-object order and issued move orders on
  an empty visible cell. Expected: second tank, first tank, engineer, medic. Remove the second
  tank and repeat: first tank, engineer, medic. Falsified by any other order. S-0701 D3 separately
  tests selection responses for enslaved and deactivated objects, including silence without
  fallback.
- *Part B, acknowledgement choice (claim 14; F5).* At speed index 6, no engine add-on and no mods (the named test files of a scenario excepted), use the isolated map above and a named test map override with distinguishable
  selection and special-attack lists. Give two own unit types nonempty special-attack lists of
  different lengths and a third an empty one. Use an area-guard cell click (force-move and
  force-fire together) to produce an order without its own response kind. Script one click
  with one unit selected, one with three selected, and one with the silent type first in
  selection order as defined by claim 12. Let all pending speech finish before each click.
  Record the first selected object's type, order, raw unsynchronised values, response requested
  and heard, and scenario-stream state. Separate choice draws from sound-instance draws using
  S-0701's playback rules and scenarios D2 and E1. Repeat with a move click and a nonempty
  move-response list as control. Expected: area guard chooses from the first object's
  special-attack list by F5, taking exactly one choice draw when nonempty and none when empty.
  Later selected objects give no response. The move control chooses from its move list and
  also takes one choice draw, not zero. No click changes the scenario stream, and the marker
  itself takes no draw. Falsified by a selection-list response to area guard, a choice draw for
  an empty applicable list, a choice draw per selected object, or a response inconsistent with
  F5. Selection-list variants and their no-fallback rule are tested by S-0701 D3.

**Scenario T3: modifiers and attack-move (claims 4, 5).** One tank, one friendly tank, one enemy
tank out of range, empty cells. With force-move on the enemy: the tank moves to it without
firing. With force-fire on the friendly tank: an attack order. With force-move and force-fire on
the friendly tank: area guard on it, and it follows the friendly tank. With force-move and
force-fire on a cell: area guard on the cell. With the gesture on a far cell: a move with the
attack-move memory (it engages enemies met on the way). Repeat the gesture with a building or an
aircraft added to the selection: expected a plain move order for the tank (no attack-move memory).
With the gesture over the enemy tank that is in range: an attack-move order whose target is that
tank (the *attack-move on a target* of table M). With the gesture, a medic (a healing infantry
type) over a wounded friendly infantry unit: expected an attack-move order on that unit, as claim 5 gives (low). Falsified if it gets a plain attack order, or no
order. If the command bar is
available, repeat with its attack-move button instead of the gesture; then press the button,
click a far cell (attack-move expected), and click another far cell without the gesture (a plain
move expected, since the mode ended with the first click); then press the button, clear the
selection, select the tank again and click a far cell (a plain move expected); then select a tank
and an unarmed engineer together (one ineligible object) and press the button: expected a message
and the mode is not newly armed, so that a click on a far cell gives plain moves.
T17 separately tests button entry with an empty selection. Row G3: an own
bomb planter, alone, force-fires at the own tank (rows G8 and I19 give *bomb*), so that its bomb
is attached to the tank; then select the tank alone and click it,
once with a named test rules file that turns both bomb-detonation settings on (expected: a
detonate order, and the bomb goes off at once) and once with a file that turns both off
(expected: no detonate order, and the bomb goes off only when its timer ends). The same with the
bomb attached to an own building instead of the tank, the building selected alone and clicked on
itself: a detonate order from the building (part 3 of table M). Falsified if any outcome differs.

**Scenario T4: attack rule (claim 6).** An own tank, alone, against: an enemy tank (attack), a
second own tank without force-fire (select), a tank of the allied bystander without force-fire
(no-move, by rows G9 and V4), an object whose type is immune (no-move cursor), a disguised
enemy spy, with and without force-fire, a tree, and an own unit of an underground type
(stock or test type), first on the surface with force-fire held (expected: attack, the baseline
that shows the unit can be targeted), then, after it has been ordered to move and is travelling
underground, again with force-fire held. A named test rules file clears the spy
type's permanent-disguise flag, makes the tree's type not a legal target, and is used in four
variants: the general attack-cursor-on-disguise setting off or on, and tree targeting off or on.
Expected: with the disguise setting off, the spy shows no attack without force-fire, and
force-fire attacks it; with the setting on, the spy shows attack without force-fire. The tree
shows attack only with tree targeting on or with force-fire. While the own unit is below the surface, no
click records an attack order whose target is that unit (a force-fire click there may attack the
cell, by table GC). Falsified if any of these differs.
- *Permanently disguised target (claim 6, `quirks.permadisguise_attack_cursor`).* With the
  attack-cursor-on-disguise setting off, an enemy spy whose type keeps the permanent-disguise flag
  (the control of the file above, which clears it), disguised as a unit of another house, the own tank
  alone, force-fire not held. Expected: the action is *attack* and a click records an attack order.
  Without the flag (the file above): no attack. Falsified if the permanently disguised spy shows no
  attack with the quirk at its default. With the quirk off, this case behaves like the flagless one.
- *Part B, the height limit of claim 6 (b) (needs scripted input and the recorder reading each
  object's height above the surface).* A named test file gives the clicking tank a weapon that does
  no damage, so that the clicks harm nothing. The unit of the underground type is ordered to dive
  and later to surface. The script holds force-fire and clicks it on every tick of the descent and
  of the climb, while the recorder logs its height above the surface and each recorded order every
  tick. *Expected:* an attack order whose target is the unit is recorded at exactly the ticks
  whose logged height is above −20 leptons, and none at heights at or below −20. *Falsified by:* an
  attack order at a logged height at or below −20, or none at a logged height above −20. The limit
  is fixed only to within the change of height between the two nearest logged ticks: if no logged
  height lies within a lepton of −20, the record says which two heights bracket the limit.

- *Part C, parts (d) and (e) of the attack rule.* Each case is tried alone, without and with
  force-fire, over an enemy tank. (d): an own armed tank held inside a tank bunker (it is not
  movable), with the enemy tank out of range of its weapon, and then in range. Expected: with the
  enemy tank out of range, no attack with or without force-fire: force-fire satisfies part (a)
  only and does not lift part (d), so the generic result is *nothing* (row G9), and no order is
  recorded. In range, an attack with or without force-fire (an attack order). (e): a test file
  gives an own unit a weapon that cannot hit ground targets, and gives the unit's type
  land-targeting mode 1 (with mode 0, the default, such a weapon is offered the attack on an
  object target: S-0306 claim 23). Expected: without force-fire the unit shows no attack (the
  fallback move, shown as no-move), and with force-fire it shows attack. Falsified by an attack
  where this says none (in particular an attack out of range with force-fire held), or none where
  it says attack. The bunker case is `measurable` only if a tank can be put in a bunker by the
  test map, and otherwise the record says so and part (d) stays open (Open questions).
- *Part D, a falling actor (claim 6 (e)).* Use S-0522 scenario P10's setup for orders during the
  fall (fresh human-owned payload infantry, scripted input, the record each tick), with an armed
  payload infantry type and an enemy building in weapon range of the predicted landing cell that
  the payload can neither garrison nor enter and whose type is not immune. Select only the
  falling units. During the fall the script moves the cursor over the building and logs the
  action shown, then clicks once without force-fire. In a second run it clicks once with
  force-fire held. The record notes that the units are still falling at each click. Control: the
  same two clicks after landing. Expected: during the fall, without force-fire, no attack action
  (the no-move image) and no attack order, the click recording a move order instead (table M).
  With force-fire, the attack action and an attack order on the building from the tick after the
  click, no shot before landing and the attack after landing. After landing both clicks show
  attack and give an attack order. Falsified by an attack action or attack order from the
  ordinary click during the fall, or by no attack order from the force-fire click. A run in which
  a click falls after landing, or the building leaves play before the units land, is void.

**Scenario T5: infantry rows (claim 7, table I).**
- Setup: own infantry, each tried alone: an engineer, a spy, a vehicle thief, a charge planter, a
  bomb planter, a medic, a garrison-capable infantry, an unarmed infiltrator that is not a spy,
  and a deployer infantry, deployed and not. Targets: an enemy capturable building at full
  health and at a health ratio below the capture field (a named test rules file sets the capture
  field to 50 %), an enemy building that is neither capturable nor spyable, an own damaged
  building, an own undamaged building, a bridge repair hut with its bridge destroyed and intact
  (repeat with own, allied and enemy huts), plus S-0511 claim 21's mixed-candidate and
  link-chain variants, a
  neutral garrisonable building, an allied hospital (tried with a plain armed infantry unit, hurt
  and unhurt, since a healing actor's own rows come first), an allied hospital whose type is repairable
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
  standing and no-enter when moving; the garrison-capable infantry unit gets capture (garrison)
  over the neutral garrisonable building, and a capture order, and with force-fire held over it
  not that action (row I8 needs force-fire not held). The orders: a capture order for capture,
  damaging entry and repair building alike (table M: *as for capture*, which needs only that
  the actor is movable). For the hut, *repair building* or *no-repair* follows S-0511 claim 21's
  last-candidate selection and low-run or high-link traversal regardless of hut owner.
  Include B13's mixed-candidate and link-chain variants: a different candidate choice or a
  nearest-link-only result falsifies the claim. Do not infer S-0511 claim 18's actual repair
  choice from that cursor: B12 and B13 test the two separately, and B14 tests entry effects. For the enemy
  capturable building, let the engineer arrive in both health cases, with no intervening
  owner change, sale or erasure. Expected: capture in both, not a damage-only entry above
  the click threshold (S-0509 claims 6 and 7, E4). A damage-only outcome falsifies this
  skirmish distinction. For garrison permission, repeat the hover with an allied non-owned
  building, full capacity, badly damaged building and mind-controlled entrant as in S-0508
  claim 1, G1 and G2: none passes row I8 merely because both type flags allow occupation.
  A refused repair
  click does not use up the engineer; a completed entry does, even when repair changes nothing
  at the original setting of S-0511 claim 17's switch. An intact-bridge entry remains unmeasured
  until a fixture establishes it, as that claim states.
- *Healing actor against the glossary's healer.* A named test file defines an infantry type whose
  default weapon heals and whose other weapon slot does large positive damage, so that its mean
  damage over the slots is above 0. Alone, over an enemy tank. Expected: *idle* (row I10), and a
  click gives an area-guard order on itself, not an attack. Falsified if it shows or orders
  *attack*, which would mean that the original tests the mean damage and not the default weapon.
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
- A vehicle clicked on an own repair depot. Expected: enter, and an enter order. Repeat
  at full health, at zero credits, and with a mutually allied depot: none of these alone
  prevents entry. Use S-0314 H16 a, b and d for permission controls (construction, sale,
  build-up, switched-off state, naval mismatch and already standing on that depot).
  At a one-place depot, wait until another vehicle has established contact but has not yet
  arrived, then select the second vehicle and click the depot without modifiers. Expected:
  select, the depot becomes selected, no second enter order and no waiting entry queue.
  Record the action, selection, orders and existing contact separately. A second enter order
  after the place is already assigned falsifies this case; two orders issued before any
  contact exists do not. Use the recorder and scripted input in the shared stock skirmish
  setup, three valid runs per case. See S-0314 claim 16 for permission and claim 17 for
  reservation timing. Contrast a group click with two damaged vehicles while the depot has
  no contact: expect two enter orders, one approaching vehicle and one waiting with queued
  entry, followed by entry without another click when the place becomes free. S-0314 claim 23
  and H18 f own this execution case and its full-health control. A missing second enter order
  at input, or immediate approach by both vehicles, falsifies the corresponding expectation.
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
  Clicked on itself again once it is primary: nothing, and no order (row B3).
- A carrier aircraft made berserk (by a test weapon with that effect) and an allied tank. Expected:
  tote and a move order with the tank as destination (claim 15, low). With the switch
  `quirks.berserk_carrier_tote` off: no order.
- A berserk own unit (made berserk by a stock weapon with that effect), clicked on a cell and on an
  enemy. Expected: no order on either click.
- A garrisonable building with no weapon of its own that own infantry, able to fire from inside,
  have entered (the map starts it neutral; the recorder logs its owner at each click, and the step
  counts only if the building belongs to the human player then, being an own actor. If it stays
  neutral, the record says so and the step is redone with a building that the human player owns
  and fills at the start, or, if that is impossible, stays open). Its type lets occupants fire.
  Over an enemy tank in range: expected attack, and an attack order. The same with a named test rules file that clears the type's
  occupants-fire flag: expected select, and no order. Then the garrisoned building and an own
  unarmed engineer selected together, hovered over the enemy tank: expected the building's
  action (attack), since rank 3 beats rank 2.

Falsified if any listed actor gets a different action or a different order. Results that would
show a specific error: an unload order for the empty transport (table VD), no order for the
barracks without force-move (row BC5), an order for the berserk unit (claim 15), and no attack
for the garrisoned building with the flag on (Terms, *armed*).

**Scenario T6 part B: unusable berserk cursors (claims 15 and 21).** Use the isolated
flat skirmish setup at speed 6 and 800×600, no engine add-on and no mods (the named test files of a scenario excepted). Reuse T5's
named test-map capture-threshold override. Place an own engineer, an own armed tank, an
enemy capturable repairable building below that threshold, an enemy tank and a stock
berserk-causing attacker. Select each own actor alone before berserk starts, in separate
runs, with a non-berserk actor as control. Script hover and click while the read-only
recorder confirms berserk at the input tick; record cursor images, selected identities and issued orders. Repeat
three valid runs per actor; a run where berserk ended before input is inconclusive.
Expected in the original and our engine with `quirks.berserk_action_cursor` at its default:
the berserk engineer shows *capture* (frames 89–98), its click issues no order and retains
that engineer as the selection. The berserk tank over the enemy tank shows *no-move*
(frame 41), likewise issuing no order and retaining the selection. Before berserk, the
same pairs show capture and attack respectively and issue their table M orders.
Any different berserk cursor, issued order or selection change falsifies the corresponding
original-behaviour claim. On our engine with the switch off, repeat the same recorded
input/state cases: both berserk cursors are the ordinary arrow, but the resolved actions,
selection and order results must be identical to the default cases. A changed action,
new selection fallback or issued order fails the presentation-only alternative. The
non-berserk controls must be unchanged by the switch.

**Scenario T7: cells (claims 9, 11, tables GC, IC, VC).**
- A tank and an impassable cell (a cliff tile) inside the usable area, with open cells around it:
  expected the no-move cursor (row GC11, since the entry outcome is not *clear*), and a click
  moves the tank to a cell near it found by the search of claim 11. Falsified if the click
  leaves the tank in place while a legal cell exists nearby. (An *enclosed* cell, which is
  passable in itself, is a different case: step (b) below.)
- A tank clicked onto an enemy vehicle standing in the open with force-move: destination the
  vehicle itself. Falsified if the destination is a cell.
- A harvester over visible ore (harvest, and a harvest order with the cell as destination) and
  over shrouded ore (move, and a move order). Falsified if the shrouded ore gives harvest, or the
  visible ore gives another order.
- A tank over a shrouded cell inside the usable area: expected move, and a move order. The same
  with a named test rules file that clears the may-move-into-shroud flag of the tank's type:
  no-move, and no order. Falsified if the first
  gives no-move or the second gives an order.
- *The clicked cell is the destination (claim 11).* A tank, alone, clicked on a reachable open cell
  that holds a cloaked enemy unit standing still (a test unit with the cloak property; its entry
  outcome for the tank is *cloaked blocker*, S-0209): expected the cursor *move* (row GC9) and a
  move order whose destination is the clicked cell itself, not a cell next to it. Control: the
  same click on an empty open cell (destination the clicked cell). Falsified if the first
  destination is another cell, which would mean that the original always searches. A test
  self-teleporting type clicked on the same cell: the clicked cell; the clicks of a unit of an
  underground type are the steps (a) and (a2) of the *Zones and destinations* step below.
- On a test map whose local area leaves a strip of legal cells outside the usable area (as in
  S-0212's scenario P11): a tank over a cell in the strip shows no-move, and a click orders a move
  to a cell inside the usable area; with force-fire held over the same cell the tank gets no order. Falsified if
  either differs.
- A jumpjet infantry over a tunnel cell (nothing). A vehicle that can use the tunnel (row VC3),
  and an infantry unit that can (row IC1), each alone over a tunnel entrance: enter tunnel, and a
  move order to the cell next to the entrance on its open side (low). Falsified if either
  differs.
- On a theater whose tiles include the destroyable-cliff role: an armed tank over a
  destroyable-cliff cell shows attack, and a test weapon whose warhead has the fire flag shows
  move or no-move there. The same tank over a cell holding the tile after the role's start also
  shows attack. Falsified if either does not show attack.
- On a test theater whose destroyable-cliff key is absent (the role is unset): the armed tank
  over an empty cell of tile 0 shows move. Falsified if it shows attack, which would mean the
  unset start is −1 rather than −2.
- *Zones and destinations (claim 11).* (a) A test vehicle of an underground type (zone
  *subterranean*, on the surface) and a control vehicle of zone *normal*, each alone, clicked onto a
  cell of the passability category *blocked* inside the usable area, with open cells around it (the
  cell table of S-0211 lets *subterranean* cross *blocked* and *normal* not). Expected: the cursor
  is *move* for the underground-type vehicle (rows GC9 or GC10) and *no-move* for the control. The
  underground-type vehicle, a free mover, gets a move order to the clicked cell itself; the
  control gets one to a nearby open cell. Falsified if the underground-type vehicle's destination
  is another cell, or the control's is the clicked cell. (a2) The substitution of zone *normal*:
  the same two vehicles, each alone, clicked onto a cell of the strip outside the usable area
  (the step above) whose nearest neighbours inside the usable area are blocked cells, with open
  cells a little farther. Both clicks are a *no-move* (row GC6), so both searches run, the first
  without a reach region. Expected: both destinations are open cells and neither is a blocked
  cell, although a *subterranean* zone could enter the blocked ones. Falsified if the
  underground-type vehicle's destination is a blocked cell. (b) A vehicle of
  an enemy house standing on an enclosed cell (Terms: a cell ringed by cliffs, passable in itself,
  in another reach region from the tank's): a ground tank clicked onto it with force-move gets
  a nearby cell found by the search as destination, while a jumpjet infantry unit, an aircraft
  and a unit of an underground type (each alone) get the vehicle itself. Falsified if the tank's
  destination is the vehicle, or one of the other three's is a cell. (c) Shroud: an object of
  the allied bystander standing in a region that is shrouded for the clicking player (allied vision is
  off, or the object is placed in a shrouded region of a map that allows it), clicked by an
  armed unit of an underground type, by an armed aircraft and, as control, by an armed ground tank with the
  may-move-into-shroud flag set on its type by a named test file. All three are armed, so row G9
  gives each *nothing* and never *select* (an unarmed actor would get *select* from row G10 on an
  allied object that can be selected, and the step would test nothing). Expected: no order for the first two and an order for
  the tank. Falsified if either of the first two gets an order. If no such object can be clicked on
  the test map, the record says so and this part stays open (Open questions).

**Scenario T8: sidebar modes (claim 13).** With nothing selected: sell mode over an own building,
an own vehicle on the repair depot, an own vehicle on open ground, an own aircraft (landed, and
in flight), an own vehicle-like building (Terms; a test type from a named test file if no stock
one can be placed), an own wall; repair mode over a
damaged own building twice; power mode over an own building whose type has a base drain above 0 twice, over one whose type has the needs-power flag but no base drain twice, and over an own power plant whose type has neither
(a test type from a named test file if no stock one can be placed). Expected: building
sold, no-sell over both vehicles, the aircraft in both states and the vehicle-like building
(nothing happens on a click), wall sold, repair on and then off,
the first two buildings powered off and then on again, and for the power plant *no-power* (the click records no order and S-0405 claim 10's refusal is not reached). Falsified if any of these differs; in particular, a vehicle or aircraft sold by a sell-mode
click would show where a sell unit action comes from, and a vehicle-like building sold by one
would show that the *not vehicle-like* condition of claim 13 is wrong. If the aircraft in flight
cannot be clicked, the record says so and that state stays open.

**Scenario T9: states that need a click at a timed tick (claims 3, 4 and 15; rows G1, G2, GC1, V5,
VC6).** Scripted input and the recorder (per tick: each object's behaviour, target, being-erased,
phasing-in, inside-a-tunnel and order state) are used. The script clicks on the first tick at which the recorder
logs the state, and again on a later tick while it still logs it. Each case has a control click
on the same objects before the state starts (or, in part C, with the flag named there).
- *Part A, being erased (row G2, claim 3, and row GC1 on a cell).* An enemy infantry unit with an
  erasing weapon (a temporal weapon) is in range of an own armed tank whose many hit points keep the
  lock going for many ticks. Each click below is its own run of the setup, with the tank alone in the
  selection at the click, so that the clicking object is the tank every time. The clicks are, in
  order: the *enemy click* on an enemy tank that is not selected; the *own click* on an own tank
  that is not selected; the *cell click* on an empty cell. The recorder logs the selection and each
  order after the click. Controls, in other runs before the lock starts: an attack order for the
  enemy click, the selection becoming the own tank for the own click, a move order for the cell
  click. *Expected* after the lock has started:
  - Enemy click: no order, and the selection becomes the enemy tank. Row G2 gives *nothing*, because
    the tank is armed and own and the target belongs to another house, and claim 3 then selects the
    enemy tank, which can be selected.
  - Own click: no order, and the selection becomes the own tank (row G2, *select*).
  - Cell click: no order, and the selection stays as it was (row GC1).

  The enemy click and the cell click differ from their controls in the order, and the own click
  matches its control. *Falsified by* an order from any of the three clicks, by a selection after
  the enemy click that is not the enemy tank (which would put the fallback of claim 3 in doubt for
  an erased actor), by a selection after the own click that is not the own tank, or by a changed
  selection after the cell click.
- *Part B, phasing in (row GC1).* On a flat, revealed test map, a named test map override
  gives an own armed tank the teleporting locomotor of S-0207 Import mapping, without the
  harvester flag. An enemy tank is in weapon range of its landing cell. At game speed 2,
  with no engine add-on and no mods (the named test files of a scenario excepted; here this named test data), order an own jump far enough that
  S-0207 F9 gives *D* ≥ 4 ticks. In separate runs, scripted input clicks an empty cell or
  the enemy tank on the first tick after the jump. Record the jump tick, phasing-in state,
  selection, orders and destination on every tick through the end of phasing in. Repeat
  each click case three times. *Expected:* the empty-cell click gives no order (row GC1).
  The enemy click gives an attack order (row G2 names only being erased). Phasing in ends
  at S-0207 claim 20 (f)'s tick for *D*. A fresh empty-cell click after that end gives a
  move order; the earlier refused click is not replayed. S-0207 D22 run 6 separately tests
  that a destination reaching the locomotor during phasing in is dropped rather than
  deferred. *Falsified by:* an order from the first cell click, no attack order from the
  enemy click, a different end tick, or movement towards the refused cell without a fresh
  order. A run whose recorder shows phasing in already ended at the click is inconclusive
  and is repeated with a longer jump.
- *Part C, melee link (rows G1, GC1, claim 15).* A named test file gives an own infantry type a
  weapon whose warhead carries the link flag, and no keep-acting flag; it fires at an enemy vehicle
  with many hit points. Once the recorder logs the infantry unit's target unchanged for several ticks
  (the link has formed), the script clicks an empty cell, then an enemy building. *Expected:* no
  order from either click, and the unit's target unchanged. *Control:* the same with the
  keep-acting flag set on the type: a move order from the first click. A second case swaps the
  sides: an enemy shooter with that weapon links itself to an own vehicle, and the script clicks
  an empty cell with that vehicle alone; expected no order, and with the keep-acting flag set on
  the vehicle's type a move order. *Falsified by* an order in a case without the keep-acting
  flag while the recorder shows the shooter's target unchanged at the click. If the flagged control
  also gives no order, or the unflagged shooter's target changed before the click, no link formed and
  the test file's weapon is replaced, not the claim. If the link never forms with any such weapon,
  this part stays open (Open questions).
- *Part D, a vehicle inside a tunnel with the deploy-to-fire flag (rows V5 and VC6).* The test map
  has a tunnel that vehicles can drive through, and a named test file gives a vehicle type a
  weapon and the deploy-to-fire flag. The vehicle is ordered through the tunnel, with an enemy tank
  in range of its weapon from inside it (or the recorder logs the vehicle's distance to the tank
  and the script clicks only while it is within range). On the first tick at which the recorder
  logs the vehicle as inside a tunnel, and on a later tick while it still does: a click on the
  enemy tank, and a click with force-fire held on an empty cell. *Expected:* no order from either
  click, and the vehicle's behaviour unchanged (rows V5 and VC6, *nothing*). *Control:* the same
  clicks before the vehicle enters the tunnel (an attack order for the tank click; with force-fire,
  an attack order on the cell), and with a second vehicle type that lacks the flag, inside the
  tunnel (an attack order for the tank click). *Falsified by* an order from a click while the
  recorder logs the flagged vehicle inside the tunnel, or no order from the controls. If the
  vehicle is never logged inside the tunnel at a click tick (the state is too short or the
  recorder cannot see it), the record says so, and this part stays open (Open questions).

**Scenario T10: the remaining rows, one pair each (claims 2, 7, 8, 9, 10; tables I to BC and M).**
Each line is one actor, alone unless the line says otherwise, and one target or cell, set up as
the line says on the usual flat map, and a named test rules file makes any test type the line
needs (a flag set on a stock type, a weapon removed or given, or a weapon with a flag). A line
never relies on what stock data happens to give a type: where it needs a property, the test file
sets it. An allied object in a line is an object of the allied bystander. The observable is the
cursor action and the order the recorder logs on a click, with the order given by table M. Expected: the action and order in the line. Falsified: any other
action or order.

| Rows | Actor and target | Expected |
|---|---|---|
| I2 | An engineer type that has no weapon (the test file removes any), over the own tank that carries a visible bomb attached in T3; then an engineer type given a weapon by the test file. Repeat with a bomb not visible to the acting player, using the detector-range comparison of S-0519 claim 10, T11 | disarm bomb (the cursor) and no order for the unarmed engineer; disarm bomb and an attack order on the tank for the armed one. With `quirks.disarm_bomb_needs_weapon` off, the unarmed engineer also gets the attack order. The invisible-bomb comparison must not give disarm bomb through I2. Falsified by that cursor in the invisible case, or if the unarmed engineer gets an order with the weapon quirk on |
| I9 | An engineer type that carries the infiltrator flag (the test file sets it, so that claim 6 (c) holds), force-fire held, over an enemy tank | no-move; a move order to a cell near the tank |
| I11, I12, I13 | A medic over itself; over a hurt own infantry unit; over an own tank (a type without passengers) | area guard (an area-guard order on itself); heal (an attack order on the infantry unit); select (no order, the selection changes) |
| I14 | A vehicle thief over a vehicle of another house whose type has the train flag; control: the same over an ordinary vehicle | select, no order; capture (hijack) for the control |
| I16 | A charge planter over an enemy building whose type accepts charges; then over one whose type refuses them; compare with an actor whose only charge capability is its current veterancy ability | sabotage (a sabotage order); attack. Veterancy alone also gives sabotage on the accepting building, but no charge on arrival at the original setting of `quirks.charge_needs_type_flag` (S-0519 claim 13, T13); a missing sabotage order or a planted charge in that last case falsifies this |
| I17, I18 | An infantry type whose weapon has the demolition-cursor flag over an enemy tank; one whose weapon has the air-strike-cursor flag over an enemy building that accepts charges, and over one that refuses them | demolish (an attack order); airstrike (an attack order); attack |
| I20 to I23 | An in-place deployer infantry type with an undeploy delay, deployed: over a far cell; over an own tank with force-move held; over an enemy tank out of its range (its type does not fire while deployed); over itself; then the same type while undeploying, over itself | no-move and no order; no-move and a move order to a cell near the tank (row I20; part 1 withholds only a no-move on a cell); no-move and the same kind of move order (row I21); no-deploy, no order; no-deploy, no order |
| I24 | A spy over an enemy building whose type is spyable | capture (infiltrate): a capture order |
| I26 | An unarmed infantry unit (it has no generic attack) over an own recycling building, and over an allied one | recycle (an eaten order); no-enter |
| I27, V6 | An armed infantry unit, and then an armed tank (each alone), over an enemy object whose type is immune | no-move; a move order (to a cell near the object) |
| VD3, VD4, VD5 | A vehicle that deploys in place, clicked on itself; a vehicle without passengers and without a deploy weapon, clicked on itself; a vehicle with a deploy weapon and no passengers, clicked on itself | deploy (an unload order); nothing, no order; deploy (an unload order) |
| VD7 | A loaded transport on a cell where a test file sets the first passenger's land-speed multiplier to 0.009, to 0.01 and to 0.011, one run each (Formulas F4) | no-deploy for 0.009 and for 0.01; deploy for 0.011 |
| V2, V3, VC4, VC5 | A vehicle in-place deployer in its deployed state: over an enemy tank, force-move held; over an enemy tank out of its range, force-move not held; over a far cell; over a cell out of its range, force-fire held | no-move (the scenery check) and a move order; no-move and a move order; nothing, no order; no-move, no order |
| b2 | A repair vehicle over an own infantry unit; over an own vehicle of an underground type that is below the surface (test type) | select (cursor) and no order, for both. The selection becomes the infantry unit. For the underground vehicle it becomes that vehicle if a unit below the surface can be selected; if the click leaves the selection as it was, the record says so, and the answer settles the open question on selecting such a unit (claim 3, *can be selected*) |
| k2 | A tank whose type has no crusher flag and that does not have the crusher ability at its rank, paralysed (a test weapon with that effect), over enemy infantry that is in range of its weapon | select (the selection becomes the infantry unit), no order |
| e1, e2 | A plain armed infantry unit placed by the map as elite, then one of a lower rank, each alone, over an allied armory | no-enter, no order; enter (an enter order) |
| v2 | An in-place deployer infantry type, route mode held, over itself | no-deploy (cursor); no order |
| v3 | An infantry type whose deploy weapon fires over an area (test file), over itself | area attack (cursor); an unload order |
| x1, x3, x4 | A plain infantry unit over an allied transport that is standing and has room; over a full one; then an infantry type with the balloon-hover flag over the standing one with room (test file) | enter (an enter order); no-enter; no-enter |
| I1, V1, AC1, B2 | An infantry unit, a tank, an aircraft and a building, each alone with force-select held, over an own unselected unit | toggle select: the unit is added to the selection, and no order |
| self | Two own tanks selected, the first clicked, no modifier; then force-fire held | no order for the first and a move order to a cell near it for the second; with force-fire: no order for the first and an attack order on it for the second |
| non-own actor (GC2, G10) | An enemy tank, selected alone by clicking it: over an empty cell, and over an own tank | no order; select (row G10) |
| shared state | Two own infantry units selected, clicked together on an allied transport with room for one passenger | both get an enter order, recorded in the same tick; neither is dropped |
| no side effects | The same click as the line above on a standing allied transport that accepts, once after 100 scripted mouse moves over it, once with none | the same cursor action, the same order and the same state of the transport and the units after each run |
| n1, n2 | A test vehicle of zone *fly* over an own repair depot; a vehicle over an own recycling building | no-enter; recycle (an eaten order) |
| attack-friendlies flag | A tank type that the test file gives the attack-friendlies flag, then the same type without it, each alone over an allied tank, without force-fire; then a tank type with the attack-cursor-on-friendlies flag and an ordinary weapon over an allied tank, and over an allied transport that carries passengers | attack (an attack order) with the flag and no-move without it; for the second type attack over the tank and over the transport (the exception needs a bomb-planting warhead) |
| plants-bombs exception | A tank type with the attack-cursor-on-friendlies flag whose weapon has a warhead with the bomb-planting flag (test file), over an allied tank without passengers, and over an allied transport that carries passengers | attack over the tank; no-move over the transport (the exception of claim 6 *May attack as a friend*) |
| prevent-attack-move | Two own tanks, the gesture held, a far cell; then the same with the test file giving the type of one of them the prevent-attack-move flag | an attack-move order for each; a plain move order for both, because one ineligible object turns the gesture off (claim 5) |
| q1, q2, q3 | A tank over an allied transport that is moving; standing; full | no-enter; enter (an enter order); no-enter |
| AC2 | A carrier aircraft over an allied tank that stands on a cell of a vehicle factory (the map places it there) | nothing, no order; with the tank elsewhere: tote (T6). If the map cannot place a tank on a factory cell, the record says so and the row stays open (Open questions) |
| AC6 | An aircraft and an own tank selected together (both selected so that row G10 gives no *select*); click the tank. The observable is the order logged for the aircraft, not the cursor | the aircraft gets the cell action of the tank's cell (a move order to it) |
| AC7, AC9, AC11 | A transport aircraft holding a passenger, clicked on itself, and then one holding none; an armed aircraft over an enemy object whose type is immune; an armed aircraft over an enemy tank | deploy (an unload order); nothing, no order; no-move, no order; attack (an attack order) |
| refusing building | A second plain infantry unit over an allied armory, and over an allied hospital (a hurt unit), while the first unit is still entering it | the cursor is recorded, with the rows it matches (no-enter for e3 and e6, enter for e2 and e5); this records the refusing state if there is one and is not a claim |
| empty first slot | A test building type of the weapon-switching kind with nothing in its first slot and a weapon in a later one, force-fire held, over a cell; the same with a test aircraft type if it can be given that behaviour; control: the first slot filled | attack (row BC3, quirk on), an attack order for the building if it can fire; for the aircraft *nothing* (table ACC) and for an object target *nothing* (row AC8); control: attack from the building (row BC4). If the type cannot fire at all, the record says so |
| B1 | An own armed tower (a ground weapon, the target in range) over a building of a test type with the invisible flag (the script puts the cursor on its cell); control: over a visible enemy building | nothing, no order; attack, an attack order. If the click never finds the invisible building as the clicked object, the record says so and this half of B1 stays open |
| B4 | A gap generator clicked on itself with spare power above the extra drain of its type; with less; with the radius already boosted | deploy (an unload order); nothing; deploy |
| B5, B6 | A building holding garrison occupants, clicked on itself; a power plant, clicked on itself | deploy (an unload order); nothing |
| B7, B8 | A building armed only with an anti-air weapon, force-fire held, over an enemy tank; a building with a ground weapon over an enemy object whose type is immune | nothing; no-move, no order |
| B9 to B12 | A factory of vehicles, force-move held, over an own vehicle standing on open ground; on a bridge; on a cell of a category other than *open*; on water | move (a guard-spot order); move (a guard-spot order); no-move, no order; no-move, no order. A test type that is also a repair depot and builds aircraft, over an own vehicle on open ground: move (B9) |
| GC3, GC5, GC7 | A tank, force-move and force-fire held, over a cell that holds a route point of route mode; a tank with a terrain-fire weapon over a rock overlay; a tank, force-select held, over a visible cell | patrol (a patrol order); attack (an attack order); move (a move order) |
| GC10 | A unit of an underground type, over a cell where a normal entry fails and the relaxed test of GC10 would pass (a cell of category *blocked* inside the usable area) | move (a move order); falsified by no-move |
| IC4, IC6, IC7, VC1 | A medic, force-fire held, over a cell; a deployed infantry in-place deployer, over a far cell and over a cell out of its range with force-fire held; a repair vehicle, force-fire held, over a cell | area guard (an area-guard order); no-move, no order; no-move, no order; idle (an area-guard order on itself) |
| BC2, BC4, BC6, BC7 | A building whose type undeploys into a vehicle and has both resource flags set (it gathers and processes resources itself), over visible ore; a ground-weapon tower, force-fire held, over a cell; an anti-air-only building, force-fire held, over a cell; a power plant, over a cell | harvest (a guard-spot order and a sell order); attack (an attack order); nothing; nothing, no order |
| GC4 | A tank, force-fire held, over an empty cell; a tank whose weapon's warhead has the wall flag over a wall of an enemy house, and over a wall of its own house; a tank over a cell that holds a live veinhole monster (the map places it; if it cannot, the record says so) | attack (an attack order on the cell); attack (an attack order on the cell); not attack (move or no-move); attack (an attack order whose target is the monster) |
| VC2 | A weed-eating vehicle over a cell with the *weeds* land type; a vehicle that gathers and processes resources itself over visible ore | harvest (a harvest order with the cell as destination); harvest |
| IC8 | A berserk infantry unit (a test weapon with that effect), alone, over a cell | no-move (the cursor); no order (part 1 of table M) |

**Scenario T11: the side of *allied* (Terms, *Allied, enemy*).** The one one-way alliance of a
skirmish is the special house's (S-0412): it allies the non-civilian houses, and no house allies
it. Setup: the map gives the special house a capturable building (placed below the capture field of
the named test rules file that sets it to 50 %), and gives the enemy bystander one of the same
type, also below the field, as control. An own engineer, alone, is hovered over and clicked on
each. Expected: the action over both is *capture* and the click records a capture order, because the
actor's house does not ally the special house, so its building is an enemy. Falsified if the
special house's building shows anything other than what the control shows (an engineer facing an
allied building gets enter, no-repair or repair building, not capture), which would mean that the
alliance of the special house with the actor's house, and not the reverse, decides. If the map cannot give the special house a capturable building, the
record says so and the side stays `unmeasured` (Open questions).

**Scenario T12: selection after destruction (claim 16, Q-0081 parts 3 and 4).**
- Setup: the usual stock-data skirmish map and bystander isolation, no engine add-on and no mods (the named test files of a scenario excepted),
  speed index 6, one own
  visible selectable building of an exploding type, one own visible selectable sellable
  building, and an enemy attacker in the observed area. Use scripted input and a read-only
  recorder of health, presence in play, selection and foundation occupancy per tick. Keep
  other units off the clicked foundation cells. Arrange a killing hit after the building's
  own turn, verified by the recorder, so that it remains until the next input opportunity.
- Procedure: clear the selection and click the exploding building's foundation while the
  recorder logs it as still in play with zero health. Repeat in a separate run after starting
  a sale of the second building and killing it during the sale. In each run include a control
  click before the killing hit and a click after removal. Record the selection immediately
  after input, before the removal turn, not just at the end of that tick. Try once per case
  with the same fixed seed; repeat with adjusted attack timing if no click fell in the interval.
- Observable: which object the click selects, its health and presence at the click, and the
  removal tick compared with an otherwise identical run without the click.
- Expected: the living control and the awaiting-removal click select the building; its health
  remains zero in the latter case, its removal is not delayed, and the post-removal click
  cannot select it. The selling variant pays no refund and does not resume selling (S-0312).
  Refusal of selection while the building is present, visible, selectable and not being
  erased falsifies the claim. A run with no click inside the interval is inconclusive, not
  evidence of refusal. The measurement procedure supplies the specified input and records the listed state and selection observables under the measurement setup.

**Scenario T13: non-producing factory categories (claim 17).** Use the usual isolated test map,
stock executable, speed index 6, no engine add-on and no mods (the named test files of a scenario excepted), and named test data as in
T10 and S-0003 V16. The test data gives
an own visible selectable building a recognised non-producing factory category from S-0003
C19, no weapon, no passengers and no other deployment ability. Select it alone and click it
on itself before it is marked primary: expected *deploy* and a recorded primary order, not
production eligibility. After that order applies, click it on itself again: expected *nothing*
and no further primary order (row B3). Repeat the first click with the explicit no-factory
value from S-0003 Import mapping C19 as a control: expected no primary order. Thus recognised
non-producing categories are distinguished from no factory as well as from producing kinds.
In separate runs give that same category a repair-depot flag, a
cloning flag, or neither, and click an empty visible reachable cell. Expected: *move* and a
guard-spot order in the first two runs, no order in the third. Record cursor, issued orders
and imported category, not the availability of production as a proxy for the click. Falsified
by absence of the first primary order, another primary order after the designation has
applied, a primary order from the no-factory control, loss of the guard-spot order in either
independently rally-capable variant, or a rally order from the category alone. The measurement procedure supplies the specified input and records the listed orders and imported category under the measurement setup. These named-data variants are not stock-data measurements.

**Scenario T14: band boxes (claims 18 and 19, Q-0278).** Use the usual isolated stock-data
map, 800×600, speed 6, no engine add-on and no mods (the named test files of a scenario excepted). Place own infantry, ordinary tanks, harvesters,
aircraft on the ground and in flight, an ordinary building and a vehicle-like building, plus
an enemy tank. Use scripted input and video plus a read-only selection/order recorder. One
run per sub-case; repeat ambiguous input samples.
- From a fixed press pixel, move by (4, 0), (3, 3), (5, 0) and (4, 4) pixels in separate runs.
  Expected: the first two remain clicks, the latter two begin boxes under F6. After beginning
  a box, move back inside the threshold and release: it remains a box. Test the same object
  with a box edge through its drawing origin and then one pixel beyond it; expect inclusive
  origin membership, not image intersection. Repeat with an airborne aircraft, enclosing
  its lifted origin but not its shadow, then its shadow but not its origin.
- Enclose the mixed group. Expect only eligible own mobile objects and the vehicle-like
  building, not the ordinary building or enemy. Repeat while the harvester is tethered at
  its refinery, and while a slave is controlled by its master: neither is added. Existing
  own selected units outside the box are removed without force-select and retained with it.
  Already-selected units inside an additive box remain selected, rather than toggling off.
  Hold force-fire together with force-select: expect the same additive selection.
  Repeat around an occupied own tank bunker and its vehicle after confirming occupancy:
  neither is newly added by the ordinary box. A successful addition falsifies that exclusion.
  a separate click-selection outcome does not test the band-box rule.
- With a tank selected outside the box, enclose only the enemy tank or ordinary building:
  expect the old selection cleared and none added. Enclose a completely empty open region
  and release over a reachable empty cell: expect the old tank retained and a move order
  from the fallback click. Repeat with force-select: old selection retained, no order.
  With the empty-box quirk off on our engine, the unmodified case clears and gives no order.
- A named test-map trigger shrouds an own test unit's cell without removing it. Release a
  box while the read-only recorder confirms the cell shrouded and its screen candidate
  present: expect it selected. If no such interval exists, this sub-test is inconclusive,
  not evidence of a release-time shroud gate. Repeat a type-select drag over two own unit
  types with same-type units elsewhere on screen: expect the type expansion of claim 12
  and no response, rather than only the units inside the box.
Any differing threshold, membership, modifier effect, empty-box result or additional
shroud-only refusal falsifies the corresponding claim. Record press and motion samples:
the threshold can be crossed between displayed frames.

**Scenario T15: selection capacity (claim 20, Q-0278 part 4).** At speed 6 and 800×600 on
an isolated map, no engine add-on and no mods (the named test files of a scenario excepted), pre-place 520 own selectable mobile units in two
non-overlapping groups that can be viewed separately. Select 300 in one view, scroll, then
add 220 in the other with force-select. Record selection count, not how many obey a move
click. Expected: 520 can be selected. In a separate run show all 520 at once and box the
whole view: at most 500 can be newly selected; other screen candidates can lower that
count. Repeat with 130 individually selected units and issue an order: only 128 can have
orders pending, as S-0213 O2 tests independently. A selection that cannot exceed 500 across
views, or a single ordinary box admitting more than 500 objects, falsifies the count claims.
*Part B, candidate priority (claims 24 and 25; Q-0392):* repeat the crowded run with
490 own mobile units and 20 ordinary buildings whose points are inside the view.
Expected: all 20 buildings consume places, so exactly 480 otherwise eligible mobile
units can be added by the full-view box. With 500 qualifying mobile units and one
vehicle-like building, the building is admitted and only 499 mobile units are admitted.
The building itself can also be added if it passes claim 19. Record candidate identities
separately from successful box additions, using the read-only recorder and tick-labelled
video. Keep the candidate total, eligibility and view stable until release.

Then use 510 mobile units with no qualifying buildings: split them between own and
enemy houses, then between ground and air/top render layers. Expected: the admitted set
is the first 500 qualifying mobile drawings in the frame's S-0102 order, not the first
500 own objects or the nearest objects to the box centre. Ground candidates take places
before air/top candidates. Use stationary ground units at distinct sort keys, then at
equal keys with reversed placement order; move a unit through the ordering lag of
S-0102 claim 6. Repeat the upper-layer case with reversed layer-entry order and record
any re-entry rather than assuming creation order equals flight order. Repeat each case
three times, then exchange ownership and make small view scrolls while keeping the
same points within the candidate bounds. Ownership alone must not change priority.
Scrolling may change which points qualify, not the relative priority of unchanged
qualifying drawings. For building order, use more than 500 qualifying buildings on a
named test map at a larger view size that fits all their points, reversing creation
order, then removing an early building and creating
another: surviving earlier-created buildings precede the new one. Any different
admitted prefix, mobile-before-building result or eligibility-based replenishment
falsifies the corresponding claim. Unrecorded flight re-entry makes that comparison
inconclusive, not evidence for a new tie-break.

**Scenario T18: frame membership at the view edges (claim 24, F7; Q-0392).** Use an
isolated purpose-made skirmish map, speed 6, 800×600, no engine add-on and no mods (the named test files of a scenario excepted), with one
bystander computer player kept outside the observed area under the glossary's void
rule. With scripted scrolling, put an ordinary stationary mobile unit's recorded box
point successively at −33, −32, −1, 0, the view dimension, the dimension + 32 and the
dimension + 33 on each axis; keep the other coordinate well inside. Read candidate
membership and points after each drawn frame, not just the visible pixels. Use three
replays of each position. Expected: the inclusive interval from −32 through dimension
+ 32 admits the drawn mobile object; the two outer samples do not. A partially visible
large vehicle with its point outside the interval remains absent; an entirely clipped
small body inside the interval still consumes a place. Named test art may enlarge the
body without changing the unit's position to separate image intersection from point
membership. This named-art run is separate from the stock-data run.

Repeat with an ordinary building at −1, 0, the view dimension and dimension + 1.
Expected: only 0 through the dimension inclusive qualify, independent of foundation
extent. Repeat with a vehicle-like building: it uses those same building bounds.
A named map override that makes a test building invisible excludes it; making it
unselectable does not. Use the existing Import mapping rows for these two flags.
Repeat with an enemy building, a selected own object and a deactivated own unit whose
body is still drawn: later selection refusal must not remove a qualifying candidate.
Compare an on-map building before and after its cell is revealed from shroud: its
candidate membership depends on its point and invisible-type flag, not that reveal.
Ordinary trees, a wall, resource overlays and visible animations added to the same
scene must not increase candidate count. In a separate run keep all unit and building
points inside their bounds and let one stationary shooter fire: log projectile and
particle presence alongside the candidate set. Named test-map weapon data may supply
visible particles if the stock weapon does not. Neither the projectile nor any particle
or particle-system effect may appear as a candidate or displace a mobile witness at
capacity. The live veinhole monster of T10's GC4 run likewise must consume no candidate
place; if that map cannot place it, this sub-test is inconclusive. Put a candidate behind
a tall building to confirm that occlusion of every body pixel does not by itself remove the candidate.

The read-only frame recorder is the primary observable for off-view points that cannot
be boxed directly. As an ordinary-input cross-check, add a crowded group with a late
own mobile witness: an earlier qualifying off-view candidate can displace that witness
at the 500-place boundary, without itself lying in the released box. Its removal from
the margin restores the witness. Record the full ordered candidate set to avoid
mistaking another object's displacement for the witness's. A different membership
boundary, dependence on opaque-image intersection, or any place consumed by an ordinary
terrain object, overlay, animation, projectile, particle effect or veinhole monster
falsifies the relevant rule. A mobile state in
which body drawing is suppressed is not a test of the margin; use an ordinary stationary
control and record the suppression separately.

**Scenario T14 part B: settling transient box points (claim 19).** Use T14's setup,
scripted input and read-only recorder, at fixed view position. Track one own vehicle
through factory exit, in-place deploying and undeploying, sinking, and magnetic lift
and release, in separate stock-data runs. Use an ordinary stationary vehicle as control.
At each reachable state, obtain the smallest inclusive rectangle that adds the object
by moving each edge one pixel per independent replay; record its world position,
height lift, drawing origin, eligibility and admitted screen point at release. Repeat
three times per state, with no unrelated unit touching the box. Expected established
rule: for an eligible admitted object, the boundary is its one admitted point, not its
image extent; the stationary control agrees with the normal projection. Any inclusion
outside that point falsifies the established inclusion rule. The correspondence of
transient points to projection and drawing corrections is unmeasured: record the pixel
difference rather than assuming zero. A state excluded by eligibility cannot settle its
point by selection alone; record that state as inconclusive and use the read-only
screen-point record to distinguish displacement from refusal. A consistent difference
across the three runs supplies the state-specific formula for a follow-up revision.

**Scenario T17: physical gestures and mode entry (claims 5 and 23, Q-0286).** Use the
usual isolated flat skirmish map at 800×600, speed 6, no engine add-on and no mods (the named test files of a scenario excepted), with two
own armed tanks, an enemy tank and an own unarmed engineer. Start with fresh keyboard
preferences. With one tank selected, try left and right Alt separately while clicking
the enemy tank: expect move, not attack. Try left and right Ctrl while clicking the
other own tank: expect attack. Try left and right Shift while clicking that tank twice:
expect addition then removal. Repeat with the keys released: expect ordinary resolution,
not a lingering force gesture. Hold Ctrl+Shift and click a distant open cell: expect
attack-move; release either key before a separate click: expect ordinary resolution.
Add the engineer and repeat Ctrl+Shift: expect no attack-move for either object. An
additive Shift or Ctrl+Shift box retains already-selected units instead of toggling them.
Record input events, selection and issued orders; any differing role or latching
falsifies the key rule. Repeat after changing the saved stop-command binding: the
physical modifiers stay the same.

If the command bar is available, press its attack-move button with the armed tank alone,
then click two distant cells without modifiers: expect attack-move then plain move.
Clear the selection after button entry: the next move is plain. With tank and engineer
selected and mode initially off, press the button: expect a refusal message and plain
moves on the next click. With nothing selected, press the button: expect a no-selection
message, the mode armed but no unit order, and the next battlefield release clearing
it. Record the mode with the read-only recorder as well as video; warning alone does
not distinguish the empty-selection case from refusal. Repeat each button case once.
Any different mode state, order or expiry falsifies the corresponding entry rule.
If the profile supplies no command bar, record the button cases as inconclusive rather
than substituting a guessed keyboard shortcut.

**Scenario T16: cursor images (claims 21 and 22, Q-0277).** Use the usual 800×600 isolated
map with speed indices 2 and 6 in separate runs, no engine add-on and no mods (the named test files of a scenario excepted). Reuse T3–T10's
actor/target pairs to obtain every table C action; named test-map overrides are only those
already needed by those scenarios. Hover without clicking, capture the pointer including
its input pixel, and compare each displayed image to the user's cursor-image frames.
Expected: table C's range and hot spot. For attack, compare a single shooter in range,
the same shooter out of range, and a two-shooter selection: only the first uses 53–57.
For damaging entry, first hover open ground and then the above-threshold building, and
repeat after hovering a capture target: expect the previous image retained. On our engine
with the retention quirk off, expect 89–98 instead. Record otherwise reachable shrouded
cases separately; do not infer cursor assignments from an order that was never offered.
Record ten advances of select, move and attack cursors at both speeds, with a wall-clock
video timestamp and presentation-update times. Expected: one frame per qualifying
4-unit clock interval, wrap at the range end, no restart on unchanged hover, first frame
on kind change, and no game-speed scaling. A deliberately delayed display update advances
only one frame. Any frame outside its range, wrong hot spot, simulation-tick pacing or
multi-frame catch-up falsifies the corresponding rule. *Fixed mapping:* with a loose copy of the
cursor file whose frame count is changed at its end (named test data, removed after the run), the
attack and move cursors still show the frame ranges of table C. A range that follows the file's
frame count falsifies the claim that the mapping is fixed. Unreachable table C cases remain
`unmeasured`, with the reason given in claim 21, rather than being forced with modified
executable behaviour.

**Coverage of table M.** Each case gives its order in these steps. *Move*, *no-move*: Scenarios T2, T7
and T10 (lines I9, I27 and V6, self). *Attack-move to a place* and *attack-move on a target*: T3.
*Attack*: T2, T6 and T10. *Enter*: T6 and T10 (lines x1 and *shared state*). *Capture*, *damaging entry* and
*repair building* (as for *capture*), and *capture (garrison)*: T2 and T5. *Recycle*: T10 (rows
I26 and n2). *Sabotage*: T10 (I16). *Area guard* on an object and on a cell: T3. *Patrol*: T10 (GC3).
*Deploy* and *area attack* (unload): T6 and T10 (lines v3, VD, AC7, B4, B5). *Bomb*: T3.
*Demolish*, *airstrike*: T10 (I17, I18). *Harvest*: T7 (the order) and T10 (VC2, BC2). *Enter
tunnel*: T7 (the order). *Idle*: T2 and T10 (VC1). *Detonate*: T3. *Heal* and *disarm bomb* (as
for *attack*): T10 (I12, I2). The self-target cases of part 1: T10 (line self). *Tote*: T6.
Part 1 aircraft cases: T6 and T10 (AC7, AC9, AC10 in T6). Part 3 (buildings): *attack* in T6
(garrisoned building), *deploy* in T6 and T10 (B3 to B6), *detonate* in T3, *move* on an object and
on a cell in T6 and T10 (B9 to B12, BC2). The part 1 row for berserk and melee-linked actors: T6
and T9 part C.

Rows with no step, `unmeasured`, each for a reason.
- B1 (its actor half) and BC1: a building whose type is invisible in the game is believed not
  to be selectable (low), so no click makes it the acting building. The target half of B1 has a
  step.
- AC4: it needs an own selected unit that is disguised as an overlay, standing on a cell where the
  aircraft's own cell action is *no-move* (a shrouded cell, for an aircraft type that may not move
  into shroud). No disguise as an overlay type is known in the setup (Open questions).
- AC5: it needs an own selected, immune object standing on a cell that is attackable without
  force-fire: a destroyable-cliff cell, a wall cell of an enemy house, or a cell with a live
  veinhole monster. A ground object cannot stand on a wall cell, and whether one can stand on
  the other two is not known (Open questions).
- Rows e3 and e6 (an armory or hospital whose docking answer is *refuses*): which states make
  such a building refuse is S-0215's and S-0314's, and this spec cannot build the state without
  their claims. The rows read the same answer as rows x4 and q3 (a full transport), which have
  steps. The T10 line *refusing building* only records what the answer is for a second visitor.
- Row AC8, table ACC's *nothing* and row BC3 (an *attack* over a cell, or on an object, by an
  actor with no weapon in its first slot): only a type that switches between weapons reaches
  them. T10's line *empty first slot* tries a test type of that kind; if the original does not
  let it fire at all, or if a test file cannot give an aircraft type that behaviour, the record
  says so and the rows stay `unmeasured` for that reason (BC3 has a quirk switch, Edge cases).
- Rows IC2 and IC3, and the fog parts of rows GC9, VC2 and BC2: fog of war (claim 9).
- Value *x*, row x2, its team-member part: the flag of a team's type that the rule reads is not
  identified, so no test file can set it. The part about a moving mobile unit has a step (T5,
  the allied transport standing and moving).
- The three parts of claim 6 listed there as `unmeasured`: the sonic exception, the blinking
  disguise and the disguise as scenery (reasons in claim 6).

**Scenario T19: mouse buttons and drag classification (claims 26 and 27).** Use an
isolated, large flat stock-data skirmish map, 800×600, speed 6, no engine add-on and no mods (the named test files of a scenario excepted),
with own tanks and infantry, an enemy tank, an own transport and one bystander computer
player kept outside the observed area. Use scripted press, motion and release events,
video and a read-only recorder of selection, issued orders and view origin. Record the
host's two drag metrics. Repeat each case three times, allowing a normal input sample
between each motion and release.
- Left-press an own tank, hold, then release: selection appears on release, not press.
  Left-click open ground, the enemy tank and a transport with suitable infantry selected:
  expect move, attack and enter respectively under the existing tables. Right-click the
  same targets: selection clears and no such order is issued. Existing orders continue
  after deselection. Observe before release to distinguish right-press from right-release.
- Left-drag using T14's (4, 0), (3, 3), (5, 0) and (4, 4) samples: expect its F6
  click/box distinction, with no right-drag scrolling substituted.
- With a unit selected and the view far from limits, right-press at an interior pixel.
  Let *D*ₓ and *D*ᵧ be twice the recorded horizontal and vertical drag metrics (the F8
  thresholds). Move by (*D*ₓ, 0), (0, *D*ᵧ), and (*D*ₓ, *D*ᵧ) in separate runs, then release:
  expect no scroll drag and ordinary deselection. Increase just one axis by one pixel:
  expect scroll drag and selection retained at release. In another run cross, return to
  the anchor, then release: selection still retained. Repeat at a view limit: a classified
  drag retains selection even where further outward scrolling cannot move the view.
- Start fresh with fresh scrolling preferences, then repeat left selection/orders,
  right deselection and right dragging with its values 0, 1 and 2 in separate launches.
  Expected: only scrolling mechanics may differ; the button roles and F8 classification
  do not. Inspect the in-game control settings for an alternative button scheme. This
  does not require a guessed key for one.
Any order created by a right click, selection change at press rather than release,
left/right exchange through a game option, radial rather than axis right-drag threshold,
or deselection after a sampled F8 crossing falsifies the corresponding claim. A script
that releases before motion is sampled is inconclusive. Record thresholds rather than
assuming the host supplies a particular pixel count.

**Scenario T20: right-click cancellation (claim 28).** Use T19's map/settings, sufficient
credits to complete a stock building, and screen capture plus a read-only mode,
production-item, credit, selection and order recorder. Repeat each case three times.
Arm placement from its ready sidebar icon and right-click empty battlefield ground:
expect the outline and placement mode to end, the item still done, no refund or place
order, and the ability to take the same ready item up again. Contrast right-clicking
its sidebar icon: S-0411 claim 5's cancellation/refund, not mere mode exit. Repeat the
battlefield click over a legal placement site and over an invalid one: both merely end
the mode. Arm sell, repair, power-switch and beacon placement separately; right-click
both a valid target and empty ground. Expect the mode to end without its target order
or beacon. Test superweapon targeting and route-editing where available with the same
cancel-not-action expectation. Enter the ordinary sidebar modes successively: the new
mode replaces the preceding one, rather than stacking another cancellation layer.
In each reachable mode repeat a right drag with a sampled F8 crossing before release:
expect scrolling without cancelling the mode.
Any item refund from battlefield placement cancellation, target order/beacon from a
right click, retained mode after a non-drag click, extra ordinary deselection on mode
exit, or mode cancellation after a classified drag falsifies the corresponding rule.
Unavailable targeting modes are inconclusive; do not force otherwise unreachable
mode combinations by changing the game's state.

**Scenario T21: box addition and type-expansion order (claim 29, Q-0452).** Use an
isolated revealed flat skirmish map, speed 6, 800×600, stock data, no engine add-on and no mods (the named test files of a scenario excepted),
with one bystander computer player kept outside the test area under the glossary's void
rule. Use scripted input, tick-labelled video with sound, and a read-only recorder of
candidate identities, successful additions and the resulting selection order. Run each
case three times, repeating a run if the candidate order changes during release.

For an ordinary box, place own tanks, unarmed engineers and a vehicle-like building
at distinct drawing sort keys, plus an ineligible ordinary building and an enemy unit.
Keep well below candidate capacity. Record candidate order rather than assuming map
placement order equals it. Box the group in both drag directions, first replacing an
outside selection and then adding with force-select. Expected: successful additions
are the eligible in-box subsequence in recorded candidate order. The building subgroup
precedes mobiles. Positive-primary-damage additions reverse that order at the front,
while others retain it at the end. Preselect one in-box unit before the additive run: it must
not be repositioned or give a new selection response. Record the responding object's
identity, not merely the eventual first selected object. Expect the earliest successful
addition to own that response under S-0701 claim 17. Repeat with ground and airborne
own mobile objects so layer order differs from creation order.

For type expansion, label two selectable own types A and B, with candidate order A0, B0, A1, B1.
Arrange a box containing only A1 and B1, with A0 and B0 elsewhere in the same view. With no tap
made earlier in the game, so that the reach is *screen*, hold the type-select role for more than
1 s and release the box. Expected addition order: A0, A1, B0, B1, before claim 12's front/end
transformation, with no selection response. Enclosing another A candidate must not reposition
the already added A objects. Reverse which type has the first in-box encounter while keeping
each type's recorded candidate order: the type groups reverse, not their within-type order.
Repeat additively with A0 preselected. Then tap the role twice with no selection change between
the taps, so that the reach is *map* (claim 32), clear the selection, which leaves the reach as
it is, and repeat with same-type objects off screen whose creation order differs from drawing
order. Expected: each encountered type expands map-wide in surviving creation order. Remove an
early-created witness and create a later one in a separate run to test that surviving creation
order is retained. Type-select runs remain silent.

A different successful-addition sequence, change with drag direction, repositioning of
an already-selected object, or speech from a later ordinary-box addition falsifies the
corresponding rule. A type drag that merely retains interleaved candidate order across
types, uses creation order for the on-screen expansion, uses drawing order map-wide,
or gives a selection response falsifies its corresponding type-expansion rule.

**Scenario T22: move-target marker clicks and bridge height (claim 30).**
- *Setup:* a tiny purpose-made skirmish map with open flat ground, a slope, elevated flat
  ground and an intact high bridge with accessible deck cells across its width. Use stock
  types and art, one human player and one enemy bystander computer player, game-speed index 2,
  800 × 600, no engine add-on and no mods (the named test files of a scenario excepted). The observed area contains the bridge and these ground
  patches, own infantry, a tank, an aircraft, a rally-capable factory and a harmless target
  object. Keep combat away from the observed area. Capture tick-labelled video and use a
  read-only recorder for the resolved input cell, marker world position, actor order and
  route points. Use scripted mouse/keyboard input where useful; no harness changes are assumed.
- *Procedure:* select the tank alone and click two visibly separated pixels that resolve to
  the same empty cell, allowing each marker to finish before the next click. Repeat on the
  elevated flat patch, slope and every transverse row of the intact deck, clicking the deck
  rather than a visible point beneath it. Repeat ordinary cell moves with infantry and
  aircraft, then a mixed mobile group. Test force-move and Ctrl+Shift attack-move onto empty
  cells, and a route-mode cell click. Contrast a blocked-cell *no-move*, ordinary attack,
  force-fire, an object-directed move, an object-directed attack-move, an enter click and
  a factory rally click. Repeat T2's first-object-refusal case. Finally observe the named
  bystander's unit moving in a separate controlled run, without issuing a local move click.
- *Observable:* marker count and first visible tick per click, fixed world position, resolved
  input cell versus final destination, and the presence or absence of immediate orders and
  route points. Repeat each case three times. Discard a click that picks a different cell or
  object than intended, and apply the bystander void rule to all other runs.
- *Expected:* one marker for each eligible first-object cell move, including attack-move to a
  place and route feedback, regardless of group size; none in the contrasted cases. Two
  pixels in the same cell yield identical marker positions. Position equals the cell centre
  at its terrain surface height, with the deck lift on marked deck rows in the original.
  Record separately whether an unmarked row leaves the original's marker at terrain height:
  our engine must instead use deck height for every intact deck cell. S-0109 claim 26's
  imported vertical draw shift can move image pixels without changing the recorded world
  position, so pixel placement alone does not establish a wrong world height. Route feedback
  accompanies added points, not an immediate order. Bystander moves give no marker.
- *Falsified if:* an eligible click gives no marker, a contrasted click gives one, a group
  creates multiple markers, two clicks within the same cell produce different world positions,
  or the original's marked-deck marker lacks its deck lift. A deck-height original marker on
  an unmarked row falsifies the stated original defect, not our corrected deck-height rule.
  On our engine any intact-deck marker at terrain height fails the correction. Lack of a
  known way to obtain a particular object-directed move is inconclusive for that sub-test,
  not evidence that all object clicks create markers.

**Scenario T23: superweapon aim images (claim 31, Q-0742).**
- *Setup:* tiny purpose-made skirmish maps with stock rules and art, no engine add-on and no mods (the named test files of a scenario excepted),
  800×600, speed indices 2 and 5, one distant bystander computer player under the usual
  void rule. Across separate runs obtain each stock superweapon targeting mode in table CS
  through its ordinary grant/charge route. Provide working radar, own and enemy buildings,
  an allied building where applicable, empty explored ground and a shroud boundary. Keep
  the bystander's unnamed objects outside the observed area. Clear the selection for the
  battlefield aim tests. Do not treat a radar outage as a cursor result.
- *Procedure:* arm each ready mode from the sidebar, hover those target categories without
  clicking, and record the pointer-inclusive screen output and input pixel with a wall-clock
  timestamp for at least two seconds per target. Compare displayed pictures with the user's
  imported cursor frames. Pick a teleport source, then hover with its destination armed and
  compare with the source-pick picture. For force shield compare own/allied building offers
  with enemy buildings and empty ground. Repeat over the active radar image, first with
  empty selection, then with a selected movable unit, and include the panel margin control.
  Use scripted input and screen capture; no simulation recorder changes are required.
- *Observable:* displayed frame identities, visible advances and wrap points, elapsed real
  time, hot-spot position relative to the input pixel, selection and armed targeting mode.
- *Expected:* exactly table CS. In particular iron curtain stays on 346, both teleport
  stages stay on 357, and paradrop stays on 259; unused later pictures in their image file
  do not imply pointer animation. Refused force shield cycles 460–469 rather than using
  an arrow. Animated rows advance once per 4 clock units with no catch-up and wrap inside
  their ranges. Both speeds give the same real-time cadence. All superweapon pictures are
  centred on the input pixel using the full canvas. Radar miniatures are static; unsupported
  modes use S-0410 claim 12's arrow or ordinary representative fallback. The panel margin
  shows the full arrow, not an aim image.
- *Falsified if:* a reachable offered action shows a different frame, a static row advances,
  the two teleport stages differ in cursor picture, the refusal uses a different range,
  timing follows game speed, the hot spot uses cropped-frame bounds, or radar substitutes
  an unsupported battlefield image. An unavailable mode or hidden radar is inconclusive
  for that sub-test, not evidence for an arrow assignment. Run each reachable mode three
  times; record any still-unreached combination explicitly for follow-up.

**Scenario T24: infantry deploy readiness and the rearm countdown (Q-1754: Terms, *deploy
readiness*, and Edge cases).**
- *Setup:* the usual isolated test map, speed 6. A named test rules and art file defines infantry
  type *D* that deploys in place, with an undeploy delay of 60 ticks, *deploy* and *undeploy*
  sequences of 10 ticks each and every other sequence kind of S-0107 given at least one stage, a primary weapon *W*₁ and a deploy weapon *W*₂ (second slot), each
  with a rate-of-fire field of 200 ticks, *W*₁'s range clearly shorter than *W*₂'s; *D* has the
  deploy-fire flag, so that a deployed *D* fires *W*₂ (S-0308 claim 8); a stationary
  test target with 1 hit point that a single shot of either weapon destroys, so that each
  case has exactly one shot. In (c) to (e) the target is an enemy standing within *W*₁'s reach.
  In (b) it stands within *W*₁'s reach, so that row I21 does not refuse the attack click on it,
  and belongs to a house at peace with the player's (for example the map's neutral house), so
  that no *guard* processing of *D* picks it before the order. In (g) it is an enemy standing
  beyond *W*₁'s reach but within *W*₂'s, so only the deployed unit can fire at it.
  Variants *D*₀ with an undeploy delay of 0 and *D*₋ with a negative undeploy
  delay, otherwise equal. A second copy of the file sets both rate-of-fire fields to 1 tick.
  Both copies set the rate-of-fire factor of every difficulty row and of the player's country
  to 1, so that the player house's rate-of-fire factor *h* is 1 (S-0301 claim 3) and the delay
  after a long-rate shot is 200 ticks plus the draw, well inside the 260-tick windows of (c)
  and (e).
- *Procedure:* one unit, selected alone; press the deploy key on every second tick, releasing it
  on the ticks between (S-0213 claim 1: one input event per cycle, and a press and a release are
  one event each), for the stated window. Tick 0 is the window's first tick, at or after tick 4
  of the game, when orders can first be issued (S-0213 claim 1). In (a), (b) and (f) the press at
  tick 0 is followed by no press until the recorder shows the unit's *deploy* sequence, which
  starts in tick *s*. The presses then resume on tick *s* + 1 and go on every second tick, so that
  no press falls after tick 0 and before the unit is in *deploy*. (a) Long rate of fire, enemy target absent, *D*: 160 ticks. (b) Long rate of
  fire, target present, *D*: 400 ticks. Once *D* is in a deployed sequence, give it one attack
  order on the target with force-fire held (the attack rule's condition (a), since the target is
  not an enemy). The click takes input events of its own (the force-fire key going down, the
  button going down and up, the key going up), one per cycle, so the presses stop for those
  events and resume on the second tick after the last of them; the expectations below are read
  against the presses actually made. The order is taken at once: a deployed sequence can be
  interrupted, so the unit is ready to change behaviour (S-0215 claim 11) and *attack* replaces
  *guard* on the next tick, with its first processing due at once rather than at the end of the
  undeploy delay. It makes the unit fire *W*₂ while deployed
  (S-0215 claim 23: a deployed infantry unit of a human-controlled house in *attack* runs its
  deployed-firing rule, S-0517 claim 16). The target is destroyed by that shot, *attack* falls back
  to rest with no target, and the unit then undeploys by itself (S-0215 claim 31). A run of (b) in
  which the order log records no attack order for *D* while it is in a deployed sequence is void. (c) Long rate
  of fire, target present, *D* undeployed: order one primary shot at the target, then start the
  presses on the tick after the shot, for 260 ticks. (d) As (c) with the 1-tick rate of fire.
  (e) As (c), but on the tick after the shot also give a move order to an adjacent cell, then a
  stop order 20 ticks later, then
  presses for 260 ticks. The
  move and the stop each take their input events, and the presses resume on the
  second tick after the last of them. A run of (e) in which *D* fires
  a second shot is void. (f) case (a) with *D*₀: no target and no shot, since a unit with an undeploy delay of 0 may
  undeploy as soon as its deploy ends (S-0215 claims 30 and 31), leaving no time to order a deployed
  shot. (g) case (b) with *D*₋, without the attack order, and with (g)'s own target of the Setup
  (an enemy beyond *W*₁'s reach), not (b)'s. The presses reach it in every sequence, so
  it can switch back before any processing of *guard* while deployed (S-0215 claim 30), and it
  need not make a shot. Part g therefore backs only the sequence part of the negative-delay rule,
  for the sequences of its deploy cycle in which a press falls; other sequences, and that the countdown does not affect such a type, stay
  `unmeasured` (Edge cases).
- *Runs:* 3 per case. Among the random draws, the shot delay's draw of S-0301 claim 2 and the
  standard delay's draw of S-0215 claim 6 (0 to 2 ticks each) shift the observed ticks: read them
  with the random-state recorder, or allow those margins.
  The test target is unarmed, and no other shooter takes part.
- *Observable:* every tick: whether a press queued a deploy order for the unit (the recorder's
  order log), the unit's current sequence, and the tick of each shot it makes with the sequence it
  is in at that tick.
- *Expected:* (a) an order at tick 0; then no order from the tick *s* on which *deploy* starts
  until *undeploy* has ended (no press falls between tick 0 and *s*, Procedure). The unit undeploys by itself with the timing that S-0215 claims 30
  and 31 give: the undeploy delay counts from the processing that started the deploy, so *undeploy*
  starts at about *s* + 60 and ends about 10 ticks later. The second order comes at the first press
  after that, at about *s* + 70 to *s* + 73 (the presses fall on every second tick), well inside the
  window, and the cycle repeats. (b) as (a), except that the deployed shot comes within a few ticks of
  the order's last input event, *undeploy* starts at a processing of *guard* after the target
  is destroyed (S-0215 claim 31's first row; this case does not time it), and after *undeploy* ends no order is given until the delay
  *R* of the deployed shot has passed
  (S-0301 claim 2: with this test data 200 ticks, since the setup makes the house's rate-of-fire
  factor 1, plus the draw), then an order from the first press after it. (c) no order until the primary shot's
  delay *R* has passed, then an order. (d) as (c), with *R* a few ticks: an order on the first
  press after that *R* has passed. (e) as (c): the move and the stop change nothing. (f) with *D*₀:
  as (a), with the undeploy timing that S-0215 claims 30 and 31 give for an undeploy delay of 0.
  (g) with *D*₋: an order from every press, in every sequence it passes through (the unit does not undeploy by itself
  and each applied order switches it at the end of its current sequence, S-0213, table T2's deploy
  rule).
- *Falsified if:* a press gives an order while the sequence is *deploy*, a deployed sequence or
  *undeploy* in (a) to (f); an undeployed unit is refused once its countdown has ended, or given an
  order while it runs; the refusal after a shot lasts the undeploy delay or a sequence length
  rather than the shot's delay; in (b), the deployed unit does not fire before its undeploy delay
  has run out (the order was not taken while it was deployed), an order is given after *undeploy*
  ends but before the deployed shot's delay *R* has passed, or the first press after *R* has
  passed gives none; any event of (e) ends the refusal early; or *D*₋ is refused in
  any sequence of (g).

**Scenario T25: type-select tap, reach and held clicks (claim 32, Q-1987).**
- *Setup:* a skirmish on a small purpose-made map, stock data, the human with an allied computer
  player and an enemy one. The human has tanks of one type A: A1 to A3 on the screen and A4 and
  A5 off it, created in the order A4, A1, A5, A2, A3. The ally has a tank of type A on the screen,
  and the enemy has one of type A and one of a type B that the human does not have. The human
  also has a power plant and two infantry units of type C on the screen. The recorder logs the
  selection in order after every input, the message-list texts and every selection response
  heard, with the object that gives it. Every hold of the role that is not meant as a tap lasts
  more than 1 s, every tap less than 200 ms.
- *(a) One tap.* Click A1 alone, then tap. Expected: A1, A2 and A3 selected, A4, A5 and the
  ally's and enemy's tanks not, the across-screen text shown, and a selection response queued by
  each of A2 and A3. Falsified if off-screen or other houses' tanks join, or nothing changes.
- *(b) Second tap.* Tap again at once, and in another run 5 s after (a). Expected in both: A4 and
  A5 added in creation order (A4, then A5), each by claim 12's front/end rule, and the across-map
  text. A screen-only result after 5 s falsifies the absence of a timing between taps. Then click
  A1 alone and tap: the screen again (the click reset the reach).
- *(c) Hold limit.* Click A1 alone, press the role, release it after 1 s. Expected: no change and
  no text. Repeat with a 300 ms hold: as (a). A change after the long hold falsifies the limit.
  Then, by scripted input (measurement setup), make 10 holds of 400 ms and 10 of 600 ms, each
  after clicking A1 alone. Expected: every 400 ms hold acts as (a), and no 600 ms hold changes
  anything. Falsified if a 400 ms hold does nothing or a 600 ms hold acts. The game reads the
  key times when it handles the key events, so the scripted lengths carry up to one frame of
  error; the 100 ms margins on each side of 500 cover it.
- *(d) Box then tap.* Box A1 to A3, then tap. Expected: straight across the map (A4 and A5 added,
  across-map text).
- *(e) Other types.* Select the power plant alone and tap. Expected: no change, across-map text.
  Select one C alone and tap: both own C units. Click the enemy's B tank alone and tap.
  Expected: an empty selection and the nothing-selected text.
- *(f) Held select.* Run (a), so the reach is *screen*, then clear the selection. Click the
  enemy's A tank with the role held: with an empty selection the click is *select* (claim 1).
  Expected: exactly A1 to A3, no response heard. Clear the selection again and repeat on the
  enemy's B tank: an empty selection. Then run (a) and (b) up to its second tap, so the reach is
  *map*, clear the selection and repeat on the enemy's A tank: A1 to A5. Clearing the selection
  leaves the reach as the last tap left it (claim 32, *held clicks*).
- *(g) Held toggle.* With A1 to A3 and one C selected and the reach at *screen*, toggle-select A2
  with the role held. Expected: A1 to A3 removed and C kept. Then select A1 to A3 again, scroll
  to A4 and A5 and toggle-select them without the role, scroll back and repeat: A4 and A5 stay.
  After (b), with A1 to A5 selected and the reach at *map*, the same click removes all five.
- *(h) Held click on the sole selection.* Select the power plant alone and click it again with the
  role held. Expected: no change.
- *(i) Order across types.* Select one A and one C, with the screen candidates of A and C
  interleaved in candidate order (claim 25, logged by the recorder), and tap. Expected: the added
  A and C units in that interleaved order before claim 12's front/end rule. All of one type added
  before the other falsifies it.
- *(j) Defeat.* In a run where the human loses, tap after the defeat. Expected: no change and no
  text.
- *(k) Unit-powering building.* The named test rules file `S0214-deploy.ini` of part (l) also
  gives a test building type P, larger than 1 × 1 and undeploying into nothing, a stock vehicle
  type to power, and a test building type P′ of the same size none. Give the human two
  buildings of type P, both on the screen. Select one alone and tap. Expected: both selected, the
  across-screen text. Control: give the human two buildings of type P′ on the screen, select one
  alone and tap. Expected: it stays alone, with the across-map text. Falsified if the second P
  is not added, or the second P′ is.
- *(l) Deploys-into types.* With a named test rules file `S0214-deploy.ini` that gives a stock
  vehicle type V a 1 × 1 building type K to deploy into, and K that vehicle type to undeploy
  into, deploy one V into K on the screen, then select another V alone and tap. Expected: K is
  added with the other Vs on the screen. Control: the same file gives a test vehicle type V′ a
  test building type K′ to deploy into, larger than 1 × 1, undeploying into nothing and powering
  no unit. Deploy one V′ into K′ on the screen, select another V′ alone and tap. Expected: K′ is
  not added. Falsified if K is not added, or if K′ is.
- *(m) Deactivated and bunkered.* In a separate run, give the human three vehicles R1 to R3,
  created in that order, of a stock type that depends on a control centre (S-0405 claim 13), R1
  and R2 on the screen and R3 off it, and one control centre. Click R1 alone and tap twice.
  Expected: R2, then R3, added. Sell the control centre, so that R1 to R3 are deactivated, then
  click R1 alone and tap twice. Expected: R1 stays alone, with the across-map text after each
  tap. Then clear the selection and click R1 with the role held (the reach is *map*). Expected:
  R1 to R3 selected. In another run, add to the setup a tank bunker of the human on the screen
  and a tank A6 of type A, created last, that has entered it before the part starts. Click A1
  alone and tap twice. Expected: A2 to A5 added, A6 not. Clear the selection and click A1 with
  the role held. Expected: A1 to A6 selected. Falsified if a tap adds a deactivated R or A6, or
  if a held click leaves one of them out.
- *(n) Tether.* In a separate run, give the human a refinery and two harvesters H1 and H2 of a
  stock type that unloads standing on its refinery's cells, both on the screen. While H2
  unloads there, click H1 alone and tap twice. Expected: H1 stays alone. After H2 has left the
  refinery's cells, click H1 alone and tap. Expected: H2 added. The record notes where H2 stood
  at each tap. A run in which H2 left the refinery's cells before the first two taps is void.
  Falsified if H2 is added while it stands on the refinery's cells, or not added after.
- *(o) to (r) What resets the reach.* Each part starts from the same state S: run (a) and (b), so
  that A1 to A5 are selected and the reach is *map*, after making group 1 of A1 to A5 and group 2
  of one C before (a). Then a new tank A7 of type A comes into play on the screen, built by the
  human, and is not selected (the recorder confirms that the selection
  does not change). A tap that keeps the reach adds A7 with the across-map text; a tap whose
  reach was set to *screen* adds A7 with the across-screen text. The text tells them apart.
- *(o) Make group.* From S, press Ctrl+3, then tap. Expected: the across-map text. Falsified by
  the across-screen text.
- *(p) Select and add group.* From S, press 1, then tap. Expected: the across-screen text.
  From S again, press Shift+2 (C joins), then tap: the across-screen text. Falsified by the
  across-map text in either.
- *(q) Centre group.* From S, with A1 still the first selected object, press Alt+1, which neither
  clears nor adds (the recorder confirms the selection), then tap. Expected: the across-map
  text. Falsified by the across-screen text.
- *(r) Type-select drag.* Give the human a third unit C3 of type C off the screen for this part.
  From S, with force-select held, so that the drag adds and does not clear (claim 19), drag a box
  with the role held over one C on the screen, leaving A7 outside the box. Expected: A1 to A5
  kept, every C in play added, C3 included, because the drag reads the reach *map* (claim 29),
  and A7 not added. Then tap: A7 added with the across-screen text, because the drag added
  objects, so the tap set the reach to *screen*. Falsified if C3 is not added, if the drag adds
  A7 or drops an A, or by the across-map text after the tap.
- *(s) Captured objects.* In a separate run, the human has two mind-controlling units and the
  enemy two vehicles of one type that the human has none of. Each controller takes one of them.
  Select one taken vehicle alone and tap. Expected: the other added, with the across-screen
  text. Falsified if it is not added.
- *(t) Held drag candidates.* Each case starts from an empty selection with the reach at
  *screen* (run (a), then clear the selection) and drags a box with the role held. The recorder
  logs which objects lie inside each box. Control: the same box without the role.
  - Over the enemy's A tank alone. Expected: A1 to A3 selected, and not the enemy's tank; the
    control selects nothing. Over the enemy's B tank alone: nothing selected.
  - For this case the human has a second power plant on the screen, outside the box. Over the
    first alone.
    Expected: both plants selected; the control selects neither.
  - With `S0214-deploy.ini` (parts k and l): over one of the two P buildings alone. Expected:
    both P selected. Then over one V alone, with a K deployed from another V on the screen
    outside the box and a third V on the screen. Expected: the Vs on the screen selected, and
    not K (a tap would add it, part l).
  - In part (m)'s first run, after the control centre is sold: click A1 alone and tap once, so
    that the reach is *screen* as in (a), clear the selection, then drag over R1 alone.
    Expected: R1 and R2 selected, and not R3, which is off the screen; the control selects
    nothing.
  Falsified if the enemy's tank, K or R3 joins, if the second plant, the second P or R2 is
  missing, or if a control box selects a plant, a deactivated R or anything over an enemy tank.
- Our engine gives only the first of (a)'s responses (claim 32, the DECIDED departure). The
  record logs the original's responses as heard.

**Scenario T26: control groups (claim 33, Q-1988).**
- *Setup:* a skirmish on a small purpose-made map, stock data, fresh keyboard bindings, the
  human and one enemy computer player. The human has tanks A1 to A3 of one stock type, created in
  that order, an infantry unit C with a weapon, an engineer, a power plant, an aircraft of a
  stock type standing on its landing building, a stock transport that carries vehicles, a
  construction vehicle, a tank bunker and a mind-controlling infantry unit. The enemy has a
  mind-controlling
  infantry unit kept away from the human's units until a part needs it, and a tank E whose map
  object line gives it group 3. The recorder logs, after every input, the selection in order,
  each object's group number, the view position in leptons, the sell, repair, power and
  placement states, the message list and every selection response heard.
- *(a) Make and select.* Select A1 and A2, press Ctrl+1, clear the selection, press 1. Expected:
  A1 and A2 selected in creation order under claim 12's rule, one response, from the first
  added. Falsified if the group is not made or the selection is not A1 and A2.
- *(b) Reassign.* Select A2 alone and press Ctrl+2. Press 1, then 2. Expected: 1 gives A1 alone,
  2 gives A2 alone. Falsified if A2 is in both groups. Then select the power plant and press
  Ctrl+8, and the aircraft and press Ctrl+9. Clear the selection and press 8, then 9. Expected:
  the power plant, then the aircraft. Falsified if either kind cannot join.
- *(c) Empty group.* Select A3 and press Ctrl+4, then clear the selection and press Ctrl+4 again.
  Select C and press 4. Expected: an empty selection. Falsified if C stays selected or A3 is
  selected.
- *(d) Add.* Select C alone and press Shift+1. Expected: C and A1 selected, A1 placed by claim
  12's rule. Falsified if C is removed.
- *(e) Repeat.* Make group 1 of A1 and A2 again and scroll the view away from them. By scripted
  input, press 1, then 1 again 600 ms later, 10 times; then the same with a 1000 ms gap, 10
  times. A press that selects the group gives a selection response and leaves the view; a repeat
  gives none and moves the view to the group. Expected: after a 600 ms gap a repeat, after a
  1000 ms gap a new selection. Then press 1 at 0, 600 and 1200 ms: the second press is a repeat,
  the third selects again (1200 ms after the first) and gives a response. In another run press
  1, toggle-select C into the selection and press 1 within 600 ms: the group is selected again,
  with a response. In a third run press 1, then Alt+1 300 ms later and 1 at 900 ms: the third
  press selects again, since the *centre group* press did not restart the 800 ms. Falsified if
  a 600 ms press selects again, a 1000 ms one is a repeat, the third press at 1200 ms is one, or
  the press at 900 ms after Alt+1 is one.
- *(f) Centre and its point.* Make group 5 of A1, A2 and A3 with A3 far from the other two.
  Clear the selection, scroll away and press Alt+5. Expected: the three selected, and the view
  centred on the mean of A1 and A2, by F9. Then select A1 alone, toggle-select the engineer (the
  recorder confirms that A1 stays first) and press Alt+5. Expected: A1 to A3
  and the engineer selected (quirk). Select C alone, toggle-select the engineer (C stays first)
  and press Alt+5: A1 to A3 only. Falsified if the view centres on the mean of all three, or if
  the engineer is dropped in the first case or kept in the second. With the switch off, the
  first case gives A1 to A3 only.
- *(g) Carried and deployed.* Make group 1 of A1 and A2 again. Put A1 into the transport and
  press 1: A2 only. Unload, and press 1: A1 and A2. Put the construction vehicle in group 6,
  deploy it and press 6: the
  construction yard. With the lobby option that lets a yard be packed up (Import mapping),
  pack it up into the vehicle again and press 6: the vehicle. Put A3 in group 7, move it into
  the tank bunker, then out, and press 7: an empty selection. Falsified if A1 is lost from its
  group, the yard or the vehicle is not selected, or A3 is still in group 7.
- *(h) Destroyed.* Let the enemy destroy A2 and press 1: A1 alone, no error.
- *(i) Owner change.* Let the enemy's controller take A1 and press 1: A1 not selected. Make
  group 1 of C while A1 is taken. Kill the controller so that A1 returns, and press 1. Expected:
  A1 and C. Then let the human's controller take E, and press 3 without having made group 3.
  Expected: E selected. The second case is void unless the recorder shows E still in group 3
  when it is taken, since the enemy's teams can give it another number. In another run, let A1
  be taken and come back without group 1 being made meanwhile, and press 1: A1. Fixed behaviour
  (the switch off): C alone in the first case, nothing in the second, A1 in the third.
- *(j) Modes.* Enter sell mode and press 1: sell mode ends and group 1 is selected. Repeat with
  repair mode and power mode. Enter sell mode and press Ctrl+0: sell mode stays. Start a ready
  building's placement and press 1: the placement stays. With the options menu open, press 1
  and close it: the selection is unchanged. Falsified if a mode survives a recall, *make group*
  ends one, or the placement ends. Superweapon targeting is S-0512 scenario X18 part e.
- *(k) First member not selectable.* In a separate run with a named test file, a test copy U of
  A1's tank type has its selection flag off (Import mapping). Map object lines put U and a tank A4
  of A1's type in group 2, U's line before A4's, so that U is created first (the recorder confirms
  the order). With an empty selection press 2. Expected in the original: A4 selected, U not, and
  no selection response heard, the display bug of claim 33. Fixed behaviour: A4's response.
  Falsified if U is selected, A4 is not, or the original gives A4's response.
- *Runs:* 3 of each part; part e as stated. The record logs the responses as heard.

## Open questions

- The state-specific transient box points of claim 19 stay `unmeasured` beyond T14 part B.
- Route-point editing cursor substitutions are not covered by table C, and building-placement
  hover images belong to the placement display's spec.

- Tables AC and B, and the building cell table BC, are low confidence throughout. Their steps are
  in Scenarios T6 and T10 (the list after Scenario T10 gives the rows that are `unmeasured`, each
  with its reason).
- Rows IC2 and IC3 (an engineer clicking a cell under fog of war) depend on what fog shows, and
  are not yet confirmed. When the original considers fog for a cell (rows GC9, IC2, IC3, VC2) is
  not identified.
- The flag of a team's type that lets a moving team member be entered (value *x* of table I), and
  the route-following state of row GC3, are not identified.
- Where a *sell unit* action would come from, if anywhere (claim 13).
- How the harvester-truce setting of row I15 is set in a skirmish game.
- Whether the command bar with its attack-move button is available in a default skirmish.
- Which disguises are overlay types (scenery check), and how often a disguise belongs to no house
  in stock data.
- Two low-confidence details are stated only by their effect and are not identified: the
  neighbour step that anchors a foundation wider or deeper than 2 cells (row VD2); and the relaxed
  entry test of row GC10.
- Row BC3: a building with no weapon in its first slot gets *attack* on a cell. Since row GC4
  needs a weapon in the slot the building currently uses, this can only concern a building type
  that switches between weapons. It is surprising and low. Scenario T10's line *empty first slot*
  tries it, and if the original does not let such a type fire at all, the record says so and the
  row stays `unmeasured` (a quirk switch covers it, Edge cases).
- Whether a tank can be placed inside a tank bunker by the test map, which Scenario T4 part C
  (part (d) of the attack rule) needs.
- Whether the test map can put a vehicle on a cell of a vehicle factory (row AC2), a live
  veinhole monster on a cell (row GC4 line of T10), and a garrisoned building that belongs to
  the human player (Scenario T6).
- What a control-group key does while the route-mode key is held (claim 33, S-0213 claim 16).
- The effects of three of the original's four further selection commands on the selection
  (claim 32). Our engine has none of them.
- Whether a selected object that changes owner (for example one that an enemy captures) stays
  selected, which decides whether the fallback of *toggle select* (claim 12) can be reached.
- Whether a click finds a building whose type is invisible in the game as the clicked object (row B1, target half).
- Whether a ground object can stand on a destroyable-cliff cell or on a cell with a live veinhole
  monster (row AC5).
- Whether the recorder can log a vehicle as inside a tunnel for long enough to click (Scenario T9
  part D).
- Whether a unit below the surface can be selected (Terms, *can be selected*; the T10 line for
  row b2 records it). Cloak and scenery-disguise exclusions are settled by S-0516 claims 26 and 36.
- Transport readiness is S-0507 claim 7's, including its exact tunnel-marked/edge predicate.
  Ordinary-input witnesses for tube-interior and non-bunker building-cell readiness remain
  open there, not the tunnel-edge geometry or the bridge and water rules here.
- Whether an object of the allied bystander can be clicked while it stands in shroud, which Scenario T7
  step (c) needs.
- Which movement zone a building type has when its section sets none (row B10), and whether S-0211's
  default of *normal* applies to buildings.
- Whether the map can give the special house a capturable building (Scenario T11); until it does,
  the side from which *allied* is judged is `unmeasured` for the one one-way alliance of a skirmish
  (medium).
- Whether a skirmish can turn fog of war on, which rows IC2, IC3 and the fog parts of rows GC9, VC2
  and BC2 depend on.
- Whether a skirmish with stock data can produce an active ion storm, which the zone rule of
  claim 11 depends on.
- The step of an underground unit's height from tick to tick, which decides how closely
  Scenario T4 part B fixes the limit of claim 6 (b).
- What creates the melee link (Terms, *melee-linked*) beyond the link flag of a warhead, and
  which stock objects can form it in a skirmish.
- What a gap generator's boosted radius is, and when it starts and ends (row B4); and how the extra
  power drain of row B4 is derived from the type's extra-power field ("the drain part of a negative
  value", Import mapping).
- A *repeat* over a selection whose first object is another house's and whose own objects are
  exactly group *N*'s members (claim 33) is not stated. It is probably unreachable: a *select
  group* press that is not a repeat clears the selection, and adding an own object replaces
  another house's by the owner rule of *toggle select* (claim 12).

- **Coordinated hand-off checks (verifications for the hand-off, not decisions).** Confirm S-0212 claim 25 and P16 runs (u) and (r)
  preserve claim 11's direct-destination bypass, unrestricted free-mover no-move search
  and distinction from rally-point searches. The current companion passage already agrees.
  Confirm S-0213 claims 22 and 23 cite claims 5 and 23 here for Q-0286 question 2, and
  S-0408 claim 12a keeps claim 13's cursor-cell and object-versus-wall-sale distinction.

## Changes for other specs

- **S-0204, claim 5, uses of the table square root.** The claim lists the uses that
  `quirks.table_root_distance` covers. S-0214's representative distance (F1) and group centre
  point (F9) are object distances taken with the same table and should be on that list.
- **S-0701, claim 17, type-select silence.** The silence should cover only the held type-select
  role (clicks and drags, S-0214 claims 29 and 32). A tap of the role gives one selection
  response, that of the first object it adds, as a band box does (S-0214 claim 32). Scenario
  D3's expectation of no response for type-select should then cover the held role only, and
  expect one response, from the first object added, for a tap.
