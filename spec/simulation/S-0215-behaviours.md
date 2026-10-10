---
id: S-0215
title: Behaviours (standing orders) and their processing
topic: simulation
revision: 10
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0215: Behaviours (standing orders) and their processing

## Summary

Every infantry unit, vehicle, aircraft and building carries a *behaviour*, a standing order such as
guard, hunt, move or attack, and this spec covers how behaviours are held, queued, started,
forced, overridden and reverted, when and with which random delay an object processes its behaviour, which
resting behaviour it falls back to, and the per-behaviour properties that the rules data can set.
It also says what the general-purpose behaviours do at each processing, and only points to the
topics that own the others (harvesting, structures, superweapons, team scripts).

## Terms

Terms from the glossary are used as defined there, in particular *tick*, *game minute*, *house*,
*human-controlled*, *lepton*, *cell*, *facing*, *foundation*, *object*, *infantry*, *vehicle*,
*aircraft*, *building*, *weapon*, *warhead*, *scatter*, *height above the surface*, *world height*,
*scenario random stream*, *rules layer*, *game-wide section*, *game start*, *recorder*, *measurement
setup*, *departure*, *importer warning*, *quirk switch*, *unspecified* and *round-toward-zero mode*.
*Sequence kind*, *interruptible*, *deploy kinds*, *frame count* and *idle action* are used as S-0107
defines them. *Turn* and *turn pass* are used as S-0203 (tick order) defines them.

Other specs cited: S-0202 (random streams), S-0203 (tick order), S-0204 (coordinates, distance
and facing), S-0205 (number domains), S-0206 (turn rates), S-0207 (ground vehicle movement), S-0208 (infantry
movement), S-0213 (human orders) and S-0214 (order targeting rules) are drafts in the same batch
as this spec. S-0001, S-0003, S-0006, S-0105 and S-0107 are approved. S-0301 (the firing cycle,
planned) is cited as a pointer only: it owns no claim that this spec relies on, and every
statement this spec makes about the firing check (Terms) is vocabulary or a claim of this spec.

**Queueing wording.** Throughout this spec, "queues *b*" without further words means a queue
request (claim 7) *without* an immediate start. A rule that asks for the readiness-dependent
immediate start of claim 7 says "with an immediate start" (or "with an immediate start if
ready"), and a rule that makes the queued behaviour current regardless of readiness says "starts
it at once, whether or not it is ready to change".

**Houses in our engine.** The project lead decided on 2026-10-09 that AI houses (Terms) use the same
unit rules and the same order legality as human-controlled houses: no behaviour rule applies only
to computer-controlled owners, the recruitable property has no effect, and AI houses get no
exemption from the owner check or from the limit on pending orders. This is a rule of our engine,
not a claim about the original. Two further points are not part of the decision but follow from
it and from the project's scope. AI houses have no teams, because the original's team system is
out of scope. And an AI house has the house skill level (Terms) of a human-controlled house,
because the skill level is part of the unit rules. Throughout this spec:
- a rule given for a "human-controlled owner" applies in our engine to the units of every house,
  human or AI,
- a rule, a table row or a condition given for a "computer-controlled owner" describes the
  original only. It is marked **Original only (note for AI designers)**, and our engine applies it
  to no house: in our engine no owner meets the condition "computer-controlled", so such a row
  never matches and such a condition never holds. What the original's computer player gets from
  the rule is recorded so that our AI may choose to reproduce it through ordinary orders,
- a condition on team membership never holds in our engine, because an AI house has no teams
  (Terms, *team member*),
- a rule gated by the house skill level, not by the owner, stays a rule of our engine. An AI house
  has the skill level of a human-controlled house: 0 in a skirmish (Terms, *house skill level*).

Each claim states its backing and confidence for the original's behaviour, including the parts
marked as the original's only. Which houses our engine applies a rule to is a design rule of our
engine, not a claim about the original, and needs no backing.

Terms used by this spec:

- **Computer-controlled:** a house of the original that its computer player plays rather than a
  person; the opposite of *human-controlled*. This spec uses the word for the original only: a
  rule given for a "computer-controlled owner" states what the original does for the units of
  such a house.
- **AI house:** a house of our engine that our own AI plays. It takes the place of the original's
  computer player, whose script and team system is out of scope. It acts only through orders, and
  it has the unit rules and order legality of a human-controlled house (houses in our engine,
  above). Where this spec gives a rule for a "computer-controlled owner", the rule describes the
  original, and an AI house follows the rule given for human-controlled owners.
- **House skill level:** an integer per house that some automatic behaviours of its units compare
  with game-wide thresholds of the AI settings (below). Every house starts with level 0. In a
  skirmish of the original, every computer-controlled house gets the maximum skill level of the AI
  settings when the houses are set up, and every human-controlled house keeps 0. In a campaign
  mission a house that the map defines takes its level from the map (claim 15). In our engine an
  AI house in a skirmish keeps the level 0 of a human-controlled house, so every rule gated by the
  skill level treats its units as it treats a human player's.
- **Sequence frame count as a delay:** claims 22, 30 and 31 use the frame count of a unit's
  sequence entry (S-0107) unchanged as a number of ticks.
- **Actor:** an infantry, vehicle, aircraft or building object. Only actors have behaviours.
  "Mobile actor" means an infantry unit, vehicle or aircraft.
- **Behaviour:** one of the 32 standing orders in table T1 (Constants). Our names, in alphabetical
  order: *ambush*, *area guard*, *attack*, *attack move*, *capture*, *construction*, *eaten*,
  *enter*, *guard*, *harmless*, *harvest*, *hunt*, *missile*, *move*, *open*, *paradrop approach*,
  *paradrop overfly*, *patrol*, *queued move*, *repair*, *rescue*, *retreat*, *return*,
  *sabotage*, *selling*, *sleep*, *spy-plane approach*, *spy-plane overfly*, *sticky*, *stop*,
  *unload* and *wait*. In this spec a behaviour name in italics always means the behaviour, never
  the sequence kind of the same name in S-0107.
- **Current, queued and suspended behaviour:** three slots each actor holds. Each holds one
  behaviour or *none* (claim 1). The current behaviour is the one the actor carries out. The queued
  behaviour waits to become current. The suspended behaviour is the one an override put aside and a
  revert brings back (claim 10).
- **Processing:** one run of the current behaviour's rule for an actor (claims 2 to 5 and 22 to 33).
  A processing ends by choosing the **processing delay**: the number of ticks until the actor's next
  processing.
- **Processing timer:** the per-actor countdown that the processing delay sets (claim 2).
- **Behaviour start:** the moment a behaviour becomes current through a start-next or a force
  (claims 8 and 9). It restarts the processing timer so that the behaviour is processed at the next
  opportunity (claim 3), and it records the **start tick**, the tick of that start. An override or
  a revert is not a behaviour start and leaves the start tick as it was.
- **Progress count:** a per-actor integer with which some processing rules remember how far they
  have got within the current behaviour (for example a SAM building's aim-then-fire cycle, claim
  32). Every behaviour start sets it to 0; an override or a revert leaves it as it was. Its other
  values are internal to those rules, which state only their observable effects. Two rules look at
  the value itself: claim 13 asks whether a *patrol*'s count is still 0, and claim 32 names the two
  stages (0 and 1) of a SAM building's *attack*, because a value outside them decides an observable
  delay. There is one progress count per actor, shared by all its
  behaviours, not one per behaviour.
- **Ready mark:** a per-building yes/no state that makes a building ready to change (claim 11).
  Some building processings set it (claim 32); a successful per-tick start of the building clears
  it. It is internal state; only its effect on when queued behaviours start is observable.
- **Attack-move order, attack-move memory, eligible for attack-move:** as S-0213 (human orders)
  and S-0214 (order targeting rules) define them. An attack-move order is the order that the
  attack-move gesture gives; the memory is what such an order leaves on a mobile actor (that its
  *move* or *attack* is an attack move, and where to); eligibility is a property of the actor's
  type (infantry or vehicle, with a weapon in its first slot, not marked as preventing attack-move; never
  a building or aircraft).
- **Distributed-fire candidates:** for an actor whose type has the distributed-fire flag, the list
  of candidate targets that its automatic target search collected, over which it spreads
  its fire. The target-acquisition specs own how the list is filled and emptied; this spec uses
  only whether it is empty (claim 24).
- **Behaviour properties:** the eight values per behaviour that the rules data can set: the
  *no-threat*, *paralysed*, *recruitable*, *retaliate*, *scatter* and *zombie* flags, the *rate* and
  the *anti-air rate* (claims 6 and 16 to 21; Import mapping). "The actor's properties" means the
  properties of its current behaviour.
- **Rate delay** *D*<sub>r</sub>: the processing delay derived from the current behaviour's rate
  (Formulas F1).
- **Draw *U*(*a*, *b*):** one ranged draw from the scenario random stream, uniform over the integers
  *a* to *b* inclusive. S-0202 (random streams, a draft in the same batch) owns the generator and
  the reduction to a range; this spec states which draws happen and in which order.
- **Ready to change:** an actor's per-kind condition for letting its queued behaviour become current
  (claim 11).
- **Resting behaviour, falling back to rest:** when an actor has finished what its behaviour asked
  for, because a move arrived or a target died, it *falls back to rest*: it goes through the common effects of
  claim 13, then chooses a resting behaviour by the rules of claims 14 and 15 and queues it. The
  resting behaviours are mostly *guard* and *area guard*. The **placement fall-back** is the
  fall-back to rest that an actor makes at the moment it is placed on the map (created, or leaving a
  factory or a transport). Fall-backs made by orders, processings and damage are ordinary ones; a
  few made by other rules may count as placement fall-backs (Open questions).
- **Target, destination:** an actor's current target object (the thing it attacks, captures or
  enters) and its current destination (a cell or object it moves to). Either may be absent. How they
  are set by orders belongs to the order specs; how a target is chosen automatically belongs to the
  target-acquisition specs (combat topic).
- **Guard spot:** the cell or object an actor in *area guard* stays near (claim 26). An order to
  guard an object or a cell sets it; an actor that starts *area guard* without one takes its own
  cell.
- **Queued destinations:** the list of further destinations a mobile actor has been given in
  advance by orders. When it falls back to rest with no destination and a non-empty list, it takes
  the first entry as its destination (claims 14 and 15). The order specs own how the list is filled.
- **Approach destinations:** a second list of destinations, separate from the queued destinations,
  that a vehicle's own target-approach rule (movement specs) sets up while the vehicle approaches a
  target. Giving the vehicle a new destination empties it. Infantry and aircraft never have entries.
  Its only use here is in the common fall-back effects (claim 13).
- **Scatter request, threat point, forced, move anywhere, tether link:** as S-0208 defines them
  (Terms). A rule of this spec that sends a scatter request always states its threat point (or
  that it has none) and both options.
- **Route:** a sequence of points that an actor has been ordered to visit one after the other
  (a route laid out by the player in advance). The order specs own routes and their points.
- **Contact:** the one pairing that an actor can hold with another object while the two carry out
  an exchange: a vehicle leaving the factory that built it, a harvester with the refinery it
  unloads at, a unit with the transport, repair bay or building it is entering, a vehicle with the
  bunker it occupies. Contact lasts until the exchange ends or one side breaks it off. When contact
  starts and ends belongs to the specs of those exchanges (production, economy, structures and the
  transport specs). A building can hold several contacts at once; the rules of this spec look only
  at a mobile actor's contact, which is one at most. An actor "in contact with nothing" holds no
  such pairing.
- **Bunker:** a building whose type has the bunker flag (Import mapping), which one vehicle at a
  time can occupy. "In a bunker" means occupying one. In this spec "bunker" always means this kind
  of building; buildings that infantry occupy are the *occupiable buildings* below.
- **Absorbing building:** a building whose type has the infantry-absorb flag or the unit-absorb
  flag (Import mapping): a building that takes in units as passengers, for example to draw power
  from them. Whether a given actor may enter a given absorbing building is decided by the building
  (structures specs).
- **Occupiable building:** a building that a given infantry actor may occupy (garrison) at that
  moment. The structures specs own that test; it involves the building type's can-be-occupied
  flag. An **owned occupiable building** of a house is a building of that house whose type has the
  can-be-occupied flag and a tech level of 0 or more (Import mapping); typically a building the
  house could build, rather than a neutral structure placed by a map.
- **Enter-nearest instruction:** one of three standing instructions that a mobile actor can hold,
  apart from its behaviour slots: *enter the nearest absorbing building*, *occupy the nearest owned
  occupiable building* and *occupy the nearest occupiable building*. Only rules of the original's
  computer player give them (team scripts, and the computer's decision on what to do with a unit
  it has just taken over by mind control). No rule of our engine gives them, so in our engine no
  actor holds one and the rules that carry them out (claims 24 and 26, row 3) never apply; they
  are recorded as the original's only (note for AI designers). This spec says only how a
  holder's *guard* and *area guard* processings carry one out in the original (claim 24). An
  actor holds at most one effective instruction at a time for this purpose: when several are
  held, the first in the order just given is the one carried out.
- **Slave master, harvesting slave:** a slave master is a vehicle whose type controls a group of
  infantry harvesters, its *harvesting slaves*, which gather resources for it. The economy topic
  owns their rules, including when the master **wakes** its slaves (sends them out to work). This
  spec only says where the slave rules interleave with behaviours.
- **Guard reset:** a rule that clears an actor's target, destination and guard spot and forces
  *guard* (claim 9).
- **Loaner:** an actor that the game marks as on loan: a transport that brings reinforcements onto
  the map and is not meant to stay. Which actors are marked belongs to the reinforcement and
  aircraft specs.
- **Targeting timer:** a per-actor countdown, separate from the processing timer, that paces the
  automatic target search (claim 33).
- **Rearm time remaining:** the number of ticks until the actor's weapon may fire again after its
  last shot. The firing cycle (planned S-0301) owns the countdown; this spec uses
  only its remaining value.
- **Firing check:** the test that decides whether an actor can fire a given weapon at a given
  target now. Its answer is one of twelve categories, listed here in alphabetical order: *busy*,
  *can fire*, *cloaked*, *illegal*, *impossible*, *moving*, *must deploy*, *not facing*, *out of
  ammunition*, *out of range*, *rearming* and *still turning*. Which situations give which answer belongs to the combat
  topic (the firing cycle, planned S-0301) and to S-0214. This spec uses only the category: claim
  32 says what a building in *attack* does with each one.
- **Armed:** as S-0214 (Terms) defines it: the actor has a weapon in the weapon slot it currently
  uses (its first slot, or, for a type that switches between weapons, the current one). A building
  that holds occupants and whose type lets them fire from inside also counts as armed. Every rule
  of this spec that says "armed" or "unarmed" (claims 11, 14, 15, 21 and 32) asks this one
  question.
- **Primary, secondary and deploy weapon:** an actor's *primary weapon* is the weapon in its first
  weapon slot and its *secondary weapon* the weapon in its second slot, in both cases the elite
  weapon of that slot instead when the actor is elite and its type gives one (the combat and
  veterancy specs own that choice). Its **deploy weapon** is its secondary weapon, or its primary
  weapon when its type has the spray-attack flag (Import mapping). So the deploy weapon of claim 30
  and the secondary weapon of claim 31 are the same weapon except for a type with the spray-attack
  flag. "No deploy weapon" means that slot holds no weapon.
- **Turreted building, voxel turret, turn-rate field:** a *turreted building* is a building whose
  type, or an upgrade installed on it, has the turret flag; its *turret facing* is the facing it
  aims with. It has a *voxel turret* when its own type's art marks the turret as drawn from a voxel
  model. The *turn-rate field* is the type's imported turn rate, an integer (Import mapping); how it
  turns facings belongs to S-0206 (turn rates), and this spec uses it only in the bound of
  Formulas F8.
- **Opportunity target:** a target that the actor's per-tick automatic search chose by itself, as
  opposed to one given by an order (claim 33).
- **Charge target:** a friendly building that an actor with an electric-charge secondary weapon aims
  at to power it up (claim 24). The weapon kind and the building flag are in the Import mapping.
- **Difficulty:** a house's difficulty setting, one of *hard*, *normal* and *easy*, used here as an
  index 0, 1 or 2 in that order into a game-wide list (claim 31; Import mapping).
- **Team member:** an actor that belongs to a team of the original's computer-player script
  system. That system is out of scope for glasnost, so this spec only notes where membership
  changes a rule of the original. An AI house of our engine has no teams (houses in our engine),
  so a rule's exception or condition for team members never applies to its units.
- **Campaign game:** a game that is a campaign mission, as opposed to a skirmish or multiplayer game.
- **2-D distance, 3-D distance:** the distance between two objects' reference points in leptons,
  from the horizontal coordinates only (2-D) or from all three (3-D). The original takes it with the
  *table square root* (the approximate root that approved S-0105, F1, also uses), truncated toward
  zero to an integer, not with the exact root (Formulas F4). This spec states that root itself.
- **Type words.** Words that describe an actor by its type, such as *engineer*, *occupier*,
  *assaulter*, *infiltrator* ("its type infiltrates"), *vehicle thief*, *charge planter* ("has
  explosive charges"), *harvester*, *weeder*, *hover-attack type*, *deployer* ("a type that
  deploys"), "deploys to fire", "immune to radiation", the *undeploy delay*, *guard range* and
  *sight*, and the building words *vehicle-factory building*, *unit repair building*, *SAM flag*,
  *electromagnetic-pulse cannon*, "accepts charges", "invisible", "needs power" and the
  *stupid-guard-mode*, *stupid-hunt*, *close-range*, *distributed-fire* and *opportunity-fire*
  flags, each mean the imported type field of that role in the Import mapping. What those fields
  do outside this spec belongs to the specs that own them.
- **Construction yard:** a building whose type is in the computer player's list of
  construction-yard types (Import mapping). "A vehicle that deploys into a construction yard"
  is a vehicle whose deploys-into type is in that list. Base building belongs to the structures
  and production topics.
- **Automatic base building:** a per-house setting of the original's computer player under which
  it builds its own base. Only rules marked as the original's only (claims 24 and 27) look at it; the
  computer player's rules own it.
- **Mind control:** an actor's capture of an enemy actor, which then changes owner to the
  capturer's house. The combat topic owns it; this spec uses only that a takeover gives the
  captured actor a guard reset (claim 9).
- **Being erased:** the state of an object that an erasing weapon (a temporal weapon) has locked
  onto, from the moment the first such weapon locks on until the last one lets go or the object is
  removed. The combat topic owns erasing weapons.
- **Chrono move, leaving stage, phase-in:** a chrono move teleports an actor to another place, by a
  superweapon or by its own teleporting locomotor. Its **leaving stage** is the first part of the
  move, from the moment the jump begins until the actor leaves its old place (its position
  changes to the new place). Its **phase-in** is the part that follows, during which the actor
  reappears at its new place. The superweapon and locomotion topics own the move, its parts and
  their lengths.
- **Tunnel:** a pair of linked tunnel entrances on a map through which infantry and vehicles
  travel out of sight to the other entrance. The movement topic owns it.
- **Radiation level:** the integer strength of radiation at a cell, which radiation weapons
  leave and which fades over time (combat topic). A weapon's *radiation level field* is the
  level it leaves (Import mapping).
- **Veterancy rank, veteran, elite:** an actor's *veterancy rank* is the experience rank that the
  veterancy specs give it; *veteran* and *elite* name ranks of those specs. This spec uses the
  words only as pointers: for the elite weapon (Terms, *primary, secondary and deploy weapon*), for
  the veterancy abilities below and in scenario B4. The veterancy specs own the ranks and how an
  actor reaches them.
- **Veterancy ability:** a capability that an actor's type grants it once it reaches a given
  veterancy rank, listed per type in the veterancy ability lists. This spec uses the guard-area,
  explosive-charges and scatter abilities; the veterancy specs own the lists.
- **AI settings, area-guard threshold:** the *AI settings* are the game-wide rules values that
  set the computer player's intelligence levels; among them the *maximum skill level* and the
  *area-guard threshold*, an integer house skill level. A house whose skill level is below the threshold
  rests its units in *guard* rather than *area guard* in the cases of claims 14 and 15 (Import
  mapping).
- **Deactivated, reactivated:** an actor is *deactivated* when a rule of another topic switches
  it off (for example a vehicle disabled by an electromagnetic pulse, or one that needs a
  control building its owner has lost), and
  *reactivated* when that rule switches it on again.
- **Open-topped transport:** a transport (an actor that carries passengers) whose passengers can
  fire from inside it. The transport specs own both.
- **Unload or deploy cycle:** a vehicle's cycle of unloading or deploying, as the deploy specs
  define it. This spec uses only whether it has finished (claim 11) and that a vehicle's *move*
  and *patrol* processings restart an unfinished one (claims 22 and 34). What the cycle is and how
  long it lasts belong to the deploy specs (Open questions).

## Behaviour

### Holding and processing behaviours

1. **Three slots per actor.** Every actor has a current, a queued and a suspended behaviour. A newly
   created actor has *none* in all three. Other objects (projectiles, animations, overlays) have no
   behaviour.
   - Backing: `measurable` (scenario B1, reading the slots with the recorder)
   - Confidence: high

2. **When an actor processes its behaviour.** On each tick on which an actor's own update runs (its
   turn in the turn pass of S-0203, tick order, a draft in the same batch, which says which objects
   take turns, in which order, and when an object loses a turn), the actor
   processes its current behaviour if, and only if, all of these hold:
   - the actor is alive and its health is above 0,
   - its processing timer has expired,
   - it is not an infantry unit or vehicle travelling through a tunnel,
   - it is not an infantry unit or vehicle that is being erased, or that is in the leaving stage of a
     chrono move (Terms). Such an actor also drops its target and destination on each such tick.

   The per-tick start of claim 11 is skipped on the ticks on which the tunnel, erasing or
   leaving-stage exclusion holds (the last two bullets above), and on no other tick without a
   processing: on a tick on which only the timer has not expired, the per-tick start still runs
   (claim 3a, scenario B3, case k). The phase-in of a chrono move does
   not stop processing: an actor processes during its phase-in as at any other time. One edge (low
   confidence): when a chrono move reaches its phase-in while the actor is being erased, the
   exclusion ends for an infantry unit at once and for a vehicle when the phase-in ends, although
   the erasing goes on.

   A processing on tick *t* that chooses the delay *d* sets the timer so that it has expired at
   any later turn of the actor on a tick *t*′ with *t*′ − *t* ≥ *d*. The actor processes again on
   the first tick *t*′ > *t* with *t*′ − *t* ≥ *d* on which its update runs, so a delay of 1 or
   less (0 and negative delays included) means "again on the next tick". A turn processes at most
   once. An actor normally has one turn per tick; when S-0203 (claim 11) gives it a second turn
   in the same tick, that second turn processes again only if the delay chosen in its first turn
   was 0 or less (or a behaviour start, claim 3, has set the timer to "expired now" since).
   - Backing: `measurable` (scenarios B1 and B18). The second-turn case is `unmeasured`: S-0203
     marks the second turn itself as not observable by a recorder. The erasing edge is
     `unmeasured`: it needs an erasing weapon locked onto an actor at the moment its chrono move
     reaches the phase-in, and no setup is known that can time that.
   - Confidence: high (alive, health and timer); medium (the tunnel, erasing and leaving-stage
     exclusions, that the phase-in does not exclude, and that no other per-kind exclusion exists
     for aircraft); low (the second-turn case and the erasing edge)

