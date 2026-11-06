---
id: S-0203
title: Tick order
topic: simulation
revision: 50
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-06
measurements: [M-0021]
requests: [Q-0018, Q-0412, Q-0430, Q-2636]
supersedes: []
---

# S-0203: Tick order

## Summary

What happens within one tick of a single-player game, and in which observable order: when the
player's orders take effect, in which order objects act, when a shot's damage lands, when a
destroyed object stops acting, how long a removed object is still kept, and the consequences, such
as two units killing each other in the same tick. How many ticks run per second is S-0201, and what
each object does in its own turn (moving, firing, harvesting) belongs to the topic specs of those
actions. This revision covers skirmishes only; network games and campaigns are out of scope.

Against revision 39, revision 40 takes in the S-0203 entry of S-0318's Changes for other specs.
Claim 5 no longer says that a projectile which loses its first turn to a removal always has that
turn in the next tick: a removal in the next tick can cost it that turn too, so its first turn is
in the first tick in which no removal costs it the turn. This corrects the committed text, which
allowed only tick *n* + 1. Claim 10 is unchanged: the objects that entered play during a turn,
which it lets miss turns, already include fragments. Nothing else changes.

Within an object's turn, S-0506 claim 24 owns local launch-pass ordering: an ordinary
vehicle or ship shot cannot supply that turn's due pass, whereas an aircraft attack-run shot
can. S-0614 claim 18 owns within-host-turn parasite ordering: self-healing, due behaviour
processing and ordinary movement precede the parasite event; ordinary separate infantry and
vehicle firing follows it, while behaviour-issued shots, including aircraft attack-run shots,
precede it. S-0516 claim 42 owns within-turn cloak ordering: due behaviour processing,
including its shots, precedes the ordinary cloak update; ordinary mobile locomotion and
separate infantry/vehicle firing follow it. An uncloak completing in that update can allow
ordinary firing later in the same turn, but cannot give an aircraft attack-run shot an extra
chance after its behaviour processing. Building behaviour-fired shots likewise precede the
building's fade update. S-0616 claim 1 owns where damage sparks and deactivated-vehicle
sparks fall within an object's turn. S-0608 claim 15 owns local drain ordering for the
drainer's position and alliance checks and the drained building's payout, including their
boundary with the owner's house power update. These are local ordering constraints, not global
firing, cloak, spark or drain phases. Their backing, confidence and proposed measurements remain those of the owning claims.

## Terms

- **Turn.** An object's share of one tick: the moment at which it moves, fires, counts down,
  starts and ends its own actions. An object that "acts in tick *n*" takes its turn in tick *n*.
  In this spec "turn" never means a rotation, as in the glossary's *facing*.
- **Update order.** The sequence in which objects take their turns. Claim 2 says which objects are
  in it, claim 3 how the sequence arises.