3. **A behaviour start is processed at once, unless the processing that caused it chooses later.**
   A behaviour start (start-next, claim 8, or force, claim 9) sets the processing timer to "expired
   now". So:
   - (a) a start that happens on tick *t* before the actor's processing for tick *t* leads to a
     processing of the new behaviour on tick *t*. For infantry, vehicles and buildings, the
     per-tick start of a queued behaviour (claim 11) happens before the processing of the same tick,
     so a queued behaviour that starts there is processed on the same tick,
   - (b) a start that happens after the actor's processing for tick *t* (for example one caused by
     another actor's update later in the tick, or a building's second per-tick start, claim 11) is
     processed on tick *t* + 1 (or in a second turn of the same tick, claim 2),
   - (c) a start that a processing itself causes is overridden by the delay that processing then
     chooses: the new behaviour is first processed when that delay runs out. Each claim below that
     starts a behaviour from within a processing says which delay it chooses.

   An override or a revert (claim 10) does not touch the processing timer: the behaviour it makes
   current is first processed when the timer set by the last processing runs out.
   - Backing: `measurable` (scenario B3, cases a and k, for the reset by a start; scenario B7,
     case b, for the override leaving the timer alone)
   - Confidence: high (timer resets); medium (that no other rule touches the timer)

4. **Behaviours without a processing rule.** For the behaviour and actor-kind pairs marked "—" in
   table T1, a processing does nothing and chooses a delay of exactly 450 ticks, with no draw. This
   holds for *sleep*, *harmless*, *ambush*, *return*, *stop* and *wait* for every actor kind. An
   actor whose current behaviour is *none* processes in the same way.
   - Backing: `measurable` (scenario B1, for the six behaviours and the infantry, vehicle and
     aircraft kinds). The building cases are `unmeasured`: a map's building entries carry no
     behaviour, and no order, trigger or other input is known that puts a building into one of
     these behaviours. The *none* case is `unmeasured`: a mobile actor gets a resting behaviour
     through its placement fall-back (Terms), an infantry unit with neither a current nor a queued
     behaviour falls back to rest before its processing (claim 11), and no setup is known that
     leaves *none* current at any actor's processing.
   - Confidence: high

5. **Behaviours that share another behaviour's processing.** For every actor kind:
   - *sticky* is processed exactly as *guard* (claim 25 says what makes it differ),
   - *sabotage* is processed exactly as *capture* (claim 28),
   - *queued move* and *attack move* are processed exactly as *sleep*: nothing happens and the delay
     is 450 ticks (`quirks.queued_move_sleeps`).

   No rule of the original's program makes *queued move* current; only data that names it can (for
   example an object that a map places with that behaviour). *Attack move* becomes current in the
   same way, and also through the attack-move orders that S-0213 (claim 6) turns into *attack move*
   itself (claim 12). As far as studied, player input never creates those orders (S-0213, claim 6),
   so in practice only data puts an actor into *attack move*.

   A shared processing still reads the properties of the behaviour that is current: an actor in
   *sticky* uses the rate (and, for a building, the anti-air rate) of *sticky*, not of *guard*, and
   an actor in *sabotage* uses the rate of *sabotage* (claim 6).
   - Backing: `measurable` (scenario B14 for *queued move* and *attack move*; scenario B22 for
     *sticky* and *sabotage* and for the rate each reads). For buildings the shared processing of
     *sticky* and its anti-air rate are `unmeasured`: a map's building entries carry no behaviour
     and no known input puts a building into *sticky*.
   - Confidence: high (the shared processing); medium (that no other rule of the program makes
     either behaviour current)

6. **The rate delay and the standard delay.** Most processings end with the **standard delay**
   *D* = *D*<sub>r</sub> + *U*(0, 2): the rate delay of Formulas F1, computed from the current
   behaviour's rate in game minutes at 900 ticks per game minute and truncated toward zero, plus one
   draw. Table T2 lists, for the general-purpose processings of mobile actors and the infantry
   rules of claims 23 and 27, which delay each outcome chooses and how many draws it takes; the
   claims give the other kind-specific outcomes. A delay
   takes a draw only where its formula shows one (*U*(*a*, *b*)). Every other delay of this spec,
   whether a fixed number of ticks (1, 2, 100, 450), a frame count, the undeploy delay, the rearm
   time remaining, 0 or *D*<sub>r</sub> alone, takes no draw for the delay itself; a target search
   that the same processing runs still takes its own draws (T2). Because the rate is held in single
   precision (S-0003, claim 6) and the delay is truncated, a rate that is written as a short
   decimal can give one tick less than the decimal times 900: a rate written as 0.06 gives
   *D*<sub>r</sub> = 53, not 54 (Formulas F1). The shortfall comes from the single-precision
   reading alone: the product of a single-precision rate with 900 is exact (S-0205, claim 9), so
   it does not depend on the rounding mode. Only a rate written with a percent sign is a
   double-precision product, and its product with 900 depends on `quirks.chop_rounding` (S-0205).

   The rate is read from the behaviour that is current at the moment the delay is chosen. When a
   processing has itself made another behaviour current (claim 3c) and then chooses a delay from a
   rate, it uses the rate of the behaviour it has just made current. Among the processings that
   this spec describes in full (claims 22 to 28 and 30 to 32), those that can do this are:
   - *hunt*, whose general with-target outcome can start *capture* or *sabotage*, and whose
     campaign rule (original only) can start *move* (claim 27),
   - *guard* and *area guard* carrying out one of the two occupy instructions (original only), which can start
     *capture* (claims 24 and 26, row 3).

   A behaviour that a processing only queues, without making it current, leaves the rate as it
   was. Each of these starts is readiness-dependent: when the actor is not ready to change, the
   new behaviour stays queued and the rate is that of the behaviour still current. The infantry
   rules of claims 23 and 27 that end with 1 tick take no rate at all.

   **The delay's draw comes last.** Within one processing, the draw of the delay itself (the
   *U*(*a*, *b*) of its formula, where it has one) is the last draw the processing takes from the
   scenario random stream. Every other draw of the same processing comes before it: the draws of a
   target search that the processing runs (claims 26 and 27, including the search's own
   *U*(0, 2) for the targeting timer, claim 33), and the draws of the rules it applies, such as
   approaching a target, an idle action or a scatter request, whose number belongs to their own
   specs. So in an *area guard* processing whose search runs, the targeting timer's *U*(0, 2) is
   drawn before the delay's *U*(1, 5).
   - Backing: `measurable` (scenario B2 for the rate delay and the standard delay; scenarios B11
     case e and B21 for the rate of a behaviour the processing has started, and scenario B22 for the rate of the
     current behaviour under a shared processing; scenarios B9 case b, B11 case b and B21 for fixed
     delays taking no draw; scenario B27, case d, for the delay's draw coming after the search's
     draw in *area guard*). That the delay's draw also follows the search of *hunt* and the draws
     of approaching, idle actions and scatter requests is `unmeasured`: the normal search of *hunt*
     looks at candidates beyond any fixed reach, and the target-acquisition specs, which own the
     draws it takes per candidate, do not state them, nor do the specs of the other rules state
     how many draws they take, so a scenario cannot tell which drawn value went to which rule.
   - Confidence: high

### Changing behaviours

7. **Queueing a behaviour.** A request to queue behaviour *b* (with or without an immediate start)
   has the effect of the first row that matches:

   | Situation | Queued behaviour afterwards | Immediate start, when asked for |
   |---|---|---|
   | the current behaviour is *selling*, or it is *wait* and *b* is *guard* | unchanged | none: the request is ignored entirely |
   | *b* is *none* | unchanged | a start-next (claim 8) if the actor is ready to change (claim 11) |
   | *b* is the current behaviour, and nothing or *b* itself is queued | unchanged | as above |
   | any other case (*b* differs from the current behaviour, or a behaviour other than *b* is queued) | *b*, replacing any queued one | as above |

   Queueing the current behaviour while nothing is queued changes nothing; queueing the current
   behaviour while another one is queued replaces the queued one with it, so the next start
   restarts the current behaviour.

   Some rules instead queue a behaviour and then make it current with a start-next that does not
   look at readiness. The claims that describe such a rule say that it starts the behaviour "at
   once, whether or not the actor is ready to change"; "with an immediate start" always means the
   readiness-dependent start of this claim; and a rule that says only "queues" asks for no
   immediate start (Terms, queueing wording).
   - Backing: `measurable` (scenario B3; case l for the replacement of a queued behaviour, case i
     for the refusal under *wait*)
   - Confidence: high

8. **Start-next.** If a behaviour is queued, it becomes current, the queue becomes empty, and the
   behaviour start of claim 3 happens: the timer expires now, the start tick becomes the current
   tick, and the progress count becomes 0. The suspended behaviour is not touched. If nothing is
   queued, nothing happens.
   - Backing: `measurable` (scenario B3)
   - Confidence: high

9. **Force.** A forced behaviour *b* becomes current at once, whether or not the actor is ready to
   change, unless the current behaviour is *wait* and *b* is *guard* (then nothing happens). The
   queue becomes empty and the behaviour start of claim 3 happens. Unlike queueing and overriding,
   force is not refused while the current behaviour is *selling*: a forced behaviour replaces
   *selling* (no stock rule is known to force a behaviour on a selling building). The suspended
   behaviour is not touched.

   Rules that force a behaviour include the guard reset (Terms), which the original applies to an
   actor taken over by mind control, to an actor that is deactivated or reactivated, to an actor
   that enters an open-topped transport, to an actor whose locomotor is replaced by another,
   and to harvesting slaves through the slave rules (economy topic); and the processings of claims 23 and 27 (their original-only rules), which force *capture*, and of claim 30, which forces *guard*. No
   player's order is known to force a behaviour on the actor it is given to (claim 12); the stop
   order given to a slave master gives its harvesting slaves a guard reset.
   - Backing: `measurable` (scenario B3, cases f and i)
   - Confidence: high (the rule); medium (the list of rules that force)

10. **Override and revert.** An override puts a temporary behaviour *b* in place with a new target
    and destination, keeping enough to come back:
    - in every case the actor's target and (for mobile actors) destination are put aside and
      replaced by the override's,
    - the slots do not change if the current behaviour is *selling*, or the current behaviour is
      *wait* and *b* is *guard*; the target and destination are replaced all the same
      (`quirks.refused_override_retargets`),
    - otherwise the suspended behaviour becomes the queued behaviour if one is queued, else the
      current behaviour (`quirks.override_suspends_queued`); *b* becomes current; the queued
      behaviour stays queued.

    A revert does nothing unless a behaviour is suspended. If one is, it becomes current, the
    suspended slot becomes empty, and the target and destination put aside by the override come
    back. Neither an override nor a revert restarts the processing timer (claim 3), the start tick or
    the progress count. A start-next or a force leaves the suspended slot as it is.

    The original overrides when an actor retaliates against an attacker (claim 18; the override is
    to *attack* with the attacker as target and no destination). It reverts, if an override is in
    effect, when the actor's target object ceases to exist (it is destroyed or removed from the map)
    or becomes untargetable. Clearing the target by any other rule (the leash of claim 26, an order)
    does not revert. Other users (team logic, some movement situations) are outside this spec.
    - Backing: `measurable` (scenario B7)
    - Confidence: high (the slot rules and the target replacement); medium (the list of users and
      of revert causes)

11. **Ready to change, and the per-tick start.** On each tick on which its update runs, an infantry
    unit, vehicle or building that is ready to change and has a behaviour queued makes it current
    by a start-next, its **per-tick start**. The behaviour that this start makes current is the one
    processed on that tick (claim 3a). For a building this start also needs the building to have
    left its construction state. A building also has a second per-tick start on the same tick,
    after its processing and without the construction-state condition, so a behaviour that the
    second start makes current is first processed on the next tick (claim 3b); see the building
    rule below. Aircraft have no such general per-tick start: their queued behaviours
    start through aircraft rules (aircraft spec), using the readiness below. An infantry unit that
    is ready to change and has neither a current nor a queued behaviour falls back to rest (claims
    13 and 14) at its per-tick start, and the behaviour that this fall-back queues, if any, becomes
    current at that same per-tick start. Readiness depends on the actor kind:
    - **Infantry** is ready unless any of these holds: its current behaviour is *sticky* or
      *rescue*; it is in the middle of firing; it is falling (for example from a parachute); it is
      moving on this tick while its current behaviour is not *guard* or *hunt* and is not *attack*
      without a target; or its current sequence (S-0107) is a kind that is not interruptible.
    - **Vehicles** are ready unless any of these holds:
      - the current behaviour is *sticky* or *rescue*,
      - the vehicle is in one of exactly three transitions: it is deploying, it is undeploying
        (the deploy specs own when each begins and ends), or it is in the middle of letting its
        passengers out under its own *unload* processing (the transport specs own that
        processing). A vehicle that has finished deploying and stays deployed is not held back
        by this condition,
      - the vehicle is moving on this tick and is not below the surface (its height above the
        surface is not negative, so a vehicle travelling underground does not meet this
        condition), except in three cases: the queued behaviour is *enter*; the current behaviour
        is *guard*, or *attack* without a target; or the queued behaviour is a *guard* that one of
        two *unload* rules queued with an allowance to interrupt movement. Those two rules belong to other specs: a transport
        vehicle that reaches the last stage of unloading its passengers queues *guard* (transport
        specs), and a jumpjet vehicle in *unload* that is sent off again while it descends to land
        queues *guard* (movement specs). The allowance lasts until the slots next change (any
        queue that takes effect, start, force, override or revert ends it),
      - its unload or deploy cycle has not finished,
      - it is in contact with a vehicle-factory building and the queued behaviour is neither *move*
        nor *enter*,
      - it is in contact with nothing and stands on a cell of a vehicle-factory building that is the
        cell (*x*, *y* + 1), where (*x*, *y*) is the top-left cell of that building's foundation
        (the cell where a new vehicle leaves the factory).
    - **Aircraft** are ready unless the current behaviour is *sticky* or *rescue*, or the aircraft is
      in a flight phase that does not allow it (the aircraft spec owns the phases).
    - **Buildings** are ready while their ready mark (Terms) is set. Every processing of an armed
      building in *guard*, *sticky* or *area guard*, and every processing of *attack* by a building
      without the SAM flag that has a target, sets the mark (claim 32); rules of the structures
      specs (the construction state, repair, selling and others) also set or clear it. Each of the building's two
      per-tick starts clears the mark when it succeeds; nothing else of this spec clears it. So a
      behaviour queued for a building becomes current at the first of its per-tick starts at which
      the mark is set (and, for the first start of a tick, the building has left its construction state); this
      can be the second start of the very tick whose processing set the
      mark, and a behaviour made current by that second start is first processed on the next tick
      (claim 3b). The starts that a building's own processing makes (claim 32) are made whether or
      not the mark is set and leave it as it is. Consequence: an armed building that has processed
      *guard* at least once since its last successful per-tick start takes a queued behaviour (for
      example an ordered *attack*) at its next per-tick start, without waiting for its next
      processing.
    - Backing: `measurable` (scenario B3, cases b, c, g and h; scenario B23 for the vehicle
      exceptions of queued *enter* and contact with a vehicle-factory building, and case c for a
      transport letting its passengers out; scenario B16, case d, for buildings). Further parts are
      `unmeasured`. The deploying and undeploying transitions and the unfinished unload or deploy
      cycle: which vehicles have them, how long they last and how they end belong to the deploy
      specs, and no setup in this spec can time an order inside them (as for claim 22). The
      vehicle below the surface: no vehicle of the
      stock game travels below the surface, and no setup is known that puts one there. The two
      *unload* rules with the allowance to interrupt movement: they fire only at the last stage of
      an unloading or of a descent, which belong to the transport and movement specs, and no setup
      is known that has the vehicle moving at that instant. Aircraft readiness: the flight phases
      belong to the aircraft spec, and no scenario here tests them (Open questions). The building's
      second per-tick start is `unmeasured`: it is observable only when a behaviour
      is queued while the mark is clear and the same tick's processing sets it, and no input or
      data sequence that produces this is known (buildings take only attack orders, and an order
      for the current behaviour changes no slot).
    - Confidence: high (infantry, and the vehicle conditions on *sticky* or *rescue* and on
      moving); medium (the list of three vehicle transitions, the unload or deploy cycle, the
      vehicle-factory conditions and buildings); low (the two *unload* rules, the vehicle below the
      surface and aircraft)

12. **Orders go through the queue.** A player's order that names a behaviour (move, attack, guard,
    area guard, enter, capture and the other orders that carry a behaviour), carried by the order
    system, queues the ordered behaviour without an immediate start; the order also sets the
    target, destination and guard spot it carries. The behaviour therefore becomes current at the
    actor's next per-tick start (claim 11; for aircraft, through the aircraft rules that claim 11
    points to), that is, at the actor's first update after the order takes effect if the actor is
    ready to change then, otherwise at the first later update at which it is. In a single-player
    game (a skirmish or a campaign mission) an order takes effect after the turn pass of one tick
    and before any object's turn in the next (S-0203, claim 7), so this is the actor's update on
    the next tick at the earliest; network games are out of scope here, as in S-0203. An order for the behaviour
    that is already current changes no slot (claim 7); its new target or destination takes effect
    directly.

    An attack-move order is first turned into a plain behaviour order by S-0213 (human orders,
    claim 6), and the behaviour that results is queued as above. In short: a mobile actor whose
    type is eligible for attack-move gets *move* (with the attack-move memory) if the order carries
    a destination, else *attack* if it carries a target, else *attack move* itself (claim 5); an
    ineligible mobile actor gets *attack* if the order carries a target, else *move*; a building
    gets *attack move* itself. S-0213 owns this translation and its backing; this spec adds only
    that the result goes through the queue like any other ordered behaviour.

    Every such order also forgets the target and destination that an override put aside (claim 10),
    but leaves the suspended slot as it is. For a mobile actor it clears the guard spot and the queued
    destinations, except an area-guard order, which takes its object or cell as both the guard spot
    and the destination and clears the target.

    The **stop order** is not a behaviour order: it changes no slot and forces nothing. It clears
    the actor's target, destination, queued destinations and route, ends its contacts, and
    leaves its guard spot alone. Two exceptions: a harvester vehicle whose current behaviour is
    *harvest* or *return* queues *guard* and starts it at once, whether or not it is ready to
    change, and a slave master also gives its harvesting slaves a guard reset. The stop order does nothing to an actor whose current behaviour
    is *construction* or *selling*. As a consequence, an actor in *move* or *attack* that has been
    stopped falls back to rest at a later processing (claims 22 and 23), once it has stopped moving.
    The order specs own the stop order's further conditions.
    - Backing: `measurable` (scenario B3; scenario B14, case b, for the two attack-move orders that
      input creates). The attack-move orders that input cannot create (an order with neither target
      nor destination, an ineligible actor, a building) are `unmeasured` here: the measurement setup
      cannot issue them, and S-0213 owns their translation.
    - Confidence: high (queue without start; the stop order changes no slot); medium (the effect on
      a moving vehicle, which follows from claim 11, the clearing rules and the stop order's
      exceptions)

13. **Falling back to rest: common effects.** Whenever an infantry unit, vehicle or aircraft falls
    back to rest, the effects of this table take place, and the tables of claims 14 and 15 (for
    aircraft, the aircraft spec's own rules) decide on the state they leave. In particular, a
    destination that one of these effects gives counts as the actor's destination there, so that the
    table's destination row applies.

    | Situation | Effect |
    |---|---|
    | its own locomotor has been temporarily replaced by another (for example while it is carried by another object) | it gets its own locomotor back (confidence low) |
    | it is a vehicle with approach destinations | the first of them becomes its destination and is removed from the list; the route row and the guard-spot row do not apply in this fall-back |
    | it follows a route, the approach row does not apply, and it is not in a *patrol* that has already begun (progress count not 0) | it heads for the next point of the route |
    | it is an infantry unit that is not a harvesting slave, its guard spot is on another cell, the approach row does not apply, and it has not got its own locomotor back in this fall-back (confidence low) | the guard spot becomes its destination; unless its current behaviour is *area guard*, it also queues *move* and forgets the guard spot |

    Where several rows give a destination, the one the actor is left with is the guard spot if the
    guard-spot row applies; otherwise the next point of the route if the route row applies; otherwise
    the first approach destination if the approach row applies. The common effects never send a
    scatter request. The scatter that can follow a fall-back to rest in *capture*, *sabotage* and
    *eaten* is sent by that processing (claim 28), not by the fall-back.

    These effects take place at most once per actor per tick: in a second fall-back to rest on the
    same tick none of them happens, and only the table of claim 14 or 15 applies. An actor using a temporal
    weapon also lets its victim go, and an actor with attack-move memory may resume its attack move; the
    combat and order specs own those effects.
    - Backing: `measurable` (scenario B15: case a for the guard-spot row, case b for the route
      row). The locomotor row and the approach row are `unmeasured`: which situations replace an
      actor's locomotor for a while, and which fill a vehicle's approach destinations, are not
      established (Open questions), so no setup is known that puts an actor in either situation
      at a fall-back to rest.
    - Confidence: medium (the effects and which destination remains); high (that no scatter request
      is sent); low (the locomotor row and its effect on the guard-spot row, and which rules fill
      the approach destinations)

14. **Falling back to rest: infantry.** When an infantry unit falls back to rest, after the common
    effects of claim 13 the result is the first matching row:

    | # | Situation | Result |
    |---|---|---|
    | 1 | the unit has attack-move memory, or its current behaviour is *wait* | nothing changes |
    | 2 | it has a target | queue *attack*, except that a current *sabotage* or *capture* is queued again instead |
    | 3 | it has no destination and its queued destinations are not empty | the first queued destination becomes its destination (the list may keep it at the end, for a looping route), then row 4 |
    | 4 | it has a destination | queue *move*, except that a current *sabotage* or *capture* is queued again instead |
    | 5 | its current behaviour is *guard* or *area guard* | nothing changes |
    | 6 | its current behaviour has the zombie or the paralysed property | nothing changes |
    | 7 | **original only:** computer-controlled owner, not a team member, and the owner's house skill level is below the area-guard threshold | queue *guard* |
    | 8 | **original only:** computer-controlled owner, not a team member, and the unit is a harvesting slave | queue *guard* |
    | 9 | **original only:** computer-controlled owner, not a team member, and the unit is armed, an engineer or a vehicle thief | queue *area guard* |
    | 10 | **original only:** computer-controlled owner, not a team member | queue *guard* |
    | 11 | human-controlled owner, not a team member, and the unit has the guard-area veterancy ability or its type defaults to area guard | queue *area guard* |
    | 12 | otherwise (human-controlled owner, or a team member) | queue *guard* |

    Whatever the row, nothing is queued while the current behaviour is *patrol* or *area guard*: an
    infantry unit in *area guard* that falls back to rest with a target does not queue *attack*.
    Every "queue" of this table is a queue without an immediate start (Terms, queueing wording).

    **In our engine** rows 7 to 10 never match and no unit is a team member (Terms, houses in our
    engine), so the infantry of every house, AI houses included, rests by rows 11 and 12: in
    *area guard* if it has the guard-area veterancy ability or its type defaults to area guard,
    otherwise in *guard*.

    **Original only (note for AI designers).** In the original, an infantry unit of a
    computer-controlled house that is not in a team rests in *area guard* when it is armed, an
    engineer or a vehicle thief, and in *guard* otherwise; it rests in *guard* in every case when it
    is a harvesting slave or when the house's skill level is below the area-guard threshold (rows
    7 to 10). Our AI can get the same result by ordering its resting infantry to guard an area.
    - Backing: `measurable` (scenario B4; row 4 by scenario B15 case c). Three parts are `unmeasured`:
      row 3, the queued destinations, which the order rules fill (S-0213) and for which no input
      is known here, row 1's attack-move memory, whose life belongs to S-0213 and S-0214 (it is cleared and set by
      the order rules, so a fall-back test here could not tell the memory from an ordinary
      stop), and row 8, the harvesting slave of a computer-controlled owner, which is the
      original's only and which only the economy topic's units can be put into.
    - Confidence: high (rows 1 to 6, 11 and 12); medium (rows 7 to 10)