- **Turn pass.** The part of a tick in which the objects in the update order take their turns, one
  after another (claim 4), apart from the turns that removals cost (claim 10). It is a part of the
  simulation. It has nothing to do with the *object pass* of S-0102 (named in the glossary's *animated terrain object*), which is a part of making a
  drawn frame.
- **Enter play, leave play, in play.** In this spec: to join the update order, to leave it, and to
  be in it. This narrows the everyday sense of the words, because being in play is not the same as
  being on the map. Most objects enter play when they are put on the map: placed by the map at game
  start, created (a produced unit leaving the building that produced it, a projectile fired, an
  animation started), or coming back onto the map (a passenger unloading from a transport that is
  not open-topped, a unit leaving a building it was inside). Most objects leave play when they are
  taken off the map: destroyed and removed, or going inside something. The one exception is a
  passenger of an open-topped transport, which is in play while it is off the map (claim 3).
- **Leftover.** An object that has been removed (glossary) in the current tick and is still kept by
  the game until the tick ends (claim 13). It is removed in every respect that play can see, except
  that the defeat sweep of S-0414 claim 5 can still reach it.
- **Order.** An instruction from a human player (move, attack, deploy, sell, build, hold
  production and so on), as in S-0213. Computer-controlled houses decide during their own updates
  and give no orders in this sense.
- **Projectile.** The object that carries a shot from the firer to the target or impact point and
  applies the shot's warhead where it detonates. For some weapons nothing is drawn for it.
- **House update.** A house's share of the once-per-tick work on house-wide matters, distinct
  from the turns of its objects and from the updates of its production lines. Claim 8 places it
  after the turn pass and the production-line updates of the same tick, and before that tick's
  orders; row 7 of **State and ordering** names this part of the tick. The topic specs own what
  each house update does.
- **Production line.** One of a house's production processes: the item it is building, its progress,
  whether it is on hold, and its queue. The production specs own how many lines a house has and
  what a line does; this spec places the lines' update in the tick.
- **Open-topped transport.** A transport whose type is open-topped, a property that the type takes
  from the rules data (scenario O5 sets it on a test type). This spec only uses its consequence for
  the update order (claim 3).
- **Planted bomb.** As in S-0213: a bomb that a bomb-planting infantry unit has attached to an
  object. It detonates when its own countdown ends (or earlier on its planter's detonate order,
  S-0213, claim 11). The combat specs own its countdown and its damage.
- **Kamikaze.** An aircraft enrolled for the shared repeated attack orders of S-0506 claim 26.
  The shared cycle is distinct from a spawn controller's launch pass in its spawner's turn.
- **Unlimited mind controller.** An unlimited controller as S-0601 defines it: a controller whose
  primary weapon has the unlimited-control flag. Its overload countdown counts its overload updates,
  which are its own updates, not ticks (S-0601 claim 16).
- **Storm bolt.** One lightning bolt of the storm that the weather superweapon calls down around
  its target. The superweapon specs own where and when bolts fall and what they damage.
- **Disc laser.** The ring of laser beams that a weapon with that property in the rules data forms
  around its target before it applies its damage. It is not an object in the update order and
  takes no turns; it is updated in a part of the tick of its own (row 12). The combat specs own it.
- **Radiation field.** A patch of ground left irradiated by a warhead with that property, whose
  strength fades over time. The combat specs own it.
- **Electromagnetic pulse.** The effect that disables the objects a warhead with that property
  reaches, for a time. Row 14 is the moment its time runs out. The combat and superweapon specs own
  it.
- **Blast ring.** The expanding ring that the psychic dominator superweapon (its lighting effect is the glossary's
  global effect *psychic dominator*), and one
  trigger action, create at their target. The superweapon specs own it.
- **Shared gas system.** The particle system that S-0613 (claim 1) creates at game start. Claim 3
  uses it only for the place of game-start objects in the update order.
- **Team.** A group of units under scripted control, created by a map trigger (S-0011, claim 30) or
  by the original's computer player. The specs of teams own what a team does.
- **Current object, counting removal.** During the turn pass, the *current object* is the object
  taking its turn. A *counting removal* is a removal, during the turn pass, of the current object
  or of any object before it in the update order, whether or not that object took its turn in this
  tick. Each counting removal costs one later object its turn (claim 10).
- **Trigger vocabulary.** *Tag*, *raise*, *polled tag*, the *any-event* code, *latch*, *trigger
  variable*, *one-time trigger*, the events *elapsed time* and *entered-by*, and the *count* action
  are used as in S-0011, which defines *tag*, *raise* and the trigger time unit of 15 ticks in its
  Terms, and the *counter* in its common measurement setup. The *chrono-flash* action is as in
  S-0201, scenario T6.
- **First infantry death-sequence entry.** The entry S-0107, claim 17, selects for an infantry death of kind 1.
- **Single-player game.** As in S-0201: a skirmish or a campaign mission. Only skirmishes are
  covered here.
- **Slave master, slave group, group update, slave place.** Used as in S-0402 claims 19,
  21 and 26: a vehicle or building controls harvesting infantry slaves, with persistent
  places in its group. A group update is the periodic opportunity to replace, restore,
  receive or release slaves and reconsider the master's movement, not a slave's own turn.
- **Cycle.** As in S-0201: the unit of real time in which the original runs a game. A cycle that is
  not frozen processes at most one waiting input event and then runs one tick. S-0213 claim 1
  owns the input-event limit, its definition, confidence and backing; S-0201 claim 2 owns pacing.
  A frozen cycle takes no input and runs no tick.

## Behaviour

1. **The parts of a tick and their places.** A tick of a single-player game contains the parts
   listed in the table of **State and ordering**. That table is a partial order. For each part it
   names the parts it comes after, and this spec fixes such a place only where it states an
   observable consequence of it. Where the table fixes no order between two parts, an
   implementation may run them in either order. A part sees every effect of the parts it comes
   after in the same tick.
   - **Draw order.** Every part that takes draws from the scenario random stream (S-0202), or can
     take them, has a fixed place relative to every other such part, and the table links every two
     of them. Moving or swapping drawing parts changes which part receives which raw values
     (S-0202, claim 10). This can change their outcomes and, when their draw counts depend on those
     outcomes, later outcomes too. It need not change the final stream state of the tick: swapping
     two fixed-length groups consumes the same total number of raw values. The consequence to
     test is the assignment of values to outcomes, not merely the tick's total draw count. The
     sequence retains rows 1, 3, 5 to 12 and 14 to 16, including the non-drawing,
     conditional and original-only qualifications below. Whatever draws these parts take
     come in this order: the polling of map
     triggers (row 1), resource growth and spread (row 9),
     diving aircraft (row 10), teams (row 11), disc lasers (row 12), psychic domination then
     the weather storm (row 3),
     the end of electromagnetic pulses (row 14), the turn pass (row 5),
     blast rings (row 15), crate respawns (row 16), the production lines (row 6), the houses (row 7)
     and the orders (row 8). Inside a part that holds several members (several teams, disc lasers,
     houses), the members take their draws in the order the row states. Within a
     part, the spec of the topic owns which draws are taken; this spec owns only where the part
     stands. The teams and computer-player decisions describe the original only. Our AI uses
     normal orders under S-0213 and the glossary's *AI house* rule; this table does not require a
     replacement team system or require AI decisions to draw during row 11 or row 7.
     - Backing: `measurable` for rows 1, 5 and 8 (O13), and for the cross-part
       assignment among rows 1, 9, 12, 3, 5, 15, 16, 7 and 8 (O16). O16 also tests
       newest-first disc-laser damage and house-order tag-action draws. S-0401 claim 6
       owns growth-before-spread and resource registration order; S-0520 claim 4 and
       CR2 own simultaneous crate-expiry priority. These are proposed tests, not measurements.
       The remaining qualifications are narrower than a lack of a test:
       - Rows 10 and 14: their conditional draw places are `unmeasured`. No reachable
         draw-producing effect of diving steering or pulse expiry has been identified.
         With no value assigned to either part, moving it across another drawing part
         leaves the labelled random outcomes and stream state identical. The recorder
         cannot distinguish that empty allocation. These links retain precautionary
         scheduling only; they do not assert that either part draws. A newly identified
         drawing path needs its own scenario before being labelled measurable.
       - Row 6: its own update takes no draws under S-0404 claim 11. Its purported draw
         comparison with row 7 is therefore empty, not an untested queued-creation batch.
         Item-start draws belong to the action that starts the item (claim 8), not row 6.
         The internal position of this non-drawing work relative to a house's draws is
         `unmeasured`: no random outcome can distinguish it. O7 still tests its credit
         and hold boundaries; claim 8 retains its place before the houses.
       - Row 11 and computer-strategy work in row 7 are deferred original-only context,
         not engine requirements. Their internal draw order is `unmeasured` in this
         skirmish contract because those systems are absent from our engine; ordinary
         AI orders expose no corresponding team or strategy-update event to compare.
         This does not assert that their order is unobservable in the original. O16
         tests in-scope house updates through tag actions, without requiring strategy draws.
       - The domination ring has no draw allocation (S-0612 claim 8, PD9). O16 tests
         row 15's placement using a damage-enabled ring's first-update animation draws.
         This establishes a phase-placement probe, not the details of every ring effect
         or an order among multiple damage-enabled rings, neither of which is specified here.
     - Confidence: high for the sequence, for row 6's absence of own-update draws
       (S-0404 claim 11), for row 12's absence of producer draws (S-0615 claim 18),
       and for the domination ring's absence of draws (S-0612 claim 8); medium for
       other row-15 effects; low that rows 10 and 14 take any draws at all.
   - **Row 1, the polling of map triggers, comes before the turn pass.** Row 1 is the per-tick raise
     of every polled tag (S-0011, claim 6), which also owns how the raise works. A trigger action
     that this polling sets off (for example one that destroys an object) takes effect before any
     object takes its turn in that tick.
     - Backing: `measurable` (scenario O6, part A)
     - Confidence: high
   - **Row 2 is planted-bomb maintenance, not detonation.** It drops bombs without a carrier,
     maintains their ticking sounds and refreshes visibility every 46 ticks, with the
     planting qualification of S-0519 claim 10. It takes no scenario random draws.
     A timed bomb instead detonates during its on-map carrier's own turn in row 5
     (S-0519 claim 4). Earlier objects have already acted, and an opportunity shot by the
     carrier can precede detonation. A later object killed and removed by the explosion
     takes no turn that tick. Removal of the carrier or an earlier object is a counting
     removal and costs a later object its turn under claim 10; removing only an object
     after the carrier does not. Claim 6's retained-death exceptions still apply.
     Detonations caused by death or an explicit order retain the timing of their cause
     (S-0312 claim 7; S-0519 claim 9), not a shared bomb-detonation pass.
     - Backing: `measurable` (scenario O10 part A; S-0519 scenarios T4, T5 and T11).
       O16 part F additionally tests the absence of maintenance draws during sound
       and visibility refreshes; it does not test every possible sound configuration.
     - Confidence: high
   - **Row 17, shared kamikaze re-orders, comes after row 2 and before row 10.**
     S-0506 claim 26 owns the shared cycle and its deadlines. Its due ammunition, target
     and queued attack changes are available to aircraft turns in the same tick. A release
     during a spawner's turn cannot receive that tick's already-completed shared-cycle order.
     The assignment itself takes no scenario random draws; later aircraft behaviour processing
     and firing keep their ordinary draws and gates. This is not the carrier-slot ordering
     of S-0501 claim 35.
     - Backing: `measurable` (S-0506 scenario R21 for the pre-turn boundary and absence
       of immediate assignment draws). A timer-driven planted-bomb release is during
       the turn pass, too late for this tick's shared cycle.
     - Confidence: high
   - **Row 3, psychic domination and the weather storm, comes before the turn pass.**
     The domination update, including its sweep and area damage, precedes the storm's own
     update (S-0612 claims 4, 9 and 14). The sweep therefore uses the storm owner before
     that tick's storm cleanup, and a taken-over unit acts under its new owner in that tick.
     An object that the sweep's damage or a storm bolt destroys in tick *n* takes no turn
     in tick *n*, unless claim 6 keeps it in play after its death. Its removal is outside
     the turn pass and costs no other object a turn (claim 10).
     - Backing: `measurable` (scenario O10, part B; S-0612 scenarios PD3 and PD5)
     - Confidence: high
   - **Row 4, the step of a chrono flash in force, comes after row 1 and before the turn pass.** The
     step that a chrono flash takes in a tick (S-0201, claim 11) is part of row 4. So a chrono flash
     started by a row-1 trigger takes its first step in the tick of the action, and one started by a
     trigger raised during the turn pass takes its first step in the next tick.
     - Backing: `measurable` (scenario O6, part B)
     - Confidence: high
   - **Rows 5 to 8.** The turn pass is row 5 (claims 2 to 6, 10 and 11). Claims 7 and 8 own the
     places of the production lines, the houses and the orders. After the last part of a tick the
     tick number grows by 1, unless the tick ends the game (S-0201, claim 1).
   - **Row 13, radiation maintenance.** Field updates take no scenario random draws
     (S-0525 claim 12). Their place after the storm and before the turn pass matters instead
     because a field made by a storm-killed object's death weapon updates in its creation tick,
     and a decay step is already reflected in that tick's radiation hits. Hits happen in each
     affected object's own turn in row 5, not in row 13; any damage-effect or death draws happen
     there too (S-0525 claims 9, 11 and 12). A field made during the turn pass first updates
     in the next tick (S-0525 claim 6). The member order remains most recently created first
     for the cell-radiation sums of S-0525 claims 3 and 5.
     - Backing: `measurable` (S-0525 scenarios RF3, RF5 part b, RF8 and RF10)
     - Confidence: high for maintenance before hits and absence of draws; medium for the
       same-tick-creation residue and within-turn hit consequences, as in S-0525
   - **Rows 9 to 12 and 14 to 16.** These parts retain the draw-order constraints above.
     The domination's blast ring takes no draws and first updates in its creation tick
     (S-0612 claim 8, scenarios PD3 part b and PD9). Apart from that and the disc laser's
     consequence below, no other consequence of their places is stated here. The specs of
     their topics own what each one does. A disc laser
     (row 12) applies its damage in its own update, outside any object's turn. So, as in row 3,
     an object it destroys takes no turn in that tick, with claim 6's exceptions, and its
     removal costs no object a turn (claim 10).
     For a shot in tick *T*'s turn pass, its damage falls in tick *T* + 17, subject to
     cancellation (S-0615 claim 13). The disc laser itself draws nothing from either random
     stream; its detonation retains its ordinary draws (S-0615 claim 18).
     - Backing: `measurable` (S-0615 scenario DL1). The cross-part draw placement retains
       the limits stated above.
     - Confidence: high
   - **Other work done once per tick.** The original does further work once per tick outside any
     object's turn that takes no draws from the scenario random stream: the regrowth of shroud and
     the re-covering by fog of war (visibility), a periodic re-check of what is visible, the step of
     a fading map-wide light level and the relighting of the map after lighting changes (S-0104),
     the update of the player's view and some purely presentational timers. This spec gives that
     work no place and no order. An owning spec that finds an observable consequence of its place
     states the place, with that consequence and a test. Until then an implementation may run it
     anywhere in the tick, which is a rule of our engine, not a claim about the original.
     This freedom covers only work that takes no draws from the scenario random stream. Work that
     another spec adds and that takes such draws needs a place in the table, set by its owning spec
     in the original's order.
     - Backing: the existence and effects of each item use its owning spec's backing. The
       aggregate assertion that none of these paths draws is `unmeasured` here: no finite
       quiet-map run exercises every condition and effect of all the listed work, so a zero
       draw count would confirm only the tested paths, not this whole exclusion.
     - Confidence: high that the original does such work once per tick outside any object's turn,
       medium that it takes no draws (none was found, but not every path was traced)

2. **Who takes turns.** Infantry, vehicles, aircraft, buildings, projectiles, animations (sprite
   and voxel), particle systems, the travelling beam or wave of the weapons that fire one, the
   searchlight beams of buildings, and terrain objects whose type has the spawns-resources flag
   (Import mapping; off when absent) take turns. The
   individual particles of a particle system take none of their own (the system moves them in its
   turn), and neither do overlays, smudges or other terrain objects. An object out of play takes no
   turns. A passenger of an open-topped transport stays in play while it is inside, so it does take
   turns (claim 3).
   - Backing: `measurable` in part (scenarios O1 and O3 for infantry, vehicles, buildings and
     projectiles, scenario O5 for open-topped passengers, O13 for a sprite animation's turn).
     S-0109 claim 29 and AN26 establish the creator placements for sprite animations:
     a successfully placed ordinary shot projectile enters play before its shot animation, and
     detonation and death animations enter play at their causing event, in that event's tick
     part. Those created before or during the turn pass are eligible for a turn in that pass,
     not guaranteed one (claims 4 and 10). AN26 tests this distinction, including the
     projectile-removal case in claim 10 below; O17 tests an ending animation's removal.
     `unmeasured` here for aircraft, voxel animations, particle systems, beams, searchlights and
     terrain objects: no scenario here isolates their turn against another object's turn.
   - Confidence: high for every kind, terrain objects included: a terrain object enters the update
     order when it is placed exactly when its type's spawns-resources flag is on, and no other field
     of a terrain type plays a part

3. **The update order is the order of entering play.**
   - **Appending.** An object that enters play is put at the end of the update order. An object
     that leaves play is taken out of it, and the others keep their order. An object that enters
     play again goes to the end, not back to its old place. The update order is never sorted by
     anything else: not by position, kind, owner or depth.
   - **Objects placed by the map.** At game start they enter play in this order: terrain objects
     (only those that take turns, claim 2), then vehicles, then aircraft, then infantry, then
     buildings, each kind in the order of the map's entries (S-0009 owns the entries). In a
     skirmish the players' starting units enter play after all of them. Because every kind is a
     block of its own, an object of one kind never stands between two map-placed objects of
     another kind. So at game start the last map-placed infantry unit is directly followed by the
     first map-placed building, unless a game-start animation or the shared gas system stands
     between them (see Other game-start objects below). As long as both stay in play, a counting
     removal (claim 10) in that infantry unit's turn costs that building its turn, whatever terrain
     objects, vehicles or aircraft the map holds.
   - **Other game-start objects.** The animations and the shared gas system that are created at
     game start are not map-placed, and this spec does not order them against the starting units.
     Each enters play by the appending rule above, at the point its game-start event falls in the
     order that S-0412 claim 14 owns (terrain animations also S-0010, cell animations S-0009). No
     observable consequence of that place is claimed here. Only the block order above (map-placed
     kinds, then the starting units) is claimed here. Whether one of these objects stands between
     two map-placed objects is not claimed here either (see the adjacency sentence above).
   - **Open-topped transports.** A unit that boards an open-topped transport leaves play and enters
     play again at once: it is taken out of the update order, which is a removal for claim 10, and
     put back at its end. It then takes turns while inside, so it can, for example, fire
     from inside. If the turn pass is running when it boards, it takes a turn later in that same
     pass (claim 4). If it had already taken its turn in that tick, that is its second turn of the
     tick (claim 11), which happens only with the switch of claims 10 and 11 on. When it gets out
     it stays where it is in the update order and does not move to the end. A passenger of a
     transport that is not open-topped is out of play while inside.
   - Backing:
     - `measurable` (scenario O3) for appending, for the kinds vehicles, infantry and buildings and
       for the entry order within one kind.
     - `measurable` (scenario O9, variant T) for the block rule between infantry and buildings with
       a terrain object that takes turns on the map.
     - `measurable` (scenario O5) for the turns of open-topped passengers.
     - `measurable` (scenario O11) for a boarding passenger's move to the end of the update order
       and for a passenger that gets out keeping its place.
     - For the second turn of a passenger that boards after its own turn: claim 11's backing
       (`unmeasured`).
     - `unmeasured` for which object loses its turn because of the removal that boarding causes.
       That depends on whose turn performs the boarding, which is not known (Open questions), so no
       setup can predict the object.
     - `unmeasured` here for the game-start animations' turns against the starting units: no
       scenario here isolates them (see the bullet above for the owning specs).
     - `unmeasured` for the place of terrain objects before the vehicles and of the aircraft between
       the vehicles and the infantry. Each of these places shows only through a removal in the turn
       of the last map-placed object before it (the last terrain object, vehicle or aircraft), and
       no terrain object, vehicle or aircraft is known to leave play in its own turn at a tick a
       setup can predict. No setup is known, either, in which a pre-placed aircraft fires at a
       steady rate from a fixed spot.
     - `unmeasured` for the starting units. The game places them itself and gives every player
       several, so a setup cannot fix which single starting unit faces a pre-placed one.
   - Confidence: high for appending and the kept order; medium for the order of the map's kinds,
     the starting units and the open-topped rule; low for the place of game-start animations and
     the shared gas system

4. **Objects that enter play during the turn pass.** An object that enters play during the turn pass
   of tick *n* (a projectile just fired, a unit just unloaded or produced, an animation just
   started) takes its first turn later in that same turn pass, after every object that was already
   in the update order, unless a removal costs it that turn (claim 10). An object that had already
   taken its turn in that tick before it left play and entered play again takes this turn only
   with the switch of claims 10 and 11 on (claim 11).
   **Within-turn falling and landing.** S-0511 claim 10 owns the ordinary infantry and
   vehicle case: the fall step and any landing hit precede due behaviour processing,
   ordinary locomotion advance and the firing check in that turn. A unit still falling
   after the step can process due behaviour, but neither advances its ordinary locomotion
   nor fires. A surviving landing removes those falling-specific refusals in the same
   turn. Falling is not a missed turn under claim 10 here, nor a separate tick part.
   - Backing for this local ordering: `measurable` (S-0511 scenario B3, turn-order variant)
   - Confidence for this local ordering: high, as in S-0511 claim 10

   **Within-turn superweapon warp: original evidence.** S-0513 claim 48 owns the confirmed boundary
   separately for vehicles, infantry and landed aircraft: a member still in its leaving
   stage after its warp update drops target and destination and skips ordinary healing,
   behaviour processing, movement, firing and turret turning for that turn. Its former
   driving does not continue during the warp. The first pending turn is not automatically
   excluded from ordinary work before entering the leaving stage. These are restrictions
   within a turn, not a missed turn under claim 10 or a separate tick part. In particular,
   no universal ordering of warp work after protection brightness processing is implied.
   For a vehicle, infantry unit or landed aircraft beginning its turn in arrival hold,
   the hold-expiry test precedes ordinary healing, behaviour, movement, firing and
   turret-work opportunities in that turn (S-0513 claim 48). This fixes ordering only:
   eligibility and continuation of each ordinary operation remain explicitly unmeasured.
   Exact continuation at departure, off-map retries, placement and arrival likewise
   remains unresolved for each kind under S-0513 claim 48. S-0513 claim 39's timeline
   assumes one warp update per tick; neither it nor the one-turn constant here establishes
   that update count or makes the warp update the whole turn through arrival.
   - Backing for these boundaries: `measurable` (S-0513 scenario SW23, including part e
     for hold-test ordering); per-operation eligibility, continuation and per-kind warp
     update counts remain `unmeasured` for the reasons in S-0513 claim 48
   - Confidence for these boundaries: high for the leaving-stage exclusion and hold-test
     ordering; low for unresolved eligibility, continuation and update counts, as in
     S-0513 claim 48

   **Within-turn superweapon warp: engine contract.** S-0513 claim 51 separately
   supplies our deterministic rule for vehicles, teleporter infantry and landed aircraft.
   A living member receives one warp update per tick throughout pending, leaving,
   landing retries, placement awaiting effects and arrival hold, including while off-map.
   From the first pending update through arrival effects, the warp update is the whole
   member turn, apart from the intrinsic exceptions below. Ordinary work is suppressed
   both before and after it: self-healing, the per-tick behaviour start and behaviour
   processing, movement, firing, turret turning, ordinary cloak updates, ammunition
   reload updates and firing-frame advancement, attached parasite steps, the member's
   economy and power work, depot queued-entry retries, expected-health turn bookkeeping,
   protection-brightness phase processing, the berserk countdown (S-0610 claim 5) and the
   overload update of an unlimited mind controller (S-0601 claim 16), whose countdown and
   pulse timing stay as they are through the warp, because they count the controller's overload
   updates, not ticks (S-0601 claim 16). The periodic held-target check (S-0309 claim 29) and the
   drop of an allied held target at its place (S-0309 claim 25 part k) are suppressed as well. No
   step is made up later. In arrival hold, including its expiry turn, the hold-end test precedes all
   these ordinary opportunities, each still subject to its own eligibility. No deferred ordinary
   updates are replayed in a catch-up burst. Launch immediately clears the old path, queued
   destinations and driving step; none resumes after the warp. Target and current destination retain
   claim 48's leaving-stage clearing rather than an added launch-time clearing. Intrinsic warp
   target searches, sensor updates, elapsed protection duration, placement and arrival-fate tests
   are not suppressed ordinary work, and neither is the erasure update of a temporal beam attached
   to the member (S-0514 claim 7), which comes before the warp work in each of the member's turns
   (S-0513 claim 39a), nor S-0514 claim 18's sparkle, made before the warp work in each of the
   member's turns in which it is then being erased or phasing in, nor S-0514 claim 17's clearing of
   target and destination, made after the warp work in each turn in which the member is then being
   erased, or is a vehicle or aircraft phasing in with a beam attached, or an infantry member in
   that state with S-0513 `quirks.phase_in_beam_spares_infantry` off (S-0513 claim 39a, *Sparkles
   and drops in the warp's turns*). So in our engine the erasure update of a member being erased
   runs in every warp turn, and S-0513 claim 39a's removal tick holds, and a member sparkles in its
   warp turns and clears its target there as S-0514 gives. Elapsed-tick clocks keep running:
   suppressed reload work grants no ammunition and restarts no countdown, but its elapsed time still
   passes. Skipping firing-frame advancement likewise does not freeze a shot cooldown. Independent
   house updates, other objects' updates, immediate damage and explicit event-driven effects remain
   eligible. In particular, launch/departure parasite releases and arrival sensor transfer still
   occur; skipping expected-health turn bookkeeping does not cancel actual damage or death. These
   restrictions are not missed turns under claim 10 and create no separate global tick part. The
   update count makes S-0513 claim 39's timeline applicable in our engine for uninterrupted
   successful landing and placement. It does not settle the original's per-kind update count or
   continuation gaps above. This is an engine design rule, not a reproduction claim, and has no
   quirk switch.
   - Backing for this contract: `measurable` in our engine (S-0513 scenario SW23 part f). For
     the berserk countdown in warp, which S-0513 does not mention, the backing is S-0610
     scenario P9 part e, also measurable in our engine (S-0610 claim 5).
     The overload update in warp is a design rule of our engine, `measurable` in our engine (SW23 part f, which the Measurement scenarios extend to the overload countdown).
     The suppression of the periodic held-target check and of the allied held-target drop is a
     design rule of our engine, `measurable` in our engine (S-0513 SW23 part f). The erasure update
     in warp turns is `measurable` in our engine (S-0513 SW12 part l1). That the original also
     makes that update in every warp turn is `measurable` (S-0513 SW12 part l1) in the stages l1
     reaches (a tank member, an empty flat
     destination and one warp with no failed landing check, no placement failure and a hold of one
     update), and `unmeasured` for a failed landing check, a placement failure, a longer hold and an
     infantry or aircraft member, as S-0513 claim 39a gives. Its order within the turn, before the
     warp work, is design, tested in our engine by S-0513 SW12 part l4; that the original keeps
     that order is `unmeasured`, as S-0513 claim 39a gives. The original-side gaps retain claim
     48's `unmeasured` backing. The sparkles and the clearing in warp turns: `measurable` in our
     engine (S-0513 SW12 part l6) for the sparkles. That the original
     makes them likewise is
     `measurable` (S-0513 SW12 part l6) for the sparkles of a tank member, and `unmeasured` for the
     clearing beyond claim 48's leaving rule and for an infantry or aircraft member, as S-0513
     claim 39a gives. The original's behaviour in the warp updates other than the leaving
     stage is an open question of S-0610
   - Confidence for this contract: high as design; low for matching the original's
     unresolved rows, as in S-0513 claim 51. The suppression of the held-target check and drop:
     high as design. The erasure update in warp turns: high as design for our engine; for the
     original, medium in every stage that it makes the update, and low for its order before the
     warp work (S-0513 claim 39a). The sparkles and the clearing in warp turns: high as design for
     our engine; for the original, medium, as S-0513 claim 39a rates it

   - Backing: `measurable` (scenarios O1 and O3). For the turn of an object that re-enters play
     after its own turn: claim 11's backing (`unmeasured`).
   - Confidence: high

5. **Projectile shots are resolved in the projectile's own turn.** A projectile shot's damage is
   applied at the moment its projectile detonates, during the projectile's own turn, to every
   object the warhead reaches. This is not a rule for damage without an in-play projectile,
   such as a death weapon, planted bomb, storm bolt or the beam and disc laser cases below.
   There is no separate damage step later in the tick. A projectile that detonates leaves play in
   that same turn. A projectile fired in tick *n* (or launched in it, as a fragment is) takes its
   first turn in tick *n* (claim 4), unless a removal costs it that turn (claim 10). Its first turn
   is then in the first later tick in which no removal costs it the turn. That is usually tick
   *n* + 1, but a removal can cost it the turn in tick *n* + 1 as well. S-0318 claim 5 owns the
   fragment cases and their backing. The points below follow from this, or add to it.
   - **(a) Mutual fire.** Take two units already in play that fire at each other in the same tick
     *n*, with weapons whose projectiles reach the target on their first turn (one with no visible
     flight, for example). Both shots are always fired, since neither projectile acts before both
     units have taken their turns. Both shots land, and both units can die from them. Which unit is
     hit first does not depend on any shot being faster: it follows from the update order.
     - Backing: `measurable` (scenarios O1 and O3)
     - Confidence: high
   - **(b) Stagger.** In the setting of point (a), when nothing else entered play between the two
     projectiles, the first projectile's detonation makes the second miss its first turn (claim
     10). The earlier unit's shot lands in tick *n*, the later unit's in tick *n* + 1. So the unit
     later in the update order is hit in tick *n* and the earlier one in tick *n* + 1. With the
     switch of claim 10 off, both are hit in tick *n*.
     - Backing: `measurable` (scenarios O1, O3 and O8)
     - Confidence: high for the rule; the first-turn detonation conditions are owned by S-0306
   - **(c) Projectiles with a flight.** A projectile with a longer flight detonates in a later
     tick, in its own turn, and leaves play in that turn. That detonation is a removal like any
     other for claim 10.
     - Backing: `measurable` (scenario O3, variant e)
     - Confidence: high
   - **(d) Detonation after an animation.** S-0306 claim 25 owns the nuclear-warhead case:
     when its waiting animation type exists, the projectile starts that animation and detonates
     in its first turn after the animation ends. When the type is absent it detonates without
     that wait. Neither case creates a separate damage phase.
     - Backing: `measurable` (S-0306 scenario P19)
     - Confidence: high for the selection and wait, as S-0306 claim 25
   - **(e) Beam and wave weapons.** A weapon that fires a travelling beam or wave instead of a
     projectile applies its damage in the beam's own turns, while the beam lasts.
     - Backing: `unmeasured`. Which weapons fire such beams, and how often a beam deals damage,
       is the combat specs' to establish, and the place of a beam's turn has no other effect a
       recorder can time.
     - Confidence: low
   - **(f) Disc lasers.** A weapon that fires a disc laser in tick *T*'s turn pass applies its
     damage in row 12 of tick *T* + 17, not in an object's turn, unless it is cancelled
     (S-0615 claim 13). The disc laser itself takes no random draws; its detonation has the
     ordinary draws (S-0615 claim 18).
     - Backing: `measurable` (S-0615 scenario DL1)
     - Confidence: high

6. **When a destroyed object stops acting.** This claim gives the general rule for each kind of
   object and where its removal falls in the tick. S-0312 owns the death effects and the exceptions
   below, in which a destroyed object stays in play for a while; the points here only point to it.
   - **(a) Vehicles.** As a rule, a vehicle whose damage destroys it leaves play at the moment that
     damage is applied, at the end of its death effects, which all come at once (S-0312 claims 1
     and 15). It takes no further turn, not even later in the same turn pass. Its removal is a
     removal for claim 10, and it costs a turn only under claim 10's condition: during the turn
     pass, when the vehicle is the current object or stands before it. The exceptions, owned by
     S-0312:
     - ordinary killing-hit sinking can retain 1 health point (S-0312 claim 10), but
       sinking does not always mean 1 health point: a non-crashable vehicle already
       sinking can remain in play at 0 hit points after another killing hit when the ordinary
       sinking conditions fail (S-0312 claims 15 and 32). The naval specs own descent
       and completion. Death-image-frame types retain S-0312's unresolved separate path,
     - a vehicle whose type can crash and which is above the terrain when it is destroyed starts
       to crash and stays in play until it hits the ground (S-0312 claims 15 and 16).
       If it cannot start a crash, it leaves play immediately even when already sinking
       (S-0312 claims 15 and 32).

     When such a vehicle does leave play, that is a removal for claim 10 like any other.
     - Backing: `measurable` (scenario O1; scenario O8, variant a, lethal against non-lethal) for
       the rule; for the exceptions, S-0312's backing (its scenarios D8, D11 and D23)
     - Confidence: high for the rule; for the exceptions, S-0312's confidence
   - **(b) Buildings.** As a rule, a building whose damage destroys it leaves play like a vehicle,
     at the moment that damage is applied, at the end of its death effects, and that removal is a
     removal for claim 10. The exception is owned by S-0312 claim 21: a building whose type has
     the explodes flag, or one destroyed while it is being sold, stays in play with 0 hit points after
     its death effects. It normally leaves in its own next turn, later in the same tick if that
     turn has not yet come, otherwise in a later tick. S-0312 claim 21 owns the erasure and
     electromagnetic-pulse exceptions and the second survivor and smudge round in the removal
     turn, with `quirks.building_removal_second_round`. This switch is on by default, preserving the original behaviour; when off, the building takes no second survivor and smudge round when it leaves play. A sale does not advance or complete once
     the building has 0 hit points; its removal follows S-0312 claim 21, not sale completion. A veterancy
     ability or a weapon's suicide property alone does not cause this deferred removal. Its
     leaving then is a removal for claim 10 like any other.
     - Backing: `measurable` (scenario O1, variant B, for the rule); for the exception, S-0312's
       backing (its scenario D14, variants c and g)
     - Confidence: high for the rule; for the exception, S-0312's confidence
   - **(c) Infantry.** S-0107, claim 17, decides what a killed infantry unit shows and when it is
     removed. An infantry unit leaves play exactly when it is removed. When claim 17 removes it at
     once, it leaves play at the moment the damage is applied, like a vehicle, and that removal
     is a removal for claim 10. When claim 17 gives it a death sequence, it stays in play while the
     sequence plays, so the kill itself is no removal for claim 10, and it leaves play on the tick
     the sequence completes. That leaving is a removal for claim 10 too, and it is believed to
     happen in the unit's own turn. An infantry unit whose type has the crash flag and which starts
     to crash instead (S-0107 claim 17, S-0312 claim 16) stays in play until it hits the ground.
     - Backing: `measurable` (scenario O8, variants d and e, for the moment of leaving play, and
       scenario O9 for the removal when the sequence completes); for the crash, S-0312's backing
     - Confidence: medium for removal at once and for staying in play during the sequence; low that
       the removal at completion happens in the unit's own turn; for the crash, S-0312's confidence
   - **(d) Projectiles of destroyed firers.** A projectile whose firer has been destroyed still
     flies, detonates and applies its damage.
     - Backing: `measurable` (scenario O1: the later firer's shot lands after it is destroyed)
     - Confidence: high
   - **(e) Passengers.** S-0312 decides, for each passenger of a destroyed object, whether it dies
     or gets out (claims 5, 10, 12, 16 and 24). As a rule, a passenger that dies with the object
     leaves play when it dies, which is at the latest when the object itself leaves play. A
     passenger of an open-topped transport is in play (claim 3), so each one that dies is one more
     removal for claim 10, under the same condition as any other. A passenger that gets out
     instead, if it was in an open-topped transport, stays in play: it is back on the map and
     keeps its place in the update order, as when it gets out by itself (claim 3), so it is no
     removal. Any other escaping passenger was out of play and enters play then (claim 4). Passengers inside an object that is not open-topped are out
     of play, so their deaths are no removals.
     - Backing: `measurable` (scenario O8, variant c against variant b for passengers that die, and
       variant f against variant c for passengers that get out)
     - Confidence: high for which passengers are in play; medium for the turns, until scenario O8
       confirms them
   - Aircraft follow S-0312 claim 16: an aircraft destroyed above the terrain starts to crash and
     stays in play until it hits the ground; one destroyed on the ground leaves play at once, like a
     vehicle. Its leaving is a removal for claim 10 like any other.
     - Backing: `measurable` (S-0312 scenario D11)
     - Confidence: S-0312's confidence

7. **Orders take effect after the turn pass.** In a single-player game an order that the player
   gives in the cycle that runs tick *n* (S-0201, *cycle*) takes effect after every object's turn
   in tick *n* and after the production lines of tick *n* (claim 8), and before any object's turn in
   tick *n* + 1, if that next tick runs. The objects it concerns first take ordinary turns on it in
   tick *n* + 1; orders in the final tick still take effect, but there is no next turn (S-0414
   claims 8 and 16). How several
   orders of the same tick are ordered among themselves is S-0213, claim 3. The drawn frame made
   before tick *n* + 1 (S-0201, claim 6) already shows what the order changes at once, such as a
   selection's new destination marker, but no object has yet moved because of it. The orders also
   come after the houses' updates of tick *n* (claim 8). A surrender order therefore causes no
   surrender destruction until the next tick's house update (S-0414 claim 11). Another consequence
   is the draw order of claim 1: carrying out an order can take draws from the scenario random
   stream (for example by creating objects or firing a superweapon), and so can a house's update.

   **Immediate target-change effects.** Waiting for the next ordinary turn does not defer
   event-driven effects. Under S-0519 claim 19, changing or losing a designator's target
   cancels its strike when that change applies, even after the designator has taken its
   turn. Thus a target lost during a later object's turn cancels in that event's tick.
   An applied order changing the target in row 8 likewise cancels in row 8 of that tick.
   The aircraft's cancellation effects do not wait for the designator's next turn,
   another designation impact or a shared strike pass. Aircraft behaviour processing
   still follows its own ordinary opportunities. Reapplying the same target does not
   cancel; changing from A to B and back to A does not undo either cancellation.
   - Backing for these event effects: `measurable` (S-0519 scenario T19)
   - Confidence for these event effects: high

   - Backing: `measurable` (scenario O2 for the turn pass; scenario O7, part C, for the production
     lines; O13 for the drawing order after row 5; S-0414 scenario V8 for surrender after the
     houses, and V12 part c for orders in the final tick); O16 part D for row-7 tag-action
     draws before row-8 order draws
   - Confidence: high for the turn pass and for the place after the houses; medium for the
     production lines

8. **Production lines and houses see the whole turn pass.**
   - **Production lines.** Every production line is updated after the turn pass of the same tick, so
     everything objects did in that pass is in effect for it. For example, a line that has been
     waiting for money, and that is ready to make its next progress step in tick *n*, makes it in
     tick *n* when a harvester delivers enough credits during the turn pass of tick *n*. The lines
     also come before the orders of the tick (claim 7). So a hold order given in the cycle of tick
     *n* does not stop a progress step that falls in tick *n*: the line is on hold at the end of
     tick *n* and has still made that step. S-0404 claim 1 owns the creation-order payment
     priority when a house's lines compete for credits. Item-start creation draws instead follow
     the causing-action precedence of S-0404 claim 11, not payment priority.
   - **Houses.** Each house's own update also comes after the turn pass of the same tick. A house
     matter decided in the house update therefore sees what the turn pass did. One example is the
     raise of a house's tags in every eighth tick (S-0011, claim 35): it sees a trigger variable that
     an object's turn set earlier in the same tick. S-0414 claims 1, 10 and 11 own the defeat
     tests and surrender destruction in this part: a defeat's end test still counts later houses
     that have not yet failed their own tests in this tick. S-0414 claim 15 owns the destruction
     hit and death-draw order across object kinds, including off-map objects and its cancellation
     exception. This is creation order, not claim 3's order of entering play, and these removals
     occur outside the turn pass, so cost no object a turn under claim 10.
   - **Within a house update and in the final tick.** S-0414 claim 16 owns the observable ordering:
     ordinary power and radar work do not see surrender or defeat destruction from that same
     update, but its due tech-tree recheck and sidebar cancellation orders do. Defeat's forced
     radar-on effect remains immediate. S-0414 claim 18 places the assigned-tag raise in ticks
     whose integer tick number is a multiple of 8, including tick 0: it sees that house's
     surrender destruction and defeat test, including defeat destruction, from the same update.
     Its actions can affect the due tech-tree recheck and sidebar cancellation orders in that
     update, but newly due power or ordinary radar work waits for the next house update.
     If a tag action removes the last qualifying object, no second defeat test runs in that
     update: the next test is in the next tick. Defeated houses keep these raises in house order,
     including the update that defeats them and later eligible ticks during the end delay.
     Cleanup-removed houses have no update and raise nothing. The separate automatic unit-release
     work stays owned by S-0411 claim 9; this adds no ordering for it.
     At the start of the houses' part of the final tick,
     pending results and expiry become committed under S-0414 claims 8 and 16; later defeat
     tests cannot change them. All remaining house updates and the orders still run. S-0414
     identifies the common cutoff for all houses as a design extension of the original's
     local-player result. Other house-wide work belongs to the economy, production and
     superweapon specs.
     - Backing: `measurable` (S-0414 scenarios V7, V8, V11, V12 and V15, with V13 for
       cleanup absence and S-0011 T20 variant C for variable effects in house order) for the original's
       consequences; `unmeasured` for the common per-house commitment, a design rule that the
       original's local-player-only result cannot confirm. Claim 10 backs the no-turn-skip rule.
     - Confidence: S-0414's high for destruction creation order and within-update ordering,
       medium for cancellation skips, same-tick results and final-tick commitment; high for
       removals outside the turn pass costing no turn, as in claim 10.
   - **Draw order among them.** A house's update can take draws from the scenario random stream,
     one house after another in house order (S-0009, claim 19), before the orders (claim 7).
     Production lines retain their place before houses, but not a draw group:
     S-0404 claim 11 distinguishes item-start creation draws from a line's own update,
     which takes none under that claim. Each start receives its
     creation draw at the action that causes it, with no value reserved for a line in advance.
     Starts caused during the turn pass inherit the causing objects' turn order and precede
     starts caused by orders at that tick's end. At the order boundary, starts follow production
     orders' application order (S-0213 claim 3), including S-0411 claim 13's tab/icon order for
     automatic cancellations. Draws from intervening actions remain between the starts.
     Across houses, turn-pass starts still follow turn order; order-boundary starts in our engine
     follow house creation order and each house's production-order issue order. That between-house
     order-boundary extension is a deterministic design rule, not an observed rule of the original's
     computer production. Completion alone starts no replacement and creates no draw batch in
     row 6; the later place order causes release and any queued replacement (S-0404 claim 11).
     - Backing: `measurable` (S-0404 P11 B for same-boundary assignment and P11 C for a turn-pass
       start before an order-boundary start). The between-house order-boundary extension is
       `unmeasured`, because it is a design rule, not an oracle outcome.
     - Confidence: medium for causing-action assignment, as in S-0404 claim 11.
   - Backing: `measurable` (scenario O7, part A for the lines after the turn pass, part C for the
     lines before the orders, and part B for the houses); O16 parts C to E for house
     tag-action draw placement and house order. The non-drawing row-6 comparison and
     excluded computer-strategy work retain claim 1's `unmeasured` qualifications
   - Confidence: high for the lines after the turn pass and for the draw order among the houses,
     medium for the lines before the orders and the houses, and for the lines before the houses

9. **Timings run on the tick number, not on turns.** A timing of game state that an object waits
   for, such as the delay before its weapon can fire again, is measured against the tick number
   (S-0201): it ends on the same tick whether or not the object took its turns meanwhile. An object
   that missed a turn (claim 10) or was out of play for a while finds its delays have run on. A few
   counters instead step once per turn; the spec that owns each such counter says so.
   - Backing: `measurable` (scenario O4)
   - Confidence: medium

10. **A removal can cost the next objects their turns.** The condition depends only on where the
    removed object stood in the update order, not on whether it took its turn. As in Terms, the
    *current object* is the object taking its turn, and a removal *counts* when the object that
    leaves play is the current object itself or stands anywhere before it in the update
    order, whether that object took its turn in this tick or missed it (by this claim, earlier in
    the same pass). Each counting removal during one turn costs one turn: *k* counting removals
    during the current object's turn make the *k* objects that directly follow it, at the end of
    that turn, miss their turns in this tick. Objects that entered play during that turn can be
    among them, and if fewer than *k* objects follow, the pass simply ends. A removal
    of an object that stands after the current one costs nobody a turn, and so does a removal while
    the turn pass is not running (every row but row 5). Causes, each with its own backing:
    - A projectile that detonates and leaves play in its own turn (one removal): `measurable`
      (scenarios O1, O3 and O8).
    - The same projectile destroying a vehicle that stands before it in the update order (a second
      removal), and those passengers of that vehicle that die with it if it is an open-topped
      transport (one more removal each, claim 6): `measurable` (scenario O8, variants a and c). Every object that was in play
      when the projectile was fired stands before it (claim 3).
    - The same projectile destroying an infantry unit that claim 6 removes at once: `measurable`
      (scenario O8, variant d), also when that unit missed its turn in the same tick (scenario
      O12).
    - An infantry unit whose death sequence completes (claim 6): `measurable` (scenario O9).
    - An animation that ends: `measurable` (O17). Its creator determines its appended place
      under S-0109 claim 29; there is no separate death-animation pass.
    - A returning slave going inside its master during delivery: `measurable` (O14 D).
      This happens in the master's turn (claim 12), so only a slave earlier than the master
      in update order counts.
    - A unit boarding an open-topped transport (claim 3), or other entry exchanges: `unmeasured`.
      Whose turn performs those exchanges is not known (Open questions), so no setup can
      predict which object loses its turn.

    **Consequence for a newly created explosion.** Suppose a projectile is last in the update
    order before its turn, creates exactly one explosion animation, kills nothing, creates
    nothing else and leaves play in that turn. The animation is appended during the pass
    before the projectile leaves (S-0109 claim 29). The projectile's own counting removal
    costs that animation its creation-tick turn. It first updates next tick, subject to any
    further missed turns; creation during the pass does not itself guarantee a same-tick
    update. With the removal-skip switch off, it first updates in its creation tick.
    S-0109 claim 5 owns the first-update stage hold, so this lost turn also postpones the
    animation's first advance and completion by one tick in an otherwise uninterrupted run.
    - Backing: `measurable` (S-0109 scenario AN26, detonation/removal fixture)
    - Confidence: high for creation placement and the conditional removal rule; medium for
      the minimal fixture before measurement

    **Consequence for an object that missed its turn.** Suppose projectile P1 detonates in its
    turn, so that unit U directly after it misses its turn, and later in the same pass another
    projectile P2, which stands after U, destroys U. U stands before P2, so U's removal counts, together with P2's own
    leaving play: the two objects directly after P2 miss their turns, although U took no turn in
    this tick. An implementation that counted only removals of objects that had taken their turn
    would skip one object there, not two. Scenario O12 tests this case, with an infantry unit's
    death sequence instead of P1.

    The skipped object acts normally from the next tick on, with its delays run on (claim 9).
    Since a missed turn can change what that object does, it can also change the number of draws
    from the scenario random stream (S-0202). The switch is `quirks.update_order_removal_skip`
    (default: the original behaviour). With the switch off, no removal costs a turn: every object
    that is in the update order when the pass comes to it, and has not yet taken a turn in this
    tick, takes its turn (claim 11).
    - Backing: `measurable` for the rule (scenarios O1, O3, O8 and O9), and for its condition on an
      object that missed its turn (scenario O12); for each cause as listed
    - Confidence: high for the rule and its condition; medium for the causes listed

11. **An object can act twice in one tick.** An object that has already taken its turn in tick *n*
    and then leaves play and enters play again during the same turn pass is at the end of the update
    order again and takes a second turn in tick *n*. A unit that boards an open-topped transport
    after its own turn of the tick does exactly this (claim 3), and so would a unit taken inside
    something and put back out by a later object's turn. The same switch
    `quirks.update_order_removal_skip` covers it: with the switch off, no object takes more than one
    turn per tick.
    - Backing: `unmeasured`. A passenger's turn just after boarding can be seen (it can fire from
      inside in the tick it boards), but a recorder cannot tell whether that turn is its second of
      the tick or its first, because it is not known whose turn performs the boarding (Open
      questions).
    - Confidence: medium

12. **Slave-group effects belong to the master's turn (Q-0412 question 1).** A due,
    unsuspended group update happens in the master's turn, not in a separate tick part
    or in one of the slaves' turns. S-0402 claims 21 and 26 already own its timing and
    within-group order; this claim places it against the master's other work.
    - **Building masters.** The group sees the operation-state effects and animation
      advancement already applied in that building turn. Their scenario draws, if any,
      precede the group's replacement and emergence draws. A construction-state change
      reported by that turn's animation advancement is already visible to the group
      decisions of S-0402 claim 27. Paid building repair comes later: a delivery to this
      master can provide the credits for its due repair step in the very same turn,
      even when the returning slave has not yet taken its own turn. That repair cannot
      retroactively affect the group's decisions or earlier operation-state work.
      Operation-state effects are S-0405 claim 9's building effects, not the house-wide
      power recalculation: the latter remains in the later house update (claim 8).
      S-0314 claim 8 owns the repair cost, due ticks and health change.
      - Backing: `measurable` (O14 parts A and C; S-0402 H22 for the construction-state gate)
      - Confidence: high for repair and the draw precedence; medium for the same-turn
        construction-state boundary
    - **Vehicle masters (Q-0430).** A due group update sees the effects of that turn's
      permitted per-tick behaviour start and due behaviour processing (S-0215 claims 2,
      3 and 11). This includes any destination set, replaced or cleared by that work,
      and any group-state change made by directed *harvest* processing (S-0402 claim 22).
      Thus successful directed harvesting has already put the group into travelling
      with its selected destination when the group makes its decisions; it is not still
      field seeking merely because it began the turn in that state. The behaviour's
      scenario draws precede the group's replacement and emergence draws.
      Conversely, that turn's ordinary behaviour processing does not see changes made
      by the later group update, such as its change to working after a deployment
      attempt. The update causes no second ordinary behaviour processing in this turn.
      An order applied at the preceding tick's end (claim 7) can start and process in
      this turn before the group update, subject to S-0215's readiness and processing
      gates; being applied does not guarantee a permitted start.

      The group nevertheless sees the position and destination before the vehicle's
      movement in that turn. Reaching a destination later in the turn does not make it
      destination-free for a group update that already ran. In particular, automatic
      deployment requiring no destination waits for a later eligible group update
      (S-0402 claim 22), not the arrival tick's already-completed update. Conversely a
      destination assigned by the group is available to movement in this turn; ordinary
      movement gates still apply.
      - Backing: `measurable` (O14 part B for movement; O15 for behaviour precedence
        and its draw-order consequence)
      - Confidence: high
    - **Consequences within the group.** Slave-state effects use increasing slave-place
      order, not separate passes for all replacements, all reloads and all deliveries.
      An earlier place's delivery therefore precedes a later place's replacement-creation
      draws. These effects precede the master's field, travelling, deployment and
      relocation decisions; subsequent emergence offers use reverse slave-place order
      where the resulting group state permits them. A replacement or completed reload
      can therefore emerge in this update. A slave received in this update cannot
      complete reload until a later update, even with a zero reload delay. These rules
      are S-0402 claims 21 and 26's, rather than an additional slave behaviour here.
      Delivery credits are available to later objects' turns and to this tick's
      production lines (claim 8). Taking the slave inside is a removal during the
      master's turn: claim 10 counts it only when the slave stood before the master
      in update order, and emergence appends it under claims 3 and 4.
      - Backing: `measurable` (S-0402 H21 for within-group effects; O14 A for delivery
        before repair; O14 D for the removal consequence)
      - Confidence: high for within-group order and delivery placement

13. **A removed object is kept to the end of its tick (Q-2636).** This claim says how long the
    original keeps an object after its removal and what can still reach it.
    - **(a) Lifetime.** An object that is removed in tick *t*, in any part of the tick and by any
      cause, is a leftover (Terms) for the rest of tick *t*. Every leftover stops existing at the
      end of tick *t*, after its last part, and none exists in any part of tick *t* + 1. This covers
      every kind of object: a vehicle destroyed by damage (claim 6 (a)), an infantry unit or
      aircraft when it leaves play after its death sequence or crash, a building when it leaves
      play, a passenger that dies with its transport, a projectile after its detonation and an
      animation that ends. An immediate deletion, as S-0414 claim 15 defines it (a production item
      cancelled with its factory), leaves no leftover. So a vehicle that leaves play at its
      destroying hit in tick *t* with no health (not one that claim 6 (a) keeps in play while it
      sinks, nor one restored to 1 health point before its removal, point (b)) is still among the
      game's objects after the last part of tick *t*, off the map and with no health, and is gone
      after tick *t* + 1.
      - Backing: `measured` (M-0021: a further hit, later in tick *t*, on the last vehicle of a
        house destroyed by a hit in *t*) for such a vehicle being kept within its tick. `measurable`
        (scenario O18 part b) for no leftover from the house update of tick *t* + 1 on. `unmeasured`
        for the parts of tick *t* + 1 before its
        house update: nothing known reaches a leftover there (point (c)), so a leftover kept there
        could not be told apart. Other kinds and other causes are `unmeasured`: no scenario here
        records them, and a leftover differs from a removed object only in the sweep of point (c).
        The immediate deletion is `unmeasured`: S-0414 claim 15 owns it, and a deleted production
        item was never in play, so no sweep could show a leftover of it
      - Confidence: high for the last vehicle of a house, destroyed by a hit and leaving play with
        no health, being kept for a later hit in the same tick; medium for the other vehicles that
        leave play with no health, for its being kept after the last part of its tick, for the end
        coming before any part of tick *t* + 1, for the other kinds, the other causes and the other
        parts of the tick, and for the immediate deletion leaving no leftover
    - **(b) What a leftover is.** A leftover is out of play and off the map. It occupies no cell,
      takes no turn and is in no update order, so it is no object of claims 3, 4 and 10, and its end
      at the end of the tick is no removal for claim 10. It keeps its owner and the health it had
      when it left play: none, if damage destroyed it and nothing restored health before its removal
      (S-0312 claims 10 and 15: a crashable surface vessel is restored to 1 health point and still
      leaves play). It is no target that an object holds or can pick, since S-0309 drops every hold
      on an object at its removal, and its house's counts treat it as S-0414 claim 4 treats an
      object that has left play (a vehicle of a not-scored type, for one, stays counted). So when
      the destruction of a vehicle in the turn pass of tick *t* leaves the counts of its house's
      defeat test at zero, the house fails that test in its house update of the same tick *t* (row
      7).
      - Backing: `measurable` (scenario O18 part a, which records the tick of the house's defeat)
        for the defeat in the same tick. M-0021's hit of point (c) on the last vehicle, in the tick
        of its destruction, agrees with it. The health of none is `measured` (M-0021's rows of a
        vehicle destroyed by a hit). No turn after the removal takes claim 6's backing, the dropped
        holds S-0309's and the counts S-0414 claim 4's
      - Confidence: high for no further turn and for the health of none of a vehicle destroyed by a
        hit that restores no health; medium for the owner and the health kept in other cases, for
        the defeat in the same tick, for no cell, no hold and no pick, which rest on S-0309 and on
        the leftover being off the map
    - **(c) The defeat sweep reaches leftovers.** The only rule known to reach a leftover is the
      destruction of a defeated or surrendering house's objects (S-0414 claim 5). Its sweep offers
      its single hit to each leftover infantry unit, vehicle, aircraft or building that the
      ownership conditions of S-0414 claim 5 attribute to the house, as to any such object of the
      house off the map, and to no leftover projectile or animation, at the leftover's place in
      S-0414 claim 15's creation order. The hit's amount is the leftover's current health, so a
      leftover that left play with no health takes a hit of no amount, with the demolition warhead,
      ignoring defences, with the no-escape marker and with no attacker and no attacking house. It
      changes no health, registers no destruction (no kill and no loss, S-0414 claim 21) and raises
      no tag. For a vehicle, the original still sets off once more the death effects of S-0312 that
      the vehicle's state allows, without the credit (**Changes for other specs**, S-0302 and S-0312
      entries). What it does to the other kinds is not traced (Open questions). Our engine offers
      the sweep's hit only to leftover vehicles with no health. A leftover with health left gets
      none until S-0414 settles that case, and a leftover of another kind gets none until its
      effects are traced (Open questions). In M-0021's part B runs this is believed to be the one
      more hit that the last vehicle of one house takes after its destroying hit: that house has
      nothing left, so it fails its defeat test in the tick of that vehicle's destruction (point
      (b)), while the house's vehicles destroyed in earlier ticks no longer exist and take no hit.
      The switch `quirks.defeat_sweep_leftovers` (Edge cases) turns the sweep's reach to leftover
      vehicles off.
      - Backing: `measured` (M-0021: one more hit in the tick of the last vehicle's destruction,
        after its destroying hit, naming no attacker, with a warhead from none of the test files and
        taking no health) for a vehicle destroyed by a hit in the defeat tick. `measurable`
        (scenario O18 part a) for the demolition warhead, no credit and the repeated death effects,
        part e for no tag raise, and part b for no hit on a vehicle destroyed in the tick before.
        The reach to leftovers of other kinds is `unmeasured`: no scenario here sets one up, and
        their effects are an Open question. That no other rule reaches a leftover is `unmeasured`:
        it rests on the survey of the specs written so far, and a rule found to reach one needs its
        own scenario
      - Confidence: high for the house's last vehicle destroyed by a hit taking, in the same tick,
        one more hit after its destroying hit that takes no health and names no attacker (M-0021).
        Medium for that hit being the defeat sweep's, for the demolition warhead, for no credit and
        no tag raise, for the repeated death effects and for no other rule reaching a leftover. Low,
        as in S-0414 claims 5 and 15, for no attacking house, the ignored defences, the no-escape
        marker and the sweep's place in creation order, and low for leftovers of other kinds. A
        leftover with health left and the other kinds' own death effects are Open questions
    - **(d) Shots of a vehicle in the tick it is destroyed.** A vehicle fires its weapons only in
      its own turn, and a vehicle that has left play takes no further turn (claim 6 (a)). So a
      vehicle that leaves play at its destroying hit in tick *t* fires any shot of tick *t* before
      that hit. A vehicle that stays in play while it sinks (claim 6 (a)) is not covered here. A hit
      of such a shot that comes after the destroying hit, in tick *t* or later, carries neither
      attacker nor attacking house, since its firer has been killed (S-0302 claim 1). A hit of such
      a shot that comes before the destroying hit, such as that of a projectile that enters the
      update order at the shot and detonates later in the same turn pass but before the destroying
      hit (S-0318 claim 5), is not changed by the destruction: it carries what S-0302 claim 1 gives
      it otherwise.
      - Backing: `unmeasured` for the order within the tick: no recording orders a shot and a hit
        within one tick, and the order follows from claim 6 (a). The hit without an attacker is
        `measured` (M-0021: each such shot's hit came in a later tick and named no attacker) for
        hits in a later tick, and S-0302 claim 1 owns it
      - Confidence: medium for the shot coming before the destroying hit; high for its hit in a
        later tick naming no attacker; medium for that hit naming no attacking house, for a hit
        later in tick *t* naming neither and for an earlier hit being unchanged
    - **(e) Our engine.** An implementation need not keep a leftover as an object. Any form
      satisfies this claim that gives the defeat sweep of the same tick the house's leftover
      vehicles with no health (our engine's reach in point (c)), each with its health and its place
      in creation order, and that a recording of the game's objects taken after the tick's last part
      lists, off the map. Nothing else in the simulation may find a leftover.
      - Backing: `unmeasured`: a freedom of our engine, not a behaviour of the original
      - Confidence: high

## Formulas

None. This spec fixes sequences, not quantities.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Turns per object per tick | 1 | turns | Except the quirk cases of claims 10 and 11 |

## State and ordering

**The parts of one tick** (single-player game). The table is a partial order (claim 1): a part comes
after exactly the parts named in its "Comes after" column, and after anything those come after.
Two parts with no such link may run in either order. Row numbers are labels. Rows 1 and 4 keep the
labels of revision 2, and rows 1 to 8 those of revision 5, so that other specs can keep citing them.
Every two parts that take draws from the scenario random stream are linked, so their order is
total (claim 1, draw order). Following the "Comes after" links, the drawing parts stand in the
order 1, 9, 10, 11, 12, 3, 14, 5, 15, 16, 6, 7, 8. Any draws occur in that order,
subject to claim 1's non-drawing, conditional and original-only qualifications.
Row 2 takes no scenario draws; timed bomb damage and its effects belong to the carrier's
turn in row 5 (S-0519 claim 4). Row 13 takes no draws; its links preserve radiation maintenance before the turn pass's hits
and same-tick maintenance of fields made by storm deaths (S-0525 claims 6, 11 and 12).
O13 and O16 test the nonempty in-scope draw allocations listed in claim 1.
Rows 6, 10 and 14 retain the non-drawing or conditional qualifications there; their
position does not itself prove a draw exists. Row 11 and computer-strategy work within
row 7 are deferred original-only context, not requirements on our AI's order interface.

| Row | Part | Comes after | Observable consequence of its place | Claim |
|---|---|---|---|---|
| 1 | Polling of map triggers: the per-tick raise of every polled tag (S-0011, claim 6) | (start of the tick) | A trigger action set off here takes effect before any object takes its turn in this tick; draw order | Claim 1, scenario O6 part A |
| 9 | Growth and spread of resources, growth first (the resource specs own both) | 1 | Draw order | Claim 1 |
| 2 | Planted-bomb maintenance: remove carrierless bombs, maintain ticking sounds and refresh visibility (S-0519 claim 10) | 9 | Sound and due visibility updates precede the turn pass and orders. No timed detonations or scenario draws occur here; carrier-turn detonations and counting removals belong to row 5 | Claim 1; S-0519 scenario T11 |
| 17 | Shared kamikaze re-orders, in enrollment order (S-0506 claim 26) | 2 | Due ammunition, target and queued attack changes are available to same-tick aircraft turns; releases during spawner turns miss this tick's cycle. Assignment takes no scenario draws | Claim 1; S-0506 scenario R21 |
| 10 | Steering of aircraft that dive into their target (the aircraft specs own it) | 17 | Draw order | Claim 1 |
| 11 | Teams, one after another in the order of the list of teams (the specs of teams own both) | 10 | Draw order | Claim 1 |
| 12 | Disc lasers, the most recently fired first; each applies its damage in its own update | 11 | An object a disc laser destroys takes no turn in this tick unless S-0312 keeps it in play (claim 6), and its removal costs no object a turn; draw order | Claim 1 |
| 3 | Psychic domination's update, including its sweep and area damage, then the weather storm's own update and bolt strikes | 12 | The sweep sees the storm owner before cleanup; takeovers affect same-tick turns. An object the sweep or a bolt destroys takes no turn unless claim 6 keeps it in play; removal costs no other turn. Draw order | Claim 1; S-0612 claims 4, 9 and 14, scenarios PD3 and PD5; O10 part B |
| 13 | Radiation maintenance, the most recently created field first (S-0525 claims 5 and 6) | 3 | A storm-death field updates in its creation tick; decay is visible to this tick's hits. Maintenance takes no scenario draws | Claim 1; S-0525 scenarios RF3, RF5 part b and RF10 |
| 14 | The end of electromagnetic pulses whose time is up (the combat and superweapon specs own them) | 3 | Draw order | Claim 1 |
| 4 | The step of a chrono flash in force (S-0201, claim 11) | 1 | A chrono flash started in row 1 steps in the tick of its action; one started in the turn pass first steps in the next tick | Claim 1, scenario O6 part B |
| 5 | Turn pass | 4, 13, 14 | Objects act one after another in update order; their effects are immediate. Timed bombs detonate in carriers' turns, with counting-removal skips under claim 10 (S-0519 claim 4). Radiation hits and their effect draws occur in affected objects' own turns (S-0525 claims 9, 11 and 12); draw order | Claims 1 to 6, 10, 11; O10 part A; S-0525 scenario RF8 |
| 15 | Blast rings; the psychic domination's ring takes no draws (S-0612 claim 8) | 5 | A domination ring first updates in its creation tick; other blast-ring paths keep their draw place | Claim 1; S-0612 claim 8, scenarios PD3 part b and PD9 |
| 16 | Crate respawns, in skirmish games with crates on (S-0520 claim 4) | 15 | Draw order | Claim 1 |
| 6 | Production lines | 16 | A line sees credits delivered in the turn pass of the same tick and precedes this tick's hold orders. Its own update takes no draws | Claim 8, scenario O7 parts A and C |
| 7 | Houses, one after another in house order; final-tick results committed at this part's start (S-0414 claim 16) | 6 | A house's tag raise and defeat test see the turn pass; later houses' defeats are not yet known to an earlier end test. Assigned-tag raises see own surrender and defeat effects; due tech-tree/sidebar work sees tag actions, but completed ordinary power/radar work does not (S-0414 claims 16 and 18). Defeated houses keep raises; cleanup-removed houses have no update. Draw order | Claim 8, scenario O7 part B; S-0414 scenarios V7, V11, V12, V13 and V15 |
| 8 | Orders given in this tick's cycle | 7 | Objects act on the orders from the next tick; a hold order does not stop a progress step of this tick; draw order | Claims 7 and 8, scenarios O2 and O7 part C |

After the last part of the tick, the tick number grows by 1, except in the final tick (S-0201,
claim 1; S-0414 claims 8 and 16). That tick still runs every part, including orders after the
houses; commitment of the result is earlier, at the start of row 7, not after the full tick.
Row 17 takes no scenario draws and does not add a drawing part to claim 1's draw sequence.
Work that other specs own, that this table does not list and that takes no draws from the
scenario random stream has no place here (claim 1).

**An object's place in the update order:**

| State | Event | Result |
|---|---|---|
| Out of play | Enters play | Appended at the end of the update order. If the turn pass is running, it takes a turn later in this pass, unless a removal costs it that turn (claim 4; with the switch off, only if it has not yet taken one in this tick) |
| In play | Leaves play (destroyed and removed, or goes inside) | Taken out, the others keep their order. If the turn pass is running and it is the object taking its turn or stands before that object, whether or not it took its turn in this tick, one more of the objects directly after the one taking its turn misses its turn (claim 10; with the switch off, nobody misses a turn) |
| In play, infantry | Killed, and S-0107 claim 17 removes it at once | Leaves play at the moment of the damage (claim 6) |
| In play, infantry | Killed, and S-0107 claim 17 gives it a death sequence | Stays in play while the sequence plays; leaves play on the tick the sequence completes (claim 6) |
| In play, on the map | Boards an open-topped transport | Taken out (a removal for claim 10) and appended at the end at once. Off the map, in play, keeps taking turns (claim 3) |
| In play, inside an open-topped transport | Gets out | Back on the map, keeps its place in the update order (claim 3) |
| In play, inside an open-topped transport | The transport is destroyed and S-0312 has the passenger die | Leaves play when it dies, one more removal for claim 10 (claim 6) |
| In play, inside an open-topped transport | The transport is destroyed and S-0312 has the passenger get out | Back on the map, keeps its place in the update order, no removal (claim 6) |
| In play, vehicle, aircraft or building | Destroyed | Leaves play at the moment of the damage, a removal for claim 10, except in S-0312's cases (sinking, crashing, exploding or sold buildings), where it stays in play and leaves later (claim 6) |
| Removed in this tick (a leftover) | The tick ends | Stops existing; it never exists in the next tick (claim 13) |
| In play | Nothing | Keeps its place |

## Edge cases and quirks

- **Removal skip** (claim 10), `quirks.update_order_removal_skip`, default: the original behaviour.
  On (the original): each removal, during the turn pass, of the object taking its turn or of any
  object before it in the update order, whether or not the removed object took its turn in this
  tick, makes one more of the objects directly after the object taking its turn miss its turn.
  Off: no removal costs any object a turn. Every object that is in the update order when the pass
  comes to it, and has not yet taken a turn in this tick, takes its turn, whatever left play
  before it.
- **Second turn** (claim 11), same switch. Off: at most one turn per object per tick. This also
  covers a unit that boards an open-topped transport after its own turn.
- **Defeat sweep on leftovers** (claim 13), `quirks.defeat_sweep_leftovers`, default: on. On, as in
  the original for this case: the sweep of S-0414 claim 5 offers its hit to the house's leftover
  vehicles with no health too, and each has the death effects of the S-0312 entry in **Changes for
  other specs** once more, with their draws. Leftovers with health left and leftovers of other kinds
  are offered no hit (claim 13 (c)). For leftovers with health left this departs from the original
  (the S-0414 entry in **Changes for other specs**). For leftovers of other kinds what the original
  does is left open (Open questions). Off: the sweep offers no hit to any leftover. A recording then
  shows no hit on the leftover after its destroying hit.
- No departure: nothing in a tick's order depends on the host or on real time.

## Import mapping

The order of a tick uses the following inputs from the user's rules data, apart from the map's
own order of entries (claim 3), whose format S-0009 owns. The slave fields select the group
whose timing S-0402 owns; they do not change claim 12's precedence:

| Key | Section | Form | Absent | Feeds |
|---|---|---|---|---|
| `SpawnsTiberium` | a terrain type's section | boolean | no | the spawns-resources flag of claim 2: a terrain object of a type with the flag on takes turns; one with it off takes none. What the flag does for resources belongs to the economy specs |
| `Enslaves` | a vehicle or building type's section | infantry type name | none | the slave type for a master's group; S-0402 claim 19 owns creation, claim 12 here its turn placement |
| `SlavesNumber` | the master's type section | signed integer | 0 | initial slave-place count; S-0402 claim 19 |
| `SlaveRegenRate`, `SlaveReloadRate` | the master's type section | signed integers, ticks | 0 each | replacement and reload delays; S-0402 claim 21 owns due group updates and claim 12 here their precedence |
| No data key | — | engine behaviour | fixed | claim 12's vehicle behaviour-start and processing precedence over a due group update; S-0402 claim 22 owns directed harvesting and its imported flags, and S-0215 claims 6 and 11 own processing delays and start gates |

## Measurement scenarios

**Within-turn falling and landing (claim 4).** S-0511 scenario B3's turn-order variant
supplies the setup, procedure, observables and falsifiers: due behaviour remains possible
during descent, ordinary movement and firing do not, and a protected survivor's landing
is visible to those later checks in the same turn. This tests local ordering within a
turn, not a shared falling phase.

**Within-turn superweapon warp (claim 4).** S-0513 scenario SW23 supplies the setup,
procedure, observables and falsifier for the leaving-stage exclusion and stopped driving,
with vehicles, infantry and landed aircraft recorded separately. Departure, placement,
arrival, hold-expiry continuation and the number of warp updates within a turn are
exploratory original-side observations in parts a to e, not promised oracle outcomes.
Part f separately tests S-0513 claim 51's engine contract: one warp update per tick,
immediate route clearing, the whole-ordinary-turn exclusion through arrival effects,
preserved intrinsic warp effects and elapsed clocks, and ordinary eligibility after the
hold-end test without catch-up updates. This includes the additional operations in claim 4,
not only healing, behaviour, movement, firing and turret turning. A contrary event falsifies the engine contract,
not claim 48's explicitly unresolved original-side continuation. For an unlimited mind controller in the warp (a named test type whose remaining overload countdown is smaller than the warp's ticks, so that a countdown counted in ticks would reach zero during the warp), record its overload countdown and each pulse check per tick from the first pending update through the first ordinary update after the hold-end test. Expected: the countdown and pulse timing do not change during the warp, and countdown steps and pulse checks resume with the first ordinary update after the hold-end test, with no step made up later. Falsified if a countdown step or pulse check occurs before the hold-end test, or a step is made up later.

**Shared kamikaze boundary (claim 1, row 17).** S-0506 scenario R21 supplies the setup,
procedure, sub-tick observables and falsifiers for same-tick aircraft access to the assigned
orders, exclusion of turn-pass releases and zero scenario draws during assignment.
A timer-driven planted-bomb release belongs on the excluded, turn-pass side of this
comparison, not before the cycle. A pre-cycle release fixture would need a distinct
cause before row 17, not the timed-bomb countdown. End-of-tick samples alone do not
establish the boundary; an unresolved sub-tick comparison is inconclusive.

**Designator target-change boundary (claim 7).** S-0519 scenario T19 supplies the
skirmish setup, scripted ordinary input and read-only recording of aircraft ammunition,
targets, destinations and queued behaviour. Its move-order test expects cancellation in
the order's application tick, after the turn pass here; its later-turn target-death test
expects cancellation in the killing tick even though the designator has already acted.
Record the event boundary and cancellation effects, not merely the next aircraft movement.
Deferral to the next designator turn falsifies this boundary. The same-target control must
not cancel; a recorded A-to-B-to-A sequence must not restore the old attack state. Coalesced
orders or an unresolved event boundary make the corresponding comparison inconclusive.
Run counts and settings remain those of T19.

**Common setup.** All scenarios run under the glossary's *measurement setup*, as skirmish games on
purpose-made maps, at game speed index 6 unless the scenario states another index, with no engine add-on and no mods (the named test files of a scenario excepted).
The slowest speed makes manual orders easiest. The ordering rules do not depend on speed.
Sequence durations can depend on it under the glossary's speed-normalised-delay rules, so the
scenarios' timing expectations use the stated index.
- *Recorder.* At the end of every tick the *recorder* reads, without changing anything: the tick
  number; for the named objects their position, health and presence and the tick in which a
  weapon's delay between shots starts; a production line's progress and whether it is on hold; the
  house's credits; the list of objects in play; and which armed objects in play fire in the tick.
  "Fires in tick *t*" means that the recorder sees the firer's delay between shots start in tick
  *t*.
- *Test quantities.* Numeric rules overrides below are invented fixture values, not stock
  values. Health and weapon damage are integer hit-point quantities; damage denotes the
  weapon's base damage before the owning combat spec's modifiers. Tick numbers and tick
  durations are integers. Fixture arithmetic is exact integer arithmetic unless an owning
  spec states a conversion; it introduces no additional rounding. Percentages are
  dimensionless ratios. Game-speed indices, infantry-death kinds, row and stage labels,
  counter and variable identifiers are dimensionless selectors, not durations or amounts.
  Run and attempt counts count complete trials.
- *Test data.* Named test data is a test map's override sections only, unless a scenario
  explicitly names additional temporary test files. Test weapons used to time
  turns have a projectile with no visible flight, unless a scenario says otherwise, no firing
  animation, and a warhead that spawns no animation, particle or other object. Test types create no
  objects when damaged or destroyed (no smoke, explosions, debris or survivors), apart from the
  objects a scenario names. This does not change how the damaged or destroyed object itself
  behaves. A pilot run of each scenario checks that no object enters play when a test weapon fires
  or hits, or when a test object is destroyed, apart from the test weapon's projectile and the
  objects the scenario names.
- *Computer player.* Use the glossary's bystander computer player: the farthest start,
  separated from the test area. Starting units and credits are game-wide settings, not a
  separate minimum for this player; a scenario that grants ample credits grants them to
  the bystander too. Its original-only extras remain possible. The void rule, not assumed
  inactivity, excludes interference.
- *Void rule.* Each scenario names its counted ticks and the events it expects in them: its
  projectiles entering and leaving play, the animations it names, and the removals its expected
  result lists. A counted tick is void, and is not used, when anything else enters or leaves play in
  it, as the list of objects in play shows, or when any armed object other than the scenario's
  firers fires in it. An object that enters and leaves play within one tick does not show in the
  end-of-tick list, which is why unexpected firing is checked separately. The rule also voids a
  counted tick in which a named unit boards a transport or gets out of one, and ticks in which the
  computer player's own units enter or leave play.

**Scenario O1: mutual kill and the projectile stagger (claims 4, 5, 6, 10).**
- *Setup:* a flat test map. Its override sections define a test vehicle type with a health field
  of 10 hit points, armed with a test weapon with damage 20 hit points at 100 % against the vehicle's armour class, a
  range of 5 cells and a long delay between shots. Two vehicles of that type are pre-placed 3 cells
  apart, facing each other, one owned by the human player and one by the computer player
  (enemies), the human's vehicle first in the map's vehicle entries. The vehicle type is not naval
  and has no crash flag, so neither sinks nor crashes (S-0312 claims 10 and 15). Variant B: the
  computer's vehicle is replaced by a test defensive building of the computer player, with the same
  weapon and health, of a type that is not exploding (S-0312 claim 21).
- *Procedure:* start, and give no orders. Repeat with the two entries swapped in the map (in
  variant B the building is always later, by claim 3, so there is nothing to swap).
- *Observable:* the tick on which each object fires, and the tick on which each disappears.
- *Runs:* runs are repeated until at least 5 runs per entry order, and 5 of variant B, count, up to
  50 of each. Identical pre-placed objects are not certain to fire in the same tick, so runs in
  which the two do not fire in the same tick are recorded but do not count. If fewer than 5 of 50
  count, the scenario is inconclusive for that entry order. Scenarios O3 and O8 still cover the
  stagger and claim 6(a), but the mutual kill of claim 5(a) (and, for variant B, claim 6(b)) then
  stays unconfirmed, and claim 6(d) stays unconfirmed if every part is inconclusive. Counted
  ticks: the shared firing tick *n* and tick *n* + 1.
- *Expected:* in every run in which both fire in tick *n*, the object later in the update order
  disappears in tick *n* and the earlier one in tick *n* + 1; with the entries swapped, the roles
  swap. In such a run neither object survives. In variant B the building disappears in tick *n* and
  the vehicle in tick *n* + 1. A firer that fires in an earlier tick than the other destroys it
  before it fires, and its victim never fires.
- *Falsified if:* in a counted run both disappear in the same tick, or the earlier entry disappears
  first, or the roles do not swap with the entries, or one survives. A result in which both
  disappear in tick *n* falsifies the stagger of claims 5 and 10 but not that both shots land. In
  variant B, a building that disappears after tick *n* although its health ran out in tick *n*
  falsifies claim 6 for buildings (it plays a destruction sequence first).

**Scenario O2: order latency (claim 7).**
- *Setup:* a flat test map with one human vehicle standing on a straight road, already facing
  along it, and the computer player far away.
- *Procedure:* give the vehicle a move order to a cell 8 cells down the road.
- *Observable:* per tick, the vehicle's destination and its position.
- *Runs:* 10.
- *Expected:* on the first tick at whose end the recorder sees the new destination, the position is
  still the old one; the position first changes on a later tick.
- *Falsified if:* in any run the position has changed by the end of the tick on which the new
  destination is first seen. The scenario is falsifiable in that direction only. A vehicle that
  needs more than one turn to start moving also passes it.

**Scenario O3: the update order by kind and entry, seen through the stagger (claims 3, 4, 5,
10).**
- *Setup:* a flat test map. Two objects E and L, enemies, stand in range of each other and fire at
  each other for the whole run. Each is armed with a test weapon with damage 1 hit point, far below the other's
  health, so that nobody dies. E's delay between shots is 10 ticks and L's is 11 ticks, so their
  shots fall in the same tick about once every 110 ticks. No other armed object is on the map.
  Variants, with E expected earlier in the update order than L:
  - (a) E a human test vehicle, L a computer test infantry unit.
  - (b) E a human test infantry unit, L a computer test defensive building.
  - (c) E a human test vehicle, L a computer test defensive building.
  - (d) both test vehicles, E the earlier of the two in the map's vehicle entries; and (d′) the
    same map with the two entries swapped, which makes the other vehicle E.
  - (e) as (d), but E's test weapon has a projectile with a visible flight at a constant speed, no
    trail and no spawned objects. Its flight from E to L takes *f* ticks, with 1 ≤ *f* ≤ 5 (the
    pilot run measures *f* and checks that it is the same for every shot). L's weapon is unchanged.
- *Procedure:* start, and give no orders. Run each variant for 1000 ticks.
- *Observable:* the ticks on which E and L fire, and the ticks on which each one's health drops.
- *Runs:* 2 per variant. Counted ticks: every tick in which E or L fires, and the tick after it.
  In variant e also count every flight tick through each projectile's detonation and the
  following tick, including shots still in flight when the 1000-tick firing interval ends.
- *Expected, variants a to d′:* in a tick in which only one of them fires, its shot lands in the same
  tick. In a tick *n* in which both fire, L is hit in tick *n* and E in tick *n* + 1.
- *Expected, variant e:* E's shot fired in tick *t* hits L in tick *t* + *f*. L's shot fired in a
  tick *u* hits E in tick *u* + 1 when one of E's projectiles hits L in tick *u*, since that
  projectile detonates and leaves play in its own turn just before the turn of L's new projectile.
  In every other tick L's shot hits E in tick *u*, also when E fires in tick *u*.
- *Falsified if:* in variants a to d′, in some tick in which both fire, E is hit in tick *n* and L
  in tick *n* + 1 (the kind or entry order is the other way round), or both are hit in tick *n* (no
  stagger), or a lone shot lands one tick late. In variant e, L's shot hits E in tick *u* in a tick
  *u* in which E's projectile hits L, or hits it one tick late in any other tick.

**Scenario O4: delays run on the tick number (claim 9).**
- *Setup:* override sections give a test infantry type a test weapon with a delay between shots of
  300 ticks. A human transport, not open-topped, stands next to it, and an enemy target stays in
  range of the unload cell.
- *Procedure:* let the infantry fire once, then load it into the transport at once, wait 100 ticks,
  and unload it.
- *Observable:* the tick of the first shot and of the second.
- *Runs:* 3.
- *Expected:* the second shot is 300 ticks after the first (or the first tick after unloading at
  which it can fire, whichever is later).
- *Falsified if:* the second shot comes about 400 ticks after the first, or later, while the target
  stayed in range.

**Scenario O5: passengers of an open-topped transport take turns (claims 2, 3).**
- *Setup:* a flat test map with a human transport whose type is open-topped (a named test type in
  the map's override sections if stock data has none), one human infantry unit standing next to it,
  and an enemy vehicle placed so that it is in the infantry's weapon range from the transport but
  not from the infantry's starting cell. A control run uses a transport type that is not
  open-topped.
- *Procedure:* order the infantry unit into the transport, then give no orders.
- *Observable:* the tick on which the infantry unit leaves the map, whether the enemy vehicle takes
  damage from the infantry's weapon while the infantry is inside, and the ticks of those shots.
- *Runs:* 3 test and 3 control runs.
- *Expected:* in the test runs the passenger fires from inside, at its own weapon's delay between
  shots; in the control runs it never fires while inside.
- *Falsified if:* the passenger of the open-topped transport never fires while inside, or, in a
  control run, the passenger fires while inside.

**Scenario O6: parts before the turn pass (claim 1).**
- *Part A, row 1 before the turn pass.*
  - *Setup:* a flat test map. A human test vehicle S, of a type that is not naval and has no crash
    flag (claim 6), is armed with a test weapon with damage 1 hit point and a
    delay between shots of *R* ticks. It faces a computer test target Z (an unarmed vehicle with high
    health) in range. In the test run S carries a tag whose one-time trigger has the single event
    elapsed time *v* (an integer count of trigger time units, each 15 ticks) and the action that destroys the objects carrying the tag (S-0011, claims 13
    and 27), so the action runs in tick 15 × *v*.
  - *Procedure:* a first control run without the trigger records the tick *t*₀ of S's first shot.
    Then *v* is chosen with 15 × *v* > *t*₀, and *R* = 15 × *v* − *t*₀. A second control run, with
    this *R* and still without the trigger, must show S's second shot in exactly tick 15 × *v*; if
    it does not, *R* is adjusted until it does. Then the test run starts, with the trigger and the
    same *R*, and no orders are given.
  - *Observable:* the ticks on which S fires, the ticks on which Z's health drops, and the tick on
    which S disappears.
  - *Runs:* 3 test runs. Counted ticks: tick 15 × *v* and the tick after it.
  - *Expected:* S disappears in tick 15 × *v*. Z's health drops in tick *t*₀ and not in tick
    15 × *v* or later.
  - *Falsified if:* Z's health drops in tick 15 × *v* or 15 × *v* + 1, because S fired before the
    action destroyed it. A test run is void, and the setup is checked, if S's first shot is not in
    tick *t*₀ or S disappears in another tick than 15 × *v*.
- *Part B, row 4 between row 1 and the turn pass.*
  - *Setup:* as S-0201, scenario T6, part B: the chrono-flash action with parameter *D* = 30
    (a dimensionless integer count, converted to steps by S-0201 claim 11). Run 1
    sets it off with a one-time trigger whose only event is elapsed time 20 trigger time units
    (15 ticks per unit), so its action runs in
    tick 300. Run 2 sets it off with a one-time trigger whose only event is entered-by, on a cell tag
    (S-0011, claim 18). That trigger's actions are, in this order, a count action on its own counter
    (the S-0011 common setup's counter) and the chrono-flash action (S-0011, claim 11, for the order
    of a trigger's actions).
  - *Procedure:* in run 2 a human vehicle is ordered into the tagged cell.
  - *Observable:* the value at which the tick number stands still during the chrono flash's freeze,
    and in run 2 also the tick *a* of the counter drop, which is the tick in which the trigger fired.
  - *Runs:* 3 of each.
  - *Expected:* run 1 stands at 374 (first step in tick 300, the tick of the action). Run 2 stands at
    *a* + 75 (first step in tick *a* + 1).
  - *Falsified if:* run 1 stands at 375, which means row 4 comes before row 1, or run 2 stands at
    *a* + 74, which means row 4 comes after the turn pass.

**Scenario O7: production lines and houses after the turn pass (claims 7, 8).**
- *Part A, production lines after the turn pass.*
  - *Setup:* a test map with a human house that starts with 0 credits and owns a refinery, a
    vehicle-producing building and a harvester working a nearby resource field. Override sections
    set the game-wide build-speed field, a test vehicle type's price and the resource's value, as
    follows. The build is short enough that, while money lasts, the line's progress advances on
    every tick. The price is high enough that one full unloading of the harvester pays for some
    progress steps but not for the whole vehicle. Each single credit increment of an unloading pays
    for at least two progress steps, so that the credits still visibly rise in a tick in which one
    step is paid from them. A pilot run checks all three.
  - *Procedure:* order one test vehicle at the start and let the harvester work.
  - *Observable:* per tick, the house's credits and the line's progress.
  - *Runs:* 3, each covering at least 5 unloadings. Counted ticks: the tick *d* of the first credit
    rise after each stretch without progress, and the tick after it.
  - *Expected:* after a stretch in which progress has stood still for lack of money, progress
    advances again in the tick *d* of the first credit rise after the stretch.
  - *Falsified if:* after such a stretch progress first advances in tick *d* + 1.
- *Part B, houses after the turn pass.*
  - *Setup:* the S-0011 common setup, with a human vehicle and a cell tag on cell *e*. The cell tag's
    one-time trigger E has the event entered-by and two actions: a count action on counter 1, and an
    action that sets map trigger variable 1. A one-time trigger H, owned by the human player's
    country (so that its tag is assigned to the human house, S-0011 claim 35), has the two events
    any-event and variable 1 set, and a count action on counter 2.
  - *Procedure:* order the vehicle into cell *e*. Orders given by hand cannot be timed to one tick,
    so runs are repeated until at least two runs have *a*, the tick of counter 1's drop, a multiple
    of 8, and at least two runs have *a* not a multiple of 8, up to 100 attempts. If either
    count is below two, that comparison is inconclusive.
  - *Observable:* the ticks of the drops of counter 1 (*a*) and counter 2.
  - *Expected:* when *a* is a multiple of 8, counter 2 drops in tick *a*, because the human house's
    raise comes after the turn pass. Otherwise it drops in tick *a* + 1, when the polling of the
    next tick (row 1) raises H's tag, which is polled because its chain holds the any-event code
    (S-0011, claim 17). Polling comes only at the start of a tick, so in such a run nothing else
    raises H's tag between E's firing and the end of tick *a*.
  - *Falsified if:* in a run with *a* a multiple of 8 counter 2 drops in tick *a* + 1, or in any
    other run it drops in tick *a*.
- *Part C, production lines before the orders.*
  - *Setup:* as part A, but the house starts with ample credits and has no harvester, so that money
    never runs out. The pilot run checks that the line's progress advances on every tick.
  - *Procedure:* order one test vehicle. After at least 10 ticks of progress, and well before the
    vehicle is done, put the line on hold by hand.
  - *Observable:* per tick, the line's progress and whether it is on hold.
  - *Runs:* 5. Counted ticks: the first tick *h* at whose end the line is on hold.
  - *Expected:* progress at the end of tick *h* is higher than at the end of tick *h* − 1, and it
    does not change after tick *h*.
  - *Falsified if:* progress at the end of tick *h* equals that at the end of tick *h* − 1 in any
    run, which means the hold took effect before the line's step of tick *h*.

**Scenario O8: several removals in one turn, open-topped passengers and infantry deaths (claims
5, 6, 10).**
- *Setup:* a flat test map.
  - *Firers.* Computer test vehicles F1, F2, … stand in this order in the map's vehicle entries.
    Each is armed with the same test weapon: its damage is 1 hit point, and its delay between shots is 30
    ticks. Its warhead does 0 % against the armour class of the test infantry type below, so no
    firer ever chooses an infantry unit as its target. In variants d and e, F1 carries only a second
    test weapon, described below, in place of the common one; F2 and F3 keep the common weapon.
  - *Targets.* Each firer Fi faces its own human target Wi in range and has no other target in
    range. The targets are unarmed test vehicles with high health, except W1 as the variants say.
    All firers fire on their first chance after the start, so they fire together every 30 ticks
    after that, in volleys.
  - *Variants a to c and f.* In all four, W1's health is 10 hit points, so the tenth volley destroys it. No
    test vehicle type here is naval, so none sinks (S-0312 claim 10). (a) Three firers; W1 a test
    vehicle with no crash flag. (b) Five firers; W1 a test transport that is not open-topped, whose
    type has the crash flag (S-0107 Terms). It stands on the ground, so it cannot crash, and by
    S-0312 claim 12 its passengers stay inside and die when it leaves play. (c) As (b), but W1's
    type is open-topped. (f) As (c), but W1's type has no crash flag, so by S-0312 claim 12 its
    passengers get out when it is destroyed. In variants b, c and f, two unarmed human test
    infantry units stand next to W1.
  - *Variants d and e.* Three firers. W1 is a human test infantry unit of a type that is not marked
    "not human" (as in S-0107), is not a cyborg or jumpjet type, has no death animations of its own,
    and has image frames in the first infantry death-sequence entry (S-0107, claim 17, which turns a death request down
    when the entry has none). W1 stands on the ground, on a land cell that is not water. F1's second
    test weapon does 100 % against W1's armour class, with damage above W1's health, so the first
    volley destroys W1. Its warhead's infantry-death
    kind is one for which S-0107, claim 17, removes the unit at once in variant d (row 8 of that
    claim's table), and kind 1, which gives a death sequence, in variant e. In variant d the death
    animations that S-0107 names for that kind are named objects. The pilot run checks that in
    variant e W1 is still present at the end of tick *n* + 3.
- *Procedure:* in variants b, c and f, right after the start, order the two infantry units into W1.
  They must be inside well before the tenth volley, which the recorder checks. Otherwise give no
  orders.
- *Observable:* for every volley, fired in tick *n*, the tick on which each Wi's health drops or Wi
  disappears.
- *Runs:* 3 per variant. Runs in which the firers do not all fire in the same ticks do not count.
  Counted ticks: ticks *n* to *n* + 3 of each volley. In variants d and e only the first volley
  counts.
- *Expected:* the ticks below, counted from the volley's tick *n*. They follow from claim 10 applied
  to the projectiles in firing order, including the skips that later detonations cause in the
  following ticks. A non-lethal volley is any volley before the tenth. The lethal volley is the one
  that destroys W1: the tenth in variants a to c and f, which in variants b and c also kills W1's
  two passengers and in variant f lets them get out, and the first in variants d and e. In variant
  e, W1's health reaches 0 hit points in tick *n*. In variant f the recorder checks that both passengers are
  present on the map at the end of tick *n*; a run in which either is not does not count.

  | Variant and volley | W1 | W2 | W3 | W4 | W5 |
  |---|---|---|---|---|---|
  | (a), non-lethal | *n* | *n* + 1 | *n* | — | — |
  | (a), lethal | *n* | *n* + 1 | *n* + 2 | — | — |
  | (b), (c) or (f), non-lethal | *n* | *n* + 1 | *n* | *n* + 2 | *n* |
  | (b) or (f), lethal | *n* | *n* + 1 | *n* + 2 | *n* | *n* + 1 |
  | (c), lethal | *n* | *n* + 1 | *n* + 2 | *n* + 1 | *n* + 3 |
  | (d), lethal | *n* | *n* + 1 | *n* + 2 | — | — |
  | (e), lethal | *n* | *n* + 1 | *n* | — | — |

- *Falsified if:* a counted volley differs from its row. In particular, the lethal volley of variant
  a equal to the non-lethal one falsifies the count of *k* removals, and the lethal volley of
  variant c equal to that of variant b falsifies that the passengers of an open-topped transport
  that die with it leave the update order then (claim 6). The lethal volley of variant f equal to
  that of variant c falsifies that passengers of an open-topped transport that get out keep their
  place in the update order (claim 6). A lethal
  volley of variant d equal to the row of variant e falsifies that an infantry unit removed at once
  leaves play at the moment of the damage. A lethal volley of variant e equal to the row of variant
  d falsifies that an infantry unit with a death sequence stays in play. If all targets are hit in
  tick *n*, the skip of claim 10 is falsified.

**Scenario O9: an infantry unit's death sequence ends (claims 6, 10).**
- *Setup:* a flat test map. W is a human test infantry unit as in scenario O8, variant d, and the
  only infantry unit on the map. B is a human test defensive building, the first building in the
  map's building entries, so by claim 3 it comes directly after W in the update order. B is armed
  with a test weapon with damage 1 hit point and a delay between shots of 2 ticks, and faces a computer
  target T (an unarmed test vehicle with high health) in range. K is a computer test vehicle armed
  with a test weapon of infantry-death kind 1, damage above W's health and a long delay between
  shots. W is in K's range. B and T are not. K is not in B's range.
- *Procedure:* start, and give no orders. K kills W with its first shot, and W plays its death
  sequence.
- *Observable:* the ticks on which B fires, and the tick *c* on which W disappears. W's corpse
  animation is a named object.
- *Runs:* 10. A pilot run without K checks that B fires in every second tick. A run counts when B's
  last shot before tick *c* was in tick *c* − 2, so that B's weapon is ready in tick *c*. About half
  the runs count. Counted ticks: tick *c* and the tick after it.
- *Expected:* in every counted run B does not fire in tick *c* and fires in tick *c* + 1, because
  W leaves play in its own turn and B, directly after it, misses its turn. B fires in every second
  tick everywhere else.
- *Falsified if:* in a counted run B fires in tick *c*. That falsifies that W leaves play in its own
  turn at the end of the sequence (claim 6). It does not tell whether W left play earlier or left it
  in another object's turn.
- *Variant T (claim 3, kinds as blocks).* As above, but the map also holds one terrain object of a
  test terrain type that spawns resources, so that it takes turns (claim 2), placed far from every
  other test object. Its entry is the only terrain entry. 10 runs. The variant counts only if the
  base scenario passed. *Expected:* as the base scenario: B does not fire in tick *c* and fires in
  tick *c* + 1. *Falsified if:* in a counted run of the variant B fires in tick *c*, while it never
  does in the base scenario. That means the terrain object stands between W and B, so that W's
  removal costs the terrain object its turn instead of B.

**Scenario O10: carrier-turn planted bombs versus pre-turn storm bolts (claim 1, rows 5 and
3).**
- *Common part:* a flat test map with no aircraft and no terrain entries.
  - *V,* the destroyed object: a computer test vehicle, the last entry in the map's vehicle
    entries, of a type that is not naval and has no crash flag (claim 6), armed with a test weapon with damage 1 hit point and a delay between shots of 2 ticks, which
    does 0 % against the armour class of the test infantry type below. V faces a human target T (an
    unarmed test vehicle with high health) in range, and has no other target.
  - *Y,* the observer: a human test infantry unit, the first entry in the map's infantry entries.
    Since V is the last vehicle and there are no aircraft, claim 3 puts Y directly after V in the
    update order. Y is armed with a test weapon with damage 1 hit point and a delay between shots of 2 ticks,
    which does 100 % against the armour class of its target T2 and 0 % against that of V, which
    differs from T2's. T2 is a computer unarmed test vehicle with high health in Y's range. Y has
    no other target, and V is not in Y's range.
  - T and T2 come before V in the map's vehicle entries. T, T2 and Y stand far enough from V that
    neither the bomb of part A nor the bolts of part B can reach them, so T's health drops only
    from V's shots, and Y stays in play.
  - A pilot run checks that V and Y each fire in every second tick. *b* is the tick in which V
    disappears. A run is *V-counted* when V's last shot before tick *b* was in tick *b* − 2, as the
    recorder sees it, so that V's weapon is ready in tick *b*. It is *Y-counted* when Y's last shot
    before tick *b* was in tick *b* − 2, so that Y's weapon is ready in tick *b*. A run can be both.
    Counted ticks: tick *b* and the tick after it.
- *Part A, planted bomb.*
  - *Setup:* the common part, plus a human test infantry unit X, the second entry in the map's
    infantry entries, whose test weapon plants a bomb whose detonation destroys V. X has no target
    in range other than V. After planting, X is ordered away, out of the bomb's reach, and the
    recorder checks that X is present at the end of tick *b*. By claim 3, V (a vehicle) comes before X (an infantry unit) in the update
    order. The bomb's explosion animation is a named object.
  - *Procedure:* order X to plant its bomb on V and then to move out of the bomb's reach, then give
    no orders.
  - *Observable:* per tick, T's health, V's presence and the ticks on which Y fires.
  - *Runs:* repeated until at least 5 runs are V-counted and at least 5 are Y-counted, up to 40.
    If either count stays below 5, that half of the scenario is inconclusive.
  - *Expected:* with claim 10's switch on, every Y-counted run has no Y shot in tick *b*:
    V's removal during its own turn costs Y its turn. Y resumes in tick *b* + 1. With the
    switch off, Y fires in tick *b*. This observer result, not carrier silence alone,
    distinguishes carrier-turn detonation from a shared pre-turn pass.
    For a V-counted run using ordinary attack firing, V dies before that firing opportunity
    and T takes no new hit in ticks *b* or *b* + 1. An opportunity-firing variant may instead
    shoot before detonation (S-0519 claim 4); record the shot without treating it as a
    different detonation phase. Exclude runs with other counting removals or a skipped
    carrier turn before the due check.
  - *Falsified if:* with the switch on and these controls, Y fires in tick *b*. A pre-turn
    removal costs Y no turn. A detonation in X's later turn also cannot undo Y's turn.
    An ordinary attack shot by V before detonation falsifies that local firing expectation,
    but an opportunity shot does not falsify carrier-turn detonation.
- *Part B, storm bolt.*
  - *Setup:* the common part, with V's health field set to 1 hit point. Superweapons are allowed in the game
    settings. The human player owns a ready, powered weather superweapon building, with test data
    that keeps the storm's area small enough that no bolt can reach T, T2, Y or the building. The
    storm's bolt animations and any cloud animations are named objects.
  - *Procedure:* fire the storm at V's cell, then give no orders.
  - *Observable:* per tick, T's health, V's presence and the ticks on which Y fires.
  - *Runs:* as part A.
  - *Expected:* in every V-counted run, V fires no shot in tick *b* and T takes no new hit
    in ticks *b* or *b* + 1. In every Y-counted run, Y fires in tick *b* and continues its
    every-second-tick cadence: V left play before the turn pass, so costs Y no turn.
  - *Falsified if:* V fires in tick *b*, or Y misses its due shot in tick *b* with no other
    counting removal. A bolt whose damage came in its own animation's later turn could
    allow V to fire; a strike removing V in V's own turn would instead cost Y its turn.

**Scenario O11: boarding an open-topped transport and getting out (claim 3).**
- *Setup:* a flat test map, with its override sections. T is a computer test vehicle, unarmed, with
  health high enough to survive the whole run, and the only computer object near the test objects.
  The human player has these map-placed objects:
  - P, a test infantry unit, the first infantry entry, armed with a test weapon with damage 1 hit point and a
    delay between shots of 10 ticks.
  - Q, a test infantry unit, the second infantry entry, armed with a test weapon with damage 4 hit points and a
    delay between shots of 13 ticks.
  - R, a test defensive building, the only building entry, armed with a test weapon with damage 2 hit points
    and a delay between shots of 11 ticks.
  - O, an unarmed test transport whose type is open-topped, and C, an unarmed test transport whose
    type is not open-topped, both standing next to P and Q.

  T stands in range of P, Q and R, of P inside O and of the cells where P and Q get out, and no
  other armed object has it in range. All three weapons are 100 % against T's armour class. Since
  every shot hits T and the three damages are 1, 2 and 4 hit points, each drop of T's health in a tick splits
  in only one way into the shots that landed in that tick.
- *Procedure:* right after the start, order Q into C. Then three stages, each at least 1500 ticks
  long:
  - stage 1, from the tick in which Q is inside C: P stands on the map.
  - stage 2: order P into O. P fires at T from inside.
  - stage 3: unload Q from C, and then, once Q has fired, unload P from O.
- *Observable:* per tick, the ticks on which P, Q and R fire, T's health and the list of objects in
  play.
- *Runs:* 2. Counted ticks: in each stage, every tick in which two or more of P, Q and R fire, and
  the tick after it, from the first tick after the stage's last boarding or getting out.
- *Expected:* by claim 3, the update order is P, R in stage 1 (infantry before buildings); R, P in
  stage 2 (P moved to the end when it boarded); and R, P, Q in stage 3 (Q appended when it got out
  of C, and P kept its place when it got out of O). In a counted tick *n*, by claim 10 applied to
  the projectiles in firing order, the shots of the firers in that order land as follows:

  | Stage | Firers in tick *n* | Lands in *n* | Lands in *n* + 1 |
  |---|---|---|---|
  | 1 | P and R | P's shot | R's shot |
  | 2 | P and R | R's shot | P's shot |
  | 3 | R and P | R's shot | P's shot |
  | 3 | P and Q | P's shot | Q's shot |
  | 3 | R and Q | R's shot | Q's shot |
  | 3 | R, P and Q | R's and Q's shots | P's shot |

- *Falsified if:* stage 2 shows the stage 1 row, which means that boarding did not move P to the
  end of the update order. (A stage 1 that differs from its row falsifies the kind order of claim
  3, as scenario O3, variant b, would, and makes the later stages inconclusive.) Or stage 3 shows
  Q's shot landing in *n* and P's in *n* + 1 when P and Q fire together, which means that getting
  out moved P to the end, as does a triple tick in which R's and P's shots land in *n* and Q's in
  *n* + 1 (the order R, Q, P). Or both shots of a pair land in *n* (no stagger).

**Scenario O12: a removal counts although the removed object missed its turn (claim 10).**
- *Setup:* a flat test map with no aircraft entries.
  - *Infantry.* W and U are human test infantry units of the type of scenario O8, variant d, and
    the map's only infantry entries: W the first and U the second, so by claim 3 U comes directly
    after W in the update order. U's health is *h* hit points. W and U stand apart.
  - *K,* a computer test vehicle, is armed as in scenario O9: a test weapon of infantry-death kind
    1, damage above W's health and a long delay between shots. W is in K's range, and nothing else
    that K's weapon can damage is.
  - *Firers.* Computer test vehicles F1, F2 and F3 stand in this order in the map's vehicle entries.
    F1's test weapon does 100 % against the infantry type's armour class, has damage *d* hit points with
    *d* < *h* ≤ 2 × *d*, so that its second hit destroys U, and has an infantry-death kind for which
    S-0107, claim 17, removes the unit at once (row 8 of that claim's table), as in scenario O8,
    variant d. U is in F1's range and W is not. F2 and F3 carry the common test weapon of scenario
    O8 (damage 1 hit point, 0 % against the infantry type's armour class) and face their own human targets W2
    and W3, unarmed test vehicles with high health. Each firer has no other target in range. All
    three weapons have the same delay between shots, *R* ticks.
  - *Choosing R.* A pilot run with a long *R* records the tick *t*₁ of the firers' first shots and
    the tick *c* in which W disappears, after K's shot and W's death sequence. Then *R* = *c* −
    *t*₁, so that the firers' second volley falls in tick *c*. U's death animations under F1's
    death kind, and W's corpse animation, are named objects.
- *Procedure:* start, and give no orders.
- *Observable:* the ticks on which K and the firers fire, the tick on which W and U disappear, and
  the ticks on which the health of W2 and W3 drops.
- *Runs:* repeated until at least 3 runs count, up to 30. A run counts when all three firers fire in
  tick *t*₁ and again in tick *c*, W disappears in tick *c*, and U is still present at the end of
  tick *c* − 1. If fewer than 3 of 30 runs count, the scenario is inconclusive. The scenario
  counts only if scenario O8, variant d, and the base of scenario O9 passed. Counted ticks: *c*, *c* + 1 and *c* + 2.
- *Expected:* U disappears in tick *c*. W2 is hit in tick *c* + 1 and W3 in tick *c* + 2. In tick *c*
  W leaves play in its own turn, so U, directly after it, misses its turn. The firers' projectiles,
  appended in the firers' turns, come after U. F1's projectile detonates and destroys U, which
  stands before it, so its turn has two counting removals, and the projectiles of F2 and F3 both
  miss their turns. In tick *c* + 1 F2's projectile detonates, and F3's misses its turn again.
- *Falsified if:* in a counted run W3 is hit in tick *c* and W2 in tick *c* + 1, which means that
  U's removal did not count because U had missed its turn. Both hit in tick *c* falsifies the skip
  itself. Any other pattern is recorded and sends claim 10 back for review.

**Scenario O13: assigning random draws to parts of one tick (claim 1, rows 1, 5 and 8).**
- *Setup:* the common flat-map skirmish setup, with crates off, no resource cells or spawning
  terrain, no combat, and one distant bystander computer player. Named test data consists of the
  map's trigger and rules overrides and a temporary test art file defining three non-bouncing,
  non-normalised animation types with no particles, damage, smudges, resource chain reactions or
  other start, middle or end effects. Animation A is played by an elapsed-time trigger in row 1
  of tick *n*. Animation B is started by an earlier trigger and switches to follow-up type C in
  its own turn in tick *n*, under S-0109 claim 15. Choose fixed playback settings for B in a pilot
  run so that its switch falls on a positive multiple of 15 ticks; set A's elapsed-time parameter
  to the integer count of 15-tick trigger time units at that tick (an exact division). A and C each have a descending random-rate pair that converts to a
  stage-delay range of 1 to 900 ticks (S-0109 claim 3). Neither has any further random delay or
  follow-up switch during the observation. A human house owns the powered factory building and
  prerequisites for a test infantry type, with ample credits and no active production. The test
  infantry has no creation effects beyond its creation draw and any initial idle-timer draws
  specified by S-0107. The building and the idle map objects have no random animation effects.
- *Tool and observable:* a read-only state recorder, extending the common recorder to read the
  scenario stream's seed and full state before and after each tick, A's and C's chosen stage
  delays, B's current animation type, and the start of production. Reconstruct the raw values
  between the recorded states using S-0202, including rejected values in ranged draws. Only
  reads and ordinary scripted mouse input are used: no seed setting, injected logging or
  suspension of the oracle. This is a recorder proposal, not a change to an existing harness.
- *Controls:* first run A alone, B's switch alone, and the production-start order alone, five
  times each. Predict their complete draw groups from S-0109 claim 3, S-0202 claim 10 and S-0107.
  Check both the count and the chosen delays, and check quiet intervals with none of the three
  effects. If controls reveal an unexplained draw group or a creation effect not accounted for
  by those specs, this setup is inconclusive until that effect is identified; it is not evidence
  for or against a phase order. The original chooses a new seed in each run; controls compare
  predictions from each run's own recorded state, not results from different seeds.
- *Procedure, part A:* let the row-1 creation of A and B's row-5 switch coincide in tick *n*,
  giving no orders. From the state before *n*, predict the two ranged draws in order A then C
  and also in order C then A. A run is discriminating only when those predictions differ in at
  least one labelled delay. Record all non-discriminating runs as such.
- *Expected, part A:* A's chosen delay uses the first ranged draw, and C's the next, with exactly
  their combined draw count. *Falsified if:* a discriminating run matches C then A instead, or
  the observed delays match neither prediction despite the controls accounting for all draws.
- *Procedure, part B:* omit A's trigger, keep B's row-5 switch, and use scripted input to start
  one infantry production in the cycle of tick *n*. Count only a run whose production starts in
  tick *n* and whose B switches to C in tick *n*. Predict C's ranged draw followed by the
  production-creation group, and the reverse order. Count as discriminating only when the
  predicted C delays differ. No object is released from the factory during the counted tick.
- *Expected, part B:* C receives the first ranged draw, before the production-start order's
  creation group, with exactly the combined count. *Falsified if:* a discriminating run matches
  the reverse allocation or matches neither predicted C delay despite fully accounted controls.
- *Runs and validity:* obtain 20 discriminating runs of each part, at most 100 attempts per part.
  The counted tick is *n*. Apply the common void rule, with A, B and C explicitly named objects,
  and also void a run when an unrelated named effect is active or the bystander acts on the test
  area. If fewer than 20 runs qualify, that part is inconclusive. Unexpected stream consumption
  that survives the control checks is reported for investigation, never silently allocated to a
  row. Part A tests row 1 before row 5, part B tests row 5 before row 8, and together they test
  row 1 before row 8. Neither part confirms the untested rows listed in claim 1's backing.

**Scenario O14: slave groups against repair, movement and power effects (claim 12).**
- *Common setup and tool:* a flat purpose-made skirmish map, one human player with a
  slave miner and one distant bystander computer player, game speed index 6, crates off,
  no engine add-on and no mods (the named test files of a scenario excepted). Stock data except explicitly named map overrides and, in
  part C, a temporary test art file. Extend the read-only recorder to record group
  state, due-update tick, slave places, creation samples, inside/on-map state,
  destinations, loads, building repair state, body construction state and animation
  types and delays. Script only ordinary input. Count three distinguishing
  observations per part within 100 attempts; fewer is inconclusive. Unexpected
  combat, unrelated credits, creation, removal or unexplained draws void the counted
  tick. The common void rule permits the named slaves' deliveries and emergence.
- *Part A, delivery before repair:* pre-place a deployed miner building, so its slaves
  first emerge later than the master in update order. Damage it without killing it and turn on
  repair. Named map overrides may set the building repair period and slave reload
  delay to align a due repair tick with a group update and retain the delivered
  slave inside afterward. In pilot runs, identify an update at which a loaded
  returning slave already stands in the unloading cell without a destination at
  the preceding tick's end. Spend through ordinary wall placement or completed
  production after the previous repair step so that available credits just before this update are less
  than one repair step's cost; the impending delivery must cover that cost. No
  other line spends credits in the counted tick. Record health, repair state,
  credits, slave load and presence in that tick. Expected: the slave is received,
  its payment less the repair cost is the credit delta, and health rises by the
  repair step with repair staying on unless full health is reached. A repair that
  turns off for insufficient money despite the qualifying delivery falsifies the
  ordering. A control with the slave one cell short gets no payment and repair
  turns off. If the slave was not delivery-ready before the master's turn, the
  run is not distinguishing. This also distinguishes delivery in the master's
  turn from delivery in the later slave's turn.
- *Part B, group before movement:* use a travelling vehicle master with a nonempty
  destination. Pilot runs tune named map overrides for movement speed and a field
  destination so that its final movement reaches that destination in tick *n*,
  which is also a due group-update tick. The destination must still be nonempty
  at the end of *n* − 1, and the site must permit deployment. Give no further
  orders. Record vehicle position, destination, group state and conversion tick.
  Expected: at the end of *n* the vehicle has arrived and lost its destination but
  has not deployed through that update. It attempts deployment at the next due
  group update with no destination, normally *n* + 10 by S-0402 claim 21.
  Deployment in *n* through the group seeing that tick's arrival falsifies the
  ordering. A failed site or an independent deploy order makes the run void.
  In a second control, let a field-seeking update assign a site on an unobstructed
  straight route, with no turning, suspension or behaviour gate delaying movement.
  Record the assignment and first movement ticks. Movement may use that new
  destination in the assignment tick; a mandatory extra tick despite satisfied
  movement gates falsifies that consequence, not the arrival test.
- *Part C, operation-state draws before group draws:* named map data makes the
  building master need power with a positive drain. Named test art gives it a
  working animation with a random stage delay, no damage or extra effects, which
  starts afresh on recovery from low power. Stop other animation and creation
  effects. Ordinary construction or sale of another power/drain building restores
  full power in the house update before the counted master turn. Align a lost
  slave's replacement expiry with that turn's group update. The replacement stays
  inside, using a blocked emergence cell. Control runs exercise recovery alone
  and replacement alone and account for their complete draw groups using S-0109
  claim 3 and S-0202 claim 10, including rejected ranged-draw values and infantry
  creation effects. If recovery resumes an old animation rather than starting a
  fresh one, or any draw remains unexplained, this setup is inconclusive.
  Read the scenario stream before and after each tick and the labelled animation
  delay and replacement creation sample. Expected: all recovery-animation draws
  precede the replacement-creation group, with exactly the combined draw count.
  Count only runs in which that allocation differs from the reversed allocation.
  A reversed labelled allocation falsifies the ordering; matching neither sends
  the draw accounting back for investigation. Also repeat S-0402 H22's working
  construction-state gate with its completion tick aligned to a due update:
  that update sees the completed construction state, not the preceding tick's.
  H22 owns the resulting group transition and emergence rules.
- *Part D, delivery removal in the master's turn:* allow a slave to emerge, then
  pack and redeploy its master while that slave remains on the map, preserving the
  group under S-0415 claims 6 and 10. The new building master is later than that
  slave in update order. Produce an armed test vehicle observer after redeployment,
  with a stationary enemy target in range, a one-tick shot delay and no firing or
  damage effects except its projectile. Record a delivery-ready slave at the end
  of the preceding tick and exactly one delivery in a due group update. Count only
  when the recorder's update order puts the observer directly after the master,
  its weapon is ready, and no other object enters or leaves play in the master's
  turn. Expected with the removal-skip switch on: the observer does not fire in
  the delivery tick, but fires in the next tick. Its normal shots otherwise occur
  every tick. For a control, use a pre-placed building master and produce the
  observer before its slaves first emerge: those slaves are later than the master.
  A qualifying delivery there costs the observer no turn. Firing in the first
  test's delivery tick, or a missed turn in the control caused solely by delivery,
  falsifies the counting-removal consequence. With the switch off, neither case
  loses a turn. The named observer's ordinary projectiles are exempt from the
  common unexpected-firing rule; reject ticks with any additional removal before
  the observer, so a different cause cannot explain its silence.

**Scenario O15: vehicle behaviour before the slave group (claim 12; Q-0430).**
- *Setup:* O14's quiet skirmish setup at game speed index 6, with a human vehicle slave
  master, a distant bystander computer player, no engine add-on and no mods (the named test files of a scenario excepted). Use stock data
  except named map overrides for the slave replacement delay and the harvest behaviour
  rate. Lose one emerged slave through ordinary combat, then remove the attacker from
  the test area and wait for all combat and death effects to end. Keep the remaining
  slaves inside by an ordinary recall order. Choose the replacement delay in pilot runs
  so its first eligible replacement update is tick *n*. Prepare an unobstructed
  resource destination for directed harvesting, away from the master's cell.
- *Tool and observable:* extend O14's read-only recorder with the current and queued
  behaviours, behaviour-start tick, selected processing delay, scenario random state
  before and after each tick, and the replacement's creation sample. Record destinations
  and group state too. Use ordinary scripted input, never set the seed or alter state.
- *Controls:* exercise directed harvest processing alone and replacement alone, five
  runs each. Account for all draws using S-0402 claim 22, S-0215 claim 6, S-0202
  claim 10 and any infantry creation effects of S-0107. Keep replacement slaves inside
  so there is no emergence draw. Unaccounted draws make the test inconclusive until
  identified. Choose a harvest rate that leaves several possible recorded delays.
- *Procedure:* queue directed *harvest* at the end of tick *n* − 1 through ordinary
  input. Count only runs in which the vehicle is ready to change, the recorded start
  tick is *n*, its harvest processing chooses a delay in *n*, and the replacement is
  created in that same turn. From the stream state before *n*, predict the labelled
  harvest delay and replacement sample for harvest-then-replacement, and for the
  reverse order, including rejected ranged-draw values. Count only predictions that
  differ. Obtain ten distinguishing runs within 100 attempts; fewer is inconclusive.
- *Expected:* the harvest delay takes its draw before the replacement's creation
  group, with exactly the accounted consumption. The new behaviour therefore starts
  and processes before the due group update even though its order was applied at the
  preceding tick's end. The later group's changed state cannot influence that already
  completed processing. Destination and group-state records must also satisfy S-0402
  claim 22; these records alone need not distinguish the two orders.
- *Falsified if:* a distinguishing run matches the reverse draw assignment. Matching
  neither prediction requires investigation of draw accounting, not an assumed order.
  Void runs with unrelated creation, removal, combat or bystander activity in the test
  area during the counted tick. This proposes a recorder extension, not a harness change.

**Scenario O16: cross-part labelled draw allocation (claim 1).**
- *Setup and tool:* O13's flat skirmish, game speed index 6, no engine add-on and no mods (the named test files of a scenario excepted), with
  a distant bystander. Use named map overrides and a temporary test art file for the
  probes below; a temporary rules file may select the blast-ring animations and crate
  settings. Remove these files afterward. Use O13's read-only random-state recorder
  and ordinary scripted input, recording the labelled delays, resource changes, crate
  cells and lifetimes, tag firings and production starts below. Never set the seed,
  suspend execution or insert logging into the game. This proposes recorder support,
  not a change to an existing harness.
- *Probe definitions:* each label identifies an outcome, not an internal observation point.
  - **P, row 1:** O13's elapsed-time creation of a random-rate animation.
  - **G, row 9:** one resource type with one growing cell, as S-0401 E4, with spreading
    disabled by a zero spread fraction. Named map data gives growth a positive fraction
    and a short period. Record the cell's stage and the new growth-entry key. Predict
    the complete update using S-0401 claim 7 and F2, including its initial raw draw,
    re-entry and any spread offer that actually adds an entry. The read-only key
    distinguishes assignments even when the same cell grows under either prediction.
  - **D, row 12:** S-0615 DL1's due disc-laser detonation at an isolated durable target.
    Its test warhead creates one non-damaging random-rate animation on that terrain,
    with no other hit effects. Its chosen delay labels the detonation's draw group.
    S-0303 owns damage-animation selection and S-0109 owns animation draws. The
    disc producer itself takes none (S-0615 claim 18).
  - **W, row 3:** a psychic domination sweep on empty ground, using S-0612 PD3/PD9's
    fixed lead-in timing, but a random-rate sweep animation as its labelled outcome.
    With no recipients, resources or damage effects there, account only for the sweep
    animation's creation group. Separately repeat with the weather storm of O10 B,
    aimed at empty ground: record bolt/cloud cells and animation delays, accounting
    for all storm placement, sound and animation draws under S-0513 claims 14 to 19,
    S-0701 claim 25 and S-0109. Do not assume lightning sounds are draw-free.
  - **T, row 5:** O13's switch from animation B to random-rate C.
  - **R, row 15:** a damage-enabled blast ring created by the original's map-trigger
    action for a ring at a waypoint, on empty flat ground with no resources, walls or
    objects in its affected area. This is an oracle-only fixture, not an extension of
    S-0011's supported action subset. Named test rules select two harmless initial
    ring animations with distinct random-rate ranges; their delays label the first
    update. Use S-0109 for both creation groups and S-0303 for any area-effect draws.
    Do not substitute the domination ring, which takes none. Count only the first
    update, in its creation tick, not later ring damage or distortion paths.
  - **C, row 16:** S-0520 CR2's zero-regeneration control with exactly one crate,
    crates on and free replacement cells: one replacement per tick, not repeated
    same-tick expiry. Record the replacement cell
    and lifetime. Predict x/y attempts, any overlay-placement draws and lifetime with
    S-0520 claims 3 and 4 and S-0202 claim 10. No unit collects a crate in this tick.
  - **H, row 7:** a house-assigned one-time tag with any-event and a variable-set
    condition, creating a harmless random-rate animation when it fires. Use O7 B's
    entered-by variable setter to enable it during row 5 of a tick divisible by 8.
    Before that turn-pass action the variable is clear, so row-1 polling cannot fire
    H. Record the setter, tag firing and animation delay. S-0011 claims 17 and 35
    and S-0414 claim 18 own the raise and gate; count only same-tick firings.
  - **O, row 8:** O13 B's infantry-production-start order and complete creation group.
- *Controls and alignment:* run each probe alone five times, with all other probes
  disabled, and account for every raw draw and labelled outcome from that run's own
  incoming state. Check quiet ticks too. Include the otherwise draw-free variable
  setter in H's controls. Tune only named timing fields and ordinary input in pilot
  runs so the two probes of a comparison occur in the same tick. For comparisons
  involving H use a tick divisible by 8; for P use a positive multiple of 15, and
  for both use a positive multiple of 120. Record actual event ticks: missed
  coincidence is inconclusive, not evidence of reversed order. Unexplained draws in
  controls must be resolved before counting a combined run.
- *Procedure and expected results:* exercise each pair separately, with no other
  probe active. Reconstruct the two complete draw groups from the incoming state,
  including rejected raw values, once in the expected order and once reversed.

  | Part | Pairs, earlier allocation first | Labelled observables |
  |---|---|---|
  | A | P then G; G then D | animation delay, growth-entry key, disc-hit animation delay |
  | B | D then W; W then T | disc-hit, sweep or storm outcomes, C's delay |
  | C | T then R; R then C; C then H | C's delay, ring-animation delays, crate cell/lifetime, house-animation delay |
  | D | H then O | house-animation delay and item-creation sample |
  | E | two D probes, newest shot first; two H probes, house order first | separately labelled animation delays |

  Part B uses both W variants separately. Part E aligns disc shots from distinct
  map-placed test vehicles in one tick with identical schedules and separated targets.
  Their entry order fixes firing order by claim 3; repeat with those entries reversed. For H use two assigned tags owned by distinct houses,
  both gated by the same turn-pass variable setter; reverse their house-creation
  order in the map, preserving tag definition order. Keep those houses undefeated.
  The expected assignment follows house order, not tag definition order.
- *Falsifiers and runs:* for each pair or variant obtain ten discriminating runs in
  at most 100 attempts. A run discriminates only when the two allocation predictions
  give different labelled outcomes. Expected: all labels and outgoing stream state
  match the table's assignment. A reversed assignment falsifies that link or member
  order; matching neither after fully accounted controls falsifies the proposed
  accounting and requires investigation, not silent reassignment. A total draw
  count alone never confirms order. Fewer than ten discriminating runs is
  inconclusive. Apply the common void rule, exempting only the named probe effects.
  Any unrelated draw or bystander activity voids the counted tick. These pairwise
  links establish the listed nonempty draw chain transitively, not the empty
  allocations of rows 6, 10 and 14 or original-only team/strategy decisions.
- *Part F, bomb maintenance without scenario draws:* separately use S-0519 T11's
  attached-bomb visibility fixture, with a long detonation delay and no combat.
  Record the scenario stream on quiet ticks, the ticking sound's start and ongoing
  playback, a due visibility refresh and carrier removal through an ordinary order
  after the observed maintenance. Account for order-caused effects separately.
  Compare with no planted bomb and with sound omitted in named test data. Obtain
  five refresh intervals with sound enabled and five with it omitted. Expected:
  maintenance adds no scenario draws on sound-start, ongoing-sound or refresh ticks.
  A reproducible extra draw attributable solely to one of these events falsifies
  that no-draw claim. Detonation and carrier-destruction draws are excluded, not
  treated as maintenance. An unexplained unrelated draw makes the interval void.

**Production-start assignment (claim 8).** S-0404 P11 B and C own the shared measurement
scenarios: game speed index 6, quiet test maps with named test data, scripted normal input and a
read-only random-state and item-creation recorder. P11 B reverses line creation order while
keeping tab/icon cancellation order fixed; replacement samples must still follow cancellation
application order. P11 C aligns a factory-removal start during the turn pass with a build-order
start at that tick's end; the former sample must occupy the earlier stream position, allowing
intervening draws. Each requires three distinguishing observations; reversed assignment falsifies
the rule. These tests do not measure the between-house order-boundary design extension or the
internal position of the non-drawing row-6 work relative to row-7 draws.

**Assigned-tag raises within a house update (claim 8).** S-0414 V15 owns the shared
measurement: a named test skirmish map, scripted input and a read-only recorder compare
surrender in tick 7 with the assigned-tag raise in tick 8, then use a turn-pass gate to make
a tag remove prerequisites or the last qualifying object in tick 16. Expected: the raise
sees surrender and defeat, due tech-tree/sidebar work sees tag removals in that update,
newly due power/radar work waits until tick 17, and a tag-caused last-object loss is tested
for defeat in tick 17, not tick 16. Continued raises after defeat and cleanup absence use
V15 part a and V13. Reversed timing or a missing eligible raise falsifies the corresponding
rule; V15 specifies controls separating the ordinary poll from the owner-house raise.

- **O17 (claims 2 and 10; ending animation).** Use S-0109 scenario AN26's tiny
  skirmish, game speed index 4, no engine add-on and no mods (the named test files of a scenario excepted), with one non-damaging sprite
  animation having an explicit finite end, no follow-up and no creation effects.
  Create it by pre-pass trigger polling. While it plays, have one existing firer launch
  exactly one long-flight projectile with no shot animation or trailer, then stop firing.
  Confirm that the projectile directly follows the animation in update order and that the
  projectile will neither detonate nor create effects during the observation window.
  Exclude all other creations and removals around the animation's final turn. Use the
  read-only recorder for update order, animation stage, removal tick and projectile
  position and turns, with end-of-tick captures; repeat three times. Expected: the
  animation's own removal costs the directly following projectile its turn in that
  tick, delaying its next movement until the next tick. Falsified by a projectile
  turn after that removal in the same tick with those preconditions confirmed.
  The engine-only companion run with the removal-skip switch off must retain that
  projectile turn. AN26 separately tests creation placement and the new explosion's
  missed first turn; this scenario tests removal at animation completion.

- **O18 (claim 13; leftovers and the defeat sweep).** A tiny skirmish map, game speed index 4, no
  engine add-on and no mods (the named test files excepted), short game off, crates off except in
  part (c). A loose test copy of the base rules, placed before the program starts, sets the bases
  key of its multiplayer dialog section to no and its minimum unit count to 0, and the unit-count
  setting is 0, so that no house has a base unit or starting units (S-0412 scenario K1, run 2 and
  part b). A test map pre-places every object named below. House D, a computer house, owns only two
  vehicles V1 and V2 of a named test vehicle type, a land vehicle that cannot crash, standing on
  land, with the explodes flag, a death weapon, minimum and maximum debris counts above 0 and
  non-empty explosion and destruction-animation lists, and no other object. House K, the local
  player's house, owns only a group of vehicles whose weapon destroys a test vehicle with one hit. D
  and K are the only houses and are enemies, so the game ends with D's defeat in every part (S-0414
  claim 6). Each run is recorded until play stops. Scripted input gives K's attack orders and their
  timing. Use the read-only recorder with end-of-tick captures of the game's objects, every hit with
  its warhead, attacker and health before and after, every death-weapon detonation and every
  animation and debris object created, the tick in which each house is marked defeated, and the
  score screen's kills and losses, read after play stops (S-0414 claim 19). (a) Let K destroy V1,
  and V2 at least 10 ticks later, so that D is defeated in the tick *t* of V2's destruction.
  Expected: D marked defeated in tick *t*. At *t*, after V2's destroying hit, one more hit on V2 of
  no amount, with the demolition warhead, ignoring defences, naming no attacker and taking no
  health. Then a second death-weapon detonation at V2's last position, a second explosion and
  destruction animation there and a second throw of debris, all in tick *t*. V2 is listed off the
  map after tick *t* and not after tick *t* + 1. D's losses count V1 and V2 once each, and K's kills
  are 2. Falsified by D's defeat in another tick, by no hit after the destroying hit, by single
  death effects of V2 in tick *t*, by a third loss of D or kill of K, or by a row of V2 after tick
  *t* + 1. (b) Place V1 and V2 side by side and order K's group to attack V1 and then V2, so that
  V1's destroying hit comes in tick *t* − 1 and V2's in tick *t*. A run whose two destroying hits
  are not in consecutive ticks is set aside and repeated with another seed, until one run has them
  so. Expected: no hit on V1 in tick *t*, and V1 not listed after tick *t*. Falsified by a hit on V1
  in tick *t*. (c) As (a), with crates on, the test type marked to carry a crate and V2 standing on
  open, clear ground, so that free cells are near it. Count the crates on the whole map in each
  end-of-tick capture. Expected: two more crates after tick *t* than before it. Falsified by one
  more. A run with three or more, which a crate of the game's own crate setting can cause, is
  repeated. (d) Engine only, `quirks.defeat_sweep_leftovers` off: as (a). Expected: no hit on V2
  after its destroying hit and single death effects. (e) Two runs as (a), V2 carrying a tag whose
  repeating trigger fires when V2 is destroyed by anything and shows a text, and each firing
  recorded. In the second run D also owns a third vehicle V3, far from K, so D is not defeated in
  tick *t*. K's group destroys V3 at least 10 ticks after V2, which defeats D and ends the game.
  Count V2's firings from V2's destroying hit to the end of the run. Expected: the same count in
  both runs. Falsified by more firings in the first run.

## Changes for other specs

- **Glossary, *destroyed, removed*.** Its sentence that a removed object "no longer exists" can add:
  a removed object is kept as a leftover to the end of the tick in which it is removed, and only the
  defeat sweep can reach it (S-0203 claim 13, Terms *leftover*). Confidence: high for the last
  vehicle of a house, destroyed by a hit, still taking a hit later in the same tick (M-0021), medium
  for the end of the tick and otherwise. Backing: `measured` (M-0021) for that later hit,
  `measurable` (S-0203 O18 part b) for the end from the house update of the next tick on,
  `unmeasured` for the parts of the next tick before it (S-0203 claim 13 (a)).
- **For S-0414 claims 5 and 15.** Claim 5's sweep also offers its hit to the house's leftovers in
  the original (S-0203 claim 13 (c)). Our engine offers it to leftover vehicles with no health only,
  under `quirks.defeat_sweep_leftovers`. Two sentences of claim 15 do not hold for leftovers in the
  original. "The offered hit uses the object's health at that time; an already-dead object does not
  undergo a second death from a zero hit." A leftover destroyed by damage takes a hit of no amount
  that registers no destruction but sets off its death effects once more (the S-0312 entry below).
  "A passenger already killed with its carrier receives no second death and contributes no second
  set of death draws." Such a passenger is believed to keep its health, so the sweep's hit on it is
  a killing hit (S-0203 Open questions). The control of scenario V11 part b, whose passenger dies
  with its transport before its own place "without a second death", would then not hold in the
  original. S-0414 owns both rules. Confidence: high for the last vehicle's further hit in the tick
  of its destruction taking no health (M-0021), medium for that hit being the sweep's and for the
  repeated death effects, low for the passenger. Backing: `measured` (M-0021) for the hit,
  `measurable` (S-0203 O18 part a) for the effects, `unmeasured` for the passenger, which no
  scenario here sets up.
- **For S-0312 claims 1 and 32.** When the defeat sweep's hit of no amount reaches a leftover
  vehicle that damage destroyed (S-0203 claim 13 (c)), the original repeats, at the sweep's place in
  the tick, the death effects that the vehicle's state still allows, in claim 1's order and with
  their draws, as at a further killing hit of claim 32: its debris (claim 4), the death weapon of an
  exploding object (claim 6), the explosion and destruction animations at its last position (claim
  9) and a crate that it carries (claim 14). Effects whose subject is gone do nothing again:
  passengers already handled (claims 5 and 12) and a bomb already detonated (claim 7). Claim 13's
  crew rule applies with the hit's no-escape marker. No destruction is registered, so there is no
  credit (S-0311 claim 9, S-0414 claim 21) and no raise of claim 29. What happens to a leftover that
  meets claim 10's sinking conditions or claim 15's crash conditions at this hit is not established.
  Confidence: medium. Backing: `measurable` (S-0203 O18 parts a and c).
- **For S-0302 claim 8 (*No-health targets*).** The elimination of a house's objects, one of the
  sources that the claim leaves to their specs, does not skip an object with no health: S-0414 claim
  5's sweep offers its hit to the house's leftovers (S-0203 claim 13 (c)), and a leftover vehicle
  that damage destroyed has no health. For such a hit on a vehicle, the row *The target is not a
  building and already has no health* still stops any change of health, and no destruction is
  registered. In the original the hit does not end there, though: the vehicle's death effects of
  S-0312 come once more, as after a killing hit (the S-0312 entry above). So, for a vehicle, the hit
  reports *now dead*, not *unaffected* (claim 8's first sentence and the *unaffected* row of its
  State and ordering table), though it takes no health: claim 14's "A hit that takes the last health
  reports *now dead*" and the table row "now dead | Health reached 0" do not cover it. The sweep is
  also a known source of a hit on an object with no health, and S-0203 O18 (a) delivers one. Claim
  8's sentence "No known source delivers one (*No-health targets* below), so no scenario sets it up,
  and the stages of those two rows are low confidence" and its Backing's "the sources that might not
  skip it belong to specs not yet written, so no scenario can deliver a hit to such an object" no
  longer hold for vehicles. Claim 8 can cite O18 (a) for the vehicle case. The other rows, the
  building row and the other sources are unchanged. What such a hit does in the original to leftover
  infantry, aircraft and buildings, under the rows that claim 8 gives them, is an Open question of
  S-0203. Our engine offers those leftovers no hit (S-0203 claim 13 (c)), so claim 8's rows stand
  for them. Confidence: medium. Backing: `measurable` (S-0203 O18 parts a and c) for the repeated
  effects. The report itself is `unmeasured`: it is seen only through the effects it sets off.

## Open questions

- Q-0412 question 1 is answered by claim 12 together with S-0402 claims 21 and 26.
  Questions 2 to 7 are answered by S-0402 claims 21 and 26. In particular,
  claim 26 resolves question 4: empty zero-capacity slaves alternate guard and
  group-driven searches without loading; empty negative-capacity slaves can remain
  harvesting resource land without changing its resources or their loads. Neither
  returns merely because its capacity is non-positive. S-0402 claim 26 owns these
  rules, their quirk alternative, backing and measurement scenario.
- Network games are out of scope here, as in S-0201 (claim 13) and S-0202 (claim 12). They run the
  same parts of a tick plus some of their own, and take the orders of every player. A future
  multiplayer spec owns them, including their extra draws from the scenario random stream.
- S-0306 owns the arrival and detonation conditions, including invisible projectiles and the
  nuclear wait. Scenarios O1, O3 and O8 require first-turn detonation for their test weapons and
  check it in pilot runs; they do not establish it for every weapon.
- Whose turn removes an infantry unit when its death sequence completes is not traced (claim 6).
  Scenario O9 tests the unit's own turn.
- Whose turn performs boarding, the boarding unit's or the transport's, is not traced, and neither
  is whose turn performs other entry exchanges, except slave delivery, which claim 12 places
  in the master's turn. It decides which object loses its turn when a
  unit boards or goes inside (claim 10) and whether a boarding open-topped passenger's next turn is
  its second of the tick (claim 11).
- The earlier simultaneous-item-start allocation question is resolved by S-0404 claim 11's
  causing-action precedence, cited in claim 8 here. P11 B and C propose assignment tests, not
  completed measurements; the between-house order-boundary extension remains an unmeasured
  design rule. Confirm this resolution together with S-0202 claim 10 at hand-off. It does not
  imply a row-6 draw comparison: that work takes none, and queued creation belongs
  to its causing action. O16 tests house tag-action draws against neighbouring drawing parts.
- Rows 10 and 14 retain precautionary places although no reachable draw-producing path
  is identified (claim 1). If their owning specs establish that they take none, those places
  can be released to claim 1's last point's freedom, absent another observable consequence.
  Row 6 is already known to take no own-update draws; its credit and hold boundaries remain
  required, while its internal position relative to house draws remains unmeasured.
  Work listed in claim 1's last point that turns out to draw needs a row.
- The open-topped rule of claim 3 is traced for boarding and getting out. That nothing else takes
  such a passenger out of the update order while it is inside is not traced in full.
- A leftover with health left, such as a passenger killed with its transport (S-0312 claims 5 and
  12), which is believed to keep its health, or a crashable surface vessel that leaves play with 1
  health point (S-0312 claims 10 and 15), whose house is defeated in the same tick: the sweep's hit
  then has an amount above zero. In the original it is believed to destroy the leftover again,
  credit included (low). Our engine offers it no hit, in line with S-0414 claim 15's no second
  death, until S-0414 settles it (**Changes for other specs**). Confidence: low.
- Leftovers of infantry, aircraft and buildings taking the sweep's hit of no amount: the shared
  death effects of S-0312 claims 4 to 7 are believed to come again, as for a vehicle (low). The
  kind's own death effects (S-0312 claims 8, 16 and 17 to 24) are not traced. S-0302 claim 8's
  no-health rows may not hold for them in the original either. Our engine offers these leftovers no
  hit until this is traced. Confidence: low.
- An order carried out in row 8 of tick *t* whose actor or target was removed earlier in tick *t*
  names a leftover. What the original does with it is not traced. Our engine treats it as naming an
  object that no longer exists. Confidence: low.

## Questions for the project lead

- **Partial order (applied).** The scrub of revision 2 asked for a choice between dropping the
  fixed places that have no stated consequence and keeping a place only where one is stated.
  Revisions 3 to 5 do both: the table keeps a place only where a consequence and a test are stated
  (rows 1 to 8), and all other once-per-tick work has no place in this spec until its owning spec
  finds a consequence (claim 1). Revision 6 narrows that: the draw order is a consequence, so every
  part that takes draws now has a place (rows 9 to 16), and the free work is only work without
  draws. Please confirm.
- **Terrain objects at game start (revision 4).** The scrub of revision 3 offered to let our engine
  put map-placed terrain objects anywhere in the update order. Revisions 4 and 5 keep the
  original's place, first in the update order, instead. Only part of that is testable: scenario O9,
  variant T, shows that a terrain object that takes turns does not stand between the map's infantry
  and buildings, where it would take the missed turn that claim 10 gives the first building. That
  terrain objects come before the vehicles, rather than anywhere else outside that gap, stays
  `unmeasured` (claim 3). The choice is between keeping that untested place for fidelity and
  stating a weaker rule of our engine, "anywhere except between the infantry and the buildings".
  Please confirm.
- **Draw order (revision 6).** Claim 1 now fixes the place of every part of a tick that takes, or
  might take, draws from the scenario random stream, in the original's sequence, and limits the
  freedom to place other once-per-tick work to work that takes no draws. Rows 9 to 16 are new. Some
  of them (teams, the decisions of computer houses inside row 7) belong to the original's computer
  player, which our engine replaces; those portions describe original scheduling only. Our AI
  still acts through normal orders, not through those rows. Rows 10 and 14 retain precautionary
  places without an identified reachable draw-producing path. Row 6 is known to take no
  own-update draws: its observable credit and hold boundaries remain required, but its
  position relative to house draws has no random-allocation test. Please confirm retaining
  the precautionary constraints for rows 10 and 14 and the non-drawing row-6 position. O13 tests rows 1, 5 and 8; O16 tests the other identified in-scope
  draw allocations. Team and computer-strategy scheduling remains deferred context.
- **Removal condition (revision 5).** The scrub of revision 4 found that claim 10 stated its
  condition two ways. Revision 5 keeps one: a removal counts when the removed object is the object
  taking its turn or stands before it in the update order, whether or not it took its turn in this
  tick. Scenario O12 tests the case in which the two readings differ. No decision is needed unless
  the lead prefers a different rule for our engine.
- **For sign-off.**
  - Revision 8 makes claim 6 agree with S-0312 (approved). Points (a), (b) and (e) used to say that
    every destroyed vehicle and building leaves play at once and that a destroyed object takes all
    its passengers with it; S-0312 claims 10, 12, 15, 16, 21 and 24 have exceptions. Claim 6 now
    states the general rule and points to S-0312 for the exceptions, which S-0312 owns. Scenario
    O8 gives W1 the crash flag in variants b and c, so that its passengers die with it as the
    scenario needs, and adds variant f, in which they get out. S-0312's sentence "This refines
    S-0203 claim 6b" remains valid. Current S-0312 claims 1 and 12 contain the previously requested
    corrections: out-of-turn deaths share the effect's place in the tick, and claim 6(e) refers
    there for passenger death or escape while claim 3 here owns open-topped passengers' in-play
    status. Revision 11 removes the obsolete sale-completion alternative for a building with
    0 hit points: S-0312 claim 21 owns its removal, including a building destroyed during a sale.
    Claim 6 rates the general rules of points (a) and (b) high, as does current S-0312 claim 21.
    Revision 10 replaces the earlier deferral to the structures specs with S-0312 claim 21's
    removal-turn rule, exceptions and second round. Only the type's explodes flag or the sale's
    behaviour causes that deferred removal, not other ways an object counts as exploding.
  - S-0011, S-0201, S-0202 and S-0213, cited above, are approved specs; where a newer draft exists,
    this revision has also checked that draft.
  - S-0107 (approved), claim 17, is assumed by scenarios O8, O9 and O12: row 8 of its table removes
    a killed unit at once, infantry-death kind 1 selects the first death sequence through row 5, and a death request
    is turned down when that entry has no image frames, which is why the test infantry type
    needs frames there.
  - The trigger words used in claims 1 and 8 and scenarios O6 and O7 come from S-0011.
    *Tag* and *raise* are defined in its Terms, and so is the trigger time unit of 15 ticks behind
    "elapsed time *v*" (tick 15 × *v*). The *counter* is defined in its common measurement setup.
  - S-0201 cites rows 1 and 4 and claims 7 and 9 of this spec. Those labels and subjects are kept.
    S-0201 calls row 4 "the map-wide effects of that tick". Row 4 now holds only the chrono flash's
    step, which is the part S-0201 means, so the citation still resolves.
  - S-0213 cites claim 8 for "the production lines of a tick are updated before that tick's
    orders". Claim 8 keeps that statement, now measurable (scenario O7, part C).
  - Revision 6 fixes again the orders after the houses' updates (claim 7), the houses in house
    order and after the production lines (claim 8), for the draw order of claim 1. S-0213's claim 3
    fixes its own order between houses' orders as a rule of our engine; that is unaffected.
  - Row 1 now follows S-0011, claim 6: the per-tick polling raises every polled tag, and the
    condition events, the any-event code, empty event lists and latches decide. Revision 5 described
    it as time-based checks plus reports of changes since the previous check, which did not match
    S-0011 (claim 22 has a variable change seen by all tags in the rest of the same tick). S-0011
    was right; this spec was the wrong one.
  - S-0202, claim 10, asks that drawing effects run in the original's order. Claim 1's draw order
    and rows 9 to 16 are this spec's side of that requirement.
  - Current S-0202 claim 10 includes a house update among the places that can start production
    and take a produced object's creation draw; the earlier cross-spec correction is complete.
  - S-0213's citations of claims 7 and 8 still hold; claim 7 also places the orders after the
    houses. Confirm the latest same-batch S-0201 cycle wording, S-0107 claim 15's
    destination-driven sequence changes and S-0213 claim 21's Q-0072 approach pointers
    together. Immediate destination or sequence changes at order application do not give an
    object a movement turn before the next tick.
  - S-0414 claims 8, 10, 11, 15 and 16 agree with claims 7 and 8 and row 7 here:
    house-order defeat tests, next-tick surrender destruction, destruction outside the turn
    pass and final-tick commitment retain their stated boundaries. S-0415 claim 6 agrees
    with claims 3, 4 and 10 here on a deployed building's appended place and possible missed
    first turn. These are confirmations, not new behaviour. Recheck the versions handed off
    together at sign-off; this check does not replace S-0414's wider joint hand-off checks.
  - S-0404 claims 1 and 11 separate payment priority from creation-draw allocation. Revision 15
    adopts claim 11's causing-action precedence, with P11 B/C's proposed assignment tests,
    including turn-pass starts before order-boundary starts and the unmeasured between-house
    design extension. Confirm claims 1 and 8 here and S-0202 claim 10 against the versions
    handed off together. Revision 17 rechecks this agreement against current S-0404
    claims 7 and 11 and S-0202 claim 10: completion alone causes no replacement-start
    draw batch in row 6. A later release or place action causes any queued successor's
    start and its creation draw. Keep P11 B/C as proposed measurements, not measured
    results. Row 6 takes no own-update draws, so no random outcome distinguishes
    its internal position relative to row-7 draws; O7 tests its observable boundaries.