15. **Falling back to rest: vehicles.** When a vehicle falls back to rest, after the common effects of
    claim 13 the result is the first matching row:

    | # | Situation | Result |
    |---|---|---|
    | 1 | it has attack-move memory, or its current behaviour is *wait* | nothing changes |
    | 2 | no destination and non-empty queued destinations | the first queued destination becomes its destination, then row 3 |
    | 3 | it has a destination | queue *move* |
    | 4 | it is a harvester type (ore or weeds) | if it is in contact with another object, or *harvest* is current or queued: nothing changes, and target and destination are kept; otherwise target and destination are cleared and it queues *harvest*, except that in an ordinary fall-back a human-controlled harvester that does not stand on its resource's land type (ore for ore harvesters, weeds for weeders) queues *guard* instead; in the placement fall-back the land type is not looked at |
    | 5 | unarmed, a loaner with passengers, not a team member | queue *unload* |
    | 6 | unarmed and deploying; or unarmed with *unload* current and a deploys-into type; or unarmed with *area guard* current | nothing changes |
    | 7 | unarmed | clear target and destination, queue *guard* |
    | 8 | armed, with *guard* or *area guard* current | nothing changes |
    | 9 | armed, current behaviour has the zombie or the paralysed property | nothing changes |
    | 10 | armed, the owner's house skill level is below the area-guard threshold, and the vehicle has neither the guard-area veterancy ability nor a type that defaults to area guard | queue *guard* |
    | 11 | armed, and a team member, or a harvesting slave, or a slave master | queue *guard* |
    | 12 | armed, otherwise | queue *area guard*, or *guard* if the vehicle sits in a bunker |

    No row queues anything while the current behaviour is *patrol*, *area guard*, *unload* or
    *eaten*, or while it is *enter* towards a bunker it is in contact with and the vehicle does not
    yet stand on that bunker's cell; the clearing of target and destination in rows 4 and 7 still
    happens then. Every "queue" of this table is a queue without an immediate start. Unlike
    infantry, rows 10 to 12 do not look at whether the owner is human-controlled; only the owner's
    house skill level counts. In skirmish and multiplayer games of the original a human-controlled
    house has skill level 0 and every computer-controlled house has the maximum skill level of the
    AI settings; in a campaign mission each house has the level its map gives (absent: 0, a value
    above the maximum: 1). So, whenever the area-guard threshold is above 0, an armed vehicle of a
    human-controlled owner in skirmish rests in *guard* unless it has the guard-area ability or its
    type defaults to area guard.

    **In our engine** the rows apply to the vehicles of every house as written, with these
    consequences of the houses rule (Terms):
    - row 4's exception for a harvester that does not stand on its resource's land type applies to
      the harvesters of every house, AI houses included,
    - an AI house has the house skill level of a human-controlled house (0 in a skirmish, Terms
      *house skill level*), so its armed vehicles rest as a human player's do,
    - no vehicle is a team member, so the team-member parts of rows 5 and 11 never hold.

    **Original only (note for AI designers).** In the original, a harvester of a computer-controlled
    house queues *harvest* at an ordinary fall-back wherever it stands (row 4); the armed vehicles
    of a computer-controlled house in skirmish rest in *area guard* whenever the maximum skill level
    is at or above the area-guard threshold (rows 10 and 12); and an armed team member rests in
    *guard* (row 11). Our AI can get these results by ordering its harvesters to harvest and its
    resting vehicles to guard an area.
    - Backing: `measurable` (scenario B4; row 3 by scenario B15 case b). Four parts are
      `unmeasured`: row 2, the queued destinations (see claim 14), row 1's attack-move memory
      (see claim 14), row 5, because which actors are
      loaners is not established (Open questions) and so no setup is known that has a loaner with
      passengers fall back to rest, and the slave-master part of row 11, which is the economy
      topic's unit.
    - Confidence: medium (the rows and the skill level of houses); low (the loaner row)

### Behaviour properties

16. **No-threat.** While an actor's current behaviour has the no-threat property, no other actor's
    automatic target search picks it as a target. When a mobile actor that is not a team member, with
    the no-threat property but not the zombie property, takes damage from a known attacker and
    survives, it falls back to rest at once (claims 13 to 15), which normally ends the no-threat
    behaviour.
    - Backing: `measurable` (scenario B5)
    - Confidence: high (the search exclusion and the fall-back); medium (the exact damage outcomes
      that trigger the fall-back: any damage that changes health and leaves the actor alive)

17. **Zombie and paralysed.** An actor whose current behaviour has the zombie or the paralysed
    property never leaves it by falling back to rest (claims 14 and 15, rows 6 and 9). Orders and
    other rules can still replace it through the queue or by force, subject to readiness (claim
    11). In addition:
    - a vehicle with the paralysed property refuses every scatter request that has a threat point,
      forced or not. A request without a threat point is not checked this way, and infantry and
      aircraft have no such check. The request that damage sends to a vehicle never has a threat
      point (claim 19), so the paralysed property never stops a vehicle's scatter from damage. The
      threat-point request of S-0208 claim 15 (request B) is sent to infantry only. Other rules
      can send a vehicle a request with a threat point. For example, once a unit that is entering
      a bunker has stopped on the bunker's cell, the unit nearest the centre of each of the
      bunker's cells, other than the entering unit, is sent a forced request, not move-anywhere,
      with the bunker's position as threat point. The structures specs own that rule,
    - the no-threat fall-back of claim 16 does not happen while the zombie property holds.
    - Backing: `measurable` (scenario B6: parts a and b for the fall-back rows and the damage
      scatter of a paralysed vehicle, part c for the refusal of a request with a threat point, and
      scenario B5, case b, for the zombie property suppressing the no-threat fall-back)
    - Confidence: high for the fall-back rows and for the request from damage having no threat
      point; medium for the refusal of a request with a threat point; low for the bunker rule as an
      example

18. **Retaliate.** An actor retaliates against an attacker only while its current behaviour has the
    retaliate property, among the other retaliation conditions that the target-acquisition specs
    own. Three of those conditions concern behaviours and are stated here:
    - a mobile actor of a human-controlled owner retaliates only while its current behaviour is
      *guard*, *area guard* or *patrol* (not *sticky*), unless the game-wide "player return fire"
      setting is on (Import mapping); buildings are not restricted this way,
    - an actor of a human-controlled owner that already has a target never retaliates,
    - retaliation happens as an override to *attack* with the attacker as target (claim 10). An actor
      of a computer-controlled owner, or one whose weapon already reaches the attacker, overrides at
      once; an actor of a human-controlled owner that cannot reach the attacker overrides only if
      its 3-D distance to the attacker is at most (*S* + 0.5) × 256 leptons, where *S* is its
      type's sight in cells (Formulas F7).

    **In our engine** the three conditions for human-controlled owners apply to the actors of every
    house (Terms, houses in our engine). **Original only (note for AI designers):** in the
    original, none of the three conditions holds back a mobile actor of a computer-controlled
    house: its behaviour list, a target it already has and its distance to the attacker do not
    stop it from retaliating, and it overrides at once. The other retaliation conditions, owned by
    the target-acquisition specs, still apply to it.
    - Backing: `measurable` (scenario B7, case g for an actor that already has a target)
    - Confidence: high (the property and the behaviour list); medium (the distance rule and the
      rule about an existing target)

19. **Scatter.** The scatter property acts on scatter requests in two ways. It decides whether
    some requests are accepted, and the scatter from damage below tests it before it sends one.
    Acceptance depends on the kind of actor:
    - A vehicle refuses a request that is not forced unless its current behaviour has the scatter
      property. The property does not decide about a forced request. The vehicle's other
      acceptance conditions, and where it then goes, belong to the vehicle movement specs, which
      do not state them yet (Open questions).
    - For an infantry unit, S-0208 claim 11 gives the whole acceptance rule. There a forced request
      counts as not forced while the unit's locomotor is moving, so the property can refuse it then.
    - What an aircraft does with a scatter request belongs to the aircraft spec.

    **Scatter from damage.** The scatter from damage of an infantry unit is owned by S-0208
    (claim 15, request A, with the acceptance and destination in its claims 11 to 13); this
    claim does not restate it. For every other mobile actor (a vehicle or an aircraft), when it
    takes damage and survives it, it is sent one scatter request when either of these cases
    holds. The request has no threat point, is forced and is not move-anywhere.
    - *Case 1.* The damage has a known attacker, and the actor may retaliate against it (claim 18)
      but does not retaliate against this hit. It has neither a target nor a destination, and the
      player-scatter setting (Import mapping) is on or the actor has the scatter veterancy ability.
      Its owner and its current behaviour do not matter. An example is a human-controlled actor
      beyond the reach of F7. Retaliating gives the actor a target (claims 10 and 18), so an actor
      that retaliates is never sent the request this way.
    - *Case 2.* The damage has no known attacker or the actor may not retaliate against it, and all
      of these hold:
      - the actor is not an aircraft,
      - its current behaviour has the scatter property,
      - it is not held by a tether link,
      - its locomotor is not moving (S-0208 Terms),
      - it has neither a target nor a destination,
      - its owner is computer-controlled, or the player-scatter setting is on, or the actor has the
        scatter veterancy ability.

    An infantry unit can also be sent request B of S-0208 claim 15, which has the attacker's
    position as threat point and is not forced. Damage sends no other actor a request with a
    threat point. The request of both cases is forced, so the scatter property never makes a
    vehicle refuse it once sent. In case 2 the property is tested before the request is sent, so
    an actor whose behaviour lacks it is not sent the request in that case.

    **In our engine** (non-infantry actors; S-0208 claim 15 for infantry) the last condition of case 2 reads "the player-scatter setting is on, or the
    actor has the scatter veterancy ability" for the actors of every house (Terms, houses in our
    engine). **Original only (note for AI designers):** in the original, the mobile actors of a
    computer-controlled house meet that condition always, so they are sent the request of case 2
    whatever the player-scatter setting says (whether they accept it is decided as for any
    other actor).
    - Backing: `measurable` (scenario B6 for vehicles, case 2 in part b and case 1 in part d, and
      S-0208 scenario W7 for infantry)
    - Confidence: high (the gate on requests that are not forced, for vehicles; the options of the
      request from damage); medium (the conditions of the two cases, for non-infantry actors)

20. **Recruitable.** The recruitable property is imported for every behaviour (Import mapping), and
    in our engine it has no effect: no rule of any house reads it (Terms, houses in our engine).
    **Original only (note for AI designers):** in the original the property affects only the
    computer player's choice of which actors to send to defend its base when the base is attacked,
    and which to put into teams: an actor whose current behaviour lacks the property is never
    picked. Our AI may honour it when it picks units for such tasks.
    - Backing: `measurable` (scenario B20, for the original's base defence)
    - Confidence: medium (base defence); low (teams)

21. **The anti-air rate is used by buildings only.** The anti-air rate replaces the rate in the
    delay of an armed building's *guard* processing, which also serves its *sticky* and *area guard*
    (each reading its own behaviour's anti-air rate), and in one rare case of the *attack*
    processing of a building whose type has the SAM flag (claim 32, out-of-stage rule). No other processing of
    any actor kind uses it. A layer of rules data that contains a behaviour's section but gives no
    anti-air rate, or gives 0, sets the anti-air rate to the rate as it stands after that layer
    (`quirks.anti_air_rate_layer_reset`, Import mapping).
    - Backing: `measurable` (scenarios B8 and B13). The SAM case is `unmeasured`: no situation that
      reaches it is known (claim 32), so it cannot be set up.
    - Confidence: high

### What the general-purpose behaviours do

The claims below describe the processing of each behaviour for mobile actors unless they say
otherwise. "Infantry", "vehicles" and "aircraft" mark rules for one kind only. Rules, rows and
conditions that apply only to computer-controlled owners are marked **original only**: they
describe the original's computer player, as notes for the AI designers, and our engine applies
none of them (Terms, houses in our engine). Rules for human-controlled owners apply in our engine
to the actors of every house.

22. **Move.** If the actor has no destination, is not moving and has nothing queued, it falls back
    to rest and chooses a delay of 1 tick. Otherwise it chooses the standard delay. The movement
    itself is carried out by the locomotion rules on every tick, not by the processing.
    - **Vehicles:** a vehicle in the middle of deploying or undeploying instead queues *guard* and
      chooses 1 tick. Otherwise, a vehicle whose unload or deploy cycle (deploy specs) has not
      finished restarts that cycle at each such processing, and the outcome and delay of that
      processing are those of the general rule: the restart changes neither.
      Consequence: since a vehicle is not ready to change until that cycle has finished (claim
      11), a behaviour queued for a vehicle in *move* waits for the restarted cycle.
    - **Infantry:** a deployed infantry unit (current sequence one of the deploy kinds) of a
      human-controlled owner instead clears its destination and chooses 1 tick; one of a
      computer-controlled owner whose type's undeploy delay is negative starts its *undeploy*
      sequence and chooses that sequence's frame count as the delay. A deployed unit of a
      computer-controlled owner whose undeploy delay is 0 or more follows the general rule. In our
      engine the rule for human-controlled owners applies to the deployed infantry of every house; the
      two rules for computer-controlled owners are the original's only (note for AI designers:
      in *move*, the original's deployed computer infantry packs up by itself if its type has a
      negative undeploy delay, and otherwise follows the general rule above).
    - Backing: `measurable` (scenario B3; scenario B25 for the deployed infantry). The vehicle
      that is deploying or undeploying while *move* is current, and the restart of an unfinished
      unload or deploy cycle, are `unmeasured` here: which vehicles have such a cycle, how long it
      lasts and how it ends belong to the deploy specs, and no input is known that leaves a vehicle
      in *move* while it deploys, so no setup in this spec can time either (Open questions).
    - Confidence: high (the general rule); medium (the vehicle and infantry special cases); low (the
      deploying-vehicle case, the cycle restart and its consequence)

23. **Attack.** The general rules of an *attack* processing are the four below. Every rule whose
    condition holds applies in the same processing:

    | Rule | Applies when | Effect |
    |---|---|---|
    | hover-attack cell | the actor's type is a hover-attack type and its height above the surface is 0, with or without a target | it takes a nearby cell as its destination |
    | charge-target check | the actor is aiming at a charge target | it looks for a real threat within its weapon range and, if it finds one, makes it the target instead |
    | approach | the actor has a target | it approaches and engages the target (the combat and movement specs own how) |
    | fall-back | the actor has no target | it falls back to rest |

    "Has a target" and "has no target" are judged on the target the actor has once the
    charge-target check has taken effect, so a threat found by that check is approached in the same
    processing.

    The delay of these general rules is the standard delay, *D* = *D*<sub>r</sub> + *U*(0, 2),
    with the draw taken whether or not the halving below applies. It is halved, trunc(*D* / 2),
    when the actor has a target, either it is an infantry unit whose type has the close-range flag
    or its primary weapon's range is at most 512 leptons, and its 2-D distance to the target is
    from 282 to 768 leptons (Formulas F3, F4).
    - **Infantry:** the rules of this table take precedence over the general rules above. When a
      row applies (the first that does), it decides the outcome and the delay, and none of the
      general rules happens (no hover-attack cell, no charge-target check, no approach, no
      halving):

      | Situation | Result | Delay | Draws |
      |---|---|---|---|
      | it has explosive charges (by type or veterancy ability), and its target is a building that accepts charges and is not invisible | takes the target as destination and queues *sabotage* | 1 tick | 0 |
      | **original only:** computer-controlled owner, its target is a building, and its type infiltrates, or occupies or assaults buildings and may occupy this one | takes the target as destination and forces *capture* | 1 tick | 0 |
      | human-controlled owner, and it is deployed (current sequence one of the deploy kinds) | runs its deployed-firing rule (infantry deploy specs) | the standard delay, not halved | 1 |

      The first row is not limited to computer-controlled owners. **In our engine** the second row
      never matches, and the third applies to the deployed infantry of every house. **Original
      only (note for AI designers):** an infiltrator, occupier or assaulter of the original's
      computer player that is attacking a building it may take goes straight for it, in *capture*; our
      AI can get the same result with a capture or occupy order (S-0213, S-0214).
    - Backing: `measurable` (scenario B9; cases b, c and d for the infantry table; scenario B24,
      case d for the hover-attack cell and case c for the charge-target check)
    - Confidence: high (the delay rule, and that the 1-tick rows replace the general rules and take
      no draw); medium (the conditions of the infantry rows)

24. **Guard.** The actor stays where it is. The kind-specific rules at the end of this claim
    (deploying infantry, vehicles) take precedence over the general rules below.

    **Enter-nearest instruction (original only).** In our engine no actor holds an enter-nearest
    instruction (Terms), so this rule never applies; it is kept as a note for the AI designers.
    Under the general rules, a processing of an actor that holds an
    enter-nearest instruction (Terms) carries out that instruction and nothing else of the general
    rules, the delay table below included. The actor looks again for the nearest suitable building
    by the 3-D distance of F4: for *enter the nearest absorbing building*, the absorbing buildings
    of its own owner that accept it as a passenger; for *occupy the nearest owned occupiable
    building*, its owner's owned occupiable buildings that it may occupy; for *occupy the nearest
    occupiable building*, every building of any owner that it may occupy. If it finds one, it takes
    that building as its destination and queues *enter* (absorbing building), or queues *capture*
    with an immediate start (the two occupy instructions), unless it is already bound for that
    building (in contact with it for *enter*; for *capture*, with that building as destination and
    *capture* current). If it finds none, the instruction lapses. Either way it chooses
    *D*<sub>r</sub> with no draw, taken from the rate of the behaviour that is current once this
    has taken effect (claim 6): *capture*'s when the immediate start made *capture* current,
    otherwise the rate of the behaviour being processed (*guard* or *sticky*). Our AI can get the
    same result by ordering such a unit to enter or occupy the building it picks (S-0213, S-0214).

    **Other actors.** Under the general rules, an actor that holds no enter-nearest instruction has
    the effect of this table at each processing, chosen by whether it had a target when the
    processing began:

    | Target when the processing began | Effect |
    |---|---|
    | none | if its secondary weapon charges friendly buildings and one of the 8 cells around it holds a building of its own owner that can be charged, it makes that building a charge target and queues *attack*; otherwise it attempts its idle action (S-0107, claim 14 for infantry) |
    | present | a hover-attack type whose height above the surface is 0 takes a nearby cell as its destination; firing at a target in range is done by the firing rules, not by the processing |

    So an actor that makes a charge target in a processing does not take such a cell in that
    processing. **Original only (note for AI designers):** in addition, an infantry unit of a
    computer-controlled owner with explosive charges whose target, as this table leaves it, is a
    building queues *sabotage* (unless that is already current). Our engine applies this to no
    house; our AI can give such a unit a sabotage order instead.

    The delay is that of the first row that matches:

    | Situation | Delay | Draws |
    |---|---|---|
    | the actor's weapon is still rearming | the rearm time remaining | 0 |
    | the type has the distributed-fire flag and the actor's distributed-fire candidates (Terms) are not empty | 0 | 0 |
    | any other case | the standard delay | 1 |

    *Guard* takes no target by itself in its processing. The automatic target search for an actor in
    *guard* runs on its targeting timer (claim 33).
    - **Infantry** of a type that deploys: the deployer rules of claim 31 take precedence. When one
      of them applies, it decides the outcome and the delay, and nothing else of this claim happens.
    - **Vehicles:** the rules of this table, which belong to other topics, take precedence over the
      general rules, and earlier rows over later ones. A row applies when its condition holds and
      no earlier row marked "yes" in the last column applies. When a row marked "yes" applies, its
      result and delay are the whole outcome: no later row and none of the general rules apply.
      Row 3, marked "no", replaces nothing: its effects come in addition to those of whatever
      applies after it (row 4 or the general rules).

      | # | Vehicle and condition | Result | Delay | Excludes later rows and the general rules |
      |---|---|---|---|---|
      | 1 | a slave master for which the economy topic's wake-up condition for its slaves holds | wakes its slaves | the standard delay | yes |
      | 2 | a harvester or weeder, in those cases of the harvesting rules (economy topic) that end the processing | as those rules say (they can queue *harvest*) | 1 tick, no draw | yes |
      | 3 | a harvester or weeder, in the other cases of those rules | as those rules say (they can queue *harvest*) | none of its own | no |
      | 4 | **original only:** a vehicle that deploys into a construction yard and belongs to a computer-controlled owner whose automatic base building is on | queues *unload* (to deploy) | the standard delay, with the rate of the behaviour being processed | yes |

      No row applies to a slave master whose wake-up condition does not hold, unless another row's
      condition holds for it. In our engine row 4 never applies; the original's computer player
      uses it to deploy its base-building vehicles by themselves, which our AI does with a deploy
      order.
    - Backing: `measurable` (scenario B10, scenario B21 for the enter-nearest instructions,
      scenario B24, case a for the distributed-fire row and case b for the charge target; scenario
      B26, case a for the original-only sabotage rule). The vehicle table's rows 1 to 4 are
      `unmeasured`: the conditions of rows 1 to 3 belong to the economy topic's harvesting and
      slave rules, which no spec states yet, so no setup here can tell which rule's condition held,
      and row 4 is the original's only and needs a computer player that deploys its base-building
      vehicles by other means in the same game. The attempt at the idle action is backed by S-0107
      (claim 14).
    - Confidence: high (the delay rule, no target search in the processing); medium (the special
      cases, the enter-nearest search and its delay, and the distributed-fire case, whose list the
      target-acquisition specs own); low (the "already bound" exceptions of the enter-nearest rule)

25. **Sticky.** *Sticky* is processed as *guard* (claim 5), with the properties of *sticky*. It
    differs from *guard* in ways that follow from other claims, whatever the data says:
    - an infantry unit, vehicle or aircraft whose current behaviour is *sticky* is never ready to
      change (claim 11), so queued behaviours, including orders, never start while it lasts and
      only a force or an override replaces it,
    - *sticky* is not among the behaviours that run the per-tick opportunity search (claim 33), and
      the processing of *guard* takes no target by itself (claim 24),
    - a mobile actor of a human-controlled owner in *sticky* does not retaliate unless the
      player-return-fire setting is on (claim 18).

    Observable consequence: a mobile actor of a human-controlled owner in *sticky* never takes a
    target by itself (unless that setting is on); in our engine that holds for the actors of every
    house. In the original, one of a computer-controlled owner takes a target only by retaliating
    (claim 18, original only). Further differences come only from the imported properties of
    *sticky*: if they mark it paralysed, the actor never falls back to rest out of it (claim 17).
    - Backing: `measurable` (scenarios B6 and B17)
    - Confidence: high (readiness); medium (the target consequence)

26. **Area guard.** The actor guards an area around its guard spot. At each processing, the first
    row of this table whose condition holds applies and ends the processing with that row's delay:

    | Row | Condition | Result | Delay |
    |---|---|---|---|
    | 1 | **vehicles:** the vehicle is a slave master and the economy topic's wake-up condition for its slaves holds | wakes its slaves | standard delay |
    | 2 | the actor is in a bunker | queues *guard* with an immediate start | 1 tick |
    | 3 | **original only:** the actor holds an enter-nearest instruction (Terms) | carries it out as in *guard* (claim 24) | *D*<sub>r</sub>, no draw, with the rate of the behaviour then current: *capture*'s when an occupy instruction's immediate start made it current, else *area guard*'s |
    | 4 | **vehicles:** the vehicle is a harvester | queues *harvest* and starts it at once, whether or not it is ready to change | 1 + *U*(1, 10) ticks |
    | 5 | **original only:** an infantry unit of a computer-controlled owner with explosive charges, whose target is a building, while *sabotage* is not current | queues *sabotage* | 1 tick |

    Each outcome of the processing comes with these further effects, and with no others:

    | Outcome | Slave master wakes its slaves anyway | Takes own cell as guard spot | Computer-owner effects |
    |---|---|---|---|
    | a deployer rule of claim 31 applies (Infantry, below) | no | no | no |
    | row 1, 2 or 3 applies | no | no | no |
    | row 4 applies | no | no | yes |
    | row 5 applies | yes | yes | yes |
    | none of rows 1 to 5 applies | yes | yes | yes |

    - *Slave master wakes its slaves anyway:* a slave master wakes its slaves, whatever the wake-up
      condition says.
    - *Takes own cell as guard spot:* an actor that has no guard spot and nothing queued (not
      counting what row 5 queues in the same processing) takes its own cell as its guard spot. So
      an actor in *area guard* lacks a guard spot after such a processing only if a behaviour is
      queued.
    - *Computer-owner effects* (original only: computer-controlled owners): a unit that has no destination
      and whose first queued destination is its guard spot takes the guard spot as its destination
      and removes that entry from its queued destinations (a looping route puts it back at the
      end); no behaviour is queued by this, so it does not count for the guard-spot effect above.
      A guard spot that is a cell holding an allied building is moved to the nearest free cell (not
      for vehicles whose type has the harvester flag; weeders are not excluded).

    **In our engine** rows 3 and 5 never apply and no actor has the computer-owner effects (Terms,
    houses in our engine): the outcomes "row 5 applies" and the column "Computer-owner effects" of
    the table above are the original's only. **Original only (note for AI designers):** the
    original's computer player gets from them a saboteur that goes for a building it has as target,
    a guard spot moved off a cell holding an allied building, and a resting unit that follows its queued
    destinations back to its guard spot; our AI can get the same through sabotage, move and
    area-guard orders.

    When no row ends the processing, an actor with a guard spot is handled by the table below. The
    **leash** holds when the actor is not firing, has no destination, and its 3-D distance to the
    guard spot (with the building reduction of F4 when the guard spot is a building) is greater
    than the leash *L* (Formulas F5). The **search** is the area search around the guard spot of the
    target-acquisition specs, which also restarts the targeting timer (claim 33). It runs only
    when those specs allow the actor to search passively (as for *guard*, claim 33) and its
    targeting timer has run out; the column "may search" below means both.

    | Leash | Target | May search | Result in this processing |
    |---|---|---|---|
    | holds | any | yes | drops any target, takes the guard spot as its destination, and searches |
    | holds | any | no | drops any target, takes the guard spot as its destination, and attempts its idle action |
    | does not hold | none | yes | searches |
    | does not hold | none | no | attempts its idle action |
    | does not hold | present | any | approaches and engages its target |

    A search that finds a target ends the processing with a delay of 1 tick; a search that finds
    nothing is followed by an attempt at the idle action. So an actor that the leash pulls back can
    take a new target around its guard spot in the same processing, and never engages, in that
    processing, the target the leash made it drop.

    An actor without a guard spot does none of this. In a processing that none of rows 1 to 5 ends,
    any actor, with or without a guard spot, that has no target once the leash and the search have
    taken effect and whose secondary weapon charges friendly buildings looks for a charge target as
    in *guard*.

    The delay is *D* = *k* × *D*<sub>r</sub> + *U*(1, 5), where *k* = 2 for aircraft and 1 for
    every other kind; it becomes trunc(*D* / 6) under the same close-range condition as in *attack*
    (Formulas F3).
    - **Infantry** of a type that deploys: the deployer rules of claim 31 take precedence over every
      row. When one of them applies, it decides the outcome and the delay, and nothing else of this
      claim happens.
    - **Computer-controlled owners** (original only) also have the computer-owner effects of the
      table above, in the processings that the table marks, and at no other processing.
    - Backing: `measurable` (scenario B10, with case h for the close-range shortening, case i for
      the aircraft factor and case j for the bunker row; row 3 is the rule of claim 24, scenario
      B21; scenario B24, case b for the charge target; scenario B26, cases b and c for the
      original-only row 5 and the guard spot moved off an allied building). Row 1, the slave
      master's wake-up and the "wakes its slaves anyway" effect are `unmeasured` for the reason
      given in claim 24 (the wake-up condition belongs to the economy topic). The computer-owner
      effect on queued destinations is `unmeasured`: no setup is known that has the guard spot as
      a computer-controlled unit's first queued destination.
    - Confidence: high (the guard spot and delay rules); medium (the leash, the order of leash and
      search, the slave-master rows, the computer-only rules and the harvester rule)

27. **Hunt.** At each processing, unless the actor's type has the stupid-hunt flag, it runs the
    normal search of the target-acquisition specs from its own position. The search restarts the
    targeting timer every time (claim 33) but only looks for a new target when the actor has none,
    so an actor that already has a target keeps it. The outcome then depends on whether the actor
    has a target after the search; an actor whose type has the stupid-hunt flag is treated as
    having none, even if it has one:
    - with a target: an infantry engineer (without explosive charges) or vehicle thief takes the
      target as destination and queues *capture*; an infantry unit with explosive charges whose
      target is a building takes it as destination and queues *sabotage*; in both cases with an
      immediate start if ready, which the processing's own delay then overrides (claim 3c). Any
      other infantry unit clears its destination and approaches the target; other kinds approach the
      target,
    - without a target (or with the stupid-hunt flag): **original only:** in a campaign game, an
      actor of a computer-controlled owner moves towards a cell its house picks for attack, if the house has
      one: it queues *move* there with an immediate start, and a slave master also gives its
      harvesting slaves a guard reset. If the house has no such cell nothing happens. In every other
      case the actor only attempts its idle action. So in skirmish and multiplayer games a hunting
      actor that finds nothing stays put; in our engine that holds in every game, for every house.

    The delay of this general rule is always the standard delay, with the rate of the behaviour
    that is current once the outcome above has taken effect: the rate of *capture*, *sabotage* or
    *move* when the outcome started one (claim 6).
    - **Vehicles** whose type deploys into a construction yard (or any deploying type that has a
      target or a human-controlled owner) instead try to find a clear spot and deploy there, and
      choose the standard delay. In our engine, where every owner counts as human-controlled, this
      covers every vehicle of a deploying type. Deploying makes no other behaviour of the vehicle
      current: while it seeks the spot and turns to deploy, *hunt* stays current, so the standard
      delay always takes the rate of *hunt*; when the deploy succeeds, the vehicle is replaced by a
      building of its deploys-into type (deploy and structures specs) and has no further
      processing. When the building may not be placed at the spot the vehicle has reached, the
      deploy is refused and the vehicle queues *guard* without an immediate start, so *guard*
      becomes current at its next per-tick start at which it is ready (claim 11). **Original only:** in addition, on every tick a
      vehicle whose type deploys into a construction-yard type queues *hunt* (without an immediate start) if its owner is
      computer-controlled, the game is not a campaign, the owner's automatic base building is on,
      the owner has no construction yard, and its current behaviour is neither *hunt* nor *unload*.
      A *hunt* queued this way becomes current through the same tick's per-tick start (claim 11) if
      the vehicle is ready to change.
    - **Infantry** of a computer-controlled owner (original only) have two rules of their own, judged on the target
      the unit has when the processing begins, which take precedence over the general rule above.
      When one applies (the first that does), it decides the outcome, the processing chooses 1 tick
      with no draw, and none of the general rule happens (no search, no targeting-timer restart, no
      idle action):
      - its target is a building and its type infiltrates, or occupies or assaults buildings and may
        occupy this one: it takes the target as destination and forces *capture* (the second row
        of claim 23's infantry table),
      - it is deployed (current sequence one of the deploy kinds) and has no target: it starts its
        *undeploy* sequence.

      The explosive-charges rule of claim 23 (the first row of its infantry table) is not among
      these. In *hunt* an infantry unit with explosive charges takes the general with-target
      outcome above instead, which queues *sabotage* with an immediate start if ready and chooses
      the standard delay.

    **In our engine** the campaign move, the per-tick *hunt* of base-building vehicles and the two
    infantry rules never apply (Terms, houses in our engine). **Original only (note for AI
    designers):** with them the original's computer player sends hunting units towards its attack
    cell in campaign missions, puts a base-building vehicle whose house has lost its last
    construction yard into *hunt* (so that it looks for a place to deploy), sends its building
    takers straight for a building they have as target, and packs up its deployed infantry when
    it hunts without a target. Our AI can get these results through move, deploy, capture and
    undeploy orders.
    - Backing: `measurable` (scenario B11; cases b and c for the infantry rules, case d for the
      vehicle deploy rule, case e for the general with-target outcome). The refused deploy is `unmeasured`: the vehicle first looks for a spot
      where the building fits, and no setup is known in which the spot it reaches then refuses the
      building (Open questions).
    - Confidence: high (the general rule and the skirmish case); medium (the special cases, and
      that deploying makes no other behaviour current); low (the refused deploy)

28. **Capture and sabotage.** Here a *building taker* is an infantry unit with explosive charges, an
    engineer, an occupier, an assaulter or an infiltrator. At each processing, the row is chosen by
    the destination and target the actor has when the processing begins, and only that row applies.
    In the second row, whether the hunt rule also applies depends on the target the actor is left
    with after its fall-back to rest:

    | Destination | Target and actor | Outcome |
    |---|---|---|
    | none | the target is a building and the actor is a building taker | the target becomes its destination |
    | none | any other case | it falls back to rest; if its position is then on a cell that holds a building, it is sent the scatter request below; if it then has no target (the fall-back to rest can clear it, claim 15 rows 4 and 7), the hunt rule applies as well |
    | present | a target | no change |
    | present | no target | the hunt rule applies |

    **Scatter off a building.** The request of the second row has no threat point, is forced and is
    not move-anywhere. The cell tested is the one that holds the actor's position after its
    fall-back to rest. An infantry unit judges the request by S-0208 claim 11 and, when it accepts,
    finds its destination by S-0208 claim 13 (the case without a threat point). For a vehicle the
    scatter property does not decide, since the request is forced (claim 19); its other acceptance conditions and its
    destination belong to the vehicle movement specs, which do not state them yet (Open questions).

    **Hunt rule (original only):** an actor of a computer-controlled owner, other than one of an
    occupier infantry type, has its destination cleared and queues *hunt*. Where it has fallen back
    to rest or scattered in the same processing, this clears any destination that gave it, and
    *hunt* replaces whatever that queued (claim 7). An actor of a human-controlled owner, or of an
    occupier type, is left as the other effects leave it. **In our engine** the hunt rule never
    applies, so every actor is left as the other effects leave it. **Note for AI designers:** the
    original's computer player puts an actor in *capture* or *sabotage* that has lost its building
    into *hunt*; our AI can do so with an order.

    The delay is always the standard delay. The capture, demolition or entry itself happens when the
    actor reaches the building (the special-abilities specs). *Eaten* is processed by the same table,
    except that occupiers, assaulters and infiltrators do not count as building takers and the hunt
    rule never applies. Its second row sends the same scatter request.
    - Backing: `measurable` (scenario B12)
    - Confidence: medium (the table); high (the options of the scatter request)

29. **Enter.** An actor in *enter* keeps asking the object it is meant to enter (a transport, a
    building) for permission; if that object refuses, the actor breaks off its contact and falls
    back to rest. The entry protocol and its timing belong to the transport and structures specs.
    - Backing: `measurable` (scenario B12)
    - Confidence: low (summary only)

30. **Unload (infantry).** For an infantry type that deploys, *unload* switches between deployed and
    undeployed, and in every case forces *guard* and clears the unit's destination:
    - a deployed unit (current sequence one of the deploy kinds) starts its *undeploy* sequence only
      if its type's undeploy delay is negative,
    - an undeployed unit starts its *deploy* sequence. If its deploy weapon (Terms: its secondary
      weapon, or its primary weapon for a type with the spray-attack flag) has the area-fire flag,
      a unit of one type singled out by name (Import mapping; `quirks.named_deploy_area_fire`)
      aims at nothing new, and a unit of any other type takes its own cell as its target.

    The delay is the first that applies in this table:

    | Unit | The type's undeploy delay | Area-fire flag of the deploy weapon | Delay |
    |---|---|---|---|
    | deployed | any | any | 450 ticks |
    | undeployed | 0 or more | any | the undeploy delay, in ticks |
    | undeployed | negative | yes, and the type is the one singled out by name | frame count of its *deploy* sequence + 1 |
    | undeployed | negative | yes, any other type | 450 ticks |
    | undeployed | negative | no (or no deploy weapon) | 450 ticks |

    Because the processing forces *guard* (claim 3c), the first processing of *guard* comes when that
    delay runs out. Any other infantry unit does nothing in *unload* and chooses 450 ticks with no
    draw, as the default processing of claim 4 does.
    What *unload* does for vehicles (transports unloading, deploying vehicles) and buildings belongs
    to the transport, deploy and structures specs (claim 34).
    - Backing: `measurable` (scenario B12; case f for the slot that gives the deploy weapon)
    - Confidence: high (the delay table and the forced *guard*); medium (the area-fire target rule
      and the choice of the deploy weapon)

31. **Deploying infantry in guard and area guard.** For an infantry unit of a type that deploys (the
    deployer flag), these rules take precedence over the rest of its *guard* or *area guard*
    processing. When one applies (the first that does), it decides the outcome and the delay, and
    nothing else of claim 24 or 26 happens; when none applies, the processing is as claim 24 or 26
    says.

    | Unit | Condition | Result | Delay |
    |---|---|---|---|
    | deployed | its type's undeploy delay is 0 or more | starts its *undeploy* sequence | frame count of that sequence |
    | deployed | undeploy delay negative, its type deploys to fire and is not immune to radiation | runs its deployed-firing rule (infantry deploy specs) | standard delay |
    | deployed | undeploy delay negative, deploys to fire, immune to radiation, and its cell has radiation of level at least *R* | nothing | *D*<sub>r</sub> + *U*(10, 20) |
    | deployed | undeploy delay negative, deploys to fire, immune to radiation, its cell has no radiation or a level below *R*, and the firing rules let it fire at its own cell | fires at its own cell, drops the target, starts its *deployed-fire* sequence | frame count of that sequence |
    | deployed | the same, but the firing rules do not let it fire | drops the target | *D*<sub>r</sub> + *U*(10, 20) |
    | undeployed | **original only:** computer-controlled owner, type deploys to fire, not immune to radiation, undeploy delay negative, no destination, no guard spot or standing on the guard spot's cell, and the ticks since the start tick exceed the auto-deploy delay *A* for the owner's difficulty | if it is moving, it is told to stop; otherwise it starts its *deploy* sequence | standard delay if moving; else frame count of the *deploy* sequence |

    Here *R* = trunc(*W* / 3), integer division truncating toward zero, with *W* the radiation level
    field of the unit's secondary weapon (Terms; always the second slot, whatever the spray-attack
    flag says, so for a spray-attack type it is not the deploy weapon of claim 30), and *A* is the entry of the game-wide auto-deploy delay
    list for the owner's difficulty (Terms; Import mapping), in ticks. "The ticks since the start
    tick exceed *A*" means *t* > *t*<sub>s</sub> + *A*, with *t*<sub>s</sub> the start tick and the
    sum *t*<sub>s</sub> + *A* formed in the domain of F0 (a signed 32-bit sum that wraps). A deployed unit whose type
    has a negative undeploy delay and does not deploy to fire, and an undeployed unit of an immune
    type, follow claim 24 or 26 unchanged.

    **In our engine** the last row never applies (Terms, houses in our engine), so this claim never
    deploys an undeployed unit of any house. **Original only (note for AI
    designers):** the original's computer infantry of such a type deploys by itself once it has
    stood on its guard spot, in *guard* or *area guard*, for longer than the auto-deploy delay of its
    house's difficulty since that behaviour started; our AI can get the same with a deploy order.
    - Backing: `measurable` (scenarios B12 and B19; case d of B19 for the radiation-immune unit
      that the firing rules do not let fire)
    - Confidence: high (the first row and the auto-deploy row's delays); medium (the radiation rows
      and the auto-deploy conditions)

32. **Buildings in guard and attack.** A building processes *guard*, *sticky* and *area guard* by the
    same rule (table T1), with the properties of whichever is current.
    - **Armed, in *guard*:** every processing sets the building's ready mark (claim 11). If the
      building has a target, it queues *attack* and starts it at once, whether or not it is ready
      to change (claim 7), and chooses 1 tick; else it chooses the anti-air delay
      *D*<sub>aa</sub> + *U*(0, 2), where *D*<sub>aa</sub> is F1 applied to *a*, the anti-air rate of
      the current behaviour (claim 21). A building whose type is an electromagnetic-pulse cannon,
      some superweapon buildings (structures specs) and a building that can be occupied but has no
      occupants choose the anti-air delay whether or not they have a target, and so do not start
      *attack* by this rule.
    - **Unarmed, in *guard*:** a building whose type has the stupid-guard-mode flag chooses 100 ticks
      and does nothing else. Any other unarmed building runs the factory, repair-bay and reload
      rules of the structures specs (which can queue *repair* and end the processing with 1 tick)
      and chooses the standard delay if its type is a unit repair building, else
      3 × *D*<sub>r</sub> + *U*(0, 2). Neither sets the ready mark.
    - **In *attack*, a building whose type does not have the SAM flag:** without a target it queues
      *guard* and starts it at once, whether or not it is ready to change, and chooses 1 tick. With
      a target, every processing sets the ready mark and asks the firing check (Terms) whether the
      building can fire at the target now. The answer's category decides the outcome; no delay
      takes a rate or a draw:

      | Firing check's answer (Terms) | Result | Delay |
      |---|---|---|
      | *can fire* | fires (combat specs) | 1 tick |
      | *not facing* or *rearming* | turns towards the target | 2 ticks |
      | *illegal*, *impossible*, *out of ammunition* or *out of range* | drops its target, queues *guard* and starts it at once, whether or not it is ready to change | 1 tick |
      | any other answer (*busy*, *cloaked*, *moving*, *must deploy*, *still turning*) | turns towards the target where that applies | 1 tick |

      So an attacking building is processed every tick or every other tick, and one whose target
      leaves its weapon's range returns to *guard* at its next processing.

      **Turret snap** (low confidence). When the answer is *not facing* and the building is a
      turreted building with a voxel turret (Terms), the building checks its aim error *ε*, the
      angle between its turret facing and the direction to the target in facing units, taken the
      shorter way round (0 to 32 768). If its type's turn-rate field is 0, or *ε* is within the
      snap bound *b* of Formulas F8, its turret facing is set exactly onto the direction of the
      target at once (no turning over ticks) and the firing check is asked again in the same
      processing; the row of the table above is then chosen by the second answer, so the building
      can fire in this processing (first row, 1 tick) instead of turning (second row, 2 ticks).
      Otherwise the first answer stands. No other answer, and no building without a voxel turret,
      takes this snap. *Rearming*, which shares the second row, never snaps.
    - **In *attack*, a building whose type has the SAM flag:** this processing never sets the ready
      mark. It keeps one of two stages in the progress count (Terms), *aiming* (0) and *aimed* (1).
      Every behaviour start puts it in *aiming*. A processing in either stage has the result of
      the first matching row for its stage and chooses 1 tick, with no rate and no draw. "A
      suitable target" means a target that is an aircraft whose world height is not 0.

      | Stage | Condition | Result | Stage afterwards |
      |---|---|---|---|
      | *aiming* | the type needs power and its owner is short of power | nothing | *aiming* |
      | *aiming* | no suitable target | drops its target, queues *guard* and starts it at once, whether or not it is ready to change | (*guard* is current) |
      | *aiming* | a suitable target | turns towards it | *aimed* once it has stopped turning and its facing is within 8192 facing units of the direction to the target; else *aiming* |
      | *aimed* | no longer a suitable target | drops the target | *aiming*, where the next processing queues *guard* |
      | *aimed* | the firing check answers *can fire* | fires both its weapons, primary and secondary, in this processing (the combat specs own the shots and their order) | *aiming* |
      | *aimed* | the firing check answers *not facing* | nothing | *aiming* |
      | *aimed* | the firing check answers *illegal*, *impossible* or *out of range* | drops the target | *aiming* |
      | *aimed* | any other answer (*rearming* included) | nothing | *aimed* |

      **Out-of-stage rule.** When, at a processing, the progress count holds neither stage (a
      value other than 0 and 1), the building does nothing in that processing, leaves the count
      as it is, and chooses the anti-air delay *D*<sub>aa</sub> + *U*(0, 2) (claim 21) instead of
      1 tick, which takes one draw. A start resets the count and the rows above set only 0 and 1,
      so this happens only when *attack* became current by an override (which keeps the count,
      claim 10) from a behaviour whose own rule had left another value; since the count stays as
      it is, every later processing of that *attack* does the same until a behaviour start
      resets the count. Whether any stock situation does this is open (Open questions).

      The combat specs own the firing itself.
    - Backing: `measurable` (scenarios B8 and B16, cases b and f for the unarmed delays, case g for
      an armed *guard* building that starts *attack* by its own processing, case e for the
      out-of-range row, case h for the turret snap). Two parts are `unmeasured`: the progress-count rule of SAM *attack* (a
      value other than 0 and 1), because no situation that reaches it is known, and the buildings
      that choose the anti-air delay even with a target (the electromagnetic-pulse cannon, the
      superweapon buildings and the empty occupiable building), because the only observable
      difference from the general rule needs such a building to hold a target at a processing, and
      no setup is known that gives one.
    - Confidence: medium (armed and unarmed *guard*, the ready mark, non-SAM *attack* and its delay
      table, SAM *attack* every tick and its stages); low (the turret snap and its bound F8, the SAM
      aiming tolerance, the power condition and the out-of-stage rule)

33. **Automatic target search under behaviours.** Separately from processing, on every tick an actor
    whose targeting timer has run out and whose current behaviour is *move*, *harvest* or *guard*
    runs the opportunity search of the target-acquisition specs, if those specs allow it to search
    passively (nearly always for *guard*; for *move* and *harvest* only under conditions those specs
    give, such as the opportunity-fire flag). *Sticky* is not in this list, although it is processed
    as *guard*. *Area guard* and *hunt* search from their processing instead
    (claims 26 and 27). Every search, whoever runs it, restarts the targeting timer with
    *N* + *U*(0, 2) ticks, where *N* is the game-wide area-guard targeting delay if the current
    behaviour is *area guard* and the normal targeting delay otherwise (Import mapping). A target
    found by the per-tick search (not by the searches of *area guard* or *hunt*) is an opportunity
    target. On each tick, an actor whose target is an opportunity target drops it if its current
    behaviour is *ambush*, *construction*, *enter*, *harmless*, *missile*, *open*, *repair*,
    *selling*, *sleep*, *stop*, *unload* or *wait* (an actor whose current behaviour is *none*
    keeps it).

    **Order within the actor's update.** For one infantry unit, vehicle or building on one tick,
    the per-tick start (claim 11), the drop rule above, the processing (claim 2) and the per-tick
    search relate as follows; each statement is stated by its observable effect:
    - (a) the drop rule judges the behaviour that is current once the per-tick start has taken
      effect (for a building, its first per-tick start of the tick), and that tick's processing
      never acts on a target the drop rule removed on that tick,
    - (b) whether the per-tick search runs is decided by the current behaviour as that tick's
      processing leaves it, so a processing that starts *guard* (for example a building's *attack*
      whose target left its range, claim 32) can be followed by a search, and a restart of the
      targeting timer, on the same tick,
    - (c) a target that the per-tick search finds on a tick is first acted on by the processing of
      a later tick, never by the processing of the same tick, and the drop rule of the same tick
      does not remove it,
    - (d) every draw that the actor's processing takes on a tick comes before every draw of its
      per-tick search on that tick, including the search's *U*(0, 2) for the targeting timer.
    - Backing: `measurable` (scenario B10 for the timer reset and the behaviour lists, with case e
      for the drop rule and part a; scenario B27, case a for part d, case b for part b and case c
      for part c)
    - Confidence: high (the timer reset, the behaviour lists and parts a to d); medium (the
      passive-search conditions, owned elsewhere)

### Behaviours owned by other topics

34. **Pointers.** The processing of these behaviours is part of other topics and is not specified
    here. Their properties (claims 16 to 21) and the slot rules (claims 7 to 11) still apply.
    - *harvest*: economy, the harvest cycle,
    - *construction*, *selling*, *repair*, *open*: structures (build-up, selling, repair bays and
      service depots, gates),
    - *unload* for vehicles and buildings: transports, vehicle deploying, and structures,
    - *missile*: buildings only (superweapons and structures); infantry, vehicles and aircraft
      process it by the default rule of claim 4 (table T1),
    - *paradrop approach*, *paradrop overfly*, *spy-plane approach*, *spy-plane overfly*: aircraft
      only (aircraft and superweapons); the other kinds process them by the default rule of claim 4,
    - *retreat*, *rescue*, *patrol*: team logic of the original's AI (out of scope) and the
      route orders of the order specs. *Patrol* for vehicles also restarts an unfinished unload or
      deploy cycle at each processing, as in *move* (claim 22): the restart changes neither the
      outcome nor the delay of the processing, and has the same consequence for readiness,
    - all aircraft-specific processing (*attack*, *guard*, *hunt*, *move*, *enter*, *unload*):
      aircraft.
    - Backing: `unmeasured` (pointer only: it states which topic owns each rule, not a behaviour).
      The default processing it names for the other kinds is the rule of claim 4 (table T1). The
      one behaviour it adds, the restart of an unfinished unload or deploy cycle in a vehicle's
      *patrol*, is `unmeasured` for the reason claim 22 gives: the deploy specs own the cycle, so
      no setup in this spec can time it.
    - Confidence: high (that these behaviours have their own processing); low (the *patrol*
      restart, as for *move* in claim 22)

## Formulas

Notation: RZ₅₃(*z*) is *z* rounded toward zero to a double, as in S-0003 (the original's
round-toward-zero mode, whose single switch `quirks.chop_rounding` belongs to S-0205, number
domains, a draft in the same batch). trunc(*z*) is *z* truncated toward zero to an integer. *U*(*a*, *b*) is one
ranged draw (Terms).

**F0. Integer domain of delays.** Every delay value, and every sum, product and quotient formed from
one (*D*<sub>r</sub> + *U*(0, 2), *k* × *D*<sub>r</sub> + *U*(1, 5), 3 × *D*<sub>r</sub> + *U*(0, 2),
*D*<sub>r</sub> + *U*(10, 20), 1 + *U*(1, 10), the halving and the division by 6, the targeting
delay *N* + *U*(0, 2) of F6, and the start-tick test of claim 31), is a signed
32-bit integer. Sums and products wrap modulo 2³² into −2³¹ … 2³¹ − 1 (two's complement). Divisions
truncate toward zero, so trunc(−7 / 2) = −3. A delay that comes out at 1 or less means "next tick"
(claim 2).

**F1. Rate delay.** Let *r* be the current behaviour's rate in game minutes, as parsed by S-0003's
decimal form (claims 6 and 7): a single-precision value widened exactly to double, or, when the
value contains `%`, the double product of claim 7. Then

> *p* = RZ₅₃(*r* × 900)  (900 = ticks per game minute)
>
> *D*<sub>r</sub> = *w*(trunc(*p*)) if |*p*| < 2⁶³;  *D*<sub>r</sub> = 0 if |*p*| ≥ 2⁶³ or *p* is not a number

where *w*(*q*) is the integer congruent to *q* modulo 2³² in −2³¹ … 2³¹ − 1 (the wrap of F0).
The product is one double-precision multiplication, rounded toward zero with
`quirks.chop_rounding` on (the original) and to nearest with it off (S-0205). For a rate without
a percent sign the product is exact, so both settings give the same *p* (S-0205, claim 9); they
differ only for a rate with a percent sign. trunc truncates toward zero. For |*r* × 900| below 2³¹ the wrap does nothing. Example: a rate written as 0.06 is held as the single 0.0599999986588955; times
900 that is 53.99999879…, so *D*<sub>r</sub> = 53. A rate written as 0.05 is held as
0.0500000007450581 and gives 45. The engine default rate is the double nearest to 0.016 (not a
single), which gives 14. The anti-air delay *D*<sub>aa</sub> of claim 32 is this same formula
applied to the anti-air rate.

**F2. Standard delay.** *D* = *D*<sub>r</sub> + *U*(0, 2), in the domain of F0.

**F3. Close-range shortening.** In *attack*: *D* = trunc((*D*<sub>r</sub> + *U*(0, 2)) / 2). In
*area guard*: *D* = trunc((*k* × *D*<sub>r</sub> + *U*(1, 5)) / 6) with *k* = 2 for aircraft, 1
otherwise. Both in the domain of F0. The draw is taken whether or not the condition holds. The
condition is: the actor has a target; it is an infantry unit whose type has the close-range flag,
or its primary weapon's range is at most 512 leptons; and 282 ≤ ρ₂ ≤ 768, with ρ₂ the 2-D distance
of F4 between the actor's and the target's positions. (The original makes this comparison in
double precision, and on the integer ρ₂ it selects exactly this band.) Because ρ₂ comes from the table square
root, a true distance of exactly 282 leptons gives ρ₂ = 281 and is not shortened, while 769 gives
768 and is.

**F4. Distances.** For positions (*x*₁, *y*₁, *z*₁) and (*x*₂, *y*₂, *z*₂) in leptons let

> *n*₂ = (*x*₁ − *x*₂)² + (*y*₁ − *y*₂)²,  *n*₃ = *n*₂ + (*z*₁ − *z*₂)²  (exact integers; the original sums them in double precision, which is exact below 2⁵³)
>
> ρ₂ = trunc(*r*(*n*₂)),  ρ₃ = trunc(*r*(*n*₃))

where *r* is the table square root: *r*(0) = 0, and for *n* ≥ 1, *r*(*n*) is *n* truncated toward
zero to 14 significant binary digits, then its exact square root truncated toward zero to 24
significant binary digits. This is the function that approved S-0105 (F1) uses, and that S-0204
(coordinates, distance and facing, a draft in the same batch) gives in exact integer form (its F1)
and analyses (its claim 5); whether our engine uses it is S-0204's switch
`quirks.table_root_distance` (default original), and this spec adds none. ρ is never more than ⌊√*n*⌋. For *n* below 536 987 929 (distances
below 23 173 leptons) it is never less than ⌊√*n*⌋ − 1; at that *n* it first falls 2 short, and
for larger *n* it can fall further (for example *n* = 40 001², a 2-D distance of 40 001 leptons
along one axis, gives ρ = 39 999). The leash of F5 measures from the actor to
its guard spot with the 3-D form and subtracts the building reduction β = 64 × (*W* + *H*) leptons
when the guard spot is a building with a *W* × *H* foundation, never going below 0. The guard spot's
reference point is that of its object; for a guard spot that is a cell it is the cell's centre at
the cell's height (confidence low, Open questions). The close-range test of F3 and the retaliation
reach of F7 use the plain ρ₂ and ρ₃ on the two positions, without the building reduction.

**F5. Area-guard leash.** Let *g* be the type's guard range in leptons (Import mapping) if it is not
0, else the larger of the ranges of the actor's primary and secondary weapons in leptons. Let
*G* = min(max(2*g*, 0), 4096) leptons. Then

> *L* = trunc(RZ₅₃(*G* × 1.1)) = ⌊11 × *G* / 10⌋ leptons

with 1.1 the double nearest to 1.1. The product is formed in floating point in the original; for
every integer *G* from 0 to 4096 it equals the exact integer form on the right (checked for all
4097 values with exact arithmetic). So *L* is at most 4505 leptons. When the guard spot is a mobile
actor (an actor guarding another unit), *L* is instead the game-wide guard stray distance in
leptons (Import mapping), with no factor and no limit. The actor drops its target and returns when
the distance of F4 (with the building reduction) is greater than *L*.

**F6. Targeting timer.** On every automatic search, the targeting timer is set to
*N* + *U*(0, 2) ticks, where *N* is the area-guard targeting delay (current behaviour *area guard*)
or the normal targeting delay (otherwise), both integers in ticks.

**F7. Retaliation reach for human-controlled owners.** The actor overrides to *attack* against an
attacker it cannot reach if ρ₃(actor, attacker) ≤ (*S* + 0.5) × 256, *S* the type's sight in cells
(an integer), the right side computed in double precision and compared with the integer ρ₃.

**F8. Turret snap bound** (claim 32). All values are integers in facing units (65 536 to a full
turn). Let *v* be the building type's turn-rate field and *w*₈ the integer in −128 … 127 congruent
to *v* modulo 256. Then

> *b* = |256 × *w*₈|  (so 0 ≤ *b* ≤ 32 768)
>
> *ε* = |*e*|, with *e* the integer in −32 768 … 32 767 congruent to *ψ*<sub>t</sub> − *ψ* modulo 65 536

where *ψ* is the turret facing and *ψ*<sub>t</sub> the direction to the target, both in facing
units. The snap happens when *v* = 0 or *ε* ≤ *b*. Exact integer arithmetic; no rounding arises.
For example *v* = 5 gives *b* = 1280, and *v* = 128 gives *w*₈ = −128 and *b* = 32 768, which every
*ε* meets. A field that is a multiple of 256 other than 0 gives *b* = 0, so only an exact aim
snaps.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Default processing delay | 450 | ticks | Claims 4 and 30; no draw |
| Ticks per game minute | 900 | ticks per game minute | F1; glossary *game minute* |
| Rate absent-key default | double nearest 0.016 | game minutes | Every behaviour; gives *D*<sub>r</sub> = 14 |
| Anti-air rate absent-key default | the rate | game minutes | Claim 21 |
| Flag absent-key defaults | no-threat no, paralysed no, recruitable yes, retaliate yes, scatter yes, zombie no | dimensionless (yes/no) | Every behaviour |
| Standard draw range | 0 to 2 | ticks | F2 |
| Area-guard draw range | 1 to 5 | ticks | Claim 26 |
| Area-guard harvester draw range | 1 to 10 | ticks | Claim 26; plus 1 |
| Deploying-infantry draw range | 10 to 20 | ticks | Claim 31; added to *D*<sub>r</sub> |
| Radiation threshold divisor | 3 | dimensionless | Claim 31; *R* = trunc(*W* / 3) |
| Attack close-range divisor | 2 | dimensionless | Claim 23, F3 |
| Area-guard close-range divisor | 6 | dimensionless | Claim 26, F3 |
| Area-guard rate factor *k* | 2 for aircraft, 1 otherwise | dimensionless | Claim 26, F3 |
| Unarmed building rate factor | 3 | dimensionless | Claim 32; 3 × *D*<sub>r</sub> + *U*(0, 2) |
| Close-range band | 282 to 768 | leptons | F3, 2-D distance |
| Close-range weapon threshold | 512 | leptons | F3, primary weapon range at most this |
| Leash cap before the factor | 4096 | leptons | F5, *G* |
| Leash factor | 1.1 (double nearest) | dimensionless | F5; *L* = ⌊11*G*/10⌋, at most 4505 leptons |
| Building reduction per foundation cell of width and height | 64 | leptons | F4, leash only |
| Factory exit cell | (0, +1) from the foundation's top-left cell | cells | Claim 11 |
| Unarmed stupid-guard building delay | 100 | ticks | Claim 32; no draw |
| Non-SAM attacking building delay | 1 or 2 | ticks | Claim 32; 2 when not facing the target or rearming, else 1; no draw |
| SAM attacking building delay | 1 | ticks | Claim 32; no draw. Exception: the out-of-stage rule chooses the anti-air delay *D*<sub>aa</sub> + *U*(0, 2) |
| SAM aiming tolerance | 8192 | facing units | Claim 32 (45 degrees) |
| Turret snap bound step | 256 per unit of *w*₈ | facing units | Claim 32, F8; *w*₈ the integer in −128 … 127 congruent to the turn-rate field modulo 256 |
| Turret snap bound maximum | 32 768 | facing units | F8 (half a turn) |
| Normal targeting delay absent-key default | 27 | ticks | F6, engine default |
| Area-guard targeting delay absent-key default | 36 | ticks | F6, engine default |
| Campaign house skill level: absent key | 0 | house skill level | Claim 15 |
| Campaign house skill level above the maximum | becomes 1 | house skill level | Claim 15 |
| Human-controlled house skill level in skirmish and multiplayer | 0 | house skill level | Claim 15; also every AI house of our engine in a skirmish |
| Charge-target search | 8 | cells | Claims 24, 26: the neighbouring cells |

**T1. Behaviours and which actor kinds have a processing rule.** "—" means the default processing
of claim 4 (nothing, 450 ticks). "own" means a rule of its own, described in the claim named or owned
by the topic named in claim 34. "as X" means the processing of behaviour X, with the
properties (rate, anti-air rate) of the behaviour that is current (claim 5).

| Behaviour | Infantry | Vehicles | Aircraft | Buildings | Claim |
|---|---|---|---|---|---|
| sleep, harmless, ambush, return, stop, wait | — | — | — | — | 4 |
| none (no current behaviour) | — | — | — | — | 4 |
| queued move, attack move | — | — | — | — | 5 |
| attack | own | own | own | own | 23, 32, 34 |
| move | own | own | own | — | 22, 34 |
| guard | own | own | own | own | 24, 31, 32, 34 |
| sticky | as guard | as guard | as guard | as guard | 25, 32 |
| area guard | own | own | own | as guard | 26, 31, 32 |
| hunt | own | own | own | — | 27, 34 |
| capture | own | own | own | — | 28 |
| sabotage | as capture | as capture | as capture | — | 28 |
| eaten | own | own | own | — | 28 |
| enter | own | own | own | — | 29, 34 |
| unload | own | own | own | own | 30, 34 |
| retreat | own | own | own | — | 34 |
| rescue | own | own | own | — | 34 |
| patrol | own | own | own | — | 34 |
| harvest | own | own | — | — | 34 |
| construction, selling, missile, open | — | — | — | own | 34 |
| repair | — | own | — | own | 34 |
| paradrop approach, paradrop overfly, spy-plane approach, spy-plane overfly | — | — | own | — | 34 |

**T2. Delays and draws of the general-purpose processings** (mobile actors), plus the infantry rows
of claims 23 and 27; see the claims for the other kind-specific rules. "Std" is the standard delay *D*<sub>r</sub> + *U*(0, 2). The draw column counts
draws of the processing itself; a target search it runs adds the search's own draws (one *U*(0, 2)
for the targeting timer, claim 33, plus whatever the target-acquisition specs draw).

| Behaviour | Outcome | Delay (ticks) | Draws |
|---|---|---|---|
| none, sleep, harmless, ambush, return, stop, wait, queued move, attack move | always | 450 | 0 |
| move | no destination, not moving, nothing queued | 1 | 0 |
| move | otherwise | Std | 1 |
| attack | general rule | Std, or trunc(Std / 2) at close range | 1 |
| attack (infantry) | explosive-charges or computer capture row of claim 23 | 1 | 0 |
| attack (infantry) | deployed, human-controlled owner | Std | 1 |
| guard, sticky | holding an enter-nearest instruction (original only) | *D*<sub>r</sub> (of *capture* if an occupy instruction started it) | 0 |
| guard, sticky | weapon rearming | rearm time remaining | 0 |
| guard, sticky | distributed-fire type with candidates | 0 | 0 |
| guard, sticky | otherwise | Std | 1 |
| guard, area guard (deploying infantry) | the rows of claim 31 | as claim 31 | 0 or 1 |
| area guard | slave master's wake-up condition holds (vehicles, row 1) | Std | 1 |
| area guard | in a bunker; or sabotage queued (original only) | 1 | 0 |
| area guard | holding an enter-nearest instruction (original only) | *D*<sub>r</sub> (of *capture* if an occupy instruction started it) | 0 |
| area guard | harvester (vehicles) | 1 + *U*(1, 10) | 1 |
| area guard | search found a target | 1 | 0 |
| area guard | otherwise | *k* × *D*<sub>r</sub> + *U*(1, 5), or trunc(that / 6) at close range | 1 |
| hunt | general rule | Std | 1 |
| hunt (infantry, computer-controlled owner; original only) | the capture or undeploy rule of claim 27 (no search) | 1 | 0 |
| capture, sabotage, eaten | always | Std | 1 |
| unload (deploying infantry) | as the table of claim 30 | as claim 30 | 0 |

Where a processing makes another behaviour current before choosing a delay from a rate (the
general *hunt* row, and the enter-nearest rows when an occupy instruction's immediate start makes
*capture* current), the rate is that of the behaviour it made current (claim 6).

Within one processing, the delay's own draw is the last draw the processing takes (claim 6, "the
delay's draw comes last").

## State and ordering

**Slot transitions** (claims 7 to 10). "Q" is the queued, "C" the current and "S" the suspended
behaviour. For each event, the first row that matches applies (as in claim 7).

| Event | Condition | Result |
|---|---|---|
| queue *b* | C is *selling*, or C is *wait* and *b* is *guard* | nothing |
| queue *b* | *b* is none | nothing |
| queue *b* | *b* is C, and Q is none or *b* | nothing |
| queue *b* | any other case (*b* differs from C, or Q holds a behaviour other than *b*) | *b* becomes Q, replacing any queued behaviour |
| start-next | Q ≠ none | the behaviour in Q becomes C and Q becomes none; timer expired now; start tick now; progress count 0 |
| start-next | Q = none | nothing |
| force *b* | C is *wait* and *b* is *guard* | nothing |
| force *b* | otherwise (C *selling* included) | *b* becomes C and Q becomes none; timer expired now; start tick now; progress count 0 |
| override *b* | C is *selling*, or C is *wait* and *b* is *guard* | no slot change; target and destination still put aside and replaced (`quirks.refused_override_retargets`) |
| override *b* | otherwise | S becomes the queued behaviour if Q ≠ none, else the behaviour that was current; *b* becomes C; Q unchanged; target and destination put aside and replaced |
| revert | S ≠ none | the behaviour in S becomes C and S becomes none; target and destination restored |
| revert | S = none | nothing |

The queue rows show the slots only. When the request asks for an immediate start, a start-next
follows as claim 7 says in every queue row except the first, whose request is ignored entirely.

**Typical behaviour transitions** (consequences of the claims, for a mobile actor of a
human-controlled owner):

| Current | Event | Result |
|---|---|---|
| any | move order | *move* queued; current at the next per-tick start when ready (claim 12) |
| *move* | destination reached, stopped, nothing queued | falls back to rest: usually *guard* or *area guard* queued, current on the next tick (claims 14, 15, 22) |
| *move* | stop order | no slot change; destination cleared; falls back to rest at a processing after it has stopped (claims 12, 22) |
| *attack* | target gone | falls back to rest (claim 23) |
| *guard* | attacked, retaliation allowed | override to *attack*; revert to *guard* when the attacker is destroyed or removed (claims 10, 18) |
| any but *wait* | taken over by mind control | *guard* current at once; target, destination and guard spot cleared (claim 9) |
| *wait* | taken over by mind control | *wait* stays current (the forced *guard* is refused); target, destination and guard spot cleared (claim 9; scenario B3, case i) |
| *area guard* | target leads it beyond the leash | target dropped, returns to the guard spot (claim 26) |
| *hunt* (skirmish) | no target found | stays in *hunt*, idles, searches again next processing (claim 27) |
| a behaviour with the no-threat property | damaged by a known attacker | falls back to rest (claim 16) |
| *sticky* | any order | queued but never started while *sticky* lasts (claim 25) |
| a behaviour with the zombie property | nothing | stays until forced or ordered while ready (claim 17) |

**Ordering within a tick, as observed.** This list only gathers, for one infantry unit, vehicle or
building on one tick, the order statements that the claims named in each item make and back; it
adds none of its own:
- a behaviour that the per-tick start makes current (for a building, its first per-tick start of
  the tick) is the one processed on that tick (claim 3a), and the one whose drop list applies on
  that tick (claim 33, part a),
- a processing never acts on an opportunity target that the drop rule removes on the same tick
  (claim 33, part a),
- a target that the per-tick search finds on a tick is first acted on by the processing of a later
  tick, and is not removed by the drop rule of the same tick (claim 33, part c),
- whether the per-tick search runs is decided by the current behaviour as that tick's processing
  leaves it (claim 33, part b),
- every draw that the actor's processing takes on a tick comes before the draws of its per-tick
  search on that tick (claim 33, part d), and within the processing the delay's draw is the last
  (claim 6),
- for an actor that is ready to change, a behaviour order's behaviour becomes current and is
  processed at the actor's first update after the order takes effect (claim 12; in a single-player
  game that is its update on the next tick, S-0203, claim 7),
- for a building, a queued behaviour can also become current after the building's processing and
  per-tick search on the same tick (its second per-tick start, claim 11); it is then first
  processed on the next tick (claim 3b).

Where orders take effect within the tick belongs to S-0203 (tick order) and S-0213 (human
orders).

## Edge cases and quirks

- **Queued move and attack move do nothing** (claim 5). The original processes both as *sleep*. A
  unit put into either does nothing by itself (it does not even search for targets): only orders,
  retaliation (claim 18) and other rules can make it act. This includes a unit that an attack-move
  order without target and destination put into *attack move* (claim 12): it stands still. Proposed
  switch `quirks.queued_move_sleeps` (default on = original). With it off, both are processed
  exactly as *move* (claim 22), and both are added to the behaviours that run the per-tick
  opportunity search of claim 33, under the same passive-search conditions as *move*.
- **Override forgets the current behaviour when one is queued** (claim 10). If a behaviour is queued
  at the moment of an override, that queued behaviour, not the current one, is put aside, and it also
  stays queued; the behaviour that was current is lost. Proposed switch
  `quirks.override_suspends_queued` (default on = original). With it off, the override always
  suspends the current behaviour and leaves the queue as it was.
- **A refused override still changes the target** (claim 10). An override on an actor in *selling*,
  or an override to *guard* on an actor in *wait*, leaves the behaviour slots alone but still puts
  the target and destination aside and replaces them. Proposed switch
  `quirks.refused_override_retargets` (default on = original). With it off, a refused override
  changes nothing.
- **Anti-air rate reset by a layer** (claim 21). A rules layer that restates a behaviour's section
  without an anti-air rate replaces any anti-air rate an earlier layer gave with the rate. Proposed
  switch `quirks.anti_air_rate_layer_reset` (default on = original). With it off, an absent anti-air
  rate keeps the held value (S-0006, claim 3), and only a behaviour whose anti-air rate was never
  given, or is given as 0, uses the rate.
- **Rates and delays that do not fit.** If *r* × 900 is 2³¹ or more (or below −2³¹), the rate delay
  wraps as F1 states; sums and products formed from it wrap as F0 states, and a negative delay means
  processing on every tick (claim 2). Our engine reproduces this. The importer warns for any rate
  whose |*D*<sub>r</sub>| (F1) is above 715 827 875, and for any negative rate. That bound is
  conservative: no delay of this spec can wrap at or below it, and the first that can,
  3 × *D*<sub>r</sub> + 2, wraps only for *D*<sub>r</sub> above 715 827 881. This is a consequence
  of the integer domain of F0, not a quirk of its own, and it has no switch of its own here.
- **Rate precision** (claim 6, F1). The one-tick shortfall of rates written as short decimals
  without a percent sign is a consequence of single-precision parsing (S-0003) alone: the product
  with 900 is exact (S-0205, claim 9), so `quirks.chop_rounding` does not change it. For a rate
  written with a percent sign the product is not exact and `quirks.chop_rounding` decides it (`6%`
  gives 53 ticks on, 54 off; S-0205 effects table). The shortfall has no switch of its own here.
- **Routes are looked up in the local player's list** (claim 13). When an actor that follows a
  route falls back to rest, the original finds the route's next point in the route list of the
  house played on the machine running the game, not in that of the actor's owner, so the result
  depends on which machine computes it and is unspecified on the others. Our engine uses the
  owner's routes on every machine (departure, no switch).
- **Auto-deploy delay list too short** (claim 31). If the game-wide auto-deploy delay list has no
  entry for an owner's difficulty, the original reads a value from outside the list, which is
  unspecified. The auto-deploy row is the original's only, so no rule of our engine reads the
  list and no departure arises. The importer still warns when the list has fewer than 3 entries,
  so that an AI that reproduces the original's auto-deploy (claim 31, note for AI designers) does
  not rely on a short list. Such an AI should then use the last entry, or no auto-deploy if the list
  is empty.
- **A rule tied to one type name** (claim 30). The original picks out the infantry type of claim
  30's area-fire rule by a type name fixed in its program (Import mapping), not by a data field, so
  a mod that renames that type loses the rule and a mod that gives another type that name gains
  it. Proposed switch `quirks.named_deploy_area_fire` (default on = original). With it off, no
  type is singled out by name: a unit of that type is treated like any other type in claim 30, so
  when it deploys with an area-fire deploy weapon it takes its own cell as its target, and its
  delay is the one claim 30's table gives for "any other type" (450 ticks when its undeploy delay
  is negative, else the undeploy delay). The switch changes nothing else.

## Import mapping

**Behaviour property sections.** Each behaviour's properties come from a game-wide section of the
rules data named after the behaviour. Section names are looked up exactly as S-0001 says (with
case), in the spelling of this table:

| Behaviour | Section | Behaviour | Section |
|---|---|---|---|
| ambush | `[Ambush]` | paradrop overfly | `[Paradrop Overfly]` |
| area guard | `[Area Guard]` (with a space) | patrol | `[Patrol]` |
| attack | `[Attack]` | queued move | `[QMove]` |
| attack move | `[Attack Move]` | repair | `[Repair]` |
| capture | `[Capture]` | rescue | `[Rescue]` |
| construction | `[Construction]` | retreat | `[Retreat]` |
| eaten | `[Eaten]` | return | `[Return]` |
| enter | `[Enter]` | sabotage | `[Sabotage]` |
| guard | `[Guard]` | selling | `[Selling]` |
| harmless | `[Harmless]` | sleep | `[Sleep]` |
| harvest | `[Harvest]` | spy-plane approach | `[Spyplane Approach]` |
| hunt | `[Hunt]` | spy-plane overfly | `[Spyplane Overfly]` |
| missile | `[Missile]` | stop | `[Stop]` |
| move | `[Move]` | sticky | `[Sticky]` |
| open | `[Open]` | unload | `[Unload]` |
| paradrop approach | `[Paradrop Approach]` | wait | `[Wait]` |

Keys in each section:

| Key | Property | Form | Absent key |
|---|---|---|---|
| `NoThreat` | no-threat | boolean (S-0003) | held value; engine default no |
| `Paralyzed` | paralysed | boolean | held value; default no |
| `Recruitable` | recruitable | boolean | held value; default yes |
| `Retaliate` | retaliate | boolean | held value; default yes |
| `Scatter` | scatter | boolean | held value; default yes |
| `Zombie` | zombie | boolean | held value; default no |
| `Rate` | rate, game minutes | decimal (S-0003 claims 6 and 7; `%` divides by 100) | held value; default the double nearest 0.016 |
| `AARate` | anti-air rate, game minutes | decimal, as `Rate` | becomes the rate (claim 21); an explicit 0 also becomes the rate |

The sections are read in every rules layer that contains them (S-0006); a layer that does not
contain a section changes none of its values. They are game-wide sections, so their values carry
over between games of a session (S-0006, claim 10). A layer that contains the section resets
`AARate` as claim 21 says.

The same 32 names, matched without regard to case, are how data names a behaviour elsewhere, for
example for objects that a map places. A name that matches none of them means *none*. Which data
fields name behaviours belongs to the specs of those fields.

**Other fields used by this spec** (owned by other specs; listed so the references above resolve).

| Key | Section | Use here | Form and absent key |
|---|---|---|---|
| `NormalTargetingDelay` | `[General]` | *N* of F6 | integer ticks; engine default 27 |
| `GuardAreaTargetingDelay` | `[General]` | *N* of F6 in *area guard* | integer ticks; engine default 36 |
| `GuardModeStray` | `[General]` | leash when guarding a mobile actor (F5) | cells, converted to leptons by × 256 and truncation; absent or −1 keeps the held value; engine default open (Open questions) |
| `AIAutoDeployFrameDelay` | `[General]` | auto-deploy delay list *A* of claim 31 (original only; read by no rule of our engine) | list of integers in ticks, entries for hard, normal and easy in that order; absent keeps the held list; engine default open (Open questions) |
| `PlayerReturnFire` | combat-damage settings section (S-0006) | lifts the behaviour restriction of claim 18 | boolean |
| `PlayerScatter` | combat-damage settings section (S-0006) | player-scatter setting of claim 19 (the scatter from damage; S-0208 claims 11 and 15 for infantry) | boolean |
| `GuardArea` | `[IQ]` | area-guard threshold of claims 14 and 15 | integer |
| `MaxIQLevels` | `[IQ]` | house skill level of computer-controlled houses of the original in skirmish and multiplayer (claim 15); not used for AI houses of our engine | integer |
| `IQ` | a map's house section | house skill level of that house in a campaign mission (claim 15) | integer; absent 0; above the maximum level becomes 1 |
| `BuildConst` | `[AI]` | construction-yard types for the vehicle rules of claims 24 and 27 (the original-only rules, and claim 27's deploy rule) | list of building types |
| `GuardRange` | type section | *g* of F5 | cells, converted to leptons by × 256 and truncation; absent or −1 keeps the held value; engine default 0 |
| `DefaultToGuardArea` | type section | claims 14 and 15 | boolean |
| `StupidHunt` | type section | claim 27 | boolean |
| `HoverAttack` | type section | claims 23 and 24 | boolean |
| `CloseRange` | infantry type section | F3 | boolean |
| `DistributedFire` | type section | claim 24 | boolean |
| `OpportunityFire` | type section | claim 33 (passive search while moving) | boolean |
| `Sight` | type section | F7 | integer cells |
| `C4` | infantry type section | explosive charges (claims 23, 24, 26, 27, 28) | boolean |
| `Engineer`, `Occupier`, `Assaulter`, `Infiltrate`, `VehicleThief` | infantry type section | claims 14, 23, 27, 28 | boolean |
| `Deployer`, `DeployFire`, `UndeployDelay` | type sections | claims 22, 30, 31 | boolean, boolean, integer ticks |
| `Harvester`, `Weeder` | vehicle type section | claims 15, 24, 26 | boolean |
| `DeploysInto` | vehicle type section | claims 15, 24, 27 | building type |
| `CanC4`, `InvisibleInGame` | building type section | claim 23 | boolean |
| `Overpowerable` | building type section | charge targets (claims 24, 26) | boolean |
| `InfantryAbsorb`, `UnitAbsorb` | building type section | absorbing buildings (Terms; claim 24) | boolean |
| `CanBeOccupied`, `TechLevel` | building type section | owned occupiable buildings (Terms; claim 24): the flag set and the tech level 0 or more | boolean; integer, engine default 255 (so a type that gives no tech level counts) |
| `WeaponsFactory`, `Bunker` | building type section | claims 11, 15 | boolean |
| `ElectricAssault` | warhead section | the secondary weapon's warhead that marks a charging weapon (claims 24, 26) | boolean |
| `HasStupidGuardMode`, `UnitRepair`, `SAM`, `CanBeOccupied`, `Powered`, `EMPulseCannon` | building type section | claim 32 | boolean |
| `ImmuneToRadiation` | type section | claim 31 | boolean |
| `RadLevel` | weapon section | *W* of claim 31 (of the secondary weapon) | integer |
| `AreaFire` | weapon section | claim 30 (of the deploy weapon, Terms) | boolean |
| the key spelled "Spray" and "Attack" run together | type section | spray-attack flag: picks the first weapon slot as the deploy weapon (Terms; claim 30) | boolean; absent no |
| `Turret` | building type section (and the building types installed on it as upgrades) | turret flag of a turreted building (Terms; claim 32) | boolean; absent no |
| `TurretAnimIsVoxel` | art, the building type's image section | voxel turret (Terms; claim 32) | boolean; absent no |
| `ROT` | building type section | turn-rate field *v* of F8 (claim 32) | integer; absent 0 |

The infantry type singled out by name in claim 30 is the one whose type name is `DESO`, compared
without regard to case. This name is fixed in the original's program, not taken from data: a mod
that renames that type loses the rule, and a mod that gives another type this name gains it
(`quirks.named_deploy_area_fire`, Edge cases and quirks).

The guard-area veterancy ability and the explosive-charges veterancy ability are entries of the
veterancy ability lists (veterancy specs).

## Measurement scenarios

Common setup: the measurement setup of the glossary, a small flat test map, skirmish, any game
speed (no claim here depends on it), stock data unless a scenario names a test file. The recorder
reads, on every tick, each actor's current, queued and suspended behaviour, the tick on which its
processing timer next runs out, its target and its destination, and, where a scenario says so,
its queued destinations, its house's skill level, its contact, and whether its distributed-fire
candidate list is empty; "processing tick" means a tick on which that
expiry tick moves forward. Scenarios with units of a computer-controlled house test the
original's own rules, including the parts marked as the original's only (notes for the AI
designers); the original's computer houses have no counterpart in our engine. A scenario that has a test map place an object with a
behaviour expects that behaviour to be current once the object is placed; a run in which it is
not is invalid (repeat it), not a falsification. A scenario that looks for draws has the recorder
also read the position of the scenario random stream in its stream sequence (S-0202) at the start
and end of every tick, which gives the number of raw values taken in the tick. A draw *U*(*a*, *b*)
with *a* < *b* takes at least one raw value (S-0202, claim 4), so a processing that takes one
advances the stream further. The scenario compares ticks on which the actor under test processes
with otherwise similar ticks on which it does not, so that raw values taken by other rules
(movement, other actors) cancel out on average; "an extra draw" means that processing ticks take
more raw values on average than the comparison ticks.

- **B1 (claims 1, 2, 4).** Create one infantry unit, one vehicle and one aircraft (a type that can
  stand landed on open ground) of a human-controlled owner, and the same of a
  computer-controlled owner that has no AI activity on the test map. Put each into *sleep*,
  *harmless*, *stop*, *ambush*, *wait* and *return* in turn (through a test map that places
  objects with these behaviours) and record 20 processing intervals each. Expected: every interval
  is exactly 450 ticks; a newly created actor shows *none* in all three slots before anything sets
  them. Falsified by any interval other than 450 while the behaviour stays current, by a
  draw-dependent spread, or by a newly created actor showing a behaviour other than *none* in any
  slot before anything sets them. A run in which another rule (for example an aircraft rule) changes the
  behaviour before 20 intervals have been recorded keeps the intervals recorded until then. Runs:
  one per kind and behaviour.
- **B2 (claim 6, F1, F2).** Named test file: rules that set the rate of *move* to 0.06 for one run
  and 0.05 for another. Order a vehicle back and forth between two cells 40 cells apart and
  record its processing intervals while it moves, counting only intervals between two *move*
  processings of the same move. Expected: intervals of 53, 54 or 55 for 0.06 and of 45, 46 or 47
  for 0.05, each value seen at roughly a third of the time over 300 intervals. Falsified by any interval
  outside the expected set of its rate, in particular any 56 with 0.06, or by no 53 at all in 300
  intervals with 0.06 (which would mean decimal rather than single-precision arithmetic). Runs: as
  many moves as needed for 300 intervals per rate.
- **B3 (claims 3, 7 to 9, 11, 12, 22).** (a) Order a stationary *guard* vehicle to move; expected:
  *move* becomes current at the vehicle's first update after the order takes effect (S-0203,
  claim 7) and is processed on that tick.
  (b) While it moves, order it to attack a distant stationary enemy building out of range; expected:
  *attack* stays queued while the vehicle moves under *move* and becomes current only on a tick on
  which it is not moving. (c) Repeat (b) with an infantry unit; expected: the same. (d) Give a moving
  vehicle a second move order to another cell; expected: no slot changes, the destination changes.
  (e) Let a vehicle arrive; expected: on the first processing after it stops, its resting
  behaviour is queued and becomes current on the next tick. (f) Force: a vehicle of a computer-controlled
  owner on a long move is taken over by a mind-control unit of the human-controlled house. Expected:
  on the tick of the takeover *guard* is current, nothing is queued and target and destination are empty, although the vehicle
  was moving (not ready to change). (g) Stop order: give a moving vehicle in *move* the stop order.
  Expected: no slot changes on that tick, destination empty; at a processing after it has stopped
  moving, its resting behaviour is queued. Repeat with a harvester in *harvest*; expected: *guard*
  current on that tick. (h) Factory exit cell: a test map places an armed vehicle, in contact with
  nothing, on the cell (*x*, *y* + 1) of a vehicle factory whose foundation's top-left cell is
  (*x*, *y*), and a second one on a cell next to the factory; give both an attack order on an enemy
  in range. Expected: the second vehicle's *attack* becomes current on that tick, the first one's
  stays queued while it stands there. (i) Refusals for *wait*: a test map places an armed infantry
  unit of the human-controlled house in *wait*. Give it a guard order; expected: nothing queued,
  *wait* stays current. Then give it a move order; expected: *move* queued and current at the next
  per-tick start at which it is ready. A second unit placed in *wait* by the same map, of a
  computer-controlled house, is taken over by a mind-control unit of the human-controlled house; expected:
  target and destination empty, but *wait* still current (the forced *guard* is refused).
  (j) Refusal for *selling*: an armed defensive building of the human-controlled house is sold; while
  *selling* is current, give it a force-fire order on an enemy in range, if the input allows
  one; expected: nothing queued and *selling* stays current (if the input gives no order to a
  selling building, the case is recorded as not applicable). (k) Timer reset by a start: named
  test file setting the rate of *guard* to 0.2 (*D*<sub>r</sub> = 180). An armed vehicle of the
  human-controlled house rests in *guard* with no enemy within sight (processing intervals 180,
  181 or 182). Ten ticks after one of its *guard* processings, order it to move 10 cells.
  Expected: *move* becomes current at the vehicle's first update after the order takes effect and
  is processed on that tick, about 170 ticks before the *guard* timer would have run out. (l)
  Queue replacement: give a vehicle of the human-controlled house in *move* (moving, so not ready
  to change) an attack order on a distant enemy building out of range, then, while it still
  moves, a move order to another cell. Expected: after the attack order the queued slot shows
  *attack*; after the move order it shows *move* (the current behaviour, queued again in place of
  *attack*); *move* stays current throughout. Falsified by *move* not becoming
  current at the vehicle's first per-tick start after the order takes effect, or not being
  processed on the tick it becomes current (case a), by a queued behaviour
  becoming current while the actor moves (cases b, c), by no resting behaviour queued at the first
  processing after the vehicle stops, or by one that does not become current on the next tick
  (case e), by any slot change in (d) or (g), by *guard*
  not current at once in (f), by the first vehicle of (h) starting *attack* on its cell, by *guard*
  queued or current in (i), by the move order not being queued in (i), by *attack* queued in
  (j), by a first *move* processing in (k) only when the *guard* timer runs out, or in (l) by
  *attack* still queued after the second order, or by any change of the current behaviour. Runs:
  three each.
- **B4 (claims 14, 15).** Named test file: AI settings with a maximum skill level of 5 and an
  area-guard threshold of 0 for run T0, of 1 for run T1 and of 6 (above the maximum) for run T6,
  so that the result does not depend on the stock values. Skirmish, so the human-controlled house
  has skill level 0 and the computer-controlled house has 5 (claim 15). In each run, compare the
  resting behaviours after a completed move, in units not in a team. Human-controlled house:
  - an armed infantry unit: expected *guard* in every run,
  - an armed vehicle: expected *area guard* in T0, *guard* in T1 and T6,
  - the same infantry unit and vehicle with a second test file that sets the default-to-area-guard
    flag on their types: expected *area guard* in every run,
  - a veteran or elite unit whose ability list contains the guard-area ability: expected *area
    guard* in every run,
  - an unarmed vehicle: expected *guard* in every run,
  - placement fall-back: an ore harvester built by a factory that stands on a cell without ore:
    expected *harvest* queued on placement; the same harvester ordered to move to a cell without
    ore and arriving there: expected *guard*.

  Computer-controlled house (the original-only rows, notes for the AI designers): the test map
  places an armed infantry unit, an engineer and an armed vehicle in *move* with no destination,
  so that their first *move* processing falls back to rest (claim 22) without any team or
  trigger: expected *area guard* in T0 and T1, *guard* in T6.
  Falsified by any other resting behaviour. A run in which the recorder shows a skill level other
  than 0 for the human-controlled house or 5 for the computer-controlled house is invalid. Runs:
  one per case and run.
- **B5 (claim 16).** Named test file: rules giving *harmless* the no-threat property and not the
  zombie property. Test map: a unit of one house placed in *harmless* next to an enemy unit in
  *guard*. Expected: the guarding unit never targets it by itself over 2000 ticks. Then damage the
  harmless unit with a third unit; expected: on that tick it falls back to rest, and the guarding
  unit targets it within one targeting delay. Falsified by a target choice before the damage, or no
  fall-back after it. Runs: three. (b) Zombie suppresses the fall-back: the same map with a second
  test file that also gives *harmless* the zombie property. Expected: the damaged unit does not
  fall back to rest, *harmless* stays current, and the guarding unit still never targets it by
  itself over 2000 ticks. Falsified by a fall-back after the damage. Runs: three.
- **B6 (claims 17, 19, 25).** Named test file: rules giving *sticky* the paralysed and the scatter
  properties and turning its retaliate property off, and giving *sleep* the zombie property, the
  scatter property and no retaliate property, and turning the player-scatter setting on (case 2
  of claim 19 then holds for a human-controlled owner, as it does for every house in our engine).
  Test map places an unarmed vehicle of the human-controlled house in *sticky* and one in *sleep*.
  (a) Give each a move order. Expected: the *sticky* vehicle never starts
  *move* (the move stays queued); the *sleep* vehicle, if ready, starts *move*.
  (b) Damage both from out of their range while they stand still with no destination. Expected:
  both move away to scatter (the scatter from damage names no point, so the paralysed refusal of
  claim 17 does not apply), and neither leaves its behaviour by itself through a fall-back to
  rest. Repeat with the scatter property off for both: expected no scatter (case 2 of claim 19
  needs the property).
  (c) The *sticky* vehicle stands on a cell of a bunker of its own house that has room for one
  vehicle and no other unit on its cells; another vehicle of the same house is ordered into the
  bunker. Expected: once the entering vehicle has stopped on the bunker's cell, the *sticky*
  vehicle stays where it is. Control run with a test file that turns the paralysed property of
  *sticky* off: expected it moves away from the bunker.
  Falsified by the sticky vehicle starting *move* in (a), by the *sleep* vehicle not starting *move*
  at a per-tick start at which it is ready, by a fall-back out of either behaviour, by a scatter
  with the property off in (b), by either vehicle not scattering with the property on in (b) in a
  run in which the vehicle movement specs' acceptance conditions (claim 19) held, or in (c) by the
  *sticky* vehicle moving away with the paralysed property on. A (c) control run in which it does
  not move either leaves claim 17's refusal untested. (d) Case 1 of claim 19: named test file
  turning the player-scatter setting on. An armed vehicle of the human-controlled house stands in
  *guard* with no target and no destination. An enemy unit whose weapon outranges the vehicle's
  fires at it from beyond the vehicle's weapon range and beyond (*S* + 0.5) × 256 leptons, *S*
  its sight (F7), so it does not retaliate. Expected: the vehicle moves away to scatter, with no
  override to *attack* (the suspended slot stays empty). Control 1: the player-scatter setting
  off and no scatter veterancy ability: expected no move. Control 2: the attacker within
  (*S* + 0.5) × 256 leptons but beyond the weapon range: expected an override to *attack* and no
  scatter move. Falsified by no move in the main run (in a run in which the vehicle movement
  specs' acceptance conditions of claim 19 held), by a move in control 1, or by a scatter move
  instead of the override in control 2. Runs: three each.
- **B7 (claims 10, 18).** (a) A human-controlled armed vehicle in *move* (long route) is fired at by
  an enemy within its weapon range; expected: no retaliation, *move* continues. (b) The same vehicle
  in *guard*; expected: an override to *attack* (the suspended slot shows *guard*), and a revert to
  *guard* when the attacker dies; on the tick of the override the expiry tick of its processing
  timer does not change (claim 3), so the first *attack* processing comes when the timer of its
  last *guard* processing runs out. (c) With a test file turning player return fire on, repeat (a).
  Expected: retaliation. (d) An attacker with longer range at a distance just above and just below
  (sight + 0.5) × 256 leptons: override only below. (e) Override while a behaviour is queued: named
  test file giving *sticky* the retaliate property and turning player return fire on (claim 18); a
  test map places an armed vehicle of the human-controlled house in *sticky*, a move order queues
  *move* for it (it stays queued, claim 25), and an enemy in range fires at it. Expected: on the tick of the override the suspended slot
  shows *move*, not *sticky* (the queued *move* may then start at the next per-tick start, since
  the vehicle is no longer in *sticky*, claim 11), and after the attacker dies the vehicle is in
  *move* and never returns to *sticky*. With the quirk off, the suspended slot would show *sticky*
  and the revert would bring *sticky* back. (f)
  Refused override: named test file giving *selling* the retaliate property; an armed defensive
  building of the human-controlled house, with no target, is sold while an enemy in range fires
  at it. Expected:
  while *selling* is current, its target becomes the attacker and its slots do not change.
  Falsified by retaliation in (a); in (b), in a run in which the vehicle had no target when it
  was first hit, by no override to *attack* with *guard* in the suspended slot, by no revert to
  *guard* after the attacker dies, or by a change of the expiry tick on the tick of the override; by no retaliation in (c); by a wrong distance boundary in (d), by suspended *sticky* or a
  return to *sticky* in (e), or by an unchanged target or a slot change in (f). (g) Existing target:
    a human-controlled armed vehicle in *guard* has taken an opportunity target A (an enemy in its
    weapon range; the recorder shows the target). A second enemy B, beyond the vehicle's weapon range
    and inside its sight, then fires at it. Expected: no override, the target stays A and the
    suspended slot stays empty. Repeat with a vehicle of the computer-controlled house (the
    original-only part of claim 18): expected an override to *attack* with B as target at once.
    Falsified in the first run by an override to *attack* or by B becoming the target, and in the
    second by no override. Runs: three per case.
- **B8 (claims 21, 32).** Named test file: rules that give *guard* an anti-air rate different from
  its rate (for example rate 0.07, anti-air rate 0.2). Record the processing intervals of an armed
  building without targets. Expected: 180, 181 or 182; an armed vehicle in *guard* at the same time
  shows 63, 64 or 65 (when not rearming; 0.07 is held as 0.0700000002980232, so
  *D*<sub>r</sub> = 63). Falsified by the building using the rate or the vehicle the anti-air rate.
  Runs: 200 intervals.
- **B9 (claim 23, F3, F4).** Named test file: rate of *attack* 0.07 (*D*<sub>r</sub> = 63). An
  infantry unit with a weapon of range 512 leptons or less attacks a stationary target that cannot
  move; record intervals while the 2-D distance is 300 to 700 leptons, and while it is beyond 768
  (approaching). Expected: 31 or 32 at close range (trunc(63/2) = 31, trunc(64/2) = trunc(65/2) =
  32, so 32 about twice as often as 31) and 63, 64 or 65 beyond. Falsified by no halving, or
  halving outside the band. Runs: repeated approaches until 100 intervals per band are recorded. (b) Infantry rules, explosive charges:
  an armed infantry unit of the human-controlled house in *guard*, whose type has the
  explosive-charges flag, stands within its sight but outside its own weapon range of an enemy
  defensive building whose type accepts charges and whose weapon reaches it. When the building
  fires, the unit retaliates (claim 18): *attack* becomes current with the building as target.
  (A player's order cannot set this up: S-0214 turns an attack order by such a unit on such a
  building into *sabotage*. A run in which it does not retaliate is invalid.) Expected: while it walks to the building with *attack* current and
  *sabotage* queued (it is not ready to change while it moves, claim 11), every processing
  interval is exactly 1 tick, and its processing ticks show no more draws than its
  non-processing ticks of the same walk (count draws as in the common setup). Falsified by any
  interval other than 1 tick in that phase, or by one extra draw on its processing ticks. (c) Deployed infantry of the human-controlled house: a deployer type
  that deploys to fire, deployed, ordered to attack an enemy in weapon range, with the rate of
  *attack* set to 0.07 and the close-range flag set on its type by the test file, so that the
  halving of the general rule would apply in the close-range band. Expected: intervals 63, 64 or 65 even when the 2-D
  distance is in the close-range band (no halving). Falsified by intervals of 31 or 32, or by
  intervals of 1 tick. (d) Computer capture row (the original's only): a test-map team whose script
    tells a computer-controlled infantry unit whose type infiltrates to attack an enemy building it
    can reach. Expected: at its first *attack* processing with that building as target, *capture* is
    current on the same tick with the building as destination, and the next processing comes exactly
    1 tick later with no extra draw on the processing tick. Falsified by *capture* only queued, by
    another interval, or by an extra draw. Runs: 100 intervals in (b) and (c); three in (d).
- **B10 (claims 24, 26, 33, F5, F6).** (a) An armed vehicle in *guard*: record targeting-timer
  restarts; expected intervals *N* + 0, 1 or 2 with the normal targeting delay. (b) The same in
  *area guard*; expected the area-guard targeting delay + 0, 1 or 2, and processing intervals
  *D*<sub>r</sub> + 1, … or 5. (c) Let an enemy lure the *area guard* vehicle away: expected the
  vehicle drops its target and turns back on the first processing at which its 3-D distance to the
  guard spot exceeds *L* of F5 (with a test file giving its type a guard range of 4 cells, so
  *G* = 2048 and *L* = 2252 leptons, not 2048; falsified by a turn-back near 2048). In the same
  processing, if its targeting timer has run out, a target search around the guard spot is
  recorded. (d) A vehicle in *guard* that is rearming: processing intervals equal to the remaining
  rearm time. (e) Drop list: a test-map team whose script assigns *sleep* to its member, an armed
  vehicle in *guard* that has just taken an opportunity target by its own per-tick search. Expected:
  on the tick *sleep* becomes current, the target is empty. Control: the same with a script that
  assigns *hunt* (not in the drop list); expected: the target is kept on the tick *hunt* becomes
  current. (f) Area guard, harvester row: an ore harvester of the human-controlled house placed in *area guard*
  by a test map, ready to change. Expected: at its first processing *harvest* becomes current on
  that tick, and the next processing comes 2 to 11 ticks later (1 + *U*(1, 10)), each value seen
  over 100 placements. (g) Area guard, guard-spot rule: an armed vehicle of the human-controlled
  house placed in *area guard* with no guard spot and nothing queued. Expected: after its first
  processing its guard spot is its own cell. Falsified by other intervals, another leash distance,
  no target search around the guard spot in a turn-back processing at which the targeting timer
  had run out (c; the vehicle is armed and in *area guard*, so the target-acquisition specs allow
  it a passive search; a run in which they would not is invalid),
  a kept target in (e), a dropped target in the control of (e), a delay outside 2 to 11 or a
  missing *harvest* in (f), or no guard spot in (g). Runs: 100 intervals per case, 5 lures, three
  runs of case e and of its control, 100 placements in (f), three in (g). (h) Close-range
  shortening of *area guard* (F3): named test file giving *area guard* the rate 0.07
  (*D*<sub>r</sub> = 63) and an armed vehicle type a primary weapon range of 2 cells (512 leptons).
  The vehicle of the human-controlled house stands in *area guard* on its guard spot; a stationary
  enemy building is placed within its weapon range and at a 2-D distance of 300 to 500 leptons.
  Expected: intervals 10 or 11 (trunc(64 / 6) = trunc(65 / 6) = 10, trunc(66 / 6) = trunc(67 / 6) =
  trunc(68 / 6) = 11), both values seen. Control: the building at a 2-D distance above 768 leptons
  and inside the leash: expected 64 to 68 (trunc(*D* / 6) not applied). Falsified by an interval
  outside 10 to 11 in the main run, or by a shortened interval in the control. Runs: 100 intervals
  each. (i) Aircraft factor: the same test file; an armed aircraft of a type that can stand landed,
  of the human-controlled house, placed in *area guard* by a test map with no target. Expected:
  intervals 127 to 131 (2 × 63 + *U*(1, 5)), each value seen; a vehicle in *area guard* at the
  same time shows 64 to 68. Falsified by intervals of 64 to 68 for the aircraft. Runs: 100
  intervals. (j) Bunker row: an armed vehicle of the human-controlled house stands in a bunker of
  its own house (ordered in); give it an area-guard order. Expected: *area guard* becomes current,
  and at its first processing *guard* becomes current on that tick, the next processing coming
  exactly 1 tick later (and the guard spot not taken as its own cell, claim 26). Falsified by
  *area guard* staying current, or by another interval. Runs: three.
- **B11 (claim 27).** Skirmish: a computer-controlled unit placed in *hunt* on an island it cannot
  leave, with no enemy reachable or in range. Expected: it stays in *hunt* without moving; with a
  test file setting the stupid-hunt flag on its type, also no target is taken when an enemy comes
  into range. Falsified by the unit moving towards the enemy base, or by target searches with the
  flag. Runs: three. (b) Infantry capture rule: a computer-controlled infantry unit whose type has
  the infiltrate flag, placed in *hunt* near an enemy building it can reach. Expected: at the first
  *hunt* processing at which its target is that building, *capture* is current on the same tick,
  with the building as destination, and the first *capture* processing comes exactly 1 tick
  later; that *hunt* processing restarts no targeting timer and shows no extra draw (common
  setup). A run in which its target never becomes that building is invalid. Falsified by
  *capture* only queued, by an interval other than 1 tick, or by a targeting-timer restart or an
  extra draw at that processing. (c) Infantry undeploy rule: a
  test-map team whose script assigns *hunt* to a computer-controlled deployer infantry unit that
  is deployed and has no target. Expected: at its first *hunt* processing it starts its *undeploy*
  sequence, and its next processing comes exactly 1 tick later. Falsified by no *undeploy*, or by
  another interval. (d) Vehicle deploy rule: named test file setting the rate of *hunt* to 0.07
  (*D*<sub>r</sub> = 63). A test map places a vehicle of the human-controlled house whose type
  deploys into a building, in *hunt*, on open ground with room for that building. Expected: *hunt*
  stays current, with nothing queued, from its first processing until the vehicle is replaced by
  the building, and every processing interval in that time is 63, 64 or 65. Falsified by another
  behaviour current or queued before the building appears, or by other intervals. Runs: three in
  (b), (c) and (d). (e) General with-target outcome (claim 6, claim 27): named test file giving
  *hunt* the rate 0.07 (*D*<sub>r</sub> = 63), *capture* 0.05 (45) and *sabotage* 0.2 (180). A
  test map places an engineer of the human-controlled house in *hunt* near an enemy building it
  can capture, and, in other runs, an infantry unit with explosive charges near an enemy building
  that accepts them. Expected: at the first *hunt* processing at which the unit's target is the
  building, *capture* (engineer) or *sabotage* (charges) is current on that tick with the building
  as destination, and the next processing comes 45, 46 or 47 ticks later (*capture*) or 180, 181
  or 182 (*sabotage*), not 63 to 65. A run in which the new behaviour is only queued (the unit
  was not ready to change) is set aside; in it the interval is 63 to 65. Falsified by an interval
  of 63 to 65 in a run that is not set aside. Runs: 20 per unit kind, with each of the three
  values seen at least once.
- **B12 (claims 28 to 31; part e for the scatter off a building).** (a) A computer-controlled
  engineer placed in *capture* without a target: expected *hunt* queued at its first processing.
  (b) A deployer infantry type given *unload* while undeployed, with a test file that removes the area-fire flag from its deploy
  weapon, once with its undeploy delay set to 30 and once to −1. Expected: it deploys and *guard*
  becomes current on that tick. In the first case the first processing of *guard* comes 30 ticks
  later and the unit then starts *undeploy* by itself (claim 31, first row). In the second case the
  first processing of *guard* comes 450 ticks later and the unit stays deployed; *unload* again
  then gives *guard*, an *undeploy* sequence and the next processing 450 ticks later. (c) The type
  singled out by name, with its area-fire deploy weapon and an undeploy delay of −1, given *unload*
  while undeployed: expected the first processing of *guard* comes (frame count of its *deploy*
  sequence + 1) ticks later, and its target does not become its own cell. (d) An infantry unit
  ordered to enter a full transport: expected it falls back to rest. (e) An armed infantry unit
  that is not a building taker, of a human-controlled owner (so that the hunt rule does not apply),
  placed in *capture* without a target on the cell of an open gate of its own house, on flat
  ground with no other unit within 2 cells; repeated with the unit on an ordinary cell. Expected,
  on the gate: at its first processing it falls back to rest and on the same tick starts to walk to
  a neighbouring empty cell, the one that S-0208 claim 13 gives for the tick number of that tick.
  Expected, on the ordinary cell: it falls back to rest and stays on its cell. Falsified, case by case: (a)
  by anything other than *hunt* queued at the first processing; (b) by *guard* not current on the
  tick of the deploy, by a first *guard* processing other than 30 ticks later in the first run or 450 ticks later in the second, by no
  self-undeploy in the first run, or by an undeploy without a new *unload* in the second; (c) by a
  first *guard* processing at any other interval, or by the target becoming the unit's own cell; (d)
  by the unit staying in *enter*, or entering; (e) by no move from the gate cell, by a destination
  other than the one S-0208 claim 13 gives, or by a move from the ordinary cell. Runs: three each.
  (f) Deploy weapon slot (claim 30, Terms): named test file giving a deployer infantry type that is
  not the one singled out by name an undeploy delay of −1, an area-fire weapon in its first slot
  only (its second slot holding a weapon without the area-fire flag) and the spray-attack flag. The
  unit, undeployed, is given *unload*. Expected: on the tick of the deploy its target becomes
  its own cell. Control: the same file without the spray-attack flag: expected its target does not
  become its own cell. Second control: the area-fire weapon in the second slot only, without the
  spray-attack flag: expected its own cell as target. Falsified by the own-cell target in the first
  control, or by its absence in the main run or the second control. Runs: three each.
- **B13 (claim 21, Import mapping).** A test map whose override section gives the *guard*
  behaviour's section a rate of 0.2 and nothing else. Expected: an armed building in *guard* shows intervals 180, 181 or 182 (the anti-air rate
  followed the new rate). With the override also giving an anti-air rate of 0.07, expected
  63, 64 or 65. Falsified by the building keeping the base rules' anti-air rate in the first case,
  or by an interval other than 63, 64 or 65 in the second.
  Runs: 100 intervals each.
- **B14 (claims 5, 12).** (a) A test map places a vehicle with the behaviour name for *queued move*,
  and another with the name for *attack move*. Expected for both: that behaviour current,
  processing every 450 ticks, no movement and no automatic target search over 3000 ticks with an
  enemy in range. (b) With the attack-move gesture (S-0214), give an armed vehicle of a type
  eligible for attack-move an attack-move order on an empty cell 10 cells away; then give it one on
  an enemy unit out of its weapon range. Expected: after the first order *move* becomes current,
  with the cell as destination; after the second, *attack* becomes current with the enemy as
  target; in neither case does *attack move* become queued or current. Falsified by any movement,
  target search or other interval in (a), or in (b) by *attack move* in a slot or by the other
  behaviour becoming current. Runs: one each in (a), three in (b).
- **B15 (claims 13, 14 row 4 and 15 row 3).** (a) Guard-spot row: a human-controlled infantry unit given an area-guard
  order on a cell 5 cells away arrives there. Named test file: player scatter on. It is then
  damaged by an attacker it cannot fire back at, so that it scatters off the guard spot's cell (an
  order would clear the guard spot, claim 12). Expected, if the scatter ends with a fall-back to
  rest: the unit takes the guard spot as its destination and walks back, keeping the guard spot.
  (b) Route row and claim 15, row 3: with the route-mode input of S-0213 (claim 16), give an
  unarmed vehicle of the human-controlled house, in *move*, a planned route of three move points,
  each 6 cells from the last, and commit it. Expected: when it arrives at the first point and
  falls back to rest, the next point becomes its destination with no new order, while the
  recorder shows its queued destinations empty; *move* stays current and nothing is queued (row 3
  queues *move*, which is already current, so by claim 7 no slot changes); and it heads there.
  If row 3 did not apply, row 7 would clear its destination and queue *guard*, which the recorder
  would show. (c) Claim 14, row 4: the same as (b) with an infantry unit of the human-controlled
  house, of a type that does not default to area guard and without the guard-area veterancy
  ability. Expected: as in (b); if row 4 did not apply, row 12 would queue *guard*. A run in which
  the recorder shows a current behaviour other than *move* when the unit reaches the first point
  is invalid. Falsified in (a) by a unit that falls back to rest away from the guard spot and
  stays there; in (b) and (c) by an actor that stops at the first point, by *guard* (or *area
  guard*) queued at that fall-back, by an empty destination after it, or by one that heads on
  only through a non-empty list of queued destinations (that would be the queued-destinations
  row of claims 14 and 15, not the route row). Runs: three each.
- **B16 (claims 11, 32, buildings).** (a) Record the processing intervals of an armed defensive
  building without the SAM flag while it attacks a stationary target in range, with a weapon whose
  rate-of-fire field (in a named test file) is 20 ticks or more: expected an interval of 1 tick
  after each processing at which it fires and 2 ticks after processings while its weapon rearms,
  never a rate delay. (b) An unarmed building whose type has the stupid-guard-mode flag: expected exactly
  100 ticks every time. (c) A SAM building attacking an aircraft in flight: expected every interval
  1 tick until the target is gone, then *guard* current at most 2 ticks after the target's loss
  (at the building's next processing if it was in *aiming*, one processing later if it was in
  *aimed*, claim 32). (d) An armed defensive building
  without the SAM flag that has been in *guard* for at least one full processing interval, with no
  target, is given a force-fire order on an enemy in range. Expected: *attack* becomes current at
  the building's first per-tick start after the order takes effect (its update on the next tick,
  S-0203 claim 7) and is processed on that tick; the time from the order to *attack* is at most 1
  tick, whatever is left of the anti-air delay. (e) Out of range: the building of case a attacks an
  enemy vehicle, which an order then drives out of the building's weapon range. Expected: at the
  building's first processing after the vehicle leaves range, the target is empty and *guard* is
  current, so at most 2 ticks after the vehicle leaves range. Falsified by *attack*
  becoming current only at the building's next scheduled *guard* processing (a wait that grows
  with the remaining anti-air delay), by an interval other than 1 or 2 ticks in case a, by
  1-tick intervals throughout a rearm period or a 2-tick interval right after a shot in case a,
  by other intervals in case b, by a target kept out of range in case e, or in case c by an
  interval other than 1 tick or by *guard* becoming current more than 2 ticks after the target's
  loss. Runs: 100 intervals
  each in cases a to c, five orders in case d, given at different points of the anti-air delay,
  three runs in case e. (f) Unarmed delays: named test file giving *guard* the rate 0.07
  (*D*<sub>r</sub> = 63). An unarmed building of the human-controlled house that is neither a
  factory nor a repair bay nor of the stupid-guard type (for example a power building), and a
  unit repair building with no vehicle on it. Expected: intervals 189 to 191 (3 × 63 + *U*(0, 2)) for
  the first and 63 to 65 for the second, each value seen. Falsified by 63 to 65 for the first or
  189 to 191 for the second. Runs: 100 intervals each. (g) Armed *guard* building: an armed
  defensive building without the SAM flag in *guard* with no target; an enemy unit walks into its
  range and the building's per-tick search takes it as target. Expected: at its next *guard*
  processing *attack* is current on that tick, and the next processing comes 1 tick later (not an
  anti-air delay). Falsified by *guard* staying current with an interval of the anti-air delay.
  Runs: five. (h) Turret snap (claim 32, F8): named test files give an armed defensive building
  type without the SAM flag the turret flag, a turn-rate field of 128 (so *w*₈ = −128 and the
  bound covers every aim error) and a weapon whose rate-of-fire field is 20 ticks or more, and
  mark its turret art as a voxel model, supplying a voxel turret model for it in the test files.
  A run in which the turret image fails to load is invalid. A stationary enemy vehicle is placed in range in the
  direction opposite to the turret's facing, and the building is given a force-fire order on it.
  Expected: at its first *attack* processing it fires (the shot is recorded on that tick) and the
  next processing comes 1 tick later, with no 2-tick interval before the first shot. Control: the
  same files without the voxel art mark: expected at least one 2-tick interval (turning) before
  the first shot. Falsified by a 2-tick interval before the first shot in the main run, or by
  none in the control. Runs: five each.
- **B17 (claim 25).** A human-controlled armed vehicle placed in *sticky* (test map), player
  return fire off, with an enemy unit driven into its weapon range and firing at it: expected no
  target taken over 2000 ticks. The same with a computer-controlled vehicle (the original-only
  part of claim 25, a note for the AI designers): expected a target only through retaliation (the
  suspended slot shows *sticky*). Falsified by a target taken otherwise.
  Runs: three.
- **B18 (claim 2).** Named test file: rates of *move*, *guard* and *area guard* 0.001 (held as a
  single just above 0.001, so *D*<sub>r</sub> = 0; an actor in *move* or *guard* then has a
  processing tick at least every 2 ticks, and one in *area guard* at least every 5).
  (a) Chrono move: an armed vehicle of the human-controlled house in *guard* is moved by the
  chronosphere; repeat with an armed vehicle that is not a harvester, whose type the test file
  gives a teleporting locomotor, ordered to move 10 cells. Expected: from the start of the jump
  until its position changes to the new place (the leaving stage), the expiry tick of its
  processing timer does not move forward and target and destination are empty; during the
  phase-in that follows, processing ticks come at least every 5 ticks again, whichever of *move*,
  *guard* or *area guard* is current. (b) Erasing: the same vehicle without a chrono move is locked onto by an erasing
  weapon; expected no processing tick from the lock-on until it is removed (or let go), and target
  and destination empty on those ticks. (c) A test map with a tunnel: an infantry unit and a
  vehicle ordered through it show no processing tick while inside. Falsified by a processing tick
  during the leaving stage, while being erased or inside the tunnel, or by a gap of more than 5
  ticks between processing ticks during a phase-in. Runs: three each.
- **B19 (claim 31).** (a) A computer-controlled deployer infantry type that deploys to fire, not
  immune to radiation, with a negative undeploy delay, placed in *guard* on its guard spot without
  a destination, at each difficulty: expected it starts its *deploy* sequence at the first
  processing after (start tick + the list entry for that difficulty), and that processing's delay
  equals the *deploy* sequence's frame count. (b) The same type deployed: expected intervals
  *D*<sub>r</sub> + 0, 1 or 2. (c) A type that deploys to fire and is immune to radiation (named test
  file giving its secondary weapon a radiation level *W*), deployed on a clean cell: expected it
  fires at its own cell and the interval equals its *deployed-fire* sequence's frame count; deployed
  on a cell with radiation of level at least trunc(*W* / 3): expected intervals *D*<sub>r</sub> +
  10 to 20, each value seen. (d) The firing rules refuse: the unit of (c) deployed on a clean
  cell, with a named test file giving its weapon a rate of fire longer than its *deployed-fire*
  sequence, so that its weapon is still rearming at the next processing. Expected: at that
  processing it drops its target and the interval is *D*<sub>r</sub> + 10 to 20, each value seen.
  Falsified by a deploy before the list delay, other intervals, or a
  draw outside 10 to 20. Runs: three per difficulty in case a, 100 intervals each in cases b to d.
- **B20 (claim 20).** Skirmish: a computer-controlled house with armed units in *guard* near its
  base and not in a team. An enemy attacks a base building out of those units' own range. With
  stock rules, record which units the computer sends to defend; then with a named test file that
  turns the recruitable property of *guard* off, repeat. Expected: with the property off, no unit
  in *guard* is sent (units can still act on their own targets). Falsified by a *guard* unit sent
  to defend with the property off. Runs: five each.
- **B21 (claims 6 and 24, enter-nearest instructions).** Named test file: rate of *guard* 0.07
  (*D*<sub>r</sub> = 63) and rate of *capture* 0.05 (*D*<sub>r</sub> = 45).
  (a) Test map: a computer-controlled house with one infantry unit of an occupier type (so the
  hunt rule of claim 28 does not apply to it), in a team whose script tells its members to
  garrison the nearest building, and two empty neutral buildings that the unit may occupy, A
  nearer and B farther. While the unit walks to A, a map trigger destroys A (or a unit of another
  house occupies it first). Expected: the unit falls back to rest, rests in *guard* as a team
  member (claim 14, row 12) and, at a later *guard* processing with no new action of its team script, takes
  B as its destination with *capture* current on that tick; the first *capture* processing comes
  exactly 45 ticks after that *guard* processing (*capture*'s *D*<sub>r</sub>, no spread). A run in
  which, on the tick of that *guard* processing, *capture* is queued but not current (the unit was
  not ready to change, for example because it was firing or in a sequence that is not
  interruptible, claim 11) is set aside; at least three of the five runs must not be set aside.
  (b) The same, but the trigger destroys both A and B while the unit walks to A. Expected: the
  unit stays in *guard*; the *guard* processing at which it finds no building is followed exactly
  63 ticks later by the next, and from then on it shows the general *guard* intervals 63, 64 or
  65, each value seen.
  (c) Absorbing building, if the team scripts offer an instruction to enter the nearest absorbing
  building (otherwise the case is recorded as not applicable): the same house owns two absorbing
  buildings that accept the unit, A nearer and B farther, and a team script gives the unit that
  instruction. While it walks to A, a map trigger destroys A. Expected: at the *guard* processing
  that finds B, B becomes the destination and *enter* is queued while *guard* stays current on
  that tick; *enter* becomes current at a later per-tick start (the next tick if the unit is ready).
  Falsified in (a) by the unit not heading for B, or by an interval of 63, or any interval other
  than 45, in a run that is not set aside; in (b) by an interval other than 63 after the
  processing that finds nothing, or by intervals of exactly 63 throughout; in (c) by *enter*
  current on the tick of the *guard* processing that finds B. Runs: five each.
- **B22 (claims 5 and 6, shared processings and their rates).** (a) Named test file: rate of *guard* 0.2
  (*D*<sub>r</sub> = 180) and rate of *sticky* 0.07
  (*D*<sub>r</sub> = 63). A test map places an armed vehicle of a human-controlled owner in
  *sticky*, with no enemy within sight. Expected: its processing intervals are 63, 64 or 65, each
  value seen, and never 180, 181 or 182. (b) Named test file: rate of *capture* 0.2 and rate of *sabotage* 0.07. An infantry
  unit of the human-controlled house whose type has the explosive-charges flag is given the
  sabotage order on an unarmed enemy building 15 cells away whose type accepts charges (S-0214
  turns it into *sabotage* with the building as destination and no target). Expected: while it
  walks there, *sabotage* is current and its processing intervals are 63, 64 or 65, each value
  seen. Falsified in either case by intervals of 180, 181 or 182 (the rate of the behaviour whose
  processing is shared), by 450 (no processing rule), or by any other interval while the
  behaviour stays current. Runs: 100 intervals per case; in case b, as many walks as
  needed, each with a fresh unit and building placed by the test map.

- **B23 (claim 11, vehicle readiness).** (a) Queued *enter*: a vehicle of the human-controlled house
  in *move* on a long route, damaged, is ordered to enter a unit repair building. Expected: *enter*
  becomes current at the vehicle's first update after the order takes effect, while the vehicle
  is still moving (the exception for a queued *enter*). Control: an attack order on a distant enemy
  building given in the same state stays queued while the vehicle moves (B3, case b). (b) Contact
  with a vehicle-factory building: a vehicle that a factory has just produced, given an attack
  order on an enemy in range while the recorder shows it in contact with the factory. Expected: the
  order stays queued until the contact ends, and then *attack* becomes current at the vehicle's
  next per-tick start. A second new vehicle given a move order in contact becomes *move* at its
  first update after the order takes effect. Falsified by *attack* current while the contact
  lasts, by *enter* staying queued while the vehicle moves in (a), or by *move* staying queued
  in (b). (c) Letting passengers out: a transport vehicle of the human-controlled house that
  carries five infantry units stands still and is ordered to unload them. While the recorder
  shows *unload* current and at least one passenger still inside or still leaving, give it a move
  order to a cell 10 cells away. Expected: *unload* stays current as long as a passenger is still
  leaving, with *move* queued (or replaced in the queue by the *guard* that the end of the
  unloading queues, transport specs); *move* or *guard* becomes current only after the last
  passenger has left. Falsified by *move* current while a passenger is still inside or leaving. A
  run in which the order system cancels the unloading at once is not applicable. Runs: three each.
- **B24 (claims 23, 24 and 26: distributed fire, charge targets, hover-attack cell).** (a)
  Distributed-fire row: a named test file sets the distributed-fire flag on an armed vehicle
  type and the rate of *guard* to 0.07 (*D*<sub>r</sub> = 63). The vehicle of the human-controlled
  house stands in *guard* with three enemy units in its weapon range. Expected: at a processing
  at which the recorder shows the candidate list non-empty, the next processing comes 1 tick later
  and the processing tick takes no more raw values than the comparison ticks (no draw for the
  delay); at a processing at which the list is empty the interval is 63 to 65. Falsified by an
  interval of 63 to 65 after a non-empty reading, or by an extra draw. A run in which the list is
  never non-empty is invalid. Runs: 100 processings. (b) Charge target: an infantry type whose
  secondary weapon charges friendly buildings stands in *guard*, with no target, next to a friendly
  building that can be charged on one of the 8 cells around it; repeat with the unit in *area
  guard*. Expected: at its first processing the building is its target and *attack* is queued
  (current at its next per-tick start). Control: the building two cells away: expected no target
  and no *attack* queued. Falsified by no charge target next to the unit, or by one two cells
  away. Runs: three each. (c) Charge-target check: the unit of (b) with *attack* current and the
  building as its charge target; an enemy unit then enters its weapon range. Expected: at its next
  *attack* processing the enemy is its target, and the unit approaches or engages it in that same
  processing. Control: an enemy beyond the weapon range and within sight: expected the building
  stays the target. Falsified by the building staying the target with an enemy in weapon range.
  Runs: three. (d) Hover-attack cell: a named test file sets the hover-attack flag on an armed
  ground vehicle type (its height above the surface is 0). The vehicle of the human-controlled
  house is ordered to attack a stationary enemy building in its range, and, in another run, stands
  in *guard* while an enemy building is in range (target present). Expected: at the processing, the
  recorder shows a destination on a cell other than the vehicle's own cell; the cell itself belongs
  to the movement specs. Falsified by an empty destination at 20 processings in a row. Runs: three.
- **B25 (claim 22, deployed infantry).** (a) Human-controlled owner: a deployer infantry type that
  deploys to fire, deployed, is given a move order. Expected: *move* becomes current, at its first
  processing the destination is cleared and the unit stays deployed, and the next processing comes
  1 tick later. A run in which the order system undeploys the unit first is not applicable.
  Falsified by a kept destination, or by an interval of the standard delay. Runs: three. (b)
  Computer-controlled owner (the original's only): a test-map team whose script moves a deployed
  computer-controlled deployer infantry unit, once with a negative undeploy delay and once with an
  undeploy delay of 0 or more (named test file). Expected: with the negative delay the unit starts
  its *undeploy* sequence and the next processing comes after exactly the sequence's frame count.
  With 0 or more the general rule applies (interval 1 tick if it has no destination, else the
  standard delay). Falsified by other intervals. Runs: three each.
- **B26 (claims 24 and 26, the original's only: computer-controlled owner).** (a) *Guard*
  sabotage: a computer-controlled infantry unit with explosive charges stands in *guard* and
  takes an enemy building in its range as target. Expected: at its next *guard* processing
  *sabotage* is queued (current at its next per-tick start), unless it is already current. (b)
  *Area guard* row 5: the same unit placed in *area guard* with a building as target. Expected: at
  its processing *sabotage* is queued and the interval is exactly 1 tick; with a human-controlled
  unit of the same type, nothing is queued. (c) Guard spot on an allied building: a
  computer-controlled armed unit placed in *area guard* with its guard spot on a cell that holds an
  allied building. Expected: after its first processing the guard spot is the nearest free cell
  (for the unit of a human-controlled owner it stays on the building's cell). Falsified by no
  *sabotage* queued in (a) and (b), by an interval other than 1 tick in (b), or by a guard spot
  that does not move in (c). Runs: three each.
- **B27 (claims 6 and 33: order of draws and of processing and search within a tick).** Setup as
  the common setup, but the recorder reads the full state of the scenario random stream at the
  start of every tick (the random-state recorder of the measurement setup), not only its
  position. From that state, S-0202's generator and its reduction to a range predict, for each
  tick, the values that the tick's ranged draws would receive in either of the two orders under
  test. A tick counts as valid when the stream advances by exactly the raw values that the draws
  named in the case take under at least one of the two orders; any other tick is set aside, since
  another rule drew on it. A valid tick whose advance fits only one order counts as evidence for
  that order. Each case uses only valid ticks on which the two orders give different readings.
  - (a) Processing before search (claim 33, part d): named test file giving *guard* an anti-air
    rate of 0.2 (*D*<sub>aa</sub> = 180). An armed defensive building of the human-controlled
    house stands alone in *guard*, with no object of another house within its reach, so that its
    per-tick search finds no candidate and takes only the timer's *U*(0, 2). On a tick on which
    both its processing and its per-tick search run (its processing timer and its targeting timer
    run out on the same tick), with predicted values *u*₁ then *u*₂: expected processing delay
    180 + *u*₁ and targeting-timer restart *N* + *u*₂ (*N* the normal targeting delay). Falsified
    by 180 + *u*₂ and *N* + *u*₁ on valid ticks. Runs: until 30 valid ticks.
  - (b) The search follows the processing's result (claim 33, part b): the building of B16, case
    e, in *attack* for longer than the normal targeting delay + 2 ticks (so its targeting timer has
    run out: *attack* runs no per-tick search), and its target driven out of range. Expected: on
    the tick of the processing that drops the target and starts *guard*, the recorder shows a
    restart of the targeting timer on that same tick. Falsified by the first restart only on a
    later tick. Runs: five.
  - (c) A target found by the search waits for a later processing (claim 33, part c): named test
    file giving *guard* an anti-air rate of 0.001 (*D*<sub>aa</sub> = 0, so the building processes
    every 1 or 2 ticks and most acquisitions fall on a processing tick). An armed defensive
    building without the SAM flag in *guard*, with no target; enemy units walk into its range one
    at a time. Over the runs, take the ticks on which the recorder first shows a target
    found by the per-tick search and the building also processes on that tick. Expected: that
    processing chooses the anti-air delay (it does not start *attack*), and *attack* becomes
    current at a later processing. Falsified by *attack* current on the tick on which the target
    first appears. Runs: until five such ticks.
  - (d) The delay's draw after the search's (claim 6): named test file giving *area guard* the
    rate 0.07 (*D*<sub>r</sub> = 63). An armed vehicle of the human-controlled house stands in
    *area guard* on its guard spot, with no object of another house within reach, so that the
    search of its processing finds no candidate and its idle action takes no draw. On a
    processing at which its targeting timer has run out, the predictor gives the readings of both
    orders: (*t*, *u*) when the timer's *U*(0, 2) is drawn first and the delay's *U*(1, 5) second,
    and (*t*′, *u*′) for the other order. Expected: targeting-timer restart *N*<sub>a</sub> + *t*
    (*N*<sub>a</sub> the area-guard targeting delay) and processing delay 63 + *u*. Only ticks on
    which the two orders give different readings count. Falsified by *N*<sub>a</sub> + *t*′ and
    63 + *u*′ on valid ticks. Runs: until 30 valid processings.

## Open questions

- Whether any stock map, campaign mission or team script puts an actor into *queued move* or
  *attack move*. The program itself never makes *queued move* current, and makes *attack move*
  current only through the attack-move orders that S-0213 turns into *attack move* (claim 12),
  which player input does not create as far as studied.
- Whether an attack-move order can reach a building in the original. Buildings are never eligible
  for attack-move (S-0214), so input does not create one; if one arrived, the building would queue
  *attack move* (claim 12).
- Stale suspended behaviour: start-next and force leave the suspended slot alone (claim 10), so a
  revert after a later order (for example when an opportunity target dies during a move ordered
  after a retaliation) might bring back the behaviour that was current before the retaliation. The
  full conditions of the revert on target loss are not established well enough to state this as a
  claim.
- Whether any situation reaches the out-of-stage rule of a SAM building's *attack* (claim 32): a
  building behaviour whose rule leaves the progress count outside 0 and 1, followed by an override
  to *attack* (for example a retaliation) while that value stands.
- Which rules of the structures specs set or clear a building's ready mark (claim 11). In
  particular, an unarmed building's own *guard* processing never sets it, so what makes a queued
  behaviour of an unarmed building current is not established here.
- The guard spot's reference point when the guard spot is a cell (F4): presumably the cell's centre
  at the cell's height.
- Which situations fill a vehicle's approach destinations, and which temporarily replace an
  actor's locomotor (claim 13).
- Which actors are marked as loaners (claim 15, row 5), and whether any fall-back other than the
  placement fall-back skips the land-type test of claim 15, row 4.
- The absent-key defaults of the guard stray distance and of the auto-deploy delay list: no engine
  default is known.
- Aircraft readiness (claim 11): which flight phases do not allow a change of behaviour. The
  aircraft spec owns the phases, and no scenario here tests aircraft readiness.
- How a vehicle accepts a scatter request and where it then goes (claims 19 and 28). No spec states
  it yet: S-0207 has no scatter rule. Apart from the gate on requests that are not forced
  (claim 19) and the paralysed refusal (claim 17), the vehicle's own conditions were only partly
  studied for this spec and are not stated here.
- The bunker rule of claim 17, the only identified sender of a scatter request with a threat point
  to a vehicle, and so the only way known to observe the paralysed refusal (scenario B6, part c).
  The structures specs own it and its exact conditions.
- Whether the recruitable property also keeps actors out of the computer player's teams (claim 20); scenario
  B20 tests base defence only.
- The SAM aiming tolerance of 8192 facing units and the power condition of claim 32; scenario B16
  does not test them.
- The vehicle *unload* processing (transports, deploying vehicles), *retreat*, *patrol* and the entry
  protocol of *enter*: claims 29, 30 and 34 point to other topics.
- The refused deploy of claim 27 (vehicles): which situations let a deploying vehicle in *hunt*
  reach a spot where its building may not then be placed, so that the refusal and its queued
  *guard* can be observed. The deploy specs own the placement test.
- The unload or deploy cycle that a vehicle's *move* and *patrol* processings restart (claims 22
  and 34): what the cycle is, its length, and so whether the restart can keep a moving vehicle from
  ever becoming ready to change. The deploy specs own the cycle.
- The *enter* processing (claim 29) can, in some cases where the actor has nothing left to enter,
  fall back to rest and start the resting behaviour at once before choosing its standard delay,
  which would then take the rate of the started behaviour (claim 6). The entry protocol's specs
  own those cases.
- The exact passive-search conditions for *move* and *harvest* (claim 33) and the draws the target
  search takes belong to the target-acquisition specs.
- *Area guard* of an actor that has no guard spot when the processing begins (claim 26): whether
  the own cell that it takes as its guard spot in that processing already counts for the leash and
  search table of the same processing, or whether that processing does none of the table (no
  search, no idle action). The two differ in the draws and the targeting-timer restart of that
  first processing; scenario B10 (g) checks only the guard spot afterwards.
- Claim 32's rows cover every answer category of the firing check (Terms), each in exactly one
  row. Which situations give which answer, in particular *illegal* and *impossible*, is not stated
  here; it belongs to the combat topic (the firing cycle, planned S-0301) and to S-0214.

## Questions for the project lead

- S-0107, claim 14 names the guard and area-guard behaviours as the trigger of idle actions. By
  claim 27 here, *hunt* without a target also attempts the idle action (and so do the processings
  of claims 24 and 26 in their own cases). Should S-0107, claim 14 gain *hunt* in its next
  revision?
- Claim 30's rule for the infantry type singled out by name now has the proposed switch
  `quirks.named_deploy_area_fire` (Edge cases and quirks), whose alternative singles out no type.
  Please confirm the default (on = original). A data-driven alternative, such as a type flag of
  our own format, would need a new field that the user's data does not have, so this draft does
  not propose one.
- S-0006, claim 5 lists the keys that reset on a layer. Claim 21's anti-air rate reset is another
  "reset to another key's value" rule; should S-0006's list gain a pointer to this spec in its next
  revision?
- One change to S-0208 that follows from claim 28 here, for its next revision: the list of
  scatter-request senders in S-0208 Terms could add claim 28 here, the scatter off a building in
  *capture*, *sabotage* and *eaten*, forced, not move-anywhere and without a threat point. The
  scatter from damage is request A, which that list already names, and S-0208 owns its
  statement for infantry.
- Vehicle scatter has no owner yet (Open questions). Should S-0207 take it over in its next
  revision: which requests a vehicle accepts, where it goes, and the paralysed refusal of claim 17?
